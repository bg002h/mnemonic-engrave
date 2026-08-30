# REVIEW-F434-F435-r1 — adversarial pre-merge review

**Target** `git diff f2007b7..f92bb02` on `f434/qr-refusal-and-body-budget` in
`/scratch/code/shibboleth/sh-worktrees/f434-f435` (2 commits: `e5ebb14` F-434
refusal, `f92bb02` F-435 body budget). 9 files, +454 / −224.
**Under review against** `design/agent-reports/IMPL-F434-F435.md` @ `a04791e`,
and `design/FOLLOWUPS.md` §F-434 / §F-435.
**Reviewer** independent; nothing modified, nothing pushed. Worktree verified
byte-identical at exit (`git status --porcelain` empty, `git diff f92bb02`
empty, HEAD `f92bb0277a0aac7ee5a5338823ca3f48c6b5c6ca`). All probes ran through
`go test -overlay` from the scratchpad; no file was written into the worktree.

---

## VERDICT

**GREEN — 0 Critical / 0 Important.** 4 Nits, recorded, non-gating.

The diff is safe to boot. No shipped plate changes (goldens byte-identical by
tree hash *and* by re-run), capacity does not move (re-derived independently and
swept exhaustively over the reachable input space), no new panic path, every
caller handles the new error the way it already handled `toPlate`'s, and the one
deliberate departure is not merely defensible — it is the only correct choice,
for a reason stronger than the one the implementer gave.

---

## 1. THE DEPARTURE — sound, and provably so

**What it is.** `backup/backup.go:526`, the guard is
`if plate.Footer != "" && offy > limit {` rather than the faithful `yBudget`
mirror `if offy > limit {`.

### (a) The golden refusal reproduced — empirically, not by argument

Mutation M2, `overlay` of `backup.go` with the `Footer != ""` conjunct removed
(the reverted both-branch version), `go test ./backup/`:

```
--- FAIL: TestText (0.01s)
    --- FAIL: TestText/0-shards-1 (0.00s)
        backup_test.go:192: EngraveText: backup: text does not fit a plate: the body ends at 529920, past the footer row 524800
--- FAIL: TestEngraveTextSurvivesAQRWiderThanTheLine (0.00s)
    engravetext_test.go:227: EngraveText: backup: text does not fit a plate: the body ends at 675840, past the footer row 524800
```

The implementer reported the first failure; **the second one is theirs too and
they did not report it** — the both-branch version also refuses
`TestEngraveTextSurvivesAQRWiderThanTheLine`. That strengthens their case, it
does not weaken it.

Re-derived from the numbers independently (probe, `backup` package):

| plate | nominal bottom | bottom margin | both-branch refuses? | measured ink | ink over margin? |
| --- | --- | --- | --- | --- | --- |
| `text-0-shards-1` | **529920** | **524800** | **yes** | **524120** | **no** (680 under) |
| `text-1-shards-1` | 384000 | 524800 | no | 377862 | no |
| `text-2-shards-1` | **19200** | 524800 | no | **491698** | no |

The report's wording ("ends 5120 units past the margin nominally and short of it
in ink") is exact: 529920 − 524800 = 5120 past nominally; 524800 − 524120 = 680
short in ink. A shipped golden, refused.

### (b) Can ink OVERRUN where nominal fits, on a footerless plate, in a way
`toPlate` would miss?

**No. Proved, not argued, on three legs.**

**Leg 1 — the footerless `limit` is bit-for-bit `toPlate`'s own bound.**
Measured at `prodParams` (`Millimeter=6400`, `StrokeWidth=1920`):
`yBudget`'s footerless limit = `F(85) − I(3)` = 544000 − 19200 = **524800**.
`gui.toPlate` tests `attrs.Bounds.In({safetyMargin, sz − safetyMargin})` with
`gui.safetyMargin = 3` (gui/gui.go:52) and `backup.outerMargin = 3`
(backup/backup.go:73) → its bottom bound is 544000 − 19200 = **524800**. Same y.

So at that boundary `toPlate` tests the *true* quantity (ink) against the *same*
line the nominal budget would have tested a *proxy* against. A proxy cannot
catch anything the exact check misses at the same boundary. It can only differ
by refusing plates `toPlate` accepts — which is precisely what row 1 of the
table above is.

**Leg 2 — the proxy is not even an upper bound, so the "ink overruns nominal"
case is real and is exactly the case the nominal check is blind to.** Row 3:
`text-2-shards-1` (QR-ONLY) has nominal bottom **19200** and ink **491698**. The
nominal accounting counts *text rows only* and sees nothing of a code's band, in
either branch. So for the one plate shape where ink genuinely overruns nominal —
a QR-carrying plate — adding the footerless branch would have bought **literally
zero** coverage while costing a shipped golden. The only thing that catches QR
ink at the bottom margin is `toPlate`, which does.

**Leg 3 — every path from `EngraveText` to steel passes `toPlate`.** Verified by
enumeration: the five surviving non-test call sites are `gui/gui.go:728`
(`validateDescriptor`), `gui/gui.go:2608` (`validateMdmkStrings`),
`gui/transaction.go:1178` (`planTransactionTextPlates`), `gui/transaction.go:1435`
(`buildQRPlates`), `gui/bundle_flow.go:543` (`bundlePlateTextFits`) — **all five
call `toPlate` on the returned plan**, and there is no sixth. `grep` for
`EngraveText(` over the whole tree finds no other non-test caller.

**What else could a footerless plate collide with, that `toPlate` cannot see?**
Nothing. The footer is the only thing cut *inside* the safety margin (footer row
481280 < bound 524800), which is exactly why that one check must live upstream.
The title is at row 0 and the body cannot grow upward (`start` is below it by
construction). Screw holes are handled per-row by `lineLayout.at`, whose hole
predicate is absolute-y and applies at any row index — not a budget question.

**Judgment: the departure is correct.** The footerless branch would add zero
safety, subtract real capacity, and silently change which variants
`validateDescriptor` offers (its `descPlate` carries no Title and no Footer —
gui/gui.go:724 — so the footered branch never binds there at all).

### (c) Bonus: the new check is decision-identical to the deleted one

Swept **121 string lengths × 10 paragraph counts = 1210 packed plates** at
`prodParams` with the worst-case marking, comparing the new nominal budget
against the deleted `bundlePlateTextFits` ink check:

```
total disagreements over 121 lengths x 10 counts = 0
```

Lengths 20..140 cover every reachable string: longest `md1` literal in the tree
is **85**, longest `mk1` **111**, longest `ms1` **128** (measured by grep over
all `*.go`). The replacement is not "close enough at the pinned boundary" — it
is the same function over the whole reachable domain.

---

## 2. THE API CHANGE — every caller handles it, none discards, none crashes

`EngraveText(params, plate) (engrave.Engraving, error)`.

| site | handling | matches how it handles `toPlate`'s error? |
| --- | --- | --- |
| `gui/gui.go:728` `validateDescriptor` | `lastErr = err; continue` | **yes** — identical two lines below |
| `gui/gui.go:2608` `validateMdmkStrings` | `lastErr = err; continue` | **yes** |
| `gui/transaction.go:1178` `planTransactionTextPlates` | `return Plate{}, err` | **yes** — the closure's `toPlate` return |
| `gui/transaction.go:1435` `buildQRPlates` | `return false` | **yes** — the next line does the same |
| `gui/bundle_flow.go:543` `bundlePlateTextFits` | `return false` | **yes** |

- **No discards.** `grep` for `_, _ :=` / `_ = EngraveText` over the tree: none.
- **No missed caller, no stale wrapper.** `grep "EngraveText"` over `*.go`
  returns only these five plus doc comments (`backup/passphrase.go:18`,
  `backup/wrap.go:94`, `cmd/vectorfont/main.go:795`, `seal/record.go:280`,
  `gui/bundle_flow.go:452` and the new prose) and the test uses. No other
  function constructs a `backup.Text`: the six literal sites
  (`gui/bundle_flow.go:537`, `gui/gui.go:724`, `gui/gui.go:2602`,
  `gui/transaction.go:1172`, `:1449`, `:1465`) all feed one of the five.
- **The variant is dropped, not lost.** In `validateDescriptor` /
  `validateMdmkStrings` a refusal removes exactly that entry from
  `validEngravings`; if *all* fail the function returns `lastErr`, which is the
  pre-existing all-variants-fail path. In `bundleEngrave` (gui/bundle_flow.go:618)
  an error aborts the set through `bundleAbortWarning` — the same behaviour a
  `toPlate` error already produced there.
- **No variant can silently vanish at engrave time**, and this is exact rather
  than probabilistic: the packer trial-fits at `bundlePlateFitMark`
  (18×`W` as *both* title and footer), while `bundleEngrave` engraves at one of
  `("","")` (cardMS1, via `bundlePlateMark`), `("", "SEED FP: …")` or
  `("PASSWORD REQUIRED", "COMB FP: …")` (`singleSigPlateMark`, gui/singlesig.go:365).
  `start` is `margin + fontSize` only when the title is non-empty and `limit` is
  `footerRowY` only when the footer is non-empty, so the engrave-time window is a
  **superset** of the packing window in all three cases. The engraver cannot
  refuse what the packer admitted.
- **No new panic path.** `pp.qrp` is dereferenced only under `pp.qr != nil`, and
  both are set together. `plans` is `make(…, len(plate.Paragraphs))` and indexed
  by the same range. `footerRowY` (which divides by `F(footerSizeMM)`) is now
  reached *only* when `Footer != ""`, where the old code called it
  unconditionally — strictly fewer division sites than before. `lineLayout.at`
  is total (clamps `n < 1` to 1). No new `panic`, no new index, no new deref.
- **The Engraving stays re-invocable**, which the refactor puts at risk because
  the layout now lives outside the closure and is shared by every invocation.
  Probed directly — three plate shapes, three planning passes each:

  ```
  packed text  ops=8903  run2==run1:true run3==run1:true
  text+qr      ops=16738 run2==run1:true run3==run1:true
  qr only      ops=13724 run2==run1:true run3==run1:true
  ```

  (`engrave.QR` was also read: all its state — `dim`, `cont`, `radius` — is
  declared *inside* its returned closure, so hoisting the construction out of
  `EngraveText`'s closure is safe.)

---

## 3. GOLDEN INTEGRITY — byte-identical, two independent ways

- **Tree hashes**, `f2007b7` → `f92bb02`:
  `backup/testdata` `36241aa9d9ccd63481d0c2ab28ef92fe4e526059` → **identical**;
  `gui/testdata` `76fd0f674d95a9adefda4d753ca4f61dcca0e014` → **identical**.
  `git diff --stat f2007b7..f92bb02 -- '*testdata*'` is empty; the diff touches
  9 source files and no data file.
- **Re-run**: `TestText` (all three goldens) and every gui golden suite pass —
  see §6.
- **Emission order preserved.** Read expression-by-expression: `start` ==
  old `titleRow (+ fontSize if Title)`; `limit` when `Footer != ""` ==
  old `footerRowY(params, plate.fontMM())`; `offy` advances by
  `len(lines)*fontSize` + `I(1)` between sections, as before; the closure draws
  paragraph text then that paragraph's QR, in the same order, then title, then
  footer. The footer's `centerRow` is now guarded on the string where it was
  previously guarded inside `centerRow` — an empty `Footer` emitted no yield
  either way. `TestTheTitleAndFooterAreEmittedLast` still passes.

**The re-anchored test** `TestEngraveTextDrawsTheQRAtItsParagraphsPlacement`:
**it still pins the same property** — the code is drawn at
`qrPlaceAt(P, qrc, scale, fontSize, offy)`, not at the plate margin — with the
same non-vacuity guard (`if offy == P.I(outerMargin) { t.Fatal(…) }`) and the
same control (with the code removed, nothing reaches the code's box). The
distinguisher moved from "a preceding paragraph" to "a title row", because the
old arrangement (code on paragraph 2 of 2) is now a refused plate. See Nit N4
for the one discriminating case that is lost — it is unreachable by construction
today.

---

## 4. CAPACITY — re-derived independently; it does not move

Measured at `prodParams`, 85-char md1 chunks, worst-case 18×`W` title+footer:

```
F(plateSize)=544000  I(outerMargin)=19200  I(innerMargin)=64000
bottom margin (footerless limit) = 524800
size=3.8  F=24320  LinesPerPlate=20  footerRowY=481280
yBudget(title,footer) = 43520, 481280
```

| paragraphs | nominal bottom | footer row | new check | deleted ink check (`ink ≤ row`) | agree? |
| --- | --- | --- | --- | --- | --- |
| 4 | 354560 | 481280 | accept | 348811 ≤ 481280 → accept | ✔ |
| **5** | **433920** | 481280 | **accept** | **428171 ≤ 481280 → accept** | ✔ |
| **6** | **513280** | 481280 | **refuse** | **507531 > 481280 → refuse** | ✔ |
| 7 | 616960 | 481280 | refuse | 611149 → refuse | ✔ |

The implementer's 433920 / 513280 / 481280 are **exact**. The nominal-vs-ink gap
is ~5750 units while the 5→6 step is 79360 (3 rows + 1 mm) and the slack at 5 is
47360 — the boundary is nowhere near the gap, which is why the two formulations
cannot disagree here (and, per §1(c), do not disagree anywhere reachable).

Note 6 paragraphs nominally end at 513280 — *inside* the 524800 plate bound.
That is F-435's whole premise: `toPlate` calls it a fit while it is cut over the
footer.

**Plate table, run:** `TestBundlePlanPacksACardOntoFewerPlates` passes
unmodified (1→1, 2→1, 3→1, 5→1, 6→2, 11→3), and `bundlePlateMD1Capacity` is
still 5, re-derived from the packer inside the test.

**Mutation M1 — disable the budget** (`if false && plate.Footer != "" && …`):

```
--- FAIL: TestABodyThatWouldCoverTheFooterIsRefused
    engravetext_test.go:490: EngraveText accepted a body that covers the footer row
--- FAIL: TestBundlePlanPacksACardOntoFewerPlates
    bundle_engrave_test.go:123: one more than fits splits, and splits ONCE: 6 strings → 1 plates, want 2
    bundle_engrave_test.go:123: and it keeps filling greedily: 11 strings → 2 plates, want 3
    bundle_engrave_test.go:133: the packer fits 6 85-char strings on a plate, not the 5 pinned here; …
--- FAIL: TestBundlePlanPlatesClearTheFooterRow
    bundle_engrave_test.go:223: plate 0 holds 6 strings and a 7th still lays out; …
    bundle_engrave_test.go:223: plate 1 holds 6 strings and a 7th still lays out; …
```

**Reproduced: with the budget gone the packer fits 6.** The budget is now what
enforces 5, and the pins are non-vacuous.

**Mutation M3 — delete the F-434 refusal:**

```
--- FAIL: TestEngraveTextRefusesAQROnAMultiParagraphPlate
    the QR is on the first of two paragraphs: EngraveText accepted it
    the QR is on the last of two paragraphs: EngraveText accepted it
    a text-less QR paragraph among text ones: EngraveText accepted it
```

and the gui suite stays green under M3 — confirming the implementer's claim that
the arrangement is unreachable from production. Verified by reading rather than
inferred: `buildQRPlates` re-creates `paras := []backup.Paragraph{}` each
iteration and appends exactly one element (gui/transaction.go:1456-1464);
`validateDescriptor` and the `len(strs)==1` branch of `validateMdmkStrings` each
build single-paragraph plates; `planTransactionTextPlates` packs multiple
paragraphs but never a QR. **The refusal costs no production caller anything.**

---

## 5. THE RESIDUE — measured, and the follow-up filing is a fact

*A QR band can still cross the footer row.* The budget counts text rows only, so
a code's band is outside it (`text-2-shards-1`: nominal 19200, ink 491698). This
is **not a regression** — the deleted `bundlePlateTextFits` check only ever saw
text-only paragraphs, and before this diff there was no check at all.

Reachable footered QR path: `engraveSingleSigFlow` → `bundleEngrave` →
`validateMdmkStrings` with `len(strs)==1`, which offers TEXT+QR / TEXT-ONLY /
QR-ONLY at `qrScale = 3` with a non-empty footer.

**Measurement at the longest strings that actually exist** (longest literals in
the tree: md1 = 85 chars, mk1 = 111 chars; ms1 = 128 but cardMS1 plates are
never marked, `bundlePlateMark`):

| string | modules | variant | body ink | footer row | **clearance** |
| --- | --- | --- | --- | --- | --- |
| md1, 85 ch | 37 | TEXT+QR | 319360 | 481280 | 161920 (**25.3 mm**) |
| md1, 85 ch | 37 | QR-ONLY | 377600 | 481280 | 103680 (**16.2 mm**) |
| mk1, 111 ch | 41 | TEXT+QR | 343040 | 481280 | 138240 (**21.6 mm**) |
| mk1, 111 ch | 41 | QR-ONLY | 389120 | 481280 | 92160 (**14.4 mm**) |

**Where the window actually opens** (swept module by module, ink measured, not
computed):

- **QR-ONLY: first crossing at 77 modules (QR v17), ≈459-character input** — ink
  492800 > 481280. At **73** modules the ink bottom is **exactly 481280**, i.e.
  touching the row, not past it. `toPlate` keeps accepting up to 85 modules
  (ink 515840 ≤ 524800), so 77..85 modules is the real window.
- **TEXT+QR: no window.** From 61 modules up (input ≥ 272 chars) the *text* rows
  overflow the footer row and the F-435 budget refuses the plate outright. At 57
  modules the body ink is 479828 — 1452 units (0.23 mm) under the row. The new
  text budget closes this variant before the band can matter.

**Conclusion: unreachable at today's md1/mk1 sizes**, by a factor of ≈4.2× in
input length (459 vs 111) and ≈1.9× in modules (77 vs 41). The implementer's
"≈73 modules, QR v14" is one version step conservative — it predicts the window
opening *earlier* than it does, so their "not reachable" conclusion holds a
fortiori. File the follow-up with **77 modules / QR v17 / ≈459 chars / QR-ONLY
only**, and record that TEXT+QR is now closed by the F-435 budget rather than
left open.

---

## 6. SUITES — run once each, captured

| gate | result |
| --- | --- |
| non-gui packages (`go test $(go list ./... \| grep -v /gui$)`) | **52 ok, exit 0**, no FAIL |
| `./backup/` (inside the above) | ok |
| gui shard (`gui-shard-test.sh ./gui/ 24`) | **`RESULT: ok -- all 1028 tests ran across 24 shards`**, wall **24s**, exit 0 |
| TinyGo device build (the exact CI line, `.github/workflows/test.yml:135`) | **exit 0**, flash **1197753**, stack **62576** — matches the report; `-print-stacks` reports only the pre-existing `runtime.runtimePanicAt may call itself` entries, none new |
| `go vet ./...` | **41 findings**, all pre-existing classes, **none in any file this diff touches** |
| `gofmt -l` on the 9 touched files | only `gui/transaction.go` — **confirmed unformatted at `f2007b7` too** (`git show f2007b7:gui/transaction.go` → `gofmt -l` lists it), drift is a blank-line run at line ~191, ~1000 lines from either hunk. Correctly left alone. |

---

## FINDINGS

**Critical: 0. Important: 0.**

### Nits (recorded, non-gating, no action required to merge)

**N1 — the residue's module figure is one version step off.** IMPL §6 states the
window opens at `qrsz > 418560` (≈73 modules, QR v14). Measured, 73 modules puts
the QR-ONLY ink bottom at exactly 481280 — *on* the footer row, not past it. The
first crossing is **77 modules (QR v17), ≈459-char input**. The error is in the
safe direction and does not change the "unreachable" conclusion; fix it in the
follow-up text so the filing is the measured number.

**N2 — IMPL §5 mis-attributes the `go vet` classes.** It says the 41 findings are
"all `testing.ArtifactDir requires go1.26`". Measured: **8** are that class; the
other **33** are `seedhammer.com/bezier.Point struct literal uses unkeyed fields`
in `bspline/bspline_test.go`. The count (41), the byte-identity to baseline, and
"none added" are all correct — only the class label is wrong.

**N3 — "six non-test call sites updated" (IMPL §1) counts one that was deleted.**
Five `EngraveText` call sites survive in non-test code; `bundle_flow.go`'s second
call (the ink measurement) was removed rather than updated. Six *touched*, five
*remaining*.

**N4 — the re-anchored QR-placement test lost one discriminating case.**
`TestEngraveTextDrawsTheQRAtItsParagraphsPlacement` previously distinguished
three anchors (the paragraph's own running `offy` / the body start / the plate
margin); with the code moved onto a single-paragraph plate under a title it now
distinguishes only two (body start / plate margin). An implementation that
anchored the code at `start` instead of at the running `offy` would now pass.
That case is unreachable today — F-434 means a QR may only appear on a
single-paragraph plate, where `offy == start` always — but it becomes reachable
again the moment F-434's *real* fix (`advance by max(textLines, qrLines)`)
re-admits multi-paragraph QR plates. Worth a line in F-434's entry: **when the
real fix lands, restore the two-paragraph anchor case to this test.**

### Residue for the controller (not findings)

- `design/FOLLOWUPS.md` is still un-updated, as the implementer states. F-434
  needs marking *cheap half done — the `max(textLines, qrLines)` fix still open*
  (plus N4's note); F-435 is fully resolved (`Text.FooterRow` and
  `bundlePlateTextFits`'s second check are both gone — `grep FooterRow` finds
  only prose and test names).
- New follow-up for the QR-band-across-footer window, with §5's numbers.
- `gui/transaction.go`'s pre-existing gofmt drift, standalone.

---

## THE ONE QUESTION, ANSWERED

**Is this diff safe to boot?** Yes.

- **No shipped plate changes** — testdata tree hashes identical at both
  directories, goldens re-run green, and the layout expressions are provably the
  same ones (`start`, `limit`, the `offy` accounting, the emission order).
- **No capacity change** — 5 per plate re-derived from the geometry, the plate
  table re-run, and the new check shown decision-identical to the deleted one
  across 1210 reachable (length, count) combinations, zero disagreements.
- **No new panic path** — no new deref, index or division; the one division
  (`footerRowY`) is now reached less often than before; the shared layout is
  proved re-invocable byte-for-byte.
- **No variant can silently vanish** — every caller drops-and-continues exactly
  as it does for `toPlate`, and the packing window is a strict subset of the
  engrave-time window, so the engraver cannot refuse what the packer admitted.
- **The departure is sound** — for a footerless plate the budget's `limit` is
  bit-for-bit `toPlate`'s own bound (both 524800), `toPlate` tests the true
  quantity there while the nominal row count is a proxy that is blind to QR ink
  entirely, and every path to steel passes `toPlate`. Binding the budget there
  would add nothing and would refuse `text-0-shards-1`, a shipped golden —
  reproduced.

**The merge proceeds.**
