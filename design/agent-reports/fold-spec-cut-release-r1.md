# Fold report: cut-release spec, fold r1 (Sonnet xhigh)

Fold commit: `6eb4a483` on `spec/cut-release` (local only), on top of `a8d2b683`. Two files: `design/SPEC_cut_release.md`, `design/FOLLOWUPS.md`. Inputs: `review-spec-cut-release-r1.md` (0C/7I/20M/5N) and the architect's `architect-cut-release-fold-r1-stop.md` (E1-E9e). Gate: **PASS**. No STOP. Every anchor (review and architect) matched exactly once; row counts stay 76 / 68.

## Per Important (anchor -> applied -> deviation)

| I | Anchor in the committed spec | Applied | Deviation |
|---|---|---|---|
| I-1 | Section 5.3 step 3, after the "If not: ... `git reset --hard origin/<default>`." sentence | The toolkit's two replacement lines, then the pointer to the step 4a pins-PR line, before "On success: `== FREEZE LIFTED`" | The review's "(I-3)" in the last sentence dropped; E3 changed "(P12, then P19)" to "(P12a, then P19)" |
| I-2 | Section 6.3 step 6 "Local master stays at (c)..." sentence; the P12 row | P12 narrowed, new P12a row, reset text (`git checkout --quiet master && git reset --quiet --keep <b>`) | E5: reset runs "Just before the `== DONE` line", not after exit 0 |
| I-3 | Section 5.3 step 4 (tag push failure), step 6 exit-7/8 sentence, Section 6.3 step 4 exit-9 text, P15 | Three-outcome tag-push failure; new step 4a (follow-on.txt, pins-pr-body.md, INT/TERM/HUP trap, finishing lines); exit-9 fix-forward text; `--body-file` in step 5's exit-9 text; P15 hint line | none |
| I-4 | after P18 | new P22 (skip refused) | none |
| I-5 | Section 7 step 1 and step 6 | `isCrossRepository` in the `pr view` fields; delete only if not cross-repo and origin's branch is still the merged head, else "left in place (<reason>)"; delete failure exit 9 | none |
| I-6 | Section 8 (c)/engrave demo block | `git commit -m '...' -- demo/sh2/src/index.html demo/sh2/CLI_SPINE.md` | none |
| I-7 | Section 9 rows 25/26/40/48, 42, 43-ish, 46, 36, 49-adjacent; fixtures; counts | After clauses (row 25 merged as a union with its existing After; rows 26, 40, 48 added); row 42/46/36/45 amendments as given; fixtures `silent`/`landed-fail`, `FAKE_GH_KILL`, `FAKE_GH_DRAFT`, `FAKE_PR_MERGE_STATE`, `FAKE_PR_BRANCH_MOVES`, `MK_ASSETS_*`; offline rows 51-68 as given; network rows renumbered 51-58 -> 69-76 (`as 56` -> `as 74` in rows 75-76); counts 76 / 68 in Section 9 and Section 10 step 5 | the row 25 union and the row 40 "no bare `release-pins-*`" clause are the STOP's resolution 2, which the architect confirmed |

Also applied: `On --push only` before "commit (b) from the index" (Section 6.3), as the review's I-1 text requires. The "Decisions the author flagged" table: all six kept.

## Architect rulings applied

- E1 + E2: Section 2 code-shape sentence; row 49 rewritten (L1 = `^[0-9]+:cut-release: ` output lines, L2 = the rest; L2 exactly 3 allowed pushes, L1 prefixes and no `"`, `$(`, backtick).
- E3 (P12a in the I-1 paragraph), E4 (`--body-file <git-dir>/cut-release/pins-pr-body.md` in step 5), E5, E6 (`<c sha>`, `<git-dir>` defined), E7 (harness empties gh.log before each row), E8 (fake `pr view` answers `"isCrossRepository": false`).
- E9a-E9e: step 5 parses `<url>`/`<number>` from `gh pr create`'s output with a new exit-9 text; `<PR number>` literal in Section 6.4, Section 8 (b), the new explanatory paragraph before Section 7, and row 42 (which now also checks `--body-file` and the body's `land it: ...` line). Step 6's stdout line keeps `<number>`.
- My STOP's resolutions: row 43 update confirmed (uses the new step 4 text); resolutions 1-8 as the architect ruled.

## FOLLOWUPS

Entry **F-709** (`design/FOLLOWUPS.md`): 20 Minors + 5 Nits verbatim (checked byte-identical, in order, against the review), `**Status:** OPEN — owning phase: cut-release build`, "deferred-minor (RULES.md)", cites the review. One addition for you to confirm: the intro has a sentence saying M1, M2, M10 and M12 cite network rows by their r1 numbers (51-55, 53, 56-58, 58), which are now 69-73, 71, 74-76, 76. The M/N lines themselves are untouched.

## Gate results (outputs in `/scratch/cache/cut-release/fold-r1/out/`)

- `mech-final.txt` (run on the committed file): PASS. Rows 1-76 unique and contiguous; count sentence and Section 10 step 5 = tables; every "row N" reference resolves to the row meant; exit codes of rows 51-68 defined in Section 2; refusal ids cited exist; 0 unmatched quoted texts.
- Row 49 vs the draft (`gate-row49-final.txt`, `bin/row49_check.py`): classified every `git push` line in the spec (3 operator texts at 326, 350, 478; 3 code pushes; fixtures/rows). A mock `cut-release.sh` built from the spec's three operator texts as column-1 here-doc lines plus the three code pushes: `bash -n` OK, L1 = 3, L2 = 3, no error. Five bypass controls each fail row 49 (`|| die`, printf-then-`push --tags`, `$(git push --force ...)` spliced in a text, a 4th plain push, a comment naming it). Unanchored option A counts 3 on the first control (the hole E2 closes).
- `gate-arch-readonly.txt`, `fragments2.sh`: `bash -n` rc 0 on 12 fragments; E9a's `gh pr list --head` command runs against the real repo; the URL regex accepts `/pull/77` and `/pull/123`, rejects 3 negatives; `gh pr create --help` says the URL is printed; real `pr view` returns `isCrossRepository:false`.
- Earlier-round read-only checks (gh run list, git ls-remote, absolute-git-dir) and scratch-repo runs of I-6 and I-2 and the P22 cases still hold (`gate-gh-run-list.txt`, `gate-git-readonly.txt`, `gate-scratch-i6-i2.txt`, `gate-p22.txt`).
- `scripts/followups-status.sh`: OK.
- Not run: `sh scripts/install-assets.test.sh` (command unchanged) and every writing command.

## Notes for the next review

- Line 67 (`--help` usage `cut-release.sh --land-pr <number>`) and step 6's stdout line keep `<number>`, as E9 rules.
- Row 42 now depends on the Section 6.4 block containing `land it: cut-release.sh --land-pr <PR number>` as a substring of the body file.
