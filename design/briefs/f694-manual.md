# Brief — F-694 part 2: the GUI manual for the secret channels and the five new forms

Read F-694 in `/scratch/code/shibboleth/mnemonic-engrave/design/FOLLOWUPS.md`.

The GUI's help icons deep-link into
`https://bg002h.github.io/mnemonic-toolkit/manual-gui/#<anchor>`. The live
manual is `manual-gui-v1.3.1` and has no anchors for the GUI's new forms.
mnemonic-gui `master` (`3ca1d60`) has:
- **five new forms:** md `compose`, `shape-key`, `descriptor`, `decompose`;
  ms `hashlock`;
- **secret channels:** on Linux every secret goes privately (stdin, env, fd).
  `-`/`@env:VAR` typed in a secret field are resolved (`-` refused, since the GUI
  has no stdin). There are lookalike and CR/LF refusals. Preview, Copy and the
  confirm dialog show where each secret comes from. Copy never embeds a secret.
  macOS/Windows use the interim argv path.

The design (`design/DESIGN_secret_channels_and_new_forms.md` in mnemonic-gui)
assigns "five manual pages" to a paired toolkit PR. A separate agent is
re-pinning the GUI to today's CLIs as **v0.63.0** in parallel. Write against
GUI `master` now; the pin numbers get updated at the end.

## Do

Toolkit worktree `/scratch/code/shibboleth/tk-worktrees/f694`, branch
`f694-manual-gui` off origin master (`af5cc1a5`).

1. **Five form pages** in `docs/manual-gui/`, matching the existing per-form
   page structure (sections, anchors, figures, transcripts). Every anchor the
   GUI's help icons produce for these forms must exist. Derive the anchor list
   from the GUI's own code (`MANUAL_BASE_URL` and the anchor builder), not by
   guessing. Use the GUI repo's form snapshots at `master` for figures, the way
   the existing pages do.
2. **A secret-channels section:** what the user sees in Preview, Copy and the
   dialog; what `-` and `@env:VAR` mean in a secret field; the refusals and why;
   the Linux-vs-other-OS difference. Every behavioural claim must be checked by
   running the GUI (kittest/headless) or quoting the GUI's code; a manual that
   tells a user a refused input is accepted, or vice versa, is a defect.
3. Update the CLI-facing statements the F-687 releases changed: `--passphrase -`
   / `@env:` meaning, the empty warning, the terminal prompt and the paste drain,
   wherever the GUI manual describes CLI behaviour. The lint phase 13
   (`check_cli_pins.py`) must stay green.
4. **Stop before pinning.** The last step (GUI pin → v0.63.0, removing the
   `[installer-ahead]` lag table and §82 lag note, tagging `manual-gui-v*`) waits
   for the GUI release. List exactly what that step will change.

## Gates

`docs/manual-gui` `make lint` (all phases; `MANUAL_GUI_UPSTREAM_ROOT` pointed at a
clean `git archive` of mnemonic-gui `master`, and the `*_BIN` variables at a
scratch install from the toolkit installer), verify-examples-gui, the form-render
census, the PDF/HTML builds, and the CLI manual `make audit`. Don't push master,
merge or tag. Scratch under `/scratch/code/shibboleth/f694-manual-scratch/`;
delete with `find … -delete`.

## Deliverable

Commits ending with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
**As your final action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f694-manual.md`**
with the full anchor list (GUI-produced vs. present), the claims checked and how,
and the pending pin step. Return a short summary plus the path.
