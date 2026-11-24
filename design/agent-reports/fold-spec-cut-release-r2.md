# Fold report: cut-release spec, fold r2 (Sonnet xhigh)

Fold commit: `313c637a` on `spec/cut-release` (local only, never pushed), on top of `22a9b1d4`. Two files: `design/SPEC_cut_release.md`, `design/FOLLOWUPS.md`. Input: `review-spec-cut-release-r2.md` (0C/3I/15M/4N). Gate: **PASS**. No STOP. Every anchor matched exactly the expected number of times; no deviation from any fix text.

## Per Important (anchor -> applied -> deviation)

| I | Anchor in the committed spec | Applied | Deviation |
|---|---|---|---|
| I-1 | §4 after the `\| P21 \|` line; §2 code-shape sentence; row 49 L1 clause; after row 68 | New P23 row (exit 3); "The four operator texts ... and P23"; row 49 prefix list gains `cut-release: origin has ci/staging at `; new offline row 69 `cr_stale_staging` | none |
| I-2 | §5.3 step 4a first and last bullets; §6.3 step 6 "(7, 8, 9, or a trapped signal)"; fake-gh bullet; after row 69 | EXIT trap text (acts only on a `set -e` abort; its own line, finish lines, exit 7); "The traps key on variables"; "(7, 8, 9, a trapped signal, or the EXIT trap)"; `FAKE_GH_FAIL=1`; new offline row 70 `cr_push_gh_fails_after_tag` | none |
| I-3 | §9 fake-pvs fixture bullet; the `$T/gh.log absent or empty;` After clause in rows 25, 26, 40, 48 (exactly 4 occurrences) | fake-pvs appends `$* <sha of HEAD>` to `$T/pvs.log` (emptied before each row); the four After clauses read `$T/gh.log and $T/pvs.log absent or empty;` | none |
| counts | §9 count sentence; §10 step 5; network rows | 78 by default, 70 with `--offline` (rows 1-70); `78 passed`; network rows 69-76 -> 71-78, names unchanged | none |

Other references to renumbered rows: only "as 74" in the two real-repo network rows (now rows 77 and 78) -> "as 76". Every other `row N` reference was resolved by the mechanical check (rows 21, 28-32, 42, 44-47, 49, 50 unchanged).

## Mechanical consequences beyond the fix texts (no choice involved)

- `design/FOLLOWUPS.md` F-709's intro sentence says where its network rows "are now 69-73, 71, 74-76 and 76". Those move +2 again, so it now reads "were ... after fold r1 and are now 71-75, 73, 76-78 and 78". Nothing else in F-709 touched.
- The fake-gh bullet line break after the new `FAKE_GH_FAIL=1` sentence is wrapped by hand; text unchanged.
- No Minor or Nit is fixed. The F-709 re-grade table is untouched (all stay Minor/Nit).

## FOLLOWUPS

Entry **F-710** (`design/FOLLOWUPS.md`, last entry): 15 Minors and 4 Nits, then a "Design concerns (ungraded)" sub-heading with the 2 bullets. All 21 lines checked byte-identical and in order against the review. `**Status:** OPEN — owning phase: cut-release build`, "deferred-minor (RULES.md)", cites `review-spec-cut-release-r2.md`. The intro says the row numbers in the lines are the review's: the network rows they call 71 (first design concern), 75 (M14) and 76 (M11, M14) are now 73, 77 and 78. The lines themselves are untouched. `scripts/followups-status.sh`: OK (`out/gate-followups-status.txt`).

## Gate results (outputs in `/scratch/cache/cut-release/fold-r2/out/`; scripts in `.../fold-r2/bin/`)

- `apply.txt` (`bin/apply_fold.py`): 15 edits, 13 matched once, the network renumber 8, the After clause 4.
- `mech-final.txt` (`bin/mech_checks.py`, the r1 script with: new-row range 51-70, "N both times" accepted as an exit-code lead, and rows 51-70 excluded from the template pool so a row cannot match itself): PASS. Rows 1-78 unique and contiguous (70 offline, 8 network); count sentence 78/70/70 = tables; §10 step 5 = 78; every `row N` reference resolves to the row meant; exit codes of rows 51-70 are all in §2's table (row 69: 3, row 70: 7); every P/C/V id cited by rows 51-70 is defined (P23 included). One quoted text without a template (row 59's `== FOLLOW-ON: ...` header), which sits in §8's code fence at line 580 and is unchanged. Negative controls: altering row 69's or row 70's operator text makes the script report it as unmatched.
- `gate-row49-final.txt` (`bin/row49_check.py`, four operator texts: P23, §5.3 step 3's toolkit line, the 4a pins-PR line, §6.3 step 4's exit-9 text): PASS. `bash -n` rc 0 on the mock; L1 = 4, L2 = 3; the 5 bypass controls each fail row 49; the pre-fold 3-prefix rule rejects the P23 text, so the row 49 edit is needed.
- `gate-readonly.txt` (`bin/gate_readonly.sh`): 12 PASS, 0 FAIL. `git ls-remote origin refs/heads/ci/staging` on the 7 real repos: rc 0, empty. `push-via-staging.sh` line 107 pushes `HEAD:refs/heads/ci/staging` as the review says. In a sandbox with a local bare remote only: after a fetch (P10 brings `ci/staging` along), a new release commit staged over a stale `ci/staging` is rejected (rc 1, non-fast-forward); the P23 rule prints the spec's text byte for byte with `<12>` = the unrelated commit, rc 3; the printed `git push origin --delete ci/staging` runs as printed and clears it; S = HEAD (resume) is allowed; after the delete the staging push succeeds; a dry run that calls fake-pvs leaves `ls-remote` unchanged (so rows 25/26/40/48 passed before) but leaves `pvs.log` non-empty (so the new clause fails the row). A trap sketch written from the 4a text: `set -e` abort -> rc 7, one "stopped" line, one finish line; normal DONE -> rc 0, no trap output; exit 8 -> rc 8, no "stopped" line; SIGTERM -> rc 7, only the "interrupted" line.
- `bash -n` rc 0 on: the printed delete command, fake-pvs, the trap sketch, the row 49 mocks.
- Not run: every writing command against a real remote; the harness and `cut-release.sh` do not exist yet.

## Notes for the next review (not folded, not STOPs; the fix texts decide the printed text)

- §5.3 step 4a bullet 2 still starts "The trap, and every exit 7, 8 or 9, ends with:". With two traps, "the trap" now means both the signal trap and the EXIT trap; the EXIT-trap text already says it prints the finishing lines, so the meaning holds.
- The EXIT-trap text does not say whether it also prints step 6's `== follow-on pins are NOT listed: the release is not verified` line. Row 70 checks only that its own line and the finish line appear (membership), so either builder reading passes.
- The r1 `mech_checks.py` template check let a row match itself (its pool included the rows it checks); the r2 copy fixes that. The r1 fold's own rows 51-68 now pass that stricter check except row 59's header noted above.
