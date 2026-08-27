# R0 ROUND 5 — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md`

**Artifact:** `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` at `64d9d14`
(worktree `review/p0-r5`).
**Object:** (1) did the fold close round 4's 2C/4I/10M/2N; (2) is the plan
executable end to end by one implementer; (3) prose/table contradictions — with
the fold's new `scripts/plan-stepref-check.sh` as the primary target.
**Date:** 2026-08-26.

## VERDICT — **NOT GREEN**

| severity | count |
| --- | --- |
| **Critical** | **0** |
| **Important** | **5** |
| Minor | 7 |
| Nit | 3 |

**Both Criticals are genuinely closed, and C-1 is closed by more than its own
gate.** I ran an independent sweep the script cannot be blamed for — whole file
joined into one line, case-insensitive, plurals, hyphens, spelled-out numerals
and bare row identifiers — and it returns **exactly one hit**, `STEP ZERO IS
INSIDE me` (line 96), which references no table row. The rule is true. This is
the first round in four where that is so.

**The single most important finding: the fold repeated, at one-third scale, the
process failure it opens by confessing.** Its commit message names C-1, C-2,
I-1, I-2 and nine Minors/Nits. It does **not** name **I-3** or **I-4**, and
neither was touched. **I-2 is named and was not actually done** — the sentence
round 4 asked to be deleted (*"P0 moves these functions"*, line 736) is still
there, two sentences after *"Every one of them stays in `me`"*, and the
substantive half (no §4 step edits the functions that stay) is untouched. Round
4's score on this axis was 9 skipped of 12; this fold's is 3 skipped of 6, one
of them mis-reported as folded. **The remaining four Importants are all round-3
findings, now unactioned across three consecutive folds.**

**On the script:** it does catch all four mechanisms it was built for — I planted
each and each goes RED, and a clean copy stays green. Its declared blind spot is
honest as far as it goes, but it is **not the only one**: four further
mechanisms pass it silently, and one of them (prose inside a table cell) is live
in this document, because §4's cells now carry multi-sentence rationale.

---

## QUESTION 1 — DISPOSITION OF ROUND 4's FINDINGS

| # | disposition | evidence |
| --- | --- | --- |
| **C-1** rule false in the plan that states it; 9 sites, certifying grep blind on 4 mechanisms | **CLOSED** | `plan-stepref-check.sh` exit 0 (verified, not through a pipe). Independent broader sweep: 1 hit, `STEP ZERO` (96), not a row reference. All ten numbered sites converted by name; line 539–543 rewritten to state the distinction without enumerating |
| **C-2** the move's mandated pty pin is in no step | **CLOSED** | Step 2's cell (505) now reads *"carrying a **pty assertion pinning the terminal refusal BEFORE and AFTER**"*. The refusal can no longer be lost in the move with every gate green. Residue — it pins no digit — folded into **I-4** below, not counted twice |
| **I-1** move has no test gate; retracted caller arithmetic; `Destination`/`WriteBlock`/`use super::` unnamed | **PARTIAL** | Parts 1 and 3 closed in step 2's cell (*"Plus a real test, not only greps"*; the private enums and `use super::` now named). **Part 2 not closed:** line 525 still asserts the retracted figure as a bolded finding → **I-1** below |
| **I-2** no step edits the staying functions; condition 10 self-contradicts | **NOT CLOSED** | Condition 10 rewritten but line 736 *"P0 moves these functions"* survives verbatim, contradicting line 734 *"Every one of them stays in `me`"*. No step added for four of F-265's five sites, for `refuse_write_block`'s decision signature (382), or for `emit`/`write_block` (condition 6) → **I-2** |
| **I-3** condition 6 requires F-260 caught by a test; §7 reassigns it to P1 | **NOT CLOSED** | Line 683 unchanged; line 756 unchanged. Not mentioned in the commit message → **I-3** |
| **I-4** digit clause violated by steps 5–8; `EXIT_INVALID` absent | **PARTIAL** | `EXIT_INVALID` now appears (556, 559) — that half came in via M-1. **The digit clause is still violated**, now at five gates rather than four, because step 2's new pty assertion also omits it → **I-4** |
| **M-1** three vs four exit constants | **CLOSED** | 556–559, *"FOUR, not three"*, `main.rs:295-298` |
| **M-2** *"used four times"* | **CLOSED** | 211, *"used **three** times"*, `:407` `:599` `:2026` |
| **M-3** `--out` overwrite gated on `channel.rs` | **CLOSED** | Step 9's cell: asserted at `write_private`, *"which stays in `me`"* |
| **M-4** no step creates the crate | **CLOSED** | Step 9b added (513) |
| **M-5** *"all 388 pass"* stale by construction | **CLOSED** | Step 10: *"the 388 pre-existing tests still pass, plus every test added along the way"* |
| **M-6** condition 4 ≠ step 8 | **CLOSED** | Condition 4 rewritten in full (667–671) |
| **M-7** step 6's observable unnamed; `--allow-argv-secret` regression | **CLOSED** | Step 6's cell names the observable and the PARSE requirement. Citations verified independently: `main.rs:252` declares `allow_argv_secret`, `:1116` binds it, `:1127` consumes it |
| **M-8** dangling *"either"* in condition 9 | **NOT CLOSED** | 727–728 unchanged → **M-1** below |
| **M-9** conditions 2 and 3 have no step | **NOT CLOSED** | No step added → **M-2** below |
| **M-10** *"the signature change"* / *"the adoption gate"* not 1:1 | **NOT CLOSED**, and now worse | 453 unchanged; 540 still unqualified; *"the crate adoption"* now has two candidate rows → **M-3** below |
| **M-11** three sites still assert *"the 11"* | **NOT CLOSED** | 96, 473, 624 unchanged → **M-4** below |
| **N-1** grep with no file; `<lib module>` placeholder | **PARTIAL** | Step 1's grep replaced with *"the `EXIT_*` count inside `no_records_guard`"*. Step 2 still carries `<lib module>` → **N-1** below |
| **N-2** step 5 pointed at condition 5 | **CLOSED** | Step 5 now *"see §6 condition 9"* |

**Score: 2 of 2 Critical closed. 1 of 4 Important closed** (C-2's core; counted
as C-2 not I), **1 partial, 2 untouched. 7 of 11 Minor closed** (round 4's
header table says 10; its body enumerates M-1…M-11). **1 of 2 Nit closed.**

*(Round 4's I-1 part 2, I-2, I-3, I-4, M-8, M-9, M-10 and M-11 originate in
round 3. This is their third fold without action.)*

---

## QUESTION 2 — THE SCRIPT

### Does it catch what it claims? — **yes, all four, verified by planting**

Each probe is a copy of the plan with one line appended; the gate was run with
its exit code read directly, never through a pipe.

| probe | planted text | result |
| --- | --- | --- |
| A case | `This paragraph asserts STEP 6 is the authority.` | **exit 1** |
| B plurality | `This paragraph says steps 1 and 7 are regression-gated.` | **exit 1** |
| C hyphenation | `This refers to the old step-3 gate.` | **exit 1** |
| D line-wrap | `…that step` ⏎ `1's own check would have passed it.` | **exit 1** |
| E plain lowercase | `This paragraph says step 2 does the move.` | **exit 1** |
| CTRL nothing planted | — | **exit 0** |

It is also not trigger-happy: `lockstep parity test` (266) does not match,
because `\b` fails between `k` and `s`. Against the pre-fold plan (`aa89bb5`) it
reports **17**, reproducing the commit message's figure.

### Is its stated blind spot honest? — **incomplete. Four more, one of them live**

Same method, one planted line each:

| probe | planted text | result |
| --- | --- | --- |
| F spelled-out | `The authority for this is step three, not the table.` | **exit 0 — MISSED** |
| G bare row id | `Row 9b is where the crate boundary is drawn, after 8 and before 10.` | **exit 0 — MISSED** |
| **H inside a table cell** | step 7's cell edited to `…value-shape, additive, done after step 3` | **exit 0 — MISSED** |
| I ordinal | `The eighth step owns the vocabulary.` | **exit 0 — MISSED** |

See **I-5**.

### The blind spot it *does* declare — by-NAME references, checked by hand

Every by-name reference, resolved against the current table:

| name | sites | row | does the row do what the name says? |
| --- | --- | --- | --- |
| *the move* | 86, 450, 470, 472, 527, 530–537, 551, 565, 611, 614–623 | **2** | **yes** — and the pty claim at 535 now resolves true, which is C-2 closed. One exception at 472, see **M-6** |
| *the mask split* | 611, 614 | **3** | yes |
| *the remedy work* | 234, 680 | **5** | yes — step 5 carries the positive RUN-it test |
| *the value-shape layer / argv guard* | 545, 597 | **7** | yes |
| *the channel/exit work* | 208 | **9** | yes — round 4 measured this referent as 9 and the fold hit it |
| *the old observation-types gate* | 699 | **4** | yes |
| *the signature change* | 540, 565, 611 | **1** | ambiguous — three signature changes exist (**M-3**) |
| *the crate adoption* | 541 | **10?** | ambiguous — 9b already says *"`me` depends on it by path"* (**M-3**) |
| *the adoption gate* | 453 | **none** | no row is named adoption (**M-3**) |
| *the step that proves the closure is really 11* | 624 | **2** | **no** — the move relocates six definitions (**M-4**) |

---

## QUESTION 3 — IS THE PLAN EXECUTABLE END TO END?

### Can every gate fail? — **yes, all twelve.** This part is now sound.

| # | can it fail? | note |
| --- | --- | --- |
| 1 signature change | yes | `EXIT_*` count in `no_records_guard` 1 → 0; N-1's grep-with-no-file is fixed |
| 2 the move | **yes, now** | the pty assertion's AFTER half fails if the terminal arm is lost. No digit (**I-4**) |
| 3 mask split | yes | `0o620 & 0o044 == 0`, RED today |
| 4 `observation.rs` + pty | yes | the strongest gate in the plan; pins the digit |
| 5 `remedy.rs` | yes | F-264 is live |
| 6 layer 1 | yes | observable named; `--allow-argv-secret` PARSE clause verified against `main.rs:252/1116/1127` |
| 7 layer 2 | yes | crate unit tests are RED; the de-gating paragraph is gone |
| 8 `--expect` | yes | the flag does not exist. No digit (**I-4**) |
| 9 `exit.rs` + `channel.rs` | yes | differential against the pre-change binary |
| 9b create the crate | yes | the crate does not exist. Not RED-first (**M-5**) |
| 10 consume | yes, as regression | count no longer stale |
| 11 publish | n/a | operator-gated |

### Does every §6 condition have a step that builds it? — **no, four do not**

| condition | step | |
| --- | --- | --- |
| 1 tests pass | 10 | ✓ |
| 2 §5b's 16 verb checks | **none** | **M-2** |
| 3 §6f `mnemonic` cell under `inspect` | **none** | **M-2** |
| 4 `--expect` refusals | 8 | ✓ content now matches |
| 5 §6h history + positive test | 5 | ✓ |
| 6 F-259 **and F-260** by test | 4 for F-259; **none for F-260, and §7 forbids it** | **I-3**. Its *"`emit` and `write_block` change signature"* clause also has no step — **I-2** |
| 7 §8 `CLOSED`-grep | n/a | process |
| 8 pre-parser guard + override | 6 | ✓ |
| 9 F-264 | 5 | ✓ |
| 10 F-265 at **all five sites** | **1 of 5 at most** | **I-2** |
| 11 R0 0C/0I | n/a | |

### Prose the table contradicts, or vice versa

**Three sites: I-1, M-5 and M-6 below.** Both of the sites round 4 called
contradictions are gone.

---

## IMPORTANT

### I-1. Step 2's cell retracts the four-caller arithmetic; a bolded paragraph twenty lines later still asserts it as a finding.

**Site:** line 525–528 vs step 2's cell, line 505.

Line 525, verbatim and unchanged since round 3:

> **FOUR CALLERS LIVE OUTSIDE THE CLOSURE (probe I-2).** `write_private` has
> **three** and `refuse_world_readable_stdout` **one**, producing four `E0425`s
> that nothing in this plan predicted.

Line 505, added by this fold:

> the **"four callers outside the closure" figure was measured against the
> REJECTED move, and both those functions stay**

Both functions stay in `me` (§3's file map, 421–422), so none of those four
callers crosses anything and none of the four `E0425`s occurs.

**The concrete failure.** The implementer at the move is told by a bolded
section heading to expect four specific undefined-name errors. They will not
occur. The two errors that *will* occur — the private `Destination`/`WriteBlock`
enums and the `use super::` at `main.rs:2165` — are named only in the table
cell. Under a build-shaped gate the cheapest green for a broken `use super::` is
to delete or `#[ignore]` the two tests, one of which is
`write_block_decides_both_gates_once` (`main.rs:2201`), the only unit pin on the
terminal decision and the test condition 6 names as *"expected to change"*.
Round 4 said this; the fold fixed the cell and left the paragraph.

**What closes it.** Rewrite line 525's heading and body to the retraction the
table already carries, or delete the paragraph — its surviving requirement
(*"the move must enumerate callers, not just callees"*) is already in step 2's
cell.

---

### I-2. No §4 step edits the functions that stay in `me`, so condition 10 cannot close — and condition 10 still contradicts itself in the sentence round 4 asked to be deleted.

**Site:** §6 condition 10, lines 734–738; §3 line 382; §6 condition 6 line 707;
§4's twelve rows.

**(a) The self-contradiction survives.** Lines 734 and 736, four lines apart:

> **F-265 fixed at ALL FIVE SITES.** **Every one of them stays in `me`** —
> `refuse_write_block` ×2, `read_records` ×2, `emit` — so this is work P0 does
> **in the donor**, not in the crate …
>
> **P0 moves these functions**, and a refactor over an untested distinction is
> how the distinction dies.

The commit message lists I-2 among the nine it folded. It rewrote the first
sentence and left the second. I verified the five sites independently against
`design/FOLLOWUPS.md:11754-11760`: `refuse_write_block` Terminal,
`refuse_write_block` WorldReadable, `read_records --in`, `read_records` stdin,
`emit` write-failure. §3's file map keeps **all five** in `me`. *"P0 moves these
functions"* is false about every one of them.

**(b) The substantive half is untouched.** `grep` over §4's rows for
`read_records` / `` `emit` `` / `refuse_write_block` returns **one** row — step 2
— and only in the clause listing them as **STAYING**. Three P0 mandates require
edits to them and none has a step:

- condition 10: F-265 at five sites. Step 4's pty digit reaches at most
  `refuse_write_block`'s Terminal arm. **Four sites unscheduled.**
- §3 line 382: *"`refuse_write_block` returns the DECISION, not `Some(i32)` —
  that signature change belongs to P0."* **No step.**
- condition 6 line 707: *"`emit` and `write_block` change signature."* **No
  step**, and by then `write_block` is in the crate while `emit` is not, so it is
  a cross-boundary change scheduled nowhere.

**The concrete failure.** An implementer walks steps 1 → 11, closes every gate,
and has discharged **one** of condition 10's five sites and neither signature
change. §6 is *"WHAT MUST BE TRUE TO CLOSE P0"*, so P0 does not close. Condition
10's own warning then comes true literally: the plan refactors across five
untested exit-code distinctions while asserting that a refactor over an untested
distinction is how the distinction dies.

**What closes it.** Delete *"P0 moves these functions"* (line 736). Add one row
before the consume step — *"pin the digit at F-265's five sites; `refuse_write_block`,
`emit` and `write_block` carry decisions rather than integers"* — or attach each
mandate to an existing row by name.

---

### I-3. Condition 6 requires F-260 caught by a test; §7 reassigns F-260 to P1 and rules that P0 does not touch `mt`.

**Site:** §6 condition 6 line 683; §7 line 756. Both unchanged; neither is
mentioned in the fold's commit message.

Line 683: *"**F-259 and F-260 are caught by a TEST**, not by construction."*
Line 756: *"**F-260 — REASSIGNED FROM P0 TO P1 (I4).** … **P0 does not touch
`mt`** … **P1 owns it.** F-259 stays with P0."*

Condition 6's **body** is entirely F-259 — `me sysw wipe --fill zeros`,
`WriteBlock::Terminal`, the pty probe. Only the heading names F-260. I confirmed
the reassignment is real outside the plan: `design/FOLLOWUPS.md:11513` carries
*"repo: **mnemonic-transaction**; owning phase: **P1**, reassigned from P0
2026-08-26"*. So the plan is the only thing still asserting it.

**The concrete failure.** Unchanged from round 4: an implementer reading §6 must
either write a test against `mt`'s message — contradicting §7 and P0's scope —
or declare a closure condition unmet. Neither closes P0.

**What closes it.** Delete *"and F-260"* from line 683. (§3's file map at line
416 also cites F-260 for `observation.rs`; that one is defensible — the mode
*type* is what prevents F-260's class — but it reads better as *"F-259, and
F-260's class"*.)

---

### I-4. Condition 10's digit clause is violated by five §4 gates — including the pty assertion this fold added to close C-2, at F-265's own site #1.

**Site:** §6 condition 10 lines 737–738; §4 steps 2, 5, 6, 7, 8.

Condition 10's closing sentence, unchanged: *"Every gate in §4 that asserts a
refusal pins the **digit**."* §4's own prose at 517–523 says the same thing
harder: *"So `!success()` is not enough: **§4's pty assertion must assert the
exit code itself**, or it misses even the arm it is named for."*

| step | refusal asserted | digit? |
| --- | --- | --- |
| **2** | *"pty assertion pinning the terminal refusal BEFORE and AFTER"* | **no** |
| 5 | the remedy refusal | no |
| 6 | *"decided before `Cli::parse()`"* | no |
| 7 | *"the argv gate **refuses** by class"* | no |
| 8 | four `--expect` refusals | no; `rc=4` appears at 341 and 349 but in **no gate** |

Step 4 is the only gate that obeys it (*"pinning the **exit digit**"*).

**The concrete failure, and why step 2 is the one that matters.** F-265's site #1
is `refuse_write_block`'s **Terminal arm** — precisely the arm step 2's new
assertion exists to protect — and F-265's measurement is that it can be respelled
**2 → 3 with 388/388 green**. An assertion that the tool *"still refuses"* passes
either way. So the cell added to close a Critical about the terminal path is
written in exactly the form the repo has already proved blind on that path, while
condition 10 and §4's own prose both assert it is not. Separately, `--expect` is
P0's newest funds-path refusal (*"`--expect descriptor,cosigner` refuses an
`md1`-only payload"*, the §6g case) and step 9's differential covers only *"every
code `me` produces today"* — which cannot reach a flag that does not exist today.
Its exit code is pinned by nothing.

**What closes it.** Add *"pinning the digit"* to step 2's assertion, name `rc=4`
in step 8's gate, and add the digit to steps 5–7's refusal clauses. Five short
edits.

---

### I-5. `plan-stepref-check.sh` declares one blind spot and has at least five; the one that is live here is that ALL prose inside a table cell is exempt, not just the row number.

**Site:** `scripts/plan-stepref-check.sh` lines 19–21 (the NOT COVERED header)
and line 32 (`next if $l[$i] =~ /^\s*\|/;`).

The header states exactly one exclusion — a by-NAME reference gone stale — and
its rationale for the table exemption is *"the TABLE may number its rows"*, i.e.
the `#` column. **The implementation exempts the whole line**, and §4's cells are
now multi-sentence rationale: step 2's cell is ~900 characters of prose
containing *"round-4 C-2"*, *"round-4 I-1"*, *"the REJECTED move"*. A
cross-step ordering claim written there — *"done after step 3"* — is invisible
to the gate. Probe H above confirms it: exit **0** with that text planted in
step 7's cell.

Three further undeclared mechanisms, all confirmed by planting (probes F, G, I):
spelled-out numerals (*"step three"*), bare row identifiers (*"Row 9b … after 8
and before 10"* — and this plan now **has** a row called 9b), and ordinals
(*"The eighth step"*).

**The concrete failure.** This gate is the entire remedy for a defect that cost
rounds 2, 3 and 4. The next fold runs it, gets exit 0, and certifies the rule —
which is the round-4 failure exactly, one mechanism over. The risk is not
hypothetical: this fold **inserted a row (9b)**, so every ordering claim in the
document is now one renumbering old, and *"9b"* is the one row identifier the
regex cannot see without the word `step` in front of it.

This repo already treats the class as a real defect: `design/FOLLOWUPS.md:11569`
is **F-261 — *"`plan-table-check.sh` silently skips INDENTED tables, and does not
list that among its blind spots"***, filed and closed. This is the same finding
about the newer script.

**What closes it.** Strip the leading `| N |` cell and check the remainder,
instead of exempting the line — that keeps the row number exempt and puts the
cell prose back under the gate. Add the other three mechanisms to the regex or,
at minimum, to the NOT COVERED header. **Also: the plan never names the command.**
`grep -n 'plan-stepref-check' <plan>` → **0 hits**, while line 118 shows the plan
already cites `plan-cite-check.sh` by name. The rule at line 606 ships with no
command attached, which is the condition that produced three rounds of ad-hoc
greps.

---

## MINOR

**M-1. Condition 9's dangling *"either"* — round 4's M-8, unchanged.** Lines
727–728: *"so P0 **either** fixes the recipe (`fc -W`, edit, `fc -R`) **The
remedy must make the recipe WORK** — flush, edit, reload …"*. Two sentences
spliced with no punctuation and an orphaned correlative. Substance unambiguous;
the sentence is not.

**M-2. Conditions 2 and 3 still have no step — round 4's M-9, unchanged.**
Condition 2 is a measurement dated *"passing as of 2026-08-26"*, condition 3 an
unassigned re-measurement. Neither is re-run by any row, so both are true at
close only because they were true when typed. State them as re-run at the consume
step, or move them to §8.

**M-3. Three by-name references still do not map 1:1 — round 4's M-10, and the
fold added a second candidate for one of them.** *"the signature change"* (540,
611) has three referents in the plan: `no_records_guard` (504, *"the ONE"*),
`refuse_write_block` (382), `emit`/`write_block` (707). *"the adoption gate"*
(453) names no row, and §7 uses *"adoption"* twice for later phases (754, 758).
And *"the crate adoption"* (541) now resolves ambiguously between step 9b — which
already says *"`me` depends on it by path"* — and step 10. Qualifying the first
as *"the `no_records_guard` signature change"* and the last as *"the consume
step"* closes two of the three.

**M-4. Three sites still assert the retracted quantity — round 4's M-11,
unchanged.** Line 96 *"Nothing crosses a crate boundary until **those 11** are a
library"* (six of the eleven never become a library), line 473 *"every type and
constant **the 11** reference"*, line 624 *"proves the closure is really **11**
and not more"* (the move relocates five definitions plus the stub, so it proves
nothing about the other six). Named in rounds 3 and 4.

**M-5. The M5 paragraph partitions the table wrongly again — and this fold is
what made it wrong.** Lines 539–546: *"**TWO PIECES OF WORK ARE REGRESSION-GATED
RATHER THAN RED-FIRST** … They are the signature change and the crate adoption …
**Everything else is RED-first**."* The same commit added **9b**, whose gate is
*"the crate builds standalone; `me` depends on it by path"* — a build, not a test
that fails first — and made **the move's** gate a **BEFORE-and-AFTER** pin, which
is by construction green before the change. Four items, not two. This is much
milder than round 4's line 543 because nothing is de-gated and the paragraph
itself concedes *"The table states each gate"* — but the failure mode is
specific: an implementer taking *"everything else is RED-first"* literally at the
move writes the assertion so it fails first, which is the **post-fix** form round
4 proved *"cannot pin behaviour across a move"*. Say *"the refactor-shaped work —
the `no_records_guard` signature change, the move's regression pin, the crate
creation and the consume step — is gated by the suite rather than RED-first"*, or
drop the count.

**M-6. *"The move must enumerate every type and constant … reachable WITHOUT an
inherent impl in the crate"* is attached to the wrong work now that 9b exists.**
Line 472–476. At the move, everything lands in `me`'s **own** lib half, where
`Class` is in the same crate and an inherent impl compiles fine — E0116 cannot
occur. The crate boundary appears at **9b**, which is where that enumeration has
teeth. Backstopped by 9b's gate (*"no `EXIT_*` and no `Class` in it"*), so this is
a mis-assignment rather than a hole, but it is the by-NAME staleness class the
script explicitly cannot see, introduced by this fold's own insertion.

**M-7. §7's F-260 paragraph claims the follow-up file is updated *"in this same
fold"*.** Line 762. It is — `FOLLOWUPS.md:11513` carries the P1 reassignment —
but it landed in **`09da392`**, an earlier fold, and `64d9d14` touches only the
plan and the script. Harmless today; a claim about a sibling file that no gate
checks and that reads as false to anyone diffing this commit.

---

## NIT

**N-1. Step 2's gate still contains `<lib module>` — round 4's N-1, half
closed.** Step 1's `` grep -c 'EXIT_' `` with a trailing space and no file is
properly replaced by *"the `EXIT_*` count inside `no_records_guard`"*. Step 2's
`` grep -c 'EXIT_' <lib module> == 0 `` is still a placeholder dressed as a
command.

**N-2. The script's count double-reports.** Every in-line hit is also reported
against the **preceding** line, whose text is then printed as the offending
prose — usually blank. Against the pre-fold plan it prints **17** for **10**
distinct sites (208, 234, 450, 517, 530, 539, 542, 543, 546, 686), and the commit
message repeats the 17 as a count of references. Cause: the probe string is
`$l[$i] . " " . $l[$i+1]`, so a hit on line *i+1* also matches at *i*. Fix: only
report at *i* when the match spans the join.

**N-3. `STEP ZERO IS INSIDE `me`` (line 96) is a spelled-out step reference the
gate cannot see.** Harmless — there is no step zero to go stale against — but it
is the fifth mechanism of the family the script was written to close, sitting in
the document the script guards.

---

## WHAT I VERIFIED HERE

Absolute paths; exit codes read directly, never through a pipe; nothing
re-derived that the brief listed as machine-checked.

| check | result |
| --- | --- |
| `plan-stepref-check.sh <plan>` | **exit 0** — reproduced |
| independent sweep: file joined, case-insensitive, plurals, hyphens, spelled-out, bare ids | **1 hit** — `STEP ZERO` (96), not a row reference. **C-1 closed** |
| `grep -n -i 'step' <plan>` | 12 hits, none a numbered reference to a row |
| planted probes A–E (case, plurality, hyphen, wrap, plain) | **exit 1 each** |
| planted probes F, G, H, I (spelled-out, bare id, **table cell**, ordinal) | **exit 0 each — MISSED** |
| control copy, nothing planted | **exit 0** |
| script vs pre-fold plan (`aa89bb5`) | **exit 1, 17 hits / 10 sites** |
| `grep -n 'plan-stepref-check' <plan>` | **0** — the plan never names its own gate |
| step-number references inside table cells today | **0** — the exemption hides nothing yet |
| `P0 moves these functions` | line **736**, still present |
| `Every one of them stays in me` | line **734** |
| F-265's five sites | `FOLLOWUPS.md:11747-11760` — matches condition 10's enumeration exactly; **all five stay in `me`** per §3's file map |
| §4 rows naming `read_records` / `` `emit` `` / `refuse_write_block` | **one** (step 2), and only as STAYING |
| `F-259 and F-260 are caught by a TEST` | line **683**, unchanged |
| `F-260 — REASSIGNED FROM P0 TO P1` | line **756**, unchanged; `FOLLOWUPS.md:11513` confirms P1 |
| `EXIT_INVALID` in the plan | **2** hits (556, 559) — was 0 |
| `rc=4` in the plan | 341, 349 — **prose only, no gate** |
| `--allow-argv-secret` citations (new this fold) | `main.rs:252` declares, `:1116` binds, `:1127` consumes — **all three correct** |
| `FOUR CALLERS LIVE OUTSIDE THE CLOSURE` | line **525**, unchanged, contradicted by line 505 |
| `either fixes the recipe` | line **727**, unchanged |
| *"the 11"* residue | lines **96, 473, 624**, unchanged |
| §3 *"seven files and §4 sequences six"* | still correct after 9b — 9b names no file |
| `lockstep parity test` (266) vs the regex | correctly **not** matched |

**Accepted as already machine-checked, per the brief:** `plan-table-check.sh`
(56 rows, 0 malformed), `plan-cite-check.sh` (17/17, 0 dangling),
`fold-propagation-check.sh` (exit 0), the four exit constants at
`main.rs:295-298`, `EXIT_REFUSED` at `:407`/`:599`/`:2026`, the moving set's
single `EXIT_*` reference, and that steps 1+2 together avoid `pub const EXIT_*`.

## WHAT THE FOLD GOT RIGHT

Recorded so round 6 does not re-open it:

- **Replacing a discipline with a command was the correct move, and the command
  works.** All four mechanisms are caught; the control stays green; the plan is
  genuinely clean under a check broader than its own. Do not replace the script —
  widen it.
- **C-2 is properly closed.** The move now carries a BEFORE-and-AFTER pty
  assertion, and the *before* half is the right shape: round 4's whole point was
  that a post-fix assertion cannot pin behaviour across a move. Only the digit is
  missing.
- **Nine skipped findings folded, and the skip named as the author's own failure
  in the commit message.** Seven Minors and a Nit are genuinely closed with
  measured values, not descriptions — *"FOUR, not three"*, *"used **three**
  times"* with the three line numbers.
- **9b is the right shape for M-4.** Making the crate boundary a step, rather
  than an implication of step 10, is what lets *"no `Class` in it"* be a gate at
  the moment it can fail.
- **Step 6's `--allow-argv-secret` PARSE clause is a genuine catch** — moving the
  decision off clap without leaving the flag declared would have regressed the
  exact flag P0 is told to preserve, and the three citations are correct.

---

**VERDICT: NOT GREEN — 0 Critical, 5 Important.** No code may be written against
this plan.

**The answer to the brief's question about the script:** it catches everything it
was built to catch, and its declared blind spot is real but not exhaustive —
prose inside a table cell is unchecked, and §4's cells are where cross-step
ordering claims live. **The answer about the by-name references:** ten of eleven
resolve to exactly one row and that row does what the name says; the eleventh
(*"the step that proves the closure is really 11"*) does not, and one mandate
(*"the move must enumerate every type and constant"*) drifted onto the wrong work
the moment this fold inserted 9b — which is the by-name staleness class, live,
in the same commit that declared it unreachable by tooling.

**The blocking work is small and none of it is design.** I-1 is one paragraph,
I-2 is one deletion plus one table row, I-3 is three words, I-4 is five short
clauses, I-5 is one line of perl plus one header line. What has not shrunk in
three folds is the *set*: four of these five are round-3 findings, and this fold
skipped two of them without naming them — the same failure it opens by
confessing.
