# Brief — review of F-695 (pipefail/`grep -q` sweep)

**One question:** is every change behaviour-preserving except that the
SIGPIPE false-failure is gone, and did the sweep miss or misjudge any
instance? Above all, in the device scripts, can any change make a check that
used to refuse now pass?

## Inputs

- Branches `f695-pipefail`, worktrees `/scratch/code/shibboleth/<prefix>-worktrees/f695`:
  engrave `facb66f8` + `448ef559` (evidence), descriptor-mnemonic `6db19494`,
  mnemonic-secret `9d26302`, mnemonic-toolkit `35fde2ec`, seedhammer `51a9e0d`.
  Diff each against origin's default branch.
- Report: `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f695-impl.md`
  (133 instances, 20 fixed, 113 judged safe).
- Evidence: engrave `design/evidence/f695-pipefail/` (search command, hit lists,
  reproduction harness and output).

## Check, with evidence

1. **Device scripts first** (`scripts/sh2-flash`, `scripts/pico2-bootkey-rehearsal.sh`,
   `scripts/sign-firmware.sh`): for every changed site, read old vs new and
   construct stubbed-producer cases (match present early in a large output;
   match absent; empty; producer failing with non-zero exit; producer printing
   the match and THEN failing). The new code must reach the same branch as the
   old code would have without SIGPIPE, in every case. A change that turns a
   refusal into a pass is Critical. **Never run anything that talks to a device.**
2. **Failing-producer semantics:** capture-then-grep loses the producer's exit
   status unless the fix checks it. Where the old pipeline's `pipefail` failure
   was *meant* to catch a failing producer (not SIGPIPE), confirm the new code
   still catches it.
3. **The 113 "safe" verdicts:** re-derive the list from the committed search
   command and spot-check 15, weighted toward external (non-builtin) producers,
   since the report measured `mk encode --help | grep -q` failing 1/200 at 3.8 KB.
   Any "safe" verdict resting on output size alone with an external producer is
   wrong. List those.
4. **push-via-staging:** the 5 copies are byte-identical; the bypass detector
   now works on a large `$OUT`; re-run the check-runs logic sanity (the
   2026-09-25 fix) still intact.
5. **emit.py → mk smoke test:** regenerating emits mk's fixed text byte-for-byte.
6. Re-run the reproduction harness; shellcheck counts unchanged.

Scratch under `/scratch/code/shibboleth/review-f695-scratch/`; delete with
`find … -delete`. Don't commit, push or edit the branches.

## Output

Critical / Important / Minor / Nit, each with a reproduction. End with
`ready to ship: yes` or `ready to ship: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f695-review.md`**
and return only a short summary plus that path.
