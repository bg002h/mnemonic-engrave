# T6c Phase B SPEC — R0 review (round 0) — VERBATIM agent report

**Agent:** `a71fac556ac5e728c` (adversarial opus architect; 3 parallel citation sweeps + first-hand reads at fork HEAD `f323dd2`). **Spec commit:** `0d0424f`. **Date:** 2026-06-20.
**Verdict:** GREEN (0C/0I) — 4 Minor; all 6 ambiguities defaulted (2 flagged genuine product calls, defaulted so the cycle is unblocked, user may override without reopening R0). Persisted per the gate discipline.

---

# T6c Phase B SPEC — R0 review (round 0)
**Reviewer:** opus architect (adversarial)  **Fork HEAD:** f323dd2  **Spec commit:** 0d0424f  **Verdict:** GREEN (0C / 0I) — 4 Minor; all 6 ambiguities defaulted (3 are genuine product calls, defaulted so the cycle is unblocked, user may override)

## Front-door / lockstep verification (MANDATE #1)
CONFIRMED lockstep-free. `engraveMultisig` is a single enum constant (`gui/gui.go:152`, iota 4); `bip85Derive` (`:153`) is the last-navigable boundary every lockstep site keys off: t5-M1 guard `:164`, dispatch `:1502-1527` (`case engraveMultisig:` → `engraveMultisigFlow` `:1515-1516`), title `:1689-1690`, `layoutMainPlates` `:1877`, carousel wrap on `m.prog > bip85Derive` `:1662`, `npage`/`npages = int(bip85Derive)+1` `:1867`/`:1886`. Adding a front-door `ChoiceScreen` at the top of `engraveMultisigFlow` (before `bundleGatherFlow` `gui/multisig.go:38`) is pure internal body editing. Precedent: `engraveMultisigFlow` already has three internal `ChoiceScreen`s (`:79`,`:98-102`,`:122`) with zero lockstep edits. ZERO lockstep edits / no guard churn / no new program.

## Reuse + binding (MANDATE #2)
CONFIRMED real + source-agnostic (all reuse fns take md1 chunks VERBATIM): `deriveMultisigLeg(...)` (`gui/multisig_derive.go:32`, md1 cloned, `Stubs=[WalletPolicyIDStubChunks(suppliedMd1)]`, `m.Valid()`→`Entropy()`/`wipeBytes`); `multisigEngraveCards` (`gui/multisig_engrave.go:11`); `multisigVerifyFlow` (`gui/multisig_verify.go:36`); `multisigRestoreDocFlow` (`gui/multisig_restore.go:58`, faithful-or-refuse `:8-32`); `bundleEngrave` (`gui/bundle_flow.go:327`). **`md.EncodeMultisig` is the ONLY md1-bytes producer** (struct-ctor `EncodeMultisig(req EncodeMultisigRequest) (out []string, stub [4]byte, slots []SlotInfo, err error)` `encode_multisig.go:88`); no GUI re-encode. Gather: `bundleGatherFlow` (`gui/bundle_flow.go:95`)→`mk.Decode` (`mk/mk.go:148`)→`decodeXpubBytes` (`gui/singlesig_derive.go:99`); self via `seedEntryFlow` (`gui/derive_xpub.go:82`)→`deriveAccountXpub` (`gui/derive.go:19`). Ordering: `EncodeMultisig` order-preserving (Phase A: 3 orders→3 ids, no key sort); `(stub, slots)` genuinely returned (`:123-133`,`:157`), spec mandates showing it pre-engrave. **I-STUB exec-proven** (assembled `suppliedMd1==out` → `deriveMultisigLeg` stub == `EncodeMultisig` stub by Phase A round-trip fidelity).

## Warning + security spine (MANDATE #3)
CONFIRMED. Unskippable warning required (IN §7, I-WARN, test A6); the mirrored idiom (`childSeedWarning` `gui/bip85.go:145`, `stubZeroWarning` `gui/derive_xpub.go:237`) returns `true` ONLY on held `ConfirmYes`, aborts protected engrave on `false`; no skip/setting path. Security spine intact: ms1 NFC-refused at classify (`clsMs1Refuse` `gui/bundle.go:46`,`:66-71`, never appended); seed typed-only; `deriveAccountXpub` neuters + serializes-before-`Zero()` (`derive.go:50-51`) — NO xprv serialized; per-leg entropy gated+wiped; assembled md1 public; mainnet-only.

## Ambiguity adjudication (the 6)
1. **Templates — DEFAULT: offer all three (`wsh`/`sh(wsh)`/`sh` sortedmulti), highlight `wsh`.** `expandedToDescriptor` (`gui/md1_expand.go:104-115`) restore-handles all three; Phase A proved all byte-correct. *(Architect-defaultable.)*
2. **n cap — DEFAULT: n∈2..5, k∈1..n.** Covers 2-of-3/3-of-5/2-of-2; each cosigner = a gather + a plate; encoder guards n≤32 regardless. *(Architect-defaultable; widen later.)*
3. **Ordering rule (the #1) — DEFAULT: self-always-`@0`, others in gather order @1..@{n-1}.** Simpler/safer (removes the `@S` picker + a self-misplacement class); byte-faithful (device authoritative, EXPORTS the order). **FLAGGED genuine PRODUCT decision** — alternative is user-chosen `@S` (coordinator-slot-matching flexibility). Defaulted to self-`@0`; user may override.
4. **Origin mode — DEFAULT: `OriginShared` only (v1).** BIP-48 norm; `pathPickerFlow` (`gui/derive_xpub.go:48`) maps to a single shared origin; `OriginDivergent` has no bounded widget (no free-form path entry) → out of scope, clean FOLLOWUP. *(Architect-defaultable.)*
5. **Self fp-presence — DEFAULT: emit self fp (FpPresent=true, from masterFP).** **FLAGGED soft PRODUCT call** — fp-presence changes WalletPolicyId (Phase A: absent `7b716421…` vs present `639cabcf…`). CAVEAT: gathered cosigners may be fp-ABSENT (T6b fixture is), so self-present + others-absent = a MIXED policy whose id matches neither homogeneous coordinator. Plan must surface per-slot fp in the slots review. Defaulted present; user may prefer "match cosigners' presence."
6. **Self-origin==policy-origin guard — DEFAULT: YES (derive self at the policy's shared origin; no separate self-origin picker).** Free in Shared mode; removes a mis-derivation class. *(Architect-defaultable; coupled to #4=Shared-only.)*
Net: all 6 defaulted, cycle unblocked. #3 + #5 are genuine product calls (defaulted, overridable without reopening R0).

## Critical
None.
## Important
None.
## Minor
- **M1 (mixed fp-presence — fold into plan).** With #5=present + fp-absent gathered cosigners, the Build flow can mint a mixed-presence policy whose id matches neither homogeneous coordinator. The slots review (I-ORDER) shows per-slot fp; plan should assert A4/A5 reflect mixed presence + warning copy should mention fp-presence affects the id.
- **M2 (citation drift §V5).** `deriveMultisigLeg`: `Stubs` set at `gui/multisig_derive.go:51`; `m.Valid()` gate at `:33` (not `:64`); `Entropy()`/`wipeBytes` at `:64-66`. No semantic impact.
- **M3 (Phase A inherited).** `OriginMode` `default:` in `encode_multisig.go:110` returns `errMultisigBadScript` (mislabel; unreachable). Harmless to Phase B (picker passes only OriginShared); fix if divergent later added. (= the logged `seedhammer-t6c-phaseA-originmode-errmsg`.)
- **M4 (A3 fixture-replay precision).** Test A3 ("reproduce `7b716421…` via the picker") must drive self-fp-ABSENT + all cosigners absent to byte-match the fp-absent T6b fixture, OR assert a different (correct) id for the default-present config. State explicitly so A3 isn't written to an impossible target.

## Verified-correct
- Front-door lockstep-free (engraveMultisigFlow already branches internally 3× with zero lockstep edits).
- `md.EncodeMultisig` shipped as struct-ctor matching the spec; sole md1 producer; `(stub,slots)` returned.
- 4 reuse seams descriptor-source-agnostic (verbatim md1, no re-encode); restore faithful-or-refuse covers all 3 wrappers via `InnerWsh`.
- I-STUB exec-proven; `WalletPolicyIDStubChunks` pure fn of the strings.
- Unskippable-warning idiom non-bypassable; ms1 NFC-refused; seed typed-only; no xprv; mainnet-only; `mk.Card.Fingerprint` string (`""`=absent).
- `ChoiceScreen` is the only bounded picker (no free-form numeric/path widget — `bip85.go:108-109`), forcing template/n/k/(slot) bounded.
- `pathPickerFlow` resolves BIP-48 `m/48'/0'/0'/2'` (basis for Shared default).
- TDD A1–A9 cover front-door routing, picker bounds, gather→assemble→`EncodeMultisig` byte-equality, `(stub,slots)` display, unskippable warning (A6), engrave/verify/restore reuse, security-spine scrub/NFC (A7). IN/OUT coherent (miniscript/self-multi-slot/>cap/free-form-index/testnet/coordinator-round-trip deferred).

## Bottom line
**GREEN (0C/0I).** Front-door verified lockstep-free first-hand; reuse real + source-agnostic with `md.EncodeMultisig` the sole producer and I-STUB exec-proven; security spine intact; the mandatory EXPERIMENTAL warning maps to a proven non-bypassable idiom. All 6 ambiguities defaulted with sound byte-faithful choices — **#3 (self-`@0`) and #5 (self-fp present) flagged as genuine product decisions, defaulted so the cycle is unblocked; user may override either without reopening R0.** 4 Minors fold into the IMPLEMENTATION_PLAN (notably M4: A3 must drive self-fp-ABSENT to byte-match `7b716421…`). Cleared to proceed to the plan-doc (single-author + its own R0), then single-subagent TDD, then the mandatory adversarial exec-review (load-bearing given the HIGH no-hardware-validation risk).
