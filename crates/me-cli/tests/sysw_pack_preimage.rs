//! `me sysw pack --pack-preimage` (SPEC_hashlock_H6 §3.2, §3.3, §8.1, §8.2).
//!
//! The flag gates ADMISSION, never CLASSIFICATION, and this file is where that
//! design earns its keep: `decide_sealing` is byte-unchanged and still seals a
//! payload holding a preimage, because the strict classifier it calls already
//! answers `Preimage`.

use assert_cmd::Command;

fn me() -> Command {
    Command::cargo_bin("me").expect("me binary")
}

/// The corpus `kind` row: 75 characters, id `hash`, X = 0xab * 32.
const PLATE: &str = "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c";
/// Its digest, from the same corpus row.
const PLATE_H: &str = "9a2db2e23f1504cd056606553ac049c5e718e8f9ce9233876df1a7a1821af885";
/// A kind-0x03 single under an id outside {entr, hash}.
const WRONG_ID: &str =
    "ms10testsqvrsu9guyv4rzwplgex4gkmzd9c8wl593jfe4gdg47mtm3xt6tv7qh3pm4xrfdlvvp";
/// The same kind byte under `entr` — the SHIPPED TagKindMismatch case.
const ENTR_ID: &str = "ms10entrsqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5gz69g08wwtz9";

fn hexs(s: &str) -> String {
    use std::fmt::Write as _;
    s.bytes().fold(String::new(), |mut o, b| {
        let _ = write!(o, "{b:02x}");
        o
    })
}

fn phrase_record(method: &str, phrase: &str) -> String {
    format!("phrase:{}", hexs(&format!("{method},{phrase}")))
}

/// `correct horse battery staple` under `hardened`, from the corpus's
/// `derivation` anchor row.
const ANCHOR: &str = "correct horse battery staple";
const ANCHOR_HARDENED_H: &str = "3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12";

/// Every invocation goes through `--in`, NEVER argv, and that is H6's own
/// consequence rather than test hygiene: `Class::Preimage` and `Class::Phrase`
/// are BEARER, so `argv_secret_guard` refuses either on the command line before
/// the parser runs. **§12 item 3's acceptance path has to use a file too**, and
/// so does every operator.
fn run_with(flags: &[&str], records: &[&str]) -> std::process::Output {
    let dir = tempfile::tempdir().unwrap();
    let recs = dir.path().join("records.txt");
    std::fs::write(&recs, records.join("\n") + "\n").unwrap();
    let out = dir.path().join("payload.bin");
    let mut a: Vec<String> = vec![
        "sysw".into(),
        "pack".into(),
        "--in".into(),
        recs.display().to_string(),
        "--out".into(),
        out.display().to_string(),
    ];
    a.extend(flags.iter().map(|s| s.to_string()));
    let o = me().args(&a).output().unwrap();
    std::mem::forget(dir);
    o
}

fn stderr(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// §8.1.1: both carriers are refused BY INDEX without the flag, and the
/// refusal names the flag.
///
/// MUTATION: drop `admit_check`'s second rule -> both pack at exit 0.
#[test]
fn no_flag_refuses_both_carriers_by_index() {
    for (i, rec) in [PLATE.to_string(), phrase_record("hardened", ANCHOR)]
        .into_iter()
        .enumerate()
    {
        let o = run_with(&["--no-passphrase"], &[&rec]);
        assert!(!o.status.success(), "carrier {i} packed without the flag");
        let e = stderr(&o);
        assert!(
            e.contains("record 0 (records count from 0)"),
            "refusal does not carry the index: {e}"
        );
        assert!(
            e.contains("this payload did not ask for one"),
            "not §8.1.1's body: {e}"
        );
        assert!(
            e.contains("Re-run with --pack-preimage if that is what you intend."),
            "§8.1.1's final sentence is missing: {e}"
        );
    }
}

/// The flag admits both.
#[test]
fn the_flag_admits_both_carriers() {
    for rec in [PLATE.to_string(), phrase_record("sha256", ANCHOR)] {
        let o = run_with(&["--no-passphrase", "--pack-preimage"], &[&rec]);
        assert!(
            o.status.success(),
            "--pack-preimage refused {rec}: {}",
            stderr(&o)
        );
    }
}

/// **THE FUNDS-RELEVANT ROW.** `--pack-preimage` alone, with no
/// `--no-passphrase`, SEALS — because classification is unconditional, so
/// `decide_sealing`'s strict classifier already sees the class and `is_secret()`
/// is true for it.
///
/// MUTATION: make `classify` admission-gated and leave `decide_sealing` on the
/// strict classifier -> this reports NOT SEALED and the payload ships bearer
/// material in cleartext. **This row is why §3.2 gates admission and not
/// classification.**
/// MUTATION: drop Preimage/Phrase from `Class::is_secret` -> the same.
#[test]
fn the_flag_seals_by_default_and_names_the_class() {
    let o = run_with(&["--pack-preimage"], &[PLATE]);
    assert!(o.status.success(), "{}", stderr(&o));
    let e = stderr(&o);
    assert!(e.contains("sealing:  SEALED"), "not sealed: {e}");
    assert!(
        e.contains("hashlock preimage plate"),
        "the sealing line does not name the class: {e}"
    );
    // §8.2.4, and it prints AFTER the sealing line.
    let seal_at = e.find("sealing:  SEALED").unwrap();
    let note_at = e
        .find("this payload is SEALED and holds a hashlock preimage")
        .expect("§8.2.4 was not printed");
    assert!(note_at > seal_at, "§8.2.4 printed BEFORE the sealing line");
}

/// §3.2 item 1, asserted directly: the class does not depend on the flag.
#[test]
fn classification_is_unconditional() {
    use mnemonic_engrave::sysw::{classify, classify_with, record::Class, Admission};
    let phrase = phrase_record("hardened", ANCHOR);
    for rec in [PLATE, phrase.as_str()] {
        let strict = classify(rec);
        let loose = classify_with(
            rec,
            Admission {
                pack_preimage: true,
                ..Default::default()
            },
        );
        assert_eq!(strict, loose, "{rec}: the class moved with the admission");
        assert!(matches!(strict, Class::Preimage | Class::Phrase), "{rec}");
        assert!(strict.is_secret(), "{rec} is not secret");
        assert!(strict.is_bearer(), "{rec} is not bearer");
    }
}

/// §4.3's three-id partition, and each id gets its OWN refusal.
///
/// MUTATION: drop the id test from `preimage_plate_admissible` -> the WRONG_ID
/// row packs. MUTATION: use `entr` as the "any other id" case -> the row
/// expects §8.1.2 and gets the shipped TagKindMismatch text.
#[test]
fn the_three_ids_each_get_their_own_refusal() {
    // id `hash`: admissible under the flag.
    assert!(run_with(&["--no-passphrase", "--pack-preimage"], &[PLATE])
        .status
        .success());

    // any other id: §8.1.2, WITH and WITHOUT the flag, and ONE refusal.
    for extra in [
        vec!["--no-passphrase"],
        vec!["--no-passphrase", "--pack-preimage"],
    ] {
        let o = run_with(&extra, &[WRONG_ID]);
        assert!(!o.status.success(), "the wrong-id record packed: {extra:?}");
        let e = stderr(&o);
        assert!(
            e.contains("4-character id is not `hash`"),
            "not §8.1.2: {e}"
        );
        assert!(
            e.contains("roughly 1 in 256 of them look like this"),
            "§8.1.2's collision sentence is missing, and it is the point: {e}"
        );
        assert!(
            !e.contains("Re-run with --pack-preimage"),
            "a wrong-id record was sent to a flag that will refuse it: {e}"
        );
    }

    // id `entr`: the SHIPPED TagKindMismatch text, UNCHANGED.
    let o = run_with(&["--no-passphrase", "--pack-preimage"], &[ENTR_ID]);
    assert!(!o.status.success());
    let e = stderr(&o);
    assert!(
        e.contains("id and kind byte disagree"),
        "the entr row did not reach the shipped TagKindMismatch text: {e}"
    );
}

/// §3.3 / §8.2, all four, and the ORDER against the passphrase ceremony
/// (F-246: "a warning the operator reads after writing a passphrase down is a
/// warning about work already done").
///
/// MUTATION: move warning 1 after the ceremony -> the ordering assertion reds.
#[test]
fn the_warnings_print_in_the_f246_order() {
    let hash_rec = format!("hash:{PLATE_H}");
    let o = run_with(&["--pack-preimage"], &[PLATE, &hash_rec]);
    assert!(o.status.success(), "{}", stderr(&o));
    let e = stderr(&o);
    let transit = e
        .find("this payload carries a hashlock PREIMAGE")
        .expect("§8.2.1 was not printed");
    let ceremony = e
        .find("write this down and store it APART")
        .expect("the passphrase ceremony did not run");
    assert!(
        transit < ceremony,
        "§8.2.1 printed AFTER the passphrase ceremony:\n{e}"
    );
    // The digests MATCH, so no orphan warning.
    assert!(
        !e.contains("matches no `hash:` record"),
        "an orphan warning fired for a matching digest:\n{e}"
    );
}

/// §8.2.2: the flag with nothing to admit is a WARNING, never a refusal.
///
/// MUTATION: refuse instead -> exit != 0, and a flag that only LOOSENS
/// admission has started refusing.
#[test]
fn the_flag_over_nothing_is_a_warning() {
    let o = run_with(
        &["--no-passphrase", "--pack-preimage"],
        &["text:48656c6c6f2c20576f726c6421"],
    );
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(
        stderr(&o).contains("--pack-preimage was passed and this payload holds no preimage plate"),
        "{}",
        stderr(&o)
    );
}

/// §8.2.2 is SILENT when a carrier-SHAPED record is present (R0 round 0,
/// journey I-3 = fidelity M-4).
///
/// Every shape §4.3 narrows OUT classifies `Unknown`, so `carriers` is empty and
/// the no-op warning fired directly above a refusal naming the same record --
/// two consecutive lines, the first saying the payload holds no preimage plate
/// and the second saying record 0 IS a kind-0x03 preimage payload. The first is
/// printed first, so it is what the operator reads first, and its plain meaning
/// -- drop the flag, it did nothing -- is the wrong next move.
///
/// MUTATION: restore the unconditional emptiness test -> every row here prints
/// both lines, and §12 item 5's "one refusal and not three" is broken on the
/// one path §4.3 exists for.
#[test]
fn the_no_op_warning_is_silent_when_a_carrier_shaped_record_is_present() {
    let bad_phrase = format!("phrase:{}", hexs(ANCHOR)); // no method, no comma
    for (what, rec) in [
        ("the wrong-id plate", WRONG_ID.to_string()),
        ("the id/kind mismatch", ENTR_ID.to_string()),
        ("the UPPERCASE plate", PLATE.to_uppercase()),
        ("a hand-built `phrase:` record with no method", bad_phrase),
    ] {
        let o = run_with(&["--no-passphrase", "--pack-preimage"], &[&rec]);
        assert!(!o.status.success(), "{what} packed");
        let e = stderr(&o);
        assert!(
            !e.contains("this payload holds no preimage plate and no `phrase:` record"),
            "§8.2.2 fired above the refusal for {what}, and the two contradict each other:\n{e}"
        );
        assert!(
            e.contains("record 0 (records count from 0)"),
            "{what} drew no refusal at all:\n{e}"
        );
    }
    // AND IT STILL FIRES when the flag really had nothing to admit: the
    // suppression is a SHAPE test, not a way of turning the warning off.
    let o = run_with(
        &["--no-passphrase", "--pack-preimage"],
        &["text:48656c6c6f2c20576f726c6421"],
    );
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(
        stderr(&o).contains("this payload holds no preimage plate and no `phrase:` record"),
        "the no-op warning went silent over a payload with no carrier shape at all:\n{}",
        stderr(&o)
    );
}

/// §8.2.3's payload-wide note prints ONCE PER PAYLOAD (R0 round 0, fidelity
/// M-2), not once per carrier.
///
/// §3.3 makes all four warnings payload-wide and this sentence's own subject is
/// "this payload"; it lived inside the per-carrier loop, so a payload holding a
/// plate and a `phrase:` record printed two byte-identical copies of it -- which
/// works against the reason it is a note and not a warning ("a WARNING on every
/// single run is how a warning stops being read").
///
/// MUTATION: move the branch back inside the loop -> the count is 2.
#[test]
fn the_no_hash_record_note_prints_once_per_payload() {
    let ph = phrase_record("hardened", ANCHOR);
    let o = run_with(&["--no-passphrase", "--pack-preimage"], &[PLATE, &ph]);
    assert!(o.status.success(), "{}", stderr(&o));
    let e = stderr(&o);
    let n = e
        .matches("me: note — this payload holds no `hash:` record")
        .count();
    assert_eq!(n, 1, "the payload-wide note printed {n} times:\n{e}");
}

/// §8.2.4 does not tell the operator to look ABOVE for a passphrase that is
/// printed BELOW (R0 round 0, fidelity I-6 = journey M-1).
///
/// `report_sealed_preimage` runs immediately after the sealing line and the
/// ceremony runs after it, so "the passphrase above" referred to nothing at the
/// moment it was printed -- and with `--passphrase-ask` the prompt has not even
/// been shown yet. The sealing line one line above says "the passphrase below".
///
/// MUTATION: restore "the passphrase above" -> this row fails, and the two
/// consecutive lines point in opposite directions at the same passphrase.
#[test]
fn the_sealed_transit_note_does_not_point_the_wrong_way() {
    let o = run_with(&["--passphrase-words", "4", "--pack-preimage"], &[PLATE]);
    assert!(o.status.success(), "{}", stderr(&o));
    let e = stderr(&o);
    let note = e
        .find("this payload is SEALED and holds a hashlock preimage")
        .expect("§8.2.4 was not printed");
    let ceremony = e
        .find("write this down and store it APART")
        .expect("the passphrase ceremony did not run");
    assert!(
        note < ceremony,
        "§8.2.4 is meant to print BEFORE the ceremony (§3.3's ordering):\n{e}"
    );
    assert!(
        !e.contains("needs the passphrase above"),
        "§8.2.4 points ABOVE at a passphrase that is printed BELOW it:\n{e}"
    );
    assert!(
        e.contains("needs this payload's passphrase"),
        "§8.2.4 does not name the passphrase at all:\n{e}"
    );
}

/// §8.2.3, the half that matters: a `phrase:` record has NO CLI producer, so
/// the operator hand-builds it, and two hand-build errors pass every rule the
/// parser applies.
///
/// **A SPACE AFTER THE COMMA IS PART OF THE PHRASE** — §3.1 cuts on the FIRST
/// comma and the remainder must be printable ASCII, and `0x20` is printable —
/// so `hardened, correct horse…` is ADMITTED and derives a different preimage.
///
/// MUTATION: drop the phrase arm from the orphan check -> the hand-built row
/// prints nothing and the operator meets the mismatch on the device, after a
/// pick, at a screen with no copy for a digest that matches nothing.
#[test]
fn a_space_after_the_comma_derives_a_different_preimage_and_warns() {
    let hash_rec = format!("hash:{ANCHOR_HARDENED_H}");

    // The correct record: its digest matches, so NO warning.
    let good = phrase_record("hardened", ANCHOR);
    let o = run_with(&["--no-passphrase", "--pack-preimage"], &[&good, &hash_rec]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(
        !stderr(&o).contains("matches no `hash:` record"),
        "the matching phrase warned:\n{}",
        stderr(&o)
    );

    // One space after the comma: a DIFFERENT phrase, a different preimage.
    let spaced = phrase_record("hardened", &format!(" {ANCHOR}"));
    assert_ne!(good, spaced);
    let o = run_with(
        &["--no-passphrase", "--pack-preimage"],
        &[&spaced, &hash_rec],
    );
    assert!(o.status.success(), "{}", stderr(&o));
    let e = stderr(&o);
    assert!(
        e.contains("is a hashlock phrase whose digest"),
        "the space-after-the-comma row did not warn:\n{e}"
    );
    assert!(
        e.contains("a space after the comma is part of the phrase"),
        "the warning does not name the hand-build error:\n{e}"
    );

    // The wrong SELECTOR is the other hand-build error, and it warns too.
    let wrong_method = phrase_record("sha256", ANCHOR);
    let o = run_with(
        &["--no-passphrase", "--pack-preimage"],
        &[&wrong_method, &hash_rec],
    );
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(
        stderr(&o).contains("is a hashlock phrase whose digest"),
        "the wrong-selector row did not warn:\n{}",
        stderr(&o)
    );
}

/// §8.2.3's INCOMPLETE-versus-CONTRADICTORY split, and it is the row that keeps
/// the warning readable.
///
/// `ms hashlock --out X.txt` writes ONLY the ms1 string and prints `hash:` to
/// stdout, so the minimal correct journey packs a payload with no `hash:`
/// record at all. A WARNING there would fire on every single run, which is how
/// a warning stops being read.
///
/// MUTATION: make the no-`hash:` case a WARNING -> §12 item 3's own acceptance
/// path fires one every time.
#[test]
fn no_hash_record_at_all_is_a_note_not_a_warning() {
    let o = run_with(&["--no-passphrase", "--pack-preimage"], &[PLATE]);
    assert!(o.status.success(), "{}", stderr(&o));
    let e = stderr(&o);
    assert!(
        e.contains("me: note — this payload holds no `hash:` record"),
        "the incomplete case did not draw the NOTE:\n{e}"
    );
    assert!(
        !e.contains("matches no `hash:` record"),
        "the incomplete case drew a WARNING:\n{e}"
    );

    // CONTRADICTORY: the payload DOES hold hash: records and none matches.
    let other = format!("hash:{ANCHOR_HARDENED_H}");
    let o = run_with(&["--no-passphrase", "--pack-preimage"], &[PLATE, &other]);
    assert!(o.status.success(), "{}", stderr(&o));
    let e = stderr(&o);
    assert!(
        e.contains("is a preimage whose digest"),
        "the contradictory case did not WARN:\n{e}"
    );
    assert!(
        !e.contains("me: note — this payload holds no"),
        "the contradictory case drew the NOTE:\n{e}"
    );
}
