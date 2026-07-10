# Post-implementation adversarial execution review — me-preview-render-goldens (Cycle B) — Round 0

- **Diff under review:** `master..me-preview-render-goldens` (worktree `/scratch/code/shibboleth/me-cycleB`)
- **Commits:** `6c5f92e` spec (GREEN @ R0 r1) → `4ba5ed6` B1 disc-brush stroke (F13) → `d9b25c1` B2 render goldens (F15)
- **Spec:** `design/SPEC_me_preview_render_goldens.md` (GREEN R0 round 1, 0C/0I)
- **Reviewer:** opus independent post-implementation adversarial execution reviewer. Standard: GREEN = 0 Critical / 0 Important.
- **Environment:** Go 1.26.4 (`/home/bcg/.local/go/bin`); cargo on PATH. Own probes run in `/scratch/code/shibboleth/me-review-scratch-b` (removed after use).

## VERDICT: GREEN — 0 Critical / 0 Important (+ 0 Low, 2 Nit).

Every load-bearing claim in the spec, the R0 folds (I1 decoded-`img.Pix` hash, L1 Bresenham-step
stamping, N1 bounds-checked stamp), and all four implementer scrutiny flags were independently
re-verified against the code AND by my own probes/perturbations. The disc-brush math is edge-clip-
and negative-index-free, the PNG golden genuinely hashes DECODED pixels (Route A == Route B, verified
to the byte, equals the pinned constant), the goldens are deterministic (5× in-process + 3× suite +
`-update` round-trip all identical), and the two independent perturbation proofs (SVG branch swap;
radius bump) show the SVG and PNG goldens have real, decoupled teeth. Scope is clean: only
`render_png.go` + two test files + the four design docs changed; `render_svg.go`, `params.go`,
`layout.go`, `main.go`/CLI, and the entire Rust side are untouched. All suites green. The two Nits are
informational (a theoretically-unbounded radius that is unreachable for real inputs; a stray
`</content>` tag in the impl-log). Implementation may proceed to PR.

---

## Evidence base (what I ran)

- `git diff --stat master..HEAD` + `--name-only` (scope); `git diff` of `render_svg.go`/`params.go`/`layout.go`/`main.go` (all EMPTY).
- `go vet ./...` (clean); `go build ./...` (OK); `go test ./...` in `preview/` (PASS, all tests); `-count=1` (no-cache PASS).
- `go test -run TestRenderGoldens -count=1` ×3 (all `ok`); `-update` (regenerates the EXACT pinned constants).
- `ME_REQUIRE_GO=1 cargo test --locked` at root: **28 passed / 0 failed** (23 lib + 1 cross_lang + 3 golden + 1 preview_cross_lang + 0 doc).
- Own probes (scratch copy): Route-A==Route-B lossless/opaque proof; 5× determinism; M/C-form dissection; hairline canvas-invariance; two perturb-then-revert teeth proofs.
- `git grep '#[test]'` count on `master` / branch / `me-preview-hardening`.

---

## 1. Disc-brush correctness (B1) — CORRECT

**`discRadius` (render_png.go:24-30)** — re-derived `radius = max(1, round(strokeWidth*scale/2))`,
`strokeWidth = 1920` (params.go:9, = 0.3mm). At the MD1_REF default render `scale = 1000/431224 =
0.00232`, so `round(1920*0.00232/2) = round(2.226) = 2`. `TestDiscRadiusMapping`/`TestDiscRadiusFloor`
pass (incl. the `scale=0 → 1` floor). Correct and matches the R0 numbers.

**Canvas growth + origin shift (render_png.go:61-77)** — re-derived the edge-clip / negative-index
claim symbolically:
- `toPx` maps `p.X∈[Min.X,Max.X]` → `int((p.X-Min.X)*scale)+radius ∈ [radius, int(dx*scale)+radius]`.
- `stampDisc` paints `±radius` around that, so painted x ∈ `[0, int(dx*scale)+2*radius]`.
- `w = int(dx*scale)+1+2*radius`, so painted x ∈ `[0, w-1]`. Left edge lands exactly on 0 (no negative
  index), right edge exactly on `w-1` (no overflow). Symmetric in y. **The `+2*radius` grow + `+radius`
  shift exactly closes the edge-clip with no negative index.** Confirmed, matching R0 §1.

**`stampDisc` bounds-check (render_png.go:151-165)** — writes go through the bounds-checked `SetRGBA`
(which itself no-ops out of range) AND an explicit `x∈[0,w) && y∈[0,h)` guard (lines 160-161); NO raw
`img.Pix[...]` indexing. A disc straddling the canvas edge cannot panic. N1 satisfied.

**Per-Bresenham-step stamping (render_png.go:131), NOT per-sample-point (L1 fold)** — confirmed the
`stampDisc` call replaced the single `SetRGBA` inside the Bresenham loop of `drawLine`. Over-draw is
harmless: `SetRGBA`-to-black is idempotent and `countBlack` counts DISTINCT black pixels (union of disc
stamps), so redundant writes cost only time, never double-count. The pixel-mass is robust. `TestPNGStroke
PixelMass` = disc 62702 / hairline 12919 = **4.85×** (≥2× floor), matching the impl log and R0 ~5× estimate.

**renderSVG / PlanEngraving / params.go UNTOUCHED** — `git diff master..HEAD -- render_svg.go params.go
layout.go main.go` is **EMPTY** for all four. `git diff --name-only` lists only `render_png.go`, the two
test files, and four `design/*.md`. Confirmed.

## 2. Golden integrity (B2) — the load-bearing part — CORRECT

**PNG golden hashes DECODED `img.Pix`, not compressed bytes (R0 I1).** Re-read `render_golden_test.go`
:60-67: `renderPNG` → `decodeRGBA` (`png.Decode` + assert `*image.RGBA`, lines 162-173 of render_png_test.go)
→ `sha256.Sum256(img.Pix)`. This is Route A, over decoded pixels. NOT `sha256(pngBytes)`. Correct.

**I verified Route A == Route B to the byte.** My probe hashed the `*image.RGBA` BEFORE `png.Encode` and
compared to the post-`png.Decode` `Pix` hash: **identical**, and both equal the pinned
`pngPixGolden = 5d7d153b…368a`; decoded type `*image.RGBA`, `Opaque()==true`. So the encode/decode is
lossless for this (always-opaque) image and the golden is genuinely the pre-encode pixel buffer,
toolchain-independent. The R0 I1 fix is real, not cosmetic.

**Perturbation proof #1 — swap the SVG pen-state branch (`if line` → `if !line` in render_svg.go):**
```
render_golden_test.go:80: SVG hash drift: got 6353219f… want e1d0311f…
render_golden_test.go:83: SVG M-command count drift: got 2578 want 2132
render_golden_test.go:86: SVG C-command count drift: got 2132 want 2578
```
SVG hash + M/C counts go RED (M/C exactly swap); **the PNG pixel-hash stayed GREEN** (render_svg change
does not touch the raster). Reverted → clean.

**Perturbation proof #2 — bump the disc radius (`discRadius` → `r+1`):**
```
render_golden_test.go:89: PNG decoded-pixel hash drift: got 020ddbd6… want 5d7d153b…
render_golden_test.go:92: PNG black-pixel mass drift: got 76049 want 62702
```
PNG pixel-hash + black-mass go RED (76049 matches the impl-log's r+1 value); **the SVG hash + M/C counts
stayed GREEN**. Reverted → clean. The two goldens are independent and each has real teeth.

**Determinism — no flake risk.** `-update` regenerates the EXACT pinned constants; the golden test passes
3× (`-count=1`, uncached); my probe rendered `renderPNG` 5× → byte-identical `Pix` hash (= the pinned
golden). The render path has no map iteration / time / rng (confirmed by inspection + R0 §3): `engraveBest`
→ `qr.Encode` deterministic, `PlanEngraving` integer B-spline timing, `bezier.Sample` pure integer,
`renderSVG` a `%d` `fmt.Fprintf` stream. No nondeterminism.

## 3. Flag #1 (M/C count form: `"M "`/`"C "` vs spec's `" M "`/`" C "`) — implementer is CORRECT, not a defect

Empirically dissected the actual MD1_REF SVG:
- letter+space (implementer): **M=2132, C=2578** — matches `mCountGolden`/`cCountGolden` exactly.
- space-letter-space (spec's literal): **M=2131, C=2578** — undercounts M by exactly 1.
- False `"M "`/`"C "` matches OUTSIDE the `d="…"` payload: **0 / 0** (coordinates are `%d` digits/`-` only;
  no uppercase C/M in the header/attrs — "viewBox" has a capital B, not C/M).
- The `d` payload's first command is **"M"**, whose leading space `strings.TrimSpace` (render_svg.go:43)
  strips — so ` M ` (space-required) misses it. Hence the spec's literal form undercounts by exactly 1.

The letter+space form counts commands **exactly and unambiguously** and is equally-teethed (perturbation
#1 shows the M/C counts swap on a pen-state flip). The deviation is a genuine correction of a
spec-literal off-by-one, and the in-test comment (render_golden_test.go:54-57) documents the rationale
accurately. Not a defect.

## 4. Flags #2–#4

**Flag #2 — `renderHairline` faithful mirror (ratio honest, not gamed).** Compared the walk
(render_png_test.go:105-147) against `renderPNG`: identical `PlanEngraving` iteration, identical
`dt==0 || !line` skips, identical `bezier.Sample`/`toPx`/`spacing`. It differs ONLY in (a) stroke fn
(1px `strokeHairline` vs disc) — the intended measured difference — and (b) the ungrown base canvas.
I proved (b) changes nothing: rendering the hairline on the SAME grown+shifted canvas yields the
**same black count (12919)** as the ungrown canvas — no centerline pixel is clipped, white margin adds
zero black. So the 4.85× ratio is honest; the ≥2× floor is not gamed.

**Flag #3 — Route A's `*image.RGBA` assertion is safe for ALL inputs.** The canvas is unconditionally
opaque: the white fill sets EVERY byte (incl. alpha) to `0xff` (render_png.go:66-68) and black is
`A=0xff`. `png.Encode` of an opaque `*image.RGBA` selects truecolor `cbTC8` (alpha dropped), which
`png.Decode` returns as `*image.RGBA`. Opacity is input-independent, so the assertion can never hit the
`*image.NRGBA` path. Probe confirms `Opaque()==true`, decoded type `*image.RGBA`.

**Flag #4 — "82 vs 90" is branch-isolation, NOT a lost test.** `git grep '#[test]'` counts: **master =
82, branch (Cycle B) = 82** (and `git diff master..HEAD -- '*.rs'` = 0 lines — Cycle B changes NO Rust
file, so its Rust suite is identical to master by construction), while **`me-preview-hardening` (Cycle A /
PR #1) = 90** (it added 8 Rust tests). The 82-vs-90 gap is purely Cycle B branching off master before
Cycle A's Rust tests; no test was deleted. `cargo test` runs 28 of the 82 markers (the rest are
feature/cfg-gated), all green, identical on both refs.

## 5. Scope + suites

- **Scope CLEAN.** `git diff --name-only master..HEAD`: `render_png.go` + `render_png_test.go` +
  `render_golden_test.go` + `SPEC_me_preview_render_goldens.md` + three agent-report `.md`. No
  `render_svg.go`/`params.go`/`layout.go`/`main.go`/CLI change, no `.rs` change. B1 production surface is
  a single file.
- **Suites:** `go vet` clean; `go build ./...` OK; `go test ./...` (preview) PASS incl. the new
  `TestDiscRadiusMapping`/`TestDiscRadiusFloor`/`TestPNGStrokePixelMass`/`TestRenderGoldens`;
  `ME_REQUIRE_GO=1 cargo test --locked` = 28 passed / 0 failed.

---

## Findings

### Critical
None.

### Important
None.

### Low
None.

### Nit

**N1 (informational, no action) — `discRadius` is unbounded above; at `scale=1` it returns 960.**
`scale` is clamped to ≤1 (never upscale), and `scale=1` requires the drawing's longest side ≤ 1000
machine units (≈0.156mm, smaller than the 0.3mm stroke itself). `radius = round(1920/2) = 960` would then
paint an absurd blob. This is **unreachable for any real md1/mk1**: `engraveBest` lays out at a fixed card
scale, so real bounds are hundreds of thousands of machine units (MD1_REF = 431224 → scale 0.00232 →
radius 2). The math still stays in-bounds even at radius 960 (no panic/clip/negative-index), and this is a
faithful implementation of the binding fable formula. No cap is warranted; flagged only for completeness.

**N2 (cosmetic) — stray `</content>` tag at the end of the impl log.**
`design/agent-reports/me-preview-render-goldens-impl-log.md:62` ends with a spurious `</content>` line (a
copy artifact). Harmless documentation blemish; trim if touching the file.

---

## Decision

**GREEN (0C / 0I).** The R0 folds (I1 decoded-pixel hash, L1 Bresenham-step stamping, N1 bounds-checked
stamp) are correctly implemented and independently verified; both goldens have real, decoupled teeth
(two perturbation proofs); the render path is deterministic (5× + 3× + `-update` round-trip); all four
implementer flags are sound (flag #1 is a correct off-by-one fix, not a defect); scope is clean and all
suites are green. No Critical or Important open. The controller may open the PR.
