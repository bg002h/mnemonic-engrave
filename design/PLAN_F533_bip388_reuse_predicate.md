# PLAN — F-533: refuse taproot internal-key reuse

**Status: DRAFT round 2, awaiting R0.** No code until 0C/0I. Risk set: admission,
address- and funds-adjacent.

**Baseline.** fork `e4ab97d`, engrave `c6ed5e87`, dm `d8bb6d2d`.

**Round 1 was NOT GREEN (2C/3I/3M/1N)** — `design/agent-reports/plan-F533-R0-round1.md`,
persisted verbatim in `c6ed5e87`. Its answer to the load-bearing question inverted
this plan's design, so round 2 is a rewrite rather than a patch.

## The question round 1 settled: NO

> Can one key slot carry two use-sites with DISJOINT multipath in the md1 wire?

**It cannot.** Use-site is per-`@N` — `descriptor.useSite` plus strictly-ascending
per-idx TLV overrides (`readSparseTLVIdx`, `md/md.go:657-671`) — and a tree
occurrence carries only an index. The primary already records this:
`Finding::MultipathDisjoint`, *"md1 cannot express it, F-417"*.

**Consequence, and it is the whole design:** in this wire format
**Core-duplicate is a STRICT SUBSET of BIP-388-forbidden**. Round 1's plan
claimed the opposite and built a two-predicate design on it. There is no policy
that is Core-duplicate and BIP-388-legal, so:

* there is no case where the refusal lifts while a duplicate remains, so the
  deleted warning block on the address branch **stays deleted**; and
* **one predicate still serves both consequences**, which is what
  `gui/policy_address.go:71-76` asks for in as many words: *"Two predicates would
  drift, and the drift would show up as a screen that warns and derives, or one
  that refuses in silence."*

That comment and F-533's entry disagreed — the entry called for a second
predicate, the code argued against one. The nesting result reconciles them: the
predicate gets **wider**, not doubled.

## The change

1. **Widen `md.DuplicateKeySlot`'s `tagTr` arm** to consider the taproot internal
   key, `trBody.keyIndex`, **only when `!isNums`**. SPEC §7: `is_nums=true` means
   the internal key is the NUMS H-point and not a placeholder reference —
   `md/canonicalize.go:109-116` already skips registration on exactly that
   condition, and round 1's C1 showed that counting it unconditionally falsely
   refuses `keyed_compose_tr_sole_sortedmulti_a` and
   `keyed_compose_tr_unsorted_sole_leaf`, both `ok/agrees` today.

2. **Return a NEW kind** for this case (working name
   `DuplicateTaprootInternalKey`), so the refusal is one line and the COPY can
   differ. No call site moves — the F-531 gate stays in `complexAddressSource`
   (`gui/policy_address.go:77`), which is what `gatheredDescriptorFlow` actually
   calls (`gui/md1_gather.go:205`). Round 1's C2: naming `policyAddressAt` would
   have reopened F-531 *and* produced a FALSE GREEN, because `cmd/policyprobe`
   wraps the router while the device takes the branch.

3. **Copy, per kind.** The existing Core-voiced sentences stay on the Core kinds
   and are still true there. The new kind needs its own, in BIP 388's voice, and
   the screens a refused card lands on — *"This device can't derive addresses"*
   and *"Complex policy - display only"* — must not be what it gets (round 1 I1;
   the repo already records both as defects).

4. **No `KeyAtDisjointUseSites` port** (round 1 I2). A tree-only predicate cannot
   compute it, and the shape it names is expressible and derives correctly today.
   This plan ports one rule: a key slot may not appear at both the taproot
   internal key and inside the taptree.

## Acceptance — corrected stage accounting (round 2 C1)

Round 2's Critical: I asserted `ok/agrees 45 → 47`, **and that number is
unreachable**. `scripts/policy-generate.py:311-327` short-circuits —

```python
if not dev.get("ok"):
    observed[cid] = {"device": dev.get("stage", "?")}
    continue
rust, _ = rust_addresses(...)
```

— so a DEVICE refusal never reaches the rust comparison at all. The two target
vectors are in `ok/refused` today, not `ok/agrees`; refusing them moves them to
their refusal STAGE, not into agreement. Verified by reading the script.

**The only correct distribution:**

| bucket | before | after |
| --- | --- | --- |
| `ok/refused` | 2 | **0** |
| `source` | 22 | **24** |
| `ok/agrees` | 45 | **45, unchanged** |
| `expand` | 1 | 1 |

`ok/agrees` staying at **45** is the over-refusal guard, and it is the assertion
that matters: any vector this predicate wrongly refuses leaves that bucket.
Assert the whole distribution, before any `--write-baseline`.

**`--write-baseline` cannot be part of the evidence.** Round 2 also killed my
round-1 claim that the script "demands" hand-checked verdicts: it overwrites and
exits 0. Rewrite the baseline only AFTER the distribution above is asserted, in
the same commit, never as the check itself.

### The `!isNums` mutation, with a measured set rather than a count

Round 2: my five named controls were wrong — **three of them are `wsh`**, so they
cannot witness a taproot guard at all, and the mutation's real blast radius is
larger than the two I named (round 2 measured **8**).

The corpus holds **17** taproot vectors currently in `ok/agrees` (counted from
`design/policy-corpus-baseline.json`). So:

* **Do not hardcode a count.** At implementation time, enumerate the taproot
  vectors whose internal key is NUMS *and* whose leaves reference the slot the
  bogus zero would collide with, and assert that exact SET by name.
* Dropping the `!isNums` guard must turn **every** member of that set red. A
  mutation that reddens two when the set is eight is a mutation test that passes
  on a broken guard.
* `keyed_compose_tr_sole_sortedmulti_a` and `keyed_compose_tr_unsorted_sole_leaf`
  are confirmed members (round 1 C1) — they are a floor, not the set.

### Still required

* Dropping the internal-key term → `ok/refused` returns to 2.
* Fork gates: `./sysw/`, non-gui packages, `./gui/` sharded ×24 (all 1338),
  `gofmt -l .` against the five-file baseline.
* The probe reaches `complexAddressSource` for these vectors — `cmd/policyprobe`
  wraps the router, and its own header warns that wrapping the wrong branch
  "manufactures device findings".

## Copy is a code change, not a wish (round 2 I1)

`composerCopyDuplicateKeys` (`gui/composer_copy.go:427-447`) is an `if` plus a
fallthrough, so a NEW kind falls through to **"Bitcoin Core refuses such a
descriptor"** — which this repo itself measured FALSE for both target vectors
(Core 25.0.0 accepts them). Shipping the kind without touching this function
would print a sentence the repo has already disproved.

* Add the branch, in BIP 388's voice, naming the internal-key-and-leaf shape.
* **The fit and golden gates enumerate kinds BY HAND and would stay green** — add
  the new kind to both, or the copy ships unmeasured.
* The screens a refused card lands on — *"This device can't derive addresses"*,
  *"Complex policy - display only"* — are already recorded as defects; the new
  kind must not inherit them.

## Six places record the decision this reverses (round 2 I2)

The rewrite reverses "a second predicate, not a wider read", which is written
down in six places. Each must be updated or explicitly superseded **in the
implementing commit** — not left to contradict the code:

1. `gui/policy_address.go:48-56` — *"closing it needs a second predicate, not a
   wider read of this one"*
2. `gui/wallet_policy.go:345` — *"F-533 proposes moving the refusal onto a
   BIP-388 predicate"*
3. `md/duplicate_keys.go:34-41` — *"whose remedy is a SECOND predicate … not a
   change to this one, whose two sentences are Core verdicts and would become
   false"*
4. `md/duplicate_keys_test.go:63-64` — asserts `DuplicateNone` for
   `keyed_tr_multi_a` and `keyed_tr_sortedmulti_a`, rationale *"Core ACCEPTS: the
   internal key is outside the miniscript"*
5. `gui/composer_flow_test.go:594` — the same two rows
6. F-533's own entry in `design/FOLLOWUPS.md`

**Those two tests going red is CORRECT** — the behaviour is deliberately
changing — and their rationale stays true about *Core* while ceasing to be the
predicate's answer. That is exactly why the new KIND exists: it keeps Core's
verdict available for the Core-voiced sentences instead of overwriting it.

## Sequencing

`keyed_tr_multi_a` and `keyed_tr_sortedmulti_a` are two of the three key-reuse
vectors **F-529** says a re-vendor would delete, taking this plan's evidence with
them. Settle F-529 first or pin the two vectors locally — round 1 judged the
earlier mitigation inadequate.
