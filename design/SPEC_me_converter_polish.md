# Spec — `mnemonic-engrave` converter polish (v0.1.0 → v0.1.1)

- **Status:** Draft for review (brainstorming output)
- **Date:** 2026-06-16
- **Provenance:** `cycle-prep-recon-me-converter-nits.md` (recon vs origin/master `1012332`); the 5 FOLLOWUP nits from the converter execution review (`design/agent-reports/me-converter-execution-review.md`).
- **SemVer:** PATCH (`me` is unreleased at 0.1.0; internal hygiene + one additive opt-in flag; no surface break, no breaking change).

## Goal
Clear the five low/nit FOLLOWUPs from the converter review — quality, hygiene, and a spec↔impl reconcile — in one small cycle.

## Scope (5 fixes)

1. **`me-validate-ms-unreachable`** — `crates/me-cli/src/validate.rs:53`: replace `panic!("validate() called on ms1 …")` with `unreachable!("ms1 is refused before validation")`. `convert()` (lib.rs) filters `Format::Ms` before calling `validate`, so this arm is provably unreachable; `unreachable!` states that invariant precisely. No behavior change.

2. **`me-decode-text-tlv-comment`** — `crates/me-cli/src/ndef.rs`: add a doc comment on `decode_text_tlv` (`:67`) noting it intentionally handles only the **1-byte** TLV length form and does **not** check the `0xFE` terminator, because it exists solely for the round-trip self-test against `me`'s own bounded output. No behavior change.

3. **`me-go-harness-shortread-loop`** — `firmware/ndef-roundtrip/main.go:22`: replace the single `rr.Read(buf)` with a loop that reads until `io.EOF` (or the record is fully consumed), accumulating into `buf`, for robustness on payloads larger than one read. The cross-language round-trip test (`crates/me-cli/tests/cross_lang.rs`) must still pass.

4. **`me-in-stdin-intermediate-zeroize`** — `crates/me-cli/src/main.rs`: read the input into a `Zeroizing` buffer (e.g. `zeroize::Zeroizing<String>`) on both the stdin path (`:53`) and the `--in` path (`:46`), so the intermediate string from `read_to_string` is scrubbed on drop. The primary `input` buffer is already zeroized (`:62`); add a one-line comment noting this is defense-in-depth (the tool refuses `ms1`, so secrets shouldn't reach here, but `--in` could be pointed at one). No user-visible behavior change.

5. **`me-canonical-string-stderr`** (the reconcile) — add an opt-in **`--echo`** flag (default off). When set, after a successful conversion print `me: validated <md1|mk1>: <string>` to **stderr** (never stdout — stdout stays binary/encoded NDEF only). Default behavior is unchanged (quiet success except the existing `--out` byte-count line). **Amend `design/SPEC_seedhammer_engrave.md` §5** so the "canonical validated string → stderr" line reads "**only when `--echo` is given**", reconciling spec↔impl. `--echo` is additive; `me` is not in the toolkit manual, so no `schema_mirror`/manual-mirror lockstep.

## Non-goals
- No change to validation semantics, the NDEF wire format, exit codes, or any other flag.
- No firmware (PR1/PR2) changes.

## Error handling
Unchanged. `--echo` only adds a success-path stderr line; it does not alter exit codes (0/2/3/4) or the refusal/error paths.

## Testing
- **`--echo`:** integration test — with `--echo`, stderr contains `validated md1:`/`validated mk1:` and the exact input string; without `--echo`, stderr does not contain it (and stdout is unchanged). Add to `crates/me-cli/tests/cli.rs`.
- **zeroize:** behavior-neutral — the full existing suite (`cargo test -p mnemonic-engrave`) stays green; no new assertion needed beyond confirming the input still converts.
- **Go short-read loop:** covered by the existing `cross_lang` round-trip (run with `go` on PATH); confirm it still passes.
- **`unreachable!`/comment:** no behavior change; covered by the existing `convert`/`ndef` tests + `cargo clippy -D warnings` + `gofmt`/`go vet` staying clean.

## Lockstep / release
- No GUI `schema_mirror` (no clap flag-NAME removal/rename; `--echo` is additive).
- No toolkit manual mirror (`me` not yet documented there).
- Bump `me` to `0.1.1` + CHANGELOG entry if/when the crate adopts versioned releases (currently unreleased; bump optional).
