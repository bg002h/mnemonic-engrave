# `me bundle --preview` Phase B — SPEC architect review — R1 (convergence)

- **Stage:** R1 convergence of the mandatory spec R0 gate. Re-dispatch after the R0 fold.
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect` (adversarial; all claims re-verified in pinned source).
- **Scope:** folded `design/SPEC_me_bundle_phaseB_preview.md` (§4.2/4.3/4.4/7) vs the R0 findings, re-checked against `bezier/bezier.go`, `bspline/bspline.go`, `gui/gui_test.go`, `cmd/controller/platform_sh2.go`, `driver/tmc2209/tmc2209.go`.
- **Verdict:** **GREEN — 0 Critical / 0 Important.** Spec R0 gate converged; spec is implementation-ready.

---

## VERBATIM REVIEW OUTPUT

### Per-finding resolution

**I-2 — RESOLVED.** §4.4 now: "Emit each pen-down `bezier.Cubic` as an SVG cubic-Bézier path directly from its exported control points `C0,C1,C2,C3` (`bezier/bezier.go`): `M C0.x C0.y C C1.x C1.y C2.x C2.y C3.x C3.y`. This is exact (no sampling/faceting)… Sampling is acceptable for the raster path; the SVG remains the exact/authoritative artifact." Field-name confirmation: `bezier/bezier.go:7–9` `type Cubic struct { C0, C1, C2, C3 Point }` — exact. Direct emission compiles from an external package. No residual polyline-for-SVG language. "exact/faithful" now internally consistent.

**I-1 — RESOLVED.** §7 go.mod note accurately describes `bspline/optimize.go` importing `gonum` unconditionally → `go mod tidy` adds `gonum v0.17.0` indirect, ~tens-of-MB binary, accepted trade-off (sidecar never uses `InterpolatePoints`).

**m-1 — RESOLVED.** §4.4 now: `seg.Knot(k)` returns `(bezier.Cubic, ticks uint, engrave bool)` — third value is the pen-down flag. Confirmed `bspline.go:47` + `:52` (`engrave := s.Knots[1].Engrave`).

**m-2 — RESOLVED.** §4.4 notes `Measure` returns a conservative convex-hull bbox; `bspline.go:30–44` confirms min/max over four control points.

**m-3 — RESOLVED.** §4.4: skip `engrave == false` (pen-up); the `C0=={0,0}` window-priming segments are pen-up by construction → caught by the same guard.

**m-4 — RESOLVED.** §4.3 gives concrete values + "`TicksPerSecond == Speed == topSpeed = 30·mm`… comment it in `params.go`." `gui/gui_test.go:336–359`: `mm=6400`, `strokeWidth=0.3*mm=1920`, `topSpeed=30*mm=192000`, `engravingSpeed=8*mm=51200`, `acceleration=250*mm=1600000`, `jerk=2600*mm=16640000`, `TicksPerSecond: topSpeed, Speed: topSpeed` — every number exact. Canonical `platform_sh2.go:394–406` identical; `tmc2209.Microsteps = 1<<8 = 256` → `mm = 200/8*256 = 6400`.

**Plate-fit fold — RESOLVED.** §4.2 exact `toPlate` replica (85×85mm, 3mm margin, `bounds.In(Min=3mm,Max=82mm)`, mm=6400) via public `PlanEngraving`+`Measure`+`Bounds.In`, no `gui` import. `Bounds.In` exported (`bspline.go:160`).

### Cross-checks
1. **Consistency:** §4.2 fit-check and §4.4 SVG both call the same `engrave.PlanEngraving(params.StepperConfig, eng)` with §4.3 params — consistent. The per-segment `M C0 C C1 C2 C3` (fresh `M` per pen-down cubic) is valid SVG and handles pen-up gaps naturally. No new contradiction.
2. **`bezier.Cubic` fields:** `C0,C1,C2,C3 Point`, all exported — spec formula compiles externally. NOT a NEEDS-CHANGE.
3. **Param values:** match `gui_test.go` exactly (host-compilable anchor, no tinygo gate).
4. **Regression:** edits confined to §4.3/4.4/7; all six CRUX groups (PlanEngraving, bspline.Segment.Knot, Measure, EngraveText/Text/Paragraph, qr.Encode/qr.L, host-portability, me-integration backward-compat) untouched and still hold. Scope/security unchanged (ms1 never rendered, no network, no secrets).

## Issues
### Critical
None.
### Important
None.
### Minor
None.

## Verdict: GREEN — 0 Critical / 0 Important
All seven R0 findings RESOLVED; folds corroborated against pinned source; no new contradictions/regressions. Cleared to implementation.

---

## Loop summary (added by main session)
| Round | Verdict | Folded |
|---|---|---|
| R0 | NOT-GREEN | 2 Important (I-1 gonum acknowledgment; I-2 SVG cubic-Bézier vs polyline) + 5 Minor (m-1 engrave-flag label, m-2 conservative bbox, m-3 skip pen-up, m-4 TicksPerSecond comment, m-5 gonum §7) + plate-fit constants folded |
| **R1** | **GREEN** | none — converged |

Spec R0 gate GREEN. All render/fidelity APIs verified in source (PlanEngraving → bspline.Segment → direct cubic-Bézier SVG; replicated SH2 params with a `gui_test.go` host-anchor + geometry-golden drift guard; host-reachable plate-fit). Next: user spec-review gate → writing-plans (Phase B plan) → plan R0 → subagent-driven impl (+ the deferred `me-preview-release-ci` follow-on).
