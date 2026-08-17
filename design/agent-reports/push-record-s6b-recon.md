# Push record — S6b cycle-prep recon (2 commits)

**Date:** 2026-08-17
**Repo:** `bg002h/mnemonic-engrave`
**Ritual:** `ci/staging` staged-check push (per project CLAUDE.md "Push master via the ci/staging ref")

## Commits pushed

- `e5859fd` reports: persist the two S6b cycle-prep recon passes, verbatim
- `8216edf` recon: fold the two S6b cycle-prep passes into a single recon

## Tip SHA staged (full 40 chars)

```
8216edfac48f596f6efcde08a204266c983cc43c
```

Confirmed via `git rev-parse HEAD` before the ritual began, and re-confirmed immediately before the final `git push origin master` (unchanged — `master` was frozen for the whole window as instructed; no commits were made during the run).

## Step 1 — stage the SHA

```sh
$ git push origin master:refs/heads/ci/staging
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      master -> ci/staging
```

## Step 2 — workflow run

Queried via `gh run list --repo bg002h/mnemonic-engrave --branch ci/staging --limit 5 --json databaseId,headSha,status,conclusion,workflowName,url,createdAt` (all `gh` calls used `--repo bg002h/mnemonic-engrave`, per hard constraint 3):

- **Run ID:** `32032202386`
- **URL:** https://github.com/bg002h/mnemonic-engrave/actions/runs/32032202386
- **headSha (full 40 chars):** `8216edfac48f596f6efcde08a204266c983cc43c` — matches the staged tip exactly
- **Workflow:** `release`
- **Trigger:** push to `ci/staging`

Watched to completion with `gh run watch 32032202386 --repo bg002h/mnemonic-engrave --exit-status` (exited 0 — success), then confirmed with:

```sh
$ gh run view 32032202386 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha
{"conclusion":"success","headSha":"8216edfac48f596f6efcde08a204266c983cc43c","status":"completed"}
```

### Per-job conclusions (verbatim, `gh run view 32032202386 --repo bg002h/mnemonic-engrave --json jobs --jq '.jobs[] | "\(.name): \(.conclusion) (status=\(.status))"'`)

```
test (rust + go): success (status=completed)
build me (macos-x86_64): success (status=completed)
build me (macos-aarch64): success (status=completed)
build me (linux-x86_64): success (status=completed)
build me (windows-x86_64): success (status=completed)
build me-preview (all targets): success (status=completed)
build me (linux-aarch64): success (status=completed)
assemble + sign + release: skipped (status=completed)
```

The required context `test (rust + go)` is **success**. `assemble + sign + release` is **skipped**, as expected — that job is gated on `refs/tags/v*`, and this was a `ci/**` push, so it cannot sign or publish. This confirms the documented behavior in `.github/workflows/release.yml`.

## Step 3 — push `master` for real

Verified `git rev-parse HEAD` immediately before pushing was still `8216edfac48f596f6efcde08a204266c983cc43c` (master was frozen throughout; no drift).

```sh
$ git rev-parse HEAD
8216edfac48f596f6efcde08a204266c983cc43c
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   5fd0b74..8216edf  master -> master
```

**The string "Bypassed rule violations" does NOT appear in this output.** This is a clean fast-forward (`5fd0b74..8216edf`), confirming the `test (rust + go)` check on this exact SHA was recognized as SATISFIED, not bypassed.

## Step 4 — delete `ci/staging` and positive-control verification

```sh
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

Positive control — `ci/staging` absent, `master` present, in the same `git ls-remote` call:

```sh
$ git ls-remote origin 'refs/heads/ci/staging' 'refs/heads/master'
8216edfac48f596f6efcde08a204266c983cc43c	refs/heads/master
```

Only one ref line returned (`master`), pointing at the exact staged/gated SHA `8216edfac48f596f6efcde08a204266c983cc43c`; `ci/staging` produced no line, confirming deletion (not merely an empty/failed query — `master` resolving correctly in the same call is the positive control).

## Verdict

**SATISFIED**
