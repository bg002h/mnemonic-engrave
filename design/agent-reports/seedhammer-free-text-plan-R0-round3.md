# Sonnet architect — R0 round 3 (final fold check), Engrave Text plan rev 3

## VERDICT
**GREEN (0C / 0I)** — **this closes the plan gate. Implementation may begin.**

## FOLD CHECK
**F1 fixed.** `Fit` now returns `qrc *qr.Code` alongside `fontMM, lines, err`. `qr.Code` is
`github.com/seedhammer/kortschak-qr`.Code — **a real exported type at `qr.go:84`, already
alias-imported by BOTH `backup/backup.go:11` and `gui/gui.go:22`** — so the value crosses the
package boundary as a plain value of an already-shared external type, not a `backup`-private one.
That is the mechanism the fix depends on, and it holds. `qrFor` stays private, called only inside
`backup`. D2a.2's call `backup.EngraveFreeText(params, fontMM, title, lines, footer, qrc)` matches
C2's declared signature exactly. D2a.4 closes the secondary blocker by relocating the module-level
assertion into a package-`backup` test where `qrFor` is visible.
Checked every other consumer: `Admissible` and `MaxCharsAt` never return or need a `*qr.Code` —
same-package internal calls, unaffected by the widened arity. Grepped the whole plan for `Fit(`
and bare `Fit`: the only uses are C1.2's declaration and D2a.2's prose, so **no task destructures
it under the old 3-value assumption**.

**F2 fixed.** `freetextPlateHook` is declared with exactly the shape `EngraveFreeText` receives,
and D2a.1a's assertion is now checkable against captured values rather than a `bspline.Curve`.
Verified the cited idiom is real and pervasive: **seven existing hooks of this exact shape**
(`passphraseSecretHook`, `bip85SeedHook`, `multisigSeedHook`, `singleSigSeedHook`,
`buildMultisigSeedHook`, and two more), every one `nil` in production with an
`if xHook != nil { xHook(args) }` call sited where the value becomes available. D2a.1a drives the
real flow end to end — typing, Confirm, Engrave — so it is no longer "trivially true by
determinism", which was F2's original objection.

## NEW FINDINGS
**None.** Two sub-Minor observations, neither gating nor worth a fold: Task D2a's file header omits
`backup/freetext_test.go` (where D2a.4's test lives), and D2a.3's parenthetical is leftover
phrasing that now reads oddly, though it correctly disclaims rather than misinstructs.

## WHAT I VERIFIED
Read plan rev 3 in full and round 2's review verbatim. Grepped every `Fit(`, `qrFor`, `qrc`,
`*qr.Code` reference for consistency. Against the real tree, read-only: `qr.Code` exported at
`kortschak-qr@v0.3.2/qr.go:84`; both packages already import `qr`; `passphraseSecretHook`'s real
call site at `gui/passphrase_flow.go:590-591` plus six siblings across `bip85.go`, `multisig.go`,
`multisig_build.go`, `singlesig.go`. Confirmed no name collision — `freetextPlateHook`,
`ftBuildPlate`, `EngraveFreeText` exist nowhere yet. Tree untouched.
