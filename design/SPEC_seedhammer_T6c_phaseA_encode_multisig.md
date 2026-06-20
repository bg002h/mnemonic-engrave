# SPEC — SeedHammer II T6c **Phase A**: headless `md.EncodeMultisig`

**Status:** DRAFT — awaiting opus R0 gate (must reach 0C/0I before any code).
**Author:** single spec author (T6c-A).
**Date:** 2026-06-19.
**Scope class:** internal Go fork feature (firmware `md/` package); NOT a `me` CLI surface, NOT a GUI change (those are Phase B).
**Fork HEAD:** `seedhammer` `8eb51d7` (Go 1.26.4). **Authoritative wire (Rust md-codec):** `descriptor-mnemonic` `c85cd49` (`md-codec` v0.36.0; CLI `md-cli` v0.7.0-5-g`c85cd49`).
**Grounding recon:** `design/cycle-prep-recon-T6c-encode-multisig.md` (verified vs Rust). This spec re-verified every load-bearing wire fact directly against the Rust ENCODER (`tree.rs`, `canonicalize.rs`, `tlv.rs`) and the running `md` CLI, per the external-protocol-fact rule.

---

## Why

The fork ships `md.EncodeSingleSig` (T6a, headless single-sig wallet-policy md1 assembler) but has **no** multisig encoder. T6b can only *consume* a supplied multisig md1 card; it cannot author one. T6c Phase A adds the headless wire core `md.EncodeMultisig` so a SeedHammer II device can assemble a full-policy `wsh(sortedmulti(k, ...))` (and the `sh(wsh(...))` / `sh(...)` wrappers) md1 from cosigner key material. Phase B (the GUI picker) is deferred (see OUT).

The recon's headline is confirmed in source: the bit-level multisig emitter **already exists, ships, and is byte-cost-tested** — `md/encode.go:188-203` `case multiKeysBody:` is byte-identical to Rust `tree.rs:115-139`. The whole `split → encodePayload → canonicalize → writeNode → writeTLVSection → computeEncodingID/deriveChunkSetID` pipeline is descriptor-shape-agnostic. There is **no encode-time key sort**. Identity (`computeEncodingID`, `WalletPolicyId`) is already n-generic. So `EncodeMultisig` is a ~90-140 LOC ASSEMBLER that mirrors `EncodeSingleSig` (107 LOC): build the canonical multi-key `*descriptor` literal, fill the per-@N TLVs, route through the shipped `split`.

The genuine risk is **not the wire format** (shipped + tested). It is **which descriptor the assembler builds** — specifically the **cosigner ordering** that fixes the @N placeholder assignment, because a wrong order yields a *valid but different* `Md1EncodingId`/`WalletPolicyId` → a card that round-trips locally but binds to a different policy than the cosigners expect. That ordering contract is what R0 must center on.

---

## Scope

### IN (Phase A)
1. A single exported headless function `md.EncodeMultisig(...)` in a new `md/encode_multisig.go`, no GUI deps, no secret bytes (caller passes parsed public key material), mirroring `encode_singlesig.go`.
2. Three top-level script wrappers over `sortedmulti(k, @0..@{n-1})`:
   - `wsh(sortedmulti(k, ...))` → root `tagWsh` ⊃ `tagSortedMulti` (P2WSH).
   - `sh(wsh(sortedmulti(k, ...)))` → root `tagSh` ⊃ `tagWsh` ⊃ `tagSortedMulti` (P2SH-P2WSH; `InnerWsh=true`).
   - `sh(sortedmulti(k, ...))` → root `tagSh` ⊃ `tagSortedMulti` (legacy P2SH; `InnerWsh=false`).
3. Per-cosigner TLV fill: N `idxPub` (65 B `chainCode‖compressedPubkey`), and **optional** per-cosigner `idxFP` (4 B). FP presence is a **caller choice** (see Verified-fact V8 — the T6b fixture carries NO fp TLV; an always-fp encoder would NOT byte-match it).
4. Origin handling: either a single **shared** origin (all cosigners share one path, e.g. `m/48'/0'/0'/2'`, `pathDecl.shared`) or **divergent** per-cosigner origins (`pathDecl.divergent`, len == n).
5. A deterministic-cosigner-ordering CONTRACT (Invariant I1), defined crisply below.
6. TDD acceptance: byte-exact-vs-Rust goldens (template-level + full-policy), the T6b fixture match, round-trip + identity invariants, and an assembler fuzz.

### OUT — Phase B / deferred (note only; NOT in this spec)
- The GUI picker (template/k/n/slot bounded `ChoiceScreen`), the choose-or-supply front door in `engraveMultisigFlow`.
- Cosigner NFC gather (N mk1 cards), threshold/slot UI, the user's-own-slot derive/insert (`deriveAccountXpub`).
- **Phase B HARD REQUIREMENT (user-mandated):** an **unskippable, loud on-device warning** — *"EXPERIMENTAL: this multisig builder is not validated end-to-end. Verify the assembled policy against your coordinator/cosigners BEFORE funding."* — gating any engrave of a device-authored multisig card. Must be non-deferrable and impossible to bypass.
- Unsorted `multi(k,...)` (tagMulti), `multi_a`/`sortedmulti_a` (taproot), tapscript trees, and any general miniscript. **OUT/REFUSED** — the encoder MUST refuse a shape it cannot round-trip (Invariant I5). Phase A emits `sortedmulti` only.

---

## Verified facts (file:line, BOTH Go fork @`8eb51d7` and Rust @`c85cd49`)

- **V1 — the multi-key bit emitter already exists and is byte-identical.**
  Go `md/encode.go:188-203` `case multiKeysBody:` writes `(k-1)` @5b, `(len(indices)-1)` @5b, then each `idx` @`kiw` width, with guards `k∈1..32`, `len∈1..32`, `k≤len`.
  Rust `tree.rs:115-139` `Body::MultiKeys { k, indices }`: `w.write_bits((k-1),5); w.write_bits((indices.len()-1),5); for idx { w.write_bits(idx, kiw) }`, same guards (`ThresholdOutOfRange`/`ChildCountOutOfRange`/`KGreaterThanN`). **Layout & guards identical; author-order indices.**
  Bit-cost test exists: Go `md/encode_test.go:137-148` `TestWriteNodeSortedMultiBitCost` — sortedmulti 2-of-3 @ n=3, kiw=2 = **22 bits** (Tag 6 + 5 + 5 + 3×2). Mirrors Rust `tree.rs:411`.

- **V2 — `kiw` is computed from `pathDecl.n` and must equal `descriptor.n`.**
  Go `md/encode.go:34-39` `kiw(n)= n<=1 ? 0 : 32 - LeadingZeros32(n-1)`; `encode.go:416` `width := kiw(dc.pathDecl.n)`; guarded by `encode.go:401-403` `if dc.pathDecl.n != dc.n { return errPathDeclNMismatch }`. Rust `encode.rs:37-41`. ⇒ the assembler MUST set `descriptor.n == pathDecl.n == n` (Invariant I-internal; see V7).

- **V3 — NO encode-time key sort. Only placeholder first-occurrence canonicalization.**
  Rust whole-crate search: the only sorts are STRUCTURAL TLV-index sorts (`canonicalize.rs:148` `sort_by_key(|(idx,_)| *idx)` re-sorts a TLV vec by *placeholder index* after the perm remap; `tlv.rs` emits by tag). `walk_collect_first` `Body::MultiKeys` arm (`canonicalize.rs:85-95`) and `remap_indices` `Body::MultiKeys` arm (`canonicalize.rs:131-136`) operate on `u8` placeholder indices, **never on key bytes**.
  Go mirror: `canonicalize.go:128-134` `walkCollectFirst case multiKeysBody:`; `canonicalize.go:167-172` `remapIndices case multiKeysBody:`; TLV re-sort `remapPubVec`/`remapFPVec` `canonicalize.go:197-209`. **The placeholder-index canonicalization is the ONLY permutation and is already shipped + tested.**
  To-miniscript preserves stored order (`to_miniscript.rs:198-248`, `build_multi_threshold` iterates indices in order — no sort). ⇒ no lexicographic key sort to reproduce; the residual is the *ordering contract* (I1), not a sort.

- **V4 — identity is n-agnostic; ZERO multisig-specific change needed.**
  `computeEncodingID` = `SHA-256(encodePayload(d))[0:16]` (Go `md/identity.go:11-17`; Rust `identity.rs:39-45`); `deriveChunkSetID` = top-20-bit extraction (Go `identity.go:31-33`; Rust `chunk.rs:175-179`). `WalletPolicyId` (Go `md/walletpolicyid.go:30-102`; Rust `identity.rs:172-240`) canonicalizes a clone, then loops `idx 0..n-1` (`walletpolicyid.go:48`) hashing `canonical_tree_bytes ‖ per-@N{presence, record, fp[4]?, xpub[65]?}` — the per-cosigner loop IS the multisig case, already generic. Uses RAW resolvers (`resolveOriginRaw`/`resolveUseSiteRaw`), not the display accessor (R0-I2 prior). ⇒ Invariant I6: identity wiring requires zero change; confirm + assert via tests.

- **V5 — tags & shape classification.** `tagWsh=0x02`, `tagSh=0x03`, `tagSortedMulti=0x07`, `tagMulti=0x06` (Go `md/md.go:42-47`; Rust `tag.rs:106-107`). The decoder summarizes these shapes (`md/md.go:1259-1315` `classifyPolicy`/`multiPolicy`); `summarize` sets `Template.InnerWsh` (`md.go:1322-1331` `innerWshNesting`) true iff root `Sh` with a single `Wsh` child — the P2SH-P2WSH vs P2SH discriminant.

- **V6 — `EncodeSingleSig` is the template to mirror.** `md/encode_singlesig.go:36-83`: builds `&descriptor{ n:1, pathDecl:{n:1, shared:&origin}, useSite:{hasMultipath, multipath {0},{1}, wildcardHardened:false}, tree, tlv:{pubPresent, pubkeys:[{0,xpub}], fpPresent, fingerprints:[{0,fp}]} }`, then `split(d)`. `singleSigTree` (`:92-106`) selects the per-shape tree node. **`EncodeMultisig` is the n>1 generalization.**

- **V7 — TLV idx columns must be strictly ascending pre-canonicalize, or `errOverrideOrder`.** `writeTLVSection` (`encode.go:271-311`) rejects `e.idx <= last` for fp/pub/origin/usesite entries. The assembler builds `idxPub[i]={idx:i}` and `idxFP[i]={idx:i}` for `i in 0..n-1` in order ⇒ already ascending; `canonicalize` re-sorts after any perm. ⇒ assembler must emit per-@N TLV entries idx-ascending.

- **V8 — FP presence is a per-cosigner choice; the T6b golden carries NO fp TLV.** Verified by decoding `gui/testdata/t6b_multisig_full.md1.txt`: a 6-chunk 2-of-3 `wsh(sortedmulti)`, **all 3 slots `FingerprintPresent=false` (fp=00000000)**, shared origin `m/48'/0'/0'/2'`, multipath use-site, `WalletPolicyId=7b716421db8b9f462967d04e0f8a3fd5`, stub `7b716421`. ⇒ `EncodeMultisig` MUST support an all-fp-ABSENT card to byte-match T6b. fp-present is golden-covered separately by `wsh_with_fingerprints` (TLV `fingerprints:[[0,deadbeef],[1,cafebabe]]`).

- **V9 — the non-circular golden generator builds & runs.** `descriptor-mnemonic` `target/debug/md encode '<template>' [--key @i=XPUB] [--fingerprint @i=HEX] --force-chunked --json`. `--key` takes a **base58check xpub** (parsed to the 65-B `chain_code‖compressed_pubkey` payload, `md-cli/src/parse/keys.rs:21,78`; bad checksum → reject). Template-only run verified: `wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*)) --force-chunked --json` → `{"chunk_set_id":"0x7b859","chunks":[...]}`. `md vectors` regenerates the corpus.

- **V10 — multi byte-goldens already vendored (Rust-sourced, non-circular).** `md/testdata/vectors/`: `wsh_sortedmulti` (`bytes.hex` template-only), `wsh_multi_2of3`, `wsh_multi_2of2`, `sh_wsh_multi` (root Sh⊃Wsh⊃Multi, `bytes.hex=2042001830860850`), `wsh_divergent_paths`, `wsh_with_fingerprints` (fp TLV), `wsh_multi_chunked`. Parity gate `md/testdata_test.go` (`byteParityVectorNames`). NB these template-only vectors use `tagMulti` not `tagSortedMulti` for some, but the bit layout is identical (V1) — `wsh_sortedmulti` is the sortedmulti template golden.

---

## Faithfulness spine (what the assembler is, end to end)

```
EncodeMultisig(cosigners[], k, script, originMode) 
  → build multiSigTree(script, k, n)                    // V5: tagWsh/tagSh wrappers ⊃ tagSortedMulti{k, [0..n-1]}
  → build *descriptor{ n, pathDecl{n, shared|divergent}, useSite{<0;1>/*}, tree, tlv{pubkeys[0..n-1], fingerprints?[present subset]} }
  → split(d)                                             // shipped, descriptor-shape-agnostic
      → encodePayload(d)                                 // canonicalize(clone) → validate → writeHeader → writePathDecl → writeUseSitePath → writeNode → writeTLVSection
          → canonicalize                                 // V3: placeholder first-occurrence renumber (here identity, since assembler emits @0..@{n-1} in order)
      → chunk @ csid                                     // V4: computeEncodingID/deriveChunkSetID, n-agnostic
  → []string (chunked md1)
```

`multiSigTree(script, k, n)` (mirrors `singleSigTree`):
- `wsh`     → `node{tagWsh, childrenBody{[ node{tagSortedMulti, multiKeysBody{k, [0..n-1]}} ]}}`
- `sh(wsh)` → `node{tagSh,  childrenBody{[ node{tagWsh, childrenBody{[ node{tagSortedMulti, multiKeysBody{k,[0..n-1]}} ]}} ]}}`
- `sh`      → `node{tagSh,  childrenBody{[ node{tagSortedMulti, multiKeysBody{k, [0..n-1]}} ]}}`

The indices `[0..n-1]` are written in cosigner-input order. `canonicalize` is the identity permutation for this AST (first-occurrence order IS [0,1,…,n-1]), so the placeholder layout is fixed entirely by **input order** ⇒ the ordering contract (I1).

### `EncodeMultisig` signature (proposed; R0 to confirm)
```go
// MultisigCosigner is one parsed PUBLIC cosigner key. ChainCode‖CompressedPubkey
// form the 65-byte Pubkeys TLV entry; Fingerprint is emitted only if FpPresent.
// Origin is the RAW BIP-32 origin ([]PathComponent, Hardened+bare value) used in
// divergent mode (ignored in shared mode).
type MultisigCosigner struct {
    ChainCode        [32]byte
    CompressedPubkey [33]byte
    Fingerprint      [4]byte
    FpPresent        bool
    Origin           []PathComponent
}

// MultisigScript selects the top-level wrapper over sortedmulti.
type MultisigScript int
const (
    MultisigWsh   MultisigScript = iota // wsh(sortedmulti(k,...))      → P2WSH
    MultisigShWsh                       // sh(wsh(sortedmulti(k,...)))  → P2SH-P2WSH
    MultisigSh                          // sh(sortedmulti(k,...))        → legacy P2SH
)

// EncodeMultisig assembles a sortedmulti k-of-n wallet-policy md1 over the given
// cosigners in CALLER ORDER (which fixes @0..@{n-1}; see the ordering contract).
// sharedOrigin != nil ⇒ Shared mode (cosigner.Origin ignored); sharedOrigin == nil
// ⇒ Divergent mode (each cosigner.Origin used; all must be non-empty).
// Returns the chunked md1 strings (>=2). Refuses unsupported shapes/params.
func EncodeMultisig(cosigners []MultisigCosigner, k uint8, script MultisigScript, sharedOrigin []PathComponent) ([]string, error)
```
*(R0 may prefer split shared/divergent constructors, or a struct param; the load-bearing contract is the cosigner-order semantics, not the exact Go surface.)*

### The DETERMINISTIC COSIGNER ORDERING CONTRACT (Invariant I1 — the R0 centerpiece)
- **There is no encode-time key sort (V3).** `canonicalize` renumbers @N to *document/first-occurrence* order, which for this assembler's AST is exactly the cosigner-input order. Therefore **the @N placeholder assignment == the order in which the caller supplies cosigners**: `cosigners[0]` → @0, `cosigners[1]` → @1, … `cosigners[n-1]` → @{n-1}.
- **Consequence:** two callers supplying the same N keys in DIFFERENT orders produce DIFFERENT (both valid) md1 cards with DIFFERENT `Md1EncodingId`/`WalletPolicyId`. Both round-trip locally; only the order that matches the cosigners' coordinator binds to the shared policy.
- **The contract `EncodeMultisig` enforces / documents:** the encoder is a faithful, order-preserving assembler — it does NOT reorder, sort, or "fix" cosigner order. It is the CALLER's responsibility (Phase B picker) to supply cosigners in the order that matches the coordinator's policy. The recommended Phase-B deterministic rule (to be locked in Phase B's spec, NOT here) is: **preserve the order the cosigner cards are gathered / the user explicitly assigns slots**, with an on-device confirm screen showing k-of-n and each slot's fp/origin before engrave. Phase A's obligation is only to (a) be exactly order-preserving and (b) document this loudly in the godoc so no caller assumes a hidden sort.
- **R0 question to resolve:** should `EncodeMultisig` additionally expose/return the assigned @N→fingerprint mapping (or the resulting `WalletPolicyIDStub`) so the caller can verify ordering against a coordinator's expected stub *before* engrave? (Recommended: yes, as a cheap determinism guard — but it may live in Phase B. Flag for R0.)

---

## Acceptance gate (TDD — tests before impl, in `md/encode_multisig_test.go` + `md/encode_multisig_fuzz_test.go`)

Mirror the `encode_singlesig_test.go` structure (vector loader, payload-parity gate, string-equality gate, round-trip gate, fuzz).

1. **A1 — byte-exact template-level parity vs Rust goldens.** Build the bare `sortedmulti` AST for each wrapper at n=2/3, encode, reassemble, and assert `encodePayload` bytes match the relevant `md/testdata/vectors/*.bytes.hex` (e.g. `wsh_sortedmulti`, `sh_wsh_multi`) — Rust-origin, non-circular. (Where a vendored vector is `tagMulti` not `tagSortedMulti`, assert the bit layout via the cost test V1 and add a fresh `md encode 'wsh(sortedmulti(...))'`-generated golden.)
2. **A2 — byte-exact FULL-POLICY parity vs a freshly `md encode`-generated golden.** Generate a 2-of-3 `wsh(sortedmulti)` md1 with `md encode '...' --key @0=XPUB0 --key @1=XPUB1 --key @2=XPUB2 [--fingerprint ...] --force-chunked --json` (V9), vendor it under `md/testdata/vectors/`, and assert `EncodeMultisig` (fed the same 65-B payloads / fps / origin / order) produces **byte-identical** chunk strings (and reassembled payload). Cover `sh(wsh(sortedmulti))` and `sh(sortedmulti)` (the InnerWsh discriminant).
3. **A3 — T6b fixture equality.** Fed the three cosigners decoded from `gui/testdata/t6b_multisig_full.md1.txt` (all fp-ABSENT — V8; shared origin `m/48'/0'/0'/2'`; the exact 65-B xpubs documented in `gui/multisig_testhelpers_test.go:18-19` for @0/@2 and the abandon-seed key for @1, in that order), `EncodeMultisig` must reproduce the fixture **byte-for-byte** AND yield `WalletPolicyId=7b716421…`. This is the strongest end-to-end gate: it proves a device could re-author the exact T6b card.
4. **A4 — round-trip + identity invariants.** `EncodeMultisig(...)` → `DecodeChunks` / `ExpandWalletPolicyChunks` recovers byte-identical template (root/policy/k/n/InnerWsh) and per-@N {xpub, fp(+presence), origin, use-site}; and `WalletPolicyIdChunks` / `Md1EncodingId` of the output match what a `Reassemble` of the same wire yields (identity zero-change, I6).
5. **A5 — fp-present and divergent-origin coverage.** One golden with the per-cosigner fp TLV present (parity vs `wsh_with_fingerprints` layout / a `md encode --fingerprint`-generated full-policy golden), and one with `pathDecl.divergent` per-cosigner origins (parity vs `wsh_divergent_paths`).
6. **A6 — refuse-unsupported.** Assert typed errors for: `k>n`, `k<1`/`k>32`, `n<1`/`n>32`, divergent-origin-count ≠ n, empty shared origin where required, and any non-sortedmulti request (Phase A emits sortedmulti only). Reuse the shipped guards (`errThresholdRange`, `errChildCount`, `errKGreaterThanN`, `errDivergentCount`, `errKeyCountRange`).
7. **A7 — fuzz the assembler.** `FuzzEncodeMultisig` (mirror `FuzzEncodeSingleSig`): arbitrary (n, k, per-cosigner cc/pk/fp/fpPresent, script, originMode) → a successful encode must round-trip via `ExpandWalletPolicyChunks` recovering inputs in order, with no panic (off-curve pubkey rejected at decode = benign skip).

---

## Invariants (R0 confirms each)

- **I1 — Cosigner-ordering contract.** `EncodeMultisig` is exactly order-preserving: `cosigners[i]` → placeholder @i. No hidden sort. Different caller order → different (valid) `WalletPolicyId`. Documented loudly; the caller owns coordinator-matching order.
- **I2 — Round-trip identity.** `EncodeMultisig` output decodes back to byte-identical template + per-@N keys (A4).
- **I3 — Byte-exact vs Rust.** Template-level (A1) and full-policy (A2) bytes are byte-identical to Rust-md-codec goldens, and to the T6b fixture (A3).
- **I4 — No key sort.** Only the shipped placeholder first-occurrence canonicalization runs (identity perm for this AST). The encoder never reorders key material (V3).
- **I5 — Refuse unsupported shape.** The encoder emits ONLY `sortedmulti` under wsh/sh/sh-wsh; it refuses (typed error) `multi`, `multi_a`, taproot trees, complex miniscript, out-of-range k/n, and any shape it cannot round-trip (A6).
- **I6 — Identity zero-change.** `computeEncodingID`/`deriveChunkSetID`/`WalletPolicyId` are used UNCHANGED; no multisig-specific identity code is added (V4; asserted by A4).
- **I7 — kiw/n lockstep.** `descriptor.n == pathDecl.n == n`, so `kiw` is correct; the shipped `errPathDeclNMismatch` guard backs this (V2).
- **I8 — TLV idx ascending.** Per-@N pub/fp/origin TLV entries are emitted idx-ascending; canonicalize re-sorts after any perm (V7).

---

## Risks

1. **(MED) Cosigner-ordering misuse by the caller.** The #1 risk (I1). A Phase-B picker that supplies cosigners in a non-deterministic / non-coordinator order mints a valid-but-wrong-binding card. *Phase-A mitigation:* be strictly order-preserving + document loudly + (R0 to decide) optionally return the @N→fp map / `WalletPolicyIDStub` so the caller can verify before steel. *Phase-B mitigation:* deterministic gather order + confirm screen + the mandated EXPERIMENTAL warning.
2. **(LOW) fp-presence mismatch.** If the assembler always emits fp, it won't byte-match the T6b golden (V8). Mitigated by per-cosigner `FpPresent` + A3.
3. **(LOW) Encoder-core byte-parity.** Fully de-risked: shipped `writeNode`/`canonicalize` + existing multi byte-goldens + the `md encode` generator. Residual: shared-vs-divergent path-decl and TLV idx ordering — both golden-covered (A5; `wsh_divergent_paths`, `wsh_with_fingerprints`).
4. **(LOW) Template-scope creep.** Keep to sortedmulti under wsh/sh/sh-wsh; defer the rest (I5).
5. **(LOW) Golden vendoring drift.** Re-pin `md/testdata/README.md` SHA if md-codec advances past `c85cd49`; any md1 multisig wire change must re-verify vs Rust.

---

## Gate

This is a brainstorm SPEC, not a plan and not code. It must pass an opus architect **R0 review to 0 Critical / 0 Important** before any IMPLEMENTATION_PLAN or code. Fold findings → persist the review verbatim to `design/agent-reports/` → re-dispatch after every fold until GREEN. R0 is expected to center on the **cosigner-ordering contract (I1)** and the exact `EncodeMultisig` surface (struct-vs-positional params; whether to return the @N→stub verification map in Phase A vs Phase B).
