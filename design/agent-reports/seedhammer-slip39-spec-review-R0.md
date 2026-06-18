# Cycle C: slip39-enablement (Tier 1) — SPEC R0 GATE REVIEW — R0

- **Stage:** mandatory spec R0 gate (0C/0I before any code).
- **Spec reviewed:** `design/SPEC_seedhammer_slip39_enablement.md` (committed `e3ae6ea`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `a26781e87706818a2`), read-only vs fork `main` `9b0a02c` + live SLIP-0039 spec + official vectors.
- **Outcome:** **RED — 2 Critical (C1,C2) + 3 Important (I1,I2,I3) + 6 Minor.** All folded; re-dispatched R1.

> NOTE: verbatim architect output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

**R0 GATE REVIEW — SLIP-39 share entry + verbatim engrave (Cycle C, Tier 1)**
**Spec:** `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_seedhammer_slip39_enablement.md`
**Reviewer:** opus architect (adversarial, read-only)
**Authoritative sources:** SLIP-0039 spec (fetched live), `trezor/python-shamir-mnemonic/vectors.json` (fetched live), fork source tree at `9b0a02c`

---

## CRITICAL

**C1 — The dormant engrave block (`gui.go:1812`) calls `scan.Words()` which has no analog in the spec's `Share` struct — the spec never resolves this, leaving an uncompilable re-enablement.**

Source: `gui.go:1812`: `w, err := scan.Words()`. The spec's proposed `Share` struct (§4.1) provides `Mnemonic []string` but no `Words()` method. The spec §3 asks "Does the dormant code reference `scan.Words()`?" and §4.2 says "re-enable as `case slip39words.Share:`" with "the share's 20 words (verbatim)" — but nowhere specifies that the re-enabled block must replace `scan.Words()` with `scan.Mnemonic`. The dormant block also checks `err` from `scan.Words()`, which a plain `[]string` field has no analog for. The plan author will encounter a compile error and must improvise, which is a gap in the spec's re-enablement prescription. Fix: §4.2 must explicitly state that the re-enabled `case slip39words.Share:` block uses `scan.Mnemonic` directly (it is always length 20 at this point, since `ParseShare` already validated it), drops the `err` check, and removes the `len(w) > maximumLength` guard (which is superseded by `ParseShare`'s 20-word-only restriction). The spec should carry these concrete substitutions.

**C2 — The dormant engrave block (`gui.go:1836`) calls `.Engrave(ctx, ops, &engraveTheme)` with three arguments; the real `EngraveScreen.Engrave` signature at `gui.go:2350` takes two `(ctx *Context, th *Colors)`. The spec never acknowledges this stale signature.**

Source: `gui.go:1836` dormant: `NewEngraveScreen(ctx, plate).Engrave(ctx, ops, &engraveTheme)`. Live signature at `gui.go:2350`: `func (s *EngraveScreen) Engrave(ctx *Context, th *Colors) bool`. The spec §3 catalogues the dormant symbols checked but omits `ops` and `&engraveTheme` as stale arguments. The spec §4.2 gives high-level pseudocode for the engrave path but doesn't translate the call site. A plan author following the spec will still encounter a three-argument compile error. Fix: §4.2 must specify the corrected call — parallel to `backupWalletFlow`'s `gui.go:1937`: `NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)` — and note that `ops` disappears (no longer a parameter to `Engrave`).

---

## IMPORTANT

**I1 — The spec proposes a title format `"SLIP-39 <id> #<member+1>/<memberThreshold>"` (§4.2) that can reach 20 characters at maximum field values, exceeding `backup.MaxTitleLen = 18` (`backup/backup.go:43`). The dormant code's title format `"%d #%d 1/%d"` stays within 18 chars at worst (14 chars); the spec's format adds "SLIP-39 " (8 chars) prefix, blowing the limit.**

Calculation: "SLIP-39 " (8) + "32767" (5) + " #" (2) + "16" (2) + "/" (1) + "16" (2) = 20 chars. `MaxTitleLen = 18`. The `frontSideSeed` function at `backup/backup.go:216` does NOT call `TitleString()` to truncate — it engraves whatever `plate.Title` is, silently overflowing the physical plate margin. The existing `backup_test.go:TestSLIP39` uses `Title: "7945 #1 1/1"` (12 chars), well within bounds. The spec must either revert to the dormant title format (`"%d #%d 1/%d"`) or explicitly acknowledge the overflow risk and choose a format that fits within 18 chars (e.g., `"S39 <id> #<m>/<t>"` = at most 16 chars). The confirm screen title "Confirm SLIP-39 Share" is unrelated to `backup.Seed.Title` and is fine.

**I2 — `"fmt"` is not imported in `gui/gui.go` (it appears only in the commented-out dormant code); re-enabling the `case slip39words.Share:` block with `fmt.Sprintf` for the title will produce a compile error unless `"fmt"` is added to the import block. The spec does not mention this.**

Source: grepping `gui.go` for `"fmt"` returns zero hits in the import block; the only occurrence is the dormant `// title := fmt.Sprintf(...)` at `gui.go:1818`. The spec §4 and §5 say nothing about import changes. This is trivially fixable but must be in the plan to avoid a broken build. Fix: the implementation plan must add `"fmt"` to `gui/gui.go`'s import block as part of C2.

**I3 — The spec's API surface question "should `ParseShare` take `string` or `slip39words.Mnemonic`?" is flagged as a "plan decision" in §4.2 but the spec itself settles it in §4.1 (`func ParseShare(mnemonic string) (Share, error)`) without acknowledging the conversion cost or the fact that the dormant code already builds the `strings.Builder` join in the `case 3:` block. This isn't wrong — the decision IS made — but §4.2 still says "convert to the space-joined word string `ParseShare` expects, or have `ParseShare` accept the `Mnemonic` — settle in the plan." This creates a contradiction: §4.1 settles the signature; §4.2 says it still needs settling. An implementer reading §4.2 after §4.1 may be confused about whether this is already decided.**

The dormant `case 3:` block (gui.go:2008-2014) already builds the space-joined string: the 14-line conversion is present in the dormant code. The spec §4.1 codifies `ParseShare(mnemonic string)` which matches. Fix: §4.2 should remove the "or have `ParseShare` accept the `Mnemonic` — settle in the plan" hedge and simply say "the dormant conversion code (lines 2008-2014) is already correct; uncomment and fix the bare `slip39.ParseShare` reference to `slip39words.ParseShare`."

---

## MINOR

**M1 — The `inputSLIP39Flow` title ("Input Words" at `gui.go:827`) is not updated by the spec. For BIP-39, `inputWordsFlow` shows a dynamic "Word N of M" progress title; for SLIP-39 the static "Input Words" is functional but inconsistent. Not a bug, but the spec should explicitly acknowledge the choice rather than silently inheriting it.**

**M2 — The spec §6 says "every 20-word mnemonic in a *valid* vector set `ParseShare`-OK with the expected decoded metadata (id, thresholds, member index — cross-checked against the vector's known structure, e.g. a 1-of-1 set has group/member thresholds 1)." The vectors.json format is `[description, [mnemonics...], master_secret_hex, xprv]` — the test would need to hard-code expected metadata (id, groupThreshold, groupCount, memberIndex, memberThreshold) per vector since vectors.json only provides the master secret, not the decoded share header fields. The spec says to "embed a small fixed set of official 20-word vectors" but does not provide the expected decoded field values. The implementer will need to compute these from the mnemonic bits (or use a reference implementation) to populate the test assertions. This is a missing test-data detail, not a showstopper, but the plan should supply at least one fully-specified vector assertion (e.g., the "duckling enlarge…" share already used in `backup_test.go`; its identifier is 7945 per `backup_test.go:215`, and for a 1-of-1 single-group share: groupThreshold=1, groupCount=1, memberIndex=0, memberThreshold=1).**

**M3 — The spec §4.2 calls the re-enabled menu choice `"SLIP-39"` but does not address the lead text "Choose number of words" in the `ChoiceScreen` (gui.go:1982), which becomes slightly misleading when SLIP-39 is a fourth option (SLIP-39 shares are identified by word count, not chosen as a word count). Minor UX inconsistency; noting for plan completeness.**

**M4 — RS1024 verification order: the implementer must extract the `ext` bit from the decoded header BEFORE choosing the customization string, then run polymod. The spec's description in §4.1 lists these steps (fields at SLIP-0039 offsets, then verify RS1024 with ext-selected cs) but does not make the sequencing dependency explicit. A naïve read might suggest verifying polymod before extracting ext. Experienced Go implementer will handle this correctly, but the implementation plan should make the order explicit: (1) words → indices, (2) extract ext bit from bit 15 of the packed bitstream, (3) verify polymod with cs = "shamir" or "shamir_extendable" per ext, (4) decode remaining header fields.**

**M5 — The GEN polynomial constant in the spec §4.1 matches the SLIP-0039 spec exactly (verified live against `https://raw.githubusercontent.com/satoshilabs/slips/master/slip-0039.md`). The 10-element constant is correct. No issue.**

**M6 — The 20-word / 200-bit arithmetic is verified: 40-bit header + 130-bit padded share value + 30-bit checksum = 200 bits = 20 × 10-bit words. The padding is 2 zero bits prefixed to the 128-bit share value (per spec: "left-padded with 0 bits so that the length…becomes the nearest multiple of 10"). Bit counts are correct in the spec.**

---

## POSITIVE FINDINGS (NOT issues)

- RS1024 algorithm, GEN constant, verification condition (`polymod == 1`), and both customization strings (`"shamir"` / `"shamir_extendable"`) are correct per the live SLIP-0039 spec.
- 128-bit/20-word vectors confirmed: both `vectors.json` vector 1 and vector 42 contain exactly 20 words. The test strategy is sound.
- The `backup.EngraveSeed` path for 20-word SLIP-39 is pre-validated by `backup_test.go:TestSLIP39` (golden file `testdata/slip39-0.bin` exists).
- `slip39words.ShortestWord = 4`, `LongestWord = 8` are confirmed exported from `slip39/wordlist.go:7-9` and match the SLIP-39 wordlist spec.
- `inputSLIP39Flow`, `emptySLIP39Mnemonic`, `completeSLIP39Word`, `updateValidSLIP39Keys` are all live (not commented), fully functional for the entry flow.
- The package alias `slip39words "seedhammer.com/slip39"` at `gui.go:40` is confirmed. The spec's identification of all bare `slip39.Xxx` references that need the alias fix is accurate.
- The NFC SLIP-39 block in `scan.go:61-65` is confirmed stale (`res.Content` references a non-existent field) and correctly identified as staying disabled.
- Tier-1 scope genuinely excludes Shamir/Feistel/PBKDF2: entry+engrave needs only RS1024 + bit-decode. No secret is reconstructed. The "silent wrong passphrase" and "BIP-32 vs BIP-39" footguns correctly do not arise.
- The `runUI`/`ExtractText`/`uiContains` test seam is confirmed in `gui_test.go:466-484`; the C2/C3 test approach follows a direct precedent in `codex32_input_test.go`.

---

## VERDICT

**RED — 2 Critical, 3 Important open**

The spec cannot be passed to implementation as-is. The two Criticals (C1: `scan.Words()` not resolved; C2: stale 3-argument `.Engrave()` call not acknowledged) will produce compile errors in the re-enabled engrave block; the three Importants (I1: title overflow, I2: missing `fmt` import, I3: §4.1/§4.2 contradiction on `ParseShare` input type) will require the implementer to make unspecified decisions or produce broken builds.

Required fold: add to §4.2 explicit substitutions for every line of the dormant `case slip39words.Share:` block — specifically replace `scan.Words()` → `scan.Mnemonic`, drop the `(_, err)` pattern, fix the 3→2 argument `.Engrave()` call; resolve the title to a format ≤18 chars; add `"fmt"` to the gui.go import list; and remove the §4.2 hedge about `ParseShare`'s input type that contradicts §4.1.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | RED — 2C / 3I / 6m | Root cause: the spec said "re-enable the dormant code" but that code is STALE. **C1** `scan.Words()` → folded to `scan.Mnemonic` (drop err + maximumLength guard). **C2** stale 3-arg `.Engrave(ctx, ops, &theme)` → folded to 2-arg `Engrave(ctx, &engraveTheme)`. **I1** 20-char title > MaxTitleLen=18 → folded to `"%d #%d/%d"` (max 12). **I2** `gui.go` lacks `"fmt"` import → folded (add fmt + strings). **I3** §4.1/§4.2 ParseShare-input contradiction → folded: keep `ParseShare(string)` (vectors are strings), GUI joins words. Also (not flagged but caught): routed engrave through an always-`true` `engraveSLIP39` helper to avoid the A1 "Unknown format"-on-cancel pitfall. **Minors folded:** M4 RS1024 decode-ordering (extract ext bit before verify), exact-wordlist-lookup, M2 concrete anchor vector (id 7945 1-of-1), M1 static entry-title kept, M3 lead-text noted. Architect positively confirmed: RS1024 GEN constant + customization strings correct vs live spec; 20-word/200-bit arithmetic; Tier-1 genuinely excludes Shamir/Feistel/PBKDF2; dormant entry helpers live; runUI test seam; backup.EngraveSeed 20-word path golden-tested. |

Re-dispatched R1 after the fold.
