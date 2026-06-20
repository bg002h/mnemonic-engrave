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

### Verified facts (file:line)
- `backup/backup.go:161-225` — `frontSideSeed`. Layout constants `maxCol1=16, maxCol2=4` (172-173).
- Col-2 TOP: words `endCol1..endCol2` = `16..min(20,N)`, anchored at `(plateY-col1Height)/2` (195-197).
- Col-2 BOTTOM: words `endCol2..N`, height `(N-endCol2)*pfs`, anchored at `(plateY+col1Height)/2 - height` (208-213).
- No check that the bottom block fits below the top block. `gui.toPlate` (`gui/gui.go:~2817-2830`) rejects only off-plate bounds (`ErrTooLarge`); **on-plate overlap passes validation** and is engraved.
- **Reproduced layout math at production scale (mm=6400, pfs=F(4.1), plateY=F(85)) — matches verifier exactly:**

  | N  | col2-TOP y (mm) | col2-BOTTOM y (mm) | overlap |
  |----|-----------------|--------------------|---------|
  | 20 | [9.70, 26.10]   | (none)             | gap     |
  | 23 | [9.70, 26.10]   | [63.00, 75.30]     | gap     |
  | 24 | [9.70, 26.10]   | [58.90, 75.30]     | gap     |
  | 27 | [9.70, 26.10]   | [46.60, 75.30]     | gap     |
  | 30 | [9.70, 26.10]   | [34.30, 75.30]     | gap     |
  | **33** | [9.70, 26.10] | **[22.00, 75.30]** | **+4.10mm (≈1 row)** |

- Reachability: `slip39LengthPick` returns `{20,23,27,30,33}` (`gui/slip39_polish.go:54-55`). `slip39Engrave` is a primary button (`:126`) → `engraveSLIP39Verbatim` (`:379,:426`) → `backup.Seed{Mnemonic: scan.Mnemonic}` (`:488-489`, all 33 words, no truncation) → `EngraveSeed` (`backup.go:62`) → **`frontSideSeed`** (the mnemonic-word path, NOT `engraveSeedString`).
- **`engraveSLIP39Verbatim` already surfaces a clean error** for the too-large case via `showError(ctx, th, "Too large", "Share doesn't fit a plate.")` on `EngraveSeed`/`toPlate` returning non-nil (`gui/slip39_polish.go:497-504`). So a guard-error (option b) surfaces cleanly with **no GUI change**.
- Test gap: `backup/backup_test.go` exercises only 12- and 24-word seeds (≤24); >24 never tested.

### Layout-vs-guard assessment + RECOMMENDATION
**Structural cause:** col-2 must hold `N-16` words (= **17** for N=33) but the vertical span between the TOP anchor (9.70mm) and the BOTTOM anchor (75.30mm) is `col1Height ≈ 65.6mm = exactly 16 rows` at pfs=4.1. **17 rows cannot fit in col-2 at full font** — option (a) therefore requires a genuine layout rework: either a **3rd column** (e.g. 16/12/5) or a **smaller font when N is large**. Horizontal room exists on the SLIP-39 verbatim path (it sets no QR, so `plate.QR==nil` and the center-right QR slot at x=60mm is free), but:
  - A 3rd column changes a load-bearing, hard-to-test geometry routine (`frontSideSeed`) that is **shared with the BIP-39 + QR path** (`EngraveSeed` from `gui/gui.go:488`, `gui/slip39_polish.go:496`). Any rework must NOT alter the ≤24-word + QR layouts (those are correct today) — a real regression surface.
  - A smaller-font-for-large-N path adds a second `pfs` regime and re-derives every offset; also touches the shared routine.
  - Either is an R0-class layout-correctness change with its own measurement/test burden, on inherited code we don't own.

**RECOMMENDATION: option (b) — GUARD with a clean error — for THIS cycle.** Add an overlap check in `frontSideSeed` (or `EngraveSeed`): if the col-2 bottom block's top edge would rise above the col-2 top block's bottom edge — i.e. `(plateY+col1Height)/2 - (N-endCol2)*pfs < (plateY-col1Height)/2 + (endCol2-endCol1)*pfs` — return `error` ("seed too long for plate"). This is small, safe, regression-free (the GUI already shows "Too large"), and fixes the hardware-safety/correctness defect immediately. It is correctly inert for N∈{20,23,24,27,30} (gaps) and fires only for N=33 (the single overlapping count), but SHOULD be written as a **general geometric overlap test**, not a hardcoded `N==33`, so it also covers any future longer input.

**Flagged trade-off (possible USER decision at R0):** option (b) means a **33-word (256-bit) SLIP-39 share — the highest-security SLIP-39 form — cannot be engraved verbatim at all** (it errors). That is a real capability gap. If the user deems 33-word verbatim engraving must-ship, option (a) (3rd-column rework) becomes in-scope and this SPEC must be re-scoped with a dedicated layout-correctness gate. **Default recommendation absent user input: ship (b) now, file option (a) as a FOLLOWUP** (`design/FOLLOWUPS.md`) for a later layout cycle.

### Fix (summary)
Geometric overlap guard in the seed-word layout path returning a clean error when the two col-2 blocks would collide. No GUI change (existing `showError` path). Recommend (b); (a) deferred unless user elects otherwise.

### Invariant
For every reachable word count, the engraved plate has **no overlapping text blocks**: either the layout places all blocks with non-negative inter-block gap, OR `frontSideSeed`/`EngraveSeed` returns an error (never silently engraves overlapping geometry). Counts {20,23,24,27,30} continue to produce the identical (gap-positive) layout they do today.

### Acceptance (TDD — MUST fail on current code, pass after fix)
In `backup/backup_test.go` (currently only 12/24-word):
1. **Overlap→error (option b):** `EngraveSeed` with a 33-word SLIP-39 mnemonic returns a non-nil error (and does not produce an overlapping plan). On current code it returns a valid (overlapping) `Engraving` with no error → test fails; after the guard it errors.
2. **Non-regression:** `EngraveSeed` with 20/23/24/27/30-word mnemonics still returns a valid `Engraving`, no error; assert (at least for one count) that col-2 block geometry is unchanged vs current (e.g. snapshot the y-anchors or assert the gap is positive).
   - *(If option (a) is chosen instead:* assert the 33-word plan has no overlapping block bounds AND ≤24-word+QR layouts are byte-for-byte unchanged.)*

---

## Scope, caller fan-out, and surface

- **Files touched:** `engrave/engrave.go` (BUG-1 guard line 1467; BUG-2 `bitmapForQRStatic`/`ConstantQR` defense-in-depth), `backup/backup.go` (BUG-2 guard in `EngraveSeedString`; BUG-3 overlap guard in `frontSideSeed`/`EngraveSeed`). Tests: `engrave/engrave_test.go`, `backup/backup_test.go`.
- **Caller fan-out (verified, so fixes don't break other callers):**
  - `ConstantQR`: callers `EngraveSeed` (`backup.go:66`) and `EngraveSeedString` (`backup.go:81`) — both already handle `err`. Safe to keep `ConstantQR`'s signature; only add an early size-check returning the existing `error`.
  - `bitmapForQRStatic` (if converting to error-return): 3 sites — `engrave.go:410` (`ConstantQR`), `engrave.go:616` (`ConstantQRCmd.Engrave` — the constraining site; see BUG-2 scope decision), `engrave_test.go:90`.
  - `frontSideSeed`: sole caller `EngraveSeed` (`backup.go:71`); `EngraveSeed` callers: `gui/gui.go:488`, `gui/slip39_polish.go:496`, plus tests. The BIP-39+QR callers cap at ≤24 words → guard never fires for them.
  - `EngraveSeedString`: sole non-test caller `gui/gui.go:2078` (`backupSeedStringFlow`), already `if err != nil { return }` — the new error routes cleanly.
  - `SafePointer.Progress`: driven only from `gui/engraver.go:158-160`; the symmetric-break change is internal to `SafePointer`, no signature change.
- **NO `me` / CLI / schema / docs surface.** Firmware-only. **NO m\*-codec edit needed** — these are SeedHammer engraving-subsystem bugs, independent of the `m`-format codecs. Confirmed: none of the three touch the constellation `md1/mk1/ms1` string encoders/decoders; BUG-2 only consumes an already-formed codex32 string at engrave time.
- **License/headers:** existing files; MIT OR Unlicense unchanged.

---

## Risks

1. **BUG-1 regressing resume safety (TOP RISK).** The fix must keep **correct** safe-points, not merely stop the wrap. A saturating-subtract would stop the wrap while still mis-retiring leading move knots → wrong safe point → the exact head-crash this bug causes. Mitigation: break-based fix (waits for true elapse) + acceptance test asserting the safe-point **reference**, not just the counter bound. The existing `TestSafePointer` MUST still pass (catches over-correction).
2. **BUG-2 return-type-change fan-out.** Converting `bitmapForQRStatic` to return an error touches `ConstantQRCmd.Engrave` (`engrave.go:616`), whose signature may not accommodate it. Mitigation: the **primary** guard lives in `EngraveSeedString` (and an early check in `ConstantQR`) and fully fixes the confirmed bug; the deeper conversion is optional/scoped at R0. Do not destabilize `Engrave`'s signature.
3. **BUG-3 layout-vs-guard scope.** Option (a) is a real rework of a shared, hard-to-test geometry routine with its own regression surface (≤24-word + QR layouts must stay identical); option (b) is small/safe but removes 33-word verbatim capability. Mitigation: recommend (b) for this cycle + FOLLOWUP for (a); surface the capability trade-off as a possible user decision at R0.
4. **General:** these are inherited bugs; keep fixes minimal and faithful so they remain offer-able upstream later (fork stays clean; upstream PR is out of scope here).

---

## Gate

- **R0 (mandatory):** opus architect review of THIS spec → fold → persist verbatim to `design/agent-reports/` → re-dispatch after every fold → converge to **0 Critical / 0 Important** before any code.
- **Open items for R0 to rule on:**
  1. BUG-2: convert `bitmapForQRStatic`→error (threading `:616`) vs. early-check-in-`ConstantQR` only. (Recommend: early check in `ConstantQR` + mandatory `EngraveSeedString` guard; leave `panic` as unreachable assertion.)
  2. BUG-3: ratify option (b) (guard + FOLLOWUP for (a)) vs. user electing option (a) (3rd-column rework) this cycle. **Possible user decision.**
  3. BUG-1: ratify the break-based symmetric guard (`if s.progress < k.T { break }`) and the exact form of the safe-point **reference** assertion in the new test.
- **Per-bug acceptance gates (all three):** a TDD test that FAILS on current `f907eea` then PASSES after the fix — especially BUG-1's wrap-catching counter-invariant assertion and BUG-3's overlap-detection.
- **Implementation:** single subagent, TDD, in a worktree (no parallel re-implementations). Stage paths explicitly.
- **Post-implementation:** mandatory independent adversarial execution review over the whole diff before ship.
