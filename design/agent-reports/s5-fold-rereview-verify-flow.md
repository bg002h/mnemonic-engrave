# S5 fold re-review — LENS: THE VERIFY (commit `9f93362`)

**Artifact:** `git diff 7da66bd..830aaf7` on branch `s5-multislot`
(`/scratch/code/shibboleth/wt-s5`, confirmed frozen and clean at start **and** at
verdict: `git status --porcelain --branch` → `## s5-multislot` only,
`git rev-parse HEAD` → `830aaf7310b4d8870a5dd893d818afa625699e04`).
**Question asked:** (1) did the fold CLOSE each finding it claims to close, or MOVE it;
(2) did the fold INTRODUCE a new defect. Scoped to the verify/report path.
**Every probe below ran in a `cp -a` copy** at
`/scratch/code/shibboleth/me-review-scratch/s5rr/wt`. The frozen tree was never written to.
**Date:** 2026-08-16.

---

## VERDICT: **3 Important. 0 Critical.**

The verify **does** now fail closed in every direction I attacked. C-1 is genuinely
closed: the number on the incomplete screen is produced by the comparator, and a
foreign plate among the presented set is a `Verify Failed`, not an `Incomplete`.
I-2, I-3, I-11, I-12, I-13 and I-14 are closed with assertions that assert.

**I-4 is the one that was MOVED rather than closed.** The fold built the retry
mechanism the review asked for, then wired it to two of the flow's five verdicts,
left the instruction on the incomplete screen unsatisfiable, and pinned the whole
thing with `strings.Contains` over the caller's source text — so a realistic edit
that inverts the two retry rows leaves the entire suite green.

| | count |
| --- | --- |
| Critical | **0** |
| Important | **3** |
| Minor / Nit | 2 (recorded, not blocking) |

---

# What I confirmed CLOSED (do not re-open)

**C-1 — closed, and closed at the right level.** `verifyMultisigLegsPartial`
(`gui/multisig_verify.go:386`) is called from the incomplete branch at `:869`
*before* the report is drawn; only the reverse "every plate must be claimed" sweep
is skipped. Two legs never claim one plate (`claimed[idx] = true` at `:396`, and
`verifyClaimPlate` skips claimed plates at `:519`), so `len(legs)` in
`multisigVerifyIncompleteText` is now exactly the number of plates the comparator
matched and passed through `bundle.Verify`. I attacked it five ways and it fails
closed each time:

* **foreign mk1 at the right count** → leg @0 finds no plate →
  `errVerifyLegHasNoPlate{0}` → `Verify Failed` naming @0 (their
  `TestVerifyIncompleteDoesNotCallAForeignPlateChecked` reproduces this; I re-derived
  the path by reading `:869-872`).
* **stops early, honest plates** → `Verify Incomplete`, measured verbatim in my own
  probe: `"Checked1keyplate:@2.Comparedagainsttheplatesyoupresented,andtheymatch.
  2slotsareNOTverified:@0and@1."`
* **duplicate plate presented** (@0's plate twice + @1's, count 3 = expected 3) → the
  two legs claim two *distinct* plates and report truthfully; typing the second master
  then drives leg @2 into `errVerifyLegHasNoPlate{2}` → `Verify Failed`.
* **right count of the wrong plates** → same, `Verify Failed`.
* **aborts mid-engrave then verifies** → unreachable: `gui/multisig.go:291` and
  `gui/multisig_build.go:402` return on `!= bundleEngraveDone`.

**I-13 — closed.** `multisigVerifyOKMessage(1, false)` now returns
`"Operator key verified. ..."`; the table test's predicate is `"secret" OR "ms1"`,
which is the correct predicate (the multi-leg full string claims the secret without
using the word). Watch-only never claims a secret on any of the four surfaces.

**I-14 — closed.** Neither arm of `multisigVerifyCoveredSeedBody` asserts a foreign
seed; `multisigVerifySeedIsInnocent` is wired into both.

**I-3 / I-12 / I-11 / I-2 — closed** (I-12 with a real flow-level test that asserts
the program *ends*). No new message in the diff asserts a comparison that did not
happen: I read every string the fold added and each claim is gated on the code path
that produces it.

---

# IMPORTANT (3)

## V-1 — I-4's retry reaches 2 of the 5 verdicts; every screen that tells the operator to "try again" on the FIRST seed dead-ends into the restore document

**Defect (one sentence).** `multisigVerifyFlow` returns `verifyAbandoned` /
`verifyRefused` from four screens that explicitly prescribe a retry, and both callers
gate the re-offer on `res != verifyIncomplete && res != verifyFailed`, so the operator
is told "add it and try again" and the next screen is the restore document.

**Site.** `gui/multisig.go:333` and `gui/multisig_build.go:449` (the gate), against
`gui/multisig_verify.go:656` (`verifyRefused` after "Read back one wallet-policy md1
AND the operator key card(s) (mk1).") and `:790` / `:809` (the two `break`s that fall
to `:847-849`, `if len(legs) == 0 { return verifyAbandoned }`).

**Triggering state (measured, two of them).**

1. **The commonest full-mode failure there is.** A single-held-slot FULL build.
   Operator chooses *Verify now*, presents the plates, re-types the seed, then
   hand-types the 48-character ms1 and gets one character wrong.
   `multisigVerifyMS1Entry` (`:914-918`) shows *"That isn't a valid ms1 secret share."*
   and returns `("", false)`; the `break` at `:809` falls to `:847`; `legs` is still
   empty because the ms1 is asked for **before** the derive loop, so the flow returns
   `verifyAbandoned`. The caller does not re-offer. The next screen is
   *"This backup is N plates…"*. I drove the byte-identical exit (Back at the first
   seed's ms1, same `break`, same `:847`) and measured the verdict and the frame:

   ```
   $ nix develop --command go test ./gui/ -run TestProbeBackAtTheFIRSTMs1IsADeadEnd -count=1 -v
   last frame after Back at the first ms1: ""
   verdict = 4 (complete=0 incomplete=1 failed=2 refused=3 abandoned=4)
   ```

   The flow leaves **no screen at all** — the exact shape I-2's M19 mutation was filed
   for, kept deliberately on the first seed (`:800-808` says so), and now load-bearing
   because it is also the branch the retry excludes.

2. **F-191's own screen.** Trace B, watch-only, the operator types a seed that fills
   no slot (or the right seed with the wrong/forgotten passphrase):

   ```
   $ nix develop --command go test ./gui/ -run TestProbeFirstSeedNoSlotIsADeadEnd -count=1 -v
   screen after the foreign seed:
     "Noslotmatchesthatseed.IfthiswalletwasbuiltwithaBIP-39passphrase,
      additandtryagain:withoutitthesamewordsderiveadifferentwallet.VerifyBundle"
   flow verdict = 4   (verifyAbandoned)
   ```

   The device says *"add it and try again"* and then removes every route to trying
   again. The sibling arm `multisigVerifyNoSlotBody(true, true)` — the one F-191 exists
   to produce — ends *"Your plates are fine. Try again and skip the passphrase."* on the
   same `break`, and so does `multisigVerifyCoveredSeedBody(_, true)`.

**Why Important.** This is I-4's finding verbatim ("prescribes a remedy that has no
implementation… on the screen that precedes funding"), surviving at four sites the fold
did not wire. The fold's stated rationale for excluding them —
`gui/multisig_verify.go:78-84`, *"a structural refusal … says nothing the operator can
fix by trying again with the same inputs"* — is refuted by the screens' own text: they
tell the operator to try again with **different** inputs (the passphrase, the ms1, the
plates). On a one-slot build, `expectedSlots` has one element, so *every* first-seed
refusal is this branch: it is the majority case, not an edge.

**Fix (resolved against the real call graph before proposing it).** `verifyAbandoned` is
returned from three places: `:653` (Back at the gather — a genuine abandon, leave it),
`:790` and `:809`. Split the latter two from the gather's abandon — e.g. return
`verifyIncomplete` from `:847` when the operator reached at least one *refusal screen*,
or add a fifth verdict `verifyRetryable` — and add `verifyRefused` from `:656` to the
caller's retry set. Both callers' condition is a single line each
(`gui/multisig.go:333`, `gui/multisig_build.go:449`). Note `:624` and `:634`
(`errVerifyNoExpectedSlots` / no engraved md1) must **stay** non-retryable: those really
cannot change between attempts.

---

## V-2 — the new incomplete screen instructs "VERIFY AGAIN … and type the remaining seed", and doing exactly that reports the plates already checked as NOT verified

**Defect (one sentence).** `multisigVerifyFlow` keeps no state across attempts, so the
retry the fold built cannot be driven to `Verify OK` by the sentence the fold wrote:
each attempt starts from zero legs, and the operator who obeys it literally sees the
previously-verified slots reported as unproved.

**Site.** `gui/multisig_verify.go:466-467` (the new instruction), against `:697-712`
(`legs`, `covered` and `typed` are all locals of the call).

**Triggering state (measured).** Trace B, three engraved slots, two masters. Attempt 1:
type master A, STOP HERE → *"Checked 2 key plates: @0 and @1 … 1 slot is NOT verified:
@2 … Choose VERIFY AGAIN on the next screen and type the remaining seed."* Attempt 2,
following that instruction exactly — type only master B:

```
$ nix develop --command go test ./gui/ -run TestProbeSecondAttemptWithOnlyTheRemainingSeed -count=1 -v
second-attempt final screen:
  "Checked1keyplate:@2.Comparedagainsttheplatesyoupresented,andtheymatch.
   2slotsareNOTverified:@0and@1.Nothinghasbeenprovedaboutthoseplates.
   ChooseVERIFYAGAINonthenextscreenandtypetheremainingseed,ordonotfundthiswalletuntilyouhave."
verdict = 1 (incomplete)
```

The operator is now told @0 and @1 are unproved, having watched the device prove them a
minute earlier, and is given the **same** instruction again. There is no sequence of
obedient actions that reaches `Verify OK`; the only route is to type *every* seed in one
attempt, which no screen says. On a 3-of-4 the predictable end state is an operator who
concludes the device is unreliable and funds the wallet — the outcome I-4 was filed to
prevent.

**Why Important, not Minor.** The sentence is newly authored in this fold and sits on the
last screen before funding, and its prescribed action produces a *worse* report than
doing nothing. "Nothing has been proved about those plates" is true within the attempt
and false as the operator reads it.

**Fix.** Either (a) carry `covered` (and the verified slot set) across retries — pass it
in and out of `multisigVerifyFlow`, so a second attempt accumulates; or (b) the cheap
correct one: change the sentence to say what the device does —
*"Choose VERIFY AGAIN and type **all** the remaining seeds in one go; a new attempt
starts from nothing."* (a) is the better artifact; (b) closes the false prescription.

---

## V-3 — I-4's entire mechanism is pinned only by `strings.Contains` over the caller's source; swapping the two retry rows leaves the whole suite GREEN

**Defect (one sentence).** No test executes a second verify: `TestBothEngraveFlowsReOfferTheVerify`
(`gui/multisig_verify_report_test.go:733-765`) reads the two functions' source text and
asserts on substrings, so the loop's *behaviour* — including which row re-verifies and
which exits — is unobserved.

**Site.** `gui/multisig_verify_report_test.go:733`, against `gui/multisig.go:325-338` and
`gui/multisig_build.go:441-454`.

**Triggering state (mutation, run).** The two retry rows are reordered — a one-token edit
of the kind a wording pass makes — so that **VERIFY AGAIN exits the loop and CONTINUE
re-runs the verify**:

```
$ perl -0pi -e 's/choices = \[\]string\{"VERIFY AGAIN", "CONTINUE"\}/choices = []string{"CONTINUE", "VERIFY AGAIN"}/g' gui/multisig.go gui/multisig_build.go
subs=2
$ nix develop --command go test ./... -count=1
EXIT=0        51 ok, 0 FAIL
```

Every source-text assertion still passes: `res := multisigVerifyFlow(...)` is present,
`res != verifyIncomplete && res != verifyFailed` is present, `multisigVerifyRetryLead` is
present. The mutation was restored (`git status --porcelain` → only my probe files) and
the copy re-verified.

Corroborating measurement: `grep -rn "VERIFY AGAIN\|verifyAbandoned\|verifyRefused" gui/*_test.go`
returns **nothing** outside `multisigVerifyRetryLead`'s own literal — three of the five
verdicts the fold introduced are asserted by no test at all, and the retry offer's rows
are named by no test.

**Why Important.** Per the project's own closure rule, *a gate that has never executed is
a hypothesis, not a gate* — and this gate is the whole of I-4's fix, on the screen that
precedes funding. It is also the same defect class round 0 filed as I-2 (the `full` half
driven by nothing) one level up: the fix landed as text rather than as behaviour.

**Fix.** One flow-level driver per path: reach the verify offer, take *Verify now*, drive
an incomplete verify, assert the screen reads `multisigVerifyRetryLead` with **VERIFY
AGAIN as row 0**, press it, and assert the gatherer is reached a second time. (On a
reader-equipped platform this is honest; note `bundleGatherFlow` nils
`ctx.syswBundleSeeds` at `gui/bundle_flow.go:167`, so a payload-fed emulator retry has an
empty accumulator — the test must feed the second gather, and that seam is worth an
explicit comment.)

---

# Minor / Nit (recorded, not blocking)

| # | Sev | Item | Site |
| --- | --- | --- | --- |
| m-1 | Minor | `s5DriveVerifyTwoSeeds`'s refusal-needle list still carries `"cannot prove any of these plates"`, a string that exists nowhere in the tree (`grep -rn` → one hit, the needle itself). A dead needle in a dismissal list is how a driver silently stops dismissing. | `gui/multisig_verify_policy_test.go:224` |
| m-2 | Nit | `multisigVerifyIncompleteText`'s *"Compared against the plates you presented, and they match"* is a claim about **all** presented plates while only `len(legs)` of them were compared; the following sentence ("Nothing has been proved about those plates") repairs it, which is why this is a Nit and not a finding. | `gui/multisig_verify.go:464` |

---

# Out of scope, one line each (NOT investigated)

* `gui/singlesig.go:127` ignores `bundleEngrave`'s new result and `:80` hard-codes
  `"Full (seed + keys)"` — **already filed as F-197 / F-198**, not re-reported.
* `errVerifyPlateUnclaimed` still cannot fire in production (the `:688` count precheck
  plus `len(legs) == len(expectedSlots)` make the reverse sweep vacuous) — this is round
  0's Minor **M-8**, unchanged by the fold, and the fold added a `multisigVerifyFailureText`
  arm for it that is therefore unreachable outside its unit test.
* On a machine without `FeatureNFC` the verify's gather has no source at all
  (`ctx.syswBundleSeeds` was consumed by the flow's earlier gather at
  `gui/bundle_flow.go:167`, and the verify deliberately makes no payload offer) — a
  pre-existing property of the shipped verify, outside this diff.

---

# Settled facts I did not re-derive

The machine baseline (`go test` exit 0 / 51 ok, `gofmt` clean, cold `go vet` exit 1 with
40 test-only findings, oracle-live 7/7, `emu.wasm` 9972075 bytes), R-1's refutation, the
I-8 (b) ruling, and the I-6 / I-13 departures. R-1 stays refuted: the frozen tree was
clean at start and at verdict, and every probe ran in a `cp -a` copy.
