# R0 architect review — SPEC_seedhammer_template_engrave (round 2)

- **Artifact:** `design/SPEC_seedhammer_template_engrave.md` (DRAFT v3.1)
- **Fork SHA:** `/scratch/code/shibboleth/seedhammer` @ `39cb5cf` (verified `git rev-parse HEAD` = `39cb5cf6351f6c87ab779e5688a60186f65412e0`)
- **Rust pins:** descriptor-mnemonic/md-codec @ `54dd765`, mnemonic-key @ `1279ef9`, mnemonic-toolkit @ `6de53879`
- **Date:** 2026-06-20
- **Reviewer role:** adversarial OPUS architect, **round 2** of the mandatory pre-implementation R0 gate (must reach 0C/0I).
- **Prior rounds:** round 0 = 3C/5I (all folded, verified resolved in round 1); round 1 = 0C/1I (the new **D1** I2-fold drift) + ratified the rest.
- **Scope of THIS round (per the gate brief):** confirm **D1 is correctly resolved**, confirm the D1 fold introduced **no new drift**, and give a final holistic 0C/0I check. This is a FOCUSED round — the security binding, strip fidelity, four mint sites, and estimate model were validated in rounds 0+1 and are NOT re-litigated absent concrete regression evidence. M1 (µs provenance) and M3 (inline §5 fixture) are accepted as plan-time deferrals, not blockers.
- **Method:** every D1 sub-claim re-verified against the fork source text (NOT the SPEC's self-description), reading the full supply flow, the gate, the cross-match, the display status, and — critically — the *actual* verbatim re-engrave entry points the D1 fix now leans on.

This review is BLOCKING.

---

## Summary verdict

**GREEN — 0 Critical / 0 Important.**

D1 is **RESOLVED**. The four D1 sub-claims all check out against `39cb5cf`:

1. `supplyMultisigPolicyFlow` is inherently full-policy-only and keys off **xpub-matching** — confirmed; it cannot serve a keyless template, `allSlotsHaveXpub` is left unweakened, and the contradictory "route around the gate via `expandTemplateOnly`" sentence from v3 is **gone**.
2. A verbatim multi-card re-engrave path that takes md1 + N mk1 cards and engraves them as-is **does exist** in the fork — the `engraveBundle` program (`bundleFlow`/`bundleEngrave`). So a supplied keyless template bundle has a real home; the SPEC is not hand-waving a non-existent flow.
3. `expandTemplateOnly` is genuinely **display-only** (an `expandStatus` enum value returned by `expandedToDescriptor` to the inspect/gather pipeline) — not a derive-leg.
4. The removed routing sentence left **no dangling reference** to the old (wrong) mechanism elsewhere in the SPEC.

The D1 fold introduced **no new Critical/Important**. The round-0 and round-1 resolved items spot-check clean (no regression). The single imprecision I found — the SPEC names `mdmkFlow` (the *single-card* engraver) as the home for a *multi-card* supplied template bundle, when the actual multi-card verbatim path is `bundleFlow`/`bundleEngrave` — is a **Nit** (carried/sharpened from round-1 N1), non-blocking, and should be tightened in the implementation plan. It does not reopen D1 because the *substance* of the D1 fix (supply-via-seed-cross-match stays full-policy-only and untouched; a pre-made template bundle is re-engraved verbatim + form-aware-verified) is correct and a real verbatim path exists.

---

## D1 — resolution status: **RESOLVED**

The round-1 blocker D1 was: S3/S4 simultaneously claimed the multisig SUPPLY path stays "verbatim / unchanged" AND "route templates around `allSlotsHaveXpub` via `expandTemplateOnly`" — the latter infeasible (the supply flow cross-matches a typed seed to a slot by xpub, which a keyless template cannot satisfy) and a miscite of a display-only status. Verifying the v3.1 fold against source:

### Claim 1 — `supplyMultisigPolicyFlow`/`findUserSlot` key off xpub-matching ⇒ inherently full-policy-only. **CONFIRMED.**
- The supply flow gathers a supplied md1, decodes it, then **hard-gates on `allSlotsHaveXpub(keys)`** before any seed is typed: `gui/multisig.go:83-86` (`if !allSlotsHaveXpub(keys) { showError("…has no public keys to match.") ; return }`).
- `allSlotsHaveXpub` (`gui/multisig_supply.go:72-82`) returns false for an empty key set OR any slot with `!k.XpubPresent` — its doc-comment is explicit: *"the supplied md1 must be a FULL wallet policy — every expanded slot must carry an xpub, else there is no public key to cross-match the typed seed against. A template-only md1 (no pubkeys) … refuses."*
- The next step, `findUserSlot` (`gui/multisig_match.go:34-60`), derives the operator's account key from the typed seed at each slot's origin and **matches on the canonical `(chainCode, compressedPubkey)` pair** of the slot's embedded xpub: `bytes.Equal(cc[:], k.Xpub[0:32]) && bytes.Equal(pk[:], k.Xpub[32:65])` (`:48`). A keyless template has no `k.Xpub` to match → zero matches → `false` → the flow refuses (`gui/multisig.go:117-119`). So even setting the gate aside, the flow dead-ends — exactly as round 1 reasoned.
- **Verdict:** the seed-cross-match flow is structurally full-policy-only. The SPEC (S3 line 55, S4 line 58) now states this correctly and leaves the flow + `allSlotsHaveXpub` UNCHANGED. The security-relevant gate is not weakened.

### Claim 2 — a SUPPLIED template bundle is engraved through an EXISTING verbatim re-engrave path (real home, not hand-wave). **CONFIRMED (with a Nit on the named function).**
- A verbatim **multi-card** bundle engraver exists: the `engraveBundle` program → `bundleFlow` (`gui/bundle_flow.go:24`) → `bundleGatherFlow` (gather md1 + N mk1 cards over NFC) → `bundleReviewFlow` → `bundleEngrave` (`gui/bundle_flow.go:327`). `bundleEngrave` lays out each gathered card's string via `validateMdmk` and engraves the plates **verbatim** (the QR/text payload is the card string itself; `validateMdmk` `gui/gui.go:1930` engraves the string as-is). It explicitly handles N mk1 + md1 ("md1 descriptors: %d / mk1 keys: %d" tally, `:83-86`) and refuses ms1 over NFC. This is a genuine home for re-engraving a pre-made keyless template bundle (template md1 + N keyless cosigner mk1 stubs) with NO re-encode and NO xpub gate.
- **Nit (not blocking):** the SPEC names **`mdmkFlow`** (S3 line 55, S4 line 58, Scope line 133) as the verbatim path. `mdmkFlow` (`gui/gui.go:1972`) is the **single-card** engraver — it takes one `mdmkText` string (one md1 OR one mk1), offers inspect + plate-variant engrave for that ONE card (`:2023` engraves `engravings[idx]`), and loops on a single card. It does NOT iterate a multi-card bundle. The correct multi-card verbatim engraver is `bundleFlow`/`bundleEngrave`. The SPEC's *substance* is right (an existing verbatim re-engrave path takes the supplied template bundle as-is); only the function name is imprecise. The plan should cite `bundleFlow`/`bundleEngrave` (or `mdmkFlow` only for a single-card md1). See N1 below — this is the same ambiguity round-0/round-1 N1 flagged, now made concrete.
- **Form-aware VERIFY binding nuance (important to state precisely, but NOT a defect):** the SPEC says the supplied template is "bound by the form-aware VERIFY." Note that `bundleFlow`/`bundleEngrave` itself does **not** call `bundle.Verify`/`checkStubBinding` — it is a pure gather→review→verbatim-engrave path. The form-aware stub binding (`bundle.Verify` → `checkStubBinding` at `bundle/verify.go:116`, which the C2 rewire makes form-aware) is exercised by the *derive*-flow readback verifies (`gui/multisig_verify.go:39`, `gui/singlesig_verify.go`) and by the off-device toolkit `verify-bundle`. For a supplied pre-made template bundle the binding that matters is: the supplied mk1 cards already carry a `WalletDescriptorTemplateId` stub (minted off-device by the toolkit), and ANY verify of that bundle (the device's `verify-bundle` offer, or off-device `restore`/`verify-bundle`) recomputes via the form-aware `checkStubBinding` and binds correctly once `verify.go:116` is rewired (C2). The SPEC's phrasing "bound by the form-aware VERIFY (S3)" is therefore accurate at the binding layer; it is just engraved (not necessarily re-verified on the same screen) by the verbatim path. This is consistent and not a contradiction — but the plan should make the engrave-vs-verify split explicit so an implementer doesn't expect `bundleEngrave` to itself perform the stub check.

### Claim 3 — `expandTemplateOnly` is display-only, not a derive bypass. **CONFIRMED.**
- `expandTemplateOnly` is an `expandStatus` enum value (`gui/md1_expand.go:13-18`): *"the md1 carries no xpubs (D3) — show the template read-only; no descriptor, no address-verify."* It is returned by `expandedToDescriptor` (`gui/md1_expand.go:42-48`: `if len(keys)==0 || any !XpubPresent → return nil, expandTemplateOnly`) and consumed only by the inspect/gather DISPLAY pipeline (`gatheredDescriptorFlow` `gui/md1_gather.go:211-212` → `md1DisplayFlow`; and `mdmkFlow`'s inspect arm). It appears **nowhere** in `supplyMultisigPolicyFlow` or any derive leg. The SPEC (S3 last sentence) now correctly frames it as "the DISPLAY step in that verbatim path, NOT a derive-leg bypass." Correct.

### Claim 4 — no dangling reference to the old (wrong) mechanism. **CONFIRMED.**
- I grepped the full SPEC for the v3 routing language. The phrase *"route templates around"* / *"Do NOT widen … `allSlotsHaveXpub`"* is **gone**. Every remaining mention of `allSlotsHaveXpub` (S3 line 55) now says the gate is "NOT weakened" / the flow is "left UNCHANGED." `expandTemplateOnly` is mentioned only as display-only (S3 line 55) and as a reuse target in Scope (line 133). DD1 (line 20), S3 (line 55), S4 (line 58), Invariants, Acceptance, and the Gate's round-1 entry (line 149) all tell one consistent story: strip applies to DEVICE-BUILT (single-sig + multisig-BUILD) only; the seed-cross-match supply flow is full-policy-only and untouched; a pre-made supplied template bundle is re-engraved verbatim + bound by form-aware verify. No S3/S4/Scope/Invariants/Acceptance residue implies the old mechanism. Internally consistent.

**D1 sub-verdict:** RESOLVED. The fold took round-1 fix-option (1) (supply-via-seed-cross-match is full-policy-only; pre-made template bundle re-engraved via the existing verbatim path), removed the infeasible routing claim and the `expandTemplateOnly`-as-routing miscite, and left no contradiction.

---

## New findings introduced by the D1 fold

### Critical — none.
### Important — none.

### Nit

#### N1 (carried/sharpened from round-0 + round-1 N1) — the verbatim multi-card path is `bundleFlow`/`bundleEngrave`, not `mdmkFlow`
- **Location:** S3 line 55 ("the EXISTING verbatim scan/`mdmkFlow` re-engrave path"), S4 line 58 (same), Scope line 133 ("reuse … the verbatim engrave `mdmkFlow`").
- **Problem:** `mdmkFlow` (`gui/gui.go:1972`) is the **single-card** md1/mk1 verbatim engraver (one `mdmkText`, inspect + plate-variant pick + engrave that one card; `:1972-2027`). A SUPPLIED multisig **template bundle** is md1 + **N** keyless cosigner mk1 cards — a multi-card set. The fork's existing multi-card verbatim engraver is the `engraveBundle` program: `bundleFlow` → `bundleGatherFlow` → `bundleEngrave` (`gui/bundle_flow.go:24,95,327`), which engraves every gathered card's string verbatim via `validateMdmk`. Citing `mdmkFlow` for the multi-card supplied template is imprecise (the substance — "an existing verbatim re-engrave path" — is correct; only the named function is wrong for N>1 cards).
- **Evidence:** `gui/gui.go:1972-2027` (`mdmkFlow` = single-card); `gui/bundle_flow.go:24-39` (`bundleFlow` loop), `:327-367` (`bundleEngrave` iterates the gathered card plan, engraving each verbatim); `:83-86` (md1/mk1 tally — multi-card by construction).
- **Severity / why Nit not Important:** does not reopen D1 — a real verbatim multi-card path exists and the security story is unchanged. It is a precision defect in the named entry point. Round-1 N1 already foresaw this ("if fix-1 of D1 is taken, the *bundle* re-engrave path becomes the legitimate home — clarify that role"). 
- **Fix (plan-time):** in S3/S4/Scope, replace "`mdmkFlow`" with "the `engraveBundle` verbatim bundle path (`bundleFlow`/`bundleEngrave`)" for the multi-card supplied template, reserving `mdmkFlow` for a single scanned md1. Also state (plan-time) that the form-aware stub binding for a supplied template is exercised at *verify* (`bundle.Verify`/`checkStubBinding` `verify.go:116`, made form-aware by C2) — `bundleEngrave` itself engraves verbatim without re-checking the stub — so the engrave-vs-verify split is unambiguous.

---

## Regression spot-check of round-0/round-1 resolved items (no re-derivation; evidence only where touched)

- **C1 (conditional origin elision):** untouched by the D1 fold; DD1/S1/Invariant 2/Acceptance still state the `canonical_origin(tree).is_some()` conditional. No regression.
- **C2 (four-site form-aware mint; complete enumeration):** untouched; the four sites (`gui/singlesig_derive.go:67`, `gui/multisig_derive.go:42`, `md/encode_multisig.go:158`, `bundle/verify.go:116`) are still the set, and I re-confirmed `verify.go:116` is the lone verify-side `WalletPolicyIDStubChunks` call inside `checkStubBinding` (`bundle/verify.go:114-124`). The supply flow's `deriveMultisigLeg` (which mints at `multisig_derive.go:42`) is unrelated to the D1 supply-template story (it serves the seed-cross-match full-policy leg) — D1 correctly does not touch it. No regression.
- **C3 / DD7 (honest-minimal consent):** untouched; `d.n`-sourced "Key slots: N" for PolicyComplex still holds (independent of `classifyPolicy`'s 0,0 verdict). No regression.
- **I1 (non-empty predicate), I3 (kiw-from-n + local guard), I4 (no-canonicalize guarantor named):** S2 text unchanged by the D1 fold. No regression.
- **I5 (four-site default regression pin):** Risk 5 / Invariant 3 / Acceptance unchanged. No regression.
- **O4 (recovery-time model):** S6 + the GUI table unchanged; still arithmetically consistent (6.9 µs/N!). No regression. (M2 row-harmonization remains a non-blocking plan-time cleanup, as in round 1.)
- **M1 (µs provenance) / M3 (inline §5 fixture):** accepted plan-time deferrals per the gate brief; not blockers.

No resolved item regressed; the D1 fold's edits are localized to S3/S4 (and the Gate's round-1 entry note), exactly where the contradiction lived.

---

## Ratifications carried forward (verified across rounds; do not re-litigate)
- WDT-Id no-canonicalize pin (#1) + `pathDecl.n`/kiw local guard (#2): re-confirmed sound in rounds 0+1.
- Byte-faithful `writeNode` tree serialization; WDT-Id ‖ WalletPolicyId preimage non-collision: re-confirmed in rounds 0+1.
- Four-site mint completeness (C2): re-confirmed complete in round 1; the lone verify-side recompute (`verify.go:116`) re-checked this round.
- `d.n` availability for PolicyComplex (C3): re-confirmed round 1.

---

## VERDICT

**GREEN — 0 Critical / 0 Important.**

- **D1: RESOLVED** — all four sub-claims verified against `39cb5cf`: `supplyMultisigPolicyFlow`/`findUserSlot` are xpub-cross-match (full-policy-only, `gui/multisig.go:83-86`, `gui/multisig_match.go:48`); `allSlotsHaveXpub` unweakened (`gui/multisig_supply.go:72-82`); a real verbatim multi-card re-engrave path exists (`bundleFlow`/`bundleEngrave`, `gui/bundle_flow.go:24,327`); `expandTemplateOnly` is display-only (`gui/md1_expand.go:13-18`); the contradictory v3 routing sentence is gone with no dangling reference.
- **No new Critical or Important** introduced by the fold; the round-0/round-1 resolved set does not regress.
- **One Nit (N1):** S3/S4/Scope name `mdmkFlow` (single-card) where the multi-card supplied-template home is `bundleFlow`/`bundleEngrave`; substance is correct, only the function name is imprecise. Tighten in the implementation plan along with the deferred M1/M2/M3 cleanups. Non-blocking.

The gate is satisfied. Proceed to the implementation plan (single-author + the R0 plan-gate), folding N1/M1/M2/M3 into the plan.
