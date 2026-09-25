# Brief — F-679 implementer: a mnemonic-gui release pinned to today's CLIs

Read F-679 in `/scratch/code/shibboleth/mnemonic-engrave/design/FOLLOWUPS.md`
(search `### F-679`). Goal: **one** mnemonic-gui version that is pinned to,
and exercised against, the current CLIs, so the toolkit installer and the GUI
manual can both name it. This brief covers the GUI side only; the controller
does the toolkit side afterwards.

## Target CLI versions (released, measured 2026-09-24)

| CLI | version | tag |
|---|---|---|
| mnemonic | 0.104.0 | `mnemonic-toolkit-v0.104.0` |
| md | 0.20.3 | `descriptor-mnemonic-md-cli-v0.20.3` |
| ms | 0.19.0 | `ms-cli-v0.19.0` |
| mk | 0.13.0 | `mk-cli-v0.13.0` |

Installed locally at those exact versions: `~/.cargo/bin/{mnemonic,md,ms}`.
**The local `mk` is NOT the release** (built from mk main before F-677; it has
`--in` flags 0.13.0 lacks). Get release binaries for all four into a scratch
root with the toolkit installer, and point the GUI's tests at them:
`sh /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh --no-gui --no-man --root <scratch>`.

## Where you work

`/scratch/code/shibboleth/mnemonic-gui` is on `master`, clean, 8 commits past
`mnemonic-gui-v0.61.0`. Make a worktree at
`/scratch/code/shibboleth/gui-worktrees/f679` on a new branch `f679-pins` off
`origin/master`. Read the repo's `CLAUDE.md`, `pinned-upstream.toml` and
`.github/workflows/schema-mirror.yml` first; the pin map and the schema-mirror
gate are how this repo keeps the GUI honest about CLI surfaces.

## What to do

1. Move every pin (`pinned-upstream.toml`, and the toolkit dependency tag in
   `Cargo.toml`, which is the load-bearing one) to the table above.
2. Make the schema mirror and snapshots match the new CLI surfaces. Every
   new, removed or renamed flag or subcommand is a GUI decision: expose it,
   deliberately hide it, or drop it. **List each one in the report with the
   decision and why.** Never silently regenerate a snapshot to make a gate green.
3. Anything secret-bearing (seeds, phrases, preimages, passphrases) the new
   surfaces add must follow the GUI's existing secret-handling rules; check how
   the repo does it and say so per item.
4. Prepare the release: bump the version (0.62.0 unless the repo's conventions
   say otherwise), write the CHANGELOG entry. **Do not tag, publish, push or
   merge to master.**
5. **Glibc floor (from the F-676 review):** the v0.59.0 x86_64-linux GUI binary
   needs GLIBC_2.39, so it refuses on Debian 12, Ubuntu 22.04 and RHEL 9. Find
   what sets that (runner image?). If lowering the floor is a small release-
   workflow change, do it and prove the new floor with `objdump -T` on a
   release-shaped build; if not, report what it would take.

## Gates you must actually run

- The repo's full test suite (nextest if it's Rust; never `--release`), clippy
  `-D warnings`, fmt, and the schema-mirror gate against the four release
  binaries above.
- Launch the GUI headless if the repo supports it, or explain what you ran in
  place of that, and drive at least one flow per CLI end to end against the
  release binaries.
- Push the branch and run CI on it (workflow_dispatch or the push trigger);
  record the run id and per-job results.

Scratch goes under `/scratch/code/shibboleth/f679-scratch/`; delete it with
`find … -delete` at the end (an `rm -rf` guard blocks recursive forced rm).

## Deliverable

Commits on `f679-pins`, each ending with
`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. **As your final
action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f679-impl.md`**:
the pin changes, the per-surface decisions, gate commands with results, the
glibc finding, and concerns. Return a short summary plus the path.
