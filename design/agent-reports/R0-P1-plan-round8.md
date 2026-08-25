# R0 — P1 container plan, ROUND 8 (v11 diff-scoped adversarial gate)

**Artifact:** `design/IMPLEMENTATION_PLAN_P1_me_container.md` @ v11 (HEAD `7936306`)
**Scope:** `git diff 6a3a8c8..HEAD` — two operator rulings + three simplification cuts. Sections the diff did not touch are audited ONLY where the diff falsified them.
**Reviewer:** independent, opus. Nothing in this report was transcribed from the controller.

---

## Commands run, and their raw output

```
$ git diff --stat 6a3a8c8..HEAD -- design/IMPLEMENTATION_PLAN_P1_me_container.md
 design/IMPLEMENTATION_PLAN_P1_me_container.md | 336 ++++++++++++++++----------
 1 file changed, 204 insertions(+), 132 deletions(-)

$ wc -l design/IMPLEMENTATION_PLAN_P1_me_container.md
2079 design/IMPLEMENTATION_PLAN_P1_me_container.md

$ git show 6a3a8c8:design/IMPLEMENTATION_PLAN_P1_me_container.md | wc -l
2007
```

```
$ ./scripts/plan-wiring-check.sh design/IMPLEMENTATION_PLAN_P1_me_container.md
--- wiring rows: 13 live, 2 retracted
--- steps parsed: 13   vectors parsed: 24
--- live:      W1 W2 W3 W4 W5 W8 W9 W10 W11 W12 W13 W14 W15
--- retracted: W6 W7

  FINDING  E1 is a STRUCK rule (row at line 381) but line 405 references it as live
  FINDING  E6 is a STRUCK rule (row at line 386) but line 341 references it as live
  FINDING  E7 is a STRUCK rule (row at line 414) but line 1834 references it as live
  FINDING  E8 is a STRUCK rule (row at line 388) but line 134 references it as live
  FINDING  E16 is a STRUCK rule (row at line 396) but line 341 references it as live
  FINDING  E16 is a STRUCK rule (row at line 396) but line 420 references it as live
  FINDING  V16 is a STRUCK vector (row at line 1210) but line 405 references it as live
  FINDING  V17b is a STRUCK vector (row at line 1212) but line 882 references it as live
EXIT=0
```

**Rule 4b is at its 8-reference baseline. No ninth. That half of Part A is clean.**

```
$ ./scripts/plan-fold-sweep.sh design/IMPLEMENTATION_PLAN_P1_me_container.md --terms \
    'Σ(3 + len)' 'n_fields = 0' 'gains a **set-level** variant' 'all refuse' \
    'the RULE NAME' 'Tag `0x01`' 'tag=0x01' 'require it COMPLETE' \
    'TWO NAMED EXCEPTIONS' 'V1–V6 and V8–V26'

─── 10 superseded term(s) named by the fold author

STILL PRESENT  Σ(3 + len)                                     1 occurrence(s)
                 384:| E4 | **`71 + Σ(3 + len) + 4 + body_len` MUST equal the decoded length exactly.**
STILL PRESENT  n_fields = 0                                   1 occurrence(s)
                 1194:| V1 | **RAW (segwit)**, no optional fields | the fixed layout; `n_fields = 0` |
STILL PRESENT  gains a **set-level** variant                  1 occurrence(s)
                 934:| **W12** | `SyswError` gains a **set-level** variant. …
STILL PRESENT  all refuse                                     1 occurrence(s)
                 1630:  | W10 | V25's three negatives all refuse; V3's complete payload packs |
STILL PRESENT  the RULE NAME                                  2 occurrence(s)
                 1423:**`expect` is a sum type, and the refusal arm carries the RULE NAME** — which is
                 1624:  | **W14** | … and **the `refuse` arm checks the RULE NAME** …
STILL PRESENT  Tag `0x01`                                     2 occurrence(s)
                 394:| **E14** | **Tag `0x01`'s value MUST be valid UTF-8. Invalid is REFUSED.**
                 395:| **E15** | **Tag `0x01`'s value is `1..=64` bytes.**
STILL PRESENT  tag=0x01                                       2 occurrence(s)
                 1216:| **V21** | **`tag=0x01` whose value is `74 6f ff 21`** |
                 1217:| **V22** | **`tag=0x01, len=65`; near-miss `len=64` must PASS** |
STILL PRESENT  require it COMPLETE                            1 occurrence(s)
                 665:| CHUNKS | gather the record's set (E20: …), require it COMPLETE and PRISTINE (E19), …
STILL PRESENT  TWO NAMED EXCEPTIONS                           1 occurrence(s)
                 403:check — WITH TWO NAMED EXCEPTIONS, because a completeness claim that is false …
STILL PRESENT  V1–V6 and V8–V26                               2 occurrence(s)
                 1335:vector file"* and none named it; §4 assigned the construction of V1–V6 and V8–V26
                 1437:`scripts/gen-tx-record-vectors.py` and files **V1–V6 and V8–V26** (27 plan ROWS …
```

**Ten for ten. Every one of those lines is a finding below.** This is not a
reviewer's cleverness — it is the plan's own gate, run against the ten terms v11
superseded and never added to its `--terms` block. See **[I5]**.

Also read, verbatim: `design/FOLLOWUPS.md:10392–10500` (RULING 2026-08-25 and
2026-08-25b). The rulings are the operator's; only the plan's fidelity to them is
reviewed here.

---

## Part A — what v11 broke

### [C1] "Report, don't refuse" was folded into the vocabulary sections and NOT into §2.2/§2.3, which are where DECODE is defined

**Severity:** Critical
**Where:** `design/IMPLEMENTATION_PLAN_P1_me_container.md:660-668` (§2.2's CHUNKS row and the both-identifiers sentence); `:762-772` (§2.3, heading and NORMATIVE line); and three downstream sites: `:609` and `:614` (§1.5's exit table — *"**E1–E20** and §2.3's decode failure | **4** `EXIT_INVALID`"*), `:1471` (§4 step 10's rule column), `:1717` (§6's near-miss bullet).

**The failure, concretely.** §2.2 is titled **"NORMATIVE — both forms end at the same proof"** and is the single place the plan says what DECODE *is*. Its CHUNKS row still reads, untouched by the diff:

> gather the record's set (E20: `chunk_set_id` == the top 20 bits of the carried txid), **require it COMPLETE** and PRISTINE (E19), reassemble via `mt_codec::decode`, THEN deserialise the result as a Bitcoin transaction, **and require BOTH carried identifiers to match it**

and the sentence beneath it: *"in both cases BOTH identifiers **must** match … MUST equal the carried `wtxid` (E17)."* §2.3, also untouched, is titled **"A `tx:` record that fails DECODE is REFUSED, not warned"** and rules **"NORMATIVE: `tx:` follows `seal`'s posture — refuse."**

An implementer builds `split` from §2.2 + §2.3, because those are the two sections that define the contract §2.4's W4 row points at (*"hex-decode, then §2.2's DECODE"*). They write: gather → if incomplete, `Err`; reassemble → if it is not a transaction, `Err`; compare identifiers → if they differ, `Err`. Then they run the step-10 tests and **V25, V27 and V28 — all three of which v11 rewrote to "PACKS at exit 0 with a `SetFinding`" — go RED and cannot be made green without deleting the code §2.2 told them to write.**

§4 step 10's own acceptance column says the same thing in the step that builds the machinery: *"the identifier-consistency **refusals** on both forms, **E20's set binding**, and W15's CHUNKS decode chain"*. On the CHUNKS form all three of those are now reports. And §1.5's NORMATIVE per-refusal exit table still assigns **exit 4** to the range **E1–E20**, which includes E20 — a rule the same fold demoted to SHOULD and to exit 0.

**Why the plan permits it.** The ruling was folded into §1.3's E20 row, §2.4's W12/W13/W15 rows, §2.5a.2's condition table and five vector rows — the places that *name* the conditions. §2.2 and §2.3 do not name a condition; they state the requirement the conditions are derived from, and neither contains the tokens `E20`, `chunk_missing`, `set_collision` or `not_a_transaction` that a search for the ruling's subject matter would hit. §2.3 does not mention chunks at all. `plan-wiring-check.sh` rule 4b sees no struck label here because nothing here is struck.

**Confidence:** Certain. Both sections are titled NORMATIVE, both are unmodified in `git diff 6a3a8c8..HEAD`, and §2.5a.2's own ruling block explicitly accepts the trade §2.3 refuses to accept (*"the smuggled bytes reach metal, under a warning legend … Deliberate trade"* vs §2.3's *"not defensible for the one record class whose failure mode is secret bytes riding in cleartext"*). One of those two sentences has to go.

---

### [C2] §6's W10 closure gate cannot pass: "V25's three negatives all refuse"

**Severity:** Critical
**Where:** `:1630` — `| W10 | V25's three negatives all refuse; V3's complete payload packs |`

**The failure, concretely.** This is a **close condition**. V25's row at `:1220`, rewritten by this same diff, says *"**(RULING 2026-08-25) all three PACK at exit 0** and each emits its own `SetFinding` on stderr — v10 required all three to REFUSE."* The two statements are the negation of each other, and one of them is the gate that decides whether P1 ships. Implement the ruling and W10's gate is unsatisfiable; satisfy W10's gate and V25 is RED. **Either way a phase gate reports the wrong colour.**

**Why the plan permits it.** The fold rewrote the W12, W13 and W15 closure rows two and three lines away, and W15's *replacement* text even quotes the superseded wording of a neighbouring row (*"v8's W10 row (**"V25 refuses, V3 packs"**) is satisfied by an E20-only implementation"*) — so the author had W10's row in view while editing the row beneath it and did not update it. This is the plan's own diagnosed defect class: *"Answering a finding is not the same as finishing its neighbourhood"* (v10's status paragraph, deleted by this diff).

**Confidence:** Certain — the two lines are 410 lines apart in the same document and directly contradictory.

---

### [C3] §2.5a's NORMATIVE table still builds the `SyswError` set-level variant that §2.4's W12 struck

**Severity:** Critical
**Where:** `:925-936` — "### 2.5a NORMATIVE — what P1 builds instead", rows W12 and W13.

**The failure, concretely.** §2.5a's table is captioned *"**A rule failure must carry the rule.** … Three additions, replacing W6's edit"* and says:

> | **W12** | `SyswError` gains a **set-level** variant. It may not carry a bare `usize` … |
> | **W13** | the printer arm for **both**, at `sysw_error` **and its outer match** … |

§2.4's W12 row, rewritten by this diff, says the opposite in bold: *"**A SET REPORT, NOT AN ERROR VARIANT.** v10 gave `SyswError` a set-level variant; **nothing set-level errors any more, so the variant is struck.** `split` returns `Vec<SetFinding>` alongside a **successful** pack."* And §2.4's W13: *"`sysw_error`'s outer match keeps only the **per-record** arms."*

An implementer who builds from §2.5a — the section explicitly headed *what P1 builds* — adds `SyswError::SetLevel(...)`. Adding an error variant to the type `split` returns is how a set-level condition becomes an abort: there is no way to return `Err(SyswError::SetLevel(..))` from `split` and still pack. **The refusal posture is reinstated by following a NORMATIVE table.**

**Why the plan permits it.** §2.5a is a 12-line summary table that predates §2.4's per-row detail and duplicates it. The diff edited the detail and not the summary. `plan-wiring-check.sh` counts W-rows in §2.4 only; a second, older W12 row elsewhere is invisible to it.

**Confidence:** Certain.

---

### [C4] E4 — a LIVE rule — still states the TLV arithmetic, and it refuses every legal record that carries a fee

**Severity:** Critical
**Where:** `:384` — `| E4 | **`71 + Σ(3 + len) + 4 + body_len` MUST equal the decoded length exactly.** | … the fixed part up to and including `n_fields` … **A Go porter transcribes this arithmetic** |`

**The failure, concretely.** Take the simplest record v11 admits with a legend: RAW, fee present, no fingerprint, no label. `flags = 0x01`. Its length is `4+1+1+32+32+1 + 8 + 4 + body_len` = **`83 + body_len`**. E4 as written computes `71 + Σ(3 + len) + 4 + body_len`; there are no TLVs, so `Σ = 0`, giving **`75 + body_len`**. `75 + body_len ≠ 83 + body_len`, so **E4 refuses a byte-perfect record.** An implementer who instead reads `Σ(3 + len)` as ranging over the present *slots* gets `71 + (3+8) + 4 + body_len = 86 + body_len` and refuses it differently. Two implementations, two wrong answers, one legal input — which is the plan's own definition of what an E-rule exists to prevent, in the rule whose justification column ends *"A Go porter transcribes this arithmetic."*

The correct expression under the new layout is `71 + 8·bit0 + 4·bit1 + (1+L)·bit2 + 4 + body_len`. §1's 3.1 blockquote gestures at it — *"E4's Σ-arithmetic becomes a constant plus the label length"* — which is itself imprecise (with three independently optional slots it is not a constant), and in any case **the blockquote is not the rule; E4's row is**, and E4's row was never edited.

Second-order: E4 is the only rule that detects a **flags bit set with its value missing** (a truncated legend). With E4 stating unreachable arithmetic, a record with `bit0` set and four bytes remaining before `body_len` has no stated verdict.

**Why the plan permits it.** SIMPLIFICATION 3.1's DELETED-OUTRIGHT list names E1, E2, E8, E10, E16 and folds E6/E7 into the flags byte — E4 is mentioned only in a subordinate clause of the same sentence, and E4 is **not struck**, so `plan-wiring-check.sh` rule 4b never looks at it. `n_fields` and `Σ(3 + len)` were never added to the fold-sweep terms list (**[I5]**).

**Confidence:** Certain. The arithmetic is checkable by hand and the row is verbatim unchanged in the diff.

---

### [C5] SIMPLIFICATION 3.2 was folded into §2.5a.1 only — §3.3 and §6's W14 still make the rule name the fixture contract, and W14's gate can no longer be run

**Severity:** Critical
**Where:** `:1395` (the schema example), `:1416-1418` (the NORMATIVE r6-C2 blockquote), `:1423` (the `expect` sum-type sentence), `:1624` (§6's W14 closure row), `:833` (§2.4's W14 row).

**The failure, concretely.** §2.5a.1 now rules: *"**NORMATIVE — the per-record rule name is a Rust-internal detail.** … **The fixture pins the OUTCOME, not the name:** `expect: {"pass": {...}}` or `expect: {"refuse": {}}`. … it need not agree on what the failure is called."*

Four other places still say the opposite, three of them NORMATIVE:

- `:1395`, the schema the generator is written against: `"expect": { "refuse": { "rule": "magic" } }`
- `:1416-1418`, inside a block headed **NORMATIVE (r6-C2)**: *"`tx_record_vectors.json` pins what the record codec produces and refuses — the parsed fields on `pass`, **the rule name on `refuse`**. **That is the whole cross-language contract** … and the Go port checks exactly it."*
- `:1423`: *"**`expect` is a sum type, and the refusal arm carries the RULE NAME**"*
- `:1624`, a **close condition**: *"the `refuse` arm checks **the RULE NAME** — **flip any expected rule name in the fixture and the test goes RED.**"*

Two distinct failures follow. **(a)** An implementer writes `scripts/gen-tx-record-vectors.py` from §3.3's schema, emits rule names, and the Go port compares against them — the exact three-hand cross-language contract 3.2 abolished, restored by the section that tells you how to build the file. **(b)** §6's W14 gate is **unrunnable**: if the fixture holds `{"refuse": {}}` there is no expected rule name to flip, so the mutation that defines the gate cannot be performed. That is the "a gate that has never been run is a hypothesis, not a gate" shape, one round after the fold that created it.

`:1423` compounds it: *"the refusal arm carries the RULE NAME — **which is what makes V27 able to go RED**"*. V27 no longer refuses at all (`:1222`, rewritten by this diff, "PACKS at exit 0"), and §3.3's own filing map puts V27 at step 10, not in the codec fixture. The sentence is false twice over, and an implementer following it files a `set_collision` refusal into a codec-level fixture that §3.3 forbids from naming container outcomes.

**Why the plan permits it.** 3.2's fold edited §2.5a.1 (which is where the *vocabulary table* lived) and the §2.5a.2 exemption note. §3.3 is 400 lines away, is about *file layout*, and contains no E-number or V-number that a struck-label gate could catch.

**Confidence:** Certain.

---

### [C6] RULING 2026-08-25's second normative MUST — the incomplete set visible in `me sysw show` — is implemented nowhere

**Severity:** Critical
**Where:** absent. The sites that would carry it are `:827` (§2.4's W9 row) and `:1629` (§6's W9 closure row).

**The failure, concretely.** `design/FOLLOWUPS.md:10430-10435` states three MUSTs for the report:

> P1's incomplete-set report MUST:
> - emit a **stderr warning at pack time** naming the set and **every** missing index, not the first (r7-M1);
> - **be visible in `me sysw show`, marked INCOMPLETE with the missing indices;**
> - carry no format change — the chunks' own `count`/`index` let any reader recompute it, so P4's device display can too.

The plan implements the first (W12/W13) and the third (no format change). **The second appears nowhere in 2,079 lines.** `grep -n "INCOMPLETE" ` returns four hits: §2.5a.2's stderr message template, a §3.1 ceiling row about a spec number, and V25's row — none of them `show`. W9's row prescribes *"one line per chunk SET, not per chunk"* and W9's closure row asserts *"on a 202-chunk payload prints **one** set line"*. Neither says what that line contains about set state, and `split`'s `Vec<SetFinding>` is rendered by W13 *"to stderr after a successful pack"* only — `show` operates on the packed blob later and is given no set state at all.

The operator packs at 09:00 and gets *"set `0x2dcf2` is INCOMPLETE — missing chunks 7, 12, 88 of 202"* on stderr. The terminal scrolls. At 14:00 they run `me sysw show payload.txt` as their pre-flash read-back. Per the plan as written, `show` prints one set line with no incompleteness marking, and they flash and engrave believing the set complete.

**Why the plan permits it.** The ruling's rationale block was folded into §2.5a.2 nearly verbatim — including the sentence *"a stderr line is gone in a week and **`me sysw show` must be re-run to be seen**"* (`:1019`), which is the plan quoting the argument for the requirement while omitting the requirement. The MUST is in a bullet list in `FOLLOWUPS.md` and the fold took the prose paragraphs above and below it.

**Confidence:** Certain that it is absent. High that it is Critical: the ruling's entire stated reason for preferring report over refuse is that the durable surface must carry the warning, and `show` is the only host-side durable surface P1 owns.

---

### [C7] The flags byte's reserved bits are a MUST with no rule, no name, no vector and no test — and they are the divergence surface E8 used to cover

**Severity:** Critical
**Where:** `:117-118` — the only statement in the document:
```
70    1    flags       bit0 fee present | bit1 fingerprint present
                       bit2 label present | bits 3-7 MUST be zero
```
`grep -n "bits 3"` → 1 hit. `grep -n "MUST be zero"` → 1 hit. §1.2's *"refused when"* column has no row for the flags byte. §1.3 has no rule. §3 has no vector. §2.5a.2 has no condition.

**The failure, concretely.** Feed both implementations a record with `flags = 0x09` (bit0 set, bit3 set) followed by a valid 8-byte fee. Rust's implementer, reading "MUST be zero" as normative on the decoder, refuses. The Go porter, reading the same line as a statement about what the *encoder* emits, masks with `& 0x07`, parses the fee, and accepts. **One implementation refuses a record the other engraves** — the sentence §1.3's own header uses to define why E-rules exist.

This is round 0's **I3** reproduced exactly, and §1.2's new blockquote quotes I3 while creating a fresh instance of it: *"a width in a tag table is **'a description of the encoder, not a refusal binding the decoder'**"*. The flags byte's reserved bits are a description of the encoder in a layout table, and nothing binds the decoder.

It is also the *replacement* for the surface 3.1 deleted. 3.1's justification for cutting TLV is that **E8** (*"An unknown tag is REFUSED, not skipped"*) had already forbidden forward compatibility. The flags byte reintroduces the same question — five spare bits a future version could use — and 3.1 deleted E8 and V6 (*"unknown tag → REFUSED"*) without re-cutting either against the bits. V12 was re-cut to the label slot; **V6 was not re-cut to anything.**

Under §6's own close condition — *"Anything else without a RED test is a defect, not an exception"* — this is a defect by the plan's definition.

**Why the plan permits it.** The constraint was written into an ASCII layout comment rather than into §1.3, so it has no E-number; nothing in the plan's machinery (rules table, vector table, rule-name vocabulary, wiring gate, refusal-coverage table) indexes anything that lacks one.

**Confidence:** High. I cannot rule out that the author intends "MUST be zero" to be self-enforcing, but the plan's own I3 argument is that a MUST in a layout table is not a decoder refusal, and the plan states that argument twice.

---

### [C8] V15's expected outcome was never folded, and it is filed in a fixture that cannot express the outcome the ruling gives it

**Severity:** Critical
**Where:** `:1209` (V15's row), `:1465` (§4 step 4 files V15 into `tx_record_vectors.json`), `:1471` (§4 step 10 also lists V15), `:1416-1418` (§3.3's NORMATIVE bar on container outcomes in that fixture).

**The failure, concretely.** V15 is *"a chunks record whose carried txid's top 20 bits ≠ its chunks' `chunk_set_id`"*, with the NORMATIVE clause that the perturbation goes into the **chunks'** embedded `set_id`, leaving the carried txid honest. The diff rewrote V25, V27 and V28's outcomes to "PACKS at exit 0 with a finding" and left V15 alone, so its row still reasons entirely in refusals: *"Perturb the **txid** instead and §2.2's full txid equality **refuses the record** on its own, so V15 would stay green with R15 deleted."*

But under the rulings V15's payload **packs**. Its chunks carry a `set_id` matching no `tx:` record → every chunk is an **ORPHAN**; the `tx:` record's set is empty → **INCOMPLETE**. Both are `REPORT LOUDLY, PACK` per §2.5a.2. So:

1. **V15 has no stated expected outcome.** Its row implies refuse; §2.5a.2 says exit 0 plus two findings; nothing reconciles them.
2. **V15 cannot be filed where the plan files it.** §4 step 4 explicitly names V15 in the codec fixture, and §3.3's NORMATIVE block rules that fixture *"may not name a container or process outcome"* — but "packs at exit 0 emitting an orphan finding and an incompleteness finding" **is** a container outcome, produced by `split`, not by the record codec. Under v10 the entry at least *looked* writable as `{"refuse": {"rule": ...}}`; the ruling turned it into an outcome the schema forbids.
3. **R15's only RED test is at risk.** V15's whole purpose is *"delete R15's comparison and every other vector stays green"*. Delete R15's comparison and the chunks bind to no `tx:` record — which is what already happens in V15 — so the same orphan/incompleteness findings appear and an outcome-level assertion **stays GREEN**. The vector guarding the binding the entire CHUNKS form rests on loses its ability to fail, and the plan gives no replacement assertion.

**Why the plan permits it.** V15 sits between the two vectors the fold *did* update (V13 above, V16 struck below) and carries no `E20`, `chunk_missing` or `set_collision` token — the fold appears to have swept by condition name, and R15's failure has never been given one of §2.5a.2's five names.

**Confidence:** High on (1) and (2), which are textual. Medium-high on (3), which depends on how "delete R15's comparison" is modelled — but the plan does not say, and that is itself the defect.

---

### [I1] §1.3 and §6 each say, in one breath, both "two exceptions" and "E11 alone"

**Severity:** Important
**Where:** `:402-404`, `:411`, `:420` (§1.3); `:1597-1600` (§6).

**The failure, concretely.** §1.3 reads *"Every one of the **THIRTEEN LIVE** rules gets a vector … **WITH TWO NAMED EXCEPTIONS**"*, then a table headed *"**THE TWO EXCEPTIONS**"* whose first row is struck, then *"**(3.1) E11 is now the ONLY one**"*. §6's closure bullet is worse, because it is a gate: *"the exception list below is now **shorter by two** … The remaining exception is **E11 alone** — **EXCEPT E7 and E11**, which §1.3 names, explains and assigns."* The tail of the old sentence was left attached to its own correction. "Shorter by two" is also wrong: the list went from two entries to one, i.e. shorter by **one**.

An implementer or the next reviewer cannot tell whether E7 is still owed a carve-out. §1.3's closing paragraph then reasons from E16's "second halves" (`:420`), a struck rule — one of the eight rule-4b baseline hits.

**Confidence:** Certain.

---

### [I2] §4 step 8's rule column claims coverage of six struck rules and cites a struck exception

**Severity:** Important
**Where:** `:1469`, right-hand cell: *"every rule in **E1–E19 EXCEPT E7, E11, E12, E13, E17 and E19**"*.

**The failure, concretely.** Expand it: E1, E2, E3, E4, E5, E6, E8, E9, E10, E14, E15, E16, E18. **Six of the thirteen — E1, E2, E6, E8, E10, E16 — were deleted by this diff.** The cell then explains itself with *"**E7 and E11** are §1.3's **two** NORMATIVE exceptions"*, where E7 is struck. The left cell of the same row *was* updated (*"V6, V9, V14, V16, V17 and V17b are STRUCK"*); the right cell was not.

An implementer working step 8 reads its acceptance column as the coverage contract for the step and goes looking for a tag-order test and a duplicate-tag test that cannot be written. This is precisely the class the brief flags: `plan-wiring-check.sh` exempts range labels, so `E1–E19` passes the gate while naming six rules that no longer exist.

**Confidence:** Certain.

---

### [I3] §4 step 8 gained V15, which is step 10's and unbuildable at the codec layer

**Severity:** Important
**Where:** `:1469` — step 8's vector list is now `V5, V10–V13, **V15**, V19, V21, V22`.

**The failure, concretely.** v10's step 8 read `V5–V6, V9–V14, V16–V17b, V19, V21, V22` — the range `V9–V14` stops at V14 and **V15 was not in it**. Compressing that list around the six strikes introduced V15. But V15 is a **payload-level** vector: it needs sibling `mt1` records, and step 8's own left cell rules that vectors of that shape are *"BARE `mt1` records with no `tx:` framing, and the layout codec never sees one … sited here they would pass vacuously."* V15 is already correctly sited at step 10, and §2.2 states why (*"The gathering is not `classify`'s … It is `split`'s, which is W10"*, six steps later). Step 8's rule column also lists no rule for it — R15 has no E-number.

An implementer at step 8 tries to build a set-binding negative against a codec whose input is one framed record, and either writes a vacuous test or stalls.

**Confidence:** Certain that V15 is new to step 8 (verified against `git show 6a3a8c8`).

---

### [I4] §3.3 and §4 step 4 disagree about which vectors the fixture holds — 27 rows vs 21, and §3.3's list names five struck vectors

**Severity:** Important
**Where:** `:1437` (§3.3) vs `:1465` (§4 step 4).

**The failure, concretely.** §4 step 4 was rewritten to *"files **V1–V5, V8, V10–V13, V15, V18–V26** — the live record-level rows after 3.1 struck V6, V9, V14, V16, V17 and V17b — **21 plan ROWS**"*. §3.3's "Which step files which vector" paragraph — the section whose entire job is that mapping — still says *"§4 **step 4** … files **V1–V6 and V8–V26** (**27 plan ROWS** …)"*, and its parenthetical was half-folded (*"counting V4a and V4b; **V17b is struck by 3.1**"*) so it names one strike and re-asserts the other five inside the range. The paragraph then totals *"That is **29**"*.

Two normative statements about the same JSON file give different contents and different counts. The generator is written from one of them. If it is §3.3's, five struck vectors are generated and W14's loader asserts vectors for rules that no longer exist.

**Confidence:** Certain.

---

### [I5] The fold-sweep terms block added ZERO v11 terms while claiming "each of which this fold removed" — the gate ran vacuously, and it would have caught ten of the findings above

**Severity:** Important
**Where:** `:1790-1845` — *"The `--terms` list is fixed, because the explicit mode is the one that works and **the fold author is the only one who knows what was superseded.** These **forty-four**, each of which **this fold** removed…"*

**The failure, concretely.** All forty-four terms are v1–v10's. v11 superseded TLV framing, `n_fields`, tag numbering, tag widths, the 26-name vocabulary, the set-level `SyswError` variant and the entire refuse posture, and added **not one term**. §6.1's own close condition reads *"**exactly 44 hits, one per term, ALL of them inside the block below** … A forty-fifth hit anywhere else is a real finding"* — which v11 satisfies trivially, because the check was never pointed at anything v11 did.

I ran it with ten terms v11 actually superseded. Ten of ten are still standing elsewhere in the document, and each one is [C1]–[C5], [I1] or [I4] above (raw output in the header). The gate works; it was handed an empty question.

**Confidence:** Certain — demonstrated by execution, not argued.

---

### [I6] E14, E15, V21 and V22 identify the label by a tag number that no longer exists, and tag→bit is a permutation, so the naive read lands on the wrong field

**Severity:** Important
**Where:** `:394` (E14), `:395` (E15), `:1216` (V21), `:1217` (V22). Two of these are LIVE rules.

**The failure, concretely.** The old numbering was `0x01` label, `0x02` fee, `0x03` fingerprint. The new numbering is bit **0** fee, bit **1** fingerprint, bit **2** label. E14 and E15 still read *"**Tag `0x01`**'s value MUST be valid UTF-8"* and *"**Tag `0x01`**'s value is `1..=64` bytes"*, and V21/V22 still read *"`tag=0x01` whose value is `74 6f ff 21`"* and *"`tag=0x01, len=65`"*. A reader mapping `0x01` to bit 1 gets the **fingerprint** — a 4-byte structural slot with no length byte, for which "len = 65" and "invalid UTF-8" are both meaningless. §1.2's table is the only place the mapping can be recovered, and it never says "tag 0x01 is now bit 2".

E15's justification is also now false on its own terms: *"a **`u16 len`** otherwise admits a 65,535-byte label"* — the label's length is a `u8`, so 255 is the ceiling the rule is protecting against, not 65,535.

**Confidence:** Certain on the staleness; high that it is Important rather than Minor, because two of the four sites are live rules an implementer transcribes and the renumbering is a permutation rather than a shift.

---

### [M1] The status paragraph's measured line count is wrong

`:10-11` — *"LINES 2,007 → **2,068**"*. Measured: `wc -l` = **2,079** (v10 = 2,007, confirmed). The header was written at `717448c`; commit `7936306` then added 11 lines and the figure was not re-measured. The delta's other three figures are correct (rules 20→13, vectors 30→24 per `plan-wiring-check.sh`, rows 156→146).

### [M2] §3.3's heading still says "THE TWENTY-NINE VECTORS"

`:1332` — *"### 3.3 (r4-I3, REWRITTEN r5-C1) WHERE THE **TWENTY-NINE** VECTORS PHYSICALLY LIVE"*. Twenty-four.

### [M3] V1's row pins a field that no longer exists

`:1194` — *"the fixed layout; **`n_fields = 0`**"*. Should be `flags = 0x00`. V1 is the fixture's first entry.

### [M4] §6's "V1–V27 pass" names six struck vectors and omits V28

`:1595`. V6, V9, V14, V16, V17 and V17b are struck; V28 (r6-C1's vector, and the only RED test for W15) is outside the range. A range label, so rule 4b is blind to it.

### [M5] Two narrative sites still describe the legend in TLV terms

`:463` (*"every TLV rule passes"*, in §1.4's E17-vs-E11 walk) and `:1651` (*"magic, version, form, both identifiers, **`n_fields`**, the TLVs and `body_len`"*, in §6's vector-provenance bullet). Both are arguments, not instructions, but both use vocabulary the layout no longer has.

### [M6] The report roster at the top stops at round 3 and calls it "the live one"

`:16-23` — lists round0–round3 with *"(the live one, 2C/7I/3M on v4)"*. `design/agent-reports/` holds `R0-P1-plan-round0.md` … `round7.md`, and this file makes eight. A reader trusting the roster misses four rounds of findings.

---

## Part B — safe to evict / must survive

An archaeology pass should execute this list. Estimated ~230 lines evictable of 2,079, which is real but is **not** where the plan's weight is — the weight is in per-row justification columns that are load-bearing.

### MUST SURVIVE — things that look like archaeology and are not

1. **`:1790-1845`, the `plan-fold-sweep.sh` `--terms` block (56 lines).** It is 44 lines of superseded phrases with no live referent, which is the exact profile of archaeology — **and it is an executable gate input.** Deleting it silently disarms the only gate that catches incomplete propagation, which is the defect class that produced [C1]–[C5]. It does not need evicting; it needs **extending** (see [I5]). This is the sharpest trap in the document.

2. **`:388`, E8's struck row — specifically its reason.** *"**This rule is WHY 3.1 was taken**: it disabled TLV's only benefit (skip-what-you-don't-know) while the plan paid TLV's entire divergence surface."* This is the only record anywhere of why TLV cannot come back. Delete it and the next reviewer who wants forward compatibility re-proposes TLVs with no counter-argument on file. **Keep verbatim.** The same is true, more weakly, of **E16's** reason (it is the disposition of round 0's I3, which would otherwise read as an unclosed finding) and **E7's** (it is what stops someone re-adding a sentinel encoding). E1's, E2's, E6's and E10's reasons are one-line re-derivations of the layout and can compress into a single sentence.

3. **`:254-330`, §1.1a's five-row measurement block.** Executed evidence that E11 does not fire on V18 and that the wtxid does. It is the entire justification for E17's existence and for a 32-byte field in the framing. Prose cannot replace it — the plan already tried, and r2-C1 was the result.

4. **The NORMATIVE construction clauses on V8, V15, V27 and V28.** They read like fold commentary and they are the only thing that makes those four vectors able to fail. §2.5a.1's cost paragraph now *depends* on them explicitly (*"Where it would not … §3 already requires the NORMATIVE construction clause that V8, V15 and V27 carry"*), so evicting them silently converts 3.2 from a simplification into a hole.

5. **`:414-416`, §1.3's exception table (E11's row and its owner column).** One live row and its "who owns it: P3" cell. It is the only record that E11 has no Rust RED test by measurement rather than by oversight.

6. **`:1078-1113`, §2.5b (E13's precedent is false, measured).** Reads as a retraction of an earlier claim; is actually a measured fact about `mt-codec`'s `to_symbols` that E13, V20 and V23 all rest on.

7. **`:986` and the r7-M1 orphan reasoning in §2.5a.2.** The 202-orphans / delete-record-0 spiral looks like a war story. It is the only statement of *why* the orphan message must summarise rather than abort, and it is a message-content requirement.

8. **`:1225-1290`, §3.1's four-ceiling table and the three falsified spec §2.3 numbers.** §6.3 owes those corrections to the spec; they are a work item, not history.

### SAFE TO EVICT

1. **`:47-81` — the "WHAT v1 GOT WRONG" and "WHAT THE v2 REWRITE BROKE" blockquotes (~35 lines).** Every fact is now a live artifact: item 1 → `plan-fold-sweep.sh`, item 2 → §1.4 exists, item 3 → §2.4 + step 6 + a closure row. **One sentence must be promoted before deleting**, item 4's: *"NORMATIVE FOR THIS DOCUMENT: no rule is added here without a stated input that would fail it, and that input measured."* That is a live standing rule and belongs in §1.3's preamble.

2. **`:914-924` — "And W6's prescribed edit is actively wrong" (~10 lines).** W6/W7 are retracted, `plan-wiring-check.sh` prints `retracted: W6 W7` on every run, and no step names them. Keep the three-mismatch table above it — that is the design rationale for a separate chunk channel — and drop the W6 paragraph.

3. **`:1332-1368` — §3.3's three-numbered refutation of `sysw_vectors.json` (~36 lines).** Compress to two sentences plus reason 1's citation (`crates/me-cli/src/sysw/vectors.rs:137` is a golden of the code under test, which is why §3.2 exists). Reasons 2 and 3 are measurements of a remedy nobody will propose again once `tx_record_vectors.json` exists on disk.

4. **`:1724-1760` — §6.1's `plan-build-gate.sh` history (~36 lines).** The live content is one NORMATIVE sentence (*"`plan-build-gate.sh` is NOT a close condition for P1"*) plus the reason (no ```rust blocks). The historical output, the "hole it demonstrates is already fixed" blockquote and the extractor walk-through are spent.

5. **The thirteen struck rows themselves** (E1, E2, E6, E7, E10, E16 and V6, V9, V14, V16, V17, V17b — **not E8**), *after* one more R0 round closes. Replace with a single paragraph: "SIMPLIFICATION 3.1 struck E1, E2, E6, E7, E10, E16 and V6, V9, V14, V16, V17, V17b — every one a TLV property that fixed slots make inexpressible. E8's row is kept because it records why TLV cannot return." Saves ~13 rows; costs nothing, because the reasons are all the same reason. **Not before then** — a reader mid-cycle who remembers v10 needs to find out what happened, which is the argument §6.2's struck R17 row makes for itself.

6. **`:16-23`, the report roster.** Stale ([M6]) and duplicated by `ls design/agent-reports/R0-P1-plan-round*.md`. Replace with that glob and the marker convention, which is the only part a reader uses.

---

## Verdict

**8C / 6I / 6M — v11 is NOT GREEN.**

The four questions Part A was asked, answered:

1. **"Report, don't refuse" does NOT hold end to end.** It was folded into every section that *names* a condition and into none that *states the requirement*: §2.2 (NORMATIVE, the definition of DECODE), §2.3 (NORMATIVE, "is REFUSED, not warned"), §2.5a's NORMATIVE build table, §1.5's exit table, §4 step 10's acceptance column and §6's W10 close condition all still refuse. Nothing was found reporting where it must still refuse — the per-record path is intact and correct. **And one of the ruling's own three MUSTs (`show` marked INCOMPLETE) is implemented nowhere.**
2. **Yes, the strikes left dangling references the gate cannot see** — six struck rules inside step 8's `E1–E19 EXCEPT` range, five struck vectors inside §3.3's `V1–V6 and V8–V26`, six inside §6's `V1–V27`, and prose reasoning from struck E7 and E16 in two places.
3. **No, the flags layout is not complete enough to implement.** E4 — a live rule — still states TLV arithmetic that refuses a legal record carrying a fee, and the reserved bits carry a MUST with no rule, no name, no vector and no test.
4. **V8 survives an outcome-only fixture; V27 survives because set-level names were exempted; V15 does not** — its outcome was never folded, and the ruling gave it a container-level outcome that §3.3 forbids its fixture from expressing.

**Part A is not clean.** Every one of the ten sites the fold-sweep found is a place v11 changed a fact and left its restatement standing — the same class the plan diagnosed in v10's own status paragraph. The cheapest single action before the next round is not a review: it is adding v11's superseded terms to `:1790`'s `--terms` block and running it.
