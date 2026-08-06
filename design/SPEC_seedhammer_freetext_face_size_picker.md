# SPEC — free-text face and size selection

**2026-08-06.** Fork `seedhammer`, `gui/freetext_flow.go`. Operator decisions
taken this session: a **shipped step for everyone**, not a test-only trigger; and
the size list is **auto-fit plus every rung**, auto-fit remaining the default.

## 1. The problem

`engraveTextFlow` hardcodes the face:

```go
plan := &ftPlanSH            // gui/freetext_flow.go:1004
var size float32             // 0 == auto-fit
```

Neither can be changed from the UI. The **only** thing that changes either is a
proof trigger (`TEXTPROOF!`, `CONSTPROOF!`, `BOTHPROOF!`, `SIZEPROOF!<side>`),
and every one of those also replaces the text with a full proof pattern.

Consequences today:

- The fork ships **two engraving faces** and a confirm screen that already prints
  `font: %s` as though it varies, but a user cannot pick one.
- A single character can only be cut at **6.0 mm**, because `ftProofForTrigger`
  returns rung `0` for the non-sizeable triggers (`freetext_proof.go:597`) and one
  character auto-fits to the largest rung. **Every other rung is unreachable for
  short text.**
- The only way to reach `font/constant` at all is to type `CONSTPROOF!`, accept a
  prompt, then delete the pattern it wrote — a workaround that happens to work
  because `plan` is held in `engraveTextFlow`'s scope, not the entry screen's.

The immediate driver is `design/RECON_cusp_dot_pileup.md`: confirming that the
cusp dot pile-up is what the eye reads as the wiggle needs the **same glyph at
3.0 mm and 6.0 mm in `font/constant`**, and neither const-at-3.0 nor
sh-at-3.0 is reachable.

**Rejected, and why it matters:** flipping `Sizeable: true` on `CONSTPROOF!` is
not the small fix it looks like. `ftProofOutcomeFor` resolves a non-zero rung by
calling `ftBothAt`, which rebuilds the **mixed** pattern — the code says so
outright: *"reached with a rung and any other proof it would hand back
BOTHPROOF!'s plate under that proof's own trigger"* (`freetext_proof.go:678-681`).
It would silently cut the wrong plate.

## 2. Scope

**In:** a face choice and a size choice in the free-text program, applying to the
whole plate.

**Out:** per-row face or size (that is what the proof compositions are for);
any change to `ftPlanBoth` / `ftPlanSizeFront` / `ftPlanSizeBack` semantics; any
change to the proof triggers; F-58's input wedge.

## 3. Design

### 3.1 Placement — two new steps, before Text

```
ftStepQR  ->  ftStepFace  ->  ftStepSize  ->  ftStepText  ->  Title -> Footer -> Confirm -> Engrave
```

**Before Text, not after.** Face and size both change plate **capacity** — 44
columns at 3.0 mm in `font/sh` against 39 in `font/constant`
(`freetext_flow.go:32-33`). The QR step is already placed before Text for exactly
that reason. A picker after Text would let the operator type against one capacity
and then have it change underneath them.

Mechanically this is an insert into the `iota` block at `freetext_flow.go:984`
plus two cases. The Back idiom is unchanged: `step -= 2` followed by the loop's
trailing `step++` (`:1069`) is a net −1, so it keeps working with no arithmetic
changes anywhere.

**Two screens rather than one combined widget.** Both defaults are today's
behaviour and both sit at index 0, so the common path costs **two checkmark
presses and nothing else**. A combined face×size widget was rejected: it is a new
UI surface on a flow that already carries an unreproduced input wedge (F-58), and
`ChoiceScreen` (`gui/gui.go:1367`) already does exactly this job everywhere else
in the program.

### 3.2 The Face screen

`ChoiceScreen{Title: "Font"}`, choices in this order:

| # | label | plan |
| --- | --- | --- |
| 0 | `sh` | `&ftPlanSH` — the free-text plate's own face, today's default |
| 1 | `constant` | `&ftPlanConst` — the face every seed, descriptor and passphrase plate uses |

Index 0 is the default **as a property of the ordering**, matching the comment
`ftQRChoiceFlow` already carries: *"choice starts at 0 … the default is a property
of this ordering, so do not reorder the choices"* (`:536-537`).

Names come from `ftFace.Name` (`ftFaceSH.Name == "sh"`,
`ftFaceConst.Name == "constant"`), never string literals, so the screen and
`ftFaceSummary` cannot disagree.

### 3.3 The Size screen

`ChoiceScreen{Title: "Size"}`:

| # | label | `size` |
| --- | --- | --- |
| 0 | `Auto-fit` | `0` — today's behaviour |
| 1..6 | `6.0 mm` … `3.0 mm` | the corresponding `backup.FontSizes` entry |

Built **by ranging over `backup.FontSizes`** (`backup/backup.go:70`), never a
hand-written list. That set is the only one every capacity number in `backup` is
measured against, so a rung that is offered is a rung that is pinned. A
hand-written list is how an unpinned size gets offered.

### 3.4 A proof composition is state, not a choice

A proof trigger may leave `plan` as `ftPlanBoth`, `ftPlanSizeFront` or
`ftPlanSizeBack` — compositions that are **not** in the face list, and whose
rungs are a property of the pattern rather than a choice. Since the pickers sit
before Text, Back out of Text after loading a proof lands on them.

**Both screens then show a single entry naming the current composition, and it is
state rather than a decision.** This is not a new idea — it is exactly what
`ftQRChoiceFlow` already does for a sized composition:

```go
// ONE answer, and it is the state rather than a decision.
cs.Choices = []string{"No QR"}          // freetext_flow.go:524-527
```

Face screen shows `plan.Name()`; Size screen shows `ftPlateRungs`-style text when
`plan.Sized()` is true. The operator can still Back out and retype the field, but
cannot half-edit a pinned proof plate into a composition nothing measures.

### 3.5 What does not change

- **The confirm screen already prints both.** `ftConfirmSummary` formats
  `"%s  %d lines  QR: %s  font: %s"` from `ftPlateRungs(f.plate)` and
  `ftFaceSummary(...)` (`:808-813`). No change; it becomes *useful* rather than
  always reading the same.
- **Evaluation is already keyed correctly.** `ftTextEntryFlow`'s cache key is
  `(text, qr, plan, size)` (`:625-627`), so the Text screen re-evaluates against
  whatever the pickers set, with no new invalidation logic.
- **Over-capacity is already handled.** Choosing a smaller rung or the narrower
  face can make existing text no longer fit; `ftRefuse` is the existing remedy and
  needs no change. This is a pre-existing path, not a new failure mode — but it
  is now reachable without a proof trigger, so it must be tested.
- **Proof triggers keep working exactly as they do now** and continue to overwrite
  both values.

## 4. Tests, written first

1. **The defaults reproduce today's plate byte-for-byte.** Walk the flow taking
   choice 0 on both new screens and assert the resulting plate equals the one the
   current code builds. **The existing SIZEPROOF goldens must not move** — that is
   the real assertion, and it is what proves this feature is additive.
2. **`constant` + `3.0 mm` yields a plate whose faces are all `constant.Font` and
   whose sizes are all 3.0** — the case the recon needs and the one that is
   unreachable today.
3. **Every rung in `backup.FontSizes` is offered, and nothing outside it.** Range
   over the package variable in the test too, so adding a rung cannot leave the
   screen behind.
4. **Auto-fit is index 0 and yields `size == 0`**, so a reorder that changed the
   default fails.
5. **Each proof composition (`Both`, `SizeFront`, `SizeBack`) shows one entry on
   both screens**, and taking it changes neither `plan` nor `size`.
6. **Back from Text lands on Size showing what a proof set**, not the picker's own
   prior value.
7. **A rung that no longer fits the typed text is refused through `ftRefuse`**, not
   engraved.

Drive them through `hookPPWidget`, as the flow's existing tests do — register the
two new screens under their own keys so a test can choose without hardware.

**Mutation-check the suite before calling it green:** flip the face default to
`constant`, flip auto-fit off index 0, and drop a rung from the Size list. Each
must fail a test. A green suite that survives those is not testing this feature.

## 5. Risks

- **An extra two screens for every user of the program.** Accepted deliberately by
  the operator's scope decision. Mitigated by both defaults being index 0 and
  today's behaviour.
- **F-58 lives in this flow.** The input wedge was seen on a `ftLineEntryFlow`
  screen and is unreproduced. These are `ChoiceScreen`s, a different widget, so
  this does not obviously widen the exposure — but two more screens in the same
  program is two more places to see it. **If the wedge appears on a new picker
  screen, that is a second sighting and a real clue; record it rather than
  working around it.**
- **No new normative behaviour.** Nothing here touches the wire format, identity,
  validation or admission, and it is fork-native GUI code with no Rust
  counterpart — so the Rust-primary rule does not apply and this is outside the
  risk set that requires an R0 gate. Implement and verify inline.

## 6. Why it is worth doing beyond the test

It retires the `CONSTPROOF!`-then-delete workaround, it makes the confirm
screen's `font:` field mean something, and it gives the fork's second engraving
face a reason to be reachable. The recon is the trigger, not the justification.
