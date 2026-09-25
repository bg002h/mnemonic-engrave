# Brief — re-review of F-676 fold 3 (mnemonic-toolkit `f675-f677`)

**One question:** is NEW-1 fixed, so that the installer can never overwrite a
binary another cargo package owns, and did fold 3 introduce a new defect?
Not a fresh audit.

## Inputs

- Findings: `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f676-fold2-rereview.md`
  (NEW-1 Critical + a test-14 Minor).
- The fold: worktree `/scratch/code/shibboleth/tk-worktrees/f675-677`,
  `git diff 4eceab6b..21669d61`.
- Fold report (claims to re-run): `.../design/agent-reports/f676-fold3.md`.

## Settled — do not re-derive

- I1, M2, M1, M3–M6 were confirmed fixed by the previous re-review.
- The design and the "no unpublished binary reported as success" property
  stand; only re-check them where fold 3 touched the path.

## Check, with evidence

1. **NEW-1 fixed, adversarially.** `crates_owner()` parses cargo's records with
   awk. Attack the parser with real cargo-written roots: the `fake-mk` repro; a
   package whose bin list is a multi-line array; a package id containing
   spaces, quotes or `(git+…)`; two packages both listing the bin; a bin name
   that's a prefix or suffix of another (`mk` vs `mk-cli`, `md` vs `mdx`);
   `.crates.toml` present but `.crates2.json` absent, and the reverse; either
   file malformed or empty. For every case, the installer must either refuse or
   leave the file untouched unless no record claims it. Test under gawk,
   mawk if available, and BusyBox awk.
2. **The printed ways out work** (`cargo uninstall --root …`, re-run with
   `--force`), and a user `--force` still overrides.
3. **The `--root "$ROOT"` behaviour change:** what happens to a user who sets
   `install.root` in cargo config, or `CARGO_INSTALL_ROOT`, without `--root`?
   Is anything installed somewhere the user wouldn't expect, and does the
   manual or `--help` say so? Is it a regression against the pre-branch
   installer?
4. **Test 14 Minor fixed:** remove both temp-root guards and show the suite goes red.
5. Spot-check 4 of the fold's 13 mutations; run the install harnesses under dash
   and BusyBox sh; shellcheck; nextest once.

Scratch goes under `/scratch/code/shibboleth/review-fold3-scratch/`; delete it
with `find … -delete` (an `rm -rf` hook blocks the usual way). Do not commit,
push or edit the branch; restore mutations and show `git status` clean.

## Output

NEW-1 and the Minor: fixed / not fixed, with evidence. Then any NEW findings as
Critical / Important / Minor / Nit. End with `ready to ship: yes` or
`ready to ship: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f676-fold3-rereview.md`**
and return only a short summary plus that path.
