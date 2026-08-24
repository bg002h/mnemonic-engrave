# CONTINUITY — the `mt` cycle, 2026-08-23

> Supersedes `CONTINUITY_mt_2026-08-22.md`, which described the pre-R0 state and
> is now wrong in most of its conclusions.

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
