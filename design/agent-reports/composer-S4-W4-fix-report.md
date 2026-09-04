# composer S4 — W-4 fix report: the digit pad's prompt and echo drawn over each other

**Defect:** W-4 in `design/S4_journey_walk_2026-09-02.md`, found by the operator
on the device. **Brief:** `design/agent-briefs/composer-S4-W4-fix-brief.md`.
**Implementer:** the same single opus agent, resumed.

**Outcome: DONE.** The decided fix shape was executed. The geometry test fails
12 of 12 on `6fb90cb` naming the overlapping pair and passes after; every gate is
green; and the emulator shows the prompt and the range/echo as two separate
legible lines on the operator's own route.

## Worktree

| repo | path | branch | base |
| --- | --- | --- | --- |
| seedhammer fork | `/scratch/code/shibboleth/wt-composer-s4d` | `composer-s4d` | `main` `6fb90cb` |

```
$ git -C /scratch/code/shibboleth/wt-composer-s4d log --oneline main..HEAD
bb50775 gui: the digit pad's box, prompt and echo are one centred group (S4 walk W-4)
```

Tree clean (`git status --porcelain` empty, exit 0). Nothing pushed, nothing
flashed, no sub-agent, no `.jsonl` read. Every mutation and every comparison
build ran in a `cp -r` copy under `/scratch/code/shibboleth/.tmp/` with its
`.git` link removed. `wt-engrave-s4-emu` no longer exists (merged), so the
capture below ran from a staged copy of the main checkout's
`design/journeys/` — the main checkout was not written to.

## The defect

`composerDigitEntry` placed the entry box on its own (`top.Center(frgSize)`) and
then hung the info lines below it, clamping **each line individually** to
`top.Max.Y - sz.Y`. A second line that did not fit was pushed **up** onto the
first. A clamp cannot be applied to one member of a group.

Measured on the harness before the fix — all four pads, empty and filled:

```
blocks/empty      box [y74..97]   ONE info band [y109..126]   <- prompt + echo
blocks/12960      box [y74..97]   ONE info band [y109..129]
days/90           box [y74..97]   ONE info band [y109..129]
date/20260901     box [y74..97]   ONE info band [y109..124]
height/905000     box [y74..97]   ONE info band [y109..126]
blocks/99999      ONE band [y74..129] — the box AND both lines, a single blob
days/999          ONE band [y74..129]
date/20990101     ONE band [y74..129]
```

The three **ceiling** messages wrap to two lines and merged with the entry box
as well.

## The fix

**`gui/composer_digitpad.go`.** Nothing is placed until everything is measured:
the box, the prompt and the echo are laid out as one vertical group whose height
is summed first, and the **group** is centred in the band above the keyboard.
Gaps collapse to zero before a line moves.

### The measurements the fix rests on (SH2 display, body style)

| quantity | value |
| --- | --- |
| keyboard | 100 × **182** |
| content band (`content`) | y 44 … 312 |
| band above the keyboard (`top`) | y 44 … 130, **86 px** |
| box drawn height (`frgSize.Y + 3 + buttonPadY`) | **24 px** |
| one info line | 17–18 px |
| a wrapped ceiling message | ~33–35 px |
| **group, normal case** (box + 8 + prompt + 4 + echo) | **70 px** — fits 86 |
| **group, ceiling case, gaps kept** | **~89 px** — does **not** fit |
| **group, ceiling case, gaps zeroed** | **74 px** — fits |

So the gaps are what give, and the "still does not fit" branch the brief asked
me to report is **unreached**: no pad needs a line dropped or overlapped. The
lowest text row on any pad after the fix is **y129** against a band ending at
**130** — the ceiling messages touch the keyboard's first row without entering
it, which the test asserts explicitly.

## The test — RED on `6fb90cb` first

**`gui/composer_digitpad_layout_test.go`** (new). It renders all four pads
through the real widget with their **real validators**, empty and filled — 12
subtests — and rasterises the frame, because a text assertion cannot see this
defect: `ExtractText` collects a glyph's rune wherever it lands, so the pad
reported both strings while showing neither. **`shScreen()` is byte-identical
before and after this commit.**

**How overlap is detected**, without counting bands (which wrapping makes
unstable): each info string is rendered **alone** at the same style and wrap
width, and its ink rows counted; the frame's info ink must occupy at least their
sum. Two lines on top of one another occupy fewer rows than the two of them
need — and nothing else can cause that.

```
$ go test -count=1 -run 'ComposerDigitPadLinesNeverOverlap|…Probe' ./gui/
W-4 TEST ON 6fb90cb EXIT=1            12 of 12 subtests FAIL

  blocks/empty: the prompt and the echo are drawn OVER EACH OTHER.
    below the entry box the frame inks 17 row(s); rendered alone the two lines need 30
    prompt "How many blocks?", echo "1 to 65535 blocks"
    ink bands in the band above the keyboard: [(184,74)-(296,97) (167,109)-(313,126)]

  blocks/12960: … inks 20 row(s); … need 37
    prompt "How many blocks?", echo "12960 blocks (about 90.0 days)"

  days/90: … inks 20 row(s); … need 37
    prompt "How many days?", echo "90 days = 15188 units of 512 s (90.0 days)"

  blocks/99999-ceiling: the band above the keyboard holds 1 ink band(s); the
    entry box and the info lines cannot all be there.  bands=[(26,74)-(427,129)]
```

**After the fix**, all 12 pass, together with the three pre-existing digit-pad
tests:

```
--- PASS: TestComposerDigitPadLinesNeverOverlap (0.66s)      [12 subtests]
--- PASS: TestComposerDigitPadGeometryProbeCanSeeOverlap
--- PASS: TestComposerDigitPadTypesOnlyDigits
--- PASS: TestComposerDigitPadDrawsItsEchoAndGatesTheConfirm
--- PASS: TestComposerDigitPadBackLeavesWithNothing
ok  seedhammer.com/gui  0.694s        EXIT=0
```

`TestComposerDigitPadGeometryProbeCanSeeOverlap` is the probe's own mutation
proof: two strings at the **same** offset must come out short, the same two
**stacked** must not. It passes on the base too, as a probe independent of the
pad should.

### The mutations — and the first is a finding

**Restoring the per-line clamp alone does NOT fail** (`EXIT=0`). Once the group
is centred the clamp is **unreachable**: no line ever exceeds `top.Max.Y - sz.Y`,
because the group was fitted before anything was placed. The clamp was only ever
reachable because the box had been placed without the lines. That is worth
recording — the brief expected this mutation to fail, and it is inert.

**Restoring both halves** — the clamp *and* the independent box centring, which
is `6fb90cb`'s behaviour reconstructed — fails **12 of 12** with the same
message:

```
CLAMP + INDEPENDENT BOX CENTRING EXIT=1        12 subtests FAIL
  the prompt and the echo are drawn OVER EACH OTHER.
    below the entry box the frame inks 17 row(s); rendered alone the two lines need 30
    prompt "How many blocks?", echo "1 to 65535 blocks"
```

So the gate binds to the **group layout**, which is the thing that was wrong.

## `composerDateBandEcho`, extracted

The date pad's validator was an anonymous closure inside `composerLockEdit`, so
the test could not use the real one. It is now named beside its three siblings —
for the reason their shared header already gives: *as closures they had no caller
a test could reach, so widening one left the suite green.* It is invoked through
a closure like the other three, because
`TestComposerEveryScreenFunctionHasAProductionCaller` counts **calls**, not
values, and caught the by-value form immediately:

```
these composer functions have no production caller, so the screens they draw
cannot be reached by any operator: [composerDateBandEcho]
```

A guard in this tree doing exactly its job.

## Gates

| gate | exit | detail |
| --- | --- | --- |
| `gofmt -l gui/` | 0 | only `transaction.go`, `transaction_golden_test.go`, `transaction_txrecord_test.go` — unformatted at `60bee002` already, outside this diff, untouched |
| `go vet ./gui/` | 1 | only the two pre-existing `testing.ArtifactDir` findings |
| `go test -count=1 -run '^TestComposer' ./gui/` | 0 | `ok  seedhammer.com/gui  5.012s` |
| `gui-shard-test.sh ./gui/ 24` | 0 | `all 1192 tests ran across 24 shards`, wall 47s |
| `CGO_ENABLED=0 go test -count=1 ./cmd/emu/` | 0 | `ok  seedhammer.com/cmd/emu  1.701s` |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` | 0 | |
| `go test -run WalkDigitPadCoordinates ./gui/` | 0 | the shipped `DIGIT_KEY_PITCH` geometry the S4 driver types by is **unchanged** (`git diff` on that test file is empty) |
| `capture_composer.py --arm both` | 0 | three legs, `all legs matched the host` — the walk types through this pad |
| firmware size recipe | 0 | **1,581,204 B flash / 62,800 B RAM** — see below |

### Firmware size — a finding, attributed by measurement

The pin is `1,580,580 B flash / 62,800 B RAM`; this branch measures
**1,581,204**, i.e. **+624 B flash, no RAM**. Three builds, not an argument:

```
pristine 6fb90cb sources          1,580,580 B     (the pin, reproduced here)
+ the date-echo extraction only   1,580,596 B     +16 B
+ the group layout                1,581,204 B     +608 B
```

The layout change is the cost: measuring a group before placing it is more code
than clamping each line as it goes. Reported rather than absorbed; the pin wants
updating to `1,581,204 / 62,800` if this merges.

## The emulator proof — the operator's own route, looked at

Built from this worktree; driven with Playwright and `shTargets()` through
exactly the route the operator took: **no payload (SKIP) → Wallet Policy → Build
a new policy → Taproot → `decaying-multisig` → Path 1 → Time lock → After a wait
→ Blocks**, then the date pad via **After a date or height → A date**. The same
route was run against a `6fb90cb` build for the comparison.

```
/scratch/code/shibboleth/.tmp/w4-shots/blocks-pad-empty-BEFORE-6fb90cb.png
/scratch/code/shibboleth/.tmp/w4-shots/blocks-pad-empty-after-W4.png
/scratch/code/shibboleth/.tmp/w4-shots/blocks-pad-12960-BEFORE-6fb90cb.png
/scratch/code/shibboleth/.tmp/w4-shots/blocks-pad-12960-after-W4.png
/scratch/code/shibboleth/.tmp/w4-shots/date-pad-empty-BEFORE-6fb90cb.png
/scratch/code/shibboleth/.tmp/w4-shots/date-pad-empty-after-W4.png
```

The route reproduced the operator's screens exactly:

```
presets:   "Startfrom?Buildmyownpaths…hashlock-gateddecaying-multisigNewpolicy"
path list: "Spendpathsslots:4Path1:2-of-2+13140blocksPath2:1key+26280blocks
            Path3:1key+block1000000AddaspendpathChangethescript"
```

**BEFORE** — `blocks-pad-empty` renders "How many blocks?" and "1 to 65535
blocks" through each other as one illegible line (it reads `Hqw ossry blocks?`).
**AFTER** — two separate legible lines under the box, on all three shots:

| shot | what I see |
| --- | --- |
| `blocks-pad-empty-after-W4.png` | box, then `How many blocks?`, then `1 to 65535 blocks` |
| `blocks-pad-12960-after-W4.png` | box holding `12960`, then `How many blocks?`, then `12960 blocks (about 90.0 days)`; the confirm tick is drawn |
| `date-pad-empty-after-W4.png` | box, then `Date as YYYYMMDD`, then `eight digits, YYYYMMDD` |

**And the point of the finding, in one line:** `shScreen()` is *identical* on both
builds —

```
"1234567890Howmanyblocks?1to65535blocksPath1lock"
"123456789012960Howmanyblocks?12960blocks(about90.0days)Path1lock"
"1234567890DateasYYYYMMDDeightdigits,YYYYMMDDPath1lock"
```

— which is why every automated walk passed while the operator could read
neither line. Same lesson as W-3, one screen over.

## What I decided, and what I could not do

1. **`composerDateBandEcho` was extracted** so the test could use the real
   validator rather than a copy. It is a behaviour-preserving move that matches
   the three siblings' existing pattern, and the call-site guard forced the
   closure form.
2. **The "group still does not fit" branch is unreached**, measured: the worst
   case is 74 px in an 86 px band once gaps collapse. The code top-aligns rather
   than dropping or overlapping a line if it ever were reached, so the failure
   would be visible rather than hidden.
3. **The clamp-alone mutation is inert** — reported above rather than presented
   as a passing mutation test, because a mutation that cannot fail proves
   nothing and saying so is the point.
4. **The firmware pin moves** (+624 B). Not absorbed, not argued: three builds
   attribute it.
5. **Not done, and not mine:** merging, pushing, flashing, and the remainder of
   the walk (W-5 is already filed on the record; this branch does not touch it).
