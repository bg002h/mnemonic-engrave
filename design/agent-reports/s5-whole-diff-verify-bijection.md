# S5 whole-diff review — LENS: the per-leg bijection in the verify

Agent: independent adversarial reviewer (opus), read-only on `/scratch/code/shibboleth/wt-s5` @ `7da66bd`.
Diff under review: `git diff main..s5-multislot`.
Date: 2026-08-16.

**Verdict on the lens question: the plate↔leg matching IS a true bijection and it DOES fail
closed. I could not construct a false GREEN.** What I found instead are three defects around
it: the half of the flow that puts a SEED on steel is never executed by any test, the
comparator's diagnosis is thrown away before it reaches the operator, and the screen that
reports a partial verify instructs a remedy the device cannot perform.

0 Critical / 3 Important / 2 Minor.

---

## Method, and what I actually ran

Everything below was executed under `nix develop --command`, unpiped, on the frozen tree.

```
export PATH="/nix/var/nix/profiles/default/bin:$PATH"
cd /scratch/code/shibboleth/wt-s5
export GOCACHE=$(mktemp -d)
nix develop --command go test ./gui/ -count=1 -coverprofile=<scratch>/gui.cov
  -> ok  seedhammer.com/gui  67.474s  coverage: 83.4% of statements
```

Then the uncovered-block extraction that Findings 1 and 5 rest on:

```
grep "multisig_verify.go" <scratch>/gui.cov | awk -F'[:,. ]+' '$NF==0 {print}'
```

I did not re-derive the build/format/vet/oracle/emu results stated as settled in the brief.

---

## The bijection, direction by direction (the lens, answered)

Read: `gui/multisig_verify.go` (whole file), `gui/multisig_verify_legs_test.go`,
`gui/multisig_verify_flow_test.go`, `gui/multisig_verify_policy_test.go`,
`gui/multisig_supply_dupslot_test.go`, `gui/multisig_supply_tail.go`,
`gui/multisig_build_tail.go`, `gui/multisig_match.go`, `gui/multisig_supply.go`,
`bundle/verify.go`, `gui/bundle.go` (`offer`/`offerChunkedMK1`).

**The structural fact that makes it a bijection, which the code never states outright:** at
the call to `verifyMultisigLegs` (multisig_verify.go:664) the three cardinalities are already
forced equal.

* `len(readbackMk1s) == len(expectedSlots)` — the precheck at :493 returns otherwise.
* `len(legs) <= len(expectedSlots)` — every leg comes from `fresh ⊆ expectedSlots` minus
  `covered` (:271-283, :614-623), so legs are one-per-distinct-expected-slot and can never
  exceed. `expectedSlots` itself is duplicate-free at both producers:
  `buildEngraveTail` ranges over `for slot, s := range sources` (multisig_build_tail.go:86,
  the index), `supplyEngraveTail` ranges over `allUserSlots`' ascending distinct indices
  (multisig_supply_tail.go:129).
* `len(legs) < len(expectedSlots)` is diverted to "Verify Incomplete" at :655.

So `len(legs) == len(mk1s)`, each leg claims a distinct unclaimed plate or the whole verify
fails, and a perfect matching on equal-size sets leaves nothing over. Both directions hold.

| Pathology from the brief | Behaviour | Where |
| --- | --- | --- |
| unmatched LEG | `errVerifyLegHasNoPlate{Slot}` → Verify Failed | :309-312 |
| unmatched PLATE | precheck refuses before a seed is asked for; sweep as backstop | :493, :318-322 |
| two plates → one leg | greedy first-match on xpub; unreachable today — **Finding 4** | :329-347 |
| one plate → two legs | impossible: `claimed[]` is set before the compare | :313 |
| leg matched by another run's plate | md1 byte-equality gate at :478 runs BEFORE the decode, plus `checkStubBinding("readback")` and the fp/xpub/path compares in `bundle.Verify` | :478, bundle/verify.go:34-60 |
| duplicate identical plates presented | the gatherer keys mk1 cards on `csid` and returns `bundleDuplicate` (gui/bundle.go:178), so the second never enters; the precheck then fires | gui/bundle.go:177-180 |
| unexpected ORDER | pairing is by xpub, which is order-free; pinned by `TestVerifyPairsByKeyNotByOrigin` driving the honest set REVERSED | legs_test:316-357 |
| MORE plates than engraved | precheck | :493 |
| FEWER, stopping early | precheck (short readback) or "Verify Incomplete"; never OK | :493, :655 |
| `expected ∩ allUserSlots` EMPTY | three-way refusal screen, then `break` → Incomplete, or silent return at zero legs | :540-591, :647 |
| `expected ∩ allUserSlots` a strict SUBSET | loop offers the next seed; declining → Incomplete | :624-634 |

**A partial match is never reported OK.** I traced every exit from the derive loop: the only
route to `showNotice(multisigVerifyOKTitle, ...)` passes `len(legs) == len(expectedSlots)`
and a nil return from `verifyMultisigLegs`.

### On the "fix the reused-key regression WHERE IT IS" commit (f0006b7)

The brief asked whether that fix relaxed coverage somewhere less visible. **It did not, and
the mechanism it added no longer exists.** `git log -S"verifyLegWithSameKey" main..s5-multislot`
returns two commits: `f0006b7` added it, `070686a` removed it. The reused-key shape it was
aimed at is now handled at the ENGRAVE, and the two halves collapse **jointly**:
multisig_supply_tail.go:152-158 appends to `slots` only inside the branch that appends to
`mk1s`, so the obligation list can never name steel that was deduped away. Verified against
the shape itself: `TestSupplyDuplicateSlotVerifiesItsOwnOutput` drives the real gatherer over
a 2-of-2 seating one key twice and reaches Verify OK. The BUILD path needs no such dedupe —
`duplicateSlotPair` (multisig_build.go:1024, sole production caller at :1278, inside
`assembleBuildPolicy`) refuses any two slots with an identical `cc‖pk`, so two held slots can
never mint byte-identical mk1s. No relaxation found.

---

# FINDINGS

## Important 1 — the `full` half of `multisigVerifyFlow` is executed by NO test, including the per-leg ms1 binding the design calls load-bearing

**File:** `gui/multisig_verify.go:596-612`, `:677-694`, `:718-721`
**Category:** test-coverage / unpinned mechanism

Every one of the five test call sites passes `full=false`:

```
$ grep -rn "multisigVerifyFlow(ctx" gui/
gui/multisig_verify_flow_test.go:114:  multisigVerifyFlow(ctx, &descriptorTheme, false, expected, engravedMd1)
gui/multisig_verify_flow_test.go:220:  multisigVerifyFlow(ctx, &descriptorTheme, false, nil, md1)
gui/multisig_verify_flow_test.go:246:  multisigVerifyFlow(ctx, &descriptorTheme, false, []int{slot}, nil)
gui/multisig_supply_multislot_test.go:271: multisigVerifyFlow(ctx, &descriptorTheme, false, expected, engravedMd1)
gui/multisig_verify_policy_test.go:177:   multisigVerifyFlow(ctx, &descriptorTheme, false, expected, engravedMd1)
```

`multisigVerifyMS1Entry` has exactly one reference outside its own definition and doc
comment — the production call at :598. The coverage profile confirms zero executions:

```
multisig_verify.go:597.11,599.11  2 0   <- if full { s, ok := multisigVerifyMS1Entry(...)
multisig_verify.go:599.11,609.10  1 0   <- the I-2 "BREAK, NOT RETURN" on an ms1 Back
multisig_verify.go:611.4,611.19   1 0   <- ms1Readback = s
multisig_verify.go:677.70,679.9   2 0   |
multisig_verify.go:679.9,681.3    1 0   |
multisig_verify.go:682.2,683.12   2 0   |  the entire body of
multisig_verify.go:683.12,686.3   2 0   |  multisigVerifyMS1Entry
multisig_verify.go:687.2,688.16   2 0   |
multisig_verify.go:688.16,691.3   2 0   |
multisig_verify.go:692.2,693.25   2 0   |
multisig_verify.go:718.10,721.3   1 0   <- the full-mode multi-leg success string
```

`full=true` is the mode that cuts the SEED plate. What is unexecuted there is not incidental:

* **The per-leg ms1 binding.** `verifyLeg.MS1Readback`'s own doc (:161-168) says the ms1
  travels WITH the leg rather than being a flow-global *because* a flow-global is how "a
  'Full' backup carrying master A twice verifies clean while master B — which k=3 needs — is
  gone". Hoisting `ms1Readback` out of the per-seed loop would leave the suite green:
  `TestVerifyCoversEveryMastersSecret` (legs_test:223-260), the only test of that property,
  builds its legs by hand through `s5ReDerivedLegs` and never enters the flow. This is
  precisely the failure mode multisig_verify_flow_test.go:15-24 was written to prevent —
  "the previous attempt at this defect shipped a mechanism inside that loop which NO test
  could see" — reproduced one field over.
* **The I-2 fix from `f0006b7`.** That commit converted the ms1 entry's `return` to a
  `break` so a partial verify reports "Verify Incomplete" instead of walking out silently to
  the restore document. Coverage on :599-609 is 0. The fix is unpinned; deleting it restores
  the silent-abandon bug with a green suite. This is the cycle's own "an unpinned fix is
  indistinguishable from an inert one".
* **The success string an operator reads after a successful full multi-plate verify**
  (:718-721) has never been rendered, so it has never been measured against the F-185
  drawn-frame class check either.

**Failure scenario:** a future edit hoists `ms1Readback` above `for len(legs) < len(expectedSlots)`
(it reads like a harmless extraction — one prompt instead of N). Trace B in full mode:
operator types master A, types ms1(A); flow now reuses ms1(A) for master B's leg; master B's
seed plate was mis-cut and actually carries master A's words. Every leg compares ms1(A)
against ms1(A) and passes. Final screen "All 3 operator key plates verified, and the ms1 you
typed for each seed." k=3 cannot be met and the operator has been told it can. `go test ./...`
stays green throughout.

**Verified by:** the coverage run above (0 executions on the whole `full` branch) plus the
`grep` of all five call sites. A test cannot detect a change in code it never executes.

**Also uncovered in the same function, same cause (the drivers all click "Skip"):**
:528-531 (the "Add passphrase" branch — so `passphraseFlow` never feeds this flow),
:563-566 (the F-191 arm's flow wiring — `multisigVerifySeedIsInnocent` and
`multisigVerifyNoSlotBody` are unit-tested at multisig_verify_passphrase_test.go:35-42,138-162
but their call site is not, and with no passphrase ever typed the `innocent` arm cannot fire),
:571-573 (the "already checked" arm), :519-520 (Back at seed entry), :647-649 (the
zero-leg silent abandon).

---

## Important 2 — the verify's failure screen discards every diagnosis, including the slot the tests assert it names

**File:** `gui/multisig_verify.go:664-667`
**Category:** correctness (operator-facing) / inert mechanism

```go
if err := verifyMultisigLegs(legs, readbackMk1s, readbackMd1); err != nil {
    showError(ctx, th, "Verify Failed", "The read-back bundle does NOT match the seed. Check the engraved plates.")
    return
}
```

`err` is dropped on the floor. Every distinct diagnosis collapses into one sentence that
blames the steel:

* `verify: ms1 entropy mismatch` — a typo in a hand-typed 48-character codex32 string
* `verify: ms1 wordlist/language mismatch`, `verify: ms1 presence mismatch`
* `verify: fingerprint mismatch`, `verify: xpub mismatch`, `verify: origin path mismatch`
* `verify: readback mk1/md1 stub mismatch`
* `errVerifyLegHasNoPlate{Slot}` — "no read-back key plate carries slot @N's key"
* `errVerifyPlateUnclaimed{Plate}` — "read-back key plate N belongs to no leg of this policy"

S5 introduced the last two **specifically so the operator could act on them**, and pinned
the naming in three places:

* multisig_verify.go:186-188 — "It names the SLOT, because that is the only thing the
  operator can act on."
* legs_test:154-157 — `t.Errorf("the failure %q does not name %s, so the operator cannot
  tell WHICH plate to re-cut", ...)`
* legs_test:464-467 — same assertion for `@0` on a zero-plate readback.

No production path renders either string. Coverage corroborates it for one of them:
`multisig_verify.go:212.49,215.2 1 0` — `errVerifyPlateUnclaimed.Error()` has never been
called by anything, test or product. The tests assert a property whose stated purpose is
false, which is the same class as F-189 (a retired API with no production caller) at a live
site rather than a dead one.

Pre-S5 the generic text was defensible: `git show main:gui/multisig_verify.go` shows the
identical string at its :126, where exactly ONE plate existed and "the engraved plates" was
unambiguous. S5 makes a run cut 3–9 plates, and the same sentence now points at all of them.

**Failure scenario:** Trace B, full mode, 3 key plates + 2 seed plates on the bench. The
operator re-types master A correctly, then mistypes one character of ms1(A) at the "Type ms1"
screen. `codex32.DecodeMS1` accepts it (a valid checksum is reachable from a single-character
slip in a different position only sometimes, but a swapped pair of characters or a wrong
`ms1` payload character that re-checksums is enough, and the presence/language arms need no
checksum luck at all). `bundle.Verify` returns `verify: ms1 entropy mismatch`. The device
says: *"Verify Failed — The read-back bundle does NOT match the seed. Check the engraved
plates."* The operator's steel is perfect and the flow has just told them to distrust it,
while the device holds a string that says the ms1 is the problem. Combined with Important 3
there is no retry, so the plausible next action is re-cutting good plates.

This is F-191's class (a keystroke reported as a wrong wallet) at a **different site** — the
comparator rather than the slot match — and F-191's fix does not reach it.

**Verified by:** reading :664-667; `git show main:gui/multisig_verify.go | grep -n "Verify Failed" -B6`
to confirm the string is unchanged from a one-plate world; the coverage line for
`errVerifyPlateUnclaimed.Error()`; `grep -rn "no read-back key plate carries"` finds the
string only in its own `Error()` and in test assertions.

---

## Important 3 — "Verify Incomplete" instructs the operator to run verify again, and nothing on the device can run it again

**File:** `gui/multisig_verify.go:655-662`
**Category:** correctness (unachievable remedy)

```go
showError(ctx, th, "Verify Incomplete", fmt.Sprintf(
    "Checked %d of the %d key plates this run engraved. The rest were NOT "+
        "verified. Run verify again with the remaining seeds before funding "+
        "this wallet.", len(legs), len(expectedSlots)))
```

`multisigVerifyFlow` has exactly two callers and both are one-shot offers immediately after
`bundleEngrave`:

* `gui/multisig_build.go:396-401` — a single `if sel == 0 { multisigVerifyFlow(...) }`, no
  loop; on return the flow proceeds to step (11), the restore document.
* `gui/multisig.go:296-298` — the same shape on the supply path.

There is no standalone bundle-verify program. The dispatch table at `gui/gui.go:1840-1870`
lists `qaProgram, engraveXpub, engraveBundle, engraveSingleSig, engraveMultisig, loadPayload,
bip85Derive, unlockPayload, engravePassphrase, engraveText, backupWallet`; `engraveBundle`
routes to `bundleFlow` (gather + engrave, no verify), and `gui/plate_verify.go` is the
WORD-PLATE verify, whose own header (:22-25) says the bundle verifies are "untouched" by it.
`showError` is a dismiss-only modal — it offers no retry.

So the sentence names an action the operator cannot take. Their only route back to a bundle
verify is to re-run the whole engrave program and cut the plates again.

The same trap sits under "Verify Failed" (Important 2): a single mistyped character ends the
only verify the run will ever get.

**Failure scenario:** Trace B build, 3 plates for slots {0,1,2}. Operator types master A,
which covers @0 and @1. The flow offers "TYPE THE NEXT SEED". They fumble one word of master
B, or type master C by mistake (`TestVerifyReportsIncompleteAfterAMidLoopRefusal` drives
exactly this), get the refusal modal, and the loop `break`s. Screen: *"Checked 2 of the 3 key
plates this run engraved. The rest were NOT verified. Run verify again with the remaining
seeds before funding this wallet."* Dismiss → the restore document → the program ends. Master
B's plate is never checked, and the instruction for fixing that has no implementation. The
predictable operator response is to fund the wallet anyway.

**Verified by:** `grep -rn "multisigVerifyFlow" gui/` (two production call sites, both
one-shot); reading multisig_build.go:396-401 and multisig.go:296-298; reading the program
dispatch at gui/gui.go:1840-1870; reading gui/plate_verify.go:1-25.

---

## Minor 4 — `verifyClaimPlate` pairs on a predicate coarser than the one that accepts, so greedy can pick the wrong plate

**File:** `gui/multisig_verify.go:329-347`
**Category:** correctness (latent false RED)

Pairing is `got.Xpub == w.Xpub` and nothing else; acceptance is `bundle.Verify`, which also
requires the fingerprint, the origin path and the readback's stub binding to this md1. When
two readback plates carry the same xpub, greedy takes the first unclaimed one and the leg
then fails on a field — even though a valid assignment existed. Greedy is only optimal when
the pairing predicate and the acceptance predicate agree; here they do not.

Not reachable today: any such extra plate makes `len(readbackMk1s) != len(expectedSlots)` and
the precheck at :493 refuses first. It becomes live the moment that precheck is relaxed —
which the code invites, calling it "a courtesy, not the mechanism" (:487).

**Failure scenario (post-relaxation):** operator re-cuts a wallet after changing k. The old
generation's @0 plate carries the same key at the same origin but binds to the superseded
policy stub. Both plates are in the pile, both gather (different `csid`, so no dedupe). @0's
leg claims the old plate, `checkStubBinding` fails, and an honest, complete readback reports
"Verify Failed".

**Verified by:** reading :329-347 and bundle/verify.go:34-60; confirming the precheck at :493
is the only thing bounding `len(mk1s)`.

---

## Minor 5 — the unclaimed-plate sweep cannot fire in production, and the comments assert the opposite arrangement

**File:** `gui/multisig_verify.go:318-322`, :157-159, :487-492
**Category:** records accuracy

`:157-159` presents "every plate must be claimed by a leg" as a live guarantee of the
comparator, and `:487-492` demotes the length precheck to "a courtesy, not the mechanism".
In production the arrangement is exactly inverted: `len(legs) == len(expectedSlots) == len(mk1s)`
at the call, so the pigeonhole leaves nothing unclaimed and the sweep at :318-322 is
unreachable. The precheck IS the mechanism for that direction. Coverage agrees —
`multisig_verify.go:212.49,215.2 1 0`, `errVerifyPlateUnclaimed.Error()` never runs.

Not a defect: the guarantee holds twice over, and defence in depth here is right. But a
future reviewer relaxing the precheck (see Minor 4) will read a comment claiming the sweep
already carries the direction, when the sweep is untested against a production path and its
message has never been rendered.

**Verified by:** the cardinality trace above (:493, :517, :614-623, :655) and the coverage
line for `errVerifyPlateUnclaimed.Error()`.

---

## Explicitly checked and found SOUND (do not re-litigate)

* `verifyFreshSlots` (:271-283) — `expected ∩ filled − covered`, refuses an empty `expected`.
  Both producers hand it a duplicate-free list.
* The md1 byte-equality gate (:478) runs before the decode, before the count precheck and
  before any secret is requested. `slices.Equal` on chunk strings, not decoded fields.
* Termination is `len(legs) < len(expectedSlots)`, never `len(readbackMk1s)`.
* One seed at several accounts (Trace B master A at @0 and @1) yields several legs from one
  entry; one seed under two different passphrases is reachable by re-entering the same seed
  at the next-seed prompt, and works.
* `bundle.Verify`'s ms1 presence semantics are not masked; a watch-only run skips the leg on
  both sides, a full run compares recovered entropy AND wordlist language.
* Build-path origin agreement: `buildSelfKeys` skips `both` slots (multisig_build_slots.go
  region / multisig_build.go:504-524), so `assembleBuildPolicy`'s `bySlot` covers exactly the
  `slotFromSeed` slots and both loops advance `gi` over the same complement — the engraved
  mk1's origin and the policy's declared origin for each slot cannot come apart.
* `csid` is a `uint32`; a collision would DROP a plate at the gatherer and trip the count
  precheck — a false RED at ~3e-10 for a 3-plate run, not a false GREEN. Not reported.
