# Push report — master → origin/master, 2026-08-22

## Task

Push local `master` (`ed1c52287d61437291fa769d543ced23ee333aa7`, 9 commits ahead
of `origin/master` at `9b0a98a0fcaf8786c36ecc0808fcd2db3851ae76`) to `origin`
using the repo's required staging ritual, so the `test (rust + go)` branch
protection check is satisfied rather than bypassed.

## Preconditions verified before starting

```
$ git status --short --branch
## master...origin/master [ahead 9]

$ git rev-parse HEAD
ed1c52287d61437291fa769d543ced23ee333aa7

$ git rev-parse origin/master
9b0a98a0fcaf8786c36ecc0808fcd2db3851ae76

$ git remote -v
origin  git@github.com:bg002h/mnemonic-engrave.git (fetch)
origin  git@github.com:bg002h/mnemonic-engrave.git (push)
```

Working tree clean, tip matched the expected `ed1c522`.

## Exact commands run, in order

```sh
git push origin master:refs/heads/ci/staging
gh run list --repo bg002h/mnemonic-engrave --limit 5 --json databaseId,headSha,headBranch,status,conclusion,event,workflowName,createdAt
gh run watch 32610607207 --repo bg002h/mnemonic-engrave --exit-status
gh run view 32610607207 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha,headBranch,jobs --jq '{status,conclusion,headSha,headBranch,jobs: [.jobs[] | {name, status, conclusion}]}'
git rev-parse HEAD   # re-check tip before real push
git push origin master
git push origin --delete ci/staging
git ls-remote origin refs/heads/master
git rev-parse HEAD   # final confirmation
```

### Step 1 — stage the SHA

```
$ git push origin master:refs/heads/ci/staging
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      master -> ci/staging
```

### Step 2 — find the run

`gh run list` (with `--repo bg002h/mnemonic-engrave` explicit, full 40-char SHA
matched) returned the run for `ed1c52287d61437291fa769d543ced23ee333aa7`:

- **Run ID: `32610607207`**
- workflow: `release`
- headBranch: `ci/staging`
- event: `push`
- initial status: `in_progress`

### Step 3 — watch to completion

`gh run watch 32610607207 --repo bg002h/mnemonic-engrave --exit-status` ran to
completion (large streaming output persisted to a local tool-output file, not
reproduced here).

### Step 4 — per-job conclusions (machine-checked via `gh run view --json`)

```json
{
  "status": "completed",
  "conclusion": "success",
  "headSha": "ed1c52287d61437291fa769d543ced23ee333aa7",
  "headBranch": "ci/staging",
  "jobs": [
    {"name": "test (rust + go)",              "status": "completed", "conclusion": "success"},
    {"name": "build me (windows-x86_64)",      "status": "completed", "conclusion": "success"},
    {"name": "build me (linux-aarch64)",       "status": "completed", "conclusion": "success"},
    {"name": "build me (linux-x86_64)",        "status": "completed", "conclusion": "success"},
    {"name": "build me-preview (all targets)", "status": "completed", "conclusion": "success"},
    {"name": "build me (macos-aarch64)",       "status": "completed", "conclusion": "success"},
    {"name": "build me (macos-x86_64)",        "status": "completed", "conclusion": "success"},
    {"name": "assemble + sign + release",      "status": "completed", "conclusion": "skipped"}
  ]
}
```

`test (rust + go)` (the required check) = **success**. `assemble + sign +
release` = **skipped**, as expected for a `ci/**` push per
`.github/workflows/release.yml` (gated on `refs/tags/v*`). No job showed any
conclusion other than `success` or the expected `skipped`.

### Step 5 — re-verify tip before the real push

```
$ git rev-parse HEAD
ed1c52287d61437291fa769d543ced23ee333aa7
```

Unchanged from the precondition check — the controller held the freeze.

### Step 6 — push master for real

```
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   9b0a98a..ed1c522  master -> master
```

**Full verbatim output above — this is the entire stdout+stderr of the
command.** No "Bypassed rule violations" text appeared anywhere in the output.

### Step 7 — clean up the staging ref

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

### Step 8 — final confirmation

```
$ git ls-remote origin refs/heads/master
ed1c52287d61437291fa769d543ced23ee333aa7    refs/heads/master

$ git rev-parse HEAD
ed1c52287d61437291fa769d543ced23ee333aa7
```

## Summary

| Check | Result |
| --- | --- |
| Run ID | `32610607207` |
| `test (rust + go)` conclusion | `success` |
| `assemble + sign + release` conclusion | `skipped` (expected, non-tag push) |
| "Bypassed rule violations" in final push output | **No** — did not appear |
| Final `origin/master` SHA | `ed1c52287d61437291fa769d543ced23ee333aa7` |
| Matches local HEAD | Yes |
| `ci/staging` deleted | Yes |
| Force-push used | No |
| Branch protection changed | No |

Push completed successfully via the required staging ritual. No hard-stop
condition was triggered.
