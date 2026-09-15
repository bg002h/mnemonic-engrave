//! Phase 3 of SPEC_hashlock_kinds: `me`'s side of the §6 record grammar, both
//! directions, and the §8.2.3 orphan check across kinds.
//!
//! WHY THIS FILE EXISTS. `me sysw pack` read every `hash:` record as sha256 and
//! derived only `sha256(X)` when checking a `phrase:` record against it. With
//! four kinds that is wrong in both directions: a correct `phrase:` + `hash256:`
//! payload WARNED that the phrase matched nothing, and `me sysw show` labelled
//! every record "sha256 hashlock" whatever it was.

use assert_cmd::Command;
use std::io::Write;

/// The phase-2 KAT row: `correct horse battery staple` under the hardened
/// method. These are the same digests `ms hashlock --kind` prints, and the same
/// ones `md compose`'s vectors carry — one preimage, three repos.
const PHRASE_HEX: &str =
    "68617264656e65642c636f727265637420686f727365206261747465727920737461706c65";
const D_SHA256: &str = "3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12";
const D_HASH256: &str = "98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488";
const D_RIPEMD160: &str = "09e7bb5051d89788fb4e4b374126721dbcc2946b";
const D_HASH160: &str = "b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd";

/// Records go through a FILE, never argv: the argv guard refuses a `phrase:`
/// record before the command line is parsed, because argv is public.
fn pack(records: &[&str], extra: &[&str]) -> (bool, String) {
    let dir = tempfile::tempdir().unwrap();
    let recs = dir.path().join("r.txt");
    let out = dir.path().join("p.bin");
    let mut f = std::fs::File::create(&recs).unwrap();
    for r in records {
        writeln!(f, "{r}").unwrap();
    }
    drop(f);
    let mut args: Vec<String> = vec![
        "sysw".into(),
        "pack".into(),
        "--in".into(),
        recs.display().to_string(),
        "--out".into(),
        out.display().to_string(),
    ];
    args.extend(extra.iter().map(|s| s.to_string()));
    let o = Command::cargo_bin("me")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    (
        o.status.success(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

fn show(records: &[&str]) -> String {
    let dir = tempfile::tempdir().unwrap();
    let recs = dir.path().join("r.txt");
    let out = dir.path().join("p.bin");
    let mut f = std::fs::File::create(&recs).unwrap();
    for r in records {
        writeln!(f, "{r}").unwrap();
    }
    drop(f);
    Command::cargo_bin("me")
        .unwrap()
        .args([
            "sysw",
            "pack",
            "--in",
            &recs.display().to_string(),
            "--out",
            &out.display().to_string(),
        ])
        .output()
        .unwrap();
    let o = Command::cargo_bin("me")
        .unwrap()
        .args(["sysw", "show", &out.display().to_string()])
        .output()
        .unwrap();
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// §6 both directions: every kind packs, and `me sysw show` names the kind it
/// packed rather than calling everything sha256.
#[test]
fn every_kind_packs_and_show_names_the_kind_it_packed() {
    for (rec, kind, digest) in [
        (format!("hash:{D_SHA256}"), "sha256", D_SHA256),
        (format!("hash:hash256:{D_HASH256}"), "hash256", D_HASH256),
        (
            format!("hash:ripemd160:{D_RIPEMD160}"),
            "ripemd160",
            D_RIPEMD160,
        ),
        (format!("hash:hash160:{D_HASH160}"), "hash160", D_HASH160),
    ] {
        let (ok, se) = pack(&[&rec], &[]);
        assert!(ok, "{kind}: refused:\n{se}");
        let out = show(&[&rec]);
        assert!(
            out.contains(&format!("{kind} hashlock (hash:)")),
            "{kind}: show does not name the kind:\n{out}"
        );
        // ...and the digest is displayed at ITS width, not sliced at [56..].
        let (first8, last8) = (&digest[..8], &digest[digest.len() - 8..]);
        assert!(
            out.contains(&format!("{first8}..{last8}")),
            "{kind}: show mangles a {}-hex digest:\n{out}",
            digest.len()
        );
    }
}

/// §8.2.3's orphan check, ACROSS KINDS. The check derived only `sha256(X)`, so
/// a correct `phrase:` + `hash256:` payload warned that the phrase matched
/// nothing — a warning on a correct payload, which is how a warning stops being
/// read.
#[test]
fn a_phrase_matching_a_non_sha256_record_does_not_warn() {
    for (kind, digest) in [
        ("sha256", D_SHA256),
        ("hash256", D_HASH256),
        ("ripemd160", D_RIPEMD160),
        ("hash160", D_HASH160),
    ] {
        let rec = if kind == "sha256" {
            format!("hash:{digest}")
        } else {
            format!("hash:{kind}:{digest}")
        };
        let (ok, se) = pack(
            &[&format!("phrase:{PHRASE_HEX}"), &rec],
            &["--pack-preimage", "--no-passphrase"],
        );
        assert!(ok, "{kind}: refused:\n{se}");
        assert!(
            !se.contains("matches no `hash:` record"),
            "{kind}: the phrase DOES match this record, under its own kind:\n{se}"
        );
    }
}

/// ...and it still warns when the phrase genuinely matches nothing, naming the
/// digest under each kind the payload carries — not one bare digest, which
/// would re-create the two-axis collision §6 forbids.
#[test]
fn a_genuinely_orphaned_phrase_warns_and_names_what_it_checked() {
    let (_, se) = pack(
        &[
            &format!("phrase:{PHRASE_HEX}"),
            // the sha256 digest, labelled hash256: matches under NO kind
            &format!("hash:hash256:{D_SHA256}"),
        ],
        &["--pack-preimage", "--no-passphrase"],
    );
    assert!(se.contains("matches no `hash:` record"), "{se}");
    assert!(
        se.contains("checked: hash256 "),
        "the warning must name the KIND it checked under, not a bare digest:\n{se}"
    );
    assert!(
        se.contains(&D_HASH256[..8]),
        "it must show the phrase's digest UNDER THAT KIND, which is what the \
         operator compares against:\n{se}"
    );
}

/// §6's producer rule **AT THE WIRE**, which is the only place it counts.
///
/// WHY THIS EXISTS BESIDE THE UNIT TEST. The rule was implemented in
/// `hash_record()` and `me sysw pack` stored the operator's text verbatim, so
/// `hash:sha256:<64hex>` reached the wire tagged — and the SHIPPED device's
/// parser demands a 64-character body, so it treated that record as
/// ClassUnknown and **inert**. `me` accepted it without a word. The unit test
/// was green the whole time because it tested a function with zero production
/// call sites (R0 round 1, I-1).
///
/// A test that drives the real producer is the only one that can see that.
#[test]
fn the_producer_rule_reaches_the_wire_not_just_the_helper() {
    let dir = tempfile::tempdir().unwrap();
    let pack_one = |name: &str, record: &str| -> Vec<u8> {
        let recs = dir.path().join(format!("{name}.txt"));
        let out = dir.path().join(format!("{name}.bin"));
        std::fs::write(&recs, format!("{record}\n")).unwrap();
        Command::cargo_bin("me")
            .unwrap()
            .args([
                "sysw",
                "pack",
                "--in",
                &recs.display().to_string(),
                "--out",
                &out.display().to_string(),
            ])
            .output()
            .unwrap();
        std::fs::read(&out).unwrap()
    };

    // "Input is liberal, output is conservative": the explicit sha256 form is
    // ACCEPTED and produces the SAME BYTES as the bare form.
    let explicit = pack_one("explicit", &format!("hash:sha256:{D_SHA256}"));
    let bare = pack_one("bare", &format!("hash:{D_SHA256}"));
    assert_eq!(
        explicit, bare,
        "an explicit sha256 record must normalise to the bare form on the \
         wire -- the shipped device reads only the bare one"
    );

    // ...and the other three keep their tag, because that is what says which
    // hash the script commits to.
    for (kind, digest) in [
        ("hash256", D_HASH256),
        ("ripemd160", D_RIPEMD160),
        ("hash160", D_HASH160),
    ] {
        let blob = pack_one(kind, &format!("hash:{kind}:{digest}"));
        let hay = String::from_utf8_lossy(&blob);
        assert!(
            hay.contains(&format!("hash:{kind}:{digest}")),
            "{kind}: the tag must survive to the wire"
        );
    }
}

/// A kind-tagged record is INERT on firmware without hashlock-kind support, and
/// `me sysw pack` is the last moment the host can say so (P3 journey walk, J-1).
///
/// The device's `ParseHashRecord` demands a 64-character body, so a tagged
/// record classifies as `ClassUnknown` and surfaces only in the door's generic
/// "not understood" count — no screen, no hashlock path. Silence at pack reads
/// as assent, and the operator's next act is to carry the payload to a machine.
///
/// **The note must be about the FIRMWARE, not a phase or a version.** Phase 4
/// makes upgraded firmware read the tag; a device not yet flashed stays inert
/// forever, so "not yet" would expire into a lie while the condition holds.
#[test]
fn a_kind_tagged_record_is_announced_as_inert_on_firmware_without_support() {
    for (kind, digest) in [
        ("hash256", D_HASH256),
        ("ripemd160", D_RIPEMD160),
        ("hash160", D_HASH160),
    ] {
        let (ok, se) = pack(&[&format!("hash:{kind}:{digest}")], &[]);
        assert!(ok, "{kind}: refused:\n{se}");
        assert!(
            se.contains("kind-tagged hash record"),
            "{kind}: pack says nothing about the tag being inert:\n{se}"
        );
        assert!(
            se.contains(kind),
            "{kind}: the note must name the kind:\n{se}"
        );
        assert!(
            se.contains("firmware"),
            "{kind}: phrased about the firmware, not a phase — a device never \
             flashed stays inert forever:\n{se}"
        );
        // ...and it forbids the workaround the refusal used to invite.
        assert!(
            se.contains("Do not strip the tag"),
            "{kind}: the note must forbid stripping, which succeeds SILENTLY \
             for hash256 because that digest is also 64 hex:\n{se}"
        );
    }
}

/// ...and it stays QUIET for a bare sha256 record, which every shipped device
/// reads. A note on the working path is a note operators learn to skip.
#[test]
fn a_bare_sha256_record_gets_no_inertness_note() {
    let (ok, se) = pack(&[&format!("hash:{D_SHA256}")], &[]);
    assert!(ok, "refused:\n{se}");
    assert!(
        !se.contains("kind-tagged hash record"),
        "the bare form is what every shipped device reads:\n{se}"
    );
    // The explicit sha256 form normalises to bare, so it must be quiet too.
    let (_, se) = pack(&[&format!("hash:sha256:{D_SHA256}")], &[]);
    assert!(
        !se.contains("kind-tagged hash record"),
        "an explicit sha256 record is normalised away, so nothing is inert:\n{se}"
    );
}
