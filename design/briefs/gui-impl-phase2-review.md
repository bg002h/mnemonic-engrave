# Brief — whole-diff review: mnemonic-gui phase 2 (secret channels + five forms)

**One question:** does the implementation do what the GREEN design says, and
can the GUI, as built, deliver a secret to the wrong flag, deliver wrong bytes,
run a secret on argv where the design routes it privately, or show one thing
and run another? This is the project's mandatory post-implementation review
(it catches what TDD misses); the design itself is settled.

## Inputs

- Worktree `/scratch/code/shibboleth/gui-worktrees/followups`, branch
  `gui-followups`, `git diff 220c265..ee26ef1` (9 commits). The branch also
  carries phase 1a (`5956da2` changelog, `e3a55a4` restore `--from` secret
  masking); review those too: `git diff origin/master..ee26ef1` is the whole
  change being merged.
- The design: `design/DESIGN_secret_channels_and_new_forms.md` at `220c265`,
  GREEN after R0–R5 (`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-design-r*.md`).
- Implementation report (claims to re-run): `.../design/agent-reports/gui-impl-phase2.md`.
- CI (controller-verified): build 36174422248 and schema-mirror 36174422278
  succeeded at `ee26ef1`.
- Release CLIs via `sh /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh --no-gui --no-man --root <scratch>`.

## Settled — don't re-derive

- The design's decisions (the lookalike predicate, the CR/LF refusal, Linux-only
  private channels, and derived data with independent oracles) are settled.
- Controller rulings: `ms hashlock --separator` offers `space` only; T9 asserts
  refusal for newline-terminated values.
- Secret-handling severity rule: exposure never gates; wrong flag or wrong
  bytes gates.

## Push hardest on

1. **Rust planner = reference planner.** The implementer says it mirrors
   `plan.py` branch for branch, and T8 compares outputs on the prototype
   shapes. Find a shape or input T8 doesn't cover (the GUI's own flag order,
   a form with a repeated secret field, bundle's slots) and compare Rust vs
   `plan.py` directly.
2. **The runner, by effect:** run the real GUI (kittest harness or the app
   headless) on 5 multi-secret forms against the release binaries and check
   fingerprints and addresses against argv-exact. Swap two field values; the
   output must change accordingly.
3. **Shown vs run:** Preview, Copy and the confirm dialog on 4 secret forms,
   compared with the process's actual argv, stdin and env (intercept them).
   Copy's shell text, pasted into bash, must derive the same wallet.
4. **Part B:** hashlock phrase byte-for-byte (trailing spaces kept);
   `--kind` "(choose)" blocks Run; "all kinds" omits the flag; the xprv
   masking in B2–B4 and not persisted (check the autosave file).
5. **CI gate T10:** hard failure now; confirm it fails if the Linux job is removed.
6. **Mutations:** re-run 4 of the 29, including a delivery one and a Part B one.
7. **Phase 1a:** the `restore --from ms1=` masking at all four sites.

Scratch under `/scratch/code/shibboleth/review-gui-p2-scratch/`; delete with
`find … -delete`. Don't commit, push or edit the branch.

## Output

Critical / Important / Minor / Nit, each with a reproduction. End with
`ready to merge: yes` or `ready to merge: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-impl-phase2-review.md`**
and return only a short summary plus that path.
