# Brief — re-review of the coordinator-compatibility design, second fold

**Artifact:** `design/DESIGN_coordinator_compatibility.md` at `23c51d8e`.
`git diff 4772aa35..23c51d8e` is exactly the fold.

**Input:** `design/agent-reports/coordinator-compat-spec-opus.md`
(4C/10I/5M/2N; its answer to implementability was **NO**).

## Three questions

**A. Did the fold close each finding?** Verdict per finding (C-1..C-4,
I-1..I-11, the Minors and Nits): FIXED / PARTIAL / NOT-FIXED, with the design
text that settles it. **Check clauses, not conclusions** — the previous fold's
documented failure was folding the conclusion of a fix and dropping a named
clause of it in seven places. For each finding ask: did every clause land?

**B. Is it implementable NOW?** Re-run the walk: plan 1 (section 5 steps 1-2)
as an implementer who may invent nothing. The previous answer was NO on four
blocking gaps — `Skeleton` undefined, the key uncomputable in Rust, the
template-only verdict undefined, and step 1's gate needing a step-2 binary.
Are those closed, and is there a fifth?

**C. Did THIS fold introduce a defect?** Both previous folds did — one dropped
clauses, one promoted an aside into normative text and deleted real work from
the plan. The new material is where nobody has looked: §1A's type definitions,
the D1-D4 disagreement table, the `kind#class` definition, the
absent-fingerprint rule, the `md compose` / `md descriptor` split, the xtask,
and the staleness plan-ownership table.

Push hardest on **§1A**, since it is wholly new and the data model hangs off
it. In particular: is `Skeleton` sufficient for the rules that must read it —
take `composerLianaOutsideModelClass`'s nine classes and check each is
computable from the struct as defined? And is `kind#class` now unambiguous
enough that two implementers produce the same key?

## Already verified by the controller — do not re-derive

- Every citation the fold added was re-grepped and resolves:
  `canonicalize.rs:168`, `render.rs:159`, `:171`, `encode.rs:51`,
  `cmd/compose.rs:5`, `decompose/walk.rs:55`, `compose/mod.rs:274`,
  `validate.rs:449-458`.
- Every type the design names is now defined except `PolicyShape`, which is
  deliberately the fork's existing `md/policy_shape.go` awaiting port.
- The retraction is measured: the per-path decomposition exists nowhere in
  Rust; the branch split is Go-only.

## Settled — do not re-litigate

The six operator rulings; the architecture's family; the three-plan split; no
device-side freshness clock; the skeleton key over `PolicyShape`.

## Rules of evidence

Read the real repos (`descriptor-mnemonic` `b2c5d693`, `seedhammer` `7b6f2fb`);
do not reason from prose about what code exists. Every finding names the state
and the wrong outcome, or the decision an implementer cannot make. Do NOT
modify tracked files; leave the trees clean.

## Output

Severity per project standard. **FINAL ACTION: write the report to
`design/agent-reports/coordinator-compat-spec-r2-verify.md`** and return ONLY
a one-paragraph summary, that path, the per-finding tally, your answer to B,
and C/I/M/N counts for anything NEW.
