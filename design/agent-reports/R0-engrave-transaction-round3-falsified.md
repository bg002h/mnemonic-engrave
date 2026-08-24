# R0 round 3 — `design/SPEC_engrave_transaction.md`

**Scope:** `git diff 6d4d099..HEAD -- design/SPEC_engrave_transaction.md`
(318 insertions / 73 deletions, 6 commits `d255124..90f8dd7`). Two parts: did each
of the six changes land, and what did they falsify elsewhere. **Not a fresh
audit.**

**What I ran, rather than read:**

- `cargo run --bin plate1` — reproduces `0.90 v10 213 B / 0.60 v19 624 B /
  0.45 v28 1190 B / 0.30 v10 12sym 2556 B`, all at ECC M, legend 4 lines = 17.0 mm,
  leaving 62.0 mm.
- Read `src/bin/select2.rs` in full and checked every spec figure against
  `RESULTS_ecc_selection_2026-08-24.txt` and `..._2026-08-22.txt` line by line.
- Three scratch probes of `select2.rs`'s own `best()` over synthetic sizes
  (14,000 / 14,543 / 14,544 / 14,545 / 14,560 / 14,600 / 16,367 B) to find where
  the constraints stop admitting anything. Scratch binaries removed; tree clean.
- Resolved `gui/freetext_proof.go:20-25`, `:24`, `:33` and `gui/freetext_flow.go:33`
  against the fork at `a91df84`.
- `grep` for every stale figure the reorder could have orphaned.

---

## Part 1 — did each change land?

| # | change | verdict | the sentence that settles it |
| --- | --- | --- | --- |
| 1 | **Architect decisions on O11 and O14** | **PARTIAL** | O14 lands clean (§4.2c, §6 S0, §7, §9 all agree), but O11's consequences stop at §3.6b: §5's **R10 still reads "unevaluable for chunks, see §3.6a"**, §2.2a's accepted cost still reads *"the device making no claim whatever about its content — no destination, no amount, **no txid**"*, §3.4's derived/asserted table gained no row for the carried txid, and §6's **P1 and P4 rows name neither the 32-byte field nor R15** while §9 assigns O11 "closed → P1 + P4". |
| 2 | **Structured Append's ordering mechanism (§4.2a)** | **CORRECT** | The 20-bit header table, all three operational consequences, and *"SYMBOL INDEX IS NOT PLATE NUMBER"* land, and the consequence was propagated: §4.3a's per-job instruction now carries *"It MUST say that order does not matter (§4.2a)"*, which the diff added at the same time. |
| 3 | **The search objective reordered (§4.5, §4.5b)** | **PARTIAL** | §4.5's block, §4.5b's rationale and §8's new ruling row match `select2.rs` exactly — constraints discarded at `:137-138`, key `(plates, symbols, Reverse(ecc), Reverse(module), version)` at `:139-145` — but **§4.2a still says *"Criterion 3 is minimise symbol count"* and *"among configurations tied on plates and ECC"***, §6's P5 row names only the 16-symbol cap (not the floor, the reorder or the tie-breaks), and §7 has **no** close condition for the objective at all. |
| 4 | **The plate table regenerated (§4.2a, §4.6)** | **PARTIAL** | Every RAW figure the spec quotes reproduces exactly (162/405/488–595/742 and all five pathological rows), the pathological wallet is present for the first time, and the **624 B ceiling reproduces from `plate1.rs`** — but **the UR half of the file was not computed under the ruled objective**: `best_ur()` (`select2.rs:160-190`) applies no ECC floor, no 16-symbol cap, and ranks ECC *above* symbols, so **18 rows read `ECC L`** under a header that says *"Floor: ECC >= M"*. |
| 5 | **Correcting "prefers small symbols"** | **PARTIAL** | The false sentence was deleted and replaced with a true claim about symbol-count minimisation — but it was written at `c410571`, one commit **before** the reorder at `0238d54`, and the reorder did not propagate into it, so the correction now states the retired ordering. |
| 6 | **3.0 mm face confirmed tested** | **CORRECT** | All four citations resolve against the fork at `a91df84`: `freetext_proof.go:24` is *"the smallest rung and the hardest legibility case"*, `:20-25` is the block quoted in §4.6, `:33` is *"at 3.0mm font/sh is 44 columns and font/constant is 39"*, and `freetext_flow.go:33` independently carries *"44 columns at 3.0mm in font/sh, 39 in font/constant"*. Both §4.5a's and §4.6's citations are right, including the two that point at different files' line 33. |

**Tally: 2 CORRECT / 4 PARTIAL / 0 WRONG.**

---

## Part 2 — what the diff falsified elsewhere

## [I1] The ruled constraints make everything above 14,560 B unengraveable, while §2.3 advertises 16,367 B and §4.1 says the cap never binds

**Severity:** Important (a Critical reading is available; see the last paragraph).

**Where:** §2.3 *"What it buys"*; §4.1 *"The 16-symbol cap does not bind anywhere
in this range"*; §5 (no refusal); §4.2a's NORMATIVE cap paragraph.

**The failure, concretely:** I ran `select2.rs`'s own `best()` over synthetic
sizes at the ruled 0.60 mm. Measured:

```
  14543 B | RAW  17 pl, 16 qr, v24 ECC M @0.60mm
  14560 B | RAW  17 pl, 16 qr, v24 ECC M @0.60mm
  14600 B | RAW  -
  16367 B | RAW  -
```

`-` is `best()` returning `None`: **every configuration was discarded.** The
ceiling is exactly **14,560 B** = 16 symbols x 910 bytes at v24 ECC M, and v24 is
the largest version that fits a bare 79 mm plate at 0.60 mm. Above it the ECC
floor and the 4-bit Structured Append count field leave nothing.

**Why it is now false:** before this diff there was no ECC floor and no hard
16-symbol cap, so the search could always answer — with ECC L, or with more
symbols. The diff made both constraints *discards*. §2.3 says the section-cap
raise **buys "a 16,367-byte raw transaction — 2x the worst measured pathological
spend"**, and that figure is now 1,807 bytes past the point where the plate search
has no answer at all. §4.1's *"The 16-symbol cap does not bind anywhere in this
range — the worst case uses nine"* is true only of the five pathological rows; it
reads as a general reassurance and it is not one. And **§5 has no refusal for
"no configuration satisfies the constraints"** — the spec says what the search
discards and never says what it does when it has discarded everything.

The operator can pack, flash, boot-load, compare the digest, pick the transaction
and pass both confirm screens before this is discovered, because nothing upstream
of the plate search knows about it. The Critical reading is that §2.3 states a
guarantee the design does not meet; I file Important because no wrong artifact is
produced and no funds are at risk — the job simply cannot be planned. The
dangerous secondary path is an implementer who, facing an empty result set,
relaxes the constraint that sounds softest: **dropping the 16-symbol cap produces
indices that cannot be expressed in 4 bits**, i.e. a plate set that looks correct
and never reassembles — which is what §4.2a's *"the cap is not a comfortable
headroom argument"* was written to prevent.

**Confidence:** High for the measurement (run against the committed generator);
High for the §2.3 and §4.1 contradiction; Medium for the relaxation path, which
is inference about an implementer.

---

## [I2] §4.2a states the retired objective — "Criterion 3", and ECC as a tie precondition

**Severity:** Important

**Where:** `SPEC_engrave_transaction.md:844-848`, against §4.5 at `:1122-1126`.

**The failure, concretely:** §4.2a reads:

> Criterion 3 is *minimise symbol count*, so among configurations tied on plates
> and ECC the search breaks toward **fewer** symbols.

§4.5 now reads `1. minimise plates / 2. minimise SYMBOL COUNT / 3. maximise ECC`.
So symbol count is **criterion 2**, not 3, and it is compared **before** ECC, not
after — configurations are never "tied on plates and ECC" before the symbol
comparison runs, because ECC has not been consulted yet. `select2.rs:139-145`
implements §4.5's order, not §4.2a's.

**Why it is now false:** the sentence was authored at `c410571`, when the objective
still ranked ECC 2nd and symbols 3rd — where it was correct. `0238d54` swapped
them the next commit and did not return to §4.2a. An implementer building the
comparator from §4.2a writes the retired one, and §4.2a is the section §6's P5
row points at for the search.

**Confidence:** High.

---

## [I3] Both of §4.3a's measured witnesses are retracted, and the tiling case now has none at all

**Severity:** Important

**Where:** `:1021-1022`, the witness column of §4.3a's three-case table.

**The failure, concretely:**

| §4.3a still says | the regenerated table says |
| --- | --- |
| several symbols on one plate (tiling): **742 B → 6 qr on 2 plates, `4 up`** | `RCW wsh tier1, 1in 742 B | RAW  2 pl, 1 qr, v22 ECC M @0.60mm` — one symbol, no tiling |
| one symbol across two plates: **1,130 B → 2 pl, 1 qr** | `9-of-11 wsh signed, 1in 1130 B | RAW  2 pl, 2 qr, v19 ECC M @0.60mm` — **two** symbols |

Worse for the first row: I grepped the whole CONSERVATIVE (0.60 mm, ruled) section
for tiling annotations — **zero rows carry `( n up)`.** Under the ruled objective
tiling does not occur anywhere in the measured range at the ruled module size, so
the case §4.3a calls out as a fall-through has no measured witness in this document
at all. It survives only at 0.30 mm, which §4.7 says to design against.

**Why it is now false:** the objective reorder collapsed 742 B from 6 symbols to 1
and pushed 1,130 B from 1 symbol to 2. The fold **did** repropagate the second fact
— §4.1's bullet was edited in this same diff from *"measured: 1,130 B is `2 pl,
1 qr`"* to *"measured: **852 B is `2 pl, 1 qr`**"* — and left §4.3a's copy of the
identical claim standing 250 lines away. The normative rule in §4.3a survives; its
entire evidence base does not, and §4.1 now carries the correct witness for one of
the two cases while §4.3a carries the wrong one.

**Confidence:** High.

---

## [I4] Half of `RESULTS_ecc_selection_2026-08-24.txt` was computed under the retired objective, and §4.6 vouches for all of it

**Severity:** Important

**Where:** `src/bin/select2.rs:160-190` (`best_ur`); the file's own section headers;
§4.6 *"What the regeneration now carries"*.

**The failure, concretely:** `best()` applies the rulings —

```rust
if ec_rank(ec) < ec_rank(EcLevel::M) { continue }   // ECC floor      :137
if symbols > 16 { continue }                        // Structured Append :138
let key = |...| (plates, symbols, Reverse(ec_rank(ec)), Reverse(module), v);  :139-145
```

— and `best_ur()`, which produces the entire **UR column of every row in the file**,
applies none of them:

```rust
let better = match &best {
    None => true,
    Some(b) => (plates, std::cmp::Reverse(ec_rank(ec)), symbols)
             < (b.plates, std::cmp::Reverse(ec_rank(b.ec)), b.symbols),
};                                                                    :180-184
```

No floor, no cap, and **ECC ranked above symbols — the retired order.** The result
is visible in the artifact: **18 rows read `ECC L`**, including 15 in the
CONSERVATIVE section, under a header line printed by `main()` that says
*"Floor: ECC >= M. Cap: 16 symbols."* for both columns.

**Why it is now false:** §4.6 states, of the file as a whole, *"What the
regeneration now carries: the ruled objective (symbols above ECC), the **ECC floor
at M**, the **16-symbol cap**, the **module-size and version tie-breaks**…"*. That
is true of the RAW column and false of the UR column. No spec figure I checked is
wrong as a consequence — every number the spec quotes comes from the RAW column —
so this is a false claim about the measurement rather than a false measurement.
It matters because O1/F-243 is still open and the UR column is exactly the evidence
a later encoding decision would reach for. The commit message for `63cbf56` also
states *"ECC L is gone from every row — the floor works"*, which is false of 18 of
them.

**Confidence:** High — read from the generator's source and counted in its output.

---

## [I5] §2.2a's accepted cost ("no txid") and §3.4's derived column both contradict §3.6b

**Severity:** Important

**Where:** §2.2a `:301`; §3.4's derived/asserted table `:672-676`; §3.6b.

**The failure, concretely:** §2.2a says, in bold, as a *ruled accepted cost*:

> **ACCEPTED COST:** a chunks plate is cut with the device making **no claim
> whatever** about its content — no destination, no amount, **no txid**.

§3.6b now has the device display a carried 32-byte txid for a chunks payload, and
R15 has it **refuse** on a mismatch against `chunk_set_id`. Those cannot both be
normative. Meanwhile §3.4 — the only table that defines the derived/asserted split
— lists `txid` under **Derived** with no qualification and **Asserted** as exactly
*"the `TO` label, the fee"*, and §3.6b instructs that the carried txid *"belongs in
§3.4's asserted column, beside `TO` and the fee"* — a column it was not added to,
in a section whose heading is *"Comprehend, then cut — **the raw form**"*, i.e.
scoped to the form that never carries it.

So the carried txid has no table anywhere, and §7's close condition *"the chunks
picker renders the txid in the ASSERTED voice, not the derived one (§3.4)"* points
at a section that says the opposite and does not cover chunks.

**Why it is now false:** §2.2a's cost was accepted when O11 was open and the device
genuinely had nothing to say about a chunks payload. O11's resolution gave it
something to say, and neither the accepted-cost paragraph nor §3.4's table was
revisited.

**Confidence:** High for the §2.2a contradiction (literal text); High for §3.4's
missing row.

---

## [I6] O11's deliverables have no phase row — the mechanism exists only in tables

**Severity:** Important

**Where:** §6's P1 and P4 rows `:1394-1397`; §9's O11 row; §2.1b's NORMATIVE.

**The failure, concretely:** §9 records O11 as `closed → P1 + P4`. §2.1b says
**"NORMATIVE: P1 defines it before anything reads it"**, listing the carried txid
as one of the things P1 must state. §7 requires *"The carried txid and R15 are
implemented (§3.6b)"*. But §6's rows read:

- **P1** — *"`ClassTransaction`, the framed record, stdin, content-based sealing,
  `MaxSectionLen` → 32,734 — with vectors"*. No txid field.
- **P4** — *"The payload menu … the program (§3.4–3.7); ALL FOUR lockstep sites"*.
  No R15, no chunks picker, no carried-txid display.

Every other thing this spec cares about is named in its phase row — the boot-path
call, the four lockstep sites, the legend reorder, the computed reservation, the
`tx:` branch in `scan.go`, the SA work over `coding`. O11's are the exception.

**Why it is now false:** the diff created these deliverables and updated §7 and §9
but not §6. This spec states the rule against itself in §3.1a: *"a mechanism
mentioned only in a table is a mechanism nobody schedules"*, and it was written
there because P4's gate had previously been satisfiable while the ruled behaviour
stayed untrue.

**Confidence:** High.

---

## [I7] R10 was written for a derived key and now runs on an asserted one

**Severity:** Important

**Where:** §5's R10 row; §3.6's opening; §3.6b.

**The failure, concretely:** two halves, one locus.

1. §5's R10 still reads *"two identical txids in one payload … **unevaluable for
   chunks, see §3.6a**"*. §3.6b resolved exactly that: the txid is carried, so the
   device *can* compare two chunks records. The refusal table still records the
   problem O11 closed.
2. More seriously, R10's remedy is *"a duplicate to refuse or **collapse**, never
   two picker entries"*, and §3.6 justifies it by calling the txid *"the
   **derived**, collision-free identifier"*. For a chunks payload it is neither —
   §3.6b says so in bold two subsections later. **Two distinct chunks records whose
   records carry the same 32 bytes would be collapsed**, and the operator would
   cut one transaction believing they had backed up two. R15 cannot catch it: R15
   compares a carried txid against `chunk_set_id`s **inside the same record**, and
   both records would pass their own check independently.

**Why it is now false:** R10 and §3.6's *"derived, collision-free"* predate the
carried txid. The diff introduced an asserted key and retired R14 (whose whole job
was to keep multiple chunks-form transactions out of one payload — the exact
situation R10's collapse branch now handles) without revisiting either.

**Confidence:** High for half 1 (literal stale text); Medium for half 2 — it
requires two records carrying the same txid, which correct `mt` output will not
produce, so the exposure is a malformed or hostile payload rather than a normal one.

---

## [I8] "with no decoder at all" is the fourth instance of the claim class this spec exists to catch

**Severity:** Important

**Where:** §3.6b: *"The device reads that field off **every chunk** with no decoder
at all."*

**The failure, concretely:** `chunk_set_id` is a 20-bit field inside a bech32 data
part. Reading it needs the codex32 charset table, symbol decoding of at least the
leading characters of the data part, and bit-slicing across symbol boundaries at
the 37-bit header layout §2.3 cites. That is new device code. §2.2a already
established the neighbouring case with a measurement: validating a chunk's own BCH
checksum *"is a new `ValidMT` over the shared GF engine, **not a call to an existing
predicate**"*, because `codex32.ValidMD`/`ValidMK` hard-code the `md`/`mk` HRPs and
targets.

**Why it is now false:** §3.6a's weaker phrasing — *"`chunk_set_id` … **yes** — read
off the string"* — was defensible as "no *transaction* decoder needed". §3.6b
strengthened it to *"no decoder at all"* while making it load-bearing for a
**refusal**. This spec's own header says three sentences it asserted were
measurably wrong and two more described reuse that does not exist; C2 (Critical)
and M1 were both this exact shape, and §2.2a's rule is that the chunks path *"may
not describe … needing 'nothing new'"*. Combined with I6 — no phase row names it —
the phrasing is what would stop it being scheduled.

**Confidence:** Medium-High. The design is sound; the claim about its cost is not,
and the cost is unowned.

---

## [M1] A dangling citation to the superseded results file

**Severity:** Minor. **Where:** `:848-851`.

`§4.2a` reads *"Measured at 0.60 mm (`RESULTS_ecc_selection_2026-08-22.txt`):"* —
a colon introducing a table that is not there, immediately followed by a second
heading *"MEASURED under the ruled objective (`RESULTS_ecc_selection_2026-08-24.txt`
…)"* and the actual table. A fold artifact: the new heading was inserted above the
old table without removing the old lead-in, and the surviving citation points at
the retired measurement.

**Confidence:** High.

---

## [M2] §4.2a's round-2 blockquote asserts the retired objective in the present tense

**Severity:** Minor. **Where:** `:832-838`.

> §4.5's ruled objective ranks **maximise ECC ABOVE minimise symbol count**, so the
> search deliberately produces *many small* symbols: this spec's own measured case
> is **742 B → 6 symbols**, about **124 B each — 11x smaller** than the divisor
> assumed.

Both clauses are now false: §4.5 ranks symbols above ECC, and 742 B is measured at
**1** symbol. It is a blockquote recording round 2's I2, so it is history — but it
is written in the present tense about §4.5, and it is the sole support for the
paragraph's *"THE 16-SYMBOL CAP CAN BIND"* heading. The NORMATIVE requirement below
it survives on the 4-bit field alone, so nothing normative falls; the argument for
it is retracted. (I1 supplies a replacement argument that is stronger.)

**Confidence:** High.

---

## [M3] "STILL OWED — one item" followed by "the two owed items above"

**Severity:** Minor. **Where:** §4.6 `:1290` and `:1300-1301`.

`90f8dd7` reduced the owed list from two items to one — the heading and the
numbered list both say one — and left the closing sentence reading *"subject to the
two owed items above, **both** of which move the table in a stated direction."*
Introduced and falsified by the same commit.

**Confidence:** High.

---

## [M4] §4.3's example screen depicts a configuration that no longer exists

**Severity:** Minor. **Where:** `:998-1010`.

The MULTI-SYMBOL screen reads `PLATE 2 OF 6` and *"this is 1 of 6 symbols"* — the
same 6 for both counts. In the regenerated table **every** multi-plate RAW row at
0.60 mm has symbols = plates − 1 (2/1, 3/2, 6/5, 10/9, 17/16), because the legend
reservation costs plate 1 its symbol. So a 6-plate job has 5 symbols and the screen
depicts an impossible pairing — while modelling exactly the conflation §4.1 warns
about (*"Plates, symbols and tiling are three different counts and the spec must
not conflate them"*) and §4.3a exists to forbid. Under the old objective 742 B →
6 symbols made 6 a real symbol count; it no longer is one anywhere.

**Confidence:** High.

---

## [M5] §4.5b's interaction note is written as pending after §4.6 declares it done

**Severity:** Minor. **Where:** §4.5b's blockquote `:1174-1181`.

> **AN INTERACTION THAT PARTLY CANCELS, and both halves must land in the same
> regeneration (§4.6).** … the measured 488–535 B cases chose **ECC L**, which the
> floor now forbids, so they must find that capacity elsewhere — a larger version,
> another symbol, or another plate. Neither change can be evaluated without the
> other, **and the old table has neither.**

The regeneration ran. The interaction resolved, and to the best of the three named
outcomes: 488 B went v15 L → v17 M, 501 B v15 L → v17 M, 535 B v16 L → v18 M, all
still **one plate, one symbol**. The note reads as an open question about work that
§4.6 records as DONE three subsections later, and the resolution is more reassuring
than the note.

**Confidence:** High.

---

## [M6] The legend character count and round 2's I7 both drifted

**Severity:** Minor. **Where:** §4.5a's table `:1216-1221` and its I7 paragraph.

Two small drifts from the regeneration:

- The table row is labelled *"packed, all five fields (**153 chars**)"*, while §4.6
  and `select2.rs:44` use **167** (`LEGEND_CHARS_FIRST = 153.0 + 14.0`). The row's
  numbers (4 lines / 17.0 mm / 62.0 mm / v19) are right for both, so nothing
  downstream is wrong — the label is stale.
- Which is because R0 round 2's I7 claim, still standing verbatim, is now
  measurably false: *"the computed reservation **under-charges every multi-plate
  job**"*. `ceil(153/44) = 4` and `ceil(167/44) = 4`. The sixth field costs zero
  lines at the 3.0 mm face, so the under-charge is nil. The NORMATIVE fix it
  produced (per-plate reservation) is right and is what `select2.rs` implements;
  the stated magnitude is not.

**Confidence:** High.

---

## [M7] `plate1.rs` is cited as authority with no persisted output, and its comparator is partly dead

**Severity:** Minor. **Where:** §4.2a `:867-877`; `src/bin/plate1.rs`.

The one-plate ceiling table is the only measured table in this spec cited to a
**binary** rather than a `RESULTS_*` file — `63cbf56` added `plate1.rs` and
`RESULTS_ecc_selection_2026-08-24.txt` but no results file for `plate1`, so its
four numbers exist in the spec and the commit message only. This is the same rot
path §4.6 explicitly guards against for `select.rs` (*"kept unchanged, so the
earlier measurement stays reproducible"*).

Separately, `plate1.rs`'s comparator does not implement its own stated objective:

```rust
// objective: max bytes, then FEWEST symbols, then strongest ECC
let key = (total, std::cmp::Reverse(tiles), ec as u8);
if best.map_or(true, |(bv, bec, bt, bb)| key > (bb, std::cmp::Reverse(bt), bec as u8) || (bv, bec) == (bv, bec) && false) {
    if best.map_or(true, |(_, _, _, bb)| total > bb) { best = Some((v, ec, tiles, total)); }
}
```

`(bv, bec) == (bv, bec) && false` is unconditionally false, and the inner `total >
bb` guard overrides the outer key comparison entirely — so the search is
strictly max-bytes with first-encountered winning ties, and the two stated
tie-breaks never run. **I re-ran it: the four printed rows are unaffected**, and
624 B is correct (v19 ECC M data capacity is 627 codewords less the 20-bit mode
and count header = 624; the v40 row's 2,331 B matches the published table
exactly). Recorded because the spec cites this file as the ceiling's provenance.

**Confidence:** High (both halves run/read directly).

---

## [M8] The only capacity gate in §7 is pinned at an ECC level the objective now discards

**Severity:** Minor. **Where:** §7, *"The mode-segmentation gate is green"*.

> Any QR sizing MUST assert measured v40 capacity against **numeric 7089 / alnum
> 4296 / byte 2953 at L**.

Those three figures are correct for v40-L. But ECC L is now a *discarded*
configuration everywhere in the product, so the sole capacity assertion in the
close conditions exercises a level the search may never select, and nothing asserts
the capacity figures at M — the level every measured row actually uses. The gate
still tests the library's mode segmentation, which is its purpose; it no longer
tests it at a level the device will ship.

**Confidence:** Medium — the gate's intent is library correctness, so this is a
coverage observation rather than a defect.

---

## [M9] Two small table drifts in §5

**Severity:** Minor. **Where:** §5's refusal table.

- The retired **R14** row has three cells against a four-column header
  (`#`, refusal, why, §), so *"RETIRED. …"* renders under **refusal** and the `—`
  under **why**, leaving § blank. (R6 has the same shape and predates this diff.)
- **R13** cites *"§4.2a's two gates"*; §4.2c now owns and defines those two gates,
  and §7's close condition says *"§4.2c's TWO Structured-Append gates"*. The
  cross-reference did not follow the content when `d255124` moved it.

**Confidence:** High.

---

## [M10] §2.1's definition sentence predates the mandatory txid

**Severity:** Minor. **Where:** §2.1 `:166-168`.

> `ClassTransaction` is one record carrying the transaction (or its chunks) **and**
> the legend fields `mt encode` already computes.

§2.1b now adds *"THE CHUNKS FORM CARRIES A MANDATORY 32-BYTE TXID"*. §2.1 is the
section that names what the record is; the field is stated only in the subsection
about the layout being undefined.

**Confidence:** High.

---

## Noted but NOT falsified by this diff — pre-existing

**§7 says "All THREE program-keyed lockstep sites carry the new program (§3.1a)"**
(`:1430`) while §3.1a's own NORMATIVE says *"all four are lockstep sites"*
(`:507`) and §6's P4 row says *"ALL FOUR lockstep sites"* (`:1397`). I checked
`git diff 6d4d099..HEAD` — **no line matching THREE/FOUR/all-four appears in the
diff at all**, so this came from the round-2 fold and is outside this round's
scope. Recorded because it is an open contradiction in a close condition, and
because §7's own next clause (*"the two that fail silently"*) is also a
three-site sentence, while §3.1a lists **three** silent sites of four.

---

## Verdict

**Part 1: 2 CORRECT / 4 PARTIAL / 0 WRONG.**

**Part 2: 0 Critical / 8 Important / 10 Minor** (plus 1 pre-existing, out of scope).

The gate does not close. The dominant failure mode in this diff is not a wrong
claim but **incomplete propagation of a right one**: the same fact was fixed in
§4.1 and left standing in §4.3a (I3), the objective was reordered in §4.5 and left
retired in §4.2a (I2), O11 was closed in §3.6b/§7/§9 and left contradicted in
§2.2a/§3.4/§5 and unscheduled in §6 (I5, I6, I7). Two of the six commits falsify
text written by the other four. **I1 is the only finding that is not propagation
— it is a consequence nobody has looked at**, and it is the one the reorder
created rather than exposed.

### What I did NOT examine

- **§2.3's arithmetic and the chunk-character formula** — declared settled, and I
  did not re-derive them. I *did* use §2.3's 16,367 B figure as an input to I1.
- **`SPEC_mt_v0_1.md`** beyond confirming `:943` is the settled `chunk_set_id`
  fact, which I took as given.
- **The `kortschak-qr` `coding` package surface** — `NewPlan(version, level, mask)`
  and the absence of Structured Append were declared settled and I did not re-verify.
- **`JOURNEY_WALK_engrave_transaction.md`** and the three prior R0 reports — I did
  not check whether any earlier finding was re-opened by this diff.
- **§2.4, §2.5, §3.2, §3.3, §4.4/§4.4a, §5's R1–R9/R11′/R12, §10** — read for
  falsification only. Their own citations into the fork were not re-resolved,
  since the diff does not reach them.
- **The 0.30 mm AGGRESSIVE column** beyond the four rows §4.2a's S0 blockquote
  quotes (all four verified). I did not check whether I1's ceiling has a 0.30 mm
  analogue, which would matter if F-234 validates.
- **`FOLLOWUPS.md`** — F-225/234/235/242/243 numbering was taken from the spec.
