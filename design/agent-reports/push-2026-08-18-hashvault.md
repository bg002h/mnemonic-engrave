# ci/staging push ritual — 2026-08-18 (hashvault round)

Ran the branch-protection-satisfying `ci/staging` push ritual against three
repos, one at a time, in the prescribed order. `master`/`main` was frozen for
the whole run per instructions; no commits were made by this agent, only
pushes.

## 1. descriptor-mnemonic

- Default branch is **`main`**, not `master` (there is no `master` branch in
  this repo — confirmed via `git branch -a`). Used `main` throughout in place
  of `master`.
- Tip SHA pushed: `819380845feef80179398aefc8da130860498949`
- `git status --porcelain`: clean.
- `origin/ci/staging` already pointed at this exact SHA before any action by
  this agent (a prior run had staged it); re-push was a no-op
  (`git push origin main:refs/heads/ci/staging --force` failed only because
  the local branch is named `main`, not `master` — corrected and confirmed
  `git rev-parse main` == `git rev-parse origin/ci/staging`).
- Workflows/jobs that ran for this SHA:
  - Workflow **`fuzz-smoke`** → `success`
  - Workflow **`CI`** → `success`, with per-job conclusions:
    - `freebsd compile-gate (whole-crate)` → success
    - `cargo doc` → success
    - `cargo test (macos-latest)` → success
    - `cargo test (ubuntu-latest)` → success
    - `musl compile/test (x86_64-unknown-linux-musl)` → success
    - `cargo fmt` → success
    - `musl compile/test (aarch64-unknown-linux-musl)` → success
    - `cargo clippy` → success
    - `cargo test (windows-latest)` → success
  - Note: `gh api repos/bg002h/descriptor-mnemonic/rules/branches/main` and
    `.../rulesets` both returned `[]` — no branch-protection ruleset is
    currently configured via the API on this repo (or it is configured at an
    org level not visible here). Proceeded per the standing ritual anyway
    since all jobs were green and the goal (a checked SHA reaching `main`) is
    satisfied regardless.
- Final `git push origin main` output:
  ```
  To github.com:bg002h/descriptor-mnemonic.git
     266e4df3..81938084  main -> main
  ```
  No "Bypassed rule violations" text present.
- `ci/staging` ref deleted afterward; confirmed `origin/main` ==
  `819380845feef80179398aefc8da130860498949` after a fresh fetch.

**VERDICT: PUSHED.**

## 2. mnemonic-secret

- Branch: `master`. Tip SHA: `fc5a9922b3f40ee05f6a35e3cb557b6a3d8cbda8`
  (`fc5a992` — "ci: build vendor-freshness on ci/** so the staging ritual can
  gate it").
- `git status --porcelain` was **NOT clean**:
  ```
  ?? cycle-prep-recon-codex32-vendor-fork-cluster.md
  ?? design/SPEC_codex32_vendor_fork_cluster.md
  ```
  Two untracked files present in the working tree.
- Per the ritual's explicit instruction ("MUST be clean; if not, STOP and
  report"), this agent took **no push action whatsoever** on this repo — did
  not stage, commit, stash, or push anything, consistent with the freeze rule
  ("You must not commit either. You only push."). CI status for the current
  tip was not queried since the push step never started.

**VERDICT: BLOCKED — dirty working tree (two untracked files:
`cycle-prep-recon-codex32-vendor-fork-cluster.md`,
`design/SPEC_codex32_vendor_fork_cluster.md`). Needs the operator/controller
to decide whether to commit, stash, or discard those files before this repo's
push can proceed. Nothing was pushed for mnemonic-secret.**

## 3. mnemonic-engrave

- Branch: `master`. Tip SHA: `93a7629087e7640051a70c17ac95806bc605bb68`
  (`93a7629` — "journeys: the hashvault's engraved set does not name its
  slots — measured").
- `git status --porcelain`: clean.
- `git push origin master:refs/heads/ci/staging --force` → `Everything
  up-to-date` (already staged from a prior run). Verified
  `git rev-parse master` == `git rev-parse origin/ci/staging` ==
  `93a7629087e7640051a70c17ac95806bc605bb68`.
- Workflow/job that ran for this SHA — workflow **`release`** →
  `success`, with per-job conclusions:
  - `build me-preview (all targets)` → success
  - `build me (windows-x86_64)` → success
  - `build me (linux-aarch64)` → success
  - `build me (linux-x86_64)` → success
  - `test (rust + go)` → **success** (this is the required status context)
  - `build me (macos-x86_64)` → success
  - `build me (macos-aarch64)` → success
  - `assemble + sign + release` → `skipped` (expected — this job is gated on
    `refs/tags/v*`, and a `ci/**` ref push is not a tag; consistent with
    CLAUDE.md's documented behavior for this workflow)
- Final `git push origin master` output:
  ```
  To github.com:bg002h/mnemonic-engrave.git
     6d76e93..93a7629  master -> master
  ```
  No "Bypassed rule violations" text present.
- `ci/staging` ref deleted afterward; confirmed `origin/master` ==
  `93a7629087e7640051a70c17ac95806bc605bb68` after a fresh fetch.

**VERDICT: PUSHED.**

## Summary

| Repo | SHA | Verdict |
| --- | --- | --- |
| descriptor-mnemonic | `819380845feef80179398aefc8da130860498949` | PUSHED (to `main`, not `master` — repo has no `master` branch) |
| mnemonic-secret | `fc5a9922b3f40ee05f6a35e3cb557b6a3d8cbda8` | BLOCKED — dirty working tree, no push attempted |
| mnemonic-engrave | `93a7629087e7640051a70c17ac95806bc605bb68` | PUSHED |

2 of 3 repos pushed cleanly with a verified passing required check
(`test (rust + go)` for mnemonic-engrave; full green `CI` + `fuzz-smoke` for
descriptor-mnemonic) and no bypass message. mnemonic-secret was left
untouched — its working tree needs attention before the ritual can run there.

## mnemonic-secret — retry after tree cleaned

The coordinator resolved the dirty tree: the two previously-untracked files
(`cycle-prep-recon-codex32-vendor-fork-cluster.md`,
`design/SPEC_codex32_vendor_fork_cluster.md`) were committed as-authored in
`7c12f66` ("design: commit the Cycle-B recon and SPEC that shipped two months
ago"), covering a cycle (codex32 vendored inline at
`crates/ms-codec/src/codex32/`) that had already shipped. Re-ran the full
ritual for mnemonic-secret only.

- Branch: `master`. `git status --porcelain`: clean (confirmed before any
  push action).
- Tip SHA pushed: `7c12f669b096468f2ff71cc1403186ffa3f37151` (40 chars
  confirmed via `wc -c`). This carries 4 commits ahead of the prior
  `origin/master` tip `7f1dbbac075bc0e462014a3cfefbfd9b6cdb2298`: `1aa932c`
  (test(ms): prove a recombined secret still controls the same funds (P2)),
  `fbbe7bb` (ms derive: add the bg002h templates), `fc5a992` (ci: build
  vendor-freshness on ci/** so the staging ritual can gate it), `7c12f66`
  (design: commit the Cycle-B recon and SPEC).
- `git push origin master:refs/heads/ci/staging --force` →
  `fc5a992..7c12f66  master -> ci/staging`. Verified via fresh
  `git fetch origin` that `git rev-parse master` ==
  `git rev-parse origin/ci/staging` ==
  `7c12f669b096468f2ff71cc1403186ffa3f37151`.
- **`vendor-freshness` check, specifically investigated per the coordinator's
  request** (this repo has a `vendor-freshness` workflow triggered on
  `push` to `ci/**` with a `paths:` filter covering `Cargo.lock`,
  `Cargo.toml`, `crates/**/Cargo.toml`, `vendor/**`,
  `ci/repro/vendor-freshness.sh`, `.github/workflows/vendor-freshness.yml`,
  and the tip's `fc5a992` commit is the one that added the `ci/**` trigger to
  this workflow):
  - `gh run list --repo bg002h/mnemonic-secret --commit
    7c12f669b096468f2ff71cc1403186ffa3f37151` returned **only** the `rust`
    workflow — `vendor-freshness` did **not** trigger for this push event.
  - Investigated whether this is a gap or expected: `git diff --stat
    fc5a992..7c12f66` touches only two markdown files
    (`cycle-prep-recon-codex32-vendor-fork-cluster.md`,
    `design/SPEC_codex32_vendor_fork_cluster.md`) — none of
    `vendor-freshness`'s trigger paths. Checked the full range too:
    `git diff --name-only 7f1dbba..7c12f66 | grep -E
    'Cargo\.(lock|toml)|vendor/|vendor-freshness'` returned only
    `.github/workflows/vendor-freshness.yml` (added in `fc5a992`) — neither
    `Cargo.lock`, `Cargo.toml`, nor `vendor/**` changed anywhere in the
    4-commit range being pushed.
  - Confirmed `vendor-freshness` **did** already run and pass, at `fc5a992`
    on `ci/staging`, from an earlier staging push in this same session
    (`databaseId 32552039413`, `conclusion: success`, event `push`, branch
    `ci/staging`), alongside `rust` at the same SHA (`databaseId 32552039433`,
    `success`). Since no vendor-relevant path changed between `fc5a992` and
    the current tip `7c12f66`, the vendored state verified fresh at `fc5a992`
    is unchanged at the tip.
  - **Conclusion: not firing for the `7c12f66` push event is correct
    behavior of the path filter, not a skipped/missing signal** — the
    workflow already gave a fresh, applicable green answer earlier in the
    same commit range, and nothing it cares about changed since. Judged this
    per-job/per-event, not by eyeballing a run list top row, per the standing
    instruction.
- `rust` workflow for SHA `7c12f66...`: `databaseId 32553624686`, watched via
  `gh run watch` to completion, overall `conclusion: success`. Per-job
  conclusions (all `success`): `clippy`, `miri (mlock unsafe)`, `musl
  compile/test (x86_64-unknown-linux-musl)`, `g6 invariant (cross-repo
  mlock.rs)`, `test (ubuntu-latest)`, `clippy (ms-codec)`, `freebsd
  compile-gate (whole-crate)`, `test (ms-codec)`, `fmt (pinned 1.95.0)`,
  `test (release, ubuntu-latest, mlock einval)`, `musl compile/test
  (aarch64-unknown-linux-musl)`, `test (macos-latest)` — 12 of 12 jobs green.
- Re-checked `gh run list --commit` immediately before pushing to master:
  still only `rust` (success) for this SHA — no new runs appeared.
- Final `git push origin master` output:
  ```
  To github.com:bg002h/mnemonic-secret.git
     7f1dbba..7c12f66  master -> master
  ```
  No "Bypassed rule violations" text present.
- `ci/staging` ref deleted afterward; confirmed `origin/master` ==
  `7c12f669b096468f2ff71cc1403186ffa3f37151` after a fresh fetch.

**VERDICT: PUSHED.**
