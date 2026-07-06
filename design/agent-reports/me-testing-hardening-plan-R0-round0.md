# R0 architect review — IMPLEMENTATION_PLAN_me_testing_hardening.md (round 0)

Reviewer: opus architect (R0 plan gate). Date: 2026-07-06.
Target: `design/IMPLEMENTATION_PLAN_me_testing_hardening.md` (round 0, pre-R0 draft).
Executes the GREEN spec `design/SPEC_me_testing_hardening.md` (spec R0 GREEN at round 1).
Standard: GREEN = 0 Critical / 0 Important. This review checks the PLAN's faithfulness to
the already-adjudicated GREEN spec and its executability against the CURRENT code — it does
NOT relitigate spec-GREEN decisions.

## Verdict

**NOT GREEN — 0 Critical / 1 Important / 6 Low-Nit.**

The plan is well-sequenced and, on the load-bearing mechanics, factually grounded: I
re-derived the codec-bump compatibility, the shared-admission-path claim, the CI
trigger/`needs` structure, the Go-oracle invocation shape, and the NDEF boundary arithmetic
against source, and they hold (details under "Verified against source"). The single Important
is a test-integrity defect in Step 10 (B6): it instructs the implementer to write a
`parse_line`-level fixture that hits the `ChunkHeaderChunkedFlagMissing` arm, but that arm is
provably UNREACHABLE and UNOBSERVABLE through `parse_line` given the pre-check in
`bundle.rs`, so a literal implementation yields a vacuous test (passes without testing the
mechanism) — exactly the test-theater the gate must block. It folds with a one-paragraph
reword (pin that arm via a direct `ChunkHeader::read` call, not `parse_line`). Everything else
is Low/Nit.

---

## Verified against source (no finding — recorded for the gate)

- **Codec bump (Step 4/A2) is API-compatible and behavior-stable 0.36→0.40.** I diffed the
  registry sources (`md-codec-0.36.0` vs `md-codec-0.40.0`): `Descriptor` struct **identical**;
  `codex32::unwrap_string(&str)->Result<(Vec<u8>,usize)>`, `codex32::wrap_payload(&[u8],usize)`,
  `bitstream::BitReader::with_bit_limit(&[u8],usize)`, `chunk::split`, `chunk::reassemble`
  signatures **identical**; `chunk::ChunkHeader::read` body **byte-identical** (0.40 only adds a
  `version` field to the returned struct and one extra unit test — `bundle.rs` reads
  `h.chunk_set_id/count/index` only, never constructs the struct, so the added field is
  harmless); `md_codec::Error` is **not** `#[non_exhaustive]` and both
  `ChunkHeaderChunkedFlagMissing` / `WireVersionMismatch{got}` are present, so `bundle.rs`'s
  `match … Err(e)=>Md1HeaderRead` still compiles when 0.40's new `StringSymbolCountOutOfRange`
  variant appears; `Body::MultiKeys` / `UseSitePath::standard_multipath` / `TlvSection::new_empty`
  / `PathDeclPaths::Divergent` all still exist, so the existing `chunked_md1_vector()` test helper
  (`bundle.rs:547-585`) still compiles. Net: the bump changes **only** accept/reject via the new
  over-length guard (the A2 fix itself); it breaks no existing compile or behavior. This
  **de-risks Step 4** ("run full suite" won't surface unrelated breakage) and makes Step 10's
  "if the bumped codec changed the probe's contract, adapt" contingency a **no-op** — the
  discriminator contract is stable.

- **Step 4 fixture ordering is not just prudent, it is REQUIRED.** 0.40's `wrap_payload` gained a
  data-symbol cap (`if data_symbols.len() > REGULAR_DATA_SYMBOLS_MAX { PayloadTooLongForSingleString }`,
  `md-codec-0.40.0/src/codex32.rs:89-92`; `REGULAR_DATA_SYMBOLS_MAX = 80`). So an 81-data-symbol
  (94-code-symbol) md1 CANNOT be built with the in-repo codec after the bump. The plan's
  "capture BEFORE the bump … regenerate in a SCRATCH crate pinning md-codec =0.36.0 via
  `wrap_payload(vec![0xA5; …], 81*5)`" and "fixture string, not a build-time dep" are therefore
  correct and necessary. TDD is sound: `assert!(convert(OVERLEN_MD1).is_err())` returns `Ok`
  today (0.36) and errors after the bump — fails-first for the right reason.

- **Step 5 shared-admission-path claim is TRUE.** `validate::validate(fmt, s)` (`validate.rs:41`)
  has exactly two callers — `convert()` (`lib.rs:62`) and `parse_line()` (`bundle.rs:101`) — and
  both pass the trimmed, separators-intact `s`. A `Format::Md`-gated interior-`{whitespace,-}`
  check inside `validate::validate` (or a helper it calls) is a genuine single shared point that
  guards both paths, wraps to `ConvertError::Validate` / `BundleError::Validate` (the latter
  redacted by Step 3), and leaves mk1 (`Format::Mk`) untouched. Chunked-md1 strings from `split`
  and clean md1 contain no interior separators, so no false positives. (But see L1 on the file
  attribution.)

- **CI structure supports Step 2.** `.github/workflows/release.yml` `on:` = `push: tags: v*` +
  `pull_request`. A new `test` job (no `if:` guard) therefore runs on PRs and on tag pushes; wiring
  `assemble.needs: [test, go-build, rust-build]` means a red `test` blocks `assemble` on a tag
  push (assemble keeps its `if: startsWith(github.ref,'refs/tags/v')`). The funds-safety property
  (red suite cannot tag-publish) holds. Root `Cargo.toml` is a workspace (`members=["crates/me-cli"]`),
  so "cargo test --locked at workspace root" covers me-cli. `assemble` is the only job with a
  `needs:` to update. (But see L2 on "every push".)

- **Step 7 fits the Go oracle invocation exactly.** `cross_lang.rs:24-32` runs `go run .` with
  `current_dir(harness)` and pipes the NDEF bytes to **stdin**; the plan's "run
  `firmware/ndef-roundtrip` (`go run .` with the bytes on stdin, as today)" matches. All Step-7
  synthetic lengths {1,63,64,96,111,248,249} are `< 250`, so `encode_text_tlv` succeeds on each
  (no `TooLong`), and the fixed `status=0x00` / lang-len-0 Text record round-trips ASCII text
  positionally. The two F3 skip sites are exactly `cross_lang.rs:11-14` and
  `preview_cross_lang.rs:82-85` (`go_available()`), both targeted by Step 2.

- **Step 6 NDEF boundary arithmetic is correct.** `encode_text_tlv(249 chars)` →
  `text_record` payload_len 250 (`<256`, OK) → message len 254 → `tlv_wrap` 254 `< 0xFF` → TLV
  length byte `0xFE`. `encode_text_tlv(250 chars)` → message len 255 → `255 >= 0xFF` →
  `NdefError::TooLong`. Plan's "249 → 0xFE; 250 → TooLong" is exact. A 93-symbol (80-data-symbol,
  400-bit) max-valid md1 is constructible with 0.40's `wrap_payload` (80 is at, not over, the cap).

- **Step 9 (B5) is feasible and correctly stated.** `params.go`: `mm=6400`, `strokeWidth=1920`;
  `tmc2209.Microsteps = 1<<8 = 256`; `200/8*256 = 6400 == mm`, `6400*3/10 = 1920 == strokeWidth`.
  The plan states the correct tinygo-tagged file (`uart.go`, fixing spec round-1 N1) and gates on a
  `go vet` host-compile check first.

- **Faithfulness / ordering / safety rails.** Every kept spec item maps 1:1 (A1→S3, A2→S4, A3→S5,
  A4→S1, A5→S2, B1→S6, B2→S7, B3→S8, B5→S9, B6→S10, B8→S3); descoped A6/A7/B4/B7 correctly absent;
  no scope creep. Ordering honors the spec (A4/A5 first; fixture capture before bump; A3 after the
  bump; redaction Step 3 before A3/B3 that depend on it; B6 against the bumped codec). Rust-primary
  STOP condition present (A2 byte-change → halt); golden-immutability rule present (`md1-short.ndef`
  byte-identical; goldens grow only by new files); redaction carried into Step 5 ("does not echo the
  input body") and Step 8 ("stderr never contains the body"). A3 is correctly a me-layer refuse (not
  a canonicalize that would need md-codec-first) — no Rust-primary violation.

---

## IMPORTANT

### I1. Step 10 (B6) tells the implementer to hit the `ChunkHeaderChunkedFlagMissing` arm via `parse_line`, but that arm is unreachable AND unobservable there → a vacuous test
**Where:** Plan Step 10 — "Fixtures hitting all four `parse_line` md1 arms (Md1Single, Md1Chunk,
ChunkHeaderChunkedFlagMissing, WireVersionMismatch→Md1WireVersion)".
**Problem (traced against `bundle.rs:144-167`):** `parse_line`'s md1 branch pre-checks the
chunked flag before ever calling `ChunkHeader::read`:
```
let chunked_flag = probe.read_bits(5).map(|sym| sym & 0x01 != 0).unwrap_or(false);
if !chunked_flag { return Ok(Md1Single); }          // pre-check consumes the non-chunked case
match ChunkHeader::read(&mut r) {
    Ok(h) => Md1Chunk,
    Err(ChunkHeaderChunkedFlagMissing) => Md1Single, // <-- this arm
    Err(WireVersionMismatch{..}) => Md1WireVersion,
    Err(e) => Md1HeaderRead,
}
```
`probe`'s `sym & 0x01` is bit 0 of the first 5-bit symbol; `ChunkHeader::read` reads
`read_bits(4)` (version) then `read_bits(1)` — **the same bit 0** — as its `chunked` flag
(md-codec `chunk.rs`, first-symbol layout `[v3][v2][v1][v0][chunked]`, verified identical in
0.36 and 0.40). `read()` returns `ChunkHeaderChunkedFlagMissing` only when that bit is 0 — but
the pre-check has already returned `Md1Single` in that case, so `read()` is reached ONLY when the
bit is 1, where `read()` never takes the `!chunked` branch. Hence the `ChunkHeaderChunkedFlagMissing`
arm is **dead relative to the pre-check** — no md1 (pristine or crafted) routes through it via
`parse_line`. Worse, that arm maps to `Ok(Md1Single)`, byte-identical to the pre-check's early
return, so even if it were reachable a `parse_line`-output assertion could not **distinguish** it.
An implementer following the step literally will either get stuck or (the real hazard) write a test
that feeds some md1, gets `Md1Single`, and labels it the `ChunkHeaderChunkedFlagMissing` case — a
test that passes without exercising the arm and cannot fail for the right reason. That is the exact
test-theater class the R0 gate exists to catch.
**Not a funds hole:** the funds-relevant protection in Step 10 (single chunk of a multi-chunk md1 →
`SetIncompleteMd`, never a lone accepted plate) is real, reachable, and already partially covered
(`md1_chunked_set_verifies_and_drop_fails`); and the dead arm loses no coverage because it is dead
in production too. This is a test-validity/executability defect, not a missing safeguard.
**Fix (concrete, cheap):** reword Step 10 to pin the version/chunked-flag discriminator at the
**codec** boundary — call `md_codec::chunk::ChunkHeader::read` directly on crafted first-symbol
bitstreams (mirroring md-codec's own `chunk.rs` unit test that feeds `[0000][chunked=1]` and asserts
`WireVersionMismatch{got:0}`, and a `[WF_REDESIGN_VERSION][chunked=0]` word asserting
`ChunkHeaderChunkedFlagMissing`). Reserve `parse_line`-level fixtures for the arms that ARE reachable
and observable there: `Md1Single` (chunked flag 0), `Md1Chunk` (valid chunked), and
`WireVersionMismatch→Md1WireVersion` via a BCH-valid crafted string whose first-symbol version nibble
≠ `WF_REDESIGN_VERSION` with bit 0 set (constructible with a 0.36-`wrap_payload` scratch fixture, the
D1-2 technique) — plus the funds case (`SetIncompleteMd`). State explicitly that
`ChunkHeaderChunkedFlagMissing` is unreachable via `parse_line` and is therefore drift-guarded at the
`ChunkHeader::read` level only.

---

## LOW / NIT

### L1. Step 5 mislocates the shared guard: Deliverable table omits `validate.rs`
Step 5 prose says "the shared admission path used by both `convert()` (`lib.rs`) and `parse_line()`
(`bundle.rs`)", and the Deliverable table row 5 lists "Touches: lib.rs, bundle.rs, tests." The
actually-shared function is `validate::validate` in **validate.rs** (both `lib.rs:62` and
`bundle.rs:101` call it). If an implementer reads the table literally and adds two separate guards in
`convert()` and `parse_line()`, it is (a) not shared and (b) risks guarding one path only. The
specified single-line-bundle acceptance test would catch a convert-only mistake, so this is
self-correcting — but it is a real doc defect. **Fix:** state that the new `Format::Md`-gated interior-
separator check goes in `validate::validate` (or a helper it calls), as a new `ValidateError` variant
naming the offending char + byte position; update the table row 5 to `validate.rs (+ tests)`.

### L2. A5's "on every push/PR" is only partly met: the workflow is tag+PR-triggered, and PR-merge blocking needs branch protection the plan doesn't mention
`release.yml` `on:` is `push: tags:v*` + `pull_request` — there is no `push: branches:`. So the new
`test` job runs on PRs and on tag pushes, but NOT on a direct (non-PR, non-tag) branch push. The
funds-safety intent is met for a PR-based flow (tag publish is gated via `assemble.needs`; PR status is
available), but two gaps deserve a one-line note: (i) blocking a **red PR from merging** requires a
GitHub **branch-protection** rule marking `test` required — that is a repo setting, not in YAML, and
the plan/PR should call it out (otherwise "a red suite can merge" from F2 is only half-closed); (ii) if
direct branch pushes are possible, add `push: branches: [master]` to honor "every push," or document
that all changes flow through PRs. **Fix:** add the branch-protection note to Step 2 and reconcile the
trigger set with the spec's "every push/PR" wording.

### L3. B3/B5/B6 are drift/coverage guards over already-correct behavior — the blanket "failing test first at EVERY step" cannot be honored literally
The Constraints section mandates "write/adjust the failing test, watch it fail for the RIGHT reason"
at every step. But Step 8 (B3 ms1-refusal table) pins behavior that is **already correct** (classify
lowercases the HRP and is checksum-agnostic, so all case/padded/bad-checksum variants already refuse;
and the ms1-refusal error never interpolates input, so its canary assertion is already true today);
Step 9 (B5) and Step 10 (B6) likewise pin already-true constants/behavior. These tests are green
against unmodified code **by design** — there is no genuine red phase. A rigid implementer could waste
effort forcing a red or, worse, manufacture a fake one. **Fix:** note that for pure drift/coverage
guards the "fail-first" is satisfied by **temporarily perturbing the guarded constant/behavior** (flip
`mm`, flip a discriminator bit, drop the ms1 pre-scan) to confirm the guard goes red, then reverting —
and that B3's stderr-canary is regression insurance, not a fail-first (the fail-first for redaction is
Step 3's `msx1` test).

### L4. Step 4's STOP condition is framed as a golden BYTE diff only; a mk1/md1 admission regression is a test failure, not a byte diff
The constraint reads "any mk1/md1 fixture byte change after codec bumps = normative drift → halt."
Because `convert()` emits the verbatim input independent of codec version, a codec bump can never change
a fixture's emitted bytes — the observable symptom of a normative regression would be a **previously-valid
fixture newly REJECTED** (a red bundle/validate test), which the byte-diff framing does not name. **Fix:**
extend the STOP condition to "any mk1/md1 fixture byte change OR any previously-passing mk1/md1 admission
test that newly fails after the bump = normative drift → halt, report, do not re-baseline." (The
mk-codec 0.4.0→0.4.1 bump is the live risk here.)

### L5. The D1-2 over-length fixture is NOT recorded verbatim in the D1 report — the scratch-crate fallback is the actual path (confirm, don't rely on the primary)
Step 4 says the 94-symbol string "is recorded in `funds-audit-D1-admission-round0.md` (D1-2 probe); if
not verbatim there, regenerate in a SCRATCH crate." It is **not** verbatim there — the report elides it
(`md15kj6tfd9...5zfqq6yyhmu3j8`, D1 report line ~80). So the scratch-crate path (pin `md-codec =0.36.0`,
`wrap_payload(vec![0xA5; 51], 405)`) is the operative one, and it is well-specified and reproducible. No
change required beyond deleting the "recorded … verbatim" implication so the implementer goes straight to
the deterministic scratch generator. (Recorded so the implementer doesn't hunt for a string that isn't
there.)

### L6. Minor wording: "every mk1 fixture must be byte-identical"
There are no per-mk1 `.ndef` goldens; the mk1 fixtures are inline string constants exercised through the
`bundle-md1-mk1.json` manifest golden and `md1-short.ndef`. **Fix:** phrase Step 4's golden check as "the
`md1-short.ndef` and `bundle-md1-mk1.json` goldens must be byte-identical, and no valid mk1/md1 fixture may
newly fail admission" (subsumes L4).

---

## Summary of required changes to reach GREEN
- **I1:** reword Step 10 to drift-guard the `ChunkHeaderChunkedFlagMissing`/version discriminator via a
  direct `md_codec::chunk::ChunkHeader::read` call on crafted first-symbol bitstreams (it is unreachable
  and unobservable through `parse_line`); keep `parse_line` fixtures for the reachable/observable arms
  (Md1Single, Md1Chunk, WireVersionMismatch→Md1WireVersion via a crafted BCH-valid fixture) plus the
  funds case (SetIncompleteMd).
- Fold L1–L6 (cheap: name `validate.rs` as Step 5's site + fix the table; add the branch-protection /
  trigger note to Step 2; carve out the perturb-then-revert fail-first for B3/B5/B6; broaden Step 4's
  STOP to admission-test regressions; drop the "recorded verbatim" implication for the D1-2 fixture;
  tighten the mk1 golden wording).

Re-dispatch after folding (folds can drift).

**VERDICT: NOT GREEN (0C / 1I).**
