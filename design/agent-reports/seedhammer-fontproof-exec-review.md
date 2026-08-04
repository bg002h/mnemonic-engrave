# Opus architect — FONTPROOF! post-implementation execution review

- **Scope:** `gui/passphrase_fontproof.go` (+ test), the FONTPROOF call sites in `gui/passphrase_flow.go`, and the interaction with `868bf3c` (the keyboard `MaxHeight` clamp) which landed in parallel against the same file.
- **Brief:** does this diff contain a defect reaching the operator's secret, what gets cut into steel, or a guarantee the flow makes — including one created by the parallel landing? Explicitly NOT a fresh audit of phases A–D.
- **Method (binding):** every finding proven by mutation or measurement, never by reasoning; work in a scratch copy; a surviving mutation is itself a finding.
- **Date:** 2026-08-04

## VERDICT
**GREEN (0C / 0I)** — 3 Minor. ~30 mutations applied, all killed except the two below.

## M1 — the prompt tells the operator something untrue in two of the three fields
`passphrase_fontproof.go:99-100`. The declining branch read *"Back = no: continue with FONTPROOF!
exactly as typed"* everywhere. True only in the passphrase field; in either fingerprint field
`ValidateFingerprint` refuses it and asks again. The project's own
`TestFontProofNoBranchInFingerprintFieldRefuses` asserts that contradiction. Nothing wrong is
engraved — but this is the one prompt whose entire justification is honesty. **FIXED** (per-field
wording + `TestFontProofKeepLineMatchesTheField`).

## M2 — nothing pins that the plate carries only `secret[:n]`  *(FILED, not fixed)*
`passphrase_flow.go:638`. The loader sets `n = 95` into a 100-byte buffer, so a longer prior
passphrase leaves a **printable** tail in `secret[95:100]`. Shipped code is correct everywhere.
**Proof:** mutating `ppBuildPlate(…, secret, …)` left `go test -run TestFontProof` **green**; the
full-suite kill came only from an unrelated test panicking on `\x00`, an accident that would not
fire in the case FONTPROOF uniquely creates. Measured on the production flow: 100×`Z` typed,
pattern loaded from Seed FP → residual `"ZZZZZ"` present at confirm. The confirm *screen* is
guarded semantically; the *plate* path is not. Hazard predates this work (backspace 100→10 does
the same).

## M3 — the yes/no icon binding was unpinned
`passphrase_fontproof.go:141-144`. `layoutNavigation` places a button by `Clickable.Button` and
draws whatever `Icon` it was given; every test taps by tag, never by glyph. **Proof:** swapping
`IconBack`/`IconCheckmark` left the **entire gui suite green** while inverting the prompt for the
only party who reads it. **FIXED** (`ppFontProofNav` + `TestFontProofPromptIconsNotSwapped`).

## CHECKED AND CLEAN (selected)
- **Trigger scope:** substring-instead-of-equality, per-keystroke, leaked into `PassphraseKeyboard.Layout`, into `typeAddressFlow`, into `bip85IndexEntryFlow` — all killed. The two shared-consumer **negative** assertions were verified to bite for the right reason (not vacuous under the `uiContains` needle-space trap), including at the 240×240 default display.
- **Loader:** skipping any of the three writes, breaking the wipeable aliasing by allocating a fresh buffer, both fingerprints wired to one constant, a missing glyph in the pattern, and `MaxLen` lowered so the pattern silently truncates — all killed.
- **Stay-on-screen:** `continue` removed in either YES branch, field not reseeded, fingerprint seeded with the other field's value, `nil` loader to each step — all killed.
- **Parallel seam:** `kbd.MaxHeight = 0` still fails after the merge and the `9e17134` reconciliation, so the clamp's guard survived integration. Pattern loaded = **228px against the 245px bound**, identical masked and revealed, counter `95/100`; strictly less demanding than the 100×`W` revealed case already covered.
- **Scenario probes on the real flow:** trigger from Expected Comb FP → confirm and plate agree, engrave completes, `secret` **fully zeroed** including the residual; Back paths from Comb FP and Seed FP both preserve all three fields; confirm body 417×227 in a 417×270 area with the 95-char unbreakable token, QR on and off.
- **Step arithmetic:** every `step -= 2; break; step++` path traced to a net −1; none can go negative because `ppStepEntry`'s Back returns.
- **Event double-consumption:** the asymmetric yes/no cannot slip a step; `EventRouter.Next` consumes from one ordered queue and `PointerFilter` is tag-scoped, so the prompt's YES cannot also fire the field's OK (both `Button3`).
- **Secret hygiene:** no path from the passphrase into an error string, log, or NFC on any FONTPROOF branch; `wipeBytes` is the last defer and zeroes all 100 bytes on both completed-engrave and mid-flow exit.
- **Two survivors correctly dismissed rather than counted:** a quadrupled prompt body (genuine slack — 167px measured against 270px, and the fit test does go red at +6 repeats) and an equivalent mutant (offer moved into a branch `FONTPROOF!` always reaches anyway).

## FOLD RECORD (2026-08-04, commit `242c9f1`)
M1 and M3 fixed; both verified by re-running the exact mutations that had survived — icons
swapped now fails, collapsing the two sentences now fails. M2 filed to `design/FOLLOWUPS.md`.
Full suite green by exit status, 61 packages, no golden changed.
