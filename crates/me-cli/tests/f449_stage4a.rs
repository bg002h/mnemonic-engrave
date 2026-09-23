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
