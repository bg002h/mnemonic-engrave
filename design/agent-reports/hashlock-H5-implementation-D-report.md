# hashlock H5 implementation report — implementer D (Task 6, records only)

Brief: `design/agent-briefs/hashlock-H5-implementer-D-brief.md`. Plan:
`design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md` (STATUS R0 GREEN, engrave
`5b77367`), executing **Task 6: Records** (spec §1.4, §2.5, §6, §7) only — no fork code.
Task 6 has no `file=`/`mode=` blocks and no RED/GREEN/MUTATION cycle: every block in it is
prose destined for a file outside the fork tree, and the plan says so in its own text
("none of it carries a `file=` header and `scripts/h5-plan-blocks-vs-tree.sh` checks none of
it"). This report is therefore a diff-and-verification account, not a test-count account.

## Worktrees

- Engrave: `git -C /scratch/code/shibboleth/mnemonic-engrave worktree add -b h5-records
  /scratch/code/shibboleth/me-worktrees/h5-records master`. Branched at `583c59f` (engrave
  master had advanced past the brief's `58261e6` by two continuity/brief-only commits
  between reading the brief and creating the worktree; `5b77367` is an ancestor of both —
  confirmed with `git merge-base --is-ancestor`).
- Toolkit: `mkdir -p /scratch/code/shibboleth/tk-worktrees && git -C
  /scratch/code/shibboleth/mnemonic-toolkit worktree add -b h5-manual
  /scratch/code/shibboleth/tk-worktrees/h5-manual master`. Branched at `46b40bbb`, matching
  the plan's own citation SHA for the manual exactly.

## Engrave commit: `4e2cf01f3e5275b1692d7b3ce9a6c4f7da864f33` on `h5-records`

`git diff --stat $(git merge-base master HEAD) HEAD`:
```
design/FOLLOWUPS.md               | 18 +++++++++++++-----
design/SPEC_hashlock_H2_device.md | 27 +++++++++++++++++++--------
2 files changed, 32 insertions(+), 13 deletions(-)
```
(merge-base = `583c59f`, the branch point; engrave `master` has since moved further as other
implementers land, which is expected and does not affect this diff.)

**Step 1 (H2 spec §4.5).** Replaced the confirm modal's write-down sentence
(`:264-266` at `e03d8e7`) with the plan's exact text, and split the fenced block: the
reconciliation lines (`:272-274`) were removed from the confirm-modal block and reissued as a
new post-HOLD block directly beneath it, with one linking sentence naming the destination
already documented at `:302-304`'s drop-order bullet. The reuse block (`:267-271`,
"One phrase per policy. Spending any path...") was left untouched, per the plan's explicit
instruction.

**Step 2 (H2 spec §4.7).** The phrase-route blockquote's last sentence: "Back up the phrase
and its method, or the / preimage plate, separately." → "Back up every phrase and its method,
and every / preimage plate, separately." The parenthetical immediately after it, which
previously only named the plain form's noun, now also quotes its corrected ending:
*"Back up every preimage separately."*

**Step 3 (FOLLOWUPS.md).** F-480, F-484, F-485, F-487, F-488 each got a closure header in
the file's own convention (as F-475/F-486 do): `~~slug~~` **CLOSED 2026-09-05 by fork
`<FORK_MERGE_SHA>`** with a gate citation and an explicit "hashlock H5 -- the next device code
cycle these were owned to, not overdue" clause (the plan requires this be grep-checkable), all
followed by the original body verbatim including each entry's original owning-phase
parenthetical and tags. `<FORK_MERGE_SHA>` is a literal placeholder per the brief; only the
controller can fill it, at the actual merge to fork `main`. Two new entries filed at the end
of the file:
- **F-491** (owning phase: H2 spec hygiene) — the §4.5 reuse-block drift Step 1 declined to
  fix. Quoted text was verified live against fork `main` (`b9a9a30`,
  `git show main:gui/composer_copy.go`): the shipped `composerCopyHashlockConfirm`
  (`:421-422`) reads "One phrase per policy. Never use this phrase as a passphrase or a
  password anywhere else." — confirming the spec's fenced block (the four-sentence
  pre-drop-order form) is stale, exactly as the plan describes.
- **F-492** (owning phase: the `me`/sysw manual chapter) — spec §5's journey M-5 note has no
  home. Verified live: `docs/manual/src/40-cli-reference/` holds exactly `41-mnemonic.md`,
  `42-md.md`, `43-ms.md`, `44-mk-cli.md`, and `grep -rn "Nothing was opened\|cannot be unlocked
  here\|not a seed" docs/manual/src/` at toolkit `46b40bbb` returns nothing.

**Step 5 commit**, message copied byte-exact from the plan's `git commit -m` block, plus the
required trailers:
```
records: H5 folds H2 §4.5's write-down line and post-HOLD reconcile body and §4.7's phrase form; F-480/F-484/F-485/F-487/F-488 CLOSED at their owning phase; §4.5 reuse-block drift filed (hashlock H5)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
```

## Toolkit commit: `b48af1c1189d2764d81bad102d993a98afd0bb60` on `h5-manual`

`git diff --stat $(git merge-base master HEAD) HEAD` (merge-base = `46b40bbb` = `master`,
unmoved):
```
docs/manual/src/40-cli-reference/43-ms.md | 23 ++++++++++++++---------
1 file changed, 14 insertions(+), 9 deletions(-)
```

Implements plan Task 6 Step 4 in full (the brief's own file list names this file with a
two-item summary — "re-quote the reconcile screen and the confirm modal's write-down line" —
but the plan's Step 4 specifies three edits to it; all three are in scope of Task 6 and none
touches another implementer's files, so all three are made):

1. `:482-483` confirm-modal write-down line re-quoted, byte-exact with Step 1's device text.
2. `:501-502` reconcile blockquote replaced with the plan's five-line block (now carrying
   `hash`/`method:` lines plus the "Before you cut plates... build it again." sentence),
   byte-exact.
3. One clause added to the sentence beneath it, naming the host counterpart of `chars: <n>`.
   Verified live in `mnemonic-secret` (repo HEAD = `504ff46`, matching the plan's citation
   exactly): `crates/ms-cli/src/cmd/hashlock.rs:329-330` writes `phrase_chars` into the JSON
   stdout record; `:351-352` prints an engraving-card stderr line. Ran
   `printf 'correct horse battery staple' | ./target/debug/ms hashlock --hashlock-phrase-stdin
   --method hardened` (repo's own built binary) and captured stdout/stderr separately: stdout
   is `hash:3cf5d421...b70a4c12` (matches the manual's existing worked example unchanged, a
   free correctness check on text this task doesn't touch); stderr's line is
   `phrase:          28 characters -- write the method line next to your phrase; ...` —
   confirming both the field name and the "28" figure the new clause cites.
4. `#### What Back does`: lead sentence qualified to "Every Back inside the route, before the
   hold, moves one step back within it and keeps the phrase." and one row added:
   `| the reconcile screen | the spend-path list | dropped; the hash is already assigned |`.

`make lint` tail, from `/scratch/code/shibboleth/tk-worktrees/h5-manual/docs/manual`
(full output captured at `/scratch/code/shibboleth/.tmp/h5-manual-lint.txt`):
```
[lint] === 5/6 glossary-coverage ===

[lint] === 6/6 index bidirectional ===

[lint] OK
```
Checks 1/6 (markdownlint), 2/6 (cspell) and 3/6 (lychee) all report 0 errors. Check 4/6
(flag-coverage) emits ~30 `WARN: no flags parsed ... skipping` lines — these are a worktree
path artifact (the flag-coverage check shells out to sibling repos at paths relative to the
toolkit checkout, e.g. `.../tk-worktrees/descriptor-mnemonic/Cargo.toml`, which doesn't exist;
the real sibling repos live at `/scratch/code/shibboleth/descriptor-mnemonic` etc., outside
`tk-worktrees/`), not a defect introduced by this change — WARN, not FAIL, and unrelated to
any file this task touches. The gate's own final line is `[lint] OK`.

## Deviations from the brief/plan

1. **Toolkit manual scope.** The brief's own summary names two of Task 6 Step 4's three
   edits to `43-ms.md`; the plan's Step 4 (the authoritative text for "Task 6," which the
   brief says to follow "exactly") specifies a third: qualifying the `#### What Back does`
   lead sentence and adding a table row. Implemented all three, in the same file and the same
   commit, since the brief also names this exact file as in-scope and nothing here touches
   another implementer's files.
2. **H2 spec post-HOLD paragraph wording.** The plan gives the two fenced blocks for §4.5
   Step 1 verbatim but not the one connective sentence between them (it describes the
   sentence's *purpose* — "moves out of the confirm modal's line list and into the post-HOLD
   paragraph beneath it" — rather than prescribing its exact wording). Wrote one minimal
   sentence reusing phrases already established at `:302-304` ("its own dismissible screen
   shown immediately after HOLD," "reachable for every policy that has a phrase-set hash") so
   nothing new is asserted beyond what the spec already states elsewhere.
3. **F-49x closure-header wording.** The plan requires each closure to name "the commit that
   closed it and the gate that proves it" and to state plainly that H5 is the owning phase (so
   the burndown grep sees it as not overdue); the exact sentence form is mine, modeled on the
   file's own `F-475`/`F-486` conventions since the plan doesn't dictate closure-header prose
   verbatim the way it does for the spec sentences.

No file outside the brief's list was touched in either repo. Nothing pushed. No sub-agents
used. No `.jsonl` files read. No phrase or preimage bytes appear in this report or in any
command output captured to disk.

## Branch tips

- engrave `h5-records`: `4e2cf01f3e5275b1692d7b3ce9a6c4f7da864f33`
- toolkit `h5-manual`: `b48af1c1189d2764d81bad102d993a98afd0bb60`
