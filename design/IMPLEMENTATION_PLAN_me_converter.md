# `mnemonic-engrave` Converter (`me`) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the host-side Rust CLI `mnemonic-engrave` (binary `me`) that validates a single constellation string and emits a TLV-wrapped NDEF Text record for the public `md1`/`mk1` strings, refusing the secret `ms1`.

**Architecture:** A small library crate (`crates/me-cli`) with four focused modules — `classify` (HRP → format), `validate` (per-string BCH check via the sibling codecs), `ndef` (TLV-wrapped Text-record encode/decode), `convert` (the pipeline) — plus a thin `main` for the CLI surface (stdin-only input, binary→stdout, human→stderr). Validation is **per-string** (single chunk OK), never whole-card, because `mk1` cards are normally multi-chunk and the engraver works one plate at a time.

**Tech Stack:** Rust (edition 2021), `clap` (derive), `md-codec` 0.36 / `mk-codec` 0.4 (crates.io, for per-string validation), `zeroize` 1.8 (input hygiene). A tiny Go harness (importing `seedhammer.com/nfc/ndef`) backs the cross-language round-trip test.

> **Spec:** `design/SPEC_seedhammer_engrave.md` (architect R-loop GREEN, R0→R3). This plan implements Component A (§5) + the NDEF contract (§6) + the converter tests (§9). **Per your standard, this plan-doc must pass its own architect R0 gate (loop to 0C/0I) before any code is written.**

> **TDD note:** the pure modules (`ndef`, `classify`, `validate`, the `convert` pipeline) are small enough that each task writes the module and its tests together, then runs to green; the binary's user-facing behavior is covered by integration tests in Task 5. This is a deliberate, documented deviation from strict per-function red-green.

> **Plan status:** architect gate **GREEN** (plan-R0 → plan-R1, 0C/0I; reports in `design/agent-reports/me-converter-plan-R{0,1}-review.md`). Eligible for execution.

---

## File Structure

| File | Responsibility |
|---|---|
| `Cargo.toml` (root) | Workspace manifest; member `crates/me-cli`. |
| `crates/me-cli/Cargo.toml` | Package `mnemonic-engrave`, binary `me`; deps. |
| `crates/me-cli/src/lib.rs` | Re-exports modules; houses `convert()` pipeline + `ConvertError`. |
| `crates/me-cli/src/classify.rs` | `Format` enum; `classify(&str)` by bech32 HRP (case-insensitive). |
| `crates/me-cli/src/validate.rs` | `validate(Format, &str)` — per-string BCH via `md_codec::codex32::unwrap_string` / `mk_codec::string_layer::decode_string`. |
| `crates/me-cli/src/ndef.rs` | `encode_text_tlv(&str)` + `decode_text_tlv(&[u8])` (NFC Forum T2T/T5T, well-known Text record). |
| `crates/me-cli/src/main.rs` | CLI: parse args, read stdin/`--in`, route binary→stdout / human→stderr, exit codes. |
| `crates/me-cli/tests/cli.rs` | Integration tests of the binary (stdin → bytes, refusal exit code). |
| `crates/me-cli/tests/vectors/` | Committed golden `.ndef` byte files. |
| `firmware/ndef-roundtrip/{go.mod,main.go}` | Go harness: reads NDEF bytes on stdin via `seedhammer.com/nfc/ndef`, prints recovered text. |

---

## Task 0: Workspace + crate skeleton

**Files:**
- Create: `Cargo.toml` (root), `crates/me-cli/Cargo.toml`, `crates/me-cli/src/lib.rs`, `crates/me-cli/src/main.rs`

- [ ] **Step 1: Create the workspace root manifest**

Create `Cargo.toml`:
```toml
[workspace]
resolver = "2"
members = ["crates/me-cli"]
```

- [ ] **Step 2: Create the crate manifest**

Create `crates/me-cli/Cargo.toml`:
```toml
[package]
name = "mnemonic-engrave"
version = "0.1.0"
edition = "2021"
description = "Convert m-format constellation strings (md1/mk1) into NFC NDEF payloads for SeedHammer II; refuses secret ms1."
license = "MIT"

[[bin]]
name = "me"
path = "src/main.rs"

[lib]
name = "mnemonic_engrave"
path = "src/lib.rs"

[dependencies]
md-codec = "0.36"
mk-codec = "0.4"
clap = { version = "4", features = ["derive"] }
zeroize = "1.8"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

- [ ] **Step 3: Create stub `lib.rs`**

Create `crates/me-cli/src/lib.rs`:
```rust
//! `mnemonic-engrave` (`me`) — converts public constellation strings (md1/mk1)
//! into NFC NDEF payloads for SeedHammer II. Refuses the secret ms1.

pub mod classify;
pub mod ndef;
pub mod validate;
```

- [ ] **Step 4: Create stub `main.rs`**

Create `crates/me-cli/src/main.rs`:
```rust
fn main() {
    eprintln!("me: not yet implemented");
    std::process::exit(2);
}
```

- [ ] **Step 5: Create empty module files so the crate compiles**

Create `crates/me-cli/src/classify.rs`, `crates/me-cli/src/validate.rs`, `crates/me-cli/src/ndef.rs` each containing only:
```rust
//! placeholder — implemented in a later task
```

- [ ] **Step 6: Build to verify the skeleton compiles**

Run: `cargo build`
Expected: compiles (downloads md-codec/mk-codec/clap/zeroize), no errors.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml crates/me-cli/Cargo.toml crates/me-cli/src
git commit -s -m "feat(me): workspace + crate skeleton"
```

---

## Task 1: NDEF encode/decode (`ndef.rs`)

**Files:**
- Modify: `crates/me-cli/src/ndef.rs`

- [ ] **Step 1: Write the module and its tests**

Replace `crates/me-cli/src/ndef.rs` with:
```rust
//! NDEF well-known Text record, TLV-wrapped for NFC Forum Type-2/Type-5 tags,
//! per `design/SPEC_seedhammer_engrave.md` §6. Matches `nfc/ndef/ndef.go`.

/// NDEF record header: MB=1 ME=1 CF=0 SR=1 IL=0 TNF=001 (well-known) => 0xD1.
const NDEF_HEADER: u8 = 0xD1;
const TYPE_TEXT: u8 = b'T'; // 0x54
const STATUS_UTF8_NOLANG: u8 = 0x00; // bit7=0 (UTF-8), lang-code length 0
const TLV_NDEF: u8 = 0x03;
const TLV_TERMINATOR: u8 = 0xFE;

/// Errors from NDEF encoding.
#[derive(Debug, PartialEq, Eq)]
pub enum NdefError {
    /// Text/message too large for the single-byte short-record/TLV length form.
    TooLong(usize),
}

impl std::fmt::Display for NdefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NdefError::TooLong(n) => write!(f, "payload too long for a short NDEF record: {n} bytes"),
        }
    }
}
impl std::error::Error for NdefError {}

/// Build a bare NDEF message: one well-known Text record (UTF-8, empty language).
pub fn text_record(text: &str) -> Result<Vec<u8>, NdefError> {
    let payload_len = 1 + text.len(); // status byte + text bytes
    if payload_len > u8::MAX as usize {
        return Err(NdefError::TooLong(text.len()));
    }
    let mut out = Vec::with_capacity(4 + payload_len);
    out.push(NDEF_HEADER);
    out.push(0x01); // type length = 1
    out.push(payload_len as u8); // payload length (SR=1, single byte)
    out.push(TYPE_TEXT);
    out.push(STATUS_UTF8_NOLANG);
    out.extend_from_slice(text.as_bytes());
    Ok(out)
}

/// Wrap an NDEF message in the NFC Forum NDEF TLV (`0x03 <len> .. 0xFE`),
/// 1-byte length form (len < 255).
pub fn tlv_wrap(message: &[u8]) -> Result<Vec<u8>, NdefError> {
    if message.len() >= 0xFF {
        return Err(NdefError::TooLong(message.len()));
    }
    let mut out = Vec::with_capacity(message.len() + 3);
    out.push(TLV_NDEF);
    out.push(message.len() as u8);
    out.extend_from_slice(message);
    out.push(TLV_TERMINATOR);
    Ok(out)
}

/// Canonical encoding: TLV-wrapped NDEF Text record carrying `text`.
pub fn encode_text_tlv(text: &str) -> Result<Vec<u8>, NdefError> {
    tlv_wrap(&text_record(text)?)
}

/// Minimal decoder mirroring SeedHammer's reader: unwrap the NDEF TLV, parse a
/// single well-known Text record, return the UTF-8 text. Used for the
/// round-trip self-test; `None` on any structural mismatch.
pub fn decode_text_tlv(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 2 || bytes[0] != TLV_NDEF {
        return None;
    }
    let len = bytes[1] as usize;
    let msg = bytes.get(2..2 + len)?;
    decode_text_record(msg)
}

fn decode_text_record(msg: &[u8]) -> Option<String> {
    if msg.len() < 3 {
        return None;
    }
    let flags = msg[0];
    if flags & 0x07 != 0x01 {
        return None; // TNF must be well-known
    }
    let type_len = msg[1] as usize;
    let plen = msg[2] as usize;
    let rest = &msg[3..];
    let typ = rest.get(..type_len)?;
    if typ != [TYPE_TEXT] {
        return None;
    }
    let payload = rest.get(type_len..type_len + plen)?;
    let status = *payload.first()?;
    if status & 0x80 != 0 {
        return None; // UTF-16 unsupported (matches ndef.go)
    }
    let lang_len = (status & 0x3F) as usize;
    let text_bytes = payload.get(1 + lang_len..)?;
    std::str::from_utf8(text_bytes).ok().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_expected_bytes() {
        // "md1q" -> TLV(03,len) D1 01 plen 54 00 <text>
        let got = encode_text_tlv("md1q").unwrap();
        let expected: Vec<u8> = vec![
            0x03, 0x09, // TLV: NDEF, message length 9
            0xD1, 0x01, 0x05, 0x54, 0x00, // header, typelen, plen(=1+4), 'T', status
            b'm', b'd', b'1', b'q', // text
            0xFE, // terminator
        ];
        assert_eq!(got, expected);
    }

    #[test]
    fn round_trips() {
        let s = "mk1qpzry9x8gf2tv";
        let bytes = encode_text_tlv(s).unwrap();
        assert_eq!(decode_text_tlv(&bytes).as_deref(), Some(s));
    }

    #[test]
    fn rejects_oversize() {
        let big = "a".repeat(255);
        assert_eq!(text_record(&big), Err(NdefError::TooLong(255)));
    }
}
```

- [ ] **Step 2: Run the tests to verify they pass**

Run: `cargo test -p mnemonic-engrave --lib ndef -- --nocapture`
Expected: 3 tests PASS (`encodes_expected_bytes`, `round_trips`, `rejects_oversize`).

- [ ] **Step 3: Commit**

```bash
git add crates/me-cli/src/ndef.rs
git commit -s -m "feat(me): NDEF TLV-wrapped Text record encode/decode"
```

---

## Task 2: HRP classification (`classify.rs`)

**Files:**
- Modify: `crates/me-cli/src/classify.rs`

- [ ] **Step 1: Write the module and its tests**

Replace `crates/me-cli/src/classify.rs` with:
```rust
//! Classify a constellation string by its bech32 HRP (the text before the
//! first `1` separator), case-insensitively.

/// The three constellation formats this tool understands.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Format {
    /// md1 — wallet descriptor / policy (public).
    Md,
    /// mk1 — xpubs (public).
    Mk,
    /// ms1 — secret entropy (refused; never over RF).
    Ms,
}

/// Why a string could not be classified.
#[derive(Debug, PartialEq, Eq)]
pub enum ClassifyError {
    /// No `1` separator, or the HRP was empty.
    NoSeparator,
    /// HRP present but not one of md/mk/ms.
    UnknownHrp(String),
}

impl std::fmt::Display for ClassifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassifyError::NoSeparator => write!(f, "not a bech32 string (no '1' separator / empty HRP)"),
            ClassifyError::UnknownHrp(h) => write!(f, "unrecognized HRP '{h}' (expected md, mk, or ms)"),
        }
    }
}
impl std::error::Error for ClassifyError {}

/// Determine the format from the HRP. Trims surrounding whitespace and lowercases
/// the HRP before matching (bech32 is case-insensitive).
pub fn classify(s: &str) -> Result<Format, ClassifyError> {
    let s = s.trim();
    let sep = s.find('1').ok_or(ClassifyError::NoSeparator)?;
    if sep == 0 {
        return Err(ClassifyError::NoSeparator);
    }
    match s[..sep].to_ascii_lowercase().as_str() {
        "md" => Ok(Format::Md),
        "mk" => Ok(Format::Mk),
        "ms" => Ok(Format::Ms),
        other => Err(ClassifyError::UnknownHrp(other.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_each_hrp() {
        assert_eq!(classify("md1yqpqqxqq8xtwhw4xwn4qh"), Ok(Format::Md));
        assert_eq!(classify("mk1qpzry9x8gf2tv"), Ok(Format::Mk));
        assert_eq!(classify("ms10entrsqqqqqqq"), Ok(Format::Ms));
    }

    #[test]
    fn is_case_insensitive_and_trims() {
        assert_eq!(classify("  MD1QQPQ  "), Ok(Format::Md));
    }

    #[test]
    fn rejects_unknown_and_malformed() {
        assert_eq!(classify("xx1qqqq"), Err(ClassifyError::UnknownHrp("xx".into())));
        assert_eq!(classify("noseparator"), Err(ClassifyError::NoSeparator));
        assert_eq!(classify("1leadingsep"), Err(ClassifyError::NoSeparator));
    }
}
```

- [ ] **Step 2: Run the tests to verify they pass**

Run: `cargo test -p mnemonic-engrave --lib classify`
Expected: 3 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/me-cli/src/classify.rs
git commit -s -m "feat(me): HRP classification (md/mk/ms)"
```

---

## Task 3: Per-string validation (`validate.rs`)

**Files:**
- Modify: `crates/me-cli/src/validate.rs`

> **Validation contract:** `md1` → `md_codec::codex32::unwrap_string(s)` (HRP `md` + `bch_verify_regular`, returns raw symbols; works for a single-string md1 *and* a single chunk). `mk1` → `mk_codec::string_layer::decode_string(s)` (per-string header + BCH, regular or long). Neither reassembles a multi-chunk card — that is recovery-time. `ms1` is never validated here (refused upstream).

- [ ] **Step 1: Write the module + failing tests**

Replace `crates/me-cli/src/validate.rs` with:
```rust
//! Per-string (single-chunk) validation of public constellation strings.
//! Confirms HRP + BCH checksum so a corrupted string is never engraved.

use crate::classify::Format;

/// A validation failure, carrying the underlying codec error for a useful message.
#[derive(Debug)]
pub enum ValidateError {
    /// md1 string failed `md_codec` per-string checks.
    Md(md_codec::Error),
    /// mk1 string failed `mk_codec` per-string checks.
    Mk(mk_codec::Error),
    /// mk1 string was not pristine — `decode_string` had to BCH-correct N
    /// symbol(s). We refuse non-pristine input rather than engrave a string
    /// that needed repair (the converter engraves the input verbatim).
    MkCorrected(usize),
}

impl std::fmt::Display for ValidateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidateError::Md(e) => write!(f, "invalid md1 string: {e}"),
            ValidateError::Mk(e) => write!(f, "invalid mk1 string: {e}"),
            ValidateError::MkCorrected(n) => write!(
                f,
                "mk1 string is not pristine: it required {n} BCH correction(s) — fix the input \
                 rather than engrave a string that needed repair"
            ),
        }
    }
}
impl std::error::Error for ValidateError {}

/// Validate one md1/mk1 string at the per-string BCH level, requiring PRISTINE
/// input. md1 (`unwrap_string`) is a pure verify — any corruption is rejected.
/// mk1 (`decode_string`) BCH error-CORRECTS up to 4 symbols, so we additionally
/// reject any string that needed correction (`corrections_applied != 0`): the
/// converter engraves the input verbatim, so a non-pristine input means the user
/// should fix their source, not engrave a string that required repair.
/// `Format::Ms` must never reach this function (it is refused before validation).
pub fn validate(fmt: Format, s: &str) -> Result<(), ValidateError> {
    match fmt {
        Format::Md => md_codec::codex32::unwrap_string(s)
            .map(|_| ())
            .map_err(ValidateError::Md),
        Format::Mk => {
            let decoded = mk_codec::string_layer::decode_string(s).map_err(ValidateError::Mk)?;
            if decoded.corrections_applied != 0 {
                return Err(ValidateError::MkCorrected(decoded.corrections_applied));
            }
            Ok(())
        }
        Format::Ms => panic!("validate() called on ms1 — must be refused before validation"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Known-good vectors, current as of md-codec 0.36 / mk-codec 0.4. To refresh:
    //   md1: regenerate via the `md` CLI or `md_codec::encode_md1_string(...)`; the
    //        canonical v0.30 example is asserted at
    //        descriptor-mnemonic/crates/md-cli/tests/smoke.rs:21. NOTE: test_vectors.rs
    //        holds TEMPLATES, not strings — you must encode to get an md1 literal.
    //   mk1: copy a string from mnemonic-key/crates/mk-codec/src/test_vectors/v0.1.json
    const MD1_VALID: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
    const MK1_VALID: &str =
        "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";

    #[test]
    fn accepts_valid_md1() {
        assert!(validate(Format::Md, MD1_VALID).is_ok());
    }

    #[test]
    fn accepts_valid_mk1() {
        assert!(validate(Format::Mk, MK1_VALID).is_ok());
    }

    #[test]
    fn rejects_corrupted_md1() {
        // Flip one data character; the BCH checksum must reject it.
        let mut bad = MD1_VALID.to_string();
        let last = bad.pop().unwrap();
        bad.push(if last == 'q' { 'p' } else { 'q' });
        assert!(validate(Format::Md, &bad).is_err());
    }

    #[test]
    fn rejects_corrupted_mk1() {
        // A single flipped symbol is BCH-correctable, so decode_string returns
        // Ok with corrections_applied=1 — which validate() rejects as non-pristine.
        let mut bad = MK1_VALID.to_string();
        let last = bad.pop().unwrap();
        bad.push(if last == 'q' { 'p' } else { 'q' });
        assert!(matches!(validate(Format::Mk, &bad), Err(ValidateError::MkCorrected(_))));
    }
}
```

- [ ] **Step 2: Confirm the vectors are current**

Run: `cargo test -p mnemonic-engrave --lib validate::tests::accepts_valid_md1 validate::tests::accepts_valid_mk1`
Expected: both PASS. If a vector FAILS, it is stale: for mk1 copy a current string from `mk-codec/src/test_vectors/v0.1.json`; for md1 regenerate it (`md` CLI or `md_codec::encode_md1_string`), since `test_vectors.rs` stores templates, not strings. Then re-run.

- [ ] **Step 3: Run the full module tests**

Run: `cargo test -p mnemonic-engrave --lib validate`
Expected: 4 tests PASS (2 accept, 2 reject-on-corruption).

- [ ] **Step 4: Commit**

```bash
git add crates/me-cli/src/validate.rs
git commit -s -m "feat(me): per-string md1/mk1 validation via sibling codecs"
```

---

## Task 4: Convert pipeline (`lib.rs`)

**Files:**
- Modify: `crates/me-cli/src/lib.rs`

- [ ] **Step 1: Add `convert()` + `ConvertError` and failing tests**

Replace `crates/me-cli/src/lib.rs` with:
```rust
//! `mnemonic-engrave` (`me`) — converts public constellation strings (md1/mk1)
//! into NFC NDEF payloads for SeedHammer II. Refuses the secret ms1.

pub mod classify;
pub mod ndef;
pub mod validate;

use classify::{ClassifyError, Format};

/// Failure modes of the end-to-end conversion.
#[derive(Debug)]
pub enum ConvertError {
    /// Could not classify the HRP.
    Classify(ClassifyError),
    /// `ms1` was supplied — refused: secret material must never go over RF.
    RefusedSecret,
    /// The md1/mk1 string failed validation.
    Validate(validate::ValidateError),
    /// NDEF encoding failed (e.g. string too long for one record).
    Ndef(ndef::NdefError),
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertError::Classify(e) => write!(f, "{e}"),
            ConvertError::RefusedSecret => write!(
                f,
                "refusing to emit ms1 over NFC: ms1 is secret seed entropy and must never be \
                 transmitted by radio (RF eavesdropping risk). Enter it by hand on the device: \
                 New > Input Seed > CODEX32."
            ),
            ConvertError::Validate(e) => write!(f, "{e}"),
            ConvertError::Ndef(e) => write!(f, "{e}"),
        }
    }
}
impl std::error::Error for ConvertError {}

/// Validate one public constellation string and return its TLV-wrapped NDEF
/// bytes. Refuses `ms1`. The input is trimmed; the verbatim trimmed string is
/// what gets encoded (and later engraved).
pub fn convert(input: &str) -> Result<Vec<u8>, ConvertError> {
    let s = input.trim();
    let fmt = classify::classify(s).map_err(ConvertError::Classify)?;
    if fmt == Format::Ms {
        return Err(ConvertError::RefusedSecret);
    }
    validate::validate(fmt, s).map_err(ConvertError::Validate)?;
    ndef::encode_text_tlv(s).map_err(ConvertError::Ndef)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MD1_VALID: &str = "md1yqpqqxqq8xtwhw4xwn4qh";

    #[test]
    fn converts_md1_to_ndef() {
        let bytes = convert(MD1_VALID).unwrap();
        assert_eq!(ndef::decode_text_tlv(&bytes).as_deref(), Some(MD1_VALID));
    }

    #[test]
    fn refuses_ms1() {
        let err = convert("ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f").unwrap_err();
        assert!(matches!(err, ConvertError::RefusedSecret));
    }

    #[test]
    fn rejects_unknown_hrp() {
        assert!(matches!(convert("xx1qqqq"), Err(ConvertError::Classify(_))));
    }
}
```

- [ ] **Step 2: Run the tests to verify they pass**

Run: `cargo test -p mnemonic-engrave --lib`
Expected: all module tests PASS, including `tests::converts_md1_to_ndef`, `tests::refuses_ms1`, `tests::rejects_unknown_hrp`.

- [ ] **Step 3: Commit**

```bash
git add crates/me-cli/src/lib.rs
git commit -s -m "feat(me): convert() pipeline (classify -> refuse ms1 -> validate -> NDEF)"
```

---

## Task 5: CLI surface (`main.rs`)

**Files:**
- Modify: `crates/me-cli/src/main.rs`
- Create: `crates/me-cli/tests/cli.rs`

> **Contract (spec §5):** input via **stdin** or `--in <file>` — never a positional argv. NDEF bytes go to `--out <file>` (default) or stdout via `--stdout`; `--hex`/`--base64` print an encoded form on stdout. Human-readable messages (the canonical validated string, errors, the ms1 refusal) always go to **stderr**. Exit codes: `0` ok, `2` usage error, `3` refused ms1, `4` validation/classify error.

- [ ] **Step 1: Implement the CLI**

Replace `crates/me-cli/src/main.rs` with:
```rust
//! `me` — convert a single md1/mk1 string to an NDEF payload (refuses ms1).

use std::io::{Read, Write};
use std::path::PathBuf;

use clap::Parser;
use mnemonic_engrave::{convert, ConvertError};
use zeroize::Zeroize;

/// Convert a public constellation string (md1/mk1) to an NFC NDEF payload.
/// Reads the string from stdin (or --in). Refuses secret ms1.
#[derive(Parser)]
#[command(name = "me", version, about)]
struct Cli {
    /// Read the input string from this file instead of stdin.
    #[arg(long, value_name = "FILE")]
    r#in: Option<PathBuf>,
    /// Write the NDEF bytes to this file (default: --stdout off => requires --out or an encoding flag).
    #[arg(long, value_name = "FILE")]
    out: Option<PathBuf>,
    /// Write raw NDEF bytes to stdout.
    #[arg(long, conflicts_with_all = ["hex", "base64", "out"])]
    stdout: bool,
    /// Print the NDEF bytes as hex on stdout.
    #[arg(long, conflicts_with_all = ["base64", "out"])]
    hex: bool,
    /// Print the NDEF bytes as base64 on stdout.
    #[arg(long, conflicts_with_all = ["hex", "out"])]
    base64: bool,
}

const EXIT_OK: i32 = 0;
const EXIT_USAGE: i32 = 2;
const EXIT_REFUSED: i32 = 3;
const EXIT_INVALID: i32 = 4;

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    let cli = Cli::parse();

    let mut input = String::new();
    if let Some(path) = &cli.r#in {
        match std::fs::read_to_string(path) {
            Ok(s) => input = s,
            Err(e) => {
                eprintln!("me: cannot read {}: {e}", path.display());
                return EXIT_USAGE;
            }
        }
    } else if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("me: cannot read stdin: {e}");
        return EXIT_USAGE;
    }

    let result = convert(&input);
    input.zeroize(); // scrub the input buffer regardless of outcome

    let bytes = match result {
        Ok(b) => b,
        Err(ConvertError::RefusedSecret) => {
            eprintln!("me: {}", ConvertError::RefusedSecret);
            return EXIT_REFUSED;
        }
        Err(e) => {
            eprintln!("me: {e}");
            return EXIT_INVALID;
        }
    };

    // Emit per the selected output mode. Human guidance -> stderr only.
    if let Some(path) = &cli.out {
        if let Err(e) = std::fs::write(path, &bytes) {
            eprintln!("me: cannot write {}: {e}", path.display());
            return EXIT_USAGE;
        }
        eprintln!("me: wrote {} NDEF bytes to {}", bytes.len(), path.display());
    } else if cli.hex {
        let s: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        println!("{s}");
    } else if cli.base64 {
        println!("{}", base64_encode(&bytes));
    } else if cli.stdout {
        if std::io::stdout().write_all(&bytes).is_err() {
            return EXIT_USAGE;
        }
    } else {
        eprintln!("me: choose an output mode: --out <file>, --stdout, --hex, or --base64");
        return EXIT_USAGE;
    }
    EXIT_OK
}

/// Minimal standard base64 (no padding-free shortcuts); avoids a dep for one use.
fn base64_encode(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(T[(n >> 18 & 63) as usize] as char);
        out.push(T[(n >> 12 & 63) as usize] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6 & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[(n & 63) as usize] as char } else { '=' });
    }
    out
}
```

- [ ] **Step 2: Write the integration tests**

Create `crates/me-cli/tests/cli.rs`:
```rust
use assert_cmd::Command;
use std::io::Write;

const MD1_VALID: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

#[test]
fn md1_hex_to_stdout() {
    let mut cmd = Command::cargo_bin("me").unwrap();
    let out = cmd.arg("--hex").write_stdin(MD1_VALID).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // TLV NDEF starts 0x03; record header 0xD1; ends 0xFE.
    assert!(stdout.trim().starts_with("03"));
    assert!(stdout.trim().ends_with("fe"));
}

#[test]
fn ms1_is_refused_with_exit_3() {
    Command::cargo_bin("me")
        .unwrap()
        .arg("--stdout")
        .write_stdin(MS1)
        .assert()
        .code(3)
        .stderr(predicates::str::contains("CODEX32"));
}

#[test]
fn missing_output_mode_is_usage_error() {
    Command::cargo_bin("me")
        .unwrap()
        .write_stdin(MD1_VALID)
        .assert()
        .code(2);
}
```

- [ ] **Step 3: Run the integration tests**

Run: `cargo test -p mnemonic-engrave --test cli`
Expected: 3 tests PASS (`md1_hex_to_stdout`, `ms1_is_refused_with_exit_3`, `missing_output_mode_is_usage_error`).

- [ ] **Step 4: Commit**

```bash
git add crates/me-cli/src/main.rs crates/me-cli/tests/cli.rs
git commit -s -m "feat(me): CLI (stdin-only, stdout/hex/base64/out, exit codes, ms1 refusal)"
```

---

## Task 6: Committed golden NDEF vectors

**Files:**
- Create: `crates/me-cli/tests/vectors/md1-short.ndef`, `crates/me-cli/tests/golden.rs`

- [ ] **Step 1: Generate the golden file from the converter**

Run (from repo root):
```bash
printf '%s' "md1yqpqqxqq8xtwhw4xwn4qh" | cargo run -q -p mnemonic-engrave --bin me -- --out crates/me-cli/tests/vectors/md1-short.ndef
```
Expected stderr: `me: wrote 32 NDEF bytes to crates/me-cli/tests/vectors/md1-short.ndef` (5-byte record prefix + 24-byte text = 29-byte message; +3 TLV wrapper = 32)

- [ ] **Step 2: Write the golden test**

Create `crates/me-cli/tests/golden.rs`:
```rust
use mnemonic_engrave::convert;

#[test]
fn md1_short_matches_golden() {
    let golden = include_bytes!("vectors/md1-short.ndef");
    let got = convert("md1yqpqqxqq8xtwhw4xwn4qh").unwrap();
    assert_eq!(&got[..], &golden[..]);
}
```

- [ ] **Step 3: Run the golden test**

Run: `cargo test -p mnemonic-engrave --test golden`
Expected: PASS.

- [ ] **Step 4: Commit (force-add the binary vector)**

```bash
git add -f crates/me-cli/tests/vectors/md1-short.ndef
git add crates/me-cli/tests/golden.rs
git commit -s -m "test(me): committed golden NDEF vector for md1"
```

---

## Task 7: Cross-language round-trip (Go harness) — spec §9 anchor

**Files:**
- Create: `firmware/ndef-roundtrip/go.mod`, `firmware/ndef-roundtrip/main.go`
- Create: `crates/me-cli/tests/cross_lang.rs`

> The authoritative cross-language assertion belongs with the firmware work; this is the early anchor that proves the Rust NDEF and SeedHammer's Go reader agree. The Go harness imports SeedHammer's own `nfc/ndef` via a `replace` pointing at the local `seedhammer-ref` checkout. The Rust test is `#[ignore]` (run explicitly when Go + the checkout are present).

- [ ] **Step 1: Create the Go module file**

Create `firmware/ndef-roundtrip/go.mod`:
```
module ndefroundtrip

go 1.25

require seedhammer.com v0.0.0

replace seedhammer.com => ../../../seedhammer-ref-v1.4.2
```

- [ ] **Step 2: Create the Go harness**

Create `firmware/ndef-roundtrip/main.go`:
```go
// Reads NDEF bytes on stdin (TLV-wrapped, as `me` emits), parses them with
// SeedHammer's own reader, and prints the recovered text record body to stdout.
package main

import (
	"fmt"
	"io"
	"os"

	"seedhammer.com/nfc/ndef"
)

func main() {
	in, err := io.ReadAll(os.Stdin)
	if err != nil {
		fmt.Fprintln(os.Stderr, "read:", err)
		os.Exit(1)
	}
	mr := ndef.NewMessageReader(byteReader(in))
	rr := ndef.NewRecordReader(mr)
	buf := make([]byte, 4096)
	n, err := rr.Read(buf)
	if err != nil && err != io.EOF {
		fmt.Fprintln(os.Stderr, "ndef:", err)
		os.Exit(1)
	}
	os.Stdout.Write(buf[:n])
}

// byteReader adapts a []byte to the io.Reader the ndef package expects.
func byteReader(b []byte) io.Reader { return &reader{b: b} }

type reader struct {
	b   []byte
	pos int
}

func (r *reader) Read(p []byte) (int, error) {
	if r.pos >= len(r.b) {
		return 0, io.EOF
	}
	n := copy(p, r.b[r.pos:])
	r.pos += n
	return n, nil
}
```

> **Note for the implementer:** `ndef.NewMessageReader` / `NewRecordReader` are the exact entry points used at `seedhammer-ref-v1.4.2/nfc/poller/poller.go:83-88`. Confirm the constructor signatures against `nfc/ndef/ndef.go` and adjust the adapter if the package takes a `*bufio.Reader` rather than a bare `io.Reader`. This is a flagged compile-iteration point.

- [ ] **Step 3: Smoke-test the Go harness by hand**

Run:
```bash
printf '%s' "md1yqpqqxqq8xtwhw4xwn4qh" | cargo run -q -p mnemonic-engrave --bin me -- --stdout \
  | (cd firmware/ndef-roundtrip && go run .)
```
Expected stdout: `md1yqpqqxqq8xtwhw4xwn4qh`
If `go` errors on the import, fix `go.mod`'s `replace` path so it resolves to the `seedhammer-ref` checkout, and reconcile the reader constructor signature.

- [ ] **Step 4: Write the ignored Rust integration test**

Create `crates/me-cli/tests/cross_lang.rs`:
```rust
use mnemonic_engrave::convert;
use std::io::Write;
use std::process::{Command, Stdio};

/// Cross-language anchor (spec §9): Rust-emitted NDEF parsed by SeedHammer's Go
/// reader must round-trip. Ignored by default; run with:
///   cargo test -p mnemonic-engrave --test cross_lang -- --ignored
#[test]
#[ignore]
fn rust_ndef_parses_in_seedhammer_go_reader() {
    let input = "md1yqpqqxqq8xtwhw4xwn4qh";
    let ndef = convert(input).unwrap();

    let mut child = Command::new("go")
        .args(["run", "."])
        .current_dir("firmware/ndef-roundtrip")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("go must be installed and firmware/ndef-roundtrip present");
    child.stdin.take().unwrap().write_all(&ndef).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "go harness failed: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8(out.stdout).unwrap(), input);
}
```

- [ ] **Step 5: Run the ignored test (when Go is available)**

Run: `cargo test -p mnemonic-engrave --test cross_lang -- --ignored`
Expected: PASS (the Go reader recovers the exact input string).

- [ ] **Step 6: Commit**

```bash
git add firmware/ndef-roundtrip crates/me-cli/tests/cross_lang.rs
git commit -s -m "test(me): cross-language NDEF round-trip via SeedHammer Go reader"
```

---

## Task 8: Length guard + workspace lint gate

**Files:**
- Modify: `crates/me-cli/src/lib.rs` (add a length-warning helper), `crates/me-cli/src/main.rs` (call it)

- [ ] **Step 1: Add a failing test for the length guard**

Add to the `tests` module in `crates/me-cli/src/lib.rs`:
```rust
    #[test]
    fn flags_plate_overflow_risk() {
        // A string longer than the ~ one-plate text budget should be flagged.
        assert!(super::exceeds_plate_budget(&"q".repeat(400)));
        assert!(!super::exceeds_plate_budget("md1yqpqqxqq8xtwhw4xwn4qh"));
    }
```

- [ ] **Step 2: Implement the guard**

Add to `crates/me-cli/src/lib.rs` (module level):
```rust
/// Conservative single-plate text budget. SeedHammer's 85x85mm text layout
/// wraps ~35 chars/line over ~20 usable lines; with a QR present, far less.
/// This is an advisory pre-check — the firmware still backstops with ErrTooLarge.
const PLATE_TEXT_BUDGET: usize = 300;

/// Reports whether a string is long enough to risk overflowing one plate.
pub fn exceeds_plate_budget(s: &str) -> bool {
    s.trim().len() > PLATE_TEXT_BUDGET
}
```

- [ ] **Step 3: Wire the warning into the CLI**

In `crates/me-cli/src/main.rs`, immediately after a successful `convert(...)` (before emitting), add:
```rust
    if mnemonic_engrave::exceeds_plate_budget(&input) {
        eprintln!("me: warning: input is long; it may exceed one plate (the device will reject with ErrTooLarge if so)");
    }
```
(Place this after `let bytes = match result { ... };` and before the output section; note `input` is zeroized after `convert`, so capture its length into a `let too_long = exceeds_plate_budget(&input);` *before* the `input.zeroize();` call and branch on `too_long` here instead.)

- [ ] **Step 4: Run the lib tests**

Run: `cargo test -p mnemonic-engrave --lib`
Expected: all PASS including `flags_plate_overflow_risk`.

- [ ] **Step 5: Clippy + fmt gate**

Run: `cargo fmt --all && cargo clippy --all-targets -- -D warnings`
Expected: no warnings, no diff after fmt. Fix any clippy findings.

- [ ] **Step 6: Full test run**

Run: `cargo test -p mnemonic-engrave`
Expected: all unit + integration tests PASS (the `cross_lang` ignored test is skipped).

- [ ] **Step 7: Commit**

```bash
git add crates/me-cli/src/lib.rs crates/me-cli/src/main.rs
git commit -s -m "feat(me): plate-overflow length warning + clippy/fmt gate"
```

---

## Self-Review (run before declaring the plan done)

- **Spec coverage:**
  - §5 converter (stdin-only, validate, NDEF for md1/mk1, refuse ms1, hygiene) → Tasks 3,4,5,8 ✓
  - §6 NDEF wire format (TLV-wrapped Text record, exact bytes) → Task 1 + golden Task 6 ✓
  - §9 tests (unit, golden NDEF, cross-language round-trip, negatives) → Tasks 1-7 ✓
  - §3 security (ms1 refused, stdin-only, zeroize) → Tasks 4,5 ✓
  - §11 version pinning (md-codec 0.36 / mk-codec 0.4) → Task 0 ✓
  - **Out of this plan (by design, separate plans):** firmware PR1/PR2 (§7), hardware verification (§10), bundle layer + plate preview (§13 step 5).
- **Placeholder scan:** none — every code step carries complete code. The one flagged compile-iteration point (Task 7 Step 2, Go reader constructor signature) is an explicit reconcile-against-source item, not a behavioral TBD.
- **Type consistency:** `Format` (classify) used identically in `validate` and `convert`; `convert() -> Result<Vec<u8>, ConvertError>` consumed consistently by `main`; `encode_text_tlv`/`decode_text_tlv` names match across `ndef` and tests.

## Open items to confirm during execution
- The two known-good vectors in Task 3 (`MD1_VALID`, `MK1_VALID`) and Tasks 4/6 must be confirmed against the current sibling test vectors; if stale, refresh from the cited sources (Task 3 Step 2 gates this).
- Task 7 Go reader constructor signatures vs `nfc/ndef/ndef.go` (flagged).
- The golden byte count in Task 6 Step 1 (`32 bytes`) assumes the 24-char md1 vector; it updates if the vector changes.
