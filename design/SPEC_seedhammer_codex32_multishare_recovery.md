# SPEC — SeedHammer CODEX32 multi-share recovery (Cycle B)

**Date:** 2026-06-18
**Target repo:** the SeedHammer II firmware fork `bg002h/seedhammer` (Go/TinyGo). Fork-side only — **no upstream PR** (post-#36 directive; the fork is the maintained line).
**Base:** fork `main` `bf7f811` (post-Cycle-A1 — the codex32 input polish, incl. `confirmCodex32Flow`, `ParsePrefix`/`Describe`, the dimmed keypad). Branch this cycle `feat/codex32-multishare-recovery` off `bf7f811`.
**Predecessors:** `design/cycle-prep-recon-codex32-multishare-recovery.md` (verified vs `bf7f811` + BIP-93), `design/RECON_seedhammer_slip39_codex32_input.md`.
**Out of scope:** decoding the recovered secret to a BIP-39 mnemonic / SeedQR (**Option B** — caveated & deferred, see §8); SLIP-39 (Cycle C); changing `Interpolate`/`Split()`/`mdmk.go` crypto; widening any length/plate-fit gate.

---

## 1. Goal

Close the one *correctness* gap in the on-device codex32 feature: the device can enter and engrave a single share verbatim but **cannot reconstruct the unshared secret from a k-of-n share set** (`codex32.Interpolate` has zero `gui` callers). Add an on-device recovery flow that collects k valid shares, reconstructs the unshared secret via `Interpolate(shares,'S')`, and engraves that recovered secret **verbatim through the existing codex32 engrave path** (Option A — user-chosen).

Crypto already exists and is BIP-93-conformant (`Interpolate` + `String.Seed()`); this is a **GUI-flow + small-API** cycle. No new crypto.

## 2. Scope

**In:**
- **B1 — codex32 package API:** a cross-share consistency checker the GUI can call incrementally, + extend `Describe` to label the cross-share sentinels.
- **B2 — share-collection flow (gui):** reached by branching the existing CODEX32 confirm — when an entered string is a *share* (index ≠ `S`), offer **Recover** alongside **Engrave this share**; collect shares 2..k with a "Share i of k" counter, validating each as it is added.
- **B3 — recovery + engrave (gui):** once k valid shares are collected, `Interpolate(shares,'S')` → recovered unshared-secret `codex32.String` → re-confirm → engrave verbatim via the **existing** `backupSeedStringFlow` path.

**Out:** Option B (BIP-39/SeedQR decode); SLIP-39; any change to `Interpolate`/`Split()`/`mdmk.go`; plate-fit changes (the recovered secret is a codex32 string of the same length class the single-share path already engraves).

**Files:** `codex32/polish.go` (+ `codex32/polish_test.go`) for B1; `gui/codex32_polish.go` + `gui/gui.go` (`engraveObjectFlow case codex32.String:`, `confirmCodex32Flow`) for B2/B3; `gui/codex32_polish_test.go` for tests. `Interpolate`/`Split()`/`codex32.go`/`mdmk.go` **unchanged**. `TestWordKeyboardScreen` + `TestInputSeedCodex32` + all A1 tests must stay green.

## 3. Background — the relevant code on `bf7f811`

- `codex32.Interpolate(shares []String, index rune) (String, error)` (`codex32.go:185`): `'S'` recovers the unshared secret. Errors (cross-share): `errMismatchedLength` (`:202`), `errMismatchedHRP` (`:205`), `errMismatchedThreshold` (`:208`), `errMismatchedID` (`:211`), `errInsufficientShares` (`:191`/`:229`, incl. threshold > len(shares)), `errRepeatedIndex` (`:245`), `errInvalidShareIndex` (`:195`). Threshold-vs-count and repeated-index are checked late; the four field-mismatches in pass 1.
- `String.Seed() []byte` (`codex32.go:386`) exists (the recovered secret → bytes path), but is **not needed** for Option A (we engrave the codex32 string verbatim).
- A1 helpers in `codex32/polish.go`: `ParsePrefix(frag) (Fields, error)` (`Fields.Threshold/ThresholdKnown`, `.Identifier`, `.ShareIndex`, `.Unshared`), `Describe(err) string` (maps the single-share `New` sentinels; does **not** yet map the cross-share mismatch sentinels), exported length consts.
- A1 GUI: `inputCodex32Flow(ctx,*Colors) (codex32.String, bool)` (`gui.go:672`) returns ONE share; `confirmCodex32Flow(ctx,*Colors, codex32.String) bool` (`gui/codex32_polish.go`) shows id/share-vs-unshared + the "engraves THIS share, not a recovered seed" warning; `engraveObjectFlow case codex32.String:` (`gui.go:1841`) confirms then engraves verbatim via `backupSeedStringFlow`, `return true` on cancel.
- The closest "item i of N" collection precedent is `inputWordsFlow` (`gui.go:539`, `selected`/`len`, `layoutTitlef("Word %d of %d", …)`). No generic list-collector exists — B2 builds one for shares.

## 4. Design

### 4.1 B1 — codex32 cross-share API (the only package change; land + test first)

**(a) `ConsistentShares` — incremental consistency checker.** The GUI must validate each newly-added share against the set *before* it has k shares, so it cannot use `Interpolate` (which returns `errInsufficientShares` for < k shares). Add:
```go
// ConsistentShares reports whether a set of codex32 shares can belong to one
// recovery set: all share the same HRP, threshold, identifier, and total length,
// and all share indices are distinct. It does NOT require the set to be complete
// (k shares) — use it to validate shares as they are collected. Returns the same
// sentinels Interpolate uses (errMismatched{Length,HRP,Threshold,ID},
// errRepeatedIndex), so Describe maps them. A set of 0 or 1 share is consistent.
//
// PRECONDITION (R0 I1): every share MUST already be New-valid — ConsistentShares
// calls the unexported parts(), which PANICS on a malformed String. Callers must
// only pass strings that passed New without error (the keypad gates the OK button
// on New==nil, so recoverCodex32Flow upholds this). State this in the godoc.
func ConsistentShares(shares []String) error
```
Implemented over `parts()` (HRP/threshold/id/shareIdx) + `len(s.s)` (length) + pairwise distinct-index — the **same comparisons `Interpolate`'s pass-1 makes**, minus the count check. **`Interpolate` is NOT modified** (it keeps its own internal checks as defense-in-depth at recovery time; whether to refactor it to call `ConsistentShares` is an R0 call — default: leave it untouched to keep the proven crypto path intact, accept the small duplication).

**(b) Extend `Describe`** to also label the cross-share sentinels (it currently returns "invalid" for them):
`errMismatchedLength`→"shares differ in length", `errMismatchedHRP`→"mismatched type", `errMismatchedThreshold`→"mismatched threshold", `errMismatchedID`→"mismatched id", `errRepeatedIndex`→"repeated share", `errInsufficientShares`→"need more shares". (`errInvalidShareIndex` already maps to "bad share index".) Keep labels short for the 480×320 display.

### 4.2 B2 — share-collection flow (gui), branched from the existing confirm

Change `confirmCodex32Flow` to return an **action** instead of a bool, so the confirm screen can offer Recover when the entered string is a share:
```go
type codex32ConfirmAction int
const (
	codex32Back    codex32ConfirmAction = iota // Button1
	codex32Engrave                             // Button3
	codex32Recover                             // Button2 — shown ONLY when the string is a share (index != S)
)
func confirmCodex32Flow(ctx *Context, th *Colors, scan codex32.String) codex32ConfirmAction
```
- For an **unshared secret** (`Unshared`): show only Back / Engrave (as A1 does today).
- For a **share** (index ≠ S): show Back / **Recover** (Button2) / Engrave, and replace the dead-end "engraves THIS share, not a recovered seed" note with an actionable one ("Recover to reconstruct the secret from k shares").
- **Title branch (M1):** `confirmCodex32Flow` currently hardcodes the title `"Confirm Codex32 Share"`. Branch it on `Unshared` → `"Confirm Codex32 Secret"` (so the re-confirm of a *recovered* unshared secret isn't mistitled "Share").

New `recoverCodex32Flow(ctx, th, first codex32.String) (codex32.String, bool)`:
- Derive k from the first share, with a defensive guard (**R0 I2** — `Fields.Threshold` is only meaningful when `ThresholdKnown`; a `New`-valid share guarantees `ThresholdKnown==true` and `Threshold ∈ [2,9]`, but make it self-documenting):
  ```go
  f, _ := codex32.ParsePrefix(first.String())
  if !f.ThresholdKnown || f.Threshold < 2 { // unreachable for a New-valid share; defensive
  	return codex32.String{}, false
  }
  k := f.Threshold
  ```
  Seed the set with `first`.
- Loop collecting shares until `len(shares) == k` (exactly k — §8): each iteration shows a "Share i of k · id NAME" header (clone `inputWordsFlow`'s `layoutTitlef` counter) and calls `inputCodex32Flow` for the next share. After each entry:
  - the entered string must be `New`-valid (the keypad already gates the OK button on `New==nil`) **and** a share (index ≠ S); **reject a second unshared secret** via `ParsePrefix(cand).Unshared` with an explicit `ErrorScreen` message **"enter a share, not the secret"** (M4 — do NOT reuse the generic "bad share index" label, which would misdescribe it);
  - run `codex32.ConsistentShares(append(shares, cand))`; on error, show an `ErrorScreen` with `codex32.Describe(err)` (e.g. "mismatched id", "repeated share") and discard the candidate (stay on the same step);
  - on success, append.
- Allow **Back** to remove the last share / exit to the original share's confirm (return `(_, false)`).
- Once k collected: `secret, err := codex32.Interpolate(shares, 'S')` (defense-in-depth — should be nil after `ConsistentShares` + exactly k). On error, `ErrorScreen(Describe(err))` and return `(_, false)`. On success return `(secret, true)`.

### 4.3 B3 — recovery → engrave (Option A), in `engraveObjectFlow`

Replace the `case codex32.String:` body with a small loop so a recovered secret is re-confirmed before engraving:
```go
case codex32.String:
	return engraveCodex32(ctx, th, scan)
```
```go
func engraveCodex32(ctx *Context, th *Colors, scan codex32.String) bool {
	for {
		switch confirmCodex32Flow(ctx, th, scan) {
		case codex32Back:
			return true // recognized, user declined (not "Unknown format")
		case codex32Recover:
			secret, ok := recoverCodex32Flow(ctx, th, scan)
			if !ok {
				continue // back to the original share's confirm
			}
			scan = secret // recovered unshared secret; loop re-confirms it (Recover not offered for S)
			continue
		case codex32Engrave:
			id, _, _ := scan.Split()
			s := backup.SeedString{Title: id, Seed: scan.String(), Font: constant.Font}
			backupSeedStringFlow(ctx, th, s)
			return true
		}
	}
}
```
The recovered secret (index S) re-enters the same confirm, which now shows "Unshared secret (S) · id NAME" and offers only Engrave/Back — then engraves **verbatim** via the unchanged `backupSeedStringFlow`. **No change to `Split()`, the engrave path, or `Interpolate`.** (M5: `scan.Split()` here is used **only for `id`** — its threshold and index returns are discarded, so `Split()`'s threshold-0→1 remap is irrelevant on this path; this matches A1's existing engrave behavior exactly.)

## 5. Error handling / backstops

`codex32.New` gates each entered share (OK button). `ConsistentShares` gates the growing set (eager, per added share). `Interpolate` is the final authority at recovery (defense-in-depth). The recovered secret is re-confirmed before engraving. The whole flow is hand-typed on the air-gapped touchscreen (no RF) — same exposure as single-share entry; the m-format `ms1` secret string is unrelated and never involved.

## 6. Testing (host: `go test ./gui/... ./codex32/...`)

- **B1 (pure, highest value):** `codex32` table tests for `ConsistentShares` over BIP-93 vector share sets — a consistent set (e.g. vector-2 shares `MS12NAMEA…` + `MS12NAMEC…`) → nil; a set differing in id/threshold/length → the matching sentinel; a repeated index → `errRepeatedIndex`; 0/1-share sets → nil. `Describe` returns the new labels for each cross-share sentinel.
- **B2/B3 (gui, `runUI`+`ExtractText`+`uiContains`):** drive `recoverCodex32Flow` (or `engraveObjectFlow`) with vector-2 share A as the first share + `runes(shareC)` + accepts: assert the collector renders "Share 2 of 2", that a mismatched candidate (e.g. a different-id share) surfaces "mismatched id", and that recovery yields the secret — assert the post-recovery confirm shows "Unshared secret" (the recovered `MS12NAMES…`). Assert that an unshared secret entered first shows **no** Recover option. Keep `TestInputSeedCodex32`/`TestWordKeyboardScreen` + all A1 tests green.
  - **Multi-screen driving (M3):** these flows span several screens, so the test advances `frame()` step-by-step (queue the runes/clicks for the next screen, call `frame()` to render+assert, repeat) — not the single "queue everything then one `frame()`" shape. Where a step pre-queues all input (as `TestInputSeedCodex32` does), document that the assertion is on the terminal frame.
  - **Long-code engrave (M2):** a recovered secret can be a long code (125–127 chars, 256-bit, threshold ≥ 2). The existing `backupSeedStringFlow`/`EngraveSeedString` path handles it (math checked: `ngroups=⌈127/10⌉=13 ≤ maxCol1=16`), but there is no existing golden engrave test for a long codex32 string. Add a `backup`-level test (or a gui assertion) engraving a 127-char codex32 secret to close the gap — recommended, not a GREEN blocker.

## 7. Versioning / commits

Firmware version is `-ldflags`-injected (no source bump). Commits on `feat/codex32-multishare-recovery` (off `bf7f811`), signed (SSH) + DCO, author Brian Goss. Fork-side; no upstream PR. Stage explicit paths.

## 8. Resolved decisions

- **Engrave artifact = Option A** (recovered codex32 secret, verbatim; reuses the existing engrave path) — user-chosen. Option B (BIP-39 mnemonic/SeedQR) **deferred & caveated**: codex32/BIP-93 backs up a BIP-32 master seed, not generally invertible to a BIP-39 mnemonic — Option B is only correct if the secret stores BIP-39 entropy, so it is a separate, larger, risk-bearing follow-on.
- **Entry point = branch the existing CODEX32 flow** (offer Recover from the confirm when the string is a share) — user-chosen; most discoverable, reuses A1's warning hook.
- **Exactly k shares** (BIP-93 canonical recover; the fork's `Interpolate` permits ≥k but does not cross-validate extras, so accepting exactly k is the conservative choice).
- **Eager per-share validation** via `ConsistentShares` + `Describe` labels (best UX; surfaces mismatches as they happen, not only at recovery).
- **`Interpolate` left untouched** (no crypto refactor); `ConsistentShares` is a standalone checker (R0 may suggest extracting Interpolate's pass-1 into it — default: keep both, accept minor duplication).

## 9. Process note

Per project standard: this spec MUST pass the opus-architect **R0 gate to 0C/0I before any code** (fold → persist verbatim to `design/agent-reports/` → re-dispatch until GREEN). Then plan → plan R0 → single-implementer subagent TDD in a worktree → mandatory whole-diff adversarial execution review. Proceeding autonomously (user directive) — the spec-review user gate is folded into the autonomous run; the architect gates + execution review are the quality controls.
