# Cycle B: codex32-multishare-recovery — SPEC R0 GATE REVIEW — R1

- **Stage:** spec gate round R1 (re-review after folding R0's 2 Important + 5 Minor).
- **Spec reviewed:** `design/SPEC_seedhammer_codex32_multishare_recovery.md` (committed `3a7b721`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `a29d94c3481eb505a`), read-only vs fork `main` `bf7f811`.
- **Outcome:** **GREEN — 0 Critical / 0 Important.** Spec cleared to the plan phase.

> NOTE: verbatim architect output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

## R1 Architect Review — SPEC_seedhammer_codex32_multishare_recovery.md

**Reviewer:** opus architect (R1, post-fold)
**Base:** fork `main` `bf7f811`; spec dated 2026-06-18
**Scope:** verify R0 fold correctness and completeness; regression scan over unchanged sections; confirm 0C/0I

---

### R0 Finding Verification

**I1 (Important) — `ConsistentShares` godoc precondition**

Verified. §4.1(a) now reads (verbatim from the spec):

> "PRECONDITION (R0 I1): every share MUST already be New-valid — ConsistentShares calls the unexported parts(), which PANICS on a malformed String. Callers must only pass strings that passed New without error (the keypad gates the OK button on New==nil, so recoverCodex32Flow upholds this). State this in the godoc."

The precondition is clearly stated, the panic mechanism is explained (calling `parts()`), the upholding mechanism is named (keypad gates on `New==nil`), and the requirement to put it in the exported godoc is explicit. The fold is complete and accurate for I1.

Cross-check against source: `codex32.go:176–183` confirms `parts()` calls `partsInner()` and panics with `panic("unreachable")` on any error. The spec's claim is correct.

**I2 (Important) — `recoverCodex32Flow` defensive guard on `k` derivation**

Verified. §4.2 now contains the exact guard:

```go
f, _ := codex32.ParsePrefix(first.String())
if !f.ThresholdKnown || f.Threshold < 2 { // unreachable for a New-valid share; defensive
    return codex32.String{}, false
}
k := f.Threshold
```

The comment "unreachable for a New-valid share; defensive" satisfies the self-documenting requirement. The condition `f.Threshold < 2` correctly excludes threshold 0 (which `ParsePrefix` does allow, for unshared secrets) and threshold 1 (which BIP-93 prohibits and `partsInner` would reject, but `ParsePrefix` would accept if somehow the data were `'1'` — though the switch in `ParsePrefix:91–98` actually excludes `'1'` as a valid threshold digit, so `ThresholdKnown` would be false). The guard is sound.

Cross-check: `ParsePrefix` at `polish.go:91–98` maps `'0','2'..'9'` to threshold values, and threshold 1 is not in the switch, so `ThresholdKnown` stays false for it. A `New`-valid share with threshold ≥ 2 always has `ThresholdKnown=true` and `Threshold ∈ {2..9}` — the guard's happy path is never blocked by a real share. The fold is correct.

**M1 — Title branch in `confirmCodex32Flow`**

Verified. §4.2 states: "Title branch (M1): `confirmCodex32Flow` currently hardcodes the title `"Confirm Codex32 Share"`. Branch it on `Unshared` → `"Confirm Codex32 Secret"` (so the re-confirm of a recovered unshared secret isn't mistitled "Share")."

This is present and correct. The current fork source at `codex32_polish.go:99` hardcodes `"Confirm Codex32 Share"` — confirming the problem is real and the fix is accurately described. The title branching covers the recovered-secret re-confirm path (`engraveCodex32` loops back with `scan = secret`, a string with `Unshared=true`). The fold is complete.

**M4 — Explicit "enter a share, not the secret" message**

Verified. §4.2 reads: "reject a second unshared secret via `ParsePrefix(cand).Unshared` with an explicit `ErrorScreen` message **"enter a share, not the secret"** (M4 — do NOT reuse the generic "bad share index" label, which would misdescribe it)"

The fold correctly distinguishes the two error cases:
1. Unshared secret entered where a share is expected → `"enter a share, not the secret"` (M4 path, using `Unshared` check before `ConsistentShares`)
2. Cross-share consistency failure → `codex32.Describe(err)` (which maps `errRepeatedIndex` → "repeated share", `errMismatchedID` → "mismatched id", etc.)

The ordering in §4.2 places the unshared-secret check before `ConsistentShares`, which is correct: a recovered unshared secret entered as a candidate would pass `ConsistentShares` only if it happens to have an `S` index that looks like a share (impossible by BIP-93 — `S` is the unshared index and `errInvalidShareIndex` would fire in `Interpolate` anyway), but the intent to check `Unshared` first is sound and gives the user a better message. Fold is complete.

**M5 — `scan.Split()` used only for `id`**

Verified. §4.3 contains: "(M5: `scan.Split()` here is used **only for `id`** — its threshold and index returns are discarded, so `Split()`'s threshold-0→1 remap is irrelevant on this path; this matches A1's existing engrave behavior exactly.)"

The code snippet in §4.3 uses `id, _, _ := scan.Split()`, which makes the discard explicit. The current fork code at `gui.go:1848` does the same: `id, _, _ := scan.Split()`. The fold is accurate.

**M2 — Long-code engrave test recommendation**

Verified. §6 contains: "**Long-code engrave (M2):** a recovered secret can be a long code (125–127 chars, 256-bit, threshold ≥ 2). The existing `backupSeedStringFlow`/`EngraveSeedString` path handles it (math checked: `ngroups=⌈127/10⌉=13 ≤ maxCol1=16`), but there is no existing golden engrave test for a long codex32 string. Add a `backup`-level test (or a gui assertion) engraving a 127-char codex32 secret to close the gap — **recommended, not a GREEN blocker**."

The caveating as "recommended, not a GREEN blocker" is the correct disposition (this is a new gap discovered, but it is a test-coverage gap for an existing path, not a correctness error). Fold is complete.

**M3 — Multi-screen test driving via `frame()`**

Verified. §6 contains: "**Multi-screen driving (M3):** these flows span several screens, so the test advances `frame()` step-by-step (queue the runes/clicks for the next screen, call `frame()` to render+assert, repeat) — not the single "queue everything then one `frame()`" shape. Where a step pre-queues all input (as `TestInputSeedCodex32` does), document that the assertion is on the terminal frame."

The pattern description is accurate to the actual `runUI`/`frame` harness in `gui_test.go:466–476`. `TestWordFlowLastWordNoFlash` (line 669) shows the step-by-step pattern in use. Fold is complete.

---

### Regression Scan

**`confirmCodex32Flow` bool→action enum change**

The spec at §4.2 changes `confirmCodex32Flow` from returning `bool` to returning `codex32ConfirmAction`. The current fork has one call site for `confirmCodex32Flow`: `gui.go:1842`, within `engraveObjectFlow`. The spec refactors that call site into `engraveCodex32` (§4.3), which becomes the sole caller of `confirmCodex32Flow`. The existing tests in `codex32_polish_test.go:108–145` (`TestConfirmCodex32Unshared`, `TestConfirmCodex32Share`) call `confirmCodex32Flow` directly and currently assert on displayed text, not on the return value — those tests remain valid after the signature change as long as the display logic is unchanged. `TestEngraveCodex32BackoutNotUnknown` (`codex32_polish_test.go:197`) calls `engraveObjectFlow`, which routes through the new `engraveCodex32`; it clicks `Button1` (Back), which maps to `codex32Back`, and `engraveCodex32` returns `true` — the test's expectation is preserved. No regression.

**`engraveCodex32` termination**

§4.3 shows `engraveCodex32` loops until `confirmCodex32Flow` returns `codex32Back` or `codex32Engrave`. When the recovered unshared secret re-enters the loop, `confirmCodex32Flow` for an unshared string only offers Back (returning `codex32Back`) and Engrave (returning `codex32Engrave`) — the `codex32Recover` branch requires `index ≠ S`. So the loop always terminates: either the user backs out, or they engrave. No infinite-loop risk. The spec's claim that "Recover not offered for S" correctly ensures termination. Confirmed.

**`Interpolate`/`Split()`/`mdmk.go` untouched**

The spec explicitly states these are out of scope in §2 and §4.3, with no edits implied. `Interpolate` at `codex32.go:185` is left intact; `Split()` at `codex32.go:394` is left intact; `mdmk.go` is not mentioned in any change. Confirmed.

**I2 guard does not break the happy path**

`ParsePrefix` on a `New`-valid share with threshold digit `'2'..'9'` returns `ThresholdKnown=true` and `Threshold ∈ {2..9}`. The guard `!f.ThresholdKnown || f.Threshold < 2` evaluates to `false || false = false`, so the guard does not fire. The happy path reaches `k := f.Threshold` intact. Confirmed.

**`scan.Split()` threshold-0→1 remap in `engraveCodex32`**

`Split()` at `codex32.go:394–401` maps threshold `0 → 1` before returning. In `engraveCodex32`, `id, _, _ := scan.Split()` discards the threshold return. The remap has no effect on this path. The spec's M5 note is accurate. Confirmed.

**`ConsistentShares` duplication vs. `Interpolate` pass-1**

The spec notes in §4.1(a) that `ConsistentShares` duplicates `Interpolate`'s pass-1 comparisons and that this is the deliberate choice ("default: leave it untouched"). `Interpolate` at `codex32.go:199–213` checks length, HRP, threshold, and ID in its first loop; `ConsistentShares` will make the same checks. The spec is architecturally consistent. The defense-in-depth call to `Interpolate` after `ConsistentShares` + count check is correct.

**`errMismatchedHRP` label**

§4.1(b) maps `errMismatchedHRP` → "mismatched type". This is mildly surprising (the error name is "mismatched hrp") but HRP is an implementation detail; "mismatched type" (ms vs. ms10? — actually HRP is the prefix before `1`, e.g. "ms") is a reasonable user-facing label for a 480×320 display. No correctness issue.

**`errInsufficientShares` in existing `Describe`**

The current `Describe` at `polish.go:46` returns `"invalid"` for `errInsufficientShares`. The spec §4.1(b) maps it to "need more shares". The existing `TestDescribe` at `polish_test.go:46` tests `errInsufficientShares → "invalid"`. That test will need to be updated when `Describe` is extended. The spec's §6 does call for updating `Describe` tests ("table tests … `Describe` returns the new labels for each cross-share sentinel"), so this is correctly scoped. However, the spec does not explicitly note that the existing `TestDescribe` assertion `{errInsufficientShares, "invalid"}` must be changed to `{errInsufficientShares, "need more shares"}`. This is a minor implementation reminder, not a spec defect — the implementer will catch it when the test fails.

**BIP-93 vector share pairing for tests**

§6 proposes using `MS12NAMEA…` + `MS12NAMEC…` as the test pair for `ConsistentShares`. From `codex32_test.go:57–59` (TestBIPVector2), the two shares are `MS12NAMEA320ZYXWVUTSRQPNMLKJHGFEDCAXRPP870HKKQRM` and `MS12NAMECACDEFGHJKLMNPQRSTUVWXYZ023FTR2GDZMPY6PN`, both threshold-2, same id "NAME". These are real BIP-93 vectors — the test data reference is valid.

**`codex32Recover` button assignment**

§4.2 assigns `codex32Recover` to Button2. The current `confirmCodex32Flow` uses Button1 (Back) and Button3 (Engrave). Adding Button2 (Recover) in the middle gives a three-button layout, which `layoutNavigation` supports (the descriptor/wallet flows already use three-button layouts). No layout constraint issue identified.

**`ParsePrefix` for a `New`-valid share's `Unshared` field**

§4.2 uses `ParsePrefix(cand).Unshared` to detect an unshared secret entered as a candidate. For a `New`-valid string, `ParsePrefix` returns `ShareIndexKnown=true` and `Unshared=(idx=='s'||idx=='S')`. Index `'S'` maps to `feS` in the gf32 table (confirmed: `gf32.go:69`). The check is correct.

---

### Minor Findings

**MINOR-R1-1: `TestDescribe` update not explicitly called out**

`polish_test.go:46` asserts `errInsufficientShares → "invalid"`. When §4.1(b) maps it to "need more shares", this test will break. The spec's §6 instructs adding `Describe` tests for new labels but does not note that the existing assertion must be changed from "invalid" to "need more shares". This is a mechanical implementation detail the implementer will catch at compile/test time; it is not a spec gap that could lead to confusion, and it does not affect correctness. No C/I.

**MINOR-R1-2: `codex32ConfirmAction` type in package scope**

The spec defines `codex32ConfirmAction` as an unexported type in the `gui` package. The existing `TestConfirmCodex32Unshared` and `TestConfirmCodex32Share` tests are in `package gui` (same package), so they can call `confirmCodex32Flow` and use the unexported type directly. No accessibility issue. No C/I.

**MINOR-R1-3: `recoverCodex32Flow` signature — `first` as `codex32.String` not `string`**

The spec writes `recoverCodex32Flow(ctx, th, first codex32.String)`. The call to `ParsePrefix` uses `first.String()` (converting to the raw string). This is consistent with `confirmCodex32Flow`'s pattern. No issue.

---

### Verdict

GREEN — 0 Critical / 0 Important

All seven R0 findings (I1, I2, M1, M2, M3, M4, M5) are correctly folded and verified against fork source `bf7f811`. No regression introduced. The design is internally consistent. The three minor findings above are implementation reminders, none of which represent a spec defect or ambiguity that could mislead an implementer into a wrong outcome.

The spec is cleared to proceed to the implementation plan phase, subject to the project's standard R0 gate on the implementation plan itself before any code is written.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | NOT GREEN — 0C/2I/5m | I1 ConsistentShares precondition; I2 k-guard; M1 title; M4 rejection msg; M5 Split()-id; M2 long-code test; M3 frame() driving. |
| R1 | **GREEN — 0C/0I** | All seven folds verified against `bf7f811`; no regression (bool→action enum callers/tests fine, engraveCodex32 terminates, Interpolate/Split/mdmk untouched, I2 guard doesn't block the happy path, vector-2 shares valid, Button2 free for Recover). 3 non-blocking R1 Minors = implementation reminders: **R1-1** existing `TestDescribe` asserts `errInsufficientShares→"invalid"` → must become "need more shares" when Describe is extended (capture in the plan); R1-2 `codex32ConfirmAction` unexported type is fine (tests same package); R1-3 `recoverCodex32Flow(first codex32.String)` signature fine. |

GATE PASSED at R1. Proceed to the implementation plan (which itself gets an R0 gate before code).
