# PLAN — F-533: refuse taproot key reuse on a BIP-388 predicate

**Status: DRAFT, awaiting R0.** No code until this is 0C/0I. Risk set: this
changes which wallets the device will serve (admission), and it is address- and
funds-adjacent.

**Baseline.** fork `e4ab97d`, engrave `f295b439`, dm `d8bb6d2d`. Re-measure
before implementing; a GREEN earned against a moved tree is a claim about a tree
nobody is building on.

## The defect, re-measured today

`scripts/policy-generate.py --corpus` at fork `e4ab97d`:

```
corpus: 70 vectors
  ok/agrees       45
  ok/refused       2
  source          22
  expand           1
every vector matches the baseline
```

The two are `keyed_tr_multi_a` and `keyed_tr_sortedmulti_a` — `tr(K, multi_a(…K…))`,
one key at the taproot internal key AND inside a leaf. The device derives an
address; the Rust primary refuses. Unchanged across the 30 commits since the
original measurement, which is expected: the divergence is about taproot key
reuse, not about anything this cycle touched.

## Why the current predicate cannot see it — located, not inferred

`md.DuplicateKeySlot` (`md/duplicate_keys.go:92`) dispatches on `tagTr` to
`duplicateInTapTree(*b.tree)`. The taproot internal key is **`trBody.keyIndex`**
(`md/md.go:114-118`), a SIBLING of `tree`, and no path from `duplicateInTapTree`
reaches it. So the internal key participates in no count, and
`tr(K, multi_a(…K…))` repeats nothing under this predicate.

That is deliberate. The function answers **Core's** question, and its own comment
records the measurement behind it: Core 25.0.0 rejects miniscript under `tr`
outright, so a tapleaf `multi_a` never reaches `CheckDuplicateKey` and all three
of `tr(A,multi_a(2,B,B))`, `tr(A,sortedmulti_a(2,B,B))`, `tr(A,multi_a(2,A,A,B))`
are ACCEPTED. The doc also records that an earlier version claimed
`DuplicateRefusedByCore` here and was wrong.

**So the device has one predicate serving two rules.** A warning should say what
Core will do; a refusal should say what BIP 388 permits.

## THE TWO RULES ARE NOT NESTED — the load-bearing question for this plan

It is tempting to assume BIP 388 ⊇ Core's rule, in which case moving the refusal
onto BIP 388 would only widen refusals and change nothing else. **That is wrong
in one direction and it matters:**

* Core's predicate counts key SLOTS inside one expression and ignores use-site
  multipath. BIP 388 explicitly PERMITS one key at several use sites when their
  multipath sets are DISJOINT (`@0/<0;1>` and `@0/<2;3>`).
* So a policy can be Core-duplicate and BIP-388-legal. On that policy the
  refusal lifts and the address branch becomes reachable **while still carrying a
  duplicate** — which is exactly the state `gui/wallet_policy.go:340-350` says it
  is not prepared for: *"IF A DUPLICATE EVER DERIVES AGAIN … this block has to
  come back, and it will have no coverage."*

**R0 MUST ANSWER THIS FIRST:** can such a policy be expressed in the md1 wire at
all? Multipath in the fork lives in the **expand/use-site layer**
(`md/expand.go:33-42`, `UseSite.HasMultipath` + `[]UseSiteAlt`), not per
occurrence in the tree, and `DuplicateKeySlot` walks the TREE. If one slot cannot
carry two different use-sites in this wire format, the reachable-warning case is
empty and step 3 below is unnecessary — and saying so with evidence is worth more
than writing the code defensively.

## The change, in order

1. **New predicate, `md` package.** `BIP388ReusedSlot(tree node) (uint8, bool)`:
   counts key-slot occurrences over the WHOLE expression, including
   `trBody.keyIndex`, and returns the lowest slot seen twice. It does NOT reuse
   `duplicateInExpression`'s per-expression scoping — that scoping is Core's
   answer and is the thing being replaced. Ported from the primary's taxonomy
   (`dm crates/md-cli/src/parse/reuse.rs`, `Finding::SamePathExpression` and
   `KeyAtDisjointUseSites`) — **convergence, so exempt from Rust-first.**

2. **Move the refusal.** `policyAddressAt` refuses on `BIP388ReusedSlot`;
   `md.DuplicateKeySlot` keeps the F-514 WARNING, whose two sentences are Core
   verdicts and would become false on the new predicate.

3. **Only if R0 answers the question above "yes":** restore the warning block on
   the address branch of `walletPolicyAddressLines`, WITH coverage. It was
   deleted deliberately, because "a test asserting coverage of code that cannot
   run is worse than no test — mutating it away left 1312/1312 green."

4. **Copy.** The refusal sentence must say BIP 388, not Core. Today's
   `composerCopyDuplicateKeys` is written in Core's voice.

## Acceptance

* The corpus check reports `ok/refused 0`, with the baseline rewritten in the
  same commit and each verdict hand-checked (the script demands this).
* `keyed_tr_multi_a` and `keyed_tr_sortedmulti_a` refuse on-device, with a
  message naming BIP 388.
* A control vector that is Core-duplicate but BIP-388-legal either refuses
  correctly or is shown not to exist in this wire format.
* Fork gates: `./sysw/`, non-gui packages, `./gui/` sharded ×24 (all 1338), and
  `gofmt -l .` against the recorded five-file baseline.
* Mutation: disable the internal-key term in the new predicate → the two vectors
  derive again and the corpus check goes red.

## Noted, not resolved

`keyed_tr_multi_a` and `keyed_tr_sortedmulti_a` are two of the three key-reuse
vectors **F-529** says a re-vendor would delete. If that re-vendor lands first,
this plan's acceptance evidence disappears with them. Sequence F-529 against this
before implementing, or pin the two vectors locally.
