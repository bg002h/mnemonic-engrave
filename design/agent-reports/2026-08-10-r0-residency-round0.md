# R0 round 0 — DESIGN_b2b_residency_zeroing.md (F-107 / F-108)

**Reviewer:** independent architect agent (adversarial), 2026-08-10.
**Artifact:** `/scratch/code/shibboleth/mnemonic-engrave/design/DESIGN_b2b_residency_zeroing.md`
**Code read (read-only):** `/scratch/code/shibboleth/seedhammer-b2b` @ `3de8aa1` (branch `b2b`).
**Verdict:** **RED — 1 Critical, 6 Important.** Not GREEN. Do not implement.

Everything numeric below was **measured**, not reasoned. A scratch copy of the
tree was made at
`$SCRATCH/shb2b` and driven with go1.26.3 from the Nix store
(`/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`); the three
measurement files are reproduced verbatim in Appendix A so every number here is
one command away. No file in the real repo was modified.

---

## 1. Is F-107's fix correct and sufficient?

### (a) SAFETY — the scrub position is SAFE. Confirmed, with one correction.

The design's safety argument holds, and holds for a slightly stronger reason
than it gives.

`Context.Frame` (gui/gui.go:84-89) calls `FrameCallback` **and only then**
`c.B.Reset()`. In production `FrameCallback` is `runWithFlow`'s closure
(gui/run_flow.go:57-84), which calls `yield(o)` — and the consumer's range body
is where `draw(content)` runs (gui/run_flow.go:124). So the draw completes
**inside** `ctx.Frame`, before `Reset` and long before `Frame` returns. At every
point in flow code that is not lexically inside a `ctx.Frame` call, `ctx.B` has
`len 0` and contains only already-drawn content. `unlockSecretSession`'s defer
is such a point. **No op built into `ctx.B` can be pending a draw when it fires.**

Three routes checked and clean:

- The **screensaver / §10.2.4 warning** loops (`run_flow.go:150-227`) run inside
  the yield, i.e. inside `ctx.Frame`, and the warning uses its own `a.warnBuf`.
  Flow code is not running there at all.
- **No screen caches an `op.Op` across frames.** The three struct fields of type
  `op.Op` in non-test code — `richText.Content` (gui.go:208),
  `Choice.W` (gui.go:1456), `ftConfirmView.Content` (freetext_flow.go:1286) —
  are all rebuilt inside the same `Draw` call that consumes them;
  `ChoiceScreen.Draw` reassigns `s.children[i].W` on every frame
  (gui.go:1536-1537). Verified by reading, not by grep count.
- **Event routing survives the scrub.** `Router.Events(d, ...)` reads
  `d.inputs`, whose `inputOp.tag` is an interface-value **copy** taken out of
  `refs` (op.go:349-353), not an alias into it. Zeroing `b.refs` cannot blank a
  touch target.

Correction to the design's wording, see **N1**: the claim "*Same position in the
frame cycle as the existing `run_flow.go:245` call*" is **not** true. `:245`
runs after the range loop has ended, on a Context that is about to be abandoned
and will never be appended to again. The session defer runs **mid-flow**, with
more frames still to come, so the zeroed prefix is immediately overwritten by
the next screen. The safety property is the same; the position is not, and the
difference is exactly what makes test 1 fragile (see **M4**).

### (b) SUFFICIENCY — NO. `unlockSecretSession` is **not** the only place a secret is rendered.

> "*What it deliberately does not do: scrub on every screen exit. **The seed is
> only ever rendered inside the secret session**, and a per-screen scrub would
> zero the buffer under ordinary navigation for no gain.*"

That sentence is false, in two independent ways.

**(i) §8's twelve-word payload passphrase — inside B2b's own feature, outside
the bracket.** `unlockPassphraseFlow` (unlock_kdf.go:109) calls
`inputWordsFlow` (gui.go:671), which renders the current word through
`widget.Labelf(&ctx.B, ...)` → `op.Glyph` → `ctx.B.args`. That flow installs its
**own** `wipeGuard` (unlock_kdf.go:135-137) precisely because the code already
rules the in-flight passphrase seed-equivalent — "*it derives the key that opens
everything, and the machine holds it beside the sealed blob in flash*"
(§10.2.4 row 4, operator ruling 2026-08-09). It **returns before**
`unlockSecretSession` is ever entered (`unlockSealedFlow`, unlock_kdf.go:402),
so the proposed defer does not cover it. On the routes where the unlock does not
succeed — Back out of word entry, N wrong passphrases then give up,
`ErrTooManyRecords`, "Payload unreadable" — `unlockSecretSession` is **never
called at all** and nothing scrubs, ever. See **I1** for the measurement.

**(ii) The entire legacy typed-seed surface, with no bracket of any kind.**
`uiFlow` → `backupWallet` → `newInputFlow` → `inputWordsFlow` → `backupWalletFlow`
→ `SeedScreen.Confirm` renders **all twelve or twenty-four words in a single
frame**. So do `seedEntryFlow` (derive_xpub.go:82), `bip85DeriveFlow`,
`recoverSLIP39Flow`, `combineSeedXORFlow`, and `passphraseFlow`'s
`PassphraseKeyboard` (gui.go:584). None is inside any bracket; `ctx.wipe` is
`nil` throughout, so §10.2.4 never arms and `run_flow.go:245` never fires
either. This is pre-existing and outside B2b's scope — but the design asserts it
does not exist, which is the part that blocks. See **M2**.

### (c) ALIASING — the `op.Drawer` aliases are handled, but not by anything the design says.

`imageOp{src: rargs[0], args: oargs, refs: rargs[1:]}` (op.go:355) stores two
slice headers **into** the Buffer plus one interface-value **copy** that lives in
the Drawer's own array. `Scrub` zeroes to capacity, so the two aliases are
covered. The `src` copy is not — and for `op.Glyph` it is the package-global
`glyphImage` handle, with the secret rune living in `args`, so nothing secret
escapes. That is why the fix works without a paired `d.Release()`; the design
never says so. See **M1**.

---

## 2. Is the F-108 re-scope right?

`bspline.Curve = iter.Seq[Knot]` at bspline/bspline.go:22 — **confirmed
verbatim**. The design's conclusion (the geometry cannot be fully zeroed, its
lifetime is what is controllable) is **substantially right**. Its two premises
are **both false**, and the brief's hypothesis was correct: it stopped one level
short.

**Premise 1 — "there is no buffer to clear / `clearSpline(plate)` cannot be
written."** False. `engrave.PlanEngraving` (engrave/engrave.go:1016-1021) is:

```go
knotBuf := make([]bspline.Knot, 0, maxSplineKnots)   // maxSplineKnots = 100
return planEngraving(knotBuf, conf, e)
```

The returned closure writes seed-derived control points into that array and
`planEngraving` **already takes the buffer as a parameter** — the
caller-owns-the-buffer seam exists. Measured after a full iteration ("the cut"):

```
knots yielded during the 'cut': 844
knotBuf cap=100, NON-ZERO entries left after the cut: 9
first residual control point: {Ctrl:{X:338476 Y:22756} T:0 Engrave:true}
after clear(buf[:cap(buf)]): 0 non-zero entries
```

`clear` compiles, runs, and works. Two more ownable knot buffers the design
never names: `engrave.SafePointer.history` (engrave.go:1637), trimmed with
`copy` + reslice (:1675-1676) so the tail beyond the new length keeps stale
knots forever, and `splineResumer.catchup` (gui/engraver.go:222). Both hang off
the `engraveJob` the `wipeGuard` holds during the cut.

**Premise 2 — "the geometry was computed into the closure beforehand, which is
why that early clear is sound."** False. Measured:

```
upstream Engraving invocations after PlanEngraving returned: 0
after ranging the spline once:  upstream invocations=1, knots=338
after ranging it a second time: upstream invocations=2
```

The plan is **lazy and re-entrant**: it re-runs the whole upstream `Engraving`
on every iteration. `toPlate` iterates it once at build time via
`bspline.Measure` (gui.go:2989) and the engrave goroutine iterates it again
during the cut (engraver.go:170). See **I3** for why the wrong reason matters,
and **M3** for the lifetime bound this falsifies.

**What is genuinely unownable** (and supports option 1, though the design never
names it): `appendLine` allocates a **fresh** `make([]bspline.Knot, len(sc))`
per line segment (engrave.go:1146) — thousands per plate, unreachable garbage
the instant it is consumed. That, not the closure-ness of `iter.Seq`, is the
real argument for option 1.

**Net:** the recommendation ("1 now, 2 later") may well still be right, but it
is currently reached from two false facts and a mis-costing, and option 1's
deliverable is a **spec amendment asserting an impossibility that is measurably
not impossible**.

---

## 3. Tests

**Test 1 (gui, `ctx.B.Residue()` after a normal exit) — would fail before and
pass after, and it has a measured false-PASS.**

The template already exists: `gui/run_flow_scrub_test.go`'s
`TestWipeScrubsTheAbandonedFrameBuffer` does exactly this shape with
`boundedFlow` + `runSession` + `first.B.Residue()`, and its mutation row is the
same one proposed. Copy it, drop `wipeNowHook`, insert a real
`unlockSecretSession`.

Three false-PASS paths, in descending severity:

1. **The one that actually bites (C1).** `Residue()` scans only the **current**
   backing array. `ctx.B.args` reallocates while the seed frame is being built,
   and the pre-growth array keeps the words. Measured on a 24-word seed with the
   realistic pre-sequence: `Residue()` returns **(0 args, 0 refs)** while the
   orphaned array reads `1: ABAN$D2O?NO5<2: ABILI(T-Y85Q3: ABLE#5f… 13: ACCO&U6NDTR`.
   The test reports PASS on a buffer holding thirteen words in order with their
   positions.
2. **Assertion point.** With the fix, `Residue()` is 0 only at the instant the
   session defer fires; the flow immediately draws the plate list and re-dirties
   the array. A test that asserts at the end of the run **fails with the fix**.
   The design does not say where the assertion is taken (**M4**).
3. **Context identity.** `runWithFlow` allocates a fresh `Context` per session
   iteration (run_flow.go:39). A test holding a stale `ctx` asserts on a dead
   buffer and passes unconditionally — the direct analogue of release_test.go's
   "canary in a slot the next frame overwrites".

Positive control required, as in `release_test.go`: assert residue is **non-zero
before** the scrub point, or the whole assertion is vacuous if the harness never
rendered a seed.

**Test 2 (mutation row: `Scrub` → no-op).** Sound and killable by test 1. No
objection.

**Test 3 (finalizer/lifetime canary on the plate) — CANNOT FAIL. Do not write
it.** See **I5**. F-108 ships no code change, so there is no before-state; the
property asserted (a local's reference drops at return) is true today and after
the proposed spec amendment; `Plate` is a struct value whose `Spline` is a
**func value**, which `runtime.SetFinalizer` cannot take; and a host-Go
finalizer proves nothing about TinyGo `-gc precise`. This is exactly the class
`gui/op/release_test.go` was rewritten to eliminate.

**Test 4 (the trap list) — correct and well-stated.** No objection.

---

## 4. What the design is silent about that a funds path cannot be silent about

1. **No threat model.** The note leans on "bounded by heap reuse" and "the
   reference drops promptly" as if they were mitigations, without ever saying
   *who reads the residue and when*. RAM-resident-until-overwritten is a
   near-total non-issue against an attacker who only ever gets a powered-down
   device, and a total failure against one who gets it powered. The note's own
   §"What R0 should attack" #2 asks whether option 1 is acceptable for a funds
   path — **that question is not answerable as written**. (**I6**)
2. **Buffer growth orphaning.** Not mentioned once, and it is the finding that
   defeats the fix (**C1**). The mechanism is already documented **in this
   repo**, twice: `run_flow.go:22-31` measured `warnBuf`'s "*~7 doublings
   memcpy'd the PARKED frame … into an array nothing ever zeroes*", and
   `passphraseBytes` fixes its capacity for exactly this reason (unlock_kdf.go:
   "*a regrow would leave a stale copy of the first half of the passphrase in an
   orphaned array that nothing can reach to wipe*"). The design applies neither
   precedent to `ctx.B`.
3. **Whether `run_flow.go:245` is load-bearing.** The design calls this "*a
   question about clarity, not correctness*". It is correctness (**I4**).
4. **Where the residency inventory ends.** `unlock_session.go:226-240`'s
   ZEROED/LIVE table has been wrong twice by its own admission. F-108 touches
   the `LIVE plate.Spline` row and adds no rows, while this review found three
   knot buffers and a lazily-re-read upstream closure that belong in it.

---

# FINDINGS

## CRITICAL (1)

### C1 — the fix does not deliver the property it claims: `Buffer.Scrub` cannot reach the pre-growth arrays the seed frame orphans

**Anchor:** design §"F-107 — The fix" and §"Tests that can fail" #1;
`gui/op/buffer_len.go:23-28` (`Scrub`), `:36-48` (`Residue`); `gui/op/op.go`
(every `b.args = append(...)` site).

`op.Buffer` has no pre-sizing: `NewContext` (gui.go:86-92) leaves `B` zero-valued,
so `args` grows from `nil` by doubling. `Scrub` zeroes `b.args[:cap(b.args)]` —
**the current array only**. Every reallocation orphans an array that still holds
every rune appended before it, and no code in the tree can ever reach it again.

**Failure scenario (measured, realistic sequence).** Payload carries a 24-word
mnemonic. Operator types the 12-word passphrase (keyboard frames drive
`cap(args)` to 1216), unlock succeeds, `SeedScreen` renders 24 words needing
2387 args → `append` reallocates to cap 3392 and orphans the 1216-word array.
Operator reads the plate, presses Back. `unlockSecretSession`'s new defer runs
`ctx.B.Scrub()`:

```
warm=word keyboard  n=24 | warm frame len=0 cap=1216 | seed frame len=2387 cap=3392 | reallocated=true
warm=word keyboard  n=24 | Residue() after Scrub = (0 args, 0 refs)
warm=word keyboard  n=24 | orphan text: "1: ABAN$D2O?NO5<2: ABILI(T-Y85Q3: ABLE#5f4: ABOU(T65{5: ABOV(E556: ABSE$N0T>57: ABSO$R4B?58: ABST$R.A9CETR59: ABSU$R2D=510: ABUS&E2511: ACCE&S2S>512: ACCI&D+E8NDTR513: ACCO&U6NDTR"
```

**Thirteen of twenty-four words, verbatim, in order, with their indices**, in an
array §10.2.2 declares wiped and that `Residue()` scores 0. The proposed test 1
reports PASS. `run_flow.go:245`'s existing wipe-path `Scrub` has the identical
hole, so this is not introduced by the fix — but the fix's whole claim is that
it makes the normal exit equal to the wipe exit, and it delivers an equality of
two incomplete wipes while the doc reads as a guarantee.

The 12-word case survives **by seven args** (`len=1209` vs `cap=1216`) and only
because the word keyboard happens to have warmed the buffer past it. With a
choice screen as the last pre-seed frame it reallocates and leaks words 1–6:

```
warm=choice screen  n=12 | warm cap=512 | seed frame len=1209 cap=1344 | reallocated=true
warm=choice screen  n=12 | orphan text: "1: ABAN$D2O?NO5<2: ABILI(T-Y85Q3: ABLE#5f4: ABOU(T65{5: ABOV(E556: ABSE$N0T"
```

A seven-arg margin is not a guarantee; any font metric, label string, or nav
change flips it.

**Smallest correct fix.** Pre-size `ctx.B` once, at `NewContext`, to the
worst-case frame, so no reallocation can occur inside a session — the same
device `passphraseBytes` already uses for the same reason, and the same reason
`warnBuf` is a separate buffer. That needs one exported sizing entry point on
`op.Buffer` (it has no way to pre-size today) and a measured constant: the
largest frame observed is 2387 args / 24-word `SeedScreen` on the 480×320 panel,
so a bound with headroom plus a test that fails if any frame exceeds it. If the
number cannot be bounded, the alternative is a grow-and-zero inside `op` (copy,
then `clear` the old array before dropping it) — but pre-sizing is strictly
cheaper on a device where the growth also costs ~7 memcpys of the seed frame.

`Residue()` must also be recognised as unable to see this class, and the design
must stop citing it as the property's witness.

---

## IMPORTANT (6)

### I1 — §8's twelve-word passphrase is rendered outside the bracket, and on the give-up routes nothing scrubs at all

**Anchor:** design §"F-107 — The fix", the sentence "*The seed is only ever
rendered inside the secret session*"; `gui/unlock_kdf.go:109-171`
(`unlockPassphraseFlow`), `gui/gui.go:671-810` (`inputWordsFlow`),
`gui/unlock_kdf.go:402`.

The passphrase bracket is a **sibling** of the secret-session bracket, not a
child: `unlockPassphraseFlow` installs and removes its own `wipeGuard` and
returns before `unlockSecretSession` is entered. The proposed defer therefore
does not cover it, and on every route where the unlock does not reach the
secret session — Back out of word entry, repeated `seal.ErrAuthentication`
then give up, `ErrTooManyRecords`, "Payload unreadable" — `unlockSecretSession`
is never called, so **no `Scrub` runs on any exit**. Meanwhile a walk-away
during entry *does* wipe, via row 4's armed timer and `run_flow.go:245`. That is
precisely F-107's own argument — protected on the rare path, exposed on the
common one — reproduced verbatim in the bracket the fix does not touch, while
the design claims to have closed it.

**Measured magnitude, and it is smaller than the seed screen's** — stated so the
fix is scoped to what is real:

```
after 12 passphrase words, NO scrub runs on this path:
  args cap=1216  Residue()=(901 args, 0 refs)
  recovered text: "...12: AB%I1L6IATFYQWord 12 of 12"
```

`inputWordsFlow` renders one word at a time at a near-constant offset, so later
frames overwrite earlier ones and it is the **last** word plus its index that
survives, not all twelve. Eleven bits off a twelve-word passphrase is not a
break. It is graded Important rather than Critical for that reason — but the
design's *premise* is false, the material is seed-equivalent by the project's
own ruling, and the residue is nonzero on a route with no wipe at all.

**Smallest correct fix.** Add `ctx.B.Scrub()` to `unlockPassphraseFlow`'s
existing defer (unlock_kdf.go:137), symmetric with the one proposed for
`unlockSecretSession`, and rewrite the false sentence.

### I2 — F-108's central premise is false: `PlanEngraving`'s `knotBuf` is a materialised, ownable buffer, and `clear` on it compiles and works

**Anchor:** design §"RESOLVED BEFORE REVIEW: the spline cannot be zeroed at
all", specifically "*There is no buffer to `clear`. `clearSpline(plate)` cannot
be written*", and option 2's costing "*a real design change to the engrave
pipeline with a memory cost on a device with 283 K free*".

`Curve = iter.Seq[Knot]` is true and irrelevant to the conclusion drawn from it.
A closure over an already-materialised slice is still a clearable slice, and
that is exactly what `engrave.PlanEngraving` builds (engrave.go:1016-1021).
Measured above: 9 non-zero knots left in `knotBuf` after a full cut;
`clear(buf[:cap(buf)])` drives it to 0. `planEngraving(knotBuf, conf, e)`
already exists as the caller-supplies-the-buffer seam, with a doc comment saying
so. Two further ownable buffers go unnamed: `SafePointer.history`
(engrave.go:1637, trimmed by `copy`+reslice at :1675-1676, so the tail is never
zeroed) and `splineResumer.catchup` (gui/engraver.go:222).

**What actually goes wrong.** Option 1's deliverable is a **spec amendment
(F-85) stating an impossibility that is measurably not impossible**, on a funds
path, in a document future work will treat as settled; and option 2 is dismissed
to a later cycle on a memory cost that does not exist — the 100-knot allocation
is already made on every `toPlate`, ~1.6 KB on the 32-bit target
(`bezier.Point{X,Y int}` = 8 B, `Knot` = 16 B padded).

Graded Important, not Critical: the measured residue is 9 knots of the final
stroke of the last glyph, which is not seed-recoverable. The defect is a
decision made on false facts, not a live leak. Inflating it would be padding.

**Smallest correct fix.** Replace the premise with the measured facts: name
`knotBuf`, `SafePointer.history` and `splineResumer.catchup` as ownable and
zeroable; name `appendLine`'s per-segment `make([]bspline.Knot, len(sc))`
(engrave.go:1146) as the genuinely unownable part and therefore as the *real*
argument for option 1; re-cost option 2 against a buffer that already exists.

### I3 — "the geometry was computed into the closure beforehand" is false; the early `clear(rec)` is sound for a different reason, and the stated reason licenses a wrong-plate defect

**Anchor:** design §"RESOLVED BEFORE REVIEW", third bullet: "*the spline does not
read `rec` during the cut — the geometry was computed into the closure
beforehand, which is why that early clear is sound*". Code:
`gui/unlock_session.go:195-204` and `:295-311`, `engrave/engrave.go:1025-1084`,
`gui/gui.go:2989`.

Measured: `PlanEngraving` performs **zero** upstream work before it is ranged,
one full upstream traversal per range, and a second traversal on a second range.
`toPlate` ranges it once (`bspline.Measure`, gui.go:2989) and the engrave
goroutine ranges it again (engraver.go:170) — the plan is read live, throughout
the ~21-minute cut.

`clear(rec)` / `clear(m)` before `Engrave` is nevertheless **sound**, and the
real reason should be written down: `engraveSeed` (gui.go:539-559) copies out to
`words []string` via `bip39.LabelFor` and to a fresh `qr.Code` *before* the plan
closure is built, and `unlockEngraveCodex32` copies to a Go string via
`s.String()` — so nothing in the closure chain aliases `rec` or `m`.

**Failure scenario the false reason enables.** A future author adds a derivation
inside the plate pipeline that closes over `rec` — say a lazily-computed
fingerprint or a re-parse for a second plate side. The design says the geometry
is precomputed, so `clear(rec)` before `Engrave` looks safe. It is not: the
closure reads zeroed bytes during the cut and the machine engraves a plate that
is internally self-consistent and **does not restore the wallet**, while the
operator watches a "seed words 1/1" job run to completion. That is the §6.4
worst-available-outcome class, produced by trusting a sentence in this document.

**Smallest correct fix.** Replace the sentence with: *the plan closure is lazy
and is re-read throughout the cut; `clear(rec)`/`clear(m)` is safe only because
`engraveSeed`/`EngraveSeedString` materialise independent copies before the
closure is built — any new derivation that captures `rec` or `m` breaks it.*

### I4 — `run_flow.go:245` is NOT subsumed by the new defer; the design frames its removal as a matter of clarity

**Anchor:** design §"What R0 should attack" #4: "*Does F-107's fix subsume
`run_flow.go:245`? … `Scrub` is idempotent, so this is a question about clarity,
not correctness.*"

It is correctness. A §10.2.4 wipe can fire while the flow is inside
`unlockPassphraseFlow`'s row-4 bracket — `armed()` returns true there
(wipe_guard.go:44-57, `g.job == nil`), and `unlockSecretSession` has not been
entered, so its defer does not exist. `run_flow.go:245` is the **only** scrub on
that path. It is likewise the only scrub for any wipe that fires with
`ctx.wipe == nil`, which cannot happen today but is not structurally prevented.

A future fold that reads "subsumed" and deletes `:245` reintroduces a Critical
on the row-4 path with a green suite (no test covers it — the design says as
much for the whole property).

**Smallest correct fix.** Answer the question in the design: `:245` is
load-bearing for the row-4 bracket and for any wipe outside a secret session;
both calls stay; add a one-line comment at `:245` naming what it uniquely
covers, so the next reader cannot conclude otherwise.

### I5 — proposed test 3 cannot fail, and a test with no mutation row is a false PASS by construction

**Anchor:** design §"Tests that can fail" #3.

Four independent reasons the finalizer/lifetime test is unkillable:

1. **No before-state.** The design itself says "*F-108 no longer has code to
   gate — its remedy is a spec amendment, not a patch.*" A test cannot fail
   before a change that does not exist.
2. **The property is already structurally true.** `plate` is a local;
   `unlockEngraveMnemonic`/`unlockEngraveCodex32` return immediately after
   `scr.Engrave`. The assertion holds identically before and after.
3. **`runtime.SetFinalizer` cannot take the object.** `Plate` is a struct
   *value*; `Plate.Spline` is a *func value*. Neither is a pointer to an object
   the runtime will finalize. There is no injection point for a canary into the
   closure's captured environment from `gui`.
4. **Host ≠ target.** A host-Go finalizer says nothing about TinyGo
   `-gc precise`, which is the only GC that runs on the machine.

This is the same class `gui/op/release_test.go` was rewritten to remove — and
that file's own comments record three separate traps found the hard way.
Shipping a fourth in the same cycle, in a design that cites that work approvingly
(§"Tests that can fail" #4), is the finding.

**Smallest correct fix.** Delete test 3. If a lifetime pin is wanted, pin the
thing that *can* regress: a compile-or-grep assertion that no field of a
long-lived struct holds a `Plate` or a `bspline.Curve` — that has a real
mutation row (add such a field, watch it fail).

### I6 — no threat model, so the note's own central question is unanswerable

**Anchor:** design §"Three honest options", "*bounded by heap reuse*"; §"What R0
should attack" #2, "*whether documenting that is good enough for a funds path …
This is the judgement call the note most needs reviewed.*"

I cannot answer it, and neither can any reviewer, because the document never
states what the zeroing defends against. The three candidates give three
different verdicts:

- **Next operator / a later UI path that renders stale memory** — reference-drop
  plus heap reuse is adequate; option 1 is fine.
- **A firmware defect or debug path that dumps RAM while powered** — reuse is
  not a bound at all (garbage can sit for the machine's whole uptime); option 1
  is inadequate and so is F-107's fix without C1.
- **Physical attacker, powered device / cold boot** — neither option matters
  without a reset-time SRAM clear, which is out of this design's scope entirely
  and must be named as such rather than left implied.

**Smallest correct fix.** One paragraph naming the adversary, the window, and
what is explicitly *out* of scope; then re-derive the option-1-vs-2 verdict from
it instead of from cost.

---

## MINOR (4)

### M1 — the new `Scrub` is added without the `d.Release()` it is paired with at the only precedent, and the design does not say why

`run_flow.go:245-262` pairs `ctx.B.Scrub()` with `d.Release()` and explains at
length why the Drawer's stale `frameOp.src` copies are unreachable by `Scrub`.
The design adds one half of that pair with no mention of the other. It is
**correct** to omit it — on the normal exit at least one more frame is always
drawn, and `Drawer.Draw` clears `maskStack` to capacity on entry (op.go:262), so
the copies drop one frame later; and for `op.Glyph` the `src` copy is the global
`glyphImage` handle, carrying nothing secret. But a residency design that adds
one half of an established pair has to say it checked.
**Owning phase: B2b, with the fold.**

### M2 — the legacy typed-seed surface has no bracket at all, and the design asserts it does not exist

`backupWalletFlow`/`SeedScreen.Confirm`, `seedEntryFlow`, `bip85DeriveFlow`,
`recoverSLIP39Flow`, `combineSeedXORFlow`, `passphraseFlow` all render full
seeds or passphrases into `ctx.B` with `ctx.wipe == nil` throughout, so neither
the new defer nor `run_flow.go:245` nor §10.2.4 ever touches them.
`SeedScreen.Confirm` puts **all** words in one frame, so the residue there is
the full seed, not a tail. This is pre-existing and correctly outside B2b, but
the design must say so explicitly instead of denying it, and file it.
**Owning phase: file as a follow-up with an owning phase; not B2b.**

### M3 — `bspline.Measure` fills the knot buffer at BUILD time, so "for the duration of the cut" is not the lifetime bound

`toPlate` (gui.go:2988-2989) ranges the plan closure once for the bounds check,
before any cut. So `knotBuf` holds seed-derived geometry on paths where **no cut
ever happens** — most visibly `ErrTooLarge`, where `Plate{}` is returned,
`showError` is shown, and the filled array becomes unreachable garbage
immediately. The design's whole model for F-108 ("*LIVE plate.Spline, for the
duration of the cut*", "*the exemption genuinely is time-boxed to the cut*")
does not describe that path. **Owning phase: B2b, with I2's fold.**

### M4 — test 1's assertion point is unspecified, and the obvious one fails with the fix

Because the session defer runs mid-flow (not on an abandoned Context like
`:245`), the plate list and start screen re-dirty `args` immediately. A test
that runs the flow to completion and then reads `Residue()` gets a non-zero
answer **with the fix applied**. The test must terminate the flow at the session
return — which is what `boundedFlow` is for, and which makes the test a test of
the session in isolation rather than of "exit normally to the start screen" as
worded. Say which. **Owning phase: B2b, with the test.**

---

## NIT (3)

- **N1.** "*Same position in the frame cycle as the existing `run_flow.go:245`
  call*" — not the same position. `:245` runs post-unwind on a Context that will
  never be appended to again; the session defer runs mid-flow. The safety
  conclusion is unaffected; the equivalence claim is wrong and is what makes M4
  invisible.
- **N2.** "*`clear(rec)` runs before `Engrave` (`unlock_session.go:195-203`)*" —
  195-203 is the comment; the `clear(rec)` is line 204.
- **N3.** "*There is no `clear` of a plate or spline anywhere in `gui/` — the
  three matches are two comments and a constructor*" — the grep that produced
  "three" is not shown, and `grep -rn 'spline\|Spline' gui/*.go | grep -v _test`
  returns 30+ lines. Every other measured claim in this document is quoted with
  its command; this one is not, and it is the one carrying F-108's negative
  result. Show the command.

**Verified correct** (checked, no finding): `bspline/bspline.go:22`;
`gui/gui.go:88` for `c.B.Reset()`; `gui/gui.go:1612` for `for !ctx.Done`;
`cmd/controller/main.go:34`; `gui/run_flow.go:245` as the sole `.Scrub()` call
site; `gui/unlock_session.go:87-90` guard bracket quoted verbatim;
`gui/op/op.go`'s `Buffer.Reset` body quoted verbatim; `gui/engraver.go:64` and
`:170`.

---

## What must be true to reach GREEN

1. C1 answered with a pre-sized (or grow-zeroing) `op.Buffer`, plus a test that
   can see orphaned arrays — `Residue()` cannot, and the design must stop citing
   it as the witness.
2. I1: `unlockPassphraseFlow`'s defer scrubs too; the false universal claim
   rewritten.
3. I2/I3/M3: F-108's premises replaced with the measured facts; options
   re-costed; the early-clear invariant restated correctly.
4. I4: `:245` declared load-bearing, not subsumed.
5. I5: test 3 deleted or replaced with something that has a mutation row.
6. I6: a threat model, then the option-1-vs-2 verdict re-derived from it.

---

# Appendix A — measurement sources (reproduce verbatim)

Applied to a **copy** of the tree; the real repo was not modified.

```
cp -a /scratch/code/shibboleth/seedhammer-b2b $SCRATCH/shb2b
export PATH=/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin:$PATH
cd $SCRATCH/shb2b
GOFLAGS=-mod=mod go test ./gui/     -run 'TestMeasureSeedFrameOrphansAPreGrowthArray|TestMeasurePassphraseResidueOutsideTheBracket' -v
GOFLAGS=-mod=mod go test ./engrave/ -run 'TestMeasureKnotBufIsAnOwnableSeedBuffer|TestMeasurePlanIsLazy' -v
```

### `gui/op/backing_probe.go` (instrumentation)

```go
package op

// TEST INSTRUMENTATION -- scratch review copy only, never for commit.

// BackingArgs returns the whole backing array of args, including past len.
func (b *Buffer) BackingArgs() []uint32 { return b.args[:cap(b.args)] }

// ArgsLenCap reports len and cap of args.
func (b *Buffer) ArgsLenCap() (int, int) { return len(b.args), cap(b.args) }
```

### `gui/residency_measure_test.go` (C1)

```go
package gui

import (
	"strings"
	"testing"
	"unsafe"

	"seedhammer.com/bip39"
)

// argText renders the backing array the way run_flow.go's comment describes
// recovering it: every arg that is a printable ASCII rune, in array order.
func argText(arr []uint32) string {
	var b strings.Builder
	for _, v := range arr {
		if v >= 0x20 && v < 0x7f {
			b.WriteByte(byte(v))
		}
	}
	return b.String()
}

func wordsIn(s string, m bip39.Mnemonic) []int {
	var out []int
	for i, w := range m {
		if strings.Contains(s, strings.ToLower(bip39.LabelFor(w))) ||
			strings.Contains(s, strings.ToUpper(bip39.LabelFor(w))) {
			out = append(out, i+1)
		}
	}
	return out
}

func TestMeasureSeedFrameOrphansAPreGrowthArray(t *testing.T) {
	for _, warm := range []string{"choice screen", "word keyboard"} {
		for _, nwords := range []int{12, 24} {
			m := validMnemonic(nwords)
			pf := newPlatform()
			pf.display = sh2DisplaySize
			ctx := NewContext(pf)

			switch warm {
			case "choice screen":
				cs := &ChoiceScreen{Title: "secret", Lead: "SECRET seed material",
					Choices: []string{"Cut this plate", "Skip"}}
				for range 4 {
					cs.Draw(ctx, &descriptorTheme, sh2DisplaySize)
					ctx.B.Reset()
				}
			case "word keyboard":
				kbd := NewKeyboard(ctx, wordKeys)
				for range 12 {
					kbd.Layout(ctx, &descriptorTheme)
					ctx.B.Reset()
				}
			}
			warmLen, warmCap := ctx.B.ArgsLenCap()

			before := ctx.B.BackingArgs()
			beforePtr := unsafe.SliceData(before)

			ss := &SeedScreen{}
			ss.Draw(ctx, &descriptorTheme, sh2DisplaySize, m)
			seedLen, seedCap := ctx.B.ArgsLenCap()
			afterPtr := unsafe.SliceData(ctx.B.BackingArgs())

			ctx.B.Scrub() // the session tail's fix
			resArgs, resRefs := ctx.B.Residue()

			orphaned := beforePtr != afterPtr
			txt := argText(before)
			found := wordsIn(txt, m)

			t.Logf("warm=%-14s n=%2d | warm frame len=%4d cap=%4d | seed frame len=%4d cap=%4d | reallocated=%v",
				warm, nwords, warmLen, warmCap, seedLen, seedCap, orphaned)
			t.Logf("warm=%-14s n=%2d | Residue() after Scrub = (%d args, %d refs)", warm, nwords, resArgs, resRefs)
			t.Logf("warm=%-14s n=%2d | words readable VERBATIM from the ORPHANED array: %d of %d %v",
				warm, nwords, len(found), nwords, found)
			t.Logf("warm=%-14s n=%2d | orphan text: %q", warm, nwords, txt)
		}
	}
}
```

Output:

```
warm=choice screen  n=12 | warm frame len=   0 cap= 512 | seed frame len=1209 cap=1344 | reallocated=true
warm=choice screen  n=12 | Residue() after Scrub = (0 args, 0 refs)
warm=choice screen  n=12 | orphan text: "1: ABAN$D2O?NO5<2: ABILI(T-Y85Q3: ABLE#5f4: ABOU(T65{5: ABOV(E556: ABSE$N0T"
warm=choice screen  n=24 | warm frame len=   0 cap= 512 | seed frame len=2387 cap=3072 | reallocated=true
warm=choice screen  n=24 | Residue() after Scrub = (0 args, 0 refs)
warm=word keyboard  n=12 | warm frame len=   0 cap=1216 | seed frame len=1209 cap=1216 | reallocated=false
warm=word keyboard  n=24 | warm frame len=   0 cap=1216 | seed frame len=2387 cap=3392 | reallocated=true
warm=word keyboard  n=24 | Residue() after Scrub = (0 args, 0 refs)
warm=word keyboard  n=24 | orphan text: "1: ABAN$D2O?NO5<2: ABILI(T-Y85Q3: ABLE#5f4: ABOU(T65{5: ABOV(E556: ABSE$N0T>57: ABSO$R4B?58: ABST$R.A9CETR59: ABSU$R2D=510: ABUS&E2511: ACCE&S2S>512: ACCI&D+E8NDTR513: ACCO&U6NDTR"
```

### `gui/residency_pp_measure_test.go` (I1)

```go
package gui

import (
	"testing"

	"seedhammer.com/bip39"
	"seedhammer.com/gui/widget"
)

func TestMeasurePassphraseResidueOutsideTheBracket(t *testing.T) {
	m := validMnemonic(12)
	pf := newPlatform()
	pf.display = sh2DisplaySize
	ctx := NewContext(pf)

	kbd := NewKeyboard(ctx, wordKeys)
	for i, w := range m {
		lbl := bip39.LabelFor(w)
		kbd.Layout(ctx, &descriptorTheme)
		widget.Labelf(&ctx.B, ctx.Styles.word, descriptorTheme.Background, "%2d: %s", i+1, lbl)
		layoutTitlef(ctx, sh2DisplaySize.X, descriptorTheme.Text, "Word %d of %d", i+1, len(m))
		ctx.B.Reset() // what Context.Frame does after every frame
	}

	args, refs := ctx.B.Residue()
	txt := argText(ctx.B.BackingArgs())
	found := wordsIn(txt, m)
	_, c := ctx.B.ArgsLenCap()
	t.Logf("after 12 passphrase words, NO scrub runs on this path:")
	t.Logf("  args cap=%d  Residue()=(%d args, %d refs)", c, args, refs)
	t.Logf("  words readable in the CURRENT backing array: %d of 12 %v", len(found), found)
	t.Logf("  recovered text: %q", txt)
}
```

Output:

```
after 12 passphrase words, NO scrub runs on this path:
  args cap=1216  Residue()=(901 args, 0 refs)
  recovered text: "$Q ,$W ,%$E ,G$R ,i$T ,$Y ,$U ,$I ,$O ,$P ,$A ,2$S ,62$D ,X2$F ,z2$G ,2$H ,2$J ,2$K ,2$L ,2$Z ,6`$X ,X`$C ,z`$V ,`$B ,`$N ,`$M ,`$ ,`12: AB%I1L6IATFYQWord 12 of 12"
```

### `engrave/knotbuf_measure_test.go` (I2, I3)

```go
package engrave

import (
	"testing"

	"seedhammer.com/bspline"
	"seedhammer.com/font/constant"
)

func TestMeasureKnotBufIsAnOwnableSeedBuffer(t *testing.T) {
	s := String(constant.Font, 40*mm/10, "ABANDON ABILITY ABLE")
	e := Engraving(s.Engrave)

	buf := make([]bspline.Knot, 0, 100)
	spline := planEngraving(buf, conf, e)

	n := 0
	for range spline {
		n++
	}

	full := buf[:cap(buf)]
	nonzero := 0
	for _, k := range full {
		if k.Ctrl.X != 0 || k.Ctrl.Y != 0 || k.T != 0 || k.Engrave {
			nonzero++
		}
	}
	t.Logf("knots yielded during the 'cut': %d", n)
	t.Logf("knotBuf cap=%d, NON-ZERO entries left after the cut: %d", cap(buf), nonzero)
	t.Logf("first 6 residual control points: %+v", full[:6])

	clear(full)
	nonzero = 0
	for _, k := range full {
		if k.Ctrl.X != 0 || k.Ctrl.Y != 0 || k.T != 0 || k.Engrave {
			nonzero++
		}
	}
	t.Logf("after clear(buf[:cap(buf)]): %d non-zero entries", nonzero)
}

func TestMeasurePlanIsLazy(t *testing.T) {
	reads := 0
	var planned int
	e := Engraving(func(yield func(Command) bool) {
		reads++
		s := String(constant.Font, 40*mm/10, "ABANDON")
		s.Engrave(yield)
	})
	spline := PlanEngraving(conf, e)
	t.Logf("upstream Engraving invocations after PlanEngraving returned: %d", reads)
	for range spline {
		planned++
	}
	t.Logf("after ranging the spline once: upstream invocations=%d, knots=%d", reads, planned)
	for range spline {
	}
	t.Logf("after ranging it a second time: upstream invocations=%d", reads)
}
```

Output:

```
knots yielded during the 'cut': 844
knotBuf cap=100, NON-ZERO entries left after the cut: 9
first 6 residual control points: [{Ctrl:{X:338476 Y:22756} T:0 Engrave:true} {Ctrl:{X:338476 Y:22756} T:0 Engrave:true} {Ctrl:{X:327098 Y:22756} T:10650 Engrave:true} {Ctrl:{X:330891 Y:22756} T:10650 Engrave:true} {Ctrl:{X:334683 Y:22756} T:21367 Engrave:true} {Ctrl:{X:338476 Y:22756} T:10650 Engrave:true}]
after clear(buf[:cap(buf)]): 0 non-zero entries

upstream Engraving invocations after PlanEngraving returned: 0
after ranging the spline once: upstream invocations=1, knots=338
after ranging it a second time: upstream invocations=2
```
