# Brief — release prep: mnemonic-secret 0.20.0 and mnemonic-toolkit 0.105.0

F-687 (all parts: `-`/`@env:` channels on every password flag, the newline rule,
the empty warning, the terminal prompt, draining leftover input with a masked
preview, F-689, F-691) plus F-683, F-684, F-686 are merged on both masters and
reviewed. This brief prepares the release commits; **the controller tags**.

## Versions

- **ms-cli 0.20.0**: minor bump, because `--passphrase -` and `@env:` now
  derive different wallets than 0.19.x (a breaking behaviour change). If other
  crates in the ms workspace changed since `ms-cli-v0.19.1`, bump each per the
  repo's convention, and say which.
- **mnemonic-toolkit 0.105.0**: minor, same reason.

## What to do

Worktrees on branch `release-f687` off origin master:
`/scratch/code/shibboleth/ms-worktrees/release` and
`/scratch/code/shibboleth/tk-worktrees/release`.

1. **ms:** bump versions and `Cargo.lock`, and move `[Unreleased]` in the
   CHANGELOG to `## [0.20.0]` with today's date. Run `ci/repro/vendor-freshness.sh`
   if `Cargo.lock` moved (the check is non-required, so the push won't stop for it).
2. **toolkit:** bump to 0.105.0, move `[Unreleased]` under `0.105.0`. The
   installer (`scripts/install.sh`) self-pin goes to `mnemonic-toolkit-v0.105.0`,
   and the ms pin to `ms-cli-v0.20.0`. Update every mirror the sibling-pin check
   requires, the manual's pinned versions, the `--from-source --dry-run`
   capture, and the **Examples golden** (regenerate in the same commit, or the
   required `examples` check goes red). The GUI-manual lint's
   `check_cli_pins.py` compares the manual against `install.sh`. mnemonic-gui
   v0.62.0 still pins the old CLIs, so decide how to keep that gate
   consistent: the GUI manual documents GUI v0.62.0's pins until the GUI
   re-pins. Explain what you did.
3. **The pin-order problem:** the installer can only pin `ms-cli-v0.20.0` and
   `mnemonic-toolkit-v0.105.0` once those tags exist with release assets. So
   produce the commits in this order and **stop between them**:
   (a) ms release commit; (b) toolkit release commit containing everything
   except the installer's pins for the two new tags; (c) a prepared, uncommitted
   patch (or a second commit on a separate branch) that moves the installer
   pins, to be applied once the tags are published. Describe the sequence
   precisely in the report.

## Gates

Per repo: nextest, pinned clippy, fmt; toolkit goldens and `make audit`.
Don't push, merge or tag. Scratch under
`/scratch/code/shibboleth/release-scratch/`; delete with `find … -delete`.

## Deliverable

Commits ending with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
**As your final action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/release-ms020-tk0105.md`**
with the exact tag names, tag commits and the step-by-step publish sequence.
Return a short summary plus the path.
