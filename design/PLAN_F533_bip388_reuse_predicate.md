# PLAN — F-533: refuse taproot internal-key reuse

**Status: GREEN (0C/0I) at R0 round 4, 2026-09-16 — CLEARED FOR IMPLEMENTATION.**
Risk set: admission, address- and funds-adjacent.

Four rounds, each finding something real: 2C → 1C → 0C/1I → GREEN. Reports in
`design/agent-reports/plan-F533-R0-round{1,2,3,4}.md`, each persisted verbatim in
its own commit before the fold that answered it. **A GREEN expires:** if the fork
moves before this is implemented, re-validate against *"what did that change
falsify here?"* rather than re-reviewing from scratch.

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
   (`gui/policy_address.go:76`), which is what `gatheredDescriptorFlow` actually
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
| `ok/refused` | 2 | **absent** — the bucket VANISHES from `counts` rather than printing `0`, so assert its absence, not the string "0" |
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

### The `!isNums` mutation — the set, and what it can prove

Round 2: three of my five named controls were `wsh` and could not witness a
taproot guard at all. Round 3: my replacement instruction was **unsatisfiable** —
executed literally it enumerates NINE vectors, and one of them,
`compose_tr_thirty_two_slots`, is already at stage `source`
(`{"device": "source"}` in the baseline, verified) and therefore cannot redden.
Round 2 had supplied that qualifier and my fold dropped it. **A gate that cannot
pass is the same defect as one that cannot fail.**

So, precisely:

* Enumerate the taproot vectors whose internal key is NUMS and whose leaves
  reference the slot the bogus zero would collide with. **Determine taproot-ness
  from the DECODED WRAPPER, never from the vector name** — my "17 taproot
  `ok/agrees`" was a name-grep artifact and the real figure is 18.
* **The reddening set is that enumeration INTERSECTED WITH `ok/agrees`.** Round 2
  measured it at **8**. Members already at another stage are excluded and named,
  `compose_tr_thirty_two_slots` among them.
* Dropping the `!isNums` guard must turn **every member of the reddening set**
  red — not two of them, and not a count taken on faith.
* `keyed_compose_tr_sole_sortedmulti_a` and `keyed_compose_tr_unsorted_sole_leaf`
  are confirmed members (round 1 C1); they are a floor, not the set.

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
* **The gates enumerate kinds BY HAND and would stay green.** They are
  `gui/composer_copy_test.go:112` and `:114` (one row per existing kind), the fit
  gate in `gui/modal_fits_test.go`, and `gui/composer_flow_test.go`. Add the new
  kind to each, or the copy ships unmeasured.
  **NOT `gui/duplicate_seat_address_test.go`** — round 4 checked: it derives its
  expectations dynamically rather than enumerating kinds, so it needs no edit.
  Telling an implementer to touch a file that needs no touching is how a plan
  spends attention it has not earned.
* The function is `if kind == md.DuplicateFewerKeys { … }` followed by an
  unconditional return, so the new kind needs its own branch BEFORE that return
  — not a reworded fallthrough. Its replacement must be MEASURED for line count
  on the Inspect screen, which pages at 7 lines; the existing comment records
  that 122 chars landed on 8 lines "only because of where the words break".
* The screens a refused card lands on — *"This device can't derive addresses"*,
  *"Complex policy - display only"* — are already recorded as defects; the new
  kind must not inherit them.

## Seven places record the decision this reverses (round 2 I2, round 3 M)

The rewrite reverses "a second predicate, not a wider read", which is written
down in seven places. Each must be updated or explicitly superseded **in the
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
7. `md/duplicate_keys.go:80-89` — `DuplicateKeySlot`'s OWN doc comment, which
   this change falsifies. Round 3 caught its omission; a function whose doc
   contradicts its behaviour is the next reader's trap.

**Those two tests going red is CORRECT** — the behaviour is deliberately
changing — and their rationale stays true about *Core* while ceasing to be the
predicate's answer. That is exactly why the new KIND exists: it keeps Core's
verdict available for the Core-voiced sentences instead of overwriting it.

## Sequencing

`keyed_tr_multi_a` and `keyed_tr_sortedmulti_a` are two of the three key-reuse
vectors **F-529** says a re-vendor would delete, taking this plan's evidence with
them.

**Decision, not an option:** pin both vectors locally BEFORE implementing, in the
same commit as the predicate, so this work does not depend on F-529's timing in
either direction. An "or" here is how a plan acquires a dependency nobody owns —
round 1 judged the earlier mitigation inadequate and round 3 found it still
phrased as a choice.
