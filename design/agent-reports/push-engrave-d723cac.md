# Push report — engrave master to d723cac

## Pre-push checks (step 1)
- `git status --short`: empty (clean tree).
- Local tip: `d723cac223f4e81a3576e9e7c253e1ea22e4b90a` — matches expected `d723cac`.
- `git log --oneline origin/master..master | wc -l`: **24** commits ahead of `origin/master`.

## Push (step 2)
Ran `PATH=$HOME/.cargo/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp scripts/push-via-staging.sh master`, tee'd to `/scratch/code/shibboleth/.tmp/push-engrave-d723cac.log`. Exit code 0.

- Staging push: `ci/staging` created at `d723cac223f4e81a3576e9e7c253e1ea22e4b90a` (branch master, 24 ahead).
- CI run: **33944274589** — waited for required context `test (rust + go)`.
- Master push: `1b0ec7e..d723cac  HEAD -> master`.
- `ci/staging` branch deleted after the push.
- Post-push straggler report (non-required jobs, informational) — all reported success/skipped (see per-job table below).

Push output's last lines, verbatim:
```
build me (macos-x86_64): success
build me (linux-x86_64): success
build me (linux-aarch64): success
assemble + sign + release: skipped
== OK: d723cac223f4e81a3576e9e7c253e1ea22e4b90a is on master with the required check earned
```

## Verification (step 3)
- `git fetch origin && git rev-parse origin/master` → `d723cac223f4e81a3576e9e7c253e1ea22e4b90a` — equals local tip.
- `grep -i "bypass" <log>` → no match (**no "Bypassed rule violations" in the output**).
- `git ls-remote origin refs/heads/ci/staging` → empty output (ref deleted, confirmed).

### Per-job conclusions (`gh api .../commits/d723cac.../check-runs`)
Two check-suite runs exist on this SHA: run **33944274589** (triggered by the `ci/staging` push, completed ~04:20 UTC) and a second run triggered by the subsequent push straight to `master` (~04:23 UTC, still `in_progress` for the non-required build jobs at query time; the required job had already completed a second time by then).

| job | run 1 (~04:20) | run 2 (~04:23) |
|---|---|---|
| test (rust + go) | success | success |
| build me (linux-x86_64) | success | in_progress |
| build me (linux-aarch64) | success | in_progress |
| build me (macos-x86_64) | success | in_progress |
| build me (macos-aarch64) | success | in_progress |
| build me (windows-x86_64) | success | in_progress |
| build me-preview (all targets) | success | in_progress |
| assemble + sign + release | skipped | (not yet re-triggered) |

The **required** context `test (rust + go)` is `success` on both runs. No job shows `failure` or `cancelled` in either run. `assemble + sign + release` is `skipped` (correct: gated on `refs/tags/v*`, not a branch push).

## Verdict

**SUCCESS**
