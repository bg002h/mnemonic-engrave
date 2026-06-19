# SPEC — T6a: single-sig flagship (seed → ms1+mk1+md1 → engrave + verify-bundle + restore doc)

**Status:** for opus R0 RE-REVIEW (R1) after folding R0 (4C/5I/6m). Must reach 0C/0I before code.
**Fork base:** `e4013a8` (T5 shipped). **Fork-side only; no upstream PR.**
**Feeds from:** `design/cycle-prep-recon-T6-flagship.md`, `design/agent-reports/seedhammer-T6-{recon-build-feasibility,recon-scope-security,architect-scope-multisig}.md`, and the R0 review `design/agent-reports/seedhammer-T6a-singlesig-spec-review-R0.md` (the 4C/5I/6m this folds). **USER decisions:** secret = ms1 + watch-only (option 1); **the user CHOOSES the wallet type from a pick list** (2026-06-19); multisig/miniscript supplied as md1 → T6b (not here). **Pinned Rust SHAs:** `md-codec` v0.36.0 @ `c85cd49`, `ms-codec` v0.4.4.

## 1. Why / context
T6 = the constellation flagship: from ONE hand-typed seed, derive + engrave the full **single-sig** constellation backup — `ms1` (the seed's BIP-39 entropy as codex32), `mk1` (account xpub), `md1` (the single-sig **wallet-policy** descriptor) — then verify-bundle (read back, parity) + watch-only restore doc. A NEW `program` (parallel to T4's `engraveXpub`). Single-sig is the only scope where one seed → complete ms1+mk1+md1; multisig/miniscript are SUPPLIED as md1 in T6b.

## 2. Plan-level split (R0 ruling e) — ONE spec, TWO gated implementation cycles
- **T6a-1 (headless):** `md.EncodeSingleSig` (4 script shapes, wallet-policy wire) + `codex32.EncodeMS1` (net-new) + the verify-bundle deterministic comparator. Carries its OWN focused plan-R0 on the wire format (the byte-lock risk) BEFORE any GUI.
- **T6a-2 (GUI):** the new `program` (8-site lockstep), typed seed entry, the **wallet-type pick list**, derive-all-3, watch-only mode, engrave reuse, verify-bundle flow, restore-doc screen.
(Mirrors #10a/#10b. This spec covers both; the two plans each get their own R0.)

## 3. Scope — IN

### Phase A (headless) — the two NET-NEW encoders + comparator
- **`md.EncodeSingleSig` — a WALLET-POLICY single-sig md1 (R0-C1/C2/C3).** Builds, inside package `md` (the only place the AST is constructible — `body.isBody()` is unexported `md/md.go:103`), a **wallet-policy** `*descriptor` (NOT template-only) and calls the shipped `split`/`encodeMD1String` (`md/chunk.go:121`, `md/encode.go:451`). Shape, matching the toolkit's `synthesize.rs:140-155` / `cell_7_wpkh_full` (`md-codec/tests/wallet_policy.rs:190-204`):
  - `n=1`; `pathDecl{n:1, paths: Shared(<explicit BIP origin>)}` — **explicit origin mandatory for ALL 4 types** (BIP-49 `sh(wpkh)` REQUIRES it — `canonical_origin` returns None, `validateExplicitOriginRequired` else rejects, `md/md.go:1030-1066`); `useSite = <0;1>/*` (hasMultipath, alts {0},{1}, wildcard unhardened);
  - `tlv`: **`pubPresent:true, pubkeys:[{idx:0, xpub:[65]byte}]`** (chainCode[32]‖compressedPubkey[33]) **AND `fpPresent:true, fingerprints:[{idx:0, fp:[4]byte}]`** (the toolkit always emits the fingerprint TLV — `synthesize.rs:153`);
  - **the 4 script tree shapes (R0-C2), distinct bodies:** `pkh → node{tagPkh, keyArgBody{0}}`; `wpkh → node{tagWpkh, keyArgBody{0}}`; `tr (key-path) → node{tagTr, trBody{isNums:false, keyIndex:0, tree:nil}}` (NOT keyArgBody); `sh-wpkh → node{tagSh, childrenBody{[node{tagWpkh, keyArgBody{0}}]}}`.
  - canonicalize is a no-op for n=1 (identity fast-path `md/canonicalize.go:53-63`) but `EncodeSingleSig` STILL routes through `encodePayload`→`canonicalize` (do not bypass). The encoder does NOT validate the pubkey on-curve (only decode does) → the round-trip leg (§6.A3) is the safety net.
  - **Concrete signature (R0-I2):** `md.EncodeSingleSig(chainCode [32]byte, compressedPubkey [33]byte, fp [4]byte, origin []PathComponent, script ScriptKind) ([]string, error)` (takes parsed components — the GUI caller does the base58-xpub→bytes + fp uint32→[4]byte conversion; keeps private material out). `PathComponent{Hardened bool, Value uint32}` — NOT the in-band-hardening `bip32.Path` convention (R0-M5: don't conflate the encoder's raw component with the expand/display `+HardenedKeyStart` form).
- **`codex32.EncodeMS1` — NET-NEW (R0-C4; the fork has only `DecodeMS1`).** `EncodeMS1(entropy []byte) (string, error)` = `codex32.NewSeed("ms", 0, "entr", 's', payload)` where `payload = append([]byte{0x00}, entropy...)` (the `0x00` entr prefix byte per `codex32/mspayload.go:5-12`; `id` is the FIXED literal `"entr"`, NOT fingerprint-derived; share index lowercase `'s'`). **Language lock:** entr/English-only this cycle (the `0x00` entr payload carries NO language byte; the fork ships only the English wordlist on-device, `ms1_decode.go:31-44`) → DROP "wordlist language" from the verify-bundle parity set and document the English-only restore caveat (R0-C4; the `0x02` mnem-prefix language-carrying variant is a follow-on). `DecodeMS1(EncodeMS1(entropy)) == entropy` is the ms1 acceptance leg.
- **verify-bundle comparator (deterministic).** Re-derive from the re-typed seed; compare: master fingerprint, account xpub, origin path, **md1 string exact-match** (deterministic encoder), **ms1 recovered-ENTROPY bytes** (hand-typed → re-derived; compare entropy, not string, unless id+prefix+share fully pinned — they are, so string-match also valid). PASS/FAIL + the diverging field. (Wordlist-language dropped per the entr lock.)

### Phase B (GUI)
- A new top-level `program` inserted BEFORE `qaProgram` (R0-M4; 8 lockstep sites: enum `gui/gui.go:147-151`, dispatch `:1491-1497`, wrap bound `:1634-1641`, title+layout arms `:1662-1664`, `npage`/`npages` derived consts `:1840,1859`, nav-test `gui/derive_xpub_program_test.go`) — BOTH the title arm (no blank fail-open) AND the layout arm present.
- **Typed seed entry ONLY (R0-I4/D12, Critical):** the T6a program calls `seedEntryFlow` (as `deriveXpubFlow` does, `gui/derive_xpub.go:106-107`); it contains NO `act.scan`→derive path. (`assembleScan` CAN parse a bip39 mnemonic + codex32 secret from NFC — `gui/scan.go:60-69` — the footgun; T6a must never route a scanned object into derivation.)
- **Wallet-type PICK LIST (USER requirement):** the user chooses the wallet type from a list of the **4 single-sig types** — BIP-44 `pkh`, BIP-49 `sh(wpkh)`, BIP-84 `wpkh`, BIP-86 `tr` (key-path). **Mainnet-only (R0-I5):** drop the "× network" axis — the md1/restore-doc stack is mainnet-locked (`gui/md1_expand.go:61`, matching #10b); testnet is a follow-on.
- Derive the 3 legs (ms1 via `EncodeMS1`; mk1 via T4 `deriveAccountXpub`+`mk.Encode`; md1 via `EncodeSingleSig`), the GUI caller converting the base58 xpub → (chainCode, compressedPubkey) + fp uint32→[4]byte for `EncodeSingleSig`.
- **Watch-only / skip-ms1 mode** (user option-1): full → engrave ms1+mk1+md1; watch-only → mk1+md1 only.
- **Engrave:** synthesize `[]bundleCard` from the derived strings + reuse T5 `bundleEngrave` (`gui/bundle_flow.go:327`, `[]bundleCard`-driven, verbatim, "Card X of Y · Plate P of Q", set-abort). Completion message variants (R0-M1): full → "ms1 engraved"; watch-only → DO show the ms1 hand-engrave reminder.
- **verify-bundle flow:** RE-TYPE the seed (D8, shorter residency) → read back mk1/md1 over NFC (PUBLIC) + ms1 HAND-TYPED (SECRET, never NFC; reuse T2a codex32 entry) → run the comparator → PASS/FAIL. Offered inline AND re-enterable standalone (D7).
- **restore doc (R0-M2):** display-only + optional NFC; master fp + the concrete descriptor + first receive/change address (from-xpub `*bip380.Descriptor`, `gui/md1_expand.go:60-77` + `address.Receive/Change`); greps clean of any xprv/private material.

### OUT (deferred)
- Multisig / complex miniscript → **T6b** (supply a complete md1 over NFC + slot-cross-match + verbatim engrave; no new encoder). The on-device policy picker + `md.EncodeMultisig` → **T6c (out of scope; FOLLOWUPS)**. Testnet → follow-on. mnem-prefix (non-English ms1) → follow-on. Self-multisig (constellation hard-rejected).

## 4. Faithfulness / security spine (most sensitive tier)
- **Seed/mnemonic/passphrase SECRET → typed-only, NEVER NFC (R0-I4/D12, Critical):** T6a uses `seedEntryFlow`; no `act.scan`→derive. ms1 engraved onto owner-held steel only, NEVER NFC.
- **`.Neuter()` everything before serialization** — no xprv ever serialized/displayed/engraved/NFC'd; restore doc greps clean.
- **Per-leg scrub schedule (R0-I3, Critical — restated precisely):** (a) gate `m.Entropy()` on mnemonic validity FIRST (it panics on invalid, `bip39/bip39.go:158`); `defer wipeBytes` the entropy buffer immediately after `EncodeMS1`; (b) seed/master/every-intermediate `*ExtendedKey` are scrubbed INSIDE `deriveAccountXpub` (reuse, no new handling — `gui/derive.go:20-52`, fp captured before zeroing, serialize-before-Zero ordering preserved); (c) the mnemonic `[]Word` is the spanning secret — zero it after its LAST consumer = the LAST of {mk1 xpub derive, entropy extraction} (md1 build needs only the PUBLIC xpub, NOT the mnemonic; restore-doc is fully public — the seed/mnemonic NEVER reaches it); (d) verify-bundle RE-TYPES the seed (fresh residency window, own scrub). Accept the un-wipeable ms1 `strings.Builder` `.String()` residual until GC (consistent with the shipped ms1-display posture, `ms1_decode.go`).
- **mk1/md1/xpub/addresses PUBLIC**; restore doc NO secret. Passphrase never engraved/transmitted (no-pp vs pp fingerprint follows `backupWalletFlow`).
- **Set-level all-or-nothing** (reuse T5 abort): a partial bundle is NOT usable → incomplete warning; re-entry re-derives deterministically (no half-state).

## 5. Verified facts (cite source)
- Wallet-policy is `pubkeys Some(non-empty)` (`md-codec/encode.rs:50-52`); the vendored `wpkh_basic`/etc. goldens are template-only (`pubkeys:null`) → cannot serve as the gate (R0-C1). Toolkit single-sig shape: `synthesize.rs:140-155` / `cell_7_wpkh_full` `wallet_policy.rs:190-204`; in-package ref `singlesigWithPubkey` `md/validate_test.go:29-40`.
- 4 tree shapes + bodies: `md/md.go:114-118` (trBody), `:346-351` (sh child), `:432-457` (tr decode), `md/encode.go:160-229` (writeNode dispatch); BIP-49 explicit-origin requirement `canonical_origin.rs:63-77` + `md/md.go:1030-1066`.
- ms1: `codex32/mspayload.go:5-12` (prefix), `:6-7` ("entr" id), `mspayload_test.go:54` (`NewSeed("ms",0,"entr",'s',[prefix‖entropy])`), `DecodeMS1` `:34-60`; no `EncodeMS1` exists (net-new).
- Scrub: `gui/derive.go:20-52`, `gui/derive_xpub.go:106-115`, `bip39/bip39.go:158`. Footgun: `gui/scan.go:60-69`.
- Engrave/restore reuse: `gui/bundle_flow.go:327`, `gui/bundle.go:29-37`, `gui/md1_expand.go:60-77`, `address/address.go:20-24`.

## 6. Acceptance gate (TDD)
### Phase A
- **A1 (PRIMARY, R0-C1/I1) — `EncodeSingleSig` wallet-policy byte parity.** Since NO key-bearing golden ships, use a **Rust↔Go cross-encoder differential**: build the identical wallet-policy AST in both (concrete `make_xpub`-style xpub bytes + fp + explicit origin) and assert `encode_md1_string` byte-equality, for **wpkh / pkh / tr / sh-wpkh × {with-fp}** (the toolkit always emits fp). (Optionally vendor NEW key-bearing goldens generated by the toolkit at the pinned SHA.) Each chunk `ValidMD`.
- **A2 — round-trip safety net:** `DecodeChunks`/`Decode`→`ExpandWalletPolicy` of the `EncodeSingleSig` output recovers the embedded xpub/fp/origin/script (the on-curve check lives in decode).
- **A3 — ms1:** `DecodeMS1(EncodeMS1(entropy)) == entropy` (English/entr); the wire begins `ms1...entrs...`; `id=="entr"`, prefix `0x00`.
- **A4 — verify comparator:** deterministic re-derive vs a correct set → PASS; a mutated xpub/descriptor/entropy → FAIL naming the field.
### Phase B
- **B1 — derive parity:** abandon-test seed at m/84'/0'/0' → mk1 == T4's known card; md1 (wpkh wallet-policy over that xpub) round-trips; ms1 decodes to the original entropy.
- **B2 — pick list:** the wallet-type list offers exactly the 4 single-sig types; selecting each drives the correct `EncodeSingleSig` script shape; mainnet-only (no network axis).
- **B3 — watch-only:** full → 3 cards; watch-only → 2 (mk1+md1), ms1 reminder shown; full → "ms1 engraved" message.
- **B4 — verify-bundle flow:** correct read-back → PASS; mutated → FAIL; ms1 hand-typed (never NFC), mk1/md1 NFC.
- **B5 — restore doc:** fp + descriptor + first recv/change addr match; greps clean of xprv.
- **B6 — typed-only (R0-I4):** a structural test that the T6a flow references `seedEntryFlow` and NOT `scan`/`assembleScan` for the secret + a behavioral test that a scanned `bip39.Mnemonic` cannot reach the derive entrypoint.
- **B7 — security/no-regression:** buffers scrubbed on ALL exit paths (incl. abort/error); fuzz `EncodeSingleSig`+`EncodeMS1`+comparator (0 panics); new program nav (before `qaProgram`, both arms, no panic, nav-test updated); `TestAllocs` re-run green; single-card/T4/T5 flows + codecs byte-unchanged.

## 7. Invariants (R0 must confirm)
- **I-1 (Critical):** `md.EncodeSingleSig` emits a WALLET-POLICY md1 (pubkeys + fingerprints TLV + explicit `path_decl.shared` origin) byte-identical to the toolkit, for all 4 script shapes (distinct bodies per R0-C2); proven by the A1 differential before GUI.
- **I-2 (Critical):** ms1 = `EncodeMS1` = `NewSeed("ms",0,"entr",'s',[0x00‖entropy])`; `DecodeMS1` round-trips; English/entr-only (wordlist-language dropped from parity, documented).
- **I-3 (Critical, D12):** seed/mnemonic typed-only; T6a NEVER consumes a scanned bip39/codex32; ms1 engraved-only/never-NFC.
- **I-4 (Critical, D11):** complete per-leg scrub — entropy (gate validity first) after `EncodeMS1`; seed/master/intermediates inside `deriveAccountXpub`; mnemonic after its last derivation consumer; restore-doc public; verify re-types. fp captured before zeroing.
- **I-5:** `.Neuter()` — no private material serialized/displayed/engraved/NFC'd.
- **I-6:** verify-bundle deterministic (fp + xpub + path + md1-exact-string + ms1-entropy → PASS/FAIL); mk1/md1 NFC, ms1 hand-typed.
- **I-7:** restore doc display-only (+optional NFC), no secret.
- **I-8:** the pick list offers ONLY the 4 single-sig types; mainnet-only.
- **I-9:** new `program` coherent across all 8 lockstep sites (before `qaProgram`, both arms, no panic, nav-test updated); `TestAllocs` intact.
- **I-10 (no-regression):** single-card flows, `deriveXpubFlow`, `backupWalletFlow`, T5 `bundleFlow`/`bundleEngrave`, codecs byte-unchanged.

## 8. Biggest risks (lock in R0)
1. **`md.EncodeSingleSig` wallet-policy wire fidelity** (dominant) — new public API on the Rust-golden-locked md pkg; the A1 Rust↔Go differential over all 4 shapes (with fp + explicit origin) is the gate; external-protocol-fact rule.
2. **ms1 recipe** (R0-C4) — prefix `0x00`, id `"entr"`, share `'s'`, `EncodeMS1` net-new, English/entr-only.
3. **Per-leg scrub (D11)** + **typed-only seed (D12)** — the two Critical security invariants; longest secret residency; the `scan.go` footgun.
4. **Program lockstep** (before `qaProgram`, both arms, derived consts).
5. No hardware to validate the multi-leg UX (residual, not a blocker).

## 9. Gate reminder
This spec MUST pass opus R0 (R1) to 0C/0I before code; fold → persist verbatim → re-dispatch until GREEN. Then the T6a-1 (headless) plan → its focused R0 (the wire-format gate) → single-implementer TDD → exec review → merge; then T6a-2 (GUI) plan → R0 → impl → exec review → merge. The `EncodeSingleSig` differential + the ms1 recipe + the typed-only/scrub spine are the must-verify items. Then T6b (multisig/miniscript via supplied md1).
