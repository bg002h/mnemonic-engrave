# seedhammer thresh r4: adversarial review of 55d31dd..017357b (aa366a2, caa93e1, 017357b)

Verdict: **0 Critical / 0 Important / 1 Minor / 3 Nit.** I found no behaviour change. The range is structure-only as it claims.

Toolchain: go1.25.10. GOTOOLCHAIN=auto switched to it from the local go1.24.7, because go.mod requires 1.25.10; go1.26.7 is not on this box. The repo was left untouched (`git status` clean). All scratch work happened in a `git archive` copy under the scratchpad, which I have since deleted.

## What was checked, and how

1. **The references really are the old code.** I extracted all 23 `*Rec` functions from md/recursive_ref_test.go, removed the `Rec` suffix and diffed each against `git show 55d31dd:md/<file>`. Every function, comments included, is identical apart from whitespace: readNodeDepth, walkShape, walkTapTree, splitBranches, branchOf, soleMulti, collect, walkForPlaceholders, validateTapScriptTree, walkCollectFirst, remapIndices, checkPlaceholderBounds, cloneNode, needsV8, writeNode, emitFragment, guardNode, duplicateInTapTree, countKeySlots, tapBranchDepth, collectTapLeaves, emitTapLeaves and collectKeyOccurrences.
   - caa93e1 does not touch script_emit/tapleaves/template_guard/duplicate_keys/liana/template_strip, so 55d31dd and caa93e1 are the same baseline for those files.
   - No `*Rec` body calls an iterative (non-Rec) name, so the differential test is not vacuous.

2. **Side-by-side reading of each pair.** All are equivalent: pre-order, children left to right (pushReversed), the first error wins, and early exits match.
   - **Decoder:** `d >= maxDecodeDepth` is checked before readTag, as before. Children are read left to right. The tr frame keeps `tree = &children[0]`, which points into the children slice, not into the frame, so it is safe when the stack reallocates. Thresh `count` is ≤ 32. The uint8 depth cannot overflow, since depth < 128 before `d++`.
   - **emitFragment:** the ops each container writes before its children are emitted in emitStep; the rest are queued in order.
     - or_i: IF X ELSE Y ENDIF
     - or_c: X NOTIF Z ENDIF
     - or_d: X IFDUP NOTIF Z ENDIF
     - and_b / or_b: X Y BOOLAND/BOOLOR
     - thresh: X1 (Xi ADD)… then `<k>` EQUAL
     - c:X CHECKSIG, with the idempotent bare-key unwrap kept
     - s: SWAP X; a: TOALTSTACK X FROMALTSTACK; d: DUP IF X ENDIF; j: SIZE 0NOTEQUAL IF X ENDIF; n: X 0NOTEQUAL
     - v: `before` is captured at the v: node's emitStep, which writes no ops of its own. The fold item sits below the child's items on the LIFO, so it runs exactly after X.
     - There is no andor/l:/u: arm in the emitter, old or new; both answer ErrScriptUnsupported.
   - **splitBranches:** has no side effects, so a fail-fast with a different stack shape gives the same answer. The andor xy half is pushed so that it pops first, and orBSecond is passed only to or_b's second child.
   - **collect, walkForPlaceholders and writeNode:** these mutate state, and their pre-order matches, so the partial state and the bits written before an error are the same.
   - **walkTapTree, duplicateInTapTree and collectTapLeaves/emitTapLeaves:** visit leaves left to right, and depth accounting is the same. A malformed TapTree hides only its own subtree in walkTapTree and duplicateInTapTree, and refuses the whole walk in collectTapLeaves/emitTapLeaves, as before.
   - **tapBranchDepth:** the edge cases agree. A root that is not a TapTree gives 0. A TapTree whose body is not childrenBody gives 0. A TapTree with no children gives 1. A malformed child adds no level.
   - **soleMulti, countKeySlots and needsV8:** visit in a different order, but they compute count-only or boolean answers, so order does not matter. soleMulti reads its fields only when found==1.
   - **rebuildTree (cloneNode/remapIndices):** gives a fresh non-nil children slice of the same length, a fresh tr tree node, and the same default arms. One difference is in Nit-1.

3. **The tests have teeth.** I ran mutants in the scratch copy.
   - **Stack test.** I replaced each entry with its recursive reference. TestMaxDepthCardsFitTheStack fails with "goroutine stack exceeds 16384-byte limit" for readNodeDepth, emitFragment, splitBranches, collect, writeNode, guardNode, cloneNode, collectTapLeaves, emitTapLeaves, collectKeyOccurrences, walkForPlaceholders, walkCollectFirst and walkTapTree. So the child process really runs under the cap and the marker logic works. For the other eight, see Minor-1.
   - **Differential tests.** Each of these semantic mutants was caught:
     - or_i children swapped
     - thresh ADD placed before the child
     - or_d ops reordered
     - v: `before` off by one
     - a: tail opcode changed
     - orBSecond dropped
     - andor halves reordered
     - tap-leaf order reversed
     - Liana key-occurrence order reversed
     - decoder depth check changed from `>=` to `>`

     Each one failed TestIterativeCardWalksMatchRecursive, TestIterativeWalkMatchesRecursive or TestIterativeDecodeMatchesRecursive.
   - **Untested mutants.** Two mutants (duplicateInTapTree with malformed→return, tapBranchDepth with malformed→return 0) could not be built: the shared disk filled to 6.8 MB free from outside this review (`df`: /dev/vda 100%). Their test power is unverified, not shown to be missing.

4. **msMaxPending.** treeTypeChecks has one production caller, policyShape (policy_shape.go:188), and it runs only when `walkShape(tree).Complete`.
   - A walk-accepted thresh has only threshConditions children (keys and locks behind s:/a:), so no thresh nests inside another on any path.
   - The pending entries are therefore at most about 3 per level (expanded frame plus andor's 2 pending siblings or their vals) plus one 32-wide thresh: about 416 at the 128 levels the decoder allows. That is well under 1024, and the test checks the worst shapes at 512.
   - Unexpanded siblings cost one entry each, so a thresh hanging off a chain costs 32 only while it is being expanded.
   - The cap answers false, so Complete=false and there is no summary. That is fail-closed, and no other caller treats false as permissive.

5. **32-bit.** `bash scripts/test-32bit.sh` gives `GOARCH=386 test: exit 0` and `GOARCH=arm build: exit 0`, with md included. `GOARCH=arm go vet ./md/` exits 0. There are no new int casts of wire values. emitItem.before is a script length. Heap use is linear in the card's node count, which the card's bit length bounds (thresh/multi ≤ 32 per node).

6. **Remaining recursion in md.** A call-graph scan of the non-test md/*.go files found no mutual recursion. The only direct recursion left is in Nit-2.

`go test -count=1 ./md/` → ok. The targeted run `-v -run 'TestIterative|TestMaxDepthCardsFitTheStack|TestTypeCheck'` → all 7 PASS.

The gui golden typed_witness_items_84 ("20 key(s), custom") matches the vendored Rust shape in md/testdata/policy_shape_cases.json: one branch, slots 0..19, k=0, n=0, complete, typed.

## Findings

**Minor-1. Eight of the replaced walks are not discriminated by the stack test** (md/policy_typecheck_depth_test.go:286).
- Swapping their recursive originals back in still passes under the 16 KB cap on the host, because their gc frames are small at 128 levels: tapBranchDepth, duplicateInTapTree, countKeySlots, validateTapScriptTree, checkPlaceholderBounds, remapIndices, needsV8 and soleMulti (each printed `ok`).
- remapIndices is never driven deep at all. Decoded max-depth cards are canonical, so canonicalize takes the identity fast path, and the non-identity path is unreachable from a decoded card (validatePlaceholderUsage enforces canonical order).
- The shipped code is iterative, so there is no defect on the device. But the test does not hold these to "no stack growth", and TinyGo's Thumb frames are not gc's.
- Optional fix: a lower cap or a deeper synthetic tree for the cheap walks.

**Nit-1. rebuildTree is post-order for a tr's own fields** (md/canonicalize.go:363).
- `leaf(trBody)` runs after the tr's subtree is rebuilt, while the recursive remapIndices remapped keyIndex before descending. Two out-of-range indices would panic with different messages.
- This is unreachable: checkPlaceholderBounds runs first in canonicalize, and the test's perm is 256 long, so neither version panics.

**Nit-2. Recursion that remains** is outside the card-load path and bounded:
- describeTapLeaf (md/tapleaves.go:137) recurses one level only.
- rejectNestedUnspendable and containsSortedMultiA (md/compose_unspendable.go:135,162) are reached only through Composed.ValidateUnspendableShape on composer-built trees, which ComposeMaxPaths = 8 bounds.

**Nit-3.** The run used go1.25.10 (go.mod's floor), not the 1.26.7 the brief mentions; 1.26.7 is not on this box.
