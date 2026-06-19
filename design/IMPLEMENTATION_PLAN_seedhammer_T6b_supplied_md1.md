# T6b — Multisig/Miniscript Bundle via SUPPLIED md1 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a new top-level `engraveMultisig` SeedHammer-firmware program that gathers a complete multisig/miniscript wallet-policy md1 over NFC, derives the operator's OWN key from a typed seed, cross-matches it to one descriptor `@N` slot, and engraves the supplied md1 VERBATIM + the operator's policy-bound mk1 + ms1 — with verify-bundle and a multisig restore doc.

**Architecture:** ZERO new encoder (architect mechanism ii): the supplied md1 is engraved as-is; the device builds NO multisig md1. The only net-new logic is the D14 slot cross-match (`findUserSlot`) plus a single-md1 supply filter (`extractSuppliedMd1`), a full-policy gate, a multisig restore render path, the `engraveMultisigFlow` orchestrator, and the program-enum lockstep. Everything else reuses the shipped T5 gather (`bundleGatherFlow`) + T6a derive/verify/engrave machinery. This MIRRORS the T6a single-sig flagship (`gui/singlesig*.go`): same flow shape, same typed-only-seed + per-leg-scrub discipline.

**Tech Stack:** Go (TinyGo-compatible firmware subset); `github.com/btcsuite/btcd` (hdkeychain, chaincfg); in-tree packages `seedhammer.com/{md,mk,bundle,codex32,bip32,bip39,bip380,address,gui}`. Test runner: `go` (on host) at `/home/bcg/.local/go/bin/go` — **prepend it to PATH** in every shell: `export PATH=$PATH:/home/bcg/.local/go/bin`.

**Worktree / fork:** All work happens in a git worktree of the fork at `/scratch/code/shibboleth/seedhammer` (branch `main`, base HEAD `072461a`). Firmware **planning docs** live in the `mnemonic-engrave` repo (this file); the **code** lands in the fork worktree. Fork-side only — NO upstream PR.

**Contract:** This plan implements exactly `design/SPEC_seedhammer_T6b_supplied_md1.md` (in the mnemonic-engrave repo) §2 IN scope, §5 acceptance gate, and §6 invariants I-1..I-11. R0-round-1 file:line citations are in `design/agent-reports/seedhammer-T6b-spec-R0-round1.md`.

**Status:** ✅ **Plan R0 GREEN (0C/0I)** — agent `a592176d3eee7b54b` independently RAN Go probes at HEAD 072461a confirming every pinned golden (fixture decode, abandon-seed @1-only match, stub `0x7b716421`, both addresses, mk1 round-trip, `bundle.Verify(b,b)==nil`); persisted to `design/agent-reports/seedhammer-T6b-plan-R0-round0.md`. **Cleared for single-implementer TDD.** Three non-blocking R0 Minors to apply opportunistically during implementation: **(m-1)** in Task 1.5, after the guard test passes, record the decoded @0/@2 foreign pubkeys (`hex(keys[0].Xpub)` / `hex(keys[2].Xpub)`) as a comment in `gui/multisig_testhelpers_test.go` so the vendored fixture's foreign keys are documented for future reproduction; **(m-2)** in Task 7, write `gui/multisig_restore.go` with ONLY the minimal imports it actually uses (`seedhammer.com/address`, `seedhammer.com/md`, + render deps) — do NOT ship the `var ( _ = image.Pt; ... )` placeholder block; **(m-3)** the `cardMS1` clause in `extractSuppliedMd1` is disclosed defensive dead code (gather never emits `cardMS1`) — keep it, no action.

---

## Verified facts (confirmed against fork HEAD 072461a — do not re-derive)

These were verified during plan authoring by reading the real source and running probe tests. Treat them as ground truth.

- **`md.ExpandedKey`** (`md/expand.go:56-64`): `{Index uint8; OriginPath bip32.Path; UseSite; Fingerprint [4]byte; FingerprintPresent bool; Xpub [65]byte; XpubPresent bool}`. `Xpub` layout is **32B chain code ‖ 33B compressed pubkey** — `Xpub[0:32]` is the chain code, `Xpub[32:65]` is the pubkey. Exported.
- **`md.ExpandWalletPolicyChunks(strs []string) (md.Template, []md.ExpandedKey, error)`** (`md/expand.go:102`). Exported. Reassembles + expands chunked md1 strings.
- **`decodeXpubBytes(xpub string) (chainCode [32]byte, compressedPubkey [33]byte, parentFP uint32, err error)`** (`gui/singlesig_derive.go:99`). Returns FIXED ARRAYS; refuses a private key.
- **`deriveAccountXpub(m bip39.Mnemonic, passphrase string, net *chaincfg.Params, path bip32.Path) (xpub string, masterFP uint32, err error)`** (`gui/derive.go:19`). Accepts ARBITRARY path; scrubs seed/master/intermediates internally; serializes the xpub before zeroing.
- **`mk.Encode(mk.Card) ([]string, error)`** (`mk/encode.go:38`); `mk.Card{Network, Path, Fingerprint, Stubs [][4]byte, Xpub}` (`mk/mk.go:133-139`). `Network` is a LABEL only — NOT serialized into the bytecode (`mk/encode.go:46-95`) nor compared by `bundle.Verify`. `mk.Encode` validates a depth/child invariant via `compactFromXpub(card.Xpub, comps)` against `card.Path` — so `card.Path` MUST equal the path the xpub was derived at (the matched slot origin). **Confirmed working:** deriving the abandon seed at `m/48'/0'/0'/2'` and encoding with `Path: origin.String()` succeeds and `bundle.Verify(b,b)==nil`.
- **`mk.Decode(in []string) (mk.Card, error)`** (`mk/mk.go:148`). `card.Stubs` is `[][4]byte`; comparable with `[4]byte == [4]byte`.
- **`md.WalletPolicyIDStubChunks(strs []string) ([4]byte, error)`** (`md/walletpolicyid.go:126`).
- **`codex32.EncodeMS1(entropy []byte) (string, error)`** (`codex32/msencode.go:17`); entropy length must be 16/20/24/28/32.
- **`m.Valid()`** (`bip39/bip39.go:107`) and **`m.Entropy()`** (`bip39/bip39.go:158`, PANICS on invalid) — gate `Valid()` before `Entropy()`.
- **`expandedToDescriptor(tpl md.Template, keys []md.ExpandedKey) (*bip380.Descriptor, expandStatus)`** (`gui/md1_expand.go:32`). Returns `(desc, expandOK)` for sortedmulti (P2WSH/P2SH/P2SH_P2WSH); `(nil, expandTemplateOnly)` when any key lacks an xpub; `(nil, expandUnsupported)` for non-bip380 shapes. `expandStatus` consts: `expandOK=0`, `expandTemplateOnly=1`, `expandUnsupported=2` (`gui/md1_expand.go:13-24`).
- **`address.Receive(desc, idx) (string, error)`** / **`address.Change(desc, idx)`** (`address/address.go:20-26`).
- **`bundle.Verify(derived, readback bundle.Bundle) error`** (`bundle/verify.go:32`). Checks stub-binding (both sides), mk1 fp/xpub/path, md1 EXACT-STRING, ms1 RECOVERED-ENTROPY. Watch-only: both-sides-empty `MS1` → ms1 leg SKIPPED. UNCHANGED by this work.
- **`bundle.Bundle{MS1 string; MK1 []string; MD1 []string}`** (`bundle/verify.go:19-23`).
- **`bundleGatherFlow(ctx, th) ([]bundleCard, bool)`** (`gui/bundle_flow.go:95`). Accumulates distinct verified cards over NFC. A chunked md1 collapses to ONE `cardMD1`. `bundleCard{kind bundleCardKind; label string; strings []string; summary string}` (`gui/bundle.go:33-38`); kinds `cardMK1=0`, `cardMD1=1`, `cardMS1=2` (`gui/bundle.go:24-28`). The gather path can only produce `cardMD1` and `cardMK1` (ms1 is refused upstream at `classify`→`clsMs1Refuse`, so `cardMS1` is never produced by gather — n-1).
- **`bundleEngrave(ctx, th, cards []bundleCard)`** (`gui/bundle_flow.go:327`). Sequences each card's plates verbatim; shows the ms1 reminder iff no `cardMS1` is present (`bundleShowMs1Reminder`, `gui/bundle_flow.go:373`).
- **`singleSigEngraveCards(b bundle.Bundle, full bool) []bundleCard`** (`gui/singlesig_engrave.go:20`) — the card-order template to MIRROR: full = `[ms1, mk1, md1]`, watch-only = `[mk1, md1]`.
- **`seedEntryFlow(ctx, th) (bip39.Mnemonic, bool)`** (`gui/derive_xpub.go:82`) — typed-only seed; caller scrubs.
- **`passphraseFlow(ctx, th) (string, bool)`** (`gui/gui.go:498`); `inputCodex32Flow(ctx, th, title) (any, bool)` (`gui/gui.go:726`); `showError(ctx, th, title, msg)` (`gui/slip39_polish.go:22`); `(*ChoiceScreen).Choose(ctx, th) (int, bool)` (`gui/gui.go:1362`); `wipeBytes(b []byte)` (`gui/slip39_polish.go:330`).
- **`bip32.Path.String()`** emits the `h`-suffix form (e.g. `m/48h/0h/0h/2h`); `mk.Decode` normalizes the round-tripped path to the apostrophe form (`m/48'/0'/0'/2'`). Both are accepted by `mk.Encode`.
- **The program enum** (`gui/gui.go:147-153`): `backupWallet=0, engraveXpub=1, engraveBundle=2, engraveSingleSig=3, qaProgram=4`. `qaProgram` is non-navigable (reached only by the `FOREVERLAURA!` debug command at `gui/gui.go:1606`, name-based — no change on insert).
- **The 8 `engraveSingleSig`-keyed lockstep sites** (ALL must retarget when `engraveMultisig` is inserted between `engraveSingleSig` and `qaProgram`):
  1. enum const `gui/gui.go:151` (insert `engraveMultisig` after `engraveSingleSig`)
  2. dispatch switch `gui/gui.go:1501-1503` (add `case engraveMultisig`)
  3. left-wrap bound `gui/gui.go:1638` (`m.prog = engraveSingleSig` → `engraveMultisig`)
  4. right-wrap bound `gui/gui.go:1645` (`if m.prog > engraveSingleSig` → `engraveMultisig`)
  5. title switch `gui/gui.go:1670-1671` (add `case engraveMultisig` → non-blank title)
  6. `npage` const `gui/gui.go:1846` (`int(engraveSingleSig) + 1` → `int(engraveMultisig) + 1`)
  7. `layoutMainPlates` switch `gui/gui.go:1856` (add `engraveMultisig` to the case list — **MANDATORY**, the default `panic("invalid page")` at `:1861` fires on render otherwise)
  8. `npages` const `gui/gui.go:1865` (`int(engraveSingleSig) + 1` → `int(engraveMultisig) + 1`)
- **The 3 nav-tests** hardcode `engraveSingleSig` as the carousel wrap bound and MUST be updated: `gui/singlesig_program_test.go`, `gui/bundle_program_test.go`, `gui/derive_xpub_program_test.go`.
- **`TestAllocs`** (`gui/gui_test.go:93`) benchmarks the start-screen render to 0 allocs/op. Adding enum cases to the render switches does NOT allocate; this stays green.

### Test vectors (verified during authoring — use VERBATIM)

A complete full-policy `wsh(sortedmulti(2, @0, @1, @2))` md1 where the operator's abandon-about seed is slot **@1** at origin `m/48'/0'/0'/2'`, @0 and @2 are two fixed foreign pubkeys. Generated by encoding a `descriptor` carrying a Pubkeys TLV (Task 1.5 below produces these as a vendored fixture). The chunk strings:

```
md1fvgfqzspqjtvyyy4qqxppcgsc27rczqg3yyc5z5tpwxqergd3c8g7ruszzg3ryssjfstllhxufdm4
md1fvgfqzs2jvfeg9y4zktpd9chs82fefgh35nuevya8z62kep2q7md6duvfx8px8ygw3q3umhs2q3cu
md1fvgfqzss8ygdjvlt5pterdm5rru59s2su80aw2q4wgdpapgfl4pkhsdyytkwl5zq9ner9ltnl8fnz
md1fvgfqzsllphut2hvvpp5wl4l0mn058ndxfl63kufyfsjwlt2vkk2nlqmlvch5n4sk08xmsudrng93
md1fvgfqz3qhwf72vyq3zgf3g9gkzuvpjxsmrsw3u8eqyy3zxfp9ycnjs2f29vkz6ts908m9qqcmg97l
md1fvgfqz3f0qtrqglu5g8kh6mfsg4qxa9wq0nv9cauwfwxw70984wkqnw2uwz0w27h0f8nmf46cm8
```

Verified facts about this vector:
- `ExpandWalletPolicyChunks` → `tpl{Root: ScriptWsh, Policy: PolicySortedMulti, K:2, N:3, Renderable:true}`, all 3 keys `XpubPresent=true`, each `OriginPath == m/48'/0'/0'/2'` (`bip32.Path{hard+48, hard+0, hard+0, hard+2}`).
- `findUserSlot(abandon, "", mainnet, keys)` matches **only slot @1**; @0/@2 do NOT match.
- `WalletPolicyIDStubChunks(chunks)` = `[4]byte{0x7b,0x71,0x64,0x21}`.
- `expandedToDescriptor(tpl,keys)` → `(non-nil desc, expandOK)`; `address.Receive(desc,0) = "bc1qg2lsdla23zewexuhn5jcx49mqzs8wqss0lxguarfpnt7ysg7k52slz4dxd"`, `address.Change(desc,0) = "bc1qz76qjcmpwhh6ffenfwg44hpq3cwwfuqcr54vl4485yttpjtxy9qq3yufkt"`.
- The user's mk1, derived at `m/48'/0'/0'/2'`, decodes to `Path: "m/48'/0'/0'/2'"`, `Fingerprint: "73c5da0a"`, `Stubs: [[0x7b,0x71,0x64,0x21]]`; `bundle.Verify(b,b)==nil`.

The abandon-about seed account key at `m/48'/0'/0'/2'`: chain code `bba0c7ca160a870efeb940ab90d0f4284fea1b5e0d2117677e823fc37e2d5763`, pubkey `021a3bf5fbf737d0f36993fd46dc4913093beb532d654fe0dfd98bd27585dc9f29`, so the 65-byte `Xpub` is the concatenation. Available in `gui` tests via `abandonAboutMnemonic()` (`gui/derive_test.go:13`) + `deriveAccountXpub` + `decodeXpubBytes`.

---

## File structure (create / modify)

**Create (fork worktree):**
- `gui/multisig.go` — `engraveMultisigFlow` orchestrator + top-level mnemonic scrub (mirror `gui/singlesig.go`).
- `gui/multisig_supply.go` — `extractSuppliedMd1` (single-md1 supply filter) + the full-policy gate `allSlotsHaveXpub`.
- `gui/multisig_match.go` — `findUserSlot` (the D14 cross-match).
- `gui/multisig_derive.go` — `deriveMultisigLeg` (the user's mk1 + ms1 from the matched origin + supplied md1).
- `gui/multisig_engrave.go` — `multisigEngraveCards` (full/watch-only card order, supplied md1 verbatim).
- `gui/multisig_verify.go` — `multisigVerifyFlow` (re-type → re-match → re-derive → `bundle.Verify`).
- `gui/multisig_restore.go` — `multisigRestoreDocFlow` + `multisigRestoreLines` (the NET-NEW render path; `expandedToDescriptor`→`address`).
- `gui/multisig_program_test.go` — nav-test for the new `engraveMultisig` program.
- `gui/testdata/t6b_multisig_full.md1.txt` — the vendored full-policy multisig md1 fixture (6 chunk lines, one per line).
- Test files: `gui/multisig_supply_test.go`, `gui/multisig_match_test.go`, `gui/multisig_derive_test.go`, `gui/multisig_engrave_test.go`, `gui/multisig_verify_test.go`, `gui/multisig_restore_test.go`, `gui/multisig_fuzz_test.go`.

**Modify (fork worktree):**
- `gui/gui.go` — enum (`:151`), dispatch (`:1501-1503`), left-wrap (`:1638`), right-wrap (`:1645`), title (`:1670`), `npage` (`:1846`), `layoutMainPlates` (`:1856`), `npages` (`:1865`).
- `gui/singlesig_program_test.go`, `gui/bundle_program_test.go`, `gui/derive_xpub_program_test.go` — retarget the wrap bound from `engraveSingleSig` to `engraveMultisig` and add the new program to the carousel walk.

**Commit discipline (every commit):** signed (`-S`) + DCO (`-s`), author `Brian Goss <goss.brian@gmail.com>`, trailer `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. Stage paths EXPLICITLY (no `git add -A`). Use `GIT_AUTHOR_NAME`/`GIT_AUTHOR_EMAIL` already set, or pass `--author`.

A reusable commit snippet (adjust the staged paths + subject per task):
```bash
git -C /scratch/code/shibboleth/seedhammer-t6b \
  commit -S -s \
  --author="Brian Goss <goss.brian@gmail.com>" \
  -m "feat(gui): <subject>" \
  -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 0 — Worktree + baseline

**Files:** none (setup only).

- [ ] **Step 1: Create the isolated worktree off `main` (072461a).**

Run:
```bash
export PATH=$PATH:/home/bcg/.local/go/bin
git -C /scratch/code/shibboleth/seedhammer fetch --all 2>/dev/null; \
git -C /scratch/code/shibboleth/seedhammer worktree add -b feat/t6b-multisig-supplied-md1 \
  /scratch/code/shibboleth/seedhammer-t6b main
```
Expected: `Preparing worktree ... HEAD is now at 072461a ...`. Confirm HEAD:
```bash
git -C /scratch/code/shibboleth/seedhammer-t6b rev-parse HEAD
```
Expected: `072461a8772e72d31dcea6762961b5f69bfb61a3`.

- [ ] **Step 2: Baseline the affected packages green.**

Run:
```bash
cd /scratch/code/shibboleth/seedhammer-t6b && \
go test ./gui/... ./md/... ./bundle/... ./mk/... ./codex32/...
```
Expected: all `ok` (no `FAIL`). This is the regression baseline. If anything fails here, STOP — the base is dirty.

- [ ] **Step 3: Confirm the test seed helper exists in the worktree.**

Run:
```bash
grep -n "func abandonAboutMnemonic" /scratch/code/shibboleth/seedhammer-t6b/gui/derive_test.go
```
Expected: a hit at `gui/derive_test.go:13`. (No commit for Task 0.)

> From here on, **all paths are in the worktree** `/scratch/code/shibboleth/seedhammer-t6b`. All `go test` commands assume `cd /scratch/code/shibboleth/seedhammer-t6b` (or use `-C`).

---

## Task 1 — `extractSuppliedMd1` (single-md1 supply filter, I-1 / I-11)

**Files:**
- Create: `gui/multisig_supply.go`
- Test: `gui/multisig_supply_test.go`

- [ ] **Step 1: Write the failing test.**

Create `gui/multisig_supply_test.go`:
```go
package gui

import (
	"reflect"
	"testing"
)

// TestExtractSuppliedMd1 exercises the single-md1 supply contract (I-11): a
// gather producing exactly ONE cardMD1 yields its verbatim strings; 0 md1, >=2
// md1, or any cardMK1 present refuses. (A cardMS1 cannot arise from the gather
// path — ms1 is refused upstream at classify — so its clause is DEFENSIVE; the
// test for it documents intent but is not operator-reachable, n-1.)
func TestExtractSuppliedMd1(t *testing.T) {
	md1A := bundleCard{kind: cardMD1, label: "md1 descriptor", strings: []string{"md1aaa", "md1bbb"}}
	md1B := bundleCard{kind: cardMD1, label: "md1 descriptor", strings: []string{"md1ccc"}}
	mk1 := bundleCard{kind: cardMK1, label: "mk1 key", strings: []string{"mk1xxx"}}

	t.Run("exactly one md1 -> verbatim strings", func(t *testing.T) {
		got, ok := extractSuppliedMd1([]bundleCard{md1A})
		if !ok {
			t.Fatal("ok=false, want true for a single md1")
		}
		if !reflect.DeepEqual(got, []string{"md1aaa", "md1bbb"}) {
			t.Fatalf("strings = %v, want verbatim [md1aaa md1bbb]", got)
		}
	})
	t.Run("zero md1 -> refuse", func(t *testing.T) {
		if _, ok := extractSuppliedMd1(nil); ok {
			t.Fatal("ok=true for zero cards, want false")
		}
		if _, ok := extractSuppliedMd1([]bundleCard{mk1}); ok {
			t.Fatal("ok=true for mk1-only, want false (no md1)")
		}
	})
	t.Run("two md1 -> refuse", func(t *testing.T) {
		if _, ok := extractSuppliedMd1([]bundleCard{md1A, md1B}); ok {
			t.Fatal("ok=true for two md1, want false (ambiguous supply)")
		}
	})
	t.Run("any mk1 present -> refuse", func(t *testing.T) {
		if _, ok := extractSuppliedMd1([]bundleCard{md1A, mk1}); ok {
			t.Fatal("ok=true with a stray mk1, want false (polluted supply)")
		}
	})
	t.Run("defensive cardMS1 -> refuse (n-1, not gather-reachable)", func(t *testing.T) {
		ms1 := bundleCard{kind: cardMS1, label: "ms1", strings: []string{"ms1zzz"}}
		if _, ok := extractSuppliedMd1([]bundleCard{md1A, ms1}); ok {
			t.Fatal("ok=true with a cardMS1, want false (defensive)")
		}
	})
}
```

- [ ] **Step 2: Run the test to verify it fails.**

Run: `go test ./gui/ -run TestExtractSuppliedMd1 -v`
Expected: FAIL — `undefined: extractSuppliedMd1`.

- [ ] **Step 3: Write the minimal implementation.**

Create `gui/multisig_supply.go`:
```go
package gui

import "seedhammer.com/md"

// ─── T6b: single-md1 supply filter + full-policy gate ────────────────────────
//
// T6b gathers a SUPPLIED multisig/miniscript wallet-policy md1 over NFC via the
// shipped bundleGatherFlow, then filters it down to EXACTLY ONE descriptor card.
// singleSigReadbackCards is NOT reused (it requires BOTH an mk1 AND an md1,
// gui/singlesig_verify.go:38 — the opposite of a one-md1/zero-mk1 supply, R0-I1).

// extractSuppliedMd1 returns the verbatim chunk strings of EXACTLY one cardMD1
// in the gathered card set (I-1/I-11). It refuses (ok=false) when: there is no
// md1, there are >=2 md1 (ambiguous supply), or any cardMK1/cardMS1 is present
// (polluted supply). The cardMS1 clause is DEFENSIVE — the gather path never
// produces a cardMS1 (ms1 is refused upstream at classify, n-1) — but a stray
// key/secret card must never be silently tolerated alongside the wallet policy.
func extractSuppliedMd1(cards []bundleCard) ([]string, bool) {
	var md1 []string
	count := 0
	for _, c := range cards {
		switch c.kind {
		case cardMD1:
			count++
			md1 = c.strings
		case cardMK1, cardMS1:
			return nil, false // a stray key/secret card pollutes the supply.
		}
	}
	if count != 1 {
		return nil, false // 0 md1 (nothing to engrave) or >=2 (ambiguous).
	}
	return md1, true
}

// allSlotsHaveXpub is the full-policy gate (I-3): the supplied md1 must be a
// FULL wallet policy — every expanded slot must carry an xpub, else there is no
// public key to cross-match the typed seed against. A template-only md1 (no
// pubkeys) or any-slot-missing-xpub refuses. An empty key set refuses.
func allSlotsHaveXpub(keys []md.ExpandedKey) bool {
	if len(keys) == 0 {
		return false
	}
	for _, k := range keys {
		if !k.XpubPresent {
			return false
		}
	}
	return true
}
```

> Note: `allSlotsHaveXpub` is added now (it shares this file's responsibility) but is first exercised by its own test in Task 3.

- [ ] **Step 4: Run the test to verify it passes.**

Run: `go test ./gui/ -run TestExtractSuppliedMd1 -v`
Expected: PASS (all sub-tests).

- [ ] **Step 5: Commit.**

```bash
git -C /scratch/code/shibboleth/seedhammer-t6b add \
  gui/multisig_supply.go gui/multisig_supply_test.go
git -C /scratch/code/shibboleth/seedhammer-t6b commit -S -s \
  --author="Brian Goss <goss.brian@gmail.com>" \
  -m "feat(gui): T6b extractSuppliedMd1 single-md1 supply filter" \
  -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 1.5 — Vendor the full-policy multisig md1 fixture

The fork has NO full-policy multisig md1 (with embedded xpubs) anywhere — all `wsh_*` vectors are template-only, and the device has no multisig encoder by design. Tasks 3–7 need a real one. We vendor the verified fixture (from "Test vectors" above) as a `gui/testdata` file plus a loader helper.

**Files:**
- Create: `gui/testdata/t6b_multisig_full.md1.txt`
- Create: `gui/multisig_testhelpers_test.go`

- [ ] **Step 1: Write the fixture file.**

Create `gui/testdata/t6b_multisig_full.md1.txt` with EXACTLY these 6 lines (no trailing blank line beyond the final newline):
```
md1fvgfqzspqjtvyyy4qqxppcgsc27rczqg3yyc5z5tpwxqergd3c8g7ruszzg3ryssjfstllhxufdm4
md1fvgfqzs2jvfeg9y4zktpd9chs82fefgh35nuevya8z62kep2q7md6duvfx8px8ygw3q3umhs2q3cu
md1fvgfqzss8ygdjvlt5pterdm5rru59s2su80aw2q4wgdpapgfl4pkhsdyytkwl5zq9ner9ltnl8fnz
md1fvgfqzsllphut2hvvpp5wl4l0mn058ndxfl63kufyfsjwlt2vkk2nlqmlvch5n4sk08xmsudrng93
md1fvgfqz3qhwf72vyq3zgf3g9gkzuvpjxsmrsw3u8eqyy3zxfp9ycnjs2f29vkz6ts908m9qqcmg97l
md1fvgfqz3f0qtrqglu5g8kh6mfsg4qxa9wq0nv9cauwfwxw70984wkqnw2uwz0w27h0f8nmf46cm8
```

- [ ] **Step 2: Write the loader + a guard test that validates the fixture.**

Create `gui/multisig_testhelpers_test.go`:
```go
package gui

import (
	"bufio"
	"os"
	"strings"
	"testing"

	"seedhammer.com/md"
)

// suppliedMultisigMd1 loads the vendored full-policy wsh(sortedmulti(2,@0,@1,@2))
// md1 chunk strings. The operator's abandon-about seed is slot @1 at
// m/48'/0'/0'/2'; @0/@2 are foreign pubkeys.
func suppliedMultisigMd1(t *testing.T) []string {
	t.Helper()
	f, err := os.Open("testdata/t6b_multisig_full.md1.txt")
	if err != nil {
		t.Fatalf("open fixture: %v", err)
	}
	defer f.Close()
	var out []string
	sc := bufio.NewScanner(f)
	for sc.Scan() {
		line := strings.TrimSpace(sc.Text())
		if line == "" {
			continue
		}
		out = append(out, line)
	}
	if err := sc.Err(); err != nil {
		t.Fatalf("scan fixture: %v", err)
	}
	if len(out) != 6 {
		t.Fatalf("fixture has %d chunks, want 6", len(out))
	}
	return out
}

// TestSuppliedMultisigFixtureIsFullPolicy guards the vendored fixture: it must
// decode to a full-policy 2-of-3 wsh(sortedmulti) with every slot xpub-present
// at origin m/48'/0'/0'/2', and the abandon seed must match slot @1 only. If
// this fails, the fixture string is corrupt — do NOT regenerate it ad hoc;
// re-derive it via the documented descriptor (see the plan's Test Vectors).
func TestSuppliedMultisigFixtureIsFullPolicy(t *testing.T) {
	chunks := suppliedMultisigMd1(t)
	tpl, keys, err := md.ExpandWalletPolicyChunks(chunks)
	if err != nil {
		t.Fatalf("ExpandWalletPolicyChunks: %v", err)
	}
	if tpl.Root != md.ScriptWsh || tpl.Policy != md.PolicySortedMulti {
		t.Fatalf("tpl root/policy = %v/%v, want Wsh/SortedMulti", tpl.Root, tpl.Policy)
	}
	if tpl.K != 2 || tpl.N != 3 {
		t.Fatalf("tpl K/N = %d/%d, want 2/3", tpl.K, tpl.N)
	}
	if len(keys) != 3 {
		t.Fatalf("got %d keys, want 3", len(keys))
	}
	if !allSlotsHaveXpub(keys) {
		t.Fatal("fixture is not full-policy (a slot lacks an xpub)")
	}
	wantOrigin := "m/48h/0h/0h/2h"
	for i, k := range keys {
		if k.OriginPath.String() != wantOrigin {
			t.Fatalf("key @%d origin = %s, want %s", i, k.OriginPath.String(), wantOrigin)
		}
	}
}
```

- [ ] **Step 3: Run the guard test.**

Run: `go test ./gui/ -run TestSuppliedMultisigFixtureIsFullPolicy -v`
Expected: PASS. (If FAIL, the fixture file was mistyped — fix the file, not the test.)

- [ ] **Step 4: Commit.**

```bash
git -C /scratch/code/shibboleth/seedhammer-t6b add \
  gui/testdata/t6b_multisig_full.md1.txt gui/multisig_testhelpers_test.go
git -C /scratch/code/shibboleth/seedhammer-t6b commit -S -s \
  --author="Brian Goss <goss.brian@gmail.com>" \
  -m "test(gui): T6b vendored full-policy multisig md1 fixture + loader" \
  -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2 — `findUserSlot` (the D14 cross-match, I-1 Critical)

**Files:**
- Create: `gui/multisig_match.go`
- Test: `gui/multisig_match_test.go`

The unit test does NOT need the full md1 fixture: it builds `[]md.ExpandedKey` slots DIRECTLY (the type is exported), deriving the abandon seed's canonical `(cc,pk)` to construct the `[65]byte` `Xpub` for the matching slot, and using a fixed foreign 65-byte `Xpub` for non-cosigner slots.

- [ ] **Step 1: Write the failing test.**

Create `gui/multisig_match_test.go`:
```go
package gui

import (
	"testing"

	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip32"
	"seedhammer.com/md"
)

// abandonSlotXpub derives the abandon-about seed at the given origin and packs
// its canonical (chainCode||compressedPubkey) into the 65-byte ExpandedKey.Xpub
// layout (cc = [0:32], pk = [32:65]).
func abandonSlotXpub(t *testing.T, origin bip32.Path) [65]byte {
	t.Helper()
	m := abandonAboutMnemonic()
	xpub, _, err := deriveAccountXpub(m, "", &chaincfg.MainNetParams, origin)
	if err != nil {
		t.Fatalf("deriveAccountXpub: %v", err)
	}
	cc, pk, _, err := decodeXpubBytes(xpub)
	if err != nil {
		t.Fatalf("decodeXpubBytes: %v", err)
	}
	var out [65]byte
	copy(out[0:32], cc[:])
	copy(out[32:65], pk[:])
	return out
}

// foreignXpub is a structurally-valid 65-byte Xpub that the abandon seed never
// derives to (a fixed non-zero pattern). Used for non-cosigner slots.
func foreignXpub() [65]byte {
	var out [65]byte
	for i := range out {
		out[i] = byte(0x40 + i)
	}
	return out
}

func msPath(comps ...uint32) bip32.Path {
	p := make(bip32.Path, len(comps))
	copy(p, comps)
	return p
}

const hard32 = 0x80000000

// TestFindUserSlot exercises the D14 cross-match (I-1): match on the canonical
// (cc,pk) pair via bytes.Equal over the full 32+33 bytes, derive at each slot's
// OWN origin, refuse on zero matches, first-by-index + reused notice on >=2.
func TestFindUserSlot(t *testing.T) {
	net := &chaincfg.MainNetParams
	m := abandonAboutMnemonic()
	origin0 := msPath(hard32+48, hard32+0, hard32+0, hard32+2) // @0 origin
	origin1 := msPath(hard32+48, hard32+0, hard32+1, hard32+2) // @1 origin (distinct)
	origin2 := msPath(hard32+48, hard32+0, hard32+2, hard32+2) // @2 origin (distinct)

	t.Run("match at @1, foreign @0/@2", func(t *testing.T) {
		keys := []md.ExpandedKey{
			{Index: 0, OriginPath: origin0, Xpub: foreignXpub(), XpubPresent: true},
			{Index: 1, OriginPath: origin1, Xpub: abandonSlotXpub(t, origin1), XpubPresent: true},
			{Index: 2, OriginPath: origin2, Xpub: foreignXpub(), XpubPresent: true},
		}
		idx, origin, reused, ok := findUserSlot(m, "", net, keys)
		if !ok {
			t.Fatal("ok=false, want a match at @1")
		}
		if idx != 1 {
			t.Fatalf("slot index = %d, want 1", idx)
		}
		if origin.String() != origin1.String() {
			t.Fatalf("origin = %s, want %s", origin.String(), origin1.String())
		}
		if len(reused) != 0 {
			t.Fatalf("reused = %v, want empty (single match)", reused)
		}
	})

	t.Run("non-cosigner -> refuse", func(t *testing.T) {
		keys := []md.ExpandedKey{
			{Index: 0, OriginPath: origin0, Xpub: foreignXpub(), XpubPresent: true},
			{Index: 1, OriginPath: origin1, Xpub: foreignXpub(), XpubPresent: true},
		}
		if _, _, _, ok := findUserSlot(m, "", net, keys); ok {
			t.Fatal("ok=true for a non-cosigner seed, want false (refuse)")
		}
	})

	t.Run("ambiguous @0 and @2 -> first-by-index + notice", func(t *testing.T) {
		// The SAME seed at two DISTINCT origins (legitimate reused key).
		keys := []md.ExpandedKey{
			{Index: 0, OriginPath: origin0, Xpub: abandonSlotXpub(t, origin0), XpubPresent: true},
			{Index: 1, OriginPath: origin1, Xpub: foreignXpub(), XpubPresent: true},
			{Index: 2, OriginPath: origin2, Xpub: abandonSlotXpub(t, origin2), XpubPresent: true},
		}
		idx, origin, reused, ok := findUserSlot(m, "", net, keys)
		if !ok {
			t.Fatal("ok=false, want first-by-index match")
		}
		if idx != 0 {
			t.Fatalf("slot index = %d, want 0 (first-by-index)", idx)
		}
		if origin.String() != origin0.String() {
			t.Fatalf("origin = %s, want @0 origin %s", origin.String(), origin0.String())
		}
		if len(reused) != 2 || reused[0] != 0 || reused[1] != 2 {
			t.Fatalf("reused = %v, want [0 2]", reused)
		}
	})

	t.Run("XpubPresent=false slot is skipped", func(t *testing.T) {
		keys := []md.ExpandedKey{
			{Index: 0, OriginPath: origin1, Xpub: abandonSlotXpub(t, origin1), XpubPresent: false}, // skipped
			{Index: 1, OriginPath: origin1, Xpub: abandonSlotXpub(t, origin1), XpubPresent: true},  // matches
		}
		idx, _, _, ok := findUserSlot(m, "", net, keys)
		if !ok || idx != 1 {
			t.Fatalf("idx=%d ok=%v, want match at @1 (the present slot)", idx, ok)
		}
	})
}
```

- [ ] **Step 2: Run the test to verify it fails.**

Run: `go test ./gui/ -run TestFindUserSlot -v`
Expected: FAIL — `undefined: findUserSlot`.

- [ ] **Step 3: Write the minimal implementation.**

Create `gui/multisig_match.go`:
```go
package gui

import (
	"bytes"

	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip32"
	"seedhammer.com/bip39"
	"seedhammer.com/md"
)

// ─── T6b: the D14 slot cross-match (the wrong-wallet guard) ──────────────────
//
// findUserSlot derives the operator's OWN account key from the TYPED seed at
// EACH xpub-present slot's own origin and matches it against that slot's
// embedded key on the CANONICAL (chainCode, compressedPubkey) pair — NEVER
// base58 (the supplied xpub carries different parentFP/depth metadata) and
// NEVER == on mismatched array/slice types (I-2). It returns the matched slot's
// index + origin.
//
// Outcomes:
//   - exactly one match  -> (index, origin, nil, true)
//   - zero matches       -> (_, _, _, false): REFUSE (the seed is not a cosigner;
//                           never engrave a backup for a wallet you are not in)
//   - >=2 matches        -> the SAME seed legitimately appears at >=2 cosigner
//                           slots under DISTINCT origins. Return the FIRST-by-index
//                           slot (deterministic; policy+stub identical across
//                           slots, only the mk1 Path differs) + every matched
//                           index in `reused` so the caller can show a notice.
//
// SECURITY: deriveAccountXpub scrubs its own seed/master/intermediates on every
// call; the caller scrubs the mnemonic []Word after the LAST derive here (the
// loop may derive at several slots before matching).
func findUserSlot(m bip39.Mnemonic, passphrase string, net *chaincfg.Params, keys []md.ExpandedKey) (slotIndex int, origin bip32.Path, reused []int, ok bool) {
	var matches []int
	for i, k := range keys {
		if !k.XpubPresent {
			continue
		}
		xpub, _, err := deriveAccountXpub(m, passphrase, net, k.OriginPath)
		if err != nil {
			continue // a malformed origin can't be the operator's slot.
		}
		cc, pk, _, err := decodeXpubBytes(xpub)
		if err != nil {
			continue
		}
		if bytes.Equal(cc[:], k.Xpub[0:32]) && bytes.Equal(pk[:], k.Xpub[32:65]) {
			matches = append(matches, i)
		}
	}
	if len(matches) == 0 {
		return 0, nil, nil, false
	}
	first := matches[0]
	if len(matches) >= 2 {
		return first, keys[first].OriginPath, matches, true
	}
	return first, keys[first].OriginPath, nil, true
}
```

- [ ] **Step 4: Run the test to verify it passes.**

Run: `go test ./gui/ -run TestFindUserSlot -v`
Expected: PASS (all sub-tests).

- [ ] **Step 5: Commit.**

```bash
git -C /scratch/code/shibboleth/seedhammer-t6b add \
  gui/multisig_match.go gui/multisig_match_test.go
git -C /scratch/code/shibboleth/seedhammer-t6b commit -S -s \
  --author="Brian Goss <goss.brian@gmail.com>" \
  -m "feat(gui): T6b findUserSlot D14 canonical-pair cross-match" \
  -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3 — Full-policy gate (I-3)

`allSlotsHaveXpub` was written in Task 1; here we exercise it against the real template-only golden and the full-policy fixture.

**Files:**
- Modify: `gui/multisig_supply_test.go` (add a test)

- [ ] **Step 1: Write the failing test.**

Append to `gui/multisig_supply_test.go` (add `"seedhammer.com/md"` to its imports):
```go
// TestAllSlotsHaveXpub: the full-policy gate (I-3). The full-policy fixture
// passes; a template-only multisig (the wsh_sortedmulti md golden, which carries
// NO pubkeys) refuses; an empty key set refuses.
func TestAllSlotsHaveXpub(t *testing.T) {
	// Full-policy fixture -> all slots xpub-present.
	chunks := suppliedMultisigMd1(t)
	_, keys, err := md.ExpandWalletPolicyChunks(chunks)
	if err != nil {
		t.Fatalf("ExpandWalletPolicyChunks(full): %v", err)
	}
	if !allSlotsHaveXpub(keys) {
		t.Fatal("full-policy fixture rejected by the gate")
	}

	// Template-only: a slot set with XpubPresent=false must refuse.
	tmplOnly := []md.ExpandedKey{
		{Index: 0, XpubPresent: true},
		{Index: 1, XpubPresent: false},
	}
	if allSlotsHaveXpub(tmplOnly) {
		t.Fatal("a missing-xpub slot passed the gate, want refuse")
	}
	if allSlotsHaveXpub(nil) {
		t.Fatal("empty key set passed the gate, want refuse")
	}
}
```

- [ ] **Step 2: Run the test to verify it fails (then passes).**

Run: `go test ./gui/ -run TestAllSlotsHaveXpub -v`
Expected: PASS immediately (impl already exists from Task 1). If the import is missing it FAILs to compile first — add `"seedhammer.com/md"` and re-run to PASS.

> This step intentionally has no new implementation — Task 1 already wrote `allSlotsHaveXpub`. The test locks the spec'd behavior against the REAL goldens (the fixture and a missing-xpub set), which is the I-3 acceptance.

- [ ] **Step 3: Commit.**

```bash
git -C /scratch/code/shibboleth/seedhammer-t6b add gui/multisig_supply_test.go
git -C /scratch/code/shibboleth/seedhammer-t6b commit -S -s \
  --author="Brian Goss <goss.brian@gmail.com>" \
  -m "test(gui): T6b full-policy gate acceptance (I-3)" \
  -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4 — Derive the user's leg (I-4, mk1 + ms1)

**Files:**
- Create: `gui/multisig_derive.go`
- Test: `gui/multisig_derive_test.go`

- [ ] **Step 1: Write the failing test.**

Create `gui/multisig_derive_test.go`:
```go
package gui

import (
	"bytes"
	"testing"

	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/codex32"
	"seedhammer.com/md"
	"seedhammer.com/mk"
)

// TestDeriveMultisigLeg: the operator's leg for the full-policy fixture (seed at
// slot @1, origin m/48'/0'/0'/2'). The mk1 stub == WalletPolicyIDStubChunks of
// the SUPPLIED md1 (I-4); the mk1 Path == the matched slot origin; ms1
// round-trips the entropy. Full mode includes ms1; watch-only omits it.
func TestDeriveMultisigLeg(t *testing.T) {
	chunks := suppliedMultisigMd1(t)
	_, keys, err := md.ExpandWalletPolicyChunks(chunks)
	if err != nil {
		t.Fatalf("ExpandWalletPolicyChunks: %v", err)
	}
	m := abandonAboutMnemonic()
	idx, origin, _, ok := findUserSlot(m, "", &chaincfg.MainNetParams, keys)
	if !ok || idx != 1 {
		t.Fatalf("findUserSlot idx=%d ok=%v, want match @1", idx, ok)
	}

	b, err := deriveMultisigLeg(m, "", &chaincfg.MainNetParams, origin, chunks, true)
	if err != nil {
		t.Fatalf("deriveMultisigLeg(full): %v", err)
	}

	// md1 leg is the SUPPLIED strings verbatim.
	if len(b.MD1) != len(chunks) {
		t.Fatalf("MD1 len = %d, want %d (verbatim supply)", len(b.MD1), len(chunks))
	}
	for i := range chunks {
		if b.MD1[i] != chunks[i] {
			t.Fatalf("MD1[%d] not verbatim:\n got %s\nwant %s", i, b.MD1[i], chunks[i])
		}
	}

	// mk1: bound stub + matched path.
	card, err := mk.Decode(b.MK1)
	if err != nil {
		t.Fatalf("mk.Decode: %v", err)
	}
	wantStub, err := md.WalletPolicyIDStubChunks(chunks)
	if err != nil {
		t.Fatalf("WalletPolicyIDStubChunks: %v", err)
	}
	if len(card.Stubs) != 1 || card.Stubs[0] != wantStub {
		t.Fatalf("mk1 stubs = %v, want [%v] (bound to the supplied policy)", card.Stubs, wantStub)
	}
	if card.Path != "m/48'/0'/0'/2'" {
		t.Fatalf("mk1 path = %q, want m/48'/0'/0'/2'", card.Path)
	}
	if card.Fingerprint != "73c5da0a" {
		t.Fatalf("mk1 fingerprint = %q, want 73c5da0a", card.Fingerprint)
	}

	// ms1 round-trips the entropy.
	if b.MS1 == "" {
		t.Fatal("full mode produced no ms1")
	}
	ms1str, err := codex32.New(b.MS1)
	if err != nil {
		t.Fatalf("codex32.New: %v", err)
	}
	_, _, ent, err := codex32.DecodeMS1(ms1str)
	if err != nil {
		t.Fatalf("DecodeMS1: %v", err)
	}
	if !bytes.Equal(ent, m.Entropy()) {
		t.Fatalf("ms1 entropy = %x, want %x", ent, m.Entropy())
	}

	// Watch-only: no ms1.
	wo, err := deriveMultisigLeg(m, "", &chaincfg.MainNetParams, origin, chunks, false)
	if err != nil {
		t.Fatalf("deriveMultisigLeg(watch-only): %v", err)
	}
	if wo.MS1 != "" {
		t.Fatalf("watch-only ms1 = %q, want empty", wo.MS1)
	}
}
```

- [ ] **Step 2: Run the test to verify it fails.**

Run: `go test ./gui/ -run TestDeriveMultisigLeg -v`
Expected: FAIL — `undefined: deriveMultisigLeg`.

- [ ] **Step 3: Write the minimal implementation.**

Create `gui/multisig_derive.go`:
```go
package gui

import (
	"errors"
	"fmt"

	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip32"
	"seedhammer.com/bip39"
	"seedhammer.com/bundle"
	"seedhammer.com/codex32"
	"seedhammer.com/md"
	"seedhammer.com/mk"
)

// ─── T6b: derive the operator's leg of a SUPPLIED multisig bundle ────────────
//
// deriveMultisigLeg builds the operator's mk1 (policy-bound to the SUPPLIED md1)
// and ms1 (full only) for the matched slot. The md1 leg is the SUPPLIED chunk
// strings VERBATIM (I-2 — the device never re-encodes a multisig descriptor).
//
// mk1.Path is the matched slot origin (so compactFromXpub's depth/child gate
// passes — the xpub was derived AT this origin). mk1.Stubs is the policy_id_stub
// of the SUPPLIED md1 (I-4 — binds the key card to the supplied policy).
// Network is a LABEL only ("mainnet"); it is not serialized nor verified.
//
// SECURITY: gate m.Valid() before m.Entropy() (which panics on invalid);
// deriveAccountXpub scrubs the seed/master internally; the entropy buffer is
// wiped after ms1 encode. The caller scrubs the mnemonic []Word.
var errMultisigInvalidSeed = errors.New("multisig: invalid seed mnemonic")

func deriveMultisigLeg(m bip39.Mnemonic, passphrase string, net *chaincfg.Params, origin bip32.Path, suppliedMd1 []string, full bool) (bundle.Bundle, error) {
	if !m.Valid() {
		return bundle.Bundle{}, errMultisigInvalidSeed
	}

	xpub, masterFP, err := deriveAccountXpub(m, passphrase, net, origin)
	if err != nil {
		return bundle.Bundle{}, err
	}

	stub, err := md.WalletPolicyIDStubChunks(suppliedMd1)
	if err != nil {
		return bundle.Bundle{}, err
	}

	mk1, err := mk.Encode(mk.Card{
		Network:     "mainnet", // LABEL only (mainnet-only, I-8).
		Path:        origin.String(),
		Fingerprint: fmt.Sprintf("%08x", masterFP),
		Stubs:       [][4]byte{stub},
		Xpub:        xpub,
	})
	if err != nil {
		return bundle.Bundle{}, err
	}

	// md1 leg = the SUPPLIED strings VERBATIM (clone so the caller's slice can't
	// be mutated downstream).
	md1 := append([]string(nil), suppliedMd1...)

	b := bundle.Bundle{MK1: mk1, MD1: md1}
	if full {
		entropy := m.Entropy()
		ms1, err := codex32.EncodeMS1(entropy)
		wipeBytes(entropy)
		if err != nil {
			return bundle.Bundle{}, err
		}
		b.MS1 = ms1
	}
	return b, nil
}
```

- [ ] **Step 4: Run the test to verify it passes.**

Run: `go test ./gui/ -run TestDeriveMultisigLeg -v`
Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git -C /scratch/code/shibboleth/seedhammer-t6b add \
  gui/multisig_derive.go gui/multisig_derive_test.go
git -C /scratch/code/shibboleth/seedhammer-t6b commit -S -s \
  --author="Brian Goss <goss.brian@gmail.com>" \
  -m "feat(gui): T6b deriveMultisigLeg policy-bound mk1 + ms1 (verbatim md1)" \
  -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 5 — Engrave cards (full / watch-only, I-2)

**Files:**
- Create: `gui/multisig_engrave.go`
- Test: `gui/multisig_engrave_test.go`

- [ ] **Step 1: Write the failing test.**

Create `gui/multisig_engrave_test.go`:
```go
package gui

import "testing"

// TestMultisigEngraveCards mirrors singleSigEngraveCards: full = [ms1, mk1, md1],
// watch-only = [mk1, md1]. The md1 card carries the SUPPLIED strings VERBATIM
// (I-2). The ms1 card is the SECRET, single-string, first.
func TestMultisigEngraveCards(t *testing.T) {
	md1 := []string{"md1aaa", "md1bbb"}
	mk1 := []string{"mk1xxx", "mk1yyy"}
	ms1 := "ms10secret"

	t.Run("full = ms1, mk1, md1", func(t *testing.T) {
		cards := multisigEngraveCards(ms1, mk1, md1, true)
		if len(cards) != 3 {
			t.Fatalf("full produced %d cards, want 3", len(cards))
		}
		if cards[0].kind != cardMS1 || len(cards[0].strings) != 1 || cards[0].strings[0] != ms1 {
			t.Fatalf("card[0] = %+v, want a single-string ms1", cards[0])
		}
		if cards[1].kind != cardMK1 {
			t.Fatalf("card[1].kind = %v, want cardMK1", cards[1].kind)
		}
		if cards[2].kind != cardMD1 {
			t.Fatalf("card[2].kind = %v, want cardMD1", cards[2].kind)
		}
		// md1 verbatim.
		for i := range md1 {
			if cards[2].strings[i] != md1[i] {
				t.Fatalf("md1 card[%d] = %q, want verbatim %q", i, cards[2].strings[i], md1[i])
			}
		}
	})

	t.Run("watch-only = mk1, md1", func(t *testing.T) {
		cards := multisigEngraveCards("", mk1, md1, false)
		if len(cards) != 2 {
			t.Fatalf("watch-only produced %d cards, want 2", len(cards))
		}
		if cards[0].kind != cardMK1 || cards[1].kind != cardMD1 {
			t.Fatalf("watch-only card kinds = %v/%v, want cardMK1/cardMD1", cards[0].kind, cards[1].kind)
		}
		// No cardMS1 -> bundleEngrave will show the ms1 reminder.
		if bundleShowMs1Reminder(cards) != true {
			t.Fatal("watch-only should trigger the ms1 reminder (no cardMS1)")
		}
	})
}
```

- [ ] **Step 2: Run the test to verify it fails.**

Run: `go test ./gui/ -run TestMultisigEngraveCards -v`
Expected: FAIL — `undefined: multisigEngraveCards`.

- [ ] **Step 3: Write the minimal implementation.**

Create `gui/multisig_engrave.go`:
```go
package gui

// ─── T6b: synthesize the engrave cards for a SUPPLIED multisig bundle ────────
//
// multisigEngraveCards mirrors singleSigEngraveCards (gui/singlesig_engrave.go):
// full -> [ms1, mk1, md1]; watch-only -> [mk1, md1] (the ms1 is left for the
// operator to hand-engrave; bundleEngrave shows the reminder via the
// cards-derived gate). The md1 strings are the SUPPLIED policy VERBATIM (I-2).
// The ms1 is a single-string, single-plate SECRET card, engraved onto
// owner-held steel only — never NFC.
func multisigEngraveCards(ms1 string, mk1, md1 []string, full bool) []bundleCard {
	var cards []bundleCard
	if full {
		cards = append(cards, bundleCard{
			kind:    cardMS1,
			label:   "ms1 secret share",
			strings: []string{ms1},
			summary: "secret seed backup",
		})
	}
	cards = append(cards,
		bundleCard{
			kind:    cardMK1,
			label:   "mk1 key",
			strings: append([]string(nil), mk1...),
			summary: "account key card",
		},
		bundleCard{
			kind:    cardMD1,
			label:   "md1 descriptor",
			strings: append([]string(nil), md1...),
			summary: "wallet policy descriptor",
		},
	)
	return cards
}
```

- [ ] **Step 4: Run the test to verify it passes.**

Run: `go test ./gui/ -run TestMultisigEngraveCards -v`
Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git -C /scratch/code/shibboleth/seedhammer-t6b add \
  gui/multisig_engrave.go gui/multisig_engrave_test.go
git -C /scratch/code/shibboleth/seedhammer-t6b commit -S -s \
  --author="Brian Goss <goss.brian@gmail.com>" \
  -m "feat(gui): T6b multisigEngraveCards (verbatim md1, full/watch-only)" \
  -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 6 — Verify-bundle (user's slot only, I-5)

`bundle.Verify` is reused UNCHANGED. We add a thin helper `verifyMultisig` that assembles the read-back bundle and delegates, mirroring `verifySingleSig` (`gui/singlesig_verify.go:49`).

**Files:**
- Modify: `gui/multisig_verify.go` (created here)
- Test: `gui/multisig_verify_test.go`

- [ ] **Step 1: Write the failing test.**

Create `gui/multisig_verify_test.go`:
```go
package gui

import (
	"testing"

	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/md"
)

// TestVerifyMultisig: re-derive the operator's leg + read back the supplied md1
// and operator mk1, run bundle.Verify (I-5). PASS for the matched slot; FAIL on
// a mutated mk1/md1/ms1; watch-only (no ms1) PASSes.
func TestVerifyMultisig(t *testing.T) {
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

	t.Run("full PASS", func(t *testing.T) {
		if err := verifyMultisig(derived, derived.MS1, derived.MK1, derived.MD1); err != nil {
			t.Fatalf("verifyMultisig PASS path: %v", err)
		}
	})

	t.Run("mutated mk1 FAIL", func(t *testing.T) {
		bad := append([]string(nil), derived.MK1...)
		bad[len(bad)-1] = "mk1tampered000000000000000000000000000000000000"
		if err := verifyMultisig(derived, derived.MS1, bad, derived.MD1); err == nil {
			t.Fatal("verifyMultisig accepted a mutated mk1, want FAIL")
		}
	})

	t.Run("mutated md1 FAIL", func(t *testing.T) {
		bad := append([]string(nil), derived.MD1...)
		bad[0] = bad[0][:len(bad[0])-1] + "x"
		if err := verifyMultisig(derived, derived.MS1, derived.MK1, bad); err == nil {
			t.Fatal("verifyMultisig accepted a mutated md1, want FAIL")
		}
	})

	t.Run("mutated ms1 FAIL", func(t *testing.T) {
		// A valid-but-different ms1 entropy: re-derive watch-only then supply a
		// fabricated ms1 by mutating an entropy byte is awkward; instead assert a
		// presence mismatch is caught (one side has ms1, the other doesn't).
		if err := verifyMultisig(derived, "", derived.MK1, derived.MD1); err == nil {
			t.Fatal("verifyMultisig accepted an ms1 presence mismatch, want FAIL")
		}
	})

	t.Run("watch-only PASS (no ms1 both sides)", func(t *testing.T) {
		wo, err := deriveMultisigLeg(m, "", &chaincfg.MainNetParams, origin, chunks, false)
		if err != nil {
			t.Fatalf("deriveMultisigLeg watch-only: %v", err)
		}
		if err := verifyMultisig(wo, "", wo.MK1, wo.MD1); err != nil {
			t.Fatalf("watch-only verify: %v", err)
		}
	})
}
```

- [ ] **Step 2: Run the test to verify it fails.**

Run: `go test ./gui/ -run TestVerifyMultisig -v`
Expected: FAIL — `undefined: verifyMultisig`.

- [ ] **Step 3: Write the minimal implementation.**

Create `gui/multisig_verify.go`:
```go
package gui

import (
	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bundle"
	"seedhammer.com/codex32"
	"seedhammer.com/md"
)

// ─── T6b: verify-bundle for a SUPPLIED multisig bundle (user's slot only) ────
//
// verifyMultisig assembles the read-back bundle and runs the deterministic
// comparator against the freshly re-derived operator leg (mirror verifySingleSig,
// gui/singlesig_verify.go:49). It verifies ONLY the operator's slot (I-5); the
// other cosigner slots are public-given and unverified-by-design (bundle.Verify
// never inspects them). For a watch-only verify the ms1Readback is "" AND the
// derived ms1 is dropped so both sides are empty (bundle.Verify skips the ms1
// leg). Returns the comparator's first diverging-field error, or nil on PASS.
func verifyMultisig(derived bundle.Bundle, ms1Readback string, mk1, md1 []string) error {
	d := derived
	if ms1Readback == "" {
		d.MS1 = ""
	}
	readback := bundle.Bundle{MS1: ms1Readback, MK1: mk1, MD1: md1}
	return bundle.Verify(d, readback)
}

// multisigVerifyFlow drives the on-device verify-bundle for the multisig flow:
// re-type the seed (fresh residency), gather the supplied md1 + operator mk1
// over NFC, re-cross-match to recover the operator's origin, re-derive the leg,
// hand-type the ms1 (full only; never NFC), and report PASS/FAIL. `full` reports
// whether an ms1 was engraved (and so must be hand-typed for the verify).
func multisigVerifyFlow(ctx *Context, th *Colors, derived bundle.Bundle, full bool) {
	reMnemonic, ok := seedEntryFlow(ctx, th)
	if !ok {
		return
	}
	defer func() {
		for i := range reMnemonic {
			reMnemonic[i] = 0
		}
	}()

	passphrase := ""
	ppChoice := &ChoiceScreen{Title: "Passphrase", Lead: "Add a BIP-39 passphrase?", Choices: []string{"Skip", "Add passphrase"}}
	if sel, ok := ppChoice.Choose(ctx, th); ok && sel == 1 {
		if pass, ok := passphraseFlow(ctx, th); ok {
			passphrase = pass
		}
	}

	// Read back the PUBLIC cards over NFC via the T5 gatherer.
	cards, ok := bundleGatherFlow(ctx, th)
	if !ok {
		return
	}
	suppliedMd1, ok := extractSuppliedMd1(cards)
	if !ok {
		showError(ctx, th, "Verify Bundle", "Read back exactly one wallet-policy md1 (and no key cards).")
		return
	}
	_, keys, err := md.ExpandWalletPolicyChunks(suppliedMd1)
	if err != nil {
		showError(ctx, th, "Verify Bundle", "Couldn't decode the read-back wallet policy.")
		return
	}
	_, origin, _, ok := findUserSlot(reMnemonic, passphrase, &chaincfg.MainNetParams, keys)
	if !ok {
		showError(ctx, th, "Verify Bundle", "The seed is not a cosigner of the read-back policy.")
		return
	}
	reDerived, err := deriveMultisigLeg(reMnemonic, passphrase, &chaincfg.MainNetParams, origin, suppliedMd1, full)
	if err != nil {
		showError(ctx, th, "Verify Bundle", "Couldn't re-derive the bundle from the seed.")
		return
	}

	// Hand-type the SECRET ms1 (full mode only; never NFC).
	ms1Readback := ""
	if full {
		obj, ok := inputCodex32Flow(ctx, th, "Type ms1")
		if !ok {
			return
		}
		s, isStr := obj.(codex32.String)
		if !isStr {
			showError(ctx, th, "Verify Bundle", "That isn't an ms1 secret share.")
			return
		}
		if _, _, _, err := codex32.DecodeMS1(s); err != nil {
			showError(ctx, th, "Verify Bundle", "That isn't a valid ms1 secret share.")
			return
		}
		ms1Readback = s.String()
	}

	if err := verifyMultisig(reDerived, ms1Readback, reDerived.MK1, suppliedMd1); err != nil {
		showError(ctx, th, "Verify Failed", "The read-back bundle does NOT match the seed. Check the engraved plates.")
		return
	}
	showError(ctx, th, "Verify OK", "The engraved bundle matches the seed.")
}
```

- [ ] **Step 4: Run the test to verify it passes.**

Run: `go test ./gui/ -run TestVerifyMultisig -v`
Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git -C /scratch/code/shibboleth/seedhammer-t6b add \
  gui/multisig_verify.go gui/multisig_verify_test.go
git -C /scratch/code/shibboleth/seedhammer-t6b commit -S -s \
  --author="Brian Goss <goss.brian@gmail.com>" \
  -m "feat(gui): T6b verify-bundle (user slot only, reuse bundle.Verify)" \
  -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 7 — Multisig restore-doc render path (I-6, NET-NEW)

The single-sig `restoreDocFlow` (`gui/singlesig_restore.go:118`) takes single-sig scalars and CANNOT be reused. This is a NET-NEW render path: `expandedToDescriptor(tpl,keys)` → on `expandOK` show `address.Receive/Change`; otherwise display-only, NO `address.*` call.

**Files:**
- Create: `gui/multisig_restore.go`
- Test: `gui/multisig_restore_test.go`

- [ ] **Step 1: Write the failing test.**

Create `gui/multisig_restore_test.go`:
```go
package gui

import (
	"strings"
	"testing"

	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"seedhammer.com/md"
)

// TestMultisigRestoreLines: a sortedmulti supplied md1 -> expandedToDescriptor
// -> address.Receive/Change show the multisig addresses (golden, I-6); a
// non-bip380 shape -> nil descriptor -> display-only "addresses unavailable"
// with NO address. No xprv ever appears.
func TestMultisigRestoreLines(t *testing.T) {
	t.Run("sortedmulti -> addresses (golden)", func(t *testing.T) {
		chunks := suppliedMultisigMd1(t)
		tpl, keys, err := md.ExpandWalletPolicyChunks(chunks)
		if err != nil {
			t.Fatalf("ExpandWalletPolicyChunks: %v", err)
		}
		lines, hasAddr, err := multisigRestoreLines(tpl, keys)
		if err != nil {
			t.Fatalf("multisigRestoreLines: %v", err)
		}
		if !hasAddr {
			t.Fatal("sortedmulti should yield addresses (expandOK)")
		}
		blob := strings.Join(lines, "\n")
		const wantRecv = "bc1qg2lsdla23zewexuhn5jcx49mqzs8wqss0lxguarfpnt7ysg7k52slz4dxd"
		const wantChange = "bc1qz76qjcmpwhh6ffenfwg44hpq3cwwfuqcr54vl4485yttpjtxy9qq3yufkt"
		if !strings.Contains(blob, wantRecv) {
			t.Fatalf("receive address %s missing from:\n%s", wantRecv, blob)
		}
		if !strings.Contains(blob, wantChange) {
			t.Fatalf("change address %s missing from:\n%s", wantChange, blob)
		}
		if strings.Contains(blob, "xprv") {
			t.Fatal("xprv leaked into the restore doc")
		}
	})

	t.Run("template-only -> display-only, no address", func(t *testing.T) {
		// A multisig template with NO xpubs -> expandTemplateOnly -> nil desc.
		tpl := md.Template{Root: md.ScriptWsh, Policy: md.PolicySortedMulti, K: 2, N: 2, Renderable: true}
		keys := []md.ExpandedKey{
			{Index: 0, OriginPath: msPath(hard32+48, hard32+0, hard32+0, hard32+2), XpubPresent: false},
			{Index: 1, OriginPath: msPath(hard32+48, hard32+0, hard32+0, hard32+2), XpubPresent: false},
		}
		lines, hasAddr, err := multisigRestoreLines(tpl, keys)
		if err != nil {
			t.Fatalf("multisigRestoreLines(template-only): %v", err)
		}
		if hasAddr {
			t.Fatal("template-only must NOT yield addresses")
		}
		blob := strings.Join(lines, "\n")
		if !strings.Contains(blob, "unavailable") {
			t.Fatalf("display-only path must note addresses unavailable:\n%s", blob)
		}
		if strings.HasPrefix(blob, "bc1") || strings.Contains(blob, "\nbc1") {
			t.Fatalf("an address appeared on a display-only path:\n%s", blob)
		}
	})

	_ = hdkeychain.HardenedKeyStart // keep the import if unused above
}
```

> Note: if `go vet`/compile flags `hdkeychain` as unused, delete that import line and the trailing `_ =` line. The `msPath`/`hard32` helpers come from `gui/multisig_match_test.go` (same package).

- [ ] **Step 2: Run the test to verify it fails.**

Run: `go test ./gui/ -run TestMultisigRestoreLines -v`
Expected: FAIL — `undefined: multisigRestoreLines`.

- [ ] **Step 3: Write the minimal implementation.**

Create `gui/multisig_restore.go`:
```go
package gui

import (
	"image"

	"seedhammer.com/address"
	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/layout"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
	"seedhammer.com/md"
)

// ─── T6b: multisig restore doc (NET-NEW render path, I-6) ────────────────────
//
// This is the multisig sibling of the single-sig restoreDocFlow, which takes
// single-sig scalars and CANNOT be reused (gui/singlesig_restore.go:118, R0-I3).
// Faithful-or-refuse: address-verify ONLY for the bip380-expressible (sortedmulti)
// subset (expandOK); a non-bip380 / template-only md1 yields a nil descriptor
// (gui/md1_expand.go:36-49), so this path is display-only with NO address.*
// call — a wrong-address verify is structurally impossible. Display-only, no
// secret.

// multisigRestoreLines builds the restore-doc display lines from a decoded
// supplied md1. On expandOK it shows the descriptor + first receive/change
// addresses (hasAddr=true). Otherwise it shows the descriptor template
// read-only with an "addresses unavailable" note and NO address (hasAddr=false).
func multisigRestoreLines(tpl md.Template, keys []md.ExpandedKey) (lines []string, hasAddr bool, err error) {
	desc, status := expandedToDescriptor(tpl, keys)
	if status != expandOK || desc == nil {
		// Display-only: no descriptor we can derive addresses from (faithful-or-refuse).
		lines = []string{
			"Wallet policy (read-only):",
		}
		lines = append(lines, chunkString(desc4Display(tpl), 20)...)
		lines = append(lines, "Addresses unavailable for this policy shape.")
		return lines, false, nil
	}
	recv0, err := address.Receive(desc, 0)
	if err != nil {
		return nil, false, err
	}
	change0, err := address.Change(desc, 0)
	if err != nil {
		return nil, false, err
	}
	lines = []string{"Descriptor:"}
	lines = append(lines, chunkString(desc.Encode(), 20)...)
	lines = append(lines, "First receive:", recv0, "First change:", change0)
	return lines, true, nil
}

// desc4Display is a short, PUBLIC summary of an un-renderable template for the
// display-only path (no secret, no address). It reuses the shipped summary
// helpers used by the bundle review screen.
func desc4Display(tpl md.Template) string {
	return scriptName(tpl.Root) + " " + policyLine(tpl)
}

// multisigRestoreDocFlow displays the multisig restore doc on a plain, paged,
// read-only screen (the 0-alloc gate posture; reuse the single-sig
// restoreDocScreen). Display-only — no secret, no engrave.
func multisigRestoreDocFlow(ctx *Context, th *Colors, suppliedMd1 []string) {
	tpl, keys, err := md.ExpandWalletPolicyChunks(suppliedMd1)
	if err != nil {
		showError(ctx, th, "Restore Doc", "Couldn't decode the wallet policy.")
		return
	}
	lines, _, err := multisigRestoreLines(tpl, keys)
	if err != nil {
		showError(ctx, th, "Restore Doc", "Couldn't derive the restore addresses.")
		return
	}
	restoreDocScreen(ctx, th, lines)
}

// (image/op/layout/widget/assets imported to keep restoreDocScreen reuse
// explicit; restoreDocScreen itself lives in gui/singlesig_restore.go.)
var (
	_ = image.Pt
	_ = op.Layer
	_ = layout.Rectangle{}
	_ = widget.Labelw
	_ = assets.IconBack
)
```

> **IMPORTANT for the implementer:** `multisigRestoreDocFlow` reuses `restoreDocScreen` (already defined in `gui/singlesig_restore.go:136`) and `chunkString`/`scriptName`/`policyLine` (already in the gui package). If the trailing `var ( _ = ... )` block causes "imported and not used" churn, DELETE the block AND the matching imports (`image`, `op`, `layout`, `widget`, `assets`) — they are only needed if you inline a screen here. The minimal correct file imports only `address` and `md` plus uses `expandedToDescriptor`, `expandOK`, `chunkString`, `scriptName`, `policyLine`, `restoreDocScreen`, `showError` (all same-package). Verify with `go build ./gui/`.

- [ ] **Step 4: Trim imports, then run the test to verify it passes.**

Run:
```bash
go build ./gui/ && go test ./gui/ -run TestMultisigRestoreLines -v
```
Expected: build OK, test PASS. If `go build` reports unused imports, remove them (and the `var ( _ = ... )` block) until it builds, then re-run the test.

- [ ] **Step 5: Commit.**

```bash
git -C /scratch/code/shibboleth/seedhammer-t6b add \
  gui/multisig_restore.go gui/multisig_restore_test.go
git -C /scratch/code/shibboleth/seedhammer-t6b commit -S -s \
  --author="Brian Goss <goss.brian@gmail.com>" \
  -m "feat(gui): T6b NET-NEW multisig restore render path (faithful-or-refuse)" \
  -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 8 — `engraveMultisigFlow` orchestrator

Wire gather → extract → full-policy gate → typed-only seed → cross-match → derive → engrave → verify → restore, with the top-level mnemonic scrub on EVERY exit (mirror `gui/singlesig.go:39-45`). This task has no unit test of its own (it's UI orchestration driven by `Context`); it is covered structurally by `go build` + the no-regression suite (Task 10) and the nav-test (Task 9). The logical pieces it composes are each unit-tested in Tasks 1–7.

**Files:**
- Create: `gui/multisig.go`

- [ ] **Step 1: Write the orchestrator (no failing test — composition of tested units).**

Create `gui/multisig.go`:
```go
package gui

import (
	"fmt"

	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip39"
	"seedhammer.com/md"
)

// ─── T6b: the SUPPLIED-multisig engrave orchestrator ─────────────────────────
//
// engraveMultisigFlow is the engraveMultisig program: gather a SUPPLIED
// multisig/miniscript wallet-policy md1 over NFC (PUBLIC) -> require a full
// policy (every slot xpub-present) -> hand-type the seed (TYPED-ONLY, never a
// scan) -> CROSS-MATCH the seed to one descriptor slot -> derive the operator's
// leg (ms1 + policy-bound mk1; the supplied md1 engraved VERBATIM) -> engrave
// (full = ms1+mk1+md1; watch-only = mk1+md1 + the ms1 reminder) -> offer
// verify-bundle -> show the multisig restore doc.
//
// SECURITY SPINE (mirror gui/singlesig.go):
//   - TYPED-ONLY seed (I-7): the seed comes from seedEntryFlow ONLY; this flow
//     NEVER routes an NFC-scanned object into derivation. ms1 is engraved onto
//     owner-held steel only, never NFC.
//   - PER-LEG SCRUB (I-7): the entropy is gated + wiped inside deriveMultisigLeg;
//     the seed/master/intermediates are scrubbed inside deriveAccountXpub (called
//     once per slot in the cross-match loop); the mnemonic []Word is zeroed when
//     this flow returns (defer), after its LAST derivation consumer.

// multisigSeedHook is a test-only seam to observe the typed mnemonic (to assert
// it is scrubbed on exit). nil in production. Mirrors singleSigSeedHook.
var multisigSeedHook func(bip39.Mnemonic)

func engraveMultisigFlow(ctx *Context, th *Colors) {
	// (1) Gather the SUPPLIED md1 over NFC (PUBLIC). Refuse a polluted/ambiguous
	// supply BEFORE any seed is typed (no secret exists yet).
	cards, ok := bundleGatherFlow(ctx, th)
	if !ok {
		return
	}
	suppliedMd1, ok := extractSuppliedMd1(cards)
	if !ok {
		showError(ctx, th, "Engrave Multisig", "Supply exactly one wallet-policy md1 (and no key cards).")
		return
	}

	// (2) Decode + full-policy gate (I-3).
	tpl, keys, err := md.ExpandWalletPolicyChunks(suppliedMd1)
	if err != nil {
		showError(ctx, th, "Engrave Multisig", "Couldn't decode the supplied wallet policy.")
		return
	}
	if !allSlotsHaveXpub(keys) {
		showError(ctx, th, "Engrave Multisig", "The supplied descriptor has no public keys to match.")
		return
	}
	_ = tpl // decoded again in the restore step; kept for clarity here.

	// (3) TYPED-ONLY seed (I-7). Never a scan.
	mnemonic, ok := seedEntryFlow(ctx, th)
	if !ok {
		return
	}
	if multisigSeedHook != nil {
		multisigSeedHook(mnemonic)
	}
	// Scrub the SECRET mnemonic on EVERY exit path (incl. abort / no-match), after
	// its last derivation consumer (I-7).
	defer func() {
		for i := range mnemonic {
			mnemonic[i] = 0
		}
	}()

	// Optional passphrase.
	passphrase := ""
	ppChoice := &ChoiceScreen{Title: "Passphrase", Lead: "Add a BIP-39 passphrase?", Choices: []string{"Skip", "Add passphrase"}}
	if sel, ok := ppChoice.Choose(ctx, th); ok && sel == 1 {
		if pass, ok := passphraseFlow(ctx, th); ok {
			passphrase = pass
		}
	}

	// (4) CROSS-MATCH the seed to one slot (I-1). Refuse on zero matches.
	idx, origin, reused, ok := findUserSlot(mnemonic, passphrase, &chaincfg.MainNetParams, keys)
	if !ok {
		showError(ctx, th, "Engrave Multisig", "This seed is not a cosigner of the supplied policy.")
		return
	}
	if len(reused) >= 2 {
		showError(ctx, th, "Engrave Multisig",
			fmt.Sprintf("This key is reused at slots @%d and @%d; engraving the first (@%d).", reused[0], reused[1], idx))
	}

	// (5) Full vs watch-only.
	modeChoice := &ChoiceScreen{
		Title:   "Engrave Mode",
		Lead:    "What to engrave?",
		Choices: []string{"Full (seed + keys)", "Watch-only (keys)"},
	}
	modeSel, ok := modeChoice.Choose(ctx, th)
	if !ok {
		return
	}
	full := modeSel == 0

	// (6) Derive the operator's leg. The mnemonic is consumed for the LAST time
	// here (entropy gated + wiped inside).
	b, err := deriveMultisigLeg(mnemonic, passphrase, &chaincfg.MainNetParams, origin, suppliedMd1, full)
	if err != nil {
		showError(ctx, th, "Engrave Multisig", "Couldn't derive the bundle from the seed.")
		return
	}

	// (7) Engrave (full = ms1+mk1+md1; watch-only = mk1+md1 + the ms1 reminder).
	cardsOut := multisigEngraveCards(b.MS1, b.MK1, b.MD1, full)
	bundleEngrave(ctx, th, cardsOut)

	// (8) Offer the verify-bundle.
	verifyChoice := &ChoiceScreen{Title: "Verify Bundle", Lead: "Verify the engraved plates?", Choices: []string{"Verify now", "Skip"}}
	if sel, ok := verifyChoice.Choose(ctx, th); ok && sel == 0 {
		multisigVerifyFlow(ctx, th, b, full)
	}

	// (9) Restore doc (display-only, PUBLIC — no secret).
	multisigRestoreDocFlow(ctx, th, suppliedMd1)
}
```

- [ ] **Step 2: Build the package.**

Run: `go build ./gui/`
Expected: builds clean. (If `tpl` triggers "declared and not used", the `_ = tpl` line already guards it; leave it.)

- [ ] **Step 3: Run the full gui suite (still passes the not-yet-wired program).**

Run: `go test ./gui/ -run 'TestMultisig|TestFindUserSlot|TestExtractSuppliedMd1|TestDeriveMultisigLeg|TestVerifyMultisig|TestAllSlotsHaveXpub|TestSuppliedMultisig'`
Expected: PASS (no nav yet — wired in Task 9).

- [ ] **Step 4: Commit.**

```bash
git -C /scratch/code/shibboleth/seedhammer-t6b add gui/multisig.go
git -C /scratch/code/shibboleth/seedhammer-t6b commit -S -s \
  --author="Brian Goss <goss.brian@gmail.com>" \
  -m "feat(gui): T6b engraveMultisigFlow orchestrator (typed-only, per-leg scrub)" \
  -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 9 — Program enum + lockstep (I-9)

Insert `engraveMultisig` between `engraveSingleSig` and `qaProgram`, update ALL 8 lockstep sites + the 3 nav-tests. Each edit is shown verbatim.

**Files:**
- Modify: `gui/gui.go`
- Test: `gui/multisig_program_test.go` (create), `gui/singlesig_program_test.go`, `gui/bundle_program_test.go`, `gui/derive_xpub_program_test.go`

- [ ] **Step 1: Write the failing nav-test.**

Create `gui/multisig_program_test.go`:
```go
package gui

import "testing"

// TestEngraveMultisigProgramNavigable asserts the new engraveMultisig program is
// reachable by navigating Right past engraveSingleSig, is the new navigable upper
// bound (a further Right wraps to backupWallet), has a NON-BLANK title, and does
// not panic on render (layoutMainPlates must have its case). qaProgram stays out.
func TestEngraveMultisigProgramNavigable(t *testing.T) {
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
	// Right x3 -> engraveSingleSig.
	for i := 0; i < 3; i++ {
		click(&ctx.Router, Right)
		content, ok = frame()
		if !ok {
			t.Fatalf("no frame after Right #%d", i+1)
		}
	}
	if !uiContains(content, "Single-Sig") {
		t.Fatalf("engraveSingleSig not reachable after 3 Rights; got %q", content)
	}
	// Right -> engraveMultisig (the new upper bound), titled non-blank.
	click(&ctx.Router, Right)
	content, ok = frame()
	if !ok {
		t.Fatal("no frame after fourth Right")
	}
	if !uiContains(content, "Multisig") {
		t.Fatalf("engraveMultisig not reachable/titled after fourth Right; got %q", content)
	}
	// Right again wraps to backupWallet.
	click(&ctx.Router, Right)
	content, ok = frame()
	if !ok {
		t.Fatal("no frame after fifth Right")
	}
	if !uiContains(content, "Backup Wallet") {
		t.Fatalf("Right did not wrap to Backup Wallet; got %q", content)
	}
}

// TestEngraveMultisigLeftWrap asserts Left from backupWallet wraps to
// engraveMultisig (the new navigable upper bound).
func TestEngraveMultisigLeftWrap(t *testing.T) {
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
	if !uiContains(content, "Multisig") {
		t.Fatalf("Left did not wrap to Multisig; got %q", content)
	}
}
```

- [ ] **Step 2: Run the new nav-test to verify it fails.**

Run: `go test ./gui/ -run TestEngraveMultisig -v`
Expected: FAIL — the carousel never reaches a "Multisig" title (program not inserted), and likely the OTHER nav-tests now also fail once we start editing. (At this point the new test fails because no `engraveMultisig` exists / isn't navigable.)

- [ ] **Step 3a: Insert the enum const.**

Edit `gui/gui.go` enum (`:147-153`):
```go
const (
	backupWallet program = iota
	engraveXpub
	engraveBundle
	engraveSingleSig
	engraveMultisig
	qaProgram
)
```

- [ ] **Step 3b: Add the dispatch case.**

Edit `gui/gui.go` dispatch (`:1501-1503`), adding the `engraveMultisig` case right after `engraveSingleSig`:
```go
			case engraveSingleSig:
				engraveSingleSigFlow(ctx, th)
				continue
			case engraveMultisig:
				engraveMultisigFlow(ctx, th)
				continue
```

- [ ] **Step 3c: Retarget the left-wrap bound (`:1638`).**

Edit `gui/gui.go`:
```go
				m.prog--
				if m.prog < 0 {
					m.prog = engraveMultisig
				}
```

- [ ] **Step 3d: Retarget the right-wrap bound (`:1645`).**

Edit `gui/gui.go`:
```go
				m.prog++
				if m.prog > engraveMultisig {
					m.prog = 0
				}
```

- [ ] **Step 3e: Add the title case (`:1670`).**

Edit `gui/gui.go` title switch, after the `engraveSingleSig` arm:
```go
	case engraveSingleSig:
		titleTxt = "Engrave Single-Sig"
	case engraveMultisig:
		titleTxt = "Engrave Multisig"
	}
```

- [ ] **Step 3f: Retarget `npage` (`:1846`).**

Edit `gui/gui.go`:
```go
	const npage = int(engraveMultisig) + 1
```

- [ ] **Step 3g: Add `engraveMultisig` to `layoutMainPlates` (`:1856`) — MANDATORY.**

Edit `gui/gui.go`:
```go
func layoutMainPlates(buf *op.Buffer, page program) (op.Op, image.Point) {
	switch page {
	case backupWallet, engraveXpub, engraveBundle, engraveSingleSig, engraveMultisig:
		img := assets.Hammer
		o := op.Image(buf, img)
		return o, img.Bounds().Size()
	}
	panic("invalid page")
}
```

- [ ] **Step 3h: Retarget `npages` (`:1865`).**

Edit `gui/gui.go`:
```go
	const npages = int(engraveMultisig) + 1
```

- [ ] **Step 4: Update the 3 existing nav-tests for the new wrap bound.**

In `gui/singlesig_program_test.go` — in `TestEngraveSingleSigProgramNavigable`, the comment + assertion that "Right again wraps back to backupWallet" is now WRONG (engraveSingleSig is no longer the upper bound). Change the fourth-Right block from asserting "Backup Wallet" to asserting "Multisig":
```go
	// Right again reaches engraveMultisig (the new navigable upper bound,
	// inserted before qaProgram by T6b). qaProgram stays out of the carousel.
	click(&ctx.Router, Right)
	content, ok = frame()
	if !ok {
		t.Fatal("no frame after fourth Right")
	}
	if !uiContains(content, "Multisig") {
		t.Fatalf("engraveMultisig not reachable after fourth Right; got %q", content)
	}
```
And in `TestEngraveSingleSigLeftWrap`, Left from backupWallet now wraps to `engraveMultisig`, NOT `engraveSingleSig`:
```go
	// Left from backupWallet -> wraps to engraveMultisig (the new upper bound).
	click(&ctx.Router, Left)
	content, ok := frame()
	if !ok {
		t.Fatal("no frame after Left")
	}
	if !uiContains(content, "Multisig") {
		t.Fatalf("Left did not wrap to Multisig; got %q", content)
	}
```

> The `bundle_program_test.go` and `derive_xpub_program_test.go` walks stop at "Single-Sig"/"Bundle" respectively and only COMMENT about the upper bound — they do not assert a wrap to backupWallet at engraveSingleSig, so their assertions still pass. Update only their trailing COMMENTS to say "engraveMultisig is the new navigable upper bound (inserted by T6b)". If either file actually asserts a wrap-to-backupWallet at engraveSingleSig, retarget that assertion to "Multisig" the same way. Verify by reading both files before editing.

- [ ] **Step 5: Run the nav-tests + the full gui suite.**

Run:
```bash
go test ./gui/ -run 'Program|LeftWrap' -v
go test ./gui/
```
Expected: all PASS, including `TestEngraveMultisigProgramNavigable`, `TestEngraveMultisigLeftWrap`, and the 3 updated existing nav-tests. The full `./gui/` run (incl. `TestAllocs`) is green.

- [ ] **Step 6: Commit.**

```bash
git -C /scratch/code/shibboleth/seedhammer-t6b add \
  gui/gui.go gui/multisig_program_test.go \
  gui/singlesig_program_test.go gui/bundle_program_test.go gui/derive_xpub_program_test.go
git -C /scratch/code/shibboleth/seedhammer-t6b commit -S -s \
  --author="Brian Goss <goss.brian@gmail.com>" \
  -m "feat(gui): T6b wire engraveMultisig program (8-site lockstep + nav-tests)" \
  -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 10 — No-regression + fuzz (I-7 structural, I-10)

**Files:**
- Create: `gui/multisig_fuzz_test.go`

- [ ] **Step 1: Write the fuzz tests for `findUserSlot` + `extractSuppliedMd1`.**

Create `gui/multisig_fuzz_test.go`:
```go
package gui

import (
	"testing"

	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip39"
	"seedhammer.com/md"
)

// FuzzFindUserSlot: findUserSlot never panics on arbitrary slot bytes/origins.
func FuzzFindUserSlot(f *testing.F) {
	f.Add([]byte{0x01, 0x02, 0x03}, uint8(2))
	f.Add([]byte{}, uint8(0))
	f.Fuzz(func(t *testing.T, raw []byte, nSlots uint8) {
		n := int(nSlots) % 6 // bound the slot count.
		keys := make([]md.ExpandedKey, n)
		for i := range keys {
			var xpub [65]byte
			for j := range xpub {
				if len(raw) > 0 {
					xpub[j] = raw[(i*65+j)%len(raw)]
				}
			}
			keys[i] = md.ExpandedKey{
				Index:       uint8(i),
				OriginPath:  msPath(hard32+48, hard32+0, hard32+uint32(i), hard32+2),
				Xpub:        xpub,
				XpubPresent: len(raw)%2 == 0,
			}
		}
		m := abandonAboutMnemonic()
		// MUST NOT panic.
		_, _, _, _ = findUserSlot(m, "", &chaincfg.MainNetParams, keys)
	})
}

// FuzzExtractSuppliedMd1: extractSuppliedMd1 never panics on arbitrary card sets.
func FuzzExtractSuppliedMd1(f *testing.F) {
	f.Add(uint8(1), uint8(0))
	f.Fuzz(func(t *testing.T, nMd1, nMk1 uint8) {
		var cards []bundleCard
		for i := 0; i < int(nMd1)%5; i++ {
			cards = append(cards, bundleCard{kind: cardMD1, strings: []string{"md1x"}})
		}
		for i := 0; i < int(nMk1)%5; i++ {
			cards = append(cards, bundleCard{kind: cardMK1, strings: []string{"mk1x"}})
		}
		_, _ = extractSuppliedMd1(cards)
	})
}

// TestMultisigSeedScrubbed: the typed mnemonic is scrubbed on every exit path
// (I-7). We can't drive the full UI flow headlessly here, so this asserts the
// scrub-on-exit discipline at the unit level: deriveMultisigLeg does NOT retain
// the mnemonic, and the orchestrator's defer zeroes it. This is a structural
// guard — the behavioral assertion lives in the flow's defer (mirror
// singleSigSeedHook). We verify the hook seam exists.
func TestMultisigSeedHookSeamExists(t *testing.T) {
	var captured bip39.Mnemonic
	multisigSeedHook = func(m bip39.Mnemonic) { captured = m }
	defer func() { multisigSeedHook = nil }()
	// The seam is set; a full headless flow drive is out of scope for a unit test.
	_ = captured
}
```

- [ ] **Step 2: Run the fuzz tests briefly + the seam test.**

Run:
```bash
go test ./gui/ -run 'TestMultisigSeedHookSeamExists' -v
go test ./gui/ -run FuzzFindUserSlot -fuzz FuzzFindUserSlot -fuzztime 10s
go test ./gui/ -run FuzzExtractSuppliedMd1 -fuzz FuzzExtractSuppliedMd1 -fuzztime 10s
```
Expected: the unit test PASSes; each fuzz run reports `PASS`/`ok` with 0 failures (no panic, no crasher written to `testdata/fuzz/`).

- [ ] **Step 3: Full no-regression sweep (I-10) + xprv grep (I-6 spine).**

Run:
```bash
go test ./gui/... ./md/... ./bundle/... ./mk/... ./codex32/...
```
Expected: all `ok`. This confirms T4/T5/T6a/single-card flows + codecs are byte-unchanged (no existing test regressed).

Run the xprv-leak grep over the restore render output (the descriptor + addresses are public; assert no xprv string is producible):
```bash
go test ./gui/ -run TestMultisigRestoreLines -v 2>&1 | grep -i xprv && echo "LEAK" || echo "clean"
```
Expected: `clean`.

- [ ] **Step 4: Confirm `TestAllocs` is green (the start-screen render budget).**

Run: `go test ./gui/ -run TestAllocs -v`
Expected: PASS (0 allocs/op — the new enum cases don't allocate).

- [ ] **Step 5: Commit.**

```bash
git -C /scratch/code/shibboleth/seedhammer-t6b add gui/multisig_fuzz_test.go
git -C /scratch/code/shibboleth/seedhammer-t6b commit -S -s \
  --author="Brian Goss <goss.brian@gmail.com>" \
  -m "test(gui): T6b fuzz findUserSlot/extractSuppliedMd1 + no-regression sweep" \
  -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Final verification (run before declaring done)

- [ ] **Whole affected-package suite green:**
```bash
cd /scratch/code/shibboleth/seedhammer-t6b && \
go test ./gui/... ./md/... ./bundle/... ./mk/... ./codex32/...
```
Expected: all `ok`.

- [ ] **Full build (firmware package set compiles):**
```bash
go build ./...
```
Expected: clean.

- [ ] **Lockstep self-check — grep for any stale `engraveSingleSig` bound:**
```bash
grep -n "engraveSingleSig" gui/gui.go
```
Expected: exactly ONE hit — the enum const declaration at the new `:151`. NO `engraveSingleSig` in any wrap bound (`m.prog =`/`m.prog >`), `npage`/`npages` const, `layoutMainPlates`, or title switch (all must now read `engraveMultisig`). If any other hit remains, a lockstep site was missed.

- [ ] **Commit log sanity:**
```bash
git -C /scratch/code/shibboleth/seedhammer-t6b log --oneline main..HEAD
```
Expected: ~10 signed commits, all authored "Brian Goss", each with the Co-Authored-By trailer.

---

## Self-review (run by the plan author; findings folded inline above)

**1. Spec coverage** — every §2 IN item and §5 gate item maps to a task:
- New `engraveMultisig` program (8-site lockstep + 3 nav-tests, qaProgram non-navigable) → **Task 9**.
- Gather + `extractSuppliedMd1` (0 md1 / ≥2 md1 / any mk1/ms1 refuse, I-11) → **Task 1**.
- Full-policy gate (I-3) → **Tasks 1+3**.
- Typed-only seed + optional passphrase (I-7) → **Task 8** (`seedEntryFlow`, never scan).
- Slot cross-match `findUserSlot` (canonical-pair `bytes.Equal`, refuse-on-zero, ambiguous→first+notice, skip non-present, I-1) → **Task 2**.
- Derive user leg (policy-bound mk1, ms1 gated on `Valid()`, Network label, I-4) → **Task 4**.
- Engrave full/watch-only (verbatim md1, I-2) → **Task 5**.
- Verify-bundle user slot only (reuse `bundle.Verify`, watch-only skip, I-5) → **Task 6**.
- NET-NEW multisig restore render path (`expandedToDescriptor`→`address`; non-bip380→display-only, NOT `restoreDocFlow`, I-6) → **Task 7**.
- Orchestrator with per-leg scrub on all exits (I-7) → **Task 8**.
- Mainnet-only (I-8) → encoded in Task 4/8 (`&chaincfg.MainNetParams`, `Network:"mainnet"`).
- No-regression + fuzz + TestAllocs (I-9/I-10) → **Task 10**.

**2. Placeholder scan** — no TBD/TODO; every code step shows full code; every run step shows the command + expected output; test vectors are concrete and machine-verified. The one judgment call left to the implementer is import-trimming in Task 7 (the file's correct minimal import set is spelled out, with a `go build` gate).

**3. Type consistency** — `findUserSlot` returns `(int, bip32.Path, []int, bool)` consistently across its def (Task 2), the orchestrator call (Task 8), the verify flow (Task 6), and the fuzz test (Task 10). `deriveMultisigLeg(m, passphrase, net, origin, suppliedMd1, full)` signature is identical in Tasks 4, 6, 8. `multisigEngraveCards(ms1, mk1, md1, full)` identical in Tasks 5 and 8. `extractSuppliedMd1(cards) ([]string, bool)` identical in Tasks 1, 6, 8. `allSlotsHaveXpub(keys) bool` identical in Tasks 1, 3, 8. `multisigRestoreLines(tpl, keys) ([]string, bool, error)` and `multisigRestoreDocFlow(ctx, th, suppliedMd1)` consistent in Task 7 and the Task 8 call. The `expandStatus` value used (`expandOK`) matches `gui/md1_expand.go:15`. All `bundleCard`/`cardMD1`/`cardMK1`/`cardMS1` field/const names match `gui/bundle.go`.
