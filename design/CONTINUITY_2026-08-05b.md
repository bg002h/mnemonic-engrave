# CONTINUITY — 2026-08-05 (b)

Written at a context clear, after the SIZEPROOF! release shipped and flashed.
Successor to `CONTINUITY_2026-08-05.md`, which is superseded in full.

## 1. THE IMMEDIATE NEXT TASK

Everything below happens on branch **`constant-glyph-cleanup`**, in worktree
`/scratch/code/shibboleth/seedhammer-wt-glyphcleanup`. **Scope decision
(operator, 2026-08-05): this branch works on `font/constant` ONLY.** `sh` gets
hand-tuned or replaced another time — it is the aesthetic face and the operator's
verdict is that it "isn't very pretty", but that is a separate cycle.

In order:

1. **Finish the two GUI follow-ups** — in flight when this was written, see §5.
   Verify they landed green before building on them. They are unrelated to
   glyphs; they ride this branch only because the operator wanted them done
   before the glyph work buried them.
2. **DONE — the golden over the `SIZEPROOF!` plates.** `2ffb38c` on
   `constant-glyph-cleanup`: `gui/testdata/sizeproof-front.bin` and
   `sizeproof-back.bin`, driven by `gui/freetext_sizeproof_golden_test.go`. 44
   packages green, **zero existing golden bytes moved**. See §4, whose "87
   glyphs" figure is now corrected by measurement.
3. **NEXT — tweak `const` glyphs.** With the goldens in place, every edit shows
   which glyph moved instead of changing something silently. **The workflow is in
   the test file's own doc comment, and it is not backup/testdata's:** these two
   goldens are *supposed* to move on a glyph edit. Run
   `go test ./gui -run TestSizeLadderGoldens -artifacts -outputdir=DIR`, diff the
   two SVG renders it drops there (the plate now vs. the plate the golden holds),
   confirm the diff is the edit you meant, then re-record with
   `go test ./gui -run TestSizeLadderGoldens -update` and commit the `.bin`
   beside the `constant.svg` change. **Never a bare `go test ./... -update`** —
   that takes backup's frozen sixteen with it.

**Read §4 before touching a glyph.**

## 2. What shipped, and where it is

**`seedhammer` `main` @ `a88686b`**, tagged **`fork-v0.0.0-ga88686b`**, released,
**and flashed to the SH2** (signed locally; sha256
`a8a0d085453e63d194599a42a8a9d3a5d1a9e0bcd96d6b4377feeea7ba475af0`).
Release: <https://github.com/bg002h/seedhammer/releases/tag/fork-v0.0.0-ga88686b>

It carries two features:

- **`SIZEPROOF!FRONT` / `SIZEPROOF!BACK`** — a two-sided proof plate carrying the
  complete 95-character sweep in BOTH faces at five rungs (5.0+3.8 front,
  4.4+3.4+3.0 back), to answer what a render cannot: which glyphs survive as the
  size drops on steel. Type the trigger into the free-text field.
- **`FONTPROOF!` → `PASSPROOF!`** — the passphrase proof trigger renamed. Roots
  now differ at the first character: `P`, `T`, `C`, `B`, `S`.

Under them, per-row sizing: `Block.SizeMM`, `Fitted.Sizes`/`Mixed`, a
plate-absolute QR band, a running y in device units, `FitSized`, and
`faceLayouts` deleted.

**The property everything rests on: zero golden bytes moved.** 16 goldens across
seed, passphrase, free-text, codex32, SLIP-39 and descriptor plates are identical
to the previous release. `-update` was never run at any point. **Keep it that
way** — a moved golden is a finding, not a baseline to refresh.

**`mnemonic-engrave` `master`** holds the spec (R11), the implementation plan
(R6), `FOLLOWUPS.md`, and every review verbatim in `design/agent-reports/`.

## 3. THE PLATES ARE NOT YET CUT

`SIZEPROOF!` has never been engraved. The whole point of the feature is the
verdict that only steel can give. Renders are in the session scratchpad but
regenerate freely:

```sh
cd /scratch/code/shibboleth/seedhammer
nix develop --command go run ./cmd/plateview -plate sizeproof-front -o /tmp/front.png
nix develop --command go run ./cmd/plateview -plate sizeproof-back  -o /tmp/back.png
```

Measured composition, pinned by test (`gui/freetext_sizeproof_table_test.go`):

| side | blocks (face@rung: rows) | per-row budgets | ends | spare |
|---|---|---|---|---|
| FRONT | sh@5.0:4, const@5.0:5, sh@3.8:3, const@3.8:4 | `[20 26 26 26]`, `[23×5]`, `[34 34 34]`, `[31 31 31 25]` | 78.400 mm | 3.600 |
| BACK | sh@4.4:4, const@4.4:4, sh@3.4:3, const@3.4:3, sh@3.0:3, const@3.0:3 | `[24 30 30 30]`, `[26×4]`, `[38×3]`, `[34×3]`, `[44×3]`, `[39 31 31]` | 79.600 mm | 2.400 |

## 4. READ THIS BEFORE TWEAKING GLYPHS

**MEASURED 2026-08-05 — what `font/constant` actually tests.** The branch is
scoped to `const` only; `sh` is hand-tuned or replaced another time (operator).

*Universal, all 95 runes:* `TestPrintableASCIICoverage` (present at all),
`TestUniformAdvance` (monospace — `NewConstantStringer` panics otherwise),
`TestGlyphsStayInTheirCell` (bounding box), `TestNoGlyphStartsAtOrigin`.
Plus `max k = 2` in `engrave/passphrase_alphabet_test.go`.

*Per-glyph, hand-listed:* `TestSmallFeaturesClearTheStroke` (a written list of
features, NOT every glyph), the quote ink gap, bowl junctions, `N`'s arch, `+`'s
arch, `S`'s foot, `=`'s bars, and `f` twice.

**The gap, MEASURED 2026-08-05 and CORRECTED.** The "roughly 87 glyphs" figure
above was read off test names. Measured properly — mutate one glyph, regenerate
`constant.bin`, run the whole tree — the picture is different in shape and the
same in conclusion:

| glyph | caught, before `2ffb38c` |
|---|---|
| `Q` uppercase | `engrave.TestConstantFont`, plus backup's seed, codex32 and slip39 plate goldens |
| `a` lowercase | `backup.TestPassphraseGolden`, and nothing else |
| `~` asciitilde | **nothing in the tree** |
| `{` leftcurlybrace | **nothing in the tree** |

The structure behind it: `engrave/testdata/font-constant.bin` sweeps
`constantAlphabet`, which is `"0123456789A-Z"` — **36 of the face's 96 glyphs** —
and widening it is forbidden at `engrave/engrave.go:770`. Every other
`font/constant` glyph was pinned only **incidentally**, by whichever characters
happened to appear in a plate golden's text: the lowercase letters had one plate
each and the punctuation had none. **`~` and `{` could have been scribbles.**

`font/sh` is the other way round and was never the problem: `engrave.TestFonts`
sweeps every rune the face decodes, so `engrave/testdata/font-sh.bin` already
pinned all 95.

The universal rules catch classes of defect — wrong advance, out of cell, starts
at origin — but not a shape that is simply wrong while staying in its cell with
the right advance. `a` drawn as a circle, `m` missing a leg, `3` reversed: all
passed. That is what "`f` could have been a scribble" really meant.

**Fixed by `2ffb38c`:** goldens over both `SIZEPROOF!` sides, which ARE the full
95-character sweep in both faces at five rungs. `TestSizeLadderGoldensPinEveryGlyph`
makes the coverage claim falsifiable rather than prose — every rune either face
decodes is on the ladder, except the visible-space mark `0x1F`, which no
free-text plate can carry and which backup's passphrase goldens pin. `0x20` inks
nothing in either face, so what is pinned for the space is its advance.

Also settled and worth not re-deriving:

- **`go generate ./font/constant/` after any `constant.svg` edit.** The compiled
  `constant.bin` is what gets engraved, and a test asserting SVG coordinates
  passes against a stale bin.
- **`f`'s top bar is 1.62 units**, below the 2-stroke-width minimum, accepted
  deliberately and pinned by `TestFTopBarIsShortByDesign`. **Untested on steel** —
  the SIZEPROOF! plate settles it.
- **`font/constant` is a B-spline, not a polyline.** Control points are not
  interpolated, so inserting a "collinear" midpoint or stacking 180° reversals
  changes the rendered curve. `#` grew a spurious diagonal and `$`'s bowl
  collapsed to a triangle that way. **Every mechanical test passed on the broken
  glyphs** — advance, decodability, origin — so only the render loop caught it.
- **The stroke-count disclosure bound HOLDS and is pinned.** `*`, `x` and `#` are
  single strokes and `$` is two, so **max k = 2** — which is what
  `T_row = rowLen + n_row` requires — and `engrave/passphrase_alphabet_test.go`
  asserts it. (An earlier draft of this file claimed `$` at k=2 broke the bound.
  It does not: the bound is a MAXIMUM. Corrected 2026-08-05 by reading the test.)
  **A glyph edit that splits any glyph into three runs breaks a security
  property, not an aesthetic one** — the bound is what limits what an observer of
  the machine's motion can infer about the text being cut.
- The alphabet's bounding box relocates every plate, and a single-feature glyph
  loses its identity to one scratch. See [[engraving-font-design-rules]].

### 4.1 THE HOUSE ANGLE — 7.125°

`font/constant` has one deliberate off-horizontal angle, and it is a **design
vocabulary**, not a stray number. **7.125° = atan(0.5 / 4)** — half a unit of
rise over a four-unit run. Verified 2026-08-05 by computing it, and by reading
`font/constant/glyph_rules_test.go`.

**Where it came from, which is the part worth keeping.** A slant was observed on
an engraved plate and looked like design. Measured, the PLANNED path was dead
flat (`n`'s top bar at y=11379 from x=2844 to x=14222) — **the slant was the
machine, not the drawing** — and it did not reproduce on `m`, `h` or `r`, so it
could not be relied on. *"Drawing it in is the only way to have it."*

Who carries it, and in which direction:

| glyph | angle | direction |
|---|---|---|
| `n` arch | 7.125° | **rises** left→right (`archLeftY, archRightY = 4.0, 3.5`) |
| `+` bar | 7.125° | **drops** left→right — the deliberate MIRROR |
| `f` crossbar | 7.125° | drops left→right |
| `f` hook | **14.25°** | twice the house angle — the one place it is doubled |
| `=` bars | 7.153° | 0.25 rise per bar; atan does not add exactly, and a tenth of a degree is the accepted slack |
| `m`, `h`, `r`, `t` | **flat, and must stay flat** | |

**Two rules that are easy to break by accident:**

1. **`n` and `+` lean APART on purpose.** A reader who learns "sloped arch = `n`"
   would be misled by a `+` sloping the same way, so the pair diverges rather
   than agreeing. At 3.0 mm a flat-barred `+` reads as a `t` that lost its foot.
2. **The siblings must stay flat or the point is lost.** `n` is only distinctive
   while `m`, `h` and `r` are flat; `+` only while `t` is flat. The tests assert
   the flatness of the siblings, not just the slope of the carrier.

**The angle is DECODED FROM `n` at test time, never written down as a constant**
(`glyph_rules_test.go:451`). That is deliberate: flattening `n` while leaving `f`
alone is then a *test failure* rather than a silent drift apart. **Do not
"helpfully" extract it into a shared constant** — that would destroy the coupling
that makes the check work.

## 5. DONE — the two GUI follow-ups (step 1 of §1 is complete)

Both landed on `constant-glyph-cleanup`, verified by the controller: 44 packages
green, **zero goldens touched**, and `TestAdmissibleBlocksVerdictDoesNotMove`
byte-for-byte untouched (zero deleted lines in its file).

- **`f466b11`** — the QR step no longer offers a code it will not honour. With a
  SIZED composition loaded the screen reads *"This pattern is cut at several
  sizes and needs the whole plate. It carries no QR and is not machine-readable."*
  with a single `No QR` button, and the prior opt-in is deliberately not carried
  in. Unsized is byte-for-byte the old screen. The privacy sentence goes with the
  offer — it describes a photograph of a code that will not exist.
- **`b2f40b4`** — new `AdmissibleSized` beside an untouched `AdmissibleBlocks`;
  `ftEvaluate` routes on `ftSizedBlocks`, the same predicate `ftFitAt` uses.
  Readout now: front `5.0+3.8mm 16/16 lines` (was `12/24`), back
  `4.4+3.4+3.0mm 20/20` (was `18/24`). A grown front now refuses with *"needs 17
  lines and this pattern's own sizes hold 16"* instead of quoting 24.

`b2f40b4` reverts cleanly on its own if needed.

### The instruction that was impossible — worth remembering

The controller's brief said *"title and footer rows stay reserved unconditionally
on the sized path too"*, carrying monotonicity over from the uniform case. **That
is arithmetically impossible and would have refused both shipped ladders:**

| side | spare (§1.1) | smallest rung | |
|---|---|---|---|
| FRONT | 3.600 mm | 3.8 mm | short by 0.200 |
| BACK | 2.400 mm | 3.0 mm | short by 0.600 — and 3.0 is the smallest rung that exists |

Spec §5 already said it from the other side. The implementer found it by MUTATION
— adding the reservation took the whole ladder flow red — rather than by obeying.
**The title is reserved; the footer is not**, and the doc comment carries the
measurement. The monotonicity that matters is intact: neither string is read, so
entering one cannot move the verdict.

`FOLLOWUPS.md` still lists both as Open, annotated as implemented-pending-merge.
Move them to Resolved when the branch lands.

## 6. Flashing — use the script

**`~/bin/sh/sh2-flash`.** Never picotool by hand. `~/bin/sh` is NOT on PATH.

```sh
~/bin/sh/sh2-flash            # build HEAD, sign, flash
~/bin/sh/sh2-flash -p         # pick a tag/commit to build
~/bin/sh/sh2-flash -b IMG     # build + sign, stop before the device
```

Two traps it exists to prevent, both of which look like something else:

- **The build output cannot boot.** `build-firmware` ends with
  `picotool seal --sign --clear`, zeroing pubkey and signature; the OTP boot key
  (slot 1, burned 2026-08-03) means the bootrom starts only signed images.
  `sign-firmware.sh` writes a separate `.signed.uf2` and never modifies its input.
- **A laptop port cannot boot the machine.** `Init()` demands a 20–28 V USB-PD
  contract *before* configuring the LCD and reboots into BOOTSEL without one — so
  correctly signed firmware gives a dark screen and an `RP2350 Boot`
  re-enumeration while tethered. **Judge only on machine power.** Expect
  `(UNLOCKED)` on the version line; that is expected.

If it does not boot, **do NOT burn another OTP slot** — the burned hash was
proven correct, so it is a signing or image problem, retryable at zero OTP cost.
`design/RUNBOOK_custom_boot_key.md` step 6.

## 7. Traps found the hard way this cycle

- **`gh`'s default repo is `seedhammer/seedhammer` (UPSTREAM).** Every fork
  release needs `--repo bg002h/seedhammer` explicitly. A bare `gh release create`
  would publish to the SeedHammer project.
- **`cat >> FOLLOWUPS.md` appends after `## Resolved`.** Two known-open items
  were filed under Resolved that way and a grep of the Open section found
  neither. Insert before the Resolved heading.
- **The runbook's "pin `VERSION` to match CI" note is stale** — the flake no
  longer uses `git describe`; it defaults to `v0.0.0-g<short-sha>`, already the
  fork convention.
- **`picotool` is only in the nix devshell**, not on PATH.

## 8. The pattern this cycle proved

**Seven review findings across the spec, plan and implementation. Six were in the
RECORD, not the code.** Including: a spec test item that could not fail against
the defect it named; a guard the spec called redundant that is load-bearing; a
code comment citing a fabricated measurement for a term that protects nothing;
and a revert predicate traced only over deleted newlines, blind to inserted ones.

The one real code defect — admission and the fit disagreeing about the QR — was
found by the whole-diff review, and it was the *sibling* of a defect fixed one
phase earlier. Fixing a stated flaw tends to leave its twin one level down.

**What worked:** measured tables instead of prose, and mutation over reading. A
test that has never been observed failing has not been tested. See
[[mutation-testing-finds-false-passes]] and
[[reviews-find-more-errors-in-records-than-code]].

## 9. Also open

- **The rest of `FOLLOWUPS.md` §Open** — 25 items, most predating this cycle.
  Glyph-adjacent ones: `seedhammer-hash-glyph-still-k4` (`$` at k=2),
  `seedhammer-font-svg-ncname-ids` (`id="!"`, `id="\"` are not valid NCNames).
- **`~/bin/sh` is not on PATH** — `fish_add_path ~/bin/sh` when wanted.
- **Stale worktrees** under `/scratch/code/shibboleth/seedhammer-wt-*`; most are
  merged and removable. `seedhammer-wt-glyphcleanup` is live.
