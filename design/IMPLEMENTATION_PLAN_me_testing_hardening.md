# IMPLEMENTATION PLAN — me funds-safety fixes + testing hardening

Status: **GREEN — plan R0 passed at round 1 (0C/0I/3 nit), nits folded inline
2026-07-06** (reviews: `agent-reports/me-testing-hardening-plan-R0-round0.md` =
0C/1I/6L all folded; `…-round1.md` = GREEN, every fold verified against codec source).
Executes `design/SPEC_me_testing_hardening.md`
(GREEN at R0 round 1). Findings F* per `agent-reports/funds-audit-SYNTHESIS.md`.
Process: plan R0 to 0C/0I → ONE implementer, worktree, strict TDD (failing test first at
every step) → mandatory post-implementation adversarial execution review of full diff.

## Constraints binding the implementer

- Single agent, isolated worktree off `master`. Stage paths explicitly (no `git add -A`).
- TDD: within each step, write/adjust the failing test, watch it fail for the RIGHT
  reason, implement, watch it pass, run the whole suite before the next step.
  **Exception for pure drift/coverage guards and additive pins of already-correct
  behavior (Steps 8, 9, 10; likewise the Step 6–7 boundary/golden pins where the
  encoder is already correct):** fail-first is satisfied by TEMPORARILY perturbing the
  guarded constant/behavior (flip `mm`, flip a discriminator bit, drop the ms1
  pre-scan) to watch the guard go red, then reverting — never manufacture a fake red.
  Step 8's stderr-canary is regression insurance; the genuine fail-first for redaction
  is Step 3's `msx1` test.
- Rust-primary rule: A2's two-symptom STOP condition as specified in Step 4 (golden
  byte-identity AND no newly-failing admission test — halt, report, never re-baseline
  on either symptom); A3 is a me-layer admission guard, no codec change.
- Never weaken an existing test to make a step pass. Goldens change ONLY by adding new
  vector files; `md1-short.ndef` bytes must remain byte-identical throughout.

## Step 0 — worktree + baseline

Create worktree; run `cargo test --locked` and (with submodule initialized)
`go test ./...` in `preview/` and a build of `firmware/ndef-roundtrip`. Record baseline:
which tests pass, which silently skip (expect the cross-lang skips when `go` absent).
Confirm `go` IS available in the worktree environment — the whole cycle needs it.

## Step 1 (A4) — hermetic Go oracle

Test first: `cd firmware/ndef-roundtrip && go build ./...` in a tree where
`../../../seedhammer-ref-v1.4.2` does not resolve (temp clone) — currently fails.
Fix: `firmware/ndef-roundtrip/go.mod` replace → `../../third_party/seedhammer`
(mirror `preview/go.mod`); `go mod tidy`. Verify: build succeeds from a clean
`git clone --recurse-submodules` scratch copy; `cargo test --locked cross_lang` passes
with `go` on PATH.

## Step 2 (A5 + F3 hard-fail) — CI test gating

1. Add `ME_REQUIRE_GO` handling to BOTH skip sites — `crates/me-cli/tests/cross_lang.rs`
   (go-missing early return) and `tests/preview_cross_lang.rs` (`go_available()` guard):
   when `ME_REQUIRE_GO=1`, a missing `go` panics with a clear message instead of
   returning. Verify by running that one test with a PATH stripped of go and the var set
   (expect fail), then unset (expect skip note + pass).
2. Add a `test` job to `.github/workflows/release.yml` (same workflow so `assemble` can
   `needs:` it): `actions/checkout@v4` with `submodules: true`; setup Rust + Go in the
   SAME job; env `ME_REQUIRE_GO: "1"`; steps: `cargo test --locked` at workspace root,
   `go test ./...` in `preview/`, `go build ./... && go test ./...` in
   `firmware/ndef-roundtrip/`. Wire `assemble` (and any publish job) `needs: [test, …]`.
   **Trigger reconciliation (R0 L2):** `release.yml` `on:` is currently
   `push: tags: v*` + `pull_request` — add `push: branches: [master]` so direct branch
   pushes are covered too ("every push/PR" per SPEC A5). Validate YAML with `actionlint`
   if available, else careful review; CI proof is on first push.
3. **Branch-protection note (repo setting, NOT YAML — user action):** blocking a red PR
   from *merging* requires a GitHub branch-protection rule marking `test` as a required
   status check. The implementer cannot set this from the repo; record it in the PR
   description and flag it to the user at handoff — without it, F2's "red suite can
   merge" is only half-closed (tag publish IS gated by `needs:` regardless).

## Step 3 (A1 + B8) — redact bundle error paths

Tests first (`tests/cli.rs`): (a) `me bundle` fed `msx1<known-secret-body>` (mangled-HRP
ms1 from the D5 report probe) → non-zero exit AND stderr does NOT contain the body
substring; (b) corrupted mk1 line → stderr does not contain the full string. Both fail
today. B8 unit test: construct each `BundleError` variant with a marker payload string
(`"CANARY_SECRET_BODY"`) and assert `format!("{e}")` never contains it.
Implement: rewrite `BundleError::{Classify, Validate, Md1HeaderRead}` Display arms
(`bundle.rs:54-60`) to bounded metadata only — mirror ConvertError's hardening (HRP +
bounded prefix at most, keep the underlying codec error's own metadata-only text).
Line numbers may drift; locate by variant name.

## Step 4 (A2) — codec bumps, fail-closed admission

Order matters: capture the over-length fixture BEFORE the bump. The 94-symbol md1
string is NOT recorded verbatim in the D1 report (it is elided there) — generate it in
a SCRATCH crate (outside the repo) pinning `md-codec = "=0.36.0"` via
`wrap_payload(vec![0xA5; 51], 405)` (405 = 81 data symbols × 5 bits; deterministic and
reproducible), and embed the resulting string as a literal fixture.
Test first: `assert!(convert(OVERLEN_MD1).is_err())` — fails on 0.36.
Bump `crates/me-cli/Cargo.toml`: `md-codec = "0.40"` (or latest ≥0.40), mk-codec →
latest 0.4.x; `cargo update -p md-codec -p mk-codec`. Run FULL suite.
**STOP condition (normative drift — halt, report, never re-baseline), two symptoms:**
(a) the `md1-short.ndef` and `bundle-md1-mk1.json` goldens are not byte-identical;
(b) any previously-passing mk1/md1 admission test newly FAILS after the bump (a codec
bump cannot change emitted bytes — `convert()` emits verbatim input — so the live
regression symptom is a previously-valid fixture newly rejected; mk-codec 0.4.0→0.4.1
is the live risk). New test now passes. Confirm the error is the fail-closed
`StringSymbolCountOutOfRange` path (assert on message substring, tolerant of wording).

## Step 5 (A3) — refuse non-canonical md1

Tests first (unit in `lib.rs` tests + `tests/cli.rs` for exit codes):
- interior `-` and interior space in an otherwise-valid md1 → error on convert AND on a
  single-line bundle input; interior `\n` → error on convert (bundle line-splits first —
  not a bundle-path case, per R0 round 1 N2);
- positive guard: clean md1 with and without trailing `\n` → byte-identical output,
  exit 0 (pins today's behavior);
- error message names offending char + byte position and (bundle path) does not echo
  the input body (composes with Step 3's canary test).
Implement in **`validate.rs`** — `validate::validate` is the single shared admission
path (called from both `lib.rs` and `bundle.rs`; verified at plan-R0): add a
`Format::Md`-gated interior-separator check there (or in a helper it calls), as a new
`ValidateError` variant naming the offending char + byte position. Do NOT add separate
guards in `convert()`/`parse_line()`. mk1 untouched (mk-codec already rejects these as
`InvalidChar`). Verify `md1-short` golden and all existing tests still green.

## Step 6 (B1) — golden corpus + NDEF boundary

1. Generate vectors (scratch generator using the bumped codecs, values embedded as
   committed fixture files): max-valid md1 at the 93-symbol cap (≈96 chars), mk1 short,
   mk1 chunk (111-char, reuse existing fixtures where present). Byte-pin each `.ndef`
   under `tests/vectors/`; goldens asserted via `include_bytes!` like `golden.rs`.
2. Alphabet-union test: iterate all vector input strings; assert every bech32 charset
   symbol (and each digit legal in the formats) appears in ≥1 vector. If union coverage
   is incomplete, add one more valid vector to close it (constructive, not synthetic).
3. NDEF-layer boundary units (no codec involvement, direct `encode_text_tlv`):
   249-char synthetic text → success AND TLV length byte == 0xFE (byte-pinned prefix);
   250-char → `NdefError::TooLong`.

## Step 7 (B2) — differential decode through the real reader

Extend `cross_lang.rs` into a table: for every B1 convert-level golden AND ndef-layer
synthetic texts of lengths {1, 63, 64, 96, 111, 248, 249}, encode (convert() for
goldens, `encode_text_tlv` for synthetics), run `firmware/ndef-roundtrip` (`go run .`
with the bytes on stdin, as today), assert decoded text == input positionally.
Honors `ME_REQUIRE_GO`. No golden may be asserted ONLY via me's own `decode_text_tlv`.

## Step 8 (B3) — ms1-refusal table

`tests/cli.rs` table over {lowercase ms1, UPPERCASE MS1, mixed-case Ms1, whitespace-
padded, bad-checksum ms1, ms1 at first/middle/last bundle line}: convert and bundle both
refuse (exit 3 / RefusedSecret), stderr never contains the body (canary from Step 3),
and no decode of the ms payload occurs (assert via error type, not timing).

## Step 9 (B5) — device-constant drift guard (Go)

`go vet seedhammer.com/driver/tmc2209` from `preview/` first (confirm host-compilable;
round-1 architect verified statically — the tinygo-tagged file is `uart.go`).
Add to `params_test.go`: `mm == 200/8*tmc2209.Microsteps` and
`strokeWidth == mm*3/10`. Run `go test ./...`.

## Step 10 (B6) — chunk-discriminator drift guards

Two layers (folded per plan-R0 I1 — the `ChunkHeaderChunkedFlagMissing` arm is provably
unreachable AND unobservable through `parse_line`: the pre-check tests bit 0 of the
first symbol before `ChunkHeader::read`, which reads the same bit as its chunked flag,
and that arm maps to `Ok(Md1Single)` identically to the pre-check's early return — a
`parse_line` fixture for it would be a vacuous test):
- **`parse_line` fixtures for the reachable/observable arms:** Md1Single, Md1Chunk, and
  WireVersionMismatch→Md1WireVersion (via a crafted BCH-valid fixture) — vary the
  first-symbol bits per the md-codec chunk layout; plus the funds-relevant case: a
  single chunk of a known multi-chunk md1 → `SetIncompleteMd`, never a lone accepted
  plate.
- **Direct codec-level drift guard** for the remaining arm: call
  `md_codec::chunk::ChunkHeader::read` on crafted first-symbol bitstreams and pin the
  discriminator behavior the bundle probe relies on (chunked flag = bit 0; the
  `ChunkHeaderChunkedFlagMissing` return requires a stream whose version nibble equals
  `WF_REDESIGN_VERSION` (=4) WITH the chunked flag clear — flag-clear alone with a
  wrong version hits a different error first).
Written against the BUMPED codec (Step 4). Plan-R0 verified `ChunkHeader::read` is
byte-identical between 0.36 and 0.40, so the "adapt the probe" contingency is expected
to be a no-op; if it somehow isn't, adapt (convergence with primary, allowed) and record
in the PR description.

## Step 11 — full verification (superpowers: verify before completion)

- `cargo test --locked` with `go` on PATH and `ME_REQUIRE_GO=1`: zero skips, all green.
- `go test ./...` in `preview/` and `firmware/ndef-roundtrip/`.
- Clean-clone hermeticity: fresh `git clone --recurse-submodules` of the worktree branch
  → full suite green.
- End-to-end: `me bundle --preview <tmpdir>` with the real sidecar on a known md1+mk1
  set; visually sanity-check one SVG; confirm exit codes.
- Negative re-probes from the audit now fail closed: `msx1…` stderr clean; 94-symbol
  md1 refused; interior-separator md1 refused.

## Step 12 — post-implementation adversarial execution review (mandatory)

Independent opus (or fable if contested) reviewer over the FULL diff vs master:
implementation-introduced regressions, test theater (tests that can't fail), redaction
completeness, CI wiring correctness. Persist verbatim to
`agent-reports/me-testing-hardening-exec-review-round0.md`; fold to 0C/0I before merge.
Then finishing-a-development-branch flow (user decides merge/PR).

## Deliverable summary

| Step | SPEC item | Touches |
|---|---|---|
| 1 | A4 | firmware/ndef-roundtrip/go.mod |
| 2 | A5 | release.yml, cross_lang.rs, preview_cross_lang.rs |
| 3 | A1+B8 | bundle.rs Display, tests/cli.rs |
| 4 | A2 | Cargo.toml/lock, lib.rs tests |
| 5 | A3 | validate.rs (+ tests; NOT separate guards in lib.rs/bundle.rs) |
| 6 | B1 | tests/vectors/, ndef.rs units |
| 7 | B2 | cross_lang.rs |
| 8 | B3 | tests/cli.rs |
| 9 | B5 | preview/params_test.go |
| 10 | B6 | tests (bundle fixtures) |
