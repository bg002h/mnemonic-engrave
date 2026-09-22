# R3 closing gate — `IMPLEMENTATION_PLAN_f449_stage1b_wire_kind.md` (r4 fold)

**Verdict: 0 Critical / 3 Important / 5 Minor / 1 Nit (new) — NOT GREEN.**

Reviewer: independent agent, opus. Scope: the r3+r4 fold (`76f3400c..7c32f0e1`)
against `design/agent-reports/f449-plan-stage1b-r2.md`. Spec read for authority,
not re-reviewed. Repo read and RUN at `descriptor-mnemonic` `37367c1f`; the tree
was left clean (`git status --porcelain` empty) after the probe below was removed.

Every finding is cited `<round>/<label>`. My own are `r3/…`.

---

## HEADLINE — all six of r2's Importants were folded; three of the folds do not survive execution

This fold is materially better than the last: it ticks r2's list one by one, it
adds the round-prefix convention as a §0 section so the label collision cannot
recur, and it retracts the Self-Review's absolute "every helper is defined"
overclaim in favour of a re-run instruction. r2's I-A, I-B, I-D and I-E are
genuinely closed, and I-E's struct literal is correct field-for-field.

But three of the fold's own new code blocks were never executed, and all three
are wrong in the same way the gate cannot see:

- **r2/I-F's replacement fixture cannot round-trip.** MEASURED: it encodes and
  then **fails to decode**, so the funds-regression test fails as written — with
  a message pointing the implementer at the decode path.
- **r2/I-B's restored item-7 test shells out to the `md` binary from md-codec's
  test file** — the exact defect r2/I-D raised and this same fold fixed in Task 4.
- **r1/I-5's prescribed tap-tree walk names an API that does not exist** in the
  pinned miniscript 13.0.0.

`./scripts/plan-api-check.sh` prints *all extracted symbols resolve* for this
plan — because `key_arg` and `descriptor_with` were **added to its ALLOW list**
as "symbols the plan declares it will CREATE", while the plan's own comment calls
`descriptor_with` an *"existing `#[cfg(test)]` helper in encode.rs"*. Those two
statements contradict each other, and grep settles it: neither helper exists
anywhere in `crates/`. The gate did not fail; it was told not to look.

---

## Q1 — status of EVERY r2 finding

### r2's six Importants

| # | status | evidence |
| --- | --- | --- |
| **r2/I-A** (two of §6's four refusals have no test) | **ADDRESSED** | New Task 7 Step 2b (`:733-763`) adds `kind_1_nested_under_wsh_is_refused` and `version_8_with_no_kind_1_node_is_refused_at_ENCODE` with fixtures `wsh_wrapping_tr_liana`, `all_nums_tr`, `encode_payload_at_forced_version`. Task 9's row 8-11 now has four tests behind it, so mutations 8-11 are each catchable. Residue: the two tests' home is unstated → **r3/M4** |
| **r2/I-B** (§8 items 2 and 7 have no task) | **ADDRESSED, and it introduced r3/I-2** | New Task 8 Step 5b (`:866-899`) adds `item_2_descriptor_equality_at_the_CORPUS_level` and `item_7_the_render_reparse_fixpoint_covers_tr_kind_1`. `grep -c fixpoint` → 2 (was 0). Self-Review's "§8 items 1/2/5/6/7 → Task 8" is now TRUE. But `item_7` calls `md(&[…])` from md-codec's test file → **r3/I-2**; `item_2`'s headline assertion is a tautology → **r3/M3** |
| **r2/I-C** (licence broader than proof; per-occurrence mechanism undescribed) | **ADDRESSED in reasoning, WRONG in code** | `:413-441`: "Implement whichever is natural; they are the same sequence" is gone; §2 is now normatively per KEY OCCURRENCE with a RIGHT/WRONG contrast naming `node_to_descriptor`'s slot-indexed `keys` slice. The reasoning is correct and I reproduced it (Q2d). The prescribed snippet does not compile → **r3/I-3** |
| **r2/I-D** (the C2 pin cannot run where the plan puts it) | **ADDRESSED** | `:448-453` relocates the pin: *"This pin lives in `crates/md-cli/tests/`, not md-codec's"*, with the dev-dep list and the `CARGO_BIN_EXE_md` reason, citing Task 8 Step 5's precedent. Verified in Q2a. Residue: Task 4's **Files** line was not updated → **r3/M1** |
| **r2/I-E** (`UseSitePath::parse` does not exist) | **ADDRESSED** | `:711-724` replaces `parse` with the struct literal and `tr_liana_at_use_site(a: u32, b: u32)`. Verified field-for-field in Q2b — correct. Residue: the import is not named → **r3/M2** |
| **r2/I-F** (undefined fixture in the funds-regression test) | **NOT ADDRESSED — the body written is structurally invalid** | `:669-681` now gives a body, but `Node { tag: Tag::SortedMultiA, body: Body::Variable {…} }` is a tag/body mismatch, and `key_arg`/`descriptor_with` do not exist. MEASURED: the test fails. → **r3/I-1** |

### r2's carried fold-checklist items

| # | status | evidence |
| --- | --- | --- |
| **r1/C-1** (the pin could not fail) | **ADDRESSED** | Both halves now closed: the mechanism was repaired in r2 and verified there; the placement half is closed by `:448-453`. `r3/M1` is the Files-line residue only |
| **r1/I-1** | **ADDRESSED** | half (i) in r2, half (ii) = r2/I-E, closed above |
| **r1/I-2** | **ADDRESSED** | = r2/I-A |
| **r1/I-3** | **ADDRESSED** | = r2/I-B |
| **r1/I-4** (§8.1's accepted nested taptree) | **ADDRESSED** (unchanged, was already) | Task 8 Step 4's non-delivery record stands; r2 verified it independently against the evidence |
| **r1/I-5** | **ADDRESSED in reasoning** | = r2/I-C |
| **§6 row 3 residual under r0/I6** | **ADDRESSED** (unchanged) | Self-Review still carries the `--unspendable liana` no-op WARN routed to stage 2 with the fable M-6 attribution |

### r2's 14 Minors

| # | status | evidence (measured against the r4 plan) |
| --- | --- | --- |
| **r2/M1** stage-2 items pulled in undeclared | **NOT ADDRESSED** | "Owned by later stages" (`:955`) still lists only §0b/§7/§7a, §8.3, §8.4+§7a.1, §8.8, §8b+§9a, §8.9. Neither the JSON-schema break nor the `UNSPENDABLE(liana)` substitution rule appears as pulled forward |
| **r2/M2** `NumsXpub` unnamed | **NOT ADDRESSED** | `grep -c NumsXpub` → **0**. Task 5 Step 5 still reads "maps to the new `KeyPathKind`" |
| **r2/M3** §8.10's seventh mutation | **NOT ADDRESSED** | `grep -c "KeyPathSpendable\|policy_id_stub"` → **0**. Task 9 rows 12-15 are still the or-pattern collapse |
| **r2/M4** §8.5's mk1 `policy_id_stub` leg | **NOT ADDRESSED** | `grep -c mk1` → **0** |
| **r2/M5** vector corpus not regenerated | **NOT ADDRESSED** | `grep -c "md vectors --out"` → **0** |
| **r2/M6** cross-repo doc path | **NOT ADDRESSED** | Task 10 Step 1 still names `DESIGN_coordinator_compatibility.md:155-158` with no repo; File Structure still omits it |
| **r2/M7** `chunk.rs:373` needs no change | **NOT ADDRESSED** | `:193` and `:282` both still carry `chunk.rs:70`/`:373`; `:282` still says both "accept `{4, 8}`". RUN-verified in r2: `:368-377` is the version-agnostic cross-chunk consistency loop |
| **r2/M8** `kind1_from_vector` unguarded | **NOT ADDRESSED** | Task 1 Step 2's body (`:139-156`) is byte-unchanged; still no `pubkeys` assert |
| **r2/M9** Task 4's structure test runs no production code | **NOT ADDRESSED** | `:326` still maps `case(n).expected_xpub` — three `cases.json` strings compared to each other; Task 8 Step 4 (`:845`) still cites that test as the thing that "pins it against the golden xpub". The traversal is in fact pinned by `the_recipe_reproduces_every_golden_xpub` |
| **r2/M10** undefined helpers vs the Self-Review's claim | **PARTIAL — the false CLAIM is gone, the helpers are not** | `grep -c "Every helper this plan calls"` → **0**; `:969` is now "RE-RUN THE SCRIPT, do not restate this sentence", with the two historical falsifications named. That is the right repair. Still undefined and still uncalled-out: `md` (4 sites), `md_err` (4 sites), `ORIGINLESS_SPENDABLE_TR` (`:604`), `IdGolden` (`:825`), `compressed_33` (`:432` → **r3/M5**). `descriptor_to_template` RESOLVES — `render.rs:146`, re-exported at `lib.rs:67` |
| **r2/M11** the marker test's multi-line stdout | **RETRACTED by r2** — no action expected, none taken |
| **r2/M12** Task 8's Files line vs its Step 5 | **NOT ADDRESSED, and the fold made it worse** | `:781` still says `crates/md-codec/tests/liana_unspendable.rs` only, while Step 5 relocates its pin to md-cli AND new Step 5b adds a second md-cli-needing test (→ r3/I-2) |
| **r2/d1** Task 7 Step 1's heading contradicts its own table | **NOT ADDRESSED** | `:650` still reads "**Step 1: Write the failing tests — as UNIT tests inside `encode.rs`**", directly above the two-home table |
| **r2/d2** Task 7's Files line omits the integration test file | **NOT ADDRESSED** | `:646` still names `validate.rs`, `error.rs`, `encode.rs:143-170` and not `crates/md-codec/tests/liana_unspendable.rs` |
| **r2/d3** Task 7 Step 2's verification command is stale | **NOT ADDRESSED** | `:730` still filters `-p md-codec encode::`, which the two relocated integration tests do not carry |

*(r2's 14 = M1-M10 + M12 (M11 retracted) + d1-d3.)*

### r2's 4 Nits

| # | status | evidence |
| --- | --- | --- |
| **r2/N1** two different `tr` counts | **NOT ADDRESSED** | `:165` "23 of the 65"; `:961` "22 of the 65". **23 is right**: `crates/md-codec/tests/internal_key_refactor.rs:30` asserts `g.tr_count, 23` (RUN-read) |
| **r2/N2** "55 call sites" | **NOT ADDRESSED** | `:562` unchanged, and it sits nine lines above Step 4's "**ALL TWELVE production callers — G-7 undercounted by nine**", which enumerates twelve |
| **r2/N3** wrong recursive fn, missing import | **NOT ADDRESSED** | `grep -c read_node_with_depth` → **0**; `grep -c "use crate::header::Header"` → **0**. Confirmed in source: `pub fn read_node` at `tree.rs` delegates to the recursive `read_node_with_depth`, which is the one that must thread the version |
| **r2/N4** the four refused cases have no addresses | **PARTIAL** (unchanged) | `Case` still declares `liana_receive`/`liana_change` as non-optional `Vec<String>`, and Task 1 Step 1 still says `liana_receive[3]`, for eight cases of which four were refused |

---

## Q2 — the five narrow questions

### (a) r2/I-D's relocation — CORRECT, with a Files-line residue

**Can the pin run in `crates/md-cli/tests/`? Yes.** RUN-verified:

- `CARGO_BIN_EXE_md` **is** defined for md-cli's own integration targets —
  `crates/md-cli/tests/cmd_encode.rs` uses `env!("CARGO_BIN_EXE_md")` at ten
  sites (`:432, :447, :479, :493, :524, :538, :573, :594, :622, :640`).
- md-codec's `[dev-dependencies]` measured: `serde`, `serde_json`, `hex`,
  `proptest`, `miniscript` — no CLI runner. The fold's stated reason is exact.

**Does md-codec still reference it? No.** The pin block (`:456-489`) now sits
under the explicit relocation sentence, and nothing in Task 4's prose puts a
`md_err` call in md-codec.

**`md_err` itself does not exist in the repo** — `grep -rn "fn md_err" crates/`
→ zero. It is on `plan-api-check.sh`'s ALLOW list as a symbol the plan creates,
which is the right disposition; the implementer writes it. Note for the
implementer: md-cli has **no shared test helper** for this — 23 separate test
files each define their own `fn md()` with three different return shapes
(`Command`, `(i32, String, String)`, `(String, String, i32)`, `(bool, String,
String)`), so `md_err` must be written locally in whichever new file the pin
lands in.

**Residue → r3/M1:** Task 4's **Files** line (`:343`) still reads *"Test:
`tests/liana_unspendable.rs` (create, with a `//!` header)"* — md-codec's path —
and names no md-cli test file at all. The prose and the manifest disagree; this
is r2/M12's shape, now in a second task.

### (b) r2/I-E's `UseSitePath` struct literal — CORRECT, field for field

Read at `crates/md-codec/src/use_site_path.rs`:

```rust
pub struct Alternative { pub hardened: bool, pub value: u32 }          // :32-37
pub struct UseSitePath { pub multipath: Option<Vec<Alternative>>,      // :64-69
                         pub wildcard_hardened: bool }
```

Every field name, every type and every `pub` matches the plan's literal. The
signature change to `tr_liana_at_use_site(a: u32, b: u32)` matches
`Alternative::value: u32` exactly, and `standard_multipath()` (`:71-86`) builds
the identical shape with values 0 and 1 — so `(2, 3)` is well-formed and is the
non-canonical use site §6 row 2 refuses. The fold's cited surface
("`standard_multipath()`, `write()`, `read()`") is the complete public surface.

**`Alternative` does need importing, and it is public** — but neither
`Alternative` nor `UseSitePath` is re-exported at md-codec's crate root
(`lib.rs:49-72` re-exports `Descriptor`, `Error`, `Header`, `Tag`, `OriginPath`,
`PathComponent`, `PathDecl`, `PathDeclPaths`, `TlvSection`, … and **not** these).
`pub mod use_site_path;` is at `lib.rs:45`, so the integration test needs
`use md_codec::use_site_path::{Alternative, UseSitePath};`. The plan does not
say so → **r3/M2** (Minor; a compile error the first `cargo build` names).

### (c) r2/I-F's in-crate fixture — **BROKEN. The test fails as written.** → r3/I-1

**Do the cited shapes exist?**

| symbol | verdict |
| --- | --- |
| `Tag::SortedMultiA` | **exists** — `tag.rs:35`, wire byte `0x09` (`:109`, `:173`) |
| `Node { tag, body }` | **exists** — `tree.rs:8-13`, both fields public |
| `Body::Variable { k: u8, children: Vec<Node> }` | **exists** — `tree.rs:38-45` — **but its doc comment reads "Variable-arity body for `Tag::Thresh` ONLY (post-v0.30 Phase C). Multi-family tags use `Body::MultiKeys` per SPEC v0.30 §4."** |
| `Body::MultiKeys { k: u8, indices: Vec<u8> }` | **exists** — `tree.rs:46-56`, and IS the body `Tag::SortedMultiA` takes |
| `key_arg(i)` | **DOES NOT EXIST** — `grep -rn "fn key_arg\b" crates/` → zero |
| `descriptor_with(tree, n)` | **DOES NOT EXIST** — `grep -rn "fn descriptor_with\b" crates/` → zero. The plan calls it an *"existing `#[cfg(test)]` helper in encode.rs"*; the nearest real thing is `descriptor_with_pubkeys` at `validate.rs:1361` and `tests/common/mod.rs:840`, neither in encode.rs |

**Can the test it serves actually FAIL? It does worse: it fails NOW, for the
wrong reason.** `write_node` dispatches on the **BODY** (`tree.rs`, `Body::Variable`
arm writes `(k-1)(5) | (n-1)(5) | n × write_node(child)`), while `read_node`
dispatches on the **TAG** (`Tag::Multi | SortedMulti | MultiA | SortedMultiA =>
Body::MultiKeys`, reading `(k-1)(5) | (n-1)(5) | n × raw index(kiw)`). A node
that is `SortedMultiA` by tag and `Variable` by body therefore encodes under one
layout and decodes under another.

MEASURED, by building the plan's fixture verbatim as an md-codec integration
test at `37367c1f` and running it:

```
PROBE  (plan's shape: Tag::SortedMultiA + Body::Variable)
       encode OK, 106 bits
       decode ERR = TlvLengthExceedsRemaining { length: 9, remaining: 5 }

PROBE2 (Tag::SortedMultiA + Body::MultiKeys { k: 2, indices: vec![0,1,2] })
       encode OK, 88 bits
       decode OK
```

So `assert!(decode_payload(&bytes, bits).is_ok(), "a mint-side refusal reached
decode")` is **RED before a line of §6 is written**, and its failure message
tells the implementer that a mint-side refusal reached decode. It did not. The
bitstream desynced. This is the worst available failure mode for this particular
test: it is the assertion form of the measured 2026-09-19 funds regression
(`encode.rs:116-131`), it names the decode path in its own message, and the plan
warns four separate times that a decode-side refusal makes existing plates
unreadable. An implementer who believes the message will go looking for a leak
in `decode_payload_with_opts` and may "fix" the very thing the test protects.

The other half of the requirement is fine once the body is corrected:
`validate_tap_script_tree` runs **outside** the `Admission::Enforce` block
(`encode.rs:152-156`), and `is_forbidden_leaf_tag` does not list `SortedMultiA`,
so the corrected fixture encodes clean under `SkipPolicy` and decodes — exactly
what the test needs — while Task 7 Step 4 correctly wires
`validate_unspendable_shape` inside the `Enforce` block so it trips at MINT only.

**Fix (measured to work):** `Body::MultiKeys { k: 2, indices: vec![0, 1, 2] }`,
and write out `key_arg` / `descriptor_with` rather than calling them existing —
or drop `key_arg` entirely, since `MultiKeys` takes raw `u8` indices, not `Node`s.

**How this passed the gate:** `plan-api-check.sh`'s `ALLOW` string now contains
`key_arg|descriptor_with`, exempting them as "symbols the plan declares it will
CREATE" — while the plan's comment declares `descriptor_with` **existing**. The
ALLOW list and the plan contradict each other, and the contradiction is what
produced *all extracted symbols resolve*.

### (d) r1/I-5's per-occurrence ruling — the MECHANISM is right, the PRESCRIBED CODE does not exist → r3/I-3

**The ordering claim is correct.** `node_to_descriptor` (`to_miniscript.rs:313`)
builds `internal_key` at `:339-345` and `script_tree` at `:346-350`, over
`keys: &[DescriptorPublicKey]` indexed by slot via `lookup_key`. So the internal
key genuinely is built **before** the tree, the natural implementation there
genuinely is slot-indexed, and the fold's RIGHT/WRONG contrast is sound. Task 5
Step 5's "from the already-built leaf miniscripts in tap-tree order" implies the
required reordering.

**The permissiveness claim is correct.** `validate_placeholder_usage` requires
each `@i` at least once and nothing more; the `Admission::Enforce` block adds only
`validate_origin_key_consistency` and `validate_no_duplicate_key_slots`. An
in-crate `Descriptor` can carry a repeated slot that `md encode` refuses.

**The code cannot compile.** The plan prescribes:

```rust
for (_depth, ms) in tap_tree.iter() {
    ms.for_each_key(|k| { pks.push(compressed_33(k)); true });
}
```

Measured against the pinned miniscript — `Cargo.lock:532-534`, `13.0.0` at git
rev `ff4732e5f75aa555682343cb180fa72ee3e8e9d5`:

1. **`TapTree::iter()` does not exist.** The only iteration accessor is
   `pub fn leaves(&self) -> TapTreeIter<'_, Pk>` (`src/descriptor/tr/taptree.rs:56`,
   and `Tr::leaves` at `src/descriptor/tr/mod.rs:110`). `grep -rn "fn iter"` over
   `src/descriptor/tr/` returns nothing.
2. **The item is not a tuple**, so `(_depth, ms)` does not destructure.
   `TapTreeIter::Item = TapTreeIterItem<'tr, Pk>` (`:161-169`), a struct whose
   accessors are `.miniscript() -> &'tr Arc<Miniscript<Pk, Tap>>` (`:201`) and
   `.depth() -> u8` (`:207`).
3. **There is no `tap_tree` binding** at that point — the local is
   `script_tree: Option<TapTree<DescriptorPublicKey>>`, so the walk needs the
   `Option` handled.
4. **`compressed_33` is undefined** (→ r3/M5). It is not trivial for every
   variant: a `DescriptorPublicKey::Single(SinglePub { key: SinglePubKey::XOnly,
   .. })` — which is exactly what `build_nums_internal_key()` produces at
   `to_miniscript.rs:360-368` — has no unambiguous 33-byte compressed form.

The repo **already uses the correct API in three places** —
`md-cli/src/parse/template.rs:1671` (`for leaf in tt.leaves()`), `:2684`, and
`md-codec/tests/skeleton_key_conformance.rs:461` — so the plan prescribes a
non-existent spelling while the working one is in-tree.

**One thing that does check out, and matters:** `leaves()` yields depth-first
left-to-right. Its own doc comment gives the worked example
`(2, A), (2, B), (2, C), (3, D), (3, E)` for `{{A,B},{C,{D,E}}}`, which is the
order §2 needs. And applying `for_each_key` to a **leaf miniscript** rather than
to the whole descriptor sidesteps the trap this repo documents at
`md-cli/src/decompose/walk.rs:36-37` — *"measured 2026-08-30, `for_each_key` on
`tr(K,pk(L))` yields `[L, K]`, the leaf before the internal key"* — because the
internal key is not inside any leaf. The design is right; only the spelling is wrong.

**Why the gate missed it:** `iter` and `for_each_key` are both in
`plan-api-check.sh`'s `KEYWORDS` set (std/prelude/common-crate methods), and
`compressed_33` is in its `ALLOW` list.

### (e) Staleness across 971 lines and four folds

**Nothing in the fold contradicts the SPEC**, and nothing contradicts Tasks 1-3,
9 or 10. The round-prefix section at `:31-42` is a genuine improvement and is
consistent with how the fold commits cite findings.

**Live contradictions, all carried rather than new:**

1. `:165` "23 of the 65" vs `:961` "22 of the 65" (r2/N1; 23 is right).
2. `:562` "55 call sites" vs `:571` "ALL TWELVE production callers" (r2/N2).
3. `:650` "as UNIT tests inside `encode.rs`" vs the two-home table beneath it
   (r2/d1).
4. `:646` Task 7's Files line omits the integration test file (r2/d2).
5. `:730` Task 7 Step 2's `encode::` filter misses the relocated tests (r2/d3).
6. `:781` Task 8's Files line vs Step 5 **and** Step 5b (r2/M12, worsened).
7. `:343` Task 4's Files line vs the `:448` relocation (**r3/M1**, new instance
   of the same shape).
8. `:282` `chunk.rs:70`/`:373` "accept `{4, 8}`" (r2/M7 — `:373` needs no change).

**Resolved by this fold:** the Self-Review's "§8 items 1/2/5/6/7 → Task 8" is now
true (Step 5b); the "Every helper this plan calls is defined by it" overclaim is
gone; "Implement whichever is natural" is gone.

---

## Blocking findings

**r3/I-1 (Important) — the r2/I-F fixture is structurally invalid, and the
funds-regression test fails as written with a message that misdirects at the
decode path.** `Node { tag: Tag::SortedMultiA, body: Body::Variable { k, children } }`
mismatches tag and body: `write_node` dispatches on the body, `read_node` on the
tag, and `Body::Variable`'s own doc comment says it is for `Tag::Thresh` only
while multi-family tags take `Body::MultiKeys`. MEASURED at `37367c1f`: the
plan's fixture encodes to 106 bits and then
`decode_payload` → `Err(TlvLengthExceedsRemaining { length: 9, remaining: 5 })`,
so `assert!(decode_payload(…).is_ok(), "a mint-side refusal reached decode")` is
RED before §6 exists — and its message accuses the decode path, which the plan
warns four times must never carry a refusal. The corrected body
`Body::MultiKeys { k: 2, indices: vec![0, 1, 2] }` was measured to encode (88
bits) and decode `Ok`. Separately, `key_arg` and `descriptor_with` do not exist
in `crates/` at all, and the plan's comment calls the latter an *"existing
`#[cfg(test)]` helper in encode.rs"* — a false claim about the source, which
reached here only because both names were added to `plan-api-check.sh`'s ALLOW
list as symbols the plan CREATES. *(Residual of r2/I-F.)*

**r3/I-2 (Important) — the fold answering r2/I-B re-introduces r2/I-D in the
same document that fixes it.** Task 8's Files line is
`crates/md-codec/tests/liana_unspendable.rs` (`:781`), and new Step 5b puts
`item_7_the_render_reparse_fixpoint_covers_tr_kind_1` there with
`md(&["encode", &t, "--path", "bip48"])` in its body (`:897`). md-codec's
`[dev-dependencies]` are `serde`, `serde_json`, `hex`, `proptest`, `miniscript`
(measured), and `CARGO_BIN_EXE_md` is defined only for md-cli's own targets — the
exact facts Task 4 now cites, 450 lines earlier, to relocate its own pin. The two
Step 5b tests also have **different** crate requirements: `item_2` is pure
md-codec (`descriptor_to_template` resolves at `render.rs:146`, re-exported at
`lib.rs:67`; `to_miniscript_descriptor_multipath_with_network` is Task 5's), while
`item_7` needs the binary — so they cannot share one home, and the plan gives
them one.

**r3/I-3 (Important) — the prescribed §2 tap-tree walk names an API the pinned
miniscript does not have.** `for (_depth, ms) in tap_tree.iter()` (`:431`) fails
three ways against miniscript `13.0.0` rev `ff4732e`: `TapTree::iter()` does not
exist (the accessor is `leaves()`, `src/descriptor/tr/taptree.rs:56`); the item is
`TapTreeIterItem` with `.miniscript()`/`.depth()` accessors, not a `(u8, &Ms)`
tuple (`:161-207`); and `node_to_descriptor`'s local is `script_tree:
Option<TapTree<…>>`, not `tap_tree`. `compressed_33` is undefined and is not
total over `DescriptorPublicKey` — `SinglePubKey::XOnly`, which
`build_nums_internal_key()` produces at `to_miniscript.rs:360-368`, has no
unambiguous 33-byte compressed form. The repo already spells this correctly in
three places (`md-cli/src/parse/template.rs:1671`, `:2684`,
`md-codec/tests/skeleton_key_conformance.rs:461`). The **design is right** —
`leaves()` is depth-first left-to-right per its own doc example, and applying
`for_each_key` per-leaf avoids the `[L, K]` ordering trap this repo documents at
`decompose/walk.rs:36-37` — so this is a spelling fix in the funds-relevant
derivation, not a redesign. *(Residual of r2/I-C / r1/I-5.)*

---

## Non-blocking findings (new this round)

- **r3/M1** — Task 4's Files line (`:343`) names only md-codec's
  `tests/liana_unspendable.rs` while `:448` relocates the pin to
  `crates/md-cli/tests/`; no md-cli test file is named. Same shape as r2/M12.
- **r3/M2** — `Alternative` and `UseSitePath` are public but **not** re-exported
  at md-codec's crate root; the integration test needs
  `use md_codec::use_site_path::{Alternative, UseSitePath};`. Unstated.
- **r3/M3** — Step 5b's `item_2_descriptor_equality_at_the_CORPUS_level` asserts
  `assert_eq!(a, b)` over two **identical** calls to the same pure function: that
  line cannot fail. Its comment claims the descriptor "re-renders identically",
  but nothing re-parses. The test is not vacuous — the `.unwrap()` over the whole
  `tr` corpus and `a.contains('#')` are both real gates — but one assertion is
  dead and the name overstates. A render → `Descriptor::from_str` → render
  fixpoint would deliver what the comment says.
- **r3/M4** — Step 2b's two new tests have no stated home, and
  `encode_payload_at_forced_version` is described as a `#[cfg(test)]` helper **in
  `encode.rs`**, which makes them unit tests — while Step 1's table puts the
  sibling refusal tests in `tests/`. Say which. Also: `encode_payload_inner` takes
  `(d, admission)` only, so "calls `encode_payload_inner` with an overridden
  version" requires a parameter the plan never prescribes adding.
- **r3/M5** — `compressed_33` (`:432`) is undefined and is not total over
  `DescriptorPublicKey` (see r3/I-3).

- **r3/N1** — `plan-api-check.sh`'s ALLOW list and the plan disagree about
  `descriptor_with`: the list exempts it as a symbol the plan CREATES, the plan
  calls it existing. Whichever is intended, the other should change — an ALLOW
  entry added without a grep is how r3/I-1 reached this gate green.

---

## Per-task soundness after the fold

- **Task 1 — UNSOUND** (r2/M8, r2/N4, r2/M10 residue). Unchanged by this fold.
- **Task 2 — SOUND** (r2/M7, r2/N3 only).
- **Task 3 — SOUND.**
- **Task 4 — UNSOUND** (**r3/I-3**, r2/M9, r3/M1); the relocation and the
  per-occurrence ruling are both correct in substance and are this fold's best work.
- **Task 5 — SOUND in design** (r2/M2, r2/N2); the twelve-caller table and the
  refusing-entry-point design hold, and r2/I-C's mechanism gap now lives in Task 4.
- **Task 6 — UNSOUND** (reserved-synthetic-key mechanism gap, r2/M1, r2/M5).
- **Task 7 — UNSOUND** (**r3/I-1**, r3/M2, r3/M4, r2/d1-d3); r2/I-A is properly
  closed and the two-home ruling remains correct.
- **Task 8 — UNSOUND** (**r3/I-2**, r2/M4, r2/M12, r3/M3); Step 4's non-delivery
  record still stands as accurate.
- **Task 9 — SOUND as a gate now** that §6's four refusals each have a test
  (r2/M3 is a coverage Minor, not a gate break).
- **Task 10 — SOUND except r2/M6.**

---

**Ready for implementation: no.** Three Importants block. All three are single
code blocks introduced by this fold and never executed, and none needs a design
decision:

- **r3/I-1** — swap `Body::Variable { k, children: vec![key_arg(…)] }` for
  `Body::MultiKeys { k: 2, indices: vec![0, 1, 2] }` (measured to encode and
  decode `Ok`), and write `descriptor_with` out instead of calling it existing.
- **r3/I-2** — move `item_7_…` to `crates/md-cli/tests/`, next to Task 8 Step 5's
  pin, and say so in Task 8's Files line.
- **r3/I-3** — `for leaf in script_tree.leaves()` / `leaf.miniscript()`, and
  define `compressed_33` (or state that only the `XPub` variant reaches it).

Three edits, all mechanical, all with the correct spelling already visible
elsewhere in this repo. Then re-run `plan-api-check.sh` **after removing
`key_arg|descriptor_with` from its ALLOW list**, since that entry is what let
r3/I-1 through. The carried Minors and Nits do not block; r2/N1's count
contradiction and r2/d1-d3's Task 7 wording are worth one sweep while the three
Importants are being folded.

One note on the round's shape: every finding above came from *executing* the
fold's new blocks, not from re-reading the plan. Three prior rounds read this
document; none of them ran a fixture. The measurable claims in the next fold
should be run before the next reviewer sees them.
