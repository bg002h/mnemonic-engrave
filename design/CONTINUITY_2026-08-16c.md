# Continuity — 2026-08-16c: S6a is at its R0 gate. NO CODE EXISTS YET, and that is correct.

Supersedes `CONTINUITY_2026-08-16b.md`. Read this one.

---

## ▶ START HERE — the first thing to do in a fresh session

**Step 1. Read the R4 adversarial verdict**, which decides everything else:

    head -20 design/agent-reports/s6a-r4-adversarial.md

That file is the last review of the S6a plan. It exists because every dispatched
agent persists its own report — the controller is never the only copy.

**Step 2, branch on its verdict:**

- **GREEN (0 Critical / 0 Important) → the R0 loop is CLOSED. Start implementing.**
  Do **not** run another review round for reassurance; a clean re-review closes
  the loop. Go to §4.8's nine-step build order in
  `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`.
  **Step 1 of that order is a GATE, not a task:** produce the single-sig
  11-exit → `verifyStatus` mapping and have it reviewed *before* any other code.
  Work in `/scratch/code/shibboleth/wt-s6a` (branch `s6a-singlesig-truth`, empty,
  already created). One implementer, TDD.

- **RED → fold it, and fold it the way that has been working:**
  1. Commit the report verbatim in its **own** commit first.
  2. **Fold from the file, never from memory** — re-read/re-run every mechanism
     claim. This is where five of six folds went wrong.
  3. **Grep the superseded phrasing** afterwards; reading the diff cannot find
     what was left behind. Check headings, not just bodies.
  4. Run `./scripts/plan-cite-check.sh` and `./scripts/plan-glyph-check.sh`.
  5. Dispatch the **cheap sonnet claim-verification pass** before any expensive
     adversarial round. It has paid for itself every time.

**Step 3. Do not re-litigate** the decisions in `design/agent-reports/`:
C-1's remedy, ONE PIECE (F-198 is not separable), and the cycle scope.

**Step 4. THE ESCALATION RULE** (agreed with the operator 2026-08-16, after five
consecutive RED rounds): **if a round returns a Critical, dispatch a FABLE
BLIND-SPOT PASS before folding it.** One question only —

> *Here is the design and the properties it claims. What failure mode do they
> collectively fail to constrain?*

Then **fold its answer yourself**; do not have fable author the fold.

**Why this shape, and not "fable writes the fold".** The operator proposed the
latter; this is the agreed refinement. A fold is mostly mechanical — apply
decisions to markdown, propagate, gate — and paying fable rates to edit prose is
the same category error this repo already names, *a reviewer being paid
design-review rates to act as a compiler*.

**What actually failed was judgement, once, and it is nameable.** Through R3 every
fold defect was a *verification* failure (a loop condition never traced, a slice
composition never read), which is why an earlier proposal to delegate the fold
was declined — a fresh author inherits that failure mode unchanged. **R4's C-1
was different.** The plan asserted two properties, P1 and P2, and both constrained
*under*-warning; nothing constrained *over*-condemning, so the design would have
stamped "Do NOT rely on this backup" on perfectly good steel. No amount of
checking finds that. Someone has to ask what direction the properties are blind
to — and a reviewer asked it, not the author. P3 exists only because of that.

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

## WHERE IT STANDS — seven lenses, every one found something

| round | lens | result |
| --- | --- | --- |
| R0-A / R0-B | adversarial funds / executability | RED — 1C 4I · 0C 1I |
| R1-A / R1-B | fold-vs-findings / adversarial on fold | RED — 0C 1I · **2C** 3I |
| R2-A / R2-B | adversarial on fold / spec coverage | RED — **2C** 5I · 0C 1I |
| R3-pre | cheap claim verification | DIRTY — 1 false claim |
| R3-A / R3-B | adversarial on fold / comprehension | RED — **2C** 5I · 0C 8I |
| R4-pre | cheap claim verification | DIRTY — 0 false, 4 stale |
| R4 | adversarial on the simplified design | **IN FLIGHT** |

All R0–R4-pre findings are folded. Reports are in `design/agent-reports/s6a-*`
(13 files), each persisted **verbatim in its own commit before** the fold
responding to it.

## THE FIVE THINGS THIS CYCLE PAID FOR

1. **The folds were the weak artifact, not the plan.** Six folds carried defects;
   the last two rounds each found a Critical *inside the algorithm written to fix
   the previous Critical*. Every one was a failure to CHECK something checkable —
   never a failure of judgement.
2. **So the fix was HOW, not WHO.** The operator asked whether an agent should
   author the fold. No: fold from the file never from memory; enumerate rather
   than argue; and add a **cheap sonnet claim-verification pass before the
   expensive adversarial round**. That pass caught a Critical-class defect on its
   first run for a fraction of an opus round, and 4 more on its second.
3. **Delete, don't patch, when a structure keeps generating defects.** §4.7's
   severity lattice had an unfixable collision — the zero value had to be *safe*
   while the accumulator seed had to be the *minimum*, and one variable cannot be
   both. Replaced by two sticky facts and a switch. Both Criticals became
   structurally impossible rather than fixed.
4. **Closure is LENS-closure.** Three of the seven lenses were first-time
   questions (spec coverage, comprehension, claim verification) and each found
   what re-running the others could not. Stop when out of QUESTIONS.
5. **A blind-spot section that overstates its coverage is worse than silence.**
   §8.4 claimed a test covered the capacity wiring; it covered the text only.
   That is precisely what a reviewer budgets away from.

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
