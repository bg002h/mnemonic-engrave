# Push report — mnemonic-engrave master @ 107839c

**SHA pushed:** `107839cbd35d0700253e6320d263876515ef95f7`

**Pre-push verification:** `git rev-parse master` matched the brief's SHA exactly.
`git status --short` showed only the untracked brief file itself
(`design/agent-briefs/engrave-push-brief-107839c.md`); no tracked file was
modified. `origin/master` before push was `3d55279` (an ancestor, 4 commits
behind), matching the brief.

**Staging run:** id `33918305371`, on branch `ci/staging`, head SHA
`107839cbd35d0700253e6320d263876515ef95f7`, event `push`, status `completed`,
conclusion `success`.

**Per-job conclusions (verbatim, from `gh run view 33918305371 --repo
bg002h/mnemonic-engrave --json jobs`):**

```
test (rust + go): success
build me-preview (all targets): success
build me (windows-x86_64): success
build me (macos-aarch64): success
build me (linux-x86_64): success
build me (linux-aarch64): success
build me (macos-x86_64): success
assemble + sign + release: skipped
```

The required context `test (rust + go)` is `success`. `assemble + sign +
release` is `skipped`, as expected for a non-tag push — it does not gate.

**`scripts/push-via-staging.sh master` output (verbatim):**

```
== staging 107839cbd35d0700253e6320d263876515ef95f7 (branch master, 4 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33918305371; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   3d55279..107839c  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me-preview (all targets): success
build me (windows-x86_64): success
build me (macos-aarch64): success
build me (linux-x86_64): success
build me (linux-aarch64): success
build me (macos-x86_64): success
assemble + sign + release: skipped
== OK: 107839cbd35d0700253e6320d263876515ef95f7 is on master with the required check earned
```

The final `master` push (`3d55279..107839c HEAD -> master`) carried **no**
"Bypassed rule violations" line — the required-context gate was satisfied on
this exact SHA via the staging ritual, not bypassed.

**Post-push verification:** `git fetch origin && git rev-parse origin/master`
returned `107839cbd35d0700253e6320d263876515ef95f7` — matches the pushed SHA.
`ci/staging` was deleted by the script (confirmed in its output above).

**Not done / not applicable:** No tag, version bump, or release was created
(none requested; `assemble + sign + release` correctly reports `skipped`).
Nothing else was left undone.
