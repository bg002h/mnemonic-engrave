architect cut-release-fold-r1-stop: 2 answered; 0 prior rulings changed

# Architect: cut-release spec fold r1 STOP (2026-11-01)

Inputs: worktree `cut-release-spec` at 151c951b (spec 9de7d34a), review r1,
the fold STOP, and the draft `/scratch/cache/cut-release/fold-r1/SPEC_cut_release.draft.md`
(line numbers below are the draft's). Checked myself: `git push` lines are
4 committed / 9 draft (320, 327, 344, 472, 473, 542, 684, 760, 773); the
three new texts hold no `"`, `$(` or backtick; `bin/mech_checks.py`: PASS;
every anchor below occurs once; a mock `cut-release.sh` (the three texts as
here-document lines plus the three code pushes) was run through both filters.

## Q1. Row 49: option A2, an anchored A. Not A as written, not B.

**Answer.** Keep the static grep and split its output: L1 = lines whose
source text starts in column 1 with `cut-release: `, L2 = the rest. L2 must
hold exactly the three allowed push forms, as today. Each L1 line must start
with one of the three known prefixes and hold no `"`, `$(` or backtick. A §2
code-shape sentence makes the three texts here-document lines.

**Does it keep the guarantee? Yes.** It covers every code path, at the same
strength as the committed row.
- **A as written has a hole.** Its filter is unanchored
  (`grep -v 'cut-release: '`), so it drops any line that *contains* the
  prefix. On the mock, A's count stays 3 with either of these added:
  `git push --force origin HEAD:master || die "cut-release: push failed"`
  (the common `|| die` idiom), or `printf "%s\n" "cut-release: x" >&2; git
  push --tags origin` (the "printf whose next statement pushes" case).
- **A2 catches both** (L2 = 4, forbidden flag present). It also catches
  `$(git push --force origin master)` spliced into the pins-PR line, through
  L1's character rule; A misses that too.
- **An L1 line is data.** A column-1 `cut-release: ` line is a here-document
  body line; only `$(` or a backtick runs a command there, and L1 refuses
  both. Refusing `"` stops a double-quoted string closing on that line.
  Measured: the three here-document functions under a logging `git` shim
  made 0 git calls.

**Residual limits.** A line inside a single-quoted multi-line string (the §2
sentence forbids it; the whole-diff review checks). `git -C <dir> push`, a
push via a variable or `eval`, and refs in a variable (N4) were limits of the
committed row already. No new FOLLOWUPS line: these are N4's class.

**Why not B (a `git` shim).** It sees only paths that rows run (no row
reaches §6.3 step 5's exit 9 or §7's delete-failure exit 9). It would also
log the fixtures' own pushes, each an exemption the spec must decide:
fake-pvs's `git push origin HEAD:refs/heads/$1` (line 684, a forbidden
`HEAD:` form), fake gh `pr merge`, `FAKE_PR_BRANCH_MOVES` and `fx_commit`.
Its one gain over A2 is N4, a Nit.

C is not needed, because A2 has no C-class defect.

**Changes** (`design/SPEC_cut_release.md`, through the fold's draft). Fold
needed: yes, in the one fold commit.

E1, §2 Code shape (line 102). Anchor:
```
source it. It uses here-strings, never `echo | grep -q` (F-695). These
```
Replacement:
```
source it. It uses here-strings, never `echo | grep -q` (F-695). The three
operator texts that contain the words `git push` (§5.3 step 3's second
toolkit line, the §5.3 step 4a pins-PR line and §6.3 step 4's exit-9 text)
are each written as one line of a here-document that starts in column 1
with `cut-release: ` and contains no `"`, `$(` or backtick. Every other
`git push` in the script is a command, and no comment names it (row 49).
These
```
E2, row 49 (line 766). Replace the whole line with:
```
| 49 | cr_static_push | `grep -nE '\bgit push\b' scripts/cut-release.sh`; L1 = its output lines that match `^[0-9]+:cut-release: `, L2 = the other lines | L2: exactly 3 lines. One contains `--atomic origin` (its refs are `refs/tags/<tag>`), one contains `refs/heads/release-pins-`, one contains `origin --delete`. None contains `--tags`, `--force`, ` -f `, `HEAD:` or `--mirror` (default-branch pushes happen only inside push-via-staging). L1: after its `<n>:`, every line starts with `cut-release: otherwise finish by hand: `, `cut-release: pins PR: ` or `cut-release: scripts/install-assets.test.sh failed on (c) `, and no line contains `"`, `$(` or a backtick (§2 code shape) |
```
The parenthesis "(its refs are `refs/tags/<tag>`)" stays verbatim. N4 is
deferred and not fixed here.

## Q2. The STOP's 8 mechanical resolutions

1. **Row 43: CONFIRM (ruling a).** It quotes §6.3 step 4's text verbatim
   (472 -> 760); the old text would fail on a conforming script. Updating a
   row that quotes changed text is mechanical. Add **E6** for its placeholders.
2. **Row 25 union and the After clauses of rows 26, 40, 48: CONFIRM.** Add
   **E7**. Fake gh logs to one `$T/gh.log` per harness run (lines 691, 638),
   so "absent or empty" in rows 25/26/40/48 cannot hold after rows 33-39. The
   same goes for row 66's "no `pr merge`" after row 46.
3. **I-1 paragraph position and the dropped "(I-3)": CONFIRM.** Rows keep
   their "(I-x)" tags (outside code spans; a tie, review text kept). Add
   **E3**: after I-2's split, the toolkit exit-5 re-run is refused by
   P12a, not P12: HEAD is (c), one commit ahead of origin (b), and §6.3 step 1
   returns to master on every path (P12 at line 171 now requires "not an
   ancestor", P12a at line 172). This is prose, not an operator text.
4. **I-2 order: AMEND with E5.** Nothing runs after `exit 0`. Doing the reset
   just before `== DONE` means a failing reset ends the run (`set -e`) with no
   success line, so there is no false PASS. Row 42's end state is the same.
5. **I-3 item 3 (the period becomes `; they end ...`): CONFIRM.**
6. **§6.3 step 5 `--body-file <f>`: OVERTURN with E4.** Step 4a writes "the
   §6.3 step 5 PR body" to `<git-dir>/cut-release/pins-pr-body.md` (line 339).
   Every printed by-hand command names that file (lines 344, 472, 478), and
   `<f>` is defined nowhere. Leaving it would be a builder choice.
7. **Fake gh `isCrossRepository` (ruling b): ADD E8.** Without it, `pr view`
   returns no such field, and step 6's "only if `isCrossRepository` is false"
   cannot pass. Row 46 would then leave the branch instead of "deleted", and
   row 67 would print the wrong reason. No row needs `true` (I-5's row is
   67 only), so there is no env knob.
8. **Row 46's argv-shape sentence stays in row 46: CONFIRM.** It restates the
   general rule for `pr merge` (lines 691-693: fake gh answers exactly the §7
   calls, anything else exits 99). Where it sits changes no behaviour, so the
   review's placement is kept.

**Exact texts** (each anchor occurs once in the draft):

E3 (line 318): `because a re-run is refused (P12, then P19):` ->
`because a re-run is refused (P12a, then P19):`

E4 (line 474): ``--body-file <f>`.`` ->
``--body-file <git-dir>/cut-release/pins-pr-body.md`.``

E5 (line 483). Anchor:
```
then `== DONE: <T> published and verified`, exit 0. Then `git checkout --quiet master && git reset --quiet --keep <b>`: local master
```
Replacement:
```
then `== DONE: <T> published and verified`, exit 0. Just before the `== DONE` line it runs `git checkout --quiet master && git reset --quiet --keep <b>`: local master
```
E6 (line 636). Anchor:
```
`<12>` and `<sha>` in expected text are computed by the harness.
```
Replacement:
```
`<12>`, `<sha>`, `<c sha>` (commit (c)'s full sha) and `<git-dir>`
(`git rev-parse --absolute-git-dir` in the fixture checkout) in expected
text are computed by the harness.
```
E7 (line 691). Anchor:
```
- **fake gh** logs every call to `$T/gh.log` and answers exactly the calls
```
Replacement:
```
- **fake gh** logs every call to `$T/gh.log`, which the harness empties
  before each row, and answers exactly the calls
```
E8 (line 702). Anchor:
```
  `pr checks` answer from `FAKE_PR_*` env. `pr merge` does
```
Replacement:
```
  `pr checks` answer from `FAKE_PR_*` env; `pr view` always answers
  `"isCrossRepository": false`. `pr merge` does
```
None of E1-E8 changes a row count or an operator text the review fixed, so
the counts stay 76/68. Fold needed: yes. Add E1-E8 to `bin/apply_fold.py`,
rebuild, run `bin/mech_checks.py`, then make the one fold commit as the STOP
report describes.

## Noticed, not ruled (route it; not one of these questions)

§6.4's `land it: cut-release.sh --land-pr <number>` goes into the PR body,
which step 4a writes before §6.3 step 1, so the number cannot exist yet (true
of the committed step 5 too). What the body prints there is undecided, and a
builder will STOP on it. Suggest the r2 brief or the next architect question.

## E9 (follow-up): the PR number in the pins-PR texts

**Ruling.** Change the `<number>` placeholders as follows:
- **Body, follow-on.txt, dry run and every §8 copy:** keep them number-free.
  The §6.4 line and §8 (b)'s line print the literal `<PR number>`.
- **§6.3 step 6's stdout line:** keep `<number>`, now defined as the digits at
  the end of the URL that `gh pr create` prints. That line is the one place
  where the number exists.
- **No `gh pr edit`.**

This applies to the body and to §8's copies alike (§8 (c) is the §6.4 block,
and §8 (b) has the same undefined `<number>`). Counts stay at 76/68.

**Why.** The body file is the same for every create:
- 4a writes it before §6.3 step 1 (draft line 339);
- the by-hand creates use it: 4a's pins-PR line, §6.3 step 4's exit 9 and
  step 5's exit 9 (lines 344, 472, 478).

So the body must stand without the number on every path. A `gh pr edit` would
fix only the script's own path. It would also add a gh call, a fake-gh
answer, and a new exit after the PR already exists, all for a number the PR
page already shows.

The same holds for follow-on.txt and the dry run, which are written before
any PR exists, and for §8 (b), whose PR someone opens later.

On `--push`, step 6's line follows §8 and carries the real number. Row 42
already expects `--land-pr 77`, taken from the fake's
`https://github.com/bg002h/mnemonic-toolkit/pull/77`. `gh pr create --help`
(gh 2.102.0) says: "Upon success, the URL of the created pull request will be
printed."

Typing the placeholder fails closed: the shell refuses the `<PR` redirect,
and a quoted copy hits U6 (exit 2).

The URL-parse refusal below gets no row. It fails closed (exit 9 names the
problem and a read-only `gh pr list --head`, checked against
`gh pr list --help`). It is reachable only if gh changes its success output,
the same class as M20.

**Exact texts** (anchors are in the draft; E1-E8 do not touch them).

E9a, §6.3 step 5 (line 478). Anchor:
```
   `cut-release: could not open the pins PR (above); (c) is at refs/heads/release-pins-<N>; open it by hand: gh pr create --repo bg002h/mnemonic-toolkit --base master --head release-pins-<N> --draft --title 'pins: installer -> mnemonic-toolkit-v<N>' --body-file <git-dir>/cut-release/pins-pr-body.md`.
```
Replacement: the anchor line unchanged, followed by:
```
   On success, `<url>` is the last line `gh pr create` printed on stdout and
   `<number>` is the digits after its final `/pull/`. If that line does not
   match `^https://github\.com/bg002h/mnemonic-toolkit/pull/[0-9]+$`, exit 9:
   `cut-release: gh pr create exited 0 but printed no pull request URL; find the PR: gh pr list --repo bg002h/mnemonic-toolkit --head release-pins-<N>`.
```
E9b, §6.4 (line 507). Anchor:
```
     land it: cut-release.sh --land-pr <number>   (from a mnemonic-toolkit checkout)
```
Replacement:
```
     land it: cut-release.sh --land-pr <PR number>   (from a mnemonic-toolkit checkout)
```
E9c, after §6.4's block. Anchor:
```
## 7. `--land-pr <n>` (merge-readiness check; any table repo)
```
Replacement:
```
`<PR number>`, here and in §8 (b), is printed as it stands: the PR it names
does not exist yet when the text is written (the PR body, `follow-on.txt`,
a dry run). On `--push`, the §6.3 step 6 line that follows §8 names the
pins PR's number.

## 7. `--land-pr <n>` (merge-readiness check; any table repo)
```
E9d, §8 (b) (line 604). Anchor:
```
     gates: sh scripts/install-assets.test.sh; then land it: cut-release.sh --land-pr <number>
```
Replacement:
```
     gates: sh scripts/install-assets.test.sh; then land it: cut-release.sh --land-pr <PR number>
```
E9e, row 42 (line 759). Anchor:
```
gh.log has `pr create` with `--draft`;
```
Replacement:
```
gh.log has `pr create` with `--draft` and `--body-file <git-dir>/cut-release/pins-pr-body.md`, and that file contains `land it: cut-release.sh --land-pr <PR number>`;
```
Row 42 amended: it now also tests E4 and E9b. No row is added, so the counts
stay 76/68. Fold needed: yes, together with E1-E8 in the one fold commit;
run `bin/mech_checks.py` on the result.
