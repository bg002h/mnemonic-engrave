# H3 `composer-spec` — independent verification (sonnet, read-only)

**Verifying:** engrave branch `h3-composer-spec`, tip `2627c4b465b78c663acb993611d2c6c08ae16a9e`
(one commit, `git diff --stat` from base: `2 files changed, 66 insertions(+), 10 deletions(-)`,
`design/SPEC_hashlock_H2_device.md` 56+/8-, `design/SPEC_wallet_policy_composer.md` 10+/2-).
**Drafter's report:** `design/agent-reports/h3-composer-spec-draft.md`.
**Method:** `git diff <base>..2627c4b` read in full; every quoted device string and line
number re-grepped against the read-only fork worktree
`/scratch/code/shibboleth/.tmp/seedhammer-hashlock-h2` at both `17b3979` (`git show`) and its
actual HEAD `a1fd139`; the plan's `## R0 round 0 folded here` (commit `f60c2df`) read and
diffed word-for-word against the folded text; the gate re-run myself against the tip's file
content, not copied from the report. No branch created, nothing committed, nothing run in
the fork worktree beyond read-only `git show`/`grep`/`sed`/`awk`.

**Base note:** the drafter's own report already caught that engrave `master` (68aae89) and
the fork tip (a1fd139) had moved past the brief's cited revisions (d81714e, 17b3979); I
independently re-confirmed both drifts (`git merge-base --is-ancestor d81714e 68aae89` → 0;
fork worktree `git log --oneline` → HEAD `a1fd139`) rather than taking them on trust.

## Claims table

| # | Claim | True/False | Evidence |
| --- | --- | --- | --- |
| 1 | Branch has exactly one commit `2627c4b`, from `master` at `68aae89`, touching only the two named spec files | TRUE | `git log --oneline 68aae89..2627c4b` → 1 commit; `git diff --stat` → 2 files, both named |
| 2 | `SPEC_wallet_policy_composer.md:386` replacement lands on the same line 386 | TRUE | `grep -n "From H2 the composer derives" /tmp/verify_composer_spec.md` (extracted from the commit tree) → `386` |
| 3 | Cross-references `SPEC_hashlock_H2_device.md:35` and `:506` still resolve correctly, untouched | TRUE | Both read at the tip: `:35` cites `SPEC_wallet_policy_composer.md:386`; `:506` still reads "the composer spec sentence H3 folds: `SPEC_wallet_policy_composer.md:386`" — line number correct, tense stale (report's own residue item 2, not hidden) |
| 4 | §14 row rewritten: storage/display/engraving stay out of scope, derivation does not | TRUE | tip `design/SPEC_wallet_policy_composer.md:1078`: `\| on-device preimage storage, display or engraving \| C25; §6c. Derivation is no longer out of scope: ... \|` |
| 5 | Provenance paragraph after §6c cites fork `hashlock-h2 a1fd139`, leg reviewed at `17b3979` | TRUE | present verbatim in the diff, immediately after line 386's paragraph |
| 6 | §4.5 drop-order departure applied is the plan's exact replacement sentence (verbatim) | TRUE | plan `f60c2df`, "H3 record item, first", replacement sentence compared word-for-word against the tip's §4.5 last clause — identical |
| 7 | §4.5 gains the other-path line, code-block form and bullet form, both the plan's wording verbatim except one substitution | TRUE | plan's "H3 record item, second" compared word-for-word; bullet is byte-identical; code-block line differs only in the quoted copy string (see #8) |
| 8 | The ONE deliberate deviation: plan quoted `"...two phrases to back up"` (the string at `17b3979`), branch now quotes the live `"...back up every phrase"` (from `a1fd139`) | TRUE | `git show 17b3979:gui/composer_copy.go` lines 451-453 → `"...two phrases to back up"`; fork HEAD `gui/composer_copy.go:454-456` → `"...back up every phrase"`. Both line ranges exact |
| 9 | New `## H3 fold` paragraph appended, citing `f60c2df`, naming both departures and the substitution | TRUE | present at end of tip file, matches diff shown above |
| 10 | Also-folded: "stay as they are until then" / present-tense "say" corrected to past tense, quotation kept | TRUE | diff line 33-41 region: now reads "SAID ... all three are now rewritten" and "H3 has now folded both (`## H3 fold`)" |
| 11 | `gui/composer_hash.go:27-28` — rewritten fork record, identical at `17b3979` and HEAD | TRUE | both revisions, lines 27-28: `THE COMPOSER DERIVES A PREIMAGE IN RAM FOR ONE SCREEN (H2) AND NEVER STORES, / SHOWS OR ENGRAVES IT. It puts a digest in a script.` |
| 12 | `gui/composer_hash.go:139,165,167,169` — phrase row constant + appended before "Type 64 hex"/"No hash lock" | TRUE | `sed -n '139p;165p;167p;169p'` → `const composerHashRowPhrase = "Type a hashlock phrase"` / `labels = append(labels, composerHashRowPhrase)` / `"Type 64 hex"` / `"No hash lock"`, exact line numbers |
| 13 | `gui/composer_hashlock.go:19-20` — preimage on the stack, dropped on return | TRUE | lines 19-20 verbatim: "phrase screen returns to `Which hash?` (§4.6). The preimage lives on the stack" / "here and is dropped when this function returns (L7, L15)." |
| 14 | `gui/composer_hashlock.go:64-69` — only `hashlock.Digest(&x)` reaches `Paths[idx].Hash` | TRUE | line 64: `h := hashlock.Digest(&x)`; line 69: `st.list.Paths[idx].Hash = &d` |
| 15 | `gui/composer_copy.go:409-417` — relation then otherPath, in that order, identical at both revisions | TRUE | `func composerCopyHashlockConfirm` at 409, `if relation != ""` at 412, `if otherPath != ""` at 415 — same line numbers at `17b3979` and HEAD |
| 16 | `gui/composer_state.go:244` (HEAD) / `:239` (fork baseline `c4a64fc`) — `composerEveryPathHashed` | TRUE | `grep -n func composerEveryPathHashed` → `:244` at HEAD and at `17b3979`; `:239` at `c4a64fc` |
| 17 | `md/compose.go:315` — `ValidatePathList` touches `p.Hash` exactly once, a nil check, never comparing two paths | TRUE | `func ValidatePathList` at line 299; `awk` over its body finds exactly one `Hash` reference, `p.Hash == nil`, at absolute line 315 — same at `17b3979` and HEAD |
| 18 | Gate: 5 patterns 0 hits, `'never derives'` exactly 1 hit at `SPEC_hashlock_H2_device.md:36` (the deliberate past-tense quotation) | TRUE | re-ran the same 6-pattern grep myself against the tip's extracted files — identical result, same line 36 |
| 19 | No Rust/Go gate needed — commit adds no executable content | TRUE | `git diff` contains zero added ``` fences; both files are prose plus one inline code block that was already present pre-fold (the §4.5 line-order block, itself not compiled) |
| 20 | Plan and `design/FOLLOWUPS.md` untouched by this commit | TRUE | `git diff 68aae89..2627c4b -- design/IMPLEMENTATION_PLAN_hashlock_H2_device.md design/FOLLOWUPS.md` → empty |
| 21 | Nothing pushed | TRUE | `git ls-remote origin` finds `master` (control) but no trace of `h3-composer-spec` or `2627c4b` |
| 22 | Residue #1 (§6c still describes two entry routes) is real, not fixed by this commit | TRUE | tip §6c still reads "Primary: pick from the payload's `hash:` records ... Fallback: type 64 hex", no phrase route named |
| 23 | Residue #3: F-481 is not in `design/FOLLOWUPS.md` at `68aae89` | TRUE | `grep -c F-481 design/FOLLOWUPS.md` → 0 |
| 24 | Residue #4: shipped confirm body (`gui/composer_copy.go:418-422`) is the shortened reuse block with no reconciliation line, matching what §4.5's code block still shows as the pre-drop original | TRUE | lines 418-422 are exactly the shortened two-sentence reuse block; no reconciliation-line text in that return |
| 25 | (Report accuracy, not branch content) diff --stat table: composer file listed as `7 ++--` | **FALSE** | actual `git diff --numstat` for that file is `10 8` → `2` deletions `10` insertions = 12 changed lines, not 7. The report's own aggregate totals (66 insertions / 10 deletions) are correct and match mine; only the per-file breakdown line is wrong — a copy/transcription slip in the report's gate section, not a defect in the spec content itself |
| 26 | (Report accuracy) §0 claim "`grep -rn 'F-481' design/` finds it only in `design/agent-reports/hashlock-H2-post-impl-*.md`" | **FALSE, imprecise** | F-481 also appears in `design/CONTINUITY_composer_2026-09-01.md` and `design/agent-briefs/hashlock-H2-post-impl*.md`. The substantive point — F-481 is absent from `FOLLOWUPS.md` — is still correct and independently confirmed (#23); only the "only in" scoping is overstated |

## Assessment

Every sentence the drafter added to either spec file is true of the fork branch at both
`17b3979` (the reviewed leg named in the brief) and the branch's actual HEAD `a1fd139` —
all eight device citations, both plan-departure replacements (word-for-word against
`f60c2df`), and the one deliberate string substitution, which is exactly what it claims to
be (the plan's string is the `17b3979` string; the branch quotes the `a1fd139` string, and
both are cited at the correct line numbers). The self-run gate reproduces identically. The
commit touches only the two named files, carries no code, and is not pushed; the plan and
FOLLOWUPS.md are untouched. The four residue items the drafter named as out-of-scope are
each independently confirmed still present and correctly characterized as not-fixed-here.

Two findings are **about the report, not the branch**: the diff --stat table under-reports
the composer file's line count (7 vs. actual 12, rows #25) and the F-481 location claim in
§0 is narrower than reality (rows #26). Both are Minor — neither changes what the commit
did, what the specs now say, or whether either is faithful to the device. Nothing rises to
Critical or Important; nothing the report claims about the spec content or the device
disagrees with the source.

## Verdict

**GREEN.** All 24 substantive claims about the branch, its diff, its citations, and its
gate hold true under independent re-derivation. Two Minor report-accuracy slips noted
(rows #25, #26) — record, do not block.
