# R0 — P1 container plan, ROUND 9 (v13: round-8 fold + archaeology eviction)

**Artifact:** `design/IMPLEMENTATION_PLAN_P1_me_container.md` @ v13 (HEAD `4189b68`)
**Scope:** `git diff 92bed3c..HEAD` — three commits: `26d3a1d` (round 8's eight
Criticals), `f9d2e8d` (its Importants and Minors), `4189b68` (the archaeology
eviction). Sections the diff did not touch are audited ONLY where the diff
falsified them, or where a round-8 finding named them and the fold did not reach
them.
**Reviewer:** independent, opus. Nothing in this report was transcribed from the
controller; every number below was produced by a command reproduced here.

---

## Commands run, and their raw output

```
$ for c in 6a3a8c8 7936306 92bed3c 26d3a1d f9d2e8d 4189b68 HEAD; do
      printf "%s  " $c; git show $c:design/IMPLEMENTATION_PLAN_P1_me_container.md | wc -l; done
6a3a8c8  2007
7936306  2079
92bed3c  2079
26d3a1d  2100
f9d2e8d  2129
4189b68  2048
HEAD     2048
```

The status header claims **"Lines 2,121 → 2039, measured."** Neither figure is a
line count of any commit in this cycle. Pre-eviction is **2129**, post-eviction
is **2048**. See [M1].

```
$ ./scripts/plan-wiring-check.sh design/IMPLEMENTATION_PLAN_P1_me_container.md
--- wiring rows: 13 live, 2 retracted
--- steps parsed: 12   vectors parsed: 25
--- live:      W1 W2 W3 W4 W5 W8 W9 W10 W11 W12 W13 W14 W15
--- retracted: W6 W7

  FINDING  E1 is a STRUCK rule (row at line 369) but line 393 references it as live
  FINDING  E6 is a STRUCK rule (row at line 374) but line 321 references it as live
  FINDING  E7 is a STRUCK rule (row at line 402) but line 1782 references it as live
  FINDING  E8 is a STRUCK rule (row at line 376) but line 114 references it as live
  FINDING  E16 is a STRUCK rule (row at line 384) but line 321 references it as live
  FINDING  E16 is a STRUCK rule (row at line 384) but line 408 references it as live
  FINDING  V16 is a STRUCK vector (row at line 1207) but line 393 references it as live
  FINDING  V17b is a STRUCK vector (row at line 1209) but line 884 references it as live
EXIT=0
```

**Rule 4b is at its 8-reference baseline. No ninth.** That half of Part A is
clean. **Rule 5 is clean too — and it is a FALSE PASS.** See [C1] and the
mutation test below.

```
$ ./scripts/plan-cite-check.sh design/IMPLEMENTATION_PLAN_P1_me_container.md | tail -1
─── citations resolved: 91 / 107 ; dangling: 16 ; ambiguous: 0

$ ./scripts/plan-cite-check.sh ... | grep DANGLING | grep -c bitcoin-0.32.9   -> 8
$ ./scripts/plan-cite-check.sh ... | grep DANGLING | grep -c mnemonic-transaction -> 8

$ ./scripts/plan-table-check.sh design/IMPLEMENTATION_PLAN_P1_me_container.md
─── table rows checked: 149 ; malformed: 0
```

§6.1's gate table states, *"measured on this fold"*: **"91 of 108 resolve; the 17
dangling are exactly the 8 into the vendored `bitcoin` crate and the 9 into
`mnemonic-transaction`"** and **"156 rows checked"**. Measured: **107 / 16 / 8**
and **149**. See [I3].

```
$ grep -nE '^\|\s*\*{0,2}(E[0-9]+)\*{0,2}\s*\|' <doc>    ->  14 live rules
  E3 E4 E5 E9 E11 E12 E13 E14 E15 E17 E18 E19 E20 E21

$ grep -nE '^\|\s*\*{0,2}(V[0-9]+[a-z]?)\*{0,2}\s*\|' <doc>  ->  25 live vectors
  V1 V2 V3 V4a V4b V5 V6 V7 V8 V10 V11 V12 V13 V15 V18 V19 V20 V21 V22 V23 V24
  V25 V26 V27 V28
```

The document says **THIRTEEN** live rules (`:390`), **TWENTY-FOUR** live vectors
(`:1575`, a close condition), **"the live set is thirteen"** (`:1449`) and
**"the numbering still runs E1–E20"** (`:1577`).

### The plan's own fold-sweep, pointed at what v12/v13 superseded

```
$ ./scripts/plan-fold-sweep.sh design/IMPLEMENTATION_PLAN_P1_me_container.md --terms \
   'THIRTEEN LIVE rules' 'TWENTY-FOUR live vectors' \
   'V6, V9, V14, V16, V17 and V17b are STRUCK' 'not V6, V9, V14, V16, V17 or V17b' \
   'a `refuse` arm against **the rule name**' 'in both cases BOTH identifiers must match' \
   'flip any expected rule name in the fixture' 'still outstanding and is the next operation' \
   'is not done here' 'The live set is thirteen' 'numbering still runs E1–E20' 'E1–E20 pass'

─── 12 superseded term(s) named by the fold author

STILL PRESENT  THIRTEEN LIVE rules                          390
STILL PRESENT  TWENTY-FOUR live vectors                     1575
STILL PRESENT  V6, V9, V14, V16, V17 and V17b are STRUCK    1449
STILL PRESENT  not V6, V9, V14, V16, V17 or V17b            1445
STILL PRESENT  a `refuse` arm against **the rule name**      835
STILL PRESENT  in both cases BOTH identifiers must match     655
STILL PRESENT  flip any expected rule name in the fixture   1604
STILL PRESENT  still outstanding and is the next operation    25
STILL PRESENT  is not done here                               32
STILL PRESENT  The live set is thirteen                     1449
STILL PRESENT  numbering still runs E1–E20                  1577
STILL PRESENT  E1–E20 pass                                   689
```

**Twelve for twelve.** Every one is a finding below. This is the same result
round 8 got with ten terms, by the same method, one round later: the `--terms`
block was extended with five of the *previous* fold's retractions and **none of
this fold's own** — E21's arrival, V6's un-striking, and the eviction. See [I5].

### Mutation test — the wiring gate's rule 5 passes on V6 vacuously

```
$ cp <doc> /tmp/.../mut.md
$ python3 -c "... replace '**(SIMPLIFICATION 3.1) V6, V9, V14, V16, V17 and V17b are STRUCK**'
                  with   '**(SIMPLIFICATION 3.1) V9, V14, V16, V17 and V17b are STRUCK**' ...
              ... and    '**not V6, V9, ...**' -> '**not V9, ...**' ..."
mutated
$ ./scripts/plan-wiring-check.sh /tmp/.../mut.md
--- steps parsed: 12   vectors parsed: 25
  FINDING  V6 is in the vector table but NO step names it
```

**The only thing making rule 5 green for V6 is the literal token `V6` inside the
two sentences that declare it STRUCK.** Correct those two stale sentences — which
is the fix — and the gate immediately reports that no step names it. The gate is
right; the document is what was hiding.

### The build gate, RUN rather than described

```
$ ./scripts/plan-build-gate.sh design/IMPLEMENTATION_PLAN_P1_me_container.md
== 2 -- extract the plan's Rust into the seal module ==
plan-build-gate: EXTRACTED NOTHING from .../IMPLEMENTATION_PLAN_P1_me_container.md
  This gate only recognises anchors naming `src/seal/*.rs` or
  `tests/seal_cli.rs`. ...
  Refusing rather than reporting a pass on an empty extraction.
EXIT=3

$ sed -n '70,74p' scripts/plan-build-gate.sh
        # only the NEW files are mechanically assemblable
        if p.startswith("src/seal/") or p == "tests/seal_cli.rs":
            cur, prepend = p, verb.lower().startswith("prepend")

$ grep -c '^```rust' design/IMPLEMENTATION_PLAN_P1_me_container.md   ->  3
$ grep -n  'plan-build-gate' design/IMPLEMENTATION_PLAN_P1_me_container.md
1704:**NORMATIVE — `plan-build-gate.sh` is NOT a close condition for P1, and the
```

The exit-3 claim the eviction kept is **true**. The mechanism claim it wrote to
replace the deleted one is **not**, and its citation is gone. See [I7].

Also read, verbatim: `design/FOLLOWUPS.md` RULING 2026-08-25 and 2026-08-25b, for
fidelity only.

---

## Part A — did the fold land, and what did it break

### Disposition of round 8's twenty findings

| # | verdict | evidence |
| --- | --- | --- |
| **C1** report-not-refuse | **PARTIAL** | §2.2's CHUNKS row, §2.3, §1.5's table row, §4 step 10, §6's W10 and §2.5a all folded. **Three sites did not**: §2.2's own next sentence `:655` (named in the finding's *Where*), §1.5's summarising sentence `:606`, §6's near-miss bullet `:1697`. → [C3], [I2], [C2] |
| **C2** W10 close condition | **FIXED** | `:1609` now reads *"V25's three conditions each PACK at exit 0 … three for V25, zero for V3"* |
| **C3** §2.5a's set-level variant | **FIXED** | `:931-932` now defer to §2.4 and say *"W12 is a REPORT"* |
| **C4** E4's TLV arithmetic | **FIXED, and correct** | `:372` is now `71 + S + 4 + body_len`, `S = 8·bit0 + 4·bit1 + (1+label_len)·bit2`. Checked against §1.1: absent = `71+0+4 = 75` ✓ (the document's own 75); all three with a 64-byte label = `71+(8+4+65)+4 = 152` ✓ (the document's own 152) |
| **C5** rule name as fixture contract | **PARTIAL** | schema `:1367`, the r6-C2 blockquote `:1387`, the `expect` sentence `:1400` and §6's W14 row `:1604` folded. **§2.4's W14 row `:835` — explicitly named in the finding's *Where* — is byte-identical to v11.** → [C4] |
| **C6** `show` marked INCOMPLETE | **FIXED** | landed in §2.4's W9 row `:829` and §6's W9 closure row `:1608`. No §4 step names it → [M7] |
| **C7** reserved flag bits | **PARTIAL / WRONGLY FIXED** | E21 and V6 were created and **neither was wired into a step**; both steps that could file V6 still declare it STRUCK → [C1] |
| **C8** V15 | **FIXED** | `:1206` re-cut to assert the orphan finding *names the expected `set_id`*, re-sited to `tests/sysw_cli.rs`, removed from step 4 and step 8 |
| **I1** two-exceptions vs E11-alone | **PARTIAL** | §1.3 `:405` fixed (*"THERE IS EXACTLY ONE"*). **§6's bullet `:1577-1579` is verbatim unchanged** → [I1] |
| **I2** step 8's rule column | **PARTIAL** | the `E1–E19 EXCEPT` range is gone; the cell's old tail survives, still asserting *"**E7 and E11** are §1.3's two NORMATIVE exceptions"* and *"the live set is thirteen"* → [I1] |
| **I3** V15 at step 8 | **FIXED** | `:1449` — *"(r8-I3) NOT V15"* |
| **I4** §3.3 vs step 4 | **PARTIAL** | both lists now read `V1–V5, V8, V10–V13, V18–V24, V26` and both say **19 plan ROWS** (counted: 19 ✓). §3.3's total *"That is **29**"* was not updated, and the map now accounts for 21 of 25 live vectors → [I4] |
| **I5** empty `--terms` block | **PARTIAL** | five terms added, all of them the *previous* fold's; this fold's own supersessions got none. Demonstrated by execution above → [I5] |
| **I6** `Tag 0x01` | **FIXED** | E14 `:382`, E15 `:383`, V21 `:1213`, V22 `:1214` all re-cut to the LABEL slot; §1.2's `u16`/65,535 corrected to `u8`/255 |
| **M1** line count | **WRONGLY FIXED** | *"measured"* figures match no commit → [M1] |
| **M2** §3.3 heading | **FIXED, now stale again** | TWENTY-NINE → TWENTY-FOUR; there are 25 → [M2] |
| **M3** V1's `n_fields = 0` | **FIXED** | `:1191` now `flags = 0x00` |
| **M4** §6's `V1–V27 pass` | **FIXED, now stale again** | → *"all TWENTY-FOUR live vectors pass"*; there are 25 → [C1] |
| **M5** TLV narrative | **PARTIAL** | `:463` and `:1651` fixed. Two other present-tense TLV sites survive → [M4] |
| **M6** report roster | **FIXED** | replaced by a glob; `ls design/agent-reports/R0-P1-plan-round*.md` resolves to round0…round8, and `R0-P1-simplification-sibling-conformance.md` exists |

---

### [C1] E21 is a LIVE REFUSAL RULE that no step builds, and its only vector is declared STRUCK by both steps that could file it

**Severity:** Critical
**Where:** `:389` (E21's row), `:1197` (V6's row), `:1445` (§4 step 4), `:1449`
(§4 step 8), `:390` (§1.3's completeness claim), `:1575` and `:1577` (§6's close
conditions), `:1417` (§3.3's filing map), `:1697` (§6's near-miss bullet).

**The failure, concretely.** The fold answered r8-C7 by creating **E21** —
*"Bits 3–7 of the flags byte MUST be zero. A record with ANY reserved bit set is
REFUSED"* — and re-cutting **V6** from *"unknown tag"* to *"`flags = 0x09` …
→ REFUSED"*. Both rows are correct. **Neither is wired to anything.**

`grep -n E21` returns exactly two hits: E21's own row, and step 8's rule column
saying *"**E20/E21 are elsewhere**"*. There is no elsewhere. No step's test-first
column names E21 or V6.

`grep -n V6` returns five hits. Two of them are the steps that would file it, and
**both still say it is struck**:

- `:1445`, §4 step 4 — *"and **not V6**, V9, V14, V16, V17 or V17b, **struck by
  3.1**"*, excluding it from `tx_record_vectors.json`;
- `:1449`, §4 step 8 — *"**(SIMPLIFICATION 3.1) V6**, V9, V14, V16, V17 and V17b
  **are STRUCK**: each tested a TLV property that no longer exists — **unknown
  tag**, duplicate tag, …"*.

So the generator never emits V6, the codec fixture never holds it, no test
asserts it, and the two sentences that mention it describe the vector it *used to
be*.

**Every close condition that could have caught this excludes it by construction.**
§1.3's completeness claim reads *"Every one of the **THIRTEEN LIVE** rules gets a
vector and a test that goes RED"* — measured, there are **fourteen**. §6 reads
*"**all TWENTY-FOUR live vectors pass**"* — measured, there are **twenty-five**.
§6 reads *"the numbering still runs **E1–E20**"* — E21 exists. §3.3's filing map
does not mention V6. §6's near-miss bullet does not carry V6's pair and still
claims the label is *"the **only** near-miss pair left in the legend"*. **P1 can
close green with E21 having no test, no vector on disk and no implementation, and
every gate will report success.**

**The wiring gate's rule 5 is a demonstrated false PASS here.** Rule 5 asks
whether a step *names* a vector; step 8's cell contains the token `V6` inside the
sentence declaring it struck, which satisfies the regex. Mutating those two
sentences to remove the stale `V6` — the correct fix — makes the gate report
*"V6 is in the vector table but NO step names it"* immediately (raw output above).
**The gate would have caught this the moment the fold corrected the strike
claims; the fold left the strike claims standing, and the gate went green.**

**Why the plan permits it.** The fold added a rule and a vector row and did not
walk forward to §4, §3.3 or §6. V6 is not struck any more, so rule 4b cannot see
the two sentences that say it is; and rule 5 is satisfied by the same two
sentences.

**Confidence:** Certain. E21's absence from every step is a `grep`; V6's double
strike-claim is quoted verbatim; the false PASS is reproduced by mutation.

**Note on V6's construction, which is sound.** `flags = 0x09` + a valid 8-byte
fee has total length `71 + 8 + 4 + body_len`, so E4 balances and E21 is the only
rule that fires. The near-miss `flags = 0x01` with the same fee is the same
length and passes. The two inputs differ in exactly bit 3, so the vector goes RED
without E21's check and green with it. **The vector is right. Nothing runs it.**

---

### [C2] §6's near-miss bullet is still a close condition requiring an incomplete set to REFUSE

**Severity:** Critical
**Where:** `:1697` — *"**V25 (an incomplete set refused / V3's complete set
passes)**"*, inside `## 6. WHAT MUST BE TRUE TO CLOSE P1`.

**The failure, concretely.** This line is byte-identical to v11
(`grep -n "an incomplete set refused"` → v11 `:1717`, HEAD `:1697`). It is the
exact statement round 8 raised as **[C2]** against §6's W10 row — at a *different*
site in the same section, which round 8 did not name.

Eighty-nine lines above it, the W10 row the fold rewrote reads *"V25's three
conditions each **PACK at exit 0** … The `SetFinding` count is the assertion:
three for V25, zero for V3."* V25's own row reads *"(RULING 2026-08-25) **all
three PACK at exit 0**"*. §2.5a.2's NORMATIVE table gives *set INCOMPLETE* the
disposition **REPORT LOUDLY, PACK**.

An implementer building §6's near-miss pairs writes a test asserting that an
incomplete set is refused. It goes RED against the ruling's implementation. To
make it green they must reinstate the refusal, at which point W10's row and V25's
row both go RED. **Two close conditions in one section demand opposite outcomes,
which is r8-C2 reproduced verbatim one round later.**

**Why the plan permits it.** The fold swept §6 for the *rows* of the W-table and
not for the *bullets* around it. The bullet contains no `E20`, no
`chunk_missing`, no `SetFinding` — nothing a condition-name sweep would hit — and
`V25` is not struck, so rule 4b is blind. `plan-fold-sweep.sh` would have caught
it, but *"V25's three negatives all refuse"* was one of the five terms §6.1
**deliberately omitted** from the `--terms` block, and the near-miss bullet's
wording (*"an incomplete set refused"*) is a different phrasing that no listed
term matches.

**Confidence:** Certain — the two lines are 89 apart in the same section, and the
line is unchanged in the diff.

---

### [C3] The CHUNKS identifier check has two postures in three places, and the report §2.2 mandates has no name it is allowed to carry

**Severity:** Critical
**Where:** `:653` (§2.2's CHUNKS row), `:655-657` (the sentence beneath it,
**unchanged**, and named in r8-C1's *Where* as `:660-668`), `:763` (§2.3's REFUSE
row), `:385` (E17), `:833` (W15), `:832` (W12), `:988-992` (§2.5a.2's five
conditions), `:606` (§1.5's summary).

**The failure, concretely — construct it.** Build a CHUNKS `tx:` record whose
carried txid binds its chunks honestly (R15/E20 pass), whose set is complete and
pristine (E19, E20 pass), which reassembles via `mt_codec::decode` into a
**valid, deserialisable Bitcoin transaction** — but a *different* transaction
from the one the carried `txid`/`wtxid` name. Feed it to `me sysw pack`. The plan
gives three answers:

1. **§2.2's CHUNKS row** (rewritten by this fold): *"… and **check** BOTH carried
   identifiers against it … **(r8-C1) EVERY failure in this chain is a
   `SetFinding`, NOT an `Err`**"*. → **packs at exit 0 with a finding.**
   §2.4's **W15** row agrees: *"checks the carried txid and wtxid against it;
   **failure yields a `SetFinding`, not an `Err`**"*. §4 step 10 agrees:
   *"`SetFinding`s on the CHUNKS form"*.
2. **§2.2's very next sentence**, two lines below the row and **unchanged by the
   fold**: *"**and in both cases BOTH identifiers must match: … MUST equal the
   carried `txid` … MUST equal the carried `wtxid` (E17).**"* → **refuse.**
   §2.3's REFUSE row lists the refusing conditions as *"magic, version, form,
   hex, `body_len`, a field, **an identifier**"*, with no form qualifier.
   **E17** — a LIVE rule, unchanged — reads *"MUST equal the carried `wtxid`
   field — **on BOTH forms**"*, and §1.5 assigns the live per-record rules
   **exit 4**. → **refuse.**
3. **§2.5a.2's NORMATIVE vocabulary is closed at five conditions** — INCOMPLETE,
   ORPHAN, DUPLICATE INDEX, `set_collision`, `not_a_transaction` — and W12 rules
   that `SetFinding` is `{ set_id, condition, detail }` **"over §2.5a.2's five
   conditions"**. **None of the five is an identifier mismatch.**
   `not_a_transaction` is explicitly *"a COMPLETE set that reassembles to
   **non-transaction bytes**"*, which this input is not. → **the report §2.2
   mandates cannot be constructed.**

So the one case is required to refuse, required to report, and forbidden from
having a report to carry. **There is no vector for it either** — V8 is the RAW
identifier mismatch, V28 is `not_a_transaction`, V15 is the set-binding negative.
It is the only branch of W15's chain with no vector, no name and no ruled
outcome.

**This is the answer to the brief's second question.** §2.3's split is
unambiguous for everything §2.5a.2 names. It is ambiguous at exactly one place:
the identifier check, which is the one condition that appears on **both** sides of
§2.3's table (*"an identifier"* in the REFUSE row, *"check BOTH carried
identifiers"* in the chain the REPORT row governs) and in neither §2.5a.2 nor the
vector table.

**Why the plan permits it.** r8-C1's *Where* named §2.2's CHUNKS row **and the
both-identifiers sentence** as one site. The fold rewrote the row and left the
sentence — the same "folded into X, not into Y" failure the finding was about,
two lines apart. And §2.3's new REFUSE row was written from the per-record
parse's field list, which legitimately contains an identifier check on the RAW
form; nothing asked what that word means on the CHUNKS form.

**Confidence:** Certain on the textual contradiction (three quotes, unchanged
lines, verified against `git show 92bed3c`). High that it is Critical: an
implementer building W15 must choose, and the choice decides whether a payload
whose transaction is not the one it claims reaches steel.

---

### [C4] §2.4's W14 row — the row an implementer builds the loader from — still makes the rule name the fixture's refusal contract

**Severity:** Critical
**Where:** `:835`. Round 8's **[C5]** named this site explicitly (`:833` at v11).
It is **byte-identical** to v11:

```
$ git show 92bed3c:<doc> | sed -n '833p' | md5sum
$ sed -n '835p' <doc> | md5sum          # same content, verified by grep -c
```

> | **W14** | `crates/me-cli/src/sysw/tx_vectors.rs` (new) + … | … the fixture
> loader and its conformance test. Loads the `expect` sum type of §3.3 and
> asserts each vector's **CODEC** outcome — a `pass` arm against the parsed
> fields, **a `refuse` arm against the rule name**. …

**The failure, concretely.** §3.3 — which the fold did correct — now rules
`expect: {"refuse": {}}`, an **empty** arm, and says so three times. §3.3 also
defers the loader's definition to §2.4: *"**W14 is the loader and its conformance
test (§2.4)**"*. §2.4's row is therefore the authoritative statement of what the
loader asserts, and it says the loader asserts a rule name that the fixture no
longer contains. An implementer writing `tx_vectors.rs` from §2.4 writes a
comparison against a field that is not in the JSON; the loader does not compile,
or it compiles against a schema §3.3 forbids.

**And the correction in §6's W14 row left its own superseded clause attached**
(`:1604`): *"… v11's row said "flip any expected rule name", **which cannot be
done once the arm is empty — the gate was unrunnable** — flip any expected rule
name in the fixture and the test goes RED."* The close condition now states two
mutations, one of which the same sentence has just declared impossible. This is
r8-I1's shape — *"the tail of the old sentence left attached to its own
correction"* — reproduced in the fold that answered it.

**Why the plan permits it.** The fold swept §3.3 (where 3.2's vocabulary lived)
and §6 (where the gate lives) and did not walk to §2.4, even though the finding
named it. §2.4's W-table carries no V-number and no E-number, so neither rule 4b
nor rule 5 looks at its prose.

**Confidence:** Certain.

---

### [I1] §6's exception bullet is verbatim unchanged, and E21 has made three of its claims false

**Severity:** Important
**Where:** `:1577-1579`, plus the tail of `:1449`.

`grep -c "the exception list below is now shorter by two"` returns **1** in v11
and **1** in HEAD. The bullet still reads:

> **Every LIVE rule has a test that goes RED without its check.** The numbering
> still runs **E1–E20** … struck seven … **leaving THIRTEEN**, and the exception
> list below is now **shorter by two**: E7 was one of the two … The remaining
> exception is **E11 alone** — **EXCEPT E7 and E11, which §1.3 names, explains
> and assigns.**

Four defects in one bullet: the numbering runs **E1–E21**; there are
**fourteen** live rules; the list went from two entries to one, i.e. shorter by
**one**; and the corrected clause still has the superseded clause welded to its
tail. §1.3 `:405` was fixed to *"THERE IS EXACTLY ONE"* and states in bold *"Every
sentence that still says 'the two exceptions' or 'EXCEPT E7 and E11' is stale"* —
naming the defect and not fixing the one instance of it. Step 8's cell `:1449`
carries the same stale claim (*"**E7 and E11** are §1.3's two NORMATIVE
exceptions"*) and *"the live set is thirteen"*.

**Confidence:** Certain.

---

### [I2] §1.5's summarising sentence still binds E1–E20, one line under the row the fold rewrote to say "NOT E20"

**Severity:** Important
**Where:** `:606`, immediately below the exit-code table.

> **So this section binds E1–E20 and §2.3 — the record-codec refusals** — and the
> other three are ruled in the row above rather than inherited.

The table row directly above now reads *"**(r8-C1) NOT E20**, and not any
set-level condition: those REPORT at **exit 0**"*. The sentence summarising that
table re-includes E20 and excludes E21. `grep -c` confirms it is unchanged from
v11. §1.5 is NORMATIVE and is the section §2.3 and §4 step 7 both defer to for
exit codes.

Same cell, a transcription defect the fold introduced: *"…which includes the rule
the same fold demoted to SHOULD **it arrives through `pack → split → Err`**…"* —
two sentences fused with no boundary.

**Confidence:** Certain.

---

### [I3] §6.1's gate baselines say "measured on this fold" and two of the four are wrong

**Severity:** Important
**Where:** `:1721` (cite-check row), `:1722` (table-check row), and `:1716-1717`
(the preamble that claims measurement).

The preamble: *"Each row states what a PASS looks like on *this* document,
**measured on this fold**, so a future run has something to diff against."*

| gate | document claims | measured now |
| --- | --- | --- |
| `plan-cite-check.sh` | 91 of **108**; **17** dangling; **9** into `mnemonic-transaction`; *"any **eighteenth** is a defect"* | 91 of **107**; **16** dangling; **8** into `mnemonic-transaction` |
| `plan-table-check.sh` | **156** rows, 0 malformed | **149** rows, 0 malformed |
| `plan-wiring-check.sh` | exactly 8 rule-4b refs | 8 ✓ |
| `plan-fold-sweep.sh` | 49 hits, all in the block | 49 ✓ |

The narrative below the table breaks the mt-codec danglings down as *"`pipeline.rs`
×6 … `header.rs` **×2** … `lib.rs` ×1"*. Measured: `pipeline.rs` ×6, `header.rs`
**×1**, `lib.rs` ×1. The eviction moved these numbers (v11 measured 95/111/16) and
they were not re-measured. The practical cost is the tripwire: *"any eighteenth is
a defect"* sits **two** above the real floor of 16, so a fold can mint a fresh
dangling citation and the stated gate will not notice.

The plan's own standing rule, quoted in its own §6.1: *"never hand-count what a
tool can count"*.

**Confidence:** Certain — reproduced by running the two gates.

---

### [I4] §3.3's filing map is the section whose job is the mapping, and it now accounts for 21 of 25 live vectors while totalling 29

**Severity:** Important
**Where:** `:1416-1425`.

The paragraph, after the fold: step 4 files **19 plan ROWS** (`V1–V5, V8,
V10–V13, V18–V24, V26` — counted, 19 ✓, counting V4a and V4b); step 9 files
**V7**; step 10 files **V27**. Then, unchanged: **"That is 29"**.

19 + 1 + 1 = **21**. The four live vectors the map does not place are **V6, V15,
V25 and V28**. V15, V25 and V28 are recoverable from §4 step 10 and from V15's own
row; **V6 is placed nowhere at all** ([C1]).

r8-I4's complaint was that §3.3 and step 4 gave different contents and different
counts. The contents were reconciled; the count was not, and the map's coverage
went *down* without the sentence that states its total moving.

`plan-table-check.sh` cannot see this (it is prose), and rule 5 cannot
(step 10 names V15/V25/V27/V28, so they are "named by a step" even though §3.3
does not file them).

**Confidence:** Certain.

---

### [I5] The `--terms` block was extended with the PREVIOUS fold's retractions and none of this one's — r8-I5, one round later

**Severity:** Important
**Where:** `:1792-1797` (the five added terms) and `:1799-1811` (the blockquote
explaining the omissions).

Five terms were added: `Tag \`0x01\``, *"follows `seal`'s posture — refuse"*,
`TWENTY-SIX rule names`, `V1–V27 pass`, `the twenty-nine`. All five are v11's
retractions, folded by commits `26d3a1d`/`f9d2e8d`. **Zero terms describe what
those commits themselves superseded** — V6's un-striking, E21's arrival, the
counts those two changed — and **zero** describe what `4189b68` evicted.

Run against twelve terms this fold actually superseded (raw output at the head of
this report): **twelve of twelve are still standing**, and every one is a finding
above. §6.1's close condition — *"exactly 49 hits, one per term, ALL of them
inside the block … A fiftieth hit anywhere else is a real finding"* — is again
satisfied trivially, because the question it was handed does not include this
fold.

The blockquote at `:1799` correctly diagnoses this for v11 (*"the gate was handed
an empty question"*) and then reproduces it. The deliberate five-term omission it
argues for is sound and is not the finding; the finding is the terms that were
never considered.

**Confidence:** Certain — demonstrated by execution, not argued.

---

### [I6] The document's deletion-discipline rule was itself deleted by the eviction, and the same commit broke it

**Severity:** Important
**Where:** deleted. It stood at v11 `:69-70`:

```
$ grep -n "landing somewhere\|repointed" <v11>
69:>    this fold is deleted without the fact landing somewhere and every citation
70:>    being repointed.
$ grep -n "landing somewhere\|repointed" <HEAD>
(no output)
```

The full sentence, from the v2 post-mortem's item 2 (r1-C2): *"Restored as
**§1.4**, and **nothing in this fold is deleted without the fact landing
somewhere and every citation being repointed.**"*

**Why this is load-bearing.** It is a standing NORMATIVE rule of the same kind as
the one the fold *did* promote (*"no rule is added here without a stated input
that would fail it"*) — and it is the rule governing the exact operation
`4189b68` performed. Round 8's Part B evict-1 named only item 4's sentence for
promotion; item 2's was not on the must-survive list, and it went with the block.

**And the eviction violated it in the same commit.** `scripts/plan-build-gate.sh:70-74`
— the citation carrying the anchor-filter fact — now appears **nowhere** in the
plan (`grep -n 'plan-build-gate'` → one hit, `:1704`, with no line reference).
The fact was deleted, its replacement is inaccurate ([I7]), and the citation was
not repointed. `plan-cite-check.sh` structurally cannot see this class: a deleted
citation is absent, not dangling.

**Confidence:** Certain on the deletion (two greps). High that it is Important:
this is the second eviction pass this cycle and there will be a third
(Part B item 5 is deferred), and the rule that would have governed it is gone.

---

### [I7] §6.1's build-gate compression replaced a cited, accurate mechanism with an uncited, inaccurate one

**Severity:** Important
**Where:** `:1704-1712`.

Deleted (v11): *"The extractor accepts a ```rust block only when a preceding
anchor names **`src/seal/*.rs` or `tests/seal_cli.rs`**
(`scripts/plan-build-gate.sh:70-74`). This plan has neither anchor, and its only
```rust block is a quoted signature from another crate."* Plus the NORMATIVE
forward statement: *"**What would have to change for it to apply**: the
extractor's anchor test at `scripts/plan-build-gate.sh:70-74` would have to
accept `src/sysw/*.rs` and `tests/sysw_cli.rs`, **and** this plan would have to
carry assemblable whole-file ```rust blocks under those anchors."*

Kept: *"The extractor accepts a ```rust block only when an anchor comment names
**a file it can assemble**, and **this plan carries no such block** — its Rust
appears as illustrative fragments inside prose."*

**Two defects, both machine-checked.**

1. *"a file it can assemble"* is not the rule. The script's test is a hard-coded
   allowlist — `if p.startswith("src/seal/") or p == "tests/seal_cli.rs"`
   (`scripts/plan-build-gate.sh:71`) — and the script's own refusal message says
   so: *"This gate only recognises anchors naming `src/seal/*.rs` or
   `tests/seal_cli.rs`."* A reader of the new sentence concludes that adding an
   assemblable `src/sysw/` block would bring this plan under the gate. **It would
   not**, and the deleted text said exactly why in a two-part condition that now
   survives nowhere.
2. *"this plan carries no such block"* — measured, `grep -c '^```rust'` returns
   **3** (`:177`, `:902`, `:1080`). The claim is defensible only under the
   reading *"no block under a recognised anchor"*, and the clause that follows it
   (*"its Rust appears as illustrative fragments inside prose"*) argues the wider,
   false reading.

Round 8 authorised evicting *"the historical output, the already-closed
false-PASS blockquote and the extractor walk-through"*. The exit-3 fact was
correctly kept and I verified it by running the gate. But this was not a
compression of history — it is a **rewrite of a live mechanism claim**, done
without opening the script, in the section whose subject is which gates read this
document. The plan legislates against this shape twice (*"never describe code
from its doc comment, its name, or an earlier agent's report"*; *"a gate that
hides its own blind spot is worse than no gate"*).

**Confidence:** Certain — the script was read at the cited lines and executed.

---

### Minors

**[M1] The status header contradicts itself, and its "measured" line counts match
no commit.** `:3-25`. It says *"the **ARCHAEOLOGY PASS has run**"* and, seventeen
lines later, *"The archaeology eviction is **still outstanding and is the next
operation**"*; `:32` adds *"Evicting it is a separate operation and **is not done
here**"*. And *"Lines 2,121 → 2039, **measured**"* — the real figures are
**2129 → 2048**. `:28` still carries r8-M1's original wrong figure
(*"LINES 2,007 → 2,068"*; v11 was 2,079). This is r8-M1 answered by adding a
second wrong measurement beside the first.

**[M2] §3.3's heading says TWENTY-FOUR; there are twenty-five.** `:1329`. r8-M2
was folded 29 → 24 and V6's un-striking made it stale in the same fold.

**[M3] §6's near-miss bullet omits V6's pair and asserts a claim V6 falsifies.**
`:1688-1692` — *"the label is the only slot that still carries a length, so it is
the **only near-miss pair left in the legend**"*. V6's row defines a second legend
near-miss pair (`flags = 0x09` refused / `flags = 0x01` passes). §3.3 says this
bullet *"enumerates the pairs and is what the entry count must satisfy"*, so the
fixture's entry-count contract omits V6 too.

**[M4] Two present-tense TLV survivors outside the struck rows.** `:340` — *"Feed
both **a TLV** whose value is `74 6f ff 21`"*, in the paragraph that justifies
E14, whose rule row the fold re-cut to *"the LABEL slot"*. `:1420` — *"That body
passes every codec rule — `magic`, `version`, `form`, **every TLV rule**, E4's
arithmetic balances"*. r8-M5 named two such sites and both were fixed; these two
were not in its list.

**[M5] §3.3 points at three reasons that no longer exist.** `:1346` — *"Not
`sysw_vectors.json`, for the **three reasons above**"* — after a blockquote the
eviction rewrote to open *"and the reason is **one line**"*. `grep -n "three
reasons"` → 1 hit; `grep -n "reason 1\|reason 2\|reason 3"` → 0.

**[M6] §3.3's r6-C2 blockquote keeps a clause 3.2 orphaned.** `:1394` — the
container assertions are *"keyed to **the same rule names**"*, referring back to
names the same blockquote has just removed from the fixture two sentences above.

**[M7] No §4 step names the `show`-marks-INCOMPLETE assertion.** r8-C6's second
MUST landed in §2.4's W9 row and §6's W9 closure row. Step 6's test-first cell
still stops at *"one line per `tx:` record, ONE line per chunk SET"*, and step 10
does not mention `show`. It is reachable (step 6's right column says *"build
W1–W5 and W9 of §2.4"*), which is why this is Minor rather than Important — but
the plan's own precedent (r5-I5) is that a closure condition no step's test-first
column names is how a gate ends up unbuilt.

**[M8] Step 4 files V20, V23 and V24 into the codec fixture that step 8 says the
layout codec cannot see.** `:1417` and `:1445` file them into
`tx_record_vectors.json`; `:1449` rules that they are *"BARE `mt1` records with
no `tx:` framing, and **the layout codec never sees one** … sited here they would
pass vacuously"*, and §4 step 6 asserts them. Pre-existing (v11 filed
`V18–V26`), but the fold re-derived both lists and re-asserted the inclusion, so
it is recorded here rather than assumed reviewed.

---

## Part B — did the eviction lose anything

**Method.** Read the deleted text in `git diff 92bed3c..HEAD`, then grep HEAD for
each fact, citation and NORMATIVE statement it carried. Spot-re-checked five of
round 8's thirteen MUST-SURVIVE items rather than re-enumerating them: E8's
struck row and its *"WHY 3.1 was taken"* reason (`:376`, present, verbatim),
§1.1a's measurement block (`:234`, present), §1.3's exception table with E11's
owner column (present), §2.5b (`:1075`, present), the r7-M1 orphan reasoning
(`:986`, present, *"202 times"*). The `--terms` block was not evicted and now
holds 49 terms. **Part B item 5 is correctly deferred and is not counted as an
omission.**

**Verdict on the five compressions.**

| eviction | claim | held? |
| --- | --- | --- |
| v1/v2 post-mortems (37 lines) | one sentence promoted, the rest live elsewhere | **NO** — see [I6] |
| §3.3's refutation (34 → 9) | keep reason 1 + its citation | **YES** |
| §6.1's build-gate history (42 → 10) | keep the NORMATIVE sentence + the reason | **NO** — see [I7] |
| the W6 paragraph (4) | fact survives in §2.4's W6 row | **YES** |
| the report roster (7 → a glob) | the glob cannot go stale | **YES** |

**Detail on the two that held that I checked hardest.**

*§3.3's three-numbered refutation.* Reason 1's citation
(`crates/me-cli/src/sysw/vectors.rs:137`) is kept verbatim, as is the
`generate() == load()` mechanism and *"a golden of the code under test"*.
Reason 2's conclusion survives as *"its schema cannot express a refusal at all
(`Vector` is the output of a successful pack)"*; the lost `pack_deterministic(…)
.expect(…)` panic mechanism is not load-bearing once the conclusion stands, and
`vectors.rs:29-40` survives in the r6-C2 blockquote. **Reason 3 — the
`assert_eq!(COVERAGE.len() as u32, 23)` fact and the "extending COVERAGE FAILS
THE BUILD" conclusion — survives in full in §2.4's W14 row and in compressed form
at §3.3 `:1360`.** Nothing load-bearing lost. The only residue is the dangling
*"three reasons above"* pointer, [M5].

*The report roster.* `ls design/agent-reports/R0-P1-plan-round*.md` resolves to
round0 … round8 (9 files) and `R0-P1-simplification-sibling-conformance.md`
exists. The marker convention — the only part a reader uses — is kept. The
per-round finding counts are lost but live in each report's own header.

**The two losses, stated once.** The eviction deleted a **NORMATIVE
deletion-discipline rule** that survives nowhere (*"nothing … is deleted without
the fact landing somewhere and every citation being repointed"*, [I6]), and it
replaced a **cited, accurate description of `plan-build-gate.sh`'s anchor filter**
with an uncited one that is wrong about the mechanism and about this plan's
```rust block count ([I7]) — the second loss being an instance of the first.
Everything else the four compressions claimed to keep, they kept.

---

## Verdict

**4C / 7I / 8M — v13 is NOT GREEN.**

The two things the brief asked me to check hard, answered:

1. **E21 and V6 are correct rows that nothing runs.** E21 is internally
   consistent with the rules around it, V6's near-miss genuinely isolates the
   reserved bit (`flags = 0x09` vs `0x01`, same length, E4 balances either way),
   and the vector would go RED without E21's check. But **no step files V6, no
   step names E21, both steps that could file V6 still say it is STRUCK, and
   every close condition that counts rules or vectors excludes them** — §1.3's
   *THIRTEEN*, §6's *TWENTY-FOUR*, §6's *E1–E20*, §3.3's filing map, §6's
   near-miss bullet. The wiring gate's rule 5 goes green only because the stale
   strike sentences contain the token `V6`; correcting them makes the gate fire,
   which I reproduced by mutation. **[C1]**
2. **§2.3's split holds everywhere §2.5a.2 has a name, and fails at the one place
   it does not.** A record that fails its own parse refuses; a set that is
   incomplete, orphaned, duplicated, colliding or not-a-transaction reports. But
   the **identifier check** appears on both sides — *"an identifier"* in §2.3's
   REFUSE row, *"check BOTH carried identifiers"* inside the chain §2.2 rules is
   *"EVERY failure … a `SetFinding`, NOT an `Err`"* — while §2.2's own next
   sentence (unchanged, and named in r8-C1) and E17 (*"on BOTH forms"*, a live
   rule at exit 4) both say refuse, and §2.5a.2's closed five-name vocabulary has
   no condition it could report under. An implementer cannot tell which side that
   failure is on, and there is no vector for it. **[C3]**

**What the fold broke.** Every Critical this round is again in the previous fold,
and three of the four are at sites the previous report **named by line number and
the fold did not reach** — §2.2's both-identifiers sentence (r8-C1's *Where*),
§2.4's W14 row (r8-C5's *Where*), and §6's near-miss bullet (the r8-C2 defect at
a site r8 missed). The fourth, [C1], is a new rule and a new vector wired into
nothing. Twelve of twelve terms this fold superseded are still standing
elsewhere, and none of them was added to the gate that exists to find exactly
that.

**The cheapest single action before round 10** is not a review: it is adding
this fold's own supersessions to `:1792`'s `--terms` block — `THIRTEEN LIVE
rules`, `TWENTY-FOUR live vectors`, `V6, … are STRUCK`, `the rule name`,
`in both cases BOTH identifiers must match` — and re-running the four gates with
their baselines re-measured.
