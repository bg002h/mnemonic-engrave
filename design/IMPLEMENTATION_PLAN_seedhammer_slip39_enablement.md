# SeedHammer SLIP-39 Share Entry + Verbatim Engrave (Cycle C, Tier 1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Re-enable on-device SLIP-39 single-share entry + verbatim engrave, gated by an in-tree RS1024 checksum + header parse (`slip39.ParseShare`). No secret recovery (Tier-2/Cycle-D). 128-bit / 20-word only.

**Architecture:** **C1** new `slip39/share.go`: RS1024 over GF(1024) (error-detection only) + `ParseShare(string) (Share, error)` + `Describe` + an exact case-insensitive reverse word lookup. **C2/C3** new `gui/slip39_polish.go`: `showError` (generalized `showCodex32Error`), `confirmSLIP39Flow`, `engraveSLIP39` (always returns `true` — mirrors `engraveCodex32`, avoids the "Unknown format"-on-cancel pitfall) — all the `fmt`-using code lives here so `gui.go` needs **no new imports**; plus `gui.go` edits: enable the `"SLIP-39"` menu choice, uncomment+fix the `case 3:` entry, and replace the dormant `case slip39.Share:` block with `case slip39words.Share: return engraveSLIP39(...)`. No Shamir/Feistel/PBKDF2/passphrase; `codex32`/`mdmk.go` untouched.

**Tech Stack:** Go/TinyGo (RP2350 — **`int` is 32-bit**, so the 40-bit share header uses `uint64`). Host tests `/home/bcg/.local/go/bin/go test ./slip39/... ./gui/...`.

**Base:** fork `main` `9b0a02c`. Branch `feat/slip39-entry-engrave`. Fork-side only; no upstream PR.

**Spec:** `design/SPEC_seedhammer_slip39_enablement.md` (R0 GREEN at R4). **PLAN GATE:** must pass the opus plan R0 gate (0C/0I) before any code.

**Pre-verified test vectors** (computed by the recon's scratch decoder against the fork wordlist; RS1024 self-checked):
- **duckling** (valid, ext=0, `shamir`): `"duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision keyboard"` → `Identifier=7945, Extendable=false, GroupThreshold=1, GroupCount=1, MemberIndex=0, MemberThreshold=1`.
- **testify** (valid, ext=1, `shamir_extendable`): `"testify swimming academic academic column loyalty smear include exotic bedroom exotic wrist lobe cover grief golden smart junior estimate learn"` → `Identifier=29019, Extendable=true`.
- **duckling-bad** (invalid checksum — last word `keyboard`→`kidney`): `"duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision kidney"`.

---

## File Structure

| File | Responsibility | Tasks |
|---|---|---|
| `slip39/share.go` *(new)* | RS1024 + `Share` + `ParseShare` + `Describe` + exact reverse lookup. | 1 |
| `slip39/share_test.go` *(new)* | Vector-gated `ParseShare`/RS1024/`Describe` tests. | 1 |
| `gui/slip39_polish.go` *(new)* | `showError`, `confirmSLIP39Flow`, `engraveSLIP39` (all `fmt`-using code). | 2 |
| `gui/gui.go` *(modify)* | Menu choice + `case 3:` entry + `case slip39words.Share:` engrave (no new imports). | 2 |
| `gui/slip39_polish_test.go` *(new)* | `confirmSLIP39Flow` render/action + `engraveSLIP39` back-returns-true. | 2 |
| `slip39/wordlist.*`, `codex32/*`, `mdmk.go`, `gui/scan.go` *(unchanged — must stay green)* | wordlist data; codex32/mdmk; NFC stays disabled. | guard |

**Commit hygiene:** explicit paths. Signed + DCO: `git commit -S -s` (fall back to `-s` if signing unavailable, and say so).

---

## Task 0: Worktree + clean baseline

- [ ] **Step 1: Create the worktree**
```bash
cd /scratch/code/shibboleth/seedhammer
git worktree add /scratch/code/shibboleth/seedhammer-wt-slip39 -b feat/slip39-entry-engrave 9b0a02c
cd /scratch/code/shibboleth/seedhammer-wt-slip39
git config user.name "Brian Goss"; git config user.email "goss.brian@gmail.com"
```
- [ ] **Step 2: Baseline** — Run: `/home/bcg/.local/go/bin/go test ./slip39/... ./gui/...`
Expected: PASS. If red, STOP and report.

---

## Task 1: C1 — `slip39.ParseShare` + RS1024 + `Describe`

**Files:** Create `slip39/share.go`, `slip39/share_test.go`.

**Context:** the `slip39` package is wordlist-only. `LabelFor(Word) string` returns the UPPERCASE label; `ClosestWord(s) (Word, bool)` is a PREFIX match (not exact). `words`/`index` (`wordlist.go`) are unexported but accessible from `share.go` (same package). RS1024 is error-detection only (no secret handling). The 40-bit header (first 4 words) MUST be assembled in `uint64` (RP2350 `int` is 32-bit). The customization string is chosen by the decoded `ext` bit BEFORE checksum verification.

- [ ] **Step 1: Write the failing tests** — `slip39/share_test.go`

```go
package slip39

import (
	"errors"
	"strings"
	"testing"
)

const (
	vecDuckling    = "duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision keyboard"
	vecTestify     = "testify swimming academic academic column loyalty smear include exotic bedroom exotic wrist lobe cover grief golden smart junior estimate learn"
	vecDucklingBad = "duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision kidney"
)

func TestParseShare(t *testing.T) {
	s, err := ParseShare(vecDuckling)
	if err != nil {
		t.Fatalf("ParseShare(duckling): %v", err)
	}
	if s.Identifier != 7945 {
		t.Errorf("Identifier = %d, want 7945", s.Identifier)
	}
	if s.Extendable {
		t.Errorf("Extendable = true, want false")
	}
	if s.GroupThreshold != 1 || s.GroupCount != 1 || s.MemberIndex != 0 || s.MemberThreshold != 1 {
		t.Errorf("fields = %+v, want 1-of-1 single-group", s)
	}
	if len(s.Mnemonic) != 20 || s.Mnemonic[0] != "DUCKLING" {
		t.Errorf("Mnemonic = %v (len %d), want 20 canonical-uppercase words starting DUCKLING", s.Mnemonic, len(s.Mnemonic))
	}

	// ext=1 vector exercises the shamir_extendable customization string.
	s, err = ParseShare(vecTestify)
	if err != nil {
		t.Fatalf("ParseShare(testify): %v", err)
	}
	if s.Identifier != 29019 || !s.Extendable {
		t.Errorf("testify fields = %+v, want Identifier=29019 Extendable=true", s)
	}

	// Uppercase input (the GUI feeds LabelFor's uppercase) parses identically.
	if _, err := ParseShare(strings.ToUpper(vecDuckling)); err != nil {
		t.Errorf("uppercase parse: %v", err)
	}

	// Bad checksum.
	if _, err := ParseShare(vecDucklingBad); !errors.Is(err, errBadChecksum) {
		t.Errorf("bad checksum: %v, want errBadChecksum", err)
	}
	// Unknown word.
	bad := "zzzz" + vecDuckling[len("duckling"):]
	if _, err := ParseShare(bad); !errors.Is(err, errNotInWordlist) {
		t.Errorf("unknown word: %v, want errNotInWordlist", err)
	}
	// Wrong length.
	if _, err := ParseShare("duckling enlarge"); !errors.Is(err, errWrongLength) {
		t.Errorf("wrong length: %v, want errWrongLength", err)
	}
}

func TestDescribe(t *testing.T) {
	cases := []struct {
		in   error
		want string
	}{
		{nil, ""},
		{errBadChecksum, "bad checksum"},
		{errNotInWordlist, "unknown word"},
		{errUnsupportedSize, "256-bit not supported"},
		{errWrongLength, "wrong length"},
		{errors.New("other"), "invalid"},
	}
	for _, c := range cases {
		if got := Describe(c.in); got != c.want {
			t.Errorf("Describe(%v) = %q, want %q", c.in, got, c.want)
		}
	}
}
```

- [ ] **Step 2: Run to verify it fails** — Run: `/home/bcg/.local/go/bin/go test ./slip39/ -run 'TestParseShare|TestDescribe'`
Expected: FAIL — `undefined: ParseShare` / `undefined: errBadChecksum` (compile error).

- [ ] **Step 3: Implement** — `slip39/share.go`

```go
package slip39

import (
	"errors"
	"strings"
)

var (
	errWrongLength     = errors.New("slip39: wrong word count")
	errUnsupportedSize = errors.New("slip39: 256-bit shares not supported")
	errNotInWordlist   = errors.New("slip39: word not in wordlist")
	errBadChecksum     = errors.New("slip39: bad checksum")
)

// Share is a parsed SLIP-39 share's header metadata (Tier 1: no secret value
// reconstruction). Fields are decoded from the share's bit layout; the RS1024
// checksum has been verified. Mnemonic holds the canonical (uppercase) words.
type Share struct {
	Mnemonic        []string
	Identifier      int  // 15-bit
	Extendable      bool // ext flag (selects the RS1024 customization string)
	IterationExp    int  // 4-bit
	GroupIndex      int  // 4-bit
	GroupThreshold  int  // decoded (stored + 1)
	GroupCount      int  // decoded (stored + 1)
	MemberIndex     int  // 4-bit
	MemberThreshold int  // decoded (stored + 1)
}

const (
	wordsShort = 20 // 128-bit
	wordsLong  = 33 // 256-bit (unsupported in Tier 1)
)

// rs1024GEN / rs1024Polymod / rs1024Verify implement the SLIP-0039 RS1024
// checksum over GF(1024) (error-detection only — NOT secret handling).
var rs1024GEN = [10]uint32{
	0xe0e040, 0x1c1c080, 0x3838100, 0x7070200, 0xe0e0009,
	0x1c0c2412, 0x38086c24, 0x3090fc48, 0x21b1f890, 0x3f3f120,
}

func rs1024Polymod(values []int) uint32 {
	chk := uint32(1)
	for _, v := range values {
		b := chk >> 20
		chk = (chk&0xfffff)<<10 ^ uint32(v)
		for i := 0; i < 10; i++ {
			if (b>>uint(i))&1 != 0 {
				chk ^= rs1024GEN[i]
			}
		}
	}
	return chk
}

func rs1024Verify(cs string, data []int) bool {
	vals := make([]int, 0, len(cs)+len(data))
	for _, c := range []byte(cs) {
		vals = append(vals, int(c))
	}
	vals = append(vals, data...)
	return rs1024Polymod(vals) == 1
}

// exactWord returns the Word index for a case-insensitive EXACT wordlist match
// (ClosestWord is a prefix match, so verify LabelFor(w) == upper(word)).
func exactWord(word string) (Word, bool) {
	u := strings.ToUpper(word)
	w, _ := ClosestWord(u)
	if w < 0 || LabelFor(w) != u {
		return -1, false
	}
	return w, true
}

// ParseShare validates a SLIP-39 share mnemonic (Tier 1, 128-bit/20-word only)
// and returns its decoded header. Checks: exactly 20 words, all in the wordlist
// (case-insensitive), valid RS1024 checksum (customization string per the ext
// bit), nothing else (no secret reconstruction). A 33-word (256-bit) share is
// rejected as unsupported. Returns a classifiable sentinel error on failure.
func ParseShare(mnemonic string) (Share, error) {
	fields := strings.Fields(mnemonic)
	switch len(fields) {
	case wordsShort:
	case wordsLong:
		return Share{}, errUnsupportedSize
	default:
		return Share{}, errWrongLength
	}
	indices := make([]int, len(fields))
	canonical := make([]string, len(fields))
	for i, f := range fields {
		w, ok := exactWord(f)
		if !ok {
			return Share{}, errNotInWordlist
		}
		indices[i] = int(w)
		canonical[i] = LabelFor(w)
	}
	// First 4 words = the 40-bit header. uint64 is REQUIRED: on RP2350/TinyGo
	// int is 32-bit and a 40-bit shift would overflow.
	hdr := uint64(indices[0])<<30 | uint64(indices[1])<<20 | uint64(indices[2])<<10 | uint64(indices[3])
	ext := (hdr>>24)&1 == 1
	cs := "shamir"
	if ext {
		cs = "shamir_extendable"
	}
	if !rs1024Verify(cs, indices) {
		return Share{}, errBadChecksum
	}
	return Share{
		Mnemonic:        canonical,
		Identifier:      int(hdr >> 25),
		Extendable:      ext,
		IterationExp:    int((hdr >> 20) & 0xf),
		GroupIndex:      int((hdr >> 16) & 0xf),
		GroupThreshold:  int((hdr>>12)&0xf) + 1,
		GroupCount:      int((hdr>>8)&0xf) + 1,
		MemberIndex:     int((hdr >> 4) & 0xf),
		MemberThreshold: int(hdr&0xf) + 1,
	}, nil
}

// Describe returns a short human label for a ParseShare error (for the GUI), or
// "" for nil; unknown errors → "invalid".
func Describe(err error) string {
	switch {
	case err == nil:
		return ""
	case errors.Is(err, errBadChecksum):
		return "bad checksum"
	case errors.Is(err, errNotInWordlist):
		return "unknown word"
	case errors.Is(err, errUnsupportedSize):
		return "256-bit not supported"
	case errors.Is(err, errWrongLength):
		return "wrong length"
	default:
		return "invalid"
	}
}
```

- [ ] **Step 4: Run to verify it passes** — Run: `/home/bcg/.local/go/bin/go test ./slip39/...`
Expected: PASS (incl. the pre-existing wordlist tests).

- [ ] **Step 5: Commit**
```bash
git add slip39/share.go slip39/share_test.go
git commit -S -s -m "slip39: add ParseShare + RS1024 checksum for share entry validation"
```

---

## Task 2: C2/C3 — gui re-enablement (atomic)

**Files:** Create `gui/slip39_polish.go`, `gui/slip39_polish_test.go`; modify `gui/gui.go`.

**Context:** atomic because the `case slip39words.Share:` type-switch case + `engraveSLIP39` must land together to compile, and the `case 3:` entry references `slip39words.ParseShare`/`showError` (added this task). All `fmt`-using code goes in the new `gui/slip39_polish.go` (which imports `fmt`), so `gui.go` needs NO import changes (`case 3:` uses only `strings.Builder` [imported] + `slip39words` [imported] + in-package helpers). `inputSLIP39Flow(ctx, th, mnemonic, 0) bool` mutates `mnemonic` in place and returns true when filled. `EngraveScreen.Engrave(ctx, th)` is 2-arg (the dormant 3-arg `ops` form is stale). Model `confirmSLIP39Flow`/`showError` on `confirmCodex32Flow`/`showCodex32Error`.

- [ ] **Step 1: Write the failing tests** — `gui/slip39_polish_test.go`

```go
package gui

import (
	"testing"

	slip39words "seedhammer.com/slip39"
)

const slip39Duckling = "duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision keyboard"

func TestConfirmSLIP39Render(t *testing.T) {
	s, err := slip39words.ParseShare(slip39Duckling)
	if err != nil {
		t.Fatalf("ParseShare: %v", err)
	}
	ctx := NewContext(newPlatform())
	frame, quit := runUI(ctx, func() { confirmSLIP39Flow(ctx, &descriptorTheme, s) })
	defer quit()
	c, ok := frame()
	if !ok {
		t.Fatal("no frame")
	}
	if !uiContains(c, "id 7945") {
		t.Errorf("confirm should show id 7945; got %q", c)
	}
	if !uiContains(c, "member 1 of 1") {
		t.Errorf("confirm should show member 1 of 1; got %q", c)
	}
}

func TestEngraveSLIP39BackoutRecognized(t *testing.T) {
	s, err := slip39words.ParseShare(slip39Duckling)
	if err != nil {
		t.Fatalf("ParseShare: %v", err)
	}
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Button1) // Back at the confirm screen
	if !engraveObjectFlow(ctx, &descriptorTheme, s) {
		t.Error("cancel at SLIP-39 confirm returned false (→ \"Unknown format\"); want true (recognized)")
	}
}
```

- [ ] **Step 2: Run to verify it fails** — Run: `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestConfirmSLIP39Render|TestEngraveSLIP39BackoutRecognized'`
Expected: FAIL — `undefined: confirmSLIP39Flow`; and `engraveObjectFlow` has no `case slip39words.Share:` yet (it hits `default: return false`).

- [ ] **Step 3: Create `gui/slip39_polish.go`**

```go
package gui

import (
	"fmt"
	"image"

	"seedhammer.com/backup"
	"seedhammer.com/font/constant"
	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/layout"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
	slip39words "seedhammer.com/slip39"
)

// showError displays a dismissible error modal (Button3 dismisses) over a blank
// background; returns when dismissed or ctx.Done. (Generalizes showCodex32Error
// with a title parameter.)
func showError(ctx *Context, th *Colors, title, msg string) {
	errScr := &ErrorScreen{Title: title, Body: msg}
	for !ctx.Done {
		dims := ctx.Platform.DisplaySize()
		d, dismissed := errScr.Layout(ctx, th, dims)
		if dismissed {
			return
		}
		ctx.Frame(op.Layer(d, op.Color(&ctx.B, th.Background)))
	}
}

// confirmSLIP39Flow shows a pre-engrave review of a parsed SLIP-39 share.
// Back (Button1) → false; Engrave (Button3) → true.
func confirmSLIP39Flow(ctx *Context, th *Colors, s slip39words.Share) bool {
	lines := []string{
		fmt.Sprintf("id %d", s.Identifier),
		fmt.Sprintf("member %d of %d", s.MemberIndex+1, s.MemberThreshold),
	}
	if s.GroupCount > 1 {
		lines = append(lines, fmt.Sprintf("group %d of %d", s.GroupIndex+1, s.GroupCount))
	}
	lines = append(lines, fmt.Sprintf("%d words", len(s.Mnemonic)))

	backBtn := &Clickable{Button: Button1}
	engraveBtn := &Clickable{Button: Button3, AltButton: Center}
	for !ctx.Done {
		if backBtn.Clicked(ctx) {
			return false
		}
		if engraveBtn.Clicked(ctx) {
			return true
		}
		dims := ctx.Platform.DisplaySize()
		nav, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack},
			{Clickable: engraveBtn, Style: StylePrimary, Icon: assets.IconHammer},
		}...)
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, "Confirm SLIP-39 Share")

		screen := layout.Rectangle{Max: dims}
		_, content := screen.CutTop(leadingSize)
		content, _ = content.CutBottom(leadingSize)
		body := make([]op.Op, 0, len(lines))
		y := content.Min.Y + 8
		for _, ln := range lines {
			lbl, sz := widget.Labelw(&ctx.B, ctx.Styles.body, dims.X-2*8, th.Text, ln)
			body = append(body, lbl.Offset(image.Pt((dims.X-sz.X)/2, y)))
			y += sz.Y + 6
		}
		frameOps := append([]op.Op{nav, titleOp}, body...)
		frameOps = append(frameOps, op.Color(&ctx.B, th.Background))
		ctx.Frame(op.Layer(frameOps...))
	}
	return false
}

// engraveSLIP39 confirms a SLIP-39 share and engraves it verbatim. Always returns
// true (recognized/handled) — Back, a fit failure, and engrave-complete all
// return true, never falling to the caller's scanUnknownFormat ("Unknown format").
func engraveSLIP39(ctx *Context, th *Colors, scan slip39words.Share) bool {
	if !confirmSLIP39Flow(ctx, th, scan) {
		return true
	}
	seedDesc := backup.Seed{
		Mnemonic:     scan.Mnemonic, // canonical uppercase words; verbatim
		ShortestWord: slip39words.ShortestWord,
		LongestWord:  slip39words.LongestWord,
		Title:        fmt.Sprintf("%d #%d/%d", scan.Identifier, scan.MemberIndex+1, scan.MemberThreshold), // max "32767 #16/16" = 12 <= MaxTitleLen 18
		Font:         constant.Font,
	}
	params := ctx.Platform.EngraverParams()
	seedSide, err := backup.EngraveSeed(params, seedDesc)
	if err != nil {
		showError(ctx, th, "Too large", "Share doesn't fit a plate.")
		return true
	}
	plate, err := toPlate(seedSide, params)
	if err != nil {
		showError(ctx, th, "Too large", "Share doesn't fit a plate.")
		return true
	}
	for {
		if NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme) {
			return true
		}
	}
}
```

- [ ] **Step 4: Wire `engraveSLIP39` into `engraveObjectFlow`** — in `gui/gui.go`, replace the dormant `case slip39.Share:` comment block (`gui.go:1810-1840`, the `// TODO: re-enable SLIP39…` line through the closing `// }`) with a real case placed after the `case bip39.Mnemonic:` block:
```go
	case slip39words.Share:
		return engraveSLIP39(ctx, th, scan)
```
(Leave the `// TODO: re-enable SLIP39. See also nfcpoller.go.` line if you wish, or remove it; the `case codex32.String:` etc. below are unchanged.)

- [ ] **Step 5: Enable the menu choice** — in `gui/gui.go:1983`:
```go
		Choices: []string{"12 WORDS", "24 WORDS", "CODEX32", "SLIP-39"},
```

- [ ] **Step 6: Uncomment + fix the `case 3:` entry** — in `gui/gui.go` (`newInputFlow`, after the `case 2:` codex32 block, ~`:2002`), replace the commented block with:
```go
		case 3:
			mnemonic := emptySLIP39Mnemonic(20)
			if ok := inputSLIP39Flow(ctx, th, mnemonic, 0); !ok {
				break
			}
			share := new(strings.Builder)
			for i, w := range mnemonic {
				if i > 0 {
					share.WriteByte(' ')
				}
				share.WriteString(slip39words.LabelFor(w))
			}
			s, err := slip39words.ParseShare(share.String())
			if err != nil {
				showError(ctx, th, "Invalid SLIP-39 share", slip39words.Describe(err))
				break
			}
			return s, true
```
(`strings` and `slip39words` are already imported in `gui.go`; `showError`/`emptySLIP39Mnemonic`/`inputSLIP39Flow` are in-package. NO `fmt` needed here. `break` exits the switch → the `newInputFlow` loop re-shows the menu, matching the codex32/back-out behavior.)

- [ ] **Step 7: gofmt + build** — Run: `/home/bcg/.local/go/bin/go build ./gui/ && /home/bcg/.local/go/bin/go vet ./gui/...`
Expected: clean build (the pre-existing `gui/op/draw_test.go` go1.26 `testing.ArtifactDir` vet note is unrelated). If gofmt reorders the new file's imports, accept it.

- [ ] **Step 8: Run the targeted + guard tests** — Run: `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestConfirmSLIP39Render|TestEngraveSLIP39BackoutRecognized|TestInputSeedCodex32|TestWordKeyboardScreen|TestEngraveCodex32BackoutNotUnknown'`
Expected: PASS — the new SLIP-39 tests + the codex32/BIP-39 guards.

- [ ] **Step 9: Full suite** — Run: `/home/bcg/.local/go/bin/go test ./...`
Expected: PASS.

- [ ] **Step 10: Commit**
```bash
git add gui/slip39_polish.go gui/slip39_polish_test.go gui/gui.go
git commit -S -s -m "gui(slip39): enable share entry + verbatim engrave (confirm + RS1024-gated)"
```

---

## Final: whole-diff adversarial execution review (mandatory)

Dispatch an independent opus adversarial execution review over the entire diff vs `9b0a02c`. Persist verbatim to `design/agent-reports/seedhammer-slip39-execution-review.md`. Fold any Critical/Important to clean.

Review focus:
- `ParseShare` cannot panic on any input (empty, 1-word, 33-word, non-ASCII, mixed case); RS1024 `uint32` polymod + the `uint64` header are correct on a 32-bit target; the `ext`-selected customization string is load-bearing (testify only verifies under `shamir_extendable`); the decoded fields match the pre-verified vectors (7945/1-of-1; 29019/ext).
- `exactWord` is genuinely exact (rejects prefixes like "academi"); case-insensitive both ways (lowercase vectors + uppercase GUI input).
- `engraveSLIP39` always returns `true` (no "Unknown format" on cancel/fit-failure); engraves `scan.Mnemonic` verbatim; the title ≤18 chars; `Engrave(ctx, th)` 2-arg.
- `gui.go` got NO new imports; `case 3:` uses only `strings.Builder`/`slip39words`/in-package helpers; the menu has 4 choices and index 3 routes to `inputSLIP39Flow`.
- Scope: `codex32`/`mdmk.go`/`slip39/wordlist.*`/`gui/scan.go` untouched; no Shamir/Feistel/PBKDF2/passphrase; `TestInputSeedCodex32`/`TestWordKeyboardScreen` green; signed + DCO.
- Consider an E2E test driving the menu→20-word entry→engrave if feasible (the SLIP-39 keyboard uses word-completion — fiddly; component tests + ParseShare vectors are the primary gate).

Then **superpowers:finishing-a-development-branch** — no upstream PR: merge `feat/slip39-entry-engrave` into fork `main` (no-ff, signed) and push to `bg002h`.

---

## Self-Review (author)

- **Spec coverage:** C1→Task 1, C2/C3→Task 2. The §8 resolved decisions (Tier-1 entry+engrave, in-tree RS1024, 128-bit/20-word, keypad-only/NFC-off, confirm screen) all realized; recovery/passphrase/256-bit/multi-group excluded.
- **R0/R1/R2/R3/R4 folds honored:** `scan.Mnemonic` (not `Words()`); 2-arg `Engrave`; title `"%d #%d/%d"` ≤18; `fmt` confined to the new file so `gui.go` imports unchanged; `ParseShare(string)`; uppercase normalization in `exactWord`; Identifier=7945 hard-coded from the re-verified vector; 4-arg `showError`.
- **Type consistency:** `Share`/`ParseShare`/`Describe`/`exactWord`/`rs1024*` (slip39 pkg); `showError`/`confirmSLIP39Flow`/`engraveSLIP39` (gui pkg) match across tasks/tests. `slip39words.Share`/`.ParseShare`/`.Describe`/`.LabelFor`/`.ShortestWord`/`.LongestWord` are the exported surface Task 1 provides.
- **Atomicity:** Task 1 (slip39) is independent; Task 2 bundles the type-switch case + helpers + entry so the package compiles. `uint64` header pinned for the 32-bit target.
