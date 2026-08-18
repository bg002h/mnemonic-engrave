# Push record — S6b P5 (ci/staging ritual)

## Staged SHA (read via `git rev-parse HEAD`, before any push)

```
8ebda45d14394ff3183516e7fd2820cb33d7902b
```

Local `master` was 5 commits ahead of `origin/master` (tip: `s6b: the arrows need
ONE PREDICATE PER DIRECTION -- the spec specified one for both, and it renders a
false affordance`).

## Ritual steps

### 1. Stage the SHA on `ci/staging`

```
$ git push origin master:refs/heads/ci/staging
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      master -> ci/staging
```

### 2. Run id and URL

- Run id: `32098354241`
- URL: https://github.com/bg002h/mnemonic-engrave/actions/runs/32098354241
- Triggering headSha: `8ebda45d14394ff3183516e7fd2820cb33d7902b` (confirmed via
  `gh run list`, matches the staged SHA)
- Workflow: `release`

### 3. `gh run watch 32098354241 --repo bg002h/mnemonic-engrave --exit-status`

Ran to completion (blocking). Final per-job status, verbatim via
`gh run view 32098354241 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha,jobs`:

```json
{
  "conclusion": "success",
  "headSha": "8ebda45d14394ff3183516e7fd2820cb33d7902b",
  "status": "completed",
  "jobs": [
    {"name": "test (rust + go)",              "status": "completed", "conclusion": "success"},
    {"name": "build me-preview (all targets)", "status": "completed", "conclusion": "success"},
    {"name": "build me (linux-aarch64)",       "status": "completed", "conclusion": "success"},
    {"name": "build me (macos-aarch64)",       "status": "completed", "conclusion": "success"},
    {"name": "build me (windows-x86_64)",      "status": "completed", "conclusion": "success"},
    {"name": "build me (macos-x86_64)",        "status": "completed", "conclusion": "success"},
    {"name": "build me (linux-x86_64)",        "status": "completed", "conclusion": "success"},
    {"name": "assemble + sign + release",      "status": "completed", "conclusion": "skipped"}
  ]
}
```

`assemble + sign + release` reports `skipped` — confirms the tag gate: a
`ci/**` push cannot sign or publish.

### 4. Check-runs on the exact SHA, filtered to `status == completed`

`gh api repos/bg002h/mnemonic-engrave/commits/8ebda45d14394ff3183516e7fd2820cb33d7902b/check-runs`:

```json
{"name": "assemble + sign + release",       "status": "completed", "conclusion": "skipped"}
{"name": "build me (linux-x86_64)",         "status": "completed", "conclusion": "success"}
{"name": "build me (macos-x86_64)",         "status": "completed", "conclusion": "success"}
{"name": "build me (windows-x86_64)",       "status": "completed", "conclusion": "success"}
{"name": "build me (macos-aarch64)",        "status": "completed", "conclusion": "success"}
{"name": "build me (linux-aarch64)",        "status": "completed", "conclusion": "success"}
{"name": "build me-preview (all targets)",  "status": "completed", "conclusion": "success"}
{"name": "test (rust + go)",                "status": "completed", "conclusion": "success"}
```

The required context `test (rust + go)` is `completed` / `success` on the exact
staged SHA before the final push.

### 5. Final push to `master`

```
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   37a083e..8ebda45  master -> master
```

No "Bypassed rule violations" string in the output — the required check was
SATISFIED, not bypassed.

### 6. Delete `ci/staging`

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

### 7. Positive control

`git ls-remote origin refs/heads/master refs/heads/ci/staging`:

```
8ebda45d14394ff3183516e7fd2820cb33d7902b	refs/heads/master
```

`master` present at the staged SHA; `ci/staging` absent from the same output.

## Verdict

**SATISFIED**
