# SPEC — T6b: multisig/miniscript bundle via SUPPLIED md1 (slot cross-match, verbatim engrave)

**Status:** for opus R0 gate (0C/0I before code).
**Fork base:** `072461a` (T6a complete). **Fork-side only; no upstream PR.**
**Feeds from:** `design/cycle-prep-recon-T6b-supplied-md1.md` + `design/agent-reports/seedhammer-T6-architect-scope-multisig.md` (mechanism ii) + `…-recon-bundle-composition-stub.md`. **USER decision (2026-06-19):** the user SUPPLIES multisig/miniscript policies as md1 strings (the device never builds them).

## 1. Why / context
The final T6 piece. A multisig/miniscript wallet needs the OTHER cosigners' keys, which one seed can't produce — so the user SUPPLIES a complete wallet-policy md1 (made elsewhere: the toolkit, another SH). The device derives the user's OWN key from the typed seed, CROSS-MATCHES it to one of the descriptor's @N slots, then engraves the supplied md1 VERBATIM + the user's mk1 (policy-bound stub) + ms1. **ZERO new encoder** (architect mechanism ii) — the md1 is engraved as-is; the only net-new logic is the slot cross-match. The multisig restore + address derivation are already shipped (`expandedToDescriptor` + `address` handle sortedmulti).

## 2. Scope

### IN
- **A new top-level `engraveMultisig` program** (between `engraveSingleSig` and `qaProgram`; 8-site lockstep + 3 nav-tests updated; `qaProgram` stays debug-only).
- **Gather the supplied md1** over NFC via the T5 `bundleGatherFlow` → the single `cardMD1` (verbatim `[]string`); decode via `md.ExpandWalletPolicyChunks` → `Template` + per-@N `[]ExpandedKey`.
- **Full-policy gate:** the supplied md1 MUST be a full wallet policy — every slot `XpubPresent`. Template-only / any-slot-missing-xpub → REFUSE ("the supplied descriptor has no public keys to match").
- **Typed-only seed** (`seedEntryFlow`; NEVER scan→derive) + optional passphrase.
- **Slot cross-match (`findUserSlot`, NET-NEW, D14 Critical):** for each `XpubPresent` slot, derive the user's account key from the seed AT that slot's `OriginPath` (`deriveAccountXpub`), decode to the canonical `(chainCode[32], compressedPubkey[33])` (`decodeXpubBytes`), compare to `slot.Xpub[0:32]`/`[32:65]` (NEVER base58 — the supplied xpub carries different parentFP/depth metadata). Exactly-one match → the user's slot (record index + origin); ZERO → REFUSE; ambiguous (≥2, reused key) → engrave the FIRST-by-index slot's mk1 (deterministic; policy+stub identical across slots, only mk1 Path differs) + a "key reused at slots @i,@j" notice.
- **Derive the user's leg:** mk1 = `mk.Encode(Card{Path: matchedOrigin.String(), Fingerprint: hex(masterFP), Stubs: [][4]byte{WalletPolicyIDStubChunks(suppliedMd1)}, Xpub})` (the stub binds to the SUPPLIED policy); ms1 = `EncodeMS1(m.Entropy())` (full) — gate `m.Valid()` first.
- **Engrave (full / watch-only):** full = supplied md1 (verbatim) + user mk1 + user ms1; watch-only = supplied md1 + user mk1 (skip ms1) + the ms1 reminder. Reuse `bundleEngrave`.
- **verify-bundle (user's slot only):** re-type seed → re-cross-match → re-derive the user's leg → `bundle.Verify` (unchanged — stub-binding + mk1 fp/xpub/path + md1-exact-string + ms1-entropy; watch-only both-empty-ms1 skip). Other slots are public-given/unverified-by-design.
- **restore doc:** for the bip380-expressible (sortedmulti) subset via `expandedToDescriptor` → `address.Receive/Change`; non-bip380 (unsorted multi/multi_a/taptree/complex miniscript) → display the descriptor read-only ("addresses unavailable"). Display-only, no secret.
- **Mainnet-only** (the cross-match is network-agnostic; derive on mainnet, matching T6a/#10b).

### OUT (deferred)
- On-device policy AUTHORING / the picker + `md.EncodeMultisig` (T6c — out of scope, FOLLOWUPS). Self-multisig (constellation hard-rejected). Testnet (follow-on). The device NEVER constructs a multisig md1 — it only engraves a supplied one verbatim.

## 3. Verified facts (cite source; full detail in the recon)
Reuse: `bundleGatherFlow` (`gui/bundle_flow.go:95`), `bundleCard`/`cardMD1`/`cardMK1`/`cardMS1` (`gui/bundle.go:24-38`), `md.ExpandWalletPolicyChunks` (`md/expand.go:102`) + `ExpandedKey{Index,OriginPath,Xpub [65]byte,XpubPresent,Fingerprint}` (`:56-64`), `deriveAccountXpub` arbitrary-path (`gui/derive.go:19,33`), `decodeXpubBytes` (`gui/singlesig_derive.go:99`), `mk.Encode`/`mk.Card` (`mk/encode.go:38`,`mk/mk.go:133-139`), `md.WalletPolicyIDStubChunks` (`md/walletpolicyid.go:126`), `codex32.EncodeMS1` (`codex32/msencode.go:17`), `bundleEngrave` (`gui/bundle_flow.go:327`) + `singleSigEngraveCards` shape (`gui/singlesig_engrave.go:20`), `bundle.Verify` (`bundle/verify.go:32`), `expandedToDescriptor` (`gui/md1_expand.go:32`, sortedmulti→P2WSH/P2SH/P2SH_P2WSH), `address.Receive/Change` (`address/address.go:20-24,97-131`). Full-policy gate: the `wsh_sortedmulti` golden is template-only (`md/expand_test.go:100-102`); full-policy multisig surfaces `XpubPresent=true` (`:143-166`). Program enum + lockstep (`gui/gui.go:147-153,1491-1503,1636-1646,1663-1672,1846,1856,1865`) + 3 nav-tests. NET-NEW: `findUserSlot` (~40 lines) + the `engraveMultisigFlow` orchestrator + the program/lockstep.

## 4. Faithfulness / security spine
- **Seed typed-only, NEVER NFC** (the `gui/scan.go` footgun); ms1 engraved onto owner-held steel only, never NFC. Per-leg scrub (D11): gate `m.Valid()`, `defer wipeBytes` entropy, scrub the mnemonic after the last derive (the cross-match derives at each slot until matched — scrub after); `deriveAccountXpub` scrubs seed/master/intermediates internally.
- **`.Neuter()`** — no xprv ever serialized/engraved/NFC'd; restore-doc no secret.
- **D14 (the wrong-wallet guard):** the cross-match compares the CANONICAL chainCode‖pubkey (not base58); REFUSE on zero matches — never engrave a backup for a wallet the seed isn't a cosigner of.
- **Verbatim:** the supplied md1 is engraved as-is (the device never re-encodes a supplied descriptor — it lacks an md multisig encoder by design).
- **Faithful-or-refuse:** address-verify/restore only for the bip380-expressible subset; non-bip380 shapes engrave verbatim but are display-only (never mis-verified against a wrong address).

## 5. Acceptance gate (TDD)
1. **Cross-match (D14):** for a full-policy 2-of-3 `wsh(sortedmulti)` md1 where the user's seed is slot @1, `findUserSlot` returns index 1 + the slot's origin; a seed that is NOT a cosigner → zero match → refuse; an ambiguous (key reused at @0,@2) → first-by-index + notice; canonical-pair compare (a base58 with different parentFP still matches). Template-only / any-slot-missing-xpub → refuse.
2. **Derive the user's leg:** mk1 stub == `WalletPolicyIDStubChunks(suppliedMd1)` (bound, decoded-field assert not raw-string); mk1 Path == the matched slot origin; ms1 round-trips the entropy.
3. **Engrave:** full = 3 cards (ms1 + user mk1 + supplied md1 verbatim); watch-only = 2 + reminder; the supplied md1 strings are engraved UNMODIFIED.
4. **verify-bundle:** re-derive + `bundle.Verify` PASS for the user's slot; a mutated user mk1/md1/ms1 → FAIL; watch-only (ms1-less) verify works.
5. **restore doc:** a sortedmulti supplied md1 → `address.Receive/Change` match the expected multisig address; a non-bip380 miniscript md1 → display-only "addresses unavailable" (no wrong-address verify); greps clean of xprv.
6. **Program nav:** `engraveMultisig` reachable (between engraveSingleSig/qaProgram, non-blank title, no panic, all 3 nav-tests updated), `TestAllocs` green.
7. **Security + no-regression:** typed-only seed (D12 structural + behavioral); per-leg scrub on all exit paths incl. abort/no-match; T4/T5/T6a/single-card flows + codecs byte-unchanged; fuzz `findUserSlot` + the flow (0 panics).

## 6. Invariants (R0 must confirm)
- **I-1 (Critical, D14):** `findUserSlot` matches on the canonical `(chainCode,compressedPubkey)` (never base58), derives at EACH slot's own origin, REFUSES on zero matches; a non-cosigner seed can NEVER produce an engraved bundle.
- **I-2 (Critical):** the supplied md1 is engraved VERBATIM — never re-encoded; the device builds NO multisig md1.
- **I-3:** full-policy required — template-only / any-slot-missing-xpub → refuse.
- **I-4:** the user's mk1 stub = `WalletPolicyIDStubChunks(suppliedMd1)` (bound to the SUPPLIED policy); Path = the matched slot origin.
- **I-5:** verify-bundle verifies the USER's slot only (reuse `bundle.Verify` unchanged); other slots unverified-by-design.
- **I-6:** restore-doc address-verify ONLY for the bip380-expressible (sortedmulti) subset; non-bip380 → display-only (faithful-or-refuse).
- **I-7 (Critical):** seed typed-only (never scan→derive); ms1 engraved-only/never-NFC; per-leg scrub (incl. the multi-derive cross-match loop) on all exit paths.
- **I-8:** mainnet-only.
- **I-9:** new `engraveMultisig` program coherent across all 8 lockstep sites (no panic/blank title; qaProgram non-navigable; all 3 nav-tests updated); `TestAllocs` green.
- **I-10 (no-regression):** T4/T5/T6a/single-card flows, `bundleEngrave`/`bundle.Verify`, codecs byte-unchanged.

## 7. Biggest risks (lock in R0)
1. **D14 cross-match correctness** (I-1) — the wrong-wallet guard; canonical-pair compare + refuse-on-zero + the ambiguous rule; the single net-new logical piece.
2. **Full-policy gate** (I-3) — a template-only supplied md1 can't cross-match; refuse cleanly.
3. **Program-lockstep drift** (8 sites + 3 nav-tests) — the mechanical risk (T6a's exact dance).
4. **Per-leg scrub across the multi-derive cross-match loop** (I-7) — scrub on no-match/abort.
5. **No hardware** to validate the supply-gather + cross-match UX.

## 8. Gate
This spec MUST pass opus R0 to 0C/0I before code; fold → persist → re-dispatch until GREEN. Then implementation plan → its own R0 → single-implementer TDD → mandatory whole-diff adversarial exec review → merge no-ff (signed+DCO) → push bg002h. T6b completes T6 (T6c deferred → FOLLOWUPS); then T7 (niceties, optional).
