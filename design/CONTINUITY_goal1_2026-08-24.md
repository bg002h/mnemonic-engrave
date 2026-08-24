# CONTINUITY — Goal 1, "Engrave a Transaction", 2026-08-24

> Supersedes nothing. `CONTINUITY_mt_2026-08-24.md` set the two goals; this
> records the first day of **Goal 1**. Goal 2 is untouched.

## State in one line

**Spec written, walked, folded, gated — no code. R0 round 0 dispatched.**

## What exists now

| artifact | what it is |
| --- | --- |
| `design/SPEC_engrave_transaction.md` | the spec. ~710 lines, DRAFT, pre-R0 |
| `design/JOURNEY_WALK_engrave_transaction.md` | the operator walk that reviewed it — **18 findings A–R**, every ruling and its reasoning |
| `design/agent-reports/R0-engrave-transaction-round0-adversarial.md` | the R0 round-0 report, **if the dispatched agent finished** |

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

## Open, and who owns it

| | owner |
| --- | --- |
| **S0 — cut the test plate** (QR at 0.3/0.45/0.6/0.9 mm + a raw-octet and a base45 symbol, external scanner) | **the operator**. Resolves BOTH live hypotheses: QR encoding (F-243) and module size (F-234) |
| **R0 to 0C/0I** | dispatched; fold, re-dispatch until green |
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
