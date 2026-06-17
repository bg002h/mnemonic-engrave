# `me bundle --preview` Phase B — PLAN architect review — R1 (convergence)

- **Stage:** plan R1 convergence (post-R0 NOT-GREEN: 1C + 2I + 5 Minor + a license-compliance fold).
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect` (C-1 verified line-by-line vs `internal/golden/golden.go:175-194`; license deps traced via the sidecar import graph).
- **Verdict:** **GREEN — 0 Critical / 0 Important.** Plan R0 gate converged; cleared for implementation.

---

## VERBATIM REVIEW OUTPUT

## Per-finding status

**C-1 — SVG walk — RESOLVED.** Verified against `internal/golden/golden.go:175-194`; the folded Task 4 `render_svg.go` matches the canonical renderer line-by-line:
| golden.go | folded plan |
|---|---|
| `var seg bspline.Segment` (zero-value) | identical |
| `for k := range spline` (bspline.Curve) | `for k := range engrave.PlanEngraving(params.StepperConfig, eng)` — same type |
| `c, dt, line := seg.Knot(k)` | identical |
| `if dt == 0 { continue }` | identical |
| `if line { … " C %d %d, %d %d, %d %d", c.C1,c.C2,c.C3 }` | identical |
| `else { … " M %d %d", c.C3.X, c.C3.Y }` | identical |
| single `<path d="…"/>` accumulation | identical |
Leading-space commands + `strings.TrimSpace` ⇒ `d` starts with `M` (first run's pen-up). C0-implicit invariant preserved (`C C1 C2 C3` carries prior C3 as implicit start → B-spline G1 continuity). Two `PlanEngraving` calls (bounds + walk) safe per R0 CRUX 2. No deviation from canonical.

**I-1 — go.mod v0.0.0 — RESOLVED.** Task 1: `require seedhammer.com v0.0.0` + `replace … ../third_party/seedhammer`, `go 1.25.10`; Step 4 hardened ("do NOT change back to v1.4.2"). Matches `firmware/ndef-roundtrip/go.mod` exactly. Proxy-404 eliminated.

**I-2 — manifest.rs test literals — RESOLVED.** Grep confirms `PlateEntry {` sites: `bundle.rs` 230/255/281/294 (4 constructors) + `manifest.rs` 115/162/171/180 (4 `#[cfg(test)]` literals) = 8 sites. Task 7 now says "Grep `PlateEntry {` across `bundle.rs` + `manifest.rs` and update each" — covers all 8. Compile-break resolved.

**m-1 — RESOLVED.** Task 2 `params_test.go` imports `"seedhammer.com/engrave"`.
**m-2 — RESOLVED.** Task 1 `preview/go.mod` `go 1.25.10`.

**License-compliance addition (Task 11) — RESOLVED (accurate + complete for the sidecar's actual import graph).**
(a) kortschak-qr v0.3.2: BSD-3 — COVERED. (b) gonum v0.17.0: BSD-3 + `THIRD_PARTY_LICENSES/` (12 files) — COVERED (plan over-includes all 12; over-inclusion is safe, not a defect — only `gonum/mat` + `gonum/optimize/convex/lp` are actually linked via `bspline/optimize.go`). (c) seedhammer: Unlicense — COVERED (courtesy note); `font/sh` `sh.bin` is SeedHammer's own monospace (NOT the OFL comfortaa/poppins, which the GUI uses and the sidecar does not import). (d) **No uncovered notice-bearing dep:** the sidecar import graph (`main → backup, bspline, engrave, bezier, font/vector, font/sh, kortschak-qr, gonum/mat, gonum/optimize/convex/lp`) contains ONLY kortschak-qr + gonum (BSD-3, covered) + seedhammer (Unlicense, noted). NOT linked: `golang.org/x/{crypto,image,text,sys}` (BSD-3), btcsuite/decred (ISC), fxamacker/cbor + x448/float16 (MIT) — all in the seedhammer module but unreachable from the sidecar path. No license CONFLICT (MIT ⊕ BSD-3 ⊕ Unlicense all permissive). crates.io `me` publish (pure Rust, no Go linkage) correctly noted unaffected.

**No new contradictions/regressions.** Tasks 3,5,6,8,9,10,12 + the spec-coverage map unchanged. C-1's rewrite consistent with spec §4.4 (exact cubic-Bézier — emitted `C C1 C2 C3` are exact internal control points). Folds touched only Tasks 1,2,4,7,11. CRUX 2/4/5 (R0) remain confirmed.

## Issues
### Critical
None.
### Important
None.
### Minor
None. (gonum `THIRD_PARTY_LICENSES/` over-inclusion is a safe conservative choice; the Go-stdlib BSD-3 point is conventional de minimis.)

## Verdict: GREEN (0 Critical / 0 Important)
All R0 findings (C-1, I-1, I-2, m-1, m-2) resolved; the license-compliance addition is accurate and complete for the sidecar's actual import graph. No new issues. Plan cleared for implementation.

---

## Loop summary (added by main session)
| Round | Verdict | Folded |
|---|---|---|
| R0 | NOT-GREEN | 1 Critical (C-1 SVG walk emitted `M C0 C…` per cubic instead of single-path pen-up-`M C3`/pen-down-`C C1 C2 C3`/skip `dt==0`) + 2 Important (I-1 go.mod `require v1.4.2` → `v0.0.0`; I-2 manifest.rs test `PlateEntry` literals) + 5 Minor; + license-compliance THIRD_PARTY_LICENSES folded into Task 11 |
| **R1** | **GREEN** | none — converged |

Plan R0 gate GREEN. C-1 verified line-by-line vs `internal/golden/golden.go`. Next: subagent-driven implementation in a worktree (release.yml gets a careful read-review; minisign keygen flagged as a one-time maintainer action).
