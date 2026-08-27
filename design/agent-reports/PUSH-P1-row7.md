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

---

## Attempt 2 (2026-08-27) — CI FAILED AGAIN, a different defect, master NOT pushed

Second attempt, after commit `00bb25e` ("ci: install the shells the
history-purge gates actually execute") was believed to fix attempt 1's zsh
gap. Pre-flight checks the dispatch cited as verified: `actionlint` exit 0,
YAML parses with four jobs intact, all four hard-coded binary paths
(`/usr/bin/{zsh,fish,script,timeout}`) exist locally, and `cargo nextest run
--locked` gave **430 passed, 1 skipped** locally.

### What happened

1. Pre-staging freeze checkpoint: `git rev-parse master` =
   `00bb25e3de9b7a5250d4a9f3385a27ff41a6b8d7`; confirmed **78 commits** ahead of
   freshly-fetched `origin/master` (`990f75acb971b76bfd67028db9f02bf63190d43c`);
   tree clean.
2. Staged it: `git push origin master:refs/heads/ci/staging` — succeeded.
3. Found the triggered run by full-SHA match (never truncated, always
   `--repo bg002h/mnemonic-engrave`):
   `gh run list --repo bg002h/mnemonic-engrave --branch ci/staging --json
   databaseId,headSha,...` → run `33119300037`, `headSha` =
   `00bb25e3de9b7a5250d4a9f3385a27ff41a6b8d7` (exact match).
4. Watched to completion, then independently queried **per-job** conclusions
   (not run-level status) via
   `gh run view 33119300037 --repo bg002h/mnemonic-engrave --json
   status,conclusion,jobs`:

   ```
   RUN_STATUS=completed CONCLUSION=failure
   JOB: build me-preview (all targets) = success
   JOB: build me (macos-aarch64)       = success
   JOB: build me (windows-x86_64)      = success
   JOB: build me (linux-x86_64)        = success
   JOB: build me (macos-x86_64)        = success
   JOB: test (rust + go)               = failure
   JOB: build me (linux-aarch64)       = success
   JOB: assemble + sign + release      = skipped
   ```

   `test (rust + go)` conclusion, verbatim: **`failure`**. `assemble + sign +
   release` correctly `skipped` (tag-gated on `refs/tags/v*`, not triggered by
   a branch push — confirmed again, same as attempt 1).

5. Because the required check failed again, per the dispatch's explicit
   instruction ("report it, do not fix it and do not retry") — **no fold, no
   fix, no retry, and no push of `master` to `origin/master`.**

### The install step DID run and DID pass this time — confirmed, not assumed

Pulled `test (rust + go)`'s own step log
(`gh run view --job 98681866200 --repo bg002h/mnemonic-engrave --log`) for the
new "Install the shells the history-purge gates execute" step:

```
ok /usr/bin/zsh
ok /usr/bin/fish
ok /usr/bin/script
ok /usr/bin/timeout
zsh 5.9 (x86_64-ubuntu-linux-gnu)
fish, version 3.7.0
```

All four asserted paths present; step conclusion **success**. Attempt 1's
defect (zsh missing on `ubuntu-latest`) is closed — the zsh-dependent test
(`the_emitted_zsh_recipe_actually_purges_the_entry`) now runs and **passes**
in CI.

### Root cause of THIS failure — a different test, a genuine finding

`cargo test --locked`'s output shows the Rust test suite got to
`crates/mnemonic-io-lib/tests/fish_history_purge.rs` (5 tests) and failed one:

```
running 5 tests
test the_harness_records_history_at_all ... ok
test history_delete_exact_reports_success_and_purges_nothing ... ok
test history_delete_prefix_hangs_and_purges_nothing ... FAILED
test the_emitted_fish_recipe_actually_purges_the_entry ... ok
test the_recipe_costs_the_whole_session_and_the_text_says_so ... ok

thread 'history_delete_prefix_hangs_and_purges_nothing' panicked at
crates/mnemonic-io-lib/tests/fish_history_purge.rs:252:5:
`history delete --prefix` returned on its own. It is supposed to be waiting
at a prompt -- if it now completes unattended, re-measure whether it also
DELETES before treating this as good news. fish_history was:
- cmd: echo an-unrelated-neighbouring-command
- cmd: example-cli pack ms1SECRETSECRETPLANTED
- cmd: history save
- cmd: history delete --prefix 'example-cli pack'

test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 0 filtered
out; finished in 1.30s
##[error]Process completed with exit code 101.
```

This is **not an infra gap like attempt 1** — fish is installed and its
version is printed. It is the test's own documented behavioral assertion
failing. Read at
`crates/mnemonic-io-lib/tests/fish_history_purge.rs:248-256`, the test's own
doc comment states the premise it depends on:

> `history delete --prefix` is the command an operator reaches for, and it is
> the reason fish shipped as prose. It prompts; the prompt lists the matching
> commands, **the secret among them**; and with nobody to answer, it purges
> nothing and never returns.
>
> If this ever stops holding, fish's delete semantics changed and the recipe
> may be able to become a targeted one — re-measure before rewriting it.

On the `ubuntu-latest` runner, `fish history delete --prefix` returned
*unattended* (`s.timed_out` was false) instead of hanging at the interactive
confirmation prompt the recipe (F-273) depends on. The assertion did exactly
what its comment says it exists to do.

**Verified this is a first-time-tested path, same pattern as attempt 1's
defect:**
- `crates/mnemonic-io-lib/tests/fish_history_purge.rs` was introduced in
  commit `2efb1b9` ("P1 row 5: fish gets a purge recipe that was RUN, not
  described (F-273)"), confirmed via `git log --oneline -- <path>` (single
  hit).
- `git merge-base --is-ancestor 2efb1b9 origin/master` → **not an ancestor**;
  it sits inside `git log origin/master..master`, i.e. this exact test file
  has never been run in CI before either. The fix commit's own message
  predicted this outright: *"fish would have gone red on the very next
  round — the fish purge recipe landed hours ago in row 5 and has never seen
  CI either."* That prediction is exactly what happened, one layer deeper
  than expected: the shell got installed correctly, but the *behavior* the
  test depends on differs on this runner.

**One concrete, unconfirmed hypothesis, offered as a lead and not a
diagnosis:** local `fish --version` on this machine is **4.8.1**; the CI
runner installed Ubuntu noble's packaged **3.7.0** (both versions quoted
verbatim from their respective `--version` output above). The test's premise
about `history delete --prefix` prompting interactively may be
version-dependent fish behavior that held on 4.8.1 (where the plan/tests were
presumably authored/verified) and does not hold on 3.7.0. This was **not**
verified by installing fish 3.7.0 locally and reproducing — it is a lead for
whoever investigates next, not a confirmed root cause. Equally unconfirmed:
whether the CI runner's non-interactive/non-TTY-adjacent execution context
(`script`/`timeout`-wrapped, no real controlling terminal the way a developer
session has one) changes fish's decision to prompt at all, independent of
version.

### Freeze verification

| Point | `git rev-parse master` |
| --- | --- |
| Before staging | `00bb25e3de9b7a5250d4a9f3385a27ff41a6b8d7` |
| Immediately before final check (post-CI, pre-cleanup) | `00bb25e3de9b7a5250d4a9f3385a27ff41a6b8d7` |

Identical — local `master` never moved during the window. `origin/master`
re-fetched and confirmed still at `990f75acb971b76bfd67028db9f02bf63190d43c`
(unchanged from before this attempt) — i.e. **no push to `origin/master` was
made**, honestly reflecting the CI failure rather than a bypass.

### Outcome

- Staged SHA: `00bb25e3de9b7a5250d4a9f3385a27ff41a6b8d7`
- Final SHA pushed to `origin/master`: **none — push did not happen.**
- `test (rust + go)` conclusion: **`failure`** (verbatim from `gh run view
  --json`)
- Install step conclusion: **`success`**; zsh `5.9 (x86_64-ubuntu-linux-gnu)`,
  fish `3.7.0` (both quoted verbatim from the step's own printed output)
- Final push to `origin/master`: **not attempted** — no bypass message is
  possible because no push was made; branch protection was neither satisfied
  nor bypassed, correctly never tested against this SHA
- `ci/staging`: deleted (`git push origin --delete ci/staging` succeeded,
  confirmed by the `[deleted] ci/staging` line in the push output)
- Branch protection: untouched, no workaround attempted, no `enforce_admins`
  change

### What this leaves for the operator

`master` is still 78 commits ahead of `origin/master`, tree otherwise clean,
tip unchanged at `00bb25e`. Attempt 1's defect (zsh absent on the runner) is
now closed and verified closed — this is a genuinely new, second defect, not
a recurrence. It is a behavioral assumption in a first-time-tested gate
(F-273's fish purge test), not an environment-provisioning gap this time. The
report does not choose a fix — that is an operator/implementation decision —
but the concrete next step is almost certainly to run the F-273 recipe
against fish 3.7.0 specifically (matching the CI runner's packaged version,
not the locally-installed 4.8.1) to see whether the "hangs at a prompt"
premise itself needs re-measuring, per the test's own doc comment.

Per the dispatch brief, this file is committed **only after the final push
has landed**. It did not land this attempt either, so — same as attempt 1 —
this append is **left uncommitted** in the working tree for the operator to
review and commit (or not) themselves.

---

## Attempt 3 (2026-08-27) — CI GREEN, master PUSHED, no bypass

Third attempt. Attempt 1's zsh-absent gap (fixed in `00bb25e`) and attempt 2's
fish 3.7.0 vs 4.8.1 behavioral mismatch in the delete-prefix test (fixed in
`d91a515`, which asserts the outcome — the secret survives — rather than the
`hangs` mechanism, plus `3954ddf` for the `dash_stdin.rs` race) were both
believed fixed going in. Local pre-flight, as stated in the dispatch and not
re-derived here: `cargo nextest run --locked` → 430 passed, 1 skipped;
`cargo clippy --all-targets --locked -- -D warnings` → 0; `actionlint` → 0.

### What happened

1. Pre-staging freeze checkpoint: `git status --short` clean; `git rev-parse
   master` = `6c24e62823e6c1ac02aa3862cd6020674bf58544`; confirmed **84
   commits** ahead of freshly-fetched `origin/master`
   (`990f75acb971b76bfd67028db9f02bf63190d43c`) via `git rev-list --left-right
   --count origin/master...master` → `0	84`. (Dispatch stated 82; actual
   measured count is 84 — noted, not corrected, since the load-bearing fact is
   the SHA, which matched the dispatch exactly.)
2. Staged it: `git push origin master:refs/heads/ci/staging` — succeeded,
   created branch `ci/staging` at that exact SHA.
3. Found the triggered run by full-SHA match (never truncated, always
   `--repo bg002h/mnemonic-engrave`): `gh run list --repo
   bg002h/mnemonic-engrave --branch ci/staging --json
   databaseId,headSha,status,conclusion,event,workflowName,createdAt` → run
   `33120450937`, `headSha` = `6c24e62823e6c1ac02aa3862cd6020674bf58544`
   (exact match, `event: push`, `workflowName: release`).
4. Watched to completion two independent ways: (a) `gh run watch 33120450937
   --repo bg002h/mnemonic-engrave --exit-status`, backgrounded, exited `0`;
   (b) a separate polling `Monitor` loop querying `gh run view 33120450937
   --repo bg002h/mnemonic-engrave --json status,conclusion,jobs` directly,
   which independently reported the same terminal per-job conclusions. Final
   `gh run view --json status,conclusion,headSha,jobs`:

   ```
   status:   completed
   conclusion: success
   headSha:  6c24e62823e6c1ac02aa3862cd6020674bf58544

   build me-preview (all targets): completed / success
   build me (macos-aarch64):       completed / success
   test (rust + go):               completed / success
   build me (windows-x86_64):      completed / success
   build me (macos-x86_64):        completed / success
   build me (linux-x86_64):        completed / success
   build me (linux-aarch64):       completed / success
   assemble + sign + release:      completed / skipped
   ```

   `test (rust + go)` conclusion, verbatim: **`success`**. `assemble + sign +
   release` correctly `skipped` (tag-gated on `refs/tags/v*`, not triggered by
   a branch push — confirmed a third time).

### The shell-install step — confirmed ran and passed, versions quoted

Pulled `test (rust + go)`'s full step log (`gh run view --job 98685726693
--repo bg002h/mnemonic-engrave --log`) for the "Install the shells the
history-purge gates execute" step:

```
ok /usr/bin/zsh
ok /usr/bin/fish
ok /usr/bin/script
ok /usr/bin/timeout
zsh 5.9 (x86_64-ubuntu-linux-gnu)
fish, version 3.7.0
```

Step conclusion **success** (job overall succeeded and the log shows no error
in this step's region).

### The renamed fish test — confirmed ran and passed

From the same job log, the Rust test suite region:

```
test history_delete_prefix_purges_nothing_however_it_fails ... ok
```

This is attempt 2's fix under test: it asserts the finding (the secret
survives the purge attempt) rather than the mechanism (`hangs`), and it now
passes on the CI runner's fish 3.7.0 — the specific outcome attempt 2 could
not achieve.

### Full Rust suite — recomputed from the log, not assumed

Summed every `test result: ok. N passed; 0 failed; ...` line across all test
binaries in the job log: **430 passed, 0 failed** total (matches local
pre-flight exactly), plus the `1 ignored` from the first binary's block — no
`FAILED` or `panicked` lines anywhere in the job log. Go tests: `ok
mnemonic-engrave/preview 0.192s`. No `dash_stdin` failures (attempt-3-relevant
race from `3954ddf`) — job log shows a clean pass with no retries.

### Freeze verification

| Point | `git rev-parse master` (local) | `origin/master` |
| --- | --- | --- |
| Before staging | `6c24e62823e6c1ac02aa3862cd6020674bf58544` | `990f75acb971b76bfd67028db9f02bf63190d43c` |
| Immediately before final push | `6c24e62823e6c1ac02aa3862cd6020674bf58544` | `990f75acb971b76bfd67028db9f02bf63190d43c` (re-fetched, unchanged) |
| After final push | `6c24e62823e6c1ac02aa3862cd6020674bf58544` | `6c24e62823e6c1ac02aa3862cd6020674bf58544` (re-fetched) |

Local `master` never moved during the whole window (staging push → CI wait →
final push). `origin/master` stayed at `990f75a` through the entire CI wait
and only advanced on the final push — the freeze held.

### The final push

`git push origin master` output:

```
To github.com:bg002h/mnemonic-engrave.git
   990f75a..6c24e62  master -> master
```

A clean fast-forward. **No "Bypassed rule violations" message and no bypass
message of any kind printed** — the required `test (rust + go)` context on
this exact SHA is what satisfied branch protection, not `enforce_admins`.

### Outcome

- Staged SHA: `6c24e62823e6c1ac02aa3862cd6020674bf58544`
- Final SHA pushed to `origin/master`: `6c24e62823e6c1ac02aa3862cd6020674bf58544`
  — **identical to the staged SHA**, confirmed by the freeze table above.
- `test (rust + go)` conclusion: **`success`** (verbatim from `gh run view
  --json`)
- Shell-install step: **`success`**; zsh `5.9 (x86_64-ubuntu-linux-gnu)`, fish
  `3.7.0` (both quoted verbatim from the step's own printed output)
- Renamed fish test (`history_delete_prefix_purges_nothing_however_it_fails`):
  **`ok`**
- Final push to `origin/master`: **landed**, no bypass message
- `ci/staging`: deleted (`git push origin --delete ci/staging` printed `-
  [deleted] ci/staging`; confirmed empty via `git ls-remote --heads origin
  ci/staging`)
- Branch protection: untouched, no workaround attempted, no `enforce_admins`
  change

### What this closes

`master` and `origin/master` now agree at `6c24e62823e6c1ac02aa3862cd6020674bf58544`.
Both defects found by attempts 1 and 2 (zsh absent on the runner; the
fish-version-dependent `hangs` assertion) are confirmed fixed under real CI,
not just locally. The downstream repo's `Cargo.toml` pin should be written
against this SHA.

Per the dispatch's mandatory-final-action instruction, this file is committed
in the same window as the successful push, since the push landed.
