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
