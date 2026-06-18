# Cycle B: codex32-multishare-recovery — IMPLEMENTATION PLAN R0 GATE REVIEW — R1

- **Stage:** plan gate round R1 (re-review after folding R0's 1 Critical + 1 Important).
- **Plan reviewed:** `design/IMPLEMENTATION_PLAN_seedhammer_codex32_multishare_recovery.md` (committed `b47fa6f`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `adde0373f90d76e5d`), read-only vs fork `main` `bf7f811`.
- **Outcome:** **GREEN — 0 Critical / 0 Important.** Plan cleared for implementation.

> NOTE: verbatim architect output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

## R1 REVIEW — Cycle B Implementation Plan Gate

**Reviewer:** Opus architect, R1 post-fold.
**Plan:** `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_seedhammer_codex32_multishare_recovery.md`
**Base:** fork `main` `bf7f811`. R0 findings: 1 Critical (C1), 1 Important (I1). Folded prior to this review.

---

### C1 VERIFICATION — `confirmCodex32Flow` Button2 drain

**Finding:** FOLD CORRECT AND COMPLETE.

The EventRouter (`gui/event.go:266-279`) implements `Next` as a strict head-of-queue check: it examines only `r.events[0]`, and if the head event does not match the caller's filters, it returns `(Event{}, false)` without consuming or advancing the queue. This is the blocking mechanism R0 identified.

The fold in Task 3b Step 7 reads:
```go
recoverClicked := recoverBtn.Clicked(ctx)
if !f.Unshared && recoverClicked {
    return codex32Recover
}
```

Mechanical trace of `TestConfirmCodex32UnsharedNoRecover` (`click(Button2, Button3)`, unshared secret, no `runUI`):

- Iteration 1: `backBtn.Clicked` → filter Button1, head=Button2-press → no match. `recoverBtn.Clicked` → filter Button2, head=Button2-press → match, consumes, `c.Pressed=true`, returns false. Guard `!Unshared && false` → skip. `engraveBtn.Clicked` → filter Button3/Center, head=Button2-release → no match. Frame (no Reset, no FrameCallback).
- Iteration 2: `backBtn.Clicked` → head=Button2-release → no match. `recoverBtn.Clicked` → Button2-release → match, consumes, `c.Pressed=true→false`, clicked=true, returns true. Guard `!Unshared(=true) && true` → false, skip. `engraveBtn.Clicked` → head=Button3-press → match, consumes, `c.Pressed=true`, returns false. Frame.
- Iteration 3: `backBtn` → head=Button3-release → no match. `recoverBtn` → Button3-release → no match. `engraveBtn` → Button3-release → match, consumes, clicked=true, returns true. Returns `codex32Engrave`. Test assertion passes.

Mechanical trace of `TestConfirmCodex32ShareOffersRecover` (`click(Button2)`, share index A, `!Unshared=true`, no `runUI`):

- Iteration 1: `backBtn` → no match (Button2-press). `recoverBtn` → Button2-press consumed, `c.Pressed=true`, clicked=false, returns false. Guard false. `engraveBtn` → head=Button2-release → no match.
- Iteration 2: `recoverBtn` → Button2-release consumed, clicked=true, returns true. Guard `true && true` → returns `codex32Recover`. Test assertion passes.

Production `runUI` path: `ctx.Reset()` called in `FrameCallback` after each `ctx.Frame`. `Reset()` calls `r.Router.Reset()` which discards head events not claimed by any registered filter. Under `runUI`, the Button2 event is registered as a filter (via `recoverBtn.Clicked → Router.Next → r.filters append`), so `Reset()` does NOT discard it — it stops at the Button2 event. On the next frame, `recoverBtn.Clicked` again processes it. Production behavior is identical to before for shares (Button2 → recover) and for unshared secrets (Button2 drained, inert).

No regression to `TestEngraveCodex32BackoutNotUnknown`: that test uses an unshared secret, clicks Button1, calls `engraveObjectFlow` (no `runUI`). After Task 3, `engraveObjectFlow case codex32.String:` delegates to `engraveCodex32` → `confirmCodex32Flow`. Button1 at head: `backBtn.Clicked` → match on press, iterate, match on release, clicked=true, returns `codex32Back`. `engraveCodex32` returns `true`. Test passes.

**C1: FULLY RESOLVED.**

---

### I1 VERIFICATION — `engraveCodex32` import completeness in `gui/codex32_polish.go`

**Finding:** FOLD CORRECT AND COMPLETE.

Current imports in `/scratch/code/shibboleth/seedhammer/gui/codex32_polish.go` (lines 3–13): `fmt`, `image`, `strings`, `seedhammer.com/codex32`, `seedhammer.com/gui/assets`, `seedhammer.com/gui/layout`, `seedhammer.com/gui/op`, `seedhammer.com/gui/widget`.

Step 11 adds `seedhammer.com/backup` and `seedhammer.com/font/constant` to this block, alongside `engraveCodex32`.

Confirmed import paths against `gui/gui.go` lines 22 and 30: `"seedhammer.com/backup"` and `"seedhammer.com/font/constant"` — exact match.

Symbol audit across all new functions in Task 3 additions to `codex32_polish.go`:

- `confirmCodex32Flow` (Step 7): `codex32.ParsePrefix` (`codex32` ✓), `strings.ToUpper` (`strings` ✓), `fmt.Sprintf` (`fmt` ✓), `Clickable`/`NavButton`/`StyleSecondary`/`StylePrimary`/`leadingSize`/`layoutNavigation`/`layoutTitle`/`ctx.Frame` (same package `gui`, no import), `assets.IconBack`/`IconRight`/`IconHammer` (`assets` ✓ — all three icons confirmed present in `assets/embed.go`), `layout.Rectangle` (`layout` ✓), `op.Op`/`op.Layer`/`op.Color` (`op` ✓), `widget.Labelw` (`widget` ✓), `image.Pt` (`image` ✓).
- `showCodex32Error` (Step 8): `ErrorScreen` (same package `gui`, no import), `op.Layer`/`op.Color` (`op` ✓).
- `recoverCodex32Flow` (Step 8): `codex32.ParsePrefix`/`codex32.ConsistentShares`/`codex32.Describe`/`codex32.Interpolate`/`codex32.String` (`codex32` ✓), `strings.ToUpper` (`strings` ✓), `fmt.Sprintf` (`fmt` ✓), `inputCodex32Flow`/`showCodex32Error` (same package `gui`, no import).
- `engraveCodex32` (Step 11): `confirmCodex32Flow`/`recoverCodex32Flow`/`backupSeedStringFlow`/`codex32.String`/`codex32ConfirmAction` enum constants (same package `gui`, no import), `scan.Split()` (method on `codex32.String`, via `codex32` ✓), `backup.SeedString` (`backup` — added Step 11 ✓), `constant.Font` (`font/constant` — added Step 11 ✓).

No symbol is missing an import at any function boundary. No import becomes unused at any stage: `backup` and `constant` are first used in `engraveCodex32` (Step 11), which is also when they are added. Task 3 is declared atomic (single commit at Step 15) so there is no intermediate commit state where `backup`/`constant` would be imported but unused.

**I1: FULLY RESOLVED.**

---

### REGRESSION SCAN

**Scope drift:** None. The two fold changes are surgically minimal — one expression split in Step 7, two import lines in Step 11. No other step text was altered.

**Internal consistency:** The `codex32ConfirmAction` enum, `confirmCodex32Flow` signature, `recoverCodex32Flow` signature, `engraveCodex32` signature, and all cross-references between tasks remain consistent. The `inputCodex32Flow(ctx, th, title string)` signature change (3a) is referenced correctly by `newInputFlow` (Step 2), `codex32Frame` test helper (Step 3), and `recoverCodex32Flow` (Step 8).

**Build-order/atomicity:** Task 0 → Task 1 → Task 2 each form independent compilable commits. Task 3 is declared atomic (Steps 3a–3e committed together at Step 15). The intra-Task-3 compile break (Steps 7–10, before Step 11 fixes the `gui.go` caller) is documented and does not affect the shipped commit sequence. Step 4 (build-check after 3a) precedes Step 7, so it tests only the `inputCodex32Flow` signature change, which is safe.

**Interpolate/Split()/mdmk.go:** Confirmed untouched. `codex32.go`, `codex32/mdmk.go` are guard-listed as unchanged. The plan adds only to `codex32/polish.go` (Tasks 1–2) and `gui/` files (Task 3). `Split()` is used solely at `engraveCodex32` line `id, _, _ := scan.Split()` — existing usage, no change.

**A1 tests:** `TestConfirmCodex32Unshared` — asserts "Unshared secret"/"id TEST"; new code still emits these strings for unshared. `TestConfirmCodex32Share` — asserts "Share A" and "not a recovered seed"; Step 12 updates the latter to "Recover the secret". `TestInputSeedCodex32` — unchanged; `inputCodex32Flow` signature change is backward-compatible for the test harness since the test calls it via `codex32Frame` which Step 3 updates. `TestEngraveCodex32BackoutNotUnknown` — verified above.

**R0 green-table items remain valid:** ConsistentShares design (correct sentinel subset of Interpolate's pass-1), `fe`-as-map-key comparability (`type fe uint8` confirmed in `codex32/gf32.go:50`), `parts()` field names (`hrp`, `threshold`, `id`, `shareIdx` confirmed in `codex32.go:404-411`), `Unshared`/`ThresholdKnown`/`Threshold`/`Identifier` field names on `Fields` struct (confirmed in `codex32/polish.go:51-59`), `ErrorScreen` in same package (confirmed `gui/gui.go:198`), `backupSeedStringFlow` in same package (confirmed `gui/gui.go:1956`), `IconRight` exists (confirmed `assets/embed.go:119`).

---

### MINOR ITEMS (no action required)

**M1 (pre-existing, not introduced by fold):** Step 10's explanatory note ("If Step 8 compiled, run them now and expect PASS") is misleading: the `./gui/` package cannot compile between Steps 7 and 11 because `gui.go:1842` still uses the old `bool` return type of `confirmCodex32Flow`. The tests in Step 9 will fail to compile until Step 11. The "Proceed to Step 11 regardless" instruction is correct; the "expect PASS" phrase is wrong. This is a pre-existing documentation imprecision from before R0, unchanged by the fold, and does not affect implementation correctness.

**M2 (pre-existing, carried forward):** The long-code (125–127 char) recovery path is noted in the execution-review scope ("note/observe; add a backup-level long-code test if cheap") but no `TestRecoverCodex32Long` is in the plan. Correctly deferred to the mandatory post-implementation adversarial review per the Final step. Not a plan-gate issue.

---

### VERDICT

**GREEN — 0 Critical / 0 Important.**

Both R0 findings are correctly and completely folded. C1's drain resolves the direct-call hang without altering production `runUI` semantics or the share-recovery path. I1's import additions are at the correct location (Step 11, first use), match the exact import paths in `gui.go`, and leave no symbol unimported and no import unused at any task boundary. No new defects were introduced by the fold. The plan is internally consistent, build-order is sound, all guard files are untouched, and the A1 regression surface is covered.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | RED — 1C/1I | C1 event-queue hang (drain Button2 unconditionally); I1 missing backup/constant imports. |
| R1 | **GREEN — 0C/0I** | Both folds verified by a full mechanical EventRouter trace (C1: Button2 consumed-but-ignored for unshared → Button3 → codex32Engrave; share → codex32Recover; runUI semantics unchanged) + import audit (I1: backup/constant paths match gui.go, no symbol unimported, no import unused at any boundary). No regression; A1 tests + TestInputSeedCodex32 covered; Interpolate/Split/mdmk untouched. 2 Minors (M1 Step-10 "expect PASS" doc imprecision — **fixed post-R1 as a doc-only clarity edit**; M2 long-code recovery test deferred to the execution review). |

GATE PASSED at R1. (Step-10 wording corrected after R1 — doc-only, no code change.) Proceed to single-implementer TDD → whole-diff execution review.
