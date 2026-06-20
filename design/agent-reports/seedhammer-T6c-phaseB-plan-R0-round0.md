# T6c Phase B plan — R0 review (round 0) — VERBATIM agent report

**Agent:** `acbd428c53f73c62a` (adversarial opus architect; RAN the A3 byte-match probe in an isolated worktree). **Fork HEAD:** `f323dd2`. **Plan commit:** `a680dd6`. **Date:** 2026-06-20.
**Verdict:** GREEN (0C/0I) — 3 non-blocking Minor; reviewer states "no fold required before implementation." Persisted per the gate discipline; M-a/M-b passed to the implementer as guidance.

---

# T6c Phase B plan — R0 review (round 0)
**Reviewer:** opus architect (adversarial)  **Fork HEAD:** f323dd2  **Plan commit:** a680dd6  **Verdict:** GREEN (0C/0I) — 3 Minor

## A3 byte-match re-verification (MANDATE #1) — RAN
Detached throwaway worktree at `f323dd2` (removed). Probe decoded `gui/testdata/t6b_multisig_full.md1.txt`, reconstructed the request EXACTLY as `assembleBuildPolicy` would, called `md.EncodeMultisig`:
```
fixture has 6 chunks
decoded slot 1: fpPresent=false cc=bba0c7ca…7e2d5763 pk=021a3bf5…dc9f29
self masterFP = 73c5da0a ; self cc=bba0c7ca… ; self pk=021a3bf5…
CONFIRMED: self @1 matches fixture keys[1]
OMIT: 6 chunks BYTE-EXACT == fixture; stub = 7b716421
I-STUB confirmed: returned stub == WalletPolicyIDStubChunks(out) == 7b716421
INCLUDE (foreign fp {1,2,3,4}, self 73c5da0a): stub = ceadba4d  (!= 7b716421, good)
```
(a) Omit → 6 chunks byte-exact + stub `7b716421` — CONFIRMED. (b) Include → distinct valid id — CONFIRMED. Self @1 from abandon seed == fixture keys[1] — CONFIRMED. I-STUB — CONFIRMED. Matches the in-tree `TestEncodeMultisigT6bByteExact` (`md/encode_multisig_test.go:370`). The headline A3 gate is correctly specified; the `assembleBuildPolicy` request shape is byte-faithful.
**M-a finding:** `639cabcf` is NOT reproducible from the fixture keys (probed 4 fp configs → `c15be9db`/`9842cf3e`/`ceadba4d`/`e5669291`); it's a stale literal from the Phase-A exec report, NOWHERE in current source. NOT load-bearing — Task 3 `_IncludeFpDiffers` only asserts `!= 7b716421` (passes); Task 4 `TestBuildReviewLines` passes a HARDCODED `[4]byte{0x63,0x9c,0xab,0xcf}` to a pure renderer (always passes). No plan test asserts the encoder reproduces `639cabcf`.

## User-decisions encoding (MANDATE #2)
- **Self-slot @S:** CORRECT. `assembleBuildPolicy` sets `all[p.SelfSlot]=self` then fills OTHER slots in gather order (`for slot…{ if slot==SelfSlot {continue}; all[slot]=cosignerFromCard(cosigners[gi]…); gi++ }`). Bounded `multisigSelfSlotChoices(n)=["@0".."@{n-1}"]`.
- **fp-presence HOMOGENEOUS:** CORRECT. `multisigIncludeFpFor(0)==false/(1)==true`. Omit → ALL `FpPresent=false`; Include → self fp from `fpBytes(masterFP)`, each cosigner from its card's 8-hex `Fingerprint`. NO MIXED path: under Include, `cosignerFromCard` ERRORS if a card has `Fingerprint==""` (R0-M1 satisfied). Review shows stub + per-slot fp + the "fingerprint choice changes the id — match your coordinator" note. fp byte-order verified consistent (`bip32.Fingerprint` BigEndian ↔ `fpBytes` ↔ `mk.Card.Fingerprint` `%08x`/`hex.DecodeString`).
- **Locked defaults:** CORRECT. 3 templates (wsh default), n∈2..5/k∈1..n (k>n unreachable), `OriginShared` hardcoded, self at `multisigSharedOrigin()=m/48'/0'/0'/2'` (self-origin==policy-origin by construction).

## Pure/orchestrator split + reuse + front-door (MANDATE #3)
- **Split justified:** `testPlatform.NFCReader()` RETURNS NIL (verified `gui/gui_test.go:408-410`) → cosigner cards can't be injected via `bundleGatherFlow` in a flow test → the pure `assembleBuildPolicy` is the only way to drive A3. Mirrors shipped `findUserSlot`/`extractSuppliedMd1`.
- **Reuse verbatim:** `deriveMultisigLeg(...,assembledMd1,full)` binds `Stubs=[WalletPolicyIDStubChunks(md1)]` (`gui/multisig_derive.go:32,42,50,60`); `multisigEngraveCards`/`bundleEngrave`/`multisigVerifyFlow`/`multisigRestoreDocFlow`/`md.ExpandWalletPolicyChunks` all consume the assembled md1 like a supplied one; `md.EncodeMultisig` sole producer.
- **Front-door zero-lockstep:** `engraveMultisig` enum idx 4 (`gui/gui.go:152`); t5-M1 guard keys off `bip85Derive` (`:164`); wrapping `engraveMultisigFlow`'s body behind a `ChoiceScreen` adds no program. Supply path = verbatim T6b body.

## Warning + security + harness + coverage (MANDATE #4)
- **Unskippable warning:** `multisigBuildExperimentalWarning` faithful clone of `childSeedWarning` (`gui/bip85.go:145`); returns `true` ONLY on `ConfirmYes`, `false` on `ConfirmNo`/Back; placed after the (stub,slots) review, before engrave; flow ABORTS on `false`. Abort test A6 + confirm test. Non-bypassable.
- **Security spine:** typed-only seed; per-leg scrub; ms1 NFC-refused (reused `clsMs1Refuse`); no xprv (`deriveAccountXpub` neuter serialize-before-zero + `TestAssembleBuildPolicy_NoXprv` grep); mainnet-only. All verified.
- **UI-harness:** `runUI`/`pumpUntil`/`click`/`press` under `testing/synctest`, mirroring shipped flow tests; `confirmReviewScreen` is a near-verbatim parameterized clone of shipped `bundleReviewFlow` (`gui/bundle_flow.go:227`) — primitives all exist. Picker/warning tests drive non-vacuously. Import-accretion handled per-task (Task 3 hedges `chaincfg`-until-Task-5; `encoding/hex` used by `cosignerFromCard` in Task 3).
- **Coverage + Minors:** A1–A9 + user decisions each map to failing-test-first tasks; the 4 spec Minors folded (M4 Omit-drive ✓, M1 homogeneous ✓, M2 line-numbers ✓, M3 origin-mode out of scope ✓).
- **n-picker Back:** Back-from-n → abandon; Back-from-k/@S/fp → re-pick n. ACCEPT as-is (internally consistent; re-picking n re-derives k's range). Minor UX.

## Critical / Important
None / None.

## Minor
- **M-a (`639cabcf` stale literal — comment/doc only).** Not reproducible (actual Include id e.g. `ceadba4d`). Only a hardcoded literal in a pure render-test (always passes) + a misleading comment (Task 3 ~:627/:694). Fix the comment to "Include yields a DIFFERENT id (e.g. `ceadba4d`)"; drop/illustrative-note the `639cabcf` literal. No code/behavior impact.
- **M-b (Task 6 empty-Done test-tuning).** `bundleGatherFlow` with `NFCReader()==nil` yields zero cards; Done shows an in-gather error and STAYS; only Back returns `(nil,false)`. The empty-gather test should drive Back, not Done. Test-tuning only.
- **M-c (n-picker Back UX).** Accept as-is; re-showing template on Back-from-n is optional polish.

## Verified-correct
A3 Omit byte-exact + stub `7b716421` + self@1==fixture + I-STUB + Include-differs (RAN); self-slot @S + cosigner-fill-in-gather-order; homogeneous fp + Include-requires-fp guard + big-endian fp consistency; locked defaults; pure/orchestrator split (`NFCReader()==nil` verified); reuse seams verbatim; `md.EncodeMultisig` sole producer; front-door zero-lockstep; unskippable warning; security spine; `confirmReviewScreen` faithful clone; baseline clean (`go build`/`go test ./gui/...`/`go vet ./gui/...` = only the pre-existing `draw_test.go:176` note).

## Bottom line
**GREEN (0C/0I).** A3 fp-absent byte-match independently RE-VERIFIED by running code (Omit → 6 chunks byte-exact + stub `7b716421`; self@1 matches; I-STUB; Include distinct). User decisions correctly encoded (self-slot @S; homogeneous fp, no mixed path); pure/orchestrator split justified by the verified `NFCReader()==nil`; reuse + front-door byte-faithful and lockstep-free; EXPERIMENTAL warning non-bypassable. The 3 Minors (stale `639cabcf` comment/literal, empty-Done test-tuning, n-picker Back UX) are non-blocking and need no fold before implementation. Cleared for single-subagent TDD in `feat/t6c-picker`, then the mandatory adversarial exec-review.
