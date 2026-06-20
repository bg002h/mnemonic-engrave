# cycle-prep recon — 2026-06-19 — T6c-encode-multisig (`md.EncodeMultisig` + on-device picker)

**Mode:** RECON ONLY (no spec, no code). Cycle-prep STRICT-GATE discipline applied to a net-new-encoder feasibility study.

**Fork (Go) HEAD:** `8eb51d7` (`seedhammer`, branch `main`, == `origin/main`, clean tree). Go 1.26.4.
**Authoritative wire format (Rust md-codec) verified against:** `/scratch/code/shibboleth/descriptor-mnemonic`, HEAD **`c85cd49`** (`descriptor-mnemonic-md-cli-v0.7.0-5-gc85cd49`; the vendored golden README pins the crate as md-codec **v0.36.0** @ `c85cd49`). Crate path `crates/md-codec`; CLI `crates/md-cli`.
**Toolkit (higher-level synth):** `/scratch/code/shibboleth/mnemonic-toolkit` @ `4e21d94` (`v0.58.1-14`).
**Grounding doc:** `design/agent-reports/seedhammer-T6-architect-scope-multisig.md` (mechanism i analysis).

Verification method: 4 parallel recon agents, every external-protocol fact cross-checked Go-port ↔ Rust source (per the ultracode external-fact rule); load-bearing claims (key-sort, encoder-existence, identity-agnosticism, golden provenance, lockstep guard) independently re-verified by reading the source directly and by building+running the Rust `md` CLI.

---

## HEADLINE VERDICT (the build-or-not numbers)

**The grounding doc materially OVERSTATES the size and risk of `md.EncodeMultisig`.** Three of its premises are STRUCTURALLY-WRONG or DRIFTED against current source:

1. **"BIG new encoder, several× `EncodeSingleSig`"** → **WRONG.** The bit-level multisig encoder *already exists, is shipped, and is byte-cost-tested* (`encode.go:188-203`, `encode_test.go:137`). `EncodeMultisig` is a thin ~assembler (build a multi-key `*descriptor` literal → call `split`), essentially the same shape and size as the shipped `EncodeSingleSig` (`encode_singlesig.go`, 107 LOC).
2. **"canonical KEY-SORT permutation (the highest-risk piece)"** → **MISATTRIBUTED.** The Rust md-codec does **NOT** lexicographically sort cosigner keys at encode time. `sortedmulti`'s "sorted" is a *spend-time* miniscript/BIP-67 semantic, not an encode-time reorder of the stored key list. The only permutation that exists is **placeholder-index first-occurrence canonicalization** (@0..@{n-1} in document order) — and that is *already ported and tested* (`canonicalize.go`, covering the `multiKeysBody` case). There is **no new sort permutation to reproduce byte-exactly.**
3. **"the device GENERATES multisig md1, so where do goldens come from?"** → **ALREADY SOLVED.** A full-policy, xpub-complete 2-of-3 `wsh(sortedmulti)` md1 golden already exists in the fork (`gui/testdata/t6b_multisig_full.md1.txt`), and Rust-sourced byte-parity payload goldens for multisig shapes are already vendored (`md/testdata/vectors/wsh_*`). The non-circular generator (`md encode … --force-chunked`) builds and runs.

Net effect: T6c's encoder core is **LOW risk and SMALL**; the real cost and the real risk shift to the **on-device picker/gather UX** (the human-factors surface), not the wire format.

---

## Per-topic verified facts (Go ↔ Rust, file:line on both sides)

### Topic 1 — What `EncodeSingleSig` does NOT already do (the net-new for multi)

`EncodeSingleSig` (`md/encode_singlesig.go:36-83`) builds a single-key `*descriptor` literal (`n:1`, `pathDecl.shared`, one `idxPub`/`idxFP`, tree per shape) and routes through `split(d)` (`md/chunk.go:121`). The **entire downstream pipeline is descriptor-shape-agnostic**:

- `split` → `encodePayload` (`md/encode.go:374`) → `canonicalize` (`md/canonicalize.go:24`) → `writeNode` (`md/encode.go:159`) → `writeTLVSection` (`md/encode.go:247`) → `computeEncodingID`/`deriveChunkSetID` (`md/identity.go:11,31`).
- **The `writeNode` `case multiKeysBody:` arm ALREADY EXISTS** (`md/encode.go:188-203`): emits `(k-1)` 5b, `(n-1)` 5b, then N raw `kiw`-width indices, with `k,n ∈ 1..32` and `k≤n` guards. Bit-cost verified at `md/encode_test.go:137-148` (`sortedmulti 2-of-3 @ n=3, kiw=2 = 22 bits`) against Rust `tree.rs:411/414-424`.
- Rust mirror (`crates/md-codec/src/tree.rs:115-139`): `w.write_bits((k-1),5); w.write_bits((indices.len()-1),5); for idx { w.write_bits(idx, kiw) }` — **byte-identical layout, author-order indices.**
- Decode mirror (Go `md/md.go:386-408`; tags `tagMulti=0x06`, `tagSortedMulti=0x07` at `md/md.go:46-47` == Rust `tag.rs:106-107`).

**So `EncodeMultisig` net-new = ONLY a new assembler** that the single-sig path doesn't have:
- a `multiSigTree(script, k, n)` helper → `node{tagWsh, childrenBody{[node{tagSortedMulti, multiKeysBody{k, [0..n-1]}}]}}` (and `tagSh`/`sh(wsh(...))` variants), mirroring `singleSigTree` (`encode_singlesig.go:92-106`);
- a multi-key TLV fill: N `idxPub` (65B `chainCode‖compressedPubkey` each) + N `idxFP` (4B each), `idx 0..n-1`;
- origin handling: `pathDecl.shared` (all cosigners share `m/48'/0'/0'/2'`) **or** `pathDecl.divergent` (per-cosigner origins) — the divergent path-decl writer already exists (`md/encode.go:103-125`) and is golden-tested (`wsh_divergent_paths` vector).
- **No new bit-emit, no new identity code, no new sort.** Threshold `k` and N flow as plain `multiKeysBody` fields; `kiw` is computed from `pathDecl.n` (`md/encode.go:416`, `kiw()` at `:34`).

### Topic 2 — The key-sort permutation (the alleged #1 risk) — VERDICT: NO ENCODE-TIME KEY SORT

- **Rust (authoritative):** searched the whole crate for `sort*`/`Ord`/`cmp`/`lexicograph`. The only sorts are **structural TLV-index sorts**, never key/xpub/pubkey byte sorts:
  - `canonicalize.rs:148` `entries.sort_by_key(|(idx,_)| *idx)` — re-sort TLV vectors by *placeholder index* after the perm remap.
  - `tlv.rs:200` `entries.sort_by_key(|(t,_,_)| *t)` — TLV emit order by *tag number*.
  - `bch_decode.rs:444` decode-side index pairing.
  - `canonicalize.rs` `walk_collect_first` (~45-98) + `canonicalize_placeholder_indices` (~168-248): operate on `u8` **placeholder indices**, NOT key bytes. The `Body::MultiKeys` arm permutes `indices` in place (`remap_indices` ~131-135) — it never sorts the key material.
- **Go (port, already shipped):** `md/canonicalize.go` mirrors this exactly — `walkCollectFirst` `case multiKeysBody:` (`:128-134`), `remapIndices` `case multiKeysBody:` (`:167-172`), TLV re-sort by idx (`remapPubVec`/`remapFPVec` `:197-209`). The grounding doc's citation "`canonicalize.go:103,143-144`" points at the `keyArgBody` (single-key) arms; the multi arms are `:128-134` and `:167-172`. **The placeholder-index canonicalization is the only permutation, and it is done.**
- **Consequence:** `EncodeMultisig` must preserve **caller-supplied cosigner order** and let the shipped `canonicalize` renumber placeholders to first-occurrence order. There is **no lexicographic key sort to get byte-exactly right.** This collapses the expected #1 risk.
- **One real residual (downgraded from "sort" to "ordering contract"):** the caller (the picker) decides which cosigner is `@0,@1,…`. Different caller orderings → different (still valid) placeholder layouts → **different `Md1EncodingId`/`WalletPolicyId`**. This is not a *correctness* bug (it round-trips), but a *determinism/identity* contract: the device must order cosigners deterministically (e.g. by gather order, or by a documented rule) so a re-run reproduces the same card. To-miniscript preserves stored order with NO sort (`to_miniscript.rs:198-248`, `build_multi_threshold` iterates indices in order).

### Topic 3 — Identity interaction — VERDICT: n-AGNOSTIC, ZERO multisig-specific change

- `computeEncodingID` (`md/identity.go:11-17`): `SHA-256(encodePayload(d))[0:16]` — hashes the canonical payload bytes, no shape assumption. `deriveChunkSetID` (`:31-33`): pure top-20-bit extraction. Both verified n-agnostic. Rust mirror `identity.rs:39-45` + `chunk.rs:175-179`.
- `WalletPolicyId` (`md/walletpolicyid.go:30-102`): canonicalizes a clone, loops `idx 0..n-1` (`:48`), hashes `canonical_template_tree_bytes ‖ per-@N{presence, record, fp[4], xpub[65]}` — **already n-generic** (the per-cosigner loop is the multisig case). Rust mirror `identity.rs:172-240`. Uses raw resolvers (`resolveOriginRaw`/`resolveUseSiteRaw`), NOT the display accessor (R0-I2 already handled).
- **Round-trip contract for the future build:** `EncodeMultisig → split → Reassemble`(`md/chunk.go:207`)`/ExpandWalletPolicyChunks`(`md/expand.go:102`)` → identical Template + []ExpandedKey + ids`. `Reassemble` already re-derives csid from the decoded descriptor and compares to the header (`chunk.go:284-291`) — the integrity gate is shape-agnostic.
- **Binding:** a device-authored multisig md1 will produce the same `WalletPolicyId` that an mk1 stub and the constellation expect, **provided** the picker feeds a canonical-able multi-key descriptor (which `split`→`canonicalize` enforces) AND the cosigner ordering matches what the eventual decoder/peers expect (Topic 2 residual).

### Topic 4 — Golden source — VERDICT: SOLVED, non-circular, partly already vendored

- **Already in the fork (Rust-sourced, non-circular):** `md/testdata/vectors/` (vendored from md-codec `c85cd49` per `md/testdata/README.md`) includes multisig payload byte-goldens: `wsh_sortedmulti` (template-only 2-of-3, `bytes.hex=2082001821c22180`), `wsh_multi_2of3`, `wsh_multi_2of2`, `sh_wsh_multi`, `wsh_divergent_paths`, `wsh_with_fingerprints` (carries fp TLV), `wsh_multi_chunked` (the force-chunked multi, IN the `byteParityVectorNames` parity gate — `md/testdata_test.go:31`). These prove `encodePayload` byte-parity for multi shapes TODAY.
- **Full-policy xpub-complete golden ALREADY exists:** `gui/testdata/t6b_multisig_full.md1.txt` — a 6-chunk 2-of-3 `wsh(sortedmulti(2,@0,@1,@2))`, all 3 slots xpub-present, origin `m/48'/0'/0'/2'` (slot @1 = the abandon-about seed; @0/@2 foreign). Consumed by `gui/multisig_testhelpers_test.go` (`suppliedMultisigMd1`, `TestSuppliedMultisigFixtureIsFullPolicy`). Its header comment forbids ad-hoc regeneration and points to the documented Rust descriptor.
- **Authoritative non-circular generator (verified to BUILD + RUN):** `md-cli` `Encode` subcommand (`crates/md-cli/src/main.rs:74-126`, impl `cmd/encode.rs`):
  ```
  md encode 'wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))' \
      --key @0=<XPUB0> --key @1=<XPUB1> --key @2=<XPUB2> \
      --fingerprint @0=<FP0> --fingerprint @1=<FP1> --fingerprint @2=<FP2> \
      --force-chunked --json
  ```
  Built `target/debug/md` cleanly (`cargo build -p md-cli` EXIT=0); ran the template-only form → `{"chunk_set_id":"0x7b859","chunks":[...]}`. `md vectors` (`main.rs:170`) regenerates the whole corpus. Higher-level: toolkit `synthesize_multisig_full` (`mnemonic-toolkit/.../synthesize.rs:374-513`) emits a full Bundle{ms1,mk1,md1} from a seed.
- **Future-build exec-review gate:** diff `EncodeMultisig` payload bytes against `wsh_sortedmulti.bytes.hex` / `wsh_multi_2of3.bytes.hex` (template-level) AND against a freshly `md encode …`-generated full-policy 2-of-3 (xpub+fp) golden. Both are Rust-origin → non-circular vs the Go code under test.

### Topic 5 — On-device picker + cosigner gather + program-enum lockstep

- **Gather (reuse T5/T6b):** `bundleGatherer` (`gui/bundle.go:119-123`); `offerChunkedMK1` (`:174-214`) accumulates a chunked mk1 → `mk.Decode` → `mk.Card{Xpub(base58), Fingerprint, Path}` (`mk1Summary` `:300-307`). T6b already gathers a supplied md1 and cross-matches the user's slot (`gui/multisig.go:35-130`, `findUserSlot`, `extractSuppliedMd1`). For T6c, gather N **mk1** cosigner cards instead of one md1; reuse the same gatherer + `bundleGatherFlow`.
- **User's own slot:** `deriveAccountXpub(m, passphrase, net, path) (xpub string, masterFP uint32, err)` (`gui/derive.go:19-53`); R0-C1 ordering note (`:46-49`) — serialize xpub before zeroing. Insert the user's key at the chosen `@N`.
- **Picker UX (no free-form numeric widget — confirmed):** compose bounded `ChoiceScreen` (`gui/gui.go:1359-1423`, `.Choose`); two-level picker precedent `gui/singlesig_pick.go:49-77`. Threshold k → `[]string{"1".."n"}`; script shape → reuse `scriptName`/`md.ScriptKind` (`gui/md1_inspect.go:20-33`, `md/md.go:1165-1179`); policy display `policyLine` (`md1_inspect.go:36-51`). Template scope in-scope: `wsh(sortedmulti)` k-of-n and `sh(wsh(sortedmulti))`/`sh(sortedmulti)` (the shapes the decoder + `expandedToDescriptor` already handle); bare miniscript = OUT (defer).
- **Program-enum LOCKSTEP (cite at HEAD `8eb51d7`):** enum `gui/gui.go:147-155` (`backupWallet…engraveSingleSig, engraveMultisig, bip85Derive, qaProgram`). **NOTE — refines the task's premise:** an `engraveMultisig` program ALREADY EXISTS (`:152`, the T6b supply-md1 flow `engraveMultisigFlow`). T6c can either (a) **extend `engraveMultisigFlow`** with a front-door ChoiceScreen ("Supply policy card" vs "Build policy (pick)") — **no enum change, no guard bump**; or (b) add a **new** navigable program. The t5-M1 compile-time guard is `var _ [1]struct{} = [qaProgram - bip85Derive]struct{}{}` (`:164`); `bip85Derive` must stay last-navigable. If (b), insert the new program **before** `bip85Derive`, and update the lockstep sites: dispatch switch (`:1502-1527`), title switch (`:1680-1693`), `layoutMainPlates` case list (`:1875-1883`), carousel wrap `m.prog>bip85Derive` (`:1662`), and the `npage`/`npages = int(bip85Derive)+1` consts (`:1867,:1886`) — the last two auto-track once the new const sits below `bip85Derive`. **Recommended: option (a)** — it satisfies the lockstep with zero guard churn and mirrors the choose-or-supply intent.

### Topic 6 — Security spine (verified)

- Classification `gui/scan.go:70-73`: ms1 (codex32 secret) → SECRET, **NFC-refused** (`gui/bundle.go` `classify` → `clsMs1Refuse`); md1/mk1 → `mdmkText` (BCH-validated) → PUBLIC, NFC-allowed.
- Cosigner xpubs arrive as PUBLIC mk1 over NFC; the user's seed is **typed-only** (`seedEntryFlow`, never a scan — `multisig.go:62`, scrubbed on every exit `:71-75`); ms1 is steel-only; the assembled md1 is PUBLIC (xpub+fp+origin+threshold only).
- **No new secret residency** introduced by the picker: it assembles public policy material. The user's seed touches derivation only via `deriveAccountXpub` (scrubs internally) to mint the user's own slot key — identical posture to T6b. If the picker also engraves a full bundle, the existing `deriveMultisigLeg` scrub path applies.

---

## Sizing (LOC estimate, Go; tests excluded from core)

| Component | Est. LOC | Risk | Notes |
|---|---:|---|---|
| `EncodeMultisig` assembler core (`encode_multisig.go`: multiSigTree shapes + multi-key TLV fill + origin shared/divergent + `split` route) | **~90-140** | **LOW** | mirrors `encode_singlesig.go` (107); reuses shipped `writeNode`/`split`/`canonicalize`/identity |
| Key-sort / permutation | **0** | **N/A** | no encode-time key sort exists; placeholder canonicalization already shipped |
| Identity wiring | **0** | **NONE** | `computeEncodingID`/`WalletPolicyId` already n-agnostic |
| Picker + cosigner-gather UX (ChoiceScreen composition: template/k/n/slot; gather N mk1; insert user slot; cosigner-order contract; front-door choose-or-supply) | **~250-400** | **MED-HIGH** | the real cost; human-factors + bounded-widget assembly; reuses `bundleGatherer`, `deriveAccountXpub` |
| Program-enum lockstep | **~5-15** (option a) / **~30** (option b) | **LOW** | option (a) = a front-door ChoiceScreen in `engraveMultisigFlow`, no guard bump |
| Tests + goldens (byte-parity vs Rust `md encode`; round-trip; picker flow tests; reuse t6b fixture + add a built full-policy golden) | **~250-400** | **MED** | golden gen is solved + non-circular |
| **Total** | **~600-950** | | encoder-core slice is the small, low-risk part |

## Ranked risks

1. **(MED) Picker UX correctness & cosigner-ordering determinism** — *the new #1.* Wrong `@N` placement of the user's own key, or a non-deterministic cosigner order, yields a valid-but-different `WalletPolicyId` → a card that binds to a *different* policy id than peers expect (round-trips locally, fails to match cosigners). Mitigation: deterministic, documented cosigner-ordering rule + an on-device confirm screen showing k-of-n + each slot's fp/origin before engrave; verify-bundle vs the user's slot.
2. **(MED) Picker mis-derivation of the user's slot** — wrong origin path / network / passphrase → wrong xpub on permanent steel. Mitigation: reuse `deriveAccountXpub` (R0-C1 scrub-ordering already handled) + display the derived fp/xpub for confirm.
3. **(LOW) Encoder-core byte-parity** — fully de-risked by shipped `writeNode`/`canonicalize` + existing multi byte-goldens + the `md encode` generator. Residual: getting `pathDecl.shared` vs `divergent` and the TLV idx ordering right — both already golden-covered (`wsh_divergent_paths`, `wsh_with_fingerprints`).
4. **(LOW) Lockstep drift** — only if option (b); the compile-time guard fails the build if missed, so it self-flags.
5. **(LOW) Template-scope creep** — keep to `wsh/sh(wsh)/sh sortedmulti`; defer general miniscript.

Every wrong byte that reaches steel is a permanent-correctness bug; but the byte surface is the *shipped, tested* encoder — the new exposure is overwhelmingly **which** descriptor the picker assembles (slot/order/origin), not **how** it serializes.

## Headless-first split — YES, clean

The encoder **is self-contained and headless** (like #10a / `EncodeSingleSig`): `EncodeMultisig(...keys, fps, origins, k, script) ([]string, error)` lives entirely in `md/`, no GUI deps, no secret bytes (callers pass parsed public key material). Recommended phasing, mirroring the #10 / T6a split:
- **Phase A (headless `md.EncodeMultisig`):** the assembler + byte-parity tests vs Rust goldens + round-trip + fuzz. Small, low-risk, fully gate-able in isolation (its R0 focuses on the multi-key descriptor literal + the cosigner-ordering contract).
- **Phase B (GUI picker):** extend `engraveMultisigFlow` with the choose-or-supply front door + cosigner-mk1 gather + bounded-ChoiceScreen template/k/slot picker + user-slot derive/insert + confirm/engrave/verify/restore-doc. This is where the MED risk and most LOC live.

## Lockstep sites (for the eventual plan)

- `gui/gui.go:147-155` program enum; `:164` t5-M1 compile-time guard; dispatch `:1502-1527`; titles `:1680-1693`; `layoutMainPlates` `:1875-1883`; carousel wrap `:1662`; `npage`/`npages` `:1867,:1886`. **Option (a) touches none of these except adding a ChoiceScreen branch inside `engraveMultisigFlow` (`gui/multisig.go:35`).**
- Golden corpus: `md/testdata/vectors/` (re-pin README SHA if md-codec advances); `gui/testdata/t6b_multisig_full.md1.txt` (do not regen ad hoc).
- md-codec cross-pin: any md1 multisig wire change must re-verify vs `descriptor-mnemonic` (currently `c85cd49`).

## SemVer / classification

Internal fork feature (firmware), not a CLI surface; no `me` clap-flag or GUI `schema_mirror` lockstep is triggered by the Go encoder itself. If/when surfaced via the `me` CLI in `mnemonic-engrave`, an additive subcommand = MINOR.

---

## Recommendation

**Build-or-not = BUILDABLE and cheaper than scoped.** The "big golden-locked encoder" framing is stale: the encoder core is ~100 LOC of low-risk assembly over a shipped, byte-tested pipeline, with no key-sort to reproduce and zero identity changes. The genuine cost/risk is the on-device picker UX (cosigner gather, bounded-widget k/n/slot selection, ordering determinism), which is independent of the wire format. Recommend a clean **Phase-A headless `EncodeMultisig` → Phase-B GUI picker** split, with Phase A's R0 centered on the descriptor-literal + cosigner-ordering contract (not a nonexistent key-sort), and Phase B's R0 centered on slot placement / derivation / confirm-before-steel.
