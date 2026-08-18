# Push record — S6b whole-diff review, `ci/staging` ritual

Date: 2026-08-18
Repo: `bg002h/mnemonic-engrave`
Operation: push `master` to `origin` via the `ci/staging` staging-and-earn-check ritual (per project CLAUDE.md "Push `master` via the `ci/staging` ref" rule).

## What was pushed

Design documents only, no fork code: the P6 and P7 implementation reports, the P7
plan addition, the whole-diff adversarial review (RED 2C/2I) committed verbatim,
and the fold report. Five commits ahead of the prior `origin/master` tip
(`aa23ca4`):

```
eb655e5 reports: commit the P8 whole-diff fold report, verbatim
7c70fd4 reports: commit the whole-diff adversarial review, verbatim -- RED 2C/2I
ed6079d reports: commit the P7 implementation report, verbatim -- F-192's real sweep
b95ccd7 plan: add P7 -- F-192 is NOT closed, because this plan narrowed the spec
c113ad4 reports: commit the P6 implementation report, verbatim
```

## SHA staged

Read directly via `git rev-parse HEAD` before any push action:

```
eb655e5e8b66ead43577d2ca56e2f647a4148e49
```

## Step 1 — stage on `ci/staging`

```
$ git push origin master:refs/heads/ci/staging
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      master -> ci/staging
```

## Step 2 — run id and per-job conclusions

Run: **32128444334**
URL: https://github.com/bg002h/mnemonic-engrave/actions/runs/32128444334
Trigger: `push` on `ci/staging`, `headSha` = `eb655e5e8b66ead43577d2ca56e2f647a4148e49` (confirmed matching the staged tip via `gh run list --json headSha`).

`gh run watch 32128444334 --repo bg002h/mnemonic-engrave --exit-status` was run and **blocked until completion**, returning exit code 0.

Per-job conclusions, from `gh run view 32128444334 --repo bg002h/mnemonic-engrave --json jobs`:

```json
{"conclusion":"success","name":"test (rust + go)","status":"completed"}
{"conclusion":"success","name":"build me-preview (all targets)","status":"completed"}
{"conclusion":"success","name":"build me (windows-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me (macos-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me (macos-aarch64)","status":"completed"}
{"conclusion":"success","name":"build me (linux-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me (linux-aarch64)","status":"completed"}
{"conclusion":"skipped","name":"assemble + sign + release","status":"completed"}
```

`assemble + sign + release` reports **skipped**, as expected — it is tag-gated
(`refs/tags/v*`) and a `ci/**` push cannot sign or publish.

Cross-check directly against the commit's check-runs (filtered to
`status == "completed"`, full 40-char SHA):

```
$ gh api repos/bg002h/mnemonic-engrave/commits/eb655e5e8b66ead43577d2ca56e2f647a4148e49/check-runs \
    --jq '.check_runs[] | select(.status=="completed") | {name, status, conclusion}'
{"conclusion":"skipped","name":"assemble + sign + release","status":"completed"}
{"conclusion":"success","name":"build me (linux-aarch64)","status":"completed"}
{"conclusion":"success","name":"build me (linux-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me (macos-aarch64)","status":"completed"}
{"conclusion":"success","name":"build me (macos-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me (windows-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me-preview (all targets)","status":"completed"}
{"conclusion":"success","name":"test (rust + go)","status":"completed"}
```

The required context, `test (rust + go)`, shows `status: completed`,
`conclusion: success` on the exact staged SHA.

## Step 3 — final push to `master`

```
$ git rev-parse HEAD
eb655e5e8b66ead43577d2ca56e2f647a4148e49
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   aa23ca4..eb655e5  master -> master
```

Exit code 0. **No "Bypassed rule violations" string appeared anywhere in the
output.** The rule accepted the push because the SHA already carried a passing
`test (rust + go)` check at evaluation time.

## Step 4 — delete `ci/staging`

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

Exit code 0.

## Positive control

Single `git ls-remote` call, `ci/staging` absent while `master` is present at
the pushed SHA:

```
$ git ls-remote origin refs/heads/master refs/heads/ci/staging
eb655e5e8b66ead43577d2ca56e2f647a4148e49	refs/heads/master
```

(No `ci/staging` line — confirms deletion. `master` present at the exact staged/pushed SHA.)

## Constraints observed

- `master` was not touched by anyone else during the window (confirmed: the tip
  read at the start, `eb655e5e8b66ead43577d2ca56e2f647a4148e49`, is the same SHA
  pushed at the end).
- Did not touch `/scratch/code/shibboleth/seedhammer` or
  `/scratch/code/shibboleth/wt-s6b`.
- `enforce_admins` was not touched or proposed.
- Full 40-character SHAs used in every `gh` query.
- `--repo bg002h/mnemonic-engrave` supplied on every `gh` call.

## Verdict

**SATISFIED**
