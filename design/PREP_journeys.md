# Prep — documenting operator journeys

Written 2026-08-11 as a starting map for the next context. It records only what
would otherwise have to be re-derived; everything else is in
`CONTINUITY_2026-08-11b.md`, which this does **not** replace.

## The spine: nine programs, in the order the operator pages through them

From `gui/gui.go`'s `program` enum (`:166`) and the titles in
`StartScreen.draw` (`:1871`) — read from the enum, not from the switch, because
the switch could omit one silently:

| # | enum | title on screen |
| --- | --- | --- |
| 1 | `backupWallet` | **Backup Wallet** |
| 2 | `engravePassphrase` | **BIP-39 Password** |
| 3 | `engraveText` | **Engrave Text** |
| 4 | `engraveXpub` | **Account Xpub** |
| 5 | `engraveBundle` | **Engrave Bundle** |
| 6 | `engraveSingleSig` | **Engrave Single-Sig** |
| 7 | `engraveMultisig` | **Engrave Multisig** |
| 8 | `bip85Derive` | **BIP-85 Child Seed** |
| 9 | `unlockPayload` | **Sealed Payload** |

Two ordering facts the enum's own comments record, and which a journeys doc
will trip over if it re-orders anything: `engravePassphrase` and `engraveText`
were **inserted** rather than appended so that `bip85Derive` stays the last
*navigable* program (wrap and pager sites are keyed to it), and `unlockPayload`
was **appended** deliberately — it is the one program that does not always
appear. It is conditional on a payload being present in flash.

Below these sit ~40 sub-flows (`gui/*_flow.go`, `*_inspect.go`, `*_pick.go`…).
Enumerate them with:

```sh
cd /scratch/code/shibboleth/seedhammer
grep -rn 'func .*Flow(ctx \*Context' --include='*.go' gui/ | grep -v _test
```

## Constraints any operator-facing document inherits

These are already normative and must not be contradicted or quietly softened:

- **The sealed-payload wipe is INCOMPLETE**, by explicit operator decision.
  `README.md` "Security limitation" and `SPEC_encrypted_payload_delivery.md`
  §2.2 item 16. **What actually protects the operator is physical custody, not
  the wipe** — the device is deliberately debuggable (SWD readable, BOOTSEL not
  disabled). Any journey that ends "and then you are safe" is wrong.
- **Program scope (§2.2 item 12, operator ruling).** Only data entering via
  **Sealed Payload** gets the security wipes. Other programs — including legacy
  ones that read encrypted payloads — do not. A journeys doc must not imply
  otherwise; this is the single most misread rule in the project.
- **`ms1` never travels over NFC.** It is typed on the air-gapped keypad, or
  delivered through the sealed-payload path.
- **§10.2.4's idle wipe**: warning at 3:00, wipe at 3:30, and it is keyed on
  *effective* input as of F-103. Row 4 (the passphrase wipe) shares that
  mechanism.
- **F-83, accepted:** a plate under the needle cannot be wiped mid-cut.

## Existing material to build on rather than duplicate

- `SPEC_encrypted_payload_delivery.md` §10.2.x — the Sealed Payload journey is
  already specified screen by screen. Journeys documentation should *reference*
  it, not restate it, or the two will drift and the spec is normative.
- `README.md` — has the operator-facing security section already.
- `design/RUNBOOK_custom_boot_key.md` — the shape to imitate for a procedure a
  human follows at the machine.
- `cmd/emu` + `sh-sim` — run any firmware ref in a browser. **This is how to
  walk a journey without a plate**, and it carries a real sealed test payload
  and its passphrase deliberately, so the Sealed Payload journey is walkable
  end to end. Note **F-121**: the emulator does *not* home, so anything about
  head motion or resumed cuts observed there is not what the machine does.

## Open threads that touch this work

- ~~**The flashed firmware's boot has not been judged.**~~ **RESOLVED
  2026-08-11 (operator): `v0.0.0-g97e38c1` boots properly on machine power.**
  So the journey-visible changes in that build are live on the machine and can
  be walked there, not just in the emulator: the `%` glyph in the KDF progress
  screen, `|` replacing the invisible `·` on four screens, and a shortened
  §10.2.3 warning.
- **`me` is at `v0.5.1`**; `v0.5.0`'s archives self-report `0.4.0` and are left
  as published. If a journey tells an operator to check `me --version`, say
  which answers are expected and what `0.4.0` means.

---

# First task (operator ruling 2026-08-11): show the plate layout while it cuts

**Extend the simulator to display the final layout of a cut plate at the
beginning of the engrave, and to indicate on that layout what is currently
being engraved.**

Both halves already exist separately. The work is joining them in the emulator,
not building either from scratch.

## ✅ BUILT 2026-08-11 — fork branch `emu-plate-overlay`

The panel sits **beside** the 480x320 canvas, never on it: `cmd/emu/platform.go`
says the emulator adds no screens, and a plate view the device does not have
belongs where nobody can mistake it for firmware.

How the plan reaches the emulator, since nothing below `gui` offered it before:
`gui.PlateAware` is an **optional** interface an `Engraver` may implement,
invoked once per job in `runEngraving` right after `pl.Engraver(stall)`. It
lives behind a `!tinygo` / `tinygo` file pair (`gui/plate_hook.go`,
`gui/plate_hook_tinygo.go`), so the **firmware image does not contain it**.
That matters because a spline is seed-derived geometry — F-107/F-108's own
subject — and the overlay's consumer is JavaScript on a page, outside anything
Go can wipe. `TestPlateHookIsAbsentFromTheFirmwareBuild` pins that structurally by
parsing the AST, not the bytes: a comment naming the rule is not a violation of
it. The hook carries `resumed` (`e.nknots > 0`) so a hold-to-resume keeps the
recording the single recorder exists to preserve.

Registration is exact **by construction, not by arrangement**: the plan's
control points and the recorder's integrated step deltas are both microsteps
from the machine origin. `cmd/emu/plate.go`'s `planPath` emits the layout and
matches `internal/golden.Vectorize` byte for byte
(`TestPlanPathMatchesVectorize`), so the overlay cannot drift from what
`cmd/plateview` shows.

Two hazards found while building, neither of them in the prep note:

- **A plan is an iterator and is not obliged to end.** `gui/qa.go`'s `qaPlan` is
  literally `for { … }`. Drawing the layout means ranging the whole spline
  *before* the first step, so an unbounded plan is an unbounded render and the
  tab never comes back. `maxPlanKnots` bounds it at 100,000 against a widest
  measured plate of 27,062, and reports `data-truncated`. `qaProgram` is
  unreachable in `cmd/emu` today only because `NFCReader()` returns nil — a
  property of the wiring, one feature away from false.
- **`cmd/emu/build.sh` failed on every rebuild after the first.** It copies
  `wasm_exec.js` out of `GOROOT`, which under Nix is mode 444 in the store, so
  the copy landed unwritable. `cp -f`.

### What the firmware actually pays for the seam — measured, after the comment lied

The TinyGo stub's comment said the empty `notifyPlate` "costs the device an
empty call the compiler removes." Nobody had asked the compiler. **It does not
remove it.** Built at production settings with `VERSION` pinned so the stamp
could not explain a difference:

| build | sha256 |
| --- | --- |
| `97e38c1` — before the hook | `0a379302…` |
| `71f1d42` — with the hook | `8c4380c4…` |
| `71f1d42` — with ONLY the `runEngraving` call deleted | `0a379302…` |

The third is byte-identical to the first, so the entire delta is that one call:
**486,697 bytes across 3,480 of 5,146 UF2 blocks at unchanged total size** —
code shifting downstream, not code being added.

**The comparison has a control**, because a difference without one proves
nothing: the same SHA built twice in two different directories gave the
identical sha256, so the build is byte-reproducible across paths.

The trade still holds — one call to an empty function per *engraving job*,
against an interface that cannot reach the image at all. Corrected in
`gui/plate_hook_tinygo.go` (fork `345d79c`). Generalises, and it is this repo's
own rule turned on its author: **a comment asserting what a compiler did is a
machine-checkable claim, and this one was checked only because the operator's
boot report prompted the question "does the machine need a reflash?"** It does
not: nothing behavioural changed on the device.

## Half 1 — the final layout is already renderable, before the cut

`cmd/plateview` renders a plate from **the same `FitBlocks` / `EngraveFitted` /
`PlanEngraving` calls the firmware makes**, stroked at the production 0.3 mm cut
width. Its own doc comment is the claim to lean on: *"It is a PREVIEW OF THE
TOOLPATH, not a drawing of it… What you see is the cut."* Parameters are pinned
against `cmd/controller/platform_sh2.go` by `internal/sh2`'s own test, so the
preview cannot silently drift from the machine.

```sh
go run ./cmd/plateview -list
go run ./cmd/plateview -plate bothproof -o /tmp/plate.png
```

The plan is available **before** any step is emitted, which is exactly what
"at the beginning of engrave" needs. `cmd/plateview` is a host command, so the
reusable part is the geometry it calls, not the command itself.

## Half 2 — what is currently being engraved is already recorded

`cmd/emu`'s `toolpathRecorder` decodes the driver's **actual step stream** into
head motion — not the plan, which is what makes it the honest source for
progress. Exposed to the page by `cmd/emu/toolpath_js.go:47` as
`window.shToolpath`:

| call | returns |
| --- | --- |
| `shToolpath.reset()` | start a fresh recording |
| `shToolpath.summary(frac)` | JSON digest + anomalies (`Summary`) |
| `shToolpath.path()` | JSON `[[x,y,needle],…]` |
| `shToolpath.svg()` | an SVG of the decoded motion |

`path()` already carries the needle flag per vertex, so "what has been cut so
far" is a filter over it rather than new instrumentation.

`cmd/emu/toolpath.go` deliberately carries **no build tag** so it is
host-testable; `toolpath_js.go` is the `//go:build js` half. Keep that split.
**Correction 2026-08-11:** it is not `confinement_test.go` that enforces it —
that test pins the *sealed payload* boundary, and `toolpath.go` names none of
its guarded identifiers. What actually pins the split is `toolpath_test.go`,
which is untagged: verified by mutation, adding `//go:build js` to
`toolpath.go` fails the host build with four undefined symbols.

## What to watch

- ~~**`maxVertices` is 200,000**… a full plate can approach that.~~
  **MEASURED 2026-08-11 and the concern is unfounded.** A full seed plate
  decodes to **3,440 vertices**, `truncated=false` — 58x headroom against the
  cap — and `path()` is 60 KB, cheap to poll whole. constproof, the widest
  plate, is 3,959. No decimation and no incremental API are needed. (The trap
  the numbers *did* find is elsewhere: a plan is an iterator and `qaPlan` never
  ends, so rendering a layout up front must be bounded. See `maxPlanKnots`.)
- **F-121 — the emulator does not home.** The device homes to the plate origin
  before every run (`homingEngraver`); the emulator does not. Any overlay that
  aligns recorded motion to planned geometry must account for that, or a
  resumed cut will render offset against the plan. **This is the trap most
  likely to be hit by this exact task.**
  **FIXED 2026-08-11, as a prerequisite rather than a neighbour** — the overlay
  cannot register without it. `toolpathRecorder.Home()` records the needle-UP
  travel to the origin that `homingEngraver` performs on the first write of
  every job and again on `Close`; `jobRecorder` holds the once-per-job state,
  in the untagged file so it is host-testable. Measured before the fix: the
  seed plate ends at (435840, 220160) = **(68.1 mm, 34.4 mm)** on an 85 mm
  plate, and that was the offset a resumed cut would have drawn at. Confirmed
  in the browser: abort mid-"hello", hold to resume, and the resumed strokes
  land on the plan.
- The recorder is deliberately **one instance across every job**
  (`cmd/emu/platform.go:185`) so an aborted-and-resumed plate records as one
  motion. A per-plate overlay may want `reset()` at engrave start — changing
  that lifetime would break the abort/resume comparison the recorder exists for,
  so prefer resetting over re-scoping.
- `sh-sim` runs any firmware ref in a browser; `cmd/emu/build.sh` and
  `index.html` are the page. The emulator carries a real sealed test payload and
  its passphrase **deliberately** (operator ruling) — do not "fix" that.
