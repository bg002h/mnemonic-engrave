# Brief — F-679 toolkit side: pin mnemonic-gui v0.62.0 and ms 0.19.1

## Context

mnemonic-gui **v0.62.0** is released (tag `mnemonic-gui-v0.62.0` at mnemonic-gui
`9f569e1`), pinned to mnemonic 0.104.0, md 0.20.3, **ms 0.19.1**, mk 0.13.0, and
reviewed ready to tag. Reports, all in
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/`:
`f679-impl.md`, `f679-review.md`, `f679-fold1.md`, `f679-fold2.md`, `f679-rereview.md`.

**ms 0.19.1** is released (`ms-cli-v0.19.1`, mnemonic-secret `91d1fd7`). It fixes
`ms verify --phrase <P> <ms1>` failing on 0.19.0, which the GUI's verify flow
depends on. **The toolkit installer must move to ms 0.19.1 in the same change**,
or it ships the ms that breaks the GUI it installs.

## Where you work

Toolkit worktree `/scratch/code/shibboleth/tk-worktrees/f679-681`, branch
`f679-681`, which already carries `65ba3c22` (F-681: `--root` covers binaries
only). Build on it. Don't push master or merge.

## What to do

1. **Pins:** installer GUI pin → `mnemonic-gui-v0.62.0`; ms pin → `ms-cli-v0.19.1`.
   Move every mirror the pin checks require (sibling-pin-check workflows, the
   manual's pinned versions, the `--from-source --dry-run` capture in the
   manual, the Examples golden — regenerate it in the same commit, or the
   required `examples` check goes red).
2. **GUI asset mapping:** check v0.62.0's real asset names and SHA256SUMS
   against the installer's platform table, and re-run `install-assets.test.sh`
   (all mappings, plus the glibc-floor re-check). The x86_64-linux-gnu GUI is
   now built with `cross`, and fold reports claim a glibc floor of 2.18;
   measure it with `readelf`/`objdump` and update the installer's floor table,
   `--help` and the manual. The help text currently says the GUI needs glibc 2.39.
3. **GUI manual** (`docs/manual-gui/`): pin v0.57.0 → v0.62.0, and **rewrite
   `tutorial/50-j4-taproot-twin.md`**. Its step 14 showed a depth-2 taptree being
   refused, but toolkit 0.104.0 exports it, and the GUI renamed the step to
   `tut-j4-14-depth2-export`. The chapter includes corpus files that no longer
   exist. Read the GUI's tutorial corpus at the v0.62.0 tag and make the chapter
   say what the tool now does. Its GUI install lines are tagless (noted in F-676);
   pin them too.
4. **CHANGELOG** entry for these changes.
5. Note, don't fix: the toolkit CHANGELOG at v0.104.0 left shipped changes
   under [Unreleased] (from the F-679 report). Say whether it still does.

## Gates you must actually run

- `install-verify`, `install-msrv-guard`, `install-man-step`,
  `install-assets` test scripts under dash and bash; shellcheck.
- A real `install.sh` into a scratch `--root` **including the GUI**: all five
  at their pins, and `mnemonic-gui --version` = 0.62.0. Then drive
  `ms verify --phrase <P> <ms1>` through the installed ms to show it works (use
  the BIP-39 test vector "abandon ×11 about", which holds no funds).
- `docs/manual` and `docs/manual-gui` builds and audits (point the `*_BIN`
  variables at the scratch root's pinned binaries; the defaults look for
  sibling checkouts that don't exist next to a worktree), the Examples golden,
  the pin checks, fmt/clippy/nextest.

Scratch under `/scratch/code/shibboleth/f679-tk-scratch/`; delete with
`find … -delete` (an `rm -rf` guard blocks recursive forced rm). Pass
`--no-man` or `--man-dir <scratch>` to scratch installs so the user's man pages
aren't overwritten.

## Deliverable

Commits on `f679-681` ending with
`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Don't push master,
merge or tag. **As your final action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f679-toolkit-impl.md`**
and return a short summary plus the path.
