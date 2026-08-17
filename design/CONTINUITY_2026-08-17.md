# Continuity — 2026-08-17: **S6a IS SHIPPED.** Next is S6b, then the hardware flash.

Supersedes `CONTINUITY_2026-08-16c.md`. Read this one.

---

## ▶ START HERE

**S6a is merged and pushed. There is nothing open on it.** Do not re-review it,
do not re-run its gates, do not re-litigate its decisions.

| repo | branch | head | state |
| --- | --- | --- | --- |
| fork `bg002h/seedhammer` | `main` | `b1479a1b38f6b045d27443764c858906e4e6e122` | pushed, direct (unprotected) |
| `bg002h/mnemonic-engrave` | `master` | see `git log -1` | pushed via `ci/staging`, check **SATISFIED** |
| worktree `wt-s6a` | `s6a-singlesig-truth` | merged | **safe to remove** |

**Next cycle: S6b** — the single compressed pre-flash cycle (operator directive
"compress", 3 cycles → 2): **F-199**, **F-204**, and the passphrase plate. See
`REQUIREMENTS_s6b_pre_flash_cycle.md`. Title decided: **`PASSWORD REQUIRED`**
(17 chars; `MaxTitleLen` is 18 and truncation is **SILENT** — "PASSPHRASE
REQUIRED" is 19 and would engrave as `PASSPHRASE REQUIRE`).

Then the hardware flash.

---

## WHAT S6a SHIPPED

The single-sig engrave flow took a BIP-39 passphrase, derived from it, engraved
**nothing about it**, labelled the result **"Full (seed + keys)"**, and printed a
restore document that never mentioned a passphrase. The words alone restore a
*different* wallet, with no error. **Permanently unspendable, and the paperwork
vouched for it.**

Now: the label says `Full (seed + keys, NOT passphrase)`; the restore document
always renders and carries **exactly one** status line from a 2×2 of two
*recorded* booleans (`fullPassRecorded` + sticky `adverseRecorded`) whose
`default:` arm is the zero cell, so **monotonicity is structural**; and the page
carries a plate inventory, a seed statement and a passphrase statement.

Closed: **F-198**, **C-1**, F-195, F-197, F-202, plus both multisig paths.
Filed, not gating: **F-206**, **F-207**.

## THE NUMBERS

- **R0 loop closed GREEN at round 17.** Step-1 gate GREEN. Whole-diff adversarial
  review RED 0C/1I → fixed → closing re-review **GREEN 0C/0I**.
- 21 files, **+3209 / −73**. `go test ./... -count=1` on the merge commit:
  **51 ok / 0 FAIL**, stderr empty, `gofmt` clean.

## THE TEST SUITE IS NEAR A HARD CEILING — this binds S6b

**`gui` runs 429–507 s against Go's default 600 s per-package timeout** (~71–85%).
Step 7's first draft blew straight through it: the package died at 600 s
mid-engrave **with every assertion passing**. A timeout is **not** a test failure,
and the fix is never to delete assertions or raise the limit reflexively.

**Scope with `-run` while iterating — it is the whole lever:**

| invocation | wall |
| --- | --- |
| `go test ./gui/ -run 'A\|B' -count=1` | **~6-7 s** |
| `go test ./gui/ -count=1` | ~430-450 s |
| `go test ./... -count=1` | ~436 s |

**Narrowing `./...` to `./gui/` buys NOTHING** — `gui` is essentially the entire
suite. Full suite once per phase gate, **stdout and stderr in separate files**.

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"

Bare `go` is not on PATH; "command not found" proves nothing. Two **pre-existing**
`go vet` go1.26 `t.ArtifactDir()` failures — `gui/freetext_sizeproof_golden_test.go:111`
and `gui/op/draw_test.go:176` — are **not yours** (it is two files, not one).

## REVIEWER TIERING (user directive, 2026-08-16)

**sonnet** — mechanical/fold verification. **opus** — design-level adversarial,
**and the final pre-irreversible review, including the hardware flash.**
**`fable` is not a reviewer tier at any stage**; the old carve-out is closed.

## THE LESSON S6a PAID FOR — read this before planning S6b

**No round found a DESIGN defect after the four-state rewrite.** Every later
defect was in the **records**, and the same class recurred five ways:

| form | instance |
| --- | --- |
| a stale **count** | "six call sites" vs §4.3's measured eight; "three" vs four |
| a stale **schedule** | one edit scheduled to three different steps; it could not compile at the earliest |
| a blind **instrument** | `restoreDocFlow(` cannot match `multisigRestoreDocFlow(` — found 1 of 4 sites, in the plan *and* in a dispatch brief |
| a stale **enumeration** | §4.8b, written expressly to end this class, decayed under the steps it was scheduled after |
| a **relocated constant** | a sentence stayed fixed while the set of pages it renders on grew underneath it |

**Two were introduced by folds fixing other defects.** A fold is authorship and
re-earns the gate.

**What actually caught them was EXECUTION, not review.** §4.7c declared itself
sole authority for the printed text and **did not contain half of it**;
seventeen rounds missed that, and one attempt to implement found it immediately.
§4.7f specifies a line and **owns no mechanism** for it. The relocated-constant
defect was invisible to every test, because each assertion checked *the line*
while the defect lived in the **relationship** between the line and its page.

### So, for S6b — the standing recommendation

1. **Front-load a throwaway executable spike** against the spec *before* it
   closes. The plan's own rule — *a plan may not close while any of its own gates
   has never been run* — should be widened to **"has every specified output ever
   been produced?"**
2. **Machine-check the counts and the quoted commands.** Every count defect here
   was a script away; so was the case-blind grep.
3. **Cut the review loop at lens-closure on design.** Rounds 12–17 found only
   transcription, which execution finds better and cheaper.
4. Keep **independence** and the **mutation checks** — three false PASSes this
   cycle were found by mutation and by nothing else, including one in a test
   written to catch that exact defect.

## GATES AND TOOLING

    ./scripts/verify-returnsite-sweep.sh <plan>   # MULTISIG ONLY -- PERMANENTLY
    ./scripts/plan-cite-check.sh        <plan>    # every path:line resolves
    ./scripts/plan-glyph-check.sh       <plan>    # strings the display font can draw
    ./scripts/plan-table-check.sh       <plan>    # no table row lost a cell
    ./scripts/plan-wiring-check.sh      <plan>    # nothing declared and left unwired
    ./scripts/plan-fold-sweep.sh <plan> --terms '<superseded phrase>' ...

**Every one prints what it does NOT cover, and two have DEMONSTRATED blind
spots.** `verify-returnsite-sweep` covers multisig **permanently** — single-sig
takes an out-parameter and returns no verdict, so a return-site sweep cannot see
it *by design*, and its count will never rise. `plan-wiring-check` still passes a
name mentioned in two sections that never reached the section the document calls
its authority. **Read those lines; a green gate is not a proof.** Never cache
their numbers in prose — that rotted twice; the commit message carries each
fold's measured output.

**The glyph gate's known gap:** its heuristics only reach blockquotes and
backtick spans ≥40 chars. On `REQUIREMENTS_s6b_*` it scanned **zero** strings and
still exited 0 — **a 0-scanned pass is not a clean pass.** Check such docs
directly.

## PUSHING — the two repos have DIFFERENT rules

**Fork `bg002h/seedhammer`: `main` is UNPROTECTED.** Direct `git push origin main`.
Never push to `upstream`.

**`bg002h/mnemonic-engrave`: `master` REQUIRES the staging ritual**, because a
status check binds to a **SHA**, not a branch — a commit pushed straight to
`master` has no check when the rule is evaluated and is **bypassed**.

    git push origin master:refs/heads/ci/staging
    gh run watch <id> --repo bg002h/mnemonic-engrave
    git push origin master          # no bypass message = SATISFIED
    git push origin --delete ci/staging

**FREEZE `master` FOR THE WHOLE WINDOW** — no commits between the staging push
and the final push; the ritual assumes the tip does not move. `gh` needs
`--repo`; use full 40-char SHAs; judge **per-job** conclusions. Verify the
staging deletion with a **positive control** (`ci/staging` absent *while* master
is present). **`enforce_admins: false` is the operator's deliberate hatch —
never propose flipping it.**
