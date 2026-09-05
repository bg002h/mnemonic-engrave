# hashlock H2 — post-implementation review, lens: GEOMETRY ON THE REAL PANEL

**Subject:** fork `hashlock-h2` @ `17b3979` (6 commits over `c4a64fc`).
**Question, and the only one answered here:** does every NEW or CHANGED screen and
modal on the branch fit, without clipping or overlap, at `sh2DisplaySize` and at the
default test display?
**Method:** measurement, not reading. Every number below comes from a Go test run in a
detached worktree at `17b3979` (`/scratch/code/shibboleth/.tmp/h2-wf-lens-geometry`,
removed after the run). Bodies were measured with the package's own
`bodyDrawnFully` / `modalHeadroom` / `firstModalFrame` / `normalizeDrawn` machinery;
non-modal screens were **rasterised** with `rasterInk` (`gui/composer_paged_geometry_test.go:101`)
and measured in pixels, because `op.Drawer.ExtractText` collects a glyph's rune wherever
it lands and cannot see occlusion or a clamp.
**Nothing was committed.** The probe files (`gui/zz_lens_*_test.go`) were deleted.

**Toolchain:** `/scratch/code/shibboleth/.toolchain/go/bin/go`.
**Panel facts, resolved rather than assumed:** `cmd/controller/platform_sh2.go:35-36`
`lcdWidth = 480`, `lcdHeight = 320`; `gui/gui_test.go:405` `sh2DisplaySize = image.Pt(480, 320)`;
`gui/gui_test.go:401-404` calls the 240x240 default *"a fiction that no shipped device
has"*, and `cmd/emu/screen_test.go:16` confirms the emulator is 480x320 too. **So
480x320 is the panel that decides a finding; 240x240 results are recorded as N-1 and
gate nothing.**

---

## The table — every NEW/CHANGED modal at sh2DisplaySize (480x320)

`margin` is `modalBodyMargin = 80` (`gui/modal_fits_test.go:52`).

| surface | renderer | body chars | drawn/expected | headroom | verdict |
|---|---|---|---|---|---|
| §8i rule modal (`composerCopyHashRule`) | `showError` | 132 | 132/132 | 418 | fits |
| refusal `ErrEmpty` | `showError` | 32 | 32/32 | 513 | fits |
| refusal `ErrNotPrintableASCII` | `showError` | 36 | 36/36 | 513 | fits |
| refusal `ErrMS1Shaped` | `showError` | 91 | 91/91 | 476 | fits |
| refusal `ErrTooLong` | `showError` | 37 | 37/37 | 513 | fits |
| refusal `ErrHex64` | `showError` | 51 | 51/51 | 513 | fits |
| reconcile screen §4.5 | `showError` | 94 | 94/94 | 455 | fits |
| HASH ON EVERY PATH, phrase form §4.7 | `showError` | 160 | 160/160 | 378 | fits |
| F-474 not-permitted (preimage arm) | `showError` | 86 | 86/86 | 476 | fits |
| F-474 not-permitted (unknown arm) | `showError` | 86 | 86/86 | 476 | fits |
| hardened warning §4.3 | `ConfirmWarningScreen` | 189 | 189/189 | 302 | fits |
| sha256 warning §4.3 | `ConfirmWarningScreen` | 226 | 226/226 | 302 | fits |
| **confirm modal, LONGEST legal body (hardened)** | `ConfirmWarningScreen` | **337** | **337/337** | **107** | fits |
| confirm modal, longest legal body (sha256) | `ConfirmWarningScreen` | 335 | 335/335 | 107 | fits |

**Every modal on the branch is drawn IN FULL on its first frame at the shipped panel,
and every one clears the 80-character margin.** The tightest is the confirm modal at
107 characters of headroom.

**The branch's own "longest variant" row IS the longest legal body — verified, not
assumed.** Enumerating the whole product of `{hardened, sha256} x {no relation,
no-record, matches-1, matches-999} x {no other-path, other-path} x {0, 1, 100 chars}`:

```
longest legal body over the whole product = 337 normalized chars; the branch's own
test row = 337; identical=true
relation lengths: none=0 no-record=46 matches-1=29 matches-999=31
```

`hashlockMethod.String()` is `"hardened"` (8) or `"sha256"` (6);
`composerCopyHashlockRelation(-1)` (46) dominates every `matches hash N` form;
`hashlock.ValidatePhrase` caps `chars` at 100; `hashlockFirst8Last8` is always 18.
So `gui/modal_fits_test.go:386-390` picks the true maximum.

**The two renderers have identical capacity, as the branch's own comment claims.**
`warningBodyClip` (`gui/gui.go:595-600`) takes only `dims`; my `showError` and
`ConfirmWarningScreen` measurements reproduce the branch's logged numbers exactly
(91/476, 94/455, 160/378, 189/302, 226/302, 337/107).

## The table — every NEW/CHANGED non-modal screen at 480x320

| surface | measurement | verdict |
|---|---|---|
| `Which hash?`, no payload records (96-char lead + 3 rows) | `drawn=5`, rows shown **3 of 3**, last band `(8,185)-(419,210)` vs contentBottom 276; no body ink under any nav button | fits |
| `Which hash?`, 2 payload records (5 rows) | rows shown **5 of 5**, last band `(8,225)-(419,250)`; no ink under nav | fits |
| method pick (`Which method?`, 2 rows) | rows shown **2 of 2**, last band `(8,138)-(419,163)`; no ink under nav | fits |
| `Deriving`, zero-state lead | pct `(184,68)-(295,132)`, lead `(92,170)-(388,193)` — **no overlap**; ink rows 15..191 of 320; no ink under nav | fits |
| `Deriving`, countdown lead (`About 993 seconds left.`) | pct unchanged, lead `(148,170)-(332,193)`; ink rows 15..187 | fits |
| reconcile screen, real frame | ink rows 16..275 of 320, body drawn in full | fits |
| **`Hashlock phrase` screen** | content.Dy **201**, `kbd.MaxHeight` **201**, block **209** (overflow **+8**), readout budget **11 px** against a 19 px line; **readout ink 0 px at every length, masked and revealed** | **FAILS — C-1, I-1** |

---

### C-1 — the `Hashlock phrase` screen draws NO readout at all: a <=100-character secret is typed with zero echo on the shipped panel

**This is the branch's own F-481, measured.** The implementation report
(`design/agent-reports/hashlock-H2-implementation-report.md:19`, :221, :266) discloses it
and explicitly hands the severity call to this review. It is not a false claim in a
record; it is an open defect the implementer could not close. Two things this lens adds:
the defect is **larger than "the inert `show` key"** — there is no readout in *either*
state — and its **cause is a specific fold**, measured below.

**Command:**

```
go test ./gui/ -run 'TestLensPhraseReadoutInk|TestLensPassphraseReadoutInkControl' -v
```

**Verbatim output** (probe: lay out `PassphraseKeyboard` exactly as
`gui/composer_hashlock.go:165-175` does, rasterise the keyboard block alone, count ink
in the readout strip above the grid):

```
    zz_lens_readout_test.go:50: hashlock phrase, revealed=false n=1: MaxHeight=201 gridY=182 avail=11 readoutH=19 readout-strip ink=0 px  frame-has-star=false frame-has-WWW=false
    zz_lens_readout_test.go:50: hashlock phrase, revealed=false n=5: MaxHeight=201 gridY=182 avail=11 readoutH=19 readout-strip ink=0 px  frame-has-star=false frame-has-WWW=false
    zz_lens_readout_test.go:50: hashlock phrase, revealed=false n=20: MaxHeight=201 gridY=182 avail=11 readoutH=19 readout-strip ink=0 px  frame-has-star=false frame-has-WWW=false
    zz_lens_readout_test.go:50: hashlock phrase, revealed=false n=100: MaxHeight=201 gridY=182 avail=11 readoutH=19 readout-strip ink=0 px  frame-has-star=false frame-has-WWW=false
    zz_lens_readout_test.go:50: hashlock phrase, revealed=true n=1: MaxHeight=201 gridY=182 avail=11 readoutH=19 readout-strip ink=0 px  frame-has-star=false frame-has-WWW=false
    zz_lens_readout_test.go:50: hashlock phrase, revealed=true n=5: MaxHeight=201 gridY=182 avail=11 readoutH=19 readout-strip ink=0 px  frame-has-star=false frame-has-WWW=false
    zz_lens_readout_test.go:50: hashlock phrase, revealed=true n=20: MaxHeight=201 gridY=182 avail=11 readoutH=19 readout-strip ink=0 px  frame-has-star=false frame-has-WWW=false
    zz_lens_readout_test.go:50: hashlock phrase, revealed=true n=100: MaxHeight=201 gridY=182 avail=11 readoutH=19 readout-strip ink=0 px  frame-has-star=false frame-has-WWW=false
```

**The control — the SAME keyboard on the pre-existing passphrase screen, which has no
lead** (`gui/passphrase_flow.go:111-146`):

```
    zz_lens_readout_test.go:85: passphrase (no lead) n=1: MaxHeight=245 gridY=182 avail=55 readoutH=19 strip ink=113 px  frame-has-WWW=false  block=(340,209) fits=true
    zz_lens_readout_test.go:85: passphrase (no lead) n=20: MaxHeight=245 gridY=182 avail=55 readoutH=19 strip ink=2260 px  frame-has-WWW=true  block=(340,209) fits=true
    zz_lens_readout_test.go:85: passphrase (no lead) n=100: MaxHeight=245 gridY=182 avail=55 readoutH=38 strip ink=4746 px  frame-has-WWW=true  block=(340,228) fits=true
```

113 / 2260 / 4746 px there; **0 px** here. The difference is the lead band.

**Mechanism, in the branch's own code.** `gui/composer_hashlock.go:165-175`:

```go
		_, content := screen.CutTop(leadingSize)
		content, _ = content.CutBottom(8)
		leadOp, leadSz := widget.Labelw(&ctx.B, ctx.Styles.lead, dims.X-2*8, th.Text,
			composerCopyHashlockPhraseLead())
		leadBand, content := content.CutTop(leadSz.Y)
		...
		counterBand, content := content.CutTop(cntsz.Y)
		...
		kbd.MaxHeight = content.Dy()
```

and `gui/passphrase_keyboard.go:454-455`:

```go
	if k.MaxHeight > 0 {
		avail := k.MaxHeight - k.size[k.page].Y - readoutGap
```

320 - 44 (title) - 8 (bottom) - **44 (lead)** - 23 (counter) = **201**; `avail` =
201 - 182 (grid) - 8 (gap) = **11 px**, against a 19 px line. The binary search at
`gui/passphrase_keyboard.go:458-470` (comment at :458) therefore drops **every** rune and `shown` is `""`.

**The cause is the r0-journey-I-5 fold, and this is measurable.** `SPEC_hashlock_H2_device.md`
§4.2 specifies the lead as *"Use a phrase you have never used anywhere else."*
`composerCopyHashlockPhraseLead()` (`gui/composer_copy.go:367-370`) prepends a second
sentence, which pushes the lead from one line to two:

```
    zz_lens_leadsize_test.go:26: lead 23 px (0 chars) -> MaxHeight 222, readout budget  32 px (one line = 19), block 209 fits=true : ""
    zz_lens_leadsize_test.go:26: lead 23 px (47 chars) -> MaxHeight 222, readout budget  32 px (one line = 19), block 209 fits=true : "Use a phrase you have never used anywhere else."
    zz_lens_leadsize_test.go:26: lead 44 px (86 chars) -> MaxHeight 201, readout budget  11 px (one line = 19), block 209 fits=false : "This screen does that hashing for you. Use a phrase you have never used anywhere else."
```

**The spec's own lead leaves 32 px — enough for a readout line and a block that fits.
The shipped lead leaves 11 px and an 8 px overflow.** A one-line lead (or a lead band
capped at `leadingSize/2`, or `kbd.MaxHeight` floored at `grid + gap + one line` with
the lead cut to what remains) restores it; the fix is a layout arithmetic change in
`hashlockPhraseFlow`, not new copy.

**Why it matters, stated against what the machine does next.** The counter is an oracle
for **length only**. The key pitch is 34 px with 8 px dead between keys (the walk's own
header, `cmd/emu/walk_hashlock_phrase.js:54-58`), so a tap that lands in a gap does not
increment and *is* caught — but a tap that lands on the **wrong** key increments exactly
the same and is not. Nothing on the route ever shows the operator the bytes: the confirm
modal shows `hash`, `method: X`, `chars: N` and then instructs *"Write down this phrase
and the method now"* — a phrase the device never echoed. After HOLD the hash is assigned
to the path and the composition proceeds to engraving; the §4.5 reconcile screen
(`ms hashlock` on the host) is the only detector, and it is advice given *after* the
assignment, not a gate before it. A single mistyped character is therefore a host/device
digest divergence that surfaces, at the earliest, on a cut plate.

Secondary, and the fork has already ruled on this shape: the `show` key is drawn,
tappable, flips to `hide`, and reveals nothing. `gui/passphrase_keyboard.go:102-104` states the
reason the settings gear was **removed** rather than left inert — the gear *"was
drawn there, was tappable, and did NOTHING AT ALL -- a live-looking control that
swallows the press, on the machine where the next thing the operator approves is cut
into steel."*

**Why nothing on the branch went red.** The package's content assertions read the string
*submitted* (`ExtractText`), which is why `TestHashlockPhraseRouteSetsTheCorpusDigest`
and the emulator walk both pass — they drive the keyboard and check the digest, never the
frame. The sibling screen has the geometry test this one lacks:
`TestPassphraseEntryFitsPanel` (`gui/passphrase_flow_test.go:1331`) asserts exactly
`"the readout shows no run of passphrase text; the height clamp emptied it instead of
keeping the tail"`. **The branch adds a second keyboard screen and no second panel-fit
test.** Ported verbatim, that test goes red on `hashlockPhraseFlow` at every length
(output under I-1).

**Severity, argued in both directions so the controller can rule.** Under this lens's own
narrower mapping — *Critical only if the DIGEST or the counter is what gets cut* — this is
Important: the digest draws in full (337/337, headroom 107) and the counter is measurably
intact (C-1 evidence continues in I-1's counter-occlusion measurement). Under the review
brief's severity list it is **Critical**: it is a *host/device digest divergence path*
whose only detector runs after the hash is assigned, and its realised cost is a cut plate.
**I report Critical**, and the mitigations are stated above in full so a downgrade to
Important is a one-line decision rather than a re-measurement.

### I-1 — the keyboard block exceeds its own `MaxHeight` bound by 8 px: the invariant `TestPassphraseEntryFitsPanel` exists to pin, violated on the new screen

**Command:**

```
go test ./gui/ -run 'TestLensPhraseScreenGeometry' -v
```

**Verbatim (sh2 480x320; identical at n = 0, 20, 70, 90, 100, 101):**

```
    zz_lens_geometry_test.go:142: MEASURE sh2 480x320/n=100: content.Dy=201 MaxHeight=201 leadSz.Y=44 (band (0,44)-(480,88)) cntSz.Y=23 (band (0,88)-(480,111)) kbdsz=(340,209) gridY=182 kbdTop=103 overflow=8
    zz_lens_geometry_test.go:156: FAIL sh2 480x320/n=100: keyboard block is 209 tall, only 201 available -- overflows upward by 8 and op.Layer draws it ON TOP of the counter/lead
    zz_lens_geometry_test.go:162: FAIL sh2 480x320/n=100: keyboard block top y=103 is above the counter band bottom y=111 -- 8 px of overlap
    zz_lens_geometry_test.go:176: FAIL sh2 480x320/n=100: readout shows no run of typed text -- the clamp emptied it
```

The spec line this measures against is `gui/passphrase_keyboard.go:53-61`:
*"MaxHeight bounds the whole block (readout + gap + grid) ... since it is bottom-aligned
and drawn by op.Layer ON TOP, the overflow silently covers whatever is above it."*

**It is HARMLESS TODAY, and only because C-1 is true.** The 8 px of overflow is the empty
readout's own line box, so it carries no ink. Measured directly:

```
    zz_lens_counter_test.go:50: n=0:   lead band (0,44)-(480,88) ink=3909   counter band (0,88)-(480,111) ink=488   text-has-counter=true
    zz_lens_counter_test.go:50: n=100: lead band (0,44)-(480,88) ink=3909   counter band (0,88)-(480,111) ink=618   text-has-counter=true
    zz_lens_counter_test.go:50: n=101: lead band (0,44)-(480,88) ink=3909   counter band (0,88)-(480,111) ink=572   text-has-counter=true
```

The counter draws (488 / 618 / 572 px) and is legible at 100 **and** at the over-length
101 signal; the lead draws in full (3909 px, `bodyDrawnFully` ok). So **nothing is cut
today.**

The defect is that the bound is violated, and the two findings are coupled in the
dangerous direction: **restore the readout without also widening the budget and the
block grows into the counter band, which is the very occlusion
`TestPassphraseEntryFitsPanel` was written for** — *"from ~70 characters the counter was
hidden in exactly the revealed state a user proof-reads in"*
(`gui/passphrase_flow.go:120-128`). Fix C-1 and I-1 together, and add the panel-fit
sibling test so the next lead edit cannot re-break either silently.

### M-1 — the phrase screen's lead paints 152 px inside the Back button's rectangle (W-3's margin, spent)

The lead wraps at `dims.X-2*8` and is centred on the **whole panel**
(`gui/composer_hashlock.go:166-168`), not on the narrower band `composerPageLines` uses
for exactly this reason (`gui/composer_paged.go:62-90`, W-3).

```
    zz_lens_navink_test.go:30: lead size (440,44) placed at (20,44); nav column starts at x=427
    zz_lens_navink_test.go:40: W-3 FAIL: the lead draws ink under nav button (427,44)-(480,97) at (431,52)
    zz_lens_navink_test.go:42: W-3 ok: the counter draws no ink under any nav button
```

**But no glyph and no chip pixel is actually lost**, and the measurement says so rather
than the reasoning:

```
    zz_lens_zorder_test.go:147: Back button rect (427,44)-(480,97): chip ink 437 px, LEAD ink inside the rect 152 px (leftmost x=431), overlapping both 0 px, of which 0 px the lead OVERWRITES on the chip
```

The lead's tail sits in the button rectangle's empty margin, 4 px past the boundary, and
z-order is the reverse of W-3's case — the lead is drawn **on top** of the nav
(`op.Layer(kbdOp, leadOp, cntOp, nav, titleOp, ...)`, `gui/composer_hashlock.go:183`):

```
    zz_lens_zorder_test.go:47: at (431,52): lead alone #8cba8c | nav alone #217d21 | background alone #217d21 | PRODUCTION Layer(lead,nav,bg) #8cba8c
```

So the failure mode if the lead is ever re-worded longer is that the lead obscures the
**Back chevron**, not that the lead loses its tail. Minor: 4 px of margin remain, nothing
is currently unreadable, and any C-1 fix that shortens the lead also fixes this.

### N-1 — informational: the 240x240 default display, which no shipped device and no emulator has

Recorded because the brief asked for both display sizes, and gating nothing:
`gui/gui_test.go:401-404` calls 240x240 *"a fiction that no shipped device has"*;
`cmd/controller/platform_sh2.go:35-36` is 480x320 and `cmd/emu/screen_test.go:16`
confirms the emulator is too. Every new hashlock test sets `p.display = sh2DisplaySize`
(`gui/composer_hashlock_test.go:58-59`, :93-94) and `newDeadlinePlatform`
(`gui/run_harness_test.go:58-61`) does the same, so no branch test measures the fiction.

At 240x240 the following would fail, and are listed only so a future 240-wide panel is
not a surprise: the confirm modal draws 131 of 337 chars; the sha256 warning 176 of 226;
the hardened warning 169 of 189; §4.7 149 of 160; the §8i rule modal fits with 23 chars of
headroom (margin 80); `Which hash?` with the no-payload lead shows **0 of 3 rows**; the
`Deriving` lead overlaps the percentage; and the phrase keyboard (340 px wide) does not
fit a 240 px panel at all.

---

## What this lens checked and found clean

- Every H2 modal body is drawn in full on its first frame at 480x320, all clearing the
  80-character margin (table above); my numbers reproduce
  `TestModalsThisBlockTouchesAreDrawnInFull` and
  `TestConfirmScreensThisBlockTouchesAreDrawnInFull` exactly.
- The confirm modal's "longest variant" test row is provably the maximum over the whole
  legal product (337 chars), so the branch's tightest measurement is the right one.
- `Which hash?` fits all rows with the new 96-character no-payload lead, with 0 and with
  2 payload records; no row band runs past `contentBottom`; no row ink under a nav button.
- The method pick fits both rows.
- The `Deriving` screen's percentage and lead do not overlap in either lead state, no
  body ink reaches a nav button, and the bottom-most ink is at row 191 of 320.
- The reconcile screen draws in full as a real frame (ink rows 16..275 of 320).
- Both F-474 arms fit with 476 characters of headroom.
- The `n/100` counter is drawn and legible at 100 and at the over-length 101 signal, and
  the phrase-screen lead is drawn in full (3909 px of ink).

## Counts

**1 Critical / 1 Important / 1 Minor / 1 Nit.**

- **C-1** — the `Hashlock phrase` screen draws no readout at all (0 px, masked and
  revealed, every length) on the shipped panel; a <=100-character phrase that decides
  spendability is typed with no echo, and its only detector runs after the hash is
  assigned. (= the implementer's own F-481, measured and causally located; downgrade to
  Important if the controller applies the narrower "digest-or-counter" mapping.)
- **I-1** — the keyboard block is 209 px against a 201 px `MaxHeight`, violating the
  bound `TestPassphraseEntryFitsPanel` pins for the sibling screen; harmless only while
  C-1 is true, and it becomes the counter occlusion that test exists for the moment the
  readout is restored without widening the budget.
- **M-1** — the phrase screen's lead paints 152 px inside the Back button's rectangle
  (W-3), with 0 px over the visible chip and 4 px of margin left.
- **N-1** — 240x240 results, informational; no shipped device, no emulator, no branch
  test uses that size.
