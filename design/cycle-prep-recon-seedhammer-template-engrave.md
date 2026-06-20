# cycle-prep recon — 2026-06-20 — seedhammer-template-engrave (single-sig + multisig)

**Design repo (mnemonic-engrave):** branch `master`, HEAD ~`02c4c1a` (3 per-stream checkpoints committed).
**Supersedes:** `cycle-prep-recon-constellation-template-only-engraving.md` (was single-sig-only because the constellation was; now multisig shipped too).
**Multi-repo recon** — primary-source SHAs (3 parallel streams, each persisted to `design/agent-reports/seedhammer-template-engrave-recon-{codec,toolkit,fork}.md`):

| Repo | Path | HEAD | Role |
|---|---|---|---|
| descriptor-mnemonic (md-codec/md-cli) | `/scratch/code/shibboleth/descriptor-mnemonic` | `54dd765` (v0.37.0) | template wire + `WalletDescriptorTemplateId` — **no change needed** |
| mnemonic-key (mk-codec/mk-cli) | `/scratch/code/shibboleth/mnemonic-key` | `1279ef9` | form-aware mk1 stub — **no change needed** |
| mnemonic-toolkit | `/scratch/code/shibboleth/mnemonic-toolkit` | `6de53879` (**v0.60.0**) | template emit + OFF-DEVICE recompose/search |
| seedhammer (fork) | `/scratch/code/shibboleth/seedhammer` | `39cb5cf` | the only unbuilt leg — engrave/bind/verify |

**Verdict: UNBLOCKED for single-sig AND non-taproot multisig; fork delta is MODEST (S–M).** The constellation shipped template engraving end-to-end — single-sig at toolkit **v0.59.0**, multisig/general at **v0.60.0** (#28 phase 2) — with md-codec/mk-codec unchanged (the wire form + `WalletDescriptorTemplateId` + form-aware stub pre-existed). The fork already engraves md1 verbatim and decodes/displays templates; the real work is a small Go codec port + form-aware verify binding + a GUI choice + the recovery-estimate display.

---

## Per-area verification

### Codec layer (md-codec `54dd765`, mk-cli `1279ef9`) — NO change needed; fork must PORT 2 primitives
- **`compute_wallet_descriptor_template_id`** (`md-codec/src/identity.rs:71-104`) = `SHA-256(use_site_path ‖ tree::write_node ‖ UseSitePathOverrides-TLV)[0..16]` — **key-independent + origin-invariant** (no keys/fp/origin/header in the preimage; unit + CLI confirmed), well-defined for single-sig AND multisig (the `MultiKeys{k,N,indices}` body, `tree.rs:115-139`, is hashed), and distinct per (script, k, N, use-site).
- **Keyless template md1 wire** confirmed for `wpkh`, `wsh(multi|sortedmulti)`, `sh(wsh(...))`, `tr(NUMS,multi_a)` (`pubkeys:null`; `is_wallet_policy()` = pubkeys-Some-&-nonempty = false).
- **Form-aware mk1 stub** (`mk-cli .../cmd/mod.rs:72-82`) = `is_wallet_policy() ? WalletPolicyId : WalletDescriptorTemplateId`, top-4 — correct for n≥2 (the N keyless cosigner mk1 cards all root on the one template-stable id).
- **Refused shapes:** `tr(sortedmulti_a)` (rust-miniscript v13 round-trip/render gap) + `sortedmulti`-in-combinator → at the template PARSER; **hardened use-site → at the DERIVE/address path** (`HardenedPublicDerivation`, `error.rs:371-379`), NOT at template encode.

### Toolkit v0.60.0 (`6de53879`) — emit on/near fork; recompose OFF-device
- **EMIT** (`synthesize.rs:1158-1283`, gate `template_admissible:1087-1122`): `bundle --md1-form=template` → keyless md1 + **N keyless cosigner mk1 stubs** (`MkField::Multi`) + per-slot ms1. Arity-split admission: n==1 = pkh/wpkh/tr-keypath; n≥2 = anything that renders + no hardened use-site (incl. `tr(NUMS,multi_a)`); refuses `tr(sortedmulti_a)` / sortedmulti-in-combinator / hardened use-site.
- **BIND** (`:1203-1209`): stub = top-4 of `WalletDescriptorTemplateId`; per-cosigner csi = `top20(stub) ^ slot`.
- **RECOMPOSE (off-device):** `restore --md1 <keyless template> --from <seed> --cosigner <mk1>…` → `complete_multisig_template` (`restore.rs:1416`) — a **parallel n! permutation search** (`permutation_search.rs`, 20-thread cap) with id-search (`--expect-wallet-id`), address-search, or explicit `@N=`; **no-match → refuse (never silent-wrong).** `verify-bundle` uses the SAME engine. This is the engine the 6.9/7.4 µs benchmark measures.
- **Division of labor (confirmed):** the fork EMITS/BINDS/VERIFIES the engraved bundle + displays the recovery estimate; the toolkit runs the recompose/search OFF-device.

### Fork (`39cb5cf`) — modest delta (load-bearing surprise: most of the engrave path already exists)
- **Gaps:** no `WalletDescriptorTemplateId` (only key-dependent `md.WalletPolicyId`); both md1 encoders force `pubPresent:true`; `bundle/verify.go:116` stub binding is unconditionally `WalletPolicyId`-derived (wrong for a keyless template).
- **Already present:** the fork DECODES keyless md1 (`md/md.go:1073-1076` "template-only mode"), EXPANDS/DISPLAYS templates (`expandTemplateOnly`, `gui/md1_expand.go:18,42-49`), and **engraves any scanned md1 verbatim with NO xpub gate** (`mdmkFlow`, `gui/gui.go:1972-2027`). The `allSlotsHaveXpub` refusal (`gui/multisig_supply.go:72`) is ONLY on the multisig **derive-leg** path (`gui/multisig.go:83`), not a global engrave gate.
- **What to change:** (1) port `md.WalletDescriptorTemplateId` (mirror `md/walletpolicyid.go`) + a Go `is_wallet_policy()` = `d.tlv.pubPresent` + a form-aware binding-stub helper rewired into `bundle/verify.go:116`; (2) route templates around `allSlotsHaveXpub` via `expandTemplateOnly` (do NOT widen the gate); (3) a "full vs template" inner ChoiceScreen on the existing `engraveSingleSig`/`engraveMultisig` programs (no new program → no `gui/gui.go:164` guard trip).
- **Plate cost:** 1 plate per chunk (`gui/bundle_flow.go:301-318`); template (Pubkeys TLV stripped) ≈ 1 plate vs keyed ≈ 2-3.

---

## Design decisions for the brainstorm (recommendations; LOCK at brainstorm/R0)

- **DD1 — Template md1 SOURCE = INGEST-supplied, not on-device emit.** The user produces the template off-device (`toolkit bundle --md1-form=template`) and the fork engraves it verbatim (the fork already does verbatim md1 engrave). Avoids a net-new keyless Go encoder (both encoders force `pubPresent:true`). Applies to single-sig AND multisig. (Both fork+toolkit streams concur.)
- **DD2 — NO on-device permutation search.** The fork is an air-gapped ENGRAVING device (slow RP2350, no recovery role). It engraves the template bundle + binds (template-stable stub) + VERIFIES the engraved readback + **DISPLAYS the recovery-time estimate** (the `seedhammer-template-engrave-key-search-time-estimate` FOLLOWUP, now UNBLOCKED). The off-device toolkit owns the recompose/search.
- **DD3 — Scope = NON-TAPROOT.** Single-sig (incl. taproot single-sig `tr(@N)`) + `wsh/sh(wsh)` multisig. **Taproot `sortedmulti_a` multisig is constellation-refused** (rust-miniscript v13 gap) → fork inherits the refusal (out of scope; revisit when rust-miniscript >13.1.0 lands). Mirror `tr(sortedmulti_a)`/sortedmulti-in-combinator refusal at the template parser; mirror hardened-use-site refusal at the **derive/address** path (not the template parser) — a nuance the fork must get right.
- **DD4 — The form-aware verify binding is the SECURITY-load-bearing piece.** The fork's on-device verify currently binds via the key-dependent `WalletPolicyId`; a template bundle binds via `WalletDescriptorTemplateId`. The port must be byte-exact vs the Rust `derive_stub_from_md1` (an off-by-one → mk1↔md1 mis-binding). This slice carries the heaviest R0 emphasis + a golden cross-check vs the toolkit.

---

## Cross-cutting observations
1. **Scope narrowed to a pure fork cycle** — md-codec + mk-codec + mk-cli + toolkit are ALL done (v0.60.0); no constellation companion work. Pin md-codec `54dd765` / mk-cli `1279ef9` / toolkit `6de53879`.
2. **The engrave half is largely free** (the fork already engraves verbatim md1 + decodes/displays templates) — the cycle's substance is the form-aware VERIFY binding + the GUI choice + the estimate display, not new engrave geometry.
3. **Hardened-use-site is a derive-time guard, not a template-encode refusal** — easy to misplace; the fork mirrors it on its derive/address path.
4. **Firmware-only; no `me`/CLI/`me-preview`/schema/docs-mirror surface; no new `program`** (extends existing ChoiceScreens). SemVer: fork firmware MINOR (additive capability).
5. **Companion FOLLOWUP** `seedhammer-template-engrave-key-search-time-estimate` is the DD2 display feature — fold it into this cycle (or as a paired slice). `mstar-prepolicy-key-backup` (the unbound-key-card design question) is RELATED but separate (still open, leaning-(a)).

---

## Recommended brainstorm-session scope
- **One fork firmware cycle:** `seedhammer-template-engrave` (single-sig + non-taproot multisig). Sizing **M**.
- **Slices:**
  1. **Headless codec port (S–M, heaviest R0):** Go `md.WalletDescriptorTemplateId` + `is_wallet_policy()` + form-aware binding stub; rewire `bundle/verify.go` binding to be form-aware. Golden-locked byte-exact vs Rust `derive_stub_from_md1` (template-id stub on a known keyless md1, e.g. wsh-sortedmulti `b02b4403…`). Security-load-bearing.
  2. **Single-sig template engrave + verify (S):** ingest a supplied single-sig template md1 → engrave (via the existing verbatim path) → form-aware verify; route around `allSlotsHaveXpub` via `expandTemplateOnly`.
  3. **Multisig template engrave + verify (S–M):** N keyless cosigner mk1 stubs (one template-stable stub each), `wsh/sh(wsh)` only; the multisig choose-or-supply ChoiceScreen gains a "template" option.
  4. **Recovery-time estimate display (S):** the key-search-time-estimate UI (display-only; off-device search; the 6.9/7.4 µs benchmark) — fold the FOLLOWUP here.
- **GUI choice:** inner ChoiceScreen on `engraveSingleSig`/`engraveMultisig` (no new program).
- **Lockstep / SemVer:** N/A for `me`/GUI-schema/docs-manual (this is fork firmware). MINOR (additive). Taproot-multisig out (rust-miniscript dep).
- **Gate reminder (CLAUDE.md):** brainstorm SPEC + IMPLEMENTATION_PLAN each pass an opus architect R0 to 0C/0I before any code (folds persisted to `design/agent-reports/`); single-implementer TDD in a worktree; mandatory whole-diff exec review. Slice 1's form-aware stub port is the heaviest R0 target (golden-locked, security binding).
