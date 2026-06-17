# `me bundle --preview` Phase B — execution review — R1 (convergence)

- **Stage:** R1 convergence of the post-implementation two-stage execution review. Re-dispatch after the R0 fold (`fc49844` m-1, `f23d955` I-1 + m-2). Fresh reviewer (also a safety net — independently re-verified the invariants).
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-reviewer` (adversarial).
- **Verdict:** **GREEN — 0 Critical / 0 Important.** Phase B execution gate converged.

---

## VERBATIM REVIEW OUTPUT

## Per-finding status

**I-1 — RESOLVED.** `main.rs` `wire_previews` render-loop `Err` arm: `match e { preview::PreviewError::Render { .. } => EXIT_INVALID, _ => EXIT_USAGE }` — `Render` (sidecar non-zero, e.g. no-fit) → 4 per spec §6; Spawn/IO → 2; version-mismatch (line 252) + non-dir (line 267) stay 2. The `render_failure_exit_4` test (`write_fake_render_fail`: matched version, `render` exits 1) asserts `.code(4)` — non-vacuous (version-match path exercised, then `PreviewError::Render` fires per `preview.rs:164-168`). `mismatched_version_exit_2` still asserts `.code(2)`.

**m-1 — RESOLVED.** `preview/main.go:113-119`: `--out -` → `modeSink = stderr` (clean stdout payload); `--out FILE` → `modeSink = stdout`. The Rust caller always passes a real file path and ignores the captured stdout `mode` (records it for stderr logging only) — no protocol break.

**m-2 — RESOLVED.** `release.yml` cross-compile step runs `go mod download` (working-directory `preview`) before the build loop — warms the cache once, fails fast on dep-fetch issues.

## Independent re-verification

**Security spine:** `wire_previews` skips `PlateKind::Ms1` before `render_plate`; the sidecar receives only the public `string` on stdin (no argv/env/file); the Go sidecar has NO `net/http`/`net`/`dial` import (grep-confirmed across `preview/`). ms1 never reaches the sidecar; no secret in any output. Intact.

**SVG correctness:** `render_svg.go` matches `internal/golden/golden.go` `Vectorize` (177-194) exactly — single `<path>`, `dt==0` skip, pen-down `C C1 C2 C3` (implicit C0), pen-up `M C3`. The rejected `M C0 C …` bug is absent; the comment names the golden reference + the fixed bug.

**Params fidelity:** `params.go` bit-identical to BOTH `platform_sh2.go` and `gui_test.go`: `mm=6400`, `strokeWidth=0.3*mm=1920`, `Speed=30*mm`, `EngravingSpeed=8*mm`, `Acceleration=250*mm`, `Jerk=2600*mm`, `TicksPerSecond=30*mm`; `plateSizeMM=85`, `safetyMarginMM=3`. The `TicksPerSecond==Speed==topSpeed` comment present.

**release.yml read-review:** matrix covers the 5 targets (windows/arm64 omitted+documented); `submodules: true` on `go-build` + `assemble` (Rust build doesn't need it — correct); concrete pins `GO_VERSION '1.25.10'` / `RUST_TOOLCHAIN '1.85.0'` (not latest/stable); `-X main.version=${VERSION}` + a `Verify baked-in version` step asserting `me-preview ${VERSION}`; minisign key → `$(mktemp)`, removed in an `if: always()` step, password via `printf '%s\n' | minisign` (never echoed/logged); in-CI `minisign -V` self-check + `VERIFY.txt` both use the real pubkey `RWQPmgBXsuw5yi8…` (key `CA39ECB257009A0F`); `THIRD_PARTY_LICENSES` assembled from `go list -m` dirs (gonum, kortschak-qr) + the submodule's `LICENSE`; `attest-build-provenance@v1` on `release/mnemonic-engrave-*`; `softprops/action-gh-release@v2` uploads archives + `SHA256SUMS` + `.minisig`; `assemble` gated `if: startsWith(github.ref,'refs/tags/v')` so PRs are build-only. No step would fail in CI given the (already-set) secrets.

**No regression:** the folds touch only the I-1 exit mapping, the m-1 mode-sink, and the m-2 warm-up — no Phase A path affected; the `no_preview_flag_is_byte_for_byte_phase_a` test guards the boundary.

## Issues
Critical: None. Important: None. Minor: None.

## Verdict: GREEN (0 Critical / 0 Important)
All three R0 findings resolved with direct code evidence; all load-bearing invariants hold (security spine, SVG-vs-golden, param fidelity, release.yml soundness). No new issues.

---

## Loop summary (added by main session)
| Round | Verdict | Folded |
|---|---|---|
| R0 (two-stage) | NOT-GREEN | 1 Important (I-1 render-failure exit 2→4) + 2 Minor (m-1 `--out -` stdout mixing → `fc49844`; m-2 release.yml `go mod download`) |
| **R1** | **GREEN** | none — converged |

Phase B execution gate GREEN. All 12 plan tasks implemented; full suite green (41 lib / 18 cli / 2 cross-lang / golden; Go suite ok); clippy/fmt/vet clean. Ready to merge `me` v0.3.0 to master.
