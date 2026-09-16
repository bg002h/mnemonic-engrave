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

## Acceptance — written so it can fail in BOTH directions

Round 1's I3 was that `ok/refused 0` plus a same-commit baseline rewrite cannot
fail in the over-refusal direction, and that my claim the script "demands"
hand-checked verdicts was **false**: `--write-baseline` overwrites and exits 0.

* **Must refuse:** `keyed_tr_multi_a`, `keyed_tr_sortedmulti_a` → corpus
  `ok/refused` goes 2 → 0.
* **Must STILL derive**, named individually and asserted BEFORE any baseline
  rewrite: `keyed_compose_tr_sole_sortedmulti_a`,
  `keyed_compose_tr_unsorted_sole_leaf`, and the three
  `keyed_compose_preset_hashlock_gated_*` vectors baselined today. The count
  `ok/agrees` must go **45 → 47**, not merely "not fewer".
* **The probe must exercise the DEVICE path.** Before trusting any corpus number,
  confirm `cmd/policyprobe` reaches `complexAddressSource` for these vectors —
  its own header warns that wrapping the wrong branch "manufactures device
  findings".
* **Mutation:** drop the `!isNums` guard → the two NUMS vectors above must go red.
  Drop the internal-key term → `ok/refused` returns to 2.
* Fork gates: `./sysw/`, non-gui packages, `./gui/` sharded ×24 (all 1338),
  `gofmt -l .` against the five-file baseline.

## Sequencing

`keyed_tr_multi_a` and `keyed_tr_sortedmulti_a` are two of the three key-reuse
vectors **F-529** says a re-vendor would delete, taking this plan's evidence with
them. Settle F-529 first or pin the two vectors locally — round 1 judged the
earlier mitigation inadequate.
