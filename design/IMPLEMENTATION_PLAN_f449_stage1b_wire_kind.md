# F-449 Stage 1b — Wire Version 8 and the Liana Internal-Key Kind

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Put a second taproot internal-key kind on the md1 wire — Liana's unspendable xpub, derived from the leaf keys — carried on wire version 8, so a `tr` policy with a multi-key primary imports into Liana.

**Architecture:** Stage 1a already replaced `Body::Tr`'s field pair with `InternalKey` and changed no wire bytes. This stage makes `LianaUnspendable` real: a new even wire version, a 1-bit kind field written only at that version, §2's derivation, §4's rendering, §4a's input recogniser, and §6's refusals. Every existing v4 plate keeps its bytes and its meaning.

**Tech Stack:** Rust (`md-codec`, `md-cli`), `bitcoin` + `rust-miniscript`, `cargo nextest`.

**Spec:** `design/SPEC_liana_unspendable_internal_key.md` (GREEN, 0C/0I after nine passes). **Read it — this plan argues from it and does not restate it.**

**Predecessor:** `design/IMPLEMENTATION_PLAN_f449_stage1a_internal_key.md`, shipped at `37367c1f` on `descriptor-mnemonic` main. Baseline there: `phase-gate.sh` all six green, 1439 passed / 3 skipped under `--all-features`.

**Status:** **r9 — GREEN, ready for implementation.** Nine rounds: 4C/7I -> 1C/5I -> 0C/6I -> 0C/3I -> 0C/2I -> 0C/1I -> 0C/1I -> 0C/2I -> **0C/0I**. Reports: `design/agent-reports/f449-plan-stage1b-{r0..r8}.md`. Eight Minors and three Nits remain open and are named in r8's report; they are recorded, not dropped.

## Global Constraints

- **The wire version is 8, never 5.** Single-payload md1 versions must be **even**: the auto-dispatch (`decode.rs:191-193`) reads bit 0 of the first symbol as the chunked flag, and for a single payload that bit *is* `v0`. Usable set `{4, 8, 12}` (`header.rs:26`). This spends 8 and leaves 12 as the format's last generation.
- **The encoder emits the minimum version that expresses the tree**: 8 only when some `Body::Tr` carries `LianaUnspendable`; otherwise 4.
- **Identity hashes take the version derived from the tree, never a constant.** `write_node`'s three non-test callers are `encode.rs:181` (wire), `identity.rs:90` (`WalletDescriptorTemplateId`), `identity.rs:200` (`WalletPolicyId`). A constant v4 would give kind 0 and kind 1 one identity phrase while their addresses differ.
- **§6's refusals hook the ENCODE/admission path and MUST NOT reach decode.** `encode.rs:116-131` records a measured 2026-09-19 regression where mint-side refusals leaked into decode and a shipped 2-of-2 **stopped reading**. A refusal on the decode side here makes existing plates unreadable.
- **There is no function called `validate`.** `validate.rs` exposes ten production `validate_*` functions, reached from `encode_payload_inner` (`encode.rs:152-172`, two behind `Admission::Enforce`) and `decode_payload_with_opts` (`decode.rs:118-153`).
- **Network comes from the render-time selector, not the wire.** md1's TLV pubkey entries are 65 bytes, `chain code ‖ compressed pubkey`, pubkey at `[32..65]` (`validate.rs:331-335`, `:348`).
- **Use the PINNED clippy.** `rust-toolchain.toml` pins 1.85.0; a bare `cargo clippy` runs 0.1.98 and fails at baseline with lints CI never runs. `export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH`, then `cargo clippy --version` must print **0.1.85**.
- **Run `./scripts/phase-gate.sh`, never a hand-typed command list** — it sets `RUSTDOCFLAGS="-D warnings"` (`:39`), which a typed list drops, and without it `cargo doc` warns and exits 0.
- **`md(...)` in md-cli's tests returns `(String, String, i32)` and does NOT panic** (`cli_bip388_double_wildcard.rs:35-45`). **Every call must bind all three and assert the exit code.** A bare `md(&[...]);` asserts nothing — that defect reached this plan three times, in two different tests, and is why every CLI call here destructures.
- **`fuzz/` is its own workspace.** Run `( cd fuzz && cargo check --all-targets )` or `gen_corpus.rs` breaks silently.

---

## Citing findings: ALWAYS prefix with the round

Reports of this plan number their findings independently, and r0's `I1..I7`
collided with r1's `I-1..I-5`. A fold commit that said *"I2, I3, I4, I7
confirmed addressed"* meant r0's set; r1's **I-2, I-3 and I-5 were silently
never folded**, and the next dispatch brief repeated the omission by asking
only about r1's I-1 and I-4.

**Cite every finding as `<round>/<label>`** — `r0/I3`, `r1/I-3` — in folds, in
commit messages and in dispatch briefs. Before folding a report, list its
findings and tick them off individually; do not carry a remembered subset.

## Findings banked against this stage before it was written

Each was found by a review of the combined or stage-1a plan and must be answered here, not rediscovered.

| # | finding | what this plan does |
| --- | --- | --- |
| **G-1** | **GATING.** There are **FIVE** `NumsPoint \| LianaUnspendable` or-patterns in the tree, not four. **Four are non-encode and have no gate** — `render.rs:192`, `to_miniscript.rs:341`, `policy_shape.rs:251`, `md-cli/src/format/json.rs:356`. The fifth, `tree.rs:160`, is the **write path** and is owned by Task 3, which replaces it with the kind bit; it is not a missed site, but anyone counting will find five. The four non-encode ones give this stage **no non-exhaustive-match error**. Proven: splitting them so `LianaUnspendable` hits `unreachable!()` left the suite at 1439/3 **unchanged**. A site forgotten here silently yields a NUMS taproot output key — a **wrong address**. | Task 5 visits all four **by name**: `render.rs:192`, `to_miniscript.rs:341`, `policy_shape.rs:251`, `md-cli/src/format/json.rs:356`. Task 9 mutates each and requires a RED. |
| **G-2** | The stage-1a claim that a non-zero internal-key slot is "canonically unreachable" is **FALSE** — a real card `md1yp802gggqpsfx2q26nd0c9nkv89j4` carrying `Slot(1)` was minted and verified byte-identical. | Do not reason from it. `wire_version()` must not assume `Slot(0)`. |
| **G-3** | The version bump is **0.46.0, not 0.45.2**, and the CHANGELOG must attribute the break to **stage 1a** (`Body::Tr`'s shape), not to this stage's wire work. No md-codec release may be tagged until this lands. | Task 10. |
| **G-4** | `encode_md1_chunks` does not exist. `chunk.rs:240 split(&Descriptor) -> Result<Vec<String>, Error>` is the producer. | Task 8. |
| **G-5** | A kind-1 `Descriptor` cannot be built from TLV bytes alone — no taptree, no `older`, no fingerprints, no divergent origins. **Build it by decoding a vendored kind-0 vector and swapping `internal_key`**, which carries all four for free. | Task 1 defines `kind1_from_vector`. |
| **G-6** | `encode_payload_unchecked` does not exist and `Admission` is `pub(crate)`, so "a refused shape still decodes" must be a **unit** test inside the crate. | Task 7. |
| **G-7** | `cmd/vectors.rs:193` is a **third** production caller of the network-less `to_miniscript_descriptor*`, beyond `cmd/descriptor.rs` and `derive.rs:134`. | Task 5. |
| **G-8** | `md decompose` currently makes the unspendable xpub a **phantom slot `@0`**, which breaks the coordinator-compat conformance gate. A non-matching origin-less key must keep **today's** annotated-slot behaviour — the phantom property is *no origin*, not *not Liana's*. | Task 6. |

---

## File Structure

| file | responsibility |
| --- | --- |
| `crates/md-codec/tests/fixtures/liana/cases.json` | **new** — the vendored evidence. **NOT** under `tests/vectors/`, which is a generated corpus guarded by a `diff -r` drift test (`md-cli/tests/vector_corpus.rs:22,26`) |
| `scripts/vendor-liana-evidence.sh` | **new** — the committed generator for that fixture |
| `crates/md-codec/src/header.rs` | `WF_UNSPENDABLE_VERSION = 8`, `is_supported_version` |
| `crates/md-codec/src/tree.rs` | the kind bit; `read_node`/`write_node` take a wire version |
| `crates/md-codec/src/lib.rs` | `pub mod nums;` (it is private today, `:30`) |
| `crates/md-codec/src/nums.rs` | `liana_unspendable_xpub`, `LIANA_UNSPENDABLE_MARKER` |
| `crates/md-codec/src/{encode,decode,chunk,identity}.rs` | thread the derived version |
| `crates/md-codec/src/render.rs` | the marker in both keyless modes |
| `crates/md-codec/src/to_miniscript.rs` | the derived xpub; two `_with_network` entry points |
| `crates/md-codec/src/validate.rs` | §6's refusals, encode-side |
| `crates/md-cli/src/decompose/{mod,walk}.rs` | the recogniser |
| `crates/md-cli/src/parse/template.rs` | the marker's substitution rule; `md encode`'s refusal |
| `crates/md-cli/src/format/json.rs` | the v1 schema's third state |

---

## Task 1: vendor the evidence and the kind-1 fixture builder

**Files:** create `scripts/vendor-liana-evidence.sh`, `crates/md-codec/tests/fixtures/liana/cases.json`, `crates/md-codec/tests/common/liana.rs`.

**Interfaces produced:** `fn case(name: &str) -> Case` and `fn kind1_from_vector(vector_name: &str) -> Descriptor`, both in `tests/common/liana.rs`, pulled in with `include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/common/liana.rs"))` — examples and tests are separate crate roots and `use` across them does not compile. Give any `tests/`-root consumer its own `//!` header or it hits a missing-docs warning, fatal under `-D warnings`.

- [ ] **Step 1: Vendor with a committed script**

`scripts/vendor-liana-evidence.sh <path-to-mnemonic-engrave>` reads
`design/evidence/composer-fable-r0/fable-liana-parse-{in,out-v15}.jsonl`,
extracts the eight `liana-unspendable-xpub` records and their `md`
counterparts, and writes one object per case: `name`, `accepted` (bool),
`leaf_tlv_hex` (the **65-byte** `chain code ‖ compressed pubkey` entries in wire
order), `leaf_pubkeys_hex` (the 33-byte slices at `[32..65]` — what §2 hashes),
`expected_xpub`, `descriptor_with_checksum`, `liana_receive[3]`,
`liana_change[3]`. Record the source commit SHA in the output.

Four are `accepted: true` — `preset-kofn-recovery-tr`, `preset-tiered-recovery-tr`,
`same-seed-two-paths-tr`, `X19-tr-kofn-nums-older5`. The other four are refused
by Liana for policy-shape reasons and are still needed: §2's recipe is correct
for them too, and one is the only nested-taptree case.

- [ ] **Step 2: Define `kind1_from_vector`**

```rust
/// A real kind-1 Descriptor, built by decoding a vendored kind-0 vector and
/// swapping only the internal key. G-5: a Descriptor cannot be built from
/// cases.json's TLV bytes alone — it would have no taptree, no older(), no
/// fingerprints and no divergent origins, all of which a byte-identical
/// descriptor gate needs.
fn kind1_from_vector(vector_name: &str) -> Descriptor {
    let mut d = decode_vendored(&load_vendored_phrase(vector_name))
        .unwrap_or_else(|e| panic!("{vector_name}: decode: {e}"));
    match &mut d.tree.body {
        Body::Tr { internal_key, .. } => {
            assert_eq!(*internal_key, InternalKey::NumsPoint,
                       "{vector_name}: fixture must start at kind 0");
            *internal_key = InternalKey::LianaUnspendable;
        }
        _ => panic!("{vector_name} is not a tr descriptor"),
    }
    d
}
```

`load_vendored_phrase` and `decode_vendored` already exist in
`tests/common/vendored.rs` from stage 1a — include that file, do not redefine them.

- [ ] **Step 2b: Define the rest of the fixture surface, so no task calls an undefined helper**

```rust
#[derive(serde::Deserialize, Clone)]
struct Case {
    name: String,
    accepted: bool,
    leaf_tlv_hex: Vec<String>,        // 65-byte chain code || compressed pubkey
    leaf_pubkeys_hex: Vec<String>,    // the 33-byte slices at [32..65]
    expected_xpub: String,
    descriptor_with_checksum: String,
    liana_receive: Vec<String>,
    liana_change: Vec<String>,
}

impl Case {
    /// The 33-byte compressed pubkeys §2 hashes, in wire order.
    fn leaf_pubkeys(&self) -> Vec<[u8; 33]> {
        self.leaf_pubkeys_hex.iter()
            .map(|h| <[u8; 33]>::try_from(hex::decode(h).expect("hex").as_slice()).expect("33 bytes"))
            .collect()
    }
}

/// Every vendored case — all EIGHT, including the four Liana refused on
/// policy shape: §2's recipe is correct for those too, and one of them is the
/// only nested taptree in the evidence.
fn all_cases() -> Vec<Case> {
    serde_json::from_str(include_str!("../fixtures/liana/cases.json")).expect("cases.json")
}

fn case(name: &str) -> Case {
    all_cases().into_iter().find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no vendored case {name}"))
}

/// The vendored vector names whose decoded tree is a root `tr` at kind 0.
/// Stage 1a measured 23 of the 65 vectors as carrying `Body::Tr`.
fn all_kind0_tr_vectors() -> Vec<String> {
    all_vendored_vector_names().into_iter()
        .filter(|n| matches!(decode_vendored(&load_vendored_phrase(n))
                                 .map(|d| d.tree.body),
                             Ok(Body::Tr { .. })))
        .collect()
}
```

`all_vendored_vector_names` already exists in stage 1a's
`tests/common/vendored.rs`. **Leave its `#[allow(dead_code)]` alone** — the
attribute is there because the file is `include!`d into several crate roots and
not every consumer calls every helper, so removing it turns `phase-gate.sh` red
in the roots that do not. (An earlier draft of this plan said to remove it; that
premise was wrong.)

- [ ] **Step 3: Commit**

```bash
git add scripts/vendor-liana-evidence.sh crates/md-codec/tests/fixtures crates/md-codec/tests/common/liana.rs
git commit -m "test: vendor the Liana evidence and the kind-1 fixture builder"
```

---

## Task 2: wire version 8 — the header and the derived version

**Files:** `header.rs:26-47`, `tree.rs` (both node fns), `encode.rs:174`/`:181`, `decode.rs:89`, `chunk.rs:70`/`:279`/`:373`, `identity.rs:90`/`:200`. Test: `tests/wire_version_8.rs` (create).

**Interfaces produced:** `Header::WF_UNSPENDABLE_VERSION: u8 = 8`; `Header::is_supported_version(u8) -> bool`; `Descriptor::wire_version(&self) -> u8`; `write_node(w, node, kiw, wire_version)` and `read_node(r, kiw, wire_version)`.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn version_8_is_even_so_the_dispatch_routes_it_as_single_payload() {
    // decode.rs:191-193 reads bit 0 of the FIRST SYMBOL as the chunked flag.
    // For a single payload that bit IS v0, so every usable version must be
    // EVEN. Version 5 would route a single-string plate into the chunk
    // reassembler, which then reports WireVersionMismatch{got:2}.
    for divergent in [false, true] {
        let sym = (u16::from(divergent) << 4) | u16::from(Header::WF_UNSPENDABLE_VERSION);
        assert_eq!(((sym << 3) as u8) >> 3 & 1, 0,
                   "version {} dispatches as CHUNKED", Header::WF_UNSPENDABLE_VERSION);
    }
}

#[test]
fn the_decoder_accepts_4_and_8_and_refuses_everything_else() {
    for v in 0u8..16 { assert_eq!(Header::is_supported_version(v), matches!(v, 4 | 8), "version {v}"); }
}

#[test]
fn wire_version_is_derived_from_the_tree_not_assumed() {
    // G-2: do NOT assume Slot(0). A non-zero slot is constructible.
    assert_eq!(kind1_from_vector("keyed_compose_tr_nums_three_leaves").wire_version(), 8);
    for name in all_kind0_tr_vectors() { assert_eq!(decode_vendored(&load_vendored_phrase(name)).unwrap().wire_version(), 4, "{name}"); }
}
```

- [ ] **Step 2: Run to verify they fail**

`cargo nextest run --locked -p md-codec wire_version_8` → FAIL, the items do not exist.

- [ ] **Step 3: Implement the header**

```rust
    /// The version carrying a non-`NumsPoint` taproot internal key (SPEC §3c).
    /// Usable WF-redesign versions are {4, 8, 12} because the single-payload
    /// auto-dispatch reads v0 as the chunked flag, so every usable version must
    /// be EVEN. This spends 8 and leaves 12 as the format's last generation.
    pub const WF_UNSPENDABLE_VERSION: u8 = 8;

    /// Versions this build decodes.
    pub fn is_supported_version(v: u8) -> bool {
        v == Self::WF_REDESIGN_VERSION || v == Self::WF_UNSPENDABLE_VERSION
    }
```

and in `Header::read`, replace the exact-match with `if !Self::is_supported_version(version) { return Err(Error::WireVersionMismatch { got: version }); }`.

**§6a's stage-1b row lands here too:** `error.rs:33`'s Display says `"...expected 4"`, which becomes false the moment `{4, 8}` is accepted. Make it name the accepted set, and assert it:

```rust
#[test]
fn the_mismatch_message_names_the_accepted_set_not_a_single_version() {
    let s = Error::WireVersionMismatch { got: 9 }.to_string();
    assert!(s.contains('9'), "must name what it got: {s}");
    assert!(!s.contains("expected 4"), "stale single-version claim: {s}");
    assert!(s.contains('4') && s.contains('8'), "must name {{4, 8}}: {s}");
}
```

- [ ] **Step 4: Add `Descriptor::wire_version` and thread it**

```rust
impl Descriptor {
    /// The minimum wire version that can express this tree (SPEC §3d).
    /// NEVER a constant: identity.rs's two hash sites call write_node
    /// directly, and a constant v4 would make kind 0 and kind 1 hash
    /// identically while their addresses differ.
    pub fn wire_version(&self) -> u8 {
        fn needs_v8(n: &Node) -> bool {
            match &n.body {
                Body::Tr { internal_key, tree } =>
                    *internal_key == InternalKey::LianaUnspendable
                        || tree.as_deref().is_some_and(needs_v8),
                Body::Children(cs) | Body::Variable { children: cs, .. } => cs.iter().any(needs_v8),
                _ => false,
            }
        }
        if needs_v8(&self.tree) { Header::WF_UNSPENDABLE_VERSION } else { Header::WF_REDESIGN_VERSION }
    }
}
```

Then add `wire_version: u8` to `write_node`/`read_node` and pass at each caller: `encode.rs:181` and `:174` → `d.wire_version()`; **`identity.rs:90` and `:200` → `d.wire_version()` (the funds-relevant pair)**; `decode.rs:89` → the version `Header::read` returned; `chunk.rs:279` (writer) → the payload's version, so chunk headers agree with their own payload; `chunk.rs:70`/`:373` accept `{4, 8}`.

- [ ] **Step 5: Gate and commit**

```bash
export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH
./scripts/phase-gate.sh && ( cd fuzz && cargo check --all-targets )
git commit -am "feat(wire): accept version 8 and derive the version from the tree"
```

Wire bytes are still unchanged here — nothing emits 8 yet. **Stage 1a's byte-equality gate must still pass**; if it fails at this step, the version leaked into v4.

---

## Task 3: the kind bit, written only at version 8

**Files:** `tree.rs` (both Tr arms). Test: extend `tests/wire_version_8.rs`.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn kind_0_and_kind_1_over_the_same_tree_encode_to_DIFFERENT_bytes() {
    let k0 = decode_vendored(&load_vendored_phrase("keyed_compose_tr_nums_three_leaves")).unwrap();
    let k1 = kind1_from_vector("keyed_compose_tr_nums_three_leaves");
    assert_ne!(encode_payload(&k0).unwrap().0, encode_payload(&k1).unwrap().0);
}

#[test]
fn a_kind_1_tree_round_trips_at_v8() {
    let d = kind1_from_vector("keyed_compose_tr_nums_three_leaves");
    let (bytes, bits) = encode_payload(&d).unwrap();
    assert_eq!(decode_payload(&bytes, bits).unwrap(), d);
}
```

- [ ] **Step 2: Run to verify they fail** — both kinds currently encode identically (stage 1a made them so deliberately).

- [ ] **Step 3: Implement**

Write side:
```rust
    InternalKey::NumsPoint | InternalKey::LianaUnspendable => {
        w.write_bits(1, 1);
        // SPEC §3d: the kind bit exists ONLY at version 8. At version 4 the
        // stream is byte-identical to what shipped.
        if wire_version == Header::WF_UNSPENDABLE_VERSION {
            w.write_bits(u64::from(*internal_key == InternalKey::LianaUnspendable), 1);
        }
    }
```
Read side mirrors it: at v8 read one bit and pick the variant; at v4 it is `NumsPoint`.

- [ ] **Step 4: Gate** — the full gate, **and stage 1a's byte-equality test must still pass**. If it fails here, the kind bit leaked into version 4.

- [ ] **Step 5: Commit**

---

## Task 4: §2's derivation

**Files:** `lib.rs:30` (`mod nums;` → `pub mod nums;` — it is private today and the tests are integration tests), `nums.rs`. Test: `tests/liana_unspendable.rs` (create, with a `//!` header).

**Interface produced:** `pub fn liana_unspendable_xpub(leaf_pubkeys: &[[u8; 33]], network: bitcoin::Network) -> bitcoin::bip32::Xpub`, and `pub const LIANA_UNSPENDABLE_MARKER: &str = "UNSPENDABLE(liana)";`.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn the_recipe_reproduces_every_golden_xpub() {
    for c in all_cases() {                       // all EIGHT, not just the four accepted
        let leaves: Vec<[u8; 33]> = c.leaf_pubkeys();
        assert_eq!(liana_unspendable_xpub(&leaves, Network::Bitcoin).to_string(),
                   c.expected_xpub, "{}", c.name);
    }
}

#[test]
fn sorting_or_deduplicating_produces_a_DIFFERENT_xpub() {
    // Liana's recipe is NOT sorted and NOT deduplicated. The abandoned BIPs
    // PR #1746 recipe is, and it is a different wallet.
    let c = case("preset-kofn-recovery-tr");
    let mut l = c.leaf_pubkeys();
    let straight = liana_unspendable_xpub(&l, Network::Bitcoin).to_string();
    l.sort(); l.dedup();
    assert_ne!(straight, liana_unspendable_xpub(&l, Network::Bitcoin).to_string());
}

#[test]
fn the_chain_code_depends_on_the_KEYS_not_the_TREE() {
    // kofn-recovery, tiered-recovery and decaying-multisig are three different
    // trees over the same four keys in the same order and share one internal
    // key. decaying-multisig is the only NESTED taptree in the evidence, so it
    // is what proves the traversal is depth-first left-to-right.
    let names = ["preset-kofn-recovery-tr", "preset-tiered-recovery-tr", "preset-decaying-multisig-tr"];
    let xs: Vec<String> = names.iter().map(|n| case(n).expected_xpub.clone()).collect();
    assert_eq!(xs[0], xs[1]); assert_eq!(xs[1], xs[2]);
}

#[test]
fn a_tpub_wallet_derives_a_tpub_internal_key() {
    // §2 step 5's testnet branch is TRANSCRIBED, not measured — all eight
    // evidence descriptors are mainnet. This is the vector that measures it.
    assert!(liana_unspendable_xpub(&case("preset-kofn-recovery-tr").leaf_pubkeys(),
                                   Network::Testnet).to_string().starts_with("tpub"));
}
```

- [ ] **Step 2: Run to verify they fail.**

- [ ] **Step 3: Implement — and the hashed vector is per KEY OCCURRENCE, not per slot**

Earlier drafts described the input two non-equivalent ways: once as
`cases.json`'s `leaf_tlv_hex` (one entry **per slot**) and once as "each leaf's
pubkey in tap-tree order" (one per **occurrence**). They differ only if a slot
can appear more than once in the taptree.

**Measured: it cannot, in any md1-expressible descriptor.** Both reuse forms are
refused, for two different and deliberate reasons:

```
$ md encode "tr(<NUMS>,{pk(@0/<0;1>/*),and_v(v:pk(@0/<0;1>/*),older(26280))})"
md: unsupported: @0 appears at 2 use sites ... with the same path expression ...
    forbidden by BIP 388's disjointness rule

$ md encode "tr(<NUMS>,{pk(@0/<0;1>/*),and_v(v:pk(@0/<2;3>/*),older(26280))})"
md: unsupported: @0 appears at use sites with DISJOINT multipath sets ...
    The WALLET is legal under BIP 388 ... md1 deliberately cannot express it,
    because an md1 card carries ONE path per key slot (F-417)
```

So the two readings coincide for every descriptor **md-cli** can mint.

**But that is not a licence to implement either one, and an earlier draft wrongly
gave one.** The code lands in **md-codec**, which is more permissive than the
CLI: `validate_placeholder_usage` explicitly allows a slot *"at least once"*, and
the `Admission::Enforce` block adds no reuse check. A `Descriptor` constructed
in-crate — which every test fixture here is — can therefore carry a repeated
slot that `md encode` would have refused.

**§2 is normatively per KEY OCCURRENCE, walked over the tap tree.** Implement
that, not the slot-indexed shortcut. The distinction is load-bearing at the one
production call site: `node_to_descriptor` holds `keys: &[DescriptorPublicKey]`
indexed **by slot**, so the natural implementation there is the **wrong** one and
needs a tree walk instead:

```rust
// RIGHT: walk the tap tree, collect each key occurrence in order.
let mut pks = Vec::new();
for leaf in tap_tree.leaves() {
    // miniscript 13.0.0 has no TapTree::iter(); it is `leaves()`, yielding
    // TapTreeIterItem. The repo already spells it this way at
    // md-cli/src/parse/template.rs:1671 and :2684.
    leaf.miniscript().for_each_key(|k| { pks.push(compressed_33(k)); true });
}

// WRONG: keys.iter() is slot order. It coincides only for CLI-minted shapes,
// and md-codec admits others.
```

The refusal pin below keeps the *CLI* path unambiguous; it does not make the
slot-indexed reading correct inside the codec.

**Do not write a test asserting the distinction** — it cannot be constructed,
and a test whose fixture cannot exist is the defect this cycle has already paid
for twice. Pin the *reason* instead, so that if md1 ever widens (F-417 is a
decision, not a law of nature) this fails loudly rather than silently changing
every kind-1 chain code:

**This pin lives in `crates/md-cli/tests/`, not md-codec's.** It shells out to
the `md` binary, and md-codec's `[dev-dependencies]` are `serde`, `serde_json`,
`hex`, `proptest`, `miniscript` — no CLI runner, and `CARGO_BIN_EXE_md` is not
defined outside md-cli's own targets, so `md_err` cannot resolve there. Task 8
Step 5 relocates an identical pin for the same reason.

```rust
const NUMS_HEX: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

#[test]
fn a_slot_cannot_appear_twice_in_a_taptree_so_occurrence_order_IS_slot_order() {
    // §2's hashed sequence is unambiguous only because md1 refuses both reuse
    // forms. If either refusal is relaxed, the recipe must be re-specified as
    // per-OCCURRENCE before that lands -- Liana walks leaves, not slots.
    //
    // TWO THINGS MAKE THIS PIN ABLE TO FAIL, and an earlier draft had neither:
    //
    // 1. The internal key is the REAL NUMS HEX, not a `<NUMS>` placeholder.
    //    Measured: `tr(<NUMS>,{pk(@0/<0;1>/*),pk(@1/<0;1>/*)})` fails with
    //    "miniscript parse failed: key too short". Today the reuse check runs
    //    FIRST, so the placeholder spelling still produced the right error --
    //    but the moment either refusal is REMOVED (the one change this test
    //    exists to catch) the template would fall through to that parse error
    //    and a bare is_err() would keep passing. The pin would survive the
    //    deletion of the thing it pins.
    // 2. Each case asserts its OWN error, not `is_err()`.
    let same_path = format!(
        "tr({NUMS_HEX},{{pk(@0/<0;1>/*),and_v(v:pk(@0/<0;1>/*),older(26280))}})");
    let disjoint = format!(
        "tr({NUMS_HEX},{{pk(@0/<0;1>/*),and_v(v:pk(@0/<2;3>/*),older(26280))}})");

    let e = md_err(&["encode", &same_path, "--path", "bip48"]);
    assert!(e.contains("same path expression") && e.contains("BIP 388"),
            "same-path reuse no longer refused for its own reason: {e}");

    let e = md_err(&["encode", &disjoint, "--path", "bip48"]);
    assert!(e.contains("DISJOINT multipath sets") && e.contains("ONE path per key slot"),
            "disjoint-path reuse no longer refused for its own reason: {e}");
}
```

Then implement: `sha256` over that sequence; pubkey = the BIP-341 NUMS point
compressed; depth 0, parent 0, child 0; network from the parameter.
**`hex_lit` is not a dependency** — use a byte array or the `bitcoin` crate
already in the graph.

- [ ] **Step 4: Gate and commit.**

---

## Task 5: rendering — TWO files, and the four gating sites

**Files:** `render.rs:192`, `to_miniscript.rs:341`, `policy_shape.rs:251`, `md-cli/src/format/json.rs:356`, `lib.rs` (new entry points), `cmd/descriptor.rs`, `derive.rs:134`, `cmd/vectors.rs:193`.

**This task discharges G-1.** Those four sites are `NumsPoint | LianaUnspendable` or-patterns, so **the compiler will not tell you if you miss one** — splitting them proved the suite unchanged at 1439/3. Visit all four by name. A missed site yields a NUMS taproot output key: a **wrong address**.

**Interfaces produced:** `to_miniscript_descriptor_with_network(&Descriptor, u32, Network)` and `to_miniscript_descriptor_multipath_with_network(&Descriptor, Network)`.

- [ ] **Step 1: Write the failing tests** — the derived literal xpub when keyed; the marker in **both** keyless modes; a tpub under `Network::Testnet`; the network-less entry points **refusing** kind 1 rather than guessing mainnet; and `skeleton_key` differing between the kinds.

- [ ] **Step 2: Run to verify they fail.**

- [ ] **Step 3: Add the two `_with_network` entry points; do NOT change the existing signatures** — `to_miniscript_descriptor*` has **55 call sites**. The existing two delegate with mainnet and **refuse** a descriptor whose `wire_version() == 8`, so no caller can silently emit a mainnet xpub for a testnet wallet.

- [ ] **Step 4: Deal with ALL TWELVE production callers — G-7 undercounted by nine**

Measured (`grep -rn "to_miniscript_descriptor(\|to_miniscript_descriptor_multipath(" crates/*/src`):

| caller | has a network? | action |
| --- | --- | --- |
| `md-cli/src/cmd/descriptor.rs:200`, `:201` | yes, `args.network` (`:62`) | switch to `_with_network` |
| `md-codec/src/derive.rs:134` | yes, `derive_address`'s own param (used at `:155`) | switch — **miss this and every kind-1 address derivation returns `Err`, disabling Task 8's address gate** |
| `md-cli/src/cmd/vectors.rs:193` | yes | switch |
| `md-cli/src/seat/matching.rs:304`, `:335`, `:454` | **no** | leave on the refusing entry point |
| `md-cli/src/seat/disposition.rs:308`, `:434`, `:437` | **no** | leave |
| `md-cli/src/seat/compose.rs:403`, `:406` | **no** | leave |

The nine `seat/*` sites have no network in scope and none of them should
acquire one in this stage — they compare and match seatings, they do not render
for an operator. Leaving them on the refusing entry point is the correct
outcome: a kind-1 descriptor reaching seat-matching should fail loudly rather
than be compared as a mainnet-rendered string.

**Add a test asserting exactly that**, so the refusal is a gate and not an
accident:

```rust
#[test]
fn a_kind_1_descriptor_is_REFUSED_by_the_network_less_entry_point() {
    let d = kind1_from_vector("keyed_compose_tr_nums_three_leaves");
    assert!(matches!(md_codec::to_miniscript_descriptor_multipath(&d),
                     Err(Error::NetworkRequiredForUnspendable)));
}
```

- [ ] **Step 5: Implement the four sites**, each named above. `render.rs` emits the marker in both `Mode::Literal` and `Mode::Abstract`; `to_miniscript.rs` builds the derived xpub from the already-built leaf miniscripts in tap-tree order; `policy_shape.rs` maps to the new `KeyPathKind`; `json.rs` gains the schema's third state.

- [ ] **Step 6: Gate and commit.**

---

## Task 6: the input side (SPEC §4a)

**Files:** `md-cli/src/decompose/mod.rs:439`, `decompose/walk.rs` (`collect_occurrences` `:175`, `Placeholders::pk` `:202`), `parse/template.rs` (`substitute_synthetic` `:1047-1084`), `format/json.rs`, `docs/json-schema-v1.md`.

**Three surfaces, three different rules** — they are not the same:

| surface | can it recompute §2? | rule |
| --- | --- | --- |
| `md decompose` | **yes**, it holds the real leaf keys | recompute; on a byte match, kind 1 and **no slot** |
| the template grammar | n/a | **both**: substitute the marker to a recognisable synthetic key in `substitute_synthetic` (`:1047-1084`) so `Descriptor::from_str` succeeds, **then map that key back to the kind in `walk_tr`** |
| `md encode` with a literal xpub | **no** | refuse, naming both working spellings |

**Where the kind is actually decided.** `walk_tr` (`parse/template.rs:1593-1605`)
is the *only* place that decides the internal-key kind today:

```rust
let key_str = t.internal_key().to_string();
…
if key_str == NUMS_H_POINT_X_ONLY_HEX {
    … InternalKey::NumsPoint …
```

An earlier draft of this plan said the rule belonged in `substitute_synthetic`
**and not** `walk_tr`. That was backwards. Both are needed and they do different
jobs: `substitute_synthetic` makes the token survive `Descriptor::from_str`
(which rejects `UNSPENDABLE(liana)` as a key expression), and `walk_tr` is where
the resulting synthetic key is recognised and turned into
`InternalKey::LianaUnspendable` — beside the existing NUMS comparison.

`md encode` still **cannot recompute** §2 from a literal xpub: `substitute_synthetic`
replaces every `@i` with a `sha256(b"md-v0.15" ‖ i ‖ depth)` placeholder before
`walk_tr` sees the tree, so the leaves are synthetic and a recipe match can never
succeed. Today it leaks `"internal: synthetic key … not found in key map"` —
replace that with a refusal naming the two working spellings.

**G-8:** a non-matching **origin-less** internal key keeps **today's** behaviour — the annotated `@0` slot, with `--emit commands` refusing. The phantom property is *no origin*, not *not Liana's*. Refusing it would lock out libnunchuk's PR-1746 form and real origin-less spendable keys.

**The JSON schema change is a published v1 break** — version it in `docs/json-schema-v1.md`, do not slip it in.

**Test file for this task: `crates/md-cli/tests/liana_input_side.rs`** (new).
Every test in Task 6 lives there — it shells out to `md`, so md-codec's test
root cannot host it.

- [ ] **Step 0: md-cli's tests need their OWN fixture access — `case` is md-codec's**

These tests shell out to `md`, so they live in `crates/md-cli/tests/`. `case`
and `kind1_from_vector` are `include!`d into **md-codec's** test root via
`env!("CARGO_MANIFEST_DIR")`; from md-cli that macro resolves to md-cli's own
directory and the file is not there. This is the third time this class has bitten
(r5/I-1 on `kind1_from_vector`, r4/I-2 on the same, now `case`), so md-cli gets
its own loader over the **same committed JSON** — one fixture, two readers:

```rust
// crates/md-cli/tests/liana_cases.rs — md-cli's reader for md-codec's fixture.
// The JSON is the shared artifact; the loader is per-crate, because include!
// and CARGO_MANIFEST_DIR are per-crate.
#[derive(serde::Deserialize, Clone)]
struct Case { name: String, accepted: bool, descriptor_with_checksum: String,
              expected_xpub: String, liana_receive: Vec<String>, liana_change: Vec<String> }

fn case(name: &str) -> Case {
    let raw = include_str!("../../md-codec/tests/fixtures/liana/cases.json");
    serde_json::from_str::<Vec<Case>>(raw).expect("cases.json")
        .into_iter().find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no vendored case {name}"))
}

/// A real `tr` whose internal key is a SPENDABLE xpub with NO recorded origin.
/// Not Liana's recipe, not NUMS — the case G-8 says must keep today's
/// annotated-slot behaviour. Taken from the evidence's `X23` row, whose
/// internal key is a plain xpub.
const ORIGINLESS_SPENDABLE_TR: &str = concat!(
    "tr(xpub6DXuQW1Q2JpZyweiMewTZuMPvjG8hKhV2qoF6wL9VFxsMBExtbfqAAoR4oMG4GyxFzVdfas1v2eAdfLxyjc4Ceo5B6w6zT/<0;1>/*,",
    "and_v(v:pk([73c5da0a/48'/0'/1'/3']xpub6DXuQW1Q2JpZzLV9kZbjmB9NcnQ7UmM8ZMDkM5yCkXqUFEJrJvVXmxfFrnBGJmFWVSYJVPNrVbTQQhZ8ryFHqPzWhBXAFsFCTGgCkeS/<0;1>/*),older(26280)))"
);
```

**The exact xpubs above are TRUNCATED and will not parse.** Step 0's first action is to take them
from the vendored `cases.json` rather than transcribe them — a hand-copied xpub
is a defect this cycle has already paid for.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn decompose_RECOGNISES_a_real_liana_descriptor_and_gives_it_no_slot() {
    let (out, err, code) = md(&["decompose",
        &case("preset-kofn-recovery-tr").descriptor_with_checksum, "--emit", "template"]);
    assert_eq!(code, 0, "decompose failed: {err}");
    assert!(out.contains("UNSPENDABLE(liana)"), "got {out}");
    assert!(!out.contains("state NO origin"), "must not become a phantom slot");
}

#[test]
fn decompose_keeps_TODAYS_behaviour_for_an_origin_less_key_that_is_NOT_lianas() {
    // G-8: the phantom property is NO ORIGIN, not "not Liana's". A real
    // spendable internal key whose owner recorded no origin is @0 and correct;
    // refusing it would also lock out libnunchuk's PR-1746 form.
    let (out, err, code) = md(&["decompose", ORIGINLESS_SPENDABLE_TR, "--emit", "template"]);
    assert_eq!(code, 0, "decompose failed: {err}");
    assert!(out.contains("tr(@0/"), "must still be a slot: {out}");
    assert!(out.contains("state NO origin"), "must still annotate");
}

#[test]
fn md_encode_refuses_a_literal_xpub_in_the_internal_key_position_CLEANLY() {
    let err = md_err(&["encode", &case("preset-kofn-recovery-tr").descriptor_with_checksum]);
    assert!(!err.contains("internal:"), "internal invariant leaked: {err}");
    assert!(err.contains("UNSPENDABLE(liana)"), "must name the working spelling");
    assert!(err.contains("decompose"), "must name the other working route");
}

#[test]
fn the_marker_PARSES_in_a_template_and_yields_kind_1() {
    let (md1, err, code) = md(&["encode",
        "tr(UNSPENDABLE(liana),{pk(@0/<0;1>/*),pk(@1/<0;1>/*)})", "--path", "bip48"]);
    assert_eq!(code, 0, "encode failed: {err}");
    let d = md_codec::decode::decode_md1_string(md1.trim()).expect("decode");
    assert!(matches!(d.tree.body, Body::Tr { internal_key: InternalKey::LianaUnspendable, .. }));
}
```

- [ ] **Step 2: Run to verify they fail** — today `md encode` leaks `"internal: synthetic key … not found in key map"` and `decompose` makes a phantom `@0`.

- [ ] **Step 3: The template grammar — BOTH halves**

`substitute_synthetic` (`:1047-1084`) rewrites `UNSPENDABLE(liana)` to a reserved synthetic key so `Descriptor::from_str` accepts it; `walk_tr` (`:1593-1605`) recognises that key beside its existing `NUMS_H_POINT_X_ONLY_HEX` comparison and yields `InternalKey::LianaUnspendable`.

- [ ] **Step 4: The `md decompose` recogniser**

In `decompose/walk.rs`, at the `Tr` node (NOT downstream of `for_each_key`, which yields keys with no structural position): separate the internal key from the leaves, recompute §2 over the leaf keys, and on a byte match emit the marker with **no slot**. On a mismatch, fall through to today's annotated-slot path unchanged.

- [ ] **Step 5: `md encode`'s refusal** — replace the internal-error leak with a message naming both working spellings.

- [ ] **Step 6: The JSON schema's third state** — `format/json.rs`'s `Tr` variant, plus the version bump and entry in `docs/json-schema-v1.md`.

- [ ] **Step 7: Gate and commit.**

---

## Task 7: §6's refusals — ENCODE-SIDE ONLY, and WIRED

**Files:** `validate.rs` (the four checks), `error.rs` (their variants), **`encode.rs:143-170`** (the call site — earlier drafts omitted this file entirely, so the refusals existed and ran nowhere).

**Interfaces produced:** `validate_unspendable_shape(&Descriptor) -> Result<(), Error>`, called from `encode_payload_inner` behind `Admission::Enforce`.

- [ ] **Step 1: Write the failing tests — as UNIT tests inside `encode.rs`**

**Two homes, decided by what each test needs — an earlier draft put all three in `encode.rs` and then had them call `kind1_from_vector`, which lives in `tests/common/liana.rs` and is unreachable from inside the crate.**

| test | needs | home |
| --- | --- | --- |
| the two refusal tests | only `encode_payload`, which is **public** (`encode.rs:99`) | **integration**, `tests/liana_unspendable.rs` — so they may use `kind1_from_vector` |
| `a_refused_shape_still_DECODES` | `encode_payload_inner` (module-private, `:143`) and `Admission::SkipPolicy` (`pub(crate)`, `:107`) | **unit**, inside `encode.rs`'s own `#[cfg(test)] mod tests` — and it must build its fixture **in-crate**, with no `tests/` helper |

```rust
#[test]
fn kind_1_with_a_sortedmulti_a_leaf_is_refused_at_MINT() {
    let d = tr_liana_with_sortedmulti_a_leaf();
    assert!(matches!(encode_payload(&d), Err(Error::UnspendableWithSortedMultiA)));
}

/// MEASURED AGAINST `37367c1f`, not reasoned about: this exact body encodes to
/// **88 bits** under `Admission::SkipPolicy` and `decode_payload` returns
/// **Ok**. (Round-trip EQUALITY does not hold until Task 3 adds the kind bit —
/// `LianaUnspendable` has no wire representation before then and decodes back
/// as `NumsPoint`. The test below asserts `is_ok()`, which is the property it
/// needs; do not strengthen it to `assert_eq!` before Task 3.)
///
/// The in-crate fixture for the test below. It must do TWO things at once:
/// trip §6's `sortedmulti_a` refusal under `Admission::Enforce`, AND survive
/// `Admission::SkipPolicy` encode followed by `decode_payload`. A fixture that
/// only does the first makes the test vacuous.
#[cfg(test)]
fn in_crate_tr_liana_with_sortedmulti_a_leaf() -> Descriptor {
    // tests/common/ is a different crate root, so kind1_from_vector is not
    // reachable here -- build the tree directly.
    // Tag::SortedMultiA takes Body::MultiKeys, NOT Body::Variable — the
    // latter's doc says "Tag::Thresh ONLY; multi-family tags use MultiKeys"
    // (tree.rs:38-56). MEASURED: the Variable spelling encodes to 106 bits and
    // decode_payload returns TlvLengthExceedsRemaining{length:9,remaining:5},
    // so the test would be RED before §6 exists, blaming the decode path.
    let leaf = Node {
        tag: Tag::SortedMultiA,
        body: Body::MultiKeys { k: 2, indices: vec![0, 1, 2] },
    };
    let tree = Node {
        tag: Tag::Tr,
        body: Body::Tr { internal_key: InternalKey::LianaUnspendable, tree: Some(Box::new(leaf)) },
    };
    // Build the Descriptor inline. There is no `descriptor_with` helper and no
    // `Descriptor::default_for_tests` — an earlier draft invented both. The
    // in-crate pattern is a full struct literal; model it on
    // `wpkh_template_only` at encode.rs:284.
    Descriptor {
        n: 3,
        // NOT an empty origin path. `wpkh_template_only` can use one; a
        // `tr(@N, TapTree)` CANNOT -- it sits in canonical_origin's
        // forced-explicit column, and an empty decl gives
        // Err(MissingExplicitOrigin{idx:0}) at decode, which reads as a
        // decode-path bug rather than a fixture bug. MEASURED: empty ->
        // encode Ok 60 bits -> decode Err(MissingExplicitOrigin{idx:0}).
        // Shape copied from origin_path.rs:216-226.
        path_decl: PathDecl {
            n: 3,
            paths: PathDeclPaths::Shared(OriginPath {
                components: vec![
                    PathComponent { hardened: true, value: 48 },
                    PathComponent { hardened: true, value: 0 },
                    PathComponent { hardened: true, value: 0 },
                    PathComponent { hardened: true, value: 3 },
                ],
            }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection::new_empty(),
    }
}

// UNIT test, inside encode.rs.
#[test]
fn a_refused_shape_still_DECODES_so_existing_cards_never_stop_reading() {
    // encode.rs:116-131 records the measured 2026-09-19 regression: mint-side
    // refusals leaking into decode made a shipped 2-of-2 stop reading. This is
    // that regression as an assertion.
    let d = in_crate_tr_liana_with_sortedmulti_a_leaf();
    let (bytes, bits) = encode_payload_inner(&d, Admission::SkipPolicy).expect("mint-bypass encode");
    assert!(decode_payload(&bytes, bits).is_ok(), "a mint-side refusal reached decode");
}

#[test]
fn kind_1_off_the_canonical_use_site_is_refused() {
    assert!(matches!(encode_payload(&tr_liana_at_use_site(2, 3)),
                     Err(Error::UnspendableUseSiteNotCanonical)));
}
```

**The three fixtures these tests use, defined here:**

```rust
/// A kind-1 tr whose taptree contains a `sortedmulti_a` leaf — the shape §6
/// row 1 refuses. VERIFIED SHAPE: `keyed_tr_sortedmulti_a` is NOT this, it is
/// `tr(@0/...,sortedmulti_a(...))` with a SPENDABLE internal key, so
/// kind1_from_vector's assert would panic on it. The NUMS one is:
fn tr_liana_with_sortedmulti_a_leaf() -> Descriptor {
    kind1_from_vector("keyed_compose_tr_sole_sortedmulti_a")
}

/// A kind-1 tr whose slots sit at a non-canonical use site. md1 carries ONE
/// path per slot (F-417), so this is a whole-descriptor change, not per-leaf.
///
/// `UseSitePath` has NO `parse` and NO `FromStr` — its surface is
/// `standard_multipath()`, `write()`, `read()` (`use_site_path.rs:70-78`).
/// Build it by struct literal; the fields are public.
fn tr_liana_at_use_site(a: u32, b: u32) -> Descriptor {
    let mut d = kind1_from_vector("keyed_compose_tr_nums_three_leaves");
    d.use_site_path = UseSitePath {
        multipath: Some(vec![
            Alternative { hardened: false, value: a },
            Alternative { hardened: false, value: b },
        ]),
        wildcard_hardened: false,
    };
    d
}
```

- [ ] **Step 2: Run to verify they fail** — `cargo nextest run --locked -p md-codec encode::` → FAIL, the variants and the validator do not exist.

- [ ] **Step 2b: The other two refusals need tests and fixtures too**

Step 1 covers the `sortedmulti_a` leaf and the non-canonical use-site. §6 has
**four**, and Task 9 maps mutations 8-11 to this task — so without these two,
two of those mutations cannot turn RED:

```rust
#[test]
fn kind_1_nested_under_wsh_is_refused() {
    // The internal key of a tr nested inside wsh is not the descriptor's
    // internal key; kind 1 there has no meaning.
    assert!(matches!(encode_payload(&wsh_wrapping_tr_liana()),
                     Err(Error::UnspendableNotRootTr)));
}

#[test]
fn version_8_with_no_kind_1_node_is_refused_at_ENCODE() {
    // The minimum-version rule, enforced on the encode side only so old
    // payloads never become invalid at decode.
    assert!(matches!(encode_payload_at_forced_version(&all_nums_tr(), 8),
                     Err(Error::NonMinimalWireVersion { .. })));
}
```

Both fixtures are built in-crate from `Node`/`Body`; `encode_payload_at_forced_version`
is a `#[cfg(test)]` helper in `encode.rs` that calls `encode_payload_inner` with an
overridden version, since no public API lets a caller pick a non-minimal one.

- [ ] **Step 3: Implement the validator** — the four §6 refusals in `validate.rs`, their `Error` variants in `error.rs`.

- [ ] **Step 4: WIRE IT — the step earlier drafts omitted**

In `encode_payload_inner`, beside the existing `Admission::Enforce` block:

```rust
    if admission == Admission::Enforce {
        …
        crate::validate::validate_unspendable_shape(d)?;   // <- new
    }
```

**Not in `decode_payload_with_opts`.** A refusal there makes existing plates unreadable — that is the regression `encode.rs:116-131` documents.

- [ ] **Step 5: Gate and commit** — `./scripts/phase-gate.sh`, `( cd fuzz && cargo check --all-targets )`.

---

## Task 8: §8's acceptance vectors — codec-level only

**Files:** `crates/md-codec/tests/liana_unspendable.rs` (extend).

**SCOPE RULING (C3).** Earlier drafts required descriptor- and address-equality against the evidence's four ACCEPT shapes. **That cannot be done in this stage**: it needs an md-codec `Descriptor` carrying the evidence's exact keys, origins, fingerprints and locks, and the only route to one is `md decompose` / `md descriptor` — **md-cli, which is stage 2**. The spec already double-gates §8.2 (`1b` at the codec level, `2` through the CLI), so this is the intended split, not a gap.

- **In 1b:** every §8 property is proven over **md's own vendored corpus** via `kind1_from_vector`.
- **In 2:** the same properties against the **evidence** shapes, plus §8.8's live Liana run.

Record this in stage 2's brief so it is not lost.

- [ ] **Step 1: Round trip and version, over the corpus**

```rust
#[test]
fn every_tr_vector_round_trips_at_kind_1_and_reports_version_8() {
    for name in all_kind0_tr_vectors() {
        let d = kind1_from_vector(&name);
        assert_eq!(d.wire_version(), 8, "{name}");
        let (bytes, bits) = match encode_payload(&d) {
            Ok(v) => v,
            Err(Error::UnspendableWithSortedMultiA) => continue,   // §6 refuses these
            Err(e) => panic!("{name}: {e}"),
        };
        assert_eq!(decode_payload(&bytes, bits).unwrap(), d, "{name}");
    }
}
```

- [ ] **Step 2: Identity distinctness AND stability**

```rust
#[test]
fn kind_0_and_kind_1_get_different_ids_and_a_different_phrase() {
    let k0 = decode_vendored(&load_vendored_phrase("keyed_compose_tr_nums_three_leaves")).unwrap();
    let k1 = kind1_from_vector("keyed_compose_tr_nums_three_leaves");
    assert_ne!(compute_wallet_policy_id(&k0).unwrap(), compute_wallet_policy_id(&k1).unwrap());
    assert_ne!(compute_wallet_descriptor_template_id(&k0).unwrap(),
               compute_wallet_descriptor_template_id(&k1).unwrap());
    assert_ne!(compute_wallet_policy_id(&k0).unwrap().to_phrase().unwrap(),
               compute_wallet_policy_id(&k1).unwrap().to_phrase().unwrap());
}

#[test]
fn every_existing_v4_identity_is_byte_preserved() {
    // Stage 1a's committed golden, 65 4-tuples. Do NOT regenerate it.
    let g: IdGolden = serde_json::from_str(
        include_str!("golden/pre_refactor_ids.json")).unwrap();
    for (name, policy, template, phrase) in &g.vectors {
        let d = decode_vendored(&load_vendored_phrase(name)).unwrap();
        assert_eq!(&compute_wallet_policy_id(&d).unwrap().to_string(), policy, "{name}");
        assert_eq!(&compute_wallet_descriptor_template_id(&d).unwrap().to_string(), template, "{name}");
        assert_eq!(&compute_wallet_policy_id(&d).unwrap().to_phrase().unwrap().to_string(), phrase, "{name}");
    }
}
```

- [ ] **Step 3: The dispatch round trip (SPEC §8.4's Rust leg)**

Through `decode_with_correction`, single-string **and** chunked — not `decode_payload`, which bypasses the auto-dispatch that made version 5 unusable. Chunk sets come from `chunk::split(&d)` (G-4: there is no `encode_md1_chunks`).

- [ ] **Step 4: §8.1's nested-taptree vector (I5)**

`preset-decaying-multisig-tr` is the evidence's only nested taptree `{A,{B,C}}`, and Liana **refused** it on policy shape — so no ACCEPT backs the traversal order. Task 4's `the_chain_code_depends_on_the_KEYS_not_the_TREE` already pins it against the golden xpub, which is the strongest available evidence. **State in the test's comment that this ordering is pinned by recipe-agreement, not by a Liana ACCEPT**, so nobody later mistakes it for measured import.

**And record the non-delivery explicitly.** §8.1 asks for "a nested taptree that
Liana ACCEPTS". **No such evidence exists** — the only nested case in the corpus
is `preset-decaying-multisig-tr`, which Liana refused on policy shape, and no
accepted case is nested. So this stage delivers the strongest thing available
(recipe agreement against the golden xpub) and **does not** deliver the thing
§8.1 literally asks for. That is a deliberate gap, not an oversight: closing it
would need a new Liana-accepted nested shape measured through the harness, which
is **stage 2's** live run. Carry it into stage 2's brief.

- [ ] **Step 5: §6's pinned invariant (I6)**

SPEC §6 carries an invariant, not a refusal: a `tr` with no leaf keys cannot be constructed, so `sha256("")` can never become a shared chain code. It belongs in **md-cli**'s tests (the guard is the template parser's):

```rust
#[test]
fn a_tr_with_no_leaf_keys_cannot_be_constructed() {
    // If this ever SUCCEEDS, liana_unspendable_xpub would derive sha256("")
    // for every such wallet and they would all share one internal key.
    assert!(md_err(&["encode", "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0)"])
            .contains("no @i placeholders"));
}
```

- [ ] **Step 5b: §8 items 2 and 7, which the C3 ruling left without a step**

The C3 ruling moved item 2's **evidence** leg to stage 2 and promised a
corpus-level replacement; no step implemented it. And item 7 — the
render-reparse fixpoint — vanished from this task silently during that fold.
Both are §9-assigned to 1b. Restore them at the corpus level:

```rust
#[test]
fn item_2_descriptor_equality_at_the_CORPUS_level() {
    // The evidence shapes move to stage 2 (they need md-cli to build). Here the
    // property is that a kind-1 descriptor renders stably and re-renders
    // identically -- md vs md, not md vs Liana.
    for name in all_kind0_tr_vectors() {
        let d = kind1_from_vector(&name);
        if encode_payload(&d).is_err() { continue; }            // §6 refusals
        let a = to_miniscript_descriptor_multipath_with_network(&d, Network::Bitcoin).unwrap().to_string();
        let b = to_miniscript_descriptor_multipath_with_network(&d, Network::Bitcoin).unwrap().to_string();
        assert_eq!(a, b, "{name}");
        assert!(a.contains('#'), "{name}: the BIP-380 checksum is part of the gate");
    }
}

```

**The test below lives in `crates/md-cli/tests/`, not md-codec's** — it shells
out to the `md` binary, and md-codec has no CLI runner in dev-deps and no
`CARGO_BIN_EXE_md`. It is in its own fence for that reason: an earlier draft had
it sharing a fence with the md-codec tests above, and `plan-api-check.sh`'s
cross-crate report flagged the whole fence, because one fence with two crate
homes is exactly how a helper ends up called from where it cannot be reached.

```rust
// ONE test, in md-cli, doing BOTH halves as a real round trip. Two earlier
// drafts of this failed the same way, and the pattern is worth naming:
//
//   draft 1: one test that only re-parsed  -> never called the renderer at all
//   draft 2: two tests, one per crate      -> item_7b DISCARDED md()'s return,
//            and md() is `fn md(args) -> (String, String, i32)` that does NOT
//            panic (cli_bip388_double_wildcard.rs:35-45), so it could not fail;
//            and the two template strings never corresponded, because
//            keyed_compose_tr_nums_three_leaves renders a NESTED FOUR-KEY tree
//            with older/multi_a/after, not the flat two-key literal.
//
// A fixpoint is encode -> render -> re-encode -> IDENTICAL. Doing it through
// the CLI keeps both halves in one crate and makes every step assert.
#[test]
fn item_7_the_render_reparse_fixpoint_covers_tr_kind_1() {
    let tpl = "tr(UNSPENDABLE(liana),{pk(@0/48'/0'/0'/3'/<0;1>/*),pk(@1/48'/0'/1'/3'/<0;1>/*)})";

    // 1. ENCODE the marker form.
    let (md1, err, code) = md(&["encode", tpl, "--path", "bip48"]);
    assert_eq!(code, 0, "encode failed: {err}");

    // 2. RENDER it back through the real renderer. `md decode` emits the
    //    plain BIP-388 template by default.
    let (rendered, err, code) = md(&["decode", md1.trim()]);
    assert_eq!(code, 0, "decode failed: {err}");
    assert!(rendered.contains("UNSPENDABLE(liana)"),
            "the renderer dropped the marker: {rendered}");
    assert!(!rendered.contains("50929b74"),
            "kind 1 must not render as the raw NUMS hex: {rendered}");

    // 3. RE-ENCODE what the renderer produced. This is the fixpoint, and it is
    //    what md-cli/src/format/text.rs:269-274 asserts for wsh and cannot
    //    currently assert for tr kind 1.
    let (md1b, err, code) = md(&["encode", rendered.trim(), "--path", "bip48"]);
    assert_eq!(code, 0, "the rendered template did NOT re-parse: {err}");
    assert_eq!(md1.trim(), md1b.trim(), "round trip is not a fixpoint");
}
```

- [ ] **Step 6: Gate and commit.**

---

## Task 9: §8.10's mutation pass

**Files:** `design/agent-reports/f449-stage1b-mutations.md` (create).

- [ ] **Step 1: For each mutation, confirm it APPLIED before trusting the result**

A no-op edit reads as a pass. Put a `panic!` in the mutated branch first and watch the suite fail; then replace it with the real mutation.

- [ ] **Step 2: Run the seventeen**

| # | mutation | must be caught by |
| --- | --- | --- |
| 1-3 | reverse / `sort()` / `dedup()` the concat | Task 4 |
| 4 | `wire_version()` returns 4 always | Task 3 + Task 8 Step 2 |
| 5 | render kind 1 as the NUMS hex | Task 5 |
| 6 | a constant 4 at `identity.rs:90`/`:200` | **Task 8 Step 2 — the funds-relevant one** |
| 7 | write the kind bit at version 4 | **stage 1a's byte-equality gate** |
| 8-11 | drop each of §6's four refusals | Task 7 |
| 12-15 | **collapse each of G-1's four or-pattern sites to the NUMS arm** | Task 5 — **if any stays green, that site has no gate and G-1 is unclosed** |
| 16 | **invert the kind bit's polarity on BOTH sides** (`write:199` and `read:377`) | Task 3's golden-bit assertion — a SYMMETRIC inversion is invisible to every round-trip test by construction, so only a golden bit catches it. Stage 3's Go port must agree on polarity or plates written by one are misread by the other. |
| 17 | **weaken the recogniser at `decompose/walk.rs:235` to a NUMS-pubkey/depth-0 pattern-match** (compare `public_key` and `depth` only, ignoring the chain code that carries the leaf hash) | Task 6's `decompose_does_not_recognise_lianas_own_recipe_computed_over_the_wrong_leaves` — a positive-only suite CANNOT see this weakening, because a pattern-match accepts every input the real check accepts. Under it a DIFFERENT wallet's Liana recipe xpub is relabelled `UNSPENDABLE(liana)`. |

- [ ] **Step 3: Record all seventeen verbatim, revert every mutation, confirm the tree is clean**

```bash
git diff --stat                      # must be empty except the report
./scripts/phase-gate.sh
```

- [ ] **Step 4: Commit the report.**

---

## Task 10: §4b's doc retirement, the version bump, and the CHANGELOG

- [ ] **Step 1: Retire the two texts that forbid this stage.** `policy_shape.rs:119-133` ends *"Do not 'restore fidelity' with the Go name here"*, and `DESIGN_coordinator_compatibility.md:155-158` says the same. Both must be retired or the source instructs the next reader not to do what the spec requires. **The phrase wraps across two `///` lines, so a contiguous grep for it returns nothing** — match one line and read the neighbourhood.

- [ ] **Step 2: Bump to 0.46.0 and move md-cli's exact pin IN THE SAME EDIT.** `md-cli/Cargo.toml:28` pins `=0.45.1`; the moment md-codec moves, that is unsatisfiable and cargo fails to resolve **before rustc runs**, so a split commit is red on its own.

- [ ] **Step 3: The CHANGELOG entry attributes the breaking change to STAGE 1A** (G-3) — `Body::Tr`'s shape changed there, at an unchanged version, deliberately. This stage's entry covers wire version 8 and the new kind. **0.46.0, not 0.45.2.**

- [ ] **Step 4: The full gate, then commit** — `./scripts/phase-gate.sh`, `( cd fuzz && cargo check --all-targets )`.

---

## Self-Review

**Spec coverage.** §2 → Task 4. §3a/3c/3d → Tasks 2-3. §3e → Task 2 Step 4. §4 → Task 5. §4a → Task 6. §4b → Task 10. §6 → Task 7. §6a's stage-1b row → Task 2 Step 3. §8 items 1/2/5/6/7 → Task 8; item 10 → Task 9. §9's pin rule → Task 10.

**§6 row 3's `--unspendable liana` no-op WARN is stage 2's**, not this stage's: the flag itself is `md compose`, which §9 assigns to stage 2. A kind-1 request on a path list that already has a bare single key must WARN rather than silently do nothing (fable M-6). Named here so it is not lost between the two plans.

**Owned by later stages, deliberately absent:** §0b's choice screen and §7/§7a's device work (stage 4), §8.3's device leg (stage 4), §8.4's Go leg and §7a.1's three-state port (stage 3), §8.8's live Liana install run (stage 2), §8b and §9a's `me` work (stage 4a), §8.9's operator-facing rows (stages 2-4a).

**Vector names cited by this plan, verified to exist** (the API gate checks Rust
symbols, not string literals naming files — this is its blind spot, and the
first draft cited `keyed_tr_nums_multi_a`, which does not exist):
`keyed_compose_tr_nums_three_leaves` (NUMS + a three-leaf tree) and
`keyed_compose_tr_sole_sortedmulti_a` (NUMS + a sole `sortedmulti_a` leaf).
22 of the 65 vendored vectors are `tr`-shaped.

**Shape was verified, not just existence.** An earlier draft used
`keyed_tr_sortedmulti_a` — the file exists, but it is
`tr(@0/...,sortedmulti_a(...))` with a **spendable** internal key, so
`kind1_from_vector`'s assert would have panicked. Checking that a fixture file
exists is not checking that it is the shape the test needs.

**Placeholder scan — RE-RUN THE SCRIPT, do not restate this sentence.** `./scripts/plan-api-check.sh <this file>` must print *all extracted symbols resolve*. This claim has been **false twice** in this plan's history: r0 asserted it while four helpers were undefined, and r2 asserted it while `in_crate_tr_liana_with_sortedmulti_a_leaf` was undefined in the test encoding a measured funds regression. Helpers currently defined here: `Case`/`leaf_pubkeys`/`all_cases`/`case`/`all_kind0_tr_vectors`/`kind1_from_vector` (Task 1, md-codec); a SECOND `Case`/`case` pair and `ORIGINLESS_SPENDABLE_TR` (Task 6 Step 0, md-cli — one fixture file, two per-crate readers); `tr_liana_with_sortedmulti_a_leaf`/`tr_liana_at_use_site`/`in_crate_tr_liana_with_sortedmulti_a_leaf` (Task 7); `wsh_wrapping_tr_liana`/`all_nums_tr`/`encode_payload_at_forced_version` (Task 7 Step 2b); plus stage 1a's `load_vendored_phrase`, `decode_vendored`, `all_vendored_vector_names`.

**Type consistency.** `InternalKey::{Slot,NumsPoint,LianaUnspendable}` and `Body::Tr { internal_key, tree }` are stage 1a's shipped shape. `Header::WF_UNSPENDABLE_VERSION`, `Descriptor::wire_version()`, `liana_unspendable_xpub`, `LIANA_UNSPENDABLE_MARKER` are used identically throughout.
