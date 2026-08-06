# IMPLEMENTATION PLAN — `SIZEPROOF!`

Status: **R3 — R0 GREEN (0 Critical / 0 Important) at round 2. Executable.**
Author: controller session, 2026-08-05.

| round | verdict | lanes |
|---|---|---|
| 0 (`sizeproof-plan-R0-round0.md`) | RED 0C/4I/3m/2n | opus + sonnet, opus synthesis |
| 1 (`sizeproof-plan-R0-round1.md`) | RED 0C/1I/3m/2n | opus + sonnet, opus synthesis |
| 2 (`sizeproof-plan-R0-round2.md`) | **GREEN 0C/0I** | single opus, scoped to the fold |

Round 2 verified by probe that the §7.7(b) constraint set is strong enough for
the fixture it had in mind — but **its "at every admissible block-1 row count"
was wrong, and P2 corrected it.** See §1's P2 constraint 2. The correction is
folded; the fixture as committed at `acda504` is stronger than the constraints
required, and the controller has independently re-run the P3 mutation against it.

(Revisions are `R`n; `P`n means a phase of §1. An earlier draft labelled itself
"P1", which collided with the phase namespace and read as though implementation
had started.)
Spec: `design/SPEC_sizeproof.md` @ R5, **R0 GREEN (0C/0I) at round 4**.

This is a **sequencing** document. It adds no design: every "what" lives in the
spec, and a disagreement between the two is a defect in this file. What it fixes
is the ORDER, the phase boundaries, and which of the spec's 20 test items gates
which phase — so that no phase can close on a suite that could not have caught
the thing that phase changed.

---

## 0. Standing constraints

- **Repo:** the fork, `/scratch/code/shibboleth/seedhammer`. Base is **branch
  `passproof-rename` @ `6d57681`**, which carries the `PASSPROOF!` rename —
  *not* `main`, which is still at `3c3a2ad`. The two land together at release
  (§3.3), so `sizeproof` stacks on the rename rather than duplicating or
  reverting it.
- **Worktree:** one, dedicated. `seedhammer-wt-sizeproof`, branch `sizeproof`.
  The primary checkout stays free — a second agent compiling `gui` against a
  half-written `backup` produces `[setup failed]` that reads exactly like a real
  failure.
- **One implementer.** Not parallel re-implementations. The controller folds
  small post-review fixes inline in the worktree rather than spawning agents.
- **TDD per phase:** the phase's gating tests are written and RED before its
  implementation, and the phase closes when they are green **and the whole suite
  is green** — not just the slice the phase touched.
- **`-update` is never run.** A moved golden is a finding, not a baseline
  refresh. This is the load-bearing property of the entire change: the spec's
  claim is that the general path reproduces the special case exactly, and the
  goldens are the only thing that can falsify it.
- **A fixture that recovers must assert WHICH panic it recovered.** Learned in
  P1, and it invalidated a requirement this plan had written: P1 told the
  implementer to order the `Faces` guard before the `Sizes` guard "so the fixture
  keeps isolating the face map". Mutation-tested, **neither half was observable**
  — swapping the guard order left the suite green, and shortening the fixture's
  `Sizes` left it green too, because `defer recover()` accepts *any* panic. The
  requirement was decoration. It becomes real only with
  `strings.Contains(msg, "faces")`, and then the combined mutation goes red.
  Every remaining phase inherits this: `recover() != nil` is not an assertion.
- **Review:** opus + sonnet two-lane on the whole diff **after P6** (§3), per the
  operator's 2026-08-05 choice. There is no P5 checkpoint review; P6 carries
  §7.3's composition pins and the §7.19 mutation pass, which is exactly the
  content a whole-diff review should see. **fable stays unspent** — nothing here
  is irreversible, funds-touching, or normative-codec; the OTP write is long done.

## 1. Phases

Ordered so the goldens stay green at every boundary. Each phase is a commit.

### P1 — the data carriers.

`Block.SizeMM` (§2.2). **`qrPlacement` and `qrPlaceAt` (§2.1)** — the type must
land here, not in P2, because `Fitted` declares a field of it in this phase.
`Fitted` gains `Mixed`, `Sizes`, `TitleSizeMM`, `FooterSizeMM`, `qrAt` (§2.3);
**both** legacy constructors populate them — `fitBlocksAt` and `EngraveFreeText`
(§2.3). Guards added: `len(Sizes) != len(Lines)`, the size/string invariant,
§2.1.1's **two panics** *and* its third row in both its enforcement points — the
`qrAt.Bottom <= plateHeight - margin` **ERROR return** from `fitBlocksAt`, plus
its defensive re-assert as a panic in `EngraveFitted`.

`textLayout` still takes `(qrc, qrScale)`; nothing narrows against the placement
yet. `EngraveFitted` still walks rows.

**Two existing fixtures must move with this phase.** P1 is not the inert pure
addition an earlier draft of this plan claimed — both bypass the legacy
constructors and hand-build a `Fitted`:

- `backup/blocks_test.go:406-417` — `mk` returns a `Fitted` with `Lines` and
  `Faces` and no `Sizes`, then engraves it twice **with no `recover`**. The new
  guard fails the test outright, so P1 cannot close whole-suite-green without
  this. It gains **`Sizes` = `len(lines)` copies of `size`, equal to `SizeMM`,
  with `Mixed` false and `TitleSizeMM`/`FooterSizeMM` 0** — its `Title` and
  `Footer` are empty, so §2.3's invariant requires 0. **Uniform-at-`SizeMM` is
  the only fill that works**: any other passes the length guard and every
  assertion today, then silently measures a different row once §2.5's running y
  lands in P3, because the test's `insetOf` reference is plate-absolute. Its
  per-row-inset assertions otherwise keep their exact form — it is the only
  existing pin on the property that rewrite endangers.
- `backup/blocks_test.go:296-309` — same omission, but it already defers a
  `recover`, so it stays green and silently stops testing what it names. It gains
  a correctly-sized `Sizes`, and **the `Faces` guard is evaluated before the
  `Sizes` guard** so it keeps isolating the face map. §7.19's mutation pass is
  scoped to tests *added* in P1-P6 and would not catch this one.

Audit for any other hand-built `Fitted{}` in `backup` outside the two
constructors. The three GUI literals the spec names
(`gui/freetext_flow_test.go:564, 893, 928`) are confirmed safe: all three flow
into `ftConfirmBody`/`ftConfirmSummary` and never reach `EngraveFitted`, so
`:893`'s `QR` with no placement does not trip §2.1.1's first panic.

**Gate:** §7.1 (every golden, byte for byte) · §7.15 (the `Sizes` guard panics).

**What P1's gate CANNOT see:** a wrong VALUE in `qrPlaceAt`, or in the new error
return. No consumer reads `qrAt` this phase — `EngraveFitted` still derives the
code's y from `lay.holeLines`/`lay.qrLines` (`freetext.go:80-85`) — and
§2.1.1's third row is measured unreachable from any shipped fit. `qrPlaceAt`
takes no face, so it cannot delegate to `textLayout` and necessarily duplicates
that derivation; an off-by-one in the `ceil`, or `qrBorder` dropped from
`KeepOutX`, closes P1 green on every golden. Both are first exercised at P2
(§7.1, §7.7(a)/(c)/(d)). **P1's guards prove nil-consistency and the `Sizes`
length, nothing more.**

*Why first:* apart from the two fixtures above it is pure addition, so a moved
golden here means the constructor FILLS that feed the existing engraving path
were wrong, with nothing else in the diff to hide behind. The placement
arithmetic is not among what the goldens pin here.

### P2 — the QR placement. Extraction, not redesign.

`textLayout` takes a `*qrPlacement` in place of `(qrc, qrScale)`; `lineLayout`
carries `qrTop`/`qrBottom` instead of `holeLines`/`qrLines`; `EngraveFitted`
reads `f.qrAt` and `EngraveText` its own local. Every producer keeps the anchor
it has today (§2.1's producer table) — **including `AdmissibleBlocks`, whose
`anchorY` is `outerMargin` while §2.4 will give it a different `start`.**

**The signature change forces more sites into this same commit**, some of which
this plan otherwise attributes to later phases. They are compiler-forced, so
nothing can go silently wrong — but they are named here so the diff is not a
surprise. `wrapBlocks` (`fit.go:147`), `faceLayouts.at` (`fit.go:327`) and
**`rowFaces` (`fit.go:302`)** thread a `*qrPlacement` instead of a `*qr.Code`,
which makes **`MaxCharsAtBlocks` and `rowFaces`** placement producers in P2
rather than P3 — all still at `anchorY = outerMargin`, one placement per plate.

Six test fixtures move too, and **their assertions and numbers are preserved
exactly**. Three read the removed `lineLayout` FIELDS:
`backup/engravetext_test.go:159-196` builds a `lineLayout` literal with
`holeLines: 2, qrLines: 19` and is the current pin for the `n < 1` clamp — it is
re-expressed in `qrTop`/`qrBottom` **with the same clamp numbers**;
`backup/freetext_test.go:195-197` and `:289` recompute the QR's y from
`lay.holeLines`/`lay.qrLines` and read the placement instead. Three more call
`textLayout` directly and change TYPE only, keeping their arguments and
assertions identical: `blocks_test.go:30`, `blocks_test.go:393` (`insetOf` —
which is the P1 pin above, so it is touched in two consecutive phases and must
survive both), and `fit_test.go:15`. A pin rewritten in the same commit that
changes its subject is where coverage quietly weakens.

Row indexing is UNCHANGED in this phase. The band is expressed in absolute y and
is measured to agree with the row-index predicate on every row at every rung, so
this phase should be behaviour-identical by construction.

**Also written here, but gated at P3: §7.7(b)**, the two-block-plus-QR fixture.
It is cheap to build now and **passes vacuously in P2** — every block still lays
out at `baseY = outerMargin`, so block 2's window is right whatever the
implementer does.

That is exactly why its SHAPE has to be written down: it is being authored in a
phase where nothing can tell a good fixture from a useless one. Four constraints,
each of which a natural implementation gets wrong:

1. **Pin the code by MODULE COUNT, not text length** (§7.7(b)) — mode selection
   follows the character set, so "700 characters" is a different plate the day
   the fixture's case changes. **Whatever module count you pin, it need not be
   89**: measured, an 89-module code is not producible *by* a composition that
   fits at 3.0 mm — the plate holds ~464 characters beside such a code and the
   code itself needs ~645 bytes. Either inject the code into the unexported
   `fitBlocksAt` (legal in-package, and the shape round 2 verified) or pin the
   count the exported path actually yields. Constraint 4 below holds for **every**
   realizable code size, because rows below any band ink full width.
2. **Block 1 must wrap to at least TWO rows and end above the band's first row**
   — and the fixture must carry a **per-row budget assertion**, not only
   constraint 4's rectangle. **Corrected by P2; round 2's claim that any
   admissible block-1 row count works is FALSE at 89 modules.** The code is
   centred in its band, and at 3.0 mm the centring offset (21120) exceeds one row
   (19200), so the code's top edge sits in the band's *second* row. With a 1-row
   block 1 the band shifts by one, leaving only plate row 3 unnarrowed — and row
   3's y-range `[76800, 96000)` does **not** intersect the code box starting at
   97920. Constraint 4 alone therefore passes on the defect.
   The committed fixture (`acda504`) uses a 2-row block 1 **and** asserts *every
   row is wrapped at the budget of the plate row it lands on*, which catches a
   shift of any size at any module count and removes the dependency on that
   geometric coincidence. **P3's mutation step runs against the fixture AS
   COMMITTED, never a re-derived one.**
3. **Block 2's text must FILL its budget on every row it spans** — a spaceless
   run, as `mixedBlocks`/`fillRows` already build. A wrong budget must produce
   INK, not merely permit it; short words leave the extra columns empty and the
   fixture passes under the defect. (`fillRows` computes its budgets with
   `qrc = nil`, so used verbatim it sizes for a no-QR plate. That does not weaken
   this constraint — spaceless text fills every row but its last whatever the
   budgets are — it only changes how many rows block 2 spans, which is visible
   immediately: the fit either lands at 3.0 mm or errors.)
4. **"No body ink enters the code box" is an intersection with the code's
   RECTANGLE** — `[qrAt.X, X+Size) x [qrAt.Y, Y+Size)` — **never a plate-wide
   max-x bound.** Measured at 3.0 mm in `sh` with an 89-module code: the band is
   plate rows 3-22 at 12 columns against 44 unobstructed, and **row 23 sits below
   the band and legitimately inks all 44 columns**. A max-x assertion therefore
   fails on a CORRECT plate, and the natural repair is to weaken it into
   something the defect also satisfies.

**Gate:** §7.1 · §7.7(a) the band equivalence at all six rungs · §7.7(c) both
engravers read the stored placement · §7.7(d), **`fitBlocksAt` half only** — the
`FitSized` half is vacuous by §2.7 (`qrAt` is always nil there) and is recorded
as such rather than scheduled into P4.

### P3 — the y budget and the running y.

`wrapBlocks` takes `sizes`, `qrp`, and device-unit `start`/`limit` (§2.4), with
**both existing caller translations** — `AdmissibleBlocks` →
`params.I(outerMargin) + params.F(size)`, `rowFaces` → `params.I(outerMargin)`.
`limit` is read off the footer. `EngraveFitted` accumulates y and reads `at(0)`
per row (§2.5). `faceLayouts` is deleted (§2.6).

**Gate:** §7.1 · §7.2 rows are cut at the sizes claimed · **§7.7(b) a two-block
plate with a QR wraps block 2 at the code's own budget, and no body ink enters
the code box** · §7.8 footer and last body row disjoint on a MIXED plate with a
3.8 mm footer · §7.11 the unbounded callers still report untruncated counts ·
**§7.20 `AdmissibleBlocks`' verdict does not move, with `useQR` true and false,
plus the `MaxCharsAtBlocks` pin.**

*This is the phase that broke twice in review, and it is where §7.7(b) earns its
keep.* P3 is the phase that gives each block its own running-y `baseY` **and**
indexes rows from 0 within the block — precisely the combination in which
reaching for a block-relative row index re-creates §2.1's measured
12-columns-vs-36-columns defect and engraves body ink across the code. No
shipped golden is a multi-block QR plate, so nothing else in this gate can see
it: §7.2 is row sizes, §7.8 the footer band, §7.11 the unbounded callers, §7.20
the admission counts.

**§7.7(b) is a REGRESSION pin and cannot be RED before this phase's
implementation** — §0's TDD rule does not apply to it. At the start of P3 the
tree is P2's tree, where the item passes vacuously; correct P3 code leaves it
green; only DEFECTIVE P3 code turns it red. So there is no phase in which it is
ever observed failing, and a fixture written in a shape that CANNOT fail would
sail through unnoticed until §7.19 at P6 — three phases of committed work later.

**Its power is demonstrated by pulling §7.19's mutation forward for this ONE
item.** After P3 is green: temporarily index block 2's rows block-relative
against `baseY = outerMargin` — **post-P3 that means pinning `wrapBlocks`'
layout to `textLayout(…, params.I(outerMargin), qrp)` instead of the running
`y`.** (An earlier draft named `widthFor(lay, row)` → `widthFor(lay, 0)`; that
is the pre-P3 form of the same defect, and after P3 `widthFor(lay, 0)` is the
CORRECT call — the block-relative index is right once paired with `baseY = y`.)
Confirm **§7.7(b) FAILS**, then revert — from a COPY made first, never from the
index. **A §7.7(b) that stays green under that mutation is a blocking finding,
not a passing gate.**

*Already exercised twice against the fixture as committed:* by P2's implementer
at authoring time, and independently by the controller at the P2 boundary. Both
halves go red — `TestTwoBlockQRPlateWrapsBlockTwoAtTheCodesBudget` and
`TestFreeTextBodyNeverEntersTheQRBox`. P3 re-runs it because P3 is the phase that
makes the defect *reachable*; the earlier runs prove only that the fixture can
speak.

Round-2's I2 also lands here whole. Round-3's I2 is split: its design half is P2's
(the `AdmissibleBlocks` anchor), and only its PIN — §7.20 with `useQR` — closes
here. Both were invisible to the compiler and to every golden: `int` to `int`,
right shape, wrong meaning.

### P4 — `FitSized`, and mixed plates become real.

`FitSized` (§2.7) with its validation, its `Mixed`/`SizeMM` computation, and nil
`QR`/`qrAt`.

**Three things P1 landed that P4 owes a matching decision on** (surfaced by the
P1 implementer, `5b2f1b2`):

1. **`ErrQRTooTall`** is P1's name for §2.1.1 row 3; no earlier artifact named it.
   P1 checks it **after** the wrap succeeds, so `ErrTooLarge` wins when both would
   fire and every reachable case returns byte-for-byte what it does today. If
   `FitSized` refuses before wrapping, the two paths diverge — **make them agree
   or state why they differ.** Note also that `FitBlocks` loops rungs with
   `if err != nil { continue }`, so this error is only ever *surfaced* by
   `FitBlocksAt`; that is correct, since the band tightens as `fontSize` drops.
2. **The per-entry non-zero `Sizes` check.** Spec §2.3 puts it inside the
   size/string invariant and P1 implemented it as a panic; it fires on nothing
   today. `FitSized` owes the matching **error** return. **Settled at P4, and the
   debt as written was itself a false-pass slot:** §2.7's rung guard already
   refuses 0 (0 is not in `FontSizes`), so this check has **no independent
   verdict** — deleting it changes the message and never the accept/refuse
   outcome. It is observable only through its wording, so P4 words it as
   `EngraveFitted`'s panic is (`is sized 0mm`) and the test asserts that
   substring. The BEHAVIOUR is fully covered by the rung check; only the message
   is this check's.
3. **`EngraveFreeText` cannot enforce the `Bottom` bound** — it returns an
   `engrave.Engraving` with no error channel, so that constructor reaches the
   bound only through `EngraveFitted`'s defensive panic. By design per §2.1.1,
   and load-bearing: it is the reason that panic is not merely redundant.

**Gate:** §7.5 a block inside the top band is inset, one below it is not · §7.6
the same face at two sizes on one plate · §7.9 `EngraveFitted` on a `Mixed`
plate does not panic · §7.14 the size/string invariant refuses, and the
no-footer path gives `limit == plateHeight - margin`.

### P5 — the GUI surface, the triggers, and the previews.

Every **GUI and preview** row of §3's table — the backup-package rows close
earlier: `EngraveFreeText` in P1, `AdmissibleBlocks`/`rowFaces`/
`MaxCharsAtBlocks` in P2-P3. Plus `ftPlan.Blocks`' part-count predicate and its
corrected doc comment (§3.1); the `SIZEPROOF!FRONT`/`BACK` entries with
`TextQR: ""` and their `Side` (§4); `ftProofOutcomeFor`'s per-proof footer;
`fittedPreviewAt` through `ftFitAt`; `cmd/plateview`'s two entries.

**Gate:** §7.10 end-to-end through `freetextPlateHook` · **§7.13 all four edit
shapes plus the synthetic `[2,1,1]` plan** · §7.16 both proofs need the whole
plate · §7.17 the confirm screen names the rungs, never `0.0mm`, and fits by
measured rectangles · §7.18 the preview's per-row sizes equal the device's.

### P6 — the composition pins and the mutation pass.

§7.3's two per-block tables (face, size, row count, per-row budgets, y-range,
total) · §7.4 the character set per `(rung, face)` · §7.12 the titles.
Then §7.19: **mutate every test added in P1-P6 and confirm each notices.**

*Why last:* §7.3 is what makes the back's 2.400 mm of spare safe, and it can
only be written against the real composition, which does not exist until P5.

**Cleanup owned by this phase:** `bodyRows` became dead production code at P3 —
only tests call it now, one of them directly. P3 deliberately left it rather than
edit the P2 §7.7(b) fixture, which is pinned as-committed. Delete it here, or
state why it stays.

## 2. What closes the phase gate

A phase is green when its listed items pass **and** `go build ./...` and
`go test ./...` are green — the whole surface, not the slice. A red suite is
itself a blocking finding.

Between phases, reconcile follow-ups: anything filed against a phase is burned
down in or before that phase, not batched to the end.

## 3. After P6

1. Two-lane adversarial review over the whole diff (opus + sonnet), R0 =
   plan correctness. Non-deferrable — it is what catches implementation-introduced
   regressions TDD misses. Persist verbatim to `design/agent-reports/`.
2. **On a passing review, MERGE EVERYTHING TO `main`.** Standing operator
   directive, 2026-08-05: *"When we pass review, merge everything to
   master/main."* No further confirmation needed — a green review IS the
   authorisation.

   The branches are a clean linear stack, so **one merge carries both**:

   ```
   sizeproof         f666c1f (P4) → 5e3d16e (P3) → acda504 (P2) → 5b2f1b2 (P1)
   passproof-rename  6d57681   the PASSPROOF! rename
   main              3c3a2ad
   ```

   Merge `sizeproof` into `main`; that brings the rename with it. Do **not**
   cherry-pick or re-merge `passproof-rename` separately — it is already an
   ancestor. Delete both branches and the `seedhammer-wt-sizeproof` worktree
   after the merge lands.
3. Cut a release carrying this **and** the un-flashed BIP-39 password work
   already on `main`, so one flash gets the operator everything
   (operator's choice, 2026-08-05). Tag as `fork-v0.0.0-g<sha>` per the previous
   release, build the `.uf2`, attach it to the GitHub release.
4. The operator flashes and engraves. Reading the plates is its own cycle.

**Gate discipline still applies to step 2.** "Merge on pass" is authorisation for
a GREEN review, not a licence to merge past an open Critical or Important. A RED
review is folded and re-run first, exactly as every gate in this cycle has been.

## 4. Out of scope here

Everything in the spec's §8, plus: this plan does not schedule the glyph test
coverage sweep, the `f` top-bar steel verdict, or the stale worktree cleanup.
Those are open items in `CONTINUITY_2026-08-05.md` §6 with no owning phase in
this change.
