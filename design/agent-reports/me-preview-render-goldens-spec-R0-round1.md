# R0 architect review — SPEC_me_preview_render_goldens.md — Round 1

- **Spec:** `/scratch/code/shibboleth/me-cycleB/design/SPEC_me_preview_render_goldens.md`
- **Worktree/branch:** `me-cycleB` @ `me-preview-render-goldens` (off mnemonic-engrave master `9fafb6b`)
- **Scope closed by spec:** F13 (`me-preview-png-stroke-width`) + F15 (`me-preview-render-goldens`), Cycle B.
- **Binding prior decision (NOT relitigated):** fable 2026-07-06 — deterministic integer disc-brush
  stroke FIRST (B1), then pin goldens over the corrected output (B2); pinning first would force an
  immediate re-baseline. This review verifies **closure** of round-0 (0C/1I/3L/4N, all folded) and
  re-reviews the whole spec fresh for fold-induced drift.
- **Reviewer:** opus R0 architect. Standard: GREEN = 0 Critical / 0 Important.

**VERDICT: GREEN — 0 Critical / 0 Important (+ 2 Low, 3 Nit).**

The blocking round-0 **I1 is substantively CLOSED**: §B2 and Open-Question 4 now pin the SHA-256 of the
**decoded RGBA pixel buffer** `img.Pix` and explicitly reject the compressed-PNG-byte hash — the
toolchain-fragility landmine is removed at the operative-test level. L2/L3/N1/N3/N4 are genuinely
addressed in the spec text (not paraphrased). Two residual **doc-consistency contradictions survive
the fold** (both Low, both one-line deletions): a stale B1 rationale line still says "B2 pins the PNG
byte hash" (contradicts §B2), and the "Implementation shape" paragraph still says "at each polyline
sample point" (contradicts the emphatic "each Bresenham step, NOT per-sample-point" directive round-0
L1 asked to delete). Neither blocks GREEN — in both cases the *operative* instruction (the section that
actually defines the code/test) is correct and emphatic, and a single implementer following §B1/§B2
implements correctly; the stale phrases live in secondary/rationale prose. Fold the two Lows + Nits
inline (controller-fold territory — all trivial edits); no re-dispatch gate is required to reach code.

---

## Verification performed (evidence base)

Read from the worktree: the spec; round-0 review (`…-spec-R0-round0.md`); `design/FOLLOWUPS.md` (both
`me-preview-*` entries incl. the verbatim disc-brush decision + ordering clause); `preview/render_png.go`
(drawLine :94, the guarded `SetRGBA` :108-110, scale :38, w/h :43-44, toPx :54-58, spacing :61),
`render_svg.go` (single `%d`-formatted `<path>`, `stroke-linecap/linejoin=round` :42), `params.go`
(`strokeWidth=1920`, `mm=6400`), `render_test.go`, `params_test.go` (`MD1_REF`, `wantDx=431224`/
`wantDy=200868` geometry golden), `layout.go` (`engraveBest` → text+qr). Git: branch at `9fafb6b`,
spec + round-0 review untracked (clean fold, no stray edits).

Recomputed the default-render numbers to re-confirm the recon: `scale = 1000/431224 = 0.0023190`,
honest stroke `1920*scale = 4.45 px`, `strokeWidth*scale/2 = 2.226 → round = 2`, `radius = max(1,2) = 2`,
effective width `2r+1 = 5 px`, single-disc pixels `dx²+dy²≤4 = 13`. All consistent with round-0.

**I1 dual-route equality — verified analytically** (Go unavailable in this env; reasoned to certainty):
the canvas is fully opaque (`img.Pix` init to `0xff` incl. alpha; black stroke `A=0xff`). `png.Encode`
of an **opaque** `*image.RGBA` selects truecolor (cbTC8, alpha channel dropped); `png.Decode` of cbTC8
allocates a fresh `*image.RGBA` with `A=0xff`. Encode/decode of 8-bit truecolor is lossless, so the
decoded `Pix` (`[R,G,B,0xff]…`, `Stride=4w`) is **byte-identical** to the pre-encode `Pix`. Therefore
Route A (`png.Decode` → assert `*image.RGBA` → hash `Pix`) and Route B (hash `img.Pix` before encode)
produce the **same** golden constant, and both are toolchain-stable because flate *decompression* is
exact regardless of the encoder's compression/filter heuristics. I1's fix is sound.

---

## 1. I1 closure (the round-0 gate-blocker) — CLOSED (with one residual Low)

- **§B2 (spec:84-90) now pins the DECODED pixel buffer, not compressed bytes.** Verbatim: "pin the
  SHA-256 of the **decoded RGBA pixel buffer** `img.Pix` … NOT the compressed PNG file bytes.
  `image/png` + `compress/flate` output can drift across Go toolchain versions for identical pixels …
  Hashing `img.Pix` has the same regression teeth (any pixel change fails) and is toolchain-stable."
  This is the exact fix round-0 mandated, stated at the operative-test level. **Open-Question 4
  (spec:122-123) agrees**: "PNG golden → **decoded `img.Pix` hash** (I1)." Two authoritative places
  concur.
- **Toolchain-stability holds** (see evidence base): decoded pixels are invariant to compression
  heuristics; the SVG hash is over a pure `%d`-integer string stream (`render_svg.go:33,35,40,42` — no
  floats, no flate, no timestamp/rng/map iteration), so **full-SVG SHA-256 is toolchain-stable** too.
  Confirmed no residual reliance on compressed-byte hashing **in the operative sections**.
- **RESIDUAL (L2 below):** one *rationale* sentence in B1 (spec:49) still reads "Determinism is
  load-bearing: **B2 pins the PNG byte hash**." That is now factually wrong (B2 pins the *pixel*
  buffer) and is precisely the unqualified byte-hash claim round-0 I1 said "should not be shipped."
  It is a secondary motivational aside (explaining *why* B1 must be integer/no-AA), not the test
  definition — hence Low, not a re-opened Important — but it must be corrected in the fold.

## 2. L1–L3 + N1–N4 closure

- **L1 (stamp per Bresenham step) — SUBSTANTIVELY closed, deletion NOT done → residual Low.** §B1
  (spec:42-46) is now emphatic and unambiguous: "Stamp a precomputed integer disc … at **each
  Bresenham step** inside `drawLine` (R0 L1: replace the single `SetRGBA` at render_png.go:109 with a
  `stampDisc(…)` call — per-step, NOT per-sample-point; a sample-point-only stamp has a 1px diagonal
  pinch at the `radius=1` floor)." Good — this pins the placement at the `:109` SetRGBA site. **BUT the
  "Implementation shape" paragraph (spec:60) STILL says** the helper is "called **at each polyline
  sample point**" — the exact phrase round-0 L1 said to *delete*. Live self-contradiction (L1 below).
  Both placements yield a correct thicker stroke and a self-consistent golden (the golden is generated
  over whatever B1 emits), and at the default `radius=2` both are gap-free, so no material defect — but
  the round-0-requested deletion was skipped. Low.
- **L2 (name the hairline baseline) — CLOSED.** §B1 (spec:68-71) now names it concretely: "assert the
  count ≥ 2× a **hairline baseline** … obtained by keeping the OLD 1px `drawLine` reachable as a
  test-only helper `strokeHairline` (**or** pinning a measured constant with a comment); 'radius=0-
  equivalent' is imprecise since a disc's minimum is a 3px plus, so name the baseline explicitly." The
  imprecise "radius=0" language is corrected and a concrete source is given. Executable via the
  **pinned-constant** branch (measure the current pre-B1 hairline black count once, record as a
  commented `const`), which is fully self-contained and gives the clean RED-first (`hairline ≥ 2×
  hairline` is false today → GREEN after B1 at ~5×). *Note (Nit N3):* the `strokeHairline`-helper
  branch is the more awkward of the two — a full hairline *render* (not just a line helper) is needed
  to get a baseline count, which `renderPNG` doesn't expose; prefer the pinned constant.
- **L3 (golden storage) — CLOSED.** §B2 (spec:94-98): "hashes and M:C counts as in-test hex/int
  constants; `testdata/` only if raw bytes are stored." Matches the round-0 disposition exactly.
- **N1 (bounds-checked stampDisc) — CLOSED.** §B1 (spec:47-48): "`stampDisc` MUST bounds-check every
  pixel via `img.Bounds()`/`SetRGBA` (R0 N1) — never raw `img.Pix[...]` indexing — so a disc
  straddling the canvas edge cannot panic." This mirrors the existing guarded `SetRGBA` pattern at
  `render_png.go:108-110`. Explicit and correct.
- **N2 (5px vs 4.45px) — inherently closed** (round-0 marked it "no change needed," informational).
- **N3 (register `-update` once) — CLOSED.** §B2 (spec:96-97): "Register the `-update` flag ONCE at
  package level (`var update = flag.Bool("update", false, …)`; R0 N3) — the SVG-hash and PNG-pixel-hash
  tests share it." No `-update` exists elsewhere in `preview/` today (no duplicate-registration panic).
- **N4 (pin exact disc pixel count) — addressed, with a mild ambiguity → Nit.** §B2 (spec:90): "Also
  pin the exact disc pixel count as a cheap invariant (R0 N4)." Round-0's intent (§4 N4) was the
  **total black-pixel MASS** of the default render as a fine exact anchor; the phrase "exact disc pixel
  count" could be misread as the single-disc count (13 at `radius=2`). Both are valid "cheap
  invariants," but they differ in teeth. Clarify (Nit N2). Either way it is pinned **for the DEFAULT
  MD1_REF render only** and is an intentional drift-guard exactly like `wantDx/wantDy`
  (`params_test.go:33-36`): a `third_party/seedhammer` submodule bump that moves the geometry (hence
  `scale`, hence `radius`) legitimately fails it and forces a re-baseline. No cross-bump-invariance is
  claimed, so N4's radius-dependence is not a hazard.

## 3. Fresh pass — fold-induced drift

- **Line-49 vs §B2 (I1 residual)** and **line-60 vs §B1:42-46 (L1 residual)** are the two live
  contradictions, both from partial folds (L2 and L1 below). Both are in rationale/"shape" prose; the
  operative sections are correct.
- **Can `strokeHairline` (test-only old 1px path) coexist with a modified `drawLine`? YES — no
  conflict.** B1 modifies `drawLine` *in place* (keeps its Bresenham stepping; swaps the `:109`
  `SetRGBA` for a per-step `stampDisc`). The hairline baseline needs the *old* 1px behavior, which is a
  **separate** test-only helper `strokeHairline` (a copy of the pre-B1 `drawLine` body). They are
  distinct functions, so both compile and coexist. (The only awkwardness — getting a *whole-render*
  hairline count out of a line helper — is why the pinned-constant branch is preferable; Nit N3.)
- **Full-SVG hash interactions — none.** `renderSVG` (`render_svg.go`) has no timestamp/rng/map
  iteration and formats every coordinate with `%d`; `viewBox` and `d` are pure integer streams. The
  full-SVG hash and the ` M `/` C ` token counts (`strings.Count` over space-delimited literals; coords
  are digits/`-` only, no letters) are clean and mutually reinforcing. The perturbation teeth
  (swap `if line`; drop `dt==0`; change `radius`) all bite as the spec claims.
- **N4 radius/submodule-bump — handled** (see N4 above): pinned for the default render only, a
  drift-guard by design, consistent with the co-located `wantDx/wantDy` golden.

## 4. Executability (single implementer) + TDD integrity + ordering

- **Ordering B1→B2 intact** (spec:110 "B1 (stroke fix) FIRST, its tests green, THEN B2 (pin goldens
  over the corrected bytes)"). Faithful to the fable no-re-baseline decision.
- **B1 executable:** extract pure `discRadius(scale) int` (`strokeWidth` is a package const, so the fn
  is a pure function of `scale`); add bounds-checked `stampDisc`; thread `radius` into `drawLine` and
  replace the `:109` `SetRGBA` with the per-step stamp; grow `w`/`h` by `+2*radius` and shift `toPx` by
  `+radius` (round-0 verified this exactly closes the edge-clip with no negative index). The
  white-fill, `spacing`, and `scale` logic are untouched; canvas growth adds only *white* margin so the
  black-pixel mass comparison is unaffected.
- **TDD fail-first is correct for the right reason:** `discRadius`/1px-floor unit tests are
  compile-RED (helper absent) → GREEN; the pixel-mass test is RED today (the current output *is* the
  hairline, so `count ≥ 2×hairline` is false) → GREEN after B1. The B2 goldens are pin/characterization
  tests (they lock corrected output, not fail-first), and the spec correctly substitutes the
  **perturb-then-revert** drift-guard proof to demonstrate teeth (spec:104-105) — the right TDD posture
  for a golden.
- **No existing test breaks under the grown canvas:** `TestRenderPNGValidHeader` (decode + non-empty
  bounds), `TestRenderPNGToFile` (decode only), `TestParamsGeometryGolden` (measures `PlanEngraving`,
  not the raster), `TestRenderSVGContainsExpectedStructure` (SVG-only) — none pins PNG dimensions.

## 5. Scope — CLEAN

B1 edits `render_png.go` only; B2 adds `render_golden_test.go` (+ `testdata/` only if raw bytes).
`render_svg.go`, `main.go`/sidecar CLI contract, `PlanEngraving`/layout, and the whole Rust side are
untouched. Non-goals (spec:28-33) explicitly exclude F8/F9/F10 (Cycle A/PR #1), F11 (sidecar
discovery), F18 (fuzz). Rust-primary rule N/A (the PNG renderer is fork-native; upstream `golden.go`
is SVG-only). No bleed.

---

## Findings

### Low

**L1 — Residual self-contradiction on disc-stamp placement (round-0 L1 not fully folded).** §B1 bullet
(spec:42-46) mandates stamping "at **each Bresenham step** … per-step, **NOT** per-sample-point," but
the "Implementation shape" paragraph (spec:60) still says the helper is "called **at each polyline
sample point**." Round-0 L1 explicitly asked to *delete* this phrasing; it survives. Both placements
are gap-free at the default `radius=2` and the golden self-adapts, so no material defect — but it leaves
an implementer a contradictory choice in a security-adjacent artifact. *Fix:* change spec:60 to "…
called at each **Bresenham step** inside `drawLine` (replacing the `SetRGBA` at render_png.go:109) …"
so both places agree.

**L2 — Residual I1 drift: a B1 rationale line still claims "B2 pins the PNG byte hash" (spec:49).**
This contradicts §B2 (spec:84-90: "pin the decoded RGBA pixel buffer … NOT the compressed PNG file
bytes") and Open-Question 4, and is exactly the unqualified byte-hash claim round-0 I1 said should not
ship. It is a motivational aside (why B1 must be integer/no-AA), not the test definition, so the
operative fix stands — but correct it. *Fix:* spec:49 → "Determinism is load-bearing: B2 pins the
**decoded-pixel (`img.Pix`) hash**, so a per-platform AA/float rounding difference must not leak into
the pixels." Also soften spec:110 "corrected bytes" → "corrected **output**/pixels" for consistency.

### Nit

**N1 — Helper naming inconsistency: `stampDisc` vs `drawDisc`.** The helper is `stampDisc` at spec:44,
48, 58 but "`drawDisc` must clip to `img.Bounds()`" at spec:56. Pick one name (`stampDisc`) throughout.

**N2 — Clarify what N4 pins (single-disc 13 vs total black mass).** "exact disc pixel count" (spec:90)
is ambiguous; round-0's intent (§4 N4) was the **total black-pixel count of the default MD1_REF
render** as a fine exact anchor alongside the pixel hash. State it as "pin the exact **total black-pixel
count** of the default MD1_REF render (a drift-guard like `wantDx/wantDy`; expected to change on a
`third_party/seedhammer` submodule bump)."

**N3 — PNG pixel-hash route + hairline-baseline route: state the cheaper choice.** For the PNG golden,
**Route A** (`renderPNG` → `png.Decode` → type-assert `*image.RGBA` → hash `Pix`) needs **no production
seam** and stays within B1 scope; it yields the identical golden to Route B (hash `img.Pix` before
encode) for this all-opaque image (verified analytically). Prefer Route A, and have the test assert the
concrete `*image.RGBA` type (opaque RGBA decodes to `*image.RGBA` under current Go). Symmetrically, for
the pixel-mass baseline (L2/§B1), prefer the **pinned measured constant** over the `strokeHairline`
helper (the helper needs a whole hairline *render*, which `renderPNG` doesn't expose).

---

## Open-question dispositions (confirmed unchanged from round 0)

1. Disc offset-set vs inline `dx²+dy²≤r²`: **either** (both deterministic); MUST bounds-check (N1,
   closed). Placement pinned to **Bresenham-step** (fix the spec:60 residual, L1).
2. Canvas `+2*radius` + `+radius` shift: **correct** — grow, do not clip-only.
3. `testdata/` vs in-test consts: **in-test hex/int consts for hashes/counts; `testdata/` only for raw
   bytes** (closed).
4. Full-SVG hash + separate M:C counts; PNG golden = **decoded `img.Pix` hash** (I1, closed at the
   operative level).
5. Pixel-mass threshold **≥2×** (actual ~5×): sound, non-flaky floor.

---

## Decision

**GREEN (0C / 0I).** The round-0 Important (I1) is closed at the operative-test level (§B2 + OQ4 pin the
decoded `img.Pix` hash; toolchain-stability verified). L2/L3/N1/N3/N4 are genuinely folded. Two residual
one-line contradictions remain (L1 = spec:60 "polyline sample point"; L2 = spec:49 "PNG byte hash") plus
3 Nits — all trivial doc edits; per the CLAUDE.md tight-implementation convention the controller should
fold them **inline** in the worktree (no fresh agent, no re-dispatch gate) before/at implementation. No
Critical/Important open → implementation may proceed.
