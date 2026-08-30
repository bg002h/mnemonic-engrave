//! **`--as` end to end — the flag, the md1 build, the identification block.**
//!
//! `SPEC_descriptor_input.md` §5.1 (the flag and its single-document contract),
//! §5.3 (the md1 path and its three representability rules), §5.4 (the two-tier
//! identification block) and §11 items 2 and 5.
//!
//! Every fixture here is a row of `testdata/descriptor_seam_vectors.json` —
//! looked up BY NAME rather than pasted, so a fixture cannot drift from the
//! file both repos pin. The one exception is a constructed input that the
//! vector file deliberately does not carry, and each of those says so at its
//! use site.

use std::process::Output;

const VECTORS: &str = "testdata/descriptor_seam_vectors.json";

/// A vector row's `input`, by row name. Panics if the row is gone — a renamed
/// row must red the suite rather than silently stop testing anything.
fn row_input(name: &str) -> String {
    let raw = std::fs::read(VECTORS).unwrap_or_else(|e| panic!("{VECTORS}: {e}"));
    let doc: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    for r in doc["vectors"].as_array().unwrap() {
        if r["name"].as_str().unwrap() == name {
            return r["input"].as_str().unwrap().to_string();
        }
    }
    panic!("{VECTORS}: no row named {name:?}");
}

/// A row's declared value for one column, as a string.
fn row_field(name: &str, field: &str) -> Option<String> {
    let raw = std::fs::read(VECTORS).unwrap();
    let doc: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    for r in doc["vectors"].as_array().unwrap() {
        if r["name"].as_str().unwrap() == name {
            return r.get(field).and_then(|v| v.as_str()).map(str::to_owned);
        }
    }
    panic!("{VECTORS}: no row named {name:?}");
}

/// Run `me sysw pack` with the given extra arguments, feeding `input` through
/// `--in` (a temp file), which is the delivery §7's gate rows use.
fn pack_in(input: &str, extra: &[&str]) -> (Output, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("document.txt");
    std::fs::write(&path, input).unwrap();
    let out = assert_cmd::Command::cargo_bin("me")
        .unwrap()
        .args(["sysw", "pack", "--no-passphrase"])
        .args(extra)
        .arg("--in")
        .arg(&path)
        .output()
        .unwrap();
    (out, dir)
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

fn code(o: &Output) -> i32 {
    o.status.code().unwrap()
}

// ───────────────────────────────────────────────────────────────────────────
// P2.1 — the flag surface, and §5.1's single-document contract
// ───────────────────────────────────────────────────────────────────────────

/// The flag exists and takes `md1`. Before P2.1 this was clap's
/// `unexpected argument '--as' found`, which is what made §5.1's own choice
/// block advertise a flag the binary rejected (IMPL-P1's F-3).
#[test]
fn as_md1_is_a_flag_the_binary_knows() {
    let (out, _d) = pack_in(
        &row_input("formats-happy/bip380-sortedmulti-multipath"),
        &["--as", "md1"],
    );
    let err = stderr(&out);
    assert!(
        !err.contains("unexpected argument"),
        "`--as md1` is still an unknown flag:\n{err}"
    );
    assert_eq!(code(&out), 0, "a carried wallet packs. stderr:\n{err}");
}

/// `descriptor` is a VALUE the flag accepts — and since S2 it PACKS. Before
/// S2 this test pinned §5.1's window refusal at 3, which is what made the value
/// "accepted but not carried"; the assertion moved with the build rather than
/// being deleted, because "not an unknown value" is the half that never moves.
#[test]
fn as_descriptor_is_a_value_the_flag_accepts_and_packs() {
    let (out, _d) = pack_in(
        &row_input("formats-happy/bip380-sortedmulti-multipath"),
        &["--as", "descriptor"],
    );
    let err = stderr(&out);
    assert!(
        !err.contains("unexpected argument") && !err.contains("invalid value"),
        "`--as descriptor` is not an accepted value:\n{err}"
    );
    assert_eq!(code(&out), 0, "a carried wallet packs. stderr:\n{err}");
}

/// A value outside the two is USAGE — nothing about the data was judged.
#[test]
fn as_refuses_a_value_outside_the_two() {
    let (out, _d) = pack_in(
        &row_input("formats-happy/bip380-sortedmulti-multipath"),
        &["--as", "qr"],
    );
    assert_eq!(code(&out), 2, "stderr:\n{}", stderr(&out));
}

/// §5.1's single-document contract: `--as` with more than one argv operand.
#[test]
fn as_with_two_argv_operands_is_usage() {
    let d = row_input("formats-happy/bip380-sortedmulti-multipath");
    let out = assert_cmd::Command::cargo_bin("me")
        .unwrap()
        .args(["sysw", "pack", "--no-passphrase", "--as", "md1", &d, &d])
        .output()
        .unwrap();
    let err = stderr(&out);
    assert_eq!(code(&out), 2, "stderr:\n{err}");
    assert!(
        err.contains("--as packs exactly one descriptor per invocation."),
        "stderr:\n{err}"
    );
}

/// §5.1's single-document contract: `--as` with BOTH argv and `--in`.
#[test]
fn as_with_argv_and_in_together_is_usage() {
    let d = row_input("formats-happy/bip380-sortedmulti-multipath");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("document.txt");
    std::fs::write(&path, &d).unwrap();
    let out = assert_cmd::Command::cargo_bin("me")
        .unwrap()
        .args(["sysw", "pack", "--no-passphrase", "--as", "md1", &d])
        .arg("--in")
        .arg(&path)
        .output()
        .unwrap();
    let err = stderr(&out);
    assert_eq!(code(&out), 2, "stderr:\n{err}");
    assert!(
        err.contains("--as packs exactly one descriptor per invocation."),
        "stderr:\n{err}"
    );
}

/// ONE argv operand IS the document — §5.1 says so explicitly, and a
/// multi-line document simply cannot arrive that way.
#[test]
fn as_with_one_argv_operand_is_the_document() {
    let d = row_input("formats-happy/bip380-sortedmulti-multipath");
    let out = assert_cmd::Command::cargo_bin("me")
        .unwrap()
        .args(["sysw", "pack", "--no-passphrase", "--as", "md1", &d])
        .output()
        .unwrap();
    let err = stderr(&out);
    assert_eq!(code(&out), 0, "stderr:\n{err}");
}

/// `--as` on an input that is not a descriptor at all gets §6's cause
/// selection, not the record refusal: with the flag present the invocation is
/// single-document by declaration, so §5.1's shape gate has nothing to decide.
#[test]
fn as_on_a_non_descriptor_gets_the_cause_selection() {
    let (out, _d) = pack_in("hello, this is not a descriptor\n", &["--as", "md1"]);
    let err = stderr(&out);
    assert_eq!(code(&out), 3, "stderr:\n{err}");
    assert!(
        err.contains("not a wallet descriptor in any of the four forms"),
        "stderr:\n{err}"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// §11 item 2 — the acceptance walk, all four formats
// ───────────────────────────────────────────────────────────────────────────

/// One exemplar of the walk: a document in one of §4's four formats, and the
/// two values the round trip has to land on.
struct Exemplar {
    what: &'static str,
    document: String,
    /// The template `md decode` reads back — WITH §5.3(a′)'s materialised
    /// `<0;1>/*` where the input was childless.
    template: &'static str,
    /// Receive address 0, as the Go `address` package derives it from the
    /// ORIGINAL descriptor. Taken from the shared vector file rather than
    /// pasted, so this half cannot drift from the device half.
    address_0: String,
    /// Whether §5.3(b)'s label warning is expected.
    label: Option<&'static str>,
}

fn exemplars() -> Vec<Exemplar> {
    let bip380 = row_input("formats-happy/bip380-sortedmulti-multipath");
    vec![
        Exemplar {
            what: "§4.2 BlueWallet — the fork's own 14-line `sh` fixture, J1's artifact",
            document: row_input("formats-happy/bluewallet-sh-fixture"),
            template: "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
            address_0: row_field("formats-happy/bluewallet-sh-fixture", "address_0").unwrap(),
            label: Some("sh"),
        },
        Exemplar {
            what: "§4.3 plain BIP-380",
            document: bip380.clone(),
            template: "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
            address_0: row_field("formats-happy/bip380-sortedmulti-multipath", "address_0")
                .unwrap(),
            label: None,
        },
        Exemplar {
            // CONSTRUCTED, and §11 item 2 requires it to be: the fork's own JSON
            // fixture is `/0/*`, which `--as md1` REFUSES per §5.3(a), so it
            // cannot be this item's exemplar. The descriptor inside is the
            // vector file's own BIP-380 row, which is why `address_0` below is
            // still a Go-measured value rather than a Rust-derived one.
            what: "§4.4 `{label, descriptor}` JSON, with a non-`/0/*` descriptor",
            document: format!(
                "{{\n  \"label\": \"Test Multisig 2-of-3\",\n  \"descriptor\": \"{bip380}\"\n}}\n"
            ),
            template: "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
            address_0: row_field("formats-happy/bip380-sortedmulti-multipath", "address_0")
                .unwrap(),
            label: Some("Test Multisig 2-of-3"),
        },
        Exemplar {
            what: "§4.5 the promoted bare key",
            document: row_input("promotion/02-bare-zpub"),
            template: "wpkh(@0/<0;1>/*)",
            address_0: row_field("promotion/02-bare-zpub", "address_0").unwrap(),
            label: None,
        },
    ]
}

/// The four formats §11 item 1 names, for the DESCRIPTOR path.
///
/// They are not item 2's four. Item 2's JSON exemplar had to be constructed
/// with a non-`/0/*` descriptor, because `--as md1` refuses `/0/*` per §5.3(a);
/// `--as descriptor` carries `/0/*` exactly — that is the whole point of the
/// path — so item 1 uses the fork's OWN shipped JSON fixture, label and all.
struct Item1 {
    what: &'static str,
    row: &'static str,
    /// §5.3(b)'s label warning: the label, where the input carries one. Named
    /// expected output rather than tolerated noise.
    label: Option<&'static str>,
}

const ITEM_1: [Item1; 4] = [
    Item1 {
        what: "§4.2 BlueWallet — the fork's own 14-line `sh` fixture",
        row: "formats-happy/bluewallet-sh-fixture",
        label: Some("sh"),
    },
    Item1 {
        what: "§4.3 plain BIP-380",
        row: "formats-happy/bip380-sortedmulti-multipath",
        label: None,
    },
    Item1 {
        what: "§4.4 the fork's own `{label, descriptor}` JSON export, `/0/*` and all",
        row: "formats-happy/json-label-descriptor",
        label: Some("Test Multisig 2-of-3"),
    },
    Item1 {
        what: "§4.5 the promoted bare key",
        row: "promotion/01-bare-xpub",
        label: None,
    },
];

/// **§11 item 1, host half.** `me sysw pack --as descriptor --in <each of the
/// four formats>` produces a container holding ONE record, and that record is
/// §5.2's: the canonical re-encode, which `sysw::classify` answers `Descriptor`
/// on.
///
/// **The classify assertion is a RECORD-CLASSIFICATION check, not a fixed
/// point.** It says the record `me` just packed is the record `me` would
/// classify as a descriptor — the property that makes the container readable by
/// the same predicate that admitted it. §7 requirement 4's real fixed point,
/// `encode(parse(canonical)) == canonical`, is a different claim and is already
/// asserted by the seam tests; conflating the two would let a broken re-encoder
/// look verified.
///
/// The round trip goes through the CONTAINER: the record is read back out of
/// the packed bytes with `sysw::open`, so a follower that agreed with itself
/// while the pack dropped or rewrote the string could not pass. And the record
/// is compared to the vector file's `canonical` column — a value MEASURED from
/// the device's own parser — so "canonical" here is not `me` marking its own
/// homework.
#[test]
fn item_1_every_format_packs_one_descriptor_record() {
    let mut formats = std::collections::BTreeSet::new();
    for e in &ITEM_1 {
        let dir = tempfile::tempdir().unwrap();
        let doc = dir.path().join("document.txt");
        let out = dir.path().join("container.bin");
        std::fs::write(&doc, row_input(e.row)).unwrap();

        let run = assert_cmd::Command::cargo_bin("me")
            .unwrap()
            .args(["sysw", "pack", "--no-passphrase", "--as", "descriptor"])
            .arg("--in")
            .arg(&doc)
            .arg("--out")
            .arg(&out)
            .output()
            .unwrap();
        let err = stderr(&run);
        assert_eq!(code(&run), 0, "{}: pack. stderr:\n{err}", e.what);

        // §5.3(b): the label is display-only and does not travel, and the
        // operator hears so on the path that packs.
        match e.label {
            Some(l) => assert!(
                err.contains(&format!(
                    "warning: the wallet's name \"{l}\" is only a label: it will not \
                     appear in the payload, on the device, or on the engraved plate. \
                     Nothing else is lost."
                )),
                "{}: no label warning. stderr:\n{err}",
                e.what
            ),
            None => assert!(
                !err.contains("warning: the wallet's name"),
                "{}: a label warning with no label. stderr:\n{err}",
                e.what
            ),
        }

        let blob = std::fs::read(&out).unwrap();
        let payload = mnemonic_engrave::sysw::open(&blob, None).unwrap();
        assert!(
            payload.secret.is_empty(),
            "{}: nothing here is secret",
            e.what
        );
        assert_eq!(
            payload.public.len(),
            1,
            "{}: §5.2's record is ONE record, got {:?}",
            e.what,
            payload.public
        );
        let record = &payload.public[0];
        assert_eq!(
            mnemonic_engrave::sysw::classify(record),
            mnemonic_engrave::sysw::record::Class::Descriptor,
            "{}: the packed record does not classify as a descriptor",
            e.what
        );
        assert_eq!(
            record,
            &row_field(e.row, "canonical").expect("an admitted row carries a canonical"),
            "{}: the packed record is not the canonical the device measured",
            e.what
        );
        formats.insert(row_field(e.row, "format").unwrap());
    }
    assert_eq!(
        formats.len(),
        4,
        "the four exemplars must be four FORMATS, not one format four times: {formats:?}"
    );
}

/// **§11 item 2.** `me sysw pack --as md1 --in <each of the four formats>`
/// produces a container whose records read back to the expected template — with
/// §5.3(a′)'s materialised `<0;1>/*` where the input was childless — and whose
/// derived receive address 0 equals the one the Go `address` package derives
/// from the original descriptor.
///
/// The round trip goes through the CONTAINER, not through the builder: the
/// records are read back out of the packed bytes with `sysw::open`, so a
/// builder that agreed with itself while the pack dropped or reordered a chunk
/// could not pass.
#[test]
fn item_2_every_format_packs_reads_back_and_derives_the_device_address() {
    let mut walked = 0usize;
    for e in exemplars() {
        let dir = tempfile::tempdir().unwrap();
        let doc = dir.path().join("document.txt");
        let out = dir.path().join("container.bin");
        std::fs::write(&doc, &e.document).unwrap();

        let run = assert_cmd::Command::cargo_bin("me")
            .unwrap()
            .args(["sysw", "pack", "--no-passphrase", "--as", "md1"])
            .arg("--in")
            .arg(&doc)
            .arg("--out")
            .arg(&out)
            .output()
            .unwrap();
        let err = stderr(&run);
        assert_eq!(code(&run), 0, "{}: pack. stderr:\n{err}", e.what);

        // §5.3(b): the label is dropped by both paths, and the operator is told.
        match e.label {
            Some(l) => assert!(
                err.contains(&format!(
                    "warning: the wallet's name \"{l}\" is only a label"
                )),
                "{}: no label warning. stderr:\n{err}",
                e.what
            ),
            None => assert!(
                !err.contains("warning: the wallet's name"),
                "{}: a label warning with no label. stderr:\n{err}",
                e.what
            ),
        }

        // The records, out of the packed container.
        let blob = std::fs::read(&out).unwrap();
        let payload = mnemonic_engrave::sysw::open(&blob, None).unwrap();
        assert!(
            payload.secret.is_empty(),
            "{}: nothing here is secret",
            e.what
        );
        assert!(
            payload.public.len() >= 2,
            "{}: md1 always splits into at least two strings, got {}",
            e.what,
            payload.public.len()
        );
        for (i, r) in payload.public.iter().enumerate() {
            assert_eq!(
                mnemonic_engrave::sysw::classify(r),
                mnemonic_engrave::sysw::record::Class::MdMk,
                "{}: record {i} is not packed as MdMk",
                e.what
            );
        }

        // `md decode`'s equivalent, in process: reassemble the chunk set and
        // render the template it carries.
        let refs: Vec<&str> = payload.public.iter().map(String::as_str).collect();
        let back = md_codec::reassemble(&refs).unwrap();
        assert_eq!(
            md_codec::descriptor_to_template(&back).unwrap(),
            e.template,
            "{}: the read-back template",
            e.what
        );

        // The address equality — the claim §5.3 makes and the string layer
        // cannot express: same wallet, different serialisation.
        let got = back
            .derive_address(0, 0, bitcoin::Network::Bitcoin)
            .unwrap()
            .assume_checked()
            .to_string();
        assert_eq!(
            got, e.address_0,
            "{}: the md1 round trip derives a DIFFERENT wallet than the device",
            e.what
        );

        // And the block the operator read carried that same address, so the
        // compare prompt points at the wallet that was actually packed.
        assert!(
            err.contains(&format!("address 0: {}", e.address_0)),
            "{}: the identification block's address 0 is not the packed wallet's",
            e.what
        );
        walked += 1;
    }
    assert_eq!(walked, 4, "§11 item 2 walks all four of §4's formats");
}

// ───────────────────────────────────────────────────────────────────────────
// P2.5 — F-421's converter referral
// ───────────────────────────────────────────────────────────────────────────

/// **F-421**, filed from the 2026-08-28 journey walk (W3). `me` owns a
/// top-level `--in FILE` — the NDEF converter — so the operator's natural
/// `me --in wallet.txt --as descriptor` half-parses THERE and clap tips
/// `--base64`, a flag from the other program, while nothing names `sysw pack`.
///
/// The referral fires on §5.1's OWN shape gate, shared rather than
/// reimplemented, so the two surfaces cannot disagree about what looks like a
/// descriptor. Three inputs: two that must refer, one that must not.
#[test]
fn the_converter_refers_a_descriptor_to_sysw_pack() {
    const REFERRAL: &str =
        "that looks like a wallet DESCRIPTOR, and this command converts one md1/mk1/mt1 \
         string to NFC bytes.";
    const NEXT: &str = "me sysw pack --as <descriptor|md1> --in <your export file>";

    let mut referred = 0;
    for row in [
        "formats-happy/bluewallet-sh-fixture",
        "formats-happy/bip380-sortedmulti-multipath",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wallet.txt");
        std::fs::write(&path, row_input(row)).unwrap();
        let out = assert_cmd::Command::cargo_bin("me")
            .unwrap()
            .arg("--in")
            .arg(&path)
            .output()
            .unwrap();
        let err = stderr(&out);
        assert_eq!(code(&out), 4, "{row}: the converter's own exit code\n{err}");
        assert!(err.contains(REFERRAL), "{row}: no referral\n{err}");
        assert!(
            err.contains(NEXT),
            "{row}: the referral names no command\n{err}"
        );
        referred += 1;
    }
    assert_eq!(referred, 2, "both descriptor spellings refer");

    // The control: an input that is NOT descriptor-shaped keeps the shipped
    // refusal unchanged. A referral on everything would be noise, and it would
    // be the same defect in the other direction.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("junk.txt");
    std::fs::write(&path, "zzz1nonsense\n").unwrap();
    let out = assert_cmd::Command::cargo_bin("me")
        .unwrap()
        .arg("--in")
        .arg(&path)
        .output()
        .unwrap();
    let err = stderr(&out);
    assert_eq!(code(&out), 4, "{err}");
    assert!(err.contains("unrecognized HRP"), "{err}");
    assert!(
        !err.contains("that looks like a wallet DESCRIPTOR"),
        "a non-descriptor was referred:\n{err}"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// Where `--as` sits among the shipped gates
// ───────────────────────────────────────────────────────────────────────────

/// **The argv bearer guard PRECEDES §5.1's shape check.** `--as` with two argv
/// operands is a usage error about the invocation's SHAPE; a `tx:` record on
/// argv is bearer material already in `/proc`, `ps` and the shell history. The
/// more urgent refusal, and the more specific one, has to win — so the `--as`
/// driver sits after `read_records`, where that guard runs.
///
/// Pinned because it is a deliberate ordering, and because the constellation
/// has already been bitten once by placing a new gate above `read_records` and
/// pre-empting exactly this refusal.
#[test]
fn the_argv_bearer_guard_precedes_the_single_document_check() {
    let out = assert_cmd::Command::cargo_bin("me")
        .unwrap()
        .args([
            "sysw",
            "pack",
            "--no-passphrase",
            "--as",
            "md1",
            "tx:00",
            "tx:00",
        ])
        .output()
        .unwrap();
    let err = stderr(&out);
    assert_eq!(
        code(&out),
        3,
        "the bearer refusal is a policy refusal\n{err}"
    );
    assert!(
        err.contains("is a `tx:` record on ARGV"),
        "the flag-shape usage error pre-empted the bearer refusal:\n{err}"
    );
    assert!(
        !err.contains("--as packs exactly one descriptor per invocation."),
        "{err}"
    );
}

/// `--as md1` hands the shipped pipeline RECORDS, so every gate below it is the
/// shipped one rather than a second implementation. `--expect` is the sharpest
/// witness: the md1 cards ARE the descriptor, so `--expect descriptor` passes
/// and `--expect transaction` refuses, with no `--as`-specific code in either.
#[test]
fn the_md1_records_reach_the_shipped_expect_gate() {
    let doc = row_input("formats-happy/bluewallet-sh-fixture");
    for (kinds, want_exit) in [("descriptor", 0), ("transaction", 4)] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("document.txt");
        let out_path = dir.path().join("container.bin");
        std::fs::write(&path, &doc).unwrap();
        let out = assert_cmd::Command::cargo_bin("me")
            .unwrap()
            .args([
                "sysw",
                "pack",
                "--no-passphrase",
                "--as",
                "md1",
                "--expect",
                kinds,
            ])
            .arg("--in")
            .arg(&path)
            .arg("--out")
            .arg(&out_path)
            .output()
            .unwrap();
        assert_eq!(
            code(&out),
            want_exit,
            "--expect {kinds}: stderr:\n{}",
            stderr(&out)
        );
    }
}

// ───────────────────────────────────────────────────────────────────────────
// C1 — `--as descriptor` on a `multi` input must not swallow conjuncts 2–8
// ───────────────────────────────────────────────────────────────────────────

/// The three flag states one file can be given. §5.4's carriage rule is
/// explicit that the no-path determination *"quantifies over both paths, so it
/// needs no flag"*, so a refusal earned by a FLAG-INDEPENDENT conjunct must be
/// the same sentence in all three.
fn under_all_three_flags(document: &str) -> [(String, i32); 3] {
    [vec![], vec!["--as", "md1"], vec!["--as", "descriptor"]].map(|flags| {
        let out = pack_in(document, &flags).0;
        (stderr(&out), code(&out))
    })
}

/// The `multi` twin of a vector row — the construction the adversarial review
/// used to find C1, and the one the vector corpus cannot express (its `multi`
/// rows are gate rows, and gate rows are `--as`-omitted by construction).
fn multi_twin(row: &str) -> String {
    row_input(row).replace("sortedmulti(", "multi(")
}

/// The refusal the flag-independent conjuncts earn, asserted to be identical
/// under all three flag states, and asserted NOT to be conjunct 1's `multi`
/// referral — which claims *"This wallet can still be engraved"* and is FALSE
/// whenever 2–8 refuse, because `--as md1` refuses the same file permanently.
fn assert_same_refusal_under_every_flag(document: &str, want: &str) {
    for (err, rc) in under_all_three_flags(document) {
        assert_eq!(rc, 3, "every flag state refuses. stderr:\n{err}");
        assert!(
            err.contains(want),
            "a flag state lost the refusal conjuncts 2-8 earn.\nWANT: {want}\nGOT:  {err}"
        );
        assert!(
            !err.contains("This wallet can still be engraved"),
            "conjunct 1's `multi` referral suppressed a conjunct 2-8 refusal:\n{err}"
        );
    }
}

/// **C1, instance 1 — conjunct 2, the anyone-can-spend half.** The spec names
/// this case as the reason the ordering rule exists: *"`sortedmulti(0,…)` must
/// hear 'treat those funds as at risk now', never 'nothing is lost by
/// waiting'."*
#[test]
fn as_descriptor_on_multi_still_reports_threshold_below_one() {
    assert_same_refusal_under_every_flag(
        &multi_twin("narrowed/threshold-zero"),
        "threshold 0 means NO signature is required: anyone who can see this script can \
         spend from it.",
    );
}

/// **C1, instance 2 — conjunct 2, the unsatisfiable half.**
#[test]
fn as_descriptor_on_multi_still_reports_threshold_exceeds_keys() {
    assert_same_refusal_under_every_flag(
        &multi_twin("narrowed/threshold-exceeds-keys"),
        "threshold 5 of 2 keys can never be satisfied",
    );
}

/// **C1, instance 3 — conjunct 3, the unspendable key count.**
#[test]
fn as_descriptor_on_multi_still_reports_key_count_exceeded() {
    assert_same_refusal_under_every_flag(
        &multi_twin("narrowed/wsh-sortedmulti-21-keys"),
        "carries at most 15 keys",
    );
}

/// **C1, instance 4 — conjunct 5, the network mixture no address derives from.**
#[test]
fn as_descriptor_on_multi_still_reports_mixed_network() {
    assert_same_refusal_under_every_flag(
        &multi_twin("narrowed/mixed-network"),
        "All keys must share one network.",
    );
}

/// **C1, instance 5 — conjunct 7, the hardened use-site step.**
#[test]
fn as_descriptor_on_multi_still_reports_use_site_hardened() {
    assert_same_refusal_under_every_flag(
        &multi_twin("narrowed/use-site-hardened"),
        "a hardened use-site step cannot be derived from an xpub (BIP-32).",
    );
}

/// **C1, instance 6 — conjunct 7, the non-consecutive multipath.**
#[test]
fn as_descriptor_on_multi_still_reports_use_site_non_consecutive() {
    assert_same_refusal_under_every_flag(
        &multi_twin("narrowed/use-site-non-consecutive"),
        "the device derives only `<i;i+1>` pairs (receive; change).",
    );
}

/// **C1, instance 7 — conjunct 8, the impossible wallet.** This row is already
/// a `multi` in the vector file, and it is `gate`-tagged — which is exactly why
/// the corpus could not see C1: the gate row is `--as`-omitted, the path that
/// was always right.
#[test]
fn as_descriptor_on_multi_still_reports_key_identity() {
    assert_same_refusal_under_every_flag(
        &row_input("gate/colliding-origin-multi"),
        "this wallet description contradicts itself: keys 0 and 1 both claim origin",
    );
}

/// The CONTROL, and it is what stops the fix from being "delete conjunct 1's
/// `multi` arm". A `multi` wallet that passes conjuncts 2–8 must STILL get
/// conjunct 1's permanent refusal under `--as descriptor` — and there the
/// referral is true, because `--as md1` really does carry it.
#[test]
fn as_descriptor_on_a_sound_multi_still_gets_conjunct_1s_permanent_refusal() {
    let document = row_input("neither/wsh-multi");
    let (out, _d) = pack_in(&document, &["--as", "descriptor"]);
    let err = stderr(&out);
    assert_eq!(code(&out), 3, "stderr:\n{err}");
    assert!(
        err.contains("the device's descriptor parser accepts `sortedmulti` and not `multi`."),
        "the sound `multi` lost conjunct 1's refusal:\n{err}"
    );
    // …and the referral it makes is TRUE: the same file packs under `--as md1`.
    let (packed, _d) = pack_in(&document, &["--as", "md1"]);
    assert_eq!(
        code(&packed),
        0,
        "the referral claims `--as md1` carries this file, and it does not:\n{}",
        stderr(&packed)
    );
}

// ───────────────────────────────────────────────────────────────────────────
// I1 — `address 0:` for a wallet whose keys want different receive indices
// ───────────────────────────────────────────────────────────────────────────

const IK1: &str = "[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan";
const IK2: &str = "[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge";

/// **I1.** The block used to decline to derive here, and to blame the keys'
/// use-site depths for it. Every clause of that explanation was false — the
/// address exists and the device derives it, the keys blamed were not at
/// differing depths at all, and a pair that genuinely does differ in depth
/// never reached the branch. The verbatim text is in
/// `design/agent-reports/IMPL-S1S3-adversarial-review.md` (I1); it is
/// deliberately NOT reproduced here, so that grepping the tree for it finds the
/// record and not a live string.
///
/// The four addresses below were derived through the fork's own
/// `address.Receive` (`scripts/descriptor-seam-vectors/goprobe` against
/// `/scratch/code/shibboleth/_work/seam-fork` @ `1f09537`, Go 1.26.3) — the
/// DEVICE, not this build and not `md_codec`. Two of them are the adversarial
/// review's own constructions, re-derived here rather than transcribed.
#[test]
fn address_0_is_derived_for_keys_that_want_different_receive_indices() {
    let cases = [
        (
            "the review's construction: <2;3> mixed with <0;1>/*",
            format!("wsh(sortedmulti(2,{IK1}/<2;3>,{IK2}/<0;1>/*))"),
            "bc1qv70wqy0t9vp4ftlku3yz845x53yqkgm5xlus47m3zq8xzzy503hscqluvy",
        ),
        (
            "the review's second shape: <2;3> mixed with <0;1>, both wildcard-less",
            format!("wsh(sortedmulti(2,{IK1}/<2;3>,{IK2}/<0;1>))"),
            "bc1qlccgxwlhr0rp7xfedcau022p50ulf9r3e33anqqdrevvdrdeqj9s8leyuw",
        ),
        (
            "uniform <2;3> — the twin already handled this, and must not regress",
            format!("wsh(sortedmulti(2,{IK1}/<2;3>,{IK2}/<2;3>))"),
            "bc1qxwcmdqhtvjp6uu6asj0vgz9yvylhwy3ky2y9r3r9lz68rgkwalgqq9dyds",
        ),
        (
            "the genuinely different-DEPTH pair, which never reached the branch",
            format!("wsh(sortedmulti(2,{IK1}/*,{IK2}/<0;1>/*))"),
            "bc1qghwumhcahkfca7qktym7f3htf5wqakz2tyvxraf3fk5k8w0yrzwsg0m3sd",
        ),
    ];
    let mut checked = 0;
    for (what, document, want) in &cases {
        let (out, _d) = pack_in(document, &["--as", "md1"]);
        let err = stderr(&out);
        assert!(
            err.contains(&format!("address 0: {want}")),
            "{what}: the block did not print the device's address 0.\nWANT: {want}\nGOT:  {err}"
        );
        assert!(
            !err.contains("not derived"),
            "{what}: the block still declines to derive:\n{err}"
        );
        checked += 1;
    }
    assert_eq!(checked, 4, "all four constructions are exercised");
}

/// The compare prompt is what `address 0:` exists FOR (walk W10/W13), so it has
/// to survive with it whatever the follower does — including on the refusal
/// paths, which is where the walk ruled the verification worth the most.
///
/// **Both flags reach both outcomes since S2**, so all three combinations are
/// exercised: `--as md1` refusing an (a″) wallet, `--as descriptor` PACKING the
/// same wallet (the carriage §5.3's window used to describe as future), and
/// `--as descriptor` refusing a `multi` on conjunct 1's permanent ground.
#[test]
fn the_compare_prompt_survives_whatever_the_follower_does() {
    let a2 = format!("wsh(sortedmulti(2,{IK1}/<2;3>,{IK2}/<0;1>/*))");
    let cases = [
        (
            "--as md1, an (a″) refusal",
            a2.clone(),
            vec!["--as", "md1"],
            3,
        ),
        (
            "--as descriptor, the same wallet PACKED",
            a2,
            vec!["--as", "descriptor"],
            0,
        ),
        (
            "--as descriptor, conjunct 1's permanent multi refusal",
            row_input("neither/wsh-multi-fixed-path"),
            vec!["--as", "descriptor"],
            3,
        ),
    ];
    let mut checked = 0;
    for (what, document, flags, exit) in cases {
        let (out, _d) = pack_in(&document, &flags);
        let err = stderr(&out);
        assert_eq!(code(&out), exit, "{what}\n{err}");
        assert!(
            err.contains(
                "compare against your wallet software's first receive address before engraving."
            ),
            "{what}: the compare prompt is missing:\n{err}"
        );
        checked += 1;
    }
    assert_eq!(checked, 3, "all three follower outcomes are exercised");
}

/// **The FULL tier always yields an address.** §5.4's `wallet-id: none` line
/// tells the operator to *"identify it … by address 0"*, so a FULL-tier block
/// that cannot print one is a dead end. Asserted over every vector row rather
/// than argued: if the per-key walk ever returns `None` in FULL tier, this reds.
#[test]
fn every_full_tier_wallet_has_an_address_0() {
    let raw = std::fs::read(VECTORS).unwrap();
    let doc: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let mut full = 0usize;
    for r in doc["vectors"].as_array().unwrap() {
        let input = r["input"].as_str().unwrap();
        let Ok(parsed) = mnemonic_engrave::descriptor::cascade::cascade(
            &mnemonic_engrave::descriptor::cascade::normalise(input),
        ) else {
            continue;
        };
        if mnemonic_engrave::descriptor::admit::admit(
            &parsed,
            mnemonic_engrave::descriptor::Path::Md1,
        )
        .is_err()
        {
            continue; // PARTIAL tier: no `address 0:` line at all.
        }
        assert!(
            mnemonic_engrave::descriptor::derive::address_0(&parsed).is_some(),
            "{}: FULL tier with no derivable address 0",
            r["name"].as_str().unwrap()
        );
        full += 1;
    }
    assert!(
        full >= 20,
        "only {full} FULL-tier rows — the loop has gone vacuous"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// M1 / M2 / M3 / N2 — the review's minors and nits
// ───────────────────────────────────────────────────────────────────────────

/// **M1, flipped by S2.** §5.1's choice block marks a build-dead value inline
/// and sends the operator to `me sysw pack --help` for the comparison, so the
/// help's possible-value list has to carry the SAME marking — which since S2
/// means neither of them marks anything, because both paths ship. The test is
/// kept in the flipped direction rather than deleted: it is what would catch a
/// marking that outlived its condition, in either surface, and the two are
/// gated on the same two constants.
#[test]
fn neither_the_help_nor_the_choice_block_marks_a_value_this_build_carries() {
    let out = assert_cmd::Command::cargo_bin("me")
        .unwrap()
        .args(["sysw", "pack", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8_lossy(&out.stdout).to_string();
    // Both values present and UNMARKED. The trailing newline on each is what
    // makes "unmarked" an assertion rather than a substring that a marked line
    // would also satisfy.
    assert!(
        help.contains(
            "- descriptor: The canonical re-encoded descriptor, as one `Descriptor` record\n"
        ),
        "the help does not carry `descriptor` unmarked:\n{help}"
    );
    assert!(
        help.contains(
            "- md1:        The BIP-388 decomposition, as md1 text cards (`MdMk` records)\n"
        ),
        "the help does not carry `md1` unmarked:\n{help}"
    );
    assert!(
        !help.contains("(not available in this build)"),
        "the help build-marks a value this build carries:\n{help}"
    );
    // The choice block's marking and the help's are gated on the same constants
    // — assert they agree rather than trusting that they do.
    let (block, _d) = pack_in(
        &row_input("formats-happy/bip380-sortedmulti-multipath"),
        &[],
    );
    assert!(
        !stderr(&block).contains("(not available in this build)"),
        "the choice block and the help disagree about what this build carries"
    );
}

/// **M2.** The label warning echoes the operator's own `Name:` header, and it
/// lands beside `address 0:` — the verification surface. A crafted export
/// carrying cursor or clear-screen sequences must not be able to move that
/// line.
#[test]
fn the_label_warning_neither_emits_control_bytes_nor_runs_long() {
    let hostile = format!("\u{1b}[2J\u{1b}[31mRED\u{1b}[0m{}", "A".repeat(400));
    let document = row_input("formats-happy/bluewallet-sh-fixture")
        .replace("Name: sh", &format!("Name: {hostile}"));
    let (out, _d) = pack_in(&document, &["--as", "md1"]);
    let err = stderr(&out);
    assert_eq!(code(&out), 0, "the wallet still packs\n{err}");

    let warning = err
        .lines()
        .find(|l| l.contains("warning: the wallet's name"))
        .unwrap_or_else(|| panic!("no label warning:\n{err}"));
    assert!(
        !warning.chars().any(|c| c.is_control()),
        "a control byte reached the verification surface: {warning:?}"
    );
    assert!(
        warning.contains("\\x1b"),
        "the escape was stripped rather than shown -- the operator should see \
         that something odd is in their file: {warning:?}"
    );
    assert!(
        warning.chars().count() < 220,
        "the warning is {} characters; a long label can push `address 0:` off \
         the screen",
        warning.chars().count()
    );
    // The line it protects is still there, after it.
    assert!(err.contains("address 0: bc1qtahtpjkgtljxl20j"), "{err}");
}

/// **M3 (controller ruling).** The label warning's text is a statement about
/// what was just packed, so it prints EXACTLY on the paths that pack. One test,
/// all three paths, presence and absence both pinned.
///
/// **S2 moved one path across the rule, not the rule.** `--as descriptor` used
/// to be a window refusal and printed no warning; it packs now, so it prints
/// one — which is §5.5's "carries a label | text only, dropped" stated at the
/// moment it happens. The `--as`-omitted choice block still packs nothing and
/// still says nothing, which is what keeps the assertion two-sided.
#[test]
fn the_label_warning_fires_exactly_on_the_paths_that_pack() {
    let document = row_input("formats-happy/bluewallet-sh-fixture");
    let cases = [
        ("--as md1 (packs)", vec!["--as", "md1"], 0, true),
        (
            "--as descriptor (packs, S2)",
            vec!["--as", "descriptor"],
            0,
            true,
        ),
        ("--as omitted (choice block)", vec![], 2, false),
    ];
    let mut checked = 0;
    for (what, flags, exit, want_warning) in cases {
        let (out, _d) = pack_in(&document, &flags);
        let err = stderr(&out);
        assert_eq!(code(&out), exit, "{what}: exit\n{err}");
        assert_eq!(
            err.contains("warning: the wallet's name"),
            want_warning,
            "{what}: the label warning's presence is wrong\n{err}"
        );
        checked += 1;
    }
    assert_eq!(checked, 3, "all three paths are exercised");
}

/// Backticks that DELIMIT, i.e. discounting the ones `quote_operator` escapes.
///
/// The naive count is not the property, and the first version of this test
/// asserted the naive one and failed on its own fixture: `` `a\`b` `` carries
/// three backticks and is correctly rendered — two delimiters plus one escaped
/// literal. What has to pair up is the DELIMITERS.
fn delimiting_backticks(line: &str) -> usize {
    let b: Vec<char> = line.chars().collect();
    (0..b.len())
        .filter(|i| b[*i] == '`' && (*i == 0 || b[i - 1] != '\\'))
        .count()
}

/// **N2, and its residual N-a.** A quoted fragment must not span a newline, and
/// must not contain an unescaped copy of the delimiter it sits inside — the
/// backtick pair has to close on the line it opened.
///
/// **Scope, stated because the fold-1 report over-claimed it:** the parity loop
/// below runs over the stderr of the inputs THIS test drives, not over every
/// line `me` can emit. A `quote_operator`-quoted fragment cannot unbalance a
/// pair, because it now escapes `` ` `` — but other refusal texts carry
/// backticks in fixed prose, and one surface still quotes an operator line
/// through `elide_line` rather than through `quote_operator` (the cosigner-line
/// row). The general property is NOT asserted here, and is not claimed.
///
/// And the property that IS asserted counts DELIMITING backticks, not all of
/// them — see [`delimiting_backticks`].
///
/// The row and the exit code are unchanged in both cases; only the rendering.
#[test]
fn a_quoted_fragment_never_spans_a_newline() {
    let one = row_input("neither/wsh-multi");
    let document = format!("{one}\n{one}");
    let (out, _d) = pack_in(&document, &["--as", "md1"]);
    let err = stderr(&out);
    assert_eq!(
        code(&out),
        3,
        "two descriptors are one malformed document\n{err}"
    );
    assert!(
        err.contains("not a wallet descriptor in any of the four forms"),
        "{err}"
    );
    for line in err.lines() {
        assert_eq!(
            delimiting_backticks(line) % 2,
            0,
            "a backtick pair is left open across a line break: {line:?}"
        );
    }
    assert!(
        err.contains("\\n"),
        "the newline inside the quoted fragment was neither escaped nor \
         truncated away:\n{err}"
    );
}

/// The bare payload of the first I1 key, for constructing key expressions whose
/// origin block is written inline.
const IK1_BARE: &str = "xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan";

/// **N-a** — the residual of N2. One backtick INSIDE the fragment falsified the
/// same property N2 installed, from an operator-supplied path that
/// `quote_operator` used to pass through untouched.
#[test]
fn a_backtick_inside_a_quoted_fragment_is_escaped() {
    let document = format!("wpkh([dc567276/48h/0h/0h/2h]{IK1_BARE}/a`b)");
    let (out, _d) = pack_in(&document, &["--as", "md1"]);
    let err = stderr(&out);
    assert_eq!(
        code(&out),
        3,
        "the row and the exit code are unchanged\n{err}"
    );
    assert!(
        err.contains("the use-site path is not a path: `a\\`b`."),
        "the backtick inside the fragment is not escaped:\n{err}"
    );
    for line in err.lines() {
        assert_eq!(
            delimiting_backticks(line) % 2,
            0,
            "a backtick pair is left open: {line:?}"
        );
    }
}

/// **M-A.** `quote_operator` neutralised C0/C1 but not the two other classes
/// that can rewrite the line it prints: the DELIMITER it is interpolated into,
/// and Unicode `Cf` (bidi overrides and isolates), which `char::is_control()`
/// does not reach. Both constructions are the fold-1 re-review's own.
#[test]
fn a_quoted_label_can_neither_close_the_quote_nor_reorder_the_line() {
    let fixture = row_input("formats-happy/bluewallet-sh-fixture");
    let label_of = |l: &str| fixture.replace("Name: sh", &format!("Name: {l}"));

    // (a) A label that closes the quote and continues in `me`'s own voice.
    let (out, _d) = pack_in(
        &label_of("ok\" -- nothing is wrong with this wallet. \""),
        &["--as", "md1"],
    );
    let err = stderr(&out);
    assert!(
        err.contains(
            "the wallet's name \"ok\\\" -- nothing is wrong with this wallet. \\\"\" is only a label"
        ),
        "the label closed the quote and continued in me's voice:\n{err}"
    );

    // (b) Bidi overrides and isolates. `a\u{202e}KCATTA` renders as `aATTACK`
    // in a bidi-aware terminal, and an unterminated override reorders the
    // remainder of the line.
    let (out, _d) = pack_in(
        &label_of("a\u{202e}KCATTA\u{202c}b\u{200b}\u{2066}x\u{2069}"),
        &["--as", "md1"],
    );
    let err = stderr(&out);
    for bad in ['\u{202e}', '\u{202c}', '\u{200b}', '\u{2066}', '\u{2069}'] {
        assert!(
            !err.contains(bad),
            "a raw U+{:04X} reached the terminal:\n{err}",
            bad as u32
        );
    }
    assert!(
        err.contains("\\u{202e}") && err.contains("\\u{2066}"),
        "the formatting characters were stripped rather than shown -- the \
         operator should see that something odd is in their file:\n{err}"
    );

    // The CONTROL, and it is what stops the fix from being "escape everything":
    // the operator's job here is to RECOGNISE their own label.
    let (out, _d) = pack_in(&label_of("Grüße — Konto Nº1 ✓ 日本語"), &["--as", "md1"]);
    let err = stderr(&out);
    assert!(
        err.contains("the wallet's name \"Grüße — Konto Nº1 ✓ 日本語\" is only a label"),
        "a legitimate non-ASCII label was mangled:\n{err}"
    );
}
