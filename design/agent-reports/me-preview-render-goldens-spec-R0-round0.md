# R0 architect review — SPEC_me_preview_render_goldens.md — Round 0

- **Spec:** `/scratch/code/shibboleth/me-cycleB/design/SPEC_me_preview_render_goldens.md`
- **Worktree/branch:** `me-cycleB` @ `me-preview-render-goldens` (off mnemonic-engrave master `9fafb6b`)
- **Scope closed by spec:** F13 (`me-preview-png-stroke-width`) + F15 (`me-preview-render-goldens`), Cycle B.
- **Binding prior decision (NOT relitigated):** fable 2026-07-06 — fix the PNG stroke via a
  deterministic integer disc-brush FIRST, then pin goldens over the corrected output. This review
  checks the spec's *faithfulness* to that decision and its *executability*, adversarially.
- **Reviewer:** opus R0 architect. Standard: GREEN = 0 Critical / 0 Important.

**VERDICT: NOT GREEN — 0 Critical / 1 Important (+ 3 Low, 4 Nit).**

The spec is faithful to the fable ordering (B1 stroke-fix before B2 golden-pin — no re-baseline),
the disc-brush math and canvas-growth fix are correct, renderSVG is untouched, and scope is clean.
The single gate-blocker is **I1**: B2 pins the *compressed* PNG byte hash, which is deterministic
only *within one Go toolchain* — `image/png` + `compress/flate` output can drift across Go versions,
making a funds-adjacent regression guard produce false failures. A zero-cost, strictly-better
alternative (hash the decoded RGBA pixel buffer) has identical teeth without the fragility. Fold I1,
re-dispatch.

---

## Verification performed (evidence base)

Read from the worktree: the spec; `design/FOLLOWUPS.md` (both `me-preview-*` entries, incl. the
verbatim disc-brush decision); `funds-audit-D4-sidecar-round0.md` (D4-3) + `funds-audit-D6-tests-round0.md`
(D6-5/P5); `preview/render_png.go`, `render_svg.go`, `params.go`, `layout.go`, `render_test.go`,
`params_test.go`, `version_test.go`; `third_party/seedhammer/bezier/bezier.go` (`Sample`).

Concrete numbers for `MD1_REF = "md1yqpqqxqq8xtwhw4xwn4qh"` at the golden bbox
(`wantDx=431224`, `wantDy=200868`, `strokeWidth=1920`), engraveBest → `text+qr`:

| quantity | value |
|---|---|
| `scale = 1000/max(dx,dy)` | 0.00231898 (no `>1` clamp) |
| honest stroke `strokeWidth*scale` | **4.45 px** (confirms D4-3's "~3–4×" under-draw) |
| `strokeWidth*scale/2` | 2.226 → `round` = 2 |
| `radius = max(1, round(...))` | **2** |
| disc pixels/stamp (`dx²+dy²≤4`) | 13 |
| effective stroke width `2r+1` | **5 px** |
| grown canvas `int(dx*scale)+1+2r × int(dy*scale)+1+2r` | ~1005 × 470 |

---

## 1. B1 disc-brush math — CORRECT

- **`radius = max(1, round(strokeWidth*scale/2))`** is right. `scale` (render_png.go:38) converts
  machine units → px; SVG stroke-*width* is `strokeWidth` (full width, render_svg.go:42), so the
  disc *radius* is half of `strokeWidth*scale`. The `max(1,…)` floor guarantees a visible stroke on
  heavy downscale. Rounds to **2** at default (above) → ~5 px, matching the SVG's ~4.45 px honest
  width (slightly over — see N2, immaterial).
- **Canvas growth (`+2*radius` each axis) + origin shift (`+radius`) — CORRECT and exactly closes
  the edge-clip.** Centerline px ∈ `[0, int(dx*scale)]`; with `toPx += radius` they become
  `[radius, int(dx*scale)+radius]`; a disc adds `±radius`, so painted px ∈ `[0, int(dx*scale)+2*radius]`
  = `[0, w-1]` with `w = int(dx*scale)+1+2*radius`. Left edge lands exactly on 0 (no negative index),
  right edge exactly on `w-1` (no overflow). Symmetric in y. This answers Open-Question 2: **grow +
  shift is correct; clipping at the old bounds would shave edge strokes — do NOT clip-only.**
- **Stamp at every Bresenham step does NOT over-draw harmfully.** `SetRGBA`-to-black is idempotent;
  redundant writes cost only time (~13 writes × ~thousands of steps on a ≤1005 px canvas — trivial).
  Correctness is unaffected. (But note the *placement* ambiguity — L1.)
- **Negative-index / off-by-one:** the `+radius` margin absorbs the extremes; the mandated
  `img.Bounds()` clip in `stampDisc` backstops any residual sub-pixel Bézier overshoot *provided the
  implementer uses bounds-checked writes* (`SetRGBA` no-ops out of range) and NOT raw `img.Pix[]`
  indexing (which would panic on a negative index). Spec says "clip to `img.Bounds()`"; make it
  explicit — N1.

## 2. B1 changes ONLY PNG stroking — CONFIRMED

- `renderSVG` (render_svg.go) is not referenced by B1; the single accumulated `<path>` walk,
  `stroke-width`, and `linecap/linejoin` are untouched. The centerline (`PlanEngraving`) and
  `engraveBest`/layout are untouched. Fidelity claim in the spec's Non-goals is accurate.
- **Existing tests stay green under the grown canvas:** `TestRenderSVGContainsExpectedStructure`
  (render_test.go:10) is SVG-only — unaffected. `TestRenderPNGValidHeader` (render_test.go:36) only
  asserts PNG magic + `Dx()>0 && Dy()>0` — a larger canvas still decodes. `TestRenderPNGToFile`
  (version_test.go:69) only `png.Decode`s — no dimension pin. `TestParamsGeometryGolden`
  (params_test.go:38) measures `PlanEngraving` bounds independently of the raster — unaffected. **No
  existing test pins PNG dimensions**, so canvas growth breaks nothing.

## 3. B2 golden feasibility — determinism OK for SVG; PNG byte-hash is the landmine (I1)

- **Render path is input-deterministic — no time/rng/map iteration.** `renderSVG` is a pure
  `fmt.Fprintf` loop over `PlanEngraving` (integer B-spline timing). `renderPNG` iterates the same
  `iter.Seq`, `bezier.Sample` (bezier.go:442 — pure integer arithmetic, `samplingRate=200`, no
  rng/map/time), and stamps discs. `engraveBest`→`qr.Encode` is a pure function of `(string, level)`
  with deterministic QR mask selection. So the SVG `d` string and the RGBA *pixel buffer* are fully
  reproducible across runs and across toolchains. The "run the golden 3× → identical" check will pass.
- **SVG golden is rock-solid.** Pinning a SHA-256 of the full SVG (Open-Question 4 — endorse
  **full-SVG hash + separate M:C counts**; the full SVG also catches `viewBox`/`stroke-width`
  regressions the bare `d` misses) is a pure-string, toolchain-independent anchor. Good.
- **M-vs-C token count is sound.** `renderSVG` emits `" C …"` per pen-down cubic and `" M …"` per
  pen-up; coordinates are decimal integers (no letters, `-` only), so `strings.Count(d, " C ")` /
  `" M "` are clean and a pen-state swap flips the ratio even if a `-update` regenerated the `d`-hash.
- **Ordering avoids the re-baseline the fable decision warns about — CONFIRMED.** B1 lands + its
  tests go green, THEN B2 pins the corrected bytes. The golden is created once, over disc-brush
  output; no just-created golden is invalidated. Faithful to the binding decision.
- **BUT the PNG *byte* hash is deterministic only intra-toolchain → I1** (below). The pixel *content*
  is deterministic; the PNG *serialization* is not stable across `image/png`/`compress/flate` versions.

## 4. TDD integrity — mostly sound; two clarifications (L2, N4)

- **Pixel-mass regression test fails-first for the right reason.** Today `renderPNG` draws 1 px
  Bresenham hairlines, so black-pixel count ≈ hairline count; asserting `≥2×` a hairline baseline is
  RED today and flips GREEN after B1. Correct teeth.
- **Threshold is robust, not flaky.** Estimated actual ratio: effective stroke width `2r+1 = 5 px`
  vs 1 px hairline → line-dominated mass ≈ **~5×**; even with junction/QR overlap it stays ≥3×.
  `≥2×` sits ~1.5× below that → comfortable, non-flaky floor. Answers Open-Question 5: **≥2× is a
  sound floor.** Even the `radius=1` floor case yields a 3-px-wide plus (~3× mass), still ≥2×.
- **discRadius / 1px-floor unit tests** are straightforward pure-function tests (new helper → compile-
  red → green); fine.
- **L2 (clarify the baseline):** the spec says assert "≥2× the count a **1px hairline** would produce"
  and "prove teeth by … forcing `radius=0-equivalent (1px)`." But a disc **cannot** be 1 px — its
  minimum (`radius=1`) is a 3-px-wide plus. So (a) the committed assertion needs a concrete hairline
  baseline source — either KEEP `drawLine` as a test-only helper to render the 1 px reference count
  live, or pin the measured hairline count as a documented constant; and (b) the teeth-perturbation
  must use that retained hairline path, not `radius=1`. Specify this so the test is self-contained.

## 5. Scope — CLEAN, no bleed

- B1 edits `render_png.go` only. B2 adds `render_golden_test.go` (+ optional `testdata/`). No change
  to `main.go`/`run` (sidecar CLI contract), no Rust-side change, no `md`/`mk` codec change.
- Non-goals explicitly exclude F8/F9/F10 (Cycle A / PR #1), F11 (sidecar discovery), F18 (fuzz).
  Confirmed untouched.
- The `-update` flag is test-package-only (`var update = flag.Bool(...)`), no production surface. No
  `-update` flag exists elsewhere in `preview/` today (no duplicate-registration panic) — N3.
- **Rust-primary rule N/A:** the PNG renderer is fork-native (upstream `golden.go` is SVG-only, per
  the FOLLOWUPS entry); no Rust counterpart to lead. Correct.

---

## Findings

### Important

**I1 — B2 pins the *compressed* PNG byte hash; that hash is deterministic only within one Go
toolchain, so the golden is toolchain-fragile.** (SPEC §B2 "pin the SHA-256 of the deterministic PNG
bytes"; §"determinism is load-bearing"; Open-Question just above.)
The disc-brush makes the *pixel buffer* (`img.Pix`) fully deterministic and toolchain-independent —
that is the artifact B1 actually controls. `png.Encode`, however, runs the pixels through
`compress/flate`, whose output has historically changed across Go releases (filter heuristics /
deflate). So a Go toolchain bump (CI runner upgrade, or a developer on a different local Go) can flip
the PNG byte-hash RED with **identical pixels** — a false regression in a funds-adjacent guard, and a
spurious-red that invites reflexive `-update` (which then *masks* a real future regression). The
spec's flat claim that "the PNG golden pins byte hashes, so determinism is load-bearing" is only true
intra-toolchain and should not be shipped unqualified.
*Fix (either closes it):* **(preferred)** pin the SHA-256 of the **decoded RGBA pixel bytes**
(`png.Decode` → `*image.RGBA` → hash `Pix`, or hash `img.Pix` directly before `png.Encode`) — same
teeth (any stroke/coord/canvas change flips it), zero compression-format coupling, toolchain-stable;
**or** keep the compressed-byte hash but amend the spec to (i) drop the unqualified "deterministic"
claim, (ii) require CI to pin the exact Go version, and (iii) document that a Go bump legitimately
requires an `-update` + visual re-check. The pixel-hash route is strictly better and cheaper; the SVG
golden already stays a pure-string hash. (This is exactly the "pinning a byte hash is a landmine"
case the task flags.)

### Low

**L1 — Spec self-contradicts on disc-stamp placement.** The B1 bullet says stamp "at each **Bresenham
step** … (replace the single `SetRGBA` in `drawLine` with a disc stamp)"; the "Implementation shape"
paragraph says stamp "at each **polyline sample point**." These paint different pixel sets → different
PNG hash. Both are gap-free at the default `radius=2` because `bezier.Sample` targets ~1 px chords
(`spacing = int(1/scale)` ⇒ chord·scale ≈ 1 px) and a radius-2 disc bridges a 2-px diagonal jump; but
at the `radius=1` floor, sample-point-only stamping has a theoretical 1-px diagonal pinch on a 2-px
chord (the two plus-shaped discs miss the shared diagonal pixel). *Fix:* mandate **Bresenham-step
stamping** (replace the `SetRGBA` inside `drawLine`), which is gap-free by construction regardless of
`Sample`'s spacing, and delete the contradicting "at each polyline sample point" phrasing. Resolves
Open-Question 1 too (offset-set vs inline `dx²+dy²≤r²`: either is deterministic and acceptable — pick
for taste; the *placement* is what must be pinned).

**L2 — Pixel-mass baseline source is unspecified and "1px = radius=0-equivalent" is imprecise.** See
§4 L2 above: a disc cannot be 1 px. Specify the hairline baseline (retain `drawLine` as a test helper
OR pin the measured hairline count as a constant) and clarify the teeth-perturbation uses that path.

**L3 — Open-Question 3 (golden storage): recommend `testdata/` files for any raw bytes, in-test hex
consts for hashes.** For hash-only goldens (SVG hash, pixel hash, M:C counts) an in-test `const` is
simplest and still `-update`-friendly. If storing the raw reference PNG/SVG bytes, use `testdata/`
(the Go idiom). Either is acceptable; state the choice so the implementer doesn't guess.

### Nit

**N1 — Require bounds-checked writes in `stampDisc`.** The `img.Bounds()` clip must be realized via
`SetRGBA` (no-ops out of range) or an explicit `x∈[0,w) && y∈[0,h)` guard, NOT raw `img.Pix[idx]`
indexing (a negative `idx` from any residual overshoot would panic). The existing `drawLine` already
uses the guarded `SetRGBA` pattern (render_png.go:108) — mirror it.

**N2 — `2r+1` effective width slightly exceeds `strokeWidth*scale`.** At default, disc width = 5 px
vs honest 4.45 px (~0.5 px over). Immaterial and arguably *better* for legibility (round cap); no
change needed — just don't be surprised the PNG stroke reads marginally bolder than the SVG.

**N3 — Register `-update` exactly once.** No `-update` flag exists in `preview/` today; ensure the new
`var update = flag.Bool("update", …)` is the sole registration to avoid a future duplicate-flag panic
if another `_test.go` adds one.

**N4 — Also pin the exact disc black-pixel count in B2** (in addition to the coarse `≥2×` guard), so
the pixel-mass has a fine, exact regression anchor alongside the pixel/PNG hash.

---

## Open-question dispositions (for the author)

1. Offset-set vs inline `dx²+dy²≤r²`: **either** (both deterministic). The *stamp placement* (L1) is
   what must be fixed to Bresenham-step.
2. Canvas `+2*radius` + `+radius` shift: **correct** — grow, do not clip-only (verified §1).
3. `testdata/` vs in-test consts: **in-test hex consts for hashes; `testdata/` if storing raw bytes** (L3).
4. Full-SVG hash vs `d`-only: **full-SVG hash + separate M:C counts** (catches viewBox/stroke-width).
5. Pixel-mass threshold: **≥2× is a sound, non-flaky floor** (actual ~5×; even radius-1 floor ~3×).

---

## Decision

**NOT GREEN (0C / 1I).** Fold **I1** (pin the decoded pixel-buffer hash, or qualify the PNG byte-hash
+ pin the Go toolchain). Address **L1/L2/L3** and the Nits in the same fold (all one-line spec edits).
Persist this review verbatim, re-dispatch R0 round 1 after the fold (folds can introduce drift).
