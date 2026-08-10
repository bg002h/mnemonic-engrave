# Phase B2b — pre-flight wipe-safety review (last look before hardware)

**Reviewer:** independent adversarial pass (opus), read-only over
`git -C /scratch/code/shibboleth/seedhammer-b2b diff a01b666..b2b`.
**Date:** 2026-08-09.
**Scope, as briefed:** ONE question — *can the wipe fire at the wrong time, or
fail to fire?* Not a re-audit of the plan, the mutation rows, or the build.

**Verdict: 0 Critical, 1 Important, 3 Minor, 1 Nit.**

The timer machinery itself is sound. Every trace the brief named came back clean
(§ "Verified clean" below, with the machine checks). The one blocking finding is
not in the timer — it is in what the wipe *reaches*: the twelve words survive it
verbatim in `ctx.B`, and the diff's own comment asserts, in the exact block where
the question is raised, that they do not.

---

## IMPORTANT

### I1 — The wipe does not erase the rendered seed. `op.Buffer.Reset()` truncates `args` without zeroing it, and `run_flow.go:236` claims the opposite.

**Location:** `gui/run_flow.go:236-245`; `gui/op/op.go:374-378`; `gui/op/op.go:124-133`.

`gui/run_flow.go:236`, at the top of the wipe's session restart, states:

> `// NOTHING to scrub here, and that is worth stating because an earlier draft`
> `// got it backwards. ... clear(b.refs) (gui/op/op.go:376) runs on the last`
> `// frame drawn ... The abandoned Context's buffer is already zeroed by the`
> `// time control reaches this line.`

`Buffer.Reset()` zeroes **only one of its two arrays**:

```go
func (b *Buffer) Reset() {
	b.args = b.args[:0]   // TRUNCATED, not zeroed
	clear(b.refs)         // zeroed
	b.refs = b.refs[:0]
}
```

and `op.Glyph` — the single path every rendered character takes — encodes the
**rune itself into `args`**:

```go
return MaskOp{encodeOp(b, opMask, 0, []any{glyphImage, face}, uint32(r))}
```

`SeedScreen.Draw` renders the twelve words through it
(`gui/gui.go:2536` → `layoutWord(&ctx.B, …)` → `widget.Label` →
`op.Glyph(buf, st.Face, g.Rune)`), so the whole mnemonic is in `ctx.B.args`, one
`uint32` per character, in order.

**Machine-checked**, not reasoned. Program at
`…/scratchpad/wipecheck/main.go`, built against this worktree via a
`replace seedhammer.com => /scratch/code/shibboleth/seedhammer-b2b`, decoding
glyph args by their exact header word
(`encodeHeader(opMask,1,2) == 1<<16|2<<8|3`):

```
args len before Reset : 593 (cap 864)
Buffer.Len() after Reset: args=0 refs=0
plaintext recovered from the args BACKING ARRAY after Reset:
  "abandon ability able about above absent absorb abstract absurd abuse access accident"

RESULT: the full plaintext survives Buffer.Reset() verbatim.
```

**Why this is the wipe's problem and not merely pre-existing.** The phase already
treats this exact buffer as seed-bearing: the `warnBuf` comment
(`gui/run_flow.go:12-22`) says in terms that the parked frame "on
`SeedScreen.Confirm` is the twelve words", and that the warning's growth memcpy'd
it "into an array nothing ever zeroes." That defect was fixed. The *base* array
was not, and `run_flow.go:236` is the comment that closes the door on anyone
noticing — it is the one place a future reader would look, and it says the work
is done.

**Failure scenario.** Vector A or B (`pub_len == 0`, one secret mnemonic).
Operator unlocks, reaches `SeedScreen.Confirm`, is called away. 3:00 → warning;
3:30 → wipe. `p.Secret[i].Record` is zeroed, `bip39.Parse`'s `[]Word` is zeroed,
the key/passphrase/blob are zeroed, the Context is dropped, the machine shows the
main menu. The operator — and §10.2.4's own promise — believe RAM holds nothing.
An SWD probe reads the twelve words out of the abandoned `ctx.B.args` backing
array, verbatim and in order. §10.2.4's stated threat model
(SPEC §10.2.4 "What it does not do", §2.2 item 9) is *precisely* physical access
plus an SWD probe.

It is worse on the ordinary exit than on the wipe: outside a wipe the `Context`
is **not** dropped — the same `ctx.B` is reused by the main menu, which draws far
fewer glyphs, so the tail of the seed survives in the backing array for the rest
of the power cycle.

**Suggested fix.** Two lines, one of them new API in the same style as the
`Buffer.Len()` this phase already added:

```go
// gui/op/buffer_len.go (or beside Reset)
func (b *Buffer) Scrub() { clear(b.args); clear(b.refs); b.args, b.refs = b.args[:0], b.refs[:0] }
```

and call `ctx.B.Scrub()` in the wipe branch of `runWithFlow` before the Context
is dropped — i.e. exactly where `run_flow.go:236` currently explains why nothing
is needed. Then **replace that comment**; a stale record beside a fixed behaviour
is how the defect comes back (this file's own words, `unlock_session.go:56`).

Note the residual honestly in the new comment: `append` growth orphans earlier
`args` arrays that no handle can reach, so `Scrub()` covers the live array only.
That is the same class as F-88/F-83 and belongs in the same list.

**Why Important and not Critical, and why I would not argue with Critical.**
Against Critical: the residue is unreferenced heap, the spec's residency
definition is written about *records*, and §10.2.4 already concedes the SWD
attacker. For Critical: it is the exact material the feature exists to erase, on
the most likely walk-away screen, and the code affirmatively claims it is
handled. Either way it should be decided before the machine is flashed, not
after.

---

## MINOR

### M1 — `armed()`'s `Status()` call is load-bearing for the cut-end re-arm, and nothing says so. The obvious "make the predicate pure" refactor silently disables the wipe for the rest of the session.

**Location:** `gui/wipe_guard.go:41-51`; `gui/engraver.go:126-151`.

`armed()` reads `j.Status().State`. `Status()` is **not** an accessor: it drains
`e.progress` and `e.errs`, performs the `engraveRunning → Done/Stopped/Failed`
transition, and contains `if e.status.State == engraveRunning { e.Start() }`.
`wipe_guard.go`'s doc reads as a plain state check and mentions none of this.

That coupling is what makes the design work, and it is invisible:

- Operator starts a 21-minute cut, then leaves the screen alone. At 3:00 the
  screensaver activates (`armed == false`, so the saver branch, not the warning)
  and the flow is **parked inside `EngraveScreen.Engrave`'s `ctx.Frame`**.
- The job goroutine finishes and writes `errs`. `EngraveScreen` is parked and
  calls nothing.
- The **only** caller of `Status()` on that tick is `armed()`. It drains `errs`,
  flips the state to `engraveDone`, returns `true`, the `false→true` edge resets
  `a.idle.start` (`run_flow.go:181-186`), `idle` recomputes false, the saver
  drops and the flow resumes. §10.2.4 row 2's fresh 3:00 happens. ✔

Now apply the refactor the doc invites — `armed()` reads `j.status.State`
directly, "since it is only a predicate". The state never leaves
`engraveRunning`, `armed()` never returns true, `a.idle.active` stays true
forever, the flow is never resumed, `EngraveScreen` never calls `Status()`, and
**no wipe fires for the remainder of the session** with the seed resident. That
is the brief's "fails to fire", one edit away, with a green suite (no test parks
a real job under the screensaver).

Second half of the same coupling: `armed()` puts
`if e.status.State == engraveRunning { e.Start() }` on **every Run tick**.
Today unreachable — verified: `engraveRunning` is written in production at
exactly one place, `gui/engraver.go:108` inside `Start()`, which sets `e.errs` at
`:104` first, so `Start()`'s own `if e.errs != nil { return }` guard always
holds. (`run_flow_test.go:476-483` already documents this branch as a hazard for
*tests*.) But the branch's name is "Restart if requested", i.e. it exists so a
caller can request a restart by setting the state — and the first caller that
does will have an idle `Run` tick spawn the engraver goroutine, needle down, from
the screensaver.

**Suggested fix.** State both invariants at the `armed()` call site (or at
`Status()`), and pin the first with a test: a real `engraveJob` that completes
while `a.idle.active` is true must re-arm.

### M2 — Plate setup and seed verification are armed states; the 3:30 clock runs while the operator's hands are in the machine.

**Location:** `gui/wipe_guard.go:32-40` (spec-conformant); SPEC §10.2.4 amendment
2026-08-09.

Not a code defect — §10.2.4 as amended deliberately arms the hold-to-start and
plate-done screens. Raised because this is the pre-flight and the operator is
about to live with it:

- **Hold-to-start** is where the plate is seated and aligned. §10.2.2's own
  comment calls re-seating shifted steel "the machine's most ordinary recovery".
  That work involves no screen contact and routinely runs past 3:30.
- **`SeedScreen.Confirm`** is where an operator verifies twelve or twenty-four
  words against an independent record. 24 words in 3:00 is 7.5 s/word.

Cost of missing the 30 s warning is the whole session: passphrase + ~31 s KDF,
and on vector F **every not-yet-cut secret record is wiped too** (traced: the
`for _, i := range at` loop in `unlockSecretSession` has no `ctx.Done` break, so
the unwind walks the remaining records and each one's `defer p.WipeSecretAt(i)`
fires — correct for wiping, and it means walking away between cards costs all the
remaining cards).

**Suggested fix.** None to the code. A conscious go/no-go, and one line in the
operator notes, so it is discovered on paper rather than on the first real plate.

### M3 — Anything resting on the touch panel refreshes the one clock indefinitely.

**Location:** `gui/run_flow.go:150`; `cmd/controller/platform_sh2.go:398-418`.

`len(evts) > 0` is the primary refresh, and `processTouch` emits an event on
**any** change in `(touching, tp)`. A hand, a plate, or a tool left on the panel
produces a continuous stream as the reported point moves, and the 3:00 never
elapses — the timer is silently off for exactly the walked-away machine it
exists for. A perfectly still object produces one event and then none, so the
wipe does fire; the failure is condition-dependent and **unmeasured**.

Pre-existing as an input path (the screensaver has the same one), but the
screensaver was not a security control.

**Suggested fix.** A bench check on the real panel — rest an object on it, watch
whether `p.dirties`/the saver ever advances. If it does not, consider requiring
the refresh to be a *press/release transition* rather than any pointer motion.

*(Cleared while checking this: no non-touch production source can refresh the
clock. `p.wakeups` returns `evts` unchanged, so the engrave goroutine's
`defer e.pl.Wakeup()` and the NFC poller do not reset it; `p.stdin` is reachable
only under `//go:build tinygo && rp && debug` (`cmd/controller/debug_sh2.go:1`),
so a UART-attached host cannot postpone a wipe on a release build.)*

---

## NIT

### N1 — After a wipe the operator lands on the main menu with no statement that a wipe happened.

`runWithFlow` restarts the session and `uiFlow` draws `StartScreen`. On a device
with **no watchdog**, an operator returning to a machine that was mid-session and
is now at the main menu has no way to distinguish "the idle wipe fired" from "it
rebooted" — and a spontaneous reboot on this hardware is the alarming reading.
They will also re-enter the passphrase without knowing why. §10.2.4 does not
require a notice; one sentence carried into the first `StartScreen` after a wipe
would make the state legible.

---

## Verified clean — traced or machine-checked, no finding

**`wipeGuard.armed()` across every `engraveJob` state** (`gui/engraver.go:55-62`):

| state | `armed()` | correct? |
| --- | --- | --- |
| guard `nil` (no secret session) | false | ✔ §10.2.4 row 3 |
| `job == nil` (choice / seed / error screens) | true | ✔ row 1 |
| `engraveIdle` (hold-to-start) | true | ✔ amendment, deliberate |
| `engraveRunning` | **false** | ✔ row 2, needle down |
| `engraveStopping` | **false** | ✔ quit closed, goroutine still unwinding |
| `engraveStopped` (paused) | true | ✔ "stop … re-arms with a fresh 3:00" |
| `engraveFailed` | true | ✔ "failure … re-arms" |
| `engraveDone` | true | ✔ "completion … re-arms" |

*Stale/nil pointer:* `g.job = scr.job` is set before `Engrave` can start anything
(`unlock_session.go:200-205`, `:309-314`); `NewEngraveScreen` draws no frame, so
the registration gap is straight-line. `defer g.job = nil` runs *after* `Engrave`'s
own `defer s.job.Stop()`, so the only window in which the guard holds a job the
arm has finished with has it in `engraveStopping` (disarmed) and contains no
`ctx.Frame`. `ctx.wipe` is installed and removed by `unlockSecretSession` alone
(`unlock_session.go:82-84`), so it is nil for the public plate list and every
non-secret program.

**The one clock — all three refresh sources.**
`len(evts) > 0`: only touch (M3) and `debug`-only stdin can produce one.
`ctx.keepAwake && !armed`: **single caller**, `unlock_kdf.go:302`, in
`unlockDerive`, which runs *before* the guard is installed; the `&& !armed` term
means no screen can postpone an armed wipe even if a future one calls it.
Armed `false→true` edge: fires on the tick after the guard installs (starting the
3:00 at the first secret ChoiceScreen) and on every cut end; deliberately does
*not* fire on `true→false`, so starting a cut does not reset the pre-cut window —
correct, since the cut-end edge supersedes it.

**Arithmetic.** Warning at `a.idle.start + idleTimeout` (`run_flow.go:188`), wipe
at `+ wipeWarningDelay` = 3:30 (`:202`) — additive, matching §10.2.4's
2026-08-09 amendment. The warning branch re-arms `WakeupAt(now+1s)`, so the wipe
lands within [30 s, 31 s). `wipeWarningOp` clamps negative remaining.

**Clock monotonicity across a 21-minute engrave.** Clean, and checked against the
toolchain rather than assumed. TinyGo 0.41.1
`share/tinygo/src/runtime/baremetal.go:75-82` implements `time.now` as
`mono = nanotime()` with `sec`/`nsec` *derived from* `mono + timeOffset`, so
`time.Now()` carries a monotonic reading, `Time.Add` preserves it, and
`now.Sub(idleWakeup)` is a monotonic subtraction. There is no RTC and nothing in
this tree calls `AdjustTimeOffset`, so no wall-clock discontinuity exists to
begin with — and even one could not perturb the timer.

**Does the unwind reach every secret (vector F, 3 × ms1)?** Yes, traced. Wipe
during secret 2: `unlockSecretPlate(2)` returns → its `defer p.WipeSecretAt(2)`.
The `for _, i := range at` loop in `unlockSecretSession` has **no** `ctx.Done`
guard, so it proceeds to secret 3; `unlockSecretPlate(3)` registers its defer,
`ChoiceScreen.Choose`'s `for !ctx.Done` returns `(0,false)` without a frame, and
the defer wipes 3. Secret 1 was wiped when its plate left. Then
`defer ctx.wipe = nil` → `unlockPlatesOrNotice` (both arms return immediately on
`ctx.Done`) → `unlockPayloadFlow`'s `defer p.Wipe()` backstop → `uiFlow`'s
`for !ctx.Done` exits. Every nested loop on the path checks `ctx.Done`
(`Engrave` :2720 and :2744, `Choose` :1465, `SeedScreen.Confirm` :2353 and its
nested confirm, `showModal` :25, `unlockDerive` :232, `unlockSealedFlow` :376,
`unlockPassphraseFlow` :123, `uiFlow` :1612). *Not covered by a test end to end:*
every `runWithFlow` wipe test drives a synthetic `boundedFlow`, not the real
`unlockSecretSession`, so "wipe during secret 2 of 3 wipes 1 and 3" is traced,
not pinned.

**`ctx.keepAwake` cannot postpone a wipe.** See above — one caller, guarded.

**The wipe cannot fire mid-cut.** `armed()` is false for `engraveRunning` and
`engraveStopping`, and the warning branch is `if armed` inside `a.idle.active`,
so the screensaver — not the warning — is what takes the screen during a cut. The
first tick after the goroutine's `errs` write flips armed and resets the clock,
so the earliest possible wipe is cut-end + 3:30.

**Warning legibility at the real display size.** Measured at 480×320
(`cmd/controller/platform_sh2.go:34-35`) with the real styles
(`poppins.Bold25` title / `poppins.Regular16` body, `NewStyles`): title 248×36 at
y=8, body 455×113 at y=52, bottom edge **165 px of 320** — 155 px of clearance,
identical at every second count from 30 down to 0. "Touch the screen to keep it."
— the only affordance that prevents the wipe — is fully on screen, and the body
is not near `Warning.Layout`'s unscrollable-overflow trap that F-95 records for
§10.2.3. Program at `…/scratchpad/wipecheck/warn/main.go`.

**Touch-to-keep routing.** On the dismissing tick `a.idle.active` is still true,
so `ctx.Router.Events` is skipped and the touch is swallowed rather than routed
against `d`, which at that moment holds the **warning's** tag bounds. The clock
resets, `a.idle.active` drops, the flow redraws through `draw(content)` and
repopulates `d` before the next tick routes anything. No stale-widget hit.

**Shutdown racing a wipe.** `if ctx.Done || !yield() { return }` — a consumer
stop during the unwind is handled on the next session's first tick; a `return`
from the range body makes `yield(o)` return false, so the flow still unwinds
through every defer.

---

## Machine checks used (reproducible)

```
export PATH=/nix/store/6rlw3brby0v26n0164a1a2shgn8sv4h3-go-1.25.10/bin:$PATH
cd <scratchpad>/wipecheck && go run .        # I1: plaintext survives Buffer.Reset
cd <scratchpad>/wipecheck && go run ./warn   # warning layout at 480x320
```

Both build against the worktree via `replace seedhammer.com =>
/scratch/code/shibboleth/seedhammer-b2b`; **neither repo was modified.**
