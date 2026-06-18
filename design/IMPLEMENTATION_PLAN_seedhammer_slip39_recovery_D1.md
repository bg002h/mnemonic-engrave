# SLIP-39 recovery — D1 (crypto port) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development.
> Steps use `- [ ]` checkboxes. This is **Phase D1** of Cycle D (crypto only, no GUI) —
> see `design/SPEC_seedhammer_slip39_recovery.md` (R1 GREEN) §0/§4/§7. D2 (GUI) is a
> separate plan written against D1's frozen `Combine` contract.

**Goal:** Add in-tree Go SLIP-0039 secret-recovery crypto (combine direction) to the
SeedHammer fork: GF(256) field, Lagrange interpolation, 4-round Feistel decrypt, two-level
`Combine`, and share-VALUE extraction for all valid lengths — a faithful port of
`mnemonic_toolkit::slip39`, TDD'd against the official vectors + Rust-generated fixtures.

**Architecture:** Five Go files in package `slip39` (`gf256.go`, `lagrange.go`, `feistel.go`,
`combine.go` new; `share.go` extended). No GUI. Dormant until D2 calls `Combine`. The Rust at
`/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/slip39/{gf256,lagrange,
feistel,share,mod}.rs` is the authoritative oracle — port faithfully and cross-check.

**Tech stack:** Go/TinyGo (RP2350, `int` is 32-bit). `crypto/sha256`, `crypto/hmac`,
`crypto/subtle`, `golang.org/x/crypto/pbkdf2` (all already in the firmware link closure).
**No `math/big`** in any of these files.

**Test command (host):** `/home/bcg/.local/go/bin/go test ./slip39/`
Full guard: `/home/bcg/.local/go/bin/go test ./slip39/ ./gui/ ./bip39/` + `go vet ./slip39/`
+ `gofmt -l`.

**Commit hygiene:** explicit paths; SSH-signed + DCO (`git commit -S -s`, author Brian Goss);
end messages with `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.

---

## File structure

| File | Responsibility |
|---|---|
| `slip39/gf256.go` | GF(2⁸) Rijndael field (port of `gf256.rs`). |
| `slip39/lagrange.go` | Interpolation over GF(256) (port of `lagrange.rs`). |
| `slip39/feistel.go` | 4-round Feistel **decrypt** + round-key scrub (port of `feistel.rs` decrypt path). |
| `slip39/combine.go` | `Combine` + `recoverSecret` + `ConsistentShares` + `wipe` (port of `mod.rs` `slip39_combine`/`recover_secret`). |
| `slip39/share.go` | **extend** — accept {20,23,27,30,33} words, extract `Value []byte`, `GroupThresholdExceedsCount`, drop `errUnsupportedSize`. |
| `slip39/*_test.go` | vectors + Rust-fixture round-trips + value + negatives + panic-safety + scrub. |
| `slip39/testdata/slip39_vectors.json` | the chosen official vectors (copied from `trezor/python-shamir-mnemonic`). |
| `slip39/testdata/slip39_fixtures.json` | Rust-`slip39_split`-generated intermediate-length round-trip fixtures (Task 6). |

**Unchanged (must stay green):** `codex32/`, `bip39/`, `backup/`, `gui/`. The existing
`slip39/share.go` RS1024 + header decode are reused as-is; only additive value extraction +
length widening + the structural check change.

---

## Task 0: Worktree + testdata + loader helpers (front-loaded per plan-R0 I1)

> **I1 fold (plan-R0):** to eliminate the inline-crypto-literal transcription-risk class (an
> R0 reviewer caught a fabricated idx-3 mnemonic), ALL crypto test vectors are loaded from a
> single `testdata/slip39_vectors.json` copied VERBATIM from upstream — **no crypto mnemonic
> or secret literal is ever hand-typed into a test.** Use `errors.Is` directly (no `errorsIs`
> wrapper — minor M2).

- [ ] **Step 1:** From `/scratch/code/shibboleth/seedhammer` (fork `main` @ `20fa4c4`):
  `git worktree add /scratch/code/shibboleth/seedhammer-wt-slip39-d1 -b feat/slip39-recovery-crypto 20fa4c4`
- [ ] **Step 2:** Baseline green: `cd /scratch/code/shibboleth/seedhammer-wt-slip39-d1 && /home/bcg/.local/go/bin/go test ./slip39/ ./gui/ ./bip39/` → all pass.
- [ ] **Step 3: Create `slip39/testdata/slip39_vectors.json` by copying VERBATIM** the needed
  entries from the upstream official vectors (raw:
  `https://raw.githubusercontent.com/trezor/python-shamir-mnemonic/master/vectors.json`).
  Do NOT hand-type mnemonics — fetch the file and extract the entries. Required indices (the
  4-tuple `[desc, [mnemonics], master_hex, xprv]` shape preserved): positives **0, 3, 17, 35,
  42**; negatives **1, 4, 5, 9, 12, 13**. (All valid vectors use passphrase `"TREZOR"`.)
- [ ] **Step 4: Add the loader helpers** in `slip39/vectors_test.go`:

```go
package slip39

import (
	"encoding/json"
	"os"
	"testing"
)

type slip39Vector struct {
	Desc      string
	Mnemonics []string
	MasterHex string
}

func loadVectors(t *testing.T) []slip39Vector {
	t.Helper()
	b, err := os.ReadFile("testdata/slip39_vectors.json")
	if err != nil {
		t.Fatalf("read vectors: %v", err)
	}
	var raw [][]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil {
		t.Fatalf("parse vectors: %v", err)
	}
	out := make([]slip39Vector, len(raw))
	for i, e := range raw {
		_ = json.Unmarshal(e[0], &out[i].Desc)
		_ = json.Unmarshal(e[1], &out[i].Mnemonics)
		_ = json.Unmarshal(e[2], &out[i].MasterHex)
	}
	return out
}

// vectorShares returns all mnemonics of official vector index idx.
func vectorShares(t *testing.T, idx int) []string { return loadVectors(t)[idx].Mnemonics }

// vectorShare returns share `share` of official vector index idx.
func vectorShare(t *testing.T, idx, share int) string { return loadVectors(t)[idx].Mnemonics[share] }

// vectorSecretHex returns the expected master-secret hex of vector idx ("" if invalid).
func vectorSecretHex(t *testing.T, idx int) string { return loadVectors(t)[idx].MasterHex }
```
  `hexEq(b []byte) string { return hex.EncodeToString(b) }` lives in `combine_test.go`
  (Task 5). All crypto tests reference these — **no inline mnemonic/secret literals anywhere.**

---

## Task 1: `slip39/gf256.go` — GF(2⁸) field

**Files:** Create `slip39/gf256.go`, `slip39/gf256_test.go`. Port of `gf256.rs`.

- [ ] **Step 1: Failing test** (`slip39/gf256_test.go`):

```go
package slip39

import "testing"

func TestGF256MulInvDiv(t *testing.T) {
	// Known GF(256) Rijndael (0x11b, generator 3) identities.
	if got := gfMul(0x53, 0xCA); got != 0x01 { // 0x53 and 0xCA are AES-inverse pair
		t.Errorf("gfMul(0x53,0xCA)=%#x want 0x01", got)
	}
	if got := gfMul(3, 0); got != 0 {
		t.Errorf("gfMul(_,0)=%#x want 0", got)
	}
	for a := 1; a < 256; a++ {
		if gfMul(byte(a), gfInv(byte(a))) != 1 {
			t.Fatalf("a*inv(a)!=1 for a=%d", a)
		}
		if gfDiv(byte(a), byte(a)) != 1 {
			t.Fatalf("a/a!=1 for a=%d", a)
		}
	}
	if gfAdd(0xAA, 0x55) != 0xFF {
		t.Errorf("gfAdd is XOR")
	}
}
```

- [ ] **Step 2:** Run `…/go test ./slip39/ -run TestGF256` → FAIL (undefined `gfMul`).

- [ ] **Step 3: Implement** `slip39/gf256.go` (faithful port of `gf256.rs:14-86`):

```go
package slip39

// GF(2^8) Rijndael field for SLIP-0039 Shamir. Reduction polynomial
// x^8+x^4+x^3+x+1 (0x11b); multiplicative generator 3. Tables built once.
// Port of mnemonic_toolkit::slip39::gf256. NO math/big.

const gf256ReductionPoly = 0x11b

var gf256Exp [256]byte
var gf256Log [256]byte

func init() {
	x := uint16(1)
	for i := 0; i < 255; i++ {
		gf256Exp[i] = byte(x)
		gf256Log[x] = byte(i)
		x = (x << 1) ^ x // multiply by generator 3 = (x+1) in GF(2^8)
		if x&0x100 != 0 {
			x ^= gf256ReductionPoly
		}
	}
	gf256Exp[255] = 1 // cyclic: exp[255]==exp[0]
}

func gfAdd(a, b byte) byte { return a ^ b }

func gfMul(a, b byte) byte {
	if a == 0 || b == 0 {
		return 0
	}
	s := uint16(gf256Log[a]) + uint16(gf256Log[b])
	if s >= 255 {
		s -= 255
	}
	return gf256Exp[s]
}

// gfInv: multiplicative inverse. Precondition a != 0 (panics otherwise —
// unreachable in the combine path, see combine.go / SPEC §4.4).
func gfInv(a byte) byte {
	if a == 0 {
		panic("slip39: gfInv(0)")
	}
	return gf256Exp[(255-uint16(gf256Log[a]))%255]
}

// gfDiv: a/b = a*inv(b). Precondition b != 0.
func gfDiv(a, b byte) byte { return gfMul(a, gfInv(b)) }
```

- [ ] **Step 4:** Run → PASS. `go vet ./slip39/`, `gofmt -l slip39/gf256.go` clean.
- [ ] **Step 5: Commit** `git add slip39/gf256.go slip39/gf256_test.go` → `feat: slip39 GF(256) field (port of toolkit gf256.rs)`.

> Cross-check note for the implementer: `gfMul(0x53,0xCA)==1` is the canonical AES S-box
> inverse pair; if your tables are built with generator 2 instead of 3 this fails — that is
> the exact bug the recon flagged. Verify against `gf256.rs:33-48`.

---

## Task 2: `slip39/lagrange.go` — interpolation

**Files:** Create `slip39/lagrange.go`, `slip39/lagrange_test.go`. Port of `lagrange.rs`.

- [ ] **Step 1: Failing test:**

```go
package slip39

import (
	"bytes"
	"testing"
)

func TestInterpolateConstantAndLinear(t *testing.T) {
	// Degree-0 (threshold 1): f(x)=0x42 everywhere.
	pts := []point{{1, 0x42}}
	if got := interpolateAt(pts, 255); got != 0x42 {
		t.Errorf("constant interp = %#x want 0x42", got)
	}
	// Multi-byte: two points define a line; recover at a third x.
	bp := []bytePoint{{1, []byte{0x01, 0x02}}, {2, []byte{0x03, 0x04}}}
	got := interpolateSecretAt(bp, 255)
	if len(got) != 2 {
		t.Fatalf("len=%d want 2", len(got))
	}
	_ = bytes.Equal // exact bytes asserted via the combine vectors (Task 5/6)
}
```

- [ ] **Step 2:** Run → FAIL (undefined `point`).

- [ ] **Step 3: Implement** `slip39/lagrange.go` (port of `lagrange.rs:37-91`):

```go
package slip39

// Lagrange interpolation over GF(256). Port of mnemonic_toolkit::slip39::lagrange.

const (
	secretIndex = 255 // SLIP-0039 SECRET_INDEX
	digestIndex = 254 // SLIP-0039 DIGEST_INDEX
	digestLen   = 4
)

type point struct{ x, y byte }
type bytePoint struct {
	x byte
	y []byte
}

// interpolateAt evaluates the degree-(len-1) polynomial through points at x.
// Precondition: all points[i].x distinct (enforced upstream; see SPEC §4.4).
func interpolateAt(points []point, x byte) byte {
	var result byte
	for i := range points {
		xi, yi := points[i].x, points[i].y
		num, den := byte(1), byte(1)
		for j := range points {
			if i == j {
				continue
			}
			xj := points[j].x
			num = gfMul(num, gfAdd(x, xj))
			den = gfMul(den, gfAdd(xi, xj))
		}
		result = gfAdd(result, gfMul(yi, gfDiv(num, den)))
	}
	return result
}

// interpolateSecretAt interpolates each byte position independently.
// All points[i].y must be equal length (enforced upstream).
func interpolateSecretAt(points []bytePoint, x byte) []byte {
	n := len(points[0].y)
	out := make([]byte, n)
	pb := make([]point, len(points))
	for k := 0; k < n; k++ {
		for i := range points {
			pb[i] = point{points[i].x, points[i].y[k]}
		}
		out[k] = interpolateAt(pb, x)
	}
	return out
}
```

- [ ] **Step 4:** Run → PASS; vet/gofmt clean.
- [ ] **Step 5: Commit** → `feat: slip39 GF(256) Lagrange interpolation (port of lagrange.rs)`.

---

## Task 3: `slip39/share.go` — value extraction + all lengths + structural check

**Files:** Modify `slip39/share.go`, `slip39/share_test.go`. Ports `share.rs:198-369`.

Current `ParseShare` (read it: `slip39/share.go:81-122`) decodes the header into a `uint64`,
verifies RS1024, and returns the `Share` WITHOUT a value. Extend it.

- [ ] **Step 1: Failing test** — add to `slip39/share_test.go`. **All shares come from the
  testdata loader (Task 0) — NO inline mnemonic literals (plan-R0 I1).**

```go
func TestParseShareExtractsValue(t *testing.T) {
	s, err := ParseShare(vectorShare(t, 3, 0)) // official idx 3, 128-bit/20-word
	if err != nil {
		t.Fatalf("ParseShare: %v", err)
	}
	if len(s.Value) != 16 {
		t.Errorf("Value len=%d want 16 (128-bit)", len(s.Value))
	}
	// Long path: idx 35 is 256-bit/33-word.
	s32, err := ParseShare(vectorShare(t, 35, 0))
	if err != nil {
		t.Fatalf("ParseShare(33-word): %v", err)
	}
	if len(s32.Value) != 32 {
		t.Errorf("Value len=%d want 32 (256-bit)", len(s32.Value))
	}
}

func TestParseShareGroupThresholdExceedsCount(t *testing.T) {
	_, err := ParseShare(vectorShare(t, 9, 0)) // official idx 9 — group thr > count
	if !errors.Is(err, errGroupThresholdExceedsCount) {
		t.Errorf("want errGroupThresholdExceedsCount, got %v", err)
	}
}
```

> NOTE to implementer (plan-R0 M3): **DELETE** (do not flip-to-expect-nil) the existing
> `slip39/share_test.go` assertion that a 33-word input returns `errUnsupportedSize` (it fed
> junk `"duckling"×33` which now fails RS1024, not parses clean) and the `Describe`
> `{errUnsupportedSize, "256-bit not supported"}` case — the symbol is removed.

- [ ] **Step 2:** Run → FAIL (`s.Value` undefined / 33-word still rejected).

- [ ] **Step 3: Implement.** In `slip39/share.go`:
  1. Add to the var block: `errBadPadding = errors.New("slip39: bad padding")` and
     `errGroupThresholdExceedsCount = errors.New("slip39: group threshold exceeds count")`.
     **Remove** `errUnsupportedSize`.
  2. Add field `Value []byte` to `Share` (after `Mnemonic`).
  3. Replace the `switch len(fields)` length gate with the **explicit** accepted set (M4 —
     canonical form): accept word count ∈ {20,23,27,30,33}; else `errWrongLength`. Then derive
     `valueWords := len(fields)-7`, `padBits := (10*valueWords)%16` (always ≤8 for these five),
     `valueBytes := (10*valueWords-padBits)/8` ∈ {16,20,24,28,32}. **Also delete the now-unused
     `wordsShort`/`wordsLong` consts (`share.go:31-32`) — plan-R0 M1.**
  4. After the RS1024 check and header decode, add the structural check (port `share.rs:248-256`):
     `if groupCount < groupThreshold { return Share{}, errGroupThresholdExceedsCount }`
     (compute these from the header BEFORE building the struct).
  5. Extract the value (port `share.rs:260-262,338-369`):

```go
// decodeValue unpacks value words (10-bit, big-endian, left-padded with
// padBits zeros) into valueBytes bytes; returns false if a leading pad bit
// is set. Byte-oriented (no value-wide accumulator) — TinyGo int is 32-bit.
func decodeValue(valueWords []int, padBits, valueBytes int) ([]byte, bool) {
	getBit := func(i int) byte {
		w := valueWords[i/10] & 0x3ff
		return byte((w >> (9 - i%10)) & 1)
	}
	for i := 0; i < padBits; i++ {
		if getBit(i) != 0 {
			return nil, false
		}
	}
	out := make([]byte, valueBytes)
	for bi := range out {
		var b byte
		for j := 0; j < 8; j++ {
			b = (b << 1) | getBit(padBits+bi*8+j)
		}
		out[bi] = b
	}
	return out, true
}
```
  Wire it into `ParseShare`: `valueWords := indices[4 : len(indices)-3]`; compute
  `padBits`/`valueBytes`; `val, ok := decodeValue(valueWords, padBits, valueBytes)`; if `!ok`
  return `errBadPadding`; set `Value: val` in the returned `Share`.
  6. Extend `Describe`: add cases for `errBadPadding` → `"bad padding"`,
     `errGroupThresholdExceedsCount` → `"group threshold exceeds count"`. Remove the
     `errUnsupportedSize` case.

- [ ] **Step 4:** Run `…/go test ./slip39/ -run 'TestParseShare|TestDescribe'` → PASS.
  `go vet ./slip39/`, `gofmt -l slip39/share.go slip39/share_test.go` clean. (The loader
  helpers + `testdata/slip39_vectors.json` already exist from Task 0 — load all shares via
  `vectorShare`; do NOT hand-type any mnemonic literal.)
- [ ] **Step 5: Commit** → `feat: slip39 share value extraction + all valid lengths + group-threshold check`.

---

## Task 4: `slip39/feistel.go` — 4-round Feistel decrypt

**Files:** Create `slip39/feistel.go`, `slip39/feistel_test.go`. Port of `feistel.rs` decrypt path.

- [ ] **Step 1: Failing test** — exercised end-to-end by the combine vectors (Task 5), but pin
  the salt + iteration formula directly:

```go
func TestFeistelSaltAndIters(t *testing.T) {
	// itersPerRound = (10000<<e)/4
	if got := itersPerRound(0); got != 2500 {
		t.Errorf("e=0 -> %d want 2500", got)
	}
	if got := itersPerRound(1); got != 5000 {
		t.Errorf("e=1 -> %d want 5000", got)
	}
	if !bytesEqual(feistelSalt(0x1234, false), append([]byte("shamir"), 0x12, 0x34)) {
		t.Errorf("non-extendable salt = shamir||be16(id)")
	}
	if len(feistelSalt(0x1234, true)) != 0 {
		t.Errorf("extendable salt is empty")
	}
}
```

- [ ] **Step 2:** Run → FAIL.

- [ ] **Step 3: Implement** `slip39/feistel.go` (port of `feistel.rs:88-206`, decrypt only):

```go
package slip39

import (
	"crypto/sha256"
	"encoding/binary"

	"golang.org/x/crypto/pbkdf2"
)

const (
	feistelRounds        = 4
	feistelBaseIterCount = 10000
)

func itersPerRound(iterationExp int) int {
	return (feistelBaseIterCount << uint(iterationExp)) / feistelRounds
}

func feistelSalt(identifier int, extendable bool) []byte {
	if extendable {
		return nil
	}
	salt := make([]byte, 0, 8)
	salt = append(salt, []byte("shamir")...)
	var idb [2]byte
	binary.BigEndian.PutUint16(idb[:], uint16(identifier))
	return append(salt, idb[:]...)
}

// feistelDecrypt turns the encrypted master secret (EMS) into the master
// secret via the 4-round Feistel run in reverse (rounds 3,2,1,0). Output is
// R||L. Port of feistel.rs decrypt. The passphrase enters ONLY here.
func feistelDecrypt(ems, passphrase []byte, iterationExp, identifier int, extendable bool) []byte {
	n := len(ems)
	half := n / 2
	l := append([]byte(nil), ems[:half]...)
	r := append([]byte(nil), ems[half:]...)
	salt := feistelSalt(identifier, extendable)
	iters := itersPerRound(iterationExp)
	for i := feistelRounds - 1; i >= 0; i-- {
		pw := append([]byte{byte(i)}, passphrase...)
		f := pbkdf2.Key(pw, append(append([]byte(nil), salt...), r...), iters, half, sha256.New)
		for j := 0; j < half; j++ {
			l[j] ^= f[j]
		}
		wipe(pw)
		wipe(f)
		l, r = r, l // swap
	}
	out := append(append([]byte(nil), r...), l...)
	wipe(l)
	wipe(r)
	return out
}
```

> Cross-check `feistel.rs:134-166`: round order `[3,2,1,0]`, body `l[j]^=f[j]` then
> `swap(l,r)`, output `r||l`. `wipe` is defined in combine.go (Task 5) — if Task 5 hasn't
> landed, define `wipe` here and move it. Provide `bytesEqual` test helper or use `bytes.Equal`.

- [ ] **Step 4:** Run → PASS; vet/gofmt clean.
- [ ] **Step 5: Commit** → `feat: slip39 4-round Feistel decrypt (port of feistel.rs)`.

---

## Task 5: `slip39/combine.go` — two-level recovery

**Files:** Create `slip39/combine.go`, `slip39/combine_test.go`. Port of `mod.rs:206-458`.

- [ ] **Step 1: Failing test** — the primary positive vector (idx 3, passphrase "TREZOR"):

```go
func TestCombineBasic2of3(t *testing.T) {
	shares := vectorShares(t, 3) // all mnemonics of official vector idx 3
	parsed := make([]Share, len(shares))
	for i, m := range shares {
		s, err := ParseShare(m)
		if err != nil { t.Fatalf("share %d: %v", i, err) }
		parsed[i] = s
	}
	got, err := Combine(parsed[:2], []byte("TREZOR")) // any 2 of 3
	if err != nil { t.Fatalf("Combine: %v", err) }
	if hexEq(got) != "b43ceb7e57a0ea8766221624d01b0864" {
		t.Errorf("recovered %x want b43c…0864", got)
	}
}
```

- [ ] **Step 2:** Run → FAIL (undefined `Combine`).

- [ ] **Step 3: Implement** `slip39/combine.go` (port of `mod.rs` `slip39_combine`
  `:206-331` + `recover_secret` `:430-458`). New error sentinels (errors.New) +:

```go
package slip39

import (
	"crypto/hmac"
	"crypto/sha256"
	"crypto/subtle"
	"errors"
	"sort"
)

var (
	errEmptyShares              = errors.New("slip39: no shares")
	errInvalidShareValueLength  = errors.New("slip39: invalid share value length")
	errIdentifierMismatch       = errors.New("slip39: identifier mismatch")
	errExtendableMismatch       = errors.New("slip39: extendable mismatch")
	errIterationExponentMismatch = errors.New("slip39: iteration exponent mismatch")
	errGroupThresholdMismatch   = errors.New("slip39: group threshold mismatch")
	errGroupCountMismatch       = errors.New("slip39: group count mismatch")
	errShareValueLengthMismatch = errors.New("slip39: share value length mismatch")
	errMemberThresholdMismatch  = errors.New("slip39: member threshold mismatch")
	errDuplicateMemberIndex     = errors.New("slip39: duplicate member index")
	errInsufficientShares       = errors.New("slip39: not enough shares")
	errDigestVerificationFailed = errors.New("slip39: bad share set")
)

func wipe(b []byte) {
	for i := range b {
		b[i] = 0
	}
}

func validSecretLen(n int) bool {
	switch n {
	case 16, 20, 24, 28, 32:
		return true
	}
	return false
}

// Combine reconstructs the SLIP-39 master secret from a set of shares.
// passphrase is the SLIP-39 EMS-decryption passphrase ("" = none). Returns
// the master-secret bytes (BIP-39 entropy sizes) or a classifiable error.
// Port of mnemonic_toolkit::slip39::slip39_combine. Panic-free on any input
// (all preconditions checked before interpolation — SPEC §4.4).
func Combine(shares []Share, passphrase []byte) ([]byte, error) {
	if len(shares) == 0 {
		return nil, errEmptyShares
	}
	for i := range shares {
		if !validSecretLen(len(shares[i].Value)) {
			return nil, errInvalidShareValueLength
		}
	}
	first := shares[0]
	for _, s := range shares[1:] {
		switch {
		case s.Identifier != first.Identifier:
			return nil, errIdentifierMismatch
		case s.Extendable != first.Extendable:
			return nil, errExtendableMismatch
		case s.IterationExp != first.IterationExp:
			return nil, errIterationExponentMismatch
		case s.GroupThreshold != first.GroupThreshold:
			return nil, errGroupThresholdMismatch
		case s.GroupCount != first.GroupCount:
			return nil, errGroupCountMismatch
		case len(s.Value) != len(first.Value):
			return nil, errShareValueLengthMismatch
		}
	}
	// Group by GroupIndex (sorted keys for determinism).
	byGroup := map[int][]Share{}
	for _, s := range shares {
		byGroup[s.GroupIndex] = append(byGroup[s.GroupIndex], s)
	}
	gids := make([]int, 0, len(byGroup))
	for g := range byGroup {
		gids = append(gids, g)
	}
	sort.Ints(gids)

	type gshare struct {
		x byte
		v []byte
	}
	groupShares := make([]gshare, 0, len(gids))
	for _, g := range gids {
		gs := byGroup[g]
		mt := gs[0].MemberThreshold
		seen := map[int]bool{}
		for _, s := range gs {
			if s.MemberThreshold != mt {
				return nil, errMemberThresholdMismatch
			}
			if seen[s.MemberIndex] {
				return nil, errDuplicateMemberIndex
			}
			seen[s.MemberIndex] = true
		}
		if len(gs) != mt {
			return nil, errInsufficientShares
		}
		pts := make([]bytePoint, len(gs))
		for i, s := range gs {
			pts[i] = bytePoint{byte(s.MemberIndex), s.Value}
		}
		gv, err := recoverSecret(mt, pts)
		if err != nil {
			return nil, err
		}
		groupShares = append(groupShares, gshare{byte(g), gv})
	}
	if len(groupShares) != first.GroupThreshold {
		return nil, errInsufficientShares
	}
	gpts := make([]bytePoint, len(groupShares))
	for i, gs := range groupShares {
		gpts[i] = bytePoint{gs.x, gs.v}
	}
	ems, err := recoverSecret(first.GroupThreshold, gpts)
	if err != nil {
		return nil, err
	}
	master := feistelDecrypt(ems, passphrase, first.IterationExp, first.Identifier, first.Extendable)
	for _, gs := range groupShares {
		wipe(gs.v)
	}
	wipe(ems)
	return master, nil
}

// recoverSecret recovers one Shamir layer. threshold==1 → the single value
// (no digest). Else interpolate at 255/254 and verify the HMAC-SHA256 digest.
func recoverSecret(threshold int, shares []bytePoint) ([]byte, error) {
	if threshold == 1 {
		return append([]byte(nil), shares[0].y...), nil
	}
	s := interpolateSecretAt(shares, secretIndex)
	d := interpolateSecretAt(shares, digestIndex)
	digest, random := d[:digestLen], d[digestLen:]
	mac := hmac.New(sha256.New, random)
	mac.Write(s)
	sum := mac.Sum(nil)
	if subtle.ConstantTimeCompare(digest, sum[:digestLen]) != 1 {
		wipe(s)
		return nil, errDigestVerificationFailed
	}
	wipe(d)
	return s, nil
}

// ConsistentShares reports whether a partial share set is mutually
// consistent (for eager GUI validation; count-agnostic). Two-level.
func ConsistentShares(shares []Share) error {
	if len(shares) == 0 {
		return nil
	}
	first := shares[0]
	type gm struct{ g, m int }
	seen := map[gm]bool{}
	for _, s := range shares {
		switch {
		case s.Identifier != first.Identifier:
			return errIdentifierMismatch
		case s.Extendable != first.Extendable:
			return errExtendableMismatch
		case s.IterationExp != first.IterationExp:
			return errIterationExponentMismatch
		case s.GroupThreshold != first.GroupThreshold:
			return errGroupThresholdMismatch
		case s.GroupCount != first.GroupCount:
			return errGroupCountMismatch
		case len(s.Value) != len(first.Value):
			return errShareValueLengthMismatch
		}
		k := gm{s.GroupIndex, s.MemberIndex}
		if seen[k] {
			return errDuplicateMemberIndex
		}
		seen[k] = true
	}
	return nil
}
```
  Extend `Describe` (in share.go) for the new combine sentinels per SPEC §6.

- [ ] **Step 4:** Run `…/go test ./slip39/ -run TestCombine` → PASS; vet/gofmt clean.
- [ ] **Step 5: Commit** → `feat: slip39 two-level Combine + recoverSecret + ConsistentShares`.

---

## Task 6: vectors, Rust-fixture round-trips, negatives, panic-safety, scrub

**Files:** Create `slip39/testdata/slip39_fixtures.json` + `slip39/vectors_test.go`'s
round-trip/negative/panic/scrub tests. Task 6 adds ONLY those. The testdata vectors file
(`slip39_vectors.json`) + loader helpers (`vectorShare`/`vectorShares`/`vectorSecretHex`) were
created in Task 0; `hexEq` lives in `combine_test.go` (Task 5).

- [ ] **Step 1:** `slip39/testdata/slip39_vectors.json` + the loader helpers were created in
  **Task 0 Steps 3–4** (front-loaded per plan-R0 I1). Confirm they exist and that Tasks 3/5
  already load from them; nothing to copy here.

- [ ] **Step 2: Generate intermediate-length fixtures** into `slip39/testdata/slip39_fixtures.json`
  via the Rust oracle (reproducible). The toolkit's deterministic test wedge lives in the CLI
  layer (`MNEMONIC_SLIP39_TEST_RNG` 32-byte-hex seed + `MNEMONIC_SLIP39_TEST_IDENTIFIER`); a
  seeded `slip39_split` call achieves the same. Generate over master-secret lengths
  {16,20,24,28,32} × topologies {1-of-1, 2-of-3, group(2-of-3 over 2-of-3 groups)} with FIXED
  seed + identifier, emitting `{secret_hex, passphrase, mnemonics[]}` per case. Document the
  exact regeneration command in `slip39/testdata/GEN.md`. **This is the only path that covers
  the 23/27/30-word (160/192/224-bit) shares** (the static corpus has only 128/256-bit). Note
  `extendable` is hardcoded `false` on the CLI wedge path, so ext=1 coverage comes from
  official idx 42 (Step 3), not fixtures. Commit the generated JSON; do NOT port `split` into
  the firmware (test-fixtures only).

- [ ] **Step 3: Positive round-trip test** (vectors + fixtures): for each, parse all shares,
  `Combine` a threshold subset, assert the recovered hex == expected. Include the idx-3
  `"TREZOR"` vs `""` distinct-secret assertion and the 33-word (idx 35) + 23/27/30-word
  (fixtures) length coverage.

- [ ] **Step 4: Negative test:** idx 1 (`errBadChecksum` @ParseShare), idx 9
  (`errGroupThresholdExceedsCount` @ParseShare), idx 4 (`errInsufficientShares`), idx 5
  (`errIdentifierMismatch`), idx 13 (`errInsufficientShares`), idx 12 (`errDigestVerificationFailed`).

- [ ] **Step 5: Panic-safety test** (SPEC §4.4): construct a malformed set — duplicate
  `(group,member)` with valid-length values, and a set whose interpolation would hit a dup x —
  assert `Combine` returns an error and NEVER panics (wrap in a `func(){defer recover…}()` that
  fails the test if it recovers a panic).

- [ ] **Step 6: Scrub test** (SPEC §4.8): assert the `wipe` path is exercised — e.g. a
  `recoverSecret` digest-fail path wipes `s` (use a small test seam or assert via a wrapper).
  Keep it simple: a unit test that a known buffer passed through an internal helper is zeroed.

- [ ] **Step 7:** Run the FULL guard: `…/go test ./slip39/ ./gui/ ./bip39/`, `go vet ./slip39/`,
  `gofmt -l slip39/`. All green/clean. The existing `gui` SLIP-39 tests
  (`TestConfirmSLIP39Render`, `TestEngraveSLIP39BackoutRecognized`) MUST stay green (D1 doesn't
  touch the GUI; `Share` gained a field but `engraveSLIP39`/`confirmSLIP39Flow` ignore `Value`).
- [ ] **Step 8: Commit** → `test: slip39 recovery vectors + Rust-fixture round-trips + negatives + panic-safety`.

---

## Self-review checklist (run before handing to the execution review)

- Every vector/negative/length from SPEC §7 has a test; the 23/27/30-word lengths come from
  the Rust-generated fixtures (the static corpus lacks them) — and that gap is NOT silently
  skipped.
- **No inline crypto literals (plan-R0 I1):** every share mnemonic and master-secret hex in a
  test comes from `testdata/slip39_vectors.json` (verbatim upstream) or `slip39_fixtures.json`
  (Rust-generated) via the loader helpers — `grep` the test files for hand-typed 20+-word
  strings and `[0-9a-f]{32}` literals → none (except the loader/testdata).
- No `math/big` in any new file (`grep -rn 'math/big' slip39/` → only the unchanged… none).
- Generator is **3** (gf256 test pins the AES inverse pair); Feistel is `[3,2,1,0]`→`R||L`;
  salt is `"shamir"||be16(id)`/empty; digest compared with `subtle.ConstantTimeCompare`.
- `ParseShare` accepts {20,23,27,30,33}; `errUnsupportedSize` fully removed (no dangling refs);
  the old `errUnsupportedSize` 256-bit-reject test + its `Describe` case DELETED (not flipped).
- Panic-safety test present and green; no `gfInv(0)`/dup-x path reachable from `Combine` input.
- `Share` gained `Value` but the GUI guards stay green.
- Signed + DCO + Brian Goss on every commit.
