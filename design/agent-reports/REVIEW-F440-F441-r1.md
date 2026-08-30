# REVIEW — F-440 + F-441 pre-merge gate, round 1

**Target:** `git diff a0c1615..4698223` on `f440/modal-back` in
`/scratch/code/shibboleth/sh-worktrees/f440-modal-back` — `9762542` (modals
answer BACK) and `4698223` (the `Poller.Close` bound). 7 files, +578/-5.

**THE ONE QUESTION: is this diff safe to boot?**

## VERDICT — GREEN. 0 Critical, 0 Important. 4 Minor, 3 Nit.

Nothing in this diff can make a modal undismissable, alter a caller's decision,
wedge the UI on a Close path, or crash. Both new waits are bounded, and the
worst-case UI stall on a back edge is ~5 s against an unstoppable reader versus
the measured 50 ms on a healthy one. **The merge should proceed.**

The worktree was not modified. All probes ran through `go test -overlay` from
the scratchpad; the tree is byte-identical to `4698223` (verified at the end).

---

## 1. THE FALSE-ABANDON ANSWER (asked directly in the brief)

**No. A healthy reader cannot be falsely abandoned, with roughly 20x of margin.**

### The bound bites exactly where it says it does — measured

Probe `TestProbeHowSlowAHealthyStopMayBe` sweeps the delay between `Interrupt()`
landing and the read reaching a cancel-covered select:

```
read reaches its cancel select   10ms after Interrupt -> Close=<nil>            in 10ms    (device closed=1)
read reaches its cancel select  500ms after Interrupt -> Close=<nil>            in 500ms   (device closed=1)
read reaches its cancel select  1.9s  after Interrupt -> Close=<nil>            in 1.901s  (device closed=1)
read reaches its cancel select  2.1s  after Interrupt -> Close=ErrCloseTimeout  in 2.001s  (device closed=0)
```

So the question reduces to: **can a real read take more than 2 s to reach
`waitForInterrupt`?**

### Bounding the real driver — it cannot

`d.cancel` is consumed in exactly one place, `waitForInterrupt`'s select
(`driver/st25r3916/st25r3916.go:455`). Everything else a read does between two
of those selects is I2C:

- The longest register run in the path is `Detect()` -> `reset()`'s 14-pair
  `writeRegs`, and `writeRegs` issues **one `Tx` per pair** (st25r3916.go:731),
  plus ~6 more before `enable()`'s first `waitForInterrupt`. `configureProtocol`
  is a 15-pair block plus `resetInterruptMask`'s 3. Call it ~21 transactions as
  the worst stretch.
- Every one goes through `multiplexI2C.Tx` (`cmd/controller/platform_sh2.go:715`)
  into TinyGo's `machine.I2C.tx`, which carries a **hard deadline**
  (`machine_rp2_i2c.go:295-297`, tinygo 0.41.1):

  ```go
  timeout_us := uint64(4_000) + uint64(txlen+rxlen)*100
  timeout_us = min(timeout_us, 500_000)
  ```

  ~4.2 ms for a 2-3 byte register write, ~55 ms for a 512-byte FIFO read.

**21 x 4.2 ms ~= 88 ms worst case, and that is against a chip that has stopped
ACKing entirely** — plus at most one queued transaction on the shared bus. Two
orders of magnitude inside `CloseTimeout`. Nothing legitimate approaches it, and
a device that does exceed it is by construction outside the driver's own timeout
discipline, which is precisely the case the bound exists for.

The corollary is worth stating because it cuts against the implementer's own
narrative: **I could not construct a >2 s non-cancellable path in the current
driver either.** Appendix C's corrected cause ("an in-flight read that never
reaches `waitForInterrupt`") names the right *shape*, but every non-select wait
in this driver is an I2C transaction bounded at <=500 ms, so no concrete instance
of that shape exists today. That does not weaken the fix — a bound that holds
whatever stalls the read is the right construction regardless of which stall it
was — but the field cause should still be considered **unidentified**, not
closed. See M-1 for the arm that plausibly *is* the one that fixed it.

## 2. THE LEAKED-READ HAZARD — constructed, not reasoned about

`NFCReader()` is `poller.New(p.nfc)` (`platform_sh2.go:572-573`): **a fresh
`Poller` on every gather entry, over the ONE shared `*st25r3916.Device`.** So
after an abandon the next visit really does build a new reader over a device
whose old read still owns it. Constructed with a fake device that shares
`cancel` exactly as the driver does (`TestProbeNextVisitAfterATransientAbandon`):

```
visit 1: abandoned after 2.001s; device Close calls=0 (must be 0); cancel token pending=1
visit 2: abandoned read A returned device: timeout; the NEW read B returned EOF
      => the new visit's FIRST read was cut short by the abandoned visit's cancel token
         (device-scoped, cap 1). Cost: one wasted poll.
```

**The second visit WORKS.** The degradation is exactly two things, both measured:

1. **One stolen cancellation.** The abandoned visit leaves a token in
   `d.cancel`, and the next visit's first `waitForInterrupt` takes it and returns
   `io.EOF`. `scanner.Scan` reports that as no-object/no-status, the scanner
   classifies it `idle`, sleeps `nfcIdlePoll` (50 ms) and the next poll succeeds.
   **Cost: ~50 ms, once.** (This is genuinely new: before the fix `Close` blocked
   until the intended reader consumed the token, so it could never cross into
   another poller. Filed M-3.)
2. **~8 KB held, transiently.** The abandoned goroutine retains its scan buffer
   (`gui/scan.go:31`, `make([]byte, 8*1024)`), the `Poller` (a 256-byte
   `bufio.Reader` and a `type4.Tag`) and its own stack. TinyGo cannot bound that
   goroutine's stack statically — it prints
   `seedhammer.com/gui.startScanner$2  recursive` — but that is the same bucket
   every goroutine entry point in this firmware already sits in, not a regression.

**It is not permanent, and that is the important part.** `closer` is already
closed before `r.Close()` runs, so the abandoned goroutine exits one iteration
after its read returns — and every driver wait is bounded (`defTimeout` 1 s,
`fieldOnTimeout` 10 s, `fieldDetectionTimeout` 700 ms) or is an I2C transaction
bounded at <=500 ms. So this is **not** "permanently dead NFC until reboot", and
it is certainly not a crash or a wedge. It is dead for as long as the underlying
stall lasts, and no longer.

`TestProbeRepeatedAbandonsWithAPermanentStall` runs three consecutive
abandon-and-re-enter cycles against a stall that never clears:

```
visit 1: back edge cost 2s, 1 leaked reads, device Close calls=0, interrupts=1
visit 2: back edge cost 2s, 2 leaked reads, device Close calls=0, interrupts=2
visit 3: back edge cost 2s, 3 leaked reads, device Close calls=0, interrupts=3
```

Every back edge returns. The UI is never staked on it. That is the whole point of
the change and it holds under repetition.

### 2b. Does anything else touch the device on the timeout path?

**Nothing in the timing-out poller.** `nfc/poller/poller.go:137` is the only
`p.d.Close()` in the package; the timeout arm returns before it, and its only
`defer` is `t.Stop()` on a `time.Timer`. There are no finalizers anywhere in the
tree (`SetFinalizer`/`AddCleanup` appear only in `gui/op/release_test.go`). The
other `.Close()` hits in `cmd/controller` are the engraver, not the NFC device.

**But the rule is per-poller, and the NEXT poller does not honour it**
(`TestProbeNextPollerClosesTheDeviceUnderTheAbandonedRead`):

```
after visit 2's idle Close, d.Close calls = 1 while visit 1's read is STILL in flight (1)
```

Visit 2's `Close` takes the free-semaphore arm and calls `d.Close()` — a
`writeReg(regOpCtrl, 0)` — while the abandoned read still owns the chip. This
**cannot corrupt the bus**: `multiplexI2C.Tx` serialises every transaction
through a capacity-1 channel, and TinyGo's rp2 `tx` contains no yield point
(checked: no `Gosched`, no `time.Sleep`, no runtime call in
`machine_rp2_i2c.go:282-470`), so under `-scheduler tasks` two transactions
cannot interleave and `d.scratch` cannot be torn. The effect is that the RF field
is turned off under the abandoned read, which makes it error out sooner. Benign,
arguably helpful. Filed M-4 for F-442's ledger.

## 3. THE MODAL CHANGE — no siblings found

Seven adversarial probes constructed against the real `ErrorScreen.Layout`, all
green:

| probe | property | result |
| --- | --- | --- |
| P1 | one BACK dismisses **once** and does not also click the screen underneath | pass |
| P2 | rapid double-press: first dismisses, second reaches the screen underneath | pass (see below) |
| P3 | OK+BACK released in the same frame, **both orders**, both armed | pass, exactly one dismissal, survivor clicks nothing |
| P4 | a press arriving in the dismissal frame is **not** swallowed | pass |
| P5 | a non-dismissing frame keeps Button1 (the fix) and still discards Button2 | pass |
| P6 | `ConfirmWarningScreen` unchanged: Button1 -> `ConfirmNo`, a bare Button3 tap does **not** confirm | pass |
| P7 | a stale unmatched head event cannot wedge BACK across frames | pass |

P2's outcome — a double tap walks back two screens — is the operator's own two
presses reaching two screens, and is identical to what Button3 already did. Not a
divergence.

**The dangerous direction is covered.** The implementer's fix was for a Button1
queued behind a dismissing Button3; the *reverse* (a single BACK press
double-consumed into the screen below) was untested, and P1 now confirms it does
not happen. Mechanism: `Clickable.Clicked` returns on the **first** clicked
event, and `EventRouter.Next` only ever inspects the head, so a dismissal takes
exactly one event.

**The short-circuit is load-bearing and pinned.** Mutating
`s.ok.Clicked(ctx) || s.back.Clicked(ctx)` into two unconditional calls:

```
M4 (drain both):  --- FAIL: TestErrorScreenDismissalLeavesTheNextClickAlone
                      "the modal swallowed the Back queued behind the dismissal"
                  and, run wider, panic: test timed out after 1m30s
                      running tests: TestRecoverRejectsNonCodex32 (1m30s)
```

Exactly the two failures the implementer reported. Removing the binding entirely
(`M5`) fails `TestErrorScreenDismissesOnBackAndOnOK/Button1` and
`TestF440BundleIncompleteModalDismissesOnBack`. Both directions are pinned.

**The return-before-`ctx.Frame` claim is true.** `Router.Reset()` is called from
`Context.Reset()` (`gui/gui.go:127`), which the run loop calls *inside* the frame
callback (`gui/run_flow.go:353`, after `flow` has parked in `ctx.Frame`). A
`Layout` that returns `true` never reaches `ctx.Frame`, so nothing is discarded
on the way out. Filters left registered by the dismissed modal survive into the
next frame's `Reset`, which is strictly *more* permissive — it can only preserve
events, never drop them.

### The force-ack audit, verified independently

- **143** `showError(`/`showNotice(` call sites outside their own definitions,
  non-test — the implementer's number, confirmed by count, not by reading.
  (Plus 6 direct `showModal(`/`showCodex32Error(` sites, all through the same
  `Layout`.)
- **5** caller loops, all read: `showModal` (slip39_polish.go:23),
  `showCodex32Error` (codex32_polish.go:160), `showSeedError` (gui.go:944), and
  the two `showErr` closures (gui.go:2981, gui.go:3168). Every one is
  `if dismissed { return | break }`. There is one exit and it leads to one place.
- **6** `ConfirmWarningScreen{}` construction sites (bip85, derive_xpub,
  multisig_build, slip39_polish, unlock_flow, gui.go:2949). It is a **separate
  type with a separate `Layout`** (gui.go:644) carrying its own
  `cancelBtn`/`confirmBtn` pair and a 1 s hold; it cannot inherit anything from
  `ErrorScreen`. P6 pins both halves at runtime.
- **Nothing else can inherit the binding**: no type embeds `ErrorScreen`, and
  every `Button3` binding in `gui/` (22 sites) already has a `Button1` sibling
  within +/-45 lines — swept mechanically, zero exceptions. So there is no
  second BACK-deaf dismiss-only surface left behind either.
- **The layout does not move.** `layoutNavigation` positions by
  `idx := int(clk.Button - Button1)` into a `[3]int`, so Button1 -> slot 0 and
  Button3 -> slot 2: adding the back button does **not** displace the checkmark.
  `warningBodyClip` already insets the body by the full nav-button column
  (`dims.X - (NavBtnPrimary.Dx() + 4)`), so the new button covers no text. No
  index can go negative here (only Up/Down would, and neither is bound).
- **The underlying screens consume no events.** `SeedScreen.Draw` and
  `DescriptorScreen.Draw` were read end to end: they register `op.Input` regions
  and lay out text, and poll no `Clickable`. So taking Button1 for the modal
  removes nothing from any caller — and it stops a touch in the top-right nav
  slot from falling through to a screen that is not the one on top.

## 4. MUTATION SPOT-CHECKS — both of the implementer's reproduced

```
M1  Poller.Close bound removed     panic: test timed out after 25s
                                     running tests:
                                       TestCloseReturnsAnErrorWhenTheReadCannotBeStopped (25s)
M2  stopScanner abandon removed    --- FAIL: TestStopScannerAbandonsAReaderThatWillNotStop (10.01s)
                                     "stopScanner never returned..."
```

Byte-for-byte the reported result.

**The strengthened arming is load-bearing — verified by removing it.** With the
abandon mutation in place *and* the `<-r.entered` wait deleted from the test:

```
M2b  abandon removed + arming removed   ok  seedhammer.com/gui  (3 runs of 3)
```

A clean FALSE PASS, three times out of three. The implementer's claim that the
first draft "stayed green under mutation" is exactly right, and the arming is
what makes M2 detectable.

## 5. CHANNEL SWEEP — 2 of 5 spot-verified

- **`d.interrupts`** (pin ISR, cap 1, non-blocking send at st25r3916.go:202).
  Receiver `waitForInterrupt` calls `interruptStatus()` after **every** wake,
  which reads the live `regTimerNFCIntr`/`regMaskMainIntr` registers — and those
  registers accumulate until read, so two coalesced interrupts are both seen in
  one status read. `resetInterruptMask` drains one first (st25r3916.go:501-505),
  which is the idiom `d.cancel` was missing. **Claim holds.**
- **`Platform.wakeups`** (cap 1, non-blocking send at platform_sh2.go:431).
  Receiver is `AppendEvents` (platform_sh2.go:391), whose only action is
  `return evts` — the frame loop then re-evaluates `syncArmed(now)`, the idle
  clock and the router queue from live state. A dropped duplicate costs one
  redundant wake, never a state transition. **Claim holds.** Worth adding, since
  the abandoned goroutine keeps calling it: a wakeup carries no `Event`, so
  `effectiveInput` stays false and a leaked scanner **cannot** hold off the
  screensaver or the S10.2.4 wipe clock.

## 6. SUITES — run once each, captured to file

| gate | result |
| --- | --- |
| `gui` shard x24 | 1034 top-level tests, **partition verified exhaustive 1034 == 1034**, all shards ok, wall **21s** |
| non-gui `go test` | exit 0, **53 ok**, 0 FAIL (`nfc/poller` 2.002s) |
| TinyGo `pico-plus2` (CI command, `nix develop -c`) | exit 0 — `1199608 269360 31628 30956 \| 1500596 62584 \| total` |
| `gofmt -l` on all **7** touched files | clean |
| `go vet ./nfc/... ./gui/` | 2 findings, both pre-existing in untouched test files (`testing.ArtifactDir requires go1.26 or later (file is go1.25)`) |

---

## FINDINGS

### Minor

**M-1 — the join-timeout arm is the only new bound with no test at all.**
Mutating `stopScanner`'s bounded join back to a bare `<-closed` (keeping the
Close-error early return) leaves the **entire gui suite green**:

```
M3  join bound removed   1034 top-level tests, partition verified exhaustive
                         RESULT: ok -- all 1034 tests ran across 24 shards   EXIT=0
```

Neither shipped test reaches `case <-t.C`. `TestStopScannerAbandonsAReaderThat
WillNotStop` gives its stub a non-nil `closeErr`, so `stop()` returns at
`if err := r.Close(); err != nil { ...; return }` and never joins;
`TestStopScannerJoinsAReaderThatStops` joins instantly. The arm is reached only
when `Close` returns **nil** and the goroutine is still stuck — which is a real
shape: `Poller.Close`'s free arm parks a token in `p.reading` and never removes
it, so a `Read` started afterwards blocks forever. Confirmed with a healthy
device (`TestProbePollerReadAfterCloseBlocksForever`):

> CONFIRMED: a Read issued after Poller.Close never returns.

I wrote the missing test and it passes (`TestProbeStopScannerJoinTimeoutArm`,
`stop() returned in 3s via the join bound (3s)`, with
`nfc: scanner did not exit within 3s; abandoning it` on the log). Reproduce it by
giving the stub `Close() error { return nil }` and a `Read` that parks.

Kept at **Minor, not Important**, deliberately: on the device the arm is not
reachable today. Getting there needs the scanner descheduled between its
`select { case <-closer: ... default: }` check and `p.reading <- struct{}{}`, and
under `-scheduler tasks` (cooperative, single core, no timer preemption) there is
no yield point in that stretch — the non-blocking select does not yield, and
`-gc precise` collects inline. So the code is correct, I executed the arm, and
what is missing is a regression pin rather than a behaviour. It costs one 25-line
test and should land, but it does not gate the flash.

**M-2 — the composite worst-case back-edge stall is ~5 s, and nothing says so.**
`CloseTimeout` is 2 s and `scannerJoinTimeout` is 3 s, and they are **not**
mutually exclusive: a `Close` whose bounded wait succeeds at 1.99 s returns nil
and then pays up to 3 s of join. Both comments describe their own bound in
isolation. The healthy path is 50 ms, measured end to end on the real types
(`TestProbeRealPollerTeardownAlwaysReturns`: `stop() returned in 50ms`), so this
is a documentation gap about the ceiling, not a defect — but a 5 s dead panel
reads to an operator exactly like the bug that was just fixed.

**M-3 — the timeout path newly lets a cancellation cross poller boundaries.**
Before, `Close` blocked until the in-flight read consumed the token, so `d.cancel`
was always empty by the time `Close` returned. Now the timeout arm leaves it
pending, and the next visit's first `waitForInterrupt` takes it. Measured cost:
one spurious `io.EOF`, classified `idle`, 50 ms. Benign, self-healing, and it
belongs on F-442's ledger next to "one reader, not one per screen" — which
removes it.

**M-4 — "do not touch the device" is a per-poller rule, not a device rule.** The
next entry's `Poller.Close` takes the free arm and issues `d.Close()` under the
abandoned read (measured: 1 call, 1 read in flight). Safe today because
`multiplexI2C.Tx` serialises through a cap-1 channel and TinyGo's rp2 `tx` has no
yield point, so no interleaving and no `d.scratch` tearing is possible. Record it
against F-442 so the invariant is stated where it can be relied on.

### Nit

**N-1 — `stopScanner` skips the join on *any* `Close` error, not just the
abandon.** `if err := r.Close(); err != nil { log; return }` also fires when
`d.Close()`'s `writeReg` returns an I2C error on the perfectly healthy arm.
Harmless — the goroutine exits by itself at the top of its next iteration — but
the code reads as if it were testing for `ErrCloseTimeout`. It presumably is not
`errors.Is(err, poller.ErrCloseTimeout)` because `gui` would then import
`poller`; if so, that trade deserves the one line it currently lacks.

**N-2 — a records slip in the gate tail.** Appendix C reports `gofmt -l` "clean
on all **five** touched files". The diff touches **seven**
(`driver/st25r3916/st25r3916.go`, `gui/gui.go`, `gui/modal_back_test.go`,
`gui/nfc_scan.go`, `gui/nfc_scan_abandon_test.go`, `nfc/poller/poller.go`,
`nfc/poller/poller_close_test.go`). All seven are clean — I re-ran it — so the
claim is true and the count is wrong.

**N-3 — PRE-EXISTING, not introduced here, flagged because it sits on the path
this diff touches.** `scanner.Scan` treats `io.EOF` as "record complete"
(`gui/scan.go:42-53`), and `sysw.DecodeBody` carries **no length field and no
checksum** (`sysw/record.go:67-88`). A `text:`/`pass:` record truncated at an even
hex boundary therefore decodes cleanly into a **shorter** body, and `passScan`
routes to `engravePassphraseFlowFrom` (`gui/gui.go:2559`). Every other scan format
— bip39, codex32, md1/mk1/mt1, descriptors, addresses — is checksum- or
BCH-guarded and rejects a truncation as `errScanUnknownFormat`, so this is the one
gap. It is already reachable without this diff: `d.Read` synthesises `io.EOF` when
a previously active field turns off (st25r3916.go:632-635), i.e. pulling the tag
away mid-record does it. M-3's stolen cancellation adds one more, much rarer,
trigger. Worth a follow-up on its own account; **not** a finding against this
branch.

---

## What was verified mechanically rather than read

- 143 / 6 / 5 / 6 / 22 counts — all `grep | wc`, none hand-counted.
- The TinyGo I2C deadline formula and the absence of yields inside `tx` — read
  out of `machine_rp2_i2c.go` in the pinned tinygo 0.41.1 store path, not
  inferred.
- Every mutation above was compiled and run through `go test -overlay`; no
  mutant was reasoned about.
- Suite counts come from a single capture each, re-read from file.

## Worktree

```
$ git status --porcelain      (empty)
$ git rev-parse HEAD          4698223b3b081f646b773c34f5fd395a628c0de1
$ git diff 4698223 --stat     (empty)
```

**Byte-identical. Nothing modified, nothing staged, nothing pushed.**
