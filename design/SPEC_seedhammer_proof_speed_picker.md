# SPEC — proof-scoped engraving speed selection

**2026-08-06.** Fork `seedhammer`, `gui/freetext_flow.go`. First slice of the
operator's "adjust velocity, acceleration and jerk" request, deliberately
narrowed. Recon: the workflow behind `design/RECON_cusp_dot_pileup.md`'s sibling
investigation, summarised in §7 below.

## 1. What this is, and what it is not

**Is:** the **engraving feed** (`StepperConfig.EngravingSpeed`) selectable from a
fixed list, in the **free-text program only**, unlocked only when a **proof
keyword** has been used, applying to the plate being composed and nothing else.

**Is not:** acceleration, jerk, travel speed, `TicksPerSecond`, persistence,
free numeric entry, or any effect on seed, descriptor or passphrase plates. Each
of those is deferred deliberately; §8 says what the upgrade path is and why this
slice does not have to be unpicked to get there.

### Operator decisions taken this session

1. **Speed only** in the first slice; accel and jerk later.
2. **Any proof keyword unlocks it** — no new trigger root, so the LEXICON's
   one-kind-per-parameter-slot rule is never engaged.
3. **A picker screen, after Text** — speed changes no geometry, only timing.
4. **A fixed list**, not a numeric box.

## 2. Why this scope is the safe one

Recon found the machine has **no motion-parameter safety envelope whatsoever**:
no `StepperConfig.Validate`, no bounds constant, no planner guard. The failure
modes are not theoretical:

- `EngravingSpeed = 0` and `Jerk = 0` **both panic**, on an integer divide by
  zero, at **different sites**: `Jerk = 0` at `engrave/engrave.go:1155` via
  `bezier/bezier.go:300`, and `EngravingSpeed = 0` at `engrave/engrave.go:1117`
  in `timeScaler.Scale`. Neither is rejected; both crash with the job in an
  undefined state.
- `Acceleration = 0` does **not** panic. It silently plans motion at **3× its
  own configured velocity limit** — measured 153,600 microsteps/s against an
  `EngravingSpeed` of 51,200 (24.0 mm/s against 8.0). No error, no warning. A
  silent limit violation is worse than a crash.
- `Speed` above `TicksPerSecond` is **silently rate-limited, not rejected**
  (`stepper/stepper.go:49-53` clamps to ±1 microstep per tick per axis; `fill()`
  has no return value and `Driver.Knot()` inspects no error, so **no error path
  exists**). The loss is permanent rather than deferred:
  `bezier.Interpolator.Step()` returns `true` only for the segment's *fixed*
  planned tick count and then stops regardless of how far behind the physical
  position has fallen, so the shortfall is never made up. Every later stroke on
  that plate is offset — a wrong glyph in steel, no warning, and a preview that
  looked right. The shipped config has `Speed == TicksPerSecond` **by
  coincidence of the constant block**; nothing enforces or documents it.

All four verified 2026-08-06 by an independent pass at HEAD `0ce071f`, the two
runtime claims by driving the real planner rather than by reading.

A fixed list of five values, none of them zero, none of them touching `Speed` or
`TicksPerSecond`, **cannot reach any of those states**. That is the argument for
the list over the numeric box: not that typing is hard, but that the validation
layer which would have to catch a typo does not exist yet.

## 3. The ceiling is physical, not arbitrary

```
ftSpeedCeiling = 8 * mm     // upstream stock engraving feed
```

Every offered speed must be `> 0` and `<= ftSpeedCeiling`, asserted by test.
Two independent reasons, either sufficient:

1. **8 mm/s is what upstream shipped and validated.** Nothing above it has ever
   been cut on this machine.
2. **Above 8 mm/s, StallGuard arms during the cut.** `minimumStallVelocity` is
   `8 * mm` and becomes `TCOOLTHRS = 234`; the TMC2209 enables the stall output
   only *above* that velocity. Engraving at 8 mm/s gives `TSTEP = 234.4`, just
   outside the window — so the cut is unprotected by design, and a hammering
   load would throw false stalls if it were not. Raising the feed past 8 would
   drag engraving **into** the StallGuard window and start tripping on the
   hammer itself. See the worked table at `cmd/controller/platform_sh2.go`'s
   `minimumStallVelocity` (commit `0ce071f`).

The list — **8.0, 6.0, 4.0, 2.0, 1.0 mm/s** — is five entries. `ChoiceScreen`
has no scrolling and `op.Layer` draws content *over* the title, so a list past
roughly seven entries is silently covered rather than clipped. Five is inside
that budget with room; a test pins the count so a later addition has to think
about it.

Note the current device default is **4 mm/s** (commit `343fb05`), so the list
spans both sides of it. The ceiling is upstream stock, not the current default —
otherwise the comparison the operator actually wants to run, 4 against 8, could
not be run.

## 4. Design

### 4.1 Placement

```
QR -> Font -> Size -> Text -> Speed -> Title -> Footer -> Confirm -> Engrave
                              ^^^^^
```

**After Text, unlike Font and Size.** Those two come first because they change
plate **capacity** (44 columns at 3.0 mm in `font/sh` against 39 in
`font/constant`). Speed changes **no geometry at all** — only the tick counts on
an already-decided toolpath — so it has no reason to precede typing, and placing
it after means **the flow already knows whether a proof keyword was used** when
the screen runs. A proof is loaded on the Text screen; a picker before Text
would need a Back to see it.

Insert `ftStepSpeed` into the `iota` block after `ftStepText`. The Back idiom is
unchanged: `step -= 2` against the loop's trailing `step++` is a net −1.

### 4.2 The Speed screen

Mirrors `ftSizeOptions` exactly, including the state-not-a-choice idiom:

- **Off a proof composition** (`!ftPlanIsProof(plan)`): one entry naming the
  machine default, e.g. `4.0mm/s (default)`. State, not a decision. Taking it
  changes nothing. This is what "any proof keyword unlocks it" means
  mechanically — the gate is the composition, not a new keyword.
- **On a proof composition**: the five rungs, with the machine default
  preselected so the common path is still one checkmark.

`PASSPROOF!` lives in the passphrase program, not free text, and is out of scope.

### 4.3 The override itself

```go
// ftParamsAtSpeed returns params with only EngravingSpeed replaced.
// Speed and TicksPerSecond are deliberately untouched -- see §2.
func ftParamsAtSpeed(params engrave.Params, mmPerSec float32) engrave.Params
```

`0` means "leave it alone". Everything else is threaded through the `params`
argument that `ftEvaluate` and `ftBuildPlate` **already take**, so no new value
flows anywhere. Seed, descriptor and passphrase plates call
`ctx.Platform.EngraverParams()` on entirely different paths and are structurally
untouched.

**This reaches the machine, and that was checked rather than assumed.** `Plate`
is `{Duration uint, Spline bspline.Curve}` (`gui/gui.go:474`); the spline is
planned by `ftBuildPlate` from the params it is handed, and the per-knot tick
counts that encode velocity are baked in at that moment. The engrave screen
consumes that spline.

### 4.4 The one place that must change outside the flow

```go
// gui/engraver.go:147
conf := e.pl.EngraverParams().StepperConfig
res := newSplineResumer(drv, e.safePoint.Resume(conf))
```

The resume-after-interruption path **re-reads the platform config** while the
spline it is resuming was planned with the flow's. Immutable today, so the two
cannot disagree; **divergent the moment speed is selectable** — a plate planned
at 1 mm/s that is paused and resumed would compute its catch-up at 4 mm/s. It is
a repositioning move rather than a cut, so it is not a wrecked plate, but it is
wrong and it is invisible until someone pauses a slow plate.

**Fix: the `Plate` carries the `StepperConfig` it was planned with**, and the
engrave job resumes from that rather than from the platform. This is also the
single most valuable piece of forward compatibility in the slice — see §8.

Related and **not** changed: the progress countdown re-reads `TicksPerSecond`
every frame (`gui/gui.go:2649-2653`). Sound here, because this slice never
touches `TicksPerSecond`. It becomes a real problem in the system-wide version,
and §8 records that.

### 4.5 The confirm screen

`ftConfirmSummary` already prints rungs, lines, QR and font. **Append the speed
only when it is not the machine default** — zero change for every ordinary
plate, and impossible to miss on the one plate where it matters. The operator
must not be able to approve a non-default speed without seeing it, because
nothing on the finished plate records what it was cut at.

## 5. Tests, written first

1. **The default reproduces today's plate.** Off a proof, and on a proof with
   the default taken, the built spline equals the pre-feature one. **No golden
   moves.**
2. **The speed reaches the motion, not just the label.** Build the same
   composition at 8 and at 1 mm/s and assert `Plate.Duration` differs by
   approximately 8×. *This is the load-bearing test* — everything else could
   pass with the value plumbed to a label and never to the planner.

   The suite is **nearly** blind here, but not entirely: all **five**
   `golden.CompareBSpline` sites plan at a test-local `engravingSpeed = 8*mm`
   and never the device's `4*mm`, so no golden can see a feed change — but
   `TestPassphraseRuneDurationPin`
   (`engrave/passphrase_alphabet_test.go:463-472`) *does* pin an exact tick
   count, `wantDuration = 572245`. So this test is the first thing to pin
   **plate** motion, not the first to pin motion at all. It also means a change
   that altered the passphrase alphabet's timing would already be caught, which
   is worth knowing before assuming the net has no threads in it.
3. **`Speed` and `TicksPerSecond` are untouched** by the override, for every
   entry in the list. Guards the silent-clamp regime of §2.
4. **Every offered speed is `> 0` and `<= ftSpeedCeiling`**, ranging over the
   list so a later addition cannot slip past. Pins the count at five for the
   `ChoiceScreen` budget.
5. **Off a proof the screen offers exactly one entry**, and taking it leaves the
   speed at the default. On each of `ftPlanBoth`, `ftPlanSizeFront`,
   `ftPlanSizeBack` the full list is offered.
6. **The resumer uses the plate's config, not the platform's** — construct a job
   from a plate planned at a non-default speed and assert the resumer's config
   matches the plate.
7. **Seed, descriptor and passphrase plates are unaffected**, asserted through
   their own builders rather than by inspection.

**Mutation checks** (a suite that survives these is not testing the feature):
override `Speed` instead of `EngravingSpeed` → test 3 fails. Drop the derived
params on the way to `ftBuildPlate` → test 2 fails. Raise a list entry above the
ceiling → test 4 fails. Return the platform config from the resumer → test 6
fails.

## 6. What this buys immediately

The wiggle investigation currently costs a **rebuild-and-reflash cycle per
speed** — two were spent this session to change one constant. With this, cutting
`~` at 8, 6, 4, 2 and 1 mm/s is five plates in a row on one firmware, and
`design/RECON_cusp_dot_pileup.md`'s prediction — that dot spacing is feed × 25 ms
and nothing else — becomes directly measurable rather than inferred.

## 7. Risks

- **A non-default plate is not self-describing.** Nothing on the steel records
  the feed. Mitigated for the operator by §4.5's confirm line, and bounded by
  scope: only proof compositions can be non-default, and nobody backs up a seed
  on a proof plate. **This is the reason the proof gate is load-bearing and not
  a convenience.**
- **`ChoiceScreen` overflow is silent**, not clipped. Five entries is safe;
  test 4 pins it.
- **No golden can see a feed change** — all five `golden.CompareBSpline` sites
  plan at test-local `8*mm`, never the device's `4*mm`, so the plate goldens are
  structurally decoupled from the machine's actual speed. One duration *is*
  pinned (`TestPassphraseRuneDurationPin`), so the tree is not wholly blind;
  test 2 is nonetheless the first pin on plate motion.
- **Outside the risk set that needs an R0 gate:** fork-native GUI, no normative
  codec behaviour, no Rust counterpart, no irreversible action. Implement and
  verify inline. The funds-safety exposure that *would* have pulled it in — a
  motion parameter reaching a seed plate — is exactly what the proof gate
  removes.

## 8. The upgrade path to system-wide

Recorded now so this slice is not throwaway. Recon priced the full feature as
**large**, and the cost is not the UI:

| needed for system-wide | this slice |
| --- | --- |
| `Plate` carries its planned `StepperConfig` | **done here** (§4.4) — the plan/run coherence rule the large version needs |
| A `StepperConfig` validation layer | **not built.** Unnecessary here because the list cannot produce a bad value; mandatory the moment a numeric box exists |
| One mutable source of truth on the device | **not touched.** Today there are *three* desynchronised copies: `engraverParams` embeds a *copy* of `engraverConf`; homing reads `engraverConf` directly, bypassing `EngraverParams()` (`cmd/controller/engraver.go:194-196`); `mjolnir2` latches `TicksPerSecond` once at boot |
| A setter on `gui.Platform` | **not needed** — the interface stays getter-only |
| Persistence | **not needed** — the value lives in `engraveTextFlow`'s scope like `size` |
| A replacement for `TestParamsMatchTheMachine` | **not needed** — the device constants are untouched, so the host/device binding keeps working |

The two that will hurt later and are worth knowing now: the **three
desynchronised device copies** (recon called this the single most likely place
for a subtle bug), and the **fourth copy outside the fork** at
`mnemonic-engrave/preview/params.go`, which is already stale at `8 * mm` against
the device's `4 * mm` with nothing binding it. That one is a live defect today,
independent of this feature, and is filed separately.
