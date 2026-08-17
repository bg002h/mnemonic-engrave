# Push record — S6b followups (33f013c, 1a749ed, add6c1e)

**Date:** 2026-08-17
**Repo:** `bg002h/mnemonic-engrave`
**Ritual:** `ci/staging` staged-check push, per project CLAUDE.md

## Preconditions

- `master` was verified FROZEN at the required SHA before any action was taken:
  `git rev-parse HEAD` → `add6c1ef721809c75774103df18306715c0e18fb` (matched the
  brief's required tip exactly).
- Working tree was clean (`git status` → "nothing to commit, working tree clean"),
  branch was "ahead of 'origin/master' by 3 commits" (the three commits listed in
  the brief: `add6c1e`, `1a749ed`, `33f013c`).

## Commits pushed

```
add6c1e followups: mark S6a's four closures, and fix F-199's owning phase
1a749ed followups: rule on F-192's remedy, and file F-208 for the affordance
33f013c reports: persist the S6b recon push record -- check SATISFIED
```

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

Located via `gh run list --repo bg002h/mnemonic-engrave --branch ci/staging
--limit 5 --json databaseId,headSha,status,conclusion,workflowName,event,url`:

- **Run ID:** `32045364252`
- **URL:** https://github.com/bg002h/mnemonic-engrave/actions/runs/32045364252
- **headSha (confirmed via JSON, full 40 chars):**
  `add6c1ef721809c75774103df18306715c0e18fb` — matches the staged/required SHA exactly.
- **workflowName:** `release`
- **event:** `push`

`gh run watch 32045364252 --repo bg002h/mnemonic-engrave --interval 15` was run
to completion; final run-level status/conclusion: `status=completed`,
`conclusion=success`.

### Per-job conclusions (verbatim, via `gh run view 32045364252 --repo
bg002h/mnemonic-engrave --json status,conclusion,headSha,url,jobs`)

```json
{
  "conclusion": "success",
  "headSha": "add6c1ef721809c75774103df18306715c0e18fb",
  "jobs": [
    {"conclusion": "success", "name": "build me-preview (all targets)", "status": "completed"},
    {"conclusion": "success", "name": "test (rust + go)", "status": "completed"},
    {"conclusion": "success", "name": "build me (macos-aarch64)", "status": "completed"},
    {"conclusion": "success", "name": "build me (linux-aarch64)", "status": "completed"},
    {"conclusion": "success", "name": "build me (macos-x86_64)", "status": "completed"},
    {"conclusion": "success", "name": "build me (linux-x86_64)", "status": "completed"},
    {"conclusion": "success", "name": "build me (windows-x86_64)", "status": "completed"},
    {"conclusion": "skipped", "name": "assemble + sign + release", "status": "completed"}
  ],
  "status": "completed",
  "url": "https://github.com/bg002h/mnemonic-engrave/actions/runs/32045364252"
}
```

**Required context `test (rust + go)`: conclusion = `success`** (ran in 2m14s per
the live watch output: "✓ test (rust + go) in 2m14s").

**`assemble + sign + release`: conclusion = `skipped`** (0s duration in the live
watch output: "- assemble + sign + release in 0s") — confirms the release/sign
job did NOT run for a `ci/**` push, per `.github/workflows/release.yml`'s
`refs/tags/v*` gate. Nothing was signed or published.

## Step 3 — final push to master

```
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   8216edf..add6c1e  master -> master
```

**No "Bypassed rule violations" string appeared in this output.** The push
completed as a plain fast-forward update line only. This is the positive
signal that the `test (rust + go)` status check, now bound to
`add6c1ef721809c75774103df18306715c0e18fb`, was consulted and SATISFIED the
branch protection rule rather than being bypassed.

## Step 4 — delete the staging ref

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

### Positive control (`ci/staging` absent, `master` present, same query)

```
$ git ls-remote origin refs/heads/master refs/heads/ci/staging
add6c1ef721809c75774103df18306715c0e18fb	refs/heads/master
```

Single line returned: `master` is present at the expected SHA
(`add6c1ef721809c75774103df18306715c0e18fb`); `ci/staging` produced no line in
the same query, i.e. it is confirmed absent — not merely unqueried, since
`master` (queried in the identical command) did return a result.

## Post-check

```
$ git rev-parse HEAD
add6c1ef721809c75774103df18306715c0e18fb
$ git fetch origin master && git rev-parse origin/master
add6c1ef721809c75774103df18306715c0e18fb
```

Local `HEAD` and `origin/master` agree, both at
`add6c1ef721809c75774103df18306715c0e18fb`.

## Verdict

**SATISFIED**
