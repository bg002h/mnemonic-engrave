# Track A (verify-cluster) PLAN — R0 review (round 0) — VERBATIM agent report

**Agent:** `a974e996e0d16a694` (adversarial opus architect; executed all 7 tasks + ran every FAIL→PASS in a throwaway worktree off `3a23dbb`). **Fork HEAD:** `3a23dbb`. **Plan commit:** `00e1530`. **Spec:** `803296a`. **Date:** 2026-06-20.
**Verdict:** GREEN (0 Critical / 0 Important). 2 cosmetic Minors. Cleared for implementation. Concurrent with Track B.

---

# R0 GATE REVIEW — IMPLEMENTATION_PLAN_seedhammer_verify_cluster_fixes (Track A)

**Reviewer:** adversarial opus architect (R0 gate; applied every diff + ran FAIL→PASS in a throwaway worktree)
**Fork HEAD:** `3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082` (`main`, verified live)
**Plan commit:** `00e1530` | **GREEN spec:** `803296a` | **Spec-R0 folds:** `seedhammer-verify-cluster-spec-R0-round0.md`
**Verdict:** **GREEN — 0 Critical / 0 Important.** I executed all 7 tasks in order; every diff applied cleanly, every FAIL→PASS reproduced, full `go build`/`go test ./...`/`go vet` green.

---

## Per-mandate findings (with observed output)

### Mandate 1 — Apply each task's diffs; confirm FAIL→PASS routing the PRODUCTION function (the core of the gate)

All three flow-level tests reproduced fail-before / pass-after on `3a23dbb`, routing production functions:

**T-M1** (`bundle/verify_test.go`, routes `Verify`/`ms1Entropy`):
- Verified the fixture is genuine: `ms10entrsqgqsqqqqqqqqqqqqqqqqqqqqqqqqqj9tawneveyd9j` decodes to `prefix=2 (mnem) language=1 entropy=00..00` — entropy-identical to `wpkhMS1` (`prefix=0/entr language=0`) but a different wordlist.
- FAIL-before: `TestVerifyBundleLanguageMismatch` → `language-differ readback (same entropy) accepted, want FAIL` (the M1 bug; `Verify` returned nil). `TestVerifyBundleLanguageEnglishNotOverRejected` PASSED (no over-rejection).
- PASS-after: both PASS; error string confirmed `verify: ms1 wordlist/language mismatch`. Full `bundle` suite `ok`.

**T-H2** (`gui/md1_gather_test.go`, routes `md1Gatherer.collected()`):
- FAIL-before: `TestMD1GathererCollectedIndexOrder` → `collected()[0]=...want index order...` (map-range ≠ index order). `TestMD1GathererShuffledGatherExpands` PASSED even before the fix (order-tolerant end-to-end guard, as the plan states — discriminator is the index-order test).
- PASS-after: both PASS across 4 arrival orders × 10 trials each; full `gui` suite `ok` (the third consumer `bundle.go:234 offerChunkedMD1` unaffected). `TestVerifyBundleMd1PositionalContract` (relabeled) stays green, assertion unchanged.

**T-H1** (`gui/multisig_verify_test.go`, routes production `extractSuppliedMd1AndMk1` + `verifyMultisig`):
- All 4 subtests PASS. Captured the discriminating error strings directly:
  - undecodable mk1 → FAIL: `verify: readback mk1 decode: codex32: not a valid mk1 string`
  - decodable-but-wrong foreign mk1 → FAIL via the stub-binding leg: `verify: readback mk1/md1 stub mismatch (key card does not bind to this policy)`
  - masking proof: today's self-compare (`verifyMultisig(derived, …, derived.MK1, derived.MD1)`) returns `<nil>` — the wrong plate silently PASSES. The bug is demonstrated.

### Mandate 2 — Diff fidelity vs GREEN spec + folds (full production diff audited)

- **H1 = option (b):** NEW `extractSuppliedMd1AndMk1` added; `extractSuppliedMd1` left untouched (`git diff` of `gui/multisig.go` vs `3a23dbb` is empty — engrave caller `:71` unaffected). `derived` param KEPT; only the *argument* changed (`reDerived.MK1` → `suppliedMk1`). Return order `(md1, mk1, ok)` consistent across def/tests/wiring.
- **H2** = `collected()` body only (index walk); no signature change. All 3 `collected()` sites (`md1_gather.go:83,147` + `bundle.go:234`) are `complete()`-guarded.
- **M1** compares LANGUAGE (`dLang != rLang`), not raw prefix. `DecodeMS1` confirmed 4-return with `language int`.
- **L2** honest copy via named constants + `uiContains` regression test; only the multisig success string changed (single-sig flow's own honest copy untouched).
- **L1** scrubs the 2 verify-flow sites only; Track B's `codex32_polish.go:103` untouched.
- **No unplanned hunk / scope creep / signature ripple.** The 9 changed files exactly match the plan's table; production diff is precisely the 5 fixes.

### Mandate 3 — No-regression + build/vet

- `go build ./...` → clean (exit 0).
- `go test ./...` → every package `ok`, no FAIL, no panic (incl. `bundle`, `gui`, and the pre-existing `TestVerifyMultisig` which calls `verifyMultisig` directly).
- `go vet ./gui/ ./bundle/` → clean (exit 0, no shadowing/unused on the new `_, _, ent, err :=` L1 bindings). `go vet ./...` shows only the pre-existing, unrelated notes (`bspline_test.go` unkeyed fields; `testing.ArtifactDir requires go1.26` across `bspline/`, `gui/op/`, `backup/`, `engrave/`) — confirmed all 38 of these appear identically on pristine `3a23dbb`. Matches the plan's "modulo the pre-existing unrelated note."

### Mandate 4 — Scope / TinyGo

- **Firmware-only:** all 9 changes in `bundle/` + `gui/`; nothing outside; no `me`/CLI/`me-preview`/schema/NDEF/codec/new-program surface. Disjoint from Track B (`slip39/`, `seedxor/`, `gui/bip85.go`, `gui/codex32_polish.go` all untouched).
- **`wipeBytes` called, not edited** (`gui/slip39_polish.go:344`, plain zeroing loop, same package).
- **TinyGo-safe by inspection:** no build tags on the touched files; the diff introduces no reflection/goroutine/generics/unsafe — only a loop reorder, two extra `int` returns, two pure helpers, string constants, and `wipeBytes` calls. `tinygo` is NOT available in this env; the plan (Task 7 Step 3) records this and defers the device build to the controller's integration gate — acceptable, not a blocker.

### Mandate 5 — Self-containment + commit discipline

- Each of the 6 commits builds and tests green at that commit (`bundle` + `gui` both `ok` at all 6 — verified by checking out each).
- 6 commits (M1, H2, H1, L1, L2, relabel), each: `-S` (SSH `gpgsig` object present), `-s` (`Signed-off-by` ×6), author `Brian Goss <goss.brian@gmail.com>` ×6, `Co-Authored-By: Claude Opus 4.8 (1M context)` trailer ×6, explicit staged paths, no merge. (Signature shows `N` only because this env has no allowed-signers file to verify against, exactly as the plan's Task 7 Step 4 anticipates.)

---

## Critical
None.

## Important
None.

## Minor
1. **Plan Task 0 vs my reproduction (cosmetic; plan is correct as written).** The plan's Task 0 creates the worktree with `-b feat/fix-verify-cluster` and sets the Brian Goss identity. I created my throwaway review worktree detached and initially had to amend the first commit to carry the correct author. This is an artifact of my review setup, not a plan defect — the plan's Task 0 Step 1–2 produce the right branch and identity. No action needed.
2. **L2 body wording is "Operator key and secret verified" vs the spec sketch's "Operator key + secret verified."** Semantically identical, and the regression test asserts the load-bearing substrings (`"taken as supplied"` present, `"matches the seed"` absent), so the wording is robust. Non-blocking.

## Verified-correct list
- All OLD-side diff anchors match live source at `3a23dbb` exactly (every `Edit` applied without fuzz).
- M1 fixture is a true entropy-identical/language-differ case; `DecodeMS1` is `(prefix, language int, entropy []byte, err error)`.
- `Template` has `.Root/.Policy/.K/.N` (T-H2 end-to-end fields valid); all T-H1 helpers/fixtures exist (`suppliedMultisigMd1`, `abandonAboutMnemonic`, `deriveSingleSigBundle`, `singleSigPath`, `md.ScriptPkh`, `equalStringSlice`, `bundleCard`, `findUserSlot`, `deriveMultisigLeg`, `showNotice`/`uiContains`/`newPlatform`/`runUI`).
- Fan-out greps re-confirmed post-fix: `ms1Entropy` only `bundle/verify.go:83,87`; `md1Gatherer.collected()` at `md1_gather.go:83,147`+`bundle.go:234` (distinct `mk1Gatherer.collected()` at `bundle.go:194`/`mk1_inspect.go` unaffected); `extractSuppliedMd1` now only at the engrave caller `multisig.go:71`; `verifyMultisig` sole non-test caller `multisig_verify.go:118` passes `suppliedMk1`.

## Bottom line
The plan faithfully implements the GREEN spec and all spec-R0 folds. The test-masking defeat — the core of this gate — holds: all three flow-level tests fail-before on `3a23dbb` and pass-after through the named production functions, with the H1 decodable-but-wrong foreign-mk1 case discriminating via the real stub-binding leg (not a trivial decode failure). No unplanned hunks, no scope creep, firmware-only, TinyGo-safe by inspection (device build correctly deferred), commit discipline satisfied, full suite green. **Verdict: GREEN (0 Critical / 0 Important) — cleared for implementation.**

**Fork hygiene:** confirmed left clean on `main @ 3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082`; throwaway worktree `/tmp/r0plan-A` removed + pruned; pre-existing `seedhammer-wt-bip39` worktree untouched.
