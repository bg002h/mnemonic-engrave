# R2 closing gate — `IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md` (r2 fold)

**Verdict: 0 Critical / 6 Important / 14 Minor / 4 Nit — NOT GREEN.**

Reviewer: independent agent, opus. Scope: the r2 fold (`81a8adbf..76f3400c`) against
`design/agent-reports/f449-plan-stage1b-r1.md`. Spec read for authority, not
re-reviewed. Repo read and RUN at `descriptor-mnemonic` `37367c1f`, `md` 0.17.0.
Every finding below was reproduced; nothing is carried on r1's word.

---

## HEADLINE — three of r1's five Importants were never folded

The r2 fold answers r1's **C-1, I-1, I-4** and the **§6 row 3** residual. r1's
**I-2, I-3 and I-5 are untouched** and are not mentioned in the fold commit.

The cause is a numbering collision. r0's findings were `I1..I7`; r1's own
blocking findings are `I-1..I-5`. The fold commit closes with *"I2, I3, I4, I7
confirmed addressed by the re-review"* — true of **r0's** I2/I3/I4/I7, which r1
marked ADDRESSED, and unrelated to **r1's** I-2/I-3/I-5. The dispatch brief for
this round repeats the gap: it asks for a status on *"the r1 report's Critical,
its I-1 and I-4"* and names no other Important.

Measured, in the r2 plan at `76f3400c`:

- `grep -c fixpoint` → **0**. §8 item 7 has no task (I-3).
- `grep -n "checksum"` → four hits, all `cases.json` field names. §8 item 2's
  codec leg has no assertion (I-3).
- Task 7 prescribes tests for two of §6's four encode-side refusals; Task 9 row
  8-11 still maps all four to Task 7 (I-2).
- Task 4 Step 3 still reads *"Implement whichever is natural; they are the same
  sequence"* (I-5), and `to_miniscript.rs:340-350` still builds the internal key
  **before** the script tree over a slot-indexed `keys` slice (RUN).

---

## Q1 — fold checklist

### The r1 Critical

**C-1 (the pin could not fail): PARTIAL — the mechanism is repaired and
verified; the placement half is untouched.**

Repaired and reproduced (see Q2a): the literal NUMS hex replaces `<NUMS>`, and
each case asserts its own error text. Both assertions' substrings resolve
against the live messages, and each is unique to the refusal it pins.

Unrepaired, and raised verbatim by r1 under the same Critical: the pin still
sits in **Task 4's test file, `crates/md-codec/tests/liana_unspendable.rs`**,
and md-codec cannot invoke `md`. Measured `[dev-dependencies]`: `serde`,
`serde_json`, `hex`, `proptest`, `miniscript` — no md-cli, and `CARGO_BIN_EXE_md`
is defined only for integration tests of the crate that declares the binary.
`md_err` remains undefined anywhere in the plan (3 call sites: `:436`, `:440`,
`:768`). Task 8 Step 5 relocates its own `md_err` pin to md-cli *for this exact
reason* — "(the guard is the template parser's)" — and Task 4's is not
relocated. Recorded below as **I-D**, not as a Critical: a test that cannot
compile is loud, not a false green.

### r1's Importants

| # | status | evidence |
| --- | --- | --- |
| **I-1** (Task 7's tests do not compile) | **PARTIAL** | Half (i) ADDRESSED — the two-home table splits the tests by what each needs, and `encode_payload` is public, so the two refusal tests may use `kind1_from_vector`. Half (ii) **untouched**: `UseSitePath::parse` still does not exist (`crates/md-codec/src/use_site_path.rs` exposes `standard_multipath()`, `write()`, `read()`; no `FromStr`, no `parse` in the crate), so `kind_1_off_the_canonical_use_site_is_refused` is still a compile error. → **I-E** |
| **I-2** (two of §6's four refusals have no test) | **NOT ADDRESSED** | SPEC §6's table has exactly four encode-side REFUSE rows: `sortedmulti_a` leaf, non-`<0;1>` use site, nesting under `sh`/`wsh`, and version 8 over a kind-0/`tr`-less tree. Task 7 prescribes tests for the first two only; no fixture exists for the other two. Task 9's row 8-11 is unchanged. → **I-A** |
| **I-3** (§8 items 2 and 7 have no task) | **NOT ADDRESSED** | §9's ownership table assigns *"1 recipe vectors, 6 structure pin, **7 fixpoint**, 10 mutation → 1b"* and *"2 descriptor equality → 1b (codec) **and** 2"*. The plan contains the word `fixpoint` zero times and asserts no rendered descriptor string anywhere. The corpus item 7 needs is live and in-repo (`crates/md-cli/src/format/text.rs:255-284`, RUN-read). Self-Review `:823` still reads "§8 items 1/2/5/6/7 → Task 8". → **I-B** |
| **I-4** (§8.1's accepted nested taptree) | **ADDRESSED** | Task 8 Step 4 now records the non-delivery explicitly and routes it to stage 2. The factual claim is correct — verified independently in Q2c. One misattribution inside it is recorded as a Minor. |
| **I-5** (licence broader than proof; the per-occurrence mechanism) | **NOT ADDRESSED** | Task 4 Step 3's "Implement whichever is natural" is unchanged; Task 5 Step 5's "in tap-tree order" is unchanged; nothing discriminates and no test can. Reproduced in the source: `validate_placeholder_usage` (`validate.rs:17-36`) requires each `@i` *"at least once"* and nothing more; `encode_payload_inner`'s `Admission::Enforce` block (`encode.rs:165-170`) adds only `validate_origin_key_consistency` and `validate_no_duplicate_key_slots`; `to_miniscript.rs:340-345` builds `internal_key` **before** `:346-350` builds `script_tree`, over `keys: &[DescriptorPublicKey]` indexed by slot. → **I-C** |

### The §6-row-3 residual under I6

**ADDRESSED.** The Self-Review now carries *"§6 row 3's `--unspendable liana`
no-op WARN is stage 2's"* with the reason (the flag is `md compose`, which §9
assigns to stage 2) and the fable M-6 attribution. It is named rather than lost
between the two plans, which is what the finding asked for.

### The 11 Minors

| # | status | evidence |
| --- | --- | --- |
| M1 stage-2 items pulled in undeclared | **PARTIAL** (unchanged) | The JSON-schema break is versioned; neither it nor the `UNSPENDABLE(liana)` substitution rule appears in the "Owned by later stages" list as pulled forward from §9's stage-2 column |
| M2 `NumsXpub` unnamed | **NOT ADDRESSED** | `grep -n NumsXpub plan` → 0 hits; `:497` still reads "maps to the new `KeyPathKind`" |
| M3 §8.10's seventh mutation | **NOT ADDRESSED** | `grep -n "KeyPathSpendable\|policy_id_stub" plan` → 0 hits; Task 9 rows 12-15 are still the or-pattern collapse |
| M4 §8.5's mk1 `policy_id_stub` leg | **NOT ADDRESSED** | `grep -n mk1 plan` → 0 hits; §8.5's Rust leg is 1b and Task 8 Step 2 asserts policy id, template id and phrase only |
| M5 vector corpus not regenerated | **NOT ADDRESSED** | `grep -n "md vectors --out" plan` → 0 hits |
| M6 cross-repo doc path | **NOT ADDRESSED** | `:810` still names `DESIGN_coordinator_compatibility.md:155-158` with no repo; File Structure omits it; the only `mnemonic-engrave` mention in the plan is Task 1's script argument at `:76` |
| M7 `chunk.rs:373` needs no change | **NOT ADDRESSED** | `:281` still says "`chunk.rs:70`/`:373` accept `{4, 8}`". RUN-verified: `chunk.rs:70` is `if version != Header::WF_REDESIGN_VERSION` (the real site); `chunk.rs:368-377` is the cross-chunk consistency loop comparing `h.version != expected_version` — already version-agnostic |
| M8 `kind1_from_vector` unguarded | **NOT ADDRESSED** | Task 1 Step 2 body unchanged; no `pubkeys` assert |
| M9 Task 4's structure test runs no production code | **NOT ADDRESSED, and the fold made it worse** | `:365` still maps `case(n).expected_xpub` — three `cases.json` strings compared to each other. Task 8 Step 4's new non-delivery record now cites this test by name as the thing that *"pins it against the golden xpub"*, which it does not do. (The traversal **is** pinned — by `the_recipe_reproduces_every_golden_xpub`, which calls `liana_unspendable_xpub` over all eight cases including the nested one. The fold names the wrong test.) |
| M10 undefined helpers vs the Self-Review's claim | **NOT ADDRESSED; list changed, one added** | Resolved by the fold: `md_encode`, `SAME_PATH_REUSE_TEMPLATE`, `DISJOINT_PATH_REUSE_TEMPLATE` are gone, `NUMS_HEX` is now defined. Still undefined: `md` (`:550`, `:561`, `:576`), `md_err` (`:436`, `:440`, `:768`), `ORIGINLESS_SPENDABLE_TR` (`:561`), `IdGolden` (`:723`), `UseSitePath::parse` (`:651`). **Added by the fold:** `in_crate_tr_liana_with_sortedmulti_a_leaf` (`:642`). Self-Review `:840` still asserts every helper is defined |
| M11 the marker test's multi-line stdout | **RETRACTED — not reproducible** | Measured: `md encode "tr(<NUMS hex>,{pk(@0/<0;1>/*),pk(@1/<0;1>/*)})" --path bip48` writes **one line** to stdout (the md1 string, `wc -l` = 1). The grouped form, `group size: 5`, `separator: space`, the keyless warning and the stdout note all go to **stderr**. `decode_md1_string(md1.trim())` is fine provided the undefined `md()` helper captures stdout only — which folds this into M10 rather than standing as its own defect |
| M12 Task 8's Files line vs its Step 5 | **NOT ADDRESSED** | Files says `crates/md-codec/tests/liana_unspendable.rs` only; Step 5 relocates the pin to md-cli |

*(r1's header said 11 Minors; the body carries M1-M9, the I6 second half demoted
to Minor, and three new ones under Q2e. The list above resolves all of them and
adds the three new ones found in this fold, below.)*

### The 4 Nits

| # | status | evidence |
| --- | --- | --- |
| N1 two different `tr` counts | **NOT ADDRESSED** | `:153` says "23 of the 65"; `:833` says "22 of the 65". 23 is right: `crates/md-codec/tests/internal_key_refactor.rs:30` asserts `g.tr_count, 23` |
| N2 "55 call sites" | **NOT ADDRESSED** | `:467` unchanged. The measured production count is 12 (plus 2 definitions) |
| N3 wrong recursive fn, missing import | **NOT ADDRESSED** | RUN-verified: `pub fn read_node(r, key_index_width)` at `tree.rs:205` delegates to `fn read_node_with_depth(r, key_index_width, depth)` at `:212` — the recursive one the version must thread through. The plan names only `read_node` / "both node fns"; the `use crate::header::Header;` gap is still unmentioned |
| N4 the four refused cases have no addresses | **PARTIAL** (unchanged) | `Case` still declares `liana_receive: Vec<String>` / `liana_change: Vec<String>` non-optional and Step 1 still says `liana_receive[3]` |

---

## Q2 — the four narrow questions

### (a) The C2 pin's repair — the reasoning is correct, the assertions are sound, the placement is not

**The fold's reasoning is correct and I reproduced its sharp edge.** RUN at
`descriptor-mnemonic` `37367c1f` with `md` 0.17.0, `H` = the literal NUMS hex:

```
$ md encode "tr(H,{pk(@0/<0;1>/*),and_v(v:pk(@0/<0;1>/*),older(26280))})" --path bip48
md: unsupported: @0 appears at 2 use sites in this template with the same path
expression, so ONE key would fill every one of them. That is forbidden by BIP 388's
disjointness rule (...), whose forbidden-example list names sh(multi(1,@0/**,@0/**))
— "Repeated keys with the same path expression". ...

$ md encode "tr(H,{pk(@0/<0;1>/*),and_v(v:pk(@0/<2;3>/*),older(26280))})" --path bip48
md: unsupported: @0 appears at use sites with DISJOINT multipath sets — <0;1> and
<2;3>. The WALLET is legal under BIP 388 ... because an md1 card carries ONE path per
key slot ... (F-417) ...
```

Both assertion pairs hold against the live text: `"same path expression"` +
`"BIP 388"` in the first, `"DISJOINT multipath sets"` + `"ONE path per key slot"`
in the second. `--path bip48` is a valid flag (`cmd/encode.rs:22-26`, named forms
`bip44|48|49|84|86`).

**Can it now fail for the right reason, and only that reason? Yes, on both
counts.** Removing the same-path refusal leaves that template either minting or
failing elsewhere; either way `"same path expression"` disappears and the
assertion goes RED. Removing the disjoint refusal does the same for the second.
Neither string can arrive from another source: `grep -rn --include='*.rs'` puts
`"same path expression"` in `decompose/mod.rs:320` (a different command) and
test files only, and `"ONE path per key slot"` at exactly one production site,
`parse/reuse.rs:318`. The case is even tighter than the fold claims — the
sibling `Finding::MultipathOverlap` message (`reuse.rs:307-313`) also contains
`"DISJOINT multipath sets"` but spells the other phrase *lowercase* ("one path
per key slot"), so the case-sensitive pair distinguishes `MultipathDisjoint`
from `MultipathOverlap` as well.

**What the repair did not fix.** r1 raised three problems under C-1 — the
spelling, the `is_err()` identity, and the **crate placement**. Two are fixed.
The third is untouched: the pin lives in md-codec's `tests/liana_unspendable.rs`
(Task 4's Files line; Task 8's Files line confirms the path), md-codec's
dev-dependencies contain no md-cli, and `md_err` is defined nowhere. → **I-D**.

### (b) The I-1 test split — one of three lands cleanly, two do not

| test | home | compiles? | can fail? |
| --- | --- | --- | --- |
| `kind_1_with_a_sortedmulti_a_leaf_is_refused_at_MINT` | integration | **yes** — `encode_payload` is public, `kind1_from_vector` is reachable from `tests/` | **yes** — RUN-verified: `is_forbidden_leaf_tag` (`validate.rs:271-276`) lists `Wpkh \| Tr \| Wsh \| Sh \| Pkh \| Multi \| SortedMulti` and **not** `SortedMultiA`, so `encode_payload` returns `Ok` today |
| `kind_1_off_the_canonical_use_site_is_refused` | integration | **no** — `UseSitePath::parse` does not exist | undeterminable. The value is constructible by struct literal; this is a spelling defect the fold did not reach → **I-E** |
| `a_refused_shape_still_DECODES_so_existing_cards_never_stop_reading` | unit, in `encode.rs` | **no** — its fixture `in_crate_tr_liana_with_sortedmulti_a_leaf()` is undefined; the plan gives only the comment *"built from Node/Body directly"* | undeterminable → **I-F** |

The **ruling** is right: `encode_payload` is public at `:99`, so the two refusal
tests genuinely belong in `tests/`, and only the third needs
`encode_payload_inner` (`:143`, module-private) and `Admission::SkipPolicy`
(`:107`, `pub(crate)`), so only it must be a unit test. `Node`, `InternalKey`
and `Body` are public (`tree.rs:9,22,34`) and `Descriptor`'s fields are public
(`encode.rs:17-28`), so an in-crate fixture is *possible* — but the plan shows
none, and the fixture has to simultaneously trip the new refusal and round-trip
through encode/decode. The fold replaced a working builder reference with an
undefined one, under a heading that still reads **"The two fixtures these tests
use, defined here"** while three fixtures are now in play.

### (c) The I-4 non-delivery record — accurate

**Verified independently against the evidence**, not against the plan.
`design/evidence/composer-fable-r0/fable-liana-parse-out-v15.jsonl` carries
exactly 8 `liana-unspendable-xpub` records; 4 are `ok: true`. Their taptrees,
extracted and printed:

| case | accepted | tree |
| --- | --- | --- |
| `preset-kofn-recovery-tr` | yes | `{multi_a(2,A,B,C), and_v(v:pk(D),older(26280))}` — flat |
| `preset-tiered-recovery-tr` | yes | `{multi_a(2,A,B), and_v(v:multi_a(1,C,D),older(26280))}` — flat |
| `same-seed-two-paths-tr` | yes | `{multi_a(2,...), and_v(v:pk(...),older(26280))}` — flat |
| `X19-tr-kofn-nums-older5` | yes | `{multi_a(2,...), and_v(v:pk(...),older(5))}` — flat |
| `preset-decaying-multisig-tr` | **no** | `{and_v(v:multi_a(2,A,B),older(13140)), {and_v(...C...), and_v(...D...)}}` — **nested** |
| `preset-hashlock-gated-tr`, `hashlock-gated-tr-hash160`, `X20-tr-hashlock-known` | no | flat two-leaf |

**The record is accurate on every clause.** No accepted case is nested; the only
nested case in the corpus is `preset-decaying-multisig-tr`, refused with
*"Descriptor is not compatible with a Liana spending policy."* Routing the gap
to stage 2's live run is a defensible disposition — §9 owns §8.8 there, and the
C3 ruling already establishes that rendering a descriptor string for the harness
needs md-cli.

Two supporting facts also check out: `kofn`, `tiered` and `decaying` really are
three different trees over the **same four keys in the same order**
(`73c5da0a/48'/0'/0'/3'`, `3f635a63/48'/0'/0'/3'`, `66d455ea/48'/0'/0'/3'`,
`73c5da0a/48'/0'/1'/3'`), so Task 4's structure fixture is sound.

The one defect inside the record is the misattribution noted at M9: the fold
says the ordering is *"pinned by Task 4's `the_chain_code_depends_on_the_KEYS_
not_the_TREE`"*, and that test compares three `cases.json` fields to each other
and executes no production code. The nested traversal **is** pinned — by
`the_recipe_reproduces_every_golden_xpub`, which runs `liana_unspendable_xpub`
over all eight cases, decaying-multisig included. The substitution is real; the
sentence names the wrong test.

### (d) Contradictions inside the plan

Three are **new in this fold**, all in Task 7, all from the split:

1. **`:625` — Step 1's heading still reads "as UNIT tests inside `encode.rs`"**,
   directly above a table that puts two of its three tests in `tests/`.
2. **Task 7's Files line** (`:617`) names `validate.rs`, `error.rs`,
   `encode.rs:143-170` and **not** `crates/md-codec/tests/liana_unspendable.rs`,
   now the home of two of its three tests.
3. **Step 2's verification command is stale** — `cargo nextest run --locked -p
   md-codec encode::` filters on the `encode::` module path, which the two
   relocated integration tests no longer carry. (Minor, not Important: the
   package still compiles all test targets, so the missing `Error` variants
   surface as a build failure rather than a silent skip.)

Four are **carried, unresolved**:

4. **Two `md_err` pins, contradictory homes.** Task 8 Step 5 relocates its pin
   to md-cli *because* the guard is the template parser's; Task 4's identical
   `md`-invoking pin stays in md-codec (→ I-D).
5. **Self-Review `:823`** — "§8 items 1/2/5/6/7 → Task 8" is false for items 2
   and 7 (→ I-B).
6. **Self-Review `:840`** — "Every helper this plan calls is defined by it" is
   false for six helpers, one of them added by this fold (→ M10).
7. **`:153` says 23 `tr` vectors, `:833` says 22** (→ N1).

No contradiction was found between the fold and the SPEC, and none between the
fold and Tasks 1-3, 9 or 10.

---

## Blocking findings

**I-A (Important) — two of §6's four encode-side refusals have no test, so
mutations 8-11 cannot all turn RED.** SPEC §6 refuses four things at encode:
a `sortedmulti_a` leaf, a use-site other than `<0;1>`, kind 1 nested under
`sh`/`wsh`, and version 8 over a kind-0 or `tr`-less tree. Task 7 prescribes
tests for the first two. Task 9's row 8-11 maps all four mutations to Task 7.
Two of the fifteen mutations are therefore uncatchable and Task 9 is not yet a
gate. *(r1 I-2, unfolded.)*

**I-B (Important) — §8 items 2 and 7 have no task in 1b while the Self-Review
claims Task 8 covers them.** §9 owns item 7 (render-reparse fixpoint) at 1b
outright and double-gates item 2 (descriptor equality incl. checksum) at 1b
(codec) and 2 (CLI). The plan says `fixpoint` zero times and asserts no rendered
descriptor string at any layer. Item 7's corpus is live and in-repo at
`crates/md-cli/src/format/text.rs:255-284`, so "every leg runnable in Rust
alone" holds. *(r1 I-3, unfolded.)*

**I-C (Important) — Task 4 Step 3's licence is broader than its proof, and the
per-occurrence mechanism is still undescribed.** §2 is normative per **leaf key
expression** in descriptor order; the plan says "Implement whichever is natural;
they are the same sequence", proven only for shapes md-cli mints. The code lands
in md-codec, where `validate_placeholder_usage` (`validate.rs:17-36`) permits a
slot *"at least once"* and the `Admission::Enforce` block (`encode.rs:165-170`)
adds no reuse check. At the one production site the natural implementation is
the wrong one: `to_miniscript.rs:340-345` builds the internal key **before**
`:346-350` builds the script tree, over a slot-indexed `keys` slice — so the
per-occurrence reading needs a DFS walk the plan never describes, while Task 5
Step 5 prescribes it in four words and Task 4 Step 3 licenses the alternative.
No test discriminates. *(r1 I-5, unfolded.)*

**I-D (Important) — the repaired C2 pin still cannot run where the plan puts
it.** Task 4's test file is `crates/md-codec/tests/liana_unspendable.rs`;
md-codec's `[dev-dependencies]` are `serde`, `serde_json`, `hex`, `proptest`,
`miniscript` (measured), and `CARGO_BIN_EXE_md` is not defined outside md-cli's
own targets. `md_err` is undefined at all three of its call sites. Task 8 Step 5
relocates an identical pin to md-cli for exactly this reason. *(Residual of r1's
Critical; the assertion mechanism itself is repaired and verified.)*

**I-E (Important) — `UseSitePath::parse` does not exist, so
`kind_1_off_the_canonical_use_site_is_refused` does not compile.**
`crates/md-codec/src/use_site_path.rs` exposes `standard_multipath()`, `write()`
and `read()`; there is no `FromStr` and no `parse` in the crate. The value is
constructible by struct literal, so this is a spelling defect — but it is the
half of r1's I-1 that the fold did not reach, and it leaves one of §6's two
tested refusals unable to build. *(r1 I-1(ii), unfolded.)*

**I-F (Important) — the fold replaced a working fixture reference with an
undefined one, in the test that encodes a measured funds regression.**
`a_refused_shape_still_DECODES_so_existing_cards_never_stop_reading` is the
assertion form of the 2026-09-19 regression at `encode.rs:116-131` (mint-side
refusals leaking into decode made a shipped 2-of-2 stop reading). Its fixture is
now `in_crate_tr_liana_with_sortedmulti_a_leaf()`, defined nowhere, described
only as *"built from Node/Body directly"*. The fixture must simultaneously trip
the new refusal and survive `encode_payload_inner(.., SkipPolicy)` →
`decode_payload`, and the plan shows neither. Task 7's fixture block still reads
"The two fixtures these tests use, defined here" while three are in use, and the
Self-Review still asserts every helper is defined. Whether this test can fail is
currently undeterminable.

---

## Per-task soundness after the fold

- **Task 1 — UNSOUND** (M8, N4, M10 residue).
- **Task 2 — SOUND** (M7, N3 only).
- **Task 3 — SOUND.**
- **Task 4 — UNSOUND** (I-C, I-D, M9); the pin's assertion mechanism is now
  correct and verified, which is the fold's strongest work.
- **Task 5 — UNSOUND** (I-C's mechanism gap, M2); the twelve-caller table and
  the refusing-entry-point design remain correct.
- **Task 6 — UNSOUND** (the reserved-synthetic-key mechanism gap, M1, M5); M11
  is retracted — its stdout is a single line.
- **Task 7 — UNSOUND** (I-A, I-E, I-F, and the three new internal
  contradictions); the two-home ruling and the Step 4 wiring are correct.
- **Task 8 — UNSOUND** (I-B, M4, M12); Step 4's non-delivery record is accurate
  and is the fold's second-strongest work.
- **Task 9 — UNSOUND as a gate** (I-A leaves 2 of 15 mutations uncatchable, M3).
- **Task 10 — SOUND except M6.**

---

**Ready for implementation: no.** Six Importants block, three of them r1
findings that this fold did not touch and did not decline. The fold itself is
good work on what it addressed — the pin repair is verified correct and the
nested-taptree non-delivery is factually accurate — but the gate is 0C/0I and
the artifact is not there.

The cheapest route to GREEN is not another review round. I-D, I-E and the three
Task 7 contradictions are single-line edits; I-A and I-B are four missing tests
and one Self-Review line; I-C needs one paragraph naming the DFS walk and one
deleted sentence. I-F needs the fixture written out. None of that needs a
reviewer to find again — it needs folding, then a scoped re-check that the six
are closed and nothing new was introduced.
