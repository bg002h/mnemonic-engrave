# R0 architect review — SPEC_seedhammer_template_engrave (round 0)

- **Artifact:** `design/SPEC_seedhammer_template_engrave.md` (DRAFT v2)
- **Fork SHA:** `/scratch/code/shibboleth/seedhammer` @ `39cb5cf`
- **Rust pins:** descriptor-mnemonic @ `54dd765`, mnemonic-key @ `1279ef9`, mnemonic-toolkit @ `6de53879`
- **Date:** 2026-06-20
- **Reviewer role:** adversarial OPUS architect, mandatory pre-implementation gate (must reach 0C/0I)
- **Method:** every load-bearing claim verified against authoritative source text, not the draft prose (false-consensus guard).

This review is BLOCKING. Findings are grouped Critical / Important / Minor / Nit, each with a stable ID, SPEC location, problem, evidence (file:line), and a concrete fix.

---

## Summary verdict

**NOT GREEN — 3 Critical / 5 Important.**

Blocking: **C1, C2, C3, I1, I2, I3, I4, I5.**

The cryptographic core the SPEC leans on (the WDT-Id port over the already-byte-faithful `writeNode`, R0 pin #1's no-canonicalize, R0 pin #2's `pathDecl.n` guard) is **correctly reasoned and verified sound** — those two pins survive scrutiny and should be RATIFIED. The blocking issues are *not* in the hash math; they are in (a) the strip transform being mis-specified vs the toolkit it golden-locks to, (b) two whole classes of call-site the rewire misses, and (c) the DD7 informed-consent surface claiming a summary the device provably cannot compute for the very shapes that need it.

---

## Critical

### C1 — The strip transform's origin handling contradicts the toolkit it golden-locks to (DD1/S1 will NOT be byte-identical, and worse, will produce a decode-rejected wire for general policy)

- **Location:** DD1 (line 20), S1 (line 45), Invariant 2 (line 102), Risk 1 (line 125), O2 (line 135). Every one says the strip "elide[s] origin" / "origin elision" unconditionally.
- **Problem:** The toolkit's `synthesize_template_descriptor` does **not** elide origin unconditionally. It is **C1-CONDITIONAL on `canonical_origin(&tree)`**:
  - `canonical_origin(&tree).is_some()` (canonical single-sig, `wsh(multi/sortedmulti)`, `sh(wsh(...))`) → elide to `Shared(empty)`.
  - `canonical_origin(&tree).is_none()` (general policy: `wsh(or_i(...))`, `thresh`, timelocks — i.e. the §5 degrading wallet the acceptance test names as the stress vector) → **KEEP the source per-`@N` origins verbatim.** Eliding here makes `md decode` REJECT the wire via `validate_explicit_origin_required → MissingExplicitOrigin` (the toolkit comment literally calls this "the C1 regression").
- **Evidence:** `mnemonic-toolkit/crates/mnemonic-toolkit/src/synthesize.rs:1185-1198` — the `if canonical_origin(&descriptor.tree).is_some() { …Shared(empty) } else { /* leave cloned source origins */ }` branch; the rejection mechanism is `validate_explicit_origin_required → MissingExplicitOrigin` (also documented in the fork at `md/md.go:1060-1062 errMissingExplicitOrigin` and `canonicalOrigin` `md/md.go:1097-1129`, where `tr(@N,TapTree)` and any non-listed wrapper return `(…, false)`).
- **Consequence:** A literal implementation of DD1/S1 ("elide origin") would (i) diverge byte-for-byte from `toolkit bundle --md1-form=template` for every divergent-origin / general-miniscript shape, failing the strip-golden acceptance test, and (ii) for the general case actually emit an **undecodable** template md1 (engrave-but-can't-read-back) — a funds-safety regression, the exact opposite of what DD3 promises ("ENGRAVE + VERIFY cover any admissible shape").
- **Fix:** Re-specify DD1/S1's mutation 3 as the toolkit's C1-conditional rule verbatim: strip = `pubkeys=nil; fingerprints=nil; if canonicalOrigin(tree) present → pathDecl = Shared(empty) else KEEP source pathDecl (Shared/Divergent)`. Update Invariant 2, Risk 1, O2 to name the conditional. Add a dedicated acceptance vector for a `canonical_origin==None` shape that asserts (a) byte-identity to the toolkit AND (b) the engraved template decodes (round-trips) on-device.

### C2 — The derive-leg stub minting is not made form-aware; the engraved template's mk1 will carry the WRONG stub and fail its own verify

- **Location:** DD4/S3 (lines 23, 54-55). The SPEC rewires only `bundle/verify.go:116` and the "Files" scope (line 118) names only `bundle/verify.go` for the binding change.
- **Problem:** The mk1 `policy_id_stub` that gets **engraved** is minted on the DERIVE side, not in verify. All three derive/build legs mint it via `md.WalletPolicyIDStubChunks(md1)` unconditionally:
  - `gui/singlesig_derive.go:67`
  - `gui/multisig_derive.go:42` (also reached by the BUILD path `gui/multisig_build.go:115`)
  - `md/encode_multisig.go:158`
  For a keyless template md1, `WalletPolicyIDStubChunks → WalletPolicyId` does **not** error — it hashes a keyless preimage (presence byte `0x00`, no fp/xpub appended; see `md/walletpolicyid.go:87-98`, `xpubForId`/`fpForId` return `present=false`) and produces a `WalletPolicyId`-of-a-keyless-descriptor stub. That stub is **structurally different** from `WalletDescriptorTemplateId` (different preimage entirely: WDT-Id is `SHA256(use_site ‖ writeNode ‖ overrides-TLV)`, WalletPolicyId is `SHA256(tree ‖ per-@N records)`).
- **Evidence:** Rust `derive_stub_from_md1` (`mnemonic-key/crates/mk-cli/src/cmd/mod.rs:72-82`) is form-aware on BOTH the mint and verify sides; the toolkit mints the template card stub from `compute_wallet_descriptor_template_id` (`synthesize.rs:1206-1209`). The fork's derive legs are NOT form-aware.
- **Consequence:** Even with verify.go rewired, an engraved template would mint a `WalletPolicyId`-stub mk1 but verify against a `WalletDescriptorTemplateId` → **the device's own readback verify FAILS every template engrave** (or, worse, if both legs were left WalletPolicyId, it would "verify" against an id no other tool — including the toolkit `restore` — can reproduce, breaking off-device recovery binding).
- **Fix:** The form-aware selector must be applied at EVERY stub-minting site, not just verify. Introduce one helper `formAwareStubChunks(md1) = isWalletPolicy ? WalletPolicyIDStub : WalletDescriptorTemplateIdStub` and route `singlesig_derive.go:67`, `multisig_derive.go:42`, `encode_multisig.go:158`, AND `verify.go:116` through it. Add the derive-side stub to the scope (line 118) and add an acceptance test that the engraved template mk1's stub equals the toolkit's WDT-Id stub byte-for-byte.

### C3 — DD7 promises a "k-of-N + cosigner-count" safe summary for shapes the device provably classifies as PolicyComplex with k=0,N=0 — the informed-consent surface is materially weaker than specified

- **Location:** DD7 (line 26), S-display, Invariant/Risk framing; the funds-safety claim that the summary "is a sound confirmation surface."
- **Problem:** For the exact shapes DD7 routes to the safe summary (general miniscript, depth-≥2 taptrees, `tr(NUMS,multi_a)`), `classifyPolicy` returns `PolicyComplex, 0, 0` — it cannot extract k, N, or cosigner count. The SPEC's promised summary content ("script type, k-of-N, cosigner count, use-site") is **not derivable** for `PolicyComplex`. The device can show at most "complex policy + template-id."
- **Evidence:** `md/md.go:1266-1315 classifyPolicy` — `tagTr` with a script tree falls through (comment: "ANY tr with a script tree is refused"), and every non-`{wpkh,pkh,tr-keypath,wsh-multi,sh-...}` shape returns `PolicyComplex, 0, 0`. The recon's Stream B independently states "Any `tr` with a script tree and all combinators hard-refuse → `PolicyComplex`" (`md-codec-readiness-verdict…:21`). `scriptForTemplate` (`gui/md1_expand.go:82-121`) returns `!ok` for the same shapes.
- **Consequence:** This is the load-bearing funds-safety question (review priority 3). For an unrenderable shape the user is asked to consent to engraving a backup whose on-device confirmation is **only an opaque 16-byte id** they must blindly trust matches their off-device toolkit — there is no structural cross-check (no k, no N, no cosigner count) the user can independently sanity-check on the air-gapped device. The SPEC's rationale ("structural summary + id is a sound confirmation surface") is built on a summary that does not exist for these shapes.
- **Note on the VERIFY gap question (priority 3, second half):** VERIFY itself does NOT degrade — readback binding is the template-id over the decoded tree and is byte-complete for any decodable shape (confirmed: WDT-Id reuses the byte-faithful `writeNode`, `md/walletpolicyid.go:42` / `encode.go:159`). So there is no verify-strength gap; the gap is purely in the human-inspectable DISPLAY/consent surface. That distinction must be made explicit, because "verify fully binds" does not rescue "the human cannot inspect what they are binding."
- **Fix:** Either (a) NARROW DD7's engrave scope this cycle to shapes the device can summarize structurally (i.e. `classifyPolicy != PolicyComplex`), deferring `PolicyComplex` engrave to the broad-renderer FOLLOWUP — so every engravable shape has a k-of-N/N-cosigner cross-check; OR (b) explicitly re-state DD7's consent surface for `PolicyComplex` as "complex policy + template-id ONLY (no structural summary)" and have the brainstxorm/owner re-affirm that an opaque-id-only consent is acceptable for a self-custody backup device, recording that decision. Do NOT ship prose claiming a k-of-N summary the code returns as 0,0. The SPEC's acceptance test for GUI summary strings must assert the ACTUAL derivable content per shape class.

---

## Important

### I1 — `is_wallet_policy() = d.tlv.pubPresent` is NOT the Rust predicate, and the gap is reachable by the strip transform

- **Location:** DD4 (line 23), S2 (line 49), O3 (line 136), and the readiness verdict (`:27` "(b) `isWalletPolicy() = d.tlv.pubPresent && len(d.tlv.pubkeys) > 0`"). The SPEC body drops the non-empty clause and states `is_wallet_policy() = d.tlv.pubPresent`.
- **Problem:** Rust is `matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty())` — **present AND non-empty** (`encode.rs:50-52`). On the pure DECODE path the gap is unreachable (an empty Pubkeys TLV is rejected at decode: `md/md.go:579-581 errEmptyTLV`), which is presumably why the SPEC shortened it. But the template strip does NOT go through decode for its mutation — it mutates an in-memory `*descriptor` (DD1: "null Pubkeys TLV"). The crux: how does the strip null pubkeys?
  - If it sets `pubkeys = nil` AND `pubPresent = false` → `pubPresent`-only predicate is correct.
  - If it sets `pubkeys = nil` but LEAVES `pubPresent = true` → the `pubPresent`-only predicate wrongly reports wallet-policy; AND `encodePayload → writeTLVSection` will **error with `errEmptyTLVEncode`** (`md/encode.go:292-295`: `if s.pubPresent && len(s.pubkeys)==0 → errEmptyTLVEncode`).
- **Evidence:** `md/encode.go:292-295` (empty-but-present TLV is a hard encode error); `md/md.go:528 pubPresent` is a separate bool field from the `pubkeys` slice, so the two can desync in an author-built/mutated descriptor; Rust collapses both into the `Option` so they cannot desync.
- **Consequence:** A under-specified strip is one assignment away from either a mis-classified template (binds via the wrong id) or a hard encode failure. The `pubPresent`-only shorthand papers over the exact invariant the strip must maintain.
- **Fix:** Specify the strip as clearing BOTH (`pubkeys=nil; pubPresent=false; fingerprints=nil; fpPresent=false`). Specify `isWalletPolicy()` as `d.tlv.pubPresent && len(d.tlv.pubkeys) > 0` (the full Rust predicate), so it is robust even if a future caller desyncs the flag. Add an acceptance assertion that a freshly-stripped descriptor satisfies `!isWalletPolicy()` and encodes without `errEmptyTLVEncode`.

### I2 — The strip + re-encode breaks the SUPPLIED-multisig flow's load-bearing "engrave VERBATIM" invariant (I-2); S4's `engraveMultisig` attachment is ambiguous and partly unsound

- **Location:** S4 (lines 57-58), DD1 (line 20 "whatever full md1 the device holds … or user-supplied"), scope (line 118).
- **Problem:** There are two distinct multisig engrave flows, with opposite re-encode semantics:
  - `supplyMultisigPolicyFlow` (`gui/multisig.go:64-`) gathers a SUPPLIED md1 and engraves it **VERBATIM** — `deriveMultisigLeg` explicitly does NOT re-encode ("I-2 — the device never re-encodes a multisig descriptor", `gui/multisig_derive.go:20`, `:58-60`). It also HARD-REQUIRES a full policy (`allSlotsHaveXpub`, `gui/multisig.go:83-86`) because it cross-matches the typed seed to a slot.
  - `buildMultisigPolicyFlow` (`gui/multisig_build.go:38-`) AUTHORS the policy on-device via `md.EncodeMultisig` (re-encodes by construction) and already gates behind a mandatory EXPERIMENTAL warning (`multisigBuildExperimentalWarning`, `:145`).
- **Evidence:** the verbatim invariant `gui/multisig_derive.go:58-60` + `:20`; the full-policy supply gate `gui/multisig.go:83-86` + `allSlotsHaveXpub` `gui/multisig_supply.go:72`.
- **Consequence:** "Strip whatever md1 the device holds and re-emit keyless" cannot attach to the SUPPLIED flow without (a) violating the verbatim-engrave security property (the device would now re-encode a third-party-supplied descriptor — a re-encode differential the verbatim rule exists to avoid) and (b) requiring a full-policy supply only to immediately strip it (odd UX, and the user already has the keys on plates). The natural home for the strip is the BUILD path (already re-encodes, already authors keyless-capable trees) and the single-sig derive path. The SPEC's claim "route templates around `allSlotsHaveXpub`" (line 55) only makes sense for a path that isn't trying to cross-match a seed — which the supply path fundamentally is.
- **Fix:** Decide and state explicitly WHICH flows expose the template opt-in. Recommended: single-sig (`engraveSingleSig`, device-built) + the BUILD multisig sub-flow (`buildMultisigPolicyFlow`, device-authored), NOT the supply sub-flow. If supply-flow template is desired, it must be re-justified against the verbatim invariant and the seed-cross-match requirement, and the "route around allSlotsHaveXpub" hand-wave replaced with a concrete flow. Update S4, DD1 ("user-supplied"), and the scope.

### I3 — The WDT-Id Go port must take its key-index width and the no-canonicalize from the as-decoded descriptor; the SPEC pins this only loosely, and the `WalletPolicyId` copy-template actively fights it

- **Location:** S2 R0 pin #1 (line 51), R0 pin #2 (line 52).
- **Problem:** R0 pin #1 is correct (Rust `compute_wallet_descriptor_template_id` does NOT canonicalize — verified `identity.rs:71-104`, no `canonicalize_placeholder_indices` call, vs `compute_wallet_policy_id` `identity.rs:175-176` which does). But the SPEC tells the implementer to "mirror `WalletPolicyId`'s `writeNode`-based preimage … but WITHOUT the `canonicalize(d)` call at `:32`." The existing `WalletPolicyId` ALSO derives its width from the canonicalized clone: `width := kiw(dc.pathDecl.n)` at `md/walletpolicyid.go:37` AFTER `dc, _ := canonicalize(d)` at `:32`. If the implementer removes only the `canonicalize` line but keeps `kiw(d.pathDecl.n)`, the width source is now `d.pathDecl.n` on the RAW input. Rust uses `d.key_index_width()` = `⌈log₂(d.n)⌉` off `descriptor.n` (`encode.rs:37-41`, `identity.rs:76`). These agree ONLY when `pathDecl.n == d.n`.
- **Evidence:** Rust width = `d.key_index_width()` off `d.n` (`identity.rs:76`, `encode.rs:37-41`); Go `WalletPolicyId` width off `dc.pathDecl.n` post-canonicalize (`md/walletpolicyid.go:37`); the lockstep guard `errPathDeclNMismatch` lives in `encodePayload` at `md/encode.go:401` — NOT in `walletpolicyid.go`. So a WDT-Id port that copies `walletpolicyid.go` minus the canonicalize line inherits NEITHER the guard NOR a defined width source.
- **Consequence:** On any descriptor where `pathDecl.n != n` (an author-built/mutated AST, or a future divergent shape), the ported WDT-Id silently uses the wrong kiw and mis-binds — exactly the "1 valid last word"-class silent-corruption the project guards against. R0 pin #2 names the guard but the SPEC must require it INSIDE the WDT-Id function (since WDT-Id bypasses `encodePayload` where the existing guard lives).
- **Fix:** Strengthen S2: the WDT-Id port MUST (i) NOT canonicalize, (ii) compute width as `kiw(d.n)` (mirroring Rust `key_index_width()` off `n`, not `pathDecl.n`), and (iii) include the `pathDecl.n == n` guard locally (returning `errPathDeclNMismatch`) before serializing. Add an acceptance test feeding a `pathDecl.n != n` AST and asserting the guard fires rather than producing a stub.

### I4 — The no-canonicalize pin is sound ONLY because of the decode-side `validate_placeholder_usage` invariant; the SPEC relies on it without naming the guarantor, leaving the strip's author-built-AST path unprotected

- **Location:** S2 R0 pin #1 (line 51, "relying on the decode-side canonical invariant"); O1 ratification (line 134).
- **Problem:** "Relying on the decode-side canonical invariant" is correct but the SPEC never states WHO enforces it, so an implementer feeding a non-decoded (author-built / freshly-stripped-then-not-re-decoded) descriptor straight to WDT-Id has no protection. The invariant is enforced by `validate_placeholder_usage`, which REJECTS any tree whose placeholder first-occurrences are not ascending — run at decode in both Rust and Go.
- **Evidence:** Rust `validate_placeholder_usage` (`md-codec/src/validate.rs:15-28` "first occurrences … in canonical ascending order"); Go equivalent at decode `md/md.go:1138 validatePlaceholderUsage` and in `encodePayload` `md/encode.go:380`. The device strip path: strip mutates an already-decoded (∴ canonical) descriptor, then `encodePayload` canonicalizes again (no-op) and the engraved md1 re-decodes canonical for verify. The toolkit computes WDT-Id over a `parse_descriptor`-built (first-occurrence-ordered ∴ canonical) descriptor (`synthesize.rs:1181,1206`). So device and toolkit agree — but ONLY because both inputs are canonical-in-tree-order.
- **Consequence:** If a future caller computes WDT-Id over a non-canonical author-built AST (e.g. a test fixture, or a refactor that moves the id computation before re-decode), Go and toolkit would still agree with EACH OTHER (both skip canonicalize) but would BOTH compute a "wrong" template-id relative to the canonical wire that gets engraved — a self-consistent-but-wrong binding. The pin is load-bearing precisely at the boundary the SPEC leaves implicit.
- **Fix:** Add to S2/O1 the explicit guarantor: "WDT-Id is computed over a descriptor that has passed `validatePlaceholderUsage` (canonical-in-tree-order); callers MUST NOT pass an un-validated author-built AST." Either compute WDT-Id only over the canonical post-`encodePayload` re-decoded form, or run `validatePlaceholderUsage` (NOT `canonicalize`) at the top of the WDT-Id function as a tripwire.

### I5 — Default-path regression pin is necessary but insufficient: it must also pin that the form-aware selector picks WalletPolicyId on EVERY full-policy stub site, not just verify.go

- **Location:** Invariant 1 (line 101), Risk 5 (line 129), acceptance "Default regression" (line 111), O5 (line 138).
- **Problem:** The regression pin as written ("full-policy engrave + verify byte/behaviour-identical to `39cb5cf`") only covers the verify leg. Once C2's fix routes all FOUR stub sites through the form-aware selector, the regression surface widens to `singlesig_derive.go:67`, `multisig_derive.go:42`, `encode_multisig.go:158`. A selector bug that mis-detects a full policy as a template (e.g. the I1 `pubPresent`-vs-non-empty gap) would silently change the full-policy stub id and break the DEFAULT path.
- **Evidence:** the four sites enumerated in C2; the full-policy multisig builder roots its bundle on `WalletPolicyIDStub` (`md/encode_multisig.go:158`).
- **Consequence:** Without pinning the selector's full-policy verdict at every mint site, the additive-and-default-unchanged guarantee (the cycle's headline safety property) is only partially tested.
- **Fix:** Extend the default-regression acceptance test to assert, for a full keyed md1, that `formAwareStubChunks` returns the identical `WalletPolicyIDStub` at all four sites (selector picks the policy branch), AND that the engraved full bundle is byte/behaviour-identical to `39cb5cf`. Add a negative: a full policy must NOT be classified as a template.

---

## Minor

### M1 — Recovery-estimate µs constants (6.9 / 7.4) are presented as fact but are benchmark outputs with no pinned provenance

- **Location:** S6 (line 96), DD7 estimate table (lines 74-75), O4 (line 137).
- **Problem:** The N! search MODEL is verified correct (`permutation_search.rs:14,243,481-510`: id-search = `n!`, sortedmulti order-invariant ⇒ no search). But 6.9/7.4 µs/perm are machine/build-specific timings (`permutation_search.rs:461-462` measures `elapsed/samples`), not constants. The displayed `N=12 ≈ ~55min` depends entirely on them.
- **Fix:** Mark the table as an order-of-magnitude estimate on a reference machine, pin the source benchmark + commit that produced 6.9/7.4, and frame the on-device copy as "minutes-to-hours, off-device" rather than precise minutes. Display-only, so non-blocking — but honesty (priority 7) wants the provenance pinned.

### M2 — The §5 "11-key degrading-miniscript" golden fixture is named but undefined in the cycle docs

- **Location:** acceptance "Strip golden" (line 108), Risk 1 (line 125).
- **Problem:** The stress fixture is referenced as "the §5 degrading wallet / 11-key example" but its descriptor string is not pinned in the SPEC or recon; it lives in an external md SPEC §5. A golden test cannot be written against an unpinned fixture.
- **Fix:** Inline the exact descriptor string (and the expected stripped template bytes + WDT-Id) into the SPEC/plan as a pinned vector, OR cite the precise toolkit test that already encodes it.

### M3 — DD6 depth-1 vs depth-≥2 distinction is drawn at the wrong granularity for `tr(NUMS, multi_a)`

- **Location:** DD6 (line 25), S5 (lines 80-93), DD7 (line 26).
- **Problem:** `tr(NUMS, multi_a)` is admissible per the toolkit (`synthesize.rs:1164` admits it) and is depth-1, so under DD6 it takes the "normal" depth-1 path — yet `classifyPolicy` returns `PolicyComplex` for it (any tr with a script tree), so it ALSO hits the DD7 safe-summary path. The SPEC treats "depth-1 = normal" and "PolicyComplex = summary-only" as if they partition cleanly; `tr(NUMS,multi_a)` is in both.
- **Evidence:** `md/md.go:1283-1284` (any tr with a tree → not PolicySingle → PolicyComplex); admitted by toolkit `synthesize.rs:1164`.
- **Fix:** State that the experimental-warning gate (DD6) and the display-breadth gate (DD7) are ORTHOGONAL axes: a shape may be depth-1-non-experimental yet still summary-only. Tie the warning to "off-device recoverability with shipped tooling," and the display to `classifyPolicy`, independently.

### M4 — DD6 cites "rust-miniscript >13.1.0 / PR #953" as the recovery blocker without a verification anchor in-repo

- **Location:** DD6 (line 25), S5 (line 88), O1 (line 134).
- **Problem:** The recovery-blocked-by-#953 claim is an external-protocol fact (per project policy, must be verified against authoritative source). The cycle docs assert it but cite no pinned check (e.g. a toolkit test that demonstrates the taptree-display failure on the shipped crates.io rust-miniscript).
- **Fix:** Anchor the claim to a reproducible artifact (a toolkit test or a documented `restore` failure on the §5/depth-≥2 fixture) so the EXPERIMENTAL warning's premise is itself verified, not asserted.

---

## Nit

### N1 — Scope line conflates `mdmkFlow` with the engrave entry points

- **Location:** scope (line 118): "reuse … the verbatim engrave `mdmkFlow`."
- **Problem:** `mdmkFlow` (`gui/gui.go:1972`) is the SCANNED-string engraver (scan an md1/mk1, engrave verbatim) — a different entry point from `engraveSingleSig`/`engraveMultisig` (S4). Citing it as the template engrave host is misleading.
- **Fix:** Drop the `mdmkFlow` reference from the template scope or clarify it is unrelated.

### N2 — The `gui/gui.go:164` "no new program" guard claim is correct — confirm-and-keep

- **Location:** S4 (line 58), scope (line 119).
- **Problem (none — confirmation):** The compile-time guard `var _ [1]struct{} = [qaProgram - bip85Derive]struct{}{}` (`gui/gui.go:164`) trips only if a program is inserted between `bip85Derive` and `qaProgram`. Adding inner ChoiceScreens to existing `engraveSingleSig`/`engraveMultisig` programs adds no enum entry, so the guard is untouched. The BUILD sub-flow precedent (`engraveMultisigFlow` branches internally without a new program, `gui/multisig.go:38-55`) confirms the pattern is established.
- **Fix:** None — accurate. Noted so it is not re-litigated.

### N3 — TinyGo/secret-hygiene assertion is plausible and consistent

- **Location:** line 122. Template emit handles only public data (xpubs/template); the strip nulls keys; no new secret path. Consistent with the derive flows' existing scrub discipline (`singlesig_derive.go` / `multisig_build.go` defer-scrub). Final TinyGo device build remains the integration gate as stated. No action.

---

## Ratifications (verified sound — carry forward, do not re-litigate)

- **R0 pin #1 (no-canonicalize on WDT-Id):** CONFIRMED against source. `compute_wallet_descriptor_template_id` has no canonicalize call (`identity.rs:71-104`); `compute_wallet_policy_id` does (`identity.rs:175-176`). Soundness depends on I4's guarantor being named.
- **R0 pin #2 (`pathDecl.n` / kiw guard):** CONFIRMED needed; see I3 — must live INSIDE the WDT-Id function since it bypasses `encodePayload`.
- **Byte-faithful tree serialization:** CONFIRMED. Go `writeNode` (`md/encode.go:159-232`) is byte-identical to Rust `tree::write_node` (tag, kiw widths, `(k-1)|(n-1)` 5-bit packing, `tr` is_nums/has_tree, 32-bit timelock, raw hash bytes). The WDT-Id preimage's three components (use-site `write`, `writeNode`, UseSitePathOverrides TLV entry at tag `0x00`) all have byte-faithful Go primitives (`writeUseSitePath` `encode.go:134`, `writeVarint` `encode.go:52`, `reEmitBits` `bits.go:156`, tag const `md/md.go:495 = 0x00` == Rust `tlv.rs:11`). No path to a wrong template-id from the serializer.
- **`is_wallet_policy` shape (modulo I1):** the Go predicate's intent matches Rust; only the empty-vec clause is dropped — fixed by I1.
- **id-space non-collision (priority 1):** WDT-Id and WalletPolicyId have structurally disjoint preimages (`SHA256(use_site‖tree‖overrides)` vs `SHA256(tree‖per-@N-records-with-presence-byte)`); a template and a full bundle cannot cross-validate provided the selector is correct at every site (C2/I5). No id-space collision found.

---

## VERDICT

**NOT GREEN — 3C / 5I.** Blocking IDs: **C1, C2, C3, I1, I2, I3, I4, I5.** Fold all eight, re-persist, and re-dispatch. The hash-math core and both R0 pins are ratified; the blockers are in the strip spec, the unaddressed derive-side stub sites, the supply-flow verbatim tension, and the DD7 consent-surface over-claim.
