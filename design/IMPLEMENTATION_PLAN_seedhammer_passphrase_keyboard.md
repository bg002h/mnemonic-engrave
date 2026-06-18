# SeedHammer Passphrase Keyboard Widget (Slice 2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** A new standalone `gui.PassphraseKeyboard` widget — 3 case-preserving pages (lowercase / UPPERCASE / symbols+digits), a function row (page-cycle / space / reveal / backspace), a masked-by-default entry readout — for later use by the Slice-3 passphrase flow. No flow, no `MnemonicSeed`/`password` threading, no fingerprint logic.

**Architecture:** A new file `gui/passphrase_keyboard.go` defines `PassphraseKeyboard` with its OWN `ppKey`/`ppAction` cell model and its own `NewPassphraseKeyboard`/`Update`/`Layout`/`Clear`, adapting the shared `Keyboard`'s layout/nav math (`gui.go:861-914`, `1056-1145`, `1157-1209`, `1211-1272`) but **dropping both `ToUpper` sites** (commit `:1153`, render `:1244`) and the `RuneEvent` `ToLower` (`:1132`). Space is a `ppRune` key (`r=' '`, label `"space"`) so it is `RuneEvent`-matchable. The shared `Keyboard` and its 3 live consumers (BIP-39/codex32/SLIP-39) are UNTOUCHED.

**Tech Stack:** Go/TinyGo. Host tests `/home/bcg/.local/go/bin/go test ./gui/...`.

**Base:** fork `main` `06b57f3`. Branch `feat/passphrase-keyboard`. Fork-side only; no upstream PR.

**Spec:** `design/SPEC_seedhammer_passphrase_keyboard.md` (R0 GREEN at R1). **PLAN R0 GATE: PASSED (GREEN — 0C/0I at R1)** — R0 caught 2 compile-fatal unused imports (C-1/C-2) + the backspace margin (I-1) + the vestigial adjust param (I-2); all folded. `design/agent-reports/seedhammer-passphrase-keyboard-plan-review-R{0,1}.md`. Cleared for implementation.

**Reused primitives (verbatim from `06b57f3`):** consts `keyPadX=3,keyPadY=4,keyCornerRadius=3,keyLineWidth=1,cornerRadius=5` (`gui.go:49-57`), `leadingSize=44` (`theme.go:43`); `mulAlpha` (`gui.go:1274`), `theme.inactiveMask=0x55` (`theme.go`); `assets.KeyBackspace` (`*alpha4.Image`); `widget.Label(buf,st,col,txt)`, `widget.Labelf(buf,st,col,fmt,…)`, `widget.Labelw(buf,st,width,col,txt)`; `op.Input/.Clip`, `op.Color`, `op.Compose`, `op.Mask`, `op.RoundedRect2`, `op.RoundedOutline2`, `op.Layer`, `(op.Op).Offset`; `InputTracker.Next(ctx, filters…)`, `ButtonFilter`/`RuneFilter`, `Event.AsButton()/AsRune()`, consts `Up/Down/Left/Right/Center` (`event.go:19-32`); `Clickable.Clicked`; `ctx.Styles.keyboard` (poppins.Bold25), `ctx.Styles.word` (comfortaa.Bold17), `ctx.Styles.body`; `ctx.Platform.DisplaySize()`. Test harness: `NewContext(newPlatform())`, `runUI`, `uiContains`, `descriptorTheme`, `runes`/`click`/`press(&ctx.Router, …)`.

---

## File Structure

| File | Responsibility | Tasks |
|---|---|---|
| `gui/passphrase_keyboard.go` *(new)* | `ppAction`/`ppKey`, `PassphraseKeyboard`, page consts, `NewPassphraseKeyboard`, `Clear`, `Valid`, commit/action dispatch, `Update`, `Layout` (readout + grid). | 1,2,3 |
| `gui/passphrase_keyboard_test.go` *(new)* | Construction, case-preservation (runes + clicks/D-pad), page-cycle, space, backspace, reveal, masked/revealed render. | 1,2,3 |
| shared `Keyboard` + BIP-39/codex32/SLIP-39 + guard tests *(unchanged — must stay green)* | | guard |

**Commit hygiene:** explicit paths. Signed + DCO: `git commit -S -s` (fall back to `-s` if signing unavailable, say so).

---

## Task 0: Worktree + clean baseline

- [ ] **Step 1:** `cd /scratch/code/shibboleth/seedhammer && git worktree add /scratch/code/shibboleth/seedhammer-wt-passphrase -b feat/passphrase-keyboard 06b57f3 && cd /scratch/code/shibboleth/seedhammer-wt-passphrase && git config user.name "Brian Goss" && git config user.email "goss.brian@gmail.com"`
- [ ] **Step 2:** Baseline — `/home/bcg/.local/go/bin/go test ./gui/...` → PASS. If red, STOP.

---

## Task 1: Types + construction + `Clear`/`Valid`

**Files:** create `gui/passphrase_keyboard.go`, `gui/passphrase_keyboard_test.go`.

**Context:** mirror `NewKeyboard`'s cell-sizing + row-centering (`gui.go:866-911`) but: per-page grids; NO backspace appended to the alphabet (the function row provides it); a function row whose keys have **per-key widths** (label-measured), unlike the uniform letter cells. `ppKey` carries its own `size`. Cursor seeded to center via `Clear` (per `Keyboard.Clear` `gui.go:916-920`).

- [ ] **Step 1: Write the failing test** — `gui/passphrase_keyboard_test.go`

```go
package gui

import "testing"

func TestPassphraseKeyboardConstruction(t *testing.T) {
	ctx := NewContext(newPlatform())
	k := NewPassphraseKeyboard(ctx)
	if len(k.pages) != 3 {
		t.Fatalf("pages = %d, want 3", len(k.pages))
	}
	for p := 0; p < 3; p++ {
		rows := k.pages[p]
		if len(rows) != 4 { // 3 letter/symbol rows + 1 function row
			t.Errorf("page %d: %d rows, want 4", p, len(rows))
		}
		fr := rows[len(rows)-1]
		if len(fr) != 4 {
			t.Errorf("page %d function row: %d keys, want 4 (page-cycle/space/reveal/backspace)", p, len(fr))
		}
		// function-row actions, in order.
		wantAct := []ppAction{ppPageCycle, ppRune, ppReveal, ppBackspace}
		for i, a := range wantAct {
			if fr[i].action != a {
				t.Errorf("page %d funcrow[%d].action = %v, want %v", p, i, fr[i].action, a)
			}
		}
		if fr[1].r != ' ' {
			t.Errorf("page %d space key r = %q, want ' '", p, fr[1].r)
		}
	}
	// page 0 row 0 is lowercase qwerty; page 1 uppercase.
	if k.pages[0][0][0].r != 'q' || k.pages[1][0][0].r != 'Q' {
		t.Errorf("page0[0][0]=%q page1[0][0]=%q, want 'q'/'Q'", k.pages[0][0][0].r, k.pages[1][0][0].r)
	}
	// Clear resets.
	k.Fragment = "secret"
	k.page = 2
	k.revealed = true
	k.Clear()
	if k.Fragment != "" || k.page != 0 || k.revealed {
		t.Errorf("after Clear: Fragment=%q page=%d revealed=%v, want \"\"/0/false", k.Fragment, k.page, k.revealed)
	}
}
```

- [ ] **Step 2: Run to verify it fails** — `/home/bcg/.local/go/bin/go test ./gui/ -run TestPassphraseKeyboardConstruction` → FAIL (`undefined: NewPassphraseKeyboard`).

- [ ] **Step 3: Implement types + construction** — create `gui/passphrase_keyboard.go`:

```go
package gui

import (
	"image"
	"math"
	"strings"
	"unicode/utf8"

	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
)
// (R0 C-1/C-2: NO "fmt" — Layout uses widget.Labelf, not stdlib fmt; NO
// "seedhammer.com/gui/layout" — this widget references no layout.* symbol.)

// Passphrase keyboard pages (case-preserving, printable-ASCII).
const (
	ppPageLower   = "qwertyuiop\nasdfghjkl\nzxcvbnm"
	ppPageUpper   = "QWERTYUIOP\nASDFGHJKL\nZXCVBNM"
	ppPageSymbols = "1234567890\n-/:;()&$@\"\n.,?!'+=_#"
)

var ppPages = [3]string{ppPageLower, ppPageUpper, ppPageSymbols}

// ppPageCycleLabel[p] is the cap shown on page p (it names the NEXT page).
var ppPageCycleLabel = [3]string{"ABC", "?123", "abc"}

type ppAction int

const (
	ppRune      ppAction = iota // commit k.r (space is ppRune with r==' ')
	ppPageCycle                 // page = (page+1)%3
	ppReveal                    // toggle revealed
	ppBackspace                 // delete last rune
)

type ppKey struct {
	r      rune        // literal char for ppRune (case as-stored); 0 otherwise
	label  string      // cap for special keys (page-cycle/space); "" → render %c r
	action ppAction
	pos    image.Point // top-left within the page grid
	size   image.Point // cell glyph extent (per-key; function row varies)
	clk    Clickable
}

type PassphraseKeyboard struct {
	Fragment string
	page     int
	revealed bool

	pages [3][][]ppKey
	size  [3]image.Point

	row, col int
	inp      InputTracker
}

// NewPassphraseKeyboard builds the 3 page grids (each = the page's letter/symbol
// rows + a shared-shape function row) with per-key positions, adapting
// NewKeyboard's cell-sizing + row-centering.
func NewPassphraseKeyboard(ctx *Context) *PassphraseKeyboard {
	k := new(PassphraseKeyboard)
	style := ctx.Styles.keyboard
	cell := style.Measure(math.MaxInt, "W") // uniform letter-cell glyph extent
	const margin = 2
	letterW := cell.X + 2*keyPadX + margin
	rowH := cell.Y + 2*keyPadY + margin

	for p := 0; p < 3; p++ {
		var rows [][]ppKey
		for _, line := range strings.Split(ppPages[p], "\n") {
			var row []ppKey
			for _, r := range line {
				row = append(row, ppKey{r: r, action: ppRune, size: cell})
			}
			rows = append(rows, row)
		}
		// Function row: page-cycle, space (a ppRune with r==' '), reveal, backspace.
		fr := []ppKey{
			{label: ppPageCycleLabel[p], action: ppPageCycle},
			{r: ' ', label: "space", action: ppRune},
			{label: "show", action: ppReveal}, // label re-derived from revealed in Layout
			{action: ppBackspace},
		}
		for i := range fr {
			fr[i].size = ppKeyExtent(ctx, fr[i], cell)
		}
		rows = append(rows, fr)

		// Position: letter rows use the uniform letterW; the function row uses
		// per-key widths. Each row is horizontally centered in maxw.
		maxw := 0
		for _, row := range rows[:len(rows)-1] {
			if w := len(row) * letterW; w > maxw {
				maxw = w
			}
		}
		if w := ppRowWidth(fr, margin); w > maxw {
			maxw = w
		}
		y := 0
		for ri, row := range rows {
			if ri < len(rows)-1 { // letter row (uniform letterW)
				w := len(row) * letterW
				x := (maxw - w) / 2
				for j := range row {
					row[j].pos = image.Pt(x+j*letterW+keyPadX, y+keyPadY)
					row[j].size = cell
				}
			} else { // function row (per-key widths)
				w := ppRowWidth(row, margin)
				x := (maxw - w) / 2
				for j := range row {
					cw := row[j].size.X + 2*keyPadX + margin
					row[j].pos = image.Pt(x+keyPadX, y+keyPadY)
					x += cw
				}
			}
			y += rowH
		}
		k.pages[p] = rows
		k.size[p] = image.Pt(maxw, y-margin)
	}
	k.Clear()
	return k
}

// ppKeyExtent measures a function key's glyph extent (label, or the backspace icon).
func ppKeyExtent(ctx *Context, key ppKey, cell image.Point) image.Point {
	switch key.action {
	case ppBackspace:
		b := assets.KeyBackspace.Bounds()
		return image.Pt(b.Min.X*2+b.Dx(), cell.Y) // R0 I-1: include the Min.X margin (matches NewKeyboard gui.go:868)
	default:
		lbl := key.label
		if lbl == "" {
			lbl = string(key.r)
		}
		return image.Pt(ctx.Styles.keyboard.Measure(math.MaxInt, lbl).X, cell.Y)
	}
}

func ppRowWidth(row []ppKey, margin int) int {
	w := 0
	for _, key := range row {
		w += key.size.X + 2*keyPadX + margin
	}
	return w
}

func (k *PassphraseKeyboard) Clear() {
	k.Fragment = ""
	k.page = 0
	k.revealed = false
	rows := k.pages[k.page]
	k.row = len(rows) / 2
	k.col = len(rows[k.row]) / 2
}

// Valid: backspace valid iff Fragment non-empty; everything else always.
func (k *PassphraseKeyboard) Valid(key ppKey) bool {
	if key.action == ppBackspace {
		return k.Fragment != ""
	}
	return true
}
```

> The `image`/`utf8`/`layout`/`op`/`widget`/`fmt`/`assets` imports are all used by Tasks 1-3 combined (`utf8` in the mask readout, `fmt` in Layout's reveal label, etc.). If Go flags an unused import after Task 1 alone, that's expected — Tasks 2-3 land in the same file before the package is built/committed as a unit (or comment the not-yet-used imports until their task). Recommended: implement Tasks 1→2→3 in sequence, build/commit once at the end of each task only if the package compiles; otherwise commit Task 1+2+3 together.

- [ ] **Step 4: Run to verify it passes** — `/home/bcg/.local/go/bin/go test ./gui/ -run TestPassphraseKeyboardConstruction`
Expected: PASS. (If unused-import errors block compilation because `Update`/`Layout` aren't written yet, temporarily comment `op`/`widget`/`utf8`/`layout`/`fmt` and the unused helpers, OR proceed to Tasks 2-3 and run this test at Task 3 Step — note which.)

- [ ] **Step 5: Commit** (only if the package compiles standalone; else fold into Task 2's commit)
```bash
git add gui/passphrase_keyboard.go gui/passphrase_keyboard_test.go
git commit -S -s -m "gui(passphrase): PassphraseKeyboard types + construction (3 pages + function row)"
```

---

## Task 2: Input + commit/action dispatch (`Update`)

**Files:** modify `gui/passphrase_keyboard.go`, `gui/passphrase_keyboard_test.go`.

**Context:** adapt `Keyboard.Update` (`gui.go:1056-1145`) + `adjust`/`adjustCol` (`:1157-1209`): touch (`clk.Clicked`), D-pad Left/Right/Up/Down + Center commit, RuneEvent. **Drop the `ToUpper` commit and the `ToLower` RuneEvent match**; RuneEvent searches ALL pages' `ppRune` keys (cross-page, case-sensitive) and commits as-is without switching `page`. Center/click commit dispatches by `action`.

- [ ] **Step 1: Write the failing tests** — append to `gui/passphrase_keyboard_test.go`

```go
func TestPassphraseRuneEntryCrossPage(t *testing.T) {
	ctx := NewContext(newPlatform())
	k := NewPassphraseKeyboard(ctx)
	// 'A' is only on page 1, '1'/'!' on page 2, 'b'/' ' on page 0 — cross-page,
	// case-honoring, no page switch.
	runes(&ctx.Router, "Ab 1!")
	for k.Update(ctx) {
	}
	if k.Fragment != "Ab 1!" {
		t.Errorf("Fragment = %q, want %q", k.Fragment, "Ab 1!")
	}
	if k.page != 0 {
		t.Errorf("page = %d, want 0 (RuneEvent must not switch pages)", k.page)
	}
}

func TestPassphraseActions(t *testing.T) {
	ctx := NewContext(newPlatform())
	k := NewPassphraseKeyboard(ctx)
	k.Fragment = "abc"
	k.commit(ppKey{action: ppBackspace})
	if k.Fragment != "ab" {
		t.Errorf("backspace: %q, want \"ab\"", k.Fragment)
	}
	k.commit(ppKey{r: ' ', action: ppRune})
	if k.Fragment != "ab " {
		t.Errorf("space: %q", k.Fragment)
	}
	pg := k.page
	k.commit(ppKey{action: ppPageCycle})
	if k.page != (pg+1)%3 {
		t.Errorf("page-cycle: page=%d, want %d", k.page, (pg+1)%3)
	}
	rev := k.revealed
	k.commit(ppKey{action: ppReveal})
	if k.revealed == rev {
		t.Errorf("reveal toggle did not flip revealed")
	}
	// backspace on empty Fragment is a no-op (Valid gates it, but commit must be safe).
	k.Fragment = ""
	k.commit(ppKey{action: ppBackspace})
	if k.Fragment != "" {
		t.Errorf("backspace on empty: %q, want \"\"", k.Fragment)
	}
}
```

- [ ] **Step 2: Run to verify it fails** — `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestPassphraseRuneEntryCrossPage|TestPassphraseActions'` → FAIL (`undefined: (*PassphraseKeyboard).Update` / `.commit`).

- [ ] **Step 3: Implement `commit` + `Update`** — append to `gui/passphrase_keyboard.go`

```go
// commit applies a key's action.
func (k *PassphraseKeyboard) commit(key ppKey) {
	switch key.action {
	case ppRune:
		k.Fragment += string(key.r) // NO ToUpper — case preserved
	case ppBackspace:
		if k.Fragment != "" {
			_, n := utf8.DecodeLastRuneInString(k.Fragment)
			k.Fragment = k.Fragment[:len(k.Fragment)-n]
		}
	case ppPageCycle:
		k.page = (k.page + 1) % 3
		rows := k.pages[k.page]
		k.row = len(rows) / 2
		k.col = len(rows[k.row]) / 2
	case ppReveal:
		k.revealed = !k.revealed
	}
}

func (k *PassphraseKeyboard) keys() [][]ppKey { return k.pages[k.page] }

func (k *PassphraseKeyboard) Update(ctx *Context) bool {
	k.adjust()
	cur := k.keys()
	for i, row := range cur {
		for j := range row {
			key := &row[j]
			if k.Valid(*key) && key.clk.Clicked(ctx) {
				k.row, k.col = i, j
				k.commit(*key)
				return true
			}
		}
	}
	for {
		e, ok := k.inp.Next(ctx, ButtonFilter(Left), ButtonFilter(Right), ButtonFilter(Up), ButtonFilter(Down), ButtonFilter(Center), RuneFilter())
		if !ok {
			break
		}
		if e, ok := e.AsButton(); ok {
			if !e.Pressed {
				continue
			}
			cur = k.keys()
			switch e.Button {
			case Left:
				k.moveCol(-1)
			case Right:
				k.moveCol(+1)
			case Up:
				k.moveRow(-1)
			case Down:
				k.moveRow(+1)
			case Center:
				k.commit(cur[k.row][k.col])
				return true
			}
		}
		if e, ok := e.AsRune(); ok {
			// Cross-page, case-sensitive, no page switch.
			for _, page := range k.pages {
				for _, row := range page {
					for _, key := range row {
						if key.action == ppRune && key.r == e.Rune {
							k.commit(key)
							return true
						}
					}
				}
			}
		}
	}
	return false
}

func (k *PassphraseKeyboard) moveCol(d int) {
	row := k.keys()[k.row]
	next := k.col
	for {
		next = (next + d + len(row)) % len(row)
		if k.Valid(row[next]) {
			k.col = next
			k.adjust()
			return
		}
		if next == k.col { // full loop, none valid (shouldn't happen)
			return
		}
	}
}

func (k *PassphraseKeyboard) moveRow(d int) {
	rows := k.keys()
	n := len(rows)
	next := k.row
	for {
		next = (next + d + n) % n
		if k.adjustCol(next) {
			k.adjust()
			return
		}
		if next == k.row {
			return
		}
	}
}

// adjust / adjustCol: same nearest-valid-key logic as Keyboard (gui.go:1157-1209),
// over the ACTIVE page's grid (k.keys()), using ppKey.pos. No allowBackspace param
// (R0 I-2): Valid already excludes an empty-Fragment backspace, so the shared
// keyboard's allowBackspace plumbing is vestigial here.
func (k *PassphraseKeyboard) adjust() {
	rows := k.keys()
	dist := int(1e6)
	current := rows[k.row][k.col].pos
	found := false
	for i, row := range rows {
		for j, key := range row {
			if !k.Valid(key) {
				continue
			}
			d := key.pos.Sub(current)
			if d2 := d.X*d.X + d.Y*d.Y; d2 < dist {
				dist = d2
				k.row, k.col = i, j
				found = true
			}
		}
	}
	if !found {
		k.row = len(rows) - 1
		k.col = len(rows[k.row]) - 1
	}
}

func (k *PassphraseKeyboard) adjustCol(row int) bool {
	rows := k.keys()
	dist := int(1e6)
	found := false
	x := rows[k.row][k.col].pos.X
	for i, key := range rows[row] {
		if !k.Valid(key) {
			continue
		}
		found = true
		k.row = row
		d := rows[row][i].pos.X - x
		if d < 0 {
			d = -d
		}
		if d < dist {
			dist = d
			k.col = i
		}
	}
	return found
}
```

> Note vs the shared `Keyboard`: `adjust` here uses the active page's grid and gates only on `Valid` (the empty-Fragment backspace is already excluded by `Valid`). The `allowBackspace` param is dropped (R0 I-2) — `adjust()` has no parameter; all call sites are `k.adjust()`.

- [ ] **Step 4: Run to verify it passes** — `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestPassphraseRuneEntryCrossPage|TestPassphraseActions|TestPassphraseKeyboardConstruction'`
Expected: PASS.

- [ ] **Step 5: D-pad commit test** — append + run:
```go
func TestPassphraseDpadCommit(t *testing.T) {
	ctx := NewContext(newPlatform())
	k := NewPassphraseKeyboard(ctx)
	// From the centered cursor, Center commits the cursor key (a letter on page 0).
	before := k.Fragment
	press(&ctx.Router, Center)
	for k.Update(ctx) {
	}
	if len(k.Fragment) != len(before)+1 {
		t.Errorf("Center commit appended %d chars, want 1 (Fragment=%q)", len(k.Fragment)-len(before), k.Fragment)
	}
}
```
Run: `/home/bcg/.local/go/bin/go test ./gui/ -run TestPassphraseDpadCommit` → PASS.

- [ ] **Step 6: Commit**
```bash
git add gui/passphrase_keyboard.go gui/passphrase_keyboard_test.go
git commit -S -s -m "gui(passphrase): input + commit dispatch (cross-page case-honoring RuneEvent, D-pad, actions)"
```

---

## Task 3: `Layout` (masked readout + grid render, no ToUpper)

**Files:** modify `gui/passphrase_keyboard.go`, `gui/passphrase_keyboard_test.go`.

**Context:** adapt `Keyboard.Layout` (`gui.go:1211-1272`) over the active page's grid: render each key (dim invalid via `mulAlpha(_, theme.inactiveMask)`, fill the active `row,col`), **using the label for special keys and `"%c", key.r` (NO `ToUpper`) for `ppRune`**, backspace via the `assets.KeyBackspace` image; register `op.Input(&ctx.B, &k.keys()[i][j].clk).Clip(...)` for touch. Above the grid, render the masked/revealed readout via `widget.Labelw`. Return the COMBINED extent.

- [ ] **Step 1: Write the failing test** — append to `gui/passphrase_keyboard_test.go`

```go
func passphraseFrame(t *testing.T, drive func(r *EventRouter)) string {
	t.Helper()
	ctx := NewContext(newPlatform())
	k := NewPassphraseKeyboard(ctx)
	frame, quit := runUI(ctx, func() {
		for !ctx.Done {
			for k.Update(ctx) {
			}
			fop, _ := k.Layout(ctx, &descriptorTheme) // 'fop' not 'op' — avoid shadowing the op package (R0 M-1)
			ctx.Frame(fop)
		}
	})
	defer quit()
	if drive != nil {
		drive(&ctx.Router)
	}
	c, ok := frame()
	if !ok {
		t.Fatal("no frame")
	}
	return c
}

func TestPassphraseMaskReveal(t *testing.T) {
	// Masked by default: typing 4 runes shows '*'×4, not the cleartext.
	c := passphraseFrame(t, func(r *EventRouter) { runes(r, "ab1!") })
	if !uiContains(c, "****") {
		t.Errorf("masked readout: want \"****\"; got %q", c)
	}
	if uiContains(c, "ab1!") {
		t.Errorf("masked readout leaked cleartext: %q", c)
	}
	// Reveal via the reveal key (drive by D-pad to it, or assert through a second flow):
	// simplest: a dedicated test toggling revealed then rendering.
	ctx := NewContext(newPlatform())
	k := NewPassphraseKeyboard(ctx)
	k.Fragment = "ab1!"
	k.revealed = true
	frame, quit := runUI(ctx, func() {
		for !ctx.Done {
			for k.Update(ctx) {
			}
			o, _ := k.Layout(ctx, &descriptorTheme)
			ctx.Frame(o)
		}
	})
	defer quit()
	c2, ok := frame()
	if !ok {
		t.Fatal("no frame")
	}
	if !uiContains(c2, "ab1!") {
		t.Errorf("revealed readout: want cleartext \"ab1!\"; got %q", c2)
	}
}
```

- [ ] **Step 2: Run to verify it fails** — `/home/bcg/.local/go/bin/go test ./gui/ -run TestPassphraseMaskReveal` → FAIL (`undefined: (*PassphraseKeyboard).Layout`).

- [ ] **Step 3: Implement `Layout`** — append to `gui/passphrase_keyboard.go`

```go
// Layout renders the masked/revealed readout above the active page's key grid.
// The returned image.Point is the COMBINED extent (readout + grid).
func (k *PassphraseKeyboard) Layout(ctx *Context, th *Colors) (op.Op, image.Point) {
	// Readout: masked '*'×len (default) or cleartext.
	shown := k.Fragment
	if !k.revealed {
		shown = strings.Repeat("*", utf8.RuneCountInString(k.Fragment))
	}
	readoutOp, readoutSz := widget.Labelw(&ctx.B, ctx.Styles.word, math.MaxInt, th.Text, shown)
	const readoutGap = 8

	gridY := readoutSz.Y + readoutGap
	var content op.Op
	rows := k.keys()
	for i, row := range rows {
		for j, key := range row {
			valid := k.Valid(key)
			bgcol := th.Text
			col := th.Text
			active := false
			switch {
			case !valid:
				bgcol = mulAlpha(bgcol, theme.inactiveMask)
				col = bgcol
			case i == k.row && j == k.col:
				active = true
				col = th.Background
			}
			bgsz := key.size
			bgr := image.Rectangle{Max: bgsz}
			inpOp := op.Input(&ctx.B, &k.pages[k.page][i][j].clk).Clip(bgr)
			var keyOp op.Op
			var sz image.Point
			switch {
			case key.action == ppBackspace:
				icn := assets.KeyBackspace
				sz = image.Pt(bgsz.X, icn.Bounds().Dy())
				keyOp = op.Compose(op.Color(&ctx.B, col), op.Mask(&ctx.B, icn))
			case key.label != "" && key.action == ppReveal:
				lbl := "show"
				if k.revealed {
					lbl = "hide"
				}
				keyOp, sz = widget.Labelf(&ctx.B, ctx.Styles.keyboard, col, "%s", lbl)
			case key.label != "":
				keyOp, sz = widget.Labelf(&ctx.B, ctx.Styles.keyboard, col, "%s", key.label)
			default:
				keyOp, sz = widget.Labelf(&ctx.B, ctx.Styles.keyboard, col, "%c", key.r) // NO ToUpper
			}
			keyOp = keyOp.Offset(bgsz.Sub(sz).Div(2))
			bgr.Min.X -= keyPadX
			bgr.Max.X += keyPadX
			bgr.Min.Y -= keyPadY
			bgr.Max.Y += keyPadY
			bgOp := op.Color(&ctx.B, bgcol)
			var mask op.MaskOp
			if active {
				mask = op.RoundedRect2(&ctx.B, bgr, keyCornerRadius)
			} else {
				mask = op.RoundedOutline2(&ctx.B, bgr, keyCornerRadius, keyLineWidth)
			}
			btnOp := op.Layer(inpOp, keyOp, op.Compose(bgOp, mask)).Offset(key.pos.Add(image.Pt(0, gridY)))
			content = op.Layer(content, btnOp)
		}
	}
	combined := image.Pt(max(readoutSz.X, k.size[k.page].X), gridY+k.size[k.page].Y)
	full := op.Layer(content, readoutOp.Offset(image.Pt((combined.X-readoutSz.X)/2, 0)))
	return full, combined
}
```

> No `layout` import (R0 C-2): the readout/grid math uses only `image`/`op`/`widget`/`math`/`strings`/`utf8`/`assets`. Keep imports minimal & gofmt-clean — the final file imports exactly: `image`, `math`, `strings`, `unicode/utf8`, `seedhammer.com/gui/assets`, `.../op`, `.../widget`.

- [ ] **Step 4: Run to verify it passes** — `/home/bcg/.local/go/bin/go test ./gui/ -run TestPassphraseMaskReveal`
Expected: PASS — masked `****`, no cleartext leak; revealed shows `ab1!`.

- [ ] **Step 5: Page-cycle render test** — append + run:
```go
func TestPassphrasePageCycleRender(t *testing.T) {
	// Render the symbols page (page 2) directly and assert its content: a digit key
	// '1' (symbols page) + the page-cycle cap "abc" are rendered. (Driving the
	// page-cycle key itself via D-pad is exercised by TestPassphraseActions'
	// commit(ppPageCycle); this test covers the per-page render.)
	ctx := NewContext(newPlatform())
	k := NewPassphraseKeyboard(ctx)
	k.page = 2
	frame, quit := runUI(ctx, func() {
		for !ctx.Done {
			for k.Update(ctx) {
			}
			o, _ := k.Layout(ctx, &descriptorTheme)
			ctx.Frame(o)
		}
	})
	defer quit()
	got, ok := frame()
	if !ok {
		t.Fatal("no frame")
	}
	if !uiContains(got, "1") || !uiContains(got, "abc") { // page 2 has digit '1' + the page-cycle cap "abc"
		t.Errorf("symbols page render: want '1' and the 'abc' page-cap; got %q", got)
	}
}
```
Run → PASS.

- [ ] **Step 6: Full gui suite + guards** — `/home/bcg/.local/go/bin/go test ./gui/...` (+ `./...`)
Expected: PASS — incl. `TestWordKeyboardScreen`, `TestInputSeedCodex32`, SLIP-39, codex32, `TestWordFlow*` (the new type touches none of them).

- [ ] **Step 7: vet + gofmt** — `/home/bcg/.local/go/bin/go vet ./gui/...` (clean modulo the pre-existing `gui/op/draw_test.go` go1.26 note); ensure no unused imports.

- [ ] **Step 8: Commit**
```bash
git add gui/passphrase_keyboard.go gui/passphrase_keyboard_test.go
git commit -S -s -m "gui(passphrase): Layout — masked/revealed readout + case-preserving grid render"
```

---

## Final: whole-diff adversarial execution review (mandatory)

Independent opus review over the whole diff vs `06b57f3`. Persist to `design/agent-reports/seedhammer-passphrase-keyboard-execution-review.md`; fold to clean.

Focus: no `ToUpper` anywhere in the widget (commit + render) and no `ToLower` on RuneEvent → case genuinely preserved; cross-page RuneEvent (`runes("Ab1!")→"Ab1!"`, no page switch); the function-row per-key-width layout doesn't overlap/clip and D-pad nav crosses into it; `Valid`/`adjust`/`adjustCol` panic-free (empty grids? backspace-only-valid?); masked readout shows `*`×len + reveal toggles; `Clear` re-masks; the new type touches NOTHING in the shared `Keyboard` / 3 flows (guard tests green); imports clean; signed+DCO. Note any coverage gaps (touch-tap can't be unit-tested — harness lacks coordinate-tap, consistent with Slice-1's deferral).

Then **superpowers:finishing-a-development-branch** — no upstream PR: merge `feat/passphrase-keyboard` into fork `main` (no-ff, signed), push to `bg002h`.

---

## Self-Review (author)

- **Spec coverage:** the standalone `PassphraseKeyboard`, 3 case-preserving pages, function row (page-cycle/space/reveal/backspace), masked-by-default readout + reveal, cross-page case-honoring RuneEvent, no-flow/no-`MnemonicSeed` scope — all realized across Tasks 1-3.
- **R0/R1 folds honored:** cross-page RuneEvent (no auto-switch); `ppKey`/`ppAction` model + per-action `Valid`/`commit`/`Layout`; function-row per-key sizing; `Clear` re-masks (`revealed=false`); `Layout` returns combined extent; `*` mask (not `•`); space as a `ppRune` so `runes(" ")` works.
- **Type consistency:** `ppAction`/`ppKey`/`PassphraseKeyboard`/`NewPassphraseKeyboard`/`Update`/`Layout`/`Clear`/`Valid`/`commit`/`adjust`/`adjustCol`/`moveCol`/`moveRow`/`keys`/`ppKeyExtent`/`ppRowWidth` consistent across tasks/tests; reused primitives match the extracted signatures (`widget.Labelw(buf,st,width,col,txt)`, `op.Input(...).Clip`, `mulAlpha`, `theme.inactiveMask`, `assets.KeyBackspace`, the `Up/Down/Left/Right/Center` consts).
- **Known soft spots flagged for the implementer:** the unused-import sequencing across Tasks 1-3 (build/commit as a unit if needed); the vestigial `allowBackspace` param in `adjust` (may simplify); the `layout` import may be unused (remove if so). These are noted inline so the R0 gate + implementer resolve them cleanly.
