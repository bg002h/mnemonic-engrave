//! **The host/device DESCRIPTOR seam, as a gate.**
//!
//! Two independent descriptor parsers will drift. `me`'s (built in S1) and the
//! Go one that ships on the device are the same question asked in two
//! languages, and the direction that matters is asymmetric: a host that admits
//! what the device refuses packs a payload the device cannot read — an
//! engraved plate for a wallet that will not load.
//!
//! This half reads `testdata/descriptor_seam_vectors.json` and asserts the
//! **host** columns; the fork's `nonstandard/descriptor_seam_test.go` reads a
//! BYTE-IDENTICAL copy and asserts the **device** columns. Neither
//! implementation is ever compared to the other — both are compared to the
//! file, which is why it has to be the same file. [`SEAM_VECTORS_SHA256`] is
//! what makes that structural.
//!
//! **What is green in P0 and what is not.** The parser does not exist yet, so
//! every assertion that would call it is ignore-tagged and each ignore reason
//! names the phase that removes it. What runs today is the file's own
//! integrity: the pin, the schema, the coverage manifest's arithmetic, the
//! per-column population counts, and requirement 5's non-vacuity. Those are
//! not filler — the manifest is what stops a required row being dropped and
//! counted around, and the population counts are what make a mistyped field
//! name red the suite instead of silently disabling an assertion.
//!
//! **P2's gate is that this file carries ZERO ignore attributes:**
//!
//! ```text
//! grep -c '^#.ignore' crates/me-cli/tests/descriptor_seam.rs   -> must be 0
//! ```
//!
//! The anchor is load-bearing. Attributes sit at column 0 and comment lines
//! never do, so the gate cannot match its own prose — a gate whose
//! documentation matches its own grep can never reach zero, and the fix a
//! future implementer reaches for is to weaken the grep rather than to finish
//! the work.

use mnemonic_engrave::sysw::record::Class;
use sha2::Digest as _;

/// The sha256 of `testdata/descriptor_seam_vectors.json`, pinned IDENTICALLY
/// in the fork's `nonstandard/descriptor_seam_test.go`. Changing a row means
/// changing this in both repos, which is the point — see the file's own
/// header, and `scripts/descriptor-seam-vectors/README.md` for the regenerate
/// + re-pin recipe.
const SEAM_VECTORS_SHA256: &str =
    "542cd492e35149b62c53f940fb755576e0ffd4d086b0e3fcda615fbc43f51974";

const PATH: &str = "testdata/descriptor_seam_vectors.json";

// ── The coverage manifest — SPEC_descriptor_input.md §7, NORMATIVE ──────────
// §11 item 3 counts against this table, not against a reading of the file.
const MANIFEST: &[(&str, usize)] = &[
    ("formats-happy", 4),
    ("promotion-near-miss", 15),
    ("narrowed-4.7", 14),
    ("accepted-extreme", 1),
    ("narrowed-4.2", 5),
    ("neither", 3),
    ("whitespace", 3),
    ("md1-splits", 6),
    ("gate", 37),
];
/// The minima sum to 88 tag-slots.
const TAG_SLOTS: usize = 88;
/// 88 − 17 overlap slots = the physical-row floor.
const ROW_FLOOR: usize = 71;
/// The fifteen §4.5 rows carry `gate` as a second tag …
const SECOND_TAGGED: usize = 15;
/// … and exactly two of them carry a third (the original overlap pair).
const THIRD_TAGGED: usize = 2;

/// Every key a row may carry. An unknown key is a typo or an un-specified
/// field, and either way it must red the suite rather than sit unread.
const KNOWN_ROW_KEYS: &[&str] = &[
    "name",
    "input",
    "sha256",
    "host_admits",
    "device_admits",
    "md1_admits",
    "format",
    "source",
    "covers",
    "canonical",
    "sysw_class",
    "device_probe",
    "address_0",
    "address_1",
    "md_descriptor_contains",
    "wallet_id",
    "gate_open",
    "outcome",
    "refusal_row",
    "exit_code",
];

const KNOWN_FORMATS: &[&str] = &["bluewallet", "bip380", "json", "promoted-key", "none"];
const KNOWN_OUTCOMES: &[&str] = &[
    "record-refusal",
    "as-decides",
    "descriptor-refusal",
    "multi-record",
];
const KNOWN_PROBES: &[&str] = &["panic:parse", "panic:encode"];

// ── Per-column POPULATION counts ────────────────────────────────────────────
// PLAN-r1's I7. Each number is the count of rows carrying that column, so a
// field renamed by a typo drops its count and reds the suite instead of
// silently disabling every assertion that reads it. Re-derive with
// `scripts/descriptor-seam-vectors/gen.py`, never by hand.
struct Pop {
    rows: usize,
    host_admits_true: usize,
    md1_admits_true: usize,
    device_admits_true: usize,
    device_admits_false: usize,
    device_admits_absent: usize,
    canonical: usize,
    address_0: usize,
    address_1: usize,
    wallet_id: usize,
    md_descriptor_contains: usize,
    sysw_class: usize,
    device_probe: usize,
    gate_fields: usize,
    refusal_row: usize,
    /// Rows where BOTH routes derive `address_0`, so the two are asserted to
    /// the same value — §5.3(a′)'s materialisation claim at the address layer.
    both_routes_address_0: usize,
}
const POP: Pop = Pop {
    rows: 71,
    host_admits_true: 19,
    md1_admits_true: 15,
    device_admits_true: 37,
    device_admits_false: 33,
    device_admits_absent: 1,
    canonical: 19,
    address_0: 20,
    address_1: 5,
    wallet_id: 4,
    md_descriptor_contains: 1,
    sysw_class: 4,
    device_probe: 3,
    gate_fields: 37,
    refusal_row: 18,
    both_routes_address_0: 11,
};

fn doc() -> serde_json::Value {
    let raw = std::fs::read(PATH).unwrap_or_else(|e| panic!("{PATH}: {e}"));
    assert_eq!(
        format!("{:x}", sha2::Sha256::digest(&raw)),
        SEAM_VECTORS_SHA256,
        "{PATH} is not the file the fork's copy is pinned to; re-pin BOTH literals"
    );
    serde_json::from_slice(&raw).unwrap()
}

fn rows(d: &serde_json::Value) -> &Vec<serde_json::Value> {
    d["vectors"].as_array().unwrap()
}

fn name(r: &serde_json::Value) -> &str {
    r["name"].as_str().unwrap()
}

fn covers(r: &serde_json::Value) -> Vec<&str> {
    r["covers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap())
        .collect()
}

fn has(r: &serde_json::Value, k: &str) -> bool {
    r.get(k).is_some()
}

#[test]
fn the_file_is_the_one_the_fork_pins() {
    let d = doc();
    assert_eq!(rows(&d).len(), POP.rows, "physical row count");
    assert_eq!(
        d["invariant"].as_str().unwrap(),
        "host_admits(input) => device_admits(canonical(input))",
        "the invariant statement is part of the file's contract"
    );
}

/// A mistyped vector must fail loudly, not quietly stop testing. The codex32
/// file uses a character count; a descriptor is long enough that a count would
/// not catch a transcription error inside an xpub, so this is a digest.
#[test]
fn every_row_pins_the_digest_of_its_own_input() {
    let d = doc();
    for r in rows(&d) {
        let input = r["input"].as_str().unwrap();
        assert_eq!(
            format!("{:x}", sha2::Sha256::digest(input.as_bytes())),
            r["sha256"].as_str().unwrap(),
            "{}: input does not hash to its declared sha256",
            name(r)
        );
    }
}

#[test]
fn the_row_schema_holds_on_every_row() {
    let d = doc();
    let slugs = d["refusal_rows"].as_object().unwrap();
    let mut names = std::collections::BTreeSet::new();
    for r in rows(&d) {
        let n = name(r);
        assert!(names.insert(n.to_string()), "{n}: duplicate row name");
        let obj = r.as_object().unwrap();
        for k in obj.keys() {
            assert!(
                KNOWN_ROW_KEYS.contains(&k.as_str()),
                "{n}: unknown row key {k:?} — a typo, or a field nobody specified"
            );
        }
        // REQUIRED on every row, no defaults.
        for k in [
            "name",
            "input",
            "sha256",
            "host_admits",
            "md1_admits",
            "format",
            "covers",
            "source",
        ] {
            assert!(has(r, k), "{n}: missing required key {k:?}");
        }
        assert!(r["host_admits"].is_boolean(), "{n}: host_admits");
        assert!(r["md1_admits"].is_boolean(), "{n}: md1_admits");
        assert!(
            KNOWN_FORMATS.contains(&r["format"].as_str().unwrap()),
            "{n}: unknown format {:?}",
            r["format"]
        );

        // `covers` is non-empty and its entries are distinct within the row.
        let c = covers(r);
        assert!(!c.is_empty(), "{n}: covers is empty");
        let uniq: std::collections::BTreeSet<_> = c.iter().collect();
        assert_eq!(uniq.len(), c.len(), "{n}: duplicate tag in covers {c:?}");
        for t in &c {
            assert!(
                MANIFEST.iter().any(|(m, _)| m == t),
                "{n}: unknown coverage tag {t:?}"
            );
        }

        // `canonical` is REQUIRED wherever host_admits is true, and meaningless
        // otherwise: a row where the host is WIDER than the device and
        // `canonical` is absent is the defect the invariant exists to catch.
        assert_eq!(
            has(r, "canonical"),
            r["host_admits"].as_bool().unwrap(),
            "{n}: `canonical` must be present iff host_admits"
        );

        // `device_probe` "panic:parse" OMITS device_admits — the predicate
        // cannot be evaluated without panicking the parser, so either boolean
        // would be a false claim.
        let probe = r.get("device_probe").and_then(|p| p.as_str());
        if let Some(p) = probe {
            assert!(KNOWN_PROBES.contains(&p), "{n}: unknown device_probe {p:?}");
        }
        assert_eq!(
            has(r, "device_admits"),
            probe != Some("panic:parse"),
            "{n}: device_admits presence must track the panic:parse marker"
        );

        // The four gate fields: REQUIRED on every `gate`-tagged row, absent
        // elsewhere.
        let gated = c.contains(&"gate");
        for k in ["gate_open", "outcome", "exit_code"] {
            assert_eq!(
                has(r, k),
                gated,
                "{n}: {k} must be present iff `gate`-tagged"
            );
        }
        if gated {
            let outcome = r["outcome"].as_str().unwrap();
            assert!(
                KNOWN_OUTCOMES.contains(&outcome),
                "{n}: unknown outcome {outcome:?}"
            );
            let exit = r["exit_code"].as_u64().unwrap();
            let want = match outcome {
                "as-decides" => 2,
                "descriptor-refusal" => 3,
                "record-refusal" | "multi-record" => 4,
                _ => unreachable!(),
            };
            assert_eq!(exit, want, "{n}: outcome {outcome} does not exit {want}");
            // `refusal_row` on the descriptor-refusal and multi-record
            // outcomes ONLY, and always a slug the file itself defines —
            // PLAN-r4's NEW-M6: P2.4's per-row text tests must not have to
            // guess the vocabulary P0 invented.
            let needs = matches!(outcome, "descriptor-refusal" | "multi-record");
            assert_eq!(
                has(r, "refusal_row"),
                needs,
                "{n}: refusal_row must be present iff the outcome names a §6 row"
            );
            if needs {
                let slug = r["refusal_row"].as_str().unwrap();
                assert!(
                    slugs.contains_key(slug),
                    "{n}: refusal_row {slug:?} is not in the file's `refusal_rows` vocabulary"
                );
            }
            // A record-refusal row is a CLOSED gate and vice versa: invariant 1
            // (no record-shaped input hears descriptor vocabulary) and
            // invariant 2 (every admitted descriptor spelling reaches the
            // descriptor surfaces) are the two halves of that equivalence.
            assert_eq!(
                r["gate_open"].as_bool().unwrap(),
                outcome != "record-refusal",
                "{n}: gate_open and the outcome class disagree"
            );
        } else {
            assert!(
                !has(r, "refusal_row"),
                "{n}: refusal_row outside a gate row"
            );
        }

        // The two value fields the Rust side owns are only meaningful on a row
        // the md1 path carries.
        if has(r, "md_descriptor_contains") {
            assert!(
                r["md1_admits"].as_bool().unwrap(),
                "{n}: an md1 read-back pin on a row md1 does not carry"
            );
        }
    }
    // Every slug in the vocabulary is spelled once, and the vocabulary covers
    // §6's 36 data rows.
    assert_eq!(slugs.len(), 36, "the §6 refusal vocabulary is 36 rows");
}

#[test]
fn the_coverage_manifest_is_met_by_count_not_by_reading() {
    let d = doc();
    let rs = rows(&d);
    let mut got: std::collections::BTreeMap<&str, usize> = Default::default();
    for r in rs {
        for t in covers(r) {
            *got.entry(t).or_default() += 1;
        }
    }
    for (tag, min) in MANIFEST {
        let n = got.get(tag).copied().unwrap_or(0);
        assert!(
            n >= *min,
            "coverage tag {tag}: {n} rows, manifest minimum {min}"
        );
    }
    assert_eq!(
        got.len(),
        MANIFEST.len(),
        "unknown coverage tags present: {:?}",
        got.keys().collect::<Vec<_>>()
    );
    let slots: usize = got.values().sum();
    assert_eq!(slots, TAG_SLOTS, "tag-slot total");
    assert!(
        rs.len() >= ROW_FLOOR,
        "{} rows, floor {ROW_FLOOR}",
        rs.len()
    );

    // The 17 overlap slots, distributed EXACTLY as §7 states: 15 second-tags on
    // the §4.5 rows plus 2 third-tags on the named pair. Without this a dropped
    // required row can be counted around by retagging (PLAN-r1's I1).
    let second = rs.iter().filter(|r| covers(r).len() >= 2).count();
    let third = rs.iter().filter(|r| covers(r).len() == 3).count();
    assert_eq!(second, SECOND_TAGGED, "rows carrying a second tag");
    assert_eq!(third, THIRD_TAGGED, "rows carrying a third tag");
    assert!(
        rs.iter().filter(|r| covers(r).len() > 3).count() == 0,
        "no row may carry a fourth tag"
    );
    assert_eq!(
        slots - rs.len(),
        SECOND_TAGGED + THIRD_TAGGED,
        "overlap slots"
    );
    // Every multi-tagged row is a §4.5 promotion row, and both three-tag rows
    // are the pair §7 names.
    for r in rs.iter().filter(|r| covers(r).len() > 1) {
        assert!(
            covers(r).contains(&"promotion-near-miss") && covers(r).contains(&"gate"),
            "{}: only the fifteen §4.5 rows may carry a second tag",
            name(r)
        );
    }
    let pair: Vec<&str> = rs
        .iter()
        .filter(|r| covers(r).len() == 3)
        .map(|r| {
            let mut c = covers(r);
            c.retain(|t| *t != "promotion-near-miss" && *t != "gate");
            c[0]
        })
        .collect();
    assert!(
        pair.contains(&"whitespace") && pair.contains(&"formats-happy"),
        "the third tags are the named pair — the trailing-newline near-miss \
         (whitespace) and the bare-xpub happy path (formats-happy); got {pair:?}"
    );
}

/// §7 requirement 5. Without all three shapes the set goes vacuous: with no
/// yes/yes row a mutant that refuses everything passes, with no no/no row one
/// that admits everything passes, and with no device-only row the seam is
/// untested. `panic:parse` rows are skipped — their `device_admits` cannot be
/// evaluated, so they are not evidence either way.
#[test]
fn the_row_set_is_not_vacuous() {
    let d = doc();
    let (mut both, mut device_only, mut neither, mut host_wider, mut skipped) = (0, 0, 0, 0, 0);
    for r in rows(&d) {
        let host = r["host_admits"].as_bool().unwrap();
        let Some(device) = r.get("device_admits").and_then(|v| v.as_bool()) else {
            skipped += 1;
            continue;
        };
        match (host, device) {
            (true, true) => both += 1,
            (false, true) => device_only += 1,
            (false, false) => neither += 1,
            // NOT unreachable here, unlike the codex32 seam: §4.6's whitespace
            // rows are host-wider ON THE INPUT by design. They are safe only
            // because `canonical` is what gets packed, and the Go test asserts
            // the invariant over `canonical`, never over `input`.
            (true, false) => host_wider += 1,
        }
    }
    assert!(
        both > 0 && device_only > 0 && neither > 0,
        "{} rows: {both} both / {device_only} device-only / {neither} neither",
        rows(&d).len()
    );
    assert_eq!(skipped, POP.device_admits_absent, "panic:parse rows");
    // Every host-wider row is a whitespace row and carries a canonical.
    assert_eq!(host_wider, 3, "the §4.6 whitespace rows, and only those");
    for r in rows(&d) {
        if r["host_admits"].as_bool().unwrap()
            && r.get("device_admits").and_then(|v| v.as_bool()) == Some(false)
        {
            assert!(
                has(r, "canonical"),
                "{}: host wider than the device with no canonical",
                name(r)
            );
        }
    }
}

/// PLAN-r1's I7: pin the POPULATION of every column, so a field renamed by a
/// typo drops its count and reds the suite instead of silently disabling every
/// assertion that reads it. The fork's suite pins the same numbers for the
/// columns it owns, so the two harnesses cannot drift on what they cover.
#[test]
fn every_column_has_the_expected_population() {
    let d = doc();
    let rs = rows(&d);
    let count = |k: &str| rs.iter().filter(|r| has(r, k)).count();
    let truthy = |k: &str| {
        rs.iter()
            .filter(|r| r.get(k).and_then(|v| v.as_bool()) == Some(true))
            .count()
    };
    assert_eq!(rs.len(), POP.rows, "rows");
    assert_eq!(
        truthy("host_admits"),
        POP.host_admits_true,
        "host_admits=true"
    );
    assert_eq!(truthy("md1_admits"), POP.md1_admits_true, "md1_admits=true");
    assert_eq!(
        truthy("device_admits"),
        POP.device_admits_true,
        "device_admits=true"
    );
    assert_eq!(
        rs.iter()
            .filter(|r| r.get("device_admits").and_then(|v| v.as_bool()) == Some(false))
            .count(),
        POP.device_admits_false,
        "device_admits=false"
    );
    assert_eq!(
        rs.len() - count("device_admits"),
        POP.device_admits_absent,
        "device_admits absent"
    );
    assert_eq!(count("canonical"), POP.canonical, "canonical");
    assert_eq!(count("address_0"), POP.address_0, "address_0");
    assert_eq!(count("address_1"), POP.address_1, "address_1");
    assert_eq!(count("wallet_id"), POP.wallet_id, "wallet_id");
    assert_eq!(
        count("md_descriptor_contains"),
        POP.md_descriptor_contains,
        "md_descriptor_contains"
    );
    assert_eq!(count("sysw_class"), POP.sysw_class, "sysw_class");
    assert_eq!(count("device_probe"), POP.device_probe, "device_probe");
    assert_eq!(count("gate_open"), POP.gate_fields, "gate_open");
    assert_eq!(count("outcome"), POP.gate_fields, "outcome");
    assert_eq!(count("exit_code"), POP.gate_fields, "exit_code");
    assert_eq!(count("refusal_row"), POP.refusal_row, "refusal_row");
    assert_eq!(
        rs.iter()
            .filter(|r| r["md1_admits"].as_bool().unwrap()
                && r.get("device_admits").and_then(|v| v.as_bool()) == Some(true)
                && has(r, "address_0"))
            .count(),
        POP.both_routes_address_0,
        "rows where the device route and the md1 route are asserted to one address_0"
    );
    // Every carried address is on a row SOME route can derive, or the value is
    // unfalsifiable.
    for r in rs {
        if has(r, "address_0") || has(r, "address_1") {
            assert!(
                r["md1_admits"].as_bool().unwrap()
                    || r.get("device_admits").and_then(|v| v.as_bool()) == Some(true),
                "{}: carries an address no route derives",
                name(r)
            );
        }
        // wallet_id is scoped to MULTISIG rows at the device-default use-site —
        // the Go route's measured domain (md.EncodeMultisig hard-codes
        // <0;1>/* and has no single-sig arm). Both suites compute it
        // independently: the F-212 class as a standing gate.
        if has(r, "wallet_id") {
            assert!(
                r["md1_admits"].as_bool().unwrap()
                    && r.get("device_admits").and_then(|v| v.as_bool()) == Some(true),
                "{}: a wallet_id only one side can compute is not a cross-language gate",
                name(r)
            );
            assert_eq!(
                r["wallet_id"].as_str().unwrap().len(),
                32,
                "{}: WalletPolicyId is 16 bytes",
                name(r)
            );
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
// HOST COLUMNS. Every test below calls code S1/S3 has not written yet. Each
// ignore reason names the phase that removes it, and this block being empty of
// them is P2's gate (see the module doc for the exact command).
// ───────────────────────────────────────────────────────────────────────────

/// §5.2's classification predicate: `me` would pack this input as a
/// `Descriptor` record. THE SAFE DIRECTION is asserted by the fork's half over
/// `canonical`; this half asserts the column itself.
///
/// It asserts `format` in the same pass, and that is not padding: `format` is
/// the column F-1 could not settle from the spec alone, and asserting it is
/// what turns P0's "MATCHED = the branch that SUCCEEDED" reading from a comment
/// in a JSON file into a claim a suite can refute. The two readings disagree on
/// thirteen rows.
#[test]
fn the_host_column_matches_the_admission_predicate() {
    let d = doc();
    let (mut host_checked, mut format_checked) = (0usize, 0usize);
    for r in rows(&d) {
        let input = r["input"].as_str().unwrap();
        assert_eq!(
            mnemonic_engrave::descriptor::host_admits(input),
            r["host_admits"].as_bool().unwrap(),
            "{}: host_admits -- the host may be NARROWER than the device, never wider",
            name(r)
        );
        host_checked += 1;
        assert_eq!(
            mnemonic_engrave::descriptor::format_of(input),
            r["format"].as_str().unwrap(),
            "{}: format -- the branch of the cascade that SUCCEEDED (`none` where \
             `me` refuses AT the cascade), not the branch the input resembles",
            name(r)
        );
        format_checked += 1;
    }
    // PLAN-r1's I7 applied to this test's own work: a loop that silently
    // iterated zero rows would pass.
    assert_eq!(host_checked, POP.rows, "host_admits assertions run");
    assert_eq!(format_checked, POP.rows, "format assertions run");
}

// ── §5.2's predicate at the RECORD layer ───────────────────────────────────
// `host_admits` above IS the predicate; what follows asserts that
// `sysw::classify` answers WITH it, per row, in both directions. The fork's
// `TestDescriptorSeamSyswClass` asserts the same derived rule over the same
// file, so a Go/Rust divergence reds one of the two instead of hiding in a
// hand-stated column.

fn row<'a>(d: &'a serde_json::Value, want: &str) -> &'a serde_json::Value {
    rows(d)
        .iter()
        .find(|r| name(r) == want)
        .unwrap_or_else(|| panic!("{PATH}: no row named {want:?}"))
}

/// The canonical re-encode is the exact string §5.2's record carries, so this
/// is the arm on the input that matters most.
#[test]
fn a_canonical_descriptor_classifies_as_a_descriptor_record() {
    let d = doc();
    let r = row(&d, "formats-happy/bip380-sortedmulti-multipath");
    assert_eq!(
        mnemonic_engrave::sysw::classify(r["canonical"].as_str().unwrap()),
        Class::Descriptor
    );
}

/// **The arm is §5.2's predicate, not "the cascade parsed it".** This row's
/// `format` is `bip380` — the cascade parses it — while `host_admits` is
/// false, because conjunct 1's `multi` refusal is permanent. A classifier
/// keyed on the parse would place a record the device's own parser rejects.
#[test]
fn a_multi_policy_the_cascade_parses_is_not_a_descriptor_record() {
    let d = doc();
    let r = row(&d, "neither/wsh-multi");
    assert_eq!(
        r["format"].as_str().unwrap(),
        "bip380",
        "the cascade parses it"
    );
    assert!(!r["host_admits"].as_bool().unwrap(), "§4.7 refuses it");
    assert_eq!(
        mnemonic_engrave::sysw::classify(r["input"].as_str().unwrap()),
        Class::Unknown
    );
}

/// The `refusal_row` vocabulary is one set, held in two places, and this is
/// what stops them drifting: the file's `refusal_rows` map and the library's
/// `Row` enum must name exactly the same 36 slugs (PLAN-r4's NEW-M6). P2.4's
/// per-row text tests key to these.
#[test]
fn the_refusal_row_vocabulary_is_the_same_set_on_both_sides() {
    let d = doc();
    let on_disk: std::collections::BTreeSet<&str> = d["refusal_rows"]
        .as_object()
        .unwrap()
        .keys()
        .map(|s| s.as_str())
        .collect();
    let in_code: std::collections::BTreeSet<&str> = mnemonic_engrave::descriptor::Row::ALL
        .iter()
        .map(|r| r.slug())
        .collect();
    assert_eq!(in_code.len(), 36, "the §6 vocabulary is 36 rows");
    assert_eq!(
        on_disk, in_code,
        "the file's refusal_rows and `descriptor::Row` name different sets"
    );
}

/// The four gate fields, asserted against the REAL `--as`-omitted invocation:
/// gate verdict, outcome class, §6 row and exit code, for each of the 37 rows.
/// §5.1's gate is NORMATIVE through these rows — where any reading of the
/// prose disagrees with a row, the row is the answer.
///
/// **All four fields come from the real run**, and the link is the refusal TEXT
/// rather than a marker: the row's expected `refusal_row` selects a `Refusal`
/// from the library, and the binary's stderr must CONTAIN that refusal's text.
/// A binary that took a different branch prints a different text and reds here,
/// so this cannot degenerate into asserting the library against itself. The
/// exit code and the outcome class are read straight off the process.
///
/// Every input is delivered with `--in` (r20's M2): on argv the shipped
/// bearer-transaction guard preempts `tx: zz` at exit 3, before any of this
/// runs.
#[test]
fn the_gate_rows_pin_the_real_invocation() {
    let d = doc();
    let dir = tempfile::tempdir().unwrap();
    let mut checked = 0usize;
    for (n, r) in rows(&d).iter().enumerate() {
        if !covers(r).contains(&"gate") {
            continue;
        }
        let input = r["input"].as_str().unwrap();
        let path = dir.path().join(format!("gate{n}.txt"));
        std::fs::write(&path, input).unwrap();

        let out = assert_cmd::Command::cargo_bin("me")
            .unwrap()
            .args(["sysw", "pack", "--no-passphrase", "--in"])
            .arg(&path)
            .output()
            .unwrap();
        let code = out.status.code().unwrap();
        let err = String::from_utf8_lossy(&out.stderr).to_string();
        let name = name(r);

        // (1) exit_code, straight off the process.
        assert_eq!(
            code,
            r["exit_code"].as_u64().unwrap() as i32,
            "{name}: exit code. stderr:\n{err}"
        );
        // (2) outcome, classified from what the run actually printed.
        let outcome = classify_run(code, &err);
        assert_eq!(
            outcome,
            r["outcome"].as_str().unwrap(),
            "{name}: outcome. stderr:\n{err}"
        );
        // (3) gate_open. A CLOSED gate means the shipped record-classification
        // refusal, unchanged and in record vocabulary — invariant 1. An OPEN
        // one means it never fires.
        let record_refusal = is_record_refusal(&err);
        assert_eq!(
            !record_refusal,
            r["gate_open"].as_bool().unwrap(),
            "{name}: gate_open. stderr:\n{err}"
        );
        // (4) refusal_row, tied to the run by the text the row prints.
        match r.get("refusal_row").and_then(|v| v.as_str()) {
            Some(slug) => {
                let lib = mnemonic_engrave::descriptor::consult(input, &records_of(input));
                assert_eq!(lib.refusal_row(), Some(slug), "{name}: refusal_row");
                let text = &lib.refusal().unwrap().text;
                assert!(
                    err.contains(text.as_str()),
                    "{name}: stderr does not carry the {slug} text.\n\
                     WANT: {text}\nGOT: {err}"
                );
            }
            None => assert!(
                !err.contains("wallet descriptor in any of the four forms"),
                "{name}: a row with no refusal_row printed a §6 refusal:\n{err}"
            ),
        }
        checked += 1;
    }
    assert_eq!(checked, POP.gate_fields, "gate rows exercised");
}

/// **The shipped record-classification refusal, recognised by its own
/// vocabulary.** Every arm of `sysw_error`'s unclassifiable case — the reserved
/// prefixes, the unparseable transaction, the unsigned inputs, the BIP-93
/// profile miss, the catch-all — opens `record N (records count from 0)`, and
/// nothing else `me` prints does. Matching one arm's tail instead was the first
/// version of this helper and it reported `unclassified` for five of the six
/// hostile-payload rows: invariant 1 is about the SURFACE, not one message.
fn is_record_refusal(err: &str) -> bool {
    err.contains("(records count from 0)")
}

/// The outcome CLASS, read off the real run rather than asked of the library.
fn classify_run(code: i32, err: &str) -> &'static str {
    if is_record_refusal(err) {
        return "record-refusal";
    }
    if code == 2 && err.contains("`--as` decides how it is packed") {
        return "as-decides";
    }
    if code == 4 && err.contains("is a wallet descriptor. A descriptor is packed ALONE") {
        return "multi-record";
    }
    if code == 3 {
        return "descriptor-refusal";
    }
    "unclassified"
}

/// The record stream the shipped `--in` contract makes of an input: non-blank
/// lines, which is what §6's multi-record row indexes into.
fn records_of(input: &str) -> Vec<String> {
    input
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

/// `md1_admits` is an INDEPENDENT axis, not a qualifier on `host_admits`. Where
/// it is false on a row whose input is otherwise ADMITTED, the refusal
/// assertion must cite §5.3(a)/(a″) — a refusal for an unrelated cause must not
/// satisfy it, which is what turns §5.3 from prose into a gate.
#[test]
fn the_md1_column_matches_the_representability_rules() {
    let d = doc();
    let (mut checked, mut cited) = (0usize, 0usize);
    for r in rows(&d) {
        let input = r["input"].as_str().unwrap();
        let want = r["md1_admits"].as_bool().unwrap();
        let parsed = mnemonic_engrave::descriptor::cascade::cascade(
            &mnemonic_engrave::descriptor::cascade::normalise(input),
        );
        let got = match &parsed {
            Ok(p) => {
                mnemonic_engrave::descriptor::admit::admit(
                    p,
                    mnemonic_engrave::descriptor::Path::Md1,
                )
                .is_ok()
                    && mnemonic_engrave::descriptor::admit::md1_offenders(p).is_empty()
            }
            Err(_) => false,
        };
        assert_eq!(got, want, "{}: md1_admits", name(r));
        checked += 1;

        // Where the CASCADE succeeded and md1 still refuses, the refusal has to
        // be §5.3's own — a refusal for an unrelated cause must not satisfy the
        // column. §5.3(a) is `md1-fixed-index`, §5.3(a″) is `md1-no-wildcard`.
        if let (Ok(p), false) = (&parsed, want) {
            if mnemonic_engrave::descriptor::admit::admit(
                p,
                mnemonic_engrave::descriptor::Path::Md1,
            )
            .is_ok()
            {
                let rs = mnemonic_engrave::descriptor::admit::md1_refusals(p, "", "");
                let slugs: Vec<&str> = rs.iter().map(|x| x.row.slug()).collect();
                assert!(
                    !slugs.is_empty()
                        && slugs
                            .iter()
                            .all(|s| *s == "md1-fixed-index" || *s == "md1-no-wildcard"),
                    "{}: md1 refuses an ADMITTED descriptor for a non-§5.3 reason: {slugs:?}",
                    name(r)
                );
                cited += 1;
            }
        }
    }
    assert_eq!(checked, POP.rows, "md1_admits assertions run");
    // The four `md1-split` rows §7 carries for exactly this purpose, plus the
    // shipped JSON fixture, whose `/0/*` is what makes §11 item 2 demand a
    // non-`/0/*` JSON exemplar.
    assert_eq!(
        cited, 5,
        "rows where §5.3 (not another conjunct) is the refusal"
    );
}

/// Every carried `address_N`, derived through the md1 round trip wherever
/// `md1_admits` — including `host_admits=false` rows like `multi`, whose
/// address assertions run only through the md1 route. On a row the device also
/// derives, the two routes must agree: that equality IS §5.3(a′)'s
/// materialisation claim, at the layer where a string comparison cannot reach.
#[test]
fn the_md1_route_derives_every_carried_address() {
    let d = doc();
    let (mut a0, mut a1, mut both) = (0usize, 0usize, 0usize);
    for r in rows(&d) {
        if !r["md1_admits"].as_bool().unwrap() {
            continue;
        }
        let built = build_md1(r);
        let net = mnemonic_engrave::descriptor::md1::network(&parse(r));
        if let Some(want) = r.get("address_0").and_then(|v| v.as_str()) {
            let got = mnemonic_engrave::descriptor::md1::address(&built, 0, 0, net).unwrap();
            assert_eq!(got, want, "{}: address_0 through the md1 route", name(r));
            a0 += 1;
            if r.get("device_admits").and_then(|v| v.as_bool()) == Some(true) {
                both += 1;
            }
        }
        // `address_N` is the RECEIVE address at INDEX N — the fork derives it
        // with `address.Receive(d, N)` (`descriptor_seam_test.go:246`), so the
        // chain stays 0 and the wildcard index moves. Reading it as the CHANGE
        // address instead derives a real, wrong address that looks equally
        // plausible: `bc1qs69gskx…` against the file's `bc1qnww8rje…`, measured.
        if let Some(want) = r.get("address_1").and_then(|v| v.as_str()) {
            let got = mnemonic_engrave::descriptor::md1::address(&built, 0, 1, net).unwrap();
            assert_eq!(got, want, "{}: address_1 through the md1 route", name(r));
            a1 += 1;
        }
    }
    // The two carried-address columns minus the rows md1 does not carry: the
    // JSON fixture and the four `md1-split` refusals hold addresses only the
    // DEVICE route derives, and the fork's suite owns those.
    assert_eq!(a0, 15, "address_0 assertions run through the md1 route");
    assert_eq!(a1, 4, "address_1 assertions run through the md1 route");
    assert_eq!(
        both, POP.both_routes_address_0,
        "rows where the device route and the md1 route are asserted to ONE address_0"
    );
}

/// `wallet_id`, computed from `me`'s own implementation. The fork computes the
/// same value from its own. A divergence is the F-212 class — an identity
/// mismatch no per-repo test can see.
#[test]
fn the_md1_route_computes_every_carried_wallet_id() {
    let d = doc();
    let mut checked = 0usize;
    for r in rows(&d) {
        let Some(want) = r.get("wallet_id").and_then(|v| v.as_str()) else {
            continue;
        };
        let built = build_md1(r);
        let got = mnemonic_engrave::descriptor::md1::wallet_id(&built).unwrap();
        assert_eq!(
            got,
            want,
            "{}: WalletPolicyId — the two languages disagree on wallet IDENTITY,              the F-212 class",
            name(r)
        );
        checked += 1;
    }
    assert_eq!(checked, POP.wallet_id, "wallet_id assertions run");
}

/// `md_descriptor_contains`, asserted against the round trip's read-back. The
/// pin on the `multi` row is `wsh(multi(` and NOT `multi(`, because
/// `sortedmulti(` CONTAINS `multi(` and the shorter pin passes on the
/// `multi` → `sortedmulti` mutant's own read-back.
#[test]
fn the_md1_route_read_back_contains_every_pin() {
    let d = doc();
    let mut checked = 0usize;
    for r in rows(&d) {
        let Some(want) = r.get("md_descriptor_contains").and_then(|v| v.as_str()) else {
            continue;
        };
        // The READ-BACK, not the built descriptor: encode to md1 strings, decode
        // them again, and render THAT. A builder that agreed with itself while
        // the wire round trip lost the tag would pass a template comparison.
        let built = build_md1(r);
        let strings = mnemonic_engrave::descriptor::md1::strings(&built).unwrap();
        let refs: Vec<&str> = strings.iter().map(String::as_str).collect();
        let back = md_codec::reassemble(&refs).unwrap();
        let template = md_codec::descriptor_to_template(&back).unwrap();
        assert!(
            template.contains(want),
            "{}: the md1 read-back is {template:?}, which does not contain {want:?}",
            name(r)
        );
        checked += 1;
    }
    assert_eq!(
        checked, POP.md_descriptor_contains,
        "md_descriptor_contains assertions run"
    );
}

fn parse(r: &serde_json::Value) -> mnemonic_engrave::descriptor::cascade::Parsed {
    let input = r["input"].as_str().unwrap();
    mnemonic_engrave::descriptor::cascade::cascade(
        &mnemonic_engrave::descriptor::cascade::normalise(input),
    )
    .unwrap_or_else(|e| {
        panic!(
            "{}: the cascade refused an md1-admitted row: {e:?}",
            name(r)
        )
    })
}

fn build_md1(r: &serde_json::Value) -> mnemonic_engrave::descriptor::md1::Built {
    mnemonic_engrave::descriptor::md1::build(&parse(r))
        .unwrap_or_else(|e| panic!("{}: the md1 build refused an admitted row: {e}", name(r)))
}
/// **The `canonical` column, on the side that PRODUCES it.**
///
/// §7 requirement 4 states the invariant over `canonical` and gives its
/// assertion to the fork: the Go test parses each `canonical` and requires the
/// re-encoding to equal it, a fixed point. What no test asserted is that `me`'s
/// own encoder produces those 19 strings in the first place — and `me` is what
/// packs them (§5.2), so a host encoder that drifted from the file would pack a
/// record the Go half is still happily calling a fixed point.
///
/// Every value in the column was measured by P0 through the DEVICE route. This
/// asserts the Rust route lands on the same 19 strings, checksum included.
#[test]
fn the_encoder_produces_every_canonical_the_file_carries() {
    let d = doc();
    let mut checked = 0;
    for r in rows(&d) {
        let Some(want) = r.get("canonical").and_then(|v| v.as_str()) else {
            continue;
        };
        let input = r["input"].as_str().unwrap();
        let parsed = mnemonic_engrave::descriptor::cascade::cascade(
            &mnemonic_engrave::descriptor::cascade::normalise(input),
        )
        .unwrap_or_else(|e| {
            panic!(
                "{}: the cascade refused a host-admitted row: {e:?}",
                name(r)
            )
        });
        assert_eq!(parsed.encode(), want, "{}: canonical", name(r));
        checked += 1;
    }
    assert_eq!(checked, POP.canonical, "canonical assertions run");
}

/// **The two derivations agree wherever both can derive.**
///
/// Two implementations of one answer is the F-212 divergence class, and §5.4's
/// `address 0:` line now has two: `md_codec`'s whole-descriptor
/// `derive_address` (the twin) and `descriptor::derive`'s per-key walk. The
/// second exists only because the first cannot express a wallet whose keys want
/// different receive indices (IMPL-S1S3-adversarial-review I1).
///
/// This is what stops them drifting: over every row the cascade parses, where
/// BOTH can answer, the answers must be equal. Twenty of these rows carry a
/// device-measured `address_0`, so agreement here is agreement with the device.
#[test]
fn the_two_derivations_agree_wherever_both_can_derive() {
    let d = doc();
    let (mut agreed, mut against_file) = (0usize, 0usize);
    for r in rows(&d) {
        let input = r["input"].as_str().unwrap();
        let Ok(parsed) = mnemonic_engrave::descriptor::cascade::cascade(
            &mnemonic_engrave::descriptor::cascade::normalise(input),
        ) else {
            continue;
        };
        let per_key = mnemonic_engrave::descriptor::derive::address_0(&parsed);
        let twin = mnemonic_engrave::descriptor::md1::derivation_twin(&parsed)
            .ok()
            .and_then(|(b, i)| {
                let net = mnemonic_engrave::descriptor::md1::network(&parsed);
                mnemonic_engrave::descriptor::md1::address(&b, 0, i?, net).ok()
            });
        if let (Some(a), Some(b)) = (&per_key, &twin) {
            assert_eq!(a, b, "{}: the two derivations disagree", name(r));
            agreed += 1;
        }
        // And where the FILE carries a device-measured address, the per-key
        // walk must land on it — the assertion that makes agreement mean
        // "agrees with the device" rather than "agrees with itself".
        if let (Some(a), Some(want)) = (&per_key, r.get("address_0").and_then(|v| v.as_str())) {
            assert_eq!(a, want, "{}: per-key address_0 vs the device", name(r));
            against_file += 1;
        }
    }
    assert!(
        agreed >= 25,
        "only {agreed} rows exercised the differential — the loop has gone vacuous"
    );
    assert_eq!(
        against_file, POP.address_0,
        "every device-measured address_0 is reached by the per-key walk"
    );
}
