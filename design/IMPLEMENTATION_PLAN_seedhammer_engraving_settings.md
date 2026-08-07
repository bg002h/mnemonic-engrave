# Engraving Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Put Speed and a new per-character Passes count behind a gear key on the
free-text keyboard, and give Title and Footer unconfirmed Clear buttons.

**Architecture:** `Passes` is a new field on `engrave.StringCmd` that repeats
`engraveSpline` before advancing `dot.X`, so a glyph is re-cut in place.
`ConstantStringer` — the type seed and passphrase plates use — has no such field,
so those plates are structurally unreachable. The UI is a gear key (a keyboard
*action*, not a nav button, because the nav budget is full) opening two levels of
the existing `ChoiceScreen`.

**Tech Stack:** Go 1.26 / TinyGo, `nix develop --command`, the fork's own
`gui`, `engrave` and `backup` packages.

## Global Constraints

- Every Go command runs as `cd /scratch/code/shibboleth/seedhammer && nix develop --command go ...`, with `export PATH=/nix/var/nix/profiles/default/bin:$PATH` first.
- **Never a bare `go test ./... -update`.** Re-record goldens scoped with `-run`, then confirm `git status` lists only the expected files.
- `layoutNavigation` indexes a **fixed `[3]int`** by `Button - Button1`. A fourth nav affordance **panics**. Text, Title and Footer each end at exactly 3/3.
- Keyboard function-row keys are **appended, never inserted**: `passphrase_keyboard_test.go:200` asserts the reveal key is at index 2.
- `ChoiceScreen` does not scroll and `op.Layer` draws content **over** its title past roughly seven entries. No screen in this plan exceeds six.
- Default `Passes` is **1** and default speed is the machine's own, so an untouched flow must plan byte-identical plates. **No golden may move in this whole plan.**
- Stage paths explicitly; no `git add -A`.

---

### Task 1: The gear icon asset

**Files:**
- Create: `gui/assets/icon-gear.alpha.png`
- Modify: `gui/assets/embed.go` (generated — do not hand-edit)
- Create: `gui/assets/icon-gear.bin` (generated)
- Test: `gui/assets/assets_test.go`

**Interfaces:**
- Produces: `assets.IconGear *alpha4.Image`, consumed by Task 5.

No gear icon exists. The generator globs `*.png` in `gui/assets`, so the file
name decides the symbol: `icon-gear.alpha.png` → `IconGear` + `icon-gear.bin`.
Match `icon-edit.alpha.png`'s 35×35 extent.

- [ ] **Step 1: Draw the icon**

```bash
cd /scratch/code/shibboleth/seedhammer/gui/assets
magick -size 35x35 xc:black \
  -fill white -draw "circle 17,17 17,5" \
  -fill black -draw "circle 17,17 17,11" \
  -fill white \
  -draw "rectangle 15,1 20,8"   -draw "rectangle 15,27 20,34" \
  -draw "rectangle 1,15 8,20"   -draw "rectangle 27,15 34,20" \
  -colorspace Gray icon-gear.alpha.png
identify icon-gear.alpha.png     # expect 35x35
```

- [ ] **Step 2: Generate the asset**

```bash
cd /scratch/code/shibboleth/seedhammer
nix develop --command go generate ./gui/assets/
grep -n "IconGear" gui/assets/embed.go     # expect the var and its //go:embed
```

- [ ] **Step 3: Write the test**

```go
// gui/assets/assets_test.go
package assets

import "testing"

// TestIconGearIsDrawn: an icon that generated to an empty rect draws nothing and
// leaves an invisible tap target, which a synthetic touch test still passes.
func TestIconGearIsDrawn(t *testing.T) {
	b := IconGear.Bounds()
	if b.Dx() < 8 || b.Dy() < 8 {
		t.Errorf("IconGear bounds are %dx%d; too small to find with a finger", b.Dx(), b.Dy())
	}
	if len(IconGearData) == 0 {
		t.Error("IconGear has no pixel data")
	}
}
```

- [ ] **Step 4: Run it**

Run: `nix develop --command go test ./gui/assets/ -run TestIconGearIsDrawn -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add gui/assets/icon-gear.alpha.png gui/assets/icon-gear.bin gui/assets/embed.go gui/assets/assets_test.go
git commit -m "gui/assets: a gear icon for the engraving settings key"
```

---

### Task 2: `Passes` on `engrave.StringCmd`

**Files:**
- Modify: `engrave/engrave.go:1490-1541` (`String`, `StringCmd`, `(*StringCmd).engrave`)
- Test: `engrave/engrave_test.go`

**Interfaces:**
- Produces: `StringCmd.Passes int` (0 or 1 → engrave once). Consumed by Task 3.

- [ ] **Step 1: Write the failing test**

```go
// engrave/engrave_test.go
// TestStringPassesRepeatsInPlace is the load-bearing test for the whole feature:
// a pass count wired to a label and never to the planner passes everything else.
func TestStringPassesRepeatsInPlace(t *testing.T) {
	// conf is the package-level StepperConfig at engrave_test.go:122.
	once := String(constant.Font, 2000, "B")
	twice := String(constant.Font, 2000, "B")
	twice.Passes = 2

	var a, b []bspline.Knot
	for k := range PlanEngraving(conf, once.Engrave) {
		a = append(a, k)
	}
	for k := range PlanEngraving(conf, twice.Engrave) {
		b = append(b, k)
	}
	if len(b) <= len(a) {
		t.Fatalf("two passes planned %d knots against one pass's %d", len(b), len(a))
	}
	// IN PLACE: the second pass must occupy the same coordinates as the first.
	// Advancing dot.X between passes would shift every control point.
	var minA, maxA, minB, maxB int
	minA, maxA = extentX(a)
	minB, maxB = extentX(b)
	if minA != minB || maxA != maxB {
		t.Errorf("two passes span x[%d,%d] against one pass's x[%d,%d]; the glyph moved between passes",
			minB, maxB, minA, maxA)
	}
}

func extentX(ks []bspline.Knot) (lo, hi int) {
	lo, hi = math.MaxInt, math.MinInt
	for _, k := range ks {
		lo, hi = min(lo, k.Ctrl.X), max(hi, k.Ctrl.X)
	}
	return
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `nix develop --command go test ./engrave/ -run TestStringPassesRepeatsInPlace`
Expected: FAIL — `twice.Passes undefined`.

- [ ] **Step 3: Add the field**

```go
// engrave/engrave.go
type StringCmd struct {
	LineHeight int
	// Passes is how many times each glyph is engraved IN PLACE before the pen
	// advances. 0 and 1 both mean once.
	//
	// In place, not a second pass over the whole string: re-cutting where the
	// tool already stands carries no repositioning error between passes, while a
	// whole-plate repeat accumulates one.
	//
	// ConstantStringer has no equivalent and must not gain one without the
	// constant-time proof -- see SPEC_seedhammer_engraving_settings.md section 3.
	Passes int

	face *vector.Face
	em   int
	txt  string
}
```

- [ ] **Step 4: Repeat the spline before advancing**

In `(*StringCmd).engrave`, replace the single `engraveSpline` call:

```go
		if yield != nil {
			passes := max(1, s.Passes)
			for range passes {
				// Decode afresh: UniformBSpline is an ITERATOR and is consumed
				// by engraveSpline, so re-using it would engrave nothing on the
				// second pass and the test would still see more knots.
				_, sp, _ := s.face.Decode(r)
				cont = cont && engraveSpline(yield, dot, s.em, mh, sp)
			}
		}
		dot.X += adv * s.em / mh
```

- [ ] **Step 5: Run it**

Run: `nix develop --command go test ./engrave/ -run TestStringPassesRepeatsInPlace -v`
Expected: PASS.

- [ ] **Step 6: Whole suite — no golden may move**

Run: `nix develop --command go test ./...`
Expected: 45 packages ok. If any golden moved, `max(1, s.Passes)` is wrong.

- [ ] **Step 7: Commit**

```bash
git add engrave/engrave.go engrave/engrave_test.go
git commit -m "engrave: StringCmd.Passes, repeating each glyph in place"
```

---

### Task 3: `Fitted.Passes` through the free-text builder

**Files:**
- Modify: `backup/freetext.go` (the `Fitted` struct, and the `engrave.String` call in `EngraveFitted`)
- Test: `backup/freetext_test.go`

**Interfaces:**
- Consumes: `engrave.StringCmd.Passes` from Task 2.
- Produces: `backup.Fitted.Passes int`. Consumed by Task 6.

- [ ] **Step 1: Write the failing test**

```go
// backup/freetext_test.go
// TestFittedPassesReachTheEngraving, and its second half: the type seed plates
// use has no Passes field at all, so they are structurally unreachable rather
// than merely un-plumbed.
func TestFittedPassesReachTheEngraving(t *testing.T) {
	// prodParams is the package-level Params at backup/sizes_test.go:21.
	// Build the Fitted through the real fit, as ftBounds (freetext_test.go:17)
	// does, so this exercises the shipped construction rather than a literal.
	P := prodParams
	size, lines, qrc, err := Fit(P, constant.Font, "BEEF", "", "", false)
	if err != nil {
		t.Fatal(err)
	}
	mk := func(passes int) int {
		f := Fitted{
			SizeMM: size, Lines: lines, QR: qrc,
			Faces:  []*vector.Face{constant.Font},
			Sizes:  []float32{size},
			Passes: passes,
		}
		n := 0
		for range engrave.PlanEngraving(P.StepperConfig, EngraveFitted(P, f)) {
			n++
		}
		return n
	}
	n, one := mk(3), mk(1)
	if n <= one {
		t.Errorf("three passes planned %d knots against one pass's %d", n, one)
	}
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `nix develop --command go test ./backup/ -run TestFittedPassesReachTheEngraving`
Expected: FAIL — `f.Passes undefined`.

- [ ] **Step 3: Add the field and wire it**

```go
// backup/freetext.go, in the Fitted struct
	// Passes is how many times each glyph is engraved in place. 0 and 1 mean
	// once. Only the FREE-TEXT path constructs a Fitted, which is what keeps a
	// pass count away from seed and passphrase plates.
	Passes int
```

and at the `engrave.String` call inside `EngraveFitted`:

```go
			cmd := engrave.String(fnt, fontSize, l)
			cmd.Passes = f.Passes
			cmd.Engrave(t.Yield)
```

- [ ] **Step 4: Run it**

Run: `nix develop --command go test ./backup/ -run TestFittedPassesReachTheEngraving -v`
Expected: PASS.

- [ ] **Step 5: Prove seed plates cannot reach it**

```go
// backup/freetext_test.go
// TestConstantStringerHasNoPasses is a COMPILE-TIME claim written as a runtime
// test: if ConstantStringer ever gains a Passes field, this stops compiling and
// somebody has to read SPEC_seedhammer_engraving_settings.md section 3.
func TestConstantStringerHasNoPasses(t *testing.T) {
	typ := reflect.TypeOf(engrave.ConstantStringer{})
	if _, ok := typ.FieldByName("Passes"); ok {
		t.Fatal("ConstantStringer gained a Passes field; constant-time engraving " +
			"needs a proof before a pass count may reach a seed plate")
	}
}
```

- [ ] **Step 6: Run both, then the whole suite**

Run: `nix develop --command go test ./backup/ && nix develop --command go test ./...`
Expected: all ok, no golden moved.

- [ ] **Step 7: Commit**

```bash
git add backup/freetext.go backup/freetext_test.go
git commit -m "backup: carry Passes on Fitted, the free-text-only plate"
```

---

### Task 4: The two settings screens

**Files:**
- Modify: `gui/freetext_flow.go` (beside `ftSpeedOptions`)
- Test: `gui/freetext_settings_test.go` (create)

**Interfaces:**
- Consumes: `ftSpeedOptions`, `ftSpeedChoiceFlow`, `ftPlanIsProof` (existing).
- Produces: `ftPassRungs []int`, `ftPassOptions(proofLoaded bool, cur int) ([]string, []int)`, `ftSettingsFlow(ctx *Context, th *Colors, params engrave.Params, proofLoaded bool, speed *float32, passes *int)`.

- [ ] **Step 1: Write the failing tests**

```go
// gui/freetext_settings_test.go
package gui

import "testing"

func TestPassOptionsAreTheAgreedRungs(t *testing.T) {
	want := []int{1, 2, 3, 4, 5, 8}
	labels, got := ftPassOptions(true, 0)
	if len(got) != len(want) {
		t.Fatalf("the Passes screen offers %d entries, want %d", len(got), len(want))
	}
	for i := range want {
		if got[i] != want[i] {
			t.Errorf("entry %d offers %d passes, want %d", i, got[i], want[i])
		}
	}
	if got[0] != 1 {
		t.Errorf("index 0 offers %d passes; index 0 IS the default and must be 1", got[0])
	}
	if len(labels) > 7 {
		t.Errorf("%d entries will silently overdraw ChoiceScreen's title", len(labels))
	}
}

func TestPassesLockedWithoutAProof(t *testing.T) {
	_, got := ftPassOptions(false, 0)
	if len(got) != 1 {
		t.Fatalf("the Passes screen offers %d entries with no proof loaded, want 1", len(got))
	}
	if got[0] != 0 {
		t.Errorf("taking the only entry set passes to %d, want it left alone", got[0])
	}
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `nix develop --command go test ./gui/ -run 'TestPassOptions|TestPassesLocked'`
Expected: FAIL — `ftPassOptions` undefined.

- [ ] **Step 3: Implement the options and the two flows**

```go
// gui/freetext_flow.go

// ftPassRungs is how many times each glyph may be engraved IN PLACE.
//
// Time is LINEAR in passes: a full proof plate at 4mm/s goes from about 15
// minutes at 1 to about two hours at 8, so the ceiling is a practical one
// rather than a safety one. Six entries, inside ChoiceScreen's budget.
var ftPassRungs = []int{1, 2, 3, 4, 5, 8}

func ftPassOptions(proofLoaded bool, cur int) ([]string, []int) {
	if !proofLoaded {
		return []string{"1 (default)"}, []int{cur}
	}
	labels := make([]string, 0, len(ftPassRungs))
	out := make([]int, 0, len(ftPassRungs))
	for _, n := range ftPassRungs {
		l := fmt.Sprintf("%d", n)
		if n == 1 {
			l += " (default)"
		}
		labels, out = append(labels, l), append(out, n)
	}
	return labels, out
}

func ftPassChoiceFlow(ctx *Context, th *Colors, proofLoaded bool, prior int) (int, bool) {
	labels, passes := ftPassOptions(proofLoaded, prior)
	cs := &ChoiceScreen{Title: "Passes", Lead: ftPassLead, Choices: labels}
	if len(passes) == 1 {
		cs.Lead = ftPassLeadFixed
	}
	want := prior
	if want <= 0 {
		want = 1
	}
	if i := slices.Index(passes, want); i > 0 {
		cs.choice = i
	}
	hookPPWidget("passes", cs)
	sel, ok := cs.Choose(ctx, th)
	if !ok {
		return prior, false
	}
	return passes[sel], true
}

// ftSettingsFlow is the gear's first level: pick a parameter, then its value.
//
// TWO LEVELS rather than one flat list, because ChoiceScreen does not scroll and
// op.Layer draws content over its own title past roughly seven entries -- so a
// flat list cannot hold this family once acceleration and jerk join it.
func ftSettingsFlow(ctx *Context, th *Colors, params engrave.Params, proofLoaded bool, speed *float32, passes *int) {
	for !ctx.Done {
		cs := &ChoiceScreen{
			Title: "Engraving",
			Lead:  ftSettingsLead,
			Choices: []string{
				fmt.Sprintf("Speed: %s", ftSpeedLabel(params, *speed)),
				fmt.Sprintf("Passes: %d", max(1, *passes)),
			},
		}
		hookPPWidget("settings", cs)
		sel, ok := cs.Choose(ctx, th)
		if !ok {
			return // Back leaves settings and returns to the keyboard.
		}
		switch sel {
		case 0:
			if v, ok := ftSpeedChoiceFlow(ctx, th, params, proofLoaded, *speed); ok {
				*speed = v
			}
		case 1:
			if v, ok := ftPassChoiceFlow(ctx, th, proofLoaded, *passes); ok {
				*passes = v
			}
		}
	}
}

// ftSpeedLabel is the settings row's value text: the chosen feed, or the
// machine's own when untouched.
func ftSpeedLabel(params engrave.Params, mmPerSec float32) string {
	if mmPerSec <= 0 {
		return fmt.Sprintf("%.1fmm/s", ftDefaultSpeedMM(params))
	}
	return fmt.Sprintf("%.1fmm/s", mmPerSec)
}
```

Add beside the existing leads:

```go
	ftSettingsLead  = "Engraving parameters for this plate. They are not saved."
	ftPassLead      = "How many times each character is cut, without moving. " +
		"More passes cut deeper and take proportionally longer."
	ftPassLeadFixed = "Passes are adjustable on test patterns only."
```

- [ ] **Step 4: Run the tests**

Run: `nix develop --command go test ./gui/ -run 'TestPassOptions|TestPassesLocked' -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add gui/freetext_flow.go gui/freetext_settings_test.go
git commit -m "gui: the engraving settings screens, Speed and Passes"
```

---

### Task 5: The gear key

**Files:**
- Modify: `gui/passphrase_keyboard.go` (the `ppAction` enum, `newPPKeyboard`, the key renderer)
- Modify: `gui/freetext_flow.go` (`ftTextEntryFlow` handles the action)
- Test: `gui/freetext_settings_test.go`

**Interfaces:**
- Consumes: `assets.IconGear` (Task 1), `ftSettingsFlow` (Task 4).
- Produces: `ppSettings ppAction`; `newPPKeyboard(ctx, newline, settings bool)`.

`NewTextKeyboard` passes `settings: true`; `NewPassphraseKeyboard` passes false,
so the gear can never appear while a passphrase is being typed.

- [ ] **Step 1: Write the failing tests**

```go
// gui/freetext_settings_test.go
func TestGearIsOnTheTextKeyboardOnly(t *testing.T) {
	h, _ := startFT(t)
	ftPastQR(h, false)
	if !ftHasKey(h, ppSettings) {
		t.Error("the text keyboard has no gear key")
	}
}

func TestGearIsNotOnThePassphraseKeyboard(t *testing.T) {
	h := newPPHarness(t)
	h.start(func() { engravePassphraseFlow(h.ctx, &descriptorTheme) })
	h.mustReach("Passphrase")
	if ftHasKey(h, ppSettings) {
		t.Error("the passphrase keyboard offers engraving settings")
	}
}

// ftHasKey reports whether the ACTIVE page's grid carries an action.
func ftHasKey(h *ppHarness, a ppAction) bool {
	kbd, ok := h.widget("kbd").(*PassphraseKeyboard)
	if !ok {
		return false
	}
	// keys() returns [][]ppKey -- rows of keys, not a flat slice
	// (passphrase_keyboard.go:246).
	for _, row := range kbd.keys() {
		for _, k := range row {
			if k.action == a {
				return true
			}
		}
	}
	return false
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `nix develop --command go test ./gui/ -run TestGearIs`
Expected: FAIL — `ppSettings` undefined.

- [ ] **Step 3: Add the action and the key**

```go
// gui/passphrase_keyboard.go — extend the enum. APPEND, never insert:
// passphrase_keyboard_test.go:200 asserts the reveal key is at index 2.
	ppSettings // open the engraving settings screen
```

In `newPPKeyboard(ctx *Context, newline, settings bool)`, after the `newline`
append:

```go
		if settings {
			// APPENDED for the same reason newline is: the reveal key's index is
			// asserted, and that assertion is worth more than the key order.
			fr = append(fr, ppKey{action: ppSettings})
		}
```

Render it with `assets.IconGear` in the two places `ppBackspace` renders
`assets.KeyBackspace` — the sizing switch at `passphrase_keyboard.go:180` and
the draw switch at `:453` — adding a `case ppSettings:` to each that mirrors the
backspace branch with the gear image.

`ppSettings` must NOT mutate `Fragment`. Add a latch beside the existing
`revealed` state, set when the key is pressed and consumed by the caller:

```go
// gui/passphrase_keyboard.go
type PassphraseKeyboard struct {
	...
	settingsReq bool // set by ppSettings, cleared by Settings()
}

// Settings reports and CLEARS a pending gear press. A latch rather than a
// callback, because Update already returns to a caller that owns the screen
// stack -- opening a screen from inside the keyboard's own update would nest
// frame loops.
func (k *PassphraseKeyboard) Settings() bool {
	req := k.settingsReq
	k.settingsReq = false
	return req
}
```

and in the action switch beside `case ppBackspace:`:

```go
		case ppSettings:
			k.settingsReq = true
```

- [ ] **Step 4: Handle it in the flow**

In `ftTextEntryFlow`'s update loop:

```go
		if kbd.Settings() {
			ftSettingsFlow(ctx, th, params, *proofLoaded, size, passes)
			continue
		}
```

- [ ] **Step 5: Run the tests, then the whole suite**

Run: `nix develop --command go test ./gui/ -run TestGearIs -v && nix develop --command go test ./...`
Expected: PASS, 45 packages ok.

- [ ] **Step 6: Commit**

```bash
git add gui/passphrase_keyboard.go gui/freetext_flow.go gui/freetext_settings_test.go
git commit -m "gui: a gear key on the text keyboard, opening engraving settings"
```

---

### Task 6: Remove the Speed step; carry passes to the plate

**Files:**
- Modify: `gui/freetext_flow.go` (`ftStep` enum, `engraveTextFlow`)
- Modify: `gui/freetext_flow_test.go` (delete `ftPastSpeed`, remove its call sites)
- Test: `gui/freetext_settings_test.go`

**Interfaces:**
- Consumes: `backup.Fitted.Passes` (Task 3), `ftSettingsFlow` (Task 4).

- [ ] **Step 1: Write the failing test**

```go
// gui/freetext_settings_test.go
// The gear's Passes must reach the PLATE, not just the screen.
func TestFlowCarriesPassesToTheEngraver(t *testing.T) {
	var got Plate
	var seen bool
	freetextEngraveHook = func(p Plate) { got, seen = p, true }
	t.Cleanup(func() { freetextEngraveHook = nil })

	h, _ := startFT(t)
	ftPastQR(h, false)
	ftTypeTrigger(h, ftProofTriggerConst)
	ftOK(h)
	h.tapWidget("proofYes")
	h.mustReach("lines")

	// Tap the gear: it is a KEY in the grid, so it is tapped through the
	// keyboard's own key bounds, not as a nav button.
	ftTapKey(h, ppSettings)
	ftChoose(h, "settings", 1)        // Passes
	ftChoose(h, "passes", 1)          // 2 passes
	h.tapNav(Button1)                 // leave settings
	h.mustReach("lines")
	ftOK(h)
	h.mustReach("Title")
	ftOK(h)
	h.mustReach("Footer")
	ftOK(h)
	h.mustReach("Confirm")
	ftOK(h)
	h.step()

	if !seen {
		t.Fatal("the flow never handed a plate to the engraver")
	}
	// The same composition at one pass, built directly, is the baseline.
	P := h.ctx.Platform.EngraverParams()
	one, err := ftBuildPlate(P, &ftPlanConst, ftProofTextFor(t, ftProofTriggerConst), "", "", false, 0, 1)
	if err != nil {
		t.Fatal(err)
	}
	if got.Duration <= one.Duration {
		t.Errorf("two passes planned %d ticks against one pass's %d", got.Duration, one.Duration)
	}
}

// ftTapKey taps the ACTIVE page's key carrying the given action. h.point fails
// if it is undrawn, off-panel or covered -- which is what a gear appended past
// the grid's right edge would be.
func ftTapKey(h *ppHarness, a ppAction) {
	h.t.Helper()
	kbd, ok := h.widget("kbd").(*PassphraseKeyboard)
	if !ok {
		h.t.Fatal(`widget "kbd" is not a *PassphraseKeyboard`)
	}
	for i := range kbd.keys() {
		for j := range kbd.keys()[i] {
			if k := &kbd.keys()[i][j]; k.action == a {
				h.tapAt(h.point(&k.clk, "keyboard key"))
				h.next("after tapping the %v key", a)
				return
			}
		}
	}
	h.t.Fatalf("no key with action %v on the active page", a)
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `nix develop --command go test ./gui/ -run TestFlowCarriesPasses`
Expected: FAIL — no gear route yet reaches the plate.

- [ ] **Step 3: Delete the step and thread passes**

Remove `ftStepSpeed` from the `iota` block and delete its `case` from
`engraveTextFlow`. Add `var passes int` beside `var speed float32`, pass
`&passes` into `ftTextEntryFlow`, and at `ftStepEngrave`:

```go
			plate, err := ftBuildPlate(ftParamsAtSpeed(params, speed), plan, text, title, footer, useQR, size, passes)
```

Thread `passes` through `ftBuildPlate` onto `Fitted.Passes`, and through
`ftEvaluate` so the confirm screen's duration estimate reflects it.

- [ ] **Step 4: Fix the E2E drivers**

Delete `ftPastSpeed` from `gui/freetext_flow_test.go` and remove every call.
The flow no longer stops between Text and Title.

```bash
grep -rn "ftPastSpeed" gui/*_test.go     # expect no matches when done
```

- [ ] **Step 5: Run the whole suite**

Run: `nix develop --command go test ./...`
Expected: 45 packages ok. **No golden may move** — the defaults are unchanged.

- [ ] **Step 6: Commit**

```bash
git add gui/freetext_flow.go gui/freetext_flow_test.go gui/freetext_settings_test.go
git commit -m "gui: Speed moves behind the gear, and passes reach the plate"
```

---

### Task 7: Clear on Title and Footer, unconfirmed

**Files:**
- Modify: `gui/freetext_flow.go` (`ftLineEntryFlow`)
- Test: `gui/freetext_clear_test.go`

**Interfaces:**
- Consumes: `assets.IconDiscard` (existing).

**The asymmetry with the Text field is intentional, in the operator's own
words: a title or footer is at most ONE LINE, so not much can be lost by
accident.** The confirmation tracks the cost of the error, not the identity of
the button. Do not "fix" it.

- [ ] **Step 1: Write the failing test**

```go
// gui/freetext_clear_test.go
func TestTitleAndFooterClearWithoutAPrompt(t *testing.T) {
	for _, step := range []string{"Title", "Footer"} {
		t.Run(step, func(t *testing.T) {
			h, _ := startFT(t)
			ftPastQR(h, false)
			ftSetText(h, "body")
			ftOK(h)
			h.mustReach("Title")
			if step == "Footer" {
				ftOK(h)
				h.mustReach("Footer")
			}
			h.typeString("ABC")
			h.tapWidget("clear")
			// No prompt: one line cannot lose much, so a confirmation costs
			// more than the mistake.
			h.mustReach(step)
			if got := ftKbd(h).Fragment; got != "" {
				t.Errorf("%s still holds %q after Clear", step, got)
			}
		})
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `nix develop --command go test ./gui/ -run TestTitleAndFooterClear`
Expected: FAIL — no `clear` widget on those screens.

- [ ] **Step 3: Implement**

In `ftLineEntryFlow`, mirror the Text screen's wiring but without the prompt:

```go
	clearBtn := &Clickable{Button: Button2}
	hookPPWidget("clear", clearBtn)
	...
		if clearBtn.Clicked(ctx) {
			// No prompt. A title or footer is at most one line, so little can be
			// lost by accident and retyping costs seconds; the Text field
			// confirms because it is uncapped and is the only copy.
			kbd.Fragment = ""
			continue
		}
	...
		navs := []NavButton{{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack}}
		if kbd.Fragment != "" {
			navs = append(navs, NavButton{Clickable: clearBtn, Style: StyleSecondary, Icon: assets.IconDiscard})
		}
		navs = append(navs, NavButton{Clickable: okBtn, Style: StylePrimary, Icon: assets.IconCheckmark})
		nav, _ := layoutNavigation(&ctx.B, th, dims, navs...)
```

- [ ] **Step 4: Run the tests, then the whole suite**

Run: `nix develop --command go test ./gui/ -run TestTitleAndFooterClear -v && nix develop --command go test ./...`
Expected: PASS, 45 packages ok.

- [ ] **Step 5: Commit**

```bash
git add gui/freetext_flow.go gui/freetext_clear_test.go
git commit -m "gui: Clear on the Title and Footer fields, without a prompt"
```

---

### Task 8: The confirm screen names a non-default pass count

**Files:**
- Modify: `gui/freetext_flow.go` (`ftSpeedNote` → `ftSettingsNote`, `ftConfirmSummary` call sites)
- Test: `gui/freetext_settings_test.go`

A pass count is invisible on the finished plate, so the operator must not be
able to approve one without seeing it — the same rule Speed already follows.

- [ ] **Step 1: Write the failing test**

```go
func TestConfirmNamesANonDefaultPassCount(t *testing.T) {
	P := newPlatform().EngraverParams()
	if got := ftSettingsNote(P, 0, 0); got != "" {
		t.Errorf("untouched settings noted %q, want nothing", got)
	}
	if got := ftSettingsNote(P, 0, 1); got != "" {
		t.Errorf("one pass noted %q, want nothing", got)
	}
	if got := ftSettingsNote(P, 0, 3); got == "" {
		t.Error("three passes produced no note; the operator would approve it unseen")
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `nix develop --command go test ./gui/ -run TestConfirmNamesANonDefault`
Expected: FAIL — `ftSettingsNote` undefined.

- [ ] **Step 3: Implement**

```go
// ftSettingsNote is the confirm screen's suffix for anything not at its
// default: nothing on the finished steel records the feed or the pass count.
func ftSettingsNote(params engrave.Params, mmPerSec float32, passes int) string {
	note := ftSpeedNote(params, mmPerSec)
	if passes > 1 {
		note += fmt.Sprintf("  passes: %d", passes)
	}
	return note
}
```

Replace the `ftSpeedNote(params, speed)` argument at the `ftConfirmFlow` call
site with `ftSettingsNote(params, speed, passes)`.

- [ ] **Step 4: Run the tests, then the whole suite**

Run: `nix develop --command go test ./gui/ -run TestConfirmNamesANonDefault -v && nix develop --command go test ./...`
Expected: PASS, 45 packages ok.

- [ ] **Step 5: Mutation checks — the suite must fail on each**

| mutation | test that must fail |
| --- | --- |
| default `Passes` to 2 in `String()` | Task 2 step 6 (a golden moves) |
| advance `dot.X` between passes | `TestStringPassesRepeatsInPlace` |
| give `ConstantStringer` a `Passes` field | `TestConstantStringerHasNoPasses` |
| drop the `proofLoaded` gate in `ftPassOptions` | `TestPassesLockedWithoutAProof` |
| pass `settings: true` to the passphrase keyboard | `TestGearIsNotOnThePassphraseKeyboard` |
| drop `passes` from the `ftBuildPlate` call | `TestFlowCarriesPassesToTheEngraver` |

Apply each with a **verified** pattern (assert the replacement matched — a
substitution that silently fails reads as a surviving mutation), run
`go test ./gui/`, restore from a **file copy**, never `git checkout`.

- [ ] **Step 6: Commit**

```bash
git add gui/freetext_flow.go gui/freetext_settings_test.go
git commit -m "gui: the confirm screen names a non-default pass count"
```

---

## Ship

- [ ] `nix develop --command go test ./...` — 45 packages green
- [ ] `nix develop --command gofmt -l gui/ engrave/ backup/` — no output
- [ ] `git status --short` — only the expected files, plus the untracked `test-e4-a125-j1300.signed.uf2`
- [ ] `~/bin/sh/sh2-flash --build-only`, then flash with the device in BOOTSEL
- [ ] Judge the boot on **machine power** — a laptop port gives a dark screen on firmware the bootrom accepted
