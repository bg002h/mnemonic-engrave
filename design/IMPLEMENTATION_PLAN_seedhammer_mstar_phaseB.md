# SeedHammer m*1 Correction — Phase B (GUI: typed md/mk entry + "Fix?" gate) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the dormant Phase A decoder into the on-device hand-typed flow: make all three m\*1 strings (`ms1`/`md1`/`mk1`) typeable + validated + engravable through one HRP-dispatched entry, and add an on-demand suggest→confirm "Fix?" gate that corrects a mistyped string only after the user confirms the per-position diff.

**Architecture:** Rework `inputCodex32Flow` from codex32/ms-only to HRP-aware: per-frame validation dispatched by parsed HRP (`New`/`ValidMD`/`ValidMK`), HRP-aware length windows + status + feedback (suppressing the codex32-share-schema `ParsePrefix` errors for md/mk), and a return type of `any` (`codex32.String` for ms, `mdmkText` for md/mk) that the existing `engraveObjectFlow` already routes. Add a Button3 "Fix?" affordance (shown when complete-but-invalid-in-window) that runs `codex32.Correct` and gates acceptance behind a new diff-confirm screen whose **universal anchor is the per-position diff** (the `id·thr·share` header line is ms-only).

**Tech Stack:** Go (host `go test ./gui/... ./codex32/...`) + TinyGo (`pico-plus2`; the Phase A CI job already covers `codex32`). Spec: `design/SPEC_seedhammer_mstar_correction.md` §4 (GREEN R1). Base: fork `main` `3342165` (Phase A merged).

---

## Source-of-truth facts (verified against fork main `3342165`)

**Phase A decoder (dormant, ready to wire):** `codex32.Correct(frag string) (CorrectionResult, bool)`; `type CorrectionResult struct { Corrected string; Edits []Edit }`; `type Edit struct { Pos int; Was, Now byte }` (`Pos` is a **rune index** into the full string, == byte index for ASCII bech32). Returns `(_, false)` for uncorrectable / >4-subs / re-verify-fail; never auto-applies. (`codex32/correct.go`.)

**Current `inputCodex32Flow`** (`gui/gui.go:721`): `func inputCodex32Flow(ctx *Context, th *Colors, title string) (codex32.String, bool)`. Per frame: `share, nerr := codex32.New(kbd.Fragment)`; `parsed, perr := codex32.ParsePrefix(kbd.Fragment)`; `valid := nerr == nil`. Back (Button1) → break; OK (Button3, `IconCheckmark`) when valid → `return share, true`. Readout: `codex32StatusLine(len)`, then `codex32Feedback(frag,perr,nerr)` else `codex32FieldLine(parsed)`. **Two callers:** menu `case 2` (`gui.go:2038`, returns into the menu's `(any,bool)`) and `recoverCodex32Flow` (`gui/codex32_polish.go:171`).

**Menu** (`gui.go:2018` `newInputFlow` → `(any,bool)`): `Choices: []string{"12 WORDS","24 WORDS","CODEX32","SLIP-39","SEED XOR"}`; `case 2:` calls `inputCodex32Flow(ctx, th, "Input Codex32 Share")`, `if ok { return s, true }`.

**Engrave dispatch** (`gui.go:1855` `engraveObjectFlow`): already routes `case codex32.String:` → `engraveCodex32` and `case mdmkText:` → `mdmkFlow`. `type mdmkText string` (`gui/scan.go:78`). `mdmkFlow` → `validateMdmk` → `ChoiceScreen` → engrave. **No change needed here.**

**codex32 package helpers** (`codex32/polish.go`): `ParsePrefix(frag) (Fields, error)` (`Fields.HRP` set after the `1`; returns `errInvalidThreshold` etc. for md/mk data — the codex32 share schema); `Describe(err) string`; `ConsistentShares`; length consts `ShortCodeMinLength`=48/`ShortCodeMaxLength`=93/`LongCodeMinLength`=125/`LongCodeMaxLength`=127. `splitHRP`, the md/mk brackets (`mdmkShortSyms`=13, `mkRegularMinLen`=14/`mkRegularMaxLen`=93/`mkLongMinLen`=96/`mkLongMaxLen`=108) and `ValidMD`/`ValidMK` live in the same package (`codex32/{codex32,mdmk}.go`).

**gui helpers** (`gui/codex32_polish.go`): `codex32StatusLine(n int)` (ms total windows), `codex32FieldLine(f Fields)` (ms field line), `codex32Feedback(frag, perr, nerr)` (ms: perr eager, nerr in-window), `confirmCodex32Flow`→`codex32ConfirmAction` (ms secret-review, Button2-drain), `showCodex32Error`, `recoverCodex32Flow`, `engraveCodex32`, `newCodex32Keyboard` (keypad permits `m d k s 1` + digits; b/i/o dimmed). A generic `showError(ctx, th, title, body)` exists (used at `gui.go:2060`).

**Test harness** (`gui/*_test.go`): `ctx := NewContext(newPlatform())`; `click(&ctx.Router, Down, Down, Button3)` injects buttons; `runes(&ctx.Router, s)` types on the keypad (**stores UPPERCASE**); `runUI(ctx, func(){...}) (frame, quit)` for frame-driven direct-call tests; `&descriptorTheme` is a usable `*Colors`. Buttons: `Button1/Button2/Button3`, `Center`, `Down`. Icons: `assets.{IconBack,IconCheckmark,IconEdit,IconHammer,IconRight,...}`.

**§4.1(b) HRP length windows (data-part vs total):** ms total 48..93 / 125..127; md data ≥13 (total ≥16, no upper); mk data 14..93 (total 17..96) / 96..108 (total 99..111); mk data 94..95 reserved-invalid.

---

## File manifest

| File | Change |
|---|---|
| `codex32/mstar.go` | **new** — `MStarInWindow(frag string) bool` (HRP-aware length-window predicate; encapsulates the md/mk brackets). |
| `codex32/mstar_test.go` | **new** — window-boundary table tests for ms/md/mk. |
| `gui/codex32_polish.go` | **modify** — add `validateMStar`, `mstarStatusLine`, `mstarFeedback`, `confirmCorrectionFlow`; `codex32StatusLine`/`codex32FieldLine`/`codex32Feedback` unchanged (delegated to). |
| `gui/gui.go` | **modify** — `inputCodex32Flow` HRP-aware rework → returns `any`, with the Button3 OK/"Fix?" dual affordance; menu `case 2` relabel + return the `any`. |
| `gui/codex32_polish.go` (recover) | **modify** — `recoverCodex32Flow` type-asserts `codex32.String` from the now-`any` entry. |
| `gui/*_test.go` | **modify/new** — md/mk entry returns `mdmkText`; ms still `codex32.String`; the Fix→confirm→accept path; reject; uncorrectable; Button2-drain no-hang; recovery rejects md/mk; update `TestInputSeedCodex32`/`codex32_input_test.go` for the `any` return. |

Unchanged/reused: `engraveObjectFlow`, `mdmkFlow`/`validateMdmk`, `engraveCodex32`, `confirmCodex32Flow`, `codex32.Correct`/`New`/`ValidMD`/`ValidMK`, the keypad.

---

## Task 0: Worktree + clean baseline

**Files:** none.

- [ ] **Step 1: Create the worktree off Phase A's merge commit**

```bash
cd /scratch/code/shibboleth/seedhammer
git worktree add -b feat/mstar-typed-entry ../seedhammer-wt-mstar-b 3342165
cd ../seedhammer-wt-mstar-b
```
Expected: worktree on `feat/mstar-typed-entry` at `3342165`.

- [ ] **Step 2: Verify clean baseline**

Run: `go test ./gui/... ./codex32/...`
Expected: PASS.

---

## Task 1: `codex32.MStarInWindow` (HRP-aware length-window predicate)

**Files:** Create `codex32/mstar.go`, `codex32/mstar_test.go`.

- [ ] **Step 1: Write the failing test**

Create `codex32/mstar_test.go`:
```go
package codex32

import (
	"strings"
	"testing"
)

func TestMStarInWindow(t *testing.T) {
	pad := func(hrp string, dataLen int) string {
		return hrp + "1" + strings.Repeat("q", dataLen)
	}
	// pad("ms",45) = "ms"+"1"+45×"q" = total length 48 (the prefix "xx1" is 3
	// chars, so total = dataLen + 3; ms windows are keyed on TOTAL length, md/mk
	// on the data-part length).
	cases := []struct {
		name string
		frag string
		want bool
	}{
		// ms uses TOTAL length 48..93 / 125..127.
		{"ms below short", pad("ms", 44), false}, // total 47 < 48
		{"ms short min", pad("ms", 45), true},    // total 48
		{"ms short max", pad("ms", 90), true},    // total 93
		{"ms dead zone", pad("ms", 91), false},   // total 94
		{"ms long lo", pad("ms", 122), true},     // total 125
		{"ms long hi", pad("ms", 124), true},     // total 127
		{"ms too long", pad("ms", 125), false},   // total 128
		// md: data ≥13, no upper bound.
		{"md below", pad("md", 12), false},
		{"md min", pad("md", 13), true},
		{"md big", pad("md", 200), true},
		// mk: data 14..93 / 96..108; 94..95 reserved.
		{"mk below reg", pad("mk", 13), false},
		{"mk reg lo", pad("mk", 14), true},
		{"mk reg hi", pad("mk", 93), true},
		{"mk reserved 94", pad("mk", 94), false},
		{"mk reserved 95", pad("mk", 95), false},
		{"mk long lo", pad("mk", 96), true},
		{"mk long hi", pad("mk", 108), true},
		{"mk too long", pad("mk", 109), false},
		// unknown / no separator.
		{"no separator", "msqqqq", false}, // no '1' ⇒ HRP "" ⇒ false
		{"foreign hrp", pad("xx", 60), false},
	}
	for _, c := range cases {
		if got := MStarInWindow(c.frag); got != c.want {
			t.Errorf("%s: MStarInWindow(%q[len %d]) = %v, want %v", c.name, c.frag, len(c.frag), got, c.want)
		}
	}
}
```

- [ ] **Step 2: Run it — expect FAIL (undefined `MStarInWindow`)**

Run: `go test ./codex32/ -run TestMStarInWindow`
Expected: FAIL — `MStarInWindow` undefined.

- [ ] **Step 3: Implement**

Create `codex32/mstar.go`:
```go
package codex32

import "strings"

// MStarInWindow reports whether an in-progress m*1 fragment is at a valid LENGTH
// for its HRP's code(s) — the per-HRP window that arms "bad checksum" feedback
// and the on-demand correction ("Fix?") affordance. ms uses total-string windows
// (48..93 / 125..127); md/mk use data-part windows (the chars after "xx1"): md
// data ≥13 (no upper bound), mk data 14..93 (regular) or 96..108 (long), with
// 94..95 reserved-invalid. Advisory; New/ValidMD/ValidMK remain the validity
// authority. (Phase B; SPEC §4.1(b).)
func MStarInWindow(frag string) bool {
	hrp, data := splitHRP(frag)
	switch {
	case strings.EqualFold(hrp, "ms"):
		n := len(frag)
		return (n >= shortCodeMinLength && n <= shortCodeMaxLength) ||
			(n >= longCodeMinLength && n <= longCodeMaxLength)
	case strings.EqualFold(hrp, "md"):
		return len(data) >= mdmkShortSyms
	case strings.EqualFold(hrp, "mk"):
		return (len(data) >= mkRegularMinLen && len(data) <= mkRegularMaxLen) ||
			(len(data) >= mkLongMinLen && len(data) <= mkLongMaxLen)
	}
	return false
}
```

- [ ] **Step 4: Run — expect PASS**

Run: `go test ./codex32/ -run TestMStarInWindow -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add codex32/mstar.go codex32/mstar_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "codex32: MStarInWindow — HRP-aware length-window predicate (Phase B)

Encapsulates the per-HRP valid-length windows (ms total 48..93/125..127;
md data ≥13; mk data 14..93/96..108, 94..95 reserved) so the GUI readout
+ the on-demand 'Fix?' affordance arm correctly for all three m*1.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2 (B1): HRP-aware typed entry — md/mk typeable + validated + engravable

**Files:** Modify `gui/codex32_polish.go`, `gui/gui.go`, and the gui tests.

- [ ] **Step 1: Write failing tests for md/mk entry + recovery rejection**

Append to `gui/codex32_input_test.go` (and update the existing `TestInputSeedCodex32` to assert via the `any` return — it already does `obj.(codex32.String)`, which still holds for ms, so it needs NO change):
```go
// Typing a valid md1 string returns it as mdmkText (routed to mdmkFlow), proving
// the HRP-dispatched entry handles md/mk, not just ms. (Phase B.)
func TestInputMStarMD1(t *testing.T) {
	const valid = "md1yqpqqxqq8xtwhw4xwn4qh"
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Down, Down, Button3) // menu -> M*1 STRING (index 2)
	runes(&ctx.Router, valid)
	click(&ctx.Router, Button3) // OK (valid)
	obj, ok := newInputFlow(ctx, &descriptorTheme)
	if !ok {
		t.Fatal("newInputFlow did not return a value")
	}
	got, isMd := obj.(mdmkText)
	if !isMd {
		t.Fatalf("returned %T, want mdmkText", obj)
	}
	if want := mdmkText(strings.ToUpper(valid)); got != want {
		t.Errorf("md1 entry = %q, want %q", got, want)
	}
}

// Typing a valid mk1 string returns mdmkText.
func TestInputMStarMK1(t *testing.T) {
	const valid = "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x"
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Down, Down, Button3)
	runes(&ctx.Router, valid)
	click(&ctx.Router, Button3)
	obj, ok := newInputFlow(ctx, &descriptorTheme)
	if !ok {
		t.Fatal("newInputFlow did not return a value")
	}
	if got, isMd := obj.(mdmkText); !isMd || got != mdmkText(strings.ToUpper(valid)) {
		t.Fatalf("mk1 entry = %v (%T), want mdmkText", obj, obj)
	}
}
```

Append to `gui/codex32_polish_test.go` a recovery-rejection test (typing an md1 during codex32 share recovery is rejected). **Event sequence matters (plan-R0 C-1):** the md1 is a *valid* string, so `inputCodex32Flow` OKs it on the first Button3 (returning `mdmkText`); `recoverCodex32Flow`'s type-assert then rejects it via `showCodex32Error` (a modal dismissed by **Button3**); a final Button1 backs out. Omitting the modal-dismiss Button3 hangs the test.
```go
// During codex32 share recovery, entering a non-codex32 (md/mk) string is
// rejected — recovery is ms-share-only. (Phase B caller-ripple guard.)
func TestRecoverRejectsNonCodex32(t *testing.T) {
	// A valid ms share with threshold ≥2 (mirrors TestRecoverCodex32's setup).
	shareA, err := codex32.New("MS12NAMEA320ZYXWVUTSRQPNMLKJHGFEDCAXRPP870HKKQRM")
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx := NewContext(newPlatform())
	// Enter a VALID md1 for "share 2": OK it (Button3 → mdmkText), the
	// type-assert rejects it with a modal (dismissed by Button3), then Back
	// (Button1) exits recovery.
	runes(&ctx.Router, "md1yqpqqxqq8xtwhw4xwn4qh")
	click(&ctx.Router, Button3, Button3, Button1) // OK md1 → dismiss modal → Back
	_, ok := recoverCodex32Flow(ctx, &descriptorTheme, shareA)
	if ok {
		t.Fatal("recovery must not accept a non-codex32 entry")
	}
}
```

- [ ] **Step 2: Run — expect FAIL (compile: `inputCodex32Flow` returns `codex32.String`, tests want `mdmkText`)**

Run: `go test ./gui/ -run 'TestInputMStar|TestRecoverRejects' 2>&1 | tail`
Expected: FAIL (the entry returns `codex32.String`, so md/mk never validate; or compile error once the rework lands).

- [ ] **Step 3: Add the HRP-aware helpers in `gui/codex32_polish.go`**

Append to `gui/codex32_polish.go`:
```go
// validateMStar runs the per-HRP completeness/validity check for an m*1 fragment
// and returns the typed value the engrave dispatch routes: a codex32.String for
// ms (via New), or an mdmkText for md/mk (via ValidMD/ValidMK). The third return
// is New's error for ms feedback (nil for md/mk). (Phase B; SPEC §4.1(a).)
func validateMStar(frag string, f codex32.Fields) (obj any, valid bool, msErr error) {
	switch {
	case strings.EqualFold(f.HRP, "ms"):
		s, err := codex32.New(frag)
		if err == nil {
			return s, true, nil
		}
		return nil, false, err
	case strings.EqualFold(f.HRP, "md"):
		if codex32.ValidMD(frag) {
			return mdmkText(frag), true, nil
		}
	case strings.EqualFold(f.HRP, "mk"):
		if codex32.ValidMK(frag) {
			return mdmkText(frag), true, nil
		}
	}
	return nil, false, nil
}

// mstarStatusLine is the HRP-aware length readout. ms reuses codex32StatusLine
// (total windows); md/mk report a data-part window state. (SPEC §4.1(b).)
func mstarStatusLine(frag string, f codex32.Fields) string {
	switch {
	case strings.EqualFold(f.HRP, "md"), strings.EqualFold(f.HRP, "mk"):
		if codex32.MStarInWindow(frag) {
			return fmt.Sprintf("%s · %d chars", strings.ToLower(f.HRP), len(frag))
		}
		return fmt.Sprintf("%d chars", len(frag))
	default: // ms or pre-separator
		return codex32StatusLine(len(frag))
	}
}

// mstarFeedback is the HRP-aware advisory error label. md/mk SUPPRESS the
// codex32-share-schema ParsePrefix errors (which fire spuriously on md/mk data,
// e.g. "bad threshold") and show only a generic "bad checksum" once in the
// per-HRP length window; ms delegates to codex32Feedback. (SPEC §4.1(c).)
func mstarFeedback(frag string, f codex32.Fields, perr, msErr error, valid bool) string {
	if valid || frag == "" {
		return ""
	}
	if strings.EqualFold(f.HRP, "md") || strings.EqualFold(f.HRP, "mk") {
		if codex32.MStarInWindow(frag) {
			return "bad checksum"
		}
		return ""
	}
	return codex32Feedback(frag, perr, msErr) // ms (or pre-separator) path unchanged
}
```

- [ ] **Step 4: Rework `inputCodex32Flow` (return `any`, HRP-aware readout) in `gui/gui.go`**

Replace the body of `inputCodex32Flow` (gui.go:721-802) with:
```go
func inputCodex32Flow(ctx *Context, th *Colors, title string) (any, bool) {
	kbd := newCodex32Keyboard(ctx)
	backBtn := &Clickable{Button: Button1}
	okBtn := &Clickable{Button: Button3} // OK when valid; "Fix?" when invalid-in-window
	for !ctx.Done {
		for kbd.Update(ctx) {
		}
		frag := kbd.Fragment
		parsed, perr := codex32.ParsePrefix(frag)
		obj, valid, msErr := validateMStar(frag, parsed)
		inWin := codex32.MStarInWindow(frag)

		if backBtn.Clicked(ctx) {
			break
		}
		// Always drain Button3 (avoid a queue-head block in direct-call tests);
		// act on it as OK when valid, or as "Fix?" when invalid-in-window.
		clicked3 := okBtn.Clicked(ctx)
		if valid && clicked3 {
			return obj, true
		}
		if !valid && inWin && clicked3 {
			res, ok := codex32.Correct(frag)
			if !ok {
				showError(ctx, th, "No correction", "No fix within 4 changes — check your typing")
			} else if confirmCorrectionFlow(ctx, th, res, strings.ToLower(parsed.HRP)) {
				kbd.Fragment = res.Corrected // accept; next frame re-validates → OK
			}
			continue
		}
		dims := ctx.Platform.DisplaySize()

		screen := layout.Rectangle{Max: dims}
		_, content := screen.CutTop(leadingSize)
		content, _ = content.CutBottom(8)

		kbdOp, kbdsz := kbd.Layout(ctx, th)
		kbdOp = kbdOp.Offset(content.S(kbdsz))

		word, frgSize := widget.Labelw(&ctx.B, ctx.Styles.word, dims.X-50, th.Background, frag)
		frgSize.X = max(frgSize.X, 100)
		r := image.Rectangle{Max: frgSize}
		r.Min.Y -= 3
		r.Max.Y += buttonPadY
		r.Min.X -= buttonPadX
		r.Max.X += buttonPadX
		top, _ := content.CutBottom(kbdsz.Y)
		wordOff := top.Center(frgSize)
		word = op.Layer(
			word,
			op.Compose(
				op.Color(&ctx.B, th.Text),
				op.RoundedRect2(&ctx.B, r, cornerRadius),
			),
		).Offset(wordOff)

		var infoOps []op.Op
		lineY := wordOff.Y + frgSize.Y + 8
		addLine := func(s string) {
			if s == "" {
				return
			}
			lbl, sz := widget.Label(&ctx.B, ctx.Styles.body, th.Text, s)
			y := lineY
			if lim := top.Max.Y - sz.Y; y > lim {
				y = lim
			}
			infoOps = append(infoOps, lbl.Offset(image.Pt((dims.X-sz.X)/2, y)))
			lineY = y + sz.Y + 4
		}
		addLine(mstarStatusLine(frag, parsed))
		if fb := mstarFeedback(frag, parsed, perr, msErr, valid); fb != "" {
			addLine(fb)
		} else if strings.EqualFold(parsed.HRP, "ms") || parsed.HRP == "" {
			addLine(codex32FieldLine(parsed)) // ms-only header line
		}

		navBtns := []NavButton{{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack}}
		switch {
		case valid:
			navBtns = append(navBtns, NavButton{Clickable: okBtn, Style: StylePrimary, Icon: assets.IconCheckmark})
		case inWin:
			navBtns = append(navBtns, NavButton{Clickable: okBtn, Style: StylePrimary, Icon: assets.IconEdit}) // "Fix?"
		}
		nav, _ := layoutNavigation(&ctx.B, th, dims, navBtns...)
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, title)

		frameOps := []op.Op{kbdOp, word}
		frameOps = append(frameOps, infoOps...)
		frameOps = append(frameOps, nav, titleOp, op.Color(&ctx.B, th.Background))
		ctx.Frame(op.Layer(frameOps...))
	}
	return nil, false
}
```
(`confirmCorrectionFlow` is added in Task 3; if you implement strictly in order, stub it as `func confirmCorrectionFlow(ctx *Context, th *Colors, res codex32.CorrectionResult, hrp string) bool { return false }` temporarily, then flesh it out in Task 3. Cleaner: implement Task 3's `confirmCorrectionFlow` body now since this references it.)

- [ ] **Step 5: Update the menu (`gui.go:2018` `newInputFlow`) — relabel + return the `any`**

In `newInputFlow`, change the choices and `case 2`:
```go
			Choices: []string{"12 WORDS", "24 WORDS", "M*1 STRING", "SLIP-39", "SEED XOR"},
```
```go
			case 2:
				obj, ok := inputCodex32Flow(ctx, th, "Input m*1 string")
				if ok {
					return obj, true
				}
```
(Index 2 is unchanged, so the `click(Down,Down,Button3)` navigation in tests still lands on it.)

- [ ] **Step 6: Update `recoverCodex32Flow` (`gui/codex32_polish.go:171`) for the `any` return**

Replace the `inputCodex32Flow` call site:
```go
		obj, ok := inputCodex32Flow(ctx, th, title)
		if !ok {
			return codex32.String{}, false // Back exits recovery
		}
		cand, isCodex32 := obj.(codex32.String)
		if !isCodex32 {
			showCodex32Error(ctx, th, "enter a codex32 share (ms1…)")
			continue
		}
```
(The rest of the loop — `ParsePrefix(cand.String())`, `ConsistentShares`, append — is unchanged.)

- [ ] **Step 7: Run the B1 tests + the existing suite**

Run: `go test ./gui/ -run 'TestInputMStar|TestRecover|TestInputSeedCodex32|TestConfirmCodex32|TestCodex32' && go test ./gui/... ./codex32/...`
Expected: PASS. (`TestInputSeedCodex32` still passes — ms returns `codex32.String`. Existing `codex32_polish_test.go` tests of `codex32StatusLine`/`codex32Feedback`/`codex32FieldLine` are unchanged.)

- [ ] **Step 8: Commit**

```bash
git add gui/gui.go gui/codex32_polish.go gui/codex32_input_test.go gui/codex32_polish_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "gui: HRP-dispatched m*1 typed entry (Phase B B1)

inputCodex32Flow validates per parsed HRP (New/ValidMD/ValidMK) and
returns any (codex32.String for ms, mdmkText for md/mk) — the existing
engraveObjectFlow already routes both. HRP-aware status + feedback
suppress the codex32 share-schema ParsePrefix errors for md/mk. Menu
relabel CODEX32 -> M*1 STRING; recoverCodex32Flow type-asserts the
codex32.String (recovery stays ms-share-only).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3 (B2): the on-demand "Fix?" suggest→confirm gate

**Files:** Modify `gui/codex32_polish.go` (add `confirmCorrectionFlow`), `gui/*_test.go`.

(The Button3 "Fix?" wiring + the `codex32.Correct` call were added to `inputCodex32Flow` in Task 2 Step 4; this task adds the confirm screen it calls and its tests.)

- [ ] **Step 1: Write failing tests for the confirm screen + the end-to-end Fix path**

Append to `gui/codex32_polish_test.go`:
```go
// The correction-confirm screen: Button3 accepts, Button1 rejects, and Button2
// is drained every frame (must not block Button3 — the multishare R0-C1 lesson).
func TestConfirmCorrectionFlow(t *testing.T) {
	res := codex32.CorrectionResult{
		Corrected: "MD1YQPQQXQQ8XTWHW4XWN4QH",
		Edits:     []codex32.Edit{{Pos: 5, Was: 'Z', Now: 'P'}},
	}
	// Accept (Button3).
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Button3)
	if !confirmCorrectionFlow(ctx, &descriptorTheme, res, "md") {
		t.Error("Button3 should accept the correction")
	}
	// Reject (Button1).
	ctx = NewContext(newPlatform())
	click(&ctx.Router, Button1)
	if confirmCorrectionFlow(ctx, &descriptorTheme, res, "md") {
		t.Error("Button1 should reject the correction")
	}
	// Button2 must not block Button3 (drain).
	ctx = NewContext(newPlatform())
	click(&ctx.Router, Button2, Button3)
	if !confirmCorrectionFlow(ctx, &descriptorTheme, res, "md") {
		t.Error("Button2 must be drained so Button3 still accepts")
	}
}
```

Append to `gui/codex32_input_test.go` the end-to-end Fix path:
```go
// A single-substitution-corrupted md1: the "Fix?" affordance (Button3 when
// invalid-in-window) corrects it; the confirm screen's Button3 accepts; the now
// valid string OKs through as mdmkText. (Phase B; orientation/diff end to end.)
func TestInputMStarFixMD1(t *testing.T) {
	const valid = "md1yqpqqxqq8xtwhw4xwn4qh"
	const corrupted = "md1yqzqqxqq8xtwhw4xwn4qh" // data index 2 ('p'->'z'); any single bech32 sub works
	if corrupted == valid {
		t.Fatal("test corruption is a no-op")
	}
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Down, Down, Button3) // menu -> M*1 STRING
	runes(&ctx.Router, corrupted)
	click(&ctx.Router, Button3) // Fix? (invalid-in-window)
	click(&ctx.Router, Button3) // accept the correction (confirmCorrectionFlow)
	click(&ctx.Router, Button3) // OK (now valid)
	obj, ok := newInputFlow(ctx, &descriptorTheme)
	if !ok {
		t.Fatal("newInputFlow did not return a value")
	}
	if got, isMd := obj.(mdmkText); !isMd || got != mdmkText(strings.ToUpper(valid)) {
		t.Fatalf("fixed md1 = %v (%T), want %q", obj, obj, strings.ToUpper(valid))
	}
}
```

- [ ] **Step 2: Run — expect FAIL (undefined `confirmCorrectionFlow`, or it's the temporary stub returning false)**

Run: `go test ./gui/ -run 'TestConfirmCorrectionFlow|TestInputMStarFix' 2>&1 | tail`
Expected: FAIL.

- [ ] **Step 3: Implement `confirmCorrectionFlow` in `gui/codex32_polish.go`**

Add (replacing the Task 2 stub if used):
```go
// confirmCorrectionFlow shows the proposed correction's per-position diff and
// asks the user to confirm it against their source card BEFORE the corrected
// string is accepted. The per-position diff is the UNIVERSAL anchor for all three
// m*1 (SPEC §2.3); for ms ONLY it also shows the decoded id·thr·share header line
// (the codex32 share schema does not exist for md/mk). Button1 rejects, Button3
// accepts; Button2 is drained every frame so it cannot block the queue head.
func confirmCorrectionFlow(ctx *Context, th *Colors, res codex32.CorrectionResult, hrp string) bool {
	lines := make([]string, 0, len(res.Edits)+2)
	for _, e := range res.Edits {
		// e.Pos is a full-string rune index (HRP + the '1' separator included);
		// +1 makes it 1-based for the human comparing against their source card.
		lines = append(lines, fmt.Sprintf("pos %d: %c → %c", e.Pos+1, rune(e.Was), rune(e.Now)))
	}
	if hrp == "ms" {
		if f, err := codex32.ParsePrefix(res.Corrected); err == nil {
			if fl := codex32FieldLine(f); fl != "" {
				lines = append(lines, fl)
			}
		}
	}
	lines = append(lines, "Compare each change to your source card.")

	backBtn := &Clickable{Button: Button1}
	drainBtn := &Clickable{Button: Button2}
	acceptBtn := &Clickable{Button: Button3, AltButton: Center}
	for !ctx.Done {
		if backBtn.Clicked(ctx) {
			return false
		}
		drainBtn.Clicked(ctx) // drain Button2 (R0-C1 idiom)
		if acceptBtn.Clicked(ctx) {
			return true
		}
		dims := ctx.Platform.DisplaySize()
		nav, _ := layoutNavigation(&ctx.B, th, dims,
			NavButton{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack},
			NavButton{Clickable: acceptBtn, Style: StylePrimary, Icon: assets.IconCheckmark},
		)
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, "Apply this correction?")

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
```

- [ ] **Step 4: Run — expect PASS**

Run: `go test ./gui/ -run 'TestConfirmCorrectionFlow|TestInputMStarFix' -v`
Expected: PASS. (If `TestInputMStarFixMD1` fails because the specific 1-char substitution isn't single-error-correctable — it always is for a single substitution — re-check the corrupted literal differs from `valid` by exactly one data-part char.)

- [ ] **Step 5: Add an ms-correction + uncorrectable + reject regression**

Append to `gui/codex32_input_test.go`:
```go
// An uncorrectable (>4-error) entry: pressing Fix? shows the "no fix" modal and
// returns to editing — it never fabricates a correction. **Event sequence
// (plan-R0 C-2):** Fix → dismiss modal → Back out of the ENTRY (returns
// (nil,false)) → Back out of the MENU (the menu loops on ok=false, so a second
// Button1 is required to make newInputFlow return). Omitting the second Back
// hangs on the re-rendered ChoiceScreen.
func TestInputMStarFixUncorrectable(t *testing.T) {
	// 5 substitutions in an md1 — beyond t=4; codex32.Correct returns (_,false).
	const corrupted = "md1zzzzzxqq8xtwhw4xwn4qh"
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Down, Down, Button3) // menu -> M*1 STRING -> entry
	runes(&ctx.Router, corrupted)
	// Fix? -> "no fix" modal -> dismiss -> Back(entry) -> Back(menu).
	click(&ctx.Router, Button3, Button3, Button1, Button1)
	obj, ok := newInputFlow(ctx, &descriptorTheme)
	if ok {
		t.Fatalf("uncorrectable entry must not yield a value, got %v (%T)", obj, obj)
	}
}
```
(The decoder's own `TestCorrectFiveErrorsNotSilentOriginal` proves the non-silent contract; this asserts the GUI surfaces an uncorrectable string as "no fix" and never auto-returns it. The chosen literal deterministically yields `(_,false)` — the modal path.)

- [ ] **Step 6: Run the full gui + codex32 suite + vet + gofmt**

Run:
```bash
go test ./gui/... ./codex32/...
go vet ./gui/... ./codex32/...
gofmt -l gui/ codex32/
```
Expected: all tests PASS; vet clean; `gofmt -l` prints nothing.

- [ ] **Step 7: Commit**

```bash
git add gui/codex32_polish.go gui/codex32_input_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "gui: on-demand m*1 'Fix?' suggest→confirm gate (Phase B B2)

When a complete-but-invalid m*1 string sits in its per-HRP length
window, Button3 becomes a 'Fix?' affordance that runs codex32.Correct
and gates acceptance behind confirmCorrectionFlow — a new screen whose
universal anchor is the per-position diff (pos N: x → y) for all three
m*1, plus the id·thr·share header line for ms only. Button2-drained;
Button1 rejects (keep editing), Button3 accepts (fall through to OK).
Uncorrectable → 'no fix' modal; never auto-applies.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Full verification

**Files:** none (verification only).

- [ ] **Step 1: Full host suite + vet + gofmt**

Run:
```bash
go test ./...
go vet ./...
gofmt -l gui/ codex32/
```
Expected: all PASS; vet clean (the pre-existing upstream `gui/op/draw_test.go` go1.26 `testing.ArtifactDir` vet note, if present, is not ours); `gofmt -l` silent.

- [ ] **Step 2: TinyGo device build (CI already covers it via Phase A's job; run locally if available)**

Run (if TinyGo present): `nix develop --command tinygo build -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller 2>&1 | tail`
Expected: compiles (the new gui code compiles for the device). If TinyGo/Nix is absent, note it — the Phase A `tinygo-device-build` CI job enforces it on push/PR.

---

## Done criteria (Phase B)

- All three m\*1 typeable through one entry; md/mk validate (`ValidMD`/`ValidMK`) and engrave via `mdmkFlow`; ms unchanged (`engraveCodex32`).
- The "Fix?" affordance corrects a mistyped m\*1 only after the user confirms the per-position diff; never auto-applies; uncorrectable → "no fix".
- `go test ./... ` + vet + gofmt clean; TinyGo build green (CI).
- After all tasks: a mandatory whole-diff adversarial execution review (persisted verbatim to `design/agent-reports/`), then merge no-ff signed+DCO into fork `main`, push `bg002h`, clean up the worktree.

---

## Self-review (against spec §4)

- **§4.1(a) HRP-dispatched per-frame validation** → `validateMStar` (New/ValidMD/ValidMK by parsed HRP); `inputCodex32Flow` returns `any`. ✔
- **§4.1(b) HRP-aware length windows** → `codex32.MStarInWindow` (Task 1) + `mstarStatusLine`; arms md1Regular(24)/mk1Long(111) correctly. ✔
- **§4.1(c) suppress codex32 ParsePrefix feedback for md/mk** → `mstarFeedback` (md/mk → generic "bad checksum" in-window, never `perr`; field line gated to ms). ✔
- **§4.1 caller ripple** → menu returns the `any`; `recoverCodex32Flow` type-asserts `codex32.String`. ✔
- **§4.2 on-demand Fix? + new confirm screen** → Button3 dual OK/"Fix?" (IconEdit) when invalid-in-window; `codex32.Correct`; `confirmCorrectionFlow` (NOT confirmCodex32Flow); accept → `kbd.Fragment = res.Corrected` → fall through to OK; `(_,false)` → "no fix" modal. ✔
- **§2.3 universal diff anchor / ms-only header** → `confirmCorrectionFlow` shows the per-position diff for all m\*1; `codex32FieldLine` only when `hrp=="ms"`. ✔
- **§2.1/§2.2** → no auto-apply (confirm gate); re-verify is inside `codex32.Correct` (Phase A) and the corrected string re-validates through `validateMStar` before OK. ✔
- **Button2-drain (R0-C1)** → `confirmCorrectionFlow` drains Button2 every frame; tested. ✔

No placeholders; types (`any`, `mdmkText`, `codex32.CorrectionResult`/`Edit`, `codex32.Fields`) consistent across tasks. The Task 2 Step 4 note offers a temporary `confirmCorrectionFlow` stub only if implementing strictly task-ordered; the recommended path is to implement Task 3's `confirmCorrectionFlow` body when the Task 2 rework first references it (one function, no real stub shipped).
