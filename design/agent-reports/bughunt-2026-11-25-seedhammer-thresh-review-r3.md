# seedhammer thresh I-1, review r3: the r2 fold (c4e61a9..55d31dd: c0ab433, 55d31dd)

Reviewer: independent fold review, read-only. Repo /home/user/seedhammer, branch
claude/project-thread-utm1aa. Toolchain: go1.26.7 from the module cache, `GOTOOLCHAIN=local`.
Scratch copy for the experiments: `scratchpad/r3sh` (git archive of 55d31dd, never committed).

**Counts: 0 Critical, 0 Important (new), 1 Important (pre-existing), 3 Minor, 3 Nit.**

Verdict: r2 C-1 and r2 I-1 are fixed. The iterative rewrite computes the same function as the
recursive version, and I found no defect it introduced.

## What I ran

- `go test -count=1 ./md/`: ok. All the new tests pass: `TestWitnessItemLimitVectors`,
  `TestTypeRuleCompletesOnMaxDepthCards`, `TestTypeRuleStackIsBoundedAtMaxDepth`,
  `TestPolicyTypeChecksAgreesWithRustCases`, `TestPolicyShapeAgreesWithRustCases` and
  `TestTypeRuleCases`.
- `CGO_ENABLED=0 GOARCH=386 go test -count=1 ./md/`: ok. The whole md suite also **runs** at 32-bit.
- `GOARCH=arm go vet ./md/`: clean. `GOARCH=arm go build ./md/ ./gui/`: ok. `GOARCH=386 go vet ./md/`: clean.
  `GOARCH=arm go vet ./gui/` fails at `gui/singlesig_restore_test.go:179` (HardenedKeyStart as int),
  which matches the workflow comment.
- `gofmt -l .`: exactly the five-file baseline.
- Vendored file: sha256 325792c5…fc3e equals `git show 1dac0e10:crates/md-codec/tests/fixtures/policy_shape/cases.json`.
  2d5c9935 is an ancestor of 1dac0e10.
- **Old vs new, side by side (a).** I put the c4e61a9 `msNode` and `tapTreeTypeChecks` beside the
  new ones as `msNodeOld` and `tapTreeOld`. Then I compared `(ok, msType)` with `==` (msType is all
  value fields, so this compares every corr and ext field, including height, the sat/dissat options
  and the free-verify flag). Inputs were 330,000 generated trees × {Legacy, Segwitv0, Tap}: untyped
  random trees, type-directed trees with k up to n+1, and trees deliberately malformed at random
  (thresh with a childrenBody or zero children, and_v with 1 child, andor with 2, Check with 2
  children, Check over a key with a wrong body, wsh/TapTree inside a miniscript, pk_k with
  children). I also compared the taptree check at depth offsets 0, 60 and 120.
  **990,000 evaluations, 231,827 typed, 0 mismatches.**
- **Differential against Rust, re-run.** I ran the implementer's generator
  (`zz_scratch_diff_test.go.txt`, seed 20261126) on the new code. Its Go output is **byte-identical**
  to the saved pre-fold `diff.txt.go`. Scored against `diff.rust.jsonl`, the result is
  **77,055 rows, 0 mismatches** on (typed, complete). The Rust side was not re-run, but it did not
  need to be: the Go answers did not change.
- **Does the stack test bite (d)?** With the c4e61a9 recursive `policy_typecheck.go` swapped in,
  `TestTypeRuleStackIsBoundedAtMaxDepth` FAILS with "goroutine stack exceeds 16384-byte limit /
  fatal error: stack overflow". At HEAD it passes.
- Frame sizes on `GOARCH=arm go build -gcflags=-S` (frame-args): msNode $388-88,
  msCombine $484-100, msLeaf $284-88, msThresh $344-88, tapTreeTypeChecks $120-20,
  tapLeafTypeChecks $228-20, treeTypeChecks $228-16. These are constant at any depth. The type
  check's worst chain is about 2.5 KB.

## (a) Equivalence: notes from the read

- Check-idempotence: `msBareCheck` drops Check over a bare key with a keyArgBody before the node is
  typed, and it is applied to the root and to every child. The old code did the same thing by
  calling `msNode(c)` without any further global check. A malformed Check (no sole child) is
  left alone and then refused by `msChildren`, as before. Check over Check gives B under the outer
  Check, which is refused in both versions.
- Thresh: the old code evaluated the children, then refused n = 0 through `msThresholdOK(k,0,0)`.
  The new code refuses zero children in `msChildren`. Any failure fails the whole tree, so the
  order does not matter. `msCombine`'s `n.body.(variableBody)` assertion cannot panic, because
  `msChildren` already checked the body type.
- `msThresh` gets `subs` as an alias of the tail of `vals`. It may sort or mutate them. Those
  entries are then truncated and overwritten, and nothing reads them afterwards. Its order-dependent
  fold sees the children in wire order, because they are pushed last-first and popped first-first.
  The differential confirms this, thresh included.
- Unknown and structural tags (wsh, sh, TapTree, tr) inside a miniscript: `msChildren` returns
  `nil, true`, then `msLeaf` reaches its default and returns false, as before.
- `tapTreeTypeChecks`: the recursive `&&` became an explicit stack (child 0 popped first). The
  result is a pure AND, so it is the same.
- `policyShape`: `s.Complete && !treeTypeChecks` gives Complete = walk && typed, exactly as before.

## (b) Int width in the files this PR touched

`msScriptNumSize(n uint32)`. Its callers pass `uint32(v)` (v < 2^31 already, by `msTimelockOK`) and
`uint32(k)` (k from a uint8), so the switch from int to uint32 changes no reachable value. Every
other int conversion in `policy_typecheck.go` is of a uint8 or a card-bounded length. Arm vet is
clean and the full md suite passes under GOARCH=386. I found no remaining 64-bit-only assumption in
md/. The test files compile and run at 32-bit.

## (c) Memory and termination of the explicit stacks

Termination: every non-expanded pop either pushes one value (a leaf) or pushes one expanded frame
plus the children of a finite, acyclic decoded tree. Every expanded pop shrinks `vals` by
arity-1 ≥ 0 and is never re-expanded. The loop ends after 2×nodes iterations at most.

Memory: see m-1. On the production path (walk-gated) it is small. Ungated, it is large.

## Important (pre-existing, not introduced here)

### I-P1. The decoder and the shape walk still need well over 16 KB of stack on a max-depth card (pre-existing; FOLLOWUP sh-md-decoder-and-walk-recursion-stack)
`md/md.go:392` `readNodeDepth` ($228-28 on arm, ≈256 B per level × 128 ≈ 32 KB).
`md/policy_shape.go:278` `splitBranches` ($264-36, ≈300 B per level × 126 ≈ 38 KB), plus
`collect` ($156-24) and `walkTapTree`. I measured these on the host with `debug.SetMaxStack` in a
child process, on the fold's own max-depth cards. `Reassemble` fails at 16 KB and passes at 32 KB
on GOARCH=386 (it needs 64 KB on amd64). `PolicyShapeChunks` fails at 32 KB and passes at 64 KB
on 386; the overflow trace is 93 frames of `splitBranches` on the 126-deep `or_d` chain.

These are gc frames, not TinyGo/Thumb frames, so this is a proxy. But the margin is 2–4× the
16 KB goroutine budget, and the decoder runs on every md1 load, not only on summaries. So the
fold's "the type check fits 16 KB" is true, but the PolicyShapeChunks path as a whole does not fit
(by proxy). The walk now runs before the type check and gates it, so a hostile deep card reaches
the walk's overflow first. The fix is outside this fold. It is either iterative decode and walk,
or a lower decode depth cap, which needs Rust parity (maxDecodeDepth is a port of tree.rs).
TinyGo `-print-stacks` reports recursive functions as unbounded, so the existing CI job cannot
gate this. An emulator or device run of the max-depth cards is the real measurement.

## Minor

### m-1. msNode's heap use is bounded in production only because the walk now gates it
`md/policy_typecheck.go:397-440` and `md/policy_shape.go:188`. `vals` holds every completed
earlier sibling along the current path. On a nested `thresh(1, 0×31, thresh(1, 0×31, …))` card at
the 64-chunk limit (100 levels), a direct `treeTypeChecks` call allocates **833 KB in total**
(GOARCH=386; msType = 68 B, node = 12 B). Live `vals` alone is about 3,100 × 68 B ≈ 211 KB, plus
the slice-growth copies. Decode allocates 116 KB for the same card. On an RP2350 with about 520 KB
of RAM this would be an OOM.

It is not reachable today. The walk refuses a nested thresh (the thresh child allowlist is one or
two levels deep, `md/policy_shape.go:640ff`), so a walk-complete tree keeps `vals` at most about
depth + 32 entries (≈ 11 KB). The recursive version allocated a 32-entry `subs` per thresh level
too, so this is no regression. But the bound is now a silent coupling between two files. Either
document it at `msNode`, or reject when `len(vals)`/`len(work)` passes a fixed cap (fail-closed,
Complete=false, so Rust parity on the summary holds wherever the walk would accept).

### m-2. The provenance pins a commit that exists only on an unmerged descriptor-mnemonic branch
`md/testdata/policy_shape_cases.provenance.json:9` (1dac0e10). In the local clone, 1dac0e10 is
contained only by `claude/project-thread-utm1aa` (local and origin), not by master. If that branch
is rebased or squash-merged, the pin points at a commit that no longer exists, and the vendor
script's `cat-file -e` guard will refuse a re-vendor. Re-pin to the merged commit when the Rust
side lands (same bytes, so the sha is unchanged).

### m-3. The md suite can run at 32-bit, but CI only builds and vets it there
`.github/workflows/test.yml:143-144`. `GOARCH=arm go vet/build` catches constant overflow (C-1's
class), but not a runtime wrap such as F-142's. `CGO_ENABLED=0 GOARCH=386 go test ./md/` passes
today (0.47 s). Adding `./md/` to `scripts/test-32bit.sh`'s package list would make md's vendored
vectors and its depth and witness tests execute at the device's word size, for nearly no cost.

## Nit

- n-1. `md/policy_typecheck_depth_test.go:187`. The parent passes whenever the child exits 0. If the
  test is renamed and the `-test.run` regex no longer matches, the child runs nothing, prints
  PASS, and the gate passes vacuously. Have the child print a marker after the type-check loop,
  and have the parent require it.
- n-2. `.github/workflows/test.yml:2-5`. The workflow triggers only on push to `main` and on
  `workflow_dispatch`, so the new arm steps do not run on this PR branch before merge unless it is
  dispatched (pre-existing trigger set). The YAML parses, and both steps are in the `tests` job
  (ubuntu-latest, setup-go 1.26) next to the other 32-bit check. `GOARCH=arm` cross-compiles with
  cgo off by default.
- n-3. `md/policy_typecheck_depth_test.go:162-166`. "The iterative check also passes at 8 KB" is
  plausible (constant ≈ 2.5 KB of arm frames), but no test pins it. Since SetMaxStack works in
  powers of two, 8 KB would be a tighter and still meaningful limit than 16 KB.

## Checked and found sound

- C-1: the uint32 signature removes the constant overflow. Arm vet and build are clean, and the
  CI steps would have caught the original (I re-verified that the c4e61a9 file fails `GOARCH=arm go build`
  in r2's run; the new step runs exactly that command).
- I-1: the type check is iterative with a constant frame, it is skipped when the walk refused, and
  the 16 KB child-process test fails against the old recursion.
- Witness-item vectors (r2 m-2): Rust has both cases (typed 84 / untyped over 100), the walk
  accepts both, and the Go type rule separates them from the tree and from the card.
- Re-vendor: 106 cases, the count constants were updated (106, 102 from a card), and the sha and
  commit in the provenance match the bytes.
