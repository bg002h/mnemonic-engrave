# Verdict — D6-1 (adversarial verifier #0)

**Finding:** No CI runs the test suites — `release.yml` is build-only; a red suite can
merge and be tagged/released.
**Cited location:** `.github/workflows/release.yml:36` (job `go-build`).
**Claimed severity:** critical.

## Verdict: CONFIRMED — refuted = false. Adjusted severity: **important** (down from critical).

---

## 1. Factual verification (all claims hold)

I read `.github/workflows/release.yml` in full and enumerated every CI / tooling surface
in the repo. Every factual assertion in the finding checks out:

- **`release.yml` is the ONLY workflow.** `find .github -type f` → single file
  `.github/workflows/release.yml`.
- **No other CI system.** No `.gitlab-ci.yml`, `azure-pipelines.yml`, `.circleci/`,
  `.travis.yml`, `.drone.yml`.
- **No Makefile / justfile / cargo alias.** `ls Makefile justfile …` → none; no
  `.cargo/config.toml` aliases.
- **No active git hook / pre-commit.** `.git/hooks/` contains only `*.sample`; no
  `.pre-commit-config.yaml`, no `.husky`.
- **The workflow builds but never tests.** The three jobs are:
  - `go-build` (line 36): `go mod download` + `go build` cross-compile for 5 targets +
    a `--version` string check. No `go test`, no `go vet`.
  - `rust-build` (line 106): a 5-target matrix running `cargo build --release` /
    `cross build --release`. No `cargo test`, no `clippy`, no `fmt --check`.
  - `assemble` (line 188): `if: startsWith(github.ref, 'refs/tags/v')`,
    `needs: [go-build, rust-build]` — downloads artifacts, generates
    THIRD_PARTY_LICENSES, archives, writes SHA256SUMS, **minisign-signs**, attests
    provenance, and **publishes the GitHub release**. No test step.
- **The only `cargo test` / `go test` token outside `third_party/` and `design/`** is a
  doc-comment in `crates/me-cli/src/preview.rs:191` — never a CI step. Matches the
  finder's own §1 note exactly.

**Reachability of the failure scenario is real.** On a `v*` tag push, `go-build` and
`rust-build` pass iff the code *compiles*; `assemble` then signs and publishes solely on
`needs` = those two build jobs succeeding. A regression that breaks a runtime invariant
(ms1 refusal, byte-exact NDEF, chunk-set integrity, public-only preview) but still
compiles would go green on PR CI, is mergeable, and on tag would be signed + published
with the suite never having run. The finder even documents concrete compile-clean
mutations (M2/M4/M6 in §4) that no test in CI would catch. So the mechanism is exactly
as described — this is not a paper claim.

I did not run `cargo test`/`go test` myself (the finding is about CI *wiring*, not about
whether the suite currently passes; the finder already reports it green, and re-running
it would not change the CI-gap verdict). The claim stands on the workflow file, which I
verified directly.

## 2. Severity assessment — honest downgrade to *important*

The finding is genuinely a keystone gap: with no automated test gate, every funds-safety
invariant is enforced only by a human remembering to run the suite locally, and the
tag→sign→publish path has zero test dependency. For a tool that engraves Bitcoin seed
backups, that is a serious defense-in-depth hole worth fixing (the §5 P1 remedy — add a
`test` job and make `assemble.needs` include it — is correct and cheap).

But in a *funds-safety* severity rubric, **critical** should be reserved for a finding
where valid input produces a wrong-but-accepted plate *today*, or a secret leaks. D6-1 is
one level removed from that:

- **No active defect.** The finder states the suite currently passes; this finding
  produces no wrong plate on its own. It is a *latent-risk enabler*, not a live bug.
- **The failure requires a compounding, hypothetical human regression** that (a) breaks
  an invariant, (b) still compiles, AND (c) survives the project's **mandatory,
  non-deferrable independent adversarial execution review over the whole diff**
  (CLAUDE.md convention, phase 4) plus the R0 gate and TDD discipline. Those are manual,
  not CI-enforced — which is precisely why the gap is real and worth an *important* — but
  they are a genuine overlapping guard that lowers the practical probability of a red
  suite reaching a tag well below a "will lose funds now" critical.
- There is **no input that passes upstream validation and yields lost funds as a direct
  consequence of this finding**; that is the verifier's litmus for critical, and it is
  not met.

So: real, confirmed, high-value to fix — but *important* (High), not *critical*. The
adjustment is a severity calibration only; the finding itself is fully substantiated.

## 3. Confidence

**High.** The factual core (build-only CI, no test job, signed tag-publish gated only on
build jobs, no alternative CI/hook/Makefile) is airtight and directly evidenced by the
workflow file and repo enumeration. The only element of judgment is the severity tier,
where I am confident the honest calibration is *important* rather than *critical*.
