# F-449 Stage 1 (1a + 1b) — md-codec Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Give md1 a second taproot internal-key kind — Liana's unspendable xpub, derived from the leaf keys — carried on a new wire version 8, so a `tr` policy with a multi-key primary imports into Liana.

**Architecture:** Two commits in sequence. **1a** replaces `Body::Tr`'s `is_nums: bool` + `key_index: u8` pair with an `InternalKey` sum type, changing **no wire bytes** — a pure refactor whose gate is byte-equality against the current encoder. **1b** then adds `Header::WF_UNSPENDABLE_VERSION = 8`, a 1-bit `kind` field written only at version 8, the §2 derivation, rendering, the input-side recogniser, and refusals. Splitting them is deliberate: a ~59-site mechanical rename in the same diff as a funds-relevant wire change produces a diff nobody can review.

**Tech Stack:** Rust (crates `md-codec`, `md-cli`), `bitcoin` + `rust-miniscript`, `cargo nextest`.

**Spec:** `design/SPEC_liana_unspendable_internal_key.md` (GREEN at `a621cfdf`, 0C/0I after nine review passes). **Read it — this plan argues from it and does not restate it.**

**Baselines:** descriptor-mnemonic `6cbd49d8`, seedhammer `7b6f2fb`. Suite at baseline: **1400 passed / 3 skipped**.

## Global Constraints

- **Rust-primary.** This lands in `md-codec` first with vectors. The fork's Go port is stage 3 and may never lead.
- **The wire version is 8, never 5.** Single-payload md1 versions must be **even**: the auto-dispatch (`decode.rs:191-193`) reads bit 0 of the first symbol as the chunked flag, and for a single payload that bit *is* `v0`. Usable set is `{4, 8, 12}` (`header.rs:26`).
- **The encoder emits the minimum version that expresses the tree**: 8 only when some `Body::Tr` carries the Liana kind; otherwise 4. Existing plates keep their bytes.
- **Identity hashes take the version derived from the tree, never a constant.** `write_node`'s three non-test callers are `encode.rs:181` (wire), `identity.rs:90` (`WalletDescriptorTemplateId`), `identity.rs:200` (`WalletPolicyId`). A constant v4 would give kind 0 and kind 1 one identity phrase while their addresses differ.
- **Version bytes come from the render-time `--network` flag, not the wire.** md1 carries no version bytes; its TLV pubkey entries are 65 bytes, `chain code ‖ compressed pubkey`, with the pubkey at `[32..65]` (`validate.rs:331-335`, `:348`).
- **`md-cli` pins `md-codec = { path = "../md-codec", version = "=0.45.1" }` (`crates/md-cli/Cargo.toml:28`) in the same workspace.** Any md-codec version bump and the md-cli call-site repairs ship **in the same commit**, or cargo cannot resolve before rustc runs.
- **Speed without losing checks:** `cargo nextest run --locked`. Never `--release` for tests — it drops `debug_assertions`.
- **Every task ends green.** A red suite is itself a blocking finding.

---

## File Structure

| file | responsibility in this plan |
| --- | --- |
| `crates/md-codec/src/tree.rs` | `InternalKey`, `Body::Tr`, versioned `read_node`/`write_node` |
| `crates/md-codec/src/header.rs` | `WF_UNSPENDABLE_VERSION = 8`, accept `{4, 8}` |
| `crates/md-codec/src/nums.rs` | **new home for §2's derivation** — it already owns the NUMS constant |
| `crates/md-codec/src/encode.rs`, `decode.rs`, `chunk.rs` | thread the wire version |
| `crates/md-codec/src/identity.rs` | pass the derived version at both hash sites |
| `crates/md-codec/src/render.rs` | three render modes for the new kind |
| `crates/md-codec/src/validate.rs` | §6 refusals |
| `crates/md-cli/src/parse/template.rs` | `UNSPENDABLE(liana)` marker, `md encode` refusal |
| `crates/md-cli/src/cmd/decompose.rs` | the recogniser |
| `crates/md-codec/tests/liana_unspendable.rs` | **new** — §8 vectors 1, 2, 5, 6 |

**Measured blast radius for Task 1** (brace-matched against `#[cfg(test)]`, not guessed):

| crate | production sites | test sites |
| --- | --- | --- |
| `md-codec/src` | **47** | 41 |
| `md-cli/src` | **12** | 25 |

---

## Task 1: `InternalKey` sum type — behaviour-preserving, ZERO wire change (stage 1a)

**Files:**
- Modify: `crates/md-codec/src/tree.rs:49-57` (the `Body::Tr` variant), `:140-160` (write), `:268-292` (read)
- Modify: the 47 md-codec + 12 md-cli production sites listed above
- Test: `crates/md-codec/tests/internal_key_refactor.rs` (create)

**Interfaces:**
- Produces: `pub enum InternalKey { Slot(u8), NumsPoint, LianaUnspendable }` in `md_codec::tree`, and `Body::Tr { internal_key: InternalKey, tree: Option<Box<Node>> }`. Every later task consumes these names exactly.

- [ ] **Step 1: Write the byte-equality guard FIRST — it is the whole gate for this task**

Create `crates/md-codec/tests/internal_key_refactor.rs`:

```rust
//! Stage 1a's gate: the InternalKey refactor must change NO wire bytes.
//!
//! md-codec has NO template parser — `parse::template` lives in md-cli — so
//! this follows the crate's own idiom (see `examples/dump_skeleton_keys.rs`):
//! read the VENDORED md1 wire strings under `tests/vectors/`, the same bytes
//! a SeedHammer II plate carries, decode them, re-encode, and compare. That
//! is a stronger gate than re-encoding a parsed template, because the input
//! is the real wire.
use md_codec::chunk::reassemble;
use md_codec::encode::encode_payload;

#[test]
fn every_vendored_wire_vector_re_encodes_to_the_same_bytes() {
    let golden: Vec<(String, String)> =
        serde_json::from_str(include_str!("golden/pre_refactor_encodings.json"))
            .expect("golden parses");
    assert!(!golden.is_empty(), "golden is empty — Step 2 did not run");
    for (name, want_hex) in &golden {
        let chunks = load_vendored_phrase(name);          // tests/vectors/<name>.phrase.txt
        let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
        let d = reassemble(&refs).unwrap_or_else(|e| panic!("{name}: reassemble: {e}"));
        let (bytes, _bits) = encode_payload(&d).unwrap_or_else(|e| panic!("{name}: encode: {e}"));
        assert_eq!(hex::encode(&bytes), *want_hex, "wire bytes changed for {name}");
    }
}
```

- [ ] **Step 2: Generate the golden BEFORE touching any code**

```bash
cd /scratch/code/shibboleth/descriptor-mnemonic
mkdir -p crates/md-codec/tests/golden
cargo run --quiet --example dump_encodings > crates/md-codec/tests/golden/pre_refactor_encodings.json
python3 -c "import json,sys; d=json.load(open('crates/md-codec/tests/golden/pre_refactor_encodings.json')); print(len(d),'vectors'); assert d"
```

Create `crates/md-codec/examples/dump_encodings.rs` modelled on the existing
`examples/dump_skeleton_keys.rs`: enumerate `tests/vectors/*.phrase.txt`,
`reassemble` each, `encode_payload`, and print `[[name, hex], …]` as JSON.
Reuse that file's `conformance_dir()` / `keyed_phrase_files()` helpers rather
than writing new ones.

**The golden must be generated from unmodified code** — a golden captured after
the refactor proves nothing. Generate and commit it as its own commit BEFORE
Step 4 touches `tree.rs`.

- [ ] **Step 3: Run it to confirm it passes on unmodified code**

Run: `cargo nextest run --locked -p md-codec internal_key_refactor`
Expected: **PASS**. (A guard that does not pass before the change cannot prove the change was neutral.)

- [ ] **Step 4: Define the type**

In `crates/md-codec/src/tree.rs`, replace the `Tr` variant's fields:

```rust
/// The taproot internal key. Replaces the `is_nums: bool` + `key_index: u8`
/// pair, whose invariant ("is_nums = true implies key_index = 0, no wire
/// representation otherwise") was enforced only by a debug_assert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalKey {
    /// A real, spendable key at this placeholder slot.
    Slot(u8),
    /// The BIP-341 NUMS H-point, spelled as raw x-only hex. Wire kind 0.
    NumsPoint,
    /// Liana's unspendable xpub, DERIVED from the leaf keys (SPEC §2).
    /// Wire kind 1, legal only at wire version 8.
    LianaUnspendable,
}
```

and the variant itself:

```rust
Tr {
    /// The internal key. See [`InternalKey`].
    internal_key: InternalKey,
    /// Optional tap-script-tree root.
    tree: Option<Box<Node>>,
},
```

- [ ] **Step 5: Port the write path, wire-identical**

`tree.rs`, the `Body::Tr` arm of `write_node`. The `debug_assert!` is deleted — the type makes it unrepresentable.

```rust
Body::Tr { internal_key, tree } => {
    // SPEC v0.30 §7, UNCHANGED at wire version 4:
    // is_nums(1) | [key_index(kiw) iff !is_nums] | has_tree(1) | [tree]
    match internal_key {
        InternalKey::Slot(i) => {
            w.write_bits(0, 1);
            w.write_bits(u64::from(*i), key_index_width as usize);
        }
        // Stage 1a: both non-slot kinds still write exactly the v4 NUMS
        // encoding. Task 3 is what makes them differ, and only at v8.
        InternalKey::NumsPoint | InternalKey::LianaUnspendable => {
            w.write_bits(1, 1);
        }
    }
    w.write_bits(u64::from(tree.is_some()), 1);
    if let Some(t) = tree {
        write_node(w, t, key_index_width)?;
    }
}
```

- [ ] **Step 6: Port the read path**

```rust
Tag::Tr => {
    let is_nums = r.read_bits(1)? != 0;
    let internal_key = if is_nums {
        InternalKey::NumsPoint
    } else {
        InternalKey::Slot(r.read_bits(key_index_width as usize)? as u8)
    };
    let has_tree = r.read_bits(1)? != 0;
    let tree = if has_tree {
        Some(Box::new(read_node_with_depth(r, key_index_width, depth + 1)?))
    } else {
        None
    };
    Body::Tr { internal_key, tree }
}
```

- [ ] **Step 7: Migrate the remaining production sites**

Mechanical. `is_nums: true` → `internal_key: InternalKey::NumsPoint`; `is_nums: false, key_index: i` → `internal_key: InternalKey::Slot(i)`; `if !*is_nums` → `if let InternalKey::Slot(i) = internal_key`. In `compose/tr.rs:45` the expression `is_nums: ik.is_none()` becomes:

```rust
internal_key: match ik {
    Some(_) => InternalKey::Slot(0),   // numbering assigns @0; see number()
    None => InternalKey::NumsPoint,
},
```

**`md-cli` breaks at four production sites and they ship in THIS commit** (`format/json.rs:348`, `parse/reuse.rs:515`, `parse/template.rs:1604`, `:1629`). `seat/compose.rs:148` matches `Body::Tr { tree: Some(t), .. }` and is absorbed by the `..` — do not touch it.

`format/json.rs`'s serde shape is a **published v1 schema**: keep emitting `is_nums: bool` in stage 1a. Task 6 versions it.

- [ ] **Step 8: Build, then run the gate**

```bash
cargo build --locked --workspace
cargo nextest run --locked --workspace
```
Expected: **1400 passed / 3 skipped**, and `internal_key_refactor` green. Any wire-byte change fails it by vector name.

- [ ] **Step 9: Commit**

```bash
git add crates/md-codec/src crates/md-cli/src crates/md-codec/tests/internal_key_refactor.rs crates/md-codec/tests/golden crates/md-codec/examples
git commit -m "refactor: Body::Tr takes an InternalKey sum type, zero wire change

Retires the is_nums/key_index pair whose invariant was a debug_assert.
Gated by tests/internal_key_refactor.rs against a golden captured from
unmodified code: every vector encodes to identical bytes."
```

---

## Task 2: wire version 8 — the header, and threading it

**Files:**
- Modify: `crates/md-codec/src/header.rs:26-47`, `tree.rs:79`/`:196` (signatures), `encode.rs:174`/`:181`, `decode.rs:89`, `chunk.rs:70`/`:279`/`:373`, `identity.rs:90`/`:200`
- Test: `crates/md-codec/tests/wire_version_8.rs` (create)

**Interfaces:**
- Produces: `Header::WF_UNSPENDABLE_VERSION: u8 = 8`; `Descriptor::wire_version(&self) -> u8`; `write_node(w, node, kiw, wire_version: u8)` and `read_node(r, kiw, wire_version: u8)`.

- [ ] **Step 1: Write the failing tests**

`crates/md-codec/tests/wire_version_8.rs`:

```rust
use md_codec::header::Header;

#[test]
fn version_8_is_even_so_the_chunked_flag_dispatch_routes_it_as_single_payload() {
    // decode.rs:191-193 reads bit 0 of the FIRST SYMBOL as the chunked flag.
    // For a single payload the first symbol is [divergent][v3][v2][v1][v0],
    // so bit 0 IS v0 and every usable version must be even. Version 5 would
    // route a single-string plate into the chunk reassembler.
    for divergent in [false, true] {
        let sym = (u16::from(divergent) << 4) | u16::from(Header::WF_UNSPENDABLE_VERSION);
        let byte0 = ((sym << 3) & 0xFF) as u8;
        assert_eq!(byte0 >> 3 & 1, 0, "version {} dispatches as CHUNKED",
                   Header::WF_UNSPENDABLE_VERSION);
    }
}

#[test]
fn the_decoder_accepts_4_and_8_and_refuses_everything_else() {
    for v in 0u8..16 {
        let ok = matches!(v, 4 | 8);
        assert_eq!(Header::is_supported_version(v), ok, "version {v}");
    }
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo nextest run --locked -p md-codec wire_version_8`
Expected: FAIL — `WF_UNSPENDABLE_VERSION` and `is_supported_version` do not exist.

- [ ] **Step 3: Implement the header**

`header.rs`:

```rust
    /// The version that carries a non-`NumsPoint` taproot internal key
    /// (SPEC §3c). Usable WF-redesign versions are {4, 8, 12} because the
    /// single-payload auto-dispatch reads v0 as the chunked flag, so every
    /// usable version must be EVEN. This spends 8 and leaves 12 as the last.
    pub const WF_UNSPENDABLE_VERSION: u8 = 8;

    /// Versions this build decodes.
    pub fn is_supported_version(v: u8) -> bool {
        v == Self::WF_REDESIGN_VERSION || v == Self::WF_UNSPENDABLE_VERSION
    }
```

and in `Header::read`, replace the exact-match with:

```rust
        if !Self::is_supported_version(version) {
            return Err(Error::WireVersionMismatch { got: version });
        }
```

- [ ] **Step 4: Add `Descriptor::wire_version` and thread it**

```rust
impl Descriptor {
    /// The minimum wire version that can express this tree (SPEC §3d).
    /// NEVER a constant: `identity.rs`'s two hash sites call `write_node`
    /// directly, and a constant v4 would make kind 0 and kind 1 hash
    /// identically while their addresses differ.
    pub fn wire_version(&self) -> u8 {
        fn needs_v8(n: &Node) -> bool {
            match &n.body {
                Body::Tr { internal_key, tree } => {
                    *internal_key == InternalKey::LianaUnspendable
                        || tree.as_deref().is_some_and(needs_v8)
                }
                Body::Children(cs) | Body::Variable { children: cs, .. } => {
                    cs.iter().any(needs_v8)
                }
                _ => false,
            }
        }
        if needs_v8(&self.tree) {
            Header::WF_UNSPENDABLE_VERSION
        } else {
            Header::WF_REDESIGN_VERSION
        }
    }
}
```

Then add `wire_version: u8` as the last parameter of `write_node` and `read_node`, and pass:
- `encode.rs:181` → `d.wire_version()`; `encode.rs:174`'s header constant → the same value.
- `identity.rs:90` and `:200` → `d.wire_version()`. **This is the funds-relevant one.**
- `decode.rs:89` → the version `Header::read` just returned.
- `chunk.rs:279` (writer) → the payload's version, so chunk headers agree with their own payload; `chunk.rs:70` and `:373` accept `{4, 8}`.

- [ ] **Step 5: Run the tests**

Run: `cargo nextest run --locked --workspace`
Expected: **PASS**, 1400+ and both new tests green. Wire bytes are still unchanged — nothing emits version 8 yet.

- [ ] **Step 6: Commit**

```bash
git add crates/md-codec/src crates/md-codec/tests/wire_version_8.rs
git commit -m "feat(wire): accept version 8 and derive the version from the tree

Version 8 is EVEN because the single-payload auto-dispatch reads v0 as
the chunked flag. Identity hashes now take the derived version, never a
constant, so kind 0 and kind 1 cannot share a WalletPolicyId."
```

---

## Task 3: the `kind` bit, written only at version 8

**Files:** Modify `crates/md-codec/src/tree.rs` (both Tr arms). Test: extend `tests/wire_version_8.rs`.

**Interfaces:** Consumes `InternalKey`, `wire_version`. Produces the on-wire encoding `Tag::Tr | is_nums(1) | [kind(1) iff is_nums] | [key_index(kiw) iff !is_nums] | has_tree(1) | [tree]` at version 8.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn a_liana_kind_tree_round_trips_at_v8_and_a_nums_tree_still_encodes_as_v4() {
    let liana = tr_descriptor(InternalKey::LianaUnspendable);
    let nums  = tr_descriptor(InternalKey::NumsPoint);
    assert_eq!(liana.wire_version(), 8);
    assert_eq!(nums.wire_version(), 4);
    for d in [&liana, &nums] {
        let bytes = encode_payload(d).expect("encode");
        let back = decode_payload(&bytes).expect("decode");
        assert_eq!(&back, d, "round trip");
    }
}

#[test]
fn a_version_8_payload_is_refused_by_a_version_4_only_decoder() {
    let bytes = encode_payload(&tr_descriptor(InternalKey::LianaUnspendable)).unwrap();
    // Simulate the shipped decoder: only version 4 supported.
    let err = decode_payload_restricted(&bytes, &[4]).unwrap_err();
    assert!(matches!(err, Error::WireVersionMismatch { got: 8 }),
            "must fail closed naming the version, got {err:?}");
}

#[test]
fn kind_0_and_kind_1_over_the_same_tree_encode_to_DIFFERENT_bytes() {
    let a = encode_payload(&tr_descriptor(InternalKey::NumsPoint)).unwrap();
    let b = encode_payload(&tr_descriptor(InternalKey::LianaUnspendable)).unwrap();
    assert_ne!(a, b, "the two kinds must be distinguishable on the wire");
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo nextest run --locked -p md-codec wire_version_8`
Expected: FAIL — both kinds currently encode identically (Task 1 Step 5 made them so deliberately).

- [ ] **Step 3: Implement — write side**

```rust
    match internal_key {
        InternalKey::Slot(i) => {
            w.write_bits(0, 1);
            w.write_bits(u64::from(*i), key_index_width as usize);
        }
        InternalKey::NumsPoint | InternalKey::LianaUnspendable => {
            w.write_bits(1, 1);
            // SPEC §3d: the kind bit exists ONLY at version 8. At version 4
            // the stream is byte-identical to what shipped.
            if wire_version == Header::WF_UNSPENDABLE_VERSION {
                let kind = u64::from(*internal_key == InternalKey::LianaUnspendable);
                w.write_bits(kind, 1);
            }
        }
    }
```

- [ ] **Step 4: Implement — read side**

```rust
    let internal_key = if is_nums {
        if wire_version == Header::WF_UNSPENDABLE_VERSION {
            if r.read_bits(1)? != 0 { InternalKey::LianaUnspendable } else { InternalKey::NumsPoint }
        } else {
            InternalKey::NumsPoint
        }
    } else {
        InternalKey::Slot(r.read_bits(key_index_width as usize)? as u8)
    };
```

- [ ] **Step 5: Run the full suite AND the stage-1a guard**

Run: `cargo nextest run --locked --workspace`
Expected: PASS, including `internal_key_refactor` — **v4 bytes must still be identical**. If that guard fails here, the kind bit leaked into version 4.

- [ ] **Step 6: Commit**

```bash
git add crates/md-codec/src crates/md-codec/tests/wire_version_8.rs
git commit -m "feat(wire): the internal-key kind bit at version 8

Kind 0 = raw H (what v4 means), kind 1 = Liana's unspendable xpub. The
bit is written only at v8, so every existing plate keeps its bytes, and
a v4-only decoder refuses a v8 payload naming the version."
```

---

## Task 4: §2's derivation — `liana_unspendable_xpub`

**Files:** Modify `crates/md-codec/src/nums.rs`. Test: `crates/md-codec/tests/liana_unspendable.rs` (create).

**Interfaces:** Produces
`pub fn liana_unspendable_xpub(leaf_pubkeys: &[[u8; 33]], network: bitcoin::Network) -> bitcoin::bip32::Xpub`.

- [ ] **Step 1: Write the failing test from the MEASURED goldens**

The four shapes Liana accepted are in `design/evidence/composer-fable-r0/fable-liana-parse-in.jsonl` (variant `liana-unspendable-xpub`). Extract leaf pubkeys from the `md` variant and assert the derived xpub equals the golden.

```rust
#[test]
fn the_recipe_reproduces_every_liana_accepted_golden_xpub() {
    for case in GOLDEN_CASES {              // name, leaf pubkeys (hex), expected xpub
        let leaves: Vec<[u8; 33]> = case.leaf_pubkeys.iter()
            .map(|h| <[u8; 33]>::try_from(hex::decode(h).unwrap().as_slice()).unwrap())
            .collect();
        let got = liana_unspendable_xpub(&leaves, bitcoin::Network::Bitcoin);
        assert_eq!(got.to_string(), case.expected_xpub, "{}", case.name);
    }
}

#[test]
fn the_chain_code_depends_on_the_ORDERED_MULTISET_of_leaf_keys_not_the_tree() {
    // kofn-recovery, tiered-recovery and decaying-multisig over the same four
    // keys in the same order share one internal key. Harmless — the output key
    // commits to the tree through the taproot tweak — but pin it so nobody
    // "fixes" it into a tree-dependent hash.
    let k = GOLDEN_CASES.iter().find(|c| c.name == "preset-kofn-recovery-tr").unwrap();
    let t = GOLDEN_CASES.iter().find(|c| c.name == "preset-tiered-recovery-tr").unwrap();
    assert_eq!(k.leaf_pubkeys, t.leaf_pubkeys, "fixture precondition");
    assert_eq!(k.expected_xpub, t.expected_xpub);
}

#[test]
fn sorting_or_deduplicating_the_leaves_produces_a_DIFFERENT_xpub() {
    // Liana's recipe is NOT sorted and NOT deduplicated (analysis.rs:398-430).
    // The abandoned BIPs PR #1746 recipe is, and it is a different wallet.
    let c = &GOLDEN_CASES[0];
    let mut leaves: Vec<[u8; 33]> = c.leaf_pubkeys.iter().map(parse33).collect();
    let straight = liana_unspendable_xpub(&leaves, bitcoin::Network::Bitcoin);
    leaves.sort(); leaves.dedup();
    let sorted = liana_unspendable_xpub(&leaves, bitcoin::Network::Bitcoin);
    assert_ne!(straight.to_string(), sorted.to_string());
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo nextest run --locked -p md-codec liana_unspendable`
Expected: FAIL — `liana_unspendable_xpub` does not exist.

- [ ] **Step 3: Implement**

```rust
/// Liana's unspendable internal key (SPEC §2, `analysis.rs:398-430`).
///
/// `chain_code = sha256(leaf pubkeys concatenated in DESCRIPTOR LEFT-TO-RIGHT
/// order)` — **not sorted, not deduplicated** — over each leaf key's 33-byte
/// compressed pubkey. In md that is bytes `[32..65]` of the 65-byte TLV entry
/// (`chain code ‖ compressed pubkey`), NOT a base58 payload, and md1 carries
/// no version bytes: `network` comes from the render-time `--network` flag.
///
/// Order is WIRE ORDER, never the derived-key-sorted order the device's
/// address builder uses for `sortedmulti_a` (§6 refuses that shape for
/// exactly this reason).
pub fn liana_unspendable_xpub(
    leaf_pubkeys: &[[u8; 33]],
    network: bitcoin::Network,
) -> bitcoin::bip32::Xpub {
    use bitcoin::hashes::{sha256, Hash};
    debug_assert!(!leaf_pubkeys.is_empty(), "SPEC §6: unreachable, pinned by a test");
    let mut concat = Vec::with_capacity(leaf_pubkeys.len() * 33);
    for pk in leaf_pubkeys {
        concat.extend_from_slice(pk);
    }
    bitcoin::bip32::Xpub {
        public_key: bitcoin::secp256k1::PublicKey::from_slice(
            &hex_lit::hex!("0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"),
        )
        .expect("the BIP-341 NUMS point is a valid compressed pubkey"),
        chain_code: bitcoin::bip32::ChainCode::from(
            sha256::Hash::hash(&concat).to_byte_array(),
        ),
        depth: 0,
        parent_fingerprint: Default::default(),
        child_number: bitcoin::bip32::ChildNumber::from_normal_idx(0).unwrap(),
        network: network.into(),
    }
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo nextest run --locked -p md-codec liana_unspendable`
Expected: **PASS**, all three.

- [ ] **Step 5: Pin the empty-concat invariant (SPEC §6)**

§6 states this as an *invariant, not a refusal*, because it is unreachable: the template parser requires at least one `@i` placeholder, and under `tr` with a NUMS-family internal key every placeholder lives in a leaf. Add a test that demonstrates **why**, so that if the placeholder rule ever changes, this fails instead of admitting `sha256("")`:

This one belongs in **md-cli**'s tests, not md-codec's — the guard it pins is
the *template parser's*, and md-codec has no parser. Put it in
`crates/md-cli/tests/liana_input_side.rs` (created in Task 6) or a small
`crates/md-cli/tests/nums_invariants.rs`:

```rust
#[test]
fn a_tr_with_no_leaf_keys_cannot_be_constructed_so_the_concat_is_never_empty() {
    // Not a refusal — a pinned invariant. If this ever starts SUCCEEDING,
    // liana_unspendable_xpub would derive sha256("") for every such wallet,
    // and every one of them would share one internal key.
    let err = md_err(&["encode",
        "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0)"]);
    assert!(err.contains("no @i placeholders"), "got {err}");
}
```

- [ ] **Step 6: Commit**

```bash
git add crates/md-codec/src/nums.rs crates/md-codec/tests/liana_unspendable.rs
git commit -m "feat: Liana's unspendable internal-key derivation (SPEC §2)

sha256 over the leaf pubkeys in descriptor left-to-right order, not
sorted and not deduplicated. Reproduces all four Liana-accepted golden
xpubs byte-for-byte; sorting or deduping is pinned as a DIFFERENT key."
```

---

## Task 5: rendering the new kind (SPEC §4)

**Files:** Modify `crates/md-codec/src/render.rs:185-205`, `to_miniscript.rs:361-366`. Test: extend `tests/liana_unspendable.rs`, `tests/render_abstract.rs`.

**Interfaces:** Consumes `liana_unspendable_xpub`. Produces the three render modes of §4's table.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn the_keyed_descriptor_renders_the_derived_literal_xpub() {
    let d = descriptor_with_real_keys_at_liana_kind();
    let s = md_codec::render::render_descriptor(&d, bitcoin::Network::Bitcoin).unwrap();
    assert!(s.starts_with("tr(xpub661MyMwAqRbcFswVugWF"), "got {s}");
    assert!(!s.contains("50929b74"), "must not spell raw H at kind 1");
}

#[test]
fn both_keyless_modes_render_the_marker_because_the_xpub_needs_seated_keys() {
    let d = descriptor_at_liana_kind();
    assert!(md_codec::render::descriptor_to_template(&d).unwrap()
        .contains("UNSPENDABLE(liana)"));
    assert!(md_codec::render::descriptor_to_abstract_template(&d).unwrap()
        .contains("UNSPENDABLE(liana)"));
}

#[test]
fn kind_0_and_kind_1_produce_DIFFERENT_skeleton_keys() {
    // A shared SkeletonKey would be a false-evidence-match between two
    // wallets with different addresses.
    let a = md_codec::skeleton::skeleton_key(&descriptor_at_nums_kind()).unwrap();
    let b = md_codec::skeleton::skeleton_key(&descriptor_at_liana_kind()).unwrap();
    assert_ne!(a, b);
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo nextest run --locked -p md-codec liana_unspendable`
Expected: FAIL — kind 1 currently renders raw `H` in all three modes.

- [ ] **Step 3: Implement**

In `render.rs`'s `Body::Tr` arm:

```rust
    match internal_key {
        InternalKey::Slot(i) => render_key(*i, default_usp, overrides, out)?,
        InternalKey::NumsPoint => out.push_str(NUMS_H_POINT_X_ONLY_HEX),
        InternalKey::LianaUnspendable => match ctx.mode {
            // The chain code is a function of the SEATED leaf keys, so the
            // xpub does not exist in a keyless form. The marker is the only
            // correct spelling, not a shortcut.
            Mode::Template | Mode::Abstract => out.push_str(LIANA_UNSPENDABLE_MARKER),
            Mode::Keyed => {
                let leaves = collect_leaf_pubkeys_in_wire_order(node, overrides)?;
                out.push_str(&liana_unspendable_xpub(&leaves, ctx.network).to_string());
                out.push_str("/<0;1>/*");
            }
        },
    }
```

with `pub const LIANA_UNSPENDABLE_MARKER: &str = "UNSPENDABLE(liana)";` in `nums.rs`.

- [ ] **Step 4: Run the tests**

Run: `cargo nextest run --locked --workspace`
Expected: **PASS**.

- [ ] **Step 5: Extend the render-reparse fixpoint corpus to `tr` kind 1**

`crates/md-cli/src/format/text.rs:269-274` asserts every rendered template re-parses, and its corpus is `wsh`-only — so it would stay green while the invariant it names is false for `tr` kind 1. Add a kind-1 case to that corpus. It will fail until Task 6 adds the marker's parse rule; that is the correct order.

- [ ] **Step 6: Commit**

```bash
git add crates/md-codec/src crates/md-cli/src/format/text.rs crates/md-codec/tests
git commit -m "feat(render): the derived xpub when keyed, UNSPENDABLE(liana) when not

The chain code is a function of the seated leaf keys, so no keyless form
can carry it. Kind 0 and kind 1 now produce different SkeletonKeys."
```

---

## Task 6: the input side (SPEC §4a) and the JSON schema bump

**Files:** Modify `crates/md-cli/src/parse/template.rs:1589-1632`, `crates/md-cli/src/cmd/decompose.rs`, `crates/md-cli/src/format/json.rs:320-330`, `docs/json-schema-v1.md`. Test: `crates/md-cli/tests/liana_input_side.rs` (create).

**Interfaces:** Consumes `liana_unspendable_xpub`, `LIANA_UNSPENDABLE_MARKER`.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn decompose_RECOGNISES_a_real_liana_descriptor_and_gives_it_no_slot() {
    let out = md(&["decompose", LIANA_KOFN_DESCRIPTOR, "--emit", "template"]);
    assert!(out.contains("UNSPENDABLE(liana)"), "got {out}");
    assert!(!out.contains("@0/<0;1>/*,{"), "the derived key must take NO slot");
    assert!(!out.contains("state NO origin"), "no phantom-slot warning");
}

#[test]
fn decompose_keeps_TODAYS_behaviour_for_an_origin_less_key_that_is_NOT_lianas() {
    // The property that makes a slot phantom is NO ORIGIN, not "not Liana's".
    // A real spendable internal key with no recorded origin is @0 and correct.
    let out = md(&["decompose", ORIGINLESS_SPENDABLE_TR, "--emit", "template"]);
    assert!(out.contains("tr(@0/"), "must still be a slot: {out}");
    assert!(out.contains("state NO origin"), "must still annotate");
}

#[test]
fn md_encode_refuses_a_literal_xpub_in_the_internal_key_position_CLEANLY() {
    let err = md_err(&["encode", LIANA_KOFN_DESCRIPTOR_AS_TEMPLATE]);
    assert!(!err.contains("internal:"), "internal invariant leaked: {err}");
    assert!(err.contains("UNSPENDABLE(liana)"), "must name the working spelling");
    assert!(err.contains("decompose"), "must name the other working route");
}

#[test]
fn the_marker_round_trips_through_compose_then_encode() {
    let tpl = md(&["compose", "--wrapper", "tr", "--preset",
                   "kofn-recovery,2of3,older=26280", "--unspendable", "liana"]);
    assert!(tpl.contains("UNSPENDABLE(liana)"));
    md(&["encode", tpl.lines().next().unwrap(), "--path", "bip48"]); // must not error
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo nextest run --locked -p md-cli liana_input_side`
Expected: FAIL — `md encode` emits `internal: synthetic key … not found in key map`, and decompose makes a phantom `@0`.

- [ ] **Step 3: Implement the three surfaces, which are NOT the same rule**

Per §4a's table:

- **`md decompose`** holds the real leaf keys, so it **recomputes** §2 and accepts kind 1 only on a byte match. On a mismatch it falls back to **today's** annotated-slot behaviour — it must not gain a new refusal, or it locks out libnunchuk's PR-1746 form and real origin-less spendable keys.
- **The template grammar** gains a substitution rule for `UNSPENDABLE(liana)` in `walk_tr`, before the `NUMS_H_POINT_X_ONLY_HEX` comparison at `:1601`.
- **`md encode` with a literal xpub** **cannot** recompute: `substitute_synthetic` (`:1047-1084`) replaces every `@i` with a `sha256(b"md-v0.15" ‖ i ‖ depth)` placeholder *before* `walk_tr` sees the tree, so the leaves are synthetic and the match can never succeed. It **refuses**, naming both working spellings. This replaces the internal-error leak, which is the actual defect.

- [ ] **Step 4: Version the JSON schema**

`format/json.rs`'s `Tr { is_nums: bool, … }` is a **published v1 contract**. Replace with `internal_key: "slot" | "nums" | "liana_unspendable"` plus `key_index` only for `slot`, bump the schema version, and record the break in `docs/json-schema-v1.md`. Do not slip it in silently.

- [ ] **Step 5: Run everything, including the fixpoint from Task 5 Step 5**

Run: `cargo nextest run --locked --workspace`
Expected: **PASS**, and `text.rs`'s render-reparse fixpoint now green for `tr` kind 1.

- [ ] **Step 6: Commit**

```bash
git add crates/md-cli/src crates/md-cli/tests/liana_input_side.rs docs/json-schema-v1.md
git commit -m "feat(input): md RECOGNISES the Liana form, it no longer only emits it

decompose recomputes the recipe and gives the derived key no slot; the
template grammar takes UNSPENDABLE(liana); md encode refuses a literal
xpub cleanly instead of leaking an internal invariant. A non-matching
origin-less key keeps today's annotated-slot behaviour."
```

---

## Task 7: §6's refusals

**Files:** Modify `crates/md-codec/src/validate.rs`. Test: extend `tests/liana_unspendable.rs`.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn kind_1_is_refused_on_a_sortedmulti_a_leaf() {
    // The device's address builder sorts that leaf's keys by DERIVED key per
    // index, while §2 hashes them in wire order. A belt against the port
    // error; the only composer shape that emits it is out of scope anyway.
    let err = validate(&tr_liana_with_sortedmulti_a_leaf()).unwrap_err();
    assert!(matches!(err, Error::UnspendableWithSortedMultiA));
}

#[test]
fn kind_1_is_refused_when_the_use_site_is_not_0_1() {
    // Liana pairs multipath alternatives positionally and derives the
    // internal key at 0/i; a use-site-following device would derive at 2/i.
    let err = validate(&tr_liana_at_use_site("<2;3>")).unwrap_err();
    assert!(matches!(err, Error::UnspendableUseSiteNotCanonical));
}

#[test]
fn kind_1_nested_under_wsh_is_refused() {
    let err = validate(&wsh_wrapping_tr_liana()).unwrap_err();
    assert!(matches!(err, Error::UnspendableNotRootTr));
}

#[test]
fn version_8_with_every_tr_at_kind_0_is_refused_at_ENCODE() {
    // The minimum-version rule is enforced on the encode side only, so old
    // payloads never become invalid at decode.
    let err = encode_payload_at_version(&tr_descriptor(InternalKey::NumsPoint), 8).unwrap_err();
    assert!(matches!(err, Error::NonMinimalWireVersion { .. }));
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo nextest run --locked -p md-codec liana_unspendable`
Expected: FAIL — none of these error variants exist.

- [ ] **Step 3: Implement the four refusals and their `Error` variants in `error.rs`.**

- [ ] **Step 4: Run**

Run: `cargo nextest run --locked --workspace`
Expected: **PASS**.

- [ ] **Step 5: Commit**

```bash
git add crates/md-codec/src
git commit -m "feat(validate): SPEC §6's refusals for the Liana internal-key kind"
```

---

## Task 8: §8's acceptance vectors 1, 2, 5, 6, 7

**Files:** Extend `crates/md-codec/tests/liana_unspendable.rs`; create `crates/md-codec/tests/identity_distinctness.rs`.

- [ ] **Step 1: Descriptor equality against the four Liana-ACCEPTED goldens, checksum included**

The gate is a deduction from a measurement: those exact strings were fed to `LianaDescriptor::from_str` at **v8.0 and v15.0** and came back ACCEPT. Byte-identical output therefore imports.

```rust
#[test]
fn md_emits_byte_identical_descriptors_for_the_four_liana_accepted_shapes() {
    for case in GOLDEN_ACCEPTED {           // 4 cases
        let got = md(&["descriptor", /* the kind-1 chunks for this shape */]);
        assert_eq!(got.trim(), case.descriptor_with_checksum,
                   "{} — checksum included, it is part of the gate", case.name);
    }
}
```

- [ ] **Step 2: Address equality, three-way, and DIFFERENCE from kind 0**

```rust
#[test]
fn addresses_agree_with_lianas_own_and_differ_from_kind_0() {
    for case in GOLDEN_ACCEPTED {
        for i in 0..3 {
            assert_eq!(md_address(case, i, Chain::Receive), case.liana_receive[i]);
            assert_eq!(md_address(case, i, Chain::Change),  case.liana_change[i]);
            // The half that proves the two kinds are DIFFERENT WALLETS —
            // which is why F-449 is a wire item, not an export substitution.
            assert_ne!(md_address(case, i, Chain::Receive),
                       md_address_at_nums_kind(case, i, Chain::Receive));
        }
    }
}
```

`case.liana_receive` / `liana_change` come from `fable-liana-parse-out-v15.jsonl`.

- [ ] **Step 3: Identity distinctness AND stability**

```rust
#[test]
fn kind_0_and_kind_1_get_different_ids_and_a_different_12_word_phrase() {
    let a = descriptor_at_nums_kind();
    let b = descriptor_at_liana_kind();
    assert_ne!(wallet_policy_id(&a), wallet_policy_id(&b));
    assert_ne!(template_id(&a), template_id(&b));
    assert_ne!(to_phrase(&a), to_phrase(&b));
}

#[test]
fn every_existing_v4_identity_is_byte_preserved() {
    let golden: Vec<(String, String)> =
        serde_json::from_str(include_str!("golden/pre_refactor_ids.json")).unwrap();
    for (name, id) in golden { assert_eq!(compute_id_by_name(&name), id, "{name}"); }
}
```

Capture `pre_refactor_ids.json` the same way as Task 1's golden: **from unmodified code, before Task 2**.

- [ ] **Step 4: Run**

Run: `cargo nextest run --locked --workspace`
Expected: **PASS**.

- [ ] **Step 5: Commit**

```bash
git add crates/md-codec/tests
git commit -m "test: SPEC §8 vectors 1, 2, 5, 6, 7 for the Liana internal key"
```

---

## Task 9: §8.10's mutation pass — prove the gates can FAIL

**Files:** `design/agent-reports/f449-stage1-mutations.md` (create).

A proof in a transcript is not a gate. Each mutation below is applied to the **source**, the suite is run, and the result recorded. **A mutation that leaves the suite green is a missing test, not a curiosity** — add the test, then re-run.

- [ ] **Step 1: Apply each mutation, one at a time, and record the failing test**

| # | mutation | must be caught by |
| --- | --- | --- |
| 1 | reverse the leaf order in `liana_unspendable_xpub`'s concat | Task 4 golden test |
| 2 | `leaves.sort()` before hashing | Task 4 sort/dedup test |
| 3 | `leaves.dedup()` before hashing | Task 4 sort/dedup test |
| 4 | `wire_version()` returns `WF_REDESIGN_VERSION` always | Task 3 round trip + Task 8 identity distinctness |
| 5 | render kind 1 as `NUMS_H_POINT_X_ONLY_HEX` | Task 5 + Task 8 descriptor equality |
| 6 | pass a constant `4` at `identity.rs:90` and `:200` | **Task 8 identity distinctness** — the funds-relevant one |
| 7 | write the kind bit at version 4 too | **Task 1's byte-equality guard** |
| 8 | drop the `sortedmulti_a` refusal | Task 7 |
| 9 | drop the use-site refusal | Task 7 |

- [ ] **Step 2: For each, assert the mutation APPLIED**

A no-op edit reads as a pass. Confirm the mutated line actually ran — put a `panic!` in the mutated branch first and see the suite fail, then replace it with the real mutation.

- [ ] **Step 3: Record all nine results verbatim, then revert every mutation**

```bash
git diff --stat            # must be empty except the report
cargo nextest run --locked --workspace   # 1400+ green
```

- [ ] **Step 4: Commit**

```bash
git add design/agent-reports/f449-stage1-mutations.md
git commit -m "test: stage 1 mutation pass — nine mutations, each caught

Proves the §8 vectors can fail. Mutation 6 (a constant version at the
identity hash sites) is the funds-relevant one: it collapses kind 0 and
kind 1 onto one WalletPolicyId and one 12-word phrase."
```

---

## Task 10: version bump and release, in ONE commit

**Files:** `crates/md-codec/Cargo.toml`, `crates/md-cli/Cargo.toml:28`, `Cargo.lock`, `CHANGELOG.md`.

- [ ] **Step 1: Bump md-codec to 0.46.0 and md-cli's exact pin IN THE SAME EDIT**

Breaking: `Body::Tr`'s shape changed, the JSON schema changed, and the decoder accepts a new version. Pre-1.0 convention makes the second component the breaking axis.

```bash
sed -i 's/^version = "0.45.1"/version = "0.46.0"/' crates/md-codec/Cargo.toml
sed -i 's/version = "=0.45.1"/version = "=0.46.0"/' crates/md-cli/Cargo.toml
cargo update -w
```

**These cannot be split.** md-cli's `=0.45.1` becomes unsatisfiable the moment md-codec moves, and cargo fails to resolve *before rustc runs* — so a split commit is red on its own.

- [ ] **Step 2: Write the CHANGELOG entry**

Name the wire change, the JSON schema break, and that existing v4 plates are unaffected.

- [ ] **Step 3: Run the full gate — all four commands CI runs**

```bash
cargo build --locked --workspace
cargo nextest run --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo doc --locked --no-deps --workspace
```

**`cargo doc` is not optional** — a `cargo doc` failure reached main once in this repo because a local gate ran three commands while CI ran four.

- [ ] **Step 4: Commit**

```bash
git add -u && git commit -m "chore: md-codec 0.46.0 — the Liana unspendable internal key

Breaking: Body::Tr takes an InternalKey, the decoder accepts wire version
8, and md decode --json's Tr shape changed. Existing version-4 plates are
byte-unaffected and keep meaning raw H."
```

---

## Self-Review

**1. Spec coverage.** §2 → Task 4. §3a/§3c/§3d → Tasks 2-3. §3e → Task 2 Step 4 (Rust half; the Go half is stage 3). §3f → Task 1. §4 → Task 5. §4a → Task 6. §5 → Task 1 (no slot, by construction). §6 → Task 7, plus the §6 invariant at Task 4 Step 5. §8 items 1/2/5/6/7 → Task 8; item 10 → Task 9. §9 stage 1a/1b's pin rule → Task 10.

**Deliberately NOT in this plan**, and owned elsewhere by the spec: §6a's three messages (stages 1b/2/3 — the `WireVersionMismatch` Display is the only 1b piece and rides Task 2), §7/§7a (stages 3-4), §8.3's device leg (stage 4), §8.4's Go leg (stage 3), §8.8's live Liana run (stage 2), §8.9's operator-facing rows (stages 2-4a), §8b/§9a (stage 4a).

**2. Placeholder scan.** No TBDs. Every code step carries real code. The two goldens (`pre_refactor_encodings.json`, `pre_refactor_ids.json`) have explicit generation steps that run *before* the code they protect changes.

**3. Type consistency.** `InternalKey::{Slot,NumsPoint,LianaUnspendable}`, `Body::Tr { internal_key, tree }`, `Header::WF_UNSPENDABLE_VERSION`, `Descriptor::wire_version()`, `liana_unspendable_xpub(&[[u8;33]], Network) -> Xpub`, `LIANA_UNSPENDABLE_MARKER` — used identically in Tasks 1-9.

**One known gap, stated rather than hidden:** Task 8's `GOLDEN_ACCEPTED` fixture (descriptors, checksums, and Liana's own address triples) must be extracted from `design/evidence/composer-fable-r0/fable-liana-parse-{in,out-v15}.jsonl` into a Rust fixture. That extraction is mechanical but is **not** written out here; the implementer does it as Task 8 Step 0 and commits the generator alongside, so the fixture is reproducible rather than transcribed.
