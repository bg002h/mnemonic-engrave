# WHOLE-DIFF ADVERSARIAL REVIEW — F-533, round 1

**Question asked:** did the implementation introduce a defect the gates do not catch?
**Scope:** the four commits `main`(`e4ab97d`)`..e84c610` on branch `f533-bip388-reuse`
in `/scratch/code/shibboleth/.tmp/seedhammer-f533`. The plan is settled; this is
about the code.

**Verdict: GREEN (0C / 0I) — clear to merge.** Three Minor/Nit items are recorded
below; none blocks.

Everything measured here was run in the worktree with
`/scratch/code/shibboleth/.toolchain/go/bin` on PATH (`go1.26.7`). Four throwaway
mutations and two scratch test files were applied and **fully reverted**; the tree
is back at `e84c6101afbc6b4e38b27b4c90a0b7d0854f6563`, `git status --porcelain`
empty, `go build ./...` clean, `gofmt -l` on all 13 changed `.go` files empty.

---

## 1. Findings

### M1 — `countKeySlots` does not walk `trBody`, and the new call site inherits that

**Not introduced here** — `countKeySlots` has always skipped `trBody`, and
`duplicateInExpression` has always carried the gap. What F-533 adds is a *second*
caller (`countKeySlots(*b.tree, &counts)` in the `tagTr` arm), which inherits the
same blind spot for the taptree.

**Constructed counterexample** (run in `package md`, scratch file since deleted):

```go
nestedTr := node{tag: tagAndV, body: childrenBody{children: []node{
    node{tag: tagPkK, body: keyArgBody{index: 1}},
    node{tag: tagTr, body: trBody{isNums: false, keyIndex: 0, tree: p(k(2))}},
}}}
DuplicateKeySlot(node{tag: tagTr, body: trBody{isNums: false, keyIndex: 0, tree: p(nestedTr)}})
// measured: DuplicateNone @0  -- @0 is the outer internal key AND the inner tr's
// internal key, and nothing reports it.
```

**Why the justification is thinner than it reads.** `countKeySlots`' doc says
"a nested tr cannot occur inside a miniscript expression". That is enforced only
at a tapleaf's **root** tag — `validateTapScriptTree` calls `isForbiddenLeafTag`
on the leaf node itself and does not recurse into the leaf's own subtree
(`md/md.go:1005-1024`). A `tagTr` nested one level down inside `and_v` is not
refused there.

**Why it is Minor and not Important.** Such a card can never produce an address
in either direction, so there is no wrong-address or wrong-warning-beside-an-
address outcome: `emitFragment` (`md/script_emit.go:157`) has no `tagTr` case and
its `default:` arm returns `ErrScriptUnsupported` (line 560), so
`EmitTapLeavesChunks` fails, `complexAddressDeriver` returns `!ok`, and the
policy is display-only regardless. No shipped encoder produces the shape either
(`md.Compose` only ever builds `tagTr` at the root, `md/compose.go:811`), and
miniscript has no `tr` fragment, so the Rust primary cannot emit one.

**Suggested follow-up (owning phase: whenever `countKeySlots` is next touched):**
either add a `trBody` arm to `countKeySlots` (count `keyIndex` when `!isNums`,
recurse into `tree`) — three lines, and it would make the walk match
`walkForPlaceholders`/`walkCollectFirst`, which both already handle `trBody` at
any depth — or amend the doc comment to say the guarantee is enforced at the leaf
ROOT only.

### M2 — an engrave-side doc the implementer's §8 list does not name is now false

`design/CONTINUITY_2026-09-13_address_routes.md:108` still says:

```
remaining `ok/refused 2`. They survive because the refusal rides
```

The implementer's report §8 enumerates four false passages in
`design/FOLLOWUPS.md` and the corpus-baseline rewrite, but not this one. Both are
in `mnemonic-engrave`, which was off-limits to the implementer, so this is
controller-owned and outside the fork merge. Recorded so it is not missed
alongside the FOLLOWUPS fold.

### N1 — `stillUnsupported` has no counterpart check to `refusedByPolicy`'s

In `gui/policy_address_test.go`, the `refusedByPolicy` branch proves the
classification (`complexAddressDeriver` must exist beneath the gate, then full
cross-language conformance). The `stillUnsupported` branch asserts only
`r == routeNone` — it does **not** assert the absence of a deriver. So a future
vector misfiled there would be silent, whereas one misfiled in `refusedByPolicy`
fails loudly. The map is empty today and the two F-533 vectors are in the right
one, so nothing is wrong now; it is an asymmetry in a gate this diff relies on.

### N2 — the pin does not cover every reader of these two vector names

`md/policy_shape_test.go:49` uses `keyed_tr_sortedmulti_a` through
`vectorPath(name, "phrase.txt")` (`md/policy_shape_test.go:11`), which reads the
**vendored** file directly and bypasses both new pin-preferring loaders. F-529's
re-vendor would still break that test. It is not an F-533 witness, so this is not
a gap in F-533's evidence — only a note that "the pin survives the re-vendor" is
true of the two loaders, not of every reader of those names.

### N3 — `md/testdata/README.md` documents every vector but not `testdata/forkbuilt/`

Neither F-531's three `dup_seat_*` pins nor F-533's two new ones appear in the
testdata README, which otherwise has a section per vector. A reader who finds
`keyed_tr_multi_a.md1.txt` there has only the test file's doc comment to explain
why a second copy exists.

---

## 2. Verified SOUND — a next round need not re-check these

**The predicate (`md/duplicate_keys.go`), nine constructed edge cases, all
correct** (scratch test, since deleted). In each row the tree was hand-built and
the returned `(slot, kind)` checked:

| case | result |
| --- | --- |
| nested taptree depth 3, reuse only in the deepest leaf | `DuplicateTaprootInternalKey @0` ✓ |
| internal key reused via `multiKeysBody` deep in a nested tree | `DuplicateTaprootInternalKey @0` ✓ |
| internal key reused inside a `thresh` (`variableBody`) | `DuplicateTaprootInternalKey @0` ✓ |
| taptree that is a BARE leaf, no `tagTapTree` branch node | `DuplicateTaprootInternalKey @0` ✓ |
| both harms on the SAME slot @0 (repeat inside one leaf **and** at the internal key) | `DuplicateFewerKeys @0` ✓ (precedence holds even when the slots coincide) |
| malformed `tagTapTree` with 1 child carrying the reuse | `DuplicateTaprootInternalKey @0` ✓ (`duplicateInTapTree` bails, the new count does not) |
| `tagTr` whose body is not `trBody` | `DuplicateNone` ✓ |
| NUMS with a non-zero `keyIndex` (7) that DOES occur in the tree | `DuplicateNone` ✓ |
| `keyIndex = 255` boundary | `DuplicateTaprootInternalKey @255` ✓ |

`b.tree == nil` (key-path-only `tr`, which is exactly what `ScriptTr` singlesig
encodes — `md/encode_singlesig.go:99`) returns early. `countKeySlots` covers every
key-bearing body except `trBody` (M1): `keyArgBody`, `multiKeysBody`,
`childrenBody`, `variableBody` are all walked.

**The `!isNums` guard is the right guard, not an approximation.** It is the exact
condition under which `walkCollectFirst` (`md/canonicalize.go:108-115`) and
`walkForPlaceholders` (`md/md.go:955-968`) register the internal key as a
placeholder. `walkForPlaceholders` also *guarantees* `b.keyIndex < n` when
`!isNums` (`errNUMSConflict`), so `counts[b.keyIndex]` can never index a slot the
payload does not have.

**The "one use-site per @N" argument the whole design rests on is true in the
code.** Overrides are `[]idxUseSite{idx, path}` keyed on `idx`
(`md/md.go:501-503`, `useSiteStringFor` at `md/md.go:1429`), so one slot cannot
carry two disjoint key expressions. Core-duplicate really is a strict subset here.

**The refusal fires on the device path, and the gui test proves what it claims.**
Mutation `kind != md.DuplicateNone && false` in `complexAddressSource`:
`TestF533RefusalHappensAtTheBranchTheDeviceTakes` goes RED at steps 3 and 4 and
prints `bc1pf4aujydl48hah9qxvk4j0dcce737pl9svne7rmzcprrh7y92znsstul4rt` and
`bc1p588jmtx4ptv76t9sclt6gt33eyydvsrea4njyayerqj2frw5m5aq5gzycw`, while
`TestEveryKeyedVectorReachesAnAddress`, `TestConsentWarnsOnDuplicateKeys` and
`TestF533SurfacesSayWhyThereIsNoAddress` also redden. Restored, tree clean.
Step 2's "the emitter below the gate does derive them" is weak on its own (it
never calls the closure), but the same claim is proved byte-equal against Rust in
`TestEveryKeyedVectorReachesAnAddress`'s `refusedByPolicy` branch, so it is not a
false PASS.

`gatheredDescriptorFlow` reaches `duplicateRefusalBody` **before** any routing
(`gui/md1_gather.go:185`), and that function is kind-agnostic
(`kind == md.DuplicateNone` is its only test), so `TestRepeatedSeatRefusalStillWarns`
— which drives the real flow on rendered frames for F-531's kind — covers the same
screen path for F-533's.

**No third route exists.** `bip380.Descriptor` is flat `{Script, Threshold, Type,
Keys}` (`bip380/bip380.go:20`) with `P2TR` only in the singlesig set (line 117),
so a scanned taproot-with-taptree descriptor cannot be represented at all — F-530's
`descriptorRepeatsAKey` route is not reachable for these shapes and needed no
widening.

**The NUMS guard mutation reddens exactly the named reddening set.**
`if !b.isNums` → `if true`: `md` fails on
`TestDuplicateKeySlotIsQuietOnPoliciesWithoutReuse/keyed_compose_tr_nums_three_leaves`,
`TestDuplicateKeySlotScopesPerTapLeaf/one_key_in_two_leaves_is_not_a_duplicate`
and `TestTaprootInternalKeyReuseIsItsOwnKind/NUMS_internal_key_beside_a_leaf_using_@0`;
`gui` fails on **exactly the eight vectors** the implementer enumerated by name
(`keyed_compose_preset_kofn_recovery`, `keyed_compose_tr_hash_leaf`,
`keyed_compose_tr_nums_three_leaves`, `keyed_compose_tr_sole_sortedmulti_a`,
`keyed_compose_tr_two_path_distinct_fingerprints`, `keyed_compose_tr_two_path_nums`,
`keyed_compose_tr_unsorted_sole_leaf`, `keyed_tr_pathological`) — no more, no fewer.

**Copy: fits, and does not fall through.** The new branch sits between the
`DuplicateFewerKeys` `if` and the unconditional Core return, so
`DuplicateTaprootInternalKey` cannot reach "Bitcoin Core refuses such a
descriptor". (`DuplicateNone` still can, but no production call site passes it —
`noAddressLines` and `duplicateRefusalBody` both guard on `!= DuplicateNone`.)
Independently re-measured the greedy wrap at width 20:

| body | chars | wrap-20 lines |
| --- | --- | --- |
| new BIP 388 sentence, `@0` | **94** | **6** |
| same, `@3` | 94 | 6 |
| same, two-digit slot `@15` | 95 | **6** |
| shipped Core sentence (comment claims 96 / 6) | 96 | 6 ✓ |
| shipped fewer-keys sentence | 115 | 7 |

Six lines against a seven-line page, and the two-digit slot does not push it to
seven. On the wire the slot is always `@0` anyway: `validatePlaceholderUsage`'s
ascending-first-occurrence rule forces `keyIndex == 0` for a root `tr` with
`!isNums`, since pre-order registers the internal key first.

**A stub does not pass.** Replacing the new sentence with
`"REVMUTC slot @%d BIP 388."` — which still satisfies both of
`TestF533SurfacesSayWhyThereIsNoAddress`'s content checks — reddens
`TestComposerCopyIsVerbatimFromTheSpec` (the §8s verbatim table) and
`TestModalsThisBlockTouchesAreDrawnInFull`. Restored.

**The pins decode to the policies their names claim** (scratch dump, since
deleted): `keyed_tr_multi_a` → `tr(isNums=false, keyIndex=0, tree=multi_a k=2
indices=[0 1])` (tag 8 = `tagMultiA`); `keyed_tr_sortedmulti_a` → same with tag 9
= `tagSortedMultiA`. Both are byte-identical to the vendored `.phrase.txt` modulo
the dropped `chunk-set-id:` header. The drift gate reddens on a one-character
mutation, naming the chunk and printing both strings, and `Reassemble` fails too.
The `forkbuilt/` directory contains only these two names that collide with a
vendored vector, so the two loaders' new pin preference changes nothing else.

**`TestTaprootScriptPathMatchesRust` did not go vacuous.** It calls
`address.TaprootScriptPath` directly, not the gated route, so the cross-language
address check still runs for both vectors — the "a new gate makes old tests
vacuous" trap was avoided in two places (there, and via `refusedByPolicy`).

**Consumers of the kind enum are all accounted for.** Every production reader
(`gui/policy_address.go:85`, `gui/wallet_policy.go:306`, `gui/md1_gather.go:222`)
tests `!= md.DuplicateNone` rather than listing kinds, so the new kind is handled
the day it exists; the only kind-listing site is `composerCopyDuplicateKeys`,
which got its branch. No exhaustive `switch` anywhere silently drops it.

**Comments/premises checked for falsification:** `gui/md1_expand.go:160`
(`PolicySortedMulti` ⇒ `DuplicateFewerKeys`, never `DuplicateRefusedByCore`) is
untouched by a taproot rule and still true; `md/testdata/README.md`'s
`keyed_tr_multi_a` section describes vector provenance and the
`multi_a`/`sortedmulti_a` address discrimination, both still true; the Core 31.1
table at the head of `md/duplicate_keys.go` is unchanged and still correct. The
`25.0.0` vs `31.1` version mix elsewhere in that file is pre-existing and not
touched by this diff.

---

## 3. Verdict

**GREEN (0 Critical / 0 Important).** 1 Minor (M1, a pre-existing `countKeySlots`
blind spot the new call site inherits, unreachable for any deriving policy),
1 Minor (M2, one more engrave-side doc to fold, controller-owned), 3 Nits.
Clear to merge.
