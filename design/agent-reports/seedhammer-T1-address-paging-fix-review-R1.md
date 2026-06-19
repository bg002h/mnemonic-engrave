<!--
Persisted verbatim. opus-architect R1 gate of the T1 paging-fix amendment after folding R0
(SPEC §1/§4.2/§6 + PLAN Task 1 @ 869ab71). Reviewer agentId a3aec28aeb663026e. Verdict: GREEN 0C/0I.
The reviewer rebuilt + ran the amended code in a throwaway copy: the corrected
TestDescriptorAddressFlowNoSkippedIndices (observe-then-advance, click inside the loop) PASSES on
both fixtures on the amended measure-and-advance impl, and FAILS on the OLD fixed-page-5 impl for
the RIGHT reasons (mid-range skips wpkh idx4/5, P2WSH idx4/5/6; index 0 now correctly observed) —
so it genuinely discriminates fix-from-bug. TestAllocs 0; full suite/vet/gofmt clean; Task 2
(gui.go) byte-identical to 979aadd; SPEC §6 prose + §1 example fixed; no stale fixed-5/addrPageSize
references; no drift. Disposition: GREEN — re-implement the amended descriptorAddressFlow + corrected
test on the branch, then the canonical whole-diff exec review on the corrected diff, then merge.
The text below is the agent's report exactly as returned; do not edit.
-->

# R1 GATE REVIEW — T1 paging fix (spec+plan amendment)

**Scope:** Focused, adversarial R1 re-review confirming the R0 fold (commit `869ab71`) closes the sole R0 blocker (IMPORTANT-1: regression test sequencing) and the MINOR-1 (SPEC §1 stale example), with no drift. Built + ran the amended `descriptorAddressFlow` (PLAN Task 1 Step 3) and the amended `TestDescriptorAddressFlowNoSkippedIndices` in a throwaway copy of worktree `seedhammer-wt-t1-address` with `/home/bcg/.local/go/bin/go` (go1.26.4). No commit/merge/push; throwaway deleted, worktree left at `979aadd`, clean.

## Verification Results

### 1. Test fold is correct — CONFIRMED
The amended `TestDescriptorAddressFlowNoSkippedIndices` (PLAN lines 148-183) now **observes then advances**: the pre-loop `click(&ctx.Router, Button3 ×8)` is **removed**; inside the per-frame loop the body pulls a frame, scans all 8 target addresses into `seen`, *then* `click(&ctx.Router, Button3)` advances one page. So **index 0 is observed on frame 0 before any page-forward**. The now-incorrect "Up to 8 page-forwards" comment is removed (R0 required this). It still loops over **both** fixtures (`descWPKH` single-sig + `descCustomChildren` long-address P2WSH) and asserts every `address.Receive(d,i)` for i∈0..7 appears in some rendered frame. The fold diff (`869ab71`) is exactly this and nothing else in the PLAN.

### 2. Build + run — full output
Applied the amended `descriptorAddressFlow` + amended test to the throwaway (impl diff and test diff both match the committed PLAN amendment byte-for-byte):
- `go build ./gui/... ./address/...` → **OK** (exit 0); `gofmt -l` on both files → **clean**.
- `go test -run TestDescriptorAddressFlow -v ./gui/` → **all 4 PASS**: `RendersReceive`, `ToggleChange`, `BackExits`, and **`NoSkippedIndices` PASS** (covers both fixtures in one run, 0.17s).
- `go test -run TestAllocs -v ./gui/` → **PASS (0 allocs)**.
- `go test ./gui/... ./address/...` → **all green** (`ok seedhammer.com/gui 6.885s`, `ok seedhammer.com/address`, op/saver/text/widget all ok).
- `go vet ./gui/... ./address/...` → clean except the **pre-existing** `gui/op/draw_test.go:176` go1.26 `ArtifactDir` note (not ours; R0 already flagged it).

**Discrimination independently re-confirmed (not just relied on from R0):** ran the corrected test against the OLD committed fixed-page-5 `address_polish.go` → **FAIL**, and crucially the failures are genuine mid-range skips, NOT the index-0 false-fail the R0 finding was about: wpkh skips idx 4/5 (`bc1q7c…`, `bc1q79…`), P2WSH skips idx 4/5/6 — **index 0 is absent from the failure list, proving observe-before-advance now sees the entry page**. Restored amended impl → PASS. The test genuinely exercises the regression and discriminates fix-from-bug.

### 3. Drift — none
- **SPEC §6** now reads "observe each rendered page, then advance one page (Button3) — observe-before-advance, so the entry page (index 0) is seen before any page-forward," with an explicit "pre-queueing … would page over index 0 and fail on *correct* code" caveat. **Matches the implementation and the test.** ✓
- **SPEC §1** example fixed: "e.g. show indices 0..4, page forward" → "show the addresses that fit the screen, page forward by the count shown — §4.2." No fixed-5. ✓
- Grep across SPEC+PLAN for `indices 0..4` / `show indices` / fixed-5 / `Button3, Button3` / `addrPageSize` / `addrMaxStart` → **no stale matches** (the only hit is the legitimate `addrMaxIndex = 49 // show indices 0..49` cap comment). No new contradiction introduced.
- **Task 2 untouched:** throwaway `gui/gui.go` is **byte-identical** to committed `979aadd`; both affordance tests (`TestDescriptorConfirmAddressAffordance`, `…Unsupported`) still PASS. The fold touched only the design docs + the Task-1 test/impl prose.

## Findings

**C/I:** None. The single R0 IMPORTANT (test fails on correct code, doesn't discriminate) is fully closed: the corrected test passes on the amended measure-and-advance impl, fails on the old buggy impl for the right reasons, and sees index 0. MINOR-1 (SPEC §1 stale example) is closed.

**MINOR:** None.

All prior R0-verified properties remain intact and were re-exercised here: measure-and-advance gap-free paging (Measure==Labelw height, index-0 always shown, cap at 49, no re-entrancy), TestAllocs 0, Task 2 byte-identical. The fold is clean with no drift.

**GREEN — 0 Critical / 0 Important.**
