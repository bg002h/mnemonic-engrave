# `me bundle --preview` Phase B — execution review (two-stage) — R0

- **Stage:** post-implementation two-stage review (spec-compliance + adversarial code-quality), the per-phase gate. Subagent-driven impl on `feat/me-bundle-phaseB-preview` (12 task commits `5e06baa..4b3f0e6`).
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-reviewer` (adversarial; release.yml read-reviewed since CI can't run locally).
- **Verdict:** **NOT-GREEN — 0 Critical / 1 Important / 2 Minor.**

## Findings

### Critical
None.

### Important
**I-1 — `me bundle --preview` render failure exits 2, but spec §6 mandates exit 4.** `main.rs` `wire_previews` mapped a sidecar render failure (`PreviewError::Render`, e.g. a string that fits no plate) to `EXIT_USAGE` (2). Spec §6: "Sidecar render failure … → exit 4." (Version-mismatch / unreadable-version / non-dir-target correctly stay at 2.)

### Minor
**m-1 — `me-preview --out -` mixed the `mode <m>` status line into the stdout payload** (Go sidecar Task 6), so `--out -` stdout wasn't a clean pipeable SVG/PNG.
**m-2 — `release.yml` Go cross-compile had no `go mod download` warm-up** before the 6-target loop — a dep-fetch problem would surface mid-matrix instead of failing fast.

## Stage 1 — spec compliance: PASS (with I-1 the lone exit-code deviation)
All §3–§10 behaviors implemented (sidecar pins upstream via submodule, no gui/cmd-controller, no network/secrets; `--version` lockstep; qr.L/scale3/3-modes/EngraveText/fit; replicated SH2 params + geometry-golden; single-`<path>` exact cubic-Bézier SVG; `--preview` per-public-plate, ms1 never rendered, discovery + degrade; release-CI matrix). The exit-code on render failure (I-1) was the one spec deviation.

## Stage 2 — code quality: the SVG single-`<path>` walk matches `golden.go`; params verified vs `platform_sh2.go`/`gui_test.go`; security spine clean (ms1 never reaches the sidecar — Rust renders only `kind != ms1`; sidecar gets only the public string on stdin; no network); Rust discovery/version/spawn sound; release.yml structurally sound (matrix, CGO_ENABLED=0, submodules, minisign temp-file+`rm` cleanup + password piped, THIRD_PARTY_LICENSES, PR dry-run) modulo m-2; minisign.pub is the real public key; tests non-vacuous (real cross-lang round-trip).

---

## Fold plan — ALL FOLDED
- **I-1** → `main.rs wire_previews`: render-loop `Err` now maps `PreviewError::Render { .. }` → `EXIT_INVALID` (4), `_` (Spawn/IO) → `EXIT_USAGE` (2); doc comment updated; added `render_failure_exit_4` CLI test. Committed `f23d955`.
- **m-1** → fixed by the Go implementer in `fc49844` (`--out -` → `mode` line to stderr; `--out FILE` keeps it on stdout per the Rust caller contract).
- **m-2** → `release.yml`: `go mod download` before the build matrix. Committed `f23d955`.
R1 (fresh reviewer) converged GREEN — see `-R1.md`.
