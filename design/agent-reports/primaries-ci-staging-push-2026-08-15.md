# CI-staging push — mnemonic-key and mnemonic-secret — 2026-08-15

Executed sequentially per the `ci/staging` procedure. `mnemonic-key` first; only
proceeded to `mnemonic-secret` because `mnemonic-key` succeeded cleanly.

**Headline answer to the question this whole change was testing: yes, in both
repos, the `ci/**` push-trigger fired on its own introducing commit.** Pushing
the commit that *adds* the `ci/**` trigger to `ci/staging` produced a workflow
run for that exact SHA in both repos — no chicken-and-egg problem, as predicted.

---

## Repo 1 — `mnemonic-key`

- Branch: `main`
- Commit pushed: `8dc5dcbf31947762a354d165ca2350ddbb15ba28` — "ci: build ci/** so
  a staged SHA can earn its required context"
- Pre-push guard: `git log --oneline origin/main..HEAD` showed exactly this one
  commit, ahead by 1. `git status --porcelain=v1` was fully empty (no tracked
  modifications, no untracked files at all).

### Staging push

```
$ git push origin main:refs/heads/ci/staging
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-key/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-key.git
 * [new branch]      main -> ci/staging
```

### Run produced for this SHA

A run was created immediately (within the first poll, ~5s after push) — the
`ci/**` trigger worked on its own introducing commit.

- Workflow: `CI`
- Run: https://github.com/bg002h/mnemonic-key/actions/runs/31902126741
- Final status: `completed`, conclusion: `success` (whole run green, not just
  the required context)

Job-level breakdown (`gh api repos/bg002h/mnemonic-key/actions/runs/31902126741/jobs`):

```
success  freebsd compile-gate (whole-crate)
success  musl compile/test (aarch64-unknown-linux-musl)
success  build (stable on windows-latest)
success  build (beta on macos-latest)
success  build (beta on windows-latest)
success  build (beta on ubuntu-latest)
success  musl compile/test (x86_64-unknown-linux-musl)
success  build (1.85 on windows-latest)
success  build (1.85 on ubuntu-latest)
success  build (stable on ubuntu-latest)      <- REQUIRED CONTEXT
success  fmt (pinned 1.95.0)
success  build (stable on macos-latest)
success  build (1.85 on macos-latest)
success  vectors-roundtrip
skipped  release-on-tag
```

- **Required context `build (stable on ubuntu-latest)`: `success`.**
- No other job failed. `release-on-tag` correctly reported `skipped` (gated on
  tag refs, not this branch push) — did not do anything unexpected.

### Final push to `main`

```
$ git push origin main
To github.com:bg002h/mnemonic-key.git
   3462157..8dc5dcb  main -> main
EXIT_CODE: 0
```

**No "Bypassed rule violations" message.** Clean, satisfied push — the SHA's
context attached and the required check was honored rather than bypassed.

### Staging ref cleanup

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-key.git
 - [deleted]         ci/staging
EXIT_CODE: 0
```

Confirmed post-cleanup: `origin/main` = `8dc5dcb...` (matches pushed commit),
`git log --oneline origin/main..HEAD` empty, `git ls-remote --heads origin
ci/staging` empty (ref gone, no stray ref left).

**Repo 1 result: SUCCESS, no bypass, `ci/**` trigger confirmed working on its
own introducing commit.**

---

## Repo 2 — `mnemonic-secret`

- Branch: `master`
- Commit pushed: `d476b770cd73a27425e443a456eddc89375aa25a` — "ci: build ci/**,
  and drop the push-side paths filter that wedges automation"
- Pre-push guard: `git log --oneline origin/master..HEAD` showed exactly this
  one commit, ahead by 1. `git status --porcelain=v1 | grep -v '^??'` matched
  nothing (no tracked modifications/staged files). Untracked entries were
  exactly the three known WIP items — `.claude/`,
  `cycle-prep-recon-codex32-vendor-fork-cluster.md`,
  `design/SPEC_codex32_vendor_fork_cluster.md` — left completely alone
  throughout (never added, staged, stashed, or deleted; confirmed still present
  and untouched after the run, see final verification below).

### Staging push

```
$ git push origin master:refs/heads/ci/staging
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-secret/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-secret.git
 * [new branch]      master -> ci/staging
```

### Run produced for this SHA

A run was created immediately (within the first poll, ~5s after push), and it
was the only workflow triggered for this SHA (checked
`actions/runs?head_sha=...` — one `rust` workflow run, nothing else) — the
`ci/**` trigger worked on its own introducing commit here too.

- Workflow: `rust`
- Run: https://github.com/bg002h/mnemonic-secret/actions/runs/31902507839
- Final status: `completed`, **overall conclusion: `failure`** — see below,
  this is expected and does not indicate the staging step failed.

Job-level breakdown, queried directly via the jobs API (not `gh run watch`,
whose exit code reflects the overall failing conclusion and is not the signal
to use here):

```
success  clippy (ms-codec)                       <- REQUIRED
success  freebsd compile-gate (whole-crate)
failure  g6 invariant (cross-repo mlock.rs)       <- NOT required, pre-existing
success  test (ubuntu-latest)                     <- REQUIRED
success  musl compile/test (aarch64-unknown-linux-musl)
success  test (ms-codec)                          <- REQUIRED
success  fmt (pinned 1.95.0)
success  test (release, ubuntu-latest, mlock einval)
success  miri (mlock unsafe)
success  clippy                                   <- REQUIRED
success  musl compile/test (x86_64-unknown-linux-musl)
success  test (macos-latest)
```

**All four required contexts are `success`: `test (ubuntu-latest)`, `clippy`,
`test (ms-codec)`, `clippy (ms-codec)`.**

The run's overall `failure` conclusion is caused solely by `g6 invariant
(cross-repo mlock.rs)`, which is **not a required context**. This job also
fails on `master`'s parent commit `de593ca` (run `31872122114`), confirmed as a
pre-existing, already-diagnosed cross-repo byte-sync breach unrelated to this
push — not something this push introduced or fixed.

**`g6 invariant`'s conclusion on this SHA: `failure` — unchanged from the
pre-existing state on the parent commit.** It did not flip to success; nothing
about this push affected it either way.

No other job failed; no other unexpected job behavior observed.

### Final push to `master`

```
$ git push origin master
To github.com:bg002h/mnemonic-secret.git
   de593ca..d476b77  master -> master
EXIT_CODE: 0
```

**No "Bypassed rule violations" message.** Clean, satisfied push — despite the
run's overall `failure` conclusion, GitHub evaluated the four required contexts
on this SHA (all `success`) and did not bypass the rule.

### Staging ref cleanup

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-secret.git
 - [deleted]         ci/staging
EXIT_CODE: 0
```

Confirmed post-cleanup: `origin/master` = `d476b77...` (matches pushed commit),
`git log --oneline origin/master..HEAD` empty, `git ls-remote --heads origin
ci/staging` empty (ref gone). Untracked WIP status re-checked and unchanged:

```
?? .claude/
?? cycle-prep-recon-codex32-vendor-fork-cluster.md
?? design/SPEC_codex32_vendor_fork_cluster.md
```

Exactly the three pre-existing untracked items, nothing added or removed.

**Repo 2 result: SUCCESS on the four required contexts, no bypass, `ci/**`
trigger confirmed working on its own introducing commit. Overall workflow
conclusion is `failure` only due to the pre-existing, non-required `g6
invariant` job, which is out of scope for this push and unchanged by it.**

---

## Summary

| Repo | Commit | Staging run | Required context(s) | Bypass? | Staging ref deleted |
|---|---|---|---|---|---|
| mnemonic-key | `8dc5dcb` | [31902126741](https://github.com/bg002h/mnemonic-key/actions/runs/31902126741) — success (whole run) | `build (stable on ubuntu-latest)`: success | No | Yes |
| mnemonic-secret | `d476b77` | [31902507839](https://github.com/bg002h/mnemonic-secret/actions/runs/31902507839) — failure (whole run, due to pre-existing non-required `g6 invariant`) | all 4 required: success | No | Yes |

Both repos: `ci/**` trigger fired correctly on its own introducing commit, both
final pushes to the default branch were satisfied by the staged SHA's context
(no bypass message either time), and both `ci/staging` refs were deleted with
no stray refs left behind.
