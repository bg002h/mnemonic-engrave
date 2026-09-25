# Brief — R5 mechanical check: mnemonic-gui design fold 5

**This is a short verification pass, not another adversarial round.** Four full
review rounds have run; fold 5 removed the GUI's dependence on CLI behaviour
outside the probe set. The one question: **do fold 5's claims hold as
written, and did it break anything?**

## Inputs

- Design and harness: `/scratch/code/shibboleth/gui-worktrees/followups/design/` at `220c265`.
- R4: `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-design-r4.md`.
- Fold map (the claims to check): `.../design/agent-reports/gui-design-fold5.md`.
- Release CLIs via `sh /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh --no-gui --no-man --root <scratch>`.

## Check, with evidence

1. **The lookalike predicate (NI8):** it is one implementation, used on every
   path and OS. It refuses `" @env:OTHER"`, `"@ENV:X"`, `"- "`, zero-width and
   full-width `@env`, and a lone `-`. It does NOT refuse `"--"`, `"-leading"`,
   `"pass@env:X"`, a normal passphrase, an empty value, or non-ASCII
   passphrases such as `"пароль"` and `"パスワード"`. Over-refusing a real
   passphrase is Important. Run the predicate on those values directly.
2. **The CR/LF refusal (NI9):** a value whose last byte is CR or LF is refused
   on every path. A value with an interior newline is handled as the design
   says. One shared value rule: grep that `run_bytes.py` imports it and no
   second copy exists.
3. **Re-run** `run_plans.py`, `test_plan.py`, `mutations.py` (the control
   survives), `regen_check` and `check_design_tables.py`: all as claimed.
4. **Nm15:** mark a GUI schema flag secret with no table entry; regen goes red.
5. **The loosened check:** `run_plans.py` no longer requires an expected
   refusal code. Confirm a crash still turns it red, and that no plan that
   should run is silently refused (compare the count of admitted plans with
   fold 4's).

Scratch under `/scratch/code/shibboleth/review-gui-r5-scratch/`; delete with
`find … -delete`. Don't commit, push or edit the branch.

## Output

Per claim: holds / doesn't. Any defect as Critical / Important / Minor / Nit.
End with `GREEN (0C/0I)` or `NOT GREEN`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-design-r5.md`**
and return only a short summary plus that path.
