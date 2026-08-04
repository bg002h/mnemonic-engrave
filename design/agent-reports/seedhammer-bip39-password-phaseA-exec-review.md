# Opus architect — Phase A post-implementation execution review

- **Diff:** `b52407d..HEAD` on `bip39-password-phaseA`, excluding the Phase B merge
- **Scope:** font extension (44 glyphs + 4 redraws), generator control-codepoint support, per-run quantization, constant-time QR to v5
- **Date:** 2026-08-03

## VERDICT
**NOT GREEN (0C / 1I)** — plus 4 Minor, 3 Nit.

Core of Phase A is correct. The reviewer independently re-derived the run partition for all 96 glyphs, proved emitted geometry identical to `engrave.String`'s, mutation-tested seven ways, and re-verified the v5 QR extension by reconstructing 500 dim-37 codes.

## I1 — The `#` redraw made a redrawn glyph the longest single run; §3.5.0's I2 budget is violated and the record misattributes it

**Where:** `font/constant/constant.svg` (`id="hash"`, `5f667dd`); constraint at spec `:398-409`; stale record at `:566`.

§3.5.0 states normatively: *"The redrawn glyphs MUST NOT become the longest single run. Re-measure `runeDuration` after the redraws and restate the worst case in absolute time."*

Measured (same params, `em = 4*mm`):

| | at `2796b2b` (pre-redraw) | at HEAD |
|---|---|---|
| `runeDuration` | **533 764**, set by `'8'` | **572 245**, set by `'#'` |
| one padded unit | 615 558 ticks = 3.206 s | 654 039 ticks = 3.406 s |

`#` overtakes `'8'` by 7.2%. Every run is padded to `runeDuration`, so this inflates **all 96 glyphs**: +0.200 s per slot, +20 s on a 100-char passphrase at k=1, +40 s at the 2L worst case.

Two aggravating facts: the mandated re-measure/restatement **does not exist** (`TestPassphraseStringerTiming` only `t.Logf`s aggregates and disclaims proving anything); and **the spec's record hides it** — §3.5.1:566 lists `runeDuration | 533764 | 572245` under *"Measured … (Phase A Task 4, 2796b2b)"*, but `2796b2b` measures 533 764. The two alphabets were **identical** until `5f667dd`. Nothing in FOLLOWUPS tracks it.

**Failure:** no wrong output, no timing leak — the bound holds *in units*; the unit grew. A normative MUST NOT is unmet, the verification that would have caught it was skipped, and the artifact that should record it conceals it.

**Fix:** (a) iterate `#` to shed ~7%, or (b) amend I2 to accept the measured value, correct §3.5.1:566's attribution, restate the worst case in absolute time (`2L × 654 039` = 681 s at L=100, 192 000 ticks/s), and **pin `runeDuration` and the setting glyph in a test** so the next font edit that inflates it fails a test rather than a review.

## M1 — D5's "provably unaffected" argument is false as written
Three pre-existing glyphs were modified: `#` (`5f667dd`, accepted exception), `*` (`381364a`, mandated by §3.5.0), `@` (`2c2f569`, artwork genuinely changed on user request). D5 argues the change is "confined to codepoints that previously had no glyph". **The guarantee holds in fact** — verified `constant.Font` reaches engraved output through exactly three constructions (`gui.go:486` empty title, `slip39_polish.go:492`, `codex32_polish.go:229` bech32 id), none of which can contain `*` or `@`; `TitleString` has no production callers; text plates use `sh.Font`. The *argument* is wrong, not the conclusion. Correct D5 to enumerate the three exceptions with their reachability audit.

## M2 — `FuzzConstantQR` never reaches an ECC-L dim-37 code
With entropy ≥ 61 bytes the ECC-Q encode lands at dim ≥ 41, errors, and `if qrcq.Size > 37 { return }` bails **before the L half runs**. ECC-L only reaches dim 37 at n ≥ 79. So the band the cap was raised to reach contributes one encode and an early return. dim 37 *was* measured via ECC-Q at n ∈ [47,60], so 664 stands. Reviewer closed the gap manually: 200 000 ECC-L strings of 79–106 chars, all dim 37, **zero** over-budget; 20 000-sample max = **652** against the 684 budget. **Fix:** skip only the Q half on error, and seed printable-ASCII 79–106-byte inputs.

## M3 — `ConstantQR`'s guard comment misstates the supported range
`engrave.go:420-426` says the guard was "raised from 33 to 41" and that `bitmapForQRStatic` supports "versions 1-6 (dims 21/25/29/33/37/41)". The guard is **37**, and the switch is `case 25, 29, 33, 37` — dim 41 hits `default: panic`. The false claim sits inside the sentence justifying the guard, so a reader who relaxes it to 41 gets the panic the guard prevents.

## M4 — `TestConstantQRLargeVersionsFailClosed` always skips
dim 41 is unconditionally rejected, so the test always takes `t.Skipf` and asserts nothing beyond "did not panic". The truncation property *is* fail-closed (`ConstantQR:494-497` errors; `modules` is append-grown) — the test just doesn't say so.

## N1 — every `engrave.go` citation in the three new test files is stale
`coverage_test.go:9` cites `:1365` (blank); `glyph_rules_test.go:22` cites `:1216-1218` (actually `:1279`); `passphrase_alphabet_test.go:16-17,44-45` cite `:1208-1210`/`:1215`/`:1218` (actually `:1271`/`:1276`/`:1279`). Only `cmd/vectorfont/main.go:414-428` is right.

## N2 — `TestNoGlyphStartsAtOrigin` guards an obsolete requirement
`2796b2b` deleted the `inf.Start != (bezier.Point{})` sentinel; `runSplitter` skips structurally. Harmless, but its rationale is stale — and if it *did* apply, §3.5.0(iii) extends it to every run's start while the test checks only the glyph's first knot.

## N3 — `PROVISIONAL (Task 5, spec O6)` markers and fuzz instrumentation left in shipped code
`engrave.go:363,408,420` and `engrave_test.go:432,449,462`; O6 is RESOLVED as of `c530e07`.

## WHAT WAS CHECKED AND FOUND CLEAN

- **Run partition, all 96 glyphs**, re-derived independently from raw knot dumps. Multi-run set exactly `! " $ % : ; = ? i j`; zero-run `' '`; 85 single-run. Max k = 2.
- **`runSplitter` reimplemented independently** — emitted knots are exactly the full list minus each run's leading triple, dropped knots are 3 contiguous at one control point, order strictly increasing, nothing emitted twice or lost. Clean on all 96.
- **Geometry equivalence, the check the suite lacks:** passphrase stringer vs `engrave.String` bounding box and total path length — **byte-identical for all 95 drawable glyphs**.
- **The removed sentinel is equivalent** for every glyph including the new ones.
- **The one undocumented structural assumption** (each run's leading triple is `F,F,T`, i.e. runs begin with a straight segment) **fails closed** — a synthetic curve-started run panics `scale already in effect` at engrave time, and `TestPassphraseEngraveAlphabet` engraves every rune individually.
- **Padding budget proved analytically**: `bounds` always contains the origin, so every inter-run, inter-glyph, opening and park move fits its pad.
- **Mutation battery, 7 mutations:** 5 CAUGHT (leading-triple knot kept → `unaligned delay`; closing knot dropped → `delay during spline`; glyph offset 1 unit → `unaligned delay`; `dot.X` per run → panic; inter-run pad ≠ `advDur` → fail; `hasMultiRun` guard removed → fail). 2 SURVIVED, both already-disclosed "untestable on this face" equivalences (bounds over first run only; `maxDur` over first run only). **No undisclosed false-PASS.**
- **Zero-run path** drives the real `paddedString`, not a parallel one, and asserts three non-vacuous properties.
- **QR:** alignment placement proved by reconstructing **500 dim-37 codes exactly**; 664+20 bound holds (max 652 over 220 000 samples); fail-closed intact.
- **Font:** 95/95 + `0x1F`; all 96 advances 600; alphabet 96 runes ascending from `0x1F`; shared alphabet single-run.
- **Existing-output invariance:** exactly three files changed under any `testdata/` in the whole range, all `slip39-*`, all in one commit. Scope claim accurate.
- **`constant.bin` byte-reproducible** from the SVG via `go generate`.
