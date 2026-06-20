# Engrave-bugfixes — whole-diff execution review (round 0) — VERBATIM agent report

**Agent:** `a9a26f2f2abf9142f` (adversarial opus architect; mandatory post-implementation whole-diff exec review; ran both negative controls + the no-false-rejection BIP-39 SeedQR-dim check in a throwaway worktree off the branch tip). **Branch:** `feat/engrave-bugfixes`. **Base:** `f907eea` (fork main). **Tip:** `f70b92d`. **Date:** 2026-06-19.
**Verdict:** GREEN (0 Critical / 0 Important / 0 Minor). MERGE recommended. Mandatory per CLAUDE.md phase (4).

---

# R0 WHOLE-DIFF EXECUTION REVIEW — SeedHammer II engrave bugfixes (VERBATIM)

**Reviewer:** adversarial opus architect (mandatory post-implementation whole-diff exec review)
**Branch:** `feat/engrave-bugfixes` · **Base:** `f907eea` (fork main) · **Tip:** `f70b92d`
**Commits:** `3056118` (BUG-1), `e10b27a` (BUG-2), `f70b92d` (BUG-3) · **Go:** go1.26.4 · module declares go1.25.10
**Verdict: GREEN — 0 Critical / 0 Important / 0 Minor.** MERGE recommended.

Every claim in the plan was re-verified against the *actual committed diff* (not the plan's intent). All tests pass, both negative controls fire, the highest-value no-false-rejection check passes, and the geometry was independently re-derived to the unit. One apparent "overlap failure" in my own probe turned out to be a probe artifact (the centered title), proven harmless by isolating it.

---

## MANDATE 1 — Diff fidelity vs the plan

The full `f907eea..f70b92d` diff touches **exactly** the 6 planned files and nothing else:
`engrave/engrave.go`, `backup/backup.go`, `engrave/engrave_test.go`, `backup/backup_test.go`, `backup/testdata/slip39-23-words.bin`, `backup/testdata/slip39-33-words.bin`. No stray files, no unplanned hunks, no leftover debug/scratch code. `git diff --name-only` and `--stat` (284 insertions, 23 deletions) confirm.

- **Source diff = plan's Old→New blocks exactly** (`engrave.go` BUG-1 guard + BUG-2 `ConstantQR` early check; `backup.go` `errors` import + `EngraveSeedString` guard + `frontSideSeed` rework). No edits beyond the plan.
- **No signature changes.** At tip: `bitmapForQRStatic(dim int) ([]bezier.Point, []bezier.Point)`, `ConstantQR(qrc *qr.Code) (*ConstantQRCmd, error)`, `SafePointer.Progress(p uint)` — all unchanged. `ConstantQRCmd.Engrave` does not appear in the diff (untouched).
- **`bitmapForQRStatic` panic intact:** `panic("unsupported qr code version")` still present at `engrave.go:399` (left as an unreachable assertion, NOT converted to error-return), exactly as R0-ratified.

## MANDATE 2 — BUG-1 correctness (the CRITICAL fix)

- **Committed guard is the exact symmetric break** at `engrave.go` tip: `if s.progress < k.T { break }` (dropping `k.Engrave &&`) — NOT a saturating subtract. Confirmed from `git show f70b92d:engrave/engrave.go`.
- **Tests pass:** `TestSafePointer` (existing, catches over-correction) PASS; `TestSafePointerNoUnderflow` PASS.
- **Negative control (load-bearing proof):** reverting ONLY the guard back to `if k.Engrave && s.progress < k.T` made `TestSafePointerNoUnderflow` FAIL — `engrave_test.go:410: step 0: sp.progress=18446744073709551316 exceeds totalTicks=300 (underflow wrap)` (= 2^64−300) — while `TestSafePointer` still PASSED (confirming the existing test tolerates the wrap; the new test is genuinely load-bearing on the committed code). Restored, re-passes.
- **Correctness beyond the test:** the loop retires a knot only when `s.progress >= k.T`, i.e. once cumulative reported ticks cover the knot's full duration — the correct definition of "fully elapsed." The symmetric break never *wrongly* withholds a legitimately-elapsed knot: a move knot is simply retired on the next `Progress` call once its `k.T` is covered, and the safe-point selection (clamped-triple `k0.Ctrl==k1.Ctrl && k1==k2`) then reads valid state instead of post-wrap garbage. The original eager move-knot retirement was correct only under the false assumption `s.progress >= k.T` always held for them. This yields correct *eventual* retirement (the spec invariant), not merely a non-wrapping counter — corroborated by `TestSafePointer` selecting identical safe points (no over-correction).

## MANDATE 3 — BUG-2 correctness + no false rejection

- **Both guards present & correct:** `qrc.Size > 33` in `EngraveSeedString` (clean `errors.New(...)` onto the existing err path) AND the early `dim > 33` check in `ConstantQR` before `bitmapForQRStatic`.
- **Tests pass:** `TestEngraveSeedStringTooLong/dim37` + `/dim41` PASS, `TestEngraveSeedStringHappy` (dim33) PASS, `TestConstantQR` PASS (not regressed).
- **Critical no-false-rejection check — PASSED (highest value).** I independently built valid 12- and 24-word BIP-39 mnemonics, encoded the real SeedQR (`seedqr.QR` → 4-digit-per-word, under `qr.M`, exactly as `gui/gui.go:472`), and measured: **12-word → QR dim 25 (V2); 24-word → QR dim 29 (V3).** Both ≤ 33, so the `ConstantQR` guard is provably **inert** for every legitimate BIP-39 plate, and `ConstantQR` accepts both without error. A 24-word SeedQR (96 numeric chars) cannot exceed dim 33 — the core feature is not broken. The `>33` cutoff is exact (dim 33 = V4 supported; dim 37 = V5 rejected).

## MANDATE 4 — BUG-3 correctness + no-regression (the broadest change)

- **≤24 `else` branch is verbatim-order:** col1 → col2-top → QR → col2-bottom → title with the original anchors; the only edits are `len(plate.Mnemonic)`→`n` renames + a block-scope change that emits identical commands. The `n > largeN` (24) predicate is the sole gate.
- **Zero churn on committed ≤24/QR goldens:** `TestSeed`, `TestSLIP39`, `TestCodex32`, `TestText`, `TestConstantSeedTiming`, `TestConstantStringTiming` all PASS; `git status backup/testdata/` shows no modified `.bin`. `TestSLIP39Large`, `TestSLIP39LargeGeometry`, `TestSLIP39_23WordPin`, `TestSLIP39_30WordInBounds` all PASS.
- **Independent golden-correctness verification (not just self-consistency):** I re-derived N=33 anchors at production scale: pfsN = 16·26240/17 = **24696 (3.859mm)**, col1Bot = **75.30mm**, col2Bot = **71.44mm**, gap = **3.86mm** — matching the plan/spec to the unit. I then measured the *emitted* N=33 and N=30 plans (`slices.Collect(engrave.PlanEngraving(conf, side))`), banding by `Ctrl.X`. **With the title isolated** (set Title="", MFP=0), N=33: col1 y[7.22, 74.87], col2 y[10.56, 71.01] → col2Bot ≤ col1Bot, **no overlap, off-plate=false**; N=30: col1 y[8.63, 72.79], col2 y[12.66, 72.79] → equal bottoms (15/15, 0.00mm gap), in-bounds.
  - **Investigated artifact (no defect):** my *first* band probe flagged "col2 81.97mm > col1 74.87mm." Root cause: the centered title "7945 #1 1/1" is engraved BELOW the columns (offy = (plateY+col1Height)/2 + F(4) = **79.30mm**, computed independently and confirmed ≤ 85mm) and spans the full plate width, leaking into both x-bands. An x-histogram of y>72mm knots showed a continuous x=10..55mm sweep — the title, exactly as in the legacy ≤24 layout. Removing the title cleared it. This is expected layout, not a regression; the plan-R0 reviewer saw the same 81.97mm bbox bottom and correctly attributed it.
- **N=27/N=30 layout change vs f907eea is the deliberate R0-ratified rebalance** (the only >24 GUI counts besides 33 are {27,30}; only N≤24 and QR plates were required byte-identical). The BUG-3 commit message documents this explicitly.
- **Negative control:** reverting `frontSideSeed` to f907eea made `TestSLIP39Large` FAIL — `backup_test.go:494: spline lengths 16753, 16277, with 16276/16277 knot mismatches` (exactly the predicted count). Restored, re-passes. The golden is a genuine production gate.

## MANDATE 5 — Build / vet / scope / commit hygiene

- `go test ./engrave/... ./backup/...` → both `ok`. `go build ./...` → clean (exit 0). `go vet ./engrave/ ./backup/` → ONLY the three pre-existing `testing.ArtifactDir requires go1.26 ... (file is go1.25)` notices (engrave_test.go:167,187; backup_test.go:389 — shifted from 388 by the new `slices` import). The diff adds **0** `ArtifactDir` calls; `go vet ./slip39/` (non-touched control) → exit 0. No new vet finding attributable to this diff.
- **Scope:** firmware-only. No me/CLI/schema/docs surface; no md/mk/codex32/ms1 constellation-codec edits (BUG-2 only consumes an already-formed codex32 string). Confirmed by name-only diff grep.
- **Commit hygiene:** all 3 commits are SSH-signed (ed25519 `gpgsig` header present — local verify only errored due to unconfigured `allowedSignersFile`, not a missing signature), carry DCO `Signed-off-by: Brian Goss`, the `Co-Authored-By: Claude Opus 4.8 (1M context)` trailer, and author `Brian Goss <goss.brian@gmail.com>`. Per-commit file sets are clean: BUG-1 = {engrave.go, engrave_test.go}; BUG-2 = {backup.go, backup_test.go, engrave.go}; BUG-3 = {backup.go, backup_test.go, 2 goldens}.
- **Self-containment spot-check:** at `3056118` (BUG-1) `go build ./...` clean + both packages `ok`; at `e10b27a` (BUG-2) both packages `ok`. History is bisectable.
- **Minor-2 strict-order concern resolved:** the BUG-2 commit's `backup_test.go` hunk contains ONLY the BUG-2 tests (`engraveStringRecovered`, `TestEngraveSeedStringTooLong`, `TestEngraveSeedStringHappy`) — no BUG-3 test code leaked in. The `slices` import landed correctly in the BUG-3 commit.

---

## Critical / Important / Minor

**Critical: none. Important: none. Minor: none.**

(The plan's optional Minor-1 — adding emitted-plan banding to `TestSLIP39LargeGeometry` — was implementer's discretion to skip; it is adequately covered by `TestSLIP39_30WordInBounds`'s `PlanEngraving` walk plus the byte-exact golden gate, so it is not a gap.)

## Verified-correct

- Exactly 6 files changed; no scope creep; no signature changes; `bitmapForQRStatic` panic intact.
- BUG-1: committed guard is the symmetric break; negative control fires (wrap = 2^64−300); `TestSafePointer` still green (no over-correction); correct eventual retirement reasoned independently.
- BUG-2: both guards present; `>33` cutoff exact; **12-word SeedQR = dim 25, 24-word = dim 29 (both ≤33)** → guard provably inert for legit BIP-39 plates; happy/too-long/ConstantQR tests all pass.
- BUG-3: N=33 geometry re-derived to the unit (pfsN=24696, 75.30/71.44mm, gap 3.86mm); emitted columns non-overlapping & in-bounds (N=33 and N=30) with the title artifact isolated; title at 79.30mm ≤ 85mm; negative control = 16276/16277 mismatches; zero churn on all ≤24/QR goldens; N=27/30 rebalance is deliberate.
- Build clean, vet shows only pre-existing notices, scope firmware-only, all 3 commits signed + DCO + co-author + correct author, each self-contained, BUG-2 commit free of BUG-3 test code.

## Bottom line

The committed diff faithfully and minimally implements the GREEN plan with no implementation-introduced defects. All three fixes are correct (not merely safe), both negative controls confirm the tests are load-bearing on the *committed* code, the highest-risk feature-break check (legit BIP-39 QR rejection) is definitively cleared, and the broad BUG-3 change preserves every ≤24/QR plate byte-identically while the N=27/30/33 changes are the ratified rebalance. The one anomaly I found was an artifact of my own probe, traced and dismissed.

**Recommendation: MERGE `feat/engrave-bugfixes` (f70b92d) to fork main.**

**Fork state at end:** `/scratch/code/shibboleth/seedhammer` on `main @ f907eea`, `git status --porcelain` empty (clean). Branch `feat/engrave-bugfixes` intact at `f70b92d`. Throwaway worktree `/tmp/r0-exec-engrave` removed + pruned; all `/tmp` probe files deleted. `git worktree list` shows only fork main, the pre-existing `seedhammer-wt-bip39` (untouched), and the implementer's `/tmp/engrave-bugfixes`. No leftover artifacts.
