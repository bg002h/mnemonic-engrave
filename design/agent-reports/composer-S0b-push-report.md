# composer-S0b push report

**Task:** merge `composer-s0b` into `main` (fast-forward) and push via the
`ci/staging` ritual, per
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-briefs/composer-S0b-push-brief.md`.

## Preconditions verified

- Main checkout `/scratch/code/shibboleth/descriptor-mnemonic`, branch `main`.
- `git rev-parse main` (before merge): `66bdf2f47e7fc703d5fb09120122b3e98cab5528`
- `git status --short` (before merge): empty
- Branch tip (`composer-s0b`, worktree `/scratch/code/shibboleth/wt-composer-s0b`):
  `1dc8d409f6e8daa099226937f5f107f56b64dd97`
- `git merge-base --is-ancestor main composer-s0b`: true (fast-forward possible)
- `git log --oneline 66bdf2f4..composer-s0b` (oldest to newest), matches the
  four expected commits exactly:
  ```
  4793619b md-codec: six preset MANIFEST vectors -- one per archetype, built by calling presets::*, singular preset:<name> tags (F-453 composer S0b task 1)
  5002ebac md-cli: md compose --preset -- the six archetypes over presets::*, mutually exclusive with --path, --json preset field (F-453 composer S0b task 2)
  87bc10ff md-codec/md-cli: preset vector corpus regenerated; release notes (F-453 composer S0b task 3)
  1dc8d409 md-cli/md-codec: fold the S0b whole-diff review's Minors -- an unknown preset parameter is named before the k-of-n count is checked (M-2), --path's older=<n>u / after=<t>t spellings refuse with the --path remedy (M-3), SINGULAR_TAGS states its two grounds (M-1), md compose gets its README row (N-3)
  ```

## Merge

`git merge --ff-only composer-s0b` in the main checkout: fast-forward,
`Updating 66bdf2f4..1dc8d409`. Post-merge `git rev-parse main` =
`1dc8d409f6e8daa099226937f5f107f56b64dd97`; `git status --short` empty.

## Staging run

An initial background run of `scripts/push-via-staging.sh main` pushed
`ci/staging` at `1dc8d409` and started waiting on the required contexts. The
coordinator asked to finish the ritual in the foreground instead of leaving
the watch backgrounded. Before switching to the foreground steps, the entire
background process tree (the script, its `bash` child, and the `gh run watch`
it had spawned) was killed with `kill -TERM`, confirmed dead (background task
exit code 144), and `ps -ef` confirmed no residual `push-via-staging` / `gh
run watch` processes before proceeding — this avoided a race between the
backgrounded watch and the foreground push.

- Staging branch: `ci/staging` at `1dc8d409f6e8daa099226937f5f107f56b64dd97`
  (created by the killed background run's earlier push; not re-pushed).
- CI run id: **33698441737** (workflow "CI"), `headBranch=ci/staging`,
  `headSha=1dc8d409f6e8daa099226937f5f107f56b64dd97`,
  url: https://github.com/bg002h/descriptor-mnemonic/actions/runs/33698441737
  (createdAt 2026-09-03T00:11:35Z, updatedAt 2026-09-03T00:19:53Z).
- `gh run watch 33698441737 --repo bg002h/descriptor-mnemonic --exit-status`
  ran to completion in the foreground (no backgrounding).
- Per-job conclusions, from
  `gh run view 33698441737 --repo bg002h/descriptor-mnemonic --json status,conclusion,jobs`
  (verbatim):

  ```
  run status: completed conclusion: success
  'cargo doc': status=completed conclusion=success
  'cargo test (ubuntu-latest)': status=completed conclusion=success
  'musl compile/test (aarch64-unknown-linux-musl)': status=completed conclusion=success
  'cargo clippy': status=completed conclusion=success
  'cargo fmt': status=completed conclusion=success
  'freebsd compile-gate (whole-crate)': status=completed conclusion=success
  'musl compile/test (x86_64-unknown-linux-musl)': status=completed conclusion=success
  'cargo test (windows-latest)': status=completed conclusion=success
  'cargo test (macos-latest)': status=completed conclusion=success
  ```

  Both required contexts (`cargo test (ubuntu-latest)`, `cargo clippy`) =
  `success`; overall run conclusion = `success`.

## Final push

`git push origin main` from the main checkout, output (verbatim):

```
To github.com:bg002h/descriptor-mnemonic.git
   66bdf2f4..1dc8d409  main -> main
```

No "Bypassed rule violations" line.

`git push origin --delete ci/staging` output (verbatim):

```
To github.com:bg002h/descriptor-mnemonic.git
 - [deleted]           ci/staging
```

## Post-push verification

`git fetch origin && git rev-parse origin/main`:
**`1dc8d409f6e8daa099226937f5f107f56b64dd97`** — matches the branch tip
exactly.

## What was not done / deviations from the brief

- The brief specified running `scripts/push-via-staging.sh main` end to end
  and leaving its watch to complete (backgrounded or not, at the agent's
  discretion). The coordinator interrupted mid-run and directed that the
  remainder be completed as discrete foreground commands (kill the
  background watch, `gh run watch --exit-status` in the foreground, judge
  jobs via `gh run view --json jobs`, then `git push origin main` /
  `git push origin --delete ci/staging` / fetch-and-confirm by hand) instead
  of via the script's own final-push step. The net effect is identical: the
  same staging run (33698441737) at the same SHA (1dc8d409) satisfied the
  same two required contexts before `main` was pushed, and the push carried
  no bypass.
- No tag, version bump, or `cargo publish` was performed (per the brief;
  md-codec's publish remains blocked by the separate follow-up
  `md-codec-derive-feature-depends-on-unpublished-miniscript-apis`).
- The worktree `/scratch/code/shibboleth/wt-composer-s0b` was left in place,
  untouched.
- No source file was modified; no commit was made by this agent (the merge
  was a fast-forward, so no new commit object was created).
