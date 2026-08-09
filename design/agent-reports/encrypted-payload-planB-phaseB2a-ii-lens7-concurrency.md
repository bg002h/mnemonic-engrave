# B2a-ii whole-diff review — LENS 7: concurrency, lifetime and the frame loop

Reviewer: independent adversarial pass (opus), 2026-08-08.
Scope: `421dca8..HEAD` on `feat/encrypted-payload-b2a-ii`
(`/scratch/code/shibboleth/seedhammer-wt-b2aii`), read against
`design/SPEC_encrypted_payload_delivery.md` (normative) and
`design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a_ii.md`.

Lens brief: the interactions nobody traced — the chunked KDF's frame pumping,
the engraver goroutine's lifetime, `ctx.Done` across the session, the
screensaver, and any new goroutine/channel/timer/defer.

Out of scope by instruction and NOT re-reported: lens 1's C1/I1/M1/D1/D2 and its
pass-3 items; F-83, F-86, F-87, F-88, F-89; "there is no idle timer" (B2b); the
sanctioned `[setup failed]` pair; the surviving `clear(blob)` mutant.

**Verdict: 1 Critical, 0 Important, 2 Minor, 1 Nit.**

---

## C1 (Critical) — the chunked KDF asks for its next frame one frame too late, so the derivation parks for the full 3-minute `idleTimeout` after its FIRST slice and the screensaver then takes the screen

**Where:** `gui/unlock_kdf.go:191-200` (`unlockDerive`), against `gui/gui.go:2968-2998`
(`Run`'s frame loop) and `gui/gui.go:2915` (`idleTimeout = 3 * time.Minute`).

**Defect.** `unlockDerive` submits the frame and *then* asks for the next one:

```go
        ctx.Frame(op.Layer(
                nav, titleOp, pctOp.Offset(...), leadOp.Offset(...),
                op.Color(&ctx.B, th.Background),
        ))
        // Ask for the next frame immediately: this loop IS the work, and a
        // deadline in the future would idle the KDF instead of running it.
        ctx.WakeupAt(time.Now())
```

`ctx.Frame` *is* the yield. `Run` reads the deadline for the frame it has just
been handed, at `gui/gui.go:2972`, **before** its own `ctx.Reset()`:

```go
wakeup := ctx.Wakeup                      // read HERE
evts = pl.AppendEvents(wakeup, evts[:0])  // parks until then
...
ctx.Reset()                               // zeroes ctx.Wakeup
...
ctx.WakeupAt(idleWakeup)                  // = last-touch + 3 min
break                                     // ctx.Frame returns to the flow
```

So `ctx.WakeupAt(time.Now())` on line 200 governs frame *N+1*, never frame *N*.
Frames 2..199 are covered by the previous iteration's call and are fine; **frame
1 is submitted with whatever the preceding screen's cycle left behind, which is
`a.idle.start + idleTimeout`** — the last touch (the twelfth word's OK release)
plus three minutes.

The house idiom is the opposite order and the fork already uses it:
`EngraveScreen.Engrave` calls `ctx.WakeupAt(time.Now().Add(time.Second/2))` at
`gui/gui.go:2734` **before** `ctx.Frame` at `:2741`, and `Run`'s own saver branch
calls `ctx.WakeupAt(now.Add(minFrameTime))` before looping back to `yield()`.

**Evidence — measured end to end, not argued.** In a copy of the worktree I drove
the *real* `unlockPayloadFlow` over vector D through the *real* word keyboard at
`sh2DisplaySize`, with a `FrameCallback` that reproduces `Run`'s ordering exactly
(read `ctx.Wakeup`, then `Reset`, then `WakeupAt(idleWakeup)`), and recorded the
deadline `AppendEvents` would be given for every frame:

```
progress frame #1 (log index 63): AppendEvents deadline 2m59.962864152s
progress frame #2 (log index 64): AppendEvents deadline -62.709µs
progress frame #3 (log index 65): AppendEvents deadline -62.438µs
progress frame #4 (log index 66): AppendEvents deadline -66.174µs
progress frame #50  ...            AppendEvents deadline -60.057µs
progress frame #100 ...            AppendEvents deadline -60.636µs
progress frame #150 ...            AppendEvents deadline -60.665µs
total progress frames: 199
1 progress frames were submitted with a FUTURE AppendEvents deadline
```

Nothing ends that park early. On the SH2, `Platform.AppendEvents`
(`cmd/controller/platform_sh2.go:369-396`) parks on
`select { timer | wakeups | stdin | touch.ints }`; during a derivation nothing
calls `Platform.Wakeup()` (its only production callers are the engraver
goroutine's `defer e.pl.Wakeup()` at `gui/engraver.go:110` and
`ConfirmDelay.Progress` at `gui/gui.go:303`, neither of which is live here), and
a *repeat* touch interrupt with no state change takes the `if !ok { break }`
branch at `:390-392`, which breaks the **select** and keeps waiting. `cmd/emu`
behaves identically (`cmd/emu/platform.go:157-168`), so this is not
hardware-only.

**What the operator sees.** Twelve words → the progress screen appears at ~0%
("Unlocking. About 31 seconds left.") → **it is frozen for three minutes** → at
`a.idle.start + 3 min` the saver activates, and because `Run` `continue`s at
`:2995` without breaking, `ctx.Frame` never returns and the derivation is stopped
dead at 500/300,000 iterations → the operator touches the screen → the saver
clears, `ctx.Frame` returns, and the remaining 299,500 iterations run in ~31 s.

**Consequences.**

1. **The §10.2 step 7 guarantee is unmet, comprehensively.** Step 7 exists so the
   operator does not "think the machine has hung"; the delivered behaviour is a
   screen frozen at 0% for three minutes followed by a screensaver on a machine
   holding their seed. Operator decision #1 of the plan ("the KDF is chunked,
   with a real progress bar … the frame loop stops for ~31 s and the screen
   freezes, which is exactly the 'machine has hung' reading step 7 exists to
   prevent") is defeated by the very code written to satisfy it — and the result
   is *worse* than the blocking `pbkdf2.Key` it replaced, which at least finished
   in 31 s.
2. **It corrupts the measurement that closes §7.1.** `start := time.Now()` is at
   `gui/unlock_kdf.go:168` and `log.Printf("seal: kdf %d iterations in %s",
   d.Total(), time.Since(start))` at `:179`, so the logged elapsed time
   **includes the park**. Task 9.3 is "record, from the screen and from the log:
   the elapsed derivation time, the iteration count, and the computed rate. **This
   closes §7.1**", and Task 9 is the only thing that ever closes §7.1's owed
   RP2350A-vs-B residual. At the 300,000 default that log line will read ~3 min 35 s
   → ~1,400 it/s instead of ~9,715 — a **6.8× understatement**, recorded verbatim
   into `design/HARDWARE_RESULT_<date>_phaseB2a.md` and used to re-derive the
   iteration count. §7.1's own history is a rate estimate wrong by 1.54× that set
   the default to 450,000; this would be the same error, larger, and this time
   with the number "measured on the real part".
3. It also inflates the on-screen "About N seconds left", since
   `unlockKDFLead(d.Done(), d.Total(), time.Since(start))` divides parked wall
   time by completed iterations.
4. It extends the window in which the plaintext passphrase is resident in SRAM
   (`pass` in `unlockAttemptOnce`, `m` in `unlockSealedFlow`) from ~31 s to
   unbounded — the operator who walks away from a screen that looks hung leaves
   the passphrase live behind a screensaver, and §2.2 item 2 treats the
   ciphertext as published.

**Why nothing caught it.** `TestUnlockDerivesWithARealProgressScreen` asserts
199 frames, monotonic advance and a 0→99 % span, and passes identically before
and after the fix — `runUITouch`'s `FrameCallback`
(`gui/start_screen_touch_test.go:32-39`) calls `ctx.Reset()` *before* yielding,
which destroys `ctx.Wakeup`, and no gui harness models `AppendEvents`' deadline
at all. The frame *count* is not the property that matters; the frame *deadline*
is, and it is invisible to every existing test.

**Fix (one line, machine-verified).** Move the call to the other side of
`ctx.Frame`, matching `EngraveScreen.Engrave`:

```go
        // BEFORE ctx.Frame: Run reads ctx.Wakeup for the frame it is handed.
        ctx.WakeupAt(time.Now())
        ctx.Frame(op.Layer(
                nav, titleOp, ..., op.Color(&ctx.B, th.Background),
        ))
```

Verified in a scratch copy: the probe above then reports every progress frame at
a negative (already-expired) deadline, and
`CGO_ENABLED=0 go test ./gui/ ./seal/` is green —
`ok seedhammer.com/gui 17.474s`, `ok seedhammer.com/seal 13.265s`.

**And add the assertion, because the suite cannot currently see this.** The
regression test is the probe: instrument `ctx.Wakeup` at frame-submission time
and assert every progress frame's deadline is already expired. A frame-count
assertion is a guaranteed false PASS here, in exactly the way §11.2's
"instrument it, don't trust the return value" rule anticipates.

---

## M1 (Minor) — the screensaver still parks the derivation at spec-legal iteration counts, and parked time is counted as derivation time

**Where:** `gui/unlock_kdf.go:162-203` against `gui/gui.go:2975-2996`.

**Defect.** `a.idle.start` is refreshed only by `len(evts) > 0`. `unlockDerive`
produces no events, so a derivation longer than `idleTimeout` will always trip
the saver, and the saver branch `continue`s without breaking — `ctx.Frame` does
not return and the KDF stops until a touch. §6.2 admits `iterations` up to
2,000,000, which at §7.1's measured 9,715 it/s is 205.9 s > 180 s, so this is
reachable with a fully spec-legal blob **even after C1 is fixed**.

**Consequence.** With such a blob the unlock stalls near the end, the "About N
seconds left" reading inflates by the parked time, and the §7.1 log line
overstates the elapsed time by however long the operator left it. Bounded and
self-healing on touch; no seed exposure beyond the residency §2.2 item 9 already
concedes.

**Fix.** Either bound the accepted iteration count below what `idleTimeout`
allows, or treat an in-progress derivation as activity (this needs a `Run`-side
change and therefore belongs with B2b's timer work — §10.2.4 already has to
reconcile a residency timer with a saver that does not unwind, which is F-89).
At minimum, make the §7.1 log line report *derivation* time rather than wall
time, so Task 9.3's number survives any park: accumulate `time.Since` around the
`d.Step` calls only. Recommend filing with **owning phase: B2b**.

---

## M2 (Minor) — `bip39.Parse`'s three error returns still orphan an unwiped `Mnemonic`

**Where:** `bip39/bip39.go:257-280`.

**This is NOT the pass-3 item.** That one was the `append` *reallocation* chain,
and `m := make(Mnemonic, 0, 24)` fixes it. This is the *error return*: on
`"mnemonic too long"` (`:266`), `"unknown word"` (`:272`) and
`ErrInvalidChecksum` (`:277`) the function returns `nil` while `m` still holds
every word accumulated so far, and the caller receives nothing it can `clear`.

**Consequence.** The materially interesting case is `ErrInvalidChecksum`, where
`m` holds the **complete** word list. `seal.Classify` (`seal/record.go:143-148`)
calls `Parse` on every record of both sections and now clears `m` on success — but
a mnemonic-shaped record with a bad checksum falls through the `err != nil` path,
leaving a full 12/24-word near-seed in the heap where no `Payload.Wipe` and no
`SecretsResident()` can reach it. Reachable only from a non-conforming or
attacker-supplied blob (which §10.2.1 explicitly refuses to assume away), and the
payload is then rejected — so the exposure is a rejected record, not the
operator's own seed. Minor, not gating.

**Fix.** `clear(m)` before each of the three error returns. One line each,
idempotent, no behaviour change.

---

## N1 (Nit) — `unlockPlateListFlow`'s comment claims labels are rebuilt every frame; they are not

**Where:** `gui/unlock_platelist.go:71-79`, `:103-108`.

The comment reads "Labels are rebuilt EACH FRAME rather than once up front, so
the '(cut)' mark appears the moment a plate completes." `relabel()` is in fact
called exactly twice: once before the loop (`:79`) and once after each
`unlockEngraveFlow` returns (`:108`). The *effect* the comment claims is correct
— nothing else can change `cut` — but the mechanism it states is wrong, and this
repo's own standard is that a record defect is a defect. Reword to "rebuilt on
entry and after every engrave, which is the only thing that can change a label".

---

## Checked and found sound (do not re-derive)

Recorded so the next reviewer spends budget elsewhere. Each was traced into the
code, not read off a comment.

**The plate outlives the record, and that is safe.** `bspline.Curve` is
`iter.Seq[Knot]` (`bspline/bspline.go:22`) — a *lazy* generator that the engraver
goroutine iterates at `gui/engraver.go:170`, long after `clear(rec)`/`clear(m)`.
I traced what the closure captures: `engraveSeed` (`gui/gui.go:521-543`) copies
the words out eagerly into `words := make([]string, len(m))` and converts
`seedqr.QR(m)` to a string before `qr.Encode`, so `backup.Seed` holds no
reference to `m`; `engraveSeedString` (`backup/backup.go:148-151`) captures a
`SeedString` whose `Seed` is an immutable Go string built from
`codex32.New(string(rec))`. **Neither `rec` nor `m` is reachable from
`plate.Spline`**, so the early wipe cannot alter what is cut into steel. This was
the one plausible funds-loss path in this lens and it is closed.

**`ctx.WakeupAt` ordering elsewhere in the diff.** `git diff 421dca8..HEAD |
grep WakeupAt` returns exactly one hit — the C1 line. No other new screen
schedules a frame.

**`ctx.Done` terminates every new loop.** `ChoiceScreen.Choose`
(`gui/gui.go:1430-1480`) and `showModal` (`gui/slip39_polish.go:23-33`) both
return on `ctx.Done`, so `unlockPassphraseFlow`, `unlockSealedFlow`,
`unlockSecretSession`/`unlockSecretPlate`, `unlockEngraveFlow`'s unguarded
`for { }` and `unlockPlateListFlow` all unwind. On the `ctx.Done` path every
secret still gets its `unlockSecretPlate` call, so every `defer p.WipeSecretAt(i)`
runs, and `unlockPayloadFlow`'s `defer p.Wipe()` closes the rest. No path blocks
forever and no screen leaks.

**`Deriver` cannot divide by zero or loop forever.** `NewDeriver` clamps
`iterations >= 1` (`seal/pbkdf2.go:54-59`) so `d.Done()*100/d.Total()` is safe;
`Step(500)` always advances `done` until `done >= total`, so the loop terminates.
`Done()*100` peaks at 2×10^8 against §6.2's 2,000,000 ceiling, inside a 32-bit
`int`. `unlockKDFLead`'s `int64(elapsed)*int64(total-done)/int64(done)` cannot
overflow inside the reachable range (worst case ≈4×10^17 against 9.2×10^18).
`d.Key()` is evaluated before the deferred `d.Wipe()`.

**Back during the derivation works.** `AppendEvents`' top-of-function drain
(`cmd/controller/platform_sh2.go:371-378`, "Don't starve touch input") processes
a pending touch interrupt even with an expired deadline, so `backBtn.Clicked` is
reachable at 19 fps. Cancelling exits the whole flow, which is `Back IS Lock`
(§10.3) and consistent.

**`clear(blob)` is not a write to XIP.** `XIPReader.Read`
(`seal/read_tinygo.go:48-58`) copies out of the mapping before returning, so the
`clear(blob)` added at `gui/unlock_flow.go:110` writes to heap, not to flash.

**The two new `defer`s in `gui/gui.go` are correctly ordered.**
`masterFingerprintFor`'s `defer mk.Zero()` (`:556`) cannot race the return value:
`return bip32.Fingerprint(pkey), nil` is evaluated before the defer runs.
`deriveMasterKey`'s `defer wipeBytes(seed)` (`:233`) is safe because
`hdkeychain.NewMaster` retains nothing from `seed` — the identical pattern is
already shipped and exercised at `gui/derive.go:20-22` (`deriveAccountXpub`).

**No new goroutine, channel or timer.** This phase adds none. The only goroutine
in the neighbourhood is the pre-existing engraver job (`gui/engraver.go:109-113`),
and this phase's code makes no assumption about its lifetime: `Engrave`'s
`defer s.job.Stop()` closes `quit`, `errs` is buffered so the goroutine cannot
leak, and the record it might race for is already zeroed before the plate is
built. `Engrave` can return in `engraveStopping` with the goroutine still
draining, so the next plate's `Start()` can in principle meet an
not-yet-`Close()`d driver — but that shape is pre-existing (`bundleEngrave`,
`unlockPlateListFlow`) and is separated by human-scale interaction, so it is not
this phase's defect.

**Test-hook lifetime is clean.** `newDeriver`, `unlockPassphraseHook`,
`unlockSecretHook`, `unlockMnemonicHook` and `unlockEngraveHook` are
package-level vars, but no `gui` or `seal` test calls `t.Parallel()` and no test
spawns a goroutine (`grep -rn "t.Parallel()" gui/ seal/` and
`grep -rn "go func" gui/*_test.go` are both empty), so there is no race and no
cross-test bleed. `iter.Pull`'s `stop` resumes the coroutine so the flow unwinds
inside `t.Cleanup`, which fires `unlockSecretHook("wiped", …)` after the
assertions — harmless, and the hook's index is bounds-guarded at
`gui/unlock_session.go:110`. No gui test ever starts the engraver job (they stop
at `engraveIdle`'s "Insert a blank plate"), so nothing outlives a test.

**`log.Printf` on the device is established practice.** `gui/scan.go:47`,
`gui/gui.go:1694` and four others already log from production flows, so
`unlock_kdf.go:179` introduces no new blocking risk.

---

## Machine-checked claims in this report

| claim | how |
| --- | --- |
| the first progress frame carries a 3-minute deadline | end-to-end probe over vector D through the real keyboard; `2m59.962864152s` |
| frames 2..199 carry expired deadlines | same probe; `-62µs` typical, 199 progress frames total |
| the existing suite cannot see it | `TestUnlockDerivesWithARealProgressScreen` passes identically before and after the fix |
| the one-line move fixes it | probe re-run: every progress frame negative |
| the fix breaks nothing | `CGO_ENABLED=0 go test ./gui/ ./seal/` → `ok gui 17.474s`, `ok seal 13.265s` |
| only one `WakeupAt` in the diff | `git diff 421dca8..HEAD \| grep -n WakeupAt` → one hit |
| no parallel tests, no test goroutines | `grep -rn "t.Parallel()" gui/ seal/`; `grep -rn "go func" gui/*_test.go` → both empty |

All work was done in a copy at `/tmp/lens7-conc-<pid>`, since deleted. Nothing
under `/scratch/code/shibboleth/seedhammer-wt-b2aii` was modified.
