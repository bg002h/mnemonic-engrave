# Track B (scrub batch) PLAN — R0 review (round 0) — VERBATIM agent report

**Agent:** `a36883999ee00ebef` (adversarial opus architect; applied all 4 diffs in a throwaway worktree off `3a23dbb`, built/vetted/tested, verified M4 fail-before/pass-after by stripping the line, probed the idx-17 vector + cold-vs-cold vet). **Fork HEAD:** `3a23dbb`. **Plan commit:** `273816a`. **Spec:** `8cd9cd9`. **Date:** 2026-06-20.
**Verdict:** GREEN (0 Critical / 0 Important). 2 advisory Minors (verification-hygiene). Cleared for implementation. Concurrent with Track A.

---

# R0 GATE REVIEW — IMPLEMENTATION_PLAN_seedhammer_scrub_batch_fixes.md (Track B)

**Reviewer:** Adversarial opus architect (R0 plan gate). Applied all 4 diffs in a throwaway worktree off `3a23dbb`; built, vetted, ran tests; verified the M4 fail-before/pass-after by stripping the line; probed the idx-17 vector structure and the before/after vet delta.
**Fork HEAD:** `3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082` (`bg002h/seedhammer` `main`) — verified live.
**Plan commit:** `273816a`. **Spec:** `8cd9cd9`. **Spec-R0 folds:** `seedhammer-scrub-batch-spec-R0-round0.md`.
**Verdict:** **GREEN — 0 Critical / 0 Important.** (2 Minors, both advisory.)

---

## Mandate 1 — Apply all 4 diffs; build, vet, test; M4 fail-before/pass-after; seam-free guards load-bearing

I applied each diff exactly as the plan's Old→New specifies (every Old block matched live source verbatim; every cited line number was correct).

**M4 is a TRUE fail-before/pass-after.** With `defer pkey.Zero()` temporarily stripped (only that line):
```
=== RUN   TestDeriveBip85Child_ScrubsPkey
    bip85_test.go:526: pkey.Key not zeroed after deriveBip85Child returned (M4: missing defer pkey.Zero())
--- FAIL: TestDeriveBip85Child_ScrubsPkey (0.00s)
```
With the line restored:
```
=== RUN   TestDeriveBip85Child_ScrubsPkey
--- PASS: TestDeriveBip85Child_ScrubsPkey (0.00s)
```
The hook is test-only and nil/no-op in production: `bip85PkeyHook` is declared `var bip85PkeyHook func(pkey *btcec.PrivateKey)` (zero value nil), guarded by `if bip85PkeyHook != nil`, and is only ever assigned inside the test (reset to nil via `defer`). It mirrors the sanctioned `bip85SeedHook` exactly. The canonical child string the test asserts (`"prosper short ramp prepare exchange stove life snack client enough purpose fold"`) matches the existing `TestDeriveBip85Child_AbandonGoldens` 12-word golden — so PASS-after also confirms no behavior change. NO production behavior change.

**M2/M3/L1 seam-free guards are load-bearing and correct.** I independently probed the idx-17 vector: `parsed[0]`=GroupIndex 3/MemberThreshold 2, `parsed[1]`=GroupIndex 1/MemberThreshold 1 (no member digest), `parsed[2]`=GroupIndex 3/MemberThreshold 2 — exactly as the plan claims. The three perturbations yield the asserted sentinels:
- path(a) `pa[0].Value[0]^=0xff` → `errDigestVerificationFailed` (member-layer fail, `:103`)
- path(b) only `parsed[1]` → `errInsufficientShares` (`:108`)
- path(c) `pc[1].Value[0]^=0xff` → `errDigestVerificationFailed` (group-layer fail, `:116`)

`TestCombineErrorPathSentinels`, `TestCombineScrubNoCorruption`, `TestConfirmCodex32Flow_ShowSecretGate` all PASS both before and after their fixes (correct: they are regression+convention guards, green-before by design). The plan states this posture plainly in every task preamble (Minor-2 fold) and introduces NO new seam for these three — confirmed: the only new symbol anywhere is `bip85PkeyHook`. M3's `TestCombineNoCallerMutation` (re-derives `parts[0].Entropy()` post-Combine and asserts unchanged) is the strong proof that wiping `e0`/`e` cannot corrupt the caller's mnemonic — it passes.

## Mandate 2 — Each fix correct + complete on every leak path

- **M2 defer:** registered at line 81 right after `groupShares` (line 80) and `var ems []byte` (line 81) are both in scope; the closure reads them at defer-execution time. Fires on the success return AND all three error returns (`:103`/`:108`/`:116`) — and, as a pure-additive bonus, the three intra-loop returns (`errMemberThresholdMismatch`/`errDuplicateMemberIndex`/`errInsufficientShares`) now wipe whatever was already appended. `ems` is nil on paths a/b (`wipe(nil)` no-op). The success-path scrub loop + `wipe(ems)` correctly removed (defer is now the single site). `wipe(d)` added to the digest-fail branch only; success-path `:142 wipe(d)` left intact (Q2). `var ems []byte` hoisted; `ems, err := recoverSecret(...)` stays `:=` (line 126) — compiles (Minor-1 satisfied).
- **M3:** `e0 := parts[0].Entropy(); out := append([]byte(nil), e0...); wipe(e0)` — `wipe(e0)` before the `interopLen` check, so the bad-length return also leaves `e0` wiped (Q3); `out` is a distinct allocation (`append([]byte(nil),…)`), confirmed safe. In-loop `wipe(e)` on the success path (after XOR) AND before the `errMismatchedLengths` return — explicit, not defer-in-loop.
- **M4:** `defer pkey.Zero()` immediately after `priv := pkey.Serialize()` (line 110), where `pkey` is guaranteed non-nil (the `err != nil` branch at `:102-105` already returned). Covers the success return and the `entLen` guard.
- **L1:** `wipeBytes(ent)` after the single probe use. Confirmed against `codex32/mspayload.go:34` that `ent` is `data[1:]`/`data[2:]` — a subslice; `confirmCodex32Flow` holds no handle to the whole `data` buffer, so wiping the subslice is the correct/complete scope (Minor-3). nil on error → no-op.

No missed path; no non-additive change.

## Mandate 3 — No public signature/control-flow change; no helper edit; one new symbol; disjoint files

The full production diff is byte-for-byte the plan's New text. No public output/signature/return/control-flow change (the M4 hook is the only test-only addition). No `wipeBytes`/`slip39.wipe`/`seedxor.wipe` helper edit. `bip85PkeyHook` is the ONLY new symbol. The 8 changed files are exactly: `slip39/combine.go(+test)`, `seedxor/seedxor.go(+test)`, `gui/bip85.go(+test)`, `gui/codex32_polish.go(+test)` — pairwise-disjoint and disjoint from Track A. Forbidden-file check (helpers `slip39/feistel.go`, `gui/slip39_polish.go`; Track-A `singlesig_verify.go`/`multisig_verify.go`/`multisig_supply.go`/`bundle/verify.go`/`md1_gather.go`): none touched.

## Mandate 4 — Build / vet / no-regression

- `go build ./...` — clean.
- `go test ./...` — 38 packages ok, 0 FAIL. Existing `slip39`/`seedxor`/`gui` suites pass unchanged; the 4 new tests green.
- **Vet:** `go vet ./slip39/ ./seedxor/ ./gui/` is clean (0 findings) in BOTH the clean `3a23dbb` checkout and the patched worktree. This diff introduces ZERO new vet findings. The plan's noted pre-existing `gui/op/draw_test.go:176` `testing.ArtifactDir` note pre-exists at clean `3a23dbb` (confirmed via `go vet ./...` on the untouched primary checkout) — it lives in package `gui/op`, not `gui`, so `go vet ./gui/` never surfaces it. I also found three further PRE-EXISTING notes the plan does not mention (`backup/backup_test.go:389`, `engrave/engrave_test.go:167/187`, and 33 `bspline/bspline_test.go` "unkeyed fields") — all go1.25/go1.26 toolchain-version artifacts in non-Track-B files, confirmed present cold at clean `3a23dbb`. None introduced by this diff.
- TinyGo-safe by inspection (`defer`/closures/`defer`-method-call/in-loop `wipe`); device build correctly deferred to the integration gate.

## Mandate 5 — Self-contained tasks; commit discipline

4 commits, in order M2→M3→M4→L1, each self-contained (test+fix together), author `Brian Goss <goss.brian@gmail.com>`, with a `Signed-off-by` (DCO `-s`, count 4) and `Co-Authored-By` trailer (count 4), explicit `git add` paths, no merge. (I used `-c commit.gpgsign=false` for the throwaway review since I do not hold the user's SSH signing key; the plan's `-S` invocation against the repo's `gpg.format=ssh` config is correct — signature production is the one thing I could not exercise, and it is config-correct.)

---

## Critical findings
None.

## Important findings
None.

## Minor findings (advisory; do not block GREEN)

1. **Task 5's vet filter is too narrow.** The plan's `go vet ./... 2>&1 | grep -v 'gui/op/draw_test.go'` paired with "If ANY other vet finding appears, STOP" would falsely trip on the OTHER pre-existing notes (`backup`, `engrave`, and especially the 33 `bspline/bspline_test.go` unkeyed-field notes) that surface on a cold `go vet ./...`. These are all pre-existing go1.25/1.26 artifacts in non-Track-B files. Recommend the implementer either (a) scope the vet to the touched packages — `go vet ./slip39/ ./seedxor/ ./gui/` (clean) — or (b) diff `go vet ./...` against a cold clean-`3a23dbb` baseline (which I did: zero new findings), rather than relying on the single-token grep. Pure verification hygiene; the underlying fixes are clean.

2. **Vet-cache can mask a fair comparison.** When verifying "no new vet finding," compare cold-cache to cold-cache (or vet the specific touched packages). A warm primary-checkout cache vs. a cold worktree produced a spurious 33-line `bspline` delta that, on cold-vs-cold re-run, vanished. Advisory note for the exec-review gate, not a code issue.

## Verified-correct list
- M4 genuine fail-before (`pkey.Key not zeroed…`) / pass-after, via the nil-in-production, in-file-precedented `bip85PkeyHook`; canonical child unchanged.
- idx-17 structure and all three M2 error-path sentinels confirmed by independent probe.
- M2 defer fires on every return (success + 3 named + 3 bonus intra-loop), `var ems` hoisted, `:=` preserved (compiles), redundant success scrub removed, `wipe(d)` digest-fail-only.
- M3 `e0` wiped before `interopLen` (bad-length safe; `out` distinct alloc), in-loop `wipe(e)` on success + mismatch; `TestCombineNoCallerMutation` proves no caller corruption.
- L1 `wipeBytes(ent)` scopes exactly to the `data[1:]`/`data[2:]` subslice; nil-safe; showSecret gate unchanged.
- Diff matches plan New text verbatim; only new symbol is `bip85PkeyHook`; no helper/Track-A/forbidden file touched.
- `go build ./...` clean; `go test ./...` 38 ok / 0 FAIL; Track-B packages vet clean cold in both trees (zero new findings).
- Commit discipline: 4 ordered self-contained commits, DCO `-s` ×4, Co-Authored-By ×4, author Brian Goss, explicit paths, no merge.

## Bottom line
The plan faithfully implements the GREEN spec and all three folded spec-R0 Minors (Minor-1 `ems`/`:=`; Minor-2 guard posture stated per task; Minor-3 L1 subslice scope) plus Q1/Q2/Q3 rulings. All four fixes are strictly additive, correct, and complete across every leak path; M4 is a true fail-before/pass-after via a sanctioned test-only seam; the other three are correctly-scoped regression+convention guards. Build/vet/test are green with zero new vet findings. The two Minors are verification-hygiene notes for the implementer/exec-review gate, neither blocks the gate.

**VERDICT: GREEN — 0 Critical / 0 Important.** Track B is cleared for implementation.

**Fork state:** clean on `main @ 3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082`; `/tmp/r0plan-B` worktree removed and pruned; temp files cleaned. The pre-existing `seedhammer-wt-bip39` and the Track-A `/tmp/r0plan-A` worktrees left untouched.
