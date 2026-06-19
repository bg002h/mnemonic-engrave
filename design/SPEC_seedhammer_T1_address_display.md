# SPEC — T1: on-device receive/change address display (descriptor case)

**Status:** GREEN at R1, then **AMENDED post-implementation** (§4.2/§6): the execution review caught a real defect — fixed-count paging silently drops off-screen indices for wrapping long addresses (P2WSH multisig: only ~3 of 5 fit on 480×320). §4.2 now mandates **measure-and-advance** paging (gap-free, no-skip). Re-gated as a combined spec+plan amendment R0→R1 GREEN. **SHIPPED to fork `main` `68e6ead` (pushed `bg002h`)** after the whole-diff execution review returned GREEN (0C/0I). Reviews: `design/agent-reports/seedhammer-T1-address-spec-review-{R0,R1}.md` + `…-paging-fix-review-{R0,R1}.md` + `…-execution-review.md`. **T1 DONE.**
**Roadmap:** `design/RECON_seedhammer_constellation_terminal.md` (cycle **T1**, the foundation).
**Base:** fork `main` `384547d`. Fork-side only (no upstream PR).

---

## 1. Goal & scope

Let the operator **verify, on-device, which addresses a descriptor controls** before/after engraving it — the canonical air-gap check ("are these the addresses I expect?"). This is **pure wiring of the in-tree `address` package** (`address/address.go`: `Receive`/`Change`/`Supported`/`addressAt`), which is tested but **not imported by `gui/`/`cmd/`** today.

### In scope (T1)
- Wire an **address-view step** into the existing descriptor flow: wherever SH already holds a `*bip380.Descriptor` (an NFC-scanned output descriptor → `engraveObjectFlow` `case *bip380.Descriptor:` → `descriptorFlow` → `DescriptorScreen.Confirm`), add an affordance to view the descriptor's **receive and change addresses**.
- Support exactly the descriptor types `address` already handles: **single-sig** (P2PKH, P2WPKH, P2SH-P2WPKH, P2TR) and **sortedmulti** (P2SH, P2WSH, P2SH-P2WSH).
- Address list with **receive/change toggle** and **index paging** (show the addresses that fit the screen, page forward by the count shown — §4.2). **Display only** — no engraving of addresses, no NFC.

### Out of scope (explicit; later tiers)
- **mk1 → address** (path-inferred single-sig) and **md1-wallet-policy → address** — both need decoding the card to a descriptor first (**T2** dependency) or new path→script-type synthesis. Deferred to a post-T2 follow-on.
- Receive-**address verification** ("is this typed address mine?" gap-limit scan) — that's **T3**.
- Engraving an address / address-QR — not in T1 (keep it display-only; revisit if useful later).
- Any change to the `address` package's crypto (it ships tested).

---

## 2. Invariants (R0 must verify each)

1. **Display-only, public, deterministic.** Addresses are derived from the descriptor's **public** keys (xpub/childated pubkeys) via `address.Receive/Change`; nothing secret is touched, nothing is engraved or sent over NFC in T1. No CSPRNG. (Address derivation is `secp256k1` CKDpub — public.)
2. **Gate on `address.Supported(desc)` — computed ONCE, never per-frame.** The address-view affordance is shown **only** when `address.Supported(desc)` is true (i.e. `Receive(desc,0)` does not return `errUnsupported`). For a template/placeholder descriptor with no concrete keys, or an unsupported script, the affordance is hidden — never a crash, never a blank screen. **`Supported` calls `Receive(desc,0)`, which runs secp256k1 child derivation + string formatting (allocating + non-trivial), so it MUST be computed once** (cached on the `DescriptorScreen` struct, or computed before the `for !ctx.Done` loop in `Confirm`) and **never inside the per-frame loop** — see invariant 6.
6. **Zero-allocation hot path preserved (R0-caught).** `DescriptorScreen.Confirm`'s frame loop is a strict **0-allocation** path gated by `gui_test.go`'s `TestAllocs`/`BenchmarkAllocs` (which drive `ds.Confirm`). The address-view affordance must NOT add per-frame allocations: the `Supported` check is hoisted (invariant 2), and the new `descriptorAddressFlow` is a **separate** screen (its own loop, entered on Button2) so its per-index `Receive/Change` allocations never run on `Confirm`'s frame path. `TestAllocs`/`BenchmarkAllocs` MUST stay green (the modified `Confirm` returns to 0 alloc/op; the address-list screen is not part of the benchmarked set).
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
On `DescriptorScreen.Confirm`'s review screen, add a secondary nav affordance on the **free Button2** (`DescriptorScreen.Confirm` currently binds only Button1=Back and Button3=Confirm; Button2 is free), icon `assets.IconRight` (or `IconInfo`), **shown only when `Supported` is true**. `Supported` is computed **once** — store it on the `DescriptorScreen` struct (set when the descriptor is assigned) or compute it before the `for !ctx.Done` loop; do **NOT** call `address.Supported`/`Receive` inside the frame loop (invariant 2/6, the 0-alloc gate).

**0-alloc-safe nav construction (R1 MINOR-1 — do NOT copy the codex32/seedxor `append`-chain idiom here):** `DescriptorScreen.Confirm` IS in the `BenchmarkAllocs`/`TestAllocs` set (codex32's confirm is not), so the per-frame `[]NavButton{…}` passed to `layoutNavigation` must stay a **fixed (non-escaping) composite literal** — `layoutNavigation` only ranges over `btns` and never stores it (`gui.go:1723,1788-1800`), so a fixed 3-element literal is stack-allocatable exactly like today's 2-element one. To make the 3rd button **conditional** without an `append` (which can heap-allocate and would break the gate), keep all three in the fixed literal and set the address button's `Style = StyleNone` when `!supported` (`layoutNavigation` renders `StyleNone` as the empty `op.Op{}`, `gui.go:1726-1728`); its `Clickable` is still constructed and **drained every frame** even when hidden (queue-head-block idiom). Acting on the Button2 click is gated on `supported`.

Pressing Button2 (when supported) opens the address-list screen (§4.2); returning comes back to the confirm screen unchanged (Back/engrave behave as before).

### 4.2 The address-list screen `descriptorAddressFlow(ctx, th, desc)`
A new screen that displays the descriptor's addresses:
- **Title:** "Receive addresses" / "Change addresses" (toggled).
- **Body — MEASURE-AND-ADVANCE paging (R1-exec defect fix; do NOT page by a fixed count).** Each line `i: <address>` (`address.Receive(desc,i)` / `Change` when toggled) is a long bech32/base58 string that **wraps across multiple rows** via `widget.Labelw` at the display width. A fixed page size (e.g. 5) overflows the content area and the off-screen lines are **dropped** by `op` clipping — silently skipping those indices forever (verified: on 480×320 a P2WSH multisig fits only ~3 of 5). So the screen MUST render only the addresses that **fit** the content height, and **page-forward by the number actually shown** (gap-free). Concretely: starting at `start`, for each index measure the line's wrapped height with the buffer-free `ctx.Styles.body.Measure(width, "%s", line)` (`text.go:56`); **include the first index unconditionally** (guarantees progress + ≥1 shown even on a tiny screen), then include each subsequent index only while it fits within the content rectangle (`screen.CutTop(leadingSize).CutBottom(leadingSize)`, `leadingSize=44`); stop at the first that doesn't fit. `shown` = the count rendered. Recompute only on entry / toggle / page (off the hot path; `Measure` allocates no draw ops).
- **Controls (pinned):** Button1 = Back; **Button2 = toggle receive⇄change** (resets `start=0`); **Button3 = page-forward** — `start += shown` (gap-free: the next page begins at the index right after the last one shown, so no index is ever skipped). Button3 is free on THIS screen (only the confirm screen uses Button3 for engrave). **Hard cap: do not advance past index 49** (`if start+shown ≤ 49 { start += shown }`, bounding the loop). Avoid Up/Down `Clickable`s (they auto-repeat — `widget.go`); use the discrete Button3. Drain all bound buttons every frame.
- **No-skip invariant (the load-bearing correctness property):** every index in `0..49` is viewable by paging — no index is silently dropped off-screen. (Tested on BOTH a single-sig fixture *and* a long-address P2WSH fixture, since fit-per-page differs by address length.)
- On a per-index `Receive/Change` error, show `showError(ctx, th, "Address", <msg>)` and return — never hang.
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
- **Address rendering:** drive `descriptorAddressFlow` (direct-call/`runUI`) on a known descriptor and assert the displayed receive[0..n]/change[0..n] strings match `address.Receive/Change` (golden values from `address/address_test.go`). **Include at least one custom-children descriptor** (e.g. the `/1234/<5;6>/*` vector in `address_test.go`) so the receive vs change toggle is genuinely distinguished (default keys use `<0;1>/*` → receive=branch-0, change=branch-1; the test must assert receive≠change, not index-0-vs-index-0).
- **Toggle:** Button2 switches receive⇄change (resets to index 0).
- **No-skip paging (the load-bearing regression test):** **observe each rendered page, then advance one page (Button3)** — observe-before-advance, so the entry page (index 0) is seen before any page-forward — and assert **every** `address.Receive(desc,i)` for `i∈0..7` appears in some rendered frame, for BOTH a single-sig fixture (more fit/page) AND a long-address P2WSH fixture (fewer fit/page). This proves measure-and-advance never silently drops an index off-screen. (Sequencing matters: pre-queueing all the Button3 clicks before the first frame would page over index 0 and fail on *correct* code — the click must follow each observed frame.)
- **No-hang / no-regression:** button-drain idiom holds; the descriptor confirm→engrave path is byte-identical when the address view isn't opened (existing `TestEngraveScreen`/descriptor tests stay green).
- **0-allocation gate (R0 IMPORTANT-1):** `TestAllocs`/`BenchmarkAllocs` (`gui_test.go`) MUST stay green — the modified `DescriptorScreen.Confirm` is 0 alloc/op (the `Supported` gate is hoisted out of the frame loop; the address-list screen is a separate, non-benchmarked flow). Run them explicitly.
- Host: `go test ./gui/... ./address/...`; `go vet`; `gofmt -l`. TinyGo `pico-plus2` build (CI) compiles the new gui code (and now imports `address`).

---

## 7. Process
cycle-prep recon (this spec's §3, verified) → R0 loop → plan → R0 loop → single-implementer TDD in worktree `seedhammer-wt-t1-address` (branch `feat/address-display` off `384547d`) → whole-diff execution review → merge no-ff signed+DCO → push `bg002h`. Reviews → `design/agent-reports/seedhammer-T1-address-*`. Signed+DCO, Brian Goss. No upstream PR.
