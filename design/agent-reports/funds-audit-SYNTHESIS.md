# funds-audit SYNTHESIS — mnemonic-engrave user-funds-safety audit

Date 2026-07-06. Controller: fable. Fan-out: 6 opus finders + 20 opus adversarial
refuters (workflow wf_bac78b95-878; 26/26 agents completed, 0 errors). Verbatim finder
reports: `funds-audit-D{1..6}-*-round0.md`; verbatim refuter verdicts:
`funds-audit-verify/`. Verification rule: critical/important = 2 independent refuter
votes, moderate = 1; refuters instructed to default-refute.

**Headline: no confirmed CRITICAL finding — no path was found by which `me` engraves
bytes that differ from the user's input, drops/reorders shares, or admits a
checksum-invalid string.** The confirmed risks are one secret-leak regression on the
bundle error path, systemic test/CI gaps that would let such a bug ship in the future,
and two admission-laxness items.

## Confirmed IMPORTANT

### F1 (D5-1) `me bundle` echoes the full input line to stderr — leaks an ms1 secret body on the mangled-HRP path
`crates/me-cli/src/bundle.rs:54-60` + `main.rs:184`. BundleError::{Classify, Validate,
Md1HeaderRead} Display arms interpolate the full input string. The convert path was
deliberately hardened to never echo input; the bundle path regressed this. The ms1
refusal is a classify-only pre-scan matching HRP `ms` exactly, so an ms1 secret with a
1-typo HRP (e.g. `msx1…`) dodges refusal and its intact codex32 secret body is printed
verbatim to stderr (shell scrollback, `2>logfile`, CI logs). Proven end-to-end with the
release binary by both refuters (2× refuted=false, high confidence).
**Fix:** redact input in all BundleError Display arms (bounded prefix/HRP only), matching
ConvertError. **Test:** assert_cmd — mangled-HRP ms1 into `me bundle`, assert secret body
absent from stderr (fails today).

### F2 (D6-1) No CI runs any test suite — release.yml is build-only; a red suite can merge and be tag-released
`.github/workflows/release.yml`. Only workflow; jobs run `cargo build`/`go build`
(+ sign/assemble on tag), never `cargo test`/`go test`. No Makefile/justfile/hooks.
Every funds-safety invariant is guarded only by locally-run tests. Both refuters
confirmed (adjusted critical→important: latent process gap, not an active wrong-plate
path). **Fix:** test job (Rust + Go in one job so differential tests actually run),
`assemble.needs` gated on it.

### F3 (D6-2) Cross-language differential tests vacuously PASS when Go is absent
`crates/me-cli/tests/cross_lang.rs:11-14`, `preview_cross_lang.rs:82-85` — early
`return` on missing `go` counts as PASS (reproduced live by both refuters). The
strongest oracle (SeedHammer's real NDEF reader parsing what `me` emits) silently
becomes a no-op. **Fix:** `ME_REQUIRE_GO=1` opt-in that hard-fails when `go` is missing;
CI sets it (composes with F2).

## Confirmed MODERATE

### F4 (D1-1) md1 separators/whitespace pass BCH validation but are emitted verbatim — checksum does not cover emitted bytes
`crates/me-cli/src/lib.rs:57-63`. md-codec's `unwrap_string` strips `-`/whitespace
before BCH-verify; `convert()` then encodes the raw trimmed input. A mid-string newline
or soft-wrap hyphen yields exit 0 + the stray byte embedded in the NDEF payload (proven;
contradicts validate.rs's "a corrupted string is never engraved" doc). Not critical:
separators are legal BIP-93 and conformant decoders strip them; but a newline cannot be
faithfully engraved and non-conformant readers may choke. **Fix options:** refuse
non-canonical md1, or canonicalize (strip) before encoding — decide in SPEC (Rust-primary
rule check: canonicalization semantics belong to md-codec upstream; see F5).

### F5 (D1-2) Stale md-codec 0.36 pin admits over-length (>93-symbol) single md1 that current md-codec rejects
`Cargo.lock` pins md-codec 0.36.0 (no upper length bound in `unwrap_string`); md-codec
0.40 added the fail-closed `StringSymbolCountOutOfRange` guard (cycle-4 I1). Reproduced:
a 94-symbol md1 built with 0.36's own `wrap_payload` → `me` exit 0; same string rejected
by 0.40. Beyond 93 symbols BCH detection guarantees lapse — `me` can stamp "validated"
on a string an updated decoder refuses. **Fix:** bump md-codec to ≥0.40 (+ mk-codec
review), add refusal test. Conforms to Rust-primary rule (guard already landed in the
primary Rust codec first).

### F6 (D6-4) NDEF TLV short-record 255-boundary entirely untested; boundary mutation yields device misparse
`crates/me-cli/src/ndef.rs:48`. Guard `message.len() >= 0xFF` is correct today (max text
249), but a `>=`→`>` mutation emits a 0xFF length byte that SeedHammer's reader
(`third_party/seedhammer/nfc/ndef/ndef.go:73`) treats as the 2-byte-length escape →
total misparse. No test pins the boundary. **Fix:** byte-pinned 249-char golden +
250-char TooLong assertion + Go-reader round-trip at the boundary. (Subsumes refuted
D2-1's test recommendation.)

## Confirmed LOW (fold into SPEC/FOLLOWUPS; verdicts on disk)

- **F7 (D2-2)** `firmware/ndef-roundtrip/go.mod` `replace` points OUTSIDE the repo
  (`../../../seedhammer-ref-v1.4.2`) instead of the pinned submodule — oracle can drift
  or break hermetic builds. Fix: repoint to `../../third_party/seedhammer`.
- **F8 (D3-1 ≡ D5-3)** `--preview` never cleans the output dir: stale higher-index
  `plate-N` images from a previous (different-wallet) run persist → cross-run plate
  mixing if the user engraves "everything in the folder". Proven.
- **F9 (D3-2)** Sidecar output never validated: fake sidecar exiting 0 with a 0-byte SVG
  → `me` records a valid preview, exit 0. Proven.
- **F10 (D5-2)** NDEF/manifest/preview artifacts written 0o644 world-readable; manifest
  embeds raw strings, images depict scannable QR.
- **F11 (D5-4)** PATH-based sidecar discovery + string-match version gate (no integrity).
- **F12 (D4-2)** Geometry golden derives from preview's own hardcoded params — a
  submodule device-constant bump (e.g. tmc2209.Microsteps) drifts silently. Fix: derive
  `mm` assertion from `seedhammer.com/driver/tmc2209` in params_test.go.
- **F13 (D4-3)** PNG preview draws 1px hairlines vs SVG's physical 0.3mm strokes
  (legibility-judgment divergence only; centerlines identical).
- **F14 (D6-3, downgraded 1-1 split → low)** Golden corpus = one 24-char md1; no
  mk1/long/alphabet goldens; in-crate round-trips use me's own decoder (symmetric-bug
  blind). Corpus expansion worthwhile even though 3 independent anchors exist.
- **F15 (D6-5, downgraded)** No SVG path-content/PNG pixel golden — pen-state swap
  renders wrong preview with green suite.
- **F16 (D6-8)** ms1-refusal tested with a single lowercase vector; add
  uppercase/padded/mixed-case/bad-checksum table.
- **F17 (D6-6 residual, refuted as important)** chunk-discriminator arms untested; add
  drift-guard pinning md-codec 0.36 discriminator behavior (funds-loss scenario refuted:
  bit layout + existing set-completeness check prevent it).
- **F18 (D6-7 residual, refuted as moderate)** No fuzz/proptest targets; panic-freedom
  fuzzing still cheap insurance.

## REFUTED (rationale in funds-audit-verify/)

- **D2-1** (single-vector cross-lang as *moderate funds bug*) — scenario unreachable by
  valid input; test value folded into F6/F14.
- **D4-1** (preview QR may diverge from device md1/mk1 engrave path) — in-repo design
  sources authoritatively specify the fork path; representations match.
- **D6-6 as important** (chunk misclassification admits lone chunk) — discriminator
  mechanics + existing tests make scenario unreachable; residual = F17.
- **D6-7 as moderate** — no reachable panic/misroute found; residual = F18.

## Sound areas (negative results — see finder reports for full checklists)

Checksum enforcement present on all entry paths; ms1 refusal correct for exact-HRP
inputs (case-folded); NDEF encoding field-correct vs spec and byte-verified against
SeedHammer's real reader for current-domain inputs; bundle share ordering/sequencing and
set-completeness sound; params.go verbatim-replication claim verified field-by-field
against upstream platform_sh2.go; QR content byte-identical to payload; no argv secret
intake (stdin/file only); zeroize used within its real limits.

## Recommended cycle scope (next session)

1. **Production fixes** (one implementer, TDD, R0-gated): F1 redaction; F5 codec bump;
   F4 canonicalization decision; F7 replace repoint; F2+F3 CI wiring; optionally
   F8/F9/F10 (cheap adjacent hardening).
2. **Test hardening** per `design/SPEC_me_testing_hardening.md` (drafted from D6 report
   + confirmed findings).
