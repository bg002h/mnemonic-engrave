# B2b R0 round 0 — design/correctness lens (opus, verbatim)

**Artifact:** `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md` at `7678b5e`
**Fork:** `seedhammer` at `a01b666`
**Brief:** one question — *is the unwind mechanism this plan specifies correct, and
does it faithfully implement §10.2.4?* Test adequacy explicitly out of scope
(reviewed separately). Told not to re-derive the type-check or the citations, and
forbidden from proposing alternative architectures — the operator-approved seam
was to be critiqued as specified, not replaced.

Persisted verbatim before folding. HTML entities from the transport layer
(`&amp;&amp;`, `&gt;`, `&lt;`) are rendered back to `&&`, `>` and `<`; nothing else
is altered.

---

Repo left clean (all measurement work was done in a scratch copy, not in `/scratch/code/shibboleth/seedhammer`).

---

## 1. VERDICT

**1C / 2I**

---

## 2. CRITICAL

### C1 — `wipeWarningOp` builds into `ctx.B`, which is not `Reset` between warning ticks: ~228 KB of unbounded append into a per-frame buffer, mid-frame, with the seed's rendered ops copied forward on every reallocation

**Plan location:** Task 4, `gui/wipe_warning.go` (`wipeWarningOp` takes `*Context` and appends via `&ctx.B`), consumed by `gui/run_flow.go`'s warning branch:

```go
if armed && now.Sub(warnAt) >= 0 {
    draw(wipeWarningOp(ctx, &descriptorTheme, pl.DisplaySize(), wipeAt.Sub(now)))
    ctx.WakeupAt(now.Add(time.Second))
    continue
}
```

**What is wrong.** `Context.Frame` is (`gui/gui.go:77`):

```go
func (c *Context) Frame(op op.Op) {
	if f := c.FrameCallback; f != nil {
		f(op)
	}
	c.B.Reset()
}
```

`c.B.Reset()` runs **after** the callback. Run's event loop executes *inside* that callback, so for the entire time the flow is parked, `ctx.B` is never reset. The warning branch appends a fresh title + wrapped body + background op to that same live buffer once per second, `continue`s, and appends again. `op.Buffer` is append-only (`args []uint32`, `refs []any`; `Reset` only truncates), so the warning accumulates for the full 30 s.

**Measured** (fork at `a01b666` + the plan's `gui/wipe_warning.go` verbatim, `ctx := NewContext(newPlatform())`, 480×320, `descriptorTheme`):

| | `args` | `refs` |
| --- | --- | --- |
| after 1 warning frame | 1,038 (4,152 B) | 433 |
| after 30 warning frames | 31,077 (**124,308 B**) | 12,963 |
| capacity reached | 32,768 (**131,072 B**) | 14,336 |

`refs` is `[]any` = 8 B on the 32-bit target, so on device one warning frame costs **7,616 B** and the 30-second warning leaves **228,012 B live in `ctx.B`, with 245,760 B of capacity reserved**, against ~440 KB of heap (520 KB SRAM − the plan's own 60,584 B static baseline − stacks). During the last `append` doubling both the old and new arrays are live: a transient ≈370 KB. This is concurrent with `p.Public`, `plate.Spline`, and the KDF-era allocations. A TinyGo heap-exhaustion panic on a watchdog-less device is a brick, and it fires on precisely the path B2b adds and nowhere else.

Second half of the same defect, and it is funds-relevant independently of whether the heap survives: those reallocations **copy the whole buffer**, including the ops of the frame the flow was parked on. When the wipe fires on `SeedScreen.Confirm` — an armed walk-away state, and the one the plan's "privacy blanking" paragraph is about — that frame is the twelve words. The buffer holds one `widget.Label` per word; the glyph bitmaps are public, but *which* glyph refs appear in *what order* is the seed, which this repo already treats as seed-equivalent (`gui/unlock_session.go:222`: "their SELECTION and ORDER are the seed"). Growth from 1 KB to 128 KB is ~7 doublings, so the rendered seed is memcpy'd into ~7 successively larger arrays and **none of the abandoned copies is zeroed**. Task 3 then drops the whole `Context` for a fresh one, so `Buffer.Reset`'s `clear(b.refs)` never runs on the final copy either.

**Failure trace.** Vector F (or any mnemonic record). Unlock → `unlockSecretPlate` → Cut → `unlockEngraveMnemonic` → `ss.Confirm` draws the 12 words → operator walks away. `armed()` is true (no job yet). At 3:00 Run enters the warning branch and appends 7.6 KB/s into the buffer that currently holds the rendered seed. At 3:30 the wipe fires; the ~245 KB buffer and its ~7 unzeroed predecessor arrays are abandoned as garbage.

**Minimal fix.** Give the warning its own buffer instead of the flow's. In `runWithFlow`, add `warnBuf op.Buffer` to the anonymous `a` struct; call `a.warnBuf.Reset()` immediately before building the warning; change `wipeWarningOp` to take `(*op.Buffer, Styles, *Colors, image.Point, time.Duration)` and inline `layoutTitlef`'s two lines (it only needs `ctx.Styles.title` and `&ctx.B`) rather than calling `layoutTitle(ctx, …)`. That bounds the cost at one frame and touches nothing else.

The one-liner alternative — `ctx.B.Reset()` immediately before `wipeWarningOp` — also bounds the growth and additionally scrubs `refs`, but it silently invalidates the `content` op the range body is still holding (its `ops{start,end}` would then index the warning). Nothing re-draws `content` in the plan's code today, so it works, but it is a landmine for the next edit. Prefer the dedicated buffer; if the one-liner is chosen, say so in a comment at the `for content := range it` head.

Also fold in: before `continue`-ing the session loop on a wipe, `ctx.B.Reset()` so `clear(b.refs)` runs on the frame that was on screen. One line, and it is the only scrubbing the abandoned Context gets.

---

## 3. IMPORTANT

### I1 — `a.idle.active` is left latched true across the `armed` false→true edge: the saver stops *drawing* but does not stop *swallowing input*, so the first tap after any cut longer than 3 minutes is silently dropped on a live-looking screen

**Plan location:** Task 4, `gui/run_flow.go`, the armed-edge block:

```go
armed := ctx.wipe.armed()
if armed != a.wipe.armed {
    a.wipe.armed = armed
    if armed {
        a.wipe.origin = now      // <- only origin is reset
    }
}
```

and the saver gate `if a.idle.active && !armed { … }`.

**What is wrong.** The plan correctly identifies that while armed the saver must not paint over the warning, and gates the *draw*. But `a.idle.active` also gates event routing, twenty lines earlier and ungated:

```go
if !a.idle.active {
    ctx.Router.Events(d, evts...)
}
```

During a secret cut, `armed()` is false (job running), the saver activates normally at 3:00 and stays active for the remaining ~18 minutes, parking the flow on the `continue` branch. At cut end `Status()` flips to `engraveDone`, `armed` goes false→true, `a.wipe.origin = now` — but `a.idle.start` is still ~21 minutes stale, so `idle` recomputes to `true` and `a.idle.active` **stays latched true**. The `&& !armed` gate now suppresses `a.idle.state.Draw(pl)`, so the flow gets control back and repaints the plate-done screen. The operator sees a normal, live screen with its nav icons — and every event is dropped by `if !a.idle.active` until one arrives to clear the latch.

**Failure trace.** Cut a secret plate (≥3 min; every real plate is ~21 min). Walk away. Return. Screen shows the finished engrave screen with Back/OK. Tap OK → **nothing happens** (the press is dropped; the latch clears on that same tick, but a click needs the press, so the release alone registers nothing). Tap OK again → works. Today the operator sees the screensaver during that window and reads the lost tap as "waking the screen"; with B2b the screen looks live and the machine looks broken, at the end of a funds-critical operation.

A second, smaller instance of the same omission: the session restart resets `a.idle.start` and `a.wipe.origin` but not `a.idle.active`, so the first tick of a post-wipe session also has the latch set.

**Minimal fix.** Two lines. In the armed false→true branch, reset the saver clock alongside the wipe clock:

```go
if armed {
    a.wipe.origin = now
    a.idle.start = now
    a.idle.active = false
}
```

and add `a.idle.active = false` beside `a.wipe.armed = false` at the top of the session loop. Setting `a.idle.start = now` also makes `idleWakeup == warnAt` again, which removes the only window in which the two clocks disagree — worth stating in the comment, because that divergence is what created this bug.

Mutation row to add to Task 4.3: *drop `a.idle.active = false` from the armed edge* → must be killed by a test that runs a job past `idleTimeout`, ends it, and asserts a tap on the plate-done screen is delivered to the flow.

### I2 — Task 5 (F-93) changes `Run`'s idle arithmetic with no code, no gate coverage, and the harness as specified makes its own test a false PASS

**Plan location:** Task 5, in full: "`Context` gains `KeepAwake()`, cleared by `Reset`; `unlockDerive` calls it each slice." Plus step 5.1's assertion and mutant.

**What is wrong, in three parts.**

1. **No `Run`-side code and no gate.** Task 4 declares that `gui/run_flow.go`'s "FINAL form — after Tasks 3 and 4 — is given in full … and that is the copy the build gate type-checks", and the gate section pins it at 174 lines. Task 5 then edits that same file again — it must, since `KeepAwake` is meaningless unless Run consumes it — and that edit exists nowhere in the plan and is covered by neither gate. The consumption point is also load-bearing and unstated: `ctx.Reset()` is what clears the flag, and it runs *before* the idle computation in the loop, so a naive "clear in `Reset`" loses the flag every tick. Whether `KeepAwake` refreshes `a.idle.start` only, or `a.wipe.origin` too, is likewise unstated — and the second reading would let a screen postpone the §10.2.4 wipe, which is exactly what the section forbids.

2. **The specified harness cannot drive this test.** `deadlinePlatform.AppendEvents` advances the bubble clock only via `time.Sleep(d)` when `d > 0`. `unlockDerive` calls `ctx.WakeupAt(time.Now())` **before every `ctx.Frame`** (`gui/unlock_kdf.go:295`, and the comment there explains why the order is load-bearing). So on every derivation frame `d <= 0`, `AppendEvents` returns immediately, no goroutine durably blocks, and a `synctest` bubble's clock **never advances**.

3. **Which makes step 5.1 a false PASS, not a hang.** With the clock frozen, `now.Sub(a.idle.start.Add(idleTimeout)) >= 0` is never true, so the saver never activates — the assertion "a derivation longer than `idleTimeout` does not trip the saver" passes, **and so does its own mutant** with the `KeepAwake` call deleted. This is the exact failure mode this plan's fact 4 and the project's mutation discipline exist to prevent, in the one task that has no code to review.

**Minimal fix.** Either (a) pull Task 5 out of B2b and file it against a phase that can specify and gate the `Run` edit, or (b) give Task 5 the same treatment as Tasks 3–4: a whole-file `gui/run_flow.go` block including the `KeepAwake` consumption, and a test that drives the clock through a seam the derivation does not defeat — e.g. a `deadlinePlatform` that sleeps a floor (`if d <= 0 { d = time.Millisecond }`) so the bubble advances even on an already-expired deadline, with the floor's effect on Task 1.3's saver test re-checked. Whichever is chosen, step 5.1's mutant must be run and shown to be *killed*, not merely listed.

---

## 4. MINOR / NIT

- **Minor.** `armed()`'s doc says `engraveStopping` counts as running "because Stop() is synchronous in the flow but the worker is still moving", but `defer func() { g.job = nil }()` disarms on `Engrave`'s return — and `Engrave` *can* return in `engraveStopping` (`gui/gui.go:2703-2709`: Back while running calls `Stop()`, a second Back then takes `st.State != engraveRunning` → `break frames`). The `engraveStopping` case is therefore unreachable via that route. No practical hazard — the false→true edge grants a fresh 3:00 and the worker stops long before — but the comment claims a guarantee the code does not provide.
- **Minor.** `deadlinePlatform.AppendEvents` ignores `p.wakeups`, so it does not model `Platform.AppendEvents`'s contract (`cmd/controller/platform_sh2.go:384` returns on a wakeup). Any Task 2 / Task 4 test that drives a real `engraveJob` will not see the job-completion `pl.Wakeup()` and must rely on `EngraveScreen`'s 500 ms poll. Workable, but the divergence should be named in the harness comment or closed with a `select`.
- **Minor.** `runSession` hard-codes `onDraw`, so Task 4.1's required "tap during the warning → window resets, no wipe" test cannot be written with it: `p.tap()` from the test goroutine while `Run` blocks is a data race, and the flow cannot tap while parked. The only same-goroutine injection point is `onDraw` (called from `draw`, including the warning draw). Add an `onDraw` parameter to `runSession`.
- **Minor.** `ctx.B` is abandoned rather than `Reset` on the wipe path (fresh `Context`), so `Buffer.Reset`'s `clear(b.refs)` never runs on the last frame drawn — which on the `SeedScreen` path is the twelve words. Covered by C1's fix list; noted separately in case C1 is fixed only with a dedicated warning buffer.
- **Minor.** §10.2.4 row 1's "**3 min**, 30 s warning" is genuinely ambiguous between warn@3:00/wipe@3:30 (the plan's reading, and what Task 8.1 would bless) and warn@2:30/wipe@3:00. The plan commits explicitly, which is right, but the spec text does not fix it — worth one amending sentence in §10.2.4 so the hardware pass cannot ratify an unstated choice.
- **Nit.** Task 3's prose describes the session-loop tail as `if wiping { continue }`; Task 4's gated code is `if !wiping { return }`. Same behaviour, but the Task 3.3 mutation row "the session loop's `continue` removed" names a token that does not exist in the file.
- **Nit.** Task 3.3's mutation row "`wiping` never cleared" is untestable as stated: `wiping := false` is declared inside the session loop, so it is a fresh variable per session and cannot fail to clear.
- **Nit.** Task 1's prose says "`testPlatform.AppendEvents` **must** honour the deadline"; 1b correctly adds a wrapper type instead and leaves `testPlatform` alone (which is what makes step 1.1's "`go test ./gui/` unchanged" achievable). Align the prose.

---

## 5. WHAT I VERIFIED AND FOUND CORRECT

**The unwind is sound.** I walked the real stack that exists at wipe time — `runWithFlow` → `it` → `uiFlow` (`for !ctx.Done`, gui.go:1595) → `unlockPayloadFlow` (straight-line) → `unlockSecretSession` → `unlockSecretPlate` → {`ChoiceScreen.Choose` (gui.go:1446 `for !ctx.Done`), `unlockEngraveCodex32`/`Mnemonic` → `showModal` (slip39_polish.go:25 `for !ctx.Done`), `SeedScreen.Confirm` (gui.go:2336 `for !ctx.Done`, plus its nested discard-confirm and `showErr` loops, both `!ctx.Done`), `EngraveScreen.Engrave` (gui.go:2701 and its nested `ConfirmDelay` loop, both `!ctx.Done`)}. Every nested `for {}` on that path is an event-drain loop that terminates on `!ok`. **No loop on a real secret-session call path can hang or spin after `ctx.Done`.** After the unwind, `unlockPlatesOrNotice` → `showNotice`/`unlockPlateListFlow` (unlock_platelist.go:103) both exit immediately, and `uiFlow` never re-enters `StartScreen.Flow`, so no NFC goroutine is leaked or re-opened.

**The unwind IS the wipe.** `unlockSecretSession`'s `for _, i := range at` has no `ctx.Done` test, and that is correct here rather than a defect: each remaining `unlockSecretPlate` runs, `Choose` returns `(0,false)` without drawing, and the deferred `p.WipeSecretAt(i)` fires — so *every* remaining secret is zeroed, not just the one on screen. Then `unlockPayloadFlow`'s `defer p.Wipe()` and `defer func(){ clear(blob) }()`, and `unlockEngraveMnemonic`'s `defer clear(m)`. No secret state on this path sits outside a defer.

**The discard guard is correctly placed and required.** Confirmed both fall-through `ctx.Frame` sites (gui.go:2455-2460 in `Confirm`, gui.go:2749-2758 in `Engrave`). With `if wiping { continue }` first in the range body, `continue` makes `yield` return `true`, so `ctx.Frame` returns normally, no range-over-func panic, no frame drawn, no event processed, no `AppendEvents` — the unwind is not blocked on input and does not spin. `break` (not `return`) is right, and the pre-existing `if ctx.Done || !yield()` return is correctly left alone.

**`wipeGuard` mechanics.** `NewEngraveScreen` (gui.go:2682) constructs the job eagerly via `newEngraverJob`, so `g.job = scr.job` captures a non-nil pointer — the "captured nil, armed through the whole cut" failure I looked for does not exist. `engraveIdle` (hold-to-start) and `engraveDone`/`Stopped`/`Failed` (plate-done) are armed, matching §10.2.4's amendment. The `job` is never left non-nil: the defer is registered in the same function that sets it, and the early-return paths (`showError`) register nothing. The extra `Status()` call per tick is safe: same goroutine as the flow, `e.status` is accumulative, and `state==engraveRunning && errs==nil` (the only path on which `Status()` would call `Start()` and restart a job) is unreachable in production — the only writers of `State` are `Stop`, `Start` and `Status` itself, and every one that nils `errs` simultaneously leaves `Running`.

**Bracket fidelity to §10.2.4.** `UnlockWithKey` assigns `p.Secret = admitted` as its last statement before `return nil`, so no error path populates `p.Secret`. The gap between that assignment and `unlockSecretSession`'s guard install (`unlockAttemptOnce` defers, `unlockSealedFlow`'s `clear(m)` + `return true`, `unlockPayloadFlow`'s `clear(blob); blob = nil`) draws no frame, and the trailing gap is likewise straight-line. The plan's "the gaps at its edges are frame-free straight-line code" is true.

**Timer arithmetic.** `a.wipe.origin`'s two sources are both real: `len(evts) > 0` is a true physical-input signal (`Platform.Wakeup` returns `evts` unchanged at platform_sh2.go:384, so an engraver wakeup does not refresh it; touches append a `PointerEvent`), and the armed edge. `ctx.Wakeup` is non-zero on every path that reaches `AppendEvents` while armed, so the machine neither sleeps past its deadline nor spins. The `EventRouter.Reset` → `ctx.Wakeup = time.Now()` hot-spin path I chased does not occur during the warning: `Reset` truncates `r.filters` unconditionally, the parked flow registers none, so every subsequent `Reset` discards and returns false. Tap-vs-wipe races resolve in the operator's favour — `if len(evts) > 0 { origin = now }` precedes the `armed && now >= wipeAt` test — and a hold-to-start in progress refreshes `origin` continuously, so no engrave can start on the tick a wipe fires (`for !ctx.Done` is re-tested before `s.job.Start()`).

**Warning presentation.** `wipeWarningOp` covers the screen: measured at 480×320, `titleRect = (116,8)-(364,44)`, `bodySz = (455,113)`, body occupies y∈[52,165] of 320 — no overflow, no horizontal clip, and `ExtractText` over the composed op returns only the warning's own text, so it does blank the seed screen underneath. `op.Layer`'s background-last ordering is right. The `&& !armed` saver gate is genuinely needed (not belt-and-braces): it is the only thing preventing the saver from painting over the warning in the origin ≠ `idle.start` window.

**Session restart.** `wiping` is per-session by construction; `a.wipe.armed` is reset; `a.mask`, `d` and `stats` correctly survive; `d` is refilled by `draw(content)` before the new session's first `Router.Events`; `a.idle.state` self-resets on the next idle transition. The fresh-`Context`-rather-than-`Reset` argument is correct — `EventRouter.Reset` (event.go:281-294) leaves `r.pointer.pressedTag`/`pressed` untouched. The only omission is `a.idle.active` (I1).

**Behaviour-neutrality of Tasks 1–3.** With `armed == false` the event loop is behaviourally identical to today line for line, including the saver covering a running cut. The move's only semantic deltas are that `HardwareVersion()`/`Features()` are read at `Run` entry instead of at the first frame, and `layoutTime` is sampled one statement earlier — both debug-only or inconsequential.
