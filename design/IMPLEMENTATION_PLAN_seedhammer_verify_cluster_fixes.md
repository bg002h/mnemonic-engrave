# SeedHammer verify-correctness cluster fixes (Track A) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden the on-device verify-bundle path so a mis-engraved operator mk1 plate FAILS (H1), a correct shuffled multi-chunk md1 PASSES (H2), a non-English `mnem` ms1 with matching entropy FAILS (M1), the multisig success copy is honest (L2), and the two verify-flow `DecodeMS1` validity probes scrub their secret entropy (L1).

**Architecture:** Five surgical fixes in the firmware module `seedhammer.com` (`bundle/` + `gui/`), each test-first (TDD), each with a REAL Old→New diff. The load-bearing GREEN bar is **three new flow-level tests** that route the *production* extraction/gather/decode functions (`extractSuppliedMd1AndMk1` / `md1Gatherer.collected()` / `ms1Entropy`) and were each proven to FAIL on `3a23dbb` and PASS after the fix. No host CLI / `me-preview` / schema / wire-format change. No new program or screen; the only user-visible change is one success-message string (L2).

**Tech Stack:** Go (host build + `go test`/`go vet`); TinyGo is the real device CI gate (deferred to the final task). Fork `bg002h/seedhammer`, branch `main`, HEAD `3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082`. Go on PATH: `export PATH=$PATH:/home/bcg/.local/go/bin`.

**Source-of-truth verification:** Every diff and test in this plan was COMPILE-CHECKED and fail-before/pass-after-PROVEN in a throwaway worktree off `3a23dbb` (the same method the spec R0 reviewer used). The three flow-level tests and their exact error strings are quoted from live probe output, not paraphrased.

---

## R0 corrections folded (from `design/agent-reports/seedhammer-verify-cluster-spec-R0-round0.md`)

- **H1 = option (b):** add a NEW helper `extractSuppliedMd1AndMk1` (modeled on `singleSigReadbackCards`, `gui/singlesig_verify.go:23-42`). Do NOT widen `extractSuppliedMd1` — it has **TWO** callers (`gui/multisig_verify.go:60` AND `gui/multisig.go:71` the live engrave/supply flow). KEEP the `derived` param of `verifyMultisig` (it is the comparator baseline, used like `verifySingleSig`); the bug was the *argument* `reDerived.MK1`, not the parameter.
- **H2 = fix in `gui/md1_gather.go` `collected()`** (index-ordered walk `0..total-1`), NOT `bundle/verify.go`. `collected()` (on `*md1Gatherer`) has **THREE** call sites — `md1_gather.go:76`, `md1_gather.go:140`, AND `gui/bundle.go:234` (`offerChunkedMD1`) — all guarded by `complete()` (no zero-value gaps). The multi-chunk verify path routes through `gui/bundle.go:234` → `bundleCard.strings`, so this fix reaches the real verify path. (Note: `gui/bundle.go:194` is `mk1Gatherer.collected()`, a DIFFERENT type — unaffected.)
- **M1 = compare LANGUAGE in `bundle.Verify`** (`ms1Entropy` also returns the language; `Verify` compares it). Compare on **language** (not raw prefix): language-0 `mnem` ≡ `entr`, so a legitimate English readback is NOT over-rejected (R0 probe-confirmed; `DecodeMS1` returns language 0 for BOTH `entr` and English-`mnem`).
- **L2 = honest success copy** at `gui/multisig_verify.go:104` (operator key + secret verified; other cosigners' keys taken as supplied). Include the `uiContains` regression test so the over-claim cannot silently return.
- **L1 = scrub the TWO verify-flow `DecodeMS1` probe sites** (`gui/singlesig_verify.go:116`, `gui/multisig_verify.go:93`): capture + `wipeBytes()` the probe entropy. Review-assertion only; no observable test. Do NOT touch `gui/codex32_polish.go:103` (Track B).
- **T-H1 must include the decodable-but-wrong companion:** a valid FOREIGN mk1 → FAIL via the stub-binding leg, in addition to the undecodable mutated-mk1 case.

---

## Files touched

| File | Fix | Change |
|------|-----|--------|
| `gui/multisig_supply.go` | H1 | ADD `extractSuppliedMd1AndMk1` helper (leave `extractSuppliedMd1` untouched) |
| `gui/multisig_verify.go` | H1, L1, L2 | call new helper + pass read-back mk1; scrub probe entropy; honest success copy; fix docstring |
| `bundle/verify.go` | M1 | `ms1Entropy` returns `(language int, entropy []byte, err error)`; `Verify` compares language |
| `gui/md1_gather.go` | H2 | `collected()` walks `0..total-1` instead of ranging the map |
| `gui/singlesig_verify.go` | L1 | scrub probe entropy |
| `gui/multisig_supply_test.go` | H1 | ADD `extractSuppliedMd1AndMk1` extraction tests |
| `gui/multisig_verify_test.go` | H1 (T-H1), L2 | ADD flow-level T-H1 + L2 notice-copy test |
| `gui/md1_gather_test.go` | H2 (T-H2) | ADD shuffled-gather index-order + end-to-end tests |
| `bundle/verify_test.go` | M1 (T-M1) | ADD language-mismatch FAIL + no-over-reject PASS; RELABEL `TestVerifyBundleMd1Reordered` |

**Order of tasks (single implementer, strictly serial):** Task 0 (worktree) → Task 1 (M1) → Task 2 (H2) → Task 3 (H1) → Task 4 (L1) → Task 5 (L2) → Task 6 (relabel reordered test) → Task 7 (final pass + cleanup). M1/H2 are independent and small; H1 is the largest; L1/L2 are co-located with H1's file (`gui/multisig_verify.go`) so they follow it.

---

## Verified call-graph facts (live `file:line` @ `3a23dbb`)

- `gui/multisig_verify.go:100` — `verifyMultisig(reDerived, ms1Readback, reDerived.MK1, suppliedMd1)`. 3rd arg is the **re-derived** mk1 (the bug).
- `gui/multisig_verify.go:60` — calls only `extractSuppliedMd1(cards)`; never extracts an mk1.
- `gui/multisig_supply.go:18-34` — `extractSuppliedMd1` returns `ok=false` on any `cardMK1` (`case cardMK1, cardMS1: return nil, false`).
- `gui/multisig_verify.go:93` & `gui/singlesig_verify.go:116` — `if _, _, _, err := codex32.DecodeMS1(s); err != nil { ... }` discard the secret entropy with `_`.
- `bundle/verify.go:122` — `ms1Entropy` does `_, _, entropy, err := codex32.DecodeMS1(str)` (discards prefix+language); `:92` compares `bytes.Equal(dEnt, rEnt)` only.
- `gui/md1_gather.go:57-63` — `collected()` ranges the Go map `g.set`; `md/chunk.go:145` `split()` emits index order; `bundle/verify.go:64,138-148` is positional `equalStrings`.
- Fan-out (verified by grep at implement time, re-run the greps in Task 7 — citations decay):
  - `ms1Entropy` → only `bundle/verify.go:83,87` (both inside `Verify`).
  - `md1Gatherer.collected()` → `gui/md1_gather.go:76,140` + `gui/bundle.go:234` (all `complete()`-guarded).
  - `extractSuppliedMd1` → `gui/multisig_verify.go:60` AND `gui/multisig.go:71` (DO NOT widen).
  - `verifyMultisig` → `gui/multisig_verify.go:100` (the only non-test caller).

---

## Task 0: Worktree + green baseline

**Files:** none (setup only).

- [ ] **Step 1: Create the worktree off `3a23dbb`**

```bash
export PATH=$PATH:/home/bcg/.local/go/bin
cd /scratch/code/shibboleth/seedhammer
git worktree add -b feat/fix-verify-cluster /tmp/seedhammer-verify-cluster 3a23dbb
cd /tmp/seedhammer-verify-cluster
git rev-parse HEAD
```

Expected: `3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082`, on branch `feat/fix-verify-cluster`.

- [ ] **Step 2: Configure commit identity for this worktree**

```bash
cd /tmp/seedhammer-verify-cluster
git config user.name "Brian Goss"
git config user.email "goss.brian@gmail.com"
```

- [ ] **Step 3: Run the baseline suites — must be GREEN before any change**

Run: `cd /tmp/seedhammer-verify-cluster && go test ./gui/... ./bundle/...`
Expected: `ok  seedhammer.com/gui`, `ok  seedhammer.com/bundle` (and `?  ...[no test files]` for sub-packages). NO failures.

---

## Task 1: M1 — compare ms1 codex32 language in `bundle.Verify`

**Files:**
- Test: `bundle/verify_test.go` (ADD two tests)
- Modify: `bundle/verify.go` (`ms1Entropy` signature + `Verify` body)

**Invariant:** A readback ms1 whose recovered entropy matches the derived ms1 but whose BIP-39 language byte differs makes `Verify` FAIL. Identical entropy under a different wordlist is a different wallet.

- [ ] **Step 1: Write the failing tests (T-M1 FAIL + no-over-reject PASS)**

Append to `bundle/verify_test.go`:

```go
// T-M1 (verify-cluster M1): a readback ms1 whose recovered entropy MATCHES the
// derived ms1 but whose BIP-39 language byte DIFFERS (Japanese mnem, lang 1, vs
// the derived English entr, lang 0) must FAIL — identical entropy under a
// different wordlist is a different wallet. The fixture is built directly because
// EncodeMS1 only emits entr/English; this is a hand-typed-readback-only string.
// Proven on 3a23dbb: Verify PASSES this (the M1 bug). After the language compare
// it FAILS with "verify: ms1 wordlist/language mismatch".
func TestVerifyBundleLanguageMismatch(t *testing.T) {
	derived := correctBundle() // entr / English, entropy = zero-16
	readback := correctBundle()
	// codex32.NewSeed("ms",0,"entr",'s',[]byte{0x02,0x01,<zero16>}) — a valid
	// language-1 (Japanese) mnem ms1 with the SAME zero-16 entropy as wpkhMS1.
	// (Verified: decodes to prefix=2/mnem, language=1, entropy=00..00.)
	readback.MS1 = "ms10entrsqgqsqqqqqqqqqqqqqqqqqqqqqqqqqj9tawneveyd9j"
	err := Verify(derived, readback)
	if err == nil {
		t.Fatal("language-differ readback (same entropy) accepted, want FAIL")
	}
	if !strings.Contains(err.Error(), "language") && !strings.Contains(err.Error(), "wordlist") {
		t.Errorf("error %q does not name language/wordlist", err)
	}
}

// TestVerifyBundleLanguageEnglishNotOverRejected: a legitimate English/entr
// readback (language 0) against an English/entr derived (language 0) must still
// PASS — the language compare must not over-reject identical-wordlist readbacks.
func TestVerifyBundleLanguageEnglishNotOverRejected(t *testing.T) {
	if err := Verify(correctBundle(), correctBundle()); err != nil {
		t.Fatalf("English/entr readback over-rejected: %v (want PASS)", err)
	}
}
```

`strings` is already imported in `bundle/verify_test.go:4`.

- [ ] **Step 2: Run the tests — verify T-M1 FAILS, no-over-reject PASSES**

Run: `cd /tmp/seedhammer-verify-cluster && go test ./bundle/ -run 'TestVerifyBundleLanguage' -v`
Expected:
- `TestVerifyBundleLanguageMismatch` → **FAIL** with `language-differ readback (same entropy) accepted, want FAIL` (on `3a23dbb` `Verify` returns nil for the language-differ case — the M1 bug).
- `TestVerifyBundleLanguageEnglishNotOverRejected` → PASS.

- [ ] **Step 3: Apply the M1 fix**

In `bundle/verify.go`, change `ms1Entropy` to also return the language.

OLD (`bundle/verify.go:122-136`):
```go
func ms1Entropy(s string) ([]byte, error) {
	str, err := codex32.New(s)
	if err != nil {
		return nil, err
	}
	_, _, entropy, err := codex32.DecodeMS1(str)
	if err != nil {
		return nil, err
	}
	// Copy so the caller owns a scrubbable buffer independent of the decoder.
	out := make([]byte, len(entropy))
	copy(out, entropy)
	wipe(entropy)
	return out, nil
}
```

NEW:
```go
func ms1Entropy(s string) (language int, entropy []byte, err error) {
	str, err := codex32.New(s)
	if err != nil {
		return 0, nil, err
	}
	_, language, ent, err := codex32.DecodeMS1(str)
	if err != nil {
		return 0, nil, err
	}
	// Copy so the caller owns a scrubbable buffer independent of the decoder.
	out := make([]byte, len(ent))
	copy(out, ent)
	wipe(ent)
	return language, out, nil
}
```

Also update the `ms1Entropy` docstring (`bundle/verify.go:120-121`).

OLD:
```go
// ms1Entropy decodes an ms1 secret string to its recovered BIP-39 entropy. The
// returned slice is SECRET; Verify scrubs it after the compare.
```

NEW:
```go
// ms1Entropy decodes an ms1 secret string to its recovered BIP-39 entropy and
// its codex32 language byte (0=entr/English; 1..9=a non-English mnem wordlist).
// The returned slice is SECRET; Verify scrubs it after the compare.
```

Then update the two call sites + add the language compare in `Verify`.

OLD (`bundle/verify.go:83-98`):
```go
	dEnt, err := ms1Entropy(derived.MS1)
	if err != nil {
		return fmt.Errorf("verify: derived ms1: %w", err)
	}
	rEnt, err := ms1Entropy(readback.MS1)
	if err != nil {
		wipe(dEnt)
		return fmt.Errorf("verify: readback ms1: %w", err)
	}
	match := bytes.Equal(dEnt, rEnt)
	wipe(dEnt)
	wipe(rEnt)
	if !match {
		return errors.New("verify: ms1 entropy mismatch")
	}
	return nil
```

NEW:
```go
	dLang, dEnt, err := ms1Entropy(derived.MS1)
	if err != nil {
		return fmt.Errorf("verify: derived ms1: %w", err)
	}
	rLang, rEnt, err := ms1Entropy(readback.MS1)
	if err != nil {
		wipe(dEnt)
		return fmt.Errorf("verify: readback ms1: %w", err)
	}
	match := bytes.Equal(dEnt, rEnt)
	wipe(dEnt)
	wipe(rEnt)
	if !match {
		return errors.New("verify: ms1 entropy mismatch")
	}
	// Compare the BIP-39 wordlist LANGUAGE, not just the entropy: identical
	// entropy under a different wordlist yields different mnemonic words → a
	// different PBKDF2 seed → a different wallet. Compare on language (not raw
	// prefix) so a legitimate English readback (entr OR English-mnem, both
	// language 0) is not over-rejected on an incidental prefix difference.
	if dLang != rLang {
		return errors.New("verify: ms1 wordlist/language mismatch")
	}
	return nil
```

- [ ] **Step 4: Run the tests — verify they PASS and the full bundle suite is green**

Run: `cd /tmp/seedhammer-verify-cluster && go test ./bundle/ -run 'TestVerifyBundleLanguage' -v && go test ./bundle/`
Expected: both `TestVerifyBundleLanguage*` PASS (`TestVerifyBundleLanguageMismatch` now fails with `verify: ms1 wordlist/language mismatch`); `ok  seedhammer.com/bundle` for the full suite.

- [ ] **Step 5: Commit**

```bash
cd /tmp/seedhammer-verify-cluster
git add bundle/verify.go bundle/verify_test.go
git commit -S -s -m "$(cat <<'EOF'
fix(bundle): compare ms1 codex32 language in Verify (M1)

A readback ms1 whose recovered entropy matched the derived ms1 but whose
BIP-39 language byte differed (a non-English mnem with identical entropy)
falsely PASSED verify. ms1Entropy now also returns the language byte and
Verify compares it (on language, not raw prefix, so a legitimate English
readback is not over-rejected). Adds T-M1 (language-differ FAIL) and a
no-over-reject PASS test.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: H2 — `md1Gatherer.collected()` returns chunks in `ChunkIndex` order

**Files:**
- Test: `gui/md1_gather_test.go` (ADD two tests)
- Modify: `gui/md1_gather.go` (`collected()` body only — no signature change)

**Invariant:** `md1Gatherer.collected()` returns chunk strings in ascending `ChunkIndex` order (0..total-1), deterministically, regardless of scan/arrival order — so a correctly-engraved multi-chunk md1 scanned in any order compares equal to the index-ordered derived md1 and verify PASSES.

- [ ] **Step 1: Write the failing tests (T-H2 index-order + end-to-end)**

Append to `gui/md1_gather_test.go`:

```go
// T-H2 (verify-cluster H2): collected() must return chunks in ChunkIndex order
// regardless of arrival order. The gatherer keys by parsed ChunkIndex, so we
// vary ARRIVAL order; the canonical wshSortedmultiChunks slice IS index-ordered.
// Proven on 3a23dbb: collected() ranges the Go map (random order) → non-index
// order on 10/10 shuffled trials (FALSE-FAIL at the positional comparator).
// After the index-walk fix it is index-ordered deterministically every time.
func TestMD1GathererCollectedIndexOrder(t *testing.T) {
	orders := [][]int{
		{5, 0, 3, 1, 4, 2},
		{2, 1, 0, 5, 4, 3},
		{0, 1, 2, 3, 4, 5},
		{3, 5, 1, 0, 2, 4},
	}
	for _, order := range orders {
		// Repeat to defeat Go's randomized map iteration (a single run could
		// coincidentally agree; 10 runs makes a map-order regression observable).
		for trial := 0; trial < 10; trial++ {
			g := &md1Gatherer{}
			for _, i := range order {
				if st := g.offer(wshSortedmultiChunks[i]); st != gatherAdded {
					t.Fatalf("order %v: offer chunk %d status %v", order, i, st)
				}
			}
			if !g.complete() {
				t.Fatalf("order %v: not complete", order)
			}
			got := g.collected()
			if len(got) != len(wshSortedmultiChunks) {
				t.Fatalf("order %v: collected len %d, want %d", order, len(got), len(wshSortedmultiChunks))
			}
			for i := range wshSortedmultiChunks {
				if got[i] != wshSortedmultiChunks[i] {
					t.Fatalf("order %v trial %d: collected()[%d]=%q, want index order %q",
						order, trial, i, got[i], wshSortedmultiChunks[i])
				}
			}
		}
	}
}

// TestMD1GathererShuffledGatherExpands (end-to-end flavour): a complete
// multi-chunk md1 gathered in shuffled order must reassemble + expand the SAME
// descriptor as the canonical index-ordered set (collected() → the production
// gather-completion consumer), confirming the ordering fix reaches the real
// gather→consume path, not just collected() in isolation.
func TestMD1GathererShuffledGatherExpands(t *testing.T) {
	g := &md1Gatherer{}
	for _, i := range []int{5, 0, 3, 1, 4, 2} {
		g.offer(wshSortedmultiChunks[i])
	}
	if !g.complete() {
		t.Fatal("not complete after shuffled gather")
	}
	tpl, keys, err := md.ExpandWalletPolicyChunks(g.collected())
	if err != nil {
		t.Fatalf("expand shuffled-gather collected(): %v", err)
	}
	tplC, keysC, err := md.ExpandWalletPolicyChunks(wshSortedmultiChunks)
	if err != nil {
		t.Fatalf("expand canonical: %v", err)
	}
	if tpl.Root != tplC.Root || tpl.Policy != tplC.Policy || tpl.K != tplC.K || tpl.N != tplC.N {
		t.Fatalf("shuffled-gather template %v/%v/%d-of-%d != canonical %v/%v/%d-of-%d",
			tpl.Root, tpl.Policy, tpl.K, tpl.N, tplC.Root, tplC.Policy, tplC.K, tplC.N)
	}
	if len(keys) != len(keysC) {
		t.Fatalf("shuffled-gather %d keys != canonical %d", len(keys), len(keysC))
	}
}
```

`md` is already imported in `gui/md1_gather_test.go` via `seedhammer.com/md`? Confirm: `gui/md1_gather_test.go:9` imports `"seedhammer.com/md"` — **it does NOT today** (the existing file imports only `os`, `path/filepath`, `strings`, `testing`). ADD the import. Change the import block at `gui/md1_gather_test.go:3-8`:

OLD:
```go
import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)
```

NEW:
```go
import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"seedhammer.com/md"
)
```

- [ ] **Step 2: Run the tests — verify T-H2 FAILS**

Run: `cd /tmp/seedhammer-verify-cluster && go test ./gui/ -run 'TestMD1GathererCollectedIndexOrder|TestMD1GathererShuffledGatherExpands' -count=1 -v`
Expected: `TestMD1GathererCollectedIndexOrder` → **FAIL** with `collected()[...]=..., want index order ...` (map-range order ≠ index order). `TestMD1GathererShuffledGatherExpands` PASSES even before the fix (reassembly is order-tolerant) — it is the end-to-end guard, not the discriminator; the discriminator is `TestMD1GathererCollectedIndexOrder`.

- [ ] **Step 3: Apply the H2 fix**

In `gui/md1_gather.go`, replace the map-range walk with an index-ordered walk.

OLD (`gui/md1_gather.go:57-63`):
```go
func (g *md1Gatherer) collected() []string {
	out := make([]string, 0, len(g.set))
	for _, s := range g.set {
		out = append(out, s)
	}
	return out
}
```

NEW:
```go
// collected returns the gathered chunk strings in ascending ChunkIndex order
// (0..total-1), deterministically — NEVER Go's randomized map-iteration order.
// The deterministic comparator (bundle.Verify) compares md1 positionally against
// the index-ordered derived side (md.split emits index order), so a random
// readback order would FALSE-FAIL a correct backup. collected() is only ever
// called after complete() (md1_gather.go:76,140; bundle.go:234), which requires
// every index 0..total-1 present, so each lookup is populated (no "" gaps).
func (g *md1Gatherer) collected() []string {
	out := make([]string, 0, len(g.set))
	for i := 0; i < g.total; i++ {
		out = append(out, g.set[i])
	}
	return out
}
```

- [ ] **Step 4: Run the tests — verify they PASS and the full gui suite is green**

Run: `cd /tmp/seedhammer-verify-cluster && go test ./gui/ -run 'TestMD1GathererCollectedIndexOrder|TestMD1GathererShuffledGatherExpands' -count=1 -v && go test ./gui/`
Expected: both new tests PASS; `ok  seedhammer.com/gui` for the full suite (confirms the 3rd consumer `bundle.go:234` is unaffected by the order change).

- [ ] **Step 5: Commit**

```bash
cd /tmp/seedhammer-verify-cluster
git add gui/md1_gather.go gui/md1_gather_test.go
git commit -S -s -m "$(cat <<'EOF'
fix(gui): md1Gatherer.collected() returns chunks in index order (H2)

collected() ranged the Go map g.set in randomized iteration order, while
the derived md1 (md.split) is index-ordered and bundle.Verify compares
positionally — so a correct multi-chunk md1 scanned in any order
FALSE-FAILED verify (<=1/6 accidental agreement for a 6-chunk set). Walk
0..total-1 instead (all indices present once complete(), the only state
collected() is called in). Adds T-H2 (index-order across arrival orders)
+ an end-to-end shuffled-gather expand test.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: H1 — read back the operator mk1 plate and compare the REAL readback

**Files:**
- Modify: `gui/multisig_supply.go` (ADD `extractSuppliedMd1AndMk1`)
- Test: `gui/multisig_supply_test.go` (ADD extraction tests)
- Test: `gui/multisig_verify_test.go` (ADD flow-level T-H1)
- Modify: `gui/multisig_verify.go` (call new helper; pass read-back mk1; fix docstring + error copy)

**Invariant:** A mis-engraved operator mk1 plate (wrong/undecodable bytes OR a valid foreign card that binds to a different policy) makes multisig verify FAIL. The mk1 legs of `bundle.Verify` compare an NFC-read-back mk1 against the re-derived mk1 — never the re-derived value against itself.

- [ ] **Step 1: Write the failing extraction tests (the helper does not exist yet)**

Append to `gui/multisig_supply_test.go`:

```go
// extractSuppliedMd1AndMk1 (verify-cluster H1): reads back BOTH the operator mk1
// AND the wallet-policy md1 from the gathered card set. Exactly one of each, else
// ok=false. (extractSuppliedMd1 is the SUPPLY filter — one md1, zero key cards —
// and must stay unchanged: it has two callers including the live engrave flow.)
func TestExtractSuppliedMd1AndMk1(t *testing.T) {
	mk1 := bundleCard{kind: cardMK1, strings: []string{"mk1a", "mk1b"}}
	md1 := bundleCard{kind: cardMD1, strings: []string{"md1a", "md1b", "md1c"}}

	t.Run("one mk1 + one md1 → ok", func(t *testing.T) {
		gotMd1, gotMk1, ok := extractSuppliedMd1AndMk1([]bundleCard{mk1, md1})
		if !ok {
			t.Fatal("valid mk1+md1 set rejected")
		}
		if !equalStringSlice(gotMd1, md1.strings) {
			t.Fatalf("md1 = %v, want %v", gotMd1, md1.strings)
		}
		if !equalStringSlice(gotMk1, mk1.strings) {
			t.Fatalf("mk1 = %v, want %v", gotMk1, mk1.strings)
		}
	})
	t.Run("missing mk1 → not ok", func(t *testing.T) {
		if _, _, ok := extractSuppliedMd1AndMk1([]bundleCard{md1}); ok {
			t.Fatal("set with no mk1 accepted")
		}
	})
	t.Run("missing md1 → not ok", func(t *testing.T) {
		if _, _, ok := extractSuppliedMd1AndMk1([]bundleCard{mk1}); ok {
			t.Fatal("set with no md1 accepted")
		}
	})
	t.Run("two mk1 → ambiguous, not ok", func(t *testing.T) {
		if _, _, ok := extractSuppliedMd1AndMk1([]bundleCard{mk1, mk1, md1}); ok {
			t.Fatal("ambiguous (two mk1) set accepted")
		}
	})
	t.Run("two md1 → ambiguous, not ok", func(t *testing.T) {
		if _, _, ok := extractSuppliedMd1AndMk1([]bundleCard{mk1, md1, md1}); ok {
			t.Fatal("ambiguous (two md1) set accepted")
		}
	})
	t.Run("stray ms1 → not ok", func(t *testing.T) {
		ms1 := bundleCard{kind: cardMS1, strings: []string{"ms1x"}}
		if _, _, ok := extractSuppliedMd1AndMk1([]bundleCard{mk1, md1, ms1}); ok {
			t.Fatal("set with a stray ms1 accepted")
		}
	})
}
```

`equalStringSlice` is defined in `gui/singlesig_engrave_test.go:141` (same package) and is already used by `gui/singlesig_verify_test.go`. `testing` is already imported in `gui/multisig_supply_test.go:7`.

- [ ] **Step 2: Run — verify it FAILS to compile (helper undefined)**

Run: `cd /tmp/seedhammer-verify-cluster && go test ./gui/ -run 'TestExtractSuppliedMd1AndMk1' -v 2>&1 | head -20`
Expected: build failure `undefined: extractSuppliedMd1AndMk1`.

- [ ] **Step 3: Add the helper to `gui/multisig_supply.go`**

Insert ABOVE the `allSlotsHaveXpub` comment block (which currently starts at `gui/multisig_supply.go:36`, `// allSlotsHaveXpub is the full-policy gate (I-3):`):

```go
// extractSuppliedMd1AndMk1 reads back BOTH the operator mk1 key card AND the
// wallet-policy md1 from the gathered card set (H1). It requires EXACTLY one of
// each; ok=false on a missing card, a duplicate (>=2 of either), or any stray
// cardMS1. Modeled on singleSigReadbackCards (gui/singlesig_verify.go:23). The
// read-back mk1 is the operator's ENGRAVED plate (compared against the
// re-derived mk1 inside verifyMultisig) — NOT a re-derived value. This is a
// distinct helper from extractSuppliedMd1 (the supply/engrave filter, which
// refuses any key card); do NOT widen that one — it has a second caller in the
// live engrave flow (gui/multisig.go:71).
func extractSuppliedMd1AndMk1(cards []bundleCard) (md1, mk1 []string, ok bool) {
	for _, c := range cards {
		switch c.kind {
		case cardMD1:
			if md1 != nil {
				return nil, nil, false // more than one descriptor — ambiguous
			}
			md1 = c.strings
		case cardMK1:
			if mk1 != nil {
				return nil, nil, false // more than one key card — ambiguous
			}
			mk1 = c.strings
		case cardMS1:
			return nil, nil, false // a stray secret card pollutes the supply.
		}
	}
	if len(md1) == 0 || len(mk1) == 0 {
		return nil, nil, false
	}
	return md1, mk1, true
}

```

- [ ] **Step 4: Run — verify the extraction tests PASS**

Run: `cd /tmp/seedhammer-verify-cluster && go test ./gui/ -run 'TestExtractSuppliedMd1AndMk1' -v`
Expected: all subtests PASS.

- [ ] **Step 5: Write the flow-level T-H1 test**

Append to `gui/multisig_verify_test.go`:

```go
// TestVerifyMultisigReadbackMk1 (T-H1, verify-cluster H1): route a readback
// []bundleCard through the PRODUCTION extractSuppliedMd1AndMk1 → verifyMultisig.
// On 3a23dbb the flow ignored the readback mk1 (passed reDerived.MK1 on both
// sides), so a WRONG engraved mk1 plate silently PASSED. This test routes the
// real readback mk1 and asserts:
//   - correct mk1 → PASS
//   - undecodable mutated mk1 → FAIL (mk1-decode leg)
//   - decodable-but-WRONG foreign mk1 (valid card, different policy) → FAIL
//     (stub-binding leg: "verify: readback mk1/md1 stub mismatch ...")
//   - masking proof: feeding reDerived.MK1 (today's self-compare) PASSES the
//     wrong-plate case — the discrimination the production flow lacked.
func TestVerifyMultisigReadbackMk1(t *testing.T) {
	chunks := suppliedMultisigMd1(t)
	_, keys, err := md.ExpandWalletPolicyChunks(chunks)
	if err != nil {
		t.Fatalf("ExpandWalletPolicyChunks: %v", err)
	}
	m := abandonAboutMnemonic()
	_, origin, _, ok := findUserSlot(m, "", &chaincfg.MainNetParams, keys)
	if !ok {
		t.Fatal("findUserSlot: no match")
	}
	derived, err := deriveMultisigLeg(m, "", &chaincfg.MainNetParams, origin, chunks, true)
	if err != nil {
		t.Fatalf("deriveMultisigLeg: %v", err)
	}

	// A foreign-but-VALID operator mk1: a different single-sig wallet's mk1 from
	// the SAME seed — it decodes fine (real card) but binds to a different policy
	// stub, so the stub-binding leg must reject it.
	foreign, _, _, _, err := deriveSingleSigBundle(m, "", &chaincfg.MainNetParams, singleSigPath(44), md.ScriptPkh)
	if err != nil {
		t.Fatalf("derive foreign mk1: %v", err)
	}

	t.Run("correct readback mk1 → PASS", func(t *testing.T) {
		cards := []bundleCard{
			{kind: cardMK1, strings: append([]string(nil), derived.MK1...)},
			{kind: cardMD1, strings: append([]string(nil), derived.MD1...)},
		}
		md1RB, mk1RB, ok := extractSuppliedMd1AndMk1(cards)
		if !ok {
			t.Fatal("helper rejected a valid mk1+md1 card set")
		}
		if err := verifyMultisig(derived, derived.MS1, mk1RB, md1RB); err != nil {
			t.Fatalf("correct readback: %v (want PASS)", err)
		}
	})

	t.Run("undecodable mutated mk1 → FAIL", func(t *testing.T) {
		mutated := append([]string(nil), derived.MK1...)
		mutated[len(mutated)-1] = "mk1tampered000000000000000000000000000000000000"
		cards := []bundleCard{
			{kind: cardMK1, strings: mutated},
			{kind: cardMD1, strings: append([]string(nil), derived.MD1...)},
		}
		md1RB, mk1RB, ok := extractSuppliedMd1AndMk1(cards)
		if !ok {
			t.Fatal("helper rejected mutated mk1 card set")
		}
		if err := verifyMultisig(derived, derived.MS1, mk1RB, md1RB); err == nil {
			t.Fatal("undecodable mk1 accepted, want FAIL")
		}
	})

	t.Run("decodable-but-wrong foreign mk1 → FAIL via stub binding", func(t *testing.T) {
		cards := []bundleCard{
			{kind: cardMK1, strings: append([]string(nil), foreign.MK1...)},
			{kind: cardMD1, strings: append([]string(nil), derived.MD1...)},
		}
		md1RB, mk1RB, ok := extractSuppliedMd1AndMk1(cards)
		if !ok {
			t.Fatal("helper rejected foreign mk1 card set")
		}
		err := verifyMultisig(derived, derived.MS1, mk1RB, md1RB)
		if err == nil {
			t.Fatal("decodable-but-wrong foreign mk1 accepted, want FAIL")
		}
		if !strings.Contains(err.Error(), "stub mismatch") {
			t.Errorf("foreign mk1 error %q does not name stub mismatch", err)
		}
	})

	t.Run("masking proof: self-compare PASSES the foreign mk1 (the bug)", func(t *testing.T) {
		// Today's flow passed reDerived.MK1 on the readback side, so the engraved
		// (here: foreign) plate was never compared — it PASSES. This is the bug
		// the production fix (routing the real readback mk1) closes.
		if err := verifyMultisig(derived, derived.MS1, derived.MK1, derived.MD1); err != nil {
			t.Fatalf("self-compare baseline: %v (want PASS, demonstrating the masked bug)", err)
		}
	})
}
```

Add `"strings"` to the `gui/multisig_verify_test.go` import block. OLD (`gui/multisig_verify_test.go:3-8`):
```go
import (
	"testing"

	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/md"
)
```
NEW:
```go
import (
	"strings"
	"testing"

	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/md"
)
```

- [ ] **Step 6: Run T-H1 — verify the discrimination holds**

Run: `cd /tmp/seedhammer-verify-cluster && go test ./gui/ -run 'TestVerifyMultisigReadbackMk1' -v`
Expected (this test routes the new helper, so it compiles only after Step 3): all four subtests PASS. The discriminating subtests are `undecodable mutated mk1 → FAIL` (error `verify: readback mk1 decode: codex32: not a valid mk1 string`) and `decodable-but-wrong foreign mk1 → FAIL via stub binding` (error `verify: readback mk1/md1 stub mismatch (key card does not bind to this policy)`). The `masking proof` subtest PASSES, documenting that the OLD self-compare wiring would have accepted the wrong plate.

- [ ] **Step 7: Wire the production flow to use the read-back mk1**

In `gui/multisig_verify.go`, change the extraction call.

OLD (`gui/multisig_verify.go:60-64`):
```go
	suppliedMd1, ok := extractSuppliedMd1(cards)
	if !ok {
		showError(ctx, th, "Verify Bundle", "Read back exactly one wallet-policy md1 (and no key cards).")
		return
	}
```

NEW:
```go
	suppliedMd1, suppliedMk1, ok := extractSuppliedMd1AndMk1(cards)
	if !ok {
		showError(ctx, th, "Verify Bundle", "Read back one wallet-policy md1 AND the operator key card (mk1).")
		return
	}
```

OLD (`gui/multisig_verify.go:100`):
```go
	if err := verifyMultisig(reDerived, ms1Readback, reDerived.MK1, suppliedMd1); err != nil {
```

NEW:
```go
	if err := verifyMultisig(reDerived, ms1Readback, suppliedMk1, suppliedMd1); err != nil {
```

Then fix the now-true flow docstring. OLD (`gui/multisig_verify.go:31-35`):
```go
// multisigVerifyFlow drives the on-device verify-bundle for the multisig flow:
// re-type the seed (fresh residency), gather the supplied md1 + operator mk1
// over NFC, re-cross-match to recover the operator's origin, re-derive the leg,
// hand-type the ms1 (full only; never NFC), and report PASS/FAIL. `full` reports
// whether an ms1 was engraved (and so must be hand-typed for the verify).
```
NEW:
```go
// multisigVerifyFlow drives the on-device verify-bundle for the multisig flow:
// re-type the seed (fresh residency), gather the supplied md1 + the operator's
// engraved mk1 plate over NFC (extractSuppliedMd1AndMk1), re-cross-match to
// recover the operator's origin, re-derive the leg, hand-type the ms1 (full
// only; never NFC), and report PASS/FAIL — comparing the READ-BACK mk1 against
// the re-derived mk1 (H1: never the re-derived value against itself). `full`
// reports whether an ms1 was engraved (and so must be hand-typed for verify).
```

- [ ] **Step 8: Run the full gui suite — production wiring + all tests green**

Run: `cd /tmp/seedhammer-verify-cluster && go test ./gui/`
Expected: `ok  seedhammer.com/gui`. (The pre-existing `TestVerifyMultisig` still passes — it calls `verifyMultisig` directly with explicit args, unaffected.)

- [ ] **Step 9: Commit**

```bash
cd /tmp/seedhammer-verify-cluster
git add gui/multisig_supply.go gui/multisig_supply_test.go gui/multisig_verify.go gui/multisig_verify_test.go
git commit -S -s -m "$(cat <<'EOF'
fix(gui): multisig verify reads back the operator mk1 plate (H1)

multisigVerifyFlow passed reDerived.MK1 as the readback mk1, comparing the
re-derived key card against itself — so a mis-engraved operator mk1 plate
silently PASSED verify. Add extractSuppliedMd1AndMk1 (modeled on
singleSigReadbackCards) to read back BOTH the md1 and the operator mk1 over
NFC, and pass the real read-back mk1 into verifyMultisig.
extractSuppliedMd1 is left untouched (it has a second caller in the engrave
flow). Adds T-H1 routing the production helper: correct mk1 PASS,
undecodable mk1 FAIL (decode leg), decodable-but-wrong foreign mk1 FAIL
(stub-binding leg), plus a masking-proof subtest.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: L1 — scrub the two verify-flow `DecodeMS1` probe-entropy sites

**Files:**
- Modify: `gui/singlesig_verify.go:116-120`
- Modify: `gui/multisig_verify.go:93-97`

**Invariant:** Each verify flow's ms1 validity probe captures and `wipeBytes()`-scrubs the secret entropy `DecodeMS1` returns, matching the codebase convention (`gui/ms1_decode.go:29`, `bundle/verify.go:131-134`). Review-assertion only — no observable test (best-effort scrub is not observable post-GC).

- [ ] **Step 1: Apply the scrub at the single-sig site**

OLD (`gui/singlesig_verify.go:116-120`):
```go
		if _, _, _, err := codex32.DecodeMS1(s); err != nil {
			showError(ctx, th, "Verify Bundle", "That isn't a valid ms1 secret share.")
			return
		}
		ms1Readback = s.String()
```

NEW:
```go
		// L1: capture + scrub the probe's secret entropy (codebase convention,
		// gui/ms1_decode.go:29) — DecodeMS1 allocates a fresh entropy slice we
		// otherwise abandon to the GC.
		_, _, ent, err := codex32.DecodeMS1(s)
		if err != nil {
			showError(ctx, th, "Verify Bundle", "That isn't a valid ms1 secret share.")
			return
		}
		wipeBytes(ent)
		ms1Readback = s.String()
```

- [ ] **Step 2: Apply the scrub at the multisig site**

OLD (`gui/multisig_verify.go:93-97`):
```go
		if _, _, _, err := codex32.DecodeMS1(s); err != nil {
			showError(ctx, th, "Verify Bundle", "That isn't a valid ms1 secret share.")
			return
		}
		ms1Readback = s.String()
```

NEW:
```go
		// L1: capture + scrub the probe's secret entropy (codebase convention,
		// gui/ms1_decode.go:29) — DecodeMS1 allocates a fresh entropy slice we
		// otherwise abandon to the GC.
		_, _, ent, err := codex32.DecodeMS1(s)
		if err != nil {
			showError(ctx, th, "Verify Bundle", "That isn't a valid ms1 secret share.")
			return
		}
		wipeBytes(ent)
		ms1Readback = s.String()
```

- [ ] **Step 3: Build + run the gui suite (no behavior change; confirm no regression)**

Run: `cd /tmp/seedhammer-verify-cluster && go vet ./gui/ && go test ./gui/`
Expected: clean `go vet` (in particular no `err` shadowing/unused complaints) and `ok  seedhammer.com/gui`.

> Verifier note: `wipeBytes` is `gui/slip39_polish.go:344` (a plain zeroing loop) — same package, no import needed. Do NOT touch `gui/codex32_polish.go:103` (the third L1 site — Track B owns it).

- [ ] **Step 4: Commit**

```bash
cd /tmp/seedhammer-verify-cluster
git add gui/singlesig_verify.go gui/multisig_verify.go
git commit -S -s -m "$(cat <<'EOF'
fix(gui): scrub DecodeMS1 probe entropy in the two verify flows (L1)

The ms1 validity probe in singleSigVerifyFlow and multisigVerifyFlow
discarded DecodeMS1's freshly-allocated secret entropy with _, abandoning
it unscrubbed. Capture and wipeBytes() it, matching the codebase
convention (ms1_decode.go:29, bundle/verify.go). Defence-in-depth
consistency fix; the third site (codex32_polish.go:103) is Track B.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: L2 — honest multisig verify success copy

**Files:**
- Modify: `gui/multisig_verify.go:104`
- Test: `gui/multisig_verify_test.go` (ADD notice-copy regression test)

**Invariant:** The multisig "Verify OK" message must not claim a guarantee the air-gapped device cannot provide. The honest scope: operator key + secret verified; the wallet policy / other cosigners' public keys are taken as supplied (no source of truth for foreign xpubs). After H1 the operator-mk1 leg is real and the M1 entropy/language leg is real, but the md1 leg remains `clone(suppliedMd1)` vs `suppliedMd1` (`gui/multisig_derive.go:60`) and foreign cosigner xpubs are inherently unverifiable.

- [ ] **Step 1: Write the failing notice-copy test**

Append to `gui/multisig_verify_test.go`:

```go
// TestMultisigVerifyNoticeIsHonest (L2): the multisig success notice must scope
// its guarantee honestly — "operator key + secret verified; other cosigners'
// keys taken as supplied" — and must NOT carry the bare full-bundle over-claim
// ("the engraved bundle matches the seed"). We drive showNotice directly with the
// production copy and assert the rendered text via uiContains (which strips
// spaces). This guards against the over-claim silently returning.
func TestMultisigVerifyNoticeIsHonest(t *testing.T) {
	ctx := NewContext(newPlatform())
	frame, quit := runUI(ctx, func() {
		showNotice(ctx, &descriptorTheme, multisigVerifyOKTitle, multisigVerifyOKBody)
	})
	defer quit()
	content, ok := frame()
	if !ok {
		t.Fatal("no frame from showNotice")
	}
	if !uiContains(content, "taken as supplied") {
		t.Errorf("notice lacks the scoped wording; got %q", content)
	}
	if uiContains(content, "matches the seed") {
		t.Errorf("notice still carries the over-claim; got %q", content)
	}
}
```

This references two new package-level string constants (`multisigVerifyOKTitle`, `multisigVerifyOKBody`) created in Step 3 — defining the copy as named constants lets the test assert the exact production strings without driving the whole NFC/seed-entry flow.

- [ ] **Step 2: Run — verify it FAILS to compile (constants undefined)**

Run: `cd /tmp/seedhammer-verify-cluster && go test ./gui/ -run 'TestMultisigVerifyNoticeIsHonest' -v 2>&1 | head -20`
Expected: build failure `undefined: multisigVerifyOKTitle` / `undefined: multisigVerifyOKBody`.

- [ ] **Step 3: Add the constants and use them in the flow**

In `gui/multisig_verify.go`, add the constants just above `verifyMultisig` (after the import block, before the `verifyMultisig` doc comment at `:10`). Insert:

```go
// Multisig verify success copy (L2). HONEST scoping: on an air-gapped device the
// only cross-checkable facts are the operator's own key card (mk1, H1) + xpub/
// origin (findUserSlot) + the secret (ms1 entropy/language, M1). The md1 policy
// string is the supplied input compared to a clone of itself, and foreign
// cosigners' xpubs have no source of truth — so we do NOT claim a full-bundle
// guarantee.
const (
	multisigVerifyOKTitle = "Verify OK"
	multisigVerifyOKBody  = "Operator key and secret verified. Other cosigners' keys are taken as supplied."
)
```

Then replace the success call. OLD (`gui/multisig_verify.go:104`):
```go
	showNotice(ctx, th, "Verify OK", "The engraved bundle matches the seed.")
```
NEW:
```go
	showNotice(ctx, th, multisigVerifyOKTitle, multisigVerifyOKBody)
```

- [ ] **Step 4: Run — verify the notice test PASSES + full gui suite green**

Run: `cd /tmp/seedhammer-verify-cluster && go test ./gui/ -run 'TestMultisigVerifyNoticeIsHonest' -v && go test ./gui/`
Expected: `TestMultisigVerifyNoticeIsHonest` PASS; `ok  seedhammer.com/gui`.

- [ ] **Step 5: Commit**

```bash
cd /tmp/seedhammer-verify-cluster
git add gui/multisig_verify.go gui/multisig_verify_test.go
git commit -S -s -m "$(cat <<'EOF'
fix(gui): honest multisig verify success copy (L2)

The multisig "Verify OK" notice claimed "the engraved bundle matches the
seed" — an over-claim: on an air-gapped device only the operator's key
(mk1, after H1) + xpub/origin + secret are cross-checked; the md1 policy
is the supplied input and foreign cosigners' xpubs have no source of
truth. Scope the copy honestly via named constants and add a uiContains
regression test so the over-claim cannot silently return.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Relabel `TestVerifyBundleMd1Reordered` as a comparator-contract test

**Files:**
- Modify: `bundle/verify_test.go:110-130` (comment/name only — assertion UNCHANGED)

**Rationale:** This test asserts a *reordered* md1 readback FAILS. Under the H2 fix the ordering responsibility moves to `md1Gatherer.collected()`; `bundle.Verify` stays positional-by-contract and the test still passes UNCHANGED. We keep it (it guards the positional contract) but reframe its name/comment to say so and cross-reference T-H2 — we do NOT delete or invert it, and we do NOT weaken `bundle.Verify` to sort internally.

- [ ] **Step 1: Rename + reframe the comment (no assertion change)**

OLD (`bundle/verify_test.go:110-130`):
```go
// TestVerifyBundleMd1FieldNamed exercises the md1 exact-string branch directly:
// a read-back that shares the derived mk1 + ms1 (so fp/xpub/path agree and the
// stub binds) but is handed a re-chunked md1 that does NOT match the derived
// md1 strings. To keep the stub-binding precondition satisfiable we reuse the
// derived md1 for the binding side and assert the comparator's md1 field check
// fires when the strings differ — using a whitespace-trimmed-but-reordered set
// is not representative, so we instead confirm md1 ordering matters.
func TestVerifyBundleMd1Reordered(t *testing.T) {
	derived := correctBundle()
	readback := correctBundle()
	// Reorder the md1 chunks: a valid set (Reassemble is order-tolerant, so the
	// stub still binds) but the exact-string sequence differs → md1 mismatch.
	readback.MD1 = []string{wpkhMD1[1], wpkhMD1[0], wpkhMD1[2]}
	err := Verify(derived, readback)
	if err == nil {
		t.Fatal("reordered md1 accepted, want FAIL")
	}
	if !strings.Contains(err.Error(), "md1") {
		t.Errorf("error %q does not name md1", err)
	}
}
```

NEW:
```go
// TestVerifyBundleMd1PositionalContract documents and guards the COMPARATOR
// CONTRACT: bundle.Verify compares md1 POSITIONALLY (equalStrings), so it
// correctly rejects an out-of-order md1 []string. Canonical ChunkIndex ordering
// is the GATHER layer's responsibility — md1Gatherer.collected()
// (gui/md1_gather.go), which the H2 fix made deterministic; see the gui test
// TestMD1GathererCollectedIndexOrder (T-H2). This is NOT product behaviour that
// rejects correct backups (the gather layer canonicalizes order before Verify);
// it asserts the comparator stays a pure positional compare and is NOT weakened
// to sort internally (which would re-introduce parsing into the deterministic
// core). Assertion unchanged from the former TestVerifyBundleMd1Reordered.
func TestVerifyBundleMd1PositionalContract(t *testing.T) {
	derived := correctBundle()
	readback := correctBundle()
	// An out-of-order md1 []string: a valid set (Reassemble is order-tolerant, so
	// the stub still binds) but the positional sequence differs → md1 mismatch.
	readback.MD1 = []string{wpkhMD1[1], wpkhMD1[0], wpkhMD1[2]}
	err := Verify(derived, readback)
	if err == nil {
		t.Fatal("out-of-order md1 accepted by the positional comparator, want FAIL")
	}
	if !strings.Contains(err.Error(), "md1") {
		t.Errorf("error %q does not name md1", err)
	}
}
```

- [ ] **Step 2: Run — confirm it still PASSES (comparator unchanged by H2)**

Run: `cd /tmp/seedhammer-verify-cluster && go test ./bundle/ -run 'TestVerifyBundleMd1PositionalContract' -v && go test ./bundle/`
Expected: `TestVerifyBundleMd1PositionalContract` PASS; `ok  seedhammer.com/bundle`. (No `TestVerifyBundleMd1Reordered` remains — confirm with `go test ./bundle/ -run 'TestVerifyBundleMd1Reordered'` printing `ok` with no matching tests run.)

- [ ] **Step 3: Commit**

```bash
cd /tmp/seedhammer-verify-cluster
git add bundle/verify_test.go
git commit -S -s -m "$(cat <<'EOF'
test(bundle): relabel md1-reordered test as the positional-contract guard

Rename TestVerifyBundleMd1Reordered → TestVerifyBundleMd1PositionalContract
and reframe its comment: bundle.Verify compares md1 positionally by
contract; canonical ChunkIndex ordering is the gather layer's job
(md1Gatherer.collected(), fixed under H2 — see the gui T-H2 test). The
assertion is unchanged. Documents that the comparator is NOT weakened to
sort internally.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Final verification pass + worktree cleanup

**Files:** none (verification + cleanup).

- [ ] **Step 1: Re-run the fan-out greps (citations decay — confirm the table is still exact)**

```bash
cd /tmp/seedhammer-verify-cluster
grep -rn "ms1Entropy" bundle/ gui/
grep -rn "collected()" gui/
grep -rn "extractSuppliedMd1" gui/
grep -rn "verifyMultisig" gui/multisig_verify.go gui/multisig.go
```
Expected: `ms1Entropy` only at `bundle/verify.go:83,87` (call sites) + its def; `md1Gatherer.collected()` consumers at `gui/md1_gather.go:76,140` + `gui/bundle.go:234` (and the unrelated `mk1Gatherer.collected()` at `gui/bundle.go:194`, `gui/mk1_inspect.go`); `extractSuppliedMd1` still at `gui/multisig_verify.go:60`? — NO: after Task 3 the verify flow uses `extractSuppliedMd1AndMk1`; `extractSuppliedMd1` should now appear only at `gui/multisig.go:71` (engrave/supply) + the supply tests. Confirm we did NOT disturb `gui/multisig.go:71`.

- [ ] **Step 2: Full host build, test, vet**

```bash
cd /tmp/seedhammer-verify-cluster
go build ./...
go test ./...
go vet ./...
```
Expected: `go build` clean; `go test ./...` all `ok` (no FAIL); `go vet ./...` clean (no output).

- [ ] **Step 3: TinyGo device-build gate (the real CI gate — best effort in this env)**

```bash
cd /tmp/seedhammer-verify-cluster
command -v tinygo && cat Makefile 2>/dev/null | grep -iE "tinygo|target" | head
```
If `tinygo` is available, run the project's device-build target (e.g. `make` / the documented `tinygo build -target=...` invocation from the fork's build docs). If `tinygo` is NOT available in this environment, RECORD that explicitly in the final summary and flag it as a residual gate for the post-implementation exec review / final integration pass (the spec defers the TinyGo build to the final pass; the changes are loop reorder + two extra `int` returns + an existing `wipeBytes` call + new helpers — no reflection/goroutines/generics added, so TinyGo-safe by inspection, but the build must still be run before ship).

- [ ] **Step 4: Confirm the commit log**

```bash
cd /tmp/seedhammer-verify-cluster
git log --oneline 3a23dbb..HEAD
git log --format='%H %G? %an <%ae>' 3a23dbb..HEAD
```
Expected: 6 commits (M1, H2, H1, L1, L2, relabel), each `G` (good signature) or `E`/`N` depending on the signing setup — confirm `-S` produced a signature; author `Brian Goss <goss.brian@gmail.com>` on every commit; each body ends with the `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>` trailer.

- [ ] **Step 5: Leave the fork clean (do NOT merge; do NOT touch `seedhammer-wt-bip39`)**

The branch `feat/fix-verify-cluster` stays in its worktree for the exec-review gate. The MAIN checkout must be left clean on `main @ 3a23dbb`:

```bash
cd /scratch/code/shibboleth/seedhammer
git status --porcelain=v1   # expect empty
git rev-parse HEAD           # expect 3a23dbb3d8fe5f9a318b8bb8adbe8b6692cf2082
git branch --show-current    # expect main
git worktree list            # feat/fix-verify-cluster present; seedhammer-wt-bip39 untouched
```

> Do NOT run `git worktree remove` on `feat/fix-verify-cluster` (the implemented branch is the deliverable for the exec-review gate). Do NOT merge to `main`. Per the orchestration plan, merge is SERIAL (Track B → Track A) and happens only after both tracks pass their exec reviews.

---

## Self-Review (author checklist, run after writing)

**Spec coverage:** H1 (Task 3 — new helper + read-back mk1 + docstring), H2 (Task 2 — `collected()` index walk + T-H2), M1 (Task 1 — language compare + T-M1), L1 (Task 4 — both scrub sites), L2 (Task 5 — honest copy + uiContains test), relabel of the reordered test (Task 6). All five findings + the relabel + the three flow-level tests + Task 0 worktree + Task 7 final pass are present.

**Placeholder scan:** no TBD/TODO/"similar to"/"add error handling". Every code step shows the exact Old→New diff or full test body. Every command has an explicit Expected output.

**Type consistency:** `ms1Entropy` returns `(language int, entropy []byte, err error)` and BOTH call sites in `Verify` updated (`dLang, dEnt` / `rLang, rEnt`). `extractSuppliedMd1AndMk1(cards) (md1, mk1 []string, ok bool)` — return order `(md1, mk1, ok)` is consistent in the helper def (Task 3 Step 3), the extraction tests (Step 1: `gotMd1, gotMk1, ok`), and the flow wiring (Step 7: `suppliedMd1, suppliedMk1, ok`). L2 constants `multisigVerifyOKTitle`/`multisigVerifyOKBody` defined (Task 5 Step 3) and used in both the flow and the test. `wipeBytes` (existing, `gui/slip39_polish.go:344`) is called, never redefined.

**Empirical proof carried into the plan:** the three flow-level tests' exact error strings — `verify: ms1 wordlist/language mismatch` (T-M1), `collected()[...] want index order` (T-H2), `verify: readback mk1 decode: codex32: not a valid mk1 string` + `verify: readback mk1/md1 stub mismatch (key card does not bind to this policy)` (T-H1) — were observed in a throwaway worktree off `3a23dbb`, fail-before / pass-after, routing the production functions.
