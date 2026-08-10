# Pre-flight review — B2b Task 8, the operator-run hardware procedure

**Reviewer:** independent agent (opus), 2026-08-09.
**Scope, and only this:** are Task 8's steps *runnable* and *observable* by one
operator, at the machine, once, with real seed material? Plan correctness, the
mutation run, the build, and the unwind design are **settled ground per the
brief and were not re-derived**.

**Artifacts read:**
`design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md` (Task 8 at
L1379–1406, the precondition set at L1530–1554), `SPEC_…_delivery.md` §10.2.4
(as amended) and §11.5, `IMPLEMENTATION_PLAN_…_phaseB2a_ii.md` Task 9,
`design/FOLLOWUPS.md` F-99/F-100, `HARDWARE_RESULT_2026-08-07_phaseB1.md`,
`scripts/sh2-flash`, `crates/me-cli/src/main.rs` + `src/seal/passphrase.rs`, and
the b2b tree at `920e1e1` (`gui/run_flow.go`, `gui/wipe_warning.go`,
`gui/wipe_guard.go`, `gui/unlock_kdf.go`, `gui/unlock_session.go`,
`cmd/controller/platform_sh2.go`, `flake.nix`).

**Verdict: the procedure is NOT ready to hand to an operator.** Two Criticals
would either burn the trip outright or produce a recorded result that does not
mean what it says. Task 8 is 5 bullets and 11 lines standing in for a two-hour
session at a machine that engraves live seed material.

Everything below is a defect in the *procedure*, not in the code. I found no
defect in the shipped behaviour; where I went looking for one — both engrave
arms registering `g.job`, `scr.job` being non-nil at construction, spurious
platform events resetting the clock, the dismissing touch leaking through to the
parked screen's widgets — the code was right. Those checks are recorded in
"What I checked and found sound" at the end so nobody re-spends them.

---

## Findings

### C1 — `sh2-flash` with no arguments builds `main @ a01b666`, which is the phase's PARENT. Task 8's only flash instruction is that bare command.

**Location:** plan L1386–1388 (Task 8 preamble).

`scripts/sh2-flash` line 32:

```sh
SH2_REPO="${SH2_REPO:-/scratch/code/shibboleth/seedhammer}"
```

and with no image argument it builds `REF="HEAD"` **in that repo**. Measured:

```
$ git -C /scratch/code/shibboleth/seedhammer rev-parse --abbrev-ref HEAD
main
$ git -C /scratch/code/shibboleth/seedhammer log --oneline -1
a01b666 Merge Plan B Phase B2a-ii -- unlock and the secret session
```

`a01b666` is exactly the baseline this phase is diffed against. The whole of
B2b lives in a *worktree*, `/scratch/code/shibboleth/seedhammer-b2b` at
`920e1e1` on branch `b2b`, and `sh2-flash` has no idea that is where the work is.

The operator follows Task 8 literally, flashes the parent commit, and then
runs 8.1: at 3:00 the ordinary screensaver appears (not the warning), and at
3:30 nothing happens. Every one of 8.1–8.3 fails. The recorded result is a
categorical failure of a feature that is in fact present and correct, on the one
document that will be cited as the hardware evidence for a release tag.

Note also that the signed image's version string is `v0.0.0-g<sha>` from
`flake.nix:103`, rendered on the StartScreen — so the *evidence* to distinguish
the two builds is on the screen the whole time, and nothing tells the operator
to read it.

**Suggested fix.** Replace the preamble's flash sentence with the command and
its own pass condition:

```sh
SH2_REPO=/scratch/code/shibboleth/seedhammer-b2b ~/bin/sh/sh2-flash
```

> The `== Build ==` header prints `git log --oneline -1` of the tree it is about
> to build. **It must read `920e1e1` (or the b2b tip).** If it reads `a01b666`
> you are about to flash the phase's parent and every step below will fail.
> `sh2-flash` also warns `tree is dirty` — the b2b worktree must be clean, or
> the version string carries `-dirty` and the flashed image is not traceable to
> a commit.

Then in 8.5: record the version string from the StartScreen, which is the
after-the-fact proof of which image was under test.

---

### C2 — 8.1 says "walk away" and then asks for three observations that exist only as instants. The post-wipe screen is indistinguishable from a boot.

**Location:** plan L1394–1396 (8.1).

> **8.1** Seal vector F, load, unlock, and **walk away**. Confirm the warning
> at 3:00, the wipe at 3:30, and that the machine **returns to the main menu and
> is still usable** — not a blank screen, not a reboot.

An operator who has walked away can confirm none of the first three:

- **"the warning at 3:00"** is a transition. Traced: `runWithFlow` schedules
  `ctx.WakeupAt(idleWakeup)` and draws the warning on the tick where `idle`
  first goes true, so it appears at `idle.start + idleTimeout`
  (`idleTimeout = 3 * time.Minute`, `gui/gui.go:2949`). Seen only if watched.
- **"the wipe at 3:30"** likewise. `wipeAt := idleWakeup.Add(wipeWarningDelay)`,
  `wipeWarningDelay = 30 * time.Second` (`gui/wipe_warning.go:16`).
- **"not a reboot"** is the dangerous one, because it is *unobservable after the
  fact*. On a wipe the session loop re-enters `flow(ctx, versionText)` with a
  fresh `Context`, and `uiFlow` (`gui/gui.go:1581`) starts at `StartScreen` with
  the same version line. **A reboot lands on the identical screen.** The only
  discriminator is that the unwind is instantaneous and involves no LCD init and
  no PD re-negotiation — i.e. it is *purely a timing observation*, available to a
  present observer and to nobody else. An operator returning at t=10:00 sees a
  StartScreen and writes "returned to main menu, still usable — PASS" for a
  machine that might have rebooted.

This is the trip's primary purpose (fact 2: `ctx.Done` has never been true in
production), and as written the step cannot produce the data it exists to
collect.

**Suggested fix.** "Walk away" means *do not touch*, not *leave the room*.
Rewrite 8.1 as:

> **8.1** … unlock, then **do not touch the screen and stand where you can see
> it**. Start a stopwatch at the release of the last touch (the OK on the
> unlock's twelfth word). Record: (a) the stopwatch reading when the
> `WIPING SECRET DATA` screen appears — expect **3:00**; (b) the first number
> the countdown shows — expect **30**; (c) the stopwatch reading when the screen
> changes — expect **3:30**; (d) that the transition to the start screen is
> instantaneous, with no blank interval and no re-init — a perceptible gap or a
> dark screen means a reboot, not an unwind; (e) the firmware version string on
> the start screen.

**Point a phone at the screen with a running stopwatch in frame** and let it
record 8.1 through 8.3 end to end. It costs nothing, it turns every timing
above into a re-readable measurement, it captures the warning's verbatim text
for 8.5, and it settles the reboot question on playback.

---

### C3 — 8.3 cannot observe either of its two claims with any plate the operator is likely to cut, and one of its readings is a guaranteed false PASS.

**Location:** plan L1399–1400 (8.3).

> **8.3** Start a secret plate and walk away **mid-cut**: confirm **no wipe**
> while the job runs, and that the 3:00 window restarts from the cut's end.

Two independent problems.

**(a) "no wipe while the job runs" needs a job that runs for more than 3:30
untouched.** This project's established engraving-test practice is a
single-character plate — the memory note is explicit: *"engraving tests cut 1
char, top-left, uncentred: ~2s per test vs ~21min per plate"*. A 2-second cut
cannot falsify anything: `armed()` returns false for `engraveRunning` and the
clock would not have expired either way. The step would be performed, and
recorded as a pass, having demonstrated nothing. This is the one SPEC §11.3 row
deferred *to this phase* — "idle timer runs during engraving" (plan L1338–1340)
— so a vacuous run here leaves that row unproven on hardware while looking
proven.

**(b) "the window restarts from the cut's end" is discriminating only if the cut
ends with NO touch.** §10.2.4 as amended says the window re-arms on
"completion, stop, or failure". If the operator **stops** the cut, the stop *is*
a touch, and `if len(evts) > 0 { a.idle.start = now }` resets the clock before
the armed edge is ever considered. Correct code and code with the armed-edge
reset deleted both produce a warning 3:00 later. **A stopped cut is a false PASS
by construction.** Only a cut that runs to *completion* untouched exercises the
`a.idle.start = now // row 2: fresh window at cut end` line.

**The observable sequence, which the step should name, is crisp.** During the
cut `armed()` is false, `keepAwake` is never set (`ctx.KeepAwake()` has exactly
one caller, `gui/unlock_kdf.go:302`), and the engrave screen emits no events —
so at 3:00 into the cut the **screensaver** takes the screen. At the cut's end
the plate-done screen replaces the saver. **That replacement is the operator's
t=0**, it is unmistakable, and it coincides with the machine going quiet.

**Suggested fix.** Rewrite 8.3 as three recorded observations:

> **8.3** Select a secret plate whose cut is **longer than 4 minutes** — a
> one-character test plate is far too short to falsify anything here. Start the
> cut and do not touch the screen again.
> (a) Confirm that ~3:00 into the cut the **screensaver** appears and **no
> warning and no wipe** occurs; record the stopwatch reading at ~4:00 with the
> job still running and the session still open.
> (b) **Let the cut run to completion. Do not press stop** — a stop is a touch,
> and a touch resets the window by a different mechanism, so a stopped cut
> cannot distinguish a working armed-edge reset from a deleted one.
> (c) Restart the stopwatch at the instant the plate-done screen replaces the
> screensaver. Record when the warning appears — expect **3:00** — and when the
> wipe fires — expect **3:30**.

---

### I1 — 8.3 is the one dangerous step and its danger is not named: the engrave screen is ARMED during plate setup.

**Location:** plan L1399 (8.3).

§10.2.4 as amended is explicit that "the hold-to-start and plate-done screens
are **armed**, because they are walk-away states with secrets still held", and
`wipeGuard.armed()` implements exactly that — it disarms only on
`engraveRunning`/`engraveStopping`. Clamping steel and seating the needle
routinely takes more than three minutes, and it is done with hands in the
machine and eyes off the screen.

So on the first hardware run of this feature, the expected experience of 8.3 is:
the operator selects the plate, starts setting up, and at 3:30 the session
silently wipes — plate clamped, needle positioned, and the cost is a full
re-unlock (twelve words on the touch keyboard plus a ~31 s KDF) before they can
even reach the engrave screen again. Nothing warns them.

This is the design behaving as specified, which is why it belongs in the
procedure rather than in a bug report — but it is also the most likely thing to
turn a two-hour session into a three-hour one, and it is real UX signal that
should be written down whether or not it bites.

**Suggested fix.** Add to 8.3's preamble:

> **Set the plate up BEFORE you unlock**, or touch the screen at least once
> every three minutes while you work. From the moment the plate is selected the
> engrave screen is *armed* (§10.2.4: hold-to-start is a walk-away state), and a
> setup that runs past 3:30 untouched wipes the session with the plate clamped.
> If it happens, **record it** — it is the feature working, and it is what an
> operator will actually experience.

---

### I2 — The flash/seal/load procedure has no commands, and the order Task 8 implies forfeits F-100 for free.

**Location:** plan L1394 ("Seal vector F, load"), L1386–1388 (flash).

Grepped: the plan mentions `picotool` **once** (in the "never do this" sense),
`me seal` **never**, BOOTSEL once (in an aside about last week), and PD power
only in the precondition list at L1550 as something Task 8 *fails* to name.
Concretely missing:

- **`me seal` needs `--seal-secret`** — required for ms1/BIP-39 records
  (`crates/me-cli/src/main.rs:82-87`), and vector F's fifteen secret records
  include three `ms1`. The obvious command fails.
- **`--out` is required** and the tool then prints the load line itself:
  `picotool load --verify <out>   (machine in BOOTSEL)` (main.rs:391-393).
- **BOOTSEL entry** — hold the button while connecting USB (sh2-flash says this
  in its failure message, but only *after* the build and sign have run).
- **PD power** — `Init()` requires a 20–28 V contract before LCD init, so a
  machine judged on a laptop port shows a dark screen and re-enumerates as
  RP2350 Boot, which is indistinguishable from a signature rejection. sh2-flash
  prints this at the end; the plan should not depend on the operator reading a
  script's epilogue for a step it does not otherwise mention. §11.5 requires it
  and the plan's own precondition list already flags that Task 8 omits it
  (L1549-1551).

**And the order is the free win nobody takes.** F-100 — SPEC §11.5's *"confirm
firmware reflash preserves the blob"* — is an open release-tag precondition
owned by nobody-in-particular, and the follow-up entry says why it is still
open: *"B2a-ii's Task 9 does not cover it either — 9.1–9.2 load the payload
**after** the firmware."* Task 8 is about to repeat that order and lose it
again. Loading the payload **first**, then flashing, then unlocking, closes
F-100 on this trip at zero risk: if the reflash did destroy the blob, the
payload is regenerable from the host in one command and the operator simply
re-loads it and continues.

**Suggested fix.** Insert an **8.0** before 8.1:

> **8.0 Host, then device, in this order.** All of it with the machine in
> BOOTSEL (hold the button while connecting USB; confirm with
> `lsusb | grep 2e8a:000f`).
> 1. `me seal --seal-secret --iterations 300000 --out /tmp/f.uf2 <vector F's 15
>    records>` — **write the printed twelve-word passphrase down now**; it is
>    generated, printed once to stderr, and cannot be recovered from the device.
> 2. `picotool load --verify /tmp/f.uf2` — the payload, **before** the firmware.
> 3. `SH2_REPO=/scratch/code/shibboleth/seedhammer-b2b ~/bin/sh/sh2-flash`
>    (see C1 for the pass condition on its Build header).
> 4. Move the machine to its **own 20–28 V PD supply** before judging anything.
>    On a laptop port a correctly signed image still gives a dark screen.
> 5. **F-100, closed for free:** confirm *Sealed Payload* is still in the menu
>    after the reflash and that the payload unlocks. The blob was written
>    before the firmware, so this answers §11.5's "confirm firmware reflash
>    preserves the blob" — an open release-tag precondition — on a trip you are
>    already making. If it did **not** survive, that is a major finding: record
>    it, re-load the payload, and carry on with the rest of Task 8.

---

### I3 — The §7.1 KDF measurement is claimed by the precondition list but appears in no step of Task 8, and the method it inherits does not exist in this configuration.

**Location:** plan L1540–1541 vs. Task 8's steps.

The precondition set asserts:

> **Task 8.1 already unlocks on the real machine, so recording the derivation
> time in Task 8.5 closes §7.1 for free and makes the two trips one.**

Three problems.

1. **No step says to.** 8.1 says "unlock"; 8.5 says "record verbatim". Neither
   mentions the KDF, a stopwatch, or an iteration count. The claim above is in
   a section the operator has no reason to read while standing at the machine.
2. **The method it defers to is unavailable.** B2a-ii Task 9.3 asks for the
   elapsed time, iteration count and rate *"from the screen and from the log"*,
   where the log is
   `log.Printf("seal: kdf %d iterations in %s derived (%s wall)", …)`
   (`gui/unlock_kdf.go:245`). But the machine must run on its own PD supply to
   boot at all, and `build-firmware` (`flake.nix:111`) passes
   `-target pico-plus2 …` with no `debug` tag, so `cmd/controller/debug_sh2.go`
   (`//go:build tinygo && rp && debug`) is not in the signed image. There is no
   console on the configuration under test.
3. **The screen does not carry the numbers either.** `unlockKDFLead` renders
   `"Unlocking. About %d seconds left."` — an **ETA**, never an elapsed time —
   and the only other figure is a percentage. No iteration count is displayed.

What *is* available is enough, and it is better than the log: the operator chose
the iteration count themselves on the `me seal` command line (default 300,000,
`main.rs:90`), and the progress screen's appearance and disappearance are both
sharp events.

**Suggested fix.** Make it a step, not an aside — and take it on the **first**
unlock, before the operator has unlocked twice without timing anything:

> **8.1a — §7.1, and it closes an open release precondition.** On the first
> unlock, stopwatch from the appearance of the `Unlocking …` progress screen to
> its disappearance. Record: the `--iterations` value you sealed with, the
> elapsed seconds, and the quotient (iterations ÷ seconds) as the in-situ
> RP2350**B** rate. Compare against §7.1's 9,715 it/s from an RP2350A. **There
> is no console on this build** — `build-firmware` passes no `debug` tag and the
> machine is on PD power, not USB — so the stopwatch is the measurement, not the
> log line in `unlockDerive`.

---

### I4 — 8.5 says "record verbatim" but not *what*, and every step above it is phrased as a tick-box over a number.

**Location:** plan L1403–1406 (8.5).

F-99's entire point was that Task 8 must not ratify a timing the normative text
never chose; the spec was amended on 2026-08-09 precisely so this run would
confirm a stated reading. But "Confirm the warning at 3:00" invites a tick, and
a tick ratifies nothing — it is indistinguishable from "looked fine", which is
the failure mode the step's own warning box ("Watch what you paste … Record what
the screen showed") is aimed at. The B1 result document is the model: it records
numbers, screen text, and dot counts, not check marks.

**Suggested fix.** Give 8.5 the actual list:

> **8.5** Record in `design/HARDWARE_RESULT_<date>_phaseB2b.md`:
> - the firmware version string from the start screen, and the `sha256` line
>   `sh2-flash` printed for the image it flashed;
> - `me seal`'s `--iterations`, the stopwatched KDF elapsed, and the derived rate
>   (8.1a);
> - for 8.1: stopwatch reading at the warning, the countdown's **first** number,
>   stopwatch reading at the wipe, and whether the change to the start screen was
>   instantaneous;
> - the warning screen's **verbatim** title and body text, and a photograph of it;
> - for 8.2: the reading at which the warning **reappeared** after the dismissing
>   touch;
> - for 8.3: the cut's length, the reading at which the screensaver appeared
>   mid-cut, and the two readings measured from the plate-done screen;
> - for 8.4: what the start screen showed (payload entry present? pager dot
>   count?) and what the re-unlock cost;
> - anything the machine did that is not in this list.

---

### M1 — Nothing asks the operator to look at the warning *as a rendered thing*, and this is the only chance.

**Location:** plan L1394 (8.1); `gui/wipe_warning.go:44-58`.

`wipeWarningOp` is a brand-new full-screen panel — a 480×320 layout with a
title and a five-paragraph body, drawn in `descriptorTheme` over whatever screen
was parked. It has never been on glass. B1's own hardware result records the gap
that makes this hardware-only:

> *"`uiContains` (`gui/gui_test.go:516`) compares **extracted text, not pixels**,
> so no test in this suite can catch a mis-drawn glyph. That is how the missing
> `·` (F-78) survived."*

Two things only glass can answer: whether the `It will be erased in N seconds`
line — the one that carries the timing — is on-screen and legible, and whether
any part of the parked screen shows through. In 8.1 the parked screen can be
`SeedScreen.Confirm`, i.e. **the twelve words**. (Tracing says it will be clean:
`draw()` repaints the full display from a single op whose backmost layer is
`op.Color(buf, th.Background)`. But "the seed is not visible behind the privacy
blanking" is worth one deliberate look rather than an inference.)

**Suggested fix.** In 8.1: *"Photograph the warning screen. Confirm the countdown
line is fully on-screen and readable at arm's length, and that no text from the
screen underneath — in this step, the twelve words — is visible anywhere behind
or around it."*

---

### M2 — Task 8 needs three or four separate unlocks; the passphrase exists in exactly one place and the step chain is unstated.

**Location:** plan L1394–1402.

`me seal` **generates** the passphrase and prints it once to stderr; there is
deliberately no way to supply your own (`crates/me-cli/src/main.rs:60-66`,
`src/seal/passphrase.rs:15`). It is a checksum-valid twelve-word BIP-39 mnemonic
(verified: `generates_twelve_valid_lowercase_words`), so it types fine on the
device keyboard — but it is unrecoverable if not written down, and the device
cannot help.

Task 8 then consumes it repeatedly and never says so. 8.1 ends in a wipe, so
**8.2 requires a fresh unlock**; 8.3 can continue in 8.2's session; 8.3 ends in a
wipe, which is the natural setup for 8.4. That is three unlocks minimum —
three × (twelve words on a touch keyboard + ~31 s of KDF). An operator who has
not planned for it will discover it one word at a time.

Note also that **8.4 is already observed at the top of 8.2** — the first
re-unlock after 8.1's wipe *is* "a re-unlock after a wipe costs the twelve words
and the KDF". It is only *recorded* if the operator knows to record it there.

**Suggested fix.** State the chain in Task 8's preamble: *"unlock → 8.1 (wipe) →
unlock → 8.2 → 8.3 in the same session (wipe) → 8.4. Three unlocks. Have the
twelve words in front of you; they exist only on the host terminal where `me
seal` printed them."* And note that 8.4's observation is available at the first
re-unlock.

---

### M3 — 8.2's "no wipe occurs" has no duration, so two operators will record different results from the same machine.

**Location:** plan L1397–1398 (8.2).

> **8.2** Repeat, touching during the warning: confirm the window resets and no
> wipe occurs.

"The window resets" and "no wipe occurs" are both satisfied by *nothing visibly
happening*, and the step gives no period over which nothing must happen. One
operator taps, sees the underlying screen return, waits fifteen seconds, and
writes PASS. Another waits a further three minutes for the warning to come back
and writes a much stronger PASS. Only the second discriminates: a reset window
means the warning **reappears at 3:00 from the dismissing touch**, and a
half-reset one would fire early or immediately.

**Suggested fix.** *"Tap once while the countdown is running. Confirm the warning
is replaced by the screen underneath it, restart the stopwatch at the tap, and
wait: the warning must **reappear at 3:00** and not before. Record the reading."*

Worth adding while they are there, because it is the safety property behind the
tap and it is free to check: *"confirm the tap did not activate whatever control
was under it"* — the dismissing touch is deliberately swallowed
(`if !a.idle.active { ctx.Router.Events(…) }` runs while `a.idle.active` is
still true), and on a hold-to-start or Confirm screen a leaked tap would be a
real problem.

---

### M4 — 8.4's "the payload is intact in flash" names no observable, and may or may not mean a power cycle.

**Location:** plan L1401–1402 (8.4).

"Intact in flash" is not something an operator can see; they can only see its
consequences. After a wipe the machine is at the start screen, and the
consequences are concrete and already have B1 baselines to compare against:
*Sealed Payload* still present in the menu, the pager showing the same dot
count, and a re-unlock reaching the same plate list with the same §6.6 hash.
(`uiFlow` re-probes the region on every session entry — `PayloadReader().Probe()`
at `gui/gui.go:1600` — so this is a genuine re-read, not a cached menu.)

Separately: if "intact in flash" is meant to include *across a power cycle*, say
so; if it is meant to be the reflash case, that is F-100 and belongs in 8.0
(I2). As written it is the weakest of the three and reads like whichever one the
reader already had in mind.

**Suggested fix.** *"After the wipe, confirm on the start screen: Sealed Payload
still present, pager dot count unchanged from before. Re-unlock: confirm it
requires the twelve words and a full KDF (no shortcut, no cached key), and that
the §6.6 hash and plate list match what 8.1 showed. Then power-cycle the machine
and confirm the entry is still there."*

---

### N1 — "Seal vector F" is ambiguous between F's *records* and F's *fixtures*, and the two give different iteration counts.

**Location:** plan L1394 (8.1).

`seal/testdata/vectors.json` vector F carries its own `passphrase`
(`beef` ×12 — measured checksum-**valid**, so it would type and unlock fine),
`iterations: 100000`, and a full `blob_hex`. B2a-ii Task 9.1 says "vector F's
**shape**", i.e. re-seal F's records with `me seal`, which generates its own
passphrase and defaults to **300,000** iterations. Task 8 drops the word
"shape".

It is only a Nit because both routes work, but the iteration count is the
divisor in I3's rate measurement and it triples the wait at every one of the
three unlocks. Say **"vector F's records, re-sealed with `me seal`
(`--iterations 300000`, the shipping default, so the measured rate is against
the number operators will actually experience)"**.

---

## Order of the steps

The chain 8.1 → 8.2 → 8.3 → 8.4 is sound and no step invalidates a later one:
8.1's wipe is the precondition for 8.2's re-unlock; 8.3 inherits 8.2's live
session; 8.3's completion-wipe sets up 8.4. **What is wrong is the order of the
things Task 8 does not mention** — the payload load relative to the firmware
flash (I2), and the KDF timing, which must be taken on the *first* unlock or it
will be taken never (I3). Add 8.0 and 8.1a and the order is right.

---

## What I checked and found sound — recorded so it is not re-spent

- **Both secret engrave arms register the job with the guard.** `unlockEngraveCodex32`
  and `unlockEngraveMnemonic` each do `g.job = scr.job` with a paired
  `defer func() { g.job = nil }()`, and `NewEngraveScreen` (`gui/gui.go:2699-2705`)
  constructs `job` eagerly — so `g.job` is never nil while a cut runs. There is
  no arm on which a wipe could fire with the needle down. (Worth stating because
  vector F is all-`ms1`, so 8.3 exercises only the codex32 arm on hardware.)
- **No wipe can catch the needle down.** `armed()` returns false for
  `engraveRunning` and `engraveStopping`; the remaining states (`engraveIdle`,
  `engraveStopped`, `engraveFailed`, `engraveDone`) are all states in which the
  machine has stopped moving. `Engrave`'s `defer s.job.Stop()` runs on the
  unwind.
- **No spurious events can corrupt the 3:00 measurement.** On the device,
  `AppendEvents` (`cmd/controller/platform_sh2.go:369`) appends only touch events
  and `p.stdin` (debug builds only); `p.wakeups` returns with `len(evts) == 0`
  and therefore does not refresh `a.idle.start`. A stopwatch reading is
  trustworthy.
- **The dismissing touch does not leak to the parked screen's widgets** —
  `Router.Events` is skipped while `a.idle.active` is still true, so the tap that
  clears the warning is swallowed exactly as a screensaver dismissal is.
- **The generated passphrase is typable on the device** — checksum-valid
  twelve-word English mnemonic, and `beef` ×12 is too (computed against
  `bip39/wordlist.txt`), so neither route hits the `isMnemonicComplete && Valid`
  gate.
- **The KDF cannot be wiped out from under itself.** `ctx.wipe` is installed by
  `unlockSecretSession`, which runs *after* `unlockAttemptOnce`, so during the
  derivation `armed()` is false and `ctx.KeepAwake()` (F-93) is honoured.
- **The screensaver, not the warning, covers a long cut** — `armed()` false plus
  no `KeepAwake` caller on the engrave path — which is what makes the plate-done
  screen a crisp t=0 for 8.3(c).
