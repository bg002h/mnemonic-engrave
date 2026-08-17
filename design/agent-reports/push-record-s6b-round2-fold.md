# Push record — S6b round-2 review + fold (ci/staging ritual)

**Date:** 2026-08-17
**Repo:** `bg002h/mnemonic-engrave`
**Content pushed:** three design-document commits (R0 round-2 review persisted verbatim, its fold, and a push record) — no code.

## SHA staged

`f4b75d154b1eb8c78de2bedd558efbacb7dcc4c9`

Verified via `git rev-parse HEAD` at the start of the run, matching the frozen tip specified by the controller. `master` was not touched by this agent at any point (no commit, amend, rebase, or other write).

## Ritual executed

```sh
git push origin master:refs/heads/ci/staging
gh run watch 32074117014 --repo bg002h/mnemonic-engrave --exit-status
git push origin master
git push origin --delete ci/staging
```

## Step 1 — stage the SHA

```
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      master -> ci/staging
```

## Step 2 — workflow run

Found via `gh run list --repo bg002h/mnemonic-engrave --branch ci/staging --json databaseId,headSha,...`, matched on the full 40-char SHA:

- **Run ID:** `32074117014`
- **Workflow:** `release`
- **URL:** https://github.com/bg002h/mnemonic-engrave/actions/runs/32074117014
- **head SHA:** `f4b75d154b1eb8c78de2bedd558efbacb7dcc4c9` (exact match to the staged/frozen SHA)

`gh run watch 32074117014 --repo bg002h/mnemonic-engrave --exit-status` completed with `EXIT_CODE=0`.

### Per-job conclusions (verbatim, via `gh api repos/bg002h/mnemonic-engrave/actions/runs/32074117014/jobs`)

| job | status | conclusion |
| --- | --- | --- |
| test (rust + go) | completed | success |
| build me-preview (all targets) | completed | success |
| build me (macos-aarch64) | completed | success |
| build me (linux-aarch64) | completed | success |
| build me (linux-x86_64) | completed | success |
| build me (macos-x86_64) | completed | success |
| build me (windows-x86_64) | completed | success |
| assemble + sign + release | completed | **skipped** |

The required context, `test (rust + go)`, is `success`. `assemble + sign + release` reports `skipped` as expected — it is gated on `refs/tags/v*`, and this run was triggered by a push to `refs/heads/ci/staging`, so it correctly did not sign or publish anything.

## Step 3 — final push to `master`

```
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   9ef6584..f4b75d1  master -> master
EXIT_CODE=0
```

**No "Bypassed rule violations" string appeared in the output.** The push completed as a plain fast-forward (`9ef6584..f4b75d1`). This is the positive signal that GitHub found the `test (rust + go)` context already satisfied for this exact SHA (from the `ci/staging` run) rather than treating the branch-protection rule as bypassed.

## Step 4 — delete `ci/staging` + positive control

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging

$ git ls-remote origin master ci/staging
f4b75d154b1eb8c78de2bedd558efbacb7dcc4c9	refs/heads/master
```

`ci/staging` is absent from the `ls-remote` output while `master` is present at the correct SHA (`f4b75d1...`), in the same query.

## Verdict

**SATISFIED**
