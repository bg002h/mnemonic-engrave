# CONTINUITY — F-449 stage 2 (written 2026-09-22)

## RESUME POINT

**Stage 1b is SHIPPED.** dm main `25acb33c` (md-codec 0.46.0, md-cli 0.18.0,
1500/1500, CI green no bypass). engrave master `b5576365`+.

**Stage 2 is at plan review round 2.** No code written yet.

### Do this first
1. Read `design/agent-reports/f449-plan-stage2-r2.md` (the R2 verdict — it was
   in flight when this file was written; it persists itself).
2. If `ready for implementation: no` → fold into
   `design/IMPLEMENTATION_PLAN_f449_stage2_compose.md`, re-gate with
   `./scripts/plan-build-gate-md.sh <plan>`, re-dispatch R3 (opus).
3. If yes → execute with superpowers:subagent-driven-development.

### The plan
`design/IMPLEMENTATION_PLAN_f449_stage2_compose.md` — 9 task headings
(1, 1b, 2, 2c, 3=pointer, 4, 5, 6, 7). Review history: r0 3C/7I → r1 0C/7I →
r2 pending. Reports in `design/agent-reports/f449-plan-stage2-r{0,1}.md`.
Recon: `design/RECON_f449_stage2.md`.

### What stage 2 actually is
**One flag.** `md compose --unspendable liana|nums` (default `nums`) at the
single site `compose/tr.rs:47`'s `None => InternalKey::NumsPoint`. Three of
SPEC §9's four stage-2 items already shipped in 1b — `md descriptor` kind 1,
the `UNSPENDABLE(liana)` substitution rule, and the JSON `md-cli/2` bump.
Everything else in the stage is evidence.

### Hard-won facts — do NOT re-derive
- **The live Liana gate works.** `harnesses/liana` builds against
  `/scratch/code/shibboleth/.tmp/fable-liana-src-v15/liana` (v15.0). md's
  rendered `preset-kofn-recovery-tr` is ACCEPTED, addresses returned.
- **F-640 is RESOLVED.** Liana ACCEPTS a nested taptree (depth 2, two recovery
  paths at older 26280/52560). Descriptors in r0's Appendix A. §8.1 is
  satisfiable — do NOT amend the spec.
- **Liana emits ONE error for at least three causes** (`analysis.rs:596-600`):
  a real policy refusal, an internal key that doesn't match the recipe over
  THESE leaves, and a raw NUMS point. **A control does not catch this** — it
  carries its own correct key. Every probe must recompute its internal key for
  its own leaf set and assert it before sending.
- `vendor-liana-evidence.sh` rebuilds `cases.json` WHOLESALE from a hardcoded
  8-name list. A hand-added vector is erased unless that list learns it.
- v8 IS in `is_supported_version` (`header.rs:36-38`); a corrupted v8 card
  repairs at exit 0. §8.9's row needs a version OUTSIDE the accepted set.
- Toolchain: `export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH`

### Carried follow-ups
F-636, F-638, F-641 (one class: the refusal is right, the message names the
wrong thing) → stage 2 Task 6. F-635 → stage 4a. F-640 → closes in Task 5.

### Remaining stages
3 = Go port (without it the DEVICE cannot read kind-1 plates) · 4 = device
screens · 4a = `me` · 5 = rebuild demo/sh2 + deploy to quantoshi.xyz/SH2/.
**Nothing an operator touches on the device has changed yet.**
