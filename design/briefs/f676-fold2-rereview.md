# Brief — re-review of F-676 fold 2 (mnemonic-toolkit `f675-f677`)

**One question:** did fold 2 fix each finding of the branch review, and did it
introduce a new defect? This is not a fresh audit of the branch.

## Inputs

- The review: `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f675-677-toolkit-review.md`
  (0C/1I/6M/7N). The controller raised **M2 to blocking**.
- The fold: worktree `/scratch/code/shibboleth/tk-worktrees/f675-677`, commit
  `4eceab6b` (`git diff d3f043e7..4eceab6b`).
- The fold report (claims to re-run): `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f676-fold2.md`.

## Settled — do not re-derive

- The review found no way to install an unpublished binary while reporting
  success; that property was checked across ~20 constructed failure cases.
  Only re-check it where the fold touched the path.
- Controller-verified: GUI v0.59.0 x86_64-linux needs GLIBC_2.39 (`objdump -T`).
- The design (pinned release binaries + SHA256SUMS, `--from-source` fallback)
  is settled.

## Check, with evidence

1. **I1:** on a host whose glibc is below a floor, the installer picks a source
   build before downloading, and a refused binary's printed recipe works
   **after a first run that installed the others**. Run that sequence.
2. **M2:** force every temp-dir failure mode you can think of (both `mktemp`
   forms fail; `TMPDIR` missing, unwritable, or a file; `mktemp` printing a
   relative path or nothing). Prove nothing outside a temp root is ever
   removed or created. Run it as a non-root user with `rm`/`mkdir` wrapped to
   log calls.
3. **New code paths the fold added:** the glibc probe (what if `ldd` is
   missing, prints something odd, or it's musl?); the `--force` decision for
   `--from-source` (can it overwrite something it shouldn't, such as a binary
   cargo *is* tracking, or a directory?); the `--root` with a space fix; the
   PATH warning; BusyBox `wget`.
4. **Mutations:** spot-check at least 6 of the fold's 28 claimed mutations,
   including the platform-table and M2 ones: break the line, prove red and
   that it ran, restore.
5. Harnesses under dash and BusyBox sh; shellcheck; the full nextest suite once.

Scratch goes under `/scratch/code/shibboleth/review-fold2-scratch/`; delete it
with `find … -delete` at the end (an `rm -rf` hook blocks the usual way). Do not
commit, push or edit the branch; restore mutations and show `git status` clean.

## Output

For each review finding ID: fixed / not fixed / partially fixed, with evidence.
Then any NEW findings as Critical / Important / Minor / Nit. End with
`ready to ship: yes` or `ready to ship: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f676-fold2-rereview.md`**
and return only a short summary plus that path.
