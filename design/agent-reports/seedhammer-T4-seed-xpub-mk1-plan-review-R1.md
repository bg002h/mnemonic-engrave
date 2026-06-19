# R1 Gate Review — IMPLEMENTATION_PLAN_seedhammer_T4_seed_xpub_mk1 (re-dispatch after fold)

**Reviewer:** opus-architect (materialize + build + run)
**Plan under review:** `design/IMPLEMENTATION_PLAN_seedhammer_T4_seed_xpub_mk1.md`
**Spec:** `SPEC_seedhammer_T4_seed_xpub_mk1.md` (GREEN at R1, `3b15251`)
**Prior review:** `seedhammer-T4-seed-xpub-mk1-plan-review-R0.md` (NOT GREEN, 1C/1I)
**Base fork:** `a4d669d` (T3 merged; throwaway worktree, removed after review; fork left clean at `a4d669d` — verified)
**Go:** go1.26.4 (`/home/bcg/.local/go/bin/go`)
**mk-codec authoritative source:** `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec` (verified every wire-format claim against this, not the plan body)
**Method:** Re-verified all wire-format facts against mk-codec Rust source + the fork's shipped `mk.Decode`/`codex32` engine. Materialized the plan's Phase-A code in the worktree (`mk.Encode`, `codex32.MKChecksumSymbols`/`SymbolRune`, and BOTH orderings of `deriveAccountXpub`), built it, and ran: the C-1 aliasing oracle, the round-trip across all 14 standard paths (mainnet+testnet, fp-present and fp-absent), the 7-golden-vector round-trip, the regular/long bracket selection, the fragment-bracket exhaustion, and a 6-entry ChoiceScreen render+height test on both the 240×240 harness and the real 480×320 device dims. Statically audited the GUI lockstep + security spine against the fork source.

---

## VERDICT: GREEN

0 Critical / 0 Important. Both prior findings CLOSED with empirical evidence. Three Minors (non-blocking) noted below.

---

## C-1 (CRITICAL from R0) — serialize-before-zero in `deriveAccountXpub` — **CLOSED**

**Evidence (empirical, `TestC1_AliasingOracle`):** I implemented both orderings and ran them on the canonical BIP-39 "abandon abandon … about" 12-word mnemonic at m/84'/0'/0' (mainnet):

- **Fold ordering (serialize BEFORE `k.Zero()`):** `xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V` — the correct, published BIP-84 account-0 xpub. ✓
- **Original plan ordering (`k.Zero()` BEFORE serialize):** `xpub6BemYiVNp19Zz8cJwY3X663dNHj5QRjN4dgPByiKNEFvKSBotr7D79u3EN7DQU8RuxoNz33yYxzCA3jyo77HJ7CtNStY17M1K9PqgTWooGf` — silently corrupted (matches the `xpub6BemY…TWooGf` prefix the R0 reviewer reported).

**Root-cause confirmed at the library source** (`btcutil/v2@v2.0.0/hdkeychain/extendedkey.go`):
- `Neuter()` (line 502) returns `NewExtendedKey(version, k.pubKeyBytes(), k.chainCode, k.parentFP, …)`.
- `NewExtendedKey` (lines 129-137) stores `chainCode`/`parentFP`/`key` **by reference — no copy**. So `acct.chainCode`/`acct.parentFP`/`acct.key` alias `k`'s backing arrays.
- `Zero()` (lines 635-638) zeroes `k.key`, `k.pubKey`, `k.chainCode`, `k.parentFP` **in place**. Zeroing before serialize corrupts the aliased public data the neutered key serializes from.

**Plan's fix verified (plan lines 162-169):** `acct, err := k.Neuter()` → check `err` first → `xpub = acct.String()` (BEFORE) → `k.Zero()` (AFTER). The plan body now carries this exact ordering, with the inline R0-C1 comment. The fixed helper produces the correct golden xpub; no `xprv` in output; `.Neuter()` confirmed public. CLOSED.

## I-1 (IMPORTANT from R0) — golden parity via round-trip, not byte-equality — **CLOSED**

**Evidence (empirical, `TestEncodeGoldenRoundTrip`):** all 7 `mk/mk_test.go` golden-vector sets pass the `decode → re-encode → re-decode → assert c1 == c2` gate, with `codex32.ValidMK` true on every re-encoded chunk. This exercises fp-present, 3-stub, explicit-path (`0xFE`+LEB128), testnet (tpub), and the long multi-chunk cards.

I independently re-confirmed the R0 reviewer's discovery that the golden vectors carry **explicit, arbitrary csids** (not the SHA-256-derived value): the decoder reads `csid` from the chunked header but `reassemble` (mk.go:192) only checks csid *consistency* across chunks, never the value — so a deterministic `top20(SHA-256(bytecode))` csid cannot byte-match the goldens, making byte-equality unsatisfiable. The plan's Task 1 Step 3b (lines 115) now mandates exactly the round-trip gate and explicitly says "Do NOT assert `strs == golden`." This is the correct, achievable gate. CLOSED.

---

## Re-verified (not assumed to still hold after the fold)

### Encoder design (`mk.Encode`) — verified faithful to mk-codec, round-trips
Every layout claim checked against mk-codec source (NOT the plan):
- **bytecode** `hdr(1)|stub_count(1,≥1)|stubs(4×N)|[fp(4) iff hdr&0x04]|path|compact73` — matches `encode.rs:55-67` and the fork's `decodeBytecode`. Header version nibble 0; fp flag `0x04`; `reservedMask=0x0b` zero.
- **compact-73** `version(4)|parentFP(4)|chainCode(32)|compressedPubkey(33)` — matches `xpub_compact.rs:30-41` and `reconstructXpub`. Depth/child reconstructed from path (not on wire).
- **path** = 14-entry std-table indicator (mainnet `0x01-0x07` / testnet `0x11-0x17`) else `0xFE`+count+LEB128 — matches `path.rs:38-98` (incl. `0x16`→`m/48'/1'/0'/1'`, present in the fork's `standardPaths`) and the fork's `decodePath`/`readLEB128`.
- **chunk split** = stream `bytecode‖SHA-256(bytecode)[0..4]`, 53-byte fragments, `total = ceil(len/53)`, header `[0,1, csid>>15&0x1f, csid>>10, csid>>5, csid&0x1f, total-1, index]` — matches `pipeline.rs:72-105` + `chunk.rs:50-93` and the fork's `parseHeaderSyms`/`reassemble`. `total-1` on wire, `chunk_index` verbatim 0-based.
- **regular(13)/long(15) selection** = mk-codec's `encode_5bit_to_string` (`bch.rs:525-544`): prefer regular if `len(dataSyms)+13 ∈ [14,93]`, else long if `+15 ∈ [96,108]`. The plan's `assembleMK1` (line 112) uses this exact rule.
- **C-1 BCH-generate**: targets `mkRegularTargetHi/Lo = 0x1/0x62435f91072fa5c`, `mkLongTargetHi/Lo = 0x418/0x90d7e441cbe97273`, `POLYMOD_INIT = 0x23181b3` — match `mdmk.go:39,58-62` AND mk-codec `consts.rs:18,21` + `bch.rs:198` byte-for-byte. The helper builds the engine like `verifyMDMK` (residue=`unpackSyms(0, POLYMOD_INIT, n)`), NOT codex32's init=1.

**Round-trip GREEN for ALL 14 standard paths** (`TestEncodeRoundTrip`): singlesig 44'/49'/84'/86', BIP-48 multisig both `/1'` and `/2'`, BIP-87 — mainnet AND testnet — each with fp-absent and fp-present. Every chunk passes `ValidMK`; ≥2 chunks; `mk.Decode(mk.Encode(card)) == card` on all fields; deterministic across runs. The depth/child encode invariant (`key.Depth()==len(comps)` AND `key.ChildIndex()==comps[last]`) rejects a mismatched xpub with a typed error, no panic (`TestEncodeRejectsBadXpub`); empty stubs rejected.

**Bracket selection (`TestBracketSelection`):** a real ~84-byte 2-chunk card → chunk 0 = **long** (data-part 108, the max), chunk 1 = **regular** (77) — exactly the spec's prediction. **Fragment-bracket exhaustion (`TestFragmentBrackets`):** every fragment byte-length 1..53 maps to exactly one valid bracket; none lands in the reserved [94,95] gap. The "regular-first-else-long" rule is total. **C-1 init trap (`TestC1_InitTrap`):** the real checksum validates; a one-char corruption fails `ValidMK` (the round-trip across all paths is the standing proof codex32's init=1 is NOT used — it would fail every `ValidMK`).

### Deterministic csid (no CSPRNG) — verified
`csid = top20(SHA-256(bytecode)) = (h[0]<<12)|(h[1]<<4)|(h[2]>>4)`. No `crypto/rand`/`math/rand` in the materialized encoder (grep-confirmed). The determinism assertion in `TestEncodeRoundTrip` (identical strings across two `Encode` calls) passes for all 14 paths. mk-codec's `fresh_chunk_set_id` (OsRng, `pipeline.rs:45`) is the path T4 deliberately does NOT take; the decoder doesn't validate the csid value, so a SHA-derived csid round-trips cleanly.

### Security spine — airtight (the single most important T4 check)
Audited against fork source:
- `validateMdmk(params, s string)` (gui.go:1891) consumes **only a string** and produces public TEXT/QR plates. The engraver (`NewEngraveScreen`, gui.go:2465; `Engrave→bool`, gui.go:2477) never sees key material. The new flow feeds it only `mk.Encode(card)` output (public mk1 chunks).
- The ONLY `.String()` on a key is on the **neutered** account key (xpub). No `xprv` is ever serialized.
- `engraveSeed` (gui.go:457) is reachable ONLY via `backupWalletFlow` (gui.go:1987), itself reached ONLY through `engraveObjectFlow`'s `case bip39.Mnemonic` (gui.go:1871). The new `deriveXpubFlow` is dispatched directly from `uiFlow` (`case engraveXpub: deriveXpubFlow(...); continue`) and `continue`s — it NEVER enters `engraveObjectFlow`, so there is NO path by which it can hand a mnemonic to `backupWalletFlow`/`engraveSeed`/`backup.EngraveSeed`.
- Scrub coverage per plan Task 2/4: `wipeBytes(seed)` via defer; master + every intermediate `*ExtendedKey` `.Zero()`'d (the in-place zero hook confirmed at `extendedkey.go:634`); mnemonic `[]Word` slice cleared after derivation. No NFC writer exists in `Platform` (only `NFCReader()`), so the seed has no emission surface.

Conclusion: there is no code path by which seed/xprv/passphrase reaches an engrave plate, NFC, or any persistent surface. ✓

### GUI integration — the 6-site program lockstep verified coherent (static)
All 6 sites exist in `gui.go` exactly as the plan enumerates (line numbers drift slightly from cited values but the structure is exact):
1. enum (`:147-150`): `backupWallet program = iota; qaProgram` → insert `engraveXpub` between (so `engraveXpub=1`, `qaProgram=2`).
2. StartScreen Right clamp (`:1633-1636`): `m.prog > backupWallet → 0` must become `> engraveXpub` (else Right can't reach the new program). Left clamp (`:1624-1626`) `< 0 → backupWallet`.
3. `layoutMainPlates` (`:1836`): `case backupWallet` only, else `panic("invalid page")` — MUST add `case engraveXpub` or the pager panics when the new plate-image is requested.
4. both page-count consts: `npage` (`:1828`) and `npages` (`:1847`), each `int(backupWallet)+1` → `int(engraveXpub)+1` (this is what makes the pager appear: `npage > 1`).
5. title switch (`:1650`): `case backupWallet` only, no `default` → add `case engraveXpub` (qaProgram stays out of nav range, so no empty-title regression).
6. `uiFlow` dispatch (`:1488-1498`): add `case engraveXpub: deriveXpubFlow(ctx, th); continue`.
Missing any of 3/4 panics or mis-renders; the plan edits all six. Task 3's `TestEngraveXpubProgramNavigable` exercises navigability + no-panic; alloc gate covers only `StartScreen.Flow`+`DescriptorScreen.Confirm` and the new program reuses the existing plate-image draw path (0-alloc preserved — I re-ran `TestAllocs`: PASS with the materialized files present).

### Two-stage path picker (R1-M5) — verified achievable on the device
- A 14-entry single `ChoiceScreen` WOULD overflow (no pagination/scroll in `ChoiceScreen.Draw`, gui.go:1411; `Up`/`Down` clamp to `[0,len-1]`). The plan's two-stage picker (6 script types → 2 networks) avoids this.
- **`TestChoiceScreen6Entries`:** a 6-entry ChoiceScreen renders all 6 entries via the real `runUI` harness and Down navigation reaches the last entry (choose returns index 5).
- **`TestChoiceScreen6Height`:** on the real device dims (480×320, `cmd/controller/platform_sh2.go:33-34` `lcdWidth=480 lcdHeight=320`), the 6-entry stacked height (168) is well within the vertical budget (≈232) — fits with ~64px to spare. See MINOR-3 for the test-harness caveat.

### Multi-plate engrave + stub-0 warning — types present; tests specified
- `Plate` (gui.go:452), `EngraveScreen`, `Engrave(ctx, th) bool` all exist for the per-chunk sequencing. Multi-plate ("Plate i of N") is net-new (only single-plate exists today) and the plan's Task 4 specifies it + the set-level abort warning ("Incomplete: i of N…discard partials"). Tested via `runUI` (NFCReader nil throughout).
- Stub-0 warning: Task 4 mandates an operator-acknowledged ("Engrave anyway") unskippable warning; `TestStubWarningUnskippable` (per the R0 reviewer's materialization) confirms Back/Cancel does not proceed. The encoder sets `stub_count=1, stubs=[[0,0,0,0]]` (structurally valid — decoder rejects only `stub_count==0`).

### No-regression / build / fmt / TinyGo
- `go build ./...` — PASS. `go test ./mk/ ./codex32/ ./gui/ ./bip39/ ./bip32/` — ALL PASS (with materialized files present). `gofmt -l` on the materialized encoder + checksum helper — clean. `go vet ./mk/ ./codex32/` — clean. `TestAllocs` — PASS.
- TinyGo controller build NOT run here (tinygo not installed). Low risk: the new code is pure-stdlib (`crypto/sha256`, `encoding/hex`) + in-tree pkgs; `codex32` stays uint64-only/no-`math/big` (confirmed by grep). Carried as MINOR-1 (plan Task 5 Step 2 already lists it for CI).

---

## Findings (all non-blocking)

### MINOR-1 — TinyGo controller build unverified in this env
CI must compile `./cmd/controller` (TinyGo) to confirm `mk.Encode` + `codex32.MKChecksumSymbols`/`SymbolRune` + the `gui` additions build for RP2350. Low risk (pure-stdlib, no new `math/big`). Plan Task 5 Step 2 already carries it. Non-blocking.

### MINOR-2 — `card.Path` cosmetic `h`-vs-`'` form (carried from R0)
`bip32.Path.String()` emits `m/84h/0h/0h`; `mk.Decode`/`pathString` emit `m/84'/0'/0'`. `mk.Encode` parses via `bip32.ParsePath` (accepts both) and matches on uint32 components, so encoding + the std-table indicator are unaffected — no round-trip break (my round-trip tests build the Card with the apostrophe form from `acct.String()`/decode and pass). Only the pre-engrave `mk1DisplayFlow` would show the `h` form if the GUI stored `path.String()`. Optionally normalize for display consistency. Non-blocking.

### MINOR-3 — the 240×240 test harness cannot faithfully assert picker no-clip
The GUI test harness (`testDisplayDim = 240`, gui_test.go:348) is a 240×240 display — smaller than the 480×320 device. At 240, the 6-entry stacked height (168) EXCEEDS the vertical budget (≈152), so the centered block would clip *on the test display*; at the device 480×320 it fits (budget ≈232). Two consequences for Task 4's `TestTwoStagePicker` (plan line 220): (a) `uiContains` finds all 6 entries even when clipped, because text extraction ignores the clip region — so an entries-present assertion is necessary but not sufficient for "no clip"; (b) a height-budget assertion driven at the default harness dims would FALSE-FAIL. The implementer should drive the picker layout test at explicit device dims (480×320) for any no-clip/fit assertion, OR rely on the device-dims height check (as I did). This is a test-authoring caveat, not a defect in the device behavior or the plan's design — the picker fits the real device. Non-blocking; recommend the plan's Task 4 note that the no-clip assertion uses 480×320 dims.

---

## Closure summary
- **C-1: CLOSED** — fold's serialize-before-zero ordering produces the correct `xpub6CatW…PC7PW6V`; plan-original ordering produces corrupted `xpub6BemY…TWooGf`; library aliasing root-cause confirmed at source.
- **I-1: CLOSED** — golden parity gated on decode→re-encode→re-decode (all 7 vectors pass); byte-equality correctly dropped (goldens use arbitrary csids).
- No new Critical or Important found. Encoder is wire-faithful to mk-codec across all 14 paths + 7 goldens; security spine airtight; lockstep coherent; picker fits the device.

**VERDICT: GREEN — implementation may proceed.**
