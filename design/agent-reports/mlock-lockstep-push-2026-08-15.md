# mlock lockstep push — 2026-08-15

## Task

Push one commit on `master` in `/scratch/code/shibboleth/mnemonic-secret` via the
`ci/staging` procedure. Push only — no tag created, pushed, or deleted.

## Commit pushed

```
d49d5c099bab89a1738f0d0c3df9306b354d62c3
fmt(mlock): 1.95.0-format ahead of the g6 pin bump — LOCKSTEP, toolkit follows
```

Pre-push verification: `git log --oneline origin/master..HEAD` showed exactly one
commit (the above). `git status` showed a clean tracked tree — no tracked file
modified or staged. Untracked files present and left untouched, as instructed:
`.claude/`, `cycle-prep-recon-codex32-vendor-fork-cluster.md`,
`design/SPEC_codex32_vendor_fork_cluster.md`.

## Procedure executed

```sh
git push origin master:refs/heads/ci/staging
# -> * [new branch]      master -> ci/staging

# waited for the run on ci/staging, then:
git push origin master
# -> 2ebea45..d49d5c0  master -> master   (no bypass message)

git push origin --delete ci/staging
# -> - [deleted]         ci/staging
```

## Run URLs

Two `rust` workflow runs fired for SHA `d49d5c09` — one from the `ci/staging`
push, one triggered directly by the `master` push itself. Both were enumerated
and waited on; job-level conclusions were pulled from the GitHub Actions Jobs
API (`.../actions/runs/<id>/jobs`), not inferred from the run's overall
conclusion.

- **ci/staging run**: https://github.com/bg002h/mnemonic-secret/actions/runs/31906339450
  (id `31906339450`, head_branch `ci/staging`, overall conclusion `failure`)
- **master run**: https://github.com/bg002h/mnemonic-secret/actions/runs/31906492540
  (id `31906492540`, head_branch `master`, overall conclusion `failure`)

Both runs' overall conclusion is `failure` — driven entirely by `g6 invariant`,
which was expected to fail per the task brief (this commit formats a file
byte-synced to a sibling repo not yet updated; g6 is not a required context).

## Every job's conclusion

### Run `31906339450` (ci/staging)

```
success  freebsd compile-gate (whole-crate)
success  clippy (ms-codec)
success  test (ubuntu-latest)
success  test (macos-latest)
success  miri (mlock unsafe)
failure  g6 invariant (cross-repo mlock.rs)
success  musl compile/test (x86_64-unknown-linux-musl)
success  fmt (pinned 1.95.0)
success  test (ms-codec)
success  musl compile/test (aarch64-unknown-linux-musl)
success  clippy
success  test (release, ubuntu-latest, mlock einval)
```

### Run `31906492540` (master)

```
success  fmt (pinned 1.95.0)
success  test (release, ubuntu-latest, mlock einval)
success  test (ms-codec)
success  test (macos-latest)
success  test (ubuntu-latest)
failure  g6 invariant (cross-repo mlock.rs)
success  clippy (ms-codec)
success  clippy
success  freebsd compile-gate (whole-crate)
success  miri (mlock unsafe)
success  musl compile/test (aarch64-unknown-linux-musl)
success  musl compile/test (x86_64-unknown-linux-musl)
```

Both runs have identical job-conclusion shape.

### Required contexts (must all be `success`) — VERIFIED on both runs

| Context | ci/staging run | master run |
| --- | --- | --- |
| `test (ubuntu-latest)` | success | success |
| `clippy` | success | success |
| `test (ms-codec)` | success | success |
| `clippy (ms-codec)` | success | success |

All four required contexts are `success` on both runs.

### Called out explicitly, non-blocking

- **`fmt (pinned 1.95.0)`**: `success` on both runs — as expected, this file is
  exempt from that gate. No surprise to flag.
- **`g6 invariant (cross-repo mlock.rs)`**: `failure` on both runs (job step
  `cargo test --test mlock_g6_invariant` failed) — expected and correct per the
  task brief; not a required context.

## Final `git push origin master` — verbatim output

```
To github.com:bg002h/mnemonic-secret.git
   2ebea45..d49d5c0  master -> master
```

Exit code `0`. **No "Bypassed rule violations" message appeared.**

## Cleanup / hard-rule confirmation

- `git push origin --delete ci/staging` succeeded (`- [deleted]  ci/staging`);
  confirmed gone via `git ls-remote origin ci/staging` (empty output).
- No tag was created, pushed, or deleted at any point. Confirmed via
  `git tag --points-at d49d5c0` (empty, local) and `git ls-remote --tags origin`
  (tail shows only pre-existing `ms-codec-v0.5.0`..`v0.7.0` tags, nothing new).
- No tracked file was modified or staged by this session; the pre-existing
  untracked WIP files were left untouched throughout.

## Outcome

`master` on `bg002h/mnemonic-secret` now points at `d49d5c0`. All four required
CI contexts passed on both the staging run and the direct master run. `g6
invariant` failed on both, exactly as anticipated (deliberate one-sided format
ahead of the sibling-repo pin bump). No bypass, no tag. `ci/staging` ref deleted.
