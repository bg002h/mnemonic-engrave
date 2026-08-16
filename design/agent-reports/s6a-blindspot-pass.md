# S6a blind-spot pass — what P1/P2/P3 collectively fail to constrain

**Trigger:** the same defect "fixed" twice, reappearing one level down each time
(R4: verdict-level conflation; R5: site-level conflation).
**Question answered:** what failure mode do the three properties fail to
constrain, and is the design's shape wrong?
**Scope:** properties of the design in §4.7 of
`design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`, read against
`design/agent-reports/s6a-r5-adversarial.md`. No code audit, no fold, no
detailed design.

---

## THE UNCONSTRAINED FAILURE MODE

**The document claiming knowledge the device does not have.** All three
properties take the classification as a given and constrain only the map from
classification to line: P1 and P2 are conditioned on "a clean pass" and "a
disagreement", P3 on "a comparison actually ran and disagreed" — every
antecedent is a fact about the *world* (what the steel is, what the operator
did), while the device possesses only *observations*, and some observations are
consistent with several worlds. Nothing constrains what the document may say
about an outcome the device cannot classify. So an ambiguous outcome —
`errVerifyLegHasNoPlate`, where "not presented" and "not the plate this run
cut" are both live — must be forced into one of five certainty-bearing buckets,
and *whichever* bucket it lands in, the line asserts a world-fact the device
never established: put it in DISAGREED and you condemn a good backup
(over-claiming, R5's Critical); put it in DID NOT COMPLETE and you erase the
adverse observation of a genuinely garbled mk1 that lands at the same site
(under-claiming — the P2-shaped harm, one proxy level up). This is also why the
defect regenerated one level down each round: P3's antecedent is a world-fact,
so the code can only implement it through a proxy — verdict, then site, then
error — and **any proxy at any level can conflate worlds**. Refining the proxy
never terminates. The regress has a fixed point only when the thing classified
is the observation itself, because there is no level below what the device saw.

## IS THE SHAPE WRONG

**Yes, as stated — the frame "classify each outcome, then map to one of five
lines" classifies the wrong thing.** But the wrong half is the classifier's
*domain*, not the classify-then-map mechanism: the two-sticky-facts-and-a-switch
back end, always-render, and status-line-first all survive (R5 verified the
switch structure sound). The current frame asks *"are the plates good?"* — a
diagnosis of the world, a question the device's own shipped text admits it
cannot always answer ("Either that plate was not presented, or it is not the
one this run cut"). The right frame is *"what did the check observe?"* — a
question the device can always answer truthfully, and truthfully is the whole
job of a durable document. Call it **evidence reporting instead of world-state
diagnosis**: classify the device's *knowledge state*, not the world, and report
it. Under that domain the classifier is total by construction — ambiguity is
itself a knowledge state, not a classification failure — so the "unclassifiable
outcome" that broke rounds 4 and 5 cannot exist. This is also not a new
directive: the operator's standing rule already commands it ("*loudly declare
assumptions we make*"), and the DISAGREED line on an ambiguous outcome is
precisely a silently-made assumption. Note the screens already speak this
frame — the `:963` text states both readings and both actions — it is only the
document that lacks the vocabulary. The document was designed as a *verdict*;
the screen evolved into a *report*. The document should follow the screen.

## THE MISSING PROPERTY

**P4 — a line may never outrun the evidence.** Stated at the same precision as
P1–P3:

> For every reachable outcome — enumerated per return path *and per error and
> per comparand provenance*, not per verdict or per site — let **W** be the set
> of world-states consistent with everything the device observed on the way to
> that outcome. Every factual claim and every instruction in the printed line
> must be true and correct in **every** member of W. If W contains worlds that
> demand different operator actions, the line must state the ambiguity and
> cover the actions; it may not select one world and assert it.

P4 subsumes P3 (P3 is P4 restricted to the DISAGREED line) and repairs P2's
phrasing (which should read "an adverse *observation* is never lost", not "a
disagreement"). It would have caught both Criticals in advance, mechanically:
R4's — at verdict granularity, W for `verifyFailed` contains seed-typo worlds,
so "a read-back check DISAGREED with these plates" is false in a member of W;
R5's — at the `errVerifyLegHasNoPlate` path, W contains the
forgot-a-plate world, same failure. It also catches R5's I-1 unprompted: the
hand-typed-ms1 divergence has a W containing "one-character transcription typo,
plates perfect", so an unqualified DISAGREED is forbidden there too. The
operational discipline is the enforcement: the review artifact is a table keyed
by *observation*, each row listing W and its line, and **a row with |W| > 1 may
not carry a line that asserts a single member of W**. That check cannot be
dodged by moving down a level, because its unit is the world-set rather than
any proxy for it. It also settles the repeat-check wording R5 flagged: after a
*full* clean pass, every plate this run cut was presented and matched, so an
earlier pairing failure is retro-explained as procedural (W collapses to
"plates fine") and plain VERIFIED is earned — whereas an earlier true
disagreement is *not* retro-explained by a later pass, and keeps its note.

## HOW MANY STATUSES

**Six, and the number is derived, not chosen: one status per distinguishable
knowledge state of the device.** The rule matters more than the count — if the
device gains or loses a way of knowing, the set moves with it. The six, and the
distinction each must carry:

1. **VERIFIED** — a comparison over plate-derived bytes completed for every
   plate and every leg matched. The device *knows* the steel encodes what this
   run intended (modulo NFC read fidelity, which §4.7 already accepts).
2. **VERIFIED on a repeat check** — as (1), after an earlier observation that a
   clean pass does **not** retro-explain: a true prior disagreement (two reads
   of the same run's steel diverged). Prior *procedural* ambiguity later
   cleared belongs in (1), per P4 — the current design's "an accusation earned
   by forgetting a plate" is exactly a P4 violation in this line.
3. **NOT VERIFIED** — no attempt ran. The device observed nothing.
4. **DID NOT COMPLETE** — an attempt ran and ended with *no adverse
   observation about the plates*: seed typo, refusal, abandon, incomplete. The
   device knows only that it doesn't know.
5. **DISAGREED** — a comparison of plate-derived bytes against this run's
   intent ran and diverged, with no ambiguity of provenance. The device knows
   the steel it read does not encode what this run cut. The only status that
   may condemn, and the only observation that earns it.
6. **NEW — PLATES UNACCOUNTED FOR / COULD NOT BE CHECKED AGAINST THIS RUN** —
   an *adverse but ambiguous* observation: W contains both an
   operator-procedure world and a bad-plate world. The line states both
   readings and both actions ("present every plate this run engraved, then
   check again; if this repeats, re-cut"), exactly as the `:963` screen text
   already does. This status is where every defect of rounds 4–5 lands:
   `errVerifyLegHasNoPlate`, the foreign-or-garbled md1 at `:719`, the garbled
   mk1 skipped at claiming, and (per I-1) the hand-typed-ms1 divergence. That
   the entire Critical-generating residue falls into the one status that does
   not exist is the strongest evidence this is the right set.

Mechanically this costs one more sticky fact (roughly: `sawDisagreement` and
`sawUnaccounted`, with explicit switch-arm order — the same shape as today, not
a resurrected severity lattice), but that is the fold's to design, not mine.

## WHAT I AM ASSUMING

- The two settled decisions stand: the document always renders, and running the
  verify never leaves the operator worse off than skipping. I read "worse off"
  as *condemned or deprived*, not *truthfully informed of an ambiguous adverse
  observation* — status 6 is softer than DISAGREED, true, and actionable, and
  its honest recovery path (fetch the plate, re-verify, get plain VERIFIED)
  ends strictly better than skipping. If the operator reads "never worse off"
  more broadly, status 6's wording needs their sign-off.
- "Exactly one status line, always, first on the page" is retained; only the
  vocabulary of statuses was in question.
- I took R5's re-classification tables (which errors reach which sites, what
  the screen texts say) as verified fact per the brief, and did not re-derive
  them against the tree.
- The claim that a full clean pass retro-explains a prior pairing failure
  assumes the later pass covered *every* plate of the run (the `:735` count
  check plus per-leg claiming); if a pass can complete over a subset, line (2)'s
  boundary moves and the fold must re-check it.
- Whether the ms1-arm divergence sits in status 6 or in a qualified status 5 is
  a fold decision; P4 only forbids it sitting in status 5 *unqualified*.
