# RECON — B2b's idle-wipe surface (§10.2.4)

Read-only recon over `seedhammer` `main` @ `a01b666` (B2a-i + B2a-ii merged) and
`mnemonic-engrave/design/`. Every line number below was read at that commit.

**Scope note.** This is the factual basis for a plan, not a plan. Where a
question is a design choice rather than a fact it is in §OPEN DESIGN CHOICES and
no decision is invented.

---

## 0. Citation drift — corrected

| cited as | actual at `a01b666` |
| --- | --- |
| SPEC §10.2.4 (×2): `idleTimeout` at `gui/gui.go:2801` | **`gui/gui.go:2932`** |
| CONTINUITY 2026-08-08 §5: `idleTimeout` at `gui/gui.go:2801` | **`gui/gui.go:2932`** |
| CONTINUITY 2026-08-08b §10: `idleTimeout` at `gui/gui.go:2879` | **`gui/gui.go:2932`** |
| CONTINUITY 2026-08-08b §9: `a.idle.start` at `gui/gui.go:2884-2891` | **`gui/gui.go:2937-2945`** |
| CONTINUITY 2026-08-08b §9: saver branch `gui/gui.go:2954-2959` | **`gui/gui.go:3007-3013`** |
| CONTINUITY 2026-08-08 §5: `AppendEvents` at `platform_sh2.go:368` | **`cmd/controller/platform_sh2.go:369`** |
| `unlock_session.go:191` comment: Back-while-running at `gui/gui.go:2651-2656`; `unlock_session_test.go:367` says `2648-2654` | **`gui/gui.go:2704-2710`** (two in-tree comments, two different stale numbers, neither right) |

`EventRouter.Reset` at `gui/event.go:281-294` and `Event` at `gui/event.go:105-109`
(CONTINUITY §9) are **correct**.

---

## 1. `Run`'s frame loop and its idle state

`Run` is `gui/gui.go:2934-3020`. The idle state is a field of an **anonymous
struct local to the closure**:

```go
2934  func Run(pl Platform, version string) func(yield func() bool) {
2935      return func(yield func() bool) {
2936          ctx := NewContext(pl)
2937          a := struct {
2938              mask *image.Alpha
2939              idle struct {
2940                  start  time.Time
2941                  active bool
2942                  state  saver.State
2943              }
2944          }{}
2945          a.idle.start = time.Now()
```

The whole inner event loop, verbatim (`gui/gui.go:2985-3016`):

```go
2985              for {
2986                  if ctx.Done || !yield() {
2987                      return
2988                  }
2989                  wakeup := ctx.Wakeup
2990                  evts = pl.AppendEvents(wakeup, evts[:0])
2991                  now := time.Now()
2992                  if len(evts) > 0 {
2993                      a.idle.start = now
2994                  }
2995                  ctx.Reset()
2996                  if !a.idle.active {
2997                      ctx.Router.Events(d, evts...)
2998                  }
2999                  idleWakeup := a.idle.start.Add(idleTimeout)
3000                  idle := now.Sub(idleWakeup) >= 0
3001                  if a.idle.active != idle {
3002                      a.idle.active = idle
3003                      if idle {
3004                          a.idle.state = saver.State{}
3005                      }
3006                  }
3007                  if a.idle.active {
3008                      a.idle.state.Draw(pl)
3009                      // Throttle screen saver speed.
3010                      const minFrameTime = 40 * time.Millisecond
3011                      ctx.WakeupAt(now.Add(minFrameTime))
3012                      continue
3013                  }
3014                  ctx.WakeupAt(idleWakeup)
3015                  break
3016              }
```

**What refreshes `a.idle.start`: `len(evts) > 0`, and nothing else** (`:2992-2993`).
`evts` comes from `Platform.AppendEvents`. On the real machine
(`cmd/controller/platform_sh2.go:369-396`) that returns a non-empty slice on
exactly two sources — a touch (`:373-375`, `:389-393`) and the debug UART
(`:386-387`). The two other wake reasons return the slice **unchanged**:

```go
382              case <-p.timer.C:
383                  return evts
384              case <-p.wakeups:
385                  return evts
```

So `a.idle.start` is a true last-physical-input time (CONTINUITY 2026-08-08 §5's
claim — **verified**), and in particular `Platform.Wakeup()` does **not** refresh
it.

**Why a flow cannot reach it.** `a` is a local of the closure `Run` returns.
`Context` (`gui/gui.go:64-73`) has fields `Platform, Styles, Wakeup, Done,
FrameCallback, B, Router` — no idle field and no back-pointer to `a`. `Event`
(`gui/event.go:105-109`) is `{typ eventKind; data [3]uint32; refs [2]any}` —
**no timestamp**. `grep -n 'a\.idle'` over the tree returns lines 2945, 2993,
2996, 2999, 3001, 3002, 3004, 3007, 3008 — all inside `Run`.

**Is there ANY existing path by which a flow can influence or observe it?**

- *Observe:* no. Not even indirectly: a flow blocked inside `ctx.Frame` gets no
  control, so it cannot even time its own frame gaps while the saver is up.
- *Influence, sanctioned:* no. `ctx.WakeupAt` (`gui/gui.go:90-94`) only lowers
  the `AppendEvents` deadline; `ctx.Platform.Wakeup()` (used by
  `ConfirmDelay.Progress`, `gui/gui.go:304`) returns zero events, per the quote
  above. `ctx.Router.Events(...)` is exported and callable from a flow, but it
  appends to the router's own queue — `a.idle.start` is computed from
  `AppendEvents`' return value, not from the router.
- *Influence, unsanctioned and real:* `Context.Platform` is an exported field
  (`gui/gui.go:65`). A flow inside package `gui` can assign
  `ctx.Platform = wrapper{pl}` whose `AppendEvents` appends one synthetic event.
  `EventRouter.Reset` (`gui/event.go:281-294`) discards any event no filter
  claimed, so an inert event would refresh `a.idle.start` invisibly to widgets.
  **This works and it is the only flow-only lever on the saver.** Its cost is in
  §OPEN DESIGN CHOICES.

**Test coverage of this loop: zero.** No test in the repo calls `gui.Run`; the
only two callers are `cmd/controller/main.go:34` and `cmd/emu/main.go:33`. Tests
enter one level down at `uiFlow` (`gui/unlock_program_test.go:179` etc.) through
`runUI`/`runUITouch` (`gui/gui_test.go:503-514`), which install their own
`FrameCallback` and never execute lines 2985-3016.

---

## 2. How would `Run` learn that a secret is resident?

`Run` has `pl Platform` and `ctx *Context`. It has no payload and must not get
one — `seal.Payload` is created inside `unlockPayloadFlow` (`gui/unlock_flow.go:64`)
and destroyed by its `defer p.Wipe()` (`:85`).

**What the codebase's own convention says.** Three candidate seams exist and the
precedents are not evenly matched:

| seam | precedent in-tree | verdict |
| --- | --- | --- |
| **`Context` data field** | `ctx.Wakeup` (`gui/gui.go:67`) is written by flows via `WakeupAt` (`:90-94`) and read by `Run` at `:2989`. `ctx.Done` (`:68`) is written by `Run` at `:2949` and read by ~58 flow loops. | **In convention.** This is exactly the flow↔`Run` channel, in both directions, and it needs no interface change. |
| **`Context` func field** | `ctx.FrameCallback func(op.Op)` (`gui/gui.go:69`), installed by `Run` at `:2948` and by every test harness. | **In convention**, and the only shape in which "absent" has a natural encoding (`nil`). |
| **`Platform` method** | `Platform` (`gui/gui.go:2889-2912`) is twelve methods, every one a hardware/OS capability: `LockBoot, AppendEvents, Wakeup, Engraver, NFCReader, PayloadReader, EngraverParams, DisplaySize, Dirty, NextChunk, Features, HardwareVersion`. | **Out of convention.** Residency is GUI state; this would make `cmd/emu`, `cmd/controller` and `testPlatform` all implement a fact the GUI owns. |
| **package-level `var`** | `unlockSecretHook`, `unlockMnemonicHook`, `unlockPassphraseHook`, `unlockPassphraseWordsHook`, `unlockKeyHook`, `newDeriver` | **Out of convention for production state.** Every one is documented "nil in production" / test-only (e.g. `gui/unlock_session.go:35-40`), and a package var is not per-`Context`. |

The minimal seam is therefore a `Context` field. `unlockPayloadFlow` already has
the one line where the predicate becomes available and the one `defer` where it
must be withdrawn (`gui/unlock_flow.go:85` `defer p.Wipe()`), so the set/clear
pair has an obvious, already-tested home.

Note a func field (`ctx.SecretsResident func() bool`) and a data field
(`ctx.SecretResident bool`, re-asserted per frame) differ in failure mode: with
the func field, a flow that forgets to clear it leaves the timer armed
(fail-safe); with a per-frame data field, a flow that forgets to assert it
disarms the timer (fail-open). That is a choice, not a fact — see §OPEN DESIGN
CHOICES.

---

## 3. How would `Run` learn that an engrave is in progress?

**It has no visibility today.** `EngraveScreen` (`gui/gui.go:2690-2695`) and
`engraveJob` (`gui/engraver.go:15-30`) are constructed inside flows
(`NewEngraveScreen`, `gui/gui.go:2682-2688`) and referenced from nowhere else.
`Run` never mentions either; `grep -n 'engrave' gui/gui.go` returns no hit
between lines 2932 and 3020.

The one datum that crosses into `Run`'s world is a wake, not a state:

```go
gui/engraver.go:109      go func() {
gui/engraver.go:110          defer e.pl.Wakeup()
gui/engraver.go:111          errs <- e.runEngraving(quit, progress)
gui/engraver.go:112      }()
```

and `Wakeup()` carries no events (§1), so `Run` cannot even infer "something
finished" from it.

**Measured consequence, and it is existing behaviour B2b must not regress.**
Because a running engrave produces no events, `a.idle.start` is not refreshed
during a cut, so **the screensaver takes the screen ~3 minutes into a ~21-minute
plate today.** The cut is unaffected — `runEngraving` is on its own goroutine
(`gui/engraver.go:109`) and `reportProgress` (`:197-207`) folds unread progress
into a running total rather than dropping it — but `EngraveScreen.Engrave` is
parked inside `ctx.Frame`, the MM:SS countdown stops, and when the job finishes
`defer e.pl.Wakeup()` fires with zero events, so **the completion screen stays
behind the saver until the operator touches the panel.**

---

## 4. How can a timer make a flow unwind?

### 4.1 What `ctx.Done` actually is

`ctx.Done` is set in exactly one production place:

```go
gui/gui.go:2948          ctx.FrameCallback = func(op op.Op) {
gui/gui.go:2949              ctx.Done = ctx.Done || !yield(op)
gui/gui.go:2950          }
```

`yield` here is the **range-over-func body** of `for content := range it`
(`:2961`). So `ctx.Done` goes true exactly when that body executes a `return` —
which happens at `:2986-2988` (`if ctx.Done || !yield() { return }`), where the
second `yield` is `Run`'s own outer yield to `for range gui.Run(...)`.

Mechanically: `it` is not a goroutine. `uiFlow` runs on the same stack; a flow
calling `ctx.Frame(op)` (`gui/gui.go:75-80`) calls `FrameCallback` → `yield(op)`
→ **the range body at `:2962-3017` runs to completion inside `ctx.Frame`** →
`ctx.Frame` returns → the flow continues. A `return` in the body makes the body's
yield return `false`, `ctx.Done` sticks true, and every `for !ctx.Done` loop
returns on its next condition test.

### 4.2 Does every loop in the unlock path honour it?

Machine-counted: **58** `for !ctx.Done` sites across **23** non-test files in
`gui/`. The complete secret-resident stack and its loops:

| frame | file:line | loop | returns on `ctx.Done`? |
| --- | --- | --- | --- |
| `uiFlow` | `gui/gui.go:1595` | `for !ctx.Done` | yes |
| `unlockPayloadFlow` | `gui/unlock_flow.go:26` | no loop; straight-line | yes |
| `showNotice`/`showError` → `showModal` | `gui/slip39_polish.go:25` | `for !ctx.Done` | yes |
| `unlockWarnUnauthenticated` | `gui/unlock_flow.go:231` | `for !ctx.Done` | yes, returns `false` |
| `unlockSealedFlow` | `gui/unlock_kdf.go:368` | `for !ctx.Done` | yes, returns `false` |
| `unlockPassphraseFlow` | `gui/unlock_kdf.go:123` | `for !ctx.Done` | yes |
| `inputWordsFlow` | `gui/gui.go:696` | `for !ctx.Done` | yes |
| `unlockDerive` | `gui/unlock_kdf.go:232` | `for !ctx.Done` | yes, returns `(nil,false)` |
| `unlockSecretPlate` → `ChoiceScreen.Choose` | `gui/gui.go:1448` | `for !ctx.Done` | yes, returns `(0,false)` at `:1496` → treated as Skip → `WipeSecretAt` |
| `SeedScreen.Confirm` | `gui/gui.go:2336` | `for !ctx.Done` (+ inner `:2352`, `:2380`) | yes, all three |
| `EngraveScreen.Engrave` | `gui/gui.go:2703` (+ inner `:2727`) | `for !ctx.Done` | yes; `defer s.job.Stop()` (`:2698`) **halts the cut** |
| `unlockPlateListFlow` | `gui/unlock_platelist.go:103` | `for !ctx.Done` | yes |
| `unlockEngraveFlow` | `gui/unlock_platelist.go:228` | **bare `for {`** | yes, but only *because* `cs.Choose` returns `!ok` → `return false` |

**No loop in this path ignores `ctx.Done`, and none blocks on a channel.** The
only blocking receives in `gui/` are `gui/mk1_inspect.go:169` and
`gui/verify_address.go:85` (`<-closed`), both in NFC flows outside the unlock
path. `engraveJob.Status()` (`gui/engraver.go:126-151`) uses non-blocking
`select`s only.

This is already exercised: `TestUnlockFlowWipesOnTheCtxDoneAndErrorExits`
(`gui/unlock_wipe_test.go:235`) sets `ctx.Done = true` mid-flow
(`gui/unlock_plates_test.go:360`) and asserts the record buffers are zero.

### 4.3 So why is `ctx.Done` not the answer

Two reasons, both hard.

**(a) `ctx.Done` terminates the machine's UI.** `uiFlow`'s own loop is
`for !ctx.Done` (`gui/gui.go:1595`), so the unwind does not stop at the main
menu — it exits `uiFlow`, ends the `for content := range it` range, returns from
`Run`, and:

```go
cmd/controller/main.go:34      for range gui.Run(p, ver) {
cmd/controller/main.go:35      }
cmd/controller/main.go:36      return nil
```

`run()` returns nil and `main` returns. Firing `ctx.Done` on a 3:30 timeout means
**the operator's machine stops having a UI until it is power-cycled.**

**(b) Nothing can set it while the saver is up.** `continue` at `gui/gui.go:3012`
skips the `break` at `:3015`, so the range body never returns, so `ctx.Frame`
never returns, so the flow never gets control. `Run` is the only code executing.

*(Correction to CONTINUITY 2026-08-08b §9: the saver branch **does** call
`yield()` — at `:2986`, on every `continue` iteration. That `yield` is `Run`'s
outer yield to `main`, not the frame yield to the flow. The substantive claim —
"a flow stays blocked inside `ctx.Frame` with its stack, and its secret, live" —
is correct; the mechanism is the skipped `break`, not a skipped `yield`.)*

### 4.4 The four §10.2.4 requirements, adjudicated

| # | requirement | flow-only? | why |
| --- | --- | --- | --- |
| 1 | keyed on **residency**, not last press | **yes** | the flow owns `p`; `p.SecretsResident()` is `seal/session.go:48`. A residency-start timestamp is a local. |
| 2 | **paused** while engraving | **yes** | the flow knows it is inside `NewEngraveScreen(...).Engrave(...)` (`gui/unlock_session.go:197`, `:294`) — it is a lexical fact. |
| 3 | **absent** when no secret resident | **yes** | the flow simply does not run the clock. |
| 4 | on firing, **UNWIND** (F-89) | **NO** | two independent blockers: (i) at the moment it must fire the flow is parked by the saver (§4.3b) — and that moment is *by construction* the saver's own moment (§5); (ii) even given frames, `ChoiceScreen.Choose` (55 lines, `gui/gui.go:1443-1497`) and `SeedScreen.Confirm` (137 lines, `:2330-2466`) are shared loops that return only on `ctx.Done` or their own affordances. A flow-local `expired` flag cannot make them return. |

**Requirement 4 requires a change inside `Run`, or a change to the shared
screens, or both.** It cannot be satisfied inside `gui/unlock_*.go` alone.

---

## 5. The screensaver interaction (F-93) — and the collision nobody has named

### 5.1 The collision

§10.2.4 row 1 says the timer *"Reuses the existing `idleTimeout` value"* and
§10.2.4's prose says *"The warning wakes the screen and any touch resets it."*
Read together with row 1's 3-minute figure, the §10.2.4 deadline is
`last-input + 3 min`, gated on residency — **numerically identical to
`idleWakeup` at `gui/gui.go:2999`.**

Therefore, at the tick where the warning is due, `Run` evaluates
`idle := now.Sub(idleWakeup) >= 0` (`:3000`) as **true on the same tick**, takes
the saver branch (`:3007-3013`), and `continue`s. **The flow is handed no frame
at 3:00, ever.** The screen the spec asks to "wake" is asleep because of the same
constant, and the code that put it to sleep is the only code still running.

This is not a race — it is deterministic and follows from reusing `idleTimeout`.

### 5.2 What a fix must not break

The saver branch is unconditional today, so any change is a change for every
screen. Constraints:

- **Ordinary screens:** with no secret resident, `a.idle.active` must evolve
  byte-identically. A residency-gated guard satisfies this trivially; a global
  change to `idleTimeout` or to the `continue` does not.
- **The engrave path:** today the saver takes the screen at 3:00 of a ~21-minute
  cut (§3). If B2b suppresses the saver whenever a secret is resident *and*
  F-90's corrected predicate stays true through the cut, the saver is suppressed
  for the whole 21 minutes — an unintended behaviour change on the path §10.2.4
  row 2 exists to protect. The pause and the suppression are separate knobs and
  must stay separate.
- **B2a-ii's wipe guarantees:** `defer p.Wipe()` (`gui/unlock_flow.go:85`),
  `defer p.WipeSecretAt(i)` + hook (`gui/unlock_session.go:109-117`),
  `defer clear(m)` (`:250`), `defer clear(pass)` / `defer clear(key)`
  (`gui/unlock_kdf.go:323`, `:329`), `defer d.Wipe()` (`:214`),
  `defer func(){ clear(blob) }()` (`gui/unlock_flow.go:58`). **All are `defer`s.**
  F-89's constraint follows directly: `Run` must never call `p.Wipe()` (it cannot
  reach `p` anyway) — it must make the flow **return** so these run.

### 5.3 Shape of a fix that does not break ordinary screens

The minimum is a guard on the branch at `gui/gui.go:3007`, conditioned on the
residency seam from §2. Beyond that the choice of *who draws the warning* is
open (§OPEN DESIGN CHOICES); what is **fact** is that at 3:00 the only code with
control is `Run`, so either `Run` draws it or `Run` must first hand control back.

---

## 6. What already exists that B2b can reuse

| thing | what it actually does | usable here? |
| --- | --- | --- |
| `idleTimeout` (`gui/gui.go:2932`) | `const idleTimeout = 3 * time.Minute`. Sole consumer: `idleWakeup := a.idle.start.Add(idleTimeout)` (`:2999`). | **As a value, yes.** As *the* timer, no — it is the saver's deadline, and §10.2.4's is a different control that happens to share the number. Sharing the constant is what creates §5.1's collision; the plan should decide deliberately, not inherit. |
| `ctx.WakeupAt(t)` (`gui/gui.go:90-94`) | monotonic min-reduce into `ctx.Wakeup`; consumed as `AppendEvents`' deadline at `:2989-2990`; cleared by `ctx.Reset()` (`:96-102`). | **Yes** — it is how any deadline gets a frame. **Ordering is load-bearing:** `Run` reads `ctx.Wakeup` at `:2989` *before* `ctx.Reset()` at `:2995`, so a `WakeupAt` after `ctx.Frame` governs the NEXT frame. This is B2a-ii's C2, fixed at `gui/unlock_kdf.go:295` and pinned by `TestKDFProgressFramesAreSubmittedWithAnExpiredDeadline` (`gui/unlock_kdf_test.go:902`). |
| `Platform.AppendEvents` (`gui/gui.go:2891`; impl `cmd/controller/platform_sh2.go:369`) | blocks until deadline / wakeup / touch / stdin; **appends only on touch or stdin**. | **Read-only, yes** — it is what makes `a.idle.start` a true last-input time. Modifying it is the wrong layer (see §2). |
| `Platform.Wakeup()` (`gui/gui.go:2892`; impl `platform_sh2.go:420-428`) | non-blocking send on `p.wakeups`; `AppendEvents` returns `evts` **unchanged** (`:384-385`). | **Yes as a "recompute now" nudge; no as an activity signal** — it deliberately does not reset the idle clock. `ConfirmDelay.Progress` (`gui/gui.go:295-306`) is the existing user. |
| `saver.State` (`gui/saver/saver.go:15-39`) | snake/logo animation; `Draw(screen Screen)` (`:315`) writes **directly** through `Dirty`/`NextChunk` (`Screen` is `gui/saver/saver.go:279-286`), bypassing `op.Drawer`. Reset by `a.idle.state = saver.State{}` (`gui/gui.go:3004`). | **As a thing to suppress, yes.** Not as a warning renderer — it draws no text. Note `Run` *does* hold `d *op.Drawer` (`:2960`) and `a.mask` (`:2938`), so `Run` drawing an `op.Op` itself is mechanically possible. |
| `StartScreen.scanTimeout` (`gui/gui.go:1658`, set `:1751`, honoured `:1831-1832`) | a flow-held deadline: `ctx.WakeupAt(m.scanTimeout)` during layout, content changes when `time.Now().Before(...)` goes false. | **Yes — this is the precedent for a self-expiring screen**, and the only one. It works *only while the flow is getting frames*. |
| `ConfirmDelay` (`gui/gui.go:287-306`) + `ConfirmWarningScreen` (used `gui/unlock_flow.go:226-241`, `gui/gui.go:2347-2363`) | hold-to-confirm with `ConfirmNone/No/Yes`. Spins via `ctx.Platform.Wakeup()` rather than `WakeupAt`. | **Shape, yes.** But its body widget is `Warning`, whose only scroll input is `ButtonFilter(Up/Down)` — **unreachable on SH2 hardware** (F-95, `gui/unlock_flow.go:207-214`). A new warning screen must be short enough not to need scrolling. |
| `engraveRemaining` (`gui/gui.go:2763+`), driven by `ctx.WakeupAt(now + 500ms)` (`:2751`) | MM:SS countdown text. | **Yes** as the pattern for a ticking 30-second readout. |
| `seal.Payload.SecretsResident()` (`seal/session.go:48-60`) | see §7. | Yes, **after** its contract is fixed. |

**Not available and worth stating: a `Run`-level test harness.** `testPlatform`
(`gui/gui_test.go:341`) implements all twelve `Platform` methods, but
`AppendEvents` (`:415-419`) **ignores the deadline and returns immediately**, so
`iter.Pull`-driving `Run` with it busy-spins and never advances a synthetic
clock. `testing/synctest` is already used in `gui` (10 files), but synctest's
clock only advances when every goroutine is blocked — which this `AppendEvents`
never is. B2b must add a deadline-respecting test platform before it can test
anything it changes in `Run`.

---

## 7. `seal.SecretsResident()` at `a01b666` — F-90 item 2 is CONFIRMED

```go
seal/session.go:48  func (p *Payload) SecretsResident() bool {
seal/session.go:49      for _, r := range p.Secret {
seal/session.go:50          if !IsSecret(r.Class) {
seal/session.go:51              continue
seal/session.go:52          }
seal/session.go:53          for _, b := range r.Record {
seal/session.go:54              if b != 0 {
seal/session.go:55                  return true
seal/session.go:56              }
seal/session.go:57          }
seal/session.go:58      }
seal/session.go:59      return false
seal/session.go:60  }
```

with `IsSecret(c) == (c == ClassCodex32Secret || c == ClassMnemonic)`
(`seal/session.go:16-18`).

**True from:** `Opener.UnlockWithKey` populating `p.Secret`. Pinned by
`TestSecretsResidentIsFalseWhenTheSessionEnds` (`gui/unlock_session_test.go:469-475`),
whose premise assertion is that a freshly unlocked vector F reads true.

**False from:** the instant the *last* secret record's bytes are zeroed. On both
arms that is **when the plate is built, before `Engrave` is called**:

```go
gui/unlock_session.go:195      clear(rec)
gui/unlock_session.go:197      NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)
```
```go
gui/unlock_session.go:289      clear(rec)
gui/unlock_session.go:290      clear(m)
...
gui/unlock_session.go:294      NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)
```

pinned by `TestSecretRecordIsZeroWHILETheEngraveScreenIsUp`
(`gui/unlock_session_test.go:371`).

### 7.1 The `ms1` arm — F-90's claim, verified, and worse than filed

`unlockEngraveCodex32` (`gui/unlock_session.go:162-198`) between admission and
`clear(rec)`:

```go
163      s, err := codex32.New(string(rec))
170      id, _, _ := s.Split()
172      plan, err := backup.EngraveSeedString(params, backup.SeedString{
173          Title: id,
174          Seed:  s.String(),
176      })
181      plate, err := toPlate(plan, params)
```

- `codex32.New` stores the whole share verbatim: `type String struct { s string }`
  (`codex32/codex32.go:16-17`), `ret := String{s}` (`:120`). `s.String()`
  (`:390-392`) returns that same field. Unwipeable.
- `id` from `Split()` (`codex32/codex32.go:394-401`) is `p.id`, the 4-character
  identifier — **metadata, not key material.** F-90 lists it among "spendable key
  material"; that one row overstates.
- `backup.EngraveSeedString` (`backup/backup.go:125-140`) additionally allocates
  `strings.ToUpper(plate.Seed)` at `:126`.

**The finding F-90 does not have, and it is the load-bearing one.**
`plate.Spline` is not a materialised buffer of geometry — it is a **lazy
closure over the plaintext**:

```go
bspline/bspline.go:22    type Curve = iter.Seq[Knot]
engrave/engrave.go:1025  func planEngraving(knotBuf []bspline.Knot, conf StepperConfig, e Engraving) bspline.Curve {
engrave/engrave.go:1026      return func(yield func(bspline.Knot) bool) {
engrave/engrave.go:1032          for c := range e {
```
```go
backup/backup.go:214     func frontSideSeed(params engrave.Params, plate Seed, qrc *engrave.ConstantQRCmd) engrave.Engraving {
backup/backup.go:215         return func(yield func(engrave.Command) bool) {
backup/backup.go:230             n := len(plate.Mnemonic)
```

`toPlate` (`gui/gui.go:3052-3065`) builds `spline := engrave.PlanEngraving(...)`
and stores it in `Plate.Spline` unmaterialised; it is iterated **twice** — once
by `bspline.Measure` (`bspline/bspline.go:206`, `for c := range spline` at `:217`)
for duration/bounds, and again by `runEngraving` (`gui/engraver.go:170`,
`for k := range e.spline`) during the cut. So the codex32 share string and the
mnemonic's `words []string` are **re-read on every knot for the whole ~21
minutes**, not merely "rendered as geometry" (F-83's framing).

**Consequence for F-88.** F-88 lists `engraveSeed`'s `words []string`
(`gui/gui.go:527-530`) as fixable — *"`clear(words)` is free and in-package"*.
That is **false as a remedy at the only place it would help**: `words` is
captured by `backup.Seed{Mnemonic: words}` (`gui/gui.go:531-532`) into
`frontSideSeed`'s closure and read lazily during the cut. Clearing it after the
plate is built cuts a corrupt plate. The only safe placement is *after* `Engrave`
returns, which shortens no window at all. F-88's severity framing ("shortens no
window that F-83 does not already hold open") is right; its stated fix is not.

### 7.2 Verdict on the predicate

F-90 item 2 is **CONFIRMED**: `SecretsResident()` reads *false* on both arms from
the instant the plate is built, while a full plaintext copy of the secret is live
for the whole cut — on the `ms1` arm as `codex32.String.s` + `ToUpper` copy +
the spline closure; on the mnemonic arm as `words []string` + the spline closure.
The function's own doc already says so (`seal/session.go:20-47`) and ends
*"Fix the contract before building the timer on it."*

A corrected predicate has a consequence the plan must own: **residency would then
be true for the entire ~21-minute cut**, which makes §10.2.4 row 2's *pause*
load-bearing rather than moot, and makes the saver-suppression question in §5.2
bite for 21 minutes rather than for a choice screen.

---

## 8. The 30-second warning

**Operator decision, recorded:** CONTINUITY 2026-08-08 §5 — *"warning at 3:00,
wipe at 3:30, paused while engraving (§10.2.4 row 2)."*

**Precedent for a countdown/warning screen: none that fits.** The complete
inventory of time-driven UI in `gui/`:

1. `StartScreen.scanTimeout` — a self-expiring *status line*, not a screen
   (`gui/gui.go:1658`, `:1751`, `:1831-1832`). Closest structural precedent.
2. `ConfirmDelay` + `ConfirmWarningScreen` — hold-to-confirm; the delay is 1 s
   (`confirmDelay`, `gui/gui.go:308`) and the *operator* drives it.
3. `engraveRemaining` (`gui/gui.go:2763+`) — MM:SS text on the engrave screen.

Nothing in the tree shows a modal that counts down and then acts on its own.
This is new UI.

**How the screensaver's activation interacts with showing one — this is the
blocker, not a detail.** Per §5.1, the warning's due instant *is* the saver's due
instant, and at that instant `Run` takes `:3007-3013` and the flow gets no frame.
Concretely, three things follow:

- A warning drawn by the flow via `ctx.Frame` **cannot appear at 3:00** unless
  `Run` first declines to take the saver branch.
- Even if the warning did appear, it would need to *survive* 30 seconds while
  `a.idle.start` is unchanged — i.e. `Run` must keep declining for the whole
  window.
- The wipe at 3:30 has the same problem in stronger form: no frame, no flow, no
  `return`, no `defer`.

**F-95 constrains the copy.** `Warning`'s body is unscrollable on SH2
(`gui/unlock_flow.go:207-214`: its only inputs are `ButtonFilter(Up)`/`(Down)`;
`processTouch`, `cmd/controller/platform_sh2.go:398-418`, emits `PointerEvent`
exclusively) and the §10.2.3 warning already sits 19 px into the overflow window.
A 3:00 warning must be short enough to fit without scrolling, and must not
inherit `Warning` unexamined.

---

## 9. What B2b must NOT break — the explicit list

1. **The screensaver for ordinary screens.** With no secret resident, lines
   2992-3015 must behave identically. Any guard must be residency-conditional,
   and `idleTimeout`'s value must not move (it is the saver's, used by every
   screen in the firmware).
2. **The engrave path's untouched screen.** Today the saver *does* cover a cut
   from 3:00 onward and the job continues on its own goroutine
   (`gui/engraver.go:109-112`, `:197-207`). §10.2.4 row 2 says the *timer* pauses;
   it does not say the saver stops. Conflating the two suppresses the saver for
   21 minutes on the arm six of seven vectors take.
3. **B2a-ii's wipe guarantees, all of which are `defer`s.**
   `gui/unlock_flow.go:58`, `:85`; `gui/unlock_session.go:109-117`, `:250`;
   `gui/unlock_kdf.go:214`, `:323`, `:329`. F-89 restated in code terms: a wipe
   that does not make the frame **return** leaves every one of these unrun.
4. **The tests that currently pin the above.**
   `TestUnlockFlowWipesOnTheCtxDoneAndErrorExits` (`gui/unlock_wipe_test.go:235`),
   `TestSecretRecordIsZeroWHILETheEngraveScreenIsUp` (`gui/unlock_session_test.go:371`),
   `TestSecretsResidentIsFalseWhenTheSessionEnds` (`:469`),
   `TestMnemonicWordsAreZeroWhenThePlateReachesEngrave` (`:649`),
   `TestKDFProgressFramesAreSubmittedWithAnExpiredDeadline` (`gui/unlock_kdf_test.go:902`).
   Note that a corrected `SecretsResident()` contract (§7.2) **will** change what
   `TestSecretsResidentIsFalseWhenTheSessionEnds` and
   `TestSecretRecordIsZeroWHILETheEngraveScreenIsUp` assert; that is a deliberate
   contract change, not a regression, and it needs saying in the plan.
5. **`Run`'s read-before-`Reset` ordering** (`:2989` before `:2995`). Any new
   `WakeupAt` must precede its `ctx.Frame`, per B2a-ii's C2.

---

## THE SPEC SENTENCE

**Misleading — half true, and the true half is not the half that matters.**

The spec:

> §10.2.4:1326-1329 — **"The timer source is already in use and needs no new
> machinery"**: `gui/gui.go:2801` `idleTimeout = 3 * time.Minute`, driven by
> `time.Now()` and `ctx.WakeupAt`/`Platform.AppendEvents` in `Run`'s frame loop.
> Monotonic elapsed time is all this needs; no RTC is involved.

**What is TRUE.** The *time source* needs nothing new. `Run` already maintains a
monotonic last-input timestamp from real input only:

```go
gui/gui.go:2990          evts = pl.AppendEvents(wakeup, evts[:0])
gui/gui.go:2991          now := time.Now()
gui/gui.go:2992          if len(evts) > 0 {
gui/gui.go:2993              a.idle.start = now
gui/gui.go:2994          }
```

and `cmd/controller/platform_sh2.go:382-385` returns `evts` unchanged on the
timer and on `Wakeup`, so it really is last-*physical*-input. No RTC. That claim
stands.

**What is FALSE.** "needs no new machinery", read as B2b will read it — that
§10.2.4 is a consumer of an existing mechanism rather than a new one. Four things
that do not exist and cannot be added from a flow:

1. `a` is `struct{ mask *image.Alpha; idle struct{ start time.Time; active bool; state saver.State } }`
   **declared at `gui/gui.go:2937-2944 inside the closure `Run` returns**.
   `Context` (`:64-73`) has no idle field; `Event` (`gui/event.go:105-109`) has no
   timestamp. **There is no accessor. A seam must be added to `Run`.**
2. `Run` has no way to know a secret is resident. `seal.Payload` exists only
   inside `unlockPayloadFlow` (`gui/unlock_flow.go:64-85`).
3. `Run` has no way to know an engrave is running. `grep -n engrave gui/gui.go`
   has no hit in `2932..3020`; the only signal that crosses is
   `defer e.pl.Wakeup()` (`gui/engraver.go:110`), which carries **zero events**.
4. Firing the timer must make the flow **unwind** (F-89). The only existing
   unwind is `ctx.Done`, and `ctx.Done` **exits the GUI** —
   `uiFlow` is itself `for !ctx.Done` (`gui/gui.go:1595`) and
   `cmd/controller/main.go:34-36` returns from `main` when `Run` returns.

And the sentence hides its own contradiction. Reusing `idleTimeout` — which
§10.2.4 row 1 explicitly requires — makes the §10.2.4 warning fall on **the same
tick** as the screensaver:

```go
gui/gui.go:2999              idleWakeup := a.idle.start.Add(idleTimeout)
gui/gui.go:3000              idle := now.Sub(idleWakeup) >= 0
...
gui/gui.go:3007              if a.idle.active {
gui/gui.go:3008                  a.idle.state.Draw(pl)
gui/gui.go:3011                  ctx.WakeupAt(now.Add(minFrameTime))
gui/gui.go:3012                  continue
gui/gui.go:3013              }
```

The `continue` skips the `break` at `:3015`, so the range body never returns and
`ctx.Frame` never returns to the flow. §10.2.4:1323 says *"The warning wakes the
screen"* — the screen is asleep at exactly that instant, and only `Run` can wake
it. **Reusing the existing machinery is the thing that breaks §10.2.4, not the
thing that implements it.**

Two smaller spec/record corrections while we are here:

- §10.2.4's line citation `gui/gui.go:2801` is wrong twice (actual **2932**);
  CONTINUITY 2026-08-08b §10's correction to 2879 is also stale.
- §10.2.4 rows 1 and 3 read `SecretsResident()` as "no seed material is
  resident". `seal/session.go:20-47` says in its own doc that this is false, and
  §7 above confirms it on both arms. **§10.2.4's third row is presently wrong**
  for the whole duration of every cut.

---

## THE MINIMAL SURFACE

Smallest set of changes **outside `gui/unlock_*.go`** that B2b requires. Items
1-4 are forced by facts above; item 5 is forced only under one of the §OPEN
choices and is marked as such.

1. **`seal/session.go` — correct `SecretsResident()`'s contract.** *(F-90 item 2,
   F-89's amended half.)* Forced: §10.2.4 keys three rows on a predicate that goes
   false while a full plaintext copy is live (§7). Whether the fix is a residency
   *flag* set/cleared by the session, a widened scan, or a renamed narrow
   predicate plus a new wide one is a design choice (§OPEN 4) — but the timer
   cannot be built on the current contract, and this is the one item that has a
   funds consequence. It also re-earns
   `TestSecretsResidentIsFalseWhenTheSessionEnds` and
   `TestSecretRecordIsZeroWHILETheEngraveScreenIsUp`.

2. **`gui/gui.go` — a residency seam on `Context`.** One field, plus its read in
   `Run`. Justified: §2 — `Run` cannot otherwise learn it, `Platform` is the wrong
   layer (twelve hardware methods, `:2889-2912`), a package var is the test-hook
   idiom and is not per-`Context`, and `ctx.Wakeup`/`ctx.Done`/`ctx.FrameCallback`
   are the three existing precedents for exactly this seam.

3. **`gui/gui.go` — gate the saver branch at `:3007` on that seam.** Justified:
   §5.1 — the warning and the wipe are due at the instant `Run` unconditionally
   parks the flow, and `Run` is the only code with control at that instant. The
   gate must be residency-conditional so ordinary screens are untouched (§9.1),
   and must be distinguishable from §10.2.4 row 2's *pause* so the 21-minute cut
   does not silently lose its screensaver (§9.2).

4. **`gui/gui.go` — an unwind that returns to the main menu instead of exiting.**
   Justified: §4.3a — `ctx.Done` unwinds correctly through all 58 loops (§4.2,
   already pinned by `gui/unlock_wipe_test.go:235`) but terminates `uiFlow`
   (`:1595`) and hence `Run` and hence `main` (`cmd/controller/main.go:34-36`).
   The shape is open (§OPEN 2); the *need* is not.

5. **`gui/gui.go` — a way for `ChoiceScreen.Choose` (`:1443-1497`) and
   `SeedScreen.Confirm` (`:2330-2466`) to return on the timer.** Required **only
   if** §OPEN 2 is resolved as "flow-local flag" rather than "reuse `ctx.Done`".
   Under the `ctx.Done` resolution these two need no edit at all, which is the
   single strongest argument for it.

6. **Test infrastructure — a deadline-respecting test `Platform`.** Justified:
   §6 — `Run` has **zero** test coverage today (no caller outside
   `cmd/controller` and `cmd/emu`), and `testPlatform.AppendEvents`
   (`gui/gui_test.go:415-419`) ignores its deadline, so neither `iter.Pull` nor
   `testing/synctest` can drive `Run`'s clock. Items 2-4 are otherwise untestable,
   which for a funds-path control is itself a blocking condition.

**Out of scope of "minimal", but owned by B2b and cheap to land beside it:**
F-96 (`scripts/mutation-run.py`), F-87 (drive `unlockEngraveMnemonic`'s three
early returns), F-94 (`deriveSeedHook` beside `bip39.MnemonicSeed`), F-88's
inventory correction per §7.1.

---

## OPEN DESIGN CHOICES

### 1. What clock does §10.2.4 measure?

The spec says both things. §10.2.4:1309-1310 *"keyed on whether any secret record
is resident, never on which button was last pressed"*; §10.2.4:1323-1324 *"any
touch resets it, so a present operator is never wiped out."*

- **(a) Idle-since-last-input, gated on residency** (`a.idle.start + 3min` while
  resident). *Cost:* identical to the saver's deadline → §5.1's collision is
  structural, and the "warning wakes the screen" sentence becomes a `Run`
  requirement. *Benefit:* satisfies 1323-1324 literally; a present operator is
  never wiped; reuses the exact quantity `Run` already maintains.
- **(b) Residency-start + 3 min, no reset on input.** *Cost:* fires at 3:00 flat
  during a legitimate steel swap between secret plate 1 and secret plate 2 of a
  2-of-3 — the very failure mode §10.2.4:1304-1307 opens by describing. *Benefit:*
  literally matches 1309-1310; the strongest bound on total exposure.
- **(c) Two clocks — residency-start as a hard ceiling, last-input as the soft
  one, whichever fires first.** *Cost:* two deadlines to reason about, two to
  test; the operator can be wiped mid-session while present. *Benefit:* the only
  option that bounds a 63-minute 2-of-3 session at all.

Nothing in the record decides this. It changes the shape of items 2-4 of the
minimal surface and should be settled before the plan is written.

### 2. What makes the flow unwind?

- **(a) Reuse `ctx.Done` + restart `uiFlow` in `Run`.** All 58 loops already
  honour it (§4.2) and the exit is already tested. *Cost:* `Run` must distinguish
  "the consumer stopped" from "we asked for an unwind" at `:2986`, must clear
  `ctx.Done` and re-enter the iterator, and `uiFlow` re-runs its §10.1
  `PayloadReader().Probe()` (`gui/unlock_flow.go` via `gui/gui.go:1583`) on
  restart. Zero changes to shared screens. **Cheapest by a wide margin.**
- **(b) A second flag (`ctx.Abort`) + change every loop condition.** *Cost:* 58
  edit sites in 23 files, every one a chance to miss one, and a missed one is a
  screen that ignores the wipe. *Benefit:* `ctx.Done` keeps meaning exactly one
  thing.
- **(c) Convert `ctx.Done` from a field to a method.** *Cost:* mechanical but
  touches all 58 sites plus 4 test sites plus `gui_test.go:38,78,510` and
  `unlock_plates_test.go:360` which *assign* it. *Benefit:* one predicate, both
  meanings, no missed loop.
- **(d) Unlock-local copies of the shared screens.** *Cost:* duplicating
  `ChoiceScreen.Choose` (55 lines) and `SeedScreen.Confirm` (137 lines) into
  `gui/unlock_*.go`, where they will drift from the originals. *Benefit:* nothing
  outside `gui/unlock_*.go` changes except the saver gate. Recorded for
  completeness; the drift cost is real and the two screens are exactly the ones
  that must not be wrong.
- **(e) `ctx.Done` with no restart — the UI simply exits.** *Cost:* the machine
  needs a power cycle after any idle wipe. *Benefit:* fail-closed, and it is
  ~1 line. Listed because it is genuinely on the table for a security control;
  it is also indistinguishable to the operator from a crash.

### 3. Who draws the 3:00 warning, and what happens to the saver?

- **(a) `Run` suppresses the saver whenever a secret is resident; the flow draws
  the warning and decides at 3:30.** *Cost:* the panel stays lit with a secret
  screen up — and on the mnemonic arm `SeedScreen.Confirm` (`gui/gui.go:2330`)
  **shows the twelve words**, so this leaves a seed legible on an unattended
  machine for the full 3.5 minutes. Under a corrected residency predicate (§7.2)
  it also suppresses the saver for the whole 21-minute cut (§9.2). *Benefit:*
  `Run` gains no drawing responsibility; the warning is ordinary flow UI; the
  30-second countdown is `StartScreen.scanTimeout`'s pattern exactly.
- **(b) `Run` suppresses the saver only for the last 30 s and signals the flow
  through a `Context` field the flow reads during layout.** *Cost:* one more
  field and a two-state machine in `Run`. *Benefit:* the saver still blanks the
  seed screen at 3:00 as it does today; the warning wakes it exactly as
  §10.2.4:1323 says; the flow still owns the pixels.
- **(c) `Run` draws the warning itself** (it holds `d *op.Drawer` at `:2960` and
  `a.mask` at `:2938`, and `ctx.Styles` is available). *Cost:* `Run` acquires
  layout code and a theme dependency it has never had; the warning cannot be
  tested through the existing flow harnesses. *Benefit:* no new `Context` state;
  works even if the flow is wedged.
- **(d) No warning — wipe silently at 3:00 or 3:30.** *Cost:* contradicts
  §10.2.4:1314 and the operator decision in CONTINUITY 2026-08-08 §5. Listed only
  so the plan records that it was considered and rejected.

### 4. What should `SecretsResident()` become?

- **(a) A session flag** — set when `p.Secret` is populated, cleared when the last
  secret's *plate is done* rather than when its record is zeroed. *Cost:* no
  longer a pure function of the buffers, so it can lie if a path forgets to clear
  it; `TestSecretsResidentIsFalseWhenTheSessionEnds` must be rewritten.
  *Benefit:* true for the whole cut, which is what §10.2.4 row 2 assumes.
- **(b) Keep the narrow predicate, rename it (`RecordsResident()`), and put the
  residency decision in `gui`** where the local copies and the plate live.
  *Cost:* the decision leaves `seal`, against `seal/session.go:3-7`'s own stated
  reason for living there ("the UI must not re-derive what the classifier already
  decided"). *Benefit:* honest — `seal` genuinely cannot see `codex32.String.s`,
  `words []string`, or `plate.Spline`.
- **(c) Leave it and accept that the timer is absent during every cut.** *Cost:*
  §10.2.4 row 2 becomes vacuous and the ~21-minute window is unguarded — which is
  the state F-89's amendment calls "a control that is correct only for the arm one
  vector reaches". *Benefit:* no contract change, no test churn.

### 5. Does the timer's 3:00/3:30 share `idleTimeout`'s constant?

§10.2.4 row 1 says it does. If (§1a) is chosen, sharing it forces the collision;
a distinct constant (e.g. 2:30 warning / 3:00 wipe, or 3:00/3:30 measured from a
different origin) would let the warning appear *before* the saver and reduce the
`Run` change to a suppression window rather than a takeover. That is a normative
change to §10.2.4 and belongs to the operator, not to the plan's author.
