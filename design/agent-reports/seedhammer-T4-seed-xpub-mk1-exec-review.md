# T4 (seed → account xpub → engrave as mk1) — whole-diff adversarial EXECUTION review

**Gate:** mandatory, non-deferrable post-implementation execution review (hard gate before merge).
**Reviewer:** opus architect (independent; did not implement).
**Diff under review:** `a4d669d..feat/t4-seed-xpub-mk1` (4 commits) in `/scratch/code/shibboleth/seedhammer-wt-t4`.
**Date:** 2026-06-19.

---

## VERDICT: GREEN

0 Critical / 0 Important. Three Minors (non-blocking), listed below. The security spine is **airtight** for every path I traced. One real-but-cosmetic interop observation (focus area 2) is classified **Minor**, with reasoning.

---

## What I ran (re-run myself; did not trust the implementer's report)

### Full test suite — PASS
`go test -count=1 ./...` → all packages `ok`. No failures. Touched packages:
- `seedhammer.com/mk` ok (incl. `TestEncodeRoundTrip` over 9 paths, `TestEncodeWithFingerprint`, `TestEncodeStubZero`, `TestEncodeRejectsBadInput`, `TestEncodeGoldenRoundTrip` over V1..V7 incl. **V5 explicit-4-comp**, **V6 3-stubs**, **V7 max-10-comp explicit path**).
- `seedhammer.com/codex32` ok (incl. `TestMKChecksumSymbolsRoundTrip` regular+long, `TestDecodeParity`, `TestCorrectRoundTrips`).
- `seedhammer.com/gui` ok (all new T4 tests below ran, none skipped):
  - `TestDeriveAccountXpub` / `…Testnet` / `…Passphrase` — PASS
  - `TestEngraveXpubProgramNavigable` — PASS
  - `TestPathPickerResolves` (bip44/84/84-testnet/**bip48**/bip87) — PASS
  - `TestPathPickerStage1NoClip` (6-entry stage at real 480×320) — PASS
  - `TestDeriveXpubFlow_StubWarningThenEngrave` (synctest; engraver must not open pre-ack) — PASS
  - `TestDeriveXpubFlowEngravesMK1NotSeed` — PASS
  - `TestMultiPlateEngravePlateTitles` (Plate 1 of N + mid-sequence abort) — PASS

### Alloc gate — PASS
`go test -run TestAllocs ./gui/...` → `gui`, `gui/op`, `gui/saver`, `gui/text`, `gui/widget` all PASS. The new navigable program did not regress the StartScreen/DescriptorScreen alloc gate.

### vet / gofmt — CLEAN (one pre-existing warning)
`go vet ./mk/... ./codex32/... ./gui/... ./bip32/...` → only `gui/op/draw_test.go:176: testing.ArtifactDir requires go1.26 or later`. **Confirmed PRE-EXISTING** (present in base `a4d669d`; `draw_test.go` is NOT in the diff). `go fmt` on all touched packages reports nothing misformatted.

### Host build — PASS
`go build ./...` (incl. `cmd/controller`) → exit 0.

### Fuzz — NO panics, NO leaks
- `FuzzEncode` (fuzzes path/fp/net/xpub/stub-count): **5.10M execs**, 0 panics. Encoded outputs also fed back through `Decode` — no panic.
- `FuzzDeriveAccountXpub` (fuzzes word list/passphrase/path/network): **110K execs**, 0 panics, and asserted no `xprv`/`tprv` ever appears in output — none did.

### Targeted probes I added (then removed; tree left clean)
- **Fragment-length sweep 1..53 bytes** → every chunk produced is `ValidMK`. The reserved `[94,95]` data-part gap is structurally unreachable; the regular/long bracket selection in `assembleMK1` is correct at both ends (1-byte tail → regular; full 53-byte fragment → dataSyms=93 → long → data-part=108 = exact long max).
- **LEB128 round-trip** for `{0,1,0x7f,0x80,0x3fff,0x80000000,0xffffffff,84|H}` → all exact.
- **Independent golden cross-check** (no scrubbing path): recomputed the abandon-about seed `5eb00bbddcf069…` (the published BIP-39 vector), master xprv `xprv9s21ZrQH143K3GJpoapnV8SFf…`, master pubkey `03d902f3…` → FP `73c5da0a` (= `knownMasterFP`), and m/84'/0'/0' xpub = `knownAccountXpub84` exactly. The golden constants are authoritative, not self-referential.
- **h-form interop** (see Minor 1).

---

## Security spine — AIRTIGHT (paths traced)

I traced every path by which seed / master xprv / intermediate xprv / passphrase / mnemonic could reach an engrave plate, NFC, QR, log, or persistent surface. None can.

1. **`mk.Encode` is public-only.** `compactFromXpub` (encode.go:101) parses the xpub via `hdkeychain.NewKeyFromString` and **rejects `key.IsPrivate()`** (encode.go:106) — an xprv input cannot be serialized. It serializes only version|parentFP|chainCode|compressedPubKey. Verified Encode never touches a private key.
2. **`deriveAccountXpub` (derive.go:19) Neuter-before-Zero ordering is correct (R0-C1).** I confirmed against the vendored btcutil `hdkeychain` (`extendedkey.go`): `Neuter()` calls `NewExtendedKey(version, k.pubKeyBytes(), k.chainCode, k.parentFP, …)` and `NewExtendedKey` stores `chainCode`/`parentFP` **by reference** (no copy); `Zero()` calls `zero(k.chainCode)`/`zero(k.parentFP)` which mutate those SAME backing arrays. So zeroing `k` before serializing `acct` would corrupt the xpub's chainCode/parentFP → a structurally-valid-but-WRONG key on a permanent steel backup. The code does `xpub = acct.String()` (line 195) **then** `k.Zero()` (line 196), with an explicit warning comment. **Correct.** (`acct` retains shared public chainCode/pubkey afterward, which is fine — all public.)
3. **Scrub coverage on ALL exit paths.**
   - `deriveAccountXpub`: `defer wipeBytes(seed)` scrubs the 64-byte PBKDF2 buffer on every return; `master.Zero()` on the `ECPubKey` error branch (line 173); the path walk `k.Zero()`s master + each intermediate after `Derive` (line 181); `k.Zero()` on the Neuter error branch (line 187) and after serialization (line 196). Every private `*ExtendedKey` (master + intermediates + pre-neuter account key) is zeroed on every internal exit path including errors.
   - `deriveXpubFlow`: the mnemonic `[]Word` is zeroed by a `defer` (lines 113–117) that runs on **every** return (the two early `ok==false` returns, and the normal return after engrave). `wipeBytes` correctly not used on `[]Word`; the explicit loop is the right scrub.
4. **No `engraveSeed` / `backup.EngraveSeed` reachable.** Grep over `derive.go`/`derive_xpub.go`: the only occurrence is in the security-spine comment. The flow engraves via `validateMdmk(params, s)` (the same public-string engrave core `mdmkFlow` uses) where `s` is an `mk1…` string. `validateMdmk` only QR/text-renders the string; no seed material crosses.
5. **Deterministic chunk_set_id (no CSPRNG).** `top20(bytecode)` = top 20 bits of `SHA-256(bytecode)`. `TestEncodeRoundTrip` asserts byte-identical strings across two Encode runs. No randomness anywhere in the encode pipeline.
6. **Passphrase residual** is a Go `string` (immutable, cannot be zeroed) — a documented, spec-acknowledged residual (§2.5 "document residuals the runtime/GC can't guarantee"; R1-M6 for `MnemonicSeed`'s internal `sentence`). Not a new defect.

---

## Wire-faithful encode vs the decoder/mk-codec (byte-for-byte)

- **C-1 BCH-init trap avoided.** `codex32.MKChecksumSymbols` (mkencode.go) builds the engine with `residue: unpackSyms(0, mdmkPolymodInitLo, n)` (POLYMOD_INIT 0x23181b3), mk1 generator (`newShortChecksum`/`newLongChecksum`), and the mk1 targets (`mkRegular*`/`mkLong*`) — identical to `verifyMDMK`, NOT `NewSeed`'s residue-init-1. The generation procedure (`inputHRP("mk")` + `inputData(data)` + `inputTarget()`, residue IS the checksum) matches `NewSeed` (codex32.go:360–372). The `TestMKChecksumSymbolsRoundTrip` and every `ValidMK` assertion in the round-trip tests prove the residue init is correct.
- **Bytecode layout** (encode.go:34): `hdr(1) | stub_count(1) | stubs(4×N) | [fp(4) iff hdr&0x04] | path | compact73` — exact inverse of `decodeBytecode`. Header version nibble 0, fp flag bit 2, reserved bits 0.
- **compact-73** (encode.go:101): `version(4) | parentFP(4) | chainCode(32) | compressedPubKey(33)`; length asserted == 73. Only `0488b21e`/`043587cf` public versions accepted. Depth/child invariant enforced: `key.Depth() == len(comps)` AND `key.ChildIndex() == comps[last]` (raw, hardened bit included), with the depth-0 `ChildIndex()==0` case handled. `TestEncodeRejectsBadInput` proves the mismatch rejection.
- **Path encode** (encode.go:164): standard-table indicator for the 14 paths (the `standardPathIndicator` reverse-lookup is deterministic — the 14 paths form a bijection, so Go map-iteration order is irrelevant), else `0xFE + count + LEB128`. `maxPathComponents`=10 cap enforced. V7 golden (max-10 explicit) round-trips.
- **Chunk split / cross-chunk integrity** (encode.go:220): stream = `bytecode ‖ SHA-256(bytecode)[0:4]`, hash at stream END — symmetric with the decoder's `reassemble` (mk.go:217–222). Split into ≤53-byte fragments. 8-symbol chunked header `version | type=chunked(1) | csid(4 syms) | total-1 | index`. `bytesToFiveBit` MSB-first with zeroed trailing pad (the decoder's `fiveBitToBytes` rejects non-zero pad — round-trip proves the pad is zeroed).
- **regular/long bracket selection** (`assembleMK1`): `long := len(dataSyms)+13 > 93`. Empirically every reachable fragment length (1..53 B → dataSyms 10..93) lands in a valid bracket; the `[94,95]` reserved gap is unreachable.
- **Edge cases probed:** fp-absent vs present (`TestEncodeWithFingerprint`), regular/long boundary (fragment sweep), max-10 path (V7), reserved gap (unreachable), ≥2-chunk guarantee (a 1-stub no-fp card = 80-byte bytecode → 84-byte stream → 2 chunks; `len(strs) >= 2` asserted).

---

## GUI integrity — coherent, no panic path

- **6-site lockstep all present and consistent** (gui.go): (1) enum `engraveXpub` between `backupWallet` and `qaProgram` (:147); (2) StartScreen Left/Right clamp uses `engraveXpub` as the bound (:1630/:1638) so nav cycles `{backupWallet, engraveXpub}` only; (3) `uiFlow` dispatch case `engraveXpub → deriveXpubFlow` (:1493); (4) title switch case `engraveXpub → "Account Xpub"` (:1656); (5) `layoutMainPlates` case `backupWallet, engraveXpub` (:1844) — no longer panics for the new program; (6) `npage`/`npages` both `int(engraveXpub)+1`=2 (:1834/:1853) so the pager renders.
- **No reachable `panic("invalid page")`.** `m.prog` is mutated ONLY by Left/Right nav, clamped to `[0, engraveXpub]`. `qaProgram` arrives solely as a `startScreenAction{prog: qaProgram}` return from the debug command `"FOREVERLAURA!"` (:1598), after which StartScreen returns without drawing — `layoutMainPlates`/`layoutMainPager`/`draw` never see `qaProgram`. `TestEngraveXpubProgramNavigable` confirms Right wraps back to Backup Wallet (qaProgram excluded from nav).
- **Two-stage picker** resolves correct `(path, net)` for all 6 script types × 2 networks. BIP-48 → `m/48h/<coin>h/0h/2h` (P2WSH), matching standard-table 0x05 (mainnet) / 0x15 (testnet). `TestPathPickerResolves` confirms `bip48 mainnet → m/48h/0h/0h/2h`. The `/1'` (P2SH) BIP-48 variant is intentionally not offered (spec §4.2).
- **Stub-0 warning is genuinely UNSKIPPABLE.** `stubZeroWarning` uses `ConfirmWarningScreen.Layout`, which returns `ConfirmYes` only when `s.confirm.Progress(ctx) == 1` (a completed HOLD of Button3); `ConfirmNo` only on Button1. `deriveXpubFlow` engraves only if `stubZeroWarning` returns `true` (== `ConfirmYes`). No bypass. `TestDeriveXpubFlow_StubWarningThenEngrave` asserts the engraver never opens before the warning is shown/acked.
- **Multi-plate sequencing** (multiPlateEngrave) iterates all N chunks in order, titled "Plate i+1 of N", never skips/duplicates. Engrave Back → re-shows the same plate's variant picker. Variant-picker Back → `abortWarning(i, total)` ("Engraved i of N… can't be restored, discard and start over") then returns — no completed-backup state recorded for a partial set (R0-I3). `done==i` is accurate (plates 0..i-1 done).
- **`xpubVerifyFlow`** sits on the success path before the warning, is read-only (no engrave/NFC/mutation), pages gap-free over the chunked xpub tail, and leaks nothing (renders only the public Card fields).

---

## TinyGo device-build proxy (real build deferred to CI — proxy HOLDS)

`tinygo` and `nix` are unavailable in this review environment (`which tinygo` → not found; `which nix` → not found). Per prior-tier policy I validated the host-compile + import-safety proxy and mark the real device build a pre-merge CI gate:
- `go build ./...` incl. `cmd/controller` → exit 0.
- **No new transitive deps:** `mk` and `codex32` are ALREADY linked into the shipped controller (the decoder `mk.Decode` is used by `gui/mk1_inspect.go`, `gui/ms1_decode.go`, etc., which the controller builds via `cmd/controller/*.go → seedhammer.com/gui`). `mk.Encode` adds only `crypto/sha256`/`encoding/hex`/`errors` — all already imported by `mk/mk.go`. `gui/derive.go` uses hdkeychain/bip32/bip39/chaincfg — all already used by `backupWalletFlow`.
- **No TinyGo-unsafe stdlib** in any new file: grep for `math/big` / `reflect` / `encoding/json` → none. Pure `uint64` BCH math; no heap-heavy additions.
- The flake builds the controller with `tinygo -target pico-plus2 -gc precise -opt 2` (flake.nix:80,102). **Action for the implementer/CI: run the actual `tinygo build` of `cmd/controller` before merge.** The proxy gives high confidence it will link, but this remains the formal CI gate.

---

## No regression (invariant 2.8)

The `gui/gui.go` diff is exactly 16 changed lines (the lockstep + dispatch). `backupWalletFlow`, `mdmkFlow`, `validateMdmk`, and `engraveSeed` bodies are byte-unchanged (grep confirmed). T1/T2/T3 decode/verify/inspect flows untouched. Alloc gate unaffected. The new flow is purely additive (own `program`).

---

## Findings

### Minor (non-blocking)

**M1 — h-form vs '-form path-string display inconsistency (cosmetic).**
`gui/derive_xpub.go:140` sets `card.Path = path.String()`, and `bip32.Path.String()` (bip32.go:20–35) emits h-form (`m/84h/0h/0h`). The decode/inspect flow's `mk1DisplayFlow` (`gui/mk1_inspect.go:96`) displays `card.Path` from `mk.Decode`, which emits '-form via `pathString` (`m/84'/0'/0'`). So the SAME device shows the SAME derivation in two notations depending on which flow you're in.
- **Why this is NOT an interop defect (Important):** the path STRING is never serialized to the card — only the path COMPONENTS go into the bytecode (standard-table indicator byte or LEB128). I verified empirically: `Encode` of an h-form `card.Path` resolves via `bip32.ParsePath` (form-agnostic, accepts both `h` and `'` — bip32.go:71) to identical bytecode, and `Decode` re-emits '-form; the round-trip at the component/bytecode level is exact. A downstream mk1 reader decodes bytecode, not the displayed string, so it is never confused. Both forms are universally accepted BIP-32 notation; a human is not misled (just sees two styles). The h-form exists only transiently in the in-memory Card and on the verify screen.
- **Evidence:** my interop probe logged `path.String()="m/84h/0h/0h"`, `decoded Path="m/84'/0'/0'"`, round-trip OK. `TestPathPickerResolves` itself asserts h-form (`wantPath:"m/84h/0h/0h"`), so the implementer is aware.
- **Fix (optional, cosmetic):** for in-product consistency, render the verify-display path in '-form to match `mk1DisplayFlow` — e.g. show `mk.Decode(strs).Path` (already '-form) on the verify screen, or normalize `card.Path` to '-form when building the Card. Pure presentation; no correctness impact.

**M2 — unmasked `byte(total-1)` / `byte(i)` in the chunked header relies on an implicit chunk-count < 32 invariant.**
`mk/encode.go:262–263` writes `byte(total - 1)` and `byte(i)` into the 8-symbol header with no `& 0x1f` mask; the render step `symRune` masks to 5 bits, so a value > 31 would silently WRAP at render time and the decoder (`syms[6]&0x1f`/`syms[7]&0x1f`) would read a wrong total/index.
- **Why harmless today:** I computed the absolute worst case under the encoder's OWN limits (255 stubs + fp + max-10-component explicit path + 73-byte compact) = 1151-byte bytecode → 1155-byte stream → **22 chunks** (`total-1`=21 ≤ 31). For T4 (stub_count=1) it is always 2 chunks. `total` cannot exceed 22, so the unmasked write is always in range and `index < total ≤ 22`. No reachable defect.
- **Fix (defensive):** mask both fields (`byte((total-1)&0x1f)`, `byte(i&0x1f)`) and/or return an error if `total > maxChunks` (32). Hardens against a future change that raises the stub limit or path size.

**M3 — `MKChecksumSymbols` swallows an `inputData` error by returning `nil`.**
`codex32/mkencode.go`: on the (per its own comment, unreachable) `inputData` failure it `return nil`, which would yield a short/empty checksum and an invalid string rather than a hard failure. Reachability: `dataSyms` are 5-bit values rendered through `fe(s).rune()` to valid bech32 runes, and the engine accepts all-lowercase data, so `inputData` cannot fail here — but a silent `nil` is a worse failure mode than a panic if the invariant were ever broken.
- **Why harmless today:** unreachable given the caller always passes 0..31 symbols; every emitted chunk is gated by `ValidMK` in the round-trip tests, which would catch a nil checksum.
- **Fix (optional): ** either `panic` on the impossible branch (matches `NewSeed`'s `panic("unreachable")` style) or have `MKChecksumSymbols` return `([]byte, error)` so `assembleMK1` can surface it. Cosmetic/robustness.

### Important / Critical
None.

---

## Bottom line

The implementation is wire-faithful to the decoder and mk-codec, deterministic, panic-free under fuzzing, and the security spine is airtight on every path (no seed/xprv/intermediate/passphrase reaches any persistent or emitted surface; the Neuter-before-Zero ordering is correct against the actual btcutil aliasing behavior). The 6-site GUI lockstep is coherent with no reachable panic, the stub-0 warning is unskippable, and multi-plate sequencing + set-level abort are correct. No regressions to existing flows or the alloc gate. The three Minors are non-blocking (M1 cosmetic, M2/M3 defensive-only on unreachable paths).

**VERDICT: GREEN.** Clear to merge once the real `tinygo build ./cmd/controller` CI gate passes (the host-compile + import-safety proxy holds; no new TinyGo-unsafe deps).
