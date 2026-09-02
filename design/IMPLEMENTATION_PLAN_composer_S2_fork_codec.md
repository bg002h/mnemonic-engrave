# Wallet Policy Composer — Stage 2 (fork codec) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the SeedHammer fork's Go packages everything the Stage 3 GUI will call: a `md` tree BUILDER that lowers an ordered spend-path list to a `descriptor` byte-identically to the Rust primary, the `pk_h` emitter arm, a `PolicyShape` that reports every alternative spend path with its lock operands and digests, the stub helpers for re-minted key cards, the three composer record classes in `sysw.Classify` in lockstep with the host, and the taproot `3'` origin — all as package APIs and tests, with the firmware still building and its size delta recorded.

**Architecture:** One new file per package (`md/compose.go`, `md/compose_stubs.go`, `mk/compose_stubs.go`, `sysw/composer_records.go`) plus four small fragments in existing files (`md/script_emit.go` gains one `case`, `md/policy_shape.go` gains a splitter and two `Branch` fields, `sysw/record.go` gains three classes and one dispatch, `gui/multisig_build_slots.go` gains a comment). The builder is a line-for-line port of `md-codec`'s `compose::{lowering,tr}` (SPEC §5, FIXED lowering): validate → number slots by first appearance → build the node tree → resolve origins (§4f lowest-free default account, pairwise-distinguishability) → `descriptor{…}` → `split`. Every behaviour is pinned to the Rust primary by vendored vectors: the 26-vector compose corpus (bytes, chunks, ids, addresses) and the 40-row record-class fixture, each with a provenance pin and a pin test that runs everywhere with no sibling checkout.

**Tech Stack:** Go 1.26 (`/nix/store/i77g9dmcd399rmxk8688qfr4g2wzgk37-go-1.26.7/bin/go`), TinyGo for `cmd/controller` via `nix run .#build-firmware`, `github.com/btcsuite/btcd/{address,btcutil/hdkeychain,chaincfg}/v2` (already in `go.mod`), `encoding/json` in `_test.go` only.

**Spec:** `design/SPEC_wallet_policy_composer.md` (mnemonic-engrave) — §4c lock bands, §4f default origins and the pairwise invariant, §5 lowering, §7c/§7d stubs, §9 items 1, 2, 8, §12 items 1, 6, 7, 8. Staged plan: `design/STAGED_PLAN_wallet_policy_composer.md` §S2. The Rust primary this stage ports: descriptor-mnemonic `crates/md-codec/src/compose/{mod,lowering,tr}.rs` at `66bdf2f4` (S0 shipped) and mnemonic-engrave `crates/me-cli/src/sysw/composer_records.rs` (S1, per `IMPLEMENTATION_PLAN_composer_S1_host_inputs.md` Task 1).

**Baselines (for `scripts/plan-staleness-check.sh`):** seedhammer fork `169073c` (the tree every `path:line` below cites); descriptor-mnemonic `66bdf2f4`; mnemonic-engrave: the S1 merge commit (record it in the fold that follows S1's ship).

## Global Constraints

- **Rust-primary rule (CLAUDE.md):** every normative behaviour here CONVERGES on Rust; nothing is decided in Go. Where a test below disagrees with a vendored vector, the vector wins and the Go code changes. If a vector looks wrong, STOP and record it — the fix lands in Rust first.
- **Byte-identical, not equivalent:** the builder's `encodePayload` bytes and `split` chunk strings equal the vendored `.bytes.hex` and `.phrase.txt` for all 26 corpus vectors, and equal the two `no-corpus` chunk sets in this plan (§12 item 1: "the Go builder reproduces every template, every CHUNK and every address byte for byte").
- **No skip paths (operator directive 2026-08-15):** every test here runs on every CI runner. Nothing reads a sibling checkout; the corpus and the fixture are vendored with provenance pins; a missing file is a FAILURE, never a skip.
- **§4c lock bands (verbatim):** `older` 1..=65535 blocks or `0x400000 + u` for `u` in 1..=65535 units of 512 s; `after` 1..=499,999,999 as a height or 500,000,000..=2,147,483,647 as a Unix time. Enforced by `Lock.Check`/`operand` (§12 item 7: "a unit gate on the emitter's input, not on md's acceptance").
- **§4f default origins:** unseated slot → `m/48'/0'/<account>'/<T>'` with `T` = 2 (wsh, sh), 1 (sh-wsh), 3 (tr), lowest account not already taken by another slot; two slots may share an origin ONLY when both declare distinct fingerprints.
- **Limits (Rust `compose::{MAX_PATHS, MAX_KEYS_PER_PATH, MAX_SLOTS}`):** 8 paths, 9 keys per path, 32 slots.
- **The composer never emits `pk_h` under `tr`** (Rust `path_body` uses `PkK`/`MultiA`/`SortedMultiA` when `tap`), but §9 item 2 asks for the arm "in both script contexts", so the tap arm exists and is tested on a hand-built tree.
- **Secret-handling defects never gate** (operator ruling 2026-08-27); none of this stage's material is secret (`key:`, `hash:`, `now:` are public; the corpus keys are BIP-39's published "abandon" mnemonic).
- **Stage paths explicitly** (no `git add -A`); commits signed + DCO as the fork requires; author Brian Goss; trailer lines below.
- **Do not touch `gui/` beyond Task 7's test and comment.** Screens are Stage 3.
- **Deprecation is comment-only** (operator: "a comment with no enforcement is a feature, not a bug") — nothing here removes or gates Multisig Build.

## What is already machine-verified (reviewer budget goes elsewhere)

- The two `no-corpus` chunk sets in Task 2 were produced 2026-09-02 by the shipped `md compose … --experimental` → `md encode --experimental --force-chunked` at descriptor-mnemonic `66bdf2f4` (commands in Task 2, Step 1).
- The corpus file count (126 = 22 keyed × 5 + 4 unkeyed × 4), the keyed conformance sub-test count moving 14 → 36, and the `family()` mirror below were read from the primary's `tests/compose_support.rs` and `tests/vectors/` at `66bdf2f4`.
- `scripts/plan-build-gate-go.sh` (mnemonic-engrave) extracts every ```go block anchored on `md/compose*.go`, `mk/compose*.go`, `sysw/composer_*.go`, `gui/composer_*.go`, vendors the corpus and the fixture into a scratch copy of the fork, and runs `go vet` + `go test -count=1 ./md/ ./mk/ ./sysw/`. It does NOT assemble the fragments for `script_emit.go`, `policy_shape.go`, `record.go`, `multisig_build_slots.go`; those were hand-wired in the scratch copy before review. The gate's own output is in the plan's fold commit message.
- Baseline before this plan: `go test -count=1 ./md/ ./mk/ ./sysw/` → `ok` ×3 at `169073c`; firmware `tinygo build -size short -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller` → flash 1,503,652 B / RAM 62,592 B.

---

### Task 1: Vendor the S0 compose corpus with a provenance pin

**Files:**
- Create: `md/testdata/vectors/<name>.{bytes.hex,phrase.txt,descriptor.json,template[,conformance.json]}` for the 26 names below (126 files)
- Create: `md/testdata/compose_vectors.provenance.json`
- Create: `scripts/vendor-compose-vectors.sh`
- Create: `md/compose_vectors_pin_test.go`
- Modify: `md/testdata/README.md` (append one section)

**Interfaces:**
- Consumes: descriptor-mnemonic `crates/md-codec/tests/vectors/` at `66bdf2f4`; the fork's existing loaders `loadBytesHex`, `loadDescriptor` (`md/testdata_test.go:71,167`), `loadPhraseChunks` (`md/conformance_keyed_test.go:133`).
- Produces: the 26 vector names as files; `md/testdata/compose_vectors.provenance.json` with `{repo, remote, commit, path, files:[{name, sha256}], vectors, recorded_at}`; the keyed conformance glob (`md/conformance_keyed_test.go:44`, `keyed_*.conformance.json`) picks up the 22 keyed vectors with no code change.

The 26 names (22 keyed, 4 unkeyed), exactly the primary's `MANIFEST` compose entries:

```text
compose_tr_seven_leaves compose_tr_thirty_two_slots compose_wsh_eight_paths compose_wsh_thirty_two_slots
keyed_compose_sh_sole keyed_compose_sh_two_of_four keyed_compose_sh_wsh_one_of_two keyed_compose_sh_wsh_sole
keyed_compose_tr_extracted_first keyed_compose_tr_extracted_later_four_paths keyed_compose_tr_hash_leaf
keyed_compose_tr_key_path_only keyed_compose_tr_nums_three_leaves keyed_compose_tr_sole_sortedmulti_a
keyed_compose_tr_three_paths_extracted_later keyed_compose_tr_two_path_distinct_fingerprints
keyed_compose_tr_two_path_nums keyed_compose_tr_unsorted_sole_leaf keyed_compose_wsh_hash_and_time
keyed_compose_wsh_locked_head_or_i keyed_compose_wsh_single_head_or_i keyed_compose_wsh_sole_sortedmulti
keyed_compose_wsh_three_paths keyed_compose_wsh_two_path_distinct_fingerprints keyed_compose_wsh_two_path_or_d
keyed_compose_wsh_unsorted_sole
```

- [ ] **Step 1: Write the failing pin test**

Create `md/compose_vectors_pin_test.go`:

```go
package md

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// The compose corpus is VENDORED from the Rust primary (descriptor-mnemonic,
// crates/md-codec/tests/vectors/, the MANIFEST's `compose_*` and
// `keyed_compose_*` entries) and pinned here, the same way sysw/testdata/
// sysw_vectors.json is: no sibling checkout, no network, no skip path. A copy
// with no pin is a file nobody can date.
const composeVectorProvenance = "testdata/compose_vectors.provenance.json"

type composeVectorPin struct {
	Comment []string `json:"_comment"`
	Repo    string   `json:"repo"`
	Remote  string   `json:"remote"`
	Commit  string   `json:"commit"`
	Path    string   `json:"path"`
	Files   []struct {
		Name   string `json:"name"`
		SHA256 string `json:"sha256"`
	} `json:"files"`
	Vectors    int    `json:"vectors"`
	RecordedAt string `json:"recorded_at"`
}

// composeVectorNames is the primary's compose corpus at the pinned commit.
// Hand-maintained like singleStringVectorNames, and checked against the pin
// below so a file copied in without a name here (or a name with no file)
// fails rather than silently asserting nothing.
var composeVectorNames = []string{
	"compose_tr_seven_leaves", "compose_tr_thirty_two_slots",
	"compose_wsh_eight_paths", "compose_wsh_thirty_two_slots",
	"keyed_compose_sh_sole", "keyed_compose_sh_two_of_four",
	"keyed_compose_sh_wsh_one_of_two", "keyed_compose_sh_wsh_sole",
	"keyed_compose_tr_extracted_first", "keyed_compose_tr_extracted_later_four_paths",
	"keyed_compose_tr_hash_leaf", "keyed_compose_tr_key_path_only",
	"keyed_compose_tr_nums_three_leaves", "keyed_compose_tr_sole_sortedmulti_a",
	"keyed_compose_tr_three_paths_extracted_later", "keyed_compose_tr_two_path_distinct_fingerprints",
	"keyed_compose_tr_two_path_nums", "keyed_compose_tr_unsorted_sole_leaf",
	"keyed_compose_wsh_hash_and_time", "keyed_compose_wsh_locked_head_or_i",
	"keyed_compose_wsh_single_head_or_i", "keyed_compose_wsh_sole_sortedmulti",
	"keyed_compose_wsh_three_paths", "keyed_compose_wsh_two_path_distinct_fingerprints",
	"keyed_compose_wsh_two_path_or_d", "keyed_compose_wsh_unsorted_sole",
}

func loadComposeVectorPin(t *testing.T) composeVectorPin {
	t.Helper()
	raw, err := os.ReadFile(composeVectorProvenance)
	if err != nil {
		t.Fatalf("INCONCLUSIVE: no provenance pin at %s: %v", composeVectorProvenance, err)
	}
	var p composeVectorPin
	if err := json.Unmarshal(raw, &p); err != nil {
		t.Fatalf("parsing %s: %v", composeVectorProvenance, err)
	}
	if strings.TrimSpace(p.Commit) == "" || strings.TrimSpace(p.Path) == "" {
		t.Fatalf("INCONCLUSIVE: %s names no primary commit and path", composeVectorProvenance)
	}
	return p
}

func TestComposeVectorsMatchTheirProvenancePin(t *testing.T) {
	p := loadComposeVectorPin(t)
	if p.Vectors != len(composeVectorNames) {
		t.Fatalf("pin says %d vectors, this test knows %d", p.Vectors, len(composeVectorNames))
	}
	// 22 keyed vectors carry five files, 4 unkeyed carry four: 126.
	if len(p.Files) != 126 {
		t.Fatalf("pin lists %d files, want 126", len(p.Files))
	}
	seen := map[string]bool{}
	for _, f := range p.Files {
		raw, err := os.ReadFile(filepath.Join("testdata", "vectors", f.Name))
		if err != nil {
			t.Fatalf("pinned file missing: %v", err)
		}
		sum := sha256.Sum256(raw)
		if got := hex.EncodeToString(sum[:]); got != f.SHA256 {
			t.Errorf("%s: sha256 %s, pin says %s", f.Name, got, f.SHA256)
		}
		seen[strings.SplitN(f.Name, ".", 2)[0]] = true
	}
	for _, name := range composeVectorNames {
		if !seen[name] {
			t.Errorf("%s: named here but no file of it is pinned", name)
		}
		delete(seen, name)
	}
	for stray := range seen {
		t.Errorf("%s: pinned file whose vector is not named here", stray)
	}
}

// Every keyed compose vector must be a MEMBER of the keyed conformance gate
// (md/conformance_keyed_test.go globs keyed_*.conformance.json): the ids the
// composer's consent screen shows are exactly what that gate checks.
func TestEveryKeyedComposeVectorHasAConformanceRecord(t *testing.T) {
	for _, name := range composeVectorNames {
		if !strings.HasPrefix(name, "keyed_") {
			continue
		}
		if _, err := os.Stat(vectorPath(name, "conformance.json")); err != nil {
			t.Errorf("%s: %v", name, err)
		}
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run 'TestComposeVectors|TestEveryKeyedComposeVector' ./md/ 2>&1 | tail -6`
Expected: FAIL, `INCONCLUSIVE: no provenance pin at testdata/compose_vectors.provenance.json`.

- [ ] **Step 3: Write the vendoring script and run it**

Create `scripts/vendor-compose-vectors.sh`:

```bash
#!/usr/bin/env bash
# Vendor the Rust primary's compose corpus (the MANIFEST's compose_* and
# keyed_compose_* vectors) into md/testdata/vectors/ and write the provenance
# pin md/compose_vectors_pin_test.go checks. Re-run on every re-pin.
#
#   scripts/vendor-compose-vectors.sh [/path/to/descriptor-mnemonic]
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="${1:-$HERE/../descriptor-mnemonic}"
VEC="$SRC/crates/md-codec/tests/vectors"
DST="$HERE/md/testdata/vectors"
PIN="$HERE/md/testdata/compose_vectors.provenance.json"
[ -d "$VEC" ] || { echo "no vectors at $VEC" >&2; exit 2; }
commit=$(git -C "$SRC" rev-parse HEAD)
clean=true; [ -z "$(git -C "$SRC" status --porcelain -- crates/md-codec/tests/vectors)" ] || clean=false
mapfile -t files < <(cd "$VEC" && ls | grep -E '^(keyed_)?compose_' | sort)
[ "${#files[@]}" -gt 0 ] || { echo "no compose_* vectors in $VEC" >&2; exit 2; }
for f in "${files[@]}"; do cp "$VEC/$f" "$DST/$f"; done
python3 - "$PIN" "$commit" "$clean" "$DST" "${files[@]}" <<'PY'
import hashlib, json, sys, os, datetime
pin, commit, clean, dst, files = sys.argv[1], sys.argv[2], sys.argv[3] == "true", sys.argv[4], sys.argv[5:]
rows = [{"name": f, "sha256": hashlib.sha256(open(os.path.join(dst, f), "rb").read()).hexdigest()} for f in files]
names = sorted({f.split(".", 1)[0] for f in files})
doc = {
  "_comment": [
    "PROVENANCE PIN for the vendored compose corpus (wallet-policy composer, Stage 2).",
    "Generated by scripts/vendor-compose-vectors.sh; never edited by hand.",
    "md/compose_vectors_pin_test.go fails if any pinned file's sha256 disagrees,",
    "if the file count is not 126, or if the vector names drift from its list.",
    "TO RE-PIN: scripts/vendor-compose-vectors.sh /path/to/descriptor-mnemonic",
  ],
  "repo": "descriptor-mnemonic",
  "remote": "git@github.com:bg002h/descriptor-mnemonic.git",
  "commit": commit,
  "repo_clean_when_recorded": clean,
  "path": "crates/md-codec/tests/vectors",
  "files": rows,
  "vectors": len(names),
  "recorded_at": datetime.date.today().isoformat(),
}
json.dump(doc, open(pin, "w"), indent=2); open(pin, "a").write("\n")
print("vendored %d files, %d vectors, primary %s" % (len(rows), len(names), commit[:12]))
PY
```

Run:
```bash
chmod +x scripts/vendor-compose-vectors.sh && scripts/vendor-compose-vectors.sh /scratch/code/shibboleth/descriptor-mnemonic
ls md/testdata/vectors | grep -cE '^(keyed_)?compose_'
```
Expected: `vendored 126 files, 26 vectors, primary 66bdf2f47e7f`; the count line prints `126`.

- [ ] **Step 4: Run the pin test and the keyed conformance gate**

Run: `CGO_ENABLED=0 go test -count=1 -run 'TestComposeVectors|TestEveryKeyedComposeVector|TestKeyedConformanceAgreesWithRust' -v ./md/ 2>&1 | grep -E '^(--- |ok|FAIL|\s+--- )' | sort | uniq -c | sort -rn | head -5`
Expected: the three top-level tests PASS; `TestKeyedConformanceAgreesWithRust` now has 36 sub-tests (14 before), all PASS — the composer's ids agree with Rust before a line of builder code exists. If any `keyed_compose_*` sub-test FAILS, stop: that is a Go-vs-Rust identity divergence on a shape the shipped port has never seen (report the vector and both ids verbatim).

- [ ] **Step 5: Append to `md/testdata/README.md`**

```text

## The compose corpus (wallet-policy composer, Stage 2)

The 26 `compose_*` / `keyed_compose_*` vectors are the Rust primary's
`MANIFEST` entries for the composer (descriptor-mnemonic
`crates/md-codec/tests/compose_support.rs::family()`), vendored by
`scripts/vendor-compose-vectors.sh` and pinned in
`compose_vectors.provenance.json` (checked by `md/compose_vectors_pin_test.go`).
They are all FORCE-CHUNKED, so they are deliberately absent from
`singleStringVectorNames`/`byteParityVectorNames`; their byte and chunk parity
is asserted by `md/compose_test.go` against the BUILDER (`md.Compose`), not
against a hand-loaded descriptor. Two further `family()` entries
(`compose_wsh_keyless_hash_path`, `compose_wsh_keyless_hash_only`) are
`no-corpus`: the primary's exporter refuses a signature-free path, so they are
mirrored as chunk-set literals in `md/compose_test.go`, produced by
`md compose ... --experimental | md encode --experimental --force-chunked`.
```

- [ ] **Step 6: gofmt, commit**

```bash
gofmt -l md/ ; git add md/testdata/vectors/*compose_* md/testdata/compose_vectors.provenance.json scripts/vendor-compose-vectors.sh md/compose_vectors_pin_test.go md/testdata/README.md
git commit -s -F - <<'MSG'
md: vendor the composer's 26-vector corpus with a provenance pin (composer S2 task 1)

126 files from descriptor-mnemonic 66bdf2f4 (22 keyed x 5, 4 unkeyed x 4);
pin test runs everywhere with no sibling checkout; the keyed conformance
gate grows 14 -> 36 sub-tests and every one agrees with Rust.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task 2: The `md` builder — `Compose`, `ComposeWith`, `Composed`, and byte parity with every vector

**Files:**
- Create: `md/compose.go`
- Modify: `md/testdata_test.go:307-317,366-373` (the `.descriptor.json` loader's hash-body and pubkey branches -- see Step 2a)
- Test: `md/compose_test.go`

**Interfaces:**
- Consumes: `descriptor`, `pathDecl`, `originPath`, `pathComponent`, `useSitePath`, `alternative`, `tlvSection`, `idxFP`, `idxPub`, `node`, the tags and bodies (`md/md.go:42-138,190-269,523-533,816-822`); `split` (`md/chunk.go:121`); `encodePayload` (`md/encode.go:374`); `FormAwareStub` (`md/template_id.go:112`); `PathComponent` (`md/encode_singlesig.go:20`).
- Produces (Stage 3 calls these): `type ComposeWrapper` (`ComposeTr|ComposeWsh|ComposeShWsh|ComposeSh`) with `ScriptType() uint32`; `type LockKind`, `type Lock{Kind, Value}` with `Check() error`; `type KeySet{K, N uint8; Sorted bool}`; `type SpendPath{Keys *KeySet; Hash *[32]byte; Lock *Lock}`; `type PathList{Wrapper; Paths []SpendPath}`; `type SlotOrigin{Origin []PathComponent; Fingerprint [4]byte; FpPresent bool}`; `type ComposeSlot{Index uint8; Path int; Ordinal uint8}`; `type ComposeExperimental{Kind ComposeExperimentalKind; Path int}`; `type Composed` with `Slots()`, `InternalKeyPath() (int, bool)`, `Experimental()`, `Chunks() ([]string, error)`, `Stub() ([4]byte, error)`, `TemplateID() ([16]byte, error)`, `Bind(pubkeys map[uint8][65]byte, fingerprints map[uint8][4]byte) error`; `func ValidatePathList(list PathList) (int, error)`; `func Compose(list PathList) (Composed, error)`; `func ComposeWith(list PathList, declared []*SlotOrigin) (Composed, error)`; `func DefaultOrigin(w ComposeWrapper, account uint32) []PathComponent`; the `ErrCompose*` sentinels.

- [ ] **Step 1: Write the failing tests**

The two `no-corpus` chunk sets below were produced at descriptor-mnemonic `66bdf2f4` by:

```bash
md compose --wrapper wsh --path 2of3 --path keyless,sha256=a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8,after=1383520 --experimental
# -> wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*),and_v(v:sha256(a8...a8),after(1383520))))
md encode --experimental --force-chunked "<that template>"      # chunk-set-id 0x3ee58
md compose --wrapper wsh --path 1of1 --path keyless,sha256=a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8 --experimental
# -> wsh(or_i(pkh(@0/48'/0'/0'/2'/<0;1>/*),sha256(a8...a8)))
md encode --experimental --force-chunked "<that template>"      # chunk-set-id 0x3dbf4
```

Create `md/compose_test.go`:

```go
package md

import (
	"bytes"
	"errors"
	"reflect"
	"strings"
	"testing"
)

// ─── the primary's family(), mirrored ─────────────────────────────────────────
//
// Each row is (name, path list, tags) exactly as descriptor-mnemonic
// crates/md-codec/tests/compose_support.rs::family() has it at 66bdf2f4. The
// 26 corpus names are vendored (Task 1); the two `no-corpus` rows are pinned
// by the chunk-set literals below, because the primary's exporter refuses a
// signature-free path and cannot write them to MANIFEST.

var composeH = func() *[32]byte {
	var h [32]byte
	for i := range h {
		h[i] = 0xa8
	}
	return &h
}()

func ck(k, n uint8) SpendPath                           { return SpendPath{Keys: &KeySet{K: k, N: n, Sorted: true}} }
func cu(k, n uint8) SpendPath                           { return SpendPath{Keys: &KeySet{K: k, N: n, Sorted: false}} }
func clk(p SpendPath, l Lock) SpendPath                 { l2 := l; p.Lock = &l2; return p }
func chs(p SpendPath) SpendPath                         { p.Hash = composeH; return p }
func ckl(l *Lock) SpendPath                             { return SpendPath{Hash: composeH, Lock: l} }
func cpl(w ComposeWrapper, paths ...SpendPath) PathList { return PathList{Wrapper: w, Paths: paths} }

func olderBlocks(n uint32) Lock { return Lock{Kind: LockOlderBlocks, Value: n} }
func olderUnits(n uint32) Lock  { return Lock{Kind: LockOlderUnits, Value: n} }
func afterHeight(n uint32) Lock { return Lock{Kind: LockAfterHeight, Value: n} }
func afterTime(n uint32) Lock   { return Lock{Kind: LockAfterTime, Value: n} }

type composeFamilyRow struct {
	name string
	list PathList
	tags []string
}

func composeFamily() []composeFamilyRow {
	eight := func(mk func(i uint32) SpendPath) []SpendPath {
		out := make([]SpendPath, 8)
		for i := range out {
			out[i] = mk(uint32(i))
		}
		return out
	}
	tr32 := eight(func(uint32) SpendPath { return ck(4, 4) })
	return []composeFamilyRow{
		{"keyed_compose_wsh_sole_sortedmulti", cpl(ComposeWsh, ck(2, 3)),
			[]string{"w:wsh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-wsh"}},
		{"keyed_compose_wsh_two_path_or_d", cpl(ComposeWsh, ck(2, 3), clk(ck(1, 1), olderBlocks(26280))),
			[]string{"w:wsh", "paths:2", "head:bare-multi", "lock:blocks", "ik:none", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-wsh"}},
		{"keyed_compose_wsh_two_path_distinct_fingerprints", cpl(ComposeWsh, ck(2, 3), clk(ck(1, 1), olderBlocks(26280))),
			[]string{"w:wsh", "paths:2", "head:bare-multi", "lock:blocks", "ik:none", "fp:distinct", "origins:default-wsh"}},
		{"keyed_compose_wsh_single_head_or_i", cpl(ComposeWsh, ck(1, 1), clk(ck(1, 1), olderUnits(15188))),
			[]string{"w:wsh", "paths:2", "head:single", "lock:units", "ik:none", "fp:one-seed-two-paths", "origins:default-wsh"}},
		{"keyed_compose_wsh_locked_head_or_i", cpl(ComposeWsh, clk(ck(2, 2), afterHeight(905_000)), ck(1, 1)),
			[]string{"w:wsh", "paths:2", "head:locked", "lock:height", "ik:none", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-wsh"}},
		{"keyed_compose_wsh_hash_and_time", cpl(ComposeWsh, ck(1, 1), clk(chs(ck(2, 2)), afterTime(1_893_456_000))),
			[]string{"w:wsh", "paths:2", "head:single", "lock:time", "hash", "ik:none", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-wsh"}},
		{"keyed_compose_wsh_three_paths", cpl(ComposeWsh, ck(1, 1), clk(ck(1, 1), olderBlocks(4032)), clk(ck(1, 1), afterHeight(1_000_000))),
			[]string{"w:wsh", "paths:3", "head:single", "lock:blocks", "lock:height", "ik:none", "fp:one-seed-two-paths", "origins:default-wsh"}},
		{"keyed_compose_wsh_unsorted_sole", cpl(ComposeWsh, cu(2, 3)),
			[]string{"w:wsh", "paths:1", "head:bare-multi", "lock:none", "unsorted", "ik:none", "fp:one-seed-one-path", "origins:default-wsh"}},
		{"keyed_compose_sh_wsh_sole", cpl(ComposeShWsh, ck(2, 3)),
			[]string{"w:sh-wsh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-sh-wsh"}},
		{"keyed_compose_sh_wsh_one_of_two", cpl(ComposeShWsh, ck(1, 2)),
			[]string{"w:sh-wsh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-sh-wsh"}},
		{"keyed_compose_sh_sole", cpl(ComposeSh, ck(2, 2)),
			[]string{"w:sh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-sh"}},
		{"keyed_compose_sh_two_of_four", cpl(ComposeSh, ck(2, 4)),
			[]string{"w:sh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-sh"}},
		{"keyed_compose_tr_two_path_nums", cpl(ComposeTr, ck(2, 3), clk(ck(1, 1), olderBlocks(26280))),
			[]string{"w:tr", "paths:2", "ik:nums", "spine:2", "lock:blocks", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-tr"}},
		{"keyed_compose_tr_two_path_distinct_fingerprints", cpl(ComposeTr, ck(2, 3), clk(ck(1, 1), olderBlocks(26280))),
			[]string{"w:tr", "paths:2", "ik:nums", "spine:2", "lock:blocks", "fp:distinct", "origins:default-tr"}},
		{"keyed_compose_tr_extracted_first", cpl(ComposeTr, ck(1, 1), clk(ck(1, 1), olderBlocks(65535))),
			[]string{"w:tr", "paths:2", "ik:extracted-first", "spine:1", "lock:blocks", "fp:one-seed-two-paths", "origins:default-tr"}},
		{"keyed_compose_tr_extracted_later_four_paths", cpl(ComposeTr, clk(ck(1, 1), olderBlocks(10)), clk(ck(1, 1), afterHeight(1_000_000)), ck(1, 1), clk(ck(1, 1), olderUnits(100))),
			[]string{"w:tr", "paths:4", "ik:extracted-later", "spine:3", "lock:blocks", "lock:height", "lock:units", "fp:one-seed-two-paths", "origins:default-tr"}},
		{"keyed_compose_tr_three_paths_extracted_later", cpl(ComposeTr, clk(ck(1, 1), olderBlocks(10)), ck(1, 1), clk(ck(1, 1), olderUnits(5))),
			[]string{"w:tr", "paths:3", "ik:extracted-later", "spine:2", "lock:blocks", "lock:units", "fp:one-seed-two-paths", "origins:default-tr"}},
		{"keyed_compose_tr_nums_three_leaves", cpl(ComposeTr, clk(ck(1, 1), olderBlocks(1)), clk(ck(1, 1), olderBlocks(2)), clk(ck(2, 2), afterHeight(2))),
			[]string{"w:tr", "paths:3", "ik:nums", "spine:3", "lock:blocks", "lock:height", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-tr"}},
		{"keyed_compose_tr_sole_sortedmulti_a", cpl(ComposeTr, ck(2, 3)),
			[]string{"w:tr", "paths:1", "ik:nums", "spine:1", "lock:none", "sorted", "fp:one-seed-one-path", "origins:default-tr"}},
		{"keyed_compose_tr_key_path_only", cpl(ComposeTr, ck(1, 1)),
			[]string{"w:tr", "paths:1", "ik:extracted-first", "spine:0", "lock:none", "origins:default-tr"}},
		{"keyed_compose_tr_unsorted_sole_leaf", cpl(ComposeTr, cu(2, 2)),
			[]string{"w:tr", "paths:1", "ik:nums", "spine:1", "lock:none", "unsorted", "fp:one-seed-one-path", "origins:default-tr"}},
		{"keyed_compose_tr_hash_leaf", cpl(ComposeTr, ck(2, 2), clk(chs(ck(1, 1)), afterTime(1_893_456_000))),
			[]string{"w:tr", "paths:2", "ik:nums", "spine:2", "hash", "lock:time", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-tr"}},
		{"compose_wsh_keyless_hash_path", cpl(ComposeWsh, ck(2, 3), ckl(&Lock{Kind: LockAfterHeight, Value: 1_383_520})),
			[]string{"w:wsh", "paths:2", "head:bare-multi", "keyless-wsh", "hash", "lock:height", "ik:none", "fp:none", "origins:default-wsh", "no-corpus"}},
		{"compose_wsh_keyless_hash_only", cpl(ComposeWsh, ck(1, 1), ckl(nil)),
			[]string{"w:wsh", "paths:2", "head:single", "keyless-wsh", "hash", "lock:none", "ik:none", "fp:none", "origins:default-wsh", "no-corpus"}},
		{"compose_wsh_eight_paths", cpl(ComposeWsh, eight(func(i uint32) SpendPath { return clk(ck(1, 1), olderBlocks(100+i)) })...),
			[]string{"w:wsh", "paths:8", "head:locked", "lock:blocks", "ik:none", "fp:none", "origins:default-wsh"}},
		{"compose_tr_seven_leaves", cpl(ComposeTr, eight(func(i uint32) SpendPath {
			if i == 0 {
				return ck(1, 1)
			}
			return clk(ck(1, 1), olderBlocks(100+i))
		})...),
			[]string{"w:tr", "paths:8", "ik:extracted-first", "spine:7", "lock:blocks", "fp:none", "origins:default-tr"}},
		{"compose_wsh_thirty_two_slots", cpl(ComposeWsh, ck(9, 9), ck(9, 9), ck(9, 9), ck(5, 5)),
			[]string{"w:wsh", "paths:4", "slots:32", "head:bare-multi", "lock:none", "ik:none", "fp:none", "origins:default-wsh"}},
		{"compose_tr_thirty_two_slots", cpl(ComposeTr, tr32...),
			[]string{"w:tr", "paths:8", "slots:32", "ik:nums", "spine:7", "lock:none", "fp:none", "origins:default-tr"}},
	}
}

// The two no-corpus entries, as `md encode --experimental --force-chunked`
// printed them at descriptor-mnemonic 66bdf2f4 (2026-09-02).
var noCorpusChunks = map[string][]string{
	"compose_wsh_keyless_hash_path": {
		"md1f8mjcqs9qjtvyyy5jmpprjjtvyy49gqpsfsxpzrye4m29g4z52329g4q6xvdgtdqavtat",
		"md1f8mjcqs252329g4z52329g4z52329g4z52329g4z52329gdsq9guvq2q8uaha9yndk0",
	},
	"compose_wsh_keyless_hash_only": {
		"md1f8kl5qspqztvyyy4qqxpxfdm29g4z52329g4z52329g559vylcxqps8u",
		"md1f8kl5qsf29g4z52329g4z52329g4z52329g4z52sq3yc9v383ler7a",
	},
}

func hasTag(tags []string, want string) bool {
	for _, t := range tags {
		if t == want {
			return true
		}
	}
	return false
}

func composeTlvPubkeys(d *descriptor) map[uint8][65]byte {
	out := map[uint8][65]byte{}
	for _, p := range d.tlv.pubkeys {
		out[p.idx] = p.xpub
	}
	return out
}

func composeTlvFingerprints(d *descriptor) map[uint8][4]byte {
	out := map[uint8][4]byte{}
	for _, f := range d.tlv.fingerprints {
		out[f.idx] = f.fp
	}
	return out
}

// TestComposeReproducesEveryVectorByteForByte is §12 item 1's Go half: the
// UNSEATED builder output equals the vendored tree, path declaration and
// use-site; after binding the vector's keys and fingerprints (what the MANIFEST
// binding did in Rust) the payload BYTES and the CHUNK STRINGS are identical.
func TestComposeReproducesEveryVectorByteForByte(t *testing.T) {
	for _, row := range composeFamily() {
		t.Run(row.name, func(t *testing.T) {
			c, err := Compose(row.list)
			if err != nil {
				t.Fatalf("Compose: %v", err)
			}
			if hasTag(row.tags, "no-corpus") {
				chunks := noCorpusChunks[row.name]
				want, err := Reassemble(chunks)
				if err != nil {
					t.Fatalf("Reassemble(no-corpus literal): %v", err)
				}
				if !reflect.DeepEqual(c.d.tree, want.tree) {
					t.Fatalf("tree differs from the primary's:\n got %+v\nwant %+v", c.d.tree, want.tree)
				}
				if !reflect.DeepEqual(c.d.pathDecl, want.pathDecl) {
					t.Fatalf("pathDecl differs: got %+v want %+v", c.d.pathDecl, want.pathDecl)
				}
				got, err := c.Chunks()
				if err != nil {
					t.Fatalf("Chunks: %v", err)
				}
				if !reflect.DeepEqual(got, chunks) {
					t.Fatalf("chunks differ:\n got %v\nwant %v", got, chunks)
				}
				return
			}
			want := loadDescriptor(t, row.name)
			if !reflect.DeepEqual(c.d.tree, want.tree) {
				t.Fatalf("tree differs from the vendored descriptor.json:\n got %+v\nwant %+v", c.d.tree, want.tree)
			}
			if !reflect.DeepEqual(c.d.pathDecl, want.pathDecl) {
				t.Fatalf("pathDecl differs:\n got %+v\nwant %+v", c.d.pathDecl, want.pathDecl)
			}
			if !reflect.DeepEqual(c.d.useSite, want.useSite) || c.d.n != want.n {
				t.Fatalf("useSite/n differ: got %+v/%d want %+v/%d", c.d.useSite, c.d.n, want.useSite, want.n)
			}
			if strings.HasPrefix(row.name, "keyed_") {
				if err := c.Bind(composeTlvPubkeys(want), composeTlvFingerprints(want)); err != nil {
					t.Fatalf("Bind: %v", err)
				}
			} else if len(want.tlv.pubkeys) != 0 || len(want.tlv.fingerprints) != 0 {
				t.Fatalf("an unkeyed vector carries keys or fingerprints")
			}
			gotBytes, _, err := encodePayload(c.d)
			if err != nil {
				t.Fatalf("encodePayload: %v", err)
			}
			if wantBytes := loadBytesHex(t, row.name); !bytes.Equal(gotBytes, wantBytes) {
				t.Fatalf("payload bytes differ:\n got %x\nwant %x", gotBytes, wantBytes)
			}
			gotChunks, err := c.Chunks()
			if err != nil {
				t.Fatalf("Chunks: %v", err)
			}
			if wantChunks := loadPhraseChunks(t, row.name); !reflect.DeepEqual(gotChunks, wantChunks) {
				t.Fatalf("chunks differ:\n got %v\nwant %v", gotChunks, wantChunks)
			}
		})
	}
}

// Every tag appears in at least two vectors, except the ones with exactly one
// legal shape (spec §12 item 1; primary's SINGULAR_TAGS = {"spine:0"}).
func TestComposeFamilyTagsAreCoveredTwice(t *testing.T) {
	count := map[string]int{}
	for _, row := range composeFamily() {
		for _, tag := range row.tags {
			count[tag]++
		}
	}
	for tag, n := range count {
		if tag == "spine:0" || tag == "no-corpus" {
			continue
		}
		if n < 2 {
			t.Errorf("tag %q appears in %d vector(s); the two-vector rule wants 2", tag, n)
		}
	}
	if len(composeFamily()) != 28 {
		t.Errorf("family has %d rows, the primary has 28", len(composeFamily()))
	}
}

// Slot numbering is first-appearance in the EMITTED text (§5): the taproot
// internal key's path is numbered first even when listed later.
func TestComposeNumbersSlotsByFirstAppearance(t *testing.T) {
	c, err := Compose(cpl(ComposeTr, clk(ck(1, 1), olderBlocks(10)), ck(1, 1), clk(ck(1, 1), olderUnits(5))))
	if err != nil {
		t.Fatal(err)
	}
	ik, ok := c.InternalKeyPath()
	if !ok || ik != 1 {
		t.Fatalf("internal key path = %d,%v; want path 1", ik, ok)
	}
	want := []ComposeSlot{{Index: 0, Path: 1, Ordinal: 0}, {Index: 1, Path: 0, Ordinal: 0}, {Index: 2, Path: 2, Ordinal: 0}}
	if !reflect.DeepEqual(c.Slots(), want) {
		t.Fatalf("slots = %+v, want %+v", c.Slots(), want)
	}
	// wsh: listed order IS emitted order.
	c, err = Compose(cpl(ComposeWsh, ck(2, 3), clk(ck(1, 1), olderBlocks(26280))))
	if err != nil {
		t.Fatal(err)
	}
	want = []ComposeSlot{{0, 0, 0}, {1, 0, 1}, {2, 0, 2}, {3, 1, 0}}
	if !reflect.DeepEqual(c.Slots(), want) {
		t.Fatalf("slots = %+v, want %+v", c.Slots(), want)
	}
	if _, ok := c.InternalKeyPath(); ok {
		t.Fatal("a wsh policy reported an internal key path")
	}
}

// §4f: unseated slots take the wrapper's default origin at the LOWEST account
// no other slot holds; a declared slot's origin is respected and skipped over.
func TestComposeWithFillsTheLowestFreeAccount(t *testing.T) {
	list := cpl(ComposeWsh, ck(2, 3))
	acct1 := DefaultOrigin(ComposeWsh, 1)
	c, err := ComposeWith(list, []*SlotOrigin{nil, {Origin: acct1, Fingerprint: [4]byte{0x73, 0xc5, 0xda, 0x0a}, FpPresent: true}, nil})
	if err != nil {
		t.Fatal(err)
	}
	got := c.d.pathDecl.divergent
	want := []originPath{
		{components: toComponents(DefaultOrigin(ComposeWsh, 0))},
		{components: toComponents(acct1)},
		{components: toComponents(DefaultOrigin(ComposeWsh, 2))},
	}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("origins = %+v, want %+v", got, want)
	}
	if !c.d.tlv.fpPresent || len(c.d.tlv.fingerprints) != 1 || c.d.tlv.fingerprints[0].idx != 1 {
		t.Fatalf("fingerprints = %+v, want exactly slot 1's", c.d.tlv.fingerprints)
	}
	// All slots at one declared origin, all with distinct fingerprints: legal,
	// and the path declaration collapses to SHARED.
	shared := DefaultOrigin(ComposeWsh, 0)
	c, err = ComposeWith(list, []*SlotOrigin{
		{Origin: shared, Fingerprint: [4]byte{1}, FpPresent: true},
		{Origin: shared, Fingerprint: [4]byte{2}, FpPresent: true},
		{Origin: shared, Fingerprint: [4]byte{3}, FpPresent: true},
	})
	if err != nil {
		t.Fatal(err)
	}
	if c.d.pathDecl.shared == nil || c.d.pathDecl.divergent != nil {
		t.Fatalf("expected a shared path declaration, got %+v", c.d.pathDecl)
	}
}

func TestComposeDefaultOriginsPerWrapper(t *testing.T) {
	h := func(v uint32) PathComponent { return PathComponent{Hardened: true, Value: v} }
	for _, tc := range []struct {
		w    ComposeWrapper
		want []PathComponent
	}{
		{ComposeWsh, []PathComponent{h(48), h(0), h(5), h(2)}},
		{ComposeSh, []PathComponent{h(48), h(0), h(5), h(2)}},
		{ComposeShWsh, []PathComponent{h(48), h(0), h(5), h(1)}},
		{ComposeTr, []PathComponent{h(48), h(0), h(5), h(3)}},
	} {
		if got := DefaultOrigin(tc.w, 5); !reflect.DeepEqual(got, tc.want) {
			t.Errorf("DefaultOrigin(%v,5) = %+v, want %+v", tc.w, got, tc.want)
		}
	}
	if ComposeTr.ScriptType() != 3 || ComposeShWsh.ScriptType() != 1 || ComposeWsh.ScriptType() != 2 || ComposeSh.ScriptType() != 2 {
		t.Fatal("script-type components do not match §4f")
	}
}

// Every refusal in the primary's validate() refuses here, by sentinel.
func TestComposeRefusesWhatThePrimaryRefuses(t *testing.T) {
	nine := func() []SpendPath {
		out := make([]SpendPath, 9)
		for i := range out {
			out[i] = ck(1, 1)
		}
		return out
	}()
	for _, tc := range []struct {
		name string
		list PathList
		want error
	}{
		{"no paths", cpl(ComposeWsh), ErrComposeNoPaths},
		{"nine paths", cpl(ComposeWsh, nine...), ErrComposeTooManyPaths},
		{"k zero", cpl(ComposeWsh, ck(0, 2)), ErrComposeBadThreshold},
		{"k above n", cpl(ComposeWsh, ck(3, 2)), ErrComposeBadThreshold},
		{"ten keys in a path", cpl(ComposeWsh, ck(1, 10)), ErrComposeBadThreshold},
		{"lock-only path", cpl(ComposeWsh, ck(1, 1), SpendPath{Lock: &Lock{Kind: LockOlderBlocks, Value: 1}}), ErrComposeLockOnlyPath},
		{"keyless under tr", cpl(ComposeTr, ck(1, 1), ckl(nil)), ErrComposeKeylessUnderTr},
		{"no keyed path", cpl(ComposeWsh, ckl(nil)), ErrComposeNoKeyedPath},
		{"33 slots", cpl(ComposeWsh, ck(9, 9), ck(9, 9), ck(9, 9), ck(6, 6)), ErrComposeTooManySlots},
		{"sh with two paths", cpl(ComposeSh, ck(2, 2), ck(1, 1)), ErrComposeLegacyWrapperShape},
		{"sh-wsh unsorted", cpl(ComposeShWsh, cu(2, 2)), ErrComposeLegacyWrapperShape},
		{"sh single key", cpl(ComposeSh, ck(1, 1)), ErrComposeLegacyWrapperShape},
		{"older zero blocks", cpl(ComposeWsh, clk(ck(1, 1), olderBlocks(0))), ErrComposeLockOutOfRange},
		{"older 65536 blocks", cpl(ComposeWsh, clk(ck(1, 1), olderBlocks(65536))), ErrComposeLockOutOfRange},
		{"older zero units", cpl(ComposeWsh, clk(ck(1, 1), olderUnits(0))), ErrComposeLockOutOfRange},
		{"after height at the time threshold", cpl(ComposeWsh, clk(ck(1, 1), afterHeight(500_000_000))), ErrComposeLockOutOfRange},
		{"after time below the threshold", cpl(ComposeWsh, clk(ck(1, 1), afterTime(499_999_999))), ErrComposeLockOutOfRange},
		{"after time above 2^31-1", cpl(ComposeWsh, clk(ck(1, 1), afterTime(2_147_483_648))), ErrComposeLockOutOfRange},
	} {
		_, err := Compose(tc.list)
		if !errors.Is(err, tc.want) {
			t.Errorf("%s: err = %v, want %v", tc.name, err, tc.want)
		}
	}
	// Declared slot count must match.
	if _, err := ComposeWith(cpl(ComposeWsh, ck(2, 3)), []*SlotOrigin{nil, nil}); !errors.Is(err, ErrComposeWrongSlotCount) {
		t.Errorf("wrong slot count: %v", err)
	}
	// Two slots at one origin without two distinct fingerprints (§4f, §8v).
	o := DefaultOrigin(ComposeWsh, 0)
	for _, decl := range [][]*SlotOrigin{
		{{Origin: o}, {Origin: o}, nil},
		{{Origin: o, Fingerprint: [4]byte{1}, FpPresent: true}, {Origin: o}, nil},
		{{Origin: o, Fingerprint: [4]byte{1}, FpPresent: true}, {Origin: o, Fingerprint: [4]byte{1}, FpPresent: true}, nil},
	} {
		if _, err := ComposeWith(cpl(ComposeWsh, ck(2, 3)), decl); !errors.Is(err, ErrComposeIndistinguishableSlots) {
			t.Errorf("indistinguishable slots accepted: %v", err)
		}
	}
}

// §12 item 7: the device-side lock range check, every boundary in and out.
func TestLockCheckIsTheDeviceSideRangeGate(t *testing.T) {
	ok := []Lock{olderBlocks(1), olderBlocks(65535), olderUnits(1), olderUnits(65535), afterHeight(1), afterHeight(499_999_999), afterTime(500_000_000), afterTime(2_147_483_647)}
	bad := []Lock{olderBlocks(0), olderBlocks(65536), olderUnits(0), olderUnits(65536), afterHeight(0), afterHeight(500_000_000), afterTime(499_999_999), afterTime(2_147_483_648), {Kind: LockKind(9), Value: 1}}
	for _, l := range ok {
		if err := l.Check(); err != nil {
			t.Errorf("%+v: %v", l, err)
		}
	}
	for _, l := range bad {
		if err := l.Check(); err == nil {
			t.Errorf("%+v accepted", l)
		}
	}
	// The operand the wire carries: units get the 0x400000 type flag.
	if tag, v, err := olderUnits(15188).operand(); err != nil || tag != tagOlder || v != 4209492 {
		t.Fatalf("older units operand = %v %d %v", tag, v, err)
	}
}

// Experimental marks mirror the primary's `experimental()`: a keyless path
// always; unsorted keys only where sorted would have been legal (the sole
// bare-multi path).
func TestComposeExperimentalMarks(t *testing.T) {
	c, err := Compose(cpl(ComposeWsh, cu(2, 3)))
	if err != nil {
		t.Fatal(err)
	}
	if want := []ComposeExperimental{{Kind: ExperimentalUnsortedKeys, Path: 0}}; !reflect.DeepEqual(c.Experimental(), want) {
		t.Fatalf("marks = %+v, want %+v", c.Experimental(), want)
	}
	c, err = Compose(cpl(ComposeWsh, cu(2, 3), clk(ck(1, 1), olderBlocks(1))))
	if err != nil {
		t.Fatal(err)
	}
	if len(c.Experimental()) != 0 {
		t.Fatalf("multi under or_d is the only legal spelling there; marks = %+v", c.Experimental())
	}
	c, err = Compose(cpl(ComposeWsh, ck(1, 1), ckl(nil)))
	if err != nil {
		t.Fatal(err)
	}
	if want := []ComposeExperimental{{Kind: ExperimentalKeylessPath, Path: 1}}; !reflect.DeepEqual(c.Experimental(), want) {
		t.Fatalf("marks = %+v, want %+v", c.Experimental(), want)
	}
}

// Stub and template id come from the same descriptor the chunks encode, so a
// consumer comparing a card's stub against the composed template agrees with
// the shipped FormAwareStubChunks on the emitted chunks.
func TestComposedStubMatchesTheChunks(t *testing.T) {
	c, err := Compose(cpl(ComposeTr, ck(2, 3), clk(ck(1, 1), olderBlocks(26280))))
	if err != nil {
		t.Fatal(err)
	}
	chunks, err := c.Chunks()
	if err != nil {
		t.Fatal(err)
	}
	fromChunks, err := FormAwareStubChunks(chunks)
	if err != nil {
		t.Fatal(err)
	}
	stub, err := c.Stub()
	if err != nil {
		t.Fatal(err)
	}
	if stub != fromChunks {
		t.Fatalf("Stub %x != FormAwareStubChunks %x", stub, fromChunks)
	}
	tid, err := c.TemplateID()
	if err != nil {
		t.Fatal(err)
	}
	if [4]byte(tid[:4]) != stub {
		t.Fatalf("a keyless template's stub is its template id's first four bytes: %x vs %x", tid, stub)
	}
}
```

- [ ] **Step 2: Run to verify it fails to compile**

Run: `CGO_ENABLED=0 go test -count=1 -run 'TestCompose|TestLockCheck' ./md/ 2>&1 | head -5`
Expected: `undefined: Compose` (and the other new names).

- [ ] **Step 2a: Fix the loader the compose vectors expose (measured in the gate's scratch run)**

`loadDescriptor` had never been called on a KEYED vector or on one carrying a `sha256` node (the keyed conformance gate reads the phrase, and the shipped shape tests load their cards the same way). Three arms of its JSON shim unmarshal into `[]byte`, i.e. expect a JSON byte ARRAY -- but every vendored `.descriptor.json`, old and new, writes pubkeys, fingerprints AND hash bodies as hex STRINGS (`[0, "bba0c7ca…"]`, `"data": "a8a8…"`), exactly as the fingerprint arm already decodes them. Against the unmodified loader, Step 1's test fails all 22 keyed rows with `illegal base64 data at input byte 128`, and after the pubkey fix the two `sha256` rows still differ because `"a8a8…"` base64-decodes to `6b c6 bc …` without an error. In `md/testdata_test.go`:

Replace, in `buildNode`:

```go
	case "Hash256Body":
		var arr []byte
		mustJSON(t, jn.Body.Data, &arr)
		var h hash256Body
		copy(h[:], arr)
		b = h
	case "Hash160Body":
		var arr []byte
		mustJSON(t, jn.Body.Data, &arr)
		var h hash160Body
		copy(h[:], arr)
```

with

```go
	case "Hash256Body":
		// A hex STRING in every vendored vector (as pubkeys and fingerprints
		// are); the []byte reading below it would base64-decode "a8a8..." into
		// 6b c6 bc ... and never fail loudly.
		var h hash256Body
		copy(h[:], hexBody(t, jn.Body.Data, 32))
		b = h
	case "Hash160Body":
		var h hash160Body
		copy(h[:], hexBody(t, jn.Body.Data, 20))
```

Replace, in `buildTLV`'s `if jt.Pubkeys != nil` loop:

```go
			var arr []byte
			mustJSON(t, pair[1], &arr)
			var xpub [65]byte
			copy(xpub[:], arr)
```

with

```go
			// The primary writes each pubkey as a 130-char hex string (as it does
			// fingerprints); older readings of this branch expected a JSON byte
			// array, which no vector carries -- the branch had never been reached,
			// because keyed vectors were loaded from their phrase, not their JSON.
			var hexstr string
			mustJSON(t, pair[1], &hexstr)
			arr, err := hex.DecodeString(hexstr)
			if err != nil || len(arr) != 65 {
				t.Fatalf("bad pubkey %q", hexstr)
			}
			var xpub [65]byte
			copy(xpub[:], arr)
```

and append at the end of the file:

```go
// hexBody decodes a JSON hex string of exactly n bytes (the primary's
// serialization for hash bodies, fingerprints and pubkeys).
func hexBody(t *testing.T, raw json.RawMessage, n int) []byte {
	t.Helper()
	var hexstr string
	mustJSON(t, raw, &hexstr)
	b, err := hex.DecodeString(hexstr)
	if err != nil || len(b) != n {
		t.Fatalf("bad %d-byte hex body %q", n, hexstr)
	}
	return b
}
```

(`encoding/hex` and `encoding/json` are already imported there.) Then run `CGO_ENABLED=0 go test -count=1 ./md/ 2>&1 | tail -2`: the existing tests do not reach these arms and stay `ok`.

- [ ] **Step 3: Write the builder**

Create `md/compose.go`:

```go
package md

// The wallet-policy COMPOSER's tree builder (SPEC_wallet_policy_composer.md
// §5, FIXED lowering) — a line-for-line port of the Rust primary's
// md-codec::compose::{lowering,tr} at descriptor-mnemonic 66bdf2f4. Rust is
// normative: every branch here has a vendored vector or a chunk-set literal
// in compose_test.go that the primary produced, and a divergence is fixed
// HERE, never by editing a vector (CLAUDE.md, Rust-primary rule).
//
// What it does, in order: validate the path list (the primary's validate()),
// number slots by first appearance in the EMITTED text (the taproot internal
// key first, then listed order), lower each path to its node, chain the paths
// (`or_d` under a bare-multi head, `or_i` otherwise; a right-leaning taptree
// spine), resolve slot origins (§4f: declared, else the wrapper's default at
// the lowest account no other slot holds; two slots may share an origin only
// with two distinct fingerprints), and assemble the descriptor the rest of
// this package already knows how to split, identify and emit.
//
// It emits no text. A rendering that cannot be re-parsed is the defect this
// package's invariant exists to prevent; the GUI shows the STRUCTURE
// (PolicyShape) and the ids, and the md1 chunks are the artifact.

import (
	"errors"
	"fmt"
)

// ComposeWrapper is the outermost script form (§4a).
type ComposeWrapper uint8

const (
	ComposeTr ComposeWrapper = iota
	ComposeWsh
	ComposeShWsh
	ComposeSh
)

// ScriptType is BIP-48's script-type component for the wrapper's default
// origins (§4f): 2 for wsh and sh, 1 for sh-wsh, 3 for tr. It is the same
// table gui/multisig_build_slots.go's multisigScriptTypeComponent applies to
// Multisig Build's three wrappers, extended by the taproot arm (§9 item 8).
func (w ComposeWrapper) ScriptType() uint32 {
	switch w {
	case ComposeShWsh:
		return 1
	case ComposeTr:
		return 3
	default:
		return 2
	}
}

func (w ComposeWrapper) isLegacy() bool { return w == ComposeSh || w == ComposeShWsh }

// LockKind is the operator's lock unit (§4c).
type LockKind uint8

const (
	// LockOlderBlocks — older(n), n blocks, 1..=65535.
	LockOlderBlocks LockKind = iota
	// LockOlderUnits — older(0x400000 + u), u units of 512 seconds, 1..=65535.
	LockOlderUnits
	// LockAfterHeight — after(h), a block height, 1..=499,999,999.
	LockAfterHeight
	// LockAfterTime — after(t), a Unix time, 500,000,000..=2,147,483,647.
	LockAfterTime
)

// Lock is one timelock in the operator's units.
type Lock struct {
	Kind  LockKind
	Value uint32
}

// Limits, the primary's compose::{MAX_PATHS, MAX_KEYS_PER_PATH, MAX_SLOTS}.
const (
	ComposeMaxPaths       = 8
	ComposeMaxKeysPerPath = 9
	ComposeMaxSlots       = 32

	sequenceTypeFlag    uint32 = 1 << 22
	locktimeThreshold   uint32 = 500_000_000
	maxAbsoluteLocktime uint32 = 0x7fff_ffff
)

// The refusals, one sentinel per arm of the primary's ComposeError so callers
// (and tests) match with errors.Is; the wrapped message carries the operands.
var (
	ErrComposeNoPaths                = errors.New("md: compose: a wallet needs at least one spend path")
	ErrComposeTooManyPaths           = errors.New("md: compose: more than 8 spend paths")
	ErrComposeNoKeyedPath            = errors.New("md: compose: every path is key-less; at least one path must hold a key")
	ErrComposeLockOnlyPath           = errors.New("md: compose: a path with neither keys nor a hash is not a spend path")
	ErrComposeKeylessUnderTr         = errors.New("md: compose: a key-less path is not expressible under tr")
	ErrComposeBadThreshold           = errors.New("md: compose: threshold needs 1 <= k <= n <= 9")
	ErrComposeTooManySlots           = errors.New("md: compose: this wallet would have more key slots than the wire holds (32)")
	ErrComposeLegacyWrapperShape     = errors.New("md: compose: sh and sh-wsh admit exactly one sortedmulti path")
	ErrComposeLockOutOfRange         = errors.New("md: compose: lock operand outside §4c")
	ErrComposeWrongSlotCount         = errors.New("md: compose: declarations given for a different number of slots than the policy has")
	ErrComposeIndistinguishableSlots = errors.New("md: compose: two slots declare the same origin without two distinct fingerprints; a template like that cannot be restored")
)

// operand is the tag and consensus operand this lock encodes to (§4c).
func (l Lock) operand() (tag, uint32, error) {
	switch l.Kind {
	case LockOlderBlocks:
		if l.Value == 0 || l.Value > 0xffff {
			return 0, 0, errors.New("older in blocks needs 1..=65535")
		}
		return tagOlder, l.Value, nil
	case LockOlderUnits:
		if l.Value == 0 || l.Value > 0xffff {
			return 0, 0, errors.New("older in 512-second units needs 1..=65535")
		}
		return tagOlder, sequenceTypeFlag + l.Value, nil
	case LockAfterHeight:
		if l.Value == 0 || l.Value >= locktimeThreshold {
			return 0, 0, errors.New("after height needs 1..=499999999")
		}
		return tagAfter, l.Value, nil
	case LockAfterTime:
		if l.Value < locktimeThreshold || l.Value > maxAbsoluteLocktime {
			return 0, 0, errors.New("after time needs 500000000..=2147483647")
		}
		return tagAfter, l.Value, nil
	}
	return 0, 0, errors.New("unknown lock kind")
}

// Check is the DEVICE-SIDE §4c range gate (§12 item 7): a unit gate on the
// emitter's input, independent of what md's decoder would accept.
func (l Lock) Check() error {
	_, _, err := l.operand()
	return err
}

// KeySet is k-of-n over FRESH slots (§4b). Sorted asks for sortedmulti /
// sortedmulti_a where the position allows it; false asks for multi / multi_a
// there, which is EXPERIMENTAL.
type KeySet struct {
	K, N   uint8
	Sorted bool
}

// SpendPath is one alternative way to spend: optional keys, optional sha256
// preimage, optional lock. A path with neither keys nor a hash is refused.
type SpendPath struct {
	Keys *KeySet
	Hash *[32]byte
	Lock *Lock
}

func (p SpendPath) isBareMulti() bool {
	return p.Keys != nil && p.Keys.N >= 2 && p.Hash == nil && p.Lock == nil
}

func (p SpendPath) isBareSingle() bool {
	return p.Keys != nil && p.Keys.N == 1 && p.Hash == nil && p.Lock == nil
}

// PathList is the operator's ordered list under one wrapper.
type PathList struct {
	Wrapper ComposeWrapper
	Paths   []SpendPath
}

// SlotOrigin is one slot's declared origin (and optional fingerprint); a nil
// *SlotOrigin in ComposeWith means "unseated: take the §4f default".
type SlotOrigin struct {
	Origin      []PathComponent
	Fingerprint [4]byte
	FpPresent   bool
}

// ComposeSlot maps an emitted slot index to the path and ordinal it came from.
type ComposeSlot struct {
	Index   uint8
	Path    int
	Ordinal uint8
}

// ComposeExperimentalKind marks a shape the primary admits only under
// --experimental (§5; the GUI shows the §8 warning for each).
type ComposeExperimentalKind uint8

const (
	ExperimentalKeylessPath ComposeExperimentalKind = iota
	ExperimentalUnsortedKeys
)

// ComposeExperimental is one mark: the kind and the path it is about.
type ComposeExperimental struct {
	Kind ComposeExperimentalKind
	Path int
}

// Composed is a built, not-yet-keyed (or keyed via Bind) descriptor with its
// slot map.
type Composed struct {
	d               *descriptor
	slots           []ComposeSlot
	internalKeyPath int // -1 when the internal key is NUMS
	experimental    []ComposeExperimental
}

// Slots is the emitted slot map, index-ascending.
func (c Composed) Slots() []ComposeSlot { return c.slots }

// InternalKeyPath is the path extracted as the taproot internal key, if any.
func (c Composed) InternalKeyPath() (int, bool) {
	return c.internalKeyPath, c.internalKeyPath >= 0
}

// Experimental lists the §5 experimental marks, path-ascending.
func (c Composed) Experimental() []ComposeExperimental { return c.experimental }

// Chunks emits the md1 chunk set (always chunk form, as the primary's
// force_chunked vectors are).
func (c Composed) Chunks() ([]string, error) { return split(c.d) }

// Stub is the form-aware 4-byte stub a key card carries for this artifact.
func (c Composed) Stub() ([4]byte, error) { return FormAwareStub(c.d) }

// TemplateID is the key-independent wallet descriptor template id.
func (c Composed) TemplateID() ([16]byte, error) { return WalletDescriptorTemplateId(c.d) }

// Bind attaches a 65-byte chaincode‖compressed-pubkey per slot (every slot
// required) and optional fingerprints (added to, or replacing, the ones the
// declarations carried), producing the KEYED form. It is what Rust's MANIFEST
// binding did to make the keyed_compose_* vectors.
func (c *Composed) Bind(pubkeys map[uint8][65]byte, fingerprints map[uint8][4]byte) error {
	n := int(c.d.n)
	if len(pubkeys) != n {
		return fmt.Errorf("md: compose: Bind needs a key for each of %d slots, got %d", n, len(pubkeys))
	}
	pubs := make([]idxPub, n)
	for i := 0; i < n; i++ {
		x, ok := pubkeys[uint8(i)]
		if !ok {
			return fmt.Errorf("md: compose: Bind has no key for slot @%d", i)
		}
		pubs[i] = idxPub{idx: uint8(i), xpub: x}
	}
	c.d.tlv.pubkeys = pubs
	c.d.tlv.pubPresent = true
	if len(fingerprints) > 0 {
		merged := map[uint8][4]byte{}
		for _, f := range c.d.tlv.fingerprints {
			merged[f.idx] = f.fp
		}
		for idx, fp := range fingerprints {
			if int(idx) >= n {
				return fmt.Errorf("md: compose: Bind fingerprint for slot @%d beyond %d slots", idx, n)
			}
			merged[idx] = fp
		}
		fps := make([]idxFP, 0, len(merged))
		for i := 0; i < n; i++ {
			if fp, ok := merged[uint8(i)]; ok {
				fps = append(fps, idxFP{idx: uint8(i), fp: fp})
			}
		}
		c.d.tlv.fingerprints = fps
		c.d.tlv.fpPresent = len(fps) > 0
	}
	return nil
}

// DefaultOrigin is §4f's m/48'/0'/<account>'/<script-type>'.
func DefaultOrigin(w ComposeWrapper, account uint32) []PathComponent {
	return []PathComponent{
		{Hardened: true, Value: 48},
		{Hardened: true, Value: 0},
		{Hardened: true, Value: account},
		{Hardened: true, Value: w.ScriptType()},
	}
}

// ValidatePathList is the primary's validate(): the slot count on success.
func ValidatePathList(list PathList) (int, error) {
	if len(list.Paths) == 0 {
		return 0, ErrComposeNoPaths
	}
	if len(list.Paths) > ComposeMaxPaths {
		return 0, fmt.Errorf("%w: got %d", ErrComposeTooManyPaths, len(list.Paths))
	}
	slots := 0
	anyKeyed := false
	for i, p := range list.Paths {
		if ks := p.Keys; ks != nil {
			if ks.K == 0 || ks.N == 0 || ks.K > ks.N || ks.N > ComposeMaxKeysPerPath {
				return 0, fmt.Errorf("%w: path %d has %d-of-%d", ErrComposeBadThreshold, i, ks.K, ks.N)
			}
			slots += int(ks.N)
			anyKeyed = true
		} else if p.Hash == nil {
			return 0, fmt.Errorf("%w: path %d", ErrComposeLockOnlyPath, i)
		} else if list.Wrapper == ComposeTr {
			return 0, fmt.Errorf("%w: path %d", ErrComposeKeylessUnderTr, i)
		}
		if p.Lock != nil {
			if err := p.Lock.Check(); err != nil {
				return 0, fmt.Errorf("%w: path %d: %v", ErrComposeLockOutOfRange, i, err)
			}
		}
	}
	if !anyKeyed {
		return 0, ErrComposeNoKeyedPath
	}
	if slots > ComposeMaxSlots {
		return 0, fmt.Errorf("%w: got %d", ErrComposeTooManySlots, slots)
	}
	if list.Wrapper.isLegacy() {
		sole := len(list.Paths) == 1 && list.Paths[0].isBareMulti()
		sorted := list.Paths[0].Keys != nil && list.Paths[0].Keys.Sorted
		if !(sole && sorted) {
			return 0, ErrComposeLegacyWrapperShape
		}
	}
	return slots, nil
}

// Compose lowers an all-unseated list: every slot takes its §4f default.
func Compose(list PathList) (Composed, error) {
	slots, err := ValidatePathList(list)
	if err != nil {
		return Composed{}, err
	}
	return lowerPathList(list, make([]*SlotOrigin, slots))
}

// ComposeWith lowers a list whose slots may carry declared origins (one entry
// per emitted slot, index-ascending; nil = unseated).
func ComposeWith(list PathList, declared []*SlotOrigin) (Composed, error) {
	slots, err := ValidatePathList(list)
	if err != nil {
		return Composed{}, err
	}
	if len(declared) != slots {
		return Composed{}, fmt.Errorf("%w: %d given, policy has %d", ErrComposeWrongSlotCount, len(declared), slots)
	}
	return lowerPathList(list, declared)
}

// ─── lowering (the primary's lowering.rs) ─────────────────────────────────────

type numberedPath struct {
	path      SpendPath
	pathIndex int
	slots     []uint8
}

func keyLeaf(single, multi, sorted tag, ks KeySet, slots []uint8, sortedLegal bool) node {
	if ks.N == 1 {
		return node{tag: single, body: keyArgBody{index: slots[0]}}
	}
	t := multi
	if sortedLegal && ks.Sorted {
		t = sorted
	}
	idx := make([]uint8, len(slots))
	copy(idx, slots)
	return node{tag: t, body: multiKeysBody{k: ks.K, indices: idx}}
}

func verifyNode(x node) node { return node{tag: tagVerify, body: childrenBody{children: []node{x}}} }

func andV(a, b node) node {
	return node{tag: tagAndV, body: childrenBody{children: []node{verifyNode(a), b}}}
}

// pathBody lowers one path: KEYS, then sha256, then the lock, right-nested as
// and_v(v:KEYS, and_v(v:sha256(H), LOCK)).
func pathBody(p numberedPath, tap, sortedLegal bool) node {
	var parts []node
	if ks := p.path.Keys; ks != nil {
		if tap {
			parts = append(parts, keyLeaf(tagPkK, tagMultiA, tagSortedMultiA, *ks, p.slots, sortedLegal))
		} else {
			parts = append(parts, keyLeaf(tagPkH, tagMulti, tagSortedMulti, *ks, p.slots, sortedLegal))
		}
	}
	if h := p.path.Hash; h != nil {
		parts = append(parts, node{tag: tagSha256, body: hash256Body(*h)})
	}
	if l := p.path.Lock; l != nil {
		t, v, err := l.operand()
		if err != nil {
			panic("md: compose: lock validated by ValidatePathList: " + err.Error())
		}
		parts = append(parts, node{tag: t, body: timelockBody(v)})
	}
	acc := parts[len(parts)-1]
	for i := len(parts) - 2; i >= 0; i-- {
		acc = andV(parts[i], acc)
	}
	return acc
}

// wshChain chains the paths: or_d when the head is a bare multi (its
// satisfaction is a clean DUP-IF-able boolean), or_i otherwise.
func wshChain(paths []numberedPath) node {
	sole := len(paths) == 1
	nodes := make([]node, len(paths))
	for i, p := range paths {
		nodes[i] = pathBody(p, false, sole && p.path.isBareMulti())
	}
	acc := nodes[len(nodes)-1]
	for i := len(paths) - 2; i >= 0; i-- {
		t := tagOrI
		if paths[i].path.isBareMulti() {
			t = tagOrD
		}
		acc = node{tag: t, body: childrenBody{children: []node{nodes[i], acc}}}
	}
	return acc
}

// numberSlots assigns slot indices by first appearance: `first` (the taproot
// internal key's path, or -1) before listed order.
func numberSlots(list PathList, first int) ([]numberedPath, []ComposeSlot) {
	order := make([]int, 0, len(list.Paths))
	if first >= 0 {
		order = append(order, first)
	}
	for i := range list.Paths {
		if i != first {
			order = append(order, i)
		}
	}
	var next uint8
	var slots []ComposeSlot
	byPath := make([]numberedPath, len(list.Paths))
	for _, pi := range order {
		p := list.Paths[pi]
		var mine []uint8
		if p.Keys != nil {
			for ord := uint8(0); ord < p.Keys.N; ord++ {
				slots = append(slots, ComposeSlot{Index: next, Path: pi, Ordinal: ord})
				mine = append(mine, next)
				next++
			}
		}
		byPath[pi] = numberedPath{path: p, pathIndex: pi, slots: mine}
	}
	return byPath, slots
}

func experimentalMarks(list PathList, soleSortedLegal func(int) bool) []ComposeExperimental {
	var out []ComposeExperimental
	for i, p := range list.Paths {
		switch {
		case p.Keys == nil:
			out = append(out, ComposeExperimental{Kind: ExperimentalKeylessPath, Path: i})
		case p.Keys.N >= 2 && !p.Keys.Sorted && soleSortedLegal(i):
			out = append(out, ComposeExperimental{Kind: ExperimentalUnsortedKeys, Path: i})
		}
	}
	return out
}

func sameOrigin(a, b []pathComponent) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}

func originTaken(taken [][]pathComponent, o []pathComponent) bool {
	for _, t := range taken {
		if sameOrigin(t, o) {
			return true
		}
	}
	return false
}

// resolveOrigins is §4f: declared origins stand; every unseated slot takes the
// wrapper's default at the lowest account no other slot (declared or filled
// earlier) holds; then the pairwise invariant.
func resolveOrigins(list PathList, declared []*SlotOrigin) (pathDecl, []idxFP, error) {
	n := len(declared)
	origins := make([][]pathComponent, n)
	fps := make([]*[4]byte, n)
	var taken [][]pathComponent
	for i, s := range declared {
		if s != nil {
			origins[i] = toComponents(s.Origin)
			taken = append(taken, origins[i])
			if s.FpPresent {
				fp := s.Fingerprint
				fps[i] = &fp
			}
		}
	}
	for i, s := range declared {
		if s != nil {
			continue
		}
		for account := uint32(0); ; account++ {
			candidate := toComponents(DefaultOrigin(list.Wrapper, account))
			if !originTaken(taken, candidate) {
				taken = append(taken, candidate)
				origins[i] = candidate
				break
			}
		}
	}
	for a := 0; a < n; a++ {
		for b := a + 1; b < n; b++ {
			if sameOrigin(origins[a], origins[b]) {
				distinct := fps[a] != nil && fps[b] != nil && *fps[a] != *fps[b]
				if !distinct {
					return pathDecl{}, nil, fmt.Errorf("%w: slots @%d and @%d", ErrComposeIndistinguishableSlots, a, b)
				}
			}
		}
	}
	allSame := true
	for i := 1; i < n; i++ {
		if !sameOrigin(origins[0], origins[i]) {
			allSame = false
			break
		}
	}
	var pd pathDecl
	if allSame {
		shared := originPath{components: origins[0]}
		pd = pathDecl{n: uint8(n), shared: &shared}
	} else {
		div := make([]originPath, n)
		for i := range origins {
			div[i] = originPath{components: origins[i]}
		}
		pd = pathDecl{n: uint8(n), divergent: div}
	}
	var out []idxFP
	for i, fp := range fps {
		if fp != nil {
			out = append(out, idxFP{idx: uint8(i), fp: *fp})
		}
	}
	return pd, out, nil
}

func finishComposed(list PathList, declared []*SlotOrigin, tree node, slots []ComposeSlot, ik int, exp []ComposeExperimental) (Composed, error) {
	pd, fps, err := resolveOrigins(list, declared)
	if err != nil {
		return Composed{}, err
	}
	d := &descriptor{
		n:        uint8(len(declared)),
		pathDecl: pd,
		useSite: useSitePath{
			hasMultipath:     true,
			multipath:        []alternative{{hardened: false, value: 0}, {hardened: false, value: 1}},
			wildcardHardened: false,
		},
		tree: tree,
		tlv: tlvSection{
			fpPresent:    len(fps) > 0,
			fingerprints: fps,
		},
	}
	return Composed{d: d, slots: slots, internalKeyPath: ik, experimental: exp}, nil
}

func lowerPathList(list PathList, declared []*SlotOrigin) (Composed, error) {
	if list.Wrapper == ComposeTr {
		return lowerTr(list, declared)
	}
	numbered, slots := numberSlots(list, -1)
	sole := len(list.Paths) == 1
	inner := wshChain(numbered)
	var tree node
	switch list.Wrapper {
	case ComposeSh:
		tree = node{tag: tagSh, body: childrenBody{children: []node{inner}}}
	case ComposeShWsh:
		tree = node{tag: tagSh, body: childrenBody{children: []node{{tag: tagWsh, body: childrenBody{children: []node{inner}}}}}}
	default:
		tree = node{tag: tagWsh, body: childrenBody{children: []node{inner}}}
	}
	exp := experimentalMarks(list, func(i int) bool { return sole && list.Paths[i].isBareMulti() })
	return finishComposed(list, declared, tree, slots, -1, exp)
}

// ─── taproot (the primary's tr.rs) ────────────────────────────────────────────

// lowerTr extracts the FIRST-LISTED unlocked, unhashed single key as the
// internal key (else NUMS); the remaining paths become leaves on a
// right-leaning spine (depth of leaf j is min(j, m-1)).
func lowerTr(list PathList, declared []*SlotOrigin) (Composed, error) {
	ik := -1
	for i, p := range list.Paths {
		if p.isBareSingle() {
			ik = i
			break
		}
	}
	numbered, slots := numberSlots(list, ik)
	var leafPaths []numberedPath
	for _, n := range numbered {
		if n.pathIndex != ik {
			leafPaths = append(leafPaths, n)
		}
	}
	m := len(leafPaths)
	leaves := make([]node, m)
	for i, n := range leafPaths {
		leaves[i] = pathBody(n, true, m == 1 && n.path.isBareMulti())
	}
	var spine *node
	if m > 0 {
		acc := leaves[m-1]
		for i := m - 2; i >= 0; i-- {
			acc = node{tag: tagTapTree, body: childrenBody{children: []node{leaves[i], acc}}}
		}
		spine = &acc
	}
	tree := node{tag: tagTr, body: trBody{isNums: ik < 0, keyIndex: 0, tree: spine}}
	exp := experimentalMarks(list, func(i int) bool { return m == 1 && i != ik && list.Paths[i].isBareMulti() })
	return finishComposed(list, declared, tree, slots, ik, exp)
}
```

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run 'TestCompose|TestLockCheck' -v ./md/ 2>&1 | grep -E '^(--- |\s+--- |ok|FAIL)' | grep -v PASS; CGO_ENABLED=0 go test -count=1 -run 'TestCompose|TestLockCheck' ./md/ 2>&1 | tail -2`
Expected: no FAIL lines; `ok seedhammer.com/md`. All 28 family rows byte- and chunk-identical. If ANY row's tree differs, print both trees and stop: the diff names the lowering rule the port got wrong; fix the port (the vector is the primary's). If the tree matches and the BYTES differ, the difference is in binding or TLV ordering, not lowering — say which.

- [ ] **Step 5: Whole-package run, gofmt, commit**

Run: `gofmt -l md/ && CGO_ENABLED=0 go test -count=1 ./md/ 2>&1 | tail -2`
Expected: gofmt prints nothing; `ok`.

```bash
git add md/compose.go md/compose_test.go
git commit -s -F - <<'MSG'
md: the composer's tree builder -- Compose/ComposeWith, FIXED lowering, 4f origins, byte parity with all 28 family vectors (composer S2 task 2)

Port of md-codec::compose::{lowering,tr} at 66bdf2f4. Every corpus vector's
tree, path declaration, payload bytes and chunk strings are reproduced; the two
no-corpus keyless-wsh shapes are pinned by md's own chunk output.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task 3: The `pk_h` emitter arm, in both contexts, against Rust's addresses

**Files:**
- Modify: `md/script_emit.go:154-178` (add one `case tagPkH:` arm beside `case tagPkK:`; import `github.com/btcsuite/btcd/address/v2`)
- Test: `md/compose_pkh_emit_test.go`

**Interfaces:**
- Consumes: `emitFragment(n node, e emitEnv, out *[]byte) error` and `emitEnv{keys map[uint8][]byte; tap bool}` (`md/script_emit.go:88-91,154`); `pushData` (`:116`); opcodes `opDUP`, `opHASH160`, `opEQUALVERIFY`, `opCHECKSIG` (`:30-42`); `EmitWitnessScriptChunks(strs []string, keys map[uint8][]byte) ([]byte, error)` (`:97`); `EmitTapLeavesChunks(strs, keys) (internalKeyIndex uint8, isNUMS bool, leaves []TapLeafScript, err)` (`md/tapleaves.go:188`); the vendored `keyed_compose_*.conformance.json` (`keys[].{index,xpub}`, `chains["0"].addresses[]`).
- Produces: `pkh(@i)` on the wire emits `OP_DUP OP_HASH160 <hash160(key)> OP_EQUALVERIFY OP_CHECKSIG` in segwit-v0 and tapscript alike (the key is whatever the caller supplied: 33-byte compressed in wsh, 32-byte x-only in tap, exactly as the `PkK` arm pushes it).

Why an oracle and not a golden: the vendored `.conformance.json` records were computed by the Rust primary from the same keys, so a P2WSH address equal to Rust's proves the whole script — opcodes, push sizes, the hash, and every other arm on the path (`or_i`, `and_v`, `older`, `after`, `sha256`, `multi`) — not just the `pk_h` bytes. Before this task, `EmitWitnessScriptChunks` returns `ErrScriptUnsupported` on every `pkh` vector (`emitFragment`'s default arm); that is the red.

- [ ] **Step 1: Write the failing test**

Create `md/compose_pkh_emit_test.go`:

```go
package md

import (
	"crypto/sha256"
	"encoding/json"
	"os"
	"testing"

	btcaddr "github.com/btcsuite/btcd/address/v2"
	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"github.com/btcsuite/btcd/chaincfg/v2"
)

// The keyed compose vectors whose wsh body contains pkh(@i) -- the shape
// Multisig Build never produced, so the emitter never had to know it.
var pkhWshVectors = []string{
	"keyed_compose_wsh_two_path_or_d",
	"keyed_compose_wsh_single_head_or_i",
	"keyed_compose_wsh_locked_head_or_i",
	"keyed_compose_wsh_hash_and_time",
	"keyed_compose_wsh_three_paths",
}

type composeConformanceKeys struct {
	Keys []struct {
		Index uint8  `json:"index"`
		Xpub  string `json:"xpub"`
	} `json:"keys"`
	Chains map[string]struct {
		Addresses []string `json:"addresses"`
	} `json:"chains"`
}

func loadComposeConformance(t *testing.T, name string) composeConformanceKeys {
	t.Helper()
	raw, err := os.ReadFile(vectorPath(name, "conformance.json"))
	if err != nil {
		t.Fatalf("read: %v", err)
	}
	var rec composeConformanceKeys
	if err := json.Unmarshal(raw, &rec); err != nil {
		t.Fatalf("parse: %v", err)
	}
	return rec
}

// derivedKeys derives every slot's compressed pubkey at <chain>/<index> from
// the record's account xpubs (use-site <0;1>/*).
func derivedKeys(t *testing.T, rec composeConformanceKeys, chain, index uint32) map[uint8][]byte {
	t.Helper()
	keys := map[uint8][]byte{}
	for _, k := range rec.Keys {
		ek, err := hdkeychain.NewKeyFromString(k.Xpub)
		if err != nil {
			t.Fatalf("@%d xpub: %v", k.Index, err)
		}
		c, err := ek.Derive(chain)
		if err != nil {
			t.Fatalf("@%d/%d: %v", k.Index, chain, err)
		}
		c, err = c.Derive(index)
		if err != nil {
			t.Fatalf("@%d/%d/%d: %v", k.Index, chain, index, err)
		}
		pub, err := c.ECPubKey()
		if err != nil {
			t.Fatalf("@%d pubkey: %v", k.Index, err)
		}
		keys[k.Index] = pub.SerializeCompressed()
	}
	return keys
}

func p2wshAddress(t *testing.T, script []byte) string {
	t.Helper()
	h := sha256.Sum256(script)
	a, err := btcaddr.NewAddressWitnessScriptHash(h[:], &chaincfg.MainNetParams)
	if err != nil {
		t.Fatalf("p2wsh: %v", err)
	}
	return a.EncodeAddress()
}

// TestPkhWitnessScriptsReproduceRustsAddresses: for each pkh-bearing vector,
// the emitted witness script's P2WSH address at receive index 0 and 1 equals
// what the Rust primary derived for the same descriptor and keys.
func TestPkhWitnessScriptsReproduceRustsAddresses(t *testing.T) {
	for _, name := range pkhWshVectors {
		t.Run(name, func(t *testing.T) {
			rec := loadComposeConformance(t, name)
			chunks := loadPhraseChunks(t, name)
			want := rec.Chains["0"].Addresses
			if len(want) < 2 {
				t.Fatalf("record has %d receive addresses, want >= 2", len(want))
			}
			for i := uint32(0); i < 2; i++ {
				script, err := EmitWitnessScriptChunks(chunks, derivedKeys(t, rec, 0, i))
				if err != nil {
					t.Fatalf("EmitWitnessScriptChunks(receive %d): %v", i, err)
				}
				if got := p2wshAddress(t, script); got != want[i] {
					t.Errorf("receive %d:\n  go:   %s\n  rust: %s", i, got, want[i])
				}
			}
		})
	}
}

// The key enters the script through its hash: a different key at the pkh slot
// moves the address. (The manual mutation in Step 5 -- swap OP_EQUALVERIFY for
// OP_EQUAL -- is the one that proves the OPCODES are checked; this one proves
// the HASH is of the supplied key and not a constant.)
func TestPkhScriptDependsOnTheKey(t *testing.T) {
	name := "keyed_compose_wsh_single_head_or_i" // or_i(pkh(@0), and_v(v:pkh(@1), older(..)))
	rec := loadComposeConformance(t, name)
	chunks := loadPhraseChunks(t, name)
	keys := derivedKeys(t, rec, 0, 0)
	base, err := EmitWitnessScriptChunks(chunks, keys)
	if err != nil {
		t.Fatal(err)
	}
	flipped := map[uint8][]byte{}
	for k, v := range keys {
		flipped[k] = append([]byte(nil), v...)
	}
	flipped[0][32] ^= 0x01
	mut, err := EmitWitnessScriptChunks(chunks, flipped)
	if err != nil {
		t.Fatal(err)
	}
	if p2wshAddress(t, base) == p2wshAddress(t, mut) {
		t.Fatal("changing slot @0's key did not change the address: the pkh arm is not hashing the key")
	}
	if len(base) != len(mut) {
		t.Fatalf("a key change altered the script LENGTH (%d vs %d); pkh must push a fixed 20-byte hash", len(base), len(mut))
	}
}

// Tapscript context: the composer never emits pkh under tr (path_body uses pk /
// multi_a there), but §9 item 2 asks for the arm in both contexts. A hand-built
// tr(NUMS, pkh(@0)) leaf must emit DUP HASH160 <hash160(xonly)> EQUALVERIFY
// CHECKSIG with the 32-byte key the caller supplied.
func TestPkhTapLeafEmitsTheHash160Form(t *testing.T) {
	leaf := node{tag: tagPkH, body: keyArgBody{index: 0}}
	shared := originPath{components: toComponents(DefaultOrigin(ComposeTr, 0))}
	d := &descriptor{
		n:        1,
		pathDecl: pathDecl{n: 1, shared: &shared},
		useSite: useSitePath{
			hasMultipath: true,
			multipath:    []alternative{{hardened: false, value: 0}, {hardened: false, value: 1}},
		},
		tree: node{tag: tagTr, body: trBody{isNums: true, keyIndex: 0, tree: &leaf}},
	}
	chunks, err := split(d)
	if err != nil {
		t.Fatalf("split: %v", err)
	}
	xonly := make([]byte, 32)
	for i := range xonly {
		xonly[i] = byte(i + 1)
	}
	_, isNUMS, leaves, err := EmitTapLeavesChunks(chunks, map[uint8][]byte{0: xonly})
	if err != nil {
		t.Fatalf("EmitTapLeavesChunks: %v", err)
	}
	if !isNUMS || len(leaves) != 1 {
		t.Fatalf("isNUMS=%v leaves=%d, want NUMS with one leaf", isNUMS, len(leaves))
	}
	want := append([]byte{opDUP, opHASH160, 0x14}, btcaddr.Hash160(xonly)...)
	want = append(want, opEQUALVERIFY, opCHECKSIG)
	if got := leaves[0].Script; string(got) != string(want) {
		t.Fatalf("leaf script\n got %x\nwant %x", got, want)
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run 'TestPkh' ./md/ 2>&1 | grep -E 'FAIL|unsupported|ok' | head -8`
Expected: every `TestPkhWitnessScriptsReproduceRustsAddresses` sub-test FAILS with `EmitWitnessScriptChunks(receive 0): md: script emission unsupported` (the `ErrScriptUnsupported` text as the package spells it); `TestPkhScriptDependsOnTheKey` and `TestPkhTapLeafEmitsTheHash160Form` FAIL the same way. If the TAP test fails for a different reason (`split` or `Reassemble` refusing the hand-built tree, or `leaves[0].Script` not being the field's name), record the exact error and fix the TEST's construction to whatever `md/tapleaves.go` requires -- the arm under test is unchanged by that.

- [ ] **Step 3: Add the arm**

In `md/script_emit.go`, add to the import block (gofmt places it between `"errors"` and `"sort"`; the block has no blank-line groups):

```go
	btcaddr "github.com/btcsuite/btcd/address/v2"
```

and directly after the `case tagPkK:` arm's `return nil` (before `case tagCheck:`), add:

```go
	case tagPkH:
		// `pkh(K)` on the wire is miniscript's `c:pk_h(K)`, the same implicit
		// `c:` as PkK above (SPEC §5.1 Q12), so the arm emits the whole
		// DUP HASH160 <hash160(K)> EQUALVERIFY CHECKSIG. The key is hashed AS
		// SUPPLIED: 33-byte compressed in segwit-v0, 32-byte x-only under tap
		// (BIP-342's pk_h hashes the x-only key), exactly as PkK pushes it.
		//
		// The composer is the first producer of this fragment on this device
		// (Multisig Build only ever wrote sortedmulti); md/compose_pkh_emit_test.go
		// pins it to the Rust primary's addresses for five vectors.
		b, ok := n.body.(keyArgBody)
		if !ok {
			return ErrScriptUnsupported
		}
		k, ok := e.keys[b.index]
		if !ok {
			return ErrScriptUnsupported
		}
		*out = append(*out, opDUP, opHASH160)
		pushData(out, btcaddr.Hash160(k))
		*out = append(*out, opEQUALVERIFY, opCHECKSIG)
		return nil
```

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run 'TestPkh' -v ./md/ 2>&1 | grep -E '^(--- |\s+--- |ok|FAIL)'`
Expected: all PASS -- ten receive addresses across five vectors equal Rust's; the key-dependence test passes; the tap leaf equals the hash160 form.

- [ ] **Step 5: The opcode mutation, by hand, with the failing output kept**

Temporarily change `opEQUALVERIFY` to `opEQUAL` in the NEW arm only, run `CGO_ENABLED=0 go test -count=1 -run 'TestPkhWitnessScriptsReproduceRustsAddresses' ./md/ 2>&1 | grep -c 'rust:'`, and paste the count into the implementation report: it must be `10` (every address moved). Revert the mutation (`git diff --stat` shows only the intended files afterwards) and re-run Step 4.

- [ ] **Step 6: Whole-package run, gofmt, commit**

Run: `gofmt -l md/ && CGO_ENABLED=0 go test -count=1 ./md/ 2>&1 | tail -2`
Expected: gofmt prints nothing; `ok`.

```bash
git add md/script_emit.go md/compose_pkh_emit_test.go
git commit -s -F - <<'MSG'
md: pk_h emitter arm in both script contexts, pinned to Rust's addresses for five pkh vectors (composer S2 task 3)

DUP HASH160 <hash160(K)> EQUALVERIFY CHECKSIG; hand mutation of EQUALVERIFY
moved all ten oracle addresses.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task 4: `PolicyShape` reports every alternative, with lock operands and digests

**Files:**
- Modify: `md/policy_shape.go:42-59` (two `Branch` fields), `:101-128` (call the splitter), `:134-154` (leaf → splitter), `:216-221` (record operands and digests)
- Test: `md/compose_shape_test.go`

**Interfaces:**
- Consumes: `PolicyShape`, `Branch`, `policyShape`, `walkTapTree`, `branchOf`, `collect`, `plainMulti` (`md/policy_shape.go`); `PolicyShapeChunks` (`:74`); `timelockBody`, `hash256Body` (`md/md.go:120,122`).
- Produces: `Branch.LockOperands []uint32` (every `older`/`after` operand in the branch, wire order) and `Branch.Sha256Digests [][32]byte` (every `sha256` digest); `or_b`/`or_c`/`or_d`/`or_i` split into one `Branch` per alternative and `andor(X,Y,Z)` into `and(X,Y)` plus `Z`; unchanged: `Complete`, `KeyPath`, `TapDepth`, `K`/`N`/`Keys`/`Timelock`/`Hashlock`/`Depth`, and every existing expectation in `md/policy_shape_test.go` (none of its vectors contains an `or_*` or `andor`; measured: `grep -c 'Or\|andor' md/policy_shape_test.go` = 0 at `169073c`).

Why: §7e's consent surface shows "Path 2: 1 key, after 26280 blocks" per alternative. Today `or_d(multi(2,...), and_v(v:pkh(@3),older(26280)))` is ONE `Branch{Keys:4, Timelock:true}` -- true, and useless to an operator deciding what they are committing to steel. `thresh(k, …)` with `k < n` is also a set of alternatives, but combinatorial; it stays one branch (the composer never emits it), and `Complete` stays honest because `collect` still understands it.

- [ ] **Step 1: Write the failing test**

Create `md/compose_shape_test.go`:

```go
package md

import (
	"reflect"
	"testing"
)

func shapeOf(t *testing.T, name string) PolicyShape {
	t.Helper()
	s, err := PolicyShapeChunks(loadPhraseChunks(t, name))
	if err != nil {
		t.Fatalf("%s: %v", name, err)
	}
	if !s.Complete {
		t.Fatalf("%s: shape reported INCOMPLETE", name)
	}
	return s
}

func TestPolicyShapeSplitsAlternativesIntoBranches(t *testing.T) {
	h := [32]byte{}
	for i := range h {
		h[i] = 0xa8
	}
	for _, tc := range []struct {
		vector string
		want   []Branch
	}{
		// or_d(multi(2,@0,@1,@2), and_v(v:pkh(@3), older(26280)))
		{"keyed_compose_wsh_two_path_or_d", []Branch{
			{K: 2, N: 3, Keys: 3},
			{Keys: 1, Timelock: true, LockOperands: []uint32{26280}},
		}},
		// or_i(pkh(@0), or_i(and_v(v:pkh(@1),older(4032)), and_v(v:pkh(@2),after(1000000))))
		{"keyed_compose_wsh_three_paths", []Branch{
			{Keys: 1},
			{Keys: 1, Timelock: true, LockOperands: []uint32{4032}},
			{Keys: 1, Timelock: true, LockOperands: []uint32{1_000_000}},
		}},
		// or_i(pkh(@0), and_v(v:multi(2,@1,@2), and_v(v:sha256(H), after(1893456000))))
		{"keyed_compose_wsh_hash_and_time", []Branch{
			{Keys: 1},
			{Keys: 2, Timelock: true, Hashlock: true, LockOperands: []uint32{1_893_456_000}, Sha256Digests: [][32]byte{h}},
		}},
		// or_i(and_v(v:multi(2,@0,@1), after(905000)), pkh(@2))
		{"keyed_compose_wsh_locked_head_or_i", []Branch{
			{Keys: 2, Timelock: true, LockOperands: []uint32{905_000}},
			{Keys: 1},
		}},
	} {
		t.Run(tc.vector, func(t *testing.T) {
			s := shapeOf(t, tc.vector)
			if s.KeyPath != KeyPathNone {
				t.Errorf("KeyPath = %v, want none for wsh", s.KeyPath)
			}
			if !reflect.DeepEqual(s.Branches, tc.want) {
				t.Fatalf("branches\n got %+v\nwant %+v", s.Branches, tc.want)
			}
		})
	}
}

// Eight or_i-chained paths: eight branches, each carrying its own operand.
func TestPolicyShapeWalksAnEightPathChain(t *testing.T) {
	s := shapeOf(t, "compose_wsh_eight_paths")
	if len(s.Branches) != 8 {
		t.Fatalf("branches = %d, want 8 (%+v)", len(s.Branches), s.Branches)
	}
	for i, b := range s.Branches {
		if b.Keys != 1 || !b.Timelock || len(b.LockOperands) != 1 || b.LockOperands[0] != uint32(100+i) {
			t.Errorf("branch %d = %+v, want 1 key, older(%d)", i, b, 100+i)
		}
	}
}

// A taproot leaf list is unchanged by the split (one leaf, one branch) but
// now carries operands: tr(NUMS,{and_v(v:pk(@0),older(1)),{and_v(v:pk(@1),older(2)),and_v(v:multi_a(2,@2,@3),after(2))}}).
func TestPolicyShapeTapLeavesCarryOperands(t *testing.T) {
	s := shapeOf(t, "keyed_compose_tr_nums_three_leaves")
	want := []Branch{
		{Keys: 1, Timelock: true, LockOperands: []uint32{1}, Depth: 1},
		{Keys: 1, Timelock: true, LockOperands: []uint32{2}, Depth: 2},
		{Keys: 2, Timelock: true, LockOperands: []uint32{2}, Depth: 2},
	}
	if s.KeyPath != KeyPathNUMS || s.TapDepth != 2 {
		t.Fatalf("KeyPath=%v TapDepth=%d, want NUMS depth 2", s.KeyPath, s.TapDepth)
	}
	if !reflect.DeepEqual(s.Branches, want) {
		t.Fatalf("branches\n got %+v\nwant %+v", s.Branches, want)
	}
}

// The keyless-wsh no-corpus shape: the second alternative has NO key, and the
// summary must say so (Keys 0) rather than refuse -- it is a legal EXPERIMENTAL
// wallet the operator asked for, and hiding the bearer path would be the lie.
func TestPolicyShapeReportsAKeylessAlternativeHonestly(t *testing.T) {
	s, err := PolicyShapeChunks(noCorpusChunks["compose_wsh_keyless_hash_path"])
	if err != nil {
		t.Fatal(err)
	}
	if !s.Complete || len(s.Branches) != 2 {
		t.Fatalf("Complete=%v branches=%d", s.Complete, len(s.Branches))
	}
	if b := s.Branches[1]; b.Keys != 0 || !b.Hashlock || !b.Timelock || len(b.Sha256Digests) != 1 || len(b.LockOperands) != 1 || b.LockOperands[0] != 1_383_520 {
		t.Fatalf("keyless branch = %+v", b)
	}
}

// andor(X,Y,Z) is (X and Y) or Z: two branches, hand-built because no vector
// carries one.
func TestPolicyShapeSplitsAndOr(t *testing.T) {
	x := node{tag: tagPkK, body: keyArgBody{index: 0}}
	y := node{tag: tagOlder, body: timelockBody(144)}
	z := node{tag: tagPkH, body: keyArgBody{index: 1}}
	tree := node{tag: tagWsh, body: childrenBody{children: []node{{tag: tagAndOr, body: childrenBody{children: []node{x, y, z}}}}}}
	s := policyShape(tree)
	want := []Branch{{Keys: 1, Timelock: true, LockOperands: []uint32{144}}, {Keys: 1}}
	if !s.Complete || !reflect.DeepEqual(s.Branches, want) {
		t.Fatalf("Complete=%v branches\n got %+v\nwant %+v", s.Complete, s.Branches, want)
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run 'TestPolicyShapeSplits|TestPolicyShapeWalksAnEight|TestPolicyShapeTapLeavesCarry|TestPolicyShapeReportsAKeyless' ./md/ 2>&1 | grep -E 'FAIL|unknown field|undefined' | head -5`
Expected: compile error `unknown field LockOperands in struct literal of type Branch`.

- [ ] **Step 3: Extend `Branch`, record operands, split alternatives**

In `md/policy_shape.go`, replace the `Branch` struct's `Timelock`/`Hashlock` block (lines 52-56) with:

```go
	// Timelock/Hashlock report whether the branch requires one ANYWHERE within
	// it. LockOperands and Sha256Digests carry the values, in wire order, so
	// the composer's consent surface (SPEC_wallet_policy_composer.md §7e) can
	// say "after 26280 blocks" instead of "time-locked"; a value is a fact from
	// the decoded tree, not a rendering, so the no-text rule above still holds.
	// Only sha256 digests are carried (the composer emits no other hash); the
	// other hash tags still set Hashlock.
	Timelock      bool
	Hashlock      bool
	LockOperands  []uint32
	Sha256Digests [][32]byte
```

Replace the three `branchOf` call sites so alternatives split. In `policyShape`'s `case tagWsh, tagSh:` arm, replace

```go
		br, ok := branchOf(inner, 0)
		if !ok {
			return PolicyShape{}
		}
		s.Branches = append(s.Branches, br)
```

with

```go
		brs, ok := splitBranches(inner, 0)
		if !ok {
			return PolicyShape{}
		}
		s.Branches = append(s.Branches, brs...)
```

and make the same replacement in the `default:` arm (`branchOf(tree, 0)` → `splitBranches(tree, 0)`, appending `brs...`). In `walkTapTree`, replace

```go
	br, ok := branchOf(n, depth-1)
	if !ok {
		s.Complete = false
		return
	}
	s.Branches = append(s.Branches, br)
```

with

```go
	brs, ok := splitBranches(n, depth-1)
	if !ok {
		s.Complete = false
		return
	}
	s.Branches = append(s.Branches, brs...)
```

Add the splitter directly above `branchOf`:

```go
// splitBranches turns a node into one Branch per ALTERNATIVE: or_b/or_c/or_d/
// or_i are two alternatives each (recursively), andor(X,Y,Z) is (X and Y) or
// Z, and anything else is one branch. thresh(k,...) with k < n is also a set
// of alternatives but a combinatorial one; it stays one branch, honestly
// described by collect. ok=false propagates an unknown tag exactly as
// branchOf does.
func splitBranches(n node, depth int) ([]Branch, bool) {
	switch n.tag {
	case tagOrB, tagOrC, tagOrD, tagOrI:
		b, ok := n.body.(childrenBody)
		if !ok || len(b.children) != 2 {
			return nil, false
		}
		left, ok := splitBranches(b.children[0], depth)
		if !ok {
			return nil, false
		}
		right, ok := splitBranches(b.children[1], depth)
		if !ok {
			return nil, false
		}
		return append(left, right...), true
	case tagAndOr:
		b, ok := n.body.(childrenBody)
		if !ok || len(b.children) != 3 {
			return nil, false
		}
		// The X-and-Y half is summarized as a conjunction; the synthetic node is
		// never emitted or encoded, only walked.
		xy := node{tag: tagAndV, body: childrenBody{children: []node{b.children[0], b.children[1]}}}
		left, ok := splitBranches(xy, depth)
		if !ok {
			return nil, false
		}
		right, ok := splitBranches(b.children[2], depth)
		if !ok {
			return nil, false
		}
		return append(left, right...), true
	default:
		br, ok := branchOf(n, depth)
		if !ok {
			return nil, false
		}
		return []Branch{br}, true
	}
}
```

In `collect`, replace the two lock/hash arms

```go
	case tagAfter, tagOlder:
		br.Timelock = true
		return true
	case tagSha256, tagHash256, tagRipemd160, tagHash160:
		br.Hashlock = true
		return true
```

with

```go
	case tagAfter, tagOlder:
		br.Timelock = true
		if v, ok := n.body.(timelockBody); ok {
			br.LockOperands = append(br.LockOperands, uint32(v))
		}
		return true
	case tagSha256:
		br.Hashlock = true
		if h, ok := n.body.(hash256Body); ok {
			br.Sha256Digests = append(br.Sha256Digests, [32]byte(h))
		}
		return true
	case tagHash256, tagRipemd160, tagHash160:
		br.Hashlock = true
		return true
```

- [ ] **Step 4: Run the new AND the existing shape tests**

Run: `CGO_ENABLED=0 go test -count=1 -run 'TestPolicyShape' -v ./md/ 2>&1 | grep -E '^(--- |\s+--- |ok|FAIL)' | grep -v PASS; CGO_ENABLED=0 go test -count=1 -run 'TestPolicyShape' ./md/ 2>&1 | tail -1`
Expected: no FAIL lines; `ok`. The four pre-existing tests (`TestPolicyShapeDescribesRealCards`, `…NeverClaimsAPlainThresholdItCannotSee`, `…RefusesAnUnknownTag`, `…ReportsEveryLeafOfADeepTree`) are UNTOUCHED and still pass; if one moves, the split changed a shape the shipped consent screen shows -- stop and record which.

- [ ] **Step 5: Whole-package run, gofmt, commit**

Run: `gofmt -l md/ && CGO_ENABLED=0 go test -count=1 ./md/ 2>&1 | tail -2`
Expected: gofmt prints nothing; `ok`.

```bash
git add md/policy_shape.go md/compose_shape_test.go
git commit -s -F - <<'MSG'
md: PolicyShape splits or_*/andor into one Branch per alternative, carrying lock operands and sha256 digests (composer S2 task 4)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task 5: Stubs for re-minted key cards (`md.ComposerStubs`, `mk.AppendStubs`)

**Files:**
- Create: `md/compose_stubs.go`
- Create: `mk/compose_stubs.go`
- Test: `md/compose_stubs_test.go`, `mk/compose_stubs_test.go`

**Interfaces:**
- Consumes: `FormAwareStubChunks(strs []string) ([4]byte, error)` (`md/template_id.go:122`; keyless template → WDT-Id stub, keyed → WalletPolicyId stub); `StripToTemplate` (`gui/template_engrave.go:25` calls `md.StripToTemplate(b.MD1)`); `mk.Card{Network, Path, Fingerprint, Stubs [][4]byte, Xpub}`, `mk.Encode`, `mk.Decode` (`mk/mk.go:133`, `mk/encode.go:39`, `mk/mk.go:148`); the vendored `keyed_compose_*.conformance.json` ids.
- Produces: `func ComposerStubs(templateChunks, keyedChunks []string) ([][4]byte, error)` -- the template stub always, then the keyed policy's stub when `keyedChunks != nil` and it differs (§7c "stamping BOTH stubs"; §12 item 6 "both stubs when a keyed policy exists, the template stub otherwise"); `func AppendStubs(card Card, stubs ...[4]byte) Card` in `mk` -- existing stubs preserved in order, each new stub appended once (§7d "APPENDED to its existing stubs").

- [ ] **Step 1: Write the failing tests**

Create `md/compose_stubs_test.go`:

```go
package md

import (
	"encoding/hex"
	"encoding/json"
	"os"
	"testing"
)

// For a keyed vector, the two stubs a re-minted card carries are the first
// four bytes of the two ids the Rust primary recorded: template id from the
// STRIPPED template, policy id from the keyed chunks.
func TestComposerStubsAreTheTwoIdsFirstFourBytes(t *testing.T) {
	name := "keyed_compose_wsh_sole_sortedmulti"
	raw, err := os.ReadFile(vectorPath(name, "conformance.json"))
	if err != nil {
		t.Fatal(err)
	}
	var rec struct {
		TemplateID string `json:"wallet_descriptor_template_id"`
		PolicyID   string `json:"wallet_policy_id"`
	}
	if err := json.Unmarshal(raw, &rec); err != nil {
		t.Fatal(err)
	}
	keyed := loadPhraseChunks(t, name)
	template, err := StripToTemplate(keyed)
	if err != nil {
		t.Fatalf("StripToTemplate: %v", err)
	}
	both, err := ComposerStubs(template, keyed)
	if err != nil {
		t.Fatal(err)
	}
	if len(both) != 2 {
		t.Fatalf("stubs = %x, want two", both)
	}
	if got, want := hex.EncodeToString(both[0][:]), rec.TemplateID[:8]; got != want {
		t.Errorf("template stub %s, rust template id starts %s", got, want)
	}
	if got, want := hex.EncodeToString(both[1][:]), rec.PolicyID[:8]; got != want {
		t.Errorf("policy stub %s, rust policy id starts %s", got, want)
	}
	// Template only (no keyed policy yet, §12 item 6): one stub.
	one, err := ComposerStubs(template, nil)
	if err != nil {
		t.Fatal(err)
	}
	if len(one) != 1 || one[0] != both[0] {
		t.Fatalf("template-only stubs = %x, want just %x", one, both[0])
	}
}
```

Create `mk/compose_stubs_test.go`:

```go
package mk

import (
	"reflect"
	"testing"
)

// The journey's cosigner @0 at m/48'/0'/0'/2' (BIP-39 "abandon" mnemonic).
const composeTestXpub = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"

func TestAppendStubsPreservesExistingAndAddsEachOnce(t *testing.T) {
	existing := [4]byte{0xde, 0xad, 0xbe, 0xef}
	tmpl := [4]byte{1, 2, 3, 4}
	pol := [4]byte{5, 6, 7, 8}
	card := Card{Network: "mainnet", Path: "m/48'/0'/0'/2'", Fingerprint: "73c5da0a", Stubs: [][4]byte{existing}, Xpub: composeTestXpub}
	got := AppendStubs(card, tmpl, pol, tmpl)
	if want := [][4]byte{existing, tmpl, pol}; !reflect.DeepEqual(got.Stubs, want) {
		t.Fatalf("stubs = %x, want %x", got.Stubs, want)
	}
	if !reflect.DeepEqual(card.Stubs, [][4]byte{existing}) {
		t.Fatal("AppendStubs mutated its input")
	}
	// A stub the card already carries is not repeated.
	again := AppendStubs(got, existing)
	if !reflect.DeepEqual(again.Stubs, got.Stubs) {
		t.Fatalf("re-appending an existing stub changed the list: %x", again.Stubs)
	}
	// The re-minted card round-trips through the wire with all three, in order.
	strs, err := Encode(got)
	if err != nil {
		t.Fatalf("Encode: %v", err)
	}
	back, err := Decode(strs)
	if err != nil {
		t.Fatalf("Decode: %v", err)
	}
	if !reflect.DeepEqual(back.Stubs, got.Stubs) || back.Xpub != composeTestXpub || back.Fingerprint != "73c5da0a" {
		t.Fatalf("round trip: %+v", back)
	}
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `CGO_ENABLED=0 go test -count=1 -run 'TestComposerStubs' ./md/ 2>&1 | head -3; CGO_ENABLED=0 go test -count=1 -run 'TestAppendStubs' ./mk/ 2>&1 | head -3`
Expected: `undefined: ComposerStubs`; `undefined: AppendStubs`.

- [ ] **Step 3: Write both helpers**

Create `md/compose_stubs.go`:

```go
package md

// ComposerStubs returns the stubs a key card minted or re-minted by the
// composer carries (SPEC_wallet_policy_composer.md §7c, §7d; §12 item 6): the
// composed TEMPLATE's stub always, and the composed KEYED policy's stub after
// seating, so one card seats into either engraved form. Both come from
// FormAwareStubChunks, which is what the shipped seating compares against
// (gui/key_card_seating.go), so a card stamped here seats there.
//
// keyedChunks may be nil (no keyed policy yet). If the two stubs happen to
// coincide the second is not repeated.
func ComposerStubs(templateChunks, keyedChunks []string) ([][4]byte, error) {
	tmpl, err := FormAwareStubChunks(templateChunks)
	if err != nil {
		return nil, err
	}
	out := [][4]byte{tmpl}
	if keyedChunks != nil {
		pol, err := FormAwareStubChunks(keyedChunks)
		if err != nil {
			return nil, err
		}
		if pol != tmpl {
			out = append(out, pol)
		}
	}
	return out, nil
}
```

Create `mk/compose_stubs.go`:

```go
package mk

// AppendStubs returns a copy of card carrying its existing stubs, in order,
// followed by each given stub it does not already carry (SPEC_wallet_policy_composer.md
// §7d: a seated card "is later cut as a RE-MINTED mk1 carrying BOTH the composed
// template's stub and the composed policy's stub APPENDED to its existing
// stubs", so it stays indexed to the wallets it already belonged to). The input
// is not mutated; Encode's stub_count bound (<= 255) is enforced by Encode.
func AppendStubs(card Card, stubs ...[4]byte) Card {
	out := card
	out.Stubs = make([][4]byte, 0, len(card.Stubs)+len(stubs))
	out.Stubs = append(out.Stubs, card.Stubs...)
	for _, s := range stubs {
		dup := false
		for _, have := range out.Stubs {
			if have == s {
				dup = true
				break
			}
		}
		if !dup {
			out.Stubs = append(out.Stubs, s)
		}
	}
	return out
}
```

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run 'TestComposerStubs' -v ./md/ 2>&1 | tail -3; CGO_ENABLED=0 go test -count=1 -run 'TestAppendStubs' -v ./mk/ 2>&1 | tail -3`
Expected: both PASS; the two stubs equal the first eight hex characters of Rust's `wallet_descriptor_template_id` and `wallet_policy_id`.

- [ ] **Step 5: gofmt, commit**

```bash
gofmt -l md/ mk/ ; git add md/compose_stubs.go md/compose_stubs_test.go mk/compose_stubs.go mk/compose_stubs_test.go
git commit -s -F - <<'MSG'
md, mk: ComposerStubs (template stub, plus the keyed policy's after seating) and AppendStubs for re-minted cards (composer S2 task 5)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task 6: The three composer record classes in `sysw.Classify`, lockstep with the host fixture

**Files:**
- Create: `sysw/composer_records.go`
- Modify: `sysw/record.go:24-44` (three `Class` values), `:100-125` (one dispatch before `classifyConstellation`)
- Create: `sysw/testdata/record_class_vectors.json` (vendored), `sysw/testdata/record_class_vectors.provenance.json`
- Test: `sysw/composer_records_test.go`

**Interfaces:**
- Consumes: `Class`, `Classify`, `classifyConstellation` (`sysw/record.go:24,100`, `sysw/classify.go:34`); `bip32.ParsePathElement`, `bip32.Path` (`bip32/bip32.go:69,18`); `hdkeychain.NewKeyFromString`; the host's `crates/me-cli/testdata/record_class_vectors.json` (40 rows, sha256 `a894e619580db8ca0e06ebfe45576cc45722f695913bf46e9285201c95f146c3`, S1 Task 3) with rows `{name, record, class, host_line}` and `class` in `Key|Hash|Now|Unknown`.
- Produces: `KeyPrefix = "key:"`, `HashPrefix = "hash:"`, `NowPrefix = "now:"`; `ClassKey`, `ClassHash`, `ClassNow` (appended after `ClassTx`; none secret); `type KeyRecord{Fingerprint [4]byte; Origin bip32.Path; Xpub string; Text string}`; `type NowRecord{Seconds uint32; Height uint32; HasHeight bool}`; `func ParseKeyRecord(record string) (KeyRecord, error)`, `func ParseHashRecord(record string) ([32]byte, error)`, `func ParseNowRecord(record string) (NowRecord, error)`; `func IsComposerRecord(record string) bool`. Every rule is the host's (`composer_records.rs`, S1 Task 1), ported predicate by predicate; the device emits NO line (§8n is host copy; the device leaves a malformed record inert, §12 item 8).

Decisions stated: `DecodeBody` (`sysw/record.go:67`) is NOT widened to the three prefixes -- it decodes bodies for ENGRAVING (`text:`/`pass:`/`tx:`), and no composer record is engraved as text; the composer's parser owns its own lowercase-hex rule so the lockstep port is one unit (the same reasoning the host module states for not sharing `record.rs`'s helpers). `H` as a hardening marker is refused here as on the host (`bip32.ParsePathElement` accepts only `h` and `'`); a leading `+` or `-` in a path element is refused by an explicit digit check because `strconv.ParseInt` would accept it and rust-bitcoin does not.

- [ ] **Step 1: Vendor the fixture and write the failing test**

```bash
cp /scratch/code/shibboleth/mnemonic-engrave/crates/me-cli/testdata/record_class_vectors.json sysw/testdata/
sha256sum sysw/testdata/record_class_vectors.json
```
Expected: `a894e619580db8ca0e06ebfe45576cc45722f695913bf46e9285201c95f146c3`. If it differs, the host fixture moved since S1 closed; stop and record both hashes.

Create `sysw/testdata/record_class_vectors.provenance.json`:

```json
{
  "_comment": [
    "PROVENANCE PIN for the vendored copy of record_class_vectors.json (wallet-policy composer, SPEC 12 item 8).",
    "The rows are GENERATED by the Rust primary's `regenerate` test (crates/me-cli/tests/sysw_composer_records.rs)",
    "from its CASES table, never edited by hand; sysw/composer_records_test.go fails if sha256 and the file disagree",
    "or if any row's class differs from Classify's answer.",
    "TO RE-SYNC: cp ../mnemonic-engrave/crates/me-cli/testdata/record_class_vectors.json sysw/testdata/ ;",
    "  git -C ../mnemonic-engrave rev-parse HEAD ; git -C ../mnemonic-engrave log -1 --format=%H -- crates/me-cli/testdata/record_class_vectors.json ;",
    "  sha256sum sysw/testdata/record_class_vectors.json"
  ],
  "repo": "mnemonic-engrave",
  "remote": "git@github.com:bg002h/mnemonic-engrave.git",
  "path": "crates/me-cli/testdata/record_class_vectors.json",
  "commit": "<git -C ../mnemonic-engrave rev-parse HEAD at vendoring>",
  "file_commit": "<git -C ../mnemonic-engrave log -1 --format=%H -- crates/me-cli/testdata/record_class_vectors.json>",
  "repo_clean_when_recorded": true,
  "sha256": "a894e619580db8ca0e06ebfe45576cc45722f695913bf46e9285201c95f146c3",
  "vectors": 40,
  "recorded_at": "2026-09-02"
}
```

Fill `commit` and `file_commit` from the two commands in the comment (full 40-character SHAs; `gh`/git queries with short SHAs have silently returned empty before).

Create `sysw/composer_records_test.go`:

```go
package sysw

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"os"
	"testing"
)

const (
	recordClassVectors    = "testdata/record_class_vectors.json"
	recordClassProvenance = "testdata/record_class_vectors.provenance.json"
)

type recordClassRow struct {
	Name     string  `json:"name"`
	Record   string  `json:"record"`
	Class    string  `json:"class"`
	HostLine *string `json:"host_line"`
}

var recordClassByName = map[string]Class{
	"Key": ClassKey, "Hash": ClassHash, "Now": ClassNow, "Unknown": ClassUnknown,
}

func loadRecordClassRows(t *testing.T) []recordClassRow {
	t.Helper()
	raw, err := os.ReadFile(recordClassVectors)
	if err != nil {
		t.Fatalf("INCONCLUSIVE: no vendored fixture at %s: %v", recordClassVectors, err)
	}
	pinRaw, err := os.ReadFile(recordClassProvenance)
	if err != nil {
		t.Fatalf("INCONCLUSIVE: no provenance pin at %s: %v", recordClassProvenance, err)
	}
	var pin struct {
		SHA256  string `json:"sha256"`
		Vectors int    `json:"vectors"`
		Commit  string `json:"commit"`
	}
	if err := json.Unmarshal(pinRaw, &pin); err != nil {
		t.Fatalf("parsing pin: %v", err)
	}
	if sum := sha256.Sum256(raw); hex.EncodeToString(sum[:]) != pin.SHA256 {
		t.Fatalf("fixture sha256 %x, pin says %s", sum, pin.SHA256)
	}
	if len(pin.Commit) != 40 {
		t.Fatalf("pin commit %q is not a full SHA", pin.Commit)
	}
	var rows []recordClassRow
	if err := json.Unmarshal(raw, &rows); err != nil {
		t.Fatalf("parsing fixture: %v", err)
	}
	if len(rows) != pin.Vectors || len(rows) != 40 {
		t.Fatalf("fixture has %d rows, pin says %d, plan says 40", len(rows), pin.Vectors)
	}
	return rows
}

// SPEC §12 item 8: each key:, hash:, now: record (valid and each §6a
// malformation) classifies identically on the host and on the device. Rust's
// answer is the row; a disagreement is fixed in Go.
func TestComposerRecordsClassifyExactlyAsTheHost(t *testing.T) {
	rows := loadRecordClassRows(t)
	seen := map[string]int{}
	for _, row := range rows {
		want, ok := recordClassByName[row.Class]
		if !ok {
			t.Fatalf("%s: fixture class %q is not one this test knows", row.Name, row.Class)
		}
		seen[row.Class]++
		if got := Classify(row.Record); got != want {
			t.Errorf("%s: Classify(%.60q) = %v, want %v (host's answer)", row.Name, row.Record, got, want)
		}
		// A malformed composer record is Unknown on the device and carries the
		// host's line; a valid one has none. The two must agree row by row.
		if (row.HostLine != nil) != (want == ClassUnknown) {
			t.Errorf("%s: host_line present=%v but class %s", row.Name, row.HostLine != nil, row.Class)
		}
	}
	for _, cls := range []string{"Key", "Hash", "Now", "Unknown"} {
		if seen[cls] == 0 {
			t.Errorf("fixture exercises no %s row; the gate would prove nothing for that class", cls)
		}
	}
}

// The parsed values behind the classes, on the fixture's valid rows.
func TestComposerRecordParsersReturnTheHostsValues(t *testing.T) {
	rows := loadRecordClassRows(t)
	byName := map[string]recordClassRow{}
	for _, r := range rows {
		byName[r.Name] = r
	}
	k, err := ParseKeyRecord(byName["key-journey-cosigner-0"].Record)
	if err != nil {
		t.Fatalf("key-journey-cosigner-0: %v", err)
	}
	if hex.EncodeToString(k.Fingerprint[:]) != "73c5da0a" || k.Origin.String() != "m/48h/0h/0h/2h" {
		t.Errorf("key = fp %x origin %s", k.Fingerprint, k.Origin.String())
	}
	if k.Xpub != "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf" {
		t.Errorf("xpub = %s", k.Xpub)
	}
	if k.Text != "[73c5da0a/48'/0'/0'/2']"+k.Xpub {
		t.Errorf("text = %s", k.Text)
	}
	if _, err := ParseKeyRecord(byName["key-depth-3-valid"].Record); err != nil {
		t.Errorf("depth-3 key: %v", err)
	}
	if _, err := ParseKeyRecord(byName["key-testnet-tpub-valid"].Record); err != nil {
		t.Errorf("tpub key: %v", err)
	}
	h, err := ParseHashRecord(byName["hash-valid"].Record)
	if err != nil {
		t.Fatal(err)
	}
	for i, b := range h {
		if b != 0xa8 {
			t.Fatalf("digest[%d] = %x", i, b)
		}
	}
	n, err := ParseNowRecord(byName["now-seconds-and-height"].Record)
	if err != nil {
		t.Fatal(err)
	}
	if n.Seconds != 1_756_684_800 || !n.HasHeight || n.Height != 910_000 {
		t.Errorf("now = %+v", n)
	}
	n, err = ParseNowRecord(byName["now-max-both"].Record)
	if err != nil {
		t.Fatal(err)
	}
	if n.Seconds != 2_147_483_647 || n.Height != 499_999_999 {
		t.Errorf("now max = %+v", n)
	}
	n, err = ParseNowRecord(byName["now-seconds-only"].Record)
	if err != nil || n.HasHeight {
		t.Errorf("seconds-only = %+v, %v", n, err)
	}
}

// Classification ORDER: a composer prefix is matched before the constellation
// sniffers, so a record that happens to be BCH-valid or mnemonic-shaped after
// its prefix is never claimed by them; and the three classes are not secret.
func TestComposerClassesArePrefixMatchedAndNotSecret(t *testing.T) {
	for _, c := range []Class{ClassKey, ClassHash, ClassNow} {
		if c.IsSecret() {
			t.Errorf("%v reports secret; key:/hash:/now: are public (SPEC §6a)", c)
		}
	}
	for _, r := range []string{"key", "hash", "now", "Key:00", "KEY:00", "key :00", " key:00"} {
		if got := Classify(r); got == ClassKey || got == ClassHash || got == ClassNow {
			t.Errorf("%q classified as a composer record", r)
		}
		if IsComposerRecord(r) {
			t.Errorf("IsComposerRecord(%q) = true", r)
		}
	}
	if !IsComposerRecord("key:") || !IsComposerRecord("hash:zz") || !IsComposerRecord("now:") {
		t.Error("a prefixed record is ours even when malformed (it is refused, not passed on)")
	}
}

// The path grammar the host accepts: ' and h harden; H, signs and blanks refuse.
func TestKeyRecordPathGrammarMatchesTheHost(t *testing.T) {
	const xpub = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"
	rec := func(origin string) string { return KeyPrefix + hex.EncodeToString([]byte("["+origin+"]"+xpub)) }
	for _, ok := range []string{"73c5da0a/48'/0'/0'/2'", "73c5da0a/48h/0h/0h/2h", "73c5da0a/48'/0h/0'/2h"} {
		if _, err := ParseKeyRecord(rec(ok)); err != nil {
			t.Errorf("%s: %v", ok, err)
		}
	}
	for _, bad := range []string{
		"73c5da0a/48H/0H/0H/2H", "73c5da0a/+48'/0'/0'/2'", "73c5da0a/-48'/0'/0'/2'", "73c5da0a/48'/0'/0'/2'/",
		"73c5da0a/48'/0'//2'", "73c5da0a/ 48'/0'/0'/2'", "73c5da0a/48'/0'/0'/2147483648'", "73C5DA0A/48'/0'/0'/2'",
		"73c5da0/48'/0'/0'/2'", "73c5da0a", "73c5da0a/", "73c5da0a/48'/0'/0'/3'", "73c5da0a/48'/0'/2'",
	} {
		if _, err := ParseKeyRecord(rec(bad)); err == nil {
			t.Errorf("%q accepted", bad)
		}
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run 'TestComposerRecord|TestComposerClasses|TestKeyRecordPath' ./sysw/ 2>&1 | head -4`
Expected: `undefined: ClassKey` (compile failure).

- [ ] **Step 3: Add the classes and the dispatch**

In `sysw/record.go`, append to the `Class` const block after `ClassTx`:

```go
	// ClassKey, ClassHash, ClassNow are the wallet-policy COMPOSER's records
	// (SPEC_wallet_policy_composer.md §6a; SPEC_systemwide_payloads.md 5.3):
	// a cosigner's [fingerprint/path]xpub, a sha256 digest for a hashlock, and
	// the pack time with an optional height. None is secret or bearer. Body
	// rules and prefixes live in composer_records.go, ported as one unit from
	// the host's composer_records.rs and pinned row-for-row by the vendored
	// record_class_vectors.json (§12 item 8). A malformed one is ClassUnknown
	// and the device leaves it inert; the §8n line is the HOST's.
	ClassKey
	ClassHash
	ClassNow
```

In `Classify`, directly before `return classifyConstellation(record)`:

```go
	if IsComposerRecord(record) {
		return classifyComposer(record)
	}
```

Create `sysw/composer_records.go`:

```go
package sysw

// The wallet-policy composer's three payload record classes -- key:, hash:,
// now: -- ported predicate by predicate from the host's
// crates/me-cli/src/sysw/composer_records.rs (SPEC_wallet_policy_composer.md
// §6a) and measured against the vendored record_class_vectors.json (§12 item 8:
// "classifies identically on the host and on the device").
//
// The device parses and classifies; it prints no §8n line (that is host copy)
// and leaves a malformed record inert. The hex rule is this file's own, like
// the host's, so the port stays one unit rather than inheriting record.go's
// history through a shared helper.

import (
	"encoding/hex"
	"errors"
	"strings"
	"unicode/utf8"

	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"seedhammer.com/bip32"
)

const (
	KeyPrefix  = "key:"
	HashPrefix = "hash:"
	NowPrefix  = "now:"

	composerMaxHeight  uint64 = 499_999_999
	composerMaxSeconds uint64 = 2_147_483_647
)

var (
	ErrKeyRecord  = errors.New("sysw: key: needs [fingerprint/path]xpub with an origin")
	ErrHashRecord = errors.New("sysw: hash: must be exactly 64 lowercase hex characters")
	ErrNowRecord  = errors.New("sysw: now: must be <seconds>[,<height>] in range")
)

// KeyRecord is a parsed key: record.
type KeyRecord struct {
	Fingerprint [4]byte
	Origin      bip32.Path
	Xpub        string
	Text        string
}

// NowRecord is a parsed now: record -- a LOWER BOUND on the present that the
// device echoes and never encodes (C24).
type NowRecord struct {
	Seconds   uint32
	Height    uint32
	HasHeight bool
}

// IsComposerRecord reports whether the record carries one of the three
// prefixes, well-formed or not (a malformed one is still OURS: refused, never
// passed to the sniffers).
func IsComposerRecord(record string) bool {
	return strings.HasPrefix(record, KeyPrefix) || strings.HasPrefix(record, HashPrefix) || strings.HasPrefix(record, NowPrefix)
}

func classifyComposer(record string) Class {
	switch {
	case strings.HasPrefix(record, KeyPrefix):
		if _, err := ParseKeyRecord(record); err == nil {
			return ClassKey
		}
	case strings.HasPrefix(record, HashPrefix):
		if _, err := ParseHashRecord(record); err == nil {
			return ClassHash
		}
	case strings.HasPrefix(record, NowPrefix):
		if _, err := ParseNowRecord(record); err == nil {
			return ClassNow
		}
	}
	return ClassUnknown
}

// unhexLower is the host's unhex_lower: even length, every character in
// [0-9a-f]. Uppercase is refused, not folded (§6.6 hashes the wire spelling).
func unhexLower(s string) ([]byte, bool) {
	if len(s)%2 != 0 {
		return nil, false
	}
	for i := 0; i < len(s); i++ {
		c := s[i]
		if !(c >= '0' && c <= '9' || c >= 'a' && c <= 'f') {
			return nil, false
		}
	}
	b, err := hex.DecodeString(s)
	if err != nil {
		return nil, false
	}
	return b, true
}

// ParseHashRecord: exactly 64 lowercase hex characters.
func ParseHashRecord(record string) ([32]byte, error) {
	body, ok := strings.CutPrefix(record, HashPrefix)
	if !ok || len(body) != 64 {
		return [32]byte{}, ErrHashRecord
	}
	b, ok := unhexLower(body)
	if !ok {
		return [32]byte{}, ErrHashRecord
	}
	var h [32]byte
	copy(h[:], b)
	return h, nil
}

// digitsInRange is the host's digits_in_range: ASCII digits only (no sign, no
// blank), at most maxDigits of them, value within [lo, hi].
func digitsInRange(s string, maxDigits int, lo, hi uint64) (uint32, bool) {
	if s == "" || len(s) > maxDigits {
		return 0, false
	}
	var v uint64
	for i := 0; i < len(s); i++ {
		c := s[i]
		if c < '0' || c > '9' {
			return 0, false
		}
		v = v*10 + uint64(c-'0')
	}
	if v < lo || v > hi {
		return 0, false
	}
	return uint32(v), true
}

// ParseNowRecord: hex of "<seconds>[,<height>]", seconds 1..=2^31-1 (10 digits
// at most), height 1..=499,999,999 (9 digits at most).
func ParseNowRecord(record string) (NowRecord, error) {
	body, ok := strings.CutPrefix(record, NowPrefix)
	if !ok {
		return NowRecord{}, ErrNowRecord
	}
	b, ok := unhexLower(body)
	if !ok || !utf8.Valid(b) {
		return NowRecord{}, ErrNowRecord
	}
	text := string(b)
	secText, heightText, hasHeight := strings.Cut(text, ",")
	secs, ok := digitsInRange(secText, 10, 1, composerMaxSeconds)
	if !ok {
		return NowRecord{}, ErrNowRecord
	}
	out := NowRecord{Seconds: secs}
	if hasHeight {
		h, ok := digitsInRange(heightText, 9, 1, composerMaxHeight)
		if !ok {
			return NowRecord{}, ErrNowRecord
		}
		out.Height, out.HasHeight = h, true
	}
	return out, nil
}

// parseOriginPath is the host's DerivationPath::from_str as applied to the
// text between "fp/" and "]": one or more elements, each ASCII digits with an
// optional ' or h hardening marker, no signs, no blanks, no empty element.
func parseOriginPath(s string) (bip32.Path, bool) {
	if s == "" {
		return nil, false
	}
	var out bip32.Path
	for _, el := range strings.Split(s, "/") {
		digits := strings.TrimSuffix(strings.TrimSuffix(el, "'"), "h")
		if len(digits) == len(el) {
			// unhardened: nothing was trimmed
		} else if len(el)-len(digits) != 1 {
			return nil, false
		}
		if digits == "" {
			return nil, false
		}
		for i := 0; i < len(digits); i++ {
			if digits[i] < '0' || digits[i] > '9' {
				return nil, false
			}
		}
		v, err := bip32.ParsePathElement(el)
		if err != nil {
			return nil, false
		}
		out = append(out, v)
	}
	return out, true
}

// ParseKeyRecord: hex of "[<8 lowercase hex fp>/<path>]<xpub>" where the xpub
// is a public extended key of depth 3 or 4, the path has as many components as
// the xpub's depth, and its last component is the xpub's own child number.
// The fingerprint, account and interior components are DECLARATIONS nothing
// here can verify (F-217); the mapping review says so.
func ParseKeyRecord(record string) (KeyRecord, error) {
	body, ok := strings.CutPrefix(record, KeyPrefix)
	if !ok {
		return KeyRecord{}, ErrKeyRecord
	}
	b, ok := unhexLower(body)
	if !ok || !utf8.Valid(b) {
		return KeyRecord{}, ErrKeyRecord
	}
	text := string(b)
	rest, ok := strings.CutPrefix(text, "[")
	if !ok {
		return KeyRecord{}, ErrKeyRecord
	}
	originText, xpubText, ok := strings.Cut(rest, "]")
	if !ok {
		return KeyRecord{}, ErrKeyRecord
	}
	fpText, pathText, ok := strings.Cut(originText, "/")
	if !ok {
		return KeyRecord{}, ErrKeyRecord
	}
	fpBytes, ok := unhexLower(fpText)
	if !ok || len(fpBytes) != 4 {
		return KeyRecord{}, ErrKeyRecord
	}
	origin, ok := parseOriginPath(pathText)
	if !ok {
		return KeyRecord{}, ErrKeyRecord
	}
	ek, err := hdkeychain.NewKeyFromString(xpubText)
	if err != nil || ek.IsPrivate() {
		return KeyRecord{}, ErrKeyRecord
	}
	depth := int(ek.Depth())
	if depth != 3 && depth != 4 {
		return KeyRecord{}, ErrKeyRecord
	}
	if len(origin) != depth {
		return KeyRecord{}, ErrKeyRecord
	}
	if origin[len(origin)-1] != ek.ChildIndex() {
		return KeyRecord{}, ErrKeyRecord
	}
	var fp [4]byte
	copy(fp[:], fpBytes)
	return KeyRecord{Fingerprint: fp, Origin: origin, Xpub: xpubText, Text: text}, nil
}
```

- [ ] **Step 4: Run the tests -- every one of the 40 rows must agree**

Run: `CGO_ENABLED=0 go test -count=1 -run 'TestComposerRecord|TestComposerClasses|TestKeyRecordPath|TestClassify' -v ./sysw/ 2>&1 | grep -E '^(--- |ok|FAIL)'`
Expected: all PASS, including the pre-existing `TestClassifyMatchesTheRustPrimary` and `TestClassifyRejectsMs1RustWouldRefuse`. A row that disagrees is a lockstep defect: quote the row's name, the host's class and Go's, and fix the Go predicate -- unless the fixture row itself looks wrong, in which case STOP (the fix lands in Rust first). `bip32.Path.String()` renders `m/48h/0h/0h/2h` (measured in the gate's scratch run; `bip32/bip32.go:103-110` writes `h`), which is what the assertion pins.

- [ ] **Step 5: gofmt, whole-package run, commit**

Run: `gofmt -l sysw/ && CGO_ENABLED=0 go test -count=1 ./sysw/ 2>&1 | tail -2`
Expected: nothing from gofmt; `ok`.

```bash
git add sysw/record.go sysw/composer_records.go sysw/composer_records_test.go sysw/testdata/record_class_vectors.json sysw/testdata/record_class_vectors.provenance.json
git commit -s -F - <<'MSG'
sysw: key:/hash:/now: record classes, lockstep with the host's 40-row fixture (composer S2 task 6)

Prefix-matched before the sniffers; body rules ported predicate by predicate
from composer_records.rs; fixture vendored with a provenance pin (sha a894e619...).

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task 7: The taproot `3'` origin arm, tied to Multisig Build's table

**Files:**
- Modify: `gui/multisig_build_slots.go:116-130` (doc comment only)
- Test: `gui/composer_origin_test.go`

**Interfaces:**
- Consumes: `multisigScriptTypeComponent(script md.MultisigScript) uint32` (`gui/multisig_build_slots.go:125`); `md.ComposeWrapper.ScriptType()` (Task 2).
- Produces: a test binding the two tables, so a future edit to either moves a test; no behaviour change in `gui/` (Stage 3 calls `md.DefaultOrigin` directly).

- [ ] **Step 1: Write the test**

Create `gui/composer_origin_test.go`:

```go
package gui

import (
	"testing"

	"seedhammer.com/md"
)

// SPEC_wallet_policy_composer.md §4f / §9 item 8: the composer's default
// origins use the same BIP-48 script-type table Multisig Build applies,
// extended by tr = 3'. The two tables live in two packages; this test is the
// tie between them.
func TestComposerOriginTableAgreesWithMultisigBuild(t *testing.T) {
	for _, tc := range []struct {
		script  md.MultisigScript
		wrapper md.ComposeWrapper
	}{
		{md.MultisigWsh, md.ComposeWsh},
		{md.MultisigShWsh, md.ComposeShWsh},
		{md.MultisigSh, md.ComposeSh},
	} {
		if a, b := multisigScriptTypeComponent(tc.script), tc.wrapper.ScriptType(); a != b {
			t.Errorf("script %v: multisig table %d, composer table %d", tc.script, a, b)
		}
	}
	if got := md.ComposeTr.ScriptType(); got != 3 {
		t.Errorf("tr script type = %d, want 3 (BIP-48 has no taproot row; §4f fixes 3')", got)
	}
	// And the whole default origin for a taproot slot at account 0.
	want := []md.PathComponent{{Hardened: true, Value: 48}, {Hardened: true, Value: 0}, {Hardened: true, Value: 0}, {Hardened: true, Value: 3}}
	got := md.DefaultOrigin(md.ComposeTr, 0)
	if len(got) != len(want) {
		t.Fatalf("DefaultOrigin(tr,0) = %+v", got)
	}
	for i := range want {
		if got[i] != want[i] {
			t.Fatalf("DefaultOrigin(tr,0)[%d] = %+v, want %+v", i, got[i], want[i])
		}
	}
}
```

- [ ] **Step 2: Run it (it passes at once -- the arm lives in md; this task's product is the tie)**

Run: `CGO_ENABLED=0 go test -count=1 -run 'TestComposerOriginTableAgreesWithMultisigBuild' ./gui/ 2>&1 | tail -2`
Expected: `ok` (a `-run` filter keeps this to seconds; the whole `gui` package is sharded, not run here).

- [ ] **Step 3: Point the existing comment at the composer's table**

In `gui/multisig_build_slots.go`, after the sentence ending `(buildOriginAnnouncement).` in the comment above `multisigScriptTypeComponent` (line 124), add:

```go
// The wallet-policy composer's md.ComposeWrapper.ScriptType() is the same table
// extended by tr = 3'; gui/composer_origin_test.go keeps the two in step.
```

- [ ] **Step 4: gofmt, commit**

```bash
gofmt -l gui/composer_origin_test.go gui/multisig_build_slots.go
git add gui/composer_origin_test.go gui/multisig_build_slots.go
git commit -s -F - <<'MSG'
gui: tie Multisig Build's script-type table to the composer's (tr = 3') (composer S2 task 7)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task 8: Firmware still builds; size delta recorded

**Files:** none changed. Output goes into the implementation report.

- [ ] **Step 1: Build the firmware the way CI does**

Run: `nix run .#build-firmware 2>&1 | tail -5`
Expected: success. Nothing in this stage is imported by `cmd/controller` yet (Stage 3 wires the GUI), so the linker drops the new code; the measured delta below is expected to be ZERO or near it, and a non-zero delta is worth a sentence in the report (it means something is already reachable).

- [ ] **Step 2: Measure against the baseline**

Run: `tinygo build -size short -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller 2>&1 | tail -4`
Expected: a `code`/`data`/`bss`/`flash`/`RAM` line. Record flash and RAM beside the baseline **1,503,652 B flash / 62,592 B RAM** (fork `169073c`) in the report, with the delta.

---

### Task 9: Whole-repository gates, as CI runs them

**Files:** none changed unless a gate fails.

- [ ] **Step 1: The three packages plus the whole tree, CI's command**

Run: `CGO_ENABLED=0 go test -timeout 20m ./... 2>&1 | grep -vE '^ok|no test files' ; echo "exit=$?"`
Expected: no FAIL lines. (`./gui/` runs here as CI runs it; `scripts/gui-shard-test.sh ./gui/ 24` is the faster local equivalent -- either, but at least one.)

- [ ] **Step 2: 32-bit, the oraclelive build, and the emulator vet**

```bash
scripts/test-32bit.sh 2>&1 | tail -3
CGO_ENABLED=0 go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/ 2>&1 | tail -3
GOOS=js GOARCH=wasm go vet ./cmd/emu/ 2>&1 | tail -3
gofmt -l md/ mk/ sysw/ gui/ scripts/ 2>/dev/null | head
```
Expected: all `ok`; gofmt prints nothing. `md/compose.go` uses `uint32` arithmetic only (no `int` overflow on 32-bit); `sysw/composer_records.go`'s `digitsInRange` works in `uint64`.

- [ ] **Step 3: Report**

Write the implementation report to `design/agent-reports/composer-S2-implementation-report.md` in mnemonic-engrave (the fork is kept clean of design records): per task the fail-then-pass evidence, the Task 3 Step 5 mutation count, the Task 8 sizes, Step 1-2 output above, and every deviation from an Expected line verbatim.

---

## Self-review against the spec and the staged plan

**Spec coverage (STAGED_PLAN §S2 "Delivers"):**
- "a Go tree BUILDER … byte-identical to every S0 vector (the 26 in the corpus, plus the two `no-corpus` keyless-wsh entries … mirrored as Go test cases)" → Task 1 (vendor), Task 2 (`Compose`, `TestComposeReproducesEveryVectorByteForByte`, `noCorpusChunks`).
- "the `pk_h` emitter arm in both contexts with the address-changes mutation test" → Task 3 (wsh arm pinned to Rust's addresses; tap leaf form; coded key-dependence test; hand opcode mutation with the count kept).
- "`md.PolicyShape` split of `or_i`/`or_d`/`andor` into separate branches carrying lock operands and digests" → Task 4.
- "the `3'` origin arm" → Task 2 (`ScriptType`, `DefaultOrigin`) + Task 7 (the tie to Multisig Build's table).
- "`mk.Encode` minting with appended stubs" → Task 5 (`ComposerStubs`, `AppendStubs`, round trip through `Encode`/`Decode`).
- "the device-side §4c lock-range check" → Task 2 (`Lock.Check`, `TestLockCheckIsTheDeviceSideRangeGate`, every boundary in and out per kind — §12 item 7).
- "the `sysw.Classify` half of the three record classes (lockstep with S1's fixture)" → Task 6 (§12 item 8).
- Exit: "`go test ./md/ ./mk/ ./sysw/` green with the S0 vectors and the S1 fixture vendored; TinyGo device build green; flash/RAM delta recorded" → Tasks 8–9.

**What this stage does NOT do (Stage 3/4, named so the reviewer does not look for it):** no screens, no `syswSession` consumption of the three classes ("Keys loaded" / "not understood" counts are §12 item 8's device half at the GUI), no seating through `seatKeyCards` (§12 item 6's card-seating leg needs the GUI's card list), no address derivation for taproot script trees in `address/` (the shipped port's Stage-3 gap; the composer's tr vectors are id-checked by the keyed conformance gate, not address-checked, and `TestPkhWitnessScriptsReproduceRustsAddresses` covers wsh only — said here rather than implied).

**Placeholder scan:** the record-class provenance JSON (Task 6, first step) carries two angle-bracket fields (`commit`, `file_commit`) that the implementer fills from the two named commands; both are checked for 40-character length by the test. No TBD/TODO elsewhere.

**Type consistency:** `ComposeSlot{Index uint8; Path int; Ordinal uint8}` used identically in Tasks 2 and 3 tests; `Branch.LockOperands []uint32` / `Sha256Digests [][32]byte` in Task 4 test and code; `SlotOrigin{Origin []PathComponent; Fingerprint [4]byte; FpPresent bool}` in Task 2 code and tests; `DefaultOrigin(w, account) []PathComponent` consumed by `toComponents` in tests; `KeyRecord.Origin bip32.Path` and `.String()` in Task 6.

**Gate coverage line for the review brief:** `scripts/plan-build-gate-go.sh` compiled and ran Tasks 1 (pin test), 2, 3 (test file), 4 (test file), 5, 6 (new files) with the fragments of `script_emit.go`, `policy_shape.go`, `record.go` hand-wired; the gate does not cover `gui/composer_origin_test.go` (Task 7, run by `-run` filter separately), the firmware build (Task 8) or the vendoring script's provenance output beyond the file count.
