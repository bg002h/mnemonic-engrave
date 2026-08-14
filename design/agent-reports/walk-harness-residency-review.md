# Walk-harness residency review — `seedhammer` `10286e4..740888d` (5 commits)

**Reviewer:** independent adversarial agent (opus), 2026-08-14
**Repo:** `/scratch/code/shibboleth/seedhammer`, branch `main`
**Worktree used for all builds:** `/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/3985bd41-08d3-42b8-a967-1493b588d215/scratchpad/wt-residency` (left clean; `git status --short` empty)

**The one question:** does this diff weaken any residency / seed-material-lifetime
guarantee in the firmware?

## VERDICT

**0 Critical, 0 Important.** The diff does not weaken any firmware residency
guarantee. All four sub-questions check out; the firmware-absence claim is proved
mechanically below rather than argued. 1 Minor and 3 Nits are recorded, none
blocking.

Scope kept to the brief: no fresh audit, no plan/spec review, no browser/JS
ergonomics, no CI, no `cmd/emu` test coverage.

---

## Q1 — Is `clear(rec)`'s justification still true after `Plate` grew an `id`?

**Yes. Verified, not assumed.**

The two `clear(rec)` sites are `gui/unlock_session.go:219` (`unlockEngraveCodex32`)
and `gui/unlock_session.go:327` (`unlockEngraveMnemonic`). The justification at
`:210-218` concludes *"nothing reads these bytes again"* — a statement about
`rec`'s bytes, not about `Plate`'s field list. Three facts, each read from source:

1. **`Plate.id` is assigned in exactly one place** — `gui/gui.go:2298-2299`, the
   loop at the end of `validateMdmk`. There is no other write in the tree
   (`grep -rn "\.id = " ` over `gui/` plus the full `validateMdmk`/`plateTextSeq`
   grep). It is unexported, so no other package can set it.
2. **Neither `clear(rec)` path calls `validateMdmk`.** `unlockEngraveCodex32`
   builds its plate through `backup.EngraveSeedString` → `toPlate`;
   `unlockEngraveMnemonic` through `engraveSeed` → `toPlate` (`gui/gui.go:634`).
   `toPlate` returns `Plate{Duration, Spline, Conf}` (`gui/gui.go:3236-3240`) and
   never touches `id`, so both plates carry `id == 0`.
3. **The value is a counter, not a function of `rec`.** `plateTextSeq`
   (`gui/gui.go:610`) is a monotone sequence number. Even on the paths where it
   *is* non-zero, it is derived from how many strings have been validated this
   uptime, not from any byte of the record.

Consequence: after `clear(rec)`, `NewEngraveScreen` (`gui/gui.go:2903-2909`) reads
`plate.Spline`, `plate.Duration`, `plate.Conf` and `plate.id == 0`. Nothing
record-derived survives the clear that did not survive it before. On completion
`notifyPlateEngraved(ctx.Platform, 0)` fires (`gui/gui.go:2952`), carrying zero
bits of the record — and in the firmware it is a no-op stub with no interface
behind it (see Q3).

Cross-check that the id can never *become* record-derived on those paths: ids start
at 1, because `plateTextSeq++` precedes the assignment (`gui/gui.go:2298`). So
`id 0` is never a `candidates` key on any consumer, and a seed/passphrase/free-text
plate can never be attributed to a string. `gui/engraved_hook_test.go:121-158`
(`TestAnUnannouncedPlateIsIgnored`) pins that, and it carries its own positive
control (`p.unknown != 1` and a following announced-id assertion), so it is not a
vacuous pass.

## Q2 — Does the diff add a live reference to seed material the §10.2.4 inventory misses?

**No.**

Everything this diff adds to `gui` that outlives a function call is three `uint64`s:

| new state | file:line | seed-derived? |
| --- | --- | --- |
| `Plate.id` | `gui/gui.go:600` | no — sequence number |
| `EngraveScreen.id` | `gui/gui.go:2921` | no — copy of the above |
| `plateTextSeq` | `gui/gui.go:610` | no — package-level counter |

The only allocation is `ids := make([]uint64, len(plates))` inside
`notifyPlateText` (`gui/engraved_hook.go:81-91`), which is transient, holds
`uint64`s, and lives in a file the firmware does not compile.

The `text string` never lands in a `gui` field. `notifyPlateText` passes it
straight to a consumer and returns; `notifyFrame` (`gui/frame_hook.go:84-88`) does
the same with the `op.Op`. `gui` retains neither.

**The audit's own platform does not implement either hook**, which I checked rather
than assumed. `gui/wipe_inventory_audit_test.go:111` drives `newHitPlatform()`
(`gui/run_reentry_test.go:81-97`), whose only methods are `AppendEvents`, `Dirty`
and `NextChunk`; it embeds `*deadlinePlatform` → `*testPlatform`, and neither
declares `Frame`, `PlateText` or `PlateEngraved`. A tree-wide grep for
implementations returns exactly three types — `cmd/emu.platform`
(`cmd/emu/platform.go:162,172,174`), `gui.frameAwarePlatform`
(`gui/frame_hook_test.go:24`) and `gui.engravedAwarePlatform`
(`gui/engraved_hook_test.go:30,36`). So in the audit the three `notify*` calls take
the `!ok` branch and are inert, and there is nothing new for the inventory to
enumerate.

Also checked, because a package-level counter invites it: there is no `t.Parallel()`
anywhere in `gui/`, and the GUI is single-goroutine on device and in the emulator,
so the unsynchronised `plateTextSeq++` is not a race.

## Q3 — Is the string really absent from the firmware, and only on `!tinygo` hooks?

**The absence is now proved mechanically, not argued.**

I built a probe module outside the repo (`replace seedhammer.com => <worktree>`)
containing only:

```go
package main

import "seedhammer.com/gui"

var _ gui.EngravedAware
var _ gui.FrameAware

func main() {}
```

Result:

```
--- host build (expect OK) ---
exit=0
--- tinygo-tagged build (expect FAIL: undefined) ---
./main.go:5:11: undefined: gui.EngravedAware
./main.go:6:11: undefined: gui.FrameAware
exit=1
```

An identifier that does not exist under `-tags tinygo` cannot be in the image. That
settles the interface half without needing to read the ELF.

Supporting: `go build -tags tinygo ./gui/...` exits 0, so no untagged file in the
package names either interface in code. (`go build -tags tinygo ./...` fails only on
`cmd/controller`'s `machine` import, which is a host-toolchain limitation, not a
finding; `go vet -tags tinygo ./gui/` fails only because `_test.go` files reference
`BuildPreview`, which is `!tinygo` — tests are never compiled for the device.)

`gui/tinygo_split_test.go` is a genuine strengthening of the guard it replaced: the
deleted `TestPlateHookIsAbsentFromTheFirmwareBuild` hard-coded `plate_hook.go` and
`PlateAware`; the replacement discovers every `_tinygo.go` pair from the tree, checks
both constraints, and — the load-bearing part — enforces the "declares an exported
interface" floor **per pair**, so moving one interface into an untagged file cannot
be masked by the other two. Nothing was lost in the swap.

**One precision on the claim as phrased in the brief.** "The string … never becomes
resident in the firmware" is stronger than what the code claims and is not true in
the absolute: the md1/mk1/ms1 string is a live local in `validateMdmk`, is copied
into `backup.Paragraph{Text: s}` (`gui/gui.go:2273-2275`), and is the QR payload.
That is all pre-existing and unchanged. What `gui/engraved_hook.go:26-33` actually
claims — *"the string never enters `Plate`"* — is true and is the claim that matters,
because `Plate` is the object that outlives `clear(rec)`. Confirmed at
`gui/gui.go:3236-3240`: `toPlate` keeps only the spline, dropping the
`engrave.Engraving` plan the text went into. Worth stating precisely so a future
reader does not inherit the stronger version as settled.

Also verified, since the design rationale leans on it: `gui.BuildPreview`
(`gui/preview.go:114-120`) really does only accept names from the `previewBuilders`
map, so nothing exported turns an arbitrary md1 back into a plate. The hook is not
redundant.

## Q4 — Same question for the frame hook

**Firmware carries nothing.** `FrameAware` is undefined under `-tags tinygo` (probe
above). The device compiles `gui/frame_hook_tinygo.go`'s
`func notifyFrame(Platform, op.Op) {}` with unnamed, unused parameters. The single
call site is `gui/run_flow.go:264`, inside `draw`, after the chunk loop and before
`onDraw`; `gui` retains nothing after it returns.

Placement is right for residency in a way worth noting as a *positive*: the wipe's
`FrameCallback` early-return (`gui/run_flow.go:~217-227`) means that once
`ctx.Done` is set, `yield` is never called again, so `draw` — and therefore
`notifyFrame` — never runs for the abandoned flow's post-wipe frames. The hook
cannot observe a frame the display did not get, and cannot observe one drawn after
the wipe decided to unwind.

The host-side consumer honours the "valid only for the call" contract:
`cmd/emu/screen.go:79-86` extracts synchronously with a fresh `op.Drawer` and keeps
the string; `gui/frame_hook_test.go:24-27` does the same. No retained `op`.

---

## Findings

### Minor 1 — the emulator now exposes an ms1 SECRET share as a plain string (firmware unaffected)

`gui/singlesig_engrave.go:22-28` puts `b.MS1` — a codex32 secret share — into a
`bundleCard`. `gui/bundle_flow.go:298` feeds every card string to `validateMdmk`,
so `notifyPlateText` (`gui/gui.go:2303`) receives that share. In `cmd/emu` it lands
in `engravedRecorder.candidates` (`cmd/emu/engraved.go:52-58`), a map documented as
never pruned and deliberately given no reset, and once the plate is cut and accepted
it reaches `shToolpath.strings()` as JSON readable from JavaScript
(`cmd/emu/toolpath_js.go:76-78`). It also survives a §10.2.4 idle wipe, which is
the one thing the wipe exists to make impossible on device.

**Failure scenario:** an operator runs a real full single-sig derive in the emulator
against a real seed. The ms1 share is then readable from the browser console for the
life of the page, including after the machine has "wiped".

**Why this is Minor and not Important:** (a) the firmware is provably unaffected —
`EngravedAware` does not exist there; (b) the exposure is not a new *class*. The
pre-existing `gui.PlateAware` hook already hands the emulator the plate's spline,
and `shToolpath.plan()` / `.svg()` already render it as glyph outlines
(`cmd/emu/plate.go:94` — "a glyph that looks thin here is thin on the metal"), so
the same share was already recoverable from the emulator with more effort; (c) the
trade is stated out loud in three places (`gui/engraved_hook.go`,
`gui/engraved_hook_tinygo.go:5-11`, `gui/singlesig_engrave.go:17-19`) rather than
assumed. Recorded so it is on the record, not because it blocks.

### Nit 1 — `unlock_session.go`'s clear-justification enumeration is now one field short

`gui/unlock_session.go:210-218` justifies the early `clear(rec)` by enumerating what
reads the plate afterwards: *"newEngraverJob holds plate.Spline … and the engrave
loop iterates e.spline, so nothing reads these bytes again."* After this diff,
`NewEngraveScreen` also reads `plate.id` (`gui/gui.go:2908`). The **conclusion is
still correct** — the sentence is scoped to `rec`'s bytes, and `id` is 0 on both
these paths — so this is not a defect. It is filed only because this project's own
standing lesson is that enumerated safety arguments go stale silently, and this is
an enumeration that just went one item out of date without being touched. A
four-word addition would keep it exact.

### Nit 2 — `Plate.id`'s own doc asserts "this struct carries only geometry"

`gui/gui.go:589-592` justifies the id by saying `unlock_session.go` clears early
"precisely because this struct carries only geometry". As of the same commit the
struct carries geometry *and* an id. The argument being made (a number carries
nothing, a string would) is sound and is the right argument; only the present-tense
phrasing is self-contradicting when read inside the field it is describing.

### Nit 3 — `EngravedAware.PlateText`'s ownership contract is silent where `FrameAware`'s is explicit

`gui/frame_hook.go:66-75` is emphatic that `content` is valid only for the duration
of the call. `gui/engraved_hook.go:59-63` says nothing about whether `ids` may be
retained. Today retention is safe — `notifyPlateText` allocates a fresh slice per
call (`gui/engraved_hook.go:86-89`) and `gui` never reads it back — so this is
harmless. But the asymmetry between two adjacent hooks invites a reader to carry
`FrameAware`'s stricter rule across, or to carry `EngravedAware`'s silence back.
One sentence would close it.

### Not findings (checked and cleared)

- **`validateMdmk`'s signature change from `engrave.Params` to `Platform`.** It now
  calls `pl.EngraverParams()` per invocation, where `bundleEngrave` and
  `multiPlateEngrave` previously hoisted it out of their loops. Both production
  implementations return a package-level value with no allocation and no side effect
  (`cmd/controller/platform_sh2.go:452-454`, `cmd/emu/platform.go:258`), as do both
  test ones. Behaviourally identical, no new per-iteration device allocation.
- **`plateTextSeq` surviving the §10.2.4 wipe and session restart.** Deliberate and
  correct: reuse across sessions is what would let a plate cut an hour ago claim a
  string validated since. It leaks only a count of validated strings for the
  uptime — no secret — and `uint64` cannot wrap here.
- **`notifyPlateEngraved` firing spuriously during a wipe.** It sits behind
  `for !ctx.Done` *and* `selectBtn.Clicked(ctx)` (`gui/gui.go:2944-2953`), so only a
  real accept reaches it. `gui/engraved_hook_test.go:161-232` pins the
  before-accept/after-accept boundary and documents a measured false-pass it had to
  defeat (pumping until `engraveDone` actually rendered), so that test is not
  vacuous.
- **`runSession`/`mustFinish` widening from `*deadlinePlatform` to `Platform`
  (`gui/run_harness_test.go:177,220`).** Test-harness-only; the wipe audit uses
  `runWithFlow` directly and is unaffected.

---

## Commands run

```
git log/diff 10286e4..HEAD                                  # scope
go build -tags tinygo ./gui/...                             # exit 0
go build -tags tinygo ./...                                 # only cmd/controller (machine pkg) — expected
go vet  -tags tinygo ./gui/                                  # only BuildPreview in _test.go — expected
probe module, host build                                     # exit 0
probe module, go build -tags tinygo                          # undefined: gui.EngravedAware / gui.FrameAware
grep sweeps: validateMdmk | notifyPlate* | plateTextSeq | EngravedAware | FrameAware
             | Plate{ | NewEngraveScreen | t.Parallel | EngraverParams | cardMS1
```

Per the brief I did not re-run `go test ./...`, `go vet ./...`, the wasm vet, the
TinyGo device build, the size measurements or the mutation checks — all stated as
settled.

Worktree left clean; the probe module lives outside both repos.
