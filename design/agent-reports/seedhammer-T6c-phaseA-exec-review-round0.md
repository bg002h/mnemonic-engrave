# T6c Phase A (md.EncodeMultisig) — whole-diff exec review (round 0) — VERBATIM agent report

**Agent:** `a881d20416bc6fd78` (adversarial opus architect; re-derived the byte-exact T6b reproduction FROM the decoded fixture, proved no key-sort across 3 orders, hand-decoded the bitstream, 1.46M-exec fuzz). **Branch:** `feat/t6c-encode-multisig`. **Base:** `4bcef4b`. **Final commit:** `011b0db`. **Date:** 2026-06-20.
**Verdict:** GREEN (0C/0I). 1 cosmetic Minor → FOLLOWUPS. Mandatory post-implementation gate per CLAUDE.md phase (4). Cleared for merge.

---

# T6c Phase A (md.EncodeMultisig) — whole-diff exec review (round 0)
**Reviewer:** opus architect (adversarial)  **Branch:** feat/t6c-encode-multisig  **Base:** 4bcef4b  **Verdict:** GREEN (0C/0I)

Diff = 14 files, all additive: `md/encode_multisig.go` (201 LOC), `md/encode_multisig_test.go` (524), `md/encode_multisig_fuzz_test.go` (69), 5 multisig golden sets + README. `git diff 4bcef4b..HEAD -- md/md.go md/encode.go md/walletpolicyid.go md/canonicalize.go md/identity.go md/encode_singlesig.go md/decode.go` = **empty** (no shipped file touched). HEAD `011b0db`, 9 commits.

## T6b byte-exact + identity re-run (MANDATE #1) — RAN
Did NOT trust the inlined `t6bChunks` constant. Probe `TestReviewT6bDecodeThenReEncode` read the **actual** `gui/testdata/t6b_multisig_full.md1.txt` (md5 `ed04f8e0…`), decoded via `ExpandWalletPolicyChunks` to extract cosigner xpubs/order/origin/fp-presence FROM THE DECODE, fed them back into `EncodeMultisig`:
```
ALL 6 chunks byte-exact vs fixture file
WalletPolicyId = 7b716421db8b9f462967d04e0f8a3fd5  stub = 7b716421
```
Committed `TestEncodeMultisigT6bByteExact` also passes (no cache). **The device-authored card binds its mk1 stub to `7b716421…`.** ✓

## Order-sensitivity / no-key-sort (MANDATE #2) — RAN
`TestReviewOrderSensitivity` fed the same 3 keys in 3 orders:
```
[0,1,2] WPID=7b716421db8b9f462967d04e0f8a3fd5
[2,1,0] WPID=7c5f2e3d3e8255650526ba4385c18421
[1,0,2] WPID=ff3dbdba7e342f9575c2ede682e44730
```
Three DISTINCT valid ids → **no silent key sort**. Decode confirms key0 lands @0 in `[0,1,2]` and @2 in `[2,1,0]` — order is the caller's contract, honored exactly. ✓

## FpPresent gating (MANDATE #3) — RAN
`TestReviewFpPresentGating`: fp-absent encode = T6b id `7b716421…`; fp-present encode of the same keys/origin = `639cabcf…` (different, correct card); fps recovered on decode. Per-cosigner `FpPresent` gating works — an always-fp or never-fp encoder would have collided. ✓

## Wire/golden + refuse + round-trip
- **A1 anchors genuinely external & byte-correct.** `wsh_sortedmulti.bytes.hex=2082001821c22180` and `sh_wsh_multi.bytes.hex=2042001830860850` were vendored in PRIOR commit `ac00093`, NOT this diff. Hand-decoded `2082001821c22180`: the 22-bit sortedmulti body (`tag=000111`, `k-1=00001`, `n-1=00010`, indices `00 01 10`) at bit offset 36 — matching V1's 22-bit cost, k=2/n=3/[0,1,2], no off-by-one. ✓
- **A2 is correctly a drift-guard, not the anchor.** The 5 full-policy `multisig_*` goldens were added by this diff = frozen assembler output. Correctness rests on A1 (Rust bit-layout) + A3 (fixture-derived). ✓
- **All wrappers byte-correct.** `TestEncodeMultisigFullPolicyParity` PASS for wsh (InnerWsh=false), sh(wsh) (Root=Sh, InnerWsh=true), sh (Root=Sh, InnerWsh=false), fp-present, divergent. Divergent payload starts `a0…` (divergent path_decl) vs shared `20…`. `TestReviewDivergentOriginIndexOrder`: @0→2', @1→3', @2→4' recovered in correct index order. ✓
- **Refuse-unsupported.** `TestEncodeMultisigRefuse` PASS: k>n→`errKGreaterThanN`, k=0→`errThresholdRange`, empty-shared-origin→`errMultisigEmptySharedOrigin`, divergent-empty→`errMultisigEmptyDivergent`, zero-cosigners→`errKeyCountRange`, bad-script→`errMultisigBadScript`, all via `errors.Is`. Boundary probe: n=33→`errKeyCountRange`, k=33→`errThresholdRange`, n=32/k=32→succeeds (kiw/n lockstep, I7). The 3-value enum + `default:` make tagMulti/taproot/miniscript structurally unemittable. ✓
- **Round-trip + identity zero-change (I6).** `TestEncodeMultisigRoundTrip` PASS: `WalletPolicyIdChunks(out)==WalletPolicyId(Reassemble(out))`, stub==id prefix, no multisig-specific identity code added. ✓
- **Fuzz.** 1.46M execs / 15s, 0 failures, no crashers. ✓
- **No regression.** `go test ./...` = ALL packages ok (gui 14.5s); `go vet ./md/...` rc=0; `go build ./...` clean. Shipped vectors untouched. ✓

## Critical
None.
## Important
None.
## Minor
- **M1 (→FOLLOWUPS).** `md/encode_multisig.go:110` — the `OriginMode` switch `default:` returns `errMultisigBadScript` ("unknown script kind"), the wrong message for a bad *origin mode*. Probed `OriginMode(99)`: safely errors (no card emitted — NOT a refuse-path hole) but mislabels. Unreachable from the public closed 2-value enum without a deliberate cast. Cosmetic; suggest a dedicated `errMultisigBadOriginMode`.

## Verified-correct
- `EncodeMultisig` is a faithful n>1 generalization of shipped `EncodeSingleSig` (identical descriptor literal: useSite `<0;1>/*`, TLV `{pubPresent, pubkeys, fpPresent, fingerprints}`). Internal type field names (`idxPub{idx,xpub}`, `idxFP{idx,fp}`, `tlvSection`) match `md/md.go`.
- `multiSigTree` emits `Cosigners[i]→@i` (indices `[0..n-1]` in input order); canonicalize is identity for this AST. tags `tagWsh=0x02`/`tagSh=0x03`/`tagSortedMulti=0x07` correct.
- The inlined `t6bChunks` constant == the on-disk fixture byte-for-byte; gui-side `TestSuppliedMultisigFixtureIsFullPolicy` independently guards the fixture as 2-of-3 wsh(sortedmulti).
- `descriptor.n==pathDecl.n==len(Cosigners)==n` in lockstep; k/n bounds surface from shipped `split` guards.

## Bottom line
Byte-exact T6b reproduction (re-derived independently from the fixture, not the constant) with a binding `7b716421…` WalletPolicyId; provably no key-sort (3 orders → 3 ids); correct FpPresent gating; all three wrappers + shared/divergent origins byte-correct against external Rust anchors; identity zero-change; clean refuse paths; 1.46M-exec fuzz clean; zero regression across the whole repo. One cosmetic error-message mislabel on an unreachable path. **GREEN — T6c Phase A is cleared for merge.**
