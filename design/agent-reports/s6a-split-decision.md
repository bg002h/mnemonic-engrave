# S6a split decision — operator ruling

Date: 2026-08-16. Scope: one scheduling question — split S6a (F-198 + C-1) into two cycles, or drive it as one. Ruled on the facts as given in the brief; no re-derivation, no plan audit.

## DECISION

**ONE PIECE.** Keep driving S6a as it stands.

## WHY

The only event that changes anything for the operator holding steel years later is the **hardware flash**, and the flash waits for *both* fixes under either option (Fact 6: C-1 stays scheduled before the flash regardless). Splitting therefore does not move the date on which a real backup stops lying about its passphrase — it only moves the date on which a commit lands in a repo no machine is running. For the operator waiting today, the split is actively slower: it re-authors one plan into two documents, and under this project's own rules a re-authoring is a fold that re-earns the gate on *both* halves — a fresh spec and review loop for the C-1 cycle, plus one or two rounds for the F-198 half. Meanwhile the thing that generated every Critical since round 0 — the C-1 status algorithm — was just deleted and replaced with something far simpler (Fact 4), so the round count overstates the distance to GREEN. The churn engine is gone; finishing the simplified plan is the shortest path to the flash. This also honors the operator's "compress" directive on **both** readings: fewer cycles (two stays two, not back to three) *and* least time until the funds defect is fixed where it counts, which is the flash date — identical or later under SPLIT.

The one real cost of ONE PIECE — F-198's finished parts getting dragged through further C-1 rounds — is already bounded by the proportional re-review rule: F-198's three changes have produced zero findings in six rounds and belong in the next brief as settled facts, with review scoped to the C-1 simplification only.

## THE SEPARABILITY RULING

**No — F-198 cannot ship safely without C-1 as fielded behavior, and that alone would decide against SPLIT even if the schedule argument were closer.**

Today the restore document says nothing; after F-198 it asserts a plate inventory and completeness. C-1 is that this document prints even when the device's own verification just told the operator the plates DO NOT MATCH the seed. Land F-198 alone and you have upgraded a silent document into a vouching one while the vouching-after-failed-verification path still exists — Fact 3 states this directly: F-198's fix makes the C-1 problem *worse*. So the intermediate state a split would create is locally more dangerous than today's state, and it is rendered harmless only by a scheduling accident: the flash gate ensures no steel is ever cut from F-198-only firmware, because C-1 lands before the flash under either option. Safety that depends entirely on a downstream gate holding is not separability — it is exactly the coupling S6a already encodes, expressed as two artifacts instead of one. The honest structure is the current one: the two fixes are one change to what the restore document is allowed to claim, and they should close under one gate.

## WHAT I AM ASSUMING

1. **Fact 6 holds as stated**: C-1 remains scheduled before the hardware flash under either option, and no intermediate flash puts F-198-only firmware on the machine. If anyone would flash between cycles, the separability ruling above makes that flatly unsafe and the decision would need revisiting — but the answer would still not be SPLIT; it would be "do not flash mid-way."
2. **Fact 4's simplification stands**: the algorithm that produced the last two Criticals is gone, so remaining rounds converge rather than churn. If the *simplified* replacement now starts producing Criticals of its own, that is new information and the question may be re-asked — but from here, one more scoped round on a simplified plan is cheaper than two fresh gates.
3. **"Compress" means less total process before the flash**, with cycle count as its proxy — the operator's own action an hour ago (merging three cycles into two) fixes that reading. I found no interpretation of "compress" under which adding a cycle back, without moving the flash date earlier, complies.
4. Review effort already spent is sunk and was given no weight; the six invested rounds argue for ONE PIECE only insofar as they leave F-198's parts settled and the brief scopeable.
