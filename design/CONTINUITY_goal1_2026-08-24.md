# CONTINUITY — Goal 1, "Engrave a Transaction", 2026-08-24

> Supersedes nothing. `CONTINUITY_mt_2026-08-24.md` set the two goals; this
> records the first day of **Goal 1**. Goal 2 is untouched.

## State in one line

**Spec written, walked, R0 rounds 0–2 folded — no code. The gate is OPEN, and
two of the blockers are now the OPERATOR's, not a reviewer's: O11 and O14.**

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
