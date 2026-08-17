# Push record — S6b R0 round-3 closure (GREEN 0C/0I)

Date: 2026-08-17

## What was pushed

`master` of `bg002h/mnemonic-engrave`, three commits, all design documents (no code):
- the R0 round-3 closure check persisted verbatim (GREEN, 0 Critical / 0 Important)
- a push record
- a one-word function-name correction

## SHA staged

Full 40-character SHA: `7e3a0f360e0ff64f0d14b749b51d8890a88dc329`

Verified at start via `git rev-parse HEAD` — matched the frozen tip exactly. `master`
was not touched by this agent; no commits, amends, or rebases were performed.

## Ritual steps executed

1. `git push origin master:refs/heads/ci/staging`
   ```
   remote: Create a pull request for 'ci/staging' on GitHub by visiting:
   remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
   To github.com:bg002h/mnemonic-engrave.git
    * [new branch]      master -> ci/staging
   ```

2. Located the triggered run via `gh run list --repo bg002h/mnemonic-engrave --branch ci/staging --limit 5 --json databaseId,headSha,status,conclusion,workflowName,url`:
   - **Run ID: 32075706216**
   - **URL: https://github.com/bg002h/mnemonic-engrave/actions/runs/32075706216**
   - Workflow: `release`
   - `headSha` at discovery: `7e3a0f360e0ff64f0d14b749b51d8890a88dc329` (exact match, full 40 chars)

3. `gh run watch 32075706216 --repo bg002h/mnemonic-engrave --exit-status` — watched to completion.

4. Confirmed run-level result via API (full SHA, `--repo` explicit):
   ```
   gh api repos/bg002h/mnemonic-engrave/actions/runs/32075706216 --jq '{head_sha, status, conclusion, event}'
   {"conclusion":"success","event":"push","head_sha":"7e3a0f360e0ff64f0d14b749b51d8890a88dc329","status":"completed"}
   ```

## Per-job conclusions (verbatim, via `gh api .../jobs --jq '.jobs[] | "\(.name)\t\(.status)\t\(.conclusion)"'`)

```
build me-preview (all targets)	completed	success
test (rust + go)	completed	success
build me (linux-aarch64)	completed	success
build me (macos-aarch64)	completed	success
build me (linux-x86_64)	completed	success
build me (macos-x86_64)	completed	success
build me (windows-x86_64)	completed	success
assemble + sign + release	completed	skipped
```

The required context `test (rust + go)` shows `completed` / `success`.

`assemble + sign + release` shows `skipped`, as expected: that job is gated on
`refs/tags/v*`, and a push to `ci/**` cannot sign or publish. Confirms the
tag-only gate held.

## Final push to master — exact output

```
$ git rev-parse HEAD
7e3a0f360e0ff64f0d14b749b51d8890a88dc329
---
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   f4b75d1..7e3a0f3  master -> master
```

No "Bypassed rule violations" string appeared anywhere in the output. The push
carried the earned `test (rust + go)` check for this exact SHA.

## Cleanup

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

## Positive control — `ci/staging` absent, `master` present, same `ls-remote` call

```
$ git ls-remote origin refs/heads/master refs/heads/ci/staging
7e3a0f360e0ff64f0d14b749b51d8890a88dc329	refs/heads/master
```

Only `master` is listed, at the correct SHA; `ci/staging` produced no line —
confirmed deleted, in the same invocation that confirms `master` landed.

## Verdict

**SATISFIED**
