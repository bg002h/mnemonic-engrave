<!--
Persisted verbatim. opus-architect R0 gate of the T1 address-display spec
(SPEC_seedhammer_T1_address_display.md @ d713e19). Reviewer agentId aaec702b688c6808a.
Verdict: NOT GREEN 0C/1I (2 minor). The reviewer verified every API/type/Script-set/wiring-point/
reachability/acyclicity/footprint claim against fork 384547d — all correct; T1 has a genuine
reachable input (NFC scan → nonstandard.OutputDescriptor → *bip380.Descriptor → descriptorFlow),
Button2 is free on DescriptorScreen.Confirm, gui→address is acyclic and adds zero new deps, golden
address vectors exist in address_test.go. The single blocker: IMPORTANT-1 — DescriptorScreen.Confirm's
frame loop is a strict 0-alloc hot path (gui_test.go TestAllocs/BenchmarkAllocs), and
address.Supported calls Receive(desc,0) (secp256k1 derivation, allocating); calling it per-frame
breaks the gate + burns a derivation/frame. Fix: hoist Supported (compute once), keep the address
list a separate screen, add TestAllocs to the regression gate. Minors: M-1 pin the paging control/cap
(Button3 free on the list screen; avoid auto-repeat Up/Down); M-2 test a custom-children descriptor so
receive≠change. Disposition: folded I-1 (invariants 2+6, §4.1, §6) + M-1 (§4.2) + M-2 (§6); re-dispatching.
The text below is the agent's report exactly as returned; do not edit.
-->

# R0 GATE REVIEW — T1 address display (spec)

**Reviewer:** opus architect (adversarial, read-only). **Spec:** `design/SPEC_seedhammer_T1_address_display.md` @ `d713e19`. **Authoritative base:** fork `main` `384547d` at `/scratch/code/shibboleth/seedhammer`.

## Verification Results (file:line evidence)

**1. API match — VERIFIED EXACT.**
- `address.Receive(desc *bip380.Descriptor, index uint32) (string, error)` — `address/address.go:24-26`. `Change(...)` same signature — `:20-22`. Spec §3 signature is byte-exact.
- `Supported(desc) bool` = `!errors.Is(Receive(desc,0), errUnsupported)` — `address/address.go:28-31`. Spec §2 invariant 2 / §3 semantics are exactly right.
- `addressAt`, `derivePubKey` — `:35`, `:116`. Named correctly.
- **Supported-type set is exactly right, not over/under-claimed:** Singlesig → P2PKH (`:83`), P2WPKH/P2SH-P2WPKH (`:86`), P2TR (`:89`). SortedMulti → P2SH (`:64`), P2WSH/P2SH-P2WSH (`:66`). Anything else returns `errUnsupported` (`:70`, `:93`, `:99`). Matches spec §1/§3 set verbatim. Multisig BIP-67 sort confirmed at `:56-58`.
- **Network-mix error** (spec invariant 5) confirmed: `address/address.go:46-47` returns `errUnsupported` when a multisig mixes networks — and because it wraps `errUnsupported`, `Supported` correctly returns false for such a descriptor (so the affordance is hidden, not crashing).
- `bip380` types match spec §3: `Descriptor{Title;Script;Threshold;Type MultisigType;Keys []Key}` (`bip380/bip380.go:20-26`); `Key{Network;MasterFingerprint;DerivationPath;Children;KeyData;ChainCode;ParentFingerprint}` (`:28-36`); `MultisigType` Singlesig/SortedMulti (`:90-95`); `Script` values (`:56-67`); `Parse` (`:269`).
- **Golden vectors exist and are usable** — `address/address_test.go:9-85`: 8 descriptors (pkh/wpkh/sh-wpkh/tr single-sig + wsh/sh-wsh/sh sortedmulti, incl. a custom-children `/1234/<5;6>/*` case) each with 2-3 receive + change golden addresses, built via `nonstandard.OutputDescriptor`. Directly mirror-able into `gui/address_polish_test.go`.

**2. Wiring feasibility — VERIFIED, and Button2 is free.**
- `*bip380.Descriptor` is in hand at `engraveObjectFlow case *bip380.Descriptor:` → `descriptorFlow(ctx,th,scan)` (`gui/gui.go:1875-1876`) → `DescriptorScreen.Confirm` (`:2310`, returns `(Plate,bool)`; loop at `:2018-2027`). Spec §3 line references are correct.
- `DescriptorScreen.Confirm` binds **only Button1 (Back, `:2322`) and Button3 (Confirm, `:2323`)**. **Button2 is free.** `layoutNavigation` is variadic `(...NavButton)` (`gui/gui.go:1723`), so adding a third nav button is mechanical.
- **`(Plate,bool)` contract is preservable:** a Button2 handler that opens `descriptorAddressFlow` and returns to the `for` loop touches neither return path — `Plate` is only produced via the existing Button3→`validateDescriptor`→`ChoiceScreen` branch (`:2329-2343`). Invariant 4 holds *by construction* as long as the affordance only opens a sub-flow.
- **Established precedent is near-identical:** `gui/codex32_polish.go:98-122` is exactly this pattern — Button1 Back, gated Button2 (Recover) drained unconditionally then conditionally acted on (`:108-111`), Button3 primary, with `layoutNavigation` appending the gated button conditionally (`:119-122`). Same idiom in `slip39_polish.go:439-445` and `seedxor_polish.go:88-94`. T1 fits the codebase grain.
- **Button-drain idiom confirmed.** `Clickable.Clicked` drains the queue via `Next` in a loop (`gui/widget.go:35-46`); the "drain every frame even when hidden, to avoid blocking the queue head" rule (the multishare R0-C1 lesson) is a live, tested convention — `gui/codex32_polish_test.go:300,318-322`, `slip39_polish_test.go:107-113`. Spec §2.4 / §4.1 / §4.2 capture it correctly.

**3. Reachability reality-check — PASS (not dead UI).**
- The on-device source of `*bip380.Descriptor` is the NFC scanner: `StartScreen.Flow` runs `scanner.Scan(r)` (`gui/gui.go:1538`) → `nonstandard.OutputDescriptor(buf)` (`gui/scan.go:66`, sig `nonstandard/parse.go:36` returns `*bip380.Descriptor`) → `engraveObjectFlow` (`gui/gui.go:1498`). This is a *live* path, so T1 genuinely has a real input to act on. **Not dead UI.**
- It is the *only* on-device producer reaching the flow (no `bip380.Parse` call sites in `gui/`), so the spec's deferral of mk1/md1-native derivation to T2 (§1 out-of-scope) is correct: those require decoding the card to a descriptor first, which T1 cannot reach yet. Correctly scoped.

**4. Invariants — all sound.**
- Display-only/public/deterministic: derivation is secp256k1 CKDpub on public xpubs (`address/address.go:116-157`); no secret, no CSPRNG, no NFC write, no engrave in the address sub-flow. Holds.
- `Supported` gate prevents crash/blank on placeholder/unsupported descriptors — confirmed by the `errUnsupported` wrapping and `Supported`'s `errors.Is` check.
- Error surfacing via `showError(ctx,th,...)` (`gui/slip39_polish.go:22`) and `ErrorScreen.Layout` (`gui/gui.go:206`) exist and don't hang.
- No new invariant missing. (See IMPORTANT-1 for a hazard the spec under-specifies.)

**5. No import cycle / footprint — CLEAN.**
- `gui → address` is acyclic: `address` imports only btcsuite/decred/`bip380` (`address/address.go:5-18`); grep confirms **no** `seedhammer.com/gui` or `seedhammer.com/cmd` import anywhere in `address/`.
- `seedhammer.com/address` is currently imported nowhere outside its own package/tests (grep returned empty) — spec §3 claim verified.
- **Zero new dependency.** Every module `address` pulls (`btcsuite/btcd/address/v2`, `txscript/v2`, `chaincfg/v2`, `btcec/v2`, `btcutil/v2`, `decred/dcrd/.../secp256k1/v4`) is already a *direct* require in `go.mod:6-11`, and `txscript/v2`/`address/v2`/`schnorr` are already pulled by in-tree `address`/`bip32`. T1 adds no new heavy stack.

**6. TDD adequacy — sufficient and runnable.**
- Harness present: `click(r, bs...)` (`gui/event_test.go:42`), `runes` (`:68`), `runUI` (`gui/gui_test.go:467`). `nonstandard` is importable from gui tests (already used at `gui/scan.go:11`), so golden vectors mirror cleanly.
- Gating, render-correctness (assert against `address.Receive/Change` goldens), toggle, paging-window, and no-hang (drained Button2) are all testable with the existing harness, matching the precedent tests (`codex32_polish_test.go:300+`).

**7. Scope — right-sized (pure wiring), with one cost the spec hides (see below).**

---

## Findings

### IMPORTANT-1 — Spec omits the `TestAllocs`/0-allocation invariant that gates `DescriptorScreen.Confirm`'s frame loop.
**Where:** spec §2 (Invariants), §4.1 (the affordance in `Confirm`), §6 (TDD list).
**Evidence:** `gui/gui_test.go:50-98` — `BenchmarkAllocs` drives `ds.Confirm(ctx, &descriptorTheme)` as one of its screens (`:60-69`) and `TestAllocs` (`:93-98`) **fails if AllocsPerOp > 0**. The descriptor confirm frame loop is therefore a strict zero-allocation hot path.
**Problem:** The spec's §4.1 says the affordance is "shown only when `address.Supported(desc)`". `Supported` calls `Receive(desc,0)`, which runs full secp256k1 child derivation and string formatting (`address/address.go:35-157`) — allocating, and not free. If the implementer calls `address.Supported(desc)` **inside the per-frame loop**, it will (a) blow the 0-alloc `TestAllocs` gate and (b) burn a key-derivation every frame on the hot path. The spec gives no guidance to hoist this.
**Required fix:** State explicitly that `address.Supported(desc)` MUST be computed **once** (e.g. cached in the `DescriptorScreen` struct or computed before the `for !ctx.Done` loop in `Confirm`), never per-frame; and add `TestAllocs`/`BenchmarkAllocs` to §6 as a must-stay-green regression gate (the modified `Confirm` and the new `descriptorAddressFlow` confirm-return must remain 0-alloc, or the alloc set must be consciously updated). This is the single concrete hazard that could turn "harmless wiring" into a CI break and a hot-path regression; pin it in the plan.

### MINOR-1 — Paging "cap e.g. 50" and "page-forward control" are illustrative, not pinned.
**Where:** §4.2 ("Center/Button3 or Up/Down nav"; "clamp/stop at a sane cap, e.g. 50").
**Note:** Acceptable for a spec, but the plan must pin (a) the exact page-forward control — note Button3 is *free* on the address-list screen (it's the confirm screen that uses Button3), and Up/Down `Clickable`s auto-repeat (`gui/widget.go:50-68`) which interacts with paging; and (b) the exact cap and clamp behavior so the paging test is deterministic. No correctness risk; flagging so the plan resolves it rather than the implementer.

### MINOR-2 — `derivePubKey` default-children behavior is worth a test note.
**Where:** §6 (TDD).
**Note:** For a key with no explicit children, `derivePubKey` defaults to `<0;1>/*` (`address/address.go:118-130`), so receive uses index-0 branch and change uses index-1 branch (`:142-144`). The golden vectors in `address_test.go` already exercise both default and custom-children (`/1234/<5;6>/*`) cases; the gui test should mirror at least one custom-children descriptor so the receive/change toggle is genuinely distinguished, not just index-0 vs index-0. Not blocking.

*(No CRITICAL findings: no dead UI, no import cycle, no engrave-path regression by construction, no wrong API claim. The wiring point, button availability, reachability, and dependency footprint all check out.)*

---

**NOT GREEN — 0 Critical / 1 Important.**

The spec is correct on every API, type, Script-set, wiring-point, reachability, acyclicity, and footprint claim — this is a genuinely well-scoped wiring spec and most of it passes hard verification. The one blocker is the unaddressed **0-allocation `TestAllocs` invariant** on `DescriptorScreen.Confirm`'s frame loop (`gui/gui_test.go:50-98`): the spec must require hoisting `address.Supported(desc)` out of the per-frame path and add `TestAllocs` to the regression gate. Fold IMPORTANT-1 (and ideally the two MINORs), persist this review verbatim to `design/agent-reports/seedhammer-T1-address-*`, and re-dispatch for GREEN.
