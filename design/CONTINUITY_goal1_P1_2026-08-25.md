# CONTINUITY — Goal 1, phase P1 (`me`'s transaction container)

*Supersedes `CONTINUITY_goal1_2026-08-24.md` as the resume point for **P1**. That
file remains authoritative for the SPEC cycle, the operator journey walk, the two
rulings, and S0's schedule — none of which changed.*

## State in one line

**The SPEC is R0 GREEN. The P1 PLAN is at v8 and NOT green** — round 5 returned
2C/5I/5M and is folded (`01ebb1e`); **round 6 (comprehension lens) is running**.
No code has been written and none may be until the plan reaches 0C/0I.

## The plan's round history — measured, not remembered

Every count below is from the persist commit that carried the verbatim report.

| round | lens | on | verdict | fold |
| --- | --- | --- | --- | --- |
| 0 | adversarial correctness (opus) | v1 | **5C / 13I / 5M** | v2 — a **rewrite**, not a fold |
| 1 | fold-check (opus architect) | v2 | **3C / 11I / 8M** — *all three C from the rewrite* | v3 (`65ad79a`) |
| 2 | fold-check + implementability (opus) | v3 | **2C / 7I / 4M**; 20/22 landed | v4 (`186a2f8`) |
| 3 | fold-check + fresh audit (opus) | v4 | **2C / 7I / 3M**; 12/13 landed | v5 (`c517286`) partial, v6 (`0b3333f`) |
| 4 | falsification (opus) | v6 | **2C / 5I / 1M**; 5 FIXED, 2 PARTIAL, **1 WRONGLY FIXED** | v7 (`0409815`) |
| 5 | falsification (opus) | v7 | **2C / 5I / 5M**; 4 FIXED, 2 PARTIAL, **2 WRONGLY FIXED** | v8 (`01ebb1e`) |
| 6 | **comprehension** — does it TRANSFER? | v8 | *running* | — |

**The curve is flat at 2C and that is not stalling.** Each round's Criticals were
*different defects*, none a survivor of the last. What changed is where they live.

## THE FINDING OF THIS CYCLE — the defects are in the FOLD, not the artifact

**Rounds 3, 4 and 5 all found their Criticals in the PREVIOUS FOLD, not in the
original design.** Traced:

| round | Critical | introduced by |
| --- | --- | --- |
| 4-C1 | the error path never reached §4 or §6 | **v5** (mine) — added §2.5a, never retracted §2.4 |
| 4-C2 | step 4's gate could not go green | **v4** (the architect's) — ruling changed V3, steps carried across |
| 5-C1 | the vector home would turn a shipped test RED | **v7** (mine) |
| 5-C2 | W8 named a type from a later step | **v7** (mine) |

**Both authors introduced Criticals, one each in the arm that was measured.** The
staffing question — *should an independent agent do folds?* — came out a **tie**,
and it is the wrong question. The failure mode is the same for both:
**incomplete propagation**, a diff falsifying text it never touches.

**And the sharpest single lesson, from 5-C1:** v7 answered a finding by citing the
two assertions the *report* named **without opening the file**. The file was a
golden of the code under test — §3.2's exact negation — and the ruling would have
turned a shipped non-`#[ignore]` test RED at step 4 and every step after.
**A reviewer's prescribed remedy is not authoritative; the defect is.**

## THE GATES — four now, and they out-perform reading by a wide margin

Run **all four before committing any fold**. A fold is authorship and re-earns them.

| gate | what it reads | PASS on v8 |
| --- | --- | --- |
| `plan-cite-check.sh` | every `path:line` against the real tree | 86/103 resolve; 17 dangling = 9 `mnemonic-transaction` + 8 vendored `bitcoin`; **0 elsewhere** |
| `plan-table-check.sh` | table rows vs header width | 138 rows, 0 malformed |
| `plan-wiring-check.sh` | **NEW** — referential integrity of §2.4's sites and §3's vectors | exit 0 |
| `plan-fold-sweep.sh` | literal tokens a fold retracted | 35 terms, 35 hits, all inside the fenced block |

**`plan-wiring-check.sh` (`13c73ad`) was built because r4-C1 and r4-C2 shared no
token with the text they falsified**, so the lexical sweep structurally could not
see either. Mutation-tested: on **v6** it returns exit 1 naming *"W11/W12/W13 are
LIVE wiring sites that NO step builds"* — r4-C1's core, in milliseconds; on **v4**
it correctly returns exit 0. **Its blind spot is stated in its own header: it
would have caught NEITHER ordering Critical (r4-C2, r5-C2).**

**Tally that settles the effort question.** Across three folds the citation gate
caught **nine** bare-path defects, the sweep caught three retraction survivors, and
the wiring gate caught one. **No reading — mine, an architect's, or four review
rounds — ever caught the citation class.** A round costs ~200k tokens and ~15
minutes; the gates cost seconds.

## THE RECURSION — four times, and worth expecting

**Documenting a retraction has re-created it four times this cycle**: quoting the
three bad citations in §6.1's PASS cell minted them again; listing the swept terms
in a table cell made all ten self-hit; §3.3's prose re-added three bare paths; and
the r5-M4 tally table quoted the stale phrases it retracts. **Every one was found
by re-running, never by re-reading.** Describe a retracted phrase; do not quote it.

## What is OPEN

- **P1 plan is NOT green** — round 6 running. Fold it, re-run four gates, re-dispatch
  until 0C/0I. **No code before then.**
- **`cargo publish mt-codec 0.1.0`** — irreversible, gated behind the plan going green.
  §2.2 and §5 own it; the manifest pin is `mt-codec = "=0.1.0"` (exact, not caret).
- **S0's test plate** — the operator's, ~a week out from 2026-08-24. Loud; time-of-day
  constrained. Its prerequisite (the Structured Append fixture) is CLOSED.
- **F-245** — `md1`/`mk1` padding rides into the public section verbatim. Important,
  not P1's.
- **F-246** — `me sysw pack` prints a passphrase before validating records. Minor,
  post-P1 UX.
- **Two spec corrections** still owed by P1 (the 16,367 headline, the 5/2 row), plus
  **§6.3's five stale prose statements** — all now carried by a §6 closure bullet.

## Lenses not yet run on the PLAN

**failure-states** (for each thing that goes wrong, what does the operator SEE?) and
whatever round 6 does not exhaust. **Closure is LENS-closure**: stop when the
questions run out, not when a round comes back clean.

## How to restart

1. `git log --oneline 5fba015..HEAD` — the whole cycle, newest first.
2. Read this file, then `design/IMPLEMENTATION_PLAN_P1_me_container.md`'s status
   header (it states the current version and what the last fold changed).
3. `design/agent-reports/R0-P1-plan-round<N>.md` — every round verbatim.
4. Run the four gates before touching anything, so you know the baseline.
5. `git diff <persist>..<fold>` for any round — that is what changed *in response
   to* what, and it is the diff the two-commit rule exists to produce.
