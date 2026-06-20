# SPEC — SeedHammer II engraving-subsystem bug fixes (3 confirmed)

- **Status:** DRAFT — awaiting opus R0 gate (must reach 0C/0I before any implementation).
- **Type:** Brainstorm SPEC (NOT a plan, NOT code). Single-author per project policy.
- **Cycle:** bug-fix. Source: `design/agent-reports/seedhammer-engrave-bughunt.md` (3 CONFIRMED of 31 raw findings).
- **Fork / source of truth:** `/scratch/code/shibboleth/seedhammer`, branch `main`, HEAD `f907eea`. Go via `export PATH=$PATH:/home/bcg/.local/go/bin`.
- **Nature:** UPSTREAM / inherited bugs (correctness + hardware-safety). Fix faithfully; do NOT regress resume/safe-point or plate-layout semantics. Fork fix MAY later be offered upstream — out of scope this cycle.

---

## Why (one paragraph)

The adversarial bug-hunt (36 agents, REFUTE-by-default verifier) confirmed exactly three bugs in the inherited engraving subsystem, all hardware-reachable on the RP2350 firmware (no watchdog → an uncaught panic or a corrupt safe-point can ruin a plate, crash the head, or hang the controller). All three are pre-existing upstream defects in `engrave/engrave.go` and `backup/backup.go`; none are in our T-series / m-format / bip85 additions. This is a firmware-only cycle.

---

## BUG-1 [CRITICAL] — `SafePointer.Progress` uint underflow on move knots

### Verified facts (file:line)
- `engrave/engrave.go:1448` — `progress uint` (the wrapping field).
- `engrave/engrave.go:1460-1487` — `SafePointer.Progress(p uint)`: the advance loop.
- `engrave/engrave.go:1467-1470` — the asymmetric guard:
  ```go
  if k.Engrave && s.progress < k.T {   // 1467 — only protects ENGRAVE knots
      break                             // 1468
  }                                     // 1469
  s.progress -= k.T                     // 1470 — unconditional; wraps for move knots
  ```
- `bspline/bspline.go:24-28` — `Knot{Ctrl bezier.Point; T uint; Engrave bool}`.
- `engrave/engrave.go:1452-1458` — `SafePointer.Resume` prepends a leading **non-engrave** move (`appendLine(..., false, ...)`), so the trigger shape is the universal start of every engraving, not a rare edge.
- `engrave/engrave.go:1472-1486` — the history-retire / safe-point selection that runs *after* the subtraction: it advances `s.completed`, and when the last 3 retired knots form a clamped triple (`k0.Ctrl==k1.Ctrl && k1==k2`) it slides `s.history`, resets `s.completed=0`, and sets `s.safePoint=k0.Ctrl`.
- `gui/engraver.go:158-160` — production driver of `Progress`: `t, _ := res.Knot(k); s.safePoint.Knot(k); s.safePoint.Progress(t)`.
- `gui/engraver.go:197-216` — `splineResumer.Knot`: `t` is the driver-reported **completed step count**, clamped `>= 0` (`p := max(0, s.progress)`), so `Progress` always receives a non-negative `uint` that can be **0** on the leading move knot.
- **Units confirmed equivalent (verifier-authoritative):** `k.T` (ticks) == pio steps for the knot (`bspline.Segment.Knot` → bezier interpolator produces exactly `k.T` steps). The reported `completed` lags only because `stepper.Driver` flushes whole words (`stepsPerWord=6`); a partial word is withheld. So `s.progress` and `k.T` are the same unit — **the fix is the guard asymmetry, NOT a units conversion.**
- **The existing test passes despite the wrap** — `engrave/engrave_test.go:321-371` `TestSafePointer`: the verifier traced seed 42 and found `s.progress` wraps to ~1.8e19 on **iteration 0**; the test passes only because its assertions check a longest-common-postfix and a "skipped engrave knot earlier than completed" property — neither inspects the counter.

### Fix
Make the early-`break` guard **symmetric** (independent of `k.Engrave`): break whenever `s.progress < k.T`, for both move and engrave knots. Concretely, change line 1467 from `if k.Engrave && s.progress < k.T` to `if s.progress < k.T`. This guarantees `s.progress -= k.T` (line 1470) only executes when `s.progress >= k.T`, so the unsigned subtraction can never underflow.

- **Rationale this is correct (not just safe):** A knot is only "fully elapsed" once the cumulative reported progress reaches its duration `k.T`. The original code retired move knots eagerly on the assumption that `s.progress >= k.T` always held for them — false, because the driver lags by up to a partial word on the *leading* knot. Breaking on `s.progress < k.T` for move knots too means a leading move knot stays un-retired until its `k.T` ticks have actually been reported; on the next `Progress(p)` call (cumulative `s.progress += p`) it retires correctly. This yields **eventual, correct** retirement with no wrap, and — crucially — leaves the safe-point selection (1472-1486) reading **valid** state instead of post-wrap garbage.
- **Equivalent acceptable alternative (implementer's choice, same invariant):** keep the structure but guard the subtraction — `if s.progress < k.T { break }` immediately before line 1470 (functionally identical once the guard is unconditional). A saturating-subtract (`s.progress = saturatingSub(s.progress, k.T)`) is **rejected**: it would let a not-yet-elapsed move knot still increment `s.completed` and feed the safe-point logic, masking the lag rather than respecting it. The fix MUST be break-based so retirement waits for true elapse.

### Invariant
Across any legal interleaving of `Knot`/`Progress` calls: **`s.progress` never underflows** (never exceeds the sum of `k.T` over all knots fed so far; equivalently it is monotone-bounded by total reported progress). AND the selected `s.safePoint` after each `Progress` call equals the most recent fully-elapsed clamped-triple control point — i.e. the largest safe point not later than `completed` ticks of real progress (the *correct* resume target).

### Acceptance (TDD — MUST fail on current code, pass after fix)
Add `TestSafePointerNoUnderflow` (or extend `TestSafePointer`) in `engrave/engrave_test.go`. It MUST:
1. **Wrap-catcher (the load-bearing assertion):** drive a plan whose **first retired history knot is a leading move knot with `T > 0`** and feed `Progress` a step count **smaller than that knot's `T`** (reproducing the universal real-start: leading move `Engrave=false`, e.g. `T=2771`, `Progress(0)` then small increments). Assert after **every** `Progress` call that `sp.progress <= totalTicks` (sum of all `k.T` fed so far), where `totalTicks` is tracked independently. On current code this fails immediately (iteration 0 yields `sp.progress ≈ 1.8e19 >> totalTicks`); after the fix it holds. Expose the field for the test via same-package (`package engrave`) test access — `sp.progress` is package-visible to `engrave_test.go`.
2. **Correct-safe-point assertion:** for a leading-move-knot plan, assert the selected `sp.safePoint` matches an independently-computed reference: the control point of the most recent fully-elapsed clamped triple at the given `completed` level (NOT merely "the wrap didn't happen"). Construct the plan so a clamped triple exists and verify the safe point advances to it only after enough progress is reported, and never selects a *later* (not-yet-reached) safe point.
3. **Non-regression:** the existing `TestSafePointer` seed-42 loop (and its postfix / skipped-engrave-knot properties) MUST still pass unchanged.

> R0 note: the new test's value is entirely in asserting the **counter invariant** and the **safe-point reference**, because the existing assertions demonstrably tolerate the wrap. The plan-shape MUST be a leading move knot, since that is what wraps.

---

## BUG-2 [HIGH] — uncaught QR-version panic in `EngraveSeedString`

### Verified facts (file:line)
- `backup/backup.go:75-87` — `EngraveSeedString`: `qr.Encode(seed, qr.M)` (77) → `engrave.ConstantQR(qrc)` (81). No guard on QR size between them.
- `engrave/engrave.go:384-402` — `bitmapForQRStatic(dim)`: supports only `{21,25,29,33}`; **`default: panic("unsupported qr code version")`** at line 399.
- `engrave/engrave.go:406-410` — `ConstantQR` calls `bitmapForQRStatic(dim)` **eagerly** (line 410), so the panic escapes `ConstantQR` and `EngraveSeedString` rather than being returned as `err`.
- `qr.Code.Size` is the dimension (pixels/side); version V → dim `17+4V`. So {21,25,29,33}=V1-4 (supported); **dim 37=V5, dim 41=V6 → panic.** (Confirmed in `kortschak-qr@v0.3.2/qr.go:79,86`.)
- Reachability: `gui/gui.go:2078` (`backupSeedStringFlow`) only checks `if err != nil { return }` — a panic bypasses it; **no `recover()` anywhere in gui/backup/engrave**; RP2350 firmware has no watchdog → controller crash/hang. The codex32 entry (`gui/codex32_polish.go`) accepts short codes up to 93 chars (→ dim 37) and the 125-127 long-code window (→ dim 41). Verifier reproduced: 93-char short code → dim 37 → panic; 127-char long vector → dim 41 → panic.

### Scope decision (defense-in-depth)
**Do BOTH, scoped:**
- **(Primary) Guard in `EngraveSeedString`** — after `qr.Encode`, before `ConstantQR`: `if qrc.Size > 33 { return nil, errors.New("seed too long to engrave QR") }`. This routes the existing `err != nil` path and is the minimal correctness fix.
- **(Defense-in-depth) Make `bitmapForQRStatic` return an error** instead of `panic`, and have `ConstantQR` propagate it — so the *other* `ConstantQR` caller path and any future caller is also panic-proof. **Caller fan-out for this return-type change (verified):** `bitmapForQRStatic` has 3 call sites — `engrave.go:410` (`ConstantQR`, already returns `error`), `engrave.go:616` (inside `ConstantQRCmd.Engrave`, **second call site**, must thread/handle the error), and `engrave_test.go:90` (test). `ConstantQR` callers (`backup.go:66` `EngraveSeed`, `backup.go:81` `EngraveSeedString`) already handle `err`. **R0-decide:** the `engrave.go:616` site (`ConstantQRCmd.Engrave`, the plan executor) may not have an ergonomic error-return; if converting it cleanly is non-trivial, the implementer MAY keep `bitmapForQRStatic` panicking *internally there* but the **primary guard in `EngraveSeedString` is mandatory and sufficient for the confirmed bug**. The defense-in-depth conversion is REQUIRED only at the `ConstantQR` path; the `ConstantQRCmd.Engrave` path is GUARDED upstream because its size was already validated at `ConstantQR` time (its `plan` is built only by `ConstantQR`, per the refuted-findings note). Net: convert `bitmapForQRStatic` → `(…, error)` is the cleaner choice IF the `:616` site threads it without disturbing `Engrave`'s signature; otherwise add an early size check inside `ConstantQR` proper (before line 410) returning the error, leaving the `panic` only as an unreachable assertion.

### Fix (summary)
Primary: size guard in `EngraveSeedString` returning a clean error for `qrc.Size > 33`. Defense-in-depth: a size check in `ConstantQR` (before `bitmapForQRStatic`) returning the same error class, so no `ConstantQR` caller can panic. Keep the `panic` only as an unreachable invariant assertion (or convert to error if the `:616` thread is clean).

### Invariant
`EngraveSeedString` (and `ConstantQR`) **never panic on caller-reachable input**: any `seed` whose QR exceeds dim 33 returns a non-nil `error`; any `seed` within {21,25,29,33} engraves as before.

### Acceptance (TDD — MUST fail on current code, pass after fix)
In `backup/backup_test.go`:
1. **Panic→error:** call `EngraveSeedString` with a real valid codex32 string that encodes to dim 37 (e.g. the 93-char short code from the report) and a dim-41 long vector (127-char). Assert it returns a non-nil error and **does not panic** (use a `recover`-wrapped helper or rely on the test simply not crashing). On current code this panics (test fails/crashes); after the fix it returns an error.
2. **Happy path unchanged:** a normal-length seed (dim ≤ 33) still returns a valid `Engraving`, no error.
3. (If the defense-in-depth conversion is implemented) an `engrave`-package test that `ConstantQR` on a dim-37 `*qr.Code` returns an error rather than panicking.

---

## BUG-3 [HIGH] — `frontSideSeed` 33-word SLIP-39 plate overlap

> **USER DECISION (2026-06-20): BUG-3 = support 33-word via layout rework (option a).** The user elected to *support* verbatim engraving of 33-word (256-bit) SLIP-39 shares — the highest-security SLIP-39 form — rather than reject them with a guard (the previously-recommended option b). This section is re-authored for the **layout rework**. The reject-guard is no longer the fix; the deliverable is a parameterized large-N layout that places all words without overlap, leaving every ≤24-word and QR-bearing layout coordinate-identical.

### Verified facts (file:line)
- `backup/backup.go:161-225` — `frontSideSeed`. Legacy layout constants `maxCol1=16, maxCol2=4` (172-173); `pfs := params.F(plateFontSize)` (168) with `plateFontSize = 4.1` (`backup.go:89`); plate `image.Point{X: F(85), Y: F(85)}` (163-166).
- Col-1: words `0..endCol1` (`endCol1=min(16,N)`), anchored at `(plateY-col1Height)/2`, `col1Height = pfs*endCol1` (175-176, 191-192).
- Col-2 TOP: words `endCol1..endCol2` = `16..min(20,N)`, anchored at `(plateY-col1Height)/2` (195-197).
- Col-2 BOTTOM: words `endCol2..N`, height `(N-endCol2)*pfs`, anchored at `(plateY+col1Height)/2 - height` (208-213). **The two col-2 blocks grow toward each other from opposite ends of the same `col1Height` band.**
- Column x-anchors: col-1 at `innerMargin = I(10)` = 10mm (191); col-2 at `I(44)` = 44mm (196, 211); the QR (BIP-39 path only) at `I(60)-qrsz/2` (204), centered in the right third. The SLIP-39 verbatim path passes `qrc==nil` (`backup.go:71` is called with the `qrc` built in `EngraveSeed`, which is `nil` when `plate.QR==nil`; SLIP-39 sets no QR), so the QR slot is free on the large-N path but **the routine is shared** with the BIP-39+QR path.
- **Production scale constants (verified):** `mm = fullStepsPerRevolution/mmPerRevolution * Microsteps = 200/8 * 256 = 6400` machine units/mm (`cmd/controller/platform_sh2.go:177-181`; `Microsteps = 1<<stepExp = 1<<8 = 256`, `driver/tmc2209/tmc2209.go:22-25`). `pfs = F(4.1) = round(4.1*6400) = 26240` units (4.1mm). `plateY = F(85) = 544000` (85mm). `strokeWidth = 0.3*mm` = 0.3mm, **fixed** (`platform_sh2.go:188`; test `backup_test.go:49` uses `mm/3 ≈ 0.33mm`). `F(v)=round(v*Millimeter)`, `I(v)=v*Millimeter` (`engrave/engrave.go:46-52`).
- **Minimum feature size:** the engrave path renders glyph strokes at the fixed `strokeWidth` (0.3mm) regardless of font `em`; font scale (`pfs`) only sets glyph height/advance. So shrinking the font does **not** shrink the stroke — the engraveability floor is the 0.3mm stroke, unchanged. Legibility floor is the font em relative to that stroke.
- **Column block width (measured against the real `constant.Font` at `pfs=F(4.1)`):** number prefix `"NN "` = 8.20mm, longest SLIP-39 word (`ShortestWord=4, LongestWord=8`, `slip39/wordlist.go:7-9`) padded = 24.60mm → **one column block = 32.80mm wide** at the full font. Two columns at x=10mm and x=44mm already reach a right edge of **76.8mm** on the 85mm plate.
- No check that the bottom block fits below the top block. `gui.toPlate` (`gui/gui.go:~2817-2830`) rejects only off-plate bounds (`ErrTooLarge`); **on-plate overlap passes validation** and is engraved.
- **Reproduced legacy layout math at production scale — matches verifier exactly:**

  | N  | col2-TOP y (mm) | col2-BOTTOM y (mm) | overlap |
  |----|-----------------|--------------------|---------|
  | 20 | [9.70, 26.10]   | (none)             | gap     |
  | 23 | [9.70, 26.10]   | [63.00, 75.30]     | gap     |
  | 24 | [9.70, 26.10]   | [58.90, 75.30]     | gap     |
  | 27 | [9.70, 26.10]   | [46.60, 75.30]     | gap     |
  | 30 | [9.70, 26.10]   | [34.30, 75.30]     | gap     |
  | **33** | [9.70, 26.10] | **[22.00, 75.30]** | **+4.10mm (≈1 row)** |

- Reachability: `slip39LengthPick` returns `{20,33,23,27,30}` (`gui/slip39_polish.go:54-55`). `slip39Engrave` is a primary button (`:126`) → `engraveSLIP39Verbatim` (`:379,:426`) → `backup.Seed{Mnemonic: scan.Mnemonic}` (`:488-489`, all 33 words, no truncation) → `EngraveSeed` (`backup.go:62`) → **`frontSideSeed`** (the mnemonic-word path, NOT `engraveSeedString`).
- **`engraveSLIP39Verbatim` already surfaces a clean error** for the genuinely-too-large case via `showError(ctx, th, "Too large", "Share doesn't fit a plate.")` on `EngraveSeed`/`toPlate` returning non-nil (`gui/slip39_polish.go:497-504`). With the rework, 33 words now FIT, so this error is no longer expected for any reachable SLIP-39 count — but the path remains as a backstop for any future overflow.
- Test gap: `backup/backup_test.go` exercises only 12- and 24-word BIP-39 seeds (`TestSeed`, :180-200) and one 20-word SLIP-39 (`TestSLIP39`, :202-225); >24 never tested.

### Structural cause + why A2 (3rd column) is infeasible — chosen approach A1
**Structural cause:** for N=33 col-2 must hold `N-endCol1 = 17` words, but the band between the col-2 TOP anchor (9.70mm) and the col-1 bottom edge (75.30mm) is exactly `col1Height = 16*pfs ≈ 65.6mm = 16 rows`. With `endCol1` fixed at 16, the col-2 band is **exactly 16 rows tall at any uniform font**, so the 17th row always overflows — shrinking the font alone does NOT help while `endCol1` stays 16 (the band shrinks with the font). The fix must **rebalance the split** so col-2 never holds more rows than col-1.

**Option A2 (3rd column) — REJECTED by geometry.** One column block is **32.80mm** wide at the full font. Three columns need three x-anchors; the most compact feasible 3-anchor scheme places the rightmost column so its right edge lands at **90.8–94.8mm**, overrunning the 85mm plate (measured: anchors {8,33,58}mm → 90.8mm right edge). A 3rd column would *itself* require a smaller font just to fit the plate width — strictly worse than A1, and a larger blast radius (a new column loop + x-anchor regime). **A2 does not fit at a legible font; A1 does.**

**Option A1 (adaptive font + rebalanced split) — CHOSEN.** For large N, (1) rebalance into two columns of `col1Rows = ceil(N/2)`, `col2Rows = floor(N/2)` (so `col2Rows ≤ col1Rows` always — col-2 is one contiguous block, no top/bottom split, which removes the overlap mechanism entirely), and (2) pick a font `pfsN = min(F(4.1), legacyCol1Height / col1Rows)` where `legacyCol1Height = 16*F(4.1)`, i.e. shrink the font only enough to keep `col1Rows` rows within the **same vertical envelope** the legacy 16-row column already occupies. This (a) preserves the outer mfp/title/centering geometry extent, (b) only shrinks the font when `col1Rows > 16` (i.e. only N=33; for N=25..30 `pfsN` stays at the full F(4.1)), and (c) keeps col-2 a single block, eliminating the legacy two-block collision.

### Chosen geometry — exact numbers (proves NO overlap, within bounds)
Layout selected by a single predicate `N > 24` (only the over-24 SLIP-39 counts take the new path; everything ≤24 is byte-identical legacy). `col1Rows = ceil(N/2)`, `col2Rows = floor(N/2)`, `pfsN = min(F(4.1), 16*F(4.1) / col1Rows)`, `col1H = pfsN*col1Rows`, top anchor `(plateY-col1H)/2`, col-1 = rows `0..col1Rows`, col-2 = rows `col1Rows..N` as **one** contiguous block from the same top anchor. Integer (production-scale) y-ranges:

| N  | col1Rows / col2Rows | pfsN (mm) | col1H (mm) | col-1 y (mm)     | col-2 y (mm)     | no-overlap & in-band |
|----|---------------------|-----------|------------|------------------|------------------|----------------------|
| 25 | 13 / 12             | 4.100     | 53.30      | [15.85, 69.15]   | [15.85, 65.05]   | ✓                    |
| 27 | 14 / 13             | 4.100     | 57.40      | [13.80, 71.20]   | [13.80, 67.10]   | ✓                    |
| 30 | 15 / 15             | 4.100     | 61.50      | [11.75, 73.25]   | [11.75, 73.25]   | ✓                    |
| **33** | **17 / 16**     | **3.859** | **65.60**  | **[9.70, 75.30]** | **[9.70, 71.44]** | **✓**               |

For **N=33** (the only count that shrinks the font): `pfsN = 16*26240 / 17 = 24696` units = **3.859mm**; both columns share top anchor 9.70mm; col-1 bottom = 75.30mm (= legacy col-1 bottom, so the title/mfp envelope is unchanged in extent), col-2 bottom = 71.44mm < col-1 bottom → **3.86mm gap, no overlap**; both within `[9.70, 75.30] ⊂ [0, 85]mm`. Width at pfsN: one block ≈ 30.9mm → col-2 right edge ≈ 74.9mm < 85mm (fits). **Legibility/engraveability:** stroke is fixed at 0.3mm; the 33-word font is 3.859mm so stroke/em = 0.078 (vs 0.073 today) — the glyph is ~5% denser-stroked but still 12.9× the stroke width; well above the 0.3mm engraveable floor. (Residual: confirm 3.86mm legibility is acceptable to the user / on hardware — see Risks.)

### Fix (summary)
Parameterize `frontSideSeed` so that for `N > 24` it uses the A1 large-N layout above: rebalanced `ceil/floor` two-column split, adaptive `pfsN = min(F(4.1), 16*F(4.1)/col1Rows)`, single contiguous col-2 block. For `N ≤ 24` the existing code path runs **unchanged** (same constants, same two-block col-2, same `pfs=F(4.1)`), so BIP-39 {12,18,20,23,24} and every QR-bearing layout are coordinate-identical. No GUI change. The legacy `gui.toPlate` "Too large" backstop remains for any future overflow but is unreachable for the now-supported SLIP-39 counts.

### Invariant
For **every** supported word count N up to 33: (1) **no row/block overlap** — every engraved text block occupies a disjoint y-range (col-2's bottom edge ≤ col-1's bottom edge, both ≥ their shared top anchor); (2) **within plate bounds** — all rows lie in `[0, F(85)]` on both axes with the centering preserved; (3) **legible/engraveable** — `pfsN ≥ 3.859mm` (the N=33 minimum) and stroke stays the fixed 0.3mm, so every glyph is ≥ ~12.9× stroke width. AND the **no-regression invariant (load-bearing):** for all N ∈ {12,18,20,23,24} (BIP-39 + the smaller SLIP-39 shares) **and every QR-bearing layout**, the produced plan is **byte/coordinate-identical** to current `f907eea` — the `N>24` branch is the *only* behavioral change, and it is unreachable for N≤24 and for any QR plate (QR plates are BIP-39, ≤24 words).

### Acceptance (TDD — MUST fail on current code, pass after fix)
In `backup/backup_test.go` (currently 12/24-word BIP-39 + one 20-word SLIP-39):
1. **N=33 no-overlap (the load-bearing new test):** build a 33-word SLIP-39 `Seed`, call `EngraveSeed`, and compute the col-1 and col-2 row y-ranges from the plan (or assert against the table above). Assert col-2's bottom edge ≤ col-1's bottom edge (non-overlap) AND all rows ⊂ `[0, F(85)]`. **On current `f907eea` this FAILS** (the +4.10mm overlap — col-2 bottom block top at 22.00mm rises above col-2 top block bottom at 26.10mm); after the rework it PASSES (3.86mm gap). Also assert `pfsN == 24696` (the 3.859mm adaptive scale) so the chosen geometry is pinned, not just "some non-overlapping layout."
2. **Regression pin — N∈{20,23,24,30} + a QR plate:** golden/coordinate snapshots that the {20,23,24}-word and a QR-bearing (BIP-39, `genSeed` with QR) plate are **byte-identical** to current output (extend the existing golden mechanism, `compareGolden`, which already pins `TestSeed` 12/24 and `TestSLIP39` 20). For **N=30** assert no-overlap + within-bounds (it newly takes the `N>24` path at full font 4.1mm — pin its y-ranges to the table). These guard the #1 risk (regressing the shared ≤24+QR path) and confirm N=30 didn't change font.
3. **Existing tests stay green:** `TestSeed` (12,24), `TestSLIP39` (20), `TestConstantSeedTiming`, `TestCodex32`, `TestText` all unchanged — no golden churn for any ≤24 case (proves the `N>24` predicate is the sole behavioral edit).

---

## Scope, caller fan-out, and surface

- **Files touched:** `engrave/engrave.go` (BUG-1 guard line 1467; BUG-2 `bitmapForQRStatic`/`ConstantQR` defense-in-depth), `backup/backup.go` (BUG-2 guard in `EngraveSeedString`; BUG-3 **large-N layout rework in `frontSideSeed`** — adaptive font + rebalanced split for `N>24`, ≤24 path untouched). Tests: `engrave/engrave_test.go`, `backup/backup_test.go` (BUG-3: new 33-word no-overlap test + N∈{20,23,24,30}+QR regression pins).
- **Caller fan-out (verified, so fixes don't break other callers):**
  - `ConstantQR`: callers `EngraveSeed` (`backup.go:66`) and `EngraveSeedString` (`backup.go:81`) — both already handle `err`. Safe to keep `ConstantQR`'s signature; only add an early size-check returning the existing `error`.
  - `bitmapForQRStatic` (if converting to error-return): 3 sites — `engrave.go:410` (`ConstantQR`), `engrave.go:616` (`ConstantQRCmd.Engrave` — the constraining site; see BUG-2 scope decision), `engrave_test.go:90`.
  - `frontSideSeed`: sole caller `EngraveSeed` (`backup.go:71`); `EngraveSeed` callers: `gui/gui.go:488`, `gui/slip39_polish.go:496`, plus tests. The BIP-39+QR callers cap at ≤24 words → the new `N>24` large-N branch never fires for them (they keep the legacy layout byte-for-byte).
  - `EngraveSeedString`: sole non-test caller `gui/gui.go:2078` (`backupSeedStringFlow`), already `if err != nil { return }` — the new error routes cleanly.
  - `SafePointer.Progress`: driven only from `gui/engraver.go:158-160`; the symmetric-break change is internal to `SafePointer`, no signature change.
- **NO `me` / CLI / schema / docs surface.** Firmware-only. **NO m\*-codec edit needed** — these are SeedHammer engraving-subsystem bugs, independent of the `m`-format codecs. Confirmed: none of the three touch the constellation `md1/mk1/ms1` string encoders/decoders; BUG-2 only consumes an already-formed codex32 string at engrave time.
- **License/headers:** existing files; MIT OR Unlicense unchanged.

---

## Risks

1. **BUG-1 regressing resume safety (TOP RISK).** The fix must keep **correct** safe-points, not merely stop the wrap. A saturating-subtract would stop the wrap while still mis-retiring leading move knots → wrong safe point → the exact head-crash this bug causes. Mitigation: break-based fix (waits for true elapse) + acceptance test asserting the safe-point **reference**, not just the counter bound. The existing `TestSafePointer` MUST still pass (catches over-correction).
2. **BUG-2 return-type-change fan-out.** Converting `bitmapForQRStatic` to return an error touches `ConstantQRCmd.Engrave` (`engrave.go:616`), whose signature may not accommodate it. Mitigation: the **primary** guard lives in `EngraveSeedString` (and an early check in `ConstantQR`) and fully fixes the confirmed bug; the deeper conversion is optional/scoped at R0. Do not destabilize `Engrave`'s signature.
3. **BUG-3 regressing the shared ≤24-word + QR layout (the #1 risk).** The chosen layout rework (option a) edits a load-bearing, hard-to-test geometry routine shared with the BIP-39 + QR path. A coding slip in the branch predicate or in the shared prologue could shift the ≤24/QR coordinates. Mitigation: the entire change is gated behind a single `N>24` predicate; ≤24 and all QR plates run the *unchanged* code path; byte-identical golden/coordinate regression pins for N∈{20,23,24} + a QR plate are mandatory acceptance gates; the existing `TestSeed`/`TestSLIP39`/`TestCodex32`/`TestText` goldens must show zero churn.
   - **Sub-risk — legibility/engraveability of the shrunk font (A1).** Only N=33 shrinks the font (to 3.859mm); the stroke is fixed at 0.3mm so engraveability is unchanged (glyph ≥ ~12.9× stroke), but the 3.86mm em is ~6% smaller than the 4.1mm baseline. Residual: whether 3.86mm reads cleanly on the physical plate is a legibility judgement that may warrant hardware/visual confirmation before ship (the geometry and engraveability are proven; legibility is subjective). Flag for R0 — no hardware blocker expected since SeedHammer already engraves a 24-word plate plus a QR at finer pitch in the same area.
   - **Sub-risk — A2 rejected.** A 3rd column was analyzed and rejected: one column block is 32.8mm wide, so 3 columns overrun the 85mm plate (right edge 90.8mm) at any legible font. No plate-width risk remains because A2 is not used.
4. **General:** these are inherited bugs; keep fixes minimal and faithful so they remain offer-able upstream later (fork stays clean; upstream PR is out of scope here).

---

## Gate

- **R0 (mandatory):** opus architect review of THIS spec → fold → persist verbatim to `design/agent-reports/` → re-dispatch after every fold → converge to **0 Critical / 0 Important** before any code.
- **Open items for R0 to rule on:**
  1. BUG-2: convert `bitmapForQRStatic`→error (threading `:616`) vs. early-check-in-`ConstantQR` only. (Recommend: early check in `ConstantQR` + mandatory `EngraveSeedString` guard; leave `panic` as unreachable assertion.)
  2. BUG-3: **USER DECISION TAKEN (2026-06-20) = option (a), layout rework.** R0 to ratify the chosen A1 geometry (rebalanced `ceil/floor` split + adaptive `pfsN = min(F(4.1), 16*F(4.1)/col1Rows)`, single contiguous col-2 block, gated on `N>24`); confirm the no-regression pins (N∈{20,23,24}+QR byte-identical) are sufficient; and rule on whether the N=33 3.859mm font needs hardware/visual legibility confirmation before ship (geometry + engraveability proven; legibility is the only residual).
  3. BUG-1: ratify the break-based symmetric guard (`if s.progress < k.T { break }`) and the exact form of the safe-point **reference** assertion in the new test.
- **Per-bug acceptance gates (all three):** a TDD test that FAILS on current `f907eea` then PASSES after the fix — especially BUG-1's wrap-catching counter-invariant assertion and BUG-3's N=33 no-overlap + pinned-`pfsN` layout test (plus its byte-identical ≤24+QR regression pins).
- **Implementation:** single subagent, TDD, in a worktree (no parallel re-implementations). Stage paths explicitly.
- **Post-implementation:** mandatory independent adversarial execution review over the whole diff before ship.
