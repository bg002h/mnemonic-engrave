# R0 architect review — SPEC_seedhammer_template_engrave (round 1)

- **Artifact:** `design/SPEC_seedhammer_template_engrave.md` (DRAFT v3)
- **Fork SHA:** `/scratch/code/shibboleth/seedhammer` @ `39cb5cf` (verified `git rev-parse HEAD` = `39cb5cf6351f6c87ab779e5688a60186f65412e0`)
- **Rust pins:** descriptor-mnemonic/md-codec @ `54dd765`, mnemonic-key @ `1279ef9`, mnemonic-toolkit @ `6de53879`
- **Date:** 2026-06-20
- **Reviewer role:** adversarial OPUS architect, round 1 of the mandatory pre-implementation R0 gate (must reach 0C/0I).
- **Prior round:** round 0 (`seedhammer-template-engrave-spec-R0-round0.md`) found 3C/5I; author folded all 8 into DRAFT v3.
- **Method:** every round-0 fold re-verified against authoritative source text (Go fork + Rust crates), NOT the SPEC's self-description. Five parallel primary-source verification agents + direct reads. Specifically re-derived: the four stub-mint sites (completeness), the toolkit's conditional origin elision, the Rust identity/predicate ports, `classifyPolicy`/`d.n` availability, the supply-flow seed-cross-match, and the N! recovery-time arithmetic.

This review is BLOCKING.

---

## Summary verdict

**NOT GREEN — 0 Critical / 1 Important.**

Seven of the eight round-0 findings are **fully and correctly resolved**, verified against source. The cryptographic core, the four-site completeness (the heaviest C2 verification), the conditional origin elision (C1), and the honest-minimal consent surface (C3) all check out byte-for-byte against the fork and toolkit. **O4 is RATIFIED** — the recovery-time model is honest and arithmetically correct.

The single blocker is a **drift introduced by the I2 fold**: S3's last sentence and S4's supply-path prose left a self-contradiction. The SPEC simultaneously says the multisig SUPPLY path "neither strips nor re-mints … engraved verbatim" (correct) AND "route templates around [`allSlotsHaveXpub`] via `expandTemplateOnly`" (incorrect — that gate guards a seed-to-slot cross-match that is impossible without xpubs, and `expandTemplateOnly` is a display-pipeline status, not a derive-leg path). This is precisely the residual round-0 I2 asked to be replaced "with a concrete flow"; the fold corrected the *placement* but retained the contradictory *routing* sentence. One focused edit closes it.

---

## Round-0 findings — resolution status

- **C1 (conditional origin elision)** — **RESOLVED.** DD1/S1/Invariant 2/Risk 1/O2 now all state the elision is conditional on `canonical_origin(tree).is_some()`, keeping source origins otherwise. Verified faithful to toolkit `synthesize.rs:1195` (`if md_codec::canonical_origin::canonical_origin(&descriptor.tree).is_some() { template.path_decl.paths = PathDeclPaths::Shared(OriginPath{components: vec![]}) }`; else leaves cloned source origins; pubkeys/fingerprints set `None` unconditionally at `:1182-1183`). `canonical_origin` (`md-codec/src/canonical_origin.rs:42-79`) returns `None` for `tr(@N,TapTree)`, general miniscript, and `sh(sortedmulti)` legacy — exactly the no-canonical shapes. The fork's `validateExplicitOriginRequired` (`md/md.go:1030-1065`) would reject an elided-origin no-canonical wire with `errMissingExplicitOrigin`, confirming the "decode-rejected" consequence the SPEC names. A `canonical_origin==None` golden vector is specified (Acceptance "Strip golden", line 122, and Invariant 2).

- **C2 (form-aware mint at ALL sites, complete enumeration)** — **RESOLVED.** DD4/S2/S3 route all four sites through one form-aware helper. **The four-site list is COMPLETE** (the most important verification): an exhaustive grep of the fork for `WalletPolicyIDStub*`/`WalletPolicyId`/`StubChunks`/`policyIDStub` across all non-test `.go` finds exactly four mint/recompute sites — `gui/singlesig_derive.go:67` (`WalletPolicyIDStubChunks(md1)`, engraved into `mk1.Stubs`), `gui/multisig_derive.go:42` (`WalletPolicyIDStubChunks(suppliedMd1)`, engraved), `md/encode_multisig.go:158` (`WalletPolicyIDStub(d)`, returned for ordering verify + engraved), `bundle/verify.go:116` (`WalletPolicyIDStubChunks(b.MD1)`, readback recompute). No mint in BIP85/qaProgram/scan/`mdmkFlow` or anywhere in `bundle/` beyond verify. The `multisig_build.go:115 → deriveMultisigLeg (multisig_derive.go:42)` call-graph claim is confirmed (build does not mint directly). The reference impl `derive_stub_from_md1` (`mk-cli/src/cmd/mod.rs:72-82`) is confirmed form-aware (`if descriptor.is_wallet_policy() { compute_wallet_policy_id } else { compute_wallet_descriptor_template_id }`).

- **C3 (honest-minimal consent for PolicyComplex)** — **RESOLVED.** DD7/S4 now specify the `{script family, key-slot count N (= d.n), template-id}` consent surface for `classifyPolicy → PolicyComplex,0,0` shapes and explicitly state VERIFY does NOT degrade (template-id binding exact for any decodable shape). Verified: `classifyPolicy` (`md/md.go:1266-1316`) returns `PolicyComplex,0,0` for any `tr` with a script tree / general shape, but `d.n` is a top-level `descriptor` field (`md/md.go:817`, set `n: pd.n` at decode `md/md.go:858`) surfaced as `Template.N` (`summarize`, `md/md.go:1380`) **independently of** the `classifyPolicy` k/N verdict — so "Key slots: N" is truthfully computable for PolicyComplex. Breadth retained (engrave+verify any admissible shape). The deferred FOLLOWUP `seedhammer-template-engrave-policy-summary-display` is filed (`design/FOLLOWUPS.md:29`).

- **I1 (`is_wallet_policy` = present AND non-empty)** — **RESOLVED.** S2/DD4/Predicate-test now say `d.tlv.pubPresent && len(d.tlv.pubkeys) > 0`. Verified vs Rust `encode.rs:50-52`: `matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty())`. The `errEmptyTLVEncode` guard the SPEC cites is real (`md/encode.go:292-295`: `if s.pubPresent { if len(s.pubkeys)==0 { return errEmptyTLVEncode } }`). The Predicate acceptance test (line 124) asserts a stripped descriptor that left `pubPresent` set with empty `pubkeys` does not slip through.

- **I2 (supply-path verbatim; placement)** — **PARTIALLY RESOLVED.** The *placement* fold is correct and verified (strip lives on single-sig + multisig-BUILD; supply stays verbatim — `deriveMultisigLeg` documents I-2 "the device never re-encodes a multisig descriptor" and clones `suppliedMd1` verbatim, `multisig_derive.go:20,58-60`; BUILD re-encodes via `EncodeMultisig` behind the mandatory `multisigBuildExperimentalWarning`, `multisig_build.go:88,99-102,145`). **BUT** the fold left a residual self-contradiction — see **D1** below. This is the still-open half round-0 I2 flagged ("replace the hand-wave with a concrete flow"). Blocking.

- **I3 (kiw from `n`; `pathDecl.n==n` guard inside WDT-Id)** — **RESOLVED.** S2 R0-pin #2 now says WDT-Id "computes kiw from `d.n`" and "must carry the `pathDecl.n == d.n` guard **inside** the WDT-Id function" because WDT-Id bypasses `encodePayload`. Verified necessary: the existing `WalletPolicyId` (`md/walletpolicyid.go:32,37`) does `dc, _ := canonicalize(d)` then `width := kiw(dc.pathDecl.n)`; the lockstep guard lives in `encodePayload` (`md/encode.go:401`, `if dc.pathDecl.n != dc.n { return errPathDeclNMismatch }`) which WDT-Id does not call. Rust `compute_wallet_descriptor_template_id` uses `d.key_index_width()` off `d.n` (`identity.rs:76`; `key_index_width` off `self.n`, `encode.rs:37-41`). The I3 acceptance test (feed `pathDecl.n != n`, assert the guard fires) is specified.

- **I4 (name the no-canonicalize guarantor)** — **RESOLVED.** S2 R0-pin #1 now names the decode-side invariant (`validatePlaceholderUsage` + decode canonical form) as the guarantor that "as-decoded == canonical," and warns a future author-built AST must be validated before hashing. Verified: Rust `validate_placeholder_usage` (`validate.rs:11-37`) enforces "first occurrences in canonical ascending order" (`@0` before `@1` …); `compute_wallet_descriptor_template_id` (`identity.rs:71-104`) has NO `canonicalize` call while `compute_wallet_policy_id` (`identity.rs:172-177`) DOES (`canonicalize_placeholder_indices`). The reasoning is sound: every descriptor on the device strip path is decoded (∴ passed `validatePlaceholderUsage`) before reaching WDT-Id, and `encodePayload` re-canonicalizes (no-op) before engrave.

- **I5 (default-regression pin at all four sites)** — **RESOLVED.** Risk 5 + Invariant 3 + Acceptance "Default regression" now state the selector must pick `WalletPolicyId` at ALL FOUR sites for a keyed policy and pin the full-policy engrave+verify byte/behaviour-identical to `39cb5cf`, with a negative (a full policy must NOT classify as a template). Consistent with the verified four-site set from C2.

---

## Open item ruled on

- **O4 (recovery-time estimate model)** — **RATIFIED (honest + arithmetically correct).** Verified against `mnemonic-toolkit/.../permutation_search.rs`:
  - **Search space is full N!** — `factorial(n)` (≈`:481-487`) with `unrank_permutation` exhaustive enumeration, no pruning. The SPEC's "ordered `multi` / distinct-origin slots → N!" is the correct space for the standard (explicit `--account`) case; the only larger space is the optional `--own-account-max K` subset×permutation feature, which the in-cycle display does not surface.
  - **`sortedmulti` → no search** is correct: sortedmulti is key-order-invariant (BIP-67 sort at script construction), so any cosigner assignment yields the same wallet. Honest nuance (the SPEC's GUI text and S6 capture it): the id-recompute path must BIP-67-normalize before hashing (still no *search*), and the operator must still *supply* the cosigner cards — which the GUI states loudly ("you ALSO need the cosigner key cards (mk1)").
  - **6.9 / 7.4 µs are runtime-calibrated empirical benchmarks** (`calibrate_per_candidate`), not hardcoded — i.e. machine-specific. (Round-0 M1's provenance note still applies; non-blocking.)
  - **Table arithmetic (independently recomputed) is exact** at 6.9 µs single-thread: N=5 → 120 perms = 0.83 ms (< 1 s ✓); N=9 → 362,880 perms = 2.50 s (✓ "2.5s"); N=12 → 479,001,600 perms = 55.09 min (✓ "~55min"). Defensible. The displayed table is conservative (single-thread; real toolkit parallelizes ~24×, making it faster, not slower — so it cannot understate recovery cost).

  One Minor consistency note (not blocking): the SPEC's GUI table uses `N=12 ≈ ~55min` while the `FOLLOWUPS.md:27` source-of-record model tabulates `N=11 ≈ 4.6 min` and `N=13 ≈ 12 h` (no N=12 row). Both are arithmetically consistent with the same 6.9 µs/N! model; the SPEC just picked a different illustrative N. Harmonizing the displayed rows with the FOLLOWUP record would avoid a future "which table is canonical?" question, but it is not a correctness defect — folded into M-list below.

---

## New findings

### Important

#### D1 (drift) — S3/S4 left a self-contradicting supply-path template story; "route around `allSlotsHaveXpub` via `expandTemplateOnly`" is unsound and contradicts the same paragraph's "supply path neither strips nor re-mints … engraved verbatim"

- **Location:** S3 last sentence (line 55): *"Do NOT widen the multisig derive-leg gate `allSlotsHaveXpub` (`gui/multisig_supply.go:72`) — route templates around it via `expandTemplateOnly` (`gui/md1_expand.go`)."* In tension with S3 mid-paragraph (line 55, "the SUPPLY path neither strips nor re-mints: a user-supplied template md1 … is engraved verbatim") and S4 (line 58, "a user wanting a template there supplies a keyless template md1, engraved verbatim").
- **Problem:** The `supplyMultisigPolicyFlow` is not a verbatim pass-through — it is a *seed-cross-match-then-derive* flow. After the `allSlotsHaveXpub` gate (`gui/multisig.go:83`) it (1) types a seed, then (2) at step (4) calls `findUserSlot(mnemonic, …, keys)` to CROSS-MATCH the seed to one of the supplied policy's slots **by xpub** (`gui/multisig.go:116`), and only then derives the operator's mk1 leg. For a SUPPLIED **template** md1 there are no xpubs, so:
  - `allSlotsHaveXpub` correctly refuses it (`gui/multisig_supply.go:68-82`, comment: "the supplied md1 must be a FULL wallet policy — every expanded slot must carry an xpub, else there is no public key to cross-match the typed seed against").
  - Even if you "route around" that gate, the very next step `findUserSlot` has nothing to match the seed against → the flow cannot determine which slot the operator occupies → it cannot derive a correct `mk1.Path`/leg. So "route around the gate" does not yield a working flow.
  - `expandTemplateOnly` (`gui/md1_expand.go:18`) is a *display-pipeline status* returned by `expandedToDescriptor` for the gather/inspect path (`md1_gather.go`); it is NOT a derive-leg mechanism and appears nowhere in `supplyMultisigPolicyFlow`. Citing it as the "route around" is a category error.
- **Evidence:** `gui/multisig.go:64-159` (the full supply flow: gate `:83` → seed `:91` → cross-match `findUserSlot` `:116` → `deriveMultisigLeg` `:140`); `gui/multisig_supply.go:68-82` (`allSlotsHaveXpub` and why it requires xpubs); `gui/md1_expand.go:9-39` (`expandTemplateOnly` is a display status, used by the gather pipeline, not the supply derive leg). This is the exact residual round-0 I2 named: *"the 'route around allSlotsHaveXpub' hand-wave replaced with a concrete flow."* The fold fixed the placement sentence but did not delete/replace the routing sentence, so v3 now asserts both "supply stays verbatim/unchanged" and "route templates around its full-policy gate."
- **Consequence:** An implementer reading S3/S4 literally would either (a) try to widen/bypass `allSlotsHaveXpub` and build a supply-template flow that then dead-ends at `findUserSlot` (no slot match) — wasted work on an infeasible path — or (b) be unsure whether the supply path is in-scope for templates at all. Worse, weakening `allSlotsHaveXpub` is a *security-relevant* gate (it is the precondition for the seed-to-slot cross-match that binds the operator's card to the right slot); touching it for an infeasible template path risks the full-policy supply flow. The contradiction must be removed before a plan is written.
- **Fix (pick one, state it explicitly):**
  1. **Cleanest — drop supply-template entirely this cycle.** Delete the S3 sentence "Do NOT widen … route templates around it via `expandTemplateOnly`," and rewrite S4 so the multisig SUPPLY path is **out of scope for templates** (it requires a full policy for the seed cross-match by construction). Templates are produced via single-sig and multisig-BUILD only (both already device-built/re-encoding paths where the strip is sound). State that a user with an already-made keyless template *bundle* (template md1 + N keyless mk1 cards) re-engraves it via the existing verbatim **scan/`mdmkFlow`** path (which engraves scanned strings as-is and whose verify is made form-aware by the C2 `verify.go:116` rewire) — NOT via `supplyMultisigPolicyFlow`. This keeps `allSlotsHaveXpub` untouched and removes the dead-end.
  2. **If supply-template is genuinely wanted**, specify a *concrete* new sub-flow that does not cross-match by xpub: e.g. supply template md1 + N pre-made keyless cosigner mk1 cards (no seed entry, no `findUserSlot`, no `deriveMultisigLeg`) → verbatim re-engrave + form-aware verify. This is a different flow from `supplyMultisigPolicyFlow`; name it, and explicitly leave `allSlotsHaveXpub` and the seed-cross-match supply flow unchanged. Do not describe it as "route around `allSlotsHaveXpub`."

  Either way: remove the `expandTemplateOnly`-as-routing claim (it is a display status), and make S3/S4 internally consistent (the supply path is *either* full-policy-only *or* gets a new, separately-named verbatim-bundle sub-flow — not a "widen the existing gate" hand-wave).

### Minor

#### M1 — (carried from round-0 M1) µs constants are runtime benchmark outputs; pin provenance
- **Location:** S6 (line 110), DD7 table (line 75).
- **Problem/Evidence:** Confirmed `6.9/7.4 µs` come from `calibrate_per_candidate` in `permutation_search.rs` (measured, not constant); `FOLLOWUPS.md:27` records the reference machine (24-core i7-13700 @ 5.3 GHz). The SPEC table presents them without that anchor.
- **Fix:** Add a one-line "reference machine; order-of-magnitude, off-device" note to the SPEC's S6/table (FOLLOWUPS.md already has it). Display-only; non-blocking.

#### M2 — illustrative N→time rows differ between the SPEC GUI mock and FOLLOWUPS.md
- **Location:** SPEC line 75 (`N=5 … N=9 … N=12 ≈ ~55min`) vs `design/FOLLOWUPS.md:27` (`N=5 … N=9 … N=11 ≈ 4.6 min, N=13 ≈ 12 h …`).
- **Problem:** Both are arithmetically correct under the same 6.9 µs/N! model (verified), but they pick different illustrative N, inviting a "which is canonical?" question when the FOLLOWUP and the cycle are implemented together.
- **Fix:** Harmonize the displayed rows (or cross-reference: "rows per FOLLOWUPS.md key-search-time-estimate"). Non-blocking.

#### M3 — (carried from round-0 M2) the §5 11-key fixture is still named, not pinned
- **Location:** Acceptance "Strip golden" (line 122), Risk 1 (line 140).
- **Problem:** The stress fixture's exact descriptor string + expected stripped bytes + WDT-Id are not inlined; a golden test needs a pinned vector.
- **Fix:** Inline the exact descriptor + expected template bytes + WDT-Id (or cite the precise toolkit test) into the SPEC or the forthcoming plan. Round-0 left this as Minor; still Minor. Recommend it be pinned in the implementation plan before the strip-golden test is written.

### Nit

#### N1 — scope line still references `mdmkFlow` ambiguously
- **Location:** Scope (line 133): "reuse … the verbatim engrave `mdmkFlow`."
- **Problem:** `mdmkFlow` is the scanned-string verbatim engraver, a different entry point from `engraveSingleSig`/`engraveMultisig`. Round-0 N1 flagged this. Note: if fix-1 of D1 is taken, `mdmkFlow` *becomes* the legitimate home for re-engraving a pre-made template bundle — in which case clarify that role rather than dropping it. Otherwise drop it from the template scope.

---

## Ratifications carried forward (verified again, do not re-litigate)

- **R0 pin #1 (no-canonicalize on WDT-Id):** re-confirmed — `compute_wallet_descriptor_template_id` (`identity.rs:71-104`) has no canonicalize; `compute_wallet_policy_id` (`identity.rs:172-177`) does. Guarantor (I4) now named in the SPEC.
- **R0 pin #2 (`pathDecl.n`/kiw guard inside WDT-Id):** re-confirmed needed — guard lives in `encodePayload` (`md/encode.go:401`), which WDT-Id bypasses; S2 now requires it locally.
- **Byte-faithful tree serialization:** unchanged from round 0 (Go `writeNode` byte-identical to Rust `tree::write_node`); WDT-Id preimage components (`use_site_path.write ‖ tree::write_node ‖ UseSitePathOverrides-TLV`, first 16 bytes) confirmed against `identity.rs:75-98`.
- **id-space non-collision:** WDT-Id and WalletPolicyId have structurally disjoint preimages; with the form-aware selector correct at all four sites (C2/I5, verified complete) a template and a full bundle cannot cross-validate.
- **Four-site completeness (C2):** independently re-enumerated — exactly four mint/recompute sites in the fork; no missed site. This was round 0's highest-risk item and is clean.
- **`d.n` availability for PolicyComplex (C3):** `Template.N` is sourced from `d.n` regardless of `classifyPolicy`'s verdict — the honest-minimal consent surface is truthfully computable.

---

## VERDICT

**NOT GREEN — 0C / 1I.** Blocking ID: **D1** (the I2 fold's residual supply-path contradiction — S3/S4 simultaneously claim the supply path stays verbatim AND "route templates around `allSlotsHaveXpub` via `expandTemplateOnly`," which is infeasible against the seed-cross-match flow and miscites a display-only status). Fix D1 with one focused edit (drop the routing sentence; make the supply path either full-policy-only or give it a concrete, separately-named verbatim-bundle sub-flow), re-persist, and re-dispatch round 2. All three round-0 Criticals and four of five Importants are fully resolved and verified against source; O4 is ratified; the hash-math core and both R0 pins continue to hold. The remaining items (M1–M3, N1) are non-blocking and should be cleared in the implementation plan.
