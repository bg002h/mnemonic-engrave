# Push record — S6b failure-states review (fold verification + failure-states review + fold)

**Date:** 2026-08-18
**Ritual:** `ci/staging` staged-check push (branch protection requires `test (rust + go)` on the commit SHA; `strict: false`)

## Staged SHA

Read directly via `git rev-parse HEAD` before staging, verified 40 characters:

```
91b0b24cac501c6fa5c74d73091101f02942e011
```

Local branch at time of staging was 4 commits ahead of `origin/master`:

```
91b0b24 reports: commit the P9 failure-states fold, verbatim
781447d reports: commit the failure-states review, verbatim -- RED 0C/3I
c78620a reports: commit the fold verification -- GREEN 0C/0I, S6b's last gate closes
95dded1 reports: the whole-diff push record -- check SATISFIED
```

## Staging push

```
$ git push origin master:refs/heads/ci/staging
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      master -> ci/staging
```

## Triggered run

Found via `gh run list --repo bg002h/mnemonic-engrave --branch ci/staging`:

- **Run ID:** `32148261375`
- **URL:** https://github.com/bg002h/mnemonic-engrave/actions/runs/32148261375
- **Workflow:** `release`
- **Head SHA:** `91b0b24cac501c6fa5c74d73091101f02942e011` (matches staged SHA exactly)
- **Triggered:** `2026-08-18T14:25:44Z`, via push to `ci/staging`

Watched to completion with `gh run watch 32148261375 --repo bg002h/mnemonic-engrave --exit-status` (blocking; exited 0).

## Overall run result (`gh run view --json status,conclusion,headSha`)

```
status:   completed
conclusion: success
headSha:  91b0b24cac501c6fa5c74d73091101f02942e011
```

## Per-job conclusions, verbatim (`gh run view --json jobs`)

```
build me-preview (all targets)   completed   success
build me (windows-x86_64)        completed   success
build me (macos-aarch64)         completed   success
build me (linux-x86_64)          completed   success
test (rust + go)                 completed   success
build me (macos-x86_64)          completed   success
build me (linux-aarch64)         completed   success
assemble + sign + release        completed   skipped
```

`assemble + sign + release` reported `skipped` as expected — it is tag-gated (`refs/tags/v*`) and this was a branch (`ci/staging`) push, so it cannot sign or publish.

## Check-runs on the staged SHA, filtered to `status == completed` (per brief constraint #6)

`gh api repos/bg002h/mnemonic-engrave/commits/91b0b24cac501c6fa5c74d73091101f02942e011/check-runs`, filtered:

```
assemble + sign + release        completed   skipped
build me (linux-aarch64)         completed   success
build me (macos-x86_64)          completed   success
test (rust + go)                 completed   success
build me (linux-x86_64)          completed   success
build me (macos-aarch64)         completed   success
build me (windows-x86_64)        completed   success
build me-preview (all targets)   completed   success
```

The required context, **`test (rust + go)`**, shows `completed` / `success` on the exact staged SHA — no `in_progress` entries present in this filtered set (no second run had fired on the SHA at query time).

## Final push to `master`

`master` was frozen for the whole window; verified immediately before the final push that `git rev-parse HEAD` still returned `91b0b24cac501c6fa5c74d73091101f02942e011` and `git status --short` was empty (no commits landed during the ritual).

```
$ git rev-parse HEAD
91b0b24cac501c6fa5c74d73091101f02942e011
$ git status --short
(empty)
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   eb655e5..91b0b24  master -> master
```

**No "Bypassed rule violations" string appeared in the output.** This is the full, exact push output — two lines, nothing else.

## Cleanup

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

## Positive control (per brief constraint #8)

`git ls-remote origin refs/heads/master refs/heads/ci/staging`, run in the same command as the deletion check:

```
91b0b24cac501c6fa5c74d73091101f02942e011	refs/heads/master
```

`ci/staging` is **absent** from the output while `master` is **present** at the pushed SHA, in the same query. Positive control passes.

## Verdict

**SATISFIED**

The required `test (rust + go)` check-run completed with conclusion `success` on the exact staged SHA before `master` was advanced to that SHA. The final push to `master` printed no bypass message. `assemble + sign + release` correctly reported `skipped` (tag-gated, not triggered by a branch push). Positive control on cleanup confirms `ci/staging` was deleted and `master` reflects the new tip.
