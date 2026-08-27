# CONSULT — P0 scope: the brief's forbidden follow-ups vs the plan's §4 rows

**Consulted 2026-08-27**, standing in for the operator on scope only. Read-only
pass over `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` and
`design/FOLLOWUPS.md` in this worktree. No code audited, no plan quality
reviewed, no new work proposed.

## Verification of the implementer's characterisations

All checked against the two files and **all are accurate**:

- F-264 — owning phase **P0** (`FOLLOWUPS.md:11711`); row 5's gate says
  "Blocked on F-264" and §6 condition 9 is titled "F-264 fixed, because the
  remedy gate cannot pass otherwise", adding that rewording "cannot make the
  gate green" and would stall everything after it.
- F-265 — owning phase **P0** (`:11747`); row 7 is "pin the exit digit at ALL
  FIVE sites"; §6 condition 10 requires it "with a step that does it".
- F-266 — owning phase **P0, gating**, `#critical` `#shipped` (`:11776`); row
  6's gate — "no secret material in stderr for any argv carrying it AS A
  TOKEN", generated cross-product — IS this fix.
- F-270 — owning phase **P0** (`:11914`); it is a clause of row 6's own cell:
  "and the donor's post-parse arm gets the same normalisation, F-270".
- F-267 — **post-P0, documentation, residue** (`:11836`) — brief and plan agree.
- F-268 — **P3** (`:11857`), restated in plan §7 — brief and plan agree.
- F-259 — **P0** (`:11460`), fixed by row 4 / condition 6, and absent from the
  brief's forbidden list.

One immaterial nit: the "15 of 24 measured argv shapes" figure is not in the
F-266 entry (which shows a 6-row sample plus "pass: leaks on 7 of 8 shapes"
and says the list is not exhaustive); the direction and severity are as stated.

**One fact the question omitted, and it decides the reading.** F-266's entry
contains, verbatim: **"OPERATOR RULING 2026-08-27: deferred, not fixed now —
'Nobody cares about leaks, we can file them for fixing later.' No emergency
fix, no yank of v0.7.0."** — followed immediately by: **"It is still what
condition 8 is FOR, and P0 fixes it as a side effect... So this is deferred in
the sense of not interrupting the cycle, not in the sense of unowned."** The
operator's recorded deferral means *no out-of-band emergency fix*; the P0 row
was always the delivery vehicle. The brief's list is that ruling transcribed.

## THE DECISION

- **F-264: BUILD.** Row 5's gate literally cannot pass without it, §6
  condition 9 exists to say exactly that, and skipping it stalls rows 6–11
  under "no step begins until the previous is green" — leaving P0 unclosable.
- **F-265: BUILD.** The brief's own carve-out — "except where a row explicitly
  pins the digit" — is row 7 described exactly, so even the strictest literal
  reading of the brief permits this; §6 condition 10 requires it.
- **F-266: BUILD.** Row 6 IS this fix, it is tagged P0-gating and `#critical`
  `#shipped`, and the FOLLOWUPS entry's own operator ruling says P0 fixes it
  as a side effect of the spec-normative pre-parser guard — "deferred" there
  means no emergency fix on v0.7.0, not skip the row. The cost asymmetry
  settles any residual doubt: a wasted cycle is recoverable; shipping a known
  secret-to-stderr leak past its owning, gating phase is not a defensible
  reading of any instruction.
- **F-270: STOP is impossible without also not building row 6 as written —
  BUILD.** It is a named clause of row 6's cell (the donor's post-parse arm
  gets the same normalisation), so it ships as part of row 6, not as separate
  follow-up burndown; do not build it as a free-standing fix outside that row.

(F-267 and F-268 were not asked, and correctly so: leave both. They are the
list's real teeth.)

## What the brief's list actually meant

The list means **"FOLLOWUPS.md is not your work queue — build the plan's rows,
and nothing beyond them."** Three signals converge. First, the F-265 carve-out
proves the author knew rows embed follow-up work and intended row-scheduled
work to proceed — the exception describes row 7 exactly. Second, F-259's
omission shows the author excluded the one follow-up they had reconciled
against the table (row 4 fixes it); the four conflicts are an incomplete
reconciliation of the same kind, an enumeration of open P0-tagged entries not
re-derived against the twelve rows. Third, the same brief calls the §4 table
"the ordering of record" and says "a plan that cannot be executed is the
finding" — a reading that silently drops four rows and leaves §6 conditions 8,
9 and 10 permanently unsatisfiable makes the brief contradict itself, while
the "rows govern; no extra-row burndown" reading keeps every sentence of it
true and matches the operator ruling recorded in F-266's entry. Where the two
readings genuinely diverge is only F-267/F-268-shaped work — free-standing
follow-up fixes no row schedules — and there the prohibition binds fully.

## The uncommitted row 5 work

**COMMIT IT.** F-264 is BUILD, the gate is "run the emitted recipe under a
real interactive zsh and assert the entry is gone", and the work is built and
green. Reverting it would re-open a P0-owned item to satisfy a reading of the
brief that the brief's own text (the ordering-of-record clause, the F-265
carve-out) rules out.

## Standing note for the record

This consult is the "stop and report it" the brief demands for a row that
conflicts with an instruction — the conflict is real, is now recorded here,
and the resolution is: **the rows govern; the list forbids work outside them.**
