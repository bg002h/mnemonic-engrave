# Cycle C: slip39-enablement (Tier 1) — SPEC R0 GATE REVIEW — R2

- **Stage:** spec gate round R2 (re-review after folding R1's 1 Critical + 4 Minor).
- **Spec reviewed:** `design/SPEC_seedhammer_slip39_enablement.md` (committed `5f92521`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `a449396d36d828107`), read-only vs fork `main` `9b0a02c`; independently re-derived the identifier against the fork's `slip39/wordlist.txt`.
- **Outcome:** **RED — 1 Critical (C1: R1's "corrected" identifier 10027 is itself wrong; actual is 7945) + 1 Minor.** Folded; re-dispatched R3.

> NOTE: verbatim architect output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

# R2 — Spec Gate Review (Cycle C, Tier 1)

**Spec:** `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_seedhammer_slip39_enablement.md` (post-R1 fold)
**Reviewer:** opus architect, independent
**Date:** 2026-06-18

---

## CRITICAL

**C1 — §6 incorrect decoded Identifier: R1 fold introduced a wrong value (10027 ≠ 7945)**

The fold replaced the "correct" identifier with 10027, citing `(313<<10|360)>>5 = 320872>>5 = 10027`. The arithmetic is correct for those indices, but the indices themselves are wrong for the fork's wordlist.

Independent derivation from the fork's authoritative file `/scratch/code/shibboleth/seedhammer/slip39/wordlist.txt`:

- "duckling" is at 1-based line 249 → 0-based word index **248**
- "enlarge" is at 1-based line 289 → 0-based word index **288**
- Neighbors confirm: line 248 = "dryer", line 250 = "duke"; line 288 = "enjoy", line 290 = "entrance"

Computation: `(248 << 10) | 288 = 253952 + 288 = 254240`. `254240 >> 5 = 7945` exactly (`7945 × 32 = 254240`).

At the indices the spec cites: line 313 (0-based 312) = "excuse"; line 360 (0-based 359) = "focus". Neither is "duckling" or "enlarge". Those indices appear to be from a different wordlist (possibly a cached or misremembered version of the Trezor/SatoshiLabs canonical SLIP-39 wordlist) and do not apply to this fork.

The `backup_test.go:215` title string `"7945 #1 1/1"` was correctly computed from the fork's wordlist. The R1 fold's characterization of 7945 as "a hand-written display string, NOT a decoded field" is partially correct (it is not derived by calling `ParseShare`), but the conclusion — that the actual decoded value is 10027 — is wrong for this wordlist. 7945 IS the correct Identifier that a correct `ParseShare` implementation should return for this mnemonic with this wordlist.

The 1-of-1 robust field assertions (`GroupThreshold==1, GroupCount==1, MemberIndex==0, MemberThreshold==1`) in §6 are independently verified correct: words 2-3 are both "academic" (index 0), which places 0s in the GroupThreshold-stored, GroupCount-stored, MemberIndex, and MemberThreshold-stored bit fields, decoding to 1, 1, 0, 1 respectively after the +1 adjustment.

**Fix required before any code:** Change §6's anchor vector assertion from `Identifier==10027` to `Identifier==7945`, and replace the derivation `(313<<10 | 360) >> 5 = 320872 >> 5 = 10027` with `(248<<10 | 288) >> 5 = 254240 >> 5 = 7945`. Remove the statement "the 7945-vs-10027 discrepancy is exactly the trap this note guards against" — the guard remains valid (don't use display strings from backup_test.go), but the "corrected" value must be 7945. The plan-level instruction to precompute + independently verify each embedded vector's header fields against the Trezor reference decoder remains valuable and correct; the anchor derivation simply uses the wrong wordlist.

---

## IMPORTANT

None.

---

## MINOR

**m1 — §4.2 line 130: 3-arg `showError` in closing parenthetical contradicts the 4-arg definition**

The sentence reads: "(`showError` is a small dismissible-`ErrorScreen` helper (reuse Cycle-B's `showCodex32Error` pattern, or a shared `showError(ctx, th, msg)`)". The fragment "`showError(ctx, th, msg)`" is a 3-arg signature that contradicts the 4-arg `func showError(ctx *Context, th *Colors, title, msg string)` defined and used consistently throughout the same section. An implementer reading quickly could end up defining the wrong signature. Fix: replace "`showError(ctx, th, msg)`" with "`showError(ctx, th, title, msg)`" in that parenthetical.

---

## Confirmation of Folded Items

| Item | Status |
|------|--------|
| R0 C2: Engrave 2-arg `(ctx, th)` | CONFIRMED ✓ — `gui.go:2350` `func (s *EngraveScreen) Engrave(ctx *Context, th *Colors) bool` |
| R0 I1: title ≤ MaxTitleLen=18 | CONFIRMED ✓ — `backup/backup.go:43` `MaxTitleLen = 18`; max title "32767 #16/16" = 12 chars |
| R0 fmt import gap | CONFIRMED ✓ — `gui.go` imports (lines 4-41) include `"strings"` but not `"fmt"` |
| R0 scan.Mnemonic | CONFIRMED ✓ — `Share.Mnemonic []string` in spec §4.1; used as `scan.Mnemonic` in `engraveSLIP39` |
| R0 always-true helper | CONFIRMED ✓ — `engraveSLIP39` returns `true` on all paths including back/cancel |
| R1 m1: strings already imported | CONFIRMED ✓ — `gui.go:14` imports `"strings"` |
| R1 m2: showError 4-arg; ErrorScreen loop shape | CONFIRMED ✓ — `ErrorScreen.Layout(ctx, th, dims) (op.Op, bool)` per `codex32_polish.go:149`; loop shape matches; all three call sites in spec use 4-arg form (subject to m1 above) |
| R1 m3: ToUpper normalization | CONFIRMED ✓ — `LabelFor` returns uppercase; `ClosestWord` compares uppercase; lowercase test-vector inputs need `strings.ToUpper` before lookup |
| ParseShare(string) consistent | CONFIRMED ✓ — §4.1 signature, §4.2 call, dormant block conversion all use space-joined string |
| Tier-1 scope exclusions | CONFIRMED ✓ — §§2, 8 explicitly exclude Shamir, Feistel, PBKDF2, passphrase, multi-group |
| codex32/mdmk.go untouched | CONFIRMED ✓ — §2 explicitly states these are unchanged |
| 1-of-1 field assertions (GroupThreshold==1, GroupCount==1, MemberIndex==0, MemberThreshold==1) | CONFIRMED ✓ — independently derived from bit layout with words 2-3 both "academic" (index 0) |

---

## Verdict

**RED — 1 Critical / 0 Important**

Open item: **C1** — §6 asserts `Identifier==10027` (derived from word indices 313 and 360, which are "execute" and "forbid" in the fork's wordlist). The correct value for this mnemonic with the fork's wordlist is **`Identifier==7945`** (word indices 248 and 288 for "duckling" and "enlarge"). This must be corrected and R3 dispatched before any code begins.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | RED — 2C/3I/6m | Stale dormant code. Folded. |
| R1 | RED — 1C/4m | Anchor-vector id: flagged 7945 as a display string, "corrected" to 10027. |
| R2 | RED — 1C/1m | **The R1 "correction" was itself WRONG**: 10027 used indices 313/360 from a different wordlist. R2 verified the FORK's `slip39/wordlist.txt`: duckling=248, enlarge=288 → `(248<<10\|288)>>5 = 7945`. So the original `backup_test.go` 7945 was correct. Folded: reverted §6 to id **7945** with the verified derivation, kept the precompute-against-the-real-wordlist guard (now better-justified by the 10027 misstep). m1: fixed a residual 3-arg `showError` parenthetical. Architect re-confirmed all R0/R1 folds otherwise correct (2-arg Engrave, MaxTitleLen=18, fmt gap, scan.Mnemonic, always-true engraveSLIP39, strings imported, 4-arg showError loop shape, ToUpper, 1-of-1 fields). |

LESSON (verify-external-facts-against-authoritative-source): an opus reviewer mis-derived a value from wrong wordlist indices; only re-deriving against the actual `wordlist.txt` caught it. Re-dispatched R3.
