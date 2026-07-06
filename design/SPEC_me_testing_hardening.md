# SPEC — me funds-safety fixes + testing hardening (post-audit cycle)

Status: **GREEN — R0 passed at round 1 (0C/0I/5 nit), nits folded inline 2026-07-06**
(reviews: `agent-reports/me-testing-hardening-spec-R0-round0.md` = 0C/2I/6L all folded;
`…-round1.md` = GREEN, closure of both Importants verified against codec source).
Source: `design/AUDIT_FUNDS_SAFETY.md` +
`design/agent-reports/funds-audit-SYNTHESIS.md` (finding ids F1–F18 refer there; verbatim
evidence in the D1–D6 finder reports and `funds-audit-verify/` verdicts).
Process: R0 architect gate to 0C/0I → single implementer, TDD, worktree → mandatory
post-implementation adversarial execution review.

## Goal

Close the confirmed audit findings: three production fixes, one dependency bump, CI
wiring, and a test suite that would actually reveal funds-safety regressions (engraved
bytes ≠ validated input, checksum bypass, secret leak, preview/bundle divergence).

## Non-goals

- md-codec / mk-codec internals (audited separately; only the pin bump here).
- The seedhammer fork's firmware (separate repo/process).
- Refuted findings D2-1, D4-1, D6-6-as-important, D6-7-as-moderate (no production change;
  residual test items folded below).

## Part A — production fixes (TDD: failing test first)

**A1 (F1, important). Redact input in bundle error paths.**
`BundleError::{Classify, Validate, Md1HeaderRead}` Display must never contain the input
string body — bounded HRP/prefix + metadata only, matching ConvertError's hardening.
Acceptance: assert_cmd tests feed (a) mangled-HRP ms1 (`msx1<secret body>`), (b)
corrupted mk1 to `me bundle`; stderr must not contain the body substring. Both fail today.

**A2 (F5, moderate). Bump md-codec 0.36 → ≥0.40; bump mk-codec 0.4.0 → latest 0.4.x and
re-run all mk1 fixtures** (concrete, not a "review": if any mk1 fixture output changes,
STOP and flag — that would be a normative drift needing Rust-primary adjudication).
Brings in the fail-closed `StringSymbolCountOutOfRange` guard.
Acceptance: 94-symbol md1 built via 0.36 `wrap_payload` (fixture string, not a build-time
dep) → `me convert` errors, non-zero exit. Fails today. Re-baseline any goldens ONLY if
bytes change for still-valid inputs (they must not — flag to reviewer if they do).

**A3 (F4, moderate). Refuse non-canonical md1 (decision adjudicated at R0 round 0).**
**Canonical is defined post-trim:** the input is first trimmed of leading/trailing
whitespace exactly as today (`str::trim`, i.e. Unicode `char::is_whitespace()`);
*canonical* = the trimmed string contains no interior character matching
`char::is_whitespace()` (same predicate as `str::trim` and md-codec's strip step — not
ASCII-only) and no `-` anywhere. Non-canonical → fail-closed error
(exit non-zero, naming the offending character and byte position; message must not echo
the full input on the bundle path per A1). Refusal (not canonicalization) because silent
stripping would emit bytes the user never supplied; canonicalize-then-emit would also
require the semantics to land in md-codec first (Rust-primary rule).
Acceptance: (a) interior `md1…x-y…` and `md1…x y…` → error on convert AND bundle paths;
interior `md1…x\ny…` → error on the convert path (on the bundle path a multi-line input
is line-split before `parse_line`, so the newline case cannot exercise the bundle-side
guard — the single-line space/dash cases cover it there); (b) **positive regression
guard:** the same clean md1 supplied with a
trailing `\n` (typical stdin pipe) and without one produce byte-identical output and
exit 0 — the refusal must not fire on post-trim-clean input.

**A4 (F7, low). Repoint `firmware/ndef-roundtrip/go.mod` replace** to
`../../third_party/seedhammer` (match preview/go.mod). Acceptance: cross_lang round-trip
builds hermetically in a clean checkout with only the submodule initialized.

**A5 (F2+F3, important). CI test gating.**
`.github/workflows/test.yml` (or job in release.yml): one job with BOTH Rust and Go
toolchains running `cargo test --locked` + `go test ./...` (preview/ and
firmware/ndef-roundtrip/) on every push/PR; `assemble` gains `needs:` on it so a red
suite blocks tag publish. The job MUST check out with `submodules: true` (the Go oracle
builds against `third_party/seedhammer`) and MUST set `ME_REQUIRE_GO=1` in its env.
Tests gain `ME_REQUIRE_GO=1` handling: when set, missing `go` is a hard failure instead
of a skip. Acceptance: deleting `go` from PATH with ME_REQUIRE_GO=1 fails the suite;
without the var, local behavior unchanged (skip note).

**A6, A7 — DESCOPED to FOLLOWUPS (R0 round 0, L6: keep one tight implementer cycle).**
Recorded in `design/FOLLOWUPS.md` as `me-preview-stale-plates-and-sidecar-output-validation`
(F8+F9) and `me-output-file-permissions` (F10).

## Part B — test hardening

**B1 (F14, F6). Golden corpus expansion** — two layers (folded per R0 I1: a valid md1
caps at ~96 chars — codex32 93-symbol limit — and mk1 at 111, so no *valid* input reaches
the 249/250 SR boundary; boundary tests therefore target the NDEF layer directly):
- **convert-level goldens** (`tests/vectors/`, byte-pinned `.ndef`, decoded via the Go
  oracle not me's own decoder): (a) a maximum-length valid md1 (at the 93-symbol codex32
  cap, ≈96 chars with HRP — "maximum-length", not "exactly 93 required") and existing
  short md1; (b) mk1 short + mk1 chunk (111-char); (c) bech32-alphabet coverage as a
  UNION across vectors (a single all-symbols string need not be constructible — every
  charset symbol must appear in ≥1 vector, positions varied).
- **ndef-layer boundary tests** (unit, on `encode_text_tlv` with synthetic text, no
  codec validation involved): 249-char text byte-pinned with TLV length byte asserted
  < 0xFF; 250-char text → `NdefError::TooLong`.

**B2 (F3, F14). Differential decode via the independent Go oracle** — round-trip through
`firmware/ndef-roundtrip` (SeedHammer's real reader, post-A4 hermetic) at both layers:
every B1 convert-level golden, PLUS ndef-layer synthetic texts at lengths
{1, 63, 64, 96, 111, 248, 249} (padding-safe synthetic strings; these bypass codec
validation by construction); assert decoded text == input positionally. Never decode
goldens with me's own `decode_text_tlv` alone.

**B3 (F16). ms1-refusal table test** — {lowercase, UPPERCASE, mixed-case, padded
whitespace, bad-checksum ms1, ms1 at each bundle line position} → RefusedSecret/exit 3,
and (post-A1) stderr never contains the body.

**B4 — DESCOPED to FOLLOWUPS (R0 round 0, L6)** as `me-preview-render-goldens` (F15).

**B5 (F12). Device-constant drift guard (Go)** — params_test.go asserts
`mm == 200/8*tmc2209.Microsteps` and `strokeWidth == mm*3/10` by importing
`seedhammer.com/driver/tmc2209` (confirmed host-compilable — round-1 architect verified
statically: the tinygo-tagged file is `uart.go` (`//go:build tinygo && rp`), and the
untagged files including `uart_pio.go` use only stdlib + pio's untagged `config.go`;
implementer re-verifies with `go vet` before relying on it), so a submodule constant
bump fails the build.

**B6 (F17). md1 chunk-discriminator drift guard** — fixtures hitting all four
`parse_line` md1 arms (incl. ChunkHeaderChunkedFlagMissing and
WireVersionMismatch→Md1WireVersion); a test pinning the md-codec discriminator behavior
the bundle probe relies on; single-chunk-of-multichunk input → SetIncompleteMd, never a
lone Md1Single. Codec pinning already adjudicated (Open-Q4): keep caret ranges +
Cargo.lock, no `=` exact pin — this drift-guard test is the load-bearing protection.

**B7 — DESCOPED to FOLLOWUPS (R0 round 0, L6)** as `me-fuzz-proptest-targets` (F18).

**B8 (F1, A1). Secret-leak regression tests** — as specified in A1 acceptance; also
grep-style assertion that no error path Display in me-cli interpolates a full input
`{s}` (unit test over error variants with a marker string).

## Finding dispositions not otherwise covered (R0 round 0, L1)

- **F11** (PATH-based sidecar discovery / spoofable version gate) → FOLLOWUPS
  (`me-sidecar-discovery-integrity`): payload sent is public-only by construction;
  a real integrity gate (hash pin or co-located-only) is its own small cycle.
- **F13** (PNG 1px hairlines vs SVG physical stroke width) → FOLLOWUPS
  (`me-preview-png-stroke-width`): centerlines identical, legibility-judgment only.

## Ordering & verification

A4 → A5 first (makes every later test actually run in CI), then A1/A2/A3 with their
failing-test-first pairs, then B1/B2/B3/B8, B5/B6. Full suite + `me bundle --preview`
end-to-end run (real sidecar) before the mandatory post-impl adversarial review.

## Open questions — adjudicated at R0 round 0

1. A3 refuse vs canonicalize → **refuse** (fail-closed, Rust-primary-safe); canonical
   defined post-trim/interior-only (see A3).
2. A6a refuse-nonempty vs clean-namespace → moot; A6 descoped to FOLLOWUPS.
3. Descoping → **applied**: A6, A7, B4, B7 out; kept set = A1–A5, B1/B2/B3/B5/B6/B8.
4. Codec pinning → **keep caret ranges + Cargo.lock** (no `=` pins); the B6 drift-guard
   test is the load-bearing protection.
