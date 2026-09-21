# R1 — IMPLEMENTATION_PLAN_f449_stage1_md_codec.md (scoped fold re-review)

**VERDICT: 0 Critical / 6 Important / 16 Minor**

Scope: the two questions asked. No fresh audit of plan or spec. All citations
resolved against `descriptor-mnemonic` at `6cbd49d8` and the plan at
`f50cf015`. Three claims were settled by RUNNING code, not reading it, and all
three are recorded in full — one of them overturns a premise r0 asserted.

**Ready for implementation: NO.** Task 0 Step 2 — the plan's very first
executable step — cannot succeed as written, and two r0 Importants (I4b, I6c)
are unchanged in the fold.

---

## Q1 — did the fold address each finding?

| # | verdict | note |
| --- | --- | --- |
| **C1** | **ADDRESSED** | Task 5 is rewritten against the real tree: `Mode::Keyed`, `render_descriptor()`, `ctx.network` and `collect_leaf_pubkeys_in_wire_order` are all gone; §4's three rows are correctly split across `render.rs` (rows 2-3) and `to_miniscript.rs` (row 1); the network problem is named and given two `_with_network` entry points. **New defects in the replacement — see Q2(b), IMP-2 and MIN-11.** |
| **I1(a)** | **ADDRESSED** | Task 4 Step 3 is new and explicit: `lib.rs:30` `mod nums;` → `pub mod nums;`, both new items `pub`. |
| **I1(b)** | **PARTIAL** | Step 3's prose forbids `hex_lit`; Step 4's code block (plan `:623`) still calls `hex_lit::hex!(…)`. Two adjacent steps contradict. → MIN-1. |
| **I2** | **ADDRESSED** | `pre_refactor_ids.json` moved to Task 0 Step 3 (before any source change); Task 8 Step 3 now says "Do NOT regenerate it here". Scheduling is fixed. **But the golden's shape and the test's shape disagree — IMP-6**, and `compute_id_by_name` is still undefined. |
| **I3** | **PARTIAL** | Task 0 Step 2 replaces the enumeration with "all 65, both header shapes". Task 1 Step 2 **still** carries the superseded "Reuse that file's `conformance_dir()` / `keyed_phrase_files()` helpers" and re-runs the generation Task 0 already committed. → MIN-2. Separately the *replacement* enumeration does not work — IMP-1. |
| **I4(a)** | **ADDRESSED** | Task 6's Files now name `decompose/mod.rs:439` and `decompose/walk.rs` (`collect_occurrences:175`, `Placeholders::pk:202` — corrected), explicitly NOT `cmd/decompose.rs`, plus a "Note on the walk" that the internal key must be separated at the `Tr` node because `for_each_key` has no structural position. |
| **I4(b)** | **NOT ADDRESSED** | Step 3 bullet 2 is byte-identical to r0's: the marker rule is still placed "in `walk_tr`, before the `NUMS_H_POINT_X_ONLY_HEX` comparison at `:1601`". → **IMP-3**. |
| **I5** | **ADDRESSED** | Task 6 Step 1's fourth test no longer uses `md compose --unspendable liana`; it asserts the grammar directly via `md encode "tr(UNSPENDABLE(liana),{pk(@0/<0;1>/*),pk(@1/<0;1>/*)})"`, with a comment saying why. (Its PASS still depends on IMP-3.) |
| **I6(a)** | **ADDRESSED** | `decode_payload_restricted` is deleted and replaced by a comment explaining that no v4-only decoder exists in-crate after Task 2 and that §8.4 is about older toolchains. |
| **I6(b)** | **ADDRESSED** | `encode_payload_at_version` is deleted; the minimum-version rule is re-expressed as a property of `wire_version()` ("unconstructible rather than merely refused"). Sound. |
| **I6(c)** | **NOT ADDRESSED** | Task 7 Step 3 is still the single sentence "Implement the four refusals and their `Error` variants in `error.rs`." Three tests still call `validate(&…)`. → **IMP-4**. |
| **I7** | **PARTIAL** | Step 3 now lists all **six** CI commands with the right flags (`--all-features`, the separate `--doc` run, `--document-private-items`, `cargo fmt --all --check`, the FreeBSD cross-check). The second half of the finding is untouched: clippy/fmt/doc still run **only** at Task 10, nine commits after the rename, with `missing_docs = "warn"` + `-D warnings` live. → MIN-3. |
| **I8(a)** | **ADDRESSED** | Task 2 Step 5b is new: the `WireVersionMismatch` Display and an asserting test. See Q2(d). Files list still omits `error.rs` → MIN-15. |
| **I8(b)** | **ADDRESSED** | Task 8b is new and owns §4b. See Q2(c). |
| **I9** | **ADDRESSED** | Task 8 Step 2b adds the three §8.1 gap vectors (`tpub`, nested taptree, and the discharge-by-refusal for `sortedmulti_a`); Step 2c adds §8.6's three-preset pin including `preset-decaying-multisig-tr`. Task 0 Step 1 vendors all **eight** records with an `accepted` field, so the refused nested case is reachable. Correct. |
| **I10** | **ADDRESSED** | Task 0 vendors the evidence into `descriptor-mnemonic` via a committed `scripts/vendor-liana-evidence.sh` recording the source SHA. **The vendored shape is insufficient for two consumers — IMP-5**, and the Self-Review's closing paragraph still describes the superseded arrangement (MIN-16). |
| **M1** | **NOT ADDRESSED** | Plan `:400` still reads "`chunk.rs:70` and `:373` accept `{4, 8}`". `:373` is `let expected_version = h0.version;`, an inter-chunk consistency check, not a version gate. |
| **M2** | **NOT ADDRESSED** | Task 2 Step 4 gives `impl Descriptor { … }` (plan `:369`) and never says which file. `Descriptor` is declared at `encode.rs:16`; the Files list names `tree.rs`, `encode.rs:174`/`:181`, and no location for the new `impl`. |
| **M3** | **NOT ADDRESSED** | Step 4 is still one paragraph ("bump the schema version"). The 66 `md-cli/1` occurrences, the ~20 snapshot files, and the contradiction of editing `docs/json-schema-v1.md` while bumping past v1 are all unchanged. |
| **M4** | **NOT ADDRESSED** | Task 4 Step 5 is still "`liana_input_side.rs` (created in Task 6) **or** … `nums_invariants.rs`", and Step 6 still stages only `nums.rs` + `tests/liana_unspendable.rs`. |
| **M5** | **NOT ADDRESSED** | Step 7's recipe is still `if !*is_nums` → `if let InternalKey::Slot(i)`. `grep -n canonicalize` over the plan returns **nothing**; the `&mut` rewriting form (`*key_index = perm[*key_index as usize]`) is absent. |
| **M6** | **NOT ADDRESSED** | Task 3 Step 1 still binds `let bytes = encode_payload(d).expect("encode")` (returns `(Vec<u8>, usize)`, `encode.rs:99`) and calls `decode_payload(&bytes)` (takes `(&[u8], usize)`, `decode.rs:66`). |
| **M7** | **NOT ADDRESSED** | `grep -n "0\.17\.0"` over the plan returns nothing; Task 10 still bumps only md-codec's version and md-cli's pin, and CHANGELOG is per-crate. |
| **N1** | **NOT ADDRESSED** | Task 2 Step 1's `for divergent in [false, true]` still asserts the same thing twice. |
| **N2** | **NOT ADDRESSED** | Tasks 2/3/4/6 still say "Expected: FAIL" where the cause is a missing symbol (a build failure, not a red test). |
| **N3** | **NOT ADDRESSED** | Task 5 Step 2's last test still calls `md_codec::skeleton::skeleton_key(&descriptor_at_nums_kind()).unwrap()`. Measured: `skeleton.rs:312` is `pub fn skeleton_key(s: &Skeleton) -> SkeletonKey` — wrong argument type **and** not a `Result`, so `.unwrap()` also fails. |
| **N4** | **NOT ADDRESSED** | `format/json.rs:348`, `parse/reuse.rs:515`, `parse/template.rs:1604`, `:1629` are unchanged (all one line above the `is_nums` token). Harmless, as r0 said. |

Tally: 11 ADDRESSED, 4 PARTIAL, 11 NOT ADDRESSED (2 of those Important,
7 Minor, 4 Nit — r0's own severities preserved except where the fold's new
content changed the picture).

---

## Q2 — is the NEW content sound?

### (a) Task 0 — vendoring, and both goldens before any source change

**Ordering: safe in principle, and one axis I checked explicitly clears.**
Steps 2 and 3 both capture from unmodified code and Step 4 commits with
`git diff HEAD~1 --stat -- crates/md-codec/src crates/md-cli/src` asserted
empty. That is the right shape, and capturing the identity golden *before*
Task 1 (rather than between Tasks 1 and 2) is strictly stronger than r0 asked
for, since Task 1 is byte-neutral.

I checked whether Step 3 would itself force a source change to be reachable —
`examples/` are external crates, so private items would have blocked it. They
do not: `lib.rs:29` is `pub mod identity;` and `compute_wallet_policy_id`
(`identity.rs:191`), `compute_wallet_descriptor_template_id` (`:76`) and
`to_phrase` (`:141`) are all `pub`. `serde_json` and `hex` are both in
md-codec's `[dev-dependencies]`, which examples inherit. `cargo run --quiet
--example …` from the workspace root resolves despite the virtual manifest —
verified by running the existing `dump_skeleton_keys`. So Task 0 has no
hidden prerequisite. **The ordering is sound.**

**But the enumeration it prescribes cannot execute. → IMP-1.**

**`cases.json`'s shape: sufficient for Task 4 and Task 8 Steps 2b/2c, NOT for
Task 5 Step 2 or Task 8 Steps 1-2. → IMP-5.** Task 7 does not consume it.

### (b) Task 5's rewrite — the highest-risk item

Four sub-questions, four separate answers.

**1. Does the refusal break an existing caller or test? NO — and this is the
half the fold got right.** Measured: `to_miniscript_descriptor` /
`to_miniscript_descriptor_multipath` have **36 call-expression lines** outside
their definitions (58 identifier occurrences repo-wide). Every md-cli `seat`
call site — `matching.rs:304,335,454`, `disposition.rs:308,434,437`,
`compose.rs:403,406` — is inside a `#[cfg(test)]` module (module starts at
`matching.rs:264`, `disposition.rs:162`, `compose.rs:300`). More decisively,
`wire_version() == 8` is **unreachable for every pre-existing descriptor**,
because `InternalKey::LianaUnspendable` does not exist until Task 1 creates it.
So no existing caller and no existing test can reach the new `Err`. The plan's
claim "zero of the 55 existing sites change" is right in substance.

**2. But it breaks the ONLY production path to an address. → IMP-2.**

**3. Are the leaf miniscripts available in tap-tree order at that point? NOT AT
THAT POINT — after a reorder, yes; and the prescribed signature is not the real
shape. → MIN-11.** `node_to_descriptor`'s `Tr` arm (`to_miniscript.rs:335-352`)
builds `internal_key` at `:341-345` **before** `script_tree` at `:346-350`, so
at plan-cited `:333-338` there are no built leaves to pass. The data *is*
reachable after swapping the two `let`s: `TapTree::leaves()`
(`descriptor/tr/taptree.rs:56`) yields `TapTreeIterItem` in left-first DFS
order — its own doc example gives `(2,A),(2,B),(2,C),(3,D),(3,E)` — which is
exactly §2's "descriptor left-to-right". The item holds `&'tr Arc<Miniscript<Pk,
Tap>>` and a `u8` depth, not the `&[(u8, Miniscript<DescriptorPublicKey, Tap>)]`
the plan's signature declares.

**4. Does `for_each_key` yield what the recipe needs? YES, semantically —
with one uncompilable detail.** `impl ForEachKey for Miniscript`
(`src/miniscript/mod.rs:665-690`) walks `pre_order_iter()` and calls the
predicate on `PkK`, `PkH`, and each member of `Multi`/`SortedMulti`/`MultiA`/
`SortedMultiA` thresholds **in order** — left-to-right within the leaf, which
is what §2 requires. `compressed_33` over a `MultiXPub`/`XPub` as
`k.xkey.public_key.serialize()` is also semantically right: `assemble_origin_
and_xkey` builds the xkey from `xpub_from_tlv_bytes(e.xpub)`, so
`.xkey.public_key` IS the TLV entry's bytes `[32..65]`, unchanged by the
multipath alt (which lands in `derivation_path`). Both entry points therefore
derive the *same* internal key for receive and change, which the design
requires. The uncompilable detail: the plan says "return `Err(failed(...))`
naming it rather than panicking" from inside the `for_each_key` closure, whose
return type is `bool`. → MIN-11.

### (c) Task 8b — retiring the `policy_shape.rs:119-133` ruling

**Largely sound, and its gate is a real one.** I ran Step 3's check:
`grep -rn -A1 'Do not "restore' crates/` returns exactly one hit today —
`policy_shape.rs:130-131` — so unlike r0's contiguous `"restore fidelity"`
grep it *can* fail, and the plan's parenthetical about why is accurate. Step 2
correctly refuses to edit the other repo from this worktree.

Two gaps, both Minor:

- **The reversal is prose-only while the type still collapses the kinds.**
  `policy_shape.rs:250-254` is `if *is_nums { KeyPathKind::Nums } else {
  KeyPathKind::Xpub }`. Task 1's behaviour-preserving rewrite sends **both**
  `NumsPoint` and `LianaUnspendable` to `KeyPathKind::Nums`. Step 1 tells the
  implementer to "say what is now true: the distinction IS codec-observable as
  of wire version 8" — which, written plainly, would be a second wrong
  instruction, since this type still cannot express it. The rewrite must say
  *observable on the wire, not yet surfaced in this type, see §7/§7a (stage 3)*.
- **Task 1 falsifies a second block Task 8b does not name.**
  `policy_shape.rs:117-119` documents `KeyPathKind::Nums` as "The wire's only
  internal-key discriminant (`crate::tree::Body::Tr::is_nums`) says exactly this
  and nothing more" — both the citation and the claim die with Task 1. Not an
  intra-doc link (backticks, not brackets), so rustdoc stays silent. → MIN-14.

Step 2's cross-repo edit has no gate — Step 3's grep covers `crates/` only —
but Step 2 does state the edit as an imperative and requires it be noted in the
stage's completion. Acceptable; not raised.

### (d) Task 2 Step 5b — the `WireVersionMismatch` Display change and its test

**Sound, and I checked the blast radius the step does not mention.** Measured:
`grep -rn "wire-format version" crates/ docs/` returns exactly two lines —
`error.rs:33` (the attribute itself) and `cmd_decode.rs:80` (a doc comment
quoting only the `got 9` prefix, which survives). No test, no snapshot and no
doc asserts the string `expected 4`; `grep -rln "wire-format version"
crates/md-cli/tests/snapshots/` is empty. So the Display change breaks nothing.
The variant's field is `pub`, so `Error::WireVersionMismatch { got: 9 }` is
constructible from an integration test.

Two small things, both Minor (MIN-15): the step forgets to add `error.rs` to
Task 2's Files list (the commit stages `crates/md-codec/src` wholesale, so
nothing is actually lost), and `assert!(!s.contains("expected 4"))` rejects
`"…got 9, expected 4 or 8"` — a correct rendering that names the accepted set.
The assertion's stated purpose ("stale single-version claim") is not what it
measures.

---

## Important findings (blocking)

### IMP-1 — Task 0 Steps 2 and 3: the "all 65" enumeration cannot execute. 13 of the 65 vectors are wire version **2**, not a different header shape.

**Lands in:** Task 0 Step 2 (`assert len(d)==65`), Task 0 Step 3 (same 65),
and by dependency Task 1's entire gate and Task 8 Step 3's preservation golden.

**Reproduction — MEASURED.** I built the enumeration Step 2 prescribes
(all `tests/vectors/*.phrase.txt`, handling both header shapes: header-bearing
files skip line 1, header-less files use every line) and ran `reassemble` then
`encode_payload` on each, at `6cbd49d8`:

```
REASSEMBLE_ERR nums_taproot: wire-format version mismatch: got 2, expected 4
REASSEMBLE_ERR pkh_basic: wire-format version mismatch: got 2, expected 4
REASSEMBLE_ERR sh_wpkh: … sh_wsh_multi … single_string_boundary …
REASSEMBLE_ERR tr_keyonly … tr_with_leaf … wpkh_basic … wsh_divergent_paths …
REASSEMBLE_ERR wsh_multi_2of2 … wsh_multi_2of3 … wsh_sortedmulti …
REASSEMBLE_ERR wsh_with_fingerprints: wire-format version mismatch: got 2, expected 4
total=65 noheader=13 ok=52 reassemble_err=13 encode_err=0
```

The 13 header-less files are exactly the 13 that fail, and they fail at
`header.rs:43` / `chunk.rs:71` because their payloads are **v0.14-era wire
version 2**, which the shipped decoder does not support at all. Step 2's
diagnosis — that they differ only in "header shape" — is wrong, and its own
`assert len(d)==65` therefore fails on the first run. Step 3 halts identically.
An implementer's only exits are to invent a skip rule the plan does not
authorise, or to rewrite the assertion to whatever number comes out — on the
step that produces the artifact Task 1 calls "the whole gate for this task".

**A premise from r0 is also overturned, and it matters for how this is fixed.**
r0 wrote that the dropped vectors "`nums_taproot`, `tr_keyonly` and
`tr_with_leaf` are precisely the `Body::Tr` shapes Task 1 edits". Measured over
the 52 decodable vectors, **20 carry a `Body::Tr` node**, 18 of them `keyed_*`:
`keyed_compose_preset_kofn_recovery`, `keyed_compose_tr_extracted_first`,
`keyed_compose_tr_extracted_later_four_paths`, `keyed_compose_tr_hash_leaf`,
`keyed_compose_tr_key_path_only`, `keyed_compose_tr_nums_three_leaves`,
`keyed_compose_tr_sole_sortedmulti_a`,
`keyed_compose_tr_three_paths_extracted_later`,
`keyed_compose_tr_two_path_distinct_fingerprints`,
`keyed_compose_tr_two_path_nums`, `keyed_compose_tr_unsorted_sole_leaf`,
`keyed_tr_depth2`, `keyed_tr_depth2_rightspine`, `keyed_tr_keyonly`,
`keyed_tr_multi_a`, `keyed_tr_pathological`, `keyed_tr_sortedmulti_a`,
`keyed_tr_with_leaf`, plus `compose_tr_seven_leaves` and
`compose_tr_thirty_two_slots`. The three r0 named are v2 legacy artifacts the
codec cannot read under **any** enumeration. So the `Body::Tr` arm is well
covered either way; what is broken is only the count the plan asserts and the
reason it gives. Note also that excluding the 13 is in fact *correct* for
`every_existing_v4_identity_is_byte_preserved`, whose subject is v4 identities.

**Why it matters.** This is the plan's first executable step, it halts, and it
halts with a diagnosis that will send the implementer looking for a header
parser rather than a version filter.

---

### IMP-2 — Task 5 Step 1's refusal silently disables address derivation for kind 1, which is exactly what Task 8 Step 2 must measure

**Lands in:** Task 5 Step 1 (the refusal), Task 8 Step 2
(`addresses_agree_with_lianas_own_and_differ_from_kind_0`), and SPEC §2 step 5's
naming of `md address --network`.

**Reproduction.** `derive.rs:88-97` is
`pub fn derive_address(&self, chain: u32, index: u32, network: Network) ->
Result<Address<NetworkUnchecked>, Error>`, and `derive.rs:134` is

```rust
let desc = crate::to_miniscript::to_miniscript_descriptor(self, chain)?;
```

— the **network-less** entry point, the one Task 5 Step 1 makes refuse
`wire_version() == 8`. `md address` reaches it directly:
`crates/md-cli/src/cmd/address.rs:82` is
`.derive_address(args.chain, i, args.network)?`.

So after Task 5, every kind-1 address derivation returns
`Err(NetworkRequiredForUnspendable)`. Task 8 Step 2's
`md_address(case, i, Chain::Receive)` — the three-way address equality against
Liana's own `receive`/`change` triples, §8.2, the funds-relevant half of the
acceptance vectors — cannot pass. Task 5's Step 1 lists only
`cmd/descriptor.rs` as switching to the `_with_network` form; no task names
`derive.rs:134`.

The fix is one line and the ingredient is already in scope — `derive_address`
**has** `network` and can pass it to
`to_miniscript_descriptor_with_network(self, chain, network)`, which also gives
the internal key's base58 prefix the same network as the address being derived.
That it is a one-line fix is the point: the plan's own argument for the refusal
("Zero of the 55 existing sites change") is a claim about *existing* sites and
was not re-asked for the *new* kind's reachability, so the one production site
that must change was not enumerated.

The other two production callers of the network-less forms are
`cmd/descriptor.rs:200-201` (the plan switches these) and `cmd/vectors.rs:193`
(`md vectors`, which already matches on the `Result` — it would report a
kind-1 row as an error, unmentioned but not wrong).

---

### IMP-3 — Task 6 Step 3 bullet 2 is unchanged and remains unimplementable; it also now conflicts with the GREEN spec

**Lands in:** Task 6 Step 3 bullet 2, Task 6 Step 1 test 4 (**new in r1**),
Task 6 Step 5 "Expected: PASS", and Task 5 Step 7's fixpoint corpus, which the
plan says "will fail until Task 6 adds the marker's parse rule".

**Reproduction.** `template.rs:1589-1592`:

```rust
fn walk_tr(
    t: &miniscript::descriptor::Tr<DescriptorPublicKey>,
    km: &std::collections::BTreeMap<String, u8>,
) -> Result<Node, CliError> {
```

and `:1593` is `let key_str = t.internal_key().to_string();`. A
`Tr<DescriptorPublicKey>` exists only after
`Descriptor::<DescriptorPublicKey>::from_str` has parsed the substituted
template. `UNSPENDABLE(liana)` is not a descriptor key expression, so the parse
fails before `walk_tr` is entered; `:1601`'s
`if key_str == NUMS_H_POINT_X_ONLY_HEX` is never reached. The textual stage is
`substitute_synthetic` (`:1047-1084`), whose regex handles `@i` only — and the
plan quotes that very function one bullet later, in the argument for why
`md encode` must refuse, while still placing the marker rule downstream of it.

**New in r1: this is now a plan/spec conflict.** SPEC §4a's ruling table reads
*"the template grammar | n/a — the marker carries no key binding | accept
`UNSPENDABLE(liana)` via a **substitution rule**"*. Substitution is the textual
stage. Per the brief, a plan/spec conflict is a plan defect.

**Why it matters.** r1 *added* a test that depends on this
(`the_marker_PARSES_in_a_template`, Task 6 Step 1), so the fold moved a gate
onto an unimplementable placement rather than off it. Task 6 Step 5's PASS and
Task 5 Step 7's deliberately-red-then-green fixpoint both hang on it.

---

### IMP-4 — Task 7 Step 3 still names neither the API its tests call nor the hook point, and the hook point has a measured funds regression attached

**Lands in:** Task 7 Step 1 tests 1-3, Task 7 Step 3, Task 8 Step 2b's third
test (**new in r1**, plan `:1082`, which also calls `validate(&…)`).

**Reproduction.** Step 3 is one sentence: *"Implement the four refusals and
their `Error` variants in `error.rs`."* Four tests call `validate(&…)`
(plan `:960`, `:968`, `:974`, `:1082`); no such function exists —
`validate.rs` exposes eleven `validate_*` functions, called from two places
with **different policy**: `encode_payload_inner` (`encode.rs:152-172`, two of
them behind `Admission::Enforce`) and `decode_payload_with_opts`
(`decode.rs:120-150`).

The choice is load-bearing and the crate documents why, verbatim at
`encode.rs:116-131`:

```
/// THIS EXISTS BECAUSE THE ENCODE-SIDE REFUSALS WERE REACHING DECODE.
…
/// That intent was correct and the implementation leaked. `chunk::reassemble`
/// -- a DECODE path -- verifies a chunk set by recomputing the md1 encoding id,
/// `compute_md1_encoding_id` called `encode_payload`, and `encode_payload`
/// applies admission policy. So every rule added to the mint path retroactively
/// made older cards of that shape undecodable. Measured 2026-09-19: a 2-of-2
/// emitted by the shipped toolkit stopped reading, `verify-bundle` reporting
/// `md1_decode: fail OriginKeyContradiction` on a backup whose keys were
/// perfectly intact.
```

A §6 refusal hooked unconditionally repeats that class on kind-1 plates. The
plan leaves the implementer to guess, on the four refusals that exist to keep a
wallet off a shape whose addresses would be wrong.

(The four refusals themselves are still the right four, and r1's replacement of
the fourth with a `wire_version()` property — I6(b) — is a genuine improvement.)

---

### IMP-5 — `cases.json` cannot feed Task 5 Step 2 or Task 8 Steps 1-2; both still contain undefined fixtures, one of them an explicit placeholder

**Lands in:** Task 0 Step 1 (the schema), Task 5 Step 2 (all four kind-1
tests), Task 8 Step 1, Task 8 Step 2.

**Reproduction.** Task 0 Step 1 defines one object per case: `name`,
`accepted`, `leaf_pubkeys_hex` (33-byte, wire order), `expected_xpub`,
`descriptor_with_checksum`, `liana_receive[3]`, `liana_change[3]`, plus the
source SHA. Consumer by consumer:

- **Task 4** (`GOLDEN_CASES`: name, leaf pubkeys, expected xpub) — **sufficient**.
- **Task 8 Step 2b** (nested case by name) and **Step 2c** (three presets) —
  **sufficient**, because Step 1 vendors all eight records including
  `accepted: false`. This is the fold's correct answer to I9.
- **Task 7** — does not consume it.
- **Task 5 Step 2** needs `descriptor_with_real_keys_at_liana_kind()`: an md1
  `Descriptor` carrying real seated keys at kind 1, so that
  `to_miniscript_descriptor_multipath_with_network(&d, Bitcoin)?.to_string()`
  starts with the golden `tr(xpub661MyMwAqRbcFswVugWF`. Building one needs the
  full **65-byte** TLV entries (`chain code ‖ compressed pubkey`,
  `validate.rs:331-335`). `cases.json` carries the 33-byte pubkeys only — no
  chain codes. Nothing in the plan constructs this fixture.
- **Task 8 Step 1** is explicit about the hole: `md(&["descriptor", /* the
  kind-1 chunks for this shape */])`. No step produces kind-1 chunks for the
  four golden shapes, and no md1 card of that kind exists anywhere yet (the
  route would be marker-template → `md encode` → `md seat`, which is IMP-3's
  territory). Task 8 Step 2 inherits the same hole.

The data is *recoverable* — each leaf xpub, chain code included, is inside
`descriptor_with_checksum` — so the shortest repair is probably a
`leaf_xpubs_base58` field plus a step that mints the kind-1 card. That is a
design decision the plan should make rather than the implementer.

This also falsifies two Self-Review claims: *"No TBDs. Every code step carries
real code"* (plan `:1297`), and the closing paragraph at `:1302`, which still
routes this extraction to a *"Task 8 Step 0"* that Task 0 replaced and that no
longer exists.

---

### IMP-6 — the identity golden's shape and its test's shape disagree, and as written the gate pins 1 of the 3 values it captures

**Lands in:** Task 0 Step 3 (new), Task 8 Step 3
(`every_existing_v4_identity_is_byte_preserved`).

**Reproduction.** Task 0 Step 3: *"`examples/dump_ids.rs` emits `[[name,
wallet_policy_id, template_id, phrase], …]`"* — a **4-tuple** per row. Task 8
Step 3 reads it back as

```rust
let golden: Vec<(String, String)> =
    serde_json::from_str(include_str!("golden/pre_refactor_ids.json")).unwrap();
for (name, id) in golden { assert_eq!(compute_id_by_name(&name), id, "{name}"); }
```

`serde_json` deserialising a 4-element array into a 2-tuple fails
("invalid length 4, expected a tuple of size 2"), so the test panics on its
first line. Beyond the shape: even once repaired, the loop compares **one**
value, leaving `template_id` and the 12-word `phrase` captured but unchecked —
and the 12-word phrase is what an operator reads off steel. §8.5's "every
existing v4 identity byte-preserved" is the half that protects plates already
minted; the plan's own commit message for Task 9 mutation 6 calls the identity
sites "the funds-relevant one".

`compute_id_by_name` is still undefined by any task (r0 noted this inside I2);
it must recompute all three values from the same vendored phrase files on the
post-change tree.

---

## Minor

- **MIN-1** — Task 4 Step 3 forbids `hex_lit`; Step 4's code block (`:623`)
  still calls `hex_lit::hex!`. Note the replacement is not a copy of the
  existing constant: `nums.rs:12`'s `NUMS_H_POINT_X_ONLY_HEX` is the **x-only**
  32-byte hex *string*, and step 4 of §2 needs the **33-byte compressed** form
  (`02 ‖ x`). Say which.
- **MIN-2** — Task 1 Step 2 still instructs "Reuse that file's
  `conformance_dir()` / `keyed_phrase_files()` helpers rather than writing new
  ones" and re-runs a generation Task 0 already committed. `keyed_phrase_files()`
  (`dump_skeleton_keys.rs:23-36`) filters `starts_with("keyed_")` and
  `:62-66` hard-asserts the header. Contradicts Task 0 Step 2. Measured
  mitigation: 18 of the 20 decodable `Body::Tr` vectors are `keyed_*`, so gate
  coverage survives either reading — this is a clarity defect, not a blind gate.
- **MIN-3** — clippy, `cargo fmt --all --check` and `cargo doc` still run only
  at Task 10, nine commits after a ~59-site rename, with
  `[workspace.lints] missing_docs = "warn"` (root `Cargo.toml:11`) and
  `-D warnings` live and the baseline commit itself a `cargo doc` CI fix.
- **MIN-4..MIN-10** — M1, M2, M3, M4, M5, M6, M7 exactly as r0 stated them;
  reproductions in the Q1 table above.
- **MIN-11** — Task 5 Step 5 places `build_liana_unspendable_internal_key` at
  `:333-338`, where no leaves have been built (`internal_key` at `:341-345`
  precedes `tree_to_taptree` at `:346-350`); its `leaves: &[(u8,
  Miniscript<DescriptorPublicKey, Tap>)]` is not the real shape
  (`TapTree::leaves()` yields `TapTreeIterItem` over `&Arc<Miniscript>`); and
  "return `Err(failed(...))`" cannot be done from a `for_each_key` closure,
  which returns `bool`. All three are mechanically repairable and the order and
  key semantics are correct — see Q2(b).
- **MIN-12** — `LIANA_UNSPENDABLE_MARKER` is consumed at plan `:706`, `:797`,
  `:871` and `:1300` and **defined with a value by no task**; Task 4 Step 3's
  "make the two new items `pub`" is the closest thing. `Error::
  NetworkRequiredForUnspendable` is used in Task 5 but only Task 7 Step 3 adds
  `Error` variants. `Header` is not currently imported in `to_miniscript.rs`
  (which contains zero occurrences of `Network` too).
- **MIN-13** — "**55** call sites (measured)" (plan `:711`, `:732`, `:858`) does
  not reproduce: 58 identifier occurrences repo-wide (including the 2
  definitions, `use` lines and doc comments), 36 call-expression lines. The
  argument holds a fortiori; the number should be re-measured or dropped.
- **MIN-14** — Task 8b leaves `policy_shape.rs:250-254` collapsing both kinds to
  `KeyPathKind::Nums`, so Step 1's "say what is now true" must be worded as
  *observable on the wire, not in this type (§7/§7a, stage 3)* or it becomes the
  next wrong instruction. `policy_shape.rs:117-119`'s `Body::Tr::is_nums`
  citation also dies with Task 1 and is not in Task 8b's scope.
- **MIN-15** — Task 2's Files list still omits `error.rs` despite the new Step 5b
  (harmless: Step 6 stages `crates/md-codec/src`). Step 5b's
  `!s.contains("expected 4")` rejects `"expected 4 or 8"`, a correct rendering.
- **MIN-16** — Self-Review §2 ("No TBDs. Every code step carries real code") and
  the closing "One known gap" paragraph at `:1302` (which routes work to a
  "Task 8 Step 0" that no longer exists) were not propagated through the fold.

---

## What the fold got right, recorded so it is not re-litigated

- Task 5's diagnosis is now measured and correct: §4's three rows do live in two
  files, `render.rs`'s `Mode { Literal, Abstract }` needs no new variant because
  **both** keyless modes emit the marker (SPEC §4's table agrees), and Step 4's
  match block type-checks against the real arm at `render.rs:183-202`
  (`render_key(*i, default_usp, overrides, out)?` and `out.push_str(...)` both
  yield `()`).
- The `_with_network` design avoids a 36-site mechanical diff and cannot emit a
  wrong-network xpub by accident. Its only casualty is `derive.rs:134` (IMP-2).
- I6(a) and I6(b) are replaced with honest assertions rather than invented
  knobs, and both replacements are sound.
- Task 8b Step 3's grep is a check that can actually fail — verified.
- Task 0's ordering, its no-source-changes commit gate, and its use of already-
  public identity APIs are all sound; only the enumeration is wrong.
- Task 8 Steps 2b/2c genuinely close §8.1's three gaps and §8.6's third pin.
- Task 2 Step 5b's Display change breaks no existing test, doc or snapshot.

---

**ready for implementation: no**
