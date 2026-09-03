# composer S4 — W-3 fix report: paged lines wrap and centre clear of the nav column

**Defect:** W-3 in `design/S4_journey_walk_2026-09-02.md`. **Brief:**
`design/agent-briefs/composer-S4-W3-fix-brief.md`. **Implementer:** the same
single opus agent, resumed.

**Outcome: DONE.** The decided fix shape was executed. The geometry test fails
on `3cc71d9b` naming the Template-ID line and passes after; every gate is green;
the firmware is unchanged to the byte; and the emulator shows the 32nd hex digit
clear of the Back button.

## Worktree

| repo | path | branch | base |
| --- | --- | --- | --- |
| seedhammer fork | `/scratch/code/shibboleth/wt-composer-s4c` | `composer-s4c` | `main` `3cc71d9b` |

```
$ git -C /scratch/code/shibboleth/wt-composer-s4c log --oneline main..HEAD
0b49f66 gui: the composer's paged lines wrap and centre clear of the nav column (S4 walk W-3)
```

Tree clean (`git status --porcelain` empty, exit 0). `wt-composer-s4-emu` and
`wt-engrave-s4-emu` were **not touched** — neither appears in this branch's
history and both were left as the fold left them. Nothing pushed, nothing
flashed, no sub-agent, no `.jsonl` read. The RED run used a `cp -r` copy at
`/scratch/code/shibboleth/.tmp/w3red` with its `.git` link removed.

## The defect, and why the walk could not see it

`composerPageLines` wrapped every line at `dims.X - 2*8` (464 px) and centred it
across the **whole** panel, while `layoutNavigation` places Back / page / take in
a column at `dims.X - NavBtnPrimary.width` (427 px). Any line measuring near the
wrap bound was **drawn under a button**.

**`op.Drawer.ExtractText` collects a glyph's rune wherever it lands, under a
button included.** So `shScreen()` returns the complete string on both builds —
byte for byte identical before and after this commit — and every text-presence
assertion in `capture_composer.py` passed against a screen whose operator cannot
read it. A presence assertion is not a legibility assertion; that is the whole
finding.

## The fix

**`gui/composer_paged.go`**, `composerPageLines`. One band, and everything uses
it: text is **wrapped and centred inside the band left of the navigation
column** — the bound the W-2 hit areas already computed — and the hit rects share
those two bounds instead of computing their own.

```go
const bandMargin = 8
bandLeft  := bandMargin                                                    // 8
bandRight := dims.X - assets.NavBtnPrimary.Bounds().Size().X - bandMargin  // 419
lineWidth := bandRight - bandLeft                                          // 411
…
lbl, sz := widget.Labelw(&ctx.B, ctx.Styles.body, lineWidth, col, lines[i])
pos := image.Pt(bandLeft+(lineWidth-sz.X)/2, y)   // centred in the BAND
```

`bandMargin` is **the same margin the left edge always had** — the old
`(dims.X - (dims.X-2*8))/2` — applied on the right of the text as well, so a
glyph never sits flush against a button it is not part of. That is my reading of
the brief's *"minus the same margin `layoutNavigation` leaves"*; `layoutNavigation`
itself places the button flush at 427 with no gap, so the margin that exists to
be matched is this function's own. Stated because it is the one number I chose.

Long lines **wrap**, nothing shrinks. The consent screen (`confirmReviewScreen`),
the navigation layout, and the hit-area computation beyond the shared bound are
untouched.

## The test — RED on `3cc71d9b` first

**`gui/composer_paged_geometry_test.go`** (new). It rasterises the body ops into
an `rgb565` buffer — exactly as `ExtractText` does — and looks for **ink inside
the button rectangles**, which is what an eye does and the only check this
symptom cannot slip past. The nav buttons are not drawn into the test frame, so
any ink inside a button rect is body ink by construction. It renders the keyed
and key-less stub screens through the real widget from `md.Compose`d fixtures,
plus the pick list, and checks **every page**.

```
$ go test -count=1 -run 'ComposerPagedLinesNeverDraw|ComposerPagedGeometryProbe' ./gui/
FINAL TEST FILE ON 3cc71d9b EXIT=1

--- FAIL: TestComposerPagedLinesNeverDrawUnderTheNavButtons
  keyed stub page 0: a line is drawn UNDER a navigation button.
    button (427,44)-(480,97) received ink at (451,57)
    the line(s) that reach under it: ["The shape changed, so this id changed. Cards
    minted with the old stub will not seat here."
    "Template-ID: 1b0e92323e7ac98f875e18c91dbc92d1"
    "mk encode --xpub <xpub> --origin-fingerprint <fp>"
    "  --origin-path <path> --policy-id-stub 1b0e9232"]
  keyed stub page 1: … ink at (429,86) … ["A wallet built here is its own wallet. …"]
  keyless stub page 0: … ink at (427,57) … ["Template-ID: 585422bf5c61f4da1649bca061c43334" …]
```

The Template-ID line is named, as the brief required. **After the fix:**

```
=== RUN   TestComposerPagedLinesNeverDrawUnderTheNavButtons
--- PASS
=== RUN   TestComposerPagedGeometryProbeCanSeeInk
    scanner sees ink: button (427,44)-(480,97) at (434,55)
--- PASS
GEOMETRY TESTS AFTER THE FIX EXIT=0
```

### The scanner's own mutation proof, and a correction I had to make

`TestComposerPagedGeometryProbeCanSeeInk` hands the scanner a label placed
**inside** a button and requires a hit, then the same label at the left margin
and requires none.

Its first form went through `composerPageLines` with a 200-character unbroken
token. That was wrong: once the fix landed, the token no longer reached a button
— `widget.Labelw` bounds it to the band — so **the proof started passing for the
same reason the gate did**, and stopped proving anything about the scanner. It
was rewritten to take ops directly (`inkUnderNavOps`), which is a fact about the
buffer rather than about the thing under test. A proof that depends on the code
it certifies is not a proof.

## Capacities — one moved

`SPEC_wallet_policy_composer.md` §13 item 1 quotes
`TestComposerMeasureSection13Numbers`' output verbatim, so it is stale until
re-pasted.

```
--- BEFORE (3cc71d9b) ---                    --- AFTER (the fix) ---
SPEC13 stub_screen  lines=42 per_frame= 7 pages=6   per_frame= 6 pages=7   <- MOVED
SPEC13 pick_list    lines=36 per_frame= 7 pages=6   per_frame= 7 pages=6
SPEC13 consent      lines=17 per_frame= 7 pages=3   per_frame= 7 pages=3
SPEC13 descriptor_plate ceiling_chars=596           ceiling_chars=596
```

The stub screen lost a row because `Template-ID: <32 hex>` no longer fits one
line inside the band and wraps to two — the honest cost of not drawing the last
digit under a button. **The spec's sentence "all three paged screens hold 7 rows
per frame" is false from this commit; the stub screen holds 6.**

**What I could not do.** `design/SPEC_wallet_policy_composer.md` lives in
**mnemonic-engrave**, and this step was given only the fork worktree — no engrave
worktree, and writing a tracked file in the shared main checkout is what the push
discipline exists to prevent. **No pin exists in the fork to update**: I grepped,
and every capacity there is logged (`t.Logf`) or asserted structurally ("it must
page", "the tail is reachable"), never as a number. So I recorded the before/after
table and the falsified sentence in `gui/composer_measure_test.go`'s own header —
the file the spec quotes from — and the spec edit is left for the controller.
The replacement block is the "AFTER" column above, verbatim from the test.

## Gates

| gate | exit | detail |
| --- | --- | --- |
| `gofmt -l gui/` | 0 | lists only `transaction.go`, `transaction_golden_test.go`, `transaction_txrecord_test.go` — unformatted at `60bee002` already, outside this diff, untouched |
| `go vet ./gui/` | 1 | only the two pre-existing `testing.ArtifactDir` findings (`freetext_sizeproof_golden_test.go:111`, `transaction_golden_test.go:104`) |
| `go test -count=1 -run '^TestComposer' ./gui/` | 0 | `ok  seedhammer.com/gui  6.149s` |
| `gui-shard-test.sh ./gui/ 24` | 0 | `all 1189 tests ran across 24 shards`, wall 38s |
| `CGO_ENABLED=0 go test -count=1 ./cmd/emu/` | 0 | `ok  seedhammer.com/cmd/emu  1.775s` |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` | 0 | |
| firmware size recipe | 0 | `1548784 code / 31796 data / 31004 bss → 1,580,580 B flash / 62,800 B RAM` |

**The firmware is the plan's pinned number to the byte** (`1,580,580 / 62,800`).
The fix replaces arithmetic with arithmetic; it costs nothing. The shard count is
1189 against 1188 before: +1, this test file's two tests minus none (the gate and
its probe are two, one of which replaced nothing — the count reflects the new
file).

## The emulator proof

`emu.wasm` built from **this** worktree; the key-less arm driven to the Template
screen by touch alone (the composer payload blob is on another branch and the
key-less arm needs none). Pick-screen rows were tapped at points read off
`composerPageLines`' own returned hit bands, not derived. The same walk was run
against a build of pristine `3cc71d9b` for the comparison.

```
/scratch/code/shibboleth/.tmp/w3-shots/k02-stub-p0-BEFORE-3cc71d9b.png
/scratch/code/shibboleth/.tmp/w3-shots/k02-stub-p0-after-W3.png
/scratch/code/shibboleth/.tmp/w3-shots/k02-stub-p1-BEFORE-3cc71d9b.png
/scratch/code/shibboleth/.tmp/w3-shots/k02-stub-p1-after-W3.png
```

**BEFORE** — `Template-ID: e0863d3ccac31a64d3b5e14b85ccd6` with `c0` cut off
under Back; `--origin-fingerprint <f` with `p>` under the pager;
`--policy-id-stub e0863d3` with the trailing `c` under the pager.

**AFTER** — `Template-ID:` on its own line, then
`e0863d3ccac31a64d3b5e14b85ccd6c0` complete and ending well clear of Back; both
`mk encode` lines end clear of the pager.

**And the point of the whole finding, in one line:** `shScreen()` is *identical*
on both builds —

```
"TemplateTemplate-ID:e0863d3ccac31a64d3b5e14b85ccd6c0mk1stub(template):e0863d3c
 mkencode--xpub<xpub>--origin-fingerprint<fp>--origin-path<path>--policy-id-stub
 e0863d3c"
```

— which is why the capture passed on a screen missing two hex digits. The only
difference the fix makes to the reported text is the *pagination*: the "A wallet
built here is its own wallet…" paragraph moved from page 0 to page 1, the
capacity change above, visible from outside.

`Template-ID: e0863d3ccac31a64d3b5e14b85ccd6c0` is the S4 fixture's own key-less
id, so the walk built the right shape as well as rendering it legibly.

## What I decided, and what I could not do

1. **The right bound is `dims.X − navWidth − 8`**, sharing `bandMargin` with the
   left edge, so the hit rect narrowed from `8..427` to `8..419` with it. The
   brief said to share the bound; this is the one number I chose, and the reason
   is above.
2. **The scanner's mutation proof was rewritten** after the fix made its first
   form vacuous — recorded rather than quietly fixed, because it is the same
   false-PASS shape this tree hunts.
3. **The SPEC §13 edit is not in this commit** — the spec is in mnemonic-engrave,
   outside the worktree I was given. The measured replacement is above and the
   falsified sentence is named; the fork carries the record in
   `composer_measure_test.go`.
4. **`design/journeys/capture_composer.py`'s own expectations are unaffected** —
   it asserts on `shScreen()` text, which is unchanged — but the walk's recorded
   *page counts* for the stub screens will move by one when it is next run
   against a build carrying this fix. `shots_composer.js` asserts
   `stub1.pages.length !== 2` for the keyed arm and `stub.pages.length !== 2` for
   the key-less one; **both of those will fail once this branch merges**, because
   the stub screen now pages 7 times where it paged 6. That is a real,
   predictable consequence of this fix on the other branch, and it is the
   controller's to schedule — I did not touch `wt-composer-s4-emu`.
5. **Not done, and not mine:** merging, pushing, flashing, and Task 4's live
   device walk (which the plan gates on the fix being flashed).
