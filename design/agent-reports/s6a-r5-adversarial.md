# S6a R5 — independent adversarial review (closing-round attempt)

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
**Code:** `/scratch/code/shibboleth/seedhammer` @ `main` = `b8a23bf`
**Question asked:** is this plan now safe to implement, and if not, what specifically breaks?
**Scope taken:** the `verifyMismatch` split, P1/P2/P3, over- vs under-warning, the
looping claim, the atomic 5+6+7. No codebase audit, no prose review.

## VERDICT: RED — 1 Critical, 4 Important

---

### C-1 — `verifyMismatch` at `:963`/`:984` fires when NO comparison ran, so P2 and P3 are jointly unsatisfiable on a sequence a shipped test drives today

**Where:** §4.7d (the five-site table), §4.7a (the `sawDisagreement` assignment),
P2 and P3.

**The defect.** §4.7d classifies `gui/multisig_verify.go:963` and `:984` as
"`verifyMultisigLegsPartial` mismatch" and "`verifyMultisigLegs` mismatch" — a
comparison, yes. That is a site-level split applied to a function whose error set
is not site-uniform. `verifyMultisigLegsPartial`
(`gui/multisig_verify.go:386-411`) returns **three structurally different**
failures, and only one of them is a comparison:

| error | what happened | did a comparison run? |
| --- | --- | --- |
| `errVerifyNoLegs` | nothing to compare | unreachable at both sites (`len(legs) >= 1` is guaranteed by the `len(legs)==0` block at `:939`) |
| `errVerifyLegHasNoPlate{Slot}` | `verifyClaimPlate` (`:532-550`) found no read-back plate carrying this leg's xpub — it **returns before `verifyMultisig` is ever called for that leg** | **NO** |
| `verifyMultisig(...)` error | `bundle.Verify` found a diverging field | yes |

`errVerifyLegHasNoPlate` is a **pairing** failure, not a comparison. Nothing was
compared. The plan never names it — `grep -n "LegHasNoPlate\|Unclaimed"` over the
plan returns nothing.

**It is not hypothetical, and it is not rare.** It is the *canonical* driven
instance of `:963` in the tree. Executed just now:

    nix develop --command go test ./gui/ -run 'TestVerifyIncompleteDoesNotCallAForeignPlateChecked' -count=1 -v
    # PASS. final screen:
    # "No read-back key plate carries slot @0's key. Either that plate was not
    #  presented, or it is not the one this run cut. Present every plate this run
    #  engraved, or re-cut slot @0's. Verify Failed"

The device's own operator text says outright that **it cannot tell whether the
plate was merely not presented**. `gui/multisig_verify.go:735` requires only that
the *count* of read-back mk1s equals the expected slots — so an operator who
brings the right number of plates with one stray among them, or who leaves one in
the safe and picks up steel from another run, lands here with a complete, correct
backup sitting in the safe.

Under §4.7d that verdict becomes `verifyMismatch` → `sawDisagreement = true`
(sticky) → the durable restore document prints

> `WARNING: a read-back check DISAGREED with these plates. Do NOT rely on this backup: engrave a fresh set and check it before use.`

**This is R4's C-1, verbatim, on the path the fold left behind.** The fold
cleared `:719` — "the plates belong to a different wallet" — as non-condemning,
and then condemns the *identical physical situation* when the foreign steel is an
mk1 instead of the md1. The same bench, the same mistake, two opposite verdicts.

**The plan contradicts itself, not just the code.** P2 says any sequence
containing a `verifyMismatch` prints `DISAGREED` or the repeat-check line, and
never `DID NOT COMPLETE`. P3 says no sequence prints `DISAGREED` unless a
comparison actually ran and disagreed. On `foreign-plate-substituted → stop`, the
final `res` is not `verifyComplete`, so the repeat-check line is unavailable: P2
demands `DISAGREED`, P3 forbids it. The two properties are **jointly
unsatisfiable on a reachable sequence**, and the sequence is one an executing
test drives today. That is a plan-internal inconsistency, not a judgement call.

Even the recovery path is wrong: the operator who fetches the missing plate and
re-verifies cleanly gets `VERIFIED on a repeat check, after an earlier read-back
DISAGREED` permanently engraved into the document — an accusation earned by
forgetting a plate.

**The harm.** A stranger reading the durable artifact in five years is told, on
page 1, not to rely on a backup that is complete and correct, and to cut a fresh
set. §4.4's own prose names giving-up-on-a-recoverable-backup as the failure mode
this document exists to prevent.

**Suggested remedy (UNVERIFIED).** The split is not site-level; it is
**error**-level. `verifyMultisigLegs`/`Partial` would have to distinguish
"pairing failed" from "the comparator disagreed" — e.g. the two call sites
`errors.As` on `errVerifyLegHasNoPlate` and return `verifyFailed` for it,
`verifyMismatch` otherwise. Marked UNVERIFIED: I did not resolve what that does
to `errVerifyPlateUnclaimed` (see below) or to the two tests in I-2, and this
project's own record says a prescribed fix has closed an Important by opening a
Critical.

**One adjacent fact, checked and reported as sound so it is not re-derived.**
`errVerifyPlateUnclaimed` — reachable only through `verifyMultisigLegs` at `:984`
— is **not** reachable in production: `:735` forces
`len(readbackMk1s) == len(expectedSlots)`, `:984` is guarded by
`len(legs) >= len(expectedSlots)`, legs are a subset of expected slots, and
`verifyClaimPlate` claims a distinct index per leg, so every plate is claimed
whenever every leg claims one. It is exercised only by direct unit tests. It is
worth naming anyway, because its operator text is *"Read-back key plate N belongs
to no slot of this wallet. It is steel from another run or another wallet"* — the
`:719` class again, one `len()` precheck away from being condemned by `:984`.

---

### I-1 — a mistyped ms1 at verify time condemns the plates, which is the exact class the plan exempted at `:897`

**Where:** §4.7d's table row for `:963`/`:984`; P3.

**The defect.** The third error reachable at `:963`/`:984` is `bundle.Verify`'s
ms1 arm, and the ms1 is **hand-typed at verify time** (`multisigVerifyMS1Entry`,
`gui/multisig_verify.go:869`; the file's own comment: "never NFC"). Executed:

    nix develop --command go test ./gui/ -run 'TestVerifyFullModeBindsEachMs1ToItsOwnSeed' -count=1 -v
    # PASS. final screen:
    # "The read-back bundle does NOT match the seed you typed. Details: slot @0:
    #  verify: ms1 entropy mismatch  Check what that names before you re-cut
    #  anything: a hand-typed ms1 is one character away from this message, and
    #  the plates may be perfect. Verify Failed"

`multisigVerifyFailureText` (`:424-470`) says it in the string the operator reads:
**"the plates may be perfect."** §4.7d exempts `:897` on precisely this reasoning
— *"the re-typed seed would not derive — a typo at verify time"* — and then
condemns the sibling case, the other hand-typed secret on the same flow, without
naming it.

Note this is not covered by C-1's remedy: here a comparison genuinely ran, so P3
as literally written is satisfied, and P3 is therefore not the property that
catches it. What fails is §4.7d's own stated goal — *"ordinary operator mistakes
at verify time with nothing wrong with the backup at all"* must not condemn — and
its factual premise, that `:963`/`:984` compare "against this run's plates". The
ms1 arm compares against a **hand transcription** of a plate.

**The harm.** The document's `DISAGREED` line instructs *"engrave a fresh set"*
while the screen the operator just dismissed instructs *"Check what that names
before you re-cut anything"*. The durable artifact contradicts the transient one,
in the direction that destroys a good backup.

**And it propagates to single-sig.** `verifySingleSig`
(`gui/singlesig_verify.go:49-58`) runs the same `bundle.Verify` against the same
hand-typed ms1 (`gui/singlesig_verify.go:123-141`), and exit `:146` is the one
comparison among the eleven. So the build-order step 1 mapping gate — which
§4.7d instructs to "state which of its eleven exits, if any, are genuine
comparisons" — will be performed against a criterion C-1 and I-1 show to be
wrong. *Deferring the eleven exits is structurally safe* (they are ten explicit
`return`s plus the fallthrough, all enumerable, and only `:144-146` calls the
comparator; single-sig has no pairing step, so C-1's leg-has-no-plate class
cannot arise there). What is not safe is deferring it **to the current
definition of "genuine comparison"**.

**Suggested remedy (UNVERIFIED).** Either accept the ms1 arm as condemning and
say so explicitly in §4.7d (with the contradiction against
`multisigVerifyFailureText`'s advice acknowledged and the `DISAGREED` wording
softened off "engrave a fresh set"), or split it out with C-1's error-level test.
Either way the plan must decide it rather than inherit it.

---

### I-2 — §5.1's "must be updated, not weakened" list omits the two existing tests that pin `verifyFailed` on scenarios that become `verifyMismatch`

**Where:** §5.1, §4.8 step 7.

**The defect.** Introducing `verifyMismatch` at `:963`/`:984` breaks two shipped,
currently-passing tests that assert the verdict by name:

| test | line | assertion |
| --- | --- | --- |
| `TestVerifyIncompleteDoesNotCallAForeignPlateChecked` | `gui/multisig_verify_report_test.go:166` | `if res != verifyFailed { t.Errorf(...) }` |
| `TestVerifyFullModeBindsEachMs1ToItsOwnSeed` | `gui/multisig_verify_report_test.go:759` | `if res != verifyFailed { t.Errorf(...) }` |

Both currently PASS (executed above); both drive `:963`; both go red the moment
the split lands. §5.1 declares itself the inventory of existing tests the cycle
disturbs and offers exactly two lists — the six `buildPlateInventoryLines` call
sites and the three census walks — and names neither of these. §4.8 asserts steps
5+6+7 land as **one green commit**; with these two red it does not.

**The harm.** The obvious repair is to relax `res != verifyFailed` to accept
either verdict. That retires the *only executing evidence in the tree* of which
verdict each site returns — which is exactly the evidence that would have caught
C-1 — and it would be done in good faith, because the plan's not-weakened rule
does not reach these rows.

**Suggested remedy (VERIFIED as to the mechanism, UNVERIFIED as to the target
verdict).** §5.1 needs a list (c) naming both tests, with the target verdict
stated per test rather than left to the implementer, and the same
not-weakenable ruling §5.1(b) carries. Which verdict each should assert is
decided by C-1 and I-1, not by this finding.

**One check that came back clean and should not be re-run:** the source
assertion at `gui/multisig_verify_report_test.go:1093` —
`strings.Contains(body, "res != verifyIncomplete && res != verifyFailed")` —
still passes after §4.7d's loop edit, because the new condition contains the old
string as a prefix. It does not break, and it does not need updating.

---

### I-3 — the five status lines collide as substrings, so T9/T12/T13a/T13b as specified have false-PASS paths, including on the mutation T13a exists to catch

**Where:** §4.7a's line table; T9, T12, T13a, T13b in §5.

**The defect.** The five verbatim lines are:

    Plates VERIFIED: each plate was read back and matched.
    Plates VERIFIED on a repeat check, after an earlier read-back DISAGREED. Confirm they restore before relying on this backup.
    Plates NOT VERIFIED. Confirm they restore before relying on this backup.
    Plate verification DID NOT COMPLETE. Confirm they restore before relying on this backup.
    WARNING: a read-back check DISAGREED with these plates. Do NOT rely on this backup: engrave a fresh set and check it before use.

`"VERIFIED"` is a substring of **three** of them, including `NOT VERIFIED`.
`"DISAGREED"` is a substring of **two** — the warning and the repeat-check pass.
The plan specifies these tests by outcome name ("prints `VERIFIED`", "prints
`DISAGREED`", "prints **bare** `VERIFIED`") and never specifies a needle, while
being scrupulous about needles elsewhere (§5.1b's "the `Card 1 of 3` /
`Card 1 of 2` distinction is not weakenable").

Concretely, with the obvious `strings.Contains` needles:

- **T13a is defeated by its own named mutation.** T13a asserts every
  final-`verifyComplete` sequence "prints `VERIFIED`". A mutation that never
  assigns `status` leaves the zero value → `Plates NOT VERIFIED` → contains
  `VERIFIED` → **T13a passes.** That is a document vouching-by-silence, which is
  the whole Critical.
- **T13b cannot see "always print the repeat-check line".** It asserts a mismatch
  sequence "prints `DISAGREED` or the repeat-check line"; both contain
  `DISAGREED`, so the two arms are indistinguishable and a design that always
  prints the softer arm passes.
- **T12 has no way to express "bare `VERIFIED`"** — its stated assertion — since
  the repeat-check line contains that string too.
- **T9's "exactly one status line per rendered document"** is uncountable under
  these needles: the `NOT VERIFIED` line matches both a `NOT VERIFIED` needle and
  a `VERIFIED` needle, so a naive count is 2 and a loose count is vacuous.

**The harm.** These four rows are the tests that carry C-1's remedy. A false PASS
here is a green suite over a document that vouches for unverified plates — the
9-of-17 false-green pattern §5's own preamble cites from S5.

**Suggested remedy (UNVERIFIED).** Assert on the **whole status line, exact
match at slice index 0** rather than on a substring, or give each row an
explicitly disambiguating needle pair (e.g. `Plates VERIFIED:` with
`!Contains("repeat check")`). Naming the needle in the plan is what makes it
reviewable; leaving it to the implementer is what produced the collision.

---

### I-4 — nothing in the test plan pins the site → verdict mapping, and T15's `:724` row is not drivable through the flow

**Where:** T15, §5's "must pin a PRODUCTION CALL SITE" paragraph.

**The defect, part one.** T15 claims to assert that *"each of the three
non-comparison `verifyFailed` sites (`:719` foreign plates, `:724` undecodable,
`:897` seed typo) yields `DID NOT COMPLETE`"*, and names as its mutation "set
`sawDisagreement` on `verifyFailed` instead of `verifyMismatch`". That mutation is
satisfiable at the **stub** seam (`s5StubVerifyFn`,
`gui/multisig_engrave_tail_walk_test.go:98`, which scripts verdicts one per call
and replaces the verdict source only). A stub-level T15 proves the
**verdict → line** mapping and says nothing about the **site → verdict**
mapping — which is the mapping §4.7d actually introduces, and the one C-1 and
I-1 show to be wrong. §5's own standard ("T9–T14 must pin a PRODUCTION CALL SITE,
not just the pure functions") is stated and then not applied to the row that most
needs it.

**The defect, part two.** `:724` cannot be reached through the flow. `:719` runs
first and returns unless `slices.Equal(readbackMd1, engravedMd1)`; so by the time
control reaches `md.ExpandWalletPolicyChunks(readbackMd1)` at `:722`, the readback
is **byte-identical to what this run engraved**. Reaching `:724` therefore
requires the device to have engraved an md1 it cannot itself expand. On the
supply path that is impossible by construction (the same chunks were expanded at
step 2 of `supplyMultisigPolicyFlow`); on the build path `engraveMd1 =
assembledMd1` whenever the verify runs at all (`gui/multisig_build.go:331,353`
with the verify gated on `!template` at `:444`), and the same expansion is
performed at `:467`. A T15 row naming `:724` as a drivable site is unsatisfiable.

**Consequence for the plan's own gloss.** §4.7d describes `:724` as "the readback
would not decode", which reads it as a readback problem. It is not one — the
readback has already been proved equal to the engraved bytes. This is a wrong
reading of a real line, the blind spot §6.2 declares the cite gate leaves open.
The *verdict* chosen for it (non-condemning) is right, and for a stronger reason
than the plan gives; it is the test row and the description that are wrong.

**The harm.** The implementer writes a T15 that cannot fail against the defect it
names, and the plan closes GREEN with its central classification pinned by
nothing. This is the mechanism by which C-1 and I-1 reach the machine.

**Suggested remedy (UNVERIFIED).** Drop `:724` from T15's row; drive `:719`
through the real flow (a foreign md1 readback is already fixture-buildable —
`TestVerifyIncompleteDoesNotCallAForeignPlateChecked` builds the mk1 half); and
add the row this plan is missing entirely — **drive `:963` to
`errVerifyLegHasNoPlate` and assert the verdict**, which is the assertion that
would have failed against §4.7d as written.

---

## THE FIVE SITES, RE-CLASSIFIED

Read site-by-site, then error-by-error inside the two the plan calls comparisons.

| site | code | what actually reaches it | comparison ran? | plan says | correct? |
| --- | --- | --- | --- | --- | --- |
| `:719` | `!slices.Equal(readbackMd1, engravedMd1)` | foreign md1 on the bench, **or a genuinely garbled md1 plate** | no | no → `verifyFailed` | **yes** (see note) |
| `:724` | `ExpandWalletPolicyChunks` err | readback is byte-identical to the engraved md1 by `:719`; requires a device-internal inconsistency | no | no → `verifyFailed` | verdict yes; **description wrong**, site not drivable (I-4) |
| `:897` | `deriveMultisigLeg` err | invalid re-typed seed, `deriveAccountXpub`, `FormAwareStubChunks(readbackMd1)`, `mk.Encode`, `EncodeMS1` — none plate-derived (`readbackMd1 == engravedMd1` here) | no | no → `verifyFailed` | **yes** |
| `:963` | `verifyMultisigLegsPartial` err | **three** errors, see below | **mixed** | yes → `verifyMismatch` | **NO — C-1 / I-1** |
| `:984` | `verifyMultisigLegs` err | same three, plus `errVerifyPlateUnclaimed` (unreachable in production) | **mixed** | yes → `verifyMismatch` | **NO — C-1 / I-1** |

Inside `:963` / `:984`:

| error | comparison? | correct verdict |
| --- | --- | --- |
| `errVerifyNoLegs` | n/a | unreachable at both sites |
| `errVerifyLegHasNoPlate{Slot}` | **no** — `verifyClaimPlate` returns before `verifyMultisig` is called | `verifyFailed` (C-1) |
| `bundle.Verify` diverging field, **ms1 arm** | yes, against a hand transcription | undecided by the plan (I-1) |
| `bundle.Verify` diverging field, mk1/md1/origin/fingerprint arms | yes, against NFC-read plate bytes | `verifyMismatch` — the only unambiguously correct case |
| `errVerifyPlateUnclaimed{Plate}` | **no** — foreign steel on the bench, the `:719` class | unreachable in production; would be `verifyFailed` |

**Note on `:719` and the under-warn question the brief raised.** A genuinely
miscut md1 plate does reach `:719`, and the document then prints only
`Plate verification DID NOT COMPLETE. Confirm they restore before relying on this
backup.` I checked whether that is an under-warn and concluded it is **not a
finding**: the line is non-vouching, it carries an actionable instruction, and the
operator has already seen the foreign-policy modal. The same trade applies to
`errVerifyLegHasNoPlate` under C-1's remedy (a garbled mk1 fails `mk.Decode` at
`:541` and is skipped in claiming, so it surfaces there). What the plan does not
do is *state* that trade: §4.7d asserts the three cleared sites are "ordinary
operator mistakes with nothing wrong with the backup at all", which is true of
`:897` and overclaims for `:719`. Recorded as a **Minor** — the verdict is right,
the certainty in the table's "what it is" column is not.

**No sixth path found.** I checked for a caller mapping a comparison failure onto
a different verdict: the derive-loop refusal arms (`:790-855`) return
`verifyIncomplete`/`verifyAbandoned` via `correctable`, and no comparator call
precedes them (`verifyMultisigLegsPartial` is first reached at `:963`, after the
`len(legs)==0` block). The `:963` **success** path returns `verifyIncomplete` over
a comparison that ran and *agreed*, which is correct. There is no other producer
of `verifyFailed` in the tree (`grep -rn verifyFailed --include=*.go gui/`, five
non-test return sites, all enumerated above).

---

## P1 / P2 / P3 — CONFLICT CHECK

**As properties over a correctly-classified verdict, the three are consistent.** I
executed §4.7a's switch against all twelve rows rather than arguing it:

- **P1** — the five rows with final `res == verifyComplete` (`complete`,
  `incomplete→complete`, `mismatch→complete`, `incomplete→mismatch→complete`,
  `failed→complete`) print `VERIFIED` or the repeat-check line. Never
  `DID NOT COMPLETE`, never `DISAGREED`. **Holds.**
- **P2** — the five rows containing a `mismatch` print `DISAGREED` (three) or the
  repeat-check line (two). Never bare `VERIFIED`, never `DID NOT COMPLETE`.
  **Holds.**
- **P3** — every `DISAGREED` row has `sawDisagreement` set, and
  `sawDisagreement` is set only by `verifyMismatch`. **Holds, conditional on
  `verifyMismatch` ⟺ a comparison ran and disagreed.**

There is **no sequence where P3 forces silence that P2 forbids**, and none where
P1 and P3 pull apart: P1's antecedent is the final verdict, P2's and P3's are the
history, and the switch reads exactly those two variables. The `failed → complete
→ VERIFIED` row that the fold added is right — nothing was ever disagreed with —
and I found no under-warn in it.

**The conflict is not between the properties; it is between P2/P3 and the
verdict.** P3's conditional is the load-bearing clause, and C-1 shows it false:
`errVerifyLegHasNoPlate` returns `verifyMismatch` with no comparison run. On the
sequence `foreign-plate-substituted → stop` — driven today by
`TestVerifyIncompleteDoesNotCallAForeignPlateChecked` — the final `res` is not
`verifyComplete`, so the repeat-check escape is closed, and P2 demands
`DISAGREED` while P3 forbids it. **Jointly unsatisfiable on a reachable
sequence.** Fixing the classification (C-1) restores consistency; weakening
either property does not.

Secondary: P1 and P2 are also unfalsifiable as *specified in the tests* because of
the needle collisions (I-3). Three properties written and none of them pinned is
the same shape as R4's "having two properties read as rigour".

---

## WHAT I CHECKED AND FOUND SOUND

- **The looping claim is true, and unchanged.** `gui/multisig.go:337` and
  `gui/multisig_build.go:453` both read
  `if res != verifyIncomplete && res != verifyFailed { break }`. All five sites
  loop today; adding `&& res != verifyMismatch` preserves that exactly, in both
  loops. I also confirmed the source-assertion test at
  `gui/multisig_verify_report_test.go:1093` still matches the edited condition
  (prefix substring), so it neither breaks nor needs weakening.
- **The `sawDisagreement` / switch structure is correct.** No ordering, no `max`,
  no seed; `status` is assigned only inside the loop body so Skip leaves the safe
  zero value; both sticky facts have safe zero values. R3 C-1 and R3 C-2 are
  structurally unreachable, as claimed.
- **`verifyStatus`'s zero value and the §4.7c argument** — including the honest
  correction that the zero value protects the variable, not the document, and that
  the document-level guarantee must be carried by a rendered-document test.
- **`:897` is genuinely non-plate.** `deriveMultisigLeg`
  (`gui/multisig_derive.go:32-72`) can fail on the mnemonic, the account
  derivation, `FormAwareStubChunks`, `mk.Encode` or `EncodeMS1`; its only
  plate-shaped input is `readbackMd1`, already proved equal to `engravedMd1` at
  `:719`. The plan's classification is right.
- **Deferring the single-sig eleven-exit mapping is structurally safe.** I read
  all eleven (`gui/singlesig_verify.go` returns at `:69, :78, :90, :98, :112,
  :117, :125, :130, :138, :146`, plus the fallthrough at `:149`). Exactly one —
  `:146` — runs a comparator; single-sig has no pairing step, so C-1's class
  cannot arise there. Step 1 being a reviewed gate is adequate *process*; the
  caveat is I-1's, that the criterion it applies is currently wrong.
- **The atomic 5+6+7 does eliminate the bad intermediate state at commit
  granularity**, which is the only granularity that matters. I checked the
  earlier steps for the same harm and found none: step 4 lands the `status`
  parameter with `nil` at the call sites, which renders documents with no status
  line — but that is today's behaviour on multisig and step 5 has not yet added
  the single-sig inventory, so no commit before 5+6+7 carries "full inventory +
  completeness claim + no status line". The reasoning in §4.8 is sound; the only
  thing that breaks the claim is I-2's two red tests inside that commit.
- **The call-site counting is internally consistent.** `buildPlateInventoryLines`
  has 8 existing call sites (2 production + 6 test, measured), which is §4.3's
  "all 8"; §5.1(a)'s "six" is the test subset, all `seedCapacityMany`; §8's
  "nine" is 8 + the new single-sig site. No drift.
- **The verdict constant insertion is safe.** `multisigVerifyResult` has no
  `String()` method, no index-keyed table, and is never persisted — inserting
  `verifyMismatch` shifts iota values harmlessly. The only consumers are the two
  loop conditions and the tests in I-2.
- **T13a/T13b/T15 are drivable in principle.** `s5StubVerifyFn`
  (`gui/multisig_engrave_tail_walk_test.go:98`) scripts verdicts one per call with
  the last repeating, and replaces the verdict source only — the offer screens,
  the row mapping, the loop condition and the retry lead stay production. The
  sequence table is testable; the problems are the needles (I-3) and the missing
  site-level row (I-4), not the harness.

## MINOR

- **M-1.** §4.7d's "what it is" column overclaims certainty on `:719`: a
  genuinely garbled md1 plate reaches it too, so "the plates belong to a different
  wallet" is one of two readings, not the reading. The verdict is unaffected.
- **M-2.** §4.7d's gloss on `:724` ("the readback would not decode") misreads the
  line — the readback is byte-identical to the engraved md1 by that point. See
  I-4.

---

*Reviewer note on scope: I did not re-derive the five-site citations, the
twelve-row table, the glyph gate or the cite gate, all of which the brief stated
were machine-verified. Every finding above is a reading of a real line, and the
two central ones were reproduced by executing shipped tests rather than by
argument.*
