//! FIX_me_output_integrity (bughunt-1119 bh-me-I4, bh-me-I10): a failed write is
//! never success, and the manifest never lands on a plate preview. One test
//! function per TRACE row of design/FIX_me_output_integrity.md section 4, named
//! exactly as the row. Every run goes through `bash -c` in a fresh temp dir with
//! `umask 077`, stderr on a pipe, stdin `/dev/null`.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const MD1: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
const MK1_A: &str = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf";
const MK1_B: &str =
    "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";

const NOSPC: &str = "me: cannot write the output: No space left on device (os error 28)";
const EBADF: &str = "me: cannot write the output: Bad file descriptor (os error 9)";
const FSIZE0: &str = "ulimit -f 0; trap '' XFSZ; ";

fn me() -> PathBuf {
    assert_cmd::cargo::cargo_bin("me")
}

struct Run {
    rc: Option<i32>,
    stdout: Vec<u8>,
    stderr: String,
}

impl Run {
    fn last(&self) -> &str {
        self.stderr.lines().last().unwrap_or("")
    }
    fn has_card(&self) -> bool {
        self.stderr.lines().any(|l| l.starts_with("digest:"))
    }
    fn panicked(&self) -> bool {
        self.stderr.lines().any(|l| l.contains("panicked"))
    }
    fn dump(&self) -> String {
        format!("rc {:?}, stderr:\n{}", self.rc, self.stderr)
    }
}

/// Run `cmd` through `bash -c` in `dir` under `umask 077`; `$ME` is the binary.
fn sh(dir: &Path, cmd: &str, envs: &[(&str, &str)], unset: &[&str]) -> Run {
    let mut c = Command::new("bash");
    c.arg("-c")
        .arg(format!("umask 077; {cmd}"))
        .current_dir(dir)
        .env("ME", me())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in envs {
        c.env(k, v);
    }
    for k in unset {
        c.env_remove(k);
    }
    let o = c.output().unwrap();
    Run {
        rc: o.status.code(),
        stdout: o.stdout,
        stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

fn run(dir: &Path, cmd: &str) -> Run {
    sh(dir, cmd, &[], &["ME_PREVIEW_BIN"])
}

/// The fake sidecar of the triage rows, plus the p10 alias line.
fn write_fake(dir: &Path) -> PathBuf {
    let path = dir.join("fake-me-preview");
    let script = format!(
        "#!/bin/sh\n\
         if [ \"$1\" = \"--version\" ]; then echo 'me-preview {}'; exit 0; fi\n\
         if [ \"$1\" = \"render\" ]; then\n\
         out=\"\"; fmt=\"\"\n\
         while [ \"$#\" -gt 0 ]; do\n\
         if [ \"$1\" = \"--out\" ]; then out=\"$2\"; fi\n\
         if [ \"$1\" = \"--format\" ]; then fmt=\"$2\"; fi\n\
         shift\n\
         done\n\
         cat > /dev/null\n\
         if [ \"$fmt\" = \"png\" ]; then printf '\\211PNG\\r\\n\\032\\n' > \"$out\"; else printf '<svg/>' > \"$out\"; fi\n\
         if [ -n \"$FAKE_ALIAS\" ] && [ ! -e \"$FAKE_ALIAS\" ]; then ln -s \"$PWD/$out\" \"$FAKE_ALIAS\"; fi\n\
         echo 'mode text'\n\
         exit 0\n\
         fi\n\
         exit 1\n",
        env!("CARGO_PKG_VERSION")
    );
    fs::write(&path, script).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

struct Fx {
    dir: tempfile::TempDir,
    fake: PathBuf,
}

impl Fx {
    fn new() -> Fx {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        fs::write(p.join("m.txt"), format!("{MD1}\n")).unwrap();
        fs::write(p.join("b.txt"), format!("{MD1}\n{MK1_A}\n{MK1_B}\n")).unwrap();
        let fake = write_fake(p);
        let r = run(p, "\"$ME\" sysw pack --no-passphrase text:6869 --out c.bin");
        assert_eq!(r.rc, Some(0), "fixture pack: {}", r.dump());
        Fx { dir, fake }
    }
    fn p(&self) -> &Path {
        self.dir.path()
    }
    fn names(&self) -> Vec<String> {
        let mut v: Vec<String> = fs::read_dir(self.p())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }
    /// A bundle run with the fake sidecar.
    fn bundle(&self, args: &str, alias: Option<&str>) -> Run {
        let fake = self.fake.to_string_lossy().into_owned();
        let mut envs = vec![("ME_PREVIEW_BIN", fake.as_str())];
        if let Some(a) = alias {
            envs.push(("FAKE_ALIAS", a));
        }
        sh(
            self.p(),
            &format!("\"$ME\" bundle --in b.txt {args}"),
            &envs,
            &[],
        )
    }
}

fn dev_full_present() -> bool {
    Path::new("/dev/full").exists()
}

// ---------------------------------------------------------------------------
// w rows: stdout writes
// ---------------------------------------------------------------------------

#[test]
fn w1_pack_stdout_rejected_by_fsize_limit_is_not_success() {
    let fx = Fx::new();
    let r = run(
        fx.p(),
        &format!("{FSIZE0}\"$ME\" sysw pack --no-passphrase text:6869 > sink.bin"),
    );
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(
        r.last(),
        "me: cannot write the output: File too large (os error 27)",
        "{}",
        r.dump()
    );
    assert!(!r.has_card(), "{}", r.dump());
    assert!(!r.stderr.contains("re-print it with"), "{}", r.dump());
}

#[test]
fn w2_pack_stdout_full_exits_1() {
    if !dev_full_present() {
        return;
    }
    let fx = Fx::new();
    let r = run(
        fx.p(),
        "\"$ME\" sysw pack --no-passphrase text:6869 > /dev/full",
    );
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(r.last(), NOSPC, "{}", r.dump());
    assert!(!r.has_card(), "{}", r.dump());
}

#[test]
fn w3_pack_read_only_stdout_exits_1() {
    let fx = Fx::new();
    let r = run(
        fx.p(),
        "\"$ME\" sysw pack --no-passphrase text:6869 1</dev/null",
    );
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(r.last(), EBADF, "{}", r.dump());
    assert!(!r.has_card(), "{}", r.dump());
}

#[test]
fn w4_converter_raw_stdout_full_exits_1() {
    if !dev_full_present() {
        return;
    }
    let fx = Fx::new();
    let r = run(fx.p(), "\"$ME\" --in m.txt --stdout > /dev/full");
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(r.stderr, format!("{NOSPC}\n"), "{}", r.dump());
}

#[test]
fn w5_converter_hex_full_exits_1() {
    if !dev_full_present() {
        return;
    }
    let fx = Fx::new();
    let r = run(fx.p(), "\"$ME\" --in m.txt --hex > /dev/full");
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(r.stderr, format!("{NOSPC}\n"), "{}", r.dump());
}

#[test]
fn w6_converter_base64_full_exits_1() {
    if !dev_full_present() {
        return;
    }
    let fx = Fx::new();
    let r = run(fx.p(), "\"$ME\" --in m.txt --base64 > /dev/full");
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(r.stderr, format!("{NOSPC}\n"), "{}", r.dump());
}

#[test]
fn w7_converter_hex_read_only_stdout_exits_1() {
    let fx = Fx::new();
    let r = run(fx.p(), "\"$ME\" --in m.txt --hex 1</dev/null");
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(r.stderr, format!("{EBADF}\n"), "{}", r.dump());
}

#[test]
fn w8_bundle_stdout_full_exits_1_without_checklist() {
    if !dev_full_present() {
        return;
    }
    let fx = Fx::new();
    let r = run(fx.p(), "\"$ME\" bundle --in b.txt > /dev/full");
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(r.last(), NOSPC, "{}", r.dump());
    assert!(!r.panicked(), "{}", r.dump());
    assert!(!r.stderr.contains("backup needs"), "{}", r.dump());
}

#[test]
fn w9_show_stdout_full_exits_1() {
    if !dev_full_present() {
        return;
    }
    let fx = Fx::new();
    let r = run(fx.p(), "\"$ME\" sysw show c.bin > /dev/full");
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(r.stderr, format!("{NOSPC}\n"), "{}", r.dump());
}

#[test]
fn w10_hash_stdout_full_exits_1() {
    if !dev_full_present() {
        return;
    }
    let fx = Fx::new();
    let r = run(
        fx.p(),
        &format!("\"$ME\" hash --unsealed {MD1} > /dev/full"),
    );
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(r.stderr, format!("{NOSPC}\n"), "{}", r.dump());
}

#[test]
fn w11_wipe_stdout_full_exits_1() {
    if !dev_full_present() {
        return;
    }
    let fx = Fx::new();
    let r = run(fx.p(), "\"$ME\" sysw wipe > /dev/full");
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(r.stderr, format!("{NOSPC}\n"), "{}", r.dump());
}

#[test]
fn w12_help_stdout_full_exits_1() {
    if !dev_full_present() {
        return;
    }
    let fx = Fx::new();
    let r = run(fx.p(), "\"$ME\" --help > /dev/full");
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(r.stderr, format!("{NOSPC}\n"), "{}", r.dump());
}

#[test]
fn w13_version_stdout_full_exits_1() {
    if !dev_full_present() {
        return;
    }
    let fx = Fx::new();
    let r = run(fx.p(), "\"$ME\" --version > /dev/full");
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(r.stderr, format!("{NOSPC}\n"), "{}", r.dump());
}

#[test]
fn w14_show_closed_pipe_keeps_exit_0() {
    let fx = Fx::new();
    let r = run(
        fx.p(),
        "(sleep 0.5; exec \"$ME\" sysw show c.bin) | true; exit ${PIPESTATUS[0]}",
    );
    assert_eq!(r.rc, Some(0), "{}", r.dump());
    assert!(!r.panicked(), "{}", r.dump());
    assert!(!r.stderr.contains("cannot write"), "{}", r.dump());
}

#[test]
fn w15_converter_raw_closed_pipe_keeps_exit_0() {
    let fx = Fx::new();
    let r = run(
        fx.p(),
        "(sleep 0.5; exec \"$ME\" --in m.txt --stdout) | true; exit ${PIPESTATUS[0]}",
    );
    assert_eq!(r.rc, Some(0), "{}", r.dump());
    assert_eq!(r.stderr, "", "{}", r.dump());
}

#[test]
fn w16_pack_stdout_success_is_unchanged() {
    let fx = Fx::new();
    let r = run(
        fx.p(),
        "\"$ME\" sysw pack --no-passphrase text:6869 > ok.bin",
    );
    assert_eq!(r.rc, Some(0), "{}", r.dump());
    let ok = fs::read(fx.p().join("ok.bin")).unwrap();
    assert_eq!(ok.len(), 61);
    assert_eq!(ok, fs::read(fx.p().join("c.bin")).unwrap());
    assert!(r.has_card(), "{}", r.dump());
}

// ---------------------------------------------------------------------------
// o rows: --out failure texts
// ---------------------------------------------------------------------------

#[test]
fn o1_pack_failed_overwrite_says_unchanged() {
    let fx = Fx::new();
    fs::write(fx.p().join("F.bin"), "OLD BACKUP\n").unwrap();
    let before = fx.names();
    let r = run(
        fx.p(),
        &format!("{FSIZE0}\"$ME\" sysw pack --no-passphrase text:6869 --out F.bin"),
    );
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(
        r.last(),
        "me: cannot write the output: F.bin: File too large (os error 27). F.bin is unchanged.",
        "{}",
        r.dump()
    );
    assert_eq!(fs::read(fx.p().join("F.bin")).unwrap(), b"OLD BACKUP\n");
    assert_eq!(fx.names(), before, "no temporary may be left behind");
    assert!(!r.has_card(), "{}", r.dump());
}

#[test]
fn o2_pack_missing_dir_says_nothing_was_written() {
    let fx = Fx::new();
    let r = run(
        fx.p(),
        "\"$ME\" sysw pack --no-passphrase text:6869 --out nodir/x.bin",
    );
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(
        r.last(),
        "me: cannot write the output: nodir/x.bin: No such file or directory (os error 2). Nothing was written.",
        "{}",
        r.dump()
    );
    assert!(!r.has_card(), "{}", r.dump());
}

#[test]
fn o3_pack_failed_new_file_leaves_nothing() {
    let fx = Fx::new();
    let before = fx.names();
    let r = run(
        fx.p(),
        &format!("{FSIZE0}\"$ME\" sysw pack --no-passphrase text:6869 --out N.bin"),
    );
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(
        r.last(),
        "me: cannot write the output: N.bin: File too large (os error 27). Nothing was written.",
        "{}",
        r.dump()
    );
    assert_eq!(fx.names(), before, "the dir holds no N.bin*");
}

#[test]
fn o4_pack_read_only_dir_says_unchanged() {
    let id = Command::new("id").arg("-u").output().unwrap();
    if String::from_utf8_lossy(&id.stdout).trim() == "0" {
        return;
    }
    let fx = Fx::new();
    let ro = fx.p().join("ro");
    fs::create_dir(&ro).unwrap();
    fs::write(ro.join("F"), "OLD\n").unwrap();
    fs::set_permissions(&ro, fs::Permissions::from_mode(0o500)).unwrap();
    let r = run(
        fx.p(),
        "\"$ME\" sysw pack --no-passphrase text:6869 --out ro/F",
    );
    let kept = fs::read(ro.join("F")).unwrap();
    fs::set_permissions(&ro, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(
        r.last(),
        "me: cannot write the output: ro/F: Permission denied (os error 13). ro/F is unchanged.",
        "{}",
        r.dump()
    );
    assert_eq!(kept, b"OLD\n");
}

#[test]
fn o5_converter_out_missing_dir() {
    let fx = Fx::new();
    let r = run(fx.p(), "\"$ME\" --in m.txt --out nodir/x.ndef");
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(
        r.stderr,
        "me: cannot write the output: nodir/x.ndef: No such file or directory (os error 2). Nothing was written.\n",
        "{}",
        r.dump()
    );
}

#[test]
fn o6_bundle_manifest_missing_dir() {
    let fx = Fx::new();
    let r = run(fx.p(), "\"$ME\" bundle --in b.txt --manifest nodir/x.json");
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(
        r.last(),
        "me: cannot write the output: nodir/x.json: No such file or directory (os error 2). Nothing was written.",
        "{}",
        r.dump()
    );
    assert!(r.stdout.is_empty());
    assert!(!r.stderr.contains("backup needs"), "{}", r.dump());
}

#[test]
fn o7_stdio_alias_onto_a_file_names_no_state() {
    let fx = Fx::new();
    fs::write(fx.p().join("s.txt"), "OLD BACKUP\n").unwrap();
    let r = run(
        fx.p(),
        &format!("{FSIZE0}\"$ME\" sysw pack --no-passphrase text:6869 --out /dev/stdout >> s.txt"),
    );
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(
        r.last(),
        "me: cannot write the output: /dev/stdout: File too large (os error 27)",
        "{}",
        r.dump()
    );
    assert_eq!(fs::read(fx.p().join("s.txt")).unwrap(), b"OLD BACKUP\n");
}

#[test]
fn o8_seal_out_missing_dir() {
    let fx = Fx::new();
    let r = run(
        fx.p(),
        "\"$ME\" seal md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3 --out nodir/x.uf2",
    );
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert_eq!(
        r.last(),
        "me: cannot write the output: nodir/x.uf2: No such file or directory (os error 2). Nothing was written.",
        "{}",
        r.dump()
    );
    assert!(!r.stderr.contains("wrote"), "{}", r.dump());
}

#[test]
fn o9_pack_out_device_names_no_state() {
    if !dev_full_present() {
        return;
    }
    let fx = Fx::new();
    let r = run(
        fx.p(),
        "\"$ME\" sysw pack --no-passphrase text:6869 --out /dev/full",
    );
    assert_eq!(r.rc, Some(1), "{}", r.dump());
    assert!(
        r.last()
            .starts_with("me: cannot write the output: /dev/full: ")
            && r.last().ends_with(')'),
        "{}",
        r.dump()
    );
    assert!(!r.last().contains("unchanged"), "{}", r.dump());
    assert!(!r.last().contains("Nothing was written"), "{}", r.dump());
}

// ---------------------------------------------------------------------------
// p rows: the manifest is never a plate preview
// ---------------------------------------------------------------------------

fn p1_text(m: &str, p: &str) -> String {
    format!("me: --manifest {m} is the same file as the plate preview {p}; write the manifest to a new name")
}

/// The shared refusal assertions of p1-p7.
fn assert_p1_refused(r: &Run, m: &str, p: &str) {
    assert_eq!(r.rc, Some(2), "{}", r.dump());
    assert_eq!(r.last(), p1_text(m, p), "{}", r.dump());
    for bad in [
        "rendered plate",
        "wrote manifest",
        "preview skipped",
        "backup needs",
    ] {
        assert!(
            !r.stderr.lines().any(|l| l.contains(bad)),
            "stderr line contains {bad:?}: {}",
            r.dump()
        );
    }
    assert!(r.stdout.is_empty());
}

fn dir_is_empty(p: &Path) -> bool {
    fs::read_dir(p).unwrap().next().is_none()
}

#[test]
fn p1_manifest_is_svg_plate_refused() {
    let fx = Fx::new();
    fs::create_dir(fx.p().join("p")).unwrap();
    let r = fx.bundle("--preview p --manifest p/plate-1.svg", None);
    assert_p1_refused(&r, "p/plate-1.svg", "p/plate-1.svg");
    assert!(dir_is_empty(&fx.p().join("p")));
}

#[test]
fn p2_manifest_is_png_plate_refused() {
    let fx = Fx::new();
    fs::create_dir(fx.p().join("p")).unwrap();
    let r = fx.bundle("--preview p --png --manifest p/plate-1.png", None);
    assert_p1_refused(&r, "p/plate-1.png", "p/plate-1.png");
    assert!(dir_is_empty(&fx.p().join("p")));
}

#[test]
fn p3_manifest_dot_slash_spelling_refused() {
    let fx = Fx::new();
    fs::create_dir(fx.p().join("p")).unwrap();
    let r = fx.bundle("--preview p --manifest ./p/./plate-1.svg", None);
    assert_p1_refused(&r, "./p/./plate-1.svg", "p/plate-1.svg");
    assert!(dir_is_empty(&fx.p().join("p")));
}

#[test]
fn p4_manifest_symlink_to_plate_refused() {
    let fx = Fx::new();
    fs::create_dir(fx.p().join("p")).unwrap();
    std::os::unix::fs::symlink("p/plate-2.svg", fx.p().join("L")).unwrap();
    let r = fx.bundle("--preview p --manifest L", None);
    assert_p1_refused(&r, "L", "p/plate-2.svg");
    assert!(dir_is_empty(&fx.p().join("p")));
    let md = fs::symlink_metadata(fx.p().join("L")).unwrap();
    assert!(md.file_type().is_symlink() && !fx.p().join("L").exists());
}

#[test]
fn p5_manifest_through_dir_symlink_refused() {
    let fx = Fx::new();
    fs::create_dir(fx.p().join("p")).unwrap();
    std::os::unix::fs::symlink("p", fx.p().join("q")).unwrap();
    let r = fx.bundle("--preview p --manifest q/plate-3.svg", None);
    assert_p1_refused(&r, "q/plate-3.svg", "p/plate-3.svg");
    assert!(dir_is_empty(&fx.p().join("p")));
}

#[test]
fn p6_manifest_hard_link_at_plate_name_refused() {
    let fx = Fx::new();
    fs::create_dir(fx.p().join("p")).unwrap();
    fs::write(fx.p().join("m.json"), "X").unwrap();
    fs::hard_link(fx.p().join("m.json"), fx.p().join("p/plate-1.svg")).unwrap();
    let r = fx.bundle("--preview p --manifest m.json", None);
    assert_p1_refused(&r, "m.json", "p/plate-1.svg");
    assert_eq!(fs::read(fx.p().join("m.json")).unwrap(), b"X");
}

#[test]
fn p7_manifest_on_plate_refused_without_sidecar() {
    if assert_cmd::cargo::cargo_bin("me")
        .with_file_name("me-preview")
        .exists()
    {
        return;
    }
    let fx = Fx::new();
    fs::create_dir(fx.p().join("p9")).unwrap();
    let r = run(
        fx.p(),
        "\"$ME\" bundle --in b.txt --preview p9 --manifest p9/plate-1.svg",
    );
    assert_p1_refused(&r, "p9/plate-1.svg", "p9/plate-1.svg");
    assert!(dir_is_empty(&fx.p().join("p9")));
}

#[test]
fn p8_distinct_manifest_still_written() {
    let fx = Fx::new();
    fs::create_dir(fx.p().join("p")).unwrap();
    let r = fx.bundle("--preview p --manifest m.json", None);
    assert_eq!(r.rc, Some(0), "{}", r.dump());
    for n in 1..=3 {
        assert_eq!(
            fs::read(fx.p().join(format!("p/plate-{n}.svg"))).unwrap(),
            b"<svg/>"
        );
    }
    let j = fs::read_to_string(fx.p().join("m.json")).unwrap();
    assert!(j.starts_with('{'), "{j}");
}

#[test]
fn p9_unplanned_plate_name_still_written() {
    let fx = Fx::new();
    fs::create_dir(fx.p().join("p")).unwrap();
    let r = fx.bundle("--preview p --manifest p/plate-9.svg", None);
    assert_eq!(r.rc, Some(0), "{}", r.dump());
    for n in 1..=3 {
        assert_eq!(
            fs::read(fx.p().join(format!("p/plate-{n}.svg"))).unwrap(),
            b"<svg/>"
        );
    }
    let j = fs::read_to_string(fx.p().join("p/plate-9.svg")).unwrap();
    assert!(j.starts_with('{'), "{j}");
}

#[test]
fn p10_manifest_alias_made_during_render_refused() {
    let fx = Fx::new();
    fs::create_dir(fx.p().join("p")).unwrap();
    let r = fx.bundle("--preview p --manifest L", Some("L"));
    assert_eq!(r.rc, Some(2), "{}", r.dump());
    assert_eq!(r.last(), p1_text("L", "p/plate-1.svg"), "{}", r.dump());
    assert_eq!(fs::read(fx.p().join("p/plate-1.svg")).unwrap(), b"<svg/>");
    assert!(r.stdout.is_empty());
    assert!(!r.stderr.contains("wrote manifest"), "{}", r.dump());
    assert!(!r.stderr.contains("backup needs"), "{}", r.dump());
}
