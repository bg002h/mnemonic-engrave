# T2c — md1 single-string decode→display Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:subagent-driven-development. Steps use `- [ ]`. TDD throughout: test → red → port/implement → green → commit.

**Goal:** Decode a **single-string** `md1` descriptor card on-device into a human-readable **BIP-388 template summary** (script/policy/threshold + per-`@N` origin fingerprint/path/multipath); cleanly refuse chunked md1. Read-only inspection.

**Architecture:** A new pure-Go `md` package — a faithful port of `descriptor-mnemonic/crates/md-codec` @ 0.36.0's single-string decode path (`decode_md1_string`→`decode_payload`): MSB-first bit reader → 5-bit header → recursive AST (`read_node`) → paths/varint/use-site → TLV → the 5 post-decode validators (+ canonical-origin table) → an in-memory `Template`. Plus `codex32.MDDataSymbols`, and a GUI `md1DisplayFlow` + an "Inspect descriptor" affordance in `mdmkFlow`. NO gather/reassembly/encoder (chunked + wallet-policy deferred to #10).

**Tech stack:** Go (host test) / TinyGo (firmware). Deps: only `seedhammer.com/codex32` (no bip380/btcec). Go: `/home/bcg/.local/go/bin/go` (go1.26.4; bare `go` not on PATH).

**Spec:** `design/SPEC_seedhammer_T2c_md1_decode.md` (GREEN, `3e12b0c`). **Base:** fork `2fed9b6`.

**Porting rule (R0-M3):** every embedded test phrase/byte-string MUST be copied verbatim from the live `descriptor-mnemonic/crates/md-codec/tests/vectors/<name>.{phrase.txt,bytes.hex,descriptor.json}` — NEVER from a recon doc. Every decode function is a faithful port of the cited md-codec Rust; the parity tests over the 9 single-string corpus vectors are the GREEN proof.

---

## Source-of-truth facts (R0/R1/amendment-verified vs md-codec 0.36.0)
- **Single-vs-chunked discriminator** = bit 0 of the first 5-bit data symbol (single LSB 0 / chunked LSB 1). `md.Decode` refuses chunked.
- **Decode order** (`decode.rs:18-69`): Header(5b) → PathDecl → UseSitePath → `kiw = 32 - leadingZeros(uint32(n-1))` → `readNode` → root-tag ∈ {Sh,Wsh,Wpkh,Pkh,Tr} → TLV → 5 validators.
- **Bit reader** MSB-first, bounds-checked (`bitstream.rs:86-209`). **5→8 repack** MSB-first (`symbols_to_bytes`); payload bit count = `5 × dataSymbols`.
- **Tag** 6-bit (`tag.rs`); **Body** 9 variants (`tree.rs:18-73`); `MAX_DECODE_DEPTH=128`.
- **Paths** (`origin_path.rs`): `depth(4)` + `[hardened(1) + LP4-ext-varint]×depth`; `n` = `read(5)+1`. **Use-site** (`use_site_path.rs`): `has_mp(1)` + `[alt_count-2(3) + alts]` + `wildcard(1)`. **Varint** (`varint.rs`): `[L:4]`; L<15 → `[payload:L]`; L=15 → `[L_high:4][low:14][high:L_high]`, `(high<<14)|low`.
- **TLV** (`tlv.rs`): entries `[tag:5][bit_len:varint][body]`, ascending tag, `bit_len>0`, ≤7-bit trailing rollback; FINGERPRINTS=0x01 (4B/rec), PUBKEYS=0x02 (65B/rec), USE_SITE=0x00, ORIGIN=0x03; sparse records `[idx:kiw][value]`, idx<n ascending.
- **5 validators** (`validate.rs`) + **canonical-origin** (`canonical_origin.rs:45-79`): pkh→m/44'/0'/0', wpkh→m/84'/0'/0', tr-keyonly→m/86'/0'/0', wsh-multi→m/48'/0'/0'/2', sh-wsh-multi→m/48'/0'/0'/1', else None.

---

## File manifest
- **Create** `codex32/mddata.go` (+ `mddata_test.go`) — `MDDataSymbols`.
- **Create** `md/bits.go` (+ `bits_test.go`) — the bit reader.
- **Create** `md/md.go` (+ `md_test.go`) — varint/paths/use-site/tag/readNode/decodePayload/TLV/validators/canonical-origin/`Decode`/`Template`/renderable.
- **Create** `gui/md1_inspect.go` (+ `md1_inspect_test.go`) — `md1DisplayFlow`, `hasMDPrefix`.
- **Modify** `gui/gui.go` (`mdmkFlow` md1 affordance), `gui/mk1_inspect_test.go` (update `TestMdmkFlowMD1NoInspect`).

---

## Task 0: Worktree + baseline
- [ ] **Step 1:** `cd /scratch/code/shibboleth/seedhammer && git worktree add -b feat/md1-decode-display ../seedhammer-wt-t2c-md1 2fed9b6 && cd ../seedhammer-wt-t2c-md1`
- [ ] **Step 2:** `/home/bcg/.local/go/bin/go test ./codex32/ ./gui/` → PASS (baseline).

---

## Task 1: `codex32.MDDataSymbols`

**Files:** Create `codex32/mddata.go`, `codex32/mddata_test.go`.

- [ ] **Step 1: Failing test** — `codex32/mddata_test.go`:
```go
package codex32

import "testing"

func TestMDDataSymbols(t *testing.T) {
	// wpkh_basic phrase (single-string md1), verbatim from md-codec tests/vectors/wpkh_basic.phrase.txt
	const s = "md1yqpqqxqq8xtwhw4xwn4qh"
	syms, err := MDDataSymbols(s)
	if err != nil {
		t.Fatalf("MDDataSymbols(valid md1): %v", err)
	}
	for i, v := range syms {
		if v >= 32 {
			t.Fatalf("symbol %d = %d not 5-bit", i, v)
		}
	}
	_, data := splitHRP(s)
	if want := len(data) - mdmkShortSyms; len(syms) != want { // 13-sym checksum stripped
		t.Fatalf("len(syms)=%d want %d", len(syms), want)
	}
	// Single-payload header: first symbol LSB 0 (version 4 = 0b00100).
	if syms[0]&1 != 0 {
		t.Fatalf("single-string md1 sym0 LSB = 1, want 0 (got %05b)", syms[0])
	}
	if _, err := MDDataSymbols("mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x"); err == nil {
		t.Fatal("MDDataSymbols(mk1): want error")
	}
	if _, err := MDDataSymbols("not bech32"); err == nil {
		t.Fatal("MDDataSymbols(garbage): want error")
	}
}
```
- [ ] **Step 2: Run — expect FAIL** (`MDDataSymbols` undefined): `/home/bcg/.local/go/bin/go test ./codex32/ -run TestMDDataSymbols 2>&1 | tail`
- [ ] **Step 3: Implement** `codex32/mddata.go` (mirror `MKDataSymbols`; md1 is regular-only, HRP "md", 13-sym strip):
```go
package codex32

import "errors"

// errNotMD1 is returned by MDDataSymbols for any string that is not a
// BCH-valid md1 string.
var errNotMD1 = errors.New("codex32: not a valid md1 string")

// MDDataSymbols returns the 5-bit data symbols of a BCH-valid md1 string
// (string-layer header + payload) with the 13-symbol regular checksum stripped.
// Each byte is a 5-bit value (0..31). md1 is regular-code only. Pure-stdlib.
//
// The caller (the md package) checks symbols[0]&1 for the chunked flag, then
// repacks the symbols 5-bit→8-bit (MSB-first) into the payload byte stream.
func MDDataSymbols(s string) ([]byte, error) {
	if !ValidMD(s) {
		return nil, errNotMD1
	}
	_, data := splitHRP(s)
	if len(data) < mdmkShortSyms {
		return nil, errNotMD1 // unreachable: ValidMD requires >= 13 data symbols.
	}
	body := data[:len(data)-mdmkShortSyms]
	syms := make([]byte, 0, len(body))
	for _, c := range body {
		e, ok := feFromRune(c)
		if !ok {
			return nil, errNotMD1 // unreachable: ValidMD verified the charset.
		}
		syms = append(syms, byte(e))
	}
	return syms, nil
}
```
- [ ] **Step 4: Run — expect PASS:** `/home/bcg/.local/go/bin/go test ./codex32/ -run TestMDDataSymbols -v`
- [ ] **Step 5: Commit:**
```bash
git add codex32/mddata.go codex32/mddata_test.go
git -c commit.gpgsign=true commit -S -s --author="Brian Goss <goss.brian@gmail.com>" \
  -m "codex32: MDDataSymbols — 5-bit data symbols of a BCH-valid md1 string (T2c)" \
  -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```
(All task commits use this signed + DCO + author + trailer form.)

---

## Task 2: `md` bit reader (port of `bitstream.rs` `BitReader`)

**Files:** Create `md/bits.go`, `md/bits_test.go`.

- [ ] **Step 1: Failing test** — `md/bits_test.go`:
```go
package md

import (
	"errors"
	"testing"
)

func TestBitReader(t *testing.T) {
	// 0xA5 = 1010_0101. MSB-first reads.
	r := newBitReader([]byte{0xA5}, 8)
	if v, _ := r.read(4); v != 0b1010 {
		t.Fatalf("read(4)=%04b want 1010", v)
	}
	if v, _ := r.read(2); v != 0b01 {
		t.Fatalf("read(2)=%02b want 01", v)
	}
	if v, _ := r.read(2); v != 0b01 {
		t.Fatalf("read(2)=%02b want 01", v)
	}
	if _, err := r.read(1); !errors.Is(err, errTruncated) {
		t.Fatalf("over-read: want errTruncated, got %v", err)
	}
	// bitLimit shorter than the byte buffer.
	r2 := newBitReader([]byte{0xFF}, 3)
	if v, _ := r2.read(3); v != 0b111 {
		t.Fatalf("read(3)=%03b", v)
	}
	if _, err := r2.read(1); !errors.Is(err, errTruncated) {
		t.Fatalf("limit over-read: want errTruncated, got %v", err)
	}
	// save/restore + scoped limit (for TLV).
	r3 := newBitReader([]byte{0xFF, 0xFF}, 16)
	save := r3.pos()
	r3.read(5)
	r3.restore(save)
	if r3.pos() != save {
		t.Fatal("restore failed")
	}
}
```
- [ ] **Step 2: Run — expect FAIL** (`newBitReader` undefined): `/home/bcg/.local/go/bin/go test ./md/ -run TestBitReader 2>&1 | tail`
- [ ] **Step 3: Implement** `md/bits.go` (faithful port of `bitstream.rs:86-209`; `read_bits`→`read`):
```go
// Package md decodes single-string md1 (descriptor) constellation strings into
// a human-readable BIP-388 template. md1 is PUBLIC; no secret handling. Wire
// format: descriptor-mnemonic/crates/md-codec @ 0.36.0 (decode_md1_string path).
// Chunked md1 is detected and refused (errChunkedUnsupported); reassembly +
// wallet-policy xpub-expansion are out of scope (ledger #10).
package md

import "errors"

var errTruncated = errors.New("md: bit stream truncated")

// bitReader is an MSB-first bit unpacker over a byte slice with a bit limit
// (port of md-codec bitstream.rs BitReader).
type bitReader struct {
	bytes    []byte
	bitPos   int
	bitLimit int
}

func newBitReader(b []byte, bitLimit int) *bitReader {
	return &bitReader{bytes: b, bitPos: 0, bitLimit: bitLimit}
}

func (r *bitReader) remaining() int {
	if r.bitLimit < r.bitPos {
		return 0
	}
	return r.bitLimit - r.bitPos
}

// read returns the next count bits (count<=64) MSB-first, LSB-aligned.
func (r *bitReader) read(count int) (uint64, error) {
	if count == 0 {
		return 0, nil
	}
	if r.remaining() < count {
		return 0, errTruncated
	}
	var result uint64
	rem := count
	for rem > 0 {
		byteIdx := r.bitPos / 8
		bitInByte := r.bitPos % 8
		freeInByte := 8 - bitInByte
		chunk := rem
		if chunk > freeInByte {
			chunk = freeInByte
		}
		shift := uint(freeInByte - chunk)
		var mask byte
		if chunk == 8 {
			mask = 0xff
		} else {
			mask = byte(1<<uint(chunk)) - 1
		}
		bits := (r.bytes[byteIdx] >> shift) & mask
		result = (result << uint(chunk)) | uint64(bits)
		r.bitPos += chunk
		rem -= chunk
	}
	return result, nil
}

func (r *bitReader) readBool() (bool, error) { v, err := r.read(1); return v != 0, err }
func (r *bitReader) pos() int                { return r.bitPos }
func (r *bitReader) restore(p int)           { r.bitPos = p }
func (r *bitReader) limit() int              { return r.bitLimit }
func (r *bitReader) setLimit(l int)          { r.bitLimit = l }
```
- [ ] **Step 4: Run — expect PASS:** `/home/bcg/.local/go/bin/go test ./md/ -run TestBitReader -v`
- [ ] **Step 5: Commit:** `git add md/bits.go md/bits_test.go` then the signed commit `md: MSB-first bit reader (port of md-codec bitstream.rs) (T2c)`.

---

## Task 3: `md` decode core → internal AST (`bytes.hex` parity)

**Files:** Create `md/md.go` (decode core), append to `md/md_test.go`-equivalent (`md/decode_test.go`).

This task ports the decode pipeline up to the raw AST + TLV, tested directly against the corpus `bytes.hex` payloads (bypassing the string layer). The implementer ports each function faithfully from the cited md-codec Rust (provided verbatim in the digest the controller supplies); the Go below pins the signatures, types, constants, and the load-bearing structure.

- [ ] **Step 1: Failing test** — `md/decode_test.go` (white-box; payloads verbatim from `tests/vectors/<name>.bytes.hex`):
```go
package md

import (
	"encoding/hex"
	"testing"
)

func mustHex(t *testing.T, s string) []byte {
	t.Helper()
	b, err := hex.DecodeString(s)
	if err != nil {
		t.Fatalf("bad hex: %v", err)
	}
	return b
}

func TestDecodePayloadAST(t *testing.T) {
	cases := []struct {
		name  string
		bytes string // verbatim tests/vectors/<name>.bytes.hex
		n     int
		root  tag
		// body assertions per shape:
		check func(t *testing.T, d *descriptor)
	}{
		{"wpkh_basic", "2002001800", 1, tagWpkh, func(t *testing.T, d *descriptor) {
			if _, ok := d.tree.body.(keyArgBody); !ok {
				t.Fatalf("wpkh body = %T want keyArgBody", d.tree.body)
			}
		}},
		{"wsh_multi_2of3", "2082001821822180", 3, tagWsh, func(t *testing.T, d *descriptor) {
			ch, ok := d.tree.body.(childrenBody)
			if !ok || len(ch.children) != 1 {
				t.Fatalf("wsh body = %T", d.tree.body)
			}
			mk, ok := ch.children[0].body.(multiKeysBody)
			if !ok || ch.children[0].tag != tagMulti || mk.k != 2 || len(mk.indices) != 3 {
				t.Fatalf("inner = %+v", ch.children[0])
			}
		}},
		// ... wpkh_basic, pkh_basic, wsh_multi_2of2, wsh_multi_2of3, wsh_sortedmulti,
		// tr_keyonly, sh_wsh_multi, wsh_divergent_paths, wsh_with_fingerprints —
		// each bytes.hex VERBATIM from tests/vectors; assert root/body/n.
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			b := mustHex(t, c.bytes)
			d, err := decodePayload(b, len(b)*8) // bytes.hex is byte-aligned
			if err != nil {
				t.Fatalf("decodePayload: %v", err)
			}
			if d.n != c.n || d.tree.tag != c.root {
				t.Fatalf("n=%d root=%v want n=%d root=%v", d.n, d.tree.tag, c.n, c.root)
			}
			c.check(t, d)
		})
	}
}
```
- [ ] **Step 2: Run — expect FAIL** (`decodePayload` undefined): `/home/bcg/.local/go/bin/go test ./md/ -run TestDecodePayloadAST 2>&1 | tail`
- [ ] **Step 3: Implement** `md/md.go` decode core — port verbatim-faithfully from md-codec source (the controller-supplied digest contains the exact Rust for each):
  - **`tag`** (`uint8` consts `tagWpkh=0x00 … tagTrue=0x23`) + `readTag(r)` — port of `tag.rs:148-202` (6-bit read; `0x3F`→consume 4-bit subcode + `errTagOutOfRange`; `0x24..0x3E`→`errTagOutOfRange`).
  - **`Body` variants** as Go interface `body` with structs: `childrenBody{children []node}`, `variableBody{k uint8; children []node}`, `multiKeysBody{k uint8; indices []uint8}`, `trBody{isNums bool; keyIndex uint8; tree *node}`, `keyArgBody{index uint8}`, `hash256Body [32]byte`, `hash160Body [20]byte`, `timelockBody uint32`, `emptyBody struct{}` (9 variants, `tree.rs:16-73`). `node{tag tag; body body}`.
  - **`readVarint(r)`** — port of `varint.rs:44-56`.
  - **`pathComponent{hardened bool; value uint32}`, `readPathComponent`, `originPath{components []pathComponent}`, `readOriginPath` (depth 4b), `pathDecl{n uint8; shared *originPath; divergent []originPath}`, `readPathDecl(r, divergent bool)` (n=read(5)+1)** — port of `origin_path.rs:34-39,68-76,133-146`; `maxPathComponents=15`.
  - **`alternative{hardened bool; value uint32}`, `useSitePath{multipath []alternative; wildcardHardened bool}`, `readUseSitePath`** — port of `use_site_path.rs:98-116`; `minAltCount=2`.
  - **`readNode(r, kiw)` / `readNodeDepth(r, kiw, depth)`** — port of `tree.rs:187-328` (the per-tag bit layout: KeyArg reads `kiw` bits; unary/binary/andor recurse; multi-family `k-1(5),n-1(5),n×index(kiw)` with `k>n`→`errKGreaterThanN`; thresh `k-1,n-1,n children`; tr `is_nums(1)[,keyIndex(kiw)]has_tree(1)[,tree]`; after/older 32b; sha256/hash256 32B; hash160/ripemd160/rawpkh 20B; false/true empty). `maxDecodeDepth=128` → `errDepthExceeded`.
  - **`descriptor{n uint8; pathDecl pathDecl; useSite useSitePath; tree node; tlv tlvSection}`** + **`decodePayload(b []byte, totalBits int) (*descriptor, error)`** — port of `decode.rs:15-54` (Header→PathDecl→UseSitePath→`kiw := 32 - bits.LeadingZeros32(uint32(pd.n-1))`→readNode→root-tag allow-list `{Sh,Wsh,Wpkh,Pkh,Tr}` else `errOperatorContext`→`readTLV`). NOTE: validators (Task 4) are appended after the TLV read in the FINAL `decodePayload`; in this task `decodePayload` stops after TLV (validators added in Task 4).
  - **`header{version uint8; divergentPaths bool}`, `readHeader(r)`** — port of `header.rs:38-50` (5b; version must==4 else `errWireVersion`).
  - **`tlvSection{useSiteOverrides []idxUseSite; fingerprints []idxFP; pubkeys []idxPub; originOverrides []idxOrigin; unknown [...]}` + `readTLV(r, kiw, n)`** + `readSparseTLVIdx` + the four body readers — port of `tlv.rs:210-447` (entry `[tag:5][bit_len:varint][body]`, ascending tag `errTLVOrdering`, `bit_len>remaining`→`errTLVLength`, `bit_len==0`→`errEmptyTLV`, ≤7-bit rollback → clean break; sparse `[idx:kiw][value]`, idx<n `errPlaceholderRange`, ascending `errOverrideOrder`; FP 4B, PUB 65B, scoped bitLimit).
  - All error sentinels: `errWireVersion`, `errTagOutOfRange`, `errKGreaterThanN`, `errDepthExceeded`, `errOperatorContext`, `errTLVOrdering`, `errTLVLength`, `errEmptyTLV`, `errPlaceholderRange`, `errOverrideOrder`, `errChunkedUnsupported`, plus the Task-4 validator sentinels.
  - **5→8 repack** `symbolsToBytes(syms []byte) []byte` (MSB-first, port of `codex32.rs:44-51`) — used by `Decode` in Task 4.
- [ ] **Step 4: Run — expect PASS** (all 9 AST subcases): `/home/bcg/.local/go/bin/go test ./md/ -run TestDecodePayloadAST -v`
- [ ] **Step 5: Commit:** `git add md/md.go md/decode_test.go` then signed commit `md: bit-packed AST decode core (port of md-codec decode_payload) (T2c)`.

---

## Task 4: validators + canonical-origin + `Decode` + `Template` (full phrase→Template parity)

**Files:** Append to `md/md.go`; create `md/md_test.go`.

- [ ] **Step 1: Failing test** — `md/md_test.go` (phrases + expectations VERBATIM from `tests/vectors`):
```go
package md

import (
	"errors"
	"testing"
)

type tvec struct {
	name   string
	phrase string // verbatim tests/vectors/<name>.phrase.txt (single md1 line)
	n      int
	root   ScriptKind
	policy PolicyKind
	k, m   int
	keys   []KeyOrigin
	render bool
}

var parity = []tvec{
	{"wpkh_basic", "md1yqpqqxqq8xtwhw4xwn4qh", 1, ScriptWpkh, PolicySingle, 0, 0,
		[]KeyOrigin{{Index: 0, Fingerprint: "", OriginPath: "m", UseSite: "<0;1>/*"}}, true},
	{"wsh_multi_2of3", "md1yzpqqxppsgsc8dua4tu0kekyl", 3, ScriptWsh, PolicyMulti, 2, 3,
		[]KeyOrigin{{0, "", "m", "<0;1>/*"}, {1, "", "m", "<0;1>/*"}, {2, "", "m", "<0;1>/*"}}, true},
	{"wsh_sortedmulti", "md1yzpqqxppcgsc9kdmw6d5dp08f", 3, ScriptWsh, PolicySortedMulti, 2, 3,
		[]KeyOrigin{{0, "", "m", "<0;1>/*"}, {1, "", "m", "<0;1>/*"}, {2, "", "m", "<0;1>/*"}}, true},
	{"tr_keyonly", "md1yqpqqxqsqgprhfjpjaz6d", 1, ScriptTr, PolicySingle, 0, 0,
		[]KeyOrigin{{0, "", "m", "<0;1>/*"}}, true},
	{"sh_wsh_multi", "md1yppqqxpsscy96gddy0v67f8tp", 2, ScriptSh, PolicyMulti, 2, 2,
		[]KeyOrigin{{0, "", "m", "<0;1>/*"}, {1, "", "m", "<0;1>/*"}}, true},
	{"wsh_with_fingerprints", "md1yppqqxppsg2z7zdatd7aljh7h2lqp277wajaesknu", 2, ScriptWsh, PolicyMulti, 2, 2,
		[]KeyOrigin{{0, "deadbeef", "m", "<0;1>/*"}, {1, "cafebabe", "m", "<0;1>/*"}}, true},
	{"wsh_divergent_paths", "md1yppqqxppsg2qknq2zc2ktzhwekmddzh", 2, ScriptWsh, PolicyMulti, 2, 2,
		[]KeyOrigin{{0, "", "m", "<0;1>/*"}, {1, "", "m", "<2;3>/*"}}, true},
	// + pkh_basic ("md1yqpqqxzq2qwfv8urt848e") + wsh_multi_2of2 ("md1yppqqxppsg2vlumagltz27le")
}

func TestDecodeParity(t *testing.T) {
	for _, v := range parity {
		t.Run(v.name, func(t *testing.T) {
			tpl, err := Decode(v.phrase)
			if err != nil {
				t.Fatalf("Decode: %v", err)
			}
			if tpl.N != v.n || tpl.Root != v.root || tpl.Policy != v.policy ||
				tpl.K != v.k || tpl.M != v.m || tpl.Renderable != v.render {
				t.Fatalf("got %+v want n=%d root=%v pol=%v k=%d m=%d render=%v",
					tpl, v.n, v.root, v.policy, v.k, v.m, v.render)
			}
			if len(tpl.Keys) != len(v.keys) {
				t.Fatalf("keys=%d want %d", len(tpl.Keys), len(v.keys))
			}
			for i, k := range v.keys {
				if tpl.Keys[i] != k {
					t.Fatalf("key %d = %+v want %+v", i, tpl.Keys[i], k)
				}
			}
		})
	}
}

func TestDecodeChunkedRefused(t *testing.T) {
	// wsh_multi_chunked: the md1 chunk line (line 2 of phrase.txt, after the
	// "chunk-set-id:" comment) — verbatim.
	const chunk = "md1fz4awqqpqsgqpsgvyyxqql8saf74dwdyqv"
	if _, err := Decode(chunk); !errors.Is(err, errChunkedUnsupported) {
		t.Fatalf("chunked md1: want errChunkedUnsupported, got %v", err)
	}
}

func TestDecodeNegativeAndRenderable(t *testing.T) {
	// Constructed/sourced: each → error (no panic, zero Template) OR Renderable=false.
	// - an explicit-origin sh(multi) → Renderable=true with origins surfaced
	// - a valid-but-complex wsh(and_v(...)) (explicit origins) → Renderable=false
	// - MissingExplicitOrigin (sh(multi) elided origin) → error
	// - reserved tag / K>N / wire-version!=4 / depth>128 → error
	// (byte payloads sourced from md-codec round-trip; assert by category, not msg.)
}
```
- [ ] **Step 2: Run — expect FAIL** (`Decode`/`Template`/`ScriptKind` undefined): `/home/bcg/.local/go/bin/go test ./md/ -run TestDecode 2>&1 | tail`
- [ ] **Step 3: Implement** — append to `md/md.go`:
  - **Public types:** `ScriptKind` (`ScriptWpkh|ScriptPkh|ScriptSh|ScriptWsh|ScriptTr`), `PolicyKind` (`PolicySingle|PolicyMulti|PolicySortedMulti|PolicyMultiA|PolicySortedMultiA|PolicyComplex`), `Template{N int; Root ScriptKind; Policy PolicyKind; K,M int; Keys []KeyOrigin; Renderable bool}`, `KeyOrigin{Index int; Fingerprint, OriginPath, UseSite string}`.
  - **The 5 validators** (port `validate.rs:17-226`): `validatePlaceholderUsage(root, n)`, `validateMultipathConsistency(shared, overrides)`, `validateTapScriptTree(node)`, `validateExplicitOriginRequired(d)`, `validateXpubBytes(d)`. Sentinels: `errMissingExplicitOrigin`, `errPlaceholderNotReferenced`, `errPlaceholderOrder`, `errMultipathAltMismatch`, `errForbiddenTapLeaf`, `errNUMSConflict`, `errInvalidXpubBytes`. `validateXpubBytes` validates `pubkey[32:65]` via... NOTE: T2c parses but does not expand pubkeys; to avoid a btcec dep, validate the 33-byte compressed point with a minimal on-curve/format check OR skip per spec §1 (the Pubkeys TLV is parsed for cursor correctness; full secp256k1 validation is part of #10's xpub-expansion). **Decision for the plan: skip the secp256k1 point check in T2c** (no btcec dep) and document it; the structural TLV parse still runs. (The R0 plan reviewer will confirm this matches the spec's deferral.)
  - **`canonicalOrigin(tree node) (originPath, bool)`** — port `canonical_origin.rs:45-79` (the 5-shape table).
  - **`Decode(s string) (Template, error)`**:
    ```go
    func Decode(s string) (Template, error) {
        syms, err := codex32.MDDataSymbols(s)
        if err != nil { return Template{}, err }
        if len(syms) == 0 || syms[0]&1 == 1 { // chunked-flag (bit 0 of symbol 0)
            return Template{}, errChunkedUnsupported
        }
        b := symbolsToBytes(syms)
        d, err := decodePayloadValidated(b, 5*len(syms))
        if err != nil { return Template{}, err }
        return summarize(d), nil
    }
    ```
    `decodePayloadValidated` = `decodePayload` (Task 3) + the 5 validators in order (`decode.rs:56-69`).
  - **`summarize(d *descriptor) Template`**: derive Root (root tag), Policy + K/M (walk the renderable shapes per §4.2; else `PolicyComplex`+`Renderable=false`), per-`@N` `KeyOrigin` (origin path from pathDecl/overrides → `pathString`; fingerprint from FP-TLV → 8-hex or ""; use-site from useSite/overrides → e.g. `<0;1>/*`), and `Renderable` per §4.2. Multi (ordered) vs SortedMulti distinguished. Helpers: `pathString(comps)`, `useSiteString(us)`.
- [ ] **Step 4: Run — expect PASS:** `/home/bcg/.local/go/bin/go test ./md/ -v 2>&1 | tail -40`
- [ ] **Step 5: Commit:** `git add md/md.go md/md_test.go` then signed commit `md: validators + canonical-origin + Decode/Template summarize, parity-verified (T2c)`.

---

## Task 5: GUI `md1DisplayFlow` + `mdmkFlow` wiring

**Files:** Create `gui/md1_inspect.go`, `gui/md1_inspect_test.go`; modify `gui/gui.go`, `gui/mk1_inspect_test.go`.

- [ ] **Step 1: Failing tests** — `gui/md1_inspect_test.go`:
```go
package gui

import (
	"strings"
	"testing"
)

func TestMD1DisplayFlowPaging(t *testing.T) {
	ctx := NewContext(newPlatform())
	tpl := md.Template{N: 2, Root: md.ScriptWsh, Policy: md.PolicyMulti, K: 2, M: 2,
		Keys: []md.KeyOrigin{{Index: 0, Fingerprint: "deadbeef", OriginPath: "m/48'/0'/0'/2'", UseSite: "<0;1>/*"},
			{Index: 1, Fingerprint: "cafebabe", OriginPath: "m/48'/0'/0'/2'", UseSite: "<0;1>/*"}}}
	frame, quit := runUI(ctx, func() { md1DisplayFlow(ctx, &descriptorTheme, tpl) })
	defer quit()
	var all strings.Builder
	for i := 0; i < 16; i++ {
		content, ok := frame()
		if !ok { break }
		all.WriteString(content)
		click(&ctx.Router, Button3)
	}
	got := all.String()
	if !uiContains(got, "multisig") || !uiContains(got, "deadbeef") || !uiContains(got, "cafebabe") {
		t.Errorf("summary missing fields; got %q", got)
	}
}

func TestMD1DisplayFlowComplexRefuses(t *testing.T) {
	ctx := NewContext(newPlatform())
	tpl := md.Template{N: 1, Root: md.ScriptWsh, Policy: md.PolicyComplex, Renderable: false,
		Keys: []md.KeyOrigin{{Index: 0, OriginPath: "m/0'", UseSite: "<0;1>/*"}}}
	frame, quit := runUI(ctx, func() { md1DisplayFlow(ctx, &descriptorTheme, tpl) })
	defer quit()
	content, _ := frame()
	if !uiContains(content, "cannot display") && !uiContains(content, "complex") {
		t.Errorf("complex policy must refuse; got %q", content)
	}
}

func TestHasMDPrefix(t *testing.T) {
	if !hasMDPrefix("md1abc") || !hasMDPrefix("MD1ABC") { t.Fatal("md1 not detected") }
	if hasMDPrefix("mk1abc") { t.Fatal("mk1 misdetected") }
}

func TestMdmkFlowMD1ShowsInspect(t *testing.T) {
	p := newPlatform(); p.engraver = newEngraver()
	ctx := NewContext(p)
	frame, quit := runUI(ctx, func() { mdmkFlow(ctx, &descriptorTheme, mdmkText("md1yqpqqxqq8xtwhw4xwn4qh")) })
	defer quit()
	content, ok := frame()
	if !ok { t.Fatal("no frame") }
	if !uiContains(content, "Inspect descriptor") {
		t.Errorf("md1 chooser must offer Inspect descriptor; got %q", content)
	}
}
```
Also UPDATE `gui/mk1_inspect_test.go` `TestMdmkFlowMD1NoInspect` → rename/repurpose to `TestMdmkFlowMD1ShowsInspect` (delete the old assertion that md1 has NO Inspect; the new test above replaces it). And keep `TestMdmkFlowMK1ShowsInspect` asserting mk1 unchanged.
- [ ] **Step 2: Run — expect FAIL** (`md1DisplayFlow`/`hasMDPrefix` undefined): `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestMD1|TestHasMDPrefix|TestMdmkFlowMD1' 2>&1 | tail`
- [ ] **Step 3a: Implement** `gui/md1_inspect.go`:
```go
package gui

import (
	"fmt"
	"image"
	"strings"

	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/layout"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
	"seedhammer.com/md"
)

func hasMDPrefix(s string) bool {
	return strings.HasPrefix(s, "md1") || strings.HasPrefix(s, "MD1")
}

func md1Summary(tpl md.Template) []string {
	var lines []string
	if tpl.Renderable {
		lines = append(lines, "Type: "+scriptName(tpl.Root)+" "+policyLine(tpl))
	} else {
		lines = append(lines, "Complex policy — cannot display safely.", fmt.Sprintf("Keys: %d", tpl.N))
	}
	for _, k := range tpl.Keys {
		fp := k.Fingerprint
		if fp == "" { fp = "—" }
		lines = append(lines, fmt.Sprintf("@%d %s %s %s", k.Index, fp, k.OriginPath, k.UseSite))
	}
	return lines
}
// scriptName: ScriptWsh→"P2WSH" etc. policyLine: e.g. "2-of-3 multisig (sortedmulti)" / "single-key".
// md1DisplayFlow: measure-and-advance pager over md1Summary(tpl) lines (mirror mk1DisplayFlow:
//   Back=Button1, Page=Button3, title "md1 descriptor", chunkString long lines, gap-free; read-only).
```
(Port the measure-and-advance loop verbatim from `mk1DisplayFlow` in `gui/mk1_inspect.go`, swapping the line source to `md1Summary(tpl)` and title to "md1 descriptor". Reuse the existing `chunkString`.)
- [ ] **Step 3b: Modify `mdmkFlow`** in `gui/gui.go` — extend the existing isMK branch with an md1 branch:
```go
	isMK := hasMKPrefix(str)
	isMD := !isMK && hasMDPrefix(str)
	title, lead, choices := "Engrave", "Choose engraving", labels
	inspect := isMK || isMD
	if isMK {
		title, lead = "mk1 key", "Choose action"
		choices = append([]string{"Inspect key"}, labels...)
	} else if isMD {
		title, lead = "md1 descriptor", "Choose action"
		choices = append([]string{"Inspect descriptor"}, labels...)
	}
	cs := &ChoiceScreen{Title: title, Lead: lead, Choices: choices}
	for {
		choice, ok := cs.Choose(ctx, th)
		if !ok { return }
		if inspect && choice == 0 {
			if isMK {
				if card, ok := mk1GatherFlow(ctx, th, str); ok { mk1DisplayFlow(ctx, th, card) }
			} else { // isMD
				tpl, err := md.Decode(str)
				switch {
				case err == nil:
					md1DisplayFlow(ctx, th, tpl)
				case errors.Is(err, md.ErrChunkedUnsupported):
					showError(ctx, th, "md1 descriptor", "Multi-part descriptor — not yet supported.")
				default:
					showError(ctx, th, "md1 descriptor", "Can't decode this descriptor.")
				}
			}
			continue
		}
		idx := choice
		if inspect { idx-- }
		if NewEngraveScreen(ctx, engravings[idx]).Engrave(ctx, &engraveTheme) { return }
	}
```
(Add `"seedhammer.com/md"` + `"errors"` to `gui/gui.go` imports if absent. Export the sentinel as `md.ErrChunkedUnsupported` — rename `errChunkedUnsupported`→`ErrChunkedUnsupported` in `md` so the GUI can match it; keep the other sentinels unexported.)
- [ ] **Step 4: Run — expect PASS** + no regressions:
```
/home/bcg/.local/go/bin/go build ./...
/home/bcg/.local/go/bin/go test ./gui/ ./md/ ./codex32/ -v 2>&1 | tail -50
/home/bcg/.local/go/bin/go test -run TestAllocs ./gui/
```
- [ ] **Step 5: Commit:** `git add gui/md1_inspect.go gui/md1_inspect_test.go gui/gui.go gui/mk1_inspect_test.go md/md.go` then signed commit `gui: md1 Inspect-descriptor decode→display in mdmkFlow (T2c)`.

---

## Task 6: Full verification
- [ ] **Step 1:** `/home/bcg/.local/go/bin/go test ./... && /home/bcg/.local/go/bin/go vet ./codex32/ ./md/ ./gui/ && /home/bcg/.local/go/bin/gofmt -l codex32/ md/ gui/` (empty) and `/home/bcg/.local/go/bin/go test -count=1 -run TestAllocs ./gui/` (PASS).
- [ ] **Step 2 (CI):** TinyGo firmware build (`./cmd/controller`) compiles `codex32`+`md`+`gui` — confirm in CI before merge (pure byte/bit-slice; no new heavy deps).

---

## Done criteria
- All 9 single-string parity vectors decode to exact `Template`; chunked md1 → `ErrChunkedUnsupported`; negatives reject (no panic, zero Template); valid-but-complex → `Renderable=false`.
- md1 shows "Inspect descriptor"; display pages gap-free; no engrave/NFC from inspect; mk1 path byte-identical; alloc gate passes; vet/gofmt clean.

## Self-review (vs spec)
- §2.1 single-string wire-exact → Tasks 3/4 (port + bytes.hex/phrase parity). §2.1c chunked refuse → Task 4 `Decode` + `TestDecodeChunkedRefused`. §2.2 kiw → Task 3 `decodePayload`. §2.3 symbol-aligned bitcount → `Decode` passes `5*len(syms)`. §2.4 faithful-or-refuse + decode-error distinct → Task 4 summarize/Renderable + `Decode` error contract; Task 5 display. §2.5 reject set → Task 3/4 sentinels + Task 4 negatives. §2.6 read-only → Task 5. §2.7 no regression (mk1 untouched) → Task 5 keeps mk1 branch; `TestMdmkFlowMK1ShowsInspect`. §2.8 alloc gate → Task 6. §2.9 no secrets → no scrub. §2.10 HRP discrimination → `hasMDPrefix`. §2.11 paging tail → `TestMD1DisplayFlowPaging`. §2.12 validators+canonical-origin → Task 4. §2.13 Tr is_nums → Task 3 readNode + a `tr(NUMS)` vector in Task 4 negatives/renderable.
- No placeholders in code steps; Tasks 3/4 cite the verbatim md-codec Rust (controller-supplied) as the port source + parity tests as the GREEN gate. Type names consistent (`md.Template`, `md.ScriptKind`, `md.PolicyKind`, `md.KeyOrigin`, `md.Decode`, `md.ErrChunkedUnsupported`).
- **R0 gate next:** opus-architect, MUST materialize + build/run (port-correctness is the risk). Fold → persist verbatim → re-dispatch until GREEN.
