# Brief — mnemonic-gui follow-ups, phase 1: two fixes + one design doc

The operator asked for these mnemonic-gui follow-ups to be done. Read them in
`/scratch/code/shibboleth/mnemonic-gui/FOLLOWUPS.md`:

- `restore-from-secret-node-unmasked-and-persisted`
- `argv-secret-via-private-channels`
- `md-ms-new-subcommands-unsurfaced`

and F-685 in `/scratch/code/shibboleth/mnemonic-engrave/design/FOLLOWUPS.md`
(the GUI CHANGELOG has no 0.60.0 or 0.61.0 entry).

Worktree `/scratch/code/shibboleth/gui-worktrees/followups` on branch
`gui-followups` off `origin/master` (v0.62.0 + merge, `9f569e1`).

## Phase 1a — implement now

1. **F-685:** reconstruct the 0.60.0 and 0.61.0 CHANGELOG entries from the tag
   messages and `git log mnemonic-gui-v0.59.0..mnemonic-gui-v0.61.0`. State
   only what the commits show.
2. **`restore-from-secret-node-unmasked-and-persisted`:** a `--from` value of
   the form `<node>=<v>` whose node is an argv-secret node (`ms1=`, `phrase=`,
   `wif=`, `xprv=`, …; use the existing `SECRET_NODE_TYPES_ARGV`, don't write a
   second list) must be treated as secret at ALL four sites: Preview masking,
   run-confirm, persistence redaction, and widget masking. Tests first. For
   each site: a test that fails on today's tree with a real secret node, and a
   test that a non-secret node (e.g. `xpub=`) is NOT masked or redacted
   (over-masking a public value hides what the user needs to check).
   Mutation-check each site.

## Phase 1b — DESIGN ONLY, no code

Write `design/DESIGN_secret_channels_and_new_forms.md` in the GUI repo, covering:

- **Private channels** (`argv-secret-via-private-channels`): route each secret
  over stdin (`-` sentinel), `--in FILE`, or `@env:VAR` instead of argv, and
  drop the GUI-managed `--allow-argv-secret` where possible. Needed: a
  per-subcommand, per-flag table of which channel each CLI accepts, **measured
  against the release binaries** (mnemonic 0.104.0, md 0.20.3, ms 0.19.1,
  mk 0.13.0; install them with
  `sh /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh --no-gui --no-man --root <scratch>`,
  which pins ms 0.19.1). Also needed: the rule for commands carrying two or
  more secrets (one stdin per invocation); how a temp file for `--in` is
  created, protected and removed, including on crash or cancel; what the
  confirm dialog, Preview and Copy command show; and the failure cases where
  the GUI must refuse rather than fall back. **The critical property is that
  every secret reaches exactly the flag the user filled.** Say how that is
  tested.
- **The five forms** (md `compose`, `shape-key`, `descriptor`, `decompose`;
  ms `hashlock`): for each, the flags from the release binary's `gui-schema`/
  `--help`, which are secret (hashlock's phrase is), required/conditional
  rules, and whether it fits the existing form machinery or needs new widget
  kinds. Keep the design proportionate: a table per form, not an essay.

Stop after writing it. The controller will have it reviewed before any of it
is implemented.

## Gates for phase 1a

Full suite against the release binaries (nextest if available, never
`--release`), clippy `-D warnings` in both feature configs, MSRV 1.88.0, the
tutorial harness, form snapshots (if a form PNG changes, explain why), and CI
on the branch. Don't tag or merge. Scratch under
`/scratch/code/shibboleth/gui-followups-scratch/`; delete with `find … -delete`.

## Deliverable

Commits on `gui-followups` ending with
`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`, the design doc
committed too. **As your final action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-followups-phase1.md`**
and return a short summary plus the path.
