# Recon stream — codec layer (md-codec + mk-cli) for fork template-engrave — VERBATIM agent report

**Agent:** `a85b76efcece2be35` (read-only; built + ran `md`/`mk` debug binaries for CLI probes). Persisted by the controller (agent was no-write). **Date:** 2026-06-20. Part of the refreshed `seedhammer-template-engrave` recon (single-sig + multisig). Streams: codec (this) + toolkit (`seedhammer-template-engrave-recon-toolkit.md`) + fork (`seedhammer-template-engrave-recon-fork.md`).

---

## Recon complete — codec-layer template support for the SeedHammer-fork engrave feature

### Sync block

| Repo | Branch | HEAD | Relevant recent commits |
|---|---|---|---|
| `descriptor-mnemonic` (md-codec/md-cli) | `main` | **`54dd765`** | `54dd765` md-codec 0.37.0 + md-cli 0.7.1 (per-cosigner use-site override); `0bd9088` faithful per-cosigner override + hardened-anywhere guard |
| `mnemonic-key` (mk-codec/mk-cli) | `main` | **`1279ef9`** | `3258271` form-aware policy_id_stub — template-id for keyless md1 (mk-cli v0.10.0) |
| `mnemonic-toolkit` (consumer; CHANGELOG cited) | — | — | `[0.60.0] 2026-06-20` — keyless multisig template; "md-codec/mk-codec NOT bumped" |

All probes built and run against these exact binaries (`target/debug/md`, `target/debug/mk`).

---

### Finding 1 — Template-stable identity (`compute_wallet_descriptor_template_id`) — CONFIRMED

`crates/md-codec/src/identity.rs:71-104`. Preimage (lines 74-98):
```
use_site_path.write || tree::write_node(tree, kiw) || [UseSitePathOverrides TLV entry bits if present]
```
then `SHA-256(bytes)[0..16]`. No key material, no fingerprints, no origin-path, no header/HRP/BCH in the preimage → key-independent + origin-invariant by construction.

- **Multisig well-defined:** `tree::write_node` (`crates/md-codec/src/tree.rs:115-139`) encodes the multi-family body `Body::MultiKeys { k, indices }` as `5 bits (k-1) | 5 bits (N-1) | N×kiw key indices`, used by `Tag::Multi | SortedMulti | MultiA | SortedMultiA` (`tree.rs:244`). The threshold k / N-slot structure is part of the hashed tree — so the WDT-Id is defined identically for single-sig and every multisig tree.
- **Origin-invariant (unit + CLI):** `identity.rs:325-338` (`wdt_id_invariant_to_origin_path_change` asserts `id1 == id2`) and `:354-362` (invariant to fingerprint addition). CLI matrix: keyless `wsh(sortedmulti(2,...))` = `b02b4403...` under default origin, `--path bip84`, and `--path m/48'/0'/0'/2'` — all identical.
- **Distinct per (script, k, N, use-site):** wsh-multi `9229657a` ≠ wsh-sortedmulti `b02b4403`; k=1 `454865ef` ≠ k=2; N=2 `aad0e0e0` ≠ N=3; wpkh `45775d4d` ≠ pkh `559e64b2`; differing use-site paths differ (`identity.rs:340-351`).

### Finding 2 — Keyless template md1 wire shape (single-sig vs multisig) — CONFIRMED

`md encode <template>` with no `--key` ⇒ `tlv.pubkeys: null`; threshold/sortedmulti/N rides in the tree. Decoded via `md decode --json`:
- single-sig `wpkh(@0/<0;1>/*)` → `pubkeys:null`
- `wsh(sortedmulti(2,@0,@1,@2))` → `pubkeys:null`, tree `Wsh→SortedMulti{MultiKeys k:2 indices:[0,1,2]}`
- `wsh(multi(2,...))` → `pubkeys:null`, `Wsh→Multi{k:2}`
- `sh(wsh(sortedmulti(2,...)))` → `pubkeys:null`, `Sh→Wsh→SortedMulti`
- `tr(<NUMS>,multi_a(2,...))` → `pubkeys:null`, `Tr{is_nums:true}→MultiA{k:2}` (round-trips with an explicit `--path`; a bare taproot template needs an explicit origin — `MissingExplicitOrigin`, expected).

`is_wallet_policy()` = `pubkeys is Some & non-empty` (`encode.rs:50-51`) — false for all keyless forms.

**Refused:**
- **`tr(...,sortedmulti_a(...))`** — rust-miniscript v13 cannot parse it back: `miniscript parse failed: unrecognized name 'xpub...'` (the `sortedmulti_a` renderer exists at `format/text.rs:63` but the parse/round-trip leg is the v13 render gap; toolkit CHANGELOG line 15: "`tr(sortedmulti_a)` ... refused").
- **`sortedmulti` inside a combinator** (e.g. `wsh(or_d(sortedmulti(...),...))`) — same `unrecognized name 'xpub...'` parse refusal (no `WshInner`/`ShInner` arm; `template.rs:560-568,533-551` only accept top-level `SortedMulti`).
- **Hardened use-site** — NOTE: NOT refused at template encode (a `/*'` wildcard and `/2'/` fixed use-site component both encode and round-trip; `wildcard_hardened:true` decodes back). The hardened refusal is a **derivation-time funds-safety guard**: `Error::HardenedPublicDerivation` ("use-site path requires hardened component, which BIP 32 forbids on xpub-only restore", `error.rs:371-379`), fired by `has_hardened_use_site` in `derive.rs:105` / `to_miniscript.rs:89`. **The fork must mirror it on its derive/address path, not its template parser.**

### Finding 3 — Form-aware mk1 stub (`derive_stub_from_md1`) — CONFIRMED

`crates/mk-cli/src/cmd/mod.rs:72-82`:
```rust
let id_bytes = if descriptor.is_wallet_policy() {
    *md_codec::compute_wallet_policy_id(&descriptor)?.as_bytes()
} else {
    *md_codec::compute_wallet_descriptor_template_id(&descriptor)?.as_bytes()
};
// top 4 bytes → stub
```
Tests pass (`cargo test -p mk-cli --test template_id_stub`: 3/3): keyless template → WDT-Id stub (`559e64b2`, not the buggy policy-id `3d190af3`); keyed → WalletPolicyId stub. Correct for n≥2 multisig: the stub roots on the WDT-Id (per-tree, key-independent), so each of the N keyless cosigner mk1 cards minted from the same template md1 carries the same template-stable stub binding them to one wallet type — CLI-confirmed (`mk encode --from-md1` on the keyless wsh-sortedmulti md1 produces cards rooting on `b02b4403...`).

**No phase-2 codec change — CONFIRMED** by primary source: toolkit `CHANGELOG.md:21` (v0.60.0): "`md-codec`/`mk-codec` are NOT bumped — the keyless multisig wire form, `WalletDescriptorTemplateId`, and the form-aware mk-cli stub all pre-existed."

### Finding 4 — What the fork's Go `md`/`mk` must add — CONFIRMED

- **(a) Bind cards by the template-stable id:** mk-codec already stores the binding (`mk-codec/src/key_card.rs:25-34`: `policy_id_stubs: Vec<[u8;4]>`, form-aware). The fork's Go has only the keyed `WalletPolicyId`; it must add the Go port of **`WalletDescriptorTemplateId`** (the preimage above) plus the **form-aware stub selector** (`is_wallet_policy() ? WalletPolicyId : WalletDescriptorTemplateId`, top-4).
- **(b) Decode/round-trip keyless template md1:** Rust md-codec already round-trips both keyless single-sig and keyless multisig at the codec layer. The fork's Go decoder must handle the `MultiKeys` body (`5|5|N×kiw`) and `Tr{is_nums}` body — the multisig keyless case round-trips at the codec layer today in Rust.

---

### Codec-layer verdict for the fork (pin: md-codec/md-cli `54dd765`, mk-cli `1279ef9`, toolkit `[0.60.0]`)

1. **Port to Go:** `WalletDescriptorTemplateId` — `SHA-256(use_site_path ‖ tree ‖ UseSitePathOverrides-TLV)[0..16]` (key-independent, origin-invariant, distinct per script/k/N/use-site; well-defined for single-sig AND multisig via `MultiKeys{k,N,indices}`).
2. **Port to Go:** the form-aware stub — `is_wallet_policy() ? WalletPolicyId : WalletDescriptorTemplateId`, top-4 — for card↔template binding (one keyless mk1 stub per cosigner, all sharing the template-stable id).
3. **Already present:** keyless md1 decode (single-sig + multisig round-trips at the Rust codec layer); mk-codec's `policy_id_stubs` binding slot.
4. **Refuse (mirror):** `tr(sortedmulti_a)` and `sortedmulti`-in-combinator at the template parser; hardened use-site at the **derive/address** path (`HardenedPublicDerivation`), NOT at template encode.
5. **No further md-codec/mk-codec change needed** — primary-source confirmed (toolkit `CHANGELOG.md:21`, v0.60.0).
