//! F-449 stage 4a (`design/SPEC_liana_unspendable_internal_key.md` §8.9's two
//! `me` rows, §8b, §6a, §9a): what `me` does with an md1 at wire version 8,
//! and with one at a version it does not read.
//!
//! Every fixture is MEASURED, never hand-built from a reading of the spec:
//! - `V8_TEMPLATE`: md-cli 0.19.0 (descriptor-mnemonic `cf35d61a`),
//!   `md encode "tr(UNSPENDABLE(liana),{pk(@0/48'/0'/0'/3'/<0;1>/*),pk(@1/48'/0'/1'/3'/<0;1>/*)})"`.
//!   A 2-key TEMPLATE at wire version 8. me 0.10.0 (md-codec 0.42) bundled it
//!   as "backup needs 1 public plate" with no template note (key_slots 0).
//! - `V12_SINGLE`: descriptor-mnemonic `crates/md-cli/tests/cli_repair_unsupported_version.rs`
//!   `V12_CLEAN` -- BCH-clean, single-string, header version 12.
//! - `V4_UNDECODABLE`: `wrap_payload([0x20, 0xff × 12], 100)` -- a v4 single
//!   string that passes its checksum and fails decode (`BitStreamTruncated`).
//! - `v12_chunk()`: `wrap_payload([0xC8, 0 × 12], 100)` -- first symbol
//!   0b11001, chunked flag set, chunk-header version 12.
#![cfg(unix)]

use assert_cmd::Command;

const V8_TEMPLATE: &str = "md1cpfdsssj6tvyywtsqrq0zjs4n7gdve74ar402";
const V12_SINGLE: &str = "md1uzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg";
const V4_UNDECODABLE: &str = "md1yrlllllllllllllllllltrn9jd5mjtn77";
const V4_ORIGINLESS_TEMPLATE: &str = "md1yppqqxqu22z54hcefkda7r46w";
/// The same policy, `md encode --force-chunked` (md 0.18.0): one chunk, a WHOLE
/// set, with no origins.
const V4_ORIGINLESS_CHUNKED: &str = "md1ffxweqqpqggqps8zjs4qqyhaq7eqm6qq6k";
/// md-codec's own rendering of the refusal. Asserted as a substring so the
/// accepted set is read from the codec, never restated here.
const V12_NAMED: &str = "wire-format version mismatch: got 12; accepted versions: 4, 8";

fn v12_chunk() -> String {
    let mut payload = vec![0u8; 13];
    payload[0] = 0xC8;
    md_codec::codex32::wrap_payload(&payload, 100).unwrap()
}

struct Out {
    code: i32,
    out: String,
    err: String,
}

fn me(args: &[&str], stdin: &str) -> Out {
    let o = Command::cargo_bin("me")
        .unwrap()
        .args(args)
        .write_stdin(stdin.to_string())
        .output()
        .unwrap();
    Out {
        code: o.status.code().unwrap(),
        out: String::from_utf8_lossy(&o.stdout).into_owned(),
        err: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

// ---- Task 1: the unpin. Both FAIL on me 0.10.0 (md-codec 0.42). ----------

/// The version-8 template is counted from what it ENCODES. On md-codec 0.42
/// the decode failed, was skipped, and the checklist said "1 public plate"
/// with no TEMPLATE note -- cut that one plate and the wallet is gone.
#[test]
fn a_version_8_template_plate_is_counted_from_what_it_encodes() {
    let r = me(&["bundle"], &format!("{V8_TEMPLATE}\n"));
    assert_eq!(r.code, 0, "{}", r.err);
    let m: serde_json::Value = serde_json::from_str(&r.out).expect("manifest JSON");
    assert_eq!(m["key_slots"], 2, "{}", r.out);
    assert_eq!(m["keyless_template"], true, "{}", r.out);
    assert!(r.err.contains("this md1 is a TEMPLATE"), "{}", r.err);
    assert!(r.err.contains("declares 2 key slots"), "{}", r.err);
}

/// `me sysw` confirms a version-8 card: the §12.6 walk decodes it.
#[test]
fn sysw_confirms_a_version_8_card() {
    assert_eq!(
        mnemonic_engrave::sysw::record::mdmk_unconfirmed(&[V8_TEMPLATE.to_string()]),
        Vec::<usize>::new()
    );
}

// ---- Task 2: F-635, the fail-open at bundle.rs's unchunked path. ----------

/// §8.9 row "`me` fail-open": a bundle never states a plate count it did
/// not compute.
#[test]
fn bundle_refuses_an_unchunked_plate_at_an_unsupported_version() {
    let r = me(&["bundle"], &format!("{V12_SINGLE}\n"));
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(r.out.is_empty(), "no manifest: {}", r.out);
    assert!(!r.err.contains("backup needs"), "no count: {}", r.err);
    assert!(r.err.contains("unsupported md1 wire version"), "{}", r.err);
    assert!(r.err.contains(V12_NAMED), "the version is NAMED: {}", r.err);
    assert!(!r.err.contains("does not decode"), "{}", r.err);
}

#[test]
fn bundle_refuses_an_unchunked_plate_that_does_not_decode() {
    let r = me(&["bundle"], &format!("{V4_UNDECODABLE}\n"));
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(r.out.is_empty(), "no manifest: {}", r.out);
    assert!(!r.err.contains("backup needs"), "no count: {}", r.err);
    assert!(r.err.contains("md1 plate does not decode"), "{}", r.err);
}

/// F-635 is not only a version-8 problem: this is a REAL encoder output at
/// wire version 4 -- md-cli 0.19.0, `md encode
/// "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{pk(@0/<0;1>/*),pk(@1/<0;1>/*)})"`
/// -- an origin-less 2-key TEMPLATE that strict decode refuses
/// (`MissingExplicitOrigin`). me 0.10.0 bundled it as "backup needs 1 public
/// plate" with no template note; the chunked shape of the same class was
/// already refused (`SetIncompleteMd`).
///
/// Whole-branch review M1: the refusal must say what is TRUE -- `md decode`
/// reads this plate (VERIFY-ME), so "does not decode" is false -- and name
/// the remedy.
#[test]
fn a_version_4_origin_less_template_is_refused_not_miscounted() {
    let r = me(&["bundle"], &format!("{V4_ORIGINLESS_TEMPLATE}\n"));
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(r.out.is_empty(), "{}", r.out);
    assert_origin_refusal(&r.err);
}

/// The chunked shape of the same plate: the set is WHOLE, so it must not be
/// called "incomplete/inconsistent" either.
#[test]
fn a_chunked_origin_less_template_is_not_called_incomplete() {
    let r = me(&["bundle"], &format!("{V4_ORIGINLESS_CHUNKED}\n"));
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(r.out.is_empty(), "{}", r.out);
    assert!(!r.err.contains("incomplete"), "{}", r.err);
    assert_origin_refusal(&r.err);
}

fn assert_origin_refusal(err: &str) {
    assert!(!err.contains("backup needs"), "no count: {err}");
    assert!(
        !err.contains("does not decode"),
        "md decode reads it: {err}"
    );
    assert!(
        err.contains("md1 plate carries no key origin for @0"),
        "says what is true: {err}"
    );
    assert!(
        err.contains("`md encode --path <PATH>`") && err.contains("complete set that carries them"),
        "names the remedy: {err}"
    );
}

/// A decodable plate beside the bad one does not rescue the bundle: the
/// refusal is per plate, not "at least one plate decoded".
#[test]
fn a_good_plate_does_not_carry_a_bad_one() {
    let r = me(&["bundle"], &format!("{V8_TEMPLATE}\n{V12_SINGLE}\n"));
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(r.out.is_empty(), "{}", r.out);
    // R0 Nit: the refusal must name the BAD plate's version, so a refusal
    // caused by the good plate cannot pass this test.
    assert!(r.err.contains(V12_NAMED), "{}", r.err);
}

/// The chunked shape already refused; it now NAMES the version too.
#[test]
fn bundle_names_the_version_of_a_chunk_it_cannot_read() {
    let r = me(&["bundle"], &format!("{}\n", v12_chunk()));
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(r.err.contains(V12_NAMED), "{}", r.err);
}

// ---- Task 3: §6a on `me sysw` -- REPORTED, never reduced to "unconfirmed". -

#[test]
fn the_walk_keeps_the_version_for_both_shapes() {
    use mnemonic_engrave::sysw::record::{mdmk_unconfirmed_why, Unconfirmed};
    for s in [V12_SINGLE.to_string(), v12_chunk()] {
        assert_eq!(
            mdmk_unconfirmed_why(&[s.clone()]),
            vec![(0, Unconfirmed::UnsupportedWireVersion(12))],
            "{s}"
        );
    }
    assert_eq!(
        mdmk_unconfirmed_why(&[V4_UNDECODABLE.to_string()]),
        vec![(0, Unconfirmed::Undecodable)]
    );
}

#[test]
fn pack_names_the_wire_version_and_does_not_call_it_undecodable() {
    let r = me(&["sysw", "pack", "--no-passphrase", V12_SINGLE], "");
    assert_eq!(r.code, 0, "D6: it WARNS and proceeds: {}", r.err);
    assert!(
        r.err.contains(&format!(
            "record 0, as given (records count from 0): an md1 this build of me does not \
             read -- {V12_NAMED}"
        )),
        "{}",
        r.err
    );
    assert!(r.err.contains("SECRET"), "{}", r.err);
    assert!(!r.err.contains("could not decode"), "{}", r.err);
}

#[test]
fn show_names_the_wire_version_beside_the_record() {
    let dir = tempfile::tempdir().unwrap();
    let bin = dir.path().join("p.bin");
    let packed = me(
        &[
            "sysw",
            "pack",
            "--no-passphrase",
            V12_SINGLE,
            "--out",
            bin.to_str().unwrap(),
        ],
        "",
    );
    assert_eq!(packed.code, 0, "{}", packed.err);
    let r = me(&["sysw", "show", bin.to_str().unwrap()], "");
    assert_eq!(r.code, 0, "{}", r.err);
    assert!(
        r.out.contains(&format!(
            "public record 0: md1/mk1 — unconfirmed — engraveable, but the device REPLACES \
             the legend; an md1 this build of me does not read -- {V12_NAMED}"
        )),
        "{}",
        r.out
    );
}

/// `--expect descriptor` read the same walk and called a whole single card
/// "present, but the set does not reassemble" -- false for this card.
#[test]
fn expect_descriptor_names_the_version_instead_of_calling_it_incomplete() {
    let dir = tempfile::tempdir().unwrap();
    let bin = dir.path().join("p.bin");
    let r = me(
        &[
            "sysw",
            "pack",
            "--no-passphrase",
            "--expect",
            "descriptor",
            V12_SINGLE,
            "--out",
            bin.to_str().unwrap(),
        ],
        "",
    );
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(!bin.exists(), "nothing written");
    assert!(r.err.contains(V12_NAMED), "{}", r.err);
    assert!(!r.err.contains("does not reassemble"), "{}", r.err);
}
