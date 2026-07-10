# SPEC — me fuzz + property targets (Cycle C: F18)

Status: **GREEN — R0 passed at round 1 (0C/0I, 1L/4N folded inline)** (reviews:
`me-fuzz-proptest-spec-R0-round0.md` = 0C/2I/4L/4N all folded [I1 byte-length round-trip
domain, I2 no-new-pub-API via #[path] include]; `…-round1.md` = GREEN, both invariant
families re-verified TRUE of current code, NDEF 249/250 boundary confirmed exact). The L1
[shared file in a tests/ subdir, not a bare tests/*.rs] + 4 nits folded inline. Cleared
for single-implementer TDD. Closes `me-fuzz-proptest-targets` (F18) from
`design/FOLLOWUPS.md`. Evidence: `design/agent-reports/funds-audit-D6-tests-round0.md`
(D6-7) + its refuting verdict — **no reachable panic or fund-losing misroute was found**;
this is *insurance* (a standing guard that arbitrary input keeps the safety invariants),
NOT a fix for a live bug. Executed locally (cloud CCR env failed to start). Process: R0
gate to 0C/0I → single implementer, TDD → post-impl adversarial review.

Recon (verified against current master `9fafb6b`, 2026-07-09):
- `mnemonic_engrave::convert(input: &str) -> Result<Vec<u8>, ConvertError>` (lib.rs:56);
  ms-HRP → `ConvertError::RefusedSecret` (lib.rs:60).
- `mnemonic_engrave::bundle::run_bundle(input: &str) -> Result<Manifest, BundleError>`
  (bundle.rs:183); any ms line → `BundleError::RefusedSecret` (parse_line, bundle.rs:104).
- `Manifest.plates: Vec<PlateEntry>`; `PlateEntry.string: Option<String>` (manifest.rs:47)
  — when set, it is the `trim()`ed input line (parse_line trims at bundle.rs:101).
- No `fuzz/` dir, no proptest/quickcheck dev-dep today (confirmed).
- `ndef::{encode_text_tlv, decode_text_tlv}` exist (ndef.rs) — the round-trip pair.

## Design: two layers, one shared invariant file (R0 I2 — no new public API)

The load-bearing, always-run coverage is **proptest in the normal test suite** (stable
toolchain — runs inside the existing `test (rust + go)` CI job with NO new CI wiring).
**cargo-fuzz** targets are added for local/deep coverage (nightly) and reuse the SAME
invariant assertions, so the two layers can never drift.

**Shared checkers are NOT new `pub` API.** `mnemonic-engrave` is published to crates.io;
adding `pub` invariant helpers would widen its public surface permanently. Instead, put
the checkers in a plain source file — **`crates/me-cli/tests/support/invariants.rs`**
(a SUBDIRECTORY, NOT a bare `tests/*.rs`: cargo auto-compiles every `tests/*.rs` as its
own integration-test binary, which would make a bare `tests/invariants.rs` a spurious
0-test target with dead-code warnings that could fail `clippy -D warnings` — R0 L1) — or
alternatively `crates/me-cli/fuzz/invariants.rs`. It expresses every invariant over the
ALREADY-`pub` API (`convert`, `run_bundle`, `ConvertError`, `BundleError`,
`Manifest`/`PlateEntry` — all confirmed `pub` incl. `Manifest.plates` and
`PlateEntry.string` fields), referenced as `mnemonic_engrave::…` NOT `crate::` (R0 N-d,
so the same file compiles in both the tests crate and the separate fuzz crate). Include
it into both consumers with `#[path = "…/invariants.rs"] mod invariants;` (the proptest
test file and each fuzz target). Zero new public items.

## C1 (proptest — CI-covered, load-bearing)

Add `proptest` as a dev-dependency. New `crates/me-cli/tests/prop.rs` (or a
`#[cfg(test)] mod prop` block) with these properties, each over generated inputs:

- **P1 convert-never-panics:** for arbitrary `String` (note `string_regex(".*")` emits NO
  newlines — R0 L3; use `(?s).*` or a byte/`any::<String>()`-based strategy for genuine
  arbitrariness, plus a valid-charset-biased strategy), `convert` returns Ok or Err —
  never panics. (proptest catches a panic as a test failure.) Teeth (R0 N1): a
  perturb-to-panic demo — temporarily `unwrap()` an internal Result — makes P1 red.
- **P2 ms-always-refused (convert):** for any input whose first token has HRP `ms`
  (strategy: `"ms1"` + arbitrary bech32-ish tail, plus case/whitespace-padded variants),
  `convert` → `Err(ConvertError::RefusedSecret)` — never Ok, never a different error that
  would let it proceed. (Guards the secret-refusal invariant against parser changes.)
- **P3 run_bundle-never-panics:** arbitrary MULTI-LINE `String` (strategy MUST inject
  `\n` — e.g. join a `Vec<String>` with newlines, or `(?s)` regex — since `.*` alone
  never produces the multi-line inputs run_bundle is about; R0 L3) → `run_bundle` never
  panics. Teeth (R0 N-a): a symmetric perturb-to-panic demo (temporarily `unwrap()` an
  internal Result in the bundle path) makes P3 red.
- **P4 ms-line-always-refused (bundle):** any input containing an `ms`-HRP line (at any
  position) → `Err(BundleError::RefusedSecret)`. The strategy MUST place `ms1…` as the
  LEADING token of its own line (mirror P2's "first token"; R0 N-c) — a co-located
  `ms1` mid-line classifies by the line's first HRP and would not refuse, so a
  mid-token-`ms1` strategy would falsely fail the property.
- **P5 manifest-strings-trace-to-input:** when `run_bundle(input)` is Ok, for every
  `plate.string == Some(s)`, `s` equals the `trim()` of some line of `input` (i.e. the
  emitted plate string is never fabricated — it is a verbatim trimmed input line). This
  is the funds-relevant "no substitution" property. Precise assertion (R0 N3 — use
  `.lines()`, which handles CRLF, NOT `split('\n')`):
  `input.lines().map(str::trim).any(|l| l == s)`.
  (R0 confirmed this holds today: reassembly output is an integrity oracle only and is
  discarded; the emitted string is always `s.clone()` of a `parse_line`-trimmed line.)
- **P6 ndef round-trip (R0 I1 — corrected domain):** the encoder is **charset-agnostic
  (raw byte copy)**, and its bound is on **UTF-8 byte length, not char count**. So the
  property is **Result-aware over arbitrary `String`**: let `n = t.len()` (bytes);
  if `n <= 249` then `encode_text_tlv(t)` is Ok and `decode_text_tlv(that) == t`;
  if `n >= 250` then `encode_text_tlv(t)` is `Err(TooLong)`. (Do NOT restrict to a
  "printable charset" — that was factually wrong; and do NOT bound on char count — a
  multibyte string of 249 chars can exceed 249 bytes, which would panic a char-count
  strategy on the encoder's real byte-length check.)

Bound proptest cases modestly (e.g. `#![proptest_config(ProptestConfig::with_cases(256))]`)
so the CI suite stays fast; document the knob. Add `proptest` as a dev-dep and **commit
the updated root `Cargo.lock`** (R0 L1) so `cargo test --locked` on stable CI resolves it.
Commit the `proptest-regressions/` seed dir; gitignore `fuzz/{corpus,artifacts,target}`
(R0 N2).

## C2 (cargo-fuzz — local/deep, optional CI smoke)

Add a workspace-detached `crates/me-cli/fuzz/` cargo-fuzz crate. It MUST carry its OWN
empty `[workspace]` table in its Cargo.toml (R0 L2) so it self-detaches from the root
workspace — AND the root `Cargo.toml` should `exclude = ["crates/me-cli/fuzz"]` for
belt-and-suspenders. **Verify after wiring** (R0 L2): `cargo tree` at the root and the
root `Cargo.lock` show NO `libfuzzer-sys` / `arbitrary` — i.e. stable `cargo test
--locked` never pulls the nightly-only fuzz deps. This is the highest-risk executability
check: if it fails, CI reds. Targets:
- `fuzz_convert`: `convert(s)` never panics; if the first token HRP is `ms`, result is
  `RefusedSecret`.
- `fuzz_run_bundle`: `run_bundle(s)` never panics; any `ms` line → `RefusedSecret`; every
  Some plate string is a trimmed input line (P5).
Each target body calls the SAME invariant checkers used by C1 via the `#[path]`-included
shared `invariants.rs` (R0 I2 — NOT via new `pub` API), so proptest and fuzz assert
identical properties. The fuzz crate is intentionally NOT built in CI (R0 L4); that is
acceptable ONLY because every invariant it checks is also exercised by the CI-covered
proptest layer — state this residual in the fuzz crate comment. Document the run command:
`cargo +nightly fuzz run fuzz_convert -- -runs=100000`.

## Ordering & verification
C1 first (the CI-covered guard), then C2 (the deep harness reusing C1's checkers). TDD:
- P-properties: write each property; confirm it PASSES on current code (these guard
  already-correct behavior — per D6-7 no panic exists). Prove *teeth* by perturb-then-revert
  (e.g. temporarily make `convert` accept `ms` → P2 goes red; break `encode/decode`
  symmetry → P6 red; fabricate a plate string → P5 red). Record reds in the log.
- Full verification: `ME_REQUIRE_GO=1 cargo test --locked` at root (proptest included,
  all green; read the actual clean-run baseline rather than assuming a fixed count — R0
  N-b — then + the new prop tests) + `go test ./...` in preview/ still green +
  `cargo +nightly fuzz build` (if nightly available) OR document that fuzz is
  build-verified locally. clippy clean on the non-fuzz crate.

## Open questions — adjudicated at R0 round 0
1. CI fuzz job → **proptest carries CI**, no nightly job (confirmed; F18 is insurance).
2. Fuzz crate placement → `crates/me-cli/fuzz/` with its own `[workspace]` self-detach +
   root `exclude`; verify `cargo tree`/root `Cargo.lock` show no libfuzzer-sys (L2).
3. P6 domain → **corrected (I1):** Result-aware over arbitrary `String`, ≤249 BYTES
   round-trips / ≥250 → TooLong; encoder is charset-agnostic (no printable restriction).
4. Shared-checker visibility → **`#[path]`-included `invariants.rs` (I2)**, zero new
   `pub` API on the published crate.
