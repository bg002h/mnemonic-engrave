# Brief — F-694 part 1: mnemonic-gui re-pin to the current CLIs → v0.63.0

Context: mnemonic-gui `master` (`3ca1d60`, CI green) carries the secret-channels
design and the five new forms, still pinned to mnemonic 0.104.0 / md 0.20.3 /
ms 0.19.1 / mk 0.13.0. Since then the CLIs released:
- **mnemonic-toolkit 0.105.1**, **ms-cli 0.20.1**: F-687 (`--passphrase -` = stdin
  and `@env:VAR` = environment on every password flag; one trailing-newline rule;
  empty-value warning; terminal prompt; leftover paste drained with a masked
  preview), F-689, F-691, F-683 (`--separator` offers only `space`), F-693.
- md 0.20.3 and mk 0.13.0 are unchanged.

The design (`design/DESIGN_secret_channels_and_new_forms.md`, §A0) makes this
pin bump a **re-measure plus data update**: `regen_check.py` re-derives every
cache against the pinned binaries (keyed by tag + sha256), and the planner reads
only data and the fixed policy decisions. If the bump needs a code change,
something in the design was wrong: stop and report it rather than work around it.

## Do

Worktree `/scratch/code/shibboleth/gui-worktrees/repin063`, branch
`repin-063` off origin master.

1. Install the pinned release binaries into a scratch root:
   `sh /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh --no-gui --no-man --root <scratch>`
   (it now installs toolkit 0.105.1, md 0.20.3, ms 0.20.1, mk 0.13.0).
2. Move every pin (`pinned-upstream.toml`, `Cargo.toml`'s toolkit dependency
   tag, the schema-mirror version strings, and the README) and update the
   schema mirror for any flag-surface change (e.g. ms `--separator` is now a
   dropdown `[space]`). Record each decision.
3. Re-derive the measurement caches with the harness; commit the regenerated
   caches. Report the channel-table diff (the ~17 former WRONG cells should
   become OK channel cells) and confirm `regen_check.py --plans` is GREEN.
4. The operator's rulings now in the CLIs: an empty value warns; a terminal
   prompt; the paste drain with a masked preview. Make sure the GUI (which
   never runs the CLIs on a terminal) is unaffected, and that the empty-value
   warning on stderr is shown sensibly.
5. Prepare release **v0.63.0**: version and CHANGELOG. Don't tag.

## Gates

Full suite against the pinned binaries (nextest, never `--release`), clippy
(both configs), MSRV 1.88.0, tutorial harness, form snapshots (explain every
changed PNG), T10, the mutation harness, every measurement script, and CI on the
branch with run ids. Don't merge or tag. Scratch under
`/scratch/code/shibboleth/gui-repin-scratch/`; delete with `find … -delete`.

## Deliverable

Commits ending with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
**As your final action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f694-gui-repin.md`**
and return a short summary plus the path.
