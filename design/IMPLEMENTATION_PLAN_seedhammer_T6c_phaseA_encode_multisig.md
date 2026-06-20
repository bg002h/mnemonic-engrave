# T6c Phase A — `md.EncodeMultisig` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. A *single* subagent executes the GREEN plan in the worktree (NOT parallel re-implementations); strict TDD; one independent adversarial execution review over the whole diff after Task 6.

**Goal:** Add a headless, byte-faithful multisig wallet-policy md1 assembler `md.EncodeMultisig` to the SeedHammer II fork — `sortedmulti(k, …)` under `wsh` / `sh(wsh(…))` / `sh(…)` — mirroring the shipped `md.EncodeSingleSig`, with no GUI/picker/warning (that is Phase B).

**Architecture:** `EncodeMultisig` is a ~120-LOC assembler. It accepts a `EncodeMultisigRequest` struct (explicit `OriginMode`), builds the canonical multi-key `*descriptor` literal (`tagWsh`/`tagSh` wrappers ⊃ `tagSortedMulti{k,[0..n-1]}`, N `idxPub` TLV entries, optional per-cosigner `idxFP`, shared or divergent `pathDecl`), and routes through the **already-shipped** `split(d)` pipeline (`encodePayload → canonicalize → writeNode → writeTLVSection → chunk`). The bit emitter (`writeNode case multiKeysBody`) and identity (`WalletPolicyId`/`computeEncodingID`) are descriptor-shape-agnostic and already byte-tested vs Rust — so this task adds **zero wire/identity code**, only an order-preserving assembler + its types. It additionally returns an ordering-verification handle `(out, stub, slots)` so a Phase-B caller can verify cosigner ordering vs a coordinator BEFORE steel.

**Tech Stack:** Go 1.26.4 (fork `seedhammer` @ `8eb51d7`, package `seedhammer.com/md`); golden generator `descriptor-mnemonic` `md` CLI v0.7.0 @ `c85cd49`. `export PATH=$PATH:/home/bcg/.local/go/bin`.

---

## Verified facts (do NOT re-derive — all probe-confirmed on `8eb51d7` + Rust `c85cd49`)

These were verified by running code during plan authoring. Trust them; do not re-litigate during implementation.

- **VF1 — NO encode-time key sort.** `canonicalize` renumbers `@N` to first-occurrence (document) order only; the AST this assembler builds emits indices `[0,1,…,n-1]` in cosigner-input order, so canonicalize is the **identity permutation**. `cosigners[i]` → placeholder `@i`, full stop (spec I1/I4). The only sorts in the encode path are TLV-index sorts (`sort by idx`), never key bytes. **Consequence:** different caller order → different (valid) `WalletPolicyId`. The assembler must NOT sort/reorder.
- **VF2 — the multi-key bit emitter is shipped + byte-identical to Rust.** `writeNode` `case multiKeysBody:` writes `(k-1)@5b, (len(indices)-1)@5b, then each idx@kiw`, with bounds guards. No new wire code needed (spec V1/I3).
- **VF3 — the bounds guards already live in `writeNode`/`writePathDecl` and surface through `split`.** Exact error vars + messages (file:line):
  - `errThresholdRange = errors.New("md: threshold k out of range 1..32")` — `md/encode.go:18`; returned by `writeNode` when `k<1||k>32`.
  - `errChildCount = errors.New("md: child count out of range 1..32")` — `md/encode.go:19`; returned when `len(indices)<1||len(indices)>32`.
  - `errKGreaterThanN = errors.New("md: threshold k greater than n")` — `md/md.go:24`; returned when `k>len(indices)`.
  - `errKeyCountRange = errors.New("md: key count n out of range 1..32")` — `md/encode.go:25`; returned by `writePathDecl` when `pathDecl.n<1||pathDecl.n>32`.
  - `errDivergentCount = errors.New("md: divergent path count != n")` — `md/encode.go:26`; returned by `writePathDecl` when `len(divergent)!=n`.
  - `errPathDeclNMismatch = errors.New("md: pathDecl.n != descriptor.n")` — `md/encode.go:27`; returned by `encodePayload` when `pathDecl.n != descriptor.n`.
  - `errOverrideOrder = errors.New("md: override order violation")` — `md/md.go:31`; returned by `writeTLVSection` when a TLV idx column is not strictly ascending.
  Because these surface through `split`, A6 asserts them via `errors.Is` on the shipped vars — the assembler does NOT redefine them. The assembler adds **only** the guards `split` cannot express (divergent mode requested with empty/`nil` per-cosigner origin; shared mode requested with empty shared origin; non-`sortedmulti` script enum — structurally impossible from the 3-value enum so it is a `default:` typed error).
- **VF4 — `kiw`/`n` lockstep (spec I7).** `kiw(pathDecl.n)` computes the index width; assembler MUST set `descriptor.n == pathDecl.n == n == len(Cosigners)`. The shipped `errPathDeclNMismatch` backs this.
- **VF5 — identity is n-generic; zero change (spec I6).** `WalletPolicyId(*descriptor) ([16]byte, error)` (`md/walletpolicyid.go:30`), `WalletPolicyIDStub(*descriptor) ([4]byte, error)` (`:106`, top-4 bytes of the id), `WalletPolicyIdChunks([]string) ([16]byte, error)` (`:119`), `WalletPolicyIDStubChunks([]string) ([4]byte, error)` (`:129`). **There is NO exported `Md1EncodingId`**; `computeEncodingID`/`deriveChunkSetID` are unexported. The Phase-A ordering handle is `WalletPolicyIDStub(d)` (computed inside `EncodeMultisig`, equals `WalletPolicyIDStubChunks(out)`).
- **VF6 — `PathComponent` ALREADY EXISTS** (exported) in `md/encode_singlesig.go:18` as `type PathComponent struct { Hardened bool; Value uint32 }` — RAW form (Hardened flag + bare value, NOT in-band `+HardenedKeyStart`). REUSE it; do NOT redefine.
- **VF7 — `MultisigScript` and `Md1EncodingId` do NOT exist** anywhere in `md/`. This plan DEFINES `MultisigScript` (3-value enum). It does NOT define any `Md1EncodingId`.
- **VF8 — T6b fixture is fp-ABSENT, shared origin, and re-encodes byte-for-byte.** `gui/testdata/t6b_multisig_full.md1.txt` is a 6-chunk 2-of-3 `wsh(sortedmulti)`, all 3 slots `FingerprintPresent=false`, shared origin `m/48'/0'/0'/2'`, multipath `<0;1>/*`, `WalletPolicyId = 7b716421db8b9f462967d04e0f8a3fd5`, stub `7b716421`. **PROVEN during plan authoring:** feeding the three exact 65-byte payloads below (in @0/@1/@2 order), k=2, shared origin `m/48'/0'/0'/2'`, `MultisigWsh`, fp-absent through `split` reproduces ALL 6 fixture chunks byte-for-byte and yields `WalletPolicyId=7b716421…`. The three payloads (hex `chainCode‖compressedPubkey`, 65 B each):
  - `@0 = 101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f03a9394a2f1a4f99613a716956c8540f6dba6f18931c2639107221b267d740af23`
  - `@1 = bba0c7ca160a870efeb940ab90d0f4284fea1b5e0d2117677e823fc37e2d5763021a3bf5fbf737d0f36993fd46dc4913093beb532d654fe0dfd98bd27585dc9f29`
  - `@2 = 101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5`
- **VF9 — the golden generator works & is non-circular; full-policy multisig requires a DEPTH-4 xpub.** `descriptor-mnemonic`'s `md encode` enforces xpub depth==4 for multisig (`md-cli/src/parse/keys.rs:67-77`). Working depth-4 xpub (abandon-seed @ `m/48'/0'/0'/2'`):
  `xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf`.
  Verified full-policy generation: `wsh` (`chunk_set_id 0x36d1b`), `sh(wsh)` (`0x58624`), `sh` (`0x90289`), fp-present `wsh` (`0x5b22c`).
- **VF10 — vendored multi byte-goldens.** Only `md/testdata/vectors/wsh_sortedmulti.bytes.hex = 2082001821c22180` carries `tagSortedMulti`; `wsh_multi_2of3 = 2082001821822180` is the same layout with `tagMulti` (differs in exactly one bit — the tag low bit). `sh_wsh_multi.bytes.hex = 2042001830860850` carries `tagSh⊃tagWsh⊃tagMulti`. There is **NO** vendored `sh(sortedmulti)` or `sh(wsh(sortedmulti))` golden → this plan vendors FRESH sortedmulti goldens for the two `sh` shapes (Task 3d).
- **VF11 — all three shapes encode AND round-trip with an explicit shared origin** (PROVEN in Go during authoring): `wsh` → decoded `Root=ScriptWsh, InnerWsh=false`; `sh(wsh)` → `Root=ScriptSh, InnerWsh=true`; `sh` → `Root=ScriptSh, InnerWsh=false`. **A bare `sh(sortedmulti)` template with NO explicit origin fails `Reassemble` with "missing explicit origin"** — therefore the `sh`/`sh-wsh` full-policy goldens are generated **in Go** with an explicit shared origin (which is exactly what `EncodeMultisig` always supplies, like single-sig), and vendored as `.bytes.hex` + `.md1.txt`. The Rust cross-check for the `sh`-wrapper layout is the vendored `sh_wsh_multi.bytes.hex` template (A1) — the wrappers are tag-only and shared with `tagMulti`.
- **VF12 — vendored-golden conventions.** Single-sig vectors use `<name>.meta.json` (chaincode/pubkey/fp/origin/script/payload_hex/wallet_policy_id/wallet_policy_id_stub) + `<name>.md1.txt` (the chunk strings, one per line). Template-only vectors use `<name>.bytes.hex` + `<name>.descriptor.json` + `<name>.phrase.txt`, registered in `md/testdata_test.go` `singleStringVectorNames`/`byteParityVectorNames`. This plan uses a multisig-specific `.meta.json` (`md/testdata/vectors/<name>.meta.json`) + `.md1.txt` for full-policy goldens (Tasks 3b/3d), and reuses the existing `wsh_sortedmulti.bytes.hex`/`sh_wsh_multi.bytes.hex` for template-parity (Task 3a). **Do NOT add the new multisig vectors to `singleStringVectorNames`/`byteParityVectorNames`** (those drive the n=1-or-template parity sweep; the multisig goldens have their own loader to avoid touching the shipped sweep).

---

## File-structure map

| Path | Action | Responsibility |
|------|--------|----------------|
| `md/encode_multisig.go` | **Create** | `MultisigScript` enum, `OriginMode` enum, `MultisigCosigner`, `EncodeMultisigRequest`, `SlotInfo`; `EncodeMultisig`; `multiSigTree` helper; the 3 new typed errors. ~150 LOC. |
| `md/encode_multisig_test.go` | **Create** | All acceptance tests A1–A6: type plumbing, template parity, full-policy parity, T6b byte-exact, round-trip+identity, fp-present/divergent, refuse-unsupported. Mirrors `encode_singlesig_test.go`. |
| `md/encode_multisig_fuzz_test.go` | **Create** | `FuzzEncodeMultisig` (A7). Mirrors `encode_singlesig_fuzz_test.go`. |
| `md/testdata/vectors/multisig_wsh_full.meta.json` | **Create** (Task 3b) | Fresh Go-generated full-policy `wsh(sortedmulti)` golden (k=2,n=3, three identical depth-4 xpubs, fp-absent, shared origin). |
| `md/testdata/vectors/multisig_wsh_full.md1.txt` | **Create** (Task 3b) | The 6 chunk strings for the above. |
| `md/testdata/vectors/multisig_sh_wsh_full.meta.json` + `.md1.txt` | **Create** (Task 3d) | Fresh `sh(wsh(sortedmulti))` full-policy golden. |
| `md/testdata/vectors/multisig_sh_full.meta.json` + `.md1.txt` | **Create** (Task 3d) | Fresh `sh(sortedmulti)` full-policy golden. |
| `md/testdata/vectors/multisig_wsh_fp.meta.json` + `.md1.txt` | **Create** (Task 4) | Fresh fp-PRESENT full-policy `wsh(sortedmulti)` golden. |
| `md/testdata/vectors/multisig_wsh_divergent.meta.json` + `.md1.txt` | **Create** (Task 4) | Fresh divergent-origin full-policy `wsh(sortedmulti)` golden. |
| `md/testdata/vectors/README_multisig.md` | **Create** (Task 3b) | Documents the depth-4 xpub + the exact generation commands (provenance, like `README_singlesig.md`). |

> **Golden provenance note (mandatory, applies to every vendored `.meta.json`/`.md1.txt`):** The full-policy `.md1.txt` chunk strings and `.meta.json.payload_hex`/`wallet_policy_id*` fields are produced by the **Go assembler under test fed Rust-validated inputs**, then cross-checked: (i) the bit layout is template-parity-checked against the Rust-sourced `wsh_sortedmulti.bytes.hex`/`sh_wsh_multi.bytes.hex` (A1, Task 3a); (ii) the per-key 65-byte payloads come from a depth-4 xpub that the Rust `md` CLI accepts (VF9). This is the same provenance pattern the shipped single-sig goldens use. The chunk strings are captured ONCE during golden generation (Task 3b/3d/4 step "generate the golden") and frozen; the equality tests then guard against drift.

---

## Task 0 — Worktree + baseline

**Files:** none (setup only).

- [ ] **Step 1: Create the worktree off `main` (`8eb51d7`)**

```bash
export PATH=$PATH:/home/bcg/.local/go/bin
cd /scratch/code/shibboleth/seedhammer
git worktree add -b feat/t6c-encode-multisig /scratch/code/shibboleth/seedhammer-wt-t6c main
cd /scratch/code/shibboleth/seedhammer-wt-t6c
git log -1 --format='%H %s'
```
Expected: HEAD prints `8eb51d7a24e6f8ab0b6641a27996e12e07a48322 Merge FOLLOWUPS burndown: …`.

> **All subsequent steps run in `/scratch/code/shibboleth/seedhammer-wt-t6c` with `export PATH=$PATH:/home/bcg/.local/go/bin`.**

- [ ] **Step 2: Baseline `go test ./md/...`**

Run: `cd /scratch/code/shibboleth/seedhammer-wt-t6c && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./md/...`
Expected: `ok  	seedhammer.com/md` (PASS). Confirms a green baseline before any change.

- [ ] **Step 3: Confirm no pre-existing multisig file**

Run: `ls md/encode_multisig* 2>&1 || echo NONE`
Expected: `NONE` (no matches) — this is a fresh file.

**Commit convention for every commit in this plan** (signed `-S`, DCO `-s`, explicit paths):
```bash
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "<subject>

<body>

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```
No `git add -A`; stage exact paths only.

---

## Task 1 — Types: enums + request/cosigner/slot structs

**Files:**
- Create: `md/encode_multisig.go`
- Test: `md/encode_multisig_test.go`

- [ ] **Step 1: Write the failing test (type plumbing)**

Create `md/encode_multisig_test.go`:

```go
package md

import (
	"encoding/hex"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"seedhammer.com/codex32"
)

// ─── T6c Phase A: EncodeMultisig — wallet-policy sortedmulti md1 ─────────────

// TestEncodeMultisigRequestPlumbing constructs a request and asserts the fields
// are wired through (compile-time + value checks on the public surface).
func TestEncodeMultisigRequestPlumbing(t *testing.T) {
	req := EncodeMultisigRequest{
		Cosigners: []MultisigCosigner{
			{Fingerprint: [4]byte{1, 2, 3, 4}, FpPresent: true},
			{Fingerprint: [4]byte{5, 6, 7, 8}, FpPresent: false},
		},
		K:            2,
		Script:       MultisigWsh,
		OriginMode:   OriginShared,
		SharedOrigin: []PathComponent{{Hardened: true, Value: 48}},
	}
	if len(req.Cosigners) != 2 || req.K != 2 {
		t.Fatalf("request fields not plumbed: %+v", req)
	}
	if req.Script != MultisigWsh || req.OriginMode != OriginShared {
		t.Fatalf("enum fields not plumbed: %+v", req)
	}
	// SlotInfo is the ordering-verification handle element.
	s := SlotInfo{Index: 1, Fingerprint: [4]byte{5, 6, 7, 8}, FpPresent: false}
	if s.Index != 1 || s.FpPresent {
		t.Fatalf("SlotInfo not plumbed: %+v", s)
	}
	// Enum identity: the three script wrappers + two origin modes are distinct.
	if MultisigWsh == MultisigShWsh || MultisigShWsh == MultisigSh {
		t.Fatal("MultisigScript values not distinct")
	}
	if OriginShared == OriginDivergent {
		t.Fatal("OriginMode values not distinct")
	}
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `go test ./md/ -run TestEncodeMultisigRequestPlumbing -v`
Expected: FAIL — compile error `undefined: EncodeMultisigRequest` (and the other new types).

- [ ] **Step 3: Write minimal implementation (types only)**

Create `md/encode_multisig.go`:

```go
package md

import "errors"

// ─── EncodeMultisig (T6c Phase A) — byte-faithful sortedmulti WALLET-POLICY md1 ─
//
// EncodeMultisig builds a wallet-policy *descriptor for a sortedmulti k-of-n
// multisig under one of three top-level wrappers (wsh / sh(wsh) / sh) and emits
// the CHUNKED md1 strings via the shipped split. It mirrors EncodeSingleSig: the
// caller supplies parsed PUBLIC key material (no secret bytes), and the wire +
// identity core (writeNode/canonicalize/WalletPolicyId) is reused UNCHANGED.
//
// ORDERING CONTRACT (load-bearing — read before calling): EncodeMultisig is
// EXACTLY order-preserving. Cosigners[i] is assigned placeholder @i; there is NO
// hidden key sort (canonicalize is the identity permutation for this AST). Two
// callers supplying the same N keys in DIFFERENT orders mint DIFFERENT, both
// valid, md1 cards with DIFFERENT WalletPolicyId — only the order matching the
// coordinator's policy binds. The caller (Phase B) owns coordinator-matching
// order. To let a caller verify ordering BEFORE engraving to steel, EncodeMultisig
// returns the assigned per-slot @N→fingerprint map and the 4-byte
// WalletPolicyIDStub (== WalletPolicyIDStubChunks(out)).

// MultisigScript selects the top-level wrapper over sortedmulti.
type MultisigScript int

const (
	MultisigWsh   MultisigScript = iota // wsh(sortedmulti(k,...))      → P2WSH
	MultisigShWsh                       // sh(wsh(sortedmulti(k,...)))  → P2SH-P2WSH
	MultisigSh                          // sh(sortedmulti(k,...))        → legacy P2SH
)

// OriginMode picks the BIP-32 origin declaration: a single shared origin for all
// cosigners (path_decl.Shared) or per-cosigner divergent origins
// (path_decl.Divergent, len == n). It is explicit so a nil/empty origin is never
// silently overloaded as the shared/divergent discriminant (R0 recommendation).
type OriginMode int

const (
	OriginShared    OriginMode = iota // all cosigners share SharedOrigin
	OriginDivergent                   // each cosigner uses its own Cosigner.Origin
)

// MultisigCosigner is one parsed PUBLIC cosigner key. ChainCode‖CompressedPubkey
// form the 65-byte Pubkeys TLV entry. Fingerprint is emitted only if FpPresent
// (the T6b card is fp-ABSENT, so an always-fp encoder would not byte-match it).
// Origin is the RAW BIP-32 origin used in OriginDivergent mode (ignored in
// OriginShared mode); RAW = Hardened flag + bare value, the PathComponent form.
type MultisigCosigner struct {
	ChainCode        [32]byte
	CompressedPubkey [33]byte
	Fingerprint      [4]byte
	FpPresent        bool
	Origin           []PathComponent
}

// EncodeMultisigRequest is the EncodeMultisig parameter struct. K is the
// threshold; n is len(Cosigners). The cosigner ORDER fixes @0..@{n-1}.
type EncodeMultisigRequest struct {
	Cosigners    []MultisigCosigner
	K            uint8
	Script       MultisigScript
	OriginMode   OriginMode
	SharedOrigin []PathComponent // used iff OriginMode == OriginShared
}

// SlotInfo is one entry of the ordering-verification handle returned by
// EncodeMultisig: it records which placeholder index a cosigner was assigned and
// that cosigner's fingerprint (so a caller can match @N against a coordinator).
type SlotInfo struct {
	Index       uint8
	Fingerprint [4]byte
	FpPresent   bool
}

var (
	errMultisigEmptySharedOrigin = errors.New("md: EncodeMultisig OriginShared requires a non-empty SharedOrigin")
	errMultisigEmptyDivergent    = errors.New("md: EncodeMultisig OriginDivergent requires a non-empty Origin for every cosigner")
	errMultisigBadScript         = errors.New("md: EncodeMultisig unknown script kind")
)
```

- [ ] **Step 4: Run test to verify it passes**

Run: `go test ./md/ -run TestEncodeMultisigRequestPlumbing -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add md/encode_multisig.go md/encode_multisig_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "feat(md): EncodeMultisig types (T6c Phase A Task 1)

Add MultisigScript/OriginMode enums, MultisigCosigner,
EncodeMultisigRequest, SlotInfo, and the three assembler-level
typed errors. Types only; EncodeMultisig follows in Task 2.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2 — `EncodeMultisig` + `multiSigTree` (the assembler)

**Files:**
- Modify: `md/encode_multisig.go` (append the function + helper)
- Test: `md/encode_multisig_test.go` (append a minimal end-to-end smoke test)

- [ ] **Step 1: Write the failing test (encode smoke + ordering handle)**

Append to `md/encode_multisig_test.go`:

```go
// mkXpub65 builds a 65-byte chainCode‖compressedPubkey from two hex strings.
func mkXpub65(t *testing.T, ccHex, pkHex string) (cc [32]byte, pk [33]byte) {
	t.Helper()
	ccb, err := hex.DecodeString(ccHex)
	if err != nil || len(ccb) != 32 {
		t.Fatalf("bad chaincode %q", ccHex)
	}
	pkb, err := hex.DecodeString(pkHex)
	if err != nil || len(pkb) != 33 {
		t.Fatalf("bad pubkey %q", pkHex)
	}
	copy(cc[:], ccb)
	copy(pk[:], pkb)
	return
}

// sharedOrigin4828 is m/48'/0'/0'/2' as RAW PathComponents (the T6b origin).
func sharedOrigin4828() []PathComponent {
	return []PathComponent{
		{Hardened: true, Value: 48}, {Hardened: true, Value: 0},
		{Hardened: true, Value: 0}, {Hardened: true, Value: 2},
	}
}

// TestEncodeMultisigSmoke: a 2-of-3 wsh(sortedmulti) over three distinct keys
// encodes to >=2 chunks, the returned stub == WalletPolicyIDStubChunks(out), and
// the slots reflect cosigner order with the right fp-presence.
func TestEncodeMultisigSmoke(t *testing.T) {
	cc, pk := mkXpub65(t, "101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f", "03a9394a2f1a4f99613a716956c8540f6dba6f18931c2639107221b267d740af23")
	cc2, pk2 := mkXpub65(t, "101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f", "02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5")
	req := EncodeMultisigRequest{
		Cosigners: []MultisigCosigner{
			{ChainCode: cc, CompressedPubkey: pk},
			{ChainCode: cc2, CompressedPubkey: pk2, Fingerprint: [4]byte{0xde, 0xad, 0xbe, 0xef}, FpPresent: true},
			{ChainCode: cc, CompressedPubkey: pk2},
		},
		K:            2,
		Script:       MultisigWsh,
		OriginMode:   OriginShared,
		SharedOrigin: sharedOrigin4828(),
	}
	out, stub, slots, err := EncodeMultisig(req)
	if err != nil {
		t.Fatalf("EncodeMultisig: %v", err)
	}
	if len(out) < 2 {
		t.Fatalf("want >=2 chunks, got %d", len(out))
	}
	for _, s := range out {
		if !codex32.ValidMD(s) {
			t.Fatalf("chunk not ValidMD: %s", s)
		}
	}
	wantStub, err := WalletPolicyIDStubChunks(out)
	if err != nil {
		t.Fatalf("WalletPolicyIDStubChunks: %v", err)
	}
	if stub != wantStub {
		t.Fatalf("returned stub %x != WalletPolicyIDStubChunks(out) %x", stub, wantStub)
	}
	if len(slots) != 3 {
		t.Fatalf("want 3 slots, got %d", len(slots))
	}
	for i, s := range slots {
		if int(s.Index) != i {
			t.Fatalf("slot %d Index = %d, want %d (order-preserving)", i, s.Index, i)
		}
	}
	if !slots[1].FpPresent || slots[1].Fingerprint != [4]byte{0xde, 0xad, 0xbe, 0xef} {
		t.Fatalf("slot 1 fp not plumbed: %+v", slots[1])
	}
	if slots[0].FpPresent {
		t.Fatalf("slot 0 should be fp-absent")
	}
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `go test ./md/ -run TestEncodeMultisigSmoke -v`
Expected: FAIL — compile error `undefined: EncodeMultisig`.

- [ ] **Step 3: Write minimal implementation (append to `md/encode_multisig.go`)**

```go
// EncodeMultisig assembles a sortedmulti k-of-n wallet-policy md1 over the given
// cosigners in CALLER ORDER (which fixes @0..@{n-1}; see the ordering contract on
// the package doc above). It returns the chunked md1 strings (>=2), the 4-byte
// WalletPolicyIDStub, and the per-slot @N→fingerprint map (SlotInfo), plus an
// error. It refuses unsupported shapes/params via typed errors (k/n bounds and
// k<=n are enforced by the shipped split pipeline; this function adds the
// origin-mode and script-kind guards).
func EncodeMultisig(req EncodeMultisigRequest) (out []string, stub [4]byte, slots []SlotInfo, err error) {
	n := len(req.Cosigners)

	// Build the path declaration per the EXPLICIT origin mode.
	var pd pathDecl
	switch req.OriginMode {
	case OriginShared:
		if len(req.SharedOrigin) == 0 {
			return nil, [4]byte{}, nil, errMultisigEmptySharedOrigin
		}
		so := originPath{components: toComponents(req.SharedOrigin)}
		pd = pathDecl{n: uint8(n), shared: &so}
	case OriginDivergent:
		paths := make([]originPath, n)
		for i, c := range req.Cosigners {
			if len(c.Origin) == 0 {
				return nil, [4]byte{}, nil, errMultisigEmptyDivergent
			}
			paths[i] = originPath{components: toComponents(c.Origin)}
		}
		pd = pathDecl{n: uint8(n), divergent: paths}
	default:
		return nil, [4]byte{}, nil, errMultisigBadScript
	}

	// The multisig tree per wrapper (sortedmulti{k, [0..n-1]} in cosigner order).
	tree, terr := multiSigTree(req.Script, req.K, n)
	if terr != nil {
		return nil, [4]byte{}, nil, terr
	}

	// N pubkey TLV entries (idx-ascending, cosigner order) + optional per-cosigner
	// fingerprint entries (only the present subset, idx-ascending).
	pubkeys := make([]idxPub, n)
	var fps []idxFP
	slots = make([]SlotInfo, n)
	for i, c := range req.Cosigners {
		var xpub [65]byte
		copy(xpub[:32], c.ChainCode[:])
		copy(xpub[32:], c.CompressedPubkey[:])
		pubkeys[i] = idxPub{idx: uint8(i), xpub: xpub}
		if c.FpPresent {
			fps = append(fps, idxFP{idx: uint8(i), fp: c.Fingerprint})
		}
		slots[i] = SlotInfo{Index: uint8(i), Fingerprint: c.Fingerprint, FpPresent: c.FpPresent}
	}

	d := &descriptor{
		n:        uint8(n),
		pathDecl: pd,
		// useSite = <0;1>/* — hasMultipath, alts {0},{1}, unhardened wildcard.
		useSite: useSitePath{
			hasMultipath:     true,
			multipath:        []alternative{{hardened: false, value: 0}, {hardened: false, value: 1}},
			wildcardHardened: false,
		},
		tree: tree,
		tlv: tlvSection{
			pubPresent: true,
			pubkeys:    pubkeys,
			fpPresent:  len(fps) > 0,
			fingerprints: fps,
		},
	}

	out, err = split(d)
	if err != nil {
		return nil, [4]byte{}, nil, err
	}
	stub, err = WalletPolicyIDStub(d)
	if err != nil {
		return nil, [4]byte{}, nil, err
	}
	return out, stub, slots, nil
}

// toComponents converts the public RAW []PathComponent into the internal
// []pathComponent (same shape; Hardened/Value → hardened/value).
func toComponents(in []PathComponent) []pathComponent {
	out := make([]pathComponent, len(in))
	for i, c := range in {
		out[i] = pathComponent{hardened: c.Hardened, value: c.Value}
	}
	return out
}

// multiSigTree returns the wallet-policy tree for the three sortedmulti wrappers,
// each wrapping sortedmulti{k, [0..n-1]} (indices in cosigner order):
//
//	MultisigWsh   -> node{tagWsh, [node{tagSortedMulti, multiKeysBody{k,[0..n-1]}}]}
//	MultisigShWsh -> node{tagSh,  [node{tagWsh, [node{tagSortedMulti, ...}]}]}
//	MultisigSh    -> node{tagSh,  [node{tagSortedMulti, ...}]}
//
// k/n bounds (k,n in 1..32, k<=n) are enforced downstream by writeNode's
// multiKeysBody guards (errThresholdRange/errChildCount/errKGreaterThanN); this
// helper only fixes the wrapper shape and rejects an unknown script kind.
func multiSigTree(script MultisigScript, k uint8, n int) (node, error) {
	indices := make([]uint8, n)
	for i := range indices {
		indices[i] = uint8(i)
	}
	sm := node{tag: tagSortedMulti, body: multiKeysBody{k: k, indices: indices}}
	switch script {
	case MultisigWsh:
		return node{tag: tagWsh, body: childrenBody{children: []node{sm}}}, nil
	case MultisigShWsh:
		inner := node{tag: tagWsh, body: childrenBody{children: []node{sm}}}
		return node{tag: tagSh, body: childrenBody{children: []node{inner}}}, nil
	case MultisigSh:
		return node{tag: tagSh, body: childrenBody{children: []node{sm}}}, nil
	default:
		return node{}, errMultisigBadScript
	}
}
```

> **Note on `n` when `len(Cosigners)==0`:** `pathDecl{n:0}` → `writePathDecl` returns `errKeyCountRange` through `split` (VF3); `EncodeMultisig` surfaces it. No extra guard needed.

- [ ] **Step 4: Run test to verify it passes**

Run: `go test ./md/ -run TestEncodeMultisigSmoke -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add md/encode_multisig.go md/encode_multisig_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "feat(md): EncodeMultisig assembler + multiSigTree (T6c Phase A Task 2)

Order-preserving sortedmulti assembler over wsh/sh(wsh)/sh wrappers;
fills N pubkey TLVs + optional per-cosigner fp; shared/divergent origin
per explicit OriginMode; routes through the shipped split. Returns
(out, WalletPolicyIDStub, slots) for pre-steel ordering verification.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3 — Byte-exact goldens (the load-bearing test)

**Files:**
- Modify: `md/encode_multisig_test.go` (add vector loaders + A1/A2/A3/A3d tests)
- Create: `md/testdata/vectors/multisig_wsh_full.meta.json`, `multisig_wsh_full.md1.txt`, `multisig_sh_wsh_full.meta.json`, `multisig_sh_wsh_full.md1.txt`, `multisig_sh_full.meta.json`, `multisig_sh_full.md1.txt`, `README_multisig.md`

### Task 3a — template-parity vs the vendored Rust goldens (A1)

- [ ] **Step 1: Write the failing test**

Append to `md/encode_multisig_test.go`:

```go
// TestEncodeMultisigTemplateParity (A1): the bare sortedmulti AST for the wsh and
// sh(wsh) wrappers at n=3 encodes to the SAME bit layout as the Rust-sourced
// vendored template goldens. wsh_sortedmulti.bytes.hex carries tagSortedMulti
// directly; sh_wsh_multi.bytes.hex carries tagSh⊃tagWsh⊃tagMulti (tag-only
// wrapper, identical layout to sortedmulti per VF10/VF2) — we assert the WRAPPER
// bytes match by building a tagMulti-bodied tree for the sh(wsh) parity leg.
func TestEncodeMultisigTemplateParity(t *testing.T) {
	mkTree := func(rootTag tag, innerWsh bool, multiTag tag) node {
		mk := node{tag: multiTag, body: multiKeysBody{k: 2, indices: []uint8{0, 1, 2}}}
		switch {
		case rootTag == tagWsh:
			return node{tag: tagWsh, body: childrenBody{children: []node{mk}}}
		case rootTag == tagSh && innerWsh:
			inner := node{tag: tagWsh, body: childrenBody{children: []node{mk}}}
			return node{tag: tagSh, body: childrenBody{children: []node{inner}}}
		default:
			return node{tag: tagSh, body: childrenBody{children: []node{mk}}}
		}
	}
	for _, tc := range []struct {
		vector   string
		tree     node
	}{
		// wsh_sortedmulti is the sortedmulti template golden — exact match.
		{"wsh_sortedmulti", mkTree(tagWsh, false, tagSortedMulti)},
		// sh_wsh_multi is the only vendored sh(wsh) wrapper golden; it carries
		// tagMulti, so build the matching tagMulti tree to assert wrapper layout.
		{"sh_wsh_multi", mkTree(tagSh, true, tagMulti)},
	} {
		t.Run(tc.vector, func(t *testing.T) {
			// The vendored template golden has NO origin/usesite/pubkeys TLV; build
			// the matching bare descriptor (shared empty origin, bare-star use-site).
			d := loadDescriptor(t, tc.vector)
			got, _, err := encodePayload(&descriptor{
				n:        d.n,
				pathDecl: d.pathDecl,
				useSite:  d.useSite,
				tree:     tc.tree,
				tlv:      d.tlv,
			})
			if err != nil {
				t.Fatalf("encodePayload: %v", err)
			}
			want := loadBytesHex(t, tc.vector)
			if hex.EncodeToString(got) != hex.EncodeToString(want) {
				t.Fatalf("template bytes mismatch:\n got  %x\n want %x", got, want)
			}
		})
	}
}
```

- [ ] **Step 2: Run test to verify it fails... or confirm it passes immediately**

Run: `go test ./md/ -run TestEncodeMultisigTemplateParity -v`
Expected: PASS immediately (this exercises the SHIPPED `encodePayload` against existing vendored goldens with hand-built trees — it is a parity guard that documents VF2/VF10, no new production code). If it FAILS, the bit layout assumption is wrong → STOP and re-verify VF2 before proceeding.

- [ ] **Step 3: (no production code needed for 3a)**

- [ ] **Step 4: Commit**

```bash
git add md/encode_multisig_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "test(md): template-parity vs vendored sortedmulti/sh-wsh goldens (T6c A1)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task 3b — generate + vendor the `wsh` full-policy golden, then assert full-policy parity (A2)

- [ ] **Step 1: Generate the golden via the Go assembler (frozen output)**

The full-policy goldens are produced by the Go assembler fed Rust-validated depth-4-xpub-derived 65-byte payloads (provenance note above). Run this throwaway generator to capture the chunk strings + payload hex + ids, then hand-write the `.meta.json`/`.md1.txt` from its output:

```bash
cat > /tmp/gen_multisig_golden.go <<'EOF'
//go:build ignore
package main
// (run with: go run inside a temp main; OR paste into a _test.go and t.Log)
EOF
```

Instead of a standalone main, add a TEMPORARY generation test, run it, COPY the output into the vendored files, then DELETE the temp test:

```go
// TEMP — delete after vendoring. Generates the wsh full-policy golden.
func TestGenWshFull(t *testing.T) {
	cc, pk := mkXpub65(t,
		"4a53a0ab21b9dc95869c4e92a161194e03c0ef3ff5014ac692f433c4765490fc", // any 32B cc
		"02707a62fdacc26ea9b63b1c197906f56ee0180d0bcf1966e1a2da34f5f3a09a9b") // any compressed pubkey
	req := EncodeMultisigRequest{
		Cosigners: []MultisigCosigner{
			{ChainCode: cc, CompressedPubkey: pk},
			{ChainCode: cc, CompressedPubkey: pk},
			{ChainCode: cc, CompressedPubkey: pk},
		},
		K: 2, Script: MultisigWsh, OriginMode: OriginShared, SharedOrigin: sharedOrigin4828(),
	}
	out, stub, _, err := EncodeMultisig(req)
	if err != nil { t.Fatal(err) }
	d, _ := Reassemble(out)
	b, _, _ := encodePayload(d)
	id, _ := WalletPolicyIdChunks(out)
	for _, s := range out { t.Logf("CHUNK %s", s) }
	t.Logf("PAYLOAD %x", b)
	t.Logf("WPID %x STUB %x", id, stub)
}
```

Run: `go test ./md/ -run TestGenWshFull -v 2>&1 | grep -E 'CHUNK|PAYLOAD|WPID'`
Expected: 6 `CHUNK md1f…` lines + one `PAYLOAD …` + one `WPID … STUB …`. (The exact xpub bytes above are arbitrary public material — the goldens just freeze whatever the assembler emits for them.)

- [ ] **Step 2: Vendor the golden files from the captured output**

Create `md/testdata/vectors/multisig_wsh_full.md1.txt` — the 6 `CHUNK` strings, one per line (no `CHUNK ` prefix).

Create `md/testdata/vectors/multisig_wsh_full.meta.json`:
```json
{
  "script": "wsh_sortedmulti",
  "k": 2,
  "n": 3,
  "origin_mode": "shared",
  "shared_origin": "m/48'/0'/0'/2'",
  "fp_present": false,
  "cosigners": [
    {"chaincode": "4a53a0ab21b9dc95869c4e92a161194e03c0ef3ff5014ac692f433c4765490fc", "compressed_pubkey": "02707a62fdacc26ea9b63b1c197906f56ee0180d0bcf1966e1a2da34f5f3a09a9b", "fingerprint": "", "fp_present": false},
    {"chaincode": "4a53a0ab21b9dc95869c4e92a161194e03c0ef3ff5014ac692f433c4765490fc", "compressed_pubkey": "02707a62fdacc26ea9b63b1c197906f56ee0180d0bcf1966e1a2da34f5f3a09a9b", "fingerprint": "", "fp_present": false},
    {"chaincode": "4a53a0ab21b9dc95869c4e92a161194e03c0ef3ff5014ac692f433c4765490fc", "compressed_pubkey": "02707a62fdacc26ea9b63b1c197906f56ee0180d0bcf1966e1a2da34f5f3a09a9b", "fingerprint": "", "fp_present": false}
  ],
  "payload_hex": "<PASTE the PAYLOAD hex from step 1>",
  "wallet_policy_id": "<PASTE the WPID from step 1>",
  "wallet_policy_id_stub": "<PASTE the STUB from step 1>"
}
```

Create `md/testdata/vectors/README_multisig.md`:
```markdown
# multisig full-policy goldens (T6c Phase A)

Generated by the Go md.EncodeMultisig assembler fed depth-4-xpub-derived 65-byte
payloads, cross-checked against the Rust-sourced template goldens
(wsh_sortedmulti.bytes.hex / sh_wsh_multi.bytes.hex) for bit-layout parity, and
against the depth-4 xpub the Rust `md` CLI accepts for multisig:

    xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf  (abandon-seed @ m/48'/0'/0'/2')

Rust CLI cross-check (descriptor-mnemonic @ c85cd49; depth-4 xpub required for
ScriptCtx::MultiSig, md-cli/src/parse/keys.rs:67-77):

    md encode 'wsh(sortedmulti(2,@0,@1,@2))' --key @0=<xpub> --key @1=<xpub> --key @2=<xpub> --force-chunked --json   # 0x36d1b
    md encode 'sh(wsh(sortedmulti(2,@0,@1,@2)))' --key @0=<xpub> --key @1=<xpub> --key @2=<xpub> --force-chunked --json # 0x58624
    md encode 'sh(sortedmulti(2,@0,@1,@2))' --key @0=<xpub> --key @1=<xpub> --key @2=<xpub> --force-chunked --json     # 0x90289

NB: a bare sh(sortedmulti) template with no explicit origin fails md Reassemble
("missing explicit origin"); EncodeMultisig always supplies an explicit shared
origin, so the sh / sh(wsh) goldens are generated in Go with that origin and
frozen here.
```

- [ ] **Step 3: Delete the temp generator test and add the real loader + A2 parity test**

Remove `TestGenWshFull`. Append the loader + A2 test:

```go
type multisigMeta struct {
	Script       string `json:"script"`
	K            uint8  `json:"k"`
	N            int    `json:"n"`
	OriginMode   string `json:"origin_mode"`
	SharedOrigin string `json:"shared_origin"`
	FpPresent    bool   `json:"fp_present"`
	Cosigners    []struct {
		ChainCode        string `json:"chaincode"`
		CompressedPubkey string `json:"compressed_pubkey"`
		Fingerprint      string `json:"fingerprint"`
		FpPresent        bool   `json:"fp_present"`
		Origin           string `json:"origin"`
	} `json:"cosigners"`
	PayloadHex string `json:"payload_hex"`
	WPID       string `json:"wallet_policy_id"`
	Stub       string `json:"wallet_policy_id_stub"`
}

func loadMultisigMeta(t *testing.T, name string) multisigMeta {
	t.Helper()
	raw, err := os.ReadFile(filepath.Join("testdata", "vectors", name+".meta.json"))
	if err != nil {
		t.Fatalf("read %s.meta.json: %v", name, err)
	}
	var m multisigMeta
	if err := jsonUnmarshalStrict(raw, &m); err != nil {
		t.Fatalf("unmarshal %s.meta.json: %v", name, err)
	}
	return m
}

func loadMultisigChunks(t *testing.T, name string) []string {
	t.Helper()
	raw, err := os.ReadFile(filepath.Join("testdata", "vectors", name+".md1.txt"))
	if err != nil {
		t.Fatalf("read %s.md1.txt: %v", name, err)
	}
	var chunks []string
	for _, l := range strings.Split(string(raw), "\n") {
		if l = strings.TrimSpace(l); l != "" {
			chunks = append(chunks, l)
		}
	}
	return chunks
}

func multisigScriptFromName(t *testing.T, s string) MultisigScript {
	t.Helper()
	switch s {
	case "wsh_sortedmulti":
		return MultisigWsh
	case "sh_wsh_sortedmulti":
		return MultisigShWsh
	case "sh_sortedmulti":
		return MultisigSh
	default:
		t.Fatalf("unknown multisig script %q", s)
		return 0
	}
}

// reqFromMeta builds the EncodeMultisigRequest the meta.json describes.
func reqFromMeta(t *testing.T, m multisigMeta) EncodeMultisigRequest {
	t.Helper()
	req := EncodeMultisigRequest{K: m.K, Script: multisigScriptFromName(t, m.Script)}
	if m.OriginMode == "divergent" {
		req.OriginMode = OriginDivergent
	} else {
		req.OriginMode = OriginShared
		req.SharedOrigin = parsePathComponents(t, m.SharedOrigin)
	}
	for _, c := range m.Cosigners {
		cc, pk := mkXpub65(t, c.ChainCode, c.CompressedPubkey)
		mc := MultisigCosigner{ChainCode: cc, CompressedPubkey: pk, FpPresent: c.FpPresent}
		if c.FpPresent {
			fb, err := hex.DecodeString(c.Fingerprint)
			if err != nil || len(fb) != 4 {
				t.Fatalf("bad fp %q", c.Fingerprint)
			}
			copy(mc.Fingerprint[:], fb)
		}
		if req.OriginMode == OriginDivergent {
			mc.Origin = parsePathComponents(t, c.Origin)
		}
		req.Cosigners = append(req.Cosigners, mc)
	}
	return req
}

var multisigFullSets = []string{"multisig_wsh_full"}

// TestEncodeMultisigFullPolicyParity (A2): EncodeMultisig fed the meta inputs
// reproduces the vendored chunk strings byte-for-byte, the reassembled payload
// equals payload_hex, and the WalletPolicyId/stub match.
func TestEncodeMultisigFullPolicyParity(t *testing.T) {
	for _, name := range multisigFullSets {
		t.Run(name, func(t *testing.T) {
			m := loadMultisigMeta(t, name)
			req := reqFromMeta(t, m)
			out, stub, _, err := EncodeMultisig(req)
			if err != nil {
				t.Fatalf("EncodeMultisig: %v", err)
			}
			want := loadMultisigChunks(t, name)
			if len(out) != len(want) {
				t.Fatalf("chunk count: got %d want %d", len(out), len(want))
			}
			for i := range out {
				if out[i] != want[i] {
					t.Fatalf("chunk %d:\n got  %s\n want %s", i, out[i], want[i])
				}
			}
			d, err := Reassemble(out)
			if err != nil {
				t.Fatalf("Reassemble: %v", err)
			}
			gotPayload, _, err := encodePayload(d)
			if err != nil {
				t.Fatalf("encodePayload: %v", err)
			}
			if hex.EncodeToString(gotPayload) != m.PayloadHex {
				t.Fatalf("payload:\n got  %x\n want %s", gotPayload, m.PayloadHex)
			}
			id, _ := WalletPolicyIdChunks(out)
			if hex.EncodeToString(id[:]) != m.WPID {
				t.Fatalf("WalletPolicyId: got %x want %s", id, m.WPID)
			}
			if hex.EncodeToString(stub[:]) != m.Stub {
				t.Fatalf("stub: got %x want %s", stub, m.Stub)
			}
		})
	}
}
```

Append the strict-unmarshal helper (rejects unknown fields, catching meta typos):
```go
func jsonUnmarshalStrict(b []byte, v any) error {
	dec := json.NewDecoder(strings.NewReader(string(b)))
	dec.DisallowUnknownFields()
	return dec.Decode(v)
}
```
…and add `"encoding/json"` to the test file imports.

- [ ] **Step 4: Run the A2 test**

Run: `go test ./md/ -run TestEncodeMultisigFullPolicyParity -v`
Expected: PASS (`multisig_wsh_full`).

- [ ] **Step 5: Commit**

```bash
git add md/encode_multisig_test.go md/testdata/vectors/multisig_wsh_full.meta.json md/testdata/vectors/multisig_wsh_full.md1.txt md/testdata/vectors/README_multisig.md
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "test(md): full-policy wsh sortedmulti golden + A2 parity (T6c Phase A Task 3b)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task 3c — T6b fixture byte-exact equality (A3 — the strongest gate)

- [ ] **Step 1: Write the failing test**

Append to `md/encode_multisig_test.go`:

```go
// t6bChunks is the vendored T6b multisig fixture (copied from
// gui/testdata/t6b_multisig_full.md1.txt — the md package cannot import gui
// testdata, so the 6 strings are inlined; they are guarded by A3 byte-equality
// AND by the gui-side TestSuppliedMultisigFixtureIsFullPolicy).
var t6bChunks = []string{
	"md1fvgfqzspqjtvyyy4qqxppcgsc27rczqg3yyc5z5tpwxqergd3c8g7ruszzg3ryssjfstllhxufdm4",
	"md1fvgfqzs2jvfeg9y4zktpd9chs82fefgh35nuevya8z62kep2q7md6duvfx8px8ygw3q3umhs2q3cu",
	"md1fvgfqzss8ygdjvlt5pterdm5rru59s2su80aw2q4wgdpapgfl4pkhsdyytkwl5zq9ner9ltnl8fnz",
	"md1fvgfqzsllphut2hvvpp5wl4l0mn058ndxfl63kufyfsjwlt2vkk2nlqmlvch5n4sk08xmsudrng93",
	"md1fvgfqz3qhwf72vyq3zgf3g9gkzuvpjxsmrsw3u8eqyy3zxfp9ycnjs2f29vkz6ts908m9qqcmg97l",
	"md1fvgfqz3f0qtrqglu5g8kh6mfsg4qxa9wq0nv9cauwfwxw70984wkqnw2uwz0w27h0f8nmf46cm8",
}

// TestEncodeMultisigT6bByteExact (A3): fed the three decoded T6b cosigners
// (fp-ABSENT, k=2, shared origin m/48'/0'/0'/2', wsh) in @0/@1/@2 order,
// EncodeMultisig reproduces the fixture chunk-for-chunk AND yields
// WalletPolicyId 7b716421db8b9f462967d04e0f8a3fd5. This proves a device could
// re-author the exact T6b card.
func TestEncodeMultisigT6bByteExact(t *testing.T) {
	cc0, pk0 := mkXpub65(t, "101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f", "03a9394a2f1a4f99613a716956c8540f6dba6f18931c2639107221b267d740af23")
	cc1, pk1 := mkXpub65(t, "bba0c7ca160a870efeb940ab90d0f4284fea1b5e0d2117677e823fc37e2d5763", "021a3bf5fbf737d0f36993fd46dc4913093beb532d654fe0dfd98bd27585dc9f29")
	cc2, pk2 := mkXpub65(t, "101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f", "02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5")
	req := EncodeMultisigRequest{
		Cosigners: []MultisigCosigner{
			{ChainCode: cc0, CompressedPubkey: pk0},
			{ChainCode: cc1, CompressedPubkey: pk1},
			{ChainCode: cc2, CompressedPubkey: pk2},
		},
		K: 2, Script: MultisigWsh, OriginMode: OriginShared, SharedOrigin: sharedOrigin4828(),
	}
	out, stub, slots, err := EncodeMultisig(req)
	if err != nil {
		t.Fatalf("EncodeMultisig: %v", err)
	}
	if len(out) != len(t6bChunks) {
		t.Fatalf("chunk count: got %d want %d", len(out), len(t6bChunks))
	}
	for i := range out {
		if out[i] != t6bChunks[i] {
			t.Fatalf("chunk %d:\n got  %s\n want %s", i, out[i], t6bChunks[i])
		}
	}
	id, _ := WalletPolicyIdChunks(out)
	if hex.EncodeToString(id[:]) != "7b716421db8b9f462967d04e0f8a3fd5" {
		t.Fatalf("WalletPolicyId = %x, want 7b716421db8b9f462967d04e0f8a3fd5", id)
	}
	if hex.EncodeToString(stub[:]) != "7b716421" {
		t.Fatalf("stub = %x, want 7b716421", stub)
	}
	for i, s := range slots {
		if s.FpPresent {
			t.Fatalf("slot %d fp-present, want absent (T6b is fp-absent)", i)
		}
	}
}
```

> Note: cc1/pk1 split the @1 payload `bba0c7ca…dc9f29` at byte 32: chaincode = first 32 B (`bba0c7ca…7e2d5763`), compressed pubkey = last 33 B (`021a3bf5…dc9f29`). cc0/cc2 share the synthetic chaincode `1011…2e2f`.

- [ ] **Step 2: Run test to verify it fails (then passes once Task 2 code is correct)**

Run: `go test ./md/ -run TestEncodeMultisigT6bByteExact -v`
Expected: PASS (Task 2's assembler already produces this — PROVEN in VF8 during plan authoring). If it FAILS, the assembler diverges from the fixture → STOP and diff `encodePayload(Reassemble(t6bChunks))` against `encodePayload(d)` to localize.

- [ ] **Step 3: (no production code — A3 validates Task 2)**

- [ ] **Step 4: Commit**

```bash
git add md/encode_multisig_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "test(md): T6b fixture byte-exact equality (T6c Phase A Task 3c, A3)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task 3d — the two `sh` shapes: generate + vendor goldens + parity

- [ ] **Step 1: Generate the two sh-shape goldens (temp tests, capture output)**

Add two TEMP generation tests mirroring `TestGenWshFull` but with `Script: MultisigShWsh` (name `multisig_sh_wsh_full`) and `Script: MultisigSh` (name `multisig_sh_full`); both with the same three identical cosigners, `OriginShared`, `sharedOrigin4828()`, fp-absent. Run each, capture `CHUNK`/`PAYLOAD`/`WPID`/`STUB`.

Run: `go test ./md/ -run 'TestGenShWshFull|TestGenShFull' -v 2>&1 | grep -E 'CHUNK|PAYLOAD|WPID'`
Expected: 6 chunks + payload + ids for each shape.

- [ ] **Step 2: Vendor `multisig_sh_wsh_full.{meta.json,md1.txt}` and `multisig_sh_full.{meta.json,md1.txt}`**

Same `.meta.json` schema as Task 3b, with `"script": "sh_wsh_sortedmulti"` and `"script": "sh_sortedmulti"` respectively, and the captured `payload_hex`/`wallet_policy_id`/`wallet_policy_id_stub`. The `.md1.txt` is the 6 captured chunk strings.

- [ ] **Step 3: Delete the temp generators; extend the parity sweep**

Remove `TestGenShWshFull`/`TestGenShFull`. Extend the A2 set:
```go
var multisigFullSets = []string{"multisig_wsh_full", "multisig_sh_wsh_full", "multisig_sh_full"}
```
Add an InnerWsh-discriminant assertion to `TestEncodeMultisigFullPolicyParity` (decode and check Root/InnerWsh):
```go
			tpl, _, err := ExpandWalletPolicyChunks(out)
			if err != nil {
				t.Fatalf("ExpandWalletPolicyChunks: %v", err)
			}
			switch req.Script {
			case MultisigWsh:
				if tpl.Root != ScriptWsh || tpl.InnerWsh {
					t.Fatalf("%s: Root=%v InnerWsh=%v, want Wsh/false", name, tpl.Root, tpl.InnerWsh)
				}
			case MultisigShWsh:
				if tpl.Root != ScriptSh || !tpl.InnerWsh {
					t.Fatalf("%s: Root=%v InnerWsh=%v, want Sh/true", name, tpl.Root, tpl.InnerWsh)
				}
			case MultisigSh:
				if tpl.Root != ScriptSh || tpl.InnerWsh {
					t.Fatalf("%s: Root=%v InnerWsh=%v, want Sh/false", name, tpl.Root, tpl.InnerWsh)
				}
			}
			if tpl.Policy != PolicySortedMulti {
				t.Fatalf("%s: Policy=%v, want SortedMulti", name, tpl.Policy)
			}
```

- [ ] **Step 4: Run the extended A2 sweep**

Run: `go test ./md/ -run TestEncodeMultisigFullPolicyParity -v`
Expected: PASS for all three sets; the InnerWsh discriminant is `false/true/false` for `wsh/sh_wsh/sh`.

- [ ] **Step 5: Commit**

```bash
git add md/encode_multisig_test.go md/testdata/vectors/multisig_sh_wsh_full.meta.json md/testdata/vectors/multisig_sh_wsh_full.md1.txt md/testdata/vectors/multisig_sh_full.meta.json md/testdata/vectors/multisig_sh_full.md1.txt
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "test(md): sh(sortedmulti) + sh(wsh(sortedmulti)) goldens + InnerWsh parity (T6c Phase A Task 3d)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4 — Round-trip + identity (A4) + fp-present + divergent coverage (A5)

**Files:**
- Modify: `md/encode_multisig_test.go`
- Create: `md/testdata/vectors/multisig_wsh_fp.{meta.json,md1.txt}`, `multisig_wsh_divergent.{meta.json,md1.txt}`

- [ ] **Step 1: Write the failing round-trip + identity test (A4)**

Append:
```go
// TestEncodeMultisigRoundTrip (A4/I2/I6): EncodeMultisig output decodes back to
// the same template (root/policy/k/n/innerWsh) and per-@N {xpub, fp(+presence),
// origin, use-site} IN ORDER; and WalletPolicyIdChunks(out) == the returned stub
// prefix and equals WalletPolicyId of Reassemble(out) (identity zero-change).
func TestEncodeMultisigRoundTrip(t *testing.T) {
	for _, name := range multisigFullSets {
		t.Run(name, func(t *testing.T) {
			m := loadMultisigMeta(t, name)
			req := reqFromMeta(t, m)
			out, stub, slots, err := EncodeMultisig(req)
			if err != nil {
				t.Fatalf("EncodeMultisig: %v", err)
			}
			tpl, keys, err := ExpandWalletPolicyChunks(out)
			if err != nil {
				t.Fatalf("ExpandWalletPolicyChunks: %v", err)
			}
			if tpl.K != int(req.K) || tpl.N != len(req.Cosigners) {
				t.Fatalf("K/N = %d/%d, want %d/%d", tpl.K, tpl.N, req.K, len(req.Cosigners))
			}
			if len(keys) != len(req.Cosigners) {
				t.Fatalf("recovered %d keys, want %d", len(keys), len(req.Cosigners))
			}
			for i, k := range keys {
				if int(k.Index) != i {
					t.Fatalf("key %d Index = %d (order not preserved)", i, k.Index)
				}
				var wantXpub [65]byte
				copy(wantXpub[:32], req.Cosigners[i].ChainCode[:])
				copy(wantXpub[32:], req.Cosigners[i].CompressedPubkey[:])
				if k.Xpub != wantXpub {
					t.Fatalf("key %d xpub mismatch", i)
				}
				if k.FingerprintPresent != req.Cosigners[i].FpPresent {
					t.Fatalf("key %d fp-present = %v, want %v", i, k.FingerprintPresent, req.Cosigners[i].FpPresent)
				}
				if k.FingerprintPresent && k.Fingerprint != req.Cosigners[i].Fingerprint {
					t.Fatalf("key %d fp = %x, want %x", i, k.Fingerprint, req.Cosigners[i].Fingerprint)
				}
				if !k.UseSite.HasMultipath || len(k.UseSite.Multipath) != 2 {
					t.Fatalf("key %d use-site = %+v, want <0;1>", i, k.UseSite)
				}
			}
			// Identity zero-change: chunks-id == descriptor-id; stub is its prefix.
			idChunks, err := WalletPolicyIdChunks(out)
			if err != nil {
				t.Fatalf("WalletPolicyIdChunks: %v", err)
			}
			d, err := Reassemble(out)
			if err != nil {
				t.Fatalf("Reassemble: %v", err)
			}
			idDesc, err := WalletPolicyId(d)
			if err != nil {
				t.Fatalf("WalletPolicyId: %v", err)
			}
			if idChunks != idDesc {
				t.Fatalf("WalletPolicyIdChunks %x != WalletPolicyId(Reassemble) %x", idChunks, idDesc)
			}
			if [4]byte(idChunks[:4]) != stub {
				t.Fatalf("stub %x != id prefix %x", stub, idChunks[:4])
			}
			// slots reflect order + fp presence.
			for i, s := range slots {
				if int(s.Index) != i || s.FpPresent != req.Cosigners[i].FpPresent {
					t.Fatalf("slot %d = %+v inconsistent with cosigner", i, s)
				}
			}
			_ = tpl
		})
	}
}
```

- [ ] **Step 2: Run it**

Run: `go test ./md/ -run TestEncodeMultisigRoundTrip -v`
Expected: PASS for the three Task-3 sets (no new production code — validates Task 2).

- [ ] **Step 3: Generate + vendor the fp-present and divergent goldens (A5)**

Add TEMP generators:
- `multisig_wsh_fp`: same three identical cosigners but `FpPresent:true` with distinct fps (`deadbeef`,`cafebabe`,`01020304`), `MultisigWsh`, `OriginShared`, `sharedOrigin4828()`.
- `multisig_wsh_divergent`: three identical cosigners, `MultisigWsh`, `OriginDivergent`, each with its own `Origin` (e.g. `m/48'/0'/0'/2'`, `m/48'/0'/0'/3'`, `m/48'/0'/0'/4'`); `SharedOrigin` empty.

Run each temp generator, capture output, vendor `.meta.json` (with `"fp_present": true` + per-cosigner fps for the fp set; `"origin_mode": "divergent"` + per-cosigner `"origin"` for the divergent set) + `.md1.txt`. Delete the temp generators. Extend:
```go
var multisigFullSets = []string{"multisig_wsh_full", "multisig_sh_wsh_full", "multisig_sh_full", "multisig_wsh_fp", "multisig_wsh_divergent"}
```

- [ ] **Step 4: Run the full A2+A4 sweep over all five sets**

Run: `go test ./md/ -run 'TestEncodeMultisigFullPolicyParity|TestEncodeMultisigRoundTrip' -v`
Expected: PASS for all five sets. The fp set recovers `FingerprintPresent=true` with the right fps; the divergent set recovers per-@N origins `m/48h/0h/0h/2h`, `…/3h`, `…/4h`.

- [ ] **Step 5: Commit**

```bash
git add md/encode_multisig_test.go md/testdata/vectors/multisig_wsh_fp.meta.json md/testdata/vectors/multisig_wsh_fp.md1.txt md/testdata/vectors/multisig_wsh_divergent.meta.json md/testdata/vectors/multisig_wsh_divergent.md1.txt
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "test(md): round-trip+identity (A4) + fp-present/divergent goldens (A5) (T6c Phase A Task 4)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 5 — Refuse-unsupported (A6)

**Files:**
- Modify: `md/encode_multisig_test.go`

- [ ] **Step 1: Write the failing test**

Append:
```go
// TestEncodeMultisigRefuse (A6/I5): invalid k/n, divergent-count/origin
// mismatch, and empty origins yield typed errors. The shipped split guards
// surface via errors.Is; the assembler's own guards are matched directly.
func TestEncodeMultisigRefuse(t *testing.T) {
	cc, pk := mkXpub65(t, "101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f", "03a9394a2f1a4f99613a716956c8540f6dba6f18931c2639107221b267d740af23")
	cosigner := MultisigCosigner{ChainCode: cc, CompressedPubkey: pk}
	three := []MultisigCosigner{cosigner, cosigner, cosigner}

	mkReq := func(mut func(*EncodeMultisigRequest)) EncodeMultisigRequest {
		r := EncodeMultisigRequest{
			Cosigners: append([]MultisigCosigner(nil), three...),
			K: 2, Script: MultisigWsh, OriginMode: OriginShared, SharedOrigin: sharedOrigin4828(),
		}
		mut(&r)
		return r
	}

	for _, tc := range []struct {
		name    string
		req     EncodeMultisigRequest
		wantErr error // matched via errors.Is when non-nil; else just "must error"
	}{
		{"k>n", mkReq(func(r *EncodeMultisigRequest) { r.K = 4 }), errKGreaterThanN},
		{"k=0", mkReq(func(r *EncodeMultisigRequest) { r.K = 0 }), errThresholdRange},
		{"empty-shared-origin", mkReq(func(r *EncodeMultisigRequest) { r.SharedOrigin = nil }), errMultisigEmptySharedOrigin},
		{"divergent-empty-origin", mkReq(func(r *EncodeMultisigRequest) {
			r.OriginMode = OriginDivergent
			r.SharedOrigin = nil
			// all three cosigners have nil Origin → empty divergent
		}), errMultisigEmptyDivergent},
		{"zero-cosigners", mkReq(func(r *EncodeMultisigRequest) { r.Cosigners = nil; r.K = 1 }), errKeyCountRange},
		{"bad-script", mkReq(func(r *EncodeMultisigRequest) { r.Script = MultisigScript(99) }), errMultisigBadScript},
	} {
		t.Run(tc.name, func(t *testing.T) {
			_, _, _, err := EncodeMultisig(tc.req)
			if err == nil {
				t.Fatalf("%s: got nil error, want %v", tc.name, tc.wantErr)
			}
			if tc.wantErr != nil && !errors.Is(err, tc.wantErr) {
				t.Fatalf("%s: err = %v, want errors.Is %v", tc.name, err, tc.wantErr)
			}
		})
	}
}
```

- [ ] **Step 2: Run it**

Run: `go test ./md/ -run TestEncodeMultisigRefuse -v`
Expected: PASS. (`k>n`/`k=0`/`zero-cosigners` surface the shipped guards through `split`; the origin/script cases hit the assembler's own guards.)

> If `zero-cosigners` returns a different error than `errKeyCountRange` (e.g. an earlier nil-deref guard), localize by logging the error; the expectation per VF3 is `writePathDecl` rejects `n<1` with `errKeyCountRange`. Do NOT add a redundant assembler guard unless `split` does not reach it.

- [ ] **Step 3: (no production code — A6 validates Task 2 guards)**

- [ ] **Step 4: Commit**

```bash
git add md/encode_multisig_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "test(md): refuse-unsupported k/n/origin/script (T6c Phase A Task 5, A6)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 6 — Fuzz (A7) + no-regression sweep

**Files:**
- Create: `md/encode_multisig_fuzz_test.go`

- [ ] **Step 1: Write the fuzz test**

Create `md/encode_multisig_fuzz_test.go`:
```go
package md

import "testing"

// FuzzEncodeMultisig feeds arbitrary (n, k, per-cosigner cc/pk/fp/fpPresent,
// script, originMode) and asserts: no panic; and any SUCCESSFUL encode
// round-trips via ExpandWalletPolicyChunks recovering the inputs in order. An
// off-curve pubkey is rejected at DECODE only — a benign skip (mirrors
// FuzzEncodeSingleSig). (T6c Phase A Task 6.)
func FuzzEncodeMultisig(f *testing.F) {
	// Seed from the vendored full-policy golden inputs.
	f.Add(uint8(3), uint8(2),
		mustHexFuzz("101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f"),
		mustHexFuzz("03a9394a2f1a4f99613a716956c8540f6dba6f18931c2639107221b267d740af23"),
		mustHexFuzz("deadbeef"), true, uint8(0), uint8(0), uint32(48))

	f.Fuzz(func(t *testing.T, nRaw, kRaw byte, ccBytes, pkBytes, fpBytes []byte, fpPresent bool, scriptRaw, originRaw byte, originHead uint32) {
		n := int(nRaw%8) + 1 // 1..8 cosigners (kept small to keep the fuzz fast)
		var cc [32]byte
		copy(cc[:], ccBytes)
		var pk [33]byte
		copy(pk[:], pkBytes)
		var fp [4]byte
		copy(fp[:], fpBytes)

		cosigners := make([]MultisigCosigner, n)
		for i := range cosigners {
			cosigners[i] = MultisigCosigner{
				ChainCode: cc, CompressedPubkey: pk, Fingerprint: fp, FpPresent: fpPresent,
				Origin: []PathComponent{{Hardened: true, Value: originHead}},
			}
		}
		req := EncodeMultisigRequest{
			Cosigners:    cosigners,
			K:            kRaw%32 + 1, // 1..32
			Script:       MultisigScript(int(scriptRaw) % 3),
			OriginMode:   OriginMode(int(originRaw) % 2),
			SharedOrigin: []PathComponent{{Hardened: true, Value: originHead}},
		}
		out, _, slots, err := EncodeMultisig(req)
		if err != nil {
			return // guarded error (k>n, etc.) — benign skip
		}
		if len(out) < 1 {
			t.Fatalf("EncodeMultisig returned %d chunks", len(out))
		}
		if len(slots) != n {
			t.Fatalf("slots=%d, want %d", len(slots), n)
		}
		_, keys, err := ExpandWalletPolicyChunks(out)
		if err != nil {
			return // off-curve pubkey rejected at decode — benign skip
		}
		if len(keys) != n {
			t.Fatalf("recovered n=%d, want %d", len(keys), n)
		}
		for i, k := range keys {
			if int(k.Index) != i {
				t.Fatalf("key %d Index=%d (order not preserved)", i, k.Index)
			}
			var wantXpub [65]byte
			copy(wantXpub[:32], cc[:])
			copy(wantXpub[32:], pk[:])
			if k.Xpub != wantXpub {
				t.Fatalf("key %d xpub not recovered", i)
			}
		}
	})
}
```

- [ ] **Step 2: Run the fuzz seed corpus (smoke), then a short fuzz**

Run (seed smoke): `go test ./md/ -run FuzzEncodeMultisig -v`
Expected: PASS (the seed input encodes + round-trips).

Run (short fuzz): `go test ./md/ -run xxx -fuzz FuzzEncodeMultisig -fuzztime 20s`
Expected: `elapsed: …, gathered … new interesting … fuzz: elapsed: 20s, … no failures`. No panics, no crashers written to `testdata/fuzz/`.

- [ ] **Step 3: No-regression sweep — full md package + vet + build**

Run: `go test ./md/...`
Expected: `ok  	seedhammer.com/md`. Confirms the new file did NOT change any existing test (the shipped `byteParityVectorNames`/single-sig/chunk/identity tests are untouched).

Run: `go vet ./md/...`
Expected: clean (no output).

Run: `go build ./...`
Expected: clean (no output). Confirms no other package broke.

Run (byte-unchanged check — the shipped vectors' bytes.hex must be untouched): `git status --porcelain md/testdata/vectors/ | grep -vE 'multisig_|README_multisig'`
Expected: EMPTY (only the new multisig vectors + README_multisig added; no shipped vector modified).

- [ ] **Step 4: Commit**

```bash
git add md/encode_multisig_fuzz_test.go
git commit -S -s --author="Brian Goss <goss.brian@gmail.com>" -m "test(md): FuzzEncodeMultisig + no-regression sweep (T6c Phase A Task 6, A7)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 5: Final full-suite confirmation before handoff**

Run: `go test ./md/... && go vet ./md/... && go build ./...`
Expected: `ok  	seedhammer.com/md`, no vet output, no build output. This is the GREEN signal for the mandatory post-implementation adversarial review.

---

## Post-implementation gate (MANDATORY, non-deferrable)

After Task 6 is GREEN, dispatch a single independent adversarial opus execution review over the WHOLE diff (`git diff main...feat/t6c-encode-multisig`):
- R0 = plan correctness (does the impl match this plan + the spec §IN/§acceptance/§invariants I1–I8?).
- Implementation-introduced regressions TDD misses (ordering, fp-presence, identity zero-change, no shipped-vector drift).
Persist the review verbatim to `design/agent-reports/`. Do NOT merge/finish with any open Critical/Important. If Agent-API dispatch fails, flag explicitly and defer the formal review to API recovery — never silently substitute inline self-review.

---

## Self-Review

**1. Spec coverage (every §IN item + A1–A7 + I1–I8 maps to a task):**
- §IN.1 single exported `md.EncodeMultisig` in new `md/encode_multisig.go`, no GUI deps, public key material only → Task 1+2.
- §IN.2 three wrappers (wsh / sh(wsh) / sh) → Task 2 `multiSigTree`; A1 (3a) + A2 (3b/3d).
- §IN.3 N pubkey TLVs + optional per-cosigner fp gated on `FpPresent` → Task 2; A5 fp set (Task 4).
- §IN.4 shared OR divergent origins → Task 2 `OriginMode`; A5 divergent set (Task 4).
- §IN.5 deterministic ordering contract (I1) → Task 2 (order-preserving, no sort) + slots return; asserted in A3/A4/fuzz (Index==i).
- §IN.6 TDD acceptance (byte-exact + T6b + round-trip + fuzz) → Tasks 3–6.
- A1 template parity → Task 3a. A2 full-policy parity → Task 3b/3d. A3 T6b byte-exact → Task 3c. A4 round-trip+identity → Task 4. A5 fp+divergent → Task 4. A6 refuse → Task 5. A7 fuzz → Task 6.
- I1 ordering (Index==i, no sort) → Tasks 2/3c/4/6. I2 round-trip → Task 4. I3 byte-exact → 3a/3b/3c/3d. I4 no key sort → VF1, asserted by Index==i in 3c/4/6. I5 refuse → Task 5 + `default:` in `multiSigTree`. I6 identity zero-change → Task 4 (idChunks==idDesc). I7 kiw/n lockstep → Task 2 sets `descriptor.n==pathDecl.n==n`; backed by `errPathDeclNMismatch`. I8 TLV idx ascending → Task 2 builds idx `0..n-1` in order; backed by `errOverrideOrder`.
- R0 adoptions: struct constructor `EncodeMultisigRequest` (Task 1); explicit `OriginMode` enum, not nil-overload (Task 1); return `(out, stub, slots)` ordering handle (Task 2); m1 depth-4 xpub recorded (VF9 + README_multisig); m2 fresh `sh`/`sh(wsh)` sortedmulti goldens (Task 3d). All covered.
- OUT (GUI/picker/warning/Phase-B) correctly absent. Confirmed headless-only.

**2. Placeholder scan:** No "TBD"/"add error handling"/"similar to Task N"/"write tests for the above". Every code step has full code. The only `<PASTE …>` markers are in the golden-vendoring steps (3b/3d/4) — these are NOT placeholders for logic; they are the captured-output transcription points inherent to vendoring a deterministic golden, with the exact generator command + grep that produces the values to paste. The generator commands are fully specified.

**3. Type/signature consistency (checked across tasks):**
- `EncodeMultisig(req EncodeMultisigRequest) (out []string, stub [4]byte, slots []SlotInfo, err error)` — defined Task 2, called identically in Tasks 2/3b/3c/3d/4/5/6 (always 4-return).
- `EncodeMultisigRequest{Cosigners, K, Script, OriginMode, SharedOrigin}` — Task 1, used consistently.
- `MultisigCosigner{ChainCode [32]byte, CompressedPubkey [33]byte, Fingerprint [4]byte, FpPresent bool, Origin []PathComponent}` — Task 1; `mkXpub65` returns the cc/pk pair fed to it.
- `MultisigScript{MultisigWsh, MultisigShWsh, MultisigSh}` / `OriginMode{OriginShared, OriginDivergent}` / `SlotInfo{Index uint8, Fingerprint [4]byte, FpPresent bool}` — Task 1; used unchanged everywhere.
- `multiSigTree(script MultisigScript, k uint8, n int) (node, error)` and `toComponents([]PathComponent) []pathComponent` — Task 2; internal.
- Reused md symbols verified to exist @8eb51d7: `PathComponent` (encode_singlesig.go:18), `pathComponent`/`originPath`/`pathDecl`/`useSitePath`/`alternative`/`node`/`childrenBody`/`multiKeysBody`/`idxPub`/`idxFP`/`tlvSection`/`descriptor`/`tagWsh`/`tagSh`/`tagSortedMulti`/`tagMulti`, `split`, `encodePayload`, `Reassemble`, `ExpandWalletPolicyChunks`, `WalletPolicyId`, `WalletPolicyIdChunks`, `WalletPolicyIDStub`, `WalletPolicyIDStubChunks`, `ScriptWsh`/`ScriptSh`/`PolicySortedMulti`, `Template.{Root,Policy,K,N,InnerWsh}`, `ExpandedKey.{Index,Xpub,Fingerprint,FingerprintPresent,UseSite}`, `codex32.ValidMD`, `loadDescriptor`/`loadBytesHex`/`parsePathComponents`/`mustHexFuzz` (test helpers). Errors `errThresholdRange`/`errChildCount`/`errKGreaterThanN`/`errKeyCountRange`/`errDivergentCount`/`errPathDeclNMismatch`/`errOverrideOrder` exist; the three new ones (`errMultisigEmptySharedOrigin`/`errMultisigEmptyDivergent`/`errMultisigBadScript`) are defined in Task 1.
- `tlvSection` field name is `fingerprints` (not `fps`); the Task 2 literal uses `fingerprints: fps`. `fpPresent: len(fps) > 0` matches the shipped presence-flag convention.

**Open items for plan-R0 (ambiguities to confirm, none blocking):**
- (a) `WalletPolicyIDStub(d)` is called inside `EncodeMultisig` on the SAME descriptor that `split` consumed; A4 asserts it equals `WalletPolicyIDStubChunks(out)`. Confirm there is no canonicalize divergence between the two paths (probe during authoring showed equality for the T6b card; A4 generalizes it).
- (b) The Go-generated `sh`/`sh(wsh)` full-policy goldens (Task 3d) are cross-checked for layout only against the Rust template `sh_wsh_multi.bytes.hex` (A1) — there is no Rust full-policy `sh(sortedmulti)` golden because the CLI rejects a no-origin template. R0 to confirm this provenance is acceptable (it matches the shipped single-sig golden provenance pattern and the spec's A2 "freshly md encode-generated" intent, with the documented Reassemble-origin constraint VF11).
