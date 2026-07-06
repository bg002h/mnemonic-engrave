# Verify D2-2 (D2-ndef) — ndef-roundtrip go.mod replace points out-of-repo, not the pinned submodule

Verdict: **CONFIRMED as a real defect, but severity downgraded moderate → low.** refuted=false, adjustedSeverity=low, confidence=high.

## The finding, restated

`firmware/ndef-roundtrip/go.mod:7` uses
`replace seedhammer.com => ../../../seedhammer-ref-v1.4.2`, pointing at a directory
*outside* the repo, while the repo already vendors the pinned reader as submodule
`third_party/seedhammer` (v1.4.2 @713aee2) and the sibling `preview/go.mod:12` correctly
uses `../third_party/seedhammer`. Claim: breaks hermeticity + lets the test oracle drift.

## Every factual claim verified TRUE

1. **go.mod:7 content** — `awk NR==7` returns exactly
   `replace seedhammer.com => ../../../seedhammer-ref-v1.4.2`. Line number and text exact.
2. **External dir is out-of-repo** — the replace resolves (relative to
   `firmware/ndef-roundtrip/`) to `/scratch/code/shibboleth/seedhammer-ref-v1.4.2`, a
   sibling of the repo root. `git status` reports it as "outside repository". A case-glob
   test confirms it is NOT under the repo tree. It presently exists (contains `address/`,
   `backup/`, `nfc/`, … — a full seedhammer checkout) but is untracked/unpinned by this repo.
3. **Submodule pin** — `git submodule status` → `713aee2… third_party/seedhammer (v1.4.2)`;
   `git -C third_party/seedhammer describe --tags` → `v1.4.2`. Matches the finding.
4. **preview/go.mod:12** — `replace seedhammer.com => ../third_party/seedhammer`. Correct,
   in-repo, submodule-based. Confirms the inconsistency between the two harnesses.
5. **Hermeticity / hard-fail** — `crates/me-cli/tests/cross_lang.rs` lines 11–14 skip ONLY
   when `go version` errors (go absent); otherwise it runs `go run .` in the harness dir and
   `assert!(out.status.success(), …)` (lines 34–38). On a clean clone that has `go` but lacks
   the external sibling, the `replace` target is missing → `go run .` build-fails →
   status non-success → **test FAILS, not skips.** Exactly as claimed.
6. **No active divergence** — `sha256sum` of the two `nfc/ndef/ndef.go` files is identical
   (`03b784ad…`); `diff -rq` of the whole `nfc/` trees is empty; the external ref is even at
   the same commit `713aee2 (v1.4.2)`. So there is no wrong-plate path *today*.

The repo's own SPEC corroborates the root cause: `SPEC_me_bundle_phaseB_preview.md:18` says
the submodule approach "retir[es] the old local `../../../seedhammer-ref-v1.4.2` dev path" —
i.e. the migration to the submodule was done for `preview/` but the older
`firmware/ndef-roundtrip/go.mod` was left on the pre-submodule dev path. This is a genuine,
unrefutable stale-config / hermeticity defect.

## Why severity is low, not moderate, for a *funds-safety* audit

The harness is a **test oracle only**, never on the shipped path:
- `grep` of `crates/me-cli/src` for `ndef-roundtrip`/`ndefroundtrip` → empty. The harness is
  invoked exclusively by `tests/cross_lang.rs`. The `D6-tests` report agrees: "ndefroundtrip
  module: no test files (exercised only indirectly by cross_lang.rs)."
- The shipped `me` converter and the `preview` sidecar are unaffected; both use the pinned
  submodule / their own code. Nothing this harness does influences what gets engraved.

Realistic failure modes:
- **Clean clone with go, no sibling** → the cross-lang test *build-fails* = a LOUD, red,
  self-correcting failure. It cannot silently ship a wrong plate.
- **"False green" via oracle drift** → requires someone to manually mutate an out-of-repo
  directory into a *strictly more lenient* NDEF reader that still returns the exact input,
  *and* a concurrent real converter bug for it to mask. This is a compound, contrived,
  manual-tampering scenario (no supply-chain/proxy vector), and even then it only weakens one
  cross-check test — the converter carries its own unit tests. No direct lost-funds path.

So the defect is real and worth fixing (trivially: `replace seedhammer.com =>
../../third_party/seedhammer`, mirroring preview), but in a funds-safety framing it is a
hermeticity / test-integrity hygiene issue (low), not a moderate wrong-plate risk. The finder
itself concedes "no active divergence," which is inconsistent with a moderate funds rating.

## Verdict
- refuted: **false** (claim is concretely substantiated at the cited location)
- adjustedSeverity: **low**
- confidence: **high**
