# Final whole-branch review — `coord-compat-1a` (`b2c5d693..ea0aa322`)

Reviewer: opus, final gate before operator handoff.
Brief: `design/agent-briefs/coord-compat-1a-final-review.md`.
Worktree: `/scratch/code/shibboleth/dm-worktrees/coord-compat-1a`, branch `coord-compat-1a`.
Diff: 10 commits, 11 files, +3665/−47.

**Verdict: NOT GREEN — 0 Critical / 3 Important / 5 Minor / 2 Nit.**

All three Importants are *missing gates on correct code*, not wrong results.
The branch as committed computes the right answer everywhere I could reach.
What it does not do is prevent the next edit from computing the wrong one —
and in two of the three cases I constructed the wrong answer by mutation and
showed 581 tests cannot see it.

## Gate state (measured, this session)

| check | result |
| --- | --- |
| `cargo nextest run -p md-codec --no-fail-fast` | **581 passed / 2 skipped / 0 failed**, 22.6s |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean, clippy **0.1.85** (pinned) |
| `cargo fmt --all --check` | clean |
| `git status --porcelain` after all experiments | **empty** — worktree left clean |

Every mutation below was applied, run against the **whole** `md-codec` suite,
and reverted with `git checkout -- crates/md-codec/src/`. Tree verified clean
after the last one.

---

## Important

### I-1 — the `sh(wsh(...))` unwrap has zero effective coverage; deleting it changes the SkeletonKey of a wire-reachable policy and reds 0 of 581

`policy_shape.rs:282` unwraps one more level when the root is `Sh` and its
child is `Wsh`, so the branch walk descends into the script that actually
runs rather than the wrapper around it.

**Mutation.** `if false && tree.tag == Tag::Sh && inner.tag == Tag::Wsh {`
→ **581 tests run: 581 passed, 2 skipped. Zero failures.**

**Counterexample, confirmed WIRE-REACHABLE, not hand-built.**
`sh(wsh(or_d(sortedmulti(2,@0,@1,@2), and_v(v:pkh(@3), older(26280)))))`
encodes to **18 bytes / 139 bits** and **strict-decodes** (`decode_payload`,
default opts) back to a byte-identical tree.

| | branches | slots | SkeletonKey tail |
| --- | --- | --- | --- |
| committed code | 2 | `[[0,1,2],[3]]` | `…\u{1f}[[][]][]\u{1f}NotTaproot` |
| unwrap removed | 1 | `[[0,1,2,3]]` | `…\u{1f}[[]][]\u{1f}NotTaproot` |

The collapsed form **merges two independently satisfiable spend paths into
one branch** — precisely the undercount Task 4's I-5 ruling implemented the
`tr` key-path branch to prevent, arriving here through a different code path.
And the key changes, so an evidence table built before such an edit silently
stops matching afterwards.

**Why nothing reaches it.** The unwrap is a *no-op* on every fixture that
exercises it, so its removal is unobservable:

- The only `sh(wsh)` unit fixture is `common::sh_wsh_2of3()` =
  `sh(wsh(multi(2,@0,@1,@2)))`. Single-branch: with or without the unwrap,
  `plain_multi` fails on `Tag::Wsh` (not in its wrapper-unwind list), then
  `sole_multi` finds the same one multi and the same `nkeys == slots.len()`
  holds, so the Branch is identical either way.
- All **6** `sh(wsh(...))` records in the 46-vector conformance corpus are
  `sh(wsh(sortedmulti(...)))` — measured by
  `grep -ho 'sh(wsh([a-z_]*' keyed_*.conformance.json | sort | uniq -c`.
  Also single-branch.

So the unit suite and the conformance corpus miss it for the *same* reason,
from two directions. That is the shape a per-task review cannot see: Task 1
wrote the unwrap, Task 5 built the corpus, and neither could observe that the
other's coverage did not reach it.

**Second-order.** This also leaves the two implementations of the sh(wsh)
predicate free to drift with only one side reporting: `skeleton::root_kind`'s
`children.first() == Some(Tag::Wsh)` (`skeleton.rs:166`) **is** guarded —
flipping it reds `root_and_inner_wsh_are_read_correctly`, per Task 4's I-4
fix — while `policy_shape`'s is not. `skeleton.rs:140-148` argues the two are
"different questions over the same one bit of tree shape"; that argument is
fine, but it is currently load-bearing with a gate on only one side.

**Fix.** One fixture: `sh(wsh(or_d(...)))` asserting `branches.len() == 2`
and the two slot sets. The descriptor above is ready to lift.

### I-2 — the `older-blocks`/`older-units` band label is unguarded; collapsing it gives two wire-reachable, semantically different policies ONE SkeletonKey, and reds 0 of 581

**Mutation.** `render.rs:139`,
`LockKind::OlderUnits => "older-units"` → `"older-blocks"`
→ **581 tests run: 581 passed, 2 skipped. Zero failures.**

**Counterexample, both halves encode and strict-decode round-trip identical.**
`wsh(and_v(v:pkh(@0), older(10)))` vs `wsh(and_v(v:pkh(@0), older(0x40000a)))`:

| | committed code | with the mutation |
| --- | --- | --- |
| `older(10)` template | `…older(older-blocks#1))` | `…older(older-blocks#1))` |
| `older(0x40000a)` template | `…older(older-units#1))` | `…older(older-blocks#1))` |
| **same SkeletonKey?** | **false** | **TRUE** |

`lock_from_wire` strips `SEQUENCE_TYPE_FLAG` before the value reaches the
class counter, so once the labels collapse the two operands render
byte-identically. 10 blocks (~100 min) and 10 × 512 s (~85 min) are different
spend conditions. **This is the false-evidence-match the plan exists to
prevent**, and it is invisible to the whole suite.

**Where the gap actually is.** Not in `lock_from_wire` — that *is* guarded:
disabling its `SEQUENCE_TYPE_FLAG` test reds
`lock_bands_round_trip_through_compose_lock_operand` (1 failure, measured).
The gap is one layer up, in the **label the key carries**. Measured: the
string `"older-units"` occurs **exactly once in the crate**, at its own
definition (`render.rs:139`). No test anywhere asserts on it.

The asymmetry is Task 2's fix round: it added "after() coverage in both
bands" (`after_locks_carry_height_and_time_bands`, which straddles
`LOCKTIME_THRESHOLD` by exactly 1 and correctly reds when I collapse
`AfterTime` alone). It did not add the `older` equivalent. Both bands of
`after` are guarded; neither band of `older` is.

**Fix.** The symmetric test: one descriptor with `older(10)` and one with
`older((1<<22)|10)`, asserting the templates differ and both labels appear.

### I-3 — no test pins any literal SkeletonKey byte; five independent serialization mutations each change the key for real fixtures and red 0 of 581

All five applied separately, whole suite each time:

| mutation | site | result |
| --- | --- | --- |
| `for path in fp.iter().rev()` | `render_fp_partition` | 581 pass |
| paths rendered then `sort()`ed | `render_fp_partition` | 581 pass |
| drop the `,` between slots | `render_slots` | 581 pass |
| drop the outer `[`/`]` | `render_group_list` | 581 pass |
| emit `key_partition` before `fp_partition` | `skeleton_key` | 581 pass |

**Why they are all invisible.** Grep of every `as_str()` use in the branch
(`skeleton_key.rs`, `skeleton_key_conformance.rs`, `dump_skeleton_keys.rs`):
one `contains('\u{1F}')` check, one self-equality check, one `println!`.
**Every other key assertion compares two *computed* keys** — `assert_ne!`
between two skeletons, or chunk-route vs descriptor-route. Any change that
applies uniformly to both sides of such a comparison cannot fail it. The
Task 4 review already made this point about the conformance gate
("several collision mutations stay green under it by construction"); it is
equally true of every other key test in the branch.

**Why it matters more than the individual mutations.** The collision-freedom
proof — formal injectivity, 190,032 enumerated `(fp,kp)` pairs, 20,000
realistic descriptors — is the single most important property this branch
has, and it exists **only in a review transcript**, against code that has
since been edited twice (Task 4 fix round, Task 5 fix round). Nothing
committed preserves it. Plan 1b will key a measured-evidence table by these
exact bytes; a table keyed by a grammar with no golden test is a table that
can be silently invalidated by a tidy-up.

I checked whether any of the five is *itself* a reachable collision. None is,
because `template` is in the key and pins the branch count and the slot
numbers present, which constrains what the partitions can render. So these
are guard failures, not live defects — but the guard is the thing plan 1b
depends on.

**Fix.** One line. A literal expected string for one fixture, e.g. the
measured key for `sh(wsh(or_d(…)))`:

```
sh(wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pkh(@3/<0;1>/*),older(older-blocks#1)))))\u{1f}[[][]][]\u{1f}NotTaproot
```

A fixture with a non-empty, asymmetric `fp_partition` and a non-empty
`key_partition` would pin all five at once; `common::seated(...)` plus
`common::seated_same_key(&[0,3])` between them supply the material.

---

## Minor

### M-1 — `plain_multi` and `sole_multi`'s caller give contradictory k/n for a duplicate-index multi (the FOURTH "one rule, two implementations")

The brief asked for a fourth instance. This is it, and it is the deferred
Task 1 minor whose stated rationale does not hold.

Measured, both shapes encode:

| descriptor | path taken | `k` | `n` | `slots` |
| --- | --- | --- | --- | --- |
| `wsh(multi(2,@0,@0,@1))` | `plain_multi` | **2** | **3** | `[0,1]` |
| `wsh(and_v(v:older(9), multi(2,@0,@0,@1)))` | `sole_multi` | **0** | **0** | `[0,1]` |

(`encode_payload` on the first: OK, 9 bytes / 66 bits.)

One multi, accounting for every key its branch references, in two spellings,
answered two ways. The mechanism: `br.slots` comes from a `BTreeSet`, which
**deduplicates**, so `sole_multi`'s guard `nkeys as usize == br.slots.len()`
rejects a duplicate-index multi (3 ≠ 2) while `plain_multi` has no guard at
all and reports `n = 3` for a branch with 2 distinct keys.

The ledger's deferral reasoning — *"a multi reached via the wrapper-unwind
chain cannot coexist with sibling key material in the same branch"* — is
**true and addresses a different failure mode**. It covers key material
*beside* the multi; it does not cover duplication *inside* it. "Safe by
construction" is therefore not established for the case the guard on the
other path actually catches.

Not in the SkeletonKey (`k`/`n`/`sorted` are not serialized), so no evidence
impact today. But `Branch::k`'s own doc says zero means "one that does not
account for the branch's keys", and plan 1b reads `shape.branches` — a rule
reading k-of-n gets "2-of-3" for a policy with two distinct keys, which is
the F-218 shape `validate_no_duplicate_key_slots` exists to refuse one layer
up. **Owning phase: before plan 1b's first rule reads `k`/`n`.**

### M-2 — `policy_shape.rs`'s module header says "Two extensions over the Go original"; there are now three

The header (`policy_shape.rs:27-45`) enumerates exactly two: `Branch::slots`
retained, and `KeyPathKind::Xpub` renamed. Task 4's I-5 added a third — the
`tr` key-path branch — which the code's own inline comment at
`policy_shape.rs:254-258` explicitly labels *"NOT ported from the Go:
`policy_shape.go`'s `walkTapTree` appends one branch per LEAF only and never
adds one for the key path."*

The design doc §1A **was** corrected for this same fact (commit `1a0a5b9a`);
the module header was not. Under the Rust-primary rule this header is the
canonical list of what the Go port must converge to, so an under-count here
is a convergence instruction that silently omits a divergence. Classic "a
diff falsifies text it never touches": Task 4's fix invalidated Task 1's
header, and no per-task review had both in scope.

### M-3 — the design doc still declares a four-valued `KeyPathKind`

`design/DESIGN_coordinator_compatibility.md:155-158`:

```
/// The three-way internal key (r2 I-9). ...
enum KeyPathKind { Nums, UnspendableXpub, Spendable, NotTaproot }
```

A doc comment saying "three-way" over an enum with four variants —
internally inconsistent, contradicting the same document's own prose at
:202-215 (which correctly states `NotTaproot | Nums | Xpub` and cites the
measurement), and contradicting the shipped code. The ledger recorded *"the
design needs the correction"*; the prose got it and the type listing did not.
This block is what a plan-1b implementer reads first as the type spec.

### M-4 — per-kind scoping of hash equality classes is unguarded

Mutation: route all four `HashKind`s to the single `sha256` list
(`class_for_hash`) → **581 pass**.

Benign *direction*: sharing one list can only make classes more distinct, so
it causes a missed match (Unproven), never a false one. But the reason it is
invisible is worth recording — the instrumented census below shows
`hash256`/`ripemd160`/`hash160` class arms are reached only via the
conformance corpus, where nothing asserts on the resulting labels.
`render_abstract.rs` exercises `sha256` only.

### M-5 — `KeyPathKind`'s three-way label spelling is only half-pinned

Mutation: `KeyPathKind::Nums => "NotTaproot"` → **581 pass**.
`key_path_kind_is_part_of_the_key_independent_of_template` distinguishes only
`Nums` from `Xpub`. Harmless today (the template always disambiguates a
taproot policy from a non-taproot one), but the token is in the key and one
of its three spellings is unpinned.

---

## Nit

### N-1 — the brief's "one dead `root_kind` arm" is three zero-coverage paths

Instrumented census (marker appended to a file from each arm, whole 581-test
suite, one run):

| arm | hits |
| --- | --- |
| `root_unrecognized` (`other => Err(UnrecognizedRoot(other))`) | **0** |
| `root_sh_nonchildren` (`Sh` with a non-`Children` body) | **0** |
| `skeleton_render_ERR` (`SkeletonError::Render`) | **0** |

All three are documented defensive arms, unreachable for a decoder-produced
`Descriptor` and correctly kept as typed errors rather than panics for a
hand-built one. No change wanted — only the count in the ledger is low.

### N-2 — `render_abstract.rs` never asserts that two DIFFERENT digests get different classes

`digests_carry_a_class_too_symmetric_with_locks` uses `[0x11; 32]` twice, so
only the "equal values share a class" half is pinned directly. The other half
is caught incidentally (forcing `class_index` to always return 1 reds the
*lock* test), but the digest half has no direct gate.

---

## Coverage census (method note)

Rather than guess at dead code, I instrumented 22 suspect arms across
`policy_shape.rs`, `render.rs` and `skeleton.rs` with a file-appending marker
and ran the full suite once. Hits, descending:

```
65 plain_multi_HIT          18 render_hash256_ABSTRACT   6 root_sh_wsh
59 label_older_blocks       18 collect_hash256           6 root_sh_bare
25 sole_multi_reached       17 label_after_height        6 collect_hash160_pad
24 tr_keypath_branch_push   16 cfh_sha256                4 render_hash160_ABSTRACT
23 sole_multi_guard_PASSED   8 label_older_units         2 cfh_ripemd160
                             7 label_after_time          2 cfh_hash256
                                                         2 cfh_hash160
                                                         1 root_sh_wpkh
                                                         0 root_unrecognized
                                                         0 root_sh_nonchildren
                                                         0 skeleton_render_ERR
```

Two things worth keeping from this. `sole_multi_reached` 25 vs
`sole_multi_guard_PASSED` 23: the k-of-n guard is exercised **both ways**, 2
refusals — genuinely guarded, unlike its `plain_multi` twin (M-1). And
`render_hash160_ABSTRACT` / `cfh_ripemd160` / `cfh_hash160` are all non-zero,
so the second 20→32 zero-pad site *is* executed — it is only unasserted.

---

## The brief's four cross-task concerns, answered

**1. Seams between tasks — does anything still assume the old branch shape?**
No live code does. Removing the `tr` key-path push (`if false`) reds exactly
3 tests — `deep_taptree_reports_every_leaf_and_the_correct_max_depth`,
`bare_xpub_taproot_keypath_only_reports_the_key_path_as_its_one_branch`,
`taproot_with_one_leaf_reports_the_key_path_then_the_leaf` — so unlike most
key-membership properties in this branch, I-5's change *is* guarded. Blast
radius re-verified independently: `PolicyShape`/`.branches` has no consumer
outside `policy_shape.rs`, `skeleton.rs`, `render.rs` (which imports only
`LockKind`/`lock_from_wire`) and `tests/`. The one thing left assuming the
old shape is **prose**, not code — M-2.

**2. The accumulated public surface — is there a fourth one-rule-two-implementations?**
Yes: **M-1**, and it is the one whose deferral rationale does not hold. The
surface is otherwise coherent and each remaining near-duplicate is documented
with a reason that survives inspection:

- `policy_shape::Lock`/`LockKind` vs `compose::Lock` — justified (the
  composer's `u16` payloads cannot hold every decodable operand).
- `RootKind` vs `compose::Wrapper` — justified (see the rulings below).
- `ABSENT_FINGERPRINT` in two places — ruled, and genuinely guarded on both
  sides, unlike Task 2's lock bands.
- the 20→32 pad in two places — see the triage.
- the sh(wsh) predicate in two places — documented, but see I-1's
  second-order note: only one side has a gate.

The added API itself is minimal and coherent: `SkeletonKey` is a `String`
newtype deriving `Hash` (the lookup handle plan 1b needs) with no public
constructor, `Skeleton` deliberately does not derive `Hash`, and
`descriptor_to_abstract_template` sits beside `descriptor_to_template` as one
more mode of one renderer rather than a second renderer.

**3. Doc comments claiming more than the code proves.** Two found — M-2 and
M-3 — both of the same shape as the two already corrected during the plan:
a fold corrected the prose in one place and left a declaration or a header
stale somewhere else. Everything else I spot-checked resolved: `crate::`
usage, `compose::Wrapper`'s arity, `HashKind::token()`'s four values
(identical to the literal names the old `render_hash256`/`render_hash160`
used, so Task 2's refactor introduced no literal-mode regression),
`digest_len()`, the design's "the key's blind spots" section at :278, and the
`DecodeOpts::partial()` call sites.

**4. Anything a green suite would hide.** Everything above. **Eight of the
seventeen mutations I ran survived a full 581-test suite**; three of those
survivors change the SkeletonKey of a real, strict-decodable descriptor, and
two of those produce an outright collision or a merged spend path.

---

## Deferred-minor triage

| # | item | verdict |
| --- | --- | --- |
| 1 | T1: `plain_multi` has no `nkeys == slots.len()` guard | **follow-up**, owning phase = before plan 1b reads `k`/`n`. NOT safe by construction (M-1, measured) — but `k`/`n` are outside the key and `shape.branches` has no consumer yet. |
| 2 | T2: the 20→32 zero-pad in a second place | **follow-up.** Both sites execute (census: 6 and 4), `copy_from_slice` is length-checked, and the padding is unobservable through `HashLock`'s hand-written `Eq`, so a silent divergence is not constructible. |
| 3 | T2: `validate.rs`'s `1 << 22` at :218/:226/:621 | **follow-up.** Verified genuinely different in purpose: `CONSENSUS_BITS = 0xFFFF \| (1 << 22)` is BIP-68 truncation detection and :226 selects a units word for an error message — not band classification. Pre-existing and outside this branch's file scope. Controller's call stands. |
| 4 | T4/5: `skeleton()` accepts what `validate()` rejects | **follow-up.** Confirmed. It is *consistent* with this crate's established decode-side position — `encode_payload_for_identity`'s own doc ("hashing a card is not minting one") records the regression that re-admitting decoded cards caused. Making `skeleton()` stricter would be the defect. |
| 5 | T4/5: one dead `root_kind` arm | **follow-up** (it is three — N-1). Defensive arms on a `pub fn` with hand-buildable input; keeping them is right. |
| 6 | T4/5: the walker pins only the rendered projection | **follow-up**, correctly handed to plan 1b. Re-verified as still true after I-5: under `is_nums` the push sits inside `if !*is_nums`, so neither the renderer, `policy_shape`, nor `skeleton_key` reads `key_index` — it is a genuine don't-care, and two payloads differing only there are the same policy. |

**None of the six is a merge blocker.** I-1, I-2 and I-3 are, being new.

---

## The controller's rulings — all four upheld

**`crate::` over `md_codec::` — upheld, and not a judgment call.**
`grep -c "extern crate self" crates/md-codec/src/lib.rs` = **0**. The
alternative does not compile. Cost of the ruling: zero.

**Six-valued `RootKind` rather than reusing `compose::Wrapper` — upheld.**
Verified at `compose/mod.rs:73-82`: `Wrapper` is `{Tr, Wsh, ShWsh, Sh}`, four
values, with no `Wpkh`, `Pkh` or `ShWpkh`. It cannot name a decoded singlesig
md1 at all, so reuse would have keyed those policies wrongly or not at all.
One consequence to carry rather than fix: `sh(wsh)` now has two spellings in
the crate (`Wrapper::ShWsh` and `(RootKind::Sh, inner_wsh = true)`). That is
acceptable only because the design forbids the compose-side input model from
crossing into the decode-side key — which is the same reason the ruling is
right.

**Reusing `compose::HashLock` instead of the plan's prescribed duplicate —
upheld, and the strongest of the four.** It closes a documented funds-safety
trap (public fields and derived `Eq` over a padded `[u8; 32]`, plus a second
digest-length site against `digest_len()`'s "the only place a digest length
is written") rather than merely avoiding duplication, and it cost nothing:
`new()`/`kind()`/`digest()` were already public, so no accessor was needed.
The padding really is unobservable — `PartialEq`/`Eq`/`Hash`/`Ord` are
hand-written over `(kind, digest())` with `digest()` sliced to
`digest_len()`.

**Three-valued `KeyPathKind` with `UnspendableXpub` removed as uncomputable —
upheld.** `Body::Tr { is_nums, key_index, tree }` makes `is_nums` the only
internal-key discriminant on the wire, and an unspendable xpub is
structurally an ordinary `key_index`. Recognising one means re-deriving a
specific coordinator's function over the whole descriptor, which is a
coordinator rule, not a codec property. The naming choice (`Xpub`, not the
Go's `Spendable`) is the right call for the same reason — the walk verifies
nothing about spendability and the Go name would assert it. One consequence
to carry into 1b, not a reason to reverse: two taproot policies differing
only by spendable-vs-convention-unspendable internal key share a SkeletonKey,
so 1b's Nunchuk rule must compute the distinction itself. And the design's
own type listing still needs M-3.

**(The fifth named) implementing the `tr` key-path branch now rather than
deferring — upheld, and the best-evidenced of the set.** The cost argument in
the ledger is right (adding a branch changes `fp_partition` and hence the key
for exactly those policies, so deferring it means regenerating plan 1b's
entire evidence table), and unlike most of this branch's key-membership
properties it is actually guarded: 3 tests red when the push is disabled.

---

## Counts

**0 Critical / 3 Important / 5 Minor / 2 Nit.**

The three Importants share one root: **this branch proved its properties in
review transcripts and did not convert the proofs into gates.** The
collision-freedom proof, the "paths in template traversal order" claim, the
`sh(wsh)` unwrap's purpose, and the `older` band split are all *correct* and
all *unpinned*. Each fix is a fixture or a single literal assertion; together
they are perhaps forty lines. Against a plan whose next step is to build a
measured-evidence table keyed by these exact bytes, they are cheap.
