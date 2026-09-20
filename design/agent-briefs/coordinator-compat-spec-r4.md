# Brief — r4: is the coordinator-compat design implementable yet?

**Artifact:** `design/DESIGN_coordinator_compatibility.md` at `3b374675`.
**Input:** `design/agent-reports/coordinator-compat-spec-r2-verify.md`
(C-1 PARTIAL; 1C/7I/5M/1N new; implementable **NO**).
`git diff 4882d2af..3b374675` is exactly the fold.

## Two questions

**A. Did the fold close r3's findings?** Verdict per finding, checking
**clauses** — the recurring failure across three folds has been taking a fix's
conclusion and dropping a named clause of it. The fold claims to have closed:
C-1's remainder (the port is now "port + two named extensions"), `root` added
to `Skeleton`, the duplicate `key_path` removed, the §3 self-contradiction
about harnesses, the two incompatible key spellings, the key's serialized
form, `key_partition`'s absence rule and "derivation", `MeasuredAt`, and all
five Minors plus both Nits.

**B. Is it implementable NOW?** Walk plan 1 (section 5 steps 1-2) as an
implementer who may invent nothing. r3's blockers were: `fp_partition` had no
derivation, `root` missing, no serialized key form, and two key spellings in
one document. Are they closed, and is there a fifth?

If the answer is still NO, **say plainly whether the remaining gaps are the
kind a design should close or the kind an implementation plan should** — this
is a design doc, and there is a point past which further detail belongs in the
plan rather than here. Three rounds have each found real defects; the question
of whether a fourth is the right instrument is a fair one to answer.

## Already verified by the controller

- `md/policy_shape.go:239-245` — `branchOf` builds `keys := map[uint8]struct{}{}`,
  writes `br.Keys = len(keys)`, discards the map. Confirmed.
- `md/policy_shape.go:33-39` — `KeyPathKind` is `None`/`NUMS`/`Spendable`,
  three-valued, no unspendable-xpub. Confirmed.
- `606ab180` committed `harnesses/liana/` and the v15.0 re-measurement
  (289/289 verdicts identical), which is what made the §3 prose false.
- Every type the design names is defined except `PolicyShape`, deliberately
  the fork's, awaiting port.

## Settled — do not re-litigate

The six operator rulings; the architecture's family; the three-plan split; the
skeleton key; no device-side freshness clock; `md descriptor` never refusing.

## Rules of evidence

Read the real repos (`descriptor-mnemonic` `b2c5d693`, `seedhammer` `7b6f2fb`).
Every finding names the state and the wrong outcome, or the decision an
implementer cannot make. Do NOT modify tracked files; leave trees clean.

## Output

Severity per project standard. **FINAL ACTION: write the report to
`design/agent-reports/coordinator-compat-spec-r4.md`** and return ONLY a
one-paragraph summary, that path, the per-finding tally, your answer to B
(including the design-vs-plan judgement if still NO), and C/I/M/N counts for
anything NEW.
