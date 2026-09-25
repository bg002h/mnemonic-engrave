# Brief — re-review of toolkit `f679-681` fold 1

**One question:** did fold 1 (`473bca17`) fix I-1 and M-1..M-3 of the toolkit
review, and did it introduce a defect, above all in its new gate? Not a fresh audit.

## Inputs

- Review: `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f679-toolkit-review.md` (0C/1I/3M).
- Fold: worktree `/scratch/code/shibboleth/tk-worktrees/f679-681`, `git diff 8ed3494f..473bca17`.
- Fold report (claims to re-run): `.../design/agent-reports/f679-toolkit-fold1.md`.
- Pinned binaries: install with this branch's `scripts/install.sh --root <scratch> --no-man`.

## Settled

- The installer pins, the tutorial rewrite and the gates were verified by the
  review; re-check only where the fold touched them.

## Check, with evidence

1. **The new gate** (`docs/manual-gui/tests/check_cli_pins.py`, lint phase 13,
   also wired into `sibling-pin-check.yml`):
   - Can it fail? Re-create at least 3 of its claimed red cases (stale prose
     version; installer-only ms bump; mismatched GUI tag) and show red.
   - Can it pass vacuously? What if `install.sh` or `pinned-upstream.toml`
     can't be parsed, the pin table is empty, or a regex matches nothing?
     It must fail closed, not report "0 findings".
   - Does it actually run in CI on a push that touches only `install.sh`, and
     on one that touches only the manual? Check the workflow's `on:` paths.
   - The 30-line history allowlist: is every entry genuinely historical, or is
     any current claim hiding there? Read all 30.
2. **I-1:** grep the GUI manual yourself for every CLI version string; all
   non-allowlisted ones must equal the pins. The chapter-12 minimums (toolkit
   ≥ 0.104.0, ms ≥ 0.19.1): spot-check that toolkit 0.103.x rejects
   `--allow-argv-secret`, if an old release binary is easy to get; otherwise say so.
3. **M-1..M-3:** re-run 5 of the corrected claims against the pinned binaries.
4. Gates: manual-gui lint, the CLI manual audit, the installer tests under dash.

Scratch under `/scratch/code/shibboleth/review-f679tk2-scratch/`; delete with
`find … -delete`. Don't commit, push or edit the branch.

## Output

Per finding: fixed / not fixed. NEW findings Critical / Important / Minor /
Nit. End with `ready to ship: yes` or `ready to ship: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f679-toolkit-rereview.md`**
and return only a short summary plus that path.
