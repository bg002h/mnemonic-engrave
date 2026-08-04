# Sonnet architect — R0 round 1 (fold check), Engrave Text plan rev 1

## VERDICT
**NOT GREEN (0C / 3I)** — no Criticals.

## FOLD CHECK — C1, I1-I6, M1-M4, N1-N2 all FIXED; C2 PARTIAL
- **C1 fixed.** B3.2a expresses exactly the reading that passed round 0's measurement, with the
  same 45281/45282 knot-mismatch and (6.450, 2.300)mm figures.
- **C2 partial.** `EngraveFreeText` exists with goldens, verbatim title/footer, inset-span
  centering and mutation checks — the core defect is addressed. Two gaps remain (NEW-1, NEW-2).
- **I1 fixed.** Measured: `grep -rln` for the six test names returns exactly six distinct files
  (`multisig_program_test.go`, `bip85_program_test.go`, `bundle_program_test.go`,
  `singlesig_program_test.go`, `derive_xpub_program_test.go`, `start_screen_touch_test.go`).
- **I2 fixed.** All four sites verified at HEAD: enum `:147-158`, flow dispatch `:1506-1533`
  (**no default** confirmed), title switch `:1676-1691` (**no default** confirmed),
  `layoutMainPlates` `:1893` (`panic("invalid page")` confirmed). D3.2a adds the press-OK test.
- **I3 fixed.** Reading (a) stated explicitly.
- **I4 fixed as scoped to `Fit`.** The hazard resurfaces unpatched in two functions the fold
  introduced — see NEW-3.
- **I5 fixed.** Three entry points, non-overlapping; C1.6/C1.8 test them, D2.2/D2.3/D2.7 consume them.
- **I6 fixed.** Concrete before/after dump comparison with a committed hash record.
- **M1 fixed.** `fixedCharWidth`'s body compiles against the real
  `vector.Face.Decode(rune) (int, UniformBSpline, bool)` / `Metrics()`, and is line-for-line the
  existing `charWidth` at `backup/backup.go:288-292`, so it reproduces the pinned grid.
- **M2/M3/M4/N1/N2 fixed.**

## NEW-1 (Important) — `EngraveFreeText` is defined but never called, and its `qrc` has no producer
No task in Phase D invokes it. D2's sub-tasks cover confirm behaviour, admission, capping and
back-preservation, but none assembles `title, lines, footer, qrc` into the `engrave.Engraving`
that step 6 of spec §7 needs. The established precedent is `gui/passphrase_flow.go`'s
`ppBuildPlate` (`:532`) feeding `NewEngraveScreen(ctx, plate).Engrave(...)` (`:645`); free text has
no analogue. Compounding it, **nothing produces the `*qr.Code`**: `Fit` returns
`(fontMM, lines, err)`, `Admissible` and `MaxCharsAt` return neither, and C1.3 uses
`qr.Encode(...).Size` only for module count, discarding the code. The implementer reaches final
assembly with no task saying to encode again, at what level, or where that belongs.

## NEW-2 (Important) — C1.7 was declared moved but is still present, duplicating C2.4
C2.4 says "(moved here from C1.7, which had no code under test)". Measured: C1.7 is still at line
284, word-for-word, inside Task C1 — before `EngraveFreeText` exists. This reproduces round 0's
original complaint exactly, as a leftover the fold failed to delete.

## NEW-3 (Important) — `Admissible`/`MaxCharsAt` inherit the `qr.Encode` hazard I4 closed on `Fit`
Verified against `kortschak-qr@v0.3.2/qr.go`: `Encode` returns `(nil, error)` at 2954+ bytes. C1.2
gave only `Fit` an `error`. But D2.2/D2.3's live per-keystroke readout is precisely `Admissible`'s
output shape, and C1.3 requires re-encoding the **explicitly unbounded** text on every keystroke
when a QR is chosen — the identical reachable-input condition used to justify patching `Fit`.
Neither `Admissible` (`… ok bool`) nor `MaxCharsAt` (`int`) has an error channel, and no task says
how either absorbs a failure rather than dereferencing a nil `*qr.Code`.

## VERIFIED BY MEASUREMENT
Repo at `abb7458`, no drift. The four `gui.go` sites read directly, both switches confirmed
default-less, `layoutMainPlates` confirmed panicking. Six test files confirmed by grep.
`fixedCharWidth` checked against the real `vector.Face` API. `cmd/vectorfont`'s flag contract
against A1.6's corrected command. `kortschak-qr`'s `Encode` failure path. `bspline.Measure` exists
at `bspline/bspline.go:194`. Grepped the plan for `EngraveFreeText`, `qrc`, `Fit(`, `Admissible`,
`MaxCharsAt` — zero call sites in Phase D (NEW-1) — and for `C1.7`/`C2.4` (NEW-2).
