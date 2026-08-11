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

- **The flashed firmware's boot has not been judged.** Hardware carries
  `v0.0.0-g97e38c1` (flashed 2026-08-11, signature verified, flash verified),
  but it must be powered from the machine supply before anything is concluded —
  a laptop port gives a dark screen indistinguishable from a rejected signature.
  Several journey-visible changes are in that build: the `%` glyph in the KDF
  progress screen, `|` replacing the invisible `·` on four screens, and a
  shortened §10.2.3 warning.
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
host-testable; `toolpath_js.go` is the `//go:build js` half. Keep that split —
`cmd/emu/confinement_test.go` enforces the file-level boundary, and it is
mutation-tested.

## What to watch

- **`maxVertices` is 200,000** (`cmd/emu/toolpath.go:45`). A full plate can
  approach that; check the cap before treating `path()` as complete for a live
  overlay, and decide whether the overlay needs decimation.
- **F-121 — the emulator does not home.** The device homes to the plate origin
  before every run (`homingEngraver`); the emulator does not. Any overlay that
  aligns recorded motion to planned geometry must account for that, or a
  resumed cut will render offset against the plan. **This is the trap most
  likely to be hit by this exact task.**
- The recorder is deliberately **one instance across every job**
  (`cmd/emu/platform.go:185`) so an aborted-and-resumed plate records as one
  motion. A per-plate overlay may want `reset()` at engrave start — changing
  that lifetime would break the abort/resume comparison the recorder exists for,
  so prefer resetting over re-scoping.
- `sh-sim` runs any firmware ref in a browser; `cmd/emu/build.sh` and
  `index.html` are the page. The emulator carries a real sealed test payload and
  its passphrase **deliberately** (operator ruling) — do not "fix" that.
