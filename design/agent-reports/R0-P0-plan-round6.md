# R0 ROUND 6 — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md`

**Artifact:** `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` at `5195eaa`
(worktree `review/p0-r6`).
**Object:** (1) did the fold close round 5's 5I/7M/3N; (2) can each of the twelve
steps be executed, and can each gate FAIL; (3) do the by-NAME references still
resolve; (4) does every §6 condition have a step.
**Date:** 2026-08-27.

## VERDICT — **NOT GREEN**

| severity | count |
| --- | --- |
| **Critical** | **1** |
| **Important** | **2** |
| Minor | 8 |
| Nit | 2 |

**The Critical is a fact, not a contradiction.** §3 states that `me` does not
leak a secret on argv today, cites one invocation as verification, and builds
two step gates on that baseline. **The fact is false.** With a real `ms1`
codex32 secret, four of the six `me` surfaces I probed echo the entire secret
to stderr at exit 2, including the bare `me <ms1>` — the most natural wrong
move an operator makes, since top-level `me` *is* the NFC converter. The one
invocation the plan names is the one that does not leak. Consequence: step 6's
named observable is already green on the untouched tree, so **the gate for
§6d's pre-parser ordering — the rule that exists to stop exactly this — cannot
fail**, and §6 condition 8 is discharged by an assertion that passes with the
guard absent.

**Trajectory note.** Criticals went 4 → 2 → 2 → 0 → 1. This one is not a
regression of an earlier finding and is not internal-consistency: five rounds of
correctness lenses could not reach it, because nothing in the document is
inconsistent. It is a **measured claim nobody re-measured** — the class the
constellation rule *"5 of 22 ungated facts were false"* names, and the class
*"a negative inherits the scope of the search that produced it"* predicts.

**On the by-NAME axis, the plan is stable.** I resolved all thirteen by hand
against the current table. **No new staleness was introduced by this fold** —
the only four that do not resolve 1:1 are round 5's M-3 and M-4, unchanged and
still Minor. That half of the brief's hypothesis came back clean.

---

## QUESTION 1 — DISPOSITION OF ROUND 5's FINDINGS

| # | disposition | evidence |
| --- | --- | --- |
| **I-1** four-caller arithmetic asserted in prose, retracted in the cell | **CLOSED** | 530–537 rewritten to *"the specific 'four callers outside the closure' figure is RETRACTED"*; the heading `FOUR CALLERS LIVE OUTSIDE THE CLOSURE` no longer exists (`grep` → 0) |
| **I-2** no step edits the staying functions; condition 10 self-contradicts | **NOT CLOSED — named in the commit message, not done, second consecutive time** | **(a)** line **754** *"P0 moves these functions"* is still there, verbatim, **three lines after** the fold's own new sentence at 750–752 describing that exact sentence as a defect *"an earlier draft"* had. **(b)** No §4 row was added: `git show 5195eaa --stat` is 34 lines and adds no table row; `grep` over §4's rows for `read_records` / `` `emit` `` / `refuse_write_block` still returns **one** row (step 2, line 510) and only in the clause listing them as **STAYING** → **I-1** below |
| **I-3** condition 6 requires F-260 by test; §7 reassigns F-260 to P1 | **CLOSED** | 692 now *"**F-259** is caught by a TEST"*; new paragraph 716–719 *"F-260 is NOT part of this condition"* |
| **I-4** the digit clause is violated by five §4 gates | **PARTIAL — 1 of 5** | Step 2 (510) gained *"asserting the exit DIGIT, not `!success()`"*. Steps 5, 6, 7, 8 (513–516) are byte-identical to the pre-fold text; condition 10's *"Every gate in §4 that asserts a refusal pins the **digit**"* (755–756) is therefore still false about the plan's own table → **I-2** below |
| **I-5** `plan-stepref-check.sh` has five blind spots, not one | **CLOSED for the three that matter** | Accepted as machine-checked per the brief. Independently: the ordinal mechanism the **commit message claims was added** is still missed → **M-8** below |
| M-1 condition 9's dangling *"either"* | **NOT CLOSED** | line **741** unchanged |
| M-2 conditions 2 and 3 have no step | **NOT CLOSED** | no row added |
| M-3 three by-name references not 1:1 | **NOT CLOSED** | 549, 550, 458 unchanged |
| M-4 *"the 11"* residue | **NOT CLOSED** | 101, 478, 633 unchanged (and 541, 572) |
| M-5 the M5 paragraph partitions the table wrongly | **NOT CLOSED**, and now load-bearing | 548–556 unchanged; *"Everything else is RED-first"* now covers step 6, which cannot go RED — see **C-1** |
| M-6 the type/constant enumeration is attached to the move, not to 9b | **NOT CLOSED** | 477–481 unchanged |
| M-7 §7 claims `FOLLOWUPS.md` updated *"in this same fold"* | **NOT CLOSED** | line **780** unchanged |
| N-1 `<lib module>` placeholder in step 2's gate | **NOT CLOSED** | `grep -c '<lib module>'` → **1** |
| N-2 the script double-reports every hit | **NOT CLOSED** | the `$l[$i] . " " . $l[$i+1]` join at script line 34 is unchanged |
| N-3 `STEP ZERO IS INSIDE me` | **CLOSED** | 101, now *"THE FIRST WORK IS INSIDE `me`"* |

**Score: 3 of 5 Important closed, 1 partial (1 of 5 edits), 1 named-and-not-done.
0 of 7 Minor closed. 1 of 3 Nit closed.**

**The pattern round 5 made its headline recurs, narrowed and sharpened.** Round
4's fold skipped 9 of 12. Round 5's skipped 3 of 6 with one mis-reported as
folded. This fold skipped **9 of 15** — all seven Minors and two Nits, none of
them named in the commit message — and **again** named I-2 among the findings it
folded while leaving the offending sentence in place. **I-2 is now unactioned
across four consecutive folds** (rounds 3, 4, 5, 6) and has been *named* in two
of them. The Minors do not gate and I am not inflating them; the observation is
that the set is not shrinking, and I-2's specific failure — quoting a finding
and editing the sentence *before* the defect rather than the defect — is now a
repeated mechanism rather than an oversight.

---

## CRITICAL

### C-1. §3 asserts `me` does not leak a secret on argv. It does — on four of six surfaces, with a real `ms1`. The false baseline de-gates step 6, whose observable cannot fail, and leaves §6 condition 8 undischarged.

**Site:** §3 lines **284–287**; §4 step **6** (line 514) and step **7** (line
515); §6 condition **8** (line 735); the M5 paragraph, line 555.

**The claim, verbatim (284–287):**

> **`me` does not currently leak this way** — verified 2026-08-26,
> `me sysw pack --nosuchflag <ms1…>` exits 2 naming only the flag, with the secret
> absent from stderr. **That is the behaviour to preserve, and it is not the same
> as having the pre-parser guard**; P0 must not regress it while adding one.

**The measurement.** Binary invoked by absolute path
(`/scratch/code/shibboleth/mnemonic-engrave/target/debug/me`), stderr captured
to a file, exit code read directly — never through a pipe. Secret used is a
**real, valid codex32 `ms1`** taken from the repo's own fixtures
(`ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw`).
`SECRET-IN-STDERR` is `grep -c` for a distinctive interior substring of the
secret body.

```
RC=2  SECRET-IN-STDERR=1   me  <REAL ms1>
RC=2  SECRET-IN-STDERR=1   me bundle <REAL ms1>
RC=2  SECRET-IN-STDERR=1   me sysw wipe <REAL ms1>
RC=2  SECRET-IN-STDERR=1   me sysw show <REAL ms1>
RC=3  SECRET-IN-STDERR=0   me sysw pack <REAL ms1>
RC=2  SECRET-IN-STDERR=0   me sysw pack --nosuchflag <REAL ms1>   ← the plan's probe
```

Verbatim stderr for `me bundle <REAL ms1>`:

```
error: unexpected argument 'ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw' found

Usage: me bundle [OPTIONS]

For more information, try '--help'.
```

**Why the plan's probe is the one exception.** `me sysw pack` is the only probed
surface that *accepts* positional `[RECORDS]...`, so an `ms1` there is a valid
positional and reaches `me`'s post-parse `is_argv_forbidden` guard — exit 3, no
leak. Adding `--nosuchflag` makes clap reject the **flag**, and clap's
unknown-flag error names the flag, never the other arguments. **Every other
surface takes no positional**, so an `ms1` is an *unexpected argument* and clap
echoes it verbatim — which is precisely the shape §3 itself records for `mt`
six lines earlier: *"clap rejected the unexpected positional first — and clap's
error echoed the entire bearer transaction to stderr."* The plan describes the
mechanism, cites it as `mt`'s, and then measures `me` on the one invocation that
cannot exhibit it.

Structural confirmation that this is not a stale binary: `grep -c 'env::args'
crates/me-cli/src/main.rs` → **0** in the current tree. Nothing runs before
`Cli::parse()`, so clap is unconditionally first on every surface.

**Corroboration that the class was already known and mis-scoped.** The repo
carries a family of *"does not leak the secret to stderr"* tests —
`bundle_msx1_mangled_hrp_does_not_leak_secret_body` (`crates/me-cli/tests/cli.rs:153`),
`bundle_corrupted_mk1_does_not_leak_full_string` (`:177`) — and **every one of
them feeds the material with `write_stdin(...)`**. The input-content path is
hardened and tested; the **argv** path, which is the one §6d exists for, is
tested nowhere and leaks.

**The concrete failure, in three parts.**

1. **The gate cannot fail.** Step 6's cell (514) states its gate as *"the
   observable is that no `ms1` appears in stderr for an argv clap would otherwise
   reject — that is what pre-parser ordering means from outside, and it is the
   only gate here whose whole content is an ordering claim."* On the invocation
   the plan names, twice (284, and again inside step 7's cell at 515), that
   observable is **green on the untouched tree**. Under §4's rule *"Each step is
   RED first"* and the M5 paragraph's *"Everything else is RED-first"* (555), an
   implementer is told the step is RED-first and simultaneously told its
   observable is already true. The cheapest reading — and the one §3 endorses —
   is to write it as a preservation check. **A preservation check on a property
   that already holds is round-4 I-1's finding verbatim** (*"`cargo build` + two
   greps are already green on the untouched tree, so they cannot fail"*), which
   this plan corrected in one place and re-created in another.

2. **It is worse than already-green: for the layer step 6 implements, it is
   structurally unreachable.** Step 6 is **layer 1 — FLAG-NAME** matching
   (§3, 292–295). `me sysw pack --help` declares no secret-bearing flag at all:
   records arrive as positionals, and `--passphrase-ask` documents *"Never argv
   and never an environment variable."* So there is no argv in which a **known
   secret-bearing flag name** is present *and* clap would leak — clap's
   unknown-flag error names only the flag. Every leaking case above is an
   **unexpected positional**, which is **layer 2 = step 7**. The ordering claim
   is therefore attached to the layer that cannot demonstrate it, and step 7 —
   the layer that *can* — asserts only *"`me sysw pack --nosuchflag <ms1…>` still
   does not echo the secret"*, a preservation on the single probe that never
   leaked. **Both argv gates point away from the four invocations that leak.**

3. **§6 condition 8 does not close.** *"The guard AND the override's own parse
   are both decided before `Cli::parse()`, asserted at least in the donor (C2)"*
   (735). The only assertion in the donor is step 6's, and it passes with the
   guard absent. An implementer can ship P0 with the flag-name recogniser and its
   toolkit parity test — both of which test the *list*, not the *ordering* — walk
   every gate green, and have asserted nothing about the property the condition
   names. The tool would still echo the operator's codex32 secret on
   `me <ms1>`.

**What closes it.**

- **Replace the false fact at 284–287** with the measurement above: `me`
  **does** leak on argv today, on every surface that accepts no positional, and
  the `sysw pack --nosuchflag` result is an artefact of that surface having a
  post-parse guard and clap naming the flag rather than the value. State the
  probe table, not a single invocation.
- **Restate P0's obligation.** It is not *"must not regress it while adding
  one"*; it is **fix**. The pre-parser guard is what removes a live leak, which
  strengthens P0's case rather than weakening it.
- **Re-point the gates at a leaking invocation and pin the digit.** For the
  ordering claim, the RED-today assertion is: `me bundle <REAL ms1>` (or bare
  `me <REAL ms1>`) must not contain the secret in stderr **and must exit 3, not
  2** — the digit is the entire discriminant between *"the guard fired
  pre-parser"* and *"clap rejected first"*, and without it the assertion passes
  in both worlds. Move it to whichever row owns the value-shape layer, or say
  explicitly that the flag-name layer's proof in `me` is the toolkit parity test
  alone and that the ordering is proven at the value-shape layer.
- **File the leak as a follow-up against `me`** with owning phase **P0** — it is
  a shipped defect in the donor, in the same family as F-259/F-260, and P0 is
  the phase that touches this code. Per the Rust-primary rule, check `mt` and
  the other four binaries for the same shape before fixing.

---

## IMPORTANT

### I-1. Condition 10 still cannot close — four of F-265's five sites have no step — and the sentence round 4 and round 5 both asked to be deleted is still there, now three lines below a new sentence describing it as historical.

**Site:** §6 condition 10, lines **748–756**; §4's twelve rows (509–520).

**(a) The self-contradiction is not merely unfixed; the fold made the plan
assert a falsehood about itself.** Lines 748–754, verbatim:

> 10. **F-265 fixed at ALL FIVE SITES, with a step that does it.** All five stay in
>    `me` — `refuse_write_block` ×2, `read_records` ×2, `emit` — so this is work P0
>    does **in the donor**. **An earlier draft asserted both that they stay and that
>    "P0 moves these functions" four lines apart**, while no step edited any of them,
>    so the condition could not close (round-5 I-2). Five refusals can swap exit
>    **2 for 3** with all 388 tests green, proven against the unmodified binary.
>    **P0 moves these functions**, and …

The fold rewrote the sentence *before* the defect and left the defect. The
result is strictly worse than the pre-fold text: the plan now says the
contradiction belonged to *"an earlier draft"* while reproducing it three lines
later. A reader who trusts the retraction will not look.

**(b) The substantive half is untouched, and the fold added no row.**
`git show 5195eaa --stat` → 34 lines changed in the plan, **no table row added**.
`grep` over §4's rows for `read_records` / `` `emit` `` / `refuse_write_block`
returns **one** row — step 2, line 510 — in the clause
*"`read_records`, `emit`, `write_private` and every `refuse_*` **STAY**."*

Step 2's new digit-pinning pty assertion reaches **F-265 site #1**
(`refuse_write_block`'s Terminal arm) and nothing else. **Sites 2–5 —
`refuse_write_block` WorldReadable, `read_records --in`, `read_records` stdin,
`emit` write-failure — are scheduled by no step.** The condition's own new
promise, *"with a step that does it"*, is unkept for four of the five.

**The concrete failure.** An implementer walks steps 1 → 11, closes every gate,
and has discharged one of condition 10's five sites. §6 is *"WHAT MUST BE TRUE
TO CLOSE P0"*, so P0 does not close — and the plan will have refactored across
four untested exit-code distinctions while its own text warns that *"a refactor
over an untested distinction is how the distinction dies."*

**What closes it.** Delete line **754**'s *"P0 moves these functions"* (the
clause it introduces, *"a refactor over an untested distinction is how the
distinction dies"*, stands alone). Add one row before the consume step —
*"pin the digit at F-265's remaining four sites"* — or attach each site to an
existing row **by name**. This is the same remedy round 5 wrote and round 4
before it; it has not changed size.

### I-2. Condition 10's closing sentence is false about §4's own table: two gates that explicitly assert a refusal still pin no digit.

**Site:** §6 condition 10, lines **755–756**; §4 steps **7** (515) and **8**
(516). *(Step 6's case is folded into C-1 and not counted twice.)*

Condition 10 closes: *"Every gate in §4 that asserts a refusal pins the
**digit**."* §4's own prose at 522–528 says it harder: *"So `!success()` is not
enough: §4's pty assertion must assert the exit code itself, or it misses even
the arm it is named for."*

| step | the refusal its gate asserts | digit? |
| --- | --- | --- |
| 7 | *"the argv gate **refuses** by class, with the override"* | **no** |
| 8 | *"`--expect descriptor,cosigner` **refuses** an `md1`-only payload"* and three more | **no** — `rc=4` appears at lines 341 and 349, in prose, in no gate |

Step 2 (fixed this fold) and step 4 are the only two that obey it.

**Why this is not cosmetic at step 8.** `--expect` is P0's **newest funds-path
refusal** — the §6g case where *"a refusing `mk encode` still yields exit 0 with
the cosigner card missing, and the operator believes a backup is complete when
it is not."* Step 9's differential covers *"every code `me` produces **today**"*,
which by construction cannot reach a flag that does not exist today. So the exit
code of the plan's own new funds-path refusal is pinned by **nothing**, in a
plan that spends a page establishing that an unpinned refusal code is
respellable with 388/388 green.

**What closes it.** Name `rc=4` in step 8's four refusal clauses and the refusal
digit in step 7's. Two short edits. Round 5 asked for five and one was made.

---

## MINOR

**M-1 … M-7 are round 5's M-1 … M-7, verified unchanged.** Line references
re-resolved against the current file; none is re-argued here.

- **M-1.** Condition 9's dangling *"either"* — line **741**, *"so P0 either
  fixes the recipe (`fc -W`, edit, `fc -R`) **The remedy must make the recipe
  WORK**"*. Two sentences spliced, orphaned correlative. Round-4 M-8.
- **M-2.** Conditions 2 and 3 still have no step (670, 672). Both are
  measurements dated *"as of 2026-08-26"* that no row re-runs, so they are true
  at close only because they were true when typed. Round-4 M-9.
- **M-3.** *"the signature change"* (549, 574, 620) has three referents;
  *"the crate adoption"* (550) resolves ambiguously to 9b or 10; *"the adoption
  gate"* (458) names no row. Round-4 M-10. Context disambiguates all three, which
  is why this is Minor and not the by-NAME class the brief hunted.
- **M-4.** *"the 11"* residue at **101**, **478**, **541**, **572**, **633** —
  the move relocates five functions plus the stub, so *"the step that proves the
  closure is really 11"* (633) proves nothing about the other six. Round-3.
- **M-5.** The M5 paragraph (548–556) still says two items are regression-gated
  when four are — and *"Everything else is RED-first"* now covers step 6, which
  cannot go RED. **This is the sentence that makes C-1 bite**, so fixing C-1
  should fix this one at the same time.
- **M-6.** *"The move must enumerate every type and constant … reachable WITHOUT
  an inherent impl in the crate"* (477–481) is attached to the move, where
  everything lands in `me`'s own lib half alongside `Class` and **E0116 cannot
  occur**. It has teeth only at **9b**. Backstopped by 9b's *"no `Class` in it"*,
  so it is a mis-assignment, not a hole.
- **M-7.** §7 line **780** still claims `FOLLOWUPS.md` was updated *"in this same
  fold"*; the update landed in `09da392`, two folds earlier.

**M-8. The fold's commit message claims the gate now catches ordinals. It does
not.** Message: *"Added the three other undeclared misses — spelled-out numerals,
**ordinals**, and bare row ids."* The script's own header (lines 19–20) is
honest and lists only *"prose INSIDE a table cell, spelled-out numerals, and bare
row ids"*. Planted against the current gate, exit code read directly:

```
The eighth step owns the vocabulary.        → exit 0 — MISSED
The second row is where the move happens.   → exit 0 — MISSED
CONTROL (unmodified plan)                   → exit 0
```

The artifact is not misleading — the header is right — but the commit message is,
and this is the third fold whose message asserts work it did not do. Add
`(?:first|second|…|twelfth)\s+(?:step|row)` to the regex, or strike ordinals from
the message.

---

## NIT

**N-1.** Step 2's gate still reads `` grep -c 'EXIT_' <lib module> == 0 `` —
`grep -c '<lib module>'` → **1**. A placeholder dressed as a command. Round-4 N-1.

**N-2.** `plan-stepref-check.sh` still double-reports: the probe is
`$l[$i] . " " . $l[$i+1]` (script line 34), so a hit on line *i+1* also matches
at *i*, and the count reported as *"step numbers in prose: N"* is roughly twice
the number of distinct sites. Round-5 N-2, unchanged.

---

## QUESTION 2 — CAN EVERY GATE FAIL? **Eleven of twelve. Step 6 cannot.**

| # | can its gate fail? | evidence |
| --- | --- | --- |
| 1 signature change | yes | the `EXIT_*` count inside `no_records_guard` is 1 today, must reach 0 |
| 2 the move | yes | the pty assertion's AFTER half; digit now pinned this fold |
| 3 mask split | yes | `0o620 & 0o044 == 0`, so `Some(0o620)` is RED against the masked implementation |
| 4 `observation.rs` + pty | yes | F-259 is live; the plan's own probe re-wrote the bug under a clean build and 391/391 green, and the assertion caught it |
| 5 `remedy.rs` | yes | F-264 is live; *"the emitted recipe, RUN under a real interactive zsh, actually removes the entry"* is mechanical and RED today |
| **6 layer 1** | **NO** | **C-1.** The named observable is green on the untouched tree (measured), and for the flag-name layer no leaking argv exists in `me` at all |
| 7 layer 2 | yes | the crate unit tests do not exist, so they are RED — though the end-to-end half asserts the one probe that never leaked (**C-1**) and pins no digit (**I-2**) |
| 8 `--expect` | yes | the flag does not exist. No digit (**I-2**) |
| 9 `exit.rs` + `channel.rs` | yes | *"`-` is IMPLEMENTED"* is RED — §3 measures `-` reading stdin nowhere in `me` today |
| 9b create the crate | yes | the crate does not exist; *"no `EXIT_*` and no `Class` in it"* is checkable at the one moment it can fail |
| 10 consume | yes, as regression | count no longer stale |
| 11 publish | n/a | operator-gated |

---

## QUESTION 3 — BY-NAME REFERENCES, RESOLVED BY HAND

Every italicised or bolded by-name reference in the document, resolved against
the current table. **This axis came back clean apart from round 5's three known
ambiguities; the fold introduced no new staleness.**

| name | sites | row | does the row do what the name says? |
| --- | --- | --- | --- |
| *the move* | 91, 455, 475, 477, 522, 535, 539, 541, 544, 546, 560, 574, 620, 623, 624, 627, 632 | **2** | **yes** — row 2 moves the five plus the stub |
| *the mask split* | 620, 623 | **3** | **yes** — row 3 is the `stdout_world_readable_mode` split |
| *the remedy work* | 239, 689 | **5** | **yes** — row 5 carries the RUN-it positive test |
| *the value-shape layer* | 606 | **7** | **yes** |
| *the channel/exit work* | 213 | **9** | **yes** |
| *the crate boundary* | 518 | **9b** | **yes** — self-referential, inside 9b's own cell |
| *the old observation-types gate* | 708 | **4** | **yes** |
| *the argv guard* | 94 | **6 + 7** | component name spanning both layers, not a work name — acceptable, though see **C-1** for which layer proves the ordering |
| *the four-function version* | 589 | none | a rejected **attempt**, not a row — correctly not an ordering claim |
| *the signature change* | 549, 574, 620 | **1** | **ambiguous** — three signature changes exist in the plan (**M-3**); context resolves each |
| *the crate adoption* | 550 | **10?** | **ambiguous** — 9b already says *"`me` depends on it by path"* (**M-3**) |
| *the adoption gate* | 458 | **none** | no row is named adoption (**M-3**) |
| *the step that proves the closure is really 11* | 632–633 | **2** | **no** — the move relocates six definitions (**M-4**) |

---

## QUESTION 4 — §6 CONDITIONS ↔ §4 STEPS

| condition | step that discharges it | |
| --- | --- | --- |
| 1 tests pass | 10 | ✓ |
| 2 §5b's 16 verb checks | **none** | **M-2** |
| 3 §6f `mnemonic` cell under `inspect` | **none** | **M-2** |
| 4 `--expect` refusals | 8 | ✓ content matches; no digit (**I-2**) |
| 5 §6h history + positive test | 5 | ✓ |
| 6 F-259 by test; `emit`/`write_block` signature | 4 | ✓ — the pty assertion cannot pass without threading the kind, and at step 4 both functions are still inside `me`, so the signature change is intra-crate and implied by the gate. Round 5 counted this under I-2; I do not — it is under-specified, not unscheduled |
| 7 §8 `CLOSED`-grep | n/a | process |
| 8 guard + override decided pre-parser | 6 | **NOT DISCHARGED** — step 6's assertion passes with the guard absent (**C-1**) |
| 9 F-264 | 5 | ✓ |
| 10 F-265 at all five sites | 2, and only site #1 | **NOT DISCHARGED** — four sites unscheduled (**I-1**) |
| 11 R0 0C/0I | n/a | |

**Conversely — does every step serve a condition?** Steps 3, 9 and 9b serve §3's
rulings (the mask must never sit inside the crate; no published integer; the
crate boundary) rather than a §6 condition. That is legitimate: §6 is a closure
checklist, not the plan's only source of requirements. **No orphan step.**

---

## WHAT I VERIFIED HERE

Absolute paths throughout. Exit codes read directly from `$?`, never through a
pipe. stdout and stderr captured to separate files. Nothing re-derived that the
brief listed as machine-checked.

| check | result |
| --- | --- |
| `me <REAL ms1>` | **RC=2, secret echoed to stderr** |
| `me bundle <REAL ms1>` | **RC=2, secret echoed to stderr** |
| `me sysw wipe <REAL ms1>` | **RC=2, secret echoed to stderr** |
| `me sysw show <REAL ms1>` | **RC=2, secret echoed to stderr** |
| `me sysw pack <REAL ms1>` | RC=3, no leak — the post-parse guard |
| `me sysw pack --nosuchflag <REAL ms1>` | RC=2, no leak — **the plan's only probe** |
| same six with a synthetic `ms1` | identical leak pattern; validity is irrelevant, clap echoes before classification |
| `grep -c 'env::args' crates/me-cli/src/main.rs` | **0** — nothing runs before `Cli::parse()` on any surface |
| existing leak tests' input channel | `write_stdin(...)` at `tests/cli.rs:153` and `:177` — the argv path is tested nowhere |
| `me sysw pack --help` flag list | no secret-bearing flag; records are positionals; `--passphrase-ask` says *"Never argv"* |
| `P0 moves these functions` | line **754**, still present |
| `git show 5195eaa --stat` | 34 lines in the plan, **no table row added** |
| §4 rows naming `read_records` / `` `emit` `` / `refuse_write_block` | **one** (step 2, 510), only as STAYING |
| `FOUR CALLERS LIVE OUTSIDE THE CLOSURE` | **0 hits** — I-1 genuinely closed |
| `F-259 and F-260 are caught by a TEST` | **0 hits**; 692 now reads `F-259` — I-3 closed |
| steps 5–8 cells vs pre-fold (`64d9d14`) | byte-identical — I-4 is 1 of 5 |
| ordinal probes vs `plan-stepref-check.sh` | *"The eighth step"*, *"The second row"* → **exit 0 each, MISSED**; control exit 0 |
| `grep -c '<lib module>'` | **1** |
| lines 741, 780, 548–556, 477–481, 101/478/633 | all unchanged since `64d9d14` |

**Accepted as already machine-checked, per the brief:** `plan-stepref-check.sh`
exit 0 and its four-direction mutation test, `plan-table-check.sh` (56 rows, 0
malformed), `plan-cite-check.sh` (17/17, 0 dangling), `fold-propagation-check.sh`
(exit 0), the four exit constants at `main.rs:295-298`, `EXIT_REFUSED` used three
times, and the moving set's single `EXIT_*` reference.

## WHAT THE FOLD GOT RIGHT

Recorded so round 7 does not re-open it:

- **The script fix is the right shape.** Stripping the `| N |` cell instead of
  exempting the line puts ~900 characters of per-cell rationale back under the
  gate, and the header now separates COVERED from NOT COVERED honestly. Widening
  the gate rather than replacing it was the correct call.
- **I-1 is cleanly closed**, and closed the right way — the bolded prose was
  rewritten to match the table's cell rather than the cell being softened to
  match the prose.
- **I-3 is closed with a positive statement**, not a deletion: the new paragraph
  says *why* F-260 is out (P0 does not touch `mt`) and what still prevents its
  recurrence, which is stronger than striking three words.
- **Naming the four gate commands in the plan's header**, with the instruction to
  run them separately from the commit, closes the condition that produced three
  rounds of ad-hoc greps.
- **Step 2's digit pin is exactly right** and is the single highest-value edit in
  the fold — it is F-265's own site #1.

---

**VERDICT: NOT GREEN — 1 Critical, 2 Important.** No code may be written against
this plan.

**The answer to the brief's two hypotheses.** *A gate that cannot fail:* found
one, step 6, and it is the gate for the pre-parser ordering rule — the plan's
own funds/secret-safety normative requirement. *A by-NAME reference gone stale:*
**none new.** Nine of the thirteen names either resolve to exactly one row that
does what the name says or are correctly not row references at all; the four
that do not — *the signature change*, *the crate adoption*, *the adoption gate*
(M-3) and *the step that proves the closure is really 11* (M-4) — are all round
5's known Minors, unchanged, and each is resolved by context. That axis has
stabilised.

**What the six rounds could not reach, and why.** C-1 is not an inconsistency —
the plan is internally coherent about the argv guard, and every round including
this one's own by-name and condition sweeps came back clean on it. It is a
**measured negative whose scope was one invocation**, restated as a property of
the whole binary, and then used to define what a step must preserve. Five
correctness rounds inherited it as a given, which is exactly how a false fact
propagates *into* review. The remedy is not another reading round: it is to
re-run the probe, on more than one surface, and write the table into §3.
