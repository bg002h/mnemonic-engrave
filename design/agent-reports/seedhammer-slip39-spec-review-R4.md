# Cycle C: slip39-enablement (Tier 1) — SPEC R0 GATE REVIEW — R4

- **Stage:** spec gate round R4 (after folding R3's 1 Important). FINAL.
- **Spec reviewed:** `design/SPEC_seedhammer_slip39_enablement.md` (committed `d992734`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `afce5e48af43dff81`); re-verified the identifier + full spec vs fork `9b0a02c` + SLIP-0039.
- **Outcome:** **GREEN — 0 Critical / 0 Important.** Spec cleared to the plan phase.

> NOTE: verbatim architect output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

## R4 Architect Review — SPEC_seedhammer_slip39_enablement.md

**Spec:** `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_seedhammer_slip39_enablement.md`
**Commit:** `d992734`. Fork base `9b0a02c`.

---

### R3 Fold Verification (the only open item from R3)

The R3 Important finding was: §4.1 step (2) was missing the normalization sentence, causing a §4.1↔§6 case mismatch where the §6 lowercase official-vector test words would fail the exact wordlist lookup.

**Sentence location:** §4.1, step (2), within the `Decode` paragraph (spec line 67).

**Sentence text (confirmed present):** "normalizing each word to uppercase first (`strings.ToUpper`) — the in-tree wordlist is UPPERCASE (`wordlist.go`'s `words` = `"ACADEMICACID…"`, so `LabelFor` returns uppercase) while the official SLIP-0039 test vectors are lowercase; without this, the §6 vector tests would fail every word as not-in-wordlist (R3 fix)."

**Verification against codebase:**
- `/scratch/code/shibboleth/seedhammer/slip39/wordlist.go` line 11: `const words = "ACADEMICACID..."` — all uppercase. Confirmed.
- `LabelFor(w Word) string` returns a slice of `words` — therefore always uppercase. Confirmed.
- `ClosestWord` at `slip39.go:37` performs `LabelFor(Word(i)) >= word` — this binary search works on uppercase strings; if called with lowercase it would fail (out-of-alphabetic-order comparison). The spec correctly directs `ParseShare` NOT to use `ClosestWord` but to do its own exact lookup after ToUpper. Consistent.
- GUI path: `case 3:` builds `words[i] = slip39words.LabelFor(w)` (line 86 in §4.2 code block) before passing to `ParseShare`. `LabelFor` returns uppercase → `ToUpper` is idempotent → GUI path unaffected. Correct.
- §6 lowercase official-vector tests will now pass: ToUpper applied before lookup means "duckling" → "DUCKLING" → found at index 248. Confirmed.

**R3 fold: CORRECT. The sentence is present, accurate, and resolves the §4.1↔§6 mismatch without breaking the GUI path.**

---

### Full Fresh Read — Churned Items Checklist

**Identifier = 7945 derivation (§6):**
- "duckling" verified at `wordlist.txt` line 249 → 0-based index 248. Confirmed.
- "enlarge" verified at `wordlist.txt` line 289 → 0-based index 288. Confirmed.
- Arithmetic: `(248 << 10 | 288) >> 5 = (253952 | 288) >> 5 = 254240 >> 5 = 7945`. Correct.
- Spec states this correctly. No defect.

**"10027" absent from assertions:**
- The string "10027" appears once in line 143 (the long concrete-anchor-vector paragraph), explicitly labeled as a mis-derivation history note ("an R1 round mis-derived this as 10027 using indices 313/360 from a *different* wordlist; R2 corrected it"). It does not appear as an asserted correct value anywhere. No defect.

**`showError` 4-arg everywhere:**
- §4.2 "Error modal helper" bullet: `func showError(ctx *Context, th *Colors, title, msg string)`. 4-arg. Confirmed.
- Entry block call: `showError(ctx, th, "Invalid SLIP-39 share", slip39words.Describe(err))`. 4-arg. Confirmed.
- Engrave helper: `showError(ctx, th, "Too large", "Share doesn't fit a plate.")`. 4-arg. Confirmed (two occurrences in `engraveSLIP39`). Confirmed.
- Spec explicitly notes at line 130: "`showError` is the new 4-arg `showError(ctx, th, title, msg)` helper... (R2 m1 — not a 3-arg form)". Consistent.

**RS1024 GEN constants:**
- Spec GEN: `[0xe0e040,0x1c1c080,0x3838100,0x7070200,0xe0e0009,0x1c0c2412,0x38086c24,0x3090fc48,0x21b1f890,0x3f3f120]`.
- SLIP-0039 canonical: identical. Confirmed.

**RS1024 decode order (ext-first before checksum verify):**
- §4.1 step (4): "extract the `ext` bit (bit 15) FIRST, then verify RS1024 with cs = 'shamir'(ext=0)/'shamir_extendable'(ext=1)". Correct ordering per SLIP-0039 (the customization string depends on ext, so ext must be known before polymod can be computed). Confirmed correct.

**20-word / 200-bit layout:**
- Header: 15+1+4+4+4+4+4+4 = 40 bits. Padded share value: 2 pad + 128 data = 130 bits. Checksum: 30 bits. Total: 200 bits = 20 × 10. Matches spec claim "totals 200 bits = 20×10". Confirmed.

**`ParseShare(string)` consistent §4.1↔§4.2:**
- §4.1: `func ParseShare(mnemonic string) (Share, error)`. String input. Confirmed.
- §4.2 `case 3:`: `slip39words.ParseShare(strings.Join(words, " "))`. String passed. Confirmed.
- §6: "every 20-word mnemonic in a valid vector set `ParseShare`-OK" — the official vectors are space-separated strings. Confirmed consistent throughout.

**Engrave 2-arg + always-true `engraveSLIP39`:**
- `NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)`: verified against `gui.go:2350`: `func (s *EngraveScreen) Engrave(ctx *Context, th *Colors) bool`. 2-arg. Confirmed.
- `engraveSLIP39` returns `true` in all branches (confirm-back: `return true`; EngraveSeed err: `return true`; toPlate err: `return true`; normal completion: `return true`). Always-true confirmed.
- `case slip39words.Share: return engraveSLIP39(ctx, th, scan)` — the type switch uses the alias `slip39words.Share`. Consistent with import `slip39words "seedhammer.com/slip39"` at `gui.go:40`. Confirmed.

**Title ≤ 18 chars:**
- `backup.MaxTitleLen = 18` at `backup/backup.go:43`. Confirmed.
- Title format: `fmt.Sprintf("%d #%d/%d", scan.Identifier, scan.MemberIndex+1, scan.MemberThreshold)`.
- Maximum: id is 15-bit → max 32767 (5 chars). MemberIndex 4-bit → 0-15 → +1 = 1-16 (2 chars). MemberThreshold stored as value-1 → 4-bit → 1-16 (2 chars). Format overhead: " #" (2) + "/" (1) = 3. Max string: "32767 #16/16" = 12 chars. Well under 18. Confirmed.
- Note: the dormant comment at `gui.go:1818` used the title `"%d #%d 1/%d"` (hardcoded `1/` — clearly wrong for multi-member). The spec's format `"%d #%d/%d"` with `MemberThreshold` is correct. Confirmed improved.

**`fmt` to add / `strings` already present:**
- `gui.go` import block: `"strings"` at line 14. Present. Confirmed.
- `"fmt"` NOT present in import block (checked lines 1-41). Spec says "add `"fmt"`". Correct.

**Tier-1 scope (no Shamir/Feistel/PBKDF2/passphrase/multi-group):**
- §1: "does NOT reconstruct the master secret". Confirmed.
- §2 out-of-scope list: "secret recovery (combining k shares — GF(256) Shamir + 4-round Feistel + PBKDF2 + two-level group/member combine); passphrase entry". Confirmed.
- §5: "The share is engraved verbatim (no decryption, no master-secret derivation). Passphrase never entered (recovery-only); the master secret is never reconstructed". Confirmed.
- §8: "Tier 1 (entry + verbatim engrave), NOT recovery". Confirmed.

**`codex32`/`mdmk` untouched:**
- §2 Files: "`codex32/*`, `mdmk.go` unchanged". Confirmed stated.
- §2 Out: "`codex32`/`mdmk.go` untouched". Confirmed.

**`case 3:` + `engraveSLIP39` code blocks compile against real APIs:**
- `emptySLIP39Mnemonic(20)` — referenced as existing dormant code at `gui.go:503`. Present.
- `inputSLIP39Flow(ctx, th, m, 0) bool` — referenced as existing dormant code at `gui.go:755`. Present.
- `slip39words.LabelFor(w)` — exists in current package. Confirmed.
- `slip39words.ParseShare(...)` — new API to be added in `slip39/share.go`. Spec is self-consistent.
- `slip39words.Describe(err)` — new helper to be added alongside `ParseShare`. Spec-consistent; §4.1 mentions "a `Describe`-style classifier for the GUI".
- `confirmSLIP39Flow(ctx, th, scan)` — new C3 function, to be created in `gui/slip39_confirm.go`. Spec-consistent.
- `backup.Seed{Mnemonic []string, ShortestWord, LongestWord, Title, Font}` — verified against `gui.go:470-473` which shows same struct fields. Confirmed.
- `ctx.Platform.EngraverParams()` — used identically in existing BIP-39 path. Confirmed.
- `backup.EngraveSeed(params, seedDesc)` — same call pattern as existing BIP-39 path. Confirmed.
- `toPlate(seedSide, params)` — `func toPlate` at `gui.go:2648`. Confirmed.
- `NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)` — `gui.go:2338,2350`. Confirmed.
- `slip39words.ShortestWord` and `slip39words.LongestWord` — `wordlist.go:7,9`. Confirmed.
- `constant.Font` — imported package `seedhammer.com/font/constant` at `gui.go:30`. Confirmed.

**`scan.go` NFC block stays disabled:**
- §2: "`gui/scan.go` stays as-is". Confirmed stated.
- §3: "SLIP-39 NFC block — structurally stale (`res.Content` no longer exists); stays disabled". Confirmed.

---

### CRITICAL

None.

### IMPORTANT

None.

### MINOR

None. The "in-package `index` map" phrasing in §4.1 is slightly imprecise (`index` is a uint16 offset array, not a Go `map`), but this is appropriate implementation-agnostic language — the spec correctly specifies behavior (exact match, not fuzzy), leaving the implementation mechanism to the implementer. Not a defect.

---

**GREEN — 0 Critical / 0 Important**

The R3 fold is correct and complete. The spec is internally consistent, accurate against the fork codebase at `9b0a02c`, and consistent with the SLIP-0039 canonical specification. All churned items from prior rounds are verified. Ready for implementation plan.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | RED — 2C/3I/6m | Stale dormant code (scan.Words/3-arg Engrave/title/fmt/ParseShare-input). |
| R1 | RED — 1C/4m | Anchor-vector id: flagged 7945, "corrected" to 10027. |
| R2 | RED — 1C/1m | R1's 10027 was wrong (wrong wordlist indices); reverted to verified 7945. |
| R3 | RED — 0C/1I | uppercase-normalization note was only in the R1 summary, not §4.1 body; added. |
| R4 | **GREEN — 0C/0I** | R3 fold confirmed; full fresh read clean — Identifier=7945 re-derived, no 10027 in assertions, showError 4-arg throughout, RS1024 GEN + ext-first ordering, 200-bit layout, 2-arg Engrave + always-true engraveSLIP39, title≤18, fmt-to-add/strings-present, Tier-1 scope, codex32/mdmk untouched, all code blocks compile against real APIs. |

GATE PASSED at R4. Proceed to the implementation plan (which gets its own R0 gate).
