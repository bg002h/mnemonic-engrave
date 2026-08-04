# Opus architect — Phase C post-implementation execution review

- **Diff:** `b68208a..bip39-password-phaseC-plate` (5 commits)
- **Scope:** `backup.Passphrase`, `EngravePassphrase`, both layouts, metadata bands, legend, the new QR decoder, 4 new goldens
- **Date:** 2026-08-03

## VERDICT
**GREEN (0C/0I)** — 2 Minor, 4 Nit. Full suite green; no input found that breaks a stated guarantee.

## M1 — `TestPassphraseEngravesNoLiteralSpace` tests none of what its name claims; the implementer's "hardened" report is wrong
**Where:** `backup/passphrase_test.go:96-100`
The committed test is **byte-identical to the plan's version** — it only calls `passphraseGlyphs("a b c")`, a pure function already exhausted by `TestPassphraseSpaceSubstitution`. It never touches the engrave path. The implementer did not harden it; it added a *separate* test (`TestPassphraseEngravesMarkNotSpace`) and left this one as-is.
**Confirmed by mutation:** replacing `l.glyphs` with `plate.Passphrase` at `passphrase.go:241` leaves this test **passing**.
**Not blocking** — the property is covered by `TestPassphraseEngravesMarkNotSpace`, `TestSpaceFidelity` and three goldens. Answering the brief directly: **there is no route by which a real `0x20` reaches the passphrase text block.** `engravePassphrase` reads only `l.glyphs`; the QR reads only the raw string. And the detector is genuine — `0x20` has advance 600 and **zero spline knots**, so a leaked space really would engrave nothing.
**Fix:** delete as a duplicate, or rewrite against `passphraseTextInk`.

## M2 — `EngravePassphrase` trusts an unvalidated fingerprint precondition; `GroupFingerprint` fails open
**Where:** `backup/passphrase.go:176-181`; `passphrase/passphrase.go:90-95`
`SeedFP`/`CombinedFP` are `string`, documented "canonical 8-hex-digit or empty", but nothing on the plate path enforces it, and `GroupFingerprint` returns non-8-char input **unchanged** rather than erroring.
**Measured:** a 32-hex-digit `SeedFP` yields a top-band line at **82 mm** — over §4.3's 64 mm cap and into both corner screw-hole bands. Silent; no error, no panic. The existing `Seed` plate is immune only incidentally (its fingerprint is a `uint32` formatted `%.8X`).
**Fix:** **Phase D must route both fields through `passphrase.ValidateFingerprint`** before constructing the plate. **Owning phase: D.** Defence-in-depth alternative: `EngravePassphrase` returns an error on non-canonical input.

## N1 — the QR decoder verifies neither ECC codewords nor pad bits
Flipping every module one at a time: **0 of 56** CHECK-region flips noticed (deliberate, documented) and **84 of 152** DATA-region flips silently ignored — exactly the terminator + `0xEC`/`0x11` padding beyond the segment payload, which `qrSegments` never reads. So the round-trip pins the segment header and payload, not the whole codeword stream.
**Why a Nit:** the gap was closed independently — the geometry-recovered grid reproduces **every module** of the encoder bitmap: 441/441, 625/625, 1089/1089, 1369/1369 across `hunter2`, spaces, edge spaces, 100×`a` (dim 37), 100×`A` (dim 33), full ASCII and numeric. **Optional fix:** one assertion, `g[y][x] == code.Black(x,y)`.

## N2 — the bottom band's inter-line clearance is unasserted
Legend ink `y=[75.667, 78.000]`, footer ink `y=[78.667, 80.667]` — **0.667 mm**, two stroke widths, ~0.333 mm of clear metal after the stroke. **Not a collision under any input**: both strings are compile-time constants, ordering is fixed, fingerprints only affect the top band. But raising `plateSmallFontSize` or deepening the mark's descender would collide with no test failing.

## N3 — `TestSpaceFidelity` only exercises the no-QR layout
The 20-char-per-row QR layout's glyph placement is pinned **only** by goldens. Reverting `l.rowLen` to `groupLen` — the exact mistake the parameter exists to prevent — is caught by `passphrase-1-qr`/`passphrase-3-max-qr` and nothing else.

## N4 — dead store
`passphrase_test.go:349` stores `prevText.Y`; only `.X` is read.

## WHAT WAS CHECKED AND FOUND CLEAN

- **The QR decoder is correct**, verified line-by-line against `kortschak-qr@v0.3.2/coding` rather than against the encoder: unmasking (plan-black = mask bit, code-black = mask⊕data; Extra/remainder correctly treated as fixed); bit order mirrors `Encode`; `Pixel.Offset()` indexes the **de-interleaved** stream so `raw[:DataBytes]` is right; mask recovery is unique (15-bit BCH, min distance 7) and fail-closed; segment parsing matches the QR spec for modes 1/2/4 and v1-9 count widths. **No compensating bug is possible** — the recovered grid provably equals the encoder bitmap module-for-module.
- **`stringColumn`'s `rowLen` change is inert.** All three pre-existing call sites pass `groupLen`; no other callers; every `seed-*`, `codex32-*`, `text-*`, `slip39-*` golden byte-identical.
- **Bands.** ≤2 lines is structural. Footer measures **exactly 64.000000 mm** — safe at the limit: ink spans x=[10.833, 74.167], clearing the corner bands by 0.667 mm, and one more character trips three tests and three goldens.
- **Layout holds over the whole valid input space**, not just samples: every printable rune × {1,100} chars × {QR,no-QR} × {with,without fingerprints}, all 95 rotations of the full charset at 100 chars, and every length 1–100 — exactly-one-region per knot, no corner-band knot, ≤2 lines/band, no overlap.
- **Nine mutations, all caught by behavioural assertions** (not only goldens), including raw-string engraving, QR encoding the mark, ECC L→M, mark as fingerprint separator, band moved into the usable area, QR not centred.
- **Geometry matches the corrected numbers:** stroke 0.33328 mm, QR pitch 0.99984, 37-module envelope 36.994 mm, QR layout 61.494 mm inside 65 (3.506 slack, matching §4.2's "3.5 mm").
- **Existing-output invariance across the whole feature** (`b52407d..phaseC`): only the 3 accepted `slip39-*` and the 4 new `passphrase-*`. Goldens created once, never rewritten; `CompareBSpline` errors on a missing file so they cannot silently no-op.
- **`qr.Encode` honours ECC-L** and a 100-char byte-mode passphrase is 812 bits ≤ v5-L's 864, so the 37-module ceiling cannot be exceeded by any `ValidatePassphrase`-accepted input.
