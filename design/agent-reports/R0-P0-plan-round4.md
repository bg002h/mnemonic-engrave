# R0 ROUND 4 — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md`

**Artifact:** `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` at `aa89bb5`
(worktree `review/p0-r4`).
**Object:** two questions only — (1) did the fold close round 3's 2C/3I/7M/2N,
and (2) is the plan executable end to end by one implementer.
**Date:** 2026-08-26.

## VERDICT — **NOT GREEN**

| severity | count |
| --- | --- |
| **Critical** | **2** |
| **Important** | **4** |
| Minor | 10 |
| Nit | 2 |

**The single most important finding: the remedy was RELOCATED, not applied.**
The fold's rule — *"PROSE IN THIS PLAN NEVER NAMES A STEP NUMBER"* — is false in
the plan that states it. **Nine numbered step references survive**, and at least
six are stale against the current table. They survive because the verification
grep the fold ran (`grep 'step [0-9]'` → 0) is a **false negative on four
mechanisms**: case (`STEP 6`), plurality (`steps 1 and 7`), hyphenation
(`step-3`), and line-wrap (`step\n1's`). The repo's own committed
`scripts/fold-propagation-check.sh`, run with those four patterns, exits **1**.

Second: the answer to the brief's other question is that **a name can go stale
exactly as a number can, and one already has.** The prose now correctly says
*"**The move** therefore carries a **pty assertion** pinning the refusal … and
without that assertion the move proves nothing about the terminal path."* The
reference resolves to step 2. **Step 2 has no pty assertion.** The referent was
fixed and the referent's content was not — the same defect one layer up.

---

## QUESTION 1 — DISPOSITION OF ROUND 3's FINDINGS

| # | disposition | evidence |
| --- | --- | --- |
| **C-1** §4's rationale on the old nine-step numbering, 13 sites | **PARTIAL** | 10 of 13 sites converted to by-name. **3 not converted** (204→208 *"STEP 6's AUTHORITY"*; 539/543 *"steps 1 and 7"* / *"Steps 2,3,4,5,5b,5c and 6"*; 450 *"step\n1's own check"*), and **4 more numbered sites the round-3 list never enumerated survive** (234, 517, 530, 546, 686). See **C-1** below |
| **C-2** *"the 11"* / split `read_records` outside §4's table, 5 sites | **PARTIAL** | 2 of 5 fixed — §3's file map line 420 now reads **`read_records` WHOLE**, §2.4's *"where the seam actually pays"* is retracted. **3 not fixed:** line 96 *"Nothing crosses a crate boundary until **those 11** are a library"*, line 474 *"every type and constant **the 11** reference"*, line 614 *"proves the closure is really **11** and not more"*. Round 3's remedy named all three explicitly. Rolled into **M-11** — the two sites that decide the *split* were the load-bearing ones and both are closed, so this no longer gates |
| **I-1** step 2 (the move) has no test gate; retracted caller arithmetic; `Destination`/`WriteBlock`/`use super::` unnamed | **NOT CLOSED** | `git show aa89bb5` does not touch line 506 (step 2's row) or the *"FOUR CALLERS LIVE OUTSIDE THE CLOSURE"* paragraph except `Step 1`→`The move`. The commit message does not mention I-1. See **I-1** below |
| **I-2** condition 10 discharges nothing; §4 does not obey its digit clause | **PARTIAL** | First half closed — condition 10 now reads *"fixed at ALL FIVE SITES"*. **Second half untouched:** steps 5, 6, 7 and 8 still assert refusals with no digit, and `EXIT_INVALID` — the code `--expect` returns — appears **0 times** in the plan (`grep -c EXIT_INVALID` → 0). See **I-4** |
| **I-3** condition 9 authorises a remedy that leaves the remedy gate permanently RED | **CLOSED** | Condition 9 now: *"**The remedy must make the recipe WORK** — flush, edit, reload (`fc -W`, `sed -i`, `fc -R`). Merely rewording … **cannot make the gate green** … If the recipe genuinely cannot be made to work, that is a finding to raise."* The stall is gone. Residue: a dangling *"either"* — **M-8** |
| **M-1** three vs four exit constants | **NOT CLOSED** | Line 548 still *"The three exit constants"*; measured: `crates/me-cli/src/main.rs` `295` `EXIT_OK`, `296` `EXIT_USAGE`, `297` `EXIT_REFUSED`, **`298` `EXIT_INVALID`** — four |
| **M-2** *"used four times in its source"* | **NOT CLOSED** | Line 211 unchanged. Measured: `EXIT_REFUSED` at `main.rs:407`, `:599`, `:2026` — **three**, plus the declaration at `:297` |
| **M-3** `--out` overwrites gated on the `channel.rs` step | **NOT CLOSED** | Line 513 unchanged; line 413 still gives `channel.rs` only `destination`; line 422 keeps `write_private` in `me` |
| **M-4** no step creates `mnemonic-io-lib` | **NOT CLOSED** | Step 2 → *"`me`'s lib half"* / `<lib module>`; steps 3–9 → crate files; step 10 → *"`me` consumes the crate"*. Unchanged |
| **M-5** step 10's *"all 388 pass"* is stale by construction | **NOT CLOSED** | Line 514 unchanged, after steps 4, 6, 7 and 8 each add tests |
| **M-6** condition 4 does not match step 8 | **NOT CLOSED** | Condition 4 (line 657) still *"`--expect descriptor,transaction` … and refuses an incomplete `md1` set"* — no `mt1`, no `descriptor,cosigner`, no `--allow-unsigned-inputs` clause |
| **M-7** step 6's observable unnamed; `--allow-argv-secret` off clap regresses the CLI | **NOT CLOSED** | Line 510 unchanged |
| **N-1** ``grep -c 'EXIT_' `` with no file; `<lib module>` placeholder | **NOT CLOSED** | Lines 505, 506 unchanged |
| **N-2** step 5 points at condition 5; the authorising condition is 9 | **NOT CLOSED** | Line 509 still *"**Blocked on F-264** — see §6 condition 5"* |

**Score: 0 of 2 Critical closed (both partial), 1 of 3 Important closed, 1
partial, 1 untouched, 0 of 7 Minor closed, 0 of 2 Nit closed.**

The fold answered the four findings its commit message names (C-1, C-2,
condition 9, condition 10) and touched **nothing else**. Nine of round 3's
twelve non-Critical findings were not acted on and are not mentioned.

---

## QUESTION 2 — IS THE PLAN EXECUTABLE END TO END?

### Can every gate fail? — §4's eleven steps

| # | can it fail? | note |
| --- | --- | --- |
| 1 signature change | **yes** | one `EXIT_*` in `no_records_guard` today → 0. Real. Gate is prose-shaped (N-1) |
| 2 **the move** | **NO, in the sense that matters** | *"builds"* + `grep -c 'pub const EXIT' main.rs == 0` — **already true on the untouched tree**, all four consts are bare `const` — + a placeholder grep. **No test is required to run.** → **I-1** |
| 3 mask split | **yes** | RED today: `main.rs:912` masks `& 0o044`, `0o620 & 0o044 == 0` |
| 4 `observation.rs` + pty | **yes** | probe mutation-checked both directions. The strongest gate in the plan |
| 5 `remedy.rs` | **yes**, and now satisfiable | condition 9's fix removes the stall |
| 6 layer 1 | **underspecified** | the observable is still unnamed (M-7) |
| 7 layer 2 | **half — and the prose now de-gates it** | crate unit tests are RED; the `--nosuchflag` clause is true today. And line 543 tells the implementer step 7 is **regression-gated** → **C-1** |
| 8 `--expect` | **yes**, but pins no digit | rc=4 named nowhere → **I-4** |
| 9 `exit.rs` + `channel.rs` | **yes** | differential is genuine, but scoped to *"codes `me` produces today"*, which cannot reach step 8's new rc=4 |
| 10 consume | **yes** as a regression gate | count stale (M-5) |
| 11 publish | n/a | operator-gated |

### Does every §6 closure condition have a step that builds it?

| condition | step | |
| --- | --- | --- |
| 1 tests pass | 10 | ✓ (count stale) |
| 2 §5b's 16 verb checks | **none** | a measurement dated 2026-08-26, re-run by nothing → M-9 |
| 3 §6f `mnemonic` cell re-measured under `inspect` | **none** | an unassigned task → M-9 |
| 4 `--expect` refusals | 8 | ✓ mapping; ✗ content (M-6) |
| 5 §6h history question + positive test | 5 | ✓ |
| 6 F-259 **and F-260** caught by a test | 4 for F-259; **none for F-260 — and §7 forbids it** | → **I-3** |
| 7 §8 `CLOSED`-grep discipline | n/a | process |
| 8 pre-parser guard + override | 6 | ✓ |
| 9 F-264 | 5 | ✓ |
| 10 F-265 at **all five sites** | **none for four of the five** | → **I-2** |
| 11 R0 0C/0I | n/a | |

### Do the by-name references resolve to exactly one step each?

**Mostly yes — this part of the mechanism works.** *"the move"* → 2,
*"the mask split"* → 3, *"the remedy work"* → 5, *"the value-shape layer"* → 7
each resolve uniquely and unambiguously. Two do not:

- ***"the signature change"*** (line 601, named in the rule itself) — the plan
  contains **three**: `no_records_guard` (line 505, called *"the ONE signature
  change"*), `refuse_write_block` returning a DECISION (line 382, *"that
  signature change belongs to P0"*), and `emit`/`write_block` (line 695). Only
  the first has a step.
- ***"the adoption gate"*** (line 454) — **no table row is named "adoption."**
  Step 10 is *"`me` consumes the crate."* Meanwhile §7 uses "adoption" twice for
  *later phases* (*"`mnemonic-toolkit`'s own adoption … not P0's work"*), so the
  sentence reads as pointing at a P1 gate, which would drop the requirement out
  of P0 entirely.

### Is anything asserted in prose that the table contradicts?

**Yes — three sites, and they are C-1, C-2 and I-2 below.** The failure this
fold exists to end is not ended; it has changed vocabulary.

---

## CRITICAL

### C-1. The "prose never names a step number" rule is false in the plan that states it — 9 sites survive, ≥6 stale — and the grep that certified it is a false negative on four mechanisms.

**Site:** lines 208, 234, 450–451, 517, 530, 539, 543, 546, 686. The rule is
stated at line 596.

**What is wrong.** Reproduced verbatim, absolute paths, no pipe:

```
grep -c 'step [0-9]' <plan>                    → 0        ← the fold's gate
grep -n 'STEP [0-9]' <plan>                    → 208 234 517 530 546
grep -ni 'steps [0-9]' <plan>                  → 539 543
grep -ni 'step-[0-9]' <plan>                   → 686
grep -n -A1 'step$' <plan>                     → 450/451  ("step\n1's own check")
scripts/fold-propagation-check.sh <plan> 'STEP [0-9]' 'steps? [0-9]' \
    'step-[0-9]' '5b|5c'                       → exit 1
```

The gate is case-sensitive, matches a single space, and is per-line. Every one
of the nine survivors escapes through one of those three properties. **The
repo's own propagation script says so in its header** — *"It checks the
phrasings you thought of, so a claim you did not think to search for survives
silently"* — and it was not the script that was run.

**Staleness, site by site, against the current table** (`1` signature change ·
`2` the move · `3` mask split · `4` `observation.rs`+pty · `5` `remedy.rs` ·
`6` layer 1 · `7` layer 2 · `8` `--expect` · `9` `exit.rs`+`channel.rs` ·
`10` consume · `11` publish):

| line | text | true step |
| --- | --- | --- |
| 208 | *"§6f IS NOT SUFFICIENT AS **STEP 6's** AUTHORITY"* — the paragraph is entirely about exit codes and ends *"the gate is now differential against the current binary"* | **9**. Step 6 is the pre-parser flag guard. Round 3 named this site; the fold left it |
| 234 | *"**STEP 4's** ORIGINAL GATE WAS FALSE AGAINST THE CORRECT TEXT"* — the `history -d` gate | **5**. The paragraph carries a live requirement (*"never OFFERED"* ≠ *"never mentioned"*) that belongs to `remedy.rs` |
| 450–451 | *"**step 1's** own check would have passed it"* — a check that *"asked only whether a type was moved or public"* | **2 / 10**. Step 1's check is the 388 tests and one grep; it has nothing to do with type visibility |
| 517 | *"**STEP 1's** GATE IS BLIND TO 5 OF 8 EXIT DECISIONS"* | **1** — accurate, by coincidence of the renumbering |
| 530 | *"**STEP 1's** GATE CANNOT FAIL FOR THE TERMINAL ARM"*, body: *"green whether or not the terminal refusal survives **the move**"* | heading **1**, requirement **2**. The fold converted the body and left the heading |
| 539/543 | *"**steps 1 and 7** do NOT have a test that must fail first … **Steps 2, 3, 4, 5, 5b, 5c and 6** are RED-first; **1 and 7 are regression-gated**"* | **stale three ways** — see below |
| 546 | *"**STEP 1** AS WRITTEN CANNOT BE DONE"*, body converted to *"the move as first written"* | The heading now asserts the table's **step 1 — the signature change — cannot be done**, when the signature change is precisely what makes the move possible. The fold rewrote this paragraph's body and left its heading inverting the table |
| 686 | *"the only shape the **old step-3** gate tested"* | historical, but today's step 3 is the mask split |

**The concrete failure is line 543, and it is not cosmetic.**

1. **It de-gates step 7 — the value-shape argv guard, the funds-path layer.**
   The table gives step 7 a RED-first gate (*"the argv gate refuses by class,
   with the override, **as unit tests**"*). The prose says step 7 is
   *"regression-gated"* and that *"the column header should not claim"*
   otherwise — an explicit instruction to disregard the table's column for that
   step. Step 7's only other clause (*"`--nosuchflag <ms1…>` still does not echo
   the secret"*) is **true today** by §3's own measurement. An implementer who
   follows the prose closes step 7 having written **no test at all**, and the
   value-shape recogniser for `tx:` / `mt1` / `ms1` / BIP-39-on-argv ships
   unexercised.
2. **It classifies the move as RED-first**, in the same sentence where line 544
   says *"the move's pty assertion is the one RED-first thing in it"* and I-1
   says the move's gate is a build. Three readings of one step, within six lines.
3. **`5b` and `5c` do not exist.** Round 3's remedy said *"delete `5b`/`5c`"* in
   as many words. They are still there.
4. Steps 8, 9, 10 and 11 are unclassified by a paragraph that claims to
   partition the table.

**What closes it.** Convert the nine sites to by-name, as the other ten were —
the mechanism is right, it was applied to 10 of 19 sites. Rewrite line 539–543
against the current table or delete it: it is the only site that *contradicts*
rather than merely mis-points. Then replace the ad-hoc grep with
`scripts/fold-propagation-check.sh <plan> 'STEP [0-9]' 'steps? [0-9]'
'step-[0-9]'` plus a wrap-tolerant check (`tr '\n' ' '` before the grep), and
put that command in the plan so the next fold cannot certify itself with a
narrower one.

---

### C-2. The pty pin the prose mandates for the move is in no step, and the plan's only pty assertion asserts the OPPOSITE behaviour two steps later.

**Site:** §4 lines 530–537 (the requirement); §4 line 506 (step 2's gate, which
does not contain it); §4 line 508 (step 4's gate, which is not it).

**What is wrong.** The prose states the requirement correctly and by name now:

> All 12 tests in `crates/me-cli/tests/world_readable_output.rs` redirect to
> files, so none of them reaches it. **The move** therefore carries a **pty
> assertion** pinning the refusal … and without that assertion the move proves
> nothing about the terminal path.

Step 2's gate, in full: *"builds with `grep -c 'pub const EXIT' main.rs == 0`
and `grep -c 'EXIT_' <lib module> == 0` — a published constant fails the step.
Callers enumerated in BOTH directions."* **No pty assertion. No test at all.**

The plan's only pty assertion is step 4's, and **it cannot serve as the pin**:
it asserts `me sysw wipe --fill zeros` **must NOT** emit the word BEARER — the
*post-F-259-fix* behaviour, which is RED today by construction. An assertion
that is RED before the change cannot pin behaviour across a move that precedes
it. Round 3 said this in the same words and the fold did not act.

**The concrete failure.** Step 2 physically relocates six definitions,
including `write_block` and `destination`, which produce the terminal decision.
Its gate is `cargo build` plus two greps, one of which is green on the untouched
tree. Under §4's *"No step begins until the previous is green,"* the implementer
advances to step 3 with the terminal refusal unexercised by anything: the 12
`world_readable_output.rs` tests all redirect to files, step 1's 388 are green
either way (that is I1's whole point), and step 4's assertion is two steps
later and points the other way. If the terminal arm is dropped or inverted in
the move, `me` writes bearer material to a terminal and **every gate in the plan
is green.** That is F-259's own funds-adjacent path, in the phase whose artifact
gets cut into metal.

**What closes it.** Put the assertion in step 2's gate cell, in the form that
pins **pre-move** behaviour — `script -qec` on a bearer payload to a terminal
still refuses, **pinning the digit** (F-265) — and add the regression clause
round 3 asked for: *"the 388 pre-existing tests still pass."* A one-cell edit.

---

## IMPORTANT

### I-1. Round 3's I-1 was not acted on in any of its three parts.

**Site:** §4 line 506 (step 2's whole gate); §4 lines 525–528.

1. **No test is required to run at the move.** Covered in C-2 above.
2. **The retracted arithmetic is still there.** Line 525: *"`write_private` has
   **three** and `refuse_world_readable_stdout` **one**, producing four
   `E0425`s."* Round 3 measured that both functions **stay in `me`** under
   Variant B, so none of those callers crosses anything — the figure was taken
   against the rejected intact move, and the plan's own line 526 says so
   (*"producing four `E0425`s"*). It now sends the implementer looking for four
   errors that will not occur, and away from the two that will.
3. **The two things that actually break are named nowhere.** `destination`
   returns `Destination` and `write_block` returns `WriteBlock`, both private
   enums in `main.rs` consumed by `refuse_write_block`, which stays; and
   `main.rs:2165` reads
   `use super::{destination, is_plate_artifact, write_block, Destination, WriteBlock};`
   inside `#[cfg(test)] mod tests`.

**The concrete failure, unchanged from round 3.** At the move the implementer
hits a broken `use super::`. With a build-only gate and no enumerate-and-justify
clause, the cheapest green is to delete or `#[ignore]` the two tests — one of
which is `write_block_decides_both_gates_once` (`main.rs:2201`), the test §6
condition 6 names as *"expected to change"* and the only unit pin on the
terminal decision. Step 10's *"all 388 pass"* would not notice, because that
count is stale by then (M-5).

**What closes it.** Round 3's remedy, unmodified.

### I-2. No §4 step edits the functions that stay in `me` — and three P0 mandates require exactly that. Condition 10 now contradicts itself in adjacent sentences.

**Site:** §6 condition 10 (lines 721–725); §3 line 382; §6 condition 6 line 695;
§4's eleven rows.

**What is wrong.** `grep -n 'refuse_write_block\|read_records\|`emit`'` over §4
returns **one row** — step 2, line 506 — and it names them only as **STAYING**:
*"`read_records`, `emit`, `write_private` and every `refuse_*` STAY."* No step
schedules an edit to any of them. Three P0 mandates require one:

- **Condition 10:** *"F-265 fixed at **ALL FIVE SITES**."* The five are
  `refuse_write_block` ×2, `read_records` ×2, `emit` — round 3 measured them at
  `FOLLOWUPS.md:11754-11760`. **All five stay in `me`.** Step 4's pty digit
  reaches at most the Terminal arm of `refuse_write_block`. **Four sites have no
  step.**
- **§3 line 382:** *"So `refuse_write_block` returns the DECISION, not
  `Some(i32)` — **that signature change belongs to P0**, and it is what makes
  this boundary real rather than asserted."* No step.
- **Condition 6 line 695:** *"`emit` and `write_block` change signature."* No
  step. (`write_block` is in the crate by then; `emit` is not — so this is a
  cross-boundary change scheduled nowhere.)

**And condition 10 contradicts itself.** Verbatim, lines 721–723:

> **F-265 fixed at ALL FIVE SITES — "for the moving set" would be vacuous,
> since every one of them **stays in `me`**** (round-3). … **P0 moves these
> functions**, and a refactor over an untested distinction is how the
> distinction dies.

*Stays in `me`* and *P0 moves these functions*, two sentences apart, about the
same five. The fold widened the scope and carried the old justification across
unedited — the identical mechanism as C-1, inside the condition it was fixing.

**The concrete failure.** An implementer walks §4 steps 1→11, closes them all,
and has fixed **one** of condition 10's five sites and neither signature change.
P0 cannot close. Worse, condition 10's own warning comes true literally: the
plan refactors around five untested exit-code distinctions while asserting that
a refactor over an untested distinction is how the distinction dies.

**What closes it.** Add one step — *"pin the digit at F-265's five sites and
change `refuse_write_block`/`emit`/`write_block` to carry decisions"* — before
the consume step, or attach each mandate to an existing row by name. And delete
*"P0 moves these functions"* from condition 10.

### I-3. Condition 6 requires F-260 caught by a test; §7 reassigns F-260 to P1 and rules that P0 does not touch `mt`.

**Site:** §6 condition 6 line 670; §7 line 743.

**What is wrong.** Condition 6's heading: *"**F-259 and F-260 are caught by a
TEST**, not by construction."* §7: *"**F-260 — REASSIGNED FROM P0 TO P1 (I4).**
… It was filed against P0, but **P0 does not touch `mt`** — §7 places `mt`'s
adoption in P1 … **P1 owns it** … F-259 stays with P0."*

Condition 6's **body** is entirely about F-259 (`me sysw wipe --fill zeros`,
`WriteBlock::Terminal`, the pty probe). Only the heading names F-260. The I-4
fold added §7's reassignment and left the heading naming F-260 — a diff
falsifying text it never touched.

**The concrete failure.** §6 is *"WHAT MUST BE TRUE TO CLOSE P0."* An
implementer reading condition 6 must either (a) write a test against `mt`'s
message, contradicting §7 and P0's scope, or (b) declare a closure condition
unmet. Neither closes P0. It is round 3's I-3 shape — a gate unsatisfiable
against the text the plan itself mandates — in a different section.

**What closes it.** Delete *"and F-260"* from condition 6's heading; the body
already scopes itself to F-259, and §7 already owns the reassignment.

### I-4. Condition 10's digit clause is still violated by four §4 steps, and `--expect`'s exit code is pinned by nothing — `EXIT_INVALID` appears zero times in the plan.

**Site:** §6 condition 10 line 724; §4 steps 5, 6, 7, 8; §4 step 9.

**What is wrong.** Condition 10's substantive sentence — *"Every gate in §4 that
asserts a refusal pins the **digit**"* — survives the fold unchanged, and §4
still does not obey it:

| step | refusal asserted | digit? |
| --- | --- | --- |
| 5 | the remedy refusal | no |
| 6 | *"decided before `Cli::parse()`, asserted end-to-end in the donor"* | no |
| 7 | *"the argv gate **refuses** by class"* | no |
| 8 | four `--expect` **refusals** | no |

Round 3 named all four and prescribed *"add the digit to steps 5–8's refusal
clauses, naming rc=4 for `--expect`."* Measured here: `grep -c 'EXIT_INVALID'`
over the plan → **0**, while `crates/me-cli/src/main.rs:298` defines
`const EXIT_INVALID: i32 = 4` and §3 lines 337–344 measure `--expect`'s refusal
at **rc=4**.

**The concrete failure.** Step 9's differential covers *"every code `me`
produces today"*, and `--expect` does not exist today — so its rc=4 refusals sit
outside the one gate that pins codes. Step 8's gate accepts any non-zero. P0's
newest funds-path refusal — *"`--expect descriptor,cosigner` refuses an
`md1`-only payload"*, the case §6g exists for — ships with its exit code pinned
by nothing, which is F-265's exact failure mode reproduced inside the condition
written to end it.

**What closes it.** Name **rc=4** in step 8's gate and the digit in steps 5–7's
refusal clauses, and add `EXIT_INVALID` to the constant enumeration at line 548
(M-1) so the plan's own list is exhaustive.

---

## MINOR

**M-1 … M-7.** Round 3's seven Minors, **all seven unclosed**, dispositions and
evidence in the Question 1 table above. M-2 and M-3 are now unfixed across
**three** folds; M-4 across three.

**M-8. Condition 9's fold left a dangling *"either"* — a sentence splice.**
Line 714–715: *"so P0 **either** fixes the recipe (`fc -W`, edit, `fc -R`)
**The remedy must make the recipe WORK** — flush, edit, reload …"* The `or`
half was deleted (correctly — that was I-3) and the `either` was not, and the
two sentences run together with no punctuation. The substance is unambiguous
from what follows; the sentence is not.

**M-9. Conditions 2 and 3 have no step and are re-run by nothing.** Condition 2
is a measurement dated *"passing as of 2026-08-26"*; condition 3 is an
unassigned re-measurement (*"Expect 2 for a bad HRP, 1 for an `md1` HRP that
fails to decode"*). Both are cheap; neither is scheduled, and neither is
re-checked at close. State them as *"re-run at the consume step"* or move them
to §8.

**M-10. Two of the new by-name references do not map 1:1 to a table row.**
*"the signature change"* (three exist in the plan; one has a step) and *"the
adoption gate"* (no row is named "adoption"; §7 uses the word for later
phases). The mechanism is sound; these two names are not yet unique. Naming the
row — *"the consume step"* — closes the second; qualifying the first as *"the
`no_records_guard` signature change"* closes it.

**M-11. Round 3's C-2 residue: three sites still assert the retracted
quantity.** Line 96 *"Nothing crosses a crate boundary until **those 11** are a
library"* (under Variant B, six of the eleven never become a library), line 474
*"every type and constant **the 11** reference"*, line 614 *"proves the closure
is really **11** and not more"* (the move relocates six definitions, so it
cannot prove anything about the other five). Round 3 named all three. Demoted
from Critical because the two sites that decided the *split* — §3's file map and
§2.4 — are both correctly fixed, so no implementer can now read the plan as
splitting `read_records`; what remains is a stale count, not a wrong
instruction.

---

## NIT

**N-1.** Round 3's N-1, unclosed: step 1's gate is ``grep -c 'EXIT_' `` —
trailing space, no file, no way to scope a grep to one function — and step 2's
is `<lib module>`. Prose gates dressed as commands.

**N-2.** Round 3's N-2, unclosed: step 5's *"Blocked on F-264 — see §6 condition
5"* points at the condition that **demands** the positive test; the condition
that **authorises the fix** is 9.

---

## WHAT I VERIFIED HERE

Absolute paths only; no exit code read through a pipe; nothing re-derived that
the brief listed as machine-checked.

| check | result |
| --- | --- |
| `grep -c 'step [0-9]'` (the fold's gate) | **0** — reproduced |
| `grep -n 'STEP [0-9]'` | `208` `234` `517` `530` `546` — **five** |
| `grep -ni 'steps [0-9]'` | `539` `543` — **two** |
| `grep -ni 'step-[0-9]'` | `686` — **one** |
| `grep -n -A1 'step$'` | `450`/`451` — **one**, line-wrapped |
| `scripts/fold-propagation-check.sh` with those patterns | **exit 1**, `5b`/`5c` still present at 543 |
| `grep -c 'EXIT_INVALID'` over the plan | **0** |
| `const EXIT` in `crates/me-cli/src/main.rs` | `295` `296` `297` **`298` `EXIT_INVALID`** — four; plan line 548 says three |
| `EXIT_REFUSED` use sites | `407` `599` `2026` — **three**; plan line 211 says four |
| step 2's gate contains a test | **no** — `grep -c 'pub const EXIT' main.rs` is green on the untouched tree (all four are bare `const`) |
| the only pty assertion in §4 | step 4, asserting the **post-fix** behaviour — cannot pin pre-move behaviour |
| §4 rows naming `refuse_write_block`/`read_records`/`emit` | **one** (line 506), and only as STAYING |
| condition 10 self-consistency | *"stays in `me`"* (721) vs *"P0 moves these functions"* (723) |
| condition 6 vs §7 on F-260 | `670` requires it; `743` reassigns it to P1 and rules P0 does not touch `mt` |
| §3's file/step count *"seven files and §4 sequences six"* | **correct** — `lib.rs` is the one never sequenced. Not a finding |
| by-name resolution | *the move*/*the mask split*/*the remedy work*/*the value-shape layer* → unique; *the signature change*/*the adoption gate* → not |
| `git show aa89bb5` touches step 2's row | **no** |

**Accepted as already machine-checked, per the brief:** `plan-table-check.sh`
(55 rows, 0 malformed), `plan-cite-check.sh` (13/13, 0 dangling), the
five-phrase fold-propagation sweep, the moving set's single `EXIT_*` reference
and its callers at `main.rs:1930/2050`, and that steps 1+2 together avoid
`pub const EXIT_*`. F-264/F-265 as P0-owned conditions 9 and 10.

## WHAT THE FOLD GOT RIGHT

Recorded so round 5 does not re-open it:

- **The by-name mechanism is the right answer.** Where applied — ten of the
  thirteen sites round 3 named — it is strictly better than renumbering, and
  four of the six names resolve to exactly one step with no ambiguity. The
  defect is coverage and verification, not design. Do not replace the mechanism.
- **Condition 9 is properly closed.** The unsatisfiable-remedy stall is gone,
  and the escape hatch is replaced with an escalation path (*"a finding to
  raise, not a wording to settle for"*) rather than another hatch.
- **`read_records` stays whole, everywhere that decides it.** §3's file map and
  §2.4 both now say so, with the two-of-three-refs-outside-the-arm reason
  attached. No reading of the plan now produces the three-reference moving set.
- **Deleting the two doomed paragraphs rather than patching them** was the
  right call — the tie-break paragraph with the false example is gone, not
  corrected.
- **Variant B's arithmetic is settled** and this round did not re-derive it.

---

**VERDICT: NOT GREEN — 2 Critical, 4 Important.** No code may be written against
this plan.

**The answer to the brief's question — did the remedy remove the failure mode or
relocate it — is: relocated, twice.** Once into the four blind spots of the grep
that certified it (nine surviving numbered sites, six stale). And once into
name-space, where *"the move carries a pty assertion"* now resolves to exactly
the right step and that step still has no pty assertion. Both Criticals are the
same shape this cycle keeps producing: **the fold corrected the sentence the
reviewer pointed at and left the thing the sentence refers to.** Neither is
design work — C-1 is nine conversions plus one rewritten paragraph, C-2 is one
table cell. The four Importants are one clause each, and three of them
(I-1, I-2, I-4) are round 3 findings the fold did not act on.
