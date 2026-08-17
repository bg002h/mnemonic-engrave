# S6b recon: what identifies a KEY in `mk1`?

**READ-ONLY research pass.** No design proposed, no recommendation made.

**Repo read:** `/scratch/code/shibboleth/mnemonic-key`
**Commit:** `8dc5dcbf31947762a354d165ca2350ddbb15ba28`
**Tree state:** clean (`git status --short` produced no output — no uncommitted changes)

Cross-check repo (downstream, per Rust-primary rule): `/scratch/code/shibboleth/seedhammer/mk/` (`mk.go`, `encode.go`). No disagreement found — see Q5.

---

## Q1. Every identifier `mk1` carries

Read from the struct definitions and the encoder/decoder (not doc comments alone):
`crates/mk-codec/src/key_card.rs` (`KeyCard`), `crates/mk-codec/src/bytecode/encode.rs` (`encode_bytecode`), `crates/mk-codec/src/bytecode/xpub_compact.rs` (`XpubCompact`), `crates/mk-codec/src/string_layer/header.rs` (`StringLayerHeader`).

| Field (code name) | Type / width | What it identifies | Derived from |
|---|---|---|---|
| `policy_id_stubs` | `Vec<[u8; 4]>`, 4 bytes each | Which `md1`-encoded **policy** card(s) (by `WalletPolicyId`) this xpub is intended to serve | A **policy**, not a key — `crates/mk-codec/src/key_card.rs:26-33`: "top 4 bytes of a canonical, encoder-divergence-free md1 identity" (WalletPolicyId or WalletDescriptorTemplateId) |
| `origin_fingerprint` | `Option<Fingerprint>` (`bitcoin::bip32::Fingerprint` = 4 bytes), present iff bytecode-header bit 2 set | "the seed from which `xpub` was derived" (master-key/BIP-32 fingerprint) | A **key** (the seed) — `crates/mk-codec/src/key_card.rs:36-40` |
| `origin_path` | `bitcoin::bip32::DerivationPath` (wire: 1-byte std-table indicator, or `0xFE` + count + ≤10 LEB128 u32 components) | Derivation path from master to `xpub` | Derived, encodes into `xpub_compact` derivation context — `crates/mk-codec/src/key_card.rs:42-46` |
| `xpub` | `bitcoin::bip32::Xpub`, wire form `XpubCompact` = 73 bytes (`version`:4B, `parent_fingerprint`:4B, `chain_code`:32B, `public_key`:33B — `crates/mk-codec/src/bytecode/xpub_compact.rs:32-41`) | The BIP-32 extended public key itself | The key's public material; `depth`/`child_number` are dropped from the wire and reconstructed from `origin_path` at decode (`crates/mk-codec/src/bytecode/xpub_compact.rs:1-15`, SPEC §3.6) |
| bytecode-header `fingerprint_flag` | 1 bit (bit 2 of the 1-byte header) | Whether `origin_fingerprint` is present | `crates/mk-codec/src/bytecode/header.rs:22-34` |
| `chunk_set_id` (string layer only, chunked cards) | `u32`, but wire-packed into **20 bits** across 4 five-bit symbols | Not a key/card identity — "opaque to the format; its only purpose is mismatch detection during reassembly" (SPEC §2.5) | `derive_chunk_set_id` = top 20 bits of `SHA-256(canonical_bytecode)`; packing proven at `crates/mk-codec/src/string_layer/header.rs:79-98` (`to_5bit_symbols`) and unpacked at lines 139-142 (`from_5bit_symbols`) |

Encoded field order on the wire (`crates/mk-codec/src/bytecode/encode.rs:1-12`, matches SPEC §3.2 exactly — verified against `encode_bytecode` body lines 50-67):
```
[bytecode_header:1B] [stub_count:1B] [policy_id_stubs: 4×N B]
[origin_fingerprint:4B, iff flag set] [origin_path: variable] [xpub_compact:73B]
```

There is **no separate "key id" or "card id" field on the wire**. The two identity-bearing fields are `policy_id_stubs` (identifies a *policy*, per-stub) and `origin_fingerprint` (identifies a *seed*, optional). `xpub` itself is the key's public identity but is not treated as a compact "identifier" anywhere in the format — it's carried in full (minus the two reconstructible fields).

## Q2. Is there anything called "key id" / "key identifier" / "kid" / "key fingerprint" or similar?

**I searched; I found none that names a distinct MK identifier.** Method: `grep -rniE "key[_ -]?id|key[_ -]?identifier|\bkid\b|key[_ -]?fingerprint"` across `*.rs`/`*.md`/`*.mediawiki` in the whole `mnemonic-key` tree (excluding `target/` and the stale worktree checkout).

Hits, all triaged:
- `crates/mk-codec/src/key_card.rs:36` and `crates/mk-codec/src/bin/gen_mk_vectors.rs:81` — doc comments literally say **"Master-key fingerprint"**, which is `origin_fingerprint` (the BIP-32 master/seed fingerprint), not a distinct "key id" concept.
- `crates/mk-codec/src/bytecode/xpub_compact.rs:35` — "4-byte parent-key fingerprint" — this is `xpub.parent_fingerprint`, a standard BIP-32 field carried inside `XpubCompact`, unrelated to card/key identity.
- `bip/bip-mnemonic-key.mediawiki:35` — defines "Origin fingerprint" (same as `origin_fingerprint` above), quoted verbatim in Q3 below.
- `vendor/bitcoin/src/bip32.rs:67-68,702,833-836` — `rust-bitcoin`'s own `XKeyIdentifier` type ("Extended key identifier as defined in BIP-32" = `HASH160(pubkey)`). This is a **vendored upstream dependency type**, not something `mk-codec` imports, constructs, or exposes anywhere — confirmed by the full grep of `crates/mk-codec/src/*.rs` (no `XKeyIdentifier` hit inside the crate).
- `vendor/bip39/src/language/english.rs:980` — the word "kid" in the BIP-39 English wordlist. Unrelated (wordlist entry, not a format field).
- `vendor/aho-corasick/...` — "key idea", false-positive substring match, unrelated.

So: **no field, type, or documented concept named "key id"/"kid"/"key identifier" exists anywhere in `mk-codec` or its spec/BIP-draft prose.** The only fingerprint-shaped identifier `mk1` defines is `origin_fingerprint`, called exactly that (never "key id" or "kid") in both code and prose.

## Q3. Does `mk1` carry a BIP-32 master fingerprint?

**Yes.** Field `origin_fingerprint: Option<Fingerprint>` (`bitcoin::bip32::Fingerprint`, 4 bytes — `crates/mk-codec/src/key_card.rs:40`; wire constant `ORIGIN_FINGERPRINT_BYTES = 4` at `crates/mk-codec/src/consts.rs:62-63`).

- **It is the seed's fingerprint, not a passphrase-combined one**, and this is stated explicitly, twice:
  - Code: `crates/mk-codec/src/key_card.rs:36-38` — *"Master-key fingerprint identifying **the seed** from which `xpub` was derived. Verbatim from BIP 380 origin notation `[fp/...]`."*
  - Spec: `design/SPEC_mk_v0_1.md:216` (§3.4) — *"The 4-byte BIP 32 master fingerprint, identifying **the seed** from which this xpub was derived. Verbatim from the BIP 380 origin-notation `[fp/...]` prefix."*
  - BIP draft: `bip/bip-mnemonic-key.mediawiki:35` — *"'''Origin fingerprint''': the 4-byte master-key fingerprint identifying the seed from which the xpub was derived. Verbatim from BIP 380 origin notation `[fp/...]`. Optional in MK v0.1; presence is signaled by the bytecode header's fingerprint flag."*
- **BIP 380 `[fp/...]` origin notation is defined to use the fingerprint of the master key that the seed (with any passphrase already applied, per BIP 32) produces** — i.e. the fingerprint context `mk1` inherits is whatever seed the deriving wallet treated as its master, but the code/spec never distinguishes a "with-passphrase" vs "without-passphrase" variant; there is exactly one `origin_fingerprint` field, sourced verbatim from BIP 380 notation. `mk1` itself carries no separate passphrase-state indicator alongside this field, and nothing in the crate or spec computes or stores a second, passphrase-combined fingerprint distinct from `origin_fingerprint`.
- **It is optional** — present iff bytecode-header bit 2 (`fingerprint_flag`) is set (closure Q-8; `design/SPEC_mk_v0_1.md:170,218`, `crates/mk-codec/src/bytecode/header.rs:8,22-34`). Rationale for optionality (privacy-preserving mode) at SPEC §6, `design/SPEC_mk_v0_1.md:376`.

## Q4. Documented relationship between an `mk1` card and the key it represents

Per SPEC §5 "Linkage to MD" (`design/SPEC_mk_v0_1.md:324-356`) and §3.3 (`:202-212`):

- The **card-to-policy** link is `policy_id_stubs` → top-4-bytes-of-`WalletPolicyId` match against a decoded `md1` policy card. This is explicitly a **human-indexing aid, not a cryptographic primitive** (`:208`): *"The stub is a human-indexing aid, not a cryptographic primitive. The cryptographic check happens at recovery time when the xpub is plugged into the policy and the Wallet Instance ID is recomputed."*
- The **card-to-key** cryptographic binding is the `xpub` field itself plus `origin_path`/`origin_fingerprint`. There is no compact "key id" a reader matches against a registry — a reader who wants to know "is this the right key" plugs the decoded `xpub` into the assembled wallet and computes the **Wallet Instance ID**:
  ```
  wallet_instance_id = SHA-256(canonical_bytecode || canonical_xpub_serialization)[0..16]
  ```
  (`design/SPEC_mk_v0_1.md:335-341`, §5 step 4), compared against a separately-anchored expected identity. This is a **wallet-instance**-level check (spans the whole assembled multisig, all cosigner xpubs), not a per-key lookup key.
- So: **given an `mk1` in hand, the field that lets a reader match it to a particular key/xpub is the `xpub` field directly** (its raw 73-byte compact serialization, reconstructed to full BIP-32 form via `origin_path`). There is no separate compact per-key identifier field — matching is either (a) via `policy_id_stubs` at the policy level (coarse, human-indexing), or (b) via full `xpub` + `origin_fingerprint` + `origin_path` compared directly against known key material / the Wallet Instance ID (cryptographic, whole-instance).

## Q5. Spec document enumerating header fields normatively

Yes: **`design/SPEC_mk_v0_1.md`**, header states *"Status: v0.1 wire format locked. All Q-1..Q-10 closures landed 2026-04-29"* (`:3`). §3.2 "Payload field order" (`:181-194`) is the normative field-order table, quoted verbatim above in Q1. §3.1 (`:158-179`) is the normative 1-byte bytecode-header bit layout. §2.5 (`:89-144`) is the normative string-layer header (single-string / chunked, including the `chunk_set_id` packing rule).

Verbatim field list from §3.2 (`design/SPEC_mk_v0_1.md:186-192`):
```
[bytecode_header   : 1 B]
[stub_count        : 1 B; MUST be ≥ 1]
[policy_id_stubs   : 4 × N B]
[origin_fingerprint: 4 B]   ← present iff bytecode_header bit 2 set
[origin_path       : 1 B (std-table indicator) OR 2..=52 B (explicit: 0xFE + count + 0..=10 LEB128 components, ≤5 B each; count 0 = no-path / depth-0 root key; see §3.5)]
[xpub_compact      : 73 B]
```
No field in this list, nor anywhere else in the spec (`§1` scope through `§11`), is called a "key id"/"key identifier"/"kid". The spec's own identifier vocabulary is: `Policy ID stub` (policy-level), `origin_fingerprint` (seed-level), `Wallet Instance ID` (whole-assembled-wallet, computed not carried), `chunk_set_id` (transport/reassembly, opaque).

**Go-port cross-check (downstream, informational):** `/scratch/code/shibboleth/seedhammer/mk/mk.go` `Card` struct carries `Fingerprint string` (hex) and `Stubs [][4]byte` — same two identifiers, same names in substance, no third "key id" field. `encode.go:70-93` builds the header/fingerprint bytes in the same order as the Rust encoder. **No disagreement found** between the Go port and the Rust source on this question.

---

## Summary table

| Identifier | Width | Identifies | Source (Rust file:line) |
|---|---|---|---|
| `policy_id_stubs[i]` | `[u8; 4]` (4 bytes) per stub, `stub_count: u8` count prefix | An `md1` **policy** (top 4 bytes of `WalletPolicyId`/`WalletDescriptorTemplateId`) — NOT a key | `crates/mk-codec/src/key_card.rs:26-34`; wire: `crates/mk-codec/src/bytecode/encode.rs:56-60` |
| `origin_fingerprint` | `Option<Fingerprint>` = 4 bytes, optional (bit-2 flag) | The **seed**'s BIP-32 master fingerprint (verbatim BIP 380 `[fp/...]`) | `crates/mk-codec/src/key_card.rs:36-40`; wire: `crates/mk-codec/src/bytecode/encode.rs:61-63`; spec: `design/SPEC_mk_v0_1.md:214-218` |
| `origin_path` | 1 byte (std-table) or 2–52 bytes (explicit LEB128) | Derivation path from master to `xpub` | `crates/mk-codec/src/key_card.rs:42-46`; spec §3.5 |
| `xpub` (`XpubCompact`) | 73 bytes (`version`4 + `parent_fingerprint`4 + `chain_code`32 + `public_key`33) | The BIP-32 extended **public key** itself — the key's own cryptographic identity | `crates/mk-codec/src/bytecode/xpub_compact.rs:32-41` |
| `chunk_set_id` | 20 bits (packed into 4× 5-bit symbols), chunked cards only | Transport/reassembly grouping — opaque, not a card or key identity | `crates/mk-codec/src/string_layer/header.rs:26-27,79-98,139-142`; spec §2.5 |
| "key id" / "kid" / "key identifier" | — | **Does not exist** in this codebase or its spec/BIP prose | Grep-verified absence (Q2) |

**Bottom line for the unsettled question:** `mk1` does **not** define a distinct "key id" concept anywhere in code, spec, or BIP draft. The only per-key identifier it carries is `origin_fingerprint` — explicitly the **BIP-32 master (seed) fingerprint**, verbatim BIP-380 `[fp/...]`, optional via the bytecode-header flag — plus the `xpub` itself as the key's full public material. `policy_id_stubs` identifies a *policy*, not a key.
