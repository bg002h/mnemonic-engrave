# R0 GATE RE-REVIEW (R1) — SPEC_seedhammer_T6a_singlesig_flagship.md (opus architect)

**Date:** 2026-06-19
**Round:** R1 (re-review after folding R0's 4C/5I/6m).
**Reviewer role:** mandatory pre-implementation R0 gate (most security-sensitive tier; derives from the live seed; introduces NEW public API on a Rust-golden-byte-locked package).
**SPEC under review:** `design/SPEC_seedhammer_T6a_singlesig_flagship.md` (folded)
**Prior review:** `design/agent-reports/seedhammer-T6a-singlesig-spec-review-R0.md` (NOT GREEN, 4C/5I/6m)
**Sources verified @ (re-confirmed this round):**
- fork `e4013a8` (`git log -1` → `e4013a88011284c71f6da1b5629555bdc52c7e88 "Merge T5: guided bundle sequencing"`), tree `/scratch/code/shibboleth/seedhammer`
- Rust md-codec v0.36.0 @ `c85cd49` (`git log -1` → `c85cd498c690d9f91c7884234cf25d0c39264608`), tree `/scratch/code/shibboleth/descriptor-mnemonic`
- Rust ms-codec v0.4.4 `/scratch/code/shibboleth/mnemonic-secret`
- mnemonic-toolkit `/scratch/code/shibboleth/mnemonic-toolkit`

Method: every wire/protocol fact below was re-verified against source text (parallel fan-out + direct first-hand reads), not against the spec's prose. file:line citations are from the pinned SHAs.

---

## VERDICT: GREEN

**0 Critical, 0 Important.** All four R0 Criticals (C1–C4) and all five R0 Importants (I1–I5) are CLOSED against source. The fold introduced no drift: the spec's stated AST, the four distinct tree bodies, the explicit-origin + fp-TLV + pubkey-TLV lock, the ms1 recipe, the rewritten §6.A1 differential gate, the concrete `EncodeSingleSig` signature, the restated scrub schedule, the typed-only test wiring, and the mainnet-only lock all match authoritative source. The plan-level split (Ruling e: T6a-1 headless / T6a-2 GUI) is adopted (§2). The wallet-type pick list (4 single-sig types, mainnet-only) is in Phase B (§3 Phase B, B2, I-8). The 10 invariants + the acceptance gate (A1–A4, B1–B7) cover every folded finding.

**The spec is cleared for the T6a-1 (headless) implementation plan**, which carries its own focused plan-R0 on the wire format (the byte-lock risk) per §2/§9.

Three non-blocking precision notes (M-level, fold opportunistically into the T6a-1 plan, NOT gate-blocking) are recorded at the end.

---

## CRITICAL FINDINGS — status

### C1 — wallet-policy vs template-only — **CLOSED**

Spec now (§3 Phase A line 18-20, §5 line 48, §7 I-1, §6.A1 line 56) specifies `EncodeSingleSig` to build a WALLET-POLICY `*descriptor` with `pubPresent:true, pubkeys:[{idx:0, xpub:[65]byte}]` AND `fpPresent:true, fingerprints:[{idx:0, fp:[4]byte}]` AND explicit `pathDecl{n:1, paths: Shared(<BIP origin>)}`, and the §6.A1 gate is now a Rust↔Go differential over WALLET-POLICY descriptors, not the template-only goldens.

Evidence:
- `is_wallet_policy()` = `matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty())` — wallet-policy iff pubkeys Some(non-empty). `descriptor-mnemonic/crates/md-codec/src/encode.rs:50-52` (read first-hand). The spec's §5 line 48 cite is exact.
- The on-device/vendored goldens are ALL template-only: `wpkh_basic.descriptor.json`, `pkh_basic.descriptor.json`, `tr_keyonly.descriptor.json` each have `"pubkeys": null` (and `path_decl` Shared `"m"`). NO key-bearing golden (`"pubkeys": [...]`) ships in EITHER the fork `md/testdata/vectors/` or `descriptor-mnemonic/crates/md-codec/tests/vectors/` (grep `'"pubkeys": *\['` → NONE in both trees). So the template-only goldens cannot serve as the A1 gate — confirmed.
- Authoritative wallet-policy shape = toolkit `synthesize.rs:143-159` (read first-hand): `path_decl.paths: PathDeclPaths::Shared(origin_path)`, `fingerprints: Some(vec![(0, fp_bytes)])`, `pubkeys: Some(vec![(0, xpub_65)])`. Matches the integration fixture `cell_7_wpkh_full()` `descriptor-mnemonic/crates/md-codec/tests/wallet_policy.rs:190-204`: `Shared(bip84_path())`, `fingerprints = Some([(0,[0xDE,0xAD,0xBE,0xEF])])`, `pubkeys = Some([(0, make_xpub(0x11))])`. The key-bearing fixture exists in Rust tests → a Rust↔Go differential is buildable even though no JSON golden ships.
- xpub TLV = `idxPub{idx uint8; xpub [65]byte}` (= chainCode[32]‖compressedPubkey[33]); fp TLV = `idxFP{idx uint8; fp [4]byte}`. `seedhammer/md/md.go:505-512`.
- The spec correctly cites `singlesigWithPubkey` (`md/validate_test.go:29-40`) only as the "minimal in-package ref" — that helper sets `pubPresent:true` but NOT `fpPresent`, and uses empty `shared:&originPath{}`. The spec does NOT lock the byte target to that minimal form; it locks to the TOOLKIT form (explicit origin + fp TLV + pubkey TLV). This is the correct decision for cross-tool parity and verify-bundle exact-match.

### C2 — the 4 distinct tree bodies — **CLOSED**

Spec now (§3 Phase A line 21, §5 line 49, §7 I-1) specifies four distinct bodies and mandatory explicit origin.

Evidence (all read first-hand):
- `pkh → keyArgBody{0}`, `wpkh → keyArgBody{0}`: decode dispatch `case tagPkK,tagPkH,tagWpkh,tagPkh: b = keyArgBody{index: uint8(idx)}` (`md/md.go:340-345`); `writeNode` `keyArgBody` arm `md/encode.go:158-167`. `keyArgBody struct{ index uint8 }` `md/md.go:119`.
- `tr (key-path) → trBody{isNums:false, keyIndex:0, tree:nil}` — a DISTINCT type from keyArgBody. `trBody struct{ isNums bool; keyIndex uint8; tree *node }` `md/md.go:114-118`; decode arm reads is_nums → key_index → has_tree `md/md.go:432-457`. Rust `tr_keypath_at_0()` = `Body::Tr{is_nums:false, key_index:0, tree:None}` `wallet_policy.rs:152-161`. The spec's "NOT keyArgBody" call-out is exact.
- `sh-wpkh → sh{childrenBody{[wpkh{keyArgBody{0}}]}}`: `case tagSh,tagWsh,...: b = childrenBody{children: []node{child}}` `md/md.go:346-351`; root allow-list permits tagSh `case tagSh,tagWsh,tagWpkh,tagPkh,tagTr:` `md/md.go:848-852`. Rust fixture `wallet_policy.rs:786-792`: `Node{tag:Sh, body:Children(vec![Node{tag:Wpkh, body:KeyArg{index:0}}])}`.
- Explicit origin for `sh(wpkh)` is MANDATORY: `canonical_origin()` for the `(Tag::Sh, Body::Children)` arm checks inner tag == Wsh; `sh(wpkh)`'s inner tag is Wpkh → falls through to `None` (`canonical_origin.rs:63-77`, read first-hand). `validateExplicitOriginRequired` rejects when no canonical origin AND decl empty `md/md.go:1033-1066`.

**Precision note (non-blocking):** the spec's §3 line 19 phrasing "explicit origin mandatory for ALL 4 types" is over-broad as a *validator* statement: per `canonical_origin.rs:48-54`, pkh→`m/44'/0'/0'`, wpkh→`m/84'/0'/0'`, and tr(key-path, tree:None)→`m/86'/0'/0'` all HAVE canonical origins, so the validator does NOT require explicit origin for those three; only `sh(wpkh)` (None) does. HOWEVER the spec's *design intent* — emit explicit origin on the wire for all 4 to match the toolkit's `synthesize.rs` (which always uses `Shared(origin_path)`) and guarantee byte-deterministic cross-tool parity (C3) — is correct and is what makes the spec right. The spec already states this correctly in §3 line 19's parenthetical ("BIP-49 sh(wpkh) REQUIRES it... validateExplicitOriginRequired else rejects") and in C3/§5 line 49. So the over-broad word "mandatory" is about the *emission policy*, not the *validator*, and is harmless. Recommend the T6a-1 plan phrase it as "explicit origin EMITTED for all 4 (validator-required only for sh-wpkh)". Minor; CLOSED.

### C3 — deterministic emission locked — **CLOSED**

Spec now (§3 Phase A line 20, §5 line 48, §7 I-1, biggest-risk §8.1) locks the toolkit form: explicit `pathDecl.shared` = full BIP origin, fp TLV PRESENT, pubkey TLV PRESENT — so verify-bundle md1-exact-string-match (§6.A4, I-6) and cross-tool parity hold. This matches `synthesize.rs:143-159` and `cell_7_wpkh_full` `wallet_policy.rs:190-204` verbatim (both carry Shared explicit origin + fingerprints Some + pubkeys Some). Canonicalize is identity for n=1 but `EncodeSingleSig` still routes through `encodePayload`→`canonicalize` (§3 line 22) — the no-op-but-still-runs ruling (a) is preserved. CLOSED.

### C4 — ms1 recipe — **CLOSED**

Spec now (§3 Phase A line 24, §5 line 50, §6.A3, §7 I-2, biggest-risk §8.2) specifies `EncodeMS1(entropy) = NewSeed("ms",0,"entr",'s',[0x00‖entropy])`, flags it NET-NEW, and locks English/entr-only (wordlist-language dropped from parity, documented).

Evidence (all read first-hand):
- prefix `0x00` = entr: `msPrefixEntr = 0x00 // RESERVED_PREFIX: payload = [0x00][entropy]` (`codex32/mspayload.go:5-11`; xref ms-codec `consts.rs:17`).
- id FIXED literal `"entr"` (not fp-derived): doc-comment "`entr` for both entr and mnem secrets" (`mspayload.go:6-7`); ms-codec `TAG_ENTR = *b"entr"` (`consts.rs:36`); test uses `"entr"` (`mspayload_test.go:54`).
- share index lowercase `'s'`: `NewSeed("ms", 0, "entr", 's', data)` (`mspayload_test.go:54`); all wire vectors `ms10entrs...` (`mspayload_test.go:25-29`); ms-codec `SHARE_INDEX_V01 = b's'`. `feFromRune`/encode normalizes case to HRP case (lowercase `"ms"` → lowercase output via `charsLowerTbl`, `codex32.go:262-275`, `gf32.go`), so `'s'` is canonical.
- `EncodeMS1` is NET-NEW: grep `func EncodeMS1` → zero hits; only `DecodeMS1` (`mspayload.go:34`) and `NewSeed(hrp string, threshold int, id string, shareIdx rune, data []byte) (String, error)` (`codex32.go:279`) exist. Spec flags this NET-NEW (§3 line 24, §5 line 50, build-inventory).
- English/entr language lock: entr payload `[0x00][entropy]` carries NO language byte (`mspayload.go:8-11`); on-device only language 0 (English) renders words via `bip39.New`, non-English shows name+hex "Words not shown on this device" (`gui/ms1_decode.go:31-44`); `MSLanguageNames` lists 10 langs but they only apply to the `0x02` mnem variant (`mspayload.go:20-26`). Spec correctly drops wordlist-language from the parity set and documents the English-only caveat + the `0x02` mnem follow-on. CLOSED.

---

## IMPORTANT FINDINGS — status

### I1 — §6.A1 gate rewrite — **CLOSED**

§6.A1 (line 56) is now a Rust↔Go cross-encoder differential over wallet-policy descriptors (concrete xpub bytes + fp + explicit origin), for wpkh/pkh/tr/sh-wpkh × {with-fp}, asserting `encode_md1_string` byte-equality; optional vendoring of NEW key-bearing goldens at the pinned SHA. No longer references the unsatisfiable template-only-golden parity. Internally consistent with C1. CLOSED.

### I2 — concrete `EncodeSingleSig` signature — **CLOSED**

§3 line 23: `md.EncodeSingleSig(chainCode [32]byte, compressedPubkey [33]byte, fp [4]byte, origin []PathComponent, script ScriptKind) ([]string, error)`; the GUI caller does base58-xpub→bytes + fp uint32→[4]byte. This matches the source plumbing: `deriveAccountXpub(...) (xpub string, masterFP uint32, err error)` returns base58 + uint32 (`gui/derive.go:19`); the TLVs need `[65]byte`/`[4]byte` (`md/md.go:505-512`). Taking parsed components keeps private material out and is the right minimal surface (Ruling a: building the unexported `*descriptor` inside package `md` is the right home — `body.isBody()` unexported `md/md.go:103`). M5's pathComponent/in-band-hardening distinction is honored (§3 line 23: "NOT the in-band-hardening `bip32.Path` convention"). CLOSED. (See precision note P1 on the exported `PathComponent` type.)

### I3 — restated scrub schedule — **CLOSED**

§4 (D11, restated precisely) now states: (a) gate `m.Entropy()` on validity FIRST (it panics on invalid) + `defer wipeBytes` the entropy after `EncodeMS1`; (b) seed/master/intermediates scrubbed INSIDE `deriveAccountXpub`; (c) the MNEMONIC `[]Word` is the spanning secret, zeroed after its last derivation consumer (md1 needs only the public xpub, NOT the mnemonic; restore-doc is fully public — the seed never reaches it); (d) verify re-types; ms1 `strings.Builder` residual accepted.

Evidence (all read first-hand):
- `m.Entropy()` `if !m.Valid() { panic("invalid mnemonic") }` returns fresh `[]byte` (`bip39/bip39.go:156-164`). Validity-gate-first is correct.
- `deriveAccountXpub` (`gui/derive.go:19-52`): `defer wipeBytes(seed)`; `masterFP = bip32.Fingerprint(pk)` captured BEFORE zeroing; each intermediate `k.Zero()`'d; the R0-C1 serialize-before-Zero ordering (`xpub = acct.String()` then `k.Zero()`) is real and load-bearing (documented in-source). The seed is internal and scrubbed before return.
- mnemonic `[]Word` zeroed on flow return (`gui/derive_xpub.go:106-117`, `for i := range mnemonic { mnemonic[i] = 0 }`).
- restore-doc is xpub/descriptor-only: `address.Receive/Change(desc *bip380.Descriptor, index)` (`address/address.go:20-26`); md1 expand builds `bip380.Key` from public material (`gui/md1_expand.go:60-77`). The seed never reaches restore-doc — the spec's correction (vs R0's mis-attribution) is accurate.
- ms1 `strings.Builder` `.String()` residual: `payload := ret.String()` / `return String{payload}` (`codex32/codex32.go:363,374,382`); accepted as a known limitation consistent with the shipped ms1-display posture. CLOSED.

### I4 — typed-only test wiring — **CLOSED**

§3 Phase B line 29 + §4 line 41 + §6.B6: the T6a program calls `seedEntryFlow` (as `deriveXpubFlow` does, `gui/derive_xpub.go:106-107`) and contains NO `act.scan`→derive path; the `assembleScan` footgun is named precisely; B6 gives a falsifiable shape — a structural assertion that the flow references `seedEntryFlow` and NOT `scan`/`assembleScan` for the secret, PLUS a behavioral test that a scanned `bip39.Mnemonic` cannot reach the derive entrypoint.

Evidence: `assembleScan` parses `bip39.Parse(buf)` → returns a mnemonic, and `codex32.New(...)` from NFC (`gui/scan.go:60-71`) — the footgun is real. `seedEntryFlow(ctx, th) (bip39.Mnemonic, bool)` is the typed entry (`gui/derive_xpub.go:107`). The ms1-never-NFC posture is reusable: `clsMs1Refuse // a codex32 secret (HRP ms) — refuse, never NFC` (`gui/bundle.go:45,70`). CLOSED.

### I5 — network LOCKED mainnet-only — **CLOSED**

§3 Phase B line 30 + §6.B2 + I-8 drop the "× network" axis: the 4 single-sig types, mainnet-only; testnet is a follow-on (§3 OUT). Source: the md1/restore stack is hard-locked to `Network: &chaincfg.MainNetParams, // D1: mainnet-only.` (`gui/md1_expand.go:61`). CLOSED.

---

## RULING e (split) — **ADOPTED**

§2 adopts T6a-1 (headless: `md.EncodeSingleSig` + `codex32.EncodeMS1` + verify-bundle comparator) / T6a-2 (GUI: new program, typed entry, pick list, derive-all-3, watch-only, engrave reuse, verify flow, restore doc), with T6a-1 carrying its OWN focused plan-R0 on the wire format before any GUI (§2 line 11, §9). Mirrors #10a/#10b. CLOSED.

## USER requirement (pick list in Phase B) — **CONFIRMED**

The wallet-type PICK LIST (4 single-sig types: BIP-44 pkh, BIP-49 sh(wpkh), BIP-84 wpkh, BIP-86 tr key-path; mainnet-only) is in Phase B (§3 Phase B line 30; §6.B2; I-8). CLOSED.

---

## RE-CONFIRM (drift checks) — all hold

- **canonicalize no-op for n=1 but still runs (ruling a):** §3 line 22 preserves it. Verified: single `@0` hits the identity fast-path (`md/canonicalize.go:53-63`) but `encodePayload`→`canonicalize` is still routed (`md/encode.go`). Not bypassed.
- **`EncodeSingleSig` home in package `md`:** §3 line 18 ("the only place the AST is constructible — `body.isBody()` is unexported `md/md.go:103`"). Correct — `isBody()` unexported, so the AST is unconstructible elsewhere; exported `md.EncodeSingleSig` calling `split`/`encodeMD1String` (`md/chunk.go:121`, `md/encode.go:451`) is the right surface.
- **encoder doesn't validate pubkey (decode does → round-trip safety net A2):** §3 line 22, §6.A2, §5 line 48. Correct — on-curve check lives in decode; A2 (`DecodeChunks`/`Decode`→`ExpandWalletPolicy` recovers xpub/fp/origin/script) is the safety net.
- **engrave reuse (`bundleEngrave` `[]bundleCard`-driven) + completion-message variants (M1):** §3 Phase B line 33, §6.B3. `bundleEngrave(ctx, th, cards []bundleCard)` is card-driven (`gui/bundle_flow.go:327`); `bundleCard.strings` verbatim (`gui/bundle.go:29-37`); full→"ms1 engraved", watch-only→show the `bundleMs1ReminderText` reminder (`gui/bundle_flow.go:374`). Variants locked in B3.
- **restore-doc public/no-secret (M2):** §3 Phase B line 35, §6.B5 ("greps clean of any xprv"). xpub/descriptor-only build (`gui/md1_expand.go:60-77` + `address.go:20-26`). Holds.
- **verify determinism (M3):** §3 line 25, §6.A4/B4, I-6. mk1 deterministic (csid from bytecode SHA, not RNG — `mk/encode.go:31-33,236-237`); md1 deterministic; ms1 compared on recovered ENTROPY bytes (string-match also valid since id+prefix+share pinned). Holds.
- **8 lockstep sites (M4):** §3 Phase B line 28, §6.B7, I-9. All 8 verified present: enum (`gui/gui.go:147-151`), dispatch (`:1491-1497`), wrap bound `if m.prog > engraveBundle` (`:1634-1641`), title arm (`:1659-1664`), npage (`:1840`), npages (`:1859`), nav-test (`gui/derive_xpub_program_test.go`). "Before `qaProgram`, both arms" is the correct insertion (qaProgram is the debug terminal enum value; the new user-facing program becomes the new nav ceiling — see precision note P2).
- **pathComponent vs in-band-hardening not conflated (M5):** §3 line 23. Raw encoder component `pathComponent{value, hardened}` depth ≤ 15 (`md/encode.go:88-93`, `errPathDepth`) vs the expand/display `componentsToPath` which adds `+hdkeychain.HardenedKeyStart` (`md/expand.go:146-162`). Spec keeps them distinct.

The 10 invariants (I-1..I-10) and the acceptance gate (A1–A4 Phase A; B1–B7 Phase B) cover every folded finding: A1↔C1/I1, A2↔round-trip safety net, A3↔C4, A4/I-6↔M3 determinism; B2/I-8↔pick-list+I5, B3↔M1, B5↔M2, B6↔I4, B7↔M4/scrub/fuzz/no-regression; I-1↔C1/C2/C3, I-2↔C4, I-3↔I4/D12, I-4↔I3/D11, I-5↔.Neuter, I-9↔lockstep.

---

## PRECISION NOTES (non-blocking; NOT gate-blocking; fold into the T6a-1 plan)

- **P1 — exported `PathComponent` is a NET-NEW exported type.** §3 line 23's signature uses `[]PathComponent` and `PathComponent{Hardened bool, Value uint32}`, but the internal type is the UNEXPORTED `pathComponent struct{...}` (`md/md.go:173`, fields `value`/`hardened`). Introducing an exported `PathComponent` (or accepting `[]uint32` BIP32 path + converting internally) is a legitimate public-API choice; the T6a-1 plan should state explicitly whether it adds a new exported `md.PathComponent` type or reuses an existing exported path representation, and map its fields to the internal `pathComponent`. Benign API-surface detail.
- **P2 — "explicit origin mandatory for ALL 4" wording (see C2 precision note).** The emission policy (emit explicit origin for all 4 to match the toolkit) is correct; the *validator* requires it only for `sh(wpkh)`. Recommend the T6a-1 plan phrase it as "explicit origin EMITTED for all 4; validator-required only for sh-wpkh" to avoid an implementer over-reading "mandatory" as a validator constraint.
- **P3 — nav-ceiling edit.** Inserting the new program before `qaProgram` means the nav wrap bound (`if m.prog > engraveBundle`, `gui/gui.go:1641`) and the derived consts (`int(engraveBundle)+1`, `:1840,:1859`) must be re-pointed to the NEW program (the new last user-facing enum value before `qaProgram`), not left at `engraveBundle`. §6.B7/I-9 cover this at spec altitude ("both arms, derived consts, nav-test updated"); the T6a-1/T6a-2 plan must enumerate the exact line edits. (M6 — no hardware — remains a known residual, not a blocker.)

---

## What GREEN means here
C1–C4 and I1–I5 are CLOSED with first-hand source evidence; the fold introduced no drift; the split (Ruling e), the pick-list placement (Phase B), and the 10 invariants + acceptance gate are coherent and source-faithful. **The spec is cleared for the T6a-1 (headless) implementation plan**, which must itself pass a focused plan-R0 on the `EncodeSingleSig` wallet-policy wire (the byte-lock risk) before any code, per the project's R0 gate. The three precision notes (P1–P3) are plan-level, not spec-gate-blocking.
