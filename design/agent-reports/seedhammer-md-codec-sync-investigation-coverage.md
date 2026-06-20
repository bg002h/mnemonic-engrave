# SeedHammer fork Go md-codec — descriptor-SHAPE coverage investigation (template-engrave cycle)

**Agent:** read-only investigation (plus this one report file). **Date:** 2026-06-20.
**Scope:** classify the fork's Go md-codec support per descriptor shape as SUPPORTED / PARTIAL / UNSUPPORTED, with **decode / encode / expand-display** reported SEPARATELY, against the question "can the fork handle *any admissible md1 template*?"

## Sync block

| Repo | HEAD | Role |
|---|---|---|
| `seedhammer` (fork, Go/TinyGo) | `39cb5cf` | Go md-codec in `md/`; expand/display in `gui/` |
| `descriptor-mnemonic` (md-codec v0.37.0, Rust) | `54dd765` | reference model — `tree.rs` (Node/Body/Tag), `tag.rs`, `to_miniscript.rs`, `validate.rs` |

## Method + the three distinct fork paths

The fork has **three structurally different layers**, and they have wildly different coverage. This is the crux of the whole report:

1. **DECODE** — `md/md.go:330-490` `readNodeDepth` + `md/md.go:82-98` `readTag`. A faithful, complete port of Rust `tree.rs:196-328` `read_node` and `tag.rs:156-202`. Wire-level: reads any node.
2. **ENCODE** — two sub-layers:
   - **low-level** `md/encode.go:159-232` `writeNode` — faithful inverse of `read_node`; can emit any body type.
   - **public encoders** — the ONLY callers that build trees: `md/encode_singlesig.go:36` `EncodeSingleSig` (4 fixed single-sig shapes) and `md/encode_multisig.go:89` `EncodeMultisig` (3 fixed multisig shapes, sortedmulti-only). **No public API accepts a caller-built `node` tree.** Confirmed: the only non-test callers of `EncodeSingleSig`/`EncodeMultisig`/`writeNode`/`encodePayload`/`split` are `gui/singlesig_derive.go:61`, `gui/multisig_build.go:417-424`, `md/identity.go:12`, `md/chunk.go:122`, `md/walletpolicyid.go:42` — none builds an arbitrary tree.
3. **EXPAND / DISPLAY** — `md/md.go:1266-1328` `classifyPolicy`/`multiPolicy` (the renderable-shape classifier producing `Template`), `gui/md1_expand.go:82-121` `scriptForTemplate` (the bip380-descriptor projection), `gui/md1_inspect.go:36-70` `policyLine`/`md1Summary` (the on-device display). This layer is **much narrower** than decode and is where the real gaps are.

The Rust reference (the "admissible md1" universe) carries **36 Tags** (`tag.rs:15-89`) and **8 Body variants** (`tree.rs:18-73`): `Children`, `Variable{k,children}` (Thresh), `MultiKeys{k,indices}` (Multi/SortedMulti/MultiA/SortedMultiA), `Tr{is_nums,key_index,tree}`, `KeyArg`, `Hash256Body`, `Hash160Body`, `Timelock`, `Empty`. The Go side carries the **identical 36 tags** (`md/md.go:39-76`) and **identical 9 body structs** (`md/md.go:103-133`; `hash256Body`+`hash160Body` are two structs but cover Rust's two hash bodies). **Tag/Body parity at the AST level is exact.**

---

## Coverage matrix (shape × {decode, encode, expand-display})

Legend: **S** = SUPPORTED, **P** = PARTIAL, **U** = UNSUPPORTED.
"expand-display" = is the shape classified renderable AND projected to a verifiable descriptor (expandOK), vs. summarized-but-display-only, vs. dropped to "Complex policy".

| Shape | Decode | Encode | Expand / Display | Evidence (fork) |
|---|:--:|:--:|:--:|---|
| **single-sig pkh** | S | S | S | dec `md.go:340` (tagPkh→keyArg); enc `encode_singlesig.go:94`; expand `md1_expand.go:90-91` (P2PKH); classify `md.go:1268-1271` |
| **single-sig wpkh** | S | S | S | dec `md.go:340`; enc `encode_singlesig.go:96-97`; expand `md1_expand.go:88-89` (P2WPKH); classify `md.go:1268-1271` |
| **single-sig sh(wpkh)** | S | S | S | dec `md.go:346` (Sh)→`md.go:340` (Wpkh); enc `encode_singlesig.go:100-102`; expand `md1_expand.go:99-102` (P2SH_P2WPKH, keyed on `InnerWpkh`); classify `md.go:1296-1299` |
| **single-sig tr(@N) keypath** | S | S | S | dec `md.go:432-457` (trBody, tree==nil); enc `encode_singlesig.go:98-99`; expand `md1_expand.go:93-94` (P2TR); classify `md.go:1282-1284` (`!isNums && tree==nil`) |
| **wsh(sortedmulti)** | S | S | S | dec `md.go:386`; enc `encode_multisig.go:192-193` (MultisigWsh); expand `md1_expand.go:106-107` (P2WSH, SortedMulti); classify→`multiPolicy md.go:1323` |
| **wsh(multi)** (unsorted) | S | **U** (no encoder) | **P** (summarize-only, no verify) | dec `md.go:386` (tagMulti); enc: NO public path (`EncodeMultisig` is sortedmulti-only, `encode_multisig.go:190`); classify→`PolicyMulti md.go:1322` (renderable, displays "k-of-m multisig"); BUT `scriptForTemplate` has **no PolicyMulti arm** (`md1_expand.go:104-116` only `PolicySortedMulti`) → falls to `expandUnsupported md1_expand.go:120` → display-only, never verified |
| **sh(wsh(sortedmulti))** | S | S | S | dec `md.go:346`(Sh)→`md.go:346`(Wsh)→`md.go:386`; enc `encode_multisig.go:194-196` (MultisigShWsh); expand `md1_expand.go:112-113` (P2SH_P2WSH, keyed on `InnerWsh`); classify `md.go:1302-1307` |
| **sh(wsh(multi))** (unsorted) | S | **U** (no encoder) | **P** (summarize-only, no verify) | dec same path, inner tagMulti; enc: none; classify→PolicyMulti (renderable); `scriptForTemplate` no PolicyMulti arm → `expandUnsupported` |
| **sh(sortedmulti)** (bare legacy P2SH) | S | S | S | dec `md.go:346`(Sh)→`md.go:386`; enc `encode_multisig.go:197-198` (MultisigSh); expand `md1_expand.go:115` (bare P2SH, SortedMulti); classify `md.go:1310-1312` |
| **sh(multi)** (bare legacy, unsorted) | S | **U** | **P** (summarize-only, no verify) | dec same; enc none; classify→PolicyMulti; no `scriptForTemplate` PolicyMulti arm → `expandUnsupported` |
| **tr(NUMS, multi_a)** | S | **U** (no encoder) | **U** | dec `md.go:432-457` (`isNums`)+`md.go:386` (tagMultiA); enc: NO path (`EncodeSingleSig` ScriptTr is keypath-only `encode_singlesig.go:99`; no taproot-multisig encoder exists); classify→**PolicyComplex** (`md.go:1272-1285`: ANY tr with a `tree` is refused; `multiPolicy md.go:1318-1328` only matches Multi/SortedMulti, never MultiA) → `Renderable=false` → "Complex policy — display only" `md1_gather.go` / `md1_inspect.go:60` |
| **tr(NUMS, sortedmulti_a)** | S | **U** | **U** | dec `md.go:386` (tagSortedMultiA); enc: none; classify→PolicyComplex (tr-with-tree refusal) |
| **taproot taptree depth-1 (single leaf)** | S | **U** | **U** | dec `md.go:432-457` then leaf via `md.go` node arms; decode-side `validateTapScriptTree md.go:1005-1020` enforces permitted leaves; enc: none; classify→PolicyComplex (tr-with-tree) |
| **taproot taptree depth-≥2 / multi-leaf** | S | **U** | **U** | dec `md.go:376-385` (tagTapTree, 2 children, recursive) + depth cap `md.go:331`; enc: none; classify→PolicyComplex |
| **miniscript or_i / or_d / or_c** | S | **U** | **U** | dec `md.go:352-361` (OrI/OrD/OrC two-child); enc: none; classify→PolicyComplex (no combinator arm in `classifyPolicy`) |
| **miniscript and_v / and_b** | S | **U** | **U** | dec `md.go:352-361` (AndV/AndB); enc: none; classify→PolicyComplex |
| **miniscript andor** | S | **U** | **U** | dec `md.go:362-375` (3-child); enc: none; classify→PolicyComplex |
| **miniscript thresh** | S | **U** | **U** | dec `md.go:409-431` (`variableBody{k,children}`); enc low-level `encode.go:170-187` exists but no public caller; classify→PolicyComplex |
| **after / older** | S | **U** | **U** | dec `md.go:458-463` (`timelockBody`, 32-bit); enc low-level `encode.go:216-217`; no public caller; classify→PolicyComplex |
| **sha256 / hash256** | S | **U** | **U** | dec `md.go:464-473` (`hash256Body` [32]B); enc `encode.go:218-221`; no public caller; classify→PolicyComplex |
| **hash160 / ripemd160 / raw pk_h** | S | **U** | **U** | dec `md.go:474-483` (`hash160Body` [20]B, covers Hash160/Ripemd160/RawPkH); enc `encode.go:222-225`; no public caller; classify→PolicyComplex |
| **pk_k / pk_h** | S | **U** | **U** | dec `md.go:340-345` (keyArg); enc `encode.go:162-163`; no public caller as a leaf; classify→PolicyComplex unless top-level wpkh/pkh |
| **multi_a inside combinators** | S | **U** | **U** | dec handles MultiA anywhere `md.go:386`; enc: none; classify→PolicyComplex |
| **use-site path overrides (per-cosigner TLV)** | S | **P** | S | dec `md.go:583-589, 674-705` (tlvUseSitePathOverrides=0x00); enc `encode.go:250-269` writes the TLV faithfully, BUT neither `EncodeSingleSig` nor `EncodeMultisig` ever **populates** `useSiteOverrides` (both hard-code one shared `<0;1>/*` use-site, `encode_singlesig.go:63-70` / `encode_multisig.go:140-144`) → no public encoder emits a per-cosigner override; expand resolves overrides `md1_expand.go:163-183` + `md.go:1429-1438` |
| **hardened use-site (`/*'` or fixed `/N'/`)** | S | **P** | **P** | dec `md.go:272-298` decodes `wildcardHardened` + per-alt `hardened`; enc `encode.go:129-151` writes them; **NOT refused at encode** (consistent with Rust — recon-codec Finding 2); expand: `useSiteToChildren md1_expand.go:129-154` **refuses** hardened wildcard (line 130-132, D5) and hardened multipath alt (line 142-144) → `expandUnsupported`, display-only, never verified (correct funds-safety guard — hardened public derivation is forbidden) |

---

## What the Rust side has that the Go side does NOT handle

### AST level (decode + low-level encode): NOTHING is missing.
Every one of the 36 Rust `Tag`s and all 8 `Body` variants is decoded by `readNodeDepth` and (re-)encodable by `writeNode`. The fork's wire codec is a **complete, faithful port** — including `Thresh`/`Variable`, `MultiA`/`SortedMultiA`, `TapTree`, all combinators (`OrI/OrD/OrC/OrB/AndV/AndB/AndOr`), all wrappers (`Check/Verify/Swap/Alt/DupIf/NonZero/ZeroNotEqual`), all leaves (`After/Older/Sha256/Hash256/Hash160/Ripemd160/RawPkH/PkK/PkH/False/True`), and `Tr{is_nums}`. Decode-side validators (`validatePlaceholderUsage`, `validateMultipathConsistency`, `validateTapScriptTree`, `validateExplicitOriginRequired`, `validateXpubBytes`) are all ported (`md.go:905-1083`). The depth cap (128) matches.

### Public ENCODE level: everything except 7 fixed shapes is unreachable.
There is **no public API to encode an arbitrary md1 template**. The only tree-builders are:
- `singleSigTree` (`encode_singlesig.go:92-106`): exactly **pkh, wpkh, tr-keypath, sh(wpkh)**.
- `multiSigTree` (`encode_multisig.go:185-202`): exactly **wsh/sh(wsh)/sh over *sortedmulti only*** — never `multi`, never `multi_a`, never `sortedmulti_a`, never a taptree, never a combinator.

So at the encode layer the Go side is missing: **unsorted `multi` (all wrappers), all taproot multisig (`multi_a`/`sortedmulti_a`), all taptrees, every general miniscript fragment/combinator, and per-cosigner use-site overrides / hardened use-sites** (no encoder populates them even though `writeNode`/`writeUseSitePath` can serialize them).

### EXPAND / DISPLAY level: the renderable classifier is the real bottleneck.
`classifyPolicy` (`md.go:1266-1315`) recognizes ONLY: top-level wpkh/pkh (single), tr-keypath (single, `!isNums && tree==nil`), wsh(multi|sortedmulti), sh(wpkh) (single), sh(wsh(multi|sortedmulti)), sh(multi|sortedmulti). **Everything else → `PolicyComplex` → `Renderable=false`.** Specifically the Go side does NOT classify/render, vs. what Rust `to_miniscript.rs:300-626` can render:
- **Any `tr` with a script tree** is hard-refused (`md.go:1276-1283` comment: summarizing a tapscript leaf would mislead) — so **tr(NUMS,multi_a), tr(NUMS,sortedmulti_a), all taptrees** never render; Rust `to_miniscript.rs:315-359, 423-451` renders them.
- **All general miniscript fragments/combinators** (or_*, and_*, andor, thresh, after, older, sha256/hash*, pk_k/pk_h as leaves) → PolicyComplex; Rust renders all of them (`to_miniscript.rs:451-620`).
- **`PolicyMultiA`/`PolicySortedMultiA` are dead enum values**: `policyLine` (`md1_inspect.go:44-47`) has display strings for them, but `classifyPolicy` never produces them (tr-with-tree is refused before reaching any MultiA path). Dead arms.
- **`PolicyMulti` (unsorted) is summarize-renderable but not projectable**: `classifyPolicy` returns `PolicyMulti` (so `md1Summary` shows "k-of-m multisig"), but `scriptForTemplate` (`md1_expand.go:104-116`) has **no `PolicyMulti` case** — only `PolicySortedMulti` — so any unsorted-`multi` wallet falls to `expandUnsupported` → display-only, **never address-verified.** This is the one place decode/summarize disagree with expand/verify.

Note these expand/display refusals are *intentional* (the D2 faithful-or-refuse policy: never build a bip380 descriptor or verify an address for a shape it can't express). They are safe (display-only, no wrong-address risk), but they are NOT "support" for engraving/round-tripping the shape.

---

## Closing verdict

**Is the fork's Go md-codec shape-coverage sufficient for "any admissible md1 template" (the cycle's stated scope)? NO — not as currently built, IF the cycle requires *encoding/engraving* or *expand-and-verify* of arbitrary templates.** It is sufficient ONLY if the cycle's "template engrave" means *re-engrave a decoded card's bytes* (decode is complete and faithful).

Breakdown by what "support" means for the cycle:

- **Decode (round-trip / WDT-Id binding):** **SUFFICIENT.** The Go decoder handles every admissible md1 the Rust codec admits — all 36 tags, all bodies, all validators. Any admissible md1 template decodes today. (Caveat: the cycle still needs the `WalletDescriptorTemplateId` + form-aware-stub Go port flagged in `seedhammer-template-engrave-recon-codec.md` Finding 4 — but that is identity, not shape decode.)

- **Encode (engrave a freshly-built template):** **NOT SUFFICIENT.** Only 7 fixed wallet-policy shapes can be built (4 single-sig + 3 sortedmulti). To engrave "any admissible md1 template" the fork needs a generic template encoder (or a much wider shape allow-list).

- **Expand / on-device display + verify:** **NOT SUFFICIENT** for the broad-miniscript goal; the renderable classifier is the bottleneck.

### Exact gap shapes (what needs shape-coverage work, in priority order for "broad miniscript")

1. **Taproot multisig — `tr(NUMS, multi_a)` and `tr(NUMS, sortedmulti_a)`:** UNSUPPORTED on **encode AND expand** (decode OK). Highest-value gap — the canonical modern hardware-wallet taproot multisig. `classifyPolicy` hard-refuses any tr-with-tree; no encoder exists. The dead `PolicyMultiA`/`PolicySortedMultiA` enum arms suggest this was anticipated but never wired.
2. **Taproot taptrees (depth-1 single-leaf and depth-≥2 / multi-leaf):** UNSUPPORTED on encode AND expand (decode OK, including the leaf-tag validator).
3. **Unsorted `multi` (wsh / sh(wsh) / bare sh):** PARTIAL — decode + summarize OK, but **no encoder** and **no `scriptForTemplate` PolicyMulti arm** (so display-only, never verified). Cheapest expand fix (add a `PolicyMulti` case mirroring the `PolicySortedMulti` arms) + an `EncodeMultisig` unsorted variant.
4. **General miniscript fragments/combinators** — `or_i, or_d, or_c, or_b, and_v, and_b, andor, thresh, after, older, sha256, hash256, hash160, ripemd160, pk_k, pk_h, multi_a-in-combinator`: UNSUPPORTED on encode AND expand (decode OK). This is the bulk of "broad miniscript coverage." Needs both a generic tree encoder and an expand/display path that renders arbitrary miniscript (the Rust `to_miniscript.rs` analog the fork does not have).
5. **Per-cosigner use-site overrides (UseSitePathOverrides TLV):** PARTIAL — decode + low-level encode + expand-resolve all OK, but no public encoder *populates* the override (both encoders hard-code one shared `<0;1>/*`). Needs encoder API surface if the cycle wants to mint per-cosigner-override cards.
6. **Hardened use-site (`/*'` or fixed `/N'/`):** PARTIAL — decode + encode OK and (correctly) NOT refused at encode; expand correctly refuses (D5 funds-safety: BIP-32 forbids hardened public derivation on xpub-only restore). This refusal is *correct* and should stay; only flag it so the cycle does not mistake it for a bug.

**Bottom line for the cycle:** the AST/wire codec is done and faithful; the work is (a) a **generic template encoder** (replace the 7 hard-coded shapes), and (b) a **wider renderable/expand classifier** (port the `to_miniscript.rs` shape universe, starting with taproot multisig + unsorted multi). Decode needs no shape work.
