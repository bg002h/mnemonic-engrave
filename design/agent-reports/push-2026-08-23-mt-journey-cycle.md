# Push report — mt journey cycle, ci/staging ritual — 2026-08-23

## Task

Push `master` (65 commits ahead of `origin/master`, all design-document work on
`design/SPEC_mt_v0_1.md` plus one change to `scripts/spec-structure-check.sh`)
to `origin` in `bg002h/mnemonic-engrave`, using the repo's mandatory
`ci/staging` ritual so the required `test (rust + go)` status check binds to
the pushed commit SHA rather than being bypassed.

## Method

`scripts/push-master.sh` exists and correctly implements the ritual described
in the repo's `CLAUDE.md` (stage to `ci/staging`, watch CI, judge per-job
conclusions of every required context, freeze-check the tip, push for real
only if no bypass line appears, delete the staging ref). Read it in full
before running it — it matched the mandated procedure — and ran it directly
via `./scripts/push-master.sh --verbose` rather than hand-running the steps.
The script's own citation gate (`scripts/plan-cite-check.sh`) ran first
against the 2 changed docs in this push (`design/SPEC_mt_v0_1.md` and,
per repo convention, `agent-reports`/`FOLLOWUPS.md` excluded) and passed —
this is a machine check the script performs before ever touching `origin`,
not something I ran separately.

## Facts observed

**SHA.** `git rev-parse HEAD` before running the script: `31edd79732b3676e394fee752776248ddf90c03c`.
Every check below was re-verified against this exact 40-character SHA after
the script completed — `HEAD` had not moved.

**Staging push.** The script staged with `git push -f origin
master:refs/heads/ci/staging`, which the script's own log line confirms was
for this SHA and this commit count:

```
staging 31edd79732b3676e394fee752776248ddf90c03c (65 commit(s)) → bg002h/mnemonic-engrave@master; requires: test (rust + go)
```

**Workflow run.** Run id `32675396723`, url
`https://github.com/bg002h/mnemonic-engrave/actions/runs/32675396723`.
Independently queried (not read off the script's summary line) via:

```
gh run view 32675396723 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha,url
→ {"conclusion":"success","headSha":"31edd79732b3676e394fee752776248ddf90c03c","status":"completed","url":"https://github.com/bg002h/mnemonic-engrave/actions/runs/32675396723"}
```

`headSha` on that run matches the SHA above exactly.

**Per-job conclusions**, independently queried via
`gh run view 32675396723 --repo bg002h/mnemonic-engrave --json jobs`
(full 40-char SHA and `--repo` used throughout, per the fork's `gh` rule):

```
test (rust + go): success (status=completed)
build me (linux-x86_64): success (status=completed)
build me (macos-aarch64): success (status=completed)
build me (linux-aarch64): success (status=completed)
build me (macos-x86_64): success (status=completed)
build me-preview (all targets): success (status=completed)
build me (windows-x86_64): success (status=completed)
assemble + sign + release: skipped (status=completed)
```

`test (rust + go)` — the sole required context, confirmed independently via
`gh api repos/bg002h/mnemonic-engrave/branches/master/protection
--jq '.required_status_checks.contexts[]?'` which returned exactly one
context, `test (rust + go)` — concluded **success**. `assemble + sign +
release` is `skipped`, as expected: that job gates on `refs/tags/v*`, and this
was a branch push, not a tag push.

**Final `git push origin master`.** The script captures this push's combined
output into a variable and only prints it if it contains the string
`bypass` (case-insensitive) — on a clean success it is silently discarded by
design, so I do not have the byte-exact text of *that specific* invocation.
What I do have, independently gathered after the script exited:

- `git fetch origin master` then `git rev-parse origin/master` →
  `31edd79732b3676e394fee752776248ddf90c03c` — matches `HEAD` exactly.
- `git log --oneline -1 origin/master` →
  `31edd79 spec: verify's most important check had no specified failure output`
- Re-running `git push origin master` immediately after returned:
  ```
  Everything up-to-date
  ```
  (no bypass message, nothing to push — consistent with the push having
  already landed cleanly).
- `grep -qi bypass` over the full captured stdout/stderr of the script run
  (2634 lines, saved separately) found **zero** matches anywhere in the
  transcript.

Taken together (origin/master == HEAD, a repeat push is a no-op, and no
"bypass" string appears anywhere in the run), the final push landed clean and
without a bypass — but this is corroborating evidence assembled after the
fact, not a verbatim capture of that one push's own stdout, and the report
says so plainly rather than papering over the gap.

**`ci/staging` deletion.** `git ls-remote origin refs/heads/ci/staging`
returned empty — the ref does not exist on `origin`. The script's own summary
line confirms it ran the delete step and saw no error: it printed
`PUSHED 65 commit(s) to bg002h/mnemonic-engrave@master — 1 context(s)
success — run 32675396723 — no bypass` as its final line, which only prints
after `cleanup` (the `--delete ci/staging` call) has already run.

**`git log --oneline -1 origin/master` (final state)**:

```
31edd79 spec: verify's most important check had no specified failure output
```

This matches `HEAD`, i.e. `origin/master` now points at exactly the commit
the controller froze the tree at.

## Anything that surprised me

Nothing failed. The one thing worth flagging rather than silently glossing
over: the script's success path never echoes the literal `git push origin
master` output, so "no bypass" for the *real* push is established indirectly
(post-hoc fetch + no-op re-push + full-transcript grep) rather than by
quoting that call's own stdout verbatim. All three independent checks agree
with the script's self-reported "no bypass", and none contradicts it, but a
future run of this script could consider echoing `$OUT` unconditionally (not
only on the die path) so this gap doesn't have to be closed by triangulation
each time.

## Outcome

**Success.** 65 commits landed on `origin/master` at
`31edd79732b3676e394fee752776248ddf90c03c`, the sole required context
`test (rust + go)` concluded `success` on run `32675396723`, no bypass
occurred, `ci/staging` was deleted, and the tip did not move at any point
(`HEAD` was `31edd79` before, during, and after).
