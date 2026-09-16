# R0 round 3 — PLAN_F533_bip388_reuse_predicate.md (re-review of the round-2 fold)

**Verdict: 0 Critical / 1 Important / 4 Minor / 1 Nit — NOT GREEN.**

Read-only at engrave `a013d6d3` (the round-3 plan commit), fork `e4ab97d`, both
trees clean. Nothing was edited in any repo; the only write is this file.

Scope was the two questions in the brief. Round 1's and round 2's settled facts
were taken as settled and are not re-derived. Every number below was measured
this round from `design/policy-corpus-baseline.json`, the 70 vendored
`.phrase.txt` / `.template` vectors, `scripts/policy-generate.py` and the fork
source — none is carried over from a prior report, including where it agrees
with one.

---

## 1. Per-finding disposition of round 2

| # | round 2 finding | disposition |
| --- | --- | --- |
| **C1(a)** | `ok/agrees 45 → 47` is unreachable; the derived distribution was discarded | **Addressed.** The corrected table is the derived one and I re-derived it independently: baseline is 70 = 45 `ok/agrees` + 2 `ok/refused` + 22 `source` + 1 `expand`; the two `ok/refused` entries are exactly `keyed_tr_multi_a` and `keyed_tr_sortedmulti_a`; `run_corpus` short-circuits at `policy-generate.py:319-321` so a device refusal never reaches the rust comparison; `gui.PolicyAddressAt` returning `!ok` is reported as stage `source` (`cmd/policyprobe/main.go:227-232`, and its header states the contract). `policyAddressAt` falls through to `complexAddressSource` for both vectors (`gui/wallet_policy.go:373-383`; `expandedToDescriptor` reports `expandUnsupported` for a taproot script tree). **The table is correct and the refusal surfaces at `source`, not at any other stage.** |
| **C1(b)** | three of five named controls are `wsh` and cannot witness a taproot guard | **Addressed by removal.** The five-control list is gone; no `wsh` vector is named as evidence anywhere in the plan. |
| **C1(c)** | the `!isNums` mutation's blast radius is 8, not 2 | **Partially — and the fold re-created a smaller version of the same defect.** The hardcoded `2` is gone and replaced by an enumeration instruction, which is the right move. But the instruction's set is **9**, the prose's is **8**, and one member cannot redden. See **I1**. |
| **I1** | copy requirement not actionable; the new kind's default landing is a false Core claim | **Addressed.** The plan now carries a dedicated section, diagnoses the fallthrough correctly — verified: `composerCopyDuplicateKeys` (`gui/composer_copy.go:427-446`) is one `if kind == md.DuplicateFewerKeys` and a default returning *"…Bitcoin Core refuses such a descriptor."* — and requires a row in both hand-enumerated gates. Both gates are correctly identified: `gui/modal_fits_test.go:319-332` holds three hand-written rows (`DuplicateFewerKeys` ×2, `DuplicateRefusedByCore` ×1) and `gui/composer_copy_test.go:112-115` two, and neither would fail on a third kind. Residue in **M3**. |
| **I2** | six recorded statements of the superseded design are named nowhere | **Addressed, minus one item round 2 supplied.** Five of the six citations resolve; the list omits `md/duplicate_keys.go:80-89`, which round 2 named explicitly. See **M2**. The claim that the two RED tests are correct checks out (below). |
| **M1** | F-529 mitigation still an "or"; no durable `md/` fixture for the new kind | **Not addressed.** The Sequencing section is verbatim *"Settle F-529 first **or** pin the two vectors locally"*, and no hand-built `trBody{isNums:false, …}` fixture is named. Carried forward as **M4**. |
| **M2** | the pairwise-distinctness delta is unfiled and softened | **Not addressed.** Step 4's sentence is unchanged (*"the shape it names is expressible and derives correctly today"*), and `design/FOLLOWUPS.md` has no such entry — the newest ids are F-608…F-618 and none covers it. Carried forward as **M5**. |
| **N1** | citation drift `gui/policy_address.go:77` | **Not addressed.** Step 2 still cites `:77`. Verified again: line 76 is `if _, kind, err := md.DuplicateKeySlotChunks(collected); …`; line 77 is `return nil, false`. Carried forward as **N1**. |

### The "two tests go red is CORRECT" reasoning — checked, and it holds

`md/duplicate_keys_test.go:63-64` and `gui/composer_flow_test.go:594-595` assert
`DuplicateNone` for `keyed_tr_multi_a` / `keyed_tr_sortedmulti_a` with the
rationale *"Core ACCEPTS: the internal key is outside the miniscript"*. That
sentence stays true: `md/duplicate_keys.go:16-19` records both as ACCEPTED
against Core 31.1, and nothing in this plan re-measures Core. The plan's
reconciliation — the rationale remains a true statement *about Core* while
ceasing to be the predicate's answer, which is what the new kind exists to keep
separable — is sound, and it is the re-scoping round 2's I2 asked for.

---

## 2. NEW findings

### I1 (Important) — the mutation-set instruction yields a set of NINE, its own prose says EIGHT, and one member of the nine cannot turn red

The plan replaces round 2's wrong count with an instruction rather than a
number, which is the right instinct. Executed literally, it produces an
assertion no correct implementation can satisfy.

**The instruction:**

> enumerate the taproot vectors whose internal key is NUMS *and* whose leaves
> reference the slot the bogus zero would collide with, and assert that exact
> SET by name.
> Dropping the `!isNums` guard must turn **every** member of that set red.

**I executed it.** Over all 70 baseline vectors: split each `tr(` template at
its first top-level comma, classify the internal key as NUMS (the literal
`50929b74…803ac0`) versus `@N`, and test the tree for `@0` — the slot the
`keyIndex == 0` default collides with. The set is **nine**:

```
compose_tr_thirty_two_slots                        source      <- CANNOT REDDEN
keyed_compose_preset_kofn_recovery                 ok/agrees
keyed_compose_tr_hash_leaf                         ok/agrees
keyed_compose_tr_nums_three_leaves                 ok/agrees
keyed_compose_tr_sole_sortedmulti_a                ok/agrees
keyed_compose_tr_two_path_distinct_fingerprints    ok/agrees
keyed_compose_tr_two_path_nums                     ok/agrees
keyed_compose_tr_unsorted_sole_leaf                ok/agrees
keyed_tr_pathological                              ok/agrees
```

**The counterexample.** `compose_tr_thirty_two_slots` satisfies the enumeration
criterion exactly — NUMS internal key, `@0` in the first `multi_a` of its tree —
and its baseline entry is already `{"device":"source"}`. Dropping the `!isNums`
guard cannot move it: it is refused upstream of the comparison either way, so
`run_corpus` records `source` before and after and the tool emits **no MOVED
line** for it. An implementer who enumerates nine and asserts "all nine redden"
gets eight, against a correct implementation.

Round 2 supplied this exact qualifier — its table marks the vector
`source (already source; invisible)` — and the fold carried the criterion across
while dropping the qualifier.

**And the plan contradicts itself on the size.** Two bullets below the
instruction:

> A mutation that reddens two when the set is eight is a mutation test that
> passes on a broken guard.

Eight is the right answer for the reddening set; nine is what the stated
procedure returns. Neither number is derived by the procedure the plan gives, so
the section still has the shape round 2 made Critical — a magnitude an
implementer must reconcile by hand — only smaller. The reconciliation the plan
itself warns against is the dangerous one: narrowing the criterion until it
yields eight can narrow it past a vector the guard really protects.

**Remedy — one clause, and the measurement is above.** Scope the enumeration to
vectors currently in `ok/agrees`, and name the exclusion so it is not
rediscovered as a failure:

> Enumerate the vectors that are NUMS-internal-key taproot **and** carry `@0` in
> the taptree **and** are `ok/agrees` today — eight at fork `e4ab97d`, listed
> above — and assert that set by name. `compose_tr_thirty_two_slots` meets the
> first two conditions and is already `source`, so it is invisible to this
> mutation and is deliberately not in the set. Dropping the `!isNums` guard
> moves `ok/agrees 45 → 37` and `source 24 → 32`.

This is a text fix over a measured set, not new work; re-checking it is the same
enumeration, not a review round.

---

## 3. Minors and Nit

### M1 — `ok/refused` does not print as `0`; the key vanishes

Round 2 raised this inside C1 and the fold did not take it. `counts` is built
only from observed values (`policy-generate.py:537-541`: `for v in
observed.values(): … counts[key] = counts.get(key, 0) + 1`), so with both
vectors gone the `ok/refused` line is absent from the output, not zero. The
plan's table says **0**. An implementer asserting the distribution by grepping
for `ok/refused 0` greps for a line that is never printed. One parenthetical:
*"the `ok/refused` line disappears from `counts` rather than printing 0"*.

### M2 — the six places are missing `DuplicateKeySlot`'s own doc comment

Five of the six cited locations resolve (`gui/policy_address.go:48-56` ✓ the
text is at 53-55; `gui/wallet_policy.go:345` ✓; `md/duplicate_keys.go:34-41` ✓
the text is at 39-41; `md/duplicate_keys_test.go:63-64` ✓; `gui/composer_flow_test.go:594`
✓, though the two rows are 594 **and** 595). The omission is
`md/duplicate_keys.go:80-89` — the doc comment on the function being changed:

> *"under tr, every taptree LEAF is its own expression and the internal key is
> in none of them, so a key shared between the internal key and a leaf, or
> between two different leaves, is not a duplicate"*

Under this plan that becomes false for `!isNums`. Round 2 listed it as one of
its four recorded statements; the plan's numbered six drops it, which makes an
exhaustive-looking list non-exhaustive. Minor rather than Important only because
it sits in the file the implementer must edit.

A seventh is falsified by the same change and is named nowhere:
`md/duplicate_keys_test.go:50-56`, *"counting the whole taptree at once instead
of per leaf leaves BOTH tr rows green"* — after the widening those rows are the
new kind, so "green" stops describing them. Worth folding into the same list.

### M3 — the copy remedy closes this kind and leaves the shape that produced it

The plan says *"Add the branch"*. That fixes the defect. Round 2's alternative —
a `switch kind` whose default cannot silently absorb a new value, or returning
the Core sentence only for `DuplicateRefusedByCore` explicitly — also fixes the
*next* kind, at the same cost. A reviewer's prescribed fix is not authoritative,
so this is recorded, not required. Separately, the two gates are named only as
"the fit and golden gates"; citing `gui/modal_fits_test.go:319-332` and
`gui/composer_copy_test.go:112-115` would make the instruction a grep. Note also
that `composerCopyNoAddressesDuplicateKeys` already serves the new kind
correctly through `noAddressLines` (`gui/wallet_policy.go:306-316`, kind-agnostic)
— the plan's *"the new kind must not inherit them"* reads as work to do when it
is already satisfied.

### M4 — round 2's M1 stands: F-529 is still an "or", and the new kind has no fixture a re-vendor cannot delete

Unchanged from round 2, and the acceptance is still entirely corpus-based
against two vectors F-529 says a re-vendor replaces.
`md/duplicate_keys_test.go:117-129` already builds trees directly, so
`trBody{isNums:false, keyIndex:0, tree:&branch(key(0), key(1))}` is a one-line
durable fixture. Decide the F-529 question rather than restating it.

### M5 — the "17 taproot vectors in `ok/agrees`" figure is a name-grep artifact; it is 18

The plan states *"The corpus holds **17** taproot vectors currently in
`ok/agrees` (counted from `design/policy-corpus-baseline.json`)"*. The baseline
holds ids and verdicts only, so the count can only have come from matching a
`tr` token in the id: that yields exactly 17. Classifying by the actual
descriptor yields **18** — the extra one is `keyed_compose_preset_kofn_recovery`,
whose id carries no `tr` token and which is a NUMS taproot **and a member of the
protected set above**. The figure is context rather than load-bearing (the plan
correctly refuses to hardcode a mutation count), so it is Minor — but it is a
hand-count of a thing a command counts, and the one vector it drops is one this
section is about.

### N1 — `gui/policy_address.go:77` is still cited; the predicate call is line 76

Round 2's N1, unfolded. Line 76 is the `DuplicateKeySlotChunks` call, line 77 is
`return nil, false`.

---

## 4. Verified this round — do not re-check in a fold

* **The stage table is right.** `ok/refused 2→0` (see M1 on how it prints),
  `source 22→24`, `ok/agrees 45 unchanged`, `expand 1`. The refusal surfaces at
  stage `source` and at no other stage: `policyAddressAt` → `complexAddressSource`
  → `!ok` → `cmd/policyprobe` `stageSource`.
* **`ok/agrees` really is unchanged**, re-derived independently: of the 22
  taproot vectors with templates, the only ones whose internal-key slot also
  appears in the taptree are `keyed_tr_multi_a` and `keyed_tr_sortedmulti_a`,
  both `ok/refused` today. The 16 baseline vectors without a `.template` are all
  `source` or `expand`, so none can leave `ok/agrees`.
* **Round 2's figure of 8 is right** for the reddening set, and the ninth member
  of the enumerated set is the one I1 is about.
* **`./gui/` is 1338 top-level tests**, measured the way
  `scripts/gui-shard-test.sh` enumerates (`go test ./gui/ -list '.*'` filtered to
  `Test|Example|Fuzz`, sorted unique) at fork `e4ab97d` on go1.26.7. The plan's
  number is correct.
* **Every round-3 citation except `gui/policy_address.go:77` resolves**:
  `scripts/policy-generate.py:311-327` ✓, `md/canonicalize.go:109-116` ✓,
  `gui/md1_gather.go:205` ✓, `gui/composer_copy.go:427-447` ✓ (the function ends
  at 446), `md/duplicate_keys.go:34-41` ✓, `md/duplicate_keys_test.go:63-64` ✓,
  `gui/composer_flow_test.go:594` ✓.
* **The status header is stale** — the plan still reads *"Status: DRAFT round 2,
  awaiting R0"* at `a013d6d3`, the round-3 commit. Not counted as a finding.

---

## 5. Verdict

**NOT GREEN — 0 Critical / 1 Important / 4 Minor / 1 Nit.**

The design is settled and the round-2 Critical is genuinely closed: the stage
table is the derived one, it is correct, and I could not construct a stage at
which the refusal would surface other than `source`. The copy section and the
six-places section are both real, actionable additions.

The one blocker is that the fold carried round 2's enumeration criterion across
without its qualifier, so the mutation gate as written cannot pass on a correct
implementation and the plan states two different set sizes, neither of which its
own procedure returns. The fix is one clause over a set already measured in this
report; folding it needs a recomputation of the enumeration, not a fourth review
round.
