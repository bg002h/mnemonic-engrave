# sh(wpkh) on-device-verify projection — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. A *single* subagent executes the GREEN plan in the worktree (NOT parallel re-implementations); TDD throughout.

**Goal:** Light up the on-device "Show / Verify addresses" flow for a decoded `sh(wpkh)` (BIP-49 P2SH-P2WPKH, `3…`) md1 wallet-policy by adding the two missing touch-points — a decoder classifier arm + an `InnerWpkh` discriminant, and the matching `ScriptSh+PolicySingle+InnerWpkh → P2SH_P2WPKH` projection arm — mirroring the existing `InnerWsh` (nested-multisig) precedent.

**Architecture:** Two surgical, additive touch-points (confirmed both needed — see Verified Facts). (1) `md/md.go`: a new `Template.InnerWpkh` field, an `innerWpkhNesting` helper set in `summarize`, and a `classifyPolicy` `sh(wpkh)→PolicySingle` arm so the shape becomes `Renderable`. (2) `gui/md1_expand.go`: the `case md.ScriptSh:` arm under `case md.PolicySingle:` returning `(P2SH_P2WPKH, Singlesig, true)` iff `InnerWpkh`. `address`/`bip380` need **zero** change (they already derive P2SH-P2WPKH). The fuzz harness (`md1_expand_fuzz_test.go`) gains `InnerWpkh` synthesis + `ScriptSh` in the expressible-shape arm. Stale comments updated. Mainnet-only.

**Tech Stack:** Go 1.26 (fork `/scratch/code/shibboleth/seedhammer`, HEAD `8eb51d7`); `go test`/`go vet`/`go build`; `btcd` (`chaincfg`, `hdkeychain`, `txscript`). Run Go via `export PATH=$PATH:/home/bcg/.local/go/bin`.

---

## Verified facts (do not re-derive)

All of the following were empirically confirmed by applying the full fix to a throwaway probe at HEAD `8eb51d7`, observing the result, then reverting to a pristine tree. **Do not re-litigate these — implement to them.**

- **VF1 — Both touch-points are genuinely needed (premise confirmed).** TODAY, the PRIMARY fixture (`EncodeSingleSig(<BIP-49 xpub>, ScriptShWpkh)` → `md.ExpandWalletPolicyChunks`) yields `Template{Root:ScriptSh(2), Policy:PolicyComplex(5), Renderable:false, InnerWsh:false, K:0, M:0}`, and `expandedToDescriptor` returns `(nil, expandUnsupported)` — display-only. `sh(wpkh)` is `PolicyComplex` today because `classifyPolicy`'s `tagSh` case (`md/md.go:1285-1300`) has no inner-`tagWpkh` arm.
- **VF2 — POST-fix the PRIMARY fixture produces the exact golden.** With BOTH touch-points applied, the SAME fixture yields `Template{Root:ScriptSh, Policy:PolicySingle, Renderable:true, InnerWsh:false, InnerWpkh:true}`, `expandedToDescriptor → (non-nil, expandOK)`, `desc.Script==P2SH_P2WPKH`, `desc.Type==Singlesig`, `address.Receive(desc,0)=="37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf"`, `address.Change(desc,0)=="34K56kSjgUCUSD8GTtuF7c9Zzwokbs6uZ7"`, `address.Supported(desc)==true`.
- **VF3 — The BIP-49 account xpub + masterFP for the abandon seed.** `deriveAccountXpub(abandonAboutMnemonic(), "", &chaincfg.MainNetParams, singleSigPath(49))` returns `xpub6C6nQwHaWbSrzs5tZ1q7m5R9cPK9eYpNMFesiXsYrgc1P8bvLLAet9JfHjYXKjToD8cBRswJXXbbFpXgwsswVPAZzKMa1jUp2kVkGVUaJa7` and masterFP `0x73c5da0a`. The chunked md1 is **3 strings** (`len(strs)==3`).
- **VF4 — `address` derives P2SH-P2WPKH with NO new code.** `address/address.go:144-146` maps `P2WPKH, P2SH_P2WPKH` → witness-pubkey-hash; `:160-170` wraps `P2SH_P2WPKH`/`P2SH_P2WSH` via `txscript.PayToAddrScript`→`NewAddressScriptHash`→`3…`. `bip380.P2SH_P2WPKH` is `Singlesig()`-classified (`bip380/bip380.go:116-117`). Do NOT touch `address/` or `bip380/`.
- **VF5 — No collision (structural).** The new arm keys on `case md.PolicySingle:`; every multisig `sh` arm keys on `case md.PolicySortedMulti:`. `PolicySingle(0) ≠ PolicySortedMulti(2)` → disjoint switch cases. For the SAME key material, `P2SH_P2WPKH=37Vuc…`, `P2SH_P2WSH=3Kdr7CoTcx8UaGuzD7aqQxXi1dxUmBdph2`, bare `P2SH=39kh1g5VzX7eEEzAnbNZsG2w1WCYNQVu3G` are pairwise-distinct.
- **VF6 — A6 is two coupled halves, BOTH required.** With the fix applied but the fuzz harness un-updated, the fuzzer NEVER reaches the new arm (false-green) because the harness sets `InnerWpkh=false` always. Synthesizing `InnerWpkh` WITHOUT also adding `ScriptSh` to `isBip380ExpressibleShape`'s `PolicySingle` arm trips the line-74 invariant (`expandOK for non-bip380 shape root=2 policy=0` — confirmed empirically). Both halves must land together.
- **VF7 — `expandedToDescriptor` is shape-agnostic; mainnet pinned at `md1_expand.go:61`.** A `PolicySingle` one-key shape flows through unchanged once `scriptForTemplate` returns `(P2SH_P2WPKH, Singlesig, true)`. `Threshold: tpl.K` (0 for singlesig, unused).
- **VF8 — Helper signatures (for fixtures).**
  - `md.EncodeSingleSig(chainCode [32]byte, compressedPubkey [33]byte, fp [4]byte, origin []PathComponent, script ScriptKind) ([]string, error)` (`md/encode_singlesig.go:36`).
  - `md.ExpandWalletPolicyChunks(strs []string) (Template, []ExpandedKey, error)` — runs `Reassemble` then `summarize`/`ExpandWalletPolicy`, i.e. routes through the changed code (`md/expand.go:102`).
  - `deriveAccountXpub(m bip39.Mnemonic, passphrase string, net *chaincfg.Params, path bip32.Path) (xpub string, masterFP uint32, err error)` (`gui/derive.go:19`).
  - `decodeXpubBytes(xpub string) (chainCode [32]byte, compressedPubkey [33]byte, parentFP uint32, err error)` (`gui/singlesig_derive.go:99`).
  - `originComponents(path bip32.Path) []md.PathComponent` (`gui/singlesig_derive.go:128`).
  - `singleSigPath(purpose int) bip32.Path` → `m/<purpose>'/0'/0'` (`gui/singlesig_pick.go:81`).
  - `singleSigRestoreDescriptor(xpub string, masterFP, parentFP uint32, script md.ScriptKind, path bip32.Path) (*bip380.Descriptor, error)` (`gui/singlesig_restore.go:60`).
  - `abandonAboutMnemonic() bip39.Mnemonic`, `knownAccountXpub84`, `knownMasterFP` (`gui/derive_test.go:13,26,27`).
  - `expandedKey(idx int, fp [4]byte) md.ExpandedKey`, `goldenXpub(i int) [65]byte`, `stdUseSite` (`gui/md1_expand_test.go:18-50`).
  - `address.Receive`/`address.Change`/`address.Find`/`address.Supported(desc *bip380.Descriptor) bool` (`address/address.go:28`).

---

## File-structure map

| File | Responsibility | Change |
|---|---|---|
| `md/md.go` | Decode → `summarize` → `Template`; `classifyPolicy`, `innerWshNesting` | **Modify:** add `Template.InnerWpkh` field; add `innerWpkhNesting` helper; set `InnerWpkh` in `summarize`; add the `sh(wpkh)→PolicySingle` arm in `classifyPolicy`. |
| `md/md_test.go` (EXISTS, `package md`) | Unit test of the decoder touch-point | **Append:** `TestClassifyPolicyShWpkhRenders` + `TestInnerWpkhNesting` + the `shWpkhNode` helper (Task 1). File already imports `testing`; do NOT re-create it. |
| `gui/md1_expand.go` | `scriptForTemplate` / `expandedToDescriptor` projection | **Modify:** add the `case md.ScriptSh:` arm under `PolicySingle`; remove the stale NOTE (`:96-97`). |
| `gui/md1_expand_test.go` | Projection unit + golden + no-collision tests | **Modify:** add `TestExpandedToDescriptorShWpkh` (Task 2), `TestShWpkhGoldenAddress` + no-collision (Task 3). `TestExpandedToDescriptorShNesting` (`:124-149`) stays UNCHANGED. |
| `gui/md1_expand_fuzz_test.go` | Fuzz invariant harness | **Modify:** synthesize `InnerWpkh`; add `ScriptSh` to `isBip380ExpressibleShape`'s `PolicySingle` arm (Task 4). |
| `md/md.go:1173-1178` & `gui/singlesig_restore.go:25-29` | Stale comments asserting "decoder never renders sh-wpkh" | **Modify:** update both to reflect the now-rendered shape (Task 4). |
| `address/`, `bip380/` | Address derivation | **NO CHANGE** (VF4). |

---

## Task 0: Worktree + baseline

**Files:** none (setup).

- [ ] **Step 1: Create the worktree off `main` at `8eb51d7`**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer
git worktree add -b feat/sh-wpkh-verify-projection /scratch/code/shibboleth/seedhammer-shwpkh 8eb51d7
```
Expected: `Preparing worktree (new branch 'feat/sh-wpkh-verify-projection')` and `HEAD is now at 8eb51d7`.

- [ ] **Step 2: Confirm HEAD + clean tree**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && git rev-parse HEAD && git status --porcelain
```
Expected: `8eb51d7a24e6f8ab0b6641a27996e12e07a48322` then NO output (clean).

- [ ] **Step 3: Baseline test sweep (must be green before any change)**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./md/... ./gui/... ./address/...
```
Expected: `ok seedhammer.com/md`, `ok seedhammer.com/gui`, `ok seedhammer.com/address` (and the sub-packages `ok`/`[no test files]`). No FAIL.

> All subsequent commits use signed + DCO (`git commit -S -s`), author `Brian Goss <goss.brian@gmail.com>`, and **explicit-path** staging (no `git add -A`). Run all `go` commands from `/scratch/code/shibboleth/seedhammer-shwpkh` with `export PATH=$PATH:/home/bcg/.local/go/bin`.

---

## Task 1: Decoder touch-point — `classifyPolicy` arm + `InnerWpkh` discriminant

**Files:**
- Modify: `md/md.go` (`Template` struct ~`:1218`; `classifyPolicy` `tagSh` case `:1285-1300`; after `innerWshNesting` `:1331`; `summarize` return `:1355`)
- Modify test: `md/md_test.go` (ALREADY EXISTS, `package md`, already imports `testing` — APPEND, do not create)

- [ ] **Step 1: Write the failing decoder unit tests**

APPEND the following to the end of the existing `md/md_test.go` (it is `package md` and already imports `testing`, so the new functions can call unexported `classifyPolicy`/`innerWpkhNesting` and reference `node`/`tagSh`/`tagWpkh`/`tagWsh`/`tagSortedMulti`/`keyArgBody`/`childrenBody`/`multiKeysBody` directly — NO new import needed):

```go
// shWpkhNode is the canonical decoded sh(wpkh) tree (md/encode_singlesig.go:100-102):
// an sh with a single wpkh child referencing placeholder @0.
func shWpkhNode() node {
	inner := node{tag: tagWpkh, body: keyArgBody{index: 0}}
	return node{tag: tagSh, body: childrenBody{children: []node{inner}}}
}

// TestClassifyPolicyShWpkhRenders: classifyPolicy(sh(wpkh)) is PolicySingle (was
// PolicyComplex), so summarize marks it renderable. The new arm must NOT alter
// the existing sh(wsh(sortedmulti)) / bare sh(sortedmulti) classifications.
func TestClassifyPolicyShWpkhRenders(t *testing.T) {
	pol, k, m := classifyPolicy(shWpkhNode())
	if pol != PolicySingle || k != 0 || m != 0 {
		t.Fatalf("classifyPolicy(sh(wpkh)) = (%v,%d,%d), want (PolicySingle,0,0)", pol, k, m)
	}
}

// TestInnerWpkhNesting: innerWpkhNesting is true only for sh(wpkh); false for a
// bare wpkh, for sh(wsh(...)), and for a non-sh root.
func TestInnerWpkhNesting(t *testing.T) {
	if !innerWpkhNesting(shWpkhNode()) {
		t.Fatal("innerWpkhNesting(sh(wpkh)) = false, want true")
	}
	bareWpkh := node{tag: tagWpkh, body: keyArgBody{index: 0}}
	if innerWpkhNesting(bareWpkh) {
		t.Fatal("innerWpkhNesting(wpkh) = true, want false")
	}
	// sh(wsh(...)) must NOT be classified as inner-wpkh.
	wsh := node{tag: tagWsh, body: childrenBody{children: []node{{tag: tagSortedMulti, body: multiKeysBody{k: 1, indices: []uint8{0, 1}}}}}}
	shWsh := node{tag: tagSh, body: childrenBody{children: []node{wsh}}}
	if innerWpkhNesting(shWsh) {
		t.Fatal("innerWpkhNesting(sh(wsh(...))) = true, want false")
	}
}
```

- [ ] **Step 2: Run the tests to verify they FAIL**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./md/ -run 'TestClassifyPolicyShWpkhRenders|TestInnerWpkhNesting' -v
```
Expected: build failure `undefined: innerWpkhNesting` (the helper does not exist yet) — a compile-time FAIL. (`classifyPolicy(sh(wpkh))` would also return `PolicyComplex`, but the build fails first on the missing symbol.)

- [ ] **Step 3: Add the `InnerWpkh` field to `Template`**

In `md/md.go`, after the `InnerWsh bool` field (currently `:1218`), inside the `Template` struct:

```go
	// Meaningful only when Root==ScriptSh; false for every other root.
	InnerWsh bool
	// InnerWpkh is the sh(wpkh) single-sig discriminant: true iff Root==ScriptSh
	// AND the immediate sh child is a wpkh key (sh(wpkh) — BIP-49 P2SH-P2WPKH).
	// A consumer building a *bip380.Descriptor uses it to pick P2SH_P2WPKH for the
	// single-sig sh root, symmetric with InnerWsh for the sorted-multi sh root.
	// Meaningful only when Root==ScriptSh && Policy==PolicySingle.
	InnerWpkh bool
}
```

- [ ] **Step 4: Add the `sh(wpkh)→PolicySingle` arm to `classifyPolicy`**

In `md/md.go`, in `classifyPolicy`'s `case tagSh:` block, insert the inner-`wpkh` arm BEFORE the `sh(wsh(...))` arm (distinct inner tags make ordering safe, but placing it first keeps it obviously additive). The block currently reads:

```go
	case tagSh:
		if b, ok := tree.body.(childrenBody); ok && len(b.children) == 1 {
			inner := b.children[0]
			// sh(wsh(multi/sortedmulti))
			if inner.tag == tagWsh {
```

Change to:

```go
	case tagSh:
		if b, ok := tree.body.(childrenBody); ok && len(b.children) == 1 {
			inner := b.children[0]
			// sh(wpkh) — BIP-49 P2SH-P2WPKH single-sig nested-segwit.
			if inner.tag == tagWpkh {
				if _, ok := inner.body.(keyArgBody); ok {
					return PolicySingle, 0, 0
				}
			}
			// sh(wsh(multi/sortedmulti))
			if inner.tag == tagWsh {
```

- [ ] **Step 5: Add the `innerWpkhNesting` helper**

In `md/md.go`, immediately AFTER the `innerWshNesting` function (after its closing `}` at `:1331`), add:

```go

// innerWpkhNesting reports whether tree is an sh(wpkh) wrapper — the single-sig
// nested-segwit discriminant (BIP-49 P2SH-P2WPKH). It mirrors innerWshNesting:
// an sh with a single wpkh-key child. Returns false for a bare wpkh, for any
// sh(wsh(...))/sh(multi), and for any non-sh root.
func innerWpkhNesting(tree node) bool {
	if tree.tag != tagSh {
		return false
	}
	b, ok := tree.body.(childrenBody)
	if !ok || len(b.children) != 1 {
		return false
	}
	if b.children[0].tag != tagWpkh {
		return false
	}
	_, ok = b.children[0].body.(keyArgBody)
	return ok
}
```

- [ ] **Step 6: Set `InnerWpkh` in `summarize`**

In `md/md.go`, in `summarize`'s returned `Template` literal, after the `InnerWsh:` line (`:1355`):

```go
		InnerWsh:   innerWshNesting(d.tree),
		InnerWpkh:  innerWpkhNesting(d.tree),
	}
```

- [ ] **Step 7: Run the tests to verify they PASS**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./md/ -run 'TestClassifyPolicyShWpkhRenders|TestInnerWpkhNesting' -v
```
Expected: `--- PASS: TestClassifyPolicyShWpkhRenders` and `--- PASS: TestInnerWpkhNesting`, `ok seedhammer.com/md`.

- [ ] **Step 8: Confirm the whole `md` package still passes (no regression to existing classifications)**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./md/...
```
Expected: `ok seedhammer.com/md`.

- [ ] **Step 9: Commit**

```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && git add md/md.go md/md_test.go && git commit -S -s --author "Brian Goss <goss.brian@gmail.com>" -m "md: render sh(wpkh) as PolicySingle with InnerWpkh discriminant

classifyPolicy gains an sh(wpkh)->PolicySingle arm and summarize carries a
new Template.InnerWpkh field (set via innerWpkhNesting), mirroring the
InnerWsh precedent. This makes a decoded BIP-49 sh(wpkh) md1 renderable;
projection follows in the next commit."
```

---

## Task 2: Projection touch-point — `ScriptSh+PolicySingle+InnerWpkh → P2SH_P2WPKH`

**Files:**
- Modify: `gui/md1_expand.go` (`scriptForTemplate`, the `case md.PolicySingle:` block `:87-97`)
- Modify test: `gui/md1_expand_test.go`

- [ ] **Step 1: Write the failing projection unit test**

In `gui/md1_expand_test.go`, append (after `TestExpandedToDescriptorShNesting`, before the unsupported tests):

```go
// TestExpandedToDescriptorShWpkh (Task 2): a hand-built sh(wpkh) Template
// (Root=ScriptSh, Policy=PolicySingle, InnerWpkh=true) with one xpub-present key
// → expandOK + a P2SH_P2WPKH/Singlesig descriptor; address.Supported lights up.
func TestExpandedToDescriptorShWpkh(t *testing.T) {
	tpl := md.Template{N: 1, Root: md.ScriptSh, Policy: md.PolicySingle, Renderable: true, InnerWpkh: true}
	keys := []md.ExpandedKey{expandedKey(0, [4]byte{0x5a, 0x8, 0x4, 0xe3})}
	desc, status := expandedToDescriptor(tpl, keys)
	if status != expandOK {
		t.Fatalf("status = %v, want expandOK", status)
	}
	if desc.Script != bip380.P2SH_P2WPKH || desc.Type != bip380.Singlesig {
		t.Fatalf("desc = {Script:%v Type:%v}, want P2SH_P2WPKH/Singlesig", desc.Script, desc.Type)
	}
	if !address.Supported(desc) {
		t.Fatal("address.Supported(P2SH_P2WPKH singlesig) = false, want true (verify must light up)")
	}
	// The derived receive address round-trips and is a mainnet P2SH (3…).
	addr, err := address.Receive(desc, 0)
	if err != nil {
		t.Fatalf("address.Receive: %v", err)
	}
	if len(addr) == 0 || addr[0] != '3' {
		t.Fatalf("receive addr = %q, want a mainnet P2SH (3…) address", addr)
	}
	if _, _, found, err := address.Find(desc, addr, 20); err != nil || !found {
		t.Fatalf("Find(%s) found=%v err=%v", addr, found, err)
	}
}
```

- [ ] **Step 2: Run the test to verify it FAILS**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run TestExpandedToDescriptorShWpkh -v
```
Expected: FAIL — `scriptForTemplate` has no `ScriptSh` arm under `PolicySingle`, so `expandedToDescriptor` returns `expandUnsupported` → `status = expandUnsupported, want expandOK`.

- [ ] **Step 3: Add the projection arm + remove the stale NOTE**

In `gui/md1_expand.go`, the `case md.PolicySingle:` block currently reads:

```go
	case md.PolicySingle:
		switch tpl.Root {
		case md.ScriptWpkh:
			return bip380.P2WPKH, bip380.Singlesig, true
		case md.ScriptPkh:
			return bip380.P2PKH, bip380.Singlesig, true
		case md.ScriptTr:
			return bip380.P2TR, bip380.Singlesig, true
		}
		// NOTE: there is deliberately NO ScriptSh+singlesig (P2SH_P2WPKH) arm —
		// classifyPolicy never renders sh-wpkh on the Go side (R0-Minor).
	case md.PolicySortedMulti:
```

Change to (add the `ScriptSh` arm; delete the two NOTE lines):

```go
	case md.PolicySingle:
		switch tpl.Root {
		case md.ScriptWpkh:
			return bip380.P2WPKH, bip380.Singlesig, true
		case md.ScriptPkh:
			return bip380.P2PKH, bip380.Singlesig, true
		case md.ScriptTr:
			return bip380.P2TR, bip380.Singlesig, true
		case md.ScriptSh:
			// sh(wpkh) — BIP-49 P2SH-P2WPKH single-sig. Keyed on the InnerWpkh
			// discriminant, symmetric with the InnerWsh sorted-multi sh arm below.
			// Disjoint from PolicySortedMulti, so it can never collide with the
			// P2SH_P2WSH / bare-P2SH multisig arms.
			if tpl.InnerWpkh {
				return bip380.P2SH_P2WPKH, bip380.Singlesig, true
			}
		}
	case md.PolicySortedMulti:
```

> NOTE: this introduces a closing `}` for the inner `switch tpl.Root` that the deleted NOTE lines previously sat after. Keep the existing `case md.PolicySortedMulti:` exactly as-is below it. A bare `sh` under `PolicySingle` with `InnerWpkh==false` falls through to the final `return 0, 0, false` (unsupported) — correct.

- [ ] **Step 4: Run the test to verify it PASSES**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run TestExpandedToDescriptorShWpkh -v
```
Expected: `--- PASS: TestExpandedToDescriptorShWpkh`, `ok seedhammer.com/gui`.

- [ ] **Step 5: Confirm the existing no-collision regression still passes (UNCHANGED test)**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestExpandedToDescriptorShNesting|TestExpandedToDescriptorSinglesig|TestExpandedToDescriptorUnsortedMultiUnsupported' -v
```
Expected: all PASS (`TestExpandedToDescriptorShNesting` is untouched; `sh(wsh(sortedmulti))→P2SH_P2WSH` and bare `sh(sortedmulti)→P2SH` still hold).

- [ ] **Step 6: Commit**

```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && git add gui/md1_expand.go gui/md1_expand_test.go && git commit -S -s --author "Brian Goss <goss.brian@gmail.com>" -m "gui: project sh(wpkh) -> P2SH_P2WPKH so verify lights up

scriptForTemplate gains a ScriptSh arm under PolicySingle, keyed on the
InnerWpkh discriminant, returning (P2SH_P2WPKH, Singlesig). Replaces the
stale deliberate-absence NOTE. address/bip380 already derive P2SH-P2WPKH."
```

---

## Task 3: Address golden (load-bearing) + no-collision — full projection path

**Files:**
- Modify test: `gui/md1_expand_test.go`

This drives the **real** path (PRIMARY fixture, VF1/VF2): `EncodeSingleSig(ScriptShWpkh)` → `md.ExpandWalletPolicyChunks` (routes through the changed `summarize`/`classifyPolicy`/`InnerWpkh`) → `expandedToDescriptor` → `address.Receive/Change`. NOT a bypass.

- [ ] **Step 1: Write the failing golden + no-collision test**

In `gui/md1_expand_test.go`, append:

```go
// TestShWpkhGoldenAddress (Task 3, A1/I1 — load-bearing): a BIP-49 sh(wpkh) md1
// for the abandon seed, decoded through the REAL projection path
// (EncodeSingleSig(ScriptShWpkh) -> ExpandWalletPolicyChunks -> expandedToDescriptor),
// derives the byte-exact P2SH-P2WPKH receive[0]/change[0] golden.
func TestShWpkhGoldenAddress(t *testing.T) {
	m := abandonAboutMnemonic()
	path := singleSigPath(49) // m/49'/0'/0'
	xpub, masterFP, err := deriveAccountXpub(m, "", &chaincfg.MainNetParams, path)
	if err != nil {
		t.Fatalf("deriveAccountXpub: %v", err)
	}
	const wantAcctXpub = "xpub6C6nQwHaWbSrzs5tZ1q7m5R9cPK9eYpNMFesiXsYrgc1P8bvLLAet9JfHjYXKjToD8cBRswJXXbbFpXgwsswVPAZzKMa1jUp2kVkGVUaJa7"
	if xpub != wantAcctXpub {
		t.Fatalf("BIP-49 account xpub = %s, want %s", xpub, wantAcctXpub)
	}
	cc, pk, _, err := decodeXpubBytes(xpub)
	if err != nil {
		t.Fatalf("decodeXpubBytes: %v", err)
	}
	var fp [4]byte
	binary.BigEndian.PutUint32(fp[:], masterFP)

	strs, err := md.EncodeSingleSig(cc, pk, fp, originComponents(path), md.ScriptShWpkh)
	if err != nil {
		t.Fatalf("EncodeSingleSig: %v", err)
	}
	tpl, keys, err := md.ExpandWalletPolicyChunks(strs)
	if err != nil {
		t.Fatalf("ExpandWalletPolicyChunks: %v", err)
	}
	// The real decode now renders it (Task 1 touch-point under test).
	if tpl.Root != md.ScriptSh || tpl.Policy != md.PolicySingle || !tpl.Renderable || !tpl.InnerWpkh {
		t.Fatalf("decoded tpl = {Root:%v Policy:%v Renderable:%v InnerWpkh:%v}, want ScriptSh/PolicySingle/true/true", tpl.Root, tpl.Policy, tpl.Renderable, tpl.InnerWpkh)
	}
	if tpl.InnerWsh {
		t.Fatal("InnerWsh = true for sh(wpkh); want false (discriminants independent)")
	}

	desc, status := expandedToDescriptor(tpl, keys)
	if status != expandOK {
		t.Fatalf("status = %v, want expandOK", status)
	}
	if desc.Script != bip380.P2SH_P2WPKH || desc.Type != bip380.Singlesig {
		t.Fatalf("desc = {Script:%v Type:%v}, want P2SH_P2WPKH/Singlesig", desc.Script, desc.Type)
	}
	r0, err := address.Receive(desc, 0)
	if err != nil {
		t.Fatalf("address.Receive: %v", err)
	}
	c0, err := address.Change(desc, 0)
	if err != nil {
		t.Fatalf("address.Change: %v", err)
	}
	const wantRecv0 = "37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf"
	const wantChange0 = "34K56kSjgUCUSD8GTtuF7c9Zzwokbs6uZ7"
	if r0 != wantRecv0 {
		t.Fatalf("BIP-49 sh(wpkh) receive[0] = %s, want %s", r0, wantRecv0)
	}
	if c0 != wantChange0 {
		t.Fatalf("BIP-49 sh(wpkh) change[0] = %s, want %s", c0, wantChange0)
	}
}

// TestShWpkhNoCollision (Task 3, A2/I2): for the SAME key material, the
// sh(wpkh) P2SH-P2WPKH receive[0] differs from both the sh(wsh(sortedmulti))
// P2SH-P2WSH and the bare sh(sortedmulti) P2SH receive[0]. Built directly so
// the three sh shapes share one key set.
func TestShWpkhNoCollision(t *testing.T) {
	k1 := []md.ExpandedKey{expandedKey(0, [4]byte{0x5a, 0x8, 0x4, 0xe3})}
	k2 := []md.ExpandedKey{
		expandedKey(0, [4]byte{0x5a, 0x8, 0x4, 0xe3}),
		expandedKey(1, [4]byte{0xdd, 0x4f, 0xad, 0xee}),
	}
	shWpkh := md.Template{N: 1, Root: md.ScriptSh, Policy: md.PolicySingle, Renderable: true, InnerWpkh: true}
	bare := md.Template{N: 2, Root: md.ScriptSh, Policy: md.PolicySortedMulti, K: 1, M: 2, Renderable: true, InnerWsh: false}
	nested := md.Template{N: 2, Root: md.ScriptSh, Policy: md.PolicySortedMulti, K: 1, M: 2, Renderable: true, InnerWsh: true}

	dWpkh, sWpkh := expandedToDescriptor(shWpkh, k1)
	dBare, sBare := expandedToDescriptor(bare, k2)
	dNested, sNested := expandedToDescriptor(nested, k2)
	if sWpkh != expandOK || sBare != expandOK || sNested != expandOK {
		t.Fatalf("statuses = %v/%v/%v, want all expandOK", sWpkh, sBare, sNested)
	}
	if dWpkh.Script != bip380.P2SH_P2WPKH || dBare.Script != bip380.P2SH || dNested.Script != bip380.P2SH_P2WSH {
		t.Fatalf("scripts = %v/%v/%v, want P2SH_P2WPKH/P2SH/P2SH_P2WSH", dWpkh.Script, dBare.Script, dNested.Script)
	}
	aWpkh, _ := address.Receive(dWpkh, 0)
	aBare, _ := address.Receive(dBare, 0)
	aNested, _ := address.Receive(dNested, 0)
	if aWpkh == aBare || aWpkh == aNested || aBare == aNested {
		t.Fatalf("collision: P2SH_P2WPKH=%s P2SH=%s P2SH_P2WSH=%s must be pairwise-distinct", aWpkh, aBare, aNested)
	}
}
```

Add the `"encoding/binary"` import to `gui/md1_expand_test.go`'s import block (it currently imports `testing`, `hdkeychain`, `address`, `bip32`, `bip380`, `md`):

```go
import (
	"encoding/binary"
	"testing"

	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"seedhammer.com/address"
	"seedhammer.com/bip32"
	"seedhammer.com/bip380"
	"seedhammer.com/md"
)
```

Add the `chaincfg` import too (used by `TestShWpkhGoldenAddress`):

```go
	"github.com/btcsuite/btcd/chaincfg/v2"
```

(Place `chaincfg` after the `hdkeychain` import, matching the ordering in `gui/singlesig_restore_test.go:7-8`.)

- [ ] **Step 2: Run the tests to verify they PASS** (Task 1+2 already make them pass; this is the load-bearing confirmation)

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestShWpkhGoldenAddress|TestShWpkhNoCollision' -v
```
Expected: `--- PASS: TestShWpkhGoldenAddress`, `--- PASS: TestShWpkhNoCollision`, `ok seedhammer.com/gui`.

> If `TestShWpkhGoldenAddress` FAILS at the `address.Receive` assertion, STOP — the golden is load-bearing (I1). Do not edit the expected constant; debug the projection. (The values are empirically verified at VF2.)

- [ ] **Step 3: Confirm the existing single-sig + sh + wsh goldens are unperturbed (I5)**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'TestSingleSigRestore|TestExpandedToDescriptor' -v 2>&1 | grep -E 'PASS|FAIL|ok '
```
Expected: every `TestSingleSigRestore*` and `TestExpandedToDescriptor*` PASS, including `TestSingleSigRestoreWpkhKnownAddress` (BIP-84 `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`) and `TestSingleSigRestoreDescriptorScripts/sh-wpkh`. No FAIL.

- [ ] **Step 4: Commit**

```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && git add gui/md1_expand_test.go && git commit -S -s --author "Brian Goss <goss.brian@gmail.com>" -m "gui: byte-exact BIP-49 sh(wpkh) golden + no-collision through the real path

TestShWpkhGoldenAddress drives EncodeSingleSig(ScriptShWpkh) ->
ExpandWalletPolicyChunks -> expandedToDescriptor (exercising the new
classifier + projection arms, not a bypass) and asserts the abandon-seed
P2SH-P2WPKH receive[0]=37Vuc… / change[0]=34K56…. TestShWpkhNoCollision
pins P2SH_P2WPKH vs P2SH_P2WSH vs bare P2SH pairwise-distinct."
```

---

## Task 4: Fuzz harness (A6) + stale-comment cleanup (R3) + no-secret grep

**Files:**
- Modify: `gui/md1_expand_fuzz_test.go` (`isBip380ExpressibleShape` `:13-23`; the `at(5)` bits + `Template` build `:48-55`)
- Modify: `md/md.go` (`:1173-1178` comment) and `gui/singlesig_restore.go` (`:25-29` comment)

- [ ] **Step 1: Update the fuzz harness — synthesize `InnerWpkh` AND add `ScriptSh` to the expressible-shape arm (both halves, VF6)**

In `gui/md1_expand_fuzz_test.go`:

(a) Extend `isBip380ExpressibleShape` to count `ScriptSh` as expressible under `PolicySingle`. Current:

```go
	switch policy {
	case md.PolicySingle:
		return root == md.ScriptWpkh || root == md.ScriptPkh || root == md.ScriptTr
	case md.PolicySortedMulti:
		return root == md.ScriptWsh || root == md.ScriptSh
	}
```

Change to:

```go
	switch policy {
	case md.PolicySingle:
		return root == md.ScriptWpkh || root == md.ScriptPkh || root == md.ScriptTr || root == md.ScriptSh
	case md.PolicySortedMulti:
		return root == md.ScriptWsh || root == md.ScriptSh
	}
```

> NOTE: the harness models the `scriptForTemplate` outcome, not the decode shape. Under `PolicySingle`, a `ScriptSh` template reaches `expandOK` **only** when `InnerWpkh==true` — but for `InnerWpkh==false` it falls through to unsupported (status≠expandOK), so the invariant `expandOK ⇒ expressible` still holds for both InnerWpkh values. (`expandOK` is the only side that must imply expressible; an expressible shape need not always reach expandOK.)

(b) Synthesize `InnerWpkh` on a free bit and thread it into the `Template`. Current `at(5)` usage:

```go
		renderable := at(4)&1 == 1
		innerWsh := at(5)&1 == 1
		xpubPresent := at(5)&2 == 2
		wildcardHardened := at(5)&4 == 4

		tpl := md.Template{
			N: n, Root: root, Policy: policy, K: k, M: n,
			Renderable: renderable, InnerWsh: innerWsh,
		}
```

Change to:

```go
		renderable := at(4)&1 == 1
		innerWsh := at(5)&1 == 1
		xpubPresent := at(5)&2 == 2
		wildcardHardened := at(5)&4 == 4
		innerWpkh := at(5)&8 == 8

		tpl := md.Template{
			N: n, Root: root, Policy: policy, K: k, M: n,
			Renderable: renderable, InnerWsh: innerWsh, InnerWpkh: innerWpkh,
		}
```

(c) Add a seed corpus entry that hits the new arm (so the new shape is exercised from the first run). After the existing `f.Add` lines:

```go
	f.Add([]byte{2, 0, 1, 1, 1, 10}) // sh single, InnerWpkh(bit8)+xpubPresent(bit2)=10, 1 key
```

> `root=2(ScriptSh)`, `policy=0(PolicySingle)`, `n=1`, `renderable=at(4)&1=1`, `at(5)=10 → innerWpkh(8)+xpubPresent(2)`, wildcardHardened off → reaches the new `(P2SH_P2WPKH, Singlesig, true)` arm with a valid `<0;1>/*` use-site.

- [ ] **Step 2: Run the fuzz corpus (seed entries only) — invariants hold, new shape exercised**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run 'FuzzExpandedToDescriptor$' -v
```
Expected: `--- PASS: FuzzExpandedToDescriptor` (runs the seed corpus deterministically; no `expandOK for non-bip380 shape`, no panic).

- [ ] **Step 3: Run a short active fuzz pass — no new crash**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run '^$' -fuzz=FuzzExpandedToDescriptor -fuzztime=15s
```
Expected: `PASS` / `ok seedhammer.com/gui` — `elapsed: 15s … new interesting: …`, NO `FAIL`, NO failing input written to `testdata/fuzz/`. (If a crash corpus file IS written, STOP and debug — the invariant was violated.)

- [ ] **Step 4: Update the stale comment in `md/md.go` (R3)**

The `ScriptShWpkh` doc comment (`:1173-1178`) says the decoder summarizes sh(wpkh) to `Root==ScriptSh` — that part stays TRUE (the root tag is still `Sh`). It does NOT claim non-renderability, so it needs only a one-line addition for accuracy. Current:

```go
	// ScriptShWpkh is APPENDED after ScriptTr (R0-M2): a BIP-49 nested-segwit
	// sh(wpkh) single-sig wrapper. It is an EncodeSingleSig input discriminant
	// only — the decoder summarizes an sh(wpkh) wire to Root==ScriptSh (the
	// on-wire root tag is Sh). Appending (not inserting) preserves the existing
	// values so rootScriptKind/#10b consumers are unaffected.
	ScriptShWpkh
```

Change to:

```go
	// ScriptShWpkh is APPENDED after ScriptTr (R0-M2): a BIP-49 nested-segwit
	// sh(wpkh) single-sig wrapper. It is an EncodeSingleSig input discriminant
	// only — the decoder summarizes an sh(wpkh) wire to Root==ScriptSh (the
	// on-wire root tag is Sh) and carries the Template.InnerWpkh discriminant so
	// the projection picks P2SH_P2WPKH. Appending (not inserting) preserves the
	// existing values so rootScriptKind/#10b consumers are unaffected.
	ScriptShWpkh
```

- [ ] **Step 5: Update the stale comment in `gui/singlesig_restore.go` (R3)**

The `:25-29` comment asserts the md1 classifier *drops* single-key sh(wpkh), which is now FALSE. Current:

```go
// R0-I1 (Option Y): the descriptor is built DIRECTLY, bypassing the md1
// classifier — classifyPolicy/scriptForTemplate drop single-key sh(wpkh)
// (md1_expand.go has no ScriptSh+single arm), so an md1→classify→descriptor path
// would lose the BIP-49 restore doc. Building the *bip380.Descriptor from the
// engraved xpub + chosen script keeps all 4 types, including sh-wpkh.
```

Change to:

```go
// R0-I1 (Option Y): the restore doc builds the descriptor DIRECTLY from the
// engraved xpub + chosen script (rather than re-encoding to md1 and decoding
// back). This keeps the restore-doc path independent of the verify-projection
// path and threads the REAL ParentFingerprint for a canonical xpub. (The md1
// verify path DOES now render sh(wpkh) → P2SH_P2WPKH via the InnerWpkh
// discriminant; both paths agree on the BIP-49 descriptor.)
```

- [ ] **Step 6: Build + grep gate — confirm the stale NOTE is gone and no secret leaks**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go build ./... && echo "BUILD OK" && \
  ! grep -rn "deliberately NO ScriptSh" gui/ md/ && echo "NOTE GONE" && \
  ! grep -rn "drop single-key sh(wpkh)\|drops single-key sh(wpkh)" gui/ md/ && echo "STALE-CLAIM GONE" && \
  ! grep -rIn "xprv\|tprv" md/md.go gui/md1_expand.go gui/md1_expand_test.go && echo "NO PRIVATE MATERIAL IN TOUCHED FILES"
```
Expected: `BUILD OK`, `NOTE GONE`, `STALE-CLAIM GONE`, `NO PRIVATE MATERIAL IN TOUCHED FILES`. (Each `! grep` succeeds — exit 0 — only when there is NO match.)

- [ ] **Step 7: Commit**

```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && git add gui/md1_expand_fuzz_test.go md/md.go gui/singlesig_restore.go && git commit -S -s --author "Brian Goss <goss.brian@gmail.com>" -m "test+docs: fuzz the sh(wpkh) arm; refresh stale comments

FuzzExpandedToDescriptor now synthesizes InnerWpkh (free bit 8 of at(5))
and counts ScriptSh as PolicySingle-expressible, so the new P2SH_P2WPKH
arm is exercised and the expandOK-implies-expressible invariant holds.
Refreshes the now-false 'decoder never renders sh-wpkh' comments in
md/md.go and gui/singlesig_restore.go."
```

---

## Task 5: No-regression sweep + vet + alloc gate

**Files:** none (verification).

- [ ] **Step 1: Full test sweep across every touched + dependent package**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./md/... ./gui/... ./address/... ./bip380/...
```
Expected: `ok seedhammer.com/md`, `ok seedhammer.com/gui`, `ok seedhammer.com/address`, `ok seedhammer.com/bip380` (+ sub-packages `ok`/`[no test files]`). No FAIL.

- [ ] **Step 2: `go vet`**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go vet ./md/... ./gui/... ./address/... ./bip380/...
```
Expected: no output, exit 0.

- [ ] **Step 3: Build the whole module**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go build ./...
```
Expected: no output, exit 0.

- [ ] **Step 4: 0-alloc gate (the on-device firmware budget — `TestAllocs` lives in `gui/gui_test.go`)**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && export PATH=$PATH:/home/bcg/.local/go/bin && go test ./gui/ -run TestAllocs -v 2>&1 | grep -E 'PASS|FAIL|ok '
```
Expected: `--- PASS: TestAllocs` (the projection change is pure switch logic — no new heap allocation on the verify hot path). If FAIL, STOP and debug (the new arm must not allocate beyond the existing `expandedToDescriptor` budget).

- [ ] **Step 5: Confirm the branch diff is exactly the intended files**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-shwpkh && git diff --stat 8eb51d7 HEAD
```
Expected: changes ONLY in `md/md.go`, `md/md_test.go`, `gui/md1_expand.go`, `gui/md1_expand_test.go`, `gui/md1_expand_fuzz_test.go`, `gui/singlesig_restore.go`. NOTHING in `address/` or `bip380/`.

- [ ] **Step 6: (No commit — verification task.)** If all green, the branch is ready for the mandatory post-implementation adversarial execution review (R0 over the whole diff) before any merge/PR.

---

## Self-Review

**1. Spec coverage (every acceptance gate A1–A7 and invariant I1–I5 maps to a task):**

| Spec item | Covered by |
|---|---|
| Scope IN-1 (decoder renderable + `InnerWpkh`) | Task 1 |
| Scope IN-2 (projection arm) | Task 2 |
| Scope IN-3 (TDD acceptance) | Tasks 1–5 |
| A1 (byte-exact BIP-49 golden via real path) | Task 3, `TestShWpkhGoldenAddress` |
| A2 (no discriminant collision, pairwise-distinct) | Task 3, `TestShWpkhNoCollision` |
| A3 (Script+status correctness, `address.Supported`) | Task 2, `TestExpandedToDescriptorShWpkh` |
| A4 (decode renders it; `InnerWpkh` true, `InnerWsh` false) | Task 1 (`classifyPolicy`/`innerWpkhNesting`) + Task 3 (tpl assertions) |
| A5 (no-regression: sh-wsh/bare-sh/wpkh/pkh/tr + BIP-84 golden + unsupported) | Task 2 Step 5, Task 3 Step 3, Task 5 Step 1 |
| A6 (fuzz update, both halves) | Task 4 Steps 1–3 |
| A7 (build + vet + grep NOTE removed) | Task 4 Step 6, Task 5 Steps 2–3 |
| I1 (correct P2SH-P2WPKH derivation, re-derived) | Task 3 `TestShWpkhGoldenAddress` |
| I2 (no collision with P2SH-P2WSH, disjoint cases) | Task 2 (disjoint switch), Task 3 `TestShWpkhNoCollision` |
| I3 (display-only fallback preserved) | Task 2 Step 3 (bare-sh `InnerWpkh==false` falls to unsupported); `TestExpandedToDescriptorUnsortedMultiUnsupported` unchanged (Task 2 Step 5) |
| I4 (mainnet-only) | No new network path; `md1_expand.go:61` pin untouched (VF7) |
| I5 (no-regression on existing projections/decode) | Task 1 Step 8, Task 2 Step 5, Task 3 Step 3, Task 5 Step 1 |
| R3 (stale-comment cleanup) | Task 4 Steps 4–5 (NOTE in `md1_expand.go` removed in Task 2 Step 3; `md.go`/`singlesig_restore.go` refreshed) |
| Risk R1 (fixture construction routes through new code) | Task 3 uses the PRIMARY fixture (`ExpandWalletPolicyChunks` → `summarize`), confirmed non-bypass (VF1/VF2). FALLBACK (hand-built `md.Template`) used by the unit tests in Tasks 1–2 |
| Risk R4 (additive ordering, no shadowing) | Task 1 Step 4 (distinct inner tag, placed first); Task 1 Step 8 + Task 5 Step 1 pin every existing arm |

No gaps.

**2. Placeholder scan:** No "TBD"/"implement later"/"add error handling"/"similar to Task N". Every code step shows the full code; every test step shows the full assertion; every run step shows the exact command + expected output. The golden constants (`37Vuc…`, `34K56…`, the account xpub, masterFP `0x73c5da0a`, chunk count 3) are pinned and empirically verified (VF2/VF3).

**3. Type/name consistency:** `InnerWpkh` (field), `innerWpkhNesting` (helper) — consistent across Task 1 (def), Task 2 (consumed in `scriptForTemplate`), Task 3 (asserted on `tpl`), Task 4 (synthesized in fuzz). `expandedToDescriptor`/`scriptForTemplate`/`expandOK`/`expandUnsupported` match `gui/md1_expand.go`. `bip380.P2SH_P2WPKH`/`bip380.Singlesig` match `address/address.go:144`/`bip380.go`. Helper signatures (`EncodeSingleSig`, `ExpandWalletPolicyChunks`, `deriveAccountXpub`, `decodeXpubBytes`, `originComponents`, `singleSigPath`, `address.Supported`) verified against source (VF8). Test helpers (`expandedKey`, `goldenXpub`, `stdUseSite`, `abandonAboutMnemonic`) exist in-package. Imports added: `encoding/binary` + `chaincfg/v2` to `gui/md1_expand_test.go` (Task 3 Step 1). `md/md_test.go` is an EXISTING in-package test file (`package md`, already imports `testing`) — Task 1 APPENDS to it (not "create"; verified the file exists and has no `shWpkhNode`/`TestClassifyPolicyShWpkhRenders`/`TestInnerWpkhNesting` symbol). Being in-package, the appended tests reach unexported `classifyPolicy`/`innerWpkhNesting`/`node`/`tagSh`/`tagWpkh`/`tagWsh`/`tagSortedMulti`/`keyArgBody`/`childrenBody`/`multiKeysBody` (all package-internal — confirmed; `classifyPolicy` returns `(PolicyKind,int,int)`).

**Mainnet-only throughout (D1/I4). No code before this plan passes the opus R0 gate (0C/0I).**
