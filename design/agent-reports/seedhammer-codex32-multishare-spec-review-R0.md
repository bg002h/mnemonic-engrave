# Cycle B: codex32-multishare-recovery — SPEC R0 GATE REVIEW — R0

- **Stage:** mandatory spec R0 gate (0C/0I before any code).
- **Spec reviewed:** `design/SPEC_seedhammer_codex32_multishare_recovery.md` (committed `5c4bc16`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `af72debd47678b45b`), read-only vs fork `main` `bf7f811` + BIP-93.
- **Outcome:** **NOT GREEN — 2 Important (I1, I2) + 5 Minor.** All folded (see Loop summary); re-dispatched R1.

> NOTE: verbatim architect output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

# R0 Spec Review — SPEC_seedhammer_codex32_multishare_recovery.md
**Reviewer:** opus architect (adversarial read-only)
**Base:** fork `main` @ `bf7f811`, BIP-93 text verified
**Verdict summary** at bottom.

---

## Verification findings by question

### Q1: API reality

**`Interpolate` signature** — confirmed `codex32.go:188`: `func Interpolate(shares []String, index rune) (String, error)`. Spec citation accurate.

**Sentinel locations** — confirmed at `codex32.go:24-37`. Spec's table in §3 is accurate: `errMismatchedLength:202`, `errMismatchedHRP:205`, `errMismatchedThreshold:208`, `errMismatchedID:211` (pass-1, loop over all shares), `errInsufficientShares:191,229` (zero-shares guard at :191; count check at :229), `errRepeatedIndex:245` (mid-loop). One nuance: the count check at :228 reads `s0Parts.threshold > len(shares)`, which fires only for insufficient shares — at exactly k it does NOT fire. Confirmed.

**Check order** — CONFIRMED: the four field-mismatches fire in pass-1 (lines 199-214) before the count check (:228). The repeated-index check fires mid-Lagrange (:245), after the count check. This is exactly the ordering the spec asserts.

**`ParsePrefix(first.String()).Threshold`** — `polish.go:92-94`: stores the raw digit value (`int(data[0] - '0')`). For a `New`-valid share (index ≠ S), threshold is 2..9 (threshold-0 forces index=S per `partsInner:169-171`). `ThresholdKnown` is always true for a `New`-valid string. So `ParsePrefix(first.String()).Threshold` directly gives k. **Correct.**

**`String.parts()` access for `ConsistentShares`** — `parts()` is unexported (`codex32.go:176`). `ConsistentShares` proposed in `codex32/polish.go` (same `codex32` package) can call `s.parts()` directly. `Describe` already uses unexported sentinels via `errors.Is`. **Architecture is sound.**

**`Describe` gap confirmed** — `polish.go:26-47`: currently maps only `errInvalidChecksum`, `errInvalidLength`, `errInvalidCharacter`, `errInvalidCase`, `errInvalidThreshold`, `errInvalidShareIndex`, `errIncompleteGroup`. None of the six cross-share sentinels (`errMismatched*`, `errRepeatedIndex`, `errInsufficientShares`) are mapped — they fall through to `"invalid"`. B1 must fix this. **Spec is accurate.**

---

### Q2: Eager-validation gap

**Can partial sets (< k shares) be validated without `errInsufficientShares` firing?** — YES, confirmed. `errInsufficientShares` fires at :191 (zero-shares) and :229 (`threshold > len(shares)`). `ConsistentShares` as designed does NOT call `Interpolate`; it makes its own comparisons over `parts()`. It omits the count check. A set of 1 share (just the first) is consistent with itself (vacuously). **The eager-validation gap the spec relies on is real and `ConsistentShares` correctly avoids it.**

**Four field-mismatch + distinct-index** — the four checks (`len(s.s)`, `.hrp`, `.threshold`, `.id`) mirror exactly what `Interpolate` does in pass-1 at :199-214. The repeated-index check must compare `parts().shareIdx` values pairwise (O(n²) for n up to 9 — trivial). This is cleanly implementable within the package. **Confirmed.**

**Does `ConsistentShares` need shares to be `New`-valid first?** — Yes, `parts()` panics on invalid input (`codex32.go:179-182`). The keypad gates the OK button on `New==nil` (`gui.go:688`), and `recoverCodex32Flow` receives shares only after they pass `New`. **The precondition is guaranteed by the calling context; the spec should state it explicitly as a contract (currently implied, not stated).**

---

### Q3: BIP-93 correctness

**"Exactly k shares, distinct index, same threshold/id/length"** — verbatim from BIP-93: "The number of shares is exactly equal to the (common) threshold value." Vector 2 uses two shares (`MS12NAMEA...`, `MS12NAMEC...`) with threshold=2, k=2. `TestBIPVector2` (`codex32_test.go:56-78`) confirms `Interpolate(shares,'S')` → `MS12NAMES...` → `d1808e09...`. **BIP-93 conformance confirmed.**

**Option A (engrave recovered `S` string verbatim) is a correct backup** — the unshared secret in codex32 form IS the canonical recovery artifact. `Interpolate(shares,'S')` produces a `codex32.String` that encodes the exact same master seed bytes as the original secret. Engraving it verbatim gives a single-plate backup that can be loaded directly into any codex32-aware wallet. **Correct and complete for its stated purpose.**

**>k shares caveat** — the spec states "exactly k" and documents the rationale (BIP-93 canonical; fork accepts ≥k but does not cross-validate extras). This is correctly resolved. The spec's decision to stop at exactly k is the right conservative choice and is stated in §8.

---

### Q4: GUI integration safety

**`confirmCodex32Flow` return-type change: callers** — exact callers found:
- `gui.go:1842`: `if !confirmCodex32Flow(ctx, th, scan)` — this is the PRODUCTION caller that must change to `engraveCodex32(...)`.
- `codex32_polish_test.go:114`: `runUI(ctx, func() { confirmCodex32Flow(ctx, &descriptorTheme, s) })` — return value discarded.
- `codex32_polish_test.go:134`: same pattern — return value discarded.

Changing `confirmCodex32Flow` from `bool` to `codex32ConfirmAction` (an `int`) will compile fine because the test callers discard the return. The production caller at `gui.go:1842` is replaced by `engraveCodex32`. **The change is surgically safe, but the spec omits explicitly confirming the test callers at `codex32_polish_test.go:114,134` are already return-value-agnostic. This is a documentation gap but not a blocker.**

**Button2 is free for "Recover"** — confirmed. A1's `confirmCodex32Flow` uses only Button1 (Back) and Button3 (Engrave + Center alt). Button2 is unused on that screen. The `layoutNavigation` function places Button2 at the middle Y position. `SeedScreen.Confirm` uses Button2 for edit (`gui.go:2045`), proving the slot works. **Button2 is available and functional.**

**`engraveCodex32` loop terminates on recovered secret** — the recovered secret has threshold=0, index=S. `ParsePrefix` returns `Unshared=true`. `confirmCodex32Flow` for an unshared secret shows only Back/Engrave (no Recover). `codex32Recover` is never returned. **Loop terminates correctly.**

**Recovered secret routes through `backupSeedStringFlow` unchanged** — `backupSeedStringFlow` (`gui.go:1956`) calls `backup.EngraveSeedString` then `toPlate`. `EngraveSeedString` calls `qr.Encode(strings.ToUpper(seed), qr.M)` — works for any codex32 string. The layout in `backup.go:95-158` uses `ngroups = (len(seed)+9)/10`. For a 127-char long-code recovered secret, `ngroups=13`, well within `maxCol1=16`. **No plate overflow. Existing path handles all codex32 length classes.**

**`return true` on Back** — `case codex32Back: return true` in `engraveCodex32`. Preserves the A1 lesson: recognized format, user declined, NOT "unknown format." **Correct.**

---

### Q5: Testability

**`runUI`+`ExtractText`+`uiContains`** — confirmed real and working in existing tests (`codex32_polish_test.go`, `gui_test.go`). `runUI` (`gui_test.go:466`) uses `iter.Pull` to drive a coroutine; `frame()` advances one frame; `uiContains` does case-insensitive substring match. **Multi-screen flows are testable: queue all runes+clicks upfront, then call `frame()` at each step to advance and assert.**

**BIP-93 vector-2 shares usable** — `codex32_test.go:57-59`: `MS12NAMEA320ZYXWVUTSRQPNMLKJHGFEDCAXRPP870HKKQRM` and `MS12NAMECACDEFGHJKLMNPQRSTUVWXYZ023FTR2GDZMPY6PN` are present and `New`-valid. Threshold=2, identifier=NAME, expected secret=`MS12NAMES6XQGUZTTXKEQNJSJZV4JV3NZ5K3KWGSPHUH6EVW`. These are the exact vectors the spec references. **Directly usable in B1 and B2/B3 tests.**

**Pre-queuing runes+clicks** — confirmed by `TestInputSeedCodex32` (`codex32_input_test.go:21-45`) which queues all input before calling `newInputFlow`. Same pattern works for multi-screen `recoverCodex32Flow`. **Spec's testing claims are achievable.**

---

### Q6: Scope discipline

**`Interpolate`/`Split()`/`mdmk.go` untouched** — spec explicitly lists these as out-of-scope. No design element requires touching them. `mdmk.go` has zero `Interpolate` references (recon confirmed). **Scope is clean.**

**Long-code (125-127 char) plate-fit** — verified above: `ngroups=13` for 127 chars, `maxCol1=16`. `EngraveSeedString` has no explicit length gate; it returns an error only if QR encoding fails. A 127-char uppercase codex32 string is QR-encodable at error-correction M. The `backup_test.go` covers `ms10leets...` (74 chars) but not a full-length long code. **No plate overflow, but no explicit long-code engrave regression test exists for `backupSeedStringFlow`. The spec should note this as a test gap (not a blocker — the math works, but a golden test for long-code engrave would be prudent).**

**Spec §2 Out-of-scope sentence** — "the recovered secret is a codex32 string of the same length class the single-share path already engraves" — this assertion needs qualification. A recovered secret from k shares is the SAME LENGTH as the input shares (Interpolate preserves length). The shares entered by the user are at most `LongCodeMaxLength=127` (the keypad guards at `shortCodeMaxLength`+length-window check). Since the recovered secret has the same total string length as the input shares, it passes through the same engrave path without issue. **The claim is correct; it could be stated more precisely (same length as inputs, not same length class as single-share path in general).**

---

### Q7: ConsistentShares-vs-extract-Interpolate decision

The spec's default — standalone `ConsistentShares`, leave `Interpolate` untouched — is the correct call. **Reasoning:**

The duplication is six comparisons (≤15 LoC). `Interpolate` is the proved-correct crypto path exercised by multiple BIP-93 test vectors; touching it introduces regression risk for zero functional gain. A refactor extracting `Interpolate`'s pass-1 into `ConsistentShares` would require `Interpolate` to call `ConsistentShares` as a precondition, which would change the error-return order (currently field-mismatch errors surface from pass-1 before the count check; `ConsistentShares` would surface them first, same order, so this is acceptable) but adds an abstraction boundary that makes the crypto function depend on a UI-support helper. The spec's rationale ("keep the proven crypto path intact, accept the small duplication") is the right engineering judgment for a TinyGo firmware where the diff must be minimal and the crypto unchanged.

**Recommendation: keep the spec's default. Do not extract.** The duplication is intentional defense-in-depth and the right tradeoff.

---

## Findings

### CRITICAL

None.

---

### IMPORTANT

**I1 — `ConsistentShares` precondition is unstated (contract gap)**

The spec's description of `ConsistentShares` (`§4.1(a)`) states "Implemented over `parts()`" but does not specify that each `String` in the input slice must be `New`-valid. `parts()` panics on an invalid `String` (`codex32.go:179-182`). If called with an invalid string (e.g., one not gated by `New`), it panics rather than returning an error. This is a contract that the caller (the GUI) must uphold, and the spec's API doc must state it explicitly.

The call site in `recoverCodex32Flow` does uphold it (keypad gates on `New==nil`), but the package-level docstring for `ConsistentShares` should state: "All shares must be New-valid; passing an invalid String panics." Omitting this creates a maintenance hazard — a future caller that skips the `New` gate will get a panic rather than an error return. Since `ConsistentShares` is an exported function, this is a public API contract that must be documented.

**Fix:** add to the `ConsistentShares` godoc: "Each share must have been successfully validated by New (i.e., s.String() must pass New without error); passing an unvalidated String panics."

---

**I2 — `ThresholdKnown` guard missing from `k` derivation**

The spec code uses `k := ParsePrefix(first.String()).Threshold` without asserting `ThresholdKnown`. For a `New`-valid share, `ThresholdKnown` is always true (full string present, threshold digit parseable). However, `Fields.Threshold` is documented as "valid only if ThresholdKnown" (`polish.go:53`), and `ParsePrefix` returns `Threshold=0` as its zero-value when `ThresholdKnown=false`. If `ThresholdKnown` were false (impossible for `New`-valid, but the spec doesn't argue this), `k=0` would cause the collection loop to be skipped entirely, producing a zero-share set passed to `Interpolate`, which would return `errInsufficientShares`.

The spec should either: (a) explicitly state the precondition "first is New-valid, therefore ThresholdKnown is always true," or (b) add a defensive check `if !f.ThresholdKnown { return _, false }` at the start of `recoverCodex32Flow`. Option (b) is safer even if unreachable in practice, because the public API of `ParsePrefix` does not guarantee `ThresholdKnown` for all inputs.

**Fix:** in `recoverCodex32Flow` spec pseudocode, change to:
```go
f, _ := codex32.ParsePrefix(first.String())
// first is New-valid, so ThresholdKnown is always true and Threshold ∈ [2,9].
k := f.Threshold
```
with a panic-or-return guard: `if !f.ThresholdKnown || f.Threshold < 2 { return String{}, false }`.

---

### MINOR

**M1 — Confirm-screen title says "Share" for recovered unshared secret**

`confirmCodex32Flow` (`codex32_polish.go:99`) hardcodes `"Confirm Codex32 Share"` as the screen title. When `engraveCodex32` loops back with the recovered secret (unshared, index=S), the body correctly shows "Unshared secret (S)" but the title still says "Confirm Codex32 Share." The spec doesn't address this. The title should branch on `f.Unshared`, e.g., `"Confirm Codex32 Secret"` vs `"Confirm Codex32 Share"`. This is a cosmetic UX inconsistency the implementer should fix in B3.

---

**M2 — No explicit test for long-code (125-127 char) engrave via `backupSeedStringFlow`**

The spec's scope says plate-fit is handled by the existing path, which is mathematically correct. However, there is no existing golden test for `EngraveSeedString` with a 125-127 char string (BIP-93 vector 5, `MS100C8VSM...`, is exercised in `codex32_test.go` for decode but not in `backup_test.go` for engraving). If a recovered secret happens to be a long-code (256-bit seed, threshold ≥ 2), it will take the long-code path. The spec should note that a `TestCodex32` entry for the long-code case in `backup_test.go` would close this gap, even if it's not a requirement for GREEN.

---

**M3 — Test description over-promises on "Back removes last share"**

`§6` says "Assert ... Back to remove the last share." The spec's `recoverCodex32Flow` design says Back at share-i goes to share (i-1) or exits. This is a natural interaction pattern but is NOT trivially testable with the pre-queue model: testing the "remove last" path requires queuing a share entry, then a Back click, and asserting the counter resets. This is achievable but requires more careful test construction than "queue all runes then call frame." The spec should note this test requires a two-phase frame sequence: advance past the first `frame()` call to get the "Share 2 of 2" state, click Back, then advance again to assert return to "Share 1 of 2" (or exit). Not a correctness issue but the test spec should be more precise.

---

**M4 — Spec §4.2 describes `recoverCodex32Flow` rejecting "a second unshared secret" without specifying the error message**

The validation logic says: "the entered string must be ... a share (index ≠ S) — reject a second unshared secret." The spec does not specify what `ErrorScreen` message to show for this case. `Describe` maps `errInvalidShareIndex` to `"bad share index"` — but that's not user-friendly for "you entered an unshared secret, not a share." A dedicated message (e.g., `"unshared secret not a share"`) should be specified. Currently the spec would have the implementer decide ad hoc.

---

**M5 — `ParsePrefix`'s `Unshared` semantics on the confirm screen use `ParsePrefix`, not `Split()` — spec correctly notes this but the `engraveCodex32` pseudocode uses `scan.Split()` for the title**

In the `case codex32Engrave:` branch of the spec's `engraveCodex32`, `id, _, _ := scan.Split()` is used for the `backup.SeedString` title. `Split()` (`codex32.go:394-401`) remaps threshold-0 to threshold-1. This is irrelevant because only the `id` return is used. No correctness issue, but the spec should note that `Split()` is used only for `id` (the threshold and index returns are discarded). This avoids implementer confusion about the threshold-remapping subtlety.

---

## Verdict

**NOT GREEN — 2 Important / 3 Minor open.**

**Required before implementation (resolve I1 and I2):**

- **I1:** Add explicit precondition to `ConsistentShares` godoc: shares must be `New`-valid; passing an unvalidated `String` panics.
- **I2:** Either state explicitly that `first` being `New`-valid guarantees `ThresholdKnown=true` and `Threshold ∈ [2,9]`, OR add a defensive check in `recoverCodex32Flow` for `!f.ThresholdKnown || f.Threshold < 2`. The latter is preferred because it makes the code self-documenting.

**Minor items (M1–M5):** address at implementer's discretion. M1 (confirm-screen title) and M4 (rejection message for unshared-secret entry) have UX impact and should be addressed in the same cycle. M2, M3, M5 are documentation/test gaps without correctness impact.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | NOT GREEN — 0C / 2I / 5m | **I1** `ConsistentShares` precondition unstated (`parts()` panics on non-New-valid input) → folded: godoc precondition added (§4.1a). **I2** `k` derivation missing `ThresholdKnown`/`Threshold≥2` guard → folded: defensive guard added to `recoverCodex32Flow` (§4.2). **M1** confirm title "Share" mistitles recovered secret → folded: branch title on `Unshared` (§4.2). **M4** unshared-secret-as-share rejection had no message → folded: explicit "enter a share, not the secret" (§4.2). **M5** `Split()`-only-for-`id` note → folded (§4.3). **M2** no long-code engrave test → folded: recommended test added (§6). **M3** multi-screen test is step-by-step `frame()` not single-shot → folded: §6 driving note. Architect confirmed all API/BIP-93/integration claims accurate (Interpolate sig+sentinels+order, ParsePrefix→k, Button2 free for Recover, engraveCodex32 loop terminates on the unshared recovered secret, backupSeedStringFlow handles long codes, vector-2 shares usable); endorsed keeping `Interpolate` untouched (standalone `ConsistentShares`, accept ~15-LoC duplication). |

Re-dispatched R1 after the fold.
