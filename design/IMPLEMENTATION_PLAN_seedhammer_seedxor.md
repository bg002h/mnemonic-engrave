# Seed XOR combine — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development.
> Steps use `- [ ]` checkboxes. **GREEN spec:** `design/SPEC_seedhammer_seedxor.md` (R1 GREEN,
> `0664fa7`). Architect consult: `design/agent-reports/seedhammer-seedxor-design-consult.md`.
> Seed-bearing flow → full gated pipeline. Base: fork `main` `bc63caa`.

**Goal:** On-device **Seed XOR combine** — collect N Coldcard Seed-XOR parts (each a valid
BIP-39 mnemonic), XOR their entropy into the original seed, and engrave it via the existing
BIP-39 plate path, gated by a mandatory Seed-XOR-specific fingerprint screen.

**Architecture:** a tiny pure `seedxor` package (`Combine`, port of `seed_xor_combine`) +
`gui/seedxor_polish.go` (two pickers, the collection loop with the per-part validity guard, the
mandatory fingerprint gate) + a `"SEED XOR"` entry on the input `ChoiceScreen` (returns a
`bip39.Mnemonic` → existing dispatch). One additive change to `inputWordsFlow` (a `title` param).

**Tech stack:** Go/TinyGo. No `math/big`, no SHA, no RNG (combine is pure XOR). Reuses
`bip39.Entropy()`/`New()`, `backupWalletFlow`, `masterFingerprintFor`, the
`confirmSLIP39Fingerprint` template + Button2-drain idiom.

**Test:** `/home/bcg/.local/go/bin/go test ./seedxor/ ./gui/ ./bip39/` + `go vet` + `gofmt -l`.

**Commit hygiene:** explicit paths; SSH-signed + DCO (`git commit -S -s`, author Brian Goss);
end messages with `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.

---

## File structure

| File | Change |
|---|---|
| `seedxor/seedxor.go` | new — `Combine` + `Describe` + sentinels + `wipe`. |
| `seedxor/seedxor_test.go` + `seedxor/testdata/` | new — Coldcard/toolkit vectors + negatives + order-independence. |
| `gui/seedxor_polish.go` | new — `seedXORPartCount`, `seedXORPartLength`, `combineSeedXORFlow`, `confirmSeedXORFingerprint`. |
| `gui/seedxor_polish_test.go` | new — combine flow + mandatory-gate + no-hang + error-path tests. |
| `gui/gui.go` | modify — add `title string` to `inputWordsFlow` (additive); update all 10 callers (2 in gui.go + 8 in gui_test.go; enumerated in Task 2); add `"SEED XOR"` to `newInputFlow`'s `ChoiceScreen` + `case 4`. |

Unchanged (reused, must stay green): `bip39/`, `backupWalletFlow`/`masterFingerprintFor`/
`SeedScreen`, `codex32/`, `slip39/`, `backup/`.

---

## Task 0: Worktree

- [ ] **Step 1:** `git -C /scratch/code/shibboleth/seedhammer worktree add /scratch/code/shibboleth/seedhammer-wt-seedxor -b feat/seedxor-combine bc63caa`
- [ ] **Step 2:** Baseline: `cd …-seedxor && /home/bcg/.local/go/bin/go test ./gui/ ./bip39/` → green.

---

## Task 1: `seedxor` package — `Combine` (pure)

**Files:** `seedxor/seedxor.go`, `seedxor/seedxor_test.go`, `seedxor/testdata/vectors.json`.
Port of `mnemonic_toolkit::seed_xor::seed_xor_combine`.

- [ ] **Step 1: testdata.** Create `seedxor/testdata/vectors.json` from **two** sources:
  (1) **offline oracle (plan-R0 M-1, primary):** the toolkit's in-repo G1 byte-pin / G2 round-trip
  relations (`mnemonic-toolkit/.../tests/lib_seed_xor.rs`) — reproducible without network; use
  these as the authoritative correctness anchor.
  (2) **Coldcard interop cross-check:** copy the published vectors **verbatim** from Coldcard
  `docs/seed-xor.md` (raw: `https://raw.githubusercontent.com/Coldcard/firmware/master/docs/seed-xor.md`)
  / `testing/test_seed_xor.py` (NOT in our repos — fetch + cite in `testdata/SOURCE.md`): at
  minimum the 24-word 3-part (→ `silent toe meat … indoor`) and 12-word 3-part (→ `cannon
  opinion … trade`). Shape: `[{words:N, parts:[…], result:…}]`. (If the network fetch is
  unavailable at impl time, the offline toolkit oracle + a locally-regenerated toolkit-CLI
  round-trip fixture still fully exercise `Combine`; do NOT block on the fetch — note any
  deferred interop vector.)

- [ ] **Step 2: Failing test** (`seedxor/seedxor_test.go`): load each vector, parse parts +
  result via `bip39` (a `parseM(t, s)` helper), assert `Combine(parts) == result`, and
  **order-independence** (reverse/shuffle parts → same result; authored fresh — XOR commutes).
  Negatives: `Combine(parts[:1])` → `errTooFewParts`; a 12-word + 24-word mix → `errMismatchedLengths`;
  a 15-word (20-byte) part → `errBadLength`. Run → FAIL (no `seedxor`).

- [ ] **Step 3: Implement** `seedxor/seedxor.go`:

```go
// Package seedxor implements Coldcard Seed XOR combine: bit-wise XOR of the
// BIP-39 entropy of N parts (each itself a valid BIP-39 mnemonic), recovering
// the original seed. Strictly N-of-N, all parts the same Coldcard-interop
// length (16/24/32-byte = 12/18/24-word). Pure: no RNG, no SHA, no math/big.
// Port of mnemonic_toolkit::seed_xor::seed_xor_combine.
package seedxor

import (
	"errors"

	"seedhammer.com/bip39"
)

var (
	errTooFewParts       = errors.New("seedxor: need at least 2 parts")
	errBadLength         = errors.New("seedxor: unsupported length (use 12/18/24 words)")
	errMismatchedLengths = errors.New("seedxor: all parts must be the same length")
)

// interopLen reports whether n is a Coldcard-interop entropy length. The
// {16,24,32} guard is LOAD-BEARING: bip39.New accepts any 16..32 multiple-of-4
// (i.e. also 20/28 = 15/21-word), so this is the only thing enforcing interop.
func interopLen(n int) bool { return n == 16 || n == 24 || n == 32 }

func wipe(b []byte) {
	for i := range b {
		b[i] = 0
	}
}

// Combine reconstructs the original BIP-39 seed from N Seed-XOR parts. Each
// part must be a VALID BIP-39 mnemonic (Entropy panics otherwise — the GUI
// flow enforces validity per-part before calling; tests pass parsed vectors).
func Combine(parts []bip39.Mnemonic) (bip39.Mnemonic, error) {
	if len(parts) < 2 {
		return nil, errTooFewParts
	}
	out := append([]byte(nil), parts[0].Entropy()...)
	if !interopLen(len(out)) {
		wipe(out)
		return nil, errBadLength
	}
	for _, p := range parts[1:] {
		e := p.Entropy()
		if len(e) != len(out) {
			wipe(out)
			return nil, errMismatchedLengths
		}
		for i := range out {
			out[i] ^= e[i]
		}
	}
	m := bip39.New(out) // safe: len(out) ∈ {16,24,32}, all valid for New
	wipe(out)
	return m, nil
}

// Describe maps a Combine error to a short GUI label.
func Describe(err error) string {
	switch {
	case err == nil:
		return ""
	case errors.Is(err, errTooFewParts):
		return "need at least 2 parts"
	case errors.Is(err, errBadLength):
		return "unsupported length (use 12/18/24 words)"
	case errors.Is(err, errMismatchedLengths):
		return "all parts must be the same length"
	default:
		return "invalid"
	}
}
```

- [ ] **Step 4:** Run → PASS; `go vet ./seedxor/`, `gofmt -l seedxor/` clean.
- [ ] **Step 5: Commit** → `feat: seedxor package — Coldcard Seed XOR combine (port of seed_xor_combine)`.

---

## Task 2: `inputWordsFlow` gains a `title` param (additive — plan-R0 I-1/I-2)

**Files:** `gui/gui.go`, `gui/gui_test.go`. **Read first:** `inputWordsFlow` (`gui.go:580`)
renders a **dynamic** title `layoutTitlef(ctx, dims.X, th.Text, "Word %d of %d", selected+1,
len(mnemonic))` (`gui.go:701`), pinned by `TestWordFlowProgressTitle` (`gui_test.go:498`,
asserts `"Word 1 of 24"`). `inputSLIP39Flow` (`gui.go:868`) by contrast renders ONLY a free-form
`layoutTitle(..., title)` and no word-position line.

- [ ] **Step 1: Signature + precise render contract.** Change to
  `func inputWordsFlow(ctx, th, mnemonic bip39.Mnemonic, selected int, title string)`. Render
  contract (truly additive):
  - **`title == ""` → render the EXISTING `layoutTitlef("Word %d of %d", selected+1,
    len(mnemonic))` line byte-identically** (current behavior; `TestWordFlowProgressTitle` stays
    green).
  - **`title != ""` → render `layoutTitle(ctx, dims.X, th.Text, title)`** in place of the
    word-position line (i.e. like `inputSLIP39Flow` — the caller-supplied context replaces
    "Word N of M"; this is the established SLIP-39 share-entry behavior).
- [ ] **Step 2: Update ALL 10 call sites** (plan-R0 I-2 — adding a param is a hard compile error
  otherwise). Pass `""` at every existing site to preserve behavior:
  - `gui/gui.go`: `:2025` (12/24 menu), `:2102` (SeedScreen edit).
  - `gui/gui_test.go`: `:285, :491, :507, :604, :625, :642, :662, :681`.
  (The new Seed XOR caller in Task 3 passes `"Part i of n"`.)
- [ ] **Step 3:** Run `…/go test ./gui/` — it must **build** (all 10 sites updated) and the
  existing wallet-backup / SeedScreen / EngraveScreen / `TestWordFlowProgressTitle` tests stay
  green (proves the empty-title path is byte-identical). vet/gofmt clean.
- [ ] **Step 4: Commit** → `refactor: inputWordsFlow takes a title param (additive; empty=unchanged)`.

---

## Task 3: `gui/seedxor_polish.go` — pickers, combine flow, mandatory gate

**Files:** `gui/seedxor_polish.go`. `confirmSeedXORFingerprint` clones `confirmSLIP39Fingerprint`
(`slip39_polish.go:433`) — **keep the unconditional `drainBtn.Clicked(ctx)` Button2-drain**.

- [ ] **Step 1: Failing test** (in `gui/seedxor_polish_test.go`): see Task 4 — but at minimum a
  test that references `combineSeedXORFlow`/`confirmSeedXORFingerprint` so the package fails to
  compile until implemented. Run → FAIL.

- [ ] **Step 2: Implement** `gui/seedxor_polish.go`:

```go
package gui

import (
	"fmt"

	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip39"
	"seedhammer.com/seedxor"
)

// seedXORPartCount asks how many parts (N-of-N; min 2). 0 = Back.
func seedXORPartCount(ctx *Context, th *Colors) int {
	cs := &ChoiceScreen{Title: "Seed XOR", Lead: "How many parts?", Choices: []string{"2", "3", "4", "5"}}
	sel, ok := cs.Choose(ctx, th)
	if !ok {
		return 0
	}
	return sel + 2
}

// seedXORPartLength asks the word length of the parts (Coldcard-interop). 0 = Back.
// Mechanically required: inputWordsFlow fills a pre-sized slice, so the length
// must be known before entry; parts 2..N inherit this (no per-part re-pick).
func seedXORPartLength(ctx *Context, th *Colors) int {
	cs := &ChoiceScreen{Title: "Seed XOR", Lead: "Words per part?", Choices: []string{"12", "18", "24"}}
	sel, ok := cs.Choose(ctx, th)
	if !ok {
		return 0
	}
	return []int{12, 18, 24}[sel]
}

// combineSeedXORFlow collects N parts, XORs them into the recovered seed, and
// gates engrave behind the mandatory fingerprint check. (nil,false) on Back/abort.
func combineSeedXORFlow(ctx *Context, th *Colors) (bip39.Mnemonic, bool) {
	n := seedXORPartCount(ctx, th)
	if n == 0 {
		return nil, false
	}
	nwords := seedXORPartLength(ctx, th)
	if nwords == 0 {
		return nil, false
	}
	parts := make([]bip39.Mnemonic, 0, n)
	for i := 0; i < n; i++ {
		m := emptyBIP39Mnemonic(nwords)
		inputWordsFlow(ctx, th, m, 0, fmt.Sprintf("Part %d of %d", i+1, n))
		// I1 guard: inputWordsFlow returns a PARTIAL slice on Back. Only a
		// complete, checksum-valid part may be collected — else Entropy()
		// panics in Combine. A partial/invalid part aborts the whole flow.
		if !isMnemonicComplete(m) || !m.Valid() {
			return nil, false
		}
		parts = append(parts, m)
	}
	seed, err := seedxor.Combine(parts)
	if err != nil {
		showError(ctx, th, "Seed XOR", seedxor.Describe(err))
		return nil, false
	}
	mfp, ferr := masterFingerprintFor(seed, &chaincfg.MainNetParams, "")
	if ferr != nil {
		showError(ctx, th, "Seed XOR", "could not derive the fingerprint")
		return nil, false
	}
	if !confirmSeedXORFingerprint(ctx, th, mfp) {
		return nil, false
	}
	return seed, true
}
```
  And `confirmSeedXORFingerprint(ctx, th, mfp uint32) bool` — a clone of `confirmSLIP39Fingerprint`
  with the title "Recovered Fingerprint" and the **Seed-XOR-specific** body (two lines):
  `fmt.Sprintf("Fingerprint %.8X", mfp)` and *"Seed XOR has no built-in check — any wrong part
  still makes a valid wallet. Confirm this matches your records before engraving."* Button1=Back→
  false; Button3/Center=Engrave→true; **`drainBtn := &Clickable{Button: Button2}` drained every
  frame** (no-hang).

- [ ] **Step 3:** Run → compiles; vet/gofmt clean.
- [ ] **Step 4: Commit** → `feat: seedxor GUI combine flow + mandatory fingerprint gate`.

---

## Task 4: Menu wiring + GUI tests

**Files:** `gui/gui.go` (menu), `gui/seedxor_polish_test.go`.

- [ ] **Step 1:** In `newInputFlow` (`gui.go:2012`): add `"SEED XOR"` to `Choices` (after
  `"SLIP-39"` → index 4) and a `case 4:` `m, ok := combineSeedXORFlow(ctx, th); if ok { return m, true }`.
  The returned `bip39.Mnemonic` rides the existing `engraveObjectFlow case bip39.Mnemonic:`
  (`gui.go:1849`) → `backupWalletFlow`. No new dispatch case.

- [ ] **Step 2: Tests** (`gui/seedxor_polish_test.go`), driving via the harness (`runUI`/`click`/
  `runes` + a `driveWord`-style per-word helper like the SLIP-39 recover tests):
  - `TestCombineSeedXOR`: pick N=2, length=24, enter 2 parts from a vector → assert the recovered
    seed's fingerprint screen appears, then (selecting Engrave) `backupWalletFlow` is reached
    (`pumpUntil` the SeedScreen words / "Insert a blank plate"). Assert the recovered fingerprint
    matches the vector's expected `%.8X`.
  - `TestSeedXORFingerprintMandatory`: the fingerprint gate is on the only success path — Back at
    the gate → no engrave (flow returns `(nil,false)`, menu loops).
  - `TestSeedXORBackoutRecognized`: Back during part entry (partial fill) → `(nil,false)`; the
    menu/`newInputFlow` does not crash and re-displays (no `Entropy()` panic — the I1 guard).
  - `TestConfirmSeedXORFingerprintButton2NoHang`: direct-call, queued Button2 then Button3 →
    the gate doesn't stall (Button2 drained).
  - `TestSeedXORLengthMismatchError` (if drivable) or a `seedxor`-level test already covers
    `errMismatchedLengths`/`errBadLength`.

- [ ] **Step 3: Full guard:** `…/go test ./seedxor/ ./gui/ ./bip39/`, `go vet ./gui/ ./seedxor/`,
  `gofmt -l`. Existing guards (codex32/SLIP-39/BIP-39/backup goldens, wallet-backup/SeedScreen) green.
- [ ] **Step 4: Commit** → `test: seedxor combine flow + mandatory-gate + no-hang guards; wire SEED XOR menu`.

---

## Self-review checklist

- `Combine` is pure (no RNG/SHA/`math/big`); the `{16,24,32}` guard rejects 15/21-word
  (verified `bip39.New` doesn't); order-independence tested; vectors sourced from Coldcard's
  authoritative doc/test with `testdata/SOURCE.md` citing them.
- The fingerprint gate is **mandatory** (on the only success path), Seed-XOR-worded, and
  Button2-drained; a recovered seed cannot reach `backupWalletFlow` without it.
- The **I1 per-part `isMnemonicComplete && Valid()` guard** is present before every part is
  collected — no `Entropy()` panic path; Back/partial aborts the flow.
- `inputWordsFlow`'s title param is additive — `title==""` renders the existing `"Word N of M"`
  byte-identically (`TestWordFlowProgressTitle` green); ALL 10 call sites updated (2 in `gui.go`,
  8 in `gui_test.go`) so `./gui/` compiles; Seed XOR passes `"Part i of n"`.
- Menu returns a `bip39.Mnemonic` → existing dispatch; no new `engraveObjectFlow` case; no new
  `gui.go` import.
- No interpretation fork / hold-to-confirm (Seed XOR result is unambiguously a BIP-39 seed).
- Signed + DCO + Brian Goss; existing guards green.
