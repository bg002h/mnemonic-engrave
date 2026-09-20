# Fold verification — fold B's r1 fold (the SeedHammer fork), composer fable review r0

**VERDICT: 10 of 11 findings VERIFIED, 1 PARTIALLY VERIFIED (M-4). 8 of 8 attempted
mutations went RED exactly as predicted; the 9th prescribed mutation (removing a
name from `composerDocOwners`) is confirmed a NO-OP by construction, not a defect.
1 finding (N-1) has ZERO protecting assertions anywhere in the 1373-test suite.
1 NEW residual defect found: the doc-theft the fold's own M-4 commit claims to
have closed is only half-closed — `composerSeedDerivedSlots` still carries
`composerSecretCards`' stolen, now doubly-stale doc comment, invisible to the
named-list gate because the thief itself was never added to the list.**

Repo `/scratch/code/shibboleth/seedhammer`, branch `fable-r0-fold`. Verified at
the r1 FOLD tip `aabef11` in a detached worktree
(`/scratch/code/shibboleth/.tmp/verify-fork`, removed on completion), Go 1.26.7
at `/scratch/code/shibboleth/.toolchain/go`, `TMPDIR=/scratch/code/shibboleth/.tmp`,
`CGO_ENABLED=0`. Read-only against the review and fold reports; every mutation
applied, tested, and reverted; tree confirmed clean (`git status --porcelain`)
before and after every mutation and before worktree removal. No sub-agents used.

Sanity test per the brief: `go test ./gui/ -run '^Name$' -count=1 -v` → `PASS`,
`ok seedhammer.com/gui 0.002s [no tests to run]` — confirms the package builds
and the harness executes cleanly (no test is actually named `Name`).

---

## Table: finding → verdict

| finding | verdict | evidence |
| --- | --- | --- |
| I-1 | VERIFIED | `md/compose.go`: the `len(keyless) > 1` cap block sits at :636, after `slots > ComposeMaxSlots` (:591) and `list.Wrapper.IsLegacy()` (:594). Mutation RED below. |
| I-2 | VERIFIED | `md/policy_shape.go:250`: `soleMulti(n); ok && nkeys == br.Keys`. Mutation RED below. |
| M-1 | VERIFIED | `gui/composer_sources.go:166`: `composerCardIsMainnet` returns `card.Network == "" \|\| card.Network == "mainnet"`. Mutation RED below. |
| M-2 | VERIFIED | `gui/composer_shape.go:421`: `composerKeylessPathCount` counts `p.Keys == nil && p.Hash != nil`. Mutation RED below. |
| M-3 | VERIFIED | `md/compose_keyless_cap_test.go`: the refused arm now dispatches `switch c.Error.Kind` with a per-kind comparison, plus a closing loop asserting every named kind in `["TooManyKeylessPaths","KeylessUnderTr","NoKeyedPath","TooManySlots","LegacyWrapperShape"]` was exercised. Mutation RED below. |
| M-4 | **PARTIALLY VERIFIED** | `composer_copy.go` side (`composerCopyMixedLockBases`/`composerCopySameSeedThreshold`) is cleanly fixed: each has its own doc comment, confirmed by `go/ast` probe. `composer_flow.go` side is only half-fixed — see "New defect found" below. Both new names (`composerSecretCards`, `composerCopySameSeedThreshold`) ARE in `composerDocOwners` as claimed, and the covered symbol's own assertion works (mutation RED below) — but the fold's claim "the stolen doc comments go back to their owners" is false for one of the two thefts. |
| M-5 | VERIFIED | `gui/composer_flow.go`: dedup key is `sha256.Sum256(entropy)` into `map[[32]byte]bool`, not `seed.MasterFP`. Mutation RED below. |
| N-1 | code VERIFIED PRESENT; **no protecting assertion** | `gui/multisig_build_slots.go:908-912`: `reg.discardLast(seedID)` added on the `bindPassphrase` error leg, exactly as the diff and commit message state. But removing that one call and running the FULL 1373-test `gui` package (24 shards) leaves **all 1373 green** — nothing in the suite exercises this leg. See "N-1 has no assertion" below. |
| N-2 | VERIFIED (declared, doc-only) | Confirmed no code outside the diff's 16 named files touches `composer_door.go` / the record-door path; matches "doc-only; accepted as stated." |
| N-3 | VERIFIED (declared, declined) | `gui/composer_copy.go:378-382`: `composerCopyRefuseTwoKeylessPaths()` body is unchanged, still names no path numbers — matches "recorded, not changed." |
| N-4 | VERIFIED (doc-only) | `gui/composer_selfcheck.go:86-98` now states the CURRENT contract: "a multi behind a timelock... reports its k-of-n... a multi beside another key reports 0/0" — matches I-2's fix, not the old contract. |

Also verified, per the ONE QUESTION's specific ask:

**Re-vendored vector byte-equality.** `sha256(md/testdata/vectors/compose_refusal_keyless_cap.json)` at `aabef11` = `7b6505f75cc1fc937fd65279fe200129746cf1534c3fe3adf2e8c9d81ce1de8f`. Extracted the same path from `descriptor-mnemonic` at `b2c5d693` (`git show b2c5d693:crates/md-codec/tests/vectors/compose_refusal_keyless_cap.json`) → same sha256, and `diff` reports no difference: **byte-identical**. `md/testdata/compose_refusal_vectors.provenance.json` names `"commit": "b2c5d69384327265da239dc13502727c8f9645fb"` (the full SHA of `b2c5d693`) and the same sha256 in its `files[0].sha256`. Confirmed `b2c5d693` is an ancestor of `descriptor-mnemonic`'s `origin/main`. `TestComposeRefusalVectorsMatchTheirProvenancePin` — PASS.

---

## Mutations run, with RED output

**I-1 — swap the cap check back above the slots/legacy checks** (`md/compose.go`, moved the `if len(keyless) > 1 {...}` block to immediately after `if !anyKeyed {...}`, before the slots/legacy checks):

```
=== gui precedence test ===
composer_fable_r0_funds_test.go:763: 36 slots: folding two key-less paths into one sheds no slot: refused with
    ...at most one key-less path...paths 5 and 6, want ...more key slots than the wire holds (32)
composer_fable_r0_funds_test.go:763: sh: folding leaves two paths, still not one sorted multisig: refused with
    ...at most one key-less path...paths 2 and 3, want ...sh and sh-wsh admit exactly one sortedmulti path
--- FAIL: TestFableValidatePathListOrdersTheCapLast (0.00s)

=== md conformance vector ===
compose_keyless_cap_test.go:283: precedence_too_many_slots: refused with ...paths 5 and 6, want TooManySlots
compose_keyless_cap_test.go:294: precedence_legacy_wrapper_shape: refused with ...paths 2 and 3, want LegacyWrapperShape
--- FAIL: TestComposeKeylessCapAgreesWithTheRustVector (0.00s)
```
Exactly the two vector cases and the two orderings the review's own counterexample table names. Reverted; `git status --porcelain` clean.

**I-2 — widen `soleMulti` back** (dropped `&& nkeys == br.Keys` from `md/policy_shape.go:250`):

```
policy_shape_test.go:144: K/N = 2/3 for a branch holding a multi(2,@0,@1,@2) AND c:pk_k(@3), want 0/0 --
    @3 can spend this branch alone after 10 blocks, and labelling it 2-of-3 tells an operator two of three
    signatures are needed
policy_shape_test.go:164: K/N = 2/3 for and_v(v:multi(2,@0,@1,@2), c:pk_k(@3)), want 0/0 (Keys = 4)
--- FAIL: TestPolicyShapeNeverClaimsAPlainThresholdItCannotSee (0.00s)
```
Both the locked and lock-free mixed-branch fixtures fail, reproducing the exact 2-of-3 misreading the review measured. Reverted; clean.

**M-1 — re-admit a testnet mk1 card** (`composerCardIsMainnet` forced to `return true`):

```
composer_fable_r0_funds_test.go:823: the composer offers a testnet mk1 card as a source (1): label
    "73c5da0a m/48h/1h/0h/2h" xpub tpubDFH9dgzv... -- the consent would derive mainnet bc1q addresses for
    material declared under coin type 1h, the same sentence sysw.ParseKeyRecord now prevents on the other door
composer_fable_r0_funds_test.go:834: a payload with one testnet and one mainnet card yields 2 sources, want 1
--- FAIL: TestFableTestnetCardIsNotOfferedAsASource (0.01s)
```
Matches the fold's own quoted RED verbatim. Reverted; clean.

**M-2 — count an empty path as key-less again** (`composerKeylessPathCount` reverted to `p.Keys == nil` alone):

```
composer_fable_r0_funds_test.go:695: an EMPTY path (no key, no hash) -- M-5's shape: composerKeylessPathCount = 1, want 0
composer_fable_r0_funds_test.go:695: an empty path carrying only a time lock: composerKeylessPathCount = 1, want 0
--- FAIL: TestFableKeylessCreationGuardYieldsToTheStructuralRefusals (0.00s)
```
Reverted; clean.

**M-3 — drop a kind from the vector dispatch** (deleted the `case "TooManySlots":` arm from the `switch c.Error.Kind` in `md/compose_keyless_cap_test.go`):

```
compose_keyless_cap_test.go:286: precedence_too_many_slots: unknown error kind "TooManySlots"
--- FAIL: TestComposeKeylessCapAgreesWithTheRustVector (0.00s)
```
Confirms the per-kind dispatch is load-bearing, not decorative — dropping any kind aborts the test on the first vector case that carries it. Reverted; clean.

**M-4 — remove one of the two new names from `composerDocOwners`** (deleted the `"composerSecretCards": "composer_flow.go",` map entry, code left untouched):

```
--- PASS: TestComposerHelpersDidNotStealADocComment (0.00s)
```
**This mutation does NOT go RED — confirmed a no-op.** The coverage check
(`len(seen) != len(composerDocOwners)`) is self-referential: removing a key from
the map it iterates over just shrinks both sides together. This is not a defect
in the test (the gate's own docstring says it is deliberately a named list, not
a whole-package rule) — but it means this specific prescribed mutation cannot
produce a RED signal by construction, and the ONE QUESTION's list should not be
read as claiming otherwise for this bullet.

The substantive version of the same question — does the assertion the fold
actually added still catch reintroduction of the real defect? — DOES go RED.
Deleting `composerSecretCards`' own 3-line doc comment (leaving the map
untouched):
```
composer_doc_comment_test.go:87: composerSecretCards has NO doc comment in composer_flow.go -- a block
    inserted beneath it with no blank line between takes it, and the record it carried goes with it
    (r0 fidelity I-3)
--- FAIL: TestComposerHelpersDidNotStealADocComment (0.00s)
```
Both reverted; clean.

**M-5 — dedup on the fingerprint again** (`map[[32]byte]bool` keyed on `sha256(entropy)` reverted to `map[uint32]bool` keyed on `seed.MasterFP`; removed the now-unused `crypto/sha256` import to keep the mutation buildable):

```
composer_fable_r0_flow_test.go:523: one seed, bare and with a passphrase, planned 2 ms1 plates
    (byte-identical: true); the plate carries no passphrase, so §7f's ONCE applies
--- FAIL: TestFableSpecOneSeedBareAndWithPassphraseIsCutOnce (0.01s)
```
Matches the fold's own quoted RED verbatim; `TestFableSpecOneSeedTypedTwiceIsCutOnce` (the bare/bare case) stayed green, as expected. Reverted; clean.

**N-1 — delete the `discardLast` call on the failure leg** (removed the four added lines in `gui/multisig_build_slots.go:908-912`):

```
=== enumerating tests in ./gui/ ===
    1373 top-level tests
    partition verified exhaustive: 1373 == 1373
=== running 24 shards in parallel ===
  [all 24 shards: ok]
RESULT: ok -- all 1373 tests ran across 24 shards
```
**Zero failures.** Ran the FULL package (not a targeted test) specifically
because no targeted test exists for this leg — see next section. Reverted;
clean.

---

## N-1 has no protecting assertion

The review filed this as a Nit ("harmless today... but it is the one exit from
this helper that does not un-register"), and the fold's diff adds the call with
no accompanying test. Searched the whole `gui` package for any test that drives
`seedPassphraseStep`'s `bindPassphrase`-error leg (`grep "Couldn't apply that
passphrase"` in `*.go`) — the only occurrence is the `showError` call site
itself, in production code. `bindPassphrase`'s only error paths are an
out-of-range `id` (never true at this call site — `seedID` was just registered)
or `deriveAccountXpub` failing on an arbitrary BIP-39 passphrase, which is not
realistically triggerable from user input. That is almost certainly why no test
reaches it and why the mutation left all 1373 tests green. Not a blocking
finding (N-1 is a Nit and the code change is exactly as claimed), but the fold's
"it now calls `reg.discardLast(seedID)` like every other decline" is a change
with zero regression coverage, worth recording plainly rather than implying it
is tested by proximity to the other, covered declines.

---

## New defect found — M-4 is only half-fixed

`gui/composer_flow.go` still exhibits the exact doc-theft the review's M-4
cited, for the same symbol pair, unresolved:

```
=== composerSeedDerivedSlots (func at composer_flow.go:566) ===
  doc composer_flow.go:552..565 (14 lines)
  first line: "// composerSecretCards is §7f's \"a seed that filled several slots is cut ONCE\"."
=== composerSecretCards (func at composer_flow.go:581) ===
  doc composer_flow.go:578..580 (3 lines)
  first line: "// composerSecretCards is §7f's \"a seed that filled several slots is cut"
```
(`go/ast` probe, `parser.ParseFile` with `ParseComments`, run against the
unmutated tip; worktree confirmed clean before and after.)

`composerSeedDerivedSlots`' doc comment (14 lines, `:552-565`) is STILL the
block originally written for `composerSecretCards` — it opens with
"composerSecretCards is §7f's..." and still contains "THE DEDUP IS BY
REGISTERED SEED, not by slot" (already noted stale in the r1 review's own M-4
text, and now doubly wrong: M-5, landed in the SAME fold, changed the dedup key
from fingerprint to `sha256(entropy)`). `composerSecretCards` itself now DOES
have its own correct 3-line doc — but `git show f222eed -- gui/composer_flow.go`
(the M-4 commit) is **empty**: that file was not touched by the commit whose
message claims "composerSecretCards gets the doc the fold owed it." The doc
actually appeared as an incidental side effect of the LATER M-5 commit
(`aabef11`), which was editing `composerSecretCards` for an unrelated reason
(the dedup-key change) and added a doc comment while it was in there — it did
not perform the "move the helper above the doc block it sits under" restructure
the review's M-4 "Expected" section prescribed, so the original misattached
block was never removed.

Net effect: two different comment blocks in the file now both open
"composerSecretCards is §7f's...\" — one correctly attached to
`composerSecretCards` (:578-580), one still incorrectly attached to
`composerSeedDerivedSlots` (:552-565). The named-list gate
(`composerDocOwners`) cannot see this because `composerSeedDerivedSlots` was
never added to the list — only the victim `composerSecretCards` was, which
satisfies the review's literal instruction ("add the two symbols") but not the
underlying defect the review's M-4 finding identified (the same instance is
named in the gate's own docstring at `gui/composer_doc_comment_test.go:37`:
"composerSeedDerivedSlots took composerSecretCards' block").

By contrast, the `composer_copy.go` side of the same M-4 finding
(`composerCopyMixedLockBases` / `composerCopySameSeedThreshold`) IS cleanly
fixed: each function has its own correctly-opening doc comment, confirmed by
the same probe technique, with a real blank-line/section-banner separation
between them.

This is Minor-shaped (a misattached, stale doc comment — no wrong result, no
data loss, no unmet guarantee; consistent with M-4's own original severity),
not a gate-blocking defect, but the fold's commit message and its account in
`composer-fable-r0-fold-fork-fold-r1.md` overstate what was done ("the stolen
doc comments go back to their owners" is true for one pair, not the other).
Worth a follow-up to either move `composerSeedDerivedSlots` above the stale
block (letting it re-attach to `composerSecretCards`, which already has its own
now) and trim the stale block, or add `composerSeedDerivedSlots` to
`composerDocOwners` with its own doc so the gate would have caught this.

---

## Worktree

`/scratch/code/shibboleth/.tmp/verify-fork` created detached at `aabef11`,
removed after this verification completed; every mutation applied above was
reverted and confirmed via `git status --porcelain` before the next step and
before removal.
