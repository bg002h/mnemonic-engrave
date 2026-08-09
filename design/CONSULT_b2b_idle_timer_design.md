# CONSULT — B2b idle wipe (§10.2.4): the design, chosen and argued

Fable-tier consult, 2026-08-09. Code read at `seedhammer` `main` @ `a01b666`
(verified: `git log` head is `a01b666 Merge Plan B Phase B2a-ii`). Every line
number below was read at that commit; facts inherited from
`RECON_b2b_idle_timer_surface.md` are cited as RECON and were spot-verified
where this design leans on them. Two facts in §1.1 and §1.2 are **new** — not
in the RECON — and both were established by reading the cited bodies, not
inferred.

**The one-paragraph verdict.** The wipe is `ctx.Done`, set by a **field write
inside `Run`**, followed by `Run` **restarting the UI** in a new outer session
loop — the flow unwinds through the 58 already-verified `for !ctx.Done` loops,
every deferred wipe runs (F-89), and the machine lands on a fresh main menu
instead of exiting. The timer lives in `Run`; what crosses the boundary is one
unexported nil-able `Context` field installed by `unlockSecretSession` for
exactly its own lifetime — the timer is keyed on the **session bracket**, not
on `seal.SecretsResident()`, which the design deliberately does not consume.
The 3:00 warning is drawn **by `Run`**, saver-style, replacing the saver
whenever the timer is armed; during a running cut the timer reads the engrave
job's live state and is paused, and the saver covers the cut exactly as today.
§10.2.4 is **not implementable as written** in two places (row 3's definition
of "resident" and the "needs no new machinery" paragraph) and needs one
semantic clarification (row 2's "paused"); exact amendment text is in §4.

---

## 1. The mechanism: what makes a secret-bearing flow return

### 1.1 A new fact first, because it forces the shape

**`ctx.Done` has never once been set in production.** Its only production
writer is `gui/gui.go:2949`:

```go
ctx.FrameCallback = func(op op.Op) {
    ctx.Done = ctx.Done || !yield(op)
}
```

(verified by grep: no other non-test assignment exists), and `yield` there is
the range-over-func body of `for content := range it` (`gui/gui.go:2961`). The
body returns false only via the `return` at `:2986-2988`, which requires
`ctx.Done` already true or the *outer* consumer stopping — and both consumers,
`cmd/controller/main.go:34-35` (`for range gui.Run(p, ver) {}`) and
`cmd/emu/main.go` (same shape, then `select {}`), never break their range. So
the entire Done-unwind machinery that `TestUnlockFlowWipesOnTheCtxDoneAndErrorExits`
exercises is, today, test-only. B2b makes it fire on real hardware for the
first time. That reframes the risk: we are not "reusing a tested exit", we are
**productionising** one, and its first production use must tolerate everything
the tests' forgiving harness tolerates silently.

### 1.2 The second new fact: two screens `Frame` once more AFTER `Done`

RECON §4.2's table is correct that all 58 `for !ctx.Done` loops *return* on
`ctx.Done`. But "returns on Done" is not "never draws after Done", and two
paths draw one more frame on the way out:

- **`SeedScreen.Confirm`**: the nested Discard-Seed confirm loop at
  `gui/gui.go:2352-2363` is `for !ctx.Done`. When `Done` goes true while parked
  there, the loop exits, control **falls through** the enclosing
  `if backBtn.Clicked` block at `:2364` into the rest of the `events` body, and
  reaches `ctx.Frame` at `:2460` before the outer condition at `:2336` is
  re-tested. One extra `Frame` with `Done` true. The parked state is ordinary:
  the operator tapped Back on the twelve-words screen, the confirm appeared,
  and they walked away.
- **`EngraveScreen.Engrave`**: the hold-to-confirm loop at `gui/gui.go:2727-2746`
  is `for !ctx.Done`; on `Done` it falls through to `:2749` and reaches
  `ctx.Frame` at `:2758` before `:2703` re-tests. Same shape.

Why this matters: after the range body has returned false, a further `yield`
call is a **runtime panic** ("range function continued iteration…") — on this
watchdog-less device, a brick. Today that panic is unreachable only because
§1.1 holds. Any wipe design in which `Done` becomes true **by the body
returning false** (i.e. by `return`/`break` out of the range) inherits a
reachable brick from these two sites. The tests never saw it because `runUI`
uses `iter.Pull` (`gui/gui_test.go:503-514`), whose yield tolerantly returns
false after stop instead of panicking, and because the existing wipe test sets
`ctx.Done` by field write (`gui/unlock_plates_test.go:360`), never through the
range machinery.

### 1.3 The chosen mechanism

**Wipe = field write + normal body completion + discard-frames-while-wiping +
restart.** Concretely, in `Run`'s event loop (`gui/gui.go:2985-3016`), when the
wipe deadline passes:

```go
wiping = true
ctx.Done = true
break            // NOT the `return` at :2987 — complete this body iteration
```

The body finishes normally, `yield(op)` inside `FrameCallback` returns *true*
(the body never returns false), `ctx.Frame` returns to the parked flow, and the
flow unwinds through its `for !ctx.Done` conditions — **every `defer` runs**,
which is F-89's whole requirement: `defer clear(m)` (`gui/unlock_session.go:250`),
`defer p.WipeSecretAt(i)` + hook (`:109-117`), `defer p.Wipe()`
(`gui/unlock_flow.go:85`), `defer func(){ clear(blob) }()` (`:58`),
`defer clear(pass)` / `defer clear(key)` (`gui/unlock_kdf.go:323`, `:329`),
`defer d.Wipe()` (`:214`). `Run` never calls a wipe function and never touches
`p` — the unwind *is* the wipe, exactly as F-89 states it ("the timer's job is
to make flows RETURN, and the wipe is what their defers then do").

Two guards make this correct rather than plausible:

1. **At the top of the range body** (`gui/gui.go:2962`, before `Dirty`/draw):

   ```go
   if wiping {
       continue   // discard frames drawn during the unwind
   }
   ```

   This is **required, not belt-and-braces**, because of §1.2: on the ordinary
   walked-away-from-Discard-Seed path the unwind emits one extra frame, and
   without the guard that frame's body iteration reaches
   `if ctx.Done || !yield() { return }` at `:2986`, executes the `return`, and
   converts the wipe into a full GUI exit — the operator's machine stops having
   a UI (RECON §4.3a) as the direct result of the timer doing its job. With
   the guard, any number of stray frames is discarded harmlessly (the body
   `continue`s, yield returns true, the flow proceeds to its own condition test
   and returns). The guard also guarantees the unwind never blocks in
   `AppendEvents` and never yields to the outer consumer mid-unwind.

2. **An outer session loop in `Run`.** The current body of `Run`'s returned
   closure (`gui/gui.go:2936-3018`) becomes the body of `for { … }`, with
   `ctx := NewContext(pl)` and the `it :=` closure **inside** the loop and the
   idle struct `a`, `evts`, `d`, `stats` staying outside. After
   `for content := range it` ends:

   - if it ended via the `return` at `:2987` (consumer stopped, or a
     hypothetical future Done-without-wiping) — `Run` has already returned;
     nothing to do;
   - if `wiping` is set: reset `wiping = false`, `continue` the session loop —
     which builds a **fresh `Context`** (clean `Router`, clean `op.Buffer`,
     clean `Wakeup`, `Done == false`) and re-enters `uiFlow`. `uiFlow`
     re-probes the payload reader (`gui/gui.go:1583`), the Sealed Payload menu
     entry reappears, and reopening the session costs the twelve words and the
     ~31 s KDF — which is precisely the intended post-wipe state (§10.2.2's
     "re-cutting needs a fresh unlock").

   A fresh `Context` rather than a scrubbed one is deliberate: `EventRouter`
   carries pointer-capture state beyond what `Reset` clears
   (`gui/event.go:281-294` clears events and filters, not `r.pointer`), and a
   wipe is rare enough that the allocation cost is irrelevant.

### 1.4 What each affected loop does

| loop | change | behaviour under a wipe |
| --- | --- | --- |
| all 58 `for !ctx.Done` sites (RECON's machine count, 23 files) | **none** | return, defers run — already pinned by `gui/unlock_wipe_test.go:235` |
| `unlockSecretSession`'s `for _, i := range at` (`gui/unlock_session.go:95-99`, no Done check) | **none** | each remaining `unlockSecretPlate` calls `cs.Choose`, which returns `(0,false)` immediately at `gui/gui.go:1496`; treated as Skip; the defer wipes that record. The loop degenerates into wipe-all, drawing nothing. Verified by reading; matches RECON §4.2's row. |
| `unlockEngraveFlow`'s bare `for` (`gui/unlock_platelist.go:228-236`) | **none** | exits because `cs.Choose` returns `!ok` (unreachable during a wipe anyway — no secrets are resident on the public list) |
| the two Frame-after-Done sites (§1.2) | **none** | their extra frame is discarded by the `wiping` guard |
| `ChoiceScreen.Choose` (`gui/gui.go:1443-1497`), `SeedScreen.Confirm` (`:2330-2466`) | **none** — this is the point | they already return on `Done`; no shared screen grows a new exit |
| `Run`'s event loop (`gui/gui.go:2985-3016`) | gains armed/warning/wipe branches (§3) | sets `wiping` + `ctx.Done`, breaks |
| `Run`'s closure | gains the session loop | restarts the UI after a wipe |
| `uiFlow` (`gui/gui.go:1564`) | **none required** (see §6, optional lock notice) | re-entered fresh per session |

No `panic`/`recover`, no new exit taught to any shared screen, no change to any
flow loop. The seam is the one that already exists — `ctx.Done` — made safe to
fire in production by the discard guard, and made non-terminal by the session
loop.

---

## 2. Where the timer lives, and what crosses the boundary

### 2.1 The split

- **The clock, the deadlines, the warning state machine, the wipe decision:
  in `Run`**, as locals of the session loop beside `a.idle`. Flows can neither
  observe nor influence them except through real input — the same isolation
  `a.idle.start` already has, which RECON §1 shows is load-bearing.
- **The arming fact: one unexported nil-able field on `Context`**, because
  `Context` is the existing flow↔`Run` channel in both directions
  (`ctx.Wakeup` written by flows, read by `Run` at `gui/gui.go:2989`;
  `ctx.Done` written by `Run`, read by flows; `ctx.FrameCallback` the third
  precedent — RECON §2's table, which I adopt). Concretely:

  ```go
  // gui/gui.go, on Context (unexported — every reader and writer is in gui):
  wipe *wipeGuard

  type wipeGuard struct {
      // job is the engrave job currently cutting a secret plate, nil otherwise.
      // Registered by the two unlock engrave arms around their Engrave call.
      job *engraveJob
  }

  func (g *wipeGuard) armed() bool {
      if g == nil {
          return false
      }
      if j := g.job; j != nil {
          st := j.Status().State
          if st == engraveRunning || st == engraveStopping {
              return false // §10.2.4 row 2: never wipe mid-plate, needle down
          }
      }
      return true
  }
  ```

  `Run` evaluates `armed := ctx.wipe.armed()` once per event-loop tick. With no
  session open the field is nil and the two nil checks are the entire cost.

### 2.2 Who installs it — and why the timer does NOT consume `SecretsResident()`

**Install/uninstall bracket: `unlockSecretSession` (`gui/unlock_session.go:81`),
its own first and last act:**

```go
func unlockSecretSession(ctx *Context, th *Colors, p *seal.Payload) {
    g := &wipeGuard{}
    ctx.wipe = g
    defer func() { ctx.wipe = nil }()
    … existing body unchanged …
}
```

**Job registration: the two arms**, `unlockEngraveCodex32`
(`gui/unlock_session.go:197`) and `unlockEngraveMnemonic` (`:294`), around
their `Engrave` call:

```go
scr := NewEngraveScreen(ctx, plate)
if g := ctx.wipe; g != nil {
    g.job = scr.job
    defer func() { g.job = nil }()
}
scr.Engrave(ctx, &engraveTheme)
```

No function signature changes anywhere; `ctx` already reaches both sites.

This resolves RECON OPEN 4 by dissolving it: **the timer keys on the session
bracket, not on any predicate over buffers.** The bracket's lifetime — from
"secrets decrypted and being offered" to "last secret plate has left the
screen" — is *exactly* the residency window §10.2.4 needs, including the two
intervals where `seal.SecretsResident()` (`seal/session.go:48-60`) lies: the
whole ~21-minute cut of the last record (buffer zeroed at plate build,
`gui/unlock_session.go:195`/`:289`, plaintext live in `codex32.String`, the
words, and the `Plate.Spline` closure — F-83 as corrected, F-90 item 2) and the
plate-done walk-away screen after it. The gaps at the bracket's edges are
frame-free straight-line code (between `unlockSealedFlow` returning true and
the session call there is only `clear(blob); blob = nil`,
`gui/unlock_flow.go:110-114`), so the bracket is tight.

Consequences worth stating plainly:

- **`seal/session.go` needs no functional change for the timer.** F-90 item
  2's "fix the contract before building the timer on it" is satisfied by *not
  building on it*. The predicate keeps its narrow, honest job — the buffer
  assertion the wipe tests are built on — and
  `TestSecretsResidentIsFalseWhenTheSessionEnds` /
  `TestSecretRecordIsZeroWHILETheEngraveScreenIsUp` keep their current meaning,
  which RECON §9.4 feared would churn. Rename it `RecordsResident` (OPEN 4b)
  so nobody builds the wide reading on it again; its doc comment already makes
  the argument.
- **The rejected alternative**, a session flag inside `seal` (OPEN 4a), is the
  same information as the bracket with an extra indirection plus a new failure
  mode (a flag that lies when a path forgets it). `seal` cannot see
  `codex32.String`, `words`, or the spline closure; the honest owner of the
  residency *lifetime* is the code whose stack holds those copies, which is
  `gui`.
- **Convention verdict** (the brief's question): the nil-able-seam *shape*
  fits — nil means absent means today's behaviour, like `Platform.NFCReader`
  and `Platform.PayloadReader` — but both named conventions are the wrong
  layer. `Platform` is twelve hardware capabilities (`gui/gui.go:2889-2912`)
  implemented by three platforms; residency is GUI state. The `*Hook` package
  vars are documented test-only and are not per-`Context`. The right precedent
  is `Context`'s own flow↔`Run` field set, and since every party is in package
  `gui`, the field can and should be unexported — smaller surface than RECON's
  exported-func-field sketch, same fail-safe direction (a bracket that fails to
  *uninstall* leaves the timer armed during the public plate list:
  operator-hostile, not funds-unsafe; a test pins it, §5.3).
- **Concurrency:** none. `Run` and every flow share one stack (RECON §4.1);
  the guard is written and read on that one goroutine. `job.Status()`
  (`gui/engraver.go:126-151`) uses only non-blocking selects against the job
  goroutine's channels and is designed for polling — the guard calling it from
  `Run`'s tick is the same access pattern as the engrave screen calling it per
  frame. Side effect to note in the plan: the guard's `Status()` call will
  collect a completion that lands while the flow is parked, which is what flips
  `armed` and starts the post-cut window — that is the mechanism working, not
  an accident, and the cached `e.status` means the screen sees the same state
  when it next polls.

---

## 3. The clock, the warning, the screensaver, the engrave

### 3.1 The clock (resolves RECON OPEN 1 — and it was less open than filed)

§10.2.4 already chooses: *"any touch resets it, so a present operator is never
wiped out"* (spec :1323-1324) is last-input semantics, and *"keyed on whether
any secret record is resident, never on which button was last pressed"*
(:1309-1310) governs when the timer **exists**, not what resets it — the
paragraph after the table (:1318-1321) says exactly this, contrasting with a
design where a Cut/Skip *press* disables the timer. So: **idle-since-last-input
while armed** (OPEN 1a), with one addition the spec's text forces once row 2 is
taken seriously:

> **The wipe origin is `max(last physical input, last transition to armed)`.**

Without the second term, the window inherits a 21-minute-stale input clock at
cut end and the warning fires the instant a plate finishes — an instant wipe
threat aimed at the machine's own just-completed work. `Run` keeps
`wipe.origin` beside `a.idle`: refreshed by `len(evts) > 0` (same source as
`a.idle.start`, `gui/gui.go:2992-2994` — a true last-physical-input, RECON §1)
and by any `armed` false→true edge (fresh install after unlock: operator just
typed, both terms agree; unpause at cut end: the cut's end restarts the 3:00).
Deadlines: warning at `origin + idleTimeout` (3:00 — row 1's *"reuses the
existing `idleTimeout` value"* is honoured as a **value**), wipe at
`+ wipeWarningDelay` (new `const … = 30 * time.Second`). `wipe.origin` is
deliberately a separate variable from `a.idle.start` (RECON §6 row 1: decide,
don't inherit): they share a source but not a lifetime, and the arm-edge reset
applies to one and not the other.

The hard residency ceiling (OPEN 1b/1c) is **not required by §10.2.4 as
written**, would fire during the legitimate steel swap the spec itself
protects (:1304-1307), and is a normative change — available to the operator,
not chosen here. The spec's own "What it does not do" paragraph (:1331-1335)
already accepts that this control is a backstop against forgetting, not a
bound on a present attacker.

### 3.2 The warning: `Run` draws it, because nothing else can

At `origin + 3:00` the flow is parked — that is not a bug to route around but
the definition of the situation: the warning exists *because* nobody is
touching the machine, and after 3:00 of no touches the only code with control
is `Run` (RECON §5.1: deterministic, same tick as the saver). The flow cannot
draw it without every shared screen learning a new signal (fact 4), and the
saver cannot draw it (no text, `gui/saver/saver.go`). So `Run` draws it — this
is RECON OPEN 3(c), and it is *forced* by the constraints, not preferred:
3(a) leaves `SeedScreen.Confirm`'s twelve words lit for 3.5 unattended minutes
and suppresses the saver for whole cuts under a corrected residency notion —
disqualified; 3(b) still needs the flow to get frames and to *know about* the
warning, which shared screens cannot without edits — disqualified by fact 4.

Shape, precisely the saver's:

- **While armed, the saver branch is skipped entirely**: the idle computation
  at `gui/gui.go:3000` becomes `idle := now.Sub(idleWakeup) >= 0 && !armed`.
  With no session open, `armed` is false and lines 2992-3015 evolve
  byte-identically — RECON §9.1 satisfied by construction.
- At `warnAt`, `Run` enters a warning state: it stops routing events to widgets
  (the same suppression the saver uses at `:2996-2998` — a wake-touch must
  reset the timer, never press "Cut this plate"), and draws a full-screen
  warning each tick through the same door the saver uses (`pl.Dirty` +
  `NextChunk` + `d.Draw`), composed from a **`Run`-local `op.Buffer`** — never
  `ctx.B`, which still holds the parked flow's in-flight frame (the flow is
  parked *inside* `FrameCallback`, before `Frame`'s `c.B.Reset()` at
  `gui/gui.go:79`). Background fill first, then text: the warning **replaces**
  whatever was on screen, which is itself the privacy blanking — the underlying
  screen may be the twelve words.
- The op construction is a pure function
  (`wipeWarningOp(b *op.Buffer, st Styles, dims image.Point, remaining time.Duration) op.Op`)
  so `op.Drawer.ExtractText` can assert its content in tests without driving
  hardware chunks. Copy constraints: short enough to need no scrolling (F-95 —
  `Warning`'s scroll input is unreachable on SH2), countdown in
  `ctx.Styles.progress` (digits and colon exist — `engraveRemaining` uses this
  face; **no `%` glyph**, F-86). Proposed copy, operator-amendable:
  `"SESSION LOCKING"` / `"Secrets will be cleared in 0:27."` /
  `"Touch the screen to stay unlocked."`
- Tick cadence: `ctx.WakeupAt(now.Add(time.Second))` and
  `ctx.WakeupAt(wipeAt)` (min-reduce, `gui/gui.go:90-94`), then `continue` —
  the saver's own loop shape at `:3011-3012`.
- A touch during the warning: `len(evts) > 0` refreshes both clocks, the
  warning state clears, and `Run` **breaks** the event loop so the parked flow
  is handed a frame and redraws the screen the warning replaced. The touch is
  swallowed (not routed), like a saver wake.
- At `wipeAt`: §1.3's three lines.

"The warning wakes the screen" (:1323) is thereby implemented literally: while
armed the screen at 3:00 shows the warning instead of the saver, and there is
never a moment where the warning is due behind a sleeping screen — the
collision (fact 2) is resolved by **precedence** (armed ⇒ the warning owns
3:00+), not by moving constants, so OPEN 5 closes as "share the value, as row 1
says".

### 3.3 The ~21-minute engrave

- **While the job runs** (`engraveRunning`/`engraveStopping`), `armed()` is
  false: no timer, no warning, no wipe — row 2 enforced by reading the job's
  live state, not by a flow-remembered flag that can go stale. The saver
  behaves **exactly as today**: it takes the screen ~3:00 into the cut, the cut
  continues on its own goroutine (`gui/engraver.go:109-112`), progress folds
  (`:197-207`). RECON §9.2's do-not-conflate requirement is met because pause
  (job state) and saver suppression (armed) are different predicates with
  different truth values during a cut.
- **"Actively engraving" is the job's state, not the screen's visibility.**
  The hold-to-start screen and the plate-done screen are armed — they are
  walk-away states, and on the plate-done screen of a 2-of-3's first plate,
  two more `ms1` records are still plaintext in `p.Secret`. The lexical
  alternative ("paused while `Engrave` is on screen") leaves that state
  untimed forever and is rejected.
- **Cut ends while the saver is up:** the job goroutine's `defer e.pl.Wakeup()`
  (`gui/engraver.go:110`) wakes `Run` with zero events; the guard's `Status()`
  collects the completion; `armed` flips true; origin resets to now. 3:00
  later the warning wakes the screen — incidentally fixing the stranded
  completion screen RECON §3 documents (today it stays behind the saver until
  a touch). 3:30 later, unattended, the wipe unwinds: `Engrave`'s
  `for !ctx.Done` exits, `defer s.job.Stop()` (`gui/gui.go:2698`) is a no-op on
  a finished job, the session loop wipes the remaining records (§1.4), restart.
  A **stopped** (operator-aborted) cut re-arms the same way, and a 3:30-later
  wipe of its paused screen is §10.2.2's stated price ("re-cutting needs a
  fresh unlock; that is the price and it is deliberate").
- **A wipe can never fire needle-down** unless the guard misreports; §5 takes
  that failure mode seriously rather than declaring it impossible.

### 3.4 F-93, landed by the same hand (scoped adjunct, not the centrepiece)

The KDF park is the same `Run`-blindness with a different victim, and F-93
names option 1 (derivation counts as activity) as preferred and B2b-owned. Give
`Context` a per-frame, `Reset`-cleared flag — `ctx.KeepAwake()` — set by
`unlockDerive` beside its existing `ctx.WakeupAt(time.Now())`
(`gui/unlock_kdf.go:295`); `Run` reads it before `ctx.Reset()` and treats it as
`a.idle.start = now`. Self-sustaining while the derivation makes progress,
fail-safe when it stops (no frames ⇒ no flag ⇒ saver returns). It refreshes the
**saver** clock only, never `wipe.origin` — during the KDF no secrets are
armed (the bracket installs after unlock), so the two controls stay disjoint,
which is F-93's own stated requirement. This also bounds the
passphrase-resident window (typed words + flash ciphertext, §2.2 item 9
adjacent): the KDF now completes in its computed ≤206 s rather than parking
unbounded. Extending the *wipe* bracket back over passphrase entry would guard
that window with the timer too; it exceeds §10.2.4 as written and fires new
questions (a present operator watching a 206 s bar must not be wiped), so it
is named here as a filed option, not done.

---

## 4. §10.2.4's text: what is implementable as written, what must be amended

The spec is GREEN and normative; whether to amend is the operator's decision.
My finding is that **an amendment is REQUIRED** — two sentences are false
against the measured code and cannot be implemented faithfully, and building
to them silently would put the spec and the control in contradiction on a
funds path.

**Implementable as written:**

- *"3 min, 30 s warning … Reuses the existing `idleTimeout` value"* (row 1) —
  yes, as a **value** (§3.2 resolves the collision by precedence). The
  parenthetical cite `gui/gui.go:2801` is stale (actual `:2932`, twice in the
  section).
- *"keyed on whether any secret record is resident, never on which button was
  last pressed"* (:1309-1310) — yes **under the amended definition of
  "resident" below**; under the code's current definition
  (`seal.SecretsResident()`) it is not safely implementable, per its own doc
  (`seal/session.go:23-44`) and F-90 item 2.
- *"The warning wakes the screen and any touch resets it, so a present
  operator is never wiped out and an absent one is."* (:1323-1324) — yes,
  literally, under §3.2.
- The rationale paragraphs (:1304-1307, :1318-1321, :1331-1335) — stand
  unchanged.

**Requires amendment:**

1. **Row 3, and the section's implicit definition of residency.**
   *"**no** secret record resident | **none** | Public data only. Nothing to
   protect."* — as measured, `SecretsResident()` reads false from the instant
   a plate is built (`gui/unlock_session.go:195`, `:289`) while a full
   plaintext copy is live for the whole cut and on the plate-done screen
   (RECON §7, F-83 as corrected). Row 3 as written would disarm the timer for
   the most dangerous stretch of the arm six of seven vectors take. **Add
   after the table:**

   > **"Resident" is a lifetime, not a buffer scan.** A secret is resident
   > from the moment the sealed branch populates `p.Secret` until the last
   > secret record's plate has left the screen — implemented as the lifetime
   > of the secret session (`unlockSecretSession`), during which the flow
   > still holds copies (`codex32.String`, the parsed words, the plate's
   > spline closure — F-83) that no buffer scan can see.
   > `seal.RecordsResident()` (née `SecretsResident()`) measures only seal's
   > own record buffers and MUST NOT be the timer's key.

2. **The "no new machinery" paragraph** (:1326-1329) — *"The timer source is
   already in use and needs no new machinery: `gui/gui.go:2801` `idleTimeout =
   3 * time.Minute`, driven by `time.Now()` and
   `ctx.WakeupAt`/`Platform.AppendEvents` in `Run`'s frame loop."* — the first
   clause is true, the bolded claim is false four ways (RECON THE SPEC
   SENTENCE, all four verified). **Replace with:**

   > **The timer VALUE and time source are reused; the timer is new
   > machinery.** `idleTimeout` (`gui/gui.go:2932`) supplies the 3-minute
   > constant, and `time.Now()`/`ctx.WakeupAt`/`Platform.AppendEvents` the
   > monotonic clock — no RTC. The mechanism is new and lives in `Run`: a
   > residency seam on `Context` installed for the secret session's lifetime,
   > a `Run`-drawn warning that takes the screensaver's place while the timer
   > is armed (at 3:00 the flow is parked and only `Run` has control), and a
   > wipe that unwinds the flow by setting `ctx.Done` — §10.2.2's existing
   > exit, so every deferred wipe runs — after which `Run` restarts the UI at
   > the main menu. The machine never exits its UI and never needs a power
   > cycle; reopening the session costs the passphrase and the KDF.

3. **Row 2, one clarifying sentence** (semantic, small, but normative):

   > "Paused" restarts the window: when a cut ends — completion, stop, or
   > failure — the timer re-arms with a fresh 3:00 measured from the cut's
   > end. "Actively engraving" means the engrave job is running, not that the
   > engrave screen is visible: the hold-to-start and plate-done screens are
   > armed, because they are walk-away states with secrets still held.

4. **Housekeeping in the same edit:** both `gui/gui.go:2801` cites → `:2932`;
   §2.2 item 9's narrative gains nothing but could cross-reference the timer's
   restart behaviour; §10.2.2's exit list ("Lock, Back, an error, `ctx.Done`")
   is already consistent — the idle wipe *is* the `ctx.Done` route and needs
   no new entry.

---

## 5. What could go wrong with this design — named, with the tests that catch it

The two prior Criticals were wipes that looked complete and were not. This
design's analogue, plus its other failure modes, in order of severity:

### 5.1 The unwind that never finishes (this design's signature failure)

If any screen on the secret stack loops on `ctx.Frame` **without** checking
`ctx.Done`, the discard guard converts it into a livelock: frames drawn and
discarded forever, the restart never reached, **secrets still resident behind
a dead-looking screen** — worse than a visible crash because it looks like the
saver. This is not a new contract — shutdown already requires it, and
`uiFlow`'s own comment (`gui/gui.go:1590-1594`) documents the hang — but the
wipe makes it a funds property instead of a shutdown nicety. What catches it:

- RECON §4.2's audit (58/58 loops conform; the two fall-through sites are
  tolerated by design, §1.2) is the baseline, machine-counted.
- The **Run-level wipe test over the real screens** (§5.4) parks the flow in
  each secret-session screen in turn — `ChoiceScreen` offer, `SeedScreen`
  words, the Discard-Seed confirm, plate-done — fires the clock to 3:30 under
  `synctest`, and asserts the **restart frame arrived** (StartScreen content
  via `ExtractText`) within a bounded number of ticks. A livelock fails the
  bound; a conversion-to-exit (guard deleted — the mutation) fails the
  restart assertion. This single test kills both §1.2-class regressions.

### 5.2 A wipe that looked complete and was not

The restart is highly observable (fresh menu), which invites exactly the
false-completeness trap F-89 names. What survives an honest wipe, stated so
the tests assert the right thing: (a) everything a `defer` clears is cleared —
assert on **buffers via the hooks**, never return values or frame counts (the
project's false-PASS precedent): `unlockSecretHook("wiped", …)` for every
index, the F-87 early-return drives for `defer clear(m)` (scheduled with these
tests, same observability), `unlockPassphraseWordsHook`/`unlockKeyHook` for the
KDF path; (b) what no wipe can reach — string copies, spline closures, the old
`Context`'s `op.Buffer` (`op.Buffer` is `{args []uint32; refs []any}`,
`gui/op/op.go`, and its refs can include the last-drawn frame's text) — is
**dropped, not zeroed**, GC-reclaimed, F-83/F-88 class. The design must not be
recorded as zeroing it; the amendment text in §4 says "cleared" only of the
record buffers. The one new residue this design adds over a Back-exit today is
none: a Back-exit drops the same references the same way.

### 5.3 The guard's four lies

| lie | effect | test that catches it |
| --- | --- | --- |
| bracket never installed (install line lost) | timer silently absent for the whole session — the pre-B2b state back again | flow-level: drive `unlockPayloadFlow` over vector F (existing harness), assert `ctx.wipe != nil` inside `unlockSecretHook("offered", …)` |
| bracket never uninstalled | timer armed over the public plate list; a 3:30 pause mid-bundle dumps the operator to the menu — the teach-them-to-disable-it failure (:1304-1307) | same drive, assert `ctx.wipe == nil` inside `unlockEngraveHook` (public-arm hook) |
| `g.job` never registered | timer runs during a cut; at 3:30 the unwind's `defer s.job.Stop()` halts the needle mid-plate — row 2 violated, plate ruined (not funds loss: the record was cleared at plate build, and §10.2.2 already defines the abort) | Run-level with `testEngraver`: job running, advance past 3:30, assert no warning, no unwind, job still running |
| `g.job` never released | after the first cut the timer is dead for the rest of the session | Run-level: complete the job, advance 3:00, assert the warning appears |

Each row is a mutation the suite must kill (the project's mutation discipline:
break it and see the test notice).

### 5.4 The harness, and the false-PASS shapes to refuse

`Run` has zero coverage and `testPlatform.AppendEvents`
(`gui/gui_test.go:415-419`) ignores its deadline, so B2b builds first:

- a deadline-respecting test platform whose `AppendEvents` **durably blocks**
  (select over `time.After(until deadline)`, an injected-event channel, and
  wakeups — mirroring `cmd/controller/platform_sh2.go:369-396`, including
  returning `evts` unchanged on timer/wakeup). Durable blocking is what lets
  `testing/synctest` (already in 9 `gui` files, measured) advance the fake
  clock deterministically: the warning must appear at exactly
  `origin + 3:00`, the wipe at exactly `+ 3:30` — no sleeps, no frame counts.
- a flow seam for `Run`'s hardwired `uiFlow` (`gui/gui.go:2955`):
  `var runFlow = uiFlow`, the `newDeriver` idiom (`gui/unlock_kdf.go:51` —
  "production is always X; a test swaps it"). Run-level tests install a small
  secret-simulating flow plus, for §5.1's test, the *real* session screens.

False-PASS shapes this feature has already produced once, banned by
construction: frame-count assertions (assert extracted text and buffer state);
"warning appeared" without content (assert the countdown digits and that **no
seed word appears** in the composed warning frame — `ExtractText` over the
full layer detects a transparent-background compositing bug that would show
twelve words under the warning); "wipe happened" from `SecretsResident()`
alone (it goes false at plate build regardless — assert the hooks' buffers
and the restart frame).

Residual, stated so it is not discovered as a surprise: no single test drives
touch→passphrase→unlock→3:30→restart through real hardware event encoding;
the composition is covered piecewise (flow-level install/uninstall, Run-level
timing/unwind/restart over real screens) plus this project's standing
hardware-rehearsal practice before the feature ships. And the two
Frame-after-Done sites (§1.2) remain latent panics for a *true* consumer-stop
shutdown — unreachable today, out of B2b's scope, worth a follow-up entry so
they are not rediscovered by the next feature that productionises `Done`.

---

## 6. Operator decisions required (framed, not made)

1. **The §4 amendments** — required for rows 2/3 and the machinery sentence;
   the residency definition is the one with funds consequence.
2. **Warning copy and the post-wipe notice.** §3.2's copy is a proposal. The
   restart lands on the main menu (behind the saver, since the machine is
   idle); an optional one-screen notice on the next wake ("Session locked
   after inactivity — the payload needs its passphrase to reopen") would
   distinguish a wipe from a crash. One parameter into `uiFlow`; recommended,
   cheap, operator's call on the words.
3. **Declined extensions, recorded:** a hard residency ceiling (OPEN 1b/c) and
   extending the wipe bracket over passphrase entry/KDF (§3.4) — both
   normative changes beyond §10.2.4, both available later without disturbing
   this design.

## 7. Change inventory (what a plan is written against)

| where | what |
| --- | --- |
| `gui/gui.go` | `Context.wipe *wipeGuard` + `wipeGuard.armed()`; `Run`: session loop, `wiping` discard guard at body top, armed/warning/wipe branches in the event loop, `wipe.origin` tracking, saver gate `&& !armed`; `wipeWarningOp` (pure); `const wipeWarningDelay = 30 * time.Second`; `Context.KeepAwake` + `Reset` clearing (F-93) |
| `gui/unlock_session.go` | bracket install/uninstall in `unlockSecretSession`; job registration around both `Engrave` calls |
| `gui/unlock_kdf.go` | `ctx.KeepAwake()` beside `:295` (F-93) |
| `seal/session.go` | rename `SecretsResident` → `RecordsResident` + doc tightening only; no behaviour change |
| tests | deadline-respecting platform; `runFlow` seam; Run-level warning/wipe/restart/pause suite (§5); flow-level bracket pins; F-87's early-return drives; mutations per §5.3 |
| spec | §4's amendments (operator) |

No shared screen changes. No `Platform` changes. No signature changes. No
`panic`/`recover`. `cmd/controller` and `cmd/emu` unchanged — both consumers
already tolerate a `Run` that simply keeps yielding.
