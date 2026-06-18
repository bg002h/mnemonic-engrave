# cycle-prep recon — 2026-06-18 — passphrase-keyboard-widget (Slice 2)

**Source repo:** SeedHammer fork `bg002h/seedhammer` (`/scratch/code/shibboleth/seedhammer`), branch `main` @ **`06b57f3`** (post-Cycle-A1/B/C).
**Predecessor:** `design/RECON_seedhammer_input_ux.md` (3-slice decomposition; line numbers drifted — refreshed below).
**Phase:** ultracode recon. Feeds the Slice-2 spec; the R0 gates follow. **Recommendation: build a NEW standalone `PassphraseKeyboard` type** (the shared `Keyboard` is single-page + force-uppercase at commit AND render, used by 3 live flows — extending it ripples; a new type isolates regression). Mask with `*` (poppins lacks `•`); no font regen for an ASCII symbol page; `MnemonicSeed(m,"")` threading is Slice 3.

---

# Recon — Slice 2: passphrase keyboard widget (vs fork `06b57f3`)

**Repo:** `/scratch/code/shibboleth/seedhammer` @ `06b57f3` (post Cycle-A1/B/C; SLIP-39 now merged & live). Go `/home/bcg/.local/go/bin/go`. Baseline keyboard/flow tests pass (`TestWordKeyboardScreen`, `TestInputSeedCodex32`, `TestWordFlow*` all green). RECON ONLY.

**Stale-recon citation audit (the prompt-cited lines from `RECON_seedhammer_input_ux.md`):** that doc's line numbers are uniformly DRIFTED but structurally accurate. New anchors: `MnemonicSeed(m,"")` is `gui.go:188` (still ACCURATE), `rune()` ToUpper is now `gui.go:1153` (recon said `:1030` — DRIFTED), `inputWordsFlow` is `gui.go:539` (recon said `:539` originally but later `:612` — file moved), `NewKeyboard` is `gui.go:861` (recon said `:790` — DRIFTED), `masterFingerprintFor` is `gui.go:482` (ACCURATE), `deriveMasterKey` is `gui.go:187` (ACCURATE).

---

## 1. The current `Keyboard` widget (vs `06b57f3`)

All keyboard internals live in `gui/gui.go`. **Single-alphabet, single-page, force-uppercase. There is NO existing multi-page / shift / symbol-page mechanism** — Slice 2 must add page-switching from scratch.

**`wordKeys` (BIP-39/SLIP-39 alphabet) — `gui.go:537`** — ACCURATE:
```go
const wordKeys = "qwertyuiop\nasdfghjkl\nzxcvbnm"
```

**`codex32Keys` + `newCodex32Keyboard` — `gui/codex32_polish.go:222,228`** — ACCURATE (post-A1; recon's `gui.go:624` is STRUCTURALLY-WRONG — the codex32 keypad moved to its own file and gained a digit row):
```go
const codex32Keys = "1234567890\nqwertyuiop\nasdfghjkl\nzxcvbnm"
func newCodex32Keyboard(ctx *Context) *Keyboard {
	kbd := NewKeyboard(ctx, codex32Keys)
	for i := range kbd.allKeys {
		switch kbd.allKeys[i].r {
		case 'b', 'i', 'o':
			kbd.allKeys[i].disabled = true
		}
	}
	return kbd
}
```
So a precedent for per-key static dimming already exists, but NOT for page-switching or case.

**`Keyboard` / `keyboardKey` structs — `gui.go:840-859`** — ACCURATE:
```go
type Keyboard struct {
	Fragment string
	keys      [][]keyboardKey
	widest    image.Point
	backspace image.Point
	size      image.Point
	row, col int
	inp      InputTracker
	allKeys []keyboardKey
}
type keyboardKey struct {
	r        rune
	disabled bool
	pos      image.Point
	clk      Clickable
}
```
Note the field set differs from the prompt's expected list: there is **no `Fragment`-embedded sub-widget, no `size`-vs-`widest` confusion** — `widest` is the per-key glyph cell, `size` is the whole-keyboard extent, `backspace` is the wider backspace cell. `keyboardKey` has **no `clk` named differently** — it is `clk Clickable`. The prompt's guess ("`size`, `widest`, `backspace`") is ACCURATE.

**`NewKeyboard(ctx, alphabet)` — `gui.go:861-914`** — ACCURATE. Body: appends `"⌫\n"` to the alphabet, measures key cell from `ctx.Styles.keyboard.Measure(…,"W")`, splits the alphabet on `\n` into rows (`k.keys` is a `[][]keyboardKey` slicing the flat `k.allKeys` backing array — *shared backing array*, important for dimming), centers each row, and the **last row's backspace is excluded from centering** (`if i==len(k.keys)-1 { n-- }`). `k.Clear()` seeds the cursor to the middle key.

**`rune()` — the commit path — `gui.go:1147-1155`** — CONFIRMED force-uppercase (the load-bearing assumption Slice 2 breaks):
```go
func (k *Keyboard) rune() {
	r := k.keys[k.row][k.col].r
	if r == '⌫' {
		_, n := utf8.DecodeLastRuneInString(k.Fragment)
		k.Fragment = k.Fragment[:len(k.Fragment)-n]
	} else {
		k.Fragment = k.Fragment + string(unicode.ToUpper(r))   // <-- forces UPPER on commit
	}
}
```

**`Layout()` ALSO force-uppercases at render — `gui.go:1244`** (second break point, easy to miss):
```go
keyOp, sz = widget.Labelf(&ctx.B, style, col, "%c", unicode.ToUpper(key.r))
```
So both the *commit* AND the *key-cap rendering* call `unicode.ToUpper`. A case-preserving keyboard must avoid BOTH. The dimming path is `gui.go:1226`: `bgcol = mulAlpha(bgcol, theme.inactiveMask)` (codex32 A1 static-dim). Active key is `i==k.row && j==k.col` → filled rounded-rect; otherwise rounded-outline.

**Input drive — `Update()` — `gui.go:1056-1145`** — ACCURATE. Three input modalities, all funnel to `rune()`:
- **Touch:** every key's `clk.Clicked(ctx)` (only if `k.Valid(*key)`) → sets `row,col` → `rune()`.
- **D-pad + Center:** `Left/Right` step within the row skipping invalid keys (with wraparound + `adjust(true)`); `Up/Down` move rows via `adjustCol`; **`Center` commits** the cursor key via `rune()`.
- **RuneEvent (physical/host keyboard):** `gui.go:1131-1132` — `r := unicode.ToLower(e.Rune)` then matches a key by rune and commits. **This lowercases the incoming rune to find the key, then `rune()` re-uppercases on commit** — so today a physical-keyboard `A` and `a` both land as `A`. For a case-preserving passphrase keyboard this is doubly wrong and is why a page-model (not a physical-shift) is the right design.

`Valid(key)` (`gui.go:1049`): backspace valid iff `Fragment` non-empty; else `!key.disabled`. `Clear()` (`gui.go:916`) resets `Fragment=""` and re-centers cursor but does NOT reset `disabled` (per the codex32 comment). On-screen layout: `k.keys[i][j].pos` is precomputed in `NewKeyboard`; `Layout` draws each key at `.pos` with input-clip rect for touch hit-testing.

The 3 side buttons (`Button1/2/3`) are **owned by the flow, not the keyboard** — the keyboard only consumes D-pad+Center+Runes+touch. Flows wire `backBtn=Button1`, `okBtn=Button3` (e.g. `inputCodex32Flow:674-675`). This is the natural home for a page-switch side button if desired, but the keyboard itself never reads Button1/2/3.

---

## 2. Passphrase entry — what exists / what's absent

**CONFIRMED: no passphrase keyboard or flow exists.** Source grep for `passphrase` = zero hits in `gui/`. `password` appears only as the `MnemonicSeed` parameter name. `preserveCase` = zero hits. No mask/reveal/dots/bullet primitive anywhere in `gui/` (grep for `reveal|masked|bullet|•|••|obscure|hidden` = empty).

**`bip39.MnemonicSeed` signature — `bip39/bip39.go:217`** (the crypto plumbing is ready):
```go
func MnemonicSeed(m Mnemonic, password string) []byte {
	...
	return pbkdf2.Key(sentence, []byte("mnemonic"+password), 2048, 64, sha512.New)
}
```

**Hardwired `""` — `gui.go:188`** (the single injection point; recon-cited, ACCURATE):
```go
seed := bip39.MnemonicSeed(m, "")   // inside deriveMasterKey
```
`deriveMasterKey` (`gui.go:187`) is the only caller of `MnemonicSeed` in the GUI; it's called by `masterFingerprintFor` (`gui.go:482`) and directly at `gui.go:2071` (the validity check in `SeedScreen.Confirm`). Threading a passphrase = adding a `password string` param to `deriveMasterKey` + `masterFingerprintFor` and their 3 call sites. **That threading is Slice 3, not Slice 2.**

**How typed text is rendered today (the fragment label):** flows render `kbd.Fragment` via `widget.Labelw(&ctx.B, ctx.Styles.word, …, kbd.Fragment)` on a rounded-rect chip (e.g. `inputCodex32Flow:700`). `widget/label.go` lays out one glyph per rune via `op.Glyph`. **For mask/reveal there is no built-in dots mode** — masking would be done at the call site by passing a derived string (e.g. `strings.Repeat("•", utf8.RuneCountInString(frag))`) into the same `Labelw`, gated by a reveal bool. (Caveat for §3: verify the GUI font has a `•` U+2022 glyph — the poppins bitmap fonts are generated over printable-ASCII only; see §3. A safe ASCII fallback is `*`.)

**Host-test text extraction:** `op.Drawer.ExtractText` (`gui/op/op.go:523`) collects every drawn glyph rune, so the harness asserts on rendered text — masked dots and revealed cleartext are both observable, and the case of committed runes is observable.

---

## 3. Design space for the passphrase keyboard widget (Slice-2 deliverable)

### Extend `Keyboard` vs new `PassphraseKeyboard` — RECOMMENDATION: **new `PassphraseKeyboard` type**

Rationale (lower-risk + more idiomatic given the shared widget):
- `Keyboard` is shared by **three** live consumers (BIP-39 `inputWordsFlow:539`, codex32 `inputCodex32Flow:672`, SLIP-39 `inputSLIP39Flow:755`), all routed from `newInputFlow:1949`. Every one relies on `rune()` force-uppercasing and on a single static alphabet. Adding `preserveCase bool` + a page/shift model + a mask flag to the shared type means every change ripples across all three and must keep `TestWordKeyboardScreen`, `TestInputSeedCodex32`, the SLIP-39 tests, and `TestWordFlow*` green — a constant regression surface.
- Both core `Keyboard` assumptions break for a passphrase: (a) **one alphabet** → passphrase needs 3 pages; (b) **force-uppercase on commit AND on render** (`gui.go:1153` + `gui.go:1244`) → passphrase must preserve case. These are not flag-gated tweaks; they're structural.
- A new type is **isolated**: no risk to the merged BIP-39/codex32/SLIP-39 flows, no SemVer-relevant behavior change to the shared widget, and it can freely **reuse** the low-level helpers (`mulAlpha`, `keyboardKey`, the `pos`/`Clickable` layout math, `assets.KeyBackspace`, `theme.inactiveMask`) by either embedding a `Keyboard` per page or copying the ~60-line layout routine. Pragmatic middle path: `PassphraseKeyboard` holds **three internal `Keyboard` instances** (one per page) built from three alphabet consts, plus a `page int`, `preserveCase`/mask state, and its own `rune()` that does NOT `ToUpper`. But note: reusing inner `Keyboard` directly still inherits its `ToUpper` render at `gui.go:1244` and commit at `:1153`, so the cleaner option is a **standalone type with its own `rune()`/`Layout`** (copy + de-uppercase) rather than embedding. Recommend the architect weigh embed-three-Keyboards (max reuse, but must suppress inner ToUpper) vs standalone (clean case semantics, ~60 LoC layout dup).

### 3-page model (lowercase / UPPERCASE / symbols+digits)

Candidate alphabet consts (printable-ASCII, mirrors the `wordKeys` row shape):
- lowercase: `"qwertyuiop\nasdfghjkl\nzxcvbnm"` (= `wordKeys`)
- UPPERCASE: `"QWERTYUIOP\nASDFGHJKL\nZXCVBNM"`
- symbols+digits: `"1234567890\n-/:;()$&@\".,?!'\n[]{}#%^*+="` (or similar; exact set TBD in spec — BIP-39 passphrases are arbitrary UTF-8 but a printable-ASCII keypad is the standard hardware-wallet scope; space-key handling TBD).

**Charset / font reality (key finding):** the passphrase string is NEVER engraved (only its fingerprint), so the **engrave `constant` font is irrelevant** to the passphrase charset — no engrave-font constraint. What matters is the **on-screen GUI font** (`Styles.keyboard = poppins.Bold25`, `Styles.word = comfortaa.Bold17`). The poppins bitmap fonts are generated by `cmd/bitmapfont` whose **default `-alphabet` is the full printable-ASCII set** `!"#$%&'()*+,-./0123456789:;<=>?@A–Z[\]^_`a–z{|}~` (`cmd/bitmapfont/main.go:32`), and the poppins `gen.go` invocations use that default (only `boldprogress45` overrides to `"0123456789:"`). **So poppins.Bold25 already renders every printable-ASCII symbol/digit/lower/upper glyph — no font regen needed for an ASCII symbol page.** Caveat: `•` (U+2022) is OUTSIDE printable-ASCII and is likely NOT in the generated font; for masking use ASCII `*` (definitely present) or regen the font to add `•` — the spec must pick. comfortaa.Bold17 (`Styles.word`, used for the fragment chip) should be checked the same way; it uses the same default alphabet via its own gen.

**Page-switching mechanism — options:**
1. **Dedicated on-key page-cycle key** (e.g. an `abc`/`ABC`/`?123` key occupying the bottom-left slot, like phone keyboards) — keeps everything inside the widget, D-pad reachable, no flow coupling. Most idiomatic given the keyboard already owns its rows.
2. **A side button (Button2, currently unused in input flows)** to cycle pages — simpler nav but couples the widget to a flow-owned button; the keyboard currently never reads Button1/2/3, so this would be a new coupling.
3. **Shift-key (modal lower↔UPPER) + a separate `?123` toggle** — closer to phone UX but more state.

Recommend (1) a self-contained page-cycle key (and possibly a separate shift), assessed in the spec. **Case-preservation** is simply: the page determines the literal rune in `allKeys` (lowercase page stores `'a'`, UPPER stores `'A'`), and the new `rune()` appends `string(r)` with **no `ToUpper`**; the render path likewise drops `unicode.ToUpper` (use `"%c", key.r`).

### Mask/reveal

Primitive: the existing `widget.Label`/`Labelw` (`gui/widget/label.go`). Toggle: a `reveal bool` on the widget (or flow). When masked, the fragment chip renders `strings.Repeat("*", utf8.RuneCountInString(frag))` (ASCII `*`, font-safe) instead of `kbd.Fragment`; when revealed, render cleartext. The reveal toggle is a side button or an on-screen key. No new render primitive needed — masking is purely a string substitution at the Label call site. (The recon's "reuse `ConfirmDelay` for timed reveal" is an option but not required for the widget.)

### Host-testability

Fully achievable with the existing harness, no new infra:
- `runUI(ctx, func(){ … })` + `frame()` (`gui_test.go:466`) drives frames; `uiContains(content,str)` (`:479`) asserts rendered text; `ExtractText` (`op.go:523`) yields drawn glyphs.
- `runes(&ctx.Router, "Ab1!")` (`event_test.go:68`) + `click(&ctx.Router, Button3)` (`:42`) + `press` drive input; `TestInputSeedCodex32` is the template (runes → click → assert returned string), and it explicitly documents the uppercasing quirk we are removing.
- **The harness CAN: drive page-switches** (click the page key or press the assigned button), **assert masked vs revealed text** (`uiContains(content,"****")` vs the cleartext), and **assert case-preserved output** (type `"Ab"`, assert committed `Fragment=="Ab"` not `"AB"`). Note `runes()` lowercases via the widget's RuneEvent path today — for a case-preserving widget the test should drive case via **explicit key clicks / page-switches**, not raw `runes`, OR the new widget's RuneEvent path must honor case (design decision; flag for spec, since the existing `Update` lowercases `e.Rune` at `gui.go:1132`).

### Standalone vs harness-demo + LoC

Slice 2 should ship a **STANDALONE widget + unit tests, no flow consumer** (Slice 3 wires the flow + fingerprint-engrave choice). A tiny test-only harness (drive the widget directly, assert `Fragment`/rendered text) is sufficient — no demo screen needed. **Rough sizing: S–M.** ~120–200 LoC for the widget (3 page alphabets + page state + non-uppercasing `rune()` + page-cycle key + mask substitution + `Layout`, much reused from the existing `Keyboard` math), plus ~80–150 LoC of tests. If implemented as a standalone type with copied layout, the upper end; if it embeds three `Keyboard`s, the lower end (but with the inner-`ToUpper` suppression caveat).

---

## 4. Cross-cutting / sizing / SemVer

- **Lockstep (shared widget):** `Keyboard` is SHARED across BIP-39 + codex32 + SLIP-39 (all three live via `newInputFlow:1949`). Any change to the shared `Keyboard` type MUST keep `TestWordKeyboardScreen` (`gui_test.go:277`), `TestInputSeedCodex32` (`codex32_input_test.go:21`), the codex32 polish tests, the SLIP-39 tests, and `TestWordFlow*` green. **This is the strongest argument for a NEW `PassphraseKeyboard` type** — it touches none of them. Baseline confirmed green at `06b57f3`.
- **Sizing:** S–M widget (see §3). No new platform/hardware interface; no font regen for ASCII (default alphabet already covers it); only `•`-masking would need a font change (avoid by using `*`).
- **SemVer / ldflags:** firmware version is set via ldflags (per CLAUDE.md); a new internal GUI widget with no flow wiring is a non-breaking additive change. **No upstream PR** for Slice 2 (widget-only; Slice 3 would be the natural PR boundary once the flow exists).
- **Slice boundary integrity:** Slice 2 deliberately does NOT thread `password` through `deriveMasterKey`/`masterFingerprintFor` (that + both-fingerprints verify + which-fingerprint-to-engrave choice is Slice 3). The `MnemonicSeed(m,"")` at `gui.go:188` stays untouched in Slice 2.

### Key files
- `gui/gui.go` — `Keyboard`/`keyboardKey` (`:840`), `NewKeyboard` (`:861`), `rune()` ToUpper (`:1147`/`:1153`), `Layout` ToUpper + dim (`:1211`/`:1226`/`:1244`), `Update` (`:1056`, RuneEvent lowercase `:1132`), `Valid`/`Clear`/`adjust`/`adjustCol`, `wordKeys` (`:537`), the three input flows (`:539`/`:672`/`:755`), `newInputFlow` (`:1949`), `deriveMasterKey` (`:187`, `MnemonicSeed(m,"")` `:188`), `masterFingerprintFor` (`:482`).
- `gui/codex32_polish.go:219-237` — `codex32Keys` const + `newCodex32Keyboard` (the per-key static-dim precedent).
- `gui/widget/label.go` — `Label`/`Labelw`/`Labelf`/`Labelwf` (fragment + mask rendering primitive).
- `gui/op/op.go:523` — `ExtractText` (host text assertions).
- `gui/event_test.go:42,57,68` — `click`/`press`/`runes`; `gui/gui_test.go:277,466,479` — `TestWordKeyboardScreen`, `runUI`, `uiContains`; `gui/codex32_input_test.go` — the runes→click→assert template.
- `gui/theme.go:11-14,65-67,104-108` — `theme.inactiveMask` (=0x55), `Styles.word` (comfortaa.Bold17), `Styles.keyboard` (poppins.Bold25).
- `cmd/bitmapfont/main.go:32` — default font alphabet = full printable-ASCII (confirms symbol/digit glyph coverage); `font/poppins/gen.go` — uses the default.
- `bip39/bip39.go:217` — `MnemonicSeed(m, password string)`.

### Recommendation summary
**Build a NEW standalone `PassphraseKeyboard` type** (3 ASCII pages: lowercase/UPPERCASE/symbols+digits, self-contained page-cycle key, case-preserving `rune()`/render with NO `ToUpper`, mask via `*`-substitution into the existing `Labelw` with a reveal toggle), reusing the existing `keyboardKey`/layout math and `theme.inactiveMask`. Ship it widget-only with unit tests (S–M, ~120–200 LoC widget + tests); no flow wiring, no `MnemonicSeed` threading, no upstream PR — those are Slice 3. This isolates the shared `Keyboard` (and its three live consumers) from any regression and keeps case semantics clean. Open spec questions for the brainstorm/R0: exact symbol-page charset + space handling; mask glyph (`*` vs regen for `•`); page-switch via on-key vs Button2; whether the widget's RuneEvent path should honor case (for host tests) given the current `unicode.ToLower(e.Rune)`.
