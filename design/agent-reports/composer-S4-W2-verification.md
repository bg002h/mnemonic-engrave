# S4 walk W-2 — independent fold-verification report

**Reviewer:** independent targeted verification (sonnet), per
`design/agent-briefs/composer-S4-W2-verification-brief.md`. Not the
implementer. No sub-agents dispatched, no `.jsonl` read.

**Target:** fork branch `composer-s4b`, tip
`2dff0ee2cf1824b0381e37fa7a1fccb739393157` on base `60bee002`, worktree
`/scratch/code/shibboleth/wt-composer-s4b`. Implementer's report:
`design/agent-reports/composer-S4-W2-fix-report.md` — treated as a claim to
re-derive, not a source of truth. Every number below was independently
measured in throwaway `cp -r` copies (`/scratch/code/shibboleth/.s4b-verify/`,
removed after use) or a throwaway `git worktree add --detach` at `60bee002`
(also removed after use). The original `wt-composer-s4b` worktree was never
written to: confirmed clean (`git status --porcelain` empty, `HEAD` still
`2dff0ee2cf1824b0381e37fa7a1fccb739393157`) after every step below.

**One-line answer:** yes — a tap on any visible row of `composerPickScreen`
selects that row on the machine's own input path (PointerEvents through the
real wasm build), the regression test fails on `60bee002` and under mutation,
and nothing outside the declared diff moved.

## Item 1 — the hunks

`git diff 60bee002..2dff0ee2c... --stat`, reproduced:

```
 gui/composer_measure_test.go    |   2 +-
 gui/composer_paged.go           | 109 ++++++++++++++++++++---
 gui/composer_paged_test.go      |   4 +-
 gui/composer_pick_touch_test.go | 185 ++++++++++++++++++++++++++++++++++++++++
 gui/composer_stub_test.go       |   4 +-
 5 files changed, 288 insertions(+), 16 deletions(-)
```

Matches the report byte-for-byte. Read every hunk in full (not just the stat):

- `composer_paged.go`: `composerPageLines` gains a third return (`[]image.Rectangle`,
  the per-row touch band) computed from the SAME layout loop that already
  produced `body`/`shown` — no second measurement site. `composerReadScreen`
  discards the new return (`_`) — no cursor, no hit area, unchanged behaviour.
  `composerPickScreen` adds `rowHits [composerPickScreenMaxRows]Clickable`
  declared outside the frame loop, one `op.Input(&ctx.B, &rowHits[j]).Clip(bands[b])`
  per row within `shown` (not `len(bands)`), and `rowHits[j].Clicked(ctx)` sets
  `sel`. **Up/Down, `Button2` paging, and `takeBtn`/`backBtn` (`Button3`/`Button1`)
  are byte-for-byte unchanged** — confirmed by reading the diff hunk boundaries;
  nothing inside those blocks is touched.
- `composer_measure_test.go`, `composer_paged_test.go` (×2),
  `composer_stub_test.go` (×2): each hunk is `_, shown := composerPageLines(...)`
  → `_, shown, _ := composerPageLines(...)`. No assertion in any of the six
  changed lines differs — confirmed by reading each hunk; only the third
  return's blank discard was added.
- `composer_pick_touch_test.go`: new file, read in full (185 lines) —
  see item 2.

**VERIFIED.** No hunk changes what a page draws, the wrap rule, or take
semantics. No production file outside `composer_paged.go` changed.

## Item 2 — the touch-path test: RED on 60bee002, GREEN on the fix, RED under mutation

Drives the real flow (`walletPolicyFlow → composerFlow`) through
`runUITouch` + `tap`, measuring row points from `composerPageLines`'s own
layout via `composerPickRowPoint` (not constants) into a private `op.Buffer`.

**RED on 60bee002** (throwaway `git worktree add --detach` at `60bee002`, the
new test file copied in):

```
--- FAIL: TestComposerPickScreenRowsAreTouchable (0.00s)
    composer_pick_touch_test.go:143: tapping the `3` row did not select it: the threshold picker offers
        1..n for the n just taken, and this frame is not 1 2 3.
        Frame: "ThresholdPath1:howmanymustsign?1"
        composerPickScreen's rows are unreachable by touch, which is the only input SeedHammer II has (W-2).
FAIL
```

Identical to the report's claimed RED, independently reproduced.

**GREEN on the fix** (`cp -r` copy):

```
--- PASS: TestComposerPickScreenRowsAreTouchable (0.01s)
PASS
ok  	seedhammer.com/gui	0.008s
```

**Mutation** (in the `cp -r` copy: `op.Input(&ctx.B, &rowHits[j]).Clip(bands[b])`
replaced with `_ = bands[b]`, keeping the `Clicked` poll — isolates the hit
area alone):

```
--- FAIL: TestComposerPickScreenRowsAreTouchable (0.00s)
    composer_pick_touch_test.go:143: tapping the `3` row did not select it: ...
        Frame: "ThresholdPath1:howmanymustsign?1"
FAIL
```

Confirmed the mutation actually ran before restoring: `grep -c VERIFY-MUTATION
gui/composer_paged.go` was `1` under the mutation, `0` after `cp` restore, and
`diff` against the original fix worktree's `composer_paged.go` was empty
(byte-identical) after restore. Re-ran GREEN post-restore: `ok`.

**VERIFIED.**

## Item 3 — all four call sites

The implementer's own test drives two of the four (the key-count picker,
`composerCountPick`, and the Spend paths list's `Done`) through the real flow.
Its report calls the other two — `Which hash?` (`composer_hash.go:149`) and
`Seat keys` (`composer_seat.go:127`) — "fixed by construction, not separately
tested." Per the brief, I wrote throwaway tests in the `cp -r` copy for both
(deleted with the copy afterward; never committed):

- **`Which hash?`, row 2 (`Type 64 hex`).** Built a session with one
  `hash:` payload record so the rows are `["hash 1 ...", "Type 64 hex", "No
  hash lock"]`; drove `composerHashEdit` directly through `runUITouch`, tapped
  row index 1 (`Type 64 hex`), took it, dismissed the §8i modal that fires on
  that arm, and asserted the hex-entry keyboard (`"0 of 64 hex"`) drew.
  **PASS on the fix.** Re-run against pristine `60bee002` (test file copied
  into the throwaway worktree): **FAILS** — the hex-entry screen never draws,
  confirming the test measures the real defect, not a tautology.
- **`Seat keys`, row 2 (second source).** Built two unused key sources with
  distinct labels/fingerprints; drove `composerSeatFlow` directly, tapped row
  index 1 (source B), took it, and asserted seating advanced to slot 1's
  prompt with slot 0 holding source B's index and fingerprint (`bbbb2222...`).
  **PASS on the fix.** Re-run against pristine `60bee002`: **FAILS** — the
  frame after tapping row 1 + take shows slot 1's prompt offering source B
  still *unused*, i.e. row 0 (source A) was taken regardless of where the tap
  landed — the exact W-2 shape at this call site.

One test-authoring mistake surfaced and was fixed during this pass, noted for
transparency: my first `Seat keys` draft checked `st.assigned[0].src`
immediately after `tap()`+`click(Button3)` with no intervening `frame()` pull,
which reads state before the coroutine has consumed the queued events (a
harness bug, not a fix defect — `composerPickScreen`'s `takeBtn.Clicked(ctx)`
check runs before the per-row loop within one resumed iteration, so the
consuming call has to be a `pumpUntil` that actually advances a frame). Fixed
by asserting on the post-advance frame instead of raw state, as shown above.

**VERIFIED — all four call sites** now select on any tapped row, not only the
first.

## Item 4 — gates

All run independently (fresh worktree/copy, `go clean -testcache` where
noted), `/scratch/code/shibboleth/.toolchain/go/bin` on `PATH`,
`CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local -mod=readonly`,
`TMPDIR=/scratch/code/shibboleth/.tmp`.

| gate | 60bee002 (measured) | fix (measured) | match report? |
| --- | --- | --- | --- |
| `gofmt -l gui/` | `transaction.go`, `transaction_golden_test.go`, `transaction_txrecord_test.go` | identical 3 | yes |
| `go vet ./gui/` | `freetext_sizeproof_golden_test.go:111`, `transaction_golden_test.go:104` (both `testing.ArtifactDir` pre-Go-1.26) | identical 2 | yes |
| `go test -run '^TestComposer' ./gui/` | — | `ok` 5.208s | yes |
| `scripts/gui-shard-test.sh ./gui/ 24` | — | `RESULT: ok -- all 1187 tests ran across 24 shards`, wall 69s | yes (1186+1) |
| `go test -count=1 ./cmd/emu/` | — | `ok` 1.477s | yes |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` | — | exit 0 | yes |
| `scripts/test-32bit.sh` (GOARCH=386 test + GOARCH=arm build) | — | both exit 0, re-run with `-count=1` uncached | **not in implementer's report — independently run per this brief** |
| `go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/` | — | `ok` all three, `[no tests to run]`, re-run after `go clean -testcache` | **not in implementer's report — independently run per this brief** |

The two gates the brief names that the implementer's report omitted
(`test-32bit.sh`, the `oraclelive` build) were run here and are green.

**Firmware size** (`nix develop -c tinygo build -size short -o /dev/null
-target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks
./cmd/controller`, `/nix/var/nix/profiles/default/bin` on `PATH`):

| commit | flash | RAM |
| --- | --- | --- |
| `60bee002` (measured fresh, not trusted from the brief) | 1,579,940 B | 62,800 B |
| `2dff0ee2c...` (the fix) | 1,580,580 B | 62,800 B |
| **delta** | **+640 B** | **+0 B** |

The `60bee002` figure matches the brief's cited baseline exactly (independent
recomputation, not copied).

**VERIFIED.**

## Item 5 — the emulator proof, reproduced on the machine's own input path

Built `emu.wasm` fresh from the `cp -r` fix copy: **10,792,051 bytes**,
matching the report's cited size exactly (independent build, not compared to
a cached artifact). Served on a fresh local port (`127.0.0.1:18453`), driven
with Python Playwright (`sync_playwright`, headless Chromium) via
`window.shTap`/`window.shWaitFor`/`window.shScreen` — the same JS driving API
`cmd/emu/walk_js.go` and `index.html` expose, i.e. the same event path
(`gui.PointerEvent` through the canvas listener) the ft6x36 panel uses in
firmware.

The three row-tap coordinates were **independently recomputed**, not copied
from the report: a throwaway Go test (`verify_geom_test.go`, deleted after
use) called the same `composerPickRowPoint` helper the regression test uses
and printed `(240,179)`, `(240,150)`, `(240,208)` for the `3` row / `2` row /
`Done` row respectively — an exact match to the report's cited points,
confirming the geometry is deterministic and reproducible rather than
eyeballed.

Ran the walk against the fix build:

```json
{
  "door": "Nokeysloaded.Thisbuildsakey-lesstemplate.ScancardsBuildanewpolicyWalletPolicy",
  "n picker": "KeysPath1:howmanykeys?12345",
  "after tapping the `3` row": "ThresholdPath1:howmanymustsign?123",
  "after tapping the `2` row": "Spendpathsslots:3Path1:2-of-3AddaspendpathChangethescriptDone",
  "after tapping `Done`": "Sortedkeys,oryourorder?Sorted(usual)KeepmyorderKeyorder"
}
```

As a paired control (per the brief's own methodology), built `emu.wasm` fresh
from a throwaway `60bee002` worktree, served on a second port
(`127.0.0.1:18454`), and ran the identical script against it:

```json
{
  "door": "Nokeysloaded.Thisbuildsakey-lesstemplate.ScancardsBuildanewpolicyWalletPolicy",
  "n picker": "KeysPath1:howmanykeys?12345",
  "after tapping the `3` row": "ThresholdPath1:howmanymustsign?1",
  "after tapping the `2` row": "Spendpathsslots:1Path1:1keyAddaspendpathChangethescriptDone",
  "after tapping `Done`": "Path1:1keyKeysTimelockHashlockRemovepathPath1"
}
```

Both `door` and `n picker` frames are identical between builds (as expected —
neither is touched by this fix). Both the "after" frames and the "before"
frames match the implementer's report table exactly, independently
reproduced end to end: on `60bee002` the `n`-picker tap never moves `n` past
1, and `Done` opens the path editor (row 0) instead of reaching the key-order
question; on the fix, `n` becomes 3, the threshold read 2-of-3, and `Done`
reaches "Sorted keys, or your order?".

**VERIFIED.**

## Cleanup / read-only compliance

All mutation and extra-test work happened in `cp -r` copies under
`/scratch/code/shibboleth/.s4b-verify/`, plus one `git worktree add --detach`
at `60bee002` for the RED baseline and the paired emulator control — both
removed (`git worktree remove --force`, `rm -rf`) after use. No `git
checkout` was ever run inside a copied worktree. Nothing was committed. The
original `wt-composer-s4b` worktree's `git status --porcelain` was empty and
`HEAD` was still `2dff0ee2cf1824b0381e37fa7a1fccb739393157` at the end of
every verification pass.

## Closing counts

**0 Critical / 0 Important / 0 Minor / 0 Nit.**

All five verification items VERIFIED independently, with every claim in the
implementer's report re-derived from scratch rather than trusted — including
two gates (`scripts/test-32bit.sh`, the `oraclelive` build) the report's own
gate table omitted, both of which are green. The fix closes S4 walk W-2 as
built: all four `composerPickScreen` call sites are touch-selectable on any
visible row, Up/Down/Button2/Button3/Button1 semantics are unchanged, the
regression test is a real (non-tautological) guard, and the firmware size
delta is +640 B flash / +0 B RAM.
