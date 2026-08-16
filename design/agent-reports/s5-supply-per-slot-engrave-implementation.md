# S5 / F-188 — the SUPPLY path engraves a plate PER MATCHED SLOT

Implementer report. Worktree `/scratch/code/shibboleth/wt-s5`, branch
`s5-multislot`. Base `070686a`, commit **`853534a`**, worktree clean.

Ruling implemented: **F-188** (operator, 2026-08-15, "Build this"). Rust-primary
does not bind — fork-native GUI/UX, exempt clause (b).

---

## 1. What changed, with `file:line`

All paths are under `/scratch/code/shibboleth/wt-s5`.

### New — `gui/multisig_supply_tail.go`

| line | what |
| --- | --- |
| `:43` | `errSupplyNoMatchedSlot`, the sentinel for a tail handed no slot |
| `:71` | `supplyEngraveTail(m, passphrase, net, keys, matched, suppliedMd1, full) ([]int, []bundleCard, error)` |
| `:106` | emits through the EXISTING `multisigEngraveCardsMulti` — no second emitter |

Behaviour: one `deriveMultisigLeg` per matched slot **at `keys[s].OriginPath`**;
one ms1 per distinct seed, keyed on the *engraved ms1 string* exactly as
`buildEngraveTail` does it (the supply path has one seed, so exactly one ms1); a
matched index outside the policy is an **error**, never a silent skip (a skip
would shrink the plate set and then hand the verify a list that agrees with the
shrunken steel, so the whole run would look consistent while a held slot went
uncut). The slot list it returns is built inside the loop that cut the plates, so
the verify's obligation list has the engraver's provenance rather than the
caller's.

### Rewired — `gui/multisig.go` (`supplyMultisigPolicyFlow`)

| line | before | after |
| --- | --- | --- |
| `:162` | `findUserSlot(...)` → first match only | `slots := allUserSlots(...)`; zero matches still refuse |
| `:173` | the false "reused key" `showError` | `showNotice` stating what WILL be cut (§2) |
| `:196` | one `deriveMultisigLeg` at `origin` | `engravedSlots, cardsOut, err := supplyEngraveTail(...)` |
| `:225` | *(nothing)* | `confirmReviewScreen(ctx, th, "Plates To Cut", buildPlateCensusLines(cardsOut))` — the count **before** the tail |
| `:231` | `bundleEngrave(ctx, th, cardsOut)` (cards from `multisigEngraveCards`) | `bundleEngrave(ctx, th, cardsOut)` (cards from the tail) |
| `:250` | `multisigVerifyFlow(ctx, th, full, []int{idx})` | `multisigVerifyFlow(ctx, th, full, engravedSlots)` |

Also updated in that file: the two flow header comments (`:15-24`, `:70-84`) and
the I-7 per-leg-scrub note (`:37-42`), all of which described the one-plate rule.

### Comments my change falsified — corrected, not left standing

- `gui/multisig_engrave.go:14-25` — `multisigEngraveCards` claimed it existed "so
  the SUPPLY path keeps byte-identical behaviour". It now has **no production
  caller** (see §7 F-1); the doc says so plainly rather than vouching for a
  caller that is gone.
- `gui/multisig_match.go:20-44` — `findUserSlot`'s doc said `reused` exists "so
  the caller can show a notice". No production consumer remains; the doc now
  says which question the function answers and points engrave/verify code at
  `allUserSlots`.
- `gui/multisig_verify.go:139-141, :150-176, :271-286` — three places asserted
  "the SUPPLY path cuts exactly ONE, for the first slot the seed matched
  (gui/multisig.go:141-149)". Corrected, **and** the load-bearing half made
  explicit: the intersection in `verifyFreshSlots` is still needed because on the
  BUILD path `allUserSlots` can exceed what was cut (a cosigner card carrying a
  DIFFERENT key from the same seed at another origin is admitted —
  `duplicateSlotPair` refuses only IDENTICAL keys).
- `gui/multisig_verify_flow_test.go` — `s5SupplyReadback` → **`s5OneSlotReadback`**
  and `TestVerifySupplyShapeChecksTheONEPlateItEngraved` →
  **`TestVerifyOneSlotRunChecksTheONEPlateItEngraved`**. The fixture is no longer
  the supply path's portrait; it is the intersection rule's fixture, and it is
  still reachable from a BUILD holding a subset of what its seed fills. Names
  that assert a false thing were renamed rather than annotated.

`verifyMultisigLegs` is **byte-untouched**. The BUILD path's engrave behaviour is
**unchanged**. `oracle/**` and `cmd/emu/*.js` were not edited.

---

## 2. Operator-facing strings — every one I added or changed, verbatim

### REMOVED (`gui/multisig.go`, was `:147-148`)

```go
showError(ctx, th, "Engrave Multisig",
    fmt.Sprintf("This key is reused at slots %s; engraving the first (@%d).", formatSlotList(reused), idx))
```

rendering, for Trace B and master A: **`This key is reused at slots @0 and @1; engraving the first (@0).`**

### ADDED — the multi-slot notice (`gui/multisig.go:172-177`)

```go
if len(slots) >= 2 {
    showNotice(ctx, th, "Engrave Multisig", fmt.Sprintf(
        "Your seed is at slots %s of this policy, at a different key path in each, so "+
            "each of those slots holds a DIFFERENT key. This run engraves %s, one per "+
            "slot.", formatSlotList(slots), plateWord(len(slots), "key plate", "key plates")))
}
```

rendered on the device (captured from the flow drive, spaces stripped by the
screen-text extractor):

```
Your seed is at slots @0 and @1 of this policy, at a different key path in each,
so each of those slots holds a DIFFERENT key. This run engraves 2 key plates,
one per slot.
[title] Engrave Multisig
```

`showNotice`, not `showError`: it is an announcement, not a refusal. Gated at
`len(slots) >= 2` — a one-slot supply engrave says nothing new. No em-dash and no
`·` (F-78 / F-151: both are zero-pixel glyphs in the body face).

### ADDED — the plate census screen title (`gui/multisig.go:225`)

**`Plates To Cut`**

The body is `buildPlateCensusLines(cardsOut)`, the **same shared function** the
build path uses, so the text the operator reads is identical in both flows.
Rendered for a FULL Trace-B supply engrave:

```
Plates To Cut
This engraves 14 plates.
ms1 secret share: 1 plate (secret seed backup)
mk1 key 1 of 2: 2 plates (account key card)
mk1 key 2 of 2: 3 plates (account key card)
md1 descriptor: 8 plates (wallet policy descriptor)
Each plate takes minutes to cut. Have that many blanks ready before you start:
a set is only a backup when all of it exists.
```

**Why the title is not the build path's.** `cmd/emu/needle_test.go` requires
every string a walk anchors on to have exactly ONE production site (F-169: a walk
that cannot tell which flow drew a screen proves nothing about the flow it claims
to have driven). Reusing the build census title made it two-site and
`TestBuildFlowNeedlesHaveExactlyOneProductionSite` failed **in the same run** —
verbatim:

```
--- FAIL: TestBuildFlowNeedlesHaveExactlyOneProductionSite (0.02s)
    needle_test.go:134: needle "Plate Count" has 2 production site(s), want exactly 1:
          gui/multisig.go
          gui/multisig_build.go
        a walk anchoring on this cannot prove which flow it is in
```

Re-anchoring `walk_s4_gate.js` was the alternative and there is nothing to
re-anchor it **on**: every other string on that screen comes from the shared
`buildPlateCensusLines`, so a single *file* site would be false uniqueness. The
emulator walk is out of scope for this block, so the title was made distinct
instead. Note the counter matches **source bytes including comments** — my first
explanatory comment quoted the build title and re-broke the gate; the comment now
avoids the literal and says why.

No other operator-facing string was added or altered. The zero-match refusal
("This seed is not a cosigner of the supplied policy.") and the derive-failure
message are byte-unchanged.

---

## 3. Tests, with verbatim RED

New file: `gui/multisig_supply_multislot_test.go` (5 tests, 3 helpers). Written
before the implementation, in two rounds.

### Round A — flow-level tests (no new symbols, so the RED is behavioural)

`nix develop --command go test ./gui/ -count=1 -run 'TestSupplyFlow|TestVerifyRefusesAPartialReadback' -v` → **exit 1**

```
=== RUN   TestSupplyFlowEngravesAPlatePerMatchedSlot
    multisig_supply_multislot_test.go:198: the plate census was not shown before the tail; got "ChooseengravingTEXT+QRTEXTONLYQRONLYCard1of3|Plate1of1".
        This is the change that makes the operator cut MORE plates than the same inputs produced yesterday, so the count has to arrive before the first one
--- FAIL: TestSupplyFlowEngravesAPlatePerMatchedSlot (0.08s)
=== RUN   TestSupplyFlowAnnouncesWhatWillBeCut
    multisig_supply_multislot_test.go:232: the plate census was not shown before the tail; got "ChooseengravingTEXT+QRTEXTONLYQRONLYCard1of3|Plate1of1".
        This is the change that makes the operator cut MORE plates than the same inputs produced yesterday, so the count has to arrive before the first one
--- FAIL: TestSupplyFlowEngravesAPlatePerMatchedSlot (0.06s)
=== RUN   TestVerifyRefusesAPartialReadbackOfAThreePlateBuild
    multisig_supply_multislot_test.go:343: short readback verdict: "Readback2keyplates,butthisrunengraved3keyplates.Presentexactlytheplatesthisruncut.VerifyBundle"
--- PASS: TestVerifyRefusesAPartialReadbackOfAThreePlateBuild (0.11s)
FAIL
FAIL	seedhammer.com/gui	0.264s
```

`Card 1 of 3` in that output **is the defect**: full mode was ms1 + ONE mk1 + md1.

**The partial-readback arm PASSED at `070686a`.** Reported as measured, not as a
success: it is a regression guard for a defect that the S5 verify work already
closed, and the honest question is whether it *can* fail — answered in §4.

### Round A2 — tail-level tests (compile RED)

`... -run 'TestSupplyEngraveTail|TestSupplyEngraveVerifies'` → **exit 1**

```
# seedhammer.com/gui [seedhammer.com/gui.test]
gui/multisig_supply_multislot_test.go:361:26: undefined: supplyEngraveTail
gui/multisig_supply_multislot_test.go:447:20: undefined: supplyEngraveTail
gui/multisig_supply_multislot_test.go:475:26: undefined: supplyEngraveTail
FAIL	seedhammer.com/gui [build failed]
```

A compile RED is a weak RED; it is reported as such. The behavioural REDs above
and the mutations in §4 are what carry the proof.

### The tests themselves

| test | line | what it pins |
| --- | --- | --- |
| `TestSupplyFlowEngravesAPlatePerMatchedSlot` | `:195` | **FLOW level.** Drives `supplyMultisigPolicyFlow` (gather → seed → passphrase → notice → mode → census → engrave picker). Census carries `mk1 key 1 of 2`, `mk1 key 2 of 2`, exactly ONE `ms1 secret share`, and `md1 descriptor`; the engrave counter reads `Card 1 of 4`. |
| `TestSupplyFlowAnnouncesWhatWillBeCut` | `:231` | **FLOW level.** The notice exists, names `@0`, `@1` and `2 key plates`, and carries neither `reused` nor `engraving the first`. |
| `TestSupplyEngraveTailCutsAPlatePerMatchedSlot` | `:359` | Card kinds in `multisigEngraveCardsMulti` order `[ms1, mk1, mk1, md1]`; each mk1 decoded and checked to carry **that slot's** key (chaincode ‖ compressed pubkey) at **that slot's** origin; md1 verbatim; watch-only emits no `cardMS1`. |
| `TestSupplyEngraveVerifiesItsOwnOutput` | `:473` | The tail's own plates + its own returned slot list verify **clean end to end** through the real `multisigVerifyFlow`. |
| `TestVerifyRefusesAPartialReadbackOfAThreePlateBuild` | `:319` | The disclosed gap: a 3-plate build, 2 plates read back, expectation `{0,1,2}` → **not** "Verify OK". |

`s5SupplyPremise` (`:57`) measures the premise rather than assuming it — master A
must fill exactly 2 slots, at different origins, with different keys — and fails
loudly with a message saying the file has stopped testing its subject if that
ever changes.

Final green, verbatim:

```
=== RUN   TestSupplyFlowEngravesAPlatePerMatchedSlot
    multisig_supply_multislot_test.go:200: census screen: "PlatesToCutThisengraves14plates.ms1secretshare:1plate(secretseedbackup)mk1key1of2:2plates(accountkeycard)mk1key2of2:3plates(accountkeycard)md1descriptor:8plates(walletpolicydescriptor)Eachplatetakesminutestocut.Havethatmanyblanksreadybeforeyoustart:asetisonlyabackupwhenallofitexists."
    multisig_supply_multislot_test.go:201: first engrave screen: "ChooseengravingTEXT+QRTEXTONLYQRONLYCard1of4|Plate1of1"
--- PASS: TestSupplyFlowEngravesAPlatePerMatchedSlot (0.15s)
=== RUN   TestSupplyFlowAnnouncesWhatWillBeCut
    multisig_supply_multislot_test.go:236: multi-slot notice: "Yourseedisatslots@0and@1ofthispolicy,atadifferentkeypathineach,soeachofthoseslotsholdsaDIFFERENTkey.Thisrunengraves2keyplates,oneperslot.EngraveMultisig"
--- PASS: TestSupplyFlowAnnouncesWhatWillBeCut (0.11s)
=== RUN   TestVerifyRefusesAPartialReadbackOfAThreePlateBuild
    multisig_supply_multislot_test.go:347: short readback verdict: "Readback2keyplates,butthisrunengraved3keyplates.Presentexactlytheplatesthisruncut.VerifyBundle"
--- PASS: TestVerifyRefusesAPartialReadbackOfAThreePlateBuild (0.03s)
=== RUN   TestSupplyEngraveTailCutsAPlatePerMatchedSlot
--- PASS: TestSupplyEngraveTailCutsAPlatePerMatchedSlot (0.01s)
=== RUN   TestSupplyEngraveVerifiesItsOwnOutput
--- PASS: TestSupplyEngraveVerifiesItsOwnOutput (0.07s)
=== RUN   TestVerifyOneSlotRunChecksTheONEPlateItEngraved
--- PASS: TestVerifyOneSlotRunChecksTheONEPlateItEngraved (0.09s)
=== RUN   TestVerifyStillFailsWhenTheENGRAVEDPlateIsWrong
--- PASS: TestVerifyStillFailsWhenTheENGRAVEDPlateIsWrong (0.03s)
=== RUN   TestVerifyBuildShapeChecksEveryEngravedPlate
--- PASS: TestVerifyBuildShapeChecksEveryEngravedPlate (0.03s)
PASS
ok  	seedhammer.com/gui	0.535s
```

---

## 4. Mutations — the wiring, at FLOW level, with proof the reverted line RAN

### MUT-SUPPLY-ONEPLATE — the required flow-level mutation

`gui/multisig.go` reverted to `findUserSlot` + ONE `deriveMultisigLeg` at the
first match + `multisigEngraveCards`, with `fmt.Fprintf(os.Stderr, ...)` markers
on the reverted lines. The census, the notice and the verify wiring were left in
place, so **only the engrave rule** was mutated.

`nix develop --command go test ./... -count=1` → **exit 1**, verbatim:

```
MUT-SUPPLY-ONEPLATE-RAN first=@0 matched=[0 1] engraving=1 plate
MUT-SUPPLY-ONEPLATE-RAN cards=3 engravedSlots=[0]
--- FAIL: TestSupplyFlowEngravesAPlatePerMatchedSlot (0.13s)
    multisig_supply_multislot_test.go:207: the plate census does not carry "mk1 key 1 of 2". The operator holds a seed that is at slots [0 1] of this policy, and every one of them needs its own key plate:
        "PlatesToCutThisengraves11plates.ms1secretshare:1plate(secretseedbackup)mk1key:2plates(accountkeycard)md1descriptor:8plates(walletpolicydescriptor)Eachplatetakesminutestocut.Havethatmanyblanksreadybeforeyoustart:asetisonlyabackupwhenallofitexists."
    multisig_supply_multislot_test.go:207: the plate census does not carry "mk1 key 2 of 2". ...
    multisig_supply_multislot_test.go:220: the engrave set is not ms1 + 2 mk1 + md1. The first plate announces "ChooseengravingTEXT+QRTEXTONLYQRONLYCard1of3|Plate1of1", want "Card 1 of 4"
MUT-SUPPLY-ONEPLATE-RAN first=@0 matched=[0 1] engraving=1 plate
MUT-SUPPLY-ONEPLATE-RAN cards=3 engravedSlots=[0]
FAIL
FAIL	seedhammer.com/gui	100.456s
```

The marker proves the reverted line **ran**: `matched=[0 1] engraving=1 plate`,
`cards=3 engravedSlots=[0]`. The suite goes RED **at the flow**, on the operator's
own census and card counter — not at a helper. 14 plates green, 11 mutated.

`TestSupplyFlowAnnouncesWhatWillBeCut` correctly stayed GREEN under this
mutation: the announcement is not the engrave, and each test fails for its own
subject.

### MUT-VERIFY-READBACKBOUND — proving the partial-readback arm can fail

`gui/multisig_verify.go`: the derive-loop bound reverted to `len(readbackMk1s)`
(the shipped defect) **and** the length precheck removed, both markered.

`... -run 'TestVerifyRefusesAPartialReadbackOfAThreePlateBuild' -v` → **exit 1**:

```
=== RUN   TestVerifyRefusesAPartialReadbackOfAThreePlateBuild
MUT-VERIFY-READBACKBOUND-RAN precheck=SKIPPED readback=2 expected=3
MUT-VERIFY-READBACKBOUND-RAN bound=len(readbackMk1s)=2 (expected=3)
    multisig_supply_multislot_test.go:336: a THREE-plate build verified clean against a TWO-plate readback. Final screen: "All2operatorkeyplatesverified.Othercosigners'keysaretakenassupplied.VerifyOK"
        Master B's plate was never presented and master B's seed was never asked for, so nothing about @2 was checked. That is the false GREEN this expectation list exists to remove
--- FAIL: TestVerifyRefusesAPartialReadbackOfAThreePlateBuild (0.16s)
```

The arm reproduces the false GREEN **exactly as described in the brief** —
"All 2 operator key plates verified … Verify OK" over a three-plate build, with
master B's seed never requested.

### MUT-VERIFY-PRECHECKONLY — which mechanism actually carries the arm

Precheck removed, `expectedSlots` loop bound kept. → **exit 0, PASS**:

```
=== RUN   TestVerifyRefusesAPartialReadbackOfAThreePlateBuild
MUT-VERIFY-PRECHECKONLY-RAN precheck=SKIPPED readback=2 expected=3
    multisig_supply_multislot_test.go:345: short readback verdict: "1keyplateisnotcheckedyet.Nextseed?TYPETHENEXTSEEDSTOPHEREVerifyBundle"
--- PASS: TestVerifyRefusesAPartialReadbackOfAThreePlateBuild (0.14s)
```

So the arm is guarded by **two independent mechanisms**: the length precheck
(which fires first today) and the `expectedSlots` loop bound (which asks for the
next seed). Killing either alone leaves it green; killing both produces the false
GREEN. Worth knowing, because a future change that removes the precheck as
"redundant courtesy" will not be caught by this arm alone.

All three mutations were reverted from the backups; `grep -c "MUT-"` over both
files returns 0 and the commit contains no marker code.

---

## 5. Gate — verbatim, with true exit codes

Commands run unpiped, redirected to files, exit code echoed from `$?` before any
`grep`.

```
$ nix develop --command go test ./... -count=1
exit 0   ok=51   FAIL=0

$ gofmt -l ./
exit 0   (empty — 0 bytes on stdout)

$ nix develop --command go vet ./...   # COLD GOCACHE (rm -rf'd, fresh dir)
exit 1   findings=40   in _test.go=40   outside _test.go=0
```

Matches the stated clean baseline exactly (`exit 0 / 51 ok / 0 FAIL`; gofmt
empty; vet `exit 1 / 40 findings`, none outside `_test.go`). None of the 40 vet
findings is in a file this block created or edited — checked by
`grep -E "multisig_supply_(tail|multislot)"` over the vet output, 0 hits.

Note on gofmt: the `nix develop` wrapper writes `warning: Git tree ... is dirty`
to **stderr**, which looked like output the first time. `2>/dev/null` confirms
stdout is 0 bytes.

Committed as `853534a` with the above in the message. Worktree clean.

---

## 6. What I could not do, and what I assumed

1. **The partial-readback arm was already GREEN at `070686a`.** The brief said it
   was the disclosed gap; it is, but the gap was *the missing test*, not a live
   defect — the length precheck at `gui/multisig_verify.go:332` already refuses.
   I did not change any verify behaviour to make the arm meaningful; I pinned the
   behaviour and proved by mutation that the arm can fail (§4). Reported here
   rather than implied by a PASS.

2. **The census screen title had to differ from the build path's.** Adding a
   second production site for the build walk's anchor broke
   `cmd/emu/needle_test.go`. The emulator walk is out of scope for this block and
   there is no build-unique string on that screen to re-anchor on, so the supply
   title is `Plates To Cut`. The **body** — including the count, which is what
   item 5 of the brief is about — is the shared `buildPlateCensusLines` output and
   is identical in both flows. If the reviewer prefers one title, the change is a
   one-line edit plus a walk re-anchor, and it needs the walk in scope.

3. **The census is now unconditional on the supply path**, including for a
   one-slot engrave. A conditional census would be a screen that appears only in
   the unusual case, which is a worse contract and untestable on the common path.
   This is a screen the supply flow did not previously draw at all.

4. **The Plate Count screen is not driven past by any emulator walk.** No walk
   drives `supplyMultisigPolicyFlow`; `walk_trace_a.js` drives Engrave Bundle and
   the other three drive Build policy. The flow-level Go tests are the only
   coverage of this path's screens, and they stop at the first engrave picker
   rather than cutting all 14 plates.

5. **I did not run the S5 emulator walk or mint a gate record** — both explicitly
   out of scope.

6. **`bundleEngrave` itself is unchanged**, so plate ORDER on the supply path is
   whatever `multisigEngraveCardsMulti` emits: ms1 FIRST in full mode. The plan's
   "public plates first, secret last" reordering is DEFERRED by S5's own text and
   was not touched.

---

## 7. Follow-ups this block found and did not act on

These are for `design/FOLLOWUPS.md`; I could not file them because
`FOLLOWUPS.md` lives in the `mnemonic-engrave` repo, not this worktree.

**F-1 — `multisigEngraveCards` now has no production caller.** It was the
one-of-each adapter retained so the supply path kept byte-identical behaviour;
that path now calls `multisigEngraveCardsMulti` through `supplyEngraveTail`. It
still has its own test (`gui/multisig_engrave_test.go:14,36`) and
`oracle/expect.go:139` cites it by name in a comment. Left in place with an
honest doc note (`gui/multisig_engrave.go:14-25`) rather than deleted: it belongs
to the build tail's block (`7910e00`) and its removal touches a comment in
`oracle/**`, both out of scope here. Owning phase: S5.

**F-2 — `findUserSlot`'s `reused` return has no production consumer.** Its only
production caller is the build slot gate (`gui/multisig_build_slots.go:384`),
which discards it. Still exercised by `gui/multisig_match_test.go` and
`FuzzFindUserSlot`. Owning phase: S5 or later cleanup.

**F-3 — `needle_test.go`'s counter has a blind spot this change widened.**
`productionSites` counts one entry per FILE, so a string drawn by N flows from a
**shared helper** counts as ONE site and reads as unique. Every line of
`buildPlateCensusLines` is now drawn by two flows from one file, so those body
strings would pass the uniqueness check while identifying nothing. Nothing
anchors on them today. The gate's doc already states a different blind spot
(NEEDLE_-less bare literals); this one is not stated. Owning phase: none —
batches to the walk/tooling work.
