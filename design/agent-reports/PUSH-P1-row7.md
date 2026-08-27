# PUSH-P1-row7: master push attempt via ci/staging ritual — CI FAILED, master NOT pushed

Date: 2026-08-27

## Task

Push `mnemonic-engrave`'s `master` (76 commits ahead of `origin/master`) to origin
using the `ci/staging` staging ritual, so the required `test (rust + go)` status
check is SATISFIED (not bypassed) on the exact SHA that lands.

## What happened

1. Recorded `git rev-parse master` before staging:
   `66904ec6ec4c097e6a4c669e852c082c75370f63` (full 40-char SHA).
2. Staged it: `git push origin master:refs/heads/ci/staging` — succeeded, created
   branch `ci/staging` at that exact SHA.
3. Found the triggered run via
   `gh run list --repo bg002h/mnemonic-engrave --branch ci/staging --json ...`:
   run `33118755496`, `headSha` = `66904ec6ec4c097e6a4c669e852c082c75370f63`
   (matches — confirmed via full-SHA comparison, not truncated).
4. Watched the run to completion (`gh run watch 33118755496 --repo
   bg002h/mnemonic-engrave --exit-status`), then independently queried job
   conclusions (not just run-level status) via:
   `gh run view 33118755496 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha,jobs`

   Result — **`test (rust + go)` job: `conclusion: "failure"`**, run-level
   `conclusion: "failure"`. All six build jobs (`build me` ×5 platforms,
   `build me-preview`) succeeded; `assemble + sign + release` correctly
   `skipped` (gated on `refs/tags/v*`, not triggered by a branch push — expected
   per `.github/workflows/release.yml`).

5. Because the required check FAILED, per the task's explicit instruction
   ("Report it; do not fix it and do not retry blindly") — **no fold, no fix, no
   retry, and no push of `master` to `origin/master`.**

## Root cause (classified, not guessed — pulled the actual failure log)

`gh run view 33118755496 --repo bg002h/mnemonic-engrave --job 98680040232
--log-failed` shows 233/236 unit tests passed, then in
`tests/history_purge.rs`:

```
test the_harness_records_history_at_all ... FAILED
test editing_the_file_alone_is_the_trap_the_message_warns_about ... FAILED
test the_emitted_zsh_recipe_actually_purges_the_entry ... FAILED

thread '...' panicked at crates/me-cli/tests/history_purge.rs:35:5:
/usr/bin/zsh is required: F-264's gate is 'the emitted recipe, RUN under a
real interactive zsh, actually removes the entry', and there is no way to run
it without zsh. This is deliberately a FAILURE and not a skip -- a skipped
gate prints ok and exit 0. If CI lacks zsh, install it there rather than
weakening this.
```

**This is a genuine, first-time-tested defect, not a flake and not the known
fmt/clippy gap.** Verified:

- `crates/me-cli/tests/history_purge.rs` was introduced in commit `1db1e81`
  ("P0 IMPLEMENTED: rows 1-10 by an opus agent, merged onto master"),
  confirmed via `git log --oneline -- crates/me-cli/tests/history_purge.rs`
  (single hit).
- `1db1e81` is one of the 76 commits ahead of `origin/master`
  (`git merge-base --is-ancestor 1db1e81 origin/master` → not an ancestor;
  it sits inside `git log origin/master..master`). So this exact test file has
  **never been run in CI before** — there is no prior green run to contradict.
- All four most recent CI runs on already-merged SHAs (`990f75a`, `9c6214a`,
  `1baccfa`, `5fba015`) show `conclusion: "success"` — consistent with none of
  them containing `history_purge.rs`.
- The design intent (per the panic message itself, citing F-264) is that this
  test hard-fails rather than silently skips when `/usr/bin/zsh` is absent —
  the GitHub Actions `ubuntu-latest` runner used by this workflow does not have
  zsh installed, so the gate fires exactly as designed. This is a CI-runner
  environment gap (no zsh), not a logic defect in the code under test.

## Freeze verification

| Point | `git rev-parse master` |
| --- | --- |
| Before staging | `66904ec6ec4c097e6a4c669e852c082c75370f63` |
| Immediately before cleanup / final check | `66904ec6ec4c097e6a4c669e852c082c75370f63` |

Identical — local `master` never moved during the window. `origin/master`
confirmed still at `990f75acb971b76bfd67028db9f02bf63190d43c` (fetched
independently after the failed run) — i.e. **no push to `origin/master` was
made**, honestly reflecting the CI failure rather than a bypass.

## Outcome

- Staged SHA: `66904ec6ec4c097e6a4c669e852c082c75370f63`
- Final SHA pushed to `origin/master`: **none — push did not happen.**
- `test (rust + go)` conclusion: **`failure`** (verbatim from `gh run view --json`)
- Final push to `origin/master`: **not attempted** (no bypass message possible,
  since no push was made — branch protection was neither satisfied nor
  bypassed, it was correctly never tested against this SHA)
- `ci/staging`: deleted (`git push origin --delete ci/staging` succeeded)
- Branch protection: untouched, no workaround attempted, no `enforce_admins`
  change

## What this leaves for the operator

`master` is still 76 commits ahead of `origin/master`, tree otherwise clean.
The blocking defect is environmental (GitHub Actions `ubuntu-latest` runner
lacks `zsh`), affecting the F-264 tests in
`crates/me-cli/tests/history_purge.rs`. Two directions consistent with F-264's
own stated design intent ("If CI lacks zsh, install it there rather than
weakening this."):

1. Add zsh installation to `.github/workflows/release.yml`'s `test (rust + go)`
   job (e.g. `apt-get install -y zsh`), or
2. Confirm/adjust if zsh availability was assumed but never actually verified
   against the real runner image.

This report does not choose between them — that is an operator/implementation
decision, not something to guess at under this dispatch's "report, don't fix"
constraint. This report file is intentionally **left uncommitted**: the
dispatch's mandatory-final-action instruction conditioned committing it on the
final push having landed ("so the commit cannot violate the freeze") — since
the push never landed, committing now would put a new commit on `master`
without that anchor, so it is left as an uncommitted file in the working tree
for the operator to review and commit (or not) themselves.
