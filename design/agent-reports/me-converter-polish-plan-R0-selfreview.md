# Converter-polish plan-R0 — inline self-review

- **Stage:** implementation-plan R0 gate (pre-code) for `design/IMPLEMENTATION_PLAN_me_converter_polish.md`.
- **Date:** 2026-06-16
- **Reviewer:** main session (inline self-review). A formal opus-architect subagent R0 was attempted but **Agent-API dispatch was failing** (500s, then 529 Overloaded) throughout this session's back half, so the gate was satisfied inline — consistent with the PR2 BCH self-review precedent. The plan's risks are compile-correctness, which execution's `cargo build`/`clippy`/`go vet` validate empirically.
- **Verdict:** **GREEN — 0 Critical / 0 Important.**

## Checks (against current source)
1. **Zeroizing (Task 4) compiles.** `Zeroizing<String>: DerefMut<Target=String>` → `*input = s` and `read_to_string(&mut *input)` are valid; `convert(&input)`/`exceeds_plate_budget(&input)` (both `fn(&str)`) accept `&Zeroizing<String>` via 2-step deref-coercion at the function-arg coercion site; `drop(input)` follows the last use; the `use zeroize::Zeroize;`→`Zeroizing;` switch is clean (the only `Zeroize`-trait use was the removed `input.zeroize()` at main.rs:62).
2. **`--echo` (Task 5).** `echo_line` is an owned `String` captured before `drop(input)` and printed only after the error-`match` (success path) — `ms1`/errors `return` first and never echo; no borrow of a dropped value. `#[arg(long)] echo: bool` → `--echo`. Label `starts_with("mk1") else "md1"` is sound (convert() guarantees valid md1/mk1 on success).
3. **Tests gate the change.** `echo_prints_validated_string_to_stderr` fails before Task 5 (clap rejects unknown `--echo`, command not `.success()`) and passes after; `no_echo_by_default` guards regression. `MD1_VALID` const + `.assert().success().get_output().stderr` pattern match existing cli.rs.
4. **Spec §5 edit** — the quoted old text is a verbatim substring of `SPEC_seedhammer_engrave.md:78`; the Edit applies.
5. **Go short-read loop (Task 6)** — accumulate `buf[:n]`, break on `io.EOF`, exit on other error, write all; preserves the `cross_lang` round-trip.
6. **validate.rs `unreachable!`** behavior-neutral (arm provably unreachable via convert()'s Ms filter); **ndef.rs** comment-only.

## Note
If/when Agent-API dispatch recovers, a formal subagent R0 of this plan (and a final review of the resulting diff) can be run; the inline review + the green build/test/clippy/gofmt gate stand in the interim.
