# Fork `main` push — 2026-08-15

## Task

Push `main` in `/scratch/code/shibboleth/seedhammer` (the SeedHammer fork,
`bg002h/seedhammer`) to `origin`. Branch protection was pre-verified by the
dispatcher (`gh api repos/bg002h/seedhammer/branches/main/protection` → 404
"Branch not protected"), so no `ci/staging` dance was used — a plain
`git push origin main` was correct.

## Pre-push verification

Commit count, `origin/main..HEAD`:

```
$ git log --oneline origin/main..HEAD
6922b43 S3b: the guard is a face lookup, not a list of runes that already bit us
c252b5b S3b: 27 runes the faces cannot draw, removed from the screens
d64e4ff S3: the walk that reads the restore doc
6400e1f S3: the stale seed-entry comments die (2.2 D-5)
4587d06 S3: nested segwit is NAMEABLE, and the restore doc says so
43a07fe S0b re-review fold (I-1): an expectation must be self-describing, not just named
05c5a73 S0b fold, operator directive: delete the opt-out; the live checks do not exist unless asked for
af00360 S0b fold (I-1, I-2): `ok` may hold only what the emulator was seen to produce, and a needle a walk declares must be one the tree has counted
9f792c3 S0b fold (C-1, C-2, C-3): a byte-identity gate that CANNOT skip, and a record that cannot be minted uncompared
afca974 S0b fold (C-4): vendor the sysw conformance vectors; the skip dies
```

10 commits confirmed — matches the S0b fold (4 commits) + re-review fold (1) +
S3/S3b (5 commits) breakdown given in the task.

Tracked-file cleanliness, `git status --porcelain`: **empty output** — no
modified or staged tracked files. (The sibling worktree at
`/scratch/code/shibboleth/seedhammer-s3` was not touched.)

## Push

Commit range pushed: `4b8488e..6922b43`

Verbatim push output:

```
$ git push origin main
To github.com:bg002h/seedhammer.git
   4b8488e..6922b43  main -> main
EXIT_CODE=0
```

## Post-push verification

```
$ git fetch origin && git log --oneline -1 origin/main
6922b43 S3b: the guard is a face lookup, not a list of runes that already bit us
EXIT_CODE=0
```

`origin/main` confirmed moved to `6922b43c9cf2caeb619f2e78707cdd46a74c1e0b`
(full SHA), matching local `HEAD`.

## CI

Two workflow runs triggered on push for this SHA:

- **Test** (`.github/workflows/test.yml`) — the one the task asked about.
  - Run: https://github.com/bg002h/seedhammer/actions/runs/31908409126
  - Started: 2026-08-15T21:04:08Z, completed: 2026-08-15T21:07:03Z (~3 min)
  - **Conclusion: success**
- **Build image** — run id 31908409100, was also in progress at push time;
  not tracked to completion (out of scope — task named `test.yml`
  specifically).

Note on tooling: `gh api "…/actions/runs?head_sha=<short-sha>"` returned
`total_count: 0` for the abbreviated 7-char SHA — the endpoint requires the
**full** 40-char SHA to match. Using the full SHA
(`6922b43c9cf2caeb619f2e78707cdd46a74c1e0b`) returned both runs correctly.
`gh run list` was not used per the dispatcher's note that it has returned
stale listings today.

## Result

Push complete, `origin/main` moved, CI Test run **green**.
