# md-codec readiness verdict — seedhammer-template-engrave (2026-06-20)

**Question (user, paused the brainstorm/R0):** "Before we begin, we probably need to update the fork md-codec. Pause here and investigate."

**Verdict: NO separate md-codec update/sync cycle is a prerequisite.** The fork's Go md-codec is already capable enough for the template-engrave cycle's *cryptographically load-bearing* paths (engrave + verify/bind). The only genuine net-new work is the small `WalletDescriptorTemplateId` composition the SPEC already scopes as slice 1, built on serialization primitives that are **already byte-faithful and in production**. The one open item is a **scope decision about on-device DISPLAY breadth**, not a codec deficiency.

Backed by 3 parallel primary-source investigations (verbatim reports in `design/agent-reports/seedhammer-md-codec-sync-investigation-{skew,coverage,primitives}.md`). Pins: fork @ `39cb5cf`; descriptor-mnemonic @ `54dd765` (md-codec v0.37.0); mnemonic-key @ `1279ef9`; mnemonic-toolkit @ `6de53879` (v0.60.0).

---

## Stream A — version skew (fork tracks v0.36.0; one minor behind v0.37.0)
- Provenance marker: `md/bits.go:3` "@ 0.36.0" (+ test markers). No PORT.md.
- The whole 0.37.0 delta is one substantive commit (`0bd9088`); **almost all of it is derive-/address-layer** (`to_miniscript_descriptor`, `HardenedPublicDerivation` guard) — a layer the fork **does not port at all** (no rust-miniscript on TinyGo) → N/A, not gaps.
- Only codec-layer 0.37 gap = `validate_use_site_overrides_canonical` decode-hardening (`@0` / redundant override rejection), ~40 LOC, **defense-in-depth vs adversarial wire only** (canonical encoders never emit those shapes; no wire/TLV break). **Not load-bearing for templates.**
- TLV tags byte-identical (0x00–0x03); no tag additions in 0.37.
- **A 0.36→0.37 sync is NOT a prerequisite.** It is a small, orthogonal hardening (sizing **S**) the cycle does not depend on; fold it in opportunistically or as a tiny separate slice.

## Stream B — shape coverage (three layers, very different coverage)
- **DECODE** (`md/md.go:330-490` `readNodeDepth`): a *complete, faithful* port of Rust `read_node` — all 36 tags, all 8 body variants, every combinator/wrapper/taptree/hashlock/timelock, all validators + depth cap. **Nothing the Rust side admits is missing at decode.**
- **ENCODE**: low-level `writeNode` (`md/encode.go:159-232`) can emit any body, but the only *public* tree-builders are 7 hard-coded shapes (4 single-sig + 3 sortedmulti). No arbitrary-template encoder. **→ Neutralized by DD1** (fork ingests + engraves verbatim; never *builds* an arbitrary template).
- **EXPAND/DISPLAY** (`classifyPolicy` `md/md.go:1266-1315`, `scriptForTemplate` `gui/md1_expand.go:82-121`): *much* narrower than decode. Any `tr` with a script tree and all combinators hard-refuse → `PolicyComplex`; unsorted `multi` summarizes but has no `scriptForTemplate` arm (display-only, never verified). **This is the only layer that limits "any admissible md1" — and only for visual rendering.**

## Stream C — template primitives + tree-encode fidelity (THE load-bearing answer)
- **YES — the fork's Go tree serialization is byte-faithful to Rust `tree::write_node` for ALL shapes.** `writeNode` is already a standalone serializer, already in production behind `WalletPolicyId` (`md/walletpolicyid.go:42`). Tag codes (all 36), `5bits(k-1)|5bits(N-1)` packing, kiw-width indices, child ordering, `Tr{is_nums/has_tree/NUMS-suppression}`, hash/timelock widths — all identical. **So `WalletDescriptorTemplateId` is NOT net-new serialization; it reuses an already-correct serializer. No path to a wrong template-id / mis-bound card.**
- **Net-new primitives (~60–90 LOC total, pure composition over existing byte-faithful pieces):**
  - (a) `WalletDescriptorTemplateId` (~40–60 LOC): `SHA-256(use_site bits ‖ writeNode(tree) ‖ UseSitePathOverrides-TLV)[0:16]` (Rust `identity.rs:71-104`).
  - (b) `isWalletPolicy()` (~1–3 LOC): `d.tlv.pubPresent && len(d.tlv.pubkeys) > 0` (Rust `encode.rs:50-52`). Fields already on `tlvSection`. Confirmed absent today.
  - (c) form-aware selector + rewire `bundle/verify.go:116` (~15–25 LOC): current code unconditionally uses `WalletPolicyIDStubChunks` → would **mis-reject a legitimate keyless-template card binding** (the security-relevant fix). Branch on `isWalletPolicy()` (mirror mk-cli `derive_stub_from_md1`, `mk-cli/src/cmd/mod.rs:72-82`).
- **No on-device strip needed** — DD1 ingest-supplied; toolkit `synthesize_template_descriptor` strips host-side.
- **R0 pins:** (1) `compute_wallet_descriptor_template_id` does **NOT** canonicalize its input (unlike `compute_wallet_policy_id`) — the Go port must hash as-decoded, relying on the decode-side canonical invariant. (2) kiw source: Rust uses `descriptor.n`, Go `WalletPolicyId` uses `pathDecl.n` — equal post-canonicalize (`errPathDeclNMismatch` guard `md/encode.go:401`); keep the guard.

---

## Synthesis — what this means for the cycle

| Path | Coverage for "any admissible md1" | Work needed |
|---|---|---|
| **Engrave** | ✅ complete (verbatim of scanned bytes; DD1) | none |
| **Verify / bind** (template-id over decoded tree) | ✅ complete — tree serialization is byte-faithful for *all* shapes; pure hash, no rendering | the ~60–90 LOC slice-1 primitives (already SPEC'd) |
| **On-device display / expand** | ⚠️ narrow — refuses tr-with-tree + all combinators today | depends on a SCOPE DECISION (below) |

**The decision the investigation exposed:** "any admissible md1" (locked DD3) has two sub-meanings that weren't distinguished when chosen:
- **Engrave + verify breadth = essentially free** (the codec is shape-complete for these).
- **Visual display breadth = expensive** — rendering arbitrary miniscript / depth-≥2 taptrees on-device needs a `to_miniscript.rs`-equivalent renderer the fork lacks (**L**-sized; would become the bulk of the cycle).

→ **OPEN-DECISION-1 — RESOLVED 2026-06-20 (user): option 3 — minimal display now, broad renderer deferred.** Ship engrave+verify for ANY admissible md1 now (the load-bearing part), with full on-device display ONLY for the shapes the fork already renders (single-sig, wsh/sh(wsh) multi & sortedmulti); unrenderable shapes (general miniscript, depth-≥2 taptrees) get a generic safe summary + template-id under the loud experimental warning. File a FOLLOWUP for the broad `to_miniscript.rs`-equivalent renderer + taproot-multisig display. Does not change the verdict above (no prerequisite md-codec cycle).

## Net recommendation
1. **Do not run a separate "update fork md-codec" cycle.** The codec is ready for the load-bearing paths; the v0.36→0.37 skew is a tiny orthogonal hardening (S) that can ride along or wait.
2. **Resume the template-engrave SPEC/R0** with slice 1 (the `WalletDescriptorTemplateId` + `isWalletPolicy()` + form-aware verify rewire) confirmed **small and low-risk** — the hard part (byte-faithful tree serialization) is already done.
3. **Settle OPEN-DECISION-1 (display breadth)** before the SPEC goes to R0, because it swings the cycle between S–M and L.
4. Add the R0 pins (no-canonicalize on WDT-Id; keep the `pathDecl.n` guard) to the SPEC's slice-1 spec.
