# Continuity — 2026-08-16c: S6a's R0 gate is **CLOSED — GREEN at R17**. Implementation may begin.

Supersedes `CONTINUITY_2026-08-16b.md`. Read this one.

---

## ▶▶ THE GATE IS CLOSED — READ THIS BEFORE THE START-HERE BELOW

**R17 returned GREEN, 0 Critical / 0 Important** (`design/agent-reports/s6a-r17-closing-fold-verify.md`).
**The R0 loop is CLOSED. Do NOT run another review round for reassurance** — that
is the explicit rule, and seventeen rounds is where it applies hardest.

**What rounds 15–17 were, and what they say about where the risk now is:**

| round | lens | result |
| --- | --- | --- |
| R15 | sonnet verify + attack | RED 0C/1I — in the *controller's own* fold |
| R16 | sonnet fold verify | RED 0C/1I — in the *controller's own* added paragraph |
| R17 | sonnet closing fold verify | **GREEN 0C/0I** + 1 Nit, fixed at `b324023` |

**Both R15 and R16 findings were in text the controller wrote on its own
initiative, not in anything a reviewer asked for.** No round has found a *design*
defect since the four-state rewrite. R16's was a real **G2** violation — the
controller had called all ten non-success single-sig returns "adverse", which
would have printed *"A verification check ran and did not pass"* after a benign
exit that never read a plate. §4.7b classifies the byte-identical multisig site
**benign**; that precedent now binds single-sig in the plan.

**So: the plan is not the fragile artifact. Edits to it are.** Anything written
during implementation re-earns the gate the same way.

### ▶ NEXT ACTION: implement

Worktree `/scratch/code/shibboleth/wt-s6a`, branch `s6a-singlesig-truth` (empty,
ready). **ONE implementer subagent, TDD, UC OFF** — executing a GREEN plan is
transcription, and parallel attempts produce reconciliation work, not coverage.
Follow §4.8's nine-step build order.

**STEP 1 IS A GATE, NOT A TASK.** It produces **two** artifacts, reviewed
*together* before step 2 and before any other code:

1. the single-sig **eleven-exit → `verifyRecord` mapping** — 11 exits = 10
   `return`s (lines 69, 78, 90, 98, 112, 117, 125, 130, 138, 146) **plus the
   implicit fall-through at `gui/singlesig_verify.go:149`, which is the ONLY
   success exit and is not a `return`**. An implementer told "write the record at
   each return site" writes ten adverse records and never writes the pass record.
2. the **`suppliedCosigners` expression**.

**§4.8 now states what a reviewer checks each against** — it did not, and a gate
with unstated acceptance criteria passes anything. Each mapping row names which
of `verifyRecord`'s two booleans the exit writes (or **NEITHER**, the right
answer for a benign exit), carries §4.7b's adverse/benign bit, and names **no**
`verifyStatus` value: the status is derived once, downstream, by §4.7a's switch.

**Measured, so do not re-derive:** single-sig has exactly **one** call site
(`gui/singlesig.go:132`) — no stub, no `singleSigVerifyFn` indirection, no test
callers — unlike multisig's twelve-plus-stub. `statusVerifiedOnRetry` is the one
state unreachable from inside the eleven exits (no retry loop;
`gui/singlesig.go:131` is a one-shot `if`).

Then: whole-diff adversarial execution review → merge → push.

---

## ▶ START HERE — for a fresh session while the loop was still OPEN (historical)

**Step 1. Read the goals**, which govern what counts as a defect:

    sed -n '/## 0.1 WHAT THIS CYCLE/,/## 1. MEASURED FACTS/p' design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md

**G1** never misdescribe what was engraved. **G2** never vouch for plates the
device has evidence against. **NG1 — reporting the verification's epistemic
status — is an explicit NON-GOAL**: nobody asked for it, and every Critical of
this cycle came from it. A finding that would expand it is **out of scope by
default**, even when correct.

**Step 2. Read the newest review verdict:**

    ls -t design/agent-reports/s6a-r*.md | head -3
    head -20 "$(ls -t design/agent-reports/s6a-r*.md | head -1)"

**Step 3, branch on it:**

- **GREEN (0C/0I) → the R0 loop is CLOSED. Start implementing.** Do **not** run
  another round for reassurance. Go to §4.8's nine-step build order.
  **Step 1 of that order is a GATE, not a task:** produce the single-sig
  11-exit → `verifyRecord` mapping and have it reviewed *before* any other code.
  Work in `/scratch/code/shibboleth/wt-s6a` (branch `s6a-singlesig-truth`,
  empty, already created). One implementer, TDD.

- **RED → fold it, in this order, which is what has been working:**
  1. Commit the report **verbatim, in its own commit**, first.
  2. **Fold from the file, never from memory.** Re-read/re-run every mechanism
     claim. This is where most fold defects came from.
  3. **Run all five gates** (below), then `plan-fold-sweep.sh --terms` naming
     every phrase this fold superseded. **Check headings, not just bodies.**
  4. Dispatch the **cheap sonnet claim-verification pass** before any expensive
     round. It has found something every single time.
  5. **If the round returned a Critical, or an Important the reviewer labelled
     JUDGEMENT → dispatch an OPUS BLIND-SPOT PASS before folding** (see Step 4).

**Step 4. THE ESCALATION RULE** (agreed 2026-08-16). On a Critical, or a
JUDGEMENT-labelled Important, ask **opus** ONE question before folding:

> *Here is the design and the properties it claims. What failure mode do they
> collectively fail to constrain?*

Then **fold its answer yourself** — the escalation buys judgement, not
transcription. Reviewers label each finding MECHANICAL or JUDGEMENT so the
trigger does not depend on the controller's own classification.

**The escalation target is OPUS, not fable** (user directive, 2026-08-16: *"we
will not use fable for final review"*, *"we will use sonnet for the next review
of mechanical fold"*). This closed the last carve-out that reserved fable for a
single pre-irreversible gate. **Opus is now the top of the ladder — including
for the final review before the hardware flash.** Sonnet keeps the mechanical
and fold-verification rounds, which is where this cycle's findings actually live.

**Step 5. THE SIX GATES.** Run all six on any plan edit:

    ./scripts/verify-returnsite-sweep.sh <plan>   # every verdict return site is rowed
    ./scripts/plan-cite-check.sh        <plan>    # every path:line resolves
    ./scripts/plan-glyph-check.sh       <plan>    # strings the display font can draw
    ./scripts/plan-table-check.sh       <plan>    # no table row lost a cell
    ./scripts/plan-wiring-check.sh      <plan>    # nothing declared and left unwired
    ./scripts/plan-fold-sweep.sh <plan> --terms '<superseded phrase>' ...

**Every one prints what it does NOT cover, and two have DEMONSTRATED blind
spots** — `verify-returnsite-sweep` sees multisig only until single-sig gains a
verdict, and `plan-wiring-check` still passes a name mentioned in two sections
that never reached the section the document calls its authority. **Read those
lines; do not treat a green gate as a proof.**

Each prints what it does **not** cover. **Never cache their numbers in prose** —
that rotted twice; the commit message carries each fold's measured output.

**Step 5b. WHAT THE OUTGOING CONTROLLER WAS UNSURE OF — ALL FOUR ARE NOW
RESOLVED.** Kept because the resolutions are load-bearing, not because they are
still open:

1. **RESOLVED — fixed.** §4.8 now states acceptance criteria for both step-1
   artifacts (commit `4f40f1f`, corrected by R15/R16 at `4c40973`/`6a2198f`).
2. **RESOLVED — measured, and it was a trap.** The write site exists, but the
   only success exit is the **fall-through** at `gui/singlesig_verify.go:149`,
   not a `return`. Now stated in the plan.
3. **RESOLVED — measured.** Single-sig has exactly **one** call site
   (`gui/singlesig.go:132`), no stub, no indirection, no test callers.
4. **RESOLVED — R15 Part 3.** T27 is genuinely schedulable at step 7, and step 7
   is its **only** valid slot. R15 also filed T27's non-vacuity risk, folded at
   `4c40973`: the self-multisig fixture yields `open == 0`
   (`gui/multisig_build.go:96`) hence `suppliedCosigners == 0`, on which T27
   passes while asserting nothing. Name or build the fixture at step 7.

The original wording of the four doubts follows, for the record:

1. **Is build-order step 1 specified well enough to be REVIEWABLE?** It now
   carries two artifacts — the single-sig eleven-exit → `verifyRecord` mapping,
   and the `suppliedCosigners` expression. The plan says both are "reviewed
   before step 2" but does not say *what a reviewer checks them against*. A gate
   whose acceptance criteria are unstated is a gate that will pass anything.
2. **Does single-sig actually have somewhere to write `suppliedCosigners = 0`?**
   R14 said its one true success exit "can trivially write 0" — but
   `singleSigVerifyFlow` is `void` today and gains the out-parameter only at step
   1. Confirm the write site exists *after* step 1, not just in principle.
3. **Are there `singleSigVerifyFlow(` call sites that gain a parameter too?**
   Two rounds running, the call-site list was incomplete: R13 found eight beyond
   the four named, R14 confirmed those twelve. Nobody has grepped the *single-sig*
   side with the same rigour.
4. **T27 asserts on "the rendered line of both flows" — can it?** The multisig
   half needs a full walk to a rendered document. §5.1 says pure-function
   assertions do not satisfy that standard. Verify T27 is schedulable where it
   now sits (step 7) and not merely asserted there.

**Step 6. Do not re-litigate** the decisions in `design/agent-reports/`: the
goals (§0.1), C-1's remedy, ONE PIECE, the four-state set, and cycle scope.

---

## THE ONE THING TO KNOW

**S6a is a PLAN, not an implementation.** Zero lines of code have been written,
deliberately: the R0 gate forbids code until the plan closes at 0 Critical /
0 Important, and it has not. The fork's `main` is **unchanged** at `b8a23bf`
since S5 merged.

Everything produced so far is design: one plan, thirteen persisted review
reports, follow-up filings, and two new gate scripts.

## STATE

| repo | branch | head | vs origin |
| --- | --- | --- | --- |
| `mnemonic-engrave` | `master` | see `git log -1` | pushed through `ci/staging`; check SATISFIED, not bypassed |
| fork `/scratch/code/shibboleth/seedhammer` | `main` | `b8a23bf` | in sync, **untouched this cycle** |
| fork worktree `/scratch/code/shibboleth/wt-s6a` | `s6a-singlesig-truth` | `b8a23bf` | **0 commits ahead — empty, ready** |

Two S5 worktrees (`wt-s5`, `seedhammer-s5`) are fully merged and clean; safe to
remove, nothing depends on it.

## WHAT S6a IS

The single-sig engrave flow takes a BIP-39 passphrase, derives from it, engraves
**nothing about it**, labels the result **"Full (seed + keys)"**, and prints a
document that never mentions a passphrase. The words alone restore a *different*
wallet, with no error. **Permanently unspendable, and the paperwork vouches for
it.**

**Verified by bytes, not by reading the call graph** — same mnemonic derived
twice:

| artifact | bare | + passphrase | |
| --- | --- | --- | --- |
| `ms1` | `ms10entrsqqq…cj9sxraq34v7f` | identical | **words only** |
| master fp | `73c5da0a` | `fc60c6df` | differs |
| `mk1` / `md1` | — | — | **differ — passphrase-bound** |

Scope: **F-198** (Critical), **C-1** (Critical, found in review — the document
prints even after the device says the plates do NOT match), **F-197**, **F-195**,
**F-202**. Plus both multisig paths, which share C-1.

## WHERE IT STANDS — TWELVE ROUNDS, and what each lens was for

| round | lens | result |
| --- | --- | --- |
| R0-A / R0-B | adversarial funds / executability | RED — 1C 4I · 0C 1I |
| R1-A / R1-B | fold-vs-findings / adversarial on fold | RED — 0C 1I · **2C** 3I |
| R2-A / R2-B | adversarial on fold / spec coverage | RED — **2C** 5I · 0C 1I |
| R3-pre · R3-A / R3-B | cheap verify / adversarial / comprehension | DIRTY · RED **2C** 5I · 0C 8I |
| R4-pre · R4 | cheap verify / adversarial | DIRTY · RED **1C** 2I |
| R5-pre · R5 · R5-B | cheap verify / adversarial / **disclosure** | DIRTY · RED **1C** 4I · **GREEN** |
| R6-pre · R6-B | cheap verify / **reader comprehension** | DIRTY · RED 0C 3I |
| R7 · R8-pre | adversarial / cheap verify | RED **3C** 3I · DIRTY 3 structural |
| R9 | adversarial | RED **5C** 4I |
| R10 | **goal conformance** | RED 1C 1I + 3 filed |
| R11-pre | cheap verify | DIRTY 6 stale, 1 structural |
| R12 | closing adversarial | RED 1C 2I — **and §4.1–§4.6, G1's actual fix, audited fresh: CLEAN** |
| R13 | sonnet verify + attack | RED **1C** 1I — a field declared and never wired |
| R14 | sonnet verify + attack | RED 0C 3I — all propagation into authoritative tables |
| R15 | sonnet verify + attack | RED 0C 1I — **in the controller's own fold** |
| R16 | sonnet fold verify | RED 0C 1I — **in the controller's own paragraph; a real G2 violation** |
| R17 | sonnet closing fold verify | **GREEN 0C 0I** — the R0 loop CLOSES |

Every report is in `design/agent-reports/s6a-*`, each persisted **verbatim in its
own commit BEFORE** the fold responding to it, so `git diff <report>..<fold>`
means something.

## THE SIX THINGS THIS CYCLE PAID FOR

1. **THE GOAL WAS WRONG, and nine rounds could not see it.** Three goals ran
   under one name: G1 (never misdescribe what was engraved — the real defect,
   **zero Criticals in eleven rounds**), G2 (never vouch against your own
   evidence), and **NG1 — report the verification's epistemic status — which
   NOBODY ASKED FOR and which produced EVERY Critical of the cycle.** The
   operator found it by asking "maybe the goal is simply wrong?"
2. **The structural reason NG1 was unaffordable, and it generalises: G2 is a
   PROHIBITION, NG1 an OBLIGATION.** "Never claim more than you know" needs one
   conservative default and no enumeration. "Always say exactly what you know"
   needs a complete correct partition of everything observable — which two
   successive properties (P4, then P5(b)) failed to deliver. **A prohibition
   fails safe; an obligation fails OPEN.**
3. **A non-goal only holds if it is ENFORCED against future findings.** NG1
   arrived by review, one *correct* increment at a time. §0.1's guard: a finding
   expanding epistemic reporting is out of scope by default, **even when
   correct** — correctness is not the test, goal membership is.
4. **The folds were the weak artifact, not the plan.** Thirteen carried defects,
   overwhelmingly incomplete propagation — the fact corrected where the reviewer
   pointed and left standing three sections away. **Reading the diff cannot find
   this**; only a whole-file sweep can.
5. **Delete, don't patch, when a structure keeps generating defects.** The
   severity lattice, then the six-state knowledge partition, both deleted rather
   than repaired. The replacement — a 2×2 of two recorded booleans — makes the
   old Criticals *structurally impossible* rather than fixed.
6. **Closure is LENS-closure.** Disclosure, reader-comprehension and
   goal-conformance were each first-time questions, and each found what
   re-running the others could not. **Stop when out of QUESTIONS, not when a
   round comes back clean.**

## WHAT THE LAST FOUR ROUNDS MEAN

**Since the four-state rewrite, no round has found a defect in the DESIGN** —
every finding has been transcription: a field declared and not wired, a call-site
list too short, a test scheduled to a step that cannot run it, a table the fold
never reached. R12 additionally audited **§4.1–§4.6 — G1's actual fix — fresh,
and found it CLEAN**, including driving the real document to prove a
~310-character line renders complete on its own page.

**So the remaining risk is the controller's accuracy, not the design's
soundness.** That is why the cheap sonnet tier keeps paying and why the six gates
exist. Two of them were written *because* a reviewer found a class the others
could not see, and one of those has a demonstrated false negative that
strengthening did **not** close — recorded in its header rather than papered over.

## THE DESIGN, IN ONE PARAGRAPH

The restore document always renders and carries **exactly one status line**,
chosen from **four states** — the 2×2 of two *recorded* booleans:
`fullPassRecorded` (written at the success return, carrying the mode via
`passRecord{full, legs}`) and a sticky `adverseRecorded`. The `default:` arm is
the zero cell, so **monotonicity is structural**: an unrecorded fact cannot set a
bit, and an unset bit only moves toward "not fully checked". One seam,
`buildVerifyStatusLine(rec verifyRecord) string`, with a `verifyRecord`
out-parameter that leaves the verdict return — and the three shipped tests
pinning it — untouched.

## DECISIONS ON DISK — do not re-litigate

- **C-1's remedy** (`s6a-c1-verify-tail-decision.md`): the restore document
  ALWAYS renders and carries exactly one of five status lines. Never gated —
  because any gate keyed on a failed verify makes the honest path worse than the
  lazy one, and the operator learns to skip the check to keep the document.
- **ONE PIECE** (`s6a-split-decision.md`): F-198 is **not separable** from C-1.
  F-198 alone turns a silent document into a *vouching* one while the
  failed-verify print path still exists — locally worse than today. *Safety that
  depends on a downstream gate is coupling, not separability.*
- **Scope** (`s6a-scope-and-design-decisions.md`).

## WHAT IS NEXT

1. ~~R0 review loop.~~ **DONE — GREEN at R17.** Do not re-loop for reassurance.
2. **Implement**, in `wt-s6a`, one implementer, TDD, **UC OFF**, following §4.8's
   **nine-step build order**. Step 1 is a gate, not a task: **both** step-1
   artifacts — the single-sig 11-exit → `verifyRecord` mapping (**including the
   fall-through at `gui/singlesig_verify.go:149`**) and the `suppliedCosigners`
   expression — reviewed together, against §4.8's stated acceptance criteria,
   before any other code.
3. Whole-diff adversarial execution review → merge → push.

**Reviewer tiering, as of 2026-08-16: sonnet for mechanical/fold verification,
opus for design-level adversarial AND for the final pre-irreversible review.
`fable` is no longer a reviewer tier at any stage** (user directive; the old
carve-out reserving it for the pre-flash gate is closed — see `CLAUDE.md` and the
escalation rule in step 4 above).

Then **S6b**, the single compressed pre-flash cycle (operator directive
"compress", 3 cycles → 2): F-199, F-204, and the passphrase plate — see
`REQUIREMENTS_s6b_pre_flash_cycle.md`. Title decided: **`PASSWORD REQUIRED`**
(17 chars; `MaxTitleLen` is 18 and truncation is SILENT — "PASSPHRASE REQUIRED"
is 19 and would engrave as `PASSPHRASE REQUIRE`).

Then the hardware flash.

## TOOLING ADDED THIS CYCLE

    ./scripts/plan-cite-check.sh  <doc>   # every path:line resolves + prints the line
    ./scripts/plan-glyph-check.sh <doc>   # operator strings vs the display font

Both **print what they do NOT cover**, and both are proven with **positive
controls**. Current: 93/93 citations, 49 strings, exit 0.

**Known gap:** the glyph gate's heuristics only reach blockquotes and backtick
spans ≥40 chars. On `REQUIREMENTS_s6b_*` it scanned **zero** strings and still
exited 0 — *a 0-scanned pass is not a clean pass*. Check such docs directly.

## TOOLCHAIN

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"
    cd /scratch/code/shibboleth/seedhammer
    nix develop --command go test ./... -count=1        # baseline: EXIT=0, stderr empty
    nix develop --command ./cmd/emu/build.sh            # go test does NOT build the emulator

**SCOPE WITH `-run` WHILE ITERATING — the full suite is a PHASE GATE, not a
per-edit check** (user directive 2026-08-16, "run them quickly as per our defined
method"). Measured here, wall clock:

| invocation | wall |
| --- | --- |
| `go test ./gui/ -run 'Name1\|Name2' -count=1` | **6 s** |
| `go test ./gui/ -count=1` | **282 s** |
| `go test ./... -count=1` | **249 s** (warm cache) |

**Narrowing `./...` to `./gui/` buys NOTHING** — `gui` *is* essentially the whole
suite cost (281 s of its 282 s is test time, not build). **The `-run` filter is
the entire lever, worth ~47x.** About 6 s of any scoped run is `nix develop`
startup, so batch alternations into one invocation rather than running them
singly. Run the full suite once per phase gate, with **stdout and stderr in
separate files**.

Bare `go` is not on PATH — "command not found" proves nothing. `go vet` needs a
**COLD** `GOCACHE`; **exit 1 with 40 test-only findings IS the clean baseline**.
Separate stderr from stdout: nix prints `Git tree is dirty` on stderr and a
`2>&1` capture has corrupted counts twice.

**Shell state does not persist between tool calls** — a `$VAR` set in one call is
empty in the next, and `grep -n "x" $EMPTY` hangs reading stdin.

## PUSHING

**FREEZE `master` WHILE A STAGING PUSH IS IN FLIGHT. THE RITUAL ASSUMES THE TIP
DOES NOT MOVE.** Learned the hard way 2026-08-16: a push agent staged a SHA and
waited for CI; the controller committed twice during the CI-watch window; the
final `git push origin master` therefore pushed a tip **two commits past** the
gated SHA. GitHub's `strict:false` accepted it against the older gated ancestor
and printed

    remote: Bypassed rule violations for refs/heads/master:
    remote: - Required status check "test (rust + go)" is expected.

Nothing was lost — but two commits reached `origin/master` with **zero CI
signal**, which is the exact failure the ritual exists to prevent. The push agent
did everything right and reported the bypass rather than claiming success; the
defect was the controller committing underneath it.

**So: no commits to `master` between `git push origin master:refs/heads/ci/staging`
and the final `git push origin master`.** If work arrives during that window,
hold it — or re-stage the new tip afterwards and verify it, which is the
remediation used here.



    git push origin master:refs/heads/ci/staging
    gh run watch <id> --repo bg002h/mnemonic-engrave
    git push origin master          # no bypass message = SATISFIED
    git push origin --delete ci/staging

`gh` needs `--repo`; full 40-char SHAs; judge **per-job** conclusions. Verify the
staging deletion with a positive control. The fork's `main` is unprotected.
**`enforce_admins: false` is the operator's deliberate hatch — never propose
flipping it.**
