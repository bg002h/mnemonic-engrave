# T6c Phase A plan — R0 review (round 0) — VERBATIM agent report

**Agent:** `aa745bed84a2170e7` (adversarial opus architect; inlined the verbatim assembler in a throwaway worktree + RAN the byte-exact reproduction / stub-parity / round-trip / refuse / fuzz / golden-provenance probes). **Fork HEAD:** `65a1018`. **Rust:** `@c85cd49`. **Plan commit:** `becb2cc`. **Date:** 2026-06-20.
**Verdict:** NOT GREEN (0C / 2I / 2M). Both Importants are plan test-code/sequencing defects (not design flaws); load-bearing correctness fully proven. Persisted before folding per the R0 gate discipline.

---

# T6c Phase A plan — R0 review (round 0)
**Reviewer:** opus architect (adversarial)  **Fork HEAD:** 65a1018  **Rust:** @c85cd49  **Plan commit:** becb2cc  **Verdict:** NOT GREEN (0C / 2I / 2M)

## T6b-fixture reproduction probe (MANDATE #1) — RAN
Inlined the plan's Task-1 types + Task-2 assembler **verbatim** into an isolated throwaway worktree at `65a1018`, fed the three exact 65-byte payloads (@0/@1/@2 order, k=2, shared origin `m/48'/0'/0'/2'`, `MultisigWsh`, fp-absent) through the shipped `split`:
```
OK: all 6 chunks byte-exact
WalletPolicyId(out) = 7b716421db8b9f462967d04e0f8a3fd5
stub(d)             = 7b716421
DECODED fixture: N=3 Root=3(Wsh) Policy=2(SortedMulti) K=2 InnerWsh=false
  @0 fpPresent=false xpub=10111213...40af23 origin=m/48h/0h/0h/2h
  @1 fpPresent=false xpub=bba0c7ca...dc9f29 origin=m/48h/0h/0h/2h   ← abandon-seed cc bba0c7ca…7e2d5763 ‖ pk 021a3bf5…dc9f29
  @2 fpPresent=false xpub=10111213...709ee5 origin=m/48h/0h/0h/2h
```
**All 6 chunks byte-exact; WalletPolicyId = 7b716421…; stub = 7b716421; inputs fully recoverable.** MANDATE #1 PASSES — the Task-3 golden strategy's central anchor holds. Three shapes round-trip with correct InnerWsh discriminant (wsh=false / sh_wsh=true / sh=false); fuzz 1.36M execs/15s, no panics/crashers; mixed/partial-fp column round-trips.

## Ambiguity adjudication
**(a) stub-handle parity — RESOLVED, no divergence.** `WalletPolicyIDStub(d)` == `WalletPolicyIDStubChunks(out)` identical across T6b (`7b716421`), fp-present (`fc852e05`), divergent-origin (`1a52b34e`), `sh` (`6da5f0ab`); full `WalletPolicyIdChunks(out)` == `WalletPolicyId(Reassemble(out))` in every case. A4 sound; they are provably equal (no need to pin one authoritative).
**(b) sh-shape golden provenance — RESOLVED, sound + non-circular.** On Rust `c85cd49` the CLI does NOT reject a no-origin `sh(sortedmulti)` at *encode* (emits keyless template `0x0ca1d`); the constraint is at **round-trip** — Go `Reassemble` of a no-origin `sh(sortedmulti)` fails `md: missing explicit origin`, while `wsh`/`sh(wsh)` round-trip fine. Since `EncodeMultisig` always supplies an explicit shared origin, generating the `sh`-shape goldens in Go anchored to the Rust `sh_wsh_multi.bytes.hex` template + assembler round-trip is correct. Rust CLI gen with the VF9 depth-4 xpub: `wsh`=0x36d1b, `sh(wsh)`=0x58624, `sh`(with origin)=0x90289; depth-3 rejected (`md-cli/src/parse/keys.rs:67-77`).

## Critical
None. Wire/identity core, ordering contract, byte-exact reproduction, refuse-paths, golden provenance all source-grounded + confirmed by running code.

## Important
- **I1 — Task 3a A1 test is wrong and will FAIL as written (implementer-blocking + misdirecting).** The plan's `mkTree` hard-codes `indices: []uint8{0, 1, 2}` for BOTH legs (plan line 537), but loads each vector's own `n` via `loadDescriptor`. `wsh_sortedmulti` is **n=3** → passes; **`sh_wsh_multi` is n=2** (`md/testdata/vectors/sh_wsh_multi.descriptor.json` "n":2, indices [0,1]) → the n=3 tree references @2, which `validatePlaceholderUsage` rejects: `encodePayload: md: placeholder index out of range`. The plan's Step 2 says "**Expected: PASS immediately** … If it FAILS … STOP and re-verify VF2" — doubly harmful (hard FAIL + wrong suspicion; VF2/VF10 are fine). **Fix:** parameterize the index list per vector (`[0,1,2]` for `wsh_sortedmulti`, `[0,1]` for `sh_wsh_multi`); with that both byte-match exactly (`2082001821c22180` / `2042001830860850`) — verified. Update the Step-2 "Expected" prose.
- **I2 — Task 1's test file will not compile at Step 4 (unused imports break the per-task commit).** Task 1 Step 1 declares the full import block (`encoding/hex`, `errors`, `os`, `path/filepath`, `strings`, `testing`, `seedhammer.com/codex32`, plan lines 116-125), but the only Task-1 body (`TestEncodeMultisigRequestPlumbing`) uses **only `testing`** + the new types → `"encoding/hex" imported and not used` hard error. Task 1 Step 4 ("Expected: PASS") instead fails to compile; the Task-1 commit is non-building in isolation, contrary to per-task TDD/commit discipline. (Final all-tasks file uses every import — final state fine; intermediate-checkpoint break.) **Fix:** import only `testing` in the Task-1 step; add `hex`/`codex32` in Task 2, `os`/`filepath`/`strings`/`json` in Task 3, `errors` in Task 5 (accrete) — or note imports accrete and Step-4 PASS holds once the using-code lands.

## Minor
- **M1 — VF6 line drift.** `PathComponent` is at `md/encode_singlesig.go:20` (struct keyword), not `:18` (doc-comment). Reused correctly; cosmetic.
- **M2 — golden-circularity framing.** A2 full-policy chunk-string goldens ARE the assembler freezing its own output (self-referential alone). The non-circular anchors are real + sufficient — A1 template-parity vs Rust `.bytes.hex` (byte-exact after the I1 fix), A3 T6b byte-exact (independently-vendored card, commit `e1c4240`, documented source xpubs `gui/multisig_testhelpers_test.go:14-19`), depth-4 Rust CLI gen. Recommend the plan state A2's role is drift-guard (frozen-output regression), while A1+A3 carry byte-correctness.

## Verified-correct
- **MANDATE #1:** byte-exact T6b reproduction by running the verbatim assembler — 6/6 chunks, WalletPolicyId `7b716421…`, inputs recoverable.
- **MANDATE #2:** `multiSigTree` builds the three wrappers ⊃ `sortedmulti{k,[0..n-1]}`; N `idxPub` TLVs + per-cosigner `idxFP` gated on `FpPresent` (fp-absent T6b byte-matches; mixed-fp round-trips); shared/divergent origin via explicit `OriginMode`; order-preserving (no sort). Both adopted recommendations present: struct constructor `EncodeMultisigRequest{…OriginMode…}` + `(out, stub, slots, err)` handle. Internal-type literals match shipped structs verbatim. NO exported `Md1EncodingId` (absent confirmed); handles: `WalletPolicyIDStub(*descriptor)([4]byte,error)` `walletpolicyid.go:106`, `WalletPolicyIDStubChunks([]string)([4]byte,error)` `:129`, `WalletPolicyIdChunks([]string)([16]byte,error)` `:119`, `WalletPolicyId(*descriptor)([16]byte,error)` `:30`. `PathComponent` reused; `MultisigScript`/`MultisigCosigner`/`SlotInfo`/`OriginMode` new, no collision. A6 `errors.Is` verified for all six guards (`errKGreaterThanN`/`errThresholdRange`/`errKeyCountRange`/`errMultisigEmptySharedOrigin`/`errMultisigEmptyDivergent`/`errMultisigBadScript`).
- **MANDATE #3:** depth-4 xpub documented+verified; A1 sound after I1; provenance non-circular; both ambiguities resolved.
- **MANDATE #4:** A4 round-trip/identity, A5 fp+divergent, A6 refuse, A7 fuzz map to tasks with FAIL→PASS; spec I1–I8 mapped; shared helpers present (reusable); new helpers no collision; headless-only; baseline `go test ./md/...` green at `65a1018`.

## Bottom line
**NOT GREEN — 0 Critical, 2 Important.** Load-bearing correctness fully proven (verbatim assembler reproduces T6b byte-for-byte with `WalletPolicyId 7b716421…`; stub handles provably equal; all three shapes round-trip with the right InnerWsh discriminant; every A6 error matches via `errors.Is`; 1.36M-exec fuzz clean; golden provenance non-circular). Both blockers are plan test-code/sequencing defects: **(I1)** Task-3a A1 hard-codes n=3 indices for the n=2 `sh_wsh_multi` vector → `placeholder index out of range` + misdirecting prose (parameterize per vector → both byte-match, verified); **(I2)** Task-1 full import block + `testing`-only body → `imported and not used` (accrete imports per task). Fold both + the two Minors, re-persist, re-dispatch for a GREEN round before any code.
