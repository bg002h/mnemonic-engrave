# Brief — F-680: turn the aarch64 reproducibility checks on (md, ms, mk, toolkit)

Read F-680 in `/scratch/code/shibboleth/mnemonic-engrave/design/FOLLOWUPS.md`
(search `### F-680`). The operator ruled on 2026-09-24: **turn the aarch64
reproducibility checks on in all four callers**, and **no backfill** of md 0.20.3.

## Measured facts (controller)

Callers of the toolkit's reusable `reproducible-musl-build.yml`, all passing
`run_aarch64: false`:

| repo | file | pinned toolkit ref |
|---|---|---|
| descriptor-mnemonic | `.github/workflows/man-pages.yml` (~l.139/149) | `6e37b18e…` |
| mnemonic-secret | `.github/workflows/man-release.yml` (~l.134/144) | `d39d9626…` |
| mnemonic-key | `.github/workflows/musl-binaries.yml` (~l.68/78) | `6e37b18e…` |
| mnemonic-toolkit | `.github/workflows/man-pages.yml` (~l.129) | same repo |

Toolkit `master` is now `4120af85`, which contains F-675 (`327f9a16`): the
aarch64 remap-off "zero residue" check had been failing for the wrong reason (a
pipefail/SIGPIPE race), and cc-validate's zero-residue check had the same shape
as a false GREEN. **Callers must pin a toolkit ref that contains `327f9a16`**,
or the aarch64 leg runs the unfixed checks. Verify ancestry; don't assume it.

The aarch64 leg is QEMU-emulated, about 30–60 min.

## What to do

For each of the four repos, in its own worktree on a new branch `f680-aarch64`
off `origin/<default branch>` (under `/scratch/code/shibboleth/<repo-prefix>-worktrees/f680`):

1. Set `run_aarch64: true`. For md, ms and mk, bump the reusable-workflow pin
   to a toolkit SHA containing `327f9a16` (use `4120af85` unless you find a
   reason not to; say which).
2. Update comments that say aarch64 is exercised only on demand, or on a
   schedule. A comment that becomes false is a finding.
3. Check the caller passes everything the pinned reusable workflow now expects
   (inputs may have changed between the old pins and `4120af85`, e.g. the
   `git_source_url`/`git_source_rev` inputs F-675 added). Diff the reusable
   workflow's `inputs:` between the old pin and the new one.
4. Push the branch and run the caller workflow via `workflow_dispatch` without
   a tag. **Gate: `repro-aarch64-musl` must run and conclude `success`,
   not skipped**, alongside every other repro/musl leg. Run the four in parallel.
5. **Prove the aarch64 check can fail.** In at least one repo, show from the
   run log that the aarch64 negative check (remap-off residue) found residue
   and the positive checks compared real hashes. Quote the lines. A green that
   compared nothing is a failure of this task.

## Do not

- tag, upload to releases, or push/merge to any default branch;
- touch `ensure-release`/`upload` steps or their tag-only guards;
- edit anything outside the caller workflows (and their comments) without
  saying why in the report.

If a repo's aarch64 leg fails for a real reason, stop on that repo, report the
cause with log lines, and keep going on the others.

## Deliverable

Commit per repo on `f680-aarch64`, ending each message with
`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. **As your final
action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f680-impl.md`**:
per repo the branch, commit, run id, per-leg conclusions, and the input diff;
the can-fail evidence; concerns. Return a short summary plus the path.
