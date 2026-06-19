# T7b — on-device BIP-85 derive-child → engrave — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a new on-device `bip85Derive` program to the SeedHammer II firmware (fork `bg002h/seedhammer`) that turns ONE typed master BIP-39 seed into a DETERMINISTIC child BIP-39 mnemonic via BIP-85 (`m/83696968'/39'/0'/{words}'/{index}'`) and engraves the child as a seed-backup plate (words + standard SeedQR) using the EXACT `engraveSeed` primitive that Backup Wallet uses.

**Architecture:** Re-create biptool's `derive bip39` algorithm inside the GUI as a pure helper (`deriveBip85Child`), wire a `ChoiceScreen`-based child-param picker (app fixed BIP-39, word-count ∈ {12,18,24}, bounded index 0–9), gate engrave behind an unskippable child-seed warning, and stamp the CHILD's own bare-seed fingerprint on the plate (never the master's). The new program slots into the program carousel between `engraveMultisig` and the non-navigable `qaProgram` sentinel, touching the 8 mechanical lockstep sites. Two secrets — the typed master mnemonic AND the derived child mnemonic — are `defer`-scrubbed on every exit, plus the intermediate privkey serialization and HMAC output are `wipeBytes`-d. ZERO new crypto: `bip85.Entropy` + `hdkeychain` + `bip39.New` + `engraveSeed` all ship.

**Tech Stack:** Go 1.26 (`export PATH=$PATH:/home/bcg/.local/go/bin`), module `seedhammer.com`. Libraries: `github.com/btcsuite/btcd/btcutil/v2/hdkeychain`, `github.com/btcsuite/btcd/chaincfg/v2`, in-tree `seedhammer.com/{bip32,bip39,bip85}` and the `gui` package. Tests are standard `go test`; the GUI uses a software `testPlatform` (`gui/gui_test.go`) and a frame-driving harness (`runUI`/`click`/`uiContains`).

## Global Constraints

- **Fork base:** `82d46b3` (T6 complete), branch `main`, repo `/scratch/code/shibboleth/seedhammer`. **Fork-side only; NO upstream PR.**
- **Go on PATH:** every `go`/`git` command runs after `export PATH=$PATH:/home/bcg/.local/go/bin`.
- **Commits:** signed (`-S`) + DCO (`-s`), author `Brian Goss <goss.brian@gmail.com>`, trailer `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. **Explicit-path staging only — never `git add -A`.**
- **TDD strictly:** failing test → run-fail → minimal impl → run-pass → commit. No production code before a red test.
- **Spec-faithful-or-nothing:** picker bounds = biptool's EXACTLY (`words ∈ {12,18,24}`, fully hardened, English=`0'`). An out-of-spec param would mint a child no other BIP-85 wallet reproduces (silent-wrong-backup). Child entropy = the **LEADING** `entLen` bytes of the 64-byte HMAC output (never trailing).
- **Child fingerprint:** the plate's `MasterFingerprint` MUST be the **CHILD's own** bare-seed fp (`masterFingerprintFor(child, &chaincfg.MainNetParams, "")`), NEVER the master's.
- **Mainnet-only.** Typed-only master (NEVER scan→derive). Child engraved onto owner-held steel only, never NFC. No xprv/extended-key serialized into the artifact.
- **`qaProgram` stays the last non-navigable sentinel.**

---

## Verified facts (do not re-derive)

All confirmed by reading the fork at HEAD `82d46b3` and by a live probe (removed after authoring). Source citations are file:line at `82d46b3`.

### Signatures (read from source)
- `bip85.Entropy(privkey []byte) []byte` — `bip85/bip85.go:16`; HMAC-SHA512(key `"bip-entropy-from-k"` `:13`, msg=32-byte privkey), 64-byte output; **PANICS if `len(privkey)!=32`** (`:17-18`). `const PathRoot = 83696968 + 0x80000000` (`:11`).
- `bip39.New(entropy []byte) bip39.Mnemonic` — `bip39/bip39.go:228`; **PANICS** if `len < 16 || 32 < len` (`:229`) or `len%4 != 0` (`:232`). `entLen ∈ {16,24,32}` all satisfy → no panic.
- `bip39.MnemonicSeed(m bip39.Mnemonic, password string) []byte` — `bip39/bip39.go:217`; PBKDF2 → 64-byte seed.
- `bip39.ParseMnemonic(mnemonic string) (bip39.Mnemonic, error)` — `bip39/bip39.go:277` (used in tests to build a fixed vector from a phrase).
- `bip39.ShortestWord = 3` (`bip39/wordlist.go:7`), `bip39.LongestWord = 8` (`:9`), `bip39.LabelFor(w Word) string` (`bip39/bip39.go:79`).
- `hdkeychain.NewMaster(seed []byte, net *chaincfg.Params) (*ExtendedKey, error)`; `(*ExtendedKey).Derive(i uint32) (*ExtendedKey, error)`; `(*ExtendedKey).ECPrivKey() (*btcec.PrivateKey, error)`; `(*ExtendedKey).Zero()`; `const HardenedKeyStart = 0x80000000`. (btcutil/v2@v2.0.0 `hdkeychain/extendedkey.go:654,228,546,634,37`.)
- `engraveSeed(params engrave.Params, m bip39.Mnemonic, mfp uint32) (Plate, error)` — `gui/gui.go:461`; stamps `MasterFingerprint: mfp` (`:475`) via `backup.EngraveSeed`. **`Plate` is `{Duration uint; Spline bspline.Curve}`** — `Spline` is a FUNC type, NOT a slice (do not `len()` it); the plate is opaque curve math, so engrave correctness is asserted on the INPUTS (child mnemonic + fp) plus `err==nil`.
- `masterFingerprintFor(m bip39.Mnemonic, network *chaincfg.Params, password string) (uint32, error)` — `gui/gui.go:485`; returns `bip32.Fingerprint(pkey), nil` (`:494`) or `0, err`. **Two-value return — propagate the err.**
- `seedEntryFlow(ctx *Context, th *Colors) (bip39.Mnemonic, bool)` — `gui/derive_xpub.go:82`; typed-only, master count `[]int{12,24}` (`:89`). Returns `ok==false` on Back. **Caller MUST scrub the returned mnemonic.**
- `(*ChoiceScreen).Choose(ctx *Context, th *Colors) (int, bool)` — `gui/gui.go:1363`; struct `ChoiceScreen{Title, Lead string; Choices []string}` (`:1349-1355`). Returns the selected index + `ok` (`false` on Back/cancel).
- `showError(ctx *Context, th *Colors, title, msg string)` — `gui/slip39_polish.go:22`.
- `wipeBytes(b []byte)` — `gui/slip39_polish.go:330` (best-effort zero a `[]byte`).
- `ConfirmWarningScreen{Title, Body string; Icon image.RGBA64Image}` + `(*ConfirmWarningScreen).Layout(ctx, th, dims) (op.Op, ConfirmResult)` returning `ConfirmNo`/`ConfirmYes`/`ConfirmNone` (`gui/gui.go:222,318`; consts `:243-245`). Hold-to-confirm pattern: see `stubZeroWarning` (`gui/derive_xpub.go:237-256`) which loops `Layout` and returns `true` only on `ConfirmYes`. `assets.IconHammer` is the warning icon used by `stubZeroWarning`.
- `passphraseFlow(ctx *Context, th *Colors) (string, bool)` — `gui/gui.go:499`.

### Scrub discipline to MIRROR (`deriveAccountXpub`, `gui/derive.go:19-53`)
`seed := bip39.MnemonicSeed(m, passphrase); defer wipeBytes(seed)`; capture any value BEFORE zeroing the key that owns its buffer; `.Zero()` each intermediate `ExtendedKey` (Derive returns fresh buffers, no aliasing). Top-level mnemonic `[]Word` scrubbed via a `defer func(){ for i := range m { m[i]=0 } }()` (see `engraveSingleSigFlow` `gui/singlesig.go:41-45`).

### The 8 lockstep sites @ `82d46b3` (all reference `engraveMultisig` as the current upper bound)
1. **enum const block** — `gui/gui.go:147-154` (`backupWallet`=0 … `engraveMultisig`@`:152`, `qaProgram`@`:153`).
2. **dispatch switch** — `gui/gui.go:1492-1514` (under `if obj == nil`).
3. **left-wrap** — `gui/gui.go:1640-1643` (`m.prog--; if m.prog < 0 { m.prog = engraveMultisig }`).
4. **right-wrap** — `gui/gui.go:1644-1652` (`m.prog++; if m.prog > engraveMultisig { m.prog = 0 }`).
5. **title switch** — `gui/gui.go:1666-1678` (`switch m.prog { … case engraveMultisig: titleTxt = "Engrave Multisig" }`).
6. **`npage`** — `gui/gui.go:1852` (`const npage = int(engraveMultisig) + 1`).
7. **`layoutMainPlates`** — `gui/gui.go:1860-1867` (`case backupWallet, …, engraveMultisig:` + the MANDATORY `panic("invalid page")` default `:1867`).
8. **`npages`** — `gui/gui.go:1871` (`const npages = int(engraveMultisig) + 1`).

Nav-test precedent (T6b): `gui/multisig_program_test.go` — exactly 2 tests (`…ProgramNavigable` + `…LeftWrap`). Prior programs' nav-tests that hard-code the carousel upper bound / Right-count and MUST be repointed: `gui/singlesig_program_test.go`, `gui/bundle_program_test.go`, `gui/derive_xpub_program_test.go`. `TestAllocs` is at `gui/gui_test.go:93`.

### Pinned goldens (computed by live probe at authoring; method below)
**Master test seed (12-word "abandon … about"):** `abandonAboutMnemonic()` (`gui/derive_test.go:13`) — 11×word 0 (`abandon`) + word 3 (`about`). Its master fp is `0x73c5da0a` (matches `knownMasterFP`, `gui/derive_test.go:27`).

BIP-85 children of the abandon master, English (`0'`), index `0'`:

| words | path | child entropy (leading `entLen` B) | child mnemonic | child master fp |
|---|---|---|---|---|
| 12 | `m/83696968'/39'/0'/12'/0'` | `ac98dac5d4f4ebad6056682ac95eb9ad` (16 B) | `prosper short ramp prepare exchange stove life snack client enough purpose fold` | `0x02e8bff2` |
| 18 | `m/83696968'/39'/0'/18'/0'` | `fc039f51d67ed7dfd01552f27de28887cf3e58655153e44b` (24 B) | `winter brother stamp provide uniform useful doctor prevent venue upper peasant auto view club next clerk tone fox` | `0x3bb5fd0c` |
| 24 | `m/83696968'/39'/0'/24'/0'` | `d5a9cb46670566c4246b6e7af22e1dfc3668744ed831afea7ce2beea44e34e23` (32 B) | `stick exact spice sock filter ginger museum horse kit multiply manual wear grief demand derive alert quiz fault december lava picture immune decade jaguar` | `0xc2f2dd51` |

Index variation (abandon master, 12 words): `idx 0` = the 12-word child above; `idx 1` = `sing slogan bar group gauge sphere rescue fossil loyal vital model desert`; `idx 9` = `earth ice square lottery area detail test spike innocent right matter bubble`.

**Canonical BIP-85 cross-reference** (the standard spec vector, drivable by the SAME helper): master phrase `install scatter logic circle pencil average fall shoe quantum disease suspect usage` → `m/83696968'/39'/0'/12'/0'` → entropy leading-16 `6250b68daf746d12a24d58b4787a714b` → child `girl mad pet galaxy egg matter matrix prison refuse sense ordinary nose` (master fp `0x627ef3a6`, child fp `0x595037d0`). This is BYTE-IDENTICAL to the spec §5.1 vector and to the R0 live re-verification.

**How the goldens were computed (so plan-R0 can re-verify by probe):** added a temp `bip85/probe_test.go` that re-creates biptool's `derive bip39` — `bip39.MnemonicSeed(m, "")` → `hdkeychain.NewMaster(seed, &chaincfg.MainNetParams)` → walk `[]uint32{PathRoot, 39+h, 0+h, uint32(words)+h, uint32(index)+h}` via `.Derive` → `xkey.ECPrivKey().Serialize()` → `bip85.Entropy(...)` → `bip39.New(ent[:(words*11-words/3)/8])` → `.String()`. A second probe computed the fps via `bip39.MnemonicSeed`→`NewMaster`→`ECPubKey`→`bip32.Fingerprint`. The abandon master fp printed `73c5da0a` (= the in-tree `knownMasterFP`), and the canonical vector printed `6250b68d…` / `girl mad pet…` IDENTICAL to BIP-85 spec — confirming the helper is correct. Probes were deleted; the tree is clean at `82d46b3`.

---

## File-structure map

**Create:**
- `gui/bip85.go` — the `deriveBip85Child` helper + the child-param picker (`bip85ParamPickFlow`) + the unskippable child-seed warning (`childSeedWarning`) + the `bip85DeriveFlow` orchestrator. One file, one responsibility (the BIP-85 program), mirroring the single-file `gui/singlesig.go` orchestrator pattern.
- `gui/bip85_test.go` — the derive-helper golden/guard tests, the child-fp engrave test, the picker-bounds tests, the warning-abort test, the scrub test, and the derive-helper fuzz.
- `gui/bip85_program_test.go` — the 2 nav-tests (`TestBip85DeriveProgramNavigable` + `TestBip85DeriveLeftWrap`), mirroring `gui/multisig_program_test.go`.

**Modify:**
- `gui/gui.go` — the 8 lockstep sites (enum `:147-154`, dispatch `:1492-1514`, left-wrap `:1640-1643`, right-wrap `:1644-1652`, title `:1666-1678`, `npage` `:1852`, `layoutMainPlates` `:1860-1867`, `npages` `:1871`).
- `gui/singlesig_program_test.go`, `gui/bundle_program_test.go`, `gui/derive_xpub_program_test.go` — repoint the carousel-upper-bound references (the wrap boundary moves one program later). NOTE (m3): the only ASSERTION that breaks in `gui/singlesig_program_test.go` is `TestEngraveSingleSigLeftWrap` (one `"Multisig"`→`"BIP-85"`); `TestEngraveSingleSigProgramNavigable` stays green (it stops at Multisig, still position 4). `bundle_program_test.go` and `derive_xpub_program_test.go` are comment-only (their assertions stop short of the wrap boundary). The full wrap-boundary repoint (both `Navigable` + `LeftWrap`) only applies to `gui/multisig_program_test.go`.

---

## Task 0 — Worktree + baseline

**Files:** none (setup).

- [ ] **Step 0.1: Create an isolated worktree off `main` (82d46b3).**

REQUIRED SUB-SKILL: `superpowers:using-git-worktrees`. Run:

```bash
export PATH=$PATH:/home/bcg/.local/go/bin
git -C /scratch/code/shibboleth/seedhammer worktree add -b feat/t7b-bip85-derive /scratch/code/shibboleth/seedhammer-t7b main
git -C /scratch/code/shibboleth/seedhammer-t7b log --oneline -1
```
Expected: the worktree HEAD prints `82d46b3 Merge T6b: …`. **All subsequent commands run in `/scratch/code/shibboleth/seedhammer-t7b`.**

- [ ] **Step 0.2: Configure commit identity in the worktree.**

```bash
git -C /scratch/code/shibboleth/seedhammer-t7b config user.name "Brian Goss"
git -C /scratch/code/shibboleth/seedhammer-t7b config user.email "goss.brian@gmail.com"
```

- [ ] **Step 0.3: Baseline the relevant packages green.**

```bash
export PATH=$PATH:/home/bcg/.local/go/bin
cd /scratch/code/shibboleth/seedhammer-t7b && go test ./gui/... ./bip85/... ./bip39/...
```
Expected: `ok seedhammer.com/gui`, `ok seedhammer.com/bip85`, `ok seedhammer.com/bip39` (no FAIL). If red, STOP — the worktree is not a clean baseline.

---

## Task 1 — the derive helper (`deriveBip85Child`)

**Files:**
- Create: `gui/bip85.go`
- Test: `gui/bip85_test.go`

**Interfaces:**
- Consumes: `bip85.Entropy`, `bip85.PathRoot`, `bip39.New`, `bip39.MnemonicSeed`, `hdkeychain.NewMaster/Derive/ECPrivKey/Zero`, `wipeBytes`.
- Produces: `deriveBip85Child(m bip39.Mnemonic, passphrase string, words, index int) (bip39.Mnemonic, error)` — the FULLY-hardened BIP-85 BIP-39 child. Returns an error on an invalid `words` (not in {12,18,24}), a negative `index`, or any `hdkeychain` failure. Scrubs the PBKDF2 seed, every intermediate `ExtendedKey`, the privkey serialization, and the HMAC output before returning.

- [ ] **Step 1.1: Write the failing golden + guard tests.**

Create `gui/bip85_test.go`:

```go
package gui

import (
	"testing"

	"seedhammer.com/bip39"
)

// canonicalBip85Master is the standard BIP-85 spec test-vector master seed.
func canonicalBip85Master(t *testing.T) bip39.Mnemonic {
	t.Helper()
	m, err := bip39.ParseMnemonic("install scatter logic circle pencil average fall shoe quantum disease suspect usage")
	if err != nil {
		t.Fatalf("ParseMnemonic(canonical master): %v", err)
	}
	return m
}

// TestDeriveBip85Child_AbandonGoldens pins the BIP-85 BIP-39 children of the
// canonical abandon-about master at index 0 for each word count. A trailing-bytes
// truncation bug, a wrong path element, or an unhardened element all yield a
// different child and fail here.
func TestDeriveBip85Child_AbandonGoldens(t *testing.T) {
	tests := []struct {
		words int
		want  string
	}{
		{12, "prosper short ramp prepare exchange stove life snack client enough purpose fold"},
		{18, "winter brother stamp provide uniform useful doctor prevent venue upper peasant auto view club next clerk tone fox"},
		{24, "stick exact spice sock filter ginger museum horse kit multiply manual wear grief demand derive alert quiz fault december lava picture immune decade jaguar"},
	}
	for _, tc := range tests {
		child, err := deriveBip85Child(abandonAboutMnemonic(), "", tc.words, 0)
		if err != nil {
			t.Fatalf("words=%d: %v", tc.words, err)
		}
		if got := child.String(); got != tc.want {
			t.Fatalf("words=%d child mismatch:\n got %q\nwant %q", tc.words, got, tc.want)
		}
		if len(child) != tc.words {
			t.Fatalf("words=%d: child has %d words", tc.words, len(child))
		}
		if !child.Valid() {
			t.Fatalf("words=%d: child fails BIP-39 checksum", tc.words)
		}
	}
}

// TestDeriveBip85Child_CanonicalVector cross-checks the helper against the
// canonical BIP-85 spec vector (master -> m/83696968'/39'/0'/12'/0').
func TestDeriveBip85Child_CanonicalVector(t *testing.T) {
	child, err := deriveBip85Child(canonicalBip85Master(t), "", 12, 0)
	if err != nil {
		t.Fatal(err)
	}
	const want = "girl mad pet galaxy egg matter matrix prison refuse sense ordinary nose"
	if got := child.String(); got != want {
		t.Fatalf("canonical vector mismatch:\n got %q\nwant %q", got, want)
	}
}

// TestDeriveBip85Child_IndexVaries confirms distinct indices yield distinct
// children (the index participates in the hardened path).
func TestDeriveBip85Child_IndexVaries(t *testing.T) {
	c0, err := deriveBip85Child(abandonAboutMnemonic(), "", 12, 0)
	if err != nil {
		t.Fatal(err)
	}
	c1, err := deriveBip85Child(abandonAboutMnemonic(), "", 12, 1)
	if err != nil {
		t.Fatal(err)
	}
	if c0.String() == c1.String() {
		t.Fatal("index 0 and index 1 produced the same child")
	}
	const wantIdx1 = "sing slogan bar group gauge sphere rescue fossil loyal vital model desert"
	if got := c1.String(); got != wantIdx1 {
		t.Fatalf("idx1 child mismatch:\n got %q\nwant %q", got, wantIdx1)
	}
}

// TestDeriveBip85Child_RejectsBadWords: out-of-spec word counts error (never panic).
func TestDeriveBip85Child_RejectsBadWords(t *testing.T) {
	for _, w := range []int{0, 11, 13, 15, 21, 25, 27, -3} {
		if _, err := deriveBip85Child(abandonAboutMnemonic(), "", w, 0); err == nil {
			t.Fatalf("words=%d: expected an error, got nil", w)
		}
	}
}

// TestDeriveBip85Child_RejectsNegativeIndex: a negative index errors.
func TestDeriveBip85Child_RejectsNegativeIndex(t *testing.T) {
	if _, err := deriveBip85Child(abandonAboutMnemonic(), "", 12, -1); err == nil {
		t.Fatal("index=-1: expected an error, got nil")
	}
}
```

- [ ] **Step 1.2: Run to verify it fails.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestDeriveBip85Child' -v`
Expected: build failure / FAIL — `undefined: deriveBip85Child`.

- [ ] **Step 1.3: Write the minimal implementation.**

Create `gui/bip85.go`:

```go
package gui

import (
	"errors"
	"fmt"

	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip39"
	"seedhammer.com/bip85"
)

// validBip85Words is the set of child word counts the BIP-39 application
// supports (biptool's guard n<12||24<n||n%3!=0 -> exactly {12,18,24}).
func validBip85Words(n int) bool {
	return n == 12 || n == 18 || n == 24
}

// deriveBip85Child re-creates biptool's `derive bip39` (cmd/biptool/main.go:137-189)
// from a TYPED master mnemonic + optional passphrase: it walks the FULLY-hardened
// BIP-85 path m/83696968'/39'/0'/{words}'/{index}', extracts the leaf's 32-byte EC
// private key, runs bip85.Entropy (HMAC-SHA512), keeps the LEADING entLen bytes,
// and maps them to a child BIP-39 mnemonic via bip39.New.
//
// SECURITY: every secret buffer is scrubbed before return — the PBKDF2 seed, each
// intermediate ExtendedKey (.Zero), the privkey serialization, and the 64-byte
// HMAC output (wipeBytes). The caller still owns scrubbing the master and the
// returned child mnemonic (see bip85DeriveFlow). Deterministic: no CSPRNG.
func deriveBip85Child(m bip39.Mnemonic, passphrase string, words, index int) (bip39.Mnemonic, error) {
	if !validBip85Words(words) {
		return nil, fmt.Errorf("bip85: invalid child word count: %d", words)
	}
	if index < 0 {
		return nil, fmt.Errorf("bip85: invalid index: %d", index)
	}

	const h = hdkeychain.HardenedKeyStart
	seed := bip39.MnemonicSeed(m, passphrase)
	defer wipeBytes(seed)

	xkey, err := hdkeychain.NewMaster(seed, &chaincfg.MainNetParams)
	if err != nil {
		return nil, err
	}
	// Fully-hardened path: 83696968' / 39' / 0' (English) / words' / index'.
	path := []uint32{
		bip85.PathRoot,
		39 + h,
		0 + h,
		uint32(words) + h,
		uint32(index) + h,
	}
	k := xkey
	for _, p := range path {
		next, derr := k.Derive(p)
		k.Zero() // scrub master + each intermediate (Derive returns fresh buffers)
		if derr != nil {
			return nil, derr
		}
		k = next
	}
	// Leaf EC private key. ECPrivKey returns (*PrivateKey, error); it cannot fire
	// for a master+hardened walk, but never .Serialize() a nil.
	pkey, err := k.ECPrivKey()
	if err != nil {
		k.Zero()
		return nil, err
	}
	priv := pkey.Serialize() // 32-byte secret
	k.Zero()
	defer wipeBytes(priv)

	hmacOut := bip85.Entropy(priv) // 64-byte secret
	defer wipeBytes(hmacOut)

	entLen := (words*11 - words/3) / 8 // 12->16, 18->24, 24->32
	if entLen < 16 || entLen > 32 || entLen%4 != 0 {
		// Unreachable for words in {12,18,24}; guard so bip39.New never panics.
		return nil, errors.New("bip85: internal entropy-length error")
	}
	child := bip39.New(hmacOut[:entLen]) // LEADING entLen bytes
	return child, nil
}
```

- [ ] **Step 1.4: Run to verify it passes.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestDeriveBip85Child' -v`
Expected: PASS — all 5 `TestDeriveBip85Child_*` tests.

- [ ] **Step 1.5: Commit.**

```bash
cd /scratch/code/shibboleth/seedhammer-t7b
git add gui/bip85.go gui/bip85_test.go
git commit -S -s -m "feat(gui): add BIP-85 derive-child helper (T7b)

Re-creates biptool's derive bip39 inside the GUI: fully-hardened path
m/83696968'/39'/0'/{words}'/{index}', bip85.Entropy over the leaf privkey,
leading entLen bytes -> bip39.New. Scrubs seed/intermediates/privkey/HMAC.
Pinned against the abandon-about goldens and the canonical BIP-85 vector.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2 — child fingerprint + engrave glue (`engraveBip85Child`)

**Files:**
- Modify: `gui/bip85.go`
- Test: `gui/bip85_test.go`

**Interfaces:**
- Consumes: `deriveBip85Child` (Task 1), `masterFingerprintFor`, `engraveSeed`, `engrave.Params`.
- Produces: `engraveBip85Child(params engrave.Params, child bip39.Mnemonic) (Plate, uint32, error)` — computes the CHILD's own bare-seed fp and returns the engraved `Plate` plus that fp. The fp is the value stamped on the plate (`MasterFingerprint`), and it is ALWAYS the child's own (never the master's). This is the glue the orchestrator (Task 5) calls; it deliberately skips `backupWalletFlow`'s passphrase-fp picker (the child is bare).

- [ ] **Step 2.1: Write the failing test.**

Append to `gui/bip85_test.go`:

```go
// TestEngraveBip85Child_UsesChildFP asserts the engrave glue stamps the CHILD's
// OWN bare-seed fingerprint (R0-I-A: wrong-identifier-on-permanent-backup) — not
// the master's — and that it engraves the child mnemonic (not the master).
func TestEngraveBip85Child_UsesChildFP(t *testing.T) {
	params := newPlatform().EngraverParams()
	master := abandonAboutMnemonic()
	masterFP, err := masterFingerprintFor(master, &chaincfg.MainNetParams, "")
	if err != nil {
		t.Fatal(err)
	}
	child, err := deriveBip85Child(master, "", 12, 0)
	if err != nil {
		t.Fatal(err)
	}
	wantChildFP, err := masterFingerprintFor(child, &chaincfg.MainNetParams, "")
	if err != nil {
		t.Fatal(err)
	}
	_, gotFP, err := engraveBip85Child(params, child)
	if err != nil {
		t.Fatalf("engraveBip85Child: %v", err)
	}
	if gotFP != wantChildFP {
		t.Fatalf("engraved fp = %08x, want the CHILD's own fp %08x", gotFP, wantChildFP)
	}
	if gotFP == masterFP {
		t.Fatalf("engraved the MASTER's fp %08x — must be the child's own", masterFP)
	}
	// Pin the concrete child fp golden (abandon master, 12 words, idx 0).
	if gotFP != 0x02e8bff2 {
		t.Fatalf("child fp = %08x, want 02e8bff2", gotFP)
	}
}
```

Add `"github.com/btcsuite/btcd/chaincfg/v2"` to the test file's imports (alongside the existing `bip39` import).

- [ ] **Step 2.2: Run to verify it fails.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestEngraveBip85Child' -v`
Expected: build failure / FAIL — `undefined: engraveBip85Child`.

- [ ] **Step 2.3: Write the minimal implementation.**

Add to `gui/bip85.go`. First extend the import block with `engrave`:

```go
import (
	"errors"
	"fmt"

	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip39"
	"seedhammer.com/bip85"
	"seedhammer.com/engrave"
)
```

Then add the function:

```go
// engraveBip85Child computes the CHILD's OWN bare-seed master fingerprint and
// engraves the child mnemonic (words + standard SeedQR) via the engraveSeed
// PRIMITIVE — the exact Backup-Wallet path. R0-I-A: the plate's MasterFingerprint
// MUST be the child's own bare fp (the child is a bare mnemonic, no passphrase),
// NEVER the master's, otherwise the steel carries a fingerprint that does not
// match the engraved words. This skips backupWalletFlow's passphrase-fp picker.
func engraveBip85Child(params engrave.Params, child bip39.Mnemonic) (Plate, uint32, error) {
	mfp, err := masterFingerprintFor(child, &chaincfg.MainNetParams, "") // child's OWN bare fp; propagate err (R0-A1)
	if err != nil {
		return Plate{}, 0, err
	}
	plate, err := engraveSeed(params, child, mfp)
	if err != nil {
		return Plate{}, 0, err
	}
	return plate, mfp, nil
}
```

- [ ] **Step 2.4: Run to verify it passes.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestEngraveBip85Child' -v`
Expected: PASS — `TestEngraveBip85Child_UsesChildFP`.

- [ ] **Step 2.5: Commit.**

```bash
cd /scratch/code/shibboleth/seedhammer-t7b
git add gui/bip85.go gui/bip85_test.go
git commit -S -s -m "feat(gui): engrave BIP-85 child via engraveSeed with the child's own fp (T7b)

engraveBip85Child reuses the engraveSeed primitive and stamps the CHILD's
own bare-seed fingerprint (never the master's) — guarding the
wrong-identifier-on-permanent-backup class. Skips backupWalletFlow's
passphrase-fp picker (the child is bare).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3 — the child-param picker (`bip85ParamPickFlow`)

**Files:**
- Modify: `gui/bip85.go`
- Test: `gui/bip85_test.go`

**Interfaces:**
- Consumes: `ChoiceScreen`, `deriveBip85Child` (for the bounds-coverage test only).
- Produces:
  - `var bip85WordChoices = []int{12, 18, 24}` and `var bip85IndexChoices = []int{0,1,2,3,4,5,6,7,8,9}` — the picker's exact, validated-by-construction bounds.
  - `bip85ParamPickFlow(ctx *Context, th *Colors) (words, index int, ok bool)` — the application is FIXED to BIP-39 (no app choice screen); a word-count `ChoiceScreen` {12,18,24} then a bounded index `ChoiceScreen` {0..9}, default 0. `ok==false` on Back. The returned `(words, index)` are ALWAYS in-spec.

- [ ] **Step 3.1: Write the failing bounds test.**

Append to `gui/bip85_test.go`:

```go
// TestBip85ParamBounds asserts the picker's choice sets are exactly the in-spec
// bounds (validated-by-construction): word count {12,18,24}, index {0..9}. Any
// drift here (e.g. a 15 or a free-form index) would mint an out-of-spec child.
func TestBip85ParamBounds(t *testing.T) {
	if len(bip85WordChoices) != 3 ||
		bip85WordChoices[0] != 12 || bip85WordChoices[1] != 18 || bip85WordChoices[2] != 24 {
		t.Fatalf("bip85WordChoices = %v, want [12 18 24]", bip85WordChoices)
	}
	if len(bip85IndexChoices) != 10 {
		t.Fatalf("bip85IndexChoices len = %d, want 10 (0..9)", len(bip85IndexChoices))
	}
	for i, v := range bip85IndexChoices {
		if v != i {
			t.Fatalf("bip85IndexChoices[%d] = %d, want %d", i, v, i)
		}
	}
	// Every advertised (words,index) pair derives a valid child (no panic, no error).
	for _, w := range bip85WordChoices {
		for _, idx := range bip85IndexChoices {
			child, err := deriveBip85Child(abandonAboutMnemonic(), "", w, idx)
			if err != nil {
				t.Fatalf("words=%d idx=%d: %v", w, idx, err)
			}
			if len(child) != w || !child.Valid() {
				t.Fatalf("words=%d idx=%d: bad child (%d words, valid=%v)", w, idx, len(child), child.Valid())
			}
		}
	}
}
```

- [ ] **Step 3.2: Run to verify it fails.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestBip85ParamBounds' -v`
Expected: build failure / FAIL — `undefined: bip85WordChoices` (and `bip85IndexChoices`).

- [ ] **Step 3.3: Write the minimal implementation.**

Add to `gui/bip85.go`:

```go
// bip85WordChoices / bip85IndexChoices are the picker's in-spec, validated-by-
// construction bounds (R0-I-B): word count = biptool's {12,18,24}; index is a
// bounded small set 0..9 (no free-form numeric entry — there is no reusable
// numeric-entry widget; a larger index space is a FOLLOWUP). The application is
// FIXED to BIP-39 (the only engrave-as-words-faithful BIP-85 app).
var bip85WordChoices = []int{12, 18, 24}
var bip85IndexChoices = []int{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}

// bip85ParamPickFlow picks the child BIP-39 word count then the bounded index.
// Returns ok==false on Back from the FIRST screen; Back from the index screen
// re-shows the word-count screen. The returned (words,index) are always in-spec.
func bip85ParamPickFlow(ctx *Context, th *Colors) (words, index int, ok bool) {
	wordCS := &ChoiceScreen{
		Title:   "Child Seed",
		Lead:    "Child word count",
		Choices: []string{"12 WORDS", "18 WORDS", "24 WORDS"},
	}
	for {
		wsel, wok := wordCS.Choose(ctx, th)
		if !wok {
			return 0, 0, false
		}
		idxCS := &ChoiceScreen{
			Title:   "Child Seed",
			Lead:    "Child index",
			Choices: []string{"0", "1", "2", "3", "4", "5", "6", "7", "8", "9"},
		}
		isel, iok := idxCS.Choose(ctx, th)
		if !iok {
			continue // Back from index -> re-pick the word count.
		}
		return bip85WordChoices[wsel], bip85IndexChoices[isel], true
	}
}
```

- [ ] **Step 3.4: Run to verify it passes.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestBip85ParamBounds' -v`
Expected: PASS.

- [ ] **Step 3.5: Commit.**

```bash
cd /scratch/code/shibboleth/seedhammer-t7b
git add gui/bip85.go gui/bip85_test.go
git commit -S -s -m "feat(gui): add validated-by-construction BIP-85 child-param picker (T7b)

App fixed to BIP-39; word-count ChoiceScreen {12,18,24}; bounded index
ChoiceScreen {0..9} default 0. No free-form numeric entry — the bounds
cannot mint an out-of-spec child. Larger index space is a FOLLOWUP.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4 — the unskippable child-seed warning (`childSeedWarning`)

**Files:**
- Modify: `gui/bip85.go`
- Test: `gui/bip85_test.go`

**Interfaces:**
- Consumes (production): `ConfirmWarningScreen`, `ConfirmNo`/`ConfirmYes`, `assets.IconHammer`.
- Consumes (test harness): `testing/synctest` (`synctest.Test`), `runUI` (`gui_test.go:467`), `pumpUntil` (`slip39_polish_test.go:329`), `click` (`event_test.go:42`).
- Produces: `childSeedWarning(ctx *Context, th *Colors) bool` — shows the MANDATORY hold-to-confirm warning that this engraves a CHILD SEED; returns `true` only on a held confirm, `false` on Back/cancel/Done. Mirrors `stubZeroWarning` (`gui/derive_xpub.go:237`).

- [ ] **Step 4.1: Write the failing test.**

Append to `gui/bip85_test.go`. This drives the warning screen through the UI harness; a Back (Button1) must drive `ConfirmWarningScreen.Layout` to `ConfirmNo`, so `childSeedWarning` returns `false` (abort) and the flow goroutine returns.

This test mirrors `TestDescriptorAddressFlowBackExits` (`gui/address_polish_test.go:77-92`): keep the `frame` handle, render the warning, `click(Button1)` to abort, then pump `frame()` until the flow goroutine returns (the iterator ends), then assert the captured result is `false` NON-vacuously (the goroutine actually ran `childSeedWarning` to completion and returned `false`). Because `childSeedWarning` only advances on `ctx.Frame` yields, the `frame()` pumping is REQUIRED — a bare `click()` without pumping would leave the warning goroutine blocked on its first yield and the test would pass vacuously. Wrapped in `synctest.Test` to match every shipped flow test.

```go
// TestChildSeedWarningAbort: pressing Back (Button1) at the child-seed warning
// drives ConfirmWarningScreen.Layout -> ConfirmNo, so childSeedWarning returns
// false (abort) and no engrave proceeds. The flow goroutine must actually reach
// and dismiss the warning (NON-vacuous): we keep the frame handle, render the
// warning, click Back, pump frames until the goroutine returns, then assert it
// returned false and that it ran to completion. Mirrors TestDescriptorAddressFlowBackExits.
func TestChildSeedWarningAbort(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		ctx := NewContext(newPlatform())
		var got bool
		done := false
		frame, quit := runUI(ctx, func() {
			got = childSeedWarning(ctx, &descriptorTheme)
			done = true
		})
		defer quit()
		// Render the warning before driving it (the goroutine blocks on its first
		// ctx.Frame yield until pumped).
		if c, ok := pumpUntil(frame, "Child Seed", 16); !ok {
			t.Fatalf("child-seed warning not shown; got %q", c)
		}
		click(&ctx.Router, Button1) // Back -> ConfirmNo
		// Pump until the warning goroutine returns (the iterator ends).
		for i := 0; i < 16 && !done; i++ {
			frame()
		}
		if !done {
			t.Fatal("childSeedWarning did not return after Back")
		}
		if got {
			t.Fatal("childSeedWarning returned true after Back; want false (abort)")
		}
	})
}
```

**Helpers/imports for this test:** `synctest.Test` (`testing/synctest`), `pumpUntil` (`gui/slip39_polish_test.go:329`), `runUI`/`click`/`Button1`/`NewContext`/`newPlatform`/`descriptorTheme` (all existing in the `gui` test package). Add `"testing/synctest"` to `gui/bip85_test.go`'s imports.

- [ ] **Step 4.2: Run to verify it fails.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestChildSeedWarningAbort' -v`
Expected: build failure / FAIL — `undefined: childSeedWarning`.

- [ ] **Step 4.3: Write the minimal implementation.**

Add `"seedhammer.com/gui/assets"` and `"seedhammer.com/gui/op"` to `gui/bip85.go`'s imports, then add:

```go
// childSeedWarning shows the MANDATORY, operator-acknowledged warning that the
// flow is about to engrave a CHILD SEED — anyone with the child mnemonic controls
// the child wallet, so engrave onto YOUR OWN steel only. Hold to confirm; Back
// cancels. Returns true only on an acknowledged confirm. Mirrors stubZeroWarning.
func childSeedWarning(ctx *Context, th *Colors) bool {
	warn := &ConfirmWarningScreen{
		Title: "Child Seed",
		Body: "This engraves a NEW CHILD SEED derived from your master. Anyone holding " +
			"these words controls the child wallet — engrave onto your OWN steel only.\n\n" +
			"Hold button to confirm.",
		Icon: assets.IconHammer,
	}
	for !ctx.Done {
		dims := ctx.Platform.DisplaySize()
		d, res := warn.Layout(ctx, th, dims)
		switch res {
		case ConfirmNo:
			return false
		case ConfirmYes:
			return true
		}
		ctx.Frame(op.Layer(d, op.Color(&ctx.B, th.Background)))
	}
	return false
}
```

The full import block of `gui/bip85.go` is now:

```go
import (
	"errors"
	"fmt"

	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip39"
	"seedhammer.com/bip85"
	"seedhammer.com/engrave"
	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/op"
)
```

- [ ] **Step 4.4: Run to verify it passes.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestChildSeedWarningAbort' -v`
Expected: PASS.

- [ ] **Step 4.5: Commit.**

```bash
cd /scratch/code/shibboleth/seedhammer-t7b
git add gui/bip85.go gui/bip85_test.go
git commit -S -s -m "feat(gui): add unskippable BIP-85 child-seed warning (T7b)

Hold-to-confirm warning before engrave (mirrors the stub-0 warning); Back
aborts. Anyone with the child mnemonic controls the child wallet.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 5 — the `bip85DeriveFlow` orchestrator + scrub

**Files:**
- Modify: `gui/bip85.go`
- Test: `gui/bip85_test.go`

**Interfaces:**
- Consumes (production): `seedEntryFlow`, `passphraseFlow`, `ChoiceScreen`, `bip85ParamPickFlow` (Task 3), `deriveBip85Child` (Task 1), `childSeedWarning` (Task 4), `engraveBip85Child` (Task 2), `NewEngraveScreen(...).Engrave`, `showError`, `engraveTheme`.
- Consumes (test harness): `testing/synctest` (`synctest.Test`/`synctest.Wait`), `time`+`confirmDelay` (`gui/gui.go:269`), `runUI` (`gui_test.go:467`), `pumpUntil` (`slip39_polish_test.go:329`), `chooseEntry` (`derive_xpub_test.go:21`), `driveWords` (`seedxor_polish_test.go:24`), `abandonAboutPhrase` (`derive_xpub_test.go:14`), `click`/`press` (`event_test.go:42,57`), `newEngraver`/`newPlatform` (+`p.engraver`/`p.wakeups`/`e.closes`, `gui_test.go:336-337,452-465,242-275`).
- Produces:
  - `var bip85SeedHook func(master, child bip39.Mnemonic)` — test-only seam (nil in production) to observe BOTH mnemonics for the scrub assertion. Mirrors `singleSigSeedHook` (`gui/singlesig.go:28`).
  - `bip85DeriveFlow(ctx *Context, th *Colors)` — the program entry point: typed master (`seedEntryFlow`, NEVER scan) → optional passphrase → param picker → derive → warning → engrave. A top-level `defer` scrubs BOTH the master AND the child mnemonic on every exit. Dispatched from the carousel in Task 6.

- [ ] **Step 5.1: Write the failing scrub test.**

Append to `gui/bip85_test.go`. This MIRRORS `TestEngraveSingleSigFlowSeedScrubbed` (`gui/singlesig_flow_test.go:117-154`) EXACTLY for the structure: `synctest.Test` wrapper, `frame, quit := runUI(...)` (keep the `frame` handle), install the seed hook, type the abandon master, drive the param pickers, confirm the warning, let the engrave job complete, drain frames until the flow goroutine returns, then assert BOTH captured slices are zeroed. The engrave-completion portion mirrors `TestEngraveScreen` (`gui/gui_test.go:242-275`): install a `testEngraver`, click to the connect step, `press(Button3)` + `time.Sleep(confirmDelay)` to hold-confirm, pump until `<-e.closes`, then `click(Button3)` + `synctest.Wait()` so `NewEngraveScreen(...).Engrave` returns `true` and the flow returns.

The hook captures the slice HEADERS of the master and child while they are still non-nil (the hook fires synchronously right after `child = c`, see Step 5.3). The flow's top-level `defer` then zeroes those same backing arrays on return; the test reads their CONTENTS after the goroutine has returned (after the final drain), by which point the scrub defer has run.

```go
// TestBip85DeriveFlow_ScrubsBothMnemonics drives the FULL flow: type the abandon
// master, pick the child params (12 words, index 0), confirm the child-seed
// warning, and let the engrave complete; then it asserts BOTH the master and the
// derived child mnemonic []Word slices are zeroed on exit (I-3: two secrets to
// scrub). Mirrors TestEngraveSingleSigFlowSeedScrubbed (the seed-hook + zeroed-
// slice pattern) plus TestEngraveScreen (the connect/hold-confirm/complete dance).
func TestBip85DeriveFlow_ScrubsBothMnemonics(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		var master, child bip39.Mnemonic
		bip85SeedHook = func(m, c bip39.Mnemonic) { master, child = m, c }
		defer func() { bip85SeedHook = nil }()

		e := newEngraver()
		p := newPlatform()
		p.engraver = e
		ctx := NewContext(p)
		done := false
		frame, quit := runUI(ctx, func() {
			bip85DeriveFlow(ctx, &descriptorTheme)
			done = true
		})
		defer quit()
		frame()

		// Master entry: word-count picker -> 12 words (choice 0), then type the
		// abandon-about phrase. (seedEntryFlow's master count is []int{12,24};
		// default index 0 = 12 words, so confirm with Button3.)
		click(&ctx.Router, Button3) // 12 WORDS
		frame()
		driveWords(&ctx.Router, abandonAboutPhrase())
		// Passphrase prompt: Skip (choice 0).
		if c, ok := pumpUntil(frame, "Passphrase", 160); !ok {
			t.Fatalf("did not reach the passphrase prompt; got %q", c)
		}
		click(&ctx.Router, Button3) // Skip
		frame()
		// Param picker: word count = 12 (index 0), child index = 0 (index 0).
		// chooseEntry queues the Down presses, pumps a frame, confirms, pumps again.
		chooseEntry(frame, &ctx.Router, 0) // word count 12
		chooseEntry(frame, &ctx.Router, 0) // child index 0
		// Child-seed warning: hold Button3 to confirm (ConfirmYes).
		if c, ok := pumpUntil(frame, "Child Seed", 160); !ok {
			t.Fatalf("did not reach the child-seed warning; got %q", c)
		}
		press(&ctx.Router, Button3) // hold to confirm
		frame()
		time.Sleep(confirmDelay)
		frame()
		// Engrave screen: click to the connect step, hold to start engraving.
		click(&ctx.Router, Button3, Button3, Button3)
		press(&ctx.Router, Button3) // hold connect
		frame()
		time.Sleep(confirmDelay)
		// Pump until the engrave job closes (completes).
	loop:
		for {
			frame()
			select {
			case <-e.closes:
				break loop
			case <-p.wakeups:
			}
		}
		click(&ctx.Router, Button3) // dismiss the success screen -> Engrave returns true
		synctest.Wait()
		// Drain remaining frames until the flow goroutine returns and the scrub
		// defer has run.
		for i := 0; i < 32 && !done; i++ {
			frame()
		}
		if !done {
			t.Fatal("bip85DeriveFlow did not return after a completed engrave")
		}
		if master == nil || child == nil {
			t.Fatal("hook never observed both mnemonics")
		}
		for i, w := range master {
			if w != 0 {
				t.Fatalf("master[%d] = %d, not scrubbed on exit (I-3)", i, w)
			}
		}
		for i, w := range child {
			if w != 0 {
				t.Fatalf("child[%d] = %d, not scrubbed on exit (I-3)", i, w)
			}
		}
	})
}
```

**Helpers/imports for this test (all confirmed present in the `gui` test package):** `synctest.Test`/`synctest.Wait` (`testing/synctest`), `time.Sleep` + `confirmDelay` (`gui/gui.go:269`), `runUI` (`gui_test.go:467`), `pumpUntil` (`slip39_polish_test.go:329`), `chooseEntry(frame, &ctx.Router, down)` (`derive_xpub_test.go:21`), `driveWords` (`seedxor_polish_test.go:24`), `abandonAboutPhrase` (`derive_xpub_test.go:14`), `click`/`press` (`event_test.go:42,57`), `newEngraver`/`newPlatform` + `p.engraver`/`p.wakeups`/`e.closes` (`gui_test.go:336-337,452-465,242-275`), `descriptorTheme` (`theme.go:47`). Add `"testing/synctest"` and `"time"` to `gui/bip85_test.go`'s imports. **No new `chooseWords`/`clickChoose` wrappers are needed — `chooseEntry` is the shipped ChoiceScreen driver (m1).**

> NOTE for the implementer: if the harness sequence drifts (the flow times out or a `pumpUntil` misses its marker), re-read the two precedents — `gui/singlesig_flow_test.go:117-154` (seed-hook + zeroed-slice) and `gui/gui_test.go:242-275` (the engrave connect/hold/complete dance) — and align the click/`frame()` sequence to match the actual screen titles. Do NOT weaken the scrub assertion (the zeroed-slice checks) to make the test pass.
>
> ONE button-state subtlety: `press(r, Button3)` (`event_test.go:57`) sends ONLY a Pressed-down event (no release), so after the warning's `press(Button3)`+`time.Sleep(confirmDelay)` reaches `ConfirmYes`, Button3 is still logically held when the engrave screen opens. `TestEngraveScreen` has no preceding warning, so it starts from a released button. If the engrave-screen drive (`click(Button3, Button3, Button3)`) does not advance because the harness sees a still-held button, release it first with `click(&ctx.Router, Button3)` (a press/release pair re-establishes the transition) BEFORE the connect clicks, then proceed. This is a mechanical alignment, not a change to the scrub assertion.

- [ ] **Step 5.2: Run to verify it fails.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestBip85DeriveFlow_ScrubsBothMnemonics' -v`
Expected: build failure / FAIL — `undefined: bip85DeriveFlow` (and `bip85SeedHook`).

- [ ] **Step 5.3: Write the minimal implementation.**

Add to `gui/bip85.go`:

```go
// bip85SeedHook is a test-only seam to observe the master + child mnemonics (to
// assert both are scrubbed on exit, I-3). nil in production. Mirrors
// singleSigSeedHook.
var bip85SeedHook func(master, child bip39.Mnemonic)

// bip85DeriveFlow is the bip85Derive program: a hand-typed BIP-39 MASTER seed
// (SECRET, typed-only — NEVER a scan) + optional passphrase ON THE MASTER -> pick
// the child params (app fixed BIP-39, word count {12,18,24}, bounded index 0..9)
// -> derive the child BIP-39 mnemonic via BIP-85 -> unskippable child-seed warning
// -> engrave the child (words + standard SeedQR) via the engraveSeed primitive,
// stamping the CHILD's own bare fingerprint.
//
// SECURITY SPINE (mirror gui/singlesig.go):
//   - TYPED-ONLY master (I-3): from seedEntryFlow ONLY; never an NFC scan.
//   - TWO secrets scrubbed (I-3): the master AND the derived child mnemonic, both
//     []Word zeroed on EVERY exit (derive/abort/warning-abort/engrave-abort/error).
//     The privkey serialization + HMAC output are wiped inside deriveBip85Child.
//   - Mainnet-only; child engraved onto owner-held steel only, never NFC.
func bip85DeriveFlow(ctx *Context, th *Colors) {
	// TYPED-ONLY master (never a scan).
	master, ok := seedEntryFlow(ctx, th)
	if !ok {
		return
	}
	var child bip39.Mnemonic
	// Scrub BOTH secrets on EVERY exit path (I-3). child is nil until derived.
	// This is the ONLY scrub defer and (being registered first) runs LAST/LIFO,
	// so it zeroes both backing arrays after every other defer. The test's
	// bip85SeedHook (called synchronously below, after child = c) holds the slice
	// headers and reads their contents AFTER the flow returns and this defer ran.
	defer func() {
		for i := range master {
			master[i] = 0
		}
		for i := range child {
			child[i] = 0
		}
	}()

	// Optional passphrase ON THE MASTER.
	passphrase := ""
	ppChoice := &ChoiceScreen{Title: "Passphrase", Lead: "Add a BIP-39 passphrase?", Choices: []string{"Skip", "Add passphrase"}}
	if sel, ok := ppChoice.Choose(ctx, th); ok && sel == 1 {
		if pass, ok := passphraseFlow(ctx, th); ok {
			passphrase = pass
		}
	}

	for {
		words, index, ok := bip85ParamPickFlow(ctx, th)
		if !ok {
			return
		}
		c, err := deriveBip85Child(master, passphrase, words, index)
		if err != nil {
			showError(ctx, th, "BIP-85 Child", "Couldn't derive the child seed.")
			continue
		}
		child = c
		// Test-only seam: observe BOTH mnemonics synchronously while they are
		// non-nil. nil in production. The captured slice headers alias the backing
		// arrays the top-level scrub defer zeroes on exit (mirrors
		// singleSigSeedHook, gui/singlesig.go:36-38 — observed-then-scrubbed).
		if bip85SeedHook != nil {
			bip85SeedHook(master, child)
		}

		// Unskippable child-seed warning before any engrave.
		if !childSeedWarning(ctx, th) {
			// Abort: scrub this child immediately and re-pick params.
			for i := range child {
				child[i] = 0
			}
			child = nil
			continue
		}

		plate, _, err := engraveBip85Child(ctx.Platform.EngraverParams(), child)
		if err != nil {
			showError(ctx, th, "BIP-85 Child", "Couldn't build the child seed plate.")
			for i := range child {
				child[i] = 0
			}
			child = nil
			continue
		}
		if NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme) {
			return
		}
		// Engrave backed out -> re-pick params (scrub this child first).
		for i := range child {
			child[i] = 0
		}
		child = nil
	}
}
```

> NOTE on the hook ordering (the design above already does this correctly): the `bip85SeedHook` is called SYNCHRONOUSLY right after `child = c` (NOT via a `defer`), and there is exactly ONE `defer` — the top-level scrub — registered first so it runs LAST/LIFO. The hook hands the test the master/child slice HEADERS while they are non-nil; the scrub defer then zeroes those same backing arrays on exit; the test reads the contents AFTER the flow goroutine returns (after the final `frame()` drain), by which point the scrub defer has run. This is exactly how `singleSigSeedHook` is used (`gui/singlesig.go:36-38`, observed-synchronously-then-scrubbed). A `defer bip85SeedHook(...)` would observe the slices BEFORE the scrub defer ran (LIFO) and defeat the test — do NOT use a defer for the hook.

- [ ] **Step 5.4: Run to verify it passes.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestBip85DeriveFlow_ScrubsBothMnemonics' -v`
Expected: PASS — the flow drives to a completed engrave, returns, and both the captured master and child slices are zeroed. If the test times out or the harness sequence is off, re-read the two precedents — `gui/singlesig_flow_test.go:117-154` (seed-hook + zeroed-slice) and `gui/gui_test.go:242-275` (engrave connect/hold/complete) — and align the `click`/`frame()` sequence to the actual screen titles; do NOT weaken the scrub assertion.

- [ ] **Step 5.5: Commit.**

```bash
cd /scratch/code/shibboleth/seedhammer-t7b
git add gui/bip85.go gui/bip85_test.go
git commit -S -s -m "feat(gui): add bip85DeriveFlow orchestrator with two-secret scrub (T7b)

Typed master -> optional passphrase -> param picker -> derive -> child-seed
warning -> engraveSeed. Top-level defer scrubs BOTH the master and the child
mnemonic on every exit; the privkey/HMAC buffers are wiped inside the helper.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 6 — program enum + 8-site lockstep + nav-tests

**Files:**
- Modify: `gui/gui.go` (8 sites)
- Modify: `gui/multisig_program_test.go` (repoint BOTH `Navigable` + `LeftWrap` wrap-boundary assertions)
- Modify: `gui/singlesig_program_test.go` (repoint ONLY `TestEngraveSingleSigLeftWrap`, one `"Multisig"`→`"BIP-85"`), `gui/bundle_program_test.go`, `gui/derive_xpub_program_test.go` (comment-only — their assertions stop short of the wrap boundary)
- Create: `gui/bip85_program_test.go`

**Interfaces:**
- Consumes: `bip85DeriveFlow` (Task 5).
- Produces: the `bip85Derive` program constant, wired across all 8 lockstep sites; the carousel now includes `bip85Derive` as the navigable upper bound, `qaProgram` stays non-navigable.

- [ ] **Step 6.1: Write the failing nav-tests.**

Create `gui/bip85_program_test.go` (mirrors `gui/multisig_program_test.go`):

```go
package gui

import "testing"

// TestBip85DeriveProgramNavigable asserts the new bip85Derive program is reachable
// by navigating Right past engraveMultisig, is the new navigable upper bound (a
// further Right wraps to backupWallet), has a NON-BLANK title, and does not panic
// on render (layoutMainPlates must have its case). qaProgram stays out.
func TestBip85DeriveProgramNavigable(t *testing.T) {
	ctx := NewContext(newPlatform())
	m := new(StartScreen)
	frame, quit := runUI(ctx, func() { m.Flow(ctx, &descriptorTheme) })
	defer quit()
	content, ok := frame()
	if !ok {
		t.Fatal("StartScreen produced no frame")
	}
	if !uiContains(content, "Backup Wallet") {
		t.Fatalf("initial program not Backup Wallet; got %q", content)
	}
	// Right x4 -> engraveMultisig.
	for i := 0; i < 4; i++ {
		click(&ctx.Router, Right)
		content, ok = frame()
		if !ok {
			t.Fatalf("no frame after Right #%d", i+1)
		}
	}
	if !uiContains(content, "Multisig") {
		t.Fatalf("engraveMultisig not reachable after 4 Rights; got %q", content)
	}
	// Right -> bip85Derive (the new upper bound), titled non-blank.
	click(&ctx.Router, Right)
	content, ok = frame()
	if !ok {
		t.Fatal("no frame after fifth Right")
	}
	if !uiContains(content, "BIP-85") {
		t.Fatalf("bip85Derive not reachable/titled after fifth Right; got %q", content)
	}
	// Right again wraps to backupWallet.
	click(&ctx.Router, Right)
	content, ok = frame()
	if !ok {
		t.Fatal("no frame after sixth Right")
	}
	if !uiContains(content, "Backup Wallet") {
		t.Fatalf("Right did not wrap to Backup Wallet; got %q", content)
	}
}

// TestBip85DeriveLeftWrap asserts Left from backupWallet wraps to bip85Derive (the
// new navigable upper bound).
func TestBip85DeriveLeftWrap(t *testing.T) {
	ctx := NewContext(newPlatform())
	m := new(StartScreen)
	frame, quit := runUI(ctx, func() { m.Flow(ctx, &descriptorTheme) })
	defer quit()
	if _, ok := frame(); !ok {
		t.Fatal("StartScreen produced no frame")
	}
	click(&ctx.Router, Left)
	content, ok := frame()
	if !ok {
		t.Fatal("no frame after Left")
	}
	if !uiContains(content, "BIP-85") {
		t.Fatalf("Left did not wrap to BIP-85; got %q", content)
	}
}
```

- [ ] **Step 6.2: Run to verify it fails.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestBip85Derive(ProgramNavigable|LeftWrap)' -v`
Expected: FAIL — Right past engraveMultisig wraps to backupWallet (the new program is not in the carousel yet), and the title is blank, so `uiContains(content, "BIP-85")` fails. (Build still succeeds — `bip85Derive` is added in Step 6.3.)

- [ ] **Step 6.3: Wire all 8 lockstep sites + dispatch.**

**Site 1 — enum const** (`gui/gui.go:147-154`). Insert `bip85Derive` between `engraveMultisig` and `qaProgram`:

```go
const (
	backupWallet program = iota
	engraveXpub
	engraveBundle
	engraveSingleSig
	engraveMultisig
	bip85Derive
	qaProgram
)
```

**Site 2 — dispatch switch** (`gui/gui.go:1492-1514`). Add the `case` (place it just before `case backupWallet:`):

```go
			case engraveMultisig:
				engraveMultisigFlow(ctx, th)
				continue
			case bip85Derive:
				bip85DeriveFlow(ctx, th)
				continue
			case backupWallet:
```

**Site 3 — left-wrap** (`gui/gui.go:1640-1643`):

```go
					m.prog--
					if m.prog < 0 {
						m.prog = bip85Derive
					}
```

**Site 4 — right-wrap** (`gui/gui.go:1644-1652`):

```go
					m.prog++
					if m.prog > bip85Derive {
						m.prog = 0
					}
```

**Site 5 — title switch** (`gui/gui.go:1666-1678`). Add the `case` after `case engraveMultisig:`:

```go
	case engraveMultisig:
		titleTxt = "Engrave Multisig"
	case bip85Derive:
		titleTxt = "BIP-85 Child Seed"
	}
```

**Site 6 — `npage`** (`gui/gui.go:1852`):

```go
	const npage = int(bip85Derive) + 1
```

**Site 7 — `layoutMainPlates`** (`gui/gui.go:1860-1867`). Add `bip85Derive` to the case:

```go
func layoutMainPlates(buf *op.Buffer, page program) (op.Op, image.Point) {
	switch page {
	case backupWallet, engraveXpub, engraveBundle, engraveSingleSig, engraveMultisig, bip85Derive:
		img := assets.Hammer
		o := op.Image(buf, img)
		return o, img.Bounds().Size()
	}
	panic("invalid page")
}
```

**Site 8 — `npages`** (`gui/gui.go:1871`):

```go
	const npages = int(bip85Derive) + 1
```

- [ ] **Step 6.4: Run the nav-tests to verify they pass.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestBip85Derive(ProgramNavigable|LeftWrap)' -v`
Expected: PASS — both new nav-tests.

- [ ] **Step 6.5: Run the prior-program nav-tests; observe the carousel-count failures.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestEngrave(SingleSig|Bundle|Multisig|Xpub)(ProgramNavigable|LeftWrap)' -v`
Expected: FAIL — `TestEngraveSingleSigLeftWrap` and `TestEngraveMultisigLeftWrap` assert Left-from-backupWallet wraps to `Multisig`, but it now wraps to `BIP-85`; `TestEngraveMultisigProgramNavigable` asserts a further Right from Multisig wraps to Backup Wallet, but it now reaches `BIP-85`.

- [ ] **Step 6.6: Repoint the prior-program nav-test assertions.**

In `gui/multisig_program_test.go`:

In `TestEngraveMultisigProgramNavigable`, change the final wrap assertion (the block after the comment `// Right again wraps to backupWallet.`) so that one more Right reaches `bip85Derive`, then a further Right wraps to Backup Wallet:

```go
	// Right -> bip85Derive (the navigable upper bound after T7b).
	click(&ctx.Router, Right)
	content, ok = frame()
	if !ok {
		t.Fatal("no frame after fifth Right")
	}
	if !uiContains(content, "BIP-85") {
		t.Fatalf("bip85Derive not reachable after fifth Right; got %q", content)
	}
	// Right again wraps to backupWallet.
	click(&ctx.Router, Right)
	content, ok = frame()
	if !ok {
		t.Fatal("no frame after sixth Right")
	}
	if !uiContains(content, "Backup Wallet") {
		t.Fatalf("Right did not wrap to Backup Wallet; got %q", content)
	}
```

In `TestEngraveMultisigLeftWrap`, change the wrap target from `Multisig` to `BIP-85`:

```go
	if !uiContains(content, "BIP-85") {
		t.Fatalf("Left did not wrap to BIP-85; got %q", content)
	}
```

In `gui/singlesig_program_test.go`, `TestEngraveSingleSigLeftWrap`, change the wrap target from `Multisig` to `BIP-85`:

```go
	if !uiContains(content, "BIP-85") {
		t.Fatalf("Left did not wrap to BIP-85; got %q", content)
	}
```

`gui/bundle_program_test.go` and `gui/derive_xpub_program_test.go` only assert reaching `Single-Sig`/`Bundle` (not the wrap boundary), so they still pass — but update the trailing comment in each that says the upper bound is `engraveMultisig` to read `bip85Derive` (comment-only; the assertions are unchanged). In `gui/derive_xpub_program_test.go` change the comment block at lines ~30-33 and in `gui/bundle_program_test.go` lines ~44-47 from naming `engraveMultisig` as the upper bound to `bip85Derive`.

- [ ] **Step 6.7: Run all program nav-tests + `TestAllocs` to verify green.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'Program|LeftWrap|TestAllocs' -v`
Expected: PASS — all nav-tests (`Xpub`, `Bundle`, `SingleSig`, `Multisig`, `Bip85Derive`) + `TestAllocs`.

- [ ] **Step 6.8: Commit.**

```bash
cd /scratch/code/shibboleth/seedhammer-t7b
git add gui/gui.go gui/bip85_program_test.go gui/multisig_program_test.go gui/singlesig_program_test.go gui/bundle_program_test.go gui/derive_xpub_program_test.go
git commit -S -s -m "feat(gui): wire bip85Derive program across the 8 lockstep sites (T7b)

Insert bip85Derive between engraveMultisig and the non-navigable qaProgram;
update enum/dispatch/wrap/title/npage/layoutMainPlates/npages; add 2 nav-tests
and repoint the prior programs' carousel-boundary assertions.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 7 — no-regression + fuzz + artifact grep

**Files:**
- Modify: `gui/bip85_test.go` (add the fuzz target)

**Interfaces:**
- Consumes: `deriveBip85Child` (Task 1).
- Produces: `FuzzDeriveBip85Child` — fuzzes the helper across word counts and indices (including out-of-spec) and asserts NO panic.

- [ ] **Step 7.1: Write the fuzz target.**

Append to `gui/bip85_test.go`:

```go
// FuzzDeriveBip85Child asserts the derive helper never panics across arbitrary
// word counts and indices (in-spec and out-of-spec). Out-of-spec inputs must
// return an error, never panic; the bip39.New bounds (16<=len<=32, len%4==0) and
// the bip85.Entropy 32-byte guard must hold for every in-spec path.
func FuzzDeriveBip85Child(f *testing.F) {
	f.Add(12, 0)
	f.Add(18, 5)
	f.Add(24, 9)
	f.Add(15, 0)  // out-of-spec word count
	f.Add(12, -1) // negative index
	f.Add(0, 0)
	f.Fuzz(func(t *testing.T, words, index int) {
		// Must not panic. Errors are fine for out-of-spec inputs.
		child, err := deriveBip85Child(abandonAboutMnemonic(), "", words, index)
		if err != nil {
			return
		}
		// On success the inputs were in-spec; the child must be valid.
		if !validBip85Words(words) || index < 0 {
			t.Fatalf("deriveBip85Child accepted out-of-spec words=%d index=%d", words, index)
		}
		if len(child) != words || !child.Valid() {
			t.Fatalf("words=%d index=%d: invalid child (%d words, valid=%v)", words, index, len(child), child.Valid())
		}
	})
}
```

- [ ] **Step 7.2: Run the fuzz target briefly.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'FuzzDeriveBip85Child' -fuzz 'FuzzDeriveBip85Child$' -fuzztime 20s -v`
Expected: PASS — `fuzz: elapsed: …, execs: …` then OK, no panic, no `t.Fatalf`. Then run the seed corpus only (no `-fuzz`) to keep it in the normal suite: `go test ./gui/ -run 'FuzzDeriveBip85Child' -v` → PASS.

- [ ] **Step 7.3: Grep the artifact for xprv/extended-key serialization.**

Confirm the engraved child artifact never serializes an extended key. The flow serializes only the EC privkey internally (scrubbed); the child plate is words + SeedQR. Run:

```bash
cd /scratch/code/shibboleth/seedhammer-t7b
export PATH=$PATH:/home/bcg/.local/go/bin
grep -nE '\.String\(\)|Neuter|xprv|tprv|NewExtendedKey' gui/bip85.go || echo "CLEAN: no extended-key serialization in gui/bip85.go"
```
Expected: `CLEAN: no extended-key serialization in gui/bip85.go` (the helper only calls `engraveSeed`/`masterFingerprintFor`/`bip85.Entropy`/`bip39.New`; no xprv/xpub is produced). If any match appears, it is a defect — the artifact must be words+SeedQR only.

- [ ] **Step 7.4: Run the full no-regression suite (the affected packages byte-unchanged).**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/... ./bip85/... ./bip39/... ./backup/... ./codex32/... ./mk/... ./md/...`
Expected: all `ok` (no FAIL). This confirms Backup Wallet (`engraveSeed`), T4/T6 flows, and the codecs are untouched (I-8). If any prior test fails, the change introduced a regression — STOP and investigate before committing.

- [ ] **Step 7.5: Run `go vet` on the new code.**

Run: `cd /scratch/code/shibboleth/seedhammer-t7b && export PATH=$PATH:/home/bcg/.local/go/bin && go vet ./gui/`
Expected: no output (clean).

- [ ] **Step 7.6: Commit.**

```bash
cd /scratch/code/shibboleth/seedhammer-t7b
git add gui/bip85_test.go
git commit -S -s -m "test(gui): fuzz the BIP-85 derive helper; no-regression sweep (T7b)

Fuzz deriveBip85Child across in/out-of-spec words+index (0 panics; out-of-spec
errors). Full suite + artifact xprv-grep confirm no regression and a
words+SeedQR-only artifact.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Self-Review

Run with fresh eyes against `SPEC_seedhammer_T7b_bip85_derive.md` §2/§5/§6.

### 1. Spec coverage (acceptance §5.1–§5.8, invariants I-1…I-8)

- **§5.1 / I-1 (spec-faithful derivation):** Task 1 — `TestDeriveBip85Child_AbandonGoldens` (12/18/24, LEADING `entLen`), `TestDeriveBip85Child_CanonicalVector` (the `girl mad pet…` / `6250b68d…` canonical vector). A trailing-bytes bug → different child → caught. ✓
- **§5.2 / I-1 (fully-hardened + exact path):** Task 1 — the path is `[]uint32{PathRoot, 39+h, 0+h, words+h, index+h}`, all ≥ `HardenedKeyStart`; `TestDeriveBip85Child_IndexVaries` proves the index is hardened-distinct; the goldens would break on a wrong app/lang. ✓
- **§5.3 / I-2 (picker bounds):** Task 3 — `TestBip85ParamBounds` pins {12,18,24} and {0..9}; the picker has no app choice (fixed BIP-39) and no free-form numeric entry. ✓
- **§5.4 / I-5 (engrave + child fp):** Task 2 — `TestEngraveBip85Child_UsesChildFP` asserts the plate fp == the child's own fp (`0x02e8bff2`) and != the master's (`0x73c5da0a`); engraves via `engraveSeed`; skips the passphrase-fp picker. ✓
- **§5.5 (warning gate):** Task 4 — `TestChildSeedWarningAbort`; Task 5 wires it before engrave and scrubs on abort. ✓
- **§5.6 / I-3 (security + scrub):** Task 5 — `TestBip85DeriveFlow_ScrubsBothMnemonics` (master AND child zeroed); Task 1 wipes the privkey + HMAC buffers; typed-only master via `seedEntryFlow` (no scan symbol in the flow). Task 7 greps the artifact clean of xprv. ✓
- **§5.7 / I-7 (program nav):** Task 6 — 2 new nav-tests + repointed prior counts; `TestAllocs` green (Step 6.7); `qaProgram` stays non-navigable (inserted before it). ✓
- **§5.8 / I-8 (no-regression + fuzz):** Task 7 — full-suite sweep (Backup Wallet/T4/T6/codecs) + `FuzzDeriveBip85Child` (0 panics). ✓
- **I-4 (channel):** child engraved via `engraveSeed`/`backup.EngraveSeed` (steel-only, same as Backup Wallet); Task 7 greps no xprv. ✓
- **I-6 (mainnet-only):** `deriveBip85Child` and `engraveBip85Child` hardcode `&chaincfg.MainNetParams`. ✓

No spec requirement is left without a task.

### 2. Placeholder scan

No `TBD`/`TODO`/"handle edge cases"/"similar to Task N"/bare prose-without-code. Every code step shows full code; every run step shows the exact command + expected FAIL/PASS. The one place that flags a subtlety (Task 5 hook-ordering) gives the EXACT corrected code (synchronous hook call after `child = c`), not a deferred placeholder.

### 3. Type / signature consistency

- `deriveBip85Child(m bip39.Mnemonic, passphrase string, words, index int) (bip39.Mnemonic, error)` — defined Task 1, consumed identically in Tasks 2/3/5/7. ✓
- `engraveBip85Child(params engrave.Params, child bip39.Mnemonic) (Plate, uint32, error)` — defined Task 2, consumed Task 5 (`plate, _, err := engraveBip85Child(ctx.Platform.EngraverParams(), child)`). ✓
- `bip85ParamPickFlow(ctx, th) (words, index int, ok bool)` — Task 3, consumed Task 5. ✓
- `childSeedWarning(ctx, th) bool` — Task 4, consumed Task 5. ✓
- `bip85DeriveFlow(ctx, th)` — Task 5, dispatched Task 6 (`case bip85Derive: bip85DeriveFlow(ctx, th)`). ✓
- `masterFingerprintFor(...) (uint32, error)` — two-value, err propagated in Task 2. ✓ `engraveSeed(params, m, mfp) (Plate, error)`, `Plate{Duration, Spline}` (Spline NOT a slice — never `len()`'d). ✓
- `bip85WordChoices`/`bip85IndexChoices`/`validBip85Words`/`bip85SeedHook` — consistent names across Tasks 1/3/5/7. ✓
- Imports per file: `gui/bip85.go` ends with `errors, fmt, hdkeychain, chaincfg/v2, bip39, bip85, engrave, gui/assets, gui/op` (built up across Tasks 1/2/4); `gui/bip85_test.go` needs `testing, testing/synctest, time, bip39, chaincfg/v2` (`testing/synctest` for the Task-4 + Task-5 flow-test wrappers, `time` for `time.Sleep(confirmDelay)` in the Task-5 engrave-completion drive). ✓

### Spec ambiguity resolved (flag for plan R0)

- **Engrave assertion granularity.** The spec §5.4 says "assert built from the CHILD mnemonic, not the master" and "plate's `MasterFingerprint` == …". The fork's `Plate.Spline` is an opaque `bspline.Curve` (a func), so the engraved PLATE cannot be byte-inspected for the words in a unit test. I resolved this by asserting on the engrave INPUTS: the test computes the child's own fp and the master's fp independently and asserts `engraveBip85Child` returns the child's fp (not the master's), plus a pinned concrete golden (`0x02e8bff2`). This is faithful to I-5 (the fp is what `engraveSeed` stamps at `gui.go:475`) without depending on opaque curve internals. **Plan-R0 should confirm this input-level assertion satisfies §5.4** — if R0 wants an on-plate words assertion, the only available seam is `engraveSeed`'s `err==nil` plus the fp, which this plan already covers; there is no public reader for the engraved words on a `Plate`.
- **Task 5 UI-harness click sequence.** The full drive (master entry → passphrase → param picker → warning hold → engrave complete → return) is built against TWO shipped precedents verbatim: `gui/singlesig_flow_test.go:117-154` (the `synctest.Test` wrapper + seed-hook + `frame`/`driveWords`/`pumpUntil` + zeroed-slice assertion) and `gui/gui_test.go:242-275` (the engrave connect/`press(Button3)`+`time.Sleep(confirmDelay)`/`<-e.closes`/`synctest.Wait` completion dance), using the shipped `chooseEntry` ChoiceScreen driver (no invented `chooseWords`/`clickChoose`). If the harness cannot deterministically reach a screen, the scrub assertion (zeroed slices) must NOT be weakened — instead align the `click`/`frame()` sequence to the actual screen titles. This is the trickiest mechanical risk and is flagged for plan-R0.

---

## Execution Handoff

**Plan complete and saved to `design/IMPLEMENTATION_PLAN_seedhammer_T7b_bip85_derive.md`.** Per project policy this plan must pass its own opus R0 gate to 0C/0I before implementation begins. After R0 GREEN, two execution options:

**1. Subagent-Driven (recommended)** — a single implementer subagent executes the GREEN plan task-by-task in the worktree (NOT parallel re-implementations), with review between tasks.

**2. Inline Execution** — execute tasks in-session via `superpowers:executing-plans` with checkpoints.

Either way: TDD strictly, signed+DCO commits, explicit-path staging, then the mandatory whole-diff adversarial execution review before merge.
