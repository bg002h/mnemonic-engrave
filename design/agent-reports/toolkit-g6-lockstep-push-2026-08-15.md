# mnemonic-toolkit `g6-lockstep` push — 2026-08-15 — two attempts

## Outcome (final)

**Master WAS pushed on the second attempt.** `origin/master` now points at
`27a68e9f12f677288fbd1ed724ad37d092a6c160`, verified by fetching (not just by
trusting push output). No bypass message occurred. No tag was created, pushed,
or deleted at any point across either attempt.

The **first attempt was aborted before touching master**, because the required
`examples` context failed for a real reason: the commit bumped the canonical
`ms-cli` pin in `scripts/install.sh` but left it stale in three workflow files
and the generated golden. That abort is preserved below in full — it is direct
evidence the ci/staging procedure caught a genuine incomplete-propagation
defect before it landed on master, which is exactly what the procedure is for.

---

## Attempt 1 — ABORTED, master untouched

### Commit staged

- Branch: `g6-lockstep`
- Full SHA: `621136943cf276cf54fa0160d30f6ae8c611bcd6`
- Subject: `g6: bump the ms-cli pin to v0.16.0, converge mlock.rs, and RETIRE the exemption`
- Pre-push verification (worktree `/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/d6d05a3a-7f6c-4441-96cf-71ef965d47e3/scratchpad/tk2`):
  - `git status` → clean, no tracked modifications, nothing staged.
  - `git log --oneline origin/master..HEAD` → exactly one commit (`62113694 ...`).
  - `git merge-base --is-ancestor origin/master HEAD` → exit 0 (fast-forward confirmed).
  - `origin/master` at push time: `c14b1e21dba83f1052e84e90254724002c3d951c`.

### Staging push

```
$ git push origin g6-lockstep:refs/heads/ci/staging
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-toolkit/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-toolkit.git
 * [new branch]        g6-lockstep -> ci/staging
```

Workflow runs were produced for this SHA (the `ci/**` trigger, added by this
commit, worked as expected — not the "no runs at all" failure mode the task
warned about).

### Workflow runs on SHA `621136943cf276cf54fa0160d30f6ae8c611bcd6`

| Workflow | Run ID | Run URL |
| --- | --- | --- |
| `rust.yml` ("rust") | 31906866892 | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31906866892 |
| `examples.yml` ("examples") | 31906866855 | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31906866855 |
| (non-required) "sibling-pin-check" | 31906866861 | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31906866861 |

All job conclusions below were read individually from the jobs API
(`.../actions/runs/<id>/jobs`), not inferred from run-level status.

#### The three required contexts

| Context | Workflow | Conclusion | Job URL |
| --- | --- | --- | --- |
| `examples` | examples.yml | **failure** | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31906866855/job/95065983974 |
| `test (ubuntu-latest)` | rust.yml | success | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31906866892/job/95065984294 |
| `clippy` | rust.yml | success | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31906866892/job/95065984223 |

**Result: 2 of 3 required contexts green; `examples` red → gate not satisfied → master not pushed.**

#### The two jobs called out explicitly in the brief

| Job | Conclusion | Job URL |
| --- | --- | --- |
| `fmt (pinned 1.95.0)` | **success** | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31906866892/job/95065984284 |
| `g6 invariant (cross-repo mlock.rs)` | **success** | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31906866892/job/95065984302 |

Both passed — the `mlock.rs` exemption removal and reformat held up on CI, and
the cross-repo g6 invariant was already satisfied from this repo's side.
Neither of these was the blocker.

#### Other rust.yml jobs (not required, reported for completeness)

| Job | Conclusion |
| --- | --- |
| `install.sh harnesses (man-step + MSRV guard)` | success |
| `test (release, ubuntu-latest, mlock einval)` | success |
| `lib cross-platform check (x86_64-pc-windows-msvc, windows-latest)` | success |
| `musl build+test (x86_64-unknown-linux-musl)` | success |
| `test (macos-latest)` | success |
| `lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest)` | success |
| `lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest)` | success |
| `miri (mlock unsafe)` | success |
| `musl build+test (aarch64-unknown-linux-musl)` | still `in_progress` when the abort decision was made — not required, moot since master was not going to be pushed regardless |

### Why `examples` failed (verbatim tail of the failing step's log)

```
##[group]Run git diff --exit-code -- .examples-build/Examples.md
git diff --exit-code -- .examples-build/Examples.md
shell: /usr/bin/bash -e {0}
env:
  GH_TOKEN: ***
  CARGO_HOME: /home/runner/.cargo
  CARGO_INCREMENTAL: 0
  CARGO_TERM_COLOR: always
  CACHE_ON_FAILURE: false
##[endgroup]
diff --git a/.examples-build/Examples.md b/.examples-build/Examples.md
index 12462689..208e4f12 100644
--- a/.examples-build/Examples.md
+++ b/.examples-build/Examples.md
@@ -101,7 +101,7 @@ COMPONENT       CARGO_PACKAGE        DEFAULT      FEATURES       GIT_TAG
 ---------       -------------        -------      --------       -------
 mnemonic        mnemonic-toolkit     git (only)   (none)         mnemonic-toolkit-v0.97.0
 md              md-cli               crates.io    cli-compiler   descriptor-mnemonic-md-cli-v0.11.2
-ms              ms-cli               crates.io    (none)         ms-cli-v0.14.1
+ms              ms-cli               crates.io    (none)         ms-cli-v0.16.0
 mk              mk-cli               crates.io    (none)         mk-cli-v0.12.0
 mnemonic-gui    mnemonic-gui         git (only)   (none)         mnemonic-gui-v0.59.0
 ```
##[error]Process completed with exit code 1.
```

The golden `Examples.md` still encoded the old `ms-cli-v0.14.1` pin; freshly
regenerated output said `v0.16.0` (correctly reflecting the commit's own pin
bump). The golden file was not regenerated/committed as part of that first
commit, so the gate correctly failed.

**Corroborating evidence (non-required `sibling-pin-check` job, same root
cause, not flakiness):**

```
Canonical sibling pins (from scripts/install.sh):
  https://github.com/bg002h/descriptor-mnemonic|descriptor-mnemonic-md-cli-v0.11.2
  https://github.com/bg002h/mnemonic-secret|ms-cli-v0.16.0
  https://github.com/bg002h/mnemonic-key|mk-cli-v0.12.0
  https://github.com/bg002h/mnemonic-gui|mnemonic-gui-v0.59.0
  OK .github/workflows/cross-tool-differential.yml:55: descriptor-mnemonic-md-cli-v0.11.2
  OK .github/workflows/manual.yml:79: mk-cli-v0.12.0
  OK .github/workflows/manual.yml:86: descriptor-mnemonic-md-cli-v0.11.2
##[error]sibling-pin-check: .github/workflows/manual.yml:90: pin 'ms-cli-v0.14.1' (url https://github.com/bg002h/mnemonic-secret) does not match scripts/install.sh canonical 'ms-cli-v0.16.0'
  OK .github/workflows/quickstart.yml:77: mk-cli-v0.12.0
  OK .github/workflows/quickstart.yml:83: descriptor-mnemonic-md-cli-v0.11.2
##[error]sibling-pin-check: .github/workflows/quickstart.yml:87: pin 'ms-cli-v0.14.1' (url https://github.com/bg002h/mnemonic-secret) does not match scripts/install.sh canonical 'ms-cli-v0.16.0'
  OK .github/workflows/technical-manual.yml:109: mk-cli-v0.12.0
  OK .github/workflows/technical-manual.yml:114: descriptor-mnemonic-md-cli-v0.11.2
##[error]sibling-pin-check: .github/workflows/technical-manual.yml:117: pin 'ms-cli-v0.14.1' (url https://github.com/bg002h/mnemonic-secret) does not match scripts/install.sh canonical 'ms-cli-v0.16.0'
  OK docs/manual/src/40-cli-reference/44-mk-cli.md:12: mk-cli-v0.12.0
##[error]sibling-pin-check: one or more sibling pins drifted from scripts/install.sh
##[error]Process completed with exit code 1.
```

So the commit's pin bump from `ms-cli-v0.14.1` → `v0.16.0` in
`scripts/install.sh` was not propagated to at least four other places:
`.github/workflows/manual.yml:90`, `.github/workflows/quickstart.yml:87`,
`.github/workflows/technical-manual.yml:117`, and `.examples-build`'s golden
`Examples.md`.

### Attempt 1 push to master: not attempted

Per the hard rule (all three required contexts must pass before proceeding),
`git push origin g6-lockstep:master` was never run on the first SHA. No final
push output to report there; no bypass message either, since nothing was
pushed. `origin/master` was re-checked after stopping and was unchanged at
`c14b1e21dba83f1052e84e90254724002c3d951c`.

### Attempt 1 cleanup

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-toolkit.git
 - [deleted]           ci/staging
```

Confirmed gone via `gh api repos/bg002h/mnemonic-toolkit/git/refs/heads/ci`
(404) and `git ls-remote origin refs/heads/ci/staging` (empty, exit 0).

---

## Attempt 2 — SUCCESS, master pushed

### What changed between attempts

The coordinator amended the commit (same branch `g6-lockstep`, same worktree)
to fix the propagation gap found by attempt 1. New full SHA:
`27a68e9f12f677288fbd1ed724ad37d092a6c160`. Per `git show --stat`, the amended
commit updates 9 files (73 insertions, 38 deletions), including all five pin
sites plus `design/FOLLOWUPS.md` (one entry flipped to RESOLVED) and expanded
`ci/**` push triggers in `rust.yml`/`examples.yml`:

```
 .examples-build/Examples.md            |  2 +-
 .github/workflows/examples.yml         | 12 +++++-
 .github/workflows/manual.yml           |  2 +-
 .github/workflows/quickstart.yml       |  2 +-
 .github/workflows/rust.yml             | 79 +++++++++++++++++++++-------------
 .github/workflows/technical-manual.yml |  2 +-
 crates/mnemonic-toolkit/src/mlock.rs   |  8 +++-
 design/FOLLOWUPS.md                    |  2 +-
 scripts/install.sh                     |  2 +-
 9 files changed, 73 insertions(+), 38 deletions(-)
```

### Commit staged

- Full SHA: `27a68e9f12f677288fbd1ed724ad37d092a6c160`
- Subject: `g6: bump the ms-cli pin to v0.16.0, converge mlock.rs, and RETIRE the exemption`
- Pre-push verification:
  - `git fetch origin` then `git status` → clean, no tracked modifications, nothing staged.
  - `git log --oneline origin/master..HEAD` → exactly one commit (`27a68e9f ...`).
  - `git merge-base --is-ancestor origin/master HEAD` → exit 0 (fast-forward confirmed).
  - `origin/master` at push time: `c14b1e21dba83f1052e84e90254724002c3d951c` (unchanged from attempt 1 — attempt 1 never touched it).

### Staging push

```
$ git push origin g6-lockstep:refs/heads/ci/staging
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-toolkit/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-toolkit.git
 * [new branch]        g6-lockstep -> ci/staging
```

### Workflow runs on SHA `27a68e9f12f677288fbd1ed724ad37d092a6c160`

| Workflow | Run ID | Run URL | Run-level conclusion |
| --- | --- | --- | --- |
| `rust.yml` ("rust") | 31907712331 | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31907712331 | success |
| `examples.yml` ("examples") | 31907712440 | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31907712440 | success |
| (non-required) "sibling-pin-check" | 31907712363 | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31907712363 | success |

Job conclusions confirmed individually via the jobs API:

#### The three required contexts — ALL GREEN

| Context | Workflow | Conclusion | Job URL |
| --- | --- | --- | --- |
| `examples` | examples.yml | **success** | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31907712440/job/95068006857 |
| `test (ubuntu-latest)` | rust.yml | **success** | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31907712331/job/95068006885 |
| `clippy` | rust.yml | **success** | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31907712331/job/95068006647 |

#### The two jobs called out explicitly in the brief

| Job | Conclusion | Job URL |
| --- | --- | --- |
| `fmt (pinned 1.95.0)` | **success** (with the `mlock.rs` exemption deleted) | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31907712331/job/95068006666 |
| `g6 invariant (cross-repo mlock.rs)` | **success** | https://github.com/bg002h/mnemonic-toolkit/actions/runs/31907712331/job/95068006697 |

#### Other rust.yml + sibling-pin-check jobs — ALL GREEN this time

| Job | Conclusion |
| --- | --- |
| `install.sh harnesses (man-step + MSRV guard)` | success |
| `test (release, ubuntu-latest, mlock einval)` | success |
| `lib cross-platform check (x86_64-pc-windows-msvc, windows-latest)` | success |
| `musl build+test (x86_64-unknown-linux-musl)` | success |
| `musl build+test (aarch64-unknown-linux-musl)` | success (was still running, unresolved, at attempt-1 abort time; now green) |
| `test (macos-latest)` | success |
| `lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest)` | success |
| `lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest)` | success |
| `miri (mlock unsafe)` | success |
| `sibling pins match install.sh` (non-required) | success — the pin-drift finding from attempt 1 no longer reproduces |

**All 3 required contexts + the 2 named jobs + every other job across all
three runs: green.** No red anywhere on this SHA.

### Final push to master

```
$ git push origin g6-lockstep:master
To github.com:bg002h/mnemonic-toolkit.git
   c14b1e21..27a68e9f  g6-lockstep -> master
```

Exit code 0. **No "Bypassed rule violations" message appeared.**

`origin/master` verified moved by fetching fresh from the remote (not by
trusting push output):

```
$ git fetch origin master
From github.com:bg002h/mnemonic-toolkit
 * branch              master     -> FETCH_HEAD
$ git ls-remote origin refs/heads/master
27a68e9f12f677288fbd1ed724ad37d092a6c160	refs/heads/master
$ git rev-parse origin/master
27a68e9f12f677288fbd1ed724ad37d092a6c160
```

Confirmed: `origin/master` now points at `27a68e9f12f677288fbd1ed724ad37d092a6c160`.

### Cleanup

```
$ git push origin --delete ci/staging
To github.com:bg002h/mnemonic-toolkit.git
 - [deleted]           ci/staging
```

Confirmed gone by two independent checks (real exit codes, not inferred from a pipe):

```
$ gh api repos/bg002h/mnemonic-toolkit/git/refs/heads/ci
{"message":"Not Found","documentation_url":"https://docs.github.com/rest/git/refs#get-all-references-in-a-namespace","status":"404"}
# gh exit code: 1

$ git ls-remote origin refs/heads/ci/staging
(no output, exit 0 — ref does not exist)
```

### Tags

No tag was created, pushed, or deleted at any point across either attempt.
`git tag -l` in the worktree lists 226 pre-existing repository tags
(`mnemonic-toolkit-v*`, `manual-*`, `quickstart-*`, etc.) that were already
present before this session and were never touched — no `git tag` command was
run in this session, and neither push used `--tags` or `--follow-tags`.

---

## Summary across both attempts

| | Attempt 1 (`62113694`) | Attempt 2 (`27a68e9f`) |
| --- | --- | --- |
| `examples` | **failure** (stale `ms-cli-v0.14.1` in golden) | success |
| `test (ubuntu-latest)` | success | success |
| `clippy` | success | success |
| `fmt (pinned 1.95.0)` | success | success |
| `g6 invariant` | success | success |
| `sibling-pin-check` (non-required) | failure (same root cause) | success |
| Pushed to master? | **No — correctly aborted** | **Yes** |

The staging procedure did exactly what it exists for: it caught a real
incomplete-propagation defect (pin bumped in one canonical location but stale
in three workflow files and a generated golden) on a required context before
the commit reached master, giving the author a chance to fix it (amend +
regenerate the golden + propagate the pin everywhere) rather than discovering
it after landing.
