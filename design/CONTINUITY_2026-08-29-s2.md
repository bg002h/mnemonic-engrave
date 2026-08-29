# CONTINUITY — S2 cycle, started 2026-08-29 morning

## Mandate

Operator, 2026-08-29 (verbatim): **"Do the prep /merge you suggest first and
flash it, but I have to leave. So after flash let's start s2"** — following
the same morning's rulings: fork main merged (F-425 resolved), debris branch
removed, F-420 shipped in descriptor-mnemonic.

## Standing state

- **SH2 flashed 2026-08-29** from fork `main` @ `a5e29b4` via `sh2-flash`
  (load + verify 100%; boot to be judged on MACHINE power — laptop port
  dark-screen+BOOTSEL is the PD-contract check, not a signature reject).
  That flash was operator-authorized. **No further flash is autonomous.**
- S1+S3 shipped (engrave `f244442`); descriptor-mnemonic at `6c4a56fd`
  (F-420); fork `main` = `a5e29b4`.

## S2 cycle plan

`design/IMPLEMENTATION_PLAN_descriptor_input_S2.md` @ `438992f`. R0 loop:
r1 RED 4C/6I/7M/3N (persisted `191bfb7`, folded `7877aa5`); r2 RED
1C/2I/4M/3N (persisted `59915b8`, folded `142258b`); r3 RED 1C/1I/4M/3N
(persisted `60e7868`, folded `438992f` -- key ruling: P3.4/F-426 STAYS
in S2, scan-door widening is seam-safe, the classifier stays host-exact
via a string-level S4.3 five-version check; the regeneration carries the
ypub row's device_admits flip; cite gate 106/106, lint clean). Round 4
in flight (opus, fold-vs-r3's-9-findings; report lands at
`design/agent-reports/R0-S2-plan-r4.md`; a clean round CLOSES at GREEN).
Recon: `design/agent-reports/RECON-S2-fork-seam.md` (`4646fa2`).
Scale: 5 expected-good / fable at 15 / hard stop 25. Persist-before-fold; agent-persisted reports; whole-repo
propagation sweeps + generators; both clippy toolchains (F-430).

## Boundaries

No tags, no releases, no publishes. **Operator-gated tail (P5.4): every
flash, §11 item 6 (ClassDescriptor DISPLAYED on the device), F-423's
physical test plate + cut.** F-422 status quo stands — no transform.
Fork `main` push only after the post-impl review closes green (the device
boots main).

## Resume protocol

After a context clear: `/resume-s2` (command saved at
`~/.claude/commands/resume-s2.md`). It reads this file, the PLAN and the
RECON report FULLY into context, carries its own digest of the recon
file:line map and the plan's edge cases (so nothing rests on one file),
names the first action (persist any uncommitted R0 report), and restates
the boundaries.
