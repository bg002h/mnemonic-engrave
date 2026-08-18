# S6b P5b implementation report — §5.1 per-direction scroll-arrow predicate

Worktree: `/scratch/code/shibboleth/wt-s6b`, branch `s6b-pre-flash`, on top of P5
(`808e403`, "S6b P5 (5.1/5.1b/5.3): scroll arrows float over the body's fade
zones"). This is a focused follow-up executing the corrected
`SPEC_s6b_pre_flash_cycle.md` §5.1 — not a redesign.

## The defect P5 flagged, and the fix

P5 implemented §5.1 exactly as originally written: one shared visibility
predicate (`scrollArrowsVisible`) gating both arrows together. That predicate
draws the **up arrow at `w.scroll == 0`**, pointing at content that does not
exist above it, and the **down arrow at full scroll**, pointing at content
that does not exist below it — a false claim under R-D, and the same failure
that killed the original arrow proposal. Worse, tapping the up arrow at
`scroll == 0` visibly does nothing (scroll clamps to 0), teaching the operator
the arrows don't work and discrediting the down arrow when it matters.

The spec now reads (§5.1, normative):

```
show UP    iff  w.scroll > 0
show DOWN  iff  bodyClip.Min.Y + scrollFadeDist + bodysz.Y - w.scroll > dims.Y
```

### Change made

`gui/gui.go`:

- `scrollArrowsVisible(bodyClip, bodysz, dims) bool` (one shared predicate) is
  replaced by two functions:
  - `scrollArrowUpVisible(scroll int) bool` → `scroll > 0`. No geometry input,
    so it cannot drift with the `fadeClip` stub.
  - `scrollArrowDownVisible(bodyClip image.Rectangle, bodysz, dims image.Point, scroll int) bool`
    → `bodyClip.Min.Y+scrollFadeDist+bodysz.Y-scroll > dims.Y`. Same formula
    P5 already had, now scroll-aware; still coupled to the stubbed `fadeClip`
    per R-E (comment retained at the call site).
- `Warning.Layout` now computes `showUp`/`showDown` independently and gates
  **both the click handler and the draw, per direction** — exactly as P5
  gated both together:
  - `if showUp && w.arrowUp.Clicked(ctx) { w.scroll -= w.txtclip / 2 }`
  - `if showDown && w.arrowDown.Clicked(ctx) { w.scroll += w.txtclip / 2 }`
  - the arrow-drawing block now builds an `[]op.Op` and appends the top chip
    only `if showUp`, the bottom chip only `if showDown`.
- The **DOWN predicate's panel-based form was not changed**, beyond adding the
  `-scroll` term the spec requires. **Body width is untouched** —
  `warningBodyClip` was not edited.

`gui/scroll_arrows_test.go`:

- Added the extended GATE 5.1 assertions (below), plus `arrowChips` /
  `arrowChipInkPresent` helpers so each chip can be checked independently
  (the old `scrollArrowsDrawnFor` only ever sampled the top chip).
- `scrollArrowsDrawnFor` (used by the pre-existing
  `TestGate51ArrowsDrawnOnlyWhenContentOverflowsThePanel`) now checks **either**
  chip, not just the top — with the predicate split, a fresh `scroll == 0`
  overflowing body draws only the DOWN arrow, so a top-only check would have
  started reporting "no arrow drawn" for a body that correctly shows one.
- `TestGate51VisibilityPredicateFormula` (unit-level formula pin) is renamed
  `TestGate51VisibilityPredicateFormulaDown` and now calls
  `scrollArrowDownVisible(bodyClip, bodysz, dims, 0)` — numerically identical
  to the old shared predicate at `scroll=0`, so its six boundary cases are
  unchanged and still pass. Added `TestGate51VisibilityPredicateFormulaUp` to
  pin `scrollArrowUpVisible`'s trivial formula.
- GATE 5.1b's call site was renamed from `scrollArrowsVisible(bodyClip,
  image.Pt(0, y), dims)` to `scrollArrowDownVisible(bodyClip, image.Pt(0, y),
  dims, 0)` — a pure rename at `scroll=0`, not a loosening; see confirmation
  below that its output is byte-identical to P5's.

## TDD: extended GATE 5.1 assertions, failing before / passing after

Two new integration-level tests were written first, deliberately using only
pre-existing package API (rendering + pixel inspection) so they would compile
and run against **P5's unmodified code**:

- `TestGate51UpArrowAbsentAtZeroScroll` — renders a `ConfirmWarningScreen`
  with a body long enough to overflow (`modalFiller(700)`) at the natural
  `scroll == 0` starting state, and asserts the up-arrow chip has **no** ink,
  while first checking (as an INCONCLUSIVE guard) that the down-arrow chip
  *does*.
- `TestGate51DownArrowAbsentAtFullScroll` — forces `scroll` past its own
  ceiling and lets `Warning.Layout`'s own clamp reduce it to the real
  `maxScroll` (the same value real press-and-hold scrolling converges to),
  renders a second frame with that value already entering it, and asserts the
  down-arrow chip has **no** ink while the up-arrow chip does.

**Before the fix** (against P5's shared predicate, `go test ./gui/ -run
'TestGate51UpArrowAbsentAtZeroScroll|TestGate51DownArrowAbsentAtFullScroll' -v`):

```
=== RUN   TestGate51UpArrowAbsentAtZeroScroll
    scroll_arrows_test.go:177: the up arrow is drawn at scroll==0, pointing at content that does not exist above it (SPEC_s6b_pre_flash_cycle.md §5.1, R-D) -- the shared-predicate defect P5 flagged
--- FAIL: TestGate51UpArrowAbsentAtZeroScroll (0.01s)
=== RUN   TestGate51DownArrowAbsentAtFullScroll
    scroll_arrows_test.go:224: the down arrow is drawn at full scroll (w.scroll=91), pointing at content that does not exist below it (SPEC_s6b_pre_flash_cycle.md §5.1, R-D) -- the shared-predicate defect P5 flagged
--- FAIL: TestGate51DownArrowAbsentAtFullScroll (0.02s)
FAIL
FAIL	seedhammer.com/gui	0.071s
```

**After the fix** (same `-run` filter, plus the rest of the GATE 5.1/5.3
family):

```
=== RUN   TestBodyClipWidthStaysAt417
--- PASS: TestBodyClipWidthStaysAt417 (0.00s)
=== RUN   TestGate51VisibilityPredicateFormulaDown
--- PASS: TestGate51VisibilityPredicateFormulaDown (0.00s)
    (all 6 subtests PASS)
=== RUN   TestGate51VisibilityPredicateFormulaUp
--- PASS: TestGate51VisibilityPredicateFormulaUp (0.00s)
=== RUN   TestGate51ArrowsDrawnOnlyWhenContentOverflowsThePanel
--- PASS: TestGate51ArrowsDrawnOnlyWhenContentOverflowsThePanel (0.00s)
=== RUN   TestGate51UpArrowAbsentAtZeroScroll
--- PASS: TestGate51UpArrowAbsentAtZeroScroll (0.00s)
=== RUN   TestGate51DownArrowAbsentAtFullScroll
--- PASS: TestGate51DownArrowAbsentAtFullScroll (0.00s)
=== RUN   TestGate51ArrowActuallyScrolls
--- PASS: TestGate51ArrowActuallyScrolls (0.00s)
=== RUN   TestGate51bMaxScrollAgreesWithVisibility
--- FAIL: TestGate51bMaxScrollAgreesWithVisibility (0.00s)   -- EXPECTED, see below
=== RUN   TestGate53ChipDoesNotOverlapDrawnTextRows
--- PASS: TestGate53ChipDoesNotOverlapDrawnTextRows (0.00s)
```

`TestGate51ArrowActuallyScrolls` is the pre-existing regression pin ("can a
user do the thing") — it stayed green throughout: it exercises Down-then-Up
from the natural `scroll==0` start, where both `showDown` (content overflows)
and, after the first click, `showUp` (`scroll>0`) are already true under the
new per-direction predicates, so no behavioral change was needed for it to
keep passing.

## GATE 5.1b — confirmed unchanged, still fails as expected

`go test ./gui/ -run TestGate51bMaxScrollAgreesWithVisibility -v`:

```
scroll_arrows_test.go:332: R-E divergence probe over bodysz.Y in [0,320] (321 values): 22 diverge
scroll_arrows_test.go:335: diverging range:
    bodysz.Y=239: maxScroll=1 (>0=true) vs GATE-5.1=false
    bodysz.Y=240: maxScroll=2 (>0=true) vs GATE-5.1=false
    ... (238..259 omitted, all the same shape) ...
    bodysz.Y=260: maxScroll=22 (>0=true) vs GATE-5.1=false
--- FAIL: TestGate51bMaxScrollAgreesWithVisibility (0.00s)
```

22 of 321 values diverge, all in `[239,260]` — byte-identical to P5's own
measurement ("22 of 321 values diverging in [239,260]"). The assertion itself
was not touched beyond renaming its call site to `scrollArrowDownVisible(...,
0)`, which is numerically identical to the old `scrollArrowsVisible(...)` at
`scroll=0` (the value this probe always used). Not loosened; not adjusted to
pass.

## Sharded gate — `./gui/` package

`/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 6 20m`:

```
=== enumerating tests in ./gui/ ===
    853 top-level tests
    partition verified exhaustive: 853 == 853
=== running 6 shards in parallel (timeout 20m each) ===
  shard 0: ok   142 tests  ok  	seedhammer.com/gui	106.189s
  shard 1: ok   141 tests  ok  	seedhammer.com/gui	29.254s
  shard 2: FAIL 141 tests  FAIL
  shard 3: ok   141 tests  ok  	seedhammer.com/gui	129.757s
  shard 4: ok   141 tests  ok  	seedhammer.com/gui	39.051s
  shard 5: ok   141 tests  ok  	seedhammer.com/gui	2.676s
=== wall: 130s ===
RESULT: FAIL -- artifacts in /tmp/tmp.4wn9lJYRJ7
```

Wall time: **130s** (spec's "~106s" estimate; still well under the 20m
timeout). Exhaustiveness line: `partition verified exhaustive: 853 == 853`.
All six `err{0..5}.txt` stderr files are **empty** (0 bytes each, confirmed
with `ls -la`). Shard 2's failure output (`out2.txt`) contains exactly one
`--- FAIL`: `TestGate51bMaxScrollAgreesWithVisibility`, with the identical
22-value `[239,260]` divergence log shown above. Grepping `^--- FAIL` across
all six `out*.txt` files confirms this is the **only** failure in the whole
853-test run.

## Non-`gui` packages

`go test $(go list ./... | grep -v '/gui$') -count=1 -timeout 20m`: every
package reports `ok` (or `[no test files]`); wall time 20.757s. No failures.

## No golden moved

`git diff --stat -- gui/testdata backup/testdata` — empty, before and after.
No file named `sizeproof-front.bin` / `sizeproof-back.bin` anywhere in the
tree shows a diff (this phase's code touches only `gui/gui.go`'s
`Warning.Layout`, not any plate-rendering path). `git status --short` /
`git diff --stat` touch exactly two files: `gui/gui.go` (63 insertions, 34
deletions) and `gui/scroll_arrows_test.go` (162 insertions, 20 deletions).

## Other checks

- `gofmt -l gui/gui.go gui/scroll_arrows_test.go` — clean, no output.
- `go build ./...` — clean.
- `go vet ./gui/` — the same 2 pre-existing go1.26 `ArtifactDir` failures P5
  reported (`freetext_sizeproof_golden_test.go`, `op/draw_test.go`), not
  introduced by this phase.

## Prohibitions honored

- Body width still 417: `TestBodyClipWidthStaysAt417` — PASS (part of the
  green run above).
- GATE 5.1b not loosened; output unchanged from P5's own measurement.
- No change to anything outside §5.1's predicate — `git diff --stat` confirms
  only `gui/gui.go` and `gui/scroll_arrows_test.go` moved, and within
  `gui.go` only `Warning.Layout`'s predicate/handler/draw block and the
  `scrollArrowsVisible`→`scrollArrowUpVisible`/`scrollArrowDownVisible`
  function definitions changed.
