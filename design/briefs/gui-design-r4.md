# Brief — R4 review: mnemonic-gui design fold 4 (the derived-data shape)

**One question:** after fold 4 (`462c648`), is there any behavioural fact the
GUI relies on that can go stale without a gate going red? And does the design
now deliver the user's exact secret bytes to the flag they filled, on every OS
and path, today and after the F-687 pin bump? GREEN (0C/0I) or not. Scope:
fold 4's new shape (§A0) and its R3 fixes only.

## Inputs

- Design and harness: `/scratch/code/shibboleth/gui-worktrees/followups/design/` at `462c648`.
- R3: `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-design-r3.md`.
- Fold map: `.../design/agent-reports/gui-design-fold4.md`.
- Release CLIs via `sh /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh --no-gui --no-man --root <scratch>`.
- F-687 is now on master (ms `d1ab447`, toolkit `9da32e2f`), not released;
  both still report 0.19.1/0.104.0. Build them for the "after" case.

## Check, with evidence

1. **Enumerate every behavioural fact the planner or runner reads** (grep
   the planner and every JSON it loads), and for each: is it derived by
   `regen_check.py` from the pinned binaries, and does some test compare
   against an independent oracle rather than against that fact? Any fact that
   is hand-kept, or checked only against itself, is Important; say which.
2. **Break `regen_check` (T4):** edit a cache cell by hand; swap in a binary
   with a different sha256 but the same version; delete a cache file; add a
   new input to the schema that the cache doesn't cover. Each must go red.
3. **The "after" case:** against ms `d1ab447` and toolkit `9da32e2f`, a
   complete re-derivation is green and a partial one is red. Measure it.
4. **Per-input rules:** the `@env:` value rule and `--flag=value` exactness.
   Spot-check 6 cells against the binary by hand, including `ms derive
   --passphrase` and `ms hashlock --hashlock-phrase`.
5. **`mutations.py`:** the control survives; a crash is not a kill. Add 2
   mutations of your own to the new derivation code; both must be killed.
6. **T10:** try 2 decoys beyond the ten.

Scratch under `/scratch/code/shibboleth/review-gui-r4-scratch/`; delete with
`find … -delete`. Don't commit, push or edit the branch.

## Output

Per R3 finding: fixed / not fixed. NEW findings Critical / Important / Minor /
Nit with evidence. End with `GREEN (0C/0I)` or `NOT GREEN`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-design-r4.md`**
and return only a short summary plus that path.
