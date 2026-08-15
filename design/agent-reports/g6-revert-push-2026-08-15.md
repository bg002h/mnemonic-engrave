# g6 invariant revert push — 2026-08-15

## Commit pushed

`2ebea45d931e9e558ede44d105987fff2334e803` — "Revert \"style(ms-cli): rustfmt
mlock.rs under the pinned 1.95.0 toolchain\"" — in
`/scratch/code/shibboleth/mnemonic-secret`, repo `bg002h/mnemonic-secret`.

Verified before starting: `git log --oneline origin/master..HEAD` showed
exactly one commit (`2ebea45`), and `git status` showed no tracked file
modified or staged (only the pre-existing untracked WIP: `.claude/`,
`cycle-prep-recon-codex32-vendor-fork-cluster.md`,
`design/SPEC_codex32_vendor_fork_cluster.md` — left untouched throughout).

## Procedure followed

```
git push origin master:refs/heads/ci/staging   # new branch ci/staging created
# found run 31905308779 for head_sha 2ebea45... via the workflow-runs API
# polled until status=completed
git push origin master                         # d476b77..2ebea45 fast-forward
git push origin --delete ci/staging             # deleted
```

## CI run

Run: `rust` workflow, run id `31905308779`
URL: https://github.com/bg002h/mnemonic-secret/actions/runs/31905308779
Head SHA: `2ebea45d931e9e558ede44d105987fff2334e803`
Run-level status/conclusion: `completed` / `success`

## Every job's conclusion (enumerated, not judged by run-level conclusion alone)

```
success  clippy (ms-codec)
success  musl compile/test (aarch64-unknown-linux-musl)
success  test (release, ubuntu-latest, mlock einval)
success  clippy
success  miri (mlock unsafe)
success  test (ms-codec)
success  test (macos-latest)
success  g6 invariant (cross-repo mlock.rs)
success  musl compile/test (x86_64-unknown-linux-musl)
success  test (ubuntu-latest)
success  fmt (pinned 1.95.0)
success  freebsd compile-gate (whole-crate)
```

**Called out explicitly, as required:**

- **`g6 invariant (cross-repo mlock.rs)`: `success`.** This FLIPPED from the
  prior `failure` (the state master had been RED in since `de593ca` landed) to
  `success`. This commit's revert achieved its stated purpose.
- **`fmt (pinned 1.95.0)`: `success`.** Stayed green as expected — the job's
  documented exemption for `mlock.rs` covers the reintroduced rustfmt diff, and
  CI confirms what was simulated locally beforehand.

**Required contexts — all four `success`:**
- `test (ubuntu-latest)`: success
- `clippy`: success
- `test (ms-codec)`: success
- `clippy (ms-codec)`: success

**Non-required jobs** (`miri`, `freebsd compile-gate`, both `musl` targets,
`macos-latest`, `test (release, ubuntu-latest, mlock einval)`): all `success`
as well — no failures to report.

## Final `git push origin master` — verbatim output

```
To github.com:bg002h/mnemonic-secret.git
   d476b77..2ebea45  master -> master
```

No "Bypassed rule violations" message appeared. Fast-forward update, clean
push.

## Staging ref cleanup

`git push origin --delete ci/staging` succeeded:

```
To github.com:bg002h/mnemonic-secret.git
 - [deleted]         ci/staging
```

Confirmed via `git branch -a | grep staging` (no matches) and a fresh
`git fetch` — `origin/master` now resolves to `2ebea45d931e9e558ede44d105987fff2334e803`,
matching the pushed commit.

## Notes / anomalies

One transient operator error during execution: the first attempt at
`git push origin --delete ci/staging` was run without an explicit `cd` into
`mnemonic-secret` (this agent's bash cwd resets between calls) and
accidentally targeted `mnemonic-engrave`'s `origin` instead. It failed
immediately (`remote ref does not exist` — `mnemonic-engrave` has no
`ci/staging` branch) with no side effects; the retry with the correct
directory succeeded as shown above. No tracked files in either repo were
touched by this misfire.
