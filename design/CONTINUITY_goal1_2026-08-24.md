# CONTINUITY — Goal 1, "Engrave a Transaction", 2026-08-24

> Supersedes nothing. `CONTINUITY_mt_2026-08-24.md` set the two goals; this
> records the first day of **Goal 1**. Goal 2 is untouched.

> **SUPERSEDED FOR P1 AS OF 2026-08-25 — see
> `design/CONTINUITY_goal1_P1_2026-08-25.md`.** The plan is at **v8** after
> five rounds, not the round-0 state described below. **This file remains
> authoritative for the SPEC cycle, the operator journey walk, the two
> rulings and S0's schedule**, none of which changed.

## State in one line

**Spec is R0 GREEN. P1's plan is in review (round 0 found 5C/13I; v2 rewritten).
Still no code — the plan gates it.** `mnemonic-transaction` is now PUBLIC so
`mt-codec` can be published and depended on.

## What exists now

| artifact | what it is |
| --- | --- |
| `design/SPEC_engrave_transaction.md` | the spec. ~710 lines, DRAFT, pre-R0 |
| `design/JOURNEY_WALK_engrave_transaction.md` | the operator walk that reviewed it — **18 findings A–R**, every ruling and its reasoning |
| `design/agent-reports/R0-engrave-transaction-round0-adversarial.md` | round 0, verbatim: **3C / 8I / 4M**, persisted at `caa90cb` before anything was folded |
| `design/agent-reports/R0-engrave-transaction-round1-foldcheck.md` | round 1's fold-check, **if the dispatched agent finished** |

## The design, compressed

```
tx.final.psbt ─▶ mt encode --record --raw|--chunks ─▶ tx: record
              ─▶ me sysw pack (stdin) ─▶ picotool 0x10D00000 | NFC tag
              ─▶ device: load ─▶ payload menu ─▶ comprehend ─▶ confirm ─▶ cut
```

**Four rulings shape everything else:**

1. **Comprehend, then cut** — the device parses the transaction and shows it.
2. **QR + legend by default**, text plates only from a chunks payload.
3. **Payload carries raw tx XOR chunks**, operator picks at pack time.
4. **`MaxSectionLen` 8191 → 32,734**, derived so two maxed sections still fit the
   region. 8191 was the NFC scan buffer minus one, inherited by flash.

## Two rulings made Goal 1 SMALLER than the first draft

Worth knowing before reading the spec, because the fold shrank scope rather than
growing it — unusual, and both came from the walk:

- **Chunks are engraved VERBATIM** via the existing `mdmkText` path, so **there
  is no `mt1` decoder in v1**. Stated in the spec as a deliberate exception to
  "comprehend then cut", *in those words*, so a later reader does not "fix" it.
- **Applicability lives in a PAYLOAD MENU**, not the carousel — so `lastNav`, the
  compile-time guard, `layoutMainPager` and every wrap site are **untouched**.
  Two earlier forms of that ruling were retracted; the walk records why each was
  worse.

## The three facts that cost the most to establish

1. **THE DEVICE HAS NO CAMERA.** `driver/` holds two NFC readers, touch, display,
   steppers, USB-PD and the machine — no image sensor — and `scanner.Scan` has
   exactly one feed (`gui/nfc_scan.go:62`). **The device writes a QR it can never
   read.** It retroactively invalidated an option offered and rejected earlier
   the same day ("comprehend + read back"), which was unbuildable all along.
2. **No `mt` verb can read a default plate.** `inspect`, `verify` and `decode`
   all take `mt1` strings; the default plate carries raw transaction bytes. Fine
   for broadcasting — that is F-234 working — but the post-cut test the spec now
   makes mandatory has no input path. `mt inspect` gains a raw subject (P2).
3. **Records were wrong more often than code.** Four followups filed in one day
   from reading records against artifacts: F-241 (closed), F-242, F-243, F-244
   (closed).

## What R0 round 0 cost and bought

**Three sentences the spec ASSERTED were measurably false**, and none was
reachable by re-reading:

| the claim | the reality |
| --- | --- |
| "nothing in the carousel changes" | `layoutMainPlates` is a per-program switch ending in `panic("invalid page")`, no compile-time guard — **the device panics when the operator pages onto the new entry** |
| "a pipe/FIFO has no file mode" | a named FIFO is **0666** and really leaks; the F-244 fix had shipped with the hole |
| "the compare screen names no command" | it names **`me sysw pack`** — the risky re-pack path — so the fix is a REPLACEMENT, not the addition that was ruled |

**And two described reuse that does not exist:** `validateMdmk` QR-encodes the
`mt1` string it was meant to engrave as text (C2), and `EngraveText` emits the
legend **first**, the opposite of §4.4 (I8 — found by the controller closing a
gap round 0 flagged as unexamined).

**C1 was the structural one.** Multi-symbol QR is the **common** case — at the
ruled 0.60 mm the largest QR that fits is v26/1,367 B, and the search *prefers*
6 small symbols at 742 B — and nothing said how symbols split or reassemble, so
§4.3's mandatory post-cut test would report failure on a **correct** plate.
**RULED: QR Structured Append**, which is self-ordering, so F-234's promise
survives multi-symbol.

**The legend was costing 54% of plate 1's AREA** — a QR is square, so 25.5 mm of
height takes 25.5 mm of width with it (v26 → v16). Packing the five fields at the
tested 3.0 mm face gives **v19**, or v21 without the two optional fields. The
reservation is now a **formula**. Faces below 3.0 mm are untested —
`gui/freetext_proof.go:24` calls 3.0 mm *"the smallest rung and the hardest
legibility case"* — and are worth ~5 versions, so they go to S0.

## THE LENS PLAN — enumerated up front, not discovered one round at a time

*Closure is LENS-closure: a clean round means the question you asked has no more
answers, not that the artifact is sound. The `mt` cycle closed GREEN under a
correctness lens and the six rounds after found seven more Criticals — every one
from a **first-time question**, none from looking harder. So the lenses are
listed here BEFORE they are needed.*

| # | lens | asks | status |
| --- | --- | --- | --- |
| 1 | **operator journey walk** | what would the operator actually do? | **DONE** — 18 findings (A–R) |
| 2 | **adversarial correctness** (R0 r0, opus) | construct a failure the spec permits | **DONE** — 3C / 8I / 4M |
| 3 | **fold-check** (R0 r1, sonnet) | did the fold fix it, or only claim to? | **DONE** — 3 PARTIAL + 1I/2M *the fold introduced* |
| 4 | **fold-check + implementability** (R0 r2, opus) | could two implementers build different things — or nothing? | **DONE** — 2 PARTIAL + **0C / 7I / 3M**; first round with no Criticals |
| 4b | **what did the diff FALSIFY** (R0 r3, opus) | what did these changes make untrue *elsewhere*? | **DONE** — **0C / 8I / 10M**, seven of eight pure propagation |
| 5 | **spec-coverage** (R0 r4, sonnet) | ruled but unbuilt, or built but unruled? | **RUNNING** |
| 6 | **failure-states** | for each thing that goes wrong, what does the operator SEE? | not run — **candidate for during implementation** |
| 7 | **comprehension** | can someone who was NOT here read this and build the right thing? | not run — **candidate for during implementation** |
| ~~8~~ | ~~Journey B — recovery~~ | ~~someone finds the plate in fifteen years~~ | **SKIPPED by operator ruling 2026-08-24** |

**Lens 8 is the one most likely to change the design**, because everything so far
has been walked from the *engraving* end. F-234's whole promise lives at the
recovery end, and the one time this session anybody glanced that way it produced
finding O — *no `mt` verb can read a default plate*.

**And a gate that has never executed is a hypothesis, not a gate.** Four of this
spec's own gates are unrun: S0's test plate (module size, byte encoding,
Structured-Append scanning, sub-3.0 mm legend face) and O12 (does our encoder
even emit Structured Append). The `mt` cycle's worst defect was exactly this —
an acceptance mechanism nobody had ever executed, invisible to thirteen readings
and about an hour to find by trying it.

## The severity curve, and what it says about stopping

| round | lens | result |
| --- | --- | --- |
| 0 | adversarial correctness | **3C** / 8I / 4M |
| 1 | fold-check | 0C / **1I** / 2M — and the I was *introduced by the round-0 fold* |
| 2 | fold-check + implementability | **0C** / 7I / 3M |

**Criticals are gone; Importants are not.** But round 2's seven were mostly
**propagation** (a ruling in one section contradicting a phase row in another)
rather than **design** — which is the signal the "when review rounds stop paying"
rule names: once only claims remain, audit them mechanically at fold time instead
of buying another round.

**Except round 2 also produced two findings no fold-check could have:** the QR
library has no Structured Append, and `ClassTransaction`'s wire layout is
defined nowhere. Both came from the **implementability** lens, not from checking
the fold. That is the argument for spending rounds on **new questions** rather
than on re-checking the last answer.

## THE CLASS FAILURE, THREE ROUNDS RUNNING — the most reusable thing learned here

| round | what was named | what was actually true |
| --- | --- | --- |
| 0 (I2) | "one program-keyed switch must change" | |
| 1 | the fold cited **one** site and stopped | grepping the class found **three**, two failing *silently* |
| 2 | the grep searched `switch .*prog` | a **fourth** switches on a scanned object's **type** — same class, different key |

**Fixing the instance a finding names, and not the class, is how the next
instance survives another round.** The spec now states the *rule* — *"every
enumeration a new program or type must join, whose default is silent"* — instead
of the list, because the list was wrong three times.

## S0's SCHEDULE, AND THE PREREQUISITE IT DEPENDS ON

**Operator, 2026-08-24: S0 is high priority but the machine is VERY LOUD and can
only run at certain times of day. Anticipate cutting it in about a week.**

**That un-blocks most of the plan rather than delaying it.** §6 reads "S0 first",
but only **P5** actually depends on S0's answers:

| phase | depends on S0? |
| --- | --- |
| P1 `me` — `ClassTransaction`, record framing, stdin, sealing, cap | **no** |
| P2 `mt` — `--record`, `inspect` raw subject, the carried txid | **no** |
| P3 fork — port P1, the `tx:` branch | **no** |
| P4 fork — payload menu, the program, the four lockstep sites | **no** |
| **P5** fork — the plate: module size, encoding, Structured Append | **YES** |

So P1–P4 can proceed against the R0 gate on their own schedule; S0 gates P5 only.
**§6 should be re-sequenced to say that** once round 3 is folded — it currently
implies a hard serial order that the loudness constraint would otherwise turn
into a week of idle time.

### THE PREREQUISITE NOBODY OWNS, and it has lead time

§4.2c says S0 cuts its Structured-Append pair from *"an independent committed
generator, `scripts/gen-sa-fixture.py`, segno-based, validated off-screen before
the cut"*.

**That generator does not exist, and no phase owns writing it.**

It must be written, `segno` installed, the symbols validated on screen, and the
fixture committed **before the loud window opens** — otherwise the window gets
spent answering three of the four questions it could have, and the
Structured-Append physics gate slips another week. Its lead time is the reason
this is recorded here rather than left to §6.

**It is also the cross-implementation oracle** P5's gate reproduces
module-for-module (§4.2c), so it is not throwaway scaffolding — it is a
committed artifact with a second job later.

## The severity curve after four rounds

| round | lens | result |
| --- | --- | --- |
| 0 | adversarial correctness | **3C** / 8I / 4M |
| 1 | fold-check | 0C / 1I / 2M — the I was *introduced by the round-0 fold* |
| 2 | fold-check + implementability | 0C / 7I / 3M — two findings no fold-check could reach |
| 3 | what did the diff falsify | 0C / 8I / 10M — **seven of eight pure propagation** |

**Three rounds with zero Criticals, and the last one was almost entirely
propagation.** That is the "rounds stop paying" signal: once only propagation
remains, audit it **mechanically at fold time** rather than buying another round.
Round 2 is the counter-example worth remembering — its two best findings came
from a **new lens**, not from re-checking the previous answer.

## P1 — and the sharpest lesson of the cycle

**The spec went GREEN, so I wrote P1's implementation plan. R0 returned 5
Critical / 13 Important.** Three Criticals were in **one 32-byte field**.

**The lesson is not "the plan had bugs". It is this:**

> **v1's §1.1 named the txid byte order *"the most likely defect in this plan"* —
> and then stated the LOSING answer as normative.**

And the escape clause made it worse: it deferred to *"whatever `mt-codec` does"*,
but `mt-codec` takes a display **string** and has no txid field, so **V4 — the
vector designed to pin the dangerous thing — was pinned to the wrong axis and
could not have caught it.**

**The answer was in the function's NAME the whole time**:
`content_id_from_txid_display` (`mt-codec/src/string_layer/pipeline.rs:17`), whose
comment had already anticipated the exact trap — *"'which 20 bits, from which
end' is exactly where two implementations diverge silently, so this takes the
display string rather than raw bytes."*

**Naming a risk is not managing it.** The moment a plan says "this is the most
likely defect", that is the thing to go READ THE SOURCE for — not the thing to
defer to a vector.

**What it would have cost.** Shipped, R15 refuses every byte-perfect chunks
record. Nothing surfaces until the Go port disagrees, or until a plate is cut.
And `cargo publish` is irreversible — a wrong constant would have frozen.

**Two further Criticals in the same field and its neighbours:** "raw
double-SHA256" over a witness-carrying body is the **wtxid**, not the txid; and
`mt_codec::decode` is a **BCH verifier** (`grep bitcoin:: → 0 hits`), so the
anti-smuggling claim v1 rested on was false — entropy round-trips as a valid
`mt1` string with an attacker-chosen `set_id`.

## RULING 2026-08-24 — THE CHUNKS FORM RIDES AS BARE RECORDS (md1/mk1's pattern)

**Found while explaining the architect's new finding to the operator, and it is a
real defect in the R0-GREEN spec.**

**The problem.** An `mt1` chunk is a bech32 string — already printable ASCII. The
reserved-prefix rule hex-encodes a record body, so **every character costs two**.
The chunks form is text encoded as though it were binary, and the cost compounds
with text's own ~2.3 chars/byte:

```
pathological 10-in/2-out, 8,067 B
  RAW     -> record 16,223 chars   fits (16,511 spare)
  CHUNKS  -> record 37,255 chars   OVER THE 32,734 CAP BY 4,521
```

**The same transaction fits comfortably as raw bytes and cannot enter the
container at all as chunks.** And the hex is there only to smuggle the LF
separators past EPD §6.4 — it carries the separators, not the data.

**The fix, ruled: follow `md1`/`mk1`.** They already solved this — a chunked
constellation format does **not** ride as one hex-encoded record. Each chunk is
its **own bare record** (`gui/scan.go:92` returns `mdmkText(buf)`, no prefix, no
hex), and the container's own LF separates them.

```
  metadata record  3 + 2x43       =     89
  202 bare chunks                 = 18,583   (LF-separated by the container)
  total                           = 18,673 chars   FITS, 14,061 spare
```

**§2.1's "one record, not siblings" ruling is reopened — and the binding gets
STRONGER.** §2.1 worried a separate legend would be merely *adjacent*. But the
metadata record carries the txid and **every chunk carries `chunk_set_id` = the
top 20 bits of that same txid**, so the association is **derivable from content,
not positional**. **R15 stops being a consistency check and becomes the binding
mechanism.**

**Safe because of a bech32 property**: the data charset
`qpzry9x8gf2tvdw0s3jn54khce6mua7l` excludes `1`, `b`, `i`, `o` — so `1` appears
only as the HRP separator and **`mt1` can only ever mark a chunk boundary**.

### The change-list, so the fold is mechanical

| where | change |
| --- | --- |
| spec §2.1 | the CHUNKS form is a metadata record **plus** bare chunk records; RAW stays one record |
| spec §2.3 | the table's chunks column is recomputed against the new framing |
| spec §3.6b | R15 is described as the **binding** mechanism, not only a cross-check |
| spec §5 | R15's wording follows |
| plan §1 | the `tx:` record's `form = 0x02` body becomes **empty**; the chunks live outside it |
| plan §2.4 | wiring gains a **`ValidMT`** classify branch for bare `mt1` records — the "new `ValidMT`, not a call to an existing predicate" the spec's round 2 already identified (M1) |
| plan §3 | V3, V18–V20 and the CHUNKS vectors re-cut against bare records |

**Not applied yet, deliberately:** R0 round 2 is reading both documents. Moving a
file under a reviewer is how a fold gets reviewed against text that no longer
exists.

## A PROCESS LESSON THE OPERATOR HAD TO REPEAT — look at the siblings first

**Twice in one day the operator had to redirect me to the precedent**: *"How are
mk and md handled here?"*, then *"this question is answered already for md and
mk, so what do they do?"*

Both times I had produced **an option list with tradeoffs** instead of reading
the sibling. Both times the answer was already in the tree — `me`'s `Cargo.toml`
for the codec dependency, the uniform `X-cli`/`X-codec` split for publishing, and
`gui/scan.go:92` for bare chunk records.

**An option list built without the precedent is imagination formatted as
analysis.** It reads as a survey of the space and is a survey of me, and it costs
the operator a turn correcting the frame rather than making the decision.

**Rule for this constellation: before generating options, ask what `md`, `mk`,
`ms` and the fork already do.** If a precedent exists, the burden is on departing
from it.

## Open, and who owns it

| | owner |
| --- | --- |
| ~~gen-sa-fixture.py~~ | **WRITTEN 2026-08-24** (`4e45933`, folded at `5e7c491`). Two symbols, `v5-M`, masks **3 and 2**, 54.0 mm of 79 mm — shares a plate with S0's other blocks, no extra cut. It also proved the 16-symbol cap executably (segno refuses 17) and that **the mask is PER SYMBOL**, which §4.2c had wrong |
| **S0 — cut the test plate** (QR at 0.3/0.45/0.6/0.9 mm + a raw-octet and a base45 symbol, external scanner) | **the operator**. Resolves BOTH live hypotheses: QR encoding (F-243) and module size (F-234) |
| **R0 to 0C/0I** | rounds 0–2 folded (`48da287`, `8290415`, `6d4d099`). **Not green** |
| **O14 — S0 is specified to cut a Structured-Append pair that nothing can produce** | **the operator.** Build SA before S0, hand-build throwaway symbols, or drop the pair — each trades against something |
| **O11 — the picker's key for a chunks payload** | **deliberately unresolved.** The device cannot derive a txid without a decoder, and every alternative trades against a ruling |
| ~~O12~~ | **ANSWERED: NO.** `kortschak-qr v0.3.2` has no Structured Append — zero occurrences, `Encode` returns one `*Code`. Buildable over its `coding` package; **P5 owns it** |
| **O15 — `ClassTransaction`'s wire layout is defined nowhere** | **P1**, and it is the largest gap in the spec: 3 mentions, 0 definitions, and four sections read it |
| **O13 — a legend face below 3.0 mm** | S0; worth ~5 QR versions |
| **Journey B — recovery**, never walked | next walk |
| per-chunk BCH checksum before cutting | proposed, not ruled (spec O3) |
| `validateMdmk`'s four callers — a live F-234 violation | **not this spec** (O5) |

## Closed today

**F-244, Critical, both halves.** `me sysw pack` wrote an unsealed
mnemonic-bearing container at mode 0644. Fixed in `me` (`46f2fd4`) and `mt`
(spec `f152aac`, code `542b391`, provenance `a76c1a9`), closed at `f8e0176`.
All gates green: me 303, mt 210, refusal-coverage 31/18, **mutate-refusals 31/31
red without their check**, journeys A/B/C, provenance.

## How to restart

1. Read `SPEC_engrave_transaction.md` §8 first — the 26 rulings. They are
   decisions, not proposals.
2. Then the **walk**, which is where the reasoning is. The spec states what;
   the walk states why, including two rulings that were retracted and re-made.
3. Check `design/agent-reports/` for the R0 report and reconcile it before
   advancing. **No code before 0C/0I.**
4. **S0 before P1.** Two of this design's gates are hypotheses and one is two
   seconds of machine time. A gate that has never executed is a hypothesis.

## The method note worth carrying

**The walk found things no correctness pass would.** It was run live with the
operator, and their own uncertainty was repeatedly the finding: *"But I might be
piping incorrectly"* (the pipeline did not compose, and the spec claimed it did),
*"I didn't realize `>` creates a world readable file"* (which opened F-244, a
Critical in shipped code, in a walk about transactions), and *"I need to change
my answer"* (which produced a design better than any option offered).

**Operator confusion is data.** Three of the day's best findings came from it.
