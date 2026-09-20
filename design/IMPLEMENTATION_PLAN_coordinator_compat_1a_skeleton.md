# Coordinator compatibility, plan 1a — the Skeleton and its key

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development`
> (recommended) or `superpowers:executing-plans` to implement this plan
> task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give `md-codec` a `Skeleton` — a canonical, coordinator-independent
summary of any decoded md1 — and a `SkeletonKey` with one defined
serialization, so a later plan can key measured evidence by it.

**Architecture:** Port the fork's Go `md/policy_shape.go` branch decomposition
into Rust with two named extensions (retain each branch's placeholder-index
set; add a fourth `KeyPathKind`), add an abstracting render mode over the
existing `descriptor_to_template`, compute two partitions over key identity,
and serialize the whole into one string that is the key.

**Tech Stack:** Rust 2024 edition, rust-version 1.85 (workspace pins), no new
dependencies.

**Spec:** `design/DESIGN_coordinator_compatibility.md` at `83ab725c`. Read §1,
§1A and §2 before starting; this plan argues from them.

**Repo:** `/scratch/code/shibboleth/descriptor-mnemonic` (primary, Rust).
Nothing in this plan touches the fork.

## Why this is plan 1a and not plan 1

The design's §"Scope" splits the work three ways and calls steps 1-2 "plan 1".
Writing it found a further boundary: **this plan produces a working,
independently testable artifact — a key computable from any decoded md1 — and
holds all of the risk.** Plan 1b (the four `Verdict` kinds, the rules, the
generated table, `md shape-key`, `md compose`'s refusal) consumes the key and
cannot start before it exists. Splitting keeps each plan reviewable against
something that runs.

## Global Constraints

- **Rust-primary.** This is the primary repo; nothing here may be ported *from*
  Go except `policy_shape.go`, which is fork-native with no Rust counterpart,
  so porting it MAKES Rust primary for it (design §1A).
- **Workspace pins:** edition 2024, `rust-version = "1.85"`. No new dependencies.
- **The key is computed from a DECODED `Descriptor` only** — never from the
  compose-side `PathList`. A restored card has no `PathList`, and two
  implementations of one key is the defect design §2 forbids.
- **Key membership is fixed** (design §1A): `root + inner_wsh + template +
  fp_partition + key_partition + key_path_kind`, and nothing else.
  `keys_present` is NOT in the key.
- **Slot ids are 0-based; equality classes are 1-based.**
- **Gate for every task:** `cargo test -p md-codec`, `cargo clippy --all-targets
  -- -D warnings`, `cargo fmt --check`.

---

### Task 1: Port the branch decomposition, retaining each branch's slot set

**Files:**
- Create: `crates/md-codec/src/policy_shape.rs`
- Modify: `crates/md-codec/src/lib.rs` (add `pub mod policy_shape;` in the
  alphabetical run, between `pub mod phrase;` and `pub mod render;`)
- Test: `crates/md-codec/tests/policy_shape.rs`

**Interfaces:**
- Consumes: `md_codec::encode::Descriptor`, `md_codec::tree::{Node, Body}`,
  `md_codec::tag::Tag`.
- Produces: `RootKind`, `PolicyShape`, `Branch`, `KeyPathKind`, and
  `pub fn policy_shape(d: &Descriptor) -> PolicyShape`.

**The extension, and why it is the first task.** The Go original
(`md/policy_shape.go:239-245`) builds `keys := map[uint8]struct{}{}` — the
placeholder indices the branch references — then writes `br.Keys = len(keys)`
and **discards the map**. `Branch.slots` below retains it. Without that field
the fingerprint partition of Task 4 has no source, and a verbatim port would
produce a `Skeleton` whose central field cannot be filled.

- [ ] **Step 1: Write the failing test**

```rust
// crates/md-codec/tests/policy_shape.rs
mod common;
use common::{keyarg, multikeys, node2, timelock, wrap};
use md_codec::policy_shape::{policy_shape, KeyPathKind};
use md_codec::tag::Tag;

/// `wsh(or_d(multi(2,@0,@1,@2), and_v(v:pkh(@3), older(26280))))`
fn kofn_recovery() -> md_codec::encode::Descriptor {
    let primary = multikeys(Tag::Multi, 2, vec![0, 1, 2]);
    let recovery = node2(
        Tag::AndV,
        wrap(Tag::Verify, keyarg(Tag::Pkh, 3)),
        timelock(Tag::Older, 26280),
    );
    let tree = wrap(Tag::Wsh, node2(Tag::OrD, primary, recovery));
    common::descriptor_of(tree, 4)
}

#[test]
fn branch_retains_its_slot_set_not_just_a_count() {
    let shape = policy_shape(&kofn_recovery());
    assert!(shape.complete, "the walk must classify every node of a shipped preset");
    assert_eq!(shape.key_path, KeyPathKind::NotTaproot);
    assert_eq!(shape.branches.len(), 2, "or_d yields two spend paths");

    // THE EXTENSION: which slots, not how many.
    assert_eq!(shape.branches[0].slots, vec![0, 1, 2]);
    assert_eq!(shape.branches[1].slots, vec![3]);
    // The count the Go original kept is still derivable, and must agree.
    assert_eq!(shape.branches[0].slots.len(), 3);
}
```

- [ ] **Step 2: Add the test helper the test needs**

`crates/md-codec/tests/common/mod.rs` has node builders but no whole-descriptor
builder. Append:

```rust
/// Build a template-only `Descriptor` around `tree` with `n` placeholders.
/// (Tests are a separate crate, so `md_codec::` is correct HERE.)
/// Template-only is deliberate: Task 1 tests structure, not key identity.
pub fn descriptor_of(tree: Node, n: u8) -> md_codec::encode::Descriptor {
    md_codec::encode::Descriptor {
        n,
        path_decl: md_codec::origin_path::PathDecl::default(),
        use_site_path: md_codec::use_site_path::UseSitePath::default(),
        tree,
        tlv: md_codec::tlv::TlvSection::default(),
    }
}
```

If any of those three types lacks a `Default`, construct the same value the
crate's own tests already use — `rg 'Descriptor \{' crates/md-codec/tests`
shows the established spelling — rather than adding a `Default` impl to the
library for a test's convenience.

- [ ] **Step 3: Run it and watch it fail**

Run: `cargo test -p md-codec --test policy_shape -- --nocapture`
Expected: FAIL to compile — `unresolved import md_codec::policy_shape`.

- [ ] **Step 4: Write the port**

Port `md/policy_shape.go` function by function. The Go file is 438 lines and
is the specification for this step — read it, do not guess. The type it
produces:

```rust
// crates/md-codec/src/policy_shape.rs

/// The top-level wrapper. SIX-valued, mirroring the fork's `ScriptKind`.
/// NOT `compose::Wrapper` (`crates/md-codec/src/compose/mod.rs:73`): that is
/// four-valued, is the compose-side input model the design forbids keying
/// from, and cannot express a decoded singlesig `wpkh`/`pkh`/`sh(wpkh)` —
/// all of which md1 encodes. `sh(wsh)` is not a variant here either; the
/// `Skeleton` carries `inner_wsh: bool` beside this (design §1A).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootKind { Wpkh, Pkh, Sh, Wsh, Tr, ShWpkh }

/// A taproot internal key. FOUR-valued: the Go original has three
/// (`None`/`NUMS`/`Spendable`, `md/policy_shape.go:33-39`) and an
/// unspendable-xpub internal key falls into `Spendable` there. Nunchuk treats
/// it as a different wallet and F-449 records it, so the port adds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyPathKind {
    NotTaproot,
    Nums,
    UnspendableXpub,
    Spendable,
}

/// One spend path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub k: u8,
    pub n: u8,
    /// EXTENSION over the Go original, which keeps only `len()` of this.
    /// Ascending, deduplicated.
    pub slots: Vec<u8>,
    pub sorted: bool,
    pub locks: Vec<Lock>,
    pub hashlocks: Vec<HashLock>,
    /// Taptree depth of this leaf; 0 for wsh/sh.
    pub depth: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lock { pub kind: LockKind, pub value: u32 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockKind { AfterHeight, AfterTime, OlderBlocks, OlderUnits }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HashLock { pub kind: HashKind, pub digest: [u8; 32], pub len: u8 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashKind { Sha256, Hash256, Ripemd160, Hash160 }

/// Structural summary of one decoded policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyShape {
    /// The honesty contract, ported verbatim in meaning from the Go: FALSE
    /// means the walk met a node it could not classify, and NO part of this
    /// summary may be presented or keyed on.
    pub complete: bool,
    pub key_path: KeyPathKind,
    pub branches: Vec<Branch>,
    pub tap_depth: u8,
}

// NOTE: `crate::`, never `md_codec::` — this crate does not self-alias
// (`lib.rs` has no `extern crate self as md_codec`). Integration tests under
// `tests/` are a separate crate and correctly say `md_codec::`.
pub fn policy_shape(d: &crate::encode::Descriptor) -> PolicyShape { todo!() }
```

- [ ] **Step 5: Run the test until it passes**

Run: `cargo test -p md-codec --test policy_shape`
Expected: PASS.

- [ ] **Step 6: Prove `complete` can be false**

A summary whose honesty flag never fires is not an honesty flag. Add:

```rust
#[test]
fn an_unclassifiable_node_sets_complete_false_and_yields_no_branches() {
    // A bare `thresh` at the root is not a spend-path decomposition this
    // walk models; the Go original returns Complete=false for it.
    let tree = wrap(Tag::Wsh, common::thresh_node(2, vec![
        keyarg(Tag::PkK, 0), keyarg(Tag::PkK, 1), keyarg(Tag::PkK, 2),
    ]));
    let shape = policy_shape(&common::descriptor_of(tree, 3));
    assert!(!shape.complete);
    assert!(shape.branches.is_empty(),
        "an incomplete walk must not hand back a partial decomposition");
}
```

If the Go original DOES classify this shape, pick whatever node it refuses —
`rg 'return.*false' md/policy_shape.go` in the fork names them — and pin that
instead. The test must fail if `complete` is hardcoded `true`.

- [ ] **Step 7: Commit**

```bash
git add crates/md-codec/src/policy_shape.rs crates/md-codec/src/lib.rs \
        crates/md-codec/tests/policy_shape.rs crates/md-codec/tests/common/mod.rs
git commit -m "md-codec: port the branch decomposition, retaining each branch's slot set

Ported from the fork's md/policy_shape.go, which is fork-native with no Rust
counterpart -- so this makes Rust primary for it. Two extensions over the
original, both required by the coordinator-compat key:

  Branch.slots keeps WHICH placeholders a branch references. branchOf
  (md/policy_shape.go:239-245) builds exactly this map and then writes
  br.Keys = len(keys), discarding it -- so a verbatim port yields a count and
  the fingerprint partition has no source.

  KeyPathKind gains UnspendableXpub. The Go enum is three-valued and an
  unspendable-xpub internal key falls into Spendable, which is a distinction
  Nunchuk makes and F-449 records."
```

---

### Task 2: The abstracting render mode

**Files:**
- Modify: `crates/md-codec/src/render.rs` (add the mode; do not change the
  existing public signature)
- Test: `crates/md-codec/tests/render_abstract.rs`

**Interfaces:**
- Consumes: the existing `descriptor_to_template(&Descriptor) -> Result<String, RenderError>`.
- Produces: `pub fn descriptor_to_abstract_template(d: &Descriptor) -> Result<String, RenderError>`.

**What this adds, precisely.** `descriptor_to_template`
(`crates/md-codec/src/render.rs:52`) already emits `@i` placeholders and
**erases origins** — that half is free. What it does not do is abstract lock
values and digests: `crates/md-codec/src/render.rs:159` writes `older({v})`
and `:171` writes `after({v})` literally, and the two hash renderers emit the
digest bytes.

- [ ] **Step 1: Write the failing test**

```rust
// crates/md-codec/tests/render_abstract.rs
mod common;
use md_codec::render::descriptor_to_abstract_template;

#[test]
fn equal_lock_values_share_a_class_and_different_ones_do_not() {
    // wsh(or_i(and_v(v:pkh(@0),older(26280)),
    //          or_i(and_v(v:pkh(@1),older(1000)),
    //               and_v(v:pkh(@2),older(26280)))))
    let d = common::three_older_descriptor(26280, 1000, 26280);
    let t = descriptor_to_abstract_template(&d).unwrap();
    assert!(t.contains("older(older-blocks#1)"), "got {t}");
    assert!(t.contains("older(older-blocks#2)"), "got {t}");
    assert_eq!(t.matches("older-blocks#1").count(), 2,
        "the two equal values share class 1: {t}");
    assert!(!t.contains("26280"), "no literal lock value may survive: {t}");
}

#[test]
fn digests_carry_a_class_too_symmetric_with_locks() {
    let d = common::two_sha256_descriptor([0x11; 32], [0x11; 32]);
    let t = descriptor_to_abstract_template(&d).unwrap();
    assert_eq!(t.matches("sha256(#1)").count(), 2,
        "two branches committing to the SAME digest share a class: {t}");
    assert!(!t.contains("1111"), "no literal digest may survive: {t}");
}

#[test]
fn the_use_site_is_kept() {
    let d = common::kofn_recovery_with_use_site();
    let t = descriptor_to_abstract_template(&d).unwrap();
    assert!(t.contains("/<0;1>/*"),
        "the key is looked up against evidence whose template field keeps it: {t}");
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p md-codec --test render_abstract`
Expected: FAIL to compile — `descriptor_to_abstract_template` not found.

- [ ] **Step 3: Implement the mode**

Thread a private mode flag through `render_node` rather than writing a second
renderer — two renderers drift, and this repo has measured that failure. Class
numbering: **per kind**, a counter over distinct values of that kind within
this policy, base 10, starting at 1, assigned in the template's own
left-to-right traversal order.

```rust
// in crates/md-codec/src/render.rs
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode { Literal, Abstract }

/// Render with lock values and digests replaced by `kind#class`.
/// Used as the coordinator-compatibility key's template component.
pub fn descriptor_to_abstract_template(d: &Descriptor) -> Result<String, RenderError> {
    render_with(d, Mode::Abstract)
}
```

- [ ] **Step 4: Run the tests until they pass**

Run: `cargo test -p md-codec --test render_abstract`
Expected: PASS, all three.

- [ ] **Step 5: Prove the literal renderer is unchanged**

Run: `cargo test -p md-codec`
Expected: PASS. Every existing `render` test exercises `Mode::Literal` and
must be untouched — if one moved, the mode flag leaked.

- [ ] **Step 6: Commit**

```bash
git add crates/md-codec/src/render.rs crates/md-codec/tests/render_abstract.rs \
        crates/md-codec/tests/common/mod.rs
git commit -m "md-codec: an abstracting render mode for the compatibility key

descriptor_to_template already emits @i placeholders and erases origins. What
it does not do is abstract lock values and digests -- render.rs:159 and :171
write older({v}) and after({v}) literally. The key needs older(older-blocks#1)
and sha256(#1), so the mode threads through render_node rather than growing a
second renderer that would drift from the first.

Digests carry a class symmetric with locks: two branches committing to the
SAME digest is a different wallet from two committing to different ones."
```

---

### Task 3: The two partitions

**Files:**
- Modify: `crates/md-codec/src/policy_shape.rs`
- Test: `crates/md-codec/tests/partitions.rs`

**Interfaces:**
- Consumes: `PolicyShape` (Task 1), `Descriptor`'s `tlv` fingerprints and pubkeys.
- Produces: `pub fn fp_partition(d: &Descriptor, s: &PolicyShape) -> Vec<Vec<Vec<u8>>>`
  and `pub fn key_partition(d: &Descriptor) -> Vec<Vec<u8>>`.

**Two relations, two partitions** (design §1A): `fp_partition` is **per path**
and exists for Liana's `DuplicateOriginSamePath`; `key_partition` is
**whole-policy** and exists for its `DuplicateKey`. Collapsing them makes one
of the two refusals uncomputable.

- [ ] **Step 1: Write the failing test**

```rust
// crates/md-codec/tests/partitions.rs
mod common;
use md_codec::policy_shape::{fp_partition, key_partition, policy_shape};

#[test]
fn slots_sharing_a_fingerprint_group_within_a_path() {
    // Path 0 seats @0 and @1 from ONE seed, @2 from another.
    let d = common::seated(&[(0, [0xaa; 4]), (1, [0xaa; 4]), (2, [0xbb; 4]), (3, [0xcc; 4])]);
    let p = fp_partition(&d, &policy_shape(&d));
    assert_eq!(p[0], vec![vec![0u8, 1], vec![2]], "path 0: {p:?}");
    assert_eq!(p[1], vec![vec![3u8]], "path 1: {p:?}");
}

#[test]
fn an_absent_fingerprint_is_its_own_singleton() {
    // md-codec's ABSENT sentinel is all-zero; two absent slots are NOT known
    // to share a signer, and grouping them would assert a relation nobody
    // measured.
    let d = common::seated(&[(0, [0x00; 4]), (1, [0x00; 4]), (2, [0xbb; 4]), (3, [0xcc; 4])]);
    let p = fp_partition(&d, &policy_shape(&d));
    assert_eq!(p[0], vec![vec![0u8], vec![1], vec![2]],
        "absent fingerprints never join: {p:?}");
}

#[test]
fn key_partition_groups_by_xpub_and_origin_path_across_the_whole_policy() {
    // @0 and @3 are the SAME key at the same origin, in different paths.
    let d = common::seated_same_key(&[0, 3]);
    let kp = key_partition(&d);
    assert!(kp.contains(&vec![0u8, 3]), "whole-policy grouping: {kp:?}");
}

#[test]
fn a_template_only_card_partitions_to_nothing() {
    let d = common::kofn_recovery();   // template-only, no Pubkeys TLV
    assert!(fp_partition(&d, &policy_shape(&d)).iter().all(|p| p.is_empty()));
    assert!(key_partition(&d).is_empty());
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p md-codec --test partitions`
Expected: FAIL to compile.

- [ ] **Step 3: Implement both**

Group ascending by slot; order groups by their lowest slot. "Derivation" in the
design's clause means the **origin path** — the only derivation a decoded md1
carries. An absent xpub is its own singleton by the same argument as the
fingerprint rule.

- [ ] **Step 4: Run until green, then run the whole crate**

Run: `cargo test -p md-codec`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/md-codec/src/policy_shape.rs crates/md-codec/tests/partitions.rs \
        crates/md-codec/tests/common/mod.rs
git commit -m "md-codec: the per-path fingerprint and whole-policy key partitions

Two relations, two partitions. fp_partition is per path, for Liana's
DuplicateOriginSamePath; key_partition is whole-policy, for its DuplicateKey.
Collapsing them makes one of the two refusals uncomputable.

An absent fingerprint is its OWN singleton. md-codec already rules that
[0,0,0,0] is the ABSENT sentinel rather than a value, so two absent slots are
not known to share a signer and grouping them would assert a key-identity
relation nobody measured."
```

---

### Task 4: `Skeleton`, `SkeletonKey`, and the one serialization

**Files:**
- Create: `crates/md-codec/src/skeleton.rs`
- Modify: `crates/md-codec/src/lib.rs` (add `pub mod skeleton;`)
- Test: `crates/md-codec/tests/skeleton_key.rs`

**Interfaces:**
- Consumes: Tasks 1-3.
- Produces: `pub struct Skeleton`, `pub struct SkeletonKey(String)`,
  `pub fn skeleton(d: &Descriptor) -> Result<Skeleton, SkeletonError>`,
  `pub fn skeleton_key(s: &Skeleton) -> SkeletonKey`.

**Membership is fixed and exhaustive** (design §1A): `root + inner_wsh +
template + fp_partition + key_partition + key_path_kind`, and nothing else.
`keys_present` is deliberately **not** in the key — it selects the verdict, it
does not make a template-only card a different policy from the same card
seated.

- [ ] **Step 1: Write the failing test**

```rust
// crates/md-codec/tests/skeleton_key.rs
mod common;
use md_codec::skeleton::{skeleton, skeleton_key};

#[test]
fn seated_and_unseated_do_not_share_a_key() {
    let unseated = common::kofn_recovery();
    let seated = common::seated(&[(0, [0xaa; 4]), (1, [0xbb; 4]), (2, [0xcc; 4]), (3, [0xdd; 4])]);
    // Distinct fingerprints => every slot its own group => same partition
    // shape as template-only? NO: template-only partitions to nothing. The
    // key differs, and that is correct -- identity is part of the key.
    assert_ne!(skeleton_key(&skeleton(&unseated).unwrap()),
               skeleton_key(&skeleton(&seated).unwrap()));
}

#[test]
fn a_nums_key_path_and_an_unspendable_xpub_do_not_share_a_key() {
    let nums = common::tr_nums_two_leaves();
    let xpub = common::tr_unspendable_xpub_two_leaves();
    assert_ne!(skeleton_key(&skeleton(&nums).unwrap()),
               skeleton_key(&skeleton(&xpub).unwrap()),
        "Nunchuk treats these as different wallets (F-449)");
}

#[test]
fn sh_wsh_and_bare_sh_do_not_share_a_key() {
    // sh(wsh) is NOT a ScriptKind value; it is root=Sh plus inner_wsh=true.
    assert_ne!(skeleton_key(&skeleton(&common::sh_wsh_2of3()).unwrap()),
               skeleton_key(&skeleton(&common::bare_sh_2of3()).unwrap()));
}

#[test]
fn the_serialization_is_stable_and_documented() {
    let k = skeleton_key(&skeleton(&common::kofn_recovery()).unwrap());
    let s = k.as_str();
    assert!(s.contains('\u{001F}'), "template and partitions are separated: {s}");
    assert_eq!(skeleton_key(&skeleton(&common::kofn_recovery()).unwrap()).as_str(), s,
        "the key is a pure function of the descriptor");
}

#[test]
fn an_incomplete_walk_yields_no_key() {
    let d = common::unclassifiable();
    assert!(skeleton(&d).is_err(),
        "PolicyShape.complete=false must not produce a key -- a partial \
         decomposition keyed as if whole is a false evidence match");
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p md-codec --test skeleton_key`
Expected: FAIL to compile.

- [ ] **Step 3: Implement**

```rust
// crates/md-codec/src/skeleton.rs
pub struct Skeleton {
    pub root: RootKind,
    pub inner_wsh: bool,
    pub template: String,
    pub shape: PolicyShape,
    pub fp_partition: Vec<Vec<Vec<u8>>>,
    pub key_partition: Vec<Vec<u8>>,
    pub keys_present: bool,
}
```

Serialization, exactly: the abstract template, `U+001F`, then the partitions
rendered `[path][group][slot]` with slots ascending, groups ordered by their
lowest slot, paths in template traversal order; then `U+001F` and the
key-path kind. Slot ids 0-based, equality classes 1-based.

- [ ] **Step 4: Run until green**

Run: `cargo test -p md-codec --test skeleton_key`
Expected: PASS, all five.

- [ ] **Step 5: Commit**

```bash
git add crates/md-codec/src/skeleton.rs crates/md-codec/src/lib.rs \
        crates/md-codec/tests/skeleton_key.rs crates/md-codec/tests/common/mod.rs
git commit -m "md-codec: Skeleton and SkeletonKey, with one defined serialization

Membership is exhaustive and fixed by the design: root + inner_wsh + template
+ fp_partition + key_partition + key_path_kind. keys_present is deliberately
NOT in the key -- it selects the verdict, it does not make a template-only
card a different policy from the same card seated.

An incomplete walk yields no key at all. A partial decomposition keyed as if
whole would match evidence measured on a policy it is not."
```

---

### Task 5: The conformance gate — one key, two routes

**Files:**
- Create: `crates/md-codec/tests/skeleton_key_conformance.rs`
- Create: `crates/md-codec/examples/dump_skeleton_keys.rs`

**Interfaces:**
- Consumes: Task 4's `skeleton_key`.
- Produces: the example binary a later plan's table generator runs. **It lives
  in `md-codec/examples/`, not in `md-cli`** — plan 1b's table is gated by a
  test in this crate, and a step-1 gate may not depend on a step-2 binary.

**This is the gate the design names:** `chunks -> key == descriptor -> key`
for every evidence row. It is what proves the key is a property of the policy
rather than of the route taken to it.

- [ ] **Step 1: Write the failing test**

```rust
// crates/md-codec/tests/skeleton_key_conformance.rs
mod common;
use md_codec::skeleton::{skeleton, skeleton_key};

/// Every vendored keyed vector, keyed twice: once from the md1 chunk set and
/// once from the descriptor the same record carries. The key must not know
/// which door it came through.
#[test]
fn chunks_and_descriptor_yield_the_same_key() {
    let mut checked = 0;
    for path in glob_keyed_conformance_vectors() {
        let rec = load(&path);
        let from_chunks = skeleton_key(&skeleton(&decode_chunks(&rec.phrase)).unwrap());
        let from_desc = skeleton_key(&skeleton(&parse_descriptor(&rec.chains["0"].descriptor)).unwrap());
        assert_eq!(from_chunks, from_desc, "{}: the key knows its route", rec.name);
        checked += 1;
    }
    assert!(checked >= 40, "only {checked} vectors keyed -- the gate is checking almost nothing");
}
```

The `checked >= 40` floor is deliberate: a glob that matches nothing passes
every assertion in its body. This repo has shipped that defect.

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p md-codec --test skeleton_key_conformance`
Expected: FAIL — either to compile, or on the first vector.

- [ ] **Step 3: Make it pass**

Any disagreement here is a real defect in Tasks 1-4, not a reason to relax the
test. The likely causes, in order: the descriptor route loses the use-site the
chunk route keeps; the two routes disagree on placeholder numbering (the
chunk route is canonical by construction, the descriptor route may not be —
call `canonicalize_placeholder_indices` on both); or the descriptor route has
no fingerprints and so partitions to nothing.

**If the third is the cause, that is a finding, not a fixup:** it means a key
is not computable from a descriptor alone, and plan 1b's evidence table — which
is keyed from descriptors — needs a different input. Stop and report it.

- [ ] **Step 4: Write the example binary**

```rust
// crates/md-codec/examples/dump_skeleton_keys.rs
//! Prints `<name>\t<key>` for every vendored keyed vector.
//! Plan 1b's evidence table is a transcript of THIS -- one implementation of
//! the key, never a second one in Python.
fn main() { /* … */ }
```

- [ ] **Step 5: Run it and eyeball the output**

Run: `cargo run -p md-codec --example dump_skeleton_keys | head -5`
Expected: five `name<TAB>key` lines, each key containing `U+001F`.

- [ ] **Step 6: Full gate and commit**

```bash
cargo test -p md-codec && cargo clippy --all-targets -- -D warnings && cargo fmt --check
git add crates/md-codec/tests/skeleton_key_conformance.rs \
        crates/md-codec/examples/dump_skeleton_keys.rs
git commit -m "md-codec: the key is a property of the policy, not of the route

chunks -> key == descriptor -> key over every vendored keyed vector, with a
floor on the count because a glob that matches nothing passes every assertion
in its body -- a defect this repo has shipped.

The example binary is the ONE implementation of the key. Plan 1b's evidence
table is a transcript of it, never a second computation in Python: two
implementations of one key drift, and the failure is silent in the worst
direction."
```

---

## Machine-check run before this plan was reviewed

`scripts/plan-build-gate.sh` is hardcoded to an older plan's crate and does not
apply here, and a full compile would be meaningless anyway — most of this
plan's Rust is code that does not exist yet by design. What IS checkable is
every **existing** item the plan says it consumes, and that was checked:

| checked | result |
| --- | --- |
| `Tag` variants used in test code | **2 of 11 were wrong** — `Tag::V` and `Tag::Pk` do not exist; corrected to `Tag::Verify` and `Tag::PkK` |
| `tests/common/mod.rs` helpers called | 12 of 12 exist |
| md-codec paths cited | 7 of 7 exist |
| `Descriptor` fields used by `descriptor_of` | `n`, `path_decl`, `use_site_path`, `tree`, `tlv` — all five confirmed at `crates/md-codec/src/encode.rs:17-28` |

The two wrong tags are the point of running it: a reviewer finding
`Tag::V` would have been a reviewer paid design rates to act as a compiler,
and the finding would have arrived a round late.

## Self-review

**Spec coverage.** Design §1A's `Skeleton` fields: `root` T4, `inner_wsh` T4,
`template` T2, `shape` T1, `fp_partition` T3, `key_partition` T3,
`keys_present` T4. The two named port extensions: T1. `kind#class`: T2. The
absent-fingerprint rule: T3. Key membership and serialization: T4. The
conformance gate: T5. **Not in this plan, and belonging to 1b:** the four
`Verdict` kinds, `ReadSet`, `EvidenceRow`, `MeasuredAt`, the rules, the
generated table, `md shape-key`, `md compose`'s refusal, the D1-D4
disagreement classes. §1(a2)'s template-only verdict and §1(a3)'s
unexpandable-keys verdict are **verdict** rules and are 1b's, but both depend
on `keys_present` and on `skeleton()` returning `Err`, which T4 provides.

**Placeholders.** None: every code step carries code; every test step carries
an assertion; no "similar to Task N".

**Type consistency.** `PolicyShape`/`Branch`/`KeyPathKind`/`Lock`/`HashLock`
defined T1, used T3-T4. `descriptor_to_abstract_template` defined T2, used T4.
`fp_partition`/`key_partition` defined T3, used T4. `skeleton`/`skeleton_key`
defined T4, used T5. `common::descriptor_of` added T1, used throughout.

**One known gap, stated rather than papered over:** Tasks 3-5 call
`common::` helpers (`seated`, `seated_same_key`, `tr_nums_two_leaves`,
`tr_unspendable_xpub_two_leaves`, `sh_wsh_2of3`, `bare_sh_2of3`,
`unclassifiable`, `three_older_descriptor`, `two_sha256_descriptor`,
`kofn_recovery_with_use_site`) that this plan does not spell out, because each
is a few lines over the builders `tests/common/mod.rs` already exports and
writing ten of them here would bury the tasks. **Each is built in the step
that first needs it, using the existing builders** — `keyarg`, `multikeys`,
`node2`, `node3`, `tr_node`, `taptree2`, `timelock`, `hash32`, `hash20`,
`wrap`, `thresh_node`, `renumbered` —
and an implementer who cannot build one from those should stop and report
rather than invent a shape.
