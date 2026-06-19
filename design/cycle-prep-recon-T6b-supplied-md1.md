# cycle-prep recon — 2026-06-19 — T6b multisig/miniscript bundle via SUPPLIED md1

**Fork HEAD:** `072461a` (T6a complete). **Recon agent (source-verified):** `a2dd9ca158d256aeb` (verbatim below). Grounding: `design/agent-reports/seedhammer-T6-architect-scope-multisig.md` (mechanism ii) + `…-recon-bundle-composition-stub.md` (stub binding) + `…-T6a2-gui-recon.md`.

## HEADLINE
T6b is feasible with **ZERO new encoder** (md1 engraved VERBATIM, architect mechanism ii). T6a already shipped nearly every primitive; the multisig restore + address derivation are ALREADY shipped (`expandedToDescriptor` + `address` handle sortedmulti). The ONE genuinely net-new logical piece is the **D14 slot cross-match** (~40 lines, built from shipped primitives). Biggest *mechanical* risk = the program-enum lockstep (8 sites + 3 nav-tests).

## Flow
User SUPPLIES a complete multisig/miniscript wallet-policy md1 over NFC (PUBLIC) → device decodes it → derives the user's OWN key from the TYPED seed → CROSS-MATCHES it to a descriptor @N slot → engraves the supplied md1 VERBATIM + the user's mk1 (policy-bound stub) + ms1 (full) / skip (watch-only) → verify (user's slot only) → restore-doc (bip380-expressible subset).

## Source-verified surface (@ 072461a)
- **Gather (REUSE):** `bundleGatherFlow(ctx,th)([]bundleCard,bool)` `gui/bundle_flow.go:95` (refuses ms1 `:55` + single mk1 `:57`); `bundleCard{kind,label,strings,summary}` `gui/bundle.go:33-38`, `bundleCardKind{cardMK1=0,cardMD1=1,cardMS1=2}` `:24-28`. Extract the single cardMD1 (clone `singleSigReadbackCards` `gui/singlesig_verify.go:23-42`).
- **Decode + per-@N (REUSE):** `md.ExpandWalletPolicyChunks(strs)(Template,[]ExpandedKey,error)` `md/expand.go:102`. `ExpandedKey{Index uint8; OriginPath bip32.Path (in-band hardening); UseSite; Fingerprint [4]byte+Present; Xpub [65]byte (chainCode[0:32]‖compressedPubkey[32:65])+XpubPresent}` `md/expand.go:56-64`. Divergent per-cosigner origins supported (`md/expand_test.go:113-141`).
- **Derive user key (REUSE):** `deriveAccountXpub(m,passphrase,net,path)(xpub,masterFP,err)` `gui/derive.go:19` — accepts ARBITRARY `bip32.Path` (`:33`), scrubs internally.
- **xpub→canonical pair (REUSE):** `decodeXpubBytes(xpub)(chainCode [32]byte, compressedPubkey [33]byte, parentFP uint32, err)` `gui/singlesig_derive.go:99` (T6a glue; refuses private).
- **User mk1 + stub (REUSE):** `mk.Encode(mk.Card{Network,Path,Fingerprint,Stubs [][4]byte,Xpub})` `mk/encode.go:38`; stub = `md.WalletPolicyIDStubChunks(suppliedMd1Strings)` `md/walletpolicyid.go:126` (bound to the SUPPLIED policy). ms1 = `codex32.EncodeMS1(m.Entropy())` `codex32/msencode.go:17` (gate `m.Valid()` first — `Entropy()` panics on invalid `bip39/bip39.go:158-161`).
- **Engrave (REUSE):** `bundleEngrave(ctx,th,cards)` `gui/bundle_flow.go:327` (verbatim per-plate); cards like `singleSigEngraveCards` `gui/singlesig_engrave.go:20` (full=[ms1,mk1,md1]; watch-only=[mk1,md1]+reminder via `bundleShowMs1Reminder` `bundle_flow.go:373`).
- **Verify (REUSE, no change):** `bundle.Verify(derived,readback)` `bundle/verify.go:32` — stub-binding both sides (`:103-118`), mk1 fp/xpub/path (`:52-60`), md1 EXACT string (`:64-66`), ms1 entropy (`:81-97`, watch-only both-empty skip `:74-79`). Verifies the USER's slot only; other slots public-given/unverified-by-design (D15) — `bundle.Verify` never inspects them.
- **Restore-doc (REUSE — multisig ALREADY handled):** `expandedToDescriptor(tpl,keys)(*bip380.Descriptor,expandStatus)` `gui/md1_expand.go:32` projects PolicySortedMulti→P2WSH/P2SH/P2SH_P2WSH (InnerWsh discriminant `:98-110`); `address.Receive/Change` handle SortedMulti (`address/address.go:97-131`). Non-bip380 (unsorted multi/multi_a/sortedmulti_a/taptree/complex) → `expandUnsupported`/`!Renderable` → display-only.

## THE CROSS-MATCH (D14 Critical — wrong match = backup for a wallet you're not in) — NET-NEW
`findUserSlot(m, passphrase, net, keys []md.ExpandedKey) (slotIndex int, originPath bip32.Path, ok bool)`: for each slot k — skip if `!k.XpubPresent`; `(xpub,_) := deriveAccountXpub(m,pp,net,k.OriginPath)`; `(cc,pk,_) := decodeXpubBytes(xpub)`; match iff `cc==k.Xpub[0:32] && pk==k.Xpub[32:65]` (canonical pair — NEVER base58; the supplied xpub carries different parentFP/depth metadata). Outcomes: exactly-one→record (index, origin); zero→REFUSE; ambiguous (≥2, reused key)→deterministic rule. Reuses `deriveAccountXpub`+`decodeXpubBytes` wholesale; ~40 lines.

## Program + UX (NET-NEW lockstep — high mechanical, low logical risk)
Enum `gui/gui.go:147-153`: `backupWallet=0,engraveXpub=1,engraveBundle=2,engraveSingleSig=3,qaProgram=4` (qaProgram non-navigable). T6b = OWN program `engraveMultisig` BETWEEN `engraveSingleSig` and `qaProgram`. Lockstep sites: enum; dispatch `:1491-1503`; left-wrap `:1636-1639`; right-wrap `:1644-1646`; title `:1663-1672`; npage `:1846`; layoutMainPlates `:1856`; npages `:1865`; + 3 nav-tests (`singlesig_program_test.go`, `bundle_program_test.go`, `derive_xpub_program_test.go`) hardcode `engraveSingleSig` as the wrap bound → all move. Typed-only seed (`seedEntryFlow`, never scan); per-leg scrub (`gui/singlesig.go:41-45`).

## DECISIONS (locked per recon recommendations + "proceed autonomously"; R0 may challenge)
- **D-prog (own program):** new top-level `engraveMultisig` (NOT extend engraveSingleSig — the gather-supplied-md1 + cross-match flow shape differs). Recon-recommended.
- **D-supply (full-policy required):** the supplied md1 MUST be a FULL wallet policy — every slot `XpubPresent`. Template-only / any-slot-missing-xpub → REFUSE ("the supplied descriptor has no public keys to match"). (The repo's `wsh_sortedmulti` golden is template-only — `md/expand_test.go:100-102` — so this gate is load-bearing; full-policy multisig md1 surfaces XpubPresent=true per `:143-166`.)
- **D-match (AUTO, no user pick):** derive at EACH slot's own origin, exactly-one canonical-pair match → the user's slot; zero → refuse; the seed determines the slot (safer than a user pick — D14). Ambiguous (≥2 match) → engrave the FIRST-by-index slot's mk1 (deterministic; the policy + stub are identical across slots, only the mk1 Path differs) + show a notice that the key is reused at slots @i,@j. (R0 may prefer a confirm.)
- **D-net (mainnet-only):** matches T6a/#10b; the cross-match is network-agnostic (chainCode+pubkey), derive on mainnet. Testnet = follow-on.
- **D-watchonly:** support watch-only (skip ms1) — reuse the T6a full/watch-only branch + `bundle.Verify` both-empty-ms1 skip.
- **D-verbatim/restore:** engrave the md1 VERBATIM always (cross-match works for ANY xpub-bearing shape); address-verify/restore-doc only for the `expandOK` sortedmulti subset; non-bip380 → display-only.
- **D12/D11 (spine):** typed-only seed (never scan→derive); per-leg scrub; ms1 engraved-only/never-NFC.

## Effort / phasing
~1 cycle. Net-new = `findUserSlot` (cross-match) + the `engraveMultisigFlow` orchestrator + the new program/lockstep + nav-tests. Everything else reuse/clone of the T6a + T5 machinery. NO new encoder, NO new codec.

## Gate
`SPEC_seedhammer_T6b_*` MUST pass opus R0 to 0C/0I. The D14 cross-match (canonical-pair compare, refuse-on-zero, the ambiguous rule) + the full-policy-required gate + the typed-only spine are the must-verify items. Fork-side only; no upstream PR. Then T7 (niceties, optional).
