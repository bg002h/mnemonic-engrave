# Brief — adversarial review of the F-677 md and me branches

**One question:** do these two branches do what they claim without weakening
any check that protects funds — and can each new test actually fail?
Do not audit the rest of either repo.

## What to review

- descriptor-mnemonic, worktree `/scratch/code/shibboleth/dm-worktrees/f677`,
  branch `f677-shapekey`: `git diff 7b03a408..1bc2331a` (fix `5aa374f5`,
  release `1bc2331a` md-codec 0.48.3 / md-cli 0.20.3).
- mnemonic-engrave (me), worktree `/scratch/code/shibboleth/me-worktrees/f677`,
  branch `f677-me`: `git diff 1ed5dc08..f4fbd7b1` (fix `d799e141`, release
  `f4fbd7b1` me 0.12.0).
- The implementer's report:
  `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f677-md-me-impl.md`.
  Treat its claims as claims: re-run what it says it ran.
- The F-677 entry in `mnemonic-engrave/design/FOLLOWUPS.md` (search `### F-677`).

## Settled — do not re-derive

- The ms and mk F-677 branches were reviewed separately and are ready to ship
  (`design/agent-reports/f677-ms-mk-review.md`). Out of scope.
- Parent fingerprints are not on the md1 wire (F-611), so a re-render always
  carries `00000000`. Don't argue that point; test whether the fix exploits it
  safely.

## Where to push hardest

1. **md `without_parent_fingerprints`:** the self-check is a funds guard. It
   exists so shape-key never keys a card that reconstructs a *different* wallet.
   Construct an input that the new normalization accepts but that differs from
   the reconstruction in anything that matters: key material, chain code, depth,
   child number, origin path, network version bytes, an origin-less key, a
   checksum. If you find one, it is Critical.
2. **me digest-shaped phrase refusal:** it must match `ms hashlock`'s rule
   exactly (the widths, the override flag's name and semantics). Compare against
   mnemonic-secret's source at `/scratch/code/shibboleth/mnemonic-secret`. Any
   phrase `ms hashlock` stops that `me sysw pack` passes, or the reverse, is a
   finding.
3. **Mutation:** for each new test, break the guarded line, prove the test goes
   red *and* that the mutated line ran, then restore. Report each one.
4. Release hygiene: version bumps, CHANGELOG entries that match the behaviour,
   `Cargo.lock` consistent. descriptor-mnemonic is a toolchain-pinned repo: run
   its pinned clippy, not the system one.

## How to run

`cargo nextest run --locked --workspace` (never `--release`), clippy with
`-D warnings`, `cargo fmt --check`, in each worktree. Use your own
`CARGO_TARGET_DIR` under `/scratch/code/shibboleth/review-target-f677/` (not
`/tmp`, which is a small tmpfs), and delete it at the end. Do not commit, push
or edit the branches; if you need to mutate, restore and show `git status` clean.

## Output

Findings as Critical / Important / Minor / Nit, each with a reproduction.
End with the line `ready to ship: yes` or `ready to ship: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f677-md-me-review.md`**
and return only a short summary plus that path.
