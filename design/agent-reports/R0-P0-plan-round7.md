# R0 ROUND 7 — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md`

**Artifact:** `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` at master `72868b9`
(worktree `review/p0-r7`, content byte-identical to master — verified by `diff`).
**Round 6's report:** `design/agent-reports/R0-P0-plan-round6.md` (1C/2I/8M/2N).
**The folds under review:** `e40d631`, `72868b9`.
**Object:** (1) did the fold close round 6's findings, including the ones its own
commit message named; (2) can an implementer execute all twelve steps, and can
each gate fail; (3) is the plan honest about F-266 being operator-deferred;
(4) did the fold disturb the by-name references or the §6 ↔ §4 mapping.
**Date:** 2026-08-27.

## VERDICT — **NOT GREEN**

| severity | count |
| --- | --- |
| **Critical** | **1** |
| **Important** | **2** |
| Minor | 8 |
| Nit | 2 |

**The fold closed the FACT and left the GATE.** Round 6's C-1 had three parts:
a false claim in §3, a step gate that could not fail, and §6 condition 8 being
discharged by that gate. **Part one is closed, and closed well** — §3 now
carries the measured leak table, names F-266, and says plainly that an earlier
draft asserted the opposite from a scope of one. **Parts two and three are
untouched.** The fold rewrote §3, row 7 and row 8 and **never edited row 6** —
the cell that owns the pre-parser ordering claim and the cell round 6's C-1 was
about. `git diff 5195eaa..72868b9` does not contain row 6.

Measured on today's binary, by absolute path, exit codes read from `$?`:
**six flag-name-shaped argvs carrying a real `ms1` all exit 2 with the secret
absent from stderr.** So row 6's observable is green today at the layer row 6
implements — round 6's C-1 verbatim, one round later.

**The second finding is the fold pattern the brief asked me to hunt, and it got
worse rather than smaller.** Round 6's I-1 was *"no step edits four of F-265's
five sites."* The fold did not add a step. It changed the sentence that admitted
this — *"while no step edited any of them"* — into **"the table carries a step
that edits all five."** Machine-checked against §4's own rows: **no row does.**
An honest admission was replaced by a false claim about the plan's own table,
which is strictly worse for a reader, and this is the **fifth** consecutive fold
in which I-1 has been named and not done.

**And two Minors the commit message lists as folded were not touched at all** —
condition 9's dangling *"either"* (line **767**) and §7's *"updated in this same
fold"* (line **808**), both byte-identical to `5195eaa`.

---

## QUESTION 1 — DISPOSITION OF ROUND 6's 1C/2I/8M/2N

One line of evidence each. **"Named"** = named in `e40d631` or `72868b9`'s
commit message as folded.

| # | named? | disposition | evidence |
| --- | --- | --- | --- |
| **C-1** false §3 fact | yes | **PARTIAL — 1 of 3 parts** | §3 284–313 replaced with the measured table (**closed**); row 6 (line **540**) absent from `git diff 5195eaa..72868b9`, so the gate half and condition 8 are untouched → **C-1 below** |
| **I-1** four F-265 sites unscheduled | yes (*"Fixed both"*) | **HALF DONE, and the other half now asserts a falsehood** | the *"P0 moves these functions"* sentence **is** gone (`grep` → 0) ✔; but no row was added — `awk` over rows 535–546 finds `read_records`/`refuse_write_block` only in row 2's **STAY** clause, `F-265` only in row 2 (*"site #1"*) and row 4 (the same Terminal arm) → **I-1 below** |
| **I-2** rows 7, 8 pin no digit | yes | **CLOSED for row 8, PARTIAL for row 7** | row 8 now opens *"every refusal below pins its exit DIGIT"* ✔; row 7 gained *"pinning the exit digit"* — but attached to the **unit-test** clause, not to the end-to-end *"no `ms1` in stderr across ALL surfaces"* clause round 6 named → **I-2 below** |
| **M-1** condition 9's dangling *"either"* | **yes** | **NOT DONE** | line **767** still *"so P0 either fixes the recipe (`fc -W`, edit, `fc -R`) **The remedy must make the recipe WORK**"* — the only change was joining a wrapped `` `sed\n-i` `` into `` `sed -i` `` |
| **M-2** conditions 2 and 3 have no step | yes | **CLOSED by reclassification** | both now open *"(assertion, not work — no step builds it; it is checked, not created)"*. Legitimate: they are measurements, not work |
| **M-3** three by-name refs not 1:1 | no | **NOT CLOSED** (carried) | *the signature change* ×3, *the crate adoption* ×1, *the adoption gate* ×1 all present; `git diff` touches none of those lines |
| **M-4** *"the 11"* residue | yes (*"the retracted quantity"*) | **4 of 5** | fixed at the §3 heading, 504, 567, 595; **line 659 *"really 11 and not more"* survives** — the one site round 6 quoted in full as the substantive error |
| **M-5** the M5 paragraph partitions wrongly | no | **NOT CLOSED, and now reinforced** | 574–584 byte-identical; *"Everything else is RED-first"* now covers row 6, which the fold's **own new §3** says cannot demonstrate the leak |
| **M-6** enumeration attached to the move, not 9b | no | **NOT CLOSED** (carried) | 496–508 changed only *"the 11"* → *"the moving set"* |
| **M-7** §7 *"updated in this same fold"* | **yes** | **NOT DONE** | line **808** unchanged: *"`FOLLOWUPS.md` is updated in this same fold rather than later"* |
| **M-8** ordinals missing from `plan-stepref-check.sh` | yes | **CLOSED** | accepted as machine-checked per the brief (mutation-tested five ways) |
| **N-1** `<lib module>` placeholder | yes | **REPLACED, with a new defect** | `grep -c '<lib module>'` → **0** ✔; the substitute is `crates/me-cli/src/io.rs`, a path appearing **once** in the whole plan and contradicting §3's seven-module tree → **M-4 below** |
| **N-2** script double-reports | yes | **CLOSED** | accepted as machine-checked per the brief |

**Score: 0 of 1 Critical fully closed, 1 of 2 Important closed, 2 of 8 Minor
closed, 2 of 2 Nit closed.** Two Minors were named in the commit message and not
done; one was done at 4 of 5 sites.

---

## CRITICAL

### C-1. Row 6 was never folded. Its gate is unfailable at the layer it implements — measured on six argvs — and unsatisfiable if read as covering the leaks the fold moved to row 7. Either way §6 condition 8 still closes with the guard absent.

**Site:** §4 row **6** (line **540**); §3 lines **284–313** (the fold's new text);
§4's header (line **531**); §6 condition **8** (line **761**).

**Row 6's cell is byte-identical to the pre-fold text.** `git diff
5195eaa..72868b9 -- design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` contains
rows 2, 7 and 8 and does **not** contain row 6. Its gate still reads:

> **the observable is that no `ms1` appears in stderr for an argv clap would
> otherwise reject** — that is what pre-parser ordering means from outside, and
> it is the only gate here whose whole content is an ordering claim

**The measurement.** Binary by absolute path
(`/scratch/code/shibboleth/mnemonic-engrave/target/debug/me`), stdout and stderr
to separate files, exit code from `$?`, never through a pipe. Secret is the same
real fixture `ms1` round 6 used. `leak` is `grep -c` for an interior substring of
the secret body.

Row 6 is **layer 1 — FLAG-NAME**. Every argv in which a secret rides a flag:

```
rc=2  leak=0   me --mnemonic <ms1>
rc=2  leak=0   me --mnemonic=<ms1>
rc=2  leak=0   me bundle --seed <ms1>
rc=2  leak=0   me bundle --seed=<ms1>
rc=2  leak=0   me sysw pack --passphrase <ms1>
rc=2  leak=0   me sysw pack --passphrase=<ms1>
```

Verbatim, for `me --mnemonic=<ms1>`:

```
error: unexpected argument '--mnemonic' found
```

**clap names the flag and never the value.** So for the layer row 6 implements
there is no argv in `me` where clap would leak — which is what §3 itself now
says, in the sentence the fold added: *"the leaking cases are UNEXPECTED
POSITIONALS, not declared flags … `me` declares no secret-bearing flag at all."*
**The fold wrote the reason row 6's gate cannot fail and did not edit row 6.**

**The concrete failure, both readings.**

1. **Read narrowly** (an argv carrying a secret-bearing *flag name*, which is
   row 6's layer): the observable is **green on the untouched tree** — six
   measurements above. Under §4's *"Each step is RED first"* the step has no
   RED. Round 6's C-1, unchanged.

2. **Read broadly** (any argv clap would reject, i.e. including
   `me bundle <ms1>`, which does leak at rc=2): the observable is RED today
   **and stays RED after row 6's work is done correctly**, because a flag-name
   recogniser does not match a bare positional. §4's header says *"No step
   begins until the previous is green."* **Row 6 can then never go green, and
   rows 7, 8, 9, 9b, 10 and 11 never begin.** A plan that cannot be executed as
   written.

**Internal corroboration that the fold left row 6 stale.** Row 6 asserts *"it is
the only gate here whose whole content is an ordering claim."* After the fold,
row 7 carries *"no `ms1` in stderr across ALL surfaces … which fails on today's
tree"*, which §3 defines as exactly what ordering *"means from outside."* The
"only" is now false about the plan's own table.

**§6 condition 8 therefore still does not close.** *"The guard AND the
override's own parse are both decided before `Cli::parse()`, asserted at least
in the donor."* The override's parse (`--allow-argv-secret` moved off clap) is
scheduled by row 6 alone; row 7 says only *"with the override, as unit tests."*
So the one end-to-end assertion condition 8 rests on is row 6's, and it passes
with the guard absent. Round 6's C-1 part 3, unchanged.

**What closes it.** Row 6 needs an observable that is RED today **and** green
after layer 1 alone. The discriminant that does both is the digit, which round 6
named and the fold applied to rows 7 and 8 instead: for an argv carrying a
recognised secret-bearing flag name, `me` must go **rc 2 → rc 3** with the
secret still absent from stderr — RED today (all six measurements are rc=2),
green after a flag-name recogniser runs pre-parser, and unreachable by any
post-parse arrangement. Alternatively, state explicitly that row 6's proof in
the donor is the toolkit parity test plus the `--allow-argv-secret` regression,
that the ordering is proven at row 7, and re-point condition 8 there — but then
row 6's *"the only gate here whose whole content is an ordering claim"* must go,
and I-2 below becomes load-bearing for condition 8.

---

## IMPORTANT

### I-1. Condition 10 now asserts "the table carries a step that edits all five." Machine-checked against §4's own rows: no row does. The fold replaced an accurate admission with a false self-claim, and this is the fifth consecutive fold in which the finding was named and not done.

**Site:** §6 condition 10, lines **773–784**; §4 rows **535–546**.

**The pre-fold text was honest.** `5195eaa` read: *"An earlier draft asserted
both that they stay and that 'P0 moves these functions' four lines apart, **while
no step edited any of them**, so the condition could not close (round-5 I-2)."*

**The post-fold text asserts the opposite.** Lines 773–776:

> All five stay in `me` — `refuse_write_block` ×2, `read_records` ×2, `emit` —
> so this is work P0 does **in the donor**, and **the table carries a step that
> edits all five** — an earlier draft scheduled only site #1 and left four
> unscheduled …

**Machine check.** `awk` over rows 535–546, reporting which row carries each
name:

```
read_records       -> row 2
emit               -> row 2
refuse_write_block -> row 2
F-265              -> row 2
emit               -> row 4
F-265              -> row 4
emit               -> row 5
```

Reading the three hits:

- **Row 2** names all three only in the clause *"`read_records`, `emit`,
  `write_private` and every `refuse_*` **STAY**"*, and its `F-265` reference is
  explicit: *"this is F-265's own **site #1** (`refuse_write_block`'s Terminal
  arm)"*.
- **Row 4**'s `F-265` is *"(F-265: `!success()` cannot fail here)"* on the pty
  assertion for `me sysw wipe --fill zeros` — the **same Terminal arm**, i.e.
  site #1 again.
- Rows 4 and 5's `emit` are the verb (*"must NOT emit the word BEARER"*, *"the
  emitted recipe"*), not the function.

**`grep -c WorldReadable` over rows 535–546 → 0.** So **sites 2–5** —
`refuse_write_block`'s WorldReadable arm, `read_records --in`, `read_records`
stdin, `emit`'s write-failure — are scheduled by **no row**, exactly as round 6
found, and exactly as round 5 and round 4 found before it.

**The concrete failure.** §6 is *"WHAT MUST BE TRUE TO CLOSE P0."* An implementer
executes rows 1 → 11, closes every gate, and has discharged **one** of condition
10's five sites. P0 does not close — and the plan will have refactored across
four untested exit-code distinctions while its own sentence four lines below
warns that *"a refactor over an untested distinction is how the distinction
dies."* The new sentence makes this **harder** to notice than the pre-fold text
did: a reader who trusts *"the table carries a step that edits all five"* has no
reason to check the table.

**What closes it.** Either add a row — *"pin the digit at F-265's remaining four
sites"*, placed before the consume step — or attach each of sites 2–5 to an
existing row **by name**, so `grep` over the rows finds them outside a STAY
clause. If neither is intended, then the honest edit is to restore the admission
and re-scope condition 10. What must not survive is a condition asserting
something about §4 that §4 does not contain.

### I-2. Row 7's end-to-end observable pins no digit, and the "no `ms1` in stderr" property is satisfiable by a guard on clap's ERROR PATH — which is the exact implementation §6d and condition 8 exist to forbid.

**Site:** §4 row **7** (line **541**); §6 condition **8** (line **761**).

Row 7's gate, verbatim:

> the argv gate refuses by class **pinning the exit digit**, with the override,
> **as unit tests**; **no `ms1` in stderr across ALL surfaces** — bare `me`,
> `bundle`, `sysw wipe`, `sysw show`, `sysw pack` — which fails on today's tree;
> `me sysw pack --nosuchflag <ms1…>` still does not echo the secret

The digit modifies *"refuses by class … **as unit tests**"*. A crate-level unit
test cannot pin a process exit code, so the digit lands where it is nearly inert.
The **end-to-end** clause — the one that is RED today and the one round 6's
remedy named explicitly (*"must not contain the secret in stderr **and must exit
3, not 2**"*) — carries no digit at all.

**Why this is not cosmetic.** The plan has **no structural observable for
ordering anywhere**: `grep -n 'env::args'` over the plan returns three hits, all
in §1/§3 prose describing today's tree (lines 93, 267, 295), and **none in §4**.
So *"no `ms1` in stderr"* is the sole proof of §6d.

The cheapest implementation that turns it green is to wrap `Cli::try_parse()`
and scan raw argv **on the error path** before rendering clap's message. It
produces no leak on all five surfaces, the implementer pins whatever digit it
returns, and every gate is green — while `Cli::parse()` has already run, which
is verbatim what condition 8 forbids: *"A guard that reaches its decision by
parsing first has reintroduced the leak §6d exists to stop."* The plan even
supplies the worked example: §3 records `me sysw pack <ms1>` as *"rc=3, clean"*
today, achieved entirely **post-parse**, with `grep -c 'env::args'` → **0**.

**What closes it.** Two things, and the digit alone is not sufficient. (a) Move
the digit onto the ALL-surfaces clause — it rules out the redact-clap's-error
variant, which stays at rc=2. (b) Add one observable that distinguishes *before*
`Cli::parse()` from *on its error path*; the structural forms available are a
donor test that exercises the guard as a free function over `&[String]` with
`Cli` never constructed, or a source-level assertion that the guard precedes
`Cli::parse()` in `main`. I am not prescribing which — the requirement is that
condition 8 have an observable it can fail.

---

## MINOR

**M-1. Condition 9's dangling *"either"* — named in `e40d631`'s message as
folded, not done.** Line **767** is unchanged apart from a line-wrap join:

> *"**That gate demands the recipe actually work**, so P0 either fixes the recipe
> (`fc -W`, edit, `fc -R`) **The remedy must make the recipe WORK** — flush, edit,
> reload …"*

The correlative *"either"* still has no *"or"*, and two sentences are still
spliced with no punctuation between `)` and `**The`. Round-4 M-8, round-5,
round-6 M-1.

**M-2. §7's *"updated in this same fold"* — named in `e40d631`'s message as
folded, not done.** Line **808** is byte-identical to `5195eaa`. The
`FOLLOWUPS.md` update landed in `09da392`, now three folds earlier. Round-6 M-7.

**M-3. The retracted quantity survives at the one site round 6 quoted.** Line
**659**: *"It is the step that proves the closure is really 11 and not more."*
The fold fixed the §3 heading, 504, 567 and 595 — four of five — and left the
sentence round 6 spelled out in full. The move relocates five functions plus the
stub, so it proves nothing about the other six. Round-3, round-6 M-4.

**M-4. N-1's placeholder was replaced with a path the plan contradicts, inside
mangled markdown.** Row 2's gate now reads ``grep -c 'EXIT_' `crates/me-cli/src/io.rs` == 0``
with nested backticks that will not render, and `grep -n 'io\.rs'` over the plan
returns **one** hit — this one. §3's tree names seven modules (`channel.rs`,
`fd.rs`, `observation.rs`, `records.rs`, `exit.rs`, `remedy.rs`, `lib.rs`) and
row 9b says *"move the lib-half **modules**"*, plural. If the implementer follows
§3, `crates/me-cli/src/io.rs` never exists and `grep -c` against it errors to
stderr with exit 2 and no count. An honest placeholder was replaced by a
confident wrong path.

**M-5. The M5 paragraph is unchanged and the fold made it more wrong.** Lines
**574–584** still say **two** pieces of work are regression-gated, and still say
*"Everything else is RED-first"* — which covers row 6, whose gate the fold's own
new §3 text explains cannot demonstrate the leak at the flag-name layer. Round 6
predicted this: *"This is the sentence that makes C-1 bite, so fixing C-1 should
fix this one at the same time."* Neither was fixed.

**M-6. The plan's own gate list contains a gate that passes vacuously.** The
header (lines 6–9) says *"Gates this plan is checked by — run each **separately**
from the commit"* and lists `scripts/fold-propagation-check.sh` beside three
scripts that take only the artifact. Run that way it prints
`no patterns given -- nothing to check` and **exits 0** — verified. Its own
header documents the real interface: `<artifact> <superseded-pattern>...`. Both
fold commit messages report *"fold-propagation clean"* / *"fold-propagation 0"*
with no patterns recorded, so the reported result is not distinguishable from
the vacuous one. Add *"with the superseded phrasings as patterns"* to the header,
or have the script exit non-zero when given no patterns.

**M-7. Round 6's M-3, carried unchanged.** *the signature change* (3 sites),
*the crate adoption*, *the adoption gate* still resolve ambiguously; `git diff
5195eaa..72868b9` touches none of those lines. Context resolves each, which is
why it stays Minor.

**M-8. Round 6's M-6, carried unchanged.** The *"enumerate every type and
constant"* requirement (lines 496–508) is still attached to the move, where
everything lands in `me`'s own lib half alongside `Class` and `E0116` cannot
occur; it has teeth only at 9b. The only edit was *"the 11"* → *"the moving
set"*. Backstopped by 9b's *"no `Class` in it"*, so it is a mis-assignment.

---

## NIT

**N-1. A doubled word introduced by this fold.** Lines **776–777**: *"… left four
unscheduled, while a / a sentence three lines below asserted the opposite …"*

**N-2. The blanket replace left a subject-verb disagreement it also claimed to
have swept.** Line **504**: *"The move must enumerate every type and constant
**the moving set reference**"*. `72868b9`'s message reports catching one such
artefact of the *"the 11"* → *"the moving set"* replace (*"moving the moving set
into the lib half"*); this is a second one, same replace, same class.

---

## QUESTION 2 — CAN AN IMPLEMENTER EXECUTE ALL TWELVE STEPS, AND CAN EACH GATE FAIL?

| # | executable? | can its gate fail? | evidence |
| --- | --- | --- | --- |
| 1 signature change | yes | yes | the `EXIT_*` count inside `no_records_guard` is 1 today, must reach 0 |
| 2 the move | yes | yes | the pty assertion's AFTER half, digit pinned. One grep clause names a path the plan contradicts (**M-4**) — the row's other content still fails |
| 3 mask split | yes | yes | `0o620 & 0o044 == 0`, so `Some(0o620)` is RED against a masked implementation |
| 4 `observation.rs` + pty | yes | yes | F-259 is live; the plan's own probe re-wrote the bug under 391/391 green and the assertion caught it |
| 5 `remedy.rs` | yes | yes | F-264 is live; *"RUN under a real interactive zsh, actually removes the entry"* is mechanical and RED today |
| **6 layer 1** | **NO** | **NO** | **C-1.** Green today on all six flag-name argvs I measured; RED-forever if read to cover the positional leaks, which blocks every later row under *"no step begins until the previous is green"* |
| 7 layer 2 | yes | yes, but greenable without the guard | RED today (4 of 5 surfaces leak, measured). The end-to-end clause pins no digit and is satisfiable on clap's error path → **I-2** |
| 8 `--expect` | yes | yes | the flag does not exist; digit now pinned (round-6 I-2 closed here). `rc=4` is stated twice in §3 |
| 9 `exit.rs` + `channel.rs` | yes | yes | *"`-` is IMPLEMENTED"* is RED — §3 measures `-` reading stdin nowhere in `me` today |
| 9b create the crate | yes | yes | the crate does not exist; *"no `EXIT_*` and no `Class` in it"* is checkable at the one moment it can fail |
| 10 consume | yes | yes, as regression | count is not stale |
| 11 publish | n/a | n/a | operator-gated |

**And condition 10 is unreachable by any of them** — see **I-1**.

---

## QUESTION 3 — IS THE PLAN HONEST ABOUT F-266? **YES. Clean.**

No finding here. Recorded so round 8 does not re-open it.

- **It does not depend on the leak already being fixed.** Row 7's gate says
  *"which fails on today's tree"* — and it does: 4 of 5 surfaces leak, measured.
  §3 states the leak as present tense (*"`me` **DOES** leak this way"*) with the
  probe table and the `env::args` → 0 mechanism.
- **It does not promise a fix the operator deferred.** The plan schedules no
  standalone F-266 fix. `design/FOLLOWUPS.md:11776` carries the entry as
  *"(owning phase: **P0**, gating)"*, records the ruling verbatim — *"OPERATOR
  RULING 2026-08-27: deferred, not fixed now"* — and then states the distinction
  precisely: *"It is still what condition 8 is FOR, and P0 fixes it as a side
  effect … deferred in the sense of not interrupting the cycle, not in the sense
  of unowned."* That is consistent with the constellation rule that a
  phase-owned item burns down in its owning phase.
- **The Rust-primary check was done, not assumed.** *"`mt` is NOT affected —
  checked, not assumed: its guard sits on `std::env::args()`."* Matches the
  brief.

The one caveat is not a plan defect: because P0's fix is a *side effect* of the
guard, whether F-266 actually closes depends entirely on **I-2** — if the guard
lands on clap's error path, every gate is green and the ordering the follow-up
names as the fix is absent.

---

## QUESTION 4 — BY-NAME REFERENCES AND §6 ↔ §4 MAPPING. **Undisturbed.**

**By-name.** All twelve of round 6's resolved names are still present at the same
counts, and `git diff 5195eaa..72868b9` touches **none** of those lines
(`grep -c` over the diff for the by-name sites → **0**). The four that do not
resolve 1:1 are round 5's known Minors, unchanged (**M-7**, **M-3** above). The
fold introduced no new by-name reference that points at a row: *"the moving set"*
(5 sites) names a **set of functions**, not a step, and *"the flag-name layer"* /
*"the value-shape layer"* resolve to rows 6 and 7 as §3 defines them.

**§6 ↔ §4.** Only conditions 2, 3, 9 and 10 were touched.

| condition | step | status |
| --- | --- | --- |
| 1 tests pass | 10 | ✓ |
| 2 §5b's 16 verb checks | none, **by design now** | ✓ reclassified *"assertion, not work"* — round-6 M-2 closed |
| 3 §6f `mnemonic` cell under `inspect` | none, **by design now** | ✓ same |
| 4 `--expect` refusals | 8 | ✓ and now digit-pinned |
| 5 §6h history + positive test | 5 | ✓ |
| 6 F-259 by test | 4 | ✓ |
| 7 §8 `CLOSED`-grep | n/a | process |
| 8 guard + override pre-parser | 6 | **NOT DISCHARGED — C-1, I-2** |
| 9 F-264 | 5 | ✓ (prose defect **M-1**) |
| 10 F-265 at all five sites | 2 and 4, **site #1 only** | **NOT DISCHARGED — I-1** |
| 11 R0 0C/0I | n/a | |

**No orphan step**, unchanged from round 6.

---

## WHAT I VERIFIED HERE

Absolute paths throughout. Exit codes read from `$?`, never through a pipe.
stdout and stderr to separate files. Nothing re-derived that the brief listed as
machine-checked.

| check | result |
| --- | --- |
| worktree artifact vs master `72868b9` | `diff` → **IDENTICAL** |
| `me --mnemonic <ms1>` / `--mnemonic=<ms1>` | rc=2, **leak=0** — clap names the flag |
| `me bundle --seed <ms1>` / `--seed=<ms1>` | rc=2, **leak=0** |
| `me sysw pack --passphrase <ms1>` / `=<ms1>` | rc=2, **leak=0** |
| `me <ms1>` / `bundle` / `sysw wipe` / `sysw show` | rc=2, **leak=1** — F-266 reproduced |
| `me sysw pack <ms1>` | rc=3, leak=0 — the post-parse guard |
| `grep -c 'env::args' crates/me-cli/src/main.rs` | **0** |
| row 6 in `git diff 5195eaa..72868b9` | **absent** — cell unchanged |
| `awk` rows 535–546 for F-265's five sites | `read_records`/`emit`/`refuse_write_block` → row 2 (STAY clause) only; `WorldReadable` → **0** |
| `grep -c 'P0 moves these functions'` | **0** — that half of I-1 closed |
| `grep -c '<lib module>'` | **0**; `grep -n 'io\.rs'` → **1** hit, row 2 only |
| lines 767, 808, 574–584, 659 vs `5195eaa` | `diff` → **IDENTICAL** (M5 paragraph diffed whole) |
| lines 496–508 vs `5195eaa` | changed only *"the 11"* → *"the moving set"* |
| `plan-stepref-check.sh <plan>` | exit **0**, *"step numbers in prose: 0"* |
| `plan-table-check.sh <plan>` | exit **0**, 56 rows, 0 malformed |
| `plan-cite-check.sh <plan>` | exit **0** |
| `fold-propagation-check.sh <plan>` (no patterns) | exit **0**, *"no patterns given -- nothing to check"* → **M-6** |
| `FOLLOWUPS.md` F-266 entry | present at 11776, owning phase P0, ruling verbatim, `mt` checked |

---

## WHAT THE FOLD GOT RIGHT

Recorded so round 8 does not re-open it.

- **The §3 rewrite is the right shape and the right length.** It states the leak
  in present tense, gives the five-row probe table, gives the `env::args` → 0
  mechanism, and names why the old probe was the exception. It does not
  editorialise. This is a model of how to fold a measured-fact Critical.
- **Row 8's digit pin is complete**, and it is the one that mattered most:
  `--expect` is P0's newest funds-path refusal and round 6 found its exit code
  pinned by nothing.
- **Conditions 2 and 3 were closed by reclassification rather than by inventing
  a step.** Recognising that a measurement is *checked, not created* is the
  correct answer to round 6's M-2, and cheaper than the row it asked for.
- **`72868b9` is the right instinct.** Splitting the propagation sweep into its
  own commit after discovering the previous one went in against a RED gate, and
  recording *"a global replace over prose is a refactor, and it needs the same
  reading a refactor does"*, is the durable lesson. It caught three survivors.
- **The F-266 follow-up entry is unusually good** — ruling verbatim, the
  deferred-vs-unowned distinction stated explicitly, the Rust-primary `mt` check
  performed rather than promised.

---

**VERDICT: NOT GREEN — 1 Critical, 2 Important.** No code may be written against
this plan.

**The answer to the brief's questions.** *Did the fold close round 6?* One
Critical part of three, one Important of two, two Minors of eight — and two
Minors its own commit message listed as folded were not touched. *Can an
implementer execute all twelve steps?* **Eleven.** Row 6 is unfailable at its own
layer and unsatisfiable at the other, and it is the row that owns the plan's only
normative ordering requirement. *Is the plan honest about F-266?* **Yes,
completely** — that axis came back clean. *Did the fold disturb the by-name
references or the §6 ↔ §4 mapping?* **No** — `git diff` touches none of those
lines.

**The one sentence for round 8.** Rounds 3 through 7 have each found the same
two things: a gate that cannot fail, and a finding named in a commit message but
not performed. This fold fixed the *fact* behind round 6's Critical and left the
*cell*, and it converted round 6's Important from an honest admission into a
false claim about the plan's own table. **The remedy is not another reading
round — it is to `grep` the table for what the condition claims is in it, and to
run each named observable against today's binary, before the fold is committed.**
Both are one command.
