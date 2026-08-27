# Push report: mnemonic-transaction P1 → origin/main

Date: 2026-08-27
Repo pushed: `bg002h/mnemonic-transaction`
Agent: sonnet subagent, dispatched to push P1 and watch CI

## Branch protection

`gh api repos/bg002h/mnemonic-transaction/branches/main/protection` returned
**404 "Branch not protected"**. The `ci/staging` ritual (mandatory on the
sibling `mnemonic-engrave`, where a required status check binds to a commit
SHA) does not apply here — there is no required check to satisfy. Per the
dispatch brief's no-protection branch: pushed `main` directly, then watched
the resulting CI run to completion anyway.

## Freeze evidence

- `git rev-parse main` before push: `a0a70a843df084e423e8306184bf42d074b6356b`
- `git rev-parse main` immediately before push (re-checked): same SHA
- `git rev-parse main` after: same SHA
- `origin/main` after (`git ls-remote origin main`): same SHA
- The freeze held for the whole window; the tip did not move.

## Push

```
git push origin main
To github.com:bg002h/mnemonic-transaction.git
   cf17591..a0a70a8  main -> main
```

12 commits landed (`566d8e3`..`a0a70a8`, P1 rows 1-13). **No bypass message
appeared** — there was nothing to bypass, since no protection rule exists.

## CI run

Run `33126435178` (workflow `ci`, job `test (rust)`), triggered by the push.
**Final conclusion: `success`.** All 14 steps green:

| # | Step | Conclusion |
|---|------|------------|
| 1 | Set up job | success |
| 2 | actions/checkout@v4 | success |
| 3 | install toolchain | success |
| 4 | fmt | success (previously RED on `origin/main`, fail-fast hid everything below) |
| 5 | clippy | success |
| 6 | build | success |
| 7 | Install the shells the history-purge gates execute | success |
| 8 | install nextest | success |
| 9 | test | success |
| 10 | refusal coverage (bijection with refusals.toml) | success |
| 11 | refusal mutation (every refusal test can fail) | success |
| 12 | journeys (encode, recover, miscut) | success |
| 24 | Post actions/checkout@v4 | success |
| 25 | Complete job | success |

This is the first time steps 5-25 have ever executed on this repo's CI,
since `fmt` was previously fail-fast-blocking them.

### Trap 1: shell-install step

Ran, and asserted all four exact hard-coded paths before printing versions:

```
ok /usr/bin/zsh
ok /usr/bin/fish
ok /usr/bin/script
ok /usr/bin/timeout
zsh 5.9 (x86_64-ubuntu-linux-gnu)
fish, version 3.7.0
```

### Trap 2: fish version skew

Confirmed present as predicted: runner fish is **3.7.0**, versus 4.8.1 local.
The history-purge tests (`tests/history_purge.rs`, run under step 11/`test`)
passed against this runner version — consistent with the brief's claim that
they assert the invariant rather than the mechanism.

## Judgment method

Queried with the full 40-character SHA and `--repo bg002h/mnemonic-transaction`
throughout. Judged the run by per-job **and** per-step conclusions (`gh run
view --json ... jobs`), not run-level status alone — all 14 steps individually
carry `"conclusion":"success"`, not just the job/run rollup.

## ci/staging

Not created (no-protection path taken), so nothing to delete.

## Result

Push landed cleanly, freeze held, CI is green end-to-end for the first time
in this repo's history, and both prepared traps (shell-install exact-path
assertion, fish 3.7.0 skew) held under real CI. No bypass, no red run, no
follow-up required.
