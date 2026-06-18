# SPEC — SeedHammer passphrase keyboard widget (Slice 2)

**Date:** 2026-06-18
**Target repo:** the SeedHammer II firmware fork `bg002h/seedhammer` (Go/TinyGo). Fork-side only — **no upstream PR**.
**Base:** fork `main` `06b57f3` (post-Cycle-A1/B/C). Branch `feat/passphrase-keyboard` off `06b57f3`.
**Predecessors:** `design/cycle-prep-recon-passphrase-keyboard.md` (verified vs `06b57f3`), `design/RECON_seedhammer_input_ux.md` (the 3-slice decomposition; Slice 1 done).
**Slice boundary:** this is the **WIDGET only**. The passphrase entry FLOW, threading `password` through `deriveMasterKey`/`bip39.MnemonicSeed`, both-fingerprints verification, and the user-chooses-which-fingerprint-to-engrave decision are all **Slice 3**. `MnemonicSeed(m, "")` at `gui.go:188` stays untouched here.

---

## 1. Goal

A reusable on-device GUI widget for entering a BIP-39 passphrase: a **case-preserving** 3-page keyboard (lowercase / UPPERCASE / symbols+digits) with a **masked-by-default entry readout** and a reveal toggle. The shared `Keyboard` widget force-uppercases (at both commit `gui.go:1153` and render `gui.go:1244`) and is single-page + shared by three live flows (BIP-39/codex32/SLIP-39), so this is a **new, isolated `PassphraseKeyboard` type** — not an extension of `Keyboard`. The passphrase string is NEVER engraved (only its fingerprint, in Slice 3); the widget exists to capture it on the air-gapped touchscreen.

## 2. Scope

**In:**
- A new `gui/passphrase_keyboard.go` defining `PassphraseKeyboard` (state: `Fragment string`, `page int`, `revealed bool`) with: 3 case-preserving page alphabets; a function row (page-cycle, space, reveal-toggle, backspace); `NewPassphraseKeyboard`, `Update`, `Layout`, `Clear` (mirroring the `Keyboard` API shape so Slice 3 can drop it into a flow like `inputCodex32Flow`); a masked/revealed entry readout. Case-honoring (no `ToUpper`/`ToLower`).
- Unit tests in `gui/passphrase_keyboard_test.go` (standalone — no flow consumer yet).

**Out:** the passphrase flow + `password` threading + fingerprint verify/choice (Slice 3); any change to the shared `Keyboard` / BIP-39 / codex32 / SLIP-39 flows; NFC; engraving the passphrase. `codex32`/`mdmk.go`/`slip39` untouched.

**Files:** `gui/passphrase_keyboard.go` (new) + `gui/passphrase_keyboard_test.go` (new). May read (not modify) the shared `Keyboard`/`keyboardKey` for reuse of low-level helpers (`mulAlpha`, `theme.inactiveMask`, `assets.KeyBackspace`, the `pos` layout math). The shared `Keyboard` and `TestWordKeyboardScreen`/`TestInputSeedCodex32`/SLIP-39 tests must stay green (a new type touches none of them).

## 3. Background (vs `06b57f3`)

- Shared `Keyboard` (`gui.go:840`), `NewKeyboard` (`:861`), `keyboardKey{r,disabled,pos,clk}` (`:853`), `rune()` force-uppercases on commit (`:1153`), `Layout` force-uppercases the key-cap (`:1244`) + dims via `mulAlpha(bgcol, theme.inactiveMask)` (`:1226`), `Update` reads touch (`clk.Clicked`), D-pad+`Center` (commit), and `RuneEvent` (lowercased at `:1132`). `wordKeys = "qwertyuiop\nasdfghjkl\nzxcvbnm"` (`:537`); `codex32Keys` + per-key static-dim precedent (`codex32_polish.go:222`).
- `bip39.MnemonicSeed(m, password string) []byte` (`bip39/bip39.go:217`) exists; the GUI hardwires `password=""` at `gui.go:188` (Slice 3 threads it).
- Fonts: `Styles.keyboard = poppins.Bold25`, `Styles.word = comfortaa.Bold17`; both generated over the full printable-ASCII alphabet (`cmd/bitmapfont/main.go:32`) — **every letter/digit/symbol glyph is available, no font regen.** `•` (U+2022) is NOT printable-ASCII → not in the font → **mask with `*`**.
- Test harness: `runUI`/`frame()`/`ExtractText`/`uiContains` (`gui_test.go`/`op.go:523`); `runes`/`click`/`press` (`event_test.go`).

## 4. Design

### 4.1 A new standalone `PassphraseKeyboard` (not an extension of `Keyboard`)

Rationale (recon): the shared `Keyboard` is single-page and force-uppercases at both commit and render; it has 3 live consumers. A new type isolates the regression surface and gives clean case semantics. Reuse the low-level layout math + `keyboardKey` cell model + `theme.inactiveMask` + `assets.KeyBackspace`, but with its own `rune()`/`Layout` that do NOT `ToUpper`, plus page state, a function row, and a masked readout.

```go
type PassphraseKeyboard struct {
	Fragment string // the entered passphrase (case preserved)
	page     int    // 0=lowercase, 1=UPPERCASE, 2=symbols+digits
	revealed bool    // false → readout shown masked as '*'
	// per-page key grids + the shared function row, plus row/col cursor,
	// InputTracker, layout extents — reusing the keyboardKey cell model.
}

func NewPassphraseKeyboard(ctx *Context) *PassphraseKeyboard
func (k *PassphraseKeyboard) Update(ctx *Context) bool        // drains input; returns true while it consumed events
func (k *PassphraseKeyboard) Layout(ctx *Context, th *Colors) (op.Op, image.Point) // readout + active page + function row; the returned image.Point is the COMBINED extent (readout height + grid), so the Slice-3 flow places the whole widget as one block (R0 M-6)
func (k *PassphraseKeyboard) Clear()                          // reset Fragment="", page=0, cursor to center, AND revealed=false (re-mask — R0 M-1)
```
The API mirrors `Keyboard` so Slice 3 wires it into a flow (title + Back/OK nav) exactly like `inputCodex32Flow` wires `Keyboard`.

### 4.2 Three case-preserving pages

- **page 0 (lowercase):** `"qwertyuiop\nasdfghjkl\nzxcvbnm"`
- **page 1 (UPPERCASE):** `"QWERTYUIOP\nASDFGHJKL\nZXCVBNM"`
- **page 2 (symbols+digits):** `"1234567890\n-/:;()&$@\"\n.,?!'+=_#"` (printable-ASCII; the exact glyph set is finalized in the plan — all are font-present).

A normal key stores its literal rune `r`; **`rune()` appends `string(r)` with NO `ToUpper`**, and the **render uses `widget.Labelf(..., "%c", key.r)` with NO `ToUpper`** — so lowercase 'a' commits/renders as 'a', UPPERCASE 'A' as 'A'. Case is page-determined (the literal in the page's grid), not a shift on commit.

### 4.3 Function row (shared across pages)

A bottom row of special keys, present identically on all 3 pages (so navigation/layout is uniform):
- **page-cycle** — cycles `0→1→2→0`; cap shows the *target* page: `"ABC"` (on the lowercase page) / `"?123"` (on UPPER) / `"abc"` (on symbols).
- **space** — appends `' '`; cap `"space"`.
- **reveal** — toggles `revealed`; cap `"show"` when masked / `"hide"` when revealed.
- **backspace** — `⌫` via `assets.KeyBackspace` (reuse the existing asset); deletes the last rune of `Fragment`.

**Key representation (R0 I-2 — committed):** the widget defines its OWN private cell struct (NOT the shared `keyboardKey`), e.g.
```go
type ppAction int
const (ppRune ppAction = iota; ppBackspace; ppPageCycle; ppSpace; ppReveal)
type ppKey struct {
	r        rune        // the literal char (ppRune only; case as-stored)
	label    string      // multi-char cap for special keys ("ABC"/"space"/"show"…)
	action   ppAction
	disabled bool
	pos      image.Point
	clk      Clickable
}
```
All three page grids are `[][]ppKey` sharing the SAME function row as their last row, stored in the active page's `keys [][]ppKey` slice — so the existing D-pad `Up/Down` (iterate rows) + `Left/Right` (within a row) traversal naturally crosses from the letter rows into the function row, exactly as the shared `Keyboard` does. **`Valid(k ppKey)`:** `ppBackspace` → `Fragment != ""`; `ppRune`/`ppPageCycle`/`ppSpace`/`ppReveal` → `!k.disabled` (always true here — nothing is disabled in this widget). **`Layout` dispatch:** `ppRune` → `widget.Labelf(..., "%c", k.r)` (NO `ToUpper`); `ppBackspace` → the `assets.KeyBackspace` image; the rest → `widget.Labelf(..., "%s", k.label)`. **`Update` dispatch** on `Center`/click/Valid: `ppRune`→append `string(r)`; `ppSpace`→append `' '`; `ppBackspace`→trim last rune; `ppPageCycle`→`page=(page+1)%3` + re-seed the cursor; `ppReveal`→toggle `revealed`.

**Function-row cell sizing (R0 M-2/M-3):** the special-key caps (`"?123"`,`"space"`,`"show"`) are wider than a single `poppins.Bold25` glyph cell. The function row is laid out with its own per-cell widths sized to each label (`ctx.Styles.keyboard.Measure(math.MaxInt, label)` + `keyPadX`), independent of the fixed-width letter cells — not forced into the letter-row cell width. Reuse `keyPadX`/`keyPadY`/`keyCornerRadius`/`keyLineWidth` (`gui.go:871-876`).

### 4.4 Masked entry readout

Above the key grid, render the entry: when `!revealed`, `strings.Repeat("*", utf8.RuneCountInString(k.Fragment))`; when `revealed`, `k.Fragment` verbatim — via `widget.Labelw(&ctx.B, ctx.Styles.word, …)`. `*` is font-safe (NOT `•`). The widget owns this readout so it is a complete, self-contained passphrase-entry component (Slice 3 just frames it with title + Back/OK).

### 4.5 Input (case-honoring)

`Update` handles the same three modalities as `Keyboard` — touch (`clk.Clicked` per visible key, gated on validity), D-pad (`Left/Right`/`Up/Down` nav skipping invalid keys) + `Center` (commit cursor key), and `RuneEvent`. Backspace valid iff `Fragment` non-empty.

**RuneEvent — case-honoring, CROSS-PAGE search, no auto-switch (R0 I-1 — committed):** a typed rune is committed AS-IS (no `ToLower`, no `ToUpper`), matched against ALL three pages' `ppRune` keys (not just the active page) — so `'A'` (only on the UPPER page) commits even while page 0 is active, and `'1'`/`'!'` (symbols page) likewise. The committed rune is appended to `Fragment` with its literal case; the **active page is NOT switched** (the display only changes via the page-cycle key). A rune not present on any page (e.g. a non-ASCII char outside the pages) is ignored. This makes `runes(&ctx.Router, "Ab1!")` → `Fragment == "Ab1!"` work directly and is the natural "just type the char" behavior for a host/physical keyboard. (Touch/D-pad entry is inherently page-scoped — you can only click/navigate to keys on the visible page — so case there is driven by switching pages, which the click-based tests in §6 exercise.)

## 5. Error handling / backstops

Pure UI widget — no crypto, no secret derivation (Slice 3 does that). It only accumulates a string. The passphrase never leaves the widget except as `Fragment` for the (Slice-3) flow; it is never engraved. No NFC.

## 6. Testing (host: `go test ./gui/...`)

Drive the widget directly (no flow) via `runUI` + `frame()` + `uiContains`, `runes`/`click`/`press`:
- **Case preservation via RuneEvent (cross-page):** `runes(&ctx.Router, "Ab1!")` → assert `Fragment == "Ab1!"` (NOT uppercased) — exercises the cross-page case-honoring RuneEvent path (§4.5).
- **Case preservation via touch/clicks:** on page 0 click `'a'`; page-cycle to UPPER, click `'B'`; page-cycle to symbols, click `'1'` then `'!'` → assert `Fragment == "aB1!"` — proves page-scoped click entry preserves the page's literal case.
- **Page cycle:** click the page-cycle key, assert the active page changed (e.g. after two cycles the symbols page is active and `'1'` is a present/valid key; the letter rows are gone).
- **Mask/reveal:** with a non-empty Fragment, assert the readout renders `"****"`-style masking (`uiContains(content, "****")`) when masked; click reveal; assert the cleartext appears; click again; masked again.
- **Space + backspace:** space appends `' '`; backspace removes the last rune.
- **RuneEvent case-honoring:** `runes(&ctx.Router, "Ab")` → `Fragment == "Ab"` (proves no ToLower/ToUpper).
- Standalone — no flow consumer. The shared-`Keyboard` guard tests (`TestWordKeyboardScreen`, `TestInputSeedCodex32`, SLIP-39, `TestWordFlow*`) must stay green (untouched type).
- **Test note (R0 M-4):** the space glyph (U+0020) renders with no pixels, so `ExtractText` does not collect it; `uiContains` also strips spaces from its needle — so a revealed `Fragment` containing spaces still matches its space-stripped cleartext. Benign; don't chase a missing-space "bug" in the extracted text.

## 7. Versioning / commits

Firmware `-ldflags`-injected (no source bump; additive widget). Commits on `feat/passphrase-keyboard`, signed (SSH) + DCO, author Brian Goss. Fork-side; no upstream PR (Slice 3 — the flow — is the natural PR boundary if ever).

## 8. Resolved decisions

- **New standalone `PassphraseKeyboard`** (not extending the shared `Keyboard`) — recon recommendation; isolates the 3 live consumers from regression, clean case semantics.
- **3 pages** lowercase/UPPERCASE/symbols+digits, cycled by an on-keyboard page-cycle key (self-contained; no flow-button coupling).
- **Case-preserving**: `rune()` + render drop `ToUpper`; RuneEvent drops `ToLower`.
- **Mask with `*`** (font-safe; `•` is absent from poppins) + a reveal-toggle key; masked by default.
- **Widget-only**: no flow, no `MnemonicSeed`/`password` threading, no fingerprint logic (all Slice 3). No upstream PR.
- Symbol-page exact charset, special-key labels, and space-key rendering finalized in the plan (all glyphs font-present).

## 9. Process note

Per project standard: this spec MUST pass the opus-architect **R0 gate to 0C/0I before any code** (fold → persist verbatim to `design/agent-reports/` → re-dispatch until GREEN). Then plan → plan R0 → single-implementer subagent TDD → mandatory whole-diff adversarial execution review. Proceeding autonomously (user directive).
