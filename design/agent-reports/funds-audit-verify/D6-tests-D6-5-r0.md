# Verifier #0 — D6-5 adversarial verification

Finding under review: **D6-5** (moderate) — "No preview output-fidelity golden — SVG/PNG
walk mutations render a wrong preview with the suite green (preview-vs-device divergence)".
Location cited: `preview/render_svg.go:29` (dt==0 skip) / `:32` (`if line` pen-state).

Verdict: **NOT refuted on accuracy; severity downgraded moderate → low.**
Confidence: high.

---

## 1. What the finding claims

1. `TestParamsGeometryGolden` (params_test.go:30) measures
   `bspline.Measure(engrave.PlanEngraving(...)).Bounds`, computed independently of
   renderSVG/renderPNG's *walk*.
2. `render_test.go` asserts only structural substrings (`<svg`, `viewBox=`, exactly one
   `<path`, at least one ` C `) and a valid PNG header/decode — no path-content or pixel golden.
3. A pen-state swap (`render_svg.go:32 if line` → `if !line`) or dropping the `dt==0` skip
   (`:29`) produces a garbage path that still satisfies every assertion.
4. Consequence: the preview a user visually approves can diverge from what the device engraves.

## 2. Static verification (code reads as claimed)

- **params_test.go:30** — confirmed: `b := bspline.Measure(engrave.PlanEngraving(params.StepperConfig, eng)).Bounds`.
  The golden compares `b.Dx()/b.Dy()` to `wantDx/wantDy`. It never inspects the SVG/PNG `d`
  attribute or the render loop. renderSVG (render_svg.go:23) recomputes the SAME bounds from
  `PlanEngraving` for the `viewBox`; the per-cubic *walk* that builds the `d` string
  (lines 27-37) is a distinct code path the golden does not touch.
- **render_test.go** — confirmed: only `<svg`, `viewBox=`, `<path`, `Count(<path)==1`,
  `Contains(" C ")`. PNG test only checks magic bytes + `png.Decode` + non-empty bounds.
  No content/pixel golden anywhere in `preview/`.
- The two exact mutation sites in the finding exist verbatim: `render_svg.go` `if dt == 0 { continue }`
  (indented two tabs) and `if line {` (two tabs; the finding's ":32" is correct, and the
  "line 29" for the dt-skip is correct).

## 3. Dynamic probe (mutations escape the suite — live-confirmed)

Copied `preview/*.go` + go.mod/go.sum to `/scratch/d6probe/work` (OUTSIDE the repo;
`replace seedhammer.com` repointed to the absolute third_party path; Go 1.26.4 from
`/home/bcg/.local/go/bin`, GOTMPDIR/GOCACHE on /scratch because /tmp tmpfs was 100% full from
an unrelated project's 31G scratch — not touched). Baseline `go test ./...`: **ok**.

**Mutation 1 — pen-state swap** (`if line {` → `if !line {`, two-tab match verified,
`-count=1` forced rebuild): full suite **ok (green)**.
Dumped the `d` attribute for MD1_REF baseline vs mutation:

| | C-tokens | M-tokens | len(d) | head of d |
|---|---|---|---|---|
| baseline | 2578 | 2131 | 149795 | `M 2465 1153 M 4754 2223 M 20750 9709 …` |
| swapped  | 2131 | 2578 | 137111 | `C 0 0, 0 0, 2465 1153 C 3077 1439, … 4754 2223 …` |

The preview is now strokes-during-pen-up (garbage) with pen-down cubics turned into jumps —
a visibly different render — yet `TestRenderSVGContainsExpectedStructure`,
`TestRenderPNGValidHeader`, and `TestParamsGeometryGolden` all pass.

**Mutation 2 — drop the `dt==0` skip** (guard neutralized to `if dt == 99999`): source dump
confirmed applied; full suite **ok (green)**. `TestParamsGeometryGolden` unchanged (bounds are
measured from `PlanEngraving`, independent of the walk).

(First attempt at mutation 1 used a one-tab sed that silently did not match — the "(cached)
ok" it produced was the UNMUTATED file and is discarded. The re-run above uses a
grep-count-verified two-tab match plus `-count=1`, so the green result is genuinely with the
mutation compiled in.)

Probe dir `/scratch/d6probe` removed after the run. No repo file was modified.

**The finding's factual/mutation-sensitivity claims are fully substantiated.** I cannot refute
it on accuracy.

## 4. Severity assessment — funds impact is bounded; moderate is overstated

The severity question is "would it really produce a wrong-but-accepted plate / lost funds?"
Analysis of the architecture says **no**, for these reasons:

- **The preview is decoupled from the engraved artifact.** `me` converts md1/mk1 → **NDEF**,
  which is what the SeedHammer II reads and engraves; the device computes its engraving
  geometry on-device from that string. `me-preview` is a *separate* Go binary that *replicates*
  SH2 curve math purely to draw an SVG/PNG for the human. A bug in me-preview's render **walk**
  changes only the picture; it cannot change the NDEF the device consumes, so the physical
  plate the device produces is unaffected and still correct.
- **The device data path is separately pinned.** Byte-exact NDEF is guarded by
  `golden.rs::md1_short_matches_golden` + `ndef::encodes_expected_bytes`, and the real
  SeedHammer `nfc/ndef` reader round-trip is guarded by `cross_lang.rs`. me→sidecar
  pass-through (exact input string piped) is guarded by
  `preview::render_plate_writes_file_and_returns_path`. None of these depends on the SVG/PNG
  walk.
- **The escaping mutations self-defeat in the SAFE direction.** A pen-state swap yields an
  *obviously* broken picture (strokes everywhere), which alarms the user rather than falsely
  reassuring them; a dropped stroke yields a slightly-wrong glyph of the *correct* string. The
  dangerous direction a preview must guard against — showing a convincingly-correct picture of
  a *different* string that the device then engraves — is NOT what these mutations produce. At
  restore time the user reads the *physical plate* (correct), not the preview.
- **Comparative calibration.** Sibling moderate D6-4 causes the *real device reader* to
  misparse the engraved string → an actual wrong-but-accepted plate (true funds risk). D6-5's
  worst case is a wrong *preview* of a *correct* plate. Its funds impact is strictly less than
  D6-4's, so if D6-4 is moderate, D6-5 belongs one rung lower.

As a **test-adequacy** observation the gap is real and honestly described (there is genuinely
no path-content or pixel golden, and even the differential `preview_cross_lang.rs` only checks
structure, not content). So the finding stands as a legitimate testing hole — but its
funds-safety severity is **low**, not moderate: a preview-render regression degrades a
human-in-the-loop confidence aid; it does not corrupt the engraved backup.

## 5. Verdict

- refuted: **false** (technical claim confirmed by static read + live mutation probe).
- adjustedSeverity: **low** (moderate overstates funds impact; the preview is a decoupled
  confidence aid, not the device data path, and the device output is separately pinned).
- confidence: **high**.
