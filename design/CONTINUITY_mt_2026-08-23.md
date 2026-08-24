# CONTINUITY — the `mt` cycle, 2026-08-23

> Supersedes `CONTINUITY_mt_2026-08-22.md`, which described the pre-R0 state and
> is now wrong in most of its conclusions.

## OVERNIGHT RUN — state as of the last checkpoint

**The plan is GREEN (0C/0I) pending ONE re-gate that has not yet returned.**
`R11-pre-implementation-gate.md` returned **NOT SAFE TO EXECUTE** (3C/6I/10m);
all Criticals and Importants plus 9 Minors are folded (`946e376`, `bd7f191`).
The re-run is `R12-gate-rerun.md`.

**A dispatch of that re-run DIED on a 529 Overloaded** — a server error, not a
result. Flagged rather than absorbed, per the rule that a failed Agent-API
dispatch is never silently replaced by the controller's own review. It was
re-dispatched fresh. **If R12 never returns, NO CODE IS WRITTEN** — the gate has
not closed, and the plan's own status line says implementation may begin only
after it does.

**S0's inputs are prepared, verified, and NOT yet committed** (scratchpad
`s0-READY.md`): two regtest transactions, never broadcast — 222 B (6×37, even)
and 284 B (8 chunks, 32-byte last chunk, uneven). All checksums verify and
vector 1 round-trips to its original bytes. The independent generator was
validated against **40/40** of `mk-codec`'s committed vectors before use, and
the HRP-separation claim was proven empirically (a genuine `mk1` string fails
under `md`/`mt` HRPs against every constant).

**Verified against the mainnet node, which is what R11 C3 turns on:** for a
regtest outpoint, `gettxout` → empty (null) and `getrawtransaction` → error −5.
So §8.5 does not fire, §6a reports UNKNOWN, and `mt encode` behaves the same
with a node and without. A confirmed *mainnet* vector fails this — its inputs
are spent and parents confirmed, which is exactly §8.5's refusal condition. That
was the defect that would have failed P2's gate at 3am.

**`bg002h/mnemonic-transaction` exists, EMPTY and PRIVATE.** v0.1 publishes
nothing, tags nothing, releases nothing, makes nothing public.

## Resume with

    Read design/CONTINUITY_mt_2026-08-23.md, design/SPEC_mt_v0_1.md and
    design/IMPLEMENTATION_PLAN_mt_v0_1.md. R0 on the PLAN was dispatched
    (two opus lenses, reports at design/agent-reports/R8-*.md) — fold
    whatever came back, then take the three operator questions below.

## Where the cycle stands

**SPEC — GREEN, 0C / 0I.** `design/SPEC_mt_v0_1.md`, 3,545 lines.
**PLAN — DRAFT, pre-R0**, `design/IMPLEMENTATION_PLAN_mt_v0_1.md`. R0 in flight.
**NO CODE MAY BE WRITTEN** until the plan closes 0C/0I. Risk-set work.

`origin/master` is at `9de904e` plus whatever landed after — the `ci/staging`
ritual is a single command now, `./scripts/push-master.sh`, which enforces the
freeze itself. Do not dispatch an agent for it.

## What the spec says, in one paragraph

`mt` reads an already-signed transaction and turns it into `mt1` chunked
codex32 strings for **hand** engraving. It does not construct transactions, does
not evaluate scripts, and has no redundancy — an operator wanting to survive a
lost plate cuts a second copy. Four verbs: `encode`, `decode`, `verify`,
`inspect`. **`mt qr` is deferred** to a cross-format cycle shared with `md1` and
`mk1`; its material lives in `design/SPEC_mt_qr_DEFERRED.md` and nothing in v0.1
reads it.

## The three questions waiting on the operator

1. **Creating `mnemonic-transaction` on GitHub** — outward-facing, needs a
   go-ahead, including whether it starts private.
2. **§10.10's flag spellings** must close before P2 ships, since P2 builds the
   CLI. The two *behavioural* flag questions are already ruled (grouping affects
   stdout and the canonical artifact is ungrouped; `--quiet` suppresses the
   inspection report only, never warnings or refusals).
3. Whether anything else should leave the spec — one out-of-scope sweep has run.

## What is already machine-verified — do NOT re-derive

- `./scripts/spec-structure-check.sh` → STRUCTURE OK, 17 sections, 58 cross-refs.
- `./scripts/plan-cite-check.sh` → spec 32/32, plan 5/5, 0 dangling.
- **The NUMS derivation, both constants:** `SHA-256("shibbolethnumstransaction")`
  top 65 bits = `0x1a2fc877f9528d7c1` (spec) and `SHA-256("shibbolethnums")` top
  65 bits = `0x0815c07747a3392e7` (`md-codec`). The *rule* is confirmed, not one
  value.
- `mk-codec` has **no** Cargo dependency on `md-codec` and carries its own
  `bch.rs`, `bch_decode.rs`, `chunk.rs`, `header.rs` under
  `crates/mk-codec/src/string_layer/`.

## The review history, and why it matters

Five correctness rounds closed the spec. Then **three live journey walks with the
operator** found things no correctness lens could — the recurring shape was that
*nothing in the section was wrong*, and a step was simply silent or a section was
written from the wrong chair. Then **R6** (three lenses: fold-propagation,
implementability, adversarial) found **6 Critical + 27 Important** on a spec that
had already been called green, and **R7** verified the fold at 30 FIXED / 3
PARTIAL / 0 NOT FIXED.

The implementability lens had never been run before and its verdict was that the
spec **was not buildable** — five rounds had asked *"is this section right"* and
none had asked *"could someone build this"*. Enumerate lenses up front; a clean
round exhausts the question, not the artifact.

## Things that bit, so they do not bite again

- **A presence check cannot catch a wrong value.** Recompute numbers from the
  artifact and diff; do not grep that the edit landed. This found a false claim
  in one of my own commit messages, plus a third defect the reviewer missed.
- **A table cell is prose that does not read like prose.** Both surviving
  partials after one fold were cells whose surrounding prose had been fixed.
- **After retracting a claim, grep what ARGUES FROM it**, not only what states
  it. Two Importants were arguments built on a superseded number.
- **Range-replace by original line numbers corrupts silently.** Doing it forward
  ate two open questions and an entire section; the diff hid it because the diff
  was huge. Work in reverse order, and assert on line *content*, not the label —
  a bare `7.` lookup found §1's item 7, not §8's.
- **Mutation-test a new gate, and check the mutation LANDED.** My first control
  mutated a string with zero matches, so the gate passed while proving nothing.
