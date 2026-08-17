# Continuity — 2026-08-16c: S6a is at its R0 gate. NO CODE EXISTS YET, and that is correct.

Supersedes `CONTINUITY_2026-08-16b.md`. Read this one.

---

## ▶ START HERE — the first thing to do in a fresh session

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
     JUDGEMENT → dispatch a FABLE BLIND-SPOT PASS before folding** (see Step 4).

**Step 4. THE ESCALATION RULE** (agreed 2026-08-16). On a Critical, or a
JUDGEMENT-labelled Important, ask fable ONE question before folding:

> *Here is the design and the properties it claims. What failure mode do they
> collectively fail to constrain?*

Then **fold its answer yourself** — fable's value is judgement, not
transcription. Reviewers now label each finding MECHANICAL or JUDGEMENT so the
trigger does not depend on the controller's own classification.

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

1. **R4 verdict.** Clean → the loop CLOSES (do not re-loop for reassurance).
2. **Implement**, in `wt-s6a`, one implementer, TDD, following §4.8's **nine-step
   build order**. Step 1 is a gate, not a task: the single-sig 11-exit → status
   mapping, reviewed before any other code.
3. Whole-diff adversarial review → merge → push.

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

Bare `go` is not on PATH — "command not found" proves nothing. `go vet` needs a
**COLD** `GOCACHE`; **exit 1 with 40 test-only findings IS the clean baseline**.
Separate stderr from stdout: nix prints `Git tree is dirty` on stderr and a
`2>&1` capture has corrupted counts twice.

**Shell state does not persist between tool calls** — a `$VAR` set in one call is
empty in the next, and `grep -n "x" $EMPTY` hangs reading stdin.

## PUSHING

    git push origin master:refs/heads/ci/staging
    gh run watch <id> --repo bg002h/mnemonic-engrave
    git push origin master          # no bypass message = SATISFIED
    git push origin --delete ci/staging

`gh` needs `--repo`; full 40-char SHAs; judge **per-job** conclusions. Verify the
staging deletion with a positive control. The fork's `main` is unprotected.
**`enforce_admins: false` is the operator's deliberate hatch — never propose
flipping it.**
