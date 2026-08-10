# B2b hang reproduction attempt — Run-level re-entry after a wipe (host)

Agent: repro (Fable 5), 2026-08-09.
Target: `/scratch/code/shibboleth/seedhammer-b2b`, branch `b2b` at `e8e78f0`.
Deliverable test: `gui/run_reentry_test.go` (uncommitted, left in the worktree).

## Verdict

**The hang does NOT reproduce on the host.** The full hardware scenario —
real `uiFlow`, real sealed payload, real `runWithFlow` session loop, real
touch routing, the real §10.2.4 idle wipe at 3:00 + 30 s, session restart,
carousel re-entry, checkmark on Sealed Payload — completes in every
combination of the fidelity axes the host can model. Session 2 re-enters the
payload flow and keeps drawing frames.

This is a real finding, not a shrug: the host now runs the exact code path
the hardware dies on, with every gui-level component in play, and it works.
The cause is therefore overwhelmingly likely to live **below gui** — in the
TinyGo runtime or the device drivers — not in the flow/session logic the
other two debuggers are tracing. Details and discriminating experiments below.

## What was built

`gui/run_reentry_test.go` — `TestRunSealedPayloadReentryAfterWipe`, five
subtests over a scripted two-session drive:

| subtest | vector | wipe | NFC goroutine | press/release |
|---|---|---|---|---|
| `F idle-wipe nfc split` (hardware-verbatim) | F (0 pub / 15 secret) | real §10.2.4 timer, warning observed | yes | separate ticks |
| `F idle-wipe nfc` | F | real timer | yes | same batch |
| `F forced-wipe nfc` | F | `wipeNowHook` | yes | same batch |
| `D idle-wipe nfc` | D (5 pub / 1 secret, hash screen) | real timer | yes | same batch |
| `D forced-wipe bare` | D | `wipeNowHook` | no | same batch |

Each subtest: eight right-taps to Sealed Payload → checkmark → (hash notice
for D) → passphrase notice → twelve words typed key-by-key on the real word
keyboard (~60 taps, hit-tested against the drawn frame) → real 100,000-iteration
KDF (~200 frames) → park on the §10.2.2 Cut/Skip screen → wipe → session
restart → eight right-taps again → **checkmark on Sealed Payload** → assert
the re-entered flow draws its entry screen. All five pass (~0.3–0.5 s each,
whole gui suite green in 61 s, `go test ./gui/ -count=1`).

### The harness hole this closed (and why nobody had reproduced anything)

Run-level tests could never land a *click* before this. `runWithFlow` routes
touch via `ctx.Router.Events(d, ...)` where `d` is Run's own drawer, populated
only by `d.Draw(fb, ...)` — and `testPlatform.NextChunk` returns `(nil,
false)`, so Run's drawer never had a single hit target. Every prior Run-level
"tap" (`deadlinePlatform.tap`) worked only as a `len(evts) > 0` idle refresh.

Fix (`hitPlatform` in the test file): serve a **1×1 framebuffer** chunk per
`Dirty`. Input registration in `op.Drawer.draw` records `state.clip` without
intersecting the destination bounds (`gui/op/op.go`, `case opInput`), so hit
geometry stays exact while the pixel pass clips to one pixel. This makes
Run-level touch-driving of any real flow cheap, and is reusable beyond this
bug.

Also modeled: `fakeNFC` (a no-tag reader whose `Read` blocks until `Close`),
so `StartScreen.Flow`'s scanner goroutine and its deferred
`close(closer); r.Close(); <-closed` handshake run on every checkmark
dispatch, as on the SH2; and split press/release delivery (the panel's shape —
the hardware's last-ever frame being a *pressed-style* frame proves press and
release arrive on different ticks).

### The test fails loudly, not by hanging

Verified, not assumed:

- **Durable block** (the hang's expected shape — flow stops calling
  `ctx.Frame`): checked with a temporary self-test whose flow blocked on a
  channel after one frame. `synctest` failed it in **0.009 s** with a stack
  naming the exact block site (`chan receive (durable), synctest bubble`).
  That panic would have been the smoking gun had the hang reproduced.
- **Stall while still drawing**: the 5000-tick cap trips and the test prints
  the stalled step and the last frames — observed firing during development
  when split delivery outpaced the script (fixed by gating steps on an empty
  event queue).
- Unbounded residual: a flow that *spins on CPU* without drawing or blocking
  would only be caught by `go test`'s timeout. Ticks advance only between
  frames; `boundedFlow` cannot wrap the real `uiFlow`.

## What the host harness cannot model (where the bug must live)

1. **TinyGo runtime — allocator/GC.** Session 2's first act after the
   checkmark, *before any frame*, is `XIPReader.Read`
   (`seal/read_tinygo.go:56`): a 64 KB contiguous allocation — ~14 % of free
   heap per F-79 — into a heap carrying a full unlock cycle's fragmentation
   (KDF, 64 KB blob from session 1, plate splines, scrubbed-but-large
   `ctx.B`). A TinyGo alloc/GC stall there matches every observed symptom:
   pressed frame drawn, then nothing, no panic, deterministic, fresh boot
   fine. Host Go's allocator cannot model this.
2. **The real NFC device under `poller.Poller`.** `Poller.Close` must acquire
   `p.reading` and relies on `d.Interrupt()` unblocking a `Read` in flight;
   my `fakeNFC` is well-behaved by construction. But note the discriminator:
   the close/reopen cycle also runs on *every ordinary program entry/exit on
   a fresh boot* (`uiFlow`'s loop re-invokes `StartScreen.Flow`, which calls
   `NFCReader()` each time), so a close-handshake deadlock should hang normal
   use too, which it does not. Weak candidate, not zero.
3. **`Platform.AppendEvents` wakeup-channel semantics** (documented
   `deadlinePlatform` gap), real flash/XIP timing, the real ft6x36 event
   stream beyond clean press/release pairs, and TinyGo's cooperative
   single-core scheduler vs `synctest`.

## One-minute hardware experiment that splits the remaining theories

After a wipe, press checkmark on **Backup Wallet** instead of Sealed Payload:

- **Hangs too** → dispatch-generic (NFC close handshake / scheduler), since
  that path runs the same `Flow` teardown but never touches the payload.
- **Works** → payload-entry-specific, and the first payload-specific act is
  the 64 KB XIP read/allocation — pointing squarely at theory 1.

## Files

- Test (uncommitted, in the worktree):
  `/scratch/code/shibboleth/seedhammer-b2b/gui/run_reentry_test.go`
- Run:
  `nix develop /scratch/code/shibboleth/seedhammer --command bash -c 'go test ./gui/ -run TestRunSealedPayloadReentryAfterWipe -count=1 -v'`
- Nothing outside that one test file was added or modified; non-test code
  untouched; nothing committed. Pre-existing unrelated vet nit:
  `gui/freetext_sizeproof_golden_test.go:111` (`testing.ArtifactDir` vs
  `go1.25` directive) — not mine, not new.
