# R4 closing gate — `IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md` (r5 fold)

**Verdict: 0 Critical / 2 Important / 4 Minor / 2 Nit (new) — NOT GREEN.**

Reviewer: independent agent, opus. Scope: the r5 fold (`7c32f0e1..b092c18c`)
against `design/agent-reports/f449-plan-stage1b-r3.md`. Spec read for authority,
not re-reviewed. Repo read and RUN at `descriptor-mnemonic` `37367c1f` with
`clippy 0.1.85 (4d91de4e48 2025-02-17)`; every probe was reverted and
`git status --porcelain` is empty.

Every finding is cited `<round>/<label>`. Mine are `r4/…`.

---

## HEADLINE — the fold ran the API check but still did not run the fixture

The r5 fold is three plan hunks and one gate repair. Two of the three hunks are
correct where r3 measured them wrong, the gate repair works and is a genuine
structural improvement, and **all three of the fold's new line citations check
out** (`encode.rs:284` = `fn wpkh_template_only`, `tree.rs:38-56` = the
`Variable`/`MultiKeys` docs, `template.rs:1671`/`:2684` = the two `leaves()`
sites) — the first fold in this cycle whose citations are clean.

But r3's closing sentence asked for the measurable claims to be **run before the
next reviewer sees them**, and the single most measurable claim in the fold was
not run:

- **r3/I-1's replacement fixture is still RED, with the same misdirecting
  message.** The tag/body mismatch is genuinely fixed — `Body::MultiKeys` encodes
  and decodes, exactly as r3 measured. The fold then replaced the invented
  `descriptor_with` with a full struct literal **modelled on
  `wpkh_template_only`**, and copied that model's
  `OriginPath { components: vec![] }`. MEASURED: `tr(_, TapTree)` is in
  `canonical_origin`'s forced-explicit column (`canonical_origin.rs:58`), so an
  empty shared origin is structurally undecodable for this shape.
  `decode_payload` → `Err(MissingExplicitOrigin { idx: 0 })`, and
  `assert!(…is_ok(), "a mint-side refusal reached decode")` fires. → **r4/I-1**
- **r3/I-2's relocation makes its own test uncompilable.** `item_7` now carries a
  comment putting it in `crates/md-cli/tests/`. Its first line calls
  `kind1_from_vector`, which Task 1 defines in **md-codec's**
  `tests/common/liana.rs` and pulls in with
  `include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/common/liana.rs"))`.
  MEASURED from an md-cli test: that concat resolves to
  `crates/md-cli/tests/common/liana.rs`, which does not exist — and neither does
  `vendored.rs`, which `kind1_from_vector` itself depends on. → **r4/I-2**
- **r3/I-3's walk now COMPILES AND RUNS** against miniscript 13.0.0 and yields
  what §2 needs. That finding is closed. Residue is two Minors.

Neither Important needs a design decision. Both are the same shape as the last
three rounds': a code block edited in response to review, never executed.

---

## Q1 — status of EVERY r3 finding

### r3's three Importants

| # | status | evidence |
| --- | --- | --- |
| **r3/I-1** (the r2/I-F fixture is structurally invalid) | **PARTIAL — half fixed, the test is still RED** | The tag/body half is **fixed and confirmed**: `Body::MultiKeys { k: 2, indices: vec![0,1,2] }` under `Tag::SortedMultiA` encodes to 88 bits and decodes `Ok` (measured below, matching r3's PROBE2). `key_arg` and `descriptor_with` are gone from the plan and from the ALLOW list. **But the replacement struct literal's `PathDeclPaths::Shared(OriginPath { components: vec![] })` makes the test fail at decode anyway**, with the identical decode-blaming message. → **r4/I-1** |
| **r3/I-2** (Step 5b re-introduces r2/I-D) | **PARTIAL — the home is now stated, and the stated home cannot compile it** | `:911-913` adds *"LIVES IN crates/md-cli/tests/ — it shells out to the `md` binary … do not re-introduce it here."* That answers the crate-requirement half, and `CARGO_BIN_EXE_md` **is** defined there (measured: `/scratch/code/shibboleth/.tmp/f449-r4-target/debug/md`). Two residues: (i) the test's other dependency, `kind1_from_vector`, is unreachable from md-cli → **r4/I-2**; (ii) Task 8's **Files** line (`:802`) still reads `crates/md-codec/tests/liana_unspendable.rs` (extend) and names no md-cli file for Step 5 **or** Step 5b → **r4/M-3** (this is r2/M12, still open, now contradicting two steps) |
| **r3/I-3** (the §2 walk names a non-existent API) | **ADDRESSED — compiled and RUN** | `:433-437` is now `for leaf in tap_tree.leaves() { leaf.miniscript().for_each_key(…) }`. Measured against miniscript `13.0.0` rev `ff4732e`: compiles, runs, yields 3 key occurrences over a 2-leaf Liana-shaped tree, internal key correctly **excluded**. Residues, both Minor: the trait import is unnamed (**r4/M-1**) and the binding is still `tap_tree`, not `script_tree: Option<…>` (**r4/M-2**, = r3/I-3's point 3) |

### r3's five Minors

| # | status | evidence (measured against the r5 plan) |
| --- | --- | --- |
| **r3/M1** — Task 4's Files line names only md-codec's test file while `:451` relocates the pin to md-cli | **NOT ADDRESSED** | `:343` is byte-unchanged: *"Test: `tests/liana_unspendable.rs` (create, with a `//!` header)"*, under a Files line whose other entries (`lib.rs:30`, `nums.rs`) are md-codec's. No md-cli test file is named anywhere in Task 4. Verified separately that the pin's **body** is fine in md-cli: it uses only `md_err` and string literals, no md-codec test helper |
| **r3/M2** — the integration test needs `use md_codec::use_site_path::{Alternative, UseSitePath};` | **NOT ADDRESSED** | `grep -n "use md_codec::use_site_path"` → no match anywhere in the plan |
| **r3/M3** — `item_2`'s headline `assert_eq!(a, b)` is a tautology | **NOT ADDRESSED** | The block at `:895-908` is byte-unchanged by the fold; two identical calls to the same pure function are still compared |
| **r3/M4** — Step 2b's two tests have no stated home; `encode_payload_inner` takes `(d, admission)` only | **NOT ADDRESSED** | `:778-780` unchanged. Confirmed at source: `fn encode_payload_inner(d: &Descriptor, admission: Admission)` — `encode.rs:143`. "calls `encode_payload_inner` with an overridden version" still requires a third parameter the plan never prescribes adding |
| **r3/M5** — `compressed_33` is undefined and not total over `DescriptorPublicKey` | **NOT ADDRESSED** | `grep -n "fn compressed_33"` over the plan → zero. It survives only at the one call site, `:435`. It is still exempted by ALLOW as a symbol the plan CREATES — see Q3 |

### r3's one Nit

| # | status | evidence |
| --- | --- | --- |
| **r3/N1** — the ALLOW list and the plan disagree about `descriptor_with` | **ADDRESSED, both sides** | `key_arg|descriptor_with` removed from `ALLOW` (`plan-api-check.sh:32`), and the plan's contradicting comment is replaced by *"There is no `descriptor_with` helper and no `Descriptor::default_for_tests` — an earlier draft invented both"* (`:689-690`). The specific loophole that produced the false green is closed |

### Carried from r2/r1 (all still open, none blocking)

`r2/M1`–`M10`, `M12`, `d1`–`d3`, `N1`–`N4` are untouched by this fold; r3's
per-item evidence stands, and I re-ran the five that gate Q4 (see there).
`r2/N1` re-verified at source: `crates/md-codec/tests/internal_key_refactor.rs:30`
asserts `g.tr_count, 23`, so `:165`'s "23" is right and `:985`'s "22" is wrong.

---

## Q2 — the three corrected code blocks, EXECUTED

### (a) the in-crate fixture — **RED. The test still fails, with the same message.** → r4/I-1

Built as the plan prescribes: a `#[cfg(test)]` module **inside
`crates/md-codec/src/encode.rs`** (the plan's stated home — the test needs
module-private `encode_payload_inner` and `pub(crate) Admission::SkipPolicy`),
with the fixture and the test copied verbatim from `:676-711`.

```
$ cargo test -p md-codec --lib f449_r4_probe -- --nocapture

PROBE encode SkipPolicy: OK, 60 bits, 8 bytes
PROBE decode: ERR MissingExplicitOrigin { idx: 0 }
PROBE shape: kind1_root_tr=true sortedmulti_a_leaf=true
PROBE encode Enforce (pre-§6): OK, 60 bits

thread 'encode::f449_r4_probe::a_refused_shape_still_DECODES_so_existing_cards_never_stop_reading'
  panicked at crates/md-codec/src/encode.rs:369:9:
a mint-side refusal reached decode

test result: FAILED. 1 passed; 1 failed; 0 ignored; 249 filtered out
```

Against the three requirements:

| requirement | verdict |
| --- | --- |
| (i) encodes clean under `Admission::SkipPolicy` | **PASS** — 60 bits, no error. `validate_tap_script_tree` runs outside the `Enforce` block (`encode.rs:152-156`) and does not list `SortedMultiA`, exactly as r3 established |
| (ii) `decode_payload` back successfully | **FAIL** — `Err(MissingExplicitOrigin { idx: 0 })` |
| (iii) is the shape §6's `sortedmulti_a` refusal would trip under `Enforce` | **PASS** — `kind1_root_tr=true`, `sortedmulti_a_leaf=true`, and `Enforce` today returns `Ok`, so a new §6 refusal is the only thing that could make it fail |

**Root cause, and it is structural rather than incidental.** The fold's comment
says *"model it on `wpkh_template_only` at encode.rs:284"*. That model carries
`PathDeclPaths::Shared(OriginPath { components: vec![] })`, and it is safe there
**only because `wpkh` has a canonical origin**. `canonical_origin.rs`'s own table
puts this fixture's shape in the other column:

```
crates/md-codec/src/canonical_origin.rs:19   | `tr(@N, TapTree)` | `None` (forced explicit) |
crates/md-codec/src/canonical_origin.rs:58   (Tag::Tr, Body::Tr { tree: Some(_), .. }) => None,
```

and `canonicalize.rs:473-480` raises `MissingExplicitOrigin` precisely when the
baseline path is empty, no override is present, and `canonical_origin` is `None`.
`decode_payload` runs that check (`decode.rs:144-153`); `encode_payload_inner`
does not — which is why encode passes and decode fails. **Every** `tr` with a
taptree and an empty shared origin fails this way; nothing about `SortedMultiA`
or kind 1 is involved.

This is the worst available failure mode for this particular test, for exactly
the reasons r3 gave: the message accuses `decode_payload`, the plan warns four
times that a decode-side refusal makes existing plates unreadable, and an
implementer who believes the message goes hunting in the decode path for a leak
that is not there.

**Measured fix** (same module, same fixture, only `path_decl` changed to a
non-empty shared origin — I used `m/48'/0'/0'/2'`):

```
PROBE3 encode SkipPolicy: OK, 88 bits
PROBE3 decode: OK
PROBE3 encode Enforce: OK, 88 bits
```

88 bits is r3's PROBE2 number, which confirms r3's own probe carried a non-empty
origin and the fold's struct literal is where the regression entered. Any
non-empty shared origin, or a per-`@N` override, satisfies the decoder; I
measured one.

### (b) the tap-tree walk — **COMPILES AND RUNS. Correct.** (one Minor)

The plan's §2 RIGHT block, transcribed verbatim (only `compressed_33` stubbed,
since the plan still does not define it) into an md-codec integration test
against the pinned miniscript, over a Liana-shaped
`tr(NUMS,{multi_a(2,A,B),and_v(v:pk(C),older(26280))})`:

```
PROBE-B walk yielded 3 key occurrences:
PROBE-B internal key (50929b74) in walk? false
PROBE-B depths: [1, 1]
PROBE-B whole-descriptor for_each_key order: ["xpub661MyMwAqRbc", "xpub661MyMwAqRbc",
                                              "xpub6BosfCnifzxc", "50929b74c1a04954"]
test result: ok. 1 passed; 0 failed
```

**Does `for_each_key` exist on what `leaf.miniscript()` returns?** Yes.
`TapTreeIterItem::miniscript()` returns `&'tr Arc<Miniscript<Pk, Tap>>`
(`src/descriptor/tr/taptree.rs:201`), and `for_each_key` resolves through
auto-deref to the `ForEachKey` impl on `Miniscript`
(`src/lib.rs:414`, `fn for_each_key<'a, F: FnMut(&'a Pk) -> bool>`). `leaves()`
exists at `taptree.rs:56`. This matches `parse/template.rs:1671`'s usage
(`for leaf in tt.leaves() { … walk_script_root(leaf.miniscript(), km)? … }`) and
`:2684`'s (`for item in inner.leaves() { item.miniscript().ext_check(…) }`).

**Does it yield what §2 needs?** Yes, on both counts the design rests on:
depth-first left-to-right leaf order (`depths: [1, 1]`, three occurrences in leaf
order), and the internal key **excluded** — the contrast line shows
whole-descriptor `for_each_key` appending `50929b74…` last, which is the ordering
trap `md-cli/src/decompose/walk.rs:36-37` documents. Applying it per-leaf avoids
it. r3/I-3 is closed.

**One measured gap → r4/M-1.** The first compile failed:

```
error[E0599]: no method named `for_each_key` found for struct `Arc<Miniscript<…>>`
  = help: trait `ForEachKey` which provides `for_each_key` is implemented but not in scope;
          perhaps you want to import it
  |
2 + use miniscript::ForEachKey;
```

`ForEachKey` is imported at four sites in the repo — **all four in md-cli**
(`decompose/walk.rs:43`, `parse/template.rs:2535`, `tests/acceptance_walks.rs:414`,
`tests/common/facts.rs:34`) — and **nowhere in md-codec**, which is where this
block lands (`to_miniscript.rs::node_to_descriptor`). The plan must name
`use miniscript::ForEachKey;`. Same class as the unaddressed r3/M2.

**And r3/I-3's third point is still open → r4/M-2.** The block says
`tap_tree.leaves()`; `node_to_descriptor`'s local is
`script_tree: Option<TapTree<DescriptorPublicKey>>` (`to_miniscript.rs:346-350`),
so the `Option` has to be handled. Non-blocking — it will not compile, and the
right name is four lines from the edit — but it was a named sub-finding of a
blocking r3 item and was not folded.

### (c) the relocated tests — **the C2 pin is fine; the fixpoint test cannot compile where it is now sent.** → r4/I-2

**`CARGO_BIN_EXE_md` in md-cli's tests: CONFIRMED.**

```
PROBE-C md-cli CARGO_MANIFEST_DIR = …/descriptor-mnemonic/crates/md-cli
PROBE-C CARGO_BIN_EXE_md = /scratch/code/shibboleth/.tmp/f449-r4-target/debug/md
```

and `grep -rc CARGO_BIN_EXE_md crates/md-cli/tests/` → 10 sites. md-codec has
**zero**, and its `[dev-dependencies]` are `serde`, `serde_json`, `hex`,
`proptest`, `miniscript` — the fold's stated reason is exact.

**The C2 reuse pin (Task 4, `:459-489`) CAN live in md-cli/tests.** Its body uses
only `md_err`, `format!` and string literals — no md-codec test helper. ✓

**`item_7` CANNOT.** Its first line is
`descriptor_to_template(&kind1_from_vector("keyed_compose_tr_nums_three_leaves"))`.
`descriptor_to_template` is fine — md-codec `render.rs:146`, re-exported at
`lib.rs:67`, and md-cli depends on md-codec. `kind1_from_vector` is not:

- Task 1 puts it in `crates/md-codec/tests/common/liana.rs` (`:82`), pulled in
  with `include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/common/liana.rs"))`
  (`:84`).
- MEASURED from an md-cli integration test, that concat resolves to
  `…/crates/md-cli/tests/common/liana.rs` — `exists? false`.
- `kind1_from_vector`'s own dependencies, `load_vendored_phrase` and
  `decode_vendored`, exist at exactly one place in the repo:
  `crates/md-codec/tests/common/vendored.rs:61` and `:93`. MEASURED:
  `…/crates/md-cli/tests/common/vendored.rs exists? false`.
- md-cli's tests reach into md-codec for **data only** — `tests/vectors` via
  `format!("{}/../md-codec/tests/vectors", env!("CARGO_MANIFEST_DIR"))`
  (`vector_corpus.rs:22`, `cmd_descriptor.rs:18`,
  `corpus_origin_consistency.rs:27`). There is **no** cross-crate include of test
  *code* anywhere in the repo: every `#[path = …]` in either crate's tests points
  inside its own `tests/` directory.

So the plan now contains two statements that cannot both hold: Task 1's include
recipe, and Task 8 Step 5b's relocation. This is r3/I-2's own defect class —
a test placed where its dependencies do not exist — reproduced by r3/I-2's fix.
The plan already knows the boundary and says so for the other direction at
`:677-678`: *"tests/common/ is a different crate root, so kind1_from_vector is
not reachable here"*.

**Does md-codec still reference the `md` binary?** The two md-codec-homed blocks
that did are the only ones, and both now carry an explicit md-cli relocation
(`:451-455` for the C2 pin, `:911-913` for `item_7`). No other block under an
md-codec Files line calls `md(…)` or `md_err(…)`: the remaining sites are
`:596`, `:607`, `:614`, `:622` (Task 6, md-cli work) and `:883` (Task 8 Step 5,
which states *"It belongs in md-cli's tests"*). The **Files lines** were not
updated to match any of it — `:343` and `:802` — which is r3/M1 and r4/M-3.

---

## Q3 — the gate's own repair

`./scripts/plan-api-check.sh design/IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md`
now prints `all extracted symbols resolve`, then 22 exemptions, then
`checked 87 candidate symbols from 22 rust blocks`.

**Direct answer: no — none of the 22 is a symbol the plan describes as already
existing.** I checked all 22 against the plan's own prose. The one that was
(`descriptor_with`, *"existing `#[cfg(test)]` helper in encode.rs"*) is gone from
both sides, together with `key_arg`. The specific contradiction that produced a
green gate on an undecodable fixture is closed, and the repair is real: the
listing is what let me clear the other 21 in one pass instead of grepping blind.

The one `[ALSO EXISTS in repo]` flag, `case`, is a **benign collision** —
`md-cli/src/seat/directive.rs:282` and `matching.rs:277` each define a local
`#[cfg(test)] fn case(text: &str) -> Case` for seat-directive tests, unrelated to
Task 1's `fn case(name: &str) -> Case` over `cases.json`. Checked by hand as the
banner asks; not a defect.

**But the repair exposes the loophole's other half, which is still open.** The
banner asserts *"each MUST be a symbol this plan CREATES"*, and for five entries
the plan creates nothing — no definition in any rust block, and for two, no prose
either:

| exempted symbol | in the plan | in the repo |
| --- | --- | --- |
| `compressed_33` | used at `:435`; **no definition, no prose** | absent |
| `md_err` | used at `:482`, `:486`, `:614`, `:883`; **no definition, no prose** | absent |
| `all_nums_tr` | used at `:774`; prose only (*"built in-crate from `Node`/`Body`"*) | absent |
| `wsh_wrapping_tr_liana` | used at `:766`; same prose | absent |
| `encode_payload_at_forced_version` | used at `:775`; prose describes it as *"a `#[cfg(test)]` helper in `encode.rs`"* with a signature that does not fit `encode_payload_inner(d, admission)` (r3/M4) | absent |

`compressed_33` and `md_err` are the sharp ones: an ALLOW entry that is neither
created nor existing is a symbol nobody has to write, and both sit in
funds-relevant or gate-relevant blocks. → **r4/M-4**. (`md_err` also has no
shared home to be written into — 23 md-cli test files each define their own
`fn md()` with three different return shapes, as r3 measured.)

Two Nits on the new output itself: the `exempted` set is computed from `cands ∩
ALLOW` without subtracting `defined`, so symbols the plan **does** define in a
rust block — `all_cases`, `kind1_from_vector`, `wire_version` (`:267`),
`all_kind0_tr_vectors` (`:166`), `in_crate_tr_liana_with_sortedmulti_a_leaf`,
`tr_liana_at_use_site`, `tr_liana_with_sortedmulti_a_leaf` — pad the list they
never needed to be on, diluting the five that matter (**r4/N-1**); and a
resolved collision like `case` has nowhere to be recorded, so it re-costs a hand
check every round (**r4/N-2**).

---

## Q4 — staleness across 995 lines and five folds

**Nothing in the fold contradicts the SPEC.** All three of the fold's new
citations were re-resolved against source and are correct (listed in the
headline). The fixture defect in r4/I-1 is a *missing* fact, not an internal
contradiction: the plan says nothing anywhere about the explicit-origin rule —
`grep -n "MissingExplicitOrigin\|explicit origin\|canonical_origin\|OriginPathOverrides"`
over the plan returns zero.

**Live contradictions — all eight of r3's are carried unchanged, and the fold
added a ninth:**

1. `:165` "23 of the 65" vs `:985` "22 of the 65". **23 is right** — re-verified at
   `md-codec/tests/internal_key_refactor.rs:30`, `assert_eq!(g.tr_count, 23)`. (r2/N1)
2. `:513` "**55 call sites**" vs `:515` "**ALL TWELVE production callers — G-7
   undercounted by nine**", nine lines apart. (r2/N2)
3. `:653` "Write the failing tests — **as UNIT tests inside `encode.rs`**" sitting
   directly above the two-home table that puts two of the three in `tests/`. (r2/d1)
4. `:649` Task 7's Files line omits `crates/md-codec/tests/liana_unspendable.rs`,
   which the table beneath it requires. (r2/d2)
5. `:751` Task 7 Step 2's `cargo nextest run --locked -p md-codec encode::` cannot
   match the two relocated integration tests. (r2/d3)
6. `:802` Task 8's Files line (md-codec only) vs Step 5's *"It belongs in md-cli's
   tests"* (`:880`) **and** Step 5b's new *"LIVES IN crates/md-cli/tests/"*
   (`:911`). (r2/M12, now contradicting two steps — **r4/M-3**)
7. `:343` Task 4's Files line (md-codec only) vs `:451`'s *"This pin lives in
   `crates/md-cli/tests/`"*. (r3/M1)
8. `:282` `chunk.rs:70`/`:373` "accept `{4, 8}`" — `:368-377` is the
   version-agnostic cross-chunk consistency loop and needs no change. (r2/M7)
9. **NEW, from this fold:** Task 1's
   `include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/common/liana.rs"))`
   (`:84`) vs Task 8 Step 5b's *"LIVES IN crates/md-cli/tests/"* (`:911`). The two
   cannot both hold. → **r4/I-2**

**Resolved by this fold:** the `descriptor_with`-existing claim; the
`Body::Variable` tag/body mismatch; the `TapTree::iter()` spelling; the ALLOW
list's `key_arg|descriptor_with` entries.

The seven carried Minors and Nits at items 1-5 and 8 are one editing sweep, and
five rounds have now each declined to take it. They do not gate.

---

## Blocking findings

**r4/I-1 (Important) — the funds-regression fixture is still RED, and still
blames the decode path.** `in_crate_tr_liana_with_sortedmulti_a_leaf` (`:676-703`)
builds `PathDeclPaths::Shared(OriginPath { components: vec![] })`, copied from
`wpkh_template_only` per the fold's own comment. `wpkh` has a canonical origin;
`tr(@N, TapTree)` is explicitly in `canonical_origin`'s forced-explicit column
(`canonical_origin.rs:19`, `:58`), so `canonicalize.rs:473-480` raises
`MissingExplicitOrigin` for every such descriptor with an empty baseline path.
MEASURED at `37367c1f`, the fixture built in-crate in `encode.rs` exactly as
prescribed: `encode_payload_inner(SkipPolicy)` → `Ok`, 60 bits;
`decode_payload` → `Err(MissingExplicitOrigin { idx: 0 })`;
`a_refused_shape_still_DECODES_so_existing_cards_never_stop_reading` **FAILED**
with `a mint-side refusal reached decode`. The tag/body half of r3/I-1 is
correctly fixed and `Body::MultiKeys` is confirmed to round-trip; only the origin
path is wrong. Measured fix: any non-empty shared origin — with
`m/48'/0'/0'/2'` the same fixture encodes to 88 bits and decodes `Ok` under
`SkipPolicy`, and still encodes `Ok` under `Enforce` today, so the §6 refusal
being added is the only thing that can turn the MINT test red.
*(Residual of r3/I-1 / r2/I-F.)*

**r4/I-2 (Important) — `item_7`'s new home cannot reach the helper it calls.**
Step 5b's fold comment (`:911-913`) sends
`item_7_the_render_reparse_fixpoint_covers_tr_kind_1` to
`crates/md-cli/tests/`, correctly, because it shells out to the `md` binary. Its
first line then calls `kind1_from_vector`, which Task 1 defines in **md-codec's**
`tests/common/liana.rs` (`:82`) and includes with
`include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/common/liana.rs"))`
(`:84`). MEASURED from an md-cli integration test: `CARGO_MANIFEST_DIR` is
`…/crates/md-cli`, so that include resolves to
`…/crates/md-cli/tests/common/liana.rs` — `exists? false`; and
`…/crates/md-cli/tests/common/vendored.rs`, which `kind1_from_vector` transitively
needs (`load_vendored_phrase` at `md-codec/tests/common/vendored.rs:61`,
`decode_vendored` at `:93`, the only definitions in the repo) — `exists? false`.
No md-cli test in the repo includes test *code* from md-codec; the three that
cross the boundary read `tests/vectors` **data** by relative path. The plan
already states this boundary at `:677-678` for the opposite direction. Task 4's
relocated C2 pin is **not** affected — checked, its body uses only `md_err` and
literals. Fix is a decision the plan must state: either give `item_7` a
`#[path = "../../md-codec/tests/common/liana.rs"]`-style include (and say so at
`:84`, whose current recipe forbids it), or drop the `kind1_from_vector` call and
build the template through `md` itself, or leave the test in md-codec and reach
the binary some other way. *(Residual of r3/I-2 / r2/I-D.)*

---

## Non-blocking findings (new this round)

- **r4/M-1** — the §2 walk needs `use miniscript::ForEachKey;`. MEASURED
  `error[E0599]` without it; the trait is imported at four sites, **all in
  md-cli**, and nowhere in md-codec, which is where the block lands
  (`to_miniscript.rs`). Same class as the still-open r3/M2.
- **r4/M-2** — the block still says `tap_tree.leaves()`;
  `node_to_descriptor`'s local is `script_tree: Option<TapTree<DescriptorPublicKey>>`
  (`to_miniscript.rs:346-350`), so the `Option` needs handling. This was r3/I-3's
  point 3 and was not folded with points 1 and 2.
- **r4/M-3** — Task 8's Files line (`:802`) names only
  `crates/md-codec/tests/liana_unspendable.rs` while Step 5 (`:880`) and Step 5b
  (`:911`) both put tests in md-cli. r2/M12, now contradicting two steps. Pairs
  with the still-open r3/M1 at `:343`.
- **r4/M-4** — five ALLOW entries are exempted as *"symbols this plan CREATES"*
  and the plan creates none of them: `compressed_33` and `md_err` have no
  definition **and no prose**; `all_nums_tr`, `wsh_wrapping_tr_liana` and
  `encode_payload_at_forced_version` have one line of prose and no body. The
  repaired gate makes this visible for the first time; it is the other half of the
  loophole r3/N1 closed.

- **r4/N-1** — the new `EXEMPTED BY ALLOW` listing does not subtract `defined`,
  so seven symbols the plan does define in a rust block pad the list of 22 and
  dilute the five that matter.
- **r4/N-2** — `case`'s `[ALSO EXISTS in repo]` flag is benign
  (`md-cli/src/seat/{directive,matching}.rs`, unrelated local test helpers) but
  there is nowhere to record that it was checked, so it re-costs a hand check
  every round.

---

## Per-task soundness after the fold

- **Task 1 — UNSOUND** (**r4/I-2**'s other half: the include recipe at `:84` is
  what Step 5b's relocation contradicts; plus r2/M8, r2/N4, r2/M10 residue).
- **Task 2 — SOUND** (r2/M7, r2/N3 only).
- **Task 3 — SOUND.**
- **Task 4 — SOUND in substance** (**r4/M-1**, **r4/M-2**, r3/M1, r2/M9); the
  per-occurrence ruling is correct, and its prescribed walk now RUNS. This is the
  first round in which Task 4's code executes.
- **Task 5 — SOUND in design** (r2/M2, r2/N2).
- **Task 6 — UNSOUND** (reserved-synthetic-key mechanism gap, r2/M1, r2/M5).
- **Task 7 — UNSOUND** (**r4/I-1**, r3/M2, r3/M4, r2/d1-d3).
- **Task 8 — UNSOUND** (**r4/I-2**, **r4/M-3**, r3/M3, r2/M4).
- **Task 9 — SOUND as a gate.**
- **Task 10 — SOUND except r2/M6.**

---

**Ready for implementation: no.** Two Importants block. Both are mechanical,
neither needs a design decision, and both are the same recurring shape — a block
the fold edited and did not run:

- **r4/I-1** — give the in-crate fixture a non-empty origin path. MEASURED:
  `PathDeclPaths::Shared(OriginPath { components: m/48'/0'/0'/2' })` → encode 88
  bits, decode `Ok`. Drop or qualify the *"model it on `wpkh_template_only`"*
  sentence, which is what carried the empty path across; `wpkh` may elide an
  origin and `tr(@N, TapTree)` may not.
- **r4/I-2** — decide where `item_7` lives and make Task 1's include recipe
  (`:84`) agree with it. If it stays in md-cli, the plan must say how
  `kind1_from_vector` gets there; MEASURED, the `CARGO_MANIFEST_DIR` spelling
  cannot.

Then re-run `./scripts/plan-api-check.sh` and hand-clear the five ALLOW entries
the plan does not create (r4/M-4) — `compressed_33` and `md_err` first, since
neither has so much as a sentence prescribing it. The Minors and Nits do not
block, but r3 already noted that r2's editing sweep (the `23`/`22` count, the
`encode::` filter, the two Files lines, Task 7's Step 1 heading) is one pass
that five rounds have each deferred; it is cheaper folded with these two than
carried into a sixth.

One note on the round's shape, echoing r3's. The three blocks r3 could not
execute were the whole of this round, and running them cost under two minutes of
compile time apiece. Two were still wrong. The gate repair in this fold is the
right instinct applied to the wrong half of the problem: it now shows which
symbols were *exempted*, but nothing in the pipeline yet **runs a fixture**.
A plan whose test fixtures are built and round-tripped by a script — even once,
even by hand — would have caught r4/I-1 before a reviewer was engaged, as it
would have caught r3/I-1 and r2/I-F before that. Three rounds, one class, each
time one level inside the previous fix.
