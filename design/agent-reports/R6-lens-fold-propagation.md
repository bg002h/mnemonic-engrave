# R6 — lens: fold propagation (`SPEC_mt_v0_1.md`)

**Question answered:** did the ~20 folds applied on 2026-08-23 leave contradictions,
orphans, or statements that some *other* edit made false?

**Scope:** propagation only. No fresh audit, no re-litigation of settled rulings.
Machine-verified inputs (structure check, cite check, legend probe, 13×89+71) were
taken as given and not re-derived, except where a finding *is* a disagreement with
one of them.

**Verdict: 0 Critical / 8 Important / 11 Minor / 4 Nit.**

Two of the Important findings are **second instances of defects this document
already claims to have fixed** (I-1, I-5), and one is a claim that a `grep` settles
in one second and that is now false (I-3).

---

## Critical

**None.** Nothing found in this pass produces a wrong result, a funds loss, or an
unmet guarantee on its own. The closest is I-1, which states a retracted chunking
rule in the present tense — but §3b's normative correction box sits 30 lines below
the worst instance and states the rule correctly, so the document as a whole still
contains the right answer.

---

## Important

### I-1. The retracted "flat 40 bytes per chunk" rule is still asserted in two places — one of them attributes it *to the section that retracted it*

> `> void**: §3b's correction established that chunk sizing is a flat 40 payload`
> `> bytes (`crates/md-codec/src/chunk.rs:224,253-254`), so the count is *exact* for a`
> `> given payload size.`

**Line 3062–3063** (§11, Provenance of the numbers).

Second site, **line 1069** (§3b, "What fits", first sentence):

> `A chunk carries **40 payload bytes** and `mt1`'s header admits **4,096 chunks**,`

Third, weaker site, **line 2752** (§10.12): "A 535 B transaction **balanced at
40 B/chunk**" — balancing gives 39 B/chunk for that payload, per line 1110.

**What makes it false.** §3b's own correction box, **lines 1100–1105**:

> "**An earlier version of this box called it "a flat 40 bytes per chunk", and
> that mis-describes the chunker — R4 lens 1.** `md-codec` computes
> `chunks_needed` against the 320-bit ceiling and then splits the payload
> **`bytes_per_chunk = ceil(len / count)`** … **No chunk is padded to 40.**"

and **lines 1108–1113**, which price the consequence: two implementers, one
following the sentence and one following the code, "produce different chunk
boundaries and therefore **plates neither can read**". §1e's measured table
(lines 642–648) gives 33, 37, 39, 40, 40, 40 bytes per chunk — 40 is the
ceiling the *count* derives from, never the size of each chunk.

This is the same defect §1.1 line 463 cites as already-fixed history: "*this
artifact has already produced that defect twice (§7's mitigations naming legend
fields §5 had deleted; **§11 asserting a chunk rule §3b had retracted**)*". §11
still asserts it, so that parenthetical is itself false.

**Minimal fix.** Line 3062: "chunk sizing derives from a 40-byte *ceiling*".
Line 1069: "A chunk carries **at most** 40 payload bytes". Line 2752: "at the
40-byte ceiling".

---

### I-2. "PSBT-only" survives in two places after §8.2e superseded it

> `    **Why PSBT-only, when `mt encode`'s PAYLOAD is a raw transaction.**`
> …
> `    So accepting raw hex would **silently disable two refusals**, including the`
> `    only check that inputs ≥ outputs, while the artifact looked identical. `mt``
> `    therefore requires a PSBT, runs the full refusal set against it, and then —`

**Lines 2589 and 2601–2603** (§10.10). Second site, **line 1464** (§6):

> `> **`mt`'s INPUT is always a finalized PSBT (§10.10), even for `mt encode`,`

**What makes it false.** §8.2e, **lines 1848–1875**: "**A raw signed transaction
is ACCEPTED, with a loud warning.** Operator ruling 2026-08-23 … **`mt` never
refuses the bytes** … **This supersedes the earlier PSBT-only input ruling.**"
And §10.10's own input row, **line 2583**: "a finalized PSBT (preferred) **or a
raw signed transaction** (§8.2e)" — so the row and the paragraph 6 lines below it
disagree, and §6 line 1464 cites §10.10 for a rule §10.10 no longer states.

The sub-table at **line 2598** is stale in the same direction: "§8.2b value
balance? … raw signed transaction | **cannot run** — no input amounts", against
§8.2e's table at **line 1861**, "raw, **node** | **✓ via `gettxout`**".

**Minimal fix.** Retitle 2589 ("Why a PSBT is *preferred*"), replace "therefore
requires a PSBT" with the §8.2e rule, add the node column to 2594–2599, and
rewrite 1464 to "`mt`'s input is a finalized PSBT or a raw signed transaction
(§8.2e)".

---

### I-3. "THE SPEC NAMES ZERO FLAGS" is false — two of `mt`'s own flags were added by today's rulings

> `    **THE SPEC NAMES ZERO FLAGS while requiring SEVEN operator inputs the PSBT`
> `    cannot supply — R4 lens 2.** A `grep` for `--[a-z]` returns one hit, and it`
> `    is the *deleted* locktime pair inside a retraction.`

**Lines 2652–2654** (§10.10).

**What makes it false.** `grep -o -- '--[a-z][a-z-]*'` now returns **five hits on
four lines**: `--template` (185, `md`'s), `--transaction` (**404**), `--quiet`
(**579**), `--timelocked` / `--immediate` (741, the retraction). Two of those are
`mt`'s own, both ruled today:

- **line 404** — "**Optionally, `--transaction <psbt|hex>`** — the sibling
  round-trip" (`mt verify`);
- **line 579** — "**`--quiet` suppresses it for scripted use; the default is
  loud**" (`mt decode`'s stderr report).

The operator-input table at **2661–2670** — the one the spec calls "a
prerequisite for implementation, not a nicety" — lists neither.

**Minimal fix.** Restate as "the spec names two flags and leaves seven inputs
unnamed", and add `--transaction` and `--quiet` rows to the table.

---

### I-4. §5 says `mt encode` prints "these five fields", which §0a retracted by name and §5 contradicts eight lines later

> `> that `mt encode` **prints these five fields on `stderr`** as suggested text`

**Line 1292** (§5's retention note).

**What makes it false.** Twice over.

1. §0a, **lines 125–127**: "**It is NOT §5's five fields, and an earlier version
   of this section said it was — U-5.** §5's set was designed for a `mt qr`
   plate…" — followed by the split table (131–133) that drops `PLATE n OF m` and
   adds a per-string `n/m`.
2. §5's own next sentence, **lines 1299–1300**: "**Six** fields, **164
   characters**, 7 lines — measured" (matches the regenerated probe).

**Minimal fix.** "§0a rules that `mt encode` prints suggested legend text on
`stderr` — §0a's split, not this section's six fields."

---

### I-5. §8.2c still argues from "five fields over six lines … no room for a sixth", and still calls the verb `mt string`

> `   > mitigation, and §5's legend has **no such field**: it is five fields over`
> `   > six lines, sized into §4's reservation, with no room for a sixth. So the`
> `   > instruction only lands where the operator controls the plate — **`mt`
> `   > string`**, whose layout is theirs by ruling (§3b).`

**Lines 1763–1766** (§8.2c).

**What makes it false.** Two independent edits:

- §5 is **six fields, 164 characters, 7 lines** (line 1300, machine-verified),
  because §10.21 added `FORMAT: mt1 codex32` — line 1352: "**+19 characters,
  145 → 164, and 6 → 7 lines**". The "no room for a sixth" argument was
  overtaken by the sixth field actually landing.
- `mt string` was renamed. **Line 169**: "**This renames the previous draft's
  `mt string`.**" This is the only surviving live use of the old verb name in the
  document (grep confirms: 169 is the retraction, 1765–1766 is an instruction).

**Minimal fix.** "six fields over seven lines, with no room for a seventh", and
`mt string` → `mt encode`.

---

### I-6. The retracted "spend" claim survives in §7's thesis sentence and in §5's own table row

> `An `mt` plate is **spendable by whoever holds it.**`

**Line 1614–1615** (§7, opening paragraph). Second site, **line 1627**: "is the
only one of the three that is **spendable by whoever picks it up**". Third,
**line 1305**, the rationale cell of the very row the ruling corrects: "| `BEARER
- ANYONE HOLDING THIS CAN BROADCAST IT` | 45 | **the plate is spendable**; this is
not a backup…". Fourth, **line 738** (§1 decision 7): "**warns if the plate would
be immediately spendable**".

**What makes it false.** §5's ruling box, **lines 1307–1319**: "**"BROADCAST", not
"SPEND" — operator ruling 2026-08-23, and the old wording contradicted a guarantee
this spec makes.** §8.6 refuses any input whose satisfaction does not bind the
outputs, so a holder **cannot redirect the money** … So `SPEND` was wrong in both
directions. It **overstates** the holder's power, implying theft that §8.6 exists
to prevent". Reinforced by §8.4, **lines 1989–1991**: "*"spendable"* is a claim
about a transaction's fate that depends on scripts, fees and unspent inputs —
**none of which `mt` evaluates**", and by line 738's own §8.4 target, **1971–1973**,
where the operator rejects "may be immediately spendable" as the output wording.

The engraved strings are all correct; it is the prose *about* them that still says
"spend". This is the class the sweep missed the first time: the string `SPEND` was
grepped, the claim was not.

**Minimal fix.** 1615 → "broadcastable by whoever holds it"; 1627 → "the only one
of the three that anyone picking it up can broadcast"; 1305 → "the plate is a
bearer instrument"; 738 → "warns if the transaction's locktime has already passed"
(§8.4's two facts).

---

### I-7. §9 states the §10.21 gap as open, in the present tense, after §10.21 closed

> `leaves one real gap, §10.21: **no legend field names the format, the tool, or`
> `the encoding**, so a recoverer holding steel has nothing on it telling them what`
> `software to look for.`

**Lines 2366–2368** (§9).

**What makes it false.** §10.21, **lines 2980–2982**: "~~Nothing on the plate names
the format.~~ **CLOSED**, operator ruling 2026-08-23: the suggested legend gains a
sixth field, **`FORMAT: mt1 codex32`**", and §5 carries the field at **line 1328**.

Note for the fold: the *practical* gap is real but for a different reason now —
§0a lines 110–115 rule that "**the realistic plate has NO legend on it**". Deleting
the sentence would lose a true hazard; restating it on §0a's grounds keeps it.

**Minimal fix.** "§5 now suggests a `FORMAT: mt1 codex32` line (§10.21), but §0a's
ruling means the realistic plate carries no legend at all, so a recoverer may still
hold steel that names nothing."

---

### I-8. The normative report block omits the set-prefix line that two other sections rule `encode` prints — and its own row count does not add up

> `   3. **`encode` appends, never edits.** Its two extra rows —`
> `      `CUT   14 strings, 1,228 characters` — go **below** `STATUS`, so the`

**Lines 511–512** (§1.1, "The report, stated once — three callers, one layout").

**What makes it false.** The block declares itself exclusive — **line 471**: "**This
block is normative and it is the only place the layout appears**", and **line 520**:
"**No caller reorders, reformats, or drops a row.**" Yet:

- §0a, **lines 150–153**, rules that "`mt encode` prints the shared prefix once and
  tells them the rule: *All 14 strings begin `mt1qzrf8x`…*";
- §10.10, **line 2649**, lists "**the set prefix** | the **first 7 characters after
  `mt1`** … — see below", and **lines 2705–2716** make it a ruled row ("**The SET
  PREFIX row, and why it is a row rather than a footnote.** Operator ruling
  2026-08-23").

That line appears nowhere in the block at 479–487, and "two extra rows" is
illustrated by **one** row. Either the prefix is the second row and the block drops
it, or the count is wrong; both readings are defects in the section whose stated
purpose (lines 459–467) is that the two views "**CANNOT DRIFT**".

**Minimal fix.** Add the prefix line to the block below `CUT`, and keep "two extra
rows".

---

## Minor

### M-1. The set-prefix example is 6 characters where the rule says 7 — twice

> `>     All 14 strings begin `mt1qzrf8x`. Strings sharing that prefix belong`

**Line 152** (§0a) and, verbatim, **line 2715** (§10.10).

`qzrf8x` is **6** characters after `mt1`. Both sites state the rule two lines above
— **line 147**: "**the first 7 characters after `mt1` are the same on every string
in it**", **line 2709**: "**the first 7 characters after `mt1` are the same on all
of them**" — and both cite the verified `md1` example `md1fveszps…`, which is
**7** (`fveszps`). §1's own chunk-7 rendering, **line 293**, gives the missing
character: `MT1QZRF8XK2V…`, so the 7-character prefix is `qzrf8xk`.

**Fix.** `mt1qzrf8x` → `mt1qzrf8xk`, both lines.

### M-2. §5's locktime field is one character over the probe, and the column no longer sums to 164

> `| `LOCKED TO BLOCK <n> ~<SEASON> <year>` / `LOCKED UNTIL <t>` | 35 | …`

**Line 1325.** `RESULTS_legend_budget_2026-08-22.txt` (regenerated today) measures
`LOCKED TO BLOCK 1383520 ~FALL 2034` at **34 ch**. With 34 the column sums
45+20+34+34+12+19 = **164**, the total stated at line 1300; with 35 it sums 165.

**Fix.** 35 → 34.

### M-3. "~96 characters" per hand-cut string, against a measured 89–90

> `What differs is what a chunk *costs*: one chunk is one hand-cut string of ~96`
> `characters, or about 1/24th of a machine-engraved QR symbol.`

**Lines 1085–1086** (§3b).

§3's header box, **line 864**: "Measured: a chunk-string goes from **89 to 90
characters**". §1e's table (642–648): 79 / 85 / 89 / 90 / 90 / 90. §1's
pre-engraving print, **line 248**: "strings 1-13 are 89 characters, string 14 is
71". 96 is the codex32 *maximum* (3 + 80 + 13) that `mt1`'s 320-bit payload budget
never reaches — and it is also the number §3 uses for the 96-**chunk** artifact
(lines 842, 875), which is how it likely arrived here.

**Fix.** "~90 characters".

### M-4. "~340 bytes per chunk of capacity" is an orphan of the 64-chunk ceiling

> `trading it for ~340 bytes per chunk of capacity is the wrong trade. The`
> `**163,840 B** ceiling stands`

**Lines 2762–2763** (§10.12).

344 B is the difference between the two **historical totals** at line 1128 — "a
**2,904 B versus 2,560 B** total ceiling … Those two totals are themselves now
historical". It was never per chunk (the per-chunk difference is ~5 B: 45.4 vs 40),
and against the 4,096-chunk ceiling named in the next clause the difference is
~22 KB.

**Fix.** "trading it for ~5 bytes per chunk of capacity".

### M-5. §4 calls `mt encode`'s layout undecided, and points at the wrong open question

> `**This section governs `mt qr` and nothing else.** `mt encode`'s layout is`
> `undecided and is §10.10.`

**Lines 1198–1199.**

§3b, **lines 1149–1155**: "*"How many codex32 characters fit a hand engraved plate?
As many as a user wants. **It is not our concern.**"* … **This spec does not
constrain any of them**". §10.11, **line 2732**: "**CLOSED — OUT OF SCOPE**".
§10.10 is the CLI surface; the layout question is §10.11 and it is closed.

**Fix.** "`mt encode`'s layout is the operator's (§3b, §10.11)".

### M-6. The `mt1 SET` row names a caller set that contradicts the paragraph four lines below it

> `   | `mt1 SET` | the caller had strings — `inspect`, `decode`, `verify` | …`

**Line 491.** Lines 516–520 enumerate the callers as `inspect` (stdout), `decode`
(stderr), `encode` (stderr + `CUT`). `encode` has strings — it just produced them,
and §0a has it printing their shared prefix — but is absent from the row; `verify`
is present, though its output is the separate format at 285–301 / 362–374, not this
report.

**Fix.** "`inspect`, `decode`, `encode`".

### M-7. A sentence is duplicated verbatim inside decision 1b

> `1b. **One ENGRAVING form in v0.1.** `mt qr` is deferred to its own cycle`
> `   (§0a) because QR conversion is a cross-format concern `md1` and `mk1` share. `mt qr` is deferred to its own cycle`
> `   (§0a) because QR conversion is a cross-format concern `md1` and `mk1` share.`

**Lines 592–594.** Straight fold artifact.

**Fix.** Delete the second copy.

### M-8. §3's "96" cites a table that does not contain it

> `> while §3b's own table measures the largest `mt qr` artifact at **96**, and`

**Line 842.** §3b's only table (1075–1082) tops out at **89** chunks and lists raw
signed-transaction bytes; 96 is derivable from §4's 3,809 B PSBT row (line 1252),
not from §3b.

**Fix.** Cite §4's table, or state "96 chunks for the 3,809 B PSBT payload (§4)".

### M-9. The stderr locktime line has two normative spellings, one of which prints the verdict §8.4 refuses

> `       LOCKTIME  block 1383520, ~FALL 2034   current height 1402887 — PASSED`

**Line 484** (§1.1's normative block) against **line 1978** (§8.4's report block):
`LOCKED TO BLOCK 1383520          current height 963663`.

§1.1 line 471 claims to be "the only place the layout appears"; §8.4 lines 1975–1976
introduce its own block as "the `stderr` report … a statement of what was read, not
a verdict", and lines 1984–1991 argue at length against printing one. `— PASSED` is
a verdict token. §8.4 lines 2033–2039 already treat two spellings of one engraved
string as a defect class.

**Fix.** Make §1.1's row match §8.4's spelling and drop `— PASSED`, or state
explicitly that §8.4's lines are the legend/field forms and §1.1's is the report row.

### M-10. The running example's transaction spends its own output

> `       TX        9a3f21c0d4e5b6a7f8091a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f`
> …
> `                   9a3f21c0:0   0.05012000 BTC   from node       LIVE`

**Lines 480 and 486.** The single input's outpoint carries the same 8 hex digits as
the transaction's own txid — impossible. (The fee arithmetic on the same block is
correct: 0.05012000 − 0.05000000 = 0.00012000.)

**Fix.** Change the input outpoint's prefix.

### M-11. The example's vsize is impossible for the example's own size

> `       FEE       0.00012000 BTC   (12 sat/vB over 1000 vB)`

**Line 483.** The same block's `CUT   14 strings, 1,228 characters` (line 512) fixes
this transaction at 14 chunks with 89-character full strings — 521–546 raw bytes
(§1e line 645: 535 B → 14 chunks → 89/71; §3b line 1079). vsize can never exceed
serialized size, so 1,000 vB is unreachable; 12,000 sat over ~535 vB is ~22 sat/vB.

**Fix.** "(22 sat/vB over 535 vB)".

---

## Nit

- **N-1, lines 27–30 (§0).** "`mt qr` decides how many symbols that takes, at what
  error-correction level, across how many plates, and what is engraved beside
  them." Present tense, two lines under the table row that strikes the verb through
  as "**DEFERRED out of v0.1 — §0a**".
- **N-2.** Everything the operator reads is 1-based — "14 strings, 1..14 all
  present" (479), "chunk 7" (289), "strings 1-13" (248), `STRING n OF m` (140) —
  while §10.13 a2 line 2818 rules `index` "**plain, zero-based**". The +1 is never
  stated.
- **N-3.** The same ratio is "2%" at line 1082 and "2.2%" at lines 71, 2267 and
  2617 (89/4096 = 2.17%).
- **N-4, lines 1408–1418 (§5).** "**Four fields were cut**" over a three-row table;
  the prose names the "full destination address" as cut but no row carries it.

---

## What I checked and found clean (recorded so the next round does not repeat it)

- **§1e's length table (642–648) is arithmetically exact** at every cell, including
  the last-string remainders, under `hdr 49 + 8·bytes → ceil(/5) + 13 + len("mt1")`:
  33 B → 79/74, 37 B → 85/82, 39 B → 89/71, 40 B → 90/61, 40 B → 90/90, 40 B → 90/55.
- **§3b's chunk counts (1075–1082) are exact** under `ceil(bytes/40)`: 162→5,
  405→11, 535→14, 742→19, 2498→63, 3538→89.
- **1,228 is consistent at every site** (309, 386, 512) and equals 13×89+71.
- **The season projection is sound**: 1,383,520 − 963,759 = 419,761 blocks (line
  2010) × 600 s = 7.98 years from `MT_REF_TIME` 2026-08-23 → **FALL 2034** (2000).
  `MT_REF_HEIGHT` 963,759 and §8.4's "current height 963663" / "current MTP
  2026-08-23T03:00Z" are mutually consistent (96 blocks ≈ 16 h apart).
- **The txid appears twice, identically, 64 hex** (480, 1539).
- **The legend deltas chain correctly**: 141 → 145 (`BROADCAST`, +4, line 1321) →
  164 (`FORMAT`, +19, line 1352), matching the probe's 164 / 7 lines.
- **§10.12's error-budget arithmetic** (2752–2754): 14×4 = 56, ceil(535/45) = 12,
  12×4 = 48.
- **Decision 3 ("The QR carries the standard form, never a codex32 string", line
  728) is NOT a contradiction** of decision 4's "the QR payload is bech32
  uppercase": §3a rules the QR carries chunk bytes under Reed-Solomon with no BCH,
  which is not a codex32 string. Checked and cleared, so it is not re-raised.
- **The `verify`/`decode`/`inspect` stream assignments are consistent** across §0a
  (99–108), §1.1 (516–520), §1a (575–580) and §3b (1179–1184): stdout = artifact,
  stderr = human, `inspect`'s artifact *is* its report.
