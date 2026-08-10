# Static trace: post-wipe Sealed Payload checkmark → missing next frame

Agent: static-trace debugger (one of three parallel angles). Repo: `/scratch/code/shibboleth/seedhammer-b2b`, branch `b2b` @ `e8e78f0`, read-only.
Date: 2026-08-09.

## Verdict up front

The checkmark→frame path contains **exactly two places that can block without ever
calling `ctx.Frame`**, both sitting *between* the pressed-style frame and the first
frame the unlock flow would draw. Everything else on the path draws every loop
iteration — the documented `!ctx.Done` spin hazard (gui/gui.go:1607-1612) does
**not** occur anywhere on this path.

1. **Most likely: `seal.XIPReader{}.Read()`'s 65,536-byte contiguous allocation
   fails post-wipe → TinyGo out-of-memory abort → permanent halt** (no watchdog).
   It is the only candidate that is *deterministic* and *post-wipe-only*.
2. **Second, a real latent deadlock either way: the NFC scanner shutdown defer in
   `StartScreen.Flow` can block forever in `Poller.Close`.** It fits the freeze
   point exactly but is probabilistic and boot-symmetric, so it fits the 2/2
   post-wipe-only reproduction poorly.

A one-command discriminator between them is given at the end. I could not prove
either from static text alone; do not fold a fix before the discriminator has run.

## The exact path from the checkmark press

1. Touch-DOWN is routed to `selectBtn`; `StartScreen.Flow`'s next iteration draws
   the nav button in pressed style (`layoutNavigation`, via `c.Pressed`). **This is
   the last frame ever drawn.**
2. Touch-UP → `Clickable.Next` (gui/widget.go:79-83) computes `clicked` on
   *release* → `StartScreen.Flow` returns `startScreenAction{prog: unlockPayload}`
   (gui/gui.go:1758-1759). No frame is drawn for the release.
3. **The function-exit defer runs first** (gui/gui.go:1701-1705):
   `close(closer); r.Close(); <-closed` — the NFC scanner goroutine shutdown.
   ← candidate 2 blocks here.
4. `uiFlow` dispatches `unlockPayloadFlow(ctx, th, payloadReader)`
   (gui/gui.go:1638-1639 → gui/unlock_flow.go:26).
5. `r == nil` guard (false here) → **`blob, err := r.Read()`**
   (gui/unlock_flow.go:38) = `seal.XIPReader{}.Read`
   (seal/read_tinygo.go:48-59): `make([]byte, 65536)` then copy out of XIP.
   ← candidate 1 dies here.
6. `o.Inspect(blob)` — bounded parsing only (seal/open.go; record loops capped by
   §6.2/§6.4). Same blob parsed fine minutes earlier on the same boot; ruled out.
7. First possible draw: `showNotice` (hash, if `p.HasHash`) or
   `unlockPassphraseNotice` — both via `showModal` (gui/slip39_polish.go:23-33),
   which calls `ctx.Frame` on every non-terminal iteration.

The observed screen (carousel + pressed checkmark, version line intact) proves no
draw happened after step 1, so the hang is inside steps 3-6, and step 6 is ruled
out. Every error path in step 5-6 leads to `showError`, which draws — even an
unreadable payload would have put a screen up. That pins it to **step 3 or step 5**.

## Candidate 1 (most likely): OOM abort in `XIPReader.Read`

### Mechanism

- `Read` allocates the whole region: `out := make([]byte, len(region))` with
  `len(region) = RegionLen = 65,536` (seal/read_tinygo.go:49,56; seal/wire.go:47).
  Per the F-79 comment (gui/gui.go:1587-1591) that is **~14% of free heap**, i.e.
  free heap ≈ 460 KB total.
- The device build is `tinygo build -target pico-plus2 -stack-size 16kb -gc
  precise -opt 2 -scheduler tasks` (.github/workflows/test.yml:29). TinyGo's
  precise GC is a **non-moving, non-compacting** mark-sweep block allocator: a
  64 KiB allocation needs that many *contiguous* free blocks, and no amount of
  total free space compensates once a long-lived object sits mid-hole.
- On allocation failure TinyGo runs a collection, retries, then
  `runtimePanic("out of memory")` → abort → infinite loop. `-scheduler tasks` is
  single-threaded and cooperative, so the abort halts *everything*: no frames, no
  touch, no §10.2.4 timer, no screensaver — the machine keeps displaying the last
  LCD frame (version line intact). **No watchdog** (settled fact) makes it
  permanent. Every observed symptom matches.

### Why it is post-wipe-only and deterministic

- Fresh boot: near-virgin heap; the first 64 KiB allocation trivially succeeds
  (observed: fresh boot unlock works).
- The failing sequence: blob #1's 64 KiB was freed *before* the plate session
  (`clear(blob); blob = nil`, gui/unlock_flow.go:110-111 — deliberate, F-79), so
  its hole was open for reuse during ~3.5 min of session frames plus the
  30-second §10.2.4 warning window. Allocations that *survive the wipe by design*
  — `a.warnBuf` (grows precisely during the warning countdown,
  gui/run_flow.go:208-210), `a.mask`, the `op.Drawer` `d`, all declared above the
  session loop ("everything above this line survives", gui/run_flow.go:43-45) —
  and the post-wipe session's own live allocations (ctx #2, its growing `op.Buffer`
  during carousel navigation) can land inside that hole. A non-moving collector
  then cannot produce 64 KiB contiguous again. Same operation sequence → same
  allocation sequence → same layout → **deterministic**, on every boot.
- The freeze lands exactly where observed: `r.Read()` is the first substantial
  allocation after the click, before any frame.

### Minimal fix

Allocate the region buffer **once at boot** (at `uiFlow`'s §10.1 probe, while the
heap is still unfragmented), retain it for the GUI's lifetime, and have
`XIPReader.Read` fill it instead of `make`-ing; `clear()` it after each session
instead of dropping it. This deliberately re-accepts F-79's ~14% cost in exchange
for determinism — the alternative (parse in place over the XIP mapping, copying
only records, which `seal.AdmitSection` already does) avoids the cost but changes
the AAD/ciphertext aliasing assumptions in `unlockAttemptOnce` and is not the
minimal change.

## Candidate 2 (real bug regardless): the NFC shutdown defer can block forever

`StartScreen.Flow` spawns a scanner goroutine whenever
`ctx.Platform.NFCReader() != nil` (gui/gui.go:1698-1747) — on SH2 hardware that is
always (`poller.New(p.nfc)`, cmd/controller/platform_sh2.go:565-567, a **fresh
`Poller` per Flow entry over the same shared `nfcDev`**). The exit defer
(gui/gui.go:1701-1705) is `close(closer); r.Close(); <-closed`, and it runs on the
flow goroutine, before any next frame. Two ways it never returns:

- **`Poller.Close` blocks on `p.reading <- struct{}{}`**
  (nfc/poller/poller.go:92-100). Close's unblocking mechanism is a *single*
  `d.Interrupt()` token (`cancel` chan, cap 1, driver/st25r3916/st25r3916.go:304-309),
  consumed by whichever `waitForInterrupt` select sees it first
  (st25r3916.go:407-414 → `io.EOF`). Most consumption points propagate the EOF
  out of `Poller.Read` — but **`Poller.poll` swallows it**: a type5 error falls
  through to the ISO14443a attempt (poller.go:107-110), and a type2 error is
  deliberately discarded (`// Ignore read errors.` → `return nil, nil`,
  poller.go:114-118), after which `Poller.Read` loops (`continue`, poller.go:79-80)
  with the token gone. In the no-tag steady state `Poller.Read` **never returns
  naturally** — the driver enables the wake-up *timer* interrupt (`i_wt` in the
  mask, st25r3916.go:229-231; period configured at reset, st25r3916.go:127), so
  the goroutine perpetually cycles Detect → 700 ms listen → field-on → poll. If
  the token lands inside a poll window, `Read` cycles forever holding the token
  and `Close` blocks forever → no frame ever again, all timers dead. Exactly the
  observed deadness.
- **`<-closed` race**: the goroutine checks `closer` only at its loop top
  (gui/gui.go:1709-1715). If it passes that check just before `close(closer)` and
  then blocks acquiring `p.reading` (poller.go:51) after Close's fast path took
  the token (poller.go:94), nothing ever releases the token; the goroutine never
  reaches `close(closed)`; the defer waits forever. Narrow window; noted for
  completeness.

**Why it fits the repro poorly:** the mechanism is identical on a fresh boot
(checkmark #1 runs the same defer and was observed to work, four times across the
runs), and its probability per close is the fraction of the scanner cycle spent in
the swallow windows (~tens of ms of a ~800 ms+ cycle). It has no post-wipe-true /
fresh-boot-false condition that I could find; the only state that persists across
the wipe (`nfcDev.iso15693`, `Device.extField`, a possibly-undrained `cancel`
token) produces extra failed scans, not a deterministic hang. It should be fixed,
but it is probably not *this* bug.

### Minimal fix

Make cancellation sticky instead of one-shot: give `Poller` a `closed` flag (or
closed channel) that `Close` sets before taking the token, checked at the top of
`Poller.Read`'s `for` loop and at `poll`'s error-ignore path (propagate `io.EOF`
instead of swallowing when closing). That removes both block modes; alternatively
(coarser) move the close+join off the flow's return path so the UI can never wait
on the scanner.

## What else was checked and ruled out

- **All GUI loops on the path draw or exit every iteration**: `uiFlow`,
  `StartScreen.Flow`, `showModal`/`ErrorScreen.Layout` (gui/gui.go:268-276),
  `unlockWarnUnauthenticated`, `unlockPassphraseFlow`, `unlockSealedFlow`,
  `unlockDerive` (frames per 500-iteration KDF slice, with `KeepAwake` +
  `WakeupAt` before `Frame` — F-93 fix present), `holdToConfirm`. No
  return-without-drawing spin exists on this path.
- **Stale-event skip-through**: impossible on hardware. SH2 emits only
  `PointerEvent`s (platform_sh2.go:398-418); pointer events are routed against
  tags that are pointers to each screen's freshly allocated `Clickable`s
  (gui/event.go:296-331, gui/widget.go:70), so a queued event can never dismiss a
  screen that has not drawn. Button events would bypass tags, but nothing on SH2
  emits them except debug stdin.
- **Run's inner loop parked** (screensaver/warning branches `continue` without
  yielding to the flow): requires 3 min idleness; the click had just been
  delivered, and the saver would have drawn. Also contradicts the settled fact
  that no frame reached the consumer.
- **`ctx.Done` set → flow unwound → `Run` returned** (main.go:34 would fall
  through and the firmware would halt "cleanly"): production `yield` never
  returns false and `wipeNowHook` is nil; the §10.2.4 timer had just been reset
  by the click's events. Nothing on this path sets `Done`.
- **`o.Inspect`**: bounded loops only; identical input succeeded on the same boot.
- **`wipeGuard` / `seal` package state**: `ctx.wipe` is per-Context (nil until
  `unlockSecretSession`), hooks are nil in production, `Opener` and `XIPReader`
  are stateless values.

## Could NOT rule out (honestly)

- I cannot *prove* the OOM statically — heap layout is runtime behavior, and I
  did not measure TinyGo's block-allocator granularity or the actual live set.
  The theory rests on: non-moving GC + a freed-then-reoccupied 64 KiB hole +
  deliberate cross-session survivors.
- I cannot fully exclude a deterministic bias that parks the scanner goroutine in
  a swallow window post-wipe (st25r3916 interrupt semantics — that `i_wt` fires
  every wake-up period — are inferred from the mask configuration and datasheet
  reading, not measured on silicon).
- Anything outside the static trace (DMA/LCD, i2c bus wedge, touch controller)
  is the other debuggers' angle; not examined here.

## One-command discriminator (for the hardware angle — run before any fix)

At the hang:

1. **Serial console**: `panic: runtime error: out of memory` → candidate 1.
   Silence → candidate 2.
2. **NFC field**: hold a phone/tag near the antenna. Field still pulsing →
   scheduler alive, UI goroutine blocked in the defer → candidate 2. Field dead →
   everything halted → candidate 1.
3. **No-wipe control**: on a fresh boot, enter Sealed Payload, back out at the
   passphrase prompt, re-enter, repeatedly (each entry re-runs the 64 KiB
   `r.Read()` *and* the NFC defer). Hang without any wipe → heap churn suffices
   (still candidate 1, wipe incidental) or candidate 2's race; never hangs →
   the wipe session's survivors are the fragmenting allocation, confirming the
   post-wipe mechanism.
