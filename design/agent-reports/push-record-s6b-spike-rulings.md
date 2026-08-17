# Push record — S6b Q2 spike + operator rulings

**Date:** 2026-08-17
**Repo:** `bg002h/mnemonic-engrave`
**Ritual:** `ci/staging` (per project CLAUDE.md "Push `master` via the `ci/staging` ref")

## Preconditions

- `git rev-parse HEAD` at start: `d47b8e38d28296b19e1ed6dd6d512d8fc4b4182f` — matches the frozen SHA specified in the dispatch brief. Confirmed before any push.
- Working tree clean, branch `master`, 4 commits ahead of `origin/master`:
  - `d47b8e3` s6b: record R-I -- arrows float over the body, dissolving the F-192 coupling
  - `f725324` s6b: record R-H -- the policy id rides IN the footer, and measure the band width
  - `149edbb` s6b: RUN the Q2 spike -- it answers Q2 and breaks part of Q3
  - `0d91230` reports: persist the S6b decision-pass push record -- check SATISFIED
- All four commits are design documents; no code.

## Step 1 — stage the SHA

```
$ git push origin master:refs/heads/ci/staging
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      master -> ci/staging
```

## Step 2 — locate and watch the run

Queried via `gh run list --repo bg002h/mnemonic-engrave --branch ci/staging --json databaseId,headSha,status,conclusion,workflowName,event,url` (full 40-char SHA matched):

- **Run id:** `32058310049`
- **URL:** https://github.com/bg002h/mnemonic-engrave/actions/runs/32058310049
- **headSha:** `d47b8e38d28296b19e1ed6dd6d512d8fc4b4182f` (exact match, confirmed)
- **workflow:** `release`, **event:** `push`

`gh run watch 32058310049 --repo bg002h/mnemonic-engrave --exit-status` completed with **exit code 0**, run-level conclusion `✓`.

### Per-job conclusions (verbatim, via `gh api repos/bg002h/mnemonic-engrave/commits/d47b8e38d28296b19e1ed6dd6d512d8fc4b4182f/check-runs`, full 40-char SHA)

```
assemble + sign + release: status=completed conclusion=skipped
build me-preview (all targets): status=completed conclusion=success
build me (windows-x86_64): status=completed conclusion=success
build me (macos-aarch64): status=completed conclusion=success
build me (linux-x86_64): status=completed conclusion=success
test (rust + go): status=completed conclusion=success
build me (linux-aarch64): status=completed conclusion=success
build me (macos-x86_64): status=completed conclusion=success
```

The required context, **`test (rust + go)`**, reports `conclusion=success`.

`assemble + sign + release` reports `conclusion=skipped` — confirms the tag-gate (`refs/tags/v*`) held; a `ci/**` push did not sign or publish.

## Step 3 — push to master (no bypass)

```
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   27bd49b..d47b8e3  master -> master
```

Exit code 0. **No "Bypassed rule violations" string appears anywhere in this output.** SATISFIED signal present.

## Step 4 — delete ci/staging

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

Exit code 0.

## Positive control

`git ls-remote origin refs/heads/master refs/heads/ci/staging` (single invocation, both refs queried together):

```
d47b8e38d28296b19e1ed6dd6d512d8fc4b4182f	refs/heads/master
```

`master` present at the expected SHA; `ci/staging` absent from the same output — not an empty/ambiguous result for both.

## Final local state

- `git rev-parse HEAD`: `d47b8e38d28296b19e1ed6dd6d512d8fc4b4182f`
- `git status`: "Your branch is up to date with 'origin/master'." / "nothing to commit, working tree clean"
- No commits, amends, rebases, or other refs touched on `master` during the window.
- `enforce_admins` was not inspected or modified.
- No other remote pushed to; no tags created; `/scratch/code/shibboleth/seedhammer` was not touched.

## Verdict

**SATISFIED**
