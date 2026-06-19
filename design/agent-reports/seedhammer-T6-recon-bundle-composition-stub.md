# RECON (T6, agent a2269681b4b870e34, 2026-06-19) — bundle composition + mk1↔md1 stub binding (source-verified)

Verified vs `mnemonic-toolkit@f7e6fca`, md-codec, mk-codec, Go fork `seedhammer@e4013a8`. External-protocol claims cross-checked vs SPEC text (surfaced a stale doc comment).

## Q1 — Bundle md1 is ALWAYS a FULL WALLET POLICY (keys embedded). No template-only mode exists.
Single-sig synth: `pubkeys: Some(vec![(0, xpub_65)])` + `fingerprints: Some(...)` then `debug_assert!(descriptor.is_wallet_policy())` (`synthesize.rs:154,195,231,346`). `is_wallet_policy()` = `pubkeys Some(non-empty)` (`md-codec/encode.rs:50-52`). Keyless descriptor REFUSED: `cli_bundle_keyless_descriptor.rs:18-35` (exit 2 "no keys to engrave"→export-wallet). Watch-only bundles STILL emit full-policy md1 (`cli_bundle_watch_only.rs:49-50`). `cell_7_wpkh_full` IS the real bundle shape. **No template-only md1 anywhere in the toolkit; no flag for it.**

## Q2 — mk1 stub = top-4 of `WalletPolicyId` (NOT Md1EncodingId); key-dependent; encoding-stable.
Toolkit sets stub = `compute_wallet_policy_id(descriptor).as_bytes()[..4]` at every synth site (`synthesize.rs:179-181,215-217,272-274,453-455,625`); KeyCard `policy_id_stubs Vec<[u8;4]>` (`mk-codec/key_card.rs:30`); pinned `assert_eq!(decoded_mk1.policy_id_stubs[0], policy_id.as_bytes()[..4])` (`synthesize.rs:1089,1134,1558`).
**THREE distinct 16-byte ids — do not conflate:**
- `Md1EncodingId` = SHA-256(encode_payload(d))[0:16] (`identity.rs:39-45`) → the md1 **chunk_set_id** (`chunk.rs:243`). = Go fork `computeEncodingID` (`md/identity.go:11-16`).
- `WalletDescriptorTemplateId` = template-tree-only, key+origin-invariant (`identity.rs:71-104`).
- `WalletPolicyId` = canonical-expanded policy hash (`identity.rs:172-240`): placeholder-tree ‖ per-@N records (`presence_byte = fp_present | xpub_present<<1` + fp[4] + xpub[65] when present). **THE STUB SOURCE.**
WalletPolicyId IS key-presence-significant: nulling pubkeys+fp changes it (`identity.rs:610-617` `walletpolicyid_template_only_differs_from_full_cell_7`). Stable across origin/use-site elision (`:572-605`) — why it beat the bytecode hash.
**STALE DOC FLAGGED:** `mk-codec/key_card.rs:27` still says "top 4 bytes of SHA-256(canonical_bytecode)" — OLD formula. Authoritative = `mnemonic-key/design/SPEC_mk_v0_1.md:186,312,385` (audit I1): stub = `compute_wallet_policy_id(descriptor)[..4]`, explicitly NOT the bytecode/Md1EncodingId. The standalone `mk-cli derive_stub_from_md1` still uses the old formula (LOW-severity divergence, manual path only — `design/PLAN_stub_formula_walletpolicyid.md`).

## Q3 — The constellation does NOT recompose a full policy from template+seed. It BINDS a full-policy md1 to keys.
(Corrects the directive's framing.) Single-sig `restore` consumes NO md1 — derives the xpub from the seed + `--template` (`cmd/restore.rs:177-199,340`). Multisig `restore` REQUIRES a full-policy md1 (`:1232-1238` "template-only ... multisig restore needs a wallet-policy md1"), reads concrete xpubs from its TLV (`expand_per_at_n`), uses mk1 only as a cross-check. Stub binding checked in self-check/verify: `bundle.rs:2157-2192` recomputes `compute_wallet_policy_id(md1)[..4]` and asserts `mk1.policy_id_stubs` contains it (`:2187`). verify re-derives parent xpub from seed, matches md1 keys (`verify_bundle.rs:442,501,2741`). **Mirror = full-policy md1 + stub-bound mk1; NO template gets "completed."**

## Q4 — Template-only md1 with origin is feasible (~1 chunk) but unused + carries a stub pitfall.
`path_decl` (origin) is a top-level field decoupled from `tlv.pubkeys` (`encode.rs:16-28,85`) → a template (`pubkeys:None`) CAN carry a non-empty origin. Chunk cap 320 bits (`chunk.rs:219`); bare wpkh template ≈ 1 chunk vs full policy ≈ 2-3 (65B xpub+4B fp TLV). **PITFALL:** WalletPolicyId is key-presence-significant → template id ≠ full-policy id; a template md1 + a full-policy-computed stub would NOT match → binding breaks.

## Q5 — T6a-1 scope delta
1. **`EncodeSingleSig` does NOT need a template mode** to mirror the constellation (bundle md1 is always full-policy; template-only breaks the stub binding). A template mode would need a SEPARATE template-stub derivation + would produce incompatible stubs — recommend NOT adding unless the owner explicitly wants a keys-excluded card.
2. **T6's mk1 stub MUST be NON-ZERO** = `WalletPolicyId(md1)[0:4]`, DIFFERING from T4's stub-0 + "Unbound Key Card" warning (`gui/derive_xpub.go:142,156-159,233-256`). T6 passes `Stubs: [][4]byte{WalletPolicyId(md1)[0:4]}` to `mk.Encode`, dropping the stub-0 warning for the bound case.
3. **SIGNIFICANT GO GAP:** the fork has ONLY `computeEncodingID` (= Md1EncodingId, the chunk-id) — NO `WalletPolicyId` (grep: zero hits). **T6 must PORT `compute_wallet_policy_id` to Go** (canonical-expanded preimage: placeholder-tree ‖ per-@N presence_byte+fp+xpub records, SHA-256[..16]), byte-exact vs `md-codec/identity.rs:172-240`, or the engraved mk1 won't verify against a toolkit-recomposed bundle. This is the main hidden T6 cost.

## Decisions for the owner
- **Full-policy vs template:** constellation has ONE answer (full policy); template-only is feasible but unused + breaks the key-dependent stub binding + needs a 2nd encoder mode. Diverging from "mirror the constellation" is the cost of offering it.
- **WalletPolicyId port:** accept that T6 needs it ported to Go (byte-exact vs identity.rs:172-240); computeEncodingID can't be reused for the stub.
- **T4 contrast:** confirm T6 moves from stub-0/unbound (T4) to a policy-bound stub, removing the "Unbound Key Card" warning for the bound flow.

## Mirror constraints (load-bearing)
mk1 stub = `compute_wallet_policy_id(full_policy)[0:4]` (SPEC_mk §3.3 post audit-I1), NOT bytecode/encoding-id; key_card.rs:27 doc is stale. md1 chunk_set_id uses Md1EncodingId (different value). md1 must `is_wallet_policy()` or multisig restore/self-check rejects it.
Key files: toolkit `synthesize.rs`/`cmd/{bundle,restore,verify_bundle}.rs`; md-codec `identity.rs`/`encode.rs`/`chunk.rs`; mk-codec `key_card.rs` + `SPEC_mk_v0_1.md` + `PLAN_stub_formula_walletpolicyid.md`; Go `md/identity.go`,`mk/mk.go`,`gui/derive_xpub.go`.
