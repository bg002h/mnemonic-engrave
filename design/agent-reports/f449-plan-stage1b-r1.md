# R1 scoped re-review — `IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md` (r1 fold)

**Verdict: 1 Critical / 5 Important / 11 Minor / 4 Nit — NOT GREEN.**

Reviewer: independent agent, opus. Scope: the r1 fold only (Q1 fold-check, Q2's
five new/unreviewed rulings). Spec read for authority, not re-reviewed; where
plan and spec disagree the plan is charged. Repo read and RUN at
`descriptor-mnemonic` `37367c1f`. Diff read at
`c342a3b0..81a8adbf`.

Every command below was executed and every line citation resolved against the
real source. Measurement correction to the dispatch brief: the plan carries
**50** `- [ ]` steps, not 43 (`grep -c '^- \[ \] \*\*Step'`); the per-task range
4–7 is correct (1:4 2:5 3:5 4:4 5:6 6:7 7:5 8:6 9:4 10:4).

---

## Q1 — fold checklist

### Criticals

**C1 — wrong `sortedmulti_a` fixture: ADDRESSED.** Task 7 now uses
`keyed_compose_tr_sole_sortedmulti_a`. Verified from the corpus: its
`.template` is
`tr(50929b74…03ac0,sortedmulti_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/…,@2/…))`, its
`.descriptor.json` tree is `is_nums: true` with a sole `SortedMultiA` child, and
`pubkeys` is present. `kind1_from_vector`'s `assert_eq!(*internal_key,
InternalKey::NumsPoint)` passes on it. Verified further that the test can fail
today: `is_forbidden_leaf_tag` (`validate.rs:271-278`) lists `Wpkh | Tr | Wsh |
Sh | Pkh | Multi | SortedMulti` and **not** `SortedMultiA`, so `encode_payload`
returns `Ok` for this fixture until the new refusal exists. The doc comment
recording *why* `keyed_tr_sortedmulti_a` is the wrong one is accurate.

**C2 — per-slot vs per-occurrence: PARTIAL.** The refusals are real (reproduced
below, Q2a) and the conclusion survives for everything `md` itself mints. Two
halves did not land: the fold's licence sentence is broader than its proof
(→ **I-5**), and the pin it rests on cannot fail (→ **C-1**). C2's *second,
compounding gap* — no route from what `to_miniscript.rs` holds to the 33-byte
account pubkeys, and the internal key built before the script tree — is **NOT
ADDRESSED**; see I-5.

**C3 — Task 8 cannot construct the four ACCEPT shapes: ADDRESSED as a ruling,
but the promised replacement was not written.** The scope move is sound (Q2b);
the sentence "In 1b: every §8 property is proven over md's own vendored corpus"
is not implemented for §8 items 2 and 7 (→ **I-3**).

**C4 — refusals never wired, no test that any refuses: PARTIAL.** Step 4 wires
`validate_unspendable_shape(d)?` into the `Admission::Enforce` block and Task
7's Files now name `encode.rs:143-170` — both correct. Test 1
(`kind_1_with_a_sortedmulti_a_leaf_is_refused_at_MINT`) is the missing control
C4 asked for. But only **two of §6's four encode-side refusals** get one
(→ **I-2**).

### Importants

**I1 — the test cannot compile where it must live: PARTIAL.** The file-location
half is fixed and correctly grounded: `encode.rs:143` is `fn
encode_payload_inner` (module-private, verified), `:139` is `pub(crate) fn
encode_payload_for_identity`, `Admission` is `pub(crate)` at `:107` — so the
unit-test-in-`encode.rs` ruling is right, and Task 7 Step 1 now argues from the
module-private function rather than from G-6's over-stated `Admission` reason.
The **second half is untouched**: the tests call `kind1_from_vector`, defined by
Task 1 in `crates/md-codec/tests/common/liana.rs` → **I-1**.

**I2 — "remove that attribute": ADDRESSED.** Task 1 Step 2b now says *"Leave its
`#[allow(dead_code)]` alone"* and names the reason. Matches
`tests/common/vendored.rs:32-36` verbatim.

**I3 — `substitute_synthetic` NOT `walk_tr`: ADDRESSED.** Task 6 now says
**both**, with the division of labour spelled out. Verified: `walk_tr` at
`parse/template.rs:1589`, `let key_str = t.internal_key().to_string();` at
`:1593`, the `NUMS_H_POINT_X_ONLY_HEX` comparison and `NumsPoint` return at
`:1599-1605`, `lookup_key` → `InternalKey::Slot` at `:1609-1632`. The plan's
claim that it is the only place the kind is decided is true.

**I4 — caller enumeration wrong by eight: ADDRESSED.** See Q2d — the twelve-row
table is exact.

**I5 — §8.1's nested taptree that Liana ACCEPTS: PARTIAL.** Task 8 Step 4
answers a required **new vector** with a required **comment**. It is parked in
no stage → **I-4**.

**I6 — §6's pinned invariant (SELF-2): ADDRESSED; its second half is not.**
Task 8 Step 5 writes the pin and correctly relocates it to md-cli. Its assertion
string is verified against the live message:
`md encode "tr(50929b74…03ac0)"` → `md: template parse error: template contains
no @i placeholders` (RUN), which contains `"no @i placeholders"`. The second
half — §6 row 3's `--unspendable liana` no-op **WARN** (fable M-6) — is in no
task and in no "owned by later stages" entry (`grep -n "WARN\|no-op" plan` → no
hits). **NOT ADDRESSED** (recorded as Minor, since §9 plausibly owns the flag at
stage 2 and the plan simply never says so).

**I7 — Tasks 6-9 have no steps: ADDRESSED.** 50 steps total, Tasks 6/7/8/9 now
carry 7/5/6/4. Each ends in a gate-and-commit step.

### Minors

| # | status | evidence |
| --- | --- | --- |
| M1 Task 6 pulls two stage-2 items in, undeclared | **PARTIAL** | The JSON-schema break is now versioned ("do not slip it in"), but neither it nor the `UNSPENDABLE(liana)` substitution rule is declared as pulled forward from §9's stage-2 content column |
| M2 Task 5 does not name the new `KeyPathKind` variant | **NOT ADDRESSED** | `:477` still reads "maps to the new `KeyPathKind`"; `grep -n NumsXpub plan` → no hits, while §7 names it and gives its reason |
| M3 Task 9 omits §8.10's seventh named mutation | **NOT ADDRESSED** | `grep -n "KeyPathSpendable\|policy_id_stub" plan` → no hits; rows 12-15 are still the or-pattern collapse |
| M4 Task 8 omits §8.5's mk1 `policy_id_stub` leg | **NOT ADDRESSED** | `grep -n "mk1" plan` → no hits; §9 assigns §8.5's Rust leg to 1b |
| M5 the generated vector corpus is not regenerated | **NOT ADDRESSED** | No step mentions `md vectors --out`. Measured escalation risk: every `tr` vector's `.descriptor.json` currently carries `"is_nums": true/false, "key_index": n`, a two-state field that cannot express Task 6 Step 6's third state — so the natural implementation drifts all 23 and turns `md-cli/tests/vector_corpus.rs`'s `diff -r` red inside Task 6's own gate |
| M6 Task 10 Step 1's second text is in another repo | **NOT ADDRESSED** | `:771` still names `DESIGN_coordinator_compatibility.md:155-158` with no repo; the File Structure table still omits it; `grep -n mnemonic-engrave plan` finds only Task 1's script argument |
| M7 `chunk.rs:373` needs no change | **NOT ADDRESSED** | `:270` still says "`chunk.rs:70`/`:373` accept `{4, 8}`". Verified `chunk.rs:370-377` is the cross-chunk consistency loop comparing `h.version != expected_version` — already version-agnostic. `chunk.rs:70` is the real site (`if version != Header::WF_REDESIGN_VERSION`) |
| M8 `kind1_from_vector` unguarded against keyless vectors | **NOT ADDRESSED** | The Step 2 body is unchanged; no `pubkeys` assert |
| M9 Task 4's structure test never runs production code | **NOT ADDRESSED, and now load-bearing** | `:365` still maps `case(n).expected_xpub`, comparing three `cases.json` strings to each other. Task 8 Step 4 now leans on this exact test as the pin for nested-taptree traversal order |

Three further Minors are **new in the fold** — see the Minor list under Q2e.

### Nits

| # | status | evidence |
| --- | --- | --- |
| N1 two different `tr` counts | **NOT ADDRESSED** | `:153` says 23, `:792` says 22. 23 is right: `internal_key_refactor.rs:30` asserts `g.tr_count == 23`, and 23 of the 65 `.descriptor.json` files are `tr`-rooted (measured) |
| N2 "55 call sites" is high | **NOT ADDRESSED** | `:444` unchanged |
| N3 wrong recursive fn, missing import | **NOT ADDRESSED** | `read_node` is `pub fn read_node(r, key_index_width)` at `tree.rs:205`, delegating to `fn read_node_with_depth(…, depth)` at `:212` — the recursive one the version must thread through. The plan names only `read_node` / "both node fns". The `use crate::header::Header;` gap is still unmentioned |
| N4 the four refused cases have no addresses | **PARTIAL** | Step 1 now says all eight are vendored and why, but `Case` still declares `liana_receive: Vec<String>` / `liana_change: Vec<String>` non-optional and Step 1 still says `liana_receive[3]`; nothing tells the generator to emit `[]` for the four |

---

## Q2 — the five new rulings

### (a) The C2 ruling — verified, then attacked

**Reproduced, and the fold's two refusals are real.** With the literal NUMS
hex (RUN, `md` 0.17.0, matching `crates/md-cli/Cargo.toml:3`):

```
$ md encode "tr(<H>,{pk(@0/<0;1>/*),and_v(v:pk(@0/<0;1>/*),older(26280))})"
md: unsupported: @0 appears at 2 use sites in this template with the same path
    expression … forbidden by BIP 388's disjointness rule …

$ md encode "tr(<H>,{pk(@0/<0;1>/*),and_v(v:pk(@0/<2;3>/*),older(26280))})"
md: unsupported: @0 appears at use sites with DISJOINT multipath sets — <0;1>
    and <2;3> … md1 deliberately cannot express it … (F-417)
```

(`<H>` = `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`.)
The control passes: `tr(<H>,{pk(@0/<0;1>/*),pk(@1/<0;1>/*)})` mints
`md1ypfdsssj5qqcr3fg2splu967h79446d`.

**A third surface confirms the fold.** `md decompose` refuses repeated key
expressions before numbering — `decompose/walk.rs:170-174`: *"the caller refuses
every repeated key BEFORE numbering (see `mod.rs`), so each rendered expression
occurs exactly once"*, with `mod.rs:245-306` carrying the rule. So neither of
md's two mint routes can put one slot's pubkey twice into §2's sequence. Two
further shapes I probed collapse into the same two refusals: a slot repeated
*inside one leaf* (`multi_a(2,@0,@0)`) is the same-path case, and two slots
holding the same xpub is not the ambiguity at all (both readings yield two
entries).

**Where the claim over-reaches.** The refusals live **entirely in md-cli**
(`parse/reuse.rs:282-311`). md-codec has no reuse check at any layer:

- `validate_placeholder_usage` (`validate.rs:17-36`) requires each `@i` *"at
  least once"* and that first occurrences ascend — nothing more.
- `encode_payload_inner`'s `Admission::Enforce` block (`encode.rs:165-170`) adds
  only `validate_origin_key_consistency` and `validate_no_duplicate_key_slots`;
  the latter (`validate.rs:372-392`) compares *slot xpubs to each other*, never
  a slot to its own use count.

So a `Descriptor` carrying `@0` in two leaves encodes and decodes through
md-codec unchallenged. "md1-expressible" and "md-cli-mintable" are not the same
set, and md1 is a wire format with a Go port and third-party writers. **Answer
to the brief's question: no md1-expressible descriptor *minted or decomposed by
md* can do it — but the codec layer, where §2's derivation will live, admits
one.**

**Is the pinning test the right instrument? No — as written it cannot fail.**
The templates the fold prints use `<NUMS>`, which is not an accepted key
spelling anywhere in the source (`grep -rn '<NUMS>' crates/*/src` → doc comments
only). Measured:

```
$ md encode "tr(<NUMS>,{pk(@0/<0;1>/*),pk(@1/<0;1>/*)})" --path bip48
md: template parse error: miniscript parse failed: key too short
```

The reuse check runs *before* the miniscript parse, so today those commands do
print the refusals — but `assert!(md_encode(tpl).is_err())` is satisfied by the
parse failure alone. **Relax either refusal and the pin stays green while every
kind-1 chain code changes silently** — precisely the event the fold says the pin
exists to catch. → **C-1**.

Two further problems with the instrument, independent of the spelling: it
asserts `is_err()` rather than the refusal's identity (any error passes), and it
is placed in Task 4, whose test file is `crates/md-codec/tests/
liana_unspendable.rs` — md-codec's `[dev-dependencies]` are `serde`,
`serde_json`, `hex`, `proptest`, `miniscript` and **not md-cli**, so it cannot
invoke `md` at all. Task 8 Step 5 relocates its own pin to md-cli for exactly
this reason; Task 4's is not relocated.

### (b) The C3 scope ruling — sound, but the 1b half was dropped

**The split is sound and the spec does authorise it.** §9's double-gate table is
explicit: *"2 descriptor equality | 1b (codec) **and** 2 (through the CLI) —
deliberate"*. And the fold is right that the evidence shapes are out of reach in
1b: `cases.json` carries no md1 phrase and no kind-0 descriptor string, the only
Descriptor builder is `kind1_from_vector`, and `internal_key_refactor.rs:3-4`
records that md-codec has no template parser.

The address-equality half needs no authorisation at all: §9's table assigns only
*"3 address equality — **device leg** | 4"*, and 1b's gate row lists items
"1, 2, 5, 6, 7, 10" — item 3 was never a 1b gate. The r0 finding was the plan's
own over-scoping, and removing it is correct. (§9 does not assign item 3's
md-vs-evidence legs to stage 2 either; the plan's "Record this in stage 2's
brief" is the right mitigation for a hole the spec left.)

**What the ruling authorises is a change of layer, not a removal — and the
replacement was not written.** The fold promises *"In 1b: every §8 property is
proven over md's own vendored corpus via `kind1_from_vector`"*, and then Task 8's
six steps prove round-trip+version, identity distinctness, identity stability,
the dispatch round trip, a comment, and the empty-concat pin. Nothing asserts a
rendered descriptor string for a kind-1 descriptor at any layer — `grep -n
checksum plan` returns only `cases.json` field names. Item **7** (render-reparse
fixpoint extended to `tr` kind 1) fared worse: pre-fold Task 8 named it
(`git show c342a3b0:…` → *"and the render-reparse fixpoint extended to `tr`
kind 1"*); r1 contains the word **zero times** (`grep -n fixpoint plan`). Both
items are spec-assigned to 1b and *"every leg runnable in Rust alone"* is true of
both — the fixpoint corpus is `format/text.rs:255-284`, md-cli, reachable once
Tasks 5 and 6 land. The Self-Review's line *"§8 items 1/2/5/6/7 → Task 8"* is
unchanged and is now false for two of the five. → **I-3**.

**Does anything in 1b's gate become unfalsifiable?** Not from C3 itself:
mutation 5 ("render kind 1 as the NUMS hex") lost its second catcher in the
table but Task 5 Step 1's rendering tests still catch it. The unfalsifiable rows
are 8-11, and they come from C4's residue, not from C3 → **I-2**.

### (c) Task 7's wiring — the wiring compiles, the tests do not

**The wiring is correct.** `crate::validate::validate_unspendable_shape(d)?`
placed beside the existing pair inside `if admission == Admission::Enforce`
(`encode.rs:165-170`) type-checks: `d` is `&Descriptor` after the
canonicalisation rebind at `:144-146`, and the block already calls two
`Result`-returning validators the same way. Putting it there and not in
`decode_payload_with_opts` is right, and it is after canonicalisation, so the
shape it inspects is the shape that gets written.

**The tests do not compile where the plan puts them, for two independent
reasons.**

1. `kind1_from_vector` (and therefore `tr_liana_with_sortedmulti_a_leaf`) is
   defined by Task 1 in `crates/md-codec/tests/common/liana.rs`, which must
   `include!` stage 1a's `tests/common/vendored.rs` for `load_vendored_phrase` /
   `decode_vendored`. That file's line 20 is `use md_codec::Descriptor;`, and
   `lib.rs` has no `extern crate self as md_codec` (verified: `grep -n "extern
   crate self" crates/md-codec/src/lib.rs` → empty; `policy_shape.rs:229-231`
   states the rule). Splicing it into `src/encode.rs`'s `#[cfg(test)] mod tests`
   fails to resolve `md_codec`. The plan says the tests live in `encode.rs` and
   never says how the fixture gets there.
2. `UseSitePath::parse` does not exist. `crates/md-codec/src/use_site_path.rs`
   exposes exactly `standard_multipath()`, `write()` and `read()` on
   `UseSitePath` (plus `write`/`read` on `Alternative`); there is no `FromStr`
   and no `parse` anywhere in the crate. `tr_liana_at_use_site` is a compile
   error as written. (The value is constructible — `UseSitePath { multipath:
   Some(vec![Alternative{hardened:false,value:2}, Alternative{…value:3}]),
   wildcard_hardened: false }` — so this is a spelling defect, not a design one.)

→ **I-1**.

**Can each of the three tests fail?**

- Test 1 (`…sortedmulti_a…refused_at_MINT`) — **yes**, once it compiles:
  `SortedMultiA` is not in `is_forbidden_leaf_tag`, so `encode_payload` returns
  `Ok` today.
- Test 2 (`a_refused_shape_still_DECODES`) — **yes**, and only because test 1
  exists. On its own it is the vacuous assertion C4 identified; the pair is the
  correct control-plus-claim shape, matching
  `mint_policy_does_not_reach_decode.rs:89-98`'s precedent.
- Test 3 (`kind_1_off_the_canonical_use_site_is_refused`) — **cannot be
  determined**: it does not compile. Assuming the fixture is written correctly,
  it can fail (nothing rejects a non-`<0;1>` use-site under `tr` today). Note
  that all 23 corpus `tr` vectors carry the standard `<0;1>` use-site (measured
  across the 23 `.descriptor.json` files), so the fixture must construct the
  divergence rather than find it — which is what the plan intends.

**What is missing.** §6's encode-side refusals are four: (1) a `sortedmulti_a`
leaf, (2) a use-site other than `<0;1>`, (4) kind 1 nested under `sh`/`wsh`,
(6) version 8 over a kind-0 or `tr`-less tree. Task 7 prescribes tests for (1)
and (2). Task 9's mutation row 8-11 says *"drop each of §6's four refusals |
must be caught by Task 7"*. Two of those four mutations have no test that could
turn RED → **I-2**.

### (d) Task 5 Step 4's twelve-caller table — complete and exact

`grep -rn "to_miniscript_descriptor(\|to_miniscript_descriptor_multipath("
crates/*/src` returns 14 lines; two are the definitions
(`to_miniscript.rs:51`, `:263`). The remaining 12 match the table **line for
line**: `disposition.rs:308,434,437`; `vectors.rs:193`;
`descriptor.rs:200,201`; `matching.rs:304,335,454`; `derive.rs:134`;
`compose.rs:403,406`. **Enumeration complete.**

The three "switch" decisions check out: `descriptor.rs:200-201` has
`args.network`; `derive.rs:134` sits inside `derive_address(chain, index,
network)` with `network` consumed at `:156` (`definite.address(network)`) — the
plan's `:155` is one line off and harmless; `vectors.rs:193` has a network, in
the sense that the generator is hard-coded mainnet at `:50` and `:199`, so
`_with_network(d, chain, Network::Bitcoin)` is a faithful substitution.

The nine "leave on the refusing entry point" decisions are **right**. All nine
are `seat/*` comparison paths — `matching.rs` matches seatings, `compose.rs:403-406`
compares two composed descriptors as strings, `disposition.rs` splits and
re-seats — none renders for an operator, and none has a network. Fail-closed is
the correct outcome there: comparing a kind-1 descriptor as a mainnet-rendered
string is the silent-wrong-answer failure mode. Reachability is also low in this
stage: the composer maps a missing internal key to `InternalKey::NumsPoint`
(`compose/tr.rs:46-48`), so nothing in `seat/*` can manufacture kind 1; only a
card minted by Task 6's marker route could reach them, and building such a card
does not go through `seat/*`. The prescribed test
(`a_kind_1_descriptor_is_REFUSED_by_the_network_less_entry_point`) makes the
refusal a gate rather than an accident, which is the right instrument here.

### (e) Tasks 6-9's new steps — executable, with three mechanism gaps

Most of the 24 new steps are executable as written: they name the file, the
line, the direction of the change and end in a gate-and-commit. Task 9 Step 1
("put a `panic!` in the mutated branch first and watch the suite fail") is a
genuine improvement — it closes the mutation-applied-but-inert hole. Task 8
Step 2's `IdGolden` shape is correct: `tests/golden/pre_refactor_ids.json` is
`{count, vectors}` with 65 entries of `[name, policy_id, template_id, phrase]`
(measured), the file exists, and it has **no consumer today**, so the test is
new coverage rather than a duplicate.

Three steps still describe an outcome without showing how:

1. **Task 5 Step 5, the 33-byte pubkeys (r0's C2 second half, unaddressed).**
   "`to_miniscript.rs` builds the derived xpub from the already-built leaf
   miniscripts in tap-tree order" is an outcome. At the site
   (`to_miniscript.rs:333-352`) the internal key is built at `:340-345`
   **before** the script tree at `:346-350`, so the order must be inverted or a
   separate tree walk added; and the only 33-byte source in scope is `keys: &[
   DescriptorPublicKey]`, indexed **by slot**. Iterating `keys` is the one-liner;
   walking `tree` to collect leaf key indices in DFS order is the step the plan
   requires and does not describe. This is where I-5 actually bites.
2. **Task 6 Step 3, the reserved synthetic key.** "rewrites
   `UNSPENDABLE(liana)` to a reserved synthetic key" does not say what key, nor
   how it avoids colliding with `substitute_synthetic`'s existing
   `sha256(b"md-v0.15" ‖ i ‖ depth)` placeholders, nor that `walk_tr` must test
   for it **before** `lookup_key` (which errors on anything not in `km`).
3. **Task 8 Step 3.** "Through `decode_with_correction`, single-string and
   chunked" — the producer is named (`chunk::split`, G-4) but no code and no
   assertion shape. Acceptable as a step; weakest of the six.

**New Minors introduced by the fold:**

- **Undefined helpers, while the Self-Review asserts none exist.** `:800` says
  *"Every helper this plan calls is defined by it … or by stage 1a"*. Measured
  undefined: `md_encode` (`:418`), `SAME_PATH_REUSE_TEMPLATE` /
  `DISJOINT_PATH_REUSE_TEMPLATE` (`:417`), `md` (`:527`, `:538`, `:553`),
  `md_err` (`:545`, `:729`), `ORIGINLESS_SPENDABLE_TR` (`:538`), `IdGolden`
  (`:701`), `UseSitePath::parse` (`:629`). This is the same sentence, asserted
  again, with nine new counterexamples — and `plan-api-check.sh` did not catch
  them, so the blind-spot note at `:787-789` needs widening, not just repeating.
- **Task 6's `the_marker_PARSES_in_a_template_and_yields_kind_1` cannot pass as
  written.** `md encode` prints three lines (measured: the md1 string, the
  grouped form, `group size: 5`), so `decode_md1_string(md1.trim())` receives a
  multi-line string. The helper must take the first line.
- **Task 8's Files line contradicts its Step 5.** Files says
  `crates/md-codec/tests/liana_unspendable.rs` only; Step 5 correctly relocates
  the pin to md-cli.

---

## Blocking findings

**C-1 (Critical) — the pin the C2 ruling rests on cannot fail.** Task 4 Step
3's `a_slot_cannot_appear_twice_in_a_taptree…` asserts `md_encode(tpl).is_err()`
over templates spelled `tr(<NUMS>,…)`. `<NUMS>` is not an accepted key spelling
(`grep -rn '<NUMS>' crates/*/src` → doc comments only; RUN:
`md encode "tr(<NUMS>,{pk(@0/<0;1>/*),pk(@1/<0;1>/*)})"` → *"miniscript parse
failed: key too short"*). The assertion is satisfied by the parse failure, so
relaxing either reuse refusal — the exact event the fold says the pin guards —
leaves it green while every kind-1 chain code changes. Compounding: `is_err()`
does not name the refusal, and the test is placed in an md-codec test file whose
crate cannot invoke md-cli (`[dev-dependencies]` measured: serde, serde_json,
hex, proptest, miniscript).

**I-1 (Important) — Task 7's prescribed tests do not compile.** (i)
`kind1_from_vector` lives in `tests/common/liana.rs`, which must splice
`tests/common/vendored.rs:20` (`use md_codec::Descriptor;`) into
`src/encode.rs`; `lib.rs` has no `extern crate self as md_codec`. (ii)
`UseSitePath::parse` does not exist. r0's I1 second half, unclosed.

**I-2 (Important) — two of §6's four encode-side refusals have no test, so
mutations 8-11 cannot all turn RED.** Task 7 tests the `sortedmulti_a` leaf and
the non-canonical use-site; §6's `sh`/`wsh`-nesting refusal and its
minimum-version refusal get neither a test nor a fixture, while Task 9 maps all
four mutations to Task 7.

**I-3 (Important) — §8 items 2 and 7 have no task in 1b, while the Self-Review
claims Task 8 covers them.** §9 assigns both to stage 1b ("every leg runnable in
Rust alone") and double-gates item 2 at 1b *and* 2. The fold's C3 ruling moved
item 2's evidence leg to stage 2 and promised a corpus-level replacement that no
step implements; item 7 (the render-reparse fixpoint, present in the pre-fold
Task 8) vanished silently — `grep -n fixpoint` returns nothing. `:783` still
reads "§8 items 1/2/5/6/7 → Task 8".

**I-4 (Important) — §8.1's "nested taptree that Liana ACCEPTS" is answered with
a comment and parked nowhere.** Spec §8.1 requires it as a **new vector**, stage
1b; §2 flags nested traversal order as one of exactly three unmeasured gaps.
Task 8 Step 4 substitutes a comment on a test that, per M9, compares two
`cases.json` fields to each other. Whatever the right disposition — run the
parse harness, or park it in the stage that owns `harnesses/liana` — the plan
does neither.

**I-5 (Important) — Task 4 Step 3's licence is broader than its proof and
conflicts with §2's normative sequence.** §2 is per **leaf key expression** in
descriptor left-to-right order. The plan says *"Implement whichever is natural;
they are the same sequence"* — proven only for shapes md-cli mints, while the
code lands in md-codec, whose `validate_placeholder_usage` explicitly permits a
slot *"at least once"* and whose `Admission::Enforce` block adds no reuse check.
At the one production call site the natural implementation is the wrong one:
`node_to_descriptor` holds `keys: &[DescriptorPublicKey]` indexed **by slot**,
and the per-occurrence reading needs a tree walk the plan does not describe.
Task 5 Step 5 prescribes the correct reading in four words ("in tap-tree order")
and Task 4 Step 3 licenses the other; nothing in the plan discriminates, and no
test can.

---

## Per-task soundness after the fold

- **Task 1 — still UNSOUND** (M8, N4, and the undefined-helper residue); the
  core builder and the I2 correction are right.
- **Task 2 — SOUND** (M7, N3 only).
- **Task 3 — SOUND.**
- **Task 4 — UNSOUND** (C-1, I-5, M9).
- **Task 5 — UNSOUND** (I-5's mechanism gap, M2); the twelve-caller table and
  the refusing-entry-point design are correct and are the fold's strongest work.
- **Task 6 — UNSOUND** (mechanism gap 2, the multi-line stdout test, M1, M5).
- **Task 7 — UNSOUND** (I-1, I-2); the wiring step itself is correct.
- **Task 8 — UNSOUND** (I-3, I-4, M4); Steps 1, 2 and 5 are executable and
  Step 2's golden checks out.
- **Task 9 — UNSOUND as a gate** (I-2 leaves 2 of 15 mutations uncatchable, M3);
  Step 1's applied-before-trusted rule is a real improvement.
- **Task 10 — SOUND except M6.**

---

**Ready for implementation: no.**
