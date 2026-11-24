# STOP: cut-release spec fold r1 (Sonnet xhigh)

Nothing is committed. The worktree is at `0672f946`, clean. One undecided point blocks the fold.
Everything else is drafted and machine-checked in a scratch copy (paths at the end).

## The point: row 49 cannot pass once I-1 and I-3 are applied

Row 49 (`cr_static_push`, spec line 729 at `0672f946`):

> `grep -nE '\bgit push\b' scripts/cut-release.sh` -> exactly 3 lines. One contains `--atomic origin`, one contains `refs/heads/release-pins-`, one contains `origin --delete`. None contains `--tags`, `--force`, ` -f `, `HEAD:` or `--mirror`.

The review's exact texts add three operator texts that contain the words `git push`. The spec says every operator-visible text is fixed, so each is a literal string in the script, and the grep matches it:

1. I-1, second toolkit line (new paragraph in §5.3 step 3): `... <c sha> && git push origin refs/tags/mnemonic-toolkit-v<N>; ...`
2. I-3 item 2, the pins-PR line in step 4a: `... && git push origin <c sha>:refs/heads/release-pins-<N> && gh pr create ...`
3. I-3 item 4, §6.3 step 4's exit-9 text: `... Fix forward on top of (c): git push origin <c sha>:refs/heads/release-pins-<N>, commit the fix ...`

Measured: the committed spec has exactly 3 literal `git push` outside the fixtures (lines 320, 455, 520). The draft has 6 (lines 320, 327, 344, 472, 473, 542; `out/gate-row49.txt`). So the grep returns at least 6 lines on any script that follows the spec. "Exactly 3" fails, and "one contains `refs/heads/release-pins-`" becomes three lines. The review's I-7 does not amend row 49 (only its N4 nit, which is a different point). The brief makes a row's expected output an undecided point, so I stopped.

### Options (the review decides none)

- **A (recommended).** Keep the grep and exclude operator texts: command `grep -nE '\bgit push\b' scripts/cut-release.sh | grep -v 'cut-release: '`. Expect: exactly 3 lines remain, as now. Add to §2 "Code shape" (or to row 49): every operator text that names `git push` is written on one source line that contains its `cut-release: ` prefix. All three new texts start with `cut-release: `. Cost: one code-shape sentence.
- **B.** Replace the static grep with a dynamic check: put a `git` shim on PATH that logs every `push` argv, and after rows 33, 42, 46, 55-57 and 67 assert that the log holds only these shapes: `--atomic origin refs/tags/<t>...`, `origin <sha>:refs/heads/release-pins-<N>`, `origin --delete <branch>`. Stronger (it also settles N4), but it adds a fixture and rewrites the row. Needs an architect pass.
- **C.** Reword the three review texts so they avoid the words `git push`. Rejected: it changes review-fixed operator texts (a printed command the operator runs).

## Mechanical resolutions already in the draft (confirm or override; none blocks)

1. **Row 43** quotes §6.3 step 4's stderr text, which I-3 item 4 changes. The review's row list omits it. The draft updates row 43 to the new text with N = 0.105.2 (`<c sha>` and `<git-dir>` left as placeholders). Leaving it would make the spec contradict itself.
2. **Row 25's After clause.** It already lists status, HEAD, ls-remote and one tag. The draft merges the review's four items into it (union; "no local tag named in the command" now covers `ms-codec-v0.10.2` too). Rows 26, 40, 48 get a new "After:" clause. Row 40 also gets "no bare `release-pins-*`".
3. **I-1 paragraph position.** It sits right after the "If not:" reset line in §5.3 step 3, before "On success: `== FREEZE LIFTED`". "(I-3)" in its last sentence is dropped. The new rows keep the review's "(I-2)", "(I-4)", "(I-5)", "(I-6)" tags verbatim.
4. **I-2 text.** The replacement begins "Then `git checkout --quiet master && git reset --quiet --keep <b>`" and follows "exit 0.", as the review writes it. It reads as if the reset comes after exit 0. No row can observe the order against `== DONE`. Command and branch names got backticks.
5. **I-3 item 3.** The anchor ends in a period, so the appended `; they end with ...` replaces it: "...and skip §8; they end with the step 4a finishing lines."
6. **§6.3 step 5** still says `--body-file <f>`. The review gives no text for it, and 4a now defines the PR body file as `<git-dir>/cut-release/pins-pr-body.md`.
7. **Fixture gap, not filled.** fake gh `pr view` must answer `isCrossRepository` (false by default) for rows 46 and 67 once §7 step 1 asks for it. The review adds no fixture sentence, and I added none.
8. Row 46's "The fake `pr merge` answers only that argv shape ..." stays in row 46, where the review puts it, not in the fixtures bullet.

## What was run (outputs in `/scratch/cache/cut-release/fold-r1/out/`)

- Draft = the committed spec plus I-1..I-7: every anchor matched exactly once; 18 new rows 51-68; network rows renumbered 69-76 (`as 56` -> `as 74` in rows 75-76); counts 76 / 68 (§9 intro, §10 step 5).
- `mech-draft.txt`: PASS. §9 rows unique and contiguous 1..76; every "row N" reference resolves to the row the text means; every exit code the new rows expect is one §2 defines; the only refusal id they cite (P12a) exists; every quoted `cut-release:` / `FAIL` text in rows 51-68 matches a spec template.
- Read-only commands against the real repos, all as printed: `gh run list --repo bg002h/mnemonic-toolkit --branch mnemonic-toolkit-v0.105.1` shows release, man-pages, install-pin-check and changelog-check completed/success; `gh pr view 123 ... --json ...,isCrossRepository` returns `isCrossRepository:false`; `git ls-remote origin refs/heads/master` and `refs/tags/ms-cli-v0.20.2^{}` print what the spec says (an absent ref prints nothing, rc 0); `git rev-parse --absolute-git-dir` works (a linked worktree gives its per-worktree dir). No mismatch.
- Scratch repos (`gate-scratch-i6-i2.txt`): I-6's `git commit -m ... -- <two demo files>` commits exactly those two files, leaves `README.md` modified, and `git status --porcelain --untracked-files=no` is then non-empty (push-via-staging's "tracked files are modified" guard, line 81, would stop); the old `-am` committed `README.md`. I-2's `git checkout --quiet master && git reset --quiet --keep <b>` leaves master = origin/master = (b), the tag and `release-pins-N` still at (c), tree clean.
- `gate-p22.txt`: the successor rule gives `0.20.2 -> 0.20.4` refused with `one of 0.20.3 0.21.0 1.0.0` (row 65), accepts every N the harness uses, and treats `0.14.0-dev` as requiring `0.14.0`.
- `gate-bash-n.txt`: the 11 shell fragments the fold adds, extracted and placeholder-substituted: `bash -n` rc 0.
- `followups-entry.md` + `gate-followups-status.txt`: next free id is F-709 (file ends at F-708); entry has 20 M + 5 N lines verbatim; `scripts/followups-status.sh` on a copy: OK.
- Not run: `sh scripts/install-assets.test.sh` (command unchanged; only repeated in a printed line; the review ran it OK), and every command that writes (tag, push, `gh pr create`, `gh pr merge`).

## To resume once row 49 is ruled

`python3 -I /scratch/cache/cut-release/fold-r1/bin/apply_fold.py <spec> <review> <out>` rebuilds the draft from the committed spec (add the row-49 edit to it), then `bin/mech_checks.py <out>`. Copy the result over `design/SPEC_cut_release.md`, append `out/followups-entry.md` to `design/FOLLOWUPS.md`, run `scripts/followups-status.sh`, and commit as the one fold commit the brief names.
