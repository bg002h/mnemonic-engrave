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

use sha2::Digest as _;

/// The sha256 of `testdata/descriptor_seam_vectors.json`, pinned IDENTICALLY
/// in the fork's `nonstandard/descriptor_seam_test.go`. Changing a row means
/// changing this in both repos, which is the point — see the file's own
/// header, and `scripts/descriptor-seam-vectors/README.md` for the regenerate
/// + re-pin recipe.
const SEAM_VECTORS_SHA256: &str =
    "0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584";

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
#[test]
fn the_host_column_matches_the_admission_predicate() {
    unimplemented!("P1.1: mnemonic_engrave::descriptor::admit over every row");
}

/// The four gate fields, asserted against the REAL `--as`-omitted invocation:
/// gate verdict, outcome class, §6 row and exit code, for each of the 37 rows.
/// §5.1's gate is NORMATIVE through these rows — where any reading of the
/// prose disagrees with a row, the row is the answer.
#[test]
fn the_gate_rows_pin_the_real_invocation() {
    unimplemented!("P1.2: run `me sysw pack` per gate row and assert all four fields");
}

/// `md1_admits` is an INDEPENDENT axis, not a qualifier on `host_admits`. Where
/// it is false on a row whose input is otherwise ADMITTED, the refusal
/// assertion must cite §5.3(a)/(a″) — a refusal for an unrelated cause must not
/// satisfy it, which is what turns §5.3 from prose into a gate.
#[test]
#[ignore = "P2: `--as md1` is not built"]
fn the_md1_column_matches_the_representability_rules() {
    unimplemented!("P2.2/P2.4: --as md1 per row, citing §5.3(a)/(a″) where it refuses");
}

/// Every carried `address_N`, derived through the md1 round trip wherever
/// `md1_admits` — including `host_admits=false` rows like `multi`, whose
/// address assertions run only through the md1 route. On a row the device also
/// derives, the two routes must agree: that equality IS §5.3(a′)'s
/// materialisation claim, at the layer where a string comparison cannot reach.
#[test]
#[ignore = "P2: `--as md1` is not built"]
fn the_md1_route_derives_every_carried_address() {
    unimplemented!("P2.2: encode, read back, derive address_0/address_1");
}

/// `wallet_id`, computed from `me`'s own implementation. The fork computes the
/// same value from its own. A divergence is the F-212 class — an identity
/// mismatch no per-repo test can see.
#[test]
#[ignore = "P2: the in-process md_codec build is not written"]
fn the_md1_route_computes_every_carried_wallet_id() {
    unimplemented!("P2.2/P2.3: compute_wallet_policy_id over the (a′)-materialised policy");
}

/// `md_descriptor_contains`, asserted against the round trip's read-back. The
/// pin on the `multi` row is `wsh(multi(` and NOT `multi(`, because
/// `sortedmulti(` CONTAINS `multi(` and the shorter pin passes on the
/// `multi` → `sortedmulti` mutant's own read-back.
#[test]
#[ignore = "P2: `--as md1` is not built"]
fn the_md1_route_read_back_contains_every_pin() {
    unimplemented!("P2.2: `md descriptor`-equivalent read-back of the encoded set");
}
