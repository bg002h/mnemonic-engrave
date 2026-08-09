# Encrypted Payload Delivery — Plan B Phase B2b (§10.2.4's residency-keyed idle wipe) — Implementation Plan

**Status:** DRAFT — R0 not yet run. **No code before 0C/0I.**

| round | verdict | report |
| --- | --- | --- |
| 0 | — | — |

**Descends from:** `SPEC_encrypted_payload_delivery.md` §10.2.4, **as amended
2026-08-09** (`0af8f97`). The amendment is a prerequisite, not a footnote: the
section could not be implemented faithfully as previously written, and this plan
is written against the amended text. Where this plan and §10 disagree, §10 wins.

**Designed against:** `design/CONSULT_b2b_idle_timer_design.md` (fable, one pass,
operator-approved 2026-08-09) and `design/RECON_b2b_idle_timer_surface.md`. Their
established facts are **not re-derived here**; the load-bearing ones are restated
in "Facts this plan is built on" and were independently verified by the
controller before authoring.

**Predecessor:** B2a-ii, merged at `a01b666`. **This phase makes the feature
operator-complete.** Only after it, plus Task 9's hardware pass and F-85, is a
release tag defensible.

---

## Facts this plan is built on — verified, do not re-derive

Four measured facts determine the whole design. Each was confirmed against the
source at `a01b666` by the controller, not taken from a report.

1. **`ctx.Done` is a SHUTDOWN, not an unwind — today.** `uiFlow` is
   `for !ctx.Done` (`gui/gui.go:1595`) and the only consumer,
   `cmd/controller/main.go:34`, is `for range gui.Run(p, ver) {}` followed by
   `return nil`. Setting `Done` today exits the GUI and the process. **Task 3 is
   what makes it non-terminal.**
2. **`ctx.Done` has NEVER been true in production.** Its only writer is
   `ctx.Done = ctx.Done || !yield(op)` (`gui/gui.go:2949`) inside `ctx.Frame`,
   set only when a consumer stops ranging — and neither `cmd/controller` nor
   `cmd/emu` ever stops. **B2b productionises a test-only path.** Operator
   decision 2026-08-09: accepted, with Task 8 exercising it on real hardware.
3. **Two screens call `ctx.Frame` once more AFTER `Done` goes true**, by
   fall-through: `SeedScreen.Confirm` (`gui/gui.go:2460`) and
   `EngraveScreen.Engrave` (`gui/gui.go:2758`). `ctx.Frame` calls `yield`, and
   **calling `yield` after it has returned false is a range-over-func panic** —
   a brick on a watchdog-less device. This is why the discard guard in Task 3 is
   *required*, not defensive.
4. **`Run` has zero test coverage**, and `testPlatform.AppendEvents` ignores its
   deadline, so neither `iter.Pull` nor `synctest` can drive its clock. **Task 1
   is a prerequisite, not a nicety** — every behaviour in Tasks 3–5 is otherwise
   unobservable, and this feature has already shipped two Criticals invisible to
   a green suite.

---

## Why the phase boundary is here, and the property that makes Tasks 1–3 cheap

**Tasks 1–3 change no operator-visible behaviour.** The harness is test-only; the
seam is installed but nothing reads it; the session loop and discard guard make
`ctx.Done` survivable but nothing sets it. A reviewer of those three never has to
reason about *when* a wipe fires — only about whether the machine still works
exactly as it does today.

**Task 4 is where the timer arms**, and it is the only task that can wipe.

| | B2b (this plan) | deferred |
| --- | --- | --- |
| Run-level test harness (fact 4) | ✅ Task 1 | — |
| the residency seam (`Context.wipe`, the bracket) | ✅ Task 2 | — |
| the unwind: session loop + discard guard | ✅ Task 3 | — |
| §10.2.4's timer, warning and wipe | ✅ Task 4 | — |
| **F-93** the screensaver parking a derivation | ✅ Task 5 | — |
| `RecordsResident` rename + **F-87**'s pins | ✅ Task 6 | — |
| §11.3 mutation rows + **F-96**'s runner | ✅ Task 7 | — |
| hardware (fact 2's first real firing) | ✅ Task 8 | — |
| **F-88, F-94** further seed copies | — | own cycle |
| **F-90** items 1 and 3 (`ms1` inventory, hook) | — | own cycle |
| **F-76**, F-80's residue | — | after B2b |

**F-90 item 2 is DISSOLVED, not deferred.** The timer keys on the session
bracket's lifetime, never on `SecretsResident()`, so the predicate's wrong
contract on the `ms1` arm cannot reach it. §10.2.4's amendment records this as
normative and forbids the predicate as the timer's key.

---

## Global Constraints

Carried forward unchanged from B2a-ii; the load-bearing ones:

- **All Go work runs under `nix develop --command …`.** `nix` is NOT on `PATH` —
  use `/nix/var/nix/profiles/default/bin/nix`.
- **Stage paths explicitly. Never `git add -A`.** One commit per task. Do not
  push, do not tag.
- **`gofmt -l` reports by PRINTING and exits 0 either way.** Test the output:
  `out=$(gofmt -l …); [ -z "$out" ]`.

### The green criterion, measured at `a01b666`

| command | expectation |
| --- | --- |
| `CGO_ENABLED=0 go test ./...` | **exit 1**, exactly TWO `[setup failed]`: `cmd/kdfbench`, `cmd/sealread`. A third is a regression. |
| `go vet ./seal/ ./bip39/` | clean |
| `go vet ./gui/` | **exit 1**, only `gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later` |
| `gofmt -l <touched>` | empty |
| TinyGo device build | baseline **1310184 flash / 60584 ram** — report the new numbers |
| `GOARCH=386 go test ./seal/ ./bip39/` | green |

### B2b-specific

- **No shared screen learns a new exit.** `ChoiceScreen.Choose` and
  `SeedScreen.Confirm` are used across the whole firmware; the design's central
  claim is that they need **no change**. If a task finds itself editing one, the
  design is wrong and the task should stop.
- **No `panic`/`recover`.** The fork has no `recover` in non-test code and this
  plan adds none.
- **No `Platform` change, no signature change.** `cmd/controller` and `cmd/emu`
  are untouched.

---

## Task 1 — the Run-level test harness (PREREQUISITE)

Fact 4: `Run` has no test, and `testPlatform.AppendEvents` ignores its deadline,
so nothing can drive `Run`'s clock. **Every assertion in Tasks 3–5 depends on
this.** It is first because writing the unwind against an untestable substrate is
how B2a-ii shipped two Criticals a green suite could not see.

Two pieces:

1. **A deadline-respecting test platform.** `testPlatform.AppendEvents` must
   honour the deadline it is passed so `synctest` can advance the bubble's clock
   and `Run`'s idle arithmetic becomes drivable.
2. **A `runFlow` seam** so a test can run `Run` with a *chosen* flow instead of
   `uiFlow`, and observe the frames it yields.

### 1a — first, move `Run`'s body into its own file

**`Run` yields NOTHING** — `func Run(pl Platform, version string) func(yield func() bool)`
(`gui/gui.go:2934`). The consumer sees ticks, not content, so **a Run-level test
cannot observe a single drawn pixel through the iterator.** The seam therefore
has to carry an observer as well as a flow.

And because every one of Tasks 3–4's edits is inside `Run`, they would all be
**fragments** — the exact category the build gate cannot type-check, in the
riskiest code this phase touches. So Task 1 moves the body:

- **Create `gui/run_flow.go`** holding `runWithFlow(pl, version, flow, onDraw)`
  — `Run`'s body verbatim, plus the two parameters.
- **`gui/gui.go`'s `Run` becomes one line:**
  `return runWithFlow(pl, version, uiFlow, nil)`.

`uiFlow` already has the shape `func(*Context, string)`, so it is passed as-is.
`onDraw` is `nil` in production and never called there.

**The file's FINAL form — after Tasks 3 and 4 — is given in full in Task 4, and
that is the copy the build gate type-checks.** Task 1's version is that file
minus the session loop, the discard guard, and the whole `a.wipe` block: a pure
move. Reviewers should read Task 4's block as the destination and Task 1 as
`git diff -M`.

### 1b — the harness

`gui/run_harness_test.go`, new file.

```go
package gui

import (
	"image"
	"testing"
	"time"

	"seedhammer.com/gui/op"
)

// The Run-level harness. Run has zero test coverage at a01b666 and
// testPlatform.AppendEvents ignores its deadline, so no test can drive Run's
// clock -- which is why §10.2.4's timer, its warning and the unwind are all
// unobservable without this. B2a-ii shipped two Criticals invisible to a green
// suite for want of exactly this kind of instrument.

// deadlinePlatform is a testPlatform whose AppendEvents actually BLOCKS until
// the deadline it is handed, so time.Sleep inside a synctest bubble advances
// Run's idle arithmetic. The embedded testPlatform supplies everything else.
type deadlinePlatform struct {
	*testPlatform
	// queued events are delivered on the next AppendEvents and refresh Run's
	// last-input clock exactly as a real touch does.
	queued []Event
}

func newDeadlinePlatform() *deadlinePlatform {
	p := newPlatform()
	// The 240x240 default is a fiction no shipped device has (gui/gui_test.go:390).
	p.display = sh2DisplaySize
	return &deadlinePlatform{testPlatform: p}
}

// AppendEvents honours the deadline. Returning the slice UNCHANGED when nothing
// is queued is the property Run's `len(evts) > 0` refresh depends on: a
// platform that appended a synthetic event every tick would refresh the idle
// clock forever and NO timer could ever fire -- the test would pass by never
// arming anything.
func (p *deadlinePlatform) AppendEvents(deadline time.Time, evts []Event) []Event {
	if len(p.queued) > 0 {
		evts = append(evts, p.queued...)
		p.queued = p.queued[:0]
		return evts
	}
	if d := time.Until(deadline); d > 0 {
		time.Sleep(d)
	}
	return evts
}

// tap queues one physical touch -- press AND release, the pair every existing
// touch test sends (gui/start_screen_touch_test.go:49).
func (p *deadlinePlatform) tap() {
	pos := image.Pt(sh2DisplaySize.X/2, sh2DisplaySize.Y/2)
	p.queued = append(p.queued,
		PointerEvent{Pressed: true, Entered: true, Pos: pos}.Event(),
		PointerEvent{Pressed: false, Entered: true, Pos: pos}.Event(),
	)
}

// runSession drives Run's real body with a chosen flow, returning the extracted
// text of everything DRAWN -- content frames and Run's own warning alike.
//
// Observation goes through onDraw, not through the iterator, because Run yields
// func() bool: no value reaches the consumer (gui.go:2934). A test that counted
// iterator ticks would be asserting on nothing at all.
func runSession(p *deadlinePlatform, flow func(ctx *Context, version string)) []string {
	var drawn []string
	r := image.Rectangle{Max: sh2DisplaySize}
	onDraw := func(o op.Op) {
		d := new(op.Drawer)
		drawn = append(drawn, d.ExtractText(r, o))
	}
	for range runWithFlow(p, "test", flow, onDraw) {
	}
	return drawn
}

// drawnContains reports whether any drawn frame carried str, with uiContains'
// whitespace-insensitive matching (gui/gui_test.go:516).
func drawnContains(drawn []string, str string) bool {
	for _, c := range drawn {
		if uiContains(c, str) {
			return true
		}
	}
	return false
}

// assertDrawn fails with the frames actually drawn, so a miss is diagnosable
// without a rerun.
func assertDrawn(t *testing.T, drawn []string, str string) {
	t.Helper()
	if !drawnContains(drawn, str) {
		t.Errorf("no drawn frame contains %q; got %d frames: %q", str, len(drawn), drawn)
	}
}
```

**Steps:**

- [ ] **1.1** Do the move (1a). `go test ./gui/` must be **unchanged** — a pure
      move that changes a test result is not a pure move.
- [ ] **1.2** Write the harness. Smoke test: a flow that returns immediately
      terminates `runSession`; a flow that loops `for !ctx.Done` drawing a label
      per tick produces frames whose text `assertDrawn` finds.
- [ ] **1.3** **Prove the deadline is honoured**, because that is the whole
      point: a test that sleeps past `idleTimeout` inside a `synctest` bubble
      must observe `Run`'s **saver** activate. If it does not, the platform is
      not driving the clock and every later task asserts on nothing.
- [ ] **1.4** `go test ./gui/`, `gofmt`, commit.

---

## Task 2 — the residency seam (installs; nothing reads it yet)

`gui/wipe_guard.go`, new file.

```go
package gui

// §10.2.4's residency seam.
//
// "Resident" is a LIFETIME, not a buffer scan (§10.2.4 as amended 2026-08-09).
// seal.RecordsResident() reads false from the instant a plate is built, while
// the flow still holds codex32.String, the parsed words, and the plate's SPLINE
// CLOSURE -- an iter.Seq over the plaintext, not a rendering (F-83 as
// corrected). The spec therefore FORBIDS that predicate as the timer's key, and
// this guard's lifetime is the key instead.
//
// The bracket is unlockSecretSession's own first and last act, so the window is
// exactly "secrets decrypted and being offered" to "the last secret plate has
// left the screen". The gaps at its edges are frame-free straight-line code.
type wipeGuard struct {
	// job is the engrave job currently cutting a secret plate, nil otherwise.
	// Registered by the two unlock engrave arms around their Engrave call.
	job *engraveJob
}

// armed reports whether §10.2.4's timer should be running.
//
// nil receiver -- no secret session open -- is the overwhelmingly common case
// and costs two nil checks per Run tick.
//
// A RUNNING job disarms it: §10.2.4 row 2, never wipe mid-plate with the needle
// down. engraveStopping counts as running because Stop() is synchronous in the
// flow but the worker is still moving. Note what is deliberately NOT here:
// screen visibility. Row 2 as amended keys on the JOB, so the hold-to-start and
// plate-done screens are ARMED -- they are walk-away states with secrets still
// held.
func (g *wipeGuard) armed() bool {
	if g == nil {
		return false
	}
	if j := g.job; j != nil {
		switch j.Status().State {
		case engraveRunning, engraveStopping:
			return false
		}
	}
	return true
}
```

**Fragments:**

Modify `gui/gui.go`: `Context` gains `wipe *wipeGuard` (unexported — every
reader and writer is in package `gui`).

Modify `gui/unlock_session.go`: `unlockSecretSession` installs and uninstalls the
guard as its first and last act.

```go
	g := &wipeGuard{}
	ctx.wipe = g
	defer func() { ctx.wipe = nil }()
```

Still in `gui/unlock_session.go`, both engrave arms register the job around
`Engrave`, so `armed()` can see a cut in progress:

```go
	scr := NewEngraveScreen(ctx, plate)
	if g := ctx.wipe; g != nil {
		g.job = scr.job
		defer func() { g.job = nil }()
	}
	scr.Engrave(ctx, &engraveTheme)
```

**Steps:**

- [ ] **2.1** Tests first, at flow level: the guard is installed for the whole
      secret session and nil before and after; `armed()` is false while a job
      runs and true on the plate-done screen. **Assert on `ctx.wipe`, not on a
      return value.**
- [ ] **2.2** Write the file and fragments. Nothing reads `armed()` yet, so the
      suite must be otherwise unchanged.
- [ ] **2.3** Mutation check: delete the `defer func() { ctx.wipe = nil }()` and
      confirm a test fails — **a bracket that fails to uninstall leaves the timer
      armed during the public plate list**, which is operator-hostile.
- [ ] **2.4** `go test ./gui/`, `gofmt`, commit.

---

## Task 3 — make `ctx.Done` survivable (the unwind; still nothing sets it)

**This is the task that changes what `ctx.Done` MEANS**, and per fact 2 it is the
first code to make that path real. Three changes, all in `gui/run_flow.go`; the
gated whole-file form is Task 4's block.

1. **The session loop.** The body of the returned closure becomes `for { … }`,
   with `ctx := NewContext(pl)`, the `it :=` closure and `var evts []Event`
   **inside** the loop; `a`, `d` and `stats` stay outside, since a wipe must not
   reallocate the mask or lose the frame-time baseline. After
   `for content := range it` ends: `if wiping { continue }` — building a **fresh
   `Context`** and re-entering the flow; otherwise `return`.

   A fresh `Context`, not a scrubbed one: `EventRouter.Reset`
   (`gui/event.go:281`) discards unfiltered events and truncates `r.filters`,
   but leaves `r.pointer` — whose `pressedTag`/`pressed` (`gui/event.go:13`)
   would carry a half-finished touch across the restart. A wipe is rare enough
   that the allocation is irrelevant, and a fresh `Context` needs no argument
   about which fields matter.

2. **The discard guard**, first statement in the range body, before any draw:

```go
	if wiping {
		continue
	}
```

   **Required, not defensive** (fact 3). On the ordinary walked-away path the
   unwind emits one extra frame; without the guard that iteration reaches
   `if ctx.Done || !yield() { return }` and executes the `return`, **converting
   the wipe into a full GUI exit** — the operator's machine stops having a UI as
   the direct result of the timer working. Note it also skips the *inner* loop,
   which is what keeps `ctx.Done` from being re-read as an exit.

3. **The wipe itself**, in the event loop — `wiping = true; ctx.Done = true;
   break`. **`break`, never the `return`**: the body iteration must complete
   normally so `yield` returns true and `ctx.Frame` returns to the parked flow,
   which then unwinds through its own `for !ctx.Done` and **runs every defer**.
   `Run` never calls a wipe function and never touches `p` — the unwind *is* the
   wipe, which is F-89 restated.

   The pre-existing `return` on `ctx.Done || !yield()` **stays exactly as it
   is**: a flow that finishes on its own, and a consumer that stops ranging, are
   still real exits. Only the wipe path is new.

**Steps:**

- [ ] **3.1** Tests first, using Task 1's harness: with a flow that parks in
      `ctx.Frame`, setting the wipe condition makes the flow **return**, its
      defers run, and `Run` **restarts** — assert the restart by extracted
      content, never by frame count. A frame-count assertion already produced a
      false PASS in this feature.
- [ ] **3.2** Write the three fragments.
- [ ] **3.3** **Mutation checks, and these are the ones that matter:**

      | mutant | must be killed by |
      | --- | --- |
      | `break` → `return` | the restart test — the GUI exits instead of restarting |
      | the discard guard deleted | a test driving a flow that `Frame`s after `Done` (fact 3's two screens) — without it the wipe becomes a GUI exit |
      | the session loop's `continue` removed | the restart test |
      | `wiping` never cleared | the second-session test |

- [ ] **3.4** `go test ./gui/`, TinyGo device build, `gofmt`, commit.

---

## Task 4 — §10.2.4's timer, warning and wipe (the only task that can wipe)

`gui/wipe_warning.go`, new file.

```go
package gui

import (
	"image"
	"time"

	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
)

// §10.2.4's 30-second warning, drawn by Run.
//
// wipeWarningDelay is the gap between the warning and the wipe. §10.2.4 row 1
// supplies the 3:00 as a VALUE via idleTimeout; this is the separate 30 s the
// same row requires and which no existing constant carries.
const wipeWarningDelay = 30 * time.Second

// wipeWarningOp draws the warning: one op out, no state touched, so a test can
// assert on its extracted text without driving Run's clock.
//
// Run draws this rather than the flow, and that is FORCED rather than
// preferred: at 3:00 of no input the flow is parked inside ctx.Frame and only
// Run has control. A flow-drawn warning would require every shared screen to
// learn a new signal, which the design's central constraint forbids, and the
// screensaver cannot carry it (saver draws no text). Replacing the screen
// entirely doubles as the privacy blanking a walked-away machine wants.
//
// It takes *Context rather than *op.Buffer because both the buffer (ctx.B) and
// the text styles (ctx.Styles, unexported) live there -- Colors carries only
// Background/Text/Primary (gui/theme.go:30), no style.
func wipeWarningOp(ctx *Context, th *Colors, dims image.Point, remaining time.Duration) op.Op {
	const margin = 8
	secs := int(remaining.Seconds() + 0.5)
	if secs < 0 {
		secs = 0
	}
	titleOp, titleRect := layoutTitle(ctx, dims.X, th.Text, "WIPING SECRET DATA")
	body, bodySz := widget.Labelwf(&ctx.B, ctx.Styles.body, dims.X-2*margin, th.Text,
		"This machine still holds decrypted seed material and has been idle.\n\n"+
			"It will be erased in %d seconds.\n\nTouch the screen to keep it.", secs)
	return op.Layer(
		body.Offset(image.Pt((dims.X-bodySz.X)/2, titleRect.Max.Y+margin)),
		titleOp,
		// Background LAST: op.Layer paints later ops BEHIND earlier ones
		// (gui.go:353, :591, :790).
		op.Color(&ctx.B, th.Background),
	)
}
```

### The screensaver cannot carry the warning — so `Run` needs a draw path

`a.idle.state.Draw(pl)` (`gui/gui.go:3008`) writes **straight to the platform**,
bypassing the op pipeline entirely. So "Run draws the warning" is not a matter of
swapping an op into the saver branch: `Run` must run the same
`Dirty` → `NextChunk` → `d.Draw` sequence the content path uses
(`gui/gui.go:2962` onward), from *inside* the event loop.

That block is therefore **extracted into a `draw(op.Op)` closure** and called from
both places. It is also the natural home for `onDraw` — which is why the harness
sees the warning at all.

### The final form of the moved body — and what the gate type-checks

Create `gui/run_flow.go` (Task 1 creates it as a pure move; this is its state
after Tasks 3 and 4, and the copy `plan-build-gate-go.sh` type-checks):

```go
package gui

import (
	"image"
	"time"

	"seedhammer.com/gui/op"
	"seedhammer.com/gui/saver"
)

// runWithFlow is Run's body, with the flow and a draw observer as parameters.
//
// The flow parameter exists because Run had no test at a01b666; onDraw exists
// because Run yields func() bool -- no content reaches the consumer, so without
// it a test can observe nothing Run draws, including §10.2.4's warning. Both
// are nil/uiFlow in production.
func runWithFlow(pl Platform, version string, flow func(ctx *Context, version string), onDraw func(op.Op)) func(yield func() bool) {
	return func(yield func() bool) {
		a := struct {
			mask *image.Alpha
			idle struct {
				start  time.Time
				active bool
				state  saver.State
			}
			wipe struct {
				// origin is the later of the last physical input and the last
				// transition to armed -- §10.2.4 row 2 as amended.
				origin time.Time
				armed  bool
			}
		}{}
		versionText := "Firmware: " + version + "\nHardware: " + pl.HardwareVersion()
		if !pl.Features().Has(FeatureSecureBoot) {
			versionText += " (UNLOCKED)"
		}
		stats := new(runtimeStats)
		d := new(op.Drawer)
		// The SESSION loop. A wipe unwinds the flow and re-enters it with a
		// fresh Context; everything above this line survives, because a wipe
		// must not reallocate the mask or restart the frame-time baseline.
		for {
			ctx := NewContext(pl)
			now := time.Now()
			a.idle.start = now
			a.wipe.origin = now
			a.wipe.armed = false
			wiping := false

			it := func(yield func(op.Op) bool) {
				ctx.FrameCallback = func(o op.Op) {
					ctx.Done = ctx.Done || !yield(o)
				}
				flow(ctx, versionText)
			}
			startTime := time.Now()
			var evts []Event

			// draw is the content path lifted out of the range body so the
			// warning can use it too: the screensaver writes straight to the
			// platform (saver.State.Draw) and cannot carry an op.
			draw := func(content op.Op) {
				d.Reset()
				dirty := image.Rectangle{Max: pl.DisplaySize()}
				if err := pl.Dirty(dirty); err != nil {
					panic(err)
				}
				for {
					fb, ok := pl.NextChunk()
					if !ok {
						break
					}
					fbdims := fb.Bounds().Size()
					npix := fbdims.X * fbdims.Y
					if a.mask == nil || len(a.mask.Pix) < npix {
						a.mask = image.NewAlpha(image.Rectangle{Max: fbdims})
					}
					a.mask.Rect = image.Rectangle{Max: fbdims}
					d.Draw(fb, a.mask, content)
				}
				if onDraw != nil {
					onDraw(content)
				}
			}

			for content := range it {
				// The DISCARD GUARD. Two screens call ctx.Frame once more after
				// Done goes true (gui.go:2460, :2758). Without this, that frame
				// reaches `if ctx.Done ... return` below and the wipe becomes a
				// full GUI exit -- the machine loses its UI because the timer
				// worked.
				if wiping {
					continue
				}
				layoutTime := time.Since(startTime)
				draw(content)
				drawTime := time.Since(startTime)
				if debug {
					stats.Dump(drawTime, layoutTime)
				}
				for {
					if ctx.Done || !yield() {
						return
					}
					wakeup := ctx.Wakeup
					evts = pl.AppendEvents(wakeup, evts[:0])
					now := time.Now()
					if len(evts) > 0 {
						a.idle.start = now
						a.wipe.origin = now
					}
					ctx.Reset()
					if !a.idle.active {
						ctx.Router.Events(d, evts...)
					}
					// §10.2.4: the timer keys on the SESSION BRACKET's
					// lifetime, never on seal.RecordsResident -- which reads
					// false while the flow still holds the words and the
					// plate's spline closure.
					armed := ctx.wipe.armed()
					if armed != a.wipe.armed {
						a.wipe.armed = armed
						if armed {
							// Row 2: restart the window when a cut ends.
							// Without this the window inherits a 21-minute
							// stale input clock and fires instantly.
							a.wipe.origin = now
						}
					}
					warnAt := a.wipe.origin.Add(idleTimeout)
					wipeAt := warnAt.Add(wipeWarningDelay)
					if armed && now.Sub(wipeAt) >= 0 {
						wiping = true
						ctx.Done = true
						break
					}
					if armed && now.Sub(warnAt) >= 0 {
						draw(wipeWarningOp(ctx, &descriptorTheme, pl.DisplaySize(), wipeAt.Sub(now)))
						ctx.WakeupAt(now.Add(time.Second))
						continue
					}
					idleWakeup := a.idle.start.Add(idleTimeout)
					idle := now.Sub(idleWakeup) >= 0
					if a.idle.active != idle {
						a.idle.active = idle
						if idle {
							a.idle.state = saver.State{}
						}
					}
					// While armed the warning owns the screen from 3:00, so the
					// saver must not paint over it. While NOT armed every line
					// here behaves exactly as it does today.
					if a.idle.active && !armed {
						a.idle.state.Draw(pl)
						// Throttle screen saver speed.
						const minFrameTime = 40 * time.Millisecond
						ctx.WakeupAt(now.Add(minFrameTime))
						continue
					}
					if armed {
						ctx.WakeupAt(warnAt)
					} else {
						ctx.WakeupAt(idleWakeup)
					}
					break
				}
				startTime = time.Now()
			}
			if !wiping {
				return
			}
		}
	}
}
```

**Fragment, `gui/gui.go`:** `Run`'s body becomes
`return runWithFlow(pl, version, uiFlow, nil)`.

Three properties worth naming, because a reviewer should confirm them rather
than re-derive them:

- **`a.wipe.origin` has two sources** — `len(evts) > 0` (the same true
  last-physical-input signal `a.idle.start` uses) and the `armed` false→true
  edge. Dropping the second is the "instant wipe the moment a plate finishes"
  defect.
- **A tap during the warning cannot activate what is underneath.** The router
  hit-tests against `d`, which the warning path has just redrawn, and the
  warning op carries no tags — so the touch refreshes `origin` and nothing else.
  That is the desired behaviour for a dismissal, not an accident.
- **When `armed` is false the event loop is byte-identical to today**, including
  the saver covering a running 21-minute cut.

**Steps:**

- [ ] **4.1** Tests first, on the harness: armed + no input → warning at 3:00,
      wipe at 3:30, flow unwound, UI restarted. A tap during the warning →
      window resets, no wipe. Not armed → **no warning ever**, saver unchanged.
- [ ] **4.2** Write the file and fragments.
- [ ] **4.3** **Mutation checks:**

      | mutant | must be killed by |
      | --- | --- |
      | `armed` hardcoded true | the not-armed test — a wipe on the public plate list |
      | `armed` hardcoded false | the wipe test |
      | the `armed` false→true origin reset removed | the post-cut test — instant wipe at cut end |
      | saver gate `&& !armed` removed | the warning test — the saver covers the warning |
      | `wipeWarningDelay` → 0 | the warning-visible test |

- [ ] **4.4** `go test ./gui/`, device build, `gofmt`, commit.

---

## Task 5 — F-93: the screensaver must not park a derivation

`Context` gains `KeepAwake()`, cleared by `Reset`; `unlockDerive` calls it each
slice. A derivation produces no events, so `Run`'s `len(evts) > 0` refresh never
fires and any derivation longer than `idleTimeout` trips the saver — measured,
**13.2% of §6.2's legal iteration range** (above 1,748,700 iterations), reachable
with a **conforming** blob.

- [ ] **5.1** Test on the harness: a derivation longer than `idleTimeout` does
      **not** trip the saver and completes. Mutant: remove the `KeepAwake` call
      → the saver activates and the derivation parks.
- [ ] **5.2** Implement, run, `gofmt`, commit.

---

## Task 6 — `RecordsResident` and F-87's pins

- **Rename `seal.SecretsResident` → `RecordsResident`.** Doc tightening only, no
  behaviour change. §10.2.4 as amended forbids it as the timer's key; the rename
  stops anyone building the wide reading on it again.
- **F-87:** nothing pins `unlockEngraveMnemonic`'s deferred `clear(m)` — deleting
  it leaves the package green, because no test drives the three early returns to
  an observable point. Drive each with `unlockMnemonicHook` set.

- [ ] **6.1** Tests, mutation-checked (delete the defer → the new tests fail).
- [ ] **6.2** Rename, update callers and docs, commit.

---

## Task 7 — the §11.3 rows, and F-96's runner

**Commit the mutation runner** as `scripts/mutation-run.py` in `mnemonic-engrave`,
with the row table as data and a printed statement of what it does **not** cover
— the shape `plan-build-gate-go.sh` uses. `CLAUDE.md`'s standing rule: *"when an
artifact will be folded repeatedly, commit the extractor as a script so the check
is a command, not a thing to remember."* B2a-ii ran ~50 mutants by hand for want
of it.

- [ ] **7.1** Write the runner; re-run every row this phase owns through it.
- [ ] **7.2** A surviving mutant is **blocking**. Record results in the commit.

---

## Task 8 — hardware (operator-run)

**This is the first time `ctx.Done` is ever true on the real machine** (fact 2),
and the operator accepted that on 2026-08-09 on condition the hardware pass
covers it.

- [ ] **8.1** Seal vector F, load, unlock, and **walk away**. Confirm the warning
      at 3:00, the wipe at 3:30, and that the machine **returns to the main menu
      and is still usable** — not a blank screen, not a reboot.
- [ ] **8.2** Repeat, touching during the warning: confirm the window resets and
      no wipe occurs.
- [ ] **8.3** Start a secret plate and walk away **mid-cut**: confirm **no wipe**
      while the job runs, and that the 3:00 window restarts from the cut's end.
- [ ] **8.4** Confirm a re-unlock after a wipe costs the twelve words and the
      KDF, and that the payload is intact in flash.
- [ ] **8.5** Record verbatim in `design/HARDWARE_RESULT_<date>_phaseB2b.md`.

> **Watch what you paste.** Three commit messages in this feature have claimed
> results that were never checked. Record what the screen showed.

---

## Gate coverage — state this in the R0 brief

Both gates apply and **both MUST be run before dispatch and after every fold.**

- `scripts/plan-cite-gate.sh` resolves every `file:line` and `pkg.Symbol`.
  Expected failures: symbols this plan creates (`wipeGuard`, `wipeWarningOp`,
  `runWithFlow`, `RecordsResident`).
- `scripts/plan-build-gate-go.sh` type-checks four whole-file blocks:
  `gui/run_flow.go` (174 lines), `gui/run_harness_test.go` (97),
  `gui/wipe_warning.go` (48), `gui/wipe_guard.go` (43).

**The gate has one blind spot in this plan, and it is named here rather than
discovered by a reviewer.** `Context` gains `wipe *wipeGuard` — a one-line field
added to an existing struct, which cannot be expressed as a whole file, so
`plan-build-gate-go.sh` reports
`gui/run_flow.go: ctx.wipe undefined` and TIER 1 fails. That failure is the gate
being honest, not the plan being broken.

**The controller therefore applied that one line by hand and type-checked the
result before dispatch:**

```
$ # fork copy + the plan's 4 files + `wipe *wipeGuard` on Context
$ CGO_ENABLED=0 go build ./gui/   →  BUILD OK
$ CGO_ENABLED=0 go vet  ./gui/    →  freetext_sizeproof_golden_test.go:111:13:
                                     testing.ArtifactDir requires go1.26 (file is go1.25)
```

That vet line is the **pre-existing baseline** recorded in "The green criterion"
above — byte-identical, and the only finding. **So every line of Go in this plan
type-checks, including the moved `Run` body and the harness.**

- **What remains a reviewer's execution pass:** the three TIER-2 fragments (the
  `Context` field, `Run`'s one-line delegation, and the `unlock_session.go`
  bracket/job registration), and — far more importantly — everything type-checking
  cannot reach: whether the unwind ordering is *correct*, whether a test can
  actually fail, and whether §10.2.4's semantics are what this implements.

**Machine-verified before this plan reached a reviewer** — do not re-derive:
facts 1–4 above, each confirmed against `a01b666`; `Colors` has no style field
(`gui/theme.go:30`); `op.Layer` paints later ops behind earlier ones
(`gui/gui.go:353`); `Drawer.ExtractText` takes `(image.Rectangle, op.Op)`;
`PointerEvent` is `{Pressed, Entered, Pos}`.

---

## What B2b does NOT cover

- **F-88, F-94** — further seed-equivalent copies. F-88's recorded remedy was
  **retracted**: `clear(words)` is not free, because `words` is captured by
  `frontSideSeed`'s closure and read during the cut, so clearing it cuts a
  corrupt plate.
- **F-90 items 1 and 3** — the `ms1` arm's inventory and its hook. Item 2 is
  dissolved by this design.
- **F-76**, F-80's residue, **F-92** (`tinygo test`), **F-85** and **F-98** (the
  GREEN spec's own amendments and stale cites).
- **A release tag.** Task 8 and F-85 both precede it.
