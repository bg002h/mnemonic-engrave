# SPEC — the engraving settings screen, and passes per character

**2026-08-06.** Fork `seedhammer`, `gui/`. Brainstormed with the operator this
session. Supersedes the flow placement in
`SPEC_seedhammer_proof_speed_picker.md` §4.1 — Speed moves off the flow and
behind a gear key; everything else in that spec stands.

## 1. Why

The operator watched stock v1.4.3 engrave **each letter twice** and cut deeper
than the fork does. That doubling is almost certainly the constant-time padding:

```go
// engrave.go, ConstantStringer
// accum accumulates the fraction each rune in txt contributes towards
// engraving the total number of runes (longest). This is to spread out
// the repeat runes.
for range longest {          // loops LONGEST times, not len(txt)
```

A word shorter than the longest in its field has its runes **re-engraved in
place** so every word takes the same time. So "engrave a character N times" is
not new machinery — it already exists, driven by timing equalisation rather than
by a depth choice. This makes the multiplier deliberate.

**Depth is the point.** A faint plate is the failure this machine exists to
prevent, and passes are the most direct lever on depth the firmware has.

## 2. Scope

**In:** a `Passes` count (1, 2, 3, 4, 5, 8) applied per character in place; a
settings screen reached from a gear key on the text keyboard; Speed relocated
onto that screen; Clear buttons on the Title and Footer fields.

**Out:** persistence; acceleration and jerk (later members of the same family);
any change to constant-time engraving; any effect on seed, descriptor or
passphrase plates.

Operator decision: **proof-scoped now, promotable to system-wide later**, the
same disposition as Speed and eventually acceleration and jerk. The design's job
is to make that promotion a deletion rather than a restructure.

## 3. The funds surface is designed OUT, not merely avoided

`ConstantStringer` is used by `backup/backup.go` (seed plates) and
`backup/passphrase.go`. **The free-text path — every proof plate — does not use
it**, going through `engrave.String` via `backup/freetext.go`. So a proof-scoped
`Passes` never touches the constant-time machinery.

`Passes` therefore lives on **`engrave.StringCmd`**, beside the existing
`LineHeight` knob. `ConstantStringer` is a **different type with no such field**,
so seed and passphrase plates are *structurally* unreachable rather than
un-plumbed. A plumbing mistake cannot leak a pass count onto a real backup; only
a deliberate edit can.

That is the whole reason the field goes there rather than on `engrave.Params`,
which would have reached every path automatically and made promotion free — at
the cost of one bug being able to change how a seed is cut.

## 4. Design

### 4.1 The gear is a KEY, not a nav button

`layoutNavigation` indexes a fixed `[3]int` by `Button - Button1`, so a fourth
nav affordance **panics** rather than laying out badly. The Text screen already
spends all three on Back / Clear / OK. So the gear cannot be a nav button.

The keyboard has an action model — `ppRune`, `ppPageCycle`, `ppReveal`,
`ppBackspace` — and the gear is a new action in the grid beside backspace. No
nav slot, no flow step.

**Text keyboard only.** `newPPKeyboard(ctx, newline)` serves both the passphrase
program and free text; the gear takes the same kind of flag `newline` already
uses, so it can never appear while a passphrase is being typed.

**Shown only once a proof pattern is loaded.** Promotion to system-wide is
deleting that condition.

### 4.2 Speed moves off the flow

```
before   QR -> Font -> Size -> Text -> Speed -> Title -> Footer -> Confirm   8 steps
after    QR -> Font -> Size -> Text ----------> Title -> Footer -> Confirm   7 steps
                                 |
                                 gear -> Engraving -> Speed  -> 8/6/4/2/1 mm/s
                                                   -> Passes -> 1/2/3/4/5/8
```

Two ways to set one value is a defect, so `ftStepSpeed` is **removed**. The
ordinary path becomes *shorter than today*, and acceleration and jerk later are
two more rows on a screen that already exists.

**Two levels of `ChoiceScreen`, not one flat list.** `ChoiceScreen` does not
scroll and `op.Layer` draws content OVER its title past roughly seven entries,
so a flat list cannot hold the family once accel and jerk arrive. Each level is
the existing, tested widget doing what it already does.

### 4.3 Passes applies in place

Repeat `engraveSpline` N times **before advancing `dot.X`**, so each glyph is
re-cut where it stands. Not a second pass over the whole plate: re-cutting in
place carries no repositioning error between passes, while a whole-plate repeat
accumulates one.

Values **1, 2, 3, 4, 5, 8**; default **1**, which is today's behaviour. Six
entries, inside the `ChoiceScreen` budget. Time is linear in passes — a full
proof plate at 4 mm/s goes from ~15 min to ~2 h at 8 — so the ceiling is a
practical one and there is no unbounded value to validate.

`backup.EngraveFitted` carries it as `Fitted.Passes`, a struct only the
free-text path constructs.

### 4.4 Clear on Title and Footer, without confirmation

| field | cap | confirm? | why |
| --- | --- | --- | --- |
| Text | **uncapped** — a proof pattern is hundreds of characters and the field is the only copy | **yes** | a mis-tap destroys work with no undo |
| Title, Footer | `MaxTitleLen`, ~18 characters | **no** | retyping costs seconds; a prompt costs more than the mistake |

**The confirmation tracks the cost of the error, not the identity of the
button.** Recorded because the asymmetry looks like an inconsistency and is not
— do not "fix" it.

Same visibility rule on all three: Clear appears only when the field is
non-empty, so it is never offered as a no-op. Every screen then sits at exactly
3/3 nav slots.

## 5. Tests, written first

1. **`Passes = 1` reproduces today's plate byte-for-byte.** No golden moves.
   That is the additive proof.
2. **`Passes = N` multiplies engraved duration by ~N**, and — load-bearing —
   **the second pass's spline is identical to the first's**, which is what
   proves *in place* rather than offset. A pass count wired to a label and never
   to the planner passes everything else.
3. **Seed, descriptor and passphrase plates are unaffected**, asserted through
   their own builders rather than by inspection.
4. **The gear is absent with no proof loaded, present with one, and absent from
   the passphrase keyboard entirely.**
5. **Speed is reachable only through the gear** — the flow has no Speed step, and
   the value still reaches the plate (the existing
   `TestFlowCarriesTheChosenSpeedToTheEngraver`, re-pointed at the new route).
6. **Clear on Title and Footer clears without a prompt**, and neither is offered
   on an empty field.

**Mutation checks.** A suite that survives these is not testing the feature:
default `Passes` to 2 → test 1 fails; advance `dot.X` between passes → test 2
fails; give `ConstantStringer` a pass count → test 3 fails; drop the
proof-loaded condition on the gear → test 4 fails.

## 6. Risks

- **A pass count is invisible on the finished plate.** Same exposure as Speed and
  handled the same way: the confirm screen names it when it is not the default.
- **Time is linear and unbounded-feeling.** 8 passes on a full plate is two
  hours. The confirm screen already prints an estimate; it must reflect passes.
- **The keyboard is shared with the passphrase program.** The gear flag is the
  only thing keeping it off a seed-adjacent screen, so it gets its own test
  rather than being trusted.
- **Promotion is a deletion, but not a free one.** Making `Passes` system-wide
  means reaching `ConstantStringer`, where the repeat count is already the
  padding mechanism. A uniform N× multiplier plausibly preserves constant time by
  scaling every rune's budget equally — **plausibly is not proven**, and that
  proof is the price of promotion, not of this slice.
