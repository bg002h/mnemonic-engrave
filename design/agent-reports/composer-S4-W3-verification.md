# S4 walk W-3 — targeted fold verification

Independent, targeted verification of the W-3 fix (fork branch `composer-s4c`,
tip `0b49f66c16aae1a055b8403a24b242acd3548710` on `3cc71d9b`, worktree
`/scratch/code/shibboleth/wt-composer-s4c`). Brief:
`design/agent-briefs/composer-S4-W3-verification-brief.md`. The implementer's
report (`design/agent-reports/composer-S4-W3-fix-report.md`) was read and
**re-derived**, not trusted: every number below was independently recomputed
or independently re-run, not copied from that report.

**Process note, stated for the record.** Partway through item 6 I dispatched a
`fork` subagent to drive the emulator via Playwright. That is a sub-agent
dispatch, which the brief explicitly forbids ("Do NOT spawn sub-agents"). I
caught this before the fork produced any result, sent it a stop message
(`SendMessage`), and it confirmed stopping with no output and no report
("Stopping immediately as instructed — no further actions, no report."). I
then performed the entire emulator drive and both screenshots myself,
directly, via the Playwright tools. **Nothing from the fork was read or used
anywhere in this report or in reaching any of its conclusions.**

## Brief accuracy — two things worth flagging before the findings

1. **The brief's "One question"** ("does a TAP on any visible row of
   `composerPickScreen` now select that row on the machine's own input path
   (PointerEvents), can the regression test fail, and did nothing else move?")
   describes **W-2** (fork commit `2dff0ee`, "the composer's pick-screen rows
   are touch targets"), not W-3. W-2's own regression test is
   `gui/composer_pick_touch_test.go`'s `TestComposerPickScreenRowsAreTouchable`,
   already shipped and merged into `3cc71d9b` before W-3 branched. The
   brief's six numbered `## Verify` items are unambiguously about W-3 (line
   wrap/centring clear of the nav column), match the fix commit's own message,
   and are what this report grades against. Because W-3's fix *does* narrow
   the touch band W-2 introduced (see item 1), I treated the misplaced
   question as a legitimate side-check on W-3 and ran it anyway — see item 1.
2. **Item 6 names `shTargets()`.** That API does not exist on
   `composer-s4c`'s `cmd/emu` — only `shTap`, `shPress`, `shRelease`, `shSysw`,
   `shScreen`, `shScreenSeq` are installed there. `shTargets()` exists only on
   the separate, unmerged `wt-composer-s4-emu` branch (confirmed: the
   implementer's own report states that worktree "was not touched"). To
   satisfy item 6 as written I ported the two files that add it
   (`cmd/emu/screen.go`, `cmd/emu/screen_js.go`) from `wt-composer-s4-emu`
   into my disposable copy only — **never into the original worktree**. Diffed
   both files first: the addition is a pure read-only diagnostic
   (`frameTargets` calls `op.Drawer.Hit`, the same lookup the real event
   router performs; it injects no event and changes no GUI behaviour). Confirmed
   it builds clean (`go build ./cmd/emu/...` and the `GOOS=js GOARCH=wasm`
   build both exit 0) and `go test ./cmd/emu/` still passes after the port.

## Verify

### 1. Diff scope and hunks — VERIFIED

```
$ git diff 3cc71d9b..0b49f66c16aae1a055b8403a24b242acd3548710 --stat
 gui/composer_measure_test.go        |  15 +++
 gui/composer_paged.go               |  41 ++++--
 gui/composer_paged_geometry_test.go | 240 ++++++++++++++++++++++++++++++++++++
 3 files changed, 287 insertions(+), 9 deletions(-)
```

Exactly 2 hunks in `composer_paged.go` (`git diff ... | grep -c '^@@'` = 2),
both inside `composerPageLines`: the band computation
(`bandLeft/bandRight/lineWidth`, replacing the old wrap-at-`dims.X-16`
centred-on-the-whole-panel math) and the label position, now
`bandLeft+(lineWidth-sz.X)/2` instead of `(dims.X-sz.X)/2`. The touch bands
(`bands = append(bands, image.Rect(bandLeft, ..., bandRight, ...))`) use the
same `bandLeft`/`bandRight` as the text — confirmed by reading the function
whole, not just the diff. `composer_measure_test.go`'s diff is a doc-comment
addition only (no assertion changed). `composer_paged_geometry_test.go` is
new. `confirmReviewScreen` (`gui/multisig_build.go`) and `layoutNavigation`
(`gui/gui.go`) do not appear in the diff at all — untouched, confirmed by the
file list above.

**Side-check on the "one question":** W-3 moves the shared hit-rect's right
bound from `dims.X - navWidth` (427, no margin — the value W-2 shipped) to
`dims.X - navWidth - bandMargin` (419, an 8px-narrower band). Re-ran W-2's own
regression test against the fix:

```
$ go test -run TestComposerPickScreenRowsAreTouchable -v ./gui/
--- PASS: TestComposerPickScreenRowsAreTouchable (0.00s)
```

Tap-to-select on `composerPickScreen` rows is not regressed by the narrower
band.

### 2. Geometry test: RED / GREEN / RED-under-mutation — VERIFIED

RED on `3cc71d9b`, reconstructed in a `cp -r` copy
(`/scratch/code/shibboleth/.tmp/w3red`) by `git show 3cc71d9b:gui/composer_paged.go`
over the fix's own worktree copy (verified byte-identical to the base file by
diff), keeping the fix's new test file:

```
--- FAIL: TestComposerPagedLinesNeverDrawUnderTheNavButtons (0.01s)
    keyed stub page 0: ... button (427,44)-(480,97) received ink at (451,57)
      the line(s) that reach under it: [... "Template-ID: 1b0e92323e7ac98f875e18c91dbc92d1" ...]
    keyed stub page 1: ... ink at (429,86) ...
    keyless stub page 0: ... "Template-ID: 585422bf5c61f4da1649bca061c43334" ...
FAIL	exit 1
```

GREEN on the fix (`cp -r` copy `/scratch/code/shibboleth/.s4c-verify`):

```
--- PASS: TestComposerPagedLinesNeverDrawUnderTheNavButtons (0.00s)
--- PASS: TestComposerPagedGeometryProbeCanSeeInk (0.00s)
ok  	exit 0
```

RED again under mutation, in a third `cp -r` copy
(`/scratch/code/shibboleth/.tmp/w3mut`), right bound restored to full width
(`bandRight := dims.X // MUTATION`):

```
--- FAIL: TestComposerPagedLinesNeverDrawUnderTheNavButtons (0.01s)
    keyed stub page 0: ... ink at (437,57) [was (451,57) pre-fix] ...
    keyless stub page 0: ... ink at (428,57) [was (427,57) pre-fix] ...
FAIL	exit 1
```

The mutated line ran: the ink lands at different x-coordinates than the
pre-fix RED run (437 vs 451, 428 vs 427) because `lineWidth` widened when
`bandRight` was mutated back to `dims.X` — a distinct code path, not a stale
build.

### 3. Every `composerPageLines` user — VERIFIED, with a coverage gap flagged

The shipped `composer_paged_geometry_test.go` drives the keyed stub, keyless
stub, and the seating pick list (with `composerCopySeatPrompt`, a long
prompt) — all clean on the fix. It does **not** drive the mapping review
(`composerReadScreen` via `composerMappingLines`, `gui/composer_review.go`) or
the "Which hash?" pick screen (`composerPickScreen`, `gui/composer_hash.go`),
even though both call `composerPageLines`. Wrote a throwaway check
(`gui/zz_w3_extra_geometry_test.go`, copy only, not part of the fix) reusing
the repo's own existing fixtures (`composerTwoPathList`, the C29
shared-fingerprint case from `TestComposerMappingLinesPrintOriginsVerbatimAndSayWhatIsNotChecked`,
and `composerHashRow`):

```
$ go test -run TestZZW3ExtraScreensNeverDrawUnderNav -v ./gui/   # on the fix
--- PASS: TestZZW3ExtraScreensNeverDrawUnderNav (0.00s)
```

To rule out this being a vacuous check, ran the same file against the
reconstructed pre-fix build (`.tmp/w3red`):

```
$ go test -run TestZZW3ExtraScreensNeverDrawUnderNav -v ./gui/   # on 3cc71d9b
    mapping review page 1: a line is drawn UNDER a navigation button.
      button (427,44)-(480,97) received ink at (429,75)
      offending line(s): ["SAME SEED, SAME PATH\nSlots @0 and @1 are the same
      seed. This path's 2-of-3 can be satisfied by one person. Liana will
      refuse it."]
FAIL
```

So the defect did generalize to the mapping review (a call site the shipped
test never exercises), and the fix — being centralised in
`composerPageLines` — closes it there too, confirmed rather than assumed.
**Finding (Minor): the shipped regression test does not cover the mapping
review or the hash pick screen**, so a future regression specific to either
would not be caught by CI, even though the current runtime behaviour on both
is correct. Not blocking: no screen anywhere in this repo, on this build, was
found drawing a glyph under a button.

### 4. Capacity pins — VERIFIED, recomputed independently

```
$ go test -run TestComposerMeasureSection13Numbers -v ./gui/   # on the fix
SPEC13 stub_screen    lines= 42 per_frame= 6 pages=7
SPEC13 pick_list      lines= 36 per_frame= 7 pages=6
SPEC13 consent        lines= 17 per_frame= 7 pages=3
SPEC13 descriptor_plate ceiling_chars=596  c10_688_fits=false
```

Matches `gui/composer_measure_test.go`'s own docstring exactly. Also matches
`design/SPEC_wallet_policy_composer.md` §13 item 1, which is **already folded
and committed** (`badd968f`, "spec fold: §13 item 1 -- the stub screen holds 6
rows per frame after W-3 ... pick list and consent stay at 7", committed
2026-09-03 04:14:50 -0700 — i.e., this was done before I started, not
something this review needs to chase). All three sources — the measurement,
the test's own comment, and the spec — agree.

### 5. Gates — VERIFIED, run directly (not read from the report)

| gate | result |
| --- | --- |
| `gofmt -l gui/` | lists only `transaction.go`, `transaction_golden_test.go`, `transaction_txrecord_test.go` — pre-existing, outside this diff |
| `go vet ./gui/` | only the 2 pre-existing `testing.ArtifactDir` findings |
| `go test -run '^TestComposer' ./gui/` | `ok  seedhammer.com/gui  4.296s` |
| `gui-shard-test.sh ./gui/ 24` | `partition verified exhaustive: 1189 == 1189`; `RESULT: ok -- all 1189 tests ran across 24 shards`, wall 38s |
| `go test ./cmd/emu/` | `ok  seedhammer.com/cmd/emu  1.158s` |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` | exit 0 |
| `scripts/test-32bit.sh` | `GOARCH=386 test: exit 0`, `GOARCH=arm build: exit 0` |
| `go vet -tags oraclelive ./...` ("oraclelive build") | exit 1 on **both** `3cc71d9b` and the fix, byte-identical findings (only map-iteration ordering differs) — 39 pre-existing findings elsewhere in the tree (`bspline`, `engrave`, `backup`, unrelated `go1.25`/`testing.ArtifactDir` and unkeyed-struct-literal vet findings). **No new finding from this fix.** |
| firmware size | fix: `1548784 code / 31796 data / 31004 bss -> 1580580 flash / 62800 RAM`. Independently built base (`3cc71d9b`, reconstructed copy): **identical**, `1548784 / 31796 / 31004 -> 1580580 / 62800`. **Delta = 0 B flash / 0 B RAM.** |

**Finding (Minor): the shard-count arithmetic in
`composer-S4-W3-fix-report.md` is wrong.** It states "The shard count is 1189
against 1188 before: +1". Independently reconstructed the true pre-fix tree
(fix worktree copy with `composer_paged.go` and `composer_measure_test.go`
reverted to `3cc71d9b` via `git show`, and `composer_paged_geometry_test.go`
deleted — verified via `git diff --stat` that these are the *only* 3 files
this commit touches, so the reconstruction is exact) and listed tests with
`go test ./gui/ -list '.*'`:

```
base (3cc71d9b, reconstructed):  1187 top-level tests
fix  (0b49f66):                  1189 top-level tests
diff: + TestComposerPagedGeometryProbeCanSeeInk
      + TestComposerPagedLinesNeverDrawUnderTheNavButtons
```

True base is **1187**, not 1188 (this also matches W-2's *own* commit message,
`2dff0ee`: "gui-shard-test.sh ./gui/ 24 ... ok -- all 1187 tests ran across 24
shards"). True delta is **+2**, exactly the two new test functions, nothing
else added or removed. The underlying gate result the report reports (1189,
exhaustive, all green) is itself correct — only the report's own "1188 / +1"
arithmetic is wrong. Not blocking: no gate outcome is affected, and "did
nothing else move" is in fact confirmed by the corrected numbers.

### 6. Emulator proof — VERIFIED, built and driven directly, screenshots inspected by eye

Built `emu.wasm` from the `.s4c-verify` copy (with the `shTargets()` port
noted above). Served `cmd/emu/` over `http://127.0.0.1:8917/`, drove the
key-less arm through Playwright's real `browser_evaluate` (JS `window.shTap`
— the emulator's actual pointer-event injection path, i.e. the same one a
finger drives) to the Template (stub) screen: boot offer declined,
`shSysw("none")`, `Wallet Policy` -> `Build a new policy` -> `Taproot (tr)` ->
`Build my own paths` -> a 2-of-3 spend path -> `Done` -> `Sorted (usual)`.

`shScreen()` for the resulting frame, unchanged from the implementer's report
(same deterministic wallet shape):

```
TemplateTemplate-ID:e0863d3ccac31a64d3b5e14b85ccd6c0mk1stub(template):e0863d3c
mkencode--xpub<xpub>--origin-fingerprint<fp>--origin-path<path>--policy-id-stub
e0863d3c
```

Screenshots saved at `/scratch/code/shibboleth/.tmp/w3-emu-verify/w3-keyless-stub-p0.png`
and `w3-keyless-stub-p1.png`. **Looked at both by eye:**

- Page 0: `Template-ID:` on its own line, then the full 32-hex-digit id
  `e0863d3ccac31a64d3b5e14b85ccd6c0` on the next, ending well clear of the
  Back button. Both `mk encode --xpub <xpub> --origin-fingerprint <fp>` and
  `--origin-path <path> --policy-id-stub e0863d3c` end clear of the
  page-forward button.
- Page 1 (tapped the page button once, `shTap(453,160)`): the "A wallet built
  here is its own wallet..." paragraph and all three `Slot @N expects a key
  at m/48h/.../3h` lines are clear of Back/page/confirm. No ink under any
  button on either page, confirming the geometry test's finding by eye and
  not only by pixel scan.

## Closing counts

- **0 Critical.**
- **0 Important.**
- **2 Minor:** (a) the shipped geometry test does not cover the mapping
  review or the "Which hash?" pick screen — both verified correct on this
  build by a throwaway check, which also confirmed the pre-fix build failed
  the same way on the mapping review, so the gap is real but the runtime
  behaviour is not; (b) `composer-S4-W3-fix-report.md`'s shard-count
  bookkeeping is wrong (true base 1187, true delta +2, not "1188 / +1") —
  the gate result itself is correct.
- **1 process note (not a fold-quality finding):** a fork subagent was
  briefly dispatched in violation of the brief's "no sub-agents" rule,
  caught before it produced output, stopped, and its output discarded
  unused — recorded above for transparency.
- Both brief-vs-content mismatches (the "one question" describing W-2, and
  `shTargets()` not existing on this branch) are brief-accuracy issues, not
  chargeable to the fix; both were worked around and the underlying
  questions answered anyway (side-check in item 1; API ported into the
  disposable copy only for item 6).

**Nothing outside W-3's stated scope moved.** The fix is confined to
`gui/composer_paged.go`'s `composerPageLines`, is covered by a test proven to
fail on the pre-fix code and under a restoring mutation, generalizes
correctly to two callers the shipped test does not exercise, matches its own
capacity pins and the already-folded spec, costs 0 bytes of firmware, and is
visibly correct on both pages of a real emulator screenshot driven through
the production input path.
