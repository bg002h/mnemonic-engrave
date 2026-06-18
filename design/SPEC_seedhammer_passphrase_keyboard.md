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
func (k *PassphraseKeyboard) Layout(ctx *Context, th *Colors) (op.Op, image.Point) // readout + active page + function row
func (k *PassphraseKeyboard) Clear()                          // reset Fragment/page/cursor (NOT revealed→stays default-masked? reset to masked)
```
The API mirrors `Keyboard` so Slice 3 wires it into a flow (title + Back/OK nav) exactly like `inputCodex32Flow` wires `Keyboard`.

### 4.2 Three case-preserving pages

- **page 0 (lowercase):** `"qwertyuiop\nasdfghjkl\nzxcvbnm"`
- **page 1 (UPPERCASE):** `"QWERTYUIOP\nASDFGHJKL\nZXCVBNM"`
- **page 2 (symbols+digits):** `"1234567890\n-/:;()&$@\"\n.,?!'+=_#"` (printable-ASCII; the exact glyph set is finalized in the plan — all are font-present).

A normal key stores its literal rune `r`; **`rune()` appends `string(r)` with NO `ToUpper`**, and the **render uses `widget.Labelf(..., "%c", key.r)` with NO `ToUpper`** — so lowercase 'a' commits/renders as 'a', UPPERCASE 'A' as 'A'. Case is page-determined (the literal in the page's grid), not a shift on commit.

### 4.3 Function row (shared across pages)

A bottom row of special keys, rendered by a short text label / icon (not a single ASCII glyph):
- **page-cycle** — cycles `0→1→2→0`; its cap shows the *target* page (`"ABC"` on the lowercase page, `"?123"` on UPPER, `"abc"` on symbols — exact labels finalized in the plan).
- **space** — appends `' '`; cap a short label (e.g. `"space"`).
- **reveal** — toggles `revealed`; cap toggles between a "show"/"hide" label (ASCII text — no eye-icon dependency).
- **backspace** — `⌫` via `assets.KeyBackspace` (reuse the existing asset); deletes the last rune of `Fragment`.

Special keys are a `keyboardKey`-like cell with a `label string` (and an action discriminator) instead of a single `r`; the standalone `Layout` renders them via `widget.Labelf("%s", label)` (or the backspace image), and `Update` dispatches their action. D-pad nav + touch + `Center`-commit work for them as for normal keys.

### 4.4 Masked entry readout

Above the key grid, render the entry: when `!revealed`, `strings.Repeat("*", utf8.RuneCountInString(k.Fragment))`; when `revealed`, `k.Fragment` verbatim — via `widget.Labelw(&ctx.B, ctx.Styles.word, …)`. `*` is font-safe (NOT `•`). The widget owns this readout so it is a complete, self-contained passphrase-entry component (Slice 3 just frames it with title + Back/OK).

### 4.5 Input (case-honoring)

`Update` handles the same three modalities as `Keyboard` — touch (`clk.Clicked` per visible key, gated on validity), D-pad (`Left/Right`/`Up/Down` nav skipping invalid keys) + `Center` (commit cursor key), and `RuneEvent` — **but RuneEvent does NOT `ToLower`**: a typed rune is matched/committed as-is (so a physical/host keyboard and the test harness preserve case). Backspace valid iff `Fragment` non-empty.

## 5. Error handling / backstops

Pure UI widget — no crypto, no secret derivation (Slice 3 does that). It only accumulates a string. The passphrase never leaves the widget except as `Fragment` for the (Slice-3) flow; it is never engraved. No NFC.

## 6. Testing (host: `go test ./gui/...`)

Drive the widget directly (no flow) via `runUI` + `frame()` + `uiContains`, `runes`/`click`/`press`:
- **Case preservation:** enter `"Ab1!"` (lowercase 'a' on page 0, then page-cycle to UPPER for 'B'? — drive via explicit key clicks/page-switches since `runes` interacts with the RuneEvent path) → assert `Fragment == "Ab1!"` (NOT uppercased).
- **Page cycle:** click the page-cycle key, assert the active page changed (e.g. after two cycles the symbols page is active and `'1'` is a present/valid key; the letter rows are gone).
- **Mask/reveal:** with a non-empty Fragment, assert the readout renders `"****"`-style masking (`uiContains(content, "****")`) when masked; click reveal; assert the cleartext appears; click again; masked again.
- **Space + backspace:** space appends `' '`; backspace removes the last rune.
- **RuneEvent case-honoring:** `runes(&ctx.Router, "Ab")` → `Fragment == "Ab"` (proves no ToLower/ToUpper).
- Standalone — no flow consumer. The shared-`Keyboard` guard tests (`TestWordKeyboardScreen`, `TestInputSeedCodex32`, SLIP-39, `TestWordFlow*`) must stay green (untouched type).

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
