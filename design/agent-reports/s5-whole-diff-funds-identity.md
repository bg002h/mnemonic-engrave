# S5 whole-diff review — FUNDS-SAFETY and CARDINALITY-vs-IDENTITY

**Artifact:** `/scratch/code/shibboleth/wt-s5`, `git diff main..s5-multislot`, frozen at `7da66bd`
(10 commits, 57 files, +8873/−607). Tree was **not modified**; verified `git status --porcelain`
empty and HEAD still `7da66bd` at the end of the review. All mutation experiments were run against
a `cp -a` copy in the session scratchpad, which has been deleted.

**Lens (the one question):** can a plate set that is NOT this wallet's pass the verify, or can a
plate this wallet needs be silently NOT cut?

**Verdict:** 1 Critical, 1 Important, 1 Minor, 1 Nit.

---

## FINDING 1 — CRITICAL

### "Verify Incomplete: Checked N of the M key plates this run engraved" asserts a comparison that never ran

**File:** `gui/multisig_verify.go:655-662` (the Incomplete arm), against `:664`
(`verifyMultisigLegs`, the only comparator call site).

```go
if len(legs) < len(expectedSlots) {
    showError(ctx, th, "Verify Incomplete", fmt.Sprintf(
        "Checked %d of the %d key plates this run engraved. The rest were NOT "+
            "verified. Run verify again with the remaining seeds before funding "+
            "this wallet.",
        len(legs), len(expectedSlots)))
    return
}

if err := verifyMultisigLegs(legs, readbackMk1s, readbackMd1); err != nil {
```

`verifyMultisigLegs` is the **only** place in this flow where a read-back plate is decoded, matched
to a leg, or compared at all (`verifyClaimPlate` → `mk.Decode`; `verifyMultisig` → `bundle.Verify`
→ fingerprint / xpub / origin path / stub binding / ms1 entropy). On the Incomplete path it is
never reached. `legs` is a count of slots **re-derived from a typed seed** — it is not a count of
plates compared. Nothing about any plate's *contents* has been established when this screen draws.

What HAS been established upstream is exactly three things, and all three are cardinality or
provenance, never plate identity:

1. `slices.Equal(readbackMd1, engravedMd1)` — the policy is this run's (`:478`);
2. `len(readbackMk1s) == len(expectedSlots)` — the right NUMBER of key cards came back (`:493`);
3. the seeds typed account for N of the expected slots (`allUserSlots` + `verifyFreshSlots`).

So the screen converts "I re-derived 2 legs" into "I checked 2 key plates". That is the same
false-GREEN shape this file's own `errVerifyNoLegs` comment calls "the single most expensive false
GREEN this flow can produce" — reporting success for a readback it never looked at — scoped down to
a partial verify. It is also the review's own class: a COUNT presented to the operator as an
IDENTITY claim.

**Concrete failure scenario (EXECUTED, see below).** Trace B: 3 key plates for slots {0,1,2} across
masters A and B. The operator has master A on the bench and master B's words at another location.
They present the md1 and three mk1 plates — but the plate they present for @0 is **not this
wallet's key card at all** (an earlier generation's plate, a mis-cut, or in the probe a single-sig
mk1 from `m/44'/…` entirely). They type master A, which covers @0 and @1, then choose STOP HERE.

Screen drawn:

```
Checked 2 of the 3 key plates this run engraved. The rest were NOT verified.
Run verify again with the remaining seeds before funding this wallet.
                                                          Verify Incomplete
```

The operator reasonably concludes @0 and @1 are proven good and only @2 is outstanding. @0's plate
belongs to a different wallet and was never decoded. Every downstream act — discarding the "spare"
that was actually the correct plate, or funding once @2 alone is later proven — rests on a check
that did not happen. On a 3-of-4 this is a plate the wallet needs, believed verified.

The `full`-mode Back-out-of-ms1 path (`:609`) lands on the same screen with the same false claim.

**How I verified it.** A probe test driven through the real screens (gather → seed entry → passphrase
Skip → next-seed ChoiceScreen → `Down` → `Button3` = STOP HERE), run under `nix develop` in a
`cp -a` copy of the tree — never in the frozen tree. It reuses the tree's own helpers
(`s5TraceBEngraved`, `s5ReDerivedLegs`, `s5PlateFor`, `driveWords`, `pumpUntil`) and substitutes a
single-sig mk1 from `deriveSingleSigBundle(m, "", s5Net, singleSigPath(44), md.ScriptPkh)` for the
plate carrying @0's key. Probe source is in the session scratchpad at
`/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/492b28ac-1008-4459-a099-717290554119/scratchpad/probe-incomplete.go.txt`
(the brief permits writing only this report into the repo, so it was not committed).

```
=== RUN   TestProbeIncompleteClaimsPlatesItNeverCompared
    FINAL SCREEN: "Checked2ofthe3keyplatesthisrunengraved.TherestwereNOTverified.
                   Runverifyagainwiththeremainingseedsbeforefundingthiswallet.VerifyIncomplete"
    the flow claims it CHECKED 2 of 3 key plates. verifyMultisigLegs never ran: no plate was
    decoded, no xpub matched, no stub binding checked. One of the two plates it says it checked
    belongs to a different wallet entirely.
--- FAIL: TestProbeIncompleteClaimsPlatesItNeverCompared (0.16s)
```

`TestVerifyReportsIncompleteAfterAMidLoopRefusal` (`gui/multisig_verify_policy_test.go:238`) reaches
this screen and asserts only that the words "Verify Incomplete" appear. Nothing pins the count's
meaning, which is why the claim could drift into being false without a test noticing.

**Suggested shape of the fix (not prescriptive).** Either (a) run the comparator over the legs
collected so far before drawing the screen — each leg must still find its plate, but the
unclaimed-plate sweep is relaxed for a partial run — and report the number that actually passed,
failing loudly if one does not; or (b) stop claiming a number: say that the plates were read back
and the remaining seeds are needed before ANY of them can be checked. (a) is strictly better: a
partial verify genuinely can prove the legs it holds, and today it throws that proof away and then
claims it anyway.

---

## FINDING 2 — IMPORTANT

### The SUPPLY path's "Full (seed + keys)" and its restore doc are silent about a BIP-39 passphrase; S5 fixed exactly this on the BUILD path and left the sibling

**Files:** `gui/multisig.go:204` (the engrave-mode label) and `gui/multisig.go:302` (the restore
doc), against `gui/multisig_build.go:340` / `:415` and `gui/multisig_build_census.go:108-144`.

S5 added, on the BUILD path only:

* `buildFullModeLabel(usedPassphrase)` → `"Full (seed + keys, NOT passphrase)"`
  (`gui/multisig_build_census.go:140`), wired at `gui/multisig_build.go:340`;
* `buildPassphraseInventoryLines(passphrase)` on the restore document
  (`gui/multisig_build_census.go:108`), wired at `gui/multisig_build.go:415`.

Its own rationale (`multisig_build_census.go:83-107`) is: *"a set labelled 'Full (seed + keys)'
could be missing the one factor that reaches the money and vouch for itself while doing it. F-132's
device sibling exactly."*

The supply path takes a passphrase (`gui/multisig.go:141-150`, `syswPassphraseFlow`), derives every
leg with it (`:162`, `:217`), and then:

```go
// gui/multisig.go:201-205
modeChoice := &ChoiceScreen{
    Title:   "Engrave Mode",
    Lead:    "What to engrave?",
    Choices: []string{"Full (seed + keys)", "Watch-only (keys)"},   // hard-coded
}
...
// gui/multisig.go:302
multisigRestoreDocFlow(ctx, th, tpl, keys, nil)                      // no inventory, no passphrase line
```

Measured: `grep -rn "buildFullModeLabel\|buildPassphraseInventoryLines\|usesPassphrase()" gui/*.go`
(excluding tests) returns call sites in `multisig_build.go` only. `grep -n "passphrase"
gui/multisig_restore.go` returns nothing.

**Concrete failure scenario.** Operator supplies a coordinator-authored md1, adds a BIP-39
passphrase, chooses "Full (seed + keys)", and cuts ms1 + one mk1 per matched slot + md1. Every
artifact — the mode label, the plate census, the restore document — says this is the complete
backup. It is not: the passphrase is a required spending factor, is not in the ms1 entropy, and no
plate in the set can be made to yield it. The reader five years later, holding the steel and the
restore doc, has no statement anywhere that a factor is missing. That is the F-132 shape the build
path just declared a harm.

This is pre-existing behaviour, but S5 is what created the asymmetry, in a flow S5 rewrote
(F-188 restructured `supplyMultisigPolicyFlow` end to end). The path that now tells the truth is the
one behind the mandatory EXPERIMENTAL warning; the path that does not is the hardware-validated one.
Reporting it as a NEW instance of the same class at a different site, per the brief.

(`gui/singlesig.go:80` carries the identical hard-coded label with a passphrase flow at `:68`. Out
of this diff entirely, so noted only — not counted as a finding.)

---

## FINDING 3 — MINOR

### An aborted engrave still offers the verify and prints the full restore-doc inventory

**Files:** `gui/multisig_build.go:364-416`, `gui/multisig.go:272-302`.

`bundleEngrave` returns `void`. On a set-level abort or an unplateable string it draws
`bundleAbortWarning` and returns (`gui/bundle_flow.go:398-403`, `:378-381`); the caller cannot tell
an abort from a completed engrave. So after stopping at "card 1 of 4" the build flow still runs:

* the verify offer (`multisig_build.go:396-401`) with the FULL `engravedSlots`. If the operator
  accepts, the length precheck prints *"Read back 1 key plate, but this run engraved 3 key plates.
  Present exactly the plates this run cut."* — a statement about what "this run engraved" that is
  false, and an instruction the operator cannot satisfy;
* the restore document with `buildPlateInventoryLines(cardsOut, …)` (`:415`), enumerating plates
  that do not exist.

It fails closed (a refusal, never a pass), and S5's own `bundleAbortWarningText` does tell the
operator the set is not usable and is byte-reproducible — which is why this is Minor rather than
Important. But the two screens after it contradict that warning. The supply path has the same shape
at `gui/multisig.go:272`/`:295`.

---

## FINDING 4 — NIT

### The verify's plate-count precheck is inert — deleting it leaves the whole suite green

**File:** `gui/multisig_verify.go:493-500`.

Measured, on the frozen tree copied to scratch: replacing `if len(readbackMk1s) != len(expectedSlots)`
with `if false && …` and running `nix develop --command go test ./gui/ -count=1` gives
`ok seedhammer.com/gui 78.244s`.

It is genuinely redundant for CORRECTNESS — the bijection in `verifyMultisigLegs` plus the
`len(legs) == len(expectedSlots)` termination rule already implies the counts agree — and the code
says so ("a courtesy, not the mechanism"). The gap is that its OPERATOR-FACING value (learning about
a mislaid plate before typing a seed, with a message naming both counts) is pinned by nothing, so a
future edit that drops or breaks it is invisible. Given this cycle's headline lesson — *an unpinned
fix is indistinguishable from an inert one* — it is worth one assertion on the early refusal's text
and on the fact that it fires before the seed prompt.

---

## NEGATIVE RESULTS — what I chased and found SOUND

These were the lens's own suspects. Each was executed, not reasoned about.

**Every funds-critical mechanism in the S5 verify/engrave tails is load-bearing and pinned.**
Mutation battery, each run alone against `go test ./gui/ -count=1` in a scratch copy:

| # | Mutation (the defect it models) | Result |
|---|---|---|
| M1 | `slices.Equal(readbackMd1, engravedMd1)` → never fires (**policy identity removed** — this cycle's Critical #3) | FAIL — `TestVerifyRefusesPlatesFromADifferentPolicy` |
| M2 | `verifyFreshSlots` drops `slices.Contains(expected, s)` (**obligation taken from the seed, not the engraver**) | FAIL ×4 — `TestSupplyDuplicateSlotVerifiesItsOwnOutput`, `TestVerifyOneSlotRunChecksTheONEPlateItEngraved`, `TestVerifyFreshSlotsIsTheEngraversList`, `TestVerifyReportsIncompleteAfterAMidLoopRefusal` |
| M3 | `verifyMultisigLegs` unclaimed-plate sweep removed (**an unattributed plate passes**) | FAIL — `TestVerifyCoversEveryLeg` |
| M4 | `verifyClaimPlate` claims the first unclaimed plate regardless of xpub (**pairing by POSITION, not IDENTITY**) | FAIL — `TestVerifyPairsByKeyNotByOrigin` |
| M5 | `buildEngraveTail` ms1 dedupe keyed on `SeedID` instead of the ms1 string (**this cycle's Critical #1, exactly**) | FAIL ×2 — `TestFullModeEngravesMs1ForEveryMaster`, `TestVerifyCoversEveryMastersSecret` |
| M8 | `supplyEngraveTail` mk1 dedupe keyed on the 4-byte master fingerprint (**a truncated key that DROPS a plate**) | FAIL ×3 — `TestSupplyFlowEngravesAPlatePerMatchedSlot`, `TestSupplyEngraveTailCutsAPlatePerMatchedSlot`, `TestSupplyEngraveVerifiesItsOwnOutput` |
| M7 | plate-count precheck removed | **GREEN** → Finding 4 |

**The multi-seed verify loop reaches a PASS through the real screens.** No test in the tree drives
`multisigVerifyFlow` past one seed to "Verify OK" — `s5DriveVerify` types one seed, and
`s5DriveVerifyTwoSeeds` is only used for the Incomplete arms. I wrote three probes and ran them:
master A then B, B then A, and a reversed-order plate readback all reach
`"All 3 operator key plates verified. Other cosigners' keys are taken as supplied. Verify OK"`.
So the never-executed happy path works, including gather-order independence.

**No session-payload leak into the verify readback (§7.4).** `ctx.syswBundleSeeds` is set by the
build path at `multisig_build.go:103` and by the supply path at `multisig.go:97`, and
`bundleGatherFlow` sets `ctx.syswBundleSeeds = nil` at `bundle_flow.go:167` on the FIRST gather. The
verify's later `bundleGatherFlow` therefore sees nil — no payload md1 or cosigner mk1 can be
auto-ingested as "readback". `multisigVerifyMS1Entry` goes through `inputCodex32Flow`
(`gui/gui.go:1021`), which is keyboard-only with no payload seam.

**md1 chunk ORDER cannot false-RED the policy-identity check.** `slices.Equal` is order-sensitive;
`md1Gatherer.collected()` (`gui/md1_gather.go:61-67`) returns index order 0..total−1, never map
order, and `bundleCard.strings` is documented and built as index order. So an honest out-of-order
scan of this run's own md1 still compares equal.

**The supply-path dedupe cannot drop a plate the operator needs.** The key is the minted mk1 string
(`multisig_supply_tail.go:152`). Two matched slots collapse only if they mint byte-identical mk1s,
which requires the same origin string AND the same base58 xpub — and `allUserSlots`
(`multisig_match.go:76-95`) can only match both slots when they carry the same 65-byte cc‖pk at the
same origin. The predicate that picks the operator-facing notice (`multisigSlotsShareAKey`, keyed on
`Xpub‖origin`) is exactly equivalent to the tail's behaviour, so the announcement and the collapse
cannot disagree. The collapse note is prepended to the census (`multisig.go:259-265`) from the
tail's own return, so it leads page one of a screen confirmable from any page.

**`buildEngraveTail` has no mk1 dedupe, and does not need one.** Two byte-identical mk1s require
identical cc‖pk, which `duplicateSlotPair` (`multisig_build.go:1024`) refuses over the FINAL
assembled set before the tail runs. So the build path cannot mint the collapse shape the supply path
guards against, and the gatherer's csid dedupe cannot make a build permanently unverifiable.

**No tautological check found in the new comparison surface.** `checkStubBinding("derived", …)` and
`equalStrings(derived.MD1, readback.MD1)` are self-comparisons, but both are named as such in the
header and both are now upstream-bound by the `readbackMd1 == engravedMd1` equality. The
load-bearing halves — `checkStubBinding("readback", …)`, fingerprint, xpub, origin path, ms1
entropy + wordlist — all compare plate-against-seed. `buildSlotKeyStrings` matches the operator's
input strings against the assembled md1 by 65 bytes; it is a display helper, not a check, and says
so.

**Slot/card index plumbing agrees across the three walkers.** `buildSlotSources`
(`multisig_build.go:439-483`), `assembleBuildPolicy`'s fill loop (`:1233-1257`) and
`buildCosignerOrigins` (`multisig_build_payload.go:365-378`) walk slots in the same order with the
same "held && !SelfFromCard" skip, so `slotSource.Card`, the assembled slot contents and the
"payload card N" provenance cannot disagree. `open` (`multisig_build.go:91-94`) and
`assembleBuildPolicy`'s `len(cosigners) != p.N-len(self)` are the same count expressed twice and
agree on both the `derived` and `both` arms.

**Account numbering keyed on the 4-byte master fingerprint (`multisig_build.go:446-455`) is not a
funds hazard.** A fingerprint collision between two genuinely different masters shifts the second
one's BIP-48 account from 0 to 1. That mints a *different but correct* key, records it in the md1,
the mk1 and the restore doc, and the verify re-derives at that slot's own recorded origin. No plate
is dropped and no wrong key is engraved — unlike a fingerprint-keyed *dedupe*, which is the shape
the tails correctly avoid.

---

## SCOPE NOTES

* Not re-reported, per the brief: F-189, F-190, F-191, F-192, F-193, F-194, F-195. Finding 1 is not
  F-191: F-191 is the *no-slot-matched* modal misattributing a passphrase typo; Finding 1 is the
  *Incomplete* screen claiming a comparison that never executed.
* Not re-derived, per the brief: `go test ./...`, `gofmt -l`, `go vet`, `oracle-live.sh`,
  `cmd/emu/build.sh`.
* One thing I could not close from inside this review: CI does not run the emulator walks. `.github/
  workflows/test.yml:87-88` only does `GOOS=js GOARCH=wasm go vet ./cmd/emu/`, so `walk_trace_b.js`
  and its siblings are gated by a human remembering to run them. That is this cycle's Critical #4
  territory rather than a new instance, and the brief states the S5.D mint has run green, so it is
  recorded here and not filed.
* `cmd/emu/walk_trace_b.js:630` deliberately SKIPS the verify offer ("Skip the verify (row 1 of 2)"),
  so no walk drives `multisigVerifyFlow` at all. The probes above are what covered that gap for this
  review; the tree's own coverage of a multi-leg PASS remains comparator-level only.
