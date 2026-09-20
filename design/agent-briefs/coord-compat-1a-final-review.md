# Brief — final whole-branch review, coordinator-compat plan 1a

**Branch:** `coord-compat-1a` in `/scratch/code/shibboleth/dm-worktrees/coord-compat-1a`,
from `b2c5d693`. Five tasks, each already task-reviewed and fix-looped to clean.

**This is the last gate before the branch is shown to the operator.** Per-task
reviews were task-scoped; yours is the only one that sees the whole thing.

## What the branch builds

A `SkeletonKey`: one canonical string computed from any decoded md1 wallet
policy, which a LATER plan will use to look up measured evidence about which
wallet coordinators import that policy. **A key collision means evidence
measured on policy A gets claimed for policy B** — a wallet an operator is told
will import somewhere it will not. That is the risk this branch carries.

Spec: `design/DESIGN_coordinator_compatibility.md` (in mnemonic-engrave).
Plan: `design/IMPLEMENTATION_PLAN_coordinator_compat_1a_skeleton.md`.

## Already established — do not re-derive

- **Collision-freedom is PROVEN**: formal injectivity plus 190,032 enumerated
  partition pairs, 0 collisions, and 20,000 realistic descriptors with 0 keys
  mapping to two membership tuples. Do not redo this; do look for anything the
  later tasks changed that could invalidate it.
- Every task's guards were mutation-proven and each mutation independently
  reproduced by a scoped re-reviewer. Do not re-run those.
- 41 distinct keys over 46 vectors: three groups are duplicate fixtures, one
  pair shares a key by design.

## What to look for, and it is mostly cross-task

Per-task reviews cannot see these:

1. **Seams between tasks.** Task 4 changed `policy_shape`'s branch
   decomposition (adding the taproot key path as branch 0) AFTER Tasks 1-3 were
   reviewed, and edited three of Task 1's tests to match. A scoped re-review
   judged those corrections-not-weakenings with a negative control. Check the
   whole picture holds: does anything else in the branch still assume the old
   branch shape?
2. **The accumulated public surface.** Five tasks each added types and
   functions. Is the result coherent, or five tasks' worth of near-duplicates?
   This plan found one-rule-two-implementations THREE times (lock bands in
   three places; hashlock types; absent-fingerprint sentinels). Look for a
   fourth.
3. **Doc comments that claim more than the code proves.** Two were found and
   corrected during the plan. That is a pattern, not two incidents.
4. **Anything a green suite would hide.** Every real finding in this plan came
   from a mutation, never from reading a diff. 580 tests passing is the
   starting point of your review, not its conclusion.

## Deferred minors — triage which must be fixed BEFORE merge

The ledger carries these unfixed, each deliberately:
- **Task 1:** `plain_multi` has no analogous `nkeys == slots.len()` guard the
  way `sole_multi`'s caller does. Believed safe by construction.
- **Task 2:** the 20→32 byte zero-pad pattern appears in a second place.
- **Task 2:** `validate.rs` hardcodes `1 << 22` at :218, :226, :621 —
  pre-existing, different in purpose (BIP-68 reserved-bit masking, not band
  classification).
- **Task 4/5:** `skeleton()` accepts a descriptor `validate()` rejects; one
  dead `root_kind` arm; the walker pins only the rendered projection
  (`Body::Tr{key_index}` under `is_nums` is ignored by the renderer).

Say for each: must-fix-before-merge, or fine as a follow-up.

## Rulings the controller made — check any that touch code

Eight are in the ledger. Four touch the code directly and you may disagree:
`crate::` over `md_codec::`; a six-valued `RootKind` rather than reusing
`compose::Wrapper`; reusing `compose::HashLock` instead of the plan's
prescribed duplicate; `KeyPathKind` three-valued with `UnspendableXpub`
removed as uncomputable; and implementing the taproot key-path branch now
rather than deferring it. If one is wrong, say so — they were decisions taken
without the operator.

## Rules of evidence

Run what you need in the worktree; leave it clean.
export CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/coord-compat-1a-target
export PATH=/home/bcg/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH

Severity per project standard. Secret-handling defects are never Critical or
Important here.

## Output

**FINAL ACTION: write the report to
`design/agent-reports/coord-compat-1a-final-review.md`** (in mnemonic-engrave)
and return ONLY a one-paragraph summary, that path, your deferred-minor
triage, your verdict on the rulings, and C/I/M/N counts.
