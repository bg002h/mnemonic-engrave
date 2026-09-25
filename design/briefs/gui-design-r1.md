# Brief — R1 re-review: mnemonic-gui design fold 1

**One question:** did fold 1 (`2d244d2`) fix every R0 finding and correctly
implement the final F-687 ruling in the design, and did it introduce a new
defect? Then: is the design GREEN (0C/0I) to implement? Not a fresh audit.

## Inputs

- Design: `/scratch/code/shibboleth/gui-worktrees/followups/design/DESIGN_secret_channels_and_new_forms.md`
  at `2d244d2`, with `design/measurements/secret-channels/` (now including
  `plan.py`, `run_plans.py`, `check_design_tables.py`, `channel_table.json`).
- R0 review: `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-design-r0.md`.
- Fold map: `.../design/agent-reports/gui-design-fold1.md`.
- F-687 ruling (engrave FOLLOWUPS F-687): `--passphrase -` = stdin and
  `--passphrase @env:VAR` = environment in both CLIs, being implemented now,
  not in today's releases. Nothing refused; a literal argv passphrase warns.
- Release CLIs: `sh /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh --no-gui --no-man --root <scratch>`.

## Settled

- R0's channel table re-measured byte-identical; the literal-value cells are real.
- Secret-handling severity rule: exposure never gates; wrong flag or wrong
  wallet gates.

## Check, with evidence

1. **C1 as designed now:** the GUI resolves `@env:VAR` itself and refuses `-`
   (it has no stdin to forward). Re-run the fold's measurements (`run_plans.py`,
   the 56/56 `@env:` runs, the `restore` fingerprints `ca2c62d2` vs `fee4dc32`
   vs `66d564d1`). Is there any secret field where a typed `@env:` or `-` can
   still reach a CLI literally? Is `MNEMONIC_GUI_*` refusal enough to stop one
   source reading another's secret? What does Copy command print for an
   `@env:` field?
2. **Is the ruling reflected correctly?** After the F-687 pin bump, the design
   says only `channel_table.json` changes. Is that true, or is there
   resolution code that would then double-resolve (the GUI reads `VAR`, then
   the CLI also treats a value as a channel)?
3. **I1:** is `plan.py` the single source, with A5 generated from it? Run
   `check_design_tables.py`, then mutate the plan and prove it goes red. The
   10 unmeasured sources must refuse; find any path where one falls back to argv.
4. **I2:** re-run T3′ (`run_plans.py`). Make a deliberately swapping runner
   (swap two sources) and show T3′ fails on the asymmetric pairs, as the fold
   claims for all 28. For the 4 output-invisible share-set swaps, does T7
   (runner echo) actually catch a swap?
5. **I3:** fd channel Linux-only; macOS/Windows refuse the three pipe shapes.
   Is the rule "replace the argv admission only once that OS has a green
   real-binary CI job" enforceable in the plan, or only prose?
6. Spot-check that 3 Minors are fixed as the map says.

Scratch under `/scratch/code/shibboleth/review-gui-r1-scratch/`; delete with
`find … -delete`. Don't commit, push or edit the branch.

## Output

Per R0 finding: fixed / not fixed. NEW findings Critical / Important / Minor /
Nit with evidence. End with `GREEN (0C/0I)` or `NOT GREEN`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-design-r1.md`**
and return only a short summary plus that path.
