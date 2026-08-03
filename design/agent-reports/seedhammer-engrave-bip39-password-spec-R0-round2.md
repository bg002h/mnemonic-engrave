# Sonnet verification — R0 round 2 — SPEC_seedhammer_engrave_bip39_password.md

- **Reviewer role:** verification pass (not an audit), round 2 of the mandatory pre-implementation R0 gate.
- **Spec under review:** `design/SPEC_seedhammer_engrave_bip39_password.md` @ `0079440` (post round-1 fold)
- **Brief:** did the round-1 fold correctly fix I1, I2, I3, I4, M1, N1, N2 — and did it introduce a new defect? Round 0's claim verifications, round 0's C1/C2 closure, and round 1's fold verification of the round-0 findings were all declared established fact and out of scope. A clean result closes the gate.
- **Round 0:** NOT GREEN (2C/6I). **Round 1:** NOT GREEN (0C/4I).
- **Model tier:** sonnet, per the project's reviewer-tiering rule — the design questions were settled by rounds 0 and 1, leaving a mechanical "does the fold match the findings" check.
- **Date:** 2026-08-03

Persisted verbatim, per the project standard.

---

## VERDICT
GREEN (0C/0I)

All six required fixes (I1–I4, M1, N1) plus N2 verified correctly folded against current source at `/scratch/code/shibboleth/seedhammer`. No Critical or Important findings. Two Nits noted (non-blocking).

## FOLD VERIFICATION

**I1 FIXED** — §3.5.2 now names all three sites with correct cites, verified exact against source: `constantTimeQRModules` (`engrave/engrave.go:349-365`, switch falls to `return 0` at :364), `bitmapForQRStatic` (`:384-401`), `ConstantQR` guard (`:406-413`, `if dim > 33` at :408). Text mandates the v5/v6 maxima be "derived by extending the existing fuzz corpus (`engrave/testdata/fuzz`), not estimated" — corpus directory confirmed to exist (`engrave/testdata/fuzz/FuzzConstantQR`).

**I2 FIXED** — §4.3's band table splits fingerprints (top, ≤2 lines) from legend+footer (bottom, ≤2 lines), exactly as round 1's fix prescribed. Arithmetic confirmed: `outerMargin=3`, `innerMargin=10` (`backup/backup.go:46-47`) → band = 7mm; `plateSmallFontSize=3` (`backup.go:95`) → 2 lines = 6mm ≤ 7mm. No band now holds 3 lines.

**I3 FIXED** — §3.5.1 makes the separate alphabet/`ConstantStringer` instance the unconditional default ("MUST NOT be widened... Add a second alphabet constant and construct a separate `ConstantStringer` instance"), not a fallback. Verified `NewConstantStringer` (`engrave.go:1195-1247`) derives all three values from the alphabet: `runeDuration` (maxDur), `startEndDist = ManhattanDist(bounds.Min, bounds.Max)`, `center = bounds.Max.Add(bounds.Min).Div(2)`. O5 correctly broadened to require measuring all three if the shared alphabet is ever widened instead.

**I4 FIXED** — §3.4 table now lists five checks, all cites verified exact: check 4 ascending order (`:1208`/panic `:1210`, confirmed `if r < lastr { panic("unsorted alphabet") }`), check 5 alphabet⊆face (`:1213`/panic `:1215`, confirmed `face.Decode(r)` / `panic("unsupported rune")`). §3.5.1.1 states the mark sorts first ("single ascending string: `<mark>` then `0x20`–`0x7E`") and that glyphs + alphabet "must land in the same commit."

**M1 FIXED** — §3.2.1 now includes "Case-only collisions... `C/c O/o S/s U/u V/v W/w X/x Z/z K/k`" verbatim as required, and O1 references "every confusable pair enumerated in §3.2.1" so the eye-check automatically covers it.

**N1 FIXED** — `bitmapForQRStatic` cite corrected to `:384-401`, verified exact against source.

**N2 FIXED (with a Nit)** — §3.5.1.1 adds "No new glyph may start at (0,0)" with the `paddedString` sentinel rationale. Substance is correct (`inf.Start != (bezier.Point{})`), but the cited range `engrave.go:1288-1295` stops one line short of the actual conditional, which is at line **1296** (`if inf.Start != (bezier.Point{}) {`). Cite drift only — the described mechanism is accurate. Non-blocking.

**FINGERPRINT-GROUPING — consistent.** §3.4 ("canonical stripped form is the only stored and compared value; the 4-and-4 grouping is presentation only"), §4.3 (separator "a plain space, never the visible-space mark"), §5 (display grouped 4-and-4 at entry), and §7 (canonicalisation test: mixed-case/whitespace-variant input → identical stored value, plain `0x20` separator not the mark) all agree with no contradiction.

## NEW FINDINGS

**Nit — N2 cite off by one line.** `engrave.go:1288-1295` should extend to include line 1296, where the actual `if inf.Start != (bezier.Point{})` sentinel lives (currently the range ends on the preceding comment line). Inherited verbatim from round 1's own finding text, not introduced by the spec author. Cosmetic only.

**Nit — §10 review history not updated.** The "Review history" section still only documents the round-0 fold; no entry summarizing round 1's fold (I1–I4/M1/N1/N2) has been added. Doesn't affect correctness — the round-1 report is separately persisted verbatim per project convention — but leaves the in-spec audit trail one round behind.

## CONSISTENCY

- §3.4 five-check table vs §3.5.1.1 constraints: **Pass** — cites for checks 3/4/5 (`:1216-1218`, `:1208-1210`, `:1213-1215`) match exactly between the two sections.
- §4.3 band budgets vs §4.1/§4.2 height budgets: **Pass** — margin bands (10mm, 7mm engraveable) are independent of the 65mm usable-area content block (60mm at 4.1's max, 56mm at 4.2's QR max); no overlap or overflow in either mode.
- §7 assertion coverage: **Pass** — "Worst-case layout fit" explicitly asserts "≤ 2 lines per margin band" and "no metadata line wider than 64 mm"; "Fingerprint canonicalisation" explicitly asserts identical stored value + plain-space separator.

No Criticals or Importants found. Gate closes GREEN.

---

## Disposition (author)

Both nits fixed inline: the sentinel cite corrected to `engrave.go:1294-1296` (verified — line 1296 is `if inf.Start != (bezier.Point{}) {`), and §10 brought current with round-1 and round-2 entries. Per the project's proportional re-review rule, wording/cite folds do not re-trigger a gate, and a re-review returning 0C/0I closes the loop.

**R0 GATE CLOSED — GREEN. Implementation may begin.**
