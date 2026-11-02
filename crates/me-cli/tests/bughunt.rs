//! Bug-hunt triage rows (me, 2026-10-31 hunt at a757199). Each asserts the
//! CORRECT behaviour; a red row is a confirmed bug.
#![cfg(unix)]

use assert_cmd::Command;
use mnemonic_engrave::{preview, seal, sysw};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

fn me() -> Command {
    Command::cargo_bin("me").unwrap()
}

/// Bug #1: `write_private` (reached by `me sysw pack --out`) truncates an
/// existing file before a chmod that can fail. The file is made writable by
/// but NOT owned by the caller inside a user namespace (subuid-mapped owner),
/// no sudo. A failed write must leave the old contents alone.
#[test]
fn bughunt_me_01_write_private_keeps_old_file_when_chmod_fails() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    let f = dir.path().join("old.bin");
    let bin = assert_cmd::cargo::cargo_bin("me");
    let script = format!(
        "set -e; cd {d}; echo OLD-ARTIFACT > old.bin; chown 1 old.bin; chmod 666 old.bin; \
         setpriv --reuid=2 --regid=2 --clear-groups {me} sysw pack --no-passphrase --out old.bin text:6869; echo rc=$? ",
        d = dir.path().display(),
        me = bin.display()
    );
    // `sh -c` with set -e: the me exit status is captured by `|| echo`.
    let script = script.replace(
        "--out old.bin text:6869; echo rc=$? ",
        "--out old.bin text:6869 >/dev/null 2>&1 || echo me-failed",
    );
    let out = std::process::Command::new("unshare")
        .args(["-U", "--map-auto", "--map-root-user", "sh", "-c", &script])
        .output()
        .expect("unshare must run (setup)");
    assert!(
        out.status.success(),
        "setup failed (needs unshare + subuid): {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("me-failed"),
        "premise: me must fail on the chmod: {stdout}"
    );
    let left = std::fs::read(&f).unwrap();
    assert_eq!(
        left, b"OLD-ARTIFACT\n",
        "the write failed (chmod EPERM) yet the old file was destroyed"
    );
}

/// Bug #2: a `plate-1.svg` from a previous run is accepted as this run's
/// render when the sidecar exits 0 and writes nothing (F9 must refuse).
#[test]
fn bughunt_me_02_stale_preview_is_not_this_runs_render() {
    let dir = tempfile::tempdir().unwrap();
    let sidecar = dir.path().join("fake-sidecar");
    std::fs::write(&sidecar, "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(&sidecar, std::fs::Permissions::from_mode(0o755)).unwrap();
    let out = dir.path().join("plate-1.svg");
    std::fs::write(&out, "<svg>STALE FROM A PREVIOUS RUN</svg>").unwrap();
    let mut r = None;
    for _ in 0..500 {
        let x = preview::render_plate(&sidecar, "md1other", dir.path(), 1, false);
        if matches!(&x, Err(preview::PreviewError::Spawn(e)) if e.kind() == std::io::ErrorKind::ExecutableFileBusy)
        {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        r = Some(x);
        break;
    }
    let r = r.expect("ETXTBSY never cleared");
    assert!(
        matches!(r, Err(preview::PreviewError::EmptyOutput { .. })),
        "a sidecar that wrote nothing must be refused, got {r:?}"
    );
}

/// Bug #3: `pack` with a passphrase and no secret records must not emit a
/// container whose header says unsealed but which carries KDF fields and a
/// trailing tag. Either refuse, or emit a well-formed 61-byte unsealed blob.
#[test]
fn bughunt_me_03_pack_passphrase_without_secrets_is_well_formed() {
    let r = sysw::pack(
        vec!["text:6869".to_string()],
        Some("abandon ability able about above"),
        sysw::wire::MIN_ITERATIONS,
    );
    let blob = match r {
        Ok(b) => b,
        Err(e) => {
            eprintln!("pack refused: {e:?}");
            return; // refusing is acceptable
        }
    };
    eprintln!("pack emitted {} bytes", blob.len());
    let h = sysw::wire::Header::parse(&blob).expect("emitted blob must parse");
    assert_eq!(
        blob.len(),
        h.total_len(),
        "blob has bytes past the header's declared end (len {}, declared {})",
        blob.len(),
        h.total_len()
    );
}

/// Bug #4: `sysw pack --in FILE` plus argv records silently drops the argv
/// records at exit 0. Refuse, or pack both. (Open: refuse vs merge.)
#[test]
fn bughunt_me_04_pack_in_file_plus_argv_does_not_drop_argv() {
    let dir = tempfile::tempdir().unwrap();
    let recs = dir.path().join("recs.txt");
    std::fs::write(&recs, "text:6162\n").unwrap();
    let out = dir.path().join("p.bin");
    let a = me()
        .args(["sysw", "pack", "--no-passphrase", "--in"])
        .arg(&recs)
        .arg("--out")
        .arg(&out)
        .arg("text:6869")
        .output()
        .unwrap();
    if !a.status.success() {
        return; // refusing is correct
    }
    let blob = std::fs::read(&out).unwrap();
    let has = |needle: &[u8]| blob.windows(needle.len()).any(|w| w == needle);
    assert!(
        has(b"text:6162") && has(b"text:6869"),
        "exit 0 but the argv record was silently dropped: {:?}\nstderr: {}",
        String::from_utf8_lossy(&blob[52.min(blob.len())..]),
        String::from_utf8_lossy(&a.stderr)
    );
}

/// L5: `to_uf2` of a >2 MiB blob must not emit a block addressed at or past
/// 0x11000000 (wraps to 0x10000000 and destroys the firmware). Open: whether
/// the refusal is a panic or an empty result; either passes here.
#[test]
fn bughunt_me_05_uf2_oversize_blob_never_addresses_past_flash_end() {
    let blob = vec![0xA5u8; 3 * 1024 * 1024];
    let r = std::panic::catch_unwind(|| seal::uf2::to_uf2(&blob));
    let Ok(uf2) = r else { return };
    for (i, b) in uf2.chunks(512).enumerate() {
        let addr = u32::from_le_bytes(b[12..16].try_into().unwrap());
        assert!(
            addr < 0x1100_0000,
            "block {i} targets {addr:#x}, past the end of flash"
        );
    }
}

/// L6: a sealed container whose header carries an absurd iteration count must
/// be refused quickly by `open`, not run.
#[test]
fn bughunt_me_06_open_refuses_absurd_iterations_quickly() {
    let pass = "abandon ability able about above";
    let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let mut blob = sysw::pack(
        vec![seed.to_string(), "text:6869".to_string()],
        Some(pass),
        sysw::wire::MIN_ITERATIONS,
    )
    .expect("pack");
    blob[12..16].copy_from_slice(&u32::MAX.to_be_bytes());
    let (tx, rx) = mpsc::channel();
    let b = blob.clone();
    std::thread::spawn(move || {
        let r = sysw::open(&b, Some(pass));
        let _ = tx.send(r.is_err());
    });
    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(refused) => assert!(refused, "absurd iterations were accepted"),
        Err(_) => panic!("open ran the KDF for u32::MAX iterations instead of refusing"),
    }
}

const MT_EVEN: [&str; 6] = [
    "mt1p9h8jqq9qqqqgqqqqqqqyqherdfykhhpey6z2cvafak8804qd7g0dl6v8ex9wr2cvky023skwkeud2229sax",
    "mt1p9h8jqq9qqphgdqqqqqqqq0mllllupyqj6vqqqqqqqqzcqpfsw7ph2rt5w54kt768636cls8zxg0najlzunp",
    "mt1p9h8jqq9qqzj8yqpnzw4vl2rwffqyqqqqqkqq282yyhc2vavd20hvk94pz39hts3u5s9a0qd8pwskxfl7ju5",
    "mt1p9h8jqq9qqrqfrnq3qzyp77h37cnxzvwutegzmzy5zrrrfvrpykdfsckvk03dcq6rcjtvlsfcglv7zx43yaz",
    "mt1p9h8jqq9qqylgpzqmhcwhuupdvnrc82rncvzzdahpgjsdwgu52jd7vmxsve9x3w5ujeqyssuvddxvwqze4ve",
    "mt1p9h8jqq9qq9qdcc7h75twfxyf340c4sgqzhfdq6xtgt7zhxngpwa049l0z59l6jqcqqqqqq5k5y2ye5nv8yf",
];

/// L7: the missing-plates sentence builder must not panic on holder input.
/// Real path: `sysw pack` of an mt1 set with strings missing (1, 2, 5 of 6
/// missing, and the complete set). None may panic (exit 101).
#[test]
fn bughunt_me_07_pack_mt_set_missing_strings_never_panics() {
    let _ = Path::new("");
    for keep in [
        vec![0usize, 1, 2, 3, 4, 5],
        vec![0, 1, 2, 3, 4],
        vec![0, 1, 2, 3],
        vec![0],
        vec![1, 2, 3, 4, 5],
    ] {
        let input: Vec<&str> = keep.iter().map(|&i| MT_EVEN[i]).collect();
        let a = me()
            .args(["sysw", "pack", "--no-passphrase"])
            .write_stdin(input.join("\n"))
            .output()
            .unwrap();
        assert_ne!(
            a.status.code(),
            Some(101),
            "panic for kept {keep:?}: {}",
            String::from_utf8_lossy(&a.stderr)
        );
    }
}

/// FIX R2 (#2): a `plate-1.svg` that `render_plate` cannot remove (here a
/// non-empty directory) is reported as `PreviewError::Spawn`, displayed as
/// `cannot run me-preview: <io error>`, before the sidecar runs.
#[test]
fn bughunt_me_02b_unremovable_stale_preview_is_a_spawn_error() {
    let dir = tempfile::tempdir().unwrap();
    let sidecar = dir.path().join("fake-sidecar");
    std::fs::write(&sidecar, "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(&sidecar, std::fs::Permissions::from_mode(0o755)).unwrap();
    let stale = dir.path().join("plate-1.svg");
    std::fs::create_dir(&stale).unwrap();
    std::fs::write(stale.join("keep"), "x").unwrap();
    let mut r = None;
    for _ in 0..500 {
        let x = preview::render_plate(&sidecar, "md1other", dir.path(), 1, false);
        if matches!(&x, Err(preview::PreviewError::Spawn(e)) if e.kind() == std::io::ErrorKind::ExecutableFileBusy)
        {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        r = Some(x);
        break;
    }
    let r = r.expect("ETXTBSY never cleared");
    let Err(e @ preview::PreviewError::Spawn(_)) = r else {
        panic!("an unremovable stale plate must be a Spawn error, got {r:?}");
    };
    assert!(
        e.to_string().starts_with("cannot run me-preview: "),
        "display: {e}"
    );
    assert!(
        stale.join("keep").exists(),
        "the directory must be left alone"
    );
}

/// FIX R3 (#3): with no secret records, `pack` with a passphrase emits the
/// same unsealed container as `pack` without one.
#[test]
fn bughunt_me_03b_pack_passphrase_without_secrets_is_the_unsealed_form() {
    let recs = || vec!["text:6869".to_string()];
    let sealed = sysw::pack(
        recs(),
        Some("abandon ability able about above"),
        sysw::wire::MIN_ITERATIONS,
    )
    .expect("a valid passphrase with no secrets must pack");
    let plain = sysw::pack(recs(), None, sysw::wire::MIN_ITERATIONS).expect("pack None");
    assert_eq!(
        sealed,
        plain,
        "no secrets: expected the unsealed form ({} bytes), got {} bytes",
        plain.len(),
        sealed.len()
    );
}

/// FIX R1 (#4): records on argv together with `--in` are refused with the
/// seal refusal's text at exit 2, and nothing is written.
#[test]
fn bughunt_me_04b_pack_in_plus_argv_refusal_text_and_exit() {
    let dir = tempfile::tempdir().unwrap();
    let recs = dir.path().join("recs.txt");
    std::fs::write(&recs, "text:6162\n").unwrap();
    let out = dir.path().join("p.bin");
    let a = me()
        .args(["sysw", "pack", "--no-passphrase", "--in"])
        .arg(&recs)
        .arg("--out")
        .arg(&out)
        .arg("text:6869")
        .output()
        .unwrap();
    assert_eq!(
        (
            a.status.code(),
            String::from_utf8_lossy(&a.stderr).into_owned()
        ),
        (
            Some(2),
            "me: pass records on argv OR via --in, not both\n".to_string()
        ),
        "refusal"
    );
    assert!(a.stdout.is_empty(), "stdout must be empty");
    assert!(!out.exists(), "nothing may be written");
}

/// FIX R4 (L5): `to_uf2` accepts a blob of exactly `seal::wire::REGION_LEN`
/// (65536) bytes and panics, with the ruled text, one byte over.
#[test]
fn bughunt_me_05b_uf2_cap_is_the_64k_seal_region() {
    let cap = seal::wire::REGION_LEN as usize;
    assert_eq!(cap, 65_536);
    let uf2 = seal::uf2::to_uf2(&vec![0xA5u8; cap]);
    assert_eq!(uf2.len(), cap / 256 * 512, "exactly 64 KiB is accepted");
    let r = std::panic::catch_unwind(|| seal::uf2::to_uf2(&vec![0xA5u8; cap + 1]));
    let Err(p) = r else {
        panic!("a {}-byte blob was accepted", cap + 1);
    };
    let msg = p
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    assert_eq!(
        msg,
        "seal blob 65537 bytes exceeds the 65536-byte seal region"
    );
}

/// FIX R1 + architect S1: with argv AND `--in`, the argv bearer/secret guard
/// still runs first (exit 3, its own refusal), so "not both" never pre-empts
/// a refusal about material already in the shell's history.
#[test]
fn bughunt_me_04c_argv_bearer_guard_precedes_not_both() {
    let dir = tempfile::tempdir().unwrap();
    let recs = dir.path().join("recs.txt");
    std::fs::write(&recs, "text:6162\n").unwrap();
    let out = dir.path().join("p.bin");
    let a = me()
        .args(["sysw", "pack", "--no-passphrase", "--in"])
        .arg(&recs)
        .arg("--out")
        .arg(&out)
        .arg("tx:00")
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&a.stderr).into_owned();
    assert_eq!(a.status.code(), Some(3), "stderr:\n{err}");
    assert!(err.contains("is a `tx:` record on ARGV"), "stderr:\n{err}");
    assert!(!err.contains("not both"), "stderr:\n{err}");
    assert!(!out.exists(), "nothing may be written");
}

/// Architect S1 + S3: the same order holds with `--as`: the bearer refusal
/// (exit 3) precedes §5.1's shape refusal.
#[test]
fn bughunt_me_04d_as_argv_bearer_guard_precedes_single_document_with_in() {
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("document.txt");
    std::fs::write(&doc, "text:6162\n").unwrap();
    let a = me()
        .args(["sysw", "pack", "--no-passphrase", "--as", "md1", "--in"])
        .arg(&doc)
        .arg("tx:00")
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&a.stderr).into_owned();
    assert_eq!(a.status.code(), Some(3), "stderr:\n{err}");
    assert!(err.contains("is a `tx:` record on ARGV"), "stderr:\n{err}");
    assert!(
        !err.contains("--as packs exactly one descriptor per invocation."),
        "stderr:\n{err}"
    );
}

/// FIX R5 (#1): `--out` to a FIFO must keep working once truncation moves
/// from `truncate(true)` at open to `set_len(0)` (a FIFO has no length).
#[test]
fn bughunt_me_01b_pack_out_to_a_fifo_still_writes() {
    let dir = tempfile::tempdir().unwrap();
    let ff = dir.path().join("ff");
    let st = std::process::Command::new("mkfifo").arg(&ff).status().unwrap();
    assert!(st.success(), "mkfifo (setup)");
    let reader = {
        let ff = ff.clone();
        std::thread::spawn(move || std::fs::read(ff).unwrap())
    };
    let out = me()
        .args(["sysw", "pack", "--no-passphrase", "--out"])
        .arg(&ff)
        .arg("text:6869")
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "pack to a FIFO must succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let fifo_bytes = reader.join().unwrap();
    let reg = dir.path().join("reg.bin");
    let out = me()
        .args(["sysw", "pack", "--no-passphrase", "--out"])
        .arg(&reg)
        .arg("text:6869")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(fifo_bytes, std::fs::read(&reg).unwrap());
}
