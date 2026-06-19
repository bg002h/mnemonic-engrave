# cycle-prep recon — 2026-06-18 — T3 receive-address verification

**Fork HEAD:** `d334861` (T2c merged). **Recon agent:** `a538eef83f440ced2` (verified vs fork source).

T3 goal: confirm "does THIS descriptor control THIS address?" — input a candidate address, gap-limit-scan the descriptor's derived range, report match + chain (receive/change) + index. Builds on T1's `address` pkg.

## Verified facts (file:line)
- **`address` pkg** (`address/address.go`): `Receive(desc *bip380.Descriptor, i uint32) (string,error)` (:24), `Change(...)` (:20), `Supported(desc) bool` (:28), `addressAt` (:35), `derivePubKey` (:116). Addresses are plain **strings** → equality is `==`. NO batch primitive — a gap scan loops `Receive`/`Change`. `addressAt` produces P2PKH/P2SH/P2WPKH/P2WSH/P2TR (+ testnet); network from `k.Network`. T3 only needs to match what the descriptor can derive.
- **In-tree parser:** `btcd/address/v2.DecodeAddress(addr, net)` (go.mod:6) handles bech32 (`bc1q`/`bc1p`) + base58 (`1`/`3`). Normalize the candidate via `DecodeAddress(...).String()` then compare to `Receive/Change` output (canonical-string match).
- **Descriptor source:** scanned `*bip380.Descriptor` (real xpubs) via `nonstandard.OutputDescriptor` → `DescriptorScreen`/`descriptorFlow` (`gui/scan.go:66`, `gui/gui.go:2052`). `address.Supported` is the guard. **Template-only md1 has no keys → out of scope** (T3 works on a scanned descriptor with xpubs).
- **T1 wiring:** `descriptorAddressFlow` (`gui/address_polish.go:26`) display flow; `DescriptorScreen.Confirm` Button2 affordance gated on hoisted `address.Supported` (`gui/gui.go:2361-2375`). T3 adds a sibling "Verify address" affordance (Button2 is taken → needs a new button or an intermediate ChoiceScreen).
- **Per-derivation cost (flag):** every `Receive/Change` runs full HD child derivation + secp256k1 point math (`derivePubKey` :116-157); multisig = N keys/index. On RP2350 a G=20×2-chain multisig scan ≈ 120 EC derivations — slow but a one-shot behind an explicit action, NOT a hot path. MUST run outside any alloc-gated frame loop (the existing code hoists `address.Supported` precisely for this). Alloc gate scope unchanged (`StartScreen.Flow`+`DescriptorScreen.Confirm` only).
- **Placement:** match logic → extend `address` pkg with `Find(desc, candidate, gap) (chain,index,found,err)` (headless-testable like `address_test.go`); UX → new `gui/verify_address.go`. Deps all present. Test harness unchanged (runUI/ExtractText/uiContains, NFCReader()==nil, click/press).

## THE input-method decision (shapes the cycle; needs a call)
The candidate address must get into the air-gapped device. Two paths, real tradeoffs:
- **SCAN (NFC):** matches SH's NFC-native ingest model; testable to the fork's standard (headless `Find` + direct result-flow call + a code-reviewed scanner-shell, exactly like the shipped `mk1GatherFlow`). BUT requires the operator's address source to emit NFC; the literal NFC byte-read isn't unit-testable (`NFCReader()==nil` — same as ALL fork scan paths).
- **TYPED:** works for any address source; BUT net-new keyboard work — both generic keyboards force-uppercase (`gui.go:1216`) → corrupt mixed-case base58, so must build on the case-preserving `PassphraseKeyboard` with a bech32+base58 charset (~150-250 LOC, the recon's #1 risk); hand-typing a 60-char address is tedious/error-prone.
- **BOTH:** ~550 LOC total.

The `address.Find` core + the verify-result display are common to all three and are the load-bearing, fully-testable part (~150-250 LOC).

## Sizing / risk
~250 LOC (scan-only) → ~550 LOC (typed+scan). Biggest risk: the typed-address keyboard (case-preserving, custom charset). Secondary: per-derivation EC cost on multisig gap-scans; keeping any `DescriptorScreen.Confirm` additions 0-alloc.

## Recommended scope ordering
Build `address.Find` + the verify-result flow first (common, tested), then the chosen input path(s). Gap limit default 20/chain (reuse the `addrMaxIndex=49` precedent as an upper bound). Fork-side only; no upstream PR. Gate: spec → opus R0 → GREEN before code.
