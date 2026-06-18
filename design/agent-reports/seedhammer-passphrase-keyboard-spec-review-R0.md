# Slice 2: passphrase-keyboard-widget — SPEC R0 GATE REVIEW — R0

- **Stage:** mandatory spec R0 gate (0C/0I before any code).
- **Spec reviewed:** `design/SPEC_seedhammer_passphrase_keyboard.md` (committed `d950654`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `a02890b611476cf42`), read-only vs fork `06b57f3`.
- **Outcome:** **NOT GREEN — 0 Critical / 2 Important + 6 Minor.** All folded; re-dispatched R1.

> NOTE: verbatim architect output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

# R0 GATE REVIEW — SPEC_seedhammer_passphrase_keyboard.md (Slice 2)

**Reviewed against:** fork `bg002h/seedhammer` @ `06b57f3`, `/scratch/code/shibboleth/seedhammer`
**Reviewer:** opus architect (adversarial, read-only)
**Date:** 2026-06-18

---

## Evidence gathered (file:line anchors)

All claims below are verified against the actual tree. Every cited line has been read.

---

## FINDINGS

### CRITICAL

None.

---

### IMPORTANT

**I-1: RuneEvent cross-page search is under-specified — the `runes("Ab") → Fragment == "Ab"` test will fail as written unless the implementation searches all pages.**

File/location: spec §4.5 + §6 ("RuneEvent case-honoring" test).

The spec says at §4.5: "a typed rune is matched/committed as-is (so a physical/host keyboard and the test harness preserve case)." It then includes a test: `runes(&ctx.Router, "Ab") → Fragment == "Ab"`.

The `runes()` helper (`gui/event_test.go:68-76`) sends `RuneEvent{Rune: 'A'}` then `RuneEvent{Rune: 'b'}` with NO lowercasing — the literal rune values are passed. In the new `PassphraseKeyboard`:

- Page 0 (lowercase) contains `'a'`–`'z'`, `'b'` is present.
- Page 1 (UPPERCASE) contains `'A'`–`'Z'`, `'A'` is present.
- Page 2 contains digits/symbols.

If the RuneEvent handler (like the current `Keyboard.Update` at `gui/gui.go:1131-1141`) searches only `k.keys[k.row]` or only the current active page, then sending `'A'` while on page 0 finds no match (page 0 has only lowercase). The rune is silently dropped and `Fragment` remains `""`. The test then asserts `Fragment == "Ab"` and fails.

The fix is one of:
1. Specify that RuneEvent searches ALL pages (not just the current page). When a rune matches a key on a different page, the widget auto-switches to that page, positions the cursor, and commits. This preserves the semantics and makes the test work.
2. Specify that RuneEvent searches only the current page — and change the test to explicitly switch to the correct page before each `runes` call (making the RuneEvent path page-aware but not page-switching). This is weaker (RuneEvent no longer accepts upper or lower regardless of active page) but consistent.
3. Specify that the test drives case preservation by explicit key-clicks + page-switches (as already noted in §6 "drive via explicit key clicks/page-switches since `runes` interacts with the RuneEvent path") — and REMOVE the `runes("Ab") → Fragment == "Ab"` test, replacing it with a click-based sequence.

The spec currently has both: it says "drive case via explicit key clicks/page-switches" (correct, avoids the issue) AND includes `runes("Ab") → Fragment == "Ab"` as a test (which requires cross-page search to work). These two statements are in direct tension. The plan author must pick exactly one model and the spec must commit to it before implementation.

Required fix: Either (a) add a sentence to §4.5 specifying that RuneEvent performs a cross-page search and switches to the matching page, OR (b) remove the `runes("Ab")` test from §6 and replace it with an explicit click-based case-preservation test. Option (b) is safer and consistent with the already-written guidance in §6 to use explicit key clicks.

---

**I-2: The function-row special-key struct and its interaction with `Valid`, `adjust`, and `adjustCol` is under-specified to the point where the implementer has a genuine fork in the implementation.**

File/location: spec §4.3.

The spec says the function row uses "a `keyboardKey`-like cell with a `label string` (and an action discriminator)." The existing `keyboardKey` (`gui/gui.go:854-859`) has `r rune`, `disabled bool`, `pos image.Point`, `clk Clickable`. It has NO `label` field and NO action discriminator.

The spec does not state:
1. Whether special-key cells will be a NEW private struct (e.g. `passphraseKey`) with a different action field, OR whether they repurpose sentinel rune values in `keyboardKey.r` (e.g. a magic rune like `'\x01'` for page-cycle), OR some other approach.
2. How `Valid()` works for function-row keys: the existing `Valid()` checks `key.r == '⌫'` specifically. A new standalone type with its own `rune()` equivalent needs its own `Valid` logic. For page-cycle and space: always valid. For reveal-toggle: always valid. For backspace: valid iff `Fragment` non-empty. This must be spelled out.
3. How `adjust()` / `adjustCol()` handle function-row keys. The existing `adjust` uses `key.r == '⌫' && !allowBackspace` as a sentinel. A new standalone type with a different discriminator must be specified.
4. Whether the function row is stored in the same `keys [][]keyboardKey` slice as the letter rows, or in a separate slice. The existing `Keyboard` has a flat `allKeys` backing array and `keys [][]keyboardKey`. If the function row is appended as another row in the same slice, the existing D-pad `Up/Down` logic (which iterates `k.keys`) will naturally include it. If it's separate, the implementer must handle it explicitly.

This is not a fatal design flaw — the decision space is bounded — but the spec must commit to ONE design before the plan, because this choice drives almost all the non-trivial implementation code (the struct definition, the Valid logic, the D-pad navigation, the Layout dispatch). Any of these designs is implementable; what is missing is the commitment.

Required fix: §4.3 must specify: (a) the struct representation of function-row entries (new struct type vs sentinel rune vs embedded label field in `keyboardKey`), (b) the `Valid` rule for each special-key type, and (c) whether the function row is in the same `keys` slice (recommended for D-pad continuity) or separate.

---

### MINOR

**M-1: `Clear()` semantics for `revealed` are internally contradictory in the spec.**

File/location: spec §4.1, the `Clear` signature comment: `"reset Fragment/page/cursor (NOT revealed→stays default-masked? reset to masked)"`.

The question mark followed by an answer is a residual draft artifact. The spec must commit: should `Clear()` reset `revealed` to `false` (masked) or leave it unchanged? From a UX standpoint, resetting to masked is safer (no accidental revelation after re-entry). This is easily resolved but must be made explicit before implementation. Not a blocker if the plan author commits to one in the plan, but the spec should be unambiguous.

**M-2: Page-cycle key label semantics are deferred with "exact labels finalized in the plan" but the spec simultaneously lists them.**

File/location: spec §4.3: `'"ABC"` on the lowercase page, `"?123"` on UPPER, `"abc"` on symbols — exact labels finalized in the plan.'`

The labels listed are `"ABC"`, `"?123"`, `"abc"`. These labels show the TARGET page (what you'll switch to). This is correctly stated and the pattern is standard. The minor issue is the hedge "finalized in the plan" — these look final enough. The plan must commit to these or alternatives; the hedge creates nominal ambiguity. The character count matters for the key cell width (the key cell is sized to fit `poppins.Bold25` single glyphs in `NewKeyboard`). A 4-char label like `"?123"` on `poppins.Bold25` will be wider than a single-char key cell. The plan must account for this: the function-row keys must be wider than the standard `widest = ctx.Styles.keyboard.Measure(math.MaxInt, "W")` cell, or the labels must be short enough to fit. The spec does not address function-row key sizing at all.

**M-3: The spec does not address function-row key geometry / sizing relative to the letter rows.**

File/location: spec §4.3, §4.4; cf. `gui/gui.go:866-913`.

The existing `NewKeyboard` sizes every key cell to `ctx.Styles.keyboard.Measure(math.MaxInt, "W")` (the widest expected single glyph). Multi-char labels like `"space"` (5 chars), `"ABC"` (3 chars), `"?123"` (4 chars), `"show"`/`"hide"` (4 chars) on `poppins.Bold25` will overflow a single-glyph cell. The plan must specify either (a) wider cells for the function row (e.g. sized to fit the longest label), or (b) abbreviated labels fitting within the existing key cell width. This is a real implementation detail the implementer will need to resolve. The spec says "exact labels finalized in the plan" but says nothing about how the cells will be sized to fit them.

This is not critical — the implementer can figure it out — but a plan that doesn't address it will produce either clipped labels or a function row with different cell sizes than the letter rows, which may look jarring or require additional layout code.

**M-4: Space glyph extraction behavior — `uiContains` space-stripping subtlety.**

File/location: spec §6 mask test; `gui/gui_test.go:479-484`; `cmd/bitmapfont/main.go:166-170`.

`uiContains` lowercases both strings AND strips spaces from the search string (`strings.ReplaceAll(strings.ToLower(str), " ", "")`), but does NOT strip spaces from the content. Space (0x20) IS added to the generated font alphabet by `bitmapfont/main.go:166-170` — however, `op.Glyph(buf, face, ' ')` at `op.go:124-132` calls `face.Glyph(' ')`: if space has no rendered pixels (zero-size bitmap), `ok=false` is returned and the `MaskOp{}` no-op is stored. The glyph is never added to `d.text` in `ExtractText` (requires `*glyph` materialization). This means spaces in `Fragment` will not appear in `ExtractText` output. This is fine for the mask test (searching `"****"` vs asterisks-only content), but the spec's cleartext reveal test (`uiContains(content, cleartext)`) will fail if `Fragment` contains a space, because `content` won't have a space in it (no space glyph collected) but the search string has its space stripped anyway by `uiContains` — actually that makes it pass. This subtlety is benign for the tests as written (fragment with spaces: search strips the space from the pattern, content has no space, but the rest of the characters match). Document this non-issue so the implementer doesn't spend time debugging it.

Actually re-reading: `uiContains(content, str)` strips spaces from `str` only, not from `content`. If `Fragment` is `"ab cd"` and revealed, `ExtractText` yields `"abcd"` (no space). `uiContains("abcd", "ab cd")` = search for `"abcd"` in `"abcd"` = match. Fine. So this is truly benign. Noting it for clarity.

**M-5: The spec omits the `keyPadX`/`keyPadY`/`keyCornerRadius`/`keyLineWidth` constants** (used in the existing layout at `gui/gui.go:871-876`, `1247-1256`) from the list of reusable helpers. These are package-private (`gui`) and accessible from `passphrase_keyboard.go`, but the spec should list them as reusable to ensure the implementer finds them rather than re-inventing sizing constants.

**M-6: The spec says `Layout` signature mirrors `Keyboard` but does not address the readout height.**

File/location: spec §4.4.

The existing `Keyboard.Layout` returns `(op.Op, image.Point)` where the `image.Point` is `k.size` (just the keyboard extent, no readout). The spec says `PassphraseKeyboard.Layout` includes the readout above the key grid. If `Layout` returns the combined size (readout + key grid), callers in Slice 3 must be aware of this. If it returns only the key grid size (and the readout is rendered at a fixed offset), the caller must know where to place the readout. The spec does not clarify which `image.Point` the new `Layout` returns. This should be specified so the Slice 3 wiring is unambiguous.

---

## Verification Summary (per review task §1–7)

**§1 — New standalone type decision:**
CONFIRMED. `gui/gui.go:1153` (`unicode.ToUpper(r)`) and `gui/gui.go:1244` (`widget.Labelf(…, unicode.ToUpper(key.r))`) both force uppercase. Three live consumers: `inputWordsFlow` (`:539`), `inputCodex32Flow` (`:672`, via `newCodex32Keyboard` at `codex32_polish.go:228`), `inputSLIP39Flow` (`:756`). New standalone type is the correct call. Low-level helpers (`keyboardKey`, `mulAlpha`, `theme.inactiveMask`, `assets.KeyBackspace`, `Clickable`, `InputTracker`, D-pad constants, `widget.Labelf`/`Labelw`, `ctx.Styles.keyboard`/`.word`) are all accessible from a new file in package `gui`. Standalone Layout/rune() is the correct approach vs embedding (embedding inherits the inner `ToUpper`).

**§2 — Font/charset reality:**
CONFIRMED. `poppins/gen.go` and `comfortaa/gen.go` both use the default `-alphabet` (`cmd/bitmapfont/main.go:32` = full printable ASCII `!"#$%…~`). All spec page-2 symbols (`1234567890-/:;()&$@".,?!'+=_#`) are in the default printable-ASCII set. `•` (U+2022) is above `~` (0x7E), outside printable ASCII, NOT in the font — `*` (0x2A) IS present. Space (0x20) IS added to the generated font's Index (via `bitmapfont/main.go:166-170`) with its TTF advance, but has no rendered pixels — it is transparent in rendering and not collected by `ExtractText`. All font claims in the spec are accurate.

**§3 — Mask/reveal placement and testability:**
CONFIRMED with a caveat. `ExtractText` (`op.go:523-534`) collects `img.r` from every `*glyph` mask in the draw stack (`op.go:327-334`). `*` is a standard printable-ASCII glyph with real pixels, so it IS collected. `uiContains(content, "****")` searches for 4+ consecutive `*` chars in lowercased content — this works. The `widget.Labelw` call for the readout goes through `text.Layout` → `op.Glyph` → `*glyph` struct → collected. The masking approach (string substitution before `Labelw`) is correct; no new rendering primitives are needed.

**§4 — Case-honoring input and testability:**
PARTIALLY CONFIRMED. The `runes()` helper sends literal rune values with no lowercasing (confirmed `event_test.go:68-76`). The new widget's RuneEvent path NOT doing `unicode.ToLower` is the correct call. However, the `runes("Ab") → Fragment == "Ab"` test requires RuneEvent to find `'A'` on page 1 and `'b'` on page 0, which requires cross-page search. This is under-specified (I-1 above). The special-key label model (`widget.Labelf("%s", label)` for multi-char labels) is implementable using existing primitives.

**§5 — API shape for Slice 3:**
CONFIRMED adequate. The `Update`/`Layout`/`Clear`/`Fragment` mirror makes the widget drop-in compatible with an `inputCodex32Flow`-style host. The under-specifications flagged (D-pad nav function-row crossing, `Clear` + `revealed`, `Layout` return size with readout) are real but Slice 3 can work around them once the plan commits; they are not blockers for Slice 2's standalone deliverable. Backspace-valid-iff-Fragment-non-empty is clear from the existing precedent.

**§6 — Scope/slice integrity:**
CONFIRMED. No `MnemonicSeed`/`deriveMasterKey` changes needed. `MnemonicSeed(m,"")` at `gui.go:188` untouched. An unused exported type in Go is not a compilation error. The new `PassphraseKeyboard` type touches none of the existing `Keyboard` consumers; `TestWordKeyboardScreen`, `TestInputSeedCodex32`, SLIP-39 tests, `TestWordFlow*` remain green. `codex32_polish.go`, `slip39_polish.go`, `mdmk_gui_test.go` untouched.

**§7 — Under-specified or wrong items blocking the plan:**
The two Important findings (I-1 and I-2) are the blockers. Neither is a fatal design flaw; both are resolvable with one or two clarifying sentences added to the spec. The Minor findings are real but would not block a competent implementer from making reasonable choices in the plan.

---

## VERDICT

**NOT YET GREEN — 0 Critical / 2 Important / 6 Minor**

The two Important findings must be resolved before the plan is written:

- **I-1:** Commit to ONE RuneEvent model: either cross-page search (spec §4.5 updated to say so), or remove the `runes("Ab")` test and use explicit key-click tests only (consistent with the guidance already in §6).
- **I-2:** Specify the special-key struct representation (new type vs sentinel rune vs embedded field), the `Valid` rule per function-row key type, and whether the function row lives in the same `keys` slice as letter rows.

Fold these into the spec, re-dispatch to this review. The Minors (M-1 through M-6) should also be resolved in the fold for a clean plan — particularly M-3 (function-row cell sizing) which the plan must address anyway.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | NOT GREEN — 0C/2I/6m | **I-1** RuneEvent model contradictory (`runes("Ab")` test needs cross-page search, §4.5 didn't specify) → folded: §4.5 now commits to **cross-page case-honoring RuneEvent, no auto-switch** (makes `runes("Ab1!")`→"Ab1!" work; touch entry is page-scoped). **I-2** function-row special-key struct/Valid/D-pad under-specified → folded: §4.3 now defines the private `ppKey{r,label,action,…}` + `ppAction` enum, per-action `Valid`/`Layout`/`Update` dispatch, same-`keys`-slice traversal, and function-row cell sizing. Minors folded: M-1 Clear re-masks (revealed=false); M-2/M-3 function-row cells sized to labels (own widths, keyPad consts); M-4 space-glyph/uiContains note; M-6 Layout returns combined readout+grid extent; M-5 keyPad consts cited. Architect confirmed the new-standalone-type decision, font printable-ASCII coverage (`*` not `•`), in-widget masked readout testability, and slice integrity (no MnemonicSeed/flow/fingerprint). |

Re-dispatched R1 after the fold.
