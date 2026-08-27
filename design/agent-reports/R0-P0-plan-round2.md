# R0 ROUND 2 — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md`

**Artifact:** `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` at `694db8a`
(worktree `review/p0-r2`).
**Object:** the 311/31 delta `09da392..HEAD`, and whether the fold that produced
it broke anything.
**Date:** 2026-08-26.

## VERDICT — **NOT GREEN**

| severity | count |
| --- | --- |
| **Critical** | **4** |
| **Important** | **5** |
| Minor | 4 |
| Nit | 2 |

**The single most important finding is C-1:** §4's step-1 row still commands
*"move the 11 … intact"* — the action the same page proves is impossible — while
the adopted answer (5 functions + stub, one signature change on
`no_records_guard`) exists **only in prose**, under a heading the fold itself
added that says *"§4's TABLE IS THE ONLY ORDERING OF RECORD … Prose must not
number steps. Where the sequence matters, amend the table."* The table was not
amended. An implementer who executes the table executes the variant the plan
documents as ending in `pub const EXIT_USAGE: i32 = 2`.

Every Critical below is a **fold-propagation** defect: the fold wrote the
correction into the narrative and left the executable half — §4's table, §3's
enumerations — carrying the retracted claim. That is six for six on this cycle's
folds.

---

## CRITICAL

### C-1. §4's step-1 row still specifies the move the plan proves cannot be done, and the adopted answer is prose-only.

**Site:** §4, table row 1 (plan line 496); the adopted-answer prose at lines
537–580.

**What is wrong.** The row reads:

> `| 1 | move the 11 into `me`'s lib half **intact — including the mask**, no behaviour change | …`

Forty lines below, the same section states:

> **STEP 1 AS WRITTEN CANNOT BE DONE, and only executing it revealed why (probe
> C-1).** … step 1's *"intact, no behaviour change"* forbids the signature change
> that would avoid it. … **No ordering avoids it. It is a language rule.**

and then adopts **variant B** — move the **five** functions §3 assigns to the
crate plus the stub, with **one** signature change,
`no_records_guard -> Result<Vec<String>, String>`.

The table never learned any of it: not the moving set (still "the 11"), not the
signature change, not the deferral of the decision type to step 5b, not the
destination file inside `me`'s lib half.

**The concrete failure.** An implementer works the table — this plan's own rule
says the table is the only ordering of record — moves 11 functions intact, and
meets `EXIT_OK`/`EXIT_USAGE`/`EXIT_REFUSED` as private `main.rs` consts
(`crates/me-cli/src/main.rs:295-297`). The cheapest green is
`pub const EXIT_USAGE: i32 = 2` on `mnemonic_engrave`'s public API — the exact
outcome §3 spends a page ruling out, and which step 7's "tests still pass" gate
carries forward silently.

**What closes it.** Rewrite row 1 to state the adopted step: the five functions
plus the `cfg(not(unix))` stub, the `no_records_guard` signature change, the
target file, and the gate. Move the three-attempt table below the row as
*rationale*, not as instruction.

---

### C-2. `read_records`'s status contradicts itself across §1, §3 and §4 — and on the §1/§3 reading, the "one signature change" answer fails.

**Site:** §1 (lines 50–54, "**`read_records` IS THE SECOND SPLIT**", and
"**TWO OF THE 11 ARE SPLITS, NOT MOVES**"); §3's file map row *"stays in `me`:
… and `read_records`'s class-keyed arm"* (line 412); §4's per-function table
(lines 566–572), which lists `read_records` whole under *"§3 keeps in `me`"* with
**3** refs.

**What is wrong.** The adopted variant B keeps `read_records` **entirely** in
`me` — `PROBE-P0-fourthsplit.md:332,337`: *"It keeps `read_records`'s class arm,
every `refuse_*`, `emit` and `write_private` in `me`"* and *"`no_records_guard`'s
only two callers are inside `read_records`, **which stays in `me`**."* §1 and §3
still say the opposite: that the class-keyed arm stays and *"calling it a move
understates the work by more than half"*. §2.4 (untouched by the fold, and
falsified by it) still calls `read_records` *"where the seam actually pays"*.

**The concrete failure, measured.** `read_records` spans
`crates/me-cli/src/main.rs:1921-2052`; the class-keyed argv block is
**1932–2030**. Its three `EXIT_*` references are at **1928** (the `--in` read
error), **2026** (inside the argv block) and **2048** (the stdin read error). So
**two of the three sit OUTSIDE the arm §3 keeps in `me`** — in exactly the IO
half §1/§2.4 direct an implementer to move. Split it as §1 says, and the moving
set holds **three** `EXIT_USAGE` refs, not one; the plan's stated remedy
("ONE SIGNATURE CHANGE, NO NEW TYPE") does not cover it, and C-1's wall returns.

The §4 table's own column header is the tell: it labels `read_records` as what
*"§3 keeps in `me`"*, which is true of the whole function only under a reading
§3 does not state.

**What closes it.** Pick one. If variant B is adopted (it is), retract §1's
"second split" paragraph and the "TWO OF THE 11 ARE SPLITS" sentence, delete the
"class-keyed arm" qualifier from §3's *stays in `me`* row so the whole function
stays, and add a line to §2.4 saying the `read_records` seam is **P1's**, not
P0's.

---

### C-3. §6 condition 6 retracts step 3's gate and nothing replaces it — §4 still carries the retracted gate verbatim.

**Site:** §6 condition 6 (lines 655–685, rewritten by this fold) vs §4 table row
3 (line 498, **untouched**).

**What is wrong.** Condition 6 now says, correctly and with evidence:

> **F-259 and F-260 are caught by a TEST, not by construction — "by
> construction" was FALSE and a probe proved it.** … *A type stops a value being
> CONFUSED for another value. It cannot stop a value being IGNORED.* … **What
> actually catches it: a pty assertion on the EMITTED WORDS**, with a positive
> control, mutation-checked.

§4's step 3 gate is still:

> *a payload kind cannot be constructed from a permission bool (**F-259 cannot
> recur**)*

— the retracted claim, in the table, including the parenthetical. `grep -n 'pty'`
over the plan returns **7** hits: five belong to step 1's terminal assertion
(lines 496, 511, 525, 526, 534) and two are inside condition 6 itself (666, 676).
**No step builds condition 6's assertion.** Step 1's pty assertion is a different
thing — it pins the *digit* of the terminal refusal on the **unmodified**
binary, which is F-265's shape; F-259 is a *false word* at the same exit code,
which step 1's assertion cannot see.

**The concrete failure.** `PROBE-P0-steps3-5c.md:302-328` reproduced F-259 twice
against the strongest form of the observation types — `cargo build` clean,
`cargo clippy --all-targets` clean, **391/391 green**, and
`me sysw wipe --fill zeros` on a pty printing *"this payload is BEARER"* again.
An implementer walking §4 satisfies step 3, never writes the pty pair, and P0
closes with a §6 condition that no step discharged and a live F-259.

Secondary: as worded, step 3's gate is not writable as a RED-first test at all —
*"a kind cannot be constructed from a bool"* is a compile-failure assertion, which
needs `trybuild` or it is a tautology the moment the type exists. §4's M5
paragraph enumerates which steps are RED-first and lists 3 among them.

**What closes it.** Replace step 3's gate with condition 6's: a pty assertion on
the emitted words, plus the positive control, plus the mutation check that proves
the pair discriminates — and say in the row that the *types* are not the gate.

---

### C-4. §6 condition 8 (the pre-parser ordering) has no step that builds it, and this fold removed the only gate that touched it.

**Site:** §6 condition 8 (line 695); §4 step 5's gate (line 500, **rewritten by
this fold**); §4 step 5b (line 501).

**What is wrong.** Condition 8 requires:

> **The guard AND the override's own parse are both decided before
> `Cli::parse()`**, asserted at least in the donor (C2).

Before the fold, step 5's gate was *"a known secret-bearing flag name is refused
**before `Cli::parse()`**"* — an ordering assertion in the donor. The fold
replaced it with:

> a unit test on the crate's flag-name table plus a lockstep parity assertion
> against `mnemonic-toolkit`'s `NodeType::is_argv_secret_bearing`

Neither half asserts ordering, and the crate has no `Cli::parse()` for anything
to run before. Step 5b puts the override *"as unit tests"* explicitly. `grep -n
'allow-argv-secret\|allow_argv_secret'` over the plan returns **one** hit —
§3's prose at line 269 — and **no §4 step**.

This is precisely the outcome round 1's N-I3 predicted when it raised the
finding: *"The implementer writes a crate unit test against a synthetic flag
table, declares green, and P0 publishes §6d's 'primary layer' without one
end-to-end assertion in any binary."* The fold answered the finding by adopting
its named failure mode, and skipped the remedy N-I3 pointed at in the same
paragraph — the donor-testable half that already exists: `SPEC…§5a` rules `me`
ships the override as an ordinary clap flag
(`crates/me-cli/src/main.rs:252`) and that **"P0 owns it."**

**The concrete failure.** P0 ships the argv guard, publishes `0.1.0` at step 8,
and no binary anywhere asserts that either layer runs before clap — the property
`mt`'s source records as the reason its own guard is correct, and the one whose
absence echoed a bearer transaction to stderr.

**What closes it.** Add the override's pre-parser move to §4 as a step of its
own (it is spec-assigned P0 work that no step currently performs), with a donor
end-to-end gate: `me sysw pack --allow-argv-secret <ms1…>` decides the override
without `Cli::parse()` having run.

---

## IMPORTANT

### I-1. §3's I5 paragraph says the three `EXIT_*` constants "actually cross" — the adopted variant says none do, and the paragraph points the implementer at the forbidden publish.

**Site:** §3, lines 457–470 (rewritten by this fold).

> omitting **all seven symbols that actually cross**: `EXIT_OK`, `EXIT_USAGE`,
> `EXIT_REFUSED`, `WriteBlock`, `Destination`, and the two enums travelling with
> them.

That list was measured against the **rejected** 464-line intact move. Under
variant B the crate-assigned set is `destination`, `stdout_world_readable_mode`
(+stub), `split_record_stream`, `no_records_guard`, `write_block`, whose
`EXIT_*` counts are **0, 0, 0, 1, 0** — and the single one is removed by the
signature change. So *"EXIT_OK, EXIT_USAGE, EXIT_REFUSED cross"* is false for the
move the plan adopts, and an implementer discharging step 1's own instruction
("confirm each is either moved or reachable") concludes they must publish them.

Two smaller defects ride along, both already reported and unfolded
(`PROBE-P0-fourthsplit.md:468` M-2): *"and the two enums travelling with them"*
double-counts `WriteBlock`/`Destination`, and the list is four symbols short of
probe 1's measured 11 (`sysw::classify`, `sysw::record::Class`,
`sysw::record::TX_PREFIX`, `sysw::wire::REGION_ADDR`, `REGION_LEN`).

**What closes it.** Re-state the crossing set for variant B's five functions, or
delete the enumeration and leave step 1's instruction to produce it.

### I-2. Step 4's gate is unsatisfiable against the donor text the plan mandates, and the fix (F-264) is P0-owned but unscheduled.

**Site:** §4 step 4 (line 499, rewritten); §3's `remedy.rs` line *"purge/remedy
text, **FROM `me` ALONE** (§6h)"*; `design/FOLLOWUPS.md:11711`.

The new gate requires *"the emitted recipe, **RUN under a real interactive
zsh**, actually removes the entry."* F-264 — filed by the probe that wrote this
very test — records that `me`'s recipe **removes nothing** when run immediately,
because zsh still holds the entry in memory and `sed -i` edits a file the secret
is not in yet. F-264's owning phase is **P0**.

So step 4 goes RED against the text `remedy.rs` is told to take verbatim, and the
only green is to **change `me`'s shipped security message** (F-264's own remedy:
`fc -W` before `sed -i`, then `fc -R`). No step schedules that; no sentence
authorises it; §6 condition 1 pins the donor's tests. Since every later step is
gated on step 4 (*"No step begins until the previous is green"*), this stalls
5, 5b, 5c, 6, 7 and 8.

**What closes it.** Say in step 4 that P0 fixes F-264 and that the recipe text
changes, or scope the gate to the assertion the donor can pass today and move
F-264's fix to a named step. Either is fine; silence is not.

### I-3. §6's nine closure conditions omit both follow-ups this cycle filed against P0.

**Site:** §6; `design/FOLLOWUPS.md:11711` (F-264, owning phase **P0**) and
`:11747` (F-265, owning phase **P0**).

The plan mentions F-264 once (line 251) and F-265 once (line 513), both as
narrative asides. Neither appears in *"WHAT MUST BE TRUE TO CLOSE P0"*. Under the
constellation's per-phase burndown rule a P0-owned item is **not deferrable past
P0**, so as written the plan can close green with two of its own phase's items
open. F-265 is arguably discharged by step 6's differential matrix — but the plan
does not say so, which is the same gap in a milder form.

**What closes it.** One condition: *F-264 and F-265 closed, or explicitly
re-assigned with an owning phase, before P0 closes.*

### I-4. Step 5c's gate misses both funds-path defects the fold just added to §3.

**Site:** §4 step 5c (line 502, **untouched**) and §6 condition 4, against §3's
new paragraphs at lines 316–331 (the third walk) and 343–350 (`Admission`).

§3 now records two measured defects in `--expect`:

- `mdmk_unconfirmed` is **blind to `mt1`** (`Class::Mt`, not `Class::MdMk`), so
  the incomplete-set half needs `sysw::mt::mt_unconfirmed`
  (`crates/me-cli/src/sysw/mt.rs:207`) — *"an implementer who follows the old
  sentence literally ships an `--expect transaction` that passes a
  half-transmitted transaction as complete."*
- omitting `Admission` makes
  `me sysw pack --allow-unsigned-inputs --expect transaction` **refuse at rc=4**
  a record the same invocation packs at exit 0 — a false refusal with a false
  message.

Step 5c's gate is still only *"`--expect descriptor,transaction` refuses a
stream with no transaction, and refuses an incomplete `md1` set"*. Both new
defects pass that gate. Condition 4 mirrors it word for word.

**What closes it.** Two clauses in step 5c: an **incomplete `mt1` set** must be
refused, and `--allow-unsigned-inputs --expect transaction` must **pack**, not
refuse. Both are RED today and both are one command.

### I-5. Step 6's reorder answers a different finding than the one it cites, and step 5b still needs the type step 6 creates.

**Site:** §4 step 6 (line 503, rewritten): *"**`exit.rs` FIRST, then
`channel.rs`** (probe I-3 — the step-1 ordering shape, one file over)"*.

`PROBE-P0-step6.md:382` I-3 says: *"**`exit.rs` must precede `records.rs`**, and
the plan orders them the other way … Order: `exit.rs` → `records.rs`."*
`records.rs` is step **5**. The fold reordered `exit.rs` against `channel.rs` —
both already inside step 6 — and left `records.rs` two steps earlier, so the
finding is untouched.

Variant B dissolves half of it (`no_records_guard`'s `i32` is gone at step 1),
but not the half that matters: the fold's own text says *"the decision type
deferred to **step 5b** — the step that actually produces a second variant"*,
while §3's file map puts decision types in **`exit.rs`**, which §4 schedules at
step **6**. Step 5b therefore needs a type that does not exist until the step
after it.

**What closes it.** Either introduce `exit.rs` before step 5b in the table, or
say in step 5b that it creates the decision type in `exit.rs`. And correct the
citation — two probes have an I-3 and this row names the wrong one.

---

## MINOR

**M-1. Step 1's gate arithmetic contradicts step 1's own instruction.** The row
demands *"388 RUN, 388 passed, 1 skipped"* **and** *"PLUS a pty assertion pinning
the terminal arm"* — after which the suite runs **389**. State the gate as
"388 pre-existing tests still pass, plus the new pty assertion".

**M-2. "used four times in its source" is off by one.** §3 line 204 says
`EXIT_REFUSED` is *"used four times"*. Measured: `crates/me-cli/src/main.rs:407`,
`:599`, `:2026` — **three** use sites, plus the declaration at `:297`. The three
refusals the sentence names also do not map one-to-one onto them (seed-on-argv
and `tx:` share site 2026; the `--seal-secret` refusal at 599 is unnamed).

**M-3. Step 6's `--out` clause gates code step 6 does not build** — unfolded
`PROBE-P0-step6.md:364` I-1. §3's file map gives `channel.rs` exactly one
function, `destination`; the `--out` overwrite lives in `write_private`, which
the same table keeps in `me`. As tabled, `channel.rs` is a nine-line file and
*"`--out` overwrites"* is a gate on the donor's code.

**M-4. The plan never says when `mnemonic-io-lib` is created.** Step 1 moves into
*"`me`'s lib half"* (no file named); steps 2–6 name §3's **crate** files; step 7
is *"`me` consumes the crate"*. Whether steps 2–6 write into the crate or into
`me` is left to the implementer, and the two readings produce different step-7
diffs.

---

## NIT

**N-1.** *"probe I-3"* is cited at §3 line 327 (the `passphrase` row,
`PROBE-P0-steps3-5c.md`) and at §4 line 503 (the ordering,
`PROBE-P0-step6.md`) for two different findings. Name the report.

**N-2.** §1's *"80 of its 131 lines, 61%"* does not reproduce: `read_records` is
`main.rs:1921-2052` (**132** lines) and the argv block is `1932-2030` (**99**
lines, 75%). Harmless if C-2 is closed by deleting the paragraph.

---

## WHAT I VERIFIED, AND WHAT I DID NOT RE-DERIVE

**Re-run here (absolute paths only; no exit code read through a pipe):**

| check | result |
| --- | --- |
| `EXIT_*` definitions | `main.rs:295,296,297` — private consts, as the plan states |
| `EXIT_REFUSED` use sites | `407`, `599`, `2026` — **3**, plan says 4 |
| `read_records` extent / argv block | `1921-2052` / `1932-2030` |
| `read_records`'s 3 `EXIT_*` refs | `1928` (`--in`), `2026` (argv arm), `2048` (stdin) — **2 outside the arm §3 keeps** |
| `no_records_guard` today | `Result<Vec<String>, (String, i32)>`, one `EXIT_USAGE` at `1915` — the adopted signature change is real |
| `seal::record::chunk_key` | `pub(crate)`, `sysw::record::chunk_key` module-private — §3's HRP-reachability claim is accurate |
| `grep 'pty'` over the plan | 7 hits, **none** in a step that builds condition 6 |
| `grep 'allow.argv.secret'` over the plan | 1 hit, prose, **no §4 step** |
| F-264 / F-265 owning phase | `FOLLOWUPS.md:11711`, `:11747` — both **P0** |
| F-260 reassignment | `FOLLOWUPS.md:11513` — **P1**, matches §7 |
| §6f `me` row | now carries `3 = policy refusal (EXIT_REFUSED)`; step 6's differential gate agrees with it — **this one is clean** |

**Accepted as already machine-checked, per the brief:** `plan-table-check.sh`
(54 rows, 0 malformed), `plan-cite-check.sh` (13/13), the four-phrase
fold-propagation sweep, and the per-function `EXIT_*` counts.

**Not re-measured:** the probes' own experiments. My findings are about whether
the plan **states** them correctly and whether §4 **carries** them.

## WHAT THE FOLD GOT RIGHT

Recorded so a later round does not re-open it:

- **§6f ↔ step 6 is closed.** The spec's `me` row gained `3 = policy refusal`
  and step 6's gate became differential against the current binary rather than a
  match against the table. The two rulings ("no shared constant" and "codes match
  §6f") no longer collide.
- **The line-count gate is genuinely gone.** No gate anywhere in §4 or §6 cites a
  line count; the surviving figures (431, 442, 464, 160) are all evidence, and §1
  says so explicitly.
- **Variant B is the right answer and the arithmetic behind it holds.** Seven of
  eight `EXIT_*` refs do stay behind *given* that `read_records` stays whole —
  which is C-2's point, not a challenge to the choice.
- **The "inert decision type" retraction is stated correctly**, including the
  reason (attempt 2's variants both mapped to `EXIT_USAGE`) and the sharper rule
  that replaces it.
- **Step 4's `history -d` gate is fixed correctly** — *never OFFERED*, not
  *never mentioned*, which is what the donor's own test file already said.

---

**VERDICT: NOT GREEN — 4 Critical, 5 Important.** No code may be written against
this plan. The Criticals are all one shape and all cheap: the fold corrected the
narrative and left §4's table and §3's enumerations carrying the retracted
claims. Fixing them is table edits, not design work.
