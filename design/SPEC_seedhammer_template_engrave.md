# SPEC — SeedHammer fork: on-device wallet-policy TEMPLATE engraving (single-sig + multisig + general)

- **Status:** DRAFT v2 — md-codec readiness investigation folded in (2026-06-20: no prerequisite cycle; O1 resolved YES; option-3 display scope = DD7; R0 pins added to S2). Now awaiting the opus R0 gate (must reach 0C/0I before any plan/code).
- **Type:** Brainstorm SPEC (single-author per project policy). NOT a plan, NOT code.
- **Cycle:** `seedhammer-template-engrave`. Recon: `design/cycle-prep-recon-seedhammer-template-engrave.md` (+ the 3 per-stream findings `design/agent-reports/seedhammer-template-engrave-recon-{codec,toolkit,fork}.md`). Supersedes the single-sig-only `cycle-prep-recon-constellation-template-only-engraving.md`.
- **Fork / source of truth:** `/scratch/code/shibboleth/seedhammer`, branch `main`, HEAD `39cb5cf`. Go via `export PATH=$PATH:/home/bcg/.local/go/bin`.
- **Constellation pins (golden-lock targets, no change needed there):** md-codec/md-cli `descriptor-mnemonic@54dd765` (v0.37.0), mk-cli `mnemonic-key@1279ef9` (v0.10.0), toolkit `mnemonic-toolkit@6de53879` (v0.60.0). Template support shipped: single-sig v0.59.0, multisig/general v0.60.0; md-codec/mk-codec unchanged (the keyless wire form, `WalletDescriptorTemplateId`, and the form-aware stub pre-existed).
- **Nature:** Additive fork-firmware feature. Default behavior UNCHANGED (full-policy md1 + existing verify). Firmware-only; no `me`/CLI/`me-preview`/schema/docs-mirror surface; no new `program`.

---

## Why (one paragraph)

A full wallet-policy md1 embeds every cosigner xpub (~2–3 plates). A **template** md1 omits the keys (`pubkeys:null`) — ~1 plate — and the watch-only wallet is recomposed off-device from the template + the cosigner key cards (mk1) ± a key-permutation search. The constellation already supports template engraving end-to-end (toolkit `bundle --md1-form=template` + the off-device permutation-search `restore`/`verify-bundle`). This cycle adds the SeedHammer on-device leg: let the user **opt in** to engraving a template (default stays full-policy) for fewer plates, with a loud warning that a template alone cannot rebuild the wallet, and a recovery-time-vs-N estimate. The fork already engraves md1 verbatim and decodes/displays templates; the real work is a tree-agnostic key-strip, a Go port of the template-stable id + form-aware verify binding, the opt-in GUI, and the estimate display.

---

## Locked decisions (from the 2026-06-20 brainstorm)

- **DD1 — On-device emit via a tree-AGNOSTIC strip-keys transform.** A template is produced by taking whatever **full** md1 the device holds (built from seed/cosigners, or user-supplied) and stripping it to keyless — decode → null `Pubkeys` TLV + null `Fingerprints` + elide origin (mirroring the toolkit's `synthesize_template_descriptor` mutations) → re-emit keyless. NOT a per-shape keyless encoder. **Golden-locked byte-for-byte to `toolkit bundle --md1-form=template`.** **(Feasibility confirmed 2026-06-20: the re-emit routes through the shape-GENERAL `encodePayload(d)` (`md/encode.go:374`) — the same serializer behind `WalletPolicyId` — NOT the 7 high-level builders, so the strip works for any admissible shape incl. general miniscript / depth-≥2 tr.)**
- **DD2 — NO on-device search.** The fork engraves the template bundle + binds (template-stable stub) + verifies the engraved readback + DISPLAYS the recovery-time estimate. The permutation-search recompose runs OFF-device in the toolkit. (RP2350 is slow; the device has no recovery role.)
- **DD3 — Scope = any ADMISSIBLE md1** (single-sig, sortedmulti/multi multisig, AND general miniscript like the §5 degrading wallet). "Admissible" = the constellation's `template_admissible` (`mnemonic-toolkit synthesize.rs:1113`): renders via `to_miniscript_descriptor` AND not a refused shape. **ENGRAVE + VERIFY cover any admissible shape** (the wire codec is shape-complete — see Pre-R0 readiness). **On-device DISPLAY is narrower and option-3-scoped — see DD7.**
- **DD4 — Form-aware verify binding (security-load-bearing).** Port `md.WalletDescriptorTemplateId` (Go) + a form-aware stub `is_wallet_policy() ? WalletPolicyId : WalletDescriptorTemplateId` (top-4), rewired into `bundle/verify.go`. Byte-exact vs Rust `derive_stub_from_md1`.
- **DD5 — Default full, template OPT-IN behind a LOUD warning.** The engrave flow defaults to the full-policy md1; "template-only (fewer plates)" is an explicit opt-in showing the warning + the recovery estimate before engraving.
- **DD6 — Taproot:** depth-1 supported normally; **depth-≥2 supported behind a LOUD EXPERIMENTAL GUI warning** naming the dependency (rust-miniscript **>13.1.0**, PR #953) and that recovery is not possible with shipped tooling / not-for-real-funds. `tr(sortedmulti_a)` and `sortedmulti`-in-combinator REFUSED (md-codec/crates.io rust-miniscript lacks `sortedmulti_a`); hardened use-site REFUSED at the derive/address path (`HardenedPublicDerivation`), not at the template parser. **(O1 RESOLVED by the 2026-06-20 investigation: the fork's md-codec has NO taptree-depth refusal — `validateTapScriptTree` checks only leaf types; the only cap is the general recursion bound `maxDecodeDepth`. A depth-≥2 taptree decodes + re-encodes + computes its template-id byte-faithfully at the WIRE level. The "experimental" framing is because off-device RECOVERY via shipped rust-miniscript is blocked, NOT the on-device wire encode.)**
- **DD7 — On-device DISPLAY breadth = MINIMAL NOW, broad renderer DEFERRED (user decision 2026-06-20, "option 3").** Engrave + verify work for ANY admissible md1 (the wire codec is shape-complete). But on-device EXPAND/DISPLAY (`md/md.go:1266` `classifyPolicy` + `gui/md1_expand.go:82` `scriptForTemplate`) only *fully renders* the shapes the fork already handles (single-sig, `wsh`/`sh(wsh)` multi & sortedmulti). General miniscript / depth-≥2 taptrees / `tr(NUMS,multi_a)` show a generic **SAFE SUMMARY** (script type, k-of-N, cosigner count, use-site) **+ the template-id**, under the loud (and where applicable experimental) warning — NOT a full on-screen script expansion. The broad `to_miniscript.rs`-equivalent renderer is deferred to FOLLOWUP **`seedhammer-broad-miniscript-renderer`**. Rationale: the cryptographically-meaningful guarantees (verbatim engrave + template-id verify binding) are shape-complete already; the device is a backup/engraving device whose user cross-checks the template-id against their off-device toolkit, so a structural summary + id is a sound confirmation surface for shapes the on-device renderer can't yet draw.

---

## Pre-R0 readiness — fork md-codec investigation (2026-06-20)

The user paused to ask whether the fork's Go md-codec must be updated FIRST. A 3-stream primary-source investigation (verdict `design/md-codec-readiness-verdict-seedhammer-template-engrave.md`; reports `design/agent-reports/seedhammer-md-codec-sync-investigation-{skew,coverage,primitives}.md`) concluded **NO prerequisite md-codec cycle**:

- **Decode + tree serialization are shape-complete and byte-faithful.** Decode is a faithful port of Rust `read_node` (all 36 tags / 8 bodies); `writeNode`+`writeTLVSection` are byte-faithful to Rust `tree::write_node` for every shape (already in production behind `WalletPolicyId`).
- **`encodePayload(d *descriptor)` (`md/encode.go:374`) is the shape-GENERAL serializer** behind `WalletPolicyId`/chunk/`encodeMD1String`. Its gates are the same validators the decoder uses (`validatePlaceholderUsage`/`validateMultipathConsistency`/`validateTapScriptTree`) + the `pathDecl.n` guard — NOT shape refusals. So **DD1's strip = decode → null pubkeys/fp TLV + elide origin → `encodePayload`** works for ANY admissible shape; the "7 hardcoded builders" (`EncodeSingleSig`/`EncodeMultisig`) are high-level CONSTRUCTORS and never enter the strip path.
- **O1 RESOLVED — depth-≥2 tr at the wire level: YES** (no taptree-depth refusal; `validateTapScriptTree` `md.go:1005` checks only permitted-leaf types; the only cap is the general recursion bound `maxDecodeDepth` `md.go:331`). The #953 render defect blocks only off-device recovery.
- **Net-new is small (slice 1 ≈ 60–90 LOC):** `WalletDescriptorTemplateId` (a thin composition over the existing byte-faithful `writeNode`/`writeUseSitePath`/varint/sha256), `isWalletPolicy()` (`= d.tlv.pubPresent`), the form-aware stub selector + the `verify.go` rewire.
- **Version skew (v0.36→v0.37) is ORTHOGONAL.** The only codec-layer 0.37 gap is ~40 LOC of decode-hardening (`validate_use_site_overrides_canonical` — reject `@0`/redundant use-site overrides), defense-in-depth vs adversarial wire, not load-bearing. OPTIONAL ride-along, not a prerequisite. The v0.37 derive-/address-layer work is N/A (the fork doesn't port rust-miniscript). [Per the Rust-primary rule, this hardening already exists in Rust → porting it to Go is compliant.]

---

## Design

### S1 — Template = strip-keys transform (headless)
A new `md` helper produces the keyless template md1 from a decoded full descriptor by applying the toolkit's mutations: `tlv.pubkeys = nil`, `tlv.fingerprints = nil`, elide origin path, then re-encode. Tree-agnostic (single-sig, multisig, general miniscript all strip identically — the tree + use-site ride through unmutated). **Acceptance: byte-identical to `toolkit bundle --md1-form=template`** for pinned vectors. (The fork already decodes keyless md1 — `md/md.go:1073-1076` "template-only mode" — so decode/round-trip is present; this adds the EMIT side as a strip, NOT a per-shape encoder.)

### S2 — Go ports: `WalletDescriptorTemplateId` + form-aware stub (headless, security-load-bearing)
- `md.WalletDescriptorTemplateId(d)` = `SHA-256(use_site_path ‖ tree::write_node ‖ UseSitePathOverrides-TLV)[0..16]` — mirror Rust `identity.rs:71-104`; key-independent, origin-invariant, well-defined for single-sig AND multisig (`MultiKeys{k,N,indices}`), distinct per (script,k,N,use-site).
- Form-aware stub selector: `is_wallet_policy() ? md.WalletPolicyId : md.WalletDescriptorTemplateId`, top-4. `is_wallet_policy()` = `d.tlv.pubPresent`.
- **Acceptance: byte-exact vs `mk-cli derive_stub_from_md1`** — e.g. a keyless `wsh(sortedmulti(2,@0,@1,@2))` template stub roots on `b02b4403…`; a keyed policy on its `WalletPolicyId`.
- **R0 pin #1 (no-canonicalize):** the WDT-Id port MUST mirror `WalletPolicyId`'s `writeNode`-based preimage (`md/walletpolicyid.go:30-64`) but **WITHOUT the `canonicalize(d)` call at `:32`** — Rust `compute_wallet_descriptor_template_id` does NOT canonicalize (unlike `compute_wallet_policy_id`). Serialize the tree as-decoded, relying on the decode-side canonical invariant. (Copying `WalletPolicyId` wholesale, incl. canonicalize, is the easy-to-make error that would diverge from Rust on a non-canonical/author-built AST.)
- **R0 pin #2 (kiw / n guard):** use the consistent key-count (`pathDecl.n` == `d.n`) and keep the `errPathDeclNMismatch` guard (already present at `encode.go:401`) so a kiw mismatch can't silently corrupt the template-id bitstream.

### S3 — Form-aware verify binding
Rewire `bundle/verify.go:116` (today unconditionally `WalletPolicyId`-derived → would FAIL a template) to the form-aware stub. An engraved **template** bundle's N keyless cosigner mk1 cards each root on the one `WalletDescriptorTemplateId`. **Acceptance:** an engraved template bundle verifies; a full bundle still verifies; a wrong/foreign mk1 fails the template-stable binding (the security test). Do NOT widen the multisig derive-leg gate `allSlotsHaveXpub` (`gui/multisig_supply.go:72`) — route templates around it via `expandTemplateOnly` (`gui/md1_expand.go`).

### S4 — GUI: default-full, opt-in template (DD5) + warning
Inner ChoiceScreen on the existing `engraveSingleSig`/`engraveMultisig` programs (no new `program` → no `gui/gui.go:164` guard trip). Default lands on full-policy. Selecting template-only shows the warning + estimate, then engraves the stripped template + the N keyless cosigner mk1 stubs.

```
 Engrave wallet policy
 ─────────────────────
> Full policy md1     (recommended)     ← default
  Template-only md1   ⚠ fewer plates

 ── select Template-only ⇒ ──────────────────
 ⚠  TEMPLATE-ONLY md1  (advanced)
 ────────────────────────────────────────────
 Omits keys → ~1 plate (vs ~2–3).
 The md1 ALONE cannot rebuild your wallet:
 you ALSO need the cosigner key cards (mk1),
 and recovery may need an off-device key search.
 Recovery search (off-device, toolkit):
   sortedmulti (usual) → NONE (order-invariant)
   ordered multi / N! :  N=5 ≈ <1s · N=9 ≈ 2.5s · N=12 ≈ ~55min
 github.com/bg002h/mnemonic-toolkit
 [Back → Full policy]      [I understand → Engrave]
```

### S5 — Taproot depth gate (DD6)
- **depth-1 tr template:** normal path (subject to S1–S4).
- **depth-≥2 tr template:** a SECOND, louder gate. **Wire-level encode/bind is CONFIRMED (O1 resolved — no taptree-depth refusal; `encodePayload` shape-general; tree serialization byte-faithful).** Engrave it behind:
```
 ⚠⚠  EXPERIMENTAL — taproot depth-≥2 template
 ─────────────────────────────────────────────
 The SHIPPED toolkit CANNOT reconstruct this
 taptree (rust-miniscript taptree-display bug,
 PR #953). Recovery currently requires an
 UNRELEASED rust-miniscript >13.1.0.
 DO NOT use for real funds until that ships.
 [Back]                 [I accept the risk → Engrave]
```
- **REFUSED with a clear message:** `tr(sortedmulti_a)`, `sortedmulti`-in-combinator (md-codec/crates.io rust-miniscript lacks `sortedmulti_a`); hardened use-site (at derive/address, `HardenedPublicDerivation`).

### S6 — Recovery-time estimate (the `seedhammer-template-engrave-key-search-time-estimate` FOLLOWUP, folded in)
Display-only at the template opt-in (S4). Honest model: **`sortedmulti` (the common case) → NO search** (order-invariant); **ordered `multi` / distinct-origin slots → N! search** at ~**6.9 µs/permutation** (policyID) / ~**7.4 µs/permutation** (first address) — the off-device toolkit engine. Show a small N→time table; link the toolkit repo. No on-device search.

---

## Invariants
1. **Default unchanged:** with no opt-in, the device engraves the full-policy md1 and verifies it exactly as on `39cb5cf` (byte/behaviour-identical; regression-pinned).
2. **Strip fidelity:** the on-device template md1 is byte-identical to `toolkit bundle --md1-form=template` for every admissible shape tested.
3. **Form-aware binding:** a template bundle verifies iff its mk1 cards root on the template's `WalletDescriptorTemplateId`; a full bundle iff on its `WalletPolicyId`; the two id spaces never cross-validate.
4. **Refusal correctness:** `tr(sortedmulti_a)` / `sortedmulti`-in-combinator / hardened-use-site are refused at the correct layer with a clear message; never silently engraved.
5. **Experimental gate:** a depth-≥2 tr template is engraved ONLY after the second loud warning; default/depth-1 paths never show it.

## Acceptance tests (TDD — MUST fail on `39cb5cf`, pass after)
- **Strip golden:** on-device strip of a full md1 → byte-identical to `toolkit bundle --md1-form=template`, for single-sig, `wsh(sortedmulti)`, AND the §5 general degrading-miniscript wallet (the 11-key example as a fixture).
- **Stub golden:** `md.WalletDescriptorTemplateId` byte-matches Rust (`b02b4403…` for the wsh-sortedmulti template); form-aware selector picks the right id per `pubPresent`.
- **Form-aware verify (security):** engraved template bundle verifies; full bundle verifies; foreign/wrong mk1 → template-stable binding mismatch FAIL. (Fails today: `verify.go:116` is `WalletPolicyId`-only → a template bundle would mis-bind.)
- **Default regression:** full-policy engrave + verify byte/behaviour-identical to `39cb5cf` (golden pin).
- **Refusals:** `tr(sortedmulti_a)`, `sortedmulti`-in-combinator, hardened-use-site rejected with the right message at the right layer.
- **GUI:** default lands on full-policy; template opt-in shows the warning + estimate (assert load-bearing strings); depth-≥2 shows the experimental warning naming ">13.1.0 / PR #953".

---

## Scope, caller fan-out, surface
- **Files (anticipated):** `md/` (new `WalletDescriptorTemplateId` + strip helper + form-aware stub; mirror `md/walletpolicyid.go`), `bundle/verify.go` (form-aware binding), `gui/` (the ChoiceScreen + warnings + estimate on `engraveSingleSig`/`engraveMultisig`; reuse `expandTemplateOnly`/`md1_expand.go`, the verbatim engrave `mdmkFlow`). Tests in each.
- **No new `program`** (extend existing ChoiceScreens → no `gui/gui.go:164` guard trip). No `me`/CLI/schema/docs surface. **No md-codec/mk-codec constellation change** (golden-lock targets only).
- **OUT of scope (deferred):** the broad on-device `to_miniscript.rs`-equivalent renderer → FOLLOWUP `seedhammer-broad-miniscript-renderer` (DD7: this cycle ships the safe-summary display for shapes the fork can't yet draw). **OPTIONAL ride-along (not a prerequisite):** the v0.36→0.37 decode-hardening (~40 LOC `validate_use_site_overrides_canonical`; pre-exists in Rust → Rust-primary-compliant) — fold in only if cheap; defer otherwise.
- **SemVer:** fork firmware MINOR (additive; default unchanged).
- **Secret hygiene / TinyGo:** template emit handles only PUBLIC data (xpubs/template); no new secret path. TinyGo device build is the final integration gate.

## Risks
1. **Strip ≠ toolkit byte-for-byte (TOP).** The strip must reproduce the toolkit's exact mutations (null pubkeys/fp, origin elision) and re-encode identically. Mitigation: golden-lock to `toolkit bundle --md1-form=template` across single-sig/multisig/general fixtures; the §5 11-key wallet is the stress vector.
2. **Form-aware stub mis-binding (SECURITY).** An off-by-one → an mk1 binds the wrong policy / a template won't verify. Mitigation: byte-exact golden vs Rust `derive_stub_from_md1`; the foreign-mk1 negative test; heaviest R0 focus.
3. **Depth-≥2 tr wire capability — RESOLVED (2026-06-20 investigation, O1).** The fork's md-codec has no taptree-depth refusal (only the general `maxDecodeDepth` recursion bound); a depth-≥2 taptree decodes/re-encodes/binds at the wire byte-faithfully via `encodePayload`. DD6's "engrave behind warning" path is feasible; the only blocked thing is off-device recovery via shipped rust-miniscript (hence the experimental framing). No longer a risk.
4. **Engraving an unrecoverable backup.** A depth-≥2 tr template can't be reconstructed with shipped tooling. Mitigation: the loud experimental warning + not-for-real-funds framing (DD6); informed-consent gate.
5. **Default-path regression.** The form-aware verify rewire touches the shared `bundle/verify.go`. Mitigation: byte/behaviour-identical golden pin for the full-policy path; the full-policy verify must be unchanged.

## Gate
- **R0 (mandatory):** opus architect review of THIS spec → fold → persist verbatim to `design/agent-reports/` → re-dispatch after every fold → converge to **0C/0I** before any plan/code.
- **Open items for R0 to rule on:**
  - **O1 — RESOLVED (2026-06-20 investigation; was CRITICAL).** YES — the fork encodes/decodes/round-trips + can compute `WalletDescriptorTemplateId` for a depth-≥2 tr taptree at the WIRE level (no taptree-depth refusal; `encodePayload` is shape-general; tree serialization byte-faithful vs Rust `tree::write_node`). The #953 render defect blocks only off-device recovery, not the on-device wire codec. R0 should RATIFY the two derived pins (S2: no-canonicalize on WDT-Id; keep the `pathDecl.n` guard) rather than re-litigate feasibility. Evidence: `design/md-codec-readiness-verdict-seedhammer-template-engrave.md`.
  - **O2.** Confirm the strip transform reproduces the toolkit's `synthesize_template_descriptor` mutations exactly (origin elision semantics for divergent-origin multisig); golden-lock.
  - **O3.** Confirm `is_wallet_policy()` = `pubPresent` is the exact Go equivalent of the Rust predicate for all admissible shapes.
  - **O4.** Ratify the recovery-estimate model (sortedmulti→none; ordered/distinct→N!; 6.9/7.4 µs) and the displayed N→time table values.
  - **O5.** Confirm the template opt-in is the right GUI placement and the full-policy default is truly unchanged (regression pin).
- **Per-test acceptance:** each acceptance test FAILS on `39cb5cf` then PASSES after; the form-aware-verify security test + the strip golden are load-bearing.
- **Implementation:** single subagent, TDD, in a worktree (no parallel re-implementations). Stage paths explicitly. Then the mandatory whole-diff adversarial exec review.
