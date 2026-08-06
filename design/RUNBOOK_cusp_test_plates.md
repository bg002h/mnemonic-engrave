# RUNBOOK — the cusp test plates

**2026-08-06.** Firmware `v0.0.0-g23b171e`, engraving feed 4 mm/s, accel/jerk
stock. What these plates decide is in `design/RECON_cusp_dot_pileup.md`.

## 0. Before anything

Boot on **machine power**, not a laptop port. `Init()` checks for a 20–28 V
USB-PD contract *before* it configures the LCD and reboots into BOOTSEL when it
does not find one, so a laptop port gives a dark screen on firmware the bootrom
**accepted**. Expect `v0.0.0-g23b171e (UNLOCKED)`; the suffix is expected.

**Hard steel throughout.** The wiggle does not appear on soft.

## 1. The common sequence

Every cut is the same walk through **Engrave Text**:

```
Engrave Text
  QR Code   -> No QR
  Font      -> sh | constant
  Size      -> the rung
  Text      -> type ONE character, OK
  Speed     -> 4.0mm/s (default)        one entry unless a proof is loaded
  Title     -> OK on the empty field
  Footer    -> OK on the empty field
  Confirm   -> read it, then OK
```

The character lands **top-left, left-aligned**, and takes about **2 seconds**.
Reposition or swap the plate between cuts — they all land in the same place.

**Read the confirm line before every cut.** It states the rung, the line count, the
QR and the font. If it does not say what you chose, stop.

## 2. The four cuts

| # | Font | Size | glyph | what it decides |
| --- | --- | --- | --- | --- |
| 1 | `constant` | 3.0 mm | `~` | baseline — are the three corners blobbed? |
| 2 | `constant` | **6.0 mm** | `~` | **the falsifiable one** |
| 3 | `sh` | 3.0 mm | `~` | control — smooth spline, no full stops |
| 4 | `constant` | 3.0 mm | `O` | 9 cusps, worst in the face, not yet designed |

Cut them **in that order**. 1 and 2 are the pair that carries the argument; 3 is
what makes them interpretable; 4 is reconnaissance and can be skipped if the
first three settle it.

## 3. What to record

**Cut #2 needs a measurement, not an impression.**

> **Measure the corner blob's width in mm on #1 and on #2.**

The model says the pile is **absolute** — roughly two 0.3 mm strikes landing
inside 0.06 mm, set by the fixed 25 ms needle period, which does not scale with
the glyph. The glyph does scale: `~` is 1.33 × 0.67 mm at 3.0 mm and
2.67 × 1.33 mm at 6.0 mm.

| what you see | what it means |
| --- | --- |
| blob the same **mm** at both rungs, glyph twice the size | **confirmed** — a fixed defect that the glyph outgrows, which is exactly the size dependence reported |
| blob scales with the glyph | **refuted** — the cause is something that scales, and the cusp/dot-pile model is wrong |

For **#3** the question is binary: does the `sh` `~` show corner artefacts at
all? It has no commanded stops, so it should show none. If it does, the cusp
story is incomplete rather than merely imprecise.

For **#4** there is no prediction to falsify. It is worth knowing whether a
9-sided polygon is viable at 3.0 mm **before** `O` is designed.

## 4. The speed experiment (optional, new)

Only now that a feed is selectable. `RECON_cusp_dot_pileup` predicts dot spacing
is **feed × 25 ms and nothing else**: 0.20 mm at 8 mm/s, 0.025 mm at 1 mm/s.

```
QR Code  -> No QR
Font     -> (either)
Size     -> (either)
Text     -> type CONSTPROOF!  -> OK -> accept the prompt
            the field fills with the proof pattern and stays on this screen
         -> clear the field, type ~        (Back here reaches Size if you want 3.0mm)
         -> OK
Speed    -> now offers 8.0 / 6.0 / 4.0 / 2.0 / 1.0 mm/s
```

Cut `~` at **8, 4 and 1 mm/s** and measure the dot pitch along a straight
segment. Three points are enough to see whether it is linear in feed.

**Note the gate deliberately stays open.** Once a proof keyword has been
accepted, the feed stays selectable for the rest of that run even after the
pattern is deleted — which is what makes a single-character speed test possible.
It closes when the program exits. Seed, descriptor and passphrase plates are
different programs and never reach this screen at all.

## 5. Things that would invalidate a plate

- **A different feed than you think.** The confirm screen appends
  `speed: N.Nmm/s` only when it is NOT the default. No suffix means 4.0 mm/s.
- **Comparing against plates cut before today.** The feed changed from 8 to
  4 mm/s in `343fb05`, so dot appearance is not comparable across that line.
- **Judging the boot on laptop power.** See §0.
- **An input wedge.** F-58 is unreproduced and lives in this program. If a
  screen stops responding, power-cycle — and say so, because a second sighting
  is a real clue. Nothing is commanded before the engrave step, so no plate is
  at risk.
