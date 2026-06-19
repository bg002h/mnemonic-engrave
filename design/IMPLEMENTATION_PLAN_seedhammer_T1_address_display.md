# T1 — On-device address display (descriptor case) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Let the operator view a descriptor's receive/change addresses on-device (verify "does this card control these addresses?") before engraving — by wiring the in-tree, tested-but-unimported `address` package into the descriptor confirm flow.

**Architecture:** Add a new display-only screen `descriptorAddressFlow` (separate, non-benchmarked) and a `address.Supported`-gated Button2 affordance on `DescriptorScreen.Confirm` that opens it. `Supported` is computed once (hoisted out of the 0-alloc-gated frame loop); the conditional 3rd nav button is a fixed composite literal with `StyleNone` when unsupported (NOT an `append` chain). Pure wiring; no new crypto, no new dependency.

**Tech Stack:** Go (host `go test ./gui/... ./address/...`) + TinyGo (`pico-plus2`; existing CI job). Spec (GREEN R1): `design/SPEC_seedhammer_T1_address_display.md`. Base: fork `main` `384547d`.

**Gate status:** plan R0 GREEN (code ran: `TestAllocs` 0 allocs/op, all tests pass), implemented (`f6df823`,`979aadd`), then the **execution review caught a real paging defect** — fixed-count paging silently drops off-screen indices for wrapping long addresses (P2WSH multisig ~3 of 5 on 480×320). **AMENDED:** `descriptorAddressFlow` now uses **measure-and-advance** (Task 1, gap-free; spec §4.2) + a no-skipped-indices regression test (both single-sig and long-address fixtures). Re-gated as a combined spec+plan amendment R0 (`design/agent-reports/seedhammer-T1-address-paging-fix-review-R0.md`). The `Confirm` wiring (Task 2) + the R0 M-2 inertness test are unchanged and already implemented. Reviews: `…-plan-review-R0.md`.

---

## Source-of-truth facts (verified against `384547d`)

- `address` pkg (`address/address.go`): `Receive(desc *bip380.Descriptor, index uint32) (string,error)`, `Change(...)`, `Supported(desc) bool` (=`!errors.Is(Receive(desc,0),errUnsupported)`). Supports single-sig {P2PKH,P2WPKH,P2SH-P2WPKH,P2TR} + sortedmulti {P2SH,P2WSH,P2SH-P2WSH}. Default-children keys derive `<0;1>/*` → Receive=branch0, Change=branch1 (`:118-144`). **Imported nowhere outside the pkg** — T1 is the first importer; `gui→address` is acyclic; all btcsuite/decred deps already in `go.mod`.
- `DescriptorScreen{Descriptor *bip380.Descriptor}` (`gui/gui.go:2306-2308`); `Confirm` (`:2310-2355`) binds only Button1=Back + Button3=Confirm (**Button2 free**); nav is a fixed `[]NavButton{back,confirm}` literal (`:2347-2350`). `descriptorFlow` (`:2014`) loops `Confirm`→engrave; reached via `engraveObjectFlow case *bip380.Descriptor:` ← NFC scan (`scan.go:66` `nonstandard.OutputDescriptor`).
- **0-alloc gate:** `BenchmarkAllocs` (`gui_test.go:50-91`) drives `ds.Confirm` via `iter.Pull` (runs `Confirm` ONCE; each `next()` resumes to the next `ctx.Frame`); `TestAllocs` (`:93-98`) fails on `AllocsPerOp>0`. ⇒ a one-time pre-loop `address.Supported` call is amortized to ~0/op; the per-frame loop body must stay 0-alloc (three `.Clicked` + a **fixed** 3-element nav literal = 0-alloc, like today's 2-element). The benchmark's descriptor is a supported P2WSH sortedmulti, so `supported=true` there (the styled 3rd button path is exercised + must stay 0-alloc).
- `layoutNavigation(buf, th, dims, btns ...NavButton)` ranges over `btns` (never stored → non-escaping literal is stack-alloc); `StyleNone` renders the empty `op.Op{}` (`gui.go:1726-1728`). 3-button nav is already used by `confirmCodex32Flow` (back/recover/engrave). `showError(ctx, th, title, body string)` exists (`gui.go:2060` usage). Screen-render primitives: `layoutTitle`, `widget.Labelw`, `layout.Rectangle`/`CutTop`/`CutBottom`, `op.Layer`/`op.Color`, `leadingSize`, `ctx.Styles.body`. Test harness: `NewContext(newPlatform())`, `click(&ctx.Router, …)`, `runUI(ctx, fn) (frame, quit)`, `uiContains(content, sub)`, `&descriptorTheme`.

---

## File manifest
| File | Change |
|---|---|
| `gui/address_polish.go` | **new** — `descriptorAddressFlow(ctx, th, desc *bip380.Descriptor)` (display-only address-list screen). |
| `gui/address_polish_test.go` | **new** — render/toggle/paging/Back tests + the Confirm-affordance gating test. |
| `gui/gui.go` | **modify** — `import "seedhammer.com/address"`; add the hoisted-`Supported` Button2 affordance to `DescriptorScreen.Confirm`. |

Unchanged/reused: the `address` pkg, `nonstandard`, `bip380`, `DescriptorScreen.Draw`, the engrave path.

---

## Task 0: Worktree + baseline
- [ ] **Step 1:** `cd /scratch/code/shibboleth/seedhammer && git worktree add -b feat/address-display ../seedhammer-wt-t1-address 384547d && cd ../seedhammer-wt-t1-address`
- [ ] **Step 2:** `go test ./gui/... ./address/...` → PASS (baseline). Also `go test -run TestAllocs ./gui/` → PASS.

---

## Task 1: `descriptorAddressFlow` (the address-list screen)

**Files:** Create `gui/address_polish.go`, `gui/address_polish_test.go`.

- [ ] **Step 1: Write the failing test**

Create `gui/address_polish_test.go` (fixtures are the exact public vectors from `address/address_test.go:11,26,46`; the xpub is `xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan`):
```go
package gui

import (
	"testing"

	"seedhammer.com/address"
	"seedhammer.com/bip380"
	"seedhammer.com/nonstandard"
)

const tvXpub = "xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan"

// descWPKH: supported single-sig (default <0;1>/* children).
// descCustomChildren: wsh sortedmulti with explicit /1234/<5;6>/* so receive(branch5) ≠ change(branch6).
const (
	descWPKH           = "wpkh(" + tvXpub + ")"
	descCustomChildren = "wsh(sortedmulti(1," + tvXpub + "/1234/<5;6>/*))"
)

func loadTestDesc(t *testing.T, descStr string) *bip380.Descriptor {
	t.Helper()
	d, err := nonstandard.OutputDescriptor([]byte(descStr))
	if err != nil {
		t.Fatalf("OutputDescriptor(%q): %v", descStr, err)
	}
	return d
}

// frameUntil drives a runUI frame iterator up to n frames, returning true once the
// rendered content contains sub.
func frameUntil(frame func() (string, bool), sub string, n int) bool {
	for i := 0; i < n; i++ {
		c, ok := frame()
		if !ok {
			return false
		}
		if uiContains(c, sub) {
			return true
		}
	}
	return false
}

func TestDescriptorAddressFlowRendersReceive(t *testing.T) {
	d := loadTestDesc(t, descWPKH)
	want0, err := address.Receive(d, 0)
	if err != nil {
		t.Fatalf("Receive: %v", err)
	}
	ctx := NewContext(newPlatform())
	frame, quit := runUI(ctx, func() { descriptorAddressFlow(ctx, &descriptorTheme, d) })
	defer quit()
	if !frameUntil(frame, want0, 8) {
		t.Fatalf("address list did not render receive[0] %q", want0)
	}
}

func TestDescriptorAddressFlowToggleChange(t *testing.T) {
	d := loadTestDesc(t, descCustomChildren)
	wantChange0, err := address.Change(d, 0)
	if err != nil {
		t.Fatalf("Change: %v", err)
	}
	wantRecv0, _ := address.Receive(d, 0)
	if wantChange0 == wantRecv0 {
		t.Fatal("fixture must distinguish receive from change")
	}
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Button2) // toggle receive→change
	frame, quit := runUI(ctx, func() { descriptorAddressFlow(ctx, &descriptorTheme, d) })
	defer quit()
	if !frameUntil(frame, wantChange0, 8) {
		t.Fatalf("toggle did not render change[0] %q", wantChange0)
	}
}

func TestDescriptorAddressFlowBackExits(t *testing.T) {
	d := loadTestDesc(t, descWPKH)
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Button1) // Back → the flow should return
	frame, quit := runUI(ctx, func() { descriptorAddressFlow(ctx, &descriptorTheme, d) })
	defer quit()
	// The flow returns on Back; the iterator must end within a few frames.
	ended := false
	for i := 0; i < 6; i++ {
		if _, ok := frame(); !ok {
			ended = true
			break
		}
	}
	if !ended {
		t.Fatal("Back did not exit descriptorAddressFlow")
	}
}

// The load-bearing regression (R1-exec defect): measure-and-advance paging must
// never silently drop an index off-screen. Page forward across enough pages to
// cover indices 0..7 and assert EVERY address appears — for both a single-sig
// fixture (more fit/page) and a long-address P2WSH fixture (fewer fit/page).
func TestDescriptorAddressFlowNoSkippedIndices(t *testing.T) {
	for _, descStr := range []string{descWPKH, descCustomChildren} {
		d := loadTestDesc(t, descStr)
		seen := make(map[string]bool)
		for i := uint32(0); i < 8; i++ {
			a, err := address.Receive(d, i)
			if err != nil {
				t.Fatalf("%s: Receive(%d): %v", descStr, i, err)
			}
			seen[a] = false
		}
		ctx := NewContext(newPlatform())
		// Up to 8 page-forwards (worst case 1 address/page still reaches idx 7).
		click(&ctx.Router, Button3, Button3, Button3, Button3, Button3, Button3, Button3, Button3)
		frame, quit := runUI(ctx, func() { descriptorAddressFlow(ctx, &descriptorTheme, d) })
		for i := 0; i < 60; i++ {
			c, ok := frame()
			if !ok {
				break
			}
			for a := range seen {
				if uiContains(c, a) {
					seen[a] = true
				}
			}
		}
		quit()
		for a, ok := range seen {
			if !ok {
				t.Errorf("%s: address %q was never viewable — paging skipped it", descStr, a)
			}
		}
	}
}
```
(`uiContains`/`runUI`/`click`/`newPlatform`/`descriptorTheme` are the existing harness, used identically by `TestRecoverCodex32Mismatch`. Assertions compare against `address.Receive/Change` computed in-test — tied to the pkg, not hardcoded address literals. `TestDescriptorAddressFlowNoSkippedIndices` is the regression that locks the measure-and-advance fix: every index 0..7 must surface across pages on both the short-address and long-address fixtures.)

- [ ] **Step 2: Run — expect FAIL** (`descriptorAddressFlow` undefined): `go test ./gui/ -run TestDescriptorAddressFlow 2>&1 | tail`

- [ ] **Step 3: Implement `gui/address_polish.go`**
```go
package gui

import (
	"fmt"
	"image"

	"seedhammer.com/address"
	"seedhammer.com/bip380"
	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/layout"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
)

const addrMaxIndex = 49 // show indices 0..49; bounds the paging loop

// descriptorAddressFlow displays the descriptor's receive/change addresses for
// on-device verification. Display-only: no engrave, no NFC, no mutation. The
// caller opens this only when address.Supported(desc).
//
// Long bech32 addresses wrap across several rows, so this MEASURES each line and
// renders only the indices that FIT the content height, paging forward by the
// count actually shown — gap-free, so no index is ever dropped off-screen (spec
// §4.2). Recomputes only on entry / toggle / page (off any hot path; Measure
// emits no draw ops). dims is stable for the flow's lifetime.
func descriptorAddressFlow(ctx *Context, th *Colors, desc *bip380.Descriptor) {
	backBtn := &Clickable{Button: Button1}
	toggleBtn := &Clickable{Button: Button2}
	pageBtn := &Clickable{Button: Button3}

	dims := ctx.Platform.DisplaySize()
	lineWidth := dims.X - 2*8
	screen := layout.Rectangle{Max: dims}
	_, content := screen.CutTop(leadingSize)
	content, _ = content.CutBottom(leadingSize)
	contentTop := content.Min.Y + 8
	contentBottom := content.Max.Y

	start := uint32(0)
	change := false
	var lines []string
	shown := 0
	// recompute fills `lines` with the addresses that fit, starting at `start`:
	// the first index is included unconditionally (guarantees ≥1 shown + forward
	// progress); each subsequent index only while it fits the content height.
	recompute := func() bool {
		lines = lines[:0]
		shown = 0
		y := contentTop
		for i := uint32(0); start+i <= addrMaxIndex; i++ {
			idx := start + i
			var a string
			var err error
			if change {
				a, err = address.Change(desc, idx)
			} else {
				a, err = address.Receive(desc, idx)
			}
			if err != nil {
				showError(ctx, th, "Address", err.Error())
				return false
			}
			line := fmt.Sprintf("%d: %s", idx, a)
			sz := ctx.Styles.body.Measure(lineWidth, "%s", line)
			if i > 0 && y+sz.Y > contentBottom { // always include the first
				break
			}
			lines = append(lines, line)
			y += sz.Y + 6
			shown++
			if y > contentBottom { // a further line cannot fit
				break
			}
		}
		return true
	}
	if !recompute() {
		return
	}
	for !ctx.Done {
		if backBtn.Clicked(ctx) {
			return
		}
		if toggleBtn.Clicked(ctx) {
			change = !change
			start = 0
			if !recompute() {
				return
			}
		}
		if pageBtn.Clicked(ctx) {
			// Advance by the count shown (gap-free), bounded by addrMaxIndex.
			if start+uint32(shown) <= addrMaxIndex {
				start += uint32(shown)
				if !recompute() {
					return
				}
			}
		}
		title := "Receive addresses"
		if change {
			title = "Change addresses"
		}
		nav, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack},
			{Clickable: toggleBtn, Style: StyleSecondary, Icon: assets.IconEdit},
			{Clickable: pageBtn, Style: StylePrimary, Icon: assets.IconRight},
		}...)
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, title)
		body := make([]op.Op, 0, len(lines))
		y := contentTop
		for _, ln := range lines {
			lbl, sz := widget.Labelw(&ctx.B, ctx.Styles.body, lineWidth, th.Text, ln)
			body = append(body, lbl.Offset(image.Pt((dims.X-sz.X)/2, y)))
			y += sz.Y + 6
		}
		frameOps := append([]op.Op{nav, titleOp}, body...)
		frameOps = append(frameOps, op.Color(&ctx.B, th.Background))
		ctx.Frame(op.Layer(frameOps...))
	}
}
```
(`recompute`'s `Measure` height and the render's `Labelw` height agree — same style, width, and text — so the lines that recompute admits fit exactly as rendered. `showError` mid-`recompute` is the self-contained modal pattern from slip39/menu — returning after it is non-re-entrant; do not engrave or mutate. dims/content computed once: `DisplaySize` is fixed on SH.)

- [ ] **Step 4: Run — expect PASS**: `go test ./gui/ -run TestDescriptorAddressFlow -v`

- [ ] **Step 5: Commit**
```bash
git add gui/address_polish.go gui/address_polish_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "gui: descriptorAddressFlow — on-device address list (T1)

Display-only screen showing a descriptor's receive/change addresses
(Button2 toggle, Button3 page, Button1 back) via the in-tree address
pkg. Recomputes only on entry/toggle/page (off any hot path). No engrave,
no NFC, no mutation.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: the `Supported`-gated affordance on `DescriptorScreen.Confirm`

**Files:** Modify `gui/gui.go`; add a gating test to `gui/address_polish_test.go`.

- [ ] **Step 1: Write the failing affordance test**

Append to `gui/address_polish_test.go`:
```go
// On a supported descriptor, Button2 opens the address view (then Back returns to
// confirm, Back again exits). On an unsupported descriptor, Button2 is inert.
func TestDescriptorConfirmAddressAffordance(t *testing.T) {
	d := loadTestDesc(t, descWPKH) // supported
	if !address.Supported(d) {
		t.Fatal("fixture must be address-supported")
	}
	ds := &DescriptorScreen{Descriptor: d}
	want0, _ := address.Receive(d, 0)
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Button2) // open address view from the confirm screen
	frame, quit := runUI(ctx, func() { ds.Confirm(ctx, &descriptorTheme) })
	defer quit()
	var content string
	saw := false
	for i := 0; i < 10; i++ {
		c, ok := frame()
		if !ok {
			break
		}
		content = c
		if uiContains(content, want0) {
			saw = true
			break
		}
	}
	_ = content
	if !saw {
		t.Fatal("Button2 did not open the address view on a supported descriptor")
	}
}
```
(An unsupported-descriptor inertness assertion: construct a descriptor `address.Supported` returns false for — e.g. an unsupported script if a vector exists, else skip — and assert Button2 does not open the view / the confirm screen stays. If no easy unsupported fixture exists, assert instead that `address.Supported` gates the `StyleNone` branch via a focused render check; the implementer picks the tractable form. Non-negotiable: a supported descriptor's Button2 opens the view.)

- [ ] **Step 2: Run — expect FAIL** (affordance not wired): `go test ./gui/ -run TestDescriptorConfirmAddressAffordance 2>&1 | tail`

- [ ] **Step 3: Wire `DescriptorScreen.Confirm`** (`gui/gui.go`)

Add `"seedhammer.com/address"` to the gui.go import block. Replace the `Confirm` button setup + the nav-literal with:
```go
	backBtn := &Clickable{Button: Button1}
	addrBtn := &Clickable{Button: Button2}
	confirmBtn := &Clickable{Button: Button3}
	// Hoisted out of the frame loop: address.Supported→Receive(desc,0) runs
	// secp256k1 derivation (allocating); computing it per-frame would break the
	// TestAllocs 0-alloc gate. Once, here. (spec §2 inv. 2/6, §4.1.)
	supported := address.Supported(s.Descriptor)
	for !ctx.Done {
		if backBtn.Clicked(ctx) {
			break
		}
		// Drain Button2 every frame; act only when supported (queue-head idiom).
		if addrBtn.Clicked(ctx) && supported {
			descriptorAddressFlow(ctx, th, s.Descriptor)
			continue
		}
		if confirmBtn.Clicked(ctx) {
			// ... UNCHANGED existing engrave branch ...
		}

		dims := ctx.Platform.DisplaySize()
		// Fixed 3-element literal (non-escaping → 0-alloc). The address button is
		// StyleNone when unsupported (rendered empty) — NOT an append chain
		// (which would heap-alloc and break TestAllocs on this benchmarked screen).
		addrStyle := StyleSecondary
		if !supported {
			addrStyle = StyleNone
		}
		nav, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack},
			{Clickable: addrBtn, Style: addrStyle, Icon: assets.IconInfo},
			{Clickable: confirmBtn, Style: StylePrimary, Icon: assets.IconCheckmark},
		}...)
		content := s.Draw(ctx, th, dims)
		ctx.Frame(op.Layer(nav, content))
	}
	return Plate{}, false
```
(Keep `showErr` and the entire `confirmBtn.Clicked` engrave branch byte-identical to the original — only the button declarations, the hoisted `supported`, the Button2 handling, and the nav literal change.)

- [ ] **Step 4: Run — expect PASS** + no regressions:
```bash
go test ./gui/ -run 'TestDescriptorConfirmAddressAffordance|TestDescriptorAddressFlow' -v
go test ./gui/... ./address/...
```

- [ ] **Step 5: Verify the 0-alloc gate stays green** (the load-bearing check):

Run: `go test -run TestAllocs ./gui/ -v`
Expected: PASS (0 allocs). If it fails: the per-frame loop allocates — confirm `supported` is computed BEFORE the loop (not inside), and the nav is a fixed 3-element literal (not `append`). Fix the construction; never weaken the gate.

- [ ] **Step 6: Commit**
```bash
git add gui/gui.go gui/address_polish_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "gui: address-view affordance on the descriptor confirm screen (T1)

A Supported-gated Button2 on DescriptorScreen.Confirm opens
descriptorAddressFlow. Supported is hoisted out of the frame loop and the
3rd nav button is a fixed StyleNone-toggled literal, keeping the
TestAllocs 0-alloc gate green. Engrave path unchanged.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Full verification
- [ ] **Step 1:** `go test ./... && go vet ./gui/... ./address/... && gofmt -l gui/`
Expected: all PASS; vet clean (the pre-existing `gui/op/draw_test.go` go1.26 note is not ours); `gofmt -l` silent. **`go test -run TestAllocs ./gui/` PASS.**
- [ ] **Step 2 (CI):** the existing `tinygo-device-build` job compiles `gui` (now importing `address`) — confirm locally if TinyGo present, else rely on CI.

---

## Done criteria
- A descriptor's receive/change addresses viewable on-device before engrave; gated by `address.Supported`; display-only (no engrave/NFC/mutation); `TestAllocs` green; engrave path unchanged.
- After all tasks: mandatory whole-diff execution review → merge no-ff signed+DCO into fork `main` → push `bg002h` → clean up worktree.

## Self-review (vs spec)
- §1 scope (descriptor case, single-sig+sortedmulti, receive/change, paging) → `descriptorAddressFlow`. ✔
- §2 inv.1 display-only/public/deterministic → no engrave/NFC/mutation; recompute only on event. ✔ inv.2/6 Supported hoisted + 0-alloc gate → Task 2 + Step 5. ✔ inv.3 errors surfaced → `showError`+return. ✔ inv.4 no engrave regression → engrave branch byte-identical. ✔ inv.5 network honesty → `address` pkg handles it. ✔
- §4.1 fixed-literal + StyleNone (not append) → Task 2 Step 3. ✔ §4.2 Button1/2/3 + **measure-and-advance (gap-free, no-skip) + cap at index 49** → Task 1 (`descriptorAddressFlow` + `TestDescriptorAddressFlowNoSkippedIndices`). ✔
- §6 TDD incl. custom-children (receive≠change) + TestAllocs gate → Tasks 1/2/3. ✔
No placeholders: the fixture constants are the exact public vectors from `address/address_test.go` (the wpkh + `/1234/<5;6>/*` rows), and the affordance test asserts against `address.Receive` computed in-test. Types (`*bip380.Descriptor`, `address.Supported/Receive/Change`, `NavButton`/`StyleNone`) are consistent across tasks.
