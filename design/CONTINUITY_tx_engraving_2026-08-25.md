# CONTINUITY — transaction engraving, after the experiment

**Resume point as of 2026-08-25.** Supersedes `CONTINUITY_goal1_P1_2026-08-25.md`,
which describes a plan that has since been **retired**.

## State in one line

**The 2,062-line P1 plan is retired.** `design/ACCEPTANCE_engrave_transaction.md`
(335 lines) is the normative surface. **P0, P1 and P2 are done**; **P3a is running**;
**P3b (the live journey walk) needs the operator**; **P4 (S0 hardware) is the
operator's and gates the release**.

## How we got here — the experiment

Two agents implemented the same target in isolated worktrees: **arm A** with the
spec, the plan and ten review rounds; **arm B** with a one-paragraph brief plus
measured QR findings and the design surface REMOVED from its worktree.

- A wrote **~8,828 lines**; B wrote **~3,596**. Their wire formats share nothing.
- B **proved the stated requirement** — independent ZXing decode round-trip, and a
  Rust-packed payload decoded by the Go side. **A never decoded a symbol it
  produced.**
- A's own verdict: *"the plan earned its size, unevenly — ~600 of 2,060 lines
  prevented three defects I would have shipped. The other ~1,400 are a
  review-process record; I read them and acted on none."*
- **Base = arm B**, with named grafts from A. Full write-up in
  `design/EXPERIMENT_plan_vs_brief_2026-08-25.md`.

## THE DESIGN CHANGE THAT MATTERS — E17 replaced

The plan carried a 32-byte `wtxid` to catch a witness-stripped transaction, and
declared the fully-consistent case unclosable *"because it IS an honest
witness-free transaction"*. **That is false.** An honest witness-free transaction
is a LEGACY transaction, and a signed one has non-empty scriptSigs. The
discriminator is a predicate over the BODY, per input:

> **every input carries a non-empty scriptSig OR at least one witness item**

It catches both stripping cases, carries no field, and needs no framing. Ten
review rounds missed it. Now live in Rust (`sysw::tx::every_input_signed`) and Go
(`mt.Tx.EveryInputSigned`), on **both** the `tx:` and `mt1` classes.

## Phases

| | state |
| --- | --- |
| **P0** | DONE — predicate, both operator rulings folded, CI pinned via git rev |
| **P1** | DONE — six grafts; delivery ceiling 8191 → **32,734** both sides |
| **P2** | DONE — 101 items classified (32 MET / 18 MET-DIFFERENTLY / 45 NOT-MET / 6 SUPERSEDED); plan retired |
| **P3a** | RUNNING — 15 gates incl. the end-to-end UI walk |
| **P3b** | **NEEDS THE OPERATOR** — the live journey walk, plus three held gates |
| **P4** | **THE OPERATOR'S** — S0 hardware, ~a week out, loud, time-of-day constrained. Gates the release and nothing earlier |
| **P5** | ship — one whole-diff opus review, ci/staging ritual, tag |

## THE THREE GATES HELD FOR THE OPERATOR

- **G-P3.10** two byte-different transactions can share a derived txid and present
  as two identical picker rows. Which one do you engrave?
- **G-P3.14** the review screen shows no outputs, amounts, fee, locktime or
  network. `mt.Tx` carries none of those fields — a **parser** change, not a
  screen change.
- **G-P3.19** `me tx` exits 0 on an unsigned transaction that `pack` then refuses
  at exit 4 one step later.

Each may legitimately be ruled *not our concern in writing*. That is the sheet's
own phrasing and the operator's standard: **reasonable effort funds safety, not
perfection.**

## Branches

`p0/tx-engraving` → `p1/grafts` → `p2/acceptance` → `p3/ui-walk`, in both
`mnemonic-engrave` and `seedhammer`. `mt` has `p1/mt-inspect-raw`. **Nothing
pushed.** The experiment arms are `exp/tx-plan-driven` and `exp/tx-brief-driven`
under `_experiment/`.

## The lesson worth carrying

**Mutation testing caught the SAME first-draft gap twice** — once in Rust, once in
Go, in the same predicate: a whole-transaction test passes a mixed transaction
with one unsigned input. I wrote a *comment* about that case both times and only
wrote the *vector* after the mutation proved I needed it.

**And read the log, never the exit code.** A truncated `go test ./...` exited 0
with the 932-test `gui` package never having run.
