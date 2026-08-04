# Sonnet architect — R0 round 2 (fold check), Engrave Text plan rev 2

## VERDICT
**NOT GREEN (0C / 2I)** + 1 Minor.

## FOLD CHECK — all three fixed
- **NEW-1 fixed.** Task D2a's `ftBuildPlate` produces every argument
  `EngraveFreeText` takes and hands the result to an engrave screen, mirroring the
  named precedent. The structural gap is closed.
- **NEW-2 fixed.** Measured: `C1.7` no longer exists as a checklist item — the only
  hits are historical backrefs inside C2.4's parenthetical. C1 now runs
  C1.1, C1.2, C1.2a, C1.3-C1.6, C1.8-C1.10. C2.4 is the sole home of the bounds test.
- **NEW-3 fixed.** C1.2a states three distinct fail-safe modes — `Fit` errors,
  `Admissible` returns `ok=false` with `linesAvail` still meaningful, `MaxCharsAt`
  returns 0, none may panic — and `ftBuildPlate` propagates. `ftBuildPlate`'s own
  2954-byte path is unreachable in practice because `Admissible` gates OK before
  Confirm→Engrave, so its absence is not a gap.

## F1 (Important) — `qrFor` is unexported; `gui.ftBuildPlate` cannot call it
C1.2a declares `func qrFor(...)` in package `backup`; D2a.2 requires
`gui/freetext_flow.go` to call it. **An unexported identifier is invisible outside its
package regardless of qualification** — `backup.qrFor` does not compile. Verified this is a
boundary the codebase already respects: `grep -rn "backup\.[a-z]" gui/*.go` (excluding tests)
returns 3 hits, **all inside comments**, never a call site; every real cross-package call uses an
exported name. The same problem independently blocks **D2a.4** — comparing "the fit path's" code
from a `gui` test is impossible, since `Fit` never returns a `*qr.Code` and `qrFor` is unreachable.
The plan as literally written does not compile, and two checklist items depend on the fix.

## F2 (Important) — D2a.1's test claim is not checkable against what `ftBuildPlate` returns
`ftBuildPlate` returns `(Plate, error)`; `lines` is internal and never surfaces. `Plate`
(`gui/gui.go:470-473`: `{Duration uint; Spline bspline.Curve}`) is **stroke geometry with no
text** — `op.Drawer.ExtractText` operates on rendered GUI ops, never on `engrave.Engraving`, so
"which strings were engraved" cannot be recovered from a curve. Comparing two independent `Fit`
calls is trivially true by determinism and does not test that the flow's Engrave step used the
confirm-approved values. Closing it needs a test seam no task names — e.g. a hook capturing the
lines handed to `EngraveFreeText`, mirroring `passphraseSecretHook`. As written an implementer can
satisfy the letter of D2a.1 without binding confirm-displayed layout to the engraved one — the
same class of gap NEW-1 was raised to close.

## MINOR
- D2a.3 attributes the ECC-mutation catch to D2a.1, but `ftBuildPlate` passes `Fit`'s `lines`
  through unmodified, so a wrong ECC level in the builder changes only `qrc`, not `lines`. The
  mutation is caught by D2a.4, not D2a.1.

## VERIFIED BY MEASUREMENT
HEAD `abb7458`, clean, no drift. Read `ppBuildPlate` (`gui/passphrase_flow.go:494-556`), the flow
dispatch (`gui/gui.go:1494-1540`), `type Plate` (`:470-473`), `toPlate` (`:2833-2844`) and
`engrave.Engraving = iter.Seq[Command]` (`engrave/engrave.go:55`) to confirm `Plate` carries no
text. `grep -rn "backup\.[a-z]" gui/*.go` → 3 comment-only hits (basis for F1). `ExtractText`
confirmed to operate only on `op.Drawer` (basis for F2). Confirmed none of the planned symbols
exist yet — this is plan-consistency analysis, not a test run.
