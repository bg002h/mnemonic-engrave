# Cycle C: slip39-enablement (Tier 1) — SPEC R0 GATE REVIEW — R3

- **Stage:** spec gate round R3 (after folding R2's 1 Critical + 1 Minor).
- **Spec reviewed:** `design/SPEC_seedhammer_slip39_enablement.md` (committed `ecbaea6`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `a8f3f4dbdf480858c`); independently re-derived the identifier from the fork's `slip39/wordlist.txt`.
- **Outcome:** **RED — 0 Critical / 1 Important** (the m3 uppercase-normalization note was in the R1 loop-summary but never written into §4.1). Folded; re-dispatched R4.

> NOTE: verbatim architect output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

R3 REVIEW — SPEC_seedhammer_slip39_enablement.md (Cycle C, Tier 1)

---

## CHECK 1: §6 Identifier Derivation (the C1 saga — be definitive)

Independent verification from `/scratch/code/shibboleth/seedhammer/slip39/wordlist.txt`:

- Line 249 = "duckling" → 0-based index **248**. Confirmed.
- Line 289 = "enlarge" → 0-based index **288**. Confirmed.

Derivation: `(248 << 10) | 288 = 253952 | 288 = 254240`. `254240 >> 5 = 7945`. Confirmed — **Identifier = 7945**.

The spec's derivation `(248<<10 | 288) >> 5 = 254240 >> 5 = 7945` is arithmetically correct and matches the fork's actual wordlist.

The string "10027" does not appear anywhere in the spec. The history note accurately describes R1→R2 correction. The anchor vector derivation is sound.

1-of-1 field assertions: words 2–3 are "academic academic" = indices 0, 0. Unpacking the header from the bit stream: GroupIndex=0, GroupThreshold stored=0 → decoded=1, GroupCount stored=0 → decoded=1, MemberIndex=0, MemberThreshold stored=0 → decoded=1. All match `GroupThreshold==1, GroupCount==1, MemberIndex==0, MemberThreshold==1`. Confirmed.

---

## CHECK 2: §4.2 showError arity — R2 m1 residual

Searching every occurrence of `showError` in §4.2:

- Line 88: `showError(ctx, th, "Invalid SLIP-39 share", slip39words.Describe(err))` — 4 args. Correct.
- Line 94: `showError(ctx, th, "Invalid SLIP-39 share", slip39words.Describe(err))` — 4 args. Correct.
- Line 115: `showError(ctx, th, "Too large", "Share doesn't fit a plate.")` — 4 args. Correct.
- Line 120: `showError(ctx, th, "Too large", "Share doesn't fit a plate.")` — 4 args. Correct.
- Line 130 (narrative text): "the new 4-arg `showError(ctx, th, title, msg)` helper". Confirmed.

No 3-arg `showError` form anywhere in the spec. The R2 m1 residual is fully resolved.

---

## CHECK 3: Regression / completeness full read

**RS1024 GEN constant:** `[0xe0e040, 0x1c1c080, 0x3838100, 0x7070200, 0xe0e0009, 0x1c0c2412, 0x38086c24, 0x3090fc48, 0x21b1f890, 0x3f3f120]` — verified against SLIP-0039 upstream. Matches exactly.

**20-word / 200-bit layout:** id(15) + ext(1) + iterExp(4) + groupIdx(4) + groupThresh(4) + groupCount(4) + memberIdx(4) + memberThresh(4) + padded value(130) + checksum(30) = 200 bits = 20 × 10. Correct.

**Engrave 2-arg:** `NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)` — confirmed 2-arg form against real code at `gui.go:1911`, `1937`, `1955`, `1971`. Correct.

**always-true engraveSLIP39:** returns `true` in all branches (Back from confirm, fit error, plate error, completed engrave). The "Unknown format" escape path is correctly avoided. Consistent.

**`case 3:` entry block:** builds uppercase words via `LabelFor`, joins with space, calls `slip39words.ParseShare`. Returns `share, true` on success; calls `showError` and loops on failure. Pattern matches `case 2:` / `inputCodex32Flow` idiom.

**Title ≤ 18 chars:** `fmt.Sprintf("%d #%d/%d", ...)` max = `"32767 #16/16"` = 12 chars. Confirmed ≤ 18.

**`"fmt"` not yet in `gui.go`:** confirmed absent; spec correctly requires adding it.

**`"strings"` already present in `gui.go:14`:** confirmed. No re-add needed.

**`ParseShare(string)` consistent:** §4.1 signature `ParseShare(mnemonic string) (Share, error)` matches §4.2 call site `slip39words.ParseShare(strings.Join(words, " "))`. Consistent.

**`codex32`/`mdmk.go` untouched:** `codex32.String` case at `gui.go:1841` and `mdmkText` case at `gui.go:1845` are unmodified by the spec. Consistent.

**Tier-1 scope:** no Shamir/Feistel/PBKDF2/passphrase/multi-group collection anywhere in the implementation sections. Consistent.

**Dormant 3-arg Engrave confirmed stale:** `gui.go:1836` shows `Engrave(ctx, ops, &engraveTheme)` (3-arg) in the commented block. The spec correctly discards this and uses the real 2-arg form.

**backup_test.go title cross-check:** `backup_test.go:215` uses `"7945 #1 1/1"` (hand-written, format `"%d #%d %d/%d"`). The spec's title format `fmt.Sprintf("%d #%d/%d", ...)` produces `"7945 #1/1"` for this share. The spec explicitly says not to assert the parse result against this string, which is correct guidance. This is a cosmetic inconsistency between the pre-existing golden test title and the spec's chosen live format — it does not break any test (the golden test is hardcoded and never calls `ParseShare`), but it means the engraved plate title will differ from what the golden test shows. This is a MINOR noting for the implementer's awareness only.

---

## IMPORTANT DEFECT FOUND

**IMPORTANT — §4.1 / §6: `ParseShare` case normalization unspecified**

File: `design/SPEC_seedhammer_slip39_enablement.md`, §4.1 decode step (2) and §6.

The spec requires `ParseShare` to be tested against the official `trezor/python-shamir-mnemonic` `vectors.json` mnemonics (§6). Those vectors use lowercase words (e.g. `"duckling enlarge academic academic ..."`). The internal wordlist stored in `slip39/wordlist.go` is entirely uppercase (`"ACADEMICACIDACNE..."`). The spec's §4.1 decode step (2) specifies "an **exact** wordlist lookup (the in-package `index` map)" with no mention of case normalization.

An implementer following the spec literally will build an exact-match map with uppercase keys. Calling `ParseShare("duckling enlarge ...")` (lowercase, as in the test vectors) will produce a "not-in-wordlist" sentinel for every word, and the C1 vector tests mandated in §6 will all fail. The GUI path is unaffected (it goes through `LabelFor` which returns uppercase), but the test gate the spec requires cannot pass without normalization.

Fix: add to §4.1 decode step (2): "Normalize each word to uppercase (e.g. `strings.ToUpper`) before lookup; the internal wordlist is uppercase and the official test vectors are lowercase."

This is IMPORTANT because it is a correctness-breaking gap between §4.1 and §6 that will cause every vector test to fail with the wrong error code, making the C1 test gate impossible to pass as written.

---

## VERDICT

**RED — 0 Critical / 1 Important**

Open defect: §4.1 decode step (2) does not specify case normalization for `ParseShare` input words; the C1 vector tests in §6 feed lowercase words and will fail against an exact uppercase lookup as currently specified. Add one sentence to §4.1 step (2) requiring `strings.ToUpper` (or equivalent) before lookup. All other spec content — the identifier derivation (7945, confirmed independently), 1-of-1 field assertions, `showError` 4-arg form throughout, GEN constant, 200-bit layout, 2-arg Engrave, always-true engraveSLIP39, title ≤ 18, import requirements — is correct.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | RED — 2C/3I/6m | Stale dormant code. |
| R1 | RED — 1C/4m | Anchor id 7945→"10027". |
| R2 | RED — 1C/1m | R1's 10027 wrong; reverted to verified 7945. |
| R3 | RED — 0C/1I | **Independently re-confirmed Identifier=7945** ((248<<10\|288)>>5) + the 1-of-1 fields by reading the actual wordlist.txt. Found the m3 uppercase-normalization fix was only in the R1 summary, never in §4.1 body — without it the §6 lowercase vector tests fail every word. Folded the actual `strings.ToUpper` sentence into §4.1 step (2). All else GREEN (GEN constant, 200-bit layout, 2-arg Engrave, always-true engraveSLIP39, title≤18, fmt/strings imports, showError 4-arg, scope, codex32/mdmk untouched). |

Re-dispatched R4. (Expect GREEN — the only open item was the one-sentence normalization fold.)
