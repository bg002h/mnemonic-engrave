# B2b post-wipe re-entry hang — surviving-state angle

Agent: surviving-state debugger (one of three; the others trace the entry path statically and build a host repro).
Firmware examined: `/scratch/code/shibboleth/seedhammer-b2b`, branch `b2b` @ `e8e78f0`, read-only.
Question assigned: what state survives `runWithFlow`'s session loop — or lives outside it — and differs on session 2?

## Verdict, one sentence

The flow is not wedged in any payload/seal code at all: it is parked forever inside
`StartScreen.Flow`'s deferred NFC-scanner shutdown (`close(closer); r.Close(); <-closed`,
gui/gui.go:1701-1705), which runs *between* the pressed-button frame and
`unlockPayloadFlow`'s first frame, blocks on channels shared with a scanner goroutine,
and can deadlock permanently via state that lives on `Platform.nfc` — a singleton created
once at boot (cmd/controller/platform_sh2.go:305-306) that no session boundary resets.

## Why this is the only place the flow can stop calling ctx.Frame

The three dead symptoms (no redraw, no touch, no 3:00/3:30 timer) are one fact: the
§10.2.4 timer, touch routing and screensaver all live in `runWithFlow`'s inner tick loop
(gui/run_flow.go:129-230), which runs only inside a `ctx.Frame` yield. The flow stopped
yielding. Additionally, `a.idle.active` cannot be stuck true (the saver would then be
*drawing*, and it isn't), and session 2's `ctx.wipe` is nil until `unlockSecretSession`
(never reached), so if the inner loop were running at all, the plain screensaver would
have fired at 3:00. It didn't. The flow is therefore blocked in straight-line code.

Enumerate that straight-line window. The pressed-style frame is drawn by the iteration
*before* `selectBtn.Clicked` returns true (gui/gui.go:1757-1760). Between that return and
the next `ctx.Frame` (the step-3 hash notice, an error screen, or the passphrase screen —
any of which is a visible screen change, and none appeared) the executed code is exactly:

1. `StartScreen.Flow`'s deferred `close(closer)`, `r.Close()`, `<-closed` (gui/gui.go:1701-1705) — **two indefinite channel blocks**;
2. `uiFlow`'s switch dispatch (gui/gui.go:1638-1640) — no blocking;
3. `unlockPayloadFlow` prologue (gui/unlock_flow.go:34-97): nil check, `r.Read()` (a bounded XIP memcpy, seal/read_tinygo.go:48-59 — settled stateless), `o.Inspect(blob)` (pure CPU on a zero-value `Opener` over bytes freshly copied from flash, which the wipe never touches — byte-identical to session 1's input, which parsed fine), `defer` registrations.

The only constructs in that window that can block for minutes are (1). On a TinyGo
cooperative scheduler with no watchdog, a parked main goroutine stays parked forever.

## The deadlock mechanism, traced

`r` is `poller.New(p.nfc)` (platform_sh2.go:565-567): the *Poller* is fresh per Flow
entry, but the wrapped `*nfcDev` → `*st25r3916.Device` is the boot-time singleton.

- `Poller.Close` (nfc/poller/poller.go:92-100): if the scanner goroutine holds the
  `p.reading` token (it does essentially always — it lives inside `Poller.Read`'s waits),
  Close calls `p.d.Interrupt()` and then **blocks on `p.reading <- struct{}{}` until
  `Poller.Read` returns. Close never releases the token afterwards.**
- `Interrupt` posts one token into `d.cancel` (cap 1, on the Device —
  driver/st25r3916/st25r3916.go:304-309). The *only* consumer is `waitForInterrupt`
  (st25r3916.go:407-414), which turns it into `io.EOF`.
- Where that EOF is consumed decides everything:
  - `Detect`'s wake-up wait (st25r3916.go:238-241): bare EOF propagates → `Poller.Read`
    returns → token released → Close completes → goroutine sees the already-closed
    `closer` at its loop top (gui/gui.go:1710-1715) → `close(closed)` → clean.
  - `Detect`'s listen wait (st25r3916.go:263) or the type4-emulator path: wrapped error /
    EOF propagates → same clean exit (worst case +1 s via scan.go:47's scanFailed sleep,
    or +10 s via `fieldOnTimeout`).
  - **`poll()`'s tag probes (nfc/poller/poller.go:103-120): the EOF is swallowed.**
    A `type5.NewReader` failure falls through to the ISO14443a probe; a `type2.NewReader`
    failure returns `(nil, nil)` — "Ignore read errors" — and the loop `continue`s.
    The cancel token is now *gone*, and the no-tag polling cycle is error-free by design
    (Detect tolerates `errTimeout` at :263-265; probe failures are silent). So
    **`Poller.Read` never returns, `Close` blocks forever on `p.reading`, the UI
    goroutine never reaches `<-closed`, and no frame is ever yielded again.** Both
    probes contain real suspension points while holding the token (`time.Sleep` at
    type5.go:46 and type2.go:31, plus each response wait), so the cancel genuinely can
    land there.

This mechanism produces a permanent, total freeze whose first casualty is the frame after
the pressed-button frame — matching every observed symptom, including "the wipe itself
worked instantly" (the wipe path never touches NFC; the carousel then *reopens* the
scanner, and the hang strikes at the next dispatch's *close*).

Supporting fact: no host artifact can reach this code. The emulator returns a nil
NFCReader (cmd/emu/platform.go:190) and so does the test platform's default
(gui/gui_test.go:354, :437), so `StartScreen.Flow` never spawns the scanner goroutine
off-hardware. Four R0 rounds, the preflight and 16/16 mutation kills never executed one
line of this shutdown. It is hardware-only code.

## Why fresh boot works and post-wipe doesn't — and an honest gap

What actually distinguishes the runs is **close ordinal and device state, not the wipe**.
Fresh boot → first dispatch is close #1 on a power-on-default chip; the observed hang is a
later close on a device whose `extField` latch, `prot`, `nfcDev.iso15693`, chip registers
(left configured-but-disabled by `Close`'s `regOpCtrl=0`, st25r3916.go:297-302) and
possibly a stale `cancel`/`interrupts` token all carry over — the Platform singleton is
the one thing `for { ctx := NewContext(pl) ... }` cannot reset. The wipe is how this
session history produced a *second* dispatch at all.

The gap I cannot close from source: my timing estimate puts the goroutine inside the
swallow window only a few percent of each ~0.9 s no-tag cycle (wake-up wait + 700 ms
listen wait dominate; the probes are ~10-30 ms). That predicts an intermittent hang, not
a deterministic one. Two readings are consistent with the evidence:
(a) the hang is a per-close race whose observed "determinism" is small-n, or
(b) surviving device state (a latched `extField`, a stale token consumed at a
phase-shifting moment, or chip-register state altering wake-up cadence) biases the
post-close cycle so the goroutine parks in the probe phase — which only instrumented
hardware or a faithful host device model can decide. Either way the *freeze point* and
the *mechanism* stand; only the hit-rate model is open.

Discriminating tests (cheap, non-destructive):
1. Fresh boot → enter Sealed Payload → **Back out normally** (no wipe) → re-enter. If it
   hangs, the wipe is fully exonerated; it is the second close.
2. Fresh boot → wipe → checkmark on **BIP-85** (or any flow) instead. This mechanism
   predicts the hang is not payload-specific: any post-wipe dispatch crosses the same defer.
3. One `log.Printf` before/after `r.Close()` and before `<-closed` in `StartScreen.Flow`
   pins which of the two blocks it is (on a debug build; serial attached).

## The assigned enumeration (Q1-Q7)

1. **`a` struct** (run_flow.go:19-36). Reset at session head: `idle.start`, `idle.active`
   (:48-52), `armed` (:53). Carried over: `mask` (content-independent scratch alpha,
   reallocated on size change, :101-104 — safe), `warnBuf` (`Reset()` precedes every use,
   :208 — safe), `idle.state` (re-zeroed on every idle false→true edge, :190-194 — safe).
   None of these run in the flow; none can stop `ctx.Frame` being called. Ruled out.
2. **`d *op.Drawer` / `stats`** (:41-42). Between sessions `d` holds session 1's last
   frame tree (including refs into the ctx.B that `Scrub` zeroes at :245). Harmless:
   the range body calls `draw(content)` — which starts with `d.Reset()` (:89) — *before*
   the inner loop can call `ctx.Router.Events(d, …)` (:175), so session 2 never
   hit-tests a stale frame. Worst conceivable failure is one misrouted click, and the
   observed pressed-style render proves routing worked. Cannot stop the flow yielding.
   `stats` is debug-only. Ruled out.
3. **`evts`** — declared *inside* the session loop (:83). Fresh each session; reused only
   across ticks via `evts[:0]`. Ruled out.
4. **Package-level `gui` state.** All `…Hook` vars are nil in production (grepped
   exhaustively). Remaining package vars are error sentinels, constant tables, themes
   written once at init (gui/theme.go), and `newDeriver = seal.NewDeriver` (never
   reassigned in production). No cross-session mutation. Ruled out.
5. **`seal` state.** Package level: error sentinels and one nil test hook
   (seal/unlock_key.go:25) only; no cache, pool, or `sync.Once`. `Opener` is a per-entry
   zero value (unlock_flow.go:63); `Payload` is per-session; `p.Wipe`/`WipeSecretAt`
   zero only that Payload's slices; `clear(blob)` zeroes the RAM copy while
   `XIPReader.Read` re-copies from untouched flash, so session 2's `Inspect` input is
   byte-identical to session 1's successful one. Ruled out.
6. **`ctx.wipe` / `keepAwake`.** Fresh `Context` per session (:47); `wipe` is nil until
   `unlockSecretSession` (unlock_session.go:82-84) — never reached in the hanging
   session, so `armed()` is the nil-receiver false path (wipe_guard.go:41-44), which is
   pure field reads and cannot block. The old guard's only field is `job`; no engrave ran,
   so it was never set, and the old Context is unreachable from session 2. `keepAwake` is
   cleared every tick by `ctx.Reset()`. Ruled out.
7. **TinyGo-specifics.** No `sync.Once`/`sync.Pool` outside third_party; package inits
   build constant tables (e.g. type4's capContainer). `op.Buffer.Scrub`
   (gui/op/buffer_len.go:23-28) is bounds-safe (`clear` within `cap`) — no heap
   corruption vector. The two TinyGo facts that *matter*: the scheduler is cooperative
   and non-preemptive, so a main goroutine parked on a channel is parked forever with no
   watchdog to reap it; and **goroutines and Platform singletons are precisely the state
   the session loop cannot reset** — which is where the finding lives. The shared I2C bus
   (`multiplexI2C`, platform_sh2.go:689-706) serializes NFC vs the power monitor via a
   channel token; a goroutine dying while holding that token would also freeze the bus,
   but nothing on this path panics between take and put. Noted, not implicated.

## Ranking

1. **NFC scanner shutdown deadlock via the session-surviving `p.nfc` device (cancel-EOF
   swallowed in `poller.poll`)** — explains the freeze location to the exact statement,
   the pressed-frame-then-nothing signature, the simultaneous death of timer + touch +
   saver, the permanence, why the wipe itself was flawless, and why nothing off-hardware
   ever saw it. Open item: whether surviving device state makes the swallow-window hit
   deterministic, or the determinism is small-n (tests 1-3 above discriminate).
2. Nothing else. Every other enumerated candidate is affirmatively ruled out above with
   the mechanism it would have needed and the code that forecloses it.

## Notes for the other two agents

- Static entry-path agent: the defect is *before* your path starts — the defer between
  `StartScreen.Flow`'s return and `unlockPayloadFlow`'s entry. Treat goroutine shutdown
  as part of the entry path.
- Host-repro agent: the emulator's nil `NFCReader` means the default harness cannot
  reproduce this. A deterministic repro: a fake `poller.Device` whose `Detect` returns
  `(true, nil)` and whose probe-phase `Read` swallows the post-`Interrupt` EOF the way
  `poll()` does — then `Poller.Close` hangs on `p.reading` every time.

## Fix directions (not folded — repo is read-only for this task)

Smallest safe fix set, any one of which breaks the deadlock: check `closer` inside
`Poller.Read`'s `continue` path (or pass a cancel the probes cannot swallow);
make `poll()` propagate `io.EOF` (`errors.Is`) instead of swallowing it; or bound the
gui-side `<-closed` with a timeout + explicit error surface. §10.2.4's own lesson
applies: a shutdown that can silently never finish on a watchdog-less device holding
seed material is itself a §2.2-class hazard.
