# me funds-safety + testing-hardening — post-implementation adversarial execution review (round 0)

**Reviewer:** independent opus execution reviewer (mandatory Step 12 gate).
**Target:** full diff `master..me-testing-hardening` (12 commits) in worktree
`/scratch/code/shibboleth/mnemonic-engrave-testing-hardening`.
**Date:** 2026-07-06.
**Sources read:** `IMPLEMENTATION_PLAN_me_testing_hardening.md` (GREEN plan-R0 r1),
`SPEC_me_testing_hardening.md` (GREEN spec-R0 r1), `funds-audit-SYNTHESIS.md`
(F1–F18), the impl-log, and the extracted codec registry sources
(md-codec 0.36.0/0.40.0, mk-codec 0.4.0/0.4.1).

## Verdict: **GREEN (0 Critical / 0 Important)**

Two Low findings + one Nit, none blocking. The diff introduces **no funds-safety
regression**, no test theater (five perturbations independently re-verified RED),
no scope creep. The three intended behavior changes (over-length md1 refused,
non-canonical md1 refused, and the mangled-HRP secret-leak closure) are correct,
fail-closed, and secret-safe. CI wiring is sound (actionlint clean, `assemble`
gated on `test`). Full suite reproduced green here: **82 tests** (54 lib + 23 cli
+ 1 cross_lang[11 oracle round-trips] + 3 golden + 1 preview_cross_lang), zero
skips under `ME_REQUIRE_GO=1` with Go present.

---

## Methodology / probes actually run

- Full `cargo test --locked` under `PATH=+go ME_REQUIRE_GO=1` → all green (82).
- Regenerated the two md1 fixtures from the committed scratch recipes and compared
  byte-for-byte to the committed literals (both **byte-identical**).
- Re-performed FIVE guard perturbations in an rsync'd scratch copy (worktree never
  touched), each reverted after observing RED:
  1. ndef text byte-flip → `cross_lang` RED (`convert-golden round-trip mismatch`) —
     proves the table genuinely shells to the Go oracle and catches me/oracle divergence.
  2. corrupt 1 byte of `md1-max.ndef` → `golden.rs::all_vectors_match_golden_ndef` RED.
  3. delete the `MdNonCanonical` check → 4 lib + 2 cli tests RED.
  4. `ME_REQUIRE_GO=1` with `go` off PATH → hard panic (exit 101) at `cross_lang.rs:55`.
  5. `mm=6401` in params.go → `TestDeviceConstantsMatchDriver` RED, green after revert.
- Ran `actionlint .github/workflows/release.yml` → exit 0, no findings.
- Enumerated every `md_codec::Error` (49) and `mk_codec::Error` (23) variant's Display
  against the registry sources for reachable-leak analysis.
- Diffed mk-codec 0.4.0→0.4.1 source in full.

---

## Priority 1 — funds-safety regressions introduced by the diff

**None found.** Behavior changes are limited to the intended refusals:

- **Over-length md1** (`lib.rs` codec bump + `refuses_overlength_md1`): fail-closed via
  `md_codec::Error::StringSymbolCountOutOfRange`. `convert()` still emits the input
  verbatim for accepted strings, so no accepted input's engraved bytes change
  (goldens byte-identical, confirmed).
- **Non-canonical md1** (`validate.rs:66-69`): the interior-separator scan uses
  `c.is_whitespace() || *c == '-'`, which is the **identical predicate** md-codec's
  own `unwrap_string` uses to strip separators
  (md-codec-0.40.0/src/codex32.rs:160: `if c.is_whitespace() || c == '-'`). Rust's
  `str::trim` is likewise defined on `char::is_whitespace`. Because a legitimately
  canonical md1 is pure codex32 charset (`qpzry9x8gf2tvdw0s3jn54khce6mua7l` — no
  whitespace, no `-`, no `1`), the check **cannot reject a legitimately-canonical
  string**. Leading/trailing `-` is caught earlier by `classify` (HRP mismatch), so
  the post-trim interior scan sees only genuinely interior separators. No Unicode
  edge: trim and the scan share the predicate, so no disagreement at the boundary.
- **New error paths do not echo input.** Grep of every new/changed
  `write!/format!/eprintln` interpolation: the three redacted bundle arms show only
  `{e}`; `MdNonCanonical` Display shows only `{ch:?}` (always a single whitespace/`-`
  char) + `{pos}`. `main.rs` is unchanged; its one input-echoing site (`:112`
  "validated {label}: {s}") is the success path (post-validation, public md1/mk1;
  ms1 is refused before it) and is out of scope.

The primary F1 vector is closed: `msx1<body>` routes to `classify` →
`UnknownHrp("msx")` (HRP = chars before the first `1`, i.e. `"msx"`), so the secret
body (after the `1`) is never interpolated. Verified end-to-end (cli test
`bundle_msx1_mangled_hrp_does_not_leak_secret_body`, green).

## Priority 2 — redaction completeness (→ one Low)

The diff's own redaction is clean: `BundleError::{Classify,Validate,Md1HeaderRead}`
now drop the raw `s` (`bundle.rs:59,60,65`) and show only the bounded underlying
error, mirroring `ConvertError`. `SetIncomplete{Mk,Md}`'s first field is
`fmt_chunk_set_id(id)` = `format!("0x{id:05x}")` — a 20-bit **public** chunk-set id
(md1/mk1 are public; ms1 never reaches here), bounded, not raw input. Adjudicated
safe.

Exhaustive variant sweep result: **`md_codec::Error` is fully clean** for every
me-reachable path (`unwrap_string`, `ChunkHeader::read`, md1 reassembly) — the only
`String`-carrying reachable variant, `Codex32DecodeError`, is constructed solely
from fixed text / the `"md1"` HRP literal / a single offending char.
`AddressDerivationFailed` (miniscript text) is unreachable — me never derives.

**Finding L1 (Low): `mk_codec::Error::InvalidHrp(String)` is not metadata-only.**
`mk-codec-0.4.1/src/string_layer/bch.rs:668,673` — `decode_string` returns
`InvalidHrp(s_lower.clone())` when the input has no `1` separator, and
`InvalidHrp(hrp.to_string())` (the substring before the **last** `1`) when that HRP
≠ `"mk"`. This flows through `ValidateError::Mk(_)` → `BundleError::Validate(_, e)` /
`ConvertError::Validate(e)` → stderr. The impl-log Step-3 claim that *"md_codec and
mk_codec Display are metadata-only"* is therefore **inaccurate for this one variant**,
and the B8 test uses `ValidateError::MkCorrected(2)` — it never exercises the
codec-wrapped `ValidateError::Mk(InvalidHrp)` sub-case, so the residual is untested.

Why this is **Low, not Important**, and does **not** block GREEN:
- **Not a regression** — pre-diff, `BundleError::Validate` echoed the *entire* input
  `s` unconditionally; the diff strictly reduces leakage. The same `{e}` pass-through
  already existed (unchanged) on the convert path.
- **The 668 (no-separator) branch is unreachable in me**: `classify` requires a `1`
  before it will return `Format::Mk`, so `decode_string` is only ever called on a
  string that already contains a `1`.
- **Cannot leak an ms1 secret body.** To hit `673` with secret material the input
  must (a) classify as `mk` (HRP mistyped `ms`→`mk`) AND (b) contain a *stray* `1` in
  the body — but `1` is not in the codex32/bech32 alphabet, so a genuine secret has
  none. A clean `mk1<ms-secret>` has only the separator `1` → `hrp=="mk"` → no
  `InvalidHrp`; it proceeds to BCH and errors with metadata only. Exploitation needs a
  compound double-typo producing a non-alphabet char, and only echoes a truncated
  prefix — materially weaker than the (now-closed) single-typo F1.
- Per the Rust-primary rule and the SPEC non-goal ("md-codec/mk-codec internals
  audited separately; only the pin bump here"), bounding this Display belongs upstream
  in mk-codec; A1's scoped job (stop **me's own** arms interpolating `s`) is done.

Recommended follow-up (non-blocking; controller may fold inline as it is one line):
either match `ValidateError::Mk(mk_codec::Error::InvalidHrp(_))` in the me Display and
render bounded text, or file a mk-codec (Rust-primary) issue to bound `InvalidHrp`;
and correct the impl-log's "metadata-only" phrasing + extend B8 to cover
`ValidateError::Mk(InvalidHrp("mk1…"))`.

## Priority 3 — test theater

No theater. All five perturbations above went genuinely RED and reverted to green.
`ME_REQUIRE_GO=1` hard-fails without Go (verified). `preview_cross_lang.rs` has
exactly one `#[test]`, and it is guarded — no unguarded skip site remains. The
cross-lang table asserts `decoded == input` for all four convert-level goldens plus
seven synthetic NDEF lengths, all through `go run .` on the real SeedHammer reader.

## Priority 4 — golden provenance

`golden.rs::all_vectors_match_golden_ndef` (green) IS the characterization proof that
each `.ndef` equals `convert(input)`. The cross_lang `GOLDEN_INPUTS` array is
byte-identical to golden.rs's four `VECTORS` inputs, so the Step-7 oracle round-trip
**covers every golden**. Regenerated `OVERLEN_MD1` (0.36 recipe) and md1-max (0.40
recipe) from the committed scratch generators — **both byte-identical** to the
committed literals. `md1-max.ndef` is 104 bytes with the exact claimed framing
`03 65 D1 01 61 54 00 …(96 text)… FE` (verified: TLV len 0x65=101=5-byte header+96
text; payload len 0x61=97=status+96; terminator 0xFE). Alphabet-union test genuinely
covers all 32 bech32 symbols (perturbation shows it goes RED on an uncovered symbol,
per impl-log; union test green here).

## Priority 5 — CI YAML

- `actionlint` → **clean (exit 0)**.
- Trigger set now `push.tags:v* + push.branches:master + pull_request`; the `test`
  job has no `if:`, so it runs on every push (incl. tag) and PR.
- `assemble.needs: [test, go-build, rust-build]` — on a `v*` tag the workflow fires,
  `test` runs, and `assemble` (guarded `if: refs/tags/v`) waits on it: a red suite
  **blocks tag publish** (F2 tag-half closed). PR-merge blocking still requires the
  branch-protection repo setting (correctly flagged as a USER ACTION in plan Step 2 /
  impl-log — outside YAML).
- `test` job: `checkout submodules:true`, Rust + Go in the **same** job,
  `env ME_REQUIRE_GO:'1'`, steps `cargo test --locked` + `go test ./...` (preview) +
  `go build ./... && go test ./...` (firmware). `cache-dependency-path: preview/go.sum`
  is valid (`preview/go.sum` exists; the firmware module has no go.sum by design — a
  local-replace v0.0.0 dep with no checksums, so its `go build/test` needs no network
  and no cache). Wiring is correct.

## Priority 6 — mk-codec 0.4.0→0.4.1

Full source diff: **zero normative/admission change.** Only a `POLYMOD_INIT`
doc-comment reword (constant value unchanged), two additive `key_card` methods the
CLI never calls, a new test file, and metadata. Every caller-touched file
(`string_layer/decode.rs`, `header.rs`, `error.rs`, `bytecode/`) is byte-identical;
the Error-variant set and `corrections_applied` behavior are unchanged. The A2
"live-risk" (a previously-valid mk1 newly rejected) did not materialize — consistent
with the impl-log's STOP-condition check.

## Priority 7 — plan fidelity + hygiene

Every plan step (1–11) is present, one commit each. Changed-file set is exactly the
deliverable table + impl-log + the 3 new `.ndef` vectors — **no scope creep**.
`git status` clean; the `ndefroundtrip` build artifact is **not tracked** (Nit N1
below). Cargo.lock shows **only** md-codec 0.36→0.40 and mk-codec 0.4.0→0.4.1
(version+checksum) with no transitive-dep drift; Cargo.toml keeps caret ranges
(no `=` pin), matching Open-Q4. Impl-log claims cross-checked against the actual
diffs are accurate (byte framing, counts, predicate, ordering) except the L1
"metadata-only" overstatement.

**Nit N1:** `firmware/ndef-roundtrip/` builds a `ndefroundtrip` binary that is not in
`.gitignore` (the module dir has no gitignore entry). Harmless in ephemeral CI and
untracked today, but a local `git add -A` could stage it. The plan mandates explicit
staging (no `-A`), so low risk; the implementer already flagged this. Add
`firmware/ndef-roundtrip/ndefroundtrip` (or a bin-glob) to `.gitignore` in a future
touch.

**Informational (not a finding):** md-codec 0.36→0.40 is a 4-minor jump; only the mk
bump was independently source-diffed for normativity in this review. The md bump's
admission delta is guarded empirically by the agreed A2 STOP-condition (goldens
byte-identical AND no previously-passing admission test newly failing) plus the
intended over-length refusal — all held. Deeper md-codec archaeology is a SPEC
non-goal (codec internals audited separately).

---

## Findings summary

| # | Sev | Location | Finding |
|---|-----|----------|---------|
| L1 | Low | mk-codec-0.4.1 bch.rs:668,673 via `ValidateError::Mk` (bundle.rs:60 / lib.rs Validate) | `mk_codec::Error::InvalidHrp` echoes an input substring; impl-log "metadata-only" claim inaccurate, B8 doesn't cover it. Not a regression, not an ms1-secret leak in practice (codex32 has no `1`). |
| L2 | Low | bundle.rs B8 test | `no_bundle_error_display_leaks_the_input_body` asserts "every arm redacts the input" but only tests non-codec Validate sub-variants; the codec-wrapped `ValidateError::Mk/Md` pass-through (the actual residual surface) is unexercised. |
| N1 | Nit | firmware/ndef-roundtrip/.gitignore | `ndefroundtrip` build artifact not gitignored. |

**0 Critical, 0 Important → GREEN.** Merge-eligible. The two Low items are
defense-in-depth / documentation-accuracy follow-ups (controller may fold the one-line
InvalidHrp match inline or record to FOLLOWUPS); neither is a funds-safety regression
nor a reachable secret leak, and the headline F1 vector is fully closed.
