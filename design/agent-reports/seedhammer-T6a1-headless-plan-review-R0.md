# R0 GATE REVIEW (FOCUSED, wire-format) — IMPLEMENTATION_PLAN_seedhammer_T6a1_headless.md (opus architect)

**Date:** 2026-06-19
**Round:** R0 (the mandatory FOCUSED plan-R0 wire-format gate for T6a-1, BEFORE any code).
**Reviewer role:** byte-lock gate. T6a-1 introduces NEW public API on a Rust-golden-byte-locked package (`md.EncodeSingleSig`) + a net-new ms1 encoder (`codex32.EncodeMS1`). The byte-lock is the whole point of this gate.
**PLAN under review:** `design/IMPLEMENTATION_PLAN_seedhammer_T6a1_headless.md`
**SPEC (GREEN @ R1):** `design/SPEC_seedhammer_T6a_singlesig_flagship.md` (Phase A = T6a-1; Phase B GUI = T6a-2, NOT in this plan).
**Prior wire-format findings honored:** `design/agent-reports/seedhammer-T6a-singlesig-spec-review-R{0,1}.md`.

**Sources verified first-hand @ pinned trees (NOT the plan's prose):**
- Fork `e4013a8` — `git rev-parse HEAD = e4013a88011284c71f6da1b5629555bdc52c7e88` (the plan's claimed base; "Merge T5"), tree `/scratch/code/shibboleth/seedhammer`. (Note: the repo submodule at `mnemonic-engrave/third_party/seedhammer` is upstream `713aee2` with no `md/`; the fork-with-T6a work is the sibling tree `/scratch/code/shibboleth/seedhammer`, which all the citations target — confirmed.)
- Rust md-codec **v0.36.0** — toolkit `Cargo.lock` resolves it to the crates.io REGISTRY crate v0.36.0 (checksum-pinned), and the local source-of-truth tree `/scratch/code/shibboleth/descriptor-mnemonic` is at `c85cd49`; the fork's vendored `md/testdata/vectors/*` were copied verbatim from that tree (see md/testdata/README.md). v0.36.0 == c85cd49 content for the cited lines.
- mnemonic-toolkit `/scratch/code/shibboleth/mnemonic-toolkit` (built + run live this round).

**Method:** every wire/protocol fact re-verified against source text and, where load-bearing, against a LIVE throwaway build/run (toolkit `mnemonic bundle`; Go host probes of `encodePayload`/`split`/`Decode`/`NewSeed`). Throwaway probes removed; working tree clean.

---

## VERDICT: NOT GREEN

**0 Critical, 2 Important, 5 Minor.**

The wire-format spine of the plan is overwhelmingly correct and source-faithful: the 4 AST tree shapes are exactly right and buildable from the unexported in-package types; the wallet-policy TLV (pubkeys+fingerprints) + explicit `pathDecl.Shared` origin matches the toolkit verbatim; the routing through `encodePayload`→`canonicalize` (n=1 no-op) is right; the `EncodeSingleSig` signature and the net-new exported `PathComponent` are sound; and the ms1 recipe is byte-exact (proven: `NewSeed("ms",0,"entr",'s',[0x00‖zeros16])` == the `ms10entrsqqqq…cj9sxraq34v7f` vector, matching both the fork's own vector and the live toolkit output). The differential-golden gate is FEASIBLE — the `mnemonic bundle` CLI emits a key-bearing single-sig md1 from seed+template+account, confirmed by a live run.

But two Important defects must be folded before code, both about the **output WIRE FORM**:

- **I1 — the plan does not lock the chunked-vs-single string form, and a single-sig wallet-policy payload is 644 bits = 81 bytes → it CANNOT be a single string.** The plan says `EncodeSingleSig` "calls the shipped `split`/`encodeMD1String`" (with a slash, as co-equal options) and returns `[]string`. But the toolkit ALWAYS emits md1 via `md_codec::chunk::split` (chunked, 3 strings for single-sig wpkh), NEVER `encode_md1_string`. `encodeMD1String` produces a single 145-char over-limit string the toolkit never emits and that routes through a DIFFERENT decoder. To match the byte-lock target, `EncodeSingleSig` MUST call `split` (chunked), and the plan must say so unambiguously and drop `encodeMD1String` as an option.
- **I2 — the round-trip leg names the wrong/ambiguous decoder.** Task 2.b says "`md.DecodeChunks`/`Decode`→`ExpandWalletPolicy`". `Decode` REFUSES chunked input (`ErrChunkedUnsupported`); `DecodeChunks`/`Reassemble` REFUSE single (non-chunked) input (`errChunkFlagMissing`). They are mutually exclusive by wire form. Since the output is chunked (I1), the round-trip leg is **`DecodeChunks`/`ExpandWalletPolicyChunks`** ONLY — and `ExpandWalletPolicy(*descriptor)` is reachable in-package but `DecodeChunks` returns a `Template`, so to recover the per-@N xpub/fp/origin the leg must use `ExpandWalletPolicyChunks([]string)` (or `Reassemble`→`ExpandWalletPolicy`).

Neither defect touches shipped behavior; both are plan-precision fixes to the new encoder's wire form. Fold I1+I2, re-persist, re-dispatch.

---

## RULINGS (as required)

### (a) The differential-golden gate feasibility — **FEASIBLE; the gate is achievable as written, with one form correction (I1).**

The plan's Task 0/Task 2.1 premise is correct and the gate is the make-or-break — and it is achievable:

1. **The template-only vendored vectors genuinely cannot serve.** Verified first-hand: EVERY `*.descriptor.json` in BOTH the fork `md/testdata/vectors/` AND the Rust source `descriptor-mnemonic/crates/md-codec/tests/vectors/` has `"pubkeys": null` (grep `'"pubkeys": *\['` → ZERO hits in either tree). `wpkh_basic.descriptor.json` is `path_decl Shared "m"` (empty origin), `tlv` all null — a bare template skeleton. `is_wallet_policy()` is true iff `pubkeys Some(non-empty)` (md-codec `encode.rs:50-52`). So a template-only golden produces a DIFFERENT, shorter wire than `EncodeSingleSig` (which embeds the Pubkeys TLV). The plan's R0-C1 carry-over is correct.

2. **The toolkit CAN generate key-bearing single-sig goldens — confirmed by a LIVE run.** The `mnemonic` binary (`crates/mnemonic-toolkit/Cargo.toml:15-17` `[[bin]] name="mnemonic"`) has a `bundle` subcommand (`main.rs:99-100,166`). It takes `--network`, `--template {bip44,bip49,bip84,bip86,…}` (`cmd/bundle.rs:29-30`; `template.rs:16-24` — bip44=pkh, bip49=sh(wpkh), bip84=wpkh, bip86=tr), `--account` (`bundle.rs:65-66`), and a secret slot `--slot @0.phrase=<mnemonic>` (`bundle.rs:108-120`), emitting `ms1`/`mk1`/`md1` (and, with `--json`, `origin_path`/`master_fingerprint`). Live run for the abandon seed @ bip84 mainnet:
   - `origin_path: "m/84'/0'/0'"`, `master_fingerprint: "73c5da0a"`
   - `ms1: ["ms10entrsqqqq…cj9sxraq34v7f"]`
   - `md1: ["md1fgdxlpq…", "md1fgdxlpq…", "md1fgdxlpq…"]` — **3 CHUNKED strings** (chunk-0 header decodes to version=4, chunked=1, chunk_set_id=0x434df, count=3, index=0).
   So Task 0's PRIMARY path (toolkit golden generation) is real. Vendor the 4-template output + xpub + fp + origin per the plan.

3. **The Rust-harness fallback is sound** (`cell_7_wpkh_full`-style AST via `md-codec`). Verified the fixtures verbatim: `cell_7_wpkh_full()` (`wallet_policy.rs:190-204`) = `Shared(bip84_path())`, `tree=wpkh_at_0()` (`KeyArg{index:0}`), `fingerprints Some([(0,[0xDE,0xAD,0xBE,0xEF])])`, `pubkeys Some([(0, make_xpub(0x11))])`; `make_xpub` = 32B chain-code-fill ‖ 33B compressed-G. The fallback is a faithful byte-reference.

4. **Robustness recommendation (NOT gate-blocking, M1):** the SUREST, form-independent reference is the canonical `encodePayload` BYTES, not the wrapped string. The fork's existing golden discipline already does exactly this — `TestEncodePayloadGoldens` asserts `encodePayload` bytes == `.bytes.hex` (`md/encode_test.go:195-217`), and `TestEncodeMD1StringGoldens` asserts the wrapped single string == `.phrase.txt` (`:258-275`). A `.bytes.hex`-equivalent key-bearing golden sidesteps the chunked-vs-single ambiguity entirely (the chunk header / chunk_set_id are a deterministic function of the same payload). Recommend Task 0 capture BOTH the toolkit's chunked md1 strings AND a payload-bytes reference (e.g. from a Rust `encode_payload` dump), and have Task 2.1 assert payload-byte parity as the primary and chunked-string parity as the wire-form check.

**Ruling (a): the gate is achievable. Fold the form correction (I1) — the vendored golden + the `EncodeSingleSig` output must both be the CHUNKED (`split`) form (and/or the form-independent payload bytes), never `encode_md1_string`.**

### (b) The 4 AST shapes' correctness + buildability — **CORRECT and BUILDABLE. PASS.**

All four bodies are exactly right and constructible from the unexported in-package types (`body.isBody()` is unexported `md/md.go:103`, so package `md` is the only home — correct):

- **pkh** → `node{tagPkh, keyArgBody{0}}` ✓ — decode dispatch `case tagPkK,tagPkH,tagWpkh,tagPkh: keyArgBody{index}` (`md/md.go:340-345`); encode `writeNode` keyArgBody arm (`md/encode.go:161-162`); `keyArgBody struct{index uint8}` (`md/md.go:119`). pkh canonical origin = `m/44'/0'/0'` (`md/md.go:1097-1100`).
- **wpkh** → `node{tagWpkh, keyArgBody{0}}` ✓ — same dispatch; canonical origin `m/84'/0'/0'` (`md/md.go:1101-1104`).
- **tr (key-path)** → `node{tagTr, trBody{isNums:false, keyIndex:0, tree:nil}}` ✓ — DISTINCT type `trBody{isNums bool; keyIndex uint8; tree *node}` (`md/md.go:114-118`); decode arm `md/md.go:432-457`; encode arm `md/encode.go:203-214`. Rust `tr_keypath_at_0()` = `Body::Tr{is_nums:false, key_index:0, tree:None}` (`wallet_policy.rs:152-161`). canonical origin (tree==nil) = `m/86'/0'/0'` (`md/md.go:1105-1111`). The plan's "NOT keyArgBody" call-out is exact.
- **sh-wpkh** → `node{tagSh, childrenBody{[node{tagWpkh, keyArgBody{0}}]}}` ✓ — decode `case tagSh,…: childrenBody{children:[child]}` (`md/md.go:346-351`); root allow-list permits `tagSh` (`md/md.go:848-852`: `tagSh,tagWsh,tagWpkh,tagPkh,tagTr`). Rust sh(wpkh) fixture (`wallet_policy.rs:778-794`): `Shared(bip49_path())`, `Tag::Sh` → `Body::Children([Node{Tag::Wpkh, KeyArg{index:0}}])`. **`canonicalOrigin` returns None for sh-wpkh** (the `tagSh` arm returns Some only for `sh(wsh(multi/sortedmulti))` — `md/md.go:1116-1124`), so sh-wpkh REQUIRES the explicit `pathDecl.Shared = m/49'/0'/0'` on the wire or `validateExplicitOriginRequired` rejects on DECODE (`md/md.go:1033-1066`). The plan's "must distinguish sh-wpkh from bare wpkh — add a `ScriptShWpkh` value or a wrap flag" is the right call (see (c)/P-finding below).

Confirmed by a live host probe: `encodePayload` of the wpkh+fp+origin descriptor yields 81 bytes / 644 bits; `Decode` of the single wrap succeeds with `Root=ScriptWpkh, Policy=PolicySingle, keys=1`; `split` yields 3 chunks. All four shapes are buildable and decode-faithful.

### (c) The EncodeSingleSig signature + PathComponent/ScriptKind public-API choices — **SOUND, with one REQUIRED clarification (M2, folds into the script enum).**

- **Signature** `EncodeSingleSig(chainCode [32]byte, compressedPubkey [33]byte, fp [4]byte, origin []PathComponent, script ScriptKind) ([]string, error)` — sound. Takes parsed components (GUI does base58→bytes + fp uint32→[4]byte), keeping private material out. The TLVs need `[65]byte`/`[4]byte` (`md/md.go:505-512`); the `[]string` return matches the chunked (`split`) form (I1). The xpub TLV must be `chainCode‖compressedPubkey` ordered (bytes 0..32 chain code, 32..65 pubkey) — the plan states this; `validateXpubBytes` checks bytes 32..65 on-curve at DECODE only (`md/md.go:1073-1083`); encode does NOT validate (`md/validate_test.go:71-77` proves it) → the round-trip leg (A2) is the necessary on-curve safety net. Correct.

- **P1 — `PathComponent` net-new exported type — CORRECT.** Verified: the internal type is UNEXPORTED `pathComponent struct{hardened bool; value uint32}` (`md/md.go:173-176`). The plan declares a NET-NEW exported `PathComponent{Hardened bool, Value uint32}` and maps it to the internal `pathComponent` (`origin[i] → pathComponent{hardened:c.Hardened, value:c.Value}`, wrapped in `originPath{components}` → `pathDecl{n:1, shared:&originPath}`). This is a legitimate, minimal public-API addition. The plan correctly flags (R0-M5) that `PathComponent` is the encoder's RAW component — NOT the in-band `+HardenedKeyStart` `bip32.Path` form used by expand/display (`md/expand.go:146-160`); the two must not be conflated. Depth ≤ 15 enforced by `writeOriginPath`→`errPathDepth` (`md/encode.go:90-93`); the plan should reject `len(origin) > 15` (or rely on the encoder's error — either is fine).

- **M2 (REQUIRED clarification, folds into the implementation but must be stated in the plan): the existing `ScriptKind` enum CANNOT distinguish sh-wpkh, so the plan's "reuse the already-exported `ScriptKind`" is only PARTLY right.** Verified: `ScriptKind` (`md/md.go:1163-1171`) has values `ScriptWpkh, ScriptPkh, ScriptSh, ScriptWsh, ScriptTr` — there is NO `ScriptShWpkh`, and `ScriptSh` is the value for ANY sh wrapper (the decode side uses the separate `Template.InnerWsh bool` to distinguish sh(wsh) from sh(non-wsh) — `md/md.go:1203-1210,1314-1323`; `InnerWsh` does NOT model sh(wpkh) specifically). So:
   - `ScriptWpkh`/`ScriptPkh`/`ScriptTr` map cleanly to the 3 non-sh shapes.
   - `ScriptSh` is AMBIGUOUS for an encoder input — it would not tell `EncodeSingleSig` to build `sh(wpkh)` vs `sh(wsh(...))`.
   The plan already anticipates this ("add a `ScriptShWpkh` value or a wrap flag"). **The plan MUST commit to one:** the cleanest is a NET-NEW exported `ScriptShWpkh ScriptKind` value appended to the enum (a pure addition; does not renumber the existing iota constants only if appended AFTER `ScriptTr` — appending is safe; inserting is NOT, as `rootScriptKind`/`summarize` depend on the values). Appending `ScriptShWpkh` after `ScriptTr` (value 5) is byte-safe and the right choice; the plan should say "append `ScriptShWpkh` (do not insert/renumber)". This is the only public-API ambiguity in the signature and is gate-relevant precision, hence M2 (not blocking, but must be resolved in the plan text before code).

- **P2 — emit explicit origin for all 4 — CORRECT emission policy.** The plan EMITS explicit `pathDecl.Shared` for all 4 (validator-required only for sh-wpkh). Verified this matches the TOOLKIT verbatim: `build_descriptor` uses `PathDeclPaths::Shared(origin_path)` unconditionally (`synthesize.rs:145-147`), `origin_path = template.md_origin_path(network, account)`. So emit-for-all is the byte-lock-matching policy, NOT a validator misstatement. The plan phrases it correctly (Locked line 14: "explicit origin EMITTED for all 4 … the validator only REQUIRES it for sh-wpkh"). Good — the R1 precision note P2 is honored.

**Ruling (c): the signature is sound; `PathComponent` net-new exported is the right choice; `ScriptKind` reuse is right for 3 of 4 shapes but MUST be extended with an appended `ScriptShWpkh` value (M2) — the plan already proposes this, just lock it as "append, do not insert".**

### (d) The ms1 recipe — **BYTE-EXACT CORRECT. PASS.**

`EncodeMS1(entropy) = NewSeed("ms", 0, "entr", 's', append([]byte{0x00}, entropy...))`, English/entr-only, NET-NEW. Verified first-hand AND by a live host probe:

- prefix `0x00` = entr (`codex32/mspayload.go:9` `msPrefixEntr = 0x00 // payload = [0x00][entropy]`); the `0x02` mnem variant carries the language byte (`:10`) and is the documented follow-on.
- id FIXED literal `"entr"` (`codex32/mspayload.go:6-7` doc; the construction form `NewSeed("ms",0,"entr",'s',[prefix‖entropy])` at `mspayload_test.go:54`); NOT fingerprint-derived.
- share index lowercase `'s'` (`mspayload_test.go:54`; all wire vectors `…entrs…` `mspayload_test.go:25-29`).
- `EncodeMS1` is NET-NEW — the fork ships only `DecodeMS1` (`codex32/mspayload.go:34`) and `NewSeed` (`codex32/codex32.go:279`); no `EncodeMS1`. Plan flags this (Task 1).
- **LIVE PROOF:** the throwaway probe built `NewSeed("ms",0,"entr",'s',[0x00‖zeros16])` → exactly `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f` — byte-identical to BOTH the fork's vector `entr16-zero` (`mspayload_test.go:25`) AND the toolkit's live `ms1` output for the abandon seed. `DecodeMS1(EncodeMS1(e)) == e` (prefix=0x00, lang=0, entropy round-trips). So `DecodeMS1(EncodeMS1(e))==e` is the correct acceptance and the English/entr-only lock is right.
- Length validation: `bip39` accepts 16..32 (`bip39/bip39.go:229-234`); `DecodeMS1` accepts {16,20,24,28,32} (`mspayload.go:54-58`); the plan validates the same set in `EncodeMS1`. Note: `DecodeMS1` requires `len(data) >= 2`, so the entr payload needs ≥1 entropy byte — satisfied by the 16-byte minimum. Correct.

**Ruling (d): the ms1 recipe is byte-exact and the acceptance leg is correct. No change.**

---

## IMPORTANT FINDINGS

### I1 — Lock the OUTPUT WIRE FORM to CHUNKED (`split`); drop `encodeMD1String`. A single-sig wallet-policy payload (644 bits) cannot be a single string and the toolkit never emits one.

**Plan location:** Architecture line 7 ("calls the shipped `split`/`encodeMD1String`"); Task 2 line 46 ("then call `encodeMD1String`/`split`"); Task 2 line 54 ("call the shipped `encodeMD1String`/`split`"); signature returns `([]string, error)`.

**Source:**
- `split(d)` (`md/chunk.go:121-178`) ALWAYS emits chunked strings — every output carries a 37-bit `ChunkHeader` with `Chunked:true` (`:159-160`), even when `count==1`.
- `encodeMD1String(d)` (`md/encode.go:447-461`) emits a SINGLE non-chunked string (one `String`, not `[]string`), with NO chunk header, and does NOT enforce the 320-bit single-string limit.
- `singleStringPayloadBitLimit = 64*5 = 320` (`md/chunk.go:39`). Live probe: the single-sig wpkh+fp+origin payload is **644 bits / 81 bytes** → `split` returns **3 chunks**; `encodeMD1String` returns one 145-char string the device/toolkit never uses.
- The TOOLKIT emits md1 EXCLUSIVELY via `md_codec::chunk::split` — `synthesize.rs:183,219,276,481,653` ALL call `split`; `Bundle.md1` is `Vec<String>` (`synthesize.rs:29`); `encode_md1_string` appears NOWHERE in toolkit `src/` (only in md-codec's own unit-test fixtures `wallet_policy.rs`). Live: the toolkit emitted 3 chunked md1 strings (chunk_set_id 0x434df, count=3).

**Why it matters:** to be byte-equal to the toolkit (the whole point of this gate), `EncodeSingleSig` MUST emit the `split` (chunked) form. `encodeMD1String` produces a different wire that no constellation tool emits and that routes through a different decoder. The current "split/encodeMD1String" phrasing (slash = co-equal) is ambiguous and admits the wrong product.

**Exact fix:** State unambiguously that `EncodeSingleSig` returns `split(d)` (chunked `[]string`); REMOVE `encodeMD1String` from the plan as an emission option (it is the wrong wire for single-sig). Update Architecture line 7, Task 2 lines 46/54 accordingly. (Optionally, per M1, ALSO assert payload-byte parity via `encodePayload` for a form-independent gate — recommended but not required.)

### I2 — The round-trip leg (Task 2.b / Task 0 Step 3) names the wrong decoder; with chunked output it is `DecodeChunks`/`ExpandWalletPolicyChunks` ONLY (not `Decode`).

**Plan location:** Task 0 Step 3 line 21 ("the shipped `md.DecodeChunks`/`md.Decode` + `md.ExpandWalletPolicy`"); Task 2 Step 1(b) line 52 ("`md.DecodeChunks`/`Decode`→`ExpandWalletPolicy`").

**Source:**
- `Decode(s string)` (`md/md.go:1216-1230`) REFUSES chunked input: `if syms[0]&1 == 1 { return …, ErrChunkedUnsupported }` (`:1221-1223`). It only accepts SINGLE strings.
- `DecodeChunks([]string)`/`Reassemble` (`md/expand.go:25`, `md/chunk.go:207`) REFUSE single (non-chunked) input: each chunk is parsed via `readChunkHeader`, which returns `errChunkFlagMissing` when the chunked bit is clear (`md/chunk.go:90-96`).
- `ExpandWalletPolicy(*descriptor)` (`md/expand.go:83`) takes the UNEXPORTED `*descriptor` — reachable from an in-package test, but `DecodeChunks` returns a `Template` (no `*descriptor`). To recover per-@N xpub/fp/origin from a chunk set, the leg is `ExpandWalletPolicyChunks([]string) (Template, []ExpandedKey, error)` (`md/expand.go:102-112`) or `Reassemble`→`ExpandWalletPolicy`.

**Why it matters:** since the output is chunked (I1), `Decode`/`ExpandWalletPolicy(single string)` would fail with `ErrChunkedUnsupported`; the leg must use the chunked decoders. The slash phrasing ("DecodeChunks/Decode") is wrong — they are mutually exclusive by wire form.

**Exact fix:** In Task 0 Step 3 and Task 2.b, replace "`md.DecodeChunks`/`Decode` + `md.ExpandWalletPolicy`" with "`md.DecodeChunks` (Template) + `md.ExpandWalletPolicyChunks` (per-@N xpub/fp/origin/script)" — the chunked path only. (If, per M1, the plan ALSO vendors a single-string `.phrase.txt`-style reference for a SMALL key-bearing case, `Decode` would apply there — but single-sig wallet-policy never fits single-string, so this path is moot for T6a-1.)

---

## MINOR FINDINGS (non-blocking; fold opportunistically)

- **M1 — prefer the form-independent payload-byte gate as PRIMARY.** The fork already proves encoder fidelity two ways: `encodePayload` bytes == `.bytes.hex` (`md/encode_test.go:195-217`) and the wrapped string == `.phrase.txt` (`:258-275`). For T6a-1, asserting `encodePayload` BYTE parity against a key-bearing reference is the cleanest gate (chunk-header/chunk_set_id are a deterministic function of the same payload, so byte parity ⇒ chunked-string parity). Recommend Task 2.1 assert payload-byte parity as PRIMARY and chunked-string equality as the wire-form check. (Resolves the I1 ambiguity at the test level too.)

- **M2 (see Ruling (c)) — lock the script-enum extension as "APPEND `ScriptShWpkh` after `ScriptTr`, do not insert/renumber."** Inserting would renumber the existing `ScriptKind` iota values and break `rootScriptKind`/`summarize`/the GUI #10b consumers. Appending is byte-safe.

- **M3 — `md-codec` provenance: registry v0.36.0 vs `c85cd49`.** The toolkit pins `md-codec = "0.36"` resolving to the crates.io REGISTRY crate v0.36.0 (Cargo.lock checksum-pinned), while the plan/spec cite the git SHA `c85cd49`. The local `descriptor-mnemonic` tree IS at `c85cd49` and the fork's vendored vectors were copied from it; v0.36.0 == c85cd49 for the cited lines. Non-blocking, but Task 0's README provenance should note "md-codec v0.36.0 (crates.io) == descriptor-mnemonic @ c85cd49" to avoid a future drift question.

- **M4 — the master fingerprint for the abandon seed @ m/84'/0'/0' is `73c5da0a`** (live toolkit `--json`). When Task 0 vendors `meta.json` + Task 1's "known vector" uses the abandon seed, the fp and the ms1 (`ms10entrsqqqq…cj9sxraq34v7f`) are the concrete expected values — fold them into the goldens/meta so the differential is pinned, not recomputed.

- **M5 — Task 1 "the wire begins `ms1...`" plus the `entrs` infix.** The plan's Task 1 Step 1 says "the wire begins `ms1...`" and the spec A3 says "`ms1...entrs...`". The exact prefix is `ms10entrs` (threshold 0 → `0`, id `entr`, share `s`). Minor: tighten the Task 1 assertion to check the `ms10entrs` prefix (or decode `id=="entr"`/share/`prefix=0x00` as the plan already does) rather than just `ms1`. Non-blocking.

---

## TDD ORDER / EXECUTABILITY / SCOPE — CONFIRMED

- **TDD order correct:** Task 0 (differential goldens) lands BEFORE Task 2 (`EncodeSingleSig`); each task is test-first (fail→run-fail→impl→run-pass→commit). The differential goldens are the Task 2 reference, so Task 0-before-2 is right.
- **No undefined type referenced before definition:** `PathComponent` + the appended `ScriptShWpkh` are introduced in Task 2 Step 3 alongside `EncodeSingleSig`; the Task 2 Step 1 tests reference them but that is the RED phase (compile-fail is the expected RED). Fine.
- **Round-trip safety net (Task 2.b) correct in intent:** encoder does NOT validate the pubkey on-curve; DECODE does (`validateXpubBytes` `md/md.go:1073-1083`; proven by `md/validate_test.go:62-78`). So the decode round-trip is the necessary on-curve guard. (The decoder used must be the chunked one — I2.)
- **HEADLESS-ONLY, no shipped-behavior touch — CONFIRMED.** Tasks add `codex32/msencode.go`, `md/encode_singlesig.go`, the comparator, tests, and goldens; they ADD an exported `PathComponent` + (appended) `ScriptShWpkh` + `EncodeSingleSig`/`EncodeMS1`/comparator. No GUI/program/picker. The picker-default refinement (BIP-84 default + Advanced) is correctly a T6a-2 concern, absent here (spec §3 Phase B / I-8; plan Self-review P3→T6a-2). Task 4 asserts existing md/mk/codex32/ms1 + #10a/#10b + T5 tests pass verbatim. Appending `ScriptShWpkh` (M2) keeps existing `ScriptKind` values byte-stable, so no-regression holds.
- **The comparator (Task 3) — sound.** Field set fp/xpub/path/md1-exact-string/ms1-recovered-entropy is deterministically comparable: mk1/md1 are deterministic (md1 via `split`/`encodePayload`; mk csid from bytecode SHA not RNG); comparing ms1 on RECOVERED ENTROPY (not string) is correct (operator hand-types ms1; the device re-derives — string can differ incidentally, entropy is the invariant; and since id/prefix/share are pinned, string-match would also hold, but entropy-match is the robust choice). Home ("a small `bundle`-like helper or a gui headless func") is a reasonable headless home; "scrub any entropy copy" is the right secret-hygiene posture. One nit: ensure the comparator's md1 comparison is the CHUNKED string-set (or payload bytes), consistent with I1.
- **No-regression + fuzz (Task 4) adequate:** full-suite green + `go vet`/`gofmt`; fuzz `FuzzEncodeSingleSig` (encode→`DecodeChunks`→`ExpandWalletPolicyChunks` recovers inputs; no panic — note: chunked decoder per I2), `FuzzEncodeMS1` (→`DecodeMS1` round-trip), comparator fuzz, ≥1M execs. Adequate. (Fold I2 into the FuzzEncodeSingleSig round-trip decoder name.)

---

## What GREEN requires
Fold **I1** (lock the output to the CHUNKED `split` form; drop `encodeMD1String` as an emission option) and **I2** (the round-trip leg + fuzz use the CHUNKED decoders `DecodeChunks`/`ExpandWalletPolicyChunks`, not `Decode`/`ExpandWalletPolicy(single)`). Lock **M2** in the plan text (append `ScriptShWpkh`; do not insert/renumber). The Minors (M1 payload-byte primary gate, M3 provenance note, M4 pinned fp/ms1 vectors, M5 prefix assertion) are non-blocking. The 4 AST shapes, the wallet-policy TLV + explicit-origin emission, the `EncodeSingleSig` signature + `PathComponent`, and the ms1 recipe are all source-verified correct and need no change. Re-persist the folded successor verbatim and re-dispatch R0 after the fold (folds can drift). No code before 0C/0I.
