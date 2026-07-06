# Adversarial verification — D6-1 (r1)

**Finding:** No CI runs the test suites — `release.yml` is build-only; a red suite can
merge and be tagged/released.
**Location cited:** `.github/workflows/release.yml:36`
**Claimed severity:** critical
**Verdict:** CONFIRMED (refuted = false). Factual claim is airtight. Severity adjusted
critical → **important** (latent CI-gating gap, not a now-reachable fund-loss path).
**Confidence:** high.

---

## What I checked

### 1. Is `release.yml` the only workflow / is there any other test runner?
- `ls .github/workflows/` and `git ls-files .github` → **`release.yml` is the only file**
  under `.github` (tracked and in the working tree). No composite actions, no dependabot.
- `find` for `*.yml/*.yaml/Makefile/makefile/*.mk/justfile` (excluding `third_party/` and
  `.git/`) → the **only** hit is `.github/workflows/release.yml`. No Makefile, no justfile.
- `git ls-files | grep -iE 'pre-commit|husky|hooks'` → **nothing**. No pre-commit / git-hook
  test gate either.

### 2. Does `release.yml` ever run the tests?
Read the entire 377-line file. Three jobs, all build-only w.r.t. testing:
- `go-build` (line 36): `go mod download` + `go build` cross-compile of 5 targets +
  a `--version` string check. **No `go test`, no `go vet`.**
- `rust-build` (line 106): 5-target matrix, `cargo build --release` / `cross build --release`.
  **No `cargo test`, no `clippy`, no `fmt`.**
- `assemble` (line 188): `if: startsWith(github.ref, 'refs/tags/v')`, `needs:
  [go-build, rust-build]` — downloads artifacts, generates THIRD_PARTY_LICENSES, packs
  archives, writes + minisign-signs SHA256SUMS, attests, and publishes the GitHub release.
  **No test step.**
- `git grep -E 'cargo test|go test|cargo clippy|go vet|cargo fmt'` over the repo (excluding
  `third_party/` and `design/`) → the **only** match is a doc-comment in
  `crates/me-cli/src/preview.rs:191` ("under `cargo test`'s"). Never in a CI file.

### 3. Is the "red suite can be released" path real?
- A real suite exists: `crates/me-cli/tests/{cli,cross_lang,golden,preview_cross_lang}.rs`
  plus in-crate `#[cfg(test)]` modules, and `preview/{layout,params,render,version}_test.go`.
  None of these is invoked by any CI job.
- `assemble.needs = [go-build, rust-build]`. Both are build-only. Therefore a `v*` tag
  reaches `assemble` (build + sign + publish) with the test suite **never executed** — a
  logic regression that still compiles ships. The gate is genuinely absent.

## Reachability of the failure scenario
The scenario (a refactor of `ndef.rs`/`bundle.rs` that corrupts the emitted NDEF or admits
an incomplete set, still compiling, merging PR-green, then tag-published unsigned-by-tests)
is **structurally reachable**: PR CI only compiles; nothing runs the invariants; the release
job has no test dependency. No in-repo layer (workflow, Makefile, hook, branch-protection
status check that could even reference a nonexistent test job) closes it. The factual claim
is fully substantiated — I could not refute it.

## Severity adjudication (critical → important)
The finder is transparent that "severity = severity of the GAP" (§6). Under the funds-safety
verifier rubric — *would it really produce a wrong-but-accepted plate / lost funds?* — D6-1
by **itself** does not. Distinguishing facts:
- **Latent, not active.** The suites currently PASS (finder §1; consistent with prior CI-green
  ship note in git log). No wrong/incomplete plate is emitted today.
- **Requires a compound future event** to cause harm: an invariant-breaking regression that
  *still compiles* (compile breakage IS caught by the build matrix) **and** is merged despite
  a local red suite **and** is then tag-released.
- **Compensating process controls exist** (though not CI-enforced): repo mandates TDD, an R0
  architect gate before code, and a mandatory independent adversarial execution review over
  the whole diff; releases are tag-triggered and human-signed (minisign).
- It is a **risk-multiplier / keystone** that makes D6-2..D6-8 non-enforced, which is exactly
  why it is legitimately the **top item within the D6-tests dimension** — but as a standalone
  funds-safety severity it maps to **important** (a serious, complete hardening gap that
  weakens every guarantee) rather than **critical** (a concretely reachable, validation-passing
  fund-loss path that exists now).

The remediation (§5 P1: add a `test.yml` / test job set and add it to `assemble.needs`) is
correct and cheap (suites run in seconds per finder §1).

## Bottom line
`refuted = false`. The cited location and every factual assertion are correct and directly
verified. Downgrade severity to **important** on funds-safety calibration; the finding is a
real and worthwhile keystone hardening gap, not an active wrong-plate defect.
