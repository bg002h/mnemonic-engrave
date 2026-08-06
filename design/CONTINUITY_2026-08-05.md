# CONTINUITY — 2026-08-05

Written at a context clear. Successor to `CONTINUITY_2026-08-04.md`.

## 1. Where things stand

**Shipped.** `seedhammer` main @ `3c3a2ad`, tagged and released as
**`fork-v0.0.0-g3c3a2ad`**, pushed, with `seedhammerii-v0.0.0-g3c3a2ad.uf2`
attached to the GitHub release. **NOT FLASHED** — the device was not in BOOTSEL.
Flashing is the one outstanding action from that release.

`mnemonic-engrave` master @ `c106fa6`, pushed.

That release contains: the `BOTHPROOF!<rung>` size ladder; two glyph fixes read
off engraved plates (`=` bars opened and diverging, `f` crossbar sloped to y5
with a doubled hook lean); and two host tools, `cmd/plateview` (headless plate
renders) and `cmd/emu` (the firmware GUI as WebAssembly). Full notes are on the
GitHub release.

**In flight: `SIZEPROOF!`** — the two-sided size-ladder plate. Spec written,
R0 run twice, **still RED**. No code has been written and none may be until R0
is 0C/0I.

> **SUPERSEDED IN FULL by `CONTINUITY_2026-08-05b.md`.** SIZEPROOF! shipped,
> merged, released and flashed; read that file instead. The note below is the
> earlier partial supersession and is kept only for the trail.

> **SUPERSEDED 2026-08-05, later the same day.** The R2 fold in §2 is DONE
> (`9680a64`), round 2 ran (`17ffb1d`, RED 0C/5I), and the R3 fold is done
> (`38b7a84`). Round 3 is the open gate. §3's measured facts have two
> corrections, marked inline below. Read `design/SPEC_sizeproof.md` — it is the
> current artifact and supersedes §2 and §3 of this file wherever they differ.

## 2. THE IMMEDIATE NEXT TASK — R2 fold

Fold round 1's findings into `design/SPEC_sizeproof.md`, then re-dispatch R0.

Round 1 verdict: **1 Critical / 6 Important new.** All six round-0 Criticals are
confirmed fixed; do not revisit them. Reports, both verbatim:

- `design/agent-reports/bothproof-all-spec-R0-round0.md` (6C/8I)
- `design/agent-reports/sizeproof-spec-R0-round1.md` (1C/6I) ← fold this one

The seven to fold, shortest form (detail is in the round-1 report):

1. **CRITICAL — the QR y-range anchor is unspecified**, and the two consumers
   need OPPOSITE anchors: `EngraveText` (backup.go:385) is `baseY`-relative,
   `EngraveFitted` (freetext.go:85) is margin-relative. They agree today only
   because `wrapBlocks` passes `outerMargin` as every block's `baseY`. An
   implementer will naturally pick the `baseY` form and re-create the original
   defect. Reachable: load `BOTHPROOF!` (QR forced off), press Back to the QR
   screen — `plan` and `text` survive Back by design — enable the QR.
2. `§2.4`'s `limit` and `§2.5`'s footer y **do not compose** (overlap by the
   `LinesPerPlate` remainder, 3.0 mm at 3.8 mm). Both were folded verbatim from
   round 0 without checking they agree.
3. `LinesPerPlate(params, 0)` **divides by zero** on the no-footer case `§5`
   itself mandates. Same for a title with `TitleSizeMM == 0`.
4. `§2.5` never says how `EngraveFitted` gets the per-row screw-hole inset once
   `rows`/`start` are gone. The natural rewrite keeps `start + i` and diverges
   from the fit on a mixed plate.
5. Dropping `useQR` from `FitSized` does not stop the operator's QR choice being
   silently discarded; the existing prompted drop runs off `TextQR == ""`.
6. `§1`'s rationale sentence is false — see §4 below.
7. Undecided whether `wrapBlocks` honours `Block.SizeMM` over the passed
   `fontMM`, while `§6` asserts one side of it.

Plus partially-closed round-0 items: `§7` still lacks a test for the
`len(Sizes) != len(Lines)` guard, and `gui/preview.go:130` hardcodes
`ftProofFooter` for every proof preview.

## 3. Facts that are MEASURED — do not re-derive, do not re-open

The composition is **decided** (operator, 2026-08-05) and reproduced
independently by the round-1 reviewer:

| side | title | blocks (face@rung: rows) | body ends | spare |
|---|---|---|---|---|
| FRONT | `FRONT 5.0+3.8` @3.8 | sh@5.0:4, const@5.0:5, sh@3.8:3, const@3.8:4 | 78.40 mm | 3.60 mm |
| BACK | `BACK 4.4+3.4+3.0` @3.0 | sh@4.4:4, const@4.4:4, sh@3.4:3, const@3.4:3, sh@3.0:3, const@3.0:3 | 79.60 mm | 2.40 mm |

Limit 82.00 mm (`plateSize - outerMargin`). Titles are 13 and 16 chars against
`MaxTitleLen` 18. No footer on either side. No QR. No 6.0 mm rung.

Other settled measurements:

- **A title makes the FRONT roomier** (3.60 mm) than no title (2.40 mm): it buys
  back a row. ~~It pushes the `sh@5.0` block below the screw-hole band.~~
  **CORRECTED 2026-08-05 (R2/R3 measurement):** it does not. `sh@5.0` starts at
  6.800 mm titled and 3.000 mm untitled, and BOTH are inside the 10 mm band —
  its first row is narrowed either way (`[20 …]`). What the title moves is the
  SECOND row, from 8.000 mm to 11.800 mm, which clears. The inversion is also
  **front-only**: the untitled back has 5.400 mm of spare against the titled
  2.400 mm.
- ~~**The band affects only the FIRST block.** Everything below the top band
  takes `ceil(95/CharsPerLine)` rows exactly.~~ **FALSE — corrected 2026-08-05.**
  The BOTTOM band bites too, and on both sides it lands inside the LAST block:
  measured budgets are `[31 31 31 25]` for `const@3.8` on the front and
  `[39 31 31]` for `const@3.0` on the back. Neither narrowing changes a row
  count in this composition, so the totals stand — but that is a **measured
  coincidence, not a rule**, which is why `SPEC_sizeproof.md` §7.3 pins the
  per-row budgets and not just the row counts.
- For a uniform plate, a running sum of `params.F(size)` and
  `margin + row*fontSize` are **exactly equal** — every rung converts exactly at
  MM=6400. So no golden moves, **provided the accumulator is in device units**.
- **Row count consumes the plate, not character count.** 8 characters cost
  exactly what 16 do at every rung, because both need one row per face.
- Two-sided is an operator flip and two plate programs; the firmware needs no
  concept of a side.

## 4. Two mistakes made this session — do not repeat

**I asserted an unverified number in a section labelled MEASURED.** I claimed
"six of ten (face, rung) pairs need a row more than the naive count". Measured,
**zero of ten** do in the titled configuration and exactly one does untitled.
My probe computed each pair as if it started at the plate top; only the FIRST
block does. RECON's original 71.6 mm content height was correct all along
(71.6 + 3.0 margin + 3.8 title = 78.40). I made this claim while correcting
someone else's records. **Verify before writing a number down, especially when
correcting.**

**I destroyed uncommitted work with `git checkout <file>`** while reverting a
mutation test. Reconstructed from history, but it cost a detour. **Revert
mutations from a copy you made first, never from the index.**

## 5. Tooling

- Go only via nix: `export PATH=/nix/var/nix/profiles/default/bin:$PATH` then
  `cd <repo> && nix develop --command <cmd>`.
- **Render any plate headlessly:** `go run ./cmd/plateview -plate bothproof
  -size 4.4 -o /tmp/p.png`; `-list` for the six plate types.
- **Run the firmware in a browser:** `./cmd/emu/build.sh`, serve `cmd/emu`,
  open it.
- `go generate ./font/constant/` after any `constant.svg` edit — the compiled
  `constant.bin` is what gets engraved, and a test asserting svg coordinates
  passes against a stale bin.

## 6. Also open

- **Flash `fork-v0.0.0-g3c3a2ad`** when the device is next in BOOTSEL.
- **`FONTPROOF!` → `PASSPROOF!` rename** — recorded in
  `LEXICON_proof_triggers.md` as open and operator's-call. It is not
  theoretical: the operator called the free-text proof "FONTPROOF!" repeatedly,
  and typing that opens the passphrase program.
- **Glyph test coverage sweep.** `f` had NO test — the whole glyph could be
  replaced with an X-shaped scribble and the suite stayed green, because plate
  goldens only cover glyphs their own text contains. Other glyphs almost
  certainly share the gap.
- **`f` top bar at 1.62 units** is below the 2-stroke-width rule, accepted
  deliberately and pinned by `TestFTopBarIsShortByDesign`. **Untested on steel**
  — the next BOTHPROOF! plate settles it.
- 10 stale worktrees under `/scratch/code/shibboleth/seedhammer-wt-*`; most are
  merged and removable. `seedhammer-wt-emu` holds `proof-sizes-and-equals`,
  which IS merged into main.

## 7. Governing conventions

- `design/LEXICON_proof_triggers.md` — trigger naming grammar. Root names an
  axis; a parameter slot holds one kind of value.
- `CLAUDE.md` — the risk set, the R0 gate, reviewer model tiering (sonnet for
  mechanical, opus for design, fable reserved for a single pre-irreversible
  gate; **fable is still unspent**).
