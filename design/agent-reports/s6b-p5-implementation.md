# S6b P5 implementation report — the scroll arrows (spec §5)

**Worktree:** `/scratch/code/shibboleth/wt-s6b`, branch `s6b-pre-flash`.
**Commit:** `808e403860b009e0628d252ce2fce46881465c5e` — "S6b P5 (5.1/5.1b/5.3): scroll arrows float over the body's fade zones".
**Gates:** 5.1, 5.1b, 5.3 (`SPEC_s6b_pre_flash_cycle.md` §5/§5.1, `REQUIREMENTS_s6b_pre_flash_cycle.md` R-E/R-I).

## What changed, and where

`gui/gui.go`, `Warning.Layout` (the body underneath `ConfirmWarningScreen` and
`ErrorScreen`) scrolled only via a raw per-frame `w.inp.Next(ctx,
ButtonFilter(Up), ButtonFilter(Down))` loop, with no on-screen affordance —
and the SH2 has no directional buttons, so on real hardware this loop was
unreachable dead code; only a test-injected `ButtonEvent` could ever drive it.

Per R-I, two arrows now float over the top-centre and bottom-centre of the
body, over the existing 16px `scrollFadeDist` fade zones, each with an opaque
background chip and an enlarged invisible touch target.

- **`warningBodyClip(dims image.Point) image.Rectangle`** (new) — the
  `bodyClip` computation extracted out of `Layout` with its arithmetic
  byte-for-byte unchanged (still `boxMargin=6`, `leadingSize=44`,
  `assets.NavBtnPrimary.Bounds().Dx()+btnMargin` on the right). Extracted so
  it has a name a test can call directly, per the hard prohibition "body
  width stays 417... assert it."
- **`scrollArrowsVisible(bodyClip, bodysz, dims image.Point) bool`** (new) —
  GATE 5.1's predicate verbatim: `bodyClip.Min.Y+scrollFadeDist+bodysz.Y >
  dims.Y`, checked against the **panel** (`dims.Y`), not `bodyClip.Dy()`. The
  call site in `Layout` carries a comment naming R-E, as the spec requires,
  because this predicate is coupled to `fadeClip` staying a stub and must be
  revisited when the real clip mask is restored.
- **`Warning`** gained two `Clickable` fields, `arrowUp`/`arrowDown`, bound to
  `Up`/`Down`, replacing the old raw-button loop entirely (the two mechanisms
  cannot coexist safely: `EventRouter.Next` is a single-consumer peek/consume
  queue, so two independent readers polling the same `ButtonFilter` in one
  frame would race). The click handlers are gated behind the same
  `showArrows` boolean that gates the drawing (`showArrows &&
  w.arrowUp.Clicked(ctx)`), not the drawing alone — but unlike
  `Button1`/`Button2`/`Button3`/`Center`, `Up`/`Down` have no physical button
  on this hardware that could route around that gate, so this does not
  reproduce the hazard `SeedScreen.Draw`'s `editBtn` comment warns about
  (`!s.NoEdit && editBtn.Clicked(ctx)` — there, `Button2` is real hardware
  and the guard has to sit on the handler, not just the layout). Each click
  moves `w.scroll` by `w.txtclip/2`, unchanged from the old loop's magnitude.
- **`scrollArrow(buf, th, chip, clk, icon) op.Op`** (new) — draws one arrow:
  **not** through `layoutNavigation` (verified: it computes `idx :=
  int(clk.Button-Button1)` into a `[3]int`, and `Up`(1)/`Down`(2) sort before
  `Button1`(6) in the `Button` enum, so `idx` is `-5`/`-4` for either — a
  panic on out-of-bounds array indexing, not merely wrong output). An opaque
  chip (`op.RoundedRect2` filled `th.Background`) sized `arrowChipWidth=36`
  wide and **exactly `scrollFadeDist=16` tall**, so it structurally cannot
  reach past the fade band into the `[60,298)` window where legitimate body
  text draws; the icon (`assets.ArrowUp`/`ArrowDown`, 15×9) centred inside
  it; `op.Input(buf, clk).Clip(chip.Inset(-arrowTouchPad))` with
  `arrowTouchPad=12` for a 60×40 touch target well past the 15×9 icon.
- **Layer order**: `op.Layer(arrows, body, titleOp, background)` — arrows
  listed first (topmost/front), matching the file's existing convention that
  earlier `op.Layer` arguments paint on top (the pre-existing `op.Layer(body,
  titleOp, background)` return already reads this way: body over the
  background).

**Files touched:** `gui/gui.go` (one function replaced, four new
functions/consts) and one new file, `gui/scroll_arrows_test.go`. No other
file changed; no `backup/*`, `cmd/me*`, or wire-format code touched, so the
Rust-primary rule and the `me`-CLI-untouched invariant are not implicated.

## TDD, per gate

**RED, captured before any production code existed** — the test file
referenced `scrollArrowsVisible` and `warningBodyClip`, neither of which
existed yet:

```
# seedhammer.com/gui
# [seedhammer.com/gui]
vet: gui/scroll_arrows_test.go:63:11: undefined: scrollArrowsVisible
```

**GREEN after implementation.** All of the following ran both standalone
(`go test ./gui/ -run TestGate5`) and inside this phase's full-suite run
below, with identical results:

```
=== RUN   TestBodyClipWidthStaysAt417
--- PASS: TestBodyClipWidthStaysAt417 (0.00s)
=== RUN   TestGate51VisibilityPredicateFormula
--- PASS: TestGate51VisibilityPredicateFormula (0.00s)
    --- PASS: .../260_--_exactly_fits,_no_arrow (0.00s)
    --- PASS: .../261_--_one_pixel_off_panel,_arrow (0.00s)
    --- PASS: .../far_under (0.00s)
    --- PASS: .../far_over (0.00s)
    --- PASS: .../270_--_the_rejected_bodysz.Y>bodyClip.Dy()_boundary (0.00s)
    --- PASS: .../239_--_the_rejected_maxScroll>0_boundary (0.00s)
=== RUN   TestGate51ArrowsDrawnOnlyWhenContentOverflowsThePanel
--- PASS: TestGate51ArrowsDrawnOnlyWhenContentOverflowsThePanel (0.01s)
=== RUN   TestGate51ArrowActuallyScrolls
--- PASS: TestGate51ArrowActuallyScrolls (0.00s)
=== RUN   TestGate53ChipDoesNotOverlapDrawnTextRows
--- PASS: TestGate53ChipDoesNotOverlapDrawnTextRows (0.01s)
```

Per test:

- **`TestBodyClipWidthStaysAt417`** — the standalone regression pin the hard
  prohibitions ask for by name: `warningBodyClip(480,320).Dx() == 417` and the
  full rectangle `(6,44)-(423,314)`.
- **`TestGate51VisibilityPredicateFormula`** (unit-level, 6 subtests) — pins
  `scrollArrowsVisible`'s exact formula, and specifically at the two
  boundaries of the two predicates spec §5.1 names and rejects: at
  `bodysz.Y=270` (the rejected `bodysz.Y>bodyClip.Dy()` false-negative
  boundary) the new predicate is already `true`; at `bodysz.Y=239` (the
  rejected `maxScroll>0` false-positive boundary) the new predicate is still
  `false`. Both prove the new predicate does not share either rejected
  predicate's blind spot, not just that it computes correctly in isolation.
- **`TestGate51ArrowsDrawnOnlyWhenContentOverflowsThePanel`** (integration) —
  proves the formula is actually *wired to the drawn pixels*, not just an
  isolated fact: renders `ConfirmWarningScreen`, samples for the arrow icon's
  own colour inside the expected top-chip rectangle. Absent for `"Short
  body."`, present for `modalFiller(700)`.
- **`TestGate51ArrowActuallyScrolls`** — the "can a user do the thing" check:
  drives `click(&ctx.Router, Down)`/`Up` (the same button-event path
  `Clickable.Next` resolves a touch tap to) and confirms `w.scroll` actually
  moves in both directions, not just that a formula or a raster pixel is
  correct in isolation.
- **`TestGate53ChipDoesNotOverlapDrawnTextRows`** — pixel-level (rasterises
  via `op.Drawer.Draw` into an `image.RGBA`, the same primitive
  `gui/raster_test.go`'s `runUITouchRaster` uses), three checks on one
  representative long modal (`modalFiller(700)`) with the arrow showing:
  1. No body-text-coloured ink appears above `y=60` (the top chip's own lower
     edge) anywhere **outside the chip's own X-footprint** — the first
     version of this check incorrectly scanned the chip's own X-range too and
     found the arrow icon's own ink there (a test bug, not a production one;
     fixed by excluding the chip's footprint from the "did text leak into the
     fade zone" scan).
  2. The last body-text row entirely inside the readable window `[60,298)` —
     located via `NewStyles().body.LineHeight()`, the **same** font metrics
     `Layout` itself uses, independently re-derived here rather than assumed
     — draws no ink past `y=298` (the bottom chip's own upper edge).
  3. Both chips' geometric-centre pixels are opaque: sampled where
     overflowing text is known to be drawn underneath (that is why the arrow
     is showing at all), and found to be exactly the icon colour or the
     background colour, never a third/blended colour that would mean a glyph
     bleeds through a mis-ordered or accidentally-transparent chip.

No pre-existing test asserted `Warning`/`ConfirmWarningScreen`/`ErrorScreen`
scroll behaviour via the old raw-button loop (checked: no hits for
`click(&ctx.Router, Up)`/`Down` against those three types anywhere in the
tree before this phase), so there was nothing to carry forward as a
"regression pin, green throughout" for the scroll mechanism itself — the
closest such pins are the pre-existing `ConfirmWarningScreen`/`ErrorScreen`
consumers (`TestConfirm*`, `TestBip85*`, `TestMultisigBuildFlow*`,
`TestUnloadNoticeIsActuallyDrawn`, `TestModalsThisBlockTouchesAreDrawnInFull`,
36 tests total run explicitly during implementation, all green before and
after — confirming the replacement of the scroll mechanism did not disturb
any of `Warning`'s other callers).

## GATE 5.1b — raw output, as data

**`TestGate51bMaxScrollAgreesWithVisibility` FAILS, in both the standalone
run and the full-suite run below, and this is the spec-sanctioned state.**
Per `SPEC_s6b_pre_flash_cycle.md` §7 ("R-E's `maxScroll` divergence probe —
**failures expected**, files findings, does not gate") and
`IMPLEMENTATION_PLAN_s6b.md` ("GATE 5.1b is expected to FAIL and does not
gate... a red result on a probe reads as 'the gate failed, loosen it', and
that is how a false-PASS gate is born"). **It was not loosened.**

Full raw output:

```
=== RUN   TestGate51bMaxScrollAgreesWithVisibility
    scroll_arrows_test.go:190: R-E divergence probe over bodysz.Y in [0,320] (321 values): 22 diverge
    scroll_arrows_test.go:193: diverging range:
        bodysz.Y=239: maxScroll=1 (>0=true) vs GATE-5.1=false
        bodysz.Y=240: maxScroll=2 (>0=true) vs GATE-5.1=false
        bodysz.Y=241: maxScroll=3 (>0=true) vs GATE-5.1=false
        bodysz.Y=242: maxScroll=4 (>0=true) vs GATE-5.1=false
        bodysz.Y=243: maxScroll=5 (>0=true) vs GATE-5.1=false
        bodysz.Y=244: maxScroll=6 (>0=true) vs GATE-5.1=false
        bodysz.Y=245: maxScroll=7 (>0=true) vs GATE-5.1=false
        bodysz.Y=246: maxScroll=8 (>0=true) vs GATE-5.1=false
        bodysz.Y=247: maxScroll=9 (>0=true) vs GATE-5.1=false
        bodysz.Y=248: maxScroll=10 (>0=true) vs GATE-5.1=false
        bodysz.Y=249: maxScroll=11 (>0=true) vs GATE-5.1=false
        bodysz.Y=250: maxScroll=12 (>0=true) vs GATE-5.1=false
        bodysz.Y=251: maxScroll=13 (>0=true) vs GATE-5.1=false
        bodysz.Y=252: maxScroll=14 (>0=true) vs GATE-5.1=false
        bodysz.Y=253: maxScroll=15 (>0=true) vs GATE-5.1=false
        bodysz.Y=254: maxScroll=16 (>0=true) vs GATE-5.1=false
        bodysz.Y=255: maxScroll=17 (>0=true) vs GATE-5.1=false
        bodysz.Y=256: maxScroll=18 (>0=true) vs GATE-5.1=false
        bodysz.Y=257: maxScroll=19 (>0=true) vs GATE-5.1=false
        bodysz.Y=258: maxScroll=20 (>0=true) vs GATE-5.1=false
        bodysz.Y=259: maxScroll=21 (>0=true) vs GATE-5.1=false
        bodysz.Y=260: maxScroll=22 (>0=true) vs GATE-5.1=false
    scroll_arrows_test.go:196: maxScroll>0 disagrees with GATE 5.1's predicate on 22 of 321 bodysz.Y values in [0,320] -- see the log above for the exact range. EXPECTED (R-E): fadeClip is a stubbed no-op, so maxScroll's reserved fade margin is never actually rendered as fade. This is a FINDING against the deferred honest-geometry work that restores fadeClip (R-E), not a defect in this phase.
--- FAIL: TestGate51bMaxScrollAgreesWithVisibility (0.00s)
```

**It came out RED, as expected — not green.** 22 of 321 sampled `bodysz.Y`
values (the integer range `[239,260]`) diverge: `maxScroll>0` reads `true`
(the old, rejected predicate believes content is scrollable) while GATE 5.1's
predicate — what the panel actually shows — reads `false` (content is
entirely visible). This is exactly the false-positive direction R-E's own
text predicts: `maxScroll` reserves `2*scrollFadeDist=32px` of margin that
`fadeClip` (a stubbed no-op) never actually renders as fade. Nothing in P5
touched `fadeClip`, so this divergence is expected to persist unchanged until
the honest-geometry work restores the real clip mask (filed after F-192, per
R-E) — it did not close, and the probe is genuinely probing (it is driven by
`scrollArrowsVisible`, the real production predicate, against the real
`maxScroll` formula, not a stand-in for either).

## Full suite — run once, blocking, stdout/stderr separated

```
export PATH="/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin:$PATH"
go build ./...                              # clean, no output
go vet ./gui/                               # only the 2 pre-existing go1.26
                                             #   ArtifactDir failures below
go test ./... -count=1 -timeout 20m -v  > stdout.log 2> stderr.log
```

`go vet ./gui/` output (run standalone, matching the plan's disclosed
exception exactly):

```
gui/op/draw_test.go:176:24: testing.ArtifactDir requires go1.26 or later (file is go1.25)
gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
```

Both pre-existing, per the brief ("two in gui, two in backup") and confirmed
by `git diff` touching neither file.

**`stderr.log` was empty.** `stdout.log`: **63 of 64 packages report `ok`**;
**exactly one** `--- FAIL` in the entire run
(`TestGate51bMaxScrollAgreesWithVisibility`, reported above verbatim), and
exactly one package-level `FAIL` line:

```
FAIL	seedhammer.com/gui	463.751s
```

Every other package line in the run is `ok`, including
`seedhammer.com/gui/op` (1.514s), `seedhammer.com/gui/widget` (1.144s),
`seedhammer.com/gui/text` (1.144s), `seedhammer.com/gui/saver` (1.621s),
`seedhammer.com/gui/assets` (0.001s), `seedhammer.com/backup` (3.426s),
`seedhammer.com/bspline` (0.975s), `seedhammer.com/seal` (16.923s) — grepped
directly from the log, not recalled.

**`gui`'s wall time: 463.751s** — against Go's 600s per-package default,
~77%. Trend: **440.6s (P3) → 496.8s (P4) → 463.751s (P5)** — down from P4,
inside the P3–P4 band, and nowhere near the 600s ceiling this phase. (The
plan's own P5 projection of "~553s" did not materialise; this phase added
five small, cheap tests — the most expensive, `TestGate51bMaxScrollAgreesWith-
Visibility`, is a pure in-memory loop over 321 integers with no rendering —
rather than a real multi-plate engrave walk of the kind that drove P4's
increase.)

**No golden moved.** `git diff --stat -- gui/testdata backup/testdata` is
empty, checked both before and after the full-suite run.
`git status --short` in the worktree, before committing, showed only
`gui/gui.go` (modified) and `gui/scroll_arrows_test.go` (new — untracked).
**`gui/testdata/sizeproof-{front,back}.bin` did NOT move.** The plan's own
per-phase table lists them as goldens that "may move" for P5, but nothing
in this phase's diff touches plate-rendering code
(`backup`/`bspline`/`engrave`, the packages those two goldens are recorded
from) — only `gui.Warning`'s on-screen layout, a different rendering path
entirely. Read as a permissive allowance ("may," not "will" or "must"), not
a requirement, so this is not a discrepancy — but flagging it as a fact
worth recording, since a future reader of the plan's table might otherwise
expect churn that this phase never had a mechanism to cause.

## Prohibitions honored

- **Not through `layoutNavigation`** — `scrollArrow` builds its own
  `op.Input`/`op.Compose` tree directly; `layoutNavigation` is not called
  anywhere in the diff (`git diff` shows no new call site).
- **Body width stays 417** — `TestBodyClipWidthStaysAt417` asserts it against
  the real `warningBodyClip` function, not a hand-copied fixture; green in
  both the standalone and full-suite runs.
- **GATE 5.1b not weakened to pass** — see above; its assertion is
  `oldPredicate == newPredicate`, unmodified from what R-E's own text asks
  for ("S6b owes a test that the two agree"), and it is left red.
- **GATE 5.1 is green** — confirmed in both runs, no subtests skipped or
  loosened.

## Spec/plan findings

**Nothing the spec got wrong on §5 itself.** The visibility-predicate
formula, the R-E coupling, and the three implementation constraints (not
`layoutNavigation`, larger hit area, mandatory chip) all matched the real
code exactly as measured (`bodyClip` = `(6,44)-(423,314)`, 417 wide;
`assets.ArrowUp`/`ArrowDown` = 15×9; `leadingSize`=44, `boxMargin`=6,
`scrollFadeDist`=16 — all confirmed against `gui/theme.go` and `gui/gui.go`
before use, not assumed from the brief).

**One thing worth recording, not a defect:** `IMPLEMENTATION_PLAN_s6b.md`'s
per-phase golden table lists `sizeproof-{front,back}.bin` as goldens that
"may move" for P5. As reported above, they did not, and P5's implementation
has no code path that could move them (they are recorded from
`backup`/`bspline` plate-rendering, not from `gui`'s on-screen layout). This
reads as the table's own stated latitude ("may," not "will"), so it is not a
spec/plan error — but a plan reader relying on that line to predict P5's
diff shape would be misled, since it is the one line in that table's P5 row
that turned out not to apply to the arrows-only implementation actually
taken.

**One design choice the spec left implicit, made explicit here:** §5.1's
predicate formula ("show arrows iff...") is stated once, not once per
direction, and GATE 5.1's own wording ("the predicate agrees with actual
visibility") never references `w.scroll`. I read this as: **both arrows
share one visibility gate** — they show or hide together, based on whether
content overflows the panel at all, not independently based on current
scroll position (e.g. hiding the up-arrow at `scroll==0`). This is
consistent with §5's own geometry (the predicate is a static function of
`bodysz.Y` alone) and is what `scrollArrowsVisible`/`showArrows` implements.
Flagging this as a reading rather than a re-derivation, since a
per-direction predicate was never ruled out in so many words — only the
single-predicate formula was ever given.
