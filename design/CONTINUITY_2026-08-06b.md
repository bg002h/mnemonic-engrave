# Continuity — 2026-08-06b

Supersedes `CONTINUITY_2026-08-06.md`, which is now badly stale: it opens with
"lengthen the z crossbar" as the first task and treats the engraving wiggle as
unexplained. Both are done, and the wiggle turned out not to be a firmware
problem at all.

## 1. THE HEADLINE — the engraving artefact was Y-AXIS PLAY, a loose screw

**The operator found mechanical play in the Y axis, tightened it, and the
artefact vanished.** Confirmed by cutting a forward and a reversed tilde: both
perfect.

Every experiment that came back null now makes sense — play is a **distance**,
not a rate, and it is in the machine rather than the workpiece or the code:

| tested | result |
| --- | --- |
| acceleration and jerk halved | no change |
| feed 4 mm/s, then the slowest | no change |
| hard steel → soft steel | no change |
| fork vs stock v1.4.3 vs **genuine official** v1.4.3 | no change |
| **`\|` and `/`** | **perfect — a straight stroke never reverses an axis, so it never takes up slack** |

**Do not re-open this.** `design/RECON_cusp_dot_pileup.md` carries a resolution
banner at the top; F-59 is withdrawn; F-62 is demoted to a trap-for-future-editors
(the constant-time panic it describes is real, but there is no longer a reason to
curve the face).

**The process lesson, which is the transferable part.** Four independent
*software* variables came back null and after each one another software
mechanism was proposed — cusps, dot pile-up, two-axis coordination, needle
footprint. **Two nulls on independent software parameters should have moved the
prior to hardware and kept it there.** The measurements were all sound; every
causal story built on them was wrong.

### Consequence still open: the 4 mm/s feed is unjustified

`343fb05` halved the engraving feed from 8 to 4 mm/s because hard@4 looked better
than hard@8 — **a comparison made on a machine with a loose screw.** Re-test at
stock 8 mm/s before treating it as settled. It would halve plate times if it
holds.

## 2. The engraving-settings feature — built, flashed, PARTLY tested

Fourteen commits, `f97c725..72e2584`, 46 packages green, **all on `main`,
nothing pushed.**

```
QR -> Font -> Size -> Text -> Title -> Footer -> Confirm      7 steps (was 8)
                        └─ gear key ─> Engraving ─> Speed  8/6/4/2/1 mm/s
                                                 └─ Passes 1/2/3/4/5/8
```

- **Passes** engraves each glyph N times IN PLACE (body, title and footer), for depth.
- **Speed** moved off the flow and behind the gear; the old standalone step is gone.
- **Clear buttons** on Text (asks first), Title and Footer (do not ask — a line is
  at most one line, so little can be lost; the Text field is uncapped and is the
  only copy).
- **The confirm screen** names either setting when it is not at its default.
- Both settings are **locked until a proof pattern is loaded**.

### Verified on hardware (2026-08-06)

Boots, hash matches, gear reachable, Engraving menu shows both parameters, both
sub-screens open. That was tested on `v0.0.0-g72e2584`; **the machine now runs
`v0.0.0-gc3ceadb`** (sha256 `81e1775e…`), which adds the whole-branch review's
fixes below.

### The whole-branch review (opus) found three Important issues — all fixed

1. **The "passes reached the plate" test could not fail.** It built its baseline
   by calling `ftBuildPlate`, the same function the flow uses, so any bug INSIDE
   `ftBuildPlate` was invisible — `fitted.Passes = passes + 1` left all 46
   packages green. Operator picks 2, machine cuts 3. Now asserts against the
   plate the engraver was actually handed, via `freetextPlateHook`.
2. **32-bit tick overflow.** `Plate.Duration`, bspline's accumulator and
   `engraveStatus.Completed` were `uint` — 32 bits on the RP2350. THREE proof
   patterns overflow at 1.0 mm/s x 8 passes (CONSTPROOF! 1.216x MaxUint32).
   The countdown would show "80:27" for a seven-hour job, then underflow.
   Widened to 64-bit (+512 bytes flash, measured). A second latent defect fell
   out: `duration - completed` is unsigned and `completed` legitimately exceeds
   `duration` after a resume, so it now saturates at `0:00`.
3. **The gear was drawn on the Title and Footer screens and did nothing** —
   `ftLineEntryFlow` also built a `NewTextKeyboard` and never called
   `Settings()`. Now a `NewLineKeyboard` without the key.

Re-reviewed and cleared to ship: the reviewer rebuilt both firmware images with
TinyGo to confirm the size cost, narrowed all five widened fields back to prove
the reflection pin fails, and reimplemented the old countdown to confirm the
strings are identical up to MaxUint32.

**BOTH REPOS ARE PUSHED.** `bg002h/seedhammer` `main` at `c3ceadb` (25 commits),
`bg002h/mnemonic-engrave` `master` at `3d82ae2` (15 commits). **No tag, no
release** — the operator withheld those deliberately, pending the plate tests.

### NOT verified — needs steel, and the operator ran out of plates

| test | why it matters |
| --- | --- |
| passes 1 vs 3 on a plate | the feature's entire purpose |
| confirm names `passes: 3` on a plate actually approved | the screen must not lie about what it cuts |
| `z` at 3.0 mm | see §4 — an open bet against the model |
| `~` at 8 mm/s vs 4 mm/s | whether §1's feed change was ever justified |

## 3. OPEN DECISION — the gear's visibility diverges from the spec

`SPEC_seedhammer_engraving_settings.md` §4.1 says the gear is **shown only once a
proof pattern is loaded**. What shipped gates the *option lists* instead:

```go
func NewTextKeyboard(ctx *Context) *PassphraseKeyboard {
	return newPPKeyboard(ctx, true, true)     // gear unconditionally on
}
```

So the gear is always visible, and drilling in off a proof gives one fixed entry
per parameter with a lead saying *"adjustable on test patterns only"*. The
operator found this confusing enough to report it as "only default options",
which suggests the lead is not carrying the message on the real panel.

**This is the one requirement that fell between two task briefs** — Task 5's
brief carried the keyboard flag but not the visibility gate, so the task was
reviewed correctly against a brief that had already dropped it.

Two options, operator to choose: **hide the gear until a proof is loaded** (spec,
no dead end, but undiscoverable), or **keep it visible and sharpen the locked
wording** (discoverable; recommended, since a hidden gear can only be found by
someone who already knows it exists).

## 4. `z` is now 4.5 units and has NEVER BEEN CUT

`5dde32b` brought `z` from 6 units (y 3..9) to 4.5 (y 3.75..8.25), keeping the
crossbar untouched — the glyph stays centred on y=6, so the diagonal still
crosses at x=459 and the bars keep the 0.5-over-4 house angle.

**The counter table disagrees with the change and that is deliberate:**

```
height 6.0 (before)   0.600mm   ok, exactly on the two-stroke floor
height 4.5 (now)      0.350mm   tight: never reaches the floor
```

The floor is a **model** of when two strokes merge at a 0.30 mm stroke. It has
never been checked against steel for this glyph, and every model in this
investigation needed correcting against a plate. **Cut `z` at 3.0 mm.** If it
reads, the floor is conservative and other glyphs have headroom nobody has
claimed. If it merges, 6 units comes back.

## 5. SYSTEM-WIDE PROMOTION IS NOT A GATE REMOVAL

Operator directive: after pushing, unlock the gear so the settings apply to every
plate, not only proofs. **The speed half and the passes half have very different
costs, and they should be split.**

**Speed — plausibly cheap.** It scales every rune's duration uniformly, so the
constant-time property is not obviously disturbed. Needs proving, not assuming.

**Passes — has a known, specific hazard.** The constant-time budget is computed
from a SINGLE pass:

```go
// engrave.go, computing the per-rune budget for constantAlphabet
infs := timeConstantPath(planEngraving(knotBuf, conf, func(yield func(c Command) bool) {
	engraveSpline(yield, bezier.Point{}, em, fh, spline)     // one pass, always
}))
```

Passes multiply a rune's engraved knots by N, so the scaled total would overrun
`runeDuration` and hit `timeScaler`'s **"unaligned delay" panic** — the exact
failure F-62 records when a curved glyph changed its knot count. **On the device
that is a firmware panic mid-plate, needle down, on a SEED plate.**

`ConstantStringer` also has no `Passes` field by design, and
`TestConstantStringerHasNoPasses` asserts its absence by reflection so that
adding one forces the author to read why.

**So promotion is risk-set work** — it puts a motion parameter on a real seed
plate — and needs brainstorm → spec → R0 before code. Verify the panic first;
it is cheap and it decides the whole shape.

## 6. Open work, ordered

- **Fold the whole-branch review** (opus, over `f97c725..72e2584`) when it returns.
- **Decide §3**, the gear visibility.
- **Push `main`** — operator has authorised push, and explicitly NOT tag or release.
- **Cut the §2 plates** when plates arrive, then `z` at 3.0 mm (§4).
- **Re-test 8 mm/s vs 4 mm/s** (§1) now that the play is fixed.
- **Then** the system-wide promotion (§5), speed and passes split apart.
- Three glyphs remain of the sixteen: `O`, `o`, `8`. Scope directive still in
  force: **only the sixteen.**
- **Both repos are pushed and in sync.** Nothing outstanding to push. Tag and
  release remain deliberately undone.
- `F-63` records that strike CURRENT is a third depth lever the firmware cannot
  reach on this board (`Ichop = 0`, resistor-fixed; the head is gated by a TI
  DRV8701). `F-64` is the `VOLTPROOF!` idea — engrave the negotiated supply
  voltage and needle dwell onto the plate so a depth plate documents its own
  conditions.

## 7. Standing constraints

- **Always `~/bin/sh/sh2-flash`, never picotool by hand.** Judge a boot only on
  MACHINE power — a laptop port gives a dark screen on firmware the bootrom
  ACCEPTED, because `Init()` wants a 20–28 V USB-PD contract before it configures
  the LCD. **A device re-enumerating as BOOTSEL after a flash on laptop power is
  EXPECTED, not a failure.**
- If it does not boot: **do NOT burn another OTP slot.**
- `gh` defaults to UPSTREAM `seedhammer/seedhammer`. Every fork operation needs
  `--repo bg002h/seedhammer` explicitly.
- **Never a bare `go test ./... -update`.** Re-record scoped with `-run`, then
  confirm `git status`.
- **max k = 2 is a security property.** k=3 is refused; see the `*` reasoning in
  `TestPassphraseRunPartition`.
- `layoutNavigation` indexes a fixed `[3]int` — **a fourth nav affordance
  panics.** Text, Title and Footer are all at 3/3.
- `ChoiceScreen` does not scroll and draws over its own title past ~7 entries.
- Insert `FOLLOWUPS.md` entries **before** `## Resolved`. Stage paths explicitly.
- All Go work runs under `nix develop --command`.

## 8. Process notes from this session, worth keeping

Subagent-driven development over an 8-task plan caught, in the plan text I wrote:
**seven errors** — four functions that do not exist, a non-existent iterator
hazard asserted twice, an under-scoped brief, a broken ImageMagick command, and a
wrong-variable typo that would have fed the plate size in as the engraving speed.

And **five tests that passed over broken code**, including one where the operator
picks 2 passes and the machine cuts 8, and one where the confirm screen stops
naming the pass count while the plate still cuts deep.

**The plan's structure held; its transcribed code did not.** Every one was caught
by the per-task review loop, none by re-reading the plan. The last was found in
the final task — an argument against relaxing the process as a run nears its end.

**Mutation testing found all five.** The two procedural rules that matter:
assert the substitution matched before running the test (a silently-failing
`sed` reads exactly like a surviving mutation, and produced two false "0
failures" today), and restore from a **file copy**, never `git checkout` — which
reverted real uncommitted work once.
