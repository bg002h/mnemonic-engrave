# F-449 Stage 1 (1a + 1b) — md-codec Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Give md1 a second taproot internal-key kind — Liana's unspendable xpub, derived from the leaf keys — carried on a new wire version 8, so a `tr` policy with a multi-key primary imports into Liana.

**Architecture:** Two commits in sequence. **1a** replaces `Body::Tr`'s `is_nums: bool` + `key_index: u8` pair with an `InternalKey` sum type, changing **no wire bytes** — a pure refactor whose gate is byte-equality against the current encoder. **1b** then adds `Header::WF_UNSPENDABLE_VERSION = 8`, a 1-bit `kind` field written only at version 8, the §2 derivation, rendering, the input-side recogniser, and refusals. Splitting them is deliberate: a ~59-site mechanical rename in the same diff as a funds-relevant wire change produces a diff nobody can review.

**Tech Stack:** Rust (crates `md-codec`, `md-cli`), `bitcoin` + `rust-miniscript`, `cargo nextest`.

**Status:** r1, folded from the stage-1a R0 (2C/8I/8M/3N, `design/agent-reports/f449-plan-stage1a-r0.md`). Awaiting re-review.
(0C/6I/16M). Reports: `design/agent-reports/f449-plan-stage1-{r0,r1}.md`.
Awaiting re-review.

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
- **USE THE PINNED CLIPPY, or you will chase lints CI never sees.**
  `rust-toolchain.toml` pins **1.85.0**, but a bare `cargo clippy` on this
  machine runs **clippy 0.1.98** and fails the workspace at baseline with
  `needless_range_loop` in `crates/md-codec/tests/parity_smoke.rs:228`,
  `contains()`/`iter().any()`, and a `format!` lint — **none of which are
  defects, and none of which CI reports.** Measured on unmodified `main`:

  | clippy | `-D warnings` exit |
  | --- | --- |
  | `0.1.98` (bare `cargo clippy`) | **101** |
  | `0.1.85` (pinned) | **0**, zero diagnostics |

  Prepend the pinned toolchain's bin, which is what makes the versions match:

  ```bash
  export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH
  cargo clippy --version   # must print 0.1.85
  ```

  If you "fix" a `parity_smoke.rs` lint, you have edited unrelated test code to
  satisfy a compiler CI does not run.
- **Baseline in the worktree is green on all six CI commands** (fmt 0,
  clippy-pinned 0, doc 0, tests 1400 passed / 3 skipped). Any gate failure
  during this stage is therefore attributable to your change.
- **Every task ends green.** A red suite is itself a blocking finding.

---

## Scope of THIS plan

**Stage 1a only** — and deliberately so. The combined 1a+1b plan reached 1300
lines and three review rounds (1C/10I, 0C/6I, 0C/3I/21M) without closing;
measured, **15 of its 17 undefined test fixtures belonged to 1b's tasks**, so
each round found one more of the same defect. Stage 1a produces working,
testable software on its own — a behaviour-preserving refactor with a
byte-equality gate — so it is planned, gated and shipped separately.

**Stage 1b** (wire version 8, the kind bit, §2's derivation, §4 rendering,
§4a's input side, §6's refusals, §8's vectors, the mutation pass, the version
bump) gets its own plan, written against a tree where 1a has already landed.
Its R0 carries forward the findings already banked against it:

| carried finding | what 1b's plan must answer |
| --- | --- |
| r2 IMP-7 | `encode_md1_chunks` does not exist — `chunk.rs:240 split(&Descriptor) -> Vec<String>` is the producer; and a kind-1 `Descriptor` cannot be built from `cases.json`'s TLV bytes alone (no taptree, no `older`, no fingerprints, no divergent origins). Build it by **decoding a vendored kind-0 vector and swapping `internal_key`**, which carries all four for free |
| r2 IMP-8 | the encode-only hook is right but the stated reason is wrong (no plate carries kind 1 yet), and it disarms §6 row 1's purpose — that row guards a **port** error, and a port's plate arrives via **decode**. The render/derive boundary is the one that catches it without making plates undecodable |
| r2 IMP-9 | the identity golden is a 4-tuple and the reader a 2-tuple, pinning 1 of 3 values; `compute_id_by_name` undefined. Real APIs: `compute_wallet_policy_id`, `compute_wallet_descriptor_template_id`, `WalletPolicyId::to_phrase` |
| r2 MIN-19 | `encode_payload_unchecked` does not exist and `Admission` is `pub(crate)`, so "a refused shape still decodes" must be a **unit** test inside the crate, not an integration test |
| r2 MIN-20 | `cmd/vectors.rs:193` is a **third** production caller of the network-less form, beyond `cmd/descriptor.rs` and `derive.rs:134` |
| r2 MIN-17 | citations: validators also run at `encode.rs:147`/`:149`; decode's run `:118`-`:153` |

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
| `crates/md-cli/src/decompose/{mod,walk}.rs` | the recogniser |
| `crates/md-codec/tests/liana_unspendable.rs` | **new** — §8 vectors 1, 2, 5, 6 |

**Measured blast radius for Task 1** (brace-matched against `#[cfg(test)]`, not guessed):

| crate | production sites | test sites |
| --- | --- | --- |
| `md-codec/src` | **47** | 41 |
| `md-cli/src` | **12** | 25 |
| `md-codec/tests/` + `md-cli/tests/` | **38** (integration tests — r0 omitted these entirely) | — |

The load-bearing one in that third row is a shared helper whose signature **is
the pair being replaced**:

```rust
// crates/md-codec/tests/common/mod.rs:85
pub fn tr_node(is_nums: bool, key_index: u8, tree: Option<Node>) -> Node
```

It has **29 call sites** across the test suites. Change its signature to
`tr_node(internal_key: InternalKey, tree: Option<Node>)` and update all 29; that
is most of the 38. A plan that counts only `src/` understates this task by a
third.

---

## Task 0: vendor the Liana evidence, and capture BOTH pre-change goldens

**r0 cited the evidence by bare relative path. It lives in a DIFFERENT
REPOSITORY** — `mnemonic-engrave/design/evidence/composer-fable-r0/` — while
the code under test is in `descriptor-mnemonic`, which has no `design/evidence/`
at all. A generator committed here cannot read it, so r0's "reproducible rather
than transcribed" claim was false.

**Files:** create `crates/md-codec/tests/golden/pre_refactor_{encodings,ids}.json`
and `crates/md-codec/examples/dump_{encodings,ids}.rs`. **Nothing under
`crates/md-codec/tests/vectors/`** — that is a generated corpus under a
`diff -r` drift test.

**`cases.json` is NOT in stage 1a.** r0 of this plan vendored the Liana
evidence here and put it under `crates/md-codec/tests/vectors/`. Two things
were wrong: that fixture serves stage **1b**'s Liana tests and stage 1a never
reads it, and `tests/vectors/` is a **generated** corpus guarded by a `diff -r`
drift test (`md-cli/tests/vector_corpus.rs:22,26`) that runs inside this plan's
own gate — a new subdirectory there turns it RED. Stage 1b vendors it, under
`crates/md-codec/tests/fixtures/liana/`, which no drift test covers.

Stage 1a needs exactly two artifacts, both under `tests/golden/`: the encodings
golden and the identity golden.

- [ ] **Step 1: Write the two example binaries — they do not exist yet**

r0 ran `cargo run --example dump_encodings` in a step before any step created
it, and never created `dump_ids.rs` at all. Both are new files.

`crates/md-codec/examples/dump_encodings.rs` and `examples/dump_ids.rs` share
the same skeleton, modelled on the existing `examples/dump_skeleton_keys.rs`:
enumerate `tests/vectors/*.phrase.txt` (**all of them — do NOT reuse
`keyed_phrase_files()`, which filters `starts_with("keyed_")` and yields 46**),
read each with `load_vendored_phrase` (Task 1 Step 1a), decode with
`decode_vendored`, then emit JSON on stdout.

- `dump_encodings` → `{count, tr_count, vectors: [[name, hex_of_encode_payload]]}`
- `dump_ids` → `{count, vectors: [[name, wallet_policy_id, template_id, phrase]]}`
  using `compute_wallet_policy_id`, `compute_wallet_descriptor_template_id` and
  `WalletPolicyId::to_phrase` — those are the real API names.

Examples are **not** built by `cargo build`; build them with `--all-targets` or
by running them.

- [ ] **Step 2: Capture the encodings golden — from UNMODIFIED code**

**The corpus is all 65 — there is nothing to skip, and r0's skip rule was built
on a misdiagnosis I propagated.** The 13 files that fail `reassemble` are not
"v0.14-era wire version 2". They are **current single-payload md1 strings**:
`test_vectors.rs` has exactly **13** entries with `force_chunked: false`
(verified). `reassemble` is the *chunk* reader, and a chunk header reads bits
4..1 of a single-payload first symbol as its version — which for version 4
yields **2**. `header.rs:109` already pins that exact error, and SPEC §3c
documents the same mechanism as the reason wire version 5 is unusable.

So the fix is the right reader, not a skip list:

```rust
/// Decode a vendored vector, choosing the reader by shape. A chunk SET goes
/// through `reassemble`; a single-payload string goes through
/// `decode_md1_string`, whose auto-dispatch reads bit 0 of the first symbol.
/// Using `reassemble` for both is what made 13 of 65 look like "version 2".
fn decode_vendored(chunks: &[String]) -> Result<Descriptor, md_codec::Error> {
    if chunks.len() == 1 {
        md_codec::decode::decode_md1_string(&chunks[0])
    } else {
        let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
        md_codec::chunk::reassemble(&refs)
    }
}
```

All **65** decode. `Body::Tr` coverage is **23**, not 20 — r0 would have dropped
20% of the corpus while asserting the gap was intentional.

`dump_encodings.rs` writes an object, so the population is asserted and not just
the payload. **There is no skip list**, and a decode failure is a hard error
rather than a quietly shrinking corpus:

```rust
let d = decode_vendored(&chunks)
    .unwrap_or_else(|e| panic!("{name}: decode failed: {e}"));   // never swallow
```

```json
{ "count": 65, "tr_count": 23, "vectors": [["keyed_...", "a1b2..."], ...] }
```

```bash
cargo run --quiet --example dump_encodings > crates/md-codec/tests/golden/pre_refactor_encodings.json
python3 - <<'CHK'
import json; d=json.load(open('crates/md-codec/tests/golden/pre_refactor_encodings.json'))
assert d["count"]==65,    f"vector population drifted: {d['count']}"
assert d["tr_count"]==23, f"Body::Tr coverage drifted: {d['tr_count']}"
assert len(d["vectors"])==65
print(d["count"], "vectors,", d["tr_count"], "carrying Body::Tr")
CHK
```

Both counts are asserted because each catches a different drift: `count` catches
a vector added or lost; `tr_count` catches the corpus silently losing the
coverage this stage is actually about.

- [ ] **Step 3: Capture the IDENTITY golden — also before anything moves**

r0 scheduled this inside Task 8, reached *after* Task 2 changes
`identity.rs:90`/`:200` to `d.wire_version()`. A golden captured there pins
post-change output to itself and `every_existing_v4_identity_is_byte_preserved`
becomes **a test that cannot fail**.

`examples/dump_ids.rs` emits `[[name, wallet_policy_id, template_id, phrase], …]`
over the same **65** vectors — a **4-tuple**. Any reader must destructure all
four and assert all three captured values; the combined plan's reader took a
2-tuple over a 4-tuple golden and so pinned **one of three**, silently
discarding the 12-word phrase an operator reads off steel.

```bash
cargo run --quiet --example dump_ids > crates/md-codec/tests/golden/pre_refactor_ids.json
python3 -c "import json;d=json.load(open('crates/md-codec/tests/golden/pre_refactor_ids.json'));assert d['count']==65;assert all(len(r)==4 for r in d['vectors']),'must be 4-tuples: name, policy_id, template_id, phrase';print(d['count'],'ids')"
```

- [ ] **Step 4: Commit — this commit must contain NO source changes**

```bash
git add crates/md-codec/tests/vectors/liana crates/md-codec/tests/golden \
        crates/md-codec/examples scripts/vendor-liana-evidence.sh
git commit -m "test: vendor the Liana evidence and capture pre-change goldens"
git diff HEAD~1 --stat -- crates/md-codec/src crates/md-cli/src   # MUST be empty
```

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

- [ ] **Step 1a: Define the one helper these tests use**

A plan that calls a function no task defines is the defect that cost the
combined plan three review rounds. This is the only such helper in stage 1a:

```rust
/// Load one vendored md1 chunk set from `tests/vectors/<name>.phrase.txt`.
///
/// Two shapes exist in that directory and both must be handled: most files
/// carry a `chunk-set-id:` header line that is NOT part of the payload, and
/// 13 carry none (their first line IS the md1 string). Those 13 are the
/// v0.14-era wire-version-2 vectors -- Task 0 skips them by name.
fn load_vendored_phrase(name: &str) -> Vec<String> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/vectors")
        .join(format!("{name}.phrase.txt"));
    let body = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
    body.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("chunk-set-id:"))
        .map(str::to_owned)
        .collect()
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

**RULING for `InternalKey::LianaUnspendable` at every non-encode site: it takes
the NUMS branch, everywhere, with no `todo!()` and no `unreachable!()`.**

This is the plan's one genuinely load-bearing choice, so it is made here rather
than left to the implementer. Four production sites have the *inverse* shape —
`if *is_nums { … } else { … key_index … }` — where the three rewrites above do
not apply, because the `else` arm consumes an index the new variant does not
carry:

| site | current | stage-1a rule |
| --- | --- | --- |
| `md-codec/src/render.rs:194` | `if *is_nums { NUMS_H_POINT_X_ONLY_HEX } else { render_key(…) }` | `NumsPoint \| LianaUnspendable => NUMS hex` |
| `md-codec/src/to_miniscript.rs:341` | `if *is_nums { build_nums_internal_key()? } else { lookup_key(…)? }` | `NumsPoint \| LianaUnspendable => build_nums_internal_key()?` |
| `md-codec/src/policy_shape.rs:250` | `s.key_path = if *is_nums { Nums } else { Xpub }` | `NumsPoint \| LianaUnspendable => Nums` |
| `md-cli/src/format/json.rs:353` | `is_nums: *is_nums, key_index: *key_index` | emit `is_nums: true, key_index: 0` for both |

**Why NUMS and not a panic.** Both are behaviour-preserving in 1a *because the
variant is never constructed* — which is precisely why no gate in this plan can
tell them apart, and why leaving it to the implementer means stage 1b inherits
a coin flip. A `todo!()` is a latent panic that fires the moment 1b constructs
the variant, in code paths (rendering, address derivation) that are reached with
funds on the line. Mapping to NUMS keeps stage 1a exactly neutral and makes
stage 1b's diff show each site being *deliberately* changed.

**Stage 1b changes all four.** Note them in the stage's completion so 1b's plan
starts from a list rather than a search.

**`md-cli` breaks at four production sites and they ship in THIS commit** (`format/json.rs:348`, `parse/reuse.rs:515`, `parse/template.rs:1604`, `:1629`). `seat/compose.rs:148` matches `Body::Tr { tree: Some(t), .. }` and is absorbed by the `..` — do not touch it.

`format/json.rs`'s serde shape is a **published v1 schema**: keep emitting `is_nums: bool` in stage 1a. Task 6 versions it.

- [ ] **Step 8: Build, then run the gate**

```bash
cargo build --locked --workspace
cargo nextest run --locked --workspace
```
Expected: **1400 + the new tests passed, 3 skipped** — the count rises, it does
not stay at 1400, because this task adds `internal_key_refactor`. Assert the
shape, not a stale number: 3 skipped, 0 failed, and `internal_key_refactor`
green. Any wire-byte change fails it by vector name.

Then run the **full six-command gate**, not two of it, with the pinned
toolchain on PATH (see Global Constraints) and `--all-targets` so the new
examples actually build:

```bash
export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH
cargo clippy --version    # must print 0.1.85
cargo test   --workspace --all-targets --all-features
cargo test   --workspace --doc         --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
cargo doc    --workspace --no-deps --document-private-items --all-features
cargo check  --target x86_64-unknown-freebsd -p md-cli
```

- [ ] **Step 9: Commit**

**Before staging, prove the golden was NOT regenerated.** r0 re-staged
`tests/golden/` in this commit with no check — and a golden regenerated after
the refactor pins post-change output to itself, leaving the gate green while
proving nothing. The golden was committed in Task 0 and must be byte-identical
now:

```bash
git diff --quiet HEAD -- crates/md-codec/tests/golden \
  || { echo "REFUSING: the golden changed since Task 0 — it must not be regenerated"; exit 1; }
```

```bash
git add crates/md-codec/src crates/md-cli/src crates/md-codec/tests
git commit -m "refactor: Body::Tr takes an InternalKey sum type, zero wire change

Retires the is_nums/key_index pair whose invariant was a debug_assert.
Gated by tests/internal_key_refactor.rs against a golden captured from
unmodified code: every vector encodes to identical bytes."
```

---

## Self-Review

**1. Spec coverage.** This plan covers SPEC §3f (the `InternalKey` type) and the
fixture/golden groundwork §8.5 and §8 item 1 depend on. Everything else in the
spec is stage 1b or later and is explicitly out of scope above.

**2. Placeholder scan.** One helper is used and not defined by any step:
`load_vendored_phrase(name)`. It is defined in Task 1 Step 1a below, because a
plan that references a function no task defines is the exact defect that cost
the combined plan three review rounds.

**3. Type consistency.** `InternalKey::{Slot,NumsPoint,LianaUnspendable}` and
`Body::Tr { internal_key, tree }` are used identically in Tasks 0 and 1.
`LianaUnspendable` is introduced here but **never constructed** in stage 1a —
Task 1 Step 5 makes it encode exactly as `NumsPoint` does, which is what keeps
the wire bytes identical. Stage 1b is what makes them differ.
