# Cycle C: slip39-enablement (Tier 1) — SPEC R0 GATE REVIEW — R1

- **Stage:** spec gate round R1 (re-review after folding R0's 2 Critical + 3 Important).
- **Spec reviewed:** `design/SPEC_seedhammer_slip39_enablement.md` (committed `9a2930c`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `ad7a8b5b978a4fbab`), read-only vs fork `main` `9b0a02c` + SLIP-0039.
- **Outcome:** **RED — 1 Critical (C1: wrong anchor-vector Identifier 7945 vs actual 10027) + 4 Minor.** Folded; re-dispatched R2.

> NOTE: verbatim architect output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

**R1 REVIEW — Cycle C, Tier 1 SLIP-39, Spec Gate**

**Reviewer:** Opus architect (R1 — post-fold verification)
**Date:** 2026-06-18
**Spec:** `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_seedhammer_slip39_enablement.md` (post-R0 fold)
**Fork base verified:** `/scratch/code/shibboleth/seedhammer` @ `main` `9b0a02c`

---

## CRITICAL

**C1 — §6 Anchor Vector: `Identifier==7945` assertion is factually wrong**

`spec:141` ("Concrete anchor vector (R0 M2): the existing `backup_test.go` SLIP-39 fixture share (id **7945** ... → assert `Identifier==7945`")

The `backup_test.go:215` value `"7945 #1 1/1"` is a **hardcoded display string** supplied directly to `backup.Seed{Title: "7945 #1 1/1"}` — it is not derived by decoding the mnemonic. The test (`TestSLIP39`) calls `EngraveSeed` for a golden-image layout test; it never calls `ParseShare` or decodes any field. The actual SLIP-39 identifier encoded in the "duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision keyboard" mnemonic is **10027**, computed as follows:

- Word 0 "DUCKLING": index 313 (canonical SLIP-39 wordlist, 0-based; confirmed via `trezor/python-shamir-mnemonic`)
- Word 1 "ENLARGE": index 360
- id_exp packed integer = 313 × 1024 + 360 = 320872
- identifier = 320872 >> (EXTENDABLE_FLAG_LENGTH_BITS + ITERATION_EXP_LENGTH_BITS) = 320872 >> 5 = 10027

An implementer following §6 literally will write `assert Identifier==7945` in their `ParseShare` test, which will fail when `ParseShare` correctly returns 10027. Alternatively, an implementer who reverse-engineers the test to pass will produce a broken `ParseShare` implementation. Either outcome is a defect.

**Required fix:** Replace the asserted value. Either:
(a) Correct the value in §6 to `Identifier==10027` (with the derivation shown above for verifiability), OR
(b) Replace this anchor vector entirely with a mnemonic from `vectors.json` whose identifier is independently computable and explicitly stated — e.g., pick the first 20-word entry from `vectors.json` (the "Valid mnemonic without sharing (128 bits)" entry is exactly this mnemonic, so option (a) applies directly), and precompute the expected identifier from the bits per the SLIP-0039 reference decoder.

Note: GroupThreshold=1, GroupCount=1, MemberIndex=0, MemberThreshold=1 for this mnemonic are correct (confirmed by bit decode of words 2-4, which are "ACADEMIC ACADEMIC AGENCY" = 0, 0, 23, encoding all-zero group/member fields → all stored values 0 → all decoded values 1/0 as appropriate for a 1-of-1 single-group share). Only the Identifier claim is wrong.

---

## IMPORTANT

None identified.

---

## MINOR

**m1 — §4.2: "add `"fmt"` (+ `"strings"`)" — `"strings"` is already imported**

`spec:73` says "add `"fmt"` (+ `"strings"`) to `gui/gui.go`'s import block". The real `gui.go:14` already imports `"strings"`. Only `"fmt"` needs to be added. The parenthetical `(+ "strings")` is misleading and could confuse the implementer into adding a duplicate import (which will cause a compile error). Fix: drop the `(+ "strings")` parenthetical or change it to "(`"strings"` is already present)."

**m2 — §4.2: `showError` function name does not exist; real function is `showCodex32Error`**

`spec:88` references `showError(ctx, th, slip39words.Describe(err))` and `spec:115` `showError(ctx, th, "too large")`. The existing function in `codex32_polish.go:145` is named `showCodex32Error(ctx *Context, th *Colors, msg string)`. The spec says "reuse Cycle-B's `showCodex32Error` pattern, or a shared `showError(ctx, th, msg)`" — so it acknowledges the ambiguity. The implementer must create a `showError` wrapper or reuse `showCodex32Error` directly with an appropriate title string. This is underspecified but not a blocking error given the "or a shared" hedge. Note that `showCodex32Error` hard-codes the title as `"Invalid share"` — the implementer will likely need a more general version or to call it directly with the appropriate SLIP-39 context. Worth a clarifying note in the plan.

**m3 — §4.1: `ParseShare`'s exact-lookup requires uppercase normalization, not stated**

The `words` string in `slip39/wordlist.go:11` is all uppercase (e.g., "ACADEMIC..."). `LabelFor` returns uppercase strings. The `ParseShare(string)` input from the GUI will be space-joined uppercase words (via `LabelFor`), but the `vectors.json` mnemonics (used for testing) are lowercase. The spec says "exact wordlist lookup" but does not specify whether `ParseShare` normalizes the input to uppercase before lookup. The implementation must do `strings.ToUpper` on each word before lookup, or the official vector tests will fail (lowercase input). This should be stated explicitly in §4.1. Not Critical because the implementer is expected to notice it when the vector tests fail, but it should be pinned in the spec to avoid test-time confusion.

**m4 — §6: `backup_test.go` cited as "anchor vector" but never calls `ParseShare` — the citation framing is wrong regardless of the corrected value**

`spec:141`: even after correcting the Identifier value, the `backup_test.go` fixture is an engraving golden-image test, not a `ParseShare` decode test. Citing it as an "anchor vector" for `ParseShare` is a category error. The spec should say "use the mnemonic from `backup_test.go:204` as the test mnemonic for a new `slip39/share_test.go` anchor test" (not "the existing `backup_test.go` fixture" as if it already validates the fields). The implementer needs to write a NEW test in `slip39/share_test.go` that calls `ParseShare` on this mnemonic and asserts the fields — the `backup_test.go` does not do this and should not be described as if it does.

---

## Regression Scan

**Tier 1 scope exclusions (recovery, Shamir/Feistel/PBKDF2, passphrase):** explicitly stated throughout §2, §3, §5, §8. No drift found.

**`codex32`/`mdmk.go` untouched:** §2 explicitly states unchanged. No spec text touches these. Confirmed.

**`engraveObjectFlow` other cases unaffected:** the `case slip39words.Share:` addition inserts into the switch at `gui.go:1807`; the `case bip39.Mnemonic:`, `case codex32.String:`, `case *bip380.Descriptor:`, `case mdmkText:`, `default:` arms are not modified. The `default: return false` (→ `scanUnknownFormat`) remains intact for truly unrecognized objects.

**`inputSLIP39Flow` real signature matches spec:** real `gui.go:755` = `func inputSLIP39Flow(ctx *Context, th *Colors, mnemonic slip39words.Mnemonic, selected int) bool`. Spec §4.2 calls `inputSLIP39Flow(ctx, th, m, 0) bool`. Match confirmed.

**`showCodex32Error`/`ErrorScreen` pattern:** `ErrorScreen` type exists at `gui.go:198`. `showCodex32Error` exists at `codex32_polish.go:145`. Pattern is real and reusable.

**`engraveCodex32` always-true mirror:** confirmed. `engraveCodex32` (`codex32_polish.go:198`) returns `true` in all terminal cases. `engraveSLIP39` as specified mirrors this.

**`engraveTheme`:** confirmed at `theme.go:39,57` as a package-level `Colors` var. The spec's 2-arg `NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)` call is correct.

**Internal consistency §4.1/§4.2/§6/§8:** §4.1 and §4.2 are consistent on `ParseShare(string)`. §8 correctly summarizes R0 findings. §6 is internally consistent except for the wrong Identifier value. §9 process note is consistent with the project standard.

**`"fmt"` import:** `gui.go` currently imports `"strings"` (line 14) but not `"fmt"`. `codex32_polish.go:4` already imports `"fmt"` in its own file — so `"fmt"` is available in the `gui` package for the new `engraveSLIP39` function only if that function lives in `codex32_polish.go` or a new file. If `engraveSLIP39` is placed in `gui.go`, `"fmt"` must be added to `gui.go`'s import block. The spec says to add it to `gui.go`. This is correct.

---

## Verdict

**RED — 1 Critical / 0 Important**

Open Critical: C1 (§6 anchor vector Identifier value is 7945, actual decoded value is 10027 — causes a false-failing or implementation-breaking test). Must be corrected and R2 dispatched before any code.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | RED — 2C/3I/6m | Stale dormant code (scan.Words/3-arg Engrave/title-overflow/fmt-import/ParseShare-input). All folded. |
| R1 | RED — 1C/0I/4m | **C1**: my anchor vector asserted `Identifier==7945`, but 7945 is a hand-written DISPLAY string in backup_test.go — the mnemonic's decoded id is **10027** (`(313<<10|360)>>5`). Folded: §6 now asserts the robust 1-of-1 fields (GroupThreshold/Count=1, MemberIndex=0, MemberThreshold=1) + the corrected id 10027, and mandates the plan precompute+verify each embedded vector's fields against the trezor reference, NOT display strings; also clarified backup_test.go is engrave-only (write a NEW slip39/share_test.go). Minors folded: m1 strings already imported, m2 added a general `showError(ctx,th,title,msg)` helper (showCodex32Error hardcodes its title), m3 uppercase-normalize before exact lookup. Architect re-verified all R0 folds correct (Engrave 2-arg sig @ gui.go:2350, MaxTitleLen=18, engraveSLIP39 always-true mirrors engraveCodex32, inputSLIP39Flow sig, RS1024 ordering). |

Re-dispatched R2 after the fold.
