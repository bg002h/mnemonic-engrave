# Hashlock H2 — Device Leg Implementation Plan (SeedHammer fork)

**STATUS: DRAFT 2026-09-05 — build gate not yet run; R0 not yet dispatched.**

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** On the SeedHammer II, type a hashlock phrase, pick the method, and set a
spend path's hash to the SAME digest `ms hashlock` derives on the host — deriving,
using and dropping the preimage; never storing, showing or engraving it.

**Architecture:** A new fork package `hashlock` ports `ms_codec::hashlock` (constants,
both derivations, the digest, the phrase rule, the host's ms1-shape test) with the
ms-codec 0.8.0 corpus vendored and pinned, and drives PBKDF2 through the fork's
existing stepwise `seal.NewDeriver` with the 14-byte salt as a slice. `codex32`
gains `DecodeMS1Preimage`. The composer's `Which hash?` becomes a label-keyed
switch with a new row that runs a loop — phrase screen → method pick (+ modal) →
derivation → hold-to-confirm — and assigns the digest only at the end. Copy lands
in `composer_copy.go` and is added to both copy gates. The emulator walk and the
firmware size close the stage; the flash is the operator's.

**Tech Stack:** Go 1.26 (`/scratch/code/shibboleth/.toolchain/go/bin/go`), the fork's
`gui` touch harness (`runUITouch`, `sessionHarness`), TinyGo via
`nix develop -c tinygo build …` for size, `cmd/emu` for the walk.

**Spec:** `design/SPEC_hashlock_H2_device.md` (R0 GREEN at `55ee7a4`) — §2 the phrase
rule, §3 the port, §4 screens and copy, §4.6 the Back contract, §5 the switch, §6 the
decoder, §7 tests, §8 acceptance. Corpus: ms `crates/ms-codec/tests/vectors/hashlock-v0.8.json`
at `cd0a60f`, sha256 `a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30`.

**Baselines (for `scripts/plan-staleness-check.sh`):** seedhammer fork main `c4a64fc`;
mnemonic-secret `cd0a60f`; mnemonic-engrave `e1c23e9`.

## Global Constraints

- **Rust-primary (CLAUDE.md):** every constant, derivation and rule below converges on
  ms-codec 0.8.0 / ms-cli 0.18.0; the corpus is the arbiter, compared against its
  CONSTANTS (never a value this Go recomputed). The provenance pin names ms `cd0a60f`.
- **The phrase is never normalised** (spec §2): no `seal.NormalisePassphrase`,
  `strings.TrimSpace/Fields/ToLower/ToUpper` or Unicode normalisation on the typed
  bytes; the shape test (rule 3) works on a COPY. The lockstep test's mutations
  (§7.1) prove it with the rows that are not fixed points.
- **The salt is a 14-byte slice through `seal.NewDeriver`** (`seal/pbkdf2.go:85`,
  `NewDeriver(passphrase, salt []byte, iterations int)`); `unlockDerive` and
  `seal.Header` are NOT used (its `Salt [16]byte` would zero-pad — spec §3).
  `Deriver.Key()` returns 32 bytes; `Wipe()` after use.
- **The Back contract (spec §4.6):** `composerHashEdit` returns `false` ONLY for Back
  at `Which hash?`; every inner Back moves within the loop with the phrase intact.
  At path creation `false` deletes the path (`gui/composer_shape.go:269`), so the
  tests run through `composerAddPath` and assert the path survives.
- **No new class; nothing calls `DecodeMS1Preimage` from a screen** this stage.
- **Copy is ASCII and gated twice:** every new string is a `composerCopy*` function in
  `gui/composer_copy.go`, a row in `composerCopyTable()` (`composer_copy_test.go:29`)
  and a row in `TestModalsThisBlockTouchesAreDrawnInFull` (`modal_fits_test.go:301`);
  `assertModalBodyFits` (per-body, margin 80) is the fit gate — no capacity number.
- **Secret-handling defects never gate** (operator ruling 2026-08-27).
- **Fork commits** `git commit -s` (DCO), author Brian Goss, branch `hashlock-h2` off
  fork `main` `c4a64fc`; trailers as this repo uses; stage paths explicitly.
- **Flash only via `~/bin/sh/sh2-flash -y` at the operator's word**; never picotool by hand.

---

## File Structure

| File | Change | Responsibility |
| --- | --- | --- |
| `hashlock/hashlock.go` | Create | constants; `PreimageHardened`, `PreimageSHA256`, `Digest`, `DeriveHardened`; `ValidatePhrase`, `IsMS1Shaped`, `PhraseMaxChars` |
| `hashlock/hashlock_test.go` | Create | the lockstep gate: sha pin, 11 derivation rows, 15 refusals rows, the kind row, mutations |
| `hashlock/testdata/hashlock-v0.8.json`, `hashlock-v0.8.provenance.json` | Create (vendored) | byte-identical corpus + pin |
| `codex32/mspayload.go` | Modify (append) | `DecodeMS1Preimage` |
| `codex32/mspayload_test.go` | Modify (append) | §7.4 |
| `gui/composer_hash.go` | Modify | header comment (spec §1 item 5); `composerHashEdit` label-keyed with the phrase row and the loop; `composerHashRows` |
| `gui/composer_hashlock.go` | Create | `hashlockPhraseFlow`, `hashlockMethodPick`, `hashlockDeriveFlow`, `hashlockConfirmBody`, the relation line |
| `gui/composer_copy.go` | Modify (append) | `composerCopyHashlock*` strings; `composerCopyHashEveryPathPhrase` |
| `gui/composer_shape.go` | Modify (one line) | Done's §8h picks the phrase-route form when a hash was set by phrase |
| `gui/composer_state.go` | Modify | `composerState.hashByPhrase bool` (set by the phrase route) |
| `gui/composer_copy_test.go`, `gui/modal_fits_test.go` | Modify (rows) | the two copy gates |
| `gui/composer_hash_test.go` | Modify (append) | §7.3 switch tests |
| `gui/composer_hashlock_test.go` | Create | §7.2 harness tests |
| `cmd/emu/walk_hashlock_phrase.js` | Create | §7.5 |
| engrave `design/FOLLOWUPS.md`, continuity | Modify | records |

**Gate coverage.** `scripts/plan-build-gate-go.sh` recognises `gui/composer_*.go` (so
`gui/composer_hashlock.go` is assembled) but not `hashlock/*.go` or `codex32/*.go`;
the controller hand-wires the whole plan into a scratch copy of the fork and runs
`go vet` + `go test ./hashlock/ ./codex32/ ./sysw/ ./seal/` + `go test -run
'TestComposer|TestHashlock' ./gui/` before review, then the gui shard script; output in
the plan commit. Whole-package gui runs use `scripts/gui-shard-test.sh` (engrave).

---

### Task 1: The `hashlock` package and its lockstep gate

**Files:**
- Create: `hashlock/hashlock.go`, `hashlock/hashlock_test.go`
- Create: `hashlock/testdata/hashlock-v0.8.json` (copy of ms `crates/ms-codec/tests/vectors/hashlock-v0.8.json` at `cd0a60f`), `hashlock/testdata/hashlock-v0.8.provenance.json`

**Interfaces:**
- Consumes: `seal.NewDeriver(passphrase, salt []byte, iterations int) *Deriver`, `(*Deriver).Step(n int) bool`, `Done()`, `Total()`, `Key() []byte` (32 bytes), `Wipe()`; `crypto/sha256`.
- Produces: `hashlock.Salt`, `Iterations`, `PreimageLen`, `PhraseMaxChars`, `PreimageHardened([]byte) [32]byte`, `PreimageSHA256([]byte) [32]byte`, `Digest(*[32]byte) [32]byte`, `DeriveHardened(phrase []byte, progress func(done, total int) bool) ([32]byte, bool)`, `ValidatePhrase([]byte) error` with the sentinel errors `ErrEmpty`, `ErrNotPrintableASCII`, `ErrMS1Shaped`, `ErrTooLong`, `ErrHex64`, and `IsMS1Shaped(string) bool`.

- [ ] **Step 1: Vendor the corpus and write the pin.**

```bash
mkdir -p hashlock/testdata
cp /scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/tests/vectors/hashlock-v0.8.json hashlock/testdata/
sha256sum hashlock/testdata/hashlock-v0.8.json   # a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30
```

Create `hashlock/testdata/hashlock-v0.8.provenance.json` in the shape of `sysw/testdata/record_class_vectors.provenance.json`:

```json
{
  "_comment": [
    "PROVENANCE PIN for the vendored copy of ms-codec 0.8.0's hashlock corpus (SPEC_hashlock_H2_device §7.1).",
    "The rows are MEASURED constants (python3 hashlib + openssl kdf) recorded by the H1 plan; never edited by hand.",
    "hashlock/hashlock_test.go fails if sha256 and the file disagree or if any derivation row disagrees with the Go port.",
    "TO RE-SYNC: cp ../mnemonic-secret/crates/ms-codec/tests/vectors/hashlock-v0.8.json hashlock/testdata/ ;",
    "  git -C ../mnemonic-secret rev-parse HEAD ; sha256sum hashlock/testdata/hashlock-v0.8.json"
  ],
  "repo": "mnemonic-secret",
  "remote": "git@github.com:bg002h/mnemonic-secret.git",
  "path": "crates/ms-codec/tests/vectors/hashlock-v0.8.json",
  "commit": "cd0a60f",
  "release": "ms-codec-v0.8.0 (crates.io 0.8.0, 2026-09-05)",
  "sha256": "a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30",
  "derivation_rows": 11,
  "refusals_rows": 15,
  "recorded_at": "2026-09-05"
}
```

- [ ] **Step 2: The failing tests.** Create `hashlock/hashlock_test.go`:

```go
package hashlock

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"os"
	"strings"
	"testing"
)

const corpusPath = "testdata/hashlock-v0.8.json"
const corpusSHA256 = "a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30"

type corpus struct {
	Derivation []struct {
		Phrase      string `json:"phrase"`
		PhraseChars int    `json:"phrase_chars"`
		HardenedX   string `json:"hardened_x"`
		HardenedH   string `json:"hardened_h"`
		SHA256X     string `json:"sha256_x"`
		SHA256H     string `json:"sha256_h"`
	} `json:"derivation"`
	Refusals []struct {
		Input         *string `json:"input"`
		InputBytesHex *string `json:"input_bytes_hex"`
		Channel       string  `json:"channel"`
		Rule          *string `json:"rule"`
		Remedy        *string `json:"remedy"`
		Note          *string `json:"note"`
	} `json:"refusals"`
	Kind []struct {
		PreimageHex string `json:"preimage_hex"`
		MS1         string `json:"ms1"`
	} `json:"kind"`
	Lockstep []string `json:"lockstep"`
}

func loadCorpus(t *testing.T) corpus {
	t.Helper()
	raw, err := os.ReadFile(corpusPath)
	if err != nil {
		t.Fatalf("%s: %v", corpusPath, err)
	}
	if sum := sha256.Sum256(raw); hex.EncodeToString(sum[:]) != corpusSHA256 {
		t.Fatalf("%s hashes to %x, not the pinned %s -- the vendored copy and ms-codec 0.8.0's have drifted",
			corpusPath, sum, corpusSHA256)
	}
	var c corpus
	if err := json.Unmarshal(raw, &c); err != nil {
		t.Fatalf("%s: %v", corpusPath, err)
	}
	if len(c.Derivation) != 11 || len(c.Refusals) != 15 || len(c.Kind) < 1 {
		t.Fatalf("corpus shape: %d derivation, %d refusals, %d kind rows", len(c.Derivation), len(c.Refusals), len(c.Kind))
	}
	return c
}

func mustHex(t *testing.T, s string) [32]byte {
	t.Helper()
	b, err := hex.DecodeString(s)
	if err != nil || len(b) != 32 {
		t.Fatalf("bad 32-byte hex %q: %v", s, err)
	}
	var out [32]byte
	copy(out[:], b)
	return out
}

// Every derivation row, both methods, compared against the corpus CONSTANTS.
// MUTATIONS: zero-pad Salt to 16 bytes -> every hardened row fails; Iterations
// 99999 -> every hardened row fails; NormalisePassphrase the phrase first ->
// the "Correct Horse Battery Staple" and "  a  b " rows fail; strip display
// separators first -> the "correct-horse,battery staple" row fails.
func TestDerivationRowsLockstep(t *testing.T) {
	c := loadCorpus(t)
	for _, r := range c.Derivation {
		phrase := []byte(r.Phrase)
		if len(phrase) != r.PhraseChars {
			t.Errorf("%q: %d bytes, corpus says %d", r.Phrase, len(phrase), r.PhraseChars)
		}
		x := PreimageHardened(phrase)
		if want := mustHex(t, r.HardenedX); x != want {
			t.Errorf("%q hardened X: got %x want %x", r.Phrase, x, want)
		}
		if h, want := Digest(&x), mustHex(t, r.HardenedH); h != want {
			t.Errorf("%q hardened H: got %x want %x", r.Phrase, h, want)
		}
		x2 := PreimageSHA256(phrase)
		if want := mustHex(t, r.SHA256X); x2 != want {
			t.Errorf("%q sha256 X: got %x want %x", r.Phrase, x2, want)
		}
		if h, want := Digest(&x2), mustHex(t, r.SHA256H); h != want {
			t.Errorf("%q sha256 H: got %x want %x", r.Phrase, h, want)
		}
		// The stepwise driver derives the same bytes as the one-shot function.
		if d, ok := DeriveHardened(phrase, func(int, int) bool { return true }); !ok || d != x {
			t.Errorf("%q DeriveHardened != PreimageHardened (ok=%v)", r.Phrase, ok)
		}
	}
}

// The three rows that are NOT fixed points of the folds spec §2 forbids exist in
// the corpus; without them the mutations above could not fail.
func TestCorpusCarriesTheNonFixedPointRows(t *testing.T) {
	c := loadCorpus(t)
	want := map[string]bool{"Correct Horse Battery Staple": false, "  a  b ": false, "correct-horse,battery staple": false}
	for _, r := range c.Derivation {
		if _, ok := want[r.Phrase]; ok {
			want[r.Phrase] = true
		}
	}
	for p, seen := range want {
		if !seen {
			t.Errorf("corpus has no derivation row %q", p)
		}
	}
}

// Every refusals row through ValidatePhrase; the ms1-shaped rows are built from
// kind[0].ms1 exactly as the corpus describes them.
func TestRefusalRowsMatchTheHost(t *testing.T) {
	c := loadCorpus(t)
	plate := c.Kind[0].MS1
	grouped := func(s string, n int) string {
		var b strings.Builder
		for i, r := range s {
			if i > 0 && i%n == 0 {
				b.WriteByte(' ')
			}
			b.WriteRune(r)
		}
		return b.String()
	}
	for i, r := range c.Refusals {
		var in []byte
		switch {
		case r.InputBytesHex != nil:
			b, err := hex.DecodeString(*r.InputBytesHex)
			if err != nil {
				t.Fatalf("row %d: %v", i, err)
			}
			in = b
		case r.Input != nil:
			s := *r.Input
			switch s {
			case "<the kind[0].ms1 string, lowercase>":
				s = strings.ToLower(plate)
			case "<the kind[0].ms1 string, UPPERCASE>":
				s = strings.ToUpper(plate)
			case "<the kind[0].ms1 string, grouped by 5 with spaces>":
				s = grouped(plate, 5)
			case "<the kind[0].ms1 string, with two leading and two trailing spaces>":
				s = "  " + plate + "  "
			case "<the kind[0].ms1 string, grouped by 2 (112 chars)>":
				s = grouped(plate, 2)
			}
			in = []byte(s)
		default:
			t.Fatalf("row %d has neither input nor input_bytes_hex", i)
		}
		err := ValidatePhrase(in)
		if r.Rule == nil {
			if err != nil {
				t.Errorf("row %d (%s) must be ACCEPTED, got %v", i, r.Channel, err)
			}
			continue
		}
		want := map[string]error{
			"empty": ErrEmpty, "printable-ascii": ErrNotPrintableASCII, "64-hex": ErrHex64,
			"ms1-shaped": ErrMS1Shaped, "too-long": ErrTooLong,
		}[*r.Rule]
		if want == nil {
			t.Fatalf("row %d: unknown rule %q", i, *r.Rule)
		}
		if err != want {
			t.Errorf("row %d rule %s: got %v want %v", i, *r.Rule, err, want)
		}
	}
}

// The kind row: the plate's preimage bytes are the corpus's preimage_hex; the
// digest of that preimage is what the confirm modal must show for a --hex X.
func TestKindRowPreimageDigest(t *testing.T) {
	c := loadCorpus(t)
	x := mustHex(t, c.Kind[0].PreimageHex)
	if h := Digest(&x); h == x {
		t.Fatalf("Digest is the identity")
	}
}

// PhraseMaxChars is the single source of the cap (mutation: change the literal
// in ValidatePhrase to 99 -> the 100-character corpus row is refused here).
func TestPhraseMaxCharsIsTheCap(t *testing.T) {
	if PhraseMaxChars != 100 {
		t.Fatalf("PhraseMaxChars = %d", PhraseMaxChars)
	}
	if err := ValidatePhrase([]byte(strings.Repeat("k", PhraseMaxChars))); err != nil {
		t.Errorf("100 characters must be accepted: %v", err)
	}
	if err := ValidatePhrase([]byte(strings.Repeat("k", PhraseMaxChars+1))); err != ErrTooLong {
		t.Errorf("101 characters: got %v want ErrTooLong", err)
	}
}

// The lockstep list is the corpus's own statement of what this file drives; if
// ms-codec adds a clause, this test names it so the port grows with it.
func TestLockstepListIsTheOneWeDrive(t *testing.T) {
	c := loadCorpus(t)
	if len(c.Lockstep) != 4 {
		t.Fatalf("lockstep clauses: %d (this test drives 4 -- read the new one)", len(c.Lockstep))
	}
}
```

- [ ] **Step 3: Run to see them fail.**

Run: `go test -count=1 ./hashlock/`
Expected: does not compile (`PreimageHardened` … undefined).

- [ ] **Step 4: The package.** Create `hashlock/hashlock.go`:

```go
// Package hashlock is the SeedHammer port of ms_codec::hashlock (ms-codec 0.8.0,
// mnemonic-secret cd0a60f): a memorable phrase becomes a 32-byte hashlock
// PREIMAGE X, and the digest H = SHA-256(X) is what a spend path's script holds.
//
// Rust is primary (CLAUDE.md): nothing here is decided in Go. The vendored corpus
// testdata/hashlock-v0.8.json pins every value, compared against its constants.
//
// SPEC_hashlock_H2_device §2, §3.
package hashlock

import (
	"crypto/sha256"
	"errors"
	"strings"

	"seedhammer.com/seal"
)

// Salt is HASHLOCK_SALT: fourteen bytes, passed to the KDF as a SLICE. Never
// through seal.Header's Salt [16]byte -- zero-padding it changes every digest.
var Salt = []byte("ms-hashlock-v1")

// Iterations is HASHLOCK_ITERATIONS; about 10 s on the SH2 (9,715 it/s measured).
const Iterations = 100000

// PreimageLen is HASHLOCK_DKLEN: a hashlock preimage is exactly 32 bytes (OP_SIZE 32).
const PreimageLen = 32

// PhraseMaxChars is ms-cli's HASHLOCK_PHRASE_MAX_CHARS: the counter's denominator
// and the rule's bound, from this one constant.
const PhraseMaxChars = 100

// The phrase rule's refusals, SPEC_ms_hashlock §4.3 / SPEC_hashlock_H2_device §2,
// in the order the rule checks them.
var (
	ErrEmpty             = errors.New("hashlock: the phrase is empty")
	ErrNotPrintableASCII = errors.New("hashlock: the phrase has a byte outside 0x20..=0x7E")
	ErrMS1Shaped         = errors.New("hashlock: that is a preimage plate, not a phrase")
	ErrTooLong           = errors.New("hashlock: the phrase is longer than 100 characters")
	ErrHex64             = errors.New("hashlock: that is a preimage in hex, not a phrase")
)

// PreimageHardened is preimage_hardened: PBKDF2-HMAC-SHA256(phrase, Salt,
// Iterations, 32). One shot; the screen uses DeriveHardened.
func PreimageHardened(phrase []byte) [32]byte {
	d := seal.NewDeriver(phrase, Salt, Iterations)
	defer d.Wipe()
	d.Step(Iterations)
	var out [32]byte
	copy(out[:], d.Key())
	return out
}

// DeriveHardened is PreimageHardened in steps, so a screen can show progress and
// honour Back: progress(done, total) is called after every 500 iterations and
// returns false to abandon (then ok is false and the result is zero).
func DeriveHardened(phrase []byte, progress func(done, total int) bool) (x [32]byte, ok bool) {
	d := seal.NewDeriver(phrase, Salt, Iterations)
	defer d.Wipe()
	for !d.Step(500) {
		if !progress(d.Done(), d.Total()) {
			return x, false
		}
	}
	copy(x[:], d.Key())
	return x, true
}

// PreimageSHA256 is preimage_sha256: one SHA-256 of the phrase bytes -- the
// brainwallet construction, warned about every time (L12).
func PreimageSHA256(phrase []byte) [32]byte {
	return sha256.Sum256(phrase)
}

// Digest is digest: H = SHA-256(X).
func Digest(x *[32]byte) [32]byte {
	return sha256.Sum256(x[:])
}

// ValidatePhrase applies SPEC_ms_hashlock §4.3 to the typed BYTES, in the host's
// order, and changes nothing: no trim, no case fold, no normalisation. The shape
// test works on a copy.
func ValidatePhrase(phrase []byte) error {
	if len(phrase) == 0 {
		return ErrEmpty
	}
	for _, b := range phrase {
		if b < 0x20 || b > 0x7e {
			return ErrNotPrintableASCII
		}
	}
	if IsMS1Shaped(string(phrase)) {
		return ErrMS1Shaped
	}
	if len(phrase) > PhraseMaxChars {
		return ErrTooLong
	}
	if len(phrase) == 64 && isHex(phrase) {
		return ErrHex64
	}
	return nil
}

const bech32Charset = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"

// minMS1Len is ms-cli's MIN_MS1_LEN.
const minMS1Len = 48

// IsMS1Shaped is the host's looks_like_ms1 (ms-cli argv_guard.rs:148-164): trim,
// lowercase, strip the display separators (whitespace, '-', ','), then at least
// 48 characters, an `ms1` prefix and only bech32 characters. NO checksum -- a
// grouped or mistyped plate the host refuses is refused here too.
func IsMS1Shaped(s string) bool {
	t := strings.ToLower(strings.TrimSpace(s))
	var b strings.Builder
	for _, r := range t {
		if r == ' ' || r == '\t' || r == '\n' || r == '\r' || r == '-' || r == ',' {
			continue
		}
		b.WriteRune(r)
	}
	t = b.String()
	if len(t) < minMS1Len || !strings.HasPrefix(t, "ms1") {
		return false
	}
	for _, r := range t[3:] {
		if !strings.ContainsRune(bech32Charset, r) {
			return false
		}
	}
	return true
}

func isHex(b []byte) bool {
	for _, c := range b {
		switch {
		case c >= '0' && c <= '9', c >= 'a' && c <= 'f', c >= 'A' && c <= 'F':
		default:
			return false
		}
	}
	return true
}
```

- [ ] **Step 5: Run to green, then the mutations.**

Run: `go vet ./hashlock/ && go test -count=1 ./hashlock/`
Expected: PASS (6 tests). Mutations, each reverted: `Salt = append(Salt, 0, 0)` → 11 hardened X/H failures; `Iterations = 99999` → 11 failures; `phrase = []byte(seal.NormalisePassphrase(string(phrase)))` at the top of `PreimageHardened` → exactly the `Correct Horse Battery Staple` and `  a  b ` rows fail; stripping `-`/`,` from the phrase first → exactly the `correct-horse,battery staple` row fails; `IsMS1Shaped` using `codex32.New` → the grouped-by-5, leading/trailing-spaces and grouped-by-2 refusals rows fail (a checksum parse rejects grouped input); the cap literal 99 → `TestPhraseMaxCharsIsTheCap` and the 100-character refusals row fail.

- [ ] **Step 6: Commit.**

```bash
git add hashlock/hashlock.go hashlock/hashlock_test.go hashlock/testdata/hashlock-v0.8.json hashlock/testdata/hashlock-v0.8.provenance.json
git commit -s -m "hashlock: port ms_codec::hashlock (0.8.0) -- both derivations, the digest, the phrase rule, the host's ms1-shape test; corpus vendored and pinned (hashlock H2)"
```

---

### Task 2: `codex32.DecodeMS1Preimage`

**Files:**
- Modify: `codex32/mspayload.go` (append, after `IsPreimage`)
- Modify: `codex32/mspayload_test.go` (append)

- [ ] **Step 1: The test.** Append to `codex32/mspayload_test.go`:

```go
// H2 (SPEC_hashlock_H2_device §6): the 0x03 kind has ONE decoder of its own;
// DecodeMS1 keeps refusing it (H0), and the two never share a code path.
func TestDecodeMS1PreimageIsShapeExact(t *testing.T) {
	const plate = "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c"
	s, err := New(plate)
	if err != nil {
		t.Fatal(err)
	}
	x, err := DecodeMS1Preimage(s)
	if err != nil {
		t.Fatalf("DecodeMS1Preimage(plate): %v", err)
	}
	if x[0] == 0 && x[31] == 0 {
		t.Fatalf("preimage looks zero: %x", x)
	}
	if _, _, _, err := DecodeMS1(s); err != errMSBadPrefix {
		t.Errorf("DecodeMS1(plate) = %v, want errMSBadPrefix (H0 contract)", err)
	}
	for _, c := range []struct{ name, s string; want error }{
		{"entr single", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f", errMSBadPrefix},
		{"a 2-of-N share beginning 0x03", "ms12testaqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdq7pl8qdc5tsp", errMSBadPrefix},
		{"the entr-id 0x03 shape (kind is the prefix byte)", "ms10entrsqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5gz69g08wwtz9", nil},
	} {
		e, err := New(c.s)
		if err != nil {
			t.Fatalf("New(%s): %v", c.name, err)
		}
		if _, err := DecodeMS1Preimage(e); err != c.want {
			t.Errorf("DecodeMS1Preimage(%s) err = %v, want %v", c.name, err, c.want)
		}
	}
	// An unshared 0x03 string whose payload is not 33 bytes: the length rule.
	d17 := make([]byte, 17)
	d17[0] = 0x03
	short, err := NewSeed("ms", 0, "hash", 's', d17)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := DecodeMS1Preimage(short); err != errMSBadLength {
		t.Errorf("17-byte 0x03 payload: err = %v, want errMSBadLength", err)
	}
}
```

- [ ] **Step 2: RED.** Run: `go test -count=1 -run TestDecodeMS1PreimageIsShapeExact ./codex32/` — Expected: does not compile.

- [ ] **Step 3: The decoder.** Append to `codex32/mspayload.go`:

```go
// DecodeMS1Preimage decodes the m-format HASHLOCK PREIMAGE kind (SPEC_ms_hashlock
// §1: payload = [0x03][32 bytes]) from a New-valid string: ONLY an unshared
// single whose data is exactly 33 bytes beginning 0x03 -- the shape IsPreimage
// tests. Every other input is refused with the same errors DecodeMS1 uses:
// errMSBadPrefix for a wrong first byte or a SHARE, errMSBadLength for an
// unshared 0x03 payload that is not 33 bytes.
//
// DecodeMS1 is deliberately NOT taught this kind (r2 review C-2): its five callers
// all treat the result as a SEED. The returned preimage is SECRET; the caller
// scrubs. No screen calls this in stage H2.
func DecodeMS1Preimage(s String) (preimage [32]byte, err error) {
	f, perr := ParsePrefix(s.String())
	if perr != nil || !f.Unshared {
		return preimage, errMSBadPrefix
	}
	d := s.Seed()
	if len(d) == 0 || d[0] != msPrefixPreimage {
		return preimage, errMSBadPrefix
	}
	if len(d) != 1+32 {
		return preimage, errMSBadLength
	}
	copy(preimage[:], d[1:])
	return preimage, nil
}
```

- [ ] **Step 4: GREEN + mutation.** Run: `go test -count=1 ./codex32/` — Expected: PASS. Mutation: drop the `!f.Unshared` clause → the share case returns a 32-byte value where `errMSBadPrefix` is wanted; revert.

- [ ] **Step 5: Commit.**

```bash
git add codex32/mspayload.go codex32/mspayload_test.go
git commit -s -m "codex32: DecodeMS1Preimage -- the 0x03 kind's own decoder, shape-exact; DecodeMS1 unchanged (hashlock H2)"
```

---

### Task 3: `Which hash?` — label-keyed rows and the phrase row

**Files:**
- Modify: `gui/composer_hash.go` — the header comment at :27-28; `composerHashEdit` (:140-172) replaced
- Modify: `gui/composer_hash_test.go` (append)
- Modify: `gui/composer_copy.go` (append `composerCopyHashlockNoPayloadLead`)

**Interfaces:**
- Consumes: `composerPickScreen(ctx, th, title, lead string, rows []string) (int, bool)`, `composerPayloadDigests(*syswSession) [][32]byte`, `composerHashRow`, `composerHexEntry`, `composerCopyHashRule`, `showError`; Task 4's `hashlockPhraseRoute(ctx, th, st, idx) hashlockOutcome`.
- Produces: `composerHashRows` struct; the phrase row label constant `composerHashRowPhrase = "Type a hashlock phrase"`.

- [ ] **Step 1: The tests (RED).** Append to `gui/composer_hash_test.go`:

```go
// H2: the row switch is keyed by LABEL (spec §5; r2 review C-4). With 0, 1 and 2
// payload digests, every row does what its label says, and `Type 64 hex` never
// clears the lock. MUTATION: restore the index arithmetic with the new row
// inserted -> "Type 64 hex" lands in the clearing arm and this fails.
func TestWhichHashRowsAreLabelKeyed(t *testing.T) {
	for _, n := range []int{0, 1, 2} {
		recs := make([]string, n)
		for i := range recs {
			recs[i] = "hash:" + strings.Repeat(fmt.Sprintf("%02x", 0xa0+i), 32)
		}
		s := composerSessionWith(recs, nil)
		rows := composerHashRows(s)
		if got := len(rows.labels); got != n+3 {
			t.Fatalf("n=%d: %d rows, want %d", n, got, n+3)
		}
		if rows.labels[rows.phraseRow] != composerHashRowPhrase ||
			rows.labels[rows.hexRow] != "Type 64 hex" ||
			rows.labels[rows.noneRow] != "No hash lock" {
			t.Fatalf("n=%d: labels misplaced: %v", n, rows.labels)
		}
		if rows.phraseRow != n || rows.hexRow != n+1 || rows.noneRow != n+2 {
			t.Fatalf("n=%d: indices %d/%d/%d", n, rows.phraseRow, rows.hexRow, rows.noneRow)
		}
		if n == 0 && !strings.Contains(rows.lead, "Type a phrase below") {
			t.Errorf("no-payload lead missing: %q", rows.lead)
		}
		if n > 0 && rows.lead != "Which hash?" {
			t.Errorf("lead with payload digests: %q", rows.lead)
		}
	}
	if composerPickScreenMaxRows < 2+3 {
		t.Fatalf("composerPickScreenMaxRows = %d < the longest row set", composerPickScreenMaxRows)
	}
}
```

Run: `go test -count=1 -run TestWhichHashRowsAreLabelKeyed ./gui/` — Expected: does not compile (`composerHashRows` undefined).

- [ ] **Step 2: The rows struct and the switch.** In `gui/composer_hash.go`, replace lines 27-28 of the header comment with:

```go
// THE COMPOSER DERIVES A PREIMAGE IN RAM FOR ONE SCREEN (H2) AND NEVER STORES,
// SHOWS OR ENGRAVES IT. It puts a digest in a script.
```

and replace `composerHashEdit` (the whole function at :140-172) with:

```go
const composerHashRowPhrase = "Type a hashlock phrase"

// composerHashRows builds `Which hash?` ONCE and records where each named row
// sits, so the dispatch below is by label, never by index arithmetic (spec §5;
// r2 review C-4: the shipped default arm cleared the lock when a row moved).
type composerHashRows struct {
	labels    []string
	lead      string
	digests   [][32]byte
	phraseRow int
	hexRow    int
	noneRow   int
}

func composerHashRowsFor(s *syswSession) composerHashRows {
	digests := composerPayloadDigests(s)
	labels := make([]string, 0, len(digests)+3)
	for i, d := range digests {
		labels = append(labels, composerHashRow(i+1, d))
	}
	r := composerHashRows{digests: digests, lead: "Which hash?"}
	r.phraseRow = len(labels)
	labels = append(labels, composerHashRowPhrase)
	r.hexRow = len(labels)
	labels = append(labels, "Type 64 hex")
	r.noneRow = len(labels)
	labels = append(labels, "No hash lock")
	r.labels = labels
	if len(digests) == 0 {
		r.lead = composerCopyHashlockNoPayloadLead()
	}
	return r
}

func composerHashEdit(ctx *Context, th *Colors, st *composerState, idx int) bool {
	title := fmt.Sprintf("Path %d hash", idx+1)
	for {
		rows := composerHashRowsFor(ctx.sysw)
		sel, ok := composerPickScreen(ctx, th, title, rows.lead, rows.labels)
		if !ok {
			return false // Back at `Which hash?` -- the ONLY false this function returns (spec §4.6)
		}
		// The §8i rule fires when the operator is TAKING a hash: a payload row,
		// the phrase row or the hex row -- stated as that predicate.
		taking := sel < len(rows.digests) || sel == rows.phraseRow || sel == rows.hexRow
		if taking {
			showError(ctx, th, title, composerCopyHashRule())
		}
		switch {
		case sel < len(rows.digests):
			d := rows.digests[sel]
			st.list.Paths[idx].Hash = &d
			return true
		case sel == rows.phraseRow:
			switch hashlockPhraseRoute(ctx, th, st, idx, rows.digests) {
			case hashlockAssigned:
				return true
			case hashlockBackToWhichHash:
				continue
			}
		case sel == rows.hexRow:
			d, ok := composerHexEntry(ctx, th)
			if !ok {
				continue // Back from hex entry returns to `Which hash?`, path intact
			}
			st.list.Paths[idx].Hash = &d
			return true
		case sel == rows.noneRow:
			st.list.Paths[idx].Hash = nil
			return true
		default:
			panic(fmt.Sprintf("composerHashEdit: pick returned row %d of %d", sel, len(rows.labels)))
		}
	}
}
```

(`composerHashRows` in the test is the struct's constructor name used by the test — to keep the test as written, add `func composerHashRows(s *syswSession) composerHashRows { return composerHashRowsFor(s) }` is NOT allowed (name clash with the type). Rename the constructor: the test calls `composerHashRows(s)`; make the TYPE `composerHashRowSet` and the FUNCTION `composerHashRows`. Apply that renaming consistently in this step.)

Note the behaviour change for `Type 64 hex`'s Back: today `composerHexEntry`'s `false` propagates out of `composerHashEdit` and, at creation, deletes the path (`composer_shape.go:269`); under §4.6 it returns to `Which hash?`. The test in Step 1 does not cover it; Task 4's harness tests do (Back from hex entry at creation keeps the path).

Append to `gui/composer_copy.go`:

```go
// ─── H2: hashlock phrase route (SPEC_hashlock_H2_device §4) ──────────────────

func composerCopyHashlockNoPayloadLead() string {
	return "No hash record in the payload. Type a phrase below, or make one with " +
		"ms hashlock on the host."
}
```

- [ ] **Step 3: GREEN.** Task 4 supplies `hashlockPhraseRoute`; until then add a one-line stub in `gui/composer_hashlock.go` returning `hashlockBackToWhichHash` so this task compiles, and replace it in Task 4. Run: `go test -count=1 -run 'TestWhichHashRowsAreLabelKeyed|TestComposerHash' ./gui/` — Expected: PASS.

- [ ] **Step 4: Commit.**

```bash
git add gui/composer_hash.go gui/composer_hash_test.go gui/composer_copy.go gui/composer_hashlock.go
git commit -s -m "composer: Which hash? rows are label-keyed; the phrase row; the no-payload lead names both routes (hashlock H2)"
```

---

### Task 4: The phrase route — screens, derivation, confirm, copy

**Files:**
- Create: `gui/composer_hashlock.go` (replacing Task 3's stub), `gui/composer_hashlock_test.go`
- Modify: `gui/composer_copy.go` (append), `gui/composer_copy_test.go` (rows), `gui/modal_fits_test.go` (rows)
- Modify: `gui/composer_state.go` (`hashByPhrase bool` on `composerState`), `gui/composer_shape.go:443` (§8h form)

**Interfaces:**
- Consumes: `NewPassphraseKeyboard(ctx)` (`gui/passphrase_keyboard.go:76`; `kbd.Update`, `kbd.Fragment`, `kbd.Layout`, `kbd.MaxHeight`), `composerPickScreen`, `composerConfirmScreen(ctx, th, title, body) bool` (`gui/composer_shape.go:77`), `composerConfirmBody` (`gui/composer_copy.go:32`), `showError`, `layoutTitle`, `layoutNavigation`, `widget.Labelf`/`Labelw`, `hashlock.*`.
- Produces: `hashlockPhraseRoute(ctx, th, st, idx, payloadDigests [][32]byte) hashlockOutcome` with `hashlockAssigned | hashlockBackToWhichHash`.

- [ ] **Step 1: The copy, and its two gates (RED).** Append to `gui/composer_copy.go`:

```go
func composerCopyHashlockPhraseLead() string {
	return "Use a phrase you have never used anywhere else."
}

func composerCopyHashlockRefusal(err error) string {
	switch err {
	case hashlock.ErrEmpty:
		return "Type a hashlock phrase, or press Back."
	case hashlock.ErrNotPrintableASCII:
		return "A hashlock phrase is printable ASCII only."
	case hashlock.ErrMS1Shaped:
		return "That is a preimage plate, not a phrase. On the host, run ms hashlock " +
			"with it and load the hash: record it prints."
	case hashlock.ErrTooLong:
		return "A hashlock phrase is at most 100 characters."
	case hashlock.ErrHex64:
		return "That is a preimage in hex, not a phrase. Use the Type 64 hex row."
	}
	return err.Error()
}

func composerCopyHashlockHardenedWarning() string {
	return "Even a 20-character phrase falls in about 72 days on one GPU, and " +
		"shorter ones fall sooner. Choose it from a generator. If you have used " +
		"this phrase anywhere else, press Back and choose another. Continue?"
}

func composerCopyHashlockSHA256Warning() string {
	return "This is the brainwallet construction: anyone holding the digest tests " +
		"10^10 phrases per second. A phrase a person chose is not safe here; use " +
		"six diceware words. If you have used this phrase anywhere else, press " +
		"Back and choose another. Continue?"
}

func composerCopyHashlockDerivingLead() string {
	return "Deriving. This takes about 10 seconds."
}

// composerCopyHashlockConfirm is the §4.5 body. relation is "" when the payload
// holds no hash: record; otherwise the matches/no-match line.
func composerCopyHashlockConfirm(first8last8, method string, chars int, relation string) string {
	b := "hash  " + first8last8 + "\n" +
		fmt.Sprintf("method: %s   chars: %d", method, chars) + "\n"
	if relation != "" {
		b += relation + "\n"
	}
	return b +
		"Write down this phrase and the method now. They are not on this device and " +
		"not on your plates. Without both, this path can never be spent.\n" +
		"One phrase per policy. Spending any path of a wsh wallet publishes this " +
		"digest. Never use this phrase as a passphrase or a password anywhere else " +
		"-- a spend publishes the preimage, and anyone can then test guesses at the " +
		"phrase itself.\n" +
		"Before you fund this wallet, run ms hashlock with this phrase and method on " +
		"the host and check the digest matches."
}

func composerCopyHashlockRelation(i int) string {
	if i < 0 {
		return "no hash: record in the payload has this digest"
	}
	return fmt.Sprintf("matches hash %d in the payload", i+1)
}

// §8h, the phrase-route form (SPEC_hashlock_H2_device §4.7).
func composerCopyHashEveryPathPhrase() string {
	return "HASH ON EVERY PATH\n" +
		"Every way to spend this wallet needs a hashlock preimage. It is not on " +
		"this device and not on these plates. Back up the phrase and its method, " +
		"or the preimage plate, separately."
}
```

Add rows to `composerCopyTable()` in `gui/composer_copy_test.go` for each new function (section "H2-4.2", "H2-4.3a", "H2-4.3b", "H2-4.4", "H2-4.5", "H2-4.7", with their expected normalised text — copy the row shape of the existing entries), and rows to `TestModalsThisBlockTouchesAreDrawnInFull` in `gui/modal_fits_test.go`:

```go
		{
			"the hashlock hardened warning (H2 §4.3)",
			composerCopyHashlockHardenedWarning(),
		},
		{
			"the hashlock sha256 warning (H2 §4.3)",
			composerCopyHashlockSHA256Warning(),
		},
		{
			"the hashlock confirm modal, longest variant (H2 §4.5)",
			composerConfirmBody(composerCopyHashlockConfirm("b867db87..edbc96cb", "hardened", 100,
				composerCopyHashlockRelation(-1))),
		},
		{
			"the hashlock ms1-plate refusal (H2 §2 rule 3)",
			composerCopyHashlockRefusal(hashlock.ErrMS1Shaped),
		},
		{
			"HASH ON EVERY PATH, phrase-route form (H2 §4.7)",
			composerCopyHashEveryPathPhrase(),
		},
```

Run: `go test -count=1 -run 'TestModalsThisBlockTouchesAreDrawnInFull|TestComposerCopy' ./gui/` — Expected: does not compile until the functions exist; then, if the §4.5 longest body does NOT fit, apply the spec's drop order (§4.5) — first the reuse block to two sentences, then move the reconciliation line to `composerCopyHashEveryPathPhrase` — and record which step was needed. The measured headroom goes in the commit message.

- [ ] **Step 2: The harness tests (RED).** Create `gui/composer_hashlock_test.go`:

```go
package gui

import (
	"encoding/hex"
	"strings"
	"testing"
)

// The anchor phrase and the corpus digests (hashlock/testdata/hashlock-v0.8.json,
// derivation row 0) -- typed on the real keyboard through the real flow.
const (
	hashlockAnchorPhrase = "correct horse battery staple"
	hashlockAnchorHardH  = "3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12"
	hashlockAnchorSHA_H  = "b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb"
	hashlockMixedPhrase  = "Correct Horse Battery Staple"
)

// runComposerAddPath drives composerAddPath (the CREATION entry point, where a
// false from composerHashEdit deletes the path -- spec §4.6) on the touch harness.
func runComposerAddPath(t *testing.T, st *composerState, s *syswSession) *sessionHarness {
	t.Helper()
	ctx := NewContext(newPlatform())
	ctx.sysw = s
	returned := false
	frame, drawer, quit := runUITouch(ctx, func() {
		composerAddPath(ctx, &descriptorTheme, st)
		returned = true
	})
	h := &sessionHarness{t: t, ctx: ctx, done: &returned}
	h.frame, h.drawer = frame, drawer
	t.Cleanup(quit)
	return h
}

// typeOnPassphraseKeyboard taps each character of s on the four-page printable
// keyboard (hookPPWidget exposes it as "kbd"; see passphrase_flow_test.go for the
// page-turn idiom this reuses).
func typeOnPassphraseKeyboard(t *testing.T, h *sessionHarness, s string) {
	t.Helper()
	for _, r := range s {
		h.tapPassphraseKey(r)
	}
}

func hashHex(h *[32]byte) string { return hex.EncodeToString(h[:]) }

// Both methods, from the creation entry point, land the corpus digest on the path.
func TestHashlockPhraseRouteSetsTheCorpusDigest(t *testing.T) {
	for _, tc := range []struct {
		name, phrase, method, want string
		methodRow                  int
	}{
		{"hardened anchor", hashlockAnchorPhrase, "hardened", hashlockAnchorHardH, 0},
		{"sha256 anchor", hashlockAnchorPhrase, "sha256", hashlockAnchorSHA_H, 1},
	} {
		t.Run(tc.name, func(t *testing.T) {
			st := composerStateForTest(t) // an empty policy shape with one path being added
			h := runComposerAddPath(t, st, composerSessionWith(nil, nil))
			h.mustReach("EXPERIMENTAL")
			h.holdConfirm() // key-less path consent (§8a)
			h.mustReach("Which hash?")
			h.tapRow(0, 3) // Type a hashlock phrase (no payload digests)
			h.mustReach("Hash lock") // §8i rule modal
			h.tapNav(Button3)
			h.mustReach("Hashlock phrase")
			typeOnPassphraseKeyboard(t, h, tc.phrase)
			h.tapNav(Button3) // OK
			h.mustReach("Which method?")
			h.tapRow(tc.methodRow, 2)
			h.tapNav(Button3)
			if tc.method == "sha256" {
				h.mustReach("brainwallet")
				h.holdConfirm()
			} else {
				// 28 characters: no hardened warning.
				h.mustReach("Deriving")
			}
			h.mustReach("Write down this phrase")
			h.holdConfirm()
			if got := st.list.Paths[len(st.list.Paths)-1].Hash; got == nil || hashHex(got) != tc.want {
				t.Fatalf("path hash = %v, want %s", got, tc.want)
			}
		})
	}
}

// The three non-fixed-point rows, typed exactly, derive the corpus digests -- a
// screen-layer fold (case, whitespace, separators) fails here (spec §2, §7.2).
func TestHashlockPhraseRouteDoesNotNormalise(t *testing.T) {
	c := loadHashlockCorpusForGUI(t) // reads hashlock/testdata via the package path
	for _, phrase := range []string{hashlockMixedPhrase, "  a  b ", "correct-horse,battery staple"} {
		row := c.row(t, phrase)
		st := composerStateForTest(t)
		h := runComposerAddPath(t, st, composerSessionWith(nil, nil))
		h.mustReach("EXPERIMENTAL")
		h.holdConfirm()
		h.mustReach("Which hash?")
		h.tapRow(0, 3)
		h.mustReach("Hash lock")
		h.tapNav(Button3)
		h.mustReach("Hashlock phrase")
		typeOnPassphraseKeyboard(t, h, phrase)
		h.tapNav(Button3)
		h.mustReach("Which method?")
		h.tapRow(1, 2) // sha256: instant
		h.tapNav(Button3)
		h.mustReach("brainwallet")
		h.holdConfirm()
		h.mustReach("Write down this phrase")
		h.holdConfirm()
		if got := st.list.Paths[len(st.list.Paths)-1].Hash; got == nil || hashHex(got) != row.SHA256H {
			t.Fatalf("%q: path hash = %v, want %s", phrase, got, row.SHA256H)
		}
	}
}

// Spec §4.6: Back at every inner step keeps the phrase and never deletes the
// path; only Back at `Which hash?` returns false (and deletes it at creation).
func TestHashlockBackContractKeepsThePath(t *testing.T) {
	st := composerStateForTest(t)
	h := runComposerAddPath(t, st, composerSessionWith(nil, nil))
	h.mustReach("EXPERIMENTAL")
	h.holdConfirm()
	h.mustReach("Which hash?")
	h.tapRow(0, 3)
	h.mustReach("Hash lock")
	h.tapNav(Button3)
	h.mustReach("Hashlock phrase")
	typeOnPassphraseKeyboard(t, h, hashlockAnchorPhrase)
	h.tapNav(Button3)
	h.mustReach("Which method?")
	h.tapNav(Button1) // Back -> phrase screen, phrase intact
	h.mustReach("Hashlock phrase")
	h.mustReach("28/100")
	h.tapNav(Button3)
	h.mustReach("Which method?")
	h.tapRow(1, 2)
	h.tapNav(Button3)
	h.mustReach("brainwallet")
	h.tapNav(Button1) // decline -> method pick, phrase intact
	h.mustReach("Which method?")
	h.tapRow(0, 2)
	h.tapNav(Button3)
	h.mustReach("Deriving")
	h.tapNav(Button1) // Back during derivation -> method pick
	h.mustReach("Which method?")
	h.tapRow(1, 2)
	h.tapNav(Button3)
	h.mustReach("brainwallet")
	h.holdConfirm()
	h.mustReach("Write down this phrase")
	h.tapNav(Button1) // Back on the confirm -> method pick, nothing assigned
	h.mustReach("Which method?")
	if n := len(st.list.Paths); n != 1 {
		t.Fatalf("path deleted by an inner Back: %d paths", n)
	}
	if st.list.Paths[0].Hash != nil {
		t.Fatalf("hash assigned before HOLD")
	}
	h.tapNav(Button1) // Back at method pick -> phrase screen
	h.mustReach("Hashlock phrase")
	h.tapNav(Button1) // Back at phrase screen -> Which hash?, phrase dropped
	h.mustReach("Which hash?")
	if n := len(st.list.Paths); n != 1 {
		t.Fatalf("path deleted by Back to Which hash?: %d paths", n)
	}
	h.tapNav(Button1) // Back at Which hash? -> false -> creation deletes the path
	h.waitDone()
	if n := len(st.list.Paths); n != 0 {
		t.Fatalf("Back at Which hash? at creation must delete the path: %d paths", n)
	}
}

// Declined SHA-256, then Hardened, with the phrase typed ONCE (spec §7.2).
func TestHashlockDeclineThenHardenedTypesOnce(t *testing.T) {
	st := composerStateForTest(t)
	h := runComposerAddPath(t, st, composerSessionWith(nil, nil))
	h.mustReach("EXPERIMENTAL")
	h.holdConfirm()
	h.mustReach("Which hash?")
	h.tapRow(0, 3)
	h.mustReach("Hash lock")
	h.tapNav(Button3)
	h.mustReach("Hashlock phrase")
	typeOnPassphraseKeyboard(t, h, hashlockAnchorPhrase)
	h.tapNav(Button3)
	h.mustReach("Which method?")
	h.tapRow(1, 2)
	h.tapNav(Button3)
	h.mustReach("brainwallet")
	h.tapNav(Button1)
	h.mustReach("Which method?")
	h.tapRow(0, 2)
	h.tapNav(Button3)
	h.mustReach("Deriving")
	h.mustReach("Write down this phrase")
	h.holdConfirm()
	if got := st.list.Paths[0].Hash; got == nil || hashHex(got) != hashlockAnchorHardH {
		t.Fatalf("hash = %v, want hardened anchor", got)
	}
}

// The §2 refusals through the screen: 101/100 visible and refused; 64 hex; an
// ms1 plate grouped and ungrouped (spec §7.2).
func TestHashlockPhraseRefusalsOnScreen(t *testing.T) {
	const plate = "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c"
	for _, tc := range []struct{ name, typed, needle string }{
		{"101 characters", strings.Repeat("k", 101), "at most 100 characters"},
		{"64 hex", hashlockAnchorHardH, "Use the Type 64 hex row"},
		{"plate ungrouped", plate, "preimage plate, not a phrase"},
		{"plate grouped by 5", groupBy(plate, 5), "preimage plate, not a phrase"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			st := composerStateForTest(t)
			h := runComposerAddPath(t, st, composerSessionWith(nil, nil))
			h.mustReach("EXPERIMENTAL")
			h.holdConfirm()
			h.mustReach("Which hash?")
			h.tapRow(0, 3)
			h.mustReach("Hash lock")
			h.tapNav(Button3)
			h.mustReach("Hashlock phrase")
			typeOnPassphraseKeyboard(t, h, tc.typed)
			if tc.name == "101 characters" {
				h.mustReach("101/100")
			}
			h.tapNav(Button3)
			h.mustReach(tc.needle)
		})
	}
}

// The method modals fire on their condition and not otherwise (19 vs 20 chars;
// sha256 always).
func TestHashlockMethodModalsFireOnCondition(t *testing.T) {
	for _, tc := range []struct {
		phrase string
		method int
		warns  bool
	}{
		{"nineteen-characters", 0, true},  // 19 chars, hardened -> 72-days modal
		{"twenty--characters!!", 0, false}, // 20 chars, hardened -> no modal
		{"twenty--characters!!", 1, true},  // sha256 -> always
	} {
		st := composerStateForTest(t)
		h := runComposerAddPath(t, st, composerSessionWith(nil, nil))
		h.mustReach("EXPERIMENTAL")
		h.holdConfirm()
		h.mustReach("Which hash?")
		h.tapRow(0, 3)
		h.mustReach("Hash lock")
		h.tapNav(Button3)
		h.mustReach("Hashlock phrase")
		typeOnPassphraseKeyboard(t, h, tc.phrase)
		h.tapNav(Button3)
		h.mustReach("Which method?")
		h.tapRow(tc.method, 2)
		h.tapNav(Button3)
		if tc.warns {
			h.mustReach("Continue?")
		} else {
			h.mustReach("Deriving")
		}
	}
}

// The relation line: with payload records, the modal says which one matches or
// that none does; with none, the line is absent (spec §4.5, journey C-2).
func TestHashlockConfirmRelationLine(t *testing.T) {
	// A hash: record whose digest IS the anchor's sha256 digest.
	s := composerSessionWith([]string{"hash:" + hashlockAnchorSHA_H, "hash:" + strings.Repeat("ab", 32)}, nil)
	st := composerStateForTest(t)
	h := runComposerAddPath(t, st, s)
	h.mustReach("EXPERIMENTAL")
	h.holdConfirm()
	h.mustReach("Which hash?")
	h.tapRow(2, 5) // the phrase row sits after the two payload rows
	h.mustReach("Hash lock")
	h.tapNav(Button3)
	h.mustReach("Hashlock phrase")
	typeOnPassphraseKeyboard(t, h, hashlockAnchorPhrase)
	h.tapNav(Button3)
	h.mustReach("Which method?")
	h.tapRow(1, 2)
	h.tapNav(Button3)
	h.mustReach("brainwallet")
	h.holdConfirm()
	h.mustReach("matches hash 1 in the payload")
}
```

(`composerStateForTest`, `h.tapRow(i, n)`, `h.holdConfirm()`, `h.tapPassphraseKey(r)`, `h.waitDone()`, `groupBy`, `loadHashlockCorpusForGUI` are small test helpers; write them in this file modelled on `composer_backleg_test.go`'s harness use and `passphrase_flow_test.go`'s keyboard driving. `tapRow` uses `rowY(i, n)` from `composer_paged.go`'s pick-screen geometry as the walks do; `holdConfirm` is the `ConfirmWarningScreen` hold gesture the existing composer tests use.)

Run: `go test -count=1 -run TestHashlock ./gui/` — Expected: does not compile.

- [ ] **Step 3: The route.** Replace Task 3's stub `gui/composer_hashlock.go` with:

```go
package gui

import (
	"encoding/hex"
	"fmt"
	"time"

	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/layout"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
	"seedhammer.com/hashlock"
)

// The phrase route of `Which hash?` (SPEC_hashlock_H2_device §4): phrase screen ->
// method pick (+ its modal) -> derivation -> hold-to-confirm. One loop, so every
// inner Back moves WITHIN the route with the phrase intact, and only Back at the
// phrase screen returns to `Which hash?` (§4.6). The preimage lives on the stack
// here and is dropped when this function returns (L7, L15).

type hashlockOutcome int

const (
	hashlockAssigned hashlockOutcome = iota
	hashlockBackToWhichHash
)

type hashlockMethod int

const (
	hashlockHardened hashlockMethod = iota
	hashlockSHA256
)

func (m hashlockMethod) String() string {
	if m == hashlockSHA256 {
		return "sha256"
	}
	return "hardened"
}

func hashlockPhraseRoute(ctx *Context, th *Colors, st *composerState, idx int, payload [][32]byte) hashlockOutcome {
	var phrase []byte
	for {
		p, ok := hashlockPhraseFlow(ctx, th, phrase)
		if !ok {
			return hashlockBackToWhichHash // phrase dropped
		}
		phrase = p
	pick:
		for {
			m, ok := hashlockMethodPick(ctx, th)
			if !ok {
				break pick // Back at the method pick -> phrase screen, phrase intact
			}
			if !hashlockMethodWarning(ctx, th, phrase, m) {
				continue // declined -> method pick, phrase intact
			}
			x, ok := hashlockDeriveFlow(ctx, th, phrase, m)
			if !ok {
				continue // Back during derivation -> method pick
			}
			h := hashlock.Digest(&x)
			rel := ""
			if len(payload) > 0 {
				match := -1
				for i, d := range payload {
					if d == h {
						match = i
						break
					}
				}
				rel = composerCopyHashlockRelation(match)
			}
			body := composerCopyHashlockConfirm(hashlockFirst8Last8(h), m.String(), len(phrase), rel)
			if composerConfirmScreen(ctx, th, "Hash lock", composerConfirmBody(body)) {
				d := h
				st.list.Paths[idx].Hash = &d
				st.hashByPhrase = true
				return hashlockAssigned
			}
			// Back on the confirm -> method pick, nothing assigned
		}
	}
}

func hashlockFirst8Last8(h [32]byte) string {
	s := hex.EncodeToString(h[:])
	return s[:8] + ".." + s[len(s)-8:]
}

// hashlockPhraseFlow is the phrase screen (§4.2): the four-page printable-ASCII
// keyboard, a lead, an unclamped n/100 counter, and the §2 rule on OK. initial
// restores what was typed before a Back from the method pick. NOT
// passphraseEntryFlow (its title, pass-proof trigger and over-length message are
// the passphrase's -- r2 M-4), and NOTHING normalises the bytes.
func hashlockPhraseFlow(ctx *Context, th *Colors, initial []byte) ([]byte, bool) {
	kbd := NewPassphraseKeyboard(ctx)
	kbd.Fragment = string(initial)
	backBtn := &Clickable{Button: Button1}
	okBtn := &Clickable{Button: Button3}
	hookPPWidget("kbd", kbd)
	hookPPWidget("back", backBtn)
	hookPPWidget("ok", okBtn)
	for !ctx.Done {
		for kbd.Update(ctx) {
		}
		if backBtn.Clicked(ctx) {
			return nil, false
		}
		if okBtn.Clicked(ctx) {
			phrase := []byte(kbd.Fragment)
			if err := hashlock.ValidatePhrase(phrase); err != nil {
				showError(ctx, th, "Hashlock phrase", composerCopyHashlockRefusal(err))
				continue
			}
			return phrase, true
		}
		dims := ctx.Platform.DisplaySize()
		screen := layout.Rectangle{Max: dims}
		_, content := screen.CutTop(leadingSize)
		content, _ = content.CutBottom(8)
		leadOp, leadSz := widget.Labelw(&ctx.B, ctx.Styles.lead, dims.X-2*8, th.Text,
			composerCopyHashlockPhraseLead())
		leadBand, content := content.CutTop(leadSz.Y)
		leadOp = leadOp.Offset(leadBand.N(leadSz))
		cntOp, cntsz := widget.Labelf(&ctx.B, ctx.Styles.subtitle, th.Text,
			"%d/%d", len(kbd.Fragment), hashlock.PhraseMaxChars)
		counterBand, content := content.CutTop(cntsz.Y)
		cntOp = cntOp.Offset(counterBand.N(cntsz))
		kbd.MaxHeight = content.Dy()
		kbdOp, kbdsz := kbd.Layout(ctx, th)
		kbdOp = kbdOp.Offset(content.S(kbdsz))
		nav, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack},
			{Clickable: okBtn, Style: StylePrimary, Icon: assets.IconCheckmark},
		}...)
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, "Hashlock phrase")
		ctx.Frame(op.Layer(kbdOp, leadOp, cntOp, nav, titleOp, op.Color(&ctx.B, th.Background)))
	}
	return nil, false
}

func hashlockMethodPick(ctx *Context, th *Colors) (hashlockMethod, bool) {
	sel, ok := composerPickScreen(ctx, th, "Hashlock method", "Which method?",
		[]string{"Hardened (about 10 s)", "SHA-256"})
	if !ok {
		return 0, false
	}
	if sel == 1 {
		return hashlockSHA256, true
	}
	return hashlockHardened, true
}

// hashlockMethodWarning shows the §4.3 modal when its condition holds; both are
// confirm-to-proceed (L12). Returns false when declined.
func hashlockMethodWarning(ctx *Context, th *Colors, phrase []byte, m hashlockMethod) bool {
	switch m {
	case hashlockSHA256:
		return composerConfirmScreen(ctx, th, "SHA-256", composerConfirmBody(composerCopyHashlockSHA256Warning()))
	case hashlockHardened:
		if len(phrase) < 20 {
			return composerConfirmScreen(ctx, th, "Hardened", composerConfirmBody(composerCopyHashlockHardenedWarning()))
		}
	}
	return true
}

// hashlockDeriveFlow derives X. SHA-256 is instant. Hardened runs on a countdown
// screen driven by hashlock.DeriveHardened (the 14-byte salt as a slice --
// NEVER unlockDerive/seal.Header, §3); Back abandons with nothing assigned.
func hashlockDeriveFlow(ctx *Context, th *Colors, phrase []byte, m hashlockMethod) ([32]byte, bool) {
	if m == hashlockSHA256 {
		return hashlock.PreimageSHA256(phrase), true
	}
	backBtn := &Clickable{Button: Button1}
	start := time.Now()
	abandoned := false
	x, ok := hashlock.DeriveHardened(phrase, func(done, total int) bool {
		if ctx.Done {
			return false
		}
		if backBtn.Clicked(ctx) {
			abandoned = true
			return false
		}
		dims := ctx.Platform.DisplaySize()
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, "Deriving")
		pctOp, pctSz := widget.Label(&ctx.B, ctx.Styles.progress, th.Text,
			fmt.Sprintf("%d%%", done*100/total))
		lead := composerCopyHashlockDerivingLead()
		if elapsed := time.Since(start); done > 0 && elapsed > 0 {
			left := time.Duration(float64(elapsed) * float64(total-done) / float64(done))
			lead = fmt.Sprintf("About %d seconds left.", int(left.Seconds()+0.5))
		}
		leadOp, leadSz := widget.Labelw(&ctx.B, ctx.Styles.lead, dims.X-2*8, th.Text, lead)
		nav, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconDiscard},
		}...)
		screen := layout.Rectangle{Max: dims}
		_, content := screen.CutTop(leadingSize)
		pctOp = pctOp.Offset(content.N(pctSz).Add(image.Pt(0, 24)))
		leadOp = leadOp.Offset(content.Center(leadSz))
		ctx.Frame(op.Layer(pctOp, leadOp, nav, titleOp, op.Color(&ctx.B, th.Background)))
		return true
	})
	if !ok || abandoned {
		return x, false
	}
	return x, true
}
```

(Layout calls mirror `unlockDerive` at `gui/unlock_kdf.go:242-290`; the implementer aligns imports (`image`) and helper names against that function and `passphraseEntryFlow`, and the gate compiles them. The frame must be drawn INSIDE the progress callback, as the countdown needs a frame per step.)

Add `hashByPhrase bool` to `composerState` (`gui/composer_state.go`), and at `gui/composer_shape.go:443` replace `composerCopyHashEveryPath()` with:

```go
				composerCopyHashEveryPathFor(st)
```

with, in `gui/composer_copy.go`:

```go
func composerCopyHashEveryPathFor(st *composerState) string {
	if st.hashByPhrase {
		return composerCopyHashEveryPathPhrase()
	}
	return composerCopyHashEveryPath()
}
```

- [ ] **Step 4: GREEN, then the mutations.**

Run: `go vet ./gui/ && go test -count=1 -run 'TestHashlock|TestWhichHash|TestComposerHash|TestComposerCopy|TestModalsThisBlockTouchesAreDrawnInFull' ./gui/`
Expected: PASS. Mutations, each reverted: fold `phrase` through `seal.NormalisePassphrase` in `hashlockPhraseFlow` before `ValidatePhrase` → `TestHashlockPhraseRouteDoesNotNormalise` fails on the mixed-case and spaces rows; make the confirm's Back return `hashlockBackToWhichHash` → `TestHashlockBackContractKeepsThePath` fails at "Which method?"; make `composerHashEdit` return `false` from the phrase route's Back → the same test fails on the path count; remove the relation line → `TestHashlockConfirmRelationLine` fails; drop `!f.Unshared` in Task 2 → its test fails.

- [ ] **Step 5: The whole gui package.**

Run: `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`
Expected: all green, partition exhaustive (the count grows by this task's tests; quote it).

- [ ] **Step 6: Commit.**

```bash
git add gui/composer_hashlock.go gui/composer_hashlock_test.go gui/composer_copy.go gui/composer_copy_test.go gui/modal_fits_test.go gui/composer_state.go gui/composer_shape.go
git commit -s -m "composer: the hashlock phrase route -- phrase screen, method pick with both warnings, stepwise derivation, hold-to-confirm with backup, relation and reconciliation lines; Back contract as a loop (hashlock H2)"
```

---

### Task 5: The emulator arm and the firmware size

**Files:**
- Create: `cmd/emu/walk_hashlock_phrase.js`

- [ ] **Step 1: The walk.** Modelled on `cmd/emu/walk_h0_preimage.js` (helpers inlined) and `shots_composer.js`'s route to `Which hash?` (`goTo("Wallet Policy")` → `Build a new policy` → the shape → a path → `chooseRow(…, "Which hash?", …)`): tap `Type a hashlock phrase`, dismiss the §8i modal, type `correct horse battery staple` on the passphrase keyboard (map the key coordinates by probing `shScreen` for the keyboard's page, as `walk_verify.js` did for the ms1 keypad), OK, pick `SHA-256`, hold through the brainwallet modal, read the confirm modal and assert it contains `b867db87..edbc96cb`, `method: sha256`, `chars: 28`, `Write down this phrase`; Back out. Negative control: type `correct horse battery stapl` and assert the digest line does NOT contain `b867db87`. Second positive: `Correct Horse Battery Staple` shows the corpus's mixed-case sha256 digest (read it from the corpus, `derivation` row for that phrase). Hardened once: assert `3cf5d421..b70a4c12` after the countdown (allow 30 s). Export `run()` returning `{typed, control, mixed, hardened, ok}`; every assertion throws.

- [ ] **Step 2: Firmware size.** `export PATH=/nix/var/nix/profiles/default/bin:$PATH; nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller` — record flash and RAM against `c4a64fc`'s 1,583,132 / 62,800.

- [ ] **Step 3: Commit.**

```bash
git add cmd/emu/walk_hashlock_phrase.js
git commit -s -m "emu: hashlock phrase walk -- both methods, the mixed-case row, a negative control (hashlock H2)"
```

The controller RUNS the walk on an emulator built from the branch (fresh port, playwright) before the post-implementation review and records the frames in the continuity; the operator's device walk is H4 (spec §8).

---

### Task 6: Records

- [ ] engrave `design/FOLLOWUPS.md`: the composer spec's §6c line 386 and §14 sentence ("never derives … a preimage this cycle") → owning phase **H3**, with the replacement wording; the M-6 seam-corpus prose correction (from H1b) is done HERE if H2 re-vendors the seam corpus — it does not (H2 vendors the hashlock corpus, a different file), so it stays filed under H3 unless the implementer touches `codex32_seam_vectors.json`; the fork's own CHANGELOG does not exist — the merge commit message is the record.
- [ ] Post-implementation review (risk set: the device sets a hash that gates funds): ONE opus adversarial execution review over the whole diff; brief `design/agent-briefs/hashlock-H2-post-impl-brief.md`, report `design/agent-reports/hashlock-H2-post-impl.md`; GREEN before merge.
- [ ] Emulator walk by the controller (Task 5 Step 1) recorded; merge to fork `main` (`--no-ff`); push; flash at the operator's word; H4 acceptance (spec §8) with the operator.

---

## Self-review

1. **Spec coverage.** §2 → Task 1 (`ValidatePhrase`, `IsMS1Shaped`, the refusals rows) and Task 4 (through the screen); §3 → Task 1 (`DeriveHardened` on `seal.NewDeriver` with the slice salt; the constants; the lockstep mutations); §4.1 → Task 3; §4.2-§4.5 → Task 4 (the flow, the copy, both gates); §4.6 → Task 4's loop and the Back test through `composerAddPath`; §4.7 → `composerCopyHashEveryPathFor`; §5 → Task 3; §6 → Task 2; §7.1 → Task 1's tests; §7.2/§7.3 → Tasks 4/3; §7.4 → Task 2; §7.5 → Task 5; §7.6 → Task 5 Step 2; §8 → Task 6 (H4); §9 → nothing to build.
2. **Placeholders.** The harness helpers in Task 4 Step 2 are named and modelled on existing files; the implementer writes them (small) — recorded, not TBD. The walk's keyboard mapping is probed on the live emulator as `walk_verify.js` did; the plan says so.
3. **Type consistency.** `seal.NewDeriver(passphrase, salt []byte, iterations int) *Deriver`, `Step(n int) bool`, `Done()/Total() int`, `Key() []byte`, `Wipe()` (`seal/pbkdf2.go:85-182`); `composerPickScreen(ctx, th, title, lead string, rows []string) (int, bool)` (`composer_paged.go:259`); `composerConfirmScreen(ctx, th, title, body string) bool` (`composer_shape.go:77`); `composerConfirmBody(body string) string` (`composer_copy.go:32`); `Hash *[32]byte` (`md/compose.go:167`); `composerSessionWith(public, secret []string) *syswSession` (`composer_door_test.go:15`); `ParsePrefix(frag string) (Fields, error)`, `Fields.Unshared` (`codex32/polish.go:82,71`); `NewSeed(hrp string, threshold int, id string, shareIdx rune, data []byte) (String, error)` (`codex32/codex32.go:279`); `composerPickScreenMaxRows = 24` (`composer_paged.go:224`).
