# CONTINUITY — 2026-08-05 (b)

Written at a context clear, after the SIZEPROOF! release shipped and flashed.
Successor to `CONTINUITY_2026-08-05.md`, which is superseded in full.

## 1. THE IMMEDIATE NEXT TASK — tweak glyphs on `constant-glyph-cleanup`

That is what the branch was made for. **Read §4 before touching a glyph** — the
sweep it describes is why the branch exists, and doing glyph edits before it is
how a broken glyph ships invisibly.

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

**The gap: roughly 87 glyphs have no IDENTITY test.** The universal rules catch
classes of defect — wrong advance, out of cell, starts at origin — but they do
NOT catch a shape that is simply wrong while staying in its cell with the right
advance. `a` drawn as a circle, `m` missing a leg, `3` reversed: all pass today.
That is what "`f` could have been a scribble" really meant, and it is still true
of most of the alphabet.

**Cheapest complete fix:** a golden over the `SIZEPROOF!` plates, which ARE the
full 95-character sweep in both faces. Two artifacts pin every glyph's path at
once, and they are the same plates being cut in steel, so test and plate agree.
Do that BEFORE tweaking, or a tweak that breaks a glyph is invisible.

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

## 5. In flight right now

**One agent** burning down the two SIZEPROOF GUI follow-ups on
`constant-glyph-cleanup`, in worktree
`/scratch/code/shibboleth/seedhammer-wt-glyphcleanup`:

1. `sizeproof-qr-step-must-not-offer-what-it-drops` — the QR step offers
   "Add QR" with a ladder loaded and silently discards it. Spec §3.0 is the
   requirement. Scoped to SIZED compositions; `BOTHPROOF!` deliberately untouched.
2. `sizeproof-admission-count-at-its-own-rungs` — admission counts a ladder at
   the 3.0 mm anchor, so the readout says 12/24 where the plate is 16 rows
   (front) and 18/24 where it is 20 (back). **`AdmissibleBlocks` must not change
   for non-sized plates** — spec §6 pins the anchor for monotonicity and
   `TestAdmissibleBlocksVerdictDoesNotMove` guards it. Preferred shape is a
   sibling `AdmissibleSized`, not a widened `AdmissibleBlocks`.

If that agent is gone when you resume, check
`git -C /scratch/code/shibboleth/seedhammer-wt-glyphcleanup log --oneline` for
one or two commits and the worktree's cleanliness before assuming anything.

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
