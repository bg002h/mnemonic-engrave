# Hashlock H2 — Device Leg Implementation Plan (SeedHammer fork)

**STATUS: DRAFT -- build gate GREEN WITH FIXES folded; R0 round 0 pending.**

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
| `gui/composer_copy_test.go` | Modify (rows + count) | the AST-scan copy gate: ten new `composerCopyTable` rows and the `declared` literal 41 → 42 → 51 (build gate fixes 1, 2) |
| `gui/modal_fits_test.go` | Modify (rows) | five new `TestModalsThisBlockTouchesAreDrawnInFull` rows |
| `gui/composer_gates_test.go` | Modify (one row) | Task 3's no-payload lead moves an EXISTING test's pump target (build gate fix 12) |
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

**Block headers — the convention every code block below carries.** A fenced block that
holds file content opens with the file it belongs to and how much of that file it is:

    ```go file=gui/composer_hash.go mode=fragment
    ```go file=gui/composer_hashlock.go mode=whole

`mode=whole` means the block IS the file; `mode=fragment` means the block must appear
VERBATIM inside it, indentation included. Markdown takes only the FIRST word of an info
string as the language, so highlighting is unaffected.
`scripts/h2-plan-blocks-vs-tree.sh` parses those headers and checks every block against
the GATED SCRATCH TREE the build gate left at `/scratch/code/shibboleth/.tmp/h2-gate`
— whole blocks by `diff`, fragments by exact substring — and prints its own blind spots
(headerless ```bash blocks, and every prose claim). A block with NO header is a command
or an illustration, not file content, and nothing checks it.
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

```json file=hashlock/testdata/hashlock-v0.8.provenance.json mode=whole
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

```go file=hashlock/hashlock_test.go mode=whole
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

```go file=hashlock/hashlock.go mode=whole
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
Expected: PASS (6 tests). Then the mutations, each reverted. What follows is the build
gate's MEASURED outcome (`design/agent-reports/hashlock-H2-plan-build-gate.md`, Task 1
Step 5 table), which corrected two of this plan's own round-0 predictions:

| Mutation | Measured outcome |
| --- | --- |
| `Salt = append(Salt, 0, 0)` | 22 failures — 11 rows × (hardened X + hardened H) |
| `Iterations = 99999` | 22 failures, the same 11 rows × X and H |
| `phrase = []byte(seal.NormalisePassphrase(string(phrase)))` at the top of `PreimageHardened` | exactly the `Correct Horse Battery Staple` and `  a  b ` rows, 4 failures (X+H each) |
| strip `-`/`,` from the phrase first | **FOUR rows fail, not one.** Gate, verbatim: *"4 rows fail, not 1: `correct-horse,battery staple`, `a-b,c`, and BOTH 64-char rows … because those two rows also contain `-` and `,` … Plan's own claim is wrong; the mutation itself still works as a gate (it does fail), just on 4 rows, not 1."* Re-measured against the vendored corpus for this fold: 4 of the 11 derivation phrases carry `-` or `,` — `correct-horse,battery staple` (28), `a-b,c` (5), `hashlock phrase row: sixty-four printable characters, no hex!!xx` (64) and its `!`-suffixed sibling (65). |
| `IsMS1Shaped` using `codex32.New` | exactly refusals rows 11, 12 and 13 (grouped-by-5, leading/trailing spaces, grouped-by-2) — a checksum parse rejects grouped input |
| the cap literal 99 | **ONLY `TestPhraseMaxCharsIsTheCap` fails.** Gate, verbatim: *"The corpus has no 100-character refusals row — its one `too-long` row is 101 characters (verified: `len(...)==101`), which is refused whether the cap is 99 or 100, so `TestRefusalRowsMatchTheHost` stays green under this mutation. Plan's second clause does not hold for this corpus."* Re-measured for this fold: the sole `too-long` refusals row is 101 characters. |

Every mutation still fails the test it is aimed at — nothing silently passed. Only two of
the plan's own descriptions of SCOPE were wrong, and both are corrected above.

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

```go file=codex32/mspayload_test.go mode=fragment
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

```go file=codex32/mspayload.go mode=fragment
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
- Modify: `gui/composer_hash.go` — the header comment at :27-28; `composerHashEdit` (:139-176, its doc comment included) replaced
- Modify: `gui/composer_hash_test.go` (append; `"fmt"` joins the import block)
- Modify: `gui/composer_copy.go` (append `composerCopyHashlockNoPayloadLead`)
- Modify: `gui/composer_copy_test.go` — the new body's `composerCopyTable` row and the AST scan's `declared` literal (**build gate fix 1**: the plan had no step for either, and `TestComposerCopyTableCoversEveryBody` fails BY NAME on any `composerCopy*` function that has no row)
- Modify: `gui/composer_gates_test.go` — one EXISTING test's pump target (**build gate fix 12**, a direct consequence of this task's no-payload lead swap; see Step 4)

**Interfaces:**
- Consumes: `composerPickScreen(ctx, th, title, lead string, rows []string) (int, bool)`, `composerPayloadDigests(*syswSession) [][32]byte`, `composerHashRow`, `composerHexEntry`, `composerCopyHashRule`, `showError`; Task 4's `hashlockPhraseRoute(ctx, th, st, idx int, payload [][32]byte) hashlockOutcome`.
- Produces: the `composerHashRowSet` struct and its constructor `composerHashRows(s *syswSession) composerHashRowSet` — Go forbids a type and a func sharing one name in a package, and the tests call the CONSTRUCTOR `composerHashRows`, so the TYPE takes the `…RowSet` name; the phrase row label constant `composerHashRowPhrase = "Type a hashlock phrase"`.

- [ ] **Step 1: The tests (RED).** Append to `gui/composer_hash_test.go`:

```go file=gui/composer_hash_test.go mode=fragment
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

and add `"fmt"` to that file's imports, which the new test needs:

```go file=gui/composer_hash_test.go mode=fragment
import (
	"encoding/hex"
	"fmt"
	"strings"
	"testing"
)
```

Run: `go test -count=1 -run TestWhichHashRowsAreLabelKeyed ./gui/` — Expected: does not compile (`composerHashRows` undefined).

- [ ] **Step 2: The rows struct and the switch.** In `gui/composer_hash.go`, replace lines 27-28 of the header comment with:

```go file=gui/composer_hash.go mode=fragment
// THE COMPOSER DERIVES A PREIMAGE IN RAM FOR ONE SCREEN (H2) AND NEVER STORES,
// SHOWS OR ENGRAVES IT. It puts a digest in a script.
```

and replace `composerHashEdit` (the whole function at :140-172) with:

```go file=gui/composer_hash.go mode=fragment
const composerHashRowPhrase = "Type a hashlock phrase"

// composerHashRowSet builds `Which hash?` ONCE and records where each named row
// sits, so the dispatch below is by label, never by index arithmetic (spec §5;
// r2 review C-4: the shipped default arm cleared the lock when a row moved).
//
// (Named composerHashRowSet rather than composerHashRows: the constructor below
// is composerHashRows, and Go does not allow a type and a func to share a name
// in the same package -- the plan's own tests call the constructor composerHashRows.)
type composerHashRowSet struct {
	labels    []string
	lead      string
	digests   [][32]byte
	phraseRow int
	hexRow    int
	noneRow   int
}

func composerHashRows(s *syswSession) composerHashRowSet {
	digests := composerPayloadDigests(s)
	labels := make([]string, 0, len(digests)+3)
	for i, d := range digests {
		labels = append(labels, composerHashRow(i+1, d))
	}
	r := composerHashRowSet{digests: digests, lead: "Which hash?"}
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

// composerHashEdit sets or clears one path's hashlock.
func composerHashEdit(ctx *Context, th *Colors, st *composerState, idx int) bool {
	title := fmt.Sprintf("Path %d hash", idx+1)
	for {
		rows := composerHashRows(ctx.sysw)
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

(The TYPE is `composerHashRowSet` and the CONSTRUCTOR is `composerHashRows` because Go
does not allow a type and a func to share one name in a package, and the Step 1 test
calls `composerHashRows(s)`. The block above already carries that naming and the comment
recording it; nothing further needs renaming.)

Note the behaviour change for `Type 64 hex`'s Back: today `composerHexEntry`'s `false` propagates out of `composerHashEdit` and, at creation, deletes the path (`composer_shape.go:269`); under §4.6 it returns to `Which hash?`. The test in Step 1 does not cover it; Task 4's harness tests do (Back from hex entry at creation keeps the path).

Append to `gui/composer_copy.go`:

```go file=gui/composer_copy.go mode=fragment
// ─── H2: hashlock phrase route (SPEC_hashlock_H2_device §4) ──────────────────

func composerCopyHashlockNoPayloadLead() string {
	return "No hash record in the payload. Type a phrase below, or make one with " +
		"ms hashlock on the host."
}
```

- [ ] **Step 3: The copy gate's row and its count (build gate fix 1).** `TestComposerCopyTableCoversEveryBody` (`gui/composer_copy_test.go`) AST-scans `composer_copy.go` for every `composerCopy*` function and requires BOTH a `composerCopyTable` row and an exact declared-count literal. This task adds one such function, so both move. Add the row:

```go file=gui/composer_copy_test.go mode=fragment
		{"composerCopyHashlockNoPayloadLead", "H2-3", composerCopyHashlockNoPayloadLead(),
			"No hash record in the payload. Type a phrase below, or make one with ms hashlock on the host."},
```

and bump the count literal from `41` to `42`, recording why above it:

```go file=gui/composer_copy_test.go mode=fragment
	// 42 SINCE H2 TASK 3 added composerCopyHashlockNoPayloadLead (the
	// no-payload lead on `Which hash?`, SPEC_hashlock_H2_device §4.1).
```

(Task 4 adds nine more bodies and bumps the same literal to `51`; the file's FINAL text — the comment block and the `if declared != 51` it guards — is the block in Task 4 Step 1, which is what the gated tree holds. Both bumps are recorded in that one comment.)

- [ ] **Step 4: The pre-existing test this task's lead swap moves (build gate fix 12).** `TestComposerLockAndHashEditsAreNotGuardedByTheDiscardConfirm` in `gui/composer_gates_test.go` drives `composerPathEdit` with `ctx.sysw == nil`, so `composerHashRows` reports 0 payload digests and this task's design swaps the LEAD to `composerCopyHashlockNoPayloadLead()` — exactly as intended on a device with no hash record loaded. That test pumped to the literal `"Which hash?"`, which no longer appears on that frame. The TITLE is what is invariant across both leads, so the pump target moves to it:

```go file=gui/composer_gates_test.go mode=fragment
		{"time lock", 1, "What kind of time lock?"},
		// H2 (SPEC_hashlock_H2_device §4.1): with no ctx.sysw session loaded
		// (this test's own state), composerHashRows reports 0 payload digests,
		// so the screen's LEAD becomes composerCopyHashlockNoPayloadLead
		// rather than the literal "Which hash?" -- the TITLE ("Path 1 hash")
		// is what stays invariant across both cases, so the pump target moves
		// to it rather than to wording this test does not otherwise assert on.
		{"hash lock", 2, "Path 1 hash"},
```

This is a genuine regression in an unrelated existing test, and the narrow `-run` selections in this task never touch that file: the build gate found it only on the full `gui` shard set (shard 11 failed). Run `scripts/gui-shard-test.sh ./gui/ 24` — or at least `go test -run TestComposerLockAndHashEdits ./gui/` — before calling this task green.

- [ ] **Step 5: GREEN.** Task 4 supplies `hashlockPhraseRoute`; until then add a one-line stub in `gui/composer_hashlock.go` returning `hashlockBackToWhichHash` so this task compiles, and replace it in Task 4. Run: `go test -count=1 -run 'TestWhichHashRowsAreLabelKeyed|TestComposerHash|TestComposerCopy|TestComposerLockAndHashEdits' ./gui/` — Expected: PASS.

- [ ] **Step 6: Commit.**

```bash
git add gui/composer_hash.go gui/composer_hash_test.go gui/composer_copy.go gui/composer_copy_test.go gui/composer_gates_test.go gui/composer_hashlock.go
git commit -s -m "composer: Which hash? rows are label-keyed; the phrase row; the no-payload lead names both routes (hashlock H2)"
```

---

### Task 4: The phrase route — screens, derivation, confirm, copy

**Files:**
- Create: `gui/composer_hashlock.go` (replacing Task 3's stub), `gui/composer_hashlock_test.go`
- Modify: `gui/composer_copy.go` (append; `seedhammer.com/hashlock` joins its imports), `gui/composer_copy_test.go` (nine rows + the `declared` literal 42 → 51, and the `hashlock` import — **build gate fix 2**), `gui/modal_fits_test.go` (five rows + the `hashlock` import)
- Modify: `gui/composer_state.go` (`hashByPhrase bool` on `composerState`), `gui/composer_shape.go:443` (§8h form)

**Interfaces:**
- Consumes: `NewPassphraseKeyboard(ctx)` (`gui/passphrase_keyboard.go:76`; `kbd.Update`, `kbd.Fragment`, `kbd.Layout`, `kbd.MaxHeight`), `composerPickScreen`, `composerConfirmScreen(ctx, th, title, body) bool` (`gui/composer_shape.go:77`), `composerConfirmBody` (`gui/composer_copy.go:32`), `showError`, `layoutTitle`, `layoutNavigation`, `widget.Labelf`/`Labelw`, `hashlock.*`.
- Produces: `hashlockPhraseRoute(ctx, th, st, idx, payloadDigests [][32]byte) hashlockOutcome` with `hashlockAssigned | hashlockBackToWhichHash`.

- [ ] **Step 1: The copy, and its two gates (RED).** `gui/composer_copy.go` now references `hashlock`'s sentinel errors, so its import turns into a block:

```go file=gui/composer_copy.go mode=fragment
import (
	"fmt"

	"seedhammer.com/hashlock"
)
```

Append the bodies (this is the text the build gate proved fits — see the §4.5 note after the gate rows below; the reuse block is the brainstorm's two sentences, and the reconciliation line lives in `composerCopyHashEveryPathPhrase`):

```go file=gui/composer_copy.go mode=fragment
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
		"One phrase per policy. Never use this phrase as a passphrase or a password " +
		"anywhere else."
}

func composerCopyHashlockRelation(i int) string {
	if i < 0 {
		return "no hash: record in the payload has this digest"
	}
	return fmt.Sprintf("matches hash %d in the payload", i+1)
}

// §8h, the phrase-route form (SPEC_hashlock_H2_device §4.7).
//
// Carries the confirm modal's reconciliation line (moved here by §4.5's own
// drop order, step 2: the modal's normalised body measured only 64 characters
// of headroom against the required 80, so the line that converts a spend-time
// divergence into a five-minute check moves to Done instead of being cut).
func composerCopyHashEveryPathPhrase() string {
	return "HASH ON EVERY PATH\n" +
		"Every way to spend this wallet needs a hashlock preimage. It is not on " +
		"this device and not on these plates. Back up the phrase and its method, " +
		"or the preimage plate, separately.\n" +
		"Before you fund this wallet, run ms hashlock with this phrase and method " +
		"on the host and check the digest matches."
}
```

Now BOTH copy gates. `TestComposerCopyTableCoversEveryBody` AST-scans `composer_copy.go`
and fails BY NAME on any `composerCopy*` function without a `composerCopyTable` row, so
every one of this task's NINE new bodies needs one. Round 0 of this plan named six §8
SECTIONS (H2-4.2/4.3a/4.3b/4.4/4.5/4.7) rather than functions, and three of those sections
carry two bodies each — so `composerCopyHashlockRefusal`, `composerCopyHashlockRelation`
and `composerCopyHashEveryPathFor` had no row at all (**build gate fix 2**; gate, verbatim:
*"the plan's own row list under-counted its own new functions by 4"* — that 4 is against
all TEN H2 rows, Task 3's `composerCopyHashlockNoPayloadLead` included). `composerCopyTable`
needs the `hashlock` import for the refusal row:

```go file=gui/composer_copy_test.go mode=fragment
import (
	"go/ast"
	"go/parser"
	"go/token"
	"strings"
	"testing"

	"seedhammer.com/hashlock"
)
```

```go file=gui/composer_copy_test.go mode=fragment
		{"composerCopyHashlockPhraseLead", "H2-4.2", composerCopyHashlockPhraseLead(),
			"Use a phrase you have never used anywhere else."},
		{"composerCopyHashlockRefusal", "H2-4.2", composerCopyHashlockRefusal(hashlock.ErrMS1Shaped),
			"That is a preimage plate, not a phrase. On the host, run ms hashlock with it and load the hash: record it prints."},
		{"composerCopyHashlockHardenedWarning", "H2-4.3a", composerCopyHashlockHardenedWarning(),
			"Even a 20-character phrase falls in about 72 days on one GPU, and shorter ones fall sooner. Choose it from a generator. If you have used this phrase anywhere else, press Back and choose another. Continue?"},
		{"composerCopyHashlockSHA256Warning", "H2-4.3b", composerCopyHashlockSHA256Warning(),
			"This is the brainwallet construction: anyone holding the digest tests 10^10 phrases per second. A phrase a person chose is not safe here; use six diceware words. If you have used this phrase anywhere else, press Back and choose another. Continue?"},
		{"composerCopyHashlockDerivingLead", "H2-4.4", composerCopyHashlockDerivingLead(),
			"Deriving. This takes about 10 seconds."},
		{"composerCopyHashlockConfirm", "H2-4.5", composerCopyHashlockConfirm("b867db87..edbc96cb", "hardened", 100,
			composerCopyHashlockRelation(-1)),
			"hash  b867db87..edbc96cb method: hardened   chars: 100 no hash: record in the payload has this digest " +
				"Write down this phrase and the method now. They are not on this device and not on your plates. Without both, this path can never be spent. " +
				"One phrase per policy. Never use this phrase as a passphrase or a password anywhere else."},
		{"composerCopyHashlockRelation", "H2-4.5", composerCopyHashlockRelation(0),
			"matches hash 1 in the payload"},
		{"composerCopyHashEveryPathPhrase", "H2-4.7", composerCopyHashEveryPathPhrase(),
			"HASH ON EVERY PATH Every way to spend this wallet needs a hashlock preimage. It is not on this device and not on these plates. Back up the phrase and its method, or the preimage plate, separately. " +
				"Before you fund this wallet, run ms hashlock with this phrase and method on the host and check the digest matches."},
		{"composerCopyHashEveryPathFor", "H2-4.7", composerCopyHashEveryPathFor(&composerState{hashByPhrase: true}),
			"HASH ON EVERY PATH Every way to spend this wallet needs a hashlock preimage. It is not on this device and not on these plates. Back up the phrase and its method, or the preimage plate, separately. " +
				"Before you fund this wallet, run ms hashlock with this phrase and method on the host and check the digest matches."},
```

and the scan's declared-count literal goes 42 → 51, carrying the reason (this is the
file's final text, both bumps recorded):

```go file=gui/composer_copy_test.go mode=fragment
	// 51 SINCE H2 TASK 4 added the phrase route's nine bodies: the phrase
	// lead, the phrase-rule refusal, both method warnings, the deriving
	// lead, the confirm body and its relation line, and the two §8h forms
	// (SPEC_hashlock_H2_device §4.2-§4.7).
	if declared != 51 {
		t.Errorf("composer_copy.go declares %d bodies, the plan and the table know 51 -- "+
			"if that is deliberate, update both", declared)
	}
```

(`composerCopyHashEveryPathFor` is created in Step 3 with the §8h wiring; its row and the
51 land here so the count gate is bumped once. Step 1 is RED by construction, as below.)

Then `TestModalsThisBlockTouchesAreDrawnInFull` in `gui/modal_fits_test.go`, which also
needs the `hashlock` import:

```go file=gui/modal_fits_test.go mode=fragment
import (
	"strings"
	"testing"

	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/op"
	"seedhammer.com/hashlock"
)
```

```go file=gui/modal_fits_test.go mode=fragment
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

Run: `go test -count=1 -run 'TestModalsThisBlockTouchesAreDrawnInFull|TestComposerCopy' ./gui/` — Expected: does not compile until Step 3's functions exist; then GREEN.

**§4.5's drop order was needed, and BOTH of its steps (build gate fixes 3 and 4).**
`assertModalBodyFits` measures each body per-body with an 80-character margin. Measured
by the build gate:

- Step 0, the unshortened §4.5 body: **484 of 504 characters drawn — CUT** after
  "…check the digest matches.", before "Hold button to confirm.".
- Step 1, shorten the reuse block to the brainstorm's two sentences ("One phrase per
  policy. Never use this phrase as a passphrase or a password anywhere else."): it fits,
  384/384 drawn, **headroom 64 — still BELOW the 80-character margin**, so the test still
  fails ("fits today with only 64 characters to spare… Shorten this body rather than
  lowering the margin.").
- Step 2, move the reconciliation line ("Before you fund this wallet, run ms hashlock…")
  out of the confirm modal and into `composerCopyHashEveryPathPhrase`, which is the spec's
  own next step: confirm modal **290 drawn, headroom 186**; the §8h phrase form **254
  drawn, headroom 262**.

The blocks above already carry both steps, so an implementer following this plan does not
rediscover them. Do NOT re-lengthen the confirm body: 64 characters of headroom is a
failing gate, not a near miss. (The spec is unchanged — §4.5 names this drop order itself.)

- [ ] **Step 2: The harness tests, and every helper they need (RED).** Create `gui/composer_hashlock_test.go`. This is the whole file as the build gate wrote and ran it — the seven helpers round 0 only NAMED (`composerStateForTest`, `runComposerAddPath`, `tapRow`, `holdConfirm`, `tapPassphraseKey`, `waitDone`, `groupBy`, `loadHashlockCorpusForGUI`) are written out, and each carries the mechanism it depends on in its own comment:

```go file=gui/composer_hashlock_test.go mode=whole
package gui

import (
	"encoding/hex"
	"encoding/json"
	"image"
	"os"
	"strings"
	"testing"
	"time"

	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/op"
	"seedhammer.com/md"
)

// The anchor phrase and the corpus digests (hashlock/testdata/hashlock-v0.8.json,
// derivation row 0) -- typed on the real keyboard through the real flow.
const (
	hashlockAnchorPhrase = "correct horse battery staple"
	hashlockAnchorHardH  = "3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12"
	hashlockAnchorSHA_H  = "b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb"
	hashlockMixedPhrase  = "Correct Horse Battery Staple"
)

// composerStateForTest is an empty policy shape with one path being added --
// the minimal state runComposerAddPath's callers need before a path exists.
//
// Wrapper is wsh: a key-less path is wsh-only (composer_shape.go:250, spec
// §4b/C16) and md.ComposeWrapper's zero value is ComposeTr, which REFUSES a
// key-less path outright ("This build will not put a key-less path in
// taproot") -- the very screen these tests drive through never appears under
// the zero-value state.
func composerStateForTest(t *testing.T) *composerState {
	t.Helper()
	return &composerState{list: md.PathList{Wrapper: md.ComposeWsh}}
}

// hashlockKbdFor captures the *PassphraseKeyboard hashlockPhraseFlow registers
// via hookPPWidget, keyed by the harness that is driving it.
//
// sessionHarness (gui/unlock_session_test.go) carries no widgets map of its
// own, and this file does not modify that struct -- so the capture lives here,
// alongside tapPassphraseKey, the only place that reads it.
var hashlockKbdFor = map[*sessionHarness]*PassphraseKeyboard{}

// runComposerAddPath drives composerAddPath (the CREATION entry point, where a
// false from composerHashEdit deletes the path -- spec §4.6) on the touch harness.
//
// p.display is set to sh2DisplaySize: the default 240x240 test display is
// narrower than the passphrase keyboard (340 px), which pushes q/p/a/l off the
// canvas -- reachable by a hit test, unreachable by a finger (the rule
// passphrase_flow_test.go states for exactly this reason).
func runComposerAddPath(t *testing.T, st *composerState, s *syswSession) *sessionHarness {
	t.Helper()
	p := newPlatform()
	p.display = sh2DisplaySize
	ctx := NewContext(p)
	ctx.sysw = s
	returned := false
	h := &sessionHarness{t: t, ctx: ctx, done: &returned}
	passphraseWidgetHook = func(name string, w any) {
		if name != "kbd" {
			return
		}
		if k, ok := w.(*PassphraseKeyboard); ok {
			hashlockKbdFor[h] = k
		}
	}
	frame, drawer, quit := runUITouch(ctx, func() {
		composerAddPath(ctx, &descriptorTheme, st)
		returned = true
	})
	h.frame, h.drawer = frame, drawer
	t.Cleanup(func() {
		quit()
		passphraseWidgetHook = nil
		delete(hashlockKbdFor, h)
	})
	return h
}

// tapRow selects row i of an n-row composerPickScreen page by touch (the
// zero-Button Clickables plateHitPoints already knows how to find,
// unlock_platelist_test.go) and takes it -- the same "tap selects, Button3
// takes" contract composer_pick_touch_test.go exercises directly. A row
// count that does not match what is actually drawn fails loudly rather than
// silently tapping the wrong target.
func (h *sessionHarness) tapRow(i, n int) {
	h.t.Helper()
	pts := plateHitPoints(h.ctx, h.drawer())
	if len(pts) != n {
		h.t.Fatalf("tapRow(%d, %d): the screen drew %d touch targets, not %d", i, n, len(pts), n)
	}
	tap(&h.ctx.Router, h.drawer(), pts[i])
	h.next("after selecting the row")
	h.tapNav(Button3)
}

// holdConfirm holds Button3 (the ConfirmWarningScreen hold gesture) past
// confirmDelay, then RELEASES.
//
// It cannot reuse wipe_guard_test.go's sessionHarness.hold verbatim: that
// helper never sends a release, which is fine for its own callers (one hold
// per test). This route holds SEVERAL ConfirmWarningScreens in sequence (the
// key-less consent, a method warning, the final Hash lock confirm), and
// EventRouter.Events (gui/event.go) tracks exactly ONE pointer contact
// GLOBALLY: while `pointer.pressed` is true it reuses the STALE
// `pointer.pressedTag` instead of hit-testing the current frame. Measured:
// two sequential holds with no release in between "succeeded" (a frame kept
// coming back) but the second one never left 0% progress, because its press
// event was routed to the FIRST screen's now-defunct Clickable, which nobody
// still polls. The release resets `pointer.pressed`, so the NEXT hold's press
// gets a fresh hit test against the CURRENT screen.
//
// Tolerant of the flow ending here (mirrors unlock_session_test.go's
// holdDiscardConfirm): the LAST hold in several of these tests is the one
// that assigns the digest and lets composerAddPath return, so no further
// frame legitimately follows it.
func (h *sessionHarness) holdConfirm() {
	h.t.Helper()
	dims := h.ctx.Platform.DisplaySize()
	sz := assets.NavBtnPrimary.Bounds().Size()
	ys := [3]int{leadingSize, (dims.Y - sz.Y) / 2, dims.Y - leadingSize - sz.Y}
	pos := image.Pt(dims.X-sz.X/2, ys[int(Button3-Button1)]+sz.Y/2)
	d := h.drawer()
	tag, _, hit := d.Hit(pos)
	if !hit {
		h.t.Fatalf("holdConfirm: no touch target at %v", pos)
	}
	if c, ok := tag.(*Clickable); !ok || (c.Button != Button3 && c.AltButton != Button3) {
		h.t.Fatalf("holdConfirm: the target at %v is %v", pos, tag)
	}
	h.ctx.Router.Events(d, PointerEvent{Pressed: true, Entered: true, Pos: pos}.Event())
	h.next("hold press")
	time.Sleep(confirmDelay)
	if c, ok := h.frame(); ok {
		h.content = c
	}
	h.ctx.Router.Events(d, PointerEvent{Pressed: false, Entered: true, Pos: pos}.Event())
	if c, ok := h.frame(); ok {
		h.content = c
	}
}

// waitDone pumps frames until composerAddPath has returned (the *done flag
// runComposerAddPath set is flipped synchronously, before the underlying
// goroutine exits, so the pump's final ok==false confirms it rather than
// racing it).
func (h *sessionHarness) waitDone() {
	h.t.Helper()
	for i := 0; i < 256; i++ {
		if _, ok := h.frame(); !ok {
			if !*h.done {
				h.t.Fatalf("the session ended without composerAddPath returning")
			}
			return
		}
		if *h.done {
			return
		}
	}
	h.t.Fatalf("composerAddPath never returned after 256 frames")
}

// tapPassphraseKey types one character on the harness's registered
// PassphraseKeyboard, cycling pages by touch until the character's page is up
// -- modelled on ppHarness.typeRune (passphrase_flow_test.go), adapted to
// sessionHarness because that is the type runComposerAddPath returns.
func (h *sessionHarness) tapPassphraseKey(r rune) {
	h.t.Helper()
	kbd, ok := hashlockKbdFor[h]
	if !ok {
		h.t.Fatal("no *PassphraseKeyboard was registered for this harness")
	}
	for range len(ppPages) + 1 {
		if tag := ppTagFor(kbd, func(k ppKey) bool { return k.action == ppRune && k.r == r }); tag != nil {
			h.tapAt(h.point(tag, "key "+string(r)))
			h.next("after typing a character")
			return
		}
		cyc := ppTagFor(kbd, func(k ppKey) bool { return k.action == ppPageCycle })
		if cyc == nil {
			h.t.Fatalf("no page-cycle key on keyboard page %d", kbd.page)
		}
		h.tapAt(h.point(cyc, "page-cycle key"))
		h.next("after cycling the keyboard page")
	}
	h.t.Fatalf("%q is not typeable on any keyboard page", string(r))
}

// tapAt and point mirror ppHarness's (passphrase_flow_test.go), adapted to
// sessionHarness: a tap aimed at the centre of a hit area actually drawn,
// failing loudly if that area is undrawn, off-panel, or covered.
func (h *sessionHarness) tapAt(pos image.Point) {
	h.t.Helper()
	tap(&h.ctx.Router, h.drawer(), pos)
}

func (h *sessionHarness) point(tag op.Tag, what string) image.Point {
	h.t.Helper()
	d := h.drawer()
	b, ok := d.TagBounds(tag)
	if !ok {
		h.t.Fatalf("%s: no hit area was drawn -- unreachable by touch", what)
	}
	c := b.Min.Add(b.Max).Div(2)
	screen := image.Rectangle{Max: h.ctx.Platform.DisplaySize()}
	if !c.In(screen) {
		h.t.Fatalf("%s: hit area %v lies off the %v panel -- unreachable by a finger", what, b, screen)
	}
	if hit, _, ok := d.Hit(c); !ok || hit != tag {
		h.t.Fatalf("%s: hit area %v is covered by another target (%v)", what, b, hit)
	}
	return c
}

// typeOnPassphraseKeyboard taps each character of s on the four-page printable
// keyboard.
func typeOnPassphraseKeyboard(t *testing.T, h *sessionHarness, s string) {
	t.Helper()
	for _, r := range s {
		h.tapPassphraseKey(r)
	}
}

func hashlockHashHex(h *[32]byte) string { return hex.EncodeToString(h[:]) }

// groupBy inserts a space every n runes -- the corpus's own "grouped" refusal
// shape (hashlock.IsMS1Shaped strips it right back out).
func groupBy(s string, n int) string {
	var b strings.Builder
	for i, r := range s {
		if i > 0 && i%n == 0 {
			b.WriteByte(' ')
		}
		b.WriteRune(r)
	}
	return b.String()
}

// hashlockCorpusRow is the one field these GUI tests read back from the
// vendored corpus: the derivation row for a given (untouched) phrase.
type hashlockCorpusForGUI struct {
	rows map[string]struct{ SHA256H string }
}

func (c hashlockCorpusForGUI) row(t *testing.T, phrase string) struct{ SHA256H string } {
	t.Helper()
	r, ok := c.rows[phrase]
	if !ok {
		t.Fatalf("the corpus has no derivation row for %q", phrase)
	}
	return r
}

// loadHashlockCorpusForGUI reads hashlock/testdata via a path RELATIVE TO THIS
// PACKAGE (`go test` runs with gui/ as its working directory, and hashlock/ is
// a sibling of it), not a duplicate copy: the hashlock package already owns
// the vendored corpus and its provenance pin (Task 1), and this file reads the
// SAME bytes rather than re-vendoring them.
func loadHashlockCorpusForGUI(t *testing.T) hashlockCorpusForGUI {
	t.Helper()
	raw, err := os.ReadFile("../hashlock/testdata/hashlock-v0.8.json")
	if err != nil {
		t.Fatalf("reading the vendored hashlock corpus: %v", err)
	}
	var doc struct {
		Derivation []struct {
			Phrase  string `json:"phrase"`
			SHA256H string `json:"sha256_h"`
		} `json:"derivation"`
	}
	if err := json.Unmarshal(raw, &doc); err != nil {
		t.Fatalf("parsing the vendored hashlock corpus: %v", err)
	}
	c := hashlockCorpusForGUI{rows: map[string]struct{ SHA256H string }{}}
	for _, r := range doc.Derivation {
		c.rows[r.Phrase] = struct{ SHA256H string }{r.SHA256H}
	}
	return c
}

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
			h.mustReach("What can spend on this path?")
			h.choose(1) // A hash, no keys
			h.mustReach("EXPERIMENTAL")
			h.holdConfirm() // key-less path consent (§8a)
			h.mustReach("Type a hashlock phrase")
			h.tapRow(0, 3)               // Type a hashlock phrase (no payload digests)
			h.mustReach("32-byte value") // the §8i rule modal (composerCopyHashRule)
			h.tapNav(Button3)
			h.mustReach("Hashlock phrase")
			typeOnPassphraseKeyboard(t, h, tc.phrase)
			h.tapNav(Button3) // OK
			h.mustReach("Which method?")
			h.tapRow(tc.methodRow, 2)
			if tc.method == "sha256" {
				h.mustReach("brainwallet")
				h.holdConfirm()
			} else {
				// 28 characters: no hardened warning.
				h.mustReach("Deriving")
			}
			h.mustReach("Write down this phrase")
			h.holdConfirm()
			if got := st.list.Paths[len(st.list.Paths)-1].Hash; got == nil || hashlockHashHex(got) != tc.want {
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
		h.mustReach("What can spend on this path?")
		h.choose(1) // A hash, no keys
		h.mustReach("EXPERIMENTAL")
		h.holdConfirm()
		h.mustReach("Type a hashlock phrase")
		h.tapRow(0, 3)
		h.mustReach("32-byte value") // the §8i rule modal (composerCopyHashRule)
		h.tapNav(Button3)
		h.mustReach("Hashlock phrase")
		typeOnPassphraseKeyboard(t, h, phrase)
		h.tapNav(Button3)
		h.mustReach("Which method?")
		h.tapRow(1, 2) // sha256: instant
		h.mustReach("brainwallet")
		h.holdConfirm()
		h.mustReach("Write down this phrase")
		h.holdConfirm()
		if got := st.list.Paths[len(st.list.Paths)-1].Hash; got == nil || hashlockHashHex(got) != row.SHA256H {
			t.Fatalf("%q: path hash = %v, want %s", phrase, got, row.SHA256H)
		}
	}
}

// Spec §4.6: Back at every inner step keeps the phrase and never deletes the
// path; only Back at `Which hash?` returns false (and deletes it at creation).
func TestHashlockBackContractKeepsThePath(t *testing.T) {
	st := composerStateForTest(t)
	h := runComposerAddPath(t, st, composerSessionWith(nil, nil))
	h.mustReach("What can spend on this path?")
	h.choose(1) // A hash, no keys
	h.mustReach("EXPERIMENTAL")
	h.holdConfirm()
	h.mustReach("Type a hashlock phrase")
	h.tapRow(0, 3)
	h.mustReach("32-byte value") // the §8i rule modal (composerCopyHashRule)
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
	h.mustReach("brainwallet")
	h.tapNav(Button1) // decline -> method pick, phrase intact
	h.mustReach("Which method?")
	h.tapRow(0, 2)
	h.mustReach("Deriving")
	h.tapNav(Button1) // Back during derivation -> method pick
	h.mustReach("Which method?")
	h.tapRow(1, 2)
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
	h.mustReach("Type a hashlock phrase")
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
	h.mustReach("What can spend on this path?")
	h.choose(1) // A hash, no keys
	h.mustReach("EXPERIMENTAL")
	h.holdConfirm()
	h.mustReach("Type a hashlock phrase")
	h.tapRow(0, 3)
	h.mustReach("32-byte value") // the §8i rule modal (composerCopyHashRule)
	h.tapNav(Button3)
	h.mustReach("Hashlock phrase")
	typeOnPassphraseKeyboard(t, h, hashlockAnchorPhrase)
	h.tapNav(Button3)
	h.mustReach("Which method?")
	h.tapRow(1, 2)
	h.mustReach("brainwallet")
	h.tapNav(Button1)
	h.mustReach("Which method?")
	h.tapRow(0, 2)
	h.mustReach("Deriving")
	h.mustReach("Write down this phrase")
	h.holdConfirm()
	if got := st.list.Paths[0].Hash; got == nil || hashlockHashHex(got) != hashlockAnchorHardH {
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
			h.mustReach("What can spend on this path?")
			h.choose(1) // A hash, no keys
			h.mustReach("EXPERIMENTAL")
			h.holdConfirm()
			h.mustReach("Type a hashlock phrase")
			h.tapRow(0, 3)
			h.mustReach("32-byte value") // the §8i rule modal (composerCopyHashRule)
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
		{"nineteen-characters", 0, true},   // 19 chars, hardened -> 72-days modal
		{"twenty--characters!!", 0, false}, // 20 chars, hardened -> no modal
		{"twenty--characters!!", 1, true},  // sha256 -> always
	} {
		st := composerStateForTest(t)
		h := runComposerAddPath(t, st, composerSessionWith(nil, nil))
		h.mustReach("What can spend on this path?")
		h.choose(1) // A hash, no keys
		h.mustReach("EXPERIMENTAL")
		h.holdConfirm()
		h.mustReach("Type a hashlock phrase")
		h.tapRow(0, 3)
		h.mustReach("32-byte value") // the §8i rule modal (composerCopyHashRule)
		h.tapNav(Button3)
		h.mustReach("Hashlock phrase")
		typeOnPassphraseKeyboard(t, h, tc.phrase)
		h.tapNav(Button3)
		h.mustReach("Which method?")
		h.tapRow(tc.method, 2)
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
	h.mustReach("What can spend on this path?")
	h.choose(1) // A hash, no keys
	h.mustReach("EXPERIMENTAL")
	h.holdConfirm()
	h.mustReach("Which hash?")
	h.tapRow(2, 5)               // the phrase row sits after the two payload rows
	h.mustReach("32-byte value") // the §8i rule modal (composerCopyHashRule)
	h.tapNav(Button3)
	h.mustReach("Hashlock phrase")
	typeOnPassphraseKeyboard(t, h, hashlockAnchorPhrase)
	h.tapNav(Button3)
	h.mustReach("Which method?")
	h.tapRow(1, 2)
	h.mustReach("brainwallet")
	h.holdConfirm()
	h.mustReach("matches hash 1 in the payload")
}
```

The helpers are no longer a placeholder, and six of the build gate's twelve fixes live in
this file. Each one is a mechanism an implementer would otherwise have to rediscover:

- **Fix 5 — the display size.** `runComposerAddPath` sets `p.display = sh2DisplaySize`.
  The default 240×240 test display is narrower than the 340 px passphrase keyboard, which
  pushes `q`/`p`/`a`/`l` off the canvas — reachable by a hit test, unreachable by a
  finger. That is the rule `gui/passphrase_flow_test.go:28-31` states for exactly this
  reason; without it, keys the anchor phrase needs are never drawn.
- **Fix 6 — the state is wsh.** `composerStateForTest` returns
  `&composerState{list: md.PathList{Wrapper: md.ComposeWsh}}`. `md.ComposeWrapper`'s zero
  value is `ComposeTr` (`md/compose.go:32`, iota 0), and a key-less path is refused
  outright under `tr` (`gui/composer_shape.go:250`, "This build will not put a key-less
  path in taproot") — so a zero-value state never reaches the screen these tests exist to
  drive.
- **Fix 7 — the spend-kind choice comes first.** Every flow opens with
  `h.mustReach("What can spend on this path?"); h.choose(1) // A hash, no keys`.
  `composerAddPath` shows that `ChoiceScreen` before anything else; "EXPERIMENTAL" appears
  only after the key-less arm is chosen. Round 0's tests started at "EXPERIMENTAL" and
  could never get there.
- **Fix 8 — the §8i rule modal's needle is `32-byte value`.** That modal is
  `showError(ctx, th, title, composerCopyHashRule())` with `title = "Path N hash"`. "Hash
  lock" is the CONFIRM screen's title, not this one; round 0's test text and its own
  `composer_hash.go` disagreed on the point. The needle is now a substring of the rule
  body itself.
- **Fix 9 — the no-payload frame's needle is `Type a hashlock phrase`.** With
  `composerSessionWith(nil, nil)` there are 0 payload digests, so Task 3's design replaces
  the lead with `composerCopyHashlockNoPayloadLead()` and the literal "Which hash?" is not
  on that frame at all. The phrase row's LABEL is present under either lead. (Same
  mechanism as fix 12, in this plan's own tests rather than a pre-existing one.)
- **Fix 10 — no stray `Button3` after `tapRow`.** `tapRow` already implements the plan's
  own "tap selects, Button3 takes" contract, so round 0's extra `h.tapNav(Button3)` after
  the method pick queued a SECOND click that landed on the next screen and misfired its
  confirm state. Every test that reached that line hung at the following screen until it
  was removed.
- **A thirteenth change the gate's fix table does NOT list: `hashHex` is
  `hashlockHashHex`.** Round 0 declared `func hashHex(h *[32]byte) string` in this file,
  and `gui` already has a `hashHex` — `gui/seal_fixture_test.go:172`, `func hashHex(h
  [16]byte) string`. Same package, so the plan as written was a redeclaration and would not
  compile. The gate renamed this file's helper (verified for this fold by grepping the fork
  at `c4a64fc`). Nothing else in the file changed name.
- **Fix 11 — `holdConfirm` must RELEASE.** It cannot reuse `wipe_guard_test.go`'s
  `sessionHarness.hold`, which never sends a release. `EventRouter.Events`
  (`gui/event.go:14-15`) tracks exactly ONE pointer contact GLOBALLY: while
  `pointer.pressed` is true it reuses the STALE `pointer.pressedTag` instead of hit-testing
  the current frame. This route holds several `ConfirmWarningScreen`s in sequence (key-less
  consent → a method warning → the final Hash lock confirm), so without an explicit release
  the second and later holds route to the FIRST screen's now-defunct `Clickable` and never
  leave 0% — no crash, no wrong-screen frame, just a stuck confirm. Measured directly with
  an isolated debug test before the cause was traced to `event.go`'s single global pointer
  struct. **MUTATION: delete the release `PointerEvent{Pressed: false, …}` from
  `holdConfirm` → every test with two or more holds hangs at its second one.** The comment
  in the file records the mechanism so a future reader does not "simplify" it away.

Run: `go test -count=1 -run TestHashlock ./gui/` — Expected: does not compile.

- [ ] **Step 3: The route.** Replace Task 3's stub `gui/composer_hashlock.go` with this file, as built:

```go file=gui/composer_hashlock.go mode=whole
package gui

import (
	"encoding/hex"
	"fmt"
	"image"
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

(Layout calls mirror `unlockDerive` at `gui/unlock_kdf.go:242-290`; the frame is drawn
INSIDE the progress callback, as the countdown needs a frame per step. `image` is in the
import block above — round 0 used `image.Pt` without importing it, and the build gate
added the import.)

Add the `hashByPhrase` field to `composerState` in `gui/composer_state.go`, next to `bound`:

```go file=gui/composer_state.go mode=fragment
	// hashByPhrase records that AT LEAST ONE path's hash was set through the
	// phrase route (H2), so Done's §8h form names the phrase/method as the
	// backup rather than a bare preimage (composerCopyHashEveryPathFor).
	hashByPhrase bool
```

and at `gui/composer_shape.go:443` swap the §8h call, so the line reads:

```go file=gui/composer_shape.go mode=fragment
				showError(ctx, th, "Spend paths", composerCopyHashEveryPathFor(st))
```

with, appended to `gui/composer_copy.go`:

```go file=gui/composer_copy.go mode=fragment
func composerCopyHashEveryPathFor(st *composerState) string {
	if st.hashByPhrase {
		return composerCopyHashEveryPathPhrase()
	}
	return composerCopyHashEveryPath()
}
```

- [ ] **Step 4: GREEN, then the mutations.**

Run: `go vet ./gui/ && go test -count=1 -run 'TestHashlock|TestWhichHash|TestComposerHash|TestComposerCopy|TestModalsThisBlockTouchesAreDrawnInFull' ./gui/`
Expected: PASS. Mutations, each reverted, with the build gate's MEASURED failure in place
of round 0's prediction where the two differ:

| Mutation | Measured failure |
| --- | --- |
| fold `phrase` through `seal.NormalisePassphrase` in `hashlockPhraseFlow` before `ValidatePhrase` | `TestHashlockPhraseRouteDoesNotNormalise`: `"Correct Horse Battery Staple": path hash = …, want 95d4447…` |
| the confirm's Back returns `hashlockBackToWhichHash` | `TestHashlockBackContractKeepsThePath`: `never reached "Which method?"` |
| `composerHashEdit` returns `false` from the phrase route's Back | `TestHashlockBackContractKeepsThePath` — but it fails EARLIER than round 0 claimed: at `never reached "Type a hashlock phrase"` (the very next inner Back), **not** at the path-count assertion. The mutation is still caught by this test; only the plan's description of *where* was imprecise (gate, Task 4 mutation table). |
| remove the relation line | `TestHashlockConfirmRelationLine`: `never reached "matches hash 1 in the payload"` |
| delete the release event from `holdConfirm` (fix 11's mechanism) | every test with two or more holds hangs at its second one — see Step 2 |
| drop `!f.Unshared` in Task 2 | its own test fails, as in Task 2 |

`go vet ./gui/` reports two PRE-EXISTING complaints that are not this plan's:
`gui/freetext_sizeproof_golden_test.go:111` and `gui/transaction_golden_test.go:104`
(`testing.ArtifactDir requires go1.26 or later (file is go1.25)`). Anything else is new.

- [ ] **Step 5: The whole gui package.**

Run: `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`
Expected: green, partition exhaustive. Measured by the build gate at the wired tree:
**1213 top-level tests, `partition verified exhaustive: 1213 == 1213`, all 24 shards ok,
34 s wall.** That is the `c4a64fc` suite plus the 8 new top-level tests this plan adds to
`gui` and `codex32` (`TestWhichHashRowsAreLabelKeyed`, the seven `TestHashlock*`, and
`TestDecodeMS1PreimageIsShapeExact`); `hashlock`'s own 6 tests are a separate package and
are not in the 1213. The FIRST run of this shard set is what caught fix 12 (shard 11
failed); a narrow `-run` selection cannot.

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

- [ ] **Step 2: Firmware size.** `export PATH=/nix/var/nix/profiles/default/bin:$PATH; nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`

Expected, measured by the build gate at the fully wired tree:

```
   code    data     bss |   flash     ram
1563384   31852   31004 | 1595236   62856
```

**Flash 1,595,236 B / RAM 62,856 B** against `c4a64fc`'s 1,583,132 / 62,800 —
**+12,104 B flash (+0.76%) and +56 B RAM (+0.09%)** for the whole port: the new `hashlock`
package, `DecodeMS1Preimage`, the label-keyed switch, the phrase route, its screens and
its copy. The rule this applies is spec §7.6's own: PBKDF2 and SHA-256 are already linked
(`seal/pbkdf2.go`, `seal/crypto.go`) and the keyboard already exists, so the stage expects
**a small delta over `c4a64fc`'s 1,583,132 / 62,800** — 0.76% is that. No numeric flash
ceiling is asserted here, because neither the spec nor this plan sets one; the acceptance
is the delta against the named baseline. A materially larger delta means something was
linked that should not have been — measure again before the merge, since the emulator walk
(Step 1) is not in this number.

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

## Build gate folded here

The build gate hand-wired every code block of Tasks 1-4 into a scratch copy of the fork
at `/scratch/code/shibboleth/.tmp/h2-gate` (fork main `c4a64fc` + this plan), ran them,
and reached **GATE GREEN WITH FIXES (12)** plus two prose corrections. Its verbatim report
is `design/agent-reports/hashlock-H2-plan-build-gate.md`. Every fix is folded above, at the
task it belongs to, and the plan's blocks are now the gated tree's own bytes.

**The twelve fixes.**

1. **`gui/composer_copy_test.go` — a row and a count Task 3 had no step for.** Task 3 adds
   `composerCopyHashlockNoPayloadLead`, and `TestComposerCopyTableCoversEveryBody` fails by
   name on any `composerCopy*` function without a `composerCopyTable` row and an exact
   declared count. → Task 3 **Step 3** (new): the row, and 41 → 42.
2. **The same gate, nine more times in Task 4.** Round 0 listed six §8 sections, not nine
   functions; `composerCopyHashlockRefusal`, `composerCopyHashlockRelation` and
   `composerCopyHashEveryPathFor` had no row. → Task 4 Step 1 now carries all nine rows,
   the `hashlock` import both test files need, and the literal 42 → 51.
3. **The §4.5 confirm body did not fit.** Unshortened it drew 484 of 504 characters and was
   CUT. Spec §4.5's drop-order step 1 (reuse block → the brainstorm's two sentences) left
   **64 characters of headroom against a required margin of 80** — still failing. → the
   shortened text is in Task 4 Step 1's block, with the measurement beside it.
4. **The reconciliation line moved out of the confirm modal.** §4.5's drop-order step 2 was
   needed too: "Before you fund this wallet, run ms hashlock…" now lives in
   `composerCopyHashEveryPathPhrase` (Done's §8h form). Measured after both steps: confirm
   **290 drawn, headroom 186**; §8h form **254 drawn, headroom 262**. → Task 4 Step 1, and
   the §4.5 note under it. The spec is unchanged; it names this drop order itself.
5. **`p.display = sh2DisplaySize` in `runComposerAddPath`.** The 240×240 default is
   narrower than the 340 px keyboard and pushes `q`/`p`/`a`/`l` off canvas. → Task 4 Step 2.
6. **`composerStateForTest` returns `md.PathList{Wrapper: md.ComposeWsh}`.**
   `md.ComposeWrapper`'s zero value is `ComposeTr` (`md/compose.go:32`), under which a
   key-less path is refused outright (`gui/composer_shape.go:250`). → Task 4 Step 2.
7. **The flow opens at `What can spend on this path?` and chooses `A hash, no keys`.**
   "EXPERIMENTAL" is only reachable through that `ChoiceScreen`. Seven call sites. → Task 4
   Step 2.
8. **The §8i rule modal's needle is `32-byte value`, not `Hash lock`.** Its title is
   `Path N hash`; "Hash lock" belongs to the later confirm screen. Seven call sites. →
   Task 4 Step 2.
9. **The no-payload frame's needle is `Type a hashlock phrase`, not `Which hash?`.** With
   zero payload digests, Task 3's design swaps the lead, so that literal is not drawn.
   Seven call sites. → Task 4 Step 2.
10. **The stray `h.tapNav(Button3)` after each `tapRow` is gone.** `tapRow` already takes
    the row; the second click landed on the NEXT screen and misfired its confirm state.
    Nine call sites. → Task 4 Step 2.
11. **`holdConfirm` holds, waits, and RELEASES.** `EventRouter.Events` (`gui/event.go:14-15`)
    tracks ONE pointer contact globally: while `pointer.pressed` is true it reuses the stale
    `pointer.pressedTag` instead of hit-testing the current frame, so a second hold with no
    release routes to the first screen's defunct `Clickable` and never leaves 0%. → Task 4
    Step 2, with the mechanism in the file's own comment and a MUTATION recorded in Step 4
    (delete the release → every test with two or more holds hangs at its second).
12. **`gui/composer_gates_test.go`'s pump target moves to `Path 1 hash`.** A genuine
    regression in a pre-existing, unrelated test, caused by Task 3's no-payload lead swap
    (`ctx.sysw == nil` → 0 digests → the lead changes; the TITLE is what is invariant).
    Found only by the full 24-shard `gui` run — shard 11 failed. → Task 3 **Step 4** (new),
    because Task 3 is the change that causes it.

**A thirteenth change the gate made but did not list.** `hashHex` in
`gui/composer_hashlock_test.go` is `hashlockHashHex` in the gated tree: `gui` already
declares `func hashHex(h [16]byte) string` at `gui/seal_fixture_test.go:172`, so round 0's
`func hashHex(h *[32]byte) string` was a redeclaration in the same package and would not
have compiled. Re-verified against the fork at `c4a64fc` for this fold. It is folded with
the rest, because the block is now the tree's own bytes — but it is named here so the count
in this section and the count in the gate report can be reconciled.

**The two prose corrections** (Task 1 Step 5's mutation table, now a measured table):

- **Separator-strip scope.** The plan claimed one row fails; **four do** —
  `correct-horse,battery staple`, `a-b,c`, and the two long rows that also carry `-` and
  `,`. Re-measured for this fold against the vendored corpus: 4 of 11 derivation phrases
  contain a `-` or a `,`.
- **The cap literal.** The plan claimed the 100-character refusals row fails too; there is
  no such row — the corpus's one `too-long` row is **101** characters, refused under either
  cap, so only `TestPhraseMaxCharsIsTheCap` fails. Re-measured for this fold: 101.

A third, smaller correction from the gate's Task 4 mutation table is folded at Task 4
Step 4: making `composerHashEdit` return `false` from the phrase route's Back does fail
`TestHashlockBackContractKeepsThePath`, but at `never reached "Type a hashlock phrase"`,
not at the path-count assertion the plan named.

**Whole-suite and size, measured at the wired tree.** `codex32`/`sysw`/`seal`/`hashlock`
green; `gui` 1213 top-level tests, partition verified exhaustive, all 24 shards ok;
firmware **1,595,236 B flash / 62,856 B RAM** (+12,104 / +56 against `c4a64fc`'s
1,583,132 / 62,800). NOT run by the gate, and still owed: the emulator walk
(`cmd/emu/walk_hashlock_phrase.js`, Task 5 Step 1) and the flash.

**The block check.** `scripts/h2-plan-blocks-vs-tree.sh` re-derives the claim this section
makes. Output:

    plan: /scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_hashlock_H2_device.md
    tree: /scratch/code/shibboleth/.tmp/h2-gate

    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:127  whole          hashlock/testdata/hashlock-v0.8.provenance.json  (18 lines, identical)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:150  whole          hashlock/hashlock_test.go                     (213 lines, identical)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:373  whole          hashlock/hashlock.go                          (143 lines, identical)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:555  fragment       codex32/mspayload_test.go                     (42 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:604  fragment       codex32/mspayload.go                          (25 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:658  fragment       gui/composer_hash_test.go                     (34 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:697  fragment       gui/composer_hash_test.go                     (6 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:710  fragment       gui/composer_hash.go                          (2 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:717  fragment       gui/composer_hash.go                          (80 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:809  fragment       gui/composer_copy.go                          (6 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:820  fragment       gui/composer_copy_test.go                     (2 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:827  fragment       gui/composer_copy_test.go                     (2 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:836  fragment       gui/composer_gates_test.go                    (8 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:873  fragment       gui/composer_copy.go                          (5 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:883  fragment       gui/composer_copy.go                          (74 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:970  fragment       gui/composer_copy_test.go                     (9 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:982  fragment       gui/composer_copy_test.go                     (23 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1011  fragment       gui/composer_copy_test.go                     (8 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1028  fragment       gui/modal_fits_test.go                        (8 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1039  fragment       gui/modal_fits_test.go                        (21 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1087  whole          gui/composer_hashlock_test.go                 (531 lines, identical)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1678  whole          gui/composer_hashlock.go                      (213 lines, identical)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1901  fragment       gui/composer_state.go                         (4 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1910  fragment       gui/composer_shape.go                         (1 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1916  fragment       gui/composer_copy.go                          (6 lines, verbatim substring)

    25 blocks checked, 0 FAIL

    NOT COVERED by this script:
      * 7 fenced blocks carry no file= header (bash recipes, illustrative
        snippets); nothing here runs or checks them:
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:119  ```bash
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:540  ```bash
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:636  ```bash
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:853  ```bash
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:1957  ```bash
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:1975  ``` (no info string)
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:1993  ```bash
      * every PROSE claim: expected test names, mutation outcomes, headroom and
        firmware numbers, spec references, file:line citations.
      * whether the tree is GREEN -- this compares TEXT only; `go test` and the
        gate report are what say the text works.
      * files the plan modifies without carrying a block for them.

---


## Self-review

1. **Spec coverage.** §2 → Task 1 (`ValidatePhrase`, `IsMS1Shaped`, the refusals rows) and Task 4 (through the screen); §3 → Task 1 (`DeriveHardened` on `seal.NewDeriver` with the slice salt; the constants; the lockstep mutations); §4.1 → Task 3; §4.2-§4.5 → Task 4 (the flow, the copy, both gates); §4.6 → Task 4's loop and the Back test through `composerAddPath`; §4.7 → `composerCopyHashEveryPathFor`; §5 → Task 3; §6 → Task 2; §7.1 → Task 1's tests; §7.2/§7.3 → Tasks 4/3; §7.4 → Task 2; §7.5 → Task 5; §7.6 → Task 5 Step 2; §8 → Task 6 (H4); §9 → nothing to build.
2. **Placeholders.** None left in the Go. The Task 4 Step 2 harness helpers round 0 only NAMED are written out in full, as the build gate wrote and ran them, and `scripts/h2-plan-blocks-vs-tree.sh` proves every block is the gated tree's own bytes. The one thing still written as prose rather than code is Task 5 Step 1's emulator walk: the gate did not run it (out of scope), and its keyboard mapping is probed on the live emulator as `walk_verify.js` did — so it is the plan's one un-gated executable artifact, and the controller runs it before the post-implementation review.
3. **Type consistency.** `seal.NewDeriver(passphrase, salt []byte, iterations int) *Deriver`, `Step(n int) bool`, `Done()/Total() int`, `Key() []byte`, `Wipe()` (`seal/pbkdf2.go:85-182`); `composerPickScreen(ctx, th, title, lead string, rows []string) (int, bool)` (`composer_paged.go:259`); `composerConfirmScreen(ctx, th, title, body string) bool` (`composer_shape.go:77`); `composerConfirmBody(body string) string` (`composer_copy.go:32`); `Hash *[32]byte` (`md/compose.go:167`); `composerSessionWith(public, secret []string) *syswSession` (`composer_door_test.go:15`); `ParsePrefix(frag string) (Fields, error)`, `Fields.Unshared` (`codex32/polish.go:82,71`); `NewSeed(hrp string, threshold int, id string, shareIdx rune, data []byte) (String, error)` (`codex32/codex32.go:279`); `composerPickScreenMaxRows = 24` (`composer_paged.go:224`).
