# S6a blind-spot pass 3 — is the status design over-scoped, and what disciplines the pass path

Artifact: `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md` §4.7 (six statuses + reserved seventh,
P5, the sticky-facts switch, the §4.7e projection).
Inputs: §4.7 as written; `design/agent-reports/s6a-r9-adversarial.md` (5C/4I, RED).
Question: test the controller's hypothesis ("the screens diagnose; the document should only scope"),
and state the property that would have forced the pass path to be audited equally.
Constraints honored: no fold written, no code audited; code facts are inherited from R9's traces
and are listed under WHAT I AM ASSUMING.

---

## IS THE DESIGN OVER-SCOPED

**Yes — on the failure side, and by a derivation error rather than excess caution; but the
controller's two-state alternative overshoots.** §4.7d derives the state count from *"one per
distinguishable knowledge state."* That is the correct derivation principle **for the screens**,
whose reader is interactive and can act on diagnosis — and the plan ported it onto an artifact
whose reader cannot. A stranger holding the document years later has exactly one repertoire:
rely, confirm-then-rely, or do-not-rely-and-recut. Every knowledge-state distinction that changes
neither the reader's action nor what evidence must survive is scope carried without a consumer.
The six-state set draws exactly two such distinctions (skip vs attempted-incomplete; DISAGREED vs
UNACCOUNTED), and the second is precisely the distinction P5(b) cannot enforce (R9 C-2: the
distinguishing fact is interior to `bundle.Verify`, which the plan does not own). That is not a
coincidence. The taxonomy demanded more resolution than the evidence layer records, and rounds 4
through 9 were spent forcing the evidence to fit the taxonomy instead of cutting the taxonomy to
fit the evidence. The trend reversal (folds introducing more than they close) is the signature of
that: each fold adds recording machinery to license a distinction the document never needed to
draw. However, the two-state collapse discards the one fact only the document can carry forward —
*the device holds adverse evidence about this steel* — and it makes P1 and P2 unsatisfiable
simultaneously (shown below). The right count is four, and it is not a taxonomy at all.

## HOW MANY STATES, AND WHICH

**Four: the image of a 2x2 of two recorded booleans.** No lattice, no ordering, no observation
enum, no reserved status.

- **`fullPassRecorded`** — a positive observation, written **at the success return site with the
  mode (`full`) in scope**, recording the comparison set that actually ran and matched in this
  mode. Never inferred from `res == verifyComplete` or any other verdict.
- **`adverseRecorded`** — sticky, written at any return site whose W contains a bad-plate world.
  The adverse/benign split per site is the one R9 already verified row-by-row: `:719`, `:724`,
  `:394`, `:738`, and any comparator (`bundle.Verify`) error are adverse; `:897`, `:938`, loop
  exits and refusals are benign. No sub-classification within adverse.

| cell | status | line (sketch, ASCII) |
| --- | --- | --- |
| pass, no adverse | `VERIFIED` | generated from the pass record: names exactly the comparisons this mode ran, and states what was not read (ms1 plate; ms1-typed clause only when the record contains it) |
| pass, adverse | `VERIFIED on a repeat check` | the same generated pass line, plus: `An earlier check did not pass; a later full check passed.` |
| no pass, adverse | `CHECK DID NOT PASS` | `A verification check ran and did not pass: a comparison did not match, or a plate could not be read or accounted for. Do NOT rely on this backup until a full check passes. Check again with every plate this run engraved; if this repeats, engrave a fresh set.` (+ the §4.7f scope line) |
| no pass, no adverse | `NOT FULLY CHECKED` | **the zero cell**: `These plates were not fully checked. Confirm they restore this wallet (master fingerprint below) before relying on this backup.` Covers skip, incomplete, benign failure, and anything unclassified |

**The membership test — two prongs, both required.** A distinction earns a document line iff:

1. **Enforceability:** its generating facts are values in scope at return sites in code the plan
   owns — the boundary lies ON the return-site partition, never through a callee's interior.
2. **Consumption:** across the boundary, either the stranger's required action differs, or a
   settled property (P2) forbids the merge.

Applied: **pass vs not-pass** passes both (action: rely-with-scope vs not; site: the success
return + `full`). **Adverse vs benign** passes both (action: "confirm before relying" vs "do not
rely until a full check passes"; sites: all gui-local, per R9's verified rows). **VERIFIED vs
on-repeat** fails the action prong and is forced by P2 — kept, and it is free, being a cell of the
same two bits. **DISAGREED vs UNACCOUNTED** fails prong 1 — the fact is interior to
`bundle.Verify` (C-2) — and once the adverse line states the disjunction with both actions, it
also fails prong 2. **Skip vs incomplete** fails prong 2. **`statusUnclassified`** is unnecessary:
the zero cell is already the fewest-claims line, so the reserved status's entire role is played by
the zero value, structurally.

**One deliberate change to a fold-settled argument, flagged for the controller:** §4.7d's
retro-explanation rule (a full clean pass retro-explains an earlier *pairing* failure, printing
plain `VERIFIED`) is **dropped**. That rule is the last remaining consumer of the
disagreed-vs-unaccounted split — keeping it re-imports the P5(b) obligation — and R9's I-1 already
showed one adverse class (the hand-typed ms1 divergence, about a plate nothing can ever re-check)
is not retro-explainable at all. Under four states, adverse is sticky, always; the cost is a
pass-with-note for an operator who once forgot a plate, which is honest and harmless because the
note states the resolution.

**Why two states is structurally too few.** Take the sequence *adverse observation → clean
completed retry* (all adverse classes loop, per §4.7a's measured retry condition). P1 demands a
pass line; P2 demands the adverse observation survive. Two states cannot express a
pass-that-carries-adverse: if adverse is sticky, the honest re-checker prints the conservative
line forever — P1 violated and the re-verify incentive destroyed (you can never clear the
document); if adverse is not sticky, the ms1 class is silently dropped — P2 violated. Four is the
minimum satisfying P1, P2, the incentive ruling, and monotonicity at once.

**What this dissolves in R9, mechanically rather than by patching:**

- **C-1**: the pass line is *generated from the mode-indexed pass record*; a watch-only record
  contains no ms1 comparison, so the generated line cannot mention one.
- **C-3**: no arm keys on a verdict. A future `verifyComplete` return site with no recorded pass
  observation prints the zero cell. Monotone-under-omission becomes structural: both booleans have
  safe zero values, and omission of either recording can only weaken the line.
- **I-1**: with a single adverse bit there is no second sticky fact for the pass arm to forget to
  read — the arm-shadowing space that produced it no longer exists.
- **C-4**: the undeclared observation enum reduces to two booleans whose zero values are safe by
  §4.7c's own precedent; nothing else to declare.
- **C-5**: the twelve-row enumeration is replaced by the 2x2 itself (four cells, plus the pass
  record's mode dimension), small enough that the table IS the switch and cannot drift from it.
- **C-2**: see below.

## WHAT THE COLLAPSE LOSES

Stated honestly, six-plus-one to four:

1. **The single-world condemnation.** `DISAGREED` (|W|=1, "engrave a fresh set, do not bother
   re-checking") is gone; truly bad steel now costs one redundant re-verify before the same
   terminus, and a stranger cannot distinguish damning evidence from ambiguous evidence. **Not
   funds-relevant:** the merged adverse line forbids reliance in every world `DISAGREED` covered
   and terminates in re-cut on repeat. And the sharper point: the |W|=1 claim is exactly what C-2
   proved the evidence layer cannot license — the six-state design was not *carrying* that
   distinction, it was *promising* it, and the promise's failure mode was a one-character ms1 typo
   minting "Do NOT rely on this backup" over perfect steel on a permanent document. What is lost
   is a line that could not be trusted; printing it was the defect.
2. **Skip vs attempted-and-stopped.** Forensic value only: a future auditor cannot tell from the
   document whether the operator tried. The reader's action is identical, and the operator — the
   only party the difference informs — is served by the screens at the only time it is actionable.
   If ever wanted, it is a neutral factual clause on the conservative line, not a state.
3. **The adverse sub-taxonomy's diagnostic prose.** The both-worlds sentences ("Either a plate was
   not presented, or it is not one this run cut...") return to the screens, where they already
   ship (`:963`, `:427-429`). The document keeps the disjunction and both actions, nothing else.
4. **What the TWO-state version would additionally lose — and this is the line not to cross.** The
   adverse/benign boundary is funds-relevant. "Not fully checked" printed over steel the device
   actively saw evidence against invites deferral: the operator parks the backup intending to
   confirm later, and the window in which a bad set can still be re-cut closes when the seed's
   other source (the operator's memory, the device, the operator) is gone. The screens carried the
   warning, but the screens do not survive the evening; **the document is the only artifact that
   persists to the moment the operator's attention returns**, and "a check actively did not pass"
   is a fact only it can carry there. A reader does need to know a comparison disagreed as opposed
   to simply not completing — but at the *adverse vs benign* granularity, not at the
   *which-leg-diverged* granularity.

On the settled incentive ruling (question 2): it is **directional** — running the verify must
never leave you *worse off* than skipping — and was never an equality claim. The six-state design
already prints far scarier lines on failure than on skip and was ruled compliant, because "worse
off" was ruled about capability loss (the document, the seed-derived wallet facts), not about the
line's tone. Under four states: a benign failure prints the same family as skip (fine — it removes
any gradient at all), an adverse failure prints a sterner, evidence-true line (permitted, as it
was under six), the pass lines remain the reward (the incentive to verify is intact), and
re-verifying after any failure weakly improves the document — `CHECK DID NOT PASS` upgrades to
`VERIFIED on a repeat check` — so the incentive to *re*-verify is intact too, which is exactly the
property the two-state sticky variant destroys.

## DOES IT KILL P5(b)

**Yes — by deleting its only unenforceable instance, not by hiding it.** Answering question 1's
three parts precisely:

- **P5(a) survives and still binds — deliberately.** It now binds exactly two lines (the pass
  family), and it becomes *dischargeable*: the positive line is generated from the pass record's
  comparison set, mode-indexed, so a claim with no generating observation is unwritable by
  construction (C-1's fix is this clause, at any state count). P5(a) is the clause that must NOT
  die, because the pass line is the only line whose claims exceed "we do not know."
- **P5(c) survives and becomes structural.** Both booleans zero to the safe value; the zero cell
  is the fewest-claims line; no arm can mint a strong line from an unrecorded observation because
  strong lines require the recorded pass. The "default arm unreachable" test (T17), which R9
  showed could not fail against its own mutation, is no longer needed — there is no default arm to
  protect.
- **P5(b) becomes vacuously dischargeable — genuinely unnecessary, not hidden.** The test for
  "hidden" is: does any printed line still depend on a distinction whose facts moved out of reach?
  It does not. All eleven of `bundle.Verify`'s untyped errors classify identically — adverse — at
  the gui call site, where "the comparator returned non-nil" is a value in scope. No printed line
  asserts a single member of any of the old multi-world sets; the adverse line's only claims are
  the recorded fact itself (a check ran and did not pass) and the two-world action. Restated
  enforceable as **P5(b)': every distinction the status map draws lies on the return-site
  partition of plan-owned code** — machine-checkable by extending the committed return-site sweep
  to assert totality of site → {pass-record, adverse, benign} over all sites, both halves. The
  same move rescues the single-sig half without touching package `bundle`: exit 10 is simply
  adverse (no provenance record needed), though `plan:864`'s false "single-sig is unaffected"
  sentence should still be corrected as a fact error.

So: no typed errors, no `bundle/verify.go` change, no Rust-primary question, no new package in the
blast radius — C-2 is closed by removing its consumer rather than by building what it demanded.

## THE PASS-PATH PROPERTY

**Why attention concentrates on failure paths — four mechanisms, all active in this cycle:**

1. **The findings gradient.** Re-review scope is (correctly) inherited from prior findings, and
   the R0 Critical was a failure-path defect — so nine rounds of briefs pointed at failure paths,
   and attention compounded where defects *were*, not where they *are*. The pass path was the only
   region no finding had ever pointed at, which made it the only region no brief ever scoped.
2. **Noise asymmetry.** Failure-path defects fail loud: condemning good steel produces a
   complaining operator, and the codebase's own comments warn about it. The pass path fails
   silent: its defect is false assurance, and its victim is a stranger years later who cannot file
   a finding. Review effort follows complainants; the pass path has none.
3. **Apparent atomicity.** Failure has visible structure — five verdicts, fifteen return sites —
   that invites enumeration. Success has one verdict and, today, one site, so it reads as already
   understood. R9 C-3 named it: *accidentally* faithful, mistaken for structurally faithful. The
   success row was added last because it looked like it needed no analysis.
4. **Property polarity — the root.** Every stated property polices what failure may print: P2, the
   W-truth of P4, and monotone-under-omission all bound claims *downward*. No property demanded
   that the strongest line on the page *justify its strength*. Truth-checking ("is there a world
   where this is false?") is near-vacuous for a scoped pass line in the common mode; only
   generation-checking ("which recorded observation says so?") catches pass-path defects, and no
   brief ever asked that question.

**P6 — A POSITIVE LINE IS AUDITED BY CLAIM, AT THE SAME GRANULARITY THE FAILURE PATHS ARE AUDITED
BY PATH.**

- **(a) Per-claim, per-mode decomposition.** Every positive line is decomposed into atomic claims
  (per artifact, per leg, per mode), and each claim names the recorded observation that generates
  it in **every reachable mode**. A claim lacking a generator in *any* reachable mode is a defect
  even while true in the common one. *(Catches C-1 at the desk: status 2's ms1 sentence has no
  generator when `full` is false.)*
- **(b) Entitlement, never inference.** The guard on any positive arm is the recorded positive
  observation itself; no positive line may be minted from any value a pass can merely be inferred
  from — a verdict, a site count, the absence of an error. Consequence: every future return site
  defaults to a non-positive line. *(Catches C-3, and is what makes P5(c) structural rather than
  asserted.)*
- **(c) Outcome-blind enumeration.** Any sweep, table, enumeration, or test defined over return
  paths is **total over all return paths**; filtering by outcome makes the artifact incomplete by
  exactly the filter's complement. The review brief states the pass-row count next to the
  failure-row count, and a pass-path count of zero — or of one, added last — is itself a finding.
  *(Would have flagged "the success return got one row, added last" mechanically, rounds earlier.)*

One-line form: **failure lines are audited by asking "is there a world where this is false?";
positive lines must be audited by asking "which recorded observation says so?"** The first
question is the one nine rounds asked, and it is nearly vacuous for a scoped positive line — which
is why the pass path survived nine rounds carrying the only unscoped positive claim on the page.

## WHAT I AM ASSUMING

1. **Code facts inherited from R9, not re-audited:** `bundle.Verify`'s eleven untyped errors; the
   `full`/watch-only threading; the 15-site multisig partition and 11-exit single-sig count; and
   R9's row-by-row adverse/benign soundness verdicts (`:897`, `:938`, `:794` benign; `:719`,
   `:724`, `:394`, comparator errors adverse). If any of those traces is wrong, the site
   classifications move but the 2x2 structure does not.
2. **P1 and P2 remain settled.** If the controller reopens P2 (adverse evidence need not survive
   onto pass lines), state 2 merges into state 1 and the count is three. I recommend against: the
   ms1 class is adverse evidence about a plate nothing can ever re-check.
3. **Dropping §4.7d's pairing retro-explanation is a controller decision**, flagged above as the
   one fold-settled argument this pass proposes to change. It is what lets the
   disagreed/unaccounted split die.
4. **The reader can check a restore against the document's own seed-derived facts** (master
   fingerprint, first addresses), so "confirm they restore this wallet" is executable from the
   page alone. The conservative lines should point at those facts explicitly — without that
   pointer, "confirm they restore" admits a restores-to-the-wrong-wallet reading.
5. **Stickiness scope** (per flow run; what survives backing out of the whole flow) is unchanged
   from the six-state design and stays whatever the plan already ruled — the collapse neither
   fixes nor worsens it.
6. **F-198 non-separability is honored:** the status line remains, always renders, leads the page;
   §4.7f's scope line attaches to `CHECK DID NOT PASS` (and harmlessly could to the repeat-check
   line); nothing here reopens the always-render ruling.
7. **The mode-indexed pass record is work that exists at any state count** — the collapse removes
   the failure taxonomy's recording burden, not C-1's.
