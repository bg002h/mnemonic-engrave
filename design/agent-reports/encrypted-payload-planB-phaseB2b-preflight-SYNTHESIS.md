# Phase B2b — preflight synthesis: one go/no-go list for the operator

**Date:** 2026-08-09
**Diff under review:** `git -C /scratch/code/shibboleth/seedhammer-b2b diff a01b666..b2b`
(6 commits, 18 files; b2b tip `920e1e1`)
**Plan:** `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md`
**Spec:** `design/SPEC_encrypted_payload_delivery.md` §10.2.4

Synthesis of four independent preflight lenses (brick/hang, wipe-safety,
procedure, regression), deduplicated. Every finding below survived an
adversarial refutation pass in its own lens; no finding was added here and no
severity was softened. Source reports:

- `…-preflight-brick-hang.md`
- `…-preflight-wipe-safety.md`
- `…-preflight-procedure.md`
- `…-preflight-regression.md`

**Not re-derived here** (settled, per the dispatch brief): plan correctness
(4 R0 rounds, 0C/0I), the mutation table (16/16 applicable rows KILLED), the
build/vet/gofmt/TinyGo-size green, and the unwind design itself.

---

## 1. VERDICT

# GO-WITH-CHANGES

**Deduplicated counts: 1 Critical, 5 Important, 8 Minor, 3 Nit.**

The **firmware** is not the problem. Nothing in this diff was found that bricks
the machine, ruins a plate, or fails to wipe on the paths §10.2.4 arms. The
Critical and four of the five Importants are in the **procedure** — Task 8 as
written flashes the wrong commit, and then asks for observations it cannot
collect. The fifth Important is a **code decision** (an in-source comment that
affirmatively claims something false about seed residue) that should be made
consciously before flashing rather than discovered later.

All blockers are edits to a markdown file and, at most, a six-line Go change.
None requires re-opening the design.

| Lens | C | I | M | N |
|---|---|---|---|---|
| procedure | 1 | 4 | 5 | 0 |
| wipe-safety | 0 | 1 (dedup) | 3 | 1 |
| brick/hang | 0 | (dedup) | 1 | 1 |
| regression | 0 | (dedup) | (dedup) | 1 |
| **deduped total** | **1** | **5** | **8** | **3** |

Deduplications applied: the `op.Buffer` residue was reported independently as
brick **F4**, wipe-safety **I1** and regression **F1** → counted **once** as
Important (B2). `armed()`'s dependence on `Status()`'s side effects was
wipe-safety **M1** and regression **F2** → counted **once** as Minor (A1).
"Armed during plate setup / seed confirm" was brick **F5**, wipe-safety **M2**
and procedure **I1** → counted **once** as Minor (A2), with its procedural half
in §3.

---

## 2. BLOCKERS — change these before flashing

### B1 — CRITICAL — `sh2-flash` with no arguments builds the phase's PARENT commit

*(procedure C1)*

**What.** Task 8's only flash instruction is the bare command
`~/bin/sh/sh2-flash`. The script defaults `SH2_REPO=/scratch/code/shibboleth/seedhammer`
(line 33) and builds `REF=HEAD` (line 132) in that repo. **Verified again for
this synthesis:** that repo is on branch `main` at `a01b666` — the exact
baseline this phase is diffed against. All of B2b lives in the separate
worktree `/scratch/code/shibboleth/seedhammer-b2b` at `920e1e1` on branch `b2b`,
unmerged. `gui/wipe_warning.go`, `gui/wipe_guard.go` and `gui/run_flow.go` do
not exist in the default repo at all.

**Where.** Plan L1386-1388 (Task 8 preamble).

**What the operator would see.** They flash, run 8.1, and at 3:00 get the
*ordinary screensaver* instead of the warning; at 3:30, nothing. 8.1, 8.2 and
8.3 all fail identically. The hardware document that will be cited as this
phase's release evidence records a categorical failure of a feature that is
present and correct — and the trip is spent.

**Minimal fix.** Replace the flash sentence with:

```sh
SH2_REPO=/scratch/code/shibboleth/seedhammer-b2b ~/bin/sh/sh2-flash
```

and give it a **pass condition**: the script's `== Build ==` header prints
`git log --oneline -1` of the tree it builds and MUST read `920e1e1` (or the
b2b tip). **If it reads `a01b666`, stop and fix the command.** Require the
worktree be clean first (`sh2-flash` warns `tree is dirty` and stamps
`-dirty` into the version string). Record the StartScreen version string in 8.5
as after-the-fact proof of which image was tested.

---

### B2 — IMPORTANT — the wipe does not erase the rendered seed, and the diff's own comment says it does

*(wipe-safety I1 = regression F1 = brick F4 — one defect, found three times independently)*

**What.** `gui/run_flow.go`'s session-loop tail comment states *"The abandoned
Context's buffer is already zeroed by the time control reaches this line."*
That is false. `op.Buffer.Reset()` (`gui/op/op.go:374-378`) does
`b.args = b.args[:0]` — a **truncation**, not a zeroing — and `clear()`s only
`b.refs`. `op.Glyph` (`op.go:132`) encodes every rendered rune into `args` as a
`uint32`. `SeedScreen.Draw` renders all twelve/twenty-four words through that
path into `ctx.B`.

**Machine-checked, independently, by two lenses, against this worktree:**

```
before Reset: len(args)=904  cap(args)=1344   Buffer.Len()=(904,336)
after  Reset: len(args)=0    cap(args)=1344   same backing array = true
recovered from the args BACKING ARRAY after Reset:
  "1: abandon 2: ability 3: able 4: about 5: above 6: absent
   7: absorb 8: abstract 9: absurd 10: abuse 11: access 12: accident"
non-nil entries in the refs BACKING ARRAY after Reset: 0 of 511
```

The refs *are* scrubbed. The args are not — the full plaintext comes back
verbatim and in order. The regression lens reproduced the same result through
the real `runWithFlow` + `SeedScreen.Draw` path after an actual armed wipe.

**This diff aggravates it.** Pre-diff, one `Context` lived for the process, so
`args` was continuously overwritten by later frames. Moving
`ctx := NewContext(pl)` inside the session loop **abandons** the wiped session's
buffer and hands the restart a fresh zero-value one — freezing the old array,
holding the last frame drawn before the wipe, until TinyGo's GC happens to
recycle it. Measured on the pre-diff shape, only the *head* of the array gets
overwritten by subsequent short frames (8/12 words still verbatim).

**Where.** `gui/run_flow.go:236-245`; `gui/op/op.go:374-378`; `gui/op/op.go:132`;
`gui/gui.go:2536` (`SeedScreen.Draw` → `layoutWord` → `widget.Label` → `op.Glyph`).

**What the operator would see.** Vector A/B, operator reaches
`SeedScreen.Confirm` and is called away. 3:00 warning, 3:30 wipe: seal's record,
`bip39.Parse`'s `[]Word`, the key, the passphrase and the blob are all zeroed;
the Context is dropped; the machine shows the main menu; **the operator believes
RAM holds nothing.** An SWD probe reads the twelve words out of the abandoned
`ctx.B.args` backing array. §10.2.4's own stated threat model (§2.2 item 9) is
exactly physical access plus an SWD probe.

**Minimal fix.** Add beside the `Buffer.Len()` this phase already added:

```go
func (b *Buffer) Scrub() {
	clear(b.args)
	clear(b.refs)
	b.args, b.refs = b.args[:0], b.refs[:0]
}
```

and call `ctx.B.Scrub()` in `runWithFlow`'s wipe branch before the Context is
dropped — i.e. exactly where the comment currently explains that nothing is
needed. **Replace that comment either way**; as written it is the one thing
that would stop the next reviewer from looking. State honestly that `append`
growth orphans earlier `args` arrays no handle can reach, and add that residual
to the F-88/F-83 list. `a.warnBuf` outlives every session and has the same
property.

**On the severity.** The reporting lens explicitly declined to call this
Critical — the residue is unreferenced heap, §10.2.4's residency definition is
written about records, and the spec already concedes the SWD attacker. It also
stated that *a Critical call would be defensible and should be decided before
flashing*. That decision is the operator's; it is recorded here unsoftened.
Nothing about this blocks the *hardware* observations of Task 8 — a
scrub-or-reword decision can be taken independently of the trip.

---

### B3 — IMPORTANT — Task 8 has no seal / load / BOOTSEL / PD commands, and its implied order forfeits F-100 for free

*(procedure I2)*

**What.** Grepped across the plan: `picotool` appears once (as a prohibition),
`me seal` **never**, BOOTSEL once as an aside, PD power only at L1550 as
something Task 8 fails to name. Three words — *"Seal vector F, load"* — stand in
for: a host command that **fails without `--seal-secret`** (`me-cli/src/main.rs:82-87`,
enforced at :327) because F carries three `ms1` records; a payload load that
needs the device in BOOTSEL; and a power change that decides whether the machine
boots at all (`platform_sh2.go` `minVoltage = 20_000`, `monitorPowerSupply`
reboots into BOOTSEL and panics on failure — a dark screen indistinguishable
from a signature rejection).

Separately: **F-100** is an open release-tag precondition ("confirm a firmware
reflash preserves the blob", SPEC §11.5). Its own entry records that B2a-ii's
9.1-9.2 load the payload *after* the firmware, which is why it is still open —
and Task 8 is about to repeat that order.

**Where.** Plan L1394 and L1386-1388.

**What the operator would see.** `me seal` refuses; or the machine boots dark
and the operator cannot tell whether the image was rejected; or everything works
and F-100 is silently left open, requiring another trip to the machine later.

**Minimal fix.** Insert an **8.0** with the ordered commands:

1. `me seal --seal-secret --iterations 300000 --out /tmp/f.uf2 <F's 15 records>`
   — **write down the printed passphrase** (see B4 and M-a).
2. `picotool load --verify /tmp/f.uf2` — **payload BEFORE firmware** (this is
   what closes F-100).
3. The `SH2_REPO=`-qualified `sh2-flash` from B1.
4. Move to the 20-28 V PD supply **before judging anything**.
5. Confirm *Sealed Payload* is still present and still unlocks. **That closes
   F-100 / SPEC §11.5 at zero cost.** If it does *not* survive, that is a major
   finding: record it, re-load, continue.

---

### B4 — IMPORTANT — 8.1 says "walk away", then asks for three observations that exist only as instants

*(procedure C2; the post-wipe half is also wipe-safety N1, §4)*

**What.** *"Confirm the warning at 3:00, the wipe at 3:30, and that the machine
returns to the main menu … not a reboot."* The first two are **transitions**,
visible only to a present observer. The third is **unobservable after the fact**:
on a wipe the session loop re-enters `flow()` and `uiFlow` (`gui/gui.go:1581`)
starts at `StartScreen` with the same version line — which is exactly where a
reboot also lands. The only discriminator is that the unwind is instantaneous,
with no LCD init and no PD re-negotiation. That is a *timing* observation.
There is no console on this build and no uptime/boot counter to fall back on.

**Where.** Plan L1394-1396.

**What the operator would see.** An operator who literally walks away returns to
a StartScreen and writes *"returned to main menu, still usable — PASS"* — the
same sentence they would write for a machine that rebooted. This is the first
time `ctx.Done` has ever been true in production, and it is the trip's entire
purpose; the step as written **cannot collect the data it exists for**.

**Minimal fix.** "Walk away" must mean **do not touch**, not leave the room.
Rewrite 8.1 as a stopwatch procedure (see §3) and **point a phone at the screen
with a stopwatch in frame** for 8.1-8.3. That makes every timing re-readable,
captures the warning text verbatim for 8.5, and settles the reboot question on
playback.

---

### B5 — IMPORTANT — the §7.1 KDF measurement is claimed by the precondition list but appears in no step, and its inherited method does not exist on this build

*(procedure I3)*

**What.** The release-precondition list (L1540-1541) says *"recording the
derivation time in Task 8.5 closes §7.1 for free"*. **No step mentions the KDF,
a stopwatch, or an iteration count.** The method it defers to (B2a-ii step 9.3:
*"from the screen and from the log"*) is unavailable here: the log is a
`log.Printf` at `gui/unlock_kdf.go:245`, `flake.nix:111` builds with **no
`debug` tag** so `cmd/controller/debug_sh2.go` (`//go:build tinygo && rp && debug`)
is absent, and the machine must run on PD power rather than a USB host. The
screen shows only a percentage and *"About N seconds left"* — an **ETA, never an
elapsed time** — and no iteration count.

**Where.** Plan L1540-1541 vs the steps at L1394-1403.

**What the operator would see.** Nothing — which is the problem. §7.1 stays open
on an RP2350**A** figure, §12.1's last open item stays open, and it is another
trip to the machine.

**Minimal fix.** Add an **8.1a**, taken on the **first** unlock: stopwatch from
the appearance of the *Unlocking* progress screen to its disappearance; record
the `--iterations` value you sealed with, the elapsed seconds, and the quotient
as the in-situ RP2350B rate; compare against §7.1's 9,715 it/s. **State
explicitly that there is no console on this build, so the stopwatch is the
measurement.**

---

### B6 — IMPORTANT — 8.5 says "record verbatim" without saying what, and every step above it is a tick-box over a number

*(procedure I4)*

**What.** F-99 was closed by amending §10.2.4 so this run would confirm a
**stated** timing reading (warn @ 3:00 / wipe @ 3:30, additive). But *"Confirm
the warning at 3:00"* invites a check mark, and a check mark ratifies nothing —
it is indistinguishable from *"looked fine"*, which is the exact failure the
step's own **"Watch what you paste"** box is aimed at. The B1 hardware result is
the model to copy: it records numbers, screen text and dot counts.

**Where.** Plan L1403-1406.

**What the operator would see.** A hardware document full of ticks, which cannot
later distinguish *"the warning appeared at 3:00"* from *"the warning appeared
at some point while I was away"*. The amended normative timing would be ratified
by an artifact that never measured it, and nothing can be re-derived without
another trip.

**Minimal fix.** Enumerate the record in 8.5 — the list is in §3 below.

---

## 3. DO THIS AT THE MACHINE

The operator gets **one trip**. A forgotten measurement is another trip. This
section is the delta between what Task 8 currently says and what should actually
be done.

### Before you leave the desk

- [ ] **Set up: payload first, firmware second.** `me seal --seal-secret
      --iterations 300000 --out /tmp/f.uf2` over vector F's **records**
      (not the `vectors.json` fixture blob — see M-d), then
      `picotool load --verify /tmp/f.uf2` with the machine in BOOTSEL, **then**
      the firmware. Confirming *Sealed Payload* still unlocks after the reflash
      **closes F-100 / SPEC §11.5**. *(B3)*
- [ ] **Write down the twelve-word passphrase.** `me seal` **generates** it and
      prints it once to stderr; there is deliberately no way to supply your own,
      and the device cannot recover it. It exists **only** on the host terminal.
      Without it you cannot complete Task 8 at all. *(M-a)*
- [ ] **Flash the right tree.** `SH2_REPO=/scratch/code/shibboleth/seedhammer-b2b
      ~/bin/sh/sh2-flash`. **Read the `== Build ==` header: it must print
      `920e1e1`.** If it prints `a01b666`, stop. *(B1)*
- [ ] **Move to the 20-28 V PD supply** before judging whether the machine
      booted. A dark screen on USB power is not a signature rejection. *(B3)*
- [ ] **Plan for three unlocks.** 8.1 ends in a wipe → unlock for 8.2 → 8.3
      continues in 8.2's session → 8.3's completion sets up 8.4. Each unlock is
      twelve words on a touch keyboard plus ~31 s of KDF. Note that **8.4's
      observation is already available at the first re-unlock, at the top of
      8.2** — take it there so it does not go unrecorded. *(M-a)*
- [ ] **Point a phone at the screen with a stopwatch in frame** for 8.1-8.3.
      *(B4)*

### 8.1 — the walk-away wipe

"Walk away" means **do not touch the machine**, not leave the room. Stand where
you can see the screen.

- [ ] Start a stopwatch at the **release of the last touch** (OK on the twelfth
      word).
- [ ] **8.1a — KDF (do this on the FIRST unlock, it is the only free chance):**
      stopwatch from the *Unlocking* progress screen appearing to it
      disappearing. Record the `--iterations` you sealed with, the elapsed
      seconds, and the quotient. Compare to §7.1's 9,715 it/s. There is **no
      console** on this build; the stopwatch *is* the measurement. *(B5)*
- [ ] Record the reading when **WIPING SECRET DATA** appears (expect **3:00**).
- [ ] Record the countdown's **first number** (expect **30**).
- [ ] Record the reading when the screen changes (expect **3:30**).
- [ ] Record that the change is **instantaneous, with no blank interval** — this
      is the only thing that distinguishes the unwind from a reboot. *(B4)*
- [ ] Record the **version string** on the start screen.

### 8.2 — the touch reset (give it a duration)

Currently *"confirm the window resets and no wipe occurs"* is satisfiable by
nothing visibly happening over an unstated period. Two operators would record
different results from the same behaviour. *(M-b)*

- [ ] Tap **once** while the countdown is running. Confirm the warning is
      replaced by the screen underneath.
- [ ] **Restart the stopwatch at the tap and wait.** The warning must reappear
      at **3:00 and not before**. Record the reading. (A half-reset window fires
      early; only this observation distinguishes a working reset from a broken
      one.)
- [ ] Free safety check: confirm the dismissing tap **did not activate whatever
      control was under it** — it is deliberately swallowed while
      `a.idle.active` is still true.

### 8.3 — the mid-cut plate (READ THIS BEFORE STARTING)

- [ ] **The engrave screen is ARMED during plate setup.** §10.2.4 as amended
      disarms only while the job is *running*; `wipeGuard.armed()` returns false
      only for `engraveRunning`/`engraveStopping`. Hold-to-start — where you
      clamp steel, seat the needle, and close the lock — is armed, and closing
      the lock generates no touch event. **Either set the plate up before
      unlocking, or touch the screen at least once every three minutes while
      your hands are in the machine.** *(A2 / procedure I1)*
- [ ] If it fires anyway: **that is the feature working.** Record it — it is
      real UX signal, and it costs a re-unlock (twelve words + ~31 s KDF), not a
      plate.
- [ ] Confirm **no wipe while the job runs**, and record the cut length.
- [ ] Record **two readings** from the plate-done screen: that the 3:00 window
      restarts **from the cut's end**, and when the warning appears.

### 8.4 — payload survival (name the observables)

*"the payload is intact in flash"* is not something an operator can see, only its
consequences. The observables exist and have **B1 baselines**. *(M-c)*

- [ ] On the post-wipe start screen: **Sealed Payload still present**, and the
      **pager dot count unchanged** (B1: absent = 8 dots, present = 9 dots).
- [ ] Re-unlock: confirm it requires the **twelve words and a full KDF**, and
      that the **§6.6 hash and plate list match 8.1's**. (`uiFlow` re-probes the
      region on every session entry, `gui/gui.go:1600` — this is a genuine
      re-read, not a cached menu.)
- [ ] **Power-cycle** and confirm the entry is still there.

### 8.5 — what to record (the enumerated list B6 asks for)

Write into `design/HARDWARE_RESULT_<date>_phaseB2b.md`:

1. Firmware **version string** and `sh2-flash`'s printed **sha256**.
2. `--iterations` sealed with, **KDF elapsed seconds**, **derived rate**.
3. 8.1's **three stopwatch readings**, the countdown's **first number**, and
   **whether the transition was instantaneous**.
4. The warning's **verbatim title and body**, plus a **photograph**.
5. 8.2's **reappearance reading**.
6. 8.3's **cut length** and the **two readings** from the plate-done screen.
7. 8.4's **start-screen state**, dot count, §6.6 hash, and what the re-unlock
   cost.
8. The F-100 result: **did the sealed payload survive the firmware reflash.**
9. **Anything the machine did that is not on this list.**

---

## 4. ACCEPTED RISKS

Real, will not be fixed before this flash, and the operator should simply know.

**A1 — `armed()` looks like a pure predicate but depends on a mutating
accessor.** *(wipe-safety M1 = regression F2; Minor)* `wipeGuard.armed()` calls
`j.Status()`, which is **not** a query: it drains `e.progress` and `e.errs`,
performs the `engraveRunning → Done/Stopped/Failed` transition, and contains an
`if State == engraveRunning { e.Start() }` branch. That drain is **load-bearing**
— it is what lets a 21-minute cut finishing behind the screensaver re-arm the
timer, because `armed()` is the only caller of `Status()` on that tick. The
wipe-safety lens **reproduced** this: applying the obvious "make the predicate
pure" refactor (`j.Status().State` → `j.status.State`) leaves the **entire
`./gui/` package green** (`ok seedhammer.com/gui 50.645s`) while silently
disabling the wipe for the rest of the session, seed resident, saver up. A
production-faithful repro test (cut ending via `errs <- nil`) passes at HEAD and
fails under the refactor; it lives at
`…/scratchpad/zz_repro_test.go` and is the pinning test the fix section asks for.
`armed()` also now polls a running secret job roughly every 40 ms — on the order
of 30,000 extra `Status()` calls across a plate. **No behaviour change today;
the risk is the next refactor.** Fix is documentation plus that pinning test.

**A2 — the armed window covers screens where a three-minute pause is normal
operator behaviour.** *(brick F5 = wipe-safety M2 = procedure I1; Minor)* This is
**deliberate** — §10.2.4's amendment arms walk-away states on purpose. But it
means `SeedScreen.Confirm` (checking 24 words against a written copy is 7.5 s
per word at 3:00) and hold-to-start (seating steel, eyes off the screen) both
run the 3:30 clock, and **only real input events refresh it** — `KeepAwake`
cannot. Fully recoverable: twelve words plus a ~31 s KDF, sealed blob untouched.
On vector F, though, the unwind walks every remaining record and each
`defer p.WipeSecretAt(i)` fires, so **every not-yet-cut secret card (2 and 3 of
a 2-of-3) goes with it.** Note also that the scripted 8.1 walk-away on vector F
never reaches `SeedScreen.Confirm` at all (`ms1` goes through
`unlockEngraveCodex32`, which has no confirm screen).

**A3 — anything resting on the touch panel refreshes the one clock
indefinitely.** *(wipe-safety M3; Minor)* `processTouch` emits an event on **any**
change in `(touching, tp)`, and `len(evts) > 0` is the timer's primary refresh.
A hand, plate or tool left on the panel produces a continuous event stream as the
reported point wanders, so the 3:00 never elapses. A perfectly still object
produces one event and then none, so the wipe does fire. **Condition-dependent
and unmeasured on the real panel.** Pre-existing as an input path (the
screensaver shares it) — but the screensaver was not a security control.
**Free bench check while you are there: rest an object on the screen and watch
whether the saver ever activates.** Cleared while checking: no non-touch
production source can refresh the clock (`p.wakeups` returns `evts` unchanged, so
the engrave goroutine's `Wakeup()` and the NFC poller do not reset it; `p.stdin`
exists only under `//go:build tinygo && rp && debug`, so a UART host cannot
postpone a wipe on a release build).

**A4 — two `for { … Engrave() … }` loops have no `ctx.Done` test.** *(brick F1;
Minor)* `gui/gui.go:2251-2256` (`backupSeedStringFlow`) and
`gui/slip39_polish.go:506-510` (`engraveSLIP39Verbatim`) loop until `Engrave`
returns true, but `EngraveScreen.Engrave` is `for !ctx.Done { … }; return false`
— with `Done` true it returns false immediately without drawing, yielding, or
touching the clock. That is a 100%-CPU freeze on the single UI goroutine, no
repaint, touch dead, **no watchdog**. **NOT reachable today**, verified
exhaustively: `ctx.Done` has exactly three assignments, all in `gui/run_flow.go`
(:77 dead, :185 test-only, :205 the §10.2.4 timer), and the timer requires
`ctx.wipe != nil`, assigned in exactly one place scoped to `unlockSecretSession`
— neither loop is in that call tree. Every **other** engrave-retry loop in the
package (7 of them) is gated on a screen returning `ok == false`, which `Done`
also produces; these two loop on `Engrave`'s completion boolean alone. **The
exposure is that this diff makes `ctx.Done` a live production signal for the
first time, and the invariant the whole unwind rests on is enforced by nothing,
with two counter-examples in the same package.** It is one `armed()` widening
away — e.g. arming the typed-seed path, which holds the same twelve words in the
same `SeedScreen`. Two-line fix (`for !ctx.Done {`) plus a lint or test, whenever
convenient.

**A5 — after a wipe the operator lands on the main menu with no statement that a
wipe happened.** *(wipe-safety N1; Nit)* On watchdog-less hardware, an operator
returning to a machine that was mid-session and is now at the main menu cannot
distinguish *"the idle wipe fired"* from *"it spontaneously rebooted"* — and the
alarming reading is the indistinguishable one. §10.2.4 does not require a notice.
One sentence carried into the first `StartScreen` after a wipe would fix it.
**This is why B4's timing capture matters on this trip: the discriminator is
currently only observable live.**

**A6 — `layoutTime` moved measurement point inside Task 1's "pure move".**
*(regression F3; Nit)* `a01b666` measured `layoutTime` after `d.Reset()` and
`pl.DisplaySize()`; the new code measures before both. It feeds only
`if debug { stats.Dump(…) }`, and production is `const debug = false`, so the
statement is compiled out — **zero effect on shipped firmware**. Recorded only
so the commit's "pure move" claim is accurate for whoever diffs against it next.

**A7 — the operator-facing ambiguity in "Seal vector F".** *(procedure N1; Minor)*
`vectors.json`'s vector F carries its own passphrase (`beef` ×12, checksum-valid,
would type and unlock fine) at **100,000** iterations; `me seal` generates its own
passphrase at the **300,000** shipping default. Both routes work, but iterations
is the **divisor in the §7.1 rate measurement** and triples the wait at each of
three unlocks (10.29 s vs 30.88 s). Use the **records, re-sealed at 300,000** so
the measured rate is against the number operators will actually experience —
already folded into §3.

---

## 5. WHAT THIS REVIEW DID NOT COVER

Stated honestly, so nobody reads a GO as broader than it is.

- **The plan's correctness, the mutation table, and the build.** Declared
  settled in the dispatch brief and deliberately not re-derived: 4 R0 rounds to
  0C/0I, 16/16 applicable mutation rows KILLED (2 NOT_APPLIED with reasons),
  `go build`/`go test`/`go vet`/`gofmt`/TinyGo size all measured at HEAD. **If
  any of those is wrong, this synthesis does not catch it.**
- **The unwind design itself.** Taken as blessed: every nested loop on the
  secret-session path exits on `ctx.Done`, the unwind runs every defer, and the
  discard guard was proved dead and removed. A4 is the one place where "every
  loop tests Done" was checked against the package as a whole — and it is checked
  *for today's reachability*, not for all future reachability.
- **The cryptography.** Nothing in this pass looked at the seal format, the KDF
  construction, AEAD usage, key derivation, or the wire encoding. Those belong
  to the funds-safety audit and earlier phases.
- **Anything outside the encrypted/secret-session path.** The lenses were scoped
  to what §10.2.4 touches. Public-record flows, bundle/preview, NFC and the Rust
  CLI were examined only where a finding's chain ran through them.
- **The RAM residue question beyond `op.Buffer`.** B2 establishes that
  `op.Buffer.args` retains the rendered seed. **No systematic sweep of other
  long-lived buffers was performed.** `a.warnBuf` is named as having the same
  property; `append`-growth orphans of `args` that no handle can reach are
  named as unaddressable. There may be more. `RecordsResident()` scans
  `p.Secret` and structurally **cannot** reach any of it — so the phase's own
  assertion machinery would not detect a further instance.
- **Real-hardware timing and the touch panel.** Everything about 3:00/3:30 is
  derived from constants and tests; **`ctx.Done` has never been true on hardware.**
  A3's panel-jitter question is explicitly unmeasured on the real FT6x36. That
  is what the trip is for.
- **Post-wipe GC behaviour on TinyGo.** How long the abandoned `Context`'s
  backing array survives before recycling was not measured on device; the
  worst case (it survives indefinitely) is what B2 assumes.
