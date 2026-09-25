# Brief — R3 re-review: mnemonic-gui design fold 3

**One question:** did fold 3 (`42dacaa`) fix NC1, NI3, NI4 and the R2 Minors
and Nits, and add a working pin-bump check, without introducing a defect?
GREEN (0C/0I) or not. Scope is fold 3's changes only. Earlier rounds verified
the rest; don't re-audit it.

## Inputs

- Design and harness: `/scratch/code/shibboleth/gui-worktrees/followups/design/` at `42dacaa`.
- R2: `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-design-r2.md`.
- Fold map: `.../design/agent-reports/gui-design-fold3.md`.
- Release CLIs via `sh /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh --no-gui --no-man --root <scratch>`.
- F-687 context: CLI branches (ms `e534917`, toolkit `9846a784`, reviewed
  ready to ship, not released) make `-` = stdin and `@env:VAR` = env on
  `--passphrase`; `-`/`--passphrase-stdin`/`@env:` strip one trailing
  `\n`/`\r\n`; an argv literal is verbatim; `@env:` holding `-` or `@env:X` is
  taken literally (no double resolution).

## Check, with evidence

1. **NC1:** re-run the new `argv_reinterprets` measurement script. Is the row
   right for 0.104.0 and 0.19.1? Build the two F-687 branches and measure what
   the row *would* become. Does updating it as data produce correct behaviour,
   with no second code path? Reproduce `MY_PW='@env:OTHER'` on both paths.
   Try `MY_PW='-'`, `MY_PW='@env:'`, and `MY_PW=' @env:OTHER'`, which exposes
   any trim mismatch between the GUI's check and the CLI's argv guard. A
   value the CLI re-reads that the GUI doesn't refuse is Critical.
2. **NI3:** make the reviewer's mutation and two of your own on the interim
   path (e.g. drop the newline rule there; swap two resolved values) and show
   the new tests go red.
3. **NI4:** re-run the Copy measurements in bash, zsh and fish, and add 3 values
   of your own (e.g. a value ending in a backslash, a value containing `\n`
   mid-string, a leading `-`). Does Copy's recipe ever change the bytes?
4. **T10:** confirm it rejects the three decoys and a job that runs only one of
   the three targets. The "linux: pending" state: is it clearly a hard failure
   in the implementing change, as written into the plan's acceptance criteria?
5. **Pin-bump check:** change `pinned-upstream.toml` to another CLI version and
   show it fails; show it passes when the recorded versions match.

Scratch under `/scratch/code/shibboleth/review-gui-r3-scratch/`; delete with
`find … -delete`. Don't commit, push or edit the branch.

## Output

Per R2 finding: fixed / not fixed. NEW findings Critical / Important / Minor /
Nit with evidence. End with `GREEN (0C/0I)` or `NOT GREEN`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-design-r3.md`**
and return only a short summary plus that path.
