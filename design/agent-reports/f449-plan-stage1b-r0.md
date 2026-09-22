# R0 review — `IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md`

**Verdict: 4 Critical / 7 Important / 9 Minor / 4 Nit**

Reviewer: independent agent, opus. Scope: the plan only. Repo read at
`descriptor-mnemonic` `37367c1f`; spec read for coverage checking only, not
re-reviewed. The G-1..G-8 table was checked for *correct and complete*
discharge, not re-derived: **G-1, G-2, G-3, G-4, G-7 and G-8 are discharged
correctly; G-5 and G-6 are discharged incorrectly (C1/C3 and C4/I1).**

Every line, file and byte cited below was resolved against the real source.

---

## C1 — Task 7's fixture panics: `keyed_tr_sortedmulti_a` is not a NUMS `tr`

**Task 7, the fixture block. Also Task 9 mutations 8-11, which depend on it.**

Task 7 defines

```rust
fn tr_liana_with_sortedmulti_a_leaf() -> Descriptor {
    kind1_from_vector("keyed_tr_sortedmulti_a")
}
```

and describes it as "Built from the vendored single-path bare-multi vector,
whose sole leaf IS a sortedmulti_a, **then switched to kind 1**."

**Reproduction.** `crates/md-codec/tests/vectors/keyed_tr_sortedmulti_a.template`:

```
tr(@0/48'/0'/0'/2'/<0;1>/*,sortedmulti_a(2,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))
```

and `keyed_tr_sortedmulti_a.descriptor.json` → `tree.body.data` is
`is_nums: false, key_index: 0`. That vector's internal key decodes to
`InternalKey::Slot(0)`, **not** `NumsPoint`.

`kind1_from_vector` (Task 1 Step 2) opens with

```rust
assert_eq!(*internal_key, InternalKey::NumsPoint,
           "{vector_name}: fixture must start at kind 0");
```

so the call panics: `keyed_tr_sortedmulti_a: fixture must start at kind 0`.

**Why it matters.** Task 7's only test cannot run, and Task 9's mutations 8-11
("drop each of §6's four refusals") have no fixture to fire against. The
NUMS-internal-key, keyed, `sortedmulti_a`-leaf vector that does exist is
`keyed_compose_tr_sole_sortedmulti_a` (`is_nums: true`, inner tag
`SortedMultiA`, `pubkeys` present) — the plan cites the wrong one of two
similarly named vectors.

The Self-Review says these vector names were "verified to exist". They exist.
Their **shape** was never checked, and existence is the weaker property here.

---

## C2 — Tasks 4 and 5 specify the hashed leaf-key vector two different,
## non-equivalent ways

**Task 4 Step 3 vs Task 5 Step 5.**

Task 1 Step 1 defines the fixture source as "`leaf_tlv_hex` (the **65-byte**
`chain code ‖ compressed pubkey` entries **in wire order**)" — one entry per
**slot**, `n` entries.

Task 4 Step 3 restates that as the implementation contract: "sha256 over each
leaf's 33-byte compressed pubkey in **descriptor left-to-right order**, which
is **wire order**".

Task 5 Step 5 says something different: "`to_miniscript.rs` builds the derived
xpub from the already-built **leaf miniscripts in tap-tree order**" — one entry
per key **occurrence**.

**Reproduction that they differ.** md1 permits one `@i` in two leaves.
`crates/md-codec/src/validate.rs:17-26`:

```rust
    // Each @i for 0 ≤ i < n must appear at least once.
    for (i, was_seen) in seen.iter().enumerate() {
        if !was_seen {
            return Err(Error::PlaceholderNotReferenced { idx: i as u8, n });
```

"at **least** once", not exactly once. So `tr(H,{pk(@0),and_v(v:pk(@0),older(5))})`
with `n = 1` passes `validate_placeholder_usage` (`seen[0] = true`,
`first_occurrences = [0]`). Under Task 4's rule the concatenation has one
33-byte entry; under Task 5's it has two. `sha256` of the two differs → a
different chain code → a different internal xpub → **a different taproot output
key and a different address**.

Nothing in the plan discriminates: all eight evidence cases and all named
vendored vectors have no repeated slot, so Task 4's goldens and Task 8's four
ACCEPT shapes pass under either reading.

**A second, compounding gap in the same step.** Task 5 gives no route from what
`to_miniscript.rs` actually holds to what `liana_unspendable_xpub` takes.
`node_to_descriptor` (`crates/md-codec/src/to_miniscript.rs:314-317`) is

```rust
fn node_to_descriptor(
    node: &Node,
    keys: &[DescriptorPublicKey],
) -> Result<miniscript::Descriptor<DescriptorPublicKey>, Error> {
```

— no `Network`, and `keys` are `DescriptorPublicKey`s carrying origin and
derivation path, not the `[u8; 33]` account-level compressed pubkeys Task 4's
signature demands. At `:340-345` the internal key is also built **before** the
script tree at `:346-350`. Threading network and recovering the 33-byte pubkeys
is left entirely to the implementer.

---

## C3 — Task 8's acceptance has no way to construct the four ACCEPT shapes

**Task 8, first two clauses.**

Task 8 requires "Descriptor equality **including the BIP-380 checksum** for the
four accepted shapes, **against `cases.json`**" and "three-way address equality
(md vs **the evidence's recorded Liana addresses** vs kind 0, which must
differ)".

Both need an md-codec `Descriptor` carrying the evidence's exact keys, origins
and tree. The plan's only Descriptor builder is `kind1_from_vector(vector_name)`,
which reads `crates/md-codec/tests/vectors/<name>.phrase.txt`.

**Reproduction — no vendored vector reproduces any evidence case.**

| evidence case | corpus counterpart | why it is not the same descriptor |
| --- | --- | --- |
| `preset-kofn-recovery-tr` | `keyed_compose_preset_kofn_recovery` (`tr`) | corpus `tlv.fingerprints` = `[[0,"73c5da0a"],[1,"73c5da0a"],[2,"73c5da0a"],[3,"73c5da0a"]]`; evidence leaves are `73c5da0a / 3f635a63 / 66d455ea / 73c5da0a` |
| `preset-tiered-recovery-tr` | `keyed_compose_preset_tiered_recovery` | corpus template is `wsh(or_d(multi(…)))`, not `tr` |
| `preset-decaying-multisig-tr` | `keyed_compose_preset_decaying_multisig` | corpus template is `wsh(or_i(…))`, not `tr` |
| `same-seed-two-paths-tr` | none | — |
| `X19-tr-kofn-nums-older5` | none | — |

`cases.json`'s field list (Task 1 Step 1) records `name`, `accepted`,
`leaf_tlv_hex`, `leaf_pubkeys_hex`, `expected_xpub`,
`descriptor_with_checksum`, `liana_receive[3]`, `liana_change[3]` — **no md1
phrase and no kind-0 descriptor string**. The evidence *does* carry the kind-0
descriptor as the `md` variant in `fable-liana-parse-in.jsonl` (verified: each
of the eight names has both an `md` and a `liana-unspendable-xpub` record), but
the plan's generator does not vendor it.

G-5 forecloses the fallback in the plan's own words: "A kind-1 `Descriptor`
cannot be built from TLV bytes alone — no taptree, no `older`, no fingerprints,
no divergent origins."

And md-codec, where `cases.json` is placed, has no parser to consume such a
string even if it were vendored —
`crates/md-codec/tests/internal_key_refactor.rs:3-4`: "md-codec has **NO**
template parser — `parse::template` lives in md-cli".

**Why it matters.** §8.2 and §8.3 are the funds-relevant legs: they are what
proves md renders the same descriptor and derives the same addresses Liana
recorded. As written the task cannot be executed, and an implementer's most
likely improvisation — running it against a *corpus* vector instead — produces
a self-consistent test that measures nothing against Liana.

---

## C4 — Task 7's refusals are never wired to a call site, and no test asserts
## that any of them refuses

**Task 7, Files list and test block.**

Task 7's Files list is `validate.rs`, `error.rs`. The admission gate is in
`encode.rs`, which Task 7 never names. `crates/md-codec/src/encode.rs:143-170`:

```rust
fn encode_payload_inner(d: &Descriptor, admission: Admission) -> … {
    …
    crate::validate::validate_placeholder_usage(&d.tree, d.n)?;          // UNCONDITIONAL
    …
    if matches!(d.tree.tag, crate::tag::Tag::Tr) {
        if let Body::Tr { tree: Some(t), .. } = &d.tree.body {
            crate::validate::validate_tap_script_tree(t)?;               // UNCONDITIONAL
        }
    }
    …
    if admission == Admission::Enforce {
        crate::validate::validate_origin_key_consistency(d)?;
        crate::validate::validate_no_duplicate_key_slots(d)?;
    }
```

The plan never says the four new `validate_*` functions must be called from
inside the `Admission::Enforce` block. `validate_tap_script_tree` — an
unconditional `Tag::Tr` check — is the natural neighbour an implementer will
reach for, and putting a §6 refusal there reproduces exactly the 2026-09-19
regression the plan's own Global Constraint cites.

**The prescribed test does not close this.** Task 7's only test asserts

```rust
assert!(decode_payload(&bytes, bits).is_ok(), "a mint-side refusal reached decode");
```

If a refusal function is written into `validate.rs` and never called from
`encode_payload_inner`, that assertion is still true, the suite is green, and
Task 9's mutations 8-11 mutate dead code — a mutation that applies and is
semantically inert. Task 7 prescribes **no test that any of the four refusals
actually refuses**.

The repo's own precedent for this class ships a control for exactly this
reason. `crates/md-codec/tests/mint_policy_does_not_reach_decode.rs:89-98`:

```rust
/// The control: minting it is still refused. Without this the test below could
/// pass because the rule stopped firing anywhere.
#[test]
fn minting_it_is_still_refused() {
```

Task 7 has no equivalent.

---

## I1 — Task 7's test cannot compile where it must live

`encode_payload_inner` is **module-private**, not `pub(crate)`:

```
crates/md-codec/src/encode.rs:99:  pub fn encode_payload(…)
crates/md-codec/src/encode.rs:139: pub(crate) fn encode_payload_for_identity(…)
crates/md-codec/src/encode.rs:143: fn encode_payload_inner(…)
```

So the prescribed test can live only in `encode.rs`'s own `#[cfg(test)] mod
tests` — not in `validate.rs` (Task 7's file) and not in any integration test.

But it also calls `kind1_from_vector`, defined by Task 1 in
`tests/common/liana.rs`, an `include!`-spliced integration-test file whose
sibling `tests/common/vendored.rs:19` opens with `use md_codec::Descriptor;`.
A crate cannot name itself — `crates/md-codec/src/policy_shape.rs:229-231`
records the rule: "NOTE: `crate::`, never `md_codec::` — this crate does not
self-alias (`lib.rs` has no `extern crate self as md_codec`)". Splicing those
helpers into a `src/` unit test does not compile.

G-6's framing is also over-stated. It says a unit test is *required* because
`Admission` is `pub(crate)`; the repo already reaches the SkipPolicy path from
an integration test through public API —
`tests/mint_policy_does_not_reach_decode.rs:101-108` calls
`md_codec::identity::compute_md1_encoding_id`, which routes to
`encode_payload_for_identity` → `Admission::SkipPolicy`.

---

## I2 — Task 1 Step 2b's "remove that attribute" turns `phase-gate.sh` red

Task 1 Step 2b: "`all_vendored_vector_names` already exists in stage 1a's
`tests/common/vendored.rs` (it carries `#[allow(dead_code)]` there because
stage 1a had no caller — this stage is the caller, so **remove that
attribute** when you first use it)."

**The premise is wrong.** `tests/common/vendored.rs:30-37` states why the
attribute is there:

```
/// `#[allow(dead_code)]`: not every spliced-in consumer calls this -- e.g.
/// `tests/internal_key_refactor.rs` iterates the golden's own vector list
/// instead of re-listing the vectors directory, so this function is dead
/// code in that compilation unit specifically.
```

Confirmed: `grep -c all_vendored_vector_names
crates/md-codec/tests/internal_key_refactor.rs` → `0`, while
`examples/dump_encodings.rs:44` and `examples/dump_ids.rs:30` both call it.
The file is spliced into three roots; removing the attribute leaves an unused
private fn in the third.

`scripts/phase-gate.sh:32` runs
`cargo clippy --locked --all-targets --all-features -- -D warnings`, which
promotes the default-warn `dead_code` lint to a hard error. Step 5's gate goes
red on a step the plan prescribes.

---

## I3 — Task 6's prescribed mechanism cannot produce `InternalKey::LianaUnspendable`

Task 6's table: "the template grammar | accept `UNSPENDABLE(liana)` via a
substitution rule in **`substitute_synthetic`**, **NOT `walk_tr`** — `walk_tr`
runs after `Descriptor::from_str`, which fails first on a non-key token".

The diagnosis of *why the token must be substituted early* is correct. The
conclusion is not: `walk_tr` is the **only** place the internal-key kind is
decided. `crates/md-cli/src/parse/template.rs:1593-1632`:

```rust
    let key_str = t.internal_key().to_string();
    …
    if key_str == NUMS_H_POINT_X_ONLY_HEX {
        return Ok(Node { tag: Tag::Tr, body: Body::Tr {
            internal_key: InternalKey::NumsPoint, tree } });
    }
    let key_index = lookup_key(&key_str, km)…?;
    Ok(Node { tag: Tag::Tr, body: Body::Tr {
        internal_key: InternalKey::Slot(key_index), tree } })
```

Whatever `substitute_synthetic` writes into the internal-key position, the
result is `NumsPoint` (if it substitutes the NUMS hex) or `Slot(i)` (if it
substitutes a synthetic xpub). There is no third outcome. `walk_tr` must also
learn the substituted sentinel, and the plan's "NOT `walk_tr`" steer points the
implementer away from the one file that has to change.

**Why it matters.** Task 8's "render-reparse fixpoint extended to `tr` kind 1"
is unsatisfiable without it, and `md compose | md encode` does not round-trip.

---

## I4 — Task 5 Step 4's production-caller enumeration is wrong by eight

G-7 and Step 4 name three callers of the network-less entry points, and all
three are correct: `crates/md-cli/src/cmd/descriptor.rs:200-201` (with
`args.network` at `:62` ✓), `crates/md-codec/src/derive.rs:134` (inside
`derive_address`, `network` in scope, used at `:155` ✓), and
`crates/md-cli/src/cmd/vectors.rs:193` ✓.

There are eight more production call sites of the network-less
`to_miniscript_descriptor_multipath`, none mentioned anywhere in the plan:

```
crates/md-cli/src/seat/disposition.rs:308
crates/md-cli/src/seat/disposition.rs:434
crates/md-cli/src/seat/disposition.rs:437
crates/md-cli/src/seat/compose.rs:403
crates/md-cli/src/seat/compose.rs:406
crates/md-cli/src/seat/matching.rs:304
crates/md-cli/src/seat/matching.rs:335
crates/md-cli/src/seat/matching.rs:454
```

Under Step 3 ("the existing two … **refuse** a descriptor whose
`wire_version() == 8`") every one of these returns `Err` for a kind-1
descriptor.

**Severity note, in the plan's favour:** this is fail-closed, not
wrong-result, and in stage 1b the composer cannot produce kind 1
(`crates/md-codec/src/compose/tr.rs:47` maps a missing internal key to
`InternalKey::NumsPoint`). I also checked that the refusal does not break the
existing property suites: `LianaUnspendable` appears in **no** test file, and
neither `proptest_to_miniscript.rs` nor `proptest_roundtrip.rs` constructs
`InternalKey` at all, so P6's "`to_miniscript_descriptor` must succeed" oracle
is untouched. The defect is that the plan asserts a complete enumeration
("the THREE production callers") that is not one.

---

## I5 — §8.1's "nested taptree that Liana ACCEPTS" has no owning task

Spec §8.1 requires, beside the eight goldens, "**new vectors** for the three
unmeasured gaps in §2: a `tpub` wallet, a **nested taptree that Liana
ACCEPTS**, and — if §6 did not refuse it — a `sortedmulti_a` leaf."

- tpub gap → Task 4's `a_tpub_wallet_derives_a_tpub_internal_key` ✓
- `sortedmulti_a` gap → excused, §6 refuses it ✓
- nested-taptree gap → **nothing**

Task 4's `the_chain_code_depends_on_the_KEYS_not_the_TREE` uses
`preset-decaying-multisig-tr`, which Liana **refused**. Measured in
`design/evidence/composer-fable-r0/fable-liana-parse-out-v15.jsonl`:
`{"name":"preset-decaying-multisig-tr","ok":false,"error":"Descriptor is not
compatible with a Liana spending policy."}`. §2 says so itself: "which Liana
**refused on policy shape**, so no ACCEPT backs it … is not measured (fable
M-2)."

The Self-Review maps "§8 items 1/2/5/6/7 → Task 8" and its
"owned by later stages" list does not park this gap anywhere.

---

## I6 — §6's pinned invariant (SELF-2) has no task

Spec §6, immediately under the refusal table:

> **A refusal that cannot fire is not a guard** … It ships as a pinned
> invariant **with a test demonstrating *why* it is unreachable**, so if the
> placeholder rule ever changes the pin fails instead of admitting an empty
> concat.

The Self-Review maps "§6 → Task 7". Task 7 lists four refusals and nothing
else. No task writes the empty-concat pin.

Related, and also unassigned: §6 row 3's `--unspendable liana` no-op **WARN**
(fable M-6) appears in neither Task 7 nor the "owned by later stages" list.

---

## I7 — Tasks 6, 7, 8 and 9 have no steps

The plan's own header: "REQUIRED SUB-SKILL: Use
superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax."

Measured checkbox count per task:

```
Task 1: 4   Task 2: 5   Task 3: 5   Task 4: 4   Task 5: 6
Task 6: 0   Task 7: 0   Task 8: 0   Task 9: 0   Task 10: 4
```

The four with zero are the input side (§4a), every §6 refusal, the entire §8
acceptance surface and the §8.10 mutation pass. They carry no test-first
ordering, no "run to verify they fail", no gate step and no commit step; Tasks
6, 8 and 9 additionally have no "Interfaces produced" block. Tasks 1-5 and 10
are executable documents; 6-9 are paragraphs of intent. The four tasks holding
the funds-relevant gates are the four with no executable structure.

---

## Minor

**M1 — Task 6 pulls two stage-2 items into stage 1b, undeclared.** Spec §9's
stage-2 content column names "the `UNSPENDABLE(liana)` template substitution
rule" and "the JSON schema version bump (§4a)"; stage 1b's column names only
"§4a's `md decompose` recogniser and `md encode` refusal". Task 6 does both.
The substitution rule is arguably forced into 1b (§8 item 7, the fixpoint, is
owned by 1b and needs re-parsing), but the plan neither says so nor parks the
JSON schema bump.

**M2 — Task 5 does not name the new `KeyPathKind` variant.** §7 does, with its
reason: "Name it `NumsXpub` rather than `Unspendable` — `Unspendable` sitting
next to `Nums` implies `Nums` is spendable". Task 5 says only "maps to the new
`KeyPathKind`". The Self-Review simultaneously parks "§7/§7a's device work
(stage 4)" while Task 5 implements §7's Rust half.

**M3 — Task 9 omits §8.10's seventh named mutation.** §8.10 names "group the
new kind with `KeyPathSpendable`", i.e. `policy_shape.rs:251` →
`KeyPathKind::Xpub`. Task 9's mutations 12-15 collapse the four or-patterns to
the **NUMS** arm, which is a different mutation.

**M4 — Task 8 omits §8.5's mk1 `policy_id_stub` leg.** §8.5 requires "a
different **mk1 `policy_id_stub`**" and §9 assigns §8.5's Rust leg to 1b. Task
8 names only `WalletPolicyId`, `WalletDescriptorTemplateId` and the 12-word
phrase.

**M5 — the generated vector corpus is not regenerated.**
`crates/md-cli/tests/vector_corpus.rs:15-30` runs `md vectors --out <tmp>` and
`diff -r`s it against `crates/md-codec/tests/vectors`. Task 5/6's `json.rs`
third state changes that output for every `tr` vector. No task says to
regenerate the committed corpus in the same commit.

**M6 — Task 10 Step 1's second text is in a different repo.**
`DESIGN_coordinator_compatibility.md` does not exist under
`descriptor-mnemonic`; it is at
`/scratch/code/shibboleth/mnemonic-engrave/design/DESIGN_coordinator_compatibility.md`
(content verified at `:155-158`, and the plan's warning about the phrase
wrapping across two `///` lines is correct for `policy_shape.rs:130-131`). The
plan names one REPO and one gate script and never declares the cross-repo edit;
the File Structure table omits the file entirely.

**M7 — `chunk.rs:373` needs no change, and nothing cross-checks the two
versions.** Task 2 Step 4 says "`chunk.rs:70`/`:373` accept `{4, 8}`". `:373`
is `|| h.version != expected_version` inside the cross-chunk consistency loop
(`chunk.rs:370-377`) — it compares chunks to each other and is already
version-agnostic. Separately, nothing verifies that a chunk header's version
agrees with the reassembled payload's `Header` version; a mixed set decodes by
the payload header and still passes the step-7 chunk-set-id check. Not a
desync, but the plan's "so chunk headers agree with their own payload"
(`:279`) is enforced only on the write side.

**M8 — `kind1_from_vector` is unguarded against keyless vectors.** It accepts
any `tr`-rooted vector, including template-only ones carrying no `Pubkeys` TLV
— measured: `compose_tr_thirty_two_slots` and `nums_taproot` are `is_nums:
true` with `pubkeys: false`; `tr_keyonly`, `tr_with_leaf`, `compose_tr_seven_leaves`
likewise carry none. §4 says "the derived xpub **cannot** appear in a keyless
form". No assert guards it.

**M9 — Task 4's structure-independence test never runs production code.**
`the_chain_code_depends_on_the_KEYS_not_the_TREE` builds
`xs = names.map(|n| case(n).expected_xpub)` and compares the three strings to
each other; `liana_unspendable_xpub` is never called. It pins a property of
`cases.json`, not of the implementation. §8.6 is still covered transitively by
`the_recipe_reproduces_every_golden_xpub` (which covers all eight), so this is
redundancy rather than a false PASS. Its premise is sound, verified: the four
leaf keys of `preset-kofn-recovery-tr`, `preset-tiered-recovery-tr` and
`preset-decaying-multisig-tr` are byte-identical in the same order
(`73c5da0a/48'/0'/0'/3'`, `3f635a63/48'/0'/0'/3'`, `66d455ea/48'/0'/0'/3'`,
`73c5da0a/48'/0'/1'/3'`) and their three internal keys are the same string.

---

## Nit

**N1 — the plan gives two different `tr` counts.** Task 1 Step 2b's doc
comment says "Stage 1a measured **23** of the 65 vectors as carrying
`Body::Tr`"; the Self-Review says "**22** of the 65 vendored vectors are
`tr`-shaped". 23 is right —
`crates/md-codec/tests/internal_key_refactor.rs:31` asserts
`g.tr_count == 23`, and 23 `.template` files are `tr`-rooted.

**N2 — "55 call sites" is high.** Measured 39 call expressions matching
`to_miniscript_descriptor(_multipath)?\(` across `crates/**` and `fuzz/`; 57
lines mention the identifiers at all, including doc comments, the two
definitions and the `lib.rs:72` re-export. The conclusion (do not change the
existing signatures) is unaffected.

**N3 — Task 2 names the wrong recursive function and omits an import.** The
actual recursive decoder is `read_node_with_depth` (`tree.rs:212`), which
`read_node` (`:205`) delegates to; and `tree.rs` currently imports only
`bitstream`, `error` and `tag`, so the Task 3 write-side snippet's
`Header::WF_UNSPENDABLE_VERSION` needs a new `use crate::header::Header;`. Both
are compile errors, not silent defects.

**N4 — the four refused cases have no addresses.** Task 1 Step 1 specifies
`liana_receive[3]` / `liana_change[3]` per case, and `Case` declares them as
non-optional `Vec<String>`. Measured: the `out-v15` records for
`preset-hashlock-gated-tr`, `preset-decaying-multisig-tr`,
`hashlock-gated-tr-hash160` and `X20-tr-hashlock-known` carry only
`{error, name, ok, variant}` — no `receive`, no `change`, no `liana_desc`.
They must be empty vectors; the plan does not say so. (`expected_xpub` and
`descriptor_with_checksum` *are* derivable for all eight, from the
`liana-unspendable-xpub` record in `fable-liana-parse-in.jsonl`, whose
internal key is the golden and whose descriptor carries a `#checksum` —
verified for all eight.)

---

## Per-task soundness

**Task 1 — vendor the evidence and the kind-1 fixture builder: UNSOUND** (C3's
missing field, I2's attribute removal, M8, N4). The core builder is otherwise
correct: `decode_vendored(&load_vendored_phrase(name))` type-checks
(`Vec<String>` → `&[String]`), `Descriptor`'s fields are all `pub`
(`encode.rs:17-28`), and swapping `internal_key` alone is sufficient — `n`,
`key_index_width()` and the TLV section are unaffected by the kind bit, so the
result encodes at v8 and round-trips. `hex`, `serde`, `serde_json` are all
md-codec dev-dependencies. The vendoring script is feasible: all eight names,
the four `accepted: true` names, and the 65-byte/33-byte slicing all check out
against the evidence.

**Task 2 — wire version 8 and the derived version: SOUND** (N3, M7 only).
The version-threading enumeration is **complete**, verified exhaustively: the
only production `WF_REDESIGN_VERSION` sites are `encode.rs:174`, `chunk.rs:70`,
`chunk.rs:279` and `header.rs:42` (`chunk.rs:97/114/129` and `header.rs:59/72/119`
are tests); `write_node`'s non-test callers are exactly `encode.rs:181`,
`identity.rs:90` and `identity.rs:200` (`identity.rs:626` is a test);
`read_node`'s sole caller is `decode.rs:89`; `decode_with_correction` holds no
version constant of its own. `Descriptor::wire_version()` as written compiles
and terminates against the real `Body`: `Option<Box<Node>>::as_deref()` gives
`Option<&Node>` and `is_some_and(needs_v8)` type-checks; `Children` and
`Variable` are the only other child-bearing variants, so the `_ => false` arm
is complete; and it **does** find a `tr` nested inside another node, including
one inside a taptree. The evenness test models `decode.rs:191`'s
`(b >> 3) & 0x01` correctly and can fail (version 5 → `1`). The
`WireVersionMismatch` Display test can fail today: `error.rs:33` is
`#[error("wire-format version mismatch: got {got}, expected 4")]`. The plan
does not say which file owns `wire_version()`; it is `encode.rs`, where
`Descriptor` lives.

**Task 3 — the kind bit, written only at version 8: SOUND.** No desync path
found. Writer and reader both take the version from the same header, so v4 and
v8 payloads can never disagree within one decode. A pre-stage-1b decoder
rejects v8 at `header.rs:42` (single-string) and `chunk.rs:70` (chunked), and
`decode.rs:191`'s dispatch routes an even version correctly. Stage 1a's
byte-equality gate still covers v4: all 65 vendored vectors decode at version
4, `wire_version()` returns 4 for them, and no kind bit is written — so
`internal_key_refactor.rs` stays green, and mutation 7 ("write the kind bit at
version 4") turns it RED as the plan claims. The write-side snippet type-checks
(`*internal_key` is `Copy`, `u64::from(bool)` is fine).

**Task 4 — §2's derivation: SOUND except C2's ambiguity** (plus M9). `bitcoin`
is a non-optional md-codec dependency, so `liana_unspendable_xpub` in the
ungated `nums` module compiles without the `derive` feature; `lib.rs:30` is
indeed `mod nums;` and needs the `pub`. All four prescribed tests can fail.

**Task 5 — rendering and the four gating sites: UNSOUND** (C2, I4, M2). G-1's
four sites are all real and correctly named, verified by exhaustive grep:
`render.rs:192`, `to_miniscript.rs:341`, `policy_shape.rs:251`,
`md-cli/src/format/json.rs:356` — and those are the only four
`NumsPoint | LianaUnspendable` or-patterns outside `tree.rs`. The
`_with_network` design is sound in principle and does not break the existing
property suites. `skeleton.rs` is not in the Files list but its
`key_path_kind_label` match (`skeleton.rs:318`) is exhaustive, so the compiler
catches it.

**Task 6 — the input side: UNSOUND** (I3, I7, M1). Every line citation
resolves: `decompose/mod.rs:439`, `walk.rs:175`, `walk.rs:202`,
`template.rs:1047-1084` are all exactly what the plan says they are, and the
three-surface split and the G-8 ruling are carried faithfully.

**Task 7 — §6's refusals: UNSOUND** (C1, C4, I1, I6, I7).

**Task 8 — §8's acceptance vectors: UNSOUND** (C3, I7, M4). The chunked leg is
feasible as described: `keyed_compose_tr_nums_three_leaves.phrase.txt` is a
10-line chunk set, and `chunk::split(&Descriptor)` at `chunk.rs:240` is the
producer G-4 names. `golden/pre_refactor_ids.json` exists.

**Task 9 — the mutation pass: UNSOUND as a gate** (C4 makes 8-11 inert, C1
removes their fixture, M3). The mapping of mutations 1-7 and 12-15 to
their catching tests is otherwise correct.

**Task 10 — doc retirement, version bump, CHANGELOG: SOUND except M6.**
`md-cli/Cargo.toml:28` is `md-codec = { path = "../md-codec", version =
"=0.45.1" }`, so the same-edit rule is right; `policy_shape.rs:122-133` carries
the "Do not 'restore fidelity' with the Go name here" text across a line wrap
at `:130-131`, exactly as warned.

---

**Ready for implementation: no.**
