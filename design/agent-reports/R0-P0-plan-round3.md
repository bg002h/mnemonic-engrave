# R0 ROUND 3 — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md`

**Artifact:** `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` at `5b0e6c9`
(worktree `review/p0-r3`).
**Object:** two questions only — (1) did the fold close round 2's 4C/5I/4M/2N,
and (2) is the new eleven-step §4 table sound.
**Date:** 2026-08-26.

## VERDICT — **NOT GREEN**

| severity | count |
| --- | --- |
| **Critical** | **2** |
| **Important** | **3** |
| Minor | 7 |
| Nit | 2 |

**The single most important finding is C-1, and it is round 2's C-1 with the
halves swapped.** Round 2 found that the fold had corrected the prose and left
§4's table carrying the retracted claim. This fold rewrote the table into eleven
steps and did not touch **one line** of the twelve rationale paragraphs beneath
it — so §4's prose is now entirely on the old nine-step numbering, in **13
instruction-shaped sites**. The paragraph that states the tie-break rule is
itself now false about the table (*"the table's is the `fd.rs` mask split"* — the
table's step 2 is the move), which removes the one mechanism that would let an
implementer resolve the other twelve. **Eight for eight: every fold in this cycle
has introduced a defect.**

---

## QUESTION 1 — DISPOSITION OF ROUND 2's FINDINGS

| # | disposition | evidence |
| --- | --- | --- |
| **C-1** table row 1 commands the impossible intact move | **CLOSED** | §4 line 501–502: step 1 is the `no_records_guard` signature change, step 2 is *"Move the five + the stub"* with `read_records`/`emit`/`write_private`/every `refuse_*` named as staying. Residue: the target file inside `me`'s lib half is still unnamed (`<lib module>`) — M-4 below |
| **C-2** `read_records` contradicts itself across §1/§3/§4 | **PARTIAL → new C-2** | §1 lines 50–56 retracted correctly; **§3's file map at line 416 still reads `read_records`'s class-keyed arm`**, and §2.4 (line 169–174) still calls it *"where the seam actually pays"*. Round 2's remedy named all three sites; one was done |
| **C-3** step 3's gate is the retracted "by construction" claim | **CLOSED** | §4 line 504: step 4's gate is `script -qec` on the emitted words, *"the assertion is the gate, not the type"*, pinning the exit digit, mutation-checked both directions — condition 6's remedy verbatim |
| **C-4** condition 8 has no step that builds it | **CLOSED** | §4 line 506: step 6 is *"pre-parser flag-name guard on raw argv, AND `me`'s `--allow-argv-secret` moved off clap"*, gated *"asserted end-to-end in the donor — not only as a crate unit test"*. Residue: no observable named — M-7 |
| **I-1** §3's I5 paragraph points at the forbidden publish | **CLOSED** | §3 lines 464–468: the `EXIT_OK`/`EXIT_USAGE`/`EXIT_REFUSED` "crossing" list is retracted as *"measured against the rejected 464-line move"*. Residue: the very next sentence still says *"Step 1 must enumerate … the 11"* — C-1/C-2 |
| **I-2** step 4's gate unsatisfiable, F-264 unscheduled | **CLOSED (scheduling)** | §4 line 505 *"Blocked on F-264"*; §6 condition 9 added. **But the fix as scoped re-opens the stall — new I-3** |
| **I-3** §6 omits F-264 and F-265 | **CLOSED** | §6 conditions 9 and 10 added (plan lines 704–714) |
| **I-4** step 5c misses both funds-path defects | **CLOSED** | §4 line 508: step 8 now carries *"an incomplete `md1` set AND an incomplete `mt1` set (three walks, §6g)"* and *"`--allow-unsigned-inputs --expect transaction` does NOT falsely refuse"*, plus the `descriptor,cosigner` clause. Residue: §6 condition 4 was **not** updated to match — M-6 |
| **I-5** step 6 reorder / step 5b needs step 6's type / wrong citation | **PARTIAL** | The mis-citation *"(probe I-3 — the step-1 ordering shape…)"* is deleted from the row (N-1 closed). The substance is not: line 583 still says *"the decision type deferred to **step 5b**"* — a step this fold deleted. Rolled into C-1 |
| **M-1** step 1's 388-vs-389 arithmetic | **CLOSED at its site** | The pty assertion left step 1, so 388 is now consistent there. The same arithmetic migrated to step 10 — M-5 |
| **M-2** *"used four times in its source"* | **NOT CLOSED** | Line 209 unchanged. Measured here: `EXIT_REFUSED` use sites are `crates/me-cli/src/main.rs:407`, `:599`, `:2026` — **three**, plus the declaration at `:297` |
| **M-3** step 6's `--out` clause gates code the step does not build | **NOT CLOSED** | Now §4 line 509 (step 9), text unchanged; §3 line 409 still gives `channel.rs` only `destination`, and `--out` overwriting lives in `write_private`, which line 418 keeps in `me` |
| **M-4** when is `mnemonic-io-lib` created | **NOT CLOSED** | Step 2 → *"`me`'s lib half"* / `<lib module>`; steps 3–9 → crate files; step 10 → *"`me` consumes the crate"*. No step creates it and no step states the handoff |
| **N-1** *"probe I-3"* cited for two findings | **CLOSED** | `grep -n 'I-3'` over the plan → **1** hit, line 330 |
| **N-2** *"80 of its 131 lines, 61%"* does not reproduce | **CLOSED** | The paragraph was deleted outright |

**Score: 2 of 4 Critical closed, 1 partial (→ C-2 below), 4 of 5 Important
closed, 1 of 4 Minor closed, 2 of 2 Nit closed.**

---

## QUESTION 2 — THE NEW §4 TABLE

### Can every gate fail? — step by step

| # | can it fail? | note |
| --- | --- | --- |
| 1 | **yes** | `grep -c 'EXIT_'` inside `no_records_guard` is **1** today (`main.rs:1915`), 0 after. Real |
| 2 | **weakly** | *"builds"* + two greps. `grep -c 'pub const EXIT' main.rs == 0` is **already true on the untouched tree** — the four consts are private (`main.rs:295-298`) — so it only fails if the implementer publishes. **Nothing asserts anything still works.** → I-1 |
| 3 | **yes** | verified RED today: `stdout_world_readable_mode` masks with `& 0o044` at `main.rs:912`, and `0o620 & 0o044 == 0`, so it returns `None` for a 0620 file where the gate demands `Some(0o620)` |
| 4 | **yes** | the probe reproduced it and mutation-checked both directions |
| 5 | **yes**, and that is the problem | see I-3 — one of the two authorised remedies leaves it permanently RED |
| 6 | **underspecified** | the property is stated, the observable is not → M-7 |
| 7 | **half** | the crate unit tests are RED (the crate has none); *"`me sysw pack --nosuchflag <ms1…>` still does not echo the secret"* is **true today** by §3 line 275's own measurement — a regression clause, not a RED-first one |
| 8 | **yes** | `--expect` does not exist, so all four clauses are RED. **No digits pinned** → I-2 |
| 9 | **yes** | differential against the pre-change binary is a genuine gate — but scoped to *"every code `me` produces today"*, which cannot cover step 8's new codes → I-2 |
| 10 | **yes** as a regression gate | count is stale → M-5 |
| 11 | n/a | operator-gated |

### Is step 2's set right? — YES on the arithmetic

Answering the brief's second bullet directly, and confirming rather than
re-deriving the counts it supplied:

- The moving set carries **exactly one** `EXIT_*` reference before step 1
  (`no_records_guard`, `main.rs:1915`) and **zero** after it.
- **Its callers do not force a publish.** `no_records_guard`'s only two call
  sites are `main.rs:1930` and `:2050`, **both inside `read_records`**, which
  stays in `main.rs` where the private consts are visible. `write_block` is
  called at `:1200` and `:2070`, `stdout_world_readable_mode` at `:433`, `:1204`,
  `:2074`, `destination` only from `write_block` — every one in `main.rs`.
- **So steps 1 and 2 together do avoid `pub const EXIT_*`.** The plan's claim
  that one signature change suffices is correct.

What the step does **not** say is in I-1 below.

### Is the `read_records` stays-whole ruling consistent? — NO

See C-2. §1 and §4 agree; §3's file map and §2.4 do not.

### Are conditions 9 and 10 buildable by the steps that claim them?

- **Condition 9 / F-264: the premise is true, the scoping is not.** Step 5's gate
  *"the emitted recipe, RUN under a real interactive zsh, actually removes the
  entry"* is genuinely RED against `me`'s current text — `FOLLOWUPS.md:11717-11725`
  measures exactly that. But condition 9 offers **two** remedies and only one of
  them can turn the gate green. → **I-3**.
- **Condition 10 / F-265: it discharges nothing.** → **I-2**.

---

## CRITICAL

### C-1. §4's rationale is entirely on the OLD nine-step numbering — round 2's C-1 with the halves swapped, in 13 instruction-shaped sites.

**Site:** §3 lines 204, 228, 450, 469; §4 lines 523, 531–533, 539–540, 583,
593–594, 598, 601–603, 606–607; §6 condition 5, line 660.

**What is wrong.** `git show 5b0e6c9` touches the table (lines 501–511), §1's
`read_records` paragraph, §3's I5 paragraph, and §6's conditions. It touches
**none** of the twelve paragraphs that follow the table and explain it. Those
paragraphs number steps against the retracted nine-step ordering. The table now
means:

`1` signature change · `2` **the move** · `3` `fd.rs` split · `4` `observation.rs`
+ pty · `5` `remedy.rs` · `6` layer 1 · `7` layer 2 · `8` `--expect` ·
`9` `exit.rs`+`channel.rs` · `10` consume · `11` publish.

The prose still says:

| line | text | true step |
| --- | --- | --- |
| 204 | *"§6f IS NOT SUFFICIENT AS **STEP 6's** AUTHORITY"* | 9 |
| 228 | *"**Step 6** implements `-`; it does not accept and ignore it."* | 9 |
| 450 | *"with **step 7's** gate written to accept it under a citation"* | 10 |
| 469 | *"**Step 1** must enumerate every type and constant the 11 reference"* | 2 — and it **contradicts line 467 two sentences above**, which this same fold wrote: *"**Step 2** enumerates the real set"* |
| 523 | *"**Step 1** must enumerate **callers**, not just callees"* | 2 |
| 531–533 | *"**Step 1** therefore carries a **pty assertion** pinning the refusal … without that assertion **step 1** proves nothing about the terminal path"* | the move, step 2 — **and step 2 has no pty assertion** |
| 539–540 | *"Steps 2, 3, 4, 5, **5b**, **5c** and 6 are RED-first; **1 and 7 are regression-gated**"* | `5b`/`5c` no longer exist; step 7 **is** RED-first per the table; the regression step is 10; steps 8 and 9 are unclassified |
| 583 | *"the decision type deferred to **step 5b**"* | deleted step |
| 593–594 | *"An earlier fold introduced a prose 'step 2' that is not the table's step 2 — **the table's is the `fd.rs` mask split**"* | **false as of this fold** — the table's step 2 is the move; `fd.rs` is step 3 |
| 598 | *"**Step 1** moves `stdout_world_readable_mode` *with* its `& 0o044` … **step 2** then splits it"* | 2 moves, 3 splits |
| 601–603 | *"An earlier draft ordered **step 1** to 'move with no behaviour change' and **step 2** to hold 'no policy assertion'"* | off by one |
| 606–607 | *"**Step 1** is not a refactor to skip. It is the step that proves the closure is really 11 and not more."* | 2 |
| 660 | condition 5: *"**Step 4** therefore carries a POSITIVE test: run the emitted recipe under an interactive shell"* | 5 |

**The concrete failures — three, and none of them is cosmetic.**

1. **A mandated safety assertion is now scheduled nowhere at the moment it
   protects.** Lines 526–533 exist because *"all 12 tests in
   `crates/me-cli/tests/world_readable_output.rs` redirect to files, so none of
   them reaches"* the terminal arm — so the move must carry a pty pin or it
   proves nothing about the terminal path. The fold moved the pty assertion from
   old step 1 to new step **4**, two steps after the move, where it is a RED-first
   test for *new* behaviour (must **not** say BEARER) and therefore cannot double
   as a pin on pre-move behaviour. The prose still asserts step 1 carries it.
   Step 1 moves nothing; step 2 has no assertion of any kind.
2. **The ordering rationale is off by one at exactly the funds-adjacent step.**
   Lines 597–604 explain the order as *"step 1 moves … with its `& 0o044` so
   nothing changes; step 2 then splits it"*, and state the reason: *"at no point
   does a masked function sit inside the crate."* An implementer reconciling that
   with the table can reasonably do the split at step 2 — which is C1's named
   failure, *"publish `me`'s mask as the crate's mechanism at the irreversible
   step"* (§1 lines 76–82).
3. **The tie-break rule can no longer be applied.** §4's own conflict rule
   (*"the TABLE IS THE ONLY ORDERING OF RECORD … Prose must not number steps"*)
   is stated in the paragraph whose own example is now wrong about the table.
   Twelve of these thirteen sites are prose numbering steps — the thing that rule
   forbids — and several are not ordering at all but *requirements* (enumerate
   callers; enumerate types and constants; implement `-`; carry a pty pin; which
   steps are RED-first).

**What closes it.** Renumber the thirteen sites against the new table, delete
`5b`/`5c`, and move each requirement onto the step that now owns it: the caller
and type/constant enumerations and the pty pin onto **step 2**, `-` and §6f onto
**step 9**, the `Class`-trap gate onto **step 10**, condition 5's positive test
onto **step 5**. Fix line 593–594's example or delete the sentence; a rule whose
worked example is false trains a reader to skip the rule.

---

### C-2. The moving set still reads as "the 11", with a split `read_records`, everywhere outside §4's table — round 2's C-2, half closed.

**Site:** §1 line 96; §2.4 lines 169–174; §3 line 416; §3 lines 469–470; §4 line
607.

**What is wrong.** Round 2's C-2 named three remedies. One was done — §1's
*"second split"* paragraph is retracted, and correctly (lines 50–56). The other
two were not:

- **§3's file map, line 416**, the table whose own header says *"without this
  table a reader cannot tell where a function lands"*:

  > `| **stays in `me`** | `is_secret`, `is_bearer`, `is_argv_forbidden`, and `read_records`'s class-keyed arm (N-C2) |`

  Naming only the **arm** as staying is the split reading, stated in the plan's
  authority on file placement. §4 step 2 says the opposite: *"`read_records` …
  STAY."*
- **§2.4 is untouched** and still says of `read_records`: *"**This is where the
  seam actually pays.**"* Round 2 asked for one line assigning that seam to P1.

And the retracted **quantity** survives with it:

- §1 line 96: *"Nothing crosses a crate boundary until **those 11** are a
  library."* Under Variant B six of the eleven never become a library.
- §3 line 469–470: *"Step 1 must enumerate every type and constant **the 11**
  reference."*
- §4 line 607: *"the step that proves the closure is really **11** and not more."*

**The concrete failure — already measured, not re-derived.** §1 lines 52–56 state
it against itself: `read_records`'s three `EXIT_*` refs are at `main.rs` **1928,
2026, 2048** and the class-keyed arm runs ~**1932–2030**, so two of three fall
outside it. An implementer who takes §3's file map at its word splits
`read_records`, the moving set carries **three** `EXIT_*` refs instead of one,
Variant B collapses, and the cheapest green is `pub const EXIT_USAGE: i32 = 2` —
the publish §3 spends a page ruling out.

**What closes it.** Delete *"'s class-keyed arm"* from line 416 so the whole
function is listed as staying; add the P1 line to §2.4; and restate lines 96,
469–470 and 607 against the adopted moving set (five functions plus the
`cfg(not(unix))` stub), keeping "11" only where it describes the *closure that
was measured*, never the set that moves.

---

## IMPORTANT

### I-1. Step 2 — the move — has no test gate at all, and its one substantive clause is a measurement of the rejected move.

**Site:** §4 line 502 (the whole of step 2's gate); §4 lines 521–524.

**What is wrong.** Step 2's gate is:

> builds with **`grep -c 'pub const EXIT' main.rs` == 0** and **`grep -c 'EXIT_'
> <lib module>` == 0** — a published constant fails the step. Callers enumerated
> in BOTH directions (four live outside the closure)

Three defects, one step:

1. **No test is required to run.** Old step 1's gate was *"388 RUN, 388 passed,
   1 skipped"* **plus** the pty assertion. Splitting it into steps 1 and 2 left
   the whole regression gate on step 1 — which moves nothing — and gave the step
   that physically relocates six definitions a gate satisfied by `cargo build`.
   With §4's *"No step begins until the previous is green"*, an implementer may
   advance from step 2 to step 3 with a red suite.
2. **"(four live outside the closure)" is retracted arithmetic.** Measured here:
   `write_private` is called at `main.rs:424`, `:496`, `:620` and `:2081` — three
   outside `emit` — and `refuse_world_readable_stdout` at `:434` and `:1000` —
   one outside `refuse_write_block`. That is the four, and **both functions stay
   in `me` under Variant B**, so none of those callers crosses anything. The
   figure was measured against the rejected intact move (§4 line 521 says so:
   *"producing four `E0425`s"*) and this fold carried it into the adopted one —
   the same shape as round 2's I-1, one table over.
3. **The two things that actually break are named nowhere.** `destination`
   returns `Destination` and `write_block` returns `WriteBlock`; both enums are
   **private in `main.rs`** (`:928`, `:953`) and must move and become `pub`, with
   `refuse_write_block` — which stays — consuming `WriteBlock` across the new
   boundary. And `main.rs:2165` reads
   `use super::{destination, is_plate_artifact, write_block, Destination, WriteBlock};`
   inside `#[cfg(test)] mod tests`, so the move necessarily edits the two unit
   tests at `:2184` and `:2201`. **Old step 1's row said exactly this** —
   *"The move necessarily edits 2 of them … enumerate the diff to those two and
   justify each"* — and the fold deleted the clause without re-attaching it.

**The concrete failure.** At step 2 the implementer hits a broken `use super::`
line. With a build-only gate and the enumerate-and-justify clause gone, the
cheapest green is to delete or `#[ignore]` the two tests. One of them is
`write_block_decides_both_gates_once` (`main.rs:2201`) — the test §6 condition 6
names as *"expected to change"* and the only unit pin on the terminal decision.
Step 10's `all 388 pass` would not catch the loss, because that count is already
stale by then (M-5).

**What closes it.** Restore the regression gate and the two clauses to step 2:
*"388 pre-existing tests still pass, with the diff to the two in `main.rs`'s
`mod tests` enumerated and each edit justified"*; add the pty pin C-1 orphaned;
replace *"(four live outside the closure)"* with the enumeration that Variant B
actually needs — the `Destination` and `WriteBlock` types, and `main.rs:2165`'s
`use super::`.

### I-2. §6 condition 10 discharges nothing — all five F-265 sites stay in `me` — and its only substantive clause is violated by four §4 steps.

**Site:** §6 condition 10 (plan lines 710–714); §4 steps 5, 6, 7, 8.

**What is wrong.** Condition 10 reads *"F-265 fixed **for the moving set** at
minimum … P0 moves these functions, and a refactor over an untested distinction
is how the distinction dies."* `FOLLOWUPS.md:11754-11760` lists F-265's five
sites:

| site | in the moving set? |
| --- | --- |
| `refuse_write_block`, Terminal arm | **no** — step 2 keeps every `refuse_*` |
| `refuse_write_block`, WorldReadable arm | **no** |
| `read_records`, `--in` error | **no** — stays whole |
| `read_records`, stdin error | **no** |
| `emit`, write failure | **no** — stays |

**Zero of five.** Under Variant B the moving set holds no exit decision at all
(that is the point of step 1), so *"F-265 fixed for the moving set"* is true the
moment step 1 completes, having fixed nothing. It is a closure condition that
cannot fail.

Its second sentence — *"Every gate in §4 that asserts a refusal pins the
**digit**"* — is the substantive half, and **§4 does not obey it**: step 5
(the remedy refusal), step 6 (*"asserted end-to-end in the donor"*), step 7
(*"`--nosuchflag`… does not echo the secret"*) and step 8 (four `--expect`
refusals) all assert refusals without naming an exit code. Step 4 pins its digit;
step 9's differential pins the codes `me` produces **today**.

**The concrete failure.** `--expect`'s refusals are new codes — §3 lines 337–339
measure the false refusal at **rc=4** (`EXIT_INVALID`, `main.rs:298`) against
exit 0 for the same invocation. They are outside step 9's *"every code `me`
produces today"* differential, and step 8's gate accepts any non-zero. So P0's
newest funds-path refusal ships with its exit code pinned by nothing — F-265's
exact failure mode, reproduced inside the condition written to prevent it.

**What closes it.** Restate condition 10 against the set that actually carries
the defect (`refuse_write_block`, `read_records`, `emit` — all of which stay, so
say P0 fixes them in place), and add the digit to steps 5–8's refusal clauses,
naming rc=4 for `--expect`.

### I-3. §6 condition 9 authorises a remedy for F-264 that leaves step 5 permanently RED, and every later step is blocked behind it.

**Site:** §6 condition 9 (plan lines 704–709); §4 step 5 (line 505); §4 line 497.

**What is wrong.** Step 5's gate requires *"the emitted recipe, **RUN under a
real interactive zsh**, actually removes the entry."* Condition 9 authorises two
remedies, quoting `FOLLOWUPS.md:11737-11740` faithfully:

> P0 either fixes the recipe (`fc -W`, edit, `fc -R`) **or changes the message to
> say the shell must be exited first**. Both are honest; the present text is not.

The second remedy is honest and **cannot make step 5 green**: a recipe whose own
instruction is *"exit your shell first"* removes nothing when run under a live
interactive zsh, which is precisely what F-264 measures
(`FOLLOWUPS.md:11721-11725`). The two are presented as interchangeable and they
are not.

**The concrete failure.** §4 line 497: *"No step begins until the previous is
green."* An implementer picks the cheaper remedy — a one-sentence message edit
over `fc -W`/`fc -R` plumbing — writes the test the gate demands, watches it fail
for a reason the plan says is acceptable, and steps **6, 7, 8, 9, 10 and 11**
are stalled behind a gate that condition 9 licensed them to make unsatisfiable.
That is round 2's I-2 in a new place: a gate unsatisfiable against the text the
plan mandates.

**What closes it.** One sentence in condition 9 or step 5: if P0 takes the
message route, step 5's gate becomes *"the recipe, run **after the shell exits**,
removes the entry — and the message says so"*; if it takes the `fc -W` route the
present wording stands. Naming which route P0 takes is simpler still.

---

## MINOR

**M-1. *"The three exit constants are private in `main.rs`"* (line 543) — there
are four.** `grep -n 'const EXIT' crates/me-cli/src/main.rs` → `295` `EXIT_OK`,
`296` `EXIT_USAGE`, `297` `EXIT_REFUSED`, **`298` `EXIT_INVALID`**. The code block
at line 546 shows three. `EXIT_INVALID` is the code `--expect`'s refusals return
(§3 lines 337, 344) and it appears nowhere in the plan. Step 2's
`grep -c 'pub const EXIT'` covers it, so nothing executable breaks — but the plan
leans on exhaustive enumeration and this one is short by one.

**M-2. Round 2's M-2, unfixed.** §3 line 209 still says `EXIT_REFUSED` is *"used
four times in its source"*. Measured: `main.rs:407`, `:599`, `:2026` — **three**.

**M-3. Round 2's M-3, unfixed.** §4 line 509 (now step 9) still gates *"`--out`
overwrites"* on a step whose `channel.rs` holds only `destination` (§3 line 409);
the overwrite lives in `write_private`, which line 418 keeps in `me`.

**M-4. Round 2's M-4, unfixed, plus C-1's residue.** No step creates
`mnemonic-io-lib`. Step 2 targets *"`me`'s lib half"* and gates on
`<lib module>` — an unnamed file; steps 3–9 name crate files; step 10 is
*"`me` consumes the crate"*. Whether steps 3–9 write into `me`'s lib half or into
the new crate decides the step-10 diff and is left to the implementer.

**M-5. Round 2's M-1, migrated to step 10.** Step 10 demands *"all 388 pass"*
after steps 4, 6, 7 and 8 have each added tests. The count is wrong by
construction. State it as *"the 388 pre-existing tests still pass, plus every
test added in steps 2–9"*.

**M-6. §6 condition 4 was not updated to match step 8.** It still reads
*"`--expect descriptor,transaction` refuses a stream missing a transaction, and
refuses an incomplete `md1` set"* — no `mt1` incomplete set, no
`--allow-unsigned-inputs` false-refusal clause, no `descriptor,cosigner`. §6 is
*"WHAT MUST BE TRUE TO CLOSE P0"*, so it now certifies less than the gate it
points at. The fold's commit message claims *"Every §6 condition now has a §4
step: 4→8"* — the mapping exists, the content does not match.

**M-7. Step 6 names a property with no observable, and does not say what clap
does with the flag afterwards.** *"the override's own parse is decided **before
`Cli::parse()`**, asserted end-to-end in the donor"* — the assertion is
constructible (whether an `ms1` appears in stderr for an argv clap would reject),
but the plan does not name it, and this is the one gate whose *whole content* is
an ordering claim. Separately: `--allow-argv-secret` is a clap field today
(`crates/me-cli/src/main.rs:252`, consumed at `:1116`, `:1127`). *"Moved off
clap"* without a matching clap declaration or an argv filter makes
`me sysw pack --allow-argv-secret …` a clap usage error — a regression in the
donor's CLI, on the flag §3 line 275 asks P0 not to regress.

---

## NIT

**N-1.** Step 1's gate is written as ``grep -c 'EXIT_' `` — trailing space, no
file and no way to scope a grep to one function; step 2's is `<lib module>`, a
placeholder. Both are prose gates dressed as commands. State the command or state
the assertion, not half of each.

**N-2.** Step 5's *"Blocked on F-264 — see §6 condition 5"* points at the
condition that **demands** the positive test; the condition that **authorises the
fix** is 9.

---

## WHAT I VERIFIED HERE

Absolute paths only; no exit code read through a pipe; nothing re-derived that
the brief listed as machine-checked.

| check | result |
| --- | --- |
| `const EXIT` in `main.rs` | `295`, `296`, `297`, **`298` (`EXIT_INVALID`)** — **four**, plan says three |
| `EXIT_REFUSED` use sites | `407`, `599`, `2026` — **three**, plan line 209 says four |
| `no_records_guard` signature / ref | `main.rs:1896`, `Result<Vec<String>, (String, i32)>`, one `EXIT_USAGE` at `:1915` — step 1 is real |
| `no_records_guard` callers | `:1930`, `:2050` — **both inside `read_records`**, which stays. No publish forced |
| `write_block` / `destination` callers | `:1200`, `:2070` / `:977` + tests — all in `main.rs` |
| `Destination`, `WriteBlock` | private enums at `:928`, `:953`; return types of two moving functions; consumed by `refuse_write_block` at `:992`, which stays |
| `main.rs`'s `mod tests` | `:2165` `use super::{destination, is_plate_artifact, write_block, Destination, WriteBlock};` — the move breaks it; 2 tests affected (`:2184`, `:2201`) |
| the "four callers outside" | `write_private` `:424`,`:496`,`:620`,`:2081`; `refuse_world_readable_stdout` `:434`,`:1000` — the four are real **and both functions stay under Variant B** |
| step 3's gate is RED today | `main.rs:912` masks `& 0o044`; `0o620 & 0o044 == 0` → `None` where the gate demands `Some(0o620)` |
| F-265's five sites | `FOLLOWUPS.md:11754-11760` — `refuse_write_block` ×2, `read_records` ×2, `emit` — **none in the moving set** |
| F-264's two remedies | `FOLLOWUPS.md:11737-11740` — condition 9 quotes them faithfully; only one satisfies step 5 |
| `grep -n 'I-3'` over the plan | **1** hit (line 330) — N-1 of round 2 closed |
| `--allow-argv-secret` | clap field `main.rs:252`, consumed `:1116`, `:1127` |
| step/condition references | 13 stale sites, tabulated in C-1 |

**Accepted as already machine-checked, per the brief:** `plan-table-check.sh`
(55 rows, 0 malformed), `plan-cite-check.sh` (13/13, 0 dangling), the
four-phrase fold-propagation sweep, `read_records`'s three `EXIT_*` refs and the
arm's extent, and the per-function `EXIT_*` counts.

## WHAT THE FOLD GOT RIGHT

Recorded so round 4 does not re-open it:

- **The table is Variant B now, and it is right.** Eleven steps, the five plus
  the stub, `read_records`/`emit`/`write_private`/every `refuse_*` named as
  staying, the signature change isolated in its own step. Two of round 2's four
  Criticals are closed by this alone.
- **The arithmetic behind Variant B holds, and I confirmed the direction round 2
  could not.** The moving set carries one `EXIT_*` ref, its callers all stay in
  `main.rs`, and steps 1+2 together publish nothing. The plan's *"ONE SIGNATURE
  CHANGE, NO NEW TYPE"* is correct.
- **Step 4 is the strongest gate in the plan.** The retracted "by construction"
  claim is gone and the replacement — a pty assertion on the emitted words,
  pinning the digit, mutation-checked in both directions — is the one gate a
  probe has already proven can fail.
- **Step 6 answers C-4 properly**, including the half round 1's N-I3 named and
  the last fold skipped: end-to-end in the donor, *"in addition, never instead"*
  of the toolkit parity test.
- **Conditions 9 and 10 exist at all.** Two P0-owned follow-ups that appeared
  nowhere in §6 now do; I-3 and I-2 above are about their scoping, not their
  presence.

---

**VERDICT: NOT GREEN — 2 Critical, 3 Important.** No code may be written against
this plan. Both Criticals are the same defect this cycle keeps producing: the
fold corrected one half of the document and left the other half asserting what
was retracted — this time the table was fixed and the prose left behind, the
exact inverse of round 2. Neither is design work; C-1 is thirteen renumberings
plus one deleted sentence, C-2 is a five-word deletion on line 416 and three
restatements of the moving set. The three Importants are one clause each.
