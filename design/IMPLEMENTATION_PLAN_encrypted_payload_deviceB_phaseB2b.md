# Encrypted Payload Delivery — Plan B Phase B2b (§10.2.4's residency-keyed idle wipe) — Implementation Plan

**Status:** DRAFT — R0 not yet run. **No code before 0C/0I.**

| round | verdict | report |
| --- | --- | --- |
| 0 | **3C / 5I** — all folded | `agent-reports/…-R0-round0-design.md` (opus, 1C/2I), `…-R0-round0-test-adequacy.md` (sonnet, 2C/3I) |
| 1 | pending re-review of the fold | — |

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

1. **A deadline-respecting test platform** — a *wrapper* around `testPlatform`,
   which is left alone so step 1.1's "`go test ./gui/` unchanged" stays
   achievable. It honours the deadline so `synctest` can advance the bubble's
   clock and `Run`'s idle arithmetic becomes drivable.
2. **A `runFlow` seam** so a test can run `Run` with a *chosen* flow instead of
   `uiFlow`, and observe what it draws.

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
// It deliberately does NOT model Platform.AppendEvents' wakeup channel
// (cmd/controller/platform_sh2.go:384 returns early on pl.Wakeup()), so a test
// driving a real engraveJob will not see the job-completion wakeup and falls
// back on EngraveScreen's 500 ms poll. Workable; named so it is not mistaken
// for fidelity.
type deadlinePlatform struct {
	*testPlatform
	// queued events are delivered on the next AppendEvents and refresh Run's
	// last-input clock exactly as a real touch does.
	queued []Event
	// tickFloor is the minimum fake time one AppendEvents costs. See the
	// method's comment: without it a derivation freezes the bubble clock.
	tickFloor time.Duration
}

func newDeadlinePlatform() *deadlinePlatform {
	p := newPlatform()
	// The 240x240 default is a fiction no shipped device has (gui/gui_test.go:390).
	p.display = sh2DisplaySize
	// 10ms crosses idleTimeout in 18,000 ticks -- cheap under synctest, and
	// well inside maxRunFrames.
	return &deadlinePlatform{testPlatform: p, tickFloor: 10 * time.Millisecond}
}

// AppendEvents honours the deadline. Returning the slice UNCHANGED when nothing
// is queued is the property Run's `len(evts) > 0` refresh depends on: a
// platform that appended a synthetic event every tick would refresh the idle
// clock forever and NO timer could ever fire -- the test would pass by never
// arming anything.
//
// The FLOOR is load-bearing, not defensive. unlockDerive calls
// ctx.WakeupAt(time.Now()) before every ctx.Frame (unlock_kdf.go:295), so on a
// derivation every deadline is already expired, `time.Until` is <= 0, nothing
// ever durably blocks, and a synctest bubble's clock NEVER ADVANCES. Without
// the floor, Task 5's test passes with its own mutant applied -- a false PASS,
// not a hang. The floor is a field so a long test can trade fidelity for ticks.
//
// The zero deadline (first tick, before any WakeupAt) lands here too:
// time.Until(time.Time{}) is hugely negative, so it takes the floor. That is
// correct, and stated because it looks like a bug worth "fixing".
func (p *deadlinePlatform) AppendEvents(deadline time.Time, evts []Event) []Event {
	if len(p.queued) > 0 {
		evts = append(evts, p.queued...)
		p.queued = p.queued[:0]
		return evts
	}
	d := time.Until(deadline)
	if d < p.tickFloor {
		d = p.tickFloor
	}
	time.Sleep(d)
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

// maxRunFrames bounds runSession. Run parks the flow whenever the screensaver
// activates -- the inner loop draws and `continue`s without ever returning
// control -- and under synctest the 40ms throttle costs no real time, so a
// mutant that trips the saver spins until `go test`'s 10-minute timeout and
// reports a SIGQUIT dump instead of a failure. A HANG IS WORSE THAN A FAILURE,
// and Task 7 runs many mutants unattended.
const maxRunFrames = 100000

// runSession drives Run's real body with a chosen flow, returning the extracted
// text of everything DRAWN -- content frames and Run's own warning alike.
//
// Observation goes through onDraw, not through the iterator, because Run yields
// func() bool: no value reaches the consumer (gui.go:2934). A test that counted
// iterator ticks would be asserting on nothing at all.
//
// onDraw is also the ONLY same-goroutine injection point a test has: Run blocks
// the test goroutine, and the parked flow cannot tap for itself, so
// "tap during the warning" must call p.tap() from inside the observer. Doing it
// from another goroutine is a data race.
func runSession(t *testing.T, p *deadlinePlatform, flow func(ctx *Context, version string), onDraw func(o op.Op, text string)) []string {
	t.Helper()
	var drawn []string
	r := image.Rectangle{Max: sh2DisplaySize}
	observe := func(o op.Op) {
		d := new(op.Drawer)
		txt := d.ExtractText(r, o)
		drawn = append(drawn, txt)
		if onDraw != nil {
			onDraw(o, txt)
		}
	}
	ticks := 0
	for range runWithFlow(p, "test", flow, observe) {
		ticks++
		if ticks > maxRunFrames {
			// Stop ranging: yield returns false, Run sets ctx.Done, the flow
			// unwinds. Never t.Fatal from in here -- that would Goexit through
			// a live iterator.
			break
		}
	}
	if ticks > maxRunFrames {
		last := ""
		if len(drawn) > 0 {
			last = drawn[len(drawn)-1]
		}
		t.Fatalf("Run exceeded %d ticks without terminating -- flow is probably parked "+
			"(screensaver?). %d frames drawn, last = %q", maxRunFrames, len(drawn), last)
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

// wipeNowHook forces a wipe on Run's next tick. Nil in production.
//
// It exists so the UNWIND can be tested on the commit that introduces it:
// §10.2.4's timer arrives a task later, and `wiping` is a local inside
// runWithFlow with no other seam, so without this there is no reachable path
// that sets it and none of the unwind's mutation rows can be run.
var wipeNowHook func() bool

// armed reports whether §10.2.4's timer should be running.
//
// nil receiver -- no secret session open -- is the overwhelmingly common case
// and costs two nil checks per Run tick.
//
// A RUNNING job disarms it: §10.2.4 row 2, never wipe mid-plate with the needle
// down. engraveStopping is listed for completeness rather than because it is
// reachable here -- Engrave CAN return in that state (gui/gui.go:2703), and
// the deferred g.job = nil then disarms anyway. Note what is deliberately NOT
// here: screen visibility. Row 2 as amended keys on the JOB, so the
// hold-to-start and plate-done screens are ARMED -- they are walk-away states
// with secrets still held.
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
   `for content := range it` ends: `if !wiping { return }` — otherwise fall
   through to `ctx.B.Reset()` and loop, building a **fresh `Context`** and
   re-entering the flow. `a.idle.start`, `a.idle.active` and `a.armed` are
   reset at the head of each session.

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

   **Its trigger in THIS task is `wipeNowHook`, nil in production.** §10.2.4's
   timer arrives in Task 4, and `wiping` is a local inside `runWithFlow` with no
   other seam — so without the hook this task's commit would contain no
   reachable path that sets it, and none of the mutation rows below could be
   run at all. Package-level test hooks are this package's established idiom
   (`gui/unlock_session.go:40`, `gui/unlock_kdf.go:60`, and eight others), and a
   nil hook keeps Tasks 1–3 behaviour-neutral in production exactly as the
   approved seam requires.

**Steps:**

- [ ] **3.1** Tests first, using Task 1's harness. The flow parks in `ctx.Frame`
      and **draws a session counter**, `fmt.Sprintf("SESSION %d", n)`, from a
      closure-captured `n` incremented on each entry; `wipeNowHook` fires on a
      chosen tick of session 1. Assert **`"SESSION 2"` is drawn** and that the
      flow's `defer` ran.

      > The marker must be **second-session-specific**. A constant label such as
      > `"PARKED"` is already in `drawn` from before the wipe, so the obvious
      > assertion passes identically whether or not the restart happened — it
      > would false-PASS the `break`→`return` mutant, which is the single most
      > important mutant in this plan.

- [ ] **3.2** Write the changes.
- [ ] **3.3** **Mutation checks, and these are the ones that matter.** Each
      names a literal token so Task 7's runner can apply it mechanically:

      | mutant | must be killed by |
      | --- | --- |
      | `break` → `return` at the wipe | the restart test — `"SESSION 2"` never drawn |
      | delete `if wiping { continue }` | a flow that `Frame`s after `Done` (fact 3's two screens) — the wipe becomes a GUI exit, so `"SESSION 2"` never drawn |
      | `if !wiping { return }` → `return` | the restart test |
      | hoist `wiping := false` out of the session loop | the two-wipe test — session 2's guard is still set, so session 2 draws nothing and `"SESSION 3"` never appears |
      | delete `ctx.B.Reset()` before looping | a test asserting the abandoned buffer's `refs` are zeroed |

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
// It takes the buffer and Styles EXPLICITLY rather than a *Context, because the
// one buffer it must never use is ctx.B -- see a.warnBuf in run_flow.go. Styles
// is passed because Colors carries only Background/Text/Primary
// (gui/theme.go:30) and no text style. That also rules out calling
// layoutTitle(ctx, ...), whose two lines are inlined below.
func wipeWarningOp(buf *op.Buffer, st Styles, th *Colors, dims image.Point, remaining time.Duration) op.Op {
	const margin = 8
	secs := int(remaining.Seconds() + 0.5)
	if secs < 0 {
		secs = 0
	}
	// layoutTitlef (gui/gui.go:1865) inlined -- it needs only ctx.B and Styles.title.
	title, titleSz := widget.Labelw(buf, st.title, dims.X-2*16, th.Text, "WIPING SECRET DATA")
	body, bodySz := widget.Labelwf(buf, st.body, dims.X-2*margin, th.Text,
		"This machine still holds decrypted seed material and has been idle.\n\n"+
			"It will be erased in %d seconds.\n\nTouch the screen to keep it.", secs)
	return op.Layer(
		body.Offset(image.Pt((dims.X-bodySz.X)/2, margin+titleSz.Y+margin)),
		title.Offset(image.Pt((dims.X-titleSz.X)/2, margin)),
		// Background LAST: op.Layer paints later ops BEHIND earlier ones
		// (gui.go:353, :591, :790).
		op.Color(buf, th.Background),
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
			// warnBuf is the warning's OWN buffer. It must not build into
			// ctx.B: Context.Frame resets that buffer AFTER the callback
			// (gui/gui.go:75) and Run's event loop runs INSIDE the callback, so
			// while the flow is parked ctx.B is never reset. Appending a
			// warning per second for 30 s grew it to 228 KB live / 245 KB
			// reserved on the 32-bit target -- measured -- and each of the ~7
			// doublings memcpy'd the PARKED frame, which on SeedScreen.Confirm
			// is the twelve words, into an array nothing ever zeroes.
			warnBuf op.Buffer
			idle    struct {
				start  time.Time
				active bool
				state  saver.State
			}
			armed bool
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
			a.idle.start = time.Now()
			// active must be reset too: it gates Router.Events, so a session
			// that inherited it would silently eat its first tap.
			a.idle.active = false
			a.armed = false
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
					// §10.2.4: the timer keys on the SESSION BRACKET's
					// lifetime, never on seal.RecordsResident -- which reads
					// false while the flow still holds the words and the
					// plate's spline closure.
					armed := ctx.wipe.armed()
					// ONE clock, not two. An earlier draft tracked a separate
					// wipe origin; every one of its refresh points was also an
					// idle.start refresh point, so the two were provably equal
					// -- and the single place they were allowed to diverge is
					// exactly where the latch bug below came from.
					//
					// keepAwake holds off the SCREENSAVER but is ignored while
					// armed: a screen must never be able to postpone a §10.2.4
					// wipe. Read before ctx.Reset(), which clears it.
					if len(evts) > 0 || (ctx.keepAwake && !armed) {
						a.idle.start = now
					}
					if armed != a.armed {
						a.armed = armed
						if armed {
							// §10.2.4 row 2: a finished cut starts a FRESH
							// window. Clearing `active` is not cosmetic -- it
							// gates Router.Events below, so leaving it latched
							// makes the plate-done screen look live while
							// silently eating the operator's first tap, at the
							// end of a 21-minute funds-critical cut.
							a.idle.start = now
							a.idle.active = false
						}
					}
					ctx.Reset()
					if !a.idle.active {
						ctx.Router.Events(d, evts...)
					}
					// The test-only wipe trigger: nil in production. It is the
					// ONLY trigger that exists in Task 3's commit, since
					// §10.2.4's timer below arrives in Task 4 -- which is what
					// lets the unwind be tested on the commit introducing it.
					// Package-level test hooks are this package's idiom
					// (unlock_session.go:40, unlock_kdf.go:60, and 8 others).
					if wipeNowHook != nil && wipeNowHook() {
						wiping = true
						ctx.Done = true
						break
					}
					idleWakeup := a.idle.start.Add(idleTimeout)
					idle := now.Sub(idleWakeup) >= 0
					if a.idle.active != idle {
						a.idle.active = idle
						if idle {
							a.idle.state = saver.State{}
						}
					}
					if a.idle.active {
						// Armed and idle IS §10.2.4's window. The warning takes
						// the screen the saver would otherwise have had, which
						// is why this is one branch and not a gate on the
						// saver: they can never both run.
						if armed {
							wipeAt := idleWakeup.Add(wipeWarningDelay)
							if now.Sub(wipeAt) >= 0 {
								wiping = true
								ctx.Done = true
								break
							}
							a.warnBuf.Reset()
							draw(wipeWarningOp(&a.warnBuf, ctx.Styles, &descriptorTheme,
								pl.DisplaySize(), wipeAt.Sub(now)))
							ctx.WakeupAt(now.Add(time.Second))
							continue
						}
						a.idle.state.Draw(pl)
						// Throttle screen saver speed.
						const minFrameTime = 40 * time.Millisecond
						ctx.WakeupAt(now.Add(minFrameTime))
						continue
					}
					ctx.WakeupAt(idleWakeup)
					break
				}
				startTime = time.Now()
			}
			if !wiping {
				return
			}
			// The ONLY scrubbing the abandoned Context gets: Buffer.Reset runs
			// clear(b.refs) (gui/op/op.go:374) over the last frame drawn, which
			// on the SeedScreen path is the twelve words.
			ctx.B.Reset()
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

- [ ] **4.1** Tests first, on the harness:
      - **warning + wipe** — armed, no input → the warning is drawn at 3:00 and
        the wipe fires at 3:30; flow unwound, UI restarted.
      - **the countdown is real** — at the first warning frame the drawn text
        contains `"erased in 30 seconds"`, and later `"erased in 15 seconds"`.
        Presence-only assertions would let a frozen or negative counter pass.
      - **tap during the warning** — `p.tap()` called from **inside `onDraw`**
        when the warning first appears (the only same-goroutine injection point;
        tapping from another goroutine is a data race). The window resets, no
        wipe, and content returns.
      - **not armed** — no warning ever, saver behaviour unchanged.
      - **post-cut** — a job running past `idleTimeout`, then ending: the
        window restarts from cut end, **and a tap on the plate-done screen
        reaches the flow**.
- [ ] **4.2** Write the file and fragments.
- [ ] **4.3** **Mutation checks:**

      | mutant | must be killed by |
      | --- | --- |
      | `armed` hardcoded true | the not-armed test — a wipe on the public plate list |
      | `armed` hardcoded false | the wipe test |
      | delete `a.idle.start = now` from the armed edge | the post-cut test — instant warning at cut end |
      | delete `a.idle.active = false` from the armed edge | the post-cut **tap** test — the plate-done screen looks live and eats the tap |
      | delete `a.idle.active = false` at session head | the post-wipe tap test |
      | delete the `if armed` inside the idle branch | the warning test — the saver draws instead of the warning |
      | `wipeWarningDelay` → 0 | the warning-visible test |
      | delete `secs < 0` clamp | a countdown test at a negative remaining |
      | delete `a.warnBuf.Reset()` | a test asserting the buffer does not grow across warning ticks |
      | `&a.warnBuf` → `&ctx.B` | the same buffer-growth test — this is C1 restored |

- [ ] **4.4** `go test ./gui/`, device build, `gofmt`, commit.

---

## Task 5 — F-93: the screensaver must not park a derivation

A derivation produces no events, so `Run`'s `len(evts) > 0` refresh never fires
and any derivation longer than `idleTimeout` trips the saver — which then
**parks the flow permanently**, because the saver branch `continue`s without
ever returning control. Verified reachable with a **conforming** blob: §6.2
allows up to 2,000,000 iterations, §7.1 measures 9,715 it/s on device, and
`idleTimeout` is 3 min (`gui/gui.go:2932`), so anything above
180 × 9,715 = **1,748,700 iterations** parks — **13.2%** of the legal range
(251,300 of 1,900,000).

**Three parts, and the `Run` side is not optional:**

1. **`Context` gains `keepAwake bool`, a `KeepAwake()` setter, and `Reset()`
   clears it** — `Reset` is the natural per-tick clear, and `Run` reads the flag
   **before** calling it. That ordering is load-bearing: reversed, the flag is
   lost every tick and the task silently does nothing.
2. **`unlockDerive` calls `ctx.KeepAwake()` each slice.**
3. **`Run` consumes it** — the `(ctx.keepAwake && !armed)` term already present
   in Task 4's gated block. **The `&& !armed` is normative, not caution:** with
   one clock, an ungated `keepAwake` would let a screen postpone the §10.2.4
   wipe indefinitely, which the section forbids. `KeepAwake` holds off the
   screensaver and nothing else.

- [ ] **5.1** Test on the harness with the `newDeriver` seam supplying a fake
      deriver, so slices are instant and the bubble crosses `idleTimeout` in
      ~18,000 ticks: a derivation longer than `idleTimeout` **completes** and
      returns its key.

      > **Why this asserts something.** The mutant — delete the `ctx.keepAwake`
      > term — parks the flow under the saver forever. With `maxRunFrames` the
      > test reports *"Run exceeded 100000 ticks… flow is probably parked"*
      > instead of hanging to the 10-minute binary timeout. The frame cap is
      > what converts this mutant from a hang into a kill.
      >
      > Without `deadlinePlatform.tickFloor` this test would be a **false
      > PASS**: `unlockDerive` calls `ctx.WakeupAt(time.Now())` before every
      > `ctx.Frame` (`gui/unlock_kdf.go:295`), so every deadline is already
      > expired, nothing durably blocks, the bubble clock never advances, the
      > saver never fires — and the mutant passes too.

- [ ] **5.2** Second mutant: **swap the read past `ctx.Reset()`** → the flag is
      always false and the derivation parks. Must be killed by the same test.
- [ ] **5.3** Third mutant: **drop `&& !armed`** → must be killed by a test in
      which an armed session calls `KeepAwake` and the wipe still fires on time.
- [ ] **5.4** Implement, run, `gofmt`, commit.

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
  `gui/run_flow.go` (204 lines), `gui/run_harness_test.go` (154),
  `gui/wipe_warning.go` (51), `gui/wipe_guard.go` (51).

**The gate has one blind spot in this plan, and it is named here rather than
discovered by a reviewer.** `Context` gains `wipe *wipeGuard` and
`keepAwake bool` — fields added to an existing struct, which cannot be expressed
as whole files, so `plan-build-gate-go.sh` reports `ctx.wipe undefined` and
`ctx.keepAwake undefined` and TIER 1 fails. That failure is the gate being
honest, not the plan being broken.

**The controller therefore applied those fragments by hand and type-checked the
result — before dispatch, and again after the round-0 fold:**

```
$ # fork copy + the plan's 4 files + Context.wipe, Context.keepAwake,
$ # KeepAwake() and Reset()'s clear
$ CGO_ENABLED=0 go build ./gui/   →  BUILD OK
$ CGO_ENABLED=0 go vet  ./gui/    →  freetext_sizeproof_golden_test.go:111:13:
                                     testing.ArtifactDir requires go1.26 (file is go1.25)
```

That vet line is the **pre-existing baseline** recorded in "The green criterion"
above — byte-identical, and the only finding. **So every line of Go in this plan
type-checks, including the moved `Run` body and the harness.**

- **What remains a reviewer's execution pass:** the TIER-2 fragments (the
  `Context` fields and `KeepAwake`/`Reset`, `Run`'s one-line delegation, the
  `unlock_session.go` bracket/job registration, and `unlockDerive`'s
  `KeepAwake` call), and — far more importantly — everything type-checking
  cannot reach: whether the unwind ordering is *correct*, whether a test can
  actually fail, and whether §10.2.4's semantics are what this implements.

### Round 0 (2026-08-09) — 3C/5I, all folded

Two independent lenses, persisted verbatim before folding:
`design/agent-reports/encrypted-payload-planB-phaseB2b-R0-round0-design.md`
(opus, 1C/2I) and `…-round0-test-adequacy.md` (sonnet, 2C/3I).

| finding | disposition |
| --- | --- |
| **A-C1** warning built into `ctx.B`, never reset while the flow is parked: 228 KB live, and ~7 unzeroed memcpys of the rendered seed | dedicated `a.warnBuf`; `wipeWarningOp` takes a buffer + `Styles`; `ctx.B.Reset()` before the session loops |
| **A-I1** `a.idle.active` latched across the armed edge — plate-done screen eats the first tap | **merged the two clocks**; armed edge resets `idle.start` *and* `idle.active`; session head too |
| **A-I2** Task 5 had no `Run` code, no gate, and a self-passing mutant | three explicit parts, `Run` consumption in the gated block, `tickFloor`, three mutants |
| **B-C1** Task 3's own tests could not be constructed — no reachable writer of `wiping` | `wipeNowHook`, nil in production, this package's own idiom |
| **B-C2** no frame cap: several mutants hang rather than fail | `maxRunFrames`, `t.Fatalf` naming the likely park |
| **B-I1** restart assertion would false-PASS the `break`→`return` mutant | `"SESSION %d"` counter, assert `"SESSION 2"` |
| **B-I2** the `&& !armed` saver-gate mutant might not discriminate | **dissolved** — one clock makes it an `if/else`, and "delete the `if armed`" is unambiguously killed |
| **B-I3** countdown number untested | explicit `"erased in 30 seconds"` assertions + a `secs < 0` mutant |

Minors/nits folded: `engraveStopping`'s over-claiming comment, the harness's
unmodelled wakeup channel, `onDraw` as a parameter, the `continue`/`return`
mismatch, and the two untestable mutation rows.

**The one finding NOT folded here is A's §10.2.4 row-1 ambiguity** — the spec
does not fix whether "3 min, 30 s warning" means warn@3:00/wipe@3:30 (this
plan's reading) or warn@2:30/wipe@3:00. Filed as **F-99, owned by Task 8**: it
needs an operator-approved one-sentence spec amendment *before* the hardware
pass, so Task 8 cannot ratify an unstated choice.

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
