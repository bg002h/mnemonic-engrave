# RECON (T6, agent a744d5d7478925cb5, 2026-06-19) — device-side build feasibility (source-verified vs fork `e4013a8`)

Recon only. `go build ./...` exit 0 (clean baseline). All citations live at `e4013a8` (T1-T5 + #10 merged).

## HEADLINE
T6 is **mostly assemblable from shipped primitives EXCEPT the md1 leg, the make-or-break GAP.** The md1 encoder exists + is byte-faithful (#10a goldens) but is **entirely UNEXPORTED + test-only**: it consumes the md-package internal `*descriptor` AST, and there is **NO public path from a seed/xpub/`*bip380.Descriptor` to that AST.** T6 cannot produce an md1 string for a self-derived descriptor without **net-new md-package API.** Every other leg (mk1, ms1, verify-bundle, restore-doc, engrave sequencing, secret spine) is reuse-with-glue.

## Primitives table
| # | Capability | Status | Evidence |
|---|---|---|---|
| 1 | seed → account xpub → mk1 (T4 chain) | **EXISTS** | `deriveAccountXpub` `gui/derive.go:19`; `mk.Encode` `mk/encode.go:38`; `multiPlateEngrave` `gui/derive_xpub.go:263`; `deriveXpubFlow` `gui/derive_xpub.go:106` |
| 2a | ms1 ENCODER (entropy→`ms1…`) | **EXISTS (lib), GAP (on-device wiring)** | `codex32.NewSeed(hrp,threshold,id,shareIdx,data)` `codex32/codex32.go:279` — used only in host `cmd/biptool/main.go:312`, never in `gui/` |
| 2b | mnemonic → entropy | **EXISTS** | `bip39.Mnemonic.Entropy()` `bip39/bip39.go:158`; inverse `bip39.New(entropy)` `:228` |
| 2c | on-device engrave seed-as-ms1 | **GAP (net-new)** | today seed engraved as BIP-39 WORDS: `engraveSeed`→`backup.Seed{Mnemonic}` `gui/gui.go:468`. The ms1-string engrave path (`backupSeedStringFlow`→`backup.EngraveSeedString` `gui/gui.go:2049`, `backup/backup.go:75`) exists but is fed by a SCANNED codex32 string `gui/codex32_polish.go:228`, never derived |
| 3 | seed/xpub → md1 descriptor | **GAP — make-or-break** | see below |
| 4 | verify-bundle (read back + cross-check) | **PARTIAL** | gather/decode exist; parity compare net-new |
| 5 | restore doc (fp+addrs+descriptors) | **PARTIAL** | display pieces exist; needs from-xpub `*bip380.Descriptor` build + new screen |
| 6 | T5 engrave sequencing reuse | **PARTIAL (small glue)** | `bundleEngrave([]bundleCard)` string-driven, not scan-bound |
| 7 | secret spine | **EXISTS** | `wipeBytes` `gui/slip39_polish.go:330`; `.Zero()` `gui/derive.go:28-51`; mnemonic scrub `gui/derive_xpub.go:113` |

## #3 — md1-from-seed: the make-or-break GAP
Encoder surface ALL unexported: `encodeMD1String(d *descriptor)(string,error)` `md/encode.go:451`; `split(d *descriptor)([]string,error)` `md/chunk.go:121`; `encodePayload(d *descriptor)` `md/encode.go:373`. Input `descriptor struct{n,pathDecl,useSite,tree,tlv}` `md/md.go:816` UNEXPORTED, as are `node` `:135`, `pathDecl` `:209`, `originPath` `:190`, `useSitePath` `:265`, `tlvSection` `:523`, all body variants `:105-123`. The `body` interface `:103` has an UNEXPORTED `isBody()` → **cannot be implemented/constructed outside package `md` at all.**
Production callers: NONE (grep `encodeMD1String|encodePayload|split(|md.Encode|EncodeMD1` over prod = only defs + internal `md/identity.go:12`,`md/chunk.go:122`). All other callers `_test.go`, building the AST by hand via struct literals (`md/encode_test.go:224`, `md/chunk_test.go:26`) — possible only because in-package.
Exported decode path is LOSSY: `md.Decode→md.Template` `md.go:1216/1196`, `md.DecodeChunks` `expand.go:25`; `Template`/`KeyOrigin` are summaries, NOT round-trippable to `*descriptor` (xpub bytes live in the AST TLV, not Template). md package does NOT import bip380 (grep); only bridge is REVERSE (md `Template`+`ExpandedKey`→`*bip380.Descriptor`, `gui/md1_expand.go:32`). NO `descriptorFromXpub`/`buildDescriptor`/exported `md.Encode` anywhere.
**⇒ T6 producing md1 requires NET-NEW md API:** (1) narrow `md.EncodeSingleSig(xpub, origin, useSite, script)([]string,error)` building the `*descriptor` internally + calling `split`/`encodeMD1String`; OR (2) exported descriptor-AST builder + `Encode(*Descriptor)`. New PUBLIC API on a package byte-locked to Rust goldens — the DOMINANT T6 risk, must be R0-gated + external-protocol-fact-verified vs Rust md-codec. For "from one seed" the descriptor is SINGLE-SIG (one derived xpub, standard path, `<0;1>/*`) = the SIMPLEST AST (`tree=node{tag:tagWpkh/tagPkh/tagTr, body:keyArgBody{0}}`, n=1, one pubkey TLV) → a narrow `EncodeSingleSig` is tractable but still new golden-gated surface, not glue.

## #4 — verify-bundle: PARTIAL
Reuse: gather back `bundleGatherer` `gui/bundle.go:118` + `bundleGatherFlow` `gui/bundle_flow.go:95`; integrity decode `mk.Decode` `bundle.go:194`, `md.DecodeChunks` `:234`, `md.Decode` `:151`; summaries `mk1Summary` `:300`, `bundleMD1Summary` `:310`. **Net-new: NO parity/compare helper** — T6 adds a comparator (derived `mk.Card` vs gathered field-equality; derived md1 strings vs gathered exact-string — works because `mk.Encode`/md encoder DETERMINISTIC, `mk/encode.go:30-34`, csid from bytecode hash not RNG). **Asymmetry: ms1 REFUSED over NFC** (`clsMs1Refuse` `bundle.go:70`) → verify-bundle covers mk1+md1 only; ms1 verification operator-VISUAL.

## #5 — restore doc: PARTIAL (assemble + one from-xpub build)
Ingredients exist: fingerprint `bip32.Fingerprint` `bip32/bip32.go:38` (already captured as `masterFP` `derive.go:31`); first addrs `address.Receive`/`Change(desc,index)` `address/address.go:20-24`; display `md1DisplayFlow` `gui/md1_inspect.go:77`, `xpubVerifyFlow` `gui/derive_xpub.go:170`. **Net-new (small):** build `*bip380.Descriptor` from the derived xpub (struct directly constructible, literal at `gui/md1_expand.go:60-77`; KeyData/ChainCode from xpub, MasterFingerprint, DerivationPath, `Children=[RangeDerivation{0,1},Wildcard]`) OR `bip380.Parse` `bip380/bip380.go:269`; then a new read-only screen (fp + first N addrs + descriptor text). No new crypto.

## #6 — T5 sequencer reuse: PARTIAL (small glue, NOT scan-bound)
`bundleEngrave(ctx,th,cards []bundleCard)` `gui/bundle_flow.go:327` is `[]bundleCard`-driven; `bundleCard.strings` = verbatim strings `bundle.go:32-36`; `bundlePlatePlan` `bundle_flow.go:303` flattens. NOT gather-coupled — T6 can synthesize `[]bundleCard` from its derived strings + call `bundleEngrave` with no signature change (`bundleCard` is unexported same-package `gui`). Note `bundleMs1ReminderText` `bundle_flow.go:374` assumes ms1 hand-engraved separately → T6 (which DOES engrave ms1) needs a different completion message. `validateMdmk` `gui/gui.go:1903` is the shared per-string plate builder.

## #7 — secret spine: EXISTS, sound; new surfaces
Reuse: `wipeBytes`; seed `defer wipeBytes` `derive.go:21`; master+intermediates `.Zero()` `:28-51` (R0-C1 ordering: serialize xpub BEFORE `k.Zero()`); mnemonic scrub `derive_xpub.go:113`; ms1 read scrubs entropy `gui/ms1_decode.go:29`.
**NEW exposure surfaces (flag for spec):** (1) raw entropy for ms1 — `mnemonic.Entropy()` returns a fresh SECRET `[]byte` → must `defer wipeBytes`; `codex32.NewSeed` builds via `strings.Builder` whose `.String()` (`codex32.go:363,374`) is an IMMUTABLE secret-bearing string un-wipeable until GC (same residual already accepted for ms1 display `ms1_decode.go:25-26`) — acknowledge. (2) ms1 IS the secret + T6 engraves it — engraving onto owner-held steel is the intended backup; invariant: ms1 typed/derived→engraved-only, NEVER NFC (bundle channel already refuses, `bundle.go:70`). (3) single derivation, multiple consumers — keep secret lifetime to one scope, one `defer` scrub block; derive public xpub first, discard entropy/mnemonic before the (all-public-except-ms1) engrave.

## Biggest build risks (ranked)
1. **md1 encoder has no public entry (#3) → NEW md-package API** (highest; new exported surface on a byte-golden-locked pkg; recommend narrow `md.EncodeSingleSig`; R0 + external-protocol-fact rule vs Rust md-codec).
2. **No verify-bundle comparator (#4)** — net-new; deterministic encode makes exact-match viable; ms1 can't be re-scanned → ms1 verification operator-visual only (state the asymmetry).
3. **ms1 engrave net-new on-device (#2c)** — `Entropy()`→`NewSeed("ms",0,id,'S',entropy)`→sequencer; the 4-char codex32 `id` (`len(id)==4` `codex32.go:280`) is a new product decision.
4. **Restore-doc descriptor build + new screen (#5)** — lowest risk.
5. **Bundle completion-message mismatch (#6)** — minor; `bundleEngrave` ends with an ms1-not-engraved reminder wrong for T6 → variant/param.
