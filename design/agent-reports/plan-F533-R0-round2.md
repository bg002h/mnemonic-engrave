# R0 round 2 — PLAN_F533_bip388_reuse_predicate.md (re-review of the rewrite)

**Verdict: 1 Critical / 2 Important / 2 Minor / 1 Nit — NOT GREEN.**

Read-only at engrave `f92303e5` (plan committed there), fork `e4ab97d`, both trees
clean. Nothing was edited in any repo; the only write is this file. Every count
below is machine-read from `design/policy-corpus-baseline.json`,
`scripts/policy-generate.py` and the 70 vendored `.template` files — none is
described or carried over.

Round 1's settled facts were taken as settled and are not re-derived.

---

## 1. Per-finding disposition of round 1

| # | round 1 finding | disposition |
| --- | --- | --- |
| **C1** | unguarded internal-key count falsely refuses every NUMS taproot | **Addressed.** Step 1 now reads "only when `!isNums`" and cites the established pattern. Verified at the cited lines: `md/canonicalize.go:108-116` is `case trBody:` → `if !b.isNums { … }`, exactly as claimed. The acceptance carries the `!isNums` mutation row round 1 asked for (its *count* is wrong — see C1 below — but the row exists). |
| **C2** | step 2 named `policyAddressAt`, not the seam both routes pass | **Addressed.** Step 2 says the gate stays in `complexAddressSource`, "No call site moves", and names the false-GREEN mechanism. Verified: `gui/md1_gather.go:205` is `if at, ok := complexAddressSource(collected, keys); ok {`. The design change (widen the predicate rather than move the gate) makes the round-1 remedy moot rather than merely satisfied. |
| **I1** | refused cards land on a device-limitation sentence | **Partially.** Step 3 states the requirement and names the two wrong *screens*, but names no function, and the new kind's actual landing place emits a **false claim about Bitcoin Core**. See **I1**. |
| **I2** | `KeyAtDisjointUseSites` claim structurally unimplementable | **Partially.** The citation is gone and step 4 states the scope honestly ("This plan ports one rule"). Round 1's remedy (b) had a third clause — file the pairwise-distinctness delta as its own follow-up — which was not done. See **M2**. |
| **I3** | acceptance cannot fail in the over-refusal direction | **Not addressed — regressed.** The section was rewritten and now asserts a distribution that **no correct implementation can produce**, discarding the machine-counted one round 1 supplied. See **C1**. |
| **M1** | `BIP388ReusedSlot(tree node)` not callable from `gui` | **Addressed by removal.** No new exported function exists; the change is inside `DuplicateKeySlot`, and `DuplicateKeySlotChunks` (`md/duplicate_keys.go:172`) is the gui-callable form all three consumers already use. Moot. |
| **M2** | mutation covers one term; add a cross-leaf fixture | **Addressed by scope change.** Step 4 no longer widens the per-leaf scoping, so the per-leaf mutation row is moot. `md/duplicate_keys_test.go:116-162` already pins that rule and is **unaffected** by the widening — its `trWith` helper builds `trBody{isNums: true, …}` (line 128), so the `!isNums` guard exempts it. Verified. |
| **M3** | F-529 mitigation stated as a choice, never made | **Not addressed.** "Settle F-529 first **or** pin the two vectors locally" is still an or, and round 1's proposed resolution (hand-built `md/` unit fixtures as the durable evidence) was not adopted. See **M1**. |
| **N1** | citation drift `md/expand.go:33-42` | **Addressed by removal** — that citation is gone. New drift introduced; see **N1**. |

---

## 2. The load-bearing judgement: the reconciliation is SOUND

> One predicate gets WIDER rather than a second being added — rationalisation, or
> sound?

**Sound.** I tried to build the counterexample and could not, and the enumeration
is why.

A second predicate's only job would be to let the refusal see a set the warning
does not. The drift hazard `gui/policy_address.go:71-76` names is real and
concrete: two evaluations can disagree, and the disagreement surfaces as "warns
and derives" or "refuses in silence" — which is exactly the shape round 1's I1
constructed from the round-1 design. One predicate returning a **kind** collapses
both consequences onto a single evaluation, so they cannot disagree by
construction.

The cost is that `md.DuplicateKeySlot` stops answering exactly Core's question.
That cost is paid by the kind, not lost: `DuplicateRefusedByCore` and
`DuplicateFewerKeys` remain Core verdicts, and the new kind is precisely the
"Core accepts, BIP 388 does not" delta. Any consumer that needs Core's boolean
can recover it from the kind. The file's own taxonomy comment
(`md/duplicate_keys.go:43-44`, "DuplicateKind says WHICH harm a repeated slot
carries") documents this as the extension point.

**Enumerated consumers — every one survives.** Non-test call sites of
`DuplicateKeySlot`/`…Chunks` (grep over the fork, excluding `_test.go`): three.

| site | shape | effect of a new kind |
| --- | --- | --- |
| `gui/policy_address.go:76` (F-531 gate) | `kind != md.DuplicateNone` | refuses — **the intent** |
| `gui/wallet_policy.go:306` (`noAddressLines`) | `kind != md.DuplicateNone` | warns + "No addresses…" — correct |
| `gui/md1_gather.go:222` (`duplicateRefusalBody`) | `err != nil \|\| kind == md.DuplicateNone` | modal fires — correct |

All three are kind-agnostic booleans. No consumer switches exhaustively on the
enum, none indexes an array by it, none serialises it. **Adding a kind breaks no
non-test consumer** — with one exception, which is I1: the *copy* function they
all funnel into is not kind-agnostic, and its fallthrough is wrong.

For completeness, the test-side consumers are enumerated in **I2**.

---

## 3. NEW findings

### C1 — the acceptance asserts a corpus distribution no correct implementation can produce; its control set is 3/5 irrelevant and 6/8 missing

Three measurable errors in one bullet, compounding, in the section titled
*"written so it can fail in BOTH directions"*.

**(a) `ok/agrees` 45 → 47 is unreachable.** `run_corpus`
(`scripts/policy-generate.py:311-327`) is explicit:

```python
if not dev.get("ok"):
    observed[cid] = {"device": dev.get("stage", "?")}
    continue
...
if rust is None:      observed[cid] = {"device": "ok", "rust": "refused"}
elif rust == dev[...]: observed[cid] = {"device": "ok", "rust": "agrees"}
```

A **device refusal never reaches the rust comparison at all** — it is recorded as
its stage, and `complexAddressSource` returning `!ok` is `stageSource`
(`cmd/policyprobe/main.go`, "A policy the device cannot derive from is a RESULT,
reported as stage `source`"). So the two target vectors move
`{"device":"ok","rust":"refused"}` → `{"device":"source"}`. **Nothing moves into
`ok/agrees`, in either direction, under any correct implementation.** Machine-read
from the baseline (70 = 45 + 2 + 22 + 1, re-confirmed), the only distribution a
correct fix can produce is the one round 1 already supplied and this fold
discarded:

```
corpus: 70 vectors
  expand           1
  ok/agrees       45      (UNCHANGED)
  source          24      (was 22)
MOVED exactly: keyed_tr_multi_a, keyed_tr_sortedmulti_a — and no others
```

(`ok/refused` does not print as `0`; the key vanishes from `counts`, which is
built only from observed values — `policy-generate.py:537-541`. An acceptance
that says "goes 2 → 0" should say "the `ok/refused` line disappears", or an
implementer greps for a line that is not there.)

**Why this is Critical and not a typo.** The `ok/agrees` number is the *entire*
over-refusal guard — the direction round 1's C1 came from. A number that no
correct run produces will be discarded by the implementer as stale, and what is
left is "not fewer", which this plan itself rejects as insufficient. That is
round 1's I3 restored: the gate is unfalsifiable in the over-refusal direction,
now wearing a precise-looking number. The plan was handed the derived
distribution and substituted a hand-made one.

**(b) three of the five named controls cannot be affected by this change at all.**
The `keyed_compose_preset_hashlock_gated_*` vectors are **wsh, not taproot**:

```
keyed_compose_preset_hashlock_gated_hash160   wsh(or_i(and_v(v:pkh(@0/…),hash160(…)),and_v(v:pkh(@1/…),older(26280))))
keyed_compose_preset_hashlock_gated_hash256   wsh(or_i(…hash256(…)…))
keyed_compose_preset_hashlock_gated_ripemd160 wsh(or_i(…ripemd160(…)…))
```

Step 1 touches only `DuplicateKeySlot`'s `tagTr` arm. These three route through
`case tagWsh` and are invariant under the change **and under both of its
mutations**. They are named as evidence that the `!isNums` guard works and cannot
witness it either way. (They appear to have been carried over from the F-533
FOLLOWUPS note, where they are the three vectors that grew the corpus from 67 to
70 — a different fact.)

**(c) the `!isNums` mutation's blast radius is 8, not 2.** Measured over all 70
templates: the vectors that are NUMS-taproot **and** carry `@0` in the taptree —
i.e. the ones the guard protects — are nine, eight of them `ok/agrees`:

```
keyed_compose_preset_kofn_recovery               ok/agrees
keyed_compose_tr_hash_leaf                       ok/agrees
keyed_compose_tr_nums_three_leaves               ok/agrees
keyed_compose_tr_sole_sortedmulti_a              ok/agrees   <- named
keyed_compose_tr_two_path_distinct_fingerprints  ok/agrees
keyed_compose_tr_two_path_nums                   ok/agrees
keyed_compose_tr_unsorted_sole_leaf              ok/agrees   <- named
keyed_tr_pathological                            ok/agrees
compose_tr_thirty_two_slots                      source      (already source; invisible)
```

Dropping the guard therefore moves **ok/agrees 45 → 37, source 24 → 32**, not
"the two NUMS vectors go red". The mutation still detects (two red is enough to
fail), but the plan's statement of what it protects is wrong by a factor of four,
and a mutation row whose expected magnitude is wrong is one an implementer will
"reconcile" by editing the expectation.

**Remedy.** Replace the two bullets with the derived block in (a); name the eight
protected vectors (or say "every NUMS-taproot vector carrying `@0` in its tree —
eight `ok/agrees` today"); drop the three wsh vectors or move them to a
"deliberately unaffected" line; state the mutation's expected magnitude as
`ok/agrees 45 → 37`. Every number here is one command away and none of them
should be written by hand again.

---

### I1 — step 3's copy requirement is not actionable, and the new kind's default landing is a FALSE claim about Bitcoin Core

Step 3 says the new kind "needs its own [sentence]" and that "the existing
Core-voiced sentences **stay on the Core kinds** and are still true there". That
describes a correspondence the code does not implement. `composerCopyDuplicateKeys`
(`gui/composer_copy.go:427-447`) is not a switch over kinds — it is one `if` and
a fallthrough:

```go
if kind == md.DuplicateFewerKeys {
        return fmt.Sprintf("Check before funding: slot @%d fills more than one seat, …", slot)
}
return fmt.Sprintf("Check before funding: slot @%d repeats in one script, and "+
        "Bitcoin Core refuses such a descriptor.", slot)
```

A third kind lands on the **default arm**, so an operator holding
`keyed_tr_multi_a` reads, on the Engrave Wallet Policy consent (one confirm from
steel) and in the Inspect modal:

> "Check before funding: slot @0 repeats in one script, and Bitcoin Core refuses
> such a descriptor."

Both halves are false, and the repo has measured them false: `@0` repeats
*across* the internal key and a leaf, not within one script — that is the entire
premise of this plan — and `md/duplicate_keys.go:18-19` records
`keyed_tr_multi_a  ACCEPTED` / `keyed_tr_sortedmulti_a  ACCEPTED` against Core
31.1. The kind whose whole reason for existing is "Core accepts this" would ship
telling the operator Core refuses it.

**And the two gates that would catch a copy defect enumerate kinds by hand, so
they stay green.**

| gate | today | with a third kind |
| --- | --- | --- |
| `gui/modal_fits_test.go:319-334` — draws the modal and checks nothing falls below the fold | three hand-written rows, `DuplicateFewerKeys` ×2 and `DuplicateRefusedByCore` ×1 | **no row** — the new sentence is never measured |
| `gui/composer_copy_test.go:112-115` — golden string per kind | two hand-written rows | **no row** |

The fit gate's own comment says why that matters: *"the modal concatenates two
sentences, and the only other test that reads this screen asserts a
40-character prefix of the FIRST one, so an edit pushing the refusal past the
fold passes everything else."* A new sentence that overflows the paginated
Inspect screen is a silent refusal — round 1's I1 harm, arriving through a
different door.

`composerCopyNoAddressesDuplicateKeys()` ("…for a wallet that reuses a key",
`gui/composer_copy.go:474-476`) is kind-agnostic and already correct for the new
kind; no change needed there.

**Remedy.** Step 3 names the places, not the screens: restructure
`composerCopyDuplicateKeys` into a `switch kind` whose default cannot silently
absorb a new value (or return the Core sentence only for
`DuplicateRefusedByCore` explicitly); add the new kind's sentence in BIP 388's
voice; add one `modal_fits_test.go` row for `new-kind + composerCopyNoAddresses…`
and one `composer_copy_test.go` golden row. The fit row is not optional — it is
the only thing standing between the new sentence and the fold.

---

### I2 — the rewrite overrules two written invariants and four recorded statements of the superseded design, and names none of them

The widening is a 180° reversal of a decision that is written down in six places
in the fork and this repo. An implementer handed this plan will meet them as
failing tests and as comments instructing them to do the opposite, with nothing
in the plan authorising the change.

**Two tests go RED, and both carry a doctrine, not just an expectation.**

1. `md/duplicate_keys_test.go:63-64` — `{"keyed_tr_multi_a", DuplicateNone, …}`,
   `{"keyed_tr_sortedmulti_a", DuplicateNone, …}`. Its doc comment (lines 44-46)
   says, in as many words:

   > *"If this test ever disagrees with the table, **the predicate is wrong, not
   > the table**: warning about a wallet Core accepts trains the operator to tap
   > through the warning that matters."*

2. `gui/composer_flow_test.go:594-595` — same two vectors, plus a **negative
   assertion on the screen**: it fails if the consent shows the duplicate warning,
   with *"the consent screen warns about duplicate keys on a wallet Core accepts
   …; a warning that cries wolf is one the operator reads past."*

The plan's design deliberately warns on a wallet Core accepts. That may well be
right — the operator's 2026-08-30 ruling on BIP-388-forbidden wallets says the
device does not serve them — but the invariant has to be **re-scoped in writing**
("the cries-wolf rule binds the *Core-voiced* sentence, not the warning as such")
rather than silently overruled by an implementer reconciling a red test against a
comment telling them they broke the predicate. That is how a prior review's
funds finding gets deleted as an inconvenience.

**Four recorded statements of the second-predicate design become false and are
named nowhere in the plan:**

| location | what it says now |
| --- | --- |
| `md/duplicate_keys.go:38-42` | *"Filed as F-533, whose remedy is a **SECOND predicate** … **not a change to this one**, whose two sentences are Core verdicts and would become false."* |
| `md/duplicate_keys.go:81-89` (`DuplicateKeySlot`'s own doc) | *"the internal key is in none of them, so a key shared between the internal key and a leaf … is **not a duplicate**"* — becomes false for `!isNums` |
| `gui/policy_address.go:49-57` | *"**Do not read the line below as the wider rule**; closing it needs a second predicate, not a wider read of this one."* |
| `gui/wallet_policy.go:344-348` | *"F-533 proposes moving the refusal onto a BIP-388 predicate while the warning stays on `md.DuplicateKeySlot`, which would do exactly that — **this block has to come back**"* — the plan correctly decides the block stays deleted, but round 1 asked for the comment to be updated to say *why* (the case is empty in md1), and the plan folded the decision without the comment |

Plus `design/FOLLOWUPS.md` F-533 ("What closing it needs … a second predicate …
Then the refusal moves onto it while the F-514 warning stays on
`md.DuplicateKeySlot`"), which `git log -- design/FOLLOWUPS.md` confirms has not
been touched since `347af936`, before round 1's report.

**Remedy.** Add a step: *"Six recorded statements of the superseded design must be
rewritten in the same commit"*, with the table above, and state explicitly that
the two RED tests are **expected** and how their invariant is re-scoped. This is
the incomplete-propagation class — grep the superseded phrasing, don't leave it
for the implementer to discover as a conflict.

---

### M1 — round 1's M3 stands: the F-529 mitigation is still an "or", and the new kind gets no durable fixture

The plan still reads *"Settle F-529 first **or** pin the two vectors locally"*,
which is the undecided form round 1 flagged. It matters more now than it did in
round 1, because round 2's acceptance is **entirely corpus-based**: five named
vectors, two mutations, all against the vendored set — and two of those vectors
are ones F-529 says a re-vendor replaces with reuse-free upstream policies of the
same name, after which the new rule has no coverage anywhere and the corpus reads
green.

Round 1's proposed resolution is a one-liner against existing precedent:
`md/duplicate_keys_test.go` builds trees directly (`trWith`, `branch`, `key` at
lines 117-129), so `trBody{isNums: false, keyIndex: 0, tree: &branch(key(0), key(1))}`
is a durable fixture for the new kind that no re-vendor can delete. Name it in
the plan and decide the F-529 question rather than restating it.

### M2 — round 1's I2 remedy (b) is two-thirds done; the delta is unfiled and softened

Step 4 correctly drops the `KeyAtDisjointUseSites` citation and states the scope.
Round 1's remedy (b) had a third clause — *"file the pairwise-distinctness delta
as its own follow-up with the reproduction above"* — and `design/FOLLOWUPS.md` has
no such entry. The plan's sentence *"the shape it names is expressible and
**derives correctly today**"* is true and incomplete: the shape (one xpub at two
slots with different use-sites) is also **BIP-388-forbidden and refused by the
primary** (`crates/md-cli/tests/duplicate_key_slots.rs:339`). As written, a future
reader takes that clause as "no problem here". File it, and say which half is
"correct" (the address) and which is not (the admission).

### N1 — citation drift at `gui/policy_address.go:77`

Step 2 cites the F-531 gate as `gui/policy_address.go:77`. Line 77 is
`return nil, false`; the predicate call is line **76**, which is what round 1 and
the code's own comment cite. One line. (Every other new citation checks out:
`gui/md1_gather.go:205` ✓, `md/canonicalize.go:109-116` ✓, `md/md.go:114-118` ✓,
`md/md.go:657-671` ✓.)

---

## 4. Verified sound in the rewrite — do not re-check next round

* **The one-wider-predicate reconciliation is sound**, for the reason in §2: three
  kind-agnostic consumers, no exhaustive switch, and the kind preserves Core's
  answer. Not a rationalisation.
* **`!isNums` is the right guard and is the established pattern.**
  `md/canonicalize.go:108-116` gates identically on `case trBody:`, and
  `md/md.go:114-118` confirms `keyIndex` is a sibling of `tree`.
* **Step 2's seam is correct.** `gui/md1_gather.go:205` calls
  `complexAddressSource` directly; the gate at `gui/policy_address.go:76` is on
  the path both routers take; no call site needs to move.
* **Exactly two corpus vectors move**, confirmed independently of the baseline by
  scanning all 70 templates: `keyed_tr_multi_a` and `keyed_tr_sortedmulti_a` are
  the only ones whose `tr(@N…)` internal-key slot also appears in the taptree.
  `md/duplicate_keys_test.go:104-108` records the same fact from the other side.
* **The per-leaf scoping test is unaffected** — `trWith` sets `isNums: true`.
* **Adding a `DuplicateKind` value breaks no non-test consumer** (§2 table). The
  test-side impact is fully enumerated in I1 and I2; there is nothing else.

---

## 5. Verdict

**NOT GREEN — 1 Critical / 2 Important / 2 Minor / 1 Nit.**

The design question round 1 inverted is now answered correctly, and the two
Criticals are closed. What the rewrite did not carry across is round 1's I3: the
acceptance still cannot fail in the over-refusal direction, and now says so with
a number — 45 → 47 — that no correct implementation produces. Every figure in
that section is one command away; none of them should have been written by hand.
