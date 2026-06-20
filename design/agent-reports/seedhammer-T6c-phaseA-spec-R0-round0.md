# T6c Phase A SPEC — R0 review (round 0) — VERBATIM agent report

**Agent:** `acd3c334549a00046` (adversarial opus architect; RAN ordering/fp/re-encode probes on both Rust md-codec `@c85cd49` and the Go port). **Fork HEAD:** `8eb51d7`. **Spec commit:** `7891eec`. **Date:** 2026-06-19.
**Verdict:** GREEN (0C/0I). 2 plan-stage Minors + 2 adopted ambiguity recommendations. Persisted per the R0 gate discipline; cleared for the implementation plan (no re-dispatch needed — folds are plan-stage, don't change load-bearing claims).

---

# T6c Phase A SPEC — R0 review (round 0)
**Reviewer:** opus architect (adversarial)  **Fork HEAD:** 8eb51d7  **Rust:** descriptor-mnemonic@c85cd49  **Spec commit:** 7891eec  **Verdict:** GREEN (0C / 0I)

## Ordering-contract + fp-presence probe (MANDATE #1) — RAN

**(a) canonicalize = first-occurrence, NO key sort — CONFIRMED on both Rust and Go.**
- Rust: the *only* active encode-path sort is `canonicalize.rs:148` `entries.sort_by_key(|(idx,_)| *idx)` — sorts a `(u8, T)` TLV vector by the `u8` **placeholder index**, payload never inspected. `walk_collect_first` MultiKeys arm (`canonicalize.rs:85-94`) iterates `indices` in document order; `remap_indices` MultiKeys arm (`:131-136`) remaps through `perm[]` with no sort; `tree.rs:115-139` writes indices in stored order; `to_miniscript.rs:487-499 build_multi_threshold` iterates in order. `grep` for `Ord/PartialOrd derive/impl` over `crates/md-codec/src/` = empty. NO lexicographic key/pubkey/xpub byte sort anywhere in the encode path.
- Go mirror identical: `canonicalize.go` `remapPubVec/remapFPVec/remapOriginVec/remapUseSiteVec` (`:183-209`) all `sort.SliceStable(... v[i].idx < v[j].idx)` — by index only; `remapIndices case multiKeysBody` (`:167-172`) remaps with no sort. The spec's central claim (V3, I1, I4) is TRUE.

**(b) T6b fixture is fp-ABSENT + WalletPolicyId — CONFIRMED by live decode.** Probe (`ExpandWalletPolicyChunks` on `gui/testdata/t6b_multisig_full.md1.txt`):
```
chunk count = 6 ; Template: N=3 Root=3(ScriptWsh) Policy=2(PolicySortedMulti) K=2 M=3 InnerWsh=false Renderable=true
@0 fpPresent=false fp=00000000 xpubPresent=true origin=m/48h/0h/0h/2h useSite=<0;1>/*
@1 fpPresent=false ... (abandon-seed key bba0c7ca…)
@2 fpPresent=false ...
WalletPolicyId = 7b716421db8b9f462967d04e0f8a3fd5  stub = 7b716421
```
All 3 slots `FingerprintPresent=false`, WalletPolicyId == `7b716421…` (V8). An always-fp encoder would NOT byte-match → the per-cosigner `FpPresent` flag (spec §IN.3) is correctly motivated.

**(c) round-trip-reproduces-fixture — CONFIRMED by live re-encode.** Probe `TestR0_ReassembleReEncode`: `Reassemble(fixture)` → re-run `split(d)` → **byte-for-byte reproduces all 6 chunk strings**. Decoded descriptor: `n=3, pathDecl{n=3, shared=m/48'/0'/0'/2'}, tree tag=Wsh⊃SortedMulti, tlv{pubPresent, fpPresent=false, 3 pubkeys, 0 fp, 0 origin-overrides}, useSite=<0;1>/*`. Every input the proposed `EncodeMultisig` needs (3×65-byte payloads in @0/@1/@2 order, k=2, shared origin, fp-absent) is recoverable from a decode; feeding them through the shipped pipeline reproduces the card → A3 acceptance test is achievable. Ordering contract sound: caller order fixes @N, canonicalize is the identity permutation for this AST.

## Critical
None.
## Important
None.
## Minor
- **m1 — V9's full-policy generator claim under-verified by the recon; now closed.** The recon/spec V9 ran only the **template-only** `md encode` form. The load-bearing **`--key`/`--fingerprint` full-policy** form has a real constraint the spec does not document: `md-cli/src/parse/keys.rs:67-77` enforces xpub **depth == 4** for `ScriptCtx::MultiSig` (vs 3 for single-sig). A depth-3 xpub is rejected. Generating A2/A3/A5 full-policy goldens requires a **depth-4** xpub (e.g. abandon-seed `xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf` at `m/48'/0'/0'/2'`). Confirmed both fp-present (`chunk_set_id 0x323a7`) and fp-absent (`0x97134`) full-policy `sortedmulti` cards generate cleanly with it. CLI-only concern — `EncodeMultisig` takes raw 65-byte payloads (no depth field, proven by re-encode probe). The plan should record the depth-4 requirement + working xpub.
- **m2 — A1 vendored-golden coverage nuance.** Of the vendored multi vectors, **only `wsh_sortedmulti.bytes.hex` carries `tagSortedMulti`**; `wsh_multi_2of3/2of2`, `wsh_divergent_paths`, `wsh_with_fingerprints`, `sh_wsh_multi` carry `tagMulti` (0x06). `wsh_sortedmulti=2082001821c22180` vs `wsh_multi_2of3=2082001821822180` differ in exactly one bit (the tag low bit) → bit layout identical, A1 approach valid. NO vendored `sh(sortedmulti)`/`sh(wsh(sortedmulti))` golden — the InnerWsh-discriminant shapes (A2) must come from fresh `md encode` generation (proven to work). Ensure the plan vendors fresh sortedmulti goldens for the two `sh` shapes, not just `wsh`.

## Ambiguity adjudication (constructor shape; return-ordering-map?)
**(1) Constructor shape — recommend a struct param** over positional `EncodeMultisig(cosigners, k, script, sharedOrigin)`: the positional form overloads `sharedOrigin==nil` as the shared/divergent discriminant (implicit, easy-to-misuse on a fund-backup path). A `EncodeMultisigRequest{Cosigners, K, Script, OriginMode (enum Shared|Divergent), SharedOrigin}` makes the mode explicit + lets A6 validate against named fields. Recommendation, not a gate item (either form passes).
**(2) Return the @N→fp / `WalletPolicyIDStub` map in Phase A? — YES.** Cheap safety win, weighted strongly since ordering is the #1 risk. (i) The handle is **already obtainable with zero new identity code** — a Phase-A `gui` caller can call `md.WalletPolicyIdChunks(out)` on the returned `[]string` (exported, takes `[]string`; the `*descriptor`-arg `WalletPolicyIDStub` is unexported-arg, so the chunks-form is the right handle). (ii) Returning the assigned `@N→Fingerprint` slice + `WalletPolicyIDStub` (4 bytes) lets the caller verify cosigner ORDERING against a coordinator's expected stub BEFORE steel — the only defense against valid-but-wrong-binding, ~5 LOC. Defer the *UI* to Phase B; expose the *data* in Phase A. Recommend `(out []string, stub [4]byte, slots []SlotInfo{Index, Fingerprint, FpPresent}, err)` or equivalent. Not a gate blocker.

## Verified-correct
- **MANDATE #1 wire emitter (V1):** Go `encode.go:188-203` writes `(k-1)@5b, (len-1)@5b, N×idx@kiw`, guards `errThresholdRange/errChildCount/errKGreaterThanN` — byte-identical to Rust `tree.rs:115-139`. Cost test `encode_test.go:137-148` asserts 2-of-3@n=3,kiw=2 = 22 bits. Tags `tagWsh=0x02 tagSh=0x03 tagMulti=0x06 tagSortedMulti=0x07` confirmed both sides (`md.go:40-47`, `tag.rs:99-107`). Refuse surface (MANDATE #3): a pure assembler from a 3-value `MultisigScript` enum structurally cannot emit `tagMulti`/taproot/miniscript — refusal reduces to input-validation (k/n bounds, divergent-count, empty origin), backed by shipped guards A6 reuses.
- **MANDATE #2 identity binding (V4, I6):** `WalletPolicyId` (`walletpolicyid.go:30-102`) canonicalizes a clone, leads with `writeNode(tree)`, loops `idx 0..n-1` (`:48`) hashing per-@N `{presence, record(origin+usesite RAW), fp?, xpub?}` — shape-agnostic over the multi-key descriptor; `computeEncodingID=SHA-256(encodePayload(d))[0:16]`, `deriveChunkSetID`=top-20-bit. Zero multisig-specific identity code. Live probe: T6b card → `WalletPolicyId 7b716421…`, re-encodes identically → device-authored card's mk1 stub will match.
- **kiw/n lockstep (I7):** `kiw(dc.pathDecl.n)` `encode.go:416`, guard `errPathDeclNMismatch` `:401-403`. TLV idx strict-ascending (I8) via `errOverrideOrder` across all four sparse writers (`:259/280/301/322`). Assembler emitting @0..@{n-1} in order satisfies both.
- **Shared/divergent origin (V5, IN.4):** `writePathDecl` (`encode.go:103-125`) handles both (`pathDecl.shared *originPath`/`pathDecl.divergent []originPath`, mutually exclusive, `errDivergentCount`); golden-covered by `wsh_divergent_paths`.
- **Phase-B deferral:** correctly defers GUI picker/cosigner-gather/user-slot-derive to Phase B, names the **unskippable EXPERIMENTAL warning** as a Phase-B hard non-bypassable requirement (§OUT). TDD gate (A1–A7) + invariants (I1–I8) cover the risks.

## Bottom line
**GREEN — 0 Critical / 0 Important.** Every load-bearing wire claim is source-grounded on BOTH the Rust encoder and the Go port and confirmed by running code: no encode-time key sort (central thesis holds), the T6b fixture is fp-absent with `WalletPolicyId 7b716421…`, the decoded descriptor re-encodes byte-for-byte, identity is n-generic so a device-authored card binds. The two Minors (document the CLI depth-4 generator constraint + working xpub; vendor fresh `sh`-shape sortedmulti goldens) are fixture/plan footnotes. Adopt: struct constructor + return the `@N→fp`/`WalletPolicyIDStub` handle in Phase A. Proceed to IMPLEMENTATION_PLAN; fold the Minors + recommendations (no re-dispatch needed for GREEN).
