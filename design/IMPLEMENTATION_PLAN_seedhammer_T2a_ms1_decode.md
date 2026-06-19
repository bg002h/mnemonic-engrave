# T2a — ms1 decode→display Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Decode a hand-typed **unshared** ms1 secret on-device and DISPLAY its BIP-39 words (English) or entropy-hex + wordlist-language name (non-English), for verification before engraving.

**Architecture:** A pure m-format payload decoder `codex32.DecodeMS1` (strip the prefix byte from `Seed()`, read the mnem language byte, validate entropy length) + a display-only `ms1DecodeFlow` screen reached from `confirmCodex32Flow`'s **Button2 when `f.Unshared`** (the slot Recover already uses for shares). Reuses codex32/bip39/the merged measure-and-advance display pattern — ports no codec.

**Tech Stack:** Go (`go test ./codex32/ ./gui/ ./bip39/`) + TinyGo (CI). Spec (GREEN R1): `design/SPEC_seedhammer_T2a_ms1_decode.md`. Base: fork `main` `68e6ead`.

**Gate:** plan R0 (1C/1I) — the architect built+ran the code (decoder + display + the Japanese vector reproduced from the live Rust encoder all verified). Folded: **C-1** the "Show secret" affordance is gated on `f.Unshared` **AND `DecodeMS1` succeeding** (a plain BIP-93 unshared secret isn't a decodable m-format ms1 → affordance hidden, Button2 inert → preserves `TestConfirmCodex32UnsharedNoRecover`, no hang); **I-1** Step-4 edits now quote the exact `codex32_polish.go:98-122` anchors; **M-1** added a 24-word paging test. R0 review: `design/agent-reports/seedhammer-T2a-ms1-plan-review-R0.md`. Re-dispatching R1.

---

## Source-of-truth facts (verified `68e6ead`; layout R0-confirmed vs ms-codec)

- **m-format ms1 payload (R0-byte-proven):** `codex32.String.Seed()` (= `parts().data()`, `codex32/codex32.go:386`) returns `[prefix][lang? for mnem][entropy]`. Prefix byte `Seed()[0]`: `0x00`=entr / `0x02`=mnem (ms-codec `consts.rs:17,39`). For mnem `data[1]` = language 0..9 (`consts.rs:47-58`). Entropy = remaining {16,20,24,28,32} B (`consts.rs:29`), byte-aligned (no `data()` pad artifact). The prefix is NOT the codex32 4-char `id`/Tag (which is `"entr"` for both forms) — branch on `Seed()[0]`, never the id.
- **codex32:** `New(s) (String,error)` (BCH validity); `String.Seed() []byte`; `String.Split() (id,threshold,idx)`; `NewSeed(hrp string, threshold int, id string, shareIdx rune, data []byte) (String,error)` (`:279` — for constructing refusal-test payloads). `ParsePrefix(s) (Fields,error)`; `Fields{Unshared, Identifier, ShareIndex, ...}` (`codex32/polish.go:63`).
- **bip39 (English-only):** `New(entropy []byte) Mnemonic` (`bip39/bip39.go:228`; PANICS on `len<16||>32` or `len%4!=0`); `Mnemonic []Word`; `LabelFor(Word) string` (`:79`); `Mnemonic.String()` (`:166`).
- **gui:** `confirmCodex32Flow(ctx, th, scan codex32.String) codex32ConfirmAction` (`gui/codex32_polish.go:83-141`) — Button1 back / Button2 recoverBtn (drained always; `return codex32Recover` only when `!f.Unshared`) / Button3 engrave; nav appends the recover button only when `!f.Unshared` (so **Button2 is free for the unshared secret**). `wipeBytes([]byte)` scrub (`gui/slip39_polish.go:328`). The merged **`descriptorAddressFlow`** (`gui/address_polish.go`) is the measure-and-advance scrollable-list template. `showError(ctx,th,title,body)`. Render primitives: `layoutNavigation`/`NavButton`/`layoutTitle`/`widget.Labelw`/`layout.Rectangle`/`CutTop`/`CutBottom`/`op.Layer`/`op.Color`/`leadingSize`/`ctx.Styles.body`/`assets.Icon{Back,Right,Info}`. Test harness: `NewContext(newPlatform())`, `click`, `runUI`, `uiContains`, `&descriptorTheme`.
- **Parity vectors (Rust-sourced — embed verbatim):**
  - entr (ms-codec `tests/vectors/v0.1.json`): `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f` → entropy `00…00`(16); `ms10entrsqqqjx3t83x4ummcpydzk0zdtehhszg69vucrgd4pcjx3kkj` → `0123456789abcdef0123456789abcdef01234567`(20, non-zero); `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w` → `00…00`(32).
  - mnem-English (ms-codec `tests/mnem.rs:144`): `ms10entrsqgqqc83yukgh23xkvmp59xf2eldpk4cdrq2y4h82yz` → prefix `0x02`, lang 0, entropy `0c1e24e5917544d666c342992acfda1b`(16).
  - mnem-Japanese (**Rust-encoder-sourced**, `encode(Tag::ENTR, &Payload::Mnem{language:1, entropy})`, captured this cycle): `ms10entrsqgqsc83yukgh23xkvmp59xf2eldpkpefrcjje3drdq` → prefix `0x02`, lang 1, entropy `0c1e24e5917544d666c342992acfda1b`(16). (Differs from the English golden only in the language-byte region — confirms the layout.)

---

## File manifest
| File | Change |
|---|---|
| `codex32/mspayload.go` | **new** (fork tree, pkg `seedhammer.com/codex32`) — `DecodeMS1` + `MSLanguageNames`. |
| `codex32/mspayload_test.go` | **new** — parity (entr + mnem-Eng + mnem-Jp) + unknown-prefix/bad-length refusal. |
| `gui/ms1_decode.go` | **new** — `ms1DecodeFlow` (display-only words / non-English block, measure-and-advance, scrub). |
| `gui/codex32_polish.go` | **modify** — `confirmCodex32Flow`: Button2 = "Show secret"→`ms1DecodeFlow` when `f.Unshared` (else Recover). |
| `gui/ms1_decode_test.go` | **new** — English-words shown; non-English name+hex+warning (words NOT shown); the `f.Unshared` gate; Back. |

Unchanged/reused: codex32 BCH/string layer, bip39, the engrave path, md1/mk1 (`mdmkText`) branch.

---

## Task 0: Worktree + baseline
- [ ] **Step 1:** `cd /scratch/code/shibboleth/seedhammer && git worktree add -b feat/ms1-decode-display ../seedhammer-wt-t2a-ms1 68e6ead && cd ../seedhammer-wt-t2a-ms1`
- [ ] **Step 2:** `go test ./codex32/ ./gui/ ./bip39/` → PASS (baseline). (Go: `/home/bcg/.local/go/bin/go` if not on PATH.)

---

## Task 1: `codex32.DecodeMS1` (the m-format payload decoder)

**Files:** Create `codex32/mspayload.go`, `codex32/mspayload_test.go`.

- [ ] **Step 1: Write the failing test**

Create `codex32/mspayload_test.go`:
```go
package codex32

import (
	"bytes"
	"encoding/hex"
	"testing"
)

func mustHexT(t *testing.T, s string) []byte {
	t.Helper()
	b, err := hex.DecodeString(s)
	if err != nil {
		t.Fatal(err)
	}
	return b
}

// Rust-sourced parity vectors: codex32.New(ms1).Seed() decoded via DecodeMS1
// must reproduce the known prefix/language/entropy byte-for-byte.
func TestDecodeMS1Parity(t *testing.T) {
	cases := []struct {
		name, ms1, entropyHex string
		wantPrefix, wantLang  int
	}{
		{"entr16-zero", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f", "00000000000000000000000000000000", 0x00, 0},
		{"entr20-nonzero", "ms10entrsqqqjx3t83x4ummcpydzk0zdtehhszg69vucrgd4pcjx3kkj", "0123456789abcdef0123456789abcdef01234567", 0x00, 0},
		{"entr32-zero", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w", "0000000000000000000000000000000000000000000000000000000000000000", 0x00, 0},
		{"mnem-english16", "ms10entrsqgqqc83yukgh23xkvmp59xf2eldpk4cdrq2y4h82yz", "0c1e24e5917544d666c342992acfda1b", 0x02, 0},
		{"mnem-japanese16", "ms10entrsqgqsc83yukgh23xkvmp59xf2eldpkpefrcjje3drdq", "0c1e24e5917544d666c342992acfda1b", 0x02, 1},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			s, err := New(c.ms1)
			if err != nil {
				t.Fatalf("New(%q): %v", c.ms1, err)
			}
			prefix, lang, entropy, err := DecodeMS1(s)
			if err != nil {
				t.Fatalf("DecodeMS1: %v", err)
			}
			if prefix != c.wantPrefix || lang != c.wantLang {
				t.Errorf("prefix=%#x lang=%d, want %#x/%d", prefix, lang, c.wantPrefix, c.wantLang)
			}
			if want := mustHexT(t, c.entropyHex); !bytes.Equal(entropy, want) {
				t.Errorf("entropy=%x, want %x", entropy, want)
			}
		})
	}
}

// Refusal: an unknown prefix byte or a non-BIP-39 entropy length → error, no panic.
func TestDecodeMS1Refusal(t *testing.T) {
	mk := func(data []byte) String {
		s, err := NewSeed("ms", 0, "entr", 's', data)
		if err != nil {
			t.Fatalf("NewSeed: %v", err)
		}
		return s
	}
	z16 := make([]byte, 16)
	// Unknown prefix 0x01 + 16B → errMSBadPrefix.
	if _, _, _, err := DecodeMS1(mk(append([]byte{0x01}, z16...))); err == nil {
		t.Error("unknown prefix accepted")
	}
	// entr prefix + 15B entropy (not in {16,20,24,28,32}) → errMSBadLength.
	if _, _, _, err := DecodeMS1(mk(append([]byte{0x00}, make([]byte, 15)...))); err == nil {
		t.Error("bad entropy length accepted")
	}
	// mnem prefix + language 10 (>9) → errMSBadLanguage.
	if _, _, _, err := DecodeMS1(mk(append([]byte{0x02, 10}, z16...))); err == nil {
		t.Error("invalid language accepted")
	}
}
```
(Note: `NewSeed` round-trips byte-aligned `data` through `Seed()` — the parity vectors above already prove `Seed()` returns the exact payload, and the refusal `data` are byte-aligned too. If a refusal case's `Seed()` length differs from the input by padding, adjust the asserted length boundary, never the algorithm.)

- [ ] **Step 2: Run — expect FAIL** (`DecodeMS1` undefined): `go test ./codex32/ -run TestDecodeMS1 2>&1 | tail`

- [ ] **Step 3: Implement `codex32/mspayload.go`**
```go
package codex32

import "errors"

// m-format ms1 payload prefix bytes (ms-codec consts.rs:17,39). The prefix is
// the FIRST byte of the codex32 data payload (Seed()[0]) — NOT the 4-char
// id/Tag, which is "entr" for both entr and mnem secrets.
const (
	msPrefixEntr  = 0x00 // RESERVED_PREFIX: payload = [0x00][entropy]
	msPrefixMnem  = 0x02 // MNEM_PREFIX:     payload = [0x02][language][entropy]
	msMaxLanguage = 9    // MNEM_LANGUAGE_NAMES indices 0..9
)

var (
	errMSBadPrefix   = errors.New("codex32: not an m-format secret payload")
	errMSBadLanguage = errors.New("codex32: invalid mnem wordlist language")
	errMSBadLength   = errors.New("codex32: invalid entropy length")
)

// MSLanguageNames are the BIP-39 wordlist names indexed by the mnem language
// byte (ms-codec consts.rs:47-58).
var MSLanguageNames = [10]string{
	"English", "Japanese", "Korean", "Spanish",
	"Chinese (Simplified)", "Chinese (Traditional)",
	"French", "Italian", "Czech", "Portuguese",
}

// DecodeMS1 decodes the m-format ms1 secret payload from a New-valid, UNSHARED
// codex32 string: prefix = Seed()[0] (msPrefixEntr/msPrefixMnem); for mnem,
// language = Seed()[1] (0..9); entropy = the remaining 16/20/24/28/32 bytes.
// language is 0 for entr. Deterministic; the returned entropy is SECRET (caller
// scrubs). Callers MUST pass only the unshared secret — a K-of-N share carries
// an SSS-evaluated point, not an m-format payload, and yields errMSBadPrefix/Length.
func DecodeMS1(s String) (prefix, language int, entropy []byte, err error) {
	data := s.Seed()
	if len(data) < 2 {
		return 0, 0, nil, errMSBadPrefix
	}
	switch data[0] {
	case msPrefixEntr:
		prefix, language, entropy = msPrefixEntr, 0, data[1:]
	case msPrefixMnem:
		if len(data) < 3 {
			return 0, 0, nil, errMSBadLength
		}
		language = int(data[1])
		if language > msMaxLanguage {
			return 0, 0, nil, errMSBadLanguage
		}
		prefix, entropy = msPrefixMnem, data[2:]
	default:
		return 0, 0, nil, errMSBadPrefix
	}
	switch len(entropy) {
	case 16, 20, 24, 28, 32:
	default:
		return 0, 0, nil, errMSBadLength
	}
	return prefix, language, entropy, nil
}
```

- [ ] **Step 4: Run — expect PASS**: `go test ./codex32/ -run TestDecodeMS1 -v`

- [ ] **Step 5: Commit**
```bash
git add codex32/mspayload.go codex32/mspayload_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "codex32: DecodeMS1 — m-format ms1 payload decoder (T2a)

Strip the prefix byte from Seed() (0x00=entr/0x02=mnem; NOT the id/Tag),
read the mnem language byte, validate entropy length {16,20,24,28,32}.
Parity vs Rust-sourced ms-codec vectors (entr + mnem English + Japanese);
unknown-prefix/bad-length refusal. Reuses the in-tree codex32 layer.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: `ms1DecodeFlow` + the `f.Unshared`-gated "Show secret" affordance

**Files:** Create `gui/ms1_decode.go`, `gui/ms1_decode_test.go`; modify `gui/codex32_polish.go`.

- [ ] **Step 1: Write the failing tests**

Create `gui/ms1_decode_test.go`:
```go
package gui

import (
	"testing"

	"seedhammer.com/bip39"
	"seedhammer.com/codex32"
)

func mustCodex32T(t *testing.T, s string) codex32.String {
	t.Helper()
	c, err := codex32.New(s)
	if err != nil {
		t.Fatalf("New(%q): %v", s, err)
	}
	return c
}

// English ms1 (entr) → the decoded BIP-39 words are shown.
func TestMS1DecodeFlowEnglishWords(t *testing.T) {
	const ms1 = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f" // entropy 0*16 → 12 abandon... words
	s := mustCodex32T(t, ms1)
	_, _, entropy, err := codex32.DecodeMS1(s)
	if err != nil {
		t.Fatal(err)
	}
	want := bip39.LabelFor(bip39.New(entropy)[0]) // first word label
	ctx := NewContext(newPlatform())
	frame, quit := runUI(ctx, func() { ms1DecodeFlow(ctx, &descriptorTheme, s) })
	defer quit()
	seen := false
	for i := 0; i < 8; i++ {
		c, ok := frame()
		if !ok {
			break
		}
		if uiContains(c, want) {
			seen = true
			break
		}
	}
	if !seen {
		t.Fatalf("English words not shown (want first word %q)", want)
	}
}

// Non-English mnem (Japanese) → language name + entropy hex shown; NO English words.
func TestMS1DecodeFlowNonEnglish(t *testing.T) {
	const ms1 = "ms10entrsqgqsc83yukgh23xkvmp59xf2eldpkpefrcjje3drdq" // mnem lang=1 (japanese)
	s := mustCodex32T(t, ms1)
	ctx := NewContext(newPlatform())
	frame, quit := runUI(ctx, func() { ms1DecodeFlow(ctx, &descriptorTheme, s) })
	defer quit()
	sawLang, sawHex := false, false
	for i := 0; i < 8; i++ {
		c, ok := frame()
		if !ok {
			break
		}
		if uiContains(c, "Japanese") {
			sawLang = true
		}
		if uiContains(c, "0c1e24e5917544d666c342992acfda1b") {
			sawHex = true
		}
	}
	if !sawLang || !sawHex {
		t.Fatalf("non-English: lang=%v hex=%v (want both)", sawLang, sawHex)
	}
}

// The "Show secret" affordance (Button2) opens the decode view ONLY for the
// unshared secret. (Drive confirmCodex32Flow on an unshared secret, press
// Button2, assert the decoded words appear.)
func TestConfirmShowSecretGate(t *testing.T) {
	const ms1 = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"
	s := mustCodex32T(t, ms1)
	want := bip39.LabelFor(bip39.New(make([]byte, 16))[0])
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Button2) // Show secret (unshared → opens ms1DecodeFlow)
	frame, quit := runUI(ctx, func() { confirmCodex32Flow(ctx, &descriptorTheme, s) })
	defer quit()
	seen := false
	for i := 0; i < 10; i++ {
		c, ok := frame()
		if !ok {
			break
		}
		if uiContains(c, want) {
			seen = true
			break
		}
	}
	if !seen {
		t.Fatal("Button2 did not open the secret view on the unshared secret")
	}
}

// 24-word secret (32B entropy) spans multiple pages → paging must not skip a
// word (the T1 measure-and-advance lesson). Observe each page, then advance.
func TestMS1DecodeFlowPaging24Words(t *testing.T) {
	const ms1 = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w" // entr 32B zero → 24 words
	s := mustCodex32T(t, ms1)
	_, _, entropy, err := codex32.DecodeMS1(s)
	if err != nil {
		t.Fatal(err)
	}
	m := bip39.New(entropy)
	want := make(map[string]bool) // word labels for indices 0,11,23 (first/mid/last)
	for _, i := range []int{0, 11, 23} {
		want[bip39.LabelFor(m[i])] = false
	}
	ctx := NewContext(newPlatform())
	frame, quit := runUI(ctx, func() { ms1DecodeFlow(ctx, &descriptorTheme, s) })
	for i := 0; i < 40; i++ {
		c, ok := frame()
		if !ok {
			break
		}
		for w := range want {
			// match the "N WORD" line form; the word label is unique enough
			if uiContains(c, w) {
				want[w] = true
			}
		}
		click(&ctx.Router, Button3) // advance one page AFTER observing
	}
	quit()
	for w, seen := range want {
		if !seen {
			t.Errorf("word %q never shown — paging skipped it", w)
		}
	}
}
```
(Note: `abandon`-heavy zero-entropy mnemonics repeat words; this asserts the first/mid/last *positions'* labels each appear at some page. If label-repetition makes the assertion ambiguous, match the full `"N LABEL"` line instead — the index prefix disambiguates.)

- [ ] **Step 2: Run — expect FAIL** (`ms1DecodeFlow` undefined): `go test ./gui/ -run 'TestMS1Decode|TestConfirmShowSecret' 2>&1 | tail`

- [ ] **Step 3: Implement `gui/ms1_decode.go`** (measure-and-advance display, mirroring `descriptorAddressFlow`; lines are precomputed)
```go
package gui

import (
	"encoding/hex"
	"fmt"
	"image"

	"seedhammer.com/bip39"
	"seedhammer.com/codex32"
	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/layout"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
)

// ms1DecodeFlow decodes and DISPLAYS the unshared ms1 secret: the BIP-39 words
// (language 0 = English) or the language name + entropy hex (non-English, since
// the fork ships only the English wordlist). Display-only: no engrave, no NFC,
// no mutation. SECRET — the entropy buffer is scrubbed on return (the displayed
// strings are immutable Go strings and live until GC, as with SeedScreen).
func ms1DecodeFlow(ctx *Context, th *Colors, scan codex32.String) {
	_, language, entropy, err := codex32.DecodeMS1(scan)
	if err != nil {
		showError(ctx, th, "Secret", "Can't decode this secret — "+err.Error())
		return
	}
	defer wipeBytes(entropy)

	var lines []string
	if language == 0 { // English (entr or mnem-English) → the words
		m := bip39.New(entropy)
		for i, w := range m {
			lines = append(lines, fmt.Sprintf("%d %s", i+1, bip39.LabelFor(w)))
		}
	} else { // non-English mnem → name + hex + warning, never English words
		name := codex32.MSLanguageNames[language]
		lines = []string{
			"Language: " + name,
			"entropy: " + hex.EncodeToString(entropy),
			"Words not shown on this device.",
			"Restore with a " + name + " BIP-39 wallet.",
		}
	}

	backBtn := &Clickable{Button: Button1}
	pageBtn := &Clickable{Button: Button3}
	dims := ctx.Platform.DisplaySize()
	lineWidth := dims.X - 2*8
	screen := layout.Rectangle{Max: dims}
	_, content := screen.CutTop(leadingSize)
	content, _ = content.CutBottom(leadingSize)
	contentTop := content.Min.Y + 8
	contentBottom := content.Max.Y
	start := 0
	for !ctx.Done {
		if backBtn.Clicked(ctx) {
			return
		}
		// Measure-and-advance: render only the lines that fit; page forward by
		// the count shown (gap-free; the T1 lesson — never fixed-page wrapping text).
		shown := 0
		y := contentTop
		body := make([]op.Op, 0, len(lines))
		for i := start; i < len(lines); i++ {
			lbl, sz := widget.Labelw(&ctx.B, ctx.Styles.body, lineWidth, th.Text, lines[i])
			if i > start && y+sz.Y > contentBottom {
				break
			}
			body = append(body, lbl.Offset(image.Pt((dims.X-sz.X)/2, y)))
			y += sz.Y + 6
			shown++
			if y > contentBottom {
				break
			}
		}
		if pageBtn.Clicked(ctx) {
			if start+shown < len(lines) {
				start += shown
			} else {
				start = 0 // wrap to the top
			}
			continue
		}
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, "Secret")
		nav, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack},
			{Clickable: pageBtn, Style: StylePrimary, Icon: assets.IconRight},
		}...)
		frameOps := append([]op.Op{nav, titleOp}, body...)
		frameOps = append(frameOps, op.Color(&ctx.B, th.Background))
		ctx.Frame(op.Layer(frameOps...))
	}
}
```
(`shown` is recomputed each frame from the precomputed `lines` — cheap, no crypto; `pageBtn` advances by the count just shown, so no line is skipped. If `len(lines)` fits on one page, page-forward wraps to top. `wipeBytes` is the slip39 scrub helper.)

- [ ] **Step 4: Modify `confirmCodex32Flow`** (`gui/codex32_polish.go`) — Button2 = "Show secret" ONLY when the unshared secret is a decodable m-format ms1

The "Show secret" affordance is gated on **`f.Unshared` AND `DecodeMS1` succeeding** (spec §2.7, plan-R0 C-1). This is the load-bearing fix: a plain BIP-93 unshared secret (e.g. the `ms10tests…` test vector) is NOT a decodable m-format ms1 (`Seed()[0]` ∉ {0x00,0x02}), so for it the affordance is hidden and Button2 stays inert — which **preserves the existing `TestConfirmCodex32UnsharedNoRecover`** and avoids a "can't decode" message on a button press.

**(a)** Immediately after the three `Clickable` declarations (`gui/codex32_polish.go:98-100`) and BEFORE the `for !ctx.Done {` loop (`:101`), add the once-computed gate:
```go
	// "Show secret" is offered only for an unshared secret that actually decodes
	// as an m-format ms1 (a plain BIP-93 secret is not decodable). Probe once.
	_, _, _, msErr := codex32.DecodeMS1(scan)
	showSecret := f.Unshared && msErr == nil
```

**(b)** Replace the EXACT recover-click block (`gui/codex32_polish.go:108-111`):
```go
		recoverClicked := recoverBtn.Clicked(ctx)
		if !f.Unshared && recoverClicked {
			return codex32Recover
		}
```
with:
```go
		recoverClicked := recoverBtn.Clicked(ctx) // always drained (queue-head idiom)
		switch {
		case showSecret && recoverClicked:
			ms1DecodeFlow(ctx, th, scan) // display-only "Show secret" sub-flow
			continue
		case !f.Unshared && recoverClicked:
			return codex32Recover
		}
```

**(c)** Replace the EXACT nav-append block (`gui/codex32_polish.go:116-122`):
```go
		navBtns := []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack},
		}
		if !f.Unshared {
			navBtns = append(navBtns, NavButton{Clickable: recoverBtn, Style: StyleSecondary, Icon: assets.IconRight})
		}
		navBtns = append(navBtns, NavButton{Clickable: engraveBtn, Style: StylePrimary, Icon: assets.IconHammer})
```
with:
```go
		navBtns := []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack},
		}
		switch {
		case showSecret:
			navBtns = append(navBtns, NavButton{Clickable: recoverBtn, Style: StyleSecondary, Icon: assets.IconInfo}) // Show secret
		case !f.Unshared:
			navBtns = append(navBtns, NavButton{Clickable: recoverBtn, Style: StyleSecondary, Icon: assets.IconRight}) // Recover
		}
		navBtns = append(navBtns, NavButton{Clickable: engraveBtn, Style: StylePrimary, Icon: assets.IconHammer})
```
(Keep the rest of `confirmCodex32Flow` — title/lines/Back/engrave/render — unchanged. `recoverBtn` is still drained every frame. `confirmCodex32Flow` is NOT in `BenchmarkAllocs`, so the `append` nav is fine. The `f.Unshared`-non-decodable case now appends NO middle button, exactly as before the change.)

- [ ] **Step 5: Run — expect PASS** + no regressions:
```bash
go test ./gui/ -run 'TestMS1Decode|TestConfirmShowSecret|TestConfirmCodex32|TestRecoverCodex32' -v
go test ./codex32/ ./gui/ ./bip39/
```
Expected: PASS. **`TestConfirmCodex32UnsharedNoRecover` stays green** because its `ms10tests…` secret is plain BIP-93 (not m-format) → `showSecret=false` → Button2 inert, identical to today. Shares still get Recover on Button2; the new `TestConfirmShowSecretGate` uses a decodable `entr` secret so its Button2 opens the decode view.

- [ ] **Step 6: Commit**
```bash
git add gui/ms1_decode.go gui/ms1_decode_test.go gui/codex32_polish.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "gui: ms1 decode→display (Show secret) on the unshared-secret confirm (T2a)

confirmCodex32Flow's Button2 becomes 'Show secret' for the unshared
secret (the slot Recover uses for shares) → ms1DecodeFlow: BIP-39 words
(English) or language name + entropy hex (non-English). Display-only,
SECRET (entropy scrubbed on return); measure-and-advance paging. Shares
still get Recover; engrave path unchanged.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Full verification
- [ ] **Step 1:** `go test ./... && go vet ./codex32/... ./gui/... && gofmt -l codex32/ gui/`
Expected: all PASS; vet clean (pre-existing `gui/op/draw_test.go` go1.26 note excepted); `gofmt -l` silent. `go test -run TestAllocs ./gui/` PASS (confirmCodex32Flow isn't alloc-gated, but verify no regression).
- [ ] **Step 2 (CI):** the `tinygo-device-build` job compiles `codex32`+`gui` (now with `DecodeMS1`/`ms1DecodeFlow`) — local if TinyGo present, else CI.

---

## Done criteria
An unshared ms1 secret can be decoded + displayed on-device (English words / non-English name+hex) before engrave; the `mnem` language byte is always surfaced; never NFC/engrave; entropy scrubbed; shares unaffected. After all tasks: whole-diff execution review → merge no-ff signed+DCO → push `bg002h` → clean up.

## Self-review (vs spec)
- §1 decode + display (English words / non-English name+hex) → Tasks 1+2. ✔
- §2.1 SECRET display-only + scrub → `wipeBytes(entropy)`, no NFC/engrave. ✔ §2.2 mnem byte surfaced → language branch always shows the name. ✔ §2.3 layout (prefix byte, Rust-sourced vectors incl. non-English) → Task 1 test. ✔ §2.4 reuse-not-port → DecodeMS1 = `Seed()`+slice+`bip39.New`. ✔ §2.5 length/prefix refusal + panic-guard → `TestDecodeMS1Refusal` + the {16..32} switch before `bip39.New`. ✔ §2.7 gate (unshared AND `DecodeMS1` succeeds) → Button2 gated on `showSecret`; a non-m-format unshared secret keeps Button2 inert (preserves the existing test). ✔
- §4 GUI hook (Button2-when-Unshared, measure-and-advance display) → Task 2. ✔ §6 TDD → Tasks 1/2/3. ✔
No placeholders: vectors are the exact Rust-sourced strings (entr v0.1.json, mnem-English mnem.rs:144, mnem-Japanese Rust-encoder-captured). Types (`codex32.String`, `DecodeMS1`, `bip39.Mnemonic`) consistent across tasks.
