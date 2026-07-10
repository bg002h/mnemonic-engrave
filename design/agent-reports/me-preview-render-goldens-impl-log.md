# Implementation log — me preview render goldens + PNG stroke width (Cycle B: F13 + F15)

Single-implementer TDD execution of `design/SPEC_me_preview_render_goldens.md`
(GREEN at R0 round 1, 0C/0I). Worktree `me-cycleB` @ `me-preview-render-goldens`.
Go toolchain: `/home/bcg/.local/go/bin` (go1.26.4). Order is load-bearing: **B1
(disc-brush stroke) FIRST, all tests green, THEN B2 (goldens over corrected output).**

One section per step: test written, failure line (right-reason), change, counts.

---

## Step B1a — discRadius mapping + 1px floor

- **Tests first** (`render_png_test.go`): `TestDiscRadiusMapping` (exact-3 via
  scale=6/strokeWidth, exact-5 via 10/strokeWidth, md1-default=pngMaxPx/wantDx→2),
  `TestDiscRadiusFloor` (scale=0.5/strokeWidth→0.25→1, scale=0→1).
- **Fail-first (right reason — undefined fn):**
  `./render_png_test.go:35:14: undefined: discRadius`
- **Change:** added pure `discRadius(scale float64) int = max(1, round(strokeWidth*scale/2))`
  to `render_png.go` (+ `math` import).
- **Green:** `--- PASS: TestDiscRadiusMapping`, `--- PASS: TestDiscRadiusFloor`.

## Step B1b — deterministic disc-brush PNG stroke

- **Test first:** `TestPNGStrokePixelMass` — renders MD1_REF via production
  `renderPNG` (disc), compares black mass vs `renderHairline` (test-only
  `strokeHairline`, the verbatim pre-B1 1px drawLine kept reachable per §B1/R0 L2),
  asserts disc ≥ 2× hairline.
- **Fail-first (right reason — output still hairline):**
  `render_png_test.go:196: pixel-mass too low: disc=12919 hairline=12919 ratio=1.00, want >=2x`
- **Change (`render_png.go`):** replaced the single per-step `SetRGBA` in `drawLine`
  with `stampDisc` (per Bresenham step → round caps+joins); `drawLine` now takes
  `radius`; added bounds-checked `stampDisc` (SetRGBA, integer `dx²+dy²≤r²`, no AA);
  `renderPNG` computes `radius := discRadius(scale)`, grows canvas `+2*radius` per
  axis, shifts `toPx` origin `+radius`. `renderSVG`/PlanEngraving/params/CLI untouched.
- **Green:** `render_png_test.go:198: pixel-mass: disc=62702 hairline=12919 ratio=4.85x`
  (≈5×, matches R0 estimate). Full preview suite PASS; `go vet` clean.
- **Measured pixel-mass ratio: 4.85× (disc 62702 vs hairline 12919 black px).**
</content>
