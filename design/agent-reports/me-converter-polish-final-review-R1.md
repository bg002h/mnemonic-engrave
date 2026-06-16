# Converter-polish (commit `5086119` + fold) — formal architect review — R1 (convergence)

- **Stage:** R1 convergence round of the deferred converter-polish formal review (`firmware-deferred-formal-reviews` item (b)). Re-dispatch after the R0 fold.
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-reviewer` (adversarial)
- **Scope:** the R0 fold on `crates/me-cli/src/main.rs` (`echo_line` built only when `cli.echo && result.is_ok()`, now `Option<Zeroizing<String>>`) + `crates/me-cli/tests/cli.rs` (stdout-purity assertions). Reviewed against `lib.rs` `convert()` and the full suite.
- **Verdict:** **GREEN — 0 Critical / 0 Important.** Final-review loop converged.

---

## VERBATIM REVIEW OUTPUT

I now have a complete picture of the code. Let me perform the rigorous adversarial trace before writing up the report.

**Tracing I-1 resolution — ms1 path through `run()`:**

1. `input` is a `Zeroizing<String>`.
2. `too_long = mnemonic_engrave::exceeds_plate_budget(&input)` — borrows `input` immutably, does not consume it.
3. `result = convert(&input)` — for ms1 input, `classify()` returns `Ok(Format::Ms)`, then `convert()` returns `Err(ConvertError::RefusedSecret)`. `result` is `Result<Vec<u8>, ConvertError>`.
4. `echo_line` construction: `if cli.echo && result.is_ok()` — `result.is_ok()` is a `bool` method that borrows `result` by shared reference and returns `bool`. It does NOT consume `result`. For ms1, `result` is `Err(...)`, so `result.is_ok()` is `false`. The `&&` short-circuits and `echo_line = None`. No allocation of any `String` containing `input`'s content occurs.
5. `drop(input)` — Zeroizing scrubs the heap allocation.
6. `match result { Err(ConvertError::RefusedSecret) => ... return EXIT_REFUSED; }` — now `result` is consumed by the match. This is fine; `is_ok()` above only borrowed it.
7. No copy of ms1 content ever escapes to an un-zeroized allocation. The `echo_line` branch is provably dead for ms1.

**Tracing borrow/lifetime correctness:**

- `input` is alive from line 51 through `drop(input)` at line 84.
- `too_long` borrows `&input` at line 66, the borrow ends before line 68.
- `convert(&input)` borrows `&input` at line 68, borrow ends after line 68.
- `echo_line` construction at line 76 borrows `input.trim()` (a `&str` slice into `*input`) — but the `format!()` call copies that `&str` into a new owned `String`. So by the time `echo_line` is bound, there is no outstanding borrow of `input`. The `Zeroizing<String>` inside `echo_line` owns its own heap buffer, independent of `input`.
- `drop(input)` at line 84 is explicit and sound: `input` is not borrowed after line 79.
- `result` is used by `is_ok()` at line 76 (shared borrow, ends), then consumed by `match` at line 86. This is valid Rust — `is_ok()` takes `&self`, not `self`.

**Tracing `Zeroizing<String>` scrub guarantee:**

- `Zeroizing<T>` implements `Drop` by calling `T::zeroize()`. `String` implements `Zeroize` (from the `zeroize` crate) by overwriting its bytes with zeros then setting length to 0. This is the same pattern already used for `input` (`Zeroizing::new(String::new())`). For `echo_line`, the `Some(Zeroizing::new(format!(...)))` wrapper means the format string buffer is also scrubbed on drop — this is the belt-and-suspenders property the comment explains. The scrub is genuine.

**Checking the label logic in echo_line:**

Line 78: `let label = if s.starts_with("mk1") { "mk1" } else { "md1" };`

This runs only inside `result.is_ok()`, meaning `convert()` returned `Ok`. `convert()` returns `Ok` only after `classify()` succeeds with `Format::Md` or `Format::Mk` AND `validate()` passes. So `s` is guaranteed to be either an md1 or mk1 string at this point. The `starts_with("mk1")` check is correct and safe — it can never be ms1 here.

**Checking N-1 resolution:**

The test `echo_prints_validated_string_to_stderr` (lines 37-57 of cli.rs) now:
1. Asserts stderr contains `"validated md1:"` — confirms echo format.
2. Asserts stderr contains `MD1_VALID` — confirms the actual string appears.
3. Asserts stdout does NOT contain `"validated"` — pins the no-bleed invariant.
4. Asserts stdout does NOT contain `MD1_VALID` — pins the no-input-leak invariant.

The false-positive risk for the stdout assertions: `MD1_VALID = "md1yqpqqxqq8xtwhw4xwn4qh"`. The hex output is NDEF bytes (binary encoded as lowercase hex). The NDEF bytes contain the literal UTF-8 text `md1yqpqqxqq8xtwhw4xwn4qh` in the payload, but encoded as hex digits (e.g., `6d6431...`), not as the literal ASCII string. So stdout hex will never contain the substring `"md1yqpqqxqq8xtwhw4xwn4qh"` literally. The assertion is sound against false positives.

**Regression sweep on the other 4 nits:**

- `unreachable!` in `validate.rs` line 53: `Format::Ms => unreachable!("ms1 is refused before validation")` — present and intact.
- `decode_text_tlv` comment in `ndef.rs` lines 68-70: "does NOT check the `0xFE` terminator" — present and intact.
- Go short-read loop in `firmware/ndef-roundtrip/main.go` lines 22-33: the read loop with `append(out, buf[:n]...)` before checking `err == io.EOF` is intact. The loop correctly handles partial reads.
- stdin/`--in` Zeroizing in `main.rs` lines 51-63: `Zeroizing::new(String::new())` for stdin path; for `--in` path, `read_to_string(path)` returns a plain `String` which is then moved into `*input` (dereferencing the `Zeroizing` wrapper), keeping it under zeroize protection. Both paths intact.

**No new issues introduced.** The fold is a minimal, targeted change: reordering `echo_line` construction and changing its type from `Option<String>` to `Option<Zeroizing<String>>`. No logic paths, exit codes, stdout output modes, or ms1 refusal behavior are altered.

---

**I-1 status:** RESOLVED.

Trace: For ms1 input, `convert(&input)` returns `Err(ConvertError::RefusedSecret)`. At line 76, `result.is_ok()` evaluates to `false` (via shared borrow — does not consume `result`). The `&&` short-circuits; the `echo_line` construction block is never entered; no `String` containing `input`'s content is allocated. `input` is then dropped and zeroized at line 84. `result` is later consumed at the `match` on line 86, returning `EXIT_REFUSED`. The ms1 secret is provably unreachable by `echo_line` allocation on any code path.

**N-1 status:** RESOLVED.

The test `echo_prints_validated_string_to_stderr` now asserts both that stderr contains the expected echo content AND that stdout contains neither `"validated"` nor the literal input string `MD1_VALID`. The stdout assertions are sound: the hex-encoded NDEF payload encodes the input as UTF-8 bytes rendered in hex (e.g. `6d6431...`), which cannot coincidentally contain the ASCII substring `"md1yqpqqxqq8xtwhw4xwn4qh"`. No false sense of security.

---

## Issues

### Critical
None.

### Important
None.

### Minor
None.

### Nit
None.

---

## Assessment

Adversarial vectors traced on the fold:

1. **ms1 reachability via `is_ok()` consuming `result`**: `Result::is_ok()` takes `&self` — confirmed it is a shared borrow, not a consuming move. `result` remains available for the downstream `match`. No double-move, no compiler error, no logic gap.

2. **Lifetime gap between `echo_line` construction and `drop(input)`**: `input.trim()` at line 77 yields a `&str` borrow into `*input`. The `format!()` macro immediately copies that slice into a new heap-owned `String`, ending the borrow. `drop(input)` at line 84 is therefore sound — no borrow of `input` is outstanding, and `echo_line` does not alias `input`'s buffer.

3. **`Zeroizing` scrub genuineness**: `Zeroizing<String>` uses `String::zeroize()` which overwrites the heap allocation byte-by-byte then truncates. The same mechanism already guards `input` throughout the file. The `echo_line` wrapper adds belt-and-suspenders protection, correctly noted in the comment at lines 73-75.

4. **Label branch correctness on success path**: The `starts_with("mk1") else "md1"` label at line 78 is evaluated only inside `result.is_ok()`, i.e., only after `convert()` has confirmed the string is a valid md1 or mk1. Ms1 cannot reach this branch. The label logic is correct.

5. **N-1 false-positive risk for stdout assertion**: Hex-encoded NDEF bytes for `MD1_VALID` would contain the hex of those ASCII characters (e.g., `6d6431797170717178717138787477683478776e347168`) never the literal ASCII string. The `!stdout.contains(MD1_VALID)` assertion is a genuine invariant, not a vacuous one.

6. **Regression: other four nits**: All four confirmed intact — `unreachable!` guard in `validate.rs`, `decode_text_tlv` partial-parser comment in `ndef.rs`, Go short-read loop in `firmware/ndef-roundtrip/main.go`, and `Zeroizing` stdin/`--in` paths in `main.rs`. No behavior change to exit codes (`EXIT_OK=0`, `EXIT_USAGE=2`, `EXIT_REFUSED=3`, `EXIT_INVALID=4`), stdout output modes, or ms1 refusal logic.

---

## Verdict: GREEN (0 Critical / 0 Important)

The fold cleanly resolves both R0 findings. I-1 is resolved by moving `echo_line` construction behind the `result.is_ok()` gate, making ms1 allocation provably unreachable. N-1 is resolved by adding stdout-purity assertions to the echo test. No new issues were introduced. The Rust borrow/lifetime semantics are correct throughout; the Zeroizing scrub is genuine; the test assertions are sound. The code is ready to commit.

---

## Loop summary (added by main session)
| Round | Verdict | Folded |
|---|---|---|
| R0 | NOT-GREEN | 1 Important (I-1: `--echo` builds an un-zeroized heap copy of the input pre-`convert()`, leaks on the ms1-refusal path) + 1 Nit (N-1: echo test lacked a stdout-purity assertion) → both folded |
| **R1** | **GREEN** | none — converged |

This closes the deferred `firmware-deferred-formal-reviews` item (b). Full suite green post-fold (lib 14 / cli 5 / golden 1 / cross-lang 1), clippy `-D warnings` + fmt clean.
