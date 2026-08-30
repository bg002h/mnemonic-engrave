# BUG — "back hangs the device" on the pathological vault

Investigated in the headless sim against fork `main` **a0c1615** (which contains
F-76/F-437/the r1 fold). Repo left **byte-identical**: everything below ran via
`go test -overlay`, with the harness in the scratchpad
(`…/scratchpad/bug/zz_bug_test.go`, `overlay.json`, `overlay-pre.json`).

Fixture: `design/journeys/out/pathological/backup-strings.txt` (**36 records: 6
md1 chunks + 30 mk1 chunks = 1 policy card + 11 key cards**) packed with
`me sysw pack --no-passphrase --in … --out …`. `sysw.Open` reports 36 public /
0 secret; `me` emitted no "could not decode" warning, so every card is complete.

---

## DIAGNOSIS — neither a leak nor a livelock. A dismiss-only modal that ignores BACK.

**`showError` modals dismiss on Button3 (checkmark) ONLY. Button1 (back) does
nothing on them, forever.** `ErrorScreen.Layout` (`gui/gui.go:353-357`) sets
`s.ok.Button = Button3` and binds no other control; `showModal`
(`gui/slip39_polish.go:23-33`) loops on it until that one button fires.

The screen the operator names is exactly one of these:
`bundleAbortWarning` → `showError(ctx, th, "Bundle Incomplete", …)`
(**`gui/bundle_flow.go:689-692`**) — *"Stopped at card 1 of 12 (md1
descriptor). This set is not a usable backup yet…"*

**Measured** (`TestBugAbortModalIgnoresBack`): walked to the pre-engrave screen,
backed out to the modal, then pressed **BACK 30 times**:

```
ABORT MODAL: "Stoppedatcard1of12(md1descriptor).Thissetisnotausablebackupyet…"
AFTER 30 BACK PRESSES the screen is STILL: "Stoppedatcard1of12(md1descriptor)…"
Button3 dismissed it and walletPolicyFlow RETURNED
```

That is a complete, self-sufficient explanation of *"I'm hung on bundle
incomplete screen"*: the device is alive and redrawing, and the button the
operator is pressing is not wired to anything on that screen.

It is a **class**, not one screen: `gui` has **143** `showError`/`showNotice`
call sites, every one of them BACK-deaf.

### It is not new, and it is not F-76's

Pre-F-76 control (`TestBugPreF76DoorReachesTheGather`, `e456970`'s
`gui/wallet_policy.go` overlaid onto the current tree):

```
PRE-F76 DOOR ok=true: "Firstcardfromwhere?FROMPAYLOADENTERITInput"
PRE-F76 GATHER counted the card = false: "md1descriptors:0mk1keys:0"
PRE-F76 after Done: "Droppedanincompletecard:thepayloaddoesnotcarryallofits
                     chunks.Rewriteitonthehostwith`mesyswpack`toincludeit."
```

So on the old door this payload dead-ends **one screen earlier**, on *another*
`showError` modal with the identical BACK-deaf trap. F-76 did not create the
trap; it moved which modal the operator meets and made the engrave route
reachable at all for this wallet.

---

## THE CYCLE — walked 25 times, no hang, no state loss, no leak

`TestBugOperatorCycle`, one `Context` and one `syswSession` for the whole run
(as on the device). Each cycle: door → FROM PAYLOAD → gather → consent →
bundle review → engrave picker → **the pre-engrave screen** → back out
(back, back, then Button3 to clear "Bundle Incomplete") → flow returns.

**Every one of 25 cycles completed the full walk.** No cycle failed to count the
card, no cycle hung, and the flow returned after exactly 3 presses each time.

### 1. No state consumption bug

`session records=36 seeds=0` after every single cycle. `takeAll` filters, it does
not consume (`gui/sysw_session.go`), so `cardSet` hands out the same 36 records on
every pass, and `ctx.syswBundleSeeds` is correctly emptied by the gather and
re-filled by the next door. Pass 2 reached `md1 descriptors: 1 / mk1 keys: 11`
exactly as pass 1. **"Going back should lose nothing" holds on this route.**

The operator's "you'll go through bundle incomplete screen" is not a state
divergence: that modal is the *normal* terminus of backing out of an engrave, on
every pass including the first.

### 2. No per-cycle leak

Retained heap (`runtime.GC()` ×2, then `ReadMemStats`), `HeapAlloc`, from one
captured 25-cycle run (`…/scratchpad/bug/cycle25.log`):

| point | HeapAlloc | HeapObjects | delta vs cycle 1 |
| --- | --- | --- | --- |
| before any cycle | 649,680 | 1,473 | — |
| after cycle 1 | 1,682,048 | 1,512 | (**+1,032,368** vs baseline) |
| after cycle 2 | 1,682,064 | 1,513 | +16 |
| after cycle 3 | 1,687,752 | 1,521 | +5,704 |
| after cycle 5 | 1,688,568 | 1,527 | +6,520 |
| after cycle 10 | 1,690,696 | 1,535 | +8,648 |
| after cycle 15 | 1,692,392 | 1,542 | +10,344 |
| after cycle 20 | 1,694,024 | 1,550 | +11,976 |
| after cycle 25 | 1,694,904 | 1,556 | +12,856 |

**Cycles 2→25: +12,856 bytes over 24 cycles = 536 B/cycle**, and +44 objects
total — flat enough to be harness noise (a second run measured 760 B/cycle with
two cycles at **+0**). The cycle-1 step reproduces at ~1.03 MB in both runs.
Test result: `PASS`, zero failed assertions, so all 25 cycles reached every
screen including `md1 descriptors: 1 / mk1 keys: 11`.

### 3. The one-time +1.03 MB is HOST-ONLY — it does not exist on the device

`TestBugRetentionProfile`: cycle 1 retains **+1,021,120 bytes in only +22
objects**. `go tool pprof -inuse_space`:

```
1133.89kB 26.89%  github.com/decred/dcrd/dcrec/secp256k1/v4.init.init.func2.func3
       ← hdkeychain.(*ExtendedKey).Derive → ScalarBaseMultNonConst
                                          → scalarBaseMultNonConstFast
```

That is secp256k1's lazily-loaded base-point table: `bytePointTable
[32][256]JacobianPoint` = 32·256·3·(10×uint32) = **983,040 bytes**, decompressed
on first use behind a `sync.Once` and kept for the process lifetime
(`loadprecomputed.go:27-90`). Clearing `ctx.B` freed only 37,400 bytes, so the
op buffer is not the retainer.

**It is compiled out on the MCU.** The dispatch is build-tagged:

- `curve_precompute.go` — `//go:build !tinygo` → `scalarBaseMultNonConstFast` (the table)
- `curve_embedded.go` — `//go:build tinygo` → `scalarBaseMultNonConstSlow` (no table)

Corroborated by the device build's own size report: `secp256k1/v4` occupies
**801 B rodata / 200 B data / 140 B bss** in the pico-plus2 image. So this
number must **not** be carried to the 450 KB heap budget — it is an artifact of
measuring on x86.

With it subtracted there is nothing left to exhaust a heap: ~536 B/cycle of
harness noise. At that rate the 450 KB device heap would take on the order of
800 cycles, against the operator's observed 2-5 — the leak hypothesis does not
fit the observation by three orders of magnitude.

### 4. TinyGo on exhaustion panics; it does not spin

`-gc precise` uses `gc_blocks.go`, which calls `runtimePanicAt(…, "out of
memory")` (lines 407, 456). An OOM would abort, leaving whatever was last drawn
frozen on the panel — it would **not** keep redrawing a modal that responds to
one button. The observed symptom (a live screen ignoring one button) matches the
BACK-deaf modal; it does not match OOM.

---

## Not reproduced: the original single-back-press report

The first report ("at the Wallet policy info screen showing change addresses,
back hangs") did not reproduce in isolation, and the numbers say why it would
not:

- The Wallet Policy **consent** screen carries the address proof as lines
  (`Receive 0/1`, `Change 0/1` on page 2 of the paged screen). Building the whole
  consent surface for this 11-key miniscript vault — `ExpandWalletPolicyChunks`,
  `FormAwareIdChunks`, `complexAddressSource`'s probe **plus 4 derivations** —
  took **8.9 ms** on x86 (36 ms including the gather). Paging is **~1.2 ms**.
- BACK from that screen returned to the gather in **10.1 ms**, correctly, with
  the cards intact (`md1 descriptors: 1 / mk1 keys: 11`).

So the consent path is neither a freeze nor a re-derivation storm. The screen
titled *"Change addresses"* (`addressListFlow`) is **not reachable from Wallet
Policy for this wallet at all** — it hangs off `md1PolicyFlow` (Inspect) and off
`DescriptorScreen`, and this policy resolves to *"Complex policy - display
only."* before either. The operator was most likely on the consent screen's
change-address page, and what they read as a hang is the same BACK-deaf modal one
step later.

Derivation counts and x86 timings, for the record: consent = **1 probe + 4
address derivations** per build, 8.9 ms total; `addressListFlow.recompute()`
derives one address per line that fits and re-derives the whole page on
toggle/page, but back triggers **zero** re-derivation.

---

## MINIMAL FIX PROPOSALS (not implemented)

**D1 — the real defect. Make the dismiss-only modal accept BACK.** In
`ErrorScreen.Layout` (`gui/gui.go:353-357`) bind a second `Clickable` on
`Button1` that dismisses identically, and draw it in the nav row with
`assets.IconBack`. One screen, one meaning: on a dismiss-only modal, BACK and OK
are the same action, so wiring both costs nothing and removes a dead control.
Fixes all 143 sites at once. A test must press BACK on a `showError` and require
it to return — that is precisely the assertion `TestBugAbortModalIgnoresBack`
already encodes, inverted.

*Alternative if a bare `Button1` binding is considered too broad:* bind the
back **AltButton** on the existing `ok` clickable, which changes no layout.
Weaker, because the nav row still shows one glyph and the operator still cannot
see that BACK works.

**D2 — the honesty half (Minor).** "Bundle Incomplete" is a terminus that ends
the program, but nothing on it says so. The operator pressing BACK is asking to
*go back*, and the screen's own text ("Finish a set in one sitting, or start
over") implies a route it does not offer. Once D1 lands, add one closing line
naming what dismissing does (returns to the menu).

**Not proposed:** any allocation change, any `cardSet`/`takeAll` change, any
consent-screen caching. The measurements above do not support a leak, a
consumption bug, or a derivation-cost problem, and a fix aimed at those would be
aimed at nothing.

## What this does NOT establish

- No hardware run. Everything here is the host sim; the device may still hold a
  defect these measurements cannot see.
- The 25-cycle run drives one specific back-out route. A different route out of
  `bundleEngrave` (e.g. aborting mid-plate on real hardware, where
  `releaseResumeState` and the engraver are live) is unmeasured.
- Whether the operator's device was additionally short of memory is not
  answerable from x86; what *is* answerable is that the largest candidate
  retainer is `!tinygo`-only and the per-cycle retention is ~0.

---

# APPENDIX A — report 2: "bundle at the top and just the last fp of a key"

**Screen identified, exactly.** It is `bundleReviewFlow`'s LAST PAGE
(`gui/bundle_flow.go`). For the pathological vault the review is 12 cards, and
the final page holds one orphaned line. Measured, verbatim from the sim:

```
BUNDLE REVIEW page 1: "Bundle12cardsverified:1.md1descriptorOKP2WSHcomplex2.mk1keyOK…"
  page 5: "Bundlemainnet|m/48h/0h/0h/2h|fp2864500611.mk1keyOKmainnet|m/48h/0h/1h/2h…"
  page 6: "Bundle|fp28645006"          ← the operator's screen, word for word
```

**Its BACK is bound and it works.** `bundleReviewFlow` binds Button1→back,
Button2→page, Button3/Center→continue, and back returns `false`, which
`walletPolicyFlow` turns into `continue` → the gather. Measured from the last
page:

```
=== on the last page, pressing BACK ===
BACK: frame 0 after 799.614µs: "WalletPolicymd1descriptors:1mk1keys:11Done…"
BACK went to the GATHER screen -- back works here
```

**So it is NOT a second BACK-deaf class.** It is not one of F-440's 143 sites
either — it is its own paged screen. Checked: every paged screen on this route
(`bundleReviewFlow`, `confirmReviewScreen`, `md1PolicyFlow`, `mk1DisplayFlow`,
`addressListFlow`, the unlock plate list) binds Button1 and returns on it. The
BACK-deaf class is `ErrorScreen` and only `ErrorScreen`.

What this screen *does* have is a cosmetic defect worth a follow-up: a final
page rendering a bare `| fp 28645006` with no card number and no context. The
paging is gap-free by construction, so it is a presentation artifact, not a lost
line.

**The hang the operator hit from here is Appendix B's**, not this screen's.

---

# APPENDIX B — reports 1 & 3: a PERMANENT hardware lock on the BACK edge

**This is a Critical, and it is a different defect from F-440.** F-440 is a
dead button on a live screen. This is a live-looking screen with a **dead
device**: no frames, no input, no recovery, power cycle only.

## What the field facts constrain

| field fact | what it excludes |
| --- | --- |
| "The screen updated every time I pressed right arrow" | derivation cost — the UI was fast right up to the edge |
| "only hung the moment I hit back arrow" | anything gradual; the lock is EDGE-TRIGGERED on BACK |
| "still locked at 2 minutes" … "at 3 minutes" | a backlog being drained |
| "the CHECKMARK does nothing either" | a drawn-but-unwired screen; **the event loop is not running** |
| sim returns from the same BACK in 0.8-10 ms | anything the sim's platform implements |

Both halves of the last row matter: the lock is in something the sim **stubs**.
`testPlatform.NFCReader()` returns `nil`, and `startScanner` answers a nil
reader with `return scans, func() {}` — **a no-op stop function**. The entire
mechanism below is unreachable in the sim by construction.

## The call path, and the blocking site

```
walletPolicyFlow                                   (gui/wallet_policy.go)
 └─ bundleGatherFlowResume                         (gui/bundle_flow.go)
     ├─ scans, stopScanner := startScanner(ctx, ctx.Platform.NFCReader())
     │                                              (gui/nfc_scan.go)
     ├─ defer stopScanner()          ← RUNS ON THE FLOW'S RETURN = the BACK edge
     └─ for { … ctx.Frame(…) }       ← the frame loop AND the event pump

stopScanner:                                        (gui/nfc_scan.go)
        close(closer)
        r.Close()                    ← ***BLOCKS HERE***
        <-closed

Poller.Close:                                       (nfc/poller/poller.go:92)
        select {
        case p.reading <- struct{}{}:   // free? take it
        default:                        // a Read is in flight:
                p.d.Interrupt()         //   ask it to stop …
                p.reading <- struct{}{} //   *** and WAIT, with no timeout ***
        }

Device.Interrupt:                       (driver/st25r3916/st25r3916.go:304)
        select {
        case d.cancel <- struct{}{}:    // d.cancel is make(chan struct{}, 1)
        default:                        // *** ALREADY FULL: SIGNAL DROPPED ***
        }
```

`d.cancel` is consumed in exactly one place — `waitInterrupt`'s select
(`st25r3916.go:410`). **Any `Interrupt()` delivered while the reader is not
parked in that select leaves a token in the channel.** The next `Interrupt()`
then hits `default` and is dropped, the in-flight `Read` is never cancelled,
`p.reading <- struct{}{}` blocks **forever**, and `stopScanner` never returns.

The frame loop, the event pump (`pl.AppendEvents`) and the flow all live in the
**same goroutine**, so blocking there means: no frame is drawn (the LCD keeps the
last one — the addresses page, or the Bundle+fp page), no input is polled (every
button dead, including the checkmark), and nothing times out (a bare channel
send). That is every reported symptom, including the two that F-440 cannot
explain.

## Why it needs repeated in-and-out visits

Every entry into the gather calls `ctx.Platform.NFCReader()`, which is
`poller.New(p.nfc)` (`cmd/controller/platform_sh2.go:572`) — **a fresh `Poller`,
with a fresh `reading` semaphore, over the ONE shared `st25r3916.Device`.** The
`d.cancel` channel belongs to the device, so it persists across pollers. A stale
token dropped by visit *n* is what poisons the `Close()` of visit *n+1*. That is
exactly the operator's cycle, and exactly why the first pass is fine.

## Fix proposals (NOT implemented — outside F-440's scope, awaiting go-ahead)

1. **Drain before signalling.** In `Device.Interrupt`, empty `d.cancel` first,
   then send — so the signal is never dropped:
   `select { case <-d.cancel: default: }` before the send. Smallest change that
   removes the wedge.
2. **Bound the wait.** `Poller.Close` must not block unboundedly on a resource a
   dropped signal can strand. Give the second `p.reading <- struct{}{}` a
   timeout and return an error rather than hanging the UI goroutine.
3. **Do not block the frame loop on teardown at all.** `stopScanner`'s
   `<-closed` join runs on the UI goroutine; a scanner that cannot be stopped
   should be abandoned (leak the goroutine, mark the device unusable) rather than
   freeze the device. A frozen panel is strictly worse than a leaked poller.
4. **One reader, not one per screen.** Re-opening a `Poller` per gather entry is
   what makes stale device state reachable at all.

(1)+(2) are the minimal pair: (1) removes the cause, (2) makes the symptom a
recoverable error instead of a brick.

## A bench prediction, if the device is still powered

If the theory holds, the device is parked in `Poller.Close` **after**
`p.d.Interrupt()` and after `close(closer)`. The scanner goroutine is inside
`Read` → `waitInterrupt`, which also selects on `d.interrupts` — so **presenting
any NFC tag or an active reader field should produce an interrupt, release the
read, unblock `Close`, and the screen should immediately jump to the gather.**
A tag waking a "dead" device confirms this diagnosis and refutes every
alternative. (The operator reports no tag is available, so this stays a
prediction.)

## Item 3's original hypothesis, killed with evidence

The backlog model does not survive contact with the input path:

- `AppendEvents` (`platform_sh2.go:376`) is called **inside** `ctx.FrameCallback`
  — i.e. only when the flow draws. Nothing accumulates while a frame is blocked.
- Input is the **touch panel**, and `p.touch.ints` is `make(chan struct{}, 1)`
  with a non-blocking send: interrupts **coalesce to at most one pending**.
- `processTouch` reads the *current* point and suppresses no-change readings, so
  a tap that begins and ends while the loop is blocked is **lost, not queued**.
- `AppendEvents` returns after **one** event, so a frame can never deliver a
  burst.

Fifty rapid presses therefore cannot build a backlog on this hardware. Combined
with the operator's own "the screen updated every time I pressed right arrow",
the recompute-storm theory is dead twice over — and the consent screen, which is
what actually pages here, re-derives **nothing**: its lines are computed once
(8.9 ms on x86 for the whole 11-key miniscript consent surface, 1.2 ms per page
press thereafter).

For completeness, the derivation costs that *would* have mattered: the consent
build performs **1 probe + 4 addresses = 5 derivations × 11 keys = 55 BIP-32
child derivations**, 8.9 ms on x86. Note that x86 uses secp256k1's precomputed
base-point table while the device is built with `//go:build tinygo` →
`scalarBaseMultNonConstSlow`, so any future MCU scaling of a derivation number
must include that penalty on top of the CPU factor. It is not needed here: the
lock is not arithmetic.

---

# APPENDIX C — F-441 implemented, and a CORRECTION to Appendix B

**Commit** `4698223` — `nfc: F-441 -- a reader that will not stop must not take
the device with it`, on `f440/modal-back` above `9762542`. Not pushed.

## CORRECTION: the dropped signal was not the cause

**Appendix B named `Device.Interrupt`'s dropped `d.cancel` send as the
mechanism. That is wrong, and the claim did not survive being executed.**

`d.cancel` has capacity 1, and a pending token is **indistinguishable from a
fresh one to the next waiter** — both make `waitForInterrupt` return `io.EOF`.
So a dropped send delivers the same cancellation the send would have. Exhausted
rather than argued:

```
stale=false  bare delivers=true   drain-then-send delivers=true   same=true
stale=true   bare delivers=true   drain-then-send delivers=true   same=true
```

The freeze therefore required an in-flight read that **never reaches
`waitForInterrupt` again** — a stalled bus transaction, or a path with no
cancel-covered wait — which no signal of either kind can rescue. Appendix B's
call-graph, its edge-triggering, its "hardware-only because the sim stubs the
reader", and its symptom mapping all stand. Only the named cause was wrong, and
it was the one link I reasoned about instead of running.

**What that changes about the fix:** the bound is not a safety net behind the
real fix — it *is* the fix, because it holds whatever stalls the read. The drain
is hygiene.

## What landed

1. **`Poller.Close` is bounded** (`nfc/poller/poller.go`). Waits at most
   `CloseTimeout` (2 s), then returns `ErrCloseTimeout`. On that path it does
   **not** touch the device: the read still owns the bus and `d.Close` writes a
   register, so leaving the chip alone is the safe half of giving up.
2. **`stopScanner` abandons** (`gui/nfc_scan.go`). On a Close error it logs,
   leaves the goroutine and returns; its own join is bounded by
   `scannerJoinTimeout` (3 s, above the scanner's 1 s worst-case sleep). A leaked
   goroutine and a degraded reader beat a machine that needs unplugging.
3. **`Device.Interrupt` drains before signalling** (`driver/st25r3916`). Kept as
   hygiene — a no-op with no error and no way for the caller to know is a shape
   defects hide in — with its comment now saying plainly that it is not the
   cause.

## The `d.cancel` sweep

`d.cancel` was the **only cancellation** carried on a lossy channel. The other
five capacity-1 channels are **doorbells** whose receivers re-read authoritative
state, where dropping a duplicate is correct:

| channel | receiver re-reads | verdict |
| --- | --- | --- |
| `d.interrupts` (pin ISR) | `interruptStatus()` after every wake; `resetInterruptMask` already drains one first — the idiom `d.cancel` lacked | safe |
| `USBPD_INT` (`platform_sh2.go:486`) | `d.ReadStatus()`, `continue` if unchanged | safe |
| `Platform.wakeups` | the frame loop re-evaluates on wake | safe |
| `touch.ints` | `processTouch` reads the live panel | safe (its transient-tap loss is the separate finding in Appendix B) |
| `engraver.busy` | blocking send used as a mutex; drops nothing | safe |

Nothing else to fix.

## Test evidence, red → green

The stub whose absence made this hardware-only is now in the tree:
`testPlatform.NFCReader()` returns nil and `startScanner` answers nil with a
no-op stop, so the mechanism was unreachable from Go tests **by construction**.

- `nfc/poller/poller_close_test.go` — a device that **cannot** be stopped (a
  model of the only thing `Close` may assume, not of the ST chip).
- `gui/nfc_scan_abandon_test.go` — the reader `stopScanner` must abandon. It
  **arms** the condition rather than assuming it: the scanner checks its closer
  at the top of each iteration, so a stop issued before the first `Read` joins
  instantly and would pass without exercising a stall. Caught in review of my own
  first draft, which did exactly that and stayed green under mutation.

```
bound removed      panic: test timed out after 25s
                     running tests:
                       TestCloseReturnsAnErrorWhenTheReadCannotBeStopped (25s)
abandon removed    --- FAIL: TestStopScannerAbandonsAReaderThatWillNotStop (10.01s)
both restored      ok seedhammer.com/nfc/poller 2.004s / ok seedhammer.com/gui
```

Healthy paths are pinned too: `Close` on a stoppable read returns nil well inside
the timeout, `Close` on an idle poller never interrupts, and a well-behaved
reader is joined without waiting out the bound.

**`Interrupt` carries no Go test, by constraint not omission:**
`driver/st25r3916` is `//go:build tinygo`, so nothing but the device build ever
type-checks it. That is the *second* reason the suite could not see this defect,
and it is now recorded beside the function.

## Gate tails (`4698223`)

```
gui shard    1034 top-level tests, partition verified exhaustive
             === wall: 22s ===  RESULT: ok -- all 1034 tests ran across 24 shards
non-gui      exit 0, 53 ok  (was 52; nfc/poller gained tests)
go vet ./... no new findings vs an a0c1615 baseline taken the same way
gofmt -l     clean on all five touched files
tinygo       pico-plus2, exit 0
             1199608  269360  31628  30956 | 1500596  62584 | total
```

## Residue for F-442

The "reader marked failed" half is **not** implemented: with a `Poller` built per
screen entry there is nowhere durable to record it, and the next entry builds a
fresh poller over the same device — so an abandoned read still owns the bus while
a new one starts. That is strictly better than freezing (the UI stays alive) but
it is a real degradation, and it is F-442's "one reader, not one per screen"
change that gives the flag somewhere to live.
