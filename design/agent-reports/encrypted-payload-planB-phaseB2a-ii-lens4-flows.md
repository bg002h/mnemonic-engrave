# B2a-ii whole-diff review — LENS 4: the operator-facing flows

Reviewer: independent adversarial pass (opus), 2026-08-08.
Branch `feat/encrypted-payload-b2a-ii`, base `main @ 421dca8`, 10 commits.
Files in lens: `gui/unlock_kdf.go`, `gui/unlock_flow.go`, `gui/unlock_plates.go`,
`gui/unlock_platelist.go` (plus `gui/unlock_session.go` and the `gui.Run` frame
loop, which the first two reach).

Normative: `design/SPEC_encrypted_payload_delivery.md`. Where the spec and the
plan disagree the spec wins; nothing in this report turns on the plan.

**Method.** Every number below was measured by executing the real packages
against `sh2DisplaySize` (480×320, `cmd/controller/platform_sh2.go:34-35`) in a
throwaway copy of the worktree, never by reading a doc comment. The copy is
deleted. The worktree was not written to.

---

## Verdict

**2 Important, 2 Minor, 2 Nit. 0 Critical.**

The seven questions in the brief resolve as follows:

| # | Question | Result |
| --- | --- | --- |
| 1 | steps 5-9 in order; checksum gate before the KDF, **instrumented** | **CLEAN.** Gate is `unlock_kdf.go:217` and the KDF seam `newDeriver` (`:51`) is counted by `kdfCounter` with a positive control. |
| 2 | step 8 offers both readings and keeps the §6.6 hash | **CLEAN**, strings read and verified. |
| 3 | can the plate list be reached without a successful unlock | **NO.** Every path traced; see below. |
| 4 | three nav slots, every screen incl. nested | **CLEAN.** No `layoutNavigation` call in this phase can index past `[3]int`. |
| 5 | encrypted md1/mk1 cards present and distinguishable | **CLEAN**, verified on vectors A–G. |
| 6 | §10.2.3 warning fires iff `ct_len == 0` | **CLEAN** on the condition. Its *rendering* is Minor-4. |
| 7 | KDF loop blocking / not yielding / `ctx.Done` | **DEFECT** — Important-1. |

---

## IMPORTANT 1 — the KDF progress loop is the only unthrottled frame loop in the firmware, and `Run`'s screensaver **suspends the derivation**

**Where.** `gui/unlock_kdf.go:159-203` (`unlockDerive`), against `gui/gui.go:2915`
(`idleTimeout = 3 * time.Minute`) and `gui/gui.go:2968-2999` (`Run`'s inner loop).

**The defect.** `unlockDerive` ends every iteration with

```go
// Ask for the next frame immediately: this loop IS the work, and a
// deadline in the future would idle the KDF instead of running it.
ctx.WakeupAt(time.Now())
```

That is the only free-running frame request in the codebase. Measured — the
complete set of `WakeupAt` call sites in `gui/`:

```
gui/gui.go:1815   ctx.WakeupAt(m.scanTimeout)                      NFC scan deadline
gui/gui.go:2734   ctx.WakeupAt(time.Now().Add(time.Second / 2))    ENGRAVE screen, 2 fps
gui/gui.go:2994   ctx.WakeupAt(now.Add(minFrameTime))              screensaver, 25 fps
gui/gui.go:2997   ctx.WakeupAt(idleWakeup)                         idle deadline
gui/widget.go:64  ctx.WakeupAt(wakeup)                             key repeat
gui/unlock_kdf.go:200  ctx.WakeupAt(time.Now())                    <-- this
```

The 21-minute engrave screen — the only other long screen on the machine —
deliberately throttles to 2 fps. The 31-second unlock screen asks for frames as
fast as the panel can be repainted, and each frame is a **full 480×320 repaint**
(`Run` sets `dirty := image.Rectangle{Max: pl.DisplaySize()}` unconditionally at
`gui.go:2946`) with the title, a 45-px progress face and a wrapped lead line
re-rasterised. At the 300,000-iteration default that is
`(300000-1)/500 = 600` steps and 599 repaints.

That alone is a cost question. The failure is what `Run` does when the total
crosses three minutes:

```go
for {
        if ctx.Done || !yield() { return }
        evts = pl.AppendEvents(wakeup, evts[:0])
        now := time.Now()
        if len(evts) > 0 { a.idle.start = now }
        ...
        if a.idle.active {
                a.idle.state.Draw(pl)
                ctx.WakeupAt(now.Add(minFrameTime))
                continue                      // <-- never breaks
        }
        ctx.WakeupAt(idleWakeup)
        break
}
```

While `a.idle.active` the inner loop never `break`s, so `for content := range it`
never runs another iteration, so the flow **coroutine is never resumed** and
`d.Step(kdfStepIterations)` is never called again. The derivation does not merely
lose its progress screen — it **stops**, indefinitely, until a touch sets
`a.idle.start = now` and clears `idle`. `a.idle.start` is updated only by
`len(evts) > 0`, and an unattended KDF produces no events.

**Reachability, stated honestly.**

- **Unconditional** at `iterations >= 180 s × 9,715 it/s = 1,748,700`, with zero
  drawing cost assumed. §6.2 admits `iterations <= 2_000_000` and §9 exposes
  `me seal --iterations N`, so this is a spec-legal payload, not a hostile one.
  §7.1 computes the 2,000,000 ceiling as 205.9 s and calls it *"long, but
  bounded, which is what the no-watchdog argument requires."* This
  implementation makes it unbounded without operator interaction, so that
  argument no longer holds.
- **At the 300,000 default** it depends on per-frame draw cost on RP2350B, which
  I could not measure: it needs `> 249 ms` per full-panel frame to reach three
  minutes. I am not claiming it does. I am claiming nobody has measured it and
  that the design has no margin argument.

**Consequence.** The screen §10.2 step 7 exists to prevent ("the operator will
think the machine has hung") is replaced by a screensaver over a derivation that
really has hung. Recovery is a screen touch, which the operator has no reason to
suspect. No funds or seed exposure.

**Why no test caught it.** `Run`'s idle branch is unreachable from the gui
suite: `runUITouch` (`gui/start_screen_touch_test.go:29`) and `runUI`
(`gui/gui_test.go:503`) are bespoke pumps with no `AppendEvents`, no
`idleTimeout` and no saver. `gui.Run` is called only from `cmd/controller` and
`cmd/emu`. `TestUnlockDerivesWithARealProgressScreen` proves the loop is chunked
and the percentage advances — it cannot see this.

**Fix.** Two independent halves, both cheap:
1. Raise `kdfStepIterations` so the frame count is ~5 fps of *wall* time rather
   than 19 fps of *KDF* time (the current constant's comment reasons about the
   KDF cost per frame and treats the repaint as free). ~2,500 is ~257 ms of KDF
   per frame at §7.1's rate, still far under any "the machine is dead" threshold,
   and cuts repaints 5×.
2. Make a running derivation suppress the screensaver — the KDF screen is the
   one screen in the firmware that is *working* while untouched. Note this is
   **not** B2b's §10.2.4 residency timer; it is `Run`'s `idleTimeout`, and the
   two must not be conflated.

---

## IMPORTANT 2 — Back on a secret plate destroys the record, unconfirmed, drawn with the "go back" icon; and the plate list afterwards shows a complete-looking set

**Where.** `gui/unlock_session.go:117-128`, against `gui/gui.go:1472-1475` and
this diff's own `gui/unlock_platelist.go:161-171`.

**The defect.** `unlockSecretPlate` offers `ChoiceScreen{Choices: ["Cut this
plate", "Skip"]}` and treats Back and Skip identically:

```go
choice, ok := cs.Choose(ctx, th)
if !ok || choice != 0 {
        return                  // deferred p.WipeSecretAt(i) destroys the record
}
```

`ChoiceScreen.Choose` hard-codes its Button1 as
`{Clickable: cancelBtn, Style: StyleSecondary, Icon: assets.IconBack}`. So the
nav slot that means *"step back one screen"* on every other screen in the
firmware here **irreversibly discards a decrypted seed record**, with no
confirmation, no hold-to-confirm, and no visual signal that it is destructive.

This diff already made exactly the opposite call one file over. `unlockPlateListFlow`
changed its Button1 from `IconBack` to `IconDiscard` in this very branch, with
the reasoning spelled out in the code:

> IconDiscard, not IconBack (§10.3, F-80's B2 item). Back here is the SESSION
> exit: it discards a decrypted payload, and getting back costs twelve words and
> a ~31 s KDF.

The secret plate screen discards a **seed** record under the same recovery cost
and kept `IconBack`. The two screens are one tap apart. Compare also
`SeedScreen.Confirm` (`gui/gui.go:2325-2344`), which does gate its Back behind a
`ConfirmWarningScreen{Title: "DISCARD SEED?"}` hold-to-confirm — for a seed the
operator **typed**, which is strictly easier to re-enter than one behind twelve
words and a 31-second KDF.

**Consequence, and it is §6.4's named worst outcome.** After the last secret is
skipped — by intent or by a mis-tap — nothing anywhere records that a secret
plate existed and was not cut. §10.2.2 removes secrets from the plate list by
design, so what the operator then sees is an ordinary, complete-looking set of
mk1/md1 cards. That is precisely the mechanism §10.2.2's own "Plural,
deliberately" paragraph identifies as the harm:

> the plate list then shows only mk1/md1 so nothing looks missing, and they store
> an **incomplete backup of a 2-of-3 believing it complete** — §6.4's own "worst
> available outcome".

The implementation fixed the *plural* half of that (all three ms1 records are
offered — verified against vector F) and left the *nothing-looks-missing* half
open on the Skip/Back path. Measured on the canonical vectors: for A and B the
list is **empty** (Minor 3 below), and for C, D, F and G it is a full-looking
5/5/12/12-entry card list with no trace of the secrets at all.

**Fix** (any one closes the single-mis-tap route; the third closes the harm):
1. Pass an icon through `ChoiceScreen`, or give the secret plate its own screen,
   so Button1 reads `IconDiscard` — consistent with the ruling this branch
   already made for the plate list.
2. Require an explicit confirmation for Skip/Back on a **secret** plate
   (`ConfirmWarningScreen`, as `SeedScreen` does).
3. Carry a session tally onto the plate list — "2 of 3 secret plates cut" — so
   "nothing looks missing" stops being true. This is the one that actually
   addresses §6.4, and it costs one line of text.

---

## MINOR 3 — an encrypted-seed-only payload lands on an EMPTY plate list

**Where.** `gui/unlock_flow.go:114-116`, `gui/unlock_platelist.go:70-176`.

Measured, running `unlockPlates` over every canonical vector:

```
vector A: pub=0 secretSection=1  secrets=1 -> plateList=0     <-- empty
vector B: pub=0 secretSection=1  secrets=1 -> plateList=0     <-- empty
vector C: pub=0 secretSection=6  secrets=1 -> plateList=5
vector D: pub=5 secretSection=1  secrets=1 -> plateList=5
vector E: pub=5 secretSection=0  secrets=0 -> plateList=5
vector F: pub=0 secretSection=15 secrets=3 -> plateList=12
vector G: pub=12 secretSection=3 secrets=3 -> plateList=12
```

A and B are the shape "seal just the seed" — plausibly the most common real use
of this feature. After the seed plate is cut, `unlockPlateListFlow` runs with
`len(plates) == 0`: the frame carries the "SEALED PAYLOAD" title and three nav
icons and **no body at all**. `okBtn.Clicked(ctx) && sel < len(plates)` is
`0 < 0`, so the hammer is a no-op; `start+shown < len(labels)` is `0 < 0`, so
Page is a no-op. Only Back does anything, and it is drawn with `IconDiscard`.

No safety consequence — the record was already cut and wiped — but the operator
is left on a blank screen with two dead buttons at the end of a funds-critical
operation, with nothing saying the session finished successfully.

**Fix.** When `unlockPlates(p)` is empty, show a terminal notice ("Nothing
further to engrave") and return, rather than entering the list flow.

---

## MINOR 4 — §10.2.3's normative warning fits the panel by **3 pixels**, and only because `fadeClip` is a stub; its scroll affordance does not exist on this hardware

**Where.** `gui/unlock_flow.go:163-187`, `gui/gui.go:309-353` (`Warning.Layout`),
`gui/gui.go:635-649` (`fadeClip`), `cmd/controller/platform_sh2.go:398-418`.

Measured at 480×320 with the real styles:

```
bodyClip (6,44)-(423,314)   dx=417  dy=270  scrollFadeDist=16
§10.2.3 warning body   height=257  drawn top y=60  bottom y=317   panel 320
                       visible(clip - 2*fade)=238   maxScroll=19   OVERFLOW
unlockRetryBody        height=149  maxScroll=-89
unlockHashBody         height= 95  maxScroll=-143
passphrase notice      height= 77  maxScroll=-161
```

Three facts stack:

1. `Warning.Layout` computes `maxScroll = 19 > 0` — the widget itself believes
   the last line (line height 18 px) is scrolled out of view.
2. Its **only** scroll input is `w.inp.Next(ctx, ButtonFilter(Up), ButtonFilter(Down))`.
   The SeedHammer II has no directional buttons: `processTouch` is the only event
   source and it returns `gui.PointerEvent` exclusively. So the warning body is
   **unscrollable on the real machine**.
3. Nothing is actually cut off today **only because `fadeClip` is a no-op stub**:

   ```go
   func fadeClip(b *op.Buffer, o op.Op, r image.Rectangle) op.Op {
           // op.ParamImageOp(ops, scrollMask, true, r, nil, nil)
           return o.Offset(image.Pt(0, 0))
   }
   ```

   with the real mask commented out immediately below. Because it does not clip,
   the body renders past `bodyClip.Max.Y = 314` down to y = 317 — three pixels
   inside a 320-pixel panel.

Cumulative heights show exactly what is riding on those three pixels:

```
through "Compare this with the value you recorded."       bottom y=263
through "…the encrypted part has been REMOVED. Do not continue."  bottom y=317
```

The paragraph in the 19-px overflow window is the §2.2 item 10 downgrade
instruction — the single sentence that tells the operator to stop. Restoring
`fadeClip` (the code to do so is three lines away), or adding one wrapped line
of copy, removes it silently with no way to scroll to it. Nothing in the suite
pins the fit.

This is **pre-existing from B1** — the diff does not touch
`unlockWarnUnauthenticated` — but it is live in the shipped §10.2.3 path and in
this lens.

**Fix.** Add a test that asserts the rendered body height plus its top offset
stays inside `DisplaySize().Y` for §10.2.3's copy at every legal record count,
and either give `Warning` a touch scroll or shorten the copy to fit
`bodyClip.Dy() - 2*scrollFadeDist`.

---

## NIT 5 — the in-situ KDF timing log measures KDF **plus** UI repaint, and is labelled as iterations

`gui/unlock_kdf.go:174-180`:

```go
start := time.Now()          // before the frame loop
...
log.Printf("seal: kdf %d iterations in %s", d.Total(), time.Since(start))
```

`start` precedes the loop, so the duration covers ~600 full-panel repaints as
well as the derivation. §7.1's still-owed obligation is to confirm 9,715 it/s on
RP2350**B** silicon, and a rate computed from this line understates the true
PBKDF2 rate by however much the UI costs. Understating it argues for a *lower*
iteration count on a funds path — the wrong direction.

§7.1 does say it wants "the number the operator actually experiences", and this
line delivers that. The problem is that it is labelled `kdf N iterations in T`,
which reads as a rate. **Log both**, named: accumulated derivation time
(a `time.Since` around the `d.Step` call only) and total wall clock.

---

## NIT 6 — the fallback plate label numbers encrypted cards across records that were never listed

`gui/unlock_plates.go:76-81` passes `idx: i` where `i` indexes `p.Secret`
(secrets included); `plateLabel`'s default branch renders `record %d` as `i+1`.
On vector C, if `labelEncryptedCards` ever discards a grouping failure — which it
does by design (`seal/label_encrypted.go:45-48`) — the five listed cards read
"record 2".."record 6" and a five-entry list has no "record 1". Only reachable
when card grouping fails, so cosmetic; number within the *listed* set instead.

## NIT 7 — the passphrase notice is re-shown on every retry

`unlockPassphraseFlow` calls `showNotice(...)` before its loop, and
`unlockSealedFlow` calls `unlockPassphraseFlow` afresh per attempt. A wrong
passphrase therefore costs two dismissals — the error carrying the §6.6 hash,
then the "These words are NOT a seed" notice — before the keyboard returns. It
also puts a screen between the hash and the retry. Hoisting the notice above the
retry loop would fix both.

---

## What I verified clean, with the evidence

**Q1 — the checksum gate is genuinely before the KDF, and it is instrumented.**
`unlock_kdf.go:217` `if !isMnemonicComplete(m) || !m.Valid() { return
errUnlockChecksum }` precedes `passphraseBytes` and `unlockDerive`. The seam is
real and not decorative: `newDeriver` (`:51`) is the *only* route to a
derivation on this path — `Opener.KDF` is no longer in it, because B2a derives
through `seal.NewDeriver` and opens through `UnlockWithKey`. `kdfCounter`
(`unlock_kdf_test.go:43-59`) counts calls, and `TestUnlockChecksumGateRunsNoKDF`
asserts `calls == 0` for `beef`×11+`bacon` **and** carries a positive control
(`calls == 1` at the header's own iteration count) so "the KDF was deleted"
cannot pass. `TestUnlockRejectsAPartialPassphraseWithoutAKDF` covers the
`isMnemonicComplete` half and pins the trap it exists for
(`NormalisePassphrase` repairing a one-word entry into a well-formed wrong
passphrase). Both drive `unlockAttemptOnce` directly, which is the only way to
present a mnemonic `LastWordCandidates` will not let the keyboard type.

**Q2 — step 8's strings.** `unlockRetryBody` emits
`"Wrong passphrase, or this payload has been altered.\n\n"` plus
`"Public data hash (%d records, %s):\n\n%s\n\nCompare this against the value you recorded."`
with `len(p.Public)` (§6.6's public count, **not** §6.4's cross-section total),
`unlockShape(p)`, and `seal.FormatHash(p.Hash)` — never a locally regrouped
digest. Both readings present. `p` survives a failed
`UnlockWithKey` intact (`seal/unlock_key.go` writes `p.Secret` only on success),
so the hash is re-derivable on every retry. `TestUnlockRetryKeepsTheHashOnScreen`
anchors `", SEALED):"` rather than `"SEALED"`, which is the right call —
`strings.Contains("UNSEALED", "SEALED")` is true. The `!p.HasHash` arm correctly
declines to invent the empty-record-set constant.

**Q3 — no path reaches the plate list without a successful unlock.** Two call
sites of `unlockPlateListFlow`:
- `unlock_flow.go:115`, guarded by `if !unlockSealedFlow(...) { return }`.
  `unlockSealedFlow` returns `true` only on `err == nil` from
  `unlockAttemptOnce`, which is `o.UnlockWithKey(blob, p, key)` — nil only after
  the GCM tag verified, `SplitSection` succeeded, the cross-section 24-record cap
  held and `AdmitSection(recs, SectionEncrypted)` admitted every record. Every
  other arm (`errUnlockCancelled`, `errUnlockChecksum` after the loop,
  `ErrAuthentication` after the loop, `ErrTooManyRecords`, `default`) returns
  `false` or re-prompts; `ctx.Done` falls out of `for !ctx.Done` to `return
  false`. `unlockDerive` returning `ok == false` (Back, or `ctx.Done`) maps to
  `errUnlockCancelled`.
- `unlock_flow.go:126`, on the `!Sealed()` path only, behind
  `unlockWarnUnauthenticated`'s hold-to-confirm.

**Q4 — nav slots.** `layoutNavigation` panics on `ys[int(clk.Button - Button1)]`
for anything outside `Button1..Button3`. Every screen this phase adds or touches:
`unlockDerive` 1 (`Button1`); `unlockPlateListFlow` 3 (`Button1/2/3`);
`ChoiceScreen` 2 (`Button1`, `Button3+Center`) for both the secret plate and the
engrave-variant picker; `SeedScreen.Confirm` with `NoEdit` calls
`layoutNavigation` twice with disjoint sets (`{Button1}` and `{Button3}`), each
in range; `ErrorScreen` 1 (`Button3`); `ConfirmWarningScreen` 2. The plate list's
per-entry `Clickable`s carry `Button: None` and are never passed to
`layoutNavigation` — they reach the router only through
`op.Input(...).Clip(hit)`, and `Filter.matches` requires `e.Tag == f.tag` for
pointer events, so an undrawn entry cannot fire. Layer order is correct: `nav` is
the first argument to `op.Layer` and therefore topmost, so a wide label's hit
rectangle cannot shadow a nav slot.

The `NoEdit` fold is right for the reason it states — `Filter.matches` gates a
`buttonEvent` on button identity with no bounds check, so guarding only the
layout would leave `editBtn` live. The short-circuit `!s.NoEdit && editBtn.Clicked(ctx)`
leaves `Button2`/`Center` events unconsumed, which is harmless:
`EventRouter.Reset` discards leading events no registered filter matched, so they
cannot accumulate — and on the SH2 they are never generated at all.

**Q5 — the encrypted section's cards, labelled distinguishably.** Verified by
execution on every vector (table in Minor 3). Vector G renders
`mk1 1/3 | 1/2 … mk1 3/3 | 2/2` and `md1 1/6 … md1 6/6` — the four-card,
three-cosigner shape that an HRP-only grouping would have failed. `(sealed)` is
appended exactly when both sections carry `ClassMDMK` records, so a public
`mk1 1/2` and an encrypted `mk1 1/2` never collide, and the ordinary payload gains
no noise. `unlockPlates` filters `p.Secret` to `ClassMDMK`, so no secret can
reach the list, and the labels come from `plateLabel` — one implementation of
"which card is this", per F-77 — never from anything the sealer asserted.

**Q6 — the §10.2.3 gate.** `Header.Sealed()` is `h.CtLen > 0`
(`seal/wire.go:95`), the warning is in the `else` of `if p.Header.Sealed()`, and
its copy reproduces §10.2.3 including the "REMOVED. Do not continue." paragraph.
The `Public data hash (%d records, UNSEALED)` line is hard-wired to `UNSEALED`
rather than going through `unlockShape` — correct, since the branch is
unreachable when sealed, though it is one refactor away from being able to lie.

**Q7 — `ctx.Done` handling in the KDF loop** is correct in itself:
`ctx.Frame` sets `ctx.Done` through `FrameCallback`, the loop condition re-reads
it, and the false return is mapped to `errUnlockCancelled` rather than falling
through. The defect is the interaction with `Run`'s idle branch (Important 1),
not `ctx.Done`.

**Also checked and clean:** `unlockKDFLead` multiplies before dividing and cannot
overflow `int64` inside §6.2's legal ranges; `d.Done()*100/d.Total()` peaks at
2×10⁸ and cannot overflow a 32-bit `int` at the §6.2 ceiling; `passphraseBytes`
pre-sizes to 128 so `append` never regrows and orphans half a passphrase, and the
caller owns and clears it; the derived key is a fresh copy from `d.Key()` with
`defer clear(key)` and `defer d.Wipe()` both registered before anything can
return; `ErrTooManyRecords` is distinguished from "payload unreadable" on both
the `Inspect` and the `Unlock` paths, per §6.4.

**Out of scope by the brief and not re-reported:** every wipe-lens finding
(C1, I1, M1, D1, D2, pass 3), F-83, F-86, F-87, F-88, F-89, the surviving
`clear(blob)` mutant, and the absence of §10.2.4's idle timer (that is B2b).
