# Cycle B: codex32-multishare-recovery — IMPLEMENTATION PLAN R0 GATE REVIEW — R0

- **Stage:** mandatory implementation-plan R0 gate (0C/0I before any code).
- **Plan reviewed:** `design/IMPLEMENTATION_PLAN_seedhammer_codex32_multishare_recovery.md` (committed `d3c3505`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `aa801a9a44efd4bfc`), read-only vs fork `main` `bf7f811`.
- **Outcome:** **RED — 1 Critical (C1) + 1 Important (I1).** Both folded; re-dispatched R1.

> NOTE: verbatim architect output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

## R0 GATE REVIEW — Cycle B: CODEX32 Multi-Share Recovery Implementation Plan

Reviewer: opus architect (adversarial, read-only)
Base: `bf7f811`, branch `feat/codex32-multishare-recovery`

---

### CRITICAL FINDINGS

**C1 — `TestConfirmCodex32UnsharedNoRecover` hangs: Button2 is never drained from the event queue in the unshared path.**

File: `gui/codex32_polish_test.go` (new test in Task 3b, Step 5)

The test queues `click(&ctx.Router, Button2, Button3)` and then calls `confirmCodex32Flow` directly (no `runUI`, no `FrameCallback`, no `ctx.Reset()`). In `confirmCodex32Flow`, the condition `if !f.Unshared && recoverBtn.Clicked(ctx)` short-circuits when `f.Unshared == true`: `recoverBtn.Clicked(ctx)` is **never called** and the Button2 press/release events accumulate permanently at the head of `EventRouter.events`. `EventRouter.Next` always checks only `events[0]`; if Button2 is there, `backBtn.Clicked(ctx)` (looking for Button1) and `engraveBtn.Clicked(ctx)` (looking for Button3) both return false every iteration. Without a `FrameCallback`, `ctx.Frame` only calls `B.Reset()` — it does not call `ctx.Router.Reset()` and does not discard the unmatched Button2 events. The loop is infinite. The test hangs.

The `runUI`-based existing tests (`TestConfirmCodex32Unshared`, `TestConfirmCodex32Share`) do not have this problem because `runUI` sets `FrameCallback` which calls `ctx.Reset()` after each frame, which calls `EventRouter.Reset()`, which discards events no registered filter claims.

Concrete fix: always drain the `recoverBtn`, but only act on it conditionally. Replace the short-circuit with:

```go
recoverClicked := recoverBtn.Clicked(ctx)
if !f.Unshared && recoverClicked {
    return codex32Recover
}
```

This drains Button2 events in both branches. Alternatively, rewrite `TestConfirmCodex32UnsharedNoRecover` using `runUI` so `ctx.Reset()` discards unmatched events after each frame.

The current plan's production code structure (short-circuit `if !f.Unshared && recoverBtn.Clicked(ctx)`) is fine in production because `runUI` calls `ctx.Reset()`. It is only broken for the direct-call test pattern. But since the plan specifies this exact test, this is a CRITICAL defect: the test will not pass as written.

---

### IMPORTANT FINDINGS

**I1 — `engraveCodex32` placed in `codex32_polish.go` requires two missing imports.**

File: `gui/codex32_polish.go`, Task 3d Step 11.

The plan places `engraveCodex32` in `gui/codex32_polish.go` "appended after `recoverCodex32Flow`". `engraveCodex32` references `backup.SeedString` and `constant.Font`. The current import block of `gui/codex32_polish.go` contains neither `seedhammer.com/backup` nor `seedhammer.com/font/constant`. The plan does not instruct the implementer to add them. The package will fail to compile at Task 3d until these are added. The plan's Step 11 code block is not compile-accurate as printed.

Concrete fix: the Step 11 instruction must include an explicit "add to the import block of `gui/codex32_polish.go`:" directive listing:
```
"seedhammer.com/backup"
"seedhammer.com/font/constant"
```

---

### MINOR FINDINGS

**M1 — `ConsistentShares` field-mismatch loop iterates `shares[0]` against itself (index 0).**

File: `codex32/polish.go`, Task 1 Step 3.

The loop `for _, share := range shares` starting from index 0 compares `shares[0]` against `s0 = shares[0].parts()`. This is a trivially-correct no-op self-comparison (all four fields match). It mirrors `Interpolate`'s loop structure exactly. No panic risk, no correctness defect. The duplication of `share.parts()` calls (in the field-check loop for `shares[0]`, and again in the distinct-index loop for all shares) is minor inefficiency, not a defect. No fix required.

**M2 — `errInvalidIDLength` is unhandled in `Describe` (pre-existing, not introduced by this plan).**

File: `codex32/polish.go`. `errInvalidIDLength` exists at `codex32.go:36` but is not mentioned in `Describe` (the plan does not add it and neither does it currently exist). Not a gap introduced by Cycle B. Note only.

**M3 — Task 3 atomicity note is accurate but Step 10's "should actually PASS" prediction is contingent.**

File: Plan Task 3c Step 10. The plan correctly notes that `TestRecoverCodex32` / `TestRecoverCodex32Mismatch` call `recoverCodex32Flow` directly and may pass after Step 8. This is correct: both tests are direct-call or `runUI`-based and do not depend on `engraveCodex32` existing. The note is honest and not a defect; minor for clarity only.

**M4 — `TestRecoverCodex32Mismatch` only checks up to 8 `frame()` iterations; if the UI is slow to reach the error modal, the test could miss it.**

The test polls 8 frames for "mismatched". In practice the frame sequence is: `inputCodex32Flow` renders its first frame (with the typed runes + Button3 pre-queued), OK is processed, `showCodex32Error` renders its first error frame, `frame()` yields it. At most 2-3 frames should be needed. 8 is generous and should be fine in the test harness. No fix required.

---

### COMPILE-ACCURACY VERIFICATION SUMMARY (file:line evidence)

| Plan claim | Reality | Verdict |
|---|---|---|
| `s.parts()` callable same package | `codex32.go:176` — `func (s String) parts() *parts` | CORRECT |
| `s.s` accessible same package | `codex32.go:17` — `type String struct { s string }` | CORRECT |
| `parts.hrp/threshold/id/shareIdx/payload/checksum` fields | `codex32.go:403-410` | CORRECT |
| `errMismatchedLength/HRP/Threshold/ID/RepeatedIndex/InsufficientShares` sentinel names | `codex32.go:32-37` | CORRECT |
| `fe` is `uint8` → comparable map key | `gf32.go:50` | CORRECT |
| `Describe` switch before `default:` insertion point | `polish.go:26-47` | CORRECT |
| `{errInsufficientShares, "invalid"}` row in existing `TestDescribe` | `polish_test.go:46` | CORRECT |
| Current `confirmCodex32Flow` returns `bool`, caller at `gui.go:1842` | `codex32_polish.go:72`, `gui.go:1842` | CORRECT |
| `inputCodex32Flow` signature `(ctx *Context, th *Colors)` | `gui.go:672` | CORRECT |
| `title, _ := layoutTitle(...)` local at line 745, used at line 749 | `gui.go:745,749` | CORRECT |
| `newInputFlow` `case 2:` call at `gui.go:2010` | `gui.go:2010` | CORRECT |
| `engraveObjectFlow case codex32.String:` block at `gui.go:1841-1854` | `gui.go:1841-1854` | CORRECT |
| `ErrorScreen{Title, Body}` struct literal | `gui.go:198-203` | CORRECT |
| `ErrorScreen.Layout(ctx, th, dims) (op.Op, bool)` | `gui.go:205` | CORRECT |
| `NavButton{Clickable, Style, Icon}` fields | `gui.go:1655-1660` | CORRECT |
| `StyleSecondary/StylePrimary` consts | `gui.go:1651-1652` | CORRECT |
| `assets.IconBack/IconRight/IconHammer` exist | `assets/embed.go:119-120` (IconRight confirmed); IconBack/Hammer grep confirmed | CORRECT |
| `layoutTitle(ctx, width, col, title) (op.Op, image.Rectangle)` | `gui.go:1633` | CORRECT |
| `Clickable{Button, AltButton}` struct fields | `widget.go:7-10` | CORRECT |
| `widget.Labelw(&ctx.B, style, width, col, str) (op.Op, image.Point)` | `gui.go:107,700` pattern confirmed | CORRECT |
| `layout.Rectangle{Max: dims}`, `CutTop`/`CutBottom` | `gui.go:615-617` pattern | CORRECT |
| `leadingSize` const exists in `gui` | `gui.go:288,1721` | CORRECT |
| `backup.SeedString{Title, Seed, Font}` | `backup/backup.go:25-30` | CORRECT |
| `backupSeedStringFlow(ctx, th, s backup.SeedString)` | `gui.go:1956` | CORRECT |
| `scan.Split()` → `(id, threshold, idx)` | `codex32.go:394` | CORRECT |
| `codex32Frame` test helper calls `inputCodex32Flow(ctx, &descriptorTheme)` | `codex32_polish_test.go:75` | CORRECT |
| `TestConfirmCodex32Share` checks `"not a recovered seed"` | `codex32_polish_test.go:144` | CORRECT (plan updates this to "Recover the secret") |
| BIP-93 vector-2 shares A+C → secret `MS12NAMES6XQGUZTTXKEQNJSJZV4JV3NZ5K3KWGSPHUH6EVW` | `codex32_test.go:58-69` | CORRECT |
| `MS13CASHA320...` uppercase is New-valid | `codex32_test.go:83` (lowercase passes `mustFromString`) | CORRECT |
| Keypad force-uppercases runes | `gui.go:1153` | CORRECT |
| `b/i/o` absent from `MS13CASHA320...` string | Manual check of bech32 chars | CORRECT |
| `engraveCodex32` in `codex32_polish.go` — missing `backup`/`constant` imports | `codex32_polish.go:6-13` (current imports) | MISSING IMPORTS — I1 |
| `!f.Unshared && recoverBtn.Clicked(ctx)` drains Button2 for unshared | Event router analysis | FAILS for direct-call test — C1 |
| `Interpolate(shares, 'S')` recovers the secret | BIP-93 confirmed | CORRECT |

---

### VERDICT

**RED — 1 Critical / 1 Important**

**C1** (CRITICAL): `TestConfirmCodex32UnsharedNoRecover` hangs. The direct-call test pattern with no `runUI` means Button2 events block the queue and Button3 is never consumed. The fix is either (a) change the short-circuit to unconditionally drain `recoverBtn` and then conditionally return, or (b) rewrite the test using `runUI`. Must be resolved before implementation begins.

**I1** (IMPORTANT): `engraveCodex32` placed in `gui/codex32_polish.go` references `backup.SeedString` and `constant.Font` but neither `seedhammer.com/backup` nor `seedhammer.com/font/constant` is in that file's import block. Step 11 is not compile-accurate as printed. Must add both imports to the instruction.

All other plan components — sentinel names, struct fields, function signatures, BIP-93 vector correctness, `ConsistentShares` two-pass logic, `Interpolate`'s pass-1 correspondence, `fe` map-key comparability, test vectors, loop termination, scope discipline, and build order — are verified correct against the `bf7f811` source tree.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | RED — 1C / 1I | **C1** (Critical): `TestConfirmCodex32UnsharedNoRecover` would hang — `if !f.Unshared && recoverBtn.Clicked(ctx)` short-circuits, so Button2 is never drained for an unshared secret, blocking the EventRouter queue head (no runUI/ctx.Reset to discard it) so Button3 never matches → infinite loop. **Folded:** drain `recoverBtn` unconditionally (`recoverClicked := recoverBtn.Clicked(ctx)`), act only when `!f.Unshared`. **I1** (Important): `engraveCodex32` in `gui/codex32_polish.go` uses `backup.SeedString`/`constant.Font` but that file imports neither. **Folded:** Step 11 now adds `seedhammer.com/backup` + `seedhammer.com/font/constant` to the import block. Architect verified everything else compile-accurate (full green table: sentinels, parts fields, fe map-key, Describe insertion point, confirmCodex32Flow/inputCodex32Flow/newInputFlow/engraveObjectFlow sites, ErrorScreen, layoutNavigation/NavButton/icons incl. IconRight, BIP-93 vector-2 recovery, MS13CASHA… New-valid + b/i/o-free, ConsistentShares two-pass order, loop termination, scope discipline, build order). 4 Minors = notes (self-comparison no-op, pre-existing errInvalidIDLength, Step-10 contingency, 8-frame poll) — no action. |

Re-dispatched R1 after the fold.
