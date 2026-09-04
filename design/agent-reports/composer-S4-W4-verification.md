# composer S4 — W-4 fold verification (targeted, independent)

**Scope:** one question only — are the digit pad's prompt and echo line now two
separate, fully visible lines on every pad, can the regression test fail, and
did nothing else move? Brief:
`design/agent-briefs/composer-S4-W4-verification-brief.md`. Fix under review:
fork branch `composer-s4d`, tip `bb5077547b422c47084850b119d772c4d9fcc28b` on
`6fb90cb` (worktree `/scratch/code/shibboleth/wt-composer-s4d`). Implementer's
report re-derived, not trusted: `design/agent-reports/composer-S4-W4-fix-report.md`.

**Method:** every mutation and every build ran in a `cp -r` copy
(`/scratch/code/shibboleth/.s4d-verify*`, deleted at the end of this session).
No `git checkout` was run in any copy. The real worktree and the main checkout
(which happened to already sit at `6fb90cb`, used as the base copy's source)
were both confirmed clean (`git status --porcelain` empty, exit 0) before and
after. Nothing committed. No sub-agents dispatched. No `.jsonl` file read.

## Item 1 — the hunks

**VERIFIED.**

```
$ git diff 6fb90cb..bb50775 --stat
 gui/composer_digitpad.go             |  83 ++++++++--
 gui/composer_digitpad_layout_test.go | 284 +++++++++++++++++++++++++++++++++++
 gui/composer_lock.go                 |  67 +++++----
 gui/composer_paged_geometry_test.go  |  20 +++
 4 files changed, 411 insertions(+), 43 deletions(-)
```

- `gui/composer_digitpad.go`: read in full. The per-line clamp
  (`if lim := top.Max.Y - sz.Y; y > lim { y = lim }`) is gone entirely; the
  box, the prompt and the echo are measured as one group (`groupHeight`),
  gaps collapse to zero if the group does not fit, and the whole group is
  centred (`y := top.Min.Y + (top.Dy()-groupHeight(...))/2`) before anything
  is placed. No cap, no Back, no keyboard-geometry code touched.
- `gui/composer_digitpad_geometry_test.go` (the S4 driver's `DIGIT_KEY_PITCH`
  pin): confirmed **byte-identical** to `6fb90cb`
  (`diff <(git show 6fb90cb:...) gui/composer_digitpad_geometry_test.go`
  produced no output). `TestWalkDigitPadCoordinatesTypeTheIntendedNumber`
  passes (below).
- `gui/composer_lock.go`: the anonymous date-validator closure was extracted
  to a named `composerDateBandEcho`, called through a one-line closure at the
  call site. Diffed the removed closure body against the new function body
  line by line: **identical control flow, identical branches, identical
  returned values**; only comment line-wrap changed (indentation dropped four
  tabs on extraction). `TestComposerEveryScreenFunctionHasAProductionCaller`
  exists in `gui/composer_join_test.go` and is the reason the by-value form
  could not be used directly — confirmed present. This is a behaviour-
  preserving refactor, not a validator change.
- `gui/composer_paged_geometry_test.go`: **not named in the fix report**, but
  present in the diff. It adds one shared helper, `rasterInk` (rasterises an
  `op.Op` into an ink-bitmap), used by the new layout test. Test-only,
  additive, no existing code in this file touched — does not trigger the
  "validator/cap/Back/keyboard geometry" Important clause. Worth recording
  since the report's own file list omitted it; not a defect.

No other file changed. **0 Important.**

## Item 2 — the geometry test: red / green / mutation, proven to run

**VERIFIED.**

- **Fix copy**, unmutated: `TestComposerDigitPadLinesNeverOverlap` — **12/12
  PASS**, all four pads (blocks/days/date/height) × empty/filled/ceiling as
  applicable. `TestComposerDigitPadGeometryProbeCanSeeOverlap` — PASS (own
  mutation proof: two strings at the same offset read short, stacked they
  don't).
- **Mutation, reproduced independently**: `composer_digitpad.go` overwritten
  byte-for-byte with `git show 6fb90cb:gui/composer_digitpad.go` in a fresh
  copy (`composer_lock.go`'s extraction and the new test files left in
  place — this is exactly the report's "restore both halves" / "6fb90cb's
  behaviour reconstructed" mutation). Confirmed the overwrite landed
  (`diff` against `6fb90cb`'s copy of the file: empty) before running.
- **Result: 12/12 FAIL**, `EXIT=1`. The per-subtest failure text matches the
  fix report's cited numbers **exactly**, byte for byte:
  - `blocks/empty`: "inks 17 row(s); … need 30" — matches.
  - `blocks/12960`: "inks 20 row(s); … need 37" — matches.
  - `days/90`: "inks 20 row(s); … need 37" — matches.
  - `blocks/99999-ceiling`: `bands=[(26,74)-(427,129)]`, 1 ink band — matches.
  This is not merely "the test failed" — the geometry numbers it reports
  are the ones a per-line-clamped layout produces, and they match the
  report's own transcript to the pixel, which is strong evidence the
  mutated line actually ran (a stale/cached binary, or a mutation that
  landed somewhere inert, would not reproduce these exact coordinates).
  `TestComposerDigitPadGeometryProbeCanSeeOverlap` still PASSED under the
  mutation, as expected (it is a probe of the row-counter, independent of
  the pad).
- Did **not** separately re-run the report's claimed-inert "clamp alone"
  mutation — the brief asks to prove *whichever* mutation is run actually
  ran, and the stronger mutation above is proven to have run by the exact
  match on its failure geometry, which is a higher bar than a generic
  "the suite went red" observation.

**0 Important.**

## Item 3 — longest echo per validator, through the real flow

**VERIFIED**, via the same test run (item 2) plus the item-5 emulator
capture below.

`TestComposerDigitPadLinesNeverOverlap` drives `composerDigitEntry` — the
real production widget — through simulated taps at the real
`DIGIT_KEY_PITCH`/`DIGIT_KEY_ROWS` coordinates, with the **real** validator
functions (`composerBlocksBandEcho`, `composerDaysBandEcho`,
`composerDateBandEcho`, `composerHeightBandEcho`), and its own table already
exercises the longest echo each validator can produce:
- blocks/days ceiling → `composerCopyRelativeCeiling()`, 86 chars, the
  longest message either validator returns (checked all of both functions'
  return points in `composer_copy.go`).
- date ceiling → `composerCopyDateCeiling()`, 89 chars, the longest of the
  four date-branch messages (floor 56, no-such-date 25, valid echo 21).
  All PASS in the fix, all FAIL under the mutation above.

Additionally drove the real device flow live (item 5) and typed `99999` into
the blocks pad on the operator's own route — the ceiling message wraps to
two lines and is the tightest fit reported (worst-case group height 74 px
in an 86 px band); screenshot below shows it fits cleanly with no overlap or
clipping.

**0 Important.**

## Item 4 — gates

**VERIFIED**, all reproduced independently in the copy:

| gate | result |
| --- | --- |
| `gofmt -l gui/` | exit 0; only `transaction.go`, `transaction_golden_test.go`, `transaction_txrecord_test.go` — pre-existing, outside this diff |
| `go vet ./gui/` | exit 1; only the 2 pre-existing `testing.ArtifactDir` (go1.26) findings |
| `go test -run WalkDigitPadCoordinates ./gui/` | PASS — the S4 driver's pin is unchanged and still typing correctly |
| `go test -count=1 ./cmd/emu/` | `ok seedhammer.com/cmd/emu 1.785s` |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` | exit 0 |
| `gui-shard-test.sh ./gui/ 24` | `partition verified exhaustive: 1192 == 1192`; all 24 shards `ok`; wall 44s |

**Test count, measured independently, not taken from the report or the
brief:** `go test -mod=readonly ./gui/ -list '.*' \| grep -E '^(Test\|Example\|Fuzz)'`
— base `6fb90cb` (copied from the main checkout, which was already sitting
at that exact revision): **1190**. Tip `bb50775`: **1192**. The two new
names are exactly `TestComposerDigitPadLinesNeverOverlap` and
`TestComposerDigitPadGeometryProbeCanSeeOverlap`; no other test name changed.
This matches the fix report's "1192" claim exactly.

Note for the record: the verification brief's own text cites a baseline of
**1189**; the measured baseline is **1190**. This is a discrepancy in the
brief's stated expectation, not in the fix — the fix report's own claim
(1192 total) and my independent measurement agree exactly, and the
shard-script's exhaustiveness assertion (which refuses to run on an
incomplete partition) passed. Not a fold defect; recorded for the next
brief author.

**Firmware size, measured independently (both endpoints, not the report's
numbers):**

```
base copy   (6fb90cb, matches the main checkout exactly): 1,580,580 B flash / 62,800 B RAM
tip copy    (bb50775, composer-s4d):                       1,581,204 B flash / 62,800 B RAM
```

Base measurement matches the brief's cited pin (`1,580,580 / 62,800`)
**exactly**. Delta: **+624 B flash, 0 B RAM** — matches the fix report's
claim exactly.

**0 Important.**

## Item 5 — the emulator proof

**VERIFIED**, on two independent legs (screenshots not reused from the fix
report):

**5a. `capture_composer.py --arm both` against the copy.** Built `emu.wasm`
from `/scratch/code/shibboleth/.s4d-verify/cmd/emu`, ran
`python3 design/journeys/capture_composer.py --arm both --emu .../cmd/emu`:
**exit 0**, `all legs matched the host` (keyed-A 62s, keyed-B 75s, keyless
28s; Template-ID, Policy-ID, all 4 addresses and every engraved md1/mk1
string matched the host byte for byte).

**5b. The operator's own route, driven fresh.** Wrote an independent
Playwright driver (`/scratch/code/shibboleth/.tmp/w4review/drive_route.py`,
not derived from the fix report's script) against the copy's `emu.wasm`,
using only the emulator's public JS API (`window.shTap`, `shTargets`,
`shWaitFor`, `shScreen`) and the menu structure read directly from
`gui/composer_shape.go`, `composer_presets.go`, `composer_lock.go` (row
indices: door row 1 "Build a new policy", wrapper row 0 "Taproot (tr)",
preset row 6 "decaying-multisig", path-list row 0 "Path 1", path-menu row 1
"Time lock", kind row 1 "After a wait" / row 2 "After a date or height",
unit row 0 "Blocks", abs row 0 "A date"). Drove: **SKIP → Wallet Policy →
Build a new policy → Taproot → decaying-multisig → Path 1 → Time lock →
After a wait → Blocks**, and separately **… → After a date or height → A
date**, confirming at each step via `shWaitFor` on the real screen text (a
case-typo on the first attempt caused one legitimate `shWaitFor` timeout,
which also independently confirms the route was exactly right — the screen
at that point already read `"Whatkindoftimelock?NoneAfterawaitAfteradateorheightPath1lock"`).

Six screenshots captured and **looked at** (not just asserted against
`shScreen()`, which cannot see an overprint — the W-3/W-4 lesson):

- `blocks-pad-empty-INDEP.png` — box empty; "How many blocks?" and "1 to
  65535 blocks" as two separate, fully legible lines; nothing under the
  keyboard or the nav column.
- `blocks-pad-12960-INDEP.png` — box "12960"; "How many blocks?" and "12960
  blocks (about 90.0 days)", two lines, confirm tick visible.
- `blocks-pad-99999-ceiling-INDEP.png` — box "99999"; the §8u ceiling
  message wraps to two lines ("Relative locks reach at most 455 days in
  blocks or 388 days in time." / "Use an absolute date."), the tightest
  case in the fix report's own measurements (74 of 86 px) — fits cleanly,
  touches nothing.
- `date-pad-empty-INDEP.png` — "Date as YYYYMMDD" / "eight digits,
  YYYYMMDD", two lines.
- `date-pad-20260901-INDEP.png` — box "20260901"; "Date as YYYYMMDD" /
  "2026-09-01 00:00 UTC", two lines, confirm tick visible.

All six: two legible lines under the box, none under the keyboard or the
buttons — no overlap, no clipping, on the empty state, a typed value, and
the wrapped ceiling case.

**0 Important.**

## Closing counts

**5 of 5 items VERIFIED. 0 Critical / 0 Important.** No hunk outside W-4's
scope changes the validator, the cap, Back, or keyboard geometry; the
geometry test is red on `6fb90cb`'s reconstructed behaviour, green on the
fix, and its failure geometry under mutation matches the fix report's own
transcript to the pixel (mutation proven to run); every gate reproduces
independently, including a firmware delta (+624 B flash, 0 RAM) that matches
exactly; and the operator's own route, driven from scratch and looked at as
PNGs, shows two separate legible lines on every pad in every state tested,
including the tightest (wrapped ceiling) case. One non-blocking note: the
verification brief's cited baseline test count (1189) does not match the
measured baseline (1190) — immaterial to the fold, flagged for the brief
author.
