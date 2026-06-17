# `me bundle --preview` Phase B — SPEC review: release-CI addition (R0→R1)

- **Stage:** R0+R1 review of the **release-CI addition** to the Phase B spec (user folded the previously-deferred signed cross-platform release-CI into v0.3.0 scope). The render/fidelity sections (§4–§6) had already converged R0→R1 GREEN separately (`me-bundle-phaseB-spec-review-R{0,1}.md`) and were NOT re-reviewed here.
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect` (adversarial).
- **Provenance of §10:** an architect research pass recommended minisign + git-submodule + the cross-build matrix (see the session's release-CI recommendation); §10 folds that recommendation.
- **Outcome:** R0 **GREEN (0C/0I)** + 4 Minor (folded) → R1 **GREEN (0C/0I)** + 1 new Minor m-2-a (`var version string`, folded). Converged.

---

## VERBATIM — R0 (release-CI addition)

### Item 1 — Submodule replace-path (CRUX): PASS. `preview/go.mod` is one level under repo root; `third_party/seedhammer` is at root ⇒ `replace seedhammer.com => ../third_party/seedhammer` is correct (one `..` up, then into third_party/seedhammer). `.gitmodules` + `actions/checkout submodules: true` + the SHA pin (`713aee2`/v1.4.2) is the canonical reproducible mechanism; `replace` to a local path bypasses proxy + sumdb correctly. `go build` must run in `preview/` (§10.2 says so).

### Item 2 — Go cross-build (all 5 targets, one ubuntu runner, CGO_ENABLED=0): PASS. All deps pure Go (render pkgs, gonum, kortschak-qr, `font/sh` `//go:embed`); only CGO is `driver/otp/otp_rp2350.go` (`//go:build tinygo && rp2350`, unreachable). `-trimpath -ldflags="-s -w"` + `.exe` handling correct. darwin/windows/arm64 cross from Linux all fine with CGO_ENABLED=0.

### Item 3 — Rust matrix: PASS. ubuntu (native x86_64 + `cross` for aarch64-linux; Docker available on GH ubuntu), macos-latest arm64 (native aarch64-apple + `rustup target add x86_64-apple-darwin` — Apple universal SDK, no `cross`), windows-latest msvc. windows/arm64 omission correctly reasoned (no GH runner) + documented. (Minor: `--remap-path-prefix` syntax abbreviated.)

### Item 4 — Assemble+sign: PASS. Fan-in (build jobs upload → assemble `needs:` → download), tar.gz/zip-per-OS, archive bundles me+me-preview+minisign.pub, `sha256sum` over archives, minisign from Secrets (key never committed, pubkey committed+README), signing-only-on-tags (PR dry-run §8), `softprops/action-gh-release`, optional attest-build-provenance layered. Verify commands correct. (Minor: secret-key on-disk cleanup unspecified.)

### Item 5 — Internal consistency: PASS. §10 references §7 files (release.yml, minisign.pub, third_party/seedhammer); build target `.` ↔ §7 `preview/main.go`; version lockstep (§4.1/§5/§8/§9) consistent. (Minor: Go version-baking mechanism unspecified.)

### Item 6 — No regression: PASS. §4 (PlanEngraving/bspline/direct-cubic-SVG/replicated-params/plate-fit), §5 (integration), §6 (exit codes) untouched by the addition. Matches the R1-GREEN scope exactly.

### Item 7 — Scope/SemVer: PASS. Release-CI now in-scope for v0.3.0; no dangling "deferred" references. (Minor: toolchain version pinning stated as intent.)

**R0 Issues:** Critical: None. Important: None. Minor: m-1 (secret-key cleanup), m-2 (Go `-X` version-baking unspecified), m-3 (toolchain pinning prose), m-4 (`--remap-path-prefix` full syntax).

**R0 Verdict: GREEN (0 Critical / 0 Important).** 4 Minors to fold.

---

## VERBATIM — R1 (convergence, 4 Minors folded)

**m-1 (§10.3 minisign secret hygiene) — RESOLVED.** Temp file scoped to the signing step + `rm` in an `if: always()` post-step (or a no-disk action) is the standard GH Actions pattern; both branches named; no residual file. No conflict.

**m-2 (§10.2 `-X main.version=`) — RESOLVED, but surfaces a new Minor.** `-X pkgpath.var=value` is the correct (only) Go link-time string-injection mechanism; `main.version` with `$VERSION` from `Cargo.toml` is the right lockstep approach, consistent with §4.1/§5. BUT the `main` package must declare `var version string` or the linker silently ignores `-X` → empty `--version` → breaks lockstep at runtime with no build error. Spec didn't state the declaration. → **new Minor m-2-a.**

**m-3 (§10.5 pin toolchains) — RESOLVED.** MUST-pin concrete `go-version`/Rust toolchain (never latest/stable) is directly consistent with the reproducibility claim; examples illustrative. No contradiction.

**m-4 (§10.2 RUSTFLAGS) — RESOLVED.** `RUSTFLAGS="--remap-path-prefix=$(pwd)=."` is correct rustc syntax (`<from>=<to>`), consistent with §10.5. 

**Cross-contradiction check:** the 4 folds touch §10.2/§10.3/§10.5 only; §10.1/§10.4/§10.6 untouched and clean; build target `.`, submodule path `../third_party/seedhammer`, version lockstep all consistent; no conflict with §4–§6.

**R1 Issues:** Critical: None. Important: None. Minor: **m-2-a** (§10.2/§4.1: `main.go` must declare `var version string` as the `-X main.version=` target — else `--version` prints empty, breaking the lockstep). Fix: one sentence in §4.1/§10.2.

**R1 Verdict: GREEN (0 Critical / 0 Important).** One new Minor (m-2-a) to fold before implementation; does not block GREEN.

---

## Loop summary (added by main session)
| Round | Verdict | Folded |
|---|---|---|
| R0 (release-CI) | GREEN (0C/0I) | 4 Minor (m-1 secret cleanup, m-2 `-X` version-baking, m-3 toolchain pinning, m-4 remap syntax) → folded into §10.2/10.3/10.5 |
| **R1 (convergence)** | **GREEN (0C/0I)** | 1 new Minor m-2-a (`var version string` declaration) → folded into §4.1 |

Phase B spec FULLY GREEN (render/fidelity R0→R1 GREEN earlier; release-CI R0→R1 GREEN here). All API/fidelity/CI claims verified in source. Cleared to writing-plans → plan R0 gate → implementation.
