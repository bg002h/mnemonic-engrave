# Recon — toolkit v0.60.0 template EMIT + RECOMPOSE model (for the SeedHammer-fork template-engrave feature)

**Purpose:** establish, from PRIMARY SOURCE, the toolkit's `--md1-form=template` emit shape (single-sig AND multisig) and the off-device recompose/permutation-search completion, so a SeedHammer-fork template-engrave feature (single-sig + multisig) mirrors the right half (emit/bind/verify) and correctly DEFERS the search.

## Sync block (PINNED)

- **Repo:** `/scratch/code/shibboleth/mnemonic-toolkit`
- **Branch:** `master`
- **HEAD:** `6de53879` (`6de5387945bf4f5bdc9b73f04fb06c0de1bf9f34`) — `git describe`: `mnemonic-toolkit-v0.60.0-1-g6de53879` → this IS toolkit **v0.60.0** (HEAD is one non-source commit past the v0.60.0 tag; the #28-phase-2 feature is SHIPPED, not draft).
- **md-codec / mk-codec:** no change needed for #28 (confirmed in FOLLOWUPS); the template-stable id (`compute_wallet_descriptor_template_id`) and the keyless template wire form already existed.
- **Prior recon (superseded):** `mnemonic-toolkit/cycle-prep-recon-bundle-md1-template-only-option.md` was written at SHA `f7e6fca` BEFORE implementation ("no template-md1 emit path today"). That is stale — the feature is now built; this recon reads the SHIPPED code.

**NOTE on the earlier RECON prompt nuance ("single-sig"):** the prompt's finding-1 sketch and the `cycle-prep-recon-bundle-md1-template-only-option.md` doc both describe phase-1 (single-sig-only). The SHIPPED v0.60.0 is **phase 1 + phase 2** — multisig/general policy template emit + the parallel permutation-search completion are present. This recon reports the FULL shipped surface.

---

## FINDING 1 — Template EMIT shape (single-sig vs multisig)

**Flag → dispatch.** `bundle --md1-form <policy|template>` selects `Md1Form` (`synthesize.rs:53-67`). `Policy` (default) = full keyed wallet-policy md1 (pubkeys/fingerprints present, explicit origin, binds on `WalletPolicyId`). `Template` routes `synthesize_descriptor` → `synthesize_template_descriptor` (`synthesize.rs:467-487`):

```
// synthesize_descriptor (synthesize.rs:484-487)
//   the binding stub re-roots on the key-stable `WalletDescriptorTemplateId`
if md1_form.is_template() {
    return synthesize_template_descriptor(descriptor, cosigners, privacy_preserving);
}
```

### The admission gate `template_admissible` (`synthesize.rs:1087-1122`) — ARITY-SPLIT

```rust
fn template_admissible(descriptor: &Descriptor) -> bool {
    if descriptor.n == 1 {
        // Phase-1 single-sig gate, verbatim: only the canonical-elidable types.
        return cli_template_from_tree(&descriptor.tree).is_some();
    }
    if md_codec::to_miniscript::has_hardened_use_site(descriptor) {
        return false;
    }
    md_codec::to_miniscript::to_miniscript_descriptor(descriptor, 0).is_ok()
}
```

- **n == 1 (single-sig):** ADMIT ONLY the three canonical-origin-elidable types — `cli_template_from_tree(tree).is_some()` (`synthesize.rs:359-368`): `pkh(@0)`→bip44, `wpkh(@0)`→bip84, `tr(@0)` key-path (no TapTree)→bip86. **REFUSED:** bip49 `sh(wpkh)`, taproot-with-tree, bare `wsh`, and a degenerate nested-multi/sortedmulti 1-of-1 (R0 I1) — these route to `--md1-form=policy`.
- **n ≥ 2 (multisig / general policy):** ADMIT "exactly what `restore` can later reconstruct from xpubs":
  - (a) the shape RENDERS via `to_miniscript_descriptor(descriptor, 0)` (single-path, chain 0) — admits non-taproot `multi`/`sortedmulti`/`thresh`/timelocks/hashlocks/`or_i` branches AND `tr(NUMS, multi_a)`;
  - (b) NO hardened use-site (`has_hardened_use_site` — #25; an xpub cannot derive a hardened public child → unrestorable).
- **REFUSED (n≥2):** `tr(sortedmulti_a)` (the rust-miniscript v13 `SortedMultiA` render gap — confirmed in the doc-comment `synthesize.rs:1096-1098` and pinned by test `template_admissible_gate` `synthesize.rs:2464-2475`), `sortedmulti` inside a combinator (render gap), any hardened use-site. Refusal → `ToolkitError::TemplateFormUnsupportedShape` with the message at `synthesize.rs:1170-1177` ("…use --md1-form=policy for a faithful keyed backup").

### The emitted bundle (the mutations, `synthesize_template_descriptor` `synthesize.rs:1158-1283`)

Gate runs on the KEYED input first (it must render). Then mutate a `descriptor.clone()`:
1. `template.tlv.pubkeys = None` (`:1182`)
2. `template.tlv.fingerprints = None` (`:1183`)
3. **C1-conditional origin** (`:1195-1198`): if `canonical_origin(&descriptor.tree).is_some()` (canonical single-sig + canonical `wsh(multi/sortedmulti)`, `sh(wsh(...))`) → ELIDE to `Shared(empty)` (byte-identical-shareable, account-agnostic). Else (general policy — `wsh(or_i)`, `thresh`, timelocks, e.g. degrade2) → KEEP the cloned source per-`@N` origins, or `md decode` rejects with `MissingExplicitOrigin` (the C1 regression). Carried origin is decode/display-ONLY; re-supplied at completion; the template-id is origin-invariant.
4. the `is_wallet_policy()` keyed-path assert is NOT asserted (template is keyless by construction).

The per-`@N` use-site structure (incl. #25 overrides), threshold k, sorted shape, and N slots ride along UNMUTATED in `descriptor.tree`.

**The emitted bundle for n≥2 (`synthesize.rs:1212-1280`):**
- `md1` = the keyless template (`md_codec::chunk::split(&template)`, `:1212`).
- `mk1` = **one keyless cosigner mk1 STUB card per cosigner** (`MkField::Multi`, `:1235-1257`): for each cosigner `i`, `mk_codec::KeyCard::new(stubs.clone(), fp?, mk1_origin_path(xpub, path), xpub)` encoded with a slot-unique csi `derive_mk1_chunk_set_id_for_slot(&stub, i)`. At n==1 it is `MkField::Single` (`:1220-1234`). `privacy_preserving` drops the fingerprint.
- `ms1` = one per slot, UNCHANGED by form (plain codex32 entropy/mnem, no id field); watch-only slots → `""` sentinel (`:1261-1280`).

---

## FINDING 2 — The binding stub (md1 + mk1 + display stub root on `WalletDescriptorTemplateId`)

The keyless template CANNOT reproduce `WalletPolicyId` (which folds in per-`@N` key presence + raw fp/xpub), so the binding re-roots on the **key-stable** `WalletDescriptorTemplateId`. Stub derivation (`synthesize.rs:1203-1209`):

```rust
// --- Binding stub: re-root on WalletDescriptorTemplateId (SPEC §4.3) ----
// Compute the id ONCE from the MUTATED template (the engraved md1 is the
// template; binding must reflect what is engraved).
let template_id =
    md_codec::compute_wallet_descriptor_template_id(&template).map_err(ToolkitError::from)?;
let mut stub = [0u8; 4];
stub.copy_from_slice(&template_id.as_bytes()[..4]);
```

- **md1** binds on the leading bytes of `template_id` (the same `stub`).
- **mk1** per cosigner `i`: `csi = derive_mk1_chunk_set_id_for_slot(&stub, i)` (`:1232`, `:1251`). `derive_mk1_chunk_set_id_for_slot(stub, slot) = derive_mk1_chunk_set_id(stub) ^ slot` (`synthesize.rs:90-92`); `derive_mk1_chunk_set_id` = top 20 bits MSB-first of the stub (`:73-75`). XOR is injective in `slot` ⇒ pairwise-distinct csi per cosigner (audit I10: same-xpub cosigners must not collide); the slot index (≤15) touches only the low nibble, so the **leading 16 bits = `template_id[0..2]`** (the bundle-binding prefix shared with md1) are preserved across all cosigners. For n==1 (slot 0) this is byte-identical to `derive_mk1_chunk_set_id`.
- Contrast: the keyed `--md1-form=policy` path (`wallet_policy_id_for_singlesig` `synthesize.rs:206`, `wallet_policy_id_for_template` `:229`, `build_descriptor` `:167-195`) binds on `WalletPolicyId` (key/origin-significant) — the full-policy md1 and the template md1 of the SAME wallet hash to DIFFERENT ids. **One keyless mk1 stub per cosigner.**

---

## FINDING 3 — The RECOMPOSE / completion model (the hard half the fork must understand)

This is an OFF-DEVICE toolkit operation. `restore --md1 <keyless template> --from <seed> --cosigner <mk1>…` completes the template into a concrete watch-only wallet by resolving the unique `@N`→key assignment.

### Routing (`restore.rs:282-318`)

```rust
if !args.md1.is_empty() {
    if let Ok(d) = md_codec::chunk::reassemble(&md1_refs) {
        let is_singlesig_template = !d.is_wallet_policy()
            && d.n == 1
            && md_codec::canonical_origin::canonical_origin(&d.tree).is_some()
            && crate::synthesize::cli_template_from_tree(&d.tree).is_some();
        if is_singlesig_template {
            return run_singlesig_template_completion(&d, args, stdin, stdout, stderr);
        }
        let is_multisig_template = !d.is_wallet_policy() && d.n >= 2;   // <-- the n>=2 route
        if is_multisig_template {
            return run_multisig_template_completion(&d, args, stdin, stdout, stderr);
        }
    }
    return run_multisig(args, stdin, stdout, stderr);  // keyed full-policy md1 falls here
}
```

- **single-sig template** (`!is_wallet_policy() && n==1 && canonical_origin().is_some() && cli_template_from_tree().is_some()`): completed from `--from` seed alone (`run_singlesig_template_completion` `restore.rs:770`); `--from` REQUIRED (the C2 funds-safety hole, `:784-790`); `--account`/`--origin` supply the origin; NO search.
- **multisig/general template** (`!is_wallet_policy() && n>=2`): `run_multisig_template_completion` `restore.rs:1321` → the shared `complete_multisig_template` engine.

### The shared engine `complete_multisig_template` (`restore.rs:1416-1814`) — the funds-safety core

Pipeline (doc-comment `:1403-1415`): cosigner parse → per-slot origin BUILD (NEVER the carried `path_decl` — the C1 invariant) → floors → mode → permutation search → unique assignment + a fresh fully-keyed descriptor.

- **I-1 gate** (`:1434-1440`): `--own-account-max` (own-account RANGE / subset-search) is REFUSED — the engine only enumerates `n!` placements of exactly `n` pool entries; over-supply would leave indices ≥ n never evaluated → silent NO-MATCH. The subset-search is DEFERRED (FOLLOWUP `template-multisig-own-account-range-subset-search`).
- **Cosigner parse** (`:1442-1516`): assigned `@N=<mk1|xpub>` (explicit) vs unassigned (search), greedy multi-chunk grouping; mixing the two forms is refused.
- **Own keys** built from `--from` at each `--account` (`:1571-1605`); own origin = `--origin` override → cosigner-family-with-account-substituted → canonical (BIP-48) fallback.
- **EXPLICIT mode** (`:1607-1610`): all `--cosigner @N=` → `complete_explicit_assignment` (no search).
- **Floors** (SPEC §7): Floor 1(ii) `pool.len() == n` exactly (`:1616-1644`); Floor 2 `reject_duplicate_keys` BEFORE search (`:1646-1648`).
- **Realized S** = `perm_count_u128(n, n)` = `n!` (`:1650-1662`).
- **Mode select** (`:1664-1783`):
  - **id-search** (`--expect-wallet-id` set, `:1696-1720`): `ps::validate_prefix_strength(prefix.len(), realized_s)` enforces the strong-prefix floor; evaluator builds a fresh keyed descriptor per assignment (`build_keyed_template_descriptor`, NEVER the carried path_decl) and matches `compute_wallet_policy_id(cand)`'s prefix; `run_capped_search(..., SearchMode::Id, ...)`.
  - **address-search** (`--search-address`, `:1721-1773`): full scriptPubKey at `(chain, idx)` over the ascending-index-OUTER range; SORTED carve-out (`is_order_independent_shape`, `:1676`) collapses `n!`→1 (sortedmulti/sortedmulti_a are order-invariant) by evaluating only the identity placement; `run_capped_search(..., SearchMode::Address(range), ...)`.
  - **no mode** → REFUSE (`:1774-1783`): "supply a recorded --expect-wallet-id, a --search-address, or explicit --cosigner @N=…".
- **Outcome** (`:1785-1814`): `SearchOutcome::Unique` → build the completed wallet; `None` → `✗ NO MATCH` + `RestoreMismatch`; `Ambiguous` → `✗ AMBIGUOUS` + refuse. Never silent-wrong.

### The search engine (`permutation_search.rs`) — the entry the 6.9/7.4 µs benchmark measures

- `pub fn search<E: CandidateEvaluator>(n, &evaluator, mode) -> Result<SearchOutcome, SearchError>` (`:551-649`). Standalone — the id/address computation is injected as a `CandidateEvaluator` predicate (`:154-175`); the real evaluators are wired in `restore.rs`/`verify_bundle.rs`.
- **PARALLEL:** `std::thread` sharded across `min(MAX_SEARCH_THREADS=20, available_parallelism())` (`search_threads` `:471-476`). The contiguous index space `[0, total)` is sharded; each thread unranks its slice (`unrank_permutation`, Lehmer/factorial-number-system, `:494-505`) and evaluates. Determinism: outcome is a pure function of match COUNT (parallel == `search_reference` single-thread `:655-707`).
- **Funds-safety uniqueness** (`:526-538`): does NOT early-terminate on the first match for `Unique` — scans the full space (or short-circuits at the 2nd match → `Ambiguous`) via a shared atomic + stop flag. `0`→`None`, `1`→`Unique`, `≥2`→`Ambiguous`.
- **Adaptive cap** (SPEC §6.4, `cap_decision` `:388-415`): `<30s`→silent; `30s..1h`→progress; `>1h`→REFUSE unless `--accept-search-time ≥ estimate` (forced acknowledgment).
- **Strong-prefix sizing** (`required_prefix_bytes(S) = ceil((log2(S)+32)/8)` `:322-337`): the `--expect-wallet-id` prefix MUST size from the realized S (a fixed 8-byte prefix hits ~1-in-275 false-positive at K=32). Ladder pinned in tests (`:731-755`).
- **Benchmark (recorded verbatim, NOT a checked-in `cargo bench` file):** per `mnemonic-engrave/design/FOLLOWUPS.md:27` — **~6.9 µs/permutation** vs a known **policyID** (id-search), **~7.4 µs/permutation** vs the **first address** (address-search), on a **24-core Intel i7-13700 @ 5.3 GHz**, Rust mnemonic toolkit. Single-thread N! illustrative: N=5≈0.8 ms, N=9≈2.5 s, N=11≈4.6 min, N=13≈12 h, N=15≈104 d (÷~24 with full parallelism). The `permutation_search.rs` doc-comment names this the "`idsearch`/`addrsearch` cost-model prior art" (`:35-36`, `:47-48`) that sets the 20-thread cap.

### Funds-safety boundary (the wrong-wallet guard)

`--expect-wallet-id` (id-search) is the wrong-wallet guard: a wrong key→slot assignment produces a different `WalletPolicyId` → NO-MATCH → refuse. `--search-address` (full scriptPubKey) is the collision-free target. The strong-prefix floor + `validate_prefix_strength` prevent a too-weak prefix from admitting a spurious match. `AMBIGUOUS`/`NONE` both refuse — never silent-wrong.

### Same intake on `verify-bundle` — CONFIRMED

`verify_bundle.rs` routes IDENTICALLY (`:364-381`: `is_singlesig_template` then `is_multisig_template = !d.is_wallet_policy() && d.n >= 2` → `verify_multisig_template`). `verify_multisig_template` (`:808-983`) calls the SAME `complete_multisig_template` (`:874`) with the same `MultisigCompletionCtx` (`:858-873`) and `resolve_template_completion_seed` (`:833`, the shared seed resolver). After completion it BINDS by:
- recomputing `compute_wallet_descriptor_template_id` on BOTH the completed wallet and the supplied template `d` and asserting equality (`md1_template_match`, `:878-894`);
- `check_mk1_template_stubs` (`:992-1010`): each supplied `--mk1` stub's csi must equal `derive_mk1_chunk_set_id_for_slot(template_id[0..4], slot)` for the recomposed template id — the SAME stub formula the emit side uses (Finding 2). Empty `--mk1` → check skipped.
- then recomposes the watch-only wallet + surfaces the completed `WalletPolicyId` + first receive address.

---

## FINDING 4 — What the FORK must MIRROR vs what it can DEFER off-device

The fork is an air-gapped ENGRAVING device (slow RP2350, no recovery role). The toolkit's design makes the division explicit, and the engrave-repo FOLLOWUPS confirm it:

- `constellation-template-only-engraving` (`mnemonic-engrave/design/FOLLOWUPS.md:25`): "**The SeedHammer on-device leg is now the only remaining work** — the constellation has adopted template engraving … phase 1 (single-sig) shipped at toolkit v0.59.0, phase 2 (multisig/general + `tr(NUMS,multi_a)` + the parallel permutation-search completion in `restore`/`verify-bundle`) shipped at toolkit **v0.60.0**; md-codec/mk-codec needed no change."
- `seedhammer-template-engrave-key-search-time-estimate` (`FOLLOWUPS.md:27`): "**Scope: SeedHammer fork UI only** (display the estimate + the repo link); **the search itself runs off-device in the toolkit**. Surface only on the template-engrave path (full-policy md1 carries the keys → no search needed)." The 6.9/7.4 µs figures + the N! model are recorded there as INPUTS for the on-device estimate display.

**Fork-mirror verdict:**

| Concern | FORK does (on-device, mirror) | TOOLKIT does (off-device, DEFER) |
|---|---|---|
| **EMIT** | Engrave the keyless template **md1** + **N keyless cosigner mk1 stubs** (+ per-slot ms1), single-sig and multisig | — |
| **BIND** | Root md1 + every mk1 stub + display on **`WalletDescriptorTemplateId`** (top-4-byte stub; mk1 csi = `derive_mk1_chunk_set_id_for_slot(stub, slot)` = `top20(stub) ^ slot`), NOT `WalletPolicyId` | — |
| **VERIFY** | Verify the engraved READBACK binds to the template id (md1↔template-id stub; mk1 stub csi per slot) — the engrave-side card↔id integrity | The full template→watch-only completion + `md1_template_match` against a recomposed wallet (`verify-bundle`) |
| **RECOMPOSE / SEARCH** | NONE — surface an estimate of `search-time-vs-N` + a link to the toolkit repo | The parallel `n!` permutation search (id-search via `--expect-wallet-id` / address-search via `--search-address` / explicit `@N=`); the unique `@N`→key resolution; all funds-safety floors (distinct keys, exact-fill, strong-prefix, adaptive cap, unique-vs-ambiguous full scan) |
| **KEY MGMT** | None — engraver holds no recovery role | `--from` seed + `--cosigner` mk1s graft concrete keys onto the keyless template tree (the inverse of `expand_per_at_n`) |

**Refused shapes the fork must NOT attempt to template (mirror the toolkit gate):**
- single-sig (n==1): ONLY pkh/bip44, wpkh/bip84, tr-keypath/bip86 (canonical-origin-elidable). Refuse bip49 `sh(wpkh)`, taproot-with-tree, bare `wsh`, nested-multi/sortedmulti 1-of-1.
- multisig (n≥2): admit only shapes that render via `to_miniscript_descriptor` AND carry no hardened use-site. Refuse `tr(sortedmulti_a)` (rust-miniscript v13 SortedMultiA render gap), `sortedmulti`-in-a-combinator, any hardened use-site (`/*h` or hardened multipath alt — an xpub cannot derive a hardened public child → unrestorable).
- A SeedHammer fork that *authors* a multisig policy on-device (the T6c path, FOLLOWUPS:23) must order-preserve (no key-sort) and produce a tree the toolkit can later complete; `sortedmulti` is key-order-invariant, so the off-device search applies to ordered `multi` / origin-distinct-slot cases (FOLLOWUPS:27 caveat).

**Division is consistent with the toolkit's model:** the toolkit OWNS the authoritative permutation model + parallel/throughput model and the recompose; the fork emits/binds/verifies and only DISPLAYS the recoverability estimate. The fork need not run the search.

---

## Key file:line index (toolkit @ 6de53879)

- `crates/mnemonic-toolkit/src/synthesize.rs` — `Md1Form` `53-67`; csi derivation `73-92`; `build_descriptor` (keyed) `167-195`; `wallet_policy_id_for_singlesig` `206`; `build_keyed_template_descriptor` `283-327`; `is_order_independent_shape` `335-346`; `cli_template_from_tree` `359-368`; `synthesize_descriptor` dispatch `467-487`; `template_admissible` `1087-1122`; `synthesize_template_descriptor` `1158-1283` (mutations `1182-1198`, stub `1203-1209`, mk1 per-cosigner `1212-1257`).
- `crates/mnemonic-toolkit/src/permutation_search.rs` — caps `47-59`; `CandidateEvaluator` `154-175`; `AddressRange/ChainScope/SearchMode/SearchOutcome` `183-273`; `reject_duplicate_keys` `289-298`; `required_prefix_bytes` `322-337`; `validate_prefix_strength` `342-354`; `cap_decision` `388-415`; `calibrate_per_candidate` `446-463`; `search_threads` `471-476`; `unrank_permutation` `494-505`; `search` `551-649`; `search_reference` `655-707`.
- `crates/mnemonic-toolkit/src/cmd/restore.rs` — routing `282-318` (singlesig `285-291`, multisig `is_multisig_template = !d.is_wallet_policy() && d.n >= 2` `312-314`); `run_singlesig_template_completion` `770`; `MultisigCompletionCtx`/`complete_multisig_template` `1416-1814` (I-1 gate `1434-1440`, floors `1616-1648`, mode select `1664-1783`, outcome `1785-1814`); `run_multisig_template_completion` `1321-1389`.
- `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` — routing `364-381`; `verify_multisig_template` `808-983` (shared engine call `874`, template-id bind `878-894`, mk1 stub bind `901-914`); `check_mk1_template_stubs` `992-1010`.
- Tests: `cli_bundle_md1_template_form.rs`, `cli_bundle_md1_template_multisig.rs`, `cli_restore_md1_template.rs`, `cli_restore_md1_template_multisig.rs`, `cli_verify_bundle_md1_template.rs`, `cli_verify_bundle_md1_template_multisig.rs`, `prop_template_completion_roundtrip.rs`.
- Cross-repo FOLLOWUPS (engrave): `mnemonic-engrave/design/FOLLOWUPS.md:25` (`constellation-template-only-engraving` — UNBLOCKED), `:27` (`seedhammer-template-engrave-key-search-time-estimate` — 6.9/7.4 µs verbatim, fork-UI-only).
