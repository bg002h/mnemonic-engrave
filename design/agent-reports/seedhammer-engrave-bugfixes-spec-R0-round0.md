# Engrave-bugfixes SPEC — R0 review (round 0) — VERBATIM agent report

**Agent:** `a07f08aed3b2a9615` (adversarial opus architect; re-verified every citation, reproduced BUG-1 wrap + BUG-2 panic, and independently re-derived the BUG-3 geometry in a throwaway worktree off `f907eea`). **Fork HEAD:** `f907eea` (main, clean). **Spec commit:** `b1db950`. **Date:** 2026-06-19.
**Verdict:** GREEN (0 Critical / 0 Important). 3 non-blocking Minors (cosmetic clarity). Cleared for the implementation-plan phase (which gets its own R0). Persisted per the gate discipline.

---

# R0 GATE REVIEW — SPEC_seedhammer_engrave_bugfixes.md

**Reviewer:** opus architect (adversarial R0) · **Fork HEAD:** `f907eea` (verified `main`, clean) · **Spec commit:** `b1db950` · **Verdict: GREEN — 0 Critical / 0 Important** (3 Minor, all non-blocking polish/clarifications)

All three bug diagnoses, the proposed fixes, and the BUG-3 geometry were independently re-derived and empirically reproduced at `f907eea`. Every spec fact I checked is accurate. The spec is implementable as written.

---

## MANDATE 1 — File:line citation re-verification

Every citation checked is **ACCURATE** at `f907eea`:

| Citation | Verdict | Evidence |
|---|---|---|
| `engrave.go:1448` `progress uint` | ACCURATE | line 1448 `progress  uint` |
| `engrave.go:1452-1458` Resume prepends non-engrave move | ACCURATE | 1455 `appendLine(move, conf, false, …)` |
| `engrave.go:1460-1487` Progress loop | ACCURATE | exact range |
| `engrave.go:1467-1470` asymmetric guard | ACCURATE | 1467 `if k.Engrave && s.progress < k.T {` … 1470 `s.progress -= k.T` |
| `engrave.go:1472-1486` retire/safe-point select | ACCURATE | clamped-triple slide at 1479-1485 |
| `engrave.go:384-402` `bitmapForQRStatic` {21,25,29,33} | ACCURATE | switch at 392-400 |
| `engrave.go:399` `panic("unsupported qr code version")` | ACCURATE | line 399 |
| `engrave.go:406-410` `ConstantQR` eager call | ACCURATE | 410 `bitmapForQRStatic(dim)` |
| `engrave.go:616` second `bitmapForQRStatic` site | ACCURATE | 616 `bitmapForQRStatic(q.Size)` inside `ConstantQRCmd.Engrave` |
| `engrave.go:46-52` `F`/`I` defs | ACCURATE | `F=round(v*Millimeter)`, `I=Millimeter*v` |
| `bspline.go:24-28` Knot struct | ACCURATE | `{Ctrl bezier.Point; T uint; Engrave bool}` |
| `backup.go:75-87` `EngraveSeedString` | ACCURATE | `qr.Encode`(77) → `ConstantQR`(81), no guard between |
| `backup.go:89` `plateFontSize = 4.1` | ACCURATE | line 89 |
| `backup.go:161-225` `frontSideSeed`; 168 `pfs`; 172-176; 191-197; 204; 208-213 | ACCURATE | col-2 BOTTOM anchor `(plateDims.Y+col1Height)/2-height` at 211; QR at `I(60)-qrsz/2` at 204 |
| `gui.go:2078` `backupSeedStringFlow` `if err!=nil{return}` | ACCURATE | 2078-2081 |
| `slip39_polish.go:54-55` length set | ACCURATE | returns `[]int{20, 33, 23, 27, 30}` — **spec text `{20,33,23,27,30}` matches the actual slice order**. (The doc-comment at :49 lists `{20,23,27,30,33}` sorted — cosmetic mismatch in the *source comment*, not in the spec; see Minor-3.) |
| `slip39_polish.go:126,488-489,496-504` | ACCURATE | button :126; `Seed{Mnemonic: scan.Mnemonic …}` :488-489; `showError` backstop :498/:503 |
| `gui/engraver.go:158-160,197-216` | ACCURATE | `t,_:=res.Knot(k); safePoint.Knot(k); safePoint.Progress(t)`; resumer clamps `p:=max(0,s.progress)` :213 |
| `platform_sh2.go:177-181,188` | ACCURATE | `mm = 200/8 * Microsteps`; `strokeWidth = 0.3*mm` |
| `tmc2209.go:22-25` `Microsteps=1<<8` | ACCURATE | `stepExp=8`, `Microsteps=256` |

No DRIFTED or STRUCTURALLY-WRONG citations.

---

## MANDATE 2 — BUG-1 (RUN)

**Reproduced (probe, same-package):** seed-42 trace, reading `sp.progress` directly — **iter 0: `sp.progress = 1.84e19`** (wrapped), totalTicks=2771. Matches spec's "~1.8e19 on iteration 0." Minimal leading-move probe (`Knot{T:2771,Engrave:false}` then `Progress(0)`) wraps to ~1.84e19. The existing `TestSafePointer` passes despite this because its assertions only check longest-common-postfix and the skipped-engrave-knot property (lines 353-369) — **neither inspects `sp.progress`** (confirmed by reading the test).

**Units claim correct (NOT a units bug):** `gui/engraver.go:158-160` feeds `t,_ := res.Knot(k)` (completed step count) into `Progress`; `bspline.Knot.T` is in the same step unit. The resumer (`:213`) clamps to `max(0,…)`, so `Progress` receives a non-negative uint that can be 0 on the leading move. The wrap is the `k.Engrave &&` guard asymmetry, not a conversion error. **The spec correctly classifies this as a guard-asymmetry bug.**

**Fix is correct, not merely safe (probe-verified):** with line 1467 → `if s.progress < k.T {`, my independent simulation and the Go probe both show: leading move knot stays un-retired until `progress >= T`, then retires correctly; **no wrap across all 37 seed-42 iterations**; `sp.progress <= totalTicks` holds everywhere; and **the original `TestSafePointer` still passes** (no over-correction). Saturating-subtract rejection is sound — it would let a not-yet-elapsed move knot increment `completed` and feed the safe-point logic, masking the lag.

**Acceptance test:** the counter-invariant assertion (`sp.progress <= totalTicks` after every `Progress`) demonstrably FAILS on `f907eea` (iter 0) and PASSES after fix. The plan-shape requirement (leading move knot, `Progress(p<T)`) is the correct trigger. The safe-point reference assertion is well-specified enough: "control point of the most recent fully-elapsed clamped triple at the given `completed` level." Implementable; see Minor-1 for a sharpening suggestion.

---

## MANDATE 3 — BUG-2 (RUN)

**Reproduced (probe):** `EngraveSeedString` with a 93-char string → `qrc.Size=37` → **panics** `"unsupported qr code version"`; 127-char → `Size=41` → **panics**; 74-char → `Size=33` → **no panic, no error** (happy path intact). QR sweep under `qr.M`: len 74-90 → dim 33 (V4, last supported), 93-120 → dim 37, 125-127 → dim 41. Version→dim formula confirmed in `kortschak-qr@v0.3.2/coding/qr.go:663` `siz := 17 + int(v)*4` and comment "4v+17 pixels on a side." So dim 37=V5, 41=V6. **The `qrc.Size > 33` guard is the exact correct cutoff** (90→33 allowed, 93→37 blocked).

**Codex32 reachability confirmed:** `codex32/codex32.go:41-44` — short 48-93, long 125-127; `gui/codex32_polish.go` accepts both windows. 93-char short → dim 37 and 125-127 long → dim 41 both panic today. Reachability claim accurate.

**Caller fan-out / `:616` ruling:** `bitmapForQRStatic` has exactly 3 sites (`engrave.go:410`, `:616`, `engrave_test.go:90`). I verified `ConstantQRCmd` is constructed **only** at `engrave.go:477` inside `ConstantQR`, which calls `bitmapForQRStatic(dim)` eagerly at 410 *before* returning the cmd. Therefore any `ConstantQRCmd` reaching `Engrave` at `:616` already passed the size check at construction — **the `:616` site is unreachable with an unvalidated size.** The defense-in-depth conversion there is therefore *optional*, exactly as the spec states.

**Ruling on the open item:** the spec's recommended resolution — **early size-check inside `ConstantQR` (before line 410) returning the error, mandatory `EngraveSeedString` guard, leave `panic` as an unreachable assertion** — is sound and the cleanest choice (avoids disturbing `Engrave`'s signature). I ratify it. Converting `bitmapForQRStatic`→error is acceptable but not required.

---

## MANDATE 4 — BUG-3 geometry (INDEPENDENTLY RE-DERIVED)

**Scale constants (recomputed):** `mm = 200/8*256 = 6400`; `pfs = F(4.1) = round(4.1*6400) = 26240`; `plateY = F(85) = 544000`. All match.

**Legacy layout (my independent recomputation = spec table EXACTLY):**

| N | col2-TOP (mm) | col2-BOTTOM (mm) | overlap |
|---|---|---|---|
| 20 | [9.70,26.10] | none | gap |
| 23 | [9.70,26.10] | [63.00,75.30] | gap |
| 24 | [9.70,26.10] | [58.90,75.30] | gap |
| 27 | [9.70,26.10] | [46.60,75.30] | gap |
| 30 | [9.70,26.10] | [34.30,75.30] | gap |
| **33** | [9.70,26.10] | **[22.00,75.30]** | **+4.10mm OVERLAP** |

Only N=33 overlaps (col2-TOP bottom 26.10mm vs col2-BOTTOM top 22.00mm). Confirmed.

**A1 fix (my independent recomputation = spec table):**

| N | col1Rows/col2Rows | pfsN (mm) | col-1 y (mm) | col-2 y (mm) | gap |
|---|---|---|---|---|---|
| 25 | 13/12 | 4.100 | [15.85,69.15] | [15.85,65.05] | 4.10 |
| 27 | 14/13 | 4.100 | [13.80,71.20] | [13.80,67.10] | 4.10 |
| 30 | 15/15 | 4.100 | [11.75,73.25] | [11.75,73.25] | 0.00 |
| **33** | **17/16** | **3.859** | **[9.70,75.30]** | **[9.70,71.44]** | **3.86** |

**N=33 confirmed:** `pfsN = 16*26240/17 = 419840/17 = 24696` units = **3.859mm**; col-1 [9.70,75.30] (= legacy col-1 extent, so title/mfp envelope unchanged), col-2 [9.70,71.44], **gap 3.86mm, no overlap**, all ⊂ [0,85]mm. (Minor numeric note: A1 col1H for N=33 = 24696*17 = 419832 vs legacy 419840 — 8 units / 0.00125mm rounding slack from integer division; negligible.)

**Width check (load-bearing) — verified the scaling assumption from source:** glyph advance scales linearly with `em` — `engrave.go:1364` `dot.X += adv*s.em/mh`, `:1239` `advDist = adv*em/fh`. So block width = 32.80mm·(pfsN/26240) = **30.87mm at pfsN=3.859mm → col-2 right edge ≈ 74.87mm < 85mm.** Fits. (Integer truncation makes the real width slightly smaller — conservative.)

**A2 rejection sound:** one block = 32.80mm at full font; 3 anchors {8,33,58}mm → rightmost right edge = 90.8mm > 85mm. A2 genuinely overruns at a legible font; A1 is strictly better. The anchors are not chosen uncharitably — even at the most compact 3-anchor packing the third block can't fit a full-font 32.8mm column inside 85mm. Confirmed infeasible.

**NO-REGRESSION (load-bearing) — verified true of the parameterization:** The `N>24` branch is genuinely unreachable for N≤24, so the legacy code path runs byte-identically for {12,18,20,23,24}. I additionally confirmed reachable N: BIP-39 GUI offers only **{12,24}** (gui.go:2124, derive_xpub.go:89), BIP-85 **{12,18,24}** (bip85.go:144); SLIP-39 **{20,23,27,30,33}**. So reachable `frontSideSeed` N = {12,18,20,23,24,27,30,33} — the spec's no-regression list {12,18,20,23,24} covers exactly the reachable ≤24 set (15/21 are unreachable; not needing pins is correct). **Every QR-bearing plate is BIP-39 ≤24** — I verified `Seed.QR` is set only in `engraveSeed` (gui.go:480-488) from a `bip39.Mnemonic`; the SLIP-39 path (slip39_polish.go:488) sets no QR. So the `N>24` branch never coincides with a QR. The no-regression invariant holds **structurally**, provided implementation keeps the gate as the sole behavioral edit and does not restructure the shared ≤24 prologue (the golden pins are the enforcement).

**PREDICATE-BREADTH ruling (explicit):** The `N>24` gate changes N=27 and N=30 from their already-correct legacy layouts to the rebalanced split. I confirmed they DO change (e.g. N=30 legacy col-1 [9.70,75.30] vs A1 [11.75,73.25]; N=27 legacy [9.70,75.30] vs A1 [13.80,71.20]). This is a genuine behavioral change to currently-correct output. **My ruling: acceptable-as-designed (Minor, not Important).** Reasoning: (1) the change is purely cosmetic balance (col1Rows≈col2Rows instead of 16+4-top+rest-bottom) and stays at full 4.1mm font — no legibility/engraveability change; (2) it is *more* visually consistent across the SLIP-39 family (all >24 counts use one rule) and removes the two-block col-2 collision mechanism entirely rather than special-casing only N=33; (3) the spec already pins N=30 to the new table (acceptance #2), so the change is intentional and tested. A narrower predicate (`N>30` or `N==33`) would leave N=27/30 on the legacy two-block path that *can* overlap as N grows and is the very mechanism being removed — narrowing would preserve a latent footgun for marginal benefit. The broad gate is the better design. **I recommend (Minor) the spec add one sentence making the deliberate N=27/30 rebalance explicit in the no-regression section**, so the implementer/post-review doesn't mistake the N=27/30 golden change for a regression. (Reachable >24 = {27,30,33}; **N=25 in the A1 table is illustrative/unreachable — confirmed, no path emits 25 words.** Spec should label it as such — Minor-2.)

**Acceptance tests sufficient:** N=33 no-overlap + pinned `pfsN==24696` FAILS on f907eea (overlap) and PASSES after. The {20,23,24}+QR byte-identical pins guard the #1 risk; N=30 new-path pin guards the predicate-breadth change. One coverage nuance: there is **no committed golden for N=23 SLIP-39 today**, so a "byte-identical to current" pin for N=23 is really a *freshly generated* pin (it asserts the new code leaves the legacy path producing the legacy formula's output — still valid, since N=23≤24 takes the untouched path — but it is not literally a diff against a pre-existing committed file). The existing committed goldens that DO enforce zero-churn are `seed-*-words-{12,24}` (BIP-39+QR) and `slip39-0` (20-word). This is adequate; noted as Minor-2.

**Legibility residual:** stroke is fixed at 0.3mm (`strokeWidth = 0.3*mm`, independent of `em` — confirmed glyph stroke comes from `strokeWidth`, font `em` only sets height/advance). For pfsN=3.859mm, em/stroke = 3.859/0.3 = **12.86 ≈ 12.9×** — spec's "≥12.9× stroke" is correct (rounds to 12.9). Flagging legibility as a **non-blocking R0 residual is the right call**: geometry + engraveability are proven; whether 3.86mm *reads* cleanly is a subjective/hardware judgement, and SeedHammer already engraves a 24-word plate + QR at finer pitch in the same area. Not a blocker.

---

## MANDATE 5 — Cross-cutting

**Scope:** firmware-only, confirmed. Touches `engrave/engrave.go`, `backup/backup.go` + their tests. No `me`/CLI/schema/docs-mirror surface; no `md`/`mk`/`codex32`/`ms1` codec edits (BUG-2 only *consumes* an already-formed codex32 string at engrave time). Confirmed accurate.

**No internal contradictions.** Open items 1-3 in the Gate section are all resolved by the spec's recommendations, which I ratify (BUG-2 early-check-in-`ConstantQR`; BUG-3 A1 geometry; BUG-1 break-based guard). No dangling items. No acceptance test is non-load-bearing: each FAILS on f907eea and PASSES after (BUG-1 counter-invariant, BUG-2 panic→error, BUG-3 N=33 overlap).

---

## Critical / Important

**None.**

## Minor (non-blocking — fold opportunistically, not gate-blocking)

1. **BUG-1 safe-point reference sharpening.** The reference is well-specified prose but the implementer would benefit from the spec stating it as a closed form: "the `k0.Ctrl` of the last clamped triple `(k0,k1,k2)` all of whose `T` sums to ≤ `completed` ticks; assert it never selects a triple whose start exceeds `completed`." Optional clarity; current wording is implementable.
2. **BUG-3 table/golden labeling.** Mark the N=25 A1-table row as *illustrative/unreachable* (no GUI path emits 25 words; reachable >24 = {27,30,33}). And clarify that the N=23 "byte-identical" pin is a freshly-generated pin over the *untouched* ≤24 path (the committed zero-churn guards are the 12/24-word BIP-39+QR and 20-word SLIP-39 goldens).
3. **Predicate-breadth note + source-comment nit.** Add one explicit sentence to the no-regression section noting N=27/30 *intentionally* move to the rebalanced split (so their golden change isn't misread as a regression during post-review). Separately, `slip39_polish.go:49`'s doc-comment lists the set as sorted `{20,23,27,30,33}` while the returned slice is `{20,33,23,27,30}` — purely cosmetic, out of this cycle's scope, mention only if a one-line comment touch-up is convenient.

## Verified-correct (independently confirmed)

- Every file:line citation in the spec (MANDATE 1 table) — all ACCURATE at f907eea.
- BUG-1 underflow reproduces to 1.84e19 on iteration 0; fix yields no-wrap + correct retirement + original test still green (probe + independent simulation).
- BUG-1 is a guard-asymmetry bug, not a units bug; saturating-subtract rejection is sound.
- BUG-2 panics on dim 37 (93-char) and dim 41 (127-char), happy path (dim 33) intact; `>33` cutoff exact; codex32 reachability 48-93/125-127.
- `ConstantQRCmd` built only inside `ConstantQR` → `:616` unreachable with unvalidated size; spec's open-item resolution ratified.
- BUG-3 legacy overlap table (only N=33 overlaps, +4.10mm) and A1 table (N=33: 17/16, pfsN=24696=3.859mm, col-1 [9.70,75.30], col-2 [9.70,71.44], gap 3.86mm) — recomputed independently, match.
- A1 width fits (block 30.87mm, right edge 74.87mm < 85mm); advance scales linearly with em (source-verified).
- A2 genuinely overruns 85mm (right edge 90.8mm); rejection sound.
- No-regression structurally true: `N>24` unreachable for ≤24; every QR plate is BIP-39 ≤24; reachable N = {12,18,20,23,24,27,30,33}.
- Legibility floor: stroke fixed 0.3mm, em/stroke=12.9×; legibility-as-non-blocking-residual is correct.
- Scope is firmware-only; no codec/CLI/schema surface.

---

## Bottom line

**Verdict: GREEN (0 Critical / 0 Important).** The spec is factually accurate to the last cited line, all three bugs reproduce as described, all three proposed fixes are independently verified to work and not regress, and the BUG-3 geometry is correct on independent re-derivation. The open items are resolved soundly. **No change is required to reach GREEN** — implementation may proceed under the project's TDD + single-subagent-in-worktree process. The three Minor items are optional clarity folds (chiefly: label N=25 as unreachable, and add one sentence flagging the deliberate N=27/30 rebalance so post-review doesn't mistake the golden change for a regression).

**Per project policy:** since this fold (if the Minor items are applied) would be cosmetic-only, a single re-dispatch after folding suffices; if the spec is shipped as-is (Minors deferred to implementation notes), this GREEN stands.

---

**Fork state confirmed at end:** `/scratch/code/shibboleth/seedhammer` is on `main @ f907eea`, `git status --porcelain` empty (clean). Throwaway worktree `/tmp/r0-engrave-x` removed and pruned. The pre-existing `seedhammer-wt-bip39` worktree (not created by this review) is untouched. No leftover probe files.
