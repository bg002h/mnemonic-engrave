# Cycle A: codex32-input-polish — PRE-SPEC design review — R0

- **Stage:** pre-spec architect design gate (user-requested, before the spec is written). Feeds the spec; the formal spec/plan R0 loop still follows.
- **Date:** 2026-06-17
- **Reviewer:** opus `feature-dev:code-architect` (read-only; files read on fork main `599ec9a`; BIP-93 re-verified vs raw.githubusercontent.com/bitcoin/bips/master/bip-0093.mediawiki).
- **Design reviewed:** Cycle A = Task 0 (merge Slice 1 into fork main) + C1 (codex32 API: error classify + ParsePrefix) + C2 (error-class feedback) + C3 (char counter + live field parse) + C4 (pre-engrave confirm) + C5 (keyboard tidy: full-QWERTY + dim b/i/o). Multi-share recovery = separate Cycle B, out of scope.
- **Verdict:** **DECOMPOSE, then proceed to spec. No blockers.** Two must-fix-before-spec items (Task-0 conflict enumeration; C3 length facts) + several refinements.

---

## VERBATIM REVIEW OUTPUT

### CRITICAL

**CRIT-1 — Task 0 merge: the codex32-test flip is NOT the only break. `TestWordKeyboardScreen` accepts on Button2 too.** On fork main there are THREE `okBtn := &Clickable{Button: Button2}` (`gui.go:543` inputWordsFlow, `:628` inputCodex32Flow, `:688` inputSLIP39Flow) and TWO host tests clicking Button2 to accept: `gui/codex32_input_test.go:31` (PR #34, fork-main-only) and `gui/gui_test.go:281` `TestWordKeyboardScreen` (driving inputWordsFlow). Slice 1 moves all three flows to Button3. `TestWordKeyboardScreen` exists on both branches — IF Slice 1 flipped it (it must have, to be green), the merge takes Slice 1's version cleanly; **confirm, don't assume.** `codex32_input_test.go` is fork-main-only → the Button2→Button3 flip at `:31` is the guaranteed manual fix. **Fix:** Task 0's acceptance must enumerate the post-merge test surface via an actual `git merge-tree`, not hand-wave "only the flip."

**CRIT-2 — Other plausible merge conflicts: overlapping `gui.go` regions + the new `bip39.LastWordCandidates`.** `bip39.LastWordCandidates`/`completeCandidateWord`/`updateValidCandidateKeys` are absent on fork main (Slice-1-only; `bip39.go` exports only `LabelFor:79`,`ClosestWord:95`). Slice 1 adds the new bip39 func (additive, low risk) + rewires `inputWordsFlow` body (`:539-621`) — the same neighborhood as `inputCodex32Flow` (`:623-682`). Possible textual conflict at the inputWordsFlow/inputCodex32Flow boundary; budget for hand-resolution keeping Slice 1's candidate logic AND fork main's codex32 flow. **Merging Slice 1 IS the right call** (the keyboard already commits on Button3 on fork main `:1009`/`:952`; only the okBtn assignments + the BIP-39 candidate path are missing; branching without it would force re-applying Button3 + a future conflict).

### IMPORTANT

**IMP-1 — C3 length facts are wrong; the readout would mislead.** Verified vs BIP-93: Regular codex32 payload ≤74, checksum 13, data-part max 93; 128-bit total=48, 256-bit total=74, 368-bit total=96. Long payload 75–103, checksum 15, data-part 96–124, total (with `ms1`) 99–127; 512-bit vector total=127. "A data part of 94 or 95 characters is never legal." The design's "(48 / up to 93 short; 125-127 long)" conflates data-part vs total. Firmware gates on TOTAL: short 48–93 (`shortCodeMinLength`/`MaxLength`, `codex32.go:41-42`), long 125–127 (`longCode…`, `:43-44`) — the firmware's long window (125–127) is far narrower than BIP-93's 99–127, and 94–124 is a dead zone `New` rejects with `errInvalidLength` (`:99-107`). **There is no single "expected" length.** **Fix:** C3 must use a WINDOW model driven by the package's real constants (expose them publicly): `<48` → "N chars" (no denominator); 48–93 → short window; 94–124 → "keep typing" (NOT an error); 125–127 → long window; `>127` → over-length. Widening the long gate to BIP-93 is OUT OF SCOPE (safe over-rejection; needs its own vectors).

**IMP-2 — C1: prefer `Describe(err) string` over exporting all sentinels; `ParsePrefix` is a fresh fail-soft parser, not a `partsInner` refactor.** Exporting all 14 `errInvalid*`/`errMismatched*` sentinels (`codex32.go:24-37`) freezes a large internal vocabulary (several are Interpolate-only). Cleaner: `func Describe(err) string` (or `Classify→FailureKind`) mapping wrapped errors to UI strings (`New` wraps with `%w`, so `Describe` can `errors.Is` internally); sentinels stay private. If `errors.Is` branching is wanted, export only the single-share subset (`ErrInvalidChecksum/Length/Case/Character/Threshold/ShareIndex/IncompleteGroup`). `ParsePrefix` CANNOT reuse `partsInner` (`:127`) — it indexes unconditionally + `panic("unreacable"):159`. Write fresh, using `splitHRP` (`:453`, returns `("",p1)` if no `1`) + the non-panicking `feFromRune` (`gf32.go:126`). Fields determinable at fixed data-part offsets: HRP (on `1`), threshold=`res[0]` (len≥1, validate ∈{0,2-9}, reject 1), id=`res[1:5]` (len≥5), shareIdx=`res[5]` (len≥6, enforce threshold0⇒S); payload/checksum NOT separable mid-entry (checksum length depends on final total). **Signature:** `(Fields, error)` — `error` non-nil only for *determinable* violations; too-short returns `(partial, nil)`. So C2 and C3 share one call/keystroke. Mixed-case is moot via the keypad (force-uppercases `:1036`) but detect it anyway for API honesty.

**IMP-3 — C4: `Split()` will mislead the confirm screen.** `Split()` (`:394-401`) does `if t==0 {t=1}` — so an unshared secret (threshold `0`, index `S`) reports threshold **1**, which BIP-93 forbids as an entered digit. A confirm screen reading "threshold 1, index S" misrepresents an unshared secret. **Fix:** branch on the RAW index — index `S`(==`feS`) ⇔ unshared secret (BIP-93 + `partsInner:169-171`). If index==S: show "Unshared secret (S) · id XXXX" (no threshold number); else "Share <idx> · id XXXX · part of k-of-n" (k informational; engraving one share ≠ recovery — that's Cycle B). Get the un-remapped threshold + raw index via the new `ParsePrefix`, NOT `Split()`. Do NOT change `Split()`'s remap (the engrave-title path `:1725` uses only `id`, unaffected; changing Split is out-of-scope risk).

### MINOR

**MIN-1 — C5 dimming: per-instance (no cross-contamination ✓) but `updateValidKeys` is BIP-39-only.** Each flow has its own `NewKeyboard`/`[]keyboardKey` (`:540`/`:626`/`:685`), so dimming b/i/o on the codex32 keyboard can't affect BIP-39. BUT `updateValidKeys` (`:921-930`) is wordlist-driven and assumes lowercase `a..z` (`idx := key.r-'a'`); codex32 keys are uppercased + include digits. So C5 must use STATIC dimming (set `disabled=true` once at construction on b/i/o, after the `NewKeyboard` loop `:808-817`), not `updateValidKeys`. Note the current codex32 alphabet (`:624`) OMITS b/i/o entirely — C5 is a deliberate layout change (add them present-but-dimmed for familiarity). `Valid()`/`adjust()` already skip disabled keys. Keep every `Alphabet` char (`gf32.go:21`) enabled; keep `1`.

**MIN-2 — C2 timing: pin to ParsePrefix determinability.** Show field errors eagerly (bad threshold len≥1; threshold0-without-S len≥6) but WITHHOLD the checksum verdict until inside an accept window — `New` returns `errInvalidLength` (not checksum) for 1–47 and 94–124, so "bad checksum" on a half-typed string is wrong. Map: `errInvalidLength` → suppress (show C3 "keep typing"); only `errInvalidChecksum` after a full valid-length string is a true error. (`New` checks length first `:99-107`.)

**MIN-3 — per-keystroke double-parse.** `New` is already called per keystroke (`:633`); adding `ParsePrefix` doubles it. Negligible (µs on RP2350), but call both once per `kbd.Update` iteration and thread results, not redundantly in the layout block. No resource concern (not the go-slip39 situation).

### Scope / decomposition
**Split into two specs along the merge boundary.** Spec A0 — "Adopt Slice 1 base" (Task 0 alone): merge, resolve enumerated conflicts (CRIT-1/2), flip `codex32_input_test.go:31`, confirm `go test ./...` green, branch `feat/codex32-input-polish`. A base-state change touching the only test-covered flow → own green gate first. Spec A1 — "codex32 input polish" (C1–C5): C1 (API) → C2/C3 (consume C1) → C4 (consume C1) → C5 (independent). C1 is the linchpin/only crypto-pkg change; land + host-test first. ~150–250 LoC + tests. Cycle B (multi-share) stays out.

### Host-testability
C1: pure `codex32/` table tests over BIP-93 vector prefixes (highest value). C2/C3: **need a test seam** — the harness drives input (`runes`/`click`) but has NO text-readback; expose the status string from `inputCodex32Flow` (test hook) or assert via frame buffer, else C2/C3 are untestable. C4: drive to confirm screen, assert "Unshared secret" (vector 1 `ms10test…`) vs "Share N" (a `ms12…` share). C5: assert b/i/o `disabled==true` + every Alphabet char enabled + a BIP-39-keyboard regression (b/i/o enabled there). Keep `TestWordKeyboardScreen` green.

### Risks/gaps
(1) Trial-merge MUST be run during spec-writing (conflict surface: `gui.go` inputWordsFlow/inputCodex32Flow boundary, `gui_test.go:281` verify, `codex32_input_test.go:31` manual flip, additive `bip39.go`). (2) C1 API is a one-way door — settle `Describe` vs sentinels (recommend `Describe` + length consts). (3) C3 length facts corrected. (4) C4 Split trap. (5) plate-fit for long codes unverified (orthogonal; QA item). (6) keyboard shared — every touch keeps `TestWordKeyboardScreen` green; on-device QA pending (no hardware). (7) md1/mk1 (`mdmk.go`, `mdmkFlow:1786`) share the GF(32) engine but are a different scheme — keep `ParsePrefix`/`Describe` codex32-only, off mdmk.

### Recommendation
Decompose into Spec A0 (merge) + Spec A1 (C1–C5), then proceed to spec. Must-fix before the spec: (1) run `git merge-tree` + enumerate post-merge fixes; (2) correct C3 to the verified window model + route C4 through `ParsePrefix` not `Split()`. Settle C1 API (recommend `Describe` + exported length consts).

---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | DECOMPOSE, proceed-to-spec (2C/3I/3m, no blockers) | CRIT-1/2 Task-0 merge conflict surface (run trial-merge); IMP-1 C3 length window-model (BIP-93-verified); IMP-2 C1 `Describe`+fresh `ParsePrefix(Fields,error)`; IMP-3 C4 raw-index-not-Split; MIN C5 static-dim, C2 timing, double-parse. Split A0(merge)+A1(C1-C5). |

NOTE: pre-spec design consult. Findings fold into the spec(s); the formal spec R0 loop (→0C/0I) still follows. Decomposition (A0+A1) + C1 API shape are reflected to the user.
