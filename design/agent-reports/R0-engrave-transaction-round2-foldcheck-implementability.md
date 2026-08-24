# R0 round-2 — fold-check of the round-1 fold, + implementability of the new normative content

**Agent:** R0 round 2, opus. **Date:** 2026-08-24.
**Scope:** (1) did `git diff 321cba6..8290415 -- design/SPEC_engrave_transaction.md`
land the six items round 1 filed; (2) a fresh **implementability** lens over the
text the round-0 and round-1 folds ADDED. Not a fresh audit.

**Machine-checked before writing:** every code citation below was resolved
against the fork at `/scratch/code/shibboleth/seedhammer` (`a91df84`) and the
`qr` dependency at `~/go/pkg/mod/github.com/seedhammer/kortschak-qr@v0.3.2`.
Go is not installed on this box, so nothing was compiled or run; every claim
about behaviour is traced from source, and I say so where it matters.

---

## Part 1 — did the round-1 fold land?

| item | verdict | the sentence that settles it |
| --- | --- | --- |
| **C1** (§4.3 asked for `mt inspect` per plate) | **FIXED** | §4.3 now carries two mock-ups and a normative rule that names the exact prohibition round 1 said was absent: *"**The device MUST NOT ask for `mt inspect` on a partial set.** Doing so reports failure on correct work and teaches the operator to stop testing."* The screen text round 1 quoted as "byte-for-byte unchanged" is gone. (What the new branches do **not** cover is Part 2, I6 — a different defect, not a survival of this one.) |
| **I4** (§2.3 item 2 still demanded "which transports its output fits") | **FIXED** | §2.3 item 2 now quotes the old sentence as a superseded draft — *"So the sentence an earlier draft had here … is close to meaningless: a container has exactly one transport"* — and replaces it with the split obligation (`me sysw pack` states the section cap; `mt encode --record` states NFC fit). R6's row already carried the same correction. |
| **I5** (nothing blocked a multi-transaction chunks payload) | **FIXED** | R14 exists — *"a payload holding MORE THAN ONE chunks-form transaction"* — and §7 gained *"**O11 is resolved, OR R14 refuses a multi-transaction chunks payload.** … One of the two must hold before P4 closes."* The hazard round 1 constructed is now reachable only past a stated gate. (Whether R14 is **writable** is Part 2, I5.) |
| **new Important** (three lockstep sites, not one) | **PARTIAL** | §3.1a is fully rewritten and its three-row table is **correct and exhaustive** — I re-ran the enumeration type-first rather than by the spec's own grep and found no fourth `program`-keyed site (below). But the fold did not propagate to **§6**, whose P4 row still reads *"the program (§3.4–3.7); **`layoutMainPlates`' case list (§3.1a)**"* — naming neither `uiFlow` nor `StartScreen.draw`, and assigning `layoutMainPlates` to **P4** while the section it cites now says *"P4 must touch `uiFlow` and `StartScreen.draw`; **P5** must touch `layoutMainPlates`."* That P4 line is the exact sentence round 1 quoted in its own finding. See Part 2, I3. |
| **new Minor** (`StartScreen.draw`'s title switch) | **FIXED** | §3.1a's table row: *"`StartScreen.draw`'s title \| `gui/gui.go:2186` \| **blank title** on the carousel page \| **SILENTLY**"*. Line verified: `switch m.prog {` is at `gui/gui.go:2186`, no `default` arm, `titleTxt` zero value `""`. |
| **new Minor** (§4.2a argued the cap with a larger bound) | **PARTIAL** | The **cited bound** is fixed — §4.2a now says *"the bound that shows it is **this spec's own container cap, not Bitcoin's standardness limit**"* and derives 16,367 B from §2.3 — and the retraction is recorded in a blockquote. But the fold changed the numerator's source and left the denominator: the conclusion still rests on **1,367 B per symbol**, which is the *maximum-capacity* symbol, while §4.5's ruled objective deliberately chooses smaller ones. The file §4.2a cites for 1,367 B carries a note warning about exactly this. See Part 2, I2. |

**Tally: 4 FIXED / 2 PARTIAL / 0 NOT FIXED.**

**The re-grep round 1 asked for, done type-first rather than pattern-first.**
The spec states its method as `grep "switch act.prog|switch m.prog|switch page"`
— an enumeration whose alternatives are the three sites already known, which
cannot find a fourth (Part 2, M2). Redone from the type: `type program int`
(`gui/gui.go:175`) is consumed by exactly five things outside tests —
`layoutMainPlates(buf, page program)`, `layoutMainPager(buf, th, page, lastNav
program)`, `StartScreen.prog`, `startScreenAction.prog`, and
`StartScreen.lastNav() program`. `grep -n "switch .*prog\|switch page\|switch p
{"` over `gui/*.go` non-test returns five switches; two of them
(`gui/gui.go:3507` over `PlateSize`, `gui/plate_verify.go:52` over
`verifyProvenance`) are not over `program`. **§3.1a's three rows are the whole
set, and the line numbers 2029 / 2186 / 2430 are correct.** `layoutMainPager`'s
numeric consumption is correct too (`npages := int(lastNav) + 1`, and `if i ==
int(page)` for the filled dot — both numeric).

I also checked the one adjacent thing §3.1's "the pager machinery is untouched"
could have been wrong about, since F-154 was filed when the dot row grew into the
version label: at 12 navigable programs the pager is `(13+4)*12-4 = 200 px`
against a `room` of roughly `480 - 174 - 8 = 298 px`, so the guarded branch at
`gui/gui.go:2239` still takes the fitted path and
`TestStartScreenPagerDoesNotCollideWithTheVersion` should still pass. **Not a
finding** — recorded so a later round does not spend the same hour.

---

## Part 2 — implementability of the new normative content

### [I1] O12 is not merely unverified — it is REFUTED, and no phase owns building the mechanism S0 is scheduled to test first

**Severity:** Important

**Where:** §4.2a gate 1 and §9 O12 (*"does our QR encoder emit **Structured
Append**? §4.2a gate 1 — unverified"*, owner **"S0 / P5"**); §6's S0 row, which
requires cutting *"**a Structured-Append pair**"*.

**The failure, concretely:** the fork's QR encoder is
`github.com/seedhammer/kortschak-qr v0.3.2` (`go.mod:13`, imported as `qr` at
`gui/gui.go:22`, `backup/backup.go:11`, `engrave/engrave.go:15`). It has **no
Structured Append and no way to reach it through its public entry point**:

- `qr.Encode(text string, level Level)` (`qr.go:33`) is the only top-level
  encoder. It picks one segment from `coding.Num` / `coding.Alpha` /
  `coding.String`, sizes one symbol, and returns one `*Code`.
- `coding/qr.go` defines exactly three `Encoding` implementations, writing mode
  indicators `1`, `2` and `4` (`b.Write(1, 4)`, `b.Write(2, 4)`, `b.Write(4, 4)`
  at `coding/qr.go:140`, `:183`, `:216`). **Mode `3`, Structured Append, is
  absent.** `grep -rni "structured" .` over the whole module returns nothing.
- `engrave.QR(strokeWidth int, scale int, qr *qr.Code)`
  (`engrave/engrave.go:277`) takes **one** `*qr.Code`, and `backup.Paragraph`
  (`backup/backup.go:65-69`) has **one** `QR *qr.Code` field. There is no
  multi-symbol plate structure either.

It is *achievable* — `coding.Encoding` is an exported interface and
`(*coding.Bits).Write(v uint, nbit int)` is exported, so a caller can write the
`0011` header, a 4-bit index, a 4-bit count and the 8-bit parity ahead of a byte
segment, and drive `coding.NewPlan` / `Plan.Encode` directly. But that means
**re-implementing `qr.Encode`'s version-selection loop in the fork**, and
**nothing in §6 assigns that work to a phase.** P5's row is *"the plate: search,
the computed legend reservation, the legend-emission REORDER, test-the-plate,
plate count"* — an encoder is not in it. O12's stated owner, "S0 / P5", is
circular for S0: **S0 is ordered first** (*"**S0 first is the closure rule
applied rather than quoted.** Four of this design's gates are hypotheses and S0
is two seconds of machine time each"*) and its deliverable includes *"a
Structured-Append pair"* — which cannot be cut, because nothing can encode one.

**Why the spec permits it:** O12 is filed as a question about a *fact*
("does our encoder emit it?") rather than as a *work item*, so it lands in §9
with an owner and never in §6 with a phase. The round-0 fold ruled the mechanism
(§4.2a) and gated on it (§7, R13) without asking who builds it. The R0 rule this
trips is the spec's own: *"a plan may not close while any of its own gates has
never been run"* — S0 is that gate, and as sequenced it cannot run.

**Confidence:** **High** on the refutation (three independent checks: the public
API, the mode-indicator enumeration, a whole-module grep). High that no §6 row
names the work. Medium on severity — the spec does gate on O12, so this is an
unowned and mis-sequenced work item rather than a false claim.

---

### [I2] §4.2a's "the 16-symbol cap is not a constraint" divides by the LARGEST symbol the search can pick, while §4.5's ruled objective picks the smallest — and the file it cites says so in a note

**Severity:** Important

**Where:** §4.2a, *"Structured Append gives 16 × 1,367 B ≈ **21.9 KB** at a
full-area v26. The largest transaction that can reach the device at all is
**16,367 B** … So the transport runs out before the symbol count does."*
Against §4.5's objective (*"1. minimise plates … 2. maximise ECC 3. minimise
symbol count"*) and §4.1 (*"the search **prefers** several small symbols over one
large one when that buys ECC"*).

**The failure, concretely:** 1,367 B is the *maximum* bytes a 0.60 mm symbol can
hold — one v26 filling a whole 79 mm plate at ECC L, with no legend and no
tiling. The ruled objective ranks **maximise ECC above minimise symbol count**,
so once the plate count is fixed the search spends the slack on ECC by tiling
more, smaller symbols. The spec's own measured witness, quoted in §4.1:

```
RCW wsh tier1, 1in           742 B | RAW  2 pl, 6 qr, v9 ECC Q @0.60mm (4 up)
```
(`design/measurements/RESULTS_ecc_selection_2026-08-22.txt:15`)

That is **~124 B per symbol, an 11× smaller divisor than 1,367**. At that rate
the 16-symbol cap binds at **~2.0 KB** — below the `5/2` (4,080 B) and `10/2`
(8,067 B) rows of §2.3's own table, and below the pathological worst case
§4.2a cites in the same sentence as reassurance.

And the file §4.2a and §4.1 both cite for the 1,367 B figure already carries the
warning, verbatim:

```
  NOTE: tiling beyond 16 symbols exceeds QR Structured Append's limit and would
  need a scheme of mt's own. Counts above are unconstrained.
```
(`design/measurements/RESULTS_qr_physical_max_2026-08-22.txt:24-25`, in the
tiling section that also gives `0.30  v26  38.7mm  2x2` — four symbols per
plate, and 0.30 mm is still live as O2.)

**Neither §4.5's search space nor its objective carries a ≤16 constraint**, and
no refusal covers "the search's optimum needs more than 16 symbols". R13 refuses
a multi-symbol job *"when Structured Append is unavailable"*, which an
implementer reads as gate 1/gate 2, not as "this particular job overflows the
index field". So an implementer who builds §4.5 exactly as written, having been
told in §4.2a that the cap is not a constraint, has no reason to bound the
search — and the 4-bit index and 4-bit count fields have nowhere to put a
17th symbol.

**Why the spec permits it:** the round-1 fold corrected the *numerator's* source
(100 KB → §2.3's 16,367 B) and inherited the *denominator* unchanged. The
finding round 1 filed was about which ceiling was cited, so the fold re-derived
the ceiling and not the per-symbol capacity — the same instance-not-class shape
§3.1a's own blockquote is about, one section away from where it is written.

**Confidence:** **High** that the bound as written is unsound and that the cited
source contradicts it. **Medium** that a real search would actually exceed 16 for
an in-range transaction: the measured tool's own outputs top out at 6 symbols in
that file, so this is a soundness defect in the argument rather than a
demonstrated overflow. Nothing in the spec makes the demonstrated-6 the ceiling.

---

### [I3] §6's P4 row was not propagated from the rewritten §3.1a — and read literally, P4 closes green on a device that panics when the operator pages onto the new entry

**Severity:** Important

**Where:** §3.1a's closing line, *"P4 must touch `uiFlow` and
`StartScreen.draw`; **P5** must touch `layoutMainPlates`."* Against §6's P4 row,
*"The payload menu (§3.3) **and the boot-path call that invokes it**; the program
(§3.4–3.7); **`layoutMainPlates`' case list (§3.1a)**"*.

**The failure, concretely:** three statements, three different assignments.

| source | who owns `layoutMainPlates` | who owns `uiFlow` / `StartScreen.draw` |
| --- | --- | --- |
| §3.1a | **P5** | P4 |
| §6, P4 row | **P4** | not named |
| §7 closure list | unphased (*"All THREE … carry the new program"*) | unphased |

The §3.1a reading is the dangerous one. `engraveTransaction` is *"inserted
mid-enum before `loadPayload`"* (§3.1) and P4 is *"the program"*, so the carousel
entry exists at the end of P4. `layoutMainPlates` (`gui/gui.go:2429`) is an
explicit case list ending in `panic("invalid page")` with no compile-time guard —
§3.1a says so itself. **So a P4 that follows §3.1a ships a build where paging
onto the new carousel entry panics, and P4 has its own gate.** §7's backstop
bullet carries no phase, so it cannot stop P4 closing.

**Why the spec permits it:** the round-1 fold rewrote §3.1a in place and did not
touch §6 — the diff `321cba6..8290415` contains no §6 hunk. §6's P4 row is the
sentence round 1 quoted in its own finding (*"P4's sequencing line — 'The payload
menu (§3.3) and the boot-path call that invokes it; the program (§3.4-3.7);
`layoutMainPlates`' case list (§3.1a)' — could be read as covering it under 'the
program'"*), so the fold read that line, answered the finding elsewhere, and left
the line itself standing — now not merely incomplete but contradicting the
section it points at.

**Confidence:** **High.** Textual, both sentences quoted in full; the panic and
the absent guard are verified at `gui/gui.go:2429-2437`.

---

### [I4] The `tx:` branch §2.1a makes normative is not sufficient: `engraveObjectFlow`'s type switch is a fourth silently-defaulting enumeration, on the exact path §2.1a exists to fix

**Severity:** Important

**Where:** §2.1a's NORMATIVE line (*"adding `tx:` to `isSyswEncoded` **without**
adding a matching branch beside the `PassPrefix` one is the defect. The branch is
the work; the prefix is not."*) and §6's P3 row (*"**Includes the `tx:` branch in
`gui/scan.go` (§2.1a)**"*). Against `engraveObjectFlow` at
`gui/gui.go:2467-2491`.

**The failure, concretely:** the branch §2.1a asks for returns a scan object.
`gui/scan.go:56-80` returns `passScan(body)` or `freeTextScan(body)`; a `tx:`
branch returns whatever §1.2 calls *"§3.3's `txScan` case"*. That value reaches
`engraveObjectFlow`, which is a **type** switch ending in:

```go
	default:
		return false
	}
	return true
```

and `uiFlow` turns `false` into `s.Status = scanUnknownFormat` (`gui/gui.go:2072`).
So an implementer who does exactly what §2.1a's NORMATIVE sentence and §6's P3
row say — add the prefix, add the branch in `gui/scan.go` — gets a device that
answers a well-formed `tx:` tag with **"unknown format"**. That is
byte-identical to the failure §3.1a calls *"the program's own front door … nothing
crashed, nothing logged"*, and it is on the NFC path, which §1.2 says has no
digest compare and no payload menu — i.e. the only path where the operator has no
second signal that anything was loaded at all.

`txScan` itself is named exactly once in the document, in a **table cell** in
§1.2 (*"enters via `engraveObjectFlow` (§3.3's `txScan` case)"*), and §3.3 does
not define it. It appears in no NORMATIVE sentence, in no §6 row, and in no §7
gate.

**Why the spec permits it:** §3.1a scoped its class as *"program enumerations
that must move in lockstep"* and grepped for `switch … prog`. `engraveObjectFlow`
switches on `obj any`, so a `prog`-shaped grep cannot see it — even though its
failure mode is the same `scanUnknownFormat`, and even though §3.1a's own
blockquote says the fix is *"to grep for the class"*. The class is
"enumerations whose `default` is silent", and it has four members, not three.

**Confidence:** **High** on the mechanism — `engraveObjectFlow`'s `default:
return false` and `uiFlow`'s `scanUnknownFormat` were both read in full, and
`gui/scan.go:56-80` matches the block §2.1a quotes. High that the spec never
states the `engraveObjectFlow` case as work.

---

### [I5] `ClassTransaction`'s record body is never defined — so P1, P3, R1, R4, R14 and §3.4's "asserted" column have nothing to implement against, and Rust and Go must agree byte-for-byte

**Severity:** Important

**Where:** §2.1 (*"`ClassTransaction` is one record carrying the transaction (or
its chunks) **and** the legend fields `mt encode` already computes. One record,
not siblings, so the legend stays bound to what it describes."*) and §6's P1 row
(*"`ClassTransaction`, **the framed record**, stdin, content-based sealing …
**with vectors**"*).

**The failure, concretely:** `ClassTransaction` occurs three times in the
document (`§1`'s ownership table, §2.1, §6's P1 row) and is **defined nowhere**.
No design file in `design/` defines it; there is no `TxPrefix` in the fork's
`sysw/`. What exists for comparison — `sysw/record.go:14-15` — is two prefixes
whose bodies are *unstructured* hex payloads (`TextPrefix`, `PassPrefix`), with
`DecodeBody` returning bare bytes. **`ClassTransaction` is the first class that
needs interior structure, and the spec never says it has any**, let alone what.

Four normative statements branch on fields that framing would have to define:

- **R1** constrains only the outer hex envelope (*"a `tx:` record whose body is
  not lowercase hex"*), which `sysw.DecodeBody` already enforces. It says nothing
  about what the decoded bytes are.
- **R4′** refuses *"a **single `tx:` record** carrying both forms"* — which
  requires a form field a reader can see.
- **R14** refuses *"a payload holding MORE THAN ONE **chunks-form** transaction"*
  — the device must classify each `tx:` record as raw-or-chunks **without a
  decoder**. §3.6a's own availability table says `chunk_set_id` is *"read off the
  string"*, so this is reachable in principle — but only once something states
  where the form and the chunk strings live in the body.
- **§3.4's Asserted column** (*"the payload's legend fields; the operator's
  words"* — the `TO` label and the fee) and §4.5a's five-field legend both read
  named fields out of that body.

Because the container is **Rust-primary** (§1, *"The container lands in `me` with
vectors first and reaches the fork as a port"*, with `check-provenance.sh` in
§7's gates), the Rust and Go readers must agree on that framing **byte for
byte** — and a vector set (P1's *"with vectors"*) cannot be written for a format
nobody has specified.

**Why the spec permits it:** §2.1's sentence reads as a *design ruling* — one
record rather than siblings — and the word "framed" in P1's row carries the whole
weight of the format without expanding it. Every subsequent section consumes the
fields as if they were already defined, so no later section is the natural place
to notice they are not. Round 0's C3 and round 1's Important both concerned the
record's **routing**, which drew attention past its **contents**.

**Confidence:** **High** on absence (grep over `design/` and the fork's `sysw/`).
High that R4 and R14 cannot be written without it. This is the one finding here
that is about text the folds *did not* add — but R14 is new in this fold, and R14
is what makes the gap load-bearing rather than latent.

---

### [I6] §4.3's two branches are keyed on SYMBOL count and instruct per PLATE — and the spec's own measured cases break the implied 1-symbol-per-plate mapping in both directions, plus the chunks form has no branch at all

**Severity:** Important

**Where:** §4.3's two mock-ups and the three NORMATIVE bullets under them.
Against §4.1's own warning: *"**Plates, symbols and tiling are three different
counts** and the spec must not conflate them."*

**The failure, concretely:** three cases the two branches do not cover, each with
a measured witness already in this repo.

1. **A plate carrying several symbols.** Measured: `742 B | RAW 2 pl, 6 qr, v9
   ECC Q @0.60mm (4 up)` — four symbols on one plate. §4.3's normative bullet
   reads *"**Per plate**, the operator checks only that **the symbol** scans"*,
   singular, and the mock-up prints *"this is 1 of 6 symbols"* under a header
   reading *"PLATE 2 OF 6"*. An operator who scans **one** symbol on a 4-up plate
   passes a plate carrying three unchecked symbols — and the whole point of the
   per-plate step is that *"that is the failure mode a cut introduces"*. The
   failure surfaces only after the last plate, i.e. after every plate has been
   cut, which is the cost §4.3 exists to avoid.
2. **A plate carrying no symbol.** Measured, and quoted in §4.1 itself: *"a plate
   may hold **one** symbol yet still be the *second* plate, because the legend
   reservation pushed it there (measured: 1,130 B is `2 pl, 1 qr`)"*. That job
   has **one** symbol, so it takes §4.3's **single-symbol branch**, whose screen
   says *"TEST IT NOW … **Scan the QR**, then run `mt inspect` on what you get"*
   — printed after **both** plates, one of which has no QR on it.
3. **A text (chunks) job.** §2.2 rules chunks as one of the two payload forms and
   §2.2a makes the text-only plate builder normative. A chunks job has **zero**
   symbols, so neither branch applies: both mock-ups say *"Scan the QR"* /
   *"check it READS"*. Nothing in §4.3, §4.4 or §6 says what the device prints
   after cutting text plate 7 of a chunks job — and §4.4a's *"a partial **text**
   plate carries chunks whose checksums all hold and **looks real**"* is exactly
   why that screen would matter most there.

Related and unspecified in the same area: the chunks form has **no plate layout
rule and no plate-count derivation anywhere in §4** — §4.5's search space is
`module size × QR version (1..40) × ECC × tiling`, which cannot produce a text
plate. §3.6a's *"a chunks job is 22–202 plates"* silently equates chunks to
plates (see M3).

**Why the spec permits it:** the branch key the fold chose is *"WHAT it says
depends on the **symbol count**"*, but every line inside both branches is
addressed to a **plate**. §4.1's three-counts warning is four sections earlier,
and the fold was answering a C1 whose walkthrough was framed as "plate 1 of 2" —
so the mock-ups inherited the one-symbol-per-plate framing of the finding they
were closing, in a document that had already ruled that framing wrong.

**Confidence:** **High** on cases 1 and 3; **high** on case 2, whose witness is
quoted in the spec's own §4.1 (the direction of the legend — plate 1 — follows
§4.5a's *"the legend costs 54% of **plate 1's** AREA"*).

---

### [I7] §4.5a's legend field set omits `PLATE N OF M`, the one field §4.4 makes normative — and whether the reservation is charged to plate 1 or to every plate is never stated

**Severity:** Important

**Where:** §4.5a (*"**NORMATIVE: the reservation is computed from the field set
and the face, never hard-coded.** Two of the five fields (`FROM WALLET`, `TO`)
are **optional**"*) and its table rows *"packed, all five fields (153 chars)"* /
*"packed, mandatory three (99 chars)"*. Against §4.4 and §4.5's objective note.

**The failure, concretely, two parts.**

*(a) The field set is short one field.* The five fields are recoverable from
`design/measurements/RESULTS_legend_budget_2026-08-22.txt:9-14`: bearer warning
(45 ch), source wallet (20), locktime (34), destination (34), format tag (19) —
152 characters, of which the two §4.5a calls optional are the 20 and the 34, so
"mandatory three" = 45+34+19 = 98. **`PLATE N OF M` is in neither set.** But
§4.4 is normative that every plate claims its own position — *"Cut last, a plate
only claims to be `PLATE 2 OF 3` once it is one"* — and §4.4a's quoted builder
emits it as `centerRow(plate.Title, offy)`, a real row costing `fontSize` of
vertical budget (`backup/backup.go:395-399`). A reservation computed from
§4.5a's field set therefore **under-charges every multi-plate job by one line**,
and §4.6 takes that reservation as an input to a regeneration whose whole point
is that *"re-running it without this input would produce a second wrong table"*.

*(b) Plate 1 or every plate is undecided, and the two answers differ by a lot.*

| source | says |
| --- | --- |
| §4.5's objective note | *"1. minimise plates ← **a plate** holds the QR(s) AND the legend"* — unqualified |
| §4.5a | *"the legend costs 54% of **plate 1's** AREA"*; *"a fixed 6-line charge bills **every plate 1** for rows that may not exist"* |
| §4.4 | every plate carries its own `PLATE N OF M` |
| `RESULTS_ecc_selection_2026-08-22.txt:3` | *"CORRECTED: the plate must hold the QR *and* the legend (6 lines = 25.5mm **on plate 1**)"* |

On the plate-1-only reading a 16,367 B transaction is ~13 full-area symbols; on
the every-plate reading each plate falls from v26 to about v21 and the same
transaction needs ~18 — which is over Structured Append's 16 (and compounds I2).
Two competent implementers pick different readings and get different plate
counts, different symbol counts, and in one case an unrepresentable job.

**Why the spec permits it:** §4.4 and §4.5a were written by different findings —
§4.4 is a walk ruling about *when* the legend is cut, §4.5a is an R0 finding
about *how much room* it takes — and neither names the other's field. The word
"legend" then does double duty: in §4.4 it is the one-row plate title
(`backup.MaxTitleLen = 18`), in §4.5a it is a 3-to-6-line, 153-character block.
§7's gate *"P5's gate asserts the legend's EMISSION ORDER"* inherits the
ambiguity: satisfying it for `plate.Title` alone leaves a four-line legend still
emitted first, and the abandoned plate still *"looks finished"* — the exact
outcome §4.4 exists to prevent.

**Confidence:** **High** on (a) — the field list is quoted from the cited
measurement file and `PLATE N OF M` is demonstrably not in it. **High** on (b)'s
ambiguity; **medium** on the ~18-symbol arithmetic, which scales v26's 1,367 B
by area to v21's 66.2 mm rather than reading a capacity table.

---

### [M1] §4.5a's reservation cannot actually be computed from what is stated: the pitch is never given, and the two metrics in the table come from different faces

**Severity:** Minor

**Where:** §4.5a's table (6 lines → 25.5 mm, 4 → 17.0 mm, 3 → 12.8 mm) and
*"measured at the **3.0 mm** face, `font/sh`, **44 columns**"*.

**The failure, concretely:** the table's line **pitch — 4.25 mm — is never stated
in the document**; it is recoverable only as 25.5 ÷ 6. That 25.5 mm comes from
`RESULTS_legend_budget_2026-08-22.txt`, whose budget line is
`PLATE_TEXT_BUDGET = 300 chars (35 chars/line x 20 lines, TEXT-ONLY plate)` —
**35 columns, not 44**. So the packed rows divide the character count by one
configuration's width (44 columns, `font/sh` at 3.0 mm, verified at
`gui/freetext_flow.go:33` and `gui/freetext_proof.go:33`) while multiplying the
line count by the *other* configuration's pitch. In this builder the row pitch
**is** the font size (`offy += fontSize`, `backup/backup.go:394`), so a face
narrow enough for 44 columns on a 79 mm plate (1.795 mm/char) cannot also carry
the pitch of a 35-column one (2.257 mm/char). Smaller corroborating slips: 3 ×
4.25 = 12.75, the table says 12.8; the source file's five fields total 152
characters, §4.5a says 153; and two of the five (`FROM WALLET fa568be0`,
`TO bc1p8rrz…s6n0vcl 0.00399 BTC`) are **operator- and transaction-dependent
strings**, so 153/99 are examples of a runtime quantity, not constants a table
can pin.

**Why the spec permits it:** §4.5a's job was to replace a hard-coded number with
a formula, and it demonstrates the formula's *effect* with a worked table rather
than stating its *inputs*. The 44-column figure came from the fork's free-text
plate and the 25.5 mm from the measurement harness, two sources that were never
put side by side.

**Confidence:** **High** that the pitch is unstated and that 35 ≠ 44. **Medium**
on the derived mm/char ratios, which assume the 79 mm usable width both
documents use.

---

### [M2] §3.1a's stated exhaustiveness method is circular

**Severity:** Minor

**Where:** §3.1a, *"Enumerated exhaustively (`grep "switch act.prog|switch
m.prog|switch page" gui/*.go`, non-test)"*.

**The failure, concretely:** the three alternatives in that pattern are the three
receivers of the three sites already known. A fourth site with any other receiver
name — `switch s.prog`, `switch pg`, an `if`-chain, a table indexed by `program`
— cannot match it, so the command cannot establish the claim the sentence uses it
to make. The enumeration **is** correct (I re-derived it from the type; see Part
1), which is what keeps this Minor: the answer is right and the stated method
cannot show it. This matters because the section's own lesson is *"the fix was to
grep for the class"*, and the recorded command is a grep for the *instances*.
I4 above is the same method failing for real one section earlier.

**Why the spec permits it:** the grep was written after the sites were found, as
a record of where they are, and then presented as the method that found them.

**Confidence:** **High.**

---

### [M3] §3.6a and R14 equate chunk count with plate count

**Severity:** Minor

**Where:** §3.6a (*"since a chunks job is **22–202 plates**"*) and R14's
justification (*"an operator committing 22–202 plates to whichever row they
guessed"*).

**The failure, concretely:** 22 and 202 are the **chunk** counts from §2.3's
table, not plate counts. A full 40-byte chunk is 91 characters (§2.3, F-242) and
a text-only plate's budget is 300 characters
(`RESULTS_legend_budget_2026-08-22.txt:1`), so a plate holds roughly three
chunks: the same jobs are on the order of 8–68 plates. §4.1 explicitly forbids
this conflation — *"the spec must not conflate them"*. The ruling R14 states does
not depend on the number, which is why this is Minor; but §4.1's rule is stated
as normative and this is a violation of it in the section that most needs the
number to be right.

**Confidence:** **High** on the arithmetic; the 300-char budget is the free-text
plate's and a legend row would reduce it further.

---

## Verdict

**Part 1: 4 FIXED / 2 PARTIAL / 0 NOT FIXED.** Both PARTIALs are
propagation, not reversal: §3.1a is correct and complete, and §6's P4 row was not
updated to match it (I3); §4.2a's cited ceiling is corrected and its per-symbol
divisor is not (I2).

**Part 2: 0 Critical / 7 Important / 3 Minor.**

The pattern the brief asked me to look for is present a third time, in three
different shapes. Round 0 fixed an instance and missed a class of `switch`.
Round 1 fixed that class and missed the same failure one type-switch away
(**I4**). The round-1 fold fixed a *cited number* and kept the assumption that
made it wrong (**I2**), and rewrote a section without propagating to the
sequencing row that quoted it (**I3**). In each case the fold's edit is correct
where it lands.

**What I did NOT examine:**

- **Nothing was compiled or executed.** Go is not on `PATH` on this box
  (`which go` → not found), so `TestStartScreenPagerDoesNotCollideWithTheVersion`
  was reasoned about arithmetically, not run, and the `engraveObjectFlow` path in
  I4 is traced from source rather than exercised.
- **The `mt` side.** `mt encode --record`, `mt inspect`'s raw-transaction
  subject, and whether `mt` can reassemble a Structured Append set (§4.3's
  per-job step depends on it) — the code is in a separate private repo and was
  not opened.
- **The `me` side.** `me sysw pack`'s stdin path, `MaxSectionLen`, and §2.5's
  mode logic were taken as settled per the brief.
- **QR capacity tables.** I did not re-derive v16/v19/v21/v24/v26 byte capacities;
  I2's and I7's version-vs-millimetre arithmetic scales the cited 1,367 B / 77.4 mm
  pair by area, which is why both carry a medium confidence on the derived
  symbol counts. I noticed but did not pursue that
  `RESULTS_qr_physical_max_2026-08-22.txt` pairs a **v26** footprint with a
  **1,367 B** capacity where the ISO byte-mode L table gives 1,367 B at v24 —
  the brief lists that figure as settled, so it is recorded here as an
  observation and nothing above depends on resolving it.
- **§8's rulings, §2.3's arithmetic, the chunk-character formula, the camera
  claim, `mk1`, F-244, and the retracted 64-chunk cap** — all listed as settled
  and not re-derived. No `fits`/`OVER` column was cited.
- **A whole-document read for defects outside the folds' additions.** This was
  scoped to the new normative content, as briefed. §5's table has three rows
  (R6, R8, R9) carrying three cells in a four-column table, which renders with a
  missing column; it predates both folds and is not counted above.
