<!--
Persisted verbatim. opus-architect combined R0 gate of the T1 measure-and-advance paging amendment
(SPEC §4.2/§6 + PLAN Task 1 @ 92ff31a). Reviewer agentId ad6ba7cc60fbc9091. Verdict: NOT GREEN
0C/1I (1 minor). The reviewer built + ran the amended code in a throwaway copy of the worktree.
The IMPLEMENTATION LOGIC is verified correct + gap-free: measure-and-advance pages
start..start+shown-1 then start+=shown (no skip); index 0 always included (shown≥1, no infinite
loop); Measure height == Labelw height (same Layout/MaxWidth/Style → no residual clipping); cap
addrMaxIndex=49 + the start+shown<=49 guard correct; idx 49 reachable; no re-entrancy/hang/mutation;
Task 2 (Confirm) byte-identical/untouched; TestAllocs still 0. The single blocker (CRITICAL-1, rated
Important): the regression test TestDescriptorAddressFlowNoSkippedIndices pre-queued all 8 Button3
clicks BEFORE pulling any frame, so the entry page (idx 0) is advanced past before frame 0 renders →
the test FAILS on correct code and doesn't discriminate fix-from-bug. Reviewer's fix (verified: passes
on amended code, fails on old buggy code): move click(Button3) INSIDE the frame loop, after observing
each frame. MINOR-1: stale "indices 0..4" example in SPEC §1. Disposition: folded the test sequencing
(plan + SPEC §6 prose) + MINOR-1 (SPEC §1); re-dispatching R1. The text below is the agent's report
exactly as returned; do not edit.
-->

# R0 GATE REVIEW — T1 paging fix (spec+plan amendment)

**Scope:** Adversarial, read-only combined R0 over the measure-and-advance paging amendment (SPEC §4.2/§6 committed `92ff31a`; PLAN Task 1 rewrite + `TestDescriptorAddressFlowNoSkippedIndices`). Task 2 (`DescriptorScreen.Confirm` affordance) verified untouched. Built and ran in a throwaway copy of worktree `seedhammer-wt-t1-address` with `/home/bcg/.local/go/bin/go` (go1.26.4); no commit/merge/push; throwaway and worktree both clean afterward (worktree HEAD still `979aadd`).

## Verification Results

### 1. The fix is correct + gap-free — CONFIRMED (logic) but the regression test does NOT exercise it (see CRITICAL-1)

The measure-and-advance *implementation logic* is correct and gap-free. I traced and empirically confirmed it under realistic one-click-per-frame interaction at the 240×240 test display:
- **wpkh** (single-sig): pages `[0,1] → [2,3] → [4,5] → [6,7] → [8,9] → [10,11]` — exactly 2 consecutive lines/page, no gaps.
- **wsh P2WSH 62-char** (`descCustomChildren`): pages `[0] → [1] → [2] → [3] → …` — exactly 1 line/page (the long address wraps to 77px, two lines exceed the ~152px content area), consecutive, no gaps.

Page N shows `start..start+shown-1` (all fit); page-forward sets `start += shown` so page N+1 begins at the immediate successor. Index 0 is always included (`if i > 0 && …break` at the fit check), guaranteeing `shown ≥ 1` (no no-advance/infinite loop).

**Measure == Labelw height: CONFIRMED.** `text.Style.Measure` (`gui/text/text.go:56-78`) and `widget.Labelwf` (`gui/widget/label.go:24-57`) both drive the identical `text.Layout.Next` wrapper with the same `MaxWidth`/`Style`/text; both compute height as `ascent.Ceil() + numNewlines*LineHeight() + descent.Ceil()`. `Measure` forcing `AlignStart` only changes horizontal `l.dot`, never the newline count. Empirically: `Measure().Y == Labelw().Y == 77` for the 65-char line at width 224 (idx 0..3). So a line admitted by `recompute` renders at exactly that height — **no residual clipping**.

**Cap arithmetic: CONFIRMED.** `recompute` loop bound `start+i <= addrMaxIndex(49)` and page-forward guard `start+shown <= 49` are correct. Empirical worst-case (1 line/page): index 49 IS reachable, every index 0..49 viewable, paging terminates at the cap (no advance past 49, no infinite loop). Near-cap reasoning holds: when the guard blocks an advance, the current page already includes index 49.

### 2. Build + run — full output

- `go build ./gui/... ./address/...` → OK.
- `go test ./gui/... ./address/...` (with the PLAN's verbatim test) → **FAIL**: `TestDescriptorAddressFlowNoSkippedIndices` fails on BOTH `descWPKH` (idx 0, 1 "never viewable") AND `descCustomChildren` (idx 0). All other tests pass.
- After correcting the test's click sequencing (one `click(Button3)` after each observed frame, not all 8 pre-queued): `go test ./gui/... ./address/...` → **all PASS** (`ok seedhammer.com/gui 8.866s`, `ok seedhammer.com/address`); `go test -run TestDescriptorAddressFlow -v` → all 4 PASS including `NoSkippedIndices` on both fixtures; `go test -run TestAllocs ./gui/ -v` → **PASS (0 allocs)**; `gofmt -l` clean; `go vet` clean except the pre-existing `gui/op/draw_test.go:176` go1.26 note (not ours, as the plan states).

### 3. The no-skip test does NOT genuinely test the bug — it fails on correct code (CRITICAL-1)

The PLAN's `TestDescriptorAddressFlowNoSkippedIndices` pre-queues all 8 Button3 clicks via a single `click(&ctx.Router, Button3×8)` *before* pulling any frame. Because `Clickable.Clicked` (`gui/widget.go:35`) drains queued clicks one press+release pair per frame and the loop advances `start` at the *top* of the body before the `ctx.Frame` render, the entry page (page 0, indices 0/1) is advanced-over before frame 0 is ever yielded. Result: frame 0 already shows `[2,3]` (wpkh) / `[1]` (P2WSH) — **index 0 is never rendered**, so the test fails. It fails identically on the OLD buggy fixed-page-5 code, so it does not discriminate the fix from the bug.

I verified a *correctly-sequenced* version (observe frame → then click one page) PASSES on the amended code and FAILS on the old buggy code (correctly catching wpkh idx 3,4 skipped; P2WSH idx 2,3,4 skipped). So the test *logic* (compare against `address.Receive`, both fixtures, off-screen clipping via `ExtractText`'s `clip.Empty()` drop at `gui/op/op.go:320-323`) is sound; only the event sequencing is wrong. The test harness DisplaySize is **240×240** (`gui_test.go:348` `testDisplayDim=240`), which is genuinely smaller than the real 480×320 and still forces multi-page (long P2WSH = 1/page, wpkh = 2/page), so the fixtures DO need >1 page — the harness is adequate; only the click pattern is broken.

### 4. No re-entrancy / hang / mutation — CONFIRMED
`showError` (`gui/slip39_polish.go:22`) is a self-contained modal (own `for !ctx.Done` loop, returns on dismiss); calling it mid-`recompute` then returning `false`→`return` is non-re-entrant and cannot hang. Back exits; toggle/page terminate; no engrave, no NFC, no descriptor mutation (`Receive`/`Change` are pure public derivations). dims-once is safe (DisplaySize fixed on SH).

### 5. Drift — Task 2 untouched (CONFIRMED); one MINOR stale example
`gui.go` in the amended set is byte-identical to committed `979aadd` (Task 2 genuinely untouched; the only change is `address_polish.go` + the new test). No stale `addrPageSize`/`addrMaxStart`/`cap-50` prescriptions remain — all "fixed count" mentions are correct historical bug-description context. See MINOR-1 for one stale illustrative example in SPEC §1.

## Findings

**CRITICAL-1 — The regression test `TestDescriptorAddressFlowNoSkippedIndices` fails on the correct (amended) implementation.** Location: PLAN Task 1 Step 1 (the `click(&ctx.Router, Button3 ×8)` before the frame loop, with no click inside the loop). A regression test that fails on correct code blocks GREEN and does not discriminate the fix from the bug. **Fix:** move the page-forward inside the per-frame loop so the entry page is observed before any advance — pull/scan a frame, *then* `click(&ctx.Router, Button3)`. Verified: this passes on the amended code and fails (correctly) on the old buggy code. Remove the now-incorrect comment "Up to 8 page-forwards (worst case 1 address/page still reaches idx 7)". (The same broken pattern appears in SPEC §6's prose; ensure the prose matches the per-frame-then-click intent.)

**MINOR-1 — Stale example in SPEC §1.** Line 16: "index paging (e.g. show indices 0..4, page forward)" implies a fixed 5-window, contradicting §4.2's measure-and-advance. Reword to e.g. "index paging (show the addresses that fit, page forward by the count shown)".

No other Critical or Important findings. Build/vet/gofmt/alloc-gate all clean; logic, height-equivalence, cap, termination, and Task-2-untouched all verified.

**NOT GREEN — 0 Critical / 1 Important.**

(Note: CRITICAL-1 is a test-correctness defect — a regression test that fails on the correct implementation — which under the prompt's stated rubric is at minimum Important and arguably Critical, since it both fails the gate and fails to exercise the regression. The implementation logic itself is correct and would pass a properly-sequenced test. Fix the test sequencing (and the matching SPEC §6 prose), re-run `go test -run TestDescriptorAddressFlow -v` to GREEN on both fixtures, then re-dispatch.)
