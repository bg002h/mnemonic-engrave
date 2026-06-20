# SPEC — SeedHammer II T6c **Phase B**: on-device multisig policy PICKER (the "choose" half)

**Status:** ✅ R0 GREEN (0C/0I) — agent `a71fac556ac5e728c`; review `design/agent-reports/seedhammer-T6c-phaseB-spec-R0-round0.md`. Cleared for the implementation plan (its own R0).

**USER DECISIONS (2026-06-20, post-R0 — resolve the 2 flagged product ambiguities; the 4 architect-defaultable ones are LOCKED as recommended):**
- **(#3 ordering) USER-PICKS self-slot `@S`** — the Build flow includes a bounded `ChoiceScreen` (`@0..@{n-1}`) for which slot is the operator's own key; gathered cosigners fill the remaining slots in gather order. (NOT the architect's self-`@0` default.)
- **(#5 fingerprints) USER CHOOSES at build time** — a bounded `ChoiceScreen` "Include key fingerprints? No / Yes"; the policy is kept HOMOGENEOUS (all slots same presence: Omit → no fp TLVs anywhere; Include → every slot's master fp). The choice affects the WalletPolicyId (Phase A: fp-absent `7b716421…` vs fp-present `639cabcf…`); show the resulting stub so the operator can match their coordinator. **A3 byte-match test MUST drive the Omit (fp-absent) path to reproduce the `7b716421…` T6b fixture (R0-M4).**
- **LOCKED architect defaults:** templates = offer all 3 sortedmulti wrappers (`wsh`/`sh(wsh)`/`sh`), highlight `wsh` (#1); n ∈ 2..5, k ∈ 1..n (#2); `OriginShared` only, divergent deferred (#4); enforce self-key derived at the policy's shared origin (#6).
- 4 R0 Minors (M1 mixed-fp clarity — now bounded by the homogeneous rule; M2 citation drift; M3 `originmode-errmsg`; M4 A3 fp-absent replay) fold into the IMPLEMENTATION_PLAN.

**(superseded) Status:** DRAFT — awaiting opus R0 gate (must reach 0C/0I before any IMPLEMENTATION_PLAN or code).
**Author:** single spec author (T6c-B).
**Date:** 2026-06-19.
**Scope class:** internal Go fork feature (firmware `gui/` package). NOT a `me` CLI surface; NOT a `md/` wire change (that was Phase A, shipped). GUI-heavy, hardware-UNvalidated.
**Fork HEAD:** `seedhammer` `f323dd2` (Go 1.26.4; T6c Phase A merged). **Authoritative wire (Rust md-codec):** `descriptor-mnemonic` `c85cd49` (`md-codec` v0.36.0) — relevant only via the shipped, byte-proven `md.EncodeMultisig`.
**Grounding:** `design/cycle-prep-recon-T6c-encode-multisig.md` (Topic 5 = picker/gather/lockstep) + `design/SPEC_seedhammer_T6c_phaseA_encode_multisig.md` + `design/agent-reports/seedhammer-T6c-phaseA-exec-review-round0.md` (the shipped encoder + its ordering contract). Every cited fork line number below was RE-READ at `f323dd2` (citations decay every merge; the recon's `8eb51d7` numbers were re-verified, several drifted — e.g. the T6b flow is now split across `gui/multisig*.go` files).

> **⚠ MANDATORY EXPERIMENTAL WARNING (USER-MANDATED, HARD REQUIREMENT — non-negotiable).** This is a GUI-heavy, **hardware-UNvalidated** authoring path with no coordinator/hardware round-trip. The Build-policy flow MUST present an **unskippable, loud, operator-acknowledged** `ConfirmWarningScreen` immediately before any engrave, stating the device-authored policy is NOT validated end-to-end and the user MUST verify the assembled descriptor against their coordinator/wallet BEFORE funding. See §"The mandatory EXPERIMENTAL warning". This requirement is itself a gate item: a plan/impl that makes it skippable, deferrable, or silent FAILS R0.

---

## Why

The fork can today only *consume* a supplied multisig wallet-policy md1 (T6b, `engraveMultisigFlow`, `gui/multisig.go:35`). T6c Phase A shipped the headless **`md.EncodeMultisig`** (`md/encode_multisig.go:88`) — a byte-faithful, order-preserving `sortedmulti` k-of-n assembler that returns `(out []string, stub [4]byte, slots []SlotInfo, err error)`. Phase B is the **"choose" half of choose-or-supply**: an on-device GUI that lets a user ASSEMBLE a multisig wallet-policy on the device — pick the shape, pick k/n, gather the other cosigners' keys (as PUBLIC mk1 cards over NFC), derive the user's own key from the typed seed, and engrave the resulting full bundle. The device is the AUTHORITATIVE creator here (there is no coordinator to match); the user EXPORTS the resulting md1 to their cosigners.

The recon (Topic 5) and the Phase A exec-review establish the spine: the wire format is shipped + byte-proven (no risk lives here); the genuine risk is the **on-device UX** — cosigner-ordering determinism, correct slot placement of the user's own key, correct mis-derivation guards, and the unvalidated-authoring hazard the EXPERIMENTAL warning addresses. Phase B is ~250–400 LOC of bounded-`ChoiceScreen` composition + gather wiring, reusing the entire T6b/T6a derive→engrave→verify→restore machinery after assembly.

---

## Scope

### IN (Phase B)

1. **Choose-or-supply front-door.** Extend `engraveMultisigFlow` (`gui/multisig.go:35`) with a `ChoiceScreen` as its FIRST screen (before the current line 38 `bundleGatherFlow`): `["Supply policy (md1)", "Build policy"]`. **"Supply policy (md1)"** → the existing T6b body UNCHANGED (current lines 38–129). **"Build policy"** → the new Phase-B authoring flow (a new function, e.g. `buildMultisigFlow`). Back from the front-door returns. This keeps the `engraveMultisig` program enum slot (`gui/gui.go:152`) and touches NO lockstep site (Invariant I-LOCKSTEP).

2. **Template / script picker.** A bounded `ChoiceScreen` selecting the top-level wrapper, mapping 1:1 to the shipped `md.MultisigScript` enum:
   - `wsh(sortedmulti)` → `md.MultisigWsh` (P2WSH) — **always offered**.
   - `sh(wsh(sortedmulti))` → `md.MultisigShWsh` (P2SH-P2WSH) — offered (encoder supports it).
   - `sh(sortedmulti)` → `md.MultisigSh` (legacy P2SH) — offered (encoder supports it).
   - **DECISION (R0 to confirm):** offer all three, default-highlight `wsh(sortedmulti)`. Rationale: the encoder + decoder + restore-doc (`expandedToDescriptor`, sortedmulti subset) handle all three; restricting to wsh-only is a defensible simpler alternative (R0 may prefer wsh-only for v1 — flagged as an open question, not a blocker).

3. **k / n selection.** Two bounded `ChoiceScreen`s (NO free-form numeric widget exists; `ChoiceScreen` is the only bounded picker — `gui/gui.go:1359`,`:1373`):
   - **n** first: `Choices = ["2","3","4","5"]` → **n ∈ 2..5** (DECISION below). (`n=1` is single-sig, out; the cap is a UX/plate-count bound well within the encoder's `n≤32` guard.)
   - **k** second: `Choices = ["1".."n"]` → **k ∈ 1..n**. Built dynamically from the chosen n.
   - **DECISION (R0 to confirm): n ∈ 2..5, k ∈ 1..n.** Rationale: 2..5 covers the overwhelming majority of real multisig (2-of-3, 3-of-5, 2-of-2); each cosigner adds an NFC gather + a plate, so a small cap bounds the UX/engrave burden. The encoder guards n≤32/k≤n regardless (Phase A I7), so the GUI cap is purely a usability ceiling. R0 may widen to 2..9 — flagged as a tunable, not load-bearing for correctness.

4. **Cosigner gather + user-slot derive.** Collect the **N total** cosigner keys:
   - The **OTHER N−1** keys arrive as **mk1 cards over NFC (PUBLIC)** via the shipped `bundleGatherFlow` (`gui/bundle_flow.go:95`) → each `mk.Decode`d card (`mk/mk.go:148`) exposing `Xpub` (base58), `Path`, `Fingerprint` (string; `""` ⇒ absent), `Network`. Each xpub is parsed to `(chainCode[32], compressedPubkey[33], parentFP)` via the shipped `decodeXpubBytes` (`gui/singlesig_derive.go:99`).
   - The **user's OWN key** is derived from the **typed-only** seed: `seedEntryFlow` (`gui/derive_xpub.go:82`) → optional passphrase → `deriveAccountXpub(m, passphrase, &chaincfg.MainNetParams, originPath)` (`gui/derive.go:19`) returning a base58 xpub + masterFP, then `decodeXpubBytes` into the same `(chainCode, compressedPubkey)` form.
   - **Collection model (DECISION, the load-bearing UX choice — R0 centerpiece):** **N = (N−1 gathered mk1) + 1 self-derived**, with the user's slot at a USER-CHOSEN, bounded index (a `ChoiceScreen` `["@0".."@{n-1}"]`). See §"Ordering contract".

5. **Assemble + engrave (reuse T6b/T6a).** Build `md.EncodeMultisigRequest{ Cosigners (in the deterministic on-device order), K, Script, OriginMode, SharedOrigin }` → `md.EncodeMultisig(req)` → `(assembledMd1, stub, slots, err)`. Then **reuse the T6b path UNCHANGED, feeding the assembled md1 where T6b feeds the supplied md1**:
   - Full vs watch-only `ChoiceScreen` (mirror `gui/multisig.go:98-102`).
   - `deriveMultisigLeg(mnemonic, passphrase, &chaincfg.MainNetParams, userOrigin, assembledMd1, full)` (`gui/multisig_derive.go:32`) — it is descriptor-SOURCE-agnostic: it takes the md1 strings VERBATIM, derives the user's policy-bound mk1 (`Stubs=[WalletPolicyIDStubChunks(assembledMd1)]`, which **must equal** the `stub` Phase-A returned — Invariant I-STUB) + ms1 (full only).
   - `multisigEngraveCards(b.MS1, b.MK1, b.MD1, full)` (`gui/multisig_engrave.go:11`) → `bundleEngrave` (`gui/multisig.go:118-119`).
   - Verify-bundle `ChoiceScreen` → `multisigVerifyFlow(ctx, th, b, full)` (`gui/multisig_verify.go:36`).
   - Restore doc `multisigRestoreDocFlow(ctx, th, tpl, keys)` (`gui/multisig_restore.go:58`), where `(tpl, keys) = md.ExpandWalletPolicyChunks(assembledMd1)` — round-trips through the same bip380-expressible (sortedmulti) path.

6. **The ordering-verification handle SHOWN to the user.** Before engrave (as part of the confirm screen sequence), display the `(stub, slots)` returned by `EncodeMultisig`: the 4-byte policy stub and the per-slot `@N → fingerprint` map (`SlotInfo{Index, Fingerprint, FpPresent}`), so the user can RECORD/VERIFY which key landed in which slot and reconcile the stub against their coordinator. This is the device's only ordering self-check (Invariant I-ORDER).

7. **The MANDATORY EXPERIMENTAL warning** (§ below; unskippable; Invariant I-WARN).

8. **TDD acceptance** (§ below): flow tests that assert the front-door routing, the picker bounds, the assembled-md1 byte-equality vs `md.EncodeMultisig` (no GUI-side re-encode), the user-slot placement, the warning's unskippability, and the security-spine scrub.

### OUT — deferred (note only; NOT in this spec)

- **Miniscript beyond `sortedmulti`** (bare `multi`, `multi_a`/`sortedmulti_a` taproot, tapscript trees, general miniscript). The encoder refuses these (Phase A I5); the picker offers only the three `sortedmulti` wrappers.
- **Self-as-multiple-slots** (the same user key occupying >1 slot). The Build flow inserts the user's key at exactly one chosen slot.
- **n > the chosen cap** (default >5; or >9 if R0 widens). The encoder allows n≤32, but the picker caps for UX.
- **Free-form numeric index entry.** No such widget exists (`ChoiceScreen` only); all of template/n/k/slot are bounded `ChoiceScreen`s.
- **Testnet authoring.** Mainnet-only (`&chaincfg.MainNetParams`), matching the flagship (`gui/singlesig_restore.go:79` D1 posture).
- **Coordinator round-trip / hardware validation.** Precisely the gap the EXPERIMENTAL warning flags.
- **Importing a partial/started bundle to "resume" assembly.** One-shot assembly per session.

---

## Verified facts (file:line at fork `f323dd2`; RE-READ, not trusted from prior docs)

- **V1 — the shipped encoder + its return handle.** `md.EncodeMultisig(req EncodeMultisigRequest) (out []string, stub [4]byte, slots []SlotInfo, err error)` (`md/encode_multisig.go:88`). `EncodeMultisigRequest{ Cosigners []MultisigCosigner; K uint8; Script MultisigScript; OriginMode OriginMode; SharedOrigin []PathComponent }` (`:58-64`). `MultisigCosigner{ ChainCode [32]byte; CompressedPubkey [33]byte; Fingerprint [4]byte; FpPresent bool; Origin []PathComponent }` (`:48-54`). `MultisigScript` ∈ `{MultisigWsh, MultisigShWsh, MultisigSh}` (`:24-30`). `OriginMode` ∈ `{OriginShared, OriginDivergent}` (`:36-41`). `SlotInfo{ Index uint8; Fingerprint [4]byte; FpPresent bool }` (`:69-73`). `PathComponent{ Hardened bool; Value uint32 }` (`md/encode_singlesig.go:20`).

- **V2 — ORDER-PRESERVING, no key sort (the load-bearing contract).** `EncodeMultisig` assigns `Cosigners[i] → @i`; canonicalize is the identity permutation for this AST (`md/encode_multisig.go:13-21` godoc; `multiSigTree` emits indices `[0..n-1]` in input order, `:184-201`). The Phase-A exec-review PROVED this: same 3 keys in 3 orders → 3 DISTINCT `WalletPolicyId`s (`design/agent-reports/seedhammer-T6c-phaseA-exec-review-round0.md`, "MANDATE #2"). **⇒ the GUI owns the deterministic order; the encoder never reorders.**

- **V3 — the stub/slots handle is already returned.** `EncodeMultisig` returns `stub` (== `WalletPolicyIDStub(d)`, `md/encode_multisig.go:157`; == `WalletPolicyIDStubChunks(out)`) and `slots` (the per-`@N`→fp map, `:123-133`). So the device need NOT recompute anything to SHOW the ordering handle.

- **V4 — T6b orchestrator + insertion point.** `engraveMultisigFlow(ctx *Context, th *Colors)` (`gui/multisig.go:35`); first user-facing screen = `bundleGatherFlow` at `:38`. Front-door ChoiceScreen inserts at the top (before `:38`). Existing internal `ChoiceScreen`s at `:79` (passphrase), `:98-102` (full/watch-only), `:122` (verify). Seed scrub `defer` at `:71-75`.

- **V5 — reusable derive→engrave→verify→restore seams (descriptor-source-agnostic).**
  - `deriveMultisigLeg(m, passphrase, net, origin bip32.Path, suppliedMd1 []string, full bool) (bundle.Bundle, error)` (`gui/multisig_derive.go:32`) — takes md1 strings VERBATIM (`:58-60`), `Stubs=[WalletPolicyIDStubChunks(suppliedMd1)]` (`:42`), gates `m.Valid()` then `m.Entropy()` and `wipeBytes(entropy)` (`:64-66`).
  - `multisigEngraveCards(ms1 string, mk1, md1 []string, full bool) []bundleCard` (`gui/multisig_engrave.go:11`).
  - `multisigVerifyFlow(ctx, th, derived bundle.Bundle, full bool)` (`gui/multisig_verify.go:36`).
  - `multisigRestoreDocFlow(ctx, th, tpl md.Template, keys []md.ExpandedKey)` (`gui/multisig_restore.go:58`); faithful-or-refuse address path = bip380-expressible (sortedmulti) subset only (`gui/multisig_restore.go:8-32`).
  - `md.ExpandWalletPolicyChunks(strs)` (`md/expand.go:102`) → `(tpl, keys)`; `allSlotsHaveXpub(keys)` gate precedent (`gui/multisig.go:54`).

- **V6 — gather + decode + xpub-parse seams.** `bundleGatherFlow(ctx, th) ([]bundleCard, bool)` (`gui/bundle_flow.go:95`); `bundleGatherer` (`gui/bundle.go:119`); `offerChunkedMK1` (`gui/bundle.go:177`); classification refuses ms1 secret over NFC — `clsMs1Refuse` (`gui/bundle.go:46`, logic `:64-71`). `mk.Decode(in []string) (Card, error)` (`mk/mk.go:148`); `Card{ Network, Path, Fingerprint, Stubs, Xpub }` (`mk/mk.go:132-139`) — **NO explicit `FpPresent` field; presence = `Fingerprint != ""`** (the wire flag is internal, `mk/mk.go` ~`:246`). `decodeXpubBytes(xpub string) (chainCode [32]byte, compressedPubkey [33]byte, parentFP uint32, err error)` (`gui/singlesig_derive.go:99`); `originComponents(path bip32.Path) []md.PathComponent` (`gui/singlesig_derive.go:128`).

- **V7 — typed-only seed + derive + mainnet.** `seedEntryFlow(ctx, th) (bip39.Mnemonic, bool)` (`gui/derive_xpub.go:82`) — typed-only, never a scan. `deriveAccountXpub(m, passphrase, net, path) (xpub string, masterFP uint32, err error)` (`gui/derive.go:19`); scrub ordering = serialize xpub (`acct.String()`, `:50`) BEFORE `k.Zero()` (`:51`). Mainnet = `&chaincfg.MainNetParams` (the T6b flow hard-uses it: `gui/multisig.go:87,111`).

- **V8 — the warning pattern to mirror.** `ConfirmWarningScreen{ Title, Body, Icon }` (`gui/gui.go:232`), `.Layout(ctx, th, dims) (op.Op, ConfirmResult)` (`gui/gui.go:328`) returning `ConfirmNone|ConfirmNo|ConfirmYes`. Mandatory-warning idiom: `childSeedWarning` (`gui/bip85.go:145`) and `stubZeroWarning` (`gui/derive_xpub.go:237`) — both build the screen, loop `.Layout`, return `true` ONLY on `ConfirmYes` (hold-to-confirm), `false` on `ConfirmNo`/Back. The caller MUST abort the engrave when the warning returns `false`.

- **V9 — the bounded picker (the ONLY one).** `ChoiceScreen{ Title, Lead, Choices []string }` (`gui/gui.go:1359`), `.Choose(ctx, th) (int, bool)` (`gui/gui.go:1373`) — returns (selected index, ok); ok=false on Back/cancel. Sequential/nested `ChoiceScreen` is an established pattern (`gui/multisig.go:79,98,122`; mdmkFlow; backupWalletFlow). **There is NO free-form numeric-entry widget.**

- **V10 — lockstep sites (option (a) touches NONE).** Program enum `gui/gui.go:147-155` (`engraveMultisig` @ `:152`, `bip85Derive` @ `:153`); t5-M1 static guard `var _ [1]struct{} = [qaProgram - bip85Derive]struct{}{}` (`gui/gui.go:164`); dispatch switch `:1502-1527` (`engraveMultisig → engraveMultisigFlow` @ `:1516`); title switch `:1680-1693`; `layoutMainPlates` case `:1876-1877`; carousel wrap `:1653-1664`; `npage`/`npages` `:1867`,`:1886`. Extending `engraveMultisigFlow` adds NO program ⇒ no enum/guard/dispatch/title/plate/carousel change.

---

## Faithfulness + security spine

### Faithfulness
- **Byte-identical to a coordinator.** Phase A proved `md.EncodeMultisig` is byte-exact vs Rust md-codec (`@c85cd49`) AND reproduces the T6b fixture (`WalletPolicyId 7b716421…`). Phase B does ZERO wire work: it builds the `EncodeMultisigRequest` and calls the shipped encoder. ⇒ the assembled md1 is byte-identical to what a coordinator would produce for the SAME shape + keys + **order** (the order being the contract the GUI must get right — §Ordering).
- **No GUI-side re-encode.** The GUI never serializes a descriptor itself; the only md1-bytes producer is `md.EncodeMultisig`, and the only md1-bytes consumers downstream (`deriveMultisigLeg`, `ExpandWalletPolicyChunks`, restore) take the assembled strings VERBATIM (Invariant I-VERBATIM).
- **Round-trip closes the loop.** `tpl,keys = ExpandWalletPolicyChunks(assembledMd1)` feeds the restore doc through the same bip380-expressible (sortedmulti) path T6b uses (V5); a non-expressible shape is structurally impossible here because the picker only emits sortedmulti.
- **Stub consistency.** `WalletPolicyIDStubChunks(assembledMd1)` (computed inside `deriveMultisigLeg`) MUST equal the `stub` returned by `EncodeMultisig` — same bytes, same id (Invariant I-STUB; asserted in tests).

### Security spine (mirror T6b/T6a)
- **Cosigner mk1s are PUBLIC** (NFC-allowed; xpub+origin+fp only). ms1 (codex32 secret) is **NFC-REFUSED** at classify (`clsMs1Refuse`, V6) — the gather cannot ingest a secret.
- **Master seed is TYPED-ONLY** (`seedEntryFlow`, V7) — never scan→derive. The user's own key is minted via `deriveAccountXpub` (scrubs seed/master internally, serialize-before-zero, V7).
- **ms1 is SECRET / steel-only**, engraved in full mode only, never NFC (reused `deriveMultisigLeg` path, V5).
- **Per-leg scrub**: entropy gated + wiped inside `deriveMultisigLeg` (`:64-66`); the typed mnemonic `[]Word` zeroed on EVERY exit via the same `defer` idiom T6b uses (`gui/multisig.go:71-75`); the Build flow MUST install this scrub defer immediately after `seedEntryFlow` returns (Invariant I-SCRUB).
- **The assembled md1 is PUBLIC** (xpub+fp+origin+threshold). **Mainnet-only.**
- **No new secret residency.** The picker assembles PUBLIC policy material from public cosigner keys + one self-derived public xpub; the only secret (the seed) follows the identical posture to T6b/engraveXpub.

---

## The mandatory EXPERIMENTAL warning (USER-MANDATED — hard requirement)

A dedicated **unskippable** `ConfirmWarningScreen`, e.g. `multisigBuildExperimentalWarning(ctx, th) bool`, mirroring `childSeedWarning` (`gui/bip85.go:145`) / `stubZeroWarning` (`gui/derive_xpub.go:237`):

- **Title:** e.g. `"EXPERIMENTAL"` (or `"Unverified Policy"`).
- **Body (substance, exact wording for impl):** the device-authored multisig policy is **NOT validated end-to-end** — there is no hardware or coordinator round-trip. The user **MUST verify the assembled descriptor (and the shown policy stub + per-slot fingerprints) against their coordinator/wallet BEFORE funding.** Hold button to confirm.
- **Icon:** a warning/hammer icon (e.g. `assets.IconHammer`, matching the existing mandatory warnings).
- **Placement & enforcement:** shown in the Build path **immediately before any engrave** (after assembly + the `(stub,slots)` review, before `bundleEngrave`), and **gating it**: the flow ABORTS the engrave if the warning returns `false` (Back / ConfirmNo). It is **non-deferrable** (cannot be reached past) and **impossible to bypass** — no setting, no skip choice. This is enforced as Invariant I-WARN and is a dedicated test (A6).
- **DECISION (R0 to confirm):** show it ONLY on the Build path (not the unchanged Supply path), since Supply engraves a user-vetted external descriptor verbatim (the existing T6b posture). R0 to confirm scope = Build-only.

---

## Acceptance gate (TDD — tests before impl)

Flow tests mirror the existing GUI test harness (`gui/multisig_test.go`, `gui/bip85_test.go` ConfirmWarningScreen driving, `gui/multisig_testhelpers_test.go` fixtures). All tests authored & RED before impl.

1. **A1 — front-door routing.** The front-door `ChoiceScreen` shows `["Supply policy (md1)","Build policy"]`; selecting Supply runs the UNCHANGED T6b body (assert the existing T6b tests still pass — no regression); selecting Build enters `buildMultisigFlow`; Back returns.
2. **A2 — picker bounds.** Template picker offers exactly the three sortedmulti wrappers mapping to `MultisigWsh/ShWsh/Sh`; n picker = `["2".."5"]`; k picker = `["1".."n"]` for the chosen n; out-of-cap and k>n are structurally unreachable (no free-form widget). DECISION ranges asserted.
3. **A3 — assembled md1 == EncodeMultisig (no GUI re-encode).** Given a fixed (template, k, n, ordered cosigners incl. the self-derived slot, origin mode), the flow's assembled md1 is **byte-identical** to a direct `md.EncodeMultisig(req)` call with the same request, AND `WalletPolicyIDStubChunks(assembled) == returned stub`. Use the abandon-about seed + foreign mk1 fixtures; reproduce the T6b `7b716421…` policy id when fed the equivalent inputs/order (strongest end-to-end faithfulness gate — a device could re-author the exact T6b card via the picker).
4. **A4 — ordering contract + handle shown.** (a) The cosigner order fed to `EncodeMultisig` matches the deterministic on-device rule (§Ordering); a different user-chosen self-slot or gather order yields a different (valid) policy id (mirror Phase-A MANDATE #2). (b) The `(stub, slots)` review screen displays the 4-byte stub and each `@N→fingerprint` before the warning/engrave.
5. **A5 — user-slot derive + placement + reuse-of-leg.** The user's self key is derived via `deriveAccountXpub` at the chosen origin and placed at the chosen `@N`; the downstream `deriveMultisigLeg(...,assembledMd1,full)` produces an mk1 whose `Stubs[0]` equals the assembled stub; full vs watch-only honored.
6. **A6 — EXPERIMENTAL warning is unskippable + gating.** Drive `ConfirmWarningScreen.Layout → ConfirmNo`/Back ⇒ the warning returns false AND **no engrave occurs** (assert `bundleEngrave` not reached / no cards engraved). `ConfirmYes` ⇒ engrave proceeds. Assert the warning is on the Build path's only route to engrave (no bypass).
7. **A7 — security-spine scrub + NFC posture.** The typed mnemonic `[]Word` is zeroed on EVERY Build-path exit (abort at each picker, gather-cancel, warning-abort, engrave-abort, success) via a seed hook (mirror `multisigSeedHook`, `gui/multisig.go:33`); the entropy is wiped (reused `deriveMultisigLeg`); the gather refuses an ms1 secret (`clsMs1Refuse`); mainnet-only asserted.
8. **A8 — restore/verify reuse.** `multisigVerifyFlow` and `multisigRestoreDocFlow` run over the assembled md1 (via `ExpandWalletPolicyChunks`) and produce the sortedmulti restore doc (bip380-expressible path), identical to T6b given the same descriptor.
9. **A9 — no lockstep / no regression.** `go build ./...`, `go vet ./...`, `go test ./...` clean; the program enum/guard/dispatch/title/plate/carousel are byte-unchanged (option (a)); the t5-M1 static guard still compiles.

---

## Invariants (R0 confirms each)

- **I-LOCKSTEP** — Phase B adds NO program; it extends `engraveMultisigFlow`. The enum (`gui/gui.go:147-155`), t5-M1 guard (`:164`), dispatch (`:1502-1527`), titles (`:1680-1693`), `layoutMainPlates` (`:1876-1877`), carousel (`:1653-1664`), npage/npages (`:1867,:1886`) are UNCHANGED.
- **I-ORDER (the centerpiece)** — the GUI assembles `Cosigners[]` in a DETERMINISTIC, user-visible order; `EncodeMultisig` is exactly order-preserving (V2). The on-device rule is fixed (§Ordering) and the resulting `(stub, slots)` handle is SHOWN to the user before engrave. A different order → a different (valid) policy id; only the order matching the user's coordinator binds.
- **I-VERBATIM** — the ONLY md1-bytes producer is `md.EncodeMultisig`; downstream consumers take the assembled strings verbatim (no GUI re-encode).
- **I-STUB** — `WalletPolicyIDStubChunks(assembledMd1)` (used to bind the user's mk1) equals the `stub` returned by `EncodeMultisig`.
- **I-WARN** — the EXPERIMENTAL `ConfirmWarningScreen` is shown on the Build path immediately before engrave, is unskippable/non-deferrable, and ABORTS the engrave on Back/ConfirmNo (A6).
- **I-SCRUB** — typed seed zeroed on every Build-path exit (defer idiom, `gui/multisig.go:71-75`); entropy wiped inside `deriveMultisigLeg`; mainnet-only; ms1 NFC-refused.
- **I-FAITHFUL** — the assembled md1 is byte-identical to a coordinator's for the same shape+keys+order; restore/verify reuse the shipped bip380-expressible (sortedmulti) path.
- **I-FP-PRESENCE** — a gathered cosigner's `FpPresent` is set from `card.Fingerprint != ""` (mk.Decode exposes no explicit bool, V6); the self-derived cosigner's fp-presence is the device's deliberate choice (DECISION: present, set from masterFP — R0 to confirm, since fp-presence changes the policy id but NOT the spending policy).
- **I-SELF-ONE-SLOT** — the user's key occupies exactly one slot (self-as-multiple-slots is OUT).

---

## Risks

1. **(HIGH — the new #1) No-hardware UX validation.** This is GUI-heavy and hardware-UNvalidated: the picker UX, gather sequencing, slot placement, and warning rendering cannot be exercised on real hardware in this cycle. A wrong on-device order/slot, or a mis-rendered warning, mints a valid-but-wrong-binding card onto PERMANENT steel. **Mitigation:** the mandatory unskippable EXPERIMENTAL warning (I-WARN); the `(stub, slots)` review-before-engrave (I-ORDER); exhaustive flow-test coverage of routing/bounds/ordering/warning/scrub (A1–A9); reuse of the already-validated T6b derive→engrave→verify→restore machinery rather than net-new engrave code; the post-implementation mandatory adversarial exec-review (CLAUDE.md phase 4).
2. **(MED) Cosigner-ordering / slot-placement determinism (I-ORDER).** A non-deterministic gather order or a wrong self-`@N` yields a valid-but-different policy id (round-trips locally, fails to match cosigners). **Mitigation:** a fixed documented on-device rule (§Ordering) + the shown stub/slots handle + the warning instructing coordinator verification.
3. **(MED) User-slot mis-derivation.** Wrong origin/network/passphrase → wrong self xpub on steel. **Mitigation:** reuse `deriveAccountXpub` (scrub-ordering handled) at the chosen origin; show the derived self fp/`@N` in the review screen; mainnet-only.
4. **(LOW) fp-presence drift between gathered and self keys (I-FP-PRESENCE).** Mixing fp-present and fp-absent cosigners changes the policy id (not the policy). **Mitigation:** derive each cosigner's `FpPresent` faithfully (gathered: `Fingerprint != ""`; self: device choice); show fps in the review handle.
5. **(LOW) Encoder/wire** — fully de-risked (Phase A shipped + byte-proven); Phase B does no wire work.
6. **(LOW) Lockstep drift** — option (a) touches no lockstep site; the t5-M1 compile-time guard self-flags any accidental enum change.
7. **(LOW) Template/k-n scope creep** — keep to the three sortedmulti wrappers + the n∈2..5 cap; everything else is OUT/refused.

---

## §Ordering — the deterministic on-device cosigner-order rule (DECISION; R0 centerpiece)

The device is the AUTHORITATIVE creator (no coordinator to match), so the order is the device's to FIX and SHOW. **Proposed rule (R0 to confirm):**
1. Pick template, n, k.
2. Pick the user's own slot index `@S` (bounded `ChoiceScreen` `["@0".."@{n-1}"]`).
3. Gather the other N−1 cosigner mk1 cards in sequence; they fill the remaining slots **in ascending slot order, in gather order** (i.e. the i-th gathered card fills the i-th remaining slot `@0..@{n-1}` skipping `@S`).
4. The user's self-derived key fills `@S`.
5. Build `Cosigners[]` indexed `@0..@{n-1}` accordingly and call `EncodeMultisig` (order-preserving, V2).
6. Show the `(stub, slots)` handle; require the EXPERIMENTAL warning; engrave.

**Alternative (R0 may prefer):** self always at `@0`, others in gather order at `@1..@{n-1}` (simpler, no self-slot picker) — at the cost of less flexibility matching a coordinator that expects a specific slot for the user. The spec recommends the user-chosen-`@S` rule for coordinator-matching flexibility but flags this as the single biggest open UX question for R0.

---

## Ambiguities for R0 (explicit open questions)

1. **Template scope:** all three sortedmulti wrappers, or wsh-only for v1? (Spec recommends all three; wsh-only is a safe simpler alt.)
2. **n cap:** 2..5 (recommended) or 2..9? (Tunable; not correctness-load-bearing.)
3. **Ordering rule:** user-chosen `@S` (recommended) vs self-always-`@0`? (The #1 UX decision; both are byte-faithful given the chosen order.)
4. **Origin mode:** offer only `OriginShared` (single `m/48'/0'/0'/2'`, matches T6b/BIP-48 P2WSH) for v1, or also expose `OriginDivergent`? (Spec recommends Shared-only for v1; divergent is encoder-supported but adds a per-cosigner origin-picker UX with no bounded widget for arbitrary paths — likely OUT for v1. R0 to rule.)
5. **Self fp-presence:** emit the user's fp (recommended, faithful) — confirm it matches the coordinator expectation, since it changes the policy id.
6. **Self-key consistency:** must the self-derived xpub's origin equal the chosen shared origin? (Yes — derive `deriveAccountXpub` at the same origin the policy declares; flag if R0 wants an explicit guard.)

---

## Gate

This is a brainstorm SPEC — NOT a plan, NOT code. It MUST pass an opus architect **R0 review to 0 Critical / 0 Important** before any IMPLEMENTATION_PLAN or code. Fold findings → persist the review verbatim to `design/agent-reports/` → re-dispatch after every fold until GREEN. R0 is expected to center on: the **ordering rule (I-ORDER, §Ordering)**, the **unskippable EXPERIMENTAL warning (I-WARN)**, the choose-or-supply front-door's zero-lockstep claim (I-LOCKSTEP), and the open questions above. Per CLAUDE.md: a single subagent then executes the GREEN plan (TDD, worktree); a mandatory independent adversarial exec-review over the whole diff follows (non-deferrable — especially load-bearing here given the no-hardware-validation risk).
