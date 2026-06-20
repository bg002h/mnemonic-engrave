# SeedHammer fork Go `md-codec` sync-skew investigation

Status: COMPLETE (verdict at bottom). READ-ONLY investigation; no source modified.

## Sync block

| Repo | Role | Path | HEAD |
|---|---|---|---|
| seedhammer (fork, Go/TinyGo) | port under investigation | `/scratch/code/shibboleth/seedhammer` | `39cb5cf6351f6c87ab779e5688a60186f65412e0` |
| descriptor-mnemonic (Rust reference) | upstream md-codec | `/scratch/code/shibboleth/descriptor-mnemonic` | `54dd765a11d490dc3d8dec2c842dae718bd3ef2b` (md-codec v0.37.0) |

The fork's Go `md` package lives at `/scratch/code/shibboleth/seedhammer/md/`.

## Port provenance (which Rust version the fork tracks)

Explicit version markers in the fork's `md/` package:

- `md/bits.go:3` — `// format: descriptor-mnemonic/crates/md-codec @ 0.36.0 (decode_md1_string path).`
- `md/md_test.go:77` — payloads "built white-box from the verified md-codec **0.36.0** wire layout (the same layout this package [encodes])".
- `md/md_test.go:327` — "All strings below were produced by the md-codec **0.36.0** encoder".

No `PORT.md` / `PROVENANCE` file exists. Conclusion: **the fork tracks Rust md-codec v0.36.0** (2026-06-15). The single open upstream release is **v0.37.0** (2026-06-19).

Note: the fork's port scope is the **encode + decode (md1) + identity (policy/template id)** surface only. It does NOT port the address-derivation / `to_miniscript` / miniscript-backed layer (the fork has no rust-miniscript equivalent). This matters for classifying 0.37.0 below.

---

## Feature delta 0.36.0 -> 0.37.0 (the only open release)

The fork tracks v0.36.0. There is exactly ONE published release ahead of it: **v0.37.0** (2026-06-19). The Rust git log between the 0.36.0 release commit (`a3f9d8f`) and the 0.37.0 release commit (`54dd765`) is just two non-release commits touching `crates/md-codec/`:

- `c85cd49` — `test(bch): NUMS drift-guard for MD_REGULAR_CONST + document POLYMOD_INIT`. **NO-BUMP, test+doc only** (`bch.rs` +40, all `#[test]` / comments). No wire/API change. → **N/A** (the fork's BCH constants are unchanged; nothing to port).
- `0bd9088` — `feat(md-codec): faithful per-cosigner use-site override reconstruction + hardened-anywhere guard`. The substance of 0.37.0. Files: `decode.rs +7`, `derive.rs ±16`, `error.rs +22`, `lib.rs +4`, `to_miniscript.rs +183`, `validate.rs +39`, + two test files.

### What the fork's `md` package DOES port (scope)

Encode + decode (md1 single-string & chunked), canonicalize, expand, the 5 post-decode validators, identity (`computeEncodingID` = chunk-set-id; `WalletPolicyId` = `compute_wallet_policy_id`, identity.rs:172-240). It already decodes ALL FOUR TLV tags including `use_site_path_overrides` (tag 0x00). It does NOT port the derive/`to_miniscript`/miniscript-backed address layer (no rust-miniscript equivalent on TinyGo).

### TLV tag space — UNCHANGED across 0.36 -> 0.37

No new TLV tag, no new wire field in 0.37.0. Rust `tlv.rs:11-19`: `USE_SITE_PATH_OVERRIDES=0x00`, `FINGERPRINTS=0x01`, `PUBKEYS=0x02`, `ORIGIN_PATH_OVERRIDES=0x03`. Fork `md/md.go:495-498`: byte-identical (`tlvUseSitePathOverrides=0x00 … tlvOriginPathOverrides=0x03`). The `UseSitePathOverrides` TLV itself is NOT a 0.37 addition — it landed at v0.11 (`215ac02 feat(v0.11): TLV section with UseSitePathOverrides and Fingerprints`) and the fork already decodes it (`md/md.go`, `md/expand.go:167`, `md/canonicalize.go:343`, `md/walletpolicyid.go:173-184 resolveUseSiteRaw`).

### Per-feature classification (0.37.0 items)

| # | 0.37.0 item (commit 0bd9088 unless noted) | Layer | Fork status | Evidence |
|---|---|---|---|---|
| 1 | **D5(a) decode-hardening: reject `@0` use-site override** (`Error::BaselineUseSiteOverride`) via `validate_use_site_overrides_canonical`, wired in `decode_payload` | codec / decode (IN SCOPE) | **MISSING** | Rust: `validate.rs:147-178` + `decode.rs:59-65` + `error.rs:189-198`. Fork `decodePayloadValidated` (`md/md.go:1133-1160`) runs only `validatePlaceholderUsage` + `validateMultipathConsistency` + tap-leaf + explicit-origin + xpub-bytes — NO canonical-override check. No `errBaselineUseSiteOverride` sentinel (`md/md.go:19-31, 893-899`). |
| 2 | **D5(a) decode-hardening: reject redundant override == baseline** (`Error::RedundantUseSiteOverride`) | codec / decode (IN SCOPE) | **MISSING** | Same as #1: `validate.rs:147-178` + `error.rs:200-209`. No `errRedundantUseSiteOverride` in the fork; the check is not invoked. |
| 3 | **D5(b): a `Some`-baseline + `None`-override mix is LEGAL (not `MultipathAltCountMismatch`)** — 0.37 only ADDED a doc-comment clarifying existing behavior; no code change | codec / validate (IN SCOPE) | **PRESENT** (behaviorally) | Fork `validateMultipathConsistency` (`md/md.go:976-1001`) already gates on `if p.hasMultipath` and skips `None`-multipath entries — a Some/None mix already passes. Identical to Rust `validate.rs` (the alt-count check is `if let Some(alts)`-guarded). The doc clarification has no porting obligation. |
| 4 | **`pub fn has_hardened_use_site(d)`** — scans baseline AND every override for a hardened wildcard/alt; the single hardened-derivation guard (`HardenedPublicDerivation`) | derive / `to_miniscript.rs` (OUT OF SCOPE) | **N/A (derive-layer)** | Rust `to_miniscript.rs` (new `pub fn`), re-exported `lib.rs:57-59` (gated `#[cfg(feature="derive")]`). The fork ports no derive/`to_miniscript` layer at all. `Error::HardenedPublicDerivation` (`error.rs:376-379`) is a derive-only variant and is **already absent** from the fork's error set (pre-existing; the fork has no `derive_address`). Not a md1-codec porting item. |
| 5 | **`to_miniscript_descriptor` now derives each key at its OWN per-`@N` use-site value (not the shared baseline)** — the funds-safety address fix | derive / `to_miniscript.rs` (OUT OF SCOPE) | **N/A (derive-layer)** | Rust `to_miniscript.rs:*`, `derive.rs ±16`. Address derivation is not in the fork. The fork's `resolveUseSiteRaw` (`md/walletpolicyid.go:173-184`) ALREADY resolves per-`@N` override-over-baseline for its identity-hash preimage, so the fork's *identity* path is not affected by this bug class. |
| 6 | **`to_miniscript_descriptor_multipath(d)`** — new descriptor-STRING builder, per-`@N` multipath group | derive / `to_miniscript.rs` (OUT OF SCOPE) | **N/A (derive-layer)** | Rust new `pub fn`, re-export `lib.rs:58`. No fork analog (no descriptor-string emission in the fork). |
| 7 | `c85cd49` BCH NUMS drift-guard test + POLYMOD_INIT doc | test/doc (NO-BUMP) | **N/A** | `bch.rs +40` test+comment only; fork BCH constants unchanged. |

### Pre-existing gaps (NOT 0.37.0 delta, but flagged load-bearing by the task)

| Item | Rust location | Introduced | Fork status | Evidence |
|---|---|---|---|---|
| **`compute_wallet_descriptor_template_id` / `WalletDescriptorTemplateId`** — the §8.1 *template* id (hashes `use_site_path_decl ‖ tree ‖ UseSitePathOverrides-TLV`; identity-of-the-keyless-template, distinct from `WalletPolicyId` which hashes the canonical-EXPANDED policy incl. per-`@N` origin/fp/xpub) | `identity.rs:71-104` (struct `WalletDescriptorTemplateId`, `compute_wallet_descriptor_template_id`) | **v0.12.0** (`5350f8a`), long pre-0.36 | **MISSING** | Fork ports `computeEncodingID` (`md/identity.go:7-11`, the chunk-set-id) and `WalletPolicyId` (`md/walletpolicyid.go:30`, the expanded policy id) but has NO `TemplateId` / §8.1 surface — grep for `TemplateId\|template_id\|§8.1` in `md/*.go` is empty. This is a **pre-existing port gap**, present in v0.36.0 too — a sync to 0.37.0 would NOT add it. |
| `Error::HardenedPublicDerivation` (the guard variant) | `error.rs:376-379` | pre-0.37 (derive layer) | **MISSING (derive-layer; pre-existing)** | Fork has no derive layer; variant absent (`md/md.go:893-899`). Orthogonal to the md1 codec port. |

---

## Verdict

**How far behind:** The fork's Go `md` package tracks Rust md-codec **v0.36.0**. It is **one MINOR release behind** (v0.37.0). The 0.37.0 substance is overwhelmingly a **derive-/address-layer** fix (`to_miniscript` per-cosigner faithful derivation + the `has_hardened_use_site` guard) that the fork **does not port at all** — so it is N/A, not a gap.

The ONLY in-scope (codec/decode-layer) delta the fork is genuinely missing is the **D5(a) decode-hardening pair**: reject an `@0` use-site override (`BaselineUseSiteOverride`) and a redundant override equal to the baseline (`RedundantUseSiteOverride`). This is a ~40-line, self-contained addition (one new validator fn `validateUseSiteOverridesCanonical` + two error sentinels + one call site in `decodePayloadValidated`, gated on `useSitePresent`). It is **defense-in-depth against hand-crafted/adversarial wire only** — the project's own encoders never emit either shape, so no canonically-produced card changes behavior, and there is no wire-format or TLV-tag break. The fork already exhibits the D5(b) Some/None-mix behavior; TLV tags are byte-identical.

**Is a sync cycle a prerequisite for template engraving? — No, not strictly; recommended scope S.**

- The 0.37.0 delta is **orthogonal** to the three template-engraving load-bearing needs:
  - *keyless md1*: served by the existing template encode/decode path — unchanged in 0.37.
  - *template-stable id*: served by `compute_wallet_descriptor_template_id` (§8.1) — which is **MISSING from the fork, but as a PRE-EXISTING gap dating to v0.12.0, NOT introduced or closed by 0.37.0**. Syncing 0.36→0.37 would NOT deliver it. If template engraving needs a template-stable id, that is a **separate net-new port of `identity.rs:55-104`**, independent of the version-skew sync.
  - *form-aware mk1 stub binding*: served by `WalletPolicyId`/`WalletPolicyIDStub` — already ported (`md/walletpolicyid.go`), unchanged in 0.37.
- The one real in-scope skew item (D5(a)) is an adversarial-wire decode-hardening guard; it does not block, and is not load-bearing for, template engraving. It is good hygiene to fold in but can ride along.

**Recommended action:** Treat the 0.36→0.37 codec sync as **size S** (port D5(a): `validateUseSiteOverridesCanonical` + 2 sentinels + 1 wired call + TDD cells; ~half a day). The derive-layer 0.37 items are permanently N/A for this fork. **The genuinely consequential prerequisite for template engraving is the SEPARATE, pre-existing `WalletDescriptorTemplateId` (§8.1) port — size S-to-M on its own — which the version-skew framing would otherwise hide.** Scope the template-engrave cycle to include that explicitly rather than assuming a "catch up to 0.37" sync covers it.

