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

/// `descriptor` is a VALUE the flag accepts, even in a build whose descriptor
/// path has not shipped: §5.1's window is a REFUSAL at 3, never a usage error
/// about an unknown value.
#[test]
fn as_descriptor_is_a_value_the_flag_accepts() {
    let (out, _d) = pack_in(
        &row_input("formats-happy/bip380-sortedmulti-multipath"),
        &["--as", "descriptor"],
    );
    let err = stderr(&out);
    assert!(
        !err.contains("unexpected argument") && !err.contains("invalid value"),
        "`--as descriptor` is not an accepted value:\n{err}"
    );
    assert_eq!(code(&out), 3, "the window is a refusal. stderr:\n{err}");
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
                    "warning: the label \"{l}\" is not carried by any record format"
                )),
                "{}: no label warning. stderr:\n{err}",
                e.what
            ),
            None => assert!(
                !err.contains("warning: the label"),
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
/// to survive with it — including on the refusal paths, which is where the walk
/// ruled the verification worth the most.
#[test]
fn the_compare_prompt_survives_on_a_refusal_path() {
    let document = format!("wsh(sortedmulti(2,{IK1}/<2;3>,{IK2}/<0;1>/*))");
    for flags in [vec!["--as", "md1"], vec!["--as", "descriptor"]] {
        let (out, _d) = pack_in(&document, &flags);
        let err = stderr(&out);
        assert_eq!(code(&out), 3, "this wallet is md1-unrepresentable\n{err}");
        assert!(
            err.contains(
                "compare against your wallet software's first receive address before engraving."
            ),
            "{flags:?}: the compare prompt is missing:\n{err}"
        );
    }
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
