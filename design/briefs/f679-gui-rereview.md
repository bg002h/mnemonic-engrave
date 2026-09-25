# Brief — re-review of mnemonic-gui `f679-pins` folds 1 and 2

**One question:** did folds 1 and 2 fix each finding of the F-679 review, and
did they introduce a new defect? Then: ready to tag v0.62.0? Not a fresh audit.

## Inputs

- Review: `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f679-review.md` (0C/2I/3M/2N).
- Folds: worktree `/scratch/code/shibboleth/gui-worktrees/f679`,
  `git diff 123f09a..5d9709c` (fold 1 `69e3adb`, fold 2 `5d9709c`).
- Fold reports (claims to re-run): `.../design/agent-reports/f679-fold1.md`, `f679-fold2.md`.
- Release CLIs: mnemonic 0.104.0, md 0.20.3, mk 0.13.0 via
  `sh /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh --no-gui --no-man --root <scratch>`;
  ms **0.19.1** from its GitHub release (`ms-cli-v0.19.1`, check against
  `SHA256SUMS.x86_64`). The installer still pins ms 0.19.0; don't use that one.

## Settled — do not re-derive

- Controller-verified: CI build 36105111445 and schema-mirror 36105111454 both
  succeeded at `5d9709c`. ms 0.19.1 fixes I1 upstream; it was reviewed
  separately (0C/0I) and is released.
- Secret-handling severity rule: never Critical/Important (M3 stays filed).
- `origin/master` already fails `cargo fmt --check` in 82 files.

## Check, with evidence

1. **I1:** the GUI's `ms verify` with phrase + positional ms1 runs at exit 0
   against ms 0.19.1; the new test fails against 0.19.0 (re-run it).
2. **I2 and the "always `--`" change:** fold 1 now puts `--` before
   positionals on EVERY command, justified by 20 probes. Re-probe a sample of
   at least 8 across all four CLIs, including a positional that starts with
   `-`, an empty positional, and ms secret positionals with and without
   `--allow-argv-secret`. Does anything the GUI shows (Copy, Preview, confirm)
   differ from what runs? Is `--allow-argv-secret` ever emitted after `--`,
   where ms would treat it as a positional?
3. **M1, M2, N1:** fixed as described? Try to defeat M1's first-source-wins
   rule (fill several sources, clear one, reorder).
4. **Tutorial and snapshots:** fold 1 says 7 screenshots regenerated
   byte-identical. Confirm, and check the tutorial harness still passes.
5. Suite, clippy (both configs), MSRV 1.88.0; spot-check 3 mutations of the
   new tests.

Scratch under `/scratch/code/shibboleth/review-f679b-scratch/`; delete with
`find … -delete` (an `rm -rf` guard blocks recursive forced rm). Don't commit,
push, tag or edit the branch.

## Output

Per finding ID: fixed / not fixed. NEW findings as Critical / Important /
Minor / Nit. End with `ready to tag: yes` or `ready to tag: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f679-rereview.md`**
and return only a short summary plus that path.
