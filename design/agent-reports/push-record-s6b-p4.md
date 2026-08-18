# Push record — S6b P4 (design docs: P4 implementation report + gate -timeout plan change)

## SHA staged

Full 40-char SHA (from `git rev-parse HEAD`, read directly, not trusted from the brief):

```
37a083e99b9d01b0b44eeb8c0e737837a1e8af07
```

3 commits ahead of `origin/master` at start:

```
37a083e plan: the gate must pass -timeout from P5 -- P6 is PROJECTED TO BLOW THE CEILING
a8ccc65 reports: commit the P4 implementation report, verbatim
334de67 reports: the P3 push record -- check SATISFIED
```

Working tree was clean; `master` was not touched by this agent between staging and final push.

## Ritual steps executed

```sh
git push origin master:refs/heads/ci/staging
```
Output:
```
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      master -> ci/staging
```

## Run

Located via `gh run list --repo bg002h/mnemonic-engrave --branch ci/staging --limit 5 --json databaseId,headSha,status,conclusion,event,workflowName,createdAt`, matched by exact 40-char SHA:

- **Run ID:** `32095562740`
- **URL:** https://github.com/bg002h/mnemonic-engrave/actions/runs/32095562740
- **Workflow:** `release`
- **headSha:** `37a083e99b9d01b0b44eeb8c0e737837a1e8af07` (exact match)
- **Trigger:** push to `ci/staging`

`gh run watch 32095562740 --repo bg002h/mnemonic-engrave --exit-status` was run and blocked until completion, then exited without error.

## Per-job conclusions (verbatim, from `gh run view 32095562740 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha,jobs`)

Run-level: `"status":"completed"`, `"conclusion":"success"`, `"headSha":"37a083e99b9d01b0b44eeb8c0e737837a1e8af07"`.

| Job | status | conclusion |
| --- | --- | --- |
| build me (windows-x86_64) | completed | success |
| build me (linux-aarch64) | completed | success |
| **test (rust + go)** | completed | **success** |
| build me (linux-x86_64) | completed | success |
| build me-preview (all targets) | completed | success |
| build me (macos-x86_64) | completed | success |
| build me (macos-aarch64) | completed | success |
| assemble + sign + release | completed | **skipped** |

`assemble + sign + release` is tag-gated (`refs/tags/v*`); a `ci/**` branch push cannot sign or publish, and it reported `skipped` as expected — no artifacts were signed or released by this push.

## Check-runs cross-check (filtered to `status == completed`, per SHA)

`gh api repos/bg002h/mnemonic-engrave/commits/37a083e99b9d01b0b44eeb8c0e737837a1e8af07/check-runs --jq '.check_runs[] | select(.status=="completed") | {name, status, conclusion}'`:

```json
{"conclusion":"skipped","name":"assemble + sign + release","status":"completed"}
{"conclusion":"success","name":"build me (macos-aarch64)","status":"completed"}
{"conclusion":"success","name":"build me (macos-x86_64)","status":"completed"}
{"conclusion":"success","name":"build me-preview (all targets)","status":"completed"}
{"conclusion":"success","name":"build me (linux-x86_64)","status":"completed"}
{"conclusion":"success","name":"test (rust + go)","status":"completed"}
{"conclusion":"success","name":"build me (linux-aarch64)","status":"completed"}
{"conclusion":"success","name":"build me (windows-x86_64)","status":"completed"}
```

The required context, `test (rust + go)`, shows `status: completed`, `conclusion: success` for this exact SHA. No `in_progress` entries filtered out — the SHA had no residual runs still in flight at query time.

## Final push (`git push origin master`) — exact output

```
To github.com:bg002h/mnemonic-engrave.git
   1079835..37a083e  master -> master
```

**No "Bypassed rule violations" string appeared.** The push updated `master` from `1079835` to `37a083e` — a fast-forward, matching the 3 staged commits.

## Cleanup

```sh
git push origin --delete ci/staging
```
Output:
```
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

## Positive control — `ci/staging` absent, `master` present, same `git ls-remote` call

`git ls-remote origin | grep -E 'refs/heads/(master|ci/staging)$'`:

```
37a083e99b9d01b0b44eeb8c0e737837a1e8af07	refs/heads/master
```

`ci/staging` does not appear in the grepped output (positive control passes); `master` is present at the exact staged/pushed SHA.

## Verdict

**SATISFIED**
