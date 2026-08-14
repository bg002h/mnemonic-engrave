# Process diagnosis — the thirteen-round multisig build-repair review cycle

Diagnostician: independent context (fable), 2026-08-13. Evidence: all 15 reports in
`design/agent-reports/multisig-build-repair-*.md` read in full; the commit timeline
`6e6c589..468e044` with timestamps; `SPEC_multisig_build_repair.md` (841 lines),
`IMPLEMENTATION_PLAN_multisig_build_repair.md` (681 lines, post-`468e044`);
FOLLOWUPS F-150/F-151/F-158; `scripts/plan-cite-gate.sh` and
`fold-propagation-check.sh` usage as recorded in the reports. This is a diagnosis of
the process, not another review of the plan.

---

## 1. Causal diagnosis

Four candidate failure classes were named in the brief. Ranked by how much of the
day each one caused:

### Dominant: sequencing — the questions were asked in an order that guaranteed late discovery

The single decisive fact of the day is a timestamp pair. **F-158 was filed at 16:03**
("no NFC gather flow can be executed by any test or in the emulator… The only
end-to-end test of the build flow stops at the gather"). **At 16:09 — six minutes
later — the spec was revised to make the payload the sole phase-1 cosigner source**
(`6c7a3b7`, "the payload is the source"). That revision was the load-bearing move of
the whole design: it routed the acceptance mechanism around the dead NFC harness and
onto the payload. Nobody then opened the payload. The author had written it the
previous day, documented its three records, and pinned them with
`TestSyswTestPayloadCarriesThreeClasses` — a test that *prints the answer*. One grep
(`ClassMDMK` under `cmd/emu/`) or one attempted click-through of Trace A at 16:09
would have surfaced, that afternoon, what instead arrived at 21:56 as the journeys
lens's Critical: **every emulator-walk gate in the plan was unsatisfiable.**

Everything else follows from that inversion. Eleven rounds ran under essentially one
question — *is the text consistent with the source and with itself* — before anyone
asked *can the acceptance mechanism run at all*. The prompt's own observation is the
diagnosis: every late Critical came from a **different question**, not from looking
harder. The five late lenses (inherited-facts, adversarial, failure-states,
journeys, comprehension, coverage) each found their Criticals/Importants **on the
first application of that question**, independent of everything the prior rounds had
settled. The rounds were not converging on those findings; the findings were
orthogonal to the rounds. That is the signature of a sequencing failure, not a
diligence failure.

The subtlest instance is spec R0 round 1's trace tables. The reviewer walked Trace A
and Trace B step by step and marked each step "Specified" or "Assumed-as-shipped,
with evidence" — and the evidence for step 1 was the Load Payload journey walked
2026-08-12 *using the very payload that has no cards*. A paper walk was accepted as
the proxy for an executable walk, in the exact document whose §4.5 exists because a
green test suite had already let a blank screen reach hardware (F-150/F-151). The
trace method was right; the substrate (prose instead of the emulator) was wrong.

### Secondary: the author's loop — fold speed, paraphrase gates, and unquarantined inheritance

Two measurable defects in the author's loop inflated the round count from ~5 to ~15:

**Fold latency.** The commit timeline shows folds landing 1–8 minutes after
multi-finding reports: 18:51 (3 min after 1C/3I/5M), 19:06 (2 min), 19:25 (1 min),
20:24 (2 min), 21:44 (5 min after 9 findings across two lens reports). Four of nine
folds were defective, all by the same mechanism — edit the sentence the reviewer
quoted, leave the restatements ("the FOURTH time", per `b474180`'s own title). The
one fold that took real time (`468e044`, 32 minutes) is also the only one whose
propagation gate used patterns taken from the reports' quoted text — and it caught
its own residual contradiction at line 63 before committing. The correlation is the
lesson: fold quality tracked fold procedure, and the procedure was skipped when the
fix looked small.

**The grep gate passed while the defect survived** because the author supplied
patterns from his paraphrase of the finding rather than the report's quoted text
(`fold-propagation-check.sh` accepts arbitrary author-chosen patterns; the A2
"by version" sweep pattern was also a single-line ERE defeated by a line wrap). A
gate whose inputs the responder chooses is a self-check wearing a gate's costume.

**Inheritance without quarantine.** The discipline that existed worked where it
pointed: all 16 fork-code citations audited came back TRUE, because claim-vs-line
plus `plan-cite-gate.sh` gated them. The five false facts (BIP-382 vectors, the mk
V19 re-pin, TYPED-ONLY = 4, "drift measured", `me` pins mk 0.4.2) all lived
**outside the gated zone** — external documents, changelog inferences, stale
comments, the author's own yesterday's fixture — where no gate ran. The 16/16 vs
5-of-22 split is not a diligence gradient; it is a coverage map of the gate. And the
cite gate's green was read as "machine-checked" globally — the repo's own rule ("a
gate that hides its blind spot is worse than no gate") applied to its own gate.

Special case worth naming: the payload was the author's **own** artifact, one day
old, fully documented by his own test. Recency of authorship functioned as immunity
from checking. "I wrote it yesterday" is inheritance, not knowledge.

### Real but smaller: reviewing-method failures

The reviewers were capable of everything they were pointed at. Two genuine method
defects: (a) round 0 of the plan review **minted** an inherited falsehood (the V19
re-pin, reasoned from `mk/mk.go:5`'s stale comment plus a changelog line) that then
survived as "settled" for two rounds under scope discipline — reviewers can create
inherited facts, and "settled inputs, not re-derived" propagates them; (b) two
scoped verdicts leaked their scope — round 3's "GREEN, implementation begins at S0"
and the propagation check's "the plan is buildable S0→S6" were both true of the
lens and false of the artifact. A per-lens clean was repeatedly read as a global
clean.

### Smallest: the artifact itself

The plan *was* wrong, but its wrongness has a precise shape: it was accurate about
the code it cited (the gated zone) and wrong about the **environment it would
execute in** — the emulator harness, the payload fixture, the operator's screens,
the contents of BIP documents, the CLI surfaces of the oracles. It is a 681-line
high-precision description of work in an environment nobody had attempted to use.
The precision (named tests, fixtures, mutations for six stages) outran the
foundation; test names for S5 were being polished while S1's gate was unexecutable.
The artifact's defects are downstream of the sequencing failure, not an independent
cause.

### Verdict on the headline question

**Thirteen rounds is both — in measurable proportions.** About seven rounds bought
real findings unreachable any other way. About four existed only to audit defective
folds — work the process manufactured for itself. Two verified closures that were
false at the whole-artifact level. The effort was not wasted — the seven late
Criticals are real, several funds-class — but roughly **half the rounds and most of
the elapsed time were spent on self-inflicted or misordered work**, and the two
GREEN gate closures were both false summits. That last point is the sharpest one:
the process's convergence signal (0C/0I) is per-lens, and it was read as global.
Had the operator obeyed the standing rule — "a re-review returning 0C/0I closes the
loop; do not keep looping for reassurance" — implementation would have started at
20:00 into five unsatisfiable gates. **The process was saved by violating its own
stopping rule.** A gate whose GREEN is overturned by the next six rounds is not a
gate; it is a lens-completion marker mislabeled.

---

## 2. Round-by-round verdict

| # | round | verdict | why |
| --- | --- | --- | --- |
| 1 | spec R0 r0 (fable) | **LOAD-BEARING** | C1 (reuse discriminator: refuses the flagship wallet AND admits `sortedmulti(2,K,K,X)` to steel) and C2 (engrave tail: unspendable "Full" backup) are genuine funds-class *design* defects. No spike or execution finds these; only design review does. This round is why "stop planning entirely" is the wrong prescription. |
| 2 | spec R0 r1 (fable) | **HALF** | The fold verification was owed (the fold was large). The paper-trace GREEN is where the unsatisfiable-walk assumption was ratified — "the structural defense… is that §4.5 makes the walk itself a per-stage closing gate" trusted a gate that could not run. Load-bearing on folds, harmful on closure. |
| 3 | plan R0 r0 (fable) | **LOAD-BEARING, mixed** | C1 (ms1 gate = presence, blind to the wrong-master plate), I1 (mk1 comparison plane has no implementable oracle), I2 (oracle unpinned) are real. It also minted the V19 falsehood from a stale comment. Net positive. |
| 4 | plan R0 r1 (sonnet) | **MANUFACTURED** | Real findings, but both were the author's half-folds. This round exists because the 3-minute fold at 18:51 was defective. |
| 5 | plan R0 r2 (sonnet) | **PIVOTAL** | First round to *run* a claim instead of reading it (the depth-0 round-trip), refuting a falsehood three rounds had inherited. The occasion was manufactured (another fold audit); the method upgrade is the one the whole day validated. |
| 6 | plan R0 r3 (sonnet) | **WASTE at the margin** | Competent adversarial verification of a one-fact fold, ending in a GREEN that was false at the artifact level. Proportionate in isolation; its closure semantics were the defect. |
| 7 | inherited-fact audit (opus) | **LOAD-BEARING — highest yield per finding** | 18 executed checks, 5 false/hollow, including S0's own oracle table (2 of 3 gate tests unwritable from the cited BIPs). Wrongly *positioned*: this is a build-gate, not a review, and it should have run as a script before round 1. |
| 8 | fold check (sonnet) | **MANUFACTURED, necessary** | Caught the 4-vs-9 TYPED-ONLY propagation Critical — real, and entirely self-inflicted by the 2-minute fold. |
| 9 | propagation check (sonnet) | **MOSTLY WASTE** | A CLEAN closure round — the third consecutive round about fold mechanics — whose "buildable S0→S6" verdict leaked scope and was globally false. |
| 10 | adversarial lens (fable) | **LOAD-BEARING** | A1 (the device's own warning directs the operator to a forgeable fingerprint check; review shows no keys) and A2 (version-string oracle spoofing) are threat-model findings no correctness pass produces. |
| 11 | failure-states lens (fable) | **LOAD-BEARING** | F1–F7 real on first asking; F2 ("discard the engraved plate(s)" applied to a seed plate) is operator-harm text shipped today. The report's own preamble proves the sequencing point: nine prior rounds contained zero hits for scrub/abort/interrupt/power. |
| 12 | lens-fold check (sonnet) | **MANUFACTURED, necessary** | Fourth incomplete propagation (A2's fix never reached the deliverable it quoted; the S4-test-8 renumbering). Exists because the 5-minute fold at 21:44 skipped the procedure again. |
| 13 | journeys lens (opus) | **THE load-bearing round — and the one that should have been FIRST** | The walk invoked in five gates, executable in none; no harness, no input API, no string extraction, no cards in the payload. Its findings invalidate the substrate every other lens's findings attach to. |
| 14 | comprehension lens (fable) | **LOAD-BEARING** | CH-1 (passphrase absent from the backup and unmentioned — F-132's shape, and the *spec itself already stated the fact* in §4.1 without anyone asking whether the operator learns it) and CH-2 (the gate's only visible exit silences the gate). |
| 15 | spec-coverage lens (opus) | **LOAD-BEARING** | The matrix (enumerate spec-side first, so absence is visible) independently re-found the walk gap and found the dropped §5.1 seam and 3-of-5 absent SAFE bullets. Mechanical, cheap, and should have been the *first* plan review, not the last. |

Tally: 7 load-bearing, 1 pivotal-but-manufactured, 4 manufactured by fold defects,
2 waste/false-closure, 1 half. Note the tier pattern: the three highest-yield rounds
of the day were the opus rounds with sharply-scoped *new questions* — consistent
with the standing rule that independence plus a sharp brief outweighs model tier.
The four fold-audit rounds are the cost of fold speed, not of reviewing.

---

## 3. The prescribed strategy

### What should have happened (the counterfactual, to calibrate the rule)

Five dispatches instead of fifteen, finding every Critical listed, most of them
earlier:

1. **Executability recon (author, ~1 hour, no reviewer).** Pack a card-bearing
   payload with `me sysw pack`, swap the blob, click Trace A through the browser
   emulator. Finds: no-MDMK payload, no input API, no string extraction, the
   "Engrave Bundle" title (F-159 — actually found this way), where D-1 lives, the
   "scan"-language dead ends. Six of the late findings for an hour of clicking,
   before the spec hardens around an unrunnable gate.
2. **Spec R0 (fable, one round).** Still finds C1/C2 — design findings the recon
   cannot reach. Fold with the quoted-text propagation procedure; one mechanical
   fold-check.
3. **Extended build gate on the plan (a script, no reviewer).** Everything the
   inherited-fact audit ran, as commands: fetch and grep the cited BIPs, invoke
   every named oracle CLI once (`mk bytecode` fails in five seconds), grep every
   count, diff every pin against Cargo.lock, print the fixture inventory of every
   payload the plan touches. Kills the BIP-382 row, V19, TYPED-ONLY=4, the pin
   drift, and the missing relation-(b) tool before any reviewer sees the document.
4. **One parallel lens batch (one round): journeys/executability, spec-coverage,
   adversarial+failure-states, comprehension — four agents, disjoint briefs.** The
   day's own ending proves this shape works: the last four lenses ran concurrently
   at 21:38–22:00 and produced 8C/14I in twenty minutes of wall time. Journeys is
   first among equals: its findings determine whether the gates the other lenses
   examine exist.
5. **One fold (slow, quoted-text patterns) + one mechanical fold-check. Then code.**

The answer to "which lens FIRST" is therefore: **the first lens is not a reviewer's.
It is the author attempting the user's task in the emulator before writing the
document that promises it.** Among reviewer lenses, journeys/executability first,
coverage second, correctness third — the exact inverse of what happened.

### Was there a cheaper artifact than the 600-line plan? Yes — with one honest caveat

The walking skeleton (harness + card-bearing payload + one scripted Trace-A walk to
wherever it breaks) would have surfaced the walk-harness gap on day one for about a
day of work, and its breakage point IS the D-1 reproduction S1's gate wants. But the
skeleton does **not** find spec R0 r0's C1/C2 — the discriminator and the engrave
tail are design errors that review caught and execution would have engraved. The
correct claim is not "build instead of plan"; it is **"the skeleton replaces
rounds 4–15's discoveries about the environment; it does not replace the two design
reviews."** Both were needed. Only one was scheduled.

### What to do now (this feature, from `468e044`)

1. **Stop reviewing the text.** One mechanical fold-check of `468e044` (sonnet;
   scope: did the 8C/14I each land, patterns from the four reports' quoted text, no
   fresh audit) — justified because the fold rewrote S0 and the author's fold-defect
   base rate this cycle was 4-of-9. That is the last text review.
2. **Build S0 immediately, harness-first, exactly as the rewritten S0 orders it:**
   structural confinement guard → second js-only payload with cosigner cards (Traces
   A/B/C, the `both`-slot pair) → `shTap`/`shScreen`/string extraction → the walk
   script with checkpoint records. S0 *is* the review of every remaining plan
   premise, and it reports in compiler errors and failed walks, which are cheaper
   than opus rounds.
3. **Walk Trace A until it breaks.** The break is S1/S2's content (D-1). From there,
   S1→S2 against a live walk. **At S2-close a user can do the thing** — the
   ordinary shared-origin descriptor engraves and byte-matches the pinned primary.
   That is the deliverable; everything after (S3–S5 flagship shapes, S6 hardware)
   hardens it.
4. **Plan amendments during implementation are findings-from-execution**, folded as
   one-line diffs with the gate output in the commit — they do not re-open text
   review unless they change a normative gate.
5. The single pre-irreversible fable review stays where the tiering rule already
   puts it: immediately before the first real hardware engrave/flash of S6, not
   before any more text.

---

## 4. The stopping rule

Reviewing stops and coding starts when **all three** hold:

1. **Lens closure, not finding closure.** The fixed lens set —
   {design-correctness, spec-coverage, executability/journeys,
   threat+failure-states, operator-comprehension} — has each run **once** on the
   current artifact, and blocking findings are folded (fold-check only when the
   fold is non-trivial). A 0C/0I from one lens closes that lens and nothing else;
   every GREEN names its lens. No lens repeats without a non-trivial fold touching
   its subject.
2. **Only claims remain.** When open items are text/claims rather than design
   (the existing "when review rounds stop paying" rule), they are machine-audited
   at fold time, never re-dispatched.
3. **The first gate has been executed.** The acceptance mechanism for stage 1 has
   run at least to its first checkpoint. A gate that cannot be run is an open
   Critical by definition — this clause alone would have voided both of this
   cycle's GREEN closures.

Plus one circuit-breaker, which fired twice today and was never read as a signal:
**if two consecutive rounds return findings only about the previous fold, stop
reviewing the artifact and fix the fold procedure** — the process, not the
document, is what is defective.

Applied to this feature now: all five lenses have run; the remaining risk is
executable, not readable; clause 3 is the only open item and only S0 can close it.
So: one fold-check of `468e044`, then code. Any further text round before the
harness exists is waste by this day's own evidence — eleven rounds of reading
missed what one hour of clicking finds.

---

## 5. What the author changes about folding and self-verification

1. **Patterns come from the report, never from the author.** Extend
   `fold-propagation-check.sh` to take the *report file* as input and extract its
   quoted spans itself (and match across line wraps — the "by version" pattern was
   defeated by an 80-column break). The author choosing the patterns is the
   documented failure mode; `468e044` proved the quoted-text discipline works and
   did it by hand. Make it the script's job so it is a command, not a virtue.
2. **Grep the claim before editing the sentence.** For every finding: search the
   artifact for the *fact* (the number, the BIP id, the phrase, the test name),
   enumerate every restatement site, list the sites in the fold commit. All four
   defective folds share one mechanism — corrected the cited line, left the
   duplicates. The enumeration turns the fold diff into something a fold-check can
   verify by inspection instead of re-derivation.
3. **Single-source every load-bearing fact.** Counts, pins, inventories, and
   rulings appear once, referenced elsewhere by name — and cross-references go by
   *name*, never index (`468e044` adopted names-not-indices for tests after the
   renumbering defect; extend it to facts). A fact restated three times went stale
   twice today.
4. **Quarantine inheritance.** Any claim sourced from a comment, changelog, prior
   report, or another design doc carries an explicit marker until a command output
   is pasted beside it, and the extended build gate fails on surviving markers. The
   evidence is the day's cleanest number: 16/16 gated code citations TRUE, 5-of-22
   ungated inherited facts false or hollow. The delta is the gate, not the care.
5. **Treat self-authored artifacts as inherited.** The fixture the plan leaned
   hardest on was the author's own, one day old, with a test that printed its
   inventory. Anything the plan depends on gets the same execution check regardless
   of who wrote it or when.
6. **Scoped verdicts stay scoped.** "GREEN", "CLEAN", "buildable" never leave a
   report without the lens name attached, and the controller never records a
   per-lens GREEN as a gate closure (rule 1 of §4 makes this structural).
7. **Keep what worked.** The persist-then-fold two-commit discipline held all day
   and made this diagnosis possible; the claim-vs-line habit produced the 16/16;
   the final fold's slow, gated shape is the template. The fixes above are narrow
   because most of the machinery is sound — it was pointed in the wrong order and
   allowed to fold faster than it could propagate.
