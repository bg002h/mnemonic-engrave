# IMPL-S2-P4 — F-423: `bundlePlatePlan` packs plates

S2 plan **P4.2**, implemented against fork worktree `s2/descriptor-arm`
(baseline `0f92554`, P3 complete and review-closed GREEN) and engrave worktree
`impl/descriptor-s2` (baseline `347b82e`). No physical cut; P5.4 owns the
single-character test plate and the real engraving.

## Commits

| repo | sha | what |
| --- | --- | --- |
| fork | `be79e3b` | `backup: Text.FooterRow, because EngraveText never budgeted its body` |
| fork | `231b7c2` | `gui: F-423 -- bundlePlatePlan packs a card's strings onto as many plates as fit` |
| engrave | `b32305c` | `spec: AMENDED 2026-08-29 (S2 P4.2) -- the plate cell and the walk's plate counts` |
| engrave | *this file* | the report |

---

## 1. The seam, and why this one

**Chosen: pack at PLAN time, inside `bundlePlatePlan`, which now takes
`engrave.Params` and trial-fits with the same `backup.EngraveText` →
`toPlate` layout the engrave path builds.**

The plan's constraint was "a plan that says *fits* must not fail at build",
and the deciding fact is **who reads the plan**. `bundlePlatePlan` has three
readers, not one:

- `bundleEngrave` (`gui/bundle_flow.go`) — the build;
- `buildPlateCensusLines` — the **"Plates To Cut" / "Plate Count" screen**,
  shown *before* the first plate, whose entire job is the number;
- `buildPlateInventoryLines` — the **restore document**, headed *"This backup
  is N plates … If any of them is missing, this backup is incomplete."*

Packing at build time would leave both of those counting something the build
does not do. The census screen is the operator's last free abort and the
document is read years later by someone who is not the operator; a count
decided after both had spoken is a promise neither could keep. So the packing
is a property of the PLAN, and `engrave.Params` is threaded to its three call
sites (`singlesig.go`, `multisig.go`, `multisig_build.go`) as
`ctx.Platform.EngraverParams()` — the same value all three already hand to
`validateMdmk` one screen later.

Two consequences of that choice, both handled rather than assumed:

**(a) The per-card lines had to move too.** Both census functions printed
`plateWord(len(c.strings), …)` per card. That was the same number as the
plate count until today. Left alone it would have printed *"md1 descriptor: 6
plates"* under a total of *2*. They now count the plan
(`bundleCardPlateCounts`), so the enumerated list and the total above it are
two readings of one object.

**(b) The marking may not change the count.** `singleSigPlateMark` resolves
*after* `buildPlateCensusLines` has already drawn a number
(`gui/singlesig.go`: census at the confirm screen, marking computed below
it). A plan packed against the actual marking would therefore differ between
the census and the engrave. Every prospective plate is instead trial-fit
against a **worst-case marking** — `backup.MaxTitleLen` (18) of the font's
widest glyph as both title and footer — so the answer is the same for all
three readers. Cost: one row of twenty on an unmarked plate.
`TestBundlePlateFitMarkIsTheWorstCase` pins that the real
`singleSigPlateMark` output is no wider.

### The sibling this follows

`planTransactionTextPlates` (`gui/transaction.go`) already packs mt1 strings:
greedy first-fit, TEXT-ONLY paragraphs, fit decided by the real
`toPlate(backup.EngraveText(...))`, counted in a first pass against a
*widest-realistic* title so the count cannot loosen. This is the same
algorithm and the same title trick, and it reaches the same arrangement for
codes (text plates text-only; codes on plates of their own) from the other
direction.

### Two things `toPlate` cannot see — both measured, both now handled

`toPlate` checks BOUNDS and nothing else. At one paragraph per plate that was
enough. Packing walked into two false PASSes:

**The footer.** `EngraveText` draws a non-empty `Footer` at `footerRowY` and
gives the body **no budget against it** — unlike the free-text path, whose
`yBudget` reads its limit off that same expression. A packed body laid over
the footer row is still inside the safety margin, so `toPlate` returns a fit
over overlapping ink. Measured: six 85-char md1 chunks with a title and
footer end at **y 511828** against a footer row of **481280**, entirely
inside the plate. Fixed by exporting `backup.Text.FooterRow` (a forward to
`footerRowY`, not a restatement — that function's own comment forbids two
expressions) and checking the body against it in `bundlePlateTextFits`.
`backup.TestAPackedBodyCanCoverTheFooterRow` demonstrates the hazard
independently, in `backup`.

**The codes.** `EngraveText` advances `offy` by a paragraph's **text lines
only**, while its QR occupies `qrLines` rows from `holeLines` below that
paragraph's top. Measured at the shipped font on three 85-char md1 chunks:

```
para 0: top 19200   code 67840..311040   text 4 lines  -> next para top 122880
para 1: top 122880  code 171520..414720  text 3 lines  -> next para top 202240
para 2: top 202240  code 250880..494080  text 3 lines
```

Every code is drawn across the paragraphs after it. `QR ONLY` is worse still:
`EngraveText` centers a **text-less** paragraph's code on the PLATE, so N of
them land on one spot. Both lay out inside the plate, so `toPlate` calls them
a fit — confirmed by probe, where a 5-string plate still reported `QR ONLY`
as fitting. So **`validateMdmkStrings` offers a packed plate `TEXT ONLY` and
nothing else**; a one-string plate still offers all three variants, in the
same order, byte for byte as before.

Making the codes work on a packed plate would mean changing `EngraveText`'s
paragraph advance — normative layout, seventeen frozen goldens, and at twelve
rows per code it would fit about one string per plate anyway. Out of scope
and pointless. Recorded as a follow-up candidate below.

### What did NOT change

`validateMdmk(pl, s, title, footer)` keeps its exact signature and is a
one-line delegate, so all six non-bundle call sites (`mdmkFlow`,
`unlockEngraveFlow`, `deriveXpubFlow`, and the tests) and the
`s6b_plate_marking_test.go` source table are untouched. A plate always holds
**at least one string** even if it fails the trial fit — that is the
no-regression rule, not a fallback: before this change every string got its
own plate and `bundleEngrave` decided at the picker whether it laid out,
aborting through `bundleAbortWarning` if not. Packing only ever merges.

---

## 2. The new arithmetic, pinned

`bundlePlateMD1Capacity = 5` — five 85-character md1 strings fit one plate
side at the shipped font, packed against the worst-case marking. Asserted as
a literal **and re-derived from the packer** in the same test, so a layout
change fails loudly instead of silently re-planning every plate in the field.

| card | strings | plates | pinned by |
| --- | --- | --- | --- |
| 1-string md1 (`wpkh_basic`) | 1 | **1** (unchanged) | `TestBundlePlanSingleMD1OnePlate` |
| **keyed single-sig — F-423's named case** | **2** | **1** (was 2) | `TestBundlePlanPacksACardOntoFewerPlates` |
| 3 strings | 3 | **1** | same |
| 5 strings (`bundlePlateMD1Capacity`) | 5 | **1** | same |
| 6 strings | 6 | **2** | same |
| 11 strings | 11 | **3** | same |
| `md1CardA` (real chunked wsh 2-of-3) | 6 | **2** (5 + 1) | `TestBundlePlanVerbatim` |
| `mk1CardA` (111 + 80 chars) | 2 | **1** | same |
| two 1-string cards | 1 + 1 | **2**, never 1 | `TestBundlePlanNeverPacksAcrossCards` |

Other measured capacities (probe, not pinned): 111-char strings → 4 per
plate; 24-char strings → at least 12.

**Whole-flow counts, all now asserted by their walks:**

| flow | strings | plates before | plates now |
| --- | --- | --- | --- |
| full single-sig (ms1 1 + mk1 2 + md1 3) | 6 | 6 | **3** |
| full 2-of-3 build (ms1 1 + mk1 2 + md1 6) | 9 | 9 | **4** (1 + 1 + 2) |
| Trace A bundle (3 mk1 cards × 2 chunks) | 6 | 6 | **3** |
| walk W14 bare bequest card (85 + 83) | 2 | 2 | **1** |
| walk W14 keyed BIP-84 card (67 × 3) | 3 | 3 | **1** |

The last two are why the spec amendment says W14's original one-plate hope is
TRUE — the walk was right about the outcome and wrong about the mechanism.

Geometry the numbers rest on (measured, `backup` internals at production
params): `CharsPerLine = 34`, `LinesPerPlate = 20`, `fontSize = 24320` units,
`footerRowY = 481280`, plate `544000`, `toPlate` limit `524800`.

---

## 3. Mutation evidence

Each mutation applied to the shipped tree, run, then restored; the tree is
green before and after.

| # | mutation | result |
| --- | --- | --- |
| M1 | `bundlePlatePlan` packs the next card's strings in too | **RED** `TestBundlePlanNeverPacksAcrossCards`: *"plate 0 carries [md1f9k2szsp… md1f9k2szsg…]; a plate may only hold its own card's strings"* |
| M2 | `bundlePlateTextFits` returns `true` — no trial fit at all | **RED** `TestBundlePlanPacksACardOntoFewerPlates`: 6 → 1 plate (want 2), 11 → 1 (want 3), *"the packer fits 64 85-char strings on a plate, not the 5 pinned here"*; also reds `TestBundlePlanPlatesClearTheFooterRow` |
| M3 | the bounds check kept, the **footer budget** dropped | **RED** `TestBundlePlanPlatesClearTheFooterRow`: *"plate 0 (6 strings): the body ends at 511828, past the footer row 481280"* — and `TestBundlePlanPacksACardOntoFewerPlates` reds at capacity 6, so the footer check is what makes the boundary 5 |
| M4 | `validateMdmkStrings` offers the QR variants on packed plates | **RED** `TestBundlePlanValidatesEachPlate`: a 2-string plate offered all three, and a **5-string plate still offered `QR ONLY`** — the stacked-code false PASS, caught |
| M5 | `backup.Text.FooterRow` off by one row | **RED** `TestFooterRowIsWhereTheFooterIsCut`: *"FooterRow says 456960 but the footer's ink ends at 499430"* |

M1 and M2 are the two the brief required. M3 and M4 are the ones that prove
the two `toPlate` blind spots are actually covered rather than described.

---

## 4. Gate

Run in `/scratch/code/shibboleth/sh-worktrees/s2-descriptor-arm` at `231b7c2`.

```
gui shard (scripts/gui-shard-test.sh ./gui/ 24)
  RESULT: ok -- all 1013 tests ran across 24 shards        (wall 25s)
  baseline at 0f92554 was 1009 tests, also ok; +4 new tests

non-gui packages (CGO_ENABLED=0 go test -count=1 -timeout 20m, 52 packages)
  exit=0, 52 ok, 0 FAIL

CGO_ENABLED=0 go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/
  ok  seedhammer.com/oracle | ok  seedhammer.com/gui | ok  seedhammer.com/sysw

./scripts/test-32bit.sh
  GOARCH=386 test:  exit 0
  GOARCH=arm build: exit 0

GOOS=js GOARCH=wasm go vet ./cmd/emu/        exit 0

TinyGo device build (nix develop -c tinygo build -size full -print-stacks
  -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2
  -scheduler tasks ./cmd/controller)
  exit 0
  image  1,496,412 bytes at 0f92554  ->  1,498,636 at 231b7c2   (+2,224)

engrave worktree: ./scripts/lint-gate.sh -> PASS
  (fmt + clippy 1.85.0 + clippy nightly, --locked; no Rust changed this phase)
```

**gofmt / go vet — the findings are pre-existing, verified against the
baseline.** `gofmt -l .` lists `gui/transaction.go`,
`gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`,
`mt/mt.go`, `mt/mt_test.go` — the identical five at `0f92554`, checked by
restoring the baseline tree and re-running. `go vet ./gui/ ./backup/` reports
four `testing.ArtifactDir requires go1.26` findings, also identical at the
baseline; `.github/workflows/test.yml` documents why CI runs `go test` rather
than `go vet ./...` (40 pre-existing findings tree-wide). None of the files I
touched appears in either list.

**NOT covered by this gate, stated plainly:**

- **No physical cut.** The single-character test plate and any real
  engraving are the operator's, P5.4.
- **The browser walk drivers were not executed.** `cmd/emu/walk_trace_a.js`
  and `cmd/emu/walk_build_policy.js` had their `plates` loop bounds moved
  (6 → 3 and 9 → 4) with the derivation written beside them, but running
  them needs a browser this environment has none of. They are loop bounds
  and appear in no term of either walk's `ok`, so a stale one would have
  cost watching time rather than correctness — but they are unexecuted.
  **P5.2 re-runs the walk journeys and is where they get proven.**
- **`oracle/gaterecords/*.expect.json` are historical.** They compare a
  committed record against a committed expectation and both are from
  pre-packing runs; the suite is green, but a future gate record minted from
  a fresh walk will carry the new counts.

---

## 5. Follow-up candidates for P5.1 (not filed — P5.1 owns FOLLOWUPS)

1. **A packed plate offers no QR variant.** Operator-visible: before F-423
   every md1/mk1 plate could be cut `TEXT + QR` or `QR ONLY`; a packed plate
   is `TEXT ONLY`. Forced by `EngraveText`'s paragraph advance (§1). Costs
   nothing for the bequest journey — the md1 plate is the hand-copyable half
   and `--as descriptor` is the scannable one — but it is a real reduction
   in what the machine can produce and should be written down where the
   operator will find it.
2. **`backup.EngraveText` lays multi-paragraph codes over one another, and
   the fit check calls it a fit.** Now unreachable from production (the only
   two multi-paragraph callers, `planTransactionTextPlates` and
   `bundlePlateTextFits`, are text-only), but it is a live trap for the next
   caller. Either fix the advance (`offy += max(textLines, qrLines)`, moves
   goldens) or refuse a `Paragraph` with a `QR` when `len(Paragraphs) > 1`.
   The second is cheap and turns a silent wrong plate into a compile-time-ish
   error.
3. **`Text.FooterRow` is a workaround for `EngraveText` having no body
   budget.** The free-text path has `yBudget`; the paragraph path has
   nothing. Giving `EngraveText` the same budget would delete
   `bundlePlateTextFits`'s second check —
   `backup.TestAPackedBodyCanCoverTheFooterRow` is written to fail when that
   happens, so the cleanup announces itself.
4. **Pre-existing:** five `gofmt`-dirty files and four `go vet` findings in
   the fork, all predating this phase.
