# R0 — IMPLEMENTATION_PLAN_f449_stage1_md_codec.md

**VERDICT: 1 Critical / 10 Important / 7 Minor / 4 Nit**

Question asked: *can a competent implementer with no context execute this plan
task-by-task and end with correct, tested code, without making a load-bearing
guess?* **No.** Task 5 is written against a rendering path that does not exist,
and eight further tasks each require at least one invention the plan does not
supply (a function, an API, a file location, a fixture, or a gate).

All line/file citations below were resolved against `descriptor-mnemonic`
at `6cbd49d8`. Two claims were settled by RUNNING code, not reading it, and
both are recorded in full (one clears a risk, one confirms a defect).

---

## C1 — Critical — Task 5, Steps 1 and 3: the keyed-descriptor render path does not exist where the plan puts it, and `collect_leaf_pubkeys_in_wire_order` cannot exist there

**Lands in:** Task 5 Step 1 (all three tests), Step 3 (the whole implementation
block), Step 4 ("Expected: PASS").

**Reproduction.**

1. `render.rs:58-61` is the complete mode enum:

   ```rust
   enum Mode {
       Literal,
       Abstract,
   }
   ```

   Step 3's match arms are `Mode::Template | Mode::Abstract` and `Mode::Keyed`.
   Neither `Mode::Template` nor `Mode::Keyed` exists. `RenderCtx`
   (`render.rs:68-78`) has nine fields — `mode`, `older_blocks`, `older_units`,
   `after_height`, `after_time`, `sha256`, `hash256`, `ripemd160`, `hash160` —
   and no `network`, so `ctx.network` in Step 3 does not resolve.

2. `grep -n "pub fn " crates/md-codec/src/render.rs` returns exactly two lines:
   `:146 descriptor_to_template` and `:152 descriptor_to_abstract_template`.
   Step 1's first test calls `md_codec::render::render_descriptor(&d,
   bitcoin::Network::Bitcoin)`, which does not exist.

3. The keyed descriptor — §4's row 1, the string an operator pastes into Liana —
   is not produced by `render.rs` at all. `cmd/descriptor.rs:198-201`:

   ```rust
   let rendered = match args.chain {
       Some(chain) => md_codec::to_miniscript_descriptor(&descriptor, chain)?.to_string(),
       None => md_codec::to_miniscript_descriptor_multipath(&descriptor)?.to_string(),
   };
   ```

   The internal-key branch is `to_miniscript.rs:333-338`, calling
   `build_nums_internal_key()` at `:361`. So §4's three rows live in **two**
   files, and Step 3 collapses them into one `match` in one file.

4. **Neither keyed entry point carries a network.**
   `to_miniscript_descriptor(d: &Descriptor, chain: u32)` (`:51`) and
   `to_miniscript_descriptor_multipath(d: &Descriptor)` (`:263`) both end in
   `node_to_descriptor(&d.tree, &keys)` (`:314`), which has no network
   parameter either. The network is baked in lower down:
   `derive.rs:57` hardcodes `network: NetworkKind::Main` inside
   `xpub_from_tlv_bytes`. Delivering §2 step 5 ("version bytes come from the
   render-time `--network` flag") therefore requires a signature change to two
   **public** md-codec functions plus every caller — work no task names, no
   Interfaces section declares, and Task 10's semver reasoning does not account
   for.

5. `collect_leaf_pubkeys_in_wire_order(node, overrides)` is used in Step 3 and
   defined by no task. In `render.rs` it is not merely missing, it is
   unimplementable: `render_with` (`:156-168`) passes `&d.tree`, `d.n`,
   `&d.use_site_path` and `d.tlv.use_site_path_overrides` — the 33-byte
   pubkeys live in `d.tlv.pubkeys`, which the renderer never receives. In
   `to_miniscript.rs` they **are** reachable (`expand_per_at_n` → `e.xpub`,
   bytes `[32..65]`), which is further evidence that Step 3 is pointed at the
   wrong file.

**Why it matters.** This is the step that emits the funds-relevant artifact.
An implementer cannot execute it; they must first redesign where §4's keyed row
lives, invent a network-threading change across two public APIs, and write the
leaf-collection walk from scratch — three load-bearing guesses, on the one task
whose output an operator pastes into a coordinator. The plan's self-review
claims "Every code step carries real code"; for Task 5 that is false.

---

## I1 — Important — Task 4, Steps 3 and 4: the derivation does not build, for two independent reasons

**Lands in:** Task 4 Step 3 (implementation), Step 4 ("Expected: **PASS**, all three").

**Reproduction (a) — the module is private.** `lib.rs:30` is `mod nums;`, not
`pub mod nums;` (compare `pub mod render;` at `:37`, `pub mod skeleton;` at
`:38`). There is no re-export of anything from `nums` in `lib.rs:49-71`; the
existing `NUMS_H_POINT_X_ONLY_HEX` is declared `pub(crate)` (`nums.rs:12`).
Task 4's Interfaces put `pub fn liana_unspendable_xpub` in `nums.rs` and Step 1
calls it from `crates/md-codec/tests/liana_unspendable.rs` — an **integration**
test, outside the crate. It cannot see a private module. The same applies to
Task 5's `pub const LIANA_UNSPENDABLE_MARKER`, which Task 6's Interfaces have
`md-cli` consuming.

**Reproduction (b) — `hex_lit` is not a dependency.** Step 3's body is

```rust
public_key: bitcoin::secp256k1::PublicKey::from_slice(
    &hex_lit::hex!("0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"),
)
```

`grep -rn "hex_lit" crates/` returns nothing. md-codec's `[dependencies]` are
`bitcoin`, `thiserror`, `bip39`, and optional `miniscript`; `hex` appears only
under `[dev-dependencies]`, so it is unavailable in `src/` too.

**Why it matters.** Step 4 asserts PASS on a step that does not compile, and the
fix for (a) is an undeclared public-API addition — exactly the kind of change
Task 10's "breaking: pre-1.0 convention makes the second component the breaking
axis" reasoning is supposed to enumerate.

---

## I2 — Important — Task 8 Step 3: `pre_refactor_ids.json` is required to predate Task 2 and is only scheduled at Task 8

**Lands in:** Task 8 Step 3, trailing sentence: *"Capture `pre_refactor_ids.json`
the same way as Task 1's golden: **from unmodified code, before Task 2**."*

**Reproduction.** Task 1's golden has a real, ordered step: Step 2 generates it
and Step 4 is the first step that edits `tree.rs`, with an explicit warning
("Generate and commit it as its own commit BEFORE Step 4"). There is **no
counterpart step anywhere in Task 1 or Task 2** for `pre_refactor_ids.json`.
The only instruction is inside Task 8 Step 3 — reached after Tasks 2 through 7
have landed and been committed. At that point `identity.rs:90` and `:200` have
already been changed from an implicit v4 to `d.wire_version()` (Task 2 Step 4).
A golden captured there pins post-change output to itself, and
`every_existing_v4_identity_is_byte_preserved` becomes a test that cannot fail.
The generator it needs (`compute_id_by_name`) is also undefined, and would
itself have to exist on the pre-Task-2 tree for the instruction to be executable.

(For precision: capturing after Task 1 but before Task 2 *is* sound — Task 1 is
byte-neutral and its own guard proves it, so the tree bits the identity sites
hash are unchanged. The requirement in the plan is correct; what is missing is
a step that executes it at the right time.)

**Why it matters.** This golden is the plan's entire answer to §8.5's "every
existing v4 id byte-preserved" — the half that protects plates already in steel.
As scheduled it reports a false PASS.

---

## I3 — Important — Task 1 Step 2: the golden's enumeration is self-contradictory, and both readings lose the `tr`/NUMS vectors

**Lands in:** Task 1 Step 2, which is the task's declared "whole gate".

**Reproduction — MEASURED, not read.** I built the example Step 2 prescribes and
ran it at `6cbd49d8`. Step 2 gives two conflicting enumerations in consecutive
paragraphs: *"enumerate `tests/vectors/*.phrase.txt`"* and *"Reuse that file's
`conformance_dir()` / `keyed_phrase_files()` helpers rather than writing new
ones."*

- `keyed_phrase_files()` (`examples/dump_skeleton_keys.rs:26-37`) filters
  `starts_with("keyed_")`. Measured: **46 files**, all reassemble and re-encode
  cleanly (`ok=46 reassemble_err=0 encode_err=0`).
- The `*.phrase.txt` glob yields **65 files**. Measured, **13 of them carry no
  `chunk-set-id:` header** — the first line *is* the md1 string:
  `nums_taproot`, `tr_keyonly`, `tr_with_leaf`, `pkh_basic`, `sh_wpkh`,
  `sh_wsh_multi`, `single_string_boundary`, `wpkh_basic`,
  `wsh_divergent_paths`, `wsh_multi_2of2`, `wsh_multi_2of3`,
  `wsh_sortedmulti`, `wsh_with_fingerprints`.

`dump_skeleton_keys.rs:62-66` is a hard `assert!(header.starts_with("chunk-set-id:"), ...)`.
Copying the helpers as instructed and enumerating as instructed therefore
**panics** on the first of those 13. Taking `keyed_phrase_files()` instead
silently covers 46 of the 65 the plan names — and the three dropped vectors
named `nums_taproot`, `tr_keyonly` and `tr_with_leaf` are precisely the
`Body::Tr` shapes Task 1 edits.

**Risk cleared while measuring, worth recording:** `encode_payload` runs
`Admission::Enforce` (`encode.rs:152-172`), and `encode.rs:114-137` documents a
measured 2026-09-19 case where admission refusals made an already-minted card
unreadable. I checked whether any vendored vector trips this. It does not:
46/46 keyed and 52/52 header-bearing vectors encode cleanly. Task 1 Step 1's
choice of `encode_payload` over `encode_payload_for_identity` is safe.

---

## I4 — Important — Task 6 Step 3: two of the three surfaces are placed where their input cannot reach them

**Lands in:** Task 6 "Files" list and Step 3, bullets 1 and 2.

**Reproduction (a) — the recogniser.** The File Structure table and Task 6's
Files both name `crates/md-cli/src/cmd/decompose.rs`. That file is 207 lines and
its only logic is `run()` reading input and dispatching on `Emit` to `println!`;
`grep -n "fn " ` on it returns `run`, `key_block`, `sh_quote`, `commands` and
nothing that sees a key. The decomposition is `crate::decompose::decompose`
at `crates/md-cli/src/decompose/mod.rs:439`, and the key walk that assigns
placeholders is `decompose/walk.rs` — `collect_occurrences` (`:175`) driven by
`desc.for_each_key`, and `Placeholders::pk` (`:205`) mapping rendered key text
to `@i`. "Recompute §2 and give the derived key no slot" has to land there, and
`for_each_key` yields keys with no structural position, so the internal key
must first be separated from the leaves — none of which the plan describes.

**Reproduction (b) — the template grammar.** Step 3 bullet 2: *"The template
grammar gains a substitution rule for `UNSPENDABLE(liana)` in `walk_tr`, before
the `NUMS_H_POINT_X_ONLY_HEX` comparison at `:1601`."* The citation is exact —
`template.rs:1601` is `if key_str == NUMS_H_POINT_X_ONLY_HEX {`. But `walk_tr`
(`:1589`) has signature

```rust
fn walk_tr(
    t: &miniscript::descriptor::Tr<DescriptorPublicKey>,
    km: &std::collections::BTreeMap<String, u8>,
) -> Result<Node, CliError>
```

and its first line is `let key_str = t.internal_key().to_string();`. A
`Tr<DescriptorPublicKey>` only exists **after** `Descriptor::<DescriptorPublicKey>::from_str`
has parsed the substituted template. `UNSPENDABLE(liana)` is not a descriptor
key expression, so the parse fails before `walk_tr` is ever entered. The textual
stage is `substitute_synthetic` (`template.rs:1047-1084`), whose regex is
`@(\d+)((?:/\d+'?)*)(?:/<[0-9;]+>)?(?:/\*(?:'|h)?)?` — `@i` only. The plan
quotes that very function two bullets later, in the argument for why `md encode`
must refuse, and still places the marker rule downstream of it.

(Bullet 3, the `md encode` refusal, **is** implementable where implied: the
`map_err` closure at `template.rs:1610-1625` is exactly where
`lookup_key`'s "synthetic key … not found in key map" leak originates.)

---

## I5 — Important — Task 6 Step 1: the fourth test depends on `md compose --unspendable liana`, which is stage-2 work and does not exist

**Lands in:** Task 6 Step 1, `the_marker_round_trips_through_compose_then_encode`;
Task 6 Step 5, "Expected: **PASS**".

**Reproduction.** `grep -rn "unspendable" crates/md-cli/src/` returns only
`compile.rs:48-95` — `--unspendable-key`, a Tap-context hint forwarded to
miniscript's `compile_tr`, unrelated. There is no `--unspendable` on
`md compose`. SPEC §9 assigns it to **stage 2**: *"`md compose --unspendable
liana|nums` (default `nums`)"*. The plan's own self-review does not claim stage
2 and lists no compose work. So Task 6 Step 5's PASS is unreachable inside this
plan's scope, and the implementer's only exits are to build a stage-2 flag
unbriefed or to delete a gate.

---

## I6 — Important — Tasks 3 and 7: two refusal gates call APIs no task produces, and the plan's own design makes them unconstructible

**Lands in:** Task 3 Step 1 test 2; Task 7 Step 1 test 4; Task 7 Step 3.

**Reproduction (a) — `decode_payload_restricted(&bytes, &[4])`.** Undefined; no
task's Interfaces produces it. The comment calls it "Simulate the shipped
decoder: only version 4 supported" — but after Task 2 Step 3 the decoder
accepts `{4, 8}` by construction (`is_supported_version`), so the only way to
observe a v4-only refusal is a knob the plan never creates. SPEC §8.4's
"v8 payload refused by a v4 decoder" is really a statement about *older
toolchains* (§6a, stages 2/3/4a); at stage 1b the honest assertion is the one
Task 2 Step 1 already makes.

**Reproduction (b) — `encode_payload_at_version(&d, 8)`.** Undefined, and
unconstructible under Task 2: `encode_payload_inner` derives the version from
`d.wire_version()`, so no encoder entry point accepts a version argument and
`Error::NonMinimalWireVersion` has no caller that can reach the refused state.
By the spec's own rule — §6's "A refusal that cannot fire is not a guard … its
test passes vacuously" — this is either a new public API the plan must name or
a vacuous gate.

**Reproduction (c) — where do the refusals hook?** Task 7 Step 3 is one
sentence: *"Implement the four refusals and their `Error` variants in
`error.rs`."* Three of the four tests call `validate(&…)`, which does not exist;
`validate.rs` exposes eleven `validate_*` functions, called from two distinct
places with **different policy**: `encode_payload_inner` (`encode.rs:152-172`,
two of them gated behind `Admission::Enforce`) and `decode_payload_with_opts`
(`decode.rs:120-150`). The choice is load-bearing and the crate documents why at
length (`encode.rs:114-137`): mint-time refusals reaching the decode path
through `chunk::reassemble` → `compute_md1_encoding_id` made an
already-engraved 2-of-2 unreadable, measured 2026-09-19. A §6 refusal hooked
unconditionally repeats that class on kind-1 plates. The plan leaves the
implementer to guess.

---

## I7 — Important — Task 10 Step 3: "all four commands CI runs" is false; CI runs six, and `cargo fmt --all --check` appears nowhere in the plan

**Lands in:** Task 10 Step 3, including its own justifying sentence
(*"a `cargo doc` failure reached main once in this repo because a local gate ran
three commands while CI ran four"*).

**Reproduction.** `.github/workflows/ci.yml`:

| line | command |
| --- | --- |
| `:48` | `cargo test --workspace --all-targets --all-features` |
| `:49` | `cargo test --workspace --doc --all-features` |
| `:65` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| `:77` | `cargo fmt --all --check` |
| `:93` | `cargo doc --workspace --no-deps --document-private-items --all-features` |
| `:124` | `cargo check --target x86_64-unknown-freebsd -p md-cli` |

Task 10 Step 3 lists four. `cargo fmt --all --check` is absent from the whole
plan — after a mechanical rename the plan itself sizes at 47 + 12 production
sites plus 66 test sites. `cargo nextest` does not run doctests, so `:49` has no
local counterpart either, and the plan's clippy/doc invocations drop
`--all-features` and `--document-private-items`. Separately, clippy, doc and fmt
run **only at Task 10** — nine commits after the rename — while
`[workspace.lints] missing_docs = "warn"` (root `Cargo.toml:11`) plus
`-D warnings` makes an undocumented `InternalKey` variant a hard CI failure, and
the baseline commit `6cbd49d8` is itself a `cargo doc` CI fix, so that gate is
demonstrably live on this tree.

---

## I8 — Important — spec coverage: §6a's stage-1b row and §4b are owned by nobody

**Lands in:** the Self-Review's "Spec coverage" and "Deliberately NOT in this
plan" paragraphs.

**Reproduction (a) — §6a's 1b row.** The self-review says *"§6a's three messages
(stages 1b/2/3 — the `WireVersionMismatch` Display is the only 1b piece and
rides Task 2)."* Task 2's Files list is `header.rs`, `tree.rs`, `encode.rs`,
`decode.rs`, `chunk.rs`, `identity.rs`, plus `tests/wire_version_8.rs`.
`error.rs` is not among them and no step edits it. The live text is
`error.rs:33`:

```rust
#[error("wire-format version mismatch: got {got}, expected 4")]
```

which becomes false at Task 2 Step 3. SPEC §8.9 assigns this row a **test** at
stage **1b** ("`WireVersionMismatch`'s message names the accepted set, not
'expected 4'"); no task writes one. "Rides Task 2" is a claim, not a step.

**Reproduction (b) — §4b.** The self-review's covered list is §2, §3a/§3c/§3d,
§3e, §3f, §4, §4a, §5, §6, §8, §9. The deferred list is §6a, §7/§7a, §8.3,
§8.4, §8.8, §8.9, §8b/§9a. **§4b is in neither.** §4b requires retiring
`DESIGN_coordinator_compatibility.md:155-158` and `policy_shape.rs:119-133`
*"in the same change"*. That text is live and reads, verbatim at
`crates/md-codec/src/policy_shape.rs:130-132`: *"Do not 'restore fidelity' with
the Go name here: a coordinator that needs the finer distinction computes it
itself, one layer above this type."* — i.e. the source instructs the next reader
against exactly what Task 6 implements.

---

## I9 — Important — spec coverage: §8.1 and §8.6, both owned by stage 1b, are under-delivered

**Lands in:** Task 4 Steps 1 and 3.

**Reproduction — §8.1.** *"md-codec derives the §2 xpub for **all eight**
evidence shapes, byte-identical. Plus **new vectors** for the three unmeasured
gaps in §2: a `tpub` wallet, a nested taptree that Liana ACCEPTS, and — if §6
did not refuse it — a `sortedmulti_a` leaf."* Measured,
`fable-liana-parse-in.jsonl` carries exactly 8 rows at variant
`liana-unspendable-xpub`: `preset-kofn-recovery-tr`, `preset-tiered-recovery-tr`,
`preset-hashlock-gated-tr`, `preset-decaying-multisig-tr`,
`hashlock-gated-tr-hash160`, `same-seed-two-paths-tr`,
`X19-tr-kofn-nums-older5`, `X20-tr-hashlock-known`. Task 4 Step 1 draws
`GOLDEN_CASES` from *"the four shapes Liana accepted"*, and every call in all
three tests passes `bitcoin::Network::Bitcoin`. No task derives at a
testnet/signet/regtest network — and §2 itself flags the `tpub` branch of step 5
as *"transcribed, not measured"*, i.e. the one leg with no evidence behind it.

**Reproduction — §8.6.** *"All three of `kofn-recovery`, `tiered-recovery` and
`decaying-multisig` over the same four keys derive one internal key — the third
is the nested case."* Measured from the evidence: all three carry the identical
internal key `xpub661MyMwAqRbcFswVugWF…` and the identical leaf order (last six
chars: `XCJ52i, 5gx5VH, fqe2nL, 6nNQe1`). Task 4's structure test asserts only
kofn vs tiered and cannot reach the third: `preset-decaying-multisig-tr` has
`ok=false` in `fable-liana-parse-out-v15.jsonl`, so it is not in
`GOLDEN_ACCEPTED`. It is also the **only** nested-taptree row in the whole
evidence set, which makes it the sole Rust-side check on the left-first DFS leaf
ordering §2 calls "the single line a port implementer would otherwise get
wrong". Dropping it removes the pin the spec calls "the stronger" one.

---

## I10 — Important — Tasks 4 and 8: the golden fixture lives in a different repository, and the reproducibility claim is false

**Lands in:** Task 4 Step 1, Task 8 Step 2, and the Self-Review's closing
"One known gap" paragraph.

**Reproduction.** Task 4 Step 1: *"The four shapes Liana accepted are in
`design/evidence/composer-fable-r0/fable-liana-parse-in.jsonl`"*; Task 8 Step 2:
*"`case.liana_receive` / `liana_change` come from `fable-liana-parse-out-v15.jsonl`"*.
Both are bare relative paths. Measured:

```
/scratch/code/shibboleth/mnemonic-engrave/design/evidence/composer-fable-r0/
  fable-liana-parse-in.jsonl       171259 bytes
  fable-liana-parse-out-v15.jsonl  193772 bytes
```

`/scratch/code/shibboleth/descriptor-mnemonic/design/evidence/` **does not
exist**. The code under test is in `descriptor-mnemonic`. No step vendors the
evidence. The Self-Review then claims the extraction is *"reproducible rather
than transcribed"* because the generator is committed alongside — a generator
committed in `descriptor-mnemonic` cannot read a file in `mnemonic-engrave`, and
CI checks out one repository.

(Recorded as useful, since I had to open the files anyway: the data Task 8
needs **is** there. `fable-liana-parse-out-v15.jsonl` rows carry `receive` and
`change` arrays of three addresses each and a `liana_desc` that includes the
BIP-380 checksum; the four `ok=true` unspendable shapes are
`preset-kofn-recovery-tr`, `preset-tiered-recovery-tr`, `same-seed-two-paths-tr`,
`X19-tr-kofn-nums-older5`. Task 5 Step 1's literal
`assert!(s.starts_with("tr(xpub661MyMwAqRbcFswVugWF"))` matches the kofn/tiered
golden exactly. The evidence does **not** carry leaf pubkeys as 33-byte hex or
the internal xpub as its own field, so "mechanical extraction" means base58
decoding each leaf and parsing the internal key out of `desc` — more than the
word "mechanical" implies.)

---

## Minor

**M1 — Task 2 Step 4 mis-describes `chunk.rs:373`.** The step says *"`chunk.rs:70`
and `:373` accept `{4, 8}`"*. `:70` is the gate
(`if version != Header::WF_REDESIGN_VERSION { return Err(WireVersionMismatch) }`
inside `ChunkHeader::read`). `:373` is `let expected_version = h0.version;`, feeding
an inter-chunk **consistency** check — it compares chunks to each other and
accepts whatever `:70` let through. Only one site is a version gate.

**M2 — Task 2 does not say where `Descriptor::wire_version` goes.** `Descriptor`
is declared in `encode.rs:16`, not `tree.rs`; Task 2's Files list names
`tree.rs:79`/`:196` for signatures and `encode.rs:174`/`:181` for call sites, and
never places the new `impl`. (The body itself is sound: `Body` has exactly nine
variants, the or-pattern `Body::Children(cs) | Body::Variable { children: cs, .. }`
binds `cs` consistently and compiles, `Body::MultiKeys` holds `indices: Vec<u8>`
rather than nodes so `_ => false` drops no recursion, and the walk terminates on
a finite tree. `Header` is already in scope in `encode.rs`.)

**M3 — Task 6 Step 4's schema bump is unsized.** "bump the schema version" is one
sentence. `grep -rn "md-cli/1"` over the repo returns **66 occurrences**,
including `format/json.rs:6` (the constant), `:94` (a test asserting it),
`cmd/compose.rs`, six `tests/*.rs`, and ~20 `tests/snapshots/*.snap` files. A
plan that measured Task 1's blast radius to the site leaves a comparable one at
one sentence, and a blanket snapshot regeneration lands in the same commit as
the `Body::Tr` shape change. The Files list also says to modify
`docs/json-schema-v1.md` while bumping past v1.

**M4 — Task 4 Step 5's md-cli test is an either/or and is staged by neither
commit.** Step 5 says to put it in *"`crates/md-cli/tests/liana_input_side.rs`
(created in Task 6) **or** a small `crates/md-cli/tests/nums_invariants.rs`"*.
Task 4 Step 6 stages only `crates/md-codec/src/nums.rs` and
`crates/md-codec/tests/liana_unspendable.rs`. Under the second choice the file
is never staged by any task — Task 10's `git add -u` stages tracked files only —
so the §6 invariant pin silently never ships.

**M5 — Task 1 Step 7's migration recipe omits the mutable form.** It gives
`if !*is_nums` → `if let InternalKey::Slot(i) = internal_key`. `canonicalize.rs:107-116`
needs `&mut` rewriting (`*key_index = perm[*key_index as usize]`), as does the
`remap_indices` walk generally; `canonicalize.rs` holds 11 of the 47 md-codec
sites.

**M6 — Task 3 Step 1's first test does not compile.** `encode_payload` returns
`Result<(Vec<u8>, usize), Error>` (`encode.rs:100`), so `let bytes =
encode_payload(d).expect("encode")` binds a tuple; `decode_payload(&bytes)` then
fails twice — a `&(Vec<u8>, usize)` where `&[u8]` is wanted, and one argument
where `decode_payload(bytes: &[u8], total_bits: usize)` (`decode.rs:66`) takes
two. Task 1 Step 1's test destructures correctly, so the plan is inconsistent
with itself.

**M7 — md-cli's own version is not bumped.** Task 10 changes md-codec `0.45.1 →
0.46.0` and md-cli's **pin** at `crates/md-cli/Cargo.toml:28`, but leaves
`crates/md-cli/Cargo.toml:3 version = "0.17.0"` untouched, although md-cli's
published JSON schema breaks (Task 6 Step 4) and its CLI behaviour changes.
`CHANGELOG.md` is per-crate (`## md-codec [0.45.1] — 2026-09-20`), so the
md-cli break has no entry either. (The `sed` pair itself is correct and
sufficient for resolution: `grep -rn "0\.45\.1"` outside `vendor/` hits only
`Cargo.lock:513`, `CHANGELOG.md:7`, and the two manifest lines.)

---

## Nit

**N1** — Task 2 Step 1's `for divergent in [false, true]` loop cannot change the
assertion: `byte0 >> 3 & 1` depends only on bit 0 of the version field, and
`divergent` occupies bit 4. Both iterations assert the same thing.

**N2** — Tasks 2, 3, 4 and 6 each say "Run to verify they fail / Expected: FAIL"
where the named cause is a missing symbol. `cargo nextest` reports a **build
failure**, not a red test; a red-then-green transcript will not exist for those
steps.

**N3** — Task 5 Step 1's third test passes a `Descriptor` to
`md_codec::skeleton::skeleton_key`, whose signature is
`pub fn skeleton_key(s: &Skeleton) -> SkeletonKey` (`skeleton.rs:312`). The
crate's route is `skeleton(&d)?` (`:185`) then `skeleton_key(&s)`, as
`examples/dump_skeleton_keys.rs` does.

**N4** — the four md-cli site citations each land one line above the `is_nums`
token (plan `json.rs:348` / actual `:349`; `reuse.rs:515` / `:516`;
`template.rs:1604` / `:1605`; `:1629` / `:1630`). Consistent and harmless — they
name the `Body::Tr {` opener. The claim that `seat/compose.rs:148` is absorbed
by `..` is **correct**: it matches `Body::Tr { tree: Some(t), .. }`.

---

## Per-task soundness

| task | verdict |
| --- | --- |
| **1** — `InternalKey` refactor | Sound in design and in its production-site enumeration (`compose/tr.rs:45` is `is_nums: ik.is_none(), key_index: 0`, so the prescribed mapping is right; the md-cli four are right; `seat/compose.rs:148` is correctly cleared). The gate's enumeration is broken — I3 — and the migration recipe is incomplete — M5. |
| **2** — version 8 and threading | **Sound on the two questions asked.** `Descriptor::wire_version()` as written compiles and terminates (M2 covers only *where* it lands). The caller enumeration is **complete and verified independently**: `write_node` has exactly three non-test callers — `encode.rs:181`, `identity.rs:90`, `identity.rs:200` — the fourth hit, `identity.rs:626`, is inside the `#[cfg(test)]` module that begins at `identity.rs:311`; `read_node` has exactly one, `decode.rs:89`; md-cli calls neither (its only hit is a doc comment at `parse/template.rs:1219`). Defects: M1, M2, and the §6a Display it claims to carry but does not — I8(a). |
| **3** — the kind bit | **The encoding is sound and cannot desync.** Write and read mirror exactly at both versions; a v4 reader meeting v8 fails closed at `header.rs:45-47` before any tree bit is consumed, and a v8 reader meeting v4 reads no kind bit. Version 8 is even, so `decode_md1_string`'s bit-3 dispatch (`decode.rs:186-189`) routes it single-payload, confirmed by computation. Defects are in the tests only: I6(a), M6. |
| **4** — the §2 derivation | Recipe and property tests are well chosen, and the structure-independence claim they rest on is true (measured). Does not build — I1. Under-delivers §8.1/§8.6 — I9. Fixture is in the wrong repo — I10. Step 5's test is orphaned — M4. |
| **5** — rendering | **Not executable.** C1. |
| **6** — input side and JSON | Bullet 3 (the `md encode` refusal) is sound and lands where implied. Bullets 1 and 2 are pointed at stages that cannot see their input — I4. Step 1's fourth test is out of scope — I5. Step 4 is unsized — M3. |
| **7** — §6 refusals | The four refusals are the right four. Step 3 names neither the hook point nor the API the fourth needs, and the hook point has a documented past regression attached — I6(b)(c). |
| **8** — §8 vectors | Steps 1 and 2 are well-specified and the data behind them exists. Step 3's preservation golden is scheduled after the change it must predate — I2. Fixture repo — I10. |
| **9** — mutation pass | **Sound.** Nine mutations, each mapped to a named gate, with the "assert the mutation APPLIED" step and a revert check. Note it inherits the gaps above: mutation 6 is credited to "Task 8 identity distinctness", which survives I2 intact, but mutation 5 ("render kind 1 as `NUMS_H_POINT_X_ONLY_HEX`") is credited to Task 5, which does not exist yet. |
| **10** — version bump | The single-commit argument is **correct**: `md-cli/Cargo.toml:28` is `md-codec = { path = "../md-codec", version = "=0.45.1" }`, so cargo fails resolution before rustc runs if the two move apart; 0.46.0 is right for a pre-1.0 breaking change; both `sed` patterns match exactly one line each and nothing else in-tree pins 0.45.1. The gate is not CI's — I7. md-cli's own version and changelog are missed — M7. |
