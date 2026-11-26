# seedhammer thresh I-1, review r2: the type-rule port (a7b805f3..c4e61a9)

Reviewer: independent adversarial review, read-only. Repo /home/user/seedhammer, branch
claude/project-thread-utm1aa, commits 8181f17, e1857e9, c4e61a9.
Rust reference: descriptor-mnemonic 2d5c9935 (crates/md-codec/src/to_miniscript.rs
tree_type_checks / node_to_descriptor / node_to_miniscript, policy_shape.rs) and its vendored
rust-miniscript 13.0.0 at ff4732e (types/correctness.rs, types/extra_props.rs, types/mod.rs,
miniscript/mod.rs from_ast, context.rs, primitives/{threshold,absolute_locktime,relative_locktime}.rs,
descriptor/{segwitv0,sh,tr}.rs).

**Counts: 1 Critical, 1 Important, 3 Minor, 2 Nit.**

## What I ran

- Toolchain: go1.26.7 from the module cache
  (`/root/go/pkg/mod/golang.org/toolchain@v0.0.1-go1.26.7.linux-amd64`, `GOTOOLCHAIN=local`;
  `/usr/local/go` is 1.24.7 and cannot be used).
- `go test -count=1 ./md/`: ok. `TestPolicyTypeChecksAgreesWithRustCases`,
  `TestPolicyShapeAgreesWithRustCases`, `TestTypeRuleCases`, `TestThreshAllowlistIsExactlyTheSpellings` pass.
- `go test -count=1 -run 'Template|Composer|Summary|Consent|Unspendable|Shape|Multisig' ./gui/`: ok (16.8 s).
- `gofmt -l .`: exactly the five-file baseline. `go vet ./md/` (host): clean.
- Vendored file: sha256 e1e2aab3...49c4 equals `git show 2d5c9935:crates/md-codec/tests/fixtures/policy_shape/cases.json`.
  ca48a05e, f8b4a675 and 5ad5a9b2 are ancestors of 2d5c9935.
- The implementer's differential: I re-scored their saved outputs (`diff.txt.go` against
  `diff.rust.jsonl`, 77,055 rows). 0 mismatches on (typed, complete). 9,779 rows are typed and
  8,185 complete. I did not re-run the Rust side.
- **32-bit build:** `GOARCH=arm GOARM=7 go build ./md/` fails at HEAD and passes at a7b805f3 (C-1).
- Frame sizes from `GOARCH=arm go build -gcflags=-S`, with C-1 patched in a scratch copy (I-1).

## Rule-by-rule check against rust-miniscript (no divergence found)

I read every Go type rule next to correctness.rs and extra_props.rs. Each of these matches Rust,
including argument order, Option (None) propagation and the +1 exec-stack terms:

- Base and input tables for pk_k and pk_h (K), multi and sortedmulti (B, AnyNonZero),
  multi_a and sortedmulti_a (B, Any), hashes, time, TRUE and FALSE.
- The wrappers a:, s: (One/OneNonZero only), c: (K only), d: (V and Zero), v:, j:
  (OneNonZero/AnyNonZero, B), n:.
- and_b and and_v (the shared input table, in the same arm order), or_b (both sides d), or_c and
  or_d (left d and u), or_i, andor (B with BB/KK/VV), and thresh (B then W children, all u and d,
  the input taken from the argument count).
- ExtData: the pk_cost and static_ops constants; has_free_verify (c:, s:, multi, multi_a, hashes,
  thresh, and and_v through its right side); the sat_concat closures (with +1 for and_b and or_b
  only); or_i's with_0 and with_1; d:'s sat edit; v: dropping dissat; j:'s and d:'s fixed dissat;
  thresh's stable sort by Option<isize> key, kept across fields, with the `i <= k` fold and the
  execStack `acc > 0` hack.
- Context and placement: from_ast checks every node (height, then check_global_validity: Legacy
  520, Segwitv0 10000 and 3600, Tap MAX_BLOCK). Multi is refused in Tap, and multi_a and
  sortedmulti_a in Legacy and Segwitv0. The sortedmulti and sortedmulti_a root handling matches,
  as do the md conversion rules (Check over a bare key; nested sortedmulti, raw_pkh, TapTree or a
  wrapper refused).
- Local checks: ops <= 201 and the sat requirement for Legacy and Segwitv0; Legacy scriptSig
  <= 1650; Segwitv0 wsCount+1 <= 100; Tap sat.ok and wsCount+execStack <= 1000. Threshold::new
  bounds are 1..=n with n <= 20/999; AbsLockTime and RelLockTime come to 1 <= v < 2^31;
  TapTree::combine depth is <= 128.
- `check_terminal_non_malleable` (Legacy or_i and d:) is never called in rust-miniscript 13
  (grep), and sanity_check (timelock mixing) is not called by tree_type_checks. The Go port
  correctly omits both.
- K never arises in practice: md's bare PkK and PkH always become c:pk_k and c:pk_h (B) in both
  implementations. Rust behaves the same way, so this causes no divergence.
- Every GUI consumer (`template_engrave.go:152`, `composer_selfcheck.go:72`,
  `composer_consent.go:171`, `composer_unspendable.go:82`) reads Branches and KeyPath only after
  `shape.Complete`. The only producer is `md.PolicyShapeChunks` → `policyShape` → walk &&
  typed. I found no device surface that renders a spend summary without the check.
  `TapLeavesChunks`/`EmitTapLeavesChunks` feed address derivation, not a summary.

## Critical

### C-1. The firmware no longer compiles: an untyped constant overflows a 32-bit `int`
`md/policy_typecheck.go:298` (`msScriptNumSize`): `case n < 0x80000000:` with `n int`.
On any 32-bit target `int` is 32 bits, so the constant 2147483648 overflows `int`. That
includes TinyGo for `pico-plus2` (RP2350, Cortex-M33), which is the shipping target.
Reproduced on the host:

    $ GOARCH=arm GOARM=7 go build ./md/
    md/policy_typecheck.go:298:11: 0x80000000 (untyped int constant 2147483648) overflows int

The same command passes at a7b805f3. TinyGo type-checks with go/types, so
`tinygo build ... ./cmd/controller` (the CI job in `.github/workflows/test.yml:159/162` and
the flake recipe) fails for both the default and the refugium builds. The implementer's
verification ran only on amd64, where `int` is 64 bits. The branch's only Actions run is "Build
image", so CI has not caught this yet.

Failure scenario: merging produces a master from which no firmware can be built. This does not
produce a wrong summary, but it blocks the only artifact this change exists for.

Fix: make the comparison width-independent, for example `func msScriptNumSize(n uint32) int`
(callers pass `uint32(v)` and `uint32(k)`) or `int64(n) < 0x80000000`. I patched the second form
in a scratch copy: `GOARCH=arm GOARM=7 go build ./md/ ./gui/` then succeeds. Add a 32-bit
compile step to the md gate, for example `GOARCH=arm go vet ./md/ ./gui/`, so the host suite
can catch this class of error.

Values are otherwise 32-bit safe. msTimelockOK runs before `int(v)`, so `v < 2^31`. All sums are
linear in a card-bounded tree, so no int32 overflow is reachable.

## Important

### I-1. New untrusted-input recursion with about 3x the decoder's frame, unmeasured on the device stack
`md/policy_typecheck.go:298ff`: `msNode` recurses once per tree level. `policyShape`
(`md/policy_shape.go:183-189`) runs it on every decoded card, after the walk, including cards
the walk has already refused.

The decoder caps depth at 128 (`md/md.go:384`), so the 402 height check never fires. A
hostile but valid card can still nest about 126 levels, for example `wsh(n:n:...n:pk(@0))`, or
an and_v chain, which costs about 6 bits per level and fits in a multi-chunk md1.

Measured with gc on GOARCH=arm (with C-1 patched), as a proxy for TinyGo:

| function | frame (locals + args) | at depth 127 |
|---|---|---|
| `msNode` | 0x2dc + 0x58 ≈ 820 B | ≈ 104 KB |
| `readNodeDepth` (decoder, existing) | 0xe4 + 0x1c ≈ 256 B | ≈ 32 KB |

The rest of `msNode`'s chain adds to this. TinyGo/LLVM frames are typically smaller
(alloca sharing), but `msNode` carries three `msType` values (about 68 B each on 32 bits) in
`s [3]msType`, plus `t`, `l`, `r`, `sub` and the sret temporaries for `msBinary`, `msAndOr` and
`msWrap`. Its frame will stay several times the decoder's.

The firmware is built with `-stack-size 16kb -scheduler tasks`. Under the tasks scheduler,
`main` runs as a goroutine, and a goroutine with recursion gets the 16 KB default. TinyGo checks
the stack canary only at task switches, so an overflow inside this recursion writes past the
goroutine stack into the heap before anything notices.

I could not run TinyGo here, so this is a measured risk, not a demonstrated crash.

Failure scenario: an operator loads a coordinator-supplied deep md1. The decoder survives, but
the type check overflows the goroutine stack. The result is a hard fault, or silent heap
corruption on a signing device, during a display-only computation.

Before merge:
1. Run the existing CI `-print-stacks` build, or an emulator run of a max-depth card through
   `PolicyShapeChunks`.
2. Either show that the deepest decodable tree fits, or bound it. Two cheap mitigations, either
   of which keeps Rust parity on every output field:
   - Skip `treeTypeChecks` when `walkShape` already returned `Complete=false`. The type check
     can only lower Complete, and the walk refuses most deep chains (wrappers it does not
     allowlist).
   - Convert `msNode` to an explicit post-order stack.

Pre-existing note: `readNodeDepth`, `splitBranches`, `branchOf` and `collect` share the same
exposure at smaller frames, so the measurement should cover the whole `PolicyShapeChunks` path.

## Minor

### m-1. Typed=true and the summary overstates the signers when a placeholder repeats (pre-existing, Rust parity)
`md/policy_shape.go` (walk) together with `gui/template_engrave.go:185-190`.
`wsh(or_d(multi(2,@0,@0,@1),...))`, or a bare `wsh(multi(2,@0,@0,@1))` in a complex-root
context, decodes. `validatePlaceholderUsage` allows a repeat. It types, and the walk reports
`{K:2 N:3 Keys:2}`, rendered "2-of-3". One CHECKMULTISIG accepts the same signature against
both @0 positions, so @0 alone spends.

I reproduced `typed=true, Complete=true` in a scratch test. Rust documents "Duplicate keys ...
are NOT refused" (policy_shape.rs:470 doc), so this is parity and outside this diff. The
template consent screen (`multisig_build.go:808` → `templateConsentLines`) shows the summary
without the `DuplicateKeySlotChunks` warning that `wallet_policy.go:313` uses. Suggest a paired
FOLLOWUP on both sides: no summary, or a loud duplicate line, when a slot repeats within one
script.

### m-2. The 100-witness-item rule has no vector on either side
`md/policy_typecheck.go:236`. Neither cases.json (its untyped_* set covers 3600 bytes and 201
ops) nor `typeRuleCases` exercises `wsCount+1 <= 100`. Legacy 1650 and Tap 1000 have no vectors
either, but those look unreachable within 520 bytes or card size.

This one is reachable and isolates the rule. `wsh(and_v(v:multi(20,K20),...,multi(20,K20)))`
with 5 multis has pk 3425 (< 3600), 105 ops (< 201) and wsCount 105, so it is untyped only by
this rule. With 4 multis it is typed (2740 B, 84 ops, 84 items); I computed both in a scratch
test. The Go formula matches Rust by reading, but no test pins it. A bug here would fail open
(a summary shown) for nothing worse than a non-standard script. Add the pair to cases.json
upstream first (Rust-primary), then re-vendor.

### m-3. The type check runs on every card even when the walk has refused it
`md/policy_shape.go:184-187`. When `walkShape` returns `Complete=false`, the result cannot
change. The check still runs the full recursion and allocation (`msThresh` makes `subs` and
`pairs` plus five escaping `less` closures per thresh). The cost is small, but this is the
cheapest I-1 mitigation and costs nothing in parity. The vendored test
`TestPolicyTypeChecksAgreesWithRustCases` calls `treeTypeChecks` directly, so it still covers
the rule on walk-incomplete trees.

## Nit

- n-1. `md/policy_typecheck.go:40` and the `msMaxRecursionDepth` check describe a 402 limit that
  `maxDecodeDepth = 128` makes unreachable for any decoded card. Say so, so that no one
  reads the 402 as the device's stack budget.
- n-2. After this change, the composer self-check's refusal text ("the decoded policy cannot be
  described", `gui/composer_selfcheck.go:73`) also fires for a composed wallet that exceeds a
  standardness limit (3600 B, 201 ops, 100 items). That is correct fail-closed behaviour. The
  error string just no longer names the most likely cause for a large composer wallet.

## Vendor script and provenance
`scripts/vendor-policy-shape-cases.sh` now reads `git show <commit>:<path>` under `set -euo
pipefail`, with a `cat-file -e` guard. The pin records commit, read_from, path and sha256, and
the test checks sha and commit and refuses unknown fields and a missing `typed`. The vendored
bytes equal 2d5c9935's. No issues.
