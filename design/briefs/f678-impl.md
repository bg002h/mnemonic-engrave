# Brief — F-678 implementer (md x86_64 musl release leg)

Read the F-678 entry in
`/scratch/code/shibboleth/mnemonic-engrave/design/FOLLOWUPS.md` (search `### F-678`).

## Where you work

descriptor-mnemonic worktree `/scratch/code/shibboleth/dm-worktrees/f678`,
branch `f678-musl`, based on `origin/main` at `0e5f31d9`. Nobody else is
editing this repo right now. Do not touch other repos.

## What is measured (controller, 2026-09-24)

- Tag run 36084793496 (`descriptor-mnemonic-md-cli-v0.20.3`):
  `musl-binary (x86_64-unknown-linux-musl)` **failed** — in the repro container
  with `--network=none`: `error: failed to load source for dependency
  \`miniscript\``. `musl-binary (aarch64…)` succeeded; `repro-aarch64-musl`
  was **skipped**; `repro-x86_64-musl` and `repro-substrate` succeeded.
- 0.20.2's run failed the same leg the same way.
- The last md release with `md-*-x86_64-linux-musl.tar.gz` and
  `SHA256SUMS.x86_64` is 0.12.0 (0.15.0–0.20.3 have neither).
- These jobs appear to live in `.github/workflows/man-pages.yml`; confirm it.

## Precedent — read before designing

mnemonic-toolkit fixed the same class in F-675, commit `327f9a16` (in
`/scratch/code/shibboleth/tk-worktrees/f675-677`): an offline build whose git
dependency no `[source]` replacement stanza covered. Its report is
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/toolkit-docs-green-impl.md`
and the F-675 FOLLOWUPS entry. Also note from that entry: a check that passes
**locally** can pass falsely because cargo's git cache already holds the rev —
reproduce offline in a clean `CARGO_HOME`, not in your normal one.

## Questions to answer, in order

1. Why does the x86_64 leg fail while aarch64 `musl-binary` succeeds and
   `repro-x86_64-musl` succeeds? The difference is the defect; find it
   exactly before changing anything.
2. Why is `repro-aarch64-musl` skipped on a tag push? If the skip means a
   reproducibility gate never runs on releases, say so. Fix it if it's in scope
   and cheap; otherwise report it.
3. Fix the x86_64 leg.

## Gates you must actually execute

- Reproduce the failure locally first (same container digest, `--network=none`,
  clean `CARGO_HOME`), then show your fix makes it pass the same way.
- Push your branch and run the workflow via `workflow_dispatch` **without a
  tag** (the man-pages workflow documents this as the reproducibility gate).
  Every musl and repro leg must be green, and not skipped without a stated reason.
  Record the run id.
- Do **not** backfill or upload to any existing release, and do not create
  tags. Backfilling 0.20.3's missing assets is a controller decision; say in
  your report exactly what command would do it.
- Stage paths explicitly. Do not merge to main or push main.

## Deliverable

Commit on `f678-musl` (end messages with
`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`). **As your final
action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f678-impl.md`**:
root cause with evidence, the fix, each gate's command and run id/output, the
aarch64-skip finding, the backfill command, concerns. Return a short summary
plus the path.
