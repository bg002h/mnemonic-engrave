# SPEC — T3: on-device receive-address verification

> Cycle T3 of the "SeedHammer as air-gapped constellation terminal" roadmap.
> Recon: `design/cycle-prep-recon-T3-receive-address-verify.md` (verified vs fork `d334861`). User chose input method = **Both (scan + typed)**.
> Base: fork `d334861` (T2c merged). Fork-side only; no upstream PR.

## 1. Goal & scope

Let an operator confirm **"does THIS descriptor control THIS address?"** on the air-gapped touchscreen: from a scanned `*bip380.Descriptor`, input a candidate Bitcoin address (by **NFC scan** OR **on-screen typing**), gap-limit-scan the descriptor's derived receive+change ranges, and report whether the candidate matches — and on which **chain (receive/change)** and **index**. Read-only; no engrave, no mutation.

### In scope (T3)
- **`address.Find`** (new, in the `address` pkg): given a descriptor + candidate address + gap limit, normalize the candidate, derive receive[0..G) and change[0..G), return `(chain, index, found)` on first match. Headless, fully unit-tested.
- **Candidate normalization/validation** via the in-tree `btcd/address/v2.DecodeAddress` (canonical-string compare).
- **GUI verify flow** off `DescriptorScreen` (gated on `address.Supported`): choose **Scan** or **Type** → obtain candidate → `address.Find` → display result (✓ chain+index, or ✗ not-in-first-G, or invalid/wrong-network).
- **Typed-address entry**: a case-preserving on-screen keyboard (built on `PassphraseKeyboard`, NOT the force-uppercasing generic `Keyboard`) for bech32+base58 addresses; validity gated by `DecodeAddress`.
- **Scan-address input**: a new `scanner.Scan` branch recognizing a bare address (via `DecodeAddress`) feeding the verify flow's own scanner-shell (the shipped `mk1GatherFlow` idiom).

### Out of scope (explicit)
- Engraving anything (T3 is read-only verification).
- Template-only md1 / any descriptor without keys (can't derive — `address.Supported` guards it; T3 operates on a scanned `*bip380.Descriptor` with real xpubs).
- Address types the descriptor can't derive (T3 only needs to match what `addressAt` produces: P2PKH/P2SH/P2WPKH/P2WSH/P2TR + testnet).
- Deriving beyond the gap limit; xpub/descriptor *authoring*; balance/UTXO lookup (no network).

## 2. Invariants (R0 MUST verify each — Critical if violated)

1. **2.1 `Find` correctness.** For a descriptor `address.Supported` accepts, `Find(desc, cand, G)` returns `(chain, idx, true)` iff `cand` (normalized) equals `Receive(desc, idx)` (chain=0) or `Change(desc, idx)` (chain=1) for some `idx ∈ [0,G)`, scanning receive then change, ascending index, returning the FIRST match. No false positive (a non-controlled address never matches) and no false negative within `[0,G)` (every derivable address in range matches). Verified by headless tests over known descriptor→address vectors (mirror `address/address_test.go`). (Degenerate-path caveat in §4.1 R0-M2: a range/wildcard-less descriptor reports `(0,0)` for any match.)
1b. **2.1a `Find` is panic-safe / total (R0-I1).** `Find` MUST NOT panic on ANY input — including a keyless descriptor (`addressAt`/`Supported`/`Receive` panic on `desc.Keys[0]`), an unparseable candidate, or a wrong-network candidate. Each returns a typed error (`ErrUnsupported`/`ErrAddrUnparseable`/`ErrAddrWrongNetwork`), never a panic. Verified by a headless keyless-descriptor test.
1c. **2.1b `Find` propagates derivation errors (R0-I2).** A `Receive`/`Change` error mid-scan MUST be returned, NOT silently compared as `"" == wantStr` (which would swallow the error and mis-report a non-match). Verified by a test injecting a derivation error.
2. **2.2 Canonical-string comparison.** The candidate is normalized via `DecodeAddress(cand, net).String()` (net from `desc.Keys[0].Network`) before comparison to `Receive`/`Change` output (also canonical strings). An unparseable candidate, or one valid only for a different network, yields a clear `err`/`found=false` (NOT a panic, NOT a spurious match). Mixed-case base58 is preserved (a correctly-typed legacy address normalizes correctly; a checksum-invalid typo → parse error).
3. **2.3 Read-only.** The verify flow + result screen + typed keyboard contain NO engrave/NFC-write/NDEF/plate/mutation call — only derivation (read), render, and navigation returns.
4. **2.4 Typed entry is case-preserving.** The address keyboard MUST NOT force-uppercase (the generic `Keyboard.rune()` does — `gui.go:1216`); it MUST preserve case so mixed-case base58 (`1…`/`3…`) is enterable. Built on `PassphraseKeyboard` (case-preserving) or equivalent. Validity is gated by `DecodeAddress` on the assembled string.
5. **2.5 0-alloc gate untouched.** `Find` and its `Receive`/`Change` derivations (which allocate) MUST run OUTSIDE any benchmarked frame loop — once per explicit user action, results rendered afterward. No allocating per-frame work added to `StartScreen.Flow` or `DescriptorScreen.Confirm` (the only `TestAllocs`-gated paths). New verify/keyboard screens are not alloc-gated.
6. **2.6 No regression.** The existing descriptor flow (`DescriptorScreen`, `descriptorAddressFlow` show-addresses, engrave) and all prior cycles (T1/T2a/T2b/T2c) are behaviorally unchanged. The verify affordance is additive and gated on `address.Supported`.
7. **2.7 Bounded gap scan.** `G` is a fixed, bounded default (20 per chain) with a hard cap (≤ the `addrMaxIndex=49` precedent). A no-match returns a clear "not found in first G" — never an unbounded scan. The multisig per-derivation secp256k1 cost (≈ 2·G·N EC ops) is acceptable as a one-shot behind an explicit action, not a hot path.
8. **2.8 No secret handling.** Descriptors/addresses are PUBLIC; no `Unshared`/`wipeBytes`/scrub. Verification is read-only public-key derivation.

## 3. Source facts (verified vs fork `d334861`; citations in the recon)
- `address.Receive(desc,i) (string,error)` (`address/address.go:24`), `Change` (:20), `Supported` (:28), `addressAt` (:35) producing P2PKH/P2SH/P2WPKH/P2WSH/P2TR (+testnet); addresses are plain strings.
- `btcd/address/v2.DecodeAddress(addr, *chaincfg.Params) (Address,error)` (go.mod:6) parses bech32+base58; `Address.String()`/`IsForNet`.
- Descriptor source: scanned `*bip380.Descriptor` (real xpubs) → `descriptorFlow`/`DescriptorScreen` (`gui/gui.go:2052`, `gui/scan.go:66`). `address.Supported` guards derivability.
- T1 affordance: `DescriptorScreen.Confirm` Button2 → `descriptorAddressFlow` (`gui/gui.go:2361-2375`, `gui/address_polish.go:26`), `address.Supported` hoisted out of the frame loop (alloc gate).
- Case-preserving keyboard: `PassphraseKeyboard` (`gui/passphrase_keyboard.go:182`, "NO ToUpper"); generic `Keyboard.rune` force-uppercases (`gui/gui.go:1216`) — unusable for base58.
- Scanner-shell idiom for an in-flow candidate scan: `mk1GatherFlow` (`gui/mk1_inspect.go`).
- Test harness: `runUI`/`ExtractText`/`uiContains`, `testPlatform.NFCReader()==nil` (scan not unit-injectable — the verify-result path is tested by calling it with a candidate string; `Find` headless; typed via keyboard drive), `click`/`press`. Alloc gate = `StartScreen.Flow`+`DescriptorScreen.Confirm` only.

## 4. Design

### 4.1 `address.Find` (headless core)
```go
// Find scans the descriptor's receive then change ranges [0,gap) for an address
// equal to candidate (canonicalized). Returns the chain (0=receive, 1=change),
// index, and found. err if the descriptor is unsupported or the candidate can't
// be parsed for the descriptor's network.
func Find(desc *bip380.Descriptor, candidate string, gap uint32) (chain int, index uint32, found bool, err error)
```
Algorithm (panic-safe / total — R0-I1): **FIRST `if len(desc.Keys) == 0 { return 0,0,false, ErrUnsupported }`** — `addressAt`/`Supported`/`Receive` all index `desc.Keys[0]` with NO length guard and PANIC on a keyless descriptor, so `Find` MUST guard before touching `desc.Keys[0]` AND before calling `Supported`. Then: `net := desc.Keys[0].Network`; `want, err := DecodeAddress(candidate, net)`; if err → `ErrAddrUnparseable`; if `!want.IsForNet(net)` → `ErrAddrWrongNetwork`; `wantStr := want.String()`. Then scan, **propagating derivation errors (R0-I2)**: for `i ∈ [0,gap)`: `got, derr := Receive(desc,i); if derr != nil { return 0,0,false, derr }; if got == wantStr { return 0,i,true,nil }`; then the same for `Change` → chain=1. Else `(0,0,false,nil)`. (Receive-first ordering is the report convention.) `gap` validated `1..=addrFindMaxGap` where **`addrFindMaxGap` is a NEW constant in package `address`** (R0-M4 — the gui `addrMaxIndex=49` can't be imported without a cycle; mirror its value as rationale). **Degenerate-path note (R0-M2):** a `Supported` descriptor whose path has neither a `RangeDerivation` nor a `WildcardDerivation` (e.g. `wpkh(xpub/0/0)`) derives the SAME address for every `(chain,index)` → `Find` reports `(0,0)` for a match (index/chain are not meaningful). This is correct (the address IS controlled) but the reported chain/index is degenerate; the result screen should phrase a match as "controlled by this descriptor" with chain/index shown only when meaningful.

### 4.2 GUI verify flow (`gui/verify_address.go`)
Reached from `DescriptorScreen` (gated `address.Supported`). **Wiring (R0-I3): there is no Button4 — `Button1/2/3` all exist and `DescriptorScreen.Confirm` uses all three (Back/addresses/Confirm).** So the existing Button2 address affordance becomes a small **ChoiceScreen** "Show addresses" / "Verify an address" (the only feasible route; a "new button" is impossible). Additive, `Supported`-gated. The verify path:
1. **Input choice**: `ChoiceScreen` "Scan" / "Type".
2. **Scan** → a scanner-shell (own `NFCReader` + goroutine, the `mk1GatherFlow` idiom; safe because **`StartScreen.Flow` already returned and closed its reader before `DescriptorScreen` ran** (R0-M6) → no concurrent readers) that accepts the first `DecodeAddress`-valid address; Back exits.
   **Type** → the case-preserving address keyboard (4.3) → candidate string on OK.
3. **Render a "Verifying…" frame (R0-M3)**, THEN run `address.Find(desc, candidate, 20)` once (outside any frame loop; a multisig 40-set scan blocks for up to seconds on RP2350 — the operator must see progress, not a frozen UI).
4. **Result screen**: `✓ Receive address #<i>` / `✓ Change address #<i>` / (degenerate-path: `✓ Controlled by this descriptor`) / `✗ Not found in first 20` / `Invalid address` / `Different network`. Back returns. Read-only.

### 4.3 Typed address keyboard
A case-preserving keyboard (built on `PassphraseKeyboard` — the `Fragment`-append-without-ToUpper mechanism, `passphrase_keyboard.go:182`) with its existing lower/upper/digit pages, which (R0-verified) cover the ENTIRE bech32+base58 character space. No charset restriction is required for correctness — the assembled string is validated by `DecodeAddress` (OK enabled only when it parses for the descriptor's network); invalid/partial input shows a status line. Mirrors `inputCodex32Flow`'s validated-entry shape (live readout + status + OK-when-valid). **The readout MUST be UNMASKED (R0-M1):** `PassphraseKeyboard.Layout` masks the fragment with `*` (`passphrase_keyboard.go:341-344`) — correct for a secret passphrase, WRONG for a public address (and it would break the `ExtractText` case-preservation test). Use `revealed=true` by default or a dedicated unmasked variant.

### 4.4 Scan-address recognition
Add a branch to `scanner.Scan` (`gui/scan.go`): after the existing format probes, try `address/v2.DecodeAddress(string(buf), <net>)`; on success return a new `addressText`-style value. Used by the verify flow's scanner-shell. **`engraveObjectFlow` intentionally has NO `addressText` case (R0-M5):** a bare address scanned at the top-level StartScreen falls through to `default: return false` → the existing "unknown format" status (unchanged user-visible behavior, preserving invariant 2.6) — the new value is only meaningful inside the verify flow's own scanner-shell, not as a standalone engrave object. The recognition (`DecodeAddress`) is headless-testable; the NFC routing is code-reviewed (consistent with the fork's other scan paths under `NFCReader()==nil`).

## 5. File manifest (indicative; plan pins)
- **Modify** `address/address.go` (+ `address_test.go`) — add `Find` (with the keyless-guard, R0-I1) + sentinels `ErrUnsupported`/`ErrAddrUnparseable`/`ErrAddrWrongNetwork` + the new cap const `addrFindMaxGap` (package `address`, R0-M4).
- **Create** `gui/verify_address.go` (+ `verify_address_test.go`) — the verify flow (input choice, scanner-shell, result screen) + the case-preserving address keyboard.
- **Modify** `gui/scan.go` — the `DecodeAddress` recognition branch (+ test for the recognizer).
- **Modify** `gui/gui.go` (`DescriptorScreen.Confirm`/`descriptorAddressFlow`) — the verify affordance (Show/Verify choice or new button), `Supported`-gated, no per-frame alloc.

## 6. TDD
- **`address.Find` (load-bearing, headless):** over known descriptors (load via `nonstandard.OutputDescriptor`, as `address_test.go` does): a real receive[k] address → `(0,k,true)`; a real change[k] → `(1,k,true)`; a foreign/random valid address → `found=false`; an address just past the gap → `found=false`; an unparseable string → `ErrAddrUnparseable`; a wrong-network address → `ErrAddrWrongNetwork`; **a keyless descriptor → `ErrUnsupported`, NO panic (R0-I1, the load-bearing safety test — use `recover` to fail on panic)**; **a descriptor that errors mid-derivation → that error propagated, not a silent non-match (R0-I2)**. Cover singlesig (wpkh/pkh/tr/sh-wpkh) AND sortedmulti (wsh/sh) descriptors.
- **Verify flow (GUI):** call the result-rendering flow directly with a candidate string (bypassing NFC, per `NFCReader()==nil`): a known receive address → "Receive address #k" rendered; a non-match → "Not found"; invalid → "Invalid address". Typed entry: drive the address keyboard via `click`/runes, assert case preserved (a base58 `1…`/`3…` round-trips un-uppercased) and OK-enables only on a valid parse.
- **No-regression / alloc / scope:** existing `descriptorAddressFlow` show-addresses + engrave unchanged; `DescriptorScreen.Confirm` additions stay 0-alloc (`TestAllocs` passes); the scan recognizer accepts a valid address + rejects a descriptor/seed string.

## 7. Process
- **R0 gate (mandatory, this doc):** opus-architect to 0C/0I before any code. Fold → persist verbatim to `design/agent-reports/seedhammer-T3-*-R*.md` → re-dispatch until GREEN.
- Then `IMPLEMENTATION_PLAN_seedhammer_T3_receive_address_verify.md` → its own R0 (plan reviewer materializes + build/runs; `address.Find` parity + the typed-keyboard case-preservation are the proofs). Phase A (`address.Find` + result flow, tested) before Phase B (scan + typed input wiring).
- Single-implementer TDD in a worktree off `d334861`. Commits signed (`-S`) + DCO (`-s`), author "Brian Goss <goss.brian@gmail.com>", trailer `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. Explicit-path staging.
- Mandatory whole-diff adversarial execution review (fuzz `Find`/`DecodeAddress` handling for panics; confirm no false-positive match) → merge no-ff → push `bg002h`.
