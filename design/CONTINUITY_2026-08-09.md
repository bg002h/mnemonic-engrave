# Continuity — 2026-08-09

Supersedes `CONTINUITY_2026-08-08b.md`. Read that one only for B2a-ii's history.

## Where things stand in one line

**B2b is planned and has been through three R0 rounds (3C/5I → 0C/9I → 0C/3I),
all folded. Round 3 is the last one; if it returns 0C/0I the gate closes and
Task 1 starts. No B2b code exists. Neither repo is pushed.**

| repo | HEAD | unpushed |
| --- | --- | --- |
| `mnemonic-engrave` | `24e8376` | **49 commits** ahead of `origin/master` |
| `seedhammer` (fork) | `a01b666` | **28 commits** ahead of `origin/main` |

Pushing goes through `ci/staging` — see the procedure in `CLAUDE.md`. A push that
prints "Bypassed rule violations" means the staging step was missed.

## What B2b is

`design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md` — §10.2.4's
residency-keyed idle wipe. Eight tasks. The operator-approved seam:

- **Tasks 1–3 change no operator-visible behaviour** — the Run-level test harness,
  the residency seam installed but unread, and the unwind made survivable.
- **Task 4 is the only task that can wipe.**
- Tasks 5–7 are F-93, the `RecordsResident` rename + F-87's pins, and the
  mutation runner. Task 8 is the hardware pass.

Design inputs: `SPEC_encrypted_payload_delivery.md` §10.2.4 (amended twice on
2026-08-09, both operator-approved), `CONSULT_b2b_idle_timer_design.md`,
`RECON_b2b_idle_timer_surface.md`.

## The R0 record

Every report is persisted verbatim in `design/agent-reports/`, each in its own
commit, with the fold in a separate commit after it.

| round | verdict | reports |
| --- | --- | --- |
| 0 | 3C/5I | `…-R0-round0-design.md` (opus), `…-R0-round0-test-adequacy.md` (sonnet) |
| 1 | 0C/9I | `…-R0-round1-fold-rereview.md` (opus), `…-R0-round1-residue-sweep.md` (36-agent workflow) |
| 2 | 0C/3I | `…-R0-round2-fold-rereview.md` (opus) |
| 3 | in flight | — |

**The trend is the useful part: the plan's runtime logic has been clean since
round 1.** Rounds 1 and 2 found nothing in the design — every finding was in
records, tooling and verification rows. Round 2 explicitly closed the two
structural risks by tracing them.

### Four things that cost a round each, worth not repeating

1. **A gate is an artifact and can itself false-PASS.**
   `scripts/plan-mutation-anchors.py` v1 graded the *longest* backticked span in
   an anchor cell, which for one row was a parenthetical context note rather
   than the anchor — so it reported `ok` for a token matching twice. The tool
   written to stop "a silently-failing `sed` reads like a surviving mutation"
   had relocated that defect into itself. Fixed in `c52ec58`; the rule is now
   structural (exactly one code span per anchor cell).
2. **Restating a command instead of citing it broke the green criterion twice,
   in consecutive folds.** First `GOARCH=386 …` lost `CGO_ENABLED=0` and was red
   at baseline; then the TinyGo row was hand-written as `-target=pico2-w`, which
   **does not compile** (RP2350A vs the SH2's RP2350B). Both rows now cite their
   source. Cite, don't transcribe.
3. **TIER 1 of `plan-build-gate-go.sh` is ADDITIVE.** It adds the plan's files to
   a fork copy and never removes the old body, so it reported OK on a
   configuration that cannot ship — package `gui` would not have compiled, since
   `saver` is used only inside `Run`. The hand-check now models the shipped
   configuration (body moved out, import deleted).
4. **A test seam that exists is not a test seam that fires.** F-87's recorded
   remedy named `unlockMnemonicHook`, whose single call site is on the success
   path *after* `clear(m)`. The natural test would range over `nil`, assert
   nothing, and pass with the defer deleted.

## Gates — all three must pass before any fold is committed

```sh
python3 scripts/plan-mutation-anchors.py <plan>   # 15 unique, 0 BAD, 2 unresolved
./scripts/plan-cite-gate.sh <plan>                # every citation resolves
./scripts/plan-build-gate-go.sh <plan>            # TIER 1 six whole files
```

The build gate reports two *expected* failures (`ctx.wipe undefined`,
`ctx.keepAwake undefined`) — fields added to an existing struct cannot be
expressed as whole files. The controller applies them by hand along with the
shipped-config changes and type-checks that; the transcript is in the plan's
gate-coverage section.

## What is owed, in order

1. **Round 3's verdict.** 0C/0I closes the gate — *do not keep looping for
   reassurance*, that is the standing rule.
2. **Implement Tasks 1–8.** One implementer, in a worktree, TDD, one commit per
   task. Task 1 is a prerequisite: `Run` has zero coverage today.
3. **Task 8 is operator-run hardware**, and is the first time `ctx.Done` is ever
   true on the real machine. Record results verbatim in
   `design/HARDWARE_RESULT_<date>_phaseB2b.md`.
4. **Whole-diff review to 0C/0I**, then merge.
5. **The release tag's precondition set** is now in ONE place — the section of
   that name at the end of the B2b plan. It includes B2a-ii's Task 9, F-85,
   F-92, F-98, F-100 and the `ci/staging` push.

**B2c** is a newly named successor phase: secret-residency cleanup, owning F-88,
F-90 items 1 and 3, and F-94. Those three were recorded as B2b-owned while the
plan deferred them to "own cycle" — which is no phase at all. Re-assigned
2026-08-09 in both the plan and the register.

## Follow-ups closed or filed today

- **F-99 CLOSED** — §10.2.4 row 1 did not fix *when* the warning starts. Amended
  operator-approved (`7c3a625`): the 30 s is **additive**, warning at 3:00, wipe
  at 3:30, with the rejected alternative named. Unblocked Task 8.
- **F-100 filed** — SPEC §11.5's "confirm firmware reflash preserves the blob"
  has never been run by anything and was owned by nobody. B1's hardware run
  covered four things and this was not among them; its closest statement is the
  converse.

## Reviewer tiering used, for reference

sonnet for the mechanical/test-adequacy pass, opus for each design-level round,
**fable not used at all this cycle**. The one place it is still worth spending is
a single review immediately before Task 8 — the first irreversible action on the
real machine.
