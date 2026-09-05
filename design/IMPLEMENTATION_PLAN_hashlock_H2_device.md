# Hashlock H2 — Device Leg Implementation Plan (SeedHammer fork)

**IMPLEMENTATION RECORD (2026-09-05, git's clock):** executed on fork branch `hashlock-h2` (ONE opus implementer; report `design/agent-reports/hashlock-H2-implementation-report.md`): 17b3979 (six commits: Tasks 1-5 + Task 6 records + Task 7 F-474 in-phase), then two controller folds 26fd1dd and a1fd139; merged to fork `main` with `--no-ff` at c284484 (then h3-seam-corpus at b9a9a30). Engrave records branch merged at ccad644 (then h3-composer-spec 657f40f, h3-seam-corpus 059833f). Controller gates at 17b3979: four packages ok; gui 1222 / 24 shards; checker 26/0; **emulator walk PASSED** (fresh port, playwright, 48.3 s: `b867db87..edbc96cb` sha256 chars 28; control `c8043156..253e7389`; mixed-case `95d44470..2297a7ff`; hardened `3cf5d421..b70a4c12`; reconcile frame; `Path 1: hash only`). Firmware at 17b3979: 1,597,276 / 62,856 (+14,144 / +56 vs c4a64fc). Post-implementation: opus review (`hashlock-H2-post-impl.md`) 0C/2I/3M/2N -- **I-1** `first8..last8` and `chars: n` survived mutation of the whole suite (now asserted; both mutations RED), **I-2** F-481 graded gating (the phrase screen drew no readout: 8 px CutBottom; removed, raster-verified 0 -> 260 ink) -- fold 26fd1dd, sonnet r1 GREEN. Ultracode round (wf_7b02c125-9b9: interruption, geometry, walk-control, host-device e2e, records-claims lenses; two sonnet refuters per C/I): the Critical and two Importants were F-481 again; NEW **e2e I-1** the other-path line hard-coded 'two phrases to back up', wrong on the three-hashlock wallet (now count-free, n=3 row, no-digit assertion), **interruption M-1** Remove path never re-synced `hashByPhrase` (sync call + flow test) -- fold a1fd139, sonnet r2 GREEN. The walk FAILS under a mutated digest display and a mutated hardened derivation (walk-control lens, two mutated emulators). Real plan defect found by the implementer: Task 3 used `st.hashByPhrase` before Task 4 added it (a gate that wires all tasks at once cannot see order). Follow-ups: F-474, F-475, F-481 CLOSED; F-482..F-489 filed. Device acceptance (H4): ASSUMED at the operator's word ("Proceed as if device tests passed for now"), not measured; flash at the operator's word only.

**STATUS: R0 GREEN 2026-09-05 (0 Critical / 0 Important open).** Round 0 (plan `02abee6`): five lenses in parallel -- fidelity (opus, `hashlock-H2-plan-R0-r0-fidelity.md`, 0C/6I/7M/4N), tests/mutation (sonnet, `-tests.md`, 1C/6I/2M/1N), journey walk (opus, `-journey.md`, 0C/5I/3M/2N), adversarial failure states (opus, `-adversarial.md`, 2C/4I/7M/5N), coverage + comprehension (sonnet, `-coverage.md`, 0C/1I/3M/1N) -- then a refute pass (sonnet, `-refute.md`: 25 C/I -> 24 CONFIRMED + 1 PARTIAL, 16 distinct defects), ONE fold (`f60c2df`, opus fold agent, plan and gated tree together, 21 mutations RED->GREEN), r1 fold verification (sonnet, `hashlock-H2-plan-R0-r1-fold-verification.md`) **GREEN** with two citation Nits folded at the commit after it. Lens-closure: fidelity, tests, journey, adversarial, coverage, refute, fold-verification. The one gate not yet executed is Task 5 Step 1's emulator walk -- the implementer writes and runs it, the controller re-runs it before the post-implementation review.

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
| `hashlock/hashlock_test.go` | Create | the lockstep gate: sha pin, 11 derivation rows, 15 refusals rows, the kind row against the corpus `digest`, `DeriveHardened`'s abandon contract, the `minMS1Len` boundary, `TrimSpace`'s effect, mutations |
| `hashlock/testdata/hashlock-v0.8.json`, `hashlock-v0.8.provenance.json` | Create (vendored) | byte-identical corpus + pin |
| `codex32/mspayload.go` | Modify (append) | `DecodeMS1Preimage` |
| `codex32/mspayload_test.go` | Modify (append) | §7.4 |
| `gui/composer_hash.go` | Modify | header comment (spec §1 item 5); `composerHashEdit` label-keyed with the phrase row and the loop; `composerHashRows`; `composerHashByPhraseSync` |
| `gui/composer_hashlock.go` | Create | `hashlockPhraseFlow`, `hashlockMethodPick`, `hashlockDeriveFlow` (with the F-93 wakeup and the zero-state frame), `hashlockDerivingLead`, `hashlockRelationLine`, `hashlockOtherPathLine` |
| `gui/composer_copy.go` | Modify (append) | `composerCopyHashlock*` strings; `composerCopyHashEveryPathPhrase` |
| `gui/composer_shape.go` | Modify (one line) | Done's §8h picks the phrase-route form when a hash was set by phrase |
| `gui/composer_state.go` | Modify | `composerState.hashByPhrase bool` (set by the phrase route; dropped by `composerHashByPhraseSync` when no path carries a hash) |
| `gui/composer_copy_test.go` | Modify (rows + count) | the AST-scan copy gate: twelve new `composerCopyTable` rows and the `declared` literal 41 → 42 → 53 (build gate fixes 1, 2; R0 round 0 added two bodies) |
| `gui/modal_fits_test.go` | Modify (rows + a test) | three new `TestModalsThisBlockTouchesAreDrawnInFull` rows, and `TestConfirmScreensThisBlockTouchesAreDrawnInFull` for the three `ConfirmWarningScreen` bodies (R0 round 0, fidelity I-3) |
| `gui/composer_gates_test.go` | Modify (one row) | Task 3's no-payload lead moves an EXISTING test's pump target (build gate fix 12) |
| `gui/composer_hash_test.go` | Modify (append) | §7.3 switch tests |
| `gui/composer_hashlock_test.go` | Create | §7.2 harness tests |
| `cmd/emu/walk_hashlock_phrase.js` | Create | §7.5 |
| engrave `design/FOLLOWUPS.md`, continuity | Modify | records |

**Gate coverage.** `scripts/plan-build-gate-go.sh` recognises `gui/composer_*.go` (so
`gui/composer_hashlock.go` is assembled) but not `hashlock/*.go` or `codex32/*.go`;
the controller hand-wires the whole plan into a scratch copy of the fork and runs
`go vet` + `go test ./hashlock/ ./codex32/ ./sysw/ ./seal/` + `go test -run
'TestComposer|TestHashlock|TestWhichHash|TestModals|TestConfirmScreens' ./gui/` before
review, then the gui shard script; output in the plan commit. Whole-package gui runs use `scripts/gui-shard-test.sh` (engrave).

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
		PreimageHex   string `json:"preimage_hex"`
		Digest        string `json:"digest"`
		MS1           string `json:"ms1"`
		Entr32PairMS1 string `json:"entr32_pair_ms1"`
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

// The kind row: the plate's preimage bytes are the corpus's preimage_hex, and
// Digest of that preimage is the corpus's own `digest` CONSTANT -- what the
// confirm modal must show for a --hex X. Compared against the corpus, never
// against a value this Go recomputed (Global Constraints, Rust-primary).
//
// MUTATION: double-hash in Digest (`inner := sha256.Sum256(x[:]); return
// sha256.Sum256(inner[:])`) -> this test fails with
// `kind[0] digest: got 88b8f02c...  want 9a2db2e2...`. The identity check this
// replaced could NOT fail on that mutation (r0 tests C-1 executed it: the
// mutated Digest still returned a value != x, so the test reported PASS).
func TestKindRowPreimageDigest(t *testing.T) {
	c := loadCorpus(t)
	if c.Kind[0].Digest == "" {
		t.Fatal("kind[0] carries no digest constant -- the corpus and this test have drifted")
	}
	x := mustHex(t, c.Kind[0].PreimageHex)
	if got, want := Digest(&x), mustHex(t, c.Kind[0].Digest); got != want {
		t.Fatalf("kind[0] digest: got %x want %x", got, want)
	}
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

// DeriveHardened's OWN abandon contract (r0 tests I-3): `progress` returning
// false must stop the KDF and report ok=false, PROMPTLY -- not after running to
// completion. TestDerivationRowsLockstep passes an always-true progress func, and
// the GUI's hashlockDeriveFlow tracks its own `abandoned` flag, so a
// DeriveHardened that ignored the callback's return value would ship green
// through both. This is the only test that can see it.
//
// MUTATION: drop the early return (`progress(d.Done(), d.Total())` in place of
// `if !progress(...) { return x, false }`) -> ok becomes true, calls becomes 199
// instead of 3, and both assertions below fail.
func TestDeriveHardenedAbandonsWhenProgressSaysStop(t *testing.T) {
	calls := 0
	x, ok := DeriveHardened([]byte("correct horse battery staple"), func(done, total int) bool {
		calls++
		if total != Iterations {
			t.Errorf("progress total = %d, want %d", total, Iterations)
		}
		return calls < 3
	})
	if ok {
		t.Errorf("DeriveHardened returned ok=true after progress abandoned it")
	}
	if calls != 3 {
		t.Errorf("progress was called %d times; abandoning must stop the KDF at the "+
			"third call, not run it to completion (%d calls)", calls, Iterations/500)
	}
	if x != ([32]byte{}) {
		t.Error("an abandoned derivation returned a non-zero value, want the zero value " +
			"(the bytes are deliberately not logged)")
	}
}

// minMS1Len's OWN boundary (r0 tests I-5): the corpus's ms1-shaped refusals are
// all 75-character plates, nowhere near 47/48, so nothing else in this package
// can see the constant move.
//
// MUTATION: minMS1Len = 47 -> the 47-character row is reported ms1-shaped and
// this test fails; minMS1Len = 49 -> the 48-character row fails.
func TestIsMS1ShapedMinLengthBoundary(t *testing.T) {
	if minMS1Len != 48 {
		t.Errorf("minMS1Len = %d -- ms-cli's MIN_MS1_LEN is 48", minMS1Len)
	}
	// The two inputs are LITERAL 47 and 48 characters, not derived from
	// minMS1Len: a test that built its own boundary out of the constant it is
	// pinning would move with the mutation and never fail on it.
	short := "ms1" + strings.Repeat("q", 44) // 47 characters
	long := "ms1" + strings.Repeat("q", 45)  // 48 characters
	if len(short) != 47 || len(long) != 48 {
		t.Fatalf("boundary inputs are %d and %d characters", len(short), len(long))
	}
	if IsMS1Shaped(short) {
		t.Errorf("%d characters must be BELOW the ms1 shape bound", len(short))
	}
	if !IsMS1Shaped(long) {
		t.Errorf("%d characters is the bound and must be ms1-shaped", len(long))
	}
	// The bound is applied to the STRIPPED length, not the typed one: the same
	// 47 characters grouped by 5 are still too short, and the same 48 still are not.
	if IsMS1Shaped(displaySpaced(short)) {
		t.Errorf("grouping must not lift a 47-character string over the bound")
	}
	if !IsMS1Shaped(displaySpaced(long)) {
		t.Errorf("grouping must not push a 48-character string under the bound")
	}
}

func displaySpaced(s string) string {
	var b strings.Builder
	for i, r := range s {
		if i > 0 && i%4 == 0 {
			b.WriteByte('-')
		}
		b.WriteRune(r)
	}
	return b.String()
}

// IsMS1Shaped's strings.TrimSpace call is LOAD-BEARING, and this test is what
// says so (r0 tests I-6 claimed it was redundant with the strip loop and should
// be deleted -- measured false, see below).
//
// The strip loop skips exactly ' ', '\t', '\n', '\r', '-' and ','. TrimSpace
// removes every character unicode.IsSpace reports at the ENDS, which is a
// strictly larger set: '\v', '\f', U+0085 and U+00A0 among them. Removing the
// call therefore changes the answer for real inputs -- and in the WRONG
// direction, since the host's own looks_like_ms1 is `is_ms1_shaped(&raw.trim()
// .to_ascii_lowercase())` (ms-cli argv_guard.rs) and Rust's str::trim uses the
// White_Space property, which covers all of them.
//
// MUTATION: `t := strings.ToLower(s)` in place of
// `t := strings.ToLower(strings.TrimSpace(s))` -> every row below except the
// first two fails, measured: '\v', '\f', U+0085, U+00A0 and U+2003 all flip
// from true to false while the host still refuses the plate.
func TestIsMS1ShapedTrimsWhatTheStripLoopCannot(t *testing.T) {
	c := loadCorpus(t)
	plate := c.Kind[0].MS1
	for _, pad := range []string{" ", "\t", "\v", "\f", "\u0085", "\u00a0", "\u2003"} {
		if !IsMS1Shaped(pad + plate) {
			t.Errorf("%q + the plate is not ms1-shaped -- the host trims this character "+
				"before its own shape test, so the port must too", pad)
		}
		if !IsMS1Shaped(plate + pad) {
			t.Errorf("the plate + %q is not ms1-shaped", pad)
		}
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
Expected: PASS (9 tests). Then the mutations, each reverted. What follows is the build
gate's MEASURED outcome (`design/agent-reports/hashlock-H2-plan-build-gate.md`, Task 1
Step 5 table), which corrected two of this plan's own round-0 predictions, plus the four
rows the R0 round 0 fold added and measured:

| Mutation | Measured outcome |
| --- | --- |
| `Salt = append(Salt, 0, 0)` | 22 failures — 11 rows × (hardened X + hardened H) |
| `Iterations = 99999` | 22 failures, the same 11 rows × X and H |
| `phrase = []byte(seal.NormalisePassphrase(string(phrase)))` at the top of `PreimageHardened` | exactly the `Correct Horse Battery Staple` and `  a  b ` rows, 4 failures (X+H each) |
| strip `-`/`,` from the phrase first | **FOUR rows fail, not one.** Gate, verbatim: *"4 rows fail, not 1: `correct-horse,battery staple`, `a-b,c`, and BOTH 64-char rows … because those two rows also contain `-` and `,` … Plan's own claim is wrong; the mutation itself still works as a gate (it does fail), just on 4 rows, not 1."* Re-measured against the vendored corpus for this fold: 4 of the 11 derivation phrases carry `-` or `,` — `correct-horse,battery staple` (28), `a-b,c` (5), `hashlock phrase row: sixty-four printable characters, no hex!!xx` (64) and its `!`-suffixed sibling (65). |
| `IsMS1Shaped` using `codex32.New` | exactly refusals rows 11, 12 and 13 (grouped-by-5, leading/trailing spaces, grouped-by-2) — a checksum parse rejects grouped input |
| the cap literal 99 | **ONLY `TestPhraseMaxCharsIsTheCap` fails.** Gate, verbatim: *"The corpus has no 100-character refusals row — its one `too-long` row is 101 characters (verified: `len(...)==101`), which is refused whether the cap is 99 or 100, so `TestRefusalRowsMatchTheHost` stays green under this mutation. Plan's second clause does not hold for this corpus."* Re-measured for this fold: the sole `too-long` refusals row is 101 characters. |
| `Digest` double-hashes (`inner := sha256.Sum256(x[:]); return sha256.Sum256(inner[:])`) | `TestKindRowPreimageDigest`: `kind[0] digest: got 88b8f02c… want 9a2db2e2…`. **This is the fold's own r0 Critical (adversarial C-2 / tests C-1):** the round-0 body was `if h := Digest(&x); h == x`, an identity check, and this same mutation was executed against it and reported **PASS**. |
| `DeriveHardened` ignores `progress`'s return value | `TestDeriveHardenedAbandonsWhenProgressSaysStop`: `returned ok=true after progress abandoned it` and `progress was called 199 times`. Nothing in round 0 could see this — the lockstep test passes an always-true callback and the GUI wrapper tracks its own `abandoned` flag (r0 tests I-3). |
| `minMS1Len = 47` | `TestIsMS1ShapedMinLengthBoundary`: `47 characters must be BELOW the ms1 shape bound`, plus the grouped row. `minMS1Len = 49` fails the two 48-character rows instead. The boundary inputs are LITERAL 47/48-character strings, not derived from the constant, so the test cannot move with the mutation (r0 tests I-5). |
| drop `strings.TrimSpace` from `IsMS1Shaped` | `TestIsMS1ShapedTrimsWhatTheStripLoopCannot`: ten failures — `\v`, `\f`, U+0085, U+00A0 and U+2003, leading and trailing. **The call is NOT redundant with the strip loop**; the reviewer's "delete it" remedy is declined, with the measurement, in `## R0 round 0 folded here`. |

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

- [ ] **Step 1: The test.** The new test reads the vendored corpus, so the file's
import block gains `encoding/json` and `os`:

```go file=codex32/mspayload_test.go mode=fragment
import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"os"
	"testing"
)
```

Then append:

```go file=codex32/mspayload_test.go mode=fragment
// hashlockCorpus is the vendored ms-codec 0.8.0 corpus, read from the hashlock
// package's own testdata by a path RELATIVE TO THIS PACKAGE (`go test` runs with
// codex32/ as its working directory and hashlock/ is a sibling). One vendored
// copy, one provenance pin (hashlock/testdata/hashlock-v0.8.provenance.json) --
// never a second copy or a literal transcribed into this file.
type hashlockCorpus struct {
	Kind []struct {
		PreimageHex   string `json:"preimage_hex"`
		Digest        string `json:"digest"`
		MS1           string `json:"ms1"`
		Entr32PairMS1 string `json:"entr32_pair_ms1"`
	} `json:"kind"`
	Derivation []struct {
		Phrase    string `json:"phrase"`
		HardenedX string `json:"hardened_x"`
	} `json:"derivation"`
}

func loadHashlockCorpus(t *testing.T) hashlockCorpus {
	t.Helper()
	raw, err := os.ReadFile("../hashlock/testdata/hashlock-v0.8.json")
	if err != nil {
		t.Fatalf("reading the vendored hashlock corpus: %v", err)
	}
	var c hashlockCorpus
	if err := json.Unmarshal(raw, &c); err != nil {
		t.Fatalf("parsing the vendored hashlock corpus: %v", err)
	}
	if len(c.Kind) < 1 || c.Kind[0].PreimageHex == "" || c.Kind[0].Entr32PairMS1 == "" {
		t.Fatalf("corpus shape: %d kind rows", len(c.Kind))
	}
	return c
}

// H2 (SPEC_hashlock_H2_device §6, §7.4): the 0x03 kind has ONE decoder of its own;
// DecodeMS1 keeps refusing it (H0), and the two never share a code path.
//
// Every value here comes from the corpus, never from a literal this file
// transcribed. MUTATION: `copy(preimage[:], d[:32])` in place of `d[1:]` -> the
// full-width comparison below fails with
// `preimage = 03abab...abab, want the corpus's preimage_hex abab...abab`. The `x[0] == 0 && x[31] == 0`
// smoke check this replaced could NOT fail on that mutation (r0 adversarial C-2:
// under it x[0] = 0x03 and x[31] = 0xab, so the && is false and the mutant
// reported PASS -- executed and confirmed for this fold).
func TestDecodeMS1PreimageIsShapeExact(t *testing.T) {
	c := loadHashlockCorpus(t)
	plate := c.Kind[0].MS1
	s, err := New(plate)
	if err != nil {
		t.Fatal(err)
	}
	x, err := DecodeMS1Preimage(s)
	if err != nil {
		t.Fatalf("DecodeMS1Preimage(plate): %v", err)
	}
	if want := mustHexT(t, c.Kind[0].PreimageHex); !bytes.Equal(x[:], want) {
		t.Fatalf("preimage = %x, want the corpus's preimage_hex %x", x, want)
	}
	if _, _, _, err := DecodeMS1(s); err != errMSBadPrefix {
		t.Errorf("DecodeMS1(plate) = %v, want errMSBadPrefix (H0 contract)", err)
	}

	// §7.4's acceptance-record case: the plate ms hashlock actually wrote on
	// the host (design/agent-reports/ms-hashlock-H1-acceptance.md, H1 item 3)
	// decodes to the corpus ANCHOR row's hardened_x. This is the one row that
	// ties this decoder to a host-produced artifact rather than to a corpus
	// string; a decoder that agreed with the corpus but not with ms would pass
	// every other row here.
	const acceptancePlate = "ms10hashsq0p7jaf9gsjjpkjvll2l274w8a388xgqzlewp73scptwxgtjugspvs8tklufg89hqj"
	ap, err := New(acceptancePlate)
	if err != nil {
		t.Fatalf("New(the H1 acceptance plate): %v", err)
	}
	ax, err := DecodeMS1Preimage(ap)
	if err != nil {
		t.Fatalf("DecodeMS1Preimage(the H1 acceptance plate): %v", err)
	}
	if len(c.Derivation) == 0 || c.Derivation[0].Phrase != "correct horse battery staple" {
		t.Fatal("derivation row 0 is not the anchor phrase -- the corpus and this test have drifted")
	}
	if want := mustHexT(t, c.Derivation[0].HardenedX); !bytes.Equal(ax[:], want) {
		t.Errorf("the H1 acceptance plate decodes to %x, want the anchor row's hardened_x %x", ax, want)
	}

	// §7.1's "kind: the entr32 pair" lockstep clause. The SAME 32 bytes under
	// Tag::ENTR are a SEED, not a preimage: the preimage decoder must refuse
	// the sibling on its prefix byte, and DecodeMS1 -- which refuses the hash
	// plate -- must decode it. That is the pair, driven in both directions.
	pair, err := New(c.Kind[0].Entr32PairMS1)
	if err != nil {
		t.Fatalf("New(entr32_pair_ms1): %v", err)
	}
	if _, err := DecodeMS1Preimage(pair); err != errMSBadPrefix {
		t.Errorf("DecodeMS1Preimage(entr32_pair_ms1) err = %v, want errMSBadPrefix", err)
	}
	prefix, lang, entropy, err := DecodeMS1(pair)
	if err != nil {
		t.Fatalf("DecodeMS1(entr32_pair_ms1): %v", err)
	}
	if prefix != msPrefixEntr || lang != 0 {
		t.Errorf("DecodeMS1(entr32_pair_ms1) prefix/language = %d/%d, want %d/0", prefix, lang, msPrefixEntr)
	}
	if want := mustHexT(t, c.Kind[0].PreimageHex); !bytes.Equal(entropy, want) {
		t.Errorf("entr32_pair_ms1 seed = %x, want the same 32 bytes as the hash plate %x", entropy, want)
	}
	for _, tc := range []struct {
		name, s string
		want    error
	}{
		{"entr single", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f", errMSBadPrefix},
		{"a 2-of-N share beginning 0x03", "ms12testaqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdq7pl8qdc5tsp", errMSBadPrefix},
		{"the entr-id 0x03 shape (kind is the prefix byte)", "ms10entrsqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5gz69g08wwtz9", nil},
	} {
		e, err := New(tc.s)
		if err != nil {
			t.Fatalf("New(%s): %v", tc.name, err)
		}
		if _, err := DecodeMS1Preimage(e); err != tc.want {
			t.Errorf("DecodeMS1Preimage(%s) err = %v, want %v", tc.name, err, tc.want)
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

- [ ] **Step 4: GREEN + mutations.** Run: `go test -count=1 ./codex32/` — Expected: PASS. Then, each reverted:

| Mutation | Measured failure |
| --- | --- |
| `copy(preimage[:], d[:32])` in place of `d[1:]` — the kind byte kept, the last byte dropped | `preimage = 03ababab…abab, want the corpus's preimage_hex ababab…abab`. **Round 0's assertion could not fail on this**: its only check was `if x[0] == 0 && x[31] == 0`, and under the mutation `x[0] = 0x03` and `x[31] = 0xab`, so the `&&` was false and the mutant reported PASS — executed and confirmed (r0 adversarial C-2). |
| drop the `!f.Unshared` clause | `DecodeMS1Preimage(a 2-of-N share beginning 0x03) err = <nil>, want codex32: not an m-format secret payload` |

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
- Create: `gui/composer_hashlock.go` — the `hashlockOutcome` type and its two constants, plus the `hashlockPhraseRoute` stub Task 4 replaces (Step 5; r0 coverage I-1 and its M-1 companion, which named this file as missing from this header)

**Interfaces:**
- Consumes: `composerPickScreen(ctx, th, title, lead string, rows []string) (int, bool)`, `composerPayloadDigests(*syswSession) [][32]byte`, `composerHashRow`, `composerHexEntry`, `composerCopyHashRule`, `showError`; Task 4's `hashlockPhraseRoute(ctx, th, st, idx int, payload [][32]byte) hashlockOutcome`.
- Produces: the `composerHashRowSet` struct and its constructor `composerHashRows(s *syswSession) composerHashRowSet` — Go forbids a type and a func sharing one name in a package, and the tests call the CONSTRUCTOR `composerHashRows`, so the TYPE takes the `…RowSet` name; the phrase row label constant `composerHashRowPhrase = "Type a hashlock phrase"`.

- [ ] **Step 1: The tests (RED).** Append to `gui/composer_hash_test.go`:

```go file=gui/composer_hash_test.go mode=fragment
// H2: `Which hash?`'s rows are built ONCE and each named row's index is recorded
// (spec §5; r2 review C-4). This test covers the ROW SET only -- the labels, the
// three recorded indices and the lead -- with 0, 1 and 2 payload digests.
//
// It does NOT drive composerHashEdit and therefore CANNOT see the dispatch
// switch at all; the round-0 comment here claimed it caught an index-arithmetic
// reversion, and that claim was false (r0 fidelity I-1). The dispatch is covered
// behaviourally by TestComposerHashEditDispatchesByRowLabel in
// composer_hashlock_test.go, which taps each row through composerHashEdit with
// two payload digests loaded -- the shape that distinguishes a surgical
// reversion (phrase row kept at the right index, hex+none merged into one
// clearing arm) from correct code. MUTATION for THIS test: swap the order of
// the phrase and hex appends in composerHashRows -> the labels-misplaced
// assertion fails.
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

// composerHashByPhraseSync drops st.hashByPhrase once NO path carries a hash at
// all -- the one event after which no phrase-set hash can still be in the
// composition (r0 adversarial I-2 = fidelity M-2 = journey M-1: the flag was set
// and never cleared anywhere).
//
// It is deliberately NOT cleared when THIS path's hash is replaced by a payload
// row or a hex digest: another path may still be phrase-set, and clearing on
// that narrower event would drop §8h's phrase form while a phrase-set hash is
// still live -- the C16 shape (a composition-wide fact edited as though it were
// per-path). The residual staleness runs the SAFE way: an over-sticky flag makes
// composerCopyHashEveryPathPhrase name "the phrase and its method, OR the
// preimage plate", so the operator is told to back up one artifact too many,
// never one too few. Per-path provenance is filed as a follow-up (owning phase
// H3) rather than bolted on here, because it needs the same splicing discipline
// composerAddPath and "Remove path" already apply to Paths.
func composerHashByPhraseSync(st *composerState) {
	for _, p := range st.list.Paths {
		if p.Hash != nil {
			return
		}
	}
	st.hashByPhrase = false
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
			composerHashByPhraseSync(st)
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

Note the behaviour change for `Type 64 hex`'s Back: today `composerHexEntry`'s `false` propagates out of `composerHashEdit` and, at creation, deletes the path (`composer_shape.go:269`); under §4.6 it `continue`s back to `Which hash?`. The test in Step 1 does not cover it — it never calls `composerHashEdit` at all — and round 0's claim that "Task 4's harness tests do" was **false**: no test anywhere selected `rows.hexRow` or pressed Back at the pad (r0 adversarial I-3 = fidelity I-4 = journey I-4). Task 4 Step 2 now carries `TestHashlockHexRowBackKeepsThePath`, which drives it through `composerAddPath` (the creation entry point, where `false` deletes the path) and asserts the path survives with `Hash == nil`.

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

(Task 4 adds eleven more bodies and bumps the same literal to `53`; the file's FINAL text — the comment block and the `if declared != 53` it guards — is the block in Task 4 Step 1, which is what the gated tree holds. Both bumps are recorded in that one comment. It was nine and `51` before the R0 round 0 fold added `composerCopyHashlockOtherPath` and `composerCopyHashlockReconcile`.)

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

- [ ] **Step 5: GREEN.** Task 4 supplies `hashlockPhraseRoute`; until then `gui/composer_hashlock.go` holds the stub below, replaced wholesale in Task 4 Step 3.

It is not "a one-line stub" (r0 coverage I-1): Step 2's switch has a `case hashlockAssigned:` arm, so the package does not compile until the TYPE and BOTH constants exist. Create the file with exactly this and nothing else:

```go
// gui/composer_hashlock.go, Task 3's transient content
package gui

// Task 3 stub. Task 4 Step 3 REPLACES this whole file; the type and BOTH
// constants are declared here because composerHashEdit's switch names
// hashlockAssigned, so a stub that declared only the function would not compile.
type hashlockOutcome int

const (
	hashlockAssigned hashlockOutcome = iota
	hashlockBackToWhichHash
)

func hashlockPhraseRoute(ctx *Context, th *Colors, st *composerState, idx int, payload [][32]byte) hashlockOutcome {
	return hashlockBackToWhichHash
}
```

(The block carries NO `file=` header, deliberately: it is a TRANSIENT file that Task 4 Step 3 overwrites, so the gated tree holds Task 4's version and `scripts/h2-plan-blocks-vs-tree.sh` has nothing to compare it against — the checker lists it among the blocks it does not cover. Its gate is the compile below, which was run for this fold: the stub was dropped into a copy of the gated tree with Task 4's copy bodies and `composer_hashlock_test.go` removed, and `go build ./gui/` exited 0. `ctx`, `th`, `st`, `idx` and `payload` are unused in the stub, which Go permits for parameters. `composerHashByPhraseSync` is NOT here — it lives in `gui/composer_hash.go` beside its only caller, for exactly this reason.)

Run: `go test -count=1 -run 'TestWhichHashRowsAreLabelKeyed|TestComposerHash|TestComposerCopy|TestComposerLockAndHashEdits' ./gui/` — Expected: PASS.

- [ ] **Step 6: Commit.**

```bash
git add gui/composer_hash.go gui/composer_hash_test.go gui/composer_copy.go gui/composer_copy_test.go gui/composer_gates_test.go gui/composer_hashlock.go
git commit -s -m "composer: Which hash? rows are label-keyed; the phrase row; the no-payload lead names both routes (hashlock H2)"
```

---

### Task 4: The phrase route — screens, derivation, confirm, copy

**Files:**
- Create: `gui/composer_hashlock.go` (replacing Task 3's stub), `gui/composer_hashlock_test.go`
- Modify: `gui/composer_copy.go` (append; `seedhammer.com/hashlock` joins its imports), `gui/composer_copy_test.go` (eleven rows + the `declared` literal 42 → 53, and the `hashlock` import — **build gate fix 2**), `gui/modal_fits_test.go` (three rows on the `showError` table + a new `TestConfirmScreensThisBlockTouchesAreDrawnInFull` carrying the three `ConfirmWarningScreen` bodies, + the `hashlock` import)
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

Append the bodies (this is the text the build gate proved fits — see the §4.5 note after the gate rows below; the reuse block is the brainstorm's two sentences, and the reconciliation line lives on its own screen in `composerCopyHashlockReconcile`):

```go file=gui/composer_copy.go mode=fragment
// The first sentence answers the §8i rule modal the operator has just dismissed
// ("A passphrase must be hashed to 32 bytes first, then hashed again") -- that
// modal fires on the phrase row too, immediately in front of the one route that
// does the hashing itself, and read cold it says this route cannot work
// (r0 journey I-5). Stating it here costs no new gate row and no new screen.
func composerCopyHashlockPhraseLead() string {
	return "This screen does that hashing for you. Use a phrase you have never " +
		"used anywhere else."
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
// holds no hash: record; otherwise the matches/no-match line. otherPath is ""
// unless another path of this policy already carries a different hash.
func composerCopyHashlockConfirm(first8last8, method string, chars int, relation, otherPath string) string {
	b := "hash  " + first8last8 + "\n" +
		fmt.Sprintf("method: %s   chars: %d", method, chars) + "\n"
	if relation != "" {
		b += relation + "\n"
	}
	if otherPath != "" {
		b += otherPath + "\n"
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

// §4.5's reconciliation line, on its own screen after HOLD.
//
// §4.5's drop-order step 2 says to move this line into the phrase-route §8h at
// Done, and the build gate did -- but §8h is guarded by composerEveryPathHashed
// (composer_state.go:239 at the fork baseline c4a64fc), so on the ordinary
// wallet with one keyed path and one
// hashlocked path it was drawn NOWHERE (r0 adversarial I-1 = fidelity I-2 =
// journey I-3, all three tracing the same loss). Its own screen after HOLD is
// reachable for every policy that has a phrase-set hash, and keeps the confirm
// modal's measured headroom (186) intact. §4.7's copy is unchanged below, as the
// spec states it.
func composerCopyHashlockReconcile() string {
	return "Before you fund this wallet, run ms hashlock with this phrase and " +
		"method on the host and check the digest matches."
}

// composerCopyHashlockOtherPath is the confirm modal's second relation line
// (r0 journey I-1): another path of this policy already carries a DIFFERENT
// hash, so spending will need two phrases and two backups.
func composerCopyHashlockOtherPath() string {
	return "another path has a different hash: two phrases to back up"
}

// §8h, the phrase-route form (SPEC_hashlock_H2_device §4.7), verbatim as the
// spec states it. The reconciliation line lives in composerCopyHashlockReconcile
// instead; see there.
func composerCopyHashEveryPathPhrase() string {
	return "HASH ON EVERY PATH\n" +
		"Every way to spend this wallet needs a hashlock preimage. It is not on " +
		"this device and not on these plates. Back up the phrase and its method, " +
		"or the preimage plate, separately."
}
```

Now BOTH copy gates. `TestComposerCopyTableCoversEveryBody` AST-scans `composer_copy.go`
and fails BY NAME on any `composerCopy*` function without a `composerCopyTable` row, so
every one of this task's ELEVEN new bodies needs one. Round 0 of this plan named six §8
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
			"This screen does that hashing for you. Use a phrase you have never used anywhere else."},
		{"composerCopyHashlockRefusal", "H2-4.2", composerCopyHashlockRefusal(hashlock.ErrMS1Shaped),
			"That is a preimage plate, not a phrase. On the host, run ms hashlock with it and load the hash: record it prints."},
		{"composerCopyHashlockHardenedWarning", "H2-4.3a", composerCopyHashlockHardenedWarning(),
			"Even a 20-character phrase falls in about 72 days on one GPU, and shorter ones fall sooner. Choose it from a generator. If you have used this phrase anywhere else, press Back and choose another. Continue?"},
		{"composerCopyHashlockSHA256Warning", "H2-4.3b", composerCopyHashlockSHA256Warning(),
			"This is the brainwallet construction: anyone holding the digest tests 10^10 phrases per second. A phrase a person chose is not safe here; use six diceware words. If you have used this phrase anywhere else, press Back and choose another. Continue?"},
		{"composerCopyHashlockDerivingLead", "H2-4.4", composerCopyHashlockDerivingLead(),
			"Deriving. This takes about 10 seconds."},
		{"composerCopyHashlockConfirm", "H2-4.5", composerCopyHashlockConfirm("b867db87..edbc96cb", "hardened", 100,
			composerCopyHashlockRelation(-1), composerCopyHashlockOtherPath()),
			"hash  b867db87..edbc96cb method: hardened   chars: 100 no hash: record in the payload has this digest " +
				"another path has a different hash: two phrases to back up " +
				"Write down this phrase and the method now. They are not on this device and not on your plates. Without both, this path can never be spent. " +
				"One phrase per policy. Never use this phrase as a passphrase or a password anywhere else."},
		{"composerCopyHashlockRelation", "H2-4.5", composerCopyHashlockRelation(0),
			"matches hash 1 in the payload"},
		{"composerCopyHashlockOtherPath", "H2-4.5", composerCopyHashlockOtherPath(),
			"another path has a different hash: two phrases to back up"},
		{"composerCopyHashlockReconcile", "H2-4.5", composerCopyHashlockReconcile(),
			"Before you fund this wallet, run ms hashlock with this phrase and method on the host and check the digest matches."},
		{"composerCopyHashEveryPathPhrase", "H2-4.7", composerCopyHashEveryPathPhrase(),
			"HASH ON EVERY PATH Every way to spend this wallet needs a hashlock preimage. It is not on this device and not on these plates. Back up the phrase and its method, or the preimage plate, separately."},
		{"composerCopyHashEveryPathFor", "H2-4.7", composerCopyHashEveryPathFor(&composerState{hashByPhrase: true}),
			"HASH ON EVERY PATH Every way to spend this wallet needs a hashlock preimage. It is not on this device and not on these plates. Back up the phrase and its method, or the preimage plate, separately."},
```

and the scan's declared-count literal goes 42 → 53, carrying the reason (this is the
file's final text, both bumps recorded):

```go file=gui/composer_copy_test.go mode=fragment
	// 53 SINCE H2 TASK 4 added the phrase route's eleven bodies: the phrase
	// lead, the phrase-rule refusal, both method warnings, the deriving
	// lead, the confirm body, its relation line and its other-path line, the
	// reconciliation screen, and the two §8h forms
	// (SPEC_hashlock_H2_device §4.2-§4.7). The last two arrived in the R0
	// round 0 fold: composerCopyHashlockOtherPath (journey I-1) and
	// composerCopyHashlockReconcile (adversarial I-1 = fidelity I-2 =
	// journey I-3, the line §8h's guard had made unreachable).
	if declared != 53 {
		t.Errorf("composer_copy.go declares %d bodies, the plan and the table know 53 -- "+
			"if that is deliberate, update both", declared)
	}
```

(`composerCopyHashEveryPathFor` is created in Step 3 with the §8h wiring; its row and the
53 land here so the count gate is bumped once. Step 1 is RED by construction, as below.)

Then the fit gate in `gui/modal_fits_test.go`, which also needs the `hashlock` import.
It is TWO tables, not one: `TestModalsThisBlockTouchesAreDrawnInFull` measures `showError`
bodies through `errorScreenBody`, and a new `TestConfirmScreensThisBlockTouchesAreDrawnInFull`
measures the three bodies `composerConfirmScreen` draws — wrapped in `composerConfirmBody`,
as production draws them — through `confirmWarningBody` (r0 fidelity I-3 = journey I-2).

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
			"the hashlock ms1-plate refusal (H2 §2 rule 3)",
			composerCopyHashlockRefusal(hashlock.ErrMS1Shaped),
		},
		{
			"the hashlock reconciliation screen (H2 §4.5)",
			composerCopyHashlockReconcile(),
		},
		{
			"HASH ON EVERY PATH, phrase-route form (H2 §4.7)",
			composerCopyHashEveryPathPhrase(),
		},
	} {
		t.Run(tc.what, func(t *testing.T) {
			assertModalBodyFits(t, tc.what, errorScreenBody, tc.body)
		})
	}
}

// The H2 bodies drawn by composerConfirmScreen -> ConfirmWarningScreen, measured
// on THAT renderer and WRAPPED in composerConfirmBody exactly as production
// draws them.
//
// A separate table rather than a renderer column on the one above, because
// TestModalsThisBlockTouchesAreDrawnInFull's rows are all showError bodies and
// a third positional field would have had to be added to every pre-existing row
// to say so.
//
// r0 fidelity I-3 = journey I-2 (PARTIAL): these three rows were measured
// through errorScreenBody and unwrapped, while being named as modals -- the
// table's claim about its own subject was false. The NUMBERS do not move: the
// refute pass measured the capacity delta between the two renderers at ZERO for
// seven different bodies, because warningBodyClip (gui/gui.go:595-600) depends
// only on dims. Journey's further suggestion -- re-run §4.5's drop order and
// restore the reconciliation line if the unshortened body "now fits" -- is
// declined on that measurement; the line's real loss was §8h's guard, fixed by
// composerCopyHashlockReconcile instead.
func TestConfirmScreensThisBlockTouchesAreDrawnInFull(t *testing.T) {
	for _, tc := range []struct {
		what string
		body string
	}{
		{
			"the hashlock hardened warning (H2 §4.3)",
			composerConfirmBody(composerCopyHashlockHardenedWarning()),
		},
		{
			"the hashlock sha256 warning (H2 §4.3)",
			composerConfirmBody(composerCopyHashlockSHA256Warning()),
		},
		{
			"the hashlock confirm modal, longest variant (H2 §4.5)",
			composerConfirmBody(composerCopyHashlockConfirm("b867db87..edbc96cb", "hardened", 100,
				composerCopyHashlockRelation(-1), composerCopyHashlockOtherPath())),
		},
	} {
		t.Run(tc.what, func(t *testing.T) {
			assertModalBodyFits(t, tc.what, confirmWarningBody, tc.body)
		})
	}
}
```

Run: `go test -count=1 -run 'TestModalsThisBlockTouchesAreDrawnInFull|TestConfirmScreens|TestComposerCopy' ./gui/` — Expected: does not compile until Step 3's functions exist; then GREEN.

**§4.5's drop order was needed, and BOTH of its steps (build gate fixes 3 and 4) —
but step 2's DESTINATION was unreachable, and the R0 round 0 fold moved it.**
`assertModalBodyFits` measures each body per-body with an 80-character margin. Measured
by the build gate:

- Step 0, the unshortened §4.5 body: **484 of 504 characters drawn — CUT** after
  "…check the digest matches.", before "Hold button to confirm.".
- Step 1, shorten the reuse block to the brainstorm's two sentences ("One phrase per
  policy. Never use this phrase as a passphrase or a password anywhere else."): it fits,
  384/384 drawn, **headroom 64 — still BELOW the 80-character margin**, so the test still
  fails ("fits today with only 64 characters to spare… Shorten this body rather than
  lowering the margin.").
- Step 2, take the reconciliation line ("Before you fund this wallet, run ms hashlock…")
  out of the confirm modal. §4.5's own next step names the phrase-route §8h at Done as its
  destination, and the build gate put it there — but **§8h is guarded by
  `composerEveryPathHashed` (`gui/composer_state.go:239` at the fork baseline `c4a64fc`), which is false the moment ONE
  path is keyed**, so on the ordinary wallet (one keyed path, one hashlocked path) the
  line was then drawn NOWHERE AT ALL. Three independent lenses traced that loss
  (r0 adversarial I-1 = fidelity I-2 = journey I-3). The fold keeps step 2's REMOVAL from
  the modal and gives the line its own `showError` immediately after HOLD in
  `hashlockPhraseRoute`, where every phrase-set hash passes — meeting §4.5's own statement
  of what the line is for ("converts a divergence discovered at spend time into a
  five-minute check") for every policy that has one. `composerCopyHashEveryPathPhrase` is
  back to §4.7's text verbatim.

**Measured for the fold, at the gated tree** (`assertModalBodyFits`, margin 80; the three
confirm bodies through `confirmWarningBody`, the rest through `errorScreenBody`):

| body | drawn | headroom |
| --- | --- | --- |
| the §4.5 confirm modal, longest variant (relation line AND other-path line, `chars: 100`) | 337 | **107** |
| the hardened warning, wrapped in `composerConfirmBody` | 189 | 302 |
| the SHA-256 warning, wrapped in `composerConfirmBody` | 226 | 302 |
| the ms1-plate refusal | 91 | 476 |
| the reconciliation screen | 94 | 455 |
| §8h, phrase-route form (§4.7's text) | 160 | 378 |

The blocks above already carry all of it, so an implementer following this plan does not
rediscover it. Do NOT re-lengthen the confirm body: 64 characters of headroom is a failing
gate, not a near miss, and the longest variant now sits at 107 because §4.5 gained a
second relation line (journey I-1). **The spec is not edited by this fold**; §4.5's
drop-order step 2 names a destination this plan departs from, and the exact replacement
sentence is recorded as an H3 item in `## R0 round 0 folded here`.

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
	"testing/synctest"
	"time"

	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/op"
	"seedhammer.com/hashlock"
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

// runComposerHashEdit drives composerHashEdit ALONE on the touch harness, at an
// existing path, so the row switch can be exercised per row without walking the
// whole add-path flow first. ret receives composerHashEdit's return value.
//
// Same platform setup as runComposerAddPath (sh2DisplaySize, the passphrase
// keyboard hook), because the phrase row leads to the same keyboard.
func runComposerHashEdit(t *testing.T, st *composerState, sess *syswSession, idx int, ret *bool) *sessionHarness {
	t.Helper()
	p := newPlatform()
	p.display = sh2DisplaySize
	ctx := NewContext(p)
	ctx.sysw = sess
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
		*ret = composerHashEdit(ctx, &descriptorTheme, st, idx)
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

// composerStateWithPaths is composerStateForTest with n paths already present,
// each key-less and un-hashed -- the shape composerHashEdit edits in place.
func composerStateWithPaths(t *testing.T, n int) *composerState {
	t.Helper()
	st := composerStateForTest(t)
	st.list.Paths = make([]md.SpendPath, n)
	return st
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

// The relation line, parameterised (r0 fidelity I-5). Round 0 had ONE case whose
// matching record sat at index 0, so `match := 0` in place of `match := -1` was
// indistinguishable from correct code: the loop found a real match at 0 either
// way. Three cases close it.
//
// MUTATIONS:
//   - `match := 0` -> the "neither record matches" case reports `matches hash 1
//     in the payload` instead of the no-match line, and fails.
//   - report `match` rather than `match+1` (1-based off-by-one) -> the "second
//     record matches" case fails, because it is the only one whose answer is not
//     also 1 under the mutation.
//   - `if len(payload) > 0` -> `if true` in hashlockRelationLine -> the "no
//     records at all" case fails on the unwanted no-match line.
func TestHashlockConfirmRelationLine(t *testing.T) {
	const otherDigest = "abababababababababababababababababababababababababababababababab"
	for _, tc := range []struct {
		name     string
		records  []string
		want     string
		unwanted string
	}{
		{
			"the SECOND record matches -- pins the 1-based index",
			[]string{"hash:" + otherDigest, "hash:" + hashlockAnchorSHA_H},
			"matches hash 2 in the payload", "matches hash 1",
		},
		{
			"records are loaded and NEITHER matches",
			[]string{"hash:" + otherDigest, "hash:" + strings.Repeat("cd", 32)},
			"no hash: record in the payload has this digest", "matches hash",
		},
	} {
		t.Run(tc.name, func(t *testing.T) {
			st := composerStateForTest(t)
			h := runComposerAddPath(t, st, composerSessionWith(tc.records, nil))
			h.mustReach("What can spend on this path?")
			h.choose(1) // A hash, no keys
			h.mustReach("EXPERIMENTAL")
			h.holdConfirm()
			h.mustReach("Which hash?")
			h.tapRow(len(tc.records), len(tc.records)+3) // the phrase row sits after the payload rows
			h.mustReach("32-byte value")                 // the §8i rule modal (composerCopyHashRule)
			h.tapNav(Button3)
			h.mustReach("Hashlock phrase")
			typeOnPassphraseKeyboard(t, h, hashlockAnchorPhrase)
			h.tapNav(Button3)
			h.mustReach("Which method?")
			h.tapRow(1, 2)
			h.mustReach("brainwallet")
			h.holdConfirm()
			body := h.mustReach(tc.want)
			if uiContains(body, tc.unwanted) {
				t.Errorf("the confirm modal also drew %q: %q", tc.unwanted, body)
			}
		})
	}

	// With NO hash: records loaded, neither line is drawn at all -- the arm the
	// two cases above cannot reach.
	if got := hashlockRelationLine(nil, hashlockMustHex(t, hashlockAnchorSHA_H)); got != "" {
		t.Errorf("no payload records drew the relation line %q", got)
	}
}

// composerHashEdit dispatches BY LABEL, driven through the screen with two
// payload digests loaded -- the shape that can tell a correct switch from a
// surgical reversion to index arithmetic (r0 fidelity I-1, refined by tests I-2).
//
// With 2 digests the rows are payload 0, payload 1, phrase (2), hex (3), none
// (4). MUTATION: replace the switch's phrase/hex/none arms with
// `case sel == len(rows.digests): // phrase` + `default: st.list.Paths[idx].Hash
// = nil` -- the reversion the plan's own C-4 comment describes. The phrase row
// still lands correctly (it IS len(digests)), so every test that runs with 0
// payload digests still passes; the "hex row opens hex entry" subtest below is
// what fails, because the hex row falls into the clearing arm and
// composerHashEdit returns true with Hash nil instead of drawing the pad.
func TestComposerHashEditDispatchesByRowLabel(t *testing.T) {
	const digestA = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
	const digestB = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
	sessionOf := func() *syswSession {
		return composerSessionWith([]string{"hash:" + digestA, "hash:" + digestB}, nil)
	}

	t.Run("payload row 2 assigns payload digest 2", func(t *testing.T) {
		st := composerStateWithPaths(t, 1)
		var ret bool
		h := runComposerHashEdit(t, st, sessionOf(), 0, &ret)
		h.mustReach("Which hash?")
		h.tapRow(1, 5)
		h.mustReach("32-byte value") // the §8i rule modal: a payload row TAKES a hash
		h.tapNav(Button3)
		h.waitDone()
		if !ret {
			t.Fatal("composerHashEdit returned false after a payload row was taken")
		}
		if got := st.list.Paths[0].Hash; got == nil || hashlockHashHex(got) != digestB {
			t.Fatalf("hash = %v, want payload digest 2", got)
		}
	})

	t.Run("hex row opens hex entry and does not clear", func(t *testing.T) {
		st := composerStateWithPaths(t, 1)
		var ret bool
		h := runComposerHashEdit(t, st, sessionOf(), 0, &ret)
		h.mustReach("Which hash?")
		h.tapRow(3, 5)
		h.mustReach("32-byte value")
		h.tapNav(Button3)
		// The hex pad, NOT a cleared lock and a returned composerHashEdit.
		h.mustReach("0 of 64 hex")
		if *h.done {
			t.Fatal("composerHashEdit returned instead of opening hex entry")
		}
		h.tapNav(Button1) // Back at the pad -> `Which hash?`, nothing assigned
		h.mustReach("Which hash?")
		if st.list.Paths[0].Hash != nil {
			t.Fatal("Back at the hex pad assigned a hash")
		}
	})

	t.Run("phrase row opens the phrase screen", func(t *testing.T) {
		st := composerStateWithPaths(t, 1)
		var ret bool
		h := runComposerHashEdit(t, st, sessionOf(), 0, &ret)
		h.mustReach("Which hash?")
		h.tapRow(2, 5)
		h.mustReach("32-byte value")
		h.tapNav(Button3)
		h.mustReach("Hashlock phrase")
	})

	t.Run("none row clears without the rule modal", func(t *testing.T) {
		st := composerStateWithPaths(t, 1)
		var preset [32]byte
		preset[0] = 0x11
		st.list.Paths[0].Hash = &preset
		st.hashByPhrase = true
		var ret bool
		h := runComposerHashEdit(t, st, sessionOf(), 0, &ret)
		h.mustReach("Which hash?")
		h.tapRow(4, 5)
		h.waitDone()
		if !ret {
			t.Fatal("composerHashEdit returned false after `No hash lock`")
		}
		if st.list.Paths[0].Hash != nil {
			t.Fatal("`No hash lock` did not clear the hash")
		}
		// r0 adversarial I-2: the provenance flag is dropped once no path
		// carries a hash at all. MUTATION: delete the composerHashByPhraseSync
		// call in composerHashEdit's noneRow arm -> this fails.
		if st.hashByPhrase {
			t.Fatal("st.hashByPhrase survived the last hash being cleared")
		}
	})
}

// Spec §4.6 through the CREATION entry point for the row this plan CHANGED:
// `Type 64 hex`'s Back used to propagate out of composerHashEdit and delete the
// path (composer_shape.go:269-272 at the fork baseline c4a64fc); under §4.6 it
// returns to `Which hash?` with
// the path intact. Round 0 claimed "Task 4's harness tests do" cover this and
// none did (r0 adversarial I-3 = fidelity I-4 = journey I-4).
//
// MUTATION: `return false` in place of `continue` in composerHashEdit's hex arm
// -> measured: `never reached "Type a hashlock phrase"; last frame
// "0123456789ABCDEF0of64hexHashlock"`. It fails EARLIER than at the path count,
// because composerHashEdit's false unwinds composerAddPath, which deletes the
// path and leaves the screen -- so `Which hash?` never comes back at all. The
// path-count assertion below is what states the device consequence.
func TestHashlockHexRowBackKeepsThePath(t *testing.T) {
	st := composerStateForTest(t)
	h := runComposerAddPath(t, st, composerSessionWith(nil, nil))
	h.mustReach("What can spend on this path?")
	h.choose(1) // A hash, no keys
	h.mustReach("EXPERIMENTAL")
	h.holdConfirm()
	h.mustReach("Type a hashlock phrase")
	h.tapRow(1, 3) // Type 64 hex (no payload digests: phrase 0, hex 1, none 2)
	h.mustReach("32-byte value")
	h.tapNav(Button3)
	h.mustReach("0 of 64 hex")
	h.tapNav(Button1) // Back at the pad
	h.mustReach("Type a hashlock phrase")
	if n := len(st.list.Paths); n != 1 {
		t.Fatalf("Back at the hex pad deleted the path: %d paths", n)
	}
	if st.list.Paths[0].Hash != nil {
		t.Fatal("Back at the hex pad assigned a hash")
	}
}

// hashlockDerivingLead is §4.4's lead, as a pure function (r0 adversarial I-4).
//
// The guard itself is not what round 0 got wrong -- `done > 0 && elapsed > 0`
// around the estimate and `done <= 0 || elapsed <= 0` around the zero state are
// the same predicate. What was wrong is WHERE it was evaluated: only inside
// DeriveHardened's callback, whose first call arrives at done = 501, so the zero
// state could never be chosen. The hoisted zero-state FRAME in hashlockDeriveFlow
// is the fix, and TestHashlockDeriveKeepsAwakeUnderTheScreensaver asserts the
// lead is drawn on frame 0.
//
// MUTATION for THIS test: drop the guard and return the estimate unconditionally
// -> the three zero-state rows below fail with
// `= "About -9223372036 seconds left."` (done = 0 divides into the estimate).
func TestHashlockDerivingLead(t *testing.T) {
	zero := composerCopyHashlockDerivingLead()
	for _, tc := range []struct {
		name        string
		done, total int
		elapsed     time.Duration
		want        string
	}{
		{"the zero-state frame", 0, hashlock.Iterations, 0, zero},
		{"zero done, time already passed", 0, hashlock.Iterations, 2 * time.Second, zero},
		{"no elapsed time yet", 500, hashlock.Iterations, 0, zero},
		{"halfway, five seconds in", 50000, 100000, 5 * time.Second, "About 5 seconds left."},
		{"a tenth in, one second", 10000, 100000, time.Second, "About 9 seconds left."},
	} {
		if got := hashlockDerivingLead(tc.done, tc.total, tc.elapsed); got != tc.want {
			t.Errorf("%s: hashlockDerivingLead(%d, %d, %v) = %q, want %q",
				tc.name, tc.done, tc.total, tc.elapsed, got, tc.want)
		}
	}
}

// The hardened derivation must not be parked by the screensaver, and its
// zero-state lead must actually be drawn (r0 adversarial C-1 and I-4).
//
// This is the fork's own F-93 regression shape (run_flow_test.go:671's
// TestRunKeepAwakeDuringDerivationDoesNotParkUnderTheScreensaver), pointed at
// hashlockDeriveFlow BY NAME: that test drives unlockDerive and cannot see this
// screen, and the touch harness the rest of this file uses is structurally blind
// to the class, because runUITouch sets ctx.FrameCallback directly and never
// runs Run's idle loop at all.
//
// The arithmetic: hashlock.Iterations = 100,000 in Step(500) slices is 200
// progress calls, and at p.tickFloor = 1s that is 200 s of bubble time against
// idleTimeout's 180 s (gui/gui.go:3584) -- the crossing happens inside the
// derivation, with margin. The floor is load-bearing for the same reason
// deadlinePlatform documents: with ctx.WakeupAt(time.Now()) every deadline is
// already expired, so without a floor the bubble clock never advances and the
// mutant would pass too.
//
// MUTATIONS, both measured:
//   - delete `ctx.KeepAwake()` from hashlockDeriveFlow's frame closure -> the
//     screensaver activates at 180 s and its branch `continue`s without
//     returning control, so ctx.Frame never returns and mustFinish reports
//     "Run exceeded 100000 ticks without terminating -- flow is probably parked
//     (screensaver?). 180 frames drawn, last = 89%About21secondsleft.Deriving".
//   - delete `ctx.WakeupAt(time.Now())` and keep KeepAwake -> the saver never
//     fires (KeepAwake refreshes a.idle.start every tick), so the PARK check
//     above stays green; what breaks is the CLOCK. Every AppendEvents then waits
//     out Run's own ctx.WakeupAt(idleWakeup) -- three minutes -- so a 10-second
//     derivation takes ten hours and the countdown freezes between slices. The
//     elapsed-time assertion below is what sees it: 9h57m1s against the 201 s a
//     1 s tick floor costs. Measured, and the reason this test asserts on device
//     time and not only on completion.
//   - delete `frame(0, hashlock.Iterations)` (the zero-state frame) -> the lead
//     assertion below fails; the derivation itself still completes.
func TestHashlockDeriveKeepsAwakeUnderTheScreensaver(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newDeadlinePlatform()
		p.tickFloor = 1 * time.Second
		var got [32]byte
		var ok bool
		flow := func(ctx *Context, version string) {
			got, ok = hashlockDeriveFlow(ctx, &descriptorTheme, []byte(hashlockAnchorPhrase), hashlockHardened)
			ctx.Done = true
		}
		start := time.Now()
		drawn := mustFinish(t, p, flow, nil)
		elapsed := time.Since(start)
		if !ok {
			t.Fatal("hashlockDeriveFlow returned ok=false -- abandoned or never finished")
		}
		// 201 frames at a 1 s tick floor is 201 s of bubble time. A frame that
		// does not ask to be woken waits out Run's idle deadline instead, which
		// is 3 minutes EACH -- two orders of magnitude, so the bound does not
		// need to be tight to be decisive.
		if elapsed > 10*time.Minute {
			t.Errorf("the derivation took %v of device time; at a %v tick floor and %d frames "+
				"it should take about %v. A frame that omits ctx.WakeupAt(time.Now()) waits out "+
				"Run's idle deadline (3 min) instead of the next 500-iteration slice",
				elapsed, p.tickFloor, len(drawn), time.Duration(len(drawn))*p.tickFloor)
		}
		if want := hashlock.PreimageHardened([]byte(hashlockAnchorPhrase)); got != want {
			t.Error("the derived preimage is not PreimageHardened's (bytes deliberately not logged)")
		}
		if len(drawn) < 200 {
			t.Errorf("only %d frames drawn; 100,000 iterations in 500-step slices is 201", len(drawn))
		}
		if !uiContains(drawn[0], "This takes about 10 seconds") {
			t.Errorf("the first frame is %q, not §4.4's zero-state lead", drawn[0])
		}
		if !uiContains(drawn[len(drawn)-1], "seconds left") {
			t.Errorf("the last frame is %q, not the countdown estimate", drawn[len(drawn)-1])
		}
	})
}

// The reconciliation line is reachable for EVERY policy that has a phrase-set
// hash, including the ordinary mixed one (r0 adversarial I-1 = fidelity I-2 =
// journey I-3). §4.5's drop-order step 2 had moved the line into the §8h form at
// Done, which composerEveryPathHashed guards -- false the moment one path is
// keyed, so on this shape the line was drawn nowhere.
//
// The state here IS that shape: path 0 already carries a hash of its own, and
// path 1 (the one being edited) gets the phrase route. composerEveryPathHashed
// is asserted below, so the test fails loudly if a future edit makes the §8h
// guard true here and the case stops being the one it was written for.
//
// MUTATION: delete the showError(..., composerCopyHashlockReconcile()) call from
// hashlockPhraseRoute -> `never reached "run ms hashlock with this phrase"`.
func TestHashlockReconcileScreenIsReachableOnAMixedPolicy(t *testing.T) {
	st := composerStateWithPaths(t, 2)
	var other [32]byte
	other[0] = 0x11
	st.list.Paths[0].Hash = &other
	st.list.Paths[1].Hash = nil
	if composerEveryPathHashed(st.list) {
		t.Fatal("this test needs a policy §8h's guard REJECTS; it no longer is one")
	}
	var ret bool
	h := runComposerHashEdit(t, st, composerSessionWith(nil, nil), 1, &ret)
	h.mustReach("Type a hashlock phrase")
	h.tapRow(0, 3)
	h.mustReach("32-byte value")
	h.tapNav(Button3)
	h.mustReach("Hashlock phrase")
	typeOnPassphraseKeyboard(t, h, hashlockAnchorPhrase)
	h.tapNav(Button3)
	h.mustReach("Which method?")
	h.tapRow(1, 2) // sha256: instant
	h.mustReach("brainwallet")
	h.holdConfirm()
	// The other path's hash differs, so §4.5's second relation line fires too
	// (r0 journey I-1). MUTATION: return "" from hashlockOtherPathLine ->
	// `never reached "two phrases to back up"`.
	h.mustReach("two phrases to back up")
	h.holdConfirm()
	h.mustReach("run ms hashlock with this phrase")
	if got := st.list.Paths[1].Hash; got == nil || hashlockHashHex(got) != hashlockAnchorSHA_H {
		t.Fatalf("path 2 hash = %v, want the anchor's sha256 digest", got)
	}
	// r0 tests I-4: the flag's real assignment, driven through the route rather
	// than built as a struct literal. MUTATION: delete `st.hashByPhrase = true`
	// from hashlockPhraseRoute -> this fails.
	if !st.hashByPhrase {
		t.Fatal("the phrase route did not record that this hash was set by phrase")
	}
}

// The confirm modal's SECOND relation line stays silent when the other path
// carries the SAME digest -- one phrase, not two (r0 journey I-1's other half).
//
// MUTATION: drop the `*p.Hash != h` comparison from hashlockOtherPathLine (warn
// whenever any other path has any hash) -> this fails at the unwanted-text check.
func TestHashlockOtherPathLineIsSilentOnAnEqualHash(t *testing.T) {
	same := hashlockMustHex(t, hashlockAnchorSHA_H)
	st := composerStateWithPaths(t, 2)
	st.list.Paths[0].Hash = &same
	if got := hashlockOtherPathLine(st, 1, same); got != "" {
		t.Errorf("an EQUAL hash on another path drew %q, want silence", got)
	}
	if got := hashlockOtherPathLine(st, 0, same); got != "" {
		t.Errorf("the path being edited must not warn about itself: %q", got)
	}
	var different [32]byte
	different[0] = 0x11
	if got := hashlockOtherPathLine(st, 1, different); got != composerCopyHashlockOtherPath() {
		t.Errorf("a DIFFERENT hash on another path drew %q, want the warning", got)
	}
	if got := hashlockOtherPathLine(composerStateWithPaths(t, 2), 1, different); got != "" {
		t.Errorf("no other path carries a hash at all; drew %q", got)
	}
}

func hashlockMustHex(t *testing.T, s string) [32]byte {
	t.Helper()
	b, err := hex.DecodeString(s)
	if err != nil || len(b) != 32 {
		t.Fatalf("bad 32-byte hex %q: %v", s, err)
	}
	var out [32]byte
	copy(out[:], b)
	return out
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

Run: `go test -count=1 -run TestHashlock ./gui/` — Expected: **RED, but not a compile
error.** Round 0 predicted "does not compile"; it does compile (r0 tests I-1). Every symbol
this file needs — `hashlockOutcome`, `hashlockAssigned`, `hashlockBackToWhichHash` — is
already declared by Task 3 Step 5's stub, and the stub never registers a keyboard, so the
failures are at RUNTIME: the `TestHashlock*` functions that type a phrase die in
`tapPassphraseKey` with `no *PassphraseKeyboard was registered for this harness`, and the
rest never reach their first screen. Either way this is the RED checkpoint; the point of
saying so is that an implementer who sees a compiling package here has NOT gone wrong.

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
			body := composerCopyHashlockConfirm(hashlockFirst8Last8(h), m.String(), len(phrase),
				hashlockRelationLine(payload, h), hashlockOtherPathLine(st, idx, h))
			if composerConfirmScreen(ctx, th, "Hash lock", composerConfirmBody(body)) {
				d := h
				st.list.Paths[idx].Hash = &d
				st.hashByPhrase = true
				// The reconciliation line, on its own screen and reachable for
				// EVERY policy that has a phrase-set hash (r0 adversarial I-1 =
				// fidelity I-2 = journey I-3). Spec §4.5's drop-order step 2
				// moved it into the phrase-route §8h at Done, but §8h is guarded
				// by composerEveryPathHashed (composer_state.go:239 at the fork
				// baseline c4a64fc), which is false the moment ONE path is keyed
				// -- so on the ordinary mixed wallet the line was drawn nowhere
				// at all. §4.5's own statement
				// of what the line is for ("converts a divergence discovered at
				// spend time into a five-minute check") is met here instead, at
				// the one moment every phrase-set hash passes through.
				showError(ctx, th, "Hash lock", composerCopyHashlockReconcile())
				return hashlockAssigned
			}
			// Back on the confirm -> method pick, nothing assigned
		}
	}
}

// hashlockRelationLine is §4.5's relation line: which payload `hash:` record
// this digest equals, or that none does. "" when the payload holds none.
//
// match starts at -1 so the "no record matches" arm is reachable at all.
// MUTATION: `match := 0` -> TestHashlockConfirmRelationLine's no-match case
// reports `matches hash 1 in the payload`.
func hashlockRelationLine(payload [][32]byte, h [32]byte) string {
	if len(payload) == 0 {
		return ""
	}
	match := -1
	for i, d := range payload {
		if d == h {
			match = i
			break
		}
	}
	return composerCopyHashlockRelation(match)
}

// hashlockOtherPathLine warns when ANOTHER path of this same policy already
// carries a DIFFERENT hash (r0 journey I-1): "One phrase per policy" is advice,
// md.ValidatePathList (md/compose.go:299-334) has no clause about two paths'
// Hash values, and nothing else on the route compares them. Two phrases is a
// legal composition; it is a backup burden the operator must choose knowingly.
//
// It reads *p.Hash directly rather than st.hashByPhrase, so it is unaffected by
// that flag's own staleness, and it skips idx because the path being edited may
// already hold the hash it is about to replace.
func hashlockOtherPathLine(st *composerState, idx int, h [32]byte) string {
	for i, p := range st.list.Paths {
		if i == idx || p.Hash == nil {
			continue
		}
		if *p.Hash != h {
			return composerCopyHashlockOtherPath()
		}
	}
	return ""
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

// hashlockDerivingLead is §4.4's lead: the zero state until the first slice has
// actually been timed, then the estimate. A pure function on the unlockKDFLead
// model (gui/unlock_kdf.go), so the zero state can be asserted without a screen.
//
// The guard is `done <= 0`, not `done > 0` -- and that distinction is the whole
// point of hoisting the zero-state frame below (r0 adversarial I-4): every call
// DeriveHardened makes arrives with done >= 501 (seal.NewDeriver sets done = 1
// and the loop calls progress only after a Step(500) returns false), so a lead
// chosen inside the callback alone can NEVER be the zero state, and §4.4's
// "Deriving. This takes about 10 seconds." would be dead copy.
//
// MUTATION: return the estimate unconditionally -> TestHashlockDerivingLead's
// zero-state rows fail, and the drawn-frame assertion in
// TestHashlockDeriveKeepsAwakeUnderTheScreensaver stops finding the lead.
func hashlockDerivingLead(done, total int, elapsed time.Duration) string {
	if done <= 0 || elapsed <= 0 || total <= 0 {
		return composerCopyHashlockDerivingLead()
	}
	left := time.Duration(float64(elapsed) * float64(total-done) / float64(done))
	return fmt.Sprintf("About %d seconds left.", int(left.Seconds()+0.5))
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
	frame := func(done, total int) {
		dims := ctx.Platform.DisplaySize()
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, "Deriving")
		pct := 0
		if total > 0 {
			pct = done * 100 / total
		}
		pctOp, pctSz := widget.Label(&ctx.B, ctx.Styles.progress, th.Text,
			fmt.Sprintf("%d%%", pct))
		leadOp, leadSz := widget.Labelw(&ctx.B, ctx.Styles.lead, dims.X-2*8, th.Text,
			hashlockDerivingLead(done, total, time.Since(start)))
		nav, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconDiscard},
		}...)
		screen := layout.Rectangle{Max: dims}
		_, content := screen.CutTop(leadingSize)
		pctOp = pctOp.Offset(content.N(pctSz).Add(image.Pt(0, 24)))
		leadOp = leadOp.Offset(content.Center(leadSz))
		// BEFORE ctx.Frame, and the order is load-bearing -- the same fix, for
		// the same reason, as unlockDerive's (gui/unlock_kdf.go:334-335, F-93).
		// ctx.Frame IS the yield, and Run reads the deadline for the frame it
		// has just been handed before its own ctx.Reset(), so a WakeupAt placed
		// AFTER Frame governs the NEXT frame and frame 1 inherits Run's own
		// ctx.WakeupAt(idleWakeup) -- three minutes. Without KeepAwake, Run
		// refreshes a.idle.start only on `effectiveInput(evts, &a.pressed) ||
		// (ctx.keepAwake && !armed)` (run_flow.go:349-350) and a derivation
		// produces no events, so once idleTimeout (3 min,
		// gui/gui.go:3584) is crossed the screensaver takes the screen and its
		// branch `continue`s without breaking (run_flow.go:401-406) -- ctx.Frame
		// never returns and the derivation stops until a touch.
		//
		// Hardened is 100,000 iterations at a measured 9,715 it/s = 10.3 s on
		// the SH2, so the crossing needs an operator who walks away mid-screen;
		// the parked KDF then never resumes, and Back is the only way out of a
		// screen that says "About N seconds left". r0 adversarial C-1.
		ctx.KeepAwake()
		ctx.WakeupAt(time.Now())
		ctx.Frame(op.Layer(pctOp, leadOp, nav, titleOp, op.Color(&ctx.B, th.Background)))
	}
	// §4.4's zero-state frame, drawn BEFORE the first Step so the zero-state
	// lead is reachable at all (r0 adversarial I-4). It also registers backBtn
	// with the router one frame earlier, so a Back pressed on the very first
	// frame is seen by the next callback.
	frame(0, hashlock.Iterations)
	x, ok := hashlock.DeriveHardened(phrase, func(done, total int) bool {
		if ctx.Done {
			return false
		}
		if backBtn.Clicked(ctx) {
			abandoned = true
			return false
		}
		frame(done, total)
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

Run: `go vet ./gui/ && go test -count=1 -run 'TestHashlock|TestWhichHash|TestComposerHash|TestComposerCopy|TestModals|TestConfirmScreens' ./gui/`
Expected: PASS. Mutations, each reverted, with the MEASURED failure in place of round 0's
prediction where the two differ. Rows 1-6 are the build gate's; rows 7-17 are the R0
round 0 fold's, each measured at the gated tree when the test it names was written:

| Mutation | Measured failure |
| --- | --- |
| fold `phrase` through `seal.NormalisePassphrase` in `hashlockPhraseFlow` before `ValidatePhrase` | `TestHashlockPhraseRouteDoesNotNormalise`: `"Correct Horse Battery Staple": path hash = …, want 95d4447…` |
| the confirm's Back returns `hashlockBackToWhichHash` | `TestHashlockBackContractKeepsThePath`: `never reached "Which method?"` |
| `composerHashEdit` returns `false` from the phrase route's Back | `TestHashlockBackContractKeepsThePath` — but it fails EARLIER than round 0 claimed: at `never reached "Type a hashlock phrase"` (the very next inner Back), **not** at the path-count assertion. The mutation is still caught by this test; only the plan's description of *where* was imprecise (gate, Task 4 mutation table). |
| remove the relation line (pass `""` for it) | `TestHashlockConfirmRelationLine`, BOTH cases: `never reached "matches hash 2 in the payload"` and `never reached "no hash: record in the payload has this digest"` |
| delete the release event from `holdConfirm` (fix 11's mechanism) | every test with two or more holds hangs at its second one — see Step 2 |
| drop `!f.Unshared` in Task 2 | its own test fails, as in Task 2 |
| delete `ctx.KeepAwake()` from `hashlockDeriveFlow`'s frame closure | `TestHashlockDeriveKeepsAwakeUnderTheScreensaver`: `Run exceeded 100000 ticks without terminating -- flow is probably parked (screensaver?). 180 frames drawn, last = "89%About21secondsleft.Deriving"`. **The r0 Critical (adversarial C-1)**, now executable. |
| delete `ctx.WakeupAt(time.Now())` and KEEP `KeepAwake` | the same test, on its clock bound: `the derivation took 9h57m1s of device time; at a 1s tick floor and 200 frames it should take about 3m20s`. The park check alone does NOT see this one — KeepAwake keeps the saver off — which is why that test asserts on device time as well as on completion. |
| delete the hoisted `frame(0, hashlock.Iterations)` zero-state frame | the same test: `only 199 frames drawn; 100,000 iterations in 500-step slices is 201` (and the first-frame lead assertion) |
| `hashlockDerivingLead` returns the estimate unconditionally | `TestHashlockDerivingLead`: `hashlockDerivingLead(0, 100000, 0s) = "About -9223372036 seconds left."` |
| `return false` in place of `continue` in `composerHashEdit`'s hex arm | `TestHashlockHexRowBackKeepsThePath`: `never reached "Type a hashlock phrase"; last frame "0123456789ABCDEF0of64hexHashlock"` — it unwinds `composerAddPath`, which deletes the path |
| the surgical index-arithmetic reversion (`case sel == len(rows.digests)` for the phrase row, `default` clears) | `TestComposerHashEditDispatchesByRowLabel/hex_row_opens_hex_entry_and_does_not_clear`: `never reached "0 of 64 hex"`. **`TestWhichHashRowsAreLabelKeyed` and `TestHashlockPhraseRouteSetsTheCorpusDigest` both stay GREEN under it** — measured — which is exactly r0 fidelity I-1's point: with 0 payload digests the phrase row's index is unchanged, so only a test with digests loaded, driving `composerHashEdit`, can see it. |
| swap the phrase and hex appends in `composerHashRows` | `TestWhichHashRowsAreLabelKeyed`: `n=0: indices 1/0/2` |
| delete the `composerHashByPhraseSync` call in the `noneRow` arm | `TestComposerHashEditDispatchesByRowLabel/none_row_clears…`: `st.hashByPhrase survived the last hash being cleared` |
| delete `st.hashByPhrase = true` in `hashlockPhraseRoute` | `TestHashlockReconcileScreenIsReachableOnAMixedPolicy`: `the phrase route did not record that this hash was set by phrase` |
| delete the post-HOLD `showError(…, composerCopyHashlockReconcile())` | the same test: `never reached "run ms hashlock with this phrase"` |
| `match := 0` in `hashlockRelationLine`; and `%d` on `i` rather than `i+1` in `composerCopyHashlockRelation` | `TestHashlockConfirmRelationLine`: the no-match case reports `matches hash 1 in the payload`; the second-record case reports `matches hash 1` where `matches hash 2` is wanted. Round 0's single case could not distinguish either (r0 fidelity I-5). |
| `hashlockOtherPathLine` returns `""`; or drops its `*p.Hash != h` comparison | `TestHashlockReconcileScreenIsReachableOnAMixedPolicy`: `never reached "two phrases to back up"`; and `TestHashlockOtherPathLineIsSilentOnAnEqualHash`: `an EQUAL hash on another path drew "another path has a different hash: …", want silence` |

`go vet ./gui/` reports two PRE-EXISTING complaints that are not this plan's:
`gui/freetext_sizeproof_golden_test.go:111` and `gui/transaction_golden_test.go:104`
(`testing.ArtifactDir requires go1.26 or later (file is go1.25)`). Anything else is new.

- [ ] **Step 5: The whole gui package.**

Run: `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`
Expected: green, partition exhaustive. Measured at the wired tree after the R0 round 0
fold: **1220 top-level tests, `partition verified exhaustive: 1220 == 1220`, all 24 shards
ok, 29 s wall.** (The build gate measured 1213 before the fold.) That is the `c4a64fc`
suite plus the 14 new top-level tests this plan adds to `gui`, and
`TestDecodeMS1PreimageIsShapeExact` is a `codex32` test outside this count:
`TestWhichHashRowsAreLabelKeyed`, the eight `TestHashlock*` harness tests
(`PhraseRouteSetsTheCorpusDigest`, `PhraseRouteDoesNotNormalise`, `BackContractKeepsThePath`,
`DeclineThenHardenedTypesOnce`, `PhraseRefusalsOnScreen`, `MethodModalsFireOnCondition`,
`ConfirmRelationLine`, `HexRowBackKeepsThePath`), `TestHashlockDerivingLead`,
`TestHashlockDeriveKeepsAwakeUnderTheScreensaver`,
`TestHashlockReconcileScreenIsReachableOnAMixedPolicy`,
`TestHashlockOtherPathLineIsSilentOnAnEqualHash`,
`TestComposerHashEditDispatchesByRowLabel` and
`TestConfirmScreensThisBlockTouchesAreDrawnInFull`. `hashlock`'s own 9 tests are a separate
package and are not in the 1220. The FIRST run of this shard set is what caught fix 12
(shard 11 failed); a narrow `-run` selection cannot.

- [ ] **Step 6: Commit.**

```bash
git add gui/composer_hashlock.go gui/composer_hashlock_test.go gui/composer_copy.go gui/composer_copy_test.go gui/modal_fits_test.go gui/composer_state.go gui/composer_shape.go gui/composer_hash.go
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

**This number PREDATES the R0 round 0 fold**, which added 183 lines of production Go across
`gui/composer_hashlock.go` (+119), `gui/composer_copy.go` (+39) and `gui/composer_hash.go`
(+25) — 78 of them non-comment (measured, not estimated): the wakeup calls, the hoisted
zero-state frame, `hashlockDerivingLead`, `hashlockRelationLine`, `hashlockOtherPathLine`,
`composerHashByPhraseSync` and two copy bodies. Re-measure before the merge and record the new delta; the acceptance is the delta
against the named baseline, not this literal.

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

- [ ] engrave `design/FOLLOWUPS.md`: the composer spec's §6c line 386 and §14 sentence ("never derives … a preimage this cycle") → owning phase **H3**, with the replacement wording; **the two R0 round 0 spec-departure items below, each with its exact replacement sentence** (§4.5's drop-order destination, and §4.5's line list gaining the other-path line) → owning phase **H3**; **per-path hash provenance in place of `composerState.hashByPhrase`** → owning phase **H3** (r0 adversarial I-2 / tests I-4; declined for this stage with its reason in `## R0 round 0 folded here`, and it needs the splicing discipline `composerAddPath` and "Remove path" already apply to `Paths`); the M-6 seam-corpus prose correction (from H1b) is done HERE if H2 re-vendors the seam corpus — it does not (H2 vendors the hashlock corpus, a different file), so it stays filed under H3 unless the implementer touches `codex32_seam_vectors.json`; the fork's own CHANGELOG does not exist — the merge commit message is the record.
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
   the `hashlock` import both test files need, and the literal 42 → 51. (**Now eleven rows
   and 42 → 53**: the R0 round 0 fold added `composerCopyHashlockOtherPath` and
   `composerCopyHashlockReconcile`, and the same gate caught both by name.)
3. **The §4.5 confirm body did not fit.** Unshortened it drew 484 of 504 characters and was
   CUT. Spec §4.5's drop-order step 1 (reuse block → the brainstorm's two sentences) left
   **64 characters of headroom against a required margin of 80** — still failing. → the
   shortened text is in Task 4 Step 1's block, with the measurement beside it.
4. **The reconciliation line moved out of the confirm modal.** §4.5's drop-order step 2 was
   needed too: "Before you fund this wallet, run ms hashlock…" left the modal. Measured
   after both steps: confirm **290 drawn, headroom 186**; §8h form **254 drawn, headroom
   262**. → Task 4 Step 1, and the §4.5 note under it.
   **SUPERSEDED IN PART by the R0 round 0 fold:** the gate put the line in
   `composerCopyHashEveryPathPhrase` (Done's §8h form), as §4.5's step 2 names — and §8h's
   `composerEveryPathHashed` guard makes that destination unreachable for any policy with
   one un-hashed path, so the line was drawn nowhere on the ordinary mixed wallet
   (r0 adversarial I-1 = fidelity I-2 = journey I-3). It now has its own `showError` after
   HOLD; the §8h form is back to §4.7's text. The removal from the modal STANDS; only the
   destination changed. New measurements are in Task 4 Step 1's table.
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
green; `gui` 1213 top-level tests, partition verified exhaustive, all 24 shards ok
(**1220 after the R0 round 0 fold** — see `## R0 round 0 folded here`);
firmware **1,595,236 B flash / 62,856 B RAM** (+12,104 / +56 against `c4a64fc`'s
1,583,132 / 62,800). NOT run by the gate, and still owed: the emulator walk
(`cmd/emu/walk_hashlock_phrase.js`, Task 5 Step 1) and the flash.

**The block check.** `scripts/h2-plan-blocks-vs-tree.sh` re-derives the claim this section
makes. Output, re-run after the R0 round 0 fold (line numbers and line counts have moved
since the gate's own run; the verdict has not):

    plan: /scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_hashlock_H2_device.md
    tree: /scratch/code/shibboleth/.tmp/h2-gate

    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:127  whole          hashlock/testdata/hashlock-v0.8.provenance.json  (18 lines, identical)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:150  whole          hashlock/hashlock_test.go                     (336 lines, identical)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:496  whole          hashlock/hashlock.go                          (143 lines, identical)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:684  fragment       codex32/mspayload_test.go                     (7 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:696  fragment       codex32/mspayload_test.go                     (132 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:835  fragment       codex32/mspayload.go                          (25 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:895  fragment       gui/composer_hash_test.go                     (44 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:944  fragment       gui/composer_hash_test.go                     (6 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:957  fragment       gui/composer_hash.go                          (2 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:964  fragment       gui/composer_hash.go                          (105 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1081  fragment       gui/composer_copy.go                          (6 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1092  fragment       gui/composer_copy_test.go                     (2 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1099  fragment       gui/composer_copy_test.go                     (2 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1108  fragment       gui/composer_gates_test.go                    (8 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1170  fragment       gui/composer_copy.go                          (5 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1180  fragment       gui/composer_copy.go                          (102 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1295  fragment       gui/composer_copy_test.go                     (9 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1307  fragment       gui/composer_copy_test.go                     (26 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1339  fragment       gui/composer_copy_test.go                     (12 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1363  fragment       gui/modal_fits_test.go                        (8 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1374  fragment       gui/modal_fits_test.go                        (61 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:1486  whole          gui/composer_hashlock_test.go                 (941 lines, identical)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:2494  whole          gui/composer_hashlock.go                      (305 lines, identical)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:2809  fragment       gui/composer_state.go                         (4 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:2818  fragment       gui/composer_shape.go                         (1 lines, verbatim substring)
    PASS IMPLEMENTATION_PLAN_hashlock_H2_device.md:2824  fragment       gui/composer_copy.go                          (6 lines, verbatim substring)

    26 blocks checked, 0 FAIL

    NOT COVERED by this script:
      * 8 fenced blocks carry no file= header (bash recipes, illustrative
        snippets); nothing here runs or checks them:
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:119  ```bash
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:668  ```bash
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:872  ```bash
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:1125  ```go
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:1150  ```bash
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:2887  ```bash
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:2905  ``` (no info string)
          IMPLEMENTATION_PLAN_hashlock_H2_device.md:2930  ```bash
      * every PROSE claim: expected test names, mutation outcomes, headroom and
        firmware numbers, spec references, file:line citations.
      * whether the tree is GREEN -- this compares TEXT only; `go test` and the
        gate report are what say the text works.
      * files the plan modifies without carrying a block for them.

---

## R0 round 0 folded here

Five lens reports and a refute pass are persisted verbatim in `design/agent-reports/`:
`hashlock-H2-plan-R0-r0-{fidelity,tests,journey,adversarial,coverage,refute}.md`. The
refute pass resolved 25 Critical/Important findings to **16 distinct defects** (15
CONFIRMED, 1 PARTIAL, 0 refuted outright) and that deduplicated list is what this section
answers, one line each. Every code change below was made in the plan AND in the gated tree
at `/scratch/code/shibboleth/.tmp/h2-gate`, and `scripts/h2-plan-blocks-vs-tree.sh` proves
the two are the same bytes.

**The sixteen.**

1. **Hardened derivation stalls under the screensaver** (adversarial C-1, Critical) →
   `hashlockDeriveFlow` now calls `ctx.KeepAwake(); ctx.WakeupAt(time.Now())` immediately
   before `ctx.Frame`, mirroring `unlockDerive` (`gui/unlock_kdf.go:334-335`, F-93), with
   the mechanism in the frame closure's own comment. Gated by
   `TestHashlockDeriveKeepsAwakeUnderTheScreensaver`, which drives `hashlockDeriveFlow` BY
   NAME on `newDeadlinePlatform()` under `synctest` at a 1 s tick floor — 201 frames of
   bubble time against `idleTimeout`'s 180 s. It asserts completion AND elapsed device
   time, because the two halves of the fix fail differently: without `KeepAwake` the run
   parks (`Run exceeded 100000 ticks … 180 frames drawn, last = "89%About21secondsleft"`),
   and without `WakeupAt` it completes but takes `9h57m1s`. The touch harness the rest of
   the file uses cannot see either — `runUITouch` sets `ctx.FrameCallback` directly and
   never runs Run's idle loop — which is why this test uses the Run-level harness instead.
   **The Critical is closed by an executed test, not by the emulator walk.**
2. **Decoder off-by-one and a vacuous digest test** (adversarial C-2 = fidelity I-6 =
   tests C-1) → `TestKindRowPreimageDigest` now compares `Digest(&x)` against the corpus's
   own `digest` constant (a `Digest` field joined the `Kind` struct);
   `TestDecodeMS1PreimageIsShapeExact` reads the vendored corpus instead of a transcribed
   literal, compares all 32 bytes to `preimage_hex`, adds §7.4's acceptance-record plate →
   the anchor row's `hardened_x`, and drives §7.1's "entr32 pair" clause in both
   directions (`DecodeMS1Preimage` refuses the sibling on its prefix byte;
   `DecodeMS1` decodes it as the same 32 bytes). Both false PASSes were reproduced first:
   the double-hash `Digest` and the `d[:32]` decoder each passed the round-0 assertions.
3. **Reconciliation line unreachable for a mixed policy** (adversarial I-1 = fidelity I-2
   = journey I-3) → new `composerCopyHashlockReconcile()` shown by its own `showError`
   right after HOLD in `hashlockPhraseRoute`; `composerCopyHashEveryPathPhrase` is back to
   §4.7's text verbatim. Gated by `TestHashlockReconcileScreenIsReachableOnAMixedPolicy`,
   which asserts `composerEveryPathHashed(st.list) == false` for its own state before
   walking the route, so the test fails loudly if it ever stops being the case §8h's guard
   rejects. Re-measured: confirm modal 337/107, reconciliation screen 94/455, §8h form
   160/378. See the H3 record item below — the spec's §4.5 step 2 is not edited here.
4. **`hashByPhrase` never cleared, and its assignment never verified** (adversarial I-2
   Important; fidelity M-2 / journey M-1 Minor; + tests I-4) → `composerHashByPhraseSync`
   (`gui/composer_hash.go`, beside its only caller) drops the flag from the `noneRow` arm
   once NO path carries a hash. It is deliberately NOT cleared on the narrower events —
   see the declined half below. The assignment is now driven through the real route and
   asserted (`TestHashlockReconcileScreenIsReachableOnAMixedPolicy`), and the clear is
   asserted in `TestComposerHashEditDispatchesByRowLabel/none_row_clears…`.
5. **`Type 64 hex` Back untested, false coverage claim** (adversarial I-3 = fidelity I-4 =
   journey I-4) → `TestHashlockHexRowBackKeepsThePath` drives it through `composerAddPath`
   (creation, where `false` deletes the path) and asserts the path survives with
   `Hash == nil`; Task 3's false sentence ("Task 4's harness tests do") is corrected in
   place and now names the test.
6. **`Deriving` zero-state lead unreachable** (adversarial I-4 Important; fidelity M-1 /
   journey M-2 Minor) → the lead is a pure function, `hashlockDerivingLead(done, total,
   elapsed)`, and `hashlockDeriveFlow` draws a zero-state frame BEFORE the first
   `Step(500)`. Gated by `TestHashlockDerivingLead` (five rows) and by the first-frame
   assertion in the wakeup test. Resolved as **Important**: §4.4 states the zero-state lead
   normatively, so an unreachable one is an unmet spec guarantee, not a cosmetic Minor.
7. **C-4 regression protection real but mis-attributed** (fidelity I-1, refined by tests
   I-2) → both halves. (a) `TestWhichHashRowsAreLabelKeyed`'s comment no longer claims to
   catch a dispatch mutation it structurally cannot see, and names the test that does;
   (b) `TestComposerHashEditDispatchesByRowLabel` drives `composerHashEdit` per row with
   two payload digests. The surgical reversion was applied and measured: that new test
   fails while `TestWhichHashRowsAreLabelKeyed` and `TestHashlockPhraseRouteSetsTheCorpusDigest`
   both stay green — fidelity I-1's claim reproduced exactly.
8. **Fit-gate renderer mismatch** (fidelity I-3 = journey I-2, PARTIAL) → the three
   `ConfirmWarningScreen` bodies move to a new
   `TestConfirmScreensThisBlockTouchesAreDrawnInFull`, measured through `confirmWarningBody`
   and wrapped in `composerConfirmBody` as production draws them. Journey's further step
   ("re-run the drop order; the line goes back if it now fits") is **declined** — see below.
9. **Relation line's no-match branch untested** (fidelity I-5) → `TestHashlockConfirmRelationLine`
   is parameterised over three cases: the SECOND record matching (pins the 1-based index),
   neither matching (reaches the no-match arm), and no records at all (asserted directly on
   `hashlockRelationLine`, extracted as a pure function). `match := 0` and an `i`-for-`i+1`
   off-by-one each fail a different case.
10. **Two paths, two phrases, no cross-check** (journey I-1) → `hashlockOtherPathLine`
    compares the new digest against every OTHER path's `*p.Hash` and adds §4.5's second
    relation line, `composerCopyHashlockOtherPath()`. It reads the live hashes, not
    `hashByPhrase`, so it is unaffected by that flag's staleness.
    `TestHashlockOtherPathLineIsSilentOnAnEqualHash` covers equal, different, self and none.
11. **§8i rule modal confusing ahead of the phrase route** (journey I-5) →
    `composerCopyHashlockPhraseLead()` now opens "This screen does that hashing for you.",
    answering the modal the operator has just dismissed. Copy only; no new gate row, no new
    screen. Its `composerCopyTable` row and the phrase screen's own tests carry the change.
12. **Task 3 Step 5 stub under-specified** (coverage I-1) → Step 5 shows the stub's real
    content (the `hashlockOutcome` type and both constants), and the block was compiled:
    dropped into a copy of the gated tree with Task 4's copy bodies and
    `composer_hashlock_test.go` removed, `go build ./gui/` exits 0. Coverage's own M-1
    companion is folded alongside: Task 3's `Files:` header now names
    `gui/composer_hashlock.go`. `composerHashByPhraseSync` was moved to
    `gui/composer_hash.go` so the Task 3 stub does not have to declare it.
13. **Task 4 Step 2's RED claim does not reproduce** (tests I-1) → the Expected line now
    describes the actual failure (the package COMPILES; the failures are at runtime, in
    `tapPassphraseKey`, with `no *PassphraseKeyboard was registered for this harness`).
14. **`DeriveHardened`'s own abandon contract untested** (tests I-3) →
    `TestDeriveHardenedAbandonsWhenProgressSaysStop` asserts `ok == false`, the zero result,
    and that `progress` was called exactly 3 times rather than 200. Under the mutation that
    ignores the callback's return value: `ok=true`, `called 199 times`.
15. **`minMS1Len` 47/48 boundary untested** (tests I-5) → `TestIsMS1ShapedMinLengthBoundary`,
    with LITERAL 47- and 48-character inputs (not derived from the constant, so the test
    cannot move with the mutation) and their display-grouped forms.
    `minMS1Len = 47` and `= 49` each fail a different pair of rows.
16. **`IsMS1Shaped`'s `TrimSpace`** (tests I-6) → the coverage gap is closed by
    `TestIsMS1ShapedTrimsWhatTheStripLoopCannot`; the reviewer's remedy (delete the call) is
    **declined with a measurement** — see below.

**Declined, with reasons.**

- **tests I-6's "delete the redundant `TrimSpace`" (the reviewer's remedy, not the
  finding).** The premise — "removing the `TrimSpace` call changes the function's behaviour
  for no input" — is **false, measured**. The strip loop skips exactly `' '`, `'\t'`,
  `'\n'`, `'\r'`, `'-'` and `','`; `TrimSpace` removes everything `unicode.IsSpace` reports
  at the ends, a strictly larger set. With the call removed, `IsMS1Shaped` flips from true
  to false for `'\v'`, `'\f'`, U+0085, U+00A0 and U+2003, leading and trailing — ten rows.
  Deleting it would also have DIVERGED FROM THE HOST, whose `looks_like_ms1` is
  `is_ms1_shaped(&raw.trim().to_ascii_lowercase())` (`ms-cli/src/argv_guard.rs:148-149`) and
  whose `str::trim` uses the White_Space property, covering all of them — a Rust-primary
  violation on a refusal rule. The FINDING (untested-as-such) is folded; the remedy is not.
- **journey I-2's "re-run §4.5's drop order from step 0; the reconciliation line goes back
  if the unshortened body now fits".** Refuted by direct measurement in the refute pass:
  the capacity delta between `errorScreenBody` and `confirmWarningBody` is **zero for seven
  different bodies**, because `warningBodyClip` (`gui/gui.go:595-600`) depends only on
  `dims`. Re-measuring reproduces the same numbers, so the drop order would not change. The
  line's real loss was §8h's guard, and item 3 is the fix.
- **Per-path hash provenance (the reviewers' preferred variant of item 4).** Declined for
  this stage and filed as a follow-up with owning phase **H3**. Clearing `hashByPhrase`
  whenever THIS path's hash is replaced would be wrong while another path is still
  phrase-set — the C16 shape — and a per-path array needs the same splicing discipline
  `composerAddPath` and "Remove path" already apply to `Paths`, which is a change to the
  composer's state model rather than to this route. The residual staleness runs the SAFE
  way: `composerCopyHashEveryPathPhrase` names "the phrase and its method, **or** the
  preimage plate", so an over-sticky flag tells the operator to back up one artifact too
  many, never one too few. **Severity resolved as Minor** on that reasoning (fidelity and
  journey rated it Minor for the same reason; adversarial rated it Important). The two
  Important halves — no clear at all, and no test proving the assignment — are folded.

**Severity disputes, resolved.**

- **(a) the decoder / digest-test gap — resolved CRITICAL.** Two of three reports rated it
  Critical and one of those executed the false PASS; project policy keeps "a test that
  reports a false PASS" in the blocking class regardless of operator-visible impact. Both
  halves were reproduced here before being fixed.
- **(b) `hashByPhrase` never cleared — resolved MINOR**, with the reason above (the failure
  is additive copy naming both possible backup artifacts; the dangerous direction, a
  phrase-set hash with the flag false, is unreachable). Folded anyway, because the cheap
  half is correct and free.
- **(c) the `Deriving` zero-state lead — resolved IMPORTANT.** §4.4 states the lead
  normatively, so dead copy there is an unmet spec guarantee, not a cosmetic defect.

**H3 record item, first — the exact spec sentence this plan departs from.**
`SPEC_hashlock_H2_device` is NOT edited by this fold; there are TWO departures, recorded
here and filed as follow-ups with owning phase H3 in Task 6. Its §4.5 drop order currently
reads, in its last clause:

> then move the reconciliation line into the phrase-route §8h at Done (§4.7).

The replacement sentence, for whoever folds the spec at H3:

> then move the reconciliation line out of the confirm modal and onto its own dismissible
> screen shown immediately after HOLD, where it is reachable for every policy that has a
> phrase-set hash — NOT into the phrase-route §8h at Done, whose
> `composerEveryPathHashed` guard is false for any policy with one un-hashed path.

**H3 record item, second — §4.5's line list gains the other-path line.** §4.5 enumerates
the confirm modal's lines "in order" and has no clause for journey I-1's cross-path
warning. The sentence to add to that list, after the relation line:

> `<other-path line, only when another path of this policy already carries a different
> hash: "another path has a different hash: two phrases to back up">`

and to §4.5's bullet list, after the relation-line bullet:

> - The other-path line (journey I-1): when any OTHER path of the same policy already
>   carries a `Hash` that differs from this digest, the modal says so, because
>   `md.ValidatePathList` has no clause about two paths' `Hash` values and "One phrase per
>   policy" is advice — a second phrase is legal, and a second backup burden the operator
>   must choose knowingly. Omitted when no other path carries a hash, or when the hashes
>   are equal.

Measured with it present: the longest §4.5 variant draws 337 with 107 characters of
headroom, above the 80 margin.

**Machine checks run for this fold, at the gated tree.**

    go test -count=1 ./hashlock/... ./codex32/... ./seal/... ./sysw/...
    ok  seedhammer.com/hashlock 0.232s
    ok  seedhammer.com/codex32  0.003s
    ok  seedhammer.com/seal     14.955s
    ok  seedhammer.com/sysw     0.038s

    scripts/gui-shard-test.sh ./gui/ 24
    1220 top-level tests
    partition verified exhaustive: 1220 == 1220
    RESULT: ok -- all 1220 tests ran across 24 shards      (wall 29s)

    scripts/h2-plan-blocks-vs-tree.sh
    26 blocks checked, 0 FAIL

`go vet` over `./hashlock/... ./codex32/... ./seal/... ./sysw/... ./gui/` reports only the
two PRE-EXISTING `testing.ArtifactDir requires go1.26` complaints
(`gui/freetext_sizeproof_golden_test.go:111`, `gui/transaction_golden_test.go:104`), and
`gofmt -l` reports only the three PRE-EXISTING `gui/transaction*` files — both verified
against the fork at `c4a64fc` before this fold, not assumed.

**Still owed, unchanged by this fold:** the emulator walk (`cmd/emu/walk_hashlock_phrase.js`,
Task 5 Step 1 — the plan's one un-gated executable artifact), the firmware size re-measure
after the fold's 183 lines of new production code (78 non-comment), and the flash.

---


## Self-review

1. **Spec coverage.** §2 → Task 1 (`ValidatePhrase`, `IsMS1Shaped` including its `minMS1Len` boundary and its trim, the refusals rows) and Task 4 (through the screen); §3 → Task 1 (`DeriveHardened` on `seal.NewDeriver` with the slice salt, and its abandon contract; the constants; the lockstep mutations); §4.1 → Task 3; §4.2-§4.5 → Task 4 (the flow, the copy, both gates); §4.4's zero-state lead → `hashlockDerivingLead` plus the hoisted zero-state frame; §4.5's reconciliation line → its own post-HOLD screen, NOT §8h (see the H3 record item in `## R0 round 0 folded here`); §4.6 → Task 4's loop, the Back test through `composerAddPath`, and `TestHashlockHexRowBackKeepsThePath` for the `Type 64 hex` arm; §4.7 → `composerCopyHashEveryPathFor`; §5 → Task 3's row set plus `TestComposerHashEditDispatchesByRowLabel` for the dispatch; §6 → Task 2; §7.1 → Task 1's tests; §7.2/§7.3 → Tasks 4/3; §7.4 → Task 2 (the corpus row, the acceptance-record plate, the entr32 pair); §7.5 → Task 5; §7.6 → Task 5 Step 2; §8 → Task 6 (H4); §9 → nothing to build.
2. **Placeholders.** None left in the Go. The Task 4 Step 2 harness helpers round 0 only NAMED are written out in full, as the build gate wrote and ran them, and `scripts/h2-plan-blocks-vs-tree.sh` proves every block is the gated tree's own bytes. Task 3 Step 5's stub is now shown rather than described (r0 coverage I-1) and was compiled. The one thing still written as prose rather than code is Task 5 Step 1's emulator walk: neither the gate nor the R0 round 0 fold ran it (out of scope for both), and its keyboard mapping is probed on the live emulator as `walk_verify.js` did — so it is the plan's one un-gated executable artifact, and the controller runs it before the post-implementation review. **It is no longer load-bearing for adversarial C-1:** that Critical is closed by `TestHashlockDeriveKeepsAwakeUnderTheScreensaver`, which runs in CI.
3. **Type consistency.** `Context.KeepAwake()` and `Context.WakeupAt(time.Time)` (`gui/gui.go:110,119`); `newDeadlinePlatform() *deadlinePlatform` with `tickFloor time.Duration`, and `mustFinish(t, p, flow func(*Context, string), onDraw) []string` (`gui/run_harness_test.go:58,183,220`); `idleTimeout = 3 * time.Minute` (`gui/gui.go:3584`); `confirmWarningBody`/`errorScreenBody` as `modalRenderer = func(t *testing.T, body string) string` (`gui/modal_fits_test.go:108`); `composerEveryPathHashed(list md.PathList) bool` (`gui/composer_state.go:239`); `seal.NewDeriver(passphrase, salt []byte, iterations int) *Deriver`, `Step(n int) bool`, `Done()/Total() int`, `Key() []byte`, `Wipe()` (`seal/pbkdf2.go:85-182`); `composerPickScreen(ctx, th, title, lead string, rows []string) (int, bool)` (`composer_paged.go:259`); `composerConfirmScreen(ctx, th, title, body string) bool` (`composer_shape.go:77`); `composerConfirmBody(body string) string` (`composer_copy.go:32`); `Hash *[32]byte` (`md/compose.go:167`); `composerSessionWith(public, secret []string) *syswSession` (`composer_door_test.go:15`); `ParsePrefix(frag string) (Fields, error)`, `Fields.Unshared` (`codex32/polish.go:82,71`); `NewSeed(hrp string, threshold int, id string, shareIdx rune, data []byte) (String, error)` (`codex32/codex32.go:279`); `composerPickScreenMaxRows = 24` (`composer_paged.go:224`).
