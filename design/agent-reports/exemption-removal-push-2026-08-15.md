# `mnemonic-secret` exemption-removal push — 2026-08-15

## Task

Push one commit on `master` in `/scratch/code/shibboleth/mnemonic-secret` via
the `ci/staging` procedure. Push only — no tag created, pushed, or deleted.

## Commit pushed

- Full SHA: `6fdfd364cfe132f50468d7c5832e6cae8744a112`
- Subject: `ci: retire the mlock.rs fmt exemption — both sides now share one formatter`
- Pre-push verification in `/scratch/code/shibboleth/mnemonic-secret`:
  - `git status` → clean; only the three pre-existing untracked WIP paths
    present (`.claude/`, `cycle-prep-recon-codex32-vendor-fork-cluster.md`,
    `design/SPEC_codex32_vendor_fork_cluster.md`) — none touched, no tracked
    file modified or staged.
  - `git log --oneline origin/master..HEAD` → exactly one commit: `6fdfd36 ci:
    retire the mlock.rs fmt exemption — both sides now share one formatter`.
  - `git rev-parse HEAD` → `6fdfd364cfe132f50468d7c5832e6cae8744a112`.

## Staging push

```
$ git push origin master:refs/heads/ci/staging
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-secret/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-secret.git
 * [new branch]      master -> ci/staging
```

## Runs on SHA `6fdfd364cfe132f50468d7c5832e6cae8744a112`

Two runs occurred on this SHA, as anticipated (staging, then master after the
final push):

| Run ID | Branch | Run URL | Conclusion |
| --- | --- | --- | --- |
| 31908944917 | `ci/staging` | https://github.com/bg002h/mnemonic-secret/actions/runs/31908944917 | success |
| 31909081876 | `master` | https://github.com/bg002h/mnemonic-secret/actions/runs/31909081876 | success |

Both queried via
`gh api "repos/bg002h/mnemonic-secret/actions/runs?head_sha=6fdfd364cfe132f50468d7c5832e6cae8744a112"`
using the full 40-character SHA. Job conclusions below were read individually
from each run's jobs API, not inferred from the run-level conclusion.

### Staging run (31908944917) — all 12 jobs, explicit

| Job | Conclusion | Job URL |
| --- | --- | --- |
| `fmt (pinned 1.95.0)` | **success** | https://github.com/bg002h/mnemonic-secret/actions/runs/31908944917/job/95070992085 |
| `g6 invariant (cross-repo mlock.rs)` | **success** | https://github.com/bg002h/mnemonic-secret/actions/runs/31908944917/job/95070992138 |
| `test (ubuntu-latest)` (required) | **success** | https://github.com/bg002h/mnemonic-secret/actions/runs/31908944917/job/95070992179 |
| `clippy` (required) | **success** | https://github.com/bg002h/mnemonic-secret/actions/runs/31908944917/job/95070992151 |
| `test (ms-codec)` (required) | **success** | https://github.com/bg002h/mnemonic-secret/actions/runs/31908944917/job/95070992201 |
| `clippy (ms-codec)` (required) | **success** | https://github.com/bg002h/mnemonic-secret/actions/runs/31908944917/job/95070992320 |
| `musl compile/test (aarch64-unknown-linux-musl)` | success | https://github.com/bg002h/mnemonic-secret/actions/runs/31908944917/job/95070992090 |
| `test (release, ubuntu-latest, mlock einval)` | success | https://github.com/bg002h/mnemonic-secret/actions/runs/31908944917/job/95070992116 |
| `musl compile/test (x86_64-unknown-linux-musl)` | success | https://github.com/bg002h/mnemonic-secret/actions/runs/31908944917/job/95070992117 |
| `freebsd compile-gate (whole-crate)` | success | https://github.com/bg002h/mnemonic-secret/actions/runs/31908944917/job/95070992130 |
| `miri (mlock unsafe)` | success | https://github.com/bg002h/mnemonic-secret/actions/runs/31908944917/job/95070992176 |
| `test (macos-latest)` | success | https://github.com/bg002h/mnemonic-secret/actions/runs/31908944917/job/95070992350 |

**12 of 12 green.** All four required contexts green, plus both jobs this
commit specifically targets green.

### Master run (31909081876) — all 12 jobs, explicit (post-push, same SHA)

| Job | Conclusion | Job URL |
| --- | --- | --- |
| `fmt (pinned 1.95.0)` | **success** | https://github.com/bg002h/mnemonic-secret/actions/runs/31909081876/job/95071326609 |
| `g6 invariant (cross-repo mlock.rs)` | **success** | https://github.com/bg002h/mnemonic-secret/actions/runs/31909081876/job/95071326536 |
| `test (ubuntu-latest)` (required) | **success** | https://github.com/bg002h/mnemonic-secret/actions/runs/31909081876/job/95071326556 |
| `clippy` (required) | **success** | https://github.com/bg002h/mnemonic-secret/actions/runs/31909081876/job/95071326525 |
| `test (ms-codec)` (required) | **success** | https://github.com/bg002h/mnemonic-secret/actions/runs/31909081876/job/95071326597 |
| `clippy (ms-codec)` (required) | **success** | https://github.com/bg002h/mnemonic-secret/actions/runs/31909081876/job/95071326527 |
| `musl compile/test (aarch64-unknown-linux-musl)` | success | https://github.com/bg002h/mnemonic-secret/actions/runs/31909081876/job/95071326584 |
| `test (release, ubuntu-latest, mlock einval)` | success | https://github.com/bg002h/mnemonic-secret/actions/runs/31909081876/job/95071326582 |
| `musl compile/test (x86_64-unknown-linux-musl)` | success | https://github.com/bg002h/mnemonic-secret/actions/runs/31909081876/job/95071326613 |
| `freebsd compile-gate (whole-crate)` | success | https://github.com/bg002h/mnemonic-secret/actions/runs/31909081876/job/95071326560 |
| `miri (mlock unsafe)` | success | https://github.com/bg002h/mnemonic-secret/actions/runs/31909081876/job/95071326723 |
| `test (macos-latest)` | success | https://github.com/bg002h/mnemonic-secret/actions/runs/31909081876/job/95071326551 |

**12 of 12 green.** Confirms the same result held on the master branch itself,
not just on staging.

## Final push to master

```
$ git push origin master
To github.com:bg002h/mnemonic-secret.git
   d49d5c0..6fdfd36  master -> master
```

Exit code 0. **No "Bypassed rule violations" message appeared anywhere in the
output.**

## `origin/master` confirmed moved (via fetch, not push output)

```
$ git fetch origin
$ git rev-parse origin/master
6fdfd364cfe132f50468d7c5832e6cae8744a112
```

Matches the pushed commit exactly. `git log --oneline origin/master..HEAD` is
now empty (fully synced, verified after fetch).

## Staging ref cleanup

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-secret.git
 - [deleted]         ci/staging
```

Confirmed gone by an independent check (real exit code, not a pipe judgment):

```
$ git ls-remote origin refs/heads/ci/staging
(no output, exit 0 — ref does not exist)
```

## Tags

No tag was created, pushed, or deleted. `git push origin --delete ci/staging`
targeted the branch ref only; the final `git push origin master` carried no
`--tags`/`--follow-tags`. `git ls-remote --tags origin` returns 69 entries
before and after this session's pushes — no `ms-codec-v0.7.0`-successor tag
appeared; the newest tag remains `ms-codec-v0.7.0` (pre-existing).

## Tracked-file safety check

`git status --porcelain` on both the pre-push and post-cleanup checks shows
**zero tracked-file changes** — only the same three pre-existing untracked WIP
paths (`.claude/`, `cycle-prep-recon-codex32-vendor-fork-cluster.md`,
`design/SPEC_codex32_vendor_fork_cluster.md`), none of which were touched.

## Bottom line

**Both repos are now green with no fmt exemption anywhere.** This was a clean,
single-attempt push (no aborts, no retries): the staging run on
`6fdfd36` came back 12/12 green — including `fmt (pinned 1.95.0)` (the plain
`cargo +1.95.0 fmt --all -- --check` with the `mlock.rs` filter shell deleted)
and `g6 invariant (cross-repo mlock.rs)` comparing successfully against
`mnemonic-toolkit`'s `master` — confirming the sibling repo's half actually
took. `master` was then pushed with no bypass, `origin/master` moved to
`6fdfd364cfe132f50468d7c5832e6cae8744a112` (fetch-verified), the post-push
run on `master` itself independently reconfirmed 12/12 green, the `ci/staging`
ref was deleted and confirmed absent, and no tag exists anywhere in this
sequence.
