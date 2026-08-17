# Push record — S6b rewrite-fidelity fold, `ci/staging` ritual

**Date:** 2026-08-17
**Repo:** `bg002h/mnemonic-engrave` (fork)
**SHA staged and pushed:** `6de1d3a80fdec2badbb8c580b11387e690b9e145`

Verified via `git rev-parse HEAD` before any action — matched the frozen tip
stated in the dispatch brief exactly. Working tree was clean; 3 commits ahead
of `origin/master`:

```
6de1d3a s6b: fold the rewrite-fidelity Minors -- restore two dropped facts, fix my dangling pointer, name section 6's mechanism
417cc4b reports: commit the rewrite fidelity check, verbatim -- GREEN 0C/0I
1ee9c19 reports: the spec-rewrite push record -- SATISFIED, ritual finished by hand
```

Design documents only — no code changes in this push.

## Ritual steps

1. `git push origin master:refs/heads/ci/staging` — pushed `master` (SHA
   `6de1d3a80fdec2badbb8c580b11387e690b9e145`) to the throwaway `ci/staging`
   ref. Remote created the branch (`* [new branch] master -> ci/staging`).

2. Located the triggered run via
   `gh run list --repo bg002h/mnemonic-engrave --branch ci/staging --json databaseId,headSha,status,conclusion,workflowName,createdAt,url`.
   Matched run **id `32080885159`**, `headSha` = `6de1d3a80fdec2badbb8c580b11387e690b9e145`
   (exact match to the staged/frozen SHA), workflow `release`, triggered
   2026-08-17T23:32:54Z.
   URL: https://github.com/bg002h/mnemonic-engrave/actions/runs/32080885159

3. `gh run watch 32080885159 --repo bg002h/mnemonic-engrave --exit-status`
   — blocked until completion. Run-level `conclusion`: **success**.

## Per-job conclusions (verbatim, via `gh run view --json status,conclusion,jobs`)

```json
{
  "conclusion": "success",
  "headSha": "6de1d3a80fdec2badbb8c580b11387e690b9e145",
  "url": "https://github.com/bg002h/mnemonic-engrave/actions/runs/32080885159",
  "jobs": [
    {"name": "build me (macos-aarch64)",        "status": "completed", "conclusion": "success"},
    {"name": "build me (linux-aarch64)",         "status": "completed", "conclusion": "success"},
    {"name": "build me-preview (all targets)",   "status": "completed", "conclusion": "success"},
    {"name": "build me (macos-x86_64)",          "status": "completed", "conclusion": "success"},
    {"name": "test (rust + go)",                 "status": "completed", "conclusion": "success"},
    {"name": "build me (windows-x86_64)",        "status": "completed", "conclusion": "success"},
    {"name": "build me (linux-x86_64)",          "status": "completed", "conclusion": "success"},
    {"name": "assemble + sign + release",        "status": "completed", "conclusion": "skipped"}
  ],
  "status": "completed"
}
```

The required context, **`test (rust + go)`**, is `success`. All build jobs
`success`. `assemble + sign + release` is `skipped` — confirmed: that job is
gated on `refs/tags/v*`, and a `ci/**` branch push does not satisfy that
condition, exactly as expected. No signing or publishing occurred.

## Final push to `master`

```
$ git push origin master
To github.com:bg002h/mnemonic-engrave.git
   6c85d28..6de1d3a  master -> master
```

Full, exact output above — **no "Bypassed rule violations" string appears**.
The required `test (rust + go)` status check, bound to SHA
`6de1d3a80fdec2badbb8c580b11387e690b9e145` from the `ci/staging` run, was
present and passing at push time, so the branch-protection rule evaluated
the push as compliant rather than bypassed.

## `ci/staging` cleanup

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
```

## Positive control (`ci/staging` absent, `master` present, same query)

```
$ git ls-remote origin refs/heads/master refs/heads/ci/staging
6de1d3a80fdec2badbb8c580b11387e690b9e145	refs/heads/master
```

Only `refs/heads/master` is returned, at the expected SHA; `refs/heads/ci/staging`
produced no line in the same `ls-remote` invocation — confirmed deleted.

## Verdict

**SATISFIED**

`origin/master` now points at `6de1d3a80fdec2badbb8c580b11387e690b9e145` with
the `test (rust + go)` required status check bound to that exact SHA and
passing, no bypass occurred, and the throwaway `ci/staging` ref was removed
after use. `master` was not touched by this agent at any point outside the
`ci/staging` staging push and the final fast-forward push — no commits,
amends, or rebases were made.
