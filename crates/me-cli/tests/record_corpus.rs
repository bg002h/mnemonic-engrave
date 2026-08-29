//! **Invariant 2, as a gate: S2 does not move the record surface.**
//!
//! S2 teaches `sysw::classify` a `Descriptor` arm and moves §5.1's gate ahead
//! of `admit_check`. Both changes are safe only if a record that was already
//! placeable keeps its class, and only if the gate stays SHUT on every input
//! that is a record rather than a descriptor. Arm order alone does not give
//! either: order protects a record an earlier arm matched, not a record that
//! used to fall through to `Unknown`, and the gate now runs on inputs it never
//! saw before.
//!
//! So both halves are pinned to a CAPTURE taken before the arm existed —
//! `testdata/record_corpus_pre_s2.json`, committed with the gate restructure
//! (P1.0) so its `class` column is genuinely pre-S2. A change to that file is a
//! change to invariant 2 and has to be argued for in the diff, which is the
//! point of capturing it rather than recomputing it.
//!
//! # The corpus
//!
//! Every record the shipped classify tests already run over, in one place:
//! `testdata/sysw_vectors.json`'s records (the packing corpus),
//! `testdata/codex32_seam_vectors.json`'s strings (the codex32 seam), and the
//! literal records the class-specific tests carry inline, each cited at its
//! entry in [`LITERALS`]. The literals are not decoration: without them no
//! data file in this crate carries an `mt1` chunk, a `tx:` record or a
//! `pass:` record, and invariant 2 names all three.

use std::collections::BTreeSet;

use mnemonic_engrave::sysw::record::Class;

const CAPTURE: &str = "testdata/record_corpus_pre_s2.json";
const SYSW_VECTORS: &str = "testdata/sysw_vectors.json";
const CODEX32_VECTORS: &str = "testdata/codex32_seam_vectors.json";
const DESCRIPTOR_VECTORS: &str = "testdata/descriptor_seam_vectors.json";

/// Records no data file in this crate carries, taken verbatim from the tests
/// that already classify them. The cite is the record's provenance: a literal
/// with no home elsewhere in the suite would be a fixture invented here, which
/// is exactly what a capture of the EXISTING surface must not contain.
const LITERALS: &[(&str, &str)] = &[
    // `mt1` — the "even" set from mt-codec's pinned corpus, src/sysw/mt.rs:256.
    ("mt.rs/even#0", "mt1p9h8jqq9qqqqgqqqqqqqyqherdfykhhpey6z2cvafak8804qd7g0dl6v8ex9wr2cvky023skwkeud2229sax"),
    ("mt.rs/even#1", "mt1p9h8jqq9qqphgdqqqqqqqq0mllllupyqj6vqqqqqqqqzcqpfsw7ph2rt5w54kt768636cls8zxg0najlzunp"),
    ("mt.rs/even#2", "mt1p9h8jqq9qqzj8yqpnzw4vl2rwffqyqqqqqkqq282yyhc2vavd20hvk94pz39hts3u5s9a0qd8pwskxfl7ju5"),
    ("mt.rs/even#3", "mt1p9h8jqq9qqrqfrnq3qzyp77h37cnxzvwutegzmzy5zrrrfvrpykdfsckvk03dcq6rcjtvlsfcglv7zx43yaz"),
    ("mt.rs/even#4", "mt1p9h8jqq9qqylgpzqmhcwhuupdvnrc82rncvzzdahpgjsdwgu52jd7vmxsve9x3w5ujeqyssuvddxvwqze4ve"),
    ("mt.rs/even#5", "mt1p9h8jqq9qq9qdcc7h75twfxyf340c4sgqzhfdq6xtgt7zhxngpwa049l0z59l6jqcqqqqqq5k5y2ye5nv8yf"),
    // `tx:` and `pass:` — tests/argv_secret_guard.rs:55,59.
    ("argv_secret_guard.rs/TX", "tx:020000000001017c8da925af70e49a12b0cea7b639df5037c87b7fa61f262b86ac32c47aa3ba1a0000000000fdffffff02404b4c0000000000160014c1de0dd435d1d4ad97ed1f51d63f91c800cc4eab3ea1b92901000000160014751097c299d6354fbb2c5a84512dd708f2902f5e0247304402207debc7d89984c7717940b622504318d2c184966a618b32cf8b700d0f125b3ffa02206ef875f9c0b5931e0ea1cf0c109bdb8512835c8e51526f99b3419929a2ea7259012103718f5fd45b926226357e2b0400574b41a32d0bf0ae69a02eebea5fbc542ff52060000000"),
    ("argv_secret_guard.rs/PASS", "pass:6869"),
    // The eight shapes tests/ms_remedy_runs.rs:162 runs the pack path over —
    // five of them `Unknown`, and `Unknown` is the class the descriptor arm
    // takes records FROM, so it is the one that has to be captured densely.
    ("ms_remedy_runs.rs/ms1-lower", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"),
    ("ms_remedy_runs.rs/ms1-spaced-upper", " MS10ENTRSQQQQQQQQQQQQQQQQQQQQQQQQQQQQCJ9SXRAQ34V7F "),
    ("ms_remedy_runs.rs/pass-plain-body", "pass:hunter2 correct horse"),
    ("ms_remedy_runs.rs/text-hello", "text:hello"),
    ("ms_remedy_runs.rs/tx-truncated", "tx:0200000001"),
    ("ms_remedy_runs.rs/tx-spaced-upper", " TX:0200000001"),
    ("ms_remedy_runs.rs/tx-non-hex", "tx:zzznothex"),
];

/// Exhaustive on purpose: a new [`Class`] variant reds this file rather than
/// slipping into the capture under a name nothing checks.
fn class_name(c: Class) -> &'static str {
    match c {
        Class::Mnemonic => "Mnemonic",
        Class::Codex32Secret => "Codex32Secret",
        Class::Passphrase => "Passphrase",
        Class::FreeText => "FreeText",
        Class::Descriptor => "Descriptor",
        Class::MdMk => "MdMk",
        Class::Mt => "Mt",
        Class::Tx => "Tx",
        Class::Address => "Address",
        Class::Unknown => "Unknown",
    }
}

fn json(path: &str) -> serde_json::Value {
    let raw = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    serde_json::from_slice(&raw).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// `(origin, record)` for the whole corpus, deduplicated on the RECORD and in a
/// fixed order — the capture is compared to this list position by position, so
/// a dropped source shows up as a diff rather than as a shorter loop.
fn corpus() -> Vec<(String, String)> {
    let mut seen = BTreeSet::new();
    let mut out: Vec<(String, String)> = Vec::new();
    let mut push = |origin: String, record: String| {
        if seen.insert(record.clone()) {
            out.push((origin, record));
        }
    };
    for v in json(SYSW_VECTORS).as_array().unwrap() {
        let name = v["name"].as_str().unwrap();
        for (i, r) in v["records"].as_array().unwrap().iter().enumerate() {
            push(
                format!("sysw_vectors/{name}#{i}"),
                r.as_str().unwrap().to_string(),
            );
        }
    }
    for r in json(CODEX32_VECTORS)["vectors"].as_array().unwrap() {
        push(
            format!("codex32_seam/{}", r["name"].as_str().unwrap()),
            r["string"].as_str().unwrap().to_string(),
        );
    }
    for (origin, record) in LITERALS {
        push((*origin).to_string(), (*record).to_string());
    }
    out
}

fn capture() -> Vec<serde_json::Value> {
    json(CAPTURE)["records"].as_array().unwrap().clone()
}

/// The capture is not allowed to be a subset of the corpus it claims to cover.
#[test]
fn the_capture_is_the_whole_corpus() {
    let want: Vec<(String, String)> = corpus();
    let got: Vec<(String, String)> = capture()
        .iter()
        .map(|e| {
            (
                e["origin"].as_str().unwrap().to_string(),
                e["record"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(want, got, "{CAPTURE} is not the enumerated corpus");
    assert!(want.len() >= 30, "corpus went thin: {} records", want.len());
}

/// **Invariant 2's first half.** Not one of these records may change class.
#[test]
fn every_corpus_record_classifies_as_it_did_before_s2() {
    let mut checked = 0usize;
    for e in capture() {
        let record = e["record"].as_str().unwrap();
        assert_eq!(
            class_name(mnemonic_engrave::sysw::classify(record)),
            e["class"].as_str().unwrap(),
            "{}: class moved under S2",
            e["origin"].as_str().unwrap()
        );
        checked += 1;
    }
    assert_eq!(checked, corpus().len(), "class assertions run");
}

/// **The mirror of the arm question** (P1.0): the arm asks whether a
/// descriptor can classify as an existing class; this asks whether `identify`
/// can claim a document whose records already classify. It has to answer no,
/// because §5.1's gate now runs BEFORE `admit_check` — an input it claimed
/// would be refused where it used to be packed.
#[test]
fn the_descriptor_gate_stays_shut_on_every_corpus_record() {
    let mut checked = 0usize;
    for e in capture() {
        let record = e["record"].as_str().unwrap().to_string();
        let outcome = mnemonic_engrave::descriptor::consult(&record, std::slice::from_ref(&record));
        assert_eq!(
            outcome.class(),
            e["consult"].as_str().unwrap(),
            "{}: the descriptor gate changed its mind about a record",
            e["origin"].as_str().unwrap()
        );
        checked += 1;
    }
    assert_eq!(checked, corpus().len(), "gate assertions run");
}

/// The same question at DOCUMENT scope, because that is the scope `consult`
/// is actually called at: `gate_opens` tests T1–T3 per line and T4 over the
/// whole input, so a multi-record document is not the sum of its records.
#[test]
fn the_descriptor_gate_stays_shut_on_every_corpus_document() {
    let mut checked = 0usize;
    for v in json(SYSW_VECTORS).as_array().unwrap() {
        let records: Vec<String> = v["records"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.as_str().unwrap().to_string())
            .collect();
        let document = records.join("\n");
        assert_eq!(
            mnemonic_engrave::descriptor::consult(&document, &records).class(),
            "record-refusal",
            "{}: a container of records opened the descriptor gate",
            v["name"].as_str().unwrap()
        );
        checked += 1;
    }
    assert!(checked >= 8, "document assertions run: {checked}");
}

/// Non-vacuity, in the shape the codex32 seam test uses: a capture missing a
/// class cannot detect that class moving, and `Unknown` matters most — it is
/// the class the descriptor arm takes records FROM.
#[test]
fn the_capture_covers_every_class_s2_must_not_move() {
    let present: BTreeSet<&str> = capture()
        .iter()
        .map(|e| e["class"].as_str().unwrap())
        .collect::<Vec<_>>()
        .iter()
        .map(|s| match *s {
            "Mnemonic" => "Mnemonic",
            "Codex32Secret" => "Codex32Secret",
            "Passphrase" => "Passphrase",
            "FreeText" => "FreeText",
            "MdMk" => "MdMk",
            "Mt" => "Mt",
            "Tx" => "Tx",
            "Unknown" => "Unknown",
            other => panic!("{CAPTURE}: unexpected class {other}"),
        })
        .collect();
    for want in [
        "Mnemonic",
        "Codex32Secret",
        "Passphrase",
        "FreeText",
        "MdMk",
        "Mt",
        "Tx",
        "Unknown",
    ] {
        assert!(present.contains(want), "{CAPTURE} carries no {want} record");
    }
}

/// **`--expect` resolves BEFORE the gate, and that ordering is a decision.**
/// The gate moved up to sit immediately before `admit_check`; one step further
/// and it would overtake `--expect`, turning an operator's stated, unmet
/// expectation (exit 4, and it names the kind) into a menu about how to pack
/// the file (exit 2).
#[test]
fn expect_resolves_before_the_descriptor_gate() {
    let doc = json(DESCRIPTOR_VECTORS)["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"].as_str().unwrap() == "formats-happy/bip380-sortedmulti-multipath")
        .map(|r| r["input"].as_str().unwrap().to_string())
        .expect("the multipath happy row");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("descriptor.txt");
    std::fs::write(&path, &doc).unwrap();
    let out = assert_cmd::Command::cargo_bin("me")
        .unwrap()
        .args(["sysw", "pack", "--no-passphrase", "--expect", "mnemonic"])
        .arg("--in")
        .arg(&path)
        .arg("--out")
        .arg(dir.path().join("out.bin"))
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert_eq!(out.status.code(), Some(4), "stderr: {err}");
    assert!(err.contains("a BIP-39 mnemonic"), "stderr: {err}");
    assert!(
        !err.contains("`--as` decides how it is packed"),
        "the gate overtook --expect: {err}"
    );
}
