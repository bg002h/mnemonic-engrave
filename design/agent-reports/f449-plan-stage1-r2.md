# R2 — IMPLEMENTATION_PLAN_f449_stage1_md_codec.md (closing gate on the r1 fold)

**VERDICT: 0 Critical / 3 Important / 21 Minor**

Scope: the two questions asked. No fresh audit of plan or spec. Citations
resolved against `descriptor-mnemonic` at `6cbd49d8` and the plan at `d879efe6`
(diff `f50cf015..d879efe6`). Facts listed as already machine-verified in the
brief were not re-derived. Five claims below were settled by running `grep`/`sed`
against the crate, not by reading the plan; each is quoted with its measurement.

**Ready for implementation: NO.** The fold replaced r1's IMP-5 (an honest TBD)
with a fixture constructor that cannot construct the fixture — the same class of
defect as the two rounds before it — and the plan's own acceptance gate (Task 8
Steps 1-2, the funds-relevant half) hangs on it.

---

## Q1 — did the fold address each finding?

### Importants

| # | verdict | note |
| --- | --- | --- |
| **IMP-1** (all-65 enumeration cannot execute) | **ADDRESSED** | Task 0 Steps 2 and 3 now enumerate **52 decodable** vectors with **13 skipped**, and give the real diagnosis (wire version 2, not a different header shape) instead of sending the implementer after a header parser. Both `assert len(d)==...` commands updated to 52. Matches the brief's measured truth. One residue: the `skipped == 13` half of the guard runs nowhere → MIN-18. |
| **IMP-2** (`derive.rs:134` disables kind-1 address derivation) | **ADDRESSED** | Task 5 Step 1 gains a named block: "**TWO call sites switch to the `_with_network` form, not one**", citing `derive.rs:134`, `derive.rs:88-97`, `cmd/address.rs:82`, and stating the consequence for Task 8 Step 2. The set is still not complete → Q2(d)/MIN-20. |
| **IMP-3** (marker rule placed in `walk_tr`) | **ADDRESSED** | Task 6 Step 3 bullet 2 is rewritten: the rule goes in **`substitute_synthetic` (`:1047-1084`)** — "NOT in `walk_tr`" — with the reason (`from_str` fails before `walk_tr` is reached) and the spec's "via a substitution rule" language. Verified: `crates/md-cli/src/parse/template.rs:1047` is the doc comment for `fn substitute_synthetic` at `:1048-1051`. The plan/spec conflict is closed. |
| **IMP-4** (Step 3 names neither API nor hook point) | **PARTIAL** | Step 3 is now six paragraphs: it names all ten production `validate_*` functions, both call sites with their differing policy, the encode-only boundary, the hook point (`Admission::Enforce`), and quotes the regression. **But the four tests that call `validate(&…)` are byte-unchanged** (plan `:1002`, `:1010`, `:1016`, `:1082`), so Step 1's code now contradicts Step 3's own sentence "There is **no function called `validate`**". The hook-point half is fixed; the API half is stated but not applied → downgraded to **MIN-21**. New content in the same step raises Q2(a)/IMP-8. |
| **IMP-5** (`cases.json` cannot feed Task 5 Step 2 or Task 8 Steps 1-2) | **PARTIAL — and the replacement is the round's worst defect** | The chain-code half is fixed: Task 0 Step 1 adds `leaf_tlv_hex`, the 65-byte entries, with a correct justification. Task 5 Step 2's consumer is now satisfiable (see Q2(b)). **Task 8 Steps 1-2 are not**, and the fold's new `descriptor_from_tlv_entries` / `encode_md1_chunks` block converts a visible hole into an invisible one → **IMP-7**. |
| **IMP-6** (identity golden 4-tuple vs. 2-tuple reader; pins 1 of 3) | **PARTIAL** | Task 0 Step 3 states the requirement in prose — "a **4-tuple**. Task 8 Step 3's reader must destructure all four and assert all three captured values". **Task 8 Step 3's code block is byte-unchanged**: still `let golden: Vec<(String, String)>`, still `for (name, id) in golden { assert_eq!(compute_id_by_name(&name), id, …) }`, and `compute_id_by_name` is still defined by no task. The fold documented the fix in one task and did not apply it in the other → **IMP-9**. |

### The 16 Minors

Measured: the fold is 124 insertions / 19 deletions in eight hunks (Status line,
Task 0 Steps 1/2/3, Task 5 Step 1, Task 6 Step 3, Task 7 Step 3, Task 8 Step 1).
**None of the sixteen Minors is in any hunk.** Spot-checks confirming, at
`d879efe6`:

| # | verdict | evidence |
| --- | --- | --- |
| **MIN-1** (`hex_lit` contradiction) | **NOT ADDRESSED** | `:620` forbids it, `:650` still calls `hex_lit::hex!("0250929b74…")`. The 33-byte-vs-x-only ambiguity is also unresolved. |
| **MIN-2** (`keyed_phrase_files()` contradicts Task 0) | **NOT ADDRESSED** | `:199` still reads "Reuse that file's `conformance_dir()` / `keyed_phrase_files()` helpers rather…". Now contradicts the *new* 52-vector rule as well as the old 65. |
| **MIN-3** (clippy/fmt/doc only at Task 10) | **NOT ADDRESSED** | unchanged. |
| **MIN-4..MIN-10** (= r0's M1..M7) | **NOT ADDRESSED** | `:427` still "chunk.rs:70` and `:373` accept `{4, 8}`" (M1); no file named for the new `impl Descriptor` (M2); Step 4 still one paragraph (M3); Task 4 Step 5's "or" unchanged (M4); `grep -c canonicalize` over the plan returns **0** (M5); Task 3 Step 1's `encode_payload`/`decode_payload` arity unchanged (M6); `grep -n "0\.17\.0"` returns nothing (M7). |
| **MIN-11** (`build_liana_unspendable_internal_key` shape ×3) | **NOT ADDRESSED** | Task 5 Step 5 unchanged: still `:333-338`, still `leaves: &[(u8, Miniscript<…>)]`, still "return `Err(failed(...))`" inside a `bool`-returning `for_each_key` closure. |
| **MIN-12** (`LIANA_UNSPENDABLE_MARKER` never given a value) | **NOT ADDRESSED** | consumed at `:616`, `:733`, `:838`, `:912`, `:1405`; defined by no task. |
| **MIN-13** ("55 call sites") | **NOT ADDRESSED** | `:759` still "Zero of the 55 existing sites change"; the commit message in Step 8 still says 55. |
| **MIN-14** (`policy_shape.rs` collapse + `:117-119`) | **NOT ADDRESSED** | Task 8b unchanged. |
| **MIN-15** (`error.rs` missing from Task 2 Files; `expected 4` assertion) | **NOT ADDRESSED** | unchanged. |
| **MIN-16** (Self-Review "No TBDs" + the vanished "Task 8 Step 0") | **NOT ADDRESSED — and now self-contradicting** | `:1403` still claims "No TBDs. Every code step carries real code"; `:1407` still routes the `GOLDEN_ACCEPTED` extraction to "**Task 8 Step 0**", a step Task 0 replaced. That paragraph now also contradicts the fold's *new* Task 8 Step 1, which says the chunks are "**CONSTRUCTED here, not vendored**". Two adjacent paragraphs of the plan's own self-review disagree with each other and with the task they describe. |

**Tally: 3 ADDRESSED, 3 PARTIAL, 16 NOT ADDRESSED.** Every Important got a
response; no Minor did.

---

## Q2 — is the new content sound?

### (a) The encode-side-only constraint (Task 7 Step 3)

Four sub-questions.

**1. Does `Admission::Enforce` actually gate where the plan says? YES in
substance, NO in citation.** Measured at `6cbd49d8`:

```
encode.rs:143  fn encode_payload_inner(d: &Descriptor, admission: Admission) -> …
encode.rs:147  crate::validate::validate_placeholder_usage(&d.tree, d.n)?;
encode.rs:149  crate::validate::validate_multipath_consistency(…)?;
encode.rs:153  crate::validate::validate_tap_script_tree(t)?;
encode.rs:166  if admission == Admission::Enforce {
encode.rs:167      crate::validate::validate_origin_key_consistency(d)?;
encode.rs:169      crate::validate::validate_no_duplicate_key_slots(d)?;
encode.rs:170  }
```

The load-bearing claim — a real `Admission::Enforce` block exists inside
`encode_payload_inner` with exactly **two** validators behind it, and a
`SkipPolicy` path (`encode_payload_for_identity`, `:139`) that the identity/
reassemble leg uses (`identity.rs:45`) — is **true**. The cited ranges are off:
`encode.rs:152-172` excludes `:147` and `:149` and its upper bound is
`let mut w = BitWriter::new()`; decode's validators run `decode.rs:118`-`:153`
(`validate_xpub_bytes` is the last), not `:120-150`. → MIN-17.

**2. Is that the right boundary? Partly — and the stated reason is false for
this kind. → IMP-8.**

**3. Would the new test compile? NO.** Three defects in six lines:

- `encode_payload_unchecked` **does not exist**. `grep -rn "encode_payload_unchecked" crates/` returns nothing. The nearest real thing is
  `encode_payload_for_identity` (`encode.rs:139`), which is **`pub(crate)`** — so
  the plan's test, living in `crates/md-codec/tests/liana_unspendable.rs` (an
  external crate), cannot reach it without a public-API widening the plan does
  not authorise.
- `decode_payload(&bytes)` takes one argument; the signature is
  `decode.rs:66  pub fn decode_payload(bytes: &[u8], total_bits: usize)`. Same
  defect r1 raised as M6/MIN-9 for Task 3 Step 1, reintroduced in new code.
- `tr_liana_with_sortedmulti_a_leaf()` is undefined (shared with Task 7 Step 1;
  fixture convention, not a new defect).

**4. Can it fail? Yes for all three misplacements — but not at the leg the plan
names, and the crate already owns a better version of this test.** If
`encode_payload_unchecked` is read as the `SkipPolicy` encode, the test fails
when the rule is hooked in decode's list (decode refuses) and when it is hooked
in `encode_payload_inner` *outside* the guard (the `.unwrap()` panics). So it has
failure power. Two things are nonetheless wrong with it:

- **Prose and code disagree on what is tested.** Step 3 says "add a test that a
  **chunk set** carrying the refused shape still *decodes*". The code decodes raw
  payload bytes. The 2026-09-19 regression was in `chunk::reassemble` →
  `compute_md1_encoding_id` → `encode_payload` — `decode_payload` was never on
  that path and never applied admission policy. The test as written does not
  reach the leg its own comment cites ("The 2026-09-19 regression in one
  assertion").
- **The plan never mentions `crates/md-codec/tests/mint_policy_does_not_reach_decode.rs`**, which already exists, pins this exact class at the right leg
  (`hashing_it_is_not_minting_it` calls `compute_md1_encoding_id`), carries a
  **control** (`minting_it_is_still_refused`, "Without this the test below could
  pass because the rule stopped firing anywhere"), needs no API widening, and
  says of itself "This pins the CLASS… and whichever rule gets added next". The
  plan re-invents it worse. Note the existing file's control uses
  `contradictory_card()` (a `wsh`/`sortedmulti` shape), so it does **not** cover
  the four new kind-1 rules — the coverage really is owed, just not in this
  shape. → MIN-19 (folded with `md_argv`, below, as fixture/API naming).

**No plan/spec conflict here.** SPEC §6's minimum-version row says REFUSE "at
**encode** … Accepted at decode so old payloads never become invalid"; the other
three rows say only "REFUSE" and are silent on the stage. The plan resolves a
silence rather than contradicting the GREEN spec.

### (b) Constructed kind-1 chunks (Task 8 Step 1 + Task 5 Step 2) — the highest-risk item

**Does `encode_md1_chunks` exist under that name? NO.** Measured:
`grep -rn "encode_md1_chunks" .` over the whole repo returns **zero** hits. The
crate's chunk-set producer is `crates/md-codec/src/chunk.rs:240`
`pub fn split(d: &Descriptor) -> Result<Vec<String>, Error>`; the only two
`pub fn encode*` in `encode.rs` are `encode_payload` (`:99`) and
`encode_md1_string` (`:229`). The plan names a nonexistent function in a
nonexistent module path (`md_codec::encode::`). The mechanism exists, so this
alone is mechanical.

**Can a `Descriptor` be constructed this way from TLV bytes alone? NO — and this
is the blocking half.** `Descriptor` (`encode.rs:17-28`) has five public fields:
`n`, `path_decl`, `use_site_path`, `tree`, `tlv`. Direct construction from an
integration test is possible and already done
(`tests/mint_policy_does_not_reach_decode.rs:57-84`). What the plan's two inputs
— `case.leaf_tlv_hex` and `InternalKey::LianaUnspendable` — cannot supply is
everything except `n` and `tlv.pubkeys`. Measured against a real accepted case
(`design/evidence/composer-fable-r0/fable-liana-parse-out-v15.jsonl`,
`preset-kofn-recovery-tr`, `ok: true`):

```
tr(xpub661MyMwAqRbcFswVugWF…/<0;1>/*,
   {multi_a(2,[73c5da0a/48'/0'/0'/3']xpub6DXuQW1Q2JpZywei…/<0;1>/*,
              [3f635a63/48'/0'/0'/3']xpub6DXuQW1Q2Jpa1hNt…/<0;1>/*,
              [66d455ea/48'/0'/0'/3']xpub6DXuQW1Q2JpZyteD…/<0;1>/*),
    and_v(v:pk([73c5da0a/48'/0'/1'/3']xpub6DXuQW1Q2JpZzLV9…/<0;1>/*),
          older(26280))})#8jc8gq6v
```

Four things in that string are load-bearing for Step 1's gate (**byte-identical
equality including the checksum**) and are in neither input nor in the
`cases.json` schema Task 0 Step 1 defines (`name`, `accepted`, `leaf_tlv_hex`,
`leaf_pubkeys_hex`, `expected_xpub`, `descriptor_with_checksum`,
`liana_receive[3]`, `liana_change[3]`, source SHA):

1. **`tree`** — the two-leaf taptree `{multi_a(2,…), and_v(v:pk(…),older(…))}`,
   with its threshold. Four TLV entries do not determine a tree; §8.6's own
   structure-independence pin is the proof that they cannot (three different
   trees over the same four keys give the same `expected_xpub`).
2. **`older(26280)`** — a wallet parameter appearing nowhere in the schema.
3. **`tlv.fingerprints`** — `73c5da0a`, `3f635a63`, `66d455ea`, `73c5da0a`.
   Origins are explicit in the output and `validate_explicit_origin_required`
   runs on both encode and decode.
4. **`path_decl` must be `PathDeclPaths::Divergent`** — keys 0-2 sit at
   `48'/0'/0'/3'` while key 3 sits at `48'/0'/1'/3'`. A single shared origin path
   cannot express it, and no field carries either path.

So `descriptor_from_tlv_entries(&case.leaf_tlv_hex, InternalKey::LianaUnspendable)`
cannot produce a descriptor that renders to `case.descriptor_with_checksum`, and
Task 8 Step 2's address equality against Liana's own triples inherits the same
gap. **IMP-5 survives → IMP-7.**

**The one place it does work.** Task 5 Step 2's
`descriptor_with_real_keys_at_liana_kind()` asserts only
`s.starts_with("tr(xpub661MyMwAqRbcFswVugWF")` and `!s.contains("50929b74")`.
Because the derived internal key is a function of the leaf pubkeys **in order
only** (§8.6), an arbitrary tree over the right keys satisfies it. The helper is
sufficient for Task 5 Step 2 and insufficient for Task 8 Steps 1-2 — the fold
solved the consumer it named in its justification and not the one in the code
block.

Also in this block: `md_argv(&argv)` is undefined and does not match the crate's
convention — every md-cli integration test defines `fn md(…)`
(`cmd_descriptor.rs:21`, `acceptance_walks.rs:163`, and seven others). → MIN-19.

### (c) The 52/13 corpus rule and its asserted-count guard

**The rule is right and the reason is right.** The brief's measurement (65
`*.phrase.txt`, 13 at wire version 2 failing reassemble, 52 decodable) is what
the plan now states, and excluding the 13 is *correct* for
`every_existing_v4_identity_is_byte_preserved`, whose subject is v4 identities.
The plan is also right that a silent `try/except` would let a decode regression
shrink the corpus invisibly, and right to forbid it.

**The guard that runs is weaker than the guard that is described.** The prose
gives `assert ok == 52 and skipped == 13`; the two commands that actually execute
assert only `len(d)==52` on the dump output. Enumerate the drift shapes:

| drift | caught by `len(d)==52`? |
| --- | --- |
| a vector becomes undecodable (65 files, 51/14) | yes |
| a new decodable vector added (66 files, 53/13) | yes |
| **a new *undecodable* vector added (66 files, 52/14)** | **no** |

The third row is exactly "the skip set silently grew", the thing the paragraph
claims to prevent. The fix is a `total` assertion the example can emit; the plan
has the assertion, just not in the executed command. → MIN-18. Two further
residues: no task specifies the skip logic inside `examples/dump_encodings.rs`
beyond "no try/except", and Task 1 Step 2 still points the implementer at
`keyed_phrase_files()` (46 files) — MIN-2, now contradicting 52 as well as 65.
Task 0's Files list does create both examples, so there is no missing-example gap.

### (d) The two `_with_network` call sites — is that the complete set?

**No. There is a third production caller, and it fails silently.** Measured —
all `to_miniscript_descriptor*` call expressions outside `crates/*/tests/`:

```
md-codec/src/derive.rs:134                      production   (plan covers)
md-cli/src/cmd/descriptor.rs:200,:201           production   (plan covers)
md-cli/src/cmd/vectors.rs:193                   production   (NOT covered)
md-cli/src/seat/{disposition,matching,compose}.rs  all inside #[cfg(test)]
                                                (:162, :264, :300 respectively)
```

`cmd/vectors.rs:193` sits in `fn conformance_json` (`:121`); the file's
`#[cfg(test)]` does not start until `:222`, so it is production. Its shape
matters:

```rust
let desc = match md_codec::to_miniscript::to_miniscript_descriptor(d, chain) {
    Ok(x) => x,
    Err(_) => continue,      // single-path vectors have no chain 1
};
```

Under Task 5's refusal a kind-1 descriptor returns `Err` for **both** chains, so
the loop `continue`s twice and the record emits `"chains": {}` — no descriptor,
no addresses, no error, and the comment ascribes the skip to a cause that is not
the one that fired. (`derive_address`'s `Err(_) => break` two lines down is the
same shape.) r1 described this site as "would report a kind-1 row as an error";
it does not report anything.

**Address routes: complete.** `derive_address` (`derive.rs:92`) is the only
address entry point in the crate — every other occurrence is a call *to* it or a
doc reference — so `derive.rs:134` really is the whole address surface, and the
plan covers it.

Graded **Minor** (MIN-20), not Important, for one reason: no task in this stage
puts a kind-1 descriptor through `md vectors`, and no CI job regenerates the
conformance corpus. What is wrong today is the plan's completeness *claim*
("**TWO call sites switch to the `_with_network` form, not one**"), which is the
same shape of claim r1 falsified in r0 ("only `cmd/descriptor.rs`").

---

## Important findings (blocking)

### IMP-7 — Task 8 Step 1's `descriptor_from_tlv_entries` cannot build the fixture, and `encode_md1_chunks` does not exist. IMP-5 survives its fold.

**Lands in:** Task 8 Step 1, Task 8 Step 2, Task 0 Step 1's schema, and the
Self-Review's "No TBDs" claim at `:1403`.

**Reproduction.** Two independent measurements, both above in Q2(b):
`grep -rn "encode_md1_chunks" .` over `descriptor-mnemonic` at `6cbd49d8`
returns zero hits (the real producer is `chunk.rs:240 pub fn split`); and the
four items — taptree shape, `older(26280)`, per-key fingerprints, and a
`PathDeclPaths::Divergent` origin declaration — that
`preset-kofn-recovery-tr`'s `descriptor_with_checksum` requires are absent from
both the helper's arguments and `cases.json`'s schema.

**Why it matters, and why it is worse than what it replaced.** r0's
`/* the kind-1 chunks for this shape */` was a hole an implementer could see.
The fold's replacement reads as solved — "Build the `Descriptor` from
`cases.json`'s 65-byte entries … and encode it; **that IS the chunk set**" — and
asserts it twice more ("`descriptor_from_tlv_entries` is a test helper built once
in this task and reused by Task 5 Step 2", "No TBDs. Every code step carries real
code"). The gate it blocks is Task 8 Steps 1-2: byte-identical descriptor
equality including checksum, and the three-way address equality against Liana's
own receive/change triples — the funds-relevant half of the acceptance vectors,
and the reason this stage exists.

**Not prescribing a fix**, but recording the two constraints any fix must meet,
because the plan should decide this rather than the implementer: (i) the tree,
timelock, fingerprints and divergent origin paths must come from somewhere —
`descriptor_with_checksum` contains all four, so parsing it is one route, and
widening `cases.json` is another; (ii) whatever route is chosen must not be the
one that runs the string back through the md1 encoder it is supposed to be
testing, or the gate pins output to itself.

### IMP-8 — the encode-only boundary is justified by a premise that is false for kind 1, and it disarms §6 row 1's stated purpose

**Lands in:** Task 7 Step 3 (new in this fold), SPEC §6 rows 1 and 2, Task 5
Step 5's `build_liana_unspendable_internal_key`.

**What is right, recorded first so it is not re-litigated.** Not putting the
rules in `decode_payload_with_opts`'s validator list is correct; hooking at
`Admission::Enforce` is correct and the block exists (`encode.rs:166-170`); and
SPEC §6's minimum-version row independently requires encode-side for that rule.

**The defect is the reason given, and what the reason hides.** The plan argues:
*"A refusal added to the decode side here would make **existing plates
unreadable**."* Six paragraphs earlier the same task states the opposite fact —
*"No md1 card of this kind exists anywhere yet — this stage is what creates the
first one."* Wire version 8 does not exist before this stage, so for rules scoped
to `kind = 1` the population the doctrine protects is **empty**. The plan
transplants a doctrine from a real regression (2026-09-19, kind-0 `wsh` cards
already in metal) onto a kind where its premise cannot hold, and so never asks
the question the doctrine exists to answer: *where should a rule fire when the
plate was minted by something that is not this encoder?*

**Constructed failure.** SPEC §6 row 1 gives its own reason for the
`sortedmulti_a` refusal: *"a belt against a **port error** … `MultiALeafScript`
(`address/taproot_script_path.go:261-270`) sorts the serialized derived x-only
keys … and a port that feeds that already-sorted list into the recipe silently
diverges"*. A plate minted by that port reaches this crate through **decode**,
not encode. Under the plan's boundary `md` decodes it, then Task 5 Step 5 derives
the internal key from `for_each_key` order while the device derived it from
sorted order — two different xpubs, two different wallets, no error on either
side. The belt is buckled on the side the port cannot reach.

Row 2 is the same shape and sharper, because the plan's own code makes it
concrete: `build_liana_unspendable_internal_key` hardcodes the internal key's
`DerivPaths` to `<0;1>` unconditionally. Decode a kind-1 payload whose use-site
is `<2;3>` — refused at mint, accepted at decode — and `md descriptor` renders
`tr(xpub…/<0;1>/*, {leaves at <2;3>/*})`: a well-formed descriptor whose internal
key and leaves disagree about which alternative is receive. Liana pairs
positionally and derives from `0/i`; `md address` follows the use-site. Neither
errors.

**What is missing is a third option the plan never considers.** "Encode or
decode" is not the whole space. The wrong address is produced at the
**render/derive** boundary — `to_miniscript_descriptor_*_with_network` and
`derive_address` — which is neither minting nor reading, and where a refusal
keeps the plate readable (the doctrine's actual goal) while blocking the funds
error. The plan should either place the kind-1 rules there, or state the residual
risk explicitly and pin it with a test. Today it does neither, and §6's two
funds-relevant refusals silently do not cover the path their own rationale names.

### IMP-9 — the identity-preservation gate is fixed in one task's prose and left broken in the other's code; as written it still pins 1 of the 3 values it captures

**Lands in:** Task 0 Step 3, Task 8 Step 3.

**Reproduction.** Task 0 Step 3 (new text): *"`examples/dump_ids.rs` emits
`[[name, wallet_policy_id, template_id, phrase], …]` … a **4-tuple**. Task 8
Step 3's reader must destructure all four and assert all three captured values;
r0's reader took a 2-tuple and so pinned **one of three**."* Task 8 Step 3, at
`d879efe6`, is byte-identical to `f50cf015`:

```rust
let golden: Vec<(String, String)> =
    serde_json::from_str(include_str!("golden/pre_refactor_ids.json")).unwrap();
for (name, id) in golden { assert_eq!(compute_id_by_name(&name), id, "{name}"); }
```

`serde_json` deserialising a 4-element array into a 2-tuple fails with "invalid
length 4, expected a tuple of size 2", so the test panics on its first line; and
once repaired the loop still compares one value, leaving `template_id` and the
12-word phrase captured but unchecked. `compute_id_by_name` remains defined by no
task.

**Why it stays Important rather than dropping to Minor.** The failure mode is
loud, not silent — which is why this is not Critical. But the repair an
implementer reaches for first is the wrong one: changing the *golden* to a
2-tuple removes the panic and permanently discards two of three pinned values,
including the 12-word phrase an operator reads off steel. §8.5's "every existing
v4 identity byte-preserved" is the half that protects plates already minted, and
the plan's own Task 9 mutation 6 calls the identity sites "the funds-relevant
one". A plan whose two halves disagree about the shape of this gate is a plan
that can be followed correctly and still ship the weak version.

---

## Minor

Sixteen carried from r1, verified unchanged above: **MIN-1** (`hex_lit`),
**MIN-2** (`keyed_phrase_files()`), **MIN-3** (late clippy/fmt/doc),
**MIN-4..MIN-10** (r0's M1-M7), **MIN-11** (`build_liana_unspendable_internal_key`'s
three shape defects), **MIN-12** (`LIANA_UNSPENDABLE_MARKER` undefined),
**MIN-13** ("55 call sites"), **MIN-14** (`policy_shape.rs`), **MIN-15**
(`error.rs`/`expected 4`), **MIN-16** (Self-Review — now also contradicting the
fold's own Task 8 Step 1).

Five new, all in fold-added text:

- **MIN-17** — the fold's two new citations are off. `encode.rs:152-172` omits
  the validators at `:147` and `:149` and ends on `let mut w = BitWriter::new()`;
  decode's validators run `decode.rs:118`-`:153`, not `:120-150`. The claim they
  support ("two of them behind `Admission::Enforce`") is true.
- **MIN-18** — Task 0 Step 2's executed guard is `assert len(d)==52` only. The
  `skipped == 13` half appears in prose and runs nowhere, so the one drift shape
  the paragraph promises to catch — a new *undecodable* vector, total 66, ok 52,
  skipped 14 — passes silently. The example's skip logic is also unspecified
  beyond "no `try/except`".
- **MIN-19** — fixture/API naming in fold-added code: `encode_md1_chunks` (no
  such symbol; `chunk::split`), `encode_payload_unchecked` (no such symbol;
  `encode_payload_for_identity` is `pub(crate)`), `decode_payload(&bytes)` (two
  args), `md_argv` (the crate's convention is `fn md(…)`, nine existing files).
  Listed here rather than under IMP-7/IMP-8 because each is a mechanical rename;
  the blocking halves of those findings are not.
- **MIN-20** — `cmd/vectors.rs:193` is a third production caller of the
  network-less form, unnamed by the plan's "TWO call sites … not one". With the
  refusal live it `continue`s on both chains and emits `"chains": {}` with no
  error, under a comment blaming a different cause. Minor because no task in this
  stage routes a kind-1 descriptor through `md vectors`.
- **MIN-21** — Task 7 Step 1's three tests and Task 8 Step 2b's third test still
  call `validate(&…)` while Task 7 Step 3 now states in the same task that no such
  function exists. (This is IMP-4's residue, downgraded: Step 3 now names the ten
  real functions and both call sites, so the implementer is told what to do —
  just not in the code blocks.)

---

## What the fold got right, recorded so it is not re-litigated

- **IMP-1 is properly closed**, and the correction is the better kind: it
  overturns r0's premise with a measurement rather than patching the number, and
  the 52-vector population is the right subject for a v4-identity golden.
- **IMP-2 is properly closed**, with the consequence for Task 8 Step 2 spelled
  out and the one-line ingredient (`network` already in `derive_address`'s scope)
  named.
- **IMP-3 is properly closed** and the plan/spec conflict with it:
  `substitute_synthetic` at `template.rs:1047-1084` is the textual stage, and the
  reason given (`from_str` fails before `walk_tr`) is the correct one.
- **Task 7 Step 3's hook point is right** — `Admission::Enforce` at
  `encode.rs:166-170` exists, the identity/reassemble leg goes through
  `SkipPolicy` (`identity.rs:45`), and quoting the 2026-09-19 regression into the
  plan is the right instinct even though the conclusion drawn from it is
  over-generalised (IMP-8).
- **`leaf_tlv_hex` is a real fix** for the half of IMP-5 it addresses, with a
  correct justification (a TLV entry needs its chain code), and it makes Task 5
  Step 2's fixture genuinely constructible.
- **The 52/13 rule's substance is correct** and the explicit ban on a silent
  `try/except` is the right call.

---

**ready for implementation: no**
