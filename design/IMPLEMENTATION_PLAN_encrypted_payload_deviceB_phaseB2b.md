# Encrypted Payload Delivery — Plan B Phase B2b (§10.2.4's residency-keyed idle wipe) — Implementation Plan

**Status: GREEN — R0 closed 2026-08-09 at round 3 (0C/0I). Implementation may begin.**

| round | verdict | report |
| --- | --- | --- |
| 0 | **3C / 5I** — all folded | `agent-reports/…-R0-round0-design.md` (opus, 1C/2I), `…-R0-round0-test-adequacy.md` (sonnet, 2C/3I) |
| 1 | **0C / 9I** — all folded | `…-R0-round1-fold-rereview.md` (opus, 0C/5I — the fold's own defects), `…-R0-round1-residue-sweep.md` (36-agent workflow, 0C/4I — Tasks 6–8, constraints, follow-up ownership) |
| 2 | **0C / 3I** — all folded | `…-R0-round2-fold-rereview.md` (opus). Closed the two structural risks by tracing: the deleted tail `ctx.B.Reset()` is genuinely dead, and the removed armed-edge `a.idle.active = false` self-clears |
| 3 | **0C / 0I — GREEN, loop CLOSED** | `…-R0-round3-fold-rereview.md` (opus). Executed the `flowBound` panic path in a reproduction rather than reasoning about it; type-checked the test package under go1.26. Its 5 Minors and 4 Nits were folded inline, not deferred. |

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
operator-complete.** It is *not* the last thing before a tag — see "The release
tag's precondition set" at the end, which is the single place that list lives.

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
| **F-88, F-94** further seed copies | — | **B2c** |
| **F-90** items 1 and 3 (`ms1` inventory, hook) | — | **B2c** |
| **F-76**, F-80's residue | — | after B2b |

> **"B2c" is a NAMED successor phase, not a synonym for later.** These three
> were recorded in `FOLLOWUPS.md` as owned by **B2b**, and an earlier draft of
> this table deferred them to "own cycle" — which is not a later phase, it is no
> phase, and `/scratch/code/CLAUDE.md` forbids parking an item on nothing. The
> work is real and is not B2b-sized (F-88's only actionable copy is a
> `bip39.MnemonicSeed` change five other flows call, wanting its own review), so
> the three entries were **re-assigned to B2c — secret-residency cleanup — on
> 2026-08-09**, and say so. Silence was the defect; a named owner is the fix.

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
| `go vet ./gui/op/` | **exit 1**, only `gui/op/draw_test.go:176:24: testing.ArtifactDir requires go1.26 or later` — new row, because Task 1 adds `gui/op/buffer_len.go` |
| `gofmt -l <touched>` | empty |
| TinyGo device build — **run `.github/workflows/test.yml:29` verbatim; do not restate it by hand** | baseline **1310184 flash / 60584 ram** — report the new numbers |
| `CGO_ENABLED=0 GOARCH=386 go test ./seal/ ./bip39/` | green, **~52 s** (not a hang) |

> **The TinyGo row is a CITATION, not a transcription, and that is deliberate.**
> An earlier draft of this row spelled the command out as
> `tinygo build -o /dev/null -target=pico2-w -size=short ./cmd/controller`. It
> **does not compile** — `cmd/controller/platform_sh2.go:128:19: undefined:
> machine.GPIO30`, because `pico2-w` is RP2350**A** and the SeedHammer II is
> RP2350**B** (`pico-plus2`). Even on the right target the numbers could not have
> matched: the recorded baseline needs `-opt 2 -gc precise -scheduler tasks
> -stack-size 16kb`, all of which the hand-written version dropped. Verified: the
> workflow's own line reproduces `1310184 / 60584` exactly. **Tasks 3.4 and 4.4
> both require this build**, so a broken row blocks two task gates, in the phase
> whose most expensive finding was a 228 KB buffer growth — this is the only
> RAM-budget signal there is.
>
> **`CGO_ENABLED=0` on the 386 row is load-bearing, not decoration.** Without it
> the row is **RED at the baseline** — `# runtime/cgo … gnu/stubs-32.h: No such
> file or directory`, `FAIL seedhammer.com/seal [build failed]` — because a
> 32-bit cgo toolchain is not installed. It was dropped in transcription from
> `agent-reports/…-phaseB2a-ii-lens8-completeness.md:241`. A definition of
> "done" with a permanently red row teaches the implementer that red is
> expected, which is how a genuine `seal` regression on the **firmware's own
> word size** gets waved through. Measured both ways before this row was fixed.

### B2b-specific

- **No shared screen learns a new exit.** `ChoiceScreen.Choose` and
  `SeedScreen.Confirm` are used across the whole firmware; the design's central
  claim is that they need **no change**. If a task finds itself editing one, the
  design is wrong and the task should stop.
- **No `recover` in NON-TEST code.** Measured: the fork has **0** real `recover()`
  calls outside tests (the one grep hit, `backup/freetext.go:56`, is inside a
  comment), and this plan adds none. **`panic` is a different matter and the rule
  does NOT forbid it** — the fork has **129** `panic(` sites in non-test Go, and
  this plan's own `gui/run_flow.go` block reproduces one verbatim (`panic(err)`
  in the `draw` closure, carried over unchanged from the existing `Run` body).
  An earlier draft said "no `panic`/`recover`", which was simply false and would
  have told the implementer that the gated block violates the plan.

  The one `recover` this plan introduces is `runSession`'s, paired with
  `boundedFlow`'s `panic(flowBound{})`, **in `run_harness_test.go` only**. It is
  there because both alternatives are worse: `t.Fatal` would `Goexit` through a
  live iterator, and `t.Errorf`-then-return ends only the current session,
  letting the very mutant it guards run to the 10-minute timeout.
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

**The file's FINAL form — after Tasks 3, 4 and 5 — is given in full in Task 4,
and that is the copy the build gate type-checks.** Task 1's version is that file
minus the session loop, the discard guard, `wipeNowHook`, the whole `a.armed` /
warning / wipe block, the `ctx.keepAwake` term and `warnBufHook`: a pure
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
	// dirties counts Dirty calls; onDirty observes them. Together they are the
	// screensaver's only visibility, and the only un-park seam.
	dirties int
	onDirty func(n int)
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

// Dirty counts refreshes and is the ONLY way a test can see the screensaver.
// saver.State.Draw writes straight to the platform (gui/saver/saver.go:311
// calls screen.Dirty) and never reaches onDraw, so without this, step 1.3's
// "must observe Run's saver activate" has nothing to observe. onDirty is also
// the only same-goroutine seam that can UN-park a parked flow.
func (p *deadlinePlatform) Dirty(r image.Rectangle) error {
	p.dirties++
	if p.onDirty != nil {
		p.onDirty(p.dirties)
	}
	return p.testPlatform.Dirty(r)
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
// It returns parked=true instead of failing, because PARKING IS SOMETIMES THE
// EXPECTED RESULT: the unarmed screensaver legitimately parks the flow forever
// (the saver branch continues without returning control), and two of this
// plan's own tests -- step 1.3's saver self-check and step 4.1's "not armed" --
// exist precisely to observe that. A harness that always t.Fatal'd on a park
// would fail them unconditionally. Callers that require completion use
// mustFinish.
func runSession(t *testing.T, p *deadlinePlatform, flow func(ctx *Context, version string), onDraw func(o op.Op, text string)) (drawn []string, parked bool) {
	t.Helper()
	defer func() {
		r := recover()
		if r == nil {
			return
		}
		if _, ok := r.(flowBound); !ok {
			panic(r) // a real bug; never swallow it
		}
		t.Errorf("test flow exceeded %d iterations without ctx.Done -- Run is "+
			"discarding every frame (wiping stuck true?)", maxRunFrames)
	}()
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
			parked = true
			break
		}
	}
	return drawn, parked
}

// mustFinish is runSession for the common case, where a park is a failure.
func mustFinish(t *testing.T, p *deadlinePlatform, flow func(ctx *Context, version string), onDraw func(o op.Op, text string)) []string {
	t.Helper()
	drawn, parked := runSession(t, p, flow, onDraw)
	if parked {
		last := ""
		if len(drawn) > 0 {
			last = drawn[len(drawn)-1]
		}
		t.Fatalf("Run exceeded %d ticks without terminating -- flow is probably parked "+
			"(screensaver?). %d frames drawn, last = %q", maxRunFrames, len(drawn), last)
	}
	return drawn
}

// boundedFlow wraps a test flow so it cannot spin forever.
//
// maxRunFrames alone is NOT enough, and the gap is specific: ticks are counted
// in runSession's range body, which is driven by yield() at the top of Run's
// INNER loop -- but the discard guard (`if wiping { continue }`) is the first
// statement of the range body and skips that loop entirely. A mutant that
// leaves `wiping` stuck true therefore burns CPU with ZERO ticks and zero fake
// time, and the cap never trips. The flow is the thing spinning, so bounding
// the flow is the only cap that survives the guard.
// It PANICS with a sentinel rather than calling t.Errorf and returning, and the
// difference is the whole point. Returning ends only the current SESSION: the
// session loop sees `wiping` still true, builds a fresh Context, and calls the
// flow again with a fresh counter -- so the mutant this exists to catch runs
// forever, now emitting an unbounded stream of failures. t.Fatal is barred too,
// since Goexit through a live iterator is not safe. The panic unwinds
// runWithFlow entirely and cannot re-enter the session loop; runSession recovers
// it and turns it into one clean failure.
func boundedFlow(t *testing.T, body func(ctx *Context) bool) func(*Context, string) {
	t.Helper()
	return func(ctx *Context, _ string) {
		for n := 0; !ctx.Done; n++ {
			if n > maxRunFrames {
				panic(flowBound{})
			}
			if !body(ctx) {
				return
			}
		}
	}
}

// flowBound is boundedFlow's sentinel. A distinct type so runSession can
// re-panic anything else rather than swallowing a real bug.
type flowBound struct{}

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

- [ ] **1.1** Do the move (1a). **Also delete the now-orphaned
      `"seedhammer.com/gui/saver"` import from `gui/gui.go`** — `saver` is
      referenced only at `gui/gui.go:2942` and `:3004`, both inside `Run`'s
      body, so moving that body out leaves the import unused and **package `gui`
      does not compile**. It is the one line that makes this "pure move" not
      purely a move, and no gate can catch it (see the blind-spot note below).
      Then `go test ./gui/` must be **unchanged** — a pure move that changes a
      test result is not a pure move.
- [ ] **1.2** Write the harness. Smoke test: a flow that returns immediately
      finishes under `mustFinish`; a flow that loops `for !ctx.Done` drawing a
      label per tick produces frames whose text `assertDrawn` finds.

      > **Entry point is part of each test's assertion, not a detail.** Use
      > `mustFinish` wherever completion is required — it fails loudly on a park.
      > Use `runSession` **only** where a park is the expected outcome (step 1.3
      > and step 4.1's "not armed"), and then **assert on `parked` explicitly**;
      > a test that ignores the second return value passes vacuously.
      >
      > Note `parked` means "hit `maxRunFrames`", not "the flow parked" in
      > general — the discard-guard spin never reaches the tick counter at all,
      > which is what `boundedFlow` is for.
- [ ] **1.3** **Prove the deadline is honoured**, because that is the whole
      point: a test that sleeps past `idleTimeout` inside a `synctest` bubble
      must observe `Run`'s **saver** activate. If it does not, the platform is
      not driving the clock and every later task asserts on nothing.

      > **The discriminator, since a raw `dirties` count is not one.** `Dirty` is
      > called by the content path (via `draw`) *and* **twice per saver frame**
      > (`saver.State.Draw` -> `newDraw`, `gui/saver/saver.go:328` and `:353`),
      > so the total proves nothing. **A saver frame is a `Dirty` with no
      > following `onDraw`** -- the saver bypasses the op pipeline entirely.
      > Record the interleaving in `onDirty`/`onDraw` and assert on that
      > pairing. Use `runSession` here, not `mustFinish`: a parked flow is the
      > expected outcome, so assert `parked == true` explicitly.
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
- [ ] **2.3** **Mutation checks:**

      | file | anchor (unique) | → replace with | must be killed by |
      | --- | --- | --- | --- |
      | `unlock_session.go` | `defer func() { ctx.wipe = nil }()` | *delete the line* | 2.1 — a bracket that fails to uninstall leaves the timer **armed during the public plate list**, which is operator-hostile |
      | `wipe_guard.go` | `		case engraveRunning, engraveStopping:` | `case engraveIdle:` | 2.1's "`armed()` is false while a job runs", and Task 4.1's post-cut test. **This is SPEC §11.3's row "idle timer runs during engraving"** — the one §11.3 row B2b owns, deferred to this phase by B2a-ii's Task 8 because no timer existed then. Task 4's `armed` hardcoded true/false does **not** substitute: under this mutant `armed()` still returns false when `ctx.wipe == nil`, so the not-armed test cannot discriminate it |
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

2. **The discard guard** — `if wiping { continue }`, the first statement in the
   range body, before any draw. *(Not repeated as a code block here: Task 4's
   gated whole file is the single source, and a second copy is a second thing to
   keep in sync — it also made `if wiping {` match twice, which is precisely the
   ambiguity `plan-mutation-anchors.py` rejects.)*

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

- [ ] **3.2** Write the changes. **Every flow in Tasks 3–5's tests goes through
      `boundedFlow`** — an unwrapped flow is what lets the hoist-`wiping` mutant
      spin forever, because the discard guard means `runSession`'s tick counter
      never advances and `maxRunFrames` cannot see it.
- [ ] **3.3** **Mutation checks, and these are the ones that matter.** Each
      names a literal token so Task 7's runner can apply it mechanically:

      | file | anchor (unique) | → replace with | must be killed by |
      | --- | --- | --- | --- |
      | `run_flow.go` | `break // unwind, never exit` | `return` | the restart test — `"SESSION 2"` never drawn. The trailing comment makes this line unique: bare `break` matches 5 sites in this file, one of them the `pl.NextChunk()` chunk walk, where substituting silently truncates every frame |
      | `run_flow.go` | `if wiping {` | delete the whole 3-line statement | a flow that `Frame`s after `Done` (fact 3's two screens) — the wipe becomes a GUI exit, so `"SESSION 2"` never drawn |
      | `run_flow.go` | `if !wiping {` | `if false {` | `mustFinish`'s cap — this mutant never *exits* the session loop (it is no longer the old "return unconditionally"), so it spins to `maxRunFrames` and fails there rather than by a missing `"SESSION 2"`. The restart property itself is covered by the `break`→`return` row above |
      | `run_flow.go` | `			wiping := false` | hoist above `for {` | the **two-wipe** test — and note this one is caught by `boundedFlow`, not by `maxRunFrames`: the discard guard skips the inner loop, so `yield()` is never called and ticks never increment |

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

// warnBufHook reports the warning buffer's size after each warning frame. Nil
// in production.
//
// It exists because A-C1's three mutation rows are otherwise UNWRITABLE:
// op.Buffer's args/refs are unexported with no accessor (gui/op/op.go:28), and
// the buffer itself is a field of a closure-local struct in runWithFlow. Without
// this, "the warning grew ctx.B unboundedly" -- the Critical this phase's most
// expensive finding was about -- has no test that can fail.
var warnBufHook func(args, refs int)

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

### One accessor, in the `op` package

`op.Buffer` is `{args []uint32; refs []any}` (`gui/op/op.go:28`) — both
unexported, no accessor — so **nothing outside package `op` can observe its
size**. That is what made A-C1's mutation rows unwritable. A method may live in
any file of its package, so this goes in a new file rather than a fragment, and
the gate type-checks it.

Create `gui/op/buffer_len.go`:

```go
package op

// Len reports the buffer's current fill. It exists so a test can assert that a
// long-lived buffer is not growing across frames -- the defect class behind the
// §10.2.4 warning's 228 KB accumulation, which was invisible to every other
// seam because both fields are unexported.
func (b *Buffer) Len() (args, refs int) {
	return len(b.args), len(b.refs)
}
```

### The final form of the moved body — and what the gate type-checks

Create `gui/run_flow.go` (Task 1 creates it as a pure move; this is its state
after Tasks 3, 4 and 5, and the copy `plan-build-gate-go.sh` type-checks):

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
			// active is reset too. It gates Router.Events, so a session
			// inheriting it eats that first TICK's events -- one tick, not the
			// whole session, since the line below recomputes it immediately.
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
							// window. This ONE line is the whole fix: with the
							// clock reset, `idle` recomputes false on this very
							// tick and the block below clears a.idle.active by
							// itself.
							//
							// Deliberately NOT also clearing a.idle.active
							// here. It would only change the edge TICK, and
							// changing it is worse: `d` still holds the frame
							// drawn before the saver activated ~18 min ago, in
							// a different EngraveScreen state, so routing a
							// touch against it could hit the wrong widget.
							// Swallowing the edge-tick touch is exactly today's
							// screensaver-dismissal behaviour.
							a.idle.start = now // row 2: fresh window at cut end
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
						break // unwind, never exit
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
						if armed { // §10.2.4's window: warn, then wipe
							wipeAt := idleWakeup.Add(wipeWarningDelay)
							if now.Sub(wipeAt) >= 0 {
								wiping = true
								ctx.Done = true
								break
							}
							a.warnBuf.Reset()
							draw(wipeWarningOp(&a.warnBuf, ctx.Styles, &descriptorTheme,
								pl.DisplaySize(), wipeAt.Sub(now)))
							// The only way a test can see WHICH buffer the
							// warning went into, or that it is not growing:
							// op.Buffer's fields are unexported and `a` is a
							// closure local. Nil in production.
							if warnBufHook != nil {
								args, refs := a.warnBuf.Len()
								warnBufHook(args, refs)
							}
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
			// NOTHING to scrub here, and that is worth stating because an
			// earlier draft got it backwards. Context.Frame runs c.B.Reset()
			// AFTER the callback (gui/gui.go:75), and the wipe path uses
			// `break`, so the range body completes, yield returns true, the
			// callback returns, and clear(b.refs) (gui/op/op.go:376) runs on
			// the last frame drawn -- then again after every discard-guarded
			// Frame during the unwind. The abandoned Context's buffer is
			// already zeroed by the time control reaches this line.
		}
	}
}
```

**Fragments, `gui/gui.go`:** `Run`'s body becomes
`return runWithFlow(pl, version, uiFlow, nil)`; the orphaned `saver` import goes
(step 1.1); and **`Context` gains `wipe *wipeGuard` and `keepAwake bool`, a
`KeepAwake()` setter, and a `keepAwake` clear in `Reset()`.**

> **The `keepAwake` fragments land HERE, in Task 4, not in Task 5.** The gated
> block above reads `ctx.keepAwake`, so a Task 4 that shipped without the field
> would not compile and step 4.4 could never go green. Task 5 supplies the
> *caller* (`unlockDerive`) and the tests; Task 4 supplies the field it reads.

Four properties worth naming, because a reviewer should confirm them rather
than re-derive them:

- **`a.idle.start` has THREE sources** — `len(evts) > 0` (true physical input),
  the `armed` false→true edge, and `ctx.keepAwake && !armed`. Dropping the
  second gives an **instant wipe with no warning at all** the moment a plate
  finishes: the clock is ~21 min stale, so `now ≥ wipeAt` is already true and
  the warning branch is never reached. Dropping the `&& !armed` lets a screen
  postpone a §10.2.4 wipe indefinitely.
- **A tap during the warning cannot activate what is underneath** — but not for
  the reason it first appears. `if !a.idle.active { ctx.Router.Events(...) }`
  skips routing entirely while the warning is up, so the touch refreshes
  `a.idle.start` and nothing else. (The warning op also carries no tags, but
  that is belt, not braces.)
- **The armed edge resets the clock only, NOT `a.idle.active`.** Clearing
  `active` there would route the edge tick's touch against a `d` last filled
  before the saver activated ~18 min earlier, in a different `EngraveScreen`
  state. Swallowing that one touch is exactly today's screensaver-dismissal
  behaviour, and `a.idle.active` clears by itself on the same tick.
- **When `armed` is false the event loop matches today, with ONE deliberate
  exception:** `ctx.keepAwake` is a new third refresh source and `unlockDerive`
  runs unarmed, so a derivation longer than `idleTimeout` no longer trips the
  saver. That is F-93's entire point (Task 5) — it is a change, and it is
  intended. Everything else, including the saver covering a running 21-minute
  cut, is unchanged.

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

      Every row is an **anchored triple** — the unique line to match, and what to
      put in its place — because a bare token is not appliable: `break` occurs
      5× in this plan's own blocks (one of them the `pl.NextChunk()` chunk walk,
      where substituting silently truncates every frame), and
      `if !wiping { return }` occurs 0× as written, being three lines. (6 `break`
      lines across all blocks; 5 in `run_flow.go`, of which one is a comment.)

      | file | anchor (unique) | → replace with | must be killed by |
      | --- | --- | --- | --- |
      | `run_flow.go` | `armed := ctx.wipe.armed()` | `armed := true` | the not-armed test — a wipe on the public plate list |
      | `run_flow.go` | `armed := ctx.wipe.armed()` | `armed := false` | the wipe test |
      | `run_flow.go` | `a.idle.start = now // row 2: fresh window at cut end` | *delete the line* | the post-cut test — **instant wipe with NO warning**: the clock is ~21 min stale, so `now ≥ wipeAt` already holds and the warning branch is skipped entirely. The trailing comment exists to make this line a UNIQUE anchor — the bare statement occurs twice, and `scripts/plan-mutation-anchors.py` fails the plan if it does |
      | `run_flow.go` | `if armed { // §10.2.4's window: warn, then wipe` | `if false {` | the warning test — the saver draws instead of the warning. The trailing comment disambiguates it from the armed-edge `if armed {` |
      | `wipe_warning.go` | `const wipeWarningDelay = 30 * time.Second` | `= 0` | the warning-visible test |
      | `wipe_warning.go` | `	if secs < 0 {` | `if false {` | a **direct unit call** of `wipeWarningOp` with a negative `remaining` — unreachable from `Run`, since `wipeAt.Sub(now)` is only evaluated after `now.Sub(wipeAt) >= 0` is ruled out |
      | `run_flow.go` | `							a.warnBuf.Reset()` | *delete the line* | the buffer test — `warnBufHook` sees `args` growing across warning ticks |
      | `run_flow.go` | `draw(wipeWarningOp(&a.warnBuf, ctx.Styles, &descriptorTheme,` | `&ctx.B` in place of `&a.warnBuf` | the same buffer test — `warnBufHook` reports `a.warnBuf` still `(0, 0)`. **This is A-C1 restored**, so it is the row that matters most |
      | `run_flow.go` | `ctx.keepAwake` | `false` | step 5.1 — the derivation parks under the saver and `mustFinish` reports the cap. F-93's own mutant; a row rather than prose because Task 7 owns "a list, not a judgement call" |
      | `run_flow.go` | `(ctx.keepAwake && !armed)` | `(ctx.keepAwake)` | step 5.3 — an armed session calling `KeepAwake` must still wipe on time. Anchored on the parenthesised sub-expression because the full `if` line contains `||`, which a markdown table cell cannot carry |

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

1. **The `Context` field, `KeepAwake()` and `Reset()`'s clear land in TASK 4**,
   not here — Task 4's gated block reads `ctx.keepAwake`, so Task 4 would not
   compile without them. What this task adds is the *caller* and the tests.
   `Run` reads the flag **before** calling `ctx.Reset()`, and that ordering is
   load-bearing: reversed, the flag is lost every tick and the task silently
   does nothing.
2. **`unlockDerive` calls `ctx.KeepAwake()` each slice.**
3. **`Run` consumes it** — the `(ctx.keepAwake && !armed)` term already present
   in Task 4's gated block. **The `&& !armed` is normative, not caution:** with
   one clock, an ungated `keepAwake` would let a screen postpone the §10.2.4
   wipe indefinitely, which the section forbids. `KeepAwake` holds off the
   screensaver and nothing else.

- [ ] **5.1** Test on the harness, driving the clock with **`p.tickFloor = 1 s`**:
      a derivation longer than `idleTimeout` **completes** and returns its key.

      > **A fake deriver is NOT available, and the arithmetic decides the test.**
      > `newDeriver` is `var newDeriver = seal.NewDeriver` returning a *concrete*
      > `*seal.Deriver` (`gui/unlock_kdf.go:51`), so a test can choose only the
      > iteration count. Each frame costs `kdfStepIterations = 500` real PBKDF2
      > iterations (`gui/unlock_kdf.go:26`). At the default 10 ms floor, crossing
      > `idleTimeout` takes ~18,000 ticks = **9,000,000 iterations — 4.5×
      > `seal.MaxIterations`** (`seal/wire.go:37`) and far too slow. At a 1 s
      > floor it is 180 ticks = **90,000 iterations**, which is legal, fast, and
      > still crosses the 3-minute deadline in the bubble.

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
  it leaves the package green, because no test drives the three early returns
  (`gui/unlock_session.go:257`, `:266`, `:272`) to an observable point.

> **F-87's recorded remedy — "drive each with `unlockMnemonicHook` set" — does
> NOT work, and Task 6 must not inherit it.** `unlockMnemonicHook` has exactly
> **one** call site, `gui/unlock_session.go:292`, on the success path *after*
> `clear(rec)` and `clear(m)`. **No early return reaches it.** The natural test —
> capture `m` in the hook, range over it asserting zeros — would range over a
> `nil` slice, assert nothing, and **pass with the defer deleted**. That is the
> exact false-PASS class this phase has already paid for twice.
>
> The exposure is bounded and worth stating so the fix is not over-built: `m` is
> `bip39.Parse(rec)` of the payload **seed record**, not §8's KDF passphrase, and
> the window is a `showError` screen, not the ~21-minute cut.

So Task 6 must **add a seam**, in a gated code block.

Create `gui/unlock_mnemonic_seam.go`:

```go
package gui

import "seedhammer.com/bip39"

// unlockMnemonicParsedHook fires immediately after unlockEngraveMnemonic's
// `defer clear(m)` is registered, so a test can hold the SAME backing array the
// defer will zero and assert it was zeroed on every early return.
//
// unlockMnemonicHook cannot do this: it has one call site
// (gui/unlock_session.go:292), on the success path after clear(m), which no
// early return reaches. A test built on it ranges over nil, asserts nothing,
// and passes with the defer deleted.
var unlockMnemonicParsedHook func(bip39.Mnemonic)
```

- [ ] **6.0** Add the call site. Modify `gui/unlock_session.go`, immediately
      after `defer clear(m)` at `:250`:

```go
	if unlockMnemonicParsedHook != nil {
		unlockMnemonicParsedHook(m)
	}
```

      It must come **after** the defer is registered, so the hook hands the test
      the same backing array the defer will zero. Without this step the seam file
      compiles and does nothing, which is how the original F-87 remedy failed.
- [ ] **6.1** Tests first, one per early return (`:257` `!ss.Confirm`, `:266`
      `masterFingerprintFor` err, `:272` `engraveSeed` err). Each captures `m`
      via `unlockMnemonicParsedHook`, drives that return, and asserts every word
      is zero.

      **Each test MUST assert the hook fired** — `if got == nil { t.Fatal(...) }`
      — before asserting on its contents. Without that guard the test is
      vacuous, which is the defect this task exists to fix. The idiom already
      exists 30 lines away (`gui/unlock_session_test.go:678`,
      `bip39/bip39_test.go:393`).
- [ ] **6.2** Mutation checks:

      | file | anchor (unique) | → replace with | must be killed by |
      | --- | --- | --- | --- |
      | `unlock_session.go` | `defer clear(m)` | *delete the line* | all three of 6.1's tests |
      | `unlock_session.go` | `unlockMnemonicParsedHook(m)` | *delete the line* | 6.1's fired-guard — proves the guard is real |
- [ ] **6.3** Rename `SecretsResident` → `RecordsResident`, update callers and
      docs, commit. **Do not rewrite the persisted `agent-reports/`** — they are
      verbatim records of what a reviewer said and are not editable artifacts.

---

## Task 7 — the §11.3 rows, and F-96's runner

**Commit the mutation runner** as `scripts/mutation-run.py` in `mnemonic-engrave`,
with the row table as data and a printed statement of what it does **not** cover
— the shape `plan-build-gate-go.sh` uses. `CLAUDE.md`'s standing rule: *"when an
artifact will be folded repeatedly, commit the extractor as a script so the check
is a command, not a thing to remember."* B2a-ii ran ~50 mutants by hand for want
of it.

**§11.3's two procedural rules are normative and MUST be in the runner**, not
left as discipline:

1. **Assert the substitution matched, exactly once.** §11.3: *"a silently-failing
   `sed` reads exactly like a surviving mutation."* A match count ≠ 1 is a hard
   error, never a mutant result. This is why every row above is an anchored
   triple rather than a bare token.
2. **Restore from a file copy, never `git checkout`.**

**The rows B2b owns**, so "every row this phase owns" is a list and not a
judgement call:

- **SPEC §11.3's one B2b row** — *"idle timer runs during engraving"*, deferred
  to this phase by B2a-ii's Task 8 because no timer existed then. It is Task
  2.3's `engraveRunning, engraveStopping` row.
- **Every anchored row in Tasks 2.3, 3.3, 4.3 and 6.2** of this plan. Task 5's
  mutants live in 4.3's table (they target `run_flow.go`), and **step 5.2 is
  deliberately NOT a row**: "swap the read past `ctx.Reset()`" is a statement
  *reordering*, not a substitution, so no anchor can express it. It stays a
  hand-run ordering check, and saying so is the point — an unrunnable row in the
  table would read as a clean run.

- [ ] **7.1** Write the runner with the row table as data. Re-run every row
      above through it. Print, in the runner's own output, what it does **not**
      cover — the shape `plan-build-gate-go.sh` uses.
- [ ] **7.2** A surviving mutant is **blocking**; so is a match count ≠ 1.
      Record results in the commit.
- [ ] **7.3** F-96 has a second half — "land it with the phase report if that is
      still owed". `ls design/agent-reports/ | grep -c b2a-ii` is **11**, all
      lens reports and **no phase report**, so the runner's row table has no
      B2a-ii source document. Either write that phase report or amend F-96 to
      drop the requirement with a reason. Do not leave it silently unmet.

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
- `scripts/plan-mutation-anchors.py` proves **every mutation-table anchor matches
  exactly once, in the file its row names** — 17 unique, 0 bad, 1 unresolved
  (fork-file anchors: `unlockMnemonicParsedHook(m)` does not exist until step
  6.0 adds it, and `defer clear(m)` already exists at `unlock_session.go:250` but
  the plan quotes that file only as a fragment, so absence from the quote proves
  nothing).
  **Version 1 of this script had the exact false-PASS it exists to prevent** —
  it graded the longest backticked span, which for one row was a parenthetical
  *context* note rather than the anchor, and reported `ok` for a token matching
  twice; and it searched every fence concatenated, so it never compared an
  anchor against the file its row named. Both are fixed, and the rule is now
  structural: **the anchor cell must hold exactly one code span**, context goes
  in the "killed by" column. Committed as a script rather
  than left as discipline, because the claim "each row names a unique anchor"
  had already been made and was already false: `break` matches **5** sites in
  `run_flow.go` alone, one of them the `pl.NextChunk()` chunk walk where
  substituting `return` silently truncates every frame. The one anchor that
  could not be made unique by wording — `a.idle.start = now`, which occurs
  twice — was made unique **in the code**, with a trailing comment that exists
  for exactly that purpose.
- `scripts/plan-build-gate-go.sh` type-checks **six** whole-file blocks:
  `gui/run_flow.go` (224 lines), `gui/run_harness_test.go` (231),
  `gui/wipe_warning.go` (61), `gui/wipe_guard.go` (52),
  `gui/unlock_mnemonic_seam.go` (13), `gui/op/buffer_len.go` (9).

**The gate has one blind spot in this plan, and it is named here rather than
discovered by a reviewer.** `Context` gains `wipe *wipeGuard` and
`keepAwake bool` — fields added to an existing struct, which cannot be expressed
as whole files, so `plan-build-gate-go.sh` reports `ctx.wipe undefined` and
`ctx.keepAwake undefined` and TIER 1 fails. That failure is the gate being
honest, not the plan being broken.

**A SECOND blind spot, and it is the more dangerous one: TIER 1 is ADDITIVE.**
The gate copies the fork and *adds* the plan's files; it never *removes* the old
`Run` body. So in the gate's scratch tree `gui.go` still uses `saver` and still
compiles — the gate reports OK on a configuration **that cannot be the shipped
one**. A gate that hides its own blind spot is worse than no gate, so:

**The controller applied the fragments by hand AND modelled the shipped
configuration — Run's body replaced by the one-line delegation, the `saver`
import removed — and type-checked that:**

```
$ # fork copy + the plan's 6 whole files
$ #   + Context.wipe, Context.keepAwake, KeepAwake(), Reset()'s clear
$ #   + Run's body REPLACED by `return runWithFlow(pl, version, uiFlow, nil)`
$ #   + the orphaned "seedhammer.com/gui/saver" import deleted
$ CGO_ENABLED=0 go build ./gui/ ./gui/op/   →  BUILD OK (shipped config)
$ CGO_ENABLED=0 go vet  ./gui/ ./gui/op/    →  freetext_sizeproof_golden_test.go:111:13
                                               gui/op/draw_test.go:176:24
                                               (both: testing.ArtifactDir requires go1.26)
$ gofmt -l <all six>                        →  empty
```

Both vet lines are the **pre-existing baseline** recorded in "The green
criterion" above — byte-identical, and the only findings. **So every line of Go
in this plan type-checks, including the moved `Run` body and the harness, in the
configuration that actually ships rather than merely in the gate's additive
one.**

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

### The release tag's precondition set — the ONE place this list lives

Previously stated three different ways in this plan, all incomplete, and a bare
"Task 9" that belongs to a **different plan**. Every item below is checkable:

- [ ] B2b Tasks 1–8 green against the full green criterion above
- [ ] B2b whole-diff review at **0C/0I**, and merged
- [ ] **B2a-ii's Task 9** — the in-situ RP2350B KDF rate, closing SPEC §7.1 and
      the last open item in §12.1. *(It is `IMPLEMENTATION_PLAN_…_phaseB2a_ii.md`
      Task 9, steps 9.1–9.8 — not a task of this plan, which ends at Task 8.)*
      **Task 8.1 already unlocks on the real machine, so recording the derivation
      time in Task 8.5 closes §7.1 for free and makes the two trips one.**
- [ ] **F-85** — the §2.2 amendment
- [ ] **F-92** — `tinygo test` cannot build `seal` at all, so the TinyGo wipe
      caveat this whole feature rests on has never run on the target toolchain
- [ ] **F-98** — two citations in the GREEN spec do not resolve
- [ ] **SPEC §11.5's "confirm firmware reflash preserves the blob"** — filed as
      **F-100** by the residue sweep; owned by nobody until then. B1's hardware
      run covered four things and this was not among them; its closest statement
      is the converse ("only the 64 KB payload region was cleared"). §11.5 also
      specifies booting on **PD power**, which neither Task 8 nor B2a-ii Task 9
      names.
- [ ] **Push both repos via `ci/staging`** — neither has been pushed this cycle
      (`origin/master..master` and `origin/main..HEAD` are both well ahead)

If an item moves, it moves *here*. The "does NOT cover" list above defers work;
this list gates the tag, and the two must not drift apart.
