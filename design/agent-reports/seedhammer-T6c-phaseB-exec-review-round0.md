# T6c Phase B (multisig picker) — whole-diff exec review (round 0) — VERBATIM agent report

**Agent:** `a1065a886b55263b4` (adversarial opus architect; re-ran the A3 byte-match through the PRODUCTION `assembleBuildPolicy` wrapper + self-slot/homogeneous-fp/no-mixed probes). **Branch:** `feat/t6c-picker`. **Base:** `f323dd2`. **Final commit:** `2abd8b3`. **Date:** 2026-06-20.
**Verdict:** GREEN (0C/0I). 2 non-blocking Minors. Mandatory post-implementation gate per CLAUDE.md phase (4). Cleared for merge.

---

# T6c Phase B (multisig picker) — whole-diff exec review (round 0)
**Reviewer:** opus architect (adversarial)  **Branch:** feat/t6c-picker  **Base:** f323dd2  **Verdict:** GREEN (0C/0I)

## A3 byte-match + homogeneous-fp re-run (MANDATE #1) — RAN
RE-RAN committed `TestAssembleBuildPolicy_T6bByteMatch` → PASS (stub 7b716421, self masterFP 73c5da0a). The committed test bypasses the `assembleBuildPolicy` WRAPPER for the two foreign slots (the fixture's foreign keys carry no base58 xpub, so it calls `md.EncodeMultisig` directly). Closed that gap with an INDEPENDENT probe (`TestReview_A3_WrapperByteMatch`): re-serialized the fixture's two foreign 65-byte keys to base58 xpubs, wrapped as gathered `mk.Card`s, derived self @1 from the abandon seed, drove the REAL committed `assembleBuildPolicy(N=3,K=2,wsh,OriginShared m/48'/0'/0'/2',SelfSlot=1,IncludeFp=false)`:
```
WRAPPER A3: 6 chunks byte-exact == fixture; stub=7b716421; self@1 fp=73c5da0a; all slots FpPresent=false
```
All 6 chunks byte-exact vs on-disk `t6b_multisig_full.md1.txt`, stub `7b716421`, via the production wrapper. Homogeneous fp + no-mixed-card (`TestReview_HomogeneousFp_NoMixedCard`, RAN): Include + a fp-less cosigner → `assembleBuildPolicy` ERRORS (`cosignerFromCard` rejects `Fingerprint==""`); Include+fp → ALL FpPresent=true; Omit+fp-bearing → ALL FpPresent=false (fp dropped). I-STUB end-to-end: review (step 6) shows the `stub` from `assembleBuildPolicy` (step 5); `deriveMultisigLeg` (step 9) recomputes `WalletPolicyIDStubChunks` from the SAME `assembledMd1` — no re-encode review→engrave. Shown stub == engraved stub. **PASS.**

## Self-slot @S placement (MANDATE #2) — RAN
`TestReview_SelfSlotPlacement` (RAN): self@0 vs self@2 →
```
self@0 stub=7c935ae6 ; self@2 stub=8a7e73c0 ; DISTINCT
```
Distinct WalletPolicyIds (order-sensitive, no silent normalization); self fp at @0 in self@0, at @2 in self@2, NOT at @0 in self@2. `assembleBuildPolicy` places self at `all[p.SelfSlot]`, fills OTHER slots in gather order (ascending, skipping SelfSlot) — NOT self-always-@0. **PASS.**

## Warning unskippable (MANDATE #3)
`multisigBuildExperimentalWarning` (`gui/multisig_build.go:147-167`) byte-faithful clone of `childSeedWarning`: returns `true` ONLY on `ConfirmYes`, `false` on `ConfirmNo`/loop-exit. Call site `:99-103` unconditionally AFTER the (stub,slots) review (`:94-97`), BEFORE the first engrave (`bundleEngrave` `:121`); abort = early `return`. Mode choice (`:106`) is AFTER the warning. Committed abort/confirm tests RAN green. Unskippable, gating.

## Front-door / Supply-path regression (MANDATE #4)
`git diff f323dd2..HEAD` = exactly 4 files; `gui/gui.go` diff EMPTY, `md/` diff EMPTY (no enum/guard/dispatch/title/plate/carousel edit; `md.EncodeMultisig` reused, not modified). T6b Supply body extracted BYTE-IDENTICALLY: `awk`-extracted old `engraveMultisigFlow` body vs new `supplyMultisigPolicyFlow` → 99 lines each, `diff` empty. Lockstep program tests + full T6b/multisig/sh-wpkh/T6a/T5/T4 set RAN green. No regression.

## Security + deviations + no-regression
- Typed-only seed (`seedEntryFlow`, no scan→derive); per-leg scrub via `defer` mnemonic-zero (`:65-69`) + entropy wipe in reused `deriveMultisigLeg`. ms1 never NFC: gather reuses unmodified `clsMs1Refuse`; `buildCosignerCards` refuses `cardMD1`/`cardMS1` (gathers KEYS only); `b.MS1` flows only into the steel-only `multisigEngraveCards`. Grep `multisig_build.go` `xprv|tprv|ECPrivKey|PrivKey|SerializeSecret` → only a code COMMENT; `deriveAccountXpub` serialize-before-zero neuter confirmed (`gui/derive.go:50-51`). Mainnet-only. `TestAssembleBuildPolicy_NoXprv` green. Cosigner mk1s PUBLIC.
- Deviation #1 (`buildCosignerCards` `return out, true`): CORRECT (sig `([]mk.Card, bool)`; failures `nil, false`).
- M-a: no test asserts a specific Include id; `_IncludeFpDiffers` only `!= 7b716421`; `TestBuildReviewLines` passes a hardcoded `ceadba4d` to a pure renderer (`639cabcf` gone from source — grep-confirmed). M-b: empty-gather test drives Done→error-stays→dismiss→Back→return (gather precedes seed). Defensive `assembleBuildPolicy` slot/count guard present; fuzz ~497k execs, 0 crashes.
- Passphrase: Build-path handling byte-identical to the T6b supply body; the SAME passphrase feeds `deriveAccountXpub` (self) and `deriveMultisigLeg` (leg) → consistent. Not a defect.
- `go build ./...` rc=0; `go vet ./gui/` rc=0; `go vet ./gui/...` only the pre-existing `gui/op/draw_test.go:176` note; `go test ./...` ALL ok; every commit (ddf56da→2abd8b3) builds. Probe removed; tree clean.

## Critical / Important
None / None.

## Minor (→ FOLLOWUPS; non-blocking)
- M-c (n-picker Back UX): Back-from-n abandons the Build flow; re-showing the template is optional polish. Accept as-is.
- The in-tree A3 test exercises `md.EncodeMultisig` directly (not the wrapper) for the foreign slots (fixture limitation — foreign keys lack base58 xpubs); gap closed by `TestAssembleBuildPolicy_Wrapper` (2-of-2) + the external probe. A future fixture with base58 foreign xpubs could let the headline byte-match drive the wrapper directly. Doc/test-polish.

## Verified-correct
A3 byte-exact via the PRODUCTION wrapper (6 chunks == fixture, stub 7b716421) — RAN; self-slot @S order-sensitive + distinct ids — RAN; homogeneous fp + Include-requires-fp guard (no mixed card) + Omit-drops-fp — RAN; I-STUB shown==engraved (no re-encode); locked defaults (3 templates wsh-first, n∈2..5, k∈1..n, OriginShared m/48'/0'/0'/2'); unskippable gating EXPERIMENTAL warning; security spine (typed-only, per-leg scrub, ms1 NFC-refused, no xprv, neuter serialize-before-zero, mainnet-only); zero-lockstep front-door (gui.go/md diff empty); Supply body byte-identical (99-line diff empty); deviation #1 correct; fuzz 0 crashes; every commit builds; full `./...` green.

## Bottom line
**GREEN (0C/0I).** The implementation faithfully realizes the GREEN plan with no implementation-introduced defects. I independently RE-RAN the A3 byte-match through the COMMITTED `assembleBuildPolicy` wrapper (not the in-tree bypass) → 6 chunks byte-exact + stub 7b716421; self-slot @S placement order-sensitive with distinct ids and self at the chosen slot; the homogeneous-fp rule provably cannot mint a mixed-presence card; the EXPERIMENTAL warning is the only, unskippable route to engrave; the shown stub equals the engraved policy's stub; no secret reaches NFC and no xprv is serialized; the Supply path and all lockstep sites are byte-unchanged; build/vet/test/fuzz clean across every commit. **T6c Phase B is cleared for merge.**
