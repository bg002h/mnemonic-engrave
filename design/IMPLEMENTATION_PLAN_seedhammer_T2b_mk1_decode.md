# T2b — mk1 decode→display Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:subagent-driven-development (single implementer per task + two-stage review). Steps use checkbox (`- [ ]`) syntax. TDD throughout: test → run-fail → implement → run-pass → commit.

**Goal:** Decode a complete set of `mk1` chunk strings on-device and DISPLAY the account `{network, path, fingerprint, policy-stub count, xpub}` for verification before engraving — read-only inspection, no engrave/NFC mutation.

**Architecture:** Phase A = a new pure-Go `mk` package (string-layer header parse → strict 5→8 repack → cross-chunk-hash reassembly → bytecode decode → `hdkeychain` xpub reconstruction), plus a tiny `codex32.MKDataSymbols` primitive. Phase B = a GUI NFC multi-chunk gather (built on a pure, unit-tested `mk1Gatherer`) + a measure-and-advance decode-display, wired as an mk1-only "Inspect key" affordance in `mdmkFlow`.

**Tech stack:** Go (host test) / TinyGo (firmware). Deps already in `go.mod`: `github.com/btcsuite/btcd/btcec/v2`, `.../btcutil/v2/hdkeychain`. Go toolchain: `/home/bcg/.local/go/bin/go` (go1.26.4; bare `go` not on PATH).

**Spec:** `design/SPEC_seedhammer_T2b_mk1_decode.md` (GREEN at R1, `b990b90`). **Base:** fork `4d02021`.

---

## Source-of-truth facts (R1-confirmed vs `mnemonic-key/crates/mk-codec`)

- **Always chunked.** `XPUB_COMPACT_BYTES=73 > SINGLE_STRING_LONG_BYTES=56`; min real bytecode ≥80 B → every real card is ≥2 chunks. Single-string handled defensively only.
- **Bytecode header byte** (`bytecode/header.rs`): bits 7-4 = version (MUST be 0), bit 3/1/0 reserved (MUST be 0; `RESERVED_MASK=0b0000_1011`), bit 2 = fingerprint flag (`0b0000_0100`). Valid bytes: `0x00`, `0x04`.
- **Bytecode layout** (`bytecode/{encode,decode}.rs`): `header(1) | stub_count(1, ≥1) | stubs(4×N) | [origin_fp(4) iff bit2] | path(var) | xpub_compact(73)`. Trailing bytes → reject.
- **compact-73** (`bytecode/xpub_compact.rs`): `version(4) | parent_fp(4) | chain_code(32) | public_key(33)`. MAINNET `0488b21e`, TESTNET `043587cf`. depth = path length; childNum = last raw component (hardened bit included) or 0 for empty path; reject unknown version + invalid point.
- **Path** (`bytecode/path.rs`): 14 std indicators (mainnet `0x01..0x07`, testnet `0x11..0x17`, incl. `0x16`); `0xFE` explicit = count(0..=10, >10→PathTooDeep) + LEB128 u32 per component (hardened bit in high bit); LEB128 bails at shift≥35 / value>u32::MAX; other indicators → reject.
- **String-layer header** (`string_layer/header.rs`): single = 2 syms (`version + type=0x00`); chunked = 8 syms (`version + type=0x01 + chunk_set_id(4 syms, BE) + total_chunks + chunk_index`). **`total_chunks` stored value−1 (decode `+1`); `chunk_index` stored verbatim 0-based (NO `+1`).** version≠0 → reject; type∉{0,1} → reject; total==0 / >32 / index≥total → reject.
- **Reassembly** (`string_layer/chunk.rs`): all chunks share version/chunk_set_id/total_chunks, differ only in chunk_index; concat fragment BYTES in index order → stream; trailing 4 B = `SHA-256(bytecode)[0..4]`; strip → bytecode. Single-string fragment IS the bytecode (no hash).
- **Strict 5→8 repack** (`string_layer/bch.rs:78-100`): reject symbol ≥32, leftover `bits≥5`, or non-zero trailing pad bits. MUST NOT reuse `codex32.parts.data()` (zero-pads / panics).
- **Fork string layer** (`codex32/mdmk.go`): `ValidMK` validates ONE string's BCH (regular data-len `[14,93]`→13-sym checksum; long `[96,108]`→15-sym; 94/95 & out-of-range reject). No correction, no header parse, no reassembly. Checksum-strip count is derivable from the same length bracket.
- **Parity corpus**: `mk-codec/src/test_vectors/v0.1.json`, `family_token "mk-codec 0.2"`, SHA-256 pin `ebd8f34d8d52896e07e1faef995f18ffa61d42e2a048fb2a8c11e67f120d78ff` (`tests/vectors.rs:41`); all positives `decoder_correction: "clean"`.

---

## File manifest
- **Create** `codex32/mkdata.go` — `MKDataSymbols(s string) ([]byte, error)`.
- **Create** `codex32/mkdata_test.go` — primitive test.
- **Create** `mk/mk.go` — `Card`, `Header`, `ParseHeader`, `Decode`, internals.
- **Create** `mk/mk_test.go` — V1–V7 parity, header-parse (incl. chunk_index no-offset), negatives.
- **Create** `gui/mk1_inspect.go` — `mk1Gatherer` (pure), `mk1GatherFlow` (NFC screen), `mk1DisplayFlow` (measure-and-advance), `hasMKPrefix`, `chunkString`.
- **Create** `gui/mk1_inspect_test.go` — gatherer logic, display paging, `hasMKPrefix`, mdmkFlow Inspect.
- **Modify** `gui/gui.go` — `mdmkFlow` mk1 "Inspect key" affordance (md1 byte-identical).

---

## Task 0: Worktree + baseline
- [ ] **Step 1:** `cd /scratch/code/shibboleth/seedhammer && git worktree add -b feat/mk1-decode-display ../seedhammer-wt-t2b-mk1 4d02021 && cd ../seedhammer-wt-t2b-mk1`
- [ ] **Step 2:** `/home/bcg/.local/go/bin/go test ./codex32/ ./gui/ ./bip380/` → PASS (clean baseline).

---

## Task 1: `codex32.MKDataSymbols` (the 5-bit data-symbol primitive)

**Files:** Create `codex32/mkdata.go`, `codex32/mkdata_test.go`.

- [ ] **Step 1: Write the failing test** — `codex32/mkdata_test.go`:
```go
package codex32

import "testing"

func TestMKDataSymbols(t *testing.T) {
	// V1 chunk 1 (the shorter regular-code chunk) — BCH-valid mk1.
	const s = "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x"
	syms, err := MKDataSymbols(s)
	if err != nil {
		t.Fatalf("MKDataSymbols(valid mk1): %v", err)
	}
	// Every symbol is a 5-bit value.
	for i, v := range syms {
		if v >= 32 {
			t.Fatalf("symbol %d = %d not in 0..31", i, v)
		}
	}
	// Data part minus the stripped 13-symbol regular checksum.
	_, data := splitHRP(s)
	if want := len(data) - mdmkShortSyms; len(syms) != want {
		t.Fatalf("len(syms) = %d, want %d", len(syms), want)
	}
	// First two symbols are the string-layer header: version 0, type 0x01 (chunked).
	if syms[0] != 0 || syms[1] != 0x01 {
		t.Fatalf("header syms = %d,%d; want 0,1", syms[0], syms[1])
	}
	// Non-mk1 input → error.
	if _, err := MKDataSymbols("ms10testsxxxxxxxxxxxxxxxxxxxxxxxx"); err == nil {
		t.Fatal("MKDataSymbols(non-mk1): want error, got nil")
	}
	if _, err := MKDataSymbols("not a bech32 string"); err == nil {
		t.Fatal("MKDataSymbols(garbage): want error, got nil")
	}
}
```
- [ ] **Step 2: Run — expect FAIL** (`MKDataSymbols` undefined): `/home/bcg/.local/go/bin/go test ./codex32/ -run TestMKDataSymbols 2>&1 | tail`
- [ ] **Step 3: Implement** `codex32/mkdata.go`:
```go
package codex32

import "errors"

// errNotMK1 is returned by MKDataSymbols for any string that is not a
// BCH-valid mk1 string.
var errNotMK1 = errors.New("codex32: not a valid mk1 string")

// MKDataSymbols returns the 5-bit data symbols of a BCH-valid mk1 string —
// the string-layer header symbols followed by the bytes_to_5bit-encoded
// fragment — with the BCH checksum (13 regular / 15 long) stripped. Each
// returned byte is a 5-bit value (0..31). It errors if s is not a BCH-valid
// mk1 string. Pure-stdlib; no key-derivation deps.
//
// Callers (the mk package) parse the string-layer header off the front and
// repack the remaining fragment symbols 5-bit→8-bit with strict padding checks.
func MKDataSymbols(s string) ([]byte, error) {
	if !ValidMK(s) {
		return nil, errNotMK1
	}
	_, data := splitHRP(s)
	// Checksum-symbol count by the same data-part length bracket ValidMK uses.
	var checksum int
	switch n := len(data); {
	case n >= mkRegularMinLen && n <= mkRegularMaxLen:
		checksum = mdmkShortSyms
	case n >= mkLongMinLen && n <= mkLongMaxLen:
		checksum = mdmkLongSyms
	default:
		return nil, errNotMK1 // unreachable: ValidMK already rejected these lengths.
	}
	body := data[:len(data)-checksum]
	syms := make([]byte, 0, len(body))
	for _, c := range body {
		e, ok := feFromRune(c)
		if !ok {
			return nil, errNotMK1 // unreachable: ValidMK already verified the charset.
		}
		syms = append(syms, byte(e))
	}
	return syms, nil
}
```
- [ ] **Step 4: Run — expect PASS:** `/home/bcg/.local/go/bin/go test ./codex32/ -run TestMKDataSymbols -v`
- [ ] **Step 5: Commit** (signed + DCO):
```bash
git add codex32/mkdata.go codex32/mkdata_test.go
git -c commit.gpgsign=true commit -S -s -m "codex32: MKDataSymbols — 5-bit data symbols of a BCH-valid mk1 string (T2b)"
```
(All commits: author `Brian Goss <goss.brian@gmail.com>`, trailer `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.)

---

## Task 2: `mk` package — `Header`/`ParseHeader` + strict `fiveBitToBytes`

**Files:** Create `mk/mk.go`, `mk/mk_test.go`.

- [ ] **Step 1: Write the failing test** — `mk/mk_test.go`:
```go
package mk

import (
	"errors"
	"testing"
)

func TestParseHeader(t *testing.T) {
	// V1 chunk 0 (chunked, index 0 of 2) and chunk 1 (index 1 of 2).
	const c0 = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf"
	const c1 = "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x"
	h0, err := ParseHeader(c0)
	if err != nil {
		t.Fatalf("ParseHeader(c0): %v", err)
	}
	if !h0.Chunked || h0.TotalChunks != 2 || h0.ChunkIndex != 0 {
		t.Fatalf("c0 header = %+v; want chunked total=2 index=0", h0)
	}
	h1, err := ParseHeader(c1)
	if err != nil {
		t.Fatalf("ParseHeader(c1): %v", err)
	}
	// R0-C1 guard: chunk_index is 0-based verbatim (NOT value-1) — chunk 1 is index 1.
	if !h1.Chunked || h1.TotalChunks != 2 || h1.ChunkIndex != 1 {
		t.Fatalf("c1 header = %+v; want chunked total=2 index=1", h1)
	}
	// Both chunks share chunk_set_id.
	if h0.ChunkSetID != h1.ChunkSetID {
		t.Fatalf("chunk_set_id mismatch: %d vs %d", h0.ChunkSetID, h1.ChunkSetID)
	}
}

func TestFiveBitToBytes(t *testing.T) {
	// 8 zero symbols = 40 bits = 5 bytes, zero padding → ok.
	out, err := fiveBitToBytes([]byte{0, 0, 0, 0, 0, 0, 0, 0})
	if err != nil || len(out) != 5 {
		t.Fatalf("zero pad: out=%v err=%v", out, err)
	}
	// A symbol >= 32 → reject.
	if _, err := fiveBitToBytes([]byte{0, 32}); !errors.Is(err, errMalformedPadding) {
		t.Fatalf("symbol>=32: want errMalformedPadding, got %v", err)
	}
	// Non-zero trailing pad bits → reject (one symbol = 5 bits, all leftover, value 1).
	if _, err := fiveBitToBytes([]byte{1}); !errors.Is(err, errMalformedPadding) {
		t.Fatalf("nonzero pad: want errMalformedPadding, got %v", err)
	}
}
```
- [ ] **Step 2: Run — expect FAIL** (package `mk` does not exist): `/home/bcg/.local/go/bin/go test ./mk/ -run 'TestParseHeader|TestFiveBitToBytes' 2>&1 | tail`
- [ ] **Step 3: Implement the header + repack half of** `mk/mk.go`:
```go
// Package mk decodes mk1 (account-xpub) constellation strings into the
// account metadata they carry: network, derivation path, origin fingerprint,
// policy-id stubs, and the BIP-32 account xpub. mk1 is PUBLIC; this package
// performs no secret handling. Wire format: mnemonic-key/crates/mk-codec
// (family_token "mk-codec 0.2").
package mk

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"strings"

	"github.com/btcsuite/btcd/btcec/v2"
	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"seedhammer.com/codex32"
)

// Decode/reassembly error sentinels.
var (
	errEmptyInput             = errors.New("mk: empty input")
	errUnexpectedEnd          = errors.New("mk: unexpected end of data")
	errTrailingBytes          = errors.New("mk: trailing bytes after xpub")
	errReservedBits           = errors.New("mk: reserved header bits set")
	errStubCount              = errors.New("mk: stub_count must be >= 1")
	errMalformedPadding       = errors.New("mk: malformed payload padding")
	errChunkedHeaderMalformed = errors.New("mk: chunked header malformed")
	errMixedHeaderTypes       = errors.New("mk: mixed header types in chunk set")
	errChunkSetIDMismatch     = errors.New("mk: chunk_set_id mismatch")
	errDuplicateChunk         = errors.New("mk: duplicate chunk index")
	errCrossChunkHash         = errors.New("mk: cross-chunk integrity hash mismatch")
	errPathTooDeep            = errors.New("mk: path too deep")
	errPathComponent          = errors.New("mk: invalid path component")
)

const (
	mkVersionV01      = 0x00
	typeSingle        = 0x00
	typeChunked       = 0x01
	singleHeaderSyms  = 2
	chunkedHeaderSyms = 8
	maxChunks         = 32
)

// Header is a parsed string-layer header for one mk1 string.
type Header struct {
	Chunked     bool
	ChunkSetID  uint32
	TotalChunks int // 1 for single-string; >=2 in practice.
	ChunkIndex  int // 0-based.
}

// ParseHeader extracts the string-layer header from one BCH-valid mk1 string.
func ParseHeader(s string) (Header, error) {
	syms, err := codex32.MKDataSymbols(s)
	if err != nil {
		return Header{}, err
	}
	h, _, err := parseHeaderSyms(syms)
	return h, err
}

// parseHeaderSyms reads the string-layer header off the front of syms and
// returns it plus the number of symbols consumed (2 single / 8 chunked).
func parseHeaderSyms(syms []byte) (Header, int, error) {
	if len(syms) < singleHeaderSyms {
		return Header{}, 0, errUnexpectedEnd
	}
	if version := syms[0] & 0x1f; version != mkVersionV01 {
		return Header{}, 0, fmt.Errorf("mk: unsupported version: 0x%02x", version)
	}
	switch syms[1] & 0x1f {
	case typeSingle:
		return Header{Chunked: false, TotalChunks: 1, ChunkIndex: 0}, singleHeaderSyms, nil
	case typeChunked:
		if len(syms) < chunkedHeaderSyms {
			return Header{}, 0, errUnexpectedEnd
		}
		csid := uint32(syms[2]&0x1f)<<15 | uint32(syms[3]&0x1f)<<10 |
			uint32(syms[4]&0x1f)<<5 | uint32(syms[5]&0x1f)
		total := int(syms[6]&0x1f) + 1 // value-1 on the wire.
		index := int(syms[7] & 0x1f)   // verbatim, 0-based — NOT value-1 (R0-C1).
		if total > maxChunks || index >= total {
			return Header{}, 0, errChunkedHeaderMalformed
		}
		return Header{Chunked: true, ChunkSetID: csid, TotalChunks: total, ChunkIndex: index}, chunkedHeaderSyms, nil
	default:
		return Header{}, 0, fmt.Errorf("mk: unsupported card type: 0x%02x", syms[1]&0x1f)
	}
}

// fiveBitToBytes repacks 5-bit symbols into bytes, rejecting any symbol >= 32,
// a leftover group of >= 5 bits, or non-zero trailing padding bits (mk-codec
// string_layer/bch.rs:78-100). Unlike codex32's parts.data() it never panics
// and never silently drops a partial byte.
func fiveBitToBytes(syms []byte) ([]byte, error) {
	var acc uint32
	var bits uint
	out := make([]byte, 0, len(syms)*5/8)
	for _, v := range syms {
		if v >= 32 {
			return nil, errMalformedPadding
		}
		acc = acc<<5 | uint32(v)
		bits += 5
		if bits >= 8 {
			bits -= 8
			out = append(out, byte(acc>>bits&0xff))
		}
	}
	if bits >= 5 {
		return nil, errMalformedPadding
	}
	if acc&(1<<bits-1) != 0 {
		return nil, errMalformedPadding
	}
	return out, nil
}
```
- [ ] **Step 4: Run — expect PASS:** `/home/bcg/.local/go/bin/go test ./mk/ -run 'TestParseHeader|TestFiveBitToBytes' -v`
- [ ] **Step 5: Commit:**
```bash
git add mk/mk.go mk/mk_test.go
git -c commit.gpgsign=true commit -S -s -m "mk: string-layer header parse + strict fiveBitToBytes (T2b)"
```

---

## Task 3: `mk.Decode` — reassembly + bytecode decode + xpub reconstruction

**Files:** Modify `mk/mk.go` (append), `mk/mk_test.go` (append).

- [ ] **Step 1: Write the failing tests** — append to `mk/mk_test.go`:
```go
type vec struct {
	name    string
	strings []string
	network string
	path    string
	fp      string
	stubs   []string
	xpub    string
}

var parityVectors = []vec{
	{
		name: "V1_bip48_mainnet_1_stub_with_fp",
		strings: []string{
			"mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf",
			"mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x",
		},
		network: "mainnet", path: "m/48'/0'/0'/2'", fp: "aabbccdd", stubs: []string{"11223344"},
		xpub: "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc",
	},
	{
		name: "V2_bip84_mainnet_1_stub_with_fp",
		strings: []string{
			"mk1qpydzkpqqsqupllwqr02m0h0qvzg3vs7zqsrqq4g4z52329g4z52329g4z52329g4z52329g4z52329g4z52329g4qpy6m8lr3sdrxkguwax",
			"mk1qpydzkppfdkdzdssxt9fh54wh8vsp2jdghv74kq2e9prxaxy2xnj2ng8vm68nf54c0vrdlfrgjzpd",
		},
		network: "mainnet", path: "m/84'/0'/0'", fp: "deadbeef", stubs: []string{"c0ffee00"},
		xpub: "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a",
	},
	{
		name: "V3_bip48_testnet_1_stub_with_fp",
		strings: []string{
			"mk1qpx3t8pqqsqh0zye4ggzqvzqz5zrtp70zqsrqqaf4x56n2df4x56n2df4x56n2df4x56n2df4x56n2df4x56n2df4yp9xx3y0h0ccw664dfd",
			"mk1qpx3t8pprlnqdqf52q7jwgcnxgnuseav37nvs0zn06dyfs79hk7uk8lrxlyw57x7v7rzx74tlflqh",
		},
		network: "testnet", path: "m/48'/1'/0'/2'", fp: "10203040", stubs: []string{"778899aa"},
		xpub: "tpubDE2QenmnfFWFjr6TXWBdoZken4gKkeo3W3iCQjW64pqrtbVAP9DDmGhMRnnwwtgey511kwptHzGF5JKrrHzJJWB3ZAy4AYubz369CSz2dhS",
	},
	{
		name: "V4_bip84_mainnet_1_stub_no_fp",
		strings: []string{
			"mk1qpg4ncpqqqq6hn00qypsfz9jrcgzqvqy46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46hqx3380xk55vxz9s95rk7jsdyt",
			"mk1qpg4ncpp45u4z3s5w5d8zzzl9ugwr3a9j0jwqv80kku8y889tv9uttaemyjd5u8sp67lj8p",
		},
		network: "mainnet", path: "m/84'/0'/0'", fp: "", stubs: []string{"abcdef01"},
		xpub: "xpub6BmeGmSNQzwjso6raQ8ea1aioo7PfaivP5sPryaBZT57AjX3eYRGTyc2T8stCLcQKnA4Pw3a5FA5iChz37gUuJbo5cwqvXdNebE5WBfWeHx",
	},
	{
		name: "V5_explicit_path_4_components_with_fp",
		strings: []string{
			"mk1qp2eufzqqsq42enh3qqsyqcylczgln5qsqyd9zvqsqyt3qyqsqyg0qyqsqyqfz9jrcgzqvq947h6lta047h6lta047h67xj4jt7g69atcpze",
			"mk1qp2eufzp47h6lta047h6lta047h6lta047h6ltcrvtq2q3k6en5xmhgrg0rd8378ns3q3wsdnjw0yjndq3kjr5sljrm3ydu6j4m83w45h234",
			"mk1qp2eufzzrscsjqdk69lrveg2fm",
		},
		network: "mainnet", path: "m/9999'/1234'/56'/7'", fp: "01020304", stubs: []string{"55667788"},
		xpub: "xpub6Den8YxgJdggPygKKEv3wiQwQ6PSGUouW98xC4obAJAqvuWcBMHuxeuXHxyZtAJHLqE7U1JdEXrNwbNPNCn1F79n4ZuBTLnzF7mPbLR3ZvB",
	},
	{
		name: "V6_3_stubs_mainnet_with_fp",
		strings: []string{
			"mk1qpv7yspqqspaatgqq8026qqzm6ksqqlsph90upgy3zepuypqxqr2et9v4jk2et9v4jk2et9v4jk2et9v4jk2et9v4jk2cfr7h56h70u9lsha",
			"mk1qpv7yspp4jk2et9v4splqp4p34t9838d75u3lu36v8crl7paydlgsrhxzxrl48ehngpguzk8j6a47h024849cnxk4n",
		},
		network: "mainnet", path: "m/48'/0'/0'/2'", fp: "f00dcafe", stubs: []string{"dead0001", "dead0002", "dead0003"},
		xpub: "xpub6Den8YxxyxkcXmP7ygCeb7Bf1Ptqw1aQNa9iaigk6EPeoZHkeHmequH8aYiT3mUALmPo7ThDTZJf5cu5eziSYeW4fsbfdFubwdBgRetAhFa",
	},
	{
		name: "V7_max_path_components_no_fp",
		strings: []string{
			"mk1qp0zgpzqqqqepyvjj0lq4qyqszqq3qvqszqq3q5qszqq3quqszqq3pyqszqq3pvqszqq3p5qszqq3puqszqq3zyqszqqse9ppcgqls67s8nv",
			"mk1qp0zgpzp3xqgpqqgqjyty8ssyqcq0tdd4kk6mtdd4kk6mtdd4kk6mtdd4kk6mtdd4kk6mtdd4kk6mtddq2vfczmkedtrj2rjl6la2h9ek48q",
			"mk1qp0zgpzzw87un0hnrmqxcdtq7vjf6mhfuhvrc4mz2ktwqhm0qwv5qvsnckdz0yclv6ky",
		},
		network: "mainnet", path: "m/0'/1'/2'/3'/4'/5'/6'/7'/8'/9'", fp: "", stubs: []string{"90919293"},
		xpub: "xpub6QwbHG5Nw7rYLo6utUHsXUqaaojc3YDdq84Ho7HV3mHuiJ1NNXB1GzUdBCMVph1HfRMMuRjW2VVVr8k5Fz7YGrKVGwVYPBcXr6dZKQenNqk",
	},
}

func TestDecodeParity(t *testing.T) {
	for _, v := range parityVectors {
		t.Run(v.name, func(t *testing.T) {
			card, err := Decode(v.strings)
			if err != nil {
				t.Fatalf("Decode: %v", err)
			}
			if card.Network != v.network {
				t.Errorf("network = %q, want %q", card.Network, v.network)
			}
			if card.Path != v.path {
				t.Errorf("path = %q, want %q", card.Path, v.path)
			}
			if card.Fingerprint != v.fp {
				t.Errorf("fp = %q, want %q", card.Fingerprint, v.fp)
			}
			if card.Xpub != v.xpub {
				t.Errorf("xpub = %q, want %q", card.Xpub, v.xpub)
			}
			if len(card.Stubs) != len(v.stubs) {
				t.Fatalf("stub count = %d, want %d", len(card.Stubs), len(v.stubs))
			}
			for i, want := range v.stubs {
				if got := hexStub(card.Stubs[i]); got != want {
					t.Errorf("stub %d = %s, want %s", i, got, want)
				}
			}
		})
	}
}

func hexStub(b [4]byte) string {
	const hexdig = "0123456789abcdef"
	out := make([]byte, 8)
	for i, c := range b {
		out[i*2] = hexdig[c>>4]
		out[i*2+1] = hexdig[c&0xf]
	}
	return string(out)
}

func TestDecodeReassemblyOrderIndependent(t *testing.T) {
	v := parityVectors[0]
	rev := []string{v.strings[1], v.strings[0]} // reversed chunk order
	card, err := Decode(rev)
	if err != nil || card.Xpub != v.xpub {
		t.Fatalf("reversed-order Decode: xpub=%q err=%v", card.Xpub, err)
	}
}

func TestDecodeNegative(t *testing.T) {
	cases := []struct {
		name    string
		strings []string
	}{
		// Corpus schema-2 reject vectors (assert rejection, not error-string equality).
		{"N5_bch_uncorrectable", []string{
			"mk1qpzg69pqpqpqql46hm02m0h0qvzg3vs7zqsrplj52329g4z52329g4z52329g4z52329g4z52329g4z52329g4z52spqcw0rafrc8fnsh6sz",
			"mk1qpzg69ppu3e2uhvfj0nkp8hyauemx38khpye5yjexa9a7550sgjqnpdlq0y74taw9wyd9vvg6cecl",
		}},
		{"N6_unsupported_card_type", []string{"mk1qzqqqqqqqqqqqqqvy5namurdhk04"}},
		{"N7_malformed_padding", []string{"mk1qqqqr396edwcs33vch"}},
		{"N11_cross_chunk_hash_mismatch", []string{
			"mk1qpzg69pqqsqu4l46hm02m0h0qvzg3vs7zqsrplj52329g4z52329g4z52329g4z52329g4z52329g4z52329g4z52spqcw0rafrc8fnsh6sz",
			"mk1qpzg69ppu3e2uhvfj0nkp8hyauemx38khpye5yjexa9a7550sgjqnpdlq0y74t63da7ac22u7at6k",
		}},
		{"N15_invalid_path_indicator", []string{
			"mk1qpzg69pqqqqu4l46hcqqfz9jrcgzqv872329g4z52329g4z52329g4z52329g4z52329g4z52329g4z5232qyr8yw2h96fyy7xfz6vg5y8j6",
			"mk1qpzg69pp3xf7wcy7unhn8v6y76uynxsjtym5hh6j37pzgzv9hupk53wd0sv3njltfwe4x4g",
		}},
		// Constructed cases.
		{"empty_input", nil},
		{"count_below_total", []string{parityVectors[0].strings[0]}}, // 1 of 2
		{"duplicate_index", []string{parityVectors[0].strings[0], parityVectors[0].strings[0]}},
		{"mixed_chunk_sets", []string{parityVectors[0].strings[0], parityVectors[2].strings[1]}}, // V1 idx0 + V3 idx1
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			card, err := Decode(c.strings)
			if err == nil {
				t.Fatalf("Decode(%s): want error, got Card %+v", c.name, card)
			}
			if card != (Card{}) {
				t.Fatalf("Decode(%s): want zero Card on error, got %+v", c.name, card)
			}
		})
	}
}
```
- [ ] **Step 2: Run — expect FAIL** (`Decode` undefined): `/home/bcg/.local/go/bin/go test ./mk/ -run TestDecode 2>&1 | tail`
- [ ] **Step 3: Implement** — append to `mk/mk.go`:
```go
const (
	fingerprintFlagMask   = 0x04
	reservedMask          = 0x0b // bits 0,1,3
	explicitPathIndicator = 0xFE
	maxPathComponents     = 10
	xpubCompactBytes      = 73
	crossChunkHashBytes   = 4
	hardenedBit           = 0x80000000
)

// Card is the decoded account metadata carried by an mk1 string set.
type Card struct {
	Network     string    // "mainnet" | "testnet"
	Path        string    // e.g. "m/48'/0'/0'/2'" ("m" for depth-0)
	Fingerprint string    // 8 lowercase hex, or "" if absent
	Stubs       [][4]byte // policy-id stubs (len >= 1)
	Xpub        string    // base58 "xpub…"/"tpub…"
}

type chunkFrag struct {
	header   Header
	fragment []byte
}

// Decode reassembles a complete set of BCH-valid mk1 chunk strings (any order)
// and decodes to a Card.
func Decode(in []string) (Card, error) {
	if len(in) == 0 {
		return Card{}, errEmptyInput
	}
	frags := make([]chunkFrag, 0, len(in))
	for _, s := range in {
		syms, err := codex32.MKDataSymbols(s)
		if err != nil {
			return Card{}, err
		}
		h, n, err := parseHeaderSyms(syms)
		if err != nil {
			return Card{}, err
		}
		frag, err := fiveBitToBytes(syms[n:])
		if err != nil {
			return Card{}, err
		}
		frags = append(frags, chunkFrag{header: h, fragment: frag})
	}
	bytecode, err := reassemble(frags)
	if err != nil {
		return Card{}, err
	}
	return decodeBytecode(bytecode)
}

func reassemble(frags []chunkFrag) ([]byte, error) {
	first := frags[0].header
	if !first.Chunked {
		if len(frags) != 1 {
			return nil, errChunkedHeaderMalformed
		}
		return frags[0].fragment, nil // single-string fragment IS the bytecode (no hash).
	}
	total := first.TotalChunks
	if len(frags) != total {
		return nil, fmt.Errorf("mk: received %d chunks, header declares %d", len(frags), total)
	}
	slots := make([][]byte, total)
	for _, f := range frags {
		if !f.header.Chunked {
			return nil, errMixedHeaderTypes
		}
		if f.header.ChunkSetID != first.ChunkSetID {
			return nil, errChunkSetIDMismatch
		}
		if f.header.TotalChunks != total {
			return nil, errChunkedHeaderMalformed
		}
		idx := f.header.ChunkIndex
		if idx >= total {
			return nil, errChunkedHeaderMalformed
		}
		if slots[idx] != nil {
			return nil, errDuplicateChunk
		}
		slots[idx] = f.fragment
	}
	var stream []byte
	for i, frag := range slots {
		if frag == nil {
			return nil, fmt.Errorf("mk: missing chunk %d", i)
		}
		stream = append(stream, frag...)
	}
	if len(stream) < crossChunkHashBytes {
		return nil, errCrossChunkHash
	}
	split := len(stream) - crossChunkHashBytes
	bytecode := stream[:split]
	sum := sha256.Sum256(bytecode)
	if !bytes.Equal(sum[:crossChunkHashBytes], stream[split:]) {
		return nil, errCrossChunkHash
	}
	return bytecode, nil
}

func decodeBytecode(b []byte) (Card, error) {
	cur := 0
	read := func(n int) ([]byte, error) {
		if cur+n > len(b) {
			return nil, errUnexpectedEnd
		}
		out := b[cur : cur+n]
		cur += n
		return out, nil
	}
	hdr, err := read(1)
	if err != nil {
		return Card{}, err
	}
	if hdr[0]>>4 != 0 {
		return Card{}, fmt.Errorf("mk: unsupported bytecode version: %d", hdr[0]>>4)
	}
	if hdr[0]&reservedMask != 0 {
		return Card{}, errReservedBits
	}
	fpPresent := hdr[0]&fingerprintFlagMask != 0
	scb, err := read(1)
	if err != nil {
		return Card{}, err
	}
	stubCount := int(scb[0])
	if stubCount == 0 {
		return Card{}, errStubCount
	}
	stubs := make([][4]byte, stubCount)
	for i := range stubs {
		sb, err := read(4)
		if err != nil {
			return Card{}, err
		}
		copy(stubs[i][:], sb)
	}
	fp := ""
	if fpPresent {
		fpb, err := read(4)
		if err != nil {
			return Card{}, err
		}
		fp = hex.EncodeToString(fpb)
	}
	comps, err := decodePath(read)
	if err != nil {
		return Card{}, err
	}
	compact, err := read(xpubCompactBytes)
	if err != nil {
		return Card{}, err
	}
	if cur != len(b) {
		return Card{}, errTrailingBytes
	}
	xpub, network, err := reconstructXpub(compact, comps)
	if err != nil {
		return Card{}, err
	}
	return Card{Network: network, Path: pathString(comps), Fingerprint: fp, Stubs: stubs, Xpub: xpub}, nil
}

func h(i uint32) uint32 { return i | hardenedBit }

var standardPaths = map[byte][]uint32{
	0x01: {h(44), h(0), h(0)},
	0x02: {h(49), h(0), h(0)},
	0x03: {h(84), h(0), h(0)},
	0x04: {h(86), h(0), h(0)},
	0x05: {h(48), h(0), h(0), h(2)},
	0x06: {h(48), h(0), h(0), h(1)},
	0x07: {h(87), h(0), h(0)},
	0x11: {h(44), h(1), h(0)},
	0x12: {h(49), h(1), h(0)},
	0x13: {h(84), h(1), h(0)},
	0x14: {h(86), h(1), h(0)},
	0x15: {h(48), h(1), h(0), h(2)},
	0x16: {h(48), h(1), h(0), h(1)},
	0x17: {h(87), h(1), h(0)},
}

func decodePath(read func(int) ([]byte, error)) ([]uint32, error) {
	ib, err := read(1)
	if err != nil {
		return nil, err
	}
	ind := ib[0]
	if ind == explicitPathIndicator {
		cb, err := read(1)
		if err != nil {
			return nil, err
		}
		count := int(cb[0])
		if count > maxPathComponents {
			return nil, errPathTooDeep
		}
		comps := make([]uint32, 0, count)
		for i := 0; i < count; i++ {
			v, err := readLEB128(read)
			if err != nil {
				return nil, err
			}
			comps = append(comps, v)
		}
		return comps, nil
	}
	if p, ok := standardPaths[ind]; ok {
		out := make([]uint32, len(p))
		copy(out, p)
		return out, nil
	}
	return nil, fmt.Errorf("mk: invalid path indicator byte: 0x%02x", ind)
}

func readLEB128(read func(int) ([]byte, error)) (uint32, error) {
	var result uint64
	var shift uint
	for {
		bb, err := read(1)
		if err != nil {
			return 0, err
		}
		result |= uint64(bb[0]&0x7f) << shift
		if bb[0]&0x80 == 0 {
			break
		}
		shift += 7
		if shift >= 35 {
			return 0, errPathComponent
		}
	}
	if result > 0xffffffff {
		return 0, errPathComponent
	}
	return uint32(result), nil
}

func pathString(comps []uint32) string {
	var b strings.Builder
	b.WriteString("m")
	for _, c := range comps {
		b.WriteByte('/')
		if c&hardenedBit != 0 {
			fmt.Fprintf(&b, "%d'", c&^uint32(hardenedBit))
		} else {
			fmt.Fprintf(&b, "%d", c)
		}
	}
	return b.String()
}

func reconstructXpub(compact []byte, comps []uint32) (xpub, network string, err error) {
	if len(compact) != xpubCompactBytes {
		return "", "", errUnexpectedEnd
	}
	version := compact[0:4]
	parentFP := compact[4:8]
	chainCode := compact[8:40]
	pubKey := compact[40:73]
	switch hex.EncodeToString(version) {
	case "0488b21e":
		network = "mainnet"
	case "043587cf":
		network = "testnet"
	default:
		return "", "", fmt.Errorf("mk: invalid xpub version: %x", version)
	}
	if _, err := btcec.ParsePubKey(pubKey); err != nil {
		return "", "", fmt.Errorf("mk: invalid xpub public key: %w", err)
	}
	depth := uint8(len(comps))
	childNum := uint32(0)
	if len(comps) > 0 {
		childNum = comps[len(comps)-1] // raw, hardened bit included (R0-M1).
	}
	key := hdkeychain.NewExtendedKey(version, pubKey, chainCode, parentFP, depth, childNum, false)
	return key.String(), network, nil
}
```
- [ ] **Step 4: Run — expect PASS:** `/home/bcg/.local/go/bin/go test ./mk/ -v 2>&1 | tail -40` (all parity + negative + header subtests pass).
- [ ] **Step 5: Commit:**
```bash
git add mk/mk.go mk/mk_test.go
git -c commit.gpgsign=true commit -S -s -m "mk: Decode — reassembly + bytecode decode + xpub reconstruction, parity-verified (T2b)"
```

---

## Task 4: GUI pure gatherer `mk1Gatherer`

**Files:** Create `gui/mk1_inspect.go` (gatherer + helpers), `gui/mk1_inspect_test.go`.

- [ ] **Step 1: Write the failing test** — `gui/mk1_inspect_test.go`:
```go
package gui

import (
	"testing"

	"seedhammer.com/mk"
)

// V1 (2-chunk) and V3 (different key set) strings.
const (
	v1c0 = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf"
	v1c1 = "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x"
	v3c1 = "mk1qpx3t8pprlnqdqf52q7jwgcnxgnuseav37nvs0zn06dyfs79hk7uk8lrxlyw57x7v7rzx74tlflqh"
)

func TestMK1Gatherer(t *testing.T) {
	g := &mk1Gatherer{}
	if st := g.offer(v1c1); st != gatherAdded { // out-of-order: index 1 first
		t.Fatalf("offer c1: status %v", st)
	}
	if g.complete() {
		t.Fatal("complete after 1 of 2")
	}
	if st := g.offer(v1c1); st != gatherDup {
		t.Fatalf("offer dup: status %v", st)
	}
	if st := g.offer(v3c1); st != gatherForeign { // different chunk_set_id
		t.Fatalf("offer foreign: status %v", st)
	}
	if st := g.offer("not an mk1 chunk"); st != gatherIgnored {
		t.Fatalf("offer garbage: status %v", st)
	}
	if st := g.offer(v1c0); st != gatherAdded {
		t.Fatalf("offer c0: status %v", st)
	}
	if !g.complete() {
		t.Fatal("not complete after 2 of 2")
	}
	card, err := mk.Decode(g.collected())
	if err != nil {
		t.Fatalf("Decode(collected): %v", err)
	}
	if card.Path != "m/48'/0'/0'/2'" {
		t.Fatalf("path = %q", card.Path)
	}
}

func TestHasMKPrefix(t *testing.T) {
	if !hasMKPrefix("mk1qpzg69p...") || !hasMKPrefix("MK1QPZG...") {
		t.Fatal("mk1 prefix not detected")
	}
	if hasMKPrefix("md1qabc...") {
		t.Fatal("md1 misdetected as mk1")
	}
}
```
- [ ] **Step 2: Run — expect FAIL** (`mk1Gatherer` undefined): `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestMK1Gatherer|TestHasMKPrefix' 2>&1 | tail`
- [ ] **Step 3: Implement the gatherer + helpers** — create `gui/mk1_inspect.go` with this prefix (the flows are added in Tasks 5–6):
```go
package gui

import (
	"errors"
	"fmt"
	"image"
	"io"
	"log"
	"strings"
	"time"

	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/layout"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
	"seedhammer.com/mk"
)

func hasMKPrefix(s string) bool {
	return strings.HasPrefix(s, "mk1") || strings.HasPrefix(s, "MK1")
}

// chunkString splits s into substrings of at most n runes (ASCII here), so the
// long base58 xpub renders as short non-wrapping display lines.
func chunkString(s string, n int) []string {
	var out []string
	for len(s) > n {
		out = append(out, s[:n])
		s = s[n:]
	}
	if len(s) > 0 {
		out = append(out, s)
	}
	return out
}

type gatherStatus int

const (
	gatherIgnored gatherStatus = iota // not an mk1 chunk / parse failed
	gatherForeign                     // valid mk1 but a different chunk set
	gatherDup                         // chunk index already captured
	gatherAdded                       // new chunk added
)

// mk1Gatherer accumulates mk1 chunk strings toward a complete set. Pure (no
// GUI/NFC) so it is unit-tested directly; mk1GatherFlow is a thin NFC shell.
type mk1Gatherer struct {
	set    map[int]string
	total  int
	setID  uint32
	primed bool
}

func (g *mk1Gatherer) offer(s string) gatherStatus {
	h, err := mk.ParseHeader(s)
	if err != nil {
		return gatherIgnored
	}
	if !g.primed {
		g.set = map[int]string{}
		g.total = h.TotalChunks
		g.setID = h.ChunkSetID
		g.primed = true
	} else if !h.Chunked || h.ChunkSetID != g.setID || h.TotalChunks != g.total {
		return gatherForeign
	}
	if _, ok := g.set[h.ChunkIndex]; ok {
		return gatherDup
	}
	g.set[h.ChunkIndex] = s
	return gatherAdded
}

func (g *mk1Gatherer) complete() bool { return g.primed && len(g.set) == g.total }

func (g *mk1Gatherer) collected() []string {
	out := make([]string, 0, len(g.set))
	for _, s := range g.set {
		out = append(out, s)
	}
	return out
}
```
(Imports `fmt`/`image`/`io`/`log`/`time`/`assets`/`layout`/`op`/`widget` are unused until Tasks 5–6; if `go build` complains before then, add a `var _ = …` — but implement Tasks 5–6 in the same branch before the Task 6 build/commit, so they resolve. To keep Task 4 self-compiling, temporarily import only `strings` + `seedhammer.com/mk`, and add the rest in Task 5.)
- [ ] **Step 4: Run — expect PASS:** `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestMK1Gatherer|TestHasMKPrefix' -v`
- [ ] **Step 5: Commit:**
```bash
git add gui/mk1_inspect.go gui/mk1_inspect_test.go
git -c commit.gpgsign=true commit -S -s -m "gui: mk1Gatherer — pure multi-chunk mk1 accumulator (T2b)"
```

---

## Task 5: GUI `mk1DisplayFlow` (measure-and-advance decode-display)

**Files:** Modify `gui/mk1_inspect.go` (add the display flow + the imports deferred from Task 4), `gui/mk1_inspect_test.go` (append).

- [ ] **Step 1: Write the failing test** — append to `gui/mk1_inspect_test.go`:
```go
import "strings" // (add to the existing import block)

func TestMK1DisplayFlowPaging(t *testing.T) {
	ctx := NewContext(newPlatform())
	card := mk.Card{
		Network:     "mainnet",
		Path:        "m/48'/0'/0'/2'",
		Fingerprint: "aabbccdd",
		Stubs:       make([][4]byte, 1),
		Xpub:        "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc",
	}
	frame, quit := runUI(ctx, func() { mk1DisplayFlow(ctx, &descriptorTheme, card) })
	defer quit()
	var all strings.Builder
	for i := 0; i < 16; i++ {
		content, ok := frame()
		if !ok {
			break
		}
		all.WriteString(content)
		click(&ctx.Router, Button3) // page forward
	}
	got := all.String()
	if !uiContains(got, "m/48'/0'/0'/2'") {
		t.Errorf("path not shown; got %q", got)
	}
	if !uiContains(got, "aabbccdd") {
		t.Errorf("fingerprint not shown")
	}
	// Invariant 2.10: paging reaches the xpub tail, gap-free.
	if !uiContains(got, "1hM7vFrc") {
		t.Errorf("xpub tail not reached via paging")
	}
}

func TestMK1DisplayFlowBackExits(t *testing.T) {
	ctx := NewContext(newPlatform())
	card := mk.Card{Network: "mainnet", Path: "m", Stubs: make([][4]byte, 1), Xpub: "xpub6x"}
	frame, quit := runUI(ctx, func() { mk1DisplayFlow(ctx, &descriptorTheme, card) })
	defer quit()
	frame()
	click(&ctx.Router, Button1) // Back
	if _, ok := frame(); ok {
		t.Fatal("mk1DisplayFlow did not exit on Back")
	}
}
```
- [ ] **Step 2: Run — expect FAIL** (`mk1DisplayFlow` undefined): `/home/bcg/.local/go/bin/go test ./gui/ -run TestMK1DisplayFlow 2>&1 | tail`
- [ ] **Step 3: Implement** — append to `gui/mk1_inspect.go` (and ensure all imports listed in Task 4's block are now present):
```go
// mk1DisplayFlow shows the decoded mk1 account metadata for verification. Read-
// only: no engrave, no NFC, no mutation. Measure-and-advance paging (the T1
// lesson): the long base58 xpub is chunked into short non-wrapping lines and
// paged gap-free so the tail is always reachable (spec invariant 2.10).
func mk1DisplayFlow(ctx *Context, th *Colors, card mk.Card) {
	fp := card.Fingerprint
	if fp == "" {
		fp = "none"
	}
	lines := []string{
		"Network: " + card.Network,
		"Path: " + card.Path,
		"Fingerprint: " + fp,
		fmt.Sprintf("Policy stubs: %d", len(card.Stubs)),
		"Account xpub:",
	}
	lines = append(lines, chunkString(card.Xpub, 20)...)

	backBtn := &Clickable{Button: Button1}
	pageBtn := &Clickable{Button: Button3}
	dims := ctx.Platform.DisplaySize()
	lineWidth := dims.X - 2*8
	screen := layout.Rectangle{Max: dims}
	_, content := screen.CutTop(leadingSize)
	content, _ = content.CutBottom(leadingSize)
	contentTop := content.Min.Y + 8
	contentBottom := content.Max.Y
	start := 0
	for !ctx.Done {
		if backBtn.Clicked(ctx) {
			return
		}
		shown := 0
		y := contentTop
		body := make([]op.Op, 0, len(lines))
		for i := start; i < len(lines); i++ {
			lbl, sz := widget.Labelw(&ctx.B, ctx.Styles.body, lineWidth, th.Text, lines[i])
			if i > start && y+sz.Y > contentBottom {
				break
			}
			body = append(body, lbl.Offset(image.Pt((dims.X-sz.X)/2, y)))
			y += sz.Y + 6
			shown++
			if y > contentBottom {
				break
			}
		}
		if pageBtn.Clicked(ctx) {
			if start+shown < len(lines) {
				start += shown
			} else {
				start = 0
			}
			continue
		}
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, "mk1 key")
		nav, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack},
			{Clickable: pageBtn, Style: StylePrimary, Icon: assets.IconRight},
		}...)
		frameOps := append([]op.Op{nav, titleOp}, body...)
		frameOps = append(frameOps, op.Color(&ctx.B, th.Background))
		ctx.Frame(op.Layer(frameOps...))
	}
}
```
- [ ] **Step 4: Run — expect PASS:** `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestMK1Display|TestMK1Gatherer|TestHasMKPrefix' -v`
- [ ] **Step 5: Commit:**
```bash
git add gui/mk1_inspect.go gui/mk1_inspect_test.go
git -c commit.gpgsign=true commit -S -s -m "gui: mk1DisplayFlow — measure-and-advance decode-display (T2b)"
```

---

## Task 6: GUI `mk1GatherFlow` (NFC screen) + wire into `mdmkFlow`

**Files:** Modify `gui/mk1_inspect.go` (add the gather screen), `gui/gui.go` (`mdmkFlow`), `gui/mk1_inspect_test.go` (append).

- [ ] **Step 1: Write the failing test** — append to `gui/mk1_inspect_test.go`:
```go
func TestMK1GatherFlowBackNoReader(t *testing.T) {
	// testPlatform.NFCReader() == nil, so a multi-chunk set can't complete;
	// only Back exits. Verifies the no-reader render path + progress.
	ctx := NewContext(newPlatform())
	var card mk.Card
	var ok bool
	frame, quit := runUI(ctx, func() { card, ok = mk1GatherFlow(ctx, &descriptorTheme, v1c0) })
	defer quit()
	content, _ := frame()
	if !uiContains(content, "1 of 2") {
		t.Errorf("progress not shown; got %q", content)
	}
	click(&ctx.Router, Button1) // Back
	if _, fok := frame(); fok {
		t.Fatal("mk1GatherFlow did not exit on Back")
	}
	if ok || card != (mk.Card{}) {
		t.Fatalf("Back should yield (zero, false); got ok=%v card=%+v", ok, card)
	}
}

func TestMdmkFlowMK1ShowsInspect(t *testing.T) {
	p := newPlatform()
	p.engraver = newEngraver()
	ctx := NewContext(p)
	frame, quit := runUI(ctx, func() { mdmkFlow(ctx, &descriptorTheme, mdmkText(v1c0)) })
	defer quit()
	content, ok := frame()
	if !ok {
		t.Fatal("mdmkFlow produced no frame")
	}
	if !uiContains(content, "Inspect key") {
		t.Errorf("mk1 chooser missing Inspect key; got %q", content)
	}
}
```
- [ ] **Step 2: Run — expect FAIL** (`mk1GatherFlow` undefined): `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestMK1GatherFlow|TestMdmkFlowMK1' 2>&1 | tail`
- [ ] **Step 3a: Implement `mk1GatherFlow`** — append to `gui/mk1_inspect.go`:
```go
// mk1GatherFlow collects a complete mk1 chunk set via NFC, starting from the
// first scanned chunk, then decodes and returns the Card. It owns its own
// scanner goroutine (StartScreen.Flow has already closed its reader before
// engraveObjectFlow runs). Returns (Card, true) on a complete valid set, or
// (zero, false) on Back / decode error.
func mk1GatherFlow(ctx *Context, th *Colors, first string) (mk.Card, bool) {
	g := &mk1Gatherer{}
	g.offer(first) // first came from a ValidMK mdmkText; primes the set.
	if g.complete() {
		return decodeGathered(ctx, th, g)
	}
	scans := make(chan scanResult, 1)
	if r := ctx.Platform.NFCReader(); r != nil {
		closer := make(chan struct{})
		closed := make(chan struct{})
		defer func() {
			close(closer)
			r.Close()
			<-closed
		}()
		wakeup := ctx.Platform.Wakeup
		go func() {
			s := new(scanner)
			for {
				select {
				case <-closer:
					close(closed)
					return
				default:
				}
				obj, err := s.Scan(r)
				scan := scanResult{Object: obj}
				switch {
				case errors.Is(err, errScanInProgress):
					scan.Status = scanStarted
				case errors.Is(err, errScanUnknownFormat):
					scan.Status = scanUnknownFormat
				case err == nil || err == io.EOF:
				default:
					scan.Status = scanFailed
					log.Printf("nfc scan: %v", err)
				}
				select {
				case old := <-scans:
					if scan.Object == nil {
						scan.Object = old.Object
					}
					scan.Status = max(scan.Status, old.Status)
				default:
				}
				scans <- scan
				wakeup()
				if scan.Status == scanFailed {
					time.Sleep(1 * time.Second)
				}
			}
		}()
	}
	backBtn := &Clickable{Button: Button1}
	dims := ctx.Platform.DisplaySize()
	msg := ""
	for !ctx.Done {
		if backBtn.Clicked(ctx) {
			return mk.Card{}, false
		}
		select {
		case scan := <-scans:
			if s, ok := scan.Object.(mdmkText); ok {
				switch g.offer(string(s)) {
				case gatherAdded:
					msg = ""
					if g.complete() {
						return decodeGathered(ctx, th, g)
					}
				case gatherForeign:
					msg = "Different key — rescan the right card."
				case gatherDup:
					msg = "Already captured that chunk."
				case gatherIgnored:
					msg = "Not an mk1 key chunk."
				}
			}
		default:
		}
		lines := []string{fmt.Sprintf("Captured %d of %d.", len(g.set), g.total), "Scan the next chunk."}
		if msg != "" {
			lines = append(lines, msg)
		}
		lineWidth := dims.X - 2*8
		y := leadingSize + 8
		body := make([]op.Op, 0, len(lines))
		for _, ln := range lines {
			lbl, sz := widget.Labelw(&ctx.B, ctx.Styles.body, lineWidth, th.Text, ln)
			body = append(body, lbl.Offset(image.Pt((dims.X-sz.X)/2, y)))
			y += sz.Y + 6
		}
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, "Inspect key")
		nav, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack},
		}...)
		frameOps := append([]op.Op{nav, titleOp}, body...)
		frameOps = append(frameOps, op.Color(&ctx.B, th.Background))
		ctx.Frame(op.Layer(frameOps...))
	}
	return mk.Card{}, false
}

func decodeGathered(ctx *Context, th *Colors, g *mk1Gatherer) (mk.Card, bool) {
	card, err := mk.Decode(g.collected())
	if err != nil {
		showError(ctx, th, "Inspect key", "Can't decode this key set.")
		return mk.Card{}, false
	}
	return card, true
}
```
- [ ] **Step 3b: Modify `mdmkFlow`** in `gui/gui.go` — replace the EXACT current function:
```go
// mdmkFlow lets the operator pick an engraving variant for an md1/mk1 string
// and engrave it, mirroring descriptorFlow.
func mdmkFlow(ctx *Context, th *Colors, s mdmkText) {
	labels, engravings, err := validateMdmk(ctx.Platform.EngraverParams(), string(s))
	if err != nil {
		// Only reached if no engraving variant fits a plate (rare for an md1/mk1
		// string). Return silently — like backupSeedStringFlow, NOT like
		// descriptorFlow (whose ErrorScreen "Too Large" copy is descriptor-specific).
		return
	}
	cs := &ChoiceScreen{Title: "Engrave", Lead: "Choose engraving", Choices: labels}
	for {
		choice, ok := cs.Choose(ctx, th)
		if !ok {
			return
		}
		if NewEngraveScreen(ctx, engravings[choice]).Engrave(ctx, &engraveTheme) {
			return
		}
	}
}
```
with (mk1-only Inspect affordance; md1 path byte-identical):
```go
// mdmkFlow lets the operator pick an engraving variant for an md1/mk1 string
// and engrave it. For mk1 (only) it also offers "Inspect key" — gather the
// chunk set, decode, and display the account metadata (read-only) before
// engraving. md1 behaviour is unchanged until T2c.
func mdmkFlow(ctx *Context, th *Colors, s mdmkText) {
	str := string(s)
	labels, engravings, err := validateMdmk(ctx.Platform.EngraverParams(), str)
	if err != nil {
		// Only reached if no engraving variant fits a plate (rare for an md1/mk1
		// string). Return silently — like backupSeedStringFlow, NOT like
		// descriptorFlow (whose ErrorScreen "Too Large" copy is descriptor-specific).
		return
	}
	isMK := hasMKPrefix(str)
	title, lead, choices := "Engrave", "Choose engraving", labels
	if isMK {
		title, lead = "mk1 key", "Choose action"
		choices = append([]string{"Inspect key"}, labels...)
	}
	cs := &ChoiceScreen{Title: title, Lead: lead, Choices: choices}
	for {
		choice, ok := cs.Choose(ctx, th)
		if !ok {
			return
		}
		if isMK && choice == 0 {
			if card, ok := mk1GatherFlow(ctx, th, str); ok {
				mk1DisplayFlow(ctx, th, card)
			}
			continue
		}
		idx := choice
		if isMK {
			idx-- // skip the prepended Inspect entry
		}
		if NewEngraveScreen(ctx, engravings[idx]).Engrave(ctx, &engraveTheme) {
			return
		}
	}
}
```
- [ ] **Step 4: Run — expect PASS** + no regressions:
```
/home/bcg/.local/go/bin/go build ./...
/home/bcg/.local/go/bin/go test ./gui/ ./mk/ ./codex32/ -v 2>&1 | tail -50
/home/bcg/.local/go/bin/go test -run TestAllocs ./gui/   # alloc gate intact
```
- [ ] **Step 5: Commit:**
```bash
git add gui/mk1_inspect.go gui/gui.go gui/mk1_inspect_test.go
git -c commit.gpgsign=true commit -S -s -m "gui: mk1 multi-chunk gather + Inspect affordance in mdmkFlow (T2b)"
```

---

## Task 7: Full verification
- [ ] **Step 1:** `/home/bcg/.local/go/bin/go test ./... && /home/bcg/.local/go/bin/go vet ./codex32/ ./mk/ ./gui/ && /home/bcg/.local/go/bin/go fmt ./codex32/ ./mk/ ./gui/ && gofmt -l codex32/ mk/ gui/` → all clean (gofmt -l empty).
- [ ] **Step 2 (CI):** the TinyGo firmware build (`tinygo build … ./cmd/controller`, flake.nix) compiles `codex32`+`mk`+`gui` — local if TinyGo present, else confirm in CI before merge. (Pure byte-slice + hdkeychain/btcec, which the fork already builds under TinyGo via bip380.)

---

## Done criteria
- All 7 V1–V7 parity vectors decode to exact `{network, path, fp, stubs, xpub}`; all negative cases reject (no panic, zero Card).
- Gatherer: out-of-order completion, dup/foreign/non-mk1 rejection.
- Display pages to the xpub tail gap-free (invariant 2.10); Back exits; no engrave/NFC from inspect.
- md1 path byte-identical (no Inspect); mk1 shows Inspect; alloc gate (`TestAllocs`) passes; vet/gofmt clean.

## Self-review (vs spec)
- §2.1 gather mandatory → Tasks 4/6. §2.2 wire-exact incl. chunk_index verbatim → Task 2 header + Task 3 reassembly; TestParseHeader's index-1 assertion is the R0-C1 guard. §2.3 xpub reconstruction (childNum raw u32) → Task 3 reconstructXpub. §2.4 read-only → Tasks 5/6 (no engrave/NFC). §2.5 no regression → Task 6 md1-identical + TestMdmkFlowMK1ShowsInspect. §2.6 alloc gate → Task 7. §2.7 no secrets → no scrub anywhere. §2.8 full reject set → Task 3 decode + Task 3 negatives. §2.9 HRP discrimination → hasMKPrefix. §2.10 paging tail → TestMK1DisplayFlowPaging. §6 vectors V1–V7 clean + negative layering → Task 3.
- No placeholders; every step has runnable code/commands. Type names consistent across tasks (`mk.Card`, `mk.Header`, `mk1Gatherer`, `gatherStatus`).
- **R0 gate next:** this plan MUST pass an opus-architect R0 to 0C/0I before any code; fold → persist verbatim to `design/agent-reports/` → re-dispatch until GREEN.
