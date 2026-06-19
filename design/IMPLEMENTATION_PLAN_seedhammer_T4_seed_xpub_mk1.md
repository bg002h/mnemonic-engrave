# T4 — seed → account xpub → engrave as mk1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`. TDD: test → red → implement → green → commit.

**Goal:** Hand-typed BIP-39 seed (SECRET) → pick a standard path → derive the account xpub (PUBLIC) → engrave it as an mk1 key card (stub `00000000` + warning). Deterministic; seed never leaves the device; only the public xpub is engraved.

**Architecture:** Phase A (headless): `mk.Encode` (the reverse of the shipped `mk.Decode` — round-trips through it) + `DeriveAccountXpub` (scrub-complete derivation). Phase B (GUI): a new top-level `program` (6 lockstep sites) → reuse seed entry → two-stage path picker → derive → stub-0 warning → encode → multi-plate engrave.

**Tech stack:** Go (host) / TinyGo. Deps in-tree (`codex32`, `bip32`, `hdkeychain`, `btcec`). Go: `/home/bcg/.local/go/bin/go` (go1.26.4).

**Spec:** `design/SPEC_seedhammer_T4_seed_xpub_mk1.md` (GREEN, `3b15251`). **Base:** fork `a4d669d`. USER DECISION: mk1 route; no-policy → warn + stub `0x00000000`.

---

## Source-of-truth facts (R0/R1-verified vs `a4d669d` + mk-codec)
- **`mk.Encode` is the exact inverse of the shipped decode** (`mk/mk.go`): compact-73 = `version(4)|parentFP(4)|chainCode(32)|compressedPubKey(33)` (no depth/child — reconstructed from path); bytecode = `hdr(1)|stub_count(1)|stubs(4×N)|[fp(4) iff hdr&0x04]|path|compact73`; hdr high-nibble version 0, `reservedMask=0x0b` zero; path = std-table indicator (the 14 paths) else `0xFE+count+LEB128`; stream = `bytecode‖SHA-256(bytecode)[0..4]`, split into ≤53-byte fragments; chunked 8-symbol header `[0, 1, csid>>15&0x1f, csid>>10&0x1f, csid>>5&0x1f, csid&0x1f, total-1, index]`; data syms = header syms ++ `bytesToFiveBit(fragment)` (MSB-first, zero-pad final); each chunk string = `"mk1" + render(dataSyms) + render(BCH checksum)`.
- **C-1 BCH-GENERATE (CRITICAL):** build the engine like `verifyMDMK` (mdmk.go:103-107) — `&engine{generator: newShortChecksum().generator (or newLongChecksum() for long), residue: unpackSyms(0, mdmkPolymodInitLo, n), target: unpackSyms(mkRegularTargetHi, mkRegularTargetLo, n) (or mkLong*)}`; `e.inputHRP("mk")`; `e.inputData(render(dataSyms))`; `e.inputTarget()`; the resulting `e.residue` (n symbols) IS the checksum. `n`=13 regular / 15 long. **Do NOT clone `codex32.NewSeed`'s checksum step** (it uses codex32's residue init `1`, not mk's `0x23181b3`) — reuse only its 5-bit-pack + string-assembly shape. `mdmkPolymodInitLo=0x23181b3`; targets `mkRegularTargetHi/Lo=0x1/0x62435f91072fa5c`, `mkLongTargetHi/Lo=0x418/0x90d7e441cbe97273` (mdmk.go:39,58-62).
- **Code selection (regular 13 / long 15)** mirrors `ValidMK`/`bch_code_for_length`: brackets on the FINAL data-part length (rendered data syms + checksum chars after `"mk1"`) — regular total ∈ [14,93], long ∈ [96,108]. Pick per chunk so the result is in range (chunk 0 typically long, trailing chunk regular). **The round-trip gate (`codex32.ValidMK(chunk)==true` AND `mk.Decode(Encode(card))==card`) is the proof** the selection is right.
- **Deterministic csid:** `top20(SHA-256(bytecode))` = `(h[0]<<12)|(h[1]<<4)|(h[2]>>4)` (uint32; decoder doesn't validate the value, only consistency). NO CSPRNG.
- **Derivation:** `bip39.MnemonicSeed(m,pass)` (PBKDF2) → `hdkeychain.NewMaster(seed,net)` → path-walk (`k.Derive(c)`) → `.Neuter()`. `(*ExtendedKey).Zero()` exists (zeros key/pubKey/chainCode/parentFP). `bip32.Fingerprint(pkey)` for master FP. Account xpub fields: `xpub.Version()`, `xpub.ParentFingerprint()`, `xpub.ChainCode()`, `xpub.ECPubKey().SerializeCompressed()`. `fillDescriptor` (gui_test.go:292-327) is the derivation template.
- **14 standard paths** (mk.go `standardPaths`): mainnet 0x01-0x07 / testnet 0x11-0x17. Build a `bip32.Path` as `bip32.Path{84|0x80000000, 0|0x80000000, 0|0x80000000}` or `bip32.ParsePath("m/84'/0'/0'")`.
- **6 program-lockstep sites (gui.go):** enum (:145), StartScreen Left/Right clamp (:1620-1636), `layoutMainPlates`+`panic("invalid page")` (:1836-1844), TWO `const ...=int(backupWallet)+1` (`:1828` npage, `:1847` npages), title switch (:1650-1654), `uiFlow` dispatch (:1488-1498). Adding a navigable program edits ALL.
- **Engrave core:** `validateMdmk(params,s)→(labels,[]Plate,err)` (gui.go:1891), `NewEngraveScreen(ctx,plate).Engrave(ctx,&engraveTheme)→bool` (gui.go:2465). `ChoiceScreen.Choose(ctx,th)→(int,bool)`. `newInputFlow`/`inputWordsFlow` (seed), `passphraseFlow`. `wipeBytes([]byte)` (slip39_polish.go:330).
- **Harness:** `runUI`/`ExtractText`/`uiContains`, `click`/`press`/`runes` (on `&ctx.Router`), `NFCReader()==nil`, alloc gate = StartScreen.Flow+DescriptorScreen.Confirm only.

---

## File manifest
- **Create** `mk/encode.go` (+ `encode_test.go`) — `mk.Encode` + helpers (compact-73 build, bytecode encode, path encode, chunk split, byte→5-bit, BCH-gen via the mk engine, deterministic csid).
- **Create** a derivation helper `gui/derive.go` (or extend `bip32`) (+ test) — `deriveAccountXpub` (scrub-complete).
- **Create** `gui/derive_xpub.go` (+ test) — the new flow (picker, warning, derive, encode, multi-plate engrave, abort).
- **Modify** `gui/gui.go` — the 6-site `program` lockstep (new `engraveXpub` program).

---

## Task 0: Worktree + baseline
- [ ] **Step 1:** `cd /scratch/code/shibboleth/seedhammer && git worktree add -b feat/seed-xpub-mk1 ../seedhammer-wt-t4 a4d669d && cd ../seedhammer-wt-t4`
- [ ] **Step 2:** `/home/bcg/.local/go/bin/go test ./mk/ ./gui/ ./bip32/` → PASS (baseline).

---

## Task 1: `mk.Encode` (headless — load-bearing)

**Files:** Create `mk/encode.go`, `mk/encode_test.go`.

- [ ] **Step 1: Write the failing round-trip test** — `mk/encode_test.go`:
```go
package mk

import (
	"testing"

	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"seedhammer.com/bip32"
	"seedhammer.com/bip39"
	"github.com/btcsuite/btcd/chaincfg/v2"
)

// deriveTestXpub: a fixed mnemonic → account xpub at path (mirrors fillDescriptor).
func deriveTestXpub(t *testing.T, path string, net *chaincfg.Params) (xpub string) {
	t.Helper()
	m := make(bip39.Mnemonic, 12)
	for j := range m { m[j] = bip39.Word(j) }
	m = m.FixChecksum()
	seed := bip39.MnemonicSeed(m, "")
	mk, err := hdkeychain.NewMaster(seed, net)
	if err != nil { t.Fatal(err) }
	p, err := bip32.ParsePath(path)
	if err != nil { t.Fatal(err) }
	acct, err := bip32.Derive(mk, p)
	if err != nil { t.Fatal(err) }
	return acct.String()
}

func TestEncodeRoundTrip(t *testing.T) {
	cases := []struct{ path, net string; params *chaincfg.Params }{
		{"m/84'/0'/0'", "mainnet", &chaincfg.MainNetParams},
		{"m/44'/0'/0'", "mainnet", &chaincfg.MainNetParams},
		{"m/48'/0'/0'/2'", "mainnet", &chaincfg.MainNetParams},
		{"m/87'/0'/0'", "mainnet", &chaincfg.MainNetParams},
		{"m/84'/1'/0'", "testnet", &chaincfg.TestNet3Params},
	}
	for _, c := range cases {
		t.Run(c.path, func(t *testing.T) {
			xpub := deriveTestXpub(t, c.path, c.params)
			card := Card{Network: c.net, Path: c.path, Fingerprint: "", Stubs: [][4]byte{{0, 0, 0, 0}}, Xpub: xpub}
			strs, err := Encode(card)
			if err != nil { t.Fatalf("Encode: %v", err) }
			if len(strs) < 2 { t.Fatalf("expected >=2 chunks, got %d", len(strs)) }
			for i, s := range strs {
				if !codex32ValidMK(s) { t.Fatalf("chunk %d fails ValidMK: %s", i, s) } // see note
			}
			got, err := Decode(strs)
			if err != nil { t.Fatalf("Decode(Encode): %v", err) }
			if got.Network != card.Network || got.Path != card.Path || got.Xpub != card.Xpub ||
				len(got.Stubs) != 1 || got.Stubs[0] != [4]byte{0, 0, 0, 0} {
				t.Fatalf("round-trip mismatch:\n got %+v\nwant %+v", got, card)
			}
			// Determinism:
			strs2, _ := Encode(card)
			for i := range strs { if strs2[i] != strs[i] { t.Fatalf("non-deterministic at chunk %d", i) } }
		})
	}
}
```
(NOTE: `ValidMK` is in package `codex32`; the test references it as `codex32.ValidMK` — add the import. The `codex32ValidMK` placeholder above → `codex32.ValidMK`.)
- [ ] **Step 2: Run — expect FAIL** (`Encode` undefined): `/home/bcg/.local/go/bin/go test ./mk/ -run TestEncodeRoundTrip 2>&1 | tail`
- [ ] **Step 3: Implement** `mk/encode.go` — invert each shipped decode step (the round-trip + per-chunk `ValidMK` are the gate):
  - `Encode(card Card) ([]string, error)`: parse `card.Xpub` via `hdkeychain.NewKeyFromString` → `version=key.Version()`, `parentFP` (4 bytes big-endian from `key.ParentFingerprint()`), `chainCode=key.ChainCode()`, `pub=key.ECPubKey().SerializeCompressed()` → `compact = version‖parentFP‖chainCode‖pub` (73 bytes; verify len). Resolve `card.Path` → `comps []uint32` (reverse `standardPaths`, or `bip32.ParsePath`). **Validate the encode invariant:** `key.Depth() == len(comps)` (if a `Depth()` accessor exists; else trust construction) AND the parsed xpub's child == `comps[last]` — else `errEncodeXpub`. Build path bytes: if `comps` matches a `standardPaths` entry → the 1-byte indicator; else `0xFE` + `byte(len)` + LEB128 each (reverse `readLEB128`). Build header byte: `0x00` if no fp, `0x04` if fp present (T4: master FP optional — include it as the 4-byte origin fp with hdr `0x04`, or omit; the spec's Card.Fingerprint drives this). Bytecode = `hdr ‖ byte(stub_count=1) ‖ stub(4) ‖ [fp(4)] ‖ pathBytes ‖ compact`.
  - `encodeChunks(bytecode)`: `csid := top20(sha256(bytecode))`; `stream := bytecode ‖ sha256(bytecode)[:4]`; split into 53-byte fragments; for each, `hdr8 := []byte{0,1, csid>>15&0x1f, csid>>10&0x1f, csid>>5&0x1f, csid&0x1f, byte(total-1), byte(i)}`; `dataSyms := append(hdr8, bytesToFiveBit(frag)...)`; `s := assembleMK1(dataSyms)`.
  - `bytesToFiveBit(b)`: MSB-first 8→5 repack, zero-pad the final partial group (inverse of `fiveBitToBytes`).
  - `assembleMK1(dataSyms)`: choose regular(13)/long(15) so `len(dataSyms)+checksumLen` lands in `ValidMK`'s bracket ([14,93] reg / [96,108] long); `cksum := mkChecksum(dataSyms, n)`; render `"mk1" + each (dataSyms‖cksum) sym via fe.rune()` (lowercase).
  - `mkChecksum(dataSyms, n)` (**C-1**): `e := &engine{generator: gen(n), residue: unpackSyms(0, mdmkPolymodInitLo, n), target: unpackSyms(tHi, tLo, n)}` (gen/targets = short vs long per n); `e.inputHRP("mk"); e.inputData(render(dataSyms)); e.inputTarget()`; return `e.residue`. (All these are package-`codex32` internals — so `Encode`'s checksum helper lives in **package `codex32`** as a new exported `MKChecksumSymbols(dataSyms []byte, long bool) []byte` (mirror of `MKDataSymbols`), and `mk.Encode` calls it. Pure-stdlib.)
  - `top20(b)`: `h := sha256.Sum256(b); return uint32(h[0])<<12 | uint32(h[1])<<4 | uint32(h[2])>>4`.
- [ ] **Step 3b: Add a golden-vector ROUND-TRIP test (R0-I1 — NOT byte-equality).** The `mk_test.go` golden vectors use arbitrary explicit chunk_set_ids, NOT a SHA-derived csid, so byte-identical re-emission is impossible (and the decoder doesn't validate the csid value). Gate golden parity on **decode→re-encode→re-decode**: for each of the 7 `mk_test.go` golden strings sets, `c1 := Decode(golden); strs := Encode(c1); for each chunk assert ValidMK; c2 := Decode(strs); assert c1 == c2`. (Covers fp-present, 3-stub, explicit-path `0xFE`, testnet, long cards.) Do NOT assert `strs == golden`.
- [ ] **Step 4: Run — expect PASS:** `/home/bcg/.local/go/bin/go test ./mk/ ./codex32/ -run 'TestEncode|TestMK|TestDecode' -v 2>&1 | tail -30`
- [ ] **Step 5: Commit** (signed+DCO+author+trailer, per the convention below).

(Commit convention, every task: `git -c commit.gpgsign=true commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "<subject>" -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"`. Stage explicit paths. If `-S` fails, STOP + report.)

---

## Task 2: `deriveAccountXpub` (scrub-complete derivation)

**Files:** Create `gui/derive.go`, `gui/derive_test.go` (or in `bip32` — plan-author's choice; `gui` keeps the scrub local).

- [ ] **Step 1: Failing test** — golden mnemonic+path → known xpub; confirm `.Neuter` (xpub, no xprv); confirm a `bip39` test-vector seed → its known account xpub:
```go
func TestDeriveAccountXpub(t *testing.T) {
	m := /* the bip39 "abandon…about" 12-word test mnemonic */
	xpub, mfp, err := deriveAccountXpub(m, "", &chaincfg.MainNetParams, mustPath(t, "m/84'/0'/0'"))
	if err != nil { t.Fatal(err) }
	if !strings.HasPrefix(xpub, "xpub") { t.Fatalf("want xpub, got %q", xpub) }
	if strings.Contains(xpub, "xprv") { t.Fatal("xprv leaked!") }
	// assert against the known BIP-84 account-0 xpub for this test seed (golden vector)
	if xpub != knownTestVectorXpub84 { t.Fatalf("xpub=%s", xpub) }
	_ = mfp
}
```
- [ ] **Step 2: Run — expect FAIL** (`deriveAccountXpub` undefined).
- [ ] **Step 3: Implement** `gui/derive.go`:
```go
// deriveAccountXpub derives the account xpub at path, scrubbing all private
// material (the seed buffer + master + every intermediate ExtendedKey). Returns
// the base58 account xpub + the master-key fingerprint. NEVER returns/serializes
// a private key (the account key is neutered).
func deriveAccountXpub(m bip39.Mnemonic, passphrase string, net *chaincfg.Params, path bip32.Path) (xpub string, masterFP uint32, err error) {
	seed := bip39.MnemonicSeed(m, passphrase)
	defer wipeBytes(seed)
	master, err := hdkeychain.NewMaster(seed, net)
	if err != nil { return "", 0, err }
	pk, err := master.ECPubKey()
	if err != nil { master.Zero(); return "", 0, err }
	masterFP = bip32.Fingerprint(pk) // capture BEFORE zeroing master
	k := master
	for _, c := range path {
		next, derr := k.Derive(c)
		k.Zero() // scrub master + each intermediate (spec §2.5; Derive returns fresh buffers, no aliasing)
		if derr != nil { return "", 0, derr }
		k = next
	}
	acct, err := k.Neuter() // public-only
	if err != nil { k.Zero(); return "", 0, err }
	// R0-C1 (CRITICAL): Neuter ALIASES k's chainCode/parentFP/pubKey by reference —
	// so serialize the xpub BEFORE zeroing k, else acct.String() reads zeroed buffers
	// and emits a silently-wrong-but-valid xpub (the WRONG key on a permanent backup).
	xpub = acct.String()
	k.Zero() // now safe — scrubs the final private account key
	return xpub, masterFP, nil
}
```
- [ ] **Step 4: Run — expect PASS.** (Source the golden xpub from a standard BIP-84 test vector for the chosen test seed.)
- [ ] **Step 5: Commit.**

---

## Task 3: New `program` (6-site lockstep)

**Files:** Modify `gui/gui.go`.

- [ ] **Step 1: Failing test** — assert the new program is navigable + titled + doesn't panic:
```go
func TestEngraveXpubProgramNavigable(t *testing.T) {
	ctx := NewContext(newPlatform())
	m := new(StartScreen)
	frame, quit := runUI(ctx, func() { m.Flow(ctx, &descriptorTheme) })
	defer quit()
	frame()
	click(&ctx.Router, Right) // navigate to the new program
	content, _ := frame()
	if !uiContains(content, "xpub") && !uiContains(content, "Account") { // the new title
		t.Errorf("new program not reachable/titled; got %q", content)
	}
}
```
- [ ] **Step 2: Run — expect FAIL** (or panic("invalid page") if a site is missed — that IS the signal).
- [ ] **Step 3: Implement** the 6 edits (ALL — missing one panics/mis-renders):
  1. enum (gui.go:145): `const ( backupWallet program = iota; engraveXpub; qaProgram )` (insert `engraveXpub` as a navigable value before `qaProgram`).
  2. StartScreen Left/Right clamp (:1620-1636): change the upper bound from `backupWallet` to `engraveXpub` (so Right cycles backupWallet↔engraveXpub; keep `qaProgram` out of the navigable range).
  3. `layoutMainPlates` (:1836-1844): add `case engraveXpub:` returning its plate image (reuse an existing asset, e.g. `assets.Hammer`, or a suitable icon).
  4. both page-count consts (:1828 `npage`, :1847 `npages`): `int(engraveXpub) + 1`.
  5. title switch (:1650-1654): add `case engraveXpub: titleTxt = "Account Xpub"` (or similar).
  6. `uiFlow` dispatch (:1488-1498): add `case engraveXpub:` → call the new `deriveXpubFlow(ctx, th)` (Task 4), `continue`.
- [ ] **Step 4: Run — expect PASS** + `go build ./...` + `TestAllocs` (the new program's steady-state draw must stay 0-alloc if it touches `StartScreen` layout — it reuses the existing plate-image path, so it does).
- [ ] **Step 5: Commit.**

---

## Task 4: `deriveXpubFlow` (GUI: picker → warning → derive → encode → multi-plate engrave)

**Files:** Create `gui/derive_xpub.go`, `gui/derive_xpub_test.go`.

- [ ] **Step 1: Failing tests** — the two-stage picker resolves a path; the stub-0 warning is shown + must be acknowledged; mk1 (not seed) is engraved. (NFCReader nil → drive via `runUI`+`click`/`runes`.)
```go
func TestDeriveXpubFlow_StubWarningShown(t *testing.T) {
	// Drive: seed entry (reuse helpers) → script-type=BIP-84 → network=mainnet →
	// (optional verify) → assert the warning screen text appears before any engrave.
	// Assert uiContains(content, "not bound") / "policy" and that proceeding requires a confirm.
}
func TestTwoStagePicker(t *testing.T) {
	// stage1 (6 script types) → stage2 (2 networks) → resolves to the expected bip32.Path.
	// Assert the 6-entry stage-1 ChoiceScreen renders all entries (no clip) — R1-M5.
}
```
- [ ] **Step 2: Run — expect FAIL.**
- [ ] **Step 3: Implement** `gui/derive_xpub.go`:
  - `deriveXpubFlow(ctx, th)`: `m, ok := newInputFlow(ctx, th)` (reuse seed entry; expect a `bip39.Mnemonic`) — actually use the same 12/24-word entry path as `backupWallet` but route to derive, NOT `engraveObjectFlow`. (Reuse `inputWordsFlow` + `emptyBIP39Mnemonic` directly, or a focused entry.) Optional `passphraseFlow`.
  - **Two-stage picker** (R0-I4/R1-M5): `pathPickerFlow(ctx, th) (bip32.Path, *chaincfg.Params, bool)` — stage 1 `ChoiceScreen{Choices: ["BIP-44 legacy","BIP-49 nested-segwit","BIP-84 native-segwit","BIP-86 taproot","BIP-48 multisig","BIP-87 multisig"]}` (6 entries — verify it renders without clip), stage 2 `ChoiceScreen{Choices:["Mainnet","Testnet"]}` → resolve to the path (the BIP-48 entry maps to `.../0'/2'` by default, or add a sub-choice for `/1'` vs `/2'`).
  - `xpub, mfp := deriveAccountXpub(m, pass, net, path)` (Task 2); scrub the mnemonic `[]Word` after (zero the slice).
  - Optional **verify display** (read-only, mirror `mk1DisplayFlow`): network / path / fingerprint / the account xpub.
  - **Stub-0 warning (mandatory, §2.4):** a screen "This card carries a placeholder policy stub (00000000) and is NOT bound to a wallet policy." + Confirm(Button3)/Back(Button1). MUST acknowledge to proceed.
  - Build `card := mk.Card{Network: netName, Path: path.String(), Fingerprint: fmt.Sprintf("%08x", mfp), Stubs: [][4]byte{{0,0,0,0}}, Xpub: xpub}`; `strs, err := mk.Encode(card)`.
  - **Multi-plate engrave** (R0-I3): for `i, s := range strs`: show "Plate i+1 of len(strs)"; `labels, plates, _ := validateMdmk(ctx.Platform.EngraverParams(), s)`; engrave the chosen/default plate via `NewEngraveScreen(ctx, plates[idx]).Engrave(ctx, &engraveTheme)`; on Back/abort mid-sequence → show "Incomplete: i of N plates; this set can't be restored — discard partials and start over", do NOT mark done.
  - NEVER call `engraveSeed`/`backup.EngraveSeed`. NEVER engrave the mnemonic.
- [ ] **Step 4: Run — expect PASS** + `go build ./...` + `go test ./gui/ ./mk/ ./codex32/` + `gofmt -l` + `TestAllocs`.
- [ ] **Step 5: Commit.**

---

## Task 5: Full verification
- [ ] **Step 1:** `/home/bcg/.local/go/bin/go test ./... && go vet ./mk/ ./codex32/ ./gui/ ./bip32/ && gofmt -l mk/ codex32/ gui/ bip32/` (empty) && `go test -count=1 -run TestAllocs ./gui/` (PASS).
- [ ] **Step 2 (CI):** TinyGo `./cmd/controller` build compiles `mk`+`codex32`+`gui`.

---

## Done criteria
- `mk.Encode` round-trips through `mk.Decode` for all standard paths (singlesig + multisig, mainnet + testnet); each chunk passes `ValidMK`; ≥2 chunks; deterministic; depth/child invariant enforced; stub `[[0,0,0,0]]`.
- `deriveAccountXpub` produces the correct golden xpub; no xprv; seed+master+intermediates scrubbed.
- New `program` navigable (all 6 sites; no panic); two-stage picker resolves the path; mandatory stub-0 warning; multi-plate engrave sequences correctly with a defined abort; seed never engraved/emitted; alloc gate intact; `backupWalletFlow` unchanged.

## Self-review (vs spec)
- §2.1 round-trip → Task 1 `TestEncodeRoundTrip`. §2.2 wire-faithful → Task 1 (invert + ValidMK gate) + C-1 BCH engine. §2.3 deterministic csid → `top20`, determinism assertion. §2.4 stub-0+warning → Task 1 stub + Task 4 unskippable warning. §2.5 SECURITY (typed seed, .Neuter, no engraveSeed, scrub seed+master+intermediates+mnemonic) → Task 2 + Task 4. §2.6 multi-plate+abort → Task 4. §2.7 picker → Task 4 two-stage. §2.8 no-regression+alloc → Tasks 3/5. §2.9 no-panic → Task 1/2 error returns. R1-M5 6-entry layout check → Task 4 `TestTwoStagePicker`.
- Type names: `mk.Encode`, `mk.Card`, `codex32.MKChecksumSymbols`, `deriveAccountXpub`, `deriveXpubFlow`, `pathPickerFlow`, `engraveXpub` (program).
- **R0 gate next:** opus-architect, materialize + build/run (the round-trip + the golden-derivation + the program-navigable/no-panic are the proofs; verify NO seed/xprv path to engrave/NFC). Fold → persist → re-dispatch until GREEN.
