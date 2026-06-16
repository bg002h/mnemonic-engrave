# Plan-stage architect review — plan-R0 — `me` converter

- **Stage:** implementation-plan R0 gate (pre-code)
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect`
- **Plan under review:** `design/IMPLEMENTATION_PLAN_me_converter.md` (post writing-plans self-review)
- **Verdict:** **NOT-GREEN** — 2 Critical, 1 Important, 2 Minor. Folded below; plan-R1 required.

---

## VERBATIM REVIEW OUTPUT

Both clap patterns are valid. `conflicts_with_all = ["hex", "base64", "out"]` accepts string literals referencing other arg IDs (clap 4 maps these to the field-derived IDs), and `r#in` with `#[arg(long)]` produces `--in` via case-conversion of the raw identifier stripped of `r#`. These compile.

Note one minor clap behavior: `conflicts_with_all` referencing `"out"` works because `out` is a defined arg, but the conflict is bidirectional only as declared — `--out` and `--stdout` conflict (declared on stdout), `--out` and `--hex` conflict (declared on hex), `--out` and `--base64` conflict (declared on base64). And `--hex`/`--base64`/`--stdout` mutually conflict. This is consistent and complete. Exit codes (0/2/3/4) match spec §5 §8. Binary→stdout, human→stderr is correctly implemented (every `eprintln!` for guidance, `println!`/`write_all` for output).

I now have everything needed. Let me do a final check that `mk1qpzry9x8gf2tv` (the ndef round_trips test string, line 245) length won't trip anything — it's 15 chars, only used in pure NDEF encode/decode (no BCH), so it's fine. And the `MS1` test strings: classify only reads HRP "ms" so they classify correctly regardless of checksum validity; `refuses_ms1` short-circuits before validation. Those are fine.

Let me verify the ms1 string used in lib.rs test (line 553) and cli.rs (line 709) — refusal happens at classify→RefusedSecret before any BCH, so any "ms1..." prefix works. Confirmed fine.

I have sufficient evidence to render the verdict.

---

# Adversarial Review: `me` Converter Implementation Plan — plan-R0 Gate

## CRITICAL

**C1 — `rejects_corrupted_mk1` test will FAIL: `decode_string` error-CORRECTS, it does not reject single-char corruption.**
Plan Task 3 Step 1 flips one trailing char of `MK1_VALID` and asserts `validate(Format::Mk, &bad).is_err()`. But `mk_codec::string_layer::decode_string` runs full BCH error *correction* (Berlekamp–Massey/Forney, t=4 substitutions) via `bch_correct_regular`/`bch_correct_long`, returning `Ok(DecodedString { corrections_applied: 1, .. })` for a single substitution. Evidence: `string_layer/bch.rs:645-690` (`decode_string` → `bch_correct_*`), and the crate's own `decode_recovers_one_error` test at `bch.rs:1286-1301` proves a single flipped data char auto-corrects to `Ok`. So `validate` returns `Ok` and `.is_err()` is `false` → test fails. The plan's stated contract ("reject a corrupted mk1") is wrong for mk1: per-string mk1 validation *repairs* up to 4 errors rather than rejecting. (Note the asymmetry: md1's `unwrap_string` uses pure `bch_verify_regular` with no correction — `codex32.rs:144` — so md1 single-flip *is* rejected. mk1 is not.) Fix: corrupt ≥5 symbols, or assert on `corrections_applied`, or document that mk1 validation auto-corrects.

**C2 — `MD1_VALID = "md1qqpqqxqq0zkd22pw8dmd3"` is a STALE v0.18 vector; it will not validate under the pinned md-codec 0.36.**
Plan uses this string in Task 3, Task 4, Task 6 golden (target "32 bytes"), Task 5 CLI (cli.rs), Task 7 cross-lang. The md-cli smoke test states verbatim: "`md1qqpqqxqq0zkd22pw8dmd3` v0.18 → `md1yqpqqxqq8xtwhw4xwn4qh` v0.30" (`descriptor-mnemonic/crates/md-cli/tests/smoke.rs:14`), and `CHANGELOG.md:408` confirms the v0.18 phrase. The wire break changed the data symbols, so the v0.18 checksum no longer satisfies `MD_REGULAR_CONST` (`md-codec/src/bch.rs:17`) over the new data. `unwrap_string` BCH-verifies (`codex32.rs:144`), so `accepts_valid_md1`, `converts_md1_to_ndef`, the golden test, all three CLI tests, and the cross-lang test all fail at the validate step. The current string is `md1yqpqqxqq8xtwhw4xwn4qh` (24 chars — golden byte count 32 stays correct). Fix: replace the vector everywhere with `md1yqpqqxqq8xtwhw4xwn4qh` (or regenerate from a `test_vectors::MANIFEST` entry).

## IMPORTANT

**I1 — Task 3 source-citation is wrong: `test_vectors.rs` contains no md1 strings.**
Plan says to copy the md1 vector from `md-codec/src/test_vectors.rs ("encode a MANIFEST entry")`. That file's `Vector` struct stores `template`/`keys`/`fingerprints`, not encoded strings (`test_vectors.rs:12-32`); a grep for any `md1…` literal there returns nothing. The "if stale, refresh from the cited source" instruction (Task 3 Step 2) therefore cannot be followed by copy-paste — the implementer must actually run the encoder. Given C2, this matters: the gate's remediation path is under-specified.

## MINOR

**M1 — Strict TDD deviation (Task 1 / Task 2).** Steps say "Write the failing tests" but the same step pastes the full implementation alongside, then "Run the tests to verify they pass." There is no red phase. Cosmetic vs. the plan's own TDD framing.

**M2 — `mk1qpzry9x8gf2tv` (ndef `round_trips`) and `MS1` strings are non-validating fixtures.** Correct by construction here — `ndef.rs` does pure encode/decode (no BCH) and `refuses_ms1` short-circuits at classify before validation — so these are fine; flagging only to confirm they are intentional.

## VERIFIED OK (challenges that did not break)

- **APIs/signatures:** `md_codec::codex32::unwrap_string(&str) -> Result<(Vec<u8>, usize), Error>` (`codex32.rs:113`); `codex32` is `pub` (`lib.rs:22`); `md_codec::Error` re-exported + Display (`lib.rs:47`). `mk_codec::string_layer::decode_string(&str) -> Result<DecodedString, Error>` (`bch.rs:645`), `string_layer` `pub` (`lib.rs:41`), `decode_string`/`DecodedString` re-exported (`string_layer/mod.rs:32-37`); `mk_codec::Error` public + Display (`lib.rs:50`). Versions md-codec 0.36.0, mk-codec 0.4.0 match pins. All compile.
- **Per-string acceptance:** both accept a single chunk — `MK1_VALID` IS literally chunk 2 of a 2-chunk card in the committed corpus (`mk-codec/src/test_vectors/v0.1.json:12`); neither reassembles. Per-string model correct. (Rejection semantics: see C1.)
- **NDEF arithmetic:** `encode_text_tlv("md1q")` expected `03 09 D1 01 05 54 00 m d 1 q FE` is byte-exact. Parses against `ndef.go` (TNF well-known :176, type 'T' :183, status bit7=0 :189, langLen :194). Golden 24-char → 29-byte message + 3 = 32 bytes correct.
- **MK1_VALID validity:** real/current committed vector — passes `decode_string`.
- **Go harness:** `NewMessageReader(io.Reader)` (`ndef.go:29`), `NewRecordReader(io.Reader)` (`ndef.go:87`), `RecordReader.Read([]byte)` (`ndef.go:95`) exist, take `io.Reader` — adapter correct. `replace ../../../seedhammer-ref-v1.4.2` resolves correctly.
- **CLI:** `r#in`→`--in`, `conflicts_with_all` compile in clap 4; exit codes/routing match spec §5/§8.
- **Spec faithfulness:** covers §5/§6/§9/§3/§11; deferrals correct.

## Verdict: NOT-GREEN
- **C1** — mk1 corruption test contradicts `decode_string` error-correction (will fail).
- **C2** — `MD1_VALID` stale v0.18; current `md1yqpqqxqq8xtwhw4xwn4qh`; breaks 6+ tests.
- **I1** — Task 3 md1 vector-source citation unusable.

---

## FOLD DISPOSITION (added by main session; not part of verbatim review)

| Finding | Sev | Disposition |
|---|---|---|
| C1 mk1 auto-corrects (test wrong + verbatim-engrave-of-typo risk) | **Critical** | FOLDED — converter now REQUIRES pristine input: mk1 validation rejects if `decode_string` applied any correction (symmetric with md1's pure-verify). Test corrupts and expects Err on that basis. (Needs DecodedString corrections accessor — verified before fold.) |
| C2 stale md1 vector | **Critical** | FOLDED — replaced `MD1_VALID` everywhere with the current md-codec-0.36 vector (verified against source), golden byte count 32 unchanged. |
| I1 wrong vector source citation | **Important** | FOLDED — Task 3 now cites the correct source for a current md1 string + the encoder route. |
| M1 strict-TDD red phase | Minor | FOLDED — Task 1/2 reworded to a genuine red phase (stub → failing run → implement). |
| M2 non-validating fixtures | Minor | No change needed (correct by construction); noted. |

plan-R1 re-dispatch follows.
