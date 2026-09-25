# Brief — adversarial review of ms 0.19.1 (`f679-stdin`)

**One question:** after this change, can any ms verb read the wrong secret, or
silently drop, duplicate or reorder one, when inputs mix argv (admitted by
`--allow-argv-secret`), a literal `-`, and `--in`? And are two genuine stdin
inputs still refused? Not a fresh audit of ms.

## Inputs

- Worktree `/scratch/code/shibboleth/ms-worktrees/f679-stdin`, branch
  `f679-stdin`, `git diff origin/master..a77c8e4` (fix `5ebb413`, release `a77c8e4`).
- Implementer report (claims to re-run):
  `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f679-ms-stdin-impl.md`.
- The defect as found: `.../design/agent-reports/f679-review.md` (I1).

## Settled — do not re-derive

- Controller-verified: the reproduced `ms verify --allow-argv-secret --phrase <P> <ms1>`
  now exits 0; `printf '%s\n%s\n' <ms1> <P> | ms verify - --phrase -` exits 1.
- `me` does not share the defect (checked separately).
- Secret-handling severity rule (operator 2026-08-27): argv exposure itself is
  never Critical/Important. **Reading the wrong secret, or dropping one, is a
  correctness defect** and gates normally.

## Push hardest on

1. **combine** changed semantics: each admitted share goes to one `-`, and any
   extra `-` reads stdin once. Construct cases with shares in every mix of
   argv / `-` / stdin, in different orders, with k-of-n thresholds, and check
   the recovered secret equals the one split (use `ms split` on the BIP-39
   test vector "abandon ×11 about", which holds no funds). Order-sensitivity,
   a share used twice, or a share silently ignored is Critical.
2. **derive** with `--passphrase-stdin`: prove the passphrase actually reaches
   the derivation in every combination (compare fingerprints against the `--in`
   path, as the tests claim to).
3. **verify**: all three mixes, plus a wrong phrase (must fail) and a wrong card.
4. **A user-typed `-` together with `--allow-argv-secret`** must still count as
   stdin. Can an admitted value ever be mistaken for a `-`, or the reverse?
5. **Mutations:** re-run 2 of the 4 claimed, and add one of your own on combine's
   assignment logic; prove red and that the line ran.
6. Release hygiene: 0.19.1 also ships the merged but untagged F-677/F-670 fixes
   under this CHANGELOG entry. Check the entry describes them accurately.

## How to run

`cargo nextest run --locked --workspace` (never `--release`), pinned clippy
`-D warnings`, `cargo fmt --check`. Scratch or target dirs under
`/scratch/code/shibboleth/review-ms0191-scratch/`; delete with `find … -delete`.
Do not commit, push, tag or edit the branch.

## Output

Critical / Important / Minor / Nit, each with a reproduction. End with
`ready to ship: yes` or `ready to ship: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f679-ms-stdin-review.md`**
and return only a short summary plus that path.
