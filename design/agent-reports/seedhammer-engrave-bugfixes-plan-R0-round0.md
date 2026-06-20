# Engrave-bugfixes PLAN — R0 review (round 0) — VERBATIM agent report

**Agent:** `a2e8d8ddfe491c06b` (adversarial opus architect; applied every diff and reproduced every FAIL→PASS in a throwaway worktree off `f907eea`). **Fork HEAD:** `f907eea` (main, clean). **Plan commit:** `c4ce050`. **Date:** 2026-06-19.
**Verdict:** GREEN (0 Critical / 0 Important). 2 non-blocking Minors (implementer guidance). Cleared for single-implementer TDD, then the mandatory whole-diff exec review. Persisted per the gate discipline.

---

# R0 GATE REVIEW — IMPLEMENTATION_PLAN_seedhammer_engrave_bugfixes.md

**Reviewer:** adversarial opus architect (R0 plan gate; applied every diff and reproduced every FAIL→PASS in a throwaway worktree off `f907eea`)
**Fork HEAD:** `f907eea8fa4cbbe6b90ed0a6da85a1e5f9783b8f` (verified `main`, clean) · **Go:** go1.26.4 · **module declares** go 1.25.10
**Plan commit:** `c4ce050` (verified present in mnemonic-engrave: "design(engrave-fix): implementation plan (3 bugs, TDD) for plan R0")
**Verdict: GREEN — 0 Critical / 0 Important** (2 Minor, non-blocking)

Every snippet in the plan compiled and ran as written. Every per-bug test FAILS on `f907eea` with the predicted message class and PASSES after its fix; all committed ≤24/QR goldens stay byte-identical with zero churn; the full suite, build, and vet are green (modulo the pre-existing go1.25 `ArtifactDir` notice, confirmed to pre-exist).

---

## MANDATE 1 — Spec coverage

Every spec requirement maps to a plan task, with no deviation from the GREEN spec or its ratified decisions:

| Spec requirement | Plan task | Status |
|---|---|---|
| BUG-1 counter-invariant assertion | Task 1.2 assertion (a) `sp.progress > totalTicks` | covered |
| BUG-1 safe-point reference (not-yet-reached) | Task 1.2 assertion (b) + final | covered, load-bearing (verified below) |
| BUG-1 break-based not saturating | Task 1.3 `if s.progress < k.T { break }` | covered, matches R0 ratification |
| BUG-1 existing `TestSafePointer` green | Task 1.4 | covered |
| BUG-2 panic→error dim 37 AND 41 | Task 2.2 `TestEngraveSeedStringTooLong` {dim37,dim41} | covered |
| BUG-2 happy path | Task 2.2 `TestEngraveSeedStringHappy` (dim 33) | covered |
| BUG-2 no `bitmapForQRStatic`→error conversion | Task 2.3 keeps `panic` as unreachable assertion | covered, matches R0 ratification |
| BUG-3 N=33 no-overlap | Task 3.2 `TestSLIP39LargeGeometry` | covered |
| BUG-3 pinned pfsN=24696 | Task 3.2 geometry assertion | covered |
| BUG-3 byte-identical ≤24/QR | Task 3.5 zero-churn check | covered, verified |
| BUG-3 N=23 fresh pin | Task 3.5 `TestSLIP39_23WordPin` | covered |
| BUG-3 N=30 new-path pin | Task 3.5 `TestSLIP39_30WordInBounds` | covered |
| BUG-3 single `N>24` predicate, ≤24 path untouched | Task 3.3 single `if n > largeN` (largeN=24) | covered, matches R0 ratification |

No gap, no deviation. Scope is firmware-only (4 source/test files + 2 new goldens), no me/CLI/schema/docs/codec surface — confirmed by `git status` (see Mandate 4).

## MANDATE 2 — Apply diffs and run FAIL→PASS independently

**BUG-1** — test against unmodified `f907eea`:
```
--- FAIL: TestSafePointerNoUnderflow (0.00s)
    engrave_test.go:410: step 0: sp.progress=18446744073709551316 exceeds totalTicks=300 (underflow wrap)
```
`18446744073709551316` = 2^64 − 300 ≈ 1.84e19 — exactly the predicted wrap. After the 1-line guard fix: `TestSafePointerNoUnderflow` and `TestSafePointer` both `--- PASS`; full engrave package `ok`.

**BUG-2** — too-long test against `f907eea`:
```
--- FAIL: TestEngraveSeedStringTooLong/dim37: dim37: EngraveSeedString panicked; want a returned error
--- FAIL: TestEngraveSeedStringTooLong/dim41: dim41: EngraveSeedString panicked; want a returned error
--- PASS: TestEngraveSeedStringHappy
```
After fix (errors import + `EngraveSeedString` guard + `ConstantQR` early check): all `--- PASS` (dim37, dim41, Happy); `TestConstantQR` not regressed (`ok`).

**BUG-3** — Task 3.4 fail-on-buggy dance reproduced: generated golden under fixed code (5029 bytes), reverted `backup.go` to `f907eea`:
```
--- FAIL: TestSLIP39Large (0.01s)
    backup_test.go:493: spline lengths 16753, 16277, with 16276/16277 knot mismatches
```
**16276/16277 knot mismatches — matches the plan's ~16276 claim exactly.** Restored fixed code → `ok`. The revert/restore dance is safe and `backup.go` was confirmed byte-restored to the fixed version after each revert.

**No regression / full run:** all committed ≤24 + QR goldens (`TestSeed` 24/12, `TestSLIP39` 20-word, `TestCodex32`, `TestText`, `TestConstant*Timing`) PASS with **zero golden churn** — `git status backup/testdata/ engrave/testdata/` reported NO modified `.bin`. Final `go test -count=1 ./engrave/... ./backup/...`:
```
ok  	seedhammer.com/engrave	0.468s
ok  	seedhammer.com/backup	0.167s
```
`go build ./...` clean (exit 0). `go vet ./engrave/ ./backup/`:
```
engrave/engrave_test.go:167:50: testing.ArtifactDir requires go1.26 or later (file is go1.25)
engrave/engrave_test.go:187:48: testing.ArtifactDir requires go1.26 or later (file is go1.25)
backup/backup_test.go:389:48: testing.ArtifactDir requires go1.26 or later (file is go1.25)
```
**The `ArtifactDir`/go1.25 notice truly pre-exists on `f907eea`** (baseline at start showed the identical three notices; backup shifted 388→389 only because the new `slices` import pushed `compareGolden` down one line). My diff adds NO `t.ArtifactDir()` calls (`git diff | grep ArtifactDir` empty). No new vet findings. `go vet ./slip39/` clean (exit 0).

## MANDATE 3 — Adversarial scrutiny of test design

**BUG-3 golden-oracle gap (the central concern):**
- (a) `TestSLIP39Large` IS a genuine production-correctness gate: byte-exact, generated from production `frontSideSeed`, and FAILS 16276/16277 knots on buggy code. Confirmed.
- (b) No-overlap is independently meaningful: I measured the ACTUAL emitted production plan (not the test oracle) — N=33 bbox `x[0,71.87] y[0,81.97]mm`, col2Bot ≤ col1Bot = true; N=30 bbox `x[0,73.61] y[0,79.92]mm`, col2Bot ≤ col1Bot = true. Both within `[0,85]` on both axes.
- (c) The ~16276 claim is TRUE (16276/16277, observed directly).
- N=33 re-derived independently in Go: pfsN = `16*26240/17 = 24696` (= 3.8588mm, remainder 8), col1Bot=481916 (75.30mm), col2Bot=457220 (71.44mm), gap=24696 (3.86mm), no-overlap & in-bounds — matches the plan table to the unit.

**Verdict on the oracle duplication:** `seedLayout` duplicates the production formula, so `TestSLIP39LargeGeometry` alone cannot catch a wrong production formula — but it is *not* the gate; the byte-exact golden is. A wrong production formula CANNOT ship green because three independent sources must agree: (1) my hand re-derivation, (2) the test oracle, (3) the actual emitted production knots (my probe). Plus I confirmed the **N=23 golden is byte-identical between `f907eea` and the fixed code** (regenerated under both, `cmp` IDENTICAL) — proving the ≤24 path is genuinely untouched, not merely self-consistent. **This is an acceptable design, not an Important defect.** (Minor-1 suggests hardening it further.)

**BUG-1 test semantics:** `SafePointer.Progress` takes an INCREMENT (`s.progress += p` internally) — confirmed by trace (step 1: d=50 → progress=50), so the test's `completed += d` bookkeeping is correct. Trace under fixed code: steps 0-4 (completed 0,50,100,149,150 < tripleEnd=250) all have `safePoint={0,0}` (NOT P) — so assertion (b)'s `sp.safePoint == P` guard is genuinely exercised and would fire on early advance; it is **load-bearing, not vacuous**. At step 5 (completed=250) the clamped triple retires and `safePoint={10,10}=P`. The trailing knot does NOT reset the safe point (after retire: histLen=1, safePoint stays P). The constructed triple `{P,50,true}×3` satisfies line 1479's strict `k0.Ctrl==k1.Ctrl && k1==k2` (full Knot equality) — verified.

**BUG-2:** All three codex32 strings valid (`codex32.New` accepts them). Under `qr.M` after `ToUpper`: 93-char→dim 37, 127-char→dim 41, 74-char happy→dim 33. **Dim 33 is the exact correct cutoff** (`>33` blocks 37/41, allows 33). These are BIP-93 codex32 (`ms` HRP) consumed at engrave time — the plan introduces NO constellation-codec coupling (no `md`/`mk`/`ms1` edits). BIP-39 QR plates are unaffected: 12-word seedqr→dim 25, 24-word→dim 29, both ≤33 so the new `ConstantQR` check never rejects a legitimate plate. `FuzzConstantQR` (40-byte cap, Q/L) seeds + an 8s fuzz run (85k execs) found no new failures.

## MANDATE 4 — Scope, compile-fidelity, commit hygiene

- **Compile fidelity:** every block compiled and ran. Symbols verified: `bezier.Pt(x,y int) Point` (319), `Knot.T uint`, `bezier.Point` comparable struct, `compareGolden(t testing.TB, name string, plan engrave.Engraving)` (real signature matches), `engrave.PlanEngraving(conf, side)`, `slices.Collect`, `slices` import added to `backup_test.go`. No symbol drift.
- **No signature changes:** `ConstantQR` keeps `(*ConstantQRCmd, error)` (early `return nil, error` only); `SafePointer.Progress(p uint)` unchanged (guard condition only); `ConstantQRCmd.Engrave` untouched; `bitmapForQRStatic` unchanged (`panic` left as unreachable assertion). Confirmed from the full diff.
- **Scope:** `git status` shows exactly `M backup/backup.go, backup/backup_test.go, engrave/engrave.go, engrave/engrave_test.go` + `?? backup/testdata/slip39-{23,33}-words.bin`. No me/CLI/schema/docs/codec surface.
- **Commit hygiene:** all three staging blocks use explicit paths (no `git add -A`); `-S -s`, author Brian Goss, Co-Authored-By trailer present. Sound.
- **Task 3.4 dance safe:** confirmed (`cp` save/restore + `git show f907eea:backup/backup.go` revert correctly restores).

## Critical
**None.**

## Important
**None.**

## Minor (non-blocking)
1. **Geometry oracle could be hardened (optional).** `seedLayout` duplicates the production formula, so `TestSLIP39LargeGeometry` cannot independently catch a wrong production formula. The byte-exact golden + the N=23 byte-identical cross-check already close this hole adequately. If the implementer wants belt-and-suspenders, `TestSLIP39LargeGeometry` could additionally measure col1/col2 y-ranges from the *emitted* `PlanEngraving` (as my probe did, x-banded at 38mm) rather than only from the test-local oracle. Not required.
2. **Staging note for the implementer.** `backup/backup_test.go` is staged in both the BUG-2 and BUG-3 commits (different hunks). This is fine *only if* tasks run in strict order (BUG-2 committed before BUG-3 tests are written, per Task sequence). If the implementer writes all tests up front, the BUG-2 commit would capture BUG-3 test code too. Follow the plan's per-bug task order to keep commits clean.

## Verified-correct (independently ran and confirmed)
- Fork HEAD `f907eea`, clean; baseline `go test`/`go vet` captured (notices pre-exist).
- BUG-1: FAILS on `f907eea` (`sp.progress=1.84e19 > totalTicks=300` at step 0); PASSES after 1-line guard; `TestSafePointer` stays green; safe-point trace proves assertion (b) load-bearing (safePoint zero at completed<250, =P at 250, trailing knot doesn't reset).
- BUG-2: dim37/dim41 panic on `f907eea` (caught→fail), happy dim33 passes; after fix all pass; cutoff `>33` exact (33 allowed, 37/41 blocked); BIP-39 QR dims 25/29 unaffected; fuzz clean.
- BUG-3: golden generated (5029B); fail-on-buggy = 16276/16277 mismatches; zero churn on all ≤24/QR goldens (no `.bin` modified on disk); N=23 golden byte-identical `f907eea` vs fixed; N=33 numbers re-derived (pfsN=24696, 75.30/71.44mm, gap 3.86mm); production emitted plan non-overlapping & in-bounds for N=30 and N=33.
- All three fixes together: `go test -count=1` both packages `ok`; `go build ./...` clean; `go vet` only pre-existing notices; `go vet ./slip39/` clean.
- No signature changes; scope firmware-only; explicit-path staging; Task 3.4 revert/restore safe.

## Bottom line
The plan faithfully and minimally implements the GREEN spec. Every diff compiles and runs as the plan claims; every per-bug test genuinely fails on `f907eea` with the predicted failure and passes after its fix; the ≤24/QR goldens are provably byte-identical (zero churn + N=23 cross-check against `f907eea`); the BUG-3 golden is a real production gate (16276/16277 mismatches on buggy code), not a self-consistent stub; and the assertions I scrutinized for vacuity (BUG-1 safe-point reference) are load-bearing. The 2 Minors are optional polish. **Verdict: GREEN (0 Critical / 0 Important). Nothing must change to reach GREEN — implementation may proceed** under TDD + single-subagent-in-worktree, followed by the mandatory post-implementation execution review (which must treat the N=27/30 layout change as expected).

**Fork state confirmed at end:** `/scratch/code/shibboleth/seedhammer` is on `main @ f907eea8fa4cbbe6b90ed0a6da85a1e5f9783b8f`, `git status --porcelain` empty (clean). Throwaway worktree `/tmp/r0-plan-engrave` removed and pruned; `git worktree list` shows only fork main + the pre-existing `seedhammer-wt-bip39` (untouched). All `/tmp` probe files removed.
