# Firmware PR2 (#35) — final architect review — R0

- **Stage:** post-implementation FINAL whole-diff review (the deferred `firmware-deferred-formal-reviews` item (a)); run once Agent-API subagent dispatch recovered.
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-reviewer`
- **Scope:** `git diff upstream/main..feat/engrave-mdmk` (`801cf1c` + `5e2336e`) in the `bg002h/seedhammer` fork = upstream PR #35.
- **Verdict:** **NOT-GREEN — 0 Critical, 1 Important, 3 Minor.** (Inline self-review + parity test had passed; the formal review caught the case-tolerance gap.) Fold + re-review required.

---

## VERBATIM REVIEW OUTPUT

# Adversarial FINAL Architect Review: PR seedhammer#35 (`feat/engrave-mdmk`)

## Strengths
- The hardest correctness risk — wrong initial residue — is genuinely closed. `unpackSyms(0, mdmkPolymodInitLo, 13)` reproduces codex32's value-1 residue convention (`[Q×12, P]`), so POLYMOD_INIT is packed in the identical MSB-first coefficient layout the engine expects. Verified by hand and guarded by Rust-sourced golden vectors.
- `unpackSyms` shift logic is sound across all three branches for both n=13 (65-bit) and n=15 (75-bit), including the `hi<<(64-shift)` straddle and the `shift>=64` high-symbol case. No overflow, no panic.
- `verifyMDMK` is panic-free on every malformed input traced: no separator, empty, `md1`/`mk1` with empty/short data, invalid bech32 runes, mixed case, 300+ char strings.
- Scanner branch ordering is correct and non-shadowing. md1/mk1 sits after `codex32.New`; an mk1 of total length 48–93 is tried by codex32 first but fails its checksum before falling through to `ValidMK`. No bip39/descriptor/codex32 string can match `ValidMD`/`ValidMK`.
- `validateMdmk` faithfully mirrors `validateDescriptor`; `mdmkFlow`'s ChoiceScreen→EngraveScreen loop matches `descriptorFlow`.

## Issues
### Critical: None.
### Important
**I1 — md1/mk1 verification is lowercase-only, diverging from the case-tolerant codex32 sibling (uppercase scans silently become "unknown format").** `codex32/mdmk.go:90-104`, surfacing at `gui/scan.go:70`. `verifyMDMK` rejects non-lowercase two ways: `gotHRP != hrp` (line 92, `hrp` is the lowercase literal), and `inputHRP("md")` sets engine `_case=lowerCase` while `inputData(data)` feeds original-cased data, so uppercase data hits `setCase`→`errInvalidCase`. By contrast `codex32.New` passes the original-cased HRP from `splitHRP(s)` into the engine, which case-normalizes via `setCase`/`unicode.ToLower` — that is why BIP-93 vectors like `"MS12NAMEA..."` parse. Net: an all-uppercase md1/mk1 payload (legal, QR-density-optimized) fails `codex32.New`, then both `ValidMD`/`ValidMK`, → `errScanUnknownFormat`. Fix: normalize case in `verifyMDMK` (compare `strings.ToLower(gotHRP)` and pass the original-cased HRP+data to the engine as codex32 does), OR document lowercase-only + add an uppercase negative test so the limitation is intentional.

### Minor
**M1 — `mdmkFlow` swallows the "no plate fits" error silently, unlike the descriptor flow it claims to mirror.** `gui/gui.go:1788-1792`. Descriptor path surfaces it via `DescriptorScreen.Confirm`→`showErr(NewErrorScreen(err))` (gui.go:2153-2156); `mdmkFlow` just returns. Comment overstates parity. Also `NewErrorScreen`'s `ErrTooLarge` body is hardcoded "The descriptor cannot fit any plate size." — wrong wording for md/mk.
**M2 — `TestMdmkOversizeRejected` does not exercise the path its comment claims.** `gui/mdmk_gui_test.go:23-29`. The 5000-char input fails at `qr.Encode` before the plate-fitting loop, so the `len(validEngravings)==0 → return lastErr` branch is untested.
**M3 — Minor test gaps.** No long-mk1 boundary positive; `TestMDMKLengthBracket` covers 94 + too-short but not reserved-95 nor 109 just-over-max; no explicit case-behavior assertion (ties to I1).

## Assessment
The cryptographic core is correct and well-guarded (packing, shift math, length gate, HRP gate, panic-safety, scanner ordering all hold under adversarial tracing). The one finding above nitpick is I1: md/mk verification silently accepts lowercase only while the adjacent codex32 path accepts both cases — an undocumented capability gap; uppercase QR vanishes into "unknown format" with no diagnostic. Fails closed → Important, not Critical.

## Verdict: NOT-GREEN (0 Critical / 1 Important)
- **I1 (Important):** lowercase-only divergence — fix (normalize case in `verifyMDMK`) or document+test. `codex32/mdmk.go:90,92`; observable at `gui/scan.go:70`.
- **M1/M2/M3 (Minor):** misleading comment; oversize test only hits QR path; missing boundary/case test cases.

---

## FOLD PLAN (added by main session)
| Finding | Sev | Disposition (pending user decision on I1 fix-vs-document) |
|---|---|---|
| I1 lowercase-only | **Important** | RECOMMENDED FIX: case-tolerance to match `codex32.New` — in `verifyMDMK`, `strings.EqualFold(gotHRP, hrp)` + `e.inputHRP(gotHRP)` (original case) so consistent-uppercase validates; add uppercase-positive + mixed-case-negative tests. (Alt: document lowercase-only + uppercase negative test.) Touches the live PR #35 branch → confirm with user. |
| M1 | Minor | Fix the misleading comment (mdmkFlow returns silently like backupSeedStringFlow, not descriptorFlow); optionally a md/mk-worded error screen. |
| M2 | Minor | Add a TEXT-only-too-large case to cover the no-plate-fits branch. |
| M3 | Minor | Add long-mk1 boundary positive + reserved-95/over-max negatives + a case-behavior assertion. |

Re-review (R1) to GREEN after the fold. Note: this modifies upstream PR #35 (push to `feat/engrave-mdmk`).
