# Cycle A1: codex32-input-polish — SPEC R0 GATE REVIEW — R0

- **Stage:** formal spec R0 architecture gate (mandatory, project standard — must reach 0 Critical / 0 Important before any code).
- **Spec reviewed:** `design/SPEC_seedhammer_codex32_input_polish.md` (committed `62d4251`).
- **Reviewer:** opus `feature-dev:code-architect` (read-only; agentId `a7fef8f8784614d6f`). Code read on fork `main` `3c4d3d3`; BIP-93 re-verified vs raw.githubusercontent.com/bitcoin/bips/master/bip-0093.mediawiki.
- **Outcome:** **GREEN — 0 Critical / 0 Important.** Three non-blocking Minors folded into the spec (see Loop summary).

> NOTE: the verbatim architect output is reproduced below, recovered from the agent transcript (`tasks/a7fef8f8784614d6f.output`). A short working-notes preamble that preceded the formal review has been omitted; the review proper (from the `# SPEC R0 GATE REVIEW` heading) is verbatim.

---

## VERBATIM REVIEW OUTPUT

# SPEC R0 GATE REVIEW — `SPEC_seedhammer_codex32_input_polish.md` (Cycle A1)

- **Stage:** formal spec R0 architecture gate (mandatory before any code). Read-only.
- **Date:** 2026-06-17
- **Reviewer:** opus code-architect. Code read on fork `main` `3c4d3d3` (the post-Slice-1 base). BIP-93 re-verified vs `raw.githubusercontent.com/bitcoin/bips/master/bip-0093.mediawiki`.
- **Spec reviewed:** C1 (codex32 `Describe` + `ParsePrefix` + exported length consts) + C2 (error-class feedback) + C3 (window-model char counter + live field parse) + C4 (pre-engrave confirm, raw-index) + C5 (full-QWERTY keypad, static-dim b/i/o). A0 (Slice-1 merge) already on the base; multi-share recovery = Cycle B, out of scope.
- **Verdict:** **GREEN — 0 Critical / 0 Important.** Proceed to implementation. Three Minor items below are non-blocking but should be folded into the implementer's brief.

---

## 1. Did the spec correctly fold the design-review findings? — YES, all six

Verified each pre-spec finding against the code on `3c4d3d3`:

- **IMP-1 / C3 window model.** Re-verified the four gate constants directly: `shortCodeMinLength=48`, `shortCodeMaxLength=93`, `longCodeMinLength=125`, `longCodeMaxLength=127` (`codex32/codex32.go:41-44`), consumed by `New`'s length gate at `:101-106` (short window 48–93, long 125–127, everything else → `errInvalidLength`). BIP-93 confirms: short data part ≤ 93, long data part 96–124, **"a data part of 94 or 95 characters is never legal"**; 128-bit total ≈ 48; threshold `0` or `2`–`9` (`1` forbidden); identifier = 4 bech32 chars; index `s`/`S` = unshared secret. The firmware's 94–124 *total* dead zone and its narrow 125–127 long window (a strict subset of BIP-93's 99–127) are real, and the spec's "no single denominator" window model (`< 48` → "N chars"; 48–93 → short; 94–124 → "keep typing", not an error; 125–127 → long; `> 127` → too long) is exactly right. Widening the long gate is correctly declared out of scope (§2, §7).
- **IMP-3 / C4 raw index, not `Split()`.** Re-verified `Split()` at `codex32.go:394-400`: it does `t := p.threshold; if t == 0 { t = 1 }` — it genuinely remaps an unshared secret (threshold 0) to threshold 1, which BIP-93 forbids as an entered digit. The spec correctly routes C4's threshold/index display through `ParsePrefix`'s raw fields and explicitly avoids `Split()` (§4.4, §8). Confirmed the *engrave* path (`engraveObjectFlow` `case codex32.String:` `gui.go:1819-1826`) uses only `id` from `Split()` (`id, _, _ := scan.Split()`), so leaving `Split()` untouched is safe — exactly as the spec states.
- **IMP-2 / C1 `Describe` + private sentinels + fresh `ParsePrefix`.** Folded verbatim (§4.1, §8).
- **MIN-1 / C5 static dim (not `updateValidKeys`).** Folded; `updateValidKeys` (`gui.go:1016-1024`) confirmed to assume lowercase `a..z` via `idx := key.r - 'a'`, which would corrupt on the uppercased/digit codex32 keys. Spec mandates one-time `disabled = true` at construction (§4.5).
- **MIN-2 / C2 windowed timing.** Folded: suppress `errInvalidLength` as a non-error ("keep typing"); only `errInvalidChecksum` on a full valid-length string is a true error; field errors shown eagerly (§4.2). Verified `New` checks length *first* (`:101-106`) before checksum (`:116`), so this ordering holds.
- **MIN-3 / parse-once-per-keystroke.** Folded: one `New` + one `ParsePrefix` per `kbd.Update` iteration, results threaded (§4.2).

## 2. C1 implementability — CLEAN as specified

- **`ParsePrefix` is cleanly implementable fresh.** It must NOT reuse `partsInner` (`codex32.go:127`): that function indexes `res[0]`, `res[5]`, `res[1:5]` unconditionally and `panic("unreacable")` at `:159` if the share index isn't valid bech32 — both fatal on in-progress input. The spec's fresh approach using `splitHRP` (`codex32.go:453`, returns `("", p1)` when there's no `1`) + the non-panicking `feFromRune` (`gf32.go:126`, returns `(0, false)` out of range) is correct and panic-free.
- **Determinability offsets are correct.** Data part = everything after the first `1`. From `partsInner`: threshold = `res[0]`, id = `res[1:5]`, shareIdx = `res[5]`, payload = `res[6:len-checkLen]` (`:161-167`). So `ParsePrefix`'s offsets (threshold at len≥1, id at len≥5, share index at len≥6) match the canonical parser. Verified against BIP-93 vector 1 `ms10testsxxx…`: HRP `ms`, data `0testsxxx…` → threshold `0`, id `test`, index `s` (= unshared). Payload/checksum correctly never split (the boundary depends on the final total length / checksum length 13 vs 15, which is unknowable mid-entry) — matches BIP-93's checksum-length rule.
- **`Fields` struct is sufficient for C2/C3/C4.** It exposes HRP, Threshold(+Known), Identifier(+Known), ShareIndex(+Known), and the derived `Unshared` bool. C3 reads the `…Known` flags to show each field segment once available; C4 branches on `Unshared`. Nothing more is needed.
- **`Describe`'s sentinel mapping is complete and correct.** Traced every error `New` can return (all wrapped with `%w`, so `errors.Is` works): `errInvalidLength` (`:106`), `errInvalidCase` (`checksum.go:80,101`), `errInvalidCharacter` (`checksum.go:84,106`), `errInvalidChecksum` (`:117`), and via `sanityCheck`→`partsInner`: `errInvalidThreshold` (`:154`), `errInvalidShareIndex` (`:170`), `errIncompleteGroup` (`:56`). The spec's mapping lists exactly these seven; the unmapped sentinels (`errInsufficientShares`, `errMismatched*`, `errInvalidIDLength`, `errRepeatedIndex`) are `Interpolate`-only and never reach `New`, correctly left to the "invalid" fallback. **Concrete proof the `errors.Is` chain works:** the existing `TestBIPBadChecksums` (`codex32_test.go:185`) already asserts `errors.Is(New(...), errInvalidChecksum)` and passes — `Describe` rests on the same mechanism.
- **Exporting the 4 length consts is clean.** Straightforward aliasing (`const ShortCodeMinLength = shortCodeMinLength`, etc.). The values are stable BIP-93/firmware facts.

(One honesty note, not a defect: `inputHRP` `:92` returns a non-sentinel `fmt.Errorf("invalid character: %c", c)` that `errors.Is(_, errInvalidCharacter)` won't match — but that branch is unreachable for `New` because `feFromInt(c & 0x1f)` always yields 0..31, and the codex32 HRP is `ms`. `Describe`'s "unknown → invalid" fallback covers it harmlessly.)

## 3. C4 correctness — index s/S ⇔ unshared secret HOLDS

BIP-93: *"a share index value of 's' (or 'S') is special and denotes the unshared secret"* and *"if the threshold parameter is '0' then the share index … MUST have a value of 's' (or 'S')."* The code enforces the converse at `partsInner:169-171` (`threshold == 0 && shareIdx != feS` → `errInvalidShareIndex`). So a `New`-valid string with threshold 0 necessarily has index S, and any index-S string is the unshared secret. C4's branch (index S → "Unshared secret (S)"; else → "Share <index> … k-of-n") is correct, and it correctly reads the raw index from `ParsePrefix` rather than `Split()` (which would mislabel the unshared secret as "1-of-1"). The "engraves THIS share, not a recovered seed" note is accurate (recovery = Cycle B).

## 4. C5 feasibility — FEASIBLE, per-instance, full coverage

- `keyboardKey.disabled` exists (`gui.go:834`); `NewKeyboard` (`:839-892`) builds a fresh per-instance `allKeys`/`keys` from the alphabet string. Setting `disabled = true` on the b/i/o keys after the build loop is a clean one-time mutation.
- **Match on lowercase `'b'/'i'/'o'`:** keys store `r` directly from the (lowercase) alphabet string (`:865`); `rune()` uppercases only on *output* (`:1131`, `unicode.ToUpper`). So the dimming predicate is `key.r == 'b' || key.r == 'i' || key.r == 'o'`, and the C5 test correctly asserts on `kbd.allKeys` lowercase `r`.
- **Composes with `Valid()`/`adjust()`/`Update()`:** all D-pad nav (`:1063,:1077,:1089,:1099`), click handling (`:1039`), rune entry (`:1113`), and `adjust` (`:1143`) gate on `k.Valid(key)`, which returns `!key.disabled` (`:1031`). Disabled keys are skipped for navigation and render dimmed via `mulAlpha(bgcol, theme.inactiveMask)` in `Layout` (`:1203-1205`). `Clear()` (`:894`) resets only `Fragment`/`row`/`col` — it does **not** clear `disabled`, so static dimming survives. `inputCodex32Flow` never calls any `updateValid*Keys`, so nothing resets it.
- **Per-instance / no BIP-39 contamination:** each flow has its own `NewKeyboard` (BIP-39 `inputWordsFlow:540`, codex32 `inputCodex32Flow:675`), so dimming the codex32 instance cannot affect BIP-39. `TestWordKeyboardScreen` (`gui_test.go:277`) is preserved as the guard.
- **It DOES add b/i/o as new (dimmed) keys** — intended. The current codex32 keypad (`"1234567890\nqwertyup\nasdfghjk\nlzxcvnm"`, `:673`) omits b/i/o entirely; C5 switches to the BIP-39 full-QWERTY (`wordKeys = "qwertyuiop\nasdfghjkl\nzxcvbnm"`, `:537`) plus the digit row, then dims b/i/o for visual familiarity.
- **Full coverage confirmed:** `codex32.Alphabet = "QPZRY9X8GF2TVDW0S3JN54KHCE6MUA7L"` (`gf32.go:21`). Its letters {Q,P,Z,R,Y,X,G,F,T,V,D,W,S,J,N,K,H,C,E,M,U,A,L} and digits {0,2,3,4,5,6,7,8,9} are all present in `1234567890` + full QWERTY (all 26 letters), and the `1` separator is in the digit row. b/i/o (dimmed) are exactly the QWERTY letters absent from the Alphabet (bech32 excludes b/i/o/1). Coverage is complete.

## 5. Testability of C2/C3 via `runUI`+`ExtractText`+`uiContains` — TRUE, no seam needed

Verified the harness: `runUI` (`gui_test.go:466`) installs `ctx.FrameCallback`, which runs `d.ExtractText(r, o)` over the op tree passed to `ctx.Frame`, so any text drawn via `widget.Label*`/`layoutTitle*` inside the frame is read back. `inputCodex32Flow` renders through `ctx.Frame(op.Layer(...))` at `gui.go:722`, so C2/C3 status/field/error labels added to that layer will be captured. The Slice-1 precedent is exact: `TestWordFlowMatchCount` (`gui_test.go:502-528`) drives `runes(&ctx.Router, "abandon")` then asserts `uiContains(content, "1 match")` against the rendered count label — which `inputWordsFlow` builds via `widget.Labelf(...)` into `countOp` and layers into `ctx.Frame` (`gui.go:638-664`). C3 mirrors that pattern exactly. `uiContains` lowercases and strips spaces (`:479-484`), so "bad threshold", "keep typing", "id ", "thr ", "share " all match cleanly. The spec's claim (§6, §8) that no special test seam is required — superseding the pre-spec review's "add a seam" note — is **correct**, because the seam (`FrameCallback`/`ExtractText`) is now on the base via A0. The C1/C4/C5 test plans are also sound: BIP-93 vectors are available in-package (`codex32_test.go`: vector 1 `ms10test…` threshold-0/index-S, vector 3 `ms13cash…` shares, vector 5 long/127), giving every prefix C1's table tests and C4's confirm-screen assertions need; the 94–124 dead-zone "keep typing" case is testable by truncating the long vector.

## 6. Scope / consistency / implementability — sound, with three Minor refinements

**MINOR-1 — C4's confirm screen has no named home; §2's "`inputCodex32Flow` only" is too narrow.** The spec (§4.4) places the confirm screen "after accept, before engraving," but §2's Files list says `gui.go (inputCodex32Flow only)`. Today, accept returns the `codex32.String` from `inputCodex32Flow` to `newInputFlow`, and the engrave happens in `engraveObjectFlow`'s `case codex32.String:` (`gui.go:1819`). A pre-engrave confirm therefore lives most naturally either (a) at the end of `inputCodex32Flow` before it returns, or (b) in the `case codex32.String:` block before `backupSeedStringFlow`. Either is fine, but the spec should name the location and widen §2 to include whichever (the `case codex32.String:` block is the cleaner mirror of `descriptorFlow`/`SeedScreen.Confirm`). *Fix:* pick one site (recommend the `case codex32.String:` block at `gui.go:1819`, so the confirm has the parsed share in hand and `inputCodex32Flow` stays input-only) and update §2's Files list to match. Non-blocking — purely a placement clarification.

**MINOR-2 — C3 layout density (two new text lines) is unverified for vertical fit.** The current `inputCodex32Flow` band between title and keyboard holds only the fragment box (`top, _ := content.CutBottom(kbdsz.Y)`, `gui.go:707`). C3 adds a status line *and* a field line (three stacked rows per the ASCII at §4.3:91-98). `inputWordsFlow` fits exactly one extra count line and clamps it so it never overlaps the keyboard (`gui.go:645-648`). Two extra lines on the ~240px display is plausible but unconfirmed. *Fix:* the implementer must reuse the same clamp pattern (`if countY > top.Max.Y - csz.Y { countY = ... }`) for both new lines, and consider merging status+fields onto fewer rows if they don't fit. Tests assert on `ExtractText` content regardless of pixel position, so this is a visual-fit QA note, not a correctness blocker.

**MINOR-3 — `ParsePrefix` HRP-completeness and the threshold-0⇒S timing need a one-line clarification.** Two edge behaviors should be pinned so the implementer and test author agree: (a) `splitHRP` returns `("", whole)` until a `1` is typed, so `Fields.HRP` stays `""` and the data offsets are measured from the post-`1` substring only after the separator appears — the spec says this implicitly ("HRP on `1`") but should state that before the `1`, threshold/id/index are all `…Known == false` (the pre-`1` typed chars are HRP candidates, not data). (b) The threshold-0⇒S rule is only determinable once *both* `data[0]=='0'` (len≥1) and `data[5]` (len≥6) are present; between len 1 and 5 a leading `0` is not yet an error. The spec's "enforce threshold-0 ⇒ index s/S … at len≥6" is correct; just make the "not before len 6" explicit so the C2 eager-error test doesn't expect "bad share index" prematurely. Non-blocking; add one sentence to §4.1(c).

**No NEW Critical/Important issues.** Cross-checks that passed: `Split()` is genuinely untouched and its `id`-only use on the engrave path is unaffected; `mdmk.go`/`mdmkFlow` are correctly excluded (different scheme sharing only the GF(32) engine); the OK button remains gated solely on `New(...) == nil` with `ParsePrefix`/`Describe` advisory-only (§5) — so the partial parser can never widen acceptance; `TestWordKeyboardScreen` + `TestInputSeedCodex32` (`codex32_input_test.go`) stay green as guards; the uppercasing keypad makes mixed-case moot for the GUI while `ParsePrefix` still detects it for API honesty (correct).

---

## Verdict

**GREEN — 0 Critical / 0 Important.** The spec correctly and completely folds every pre-spec finding (C1–C5), every architectural claim checks out against the code on `3c4d3d3` and against BIP-93, and the testability story (host `go test ./gui/... ./codex32/...` via the existing `runUI`/`ExtractText`/`uiContains` seam + in-package BIP-93 vectors + `errors.Is`) is real. The three Minor items (name C4's confirm-screen site + widen §2's Files list; reuse `inputWordsFlow`'s clamp for C3's two lines; one clarifying sentence on `ParsePrefix` pre-`1`/threshold-0 timing) are refinements for the implementer's brief, not gate blockers.

**Proceed to implementation.** Recommended order matches the spec: C1 (codex32 API — the one-way-door crypto-pkg change) landed and host-tested first, then C2/C3 and C4 (consuming C1), then C5 (independent). Each task: single subagent + two-stage review, then the mandatory whole-diff adversarial execution review.

### Relevant files
- `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_seedhammer_codex32_input_polish.md` (the reviewed spec)
- `/scratch/code/shibboleth/seedhammer/codex32/codex32.go` (`New:98`, `partsInner:127`, `Split:394` remap, `splitHRP:453`, length consts `:41-44`, sentinels `:24-37`)
- `/scratch/code/shibboleth/seedhammer/codex32/gf32.go` (`feFromRune:126`, `Alphabet:21`, `feS` ordinal)
- `/scratch/code/shibboleth/seedhammer/codex32/checksum.go` (`inputHRP:77`, `inputChar:100`, `setCase:132`, the non-sentinel `:92`)
- `/scratch/code/shibboleth/seedhammer/codex32/codex32_test.go` (BIP-93 vectors; `errors.Is` precedent at `:185`)
- `/scratch/code/shibboleth/seedhammer/gui/gui.go` (`inputCodex32Flow:672`, `wordKeys:537`, `NewKeyboard:839`, `keyboardKey:832`, `updateValidKeys:1016`, `Valid:1027`, `Update:1034`, `rune:1125`, `adjust:1136`, `Layout:1189`, `engraveObjectFlow case codex32.String::1819`, `backupSeedStringFlow:1928`)
- `/scratch/code/shibboleth/seedhammer/gui/gui_test.go` (`runUI:466`, `uiContains:479`, `TestWordFlowMatchCount:502`, `TestWordKeyboardScreen:277`)
- `/scratch/code/shibboleth/seedhammer/gui/codex32_input_test.go` (`TestInputSeedCodex32` — keep green)


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | **GREEN — 0C / 0I** (3 non-blocking Minors) | Confirmed all six pre-spec findings folded (C1 length window, `Split()` 0->1 trap avoided via raw-index C4, `Describe`+private sentinels, `ParsePrefix` fresh/panic-free, C5 static-dim, C2 windowed timing, parse-once). Proved `Describe`'s `errors.Is` chain via existing `TestBIPBadChecksums` (codex32_test.go:185). Confirmed `runUI`+`ExtractText`+`uiContains` testability (no seam) via Slice-1 `TestWordFlowMatchCount`. **MINOR-1**: name C4 confirm site (`engraveObjectFlow case codex32.String:` gui.go:1819) + widen §2 Files. **MINOR-2**: reuse `inputWordsFlow` clamp for C3's two lines. **MINOR-3**: one sentence on `ParsePrefix` pre-`1` / threshold-0-at-len>=6 timing. |

All three Minors folded into `SPEC_seedhammer_codex32_input_polish.md` after this review. Verdict: **proceed to implementation** (order: C1 -> C2/C3/C4 -> C5; single subagent + two-stage review per task + mandatory whole-diff adversarial execution review).
