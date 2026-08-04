# Fable architect — whole-feature integration review

- **Diff:** `b52407d..HEAD` on `bip39-password-phaseA` — 37 files, +4541/−132, 36 commits, all four phases assembled
- **Brief:** the first review to see the feature assembled rather than one phase at a time. Does it contain a defect the four per-phase reviews structurally could not see?
- **Tier rationale:** one of two fable runs authorised for this feature. Justified by precedent — Phase D found a Phase B defect that Phase B's own review missed, because D was the first real consumer of B. There is no phase above D.
- **Date:** 2026-08-03

## VERDICT
**NOT GREEN (0C / 3I)** — plus 2 Minor, 3 Nit.

No Critical: no path found by which a wrong character reaches metal or a secret leaks; full suite green. **All three Importants are seam-class — guards the per-phase reviews recorded as folded that assembly shows do not actually guard. Each proven by mutation in a scratch copy.**

## I1 — The whole-charset no-panic guarantee stops one layer above the panics it exists to catch
**Where:** `backup/passphrase_test.go:714-737`, drain at `:733-734`
Iterates the raw command stream (`for range eng {}`) but never runs `engrave.PlanEngraving`. The panic class §3.4 checks 7–8 names as the dangerous one — `unaligned delay`, `scale already in effect`, `delay during spline`, which "survive a green test suite and crash the device mid-plate" — fires **inside the planner**, which this test never reaches. Full-charset planning exists only at em = 4 mm in `engrave/`; full-plate planning at the real ems (6 / 4.5 mm) covers only the goldens' narrow character sets. **Nothing plans the full charset through the real entry point at the real ems.**
**Proven:** injected a plan-time-only defect (Delay denominator +1 for `'='`). The **entire backup suite including this test, and the entire gui suite, stayed green**; only engrave's 4 mm test caught it. On device that is a firmware panic in `ppBuildPlate` for any passphrase containing `=`, after confirm. A variant born of font/em rounding invisible at 4 mm would ship.
**Fix:** drain `engrave.PlanEngraving(conf, eng)`, both QR states.

## I2 — The Phase D M2 regression test is vacuous; Back-preservation is unguarded
**Where:** `gui/passphrase_flow_test.go:1215-1228`; behaviours at `gui/passphrase_flow.go:222`, `:359-361`
Sets `cs.choice = 1` then asserts `cs.choice == 1`, and asserts `GroupFingerprint("DEADBEEF") == "DEAD BEEF"`. **Neither assertion invokes `ppQRChoiceFlow` or `fingerprintEntryFlow`.** Deleting **both** M2 fixes leaves the whole gui suite green. The fold is real in code but pinned by nothing; silent data loss on Back — in a flow whose stated promise is that Back never abandons — can regress silently.
**Fix:** drive the real flow: enter a fingerprint, opt into QR, Back from Confirm, forward again, assert the confirm screen still shows both.

## I3 — The counter-occlusion fix cites a test that does not exist
**Where:** `gui/passphrase_flow.go:106-114`
The comment ends "`TestPassphraseEntryFitsPanel` measures rectangles instead." **No such test exists anywhere in the tree.** Reverting the fix leaves the gui suite green — as the comment itself predicts, since `ExtractText` collects runes regardless of occlusion, so the `101/100` assertion passes even when the counter is hidden. The occluded element is the over-length signal and live counter, in exactly the revealed proof-reading state, on a permanent-medium flow.
**Fix:** write the promised test, or at minimum correct the comment so maintainers do not believe the property is pinned.

## M1 — The confirm screen cannot catch compensating space↔underscore swaps
Intend `a_b c`, type `a b_c`: both render `a_b_c`, both count "5 chars, 1 space", both show the legend — two wallets, one screen. **The mitigation is stronger than it looks**: every *single* substitution error changes the space count and IS caught; only compensating double errors are invisible, and the plate (different glyphs) plus the opt-in QR disambiguate after the fact. Within the spec's accepted bitmap-font limitation. Cheap hardening if wanted: name space positions ("1 space at 4").

## M2 — QR quiet zone below the 4-module convention in the worst case
Worst case (dim 37, legend+footer): 2 mm above the envelope, bottom band ~1.75 mm below — under the ISO 4-module (4 mm) quiet zone. Precedent: existing seed plates are equally tight and scan. **O1 Plate A is exactly this worst case and mandates a scan** — attribute a scan failure to quiet zone before blaming module size.

## N1 — `engrave.go:420-426` guard comment says "raised from 33 to 41"; code enforces `dim > 37`.
## N2 — `TestConstantQRLargeVersionsFailClosed` references a "Phase B 78-char cap fallback" O6 resolved as unnecessary and never built.
## N3 — `engrave.go:770-775` repeats the "descenders move center" mechanism §3.5.1 measured as **false on this face**; `startEndDist` is the operative value.

## WHAT WAS CHECKED AND FOUND CLEAN
- **Raw-vs-substituted, every site.** Raw (`0x20`) → validation, secret buffer, counts, legend gates, QR encode. Substituted → confirm render (`'_'`), engraver text (`0x1F`), plate legend glyph. **No crossover anywhere, including error paths.** `TestPassphraseQRIsByteExact` decodes via a near-independent decoder; residual shared-layout risk is exactly what O1's real-scanner check closes.
- **Interface drift B→C→D:** `ValidateFingerprint` returns "" or exactly 8 canonical chars; `fingerprintEntryFlow` returns only canonical values, satisfying the precondition `backup.Passphrase` documents but cannot check. Entry copies ≤100 ASCII bytes into a 100-byte buffer. Back-navigation arithmetic verified for every step.
- **Spec arithmetic re-derived:** module pitch exactly 1.0 mm; 22.5+2+37 = 61.5 ≤ 65; 10×6 = 60 ≤ 65; 64 mm band cap = 32×2.0; alphabet 96 = 1+95; keyboard 95 = 26+26+10+32+1; 10 two-run glyphs; `runeDuration` 572245 set by `#` and pinned; 2L worst case 681 s.
- **Golden scope:** exactly the three `slip39-*` plus four new `passphrase-*`; seed/codex32 untouched, confirming the shared alphabet's artwork is unchanged (O5).
- **Per-run quantization seams:** Delay denominators and run segmentation derive from the same scaled geometry at the same em on both sides; the padding-covers-every-move invariant holds at every em by the bounding-box argument.
- **Menu wiring, wipe discipline, and the toPlate seam** (full plan runs at `ppBuildPlate` before the engrave screen with a bounds check, so layout overflow fails closed pre-metal).

**All three Importants share one shape: a fold recorded as done whose guard does not bite.** Each has a small mechanical fix. Nothing found requires design change.
