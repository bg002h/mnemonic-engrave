# SPEC — T1: on-device receive/change address display (descriptor case)

**Status:** draft for the opus-architect R0 gate.
**Roadmap:** `design/RECON_seedhammer_constellation_terminal.md` (cycle **T1**, the foundation).
**Base:** fork `main` `384547d`. Fork-side only (no upstream PR).

---

## 1. Goal & scope

Let the operator **verify, on-device, which addresses a descriptor controls** before/after engraving it — the canonical air-gap check ("are these the addresses I expect?"). This is **pure wiring of the in-tree `address` package** (`address/address.go`: `Receive`/`Change`/`Supported`/`addressAt`), which is tested but **not imported by `gui/`/`cmd/`** today.

### In scope (T1)
- Wire an **address-view step** into the existing descriptor flow: wherever SH already holds a `*bip380.Descriptor` (an NFC-scanned output descriptor → `engraveObjectFlow` `case *bip380.Descriptor:` → `descriptorFlow` → `DescriptorScreen.Confirm`), add an affordance to view the descriptor's **receive and change addresses**.
- Support exactly the descriptor types `address` already handles: **single-sig** (P2PKH, P2WPKH, P2SH-P2WPKH, P2TR) and **sortedmulti** (P2SH, P2WSH, P2SH-P2WSH).
- Address list with **receive/change toggle** and **index paging** (e.g. show indices 0..4, page forward). **Display only** — no engraving of addresses, no NFC.

### Out of scope (explicit; later tiers)
- **mk1 → address** (path-inferred single-sig) and **md1-wallet-policy → address** — both need decoding the card to a descriptor first (**T2** dependency) or new path→script-type synthesis. Deferred to a post-T2 follow-on.
- Receive-**address verification** ("is this typed address mine?" gap-limit scan) — that's **T3**.
- Engraving an address / address-QR — not in T1 (keep it display-only; revisit if useful later).
- Any change to the `address` package's crypto (it ships tested).

---

## 2. Invariants (R0 must verify each)

1. **Display-only, public, deterministic.** Addresses are derived from the descriptor's **public** keys (xpub/childated pubkeys) via `address.Receive/Change`; nothing secret is touched, nothing is engraved or sent over NFC in T1. No CSPRNG. (Address derivation is `secp256k1` CKDpub — public.)
2. **Gate on `address.Supported(desc)`.** The address-view affordance is shown **only** when `address.Supported(desc)` is true (i.e. `Receive(desc,0)` does not return `errUnsupported`). For a template/placeholder descriptor with no concrete keys, or an unsupported script, the affordance is hidden — never a crash, never a blank screen.
3. **Errors are surfaced, not swallowed.** If `Receive`/`Change` returns an error for a supported type at some index (e.g. a key derivation edge), show a readable message and return to the confirm screen — never hang, never engrave as a side effect.
4. **No regression to the engrave path.** The existing `descriptorFlow` confirm→engrave behavior is unchanged when the user does not open the address view; adding the affordance must not alter the Plate/engrave result. Button-event draining must follow the established idiom (no queue-head block — the multishare/Fix? R0-C1 lesson).
5. **Network honesty.** The address string reflects the descriptor's key network (`Key.Network`); mainnet vs testnet is whatever the descriptor encodes (the `address` pkg already errors on a multisig mixing networks — surface that).

---

## 3. Source facts (verified against fork `384547d`)

- `address/address.go`: `Receive(desc *bip380.Descriptor, index uint32) (string, error)`, `Change(...)`, `Supported(desc) bool` (= `!errors.Is(Receive(desc,0), errUnsupported)`), `addressAt`, `derivePubKey`. Supports `bip380.Singlesig` {P2PKH, P2WPKH, P2SH-P2WPKH, P2TR} and `bip380.SortedMulti` {P2SH, P2WSH, P2SH-P2WSH}; multisig sorts keys (BIP-67) and builds the script. **Tested** (`address/address_test.go`) but **not imported by `gui/`/`cmd/`** (verified: no `seedhammer.com/address` import outside the pkg/tests).
- `bip380.Descriptor{Title; Script; Threshold; Type MultisigType; Keys []Key}`; `Key{Network *chaincfg.Params; MasterFingerprint; DerivationPath bip32.Path; Children []Derivation; KeyData; ChainCode; ParentFingerprint}`. `bip380.Parse(string) (*Descriptor, error)`.
- Producers of `*bip380.Descriptor` on-device: NFC scan → `nonstandard.OutputDescriptor(buf) (*bip380.Descriptor, error)` (`gui/scan.go:66`); also `bip380.Parse`. The descriptor reaches the UI via `engraveObjectFlow` `case *bip380.Descriptor:` → `descriptorFlow(ctx, th, desc)` (`gui/gui.go:2014`) → `DescriptorScreen.Confirm` (`gui/gui.go:2310`, returns `(Plate, bool)`; `descriptorFlow` loops Confirm→`NewEngraveScreen(...).Engrave`).
- GUI primitives available (from prior cycles): `ChoiceScreen`, `ErrorScreen`/`showError`, `layoutNavigation`/`NavButton`/`Clickable`/`Button1..3`/`Center`, `layoutTitle`, `widget.Label`/`Labelw`, the `runUI`/`click`/`runes` test harness, `assets.Icon{Back,Right,Left,Checkmark,Info,...}`.

---

## 4. Design

### 4.1 The address-view affordance
On `DescriptorScreen.Confirm`'s review screen, add a secondary nav affordance (e.g. **Button2**, `assets.IconRight` or `IconInfo`) **shown only when `address.Supported(desc)`**. Pressing it opens the address-list screen (§4.2); returning from it comes back to the confirm screen unchanged (the existing Back/engrave buttons behave as before). Button2 is **drained every frame** even when the affordance is hidden (avoid the queue-head-block idiom).

### 4.2 The address-list screen `descriptorAddressFlow(ctx, th, desc)`
A new screen that displays the descriptor's addresses:
- **Title:** "Receive addresses" / "Change addresses" (toggled).
- **Body:** a short list (default the first **5** indices) — each line `i: <address>` from `address.Receive(desc, i)` (or `Change` when toggled). Addresses are bech32/base58 strings; wrap/clip to the display width (mirror the codex32 field-line/label rendering).
- **Controls:** Button1 = Back (to the confirm screen); a toggle (e.g. **Button2**) = switch receive⇄change; a page-forward (e.g. **Center/Button3** or Up/Down nav) = next 5 indices (index window `start..start+4`, `start += 5`; clamp/stop at a sane cap, e.g. 50, to bound the loop). Drain all bound buttons every frame.
- On a per-index `Receive/Change` error, show `showError(ctx, th, "Address", <msg>)` and stay on the list (or fall back to Back) — never hang.
- **Pure display:** no engrave, no NFC, no state mutation of the descriptor.

### 4.3 Wiring
- `gui/gui.go` (or a new `gui/address_polish.go`): import `seedhammer.com/address`; add `descriptorAddressFlow`; add the `address.Supported`-gated affordance to `DescriptorScreen.Confirm` (or to `descriptorFlow` around the Confirm call — the plan pins which, to keep `Confirm`'s `(Plate,bool)` contract intact).

---

## 5. File manifest

| File | Change |
|---|---|
| `gui/address_polish.go` | **new** — `descriptorAddressFlow(ctx, th, desc *bip380.Descriptor)` (the address-list screen: receive/change toggle, index paging, per-index `address.Receive/Change`, error handling, button-drain). |
| `gui/gui.go` | **modify** — import `seedhammer.com/address`; add the `address.Supported`-gated address-view affordance into the descriptor confirm/`descriptorFlow` path (preserving the existing confirm→engrave behavior). |
| `gui/address_polish_test.go` | **new** — `address.Supported` gating; address-list renders the correct addresses (golden vectors mirrored from `address/address_test.go`); receive/change toggle; paging window; unsupported-descriptor hides the affordance; no-hang (button-drain). |

Unchanged/reused: the `address` package (crypto, tested), `nonstandard`, `bip380`, `DescriptorScreen` engrave path.

---

## 6. TDD

- **`address.Supported` gating:** a supported descriptor (single-sig wpkh + a sortedmulti, from `address_test.go` vectors) → affordance shown; an unsupported/placeholder descriptor → affordance hidden, confirm/engrave unaffected.
- **Address rendering:** drive `descriptorAddressFlow` (direct-call/`runUI`) on a known descriptor and assert the displayed receive[0..n]/change[0..n] strings match `address.Receive/Change` (golden values from `address/address_test.go`).
- **Toggle + paging:** receive⇄change switches the list; page-forward advances the index window; bounds clamp.
- **No-hang / no-regression:** button-drain idiom holds; the descriptor confirm→engrave path is byte-identical when the address view isn't opened (existing `TestEngraveScreen`/descriptor tests stay green).
- Host: `go test ./gui/... ./address/...`; `go vet`; `gofmt -l`. TinyGo `pico-plus2` build (CI) compiles the new gui code (and now imports `address`).

---

## 7. Process
cycle-prep recon (this spec's §3, verified) → R0 loop → plan → R0 loop → single-implementer TDD in worktree `seedhammer-wt-t1-address` (branch `feat/address-display` off `384547d`) → whole-diff execution review → merge no-ff signed+DCO → push `bg002h`. Reviews → `design/agent-reports/seedhammer-T1-address-*`. Signed+DCO, Brian Goss. No upstream PR.
