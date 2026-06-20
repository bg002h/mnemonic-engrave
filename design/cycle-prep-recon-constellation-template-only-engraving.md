# cycle-prep recon — 2026-06-20 — constellation-template-only-engraving

**Design repo (mnemonic-engrave) SHA at recon time:** `2ea6769` (branch `master`)
**Sync state:** clean working tree; no untracked files at recon start.
**This is a MULTI-REPO recon** (a fork-side firmware cycle that depends on constellation-side changes). Repos verified at PRIMARY SOURCE:

| Repo | Path | Branch | HEAD (short) | Role |
|---|---|---|---|---|
| descriptor-mnemonic (md-codec) | `/scratch/code/shibboleth/descriptor-mnemonic` | `main` | `54dd765` (v0.37.0) | md1 wire format + identities |
| mnemonic-key (mk-codec / mk-cli) | `/scratch/code/shibboleth/mnemonic-key` | `main` | `3258271` (mk-cli v0.10.0) | mk1↔md1 stub binding |
| mnemonic-toolkit | `/scratch/code/shibboleth/mnemonic-toolkit` | `master` | `cbdadbb7` (rel `d72856f1`, v0.59.0) | bundle / synthesize / restore / verify-bundle |
| seedhammer (fork) | `/scratch/code/shibboleth/seedhammer` | `main` | `3a23dbb` | the device that engraves; vendors Go ports of md/mk |
| rust-miniscript-template-accessor | `/scratch/code/shibboleth/rust-miniscript-template-accessor` | `feat/wallet-policy-template-accessor` | `489e799` | **UNUSED red herring** (not wired into md-codec) |

Slug verified: `constellation-template-only-engraving`. **Verdict: the constellation BLOCKER IS RESOLVED for the SINGLE-SIG form** (toolkit v0.59.0 + mk-cli v0.10.0, both 2026-06-19) — so this fork-side cycle is now UNBLOCKED. The FOLLOWUP body is materially STALE (it describes the pre-resolution world) and one of its premises (recompose-from-template+key-card) was already corrected by the prior recon.

---

## Per-slug verification

### `constellation-template-only-engraving`

**WHAT:** Let the SeedHammer user CHOOSE, at engrave time, between a full md1 wallet policy (keys embedded) and a wallet-policy TEMPLATE md1 (`pubkeys:null`, script+origin+use-site only), to cut engraving-plate count (~1 plate template vs ~3 full single-sig). The watch-only wallet is later recomposed from the template + the seed (and bound by the mk1 key card).

**Citations / claims (each re-checked against PRIMARY SOURCE at the SHAs above):**

- **(a)** "constellation TODAY emits ONLY full-policy md1 in bundles (template unused/refused)" — **STRUCTURALLY-WRONG NOW (drifted, was true at the cited recon's `toolkit@f7e6fca`).** Resolved by `mnemonic-toolkit` `b0bad50e`/`d72856f1` (v0.59.0, 2026-06-19): `bundle --md1-form=policy|template` (clap arg `bundle.rs:169-170`, default `Policy`; enum `synthesize.rs:53-60`; routing `synthesize.rs:355-356` → `synthesize_template_descriptor` `:981-1043`). Default bundle is still full-policy; template is now an explicit opt-in.

- **(b)** "mk1↔md1 stub = top-4 of the KEY-DEPENDENT `WalletPolicyId`" — **DRIFTED → now FORM-AWARE.** mk-cli `3258271` (v0.10.0, 2026-06-19) `derive_stub_from_md1` (`mnemonic-key/crates/mk-cli/src/cmd/mod.rs:72-82`): `if descriptor.is_wallet_policy() { WalletPolicyId } else { WalletDescriptorTemplateId }`, top-4. Toolkit uses the identical discriminator (`bundle.rs:1151-1160` `bundle_binding_stub`, "single source of truth"). Pre-#28 it WAS the unconditional `WalletPolicyId` (`93eba96`, v0.8.0) — exactly the FOLLOWUP's claim, now superseded.

- **(c)** "a template md1 and the full-policy md1 of the same wallet hash to DIFFERENT ids → would BREAK the stub binding" — **ACCURATE as stated, but the RESOLUTION differs from the FOLLOWUP's implied fix.** TemplateId ≠ PolicyId still holds (md-codec `identity.rs` test `walletpolicyid_template_only_differs_from_full_cell_7`; Go fork mirror `md/walletpolicyid_test.go:120-139 TestWalletPolicyIdPresenceSignificant`). The constellation did NOT make "one stub bind both forms"; instead the mk1 stub is derived from **whichever form is actually engraved** (form-aware), so a template card and a policy card are each internally coherent. **An mk1 binds to the form that was engraved, not to "either form of the same wallet."**

- **(d)** template-stable id named `WalletDescriptorTemplateId` at `identity.rs:71-104` — **ACCURATE (exact at `descriptor-mnemonic@54dd765`).** `compute_wallet_descriptor_template_id` hashes only use-site-path + tree bits + the use-site-override TLV; key/origin/fingerprint-invariant (doc `identity.rs:47-53`). Key-dependent sibling `compute_wallet_policy_id` at `identity.rs:172-240` (presence-byte `fp|xpub<<1` folded in, `:217-228`). **This primitive predates the cycle (public since md-codec v0.34.0) — NO md-codec change was needed; the FOLLOWUP's pointer is correct and current.**

- **(e)** "needs a coordinated change across md-codec + mk-codec + the toolkit + SH" — **OVERSTATED / now PARTIALLY-DONE.** Reality: md-codec needed **nothing** (template-id + template encode pre-existed); mk-**cli** + **toolkit** shipped the change 2026-06-19 (no mk-codec or md-codec crate bump — both releases note "NO-BUMP"). The ONLY remaining leg is **SH (the fork)**. So the "coordinated 4-repo change" is really "toolkit+mk-cli (done) → fork (this cycle)."

- **(f)** "T6 is full-policy-only (user-confirmed); revisit only AFTER the constellation adopts it" — **ACCURATE + the precondition is now MET.** Confirmed scope-out in `design/SPEC_seedhammer_T6a_singlesig_flagship.md:39,77` ("Template-only md1 … → OUT, full-policy only … revisit … after constellation adopts"). The constellation has now adopted it (single-sig) → this cycle is the sanctioned "revisit."

- **(g)** "recomposable from template + the key card (mk1, public) OR template + ms1 (secret)" — **STRUCTURALLY-WRONG (corrected by the prior recon's Q3 and confirmed here).** The watch-only wallet is recomposed from **template md1 + SEED** via `restore … --from <seed>` (`mnemonic-toolkit/.../restore.rs:207-217` gate → `run_singlesig_template_completion` `:671-861`, which REQUIRES `--from <seed>` `:685`). The **mk1 is the binding/verification card, not a recompose input**; `verify-bundle` reconstructs and compares under the template-stable stub (`verify_bundle.rs:310-319,478-582`). "template + mk1" alone does NOT yield keys.

**Crucial scope fact (NEW, not in the FOLLOWUP):** the constellation's template form is **SINGLE-SIG ONLY**. `synthesize_template_descriptor` refuses `descriptor.n != 1` → `TemplateFormUnsupportedShape` (`synthesize.rs:987-994`) and refuses a nested-multi 1-of-1 (`:1005-1012`, commit `d8b6ecaa`). Multisig `restore` still REQUIRES a full-policy md1 (`restore.rs:1655-1661`). **A fork template cycle MUST therefore be single-sig-only to stay mirror-faithful; offering a multisig template would diverge from (and be unverifiable against) the constellation.**

**Action for brainstorm spec:**
1. **Scope the cycle to SINGLE-SIG templates only** (mirror the constellation; multisig template is constellation-refused = out). The fork's single-sig wallet-policy engrave path (`gui/singlesig*.go`, T6a flagship) is the target — NOT the multisig path (`gui/multisig*.go`), which stays full-policy-only.
2. **Port two Go primitives** from the constellation (golden-locked):
   - `md.WalletDescriptorTemplateId` ≡ Rust `identity.rs:71-104` @ `descriptor-mnemonic 54dd765` (the fork currently has only the presence-significant `md.WalletPolicyId`).
   - a **form-aware stub**: `is_wallet_policy() ? WalletPolicyId : WalletDescriptorTemplateId`, top-4 — mirror mk-cli `mod.rs:72-82` / toolkit `bundle.rs:1151-1160` @ `mnemonic-key 3258271` / `mnemonic-toolkit d72856f1`. This rewires `bundle/verify.go:108 checkStubBinding`.
3. **Decide the template-md1 SOURCE (key design question):** does the device (a) only **ingest** a user-supplied template md1 (produced by `toolkit bundle --md1-form=template`) — then the fork needs NO template encoder, just relax the `allSlotsHaveXpub` refusal (`gui/multisig_supply.go:36-50`, but on the single-sig path) + the form-aware stub; or (b) also **emit** a template on-device — then the fork needs a net-new pubkeys-null Go encoder (`md/encode_singlesig.go` forces `pubPresent:true` today). Option (a) is far smaller/lower-risk and matches the supply-path philosophy.
4. **GUI:** add a "full md1 vs template md1" ChoiceScreen branch on the single-sig engrave flow — **no new `program`** (extend the existing flow; avoids the `gui/gui.go:164` program-count guard), per the established pattern.
5. Cite source SHAs: `descriptor-mnemonic 54dd765`, `mnemonic-key 3258271`, `mnemonic-toolkit d72856f1` (v0.59.0), fork base `seedhammer 3a23dbb`. Golden anchors: the ported `md.WalletPolicyId` golden `6650b980 3b3c6621 0140540d a8d765a0` (from T6a) must coexist with the new TemplateId golden.

---

## Cross-cutting observations

1. **Both FOLLOWUP entries are STALE and should be flipped.** `mnemonic-engrave/design/FOLLOWUPS.md:25` (`constellation-template-only-engraving`) and its sibling `mnemonic-toolkit/design/FOLLOWUPS.md:29` (`bundle-md1-template-only-option`) both still read as "constellation is the blocker / open" — but the constellation IMPLEMENTED the single-sig template the SAME DAY it filed the toolkit FOLLOWUP (`mnemonic-toolkit 4e21d94d` filed it; `b0bad50e`/`d72856f1` implemented it). The mnemonic-engrave entry should be updated to "constellation adopted (single-sig) 2026-06-19; fork cycle now actionable, single-sig-only."

2. **The `rust-miniscript-template-accessor` fork is a red herring.** descriptor-mnemonic depends on crates.io `miniscript 13.0.0` (`Cargo.lock:518-521`), with NO `[patch]`/`git`/`path` redirect to either local miniscript fork. The template capability is **native md-codec code**, not delegated. Any brainstorm assuming that fork supplies template extraction is wrong; do not pin or build against it.

3. **The constellation change was SMALLER than the FOLLOWUP feared** — no md-codec or mk-codec crate bump (the `WalletDescriptorTemplateId` primitive and template encode pre-existed since v0.34.0); only toolkit + mk-**cli** derivation/discrimination logic. So the fork's port is bounded: one identity function + one form-aware branch, both already byte-specified by stable Rust source and tests.

4. **The recompose semantics are template + SEED, not template + key card** (claim g corrected). This reshapes the user-facing story: a template plate is only "completable" by the holder of the seed; the mk1 card binds/verifies the template form. The brainstorm should state this plainly so the on-device warning/labelling is accurate (don't imply a template plate alone, or template+mk1, reconstitutes keys).

5. **Stale source-comment caveat (constellation-side, informational):** `mnemonic-key/.../key_card.rs:27` historically described the stub as "top-4 of SHA-256(bytecode)" — the authoritative formula is the form-aware id (`SPEC_mk_v0_1.md`). Not load-bearing for the fork cycle (the fork mirrors the CODE, not the comment), but noted so the brainstorm cites `mod.rs:72-82`, not the doc comment.

6. **Verify-leg is the risk center, not the encoder.** The fork's mk1↔md1 stub binding (`bundle/verify.go:108`) is security-load-bearing; changing it to form-aware is the piece that most needs its own R0 + golden cross-check against the constellation (an off-by-one in form discrimination → an mk1 that binds the wrong policy, or a template that won't verify). TDD must include: template md1 → TemplateId stub matches; full md1 → WalletPolicyId stub matches (unchanged); the two never collide.

---

## Recommended brainstorm-session scope

- **One cycle, fork-side, firmware-only, SINGLE-SIG template only.** Name suggestion: `seedhammer-singlesig-template-engrave` (the on-device half of `constellation-template-only-engraving`).
- **Sizing: M.** Three slices:
  - **Slice 1 (headless, golden-locked):** port `md.WalletDescriptorTemplateId` (≡ `identity.rs:71-104`) + the form-aware stub (`is_wallet_policy() ? WalletPolicyId : WalletDescriptorTemplateId`) into the fork's Go `md`/`bundle`. Byte-exact vs the constellation; this is the riskiest leg (security stub) — its own R0 emphasis.
  - **Slice 2 (ingest + verify):** relax the single-sig full-policy refusal to ACCEPT a supplied template md1; route `bundle/verify.go` stub binding through the form-aware id; ensure restore-doc/verify messaging is correct for the template form (template completes with the seed).
  - **Slice 3 (GUI choice + engrave):** "full md1 vs template md1" ChoiceScreen on the single-sig flow (no new `program`), engrave verbatim, plate-count win surfaced; loud-but-accurate labelling (template needs the seed to complete).
  - **(Optional / decide at brainstorm)** on-device template ENCODER (pubkeys-null emit) — only if the device must AUTHOR templates rather than ingest supplied ones. Recommend deferring (supply-path mirrors the constellation and avoids a net-new encoder).
- **SemVer / lockstep:** N/A for the `me` CLI (this is the fork firmware, not the mnemonic-engrave CLI — no clap `schema_mirror` / docs-manual mirror). Fork-side: **no new `program`** (so the `gui/gui.go:164` program-count guard is NOT tripped) — extend the existing single-sig flow's ChoiceScreen. **Multisig path UNCHANGED** (stays full-policy; constellation refuses multisig template).
- **Dependencies / ordering:** Slice 1 → Slice 2 → Slice 3 (verify depends on the ported identity; GUI depends on ingest+verify). No constellation work required (already shipped); pin the three constellation SHAs as the golden source of truth.
- **Companion housekeeping:** flip both stale FOLLOWUP entries (this repo's `constellation-template-only-engraving` and `mnemonic-toolkit`'s `bundle-md1-template-only-option`) to reflect the 2026-06-19 constellation adoption.
- **No upstream PRs** (fork-side firmware cycle).
- **Gate reminder:** per project standard, the brainstorm SPEC and the IMPLEMENTATION_PLAN each MUST pass an opus architect R0 to 0C/0I before any code (folds persisted verbatim to `design/agent-reports/`), then single-implementer TDD in a worktree, then the mandatory whole-diff adversarial exec review. The Slice-1 stub/identity port warrants the heaviest R0 scrutiny (security-load-bearing, golden-locked vs Rust).
