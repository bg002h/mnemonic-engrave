# SPEC — me preview render goldens + PNG stroke width (Cycle B: F13 + F15)

Status: **GREEN — R0 passed at round 1 (0C/0I, 2L/3N folded inline)** (reviews:
`me-preview-render-goldens-spec-R0-round0.md` = 0C/1I/3L/4N all folded [I1 = pin decoded
`img.Pix` not compressed PNG bytes]; `…-round1.md` = GREEN, closure verified; the 2 Lows
were residual doc-drift, folded inline). Cleared for single-implementer TDD.
Closes two confirmed-low funds-audit follow-ups
recorded in `design/FOLLOWUPS.md`: `me-preview-png-stroke-width` (F13) and
`me-preview-render-goldens` (F15). The FOLLOWUPS entries record a **binding fable
decision (2026-07-06)**: fix the PNG stroke FIRST, then pin goldens over the corrected
output — pinning first would force an immediate re-baseline. This spec implements that
order. Evidence: `design/agent-reports/funds-audit-D4-sidecar-round0.md` (D4-3),
`funds-audit-D6-tests-round0.md` (D6-5). Executed locally (cloud CCR env failed to start).
Process: R0 architect gate to 0C/0I → single implementer, TDD → post-impl adversarial
review.

Recon (verified against current master `9fafb6b`, 2026-07-09):
- F13: `preview/render_png.go` walks the SAME pen-down cubics as the SVG but joins samples
  with `drawLine` — a **1px Bresenham hairline** (render_png.go:81, :94). The SVG strokes
  at `stroke-width=1920` (0.3mm, params.go), `stroke-linecap/linejoin=round`
  (render_svg.go:42). `scale = pngMaxPx(1000)/max(dx,dy)` (render_png.go:38); so the honest
  pixel stroke width is `strokeWidth*scale` (~3–4px at default), and the PNG under-draws
  ~3–4×. Centerlines are identical (both walk `PlanEngraving`), so this is legibility-only.
  CONFIRMED.
- F15: `render_test.go` asserts only STRUCTURE (SVG contains `<svg`/`<path`/one path/a
  ` C `; PNG has magic + non-empty bounds). No path-content or pixel golden. A pen-state
  swap or dropped-segment in the walk renders a wrong preview with the Go suite green.
  CONFIRMED.

## Non-goals
- No change to the engrave centerline math, `PlanEngraving`, `renderSVG`'s path
  accumulation, layout, or the sidecar CLI contract. F13 changes ONLY how PNG samples are
  stroked (hairline → disc); F15 adds tests + an `-update` harness, no production change
  beyond B1.
- Other cycles (F8/F9/F10 = Cycle A/PR #1; F11 sidecar discovery; F18 fuzz) untouched.

## B1 (F13) — deterministic disc-brush PNG stroke (DO THIS FIRST)

Replace the 1px `drawLine` join in `renderPNG` with a **deterministic integer disc-brush**
(fable decision, verbatim from the FOLLOWUPS entry):
- Compute `radius = max(1, int(round(strokeWidth*scale/2)))` once per render (`scale` as
  already computed at render_png.go:38). The 1px floor guarantees strokes never vanish on
  a heavily-downscaled render.
- Stamp a **precomputed integer disc** of that radius at **each Bresenham step** inside
  `drawLine` (R0 L1: replace the single `SetRGBA` at render_png.go:109 with a
  `stampDisc(img, x0, y0, radius, color)` call — per-step, NOT per-sample-point; a
  sample-point-only stamp has a 1px diagonal pinch at the `radius=1` floor). This yields
  round caps AND round joins for free, matching the SVG's `stroke-linecap/linejoin: round`.
- `stampDisc` MUST bounds-check every pixel via `img.Bounds()`/`SetRGBA` (R0 N1) — never
  raw `img.Pix[...]` indexing — so a disc straddling the canvas edge cannot panic.
- **Fully integer, NO anti-aliasing.** Determinism is load-bearing: B2 pins the PNG
  decoded-pixel (`img.Pix`) hash. Rejected alternatives (recorded): `x/image/vector` AA (golden-fragile, adds a
  dependency) and centerline-only documentation (leaves the artifact misleading).
- No new dependency. `render_svg.go` is NOT touched (it is already physically honest).
- Canvas sizing: the disc extends `radius` px beyond the centerline bounds, so the current
  `w=dx*scale+1 / h=dy*scale+1` (render_png.go:43-44) must grow by `+2*radius` in each
  axis AND the `toPx` origin must shift by `+radius`, so a stroke at the bounds edge is not
  clipped. (Verify no negative-index writes; `stampDisc` must clip to `img.Bounds()`.)

Implementation shape: a `stampDisc(img, cx, cy, radius, color)` helper (precompute the
disc offset set once, or test `dx*dx+dy*dy <= radius*radius` inline) called at **each
Bresenham step** inside `drawLine` (replacing the single `SetRGBA` at render_png.go:109),
NOT per polyline sample point; the existing pen-up/`dt==0` skips are unchanged.

Acceptance (Go, render_png_test.go / a new stroke test):
- **strokeWidth→px mapping unit test:** for a known `scale`, `radius == max(1,
  round(strokeWidth*scale/2))`; assert the helper computes it (extract a
  `discRadius(scale) int` pure fn to test directly).
- **Pixel-mass regression test:** render `MD1_REF` at default scale; count black pixels;
  assert the count ≥ 2× a **hairline baseline** (R0 L2). The baseline is obtained by
  keeping the OLD 1px `drawLine` reachable as a test-only helper `strokeHairline` (or
  pinning a measured constant with a comment); "radius=0-equivalent" is imprecise since a
  disc's minimum is a 3px plus, so name the baseline explicitly. Actual measured ratio is
  ~5×, so ≥2× is a robust non-flaky floor. Fails today (current output IS the hairline).
- **1px-floor test:** at a tiny synthetic scale where `strokeWidth*scale/2 < 0.5`,
  `discRadius` returns 1 (never 0).
- **Round-cap presence:** the happy-path PNG still decodes (existing TestRenderPNGValidHeader
  stays green) and bounds grew by the disc margin.

## B2 (F15) — render goldens (AFTER B1, over the corrected output)

Add `preview/render_golden_test.go`:
- **SVG golden:** pin the SHA-256 of the **whole SVG** for `MD1_REF` as an in-test hex
  constant (full-SVG catches viewBox/stroke-width regressions too, not just the `d`);
  assert equality. This catches pen-state swaps / dropped/reordered segments that the
  structural test misses. (SVG output is toolchain-stable — plain string formatting.)
- **PNG golden (R0 I1 — DECODED PIXELS, not compressed bytes):** pin the SHA-256 of the
  **decoded RGBA pixel buffer** `img.Pix` (decode the produced PNG with `png.Decode`, or
  hash the `*image.RGBA` before encoding), NOT the compressed PNG file bytes. `image/png`
  + `compress/flate` output can drift across Go toolchain versions for identical pixels,
  which would false-fail the golden on a runner/local Go bump. Hashing `img.Pix` has the
  same regression teeth (any pixel change fails) and is toolchain-stable. Also pin the
  total black-pixel count of the default `MD1_REF` render as a cheap drift-guard
  invariant (R0 N2/N4 — the whole-image black mass, like `wantDx/wantDy`, not a
  single disc's 13 px). Prefer Route A for the PNG golden: `png.Decode` the produced
  bytes → assert `*image.RGBA` → hash `Pix` (avoids a production seam; R0 N3).
- **M-vs-C token-count assertion:** count ` M ` and ` C ` tokens in the SVG `d`; pin both
  exact counts. A pen-up/pen-down swap changes the M:C ratio and flips this even if a hash
  regeneration masked the `d` golden.
- **Golden storage (R0 L3):** hashes and M:C counts as in-test hex/int constants;
  `testdata/` only if raw bytes are stored. Register the `-update` flag ONCE at package
  level (`var update = flag.Bool("update", false, ...)`; R0 N3) — the SVG-hash and
  PNG-pixel-hash tests share it. Document the regen command
  (`go test ./preview -run TestRenderGoldens -update`) in a test comment.

Acceptance:
- Goldens match the corrected (disc-brush) output. Regenerating with `-update` then
  running without it is green (round-trip).
- Prove teeth (perturb-then-revert, per the drift-guard TDD rule): swap the SVG walk's
  `if line` branch (pen state) in a scratch edit → the M-vs-C and `d`-hash assertions go
  red; revert. Drop the `dt==0` skip → red. Change `radius` → PNG-hash red.
- Determinism: run the golden test 3× → identical (no map-iteration / time / rng in the
  render path; if any nondeterminism surfaces, that is a finding, not a golden to pin).

## Ordering & verification
B1 (stroke fix) FIRST, its tests green, THEN B2 (pin goldens over the corrected output —
decoded-pixel hash for PNG, full-SVG hash for SVG).
Full verification before PR: `env PATH=".../go/bin:$PATH" ME_REQUIRE_GO=1 cargo test
--locked` (root — Rust unaffected, must stay green) + `go test ./...` in `preview/`
(includes the new stroke + golden tests) + `go vet` + a manual visual check: render one
`MD1_REF` PNG before/after and eyeball that the stroke is now visibly thicker.

## Open questions — adjudicated at R0 round 0
1. B1 disc stamp offset-set vs inline `dx²+dy²<=r²` → either acceptable (both
   deterministic); MUST bounds-check (N1).
2. B1 canvas growth `+2*radius` + origin shift → confirmed CORRECT (closes edge-clip, no
   negative index); NOT clip-at-old-bounds.
3. B2 golden storage → in-test hex/int constants for hashes/counts (L3).
4. B2 SVG golden → **full-SVG SHA-256 + separate M:C counts** (full-SVG also catches
   viewBox/stroke-width regressions); PNG golden → **decoded `img.Pix` hash** (I1).
5. Pixel-mass threshold → **≥2×** confirmed robust (actual measured ~5×).
