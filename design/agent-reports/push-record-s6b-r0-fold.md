# Push record — S6b R0 round-1 fold (ci/staging ritual)

**Date:** 2026-08-17
**Repo:** `/scratch/code/shibboleth/mnemonic-engrave` → `origin` = `bg002h/mnemonic-engrave`
**Verdict: SATISFIED**

## SHA staged

`9ef65843c9a972ffd47c55c7416db8b974750bb6`

Verified at the start of the run (`git rev-parse HEAD`) to match the frozen
tip specified in the dispatch brief, and re-verified after the whole ritual
completed — still `9ef65843c9a972ffd47c55c7416db8b974750bb6`. No commits
landed on `master` during the window; nothing was committed, amended, or
rebased by this agent.

Local branch state at start: `master` was 8 commits ahead of `origin/master`
(the S6b R0 round-1 review persist, its fold, and rulings R-J through R-M —
all design documents, no code), working tree clean.

## Ritual steps executed, in order

1. `git push origin master:refs/heads/ci/staging`
   ```
   remote:
   remote: Create a pull request for 'ci/staging' on GitHub by visiting:
   remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
   remote:
   To github.com:bg002h/mnemonic-engrave.git
    * [new branch]      master -> ci/staging
   ```

2. Located the triggered run via
   `gh run list --repo bg002h/mnemonic-engrave --branch ci/staging --commit 9ef65843c9a972ffd47c55c7416db8b974750bb6`
   → workflow **release**, run id **32071652268**,
   `https://github.com/bg002h/mnemonic-engrave/actions/runs/32071652268`

3. `gh run watch 32071652268 --repo bg002h/mnemonic-engrave --exit-status` — watched to completion, exit code `0`.

## Per-job conclusions (verbatim, via `gh api repos/bg002h/mnemonic-engrave/actions/runs/32071652268/jobs`)

```
test (rust + go)              completed   success
build me (windows-x86_64)     completed   success
build me (macos-aarch64)      completed   success
build me (macos-x86_64)       completed   success
build me (linux-aarch64)      completed   success
build me-preview (all targets) completed  success
build me (linux-x86_64)       completed   success
assemble + sign + release     completed   skipped
```

Required context **`test (rust + go)`**: **success**.

`assemble + sign + release` reports **skipped**, as expected — that job is
gated on `refs/tags/v*` and a `ci/**` branch push cannot sign or publish.

Run-level (`gh api repos/bg002h/mnemonic-engrave/actions/runs/32071652268`):
```json
{"conclusion":"success","event":"push","head_sha":"9ef65843c9a972ffd47c55c7416db8b974750bb6","html_url":"https://github.com/bg002h/mnemonic-engrave/actions/runs/32071652268","status":"completed"}
```
`head_sha` confirmed to be the exact frozen SHA — the check is bound to the
correct commit.

## Final push to `master`

```
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   d47b8e3..9ef6584  master -> master
```

**No "Bypassed rule violations" string appeared.** The required `test (rust + go)`
context had already been recorded against `9ef6584...` by the staged run, so
the rule evaluated the push as carrying a passing check — **SATISFIED**, not
bypassed.

## Cleanup

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

## Positive control — `ci/staging` absent, `master` present, same query

```
$ git ls-remote origin refs/heads/master refs/heads/ci/staging
9ef65843c9a972ffd47c55c7416db8b974750bb6	refs/heads/master
```

Only `refs/heads/master` is returned, at the exact staged/gated SHA;
`refs/heads/ci/staging` produced no row in the same `ls-remote` call —
confirmed deleted, not merely unlisted due to a filter mistake.

## Verdict

**SATISFIED.** The required `test (rust + go)` check passed against the exact
frozen SHA on the throwaway `ci/staging` ref before the real push to `master`,
the final push to `master` produced no bypass message, `ci/staging` was
deleted, and the positive control confirms both facts in one query. No other
remotes were touched; no tags were created; `/scratch/code/shibboleth/seedhammer`
was not touched; `enforce_admins` was not modified or discussed as an action.
