//! **§6, row by row — the TEXT, not the exit code.**
//!
//! `SPEC_descriptor_input.md` §11 item 4: *"Every refusal in §6 has a test that
//! reaches it and asserts the text, not just the exit code."* All 35 rows, and
//! the S2-parked set is EMPTY — every §6 trigger is reachable in this build.
//!
//! **S2 SUBTRACTED a row.** `window-not-in-build` fired when `--as descriptor`
//! had no path; the path shipped, so no build state reaches it and the row
//! retired with §5.1's window refusal itself.
//!
//! # What "verbatim" means here
//!
//! **What THIS BUILD PRINTS**, which is §6's text with its substitutions
//! applied. Three classes of row substitute, and each says so at its test:
//!
//! * the two **window-substituted** rows (§5.3's `/i/*` and `<i;i+1>` rows) —
//!   the substitution is conditional on the build, and since S2 the descriptor
//!   path SHIPS, so what these two rows print is §6's own remedy naming the
//!   flag. For a `multi`-form input the substitution is exempt in every build
//!   and the neither-path replacement fires instead;
//! * the **enumeration-substituted** row (`bluewallet-no-name`) — §6 spells the
//!   enumeration as *"it has `Policy`, `Derivation` and `Format` headers and
//!   `N` cosigner lines"*, and the input that reaches it is a ONE-LINE file
//!   with none of those headers, so a fixed enumeration would be false about
//!   the operator's own file;
//! * the **`multi`-class device clause** (spec amendment 2026-08-29) — the
//!   device refuses every `multi` form at PARSE, so no device-behaviour claim
//!   transposes to one.
//!
//! # The row-count gate
//!
//! This file counts its OWN row tests, by reading its own source, and asserts
//! the set equals `descriptor::Row::ALL`'s 36 slugs. A dropped or renamed row
//! test reds the suite rather than shrinking the coverage silently. The anchor
//! is `fn row_` at COLUMN 0 — doc comments start with `///` and never sit at
//! column 0, so the gate cannot match its own documentation.

use std::process::Output;

/// This file, read by the row-count gate below.
const SELF: &str = include_str!("descriptor_refusals.rs");

const VECTORS: &str = "testdata/descriptor_seam_vectors.json";

fn vector_input(name: &str) -> String {
    let raw = std::fs::read(VECTORS).unwrap_or_else(|e| panic!("{VECTORS}: {e}"));
    let doc: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    for r in doc["vectors"].as_array().unwrap() {
        if r["name"].as_str().unwrap() == name {
            return r["input"].as_str().unwrap().to_string();
        }
    }
    panic!("{VECTORS}: no row named {name:?}");
}

/// Run `me sysw pack` over `document`, delivered whole through `--in`.
fn run(document: &str, extra: &[&str]) -> Output {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("document.txt");
    std::fs::write(&path, document).unwrap();
    assert_cmd::Command::cargo_bin("me")
        .unwrap()
        .args(["sysw", "pack", "--no-passphrase"])
        .args(extra)
        .arg("--in")
        .arg(&path)
        .output()
        .unwrap()
}

/// The workhorse: run the input, assert the exit code, assert stderr carries
/// `want` VERBATIM, and assert `me` selected the §6 row it was supposed to.
fn assert_row(slug: &str, document: &str, extra: &[&str], exit: i32, want: &str) {
    let out = run(document, extra);
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert_eq!(
        out.status.code().unwrap(),
        exit,
        "{slug}: exit code. stderr:\n{err}"
    );
    assert!(
        err.contains(want),
        "{slug}: stderr does not carry §6's text.\nWANT: {want}\nGOT:  {err}"
    );
    // A refusal writes 0 bytes to stdout (`SPEC_constellation_cli_uniformity`
    // §2). Checked on every row rather than once, because the rows reach the
    // exit through four different code paths.
    if exit != 0 {
        assert!(
            out.stdout.is_empty(),
            "{slug}: a refusal wrote {} bytes to stdout",
            out.stdout.len()
        );
    }
}

// ── fixtures the vector file deliberately does not carry ───────────────────

const XPUB: &str = "xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan";
const XPUB2: &str = "xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge";

/// A BlueWallet file whose `Policy:` header declares a different cosigner count
/// than the file carries. §7 has no row for it — the gate rows cover the shape
/// gate, and this is a §4.2 rule the cascade enforces.
fn bluewallet_policy_mismatch() -> String {
    format!(
        "Name: x\nPolicy: 2 of 3\nDerivation: m/48'/0'/0'/2'\nFormat: P2WSH\n\
         DC567276: {XPUB}\nF245AE38: {XPUB2}\n"
    )
}

// ───────────────────────────────────────────────────────────────────────────
// THE ROW-COUNT GATE
// ───────────────────────────────────────────────────────────────────────────

/// Every `fn row_*` in this file, as the §6 slug it names.
///
/// The anchor is a `fn row_` at COLUMN 0 — every helper in this file is named
/// so it cannot match (`vector_input`, `named_row_tests`, `assert_row`), and doc
/// comments start with `///` and never sit at column 0, so the gate can match
/// neither its own documentation nor its own scaffolding.
fn named_row_tests() -> std::collections::BTreeSet<String> {
    SELF.lines()
        .filter_map(|l| l.strip_prefix("fn row_"))
        .filter_map(|l| l.split('(').next())
        .map(|n| n.replace('_', "-"))
        .collect()
}

/// **§11 item 4's own gate.** The plan's clause: *"The test file asserts its own
/// row-test count"* — 35 since S2 retired `window-not-in-build` — and the
/// S2-parked set is EMPTY.
#[test]
fn the_file_carries_one_named_test_per_section_6_row() {
    let mine = named_row_tests();
    assert_eq!(
        mine.len(),
        35,
        "§6 is 35 data rows; this file names {mine:?}"
    );
    let vocabulary: std::collections::BTreeSet<String> = mnemonic_engrave::descriptor::Row::ALL
        .iter()
        .map(|r| r.slug().to_string())
        .collect();
    assert_eq!(
        mine, vocabulary,
        "the row tests and `descriptor::Row` name different sets"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// §4.1/§4.4 — the cascade rows
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn row_unparseable() {
    assert_row(
        "unparseable",
        "this file is not any of the four\n",
        &["--as", "md1"],
        3,
        "this is not a wallet descriptor in any of the four forms `me` reads: a BlueWallet \
         `Key: value` setup file, a plain BIP-380 descriptor, a `{\"label\":…,\"descriptor\":…}` \
         JSON export, or a single extended key.",
    );
}

/// The SHIPPED `me 0.7.0` refusal, kept verbatim per §6 (R0's I7), at
/// **`EXIT_USAGE` (2)** — this spec records the existing behaviour rather than
/// silently regressing a tested surface.
#[test]
fn row_empty_file() {
    assert_row("empty-file", "", &["--as", "md1"], 2, "no records in ");
}

/// Blank records are skipped, so a whitespace-only file reaches the same
/// shipped path — same message, same exit code.
#[test]
fn row_whitespace_only() {
    assert_row(
        "whitespace-only",
        "   \n\n \t \n",
        &["--as", "md1"],
        2,
        "no records in ",
    );
}

/// §5.1's choice block. **`EXIT_USAGE` (2)**, not 3 — nothing was refused, a
/// choice was not made.
#[test]
fn row_as_omitted() {
    assert_row(
        "as-omitted",
        &vector_input("formats-happy/bip380-sortedmulti-multipath"),
        &[],
        2,
        "this input is a wallet descriptor, and `--as` decides how it is packed.",
    );
}

/// **§5.1's NORMATIVE block, VERBATIM — the whole thing, not its first line.**
///
/// `row_as_omitted` above pins the verdict sentence, which is what §6 requires
/// of it; nothing pinned the sixteen lines under it, and that is how the head
/// spent S1 and S3 rendered on its own line while §5.1's block puts the
/// description INLINE on a padded head (IMPL-P1 review's M2, fixed in S2). A
/// `contains` on one sentence cannot see a column, so this asserts the block
/// as a whole and by equality.
///
/// The `me: ` prefix is the caller's, so the block itself starts at the verdict
/// and every continuation line is indented to the description column.
#[test]
fn the_choice_block_is_section_5_1s_normative_text_verbatim() {
    const BLOCK: &str = "\
this input is a wallet descriptor, and `--as` decides how it is packed.
      --as descriptor   the SCANNABLE plate. The device engraves the wallet
                        as a QR that any phone or wallet app can read -- no
                        special tooling to restore, ever. Packed in CANONICAL
                        form (SLIP-132 versions become xpub, ' becomes h,
                        checksum recomputed). The engraver itself cannot
                        read the QR back (it has no camera).
      --as md1          the HAND-COPYABLE plate. me converts the descriptor
                        and packs error-corrected md1 text cards in ONE step
                        (no md invocation needed). Restored by transcription;
                        each string survives up to 4 MIS-STRUCK characters
                        (substitutions -- a missing or extra strike is not
                        correctable), so it can even be hand-stamped. Carries
                        policies --as descriptor cannot. Restoring needs an
                        md1 decoder (an open spec; the tooling today is this
                        project's).
    They are not interchangeable -- `me sysw pack --help` has the comparison.";

    // The library's own rendering, and the one an operator actually sees.
    assert_eq!(
        mnemonic_engrave::descriptor::choice_block(),
        BLOCK,
        "§5.1's block changed"
    );
    let out = run(
        &vector_input("formats-happy/bip380-sortedmulti-multipath"),
        &[],
    );
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert_eq!(out.status.code().unwrap(), 2, "stderr:\n{err}");
    assert!(
        err.contains(&format!("me: {BLOCK}\n")),
        "the binary does not print §5.1's block verbatim:\n{err}"
    );

    // Both descriptions start in ONE column, and it is the column §5.1 draws.
    // Stated as an independent property rather than trusted to the literal
    // above: a transcription error in the literal would otherwise be pinned as
    // if it were the spec.
    let mut heads = 0;
    for l in BLOCK.lines().skip(1) {
        if l.starts_with("    They are not") {
            continue;
        }
        match l.strip_prefix("      --as ") {
            // An entry head: `--as <value>` padded so the description starts in
            // the description column, and unmarked, because this build carries
            // both values.
            Some(rest) => {
                let value = rest.split_whitespace().next().unwrap();
                assert_eq!(
                    l.len() - l.trim_start().len(),
                    6,
                    "an entry head is indented six: {l:?}"
                );
                // Where the DESCRIPTION starts: the head's own length, which
                // is the padding's whole job.
                let after = &rest[value.len()..];
                assert_eq!(
                    l.len() - after.trim_start().len(),
                    24,
                    "`--as {value}`'s description does not start in column 24: {l:?}"
                );
                assert!(
                    !l.contains("not available in this build"),
                    "a value this build carries is build-marked: {l:?}"
                );
                heads += 1;
            }
            // A continuation line, in the same column.
            None => assert_eq!(
                l.len() - l.trim_start().len(),
                24,
                "a continuation line is out of column: {l:?}"
            ),
        }
    }
    assert_eq!(heads, 2, "§5.1's block offers exactly two values");
}

#[test]
fn row_json_inner_malformed() {
    assert_row(
        "json-inner-malformed",
        "{\"label\":\"my wallet\",\"descriptor\":\"wsh(sortedmulti(2,\"}",
        &["--as", "md1"],
        3,
        "the `{label, descriptor}` JSON parsed, and its `descriptor` field did not: \
         script: missing `)`. The label was \"my wallet\". The problem is in the \
         descriptor string, not the JSON.",
    );
}

/// §6 row 5's ABSENT-field half (F-438, measured in the F-429 journey walk):
/// before the split, `{"label":"my wallet"}` coerced the missing key to `""`
/// and answered with the sibling's sentence above — "the problem is in the
/// descriptor string, not the JSON" — which for this input is false twice
/// over: there is no descriptor string, and the missing key IS a JSON
/// problem. Same row, split cause; the negation is asserted so the lie
/// cannot regress silently.
#[test]
fn json_missing_descriptor_field_gets_its_own_sentence() {
    assert_row(
        "json-inner-malformed",
        "{\"label\":\"my wallet\"}",
        &["--as", "descriptor"],
        3,
        "the `{label, descriptor}` JSON parsed and carries no `descriptor` field -- \
         its keys are: label. The label was \"my wallet\". The problem is the \
         missing field, not a descriptor string: re-export from your wallet \
         software with the descriptor included.",
    );
    let out = run("{\"label\":\"my wallet\"}", &["--as", "descriptor"]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.contains("not the JSON"),
        "the sibling's false-for-this-input sentence is back: {err}"
    );
    // T4 is NOT widened: without --as the same input still gets the record
    // refusal at exit 4, never descriptor vocabulary (F-429's invariant).
    let out = run("{\"label\":\"my wallet\"}", &[]);
    assert_eq!(out.status.code().unwrap(), 4);
}

// ───────────────────────────────────────────────────────────────────────────
// §4.2 — the BlueWallet rows
// ───────────────────────────────────────────────────────────────────────────

/// **ENUMERATION-SUBSTITUTED, and IMPL-P1's F-2.** §6 spells the enumeration as
/// *"it has `Policy`, `Derivation` and `Format` headers and `N` cosigner
/// lines"*; the gate row that reaches this is a ONE-LINE file with none of those
/// headers, so `me` substitutes what the file actually contains. A fixed
/// enumeration would be false about the operator's own file, which is the defect
/// §6 exists to remove.
#[test]
fn row_bluewallet_no_name() {
    assert_row(
        "bluewallet-no-name",
        &vector_input("gate/deadbeef-fronts-an-xpub"),
        &[],
        3,
        "this is a BlueWallet setup file -- it has 1 cosigner line -- but no `Name:` \
         header, and the device requires one. Add a line `Name: <anything>`.",
    );
}

#[test]
fn row_bluewallet_no_format() {
    assert_row(
        "bluewallet-no-format",
        &vector_input("bluewallet/no-format-header"),
        &["--as", "md1"],
        3,
        "this BlueWallet setup file has no `Format:` header, so the script type is \
         undefined. Add `Format: P2WSH` (or `P2SH`, or `P2WSH-P2SH`).",
    );
}

#[test]
fn row_bluewallet_zero_cosigners() {
    assert_row(
        "bluewallet-zero-cosigners",
        &vector_input("bluewallet/zero-cosigner-lines"),
        &["--as", "md1"],
        3,
        "this BlueWallet file has headers but no cosigner lines \
         (`<8-hex-fingerprint>: <xpub>`). There is no wallet here to pack -- was the \
         export truncated? Re-export from the coordinator.",
    );
}

#[test]
fn row_bluewallet_policy_count() {
    assert_row(
        "bluewallet-policy-count",
        &bluewallet_policy_mismatch(),
        &["--as", "md1"],
        3,
        "`Policy: 2 of 3` declares 3 cosigners; the file has 2. Cosigner lines are \
         `<8-hex-fingerprint>: <xpub>`.",
    );
}

#[test]
fn row_bluewallet_no_origin() {
    assert_row(
        "bluewallet-no-origin",
        &vector_input("bluewallet/derivation-after-keys"),
        &["--as", "md1"],
        3,
        "cosigner `dc567276` has no derivation path -- the `Derivation:` header appears \
         after the cosigner lines. The descriptor this file produces cannot be re-read \
         by the device. Put `Derivation: <path>` above the first cosigner line.",
    );
}

/// §4.2 defect 4: the device PANICS on fewer than 4 bytes, so this file must
/// never reach it.
#[test]
fn row_bluewallet_bad_fingerprint() {
    assert_row(
        "bluewallet-bad-fingerprint",
        &vector_input("bluewallet/short-fingerprint"),
        &["--as", "md1"],
        3,
        "-- a master fingerprint is exactly 8 hex characters (4 bytes).",
    );
}

// ───────────────────────────────────────────────────────────────────────────
// §5.1 — the choice block's siblings
//
// `row_window_not_in_build` lived here. §5.1's window refusal fired only "in a
// build where the `--as descriptor` path has not shipped"; S2 shipped it, so
// the row has no trigger and retired rather than being kept green by a weaker
// assertion. `item_5_the_five_case_matrix` below carries what replaced it.
// ───────────────────────────────────────────────────────────────────────────

/// §6's multi-record row, reached with the MNEMONIC-FIRST ordering (§11 item
/// 4's clause): a descriptor-FIRST input passes with or without the per-line
/// gate scope and is not a witness for the repair that rule exists to pin.
/// **`EXIT_INVALID` (4)**, as today.
#[test]
fn row_multi_record_descriptor() {
    assert_row(
        "multi-record-descriptor",
        &vector_input("gate/multi-record-mnemonic-first"),
        &[],
        4,
        " is a wallet descriptor. A descriptor is packed ALONE: run \
         `me sysw pack --as <descriptor|md1>` with just the descriptor -- one container \
         cannot yet carry a descriptor plus other records. The other records pack \
         without `--as`, as usual.",
    );
}

// ───────────────────────────────────────────────────────────────────────────
// §4.7 — the admission rows
// ───────────────────────────────────────────────────────────────────────────

/// Conjunct 1's PERMANENT refusal under `--as descriptor`, in every build: the
/// window text's "come back for the QR plate" would be false forever for a
/// shape the descriptor record can never carry.
#[test]
fn row_multi_under_descriptor() {
    assert_row(
        "multi-under-descriptor",
        &vector_input("neither/wsh-multi"),
        &["--as", "descriptor"],
        3,
        "the device's descriptor parser accepts `sortedmulti` and not `multi`. This \
         wallet can still be engraved: `--as md1` encodes `multi` policies (for \
         use-site paths md1 can represent -- otherwise no path carries it, and the \
         refusal says so). (`sortedmulti` differs from `multi` only in key ordering at \
         spend time -- it is not a synonym, so `me` will not rewrite it for you.)",
    );
}

#[test]
fn row_miniscript() {
    assert_row(
        "miniscript",
        &vector_input("neither/miniscript"),
        &["--as", "md1"],
        3,
        "`me` reads the descriptor family the device reads: single-sig and \
         `sortedmulti`, optionally under `sh`. This descriptor uses miniscript \
         fragments (`or_d`), which neither path handles in this release. `md encode` \
         accepts miniscript TEMPLATES -- a different tool and input form.",
    );
}

#[test]
fn row_threshold_exceeds_keys() {
    assert_row(
        "threshold-exceeds-keys",
        &vector_input("narrowed/threshold-exceeds-keys"),
        &["--as", "md1"],
        3,
        "threshold 5 of 2 keys can never be satisfied -- no combination of signatures \
         reaches 5. Funds sent to this wallet would be unspendable. Nothing was packed.",
    );
}

/// The one refusal in §6 that tells the operator to act NOW: the device derives
/// a real address for `k = 0`, so this refusal is the host's alone.
#[test]
fn row_threshold_below_one() {
    assert_row(
        "threshold-below-one",
        &vector_input("narrowed/threshold-zero"),
        &["--as", "md1"],
        3,
        "threshold 0 means NO signature is required: anyone who can see this script can \
         spend from it. This is almost certainly not the wallet you meant -- and if it \
         already holds funds, treat them as at risk now. Nothing was packed.",
    );
}

#[test]
fn row_key_count_exceeded() {
    assert_row(
        "key-count-exceeded",
        &vector_input("narrowed/sh-sortedmulti-16-keys"),
        &["--as", "md1"],
        3,
        "`sh(sortedmulti(…))` carries at most 15 keys -- there the multi's output script \
         IS the redeemScript, one 520-byte script element (BIP-383). `wsh(…)` and \
         `sh(wsh(…))` carry at most 20 (`OP_CHECKMULTISIG`); their redeemScript is 34 \
         bytes and the 520-byte limit never binds. This descriptor has 16 keys under \
         `sh(…)`. The device would accept it and derive addresses whose coins cannot be \
         spent.",
    );
}

/// Conjunct 8(a). **The cause clause is stated over ENTRIES**, per the
/// 2026-08-29 amendment: the row cannot fire for a BlueWallet file at all
/// (same-fingerprint collisions die earlier as `inconsistent header value`), so
/// naming "a duplicated cosigner LINE" advertised a route that does not exist.
#[test]
fn row_key_identity() {
    assert_row(
        "key-identity",
        &vector_input("gate/colliding-origin-sortedmulti"),
        &[],
        3,
        "this wallet description contradicts itself: keys 0 and 1 both claim origin \
         `dc567276/48h/0h/0h/2h` but name different keys -- one origin identifies \
         exactly one key, so no wallet matches this description. Check the export: one \
         of the two entries carries the wrong key, and a copied-and-edited cosigner is \
         the usual cause.",
    );
}

/// Conjunct 8(b), split from the row above because *"no wallet matches"* is
/// FALSE for a duplicate — this wallet exists, it is simply not the multisig
/// the file describes.
#[test]
fn row_key_identity_duplicate() {
    assert_row(
        "key-identity-duplicate",
        &vector_input("gate/duplicate-key-same-use-site"),
        &[],
        3,
        "keys 0 and 1 are the same key at the same derivation -- a threshold that needs \
         the same key twice is not the multisig this file describes, and it lets one \
         holder produce two of the required signatures. Remove the duplicate line, or \
         supply the missing cosigner's key.",
    );
}

#[test]
fn row_mixed_network() {
    assert_row(
        "mixed-network",
        &vector_input("narrowed/mixed-network"),
        &["--as", "md1"],
        3,
        "key 1 is `tpub` (testnet) while key 0 is `xpub` (mainnet). The device accepts \
         this descriptor and then cannot derive any address from it. All keys must \
         share one network.",
    );
}

/// The remedy names the **per-version** target — one template cannot serve
/// five, and four of the five are TESTNET keys.
#[test]
fn row_unsupported_key_version() {
    assert_row(
        "unsupported-key-version",
        &vector_input("version-gap/full-origin-ypub"),
        &["--as", "md1"],
        3,
        "`me` admits exactly `xpub`, `tpub`, `zpub`, `Ypub`, `Zpub`. This key is \
         `ypub`, whose equivalent is `xpub`: sh(wpkh([4bbaa801/49h/0h/0h]",
    );
}

#[test]
fn row_taproot_multisig() {
    assert_row(
        "taproot-multisig",
        &vector_input("narrowed/tr-sortedmulti"),
        &["--as", "md1"],
        3,
        "taproot multisig is `multi_a`/`sortedmulti_a` (BIP-387); `tr(sortedmulti(…))` \
         is not a valid descriptor even though the device's parser accepts it. Check \
         the export.",
    );
}

#[test]
fn row_multi_in_single_key_script() {
    assert_row(
        "multi-in-single-key-script",
        &vector_input("narrowed/wpkh-sortedmulti"),
        &["--as", "md1"],
        3,
        "a multisig policy cannot live inside a single-key script. The device's parser \
         accepts this spelling and then cannot derive any address from it (measured: \
         `address: multisig script: … unsupported descriptor`). The forms the device \
         derives are `wsh(sortedmulti(…))`, `sh(wsh(sortedmulti(…)))` and \
         `sh(sortedmulti(…))`.",
    );
}

#[test]
fn row_key_in_script_slot() {
    assert_row(
        "key-in-script-slot",
        &vector_input("narrowed/wsh-of-key"),
        &["--as", "md1"],
        3,
        "`wsh`/`sh` of a single key is not a wallet form the device can derive addresses \
         for (measured: `Supported=false`, `address: singlesig script: … unsupported \
         descriptor`). A single-key wallet is `pkh(…)`, `wpkh(…)`, `sh(wpkh(…))` or \
         `tr(…)`.",
    );
}

#[test]
fn row_use_site_hardened() {
    assert_row(
        "use-site-hardened",
        &vector_input("narrowed/use-site-hardened"),
        &["--as", "md1"],
        3,
        "a hardened use-site step cannot be derived from an xpub (BIP-32). The device \
         would silently derive the UNhardened child and display addresses for a wallet \
         that cannot exist, so this is refused on both `--as` paths.",
    );
}

#[test]
fn row_use_site_non_consecutive() {
    assert_row(
        "use-site-non-consecutive",
        &vector_input("narrowed/use-site-non-consecutive"),
        &["--as", "md1"],
        3,
        "the device derives only `<i;i+1>` pairs (receive; change). It accepts this \
         descriptor and then errors on every address.",
    );
}

/// Conjunct 7's closed set. "ACCEPTS" not "packs": admission is
/// build-independent, and which flag packs which member is §5.3's business.
/// §7 carries no row for this one — the closed-set residue is refused as
/// UNMEASURED, so there is nothing measured to pin.
#[test]
fn row_use_site_out_of_set() {
    assert_row(
        "use-site-out-of-set",
        &format!("wpkh([4bbaa801/84h/0h/0h]{XPUB}/0/1/*)"),
        &["--as", "md1"],
        3,
        "use-site paths `me` ACCEPTS: absent, `/*`, `/i/*`, `<i;i+1>`, `<i;i+1>/*`. This \
         one is `/0/1/*`, outside the set the device is measured to handle.",
    );
}

// ───────────────────────────────────────────────────────────────────────────
// §4.5 — the promotion rows
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn row_promotion_path_not_inferable() {
    assert_row(
        "promotion-path-not-inferable",
        &vector_input("promotion/08-origin-86h-refused"),
        &[],
        3,
        "is a single extended key. `me` can infer a whole wallet from one only when its \
         origin is `m/44h/0h/0h` (-> `pkh`), `m/84h/0h/0h` (-> `wpkh`) or `m/49h/0h/0h` \
         (-> `sh(wpkh)`). This one is `m/86h/0h/0h` -- taproot single-sig, which is not \
         inferable. Supply the descriptor instead: tr([4bbaa801/86h/0h/0h]",
    );
}

#[test]
fn row_promotion_account_not_zero() {
    assert_row(
        "promotion-account-not-zero",
        &vector_input("promotion/10-account-one-refused"),
        &[],
        3,
        "is a single extended key, and this one is `m/84h/0h/1h`. Only account 0 is \
         inferable. Supply the descriptor: wpkh([4bbaa801/84h/0h/1h]",
    );
}

#[test]
fn row_promotion_fingerprint_no_path() {
    assert_row(
        "promotion-fingerprint-no-path",
        &vector_input("promotion/12-fingerprint-no-path-refused"),
        &[],
        3,
        "gives a fingerprint with no derivation path, so there is nothing to match a \
         script against. Either give the full origin -- `[4bbaa801/84h/0h/0h]xpub6C9j4",
    );
}

#[test]
fn row_promotion_multisig_cosigner_key() {
    assert_row(
        "promotion-multisig-cosigner-key",
        &vector_input("promotion/03-bare-Zpub-refused"),
        &[],
        3,
        "a `Zpub` declares a MULTISIG account (`m/48h/0h/0h/2h`). A multisig cosigner \
         key is not a wallet -- supply the full descriptor (`wsh(sortedmulti(…))`), or a \
         BlueWallet setup file listing every cosigner.",
    );
}

#[test]
fn row_promotion_testnet_key() {
    assert_row(
        "promotion-testnet-key",
        &vector_input("promotion/15-bare-tpub-host-refused"),
        &[],
        3,
        "this is a testnet key. Its version byte would map to the MAINNET path \
         `m/44h/0h/0h`, which `me` will not assume. Supply the descriptor with its real \
         origin.",
    );
}

// ───────────────────────────────────────────────────────────────────────────
// §5.3 — the md1 representability rows (WINDOW-SUBSTITUTED)
// ───────────────────────────────────────────────────────────────────────────

/// **NOT window-substituted since S2.** §5.3's substitution fires only in a
/// build where `--as descriptor` refuses; the path ships, so this row prints
/// §6's own remedy — which names the flag that carries this exact shape. The
/// verdict, the cause and the named key are §6's and did not move.
#[test]
fn row_md1_fixed_index() {
    assert_row(
        "md1-fixed-index",
        &vector_input("md1-split/fixed-index"),
        &["--as", "md1"],
        3,
        "md1 cannot carry this wallet as written: key @0 ([dc567276/48h/0h/0h/2h]\
         xpub6DiYrfRwNn…EUhpan) uses `/0/*`, and key @1 \
         ([f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8…sY39Ge) uses `/0/*`, a single fixed \
         chain index, which has no md1 form -- encoding it would silently produce a \
         DIFFERENT wallet. Use `--as descriptor`, which carries `/0/*` exactly.",
    );
}

/// **NOT window-substituted since S2**, the same way as the row above. The
/// remedy names the offender as the encoder writes it (`/<0;1>`, leading slash
/// included), which is the same spelling the cause clause above it uses.
#[test]
fn row_md1_no_wildcard() {
    assert_row(
        "md1-no-wildcard",
        &vector_input("md1-split/multipath-no-wildcard"),
        &["--as", "md1"],
        3,
        "md1 cannot carry this wallet as written: key @0 ([dc567276/48h/0h/0h/2h]\
         xpub6DiYrfRwNn…EUhpan) uses `/<0;1>`, and key @1 \
         ([f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8…sY39Ge) uses `/<0;1>` with no trailing \
         wildcard, which has no md1 form -- encoding it would silently produce the \
         `<0;1>/*` wallet, which derives DIFFERENT addresses. Use `--as descriptor`, \
         which carries `/<0;1>` exactly.",
    );
}

// ───────────────────────────────────────────────────────────────────────────
// §10 — the address row
// ───────────────────────────────────────────────────────────────────────────

/// The reasoning — why an address is not a thing to engrave — is §10's and
/// stays OUTSIDE the quote, per the walk-W5 rule.
#[test]
fn row_bitcoin_address() {
    assert_row(
        "bitcoin-address",
        "bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a\n",
        &["--as", "md1"],
        3,
        "that is a bitcoin address, not a descriptor. No program on the device consumes \
         an address record.",
    );
}

// ───────────────────────────────────────────────────────────────────────────
// The rows §6 states over the `multi` twins, and the sibling matrices
// ───────────────────────────────────────────────────────────────────────────

/// **The spec amendment of 2026-08-29, as a gate.** Five §6 rows printed a
/// device-behaviour claim that is measurably FALSE for a `multi` input:
/// `bip380.Parse`'s script switch has a `sortedmulti` case and no `multi` case,
/// so the device refuses every `multi` form at PARSE and none reaches address
/// derivation.
///
/// Each row is reached by taking its own vector input and rewriting
/// `sortedmulti(` to `multi(` — the same construction the IMPL-P1 review used
/// to find them.
#[test]
fn no_device_behaviour_claim_transposes_to_a_multi_input() {
    let cases = [
        ("narrowed/tr-sortedmulti", "taproot-multisig"),
        ("narrowed/mixed-network", "mixed-network"),
        ("narrowed/use-site-hardened", "use-site-hardened"),
        (
            "narrowed/use-site-non-consecutive",
            "use-site-non-consecutive",
        ),
        ("narrowed/sh-sortedmulti-16-keys", "key-count-exceeded"),
    ];
    // The false sentences, verbatim as they were emitted before the amendment.
    let false_claims = [
        "even though the device's parser accepts it",
        "The device accepts this descriptor and then cannot derive",
        "The device would silently derive the UNhardened child",
        "It accepts this descriptor and then errors on every address",
        "The device would accept it and derive addresses whose coins cannot be spent",
    ];
    for (row, slug) in cases {
        let document = vector_input(row).replace("sortedmulti(", "multi(");
        let out = run(&document, &["--as", "md1"]);
        let err = String::from_utf8_lossy(&out.stderr).to_string();
        assert_eq!(out.status.code().unwrap(), 3, "{slug}: stderr:\n{err}");
        assert!(
            err.contains(
                // The taproot row splices the clause mid-sentence, so the
                // shared span starts after the article.
                "device's parser refuses `multi` outright, so this file never reaches \
                 address derivation there"
            ),
            "{slug}: the `multi` device clause is missing:\n{err}"
        );
        for claim in false_claims {
            assert!(
                !err.contains(claim),
                "{slug}: still prints a device claim that is false for `multi`: {claim:?}\n{err}"
            );
        }
    }

    // The sixth row of the class — the single-key wrapper — was already carved
    // out, and its remedy transposes too. It is the control: if the amendment
    // had been written as "all six take the new clause", this would red.
    let document = vector_input("narrowed/wpkh-sortedmulti").replace("sortedmulti(", "multi(");
    let out = run(&document, &["--as", "md1"]);
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        err.contains(
            "a multisig policy cannot live inside a single-key script on EITHER path. \
             Change the wrapper -- `wsh(multi(…))`, `sh(multi(…))` or `sh(wsh(multi(…)))` \
             -- and use `--as md1`, which carries those forms."
        ),
        "the single-key-wrapper row lost its own `multi` remedy:\n{err}"
    );
}

/// **§6's both-rows-fire case.** A descriptor mixing an (a)-shaped and an
/// (a″)-shaped key matches BOTH §5.3 rows: *"both fire, both are true, and both
/// name the same remedy — no precedence is needed."* §7 carries no row for it
/// (a gate row names exactly one `refusal_row`), which is why it is asserted
/// here instead.
#[test]
fn a_mixed_a_and_a2_input_fires_both_section_5_3_rows() {
    let document = format!(
        "wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]{XPUB}/0/*,[f245ae38/48h/0h/0h/2h]{XPUB2}/<0;1>))"
    );
    let out = run(&document, &["--as", "md1"]);
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert_eq!(out.status.code().unwrap(), 3, "stderr:\n{err}");
    assert!(
        err.contains("uses `/0/*`, a single fixed chain index"),
        "the (a) row did not fire:\n{err}"
    );
    assert!(
        err.contains("uses `/<0;1>` with no trailing wildcard"),
        "the (a″) row did not fire:\n{err}"
    );
}

// `the_window_refusal_has_two_variants` lived here — §11 item 5's sibling case,
// `--as descriptor` in a build where its path had not shipped, in both of the
// variants md1-representability chose between. S2 shipped the path: the same
// two inputs now PACK (`formats-happy/bip380-sortedmulti-multipath`) and REFUSE
// on their own §4.7/§5.3 grounds (`md1-split/fixed-index` — §5.3(a), reached
// through `--as md1`, `row_md1_fixed_index` above). Neither can reach a window
// text, so the sibling retired with §5.1's window and the row that named it.

/// **§11 item 5's five cases**, in one table so a missing case is visible.
///
/// `--as` omitted with an input at least one value CARRIES exits 2 with §5.1's
/// block; with an input nothing carries, the input's OWN refusal fires directly
/// at 3 — §5.4's carriage rule. A two-option menu whose options both refuse is
/// the dead-flag defect the choice block was ruled never to be.
///
/// **The FULL-BUILD truth table since S2.** Both `--as` values carry now, so
/// case 1's input is carried by both and case 3 needs a wallet NEITHER carries:
/// `wsh(multi(…/0/*))` — conjunct 1 refuses `multi` under `--as descriptor`
/// PERMANENTLY and md1 cannot represent a fixed `/0/*` (F-417). The shipped
/// witness, `md1-split/fixed-index`, moved to case 1 when the descriptor path
/// shipped: it is the exit-code flip S2 made, 3 → 2. Every `forbid` is the
/// build marking, which no path here may ever print again.
#[test]
fn item_5_the_five_case_matrix() {
    struct Case {
        what: &'static str,
        document: String,
        flags: Vec<&'static str>,
        exit: i32,
        want: &'static str,
        forbid: &'static str,
    }
    let cases = vec![
        Case {
            what: "1. carried -- the choice block",
            document: vector_input("formats-happy/bip380-sortedmulti-multipath"),
            flags: vec![],
            exit: 2,
            want: "this input is a wallet descriptor, and `--as` decides how it is packed.",
            forbid: "not available in this build",
        },
        Case {
            what: "2. inadmissible, `--as` omitted -- the admission refusal, directly",
            document: vector_input("narrowed/threshold-zero"),
            flags: vec![],
            exit: 3,
            want: "threshold 0 means NO signature is required",
            forbid: "`--as` decides how it is packed",
        },
        Case {
            what: "3. carried by NEITHER value -- the input's own refusal, directly",
            document: vector_input("neither/wsh-multi-fixed-path"),
            flags: vec![],
            exit: 3,
            want: "No `me` path engraves this file as written, in any build.",
            forbid: "`--as` decides how it is packed",
        },
        Case {
            what: "3b. the S3 case-3 witness is CARRIED now -- the exit-code flip S2 made",
            document: vector_input("md1-split/fixed-index"),
            flags: vec![],
            exit: 2,
            want: "this input is a wallet descriptor, and `--as` decides how it is packed.",
            forbid: "not available in this build",
        },
        Case {
            what: "4. inadmissible WITH `--as descriptor` -- admission precedes the window",
            document: vector_input("narrowed/threshold-zero"),
            flags: vec!["--as", "descriptor"],
            exit: 3,
            want: "threshold 0 means NO signature is required",
            forbid: "not available in this build",
        },
        Case {
            what: "5. a `multi` form WITH `--as descriptor` -- conjunct 1's PERMANENT refusal",
            document: vector_input("neither/wsh-multi"),
            flags: vec!["--as", "descriptor"],
            exit: 3,
            want: "the device's descriptor parser accepts `sortedmulti` and not `multi`.",
            forbid: "not available in this build",
        },
    ];
    let n = cases.len();
    for c in cases {
        let out = run(&c.document, &c.flags);
        let err = String::from_utf8_lossy(&out.stderr).to_string();
        assert_eq!(
            out.status.code().unwrap(),
            c.exit,
            "{}: exit\n{err}",
            c.what
        );
        assert!(
            err.contains(c.want),
            "{}: wanted {:?}\n{err}",
            c.what,
            c.want
        );
        assert!(
            !err.contains(c.forbid),
            "{}: must not say {:?}\n{err}",
            c.what,
            c.forbid
        );
        // A non-zero exit writes nothing to stdout, and case 4 and 5 in
        // particular must not have packed anything before refusing.
        if c.exit != 0 {
            assert!(
                out.stdout.is_empty(),
                "{}: a refusal wrote {} bytes to stdout",
                c.what,
                out.stdout.len()
            );
        }
    }
    assert_eq!(n, 6, "§11 item 5's five cases, plus S2's flipped witness");
}
