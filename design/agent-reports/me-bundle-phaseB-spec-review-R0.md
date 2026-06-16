# `me bundle --preview` Phase B — SPEC architect review (R0 gate)

- **Stage:** mandatory spec R0 gate before any code (project standard). Spec promoted from design §B.
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect` (adversarial; all CRUX render/fidelity claims verified in pinned-source).
- **Scope:** `design/SPEC_me_bundle_phaseB_preview.md` vs seedhammer v1.4.2 ref (`backup`/`engrave`/`bspline`/`bezier`/`font`), fork `6ab12c0` (`validateMdmk`/`toPlate`), `kortschak-qr v0.3.2`, the existing `me`.
- **Verdict:** **NOT-GREEN — 0 Critical / 2 Important / 5 Minor.** All 6 CRUX API/fidelity claims CONFIRMED implementable; 2 Important are local spec-text fixes.

---

## VERBATIM REVIEW OUTPUT

[Preamble — gonum transitive dep:] The upstream `go.mod` includes `gonum.org/v1/gonum v0.17.0` as a direct dependency, because `bspline/optimize.go` imports it unconditionally (no build tag). When the sidecar's `go.mod` uses `replace seedhammer.com => ../../../seedhammer-ref-v1.4.2`, it inherits all these dependencies including `gonum`. NOT a blocker (Go handles transitive deps via `go.sum`) but inflates the sidecar's dependency set and binary size (~tens of MB). The sidecar only uses `bspline.{Segment,Knot,Curve,Measure,Bounds}`, never `InterpolatePoints`. Security: the sidecar receives only the PUBLIC string via stdin; `me` never passes `ms1`; no network/env-secrets/filesystem beyond `--out`. CONFIRMED. Scope: deferring `me-preview-release-ci` is sound — v0.3.0 is fully testable with a locally-built sidecar; runtime contract (co-located, version-matched) is complete. `AsKnot()` returning unexported `splineKnot` is moot — the spec uses `PlanEngraving`, not `AsKnot`.

## API/fidelity verification

### CRUX 1a — `engrave.PlanEngraving` — CONFIRMED `engrave/engrave.go:969`: `func PlanEngraving(conf StepperConfig, e Engraving) bspline.Curve`. `Engraving = iter.Seq[Command]` (:55).

### CRUX 1b — `bspline.Curve`/`Knot` — CONFIRMED `bspline.go:22` `type Curve = iter.Seq[Knot]`; `Knot{ Ctrl bezier.Point, T uint, Engrave bool }` (:24-28). Exact match.

### CRUX 1c — `bspline.Segment.Knot` + external `bezier.Cubic` sampling — CONFIRMED with spec imprecision. `bspline.go:47`: `func (s *Segment) Knot(k Knot) (bezier.Cubic, uint, bool)`. The third return is NOT "validity" — it is the `engrave` (pen-down) bool. `bezier.Cubic` (`bezier/bezier.go:7`) has EXPORTED `C0,C1,C2,C3 bezier.Point`. ⇒ External pkg can emit SVG cubic Bézier `C` commands DIRECTLY (exact), no sampling. The spec's "de Casteljau / N steps polyline" is suboptimal/inconsistent with "exact". `bezier.Sample` (:442) also public.

### CRUX 1d — Does `PlanEngraving` preserve cut geometry? — CONDITIONALLY CONFIRMED (params-dependent). `planEngraving` (:978-1039) time-weights knots via `bspline.ComputeKinematics` + StepperConfig; `bspline.Segment.Knot()` uses `T` in Böhm's algorithm → intermediate C1/C2 depend on `T`. Line segments stay exactly straight regardless of T; glyph CURVES depend on T. ⇒ **With correct params, the rendered curve exactly matches the device's cut; with wrong params, glyph curves deviate.** The spec's use of `PlanEngraving` (not a geometry-only bypass) is CORRECT — it makes render AND fit-check use the same curve the device uses. Keep as-is.

### CRUX 1e — `bspline.Measure` bounds — CONFIRMED `bspline.go:194` `func Measure(spline Curve) Attributes`; `Bounds{Min,Max bezier.Point}` (:157) + `Dx()`/`Dy()`. Convex-hull (conservative) bbox — viewBox slightly larger than content; acceptable.

### CRUX 2 — SH2 `engrave.Params` — CONFIRMED from two sources. Canonical `cmd/controller/platform_sh2.go:394-407` (TinyGo-gated): `engraverConf = StepperConfig{ TicksPerSecond: topSpeed, Speed: topSpeed, EngravingSpeed: engravingSpeed, Acceleration: acceleration, Jerk: jerk }`, `engraverParams = Params{ StrokeWidth: strokeWidth, Millimeter: mm, StepperConfig: engraverConf }`; `mm = 200/8 * 256 = 6400`. Host-compilable cross-check `gui/gui_test.go:336-359` (v1.4.2 ref) — IDENTICAL: `mm=6400, strokeWidth=0.3·mm=1920, topSpeed=30·mm=192000, engravingSpeed=8·mm=51200, acceleration=250·mm=1600000, jerk=2600·mm=16640000, TicksPerSecond=192000`. `Params` fields used by `EngraveText`/`PlanEngraving`/`F`/`I` are exactly `StrokeWidth`/`Millimeter`/`StepperConfig`. Geometry-golden drift guard is SOUND (param change → T → Segment.Knot output → Measure bounds).

### CRUX 3 — Layout APIs — ALL CONFIRMED: `backup.EngraveText(engrave.Params, Text) engrave.Engraving` (backup.go:252); `backup.Text{Paragraphs []Paragraph, Font *vector.Face}` (:32); `backup.Paragraph{Text string, QR *qr.Code, QRScale int}` (:37); `font/sh.Font` is `*vector.Face` (sh.go:10); `qr.Encode(string, qr.Level) (*qr.Code, error)` + `qr.L` (fork gui.go:1747).

### CRUX 4 — Plate-fit / dims / mode-selection — CONFIRMED host-reachable. `toPlate` IDENTICAL in fork `gui/gui.go:2537-2550` AND upstream ref `gui/gui.go:2471-2479`: plate `85×85 mm`, `3 mm` margin, fit = `attrs.Bounds.In(bspline.Bounds{Min: bezier.Pt(3*mm,3*mm), Max: bezier.Pt(82*mm,82*mm)})`. Pure arithmetic constants; sidecar replicates `mm`/`85`/`3` and uses public `PlanEngraving`+`Measure`+`Bounds.In` — NO `gui` import. Mode loop (TEXT+QR→TEXT→QR) is plain iteration. CONFIRMED.

### CRUX 5 — Host portability — CONFIRMED. No `//go:build` in `backup/backup.go`, `engrave/engrave.go`, `bspline/bspline.go`, `bezier/bezier.go`, `font/sh/sh.go`, `font/vector/font.go`. Only `bspline/optimize.go`→gonum (compiles host-side; binary-size only). ndef-roundtrip already proves seedhammer.com host-imports.

### CRUX 6 — `me` integration — CONFIRMED. `std::env::current_exe()` sound; `--version` lockstep + degrade + exit codes (0/2/3/4) consistent with Phase A; `PlateEntry.preview: Option<String>` with `skip_serializing_if="Option::is_none"` is backward-compatible — without `--preview` the field is absent ⇒ Phase A golden stays green.

## Issues

### Critical
None.

### Important

**I-1: `bspline/optimize.go` pulls in `gonum` unconditionally.** The sidecar's `go.mod` (via the upstream `replace`) gets `gonum` as a transitive compile dep → binary inflated ~tens of MB; `go mod tidy` adds it as indirect. Not a compile failure. **Fix:** acknowledge in §7 (go.mod note) as an accepted trade-off (excluding it needs an upstream build-tag change = out of scope; sidecar never uses `InterpolatePoints`).

**I-2: SVG approach inconsistent with the "faithful" claim.** §4.4 said "sample each cubic (de Casteljau / fixed N steps) → polyline" but also "exact B-spline strokes / faithful". `bezier.Cubic.C0/C1/C2/C3` are exported ⇒ SVG cubic Bézier `C` paths are exact, simpler, more compact. Polyline sampling is a fidelity downgrade. **Fix:** emit SVG cubic Bézier directly from `C0..C3` for pen-down segments; reserve sampling for the PNG raster path only.

### Minor
- **m-1:** §4.4 mislabels `seg.Knot()`'s third bool as "validity" — it's the pen-down `engrave` flag. Relabel.
- **m-2:** `bspline.Measure` is a conservative convex-hull bbox — note the viewBox margin.
- **m-3:** skip `engrave==false` (pen-up) segments; the first window-priming segments have `C0=={0,0}` (pen-up, naturally skipped).
- **m-4:** `TicksPerSecond == Speed == topSpeed = 30·mm` is a real SH2 equality — comment it in `params.go`.
- **m-5:** §7 should note `go mod tidy` adds `gonum` indirect (aspect of I-1).

## Assessment
Scope split sound (feature fully testable with a locally-built sidecar; release-CI deferral is packaging-only). Security clean (only the public string reaches the sidecar; no `ms1`, no network). Fidelity holds with a well-defined caveat (faithful **given replicated params at the pinned ref**; geometry-golden guards drift; `gui_test.go` is a host-compilable value anchor). All design §B requirements addressed (I-3 qr.L/scale3 ✓, I-4 version-check ✓, m-5 B-spline ✓). No internal contradictions. PNG-from-scratch raster difficulty noted (use `golang.org/x/image`/`bezier.Sample`). 

## Verdict: NOT-GREEN — 0 Critical / 2 Important
I-1 (gonum acknowledgment in §7) + I-2 (direct SVG cubic Bézier, not polyline). Both local spec-text fixes; no architecture change. Resubmit R1.

---

## Fold plan (main session) — ALL FOLDED
- **I-1 / m-5** → §7 go.mod note (gonum indirect, accepted trade-off).
- **I-2 / m-1 / m-2 / m-3** → §4.4 rewritten: direct cubic-Bézier SVG from `C0..C3`; `seg.Knot` third value relabeled pen-down flag; skip `engrave==false`; conservative-bbox note; PNG uses sampling/raster only.
- **m-4** → §4.3: explicit `TicksPerSecond==Speed==topSpeed` comment requirement + the concrete param values + the `gui_test.go` host-anchor.
- Plate-fit constants (CRUX 4) folded into §4.2; concrete params (CRUX 2) into §4.3.
Re-dispatch R1 to converge.
