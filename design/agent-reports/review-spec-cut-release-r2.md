r2: 0C/3I/15M/4N

# Review r2: `design/SPEC_cut_release.md` (whole artifact), one Sonnet xhigh slice

- Artifact: `design/SPEC_cut_release.md` at head `f54464bc` (fold commit `6eb4a483`), branch `spec/cut-release`, worktree `/scratch/code/shibboleth/me-worktrees/cut-release-spec`.
- Brief: `coordinator/briefs/review-spec-cut-release-r2.md`. Grades follow RULES.md and its likelihood test. Nothing was pushed, tagged, merged or created on any remote. All probes ran in scratch dirs under `/scratch/cache/review-cut-release-r2/` (sandboxes `sandbox-a/b/c`, clones, `out/`, scripts in `bin/`).
- Outcome: **no Critical. Three Important, all fail closed and all with exact fix texts below.** The fold itself is faithful: all 20 `cut-release: ` operator texts of the r1 report occur verbatim in the spec, and the fold's `mech_checks.py` (rows 1-76 contiguous, 76/68) and `row49_check.py` (3 L1 + 3 L2 pass, 5 bypass controls caught) re-ran PASS on the head. Both fold-sourced Importants sit in the reviewer's own fix texts, not in anything the Sonnet author chose.

## Answers to the seven questions

**Q1. Wrong tag, wrong commit, wrong landing, bypass, false success, crash/Ctrl-C and re-run.** Nothing in the spec can push a wrong tag name (tags are regex-validated and pushed by full ref), tag a commit other than the release commit (toolkit: (c); resume path: `HEAD` = origin tip with the manifest at N), land a default branch except through `push-via-staging.sh` (PVS) or `gh pr merge --merge --match-head-commit` after CLEAN, or report DONE without V1-V6. The failure points are walked below; three of them break (I-1, I-2; I-3 is a test gap).

| failure point | state left | re-run / advice | verdict |
|---|---|---|---|
| Ctrl-C or crash in the local phase (`--push`) | tracked files modified | P13 lists them | ok (F-709 M17) |
| after the commit(s), before PVS | local `release:` commit(s) | P12a: drop it, run again | ok |
| during PVS: Ctrl-C, 30-minute timeout, "no run appeared", "tip moved", or the final master push rejected | local commit; origin master unchanged; **`ci/staging` left on origin at the old sha**; toolkit: detached at (b) | P12a / exit-5 text says reset and run again; the new commit has a new sha; PVS's `git push origin HEAD:refs/heads/ci/staging` is refused non-fast-forward; exit 5 again, same advice | **I-1** |
| PVS red CI or missing contexts | PVS deletes `ci/staging` | reset, re-run | ok |
| PVS landed the commit, then exits non-zero (non-toolkit) | commit on master | "re-run the same command" finds it (row 55) | ok |
| same, toolkit | (b) on origin, (c) local-only | two by-hand lines, executed below | ok (M2) |
| PVS reports a bypass | commit on master, `ci/staging` kept | STOP, no tag | ok (F-709 M3) |
| atomic tag push fails | none on origin, local tags deleted | re-run resumes (row 57) | ok |
| INT, TERM or HUP after the tag push | trap prints interrupted + finish lines | row 59 | ok |
| **any other abort after the tag push** (a failing `gh`, `git ls-remote`, `jq`, download under `set -e`) | tags pushed; no text; exit 1; toolkit master left at (c) | non-toolkit: P15 hint on re-run; toolkit: P12a says "finish as cut-release.sh printed", nothing was printed | **I-2** |
| run fails / never appears / 90 min / verify FAIL / install-assets fail / PR create fails | exit 7/8/9 with texts | rows 36, 58, 37-39, 61-64, 43 | ok (M4, M5) |
| `--land-pr` | draft, base, checks, CLEAN, `--match-head-commit`, ancestry | rows 44-47, 66, 67 | ok (M1) |

**Q2. Does `--dry-run` make zero remote writes in every path?** As specified, yes. Its remote I/O is `git fetch`, `git ls-remote`, the sibling `git fetch`, engrave's curl GETs, and no `gh` call. **But no row can detect a dry run that calls `push-via-staging.sh`** (I-3), and that call is the one real remote write (a `ci/staging` push plus CI) a dry run could make.

**Q3. Rows that cannot fail; rules without rows.** I-3 (dry-run rows vs a PVS call). Weaker cases are M6, M7, M9, M10. Rules without a row, beyond F-709 M20: the tag-push outcomes "origin has every tag" and "mixed", the peel-mismatch exit 6 (M7), P12a with k >= 2, the reset and finish lines on exits 8, 9 and the signal trap (M6), the toolkit exit-5 pins-PR line (M6), `--trailer` on commit (c) (M8). Checked by execution: row 32's repad line equals what the rule gives on the real `Examples.md`; row 40's 5/4/4 counts equal the forms on today's toolkit files; rows 69-72 (verify rows) equal the V1-V6 results below; row numbering, references and exit codes of all 76 rows are the fold's `mech_checks.py` PASS, which I re-ran. For every other row I read the setup and expected text against the rule, looking for a way it passes with the rule broken; the cases found are I-3, M6, M7, M9 and M10.

**Q4. Run what you print.** Table at the end. Every read-only printed command was run on the real repos and latest releases; the writing ones (tag push, `pr create`, `pr merge`, branch delete) were run only in local sandboxes with a stub `gh`.

**Q5. Advice lens on I-1..I-7 and E1-E9 texts.** P12a, the two I-1 toolkit lines, the 4a pins-PR line, the three exit-9 texts, P15's hint, E9's `gh pr list` and the §8(a) `git commit -m ... -- paths` line were followed literally. They run as printed and cannot tag or land the wrong thing, except the states in I-1 and I-2, and M2-M5, M12, M13.

**Q6. Fold accounting for `6eb4a483` (Sonnet xhigh author, Opus xhigh architect texts).** Of my 0C/3I/15M/4N: **2I, 6M and 1N come from the fold** (I-2, I-3; M2, M3, M5, M6, M7, M12; N4). Pre-existing: 1I (I-1), 9M, 3N. The fold made no choice beyond the given texts (20/20 operator texts verbatim); every fold-sourced defect traces to the r1 reviewer's or the architect's prescribed text.

**Q7. Deferred minors.** None of F-709's open items becomes C or I; see the last section.

## Important

### I-1 [pre-existing] A stale `ci/staging` makes the printed retry loop forever (§5.3 step 3, P12a; PVS lines `git push origin "HEAD:refs/heads/ci/staging"` and `OUT="$(git push origin "HEAD:$BRANCH" 2>&1)"`)

**Scenario (measured).**
- PVS deletes `ci/staging` only on red CI and on missing contexts. It leaves it on origin on: "no workflow run appeared" (5 min), "no check runs visible", "check runs still incomplete after 30 minutes", "the tip moved", a rejected final master push (the `OUT=$(...)` line aborts under `set -e` before the delete, and before `echo "$OUT"`, measured), a bypass (left on purpose), and any Ctrl-C or crash after the staging push. A Ctrl-C during the CI wait (up to 30 minutes) is the likeliest interruption in the whole release.
- cut-release then exits 5, or the operator re-runs and meets P12a. Both say: drop the local commit (`git reset --hard origin/<default>`) and run again.
- The new release commit has a new timestamp, so a new sha, whose parent is the (possibly moved) tip. It does not descend from the old staging commit. Sandbox: `! [rejected] HEAD -> ci/staging (non-fast-forward)`, rc 1 (output in `out/`, sandbox `sandbox-a`). PVS exits 1 at its first push, so cut-release exits 5 with the same advice. The loop has no exit.
- Toolkit: same, and the checkout is first left detached at (b).
- This is the "crash or Ctrl-C midway and the re-run after it" the brief asks about. It fails closed (nothing wrong is tagged or landed), but the printed recovery cannot succeed and nothing names the cause. Needs one unusual condition, not two, so the likelihood test does not lower it.

**Fix.** A read-only preflight that names the cause before anything is changed.

1. Anchor (§4, directly after the line starting `| P21 |`): insert

   ```
   | P23 | `--dry-run`/`--push`: `git ls-remote origin refs/heads/ci/staging` prints a sha S, and S is not the sha of `HEAD` | `cut-release: origin has ci/staging at <12>; push-via-staging.sh cannot push a new release commit over it (an interrupted or failed staging run left it). If no staging run is in progress, delete it, then run again: git push origin --delete ci/staging` | 3 |
   ```
   (`<12>` = the first 12 characters of S. S = `HEAD` is the resume path and is allowed.)
2. Anchor (§2 code shape), old:
   ```
   source it. It uses here-strings, never `echo | grep -q` (F-695). The three
   operator texts that contain the words `git push` (§5.3 step 3's second
   toolkit line, the §5.3 step 4a pins-PR line and §6.3 step 4's exit-9 text)
   are each written
   ```
   new:
   ```
   source it. It uses here-strings, never `echo | grep -q` (F-695). The four
   operator texts that contain the words `git push` (§5.3 step 3's second
   toolkit line, the §5.3 step 4a pins-PR line, §6.3 step 4's exit-9 text and
   P23) are each written
   ```
3. Anchor (row 49, L1 clause), old: ``every line starts with `cut-release: otherwise finish by hand: `, `cut-release: pins PR: ` or `cut-release: scripts/install-assets.test.sh failed on (c) `, and no line contains`` new: ``every line starts with `cut-release: otherwise finish by hand: `, `cut-release: pins PR: `, `cut-release: scripts/install-assets.test.sh failed on (c) ` or `cut-release: origin has ci/staging at `, and no line contains``.
4. New offline row 69, after row 68:
   ```
   | 69 | cr_stale_staging | a second clone pushes an unrelated commit to `refs/heads/ci/staging` of the bare remote; `--push ms-cli-v0.20.3`, then `--dry-run ms-cli-v0.20.3` | 3 both times; `cut-release: origin has ci/staging at <12>; push-via-staging.sh cannot push a new release commit over it (an interrupted or failed staging run left it). If no staging run is in progress, delete it, then run again: git push origin --delete ci/staging` (`<12>` = the unrelated commit); no tag on the bare remote; bare master unchanged; HEAD unchanged; tree clean; `$T/pvs.log` absent or empty |
   ```
5. Counts, with rows 69 and 70 (I-2) both adopted: §9 ``76 by default, 68 with `--offline` (rows 1-68)`` -> ``78 by default, 70 with `--offline` (rows 1-70)``; §10 step 5 `cut-release.test: 76 passed, 0 failed` -> `78 passed`; the network rows 69-76 become 71-78 (names unchanged).

### I-2 [fold; text is the r1 reviewer's I-3 fix] Any failure after the tag push other than INT/TERM/HUP and the numbered exits 7/8/9 prints nothing and skips the toolkit reset (§5.3 step 4a, §6.3 step 6)

**Scenario (measured).**
- 4a installs a trap for INT, TERM and HUP only, and §6.3 step 6 lists "7, 8, 9, or a trapped signal". The script is `set -euo pipefail` (§2). The spec decides no behaviour for a failing `gh run list`, `gh run view`, `git ls-remote` (step 4's post-push check), `gh release download` or `jq` call during the up-to-90-minute wait. By default `set -e` kills the script.
- Demo (`bin/trap_demo.sh`): with the spec's trap design a failing call after `PUSHED=1` gives `exit status: 1` and no finishing lines; with an EXIT trap keyed on the variable the trap fires.
- Result: tags irreversibly pushed, exit 1 (not in the exit table), no "never delete or move them" warning, no finish line. Toolkit: local master stays at (c). The re-run meets P12a, whose text says "finish as cut-release.sh printed when it stopped", but nothing was printed, and the pins-PR line (the only route to open the PR) exists only in that unprinted text. Non-toolkit re-runs reach P15, whose hint is adequate.
- r1's I-3(c) title read "A signal or crash during the waits leaves no message"; the prescribed fix covered signals only. A transient `gh` HTTP error in a 90-minute poll is not a rare condition, and one condition suffices. Fails closed (no wrong release), so Important, not Critical.

**Fix.**
1. Anchor (§5.3 step 4a, first bullet), old: ``- From the tag push on, the script traps INT, TERM and HUP. The trap prints `cut-release: interrupted after the tag push; the tags are pushed and stay pushed: never delete or move them.` and exits 7.`` new:
   ```
   - From the tag push on, the script traps INT, TERM and HUP, and also EXIT. The signal trap prints `cut-release: interrupted after the tag push; the tags are pushed and stay pushed: never delete or move them.` and exits 7. The EXIT trap acts only when the variable set after the tag push is set, the variable set just before the `== DONE` line is unset, and no exit 7, 8 or 9 and no signal trap has begun (a third variable they set). That case is a `set -e` abort, for example a failing `gh`, `git ls-remote` or `jq` call. It prints `cut-release: stopped after the tag push by a failing command (above); the tags are pushed and stay pushed: never delete or move them.`, then the finishing lines below (for the toolkit after the §6.3 step 6 reset), and exits 7.
   ```
2. Anchor (same step, last bullet), old: ``- The trap keys on a variable set after the tag push, never on `$?`.`` new: ``- The traps key on variables set after the tag push, never on `$?`.``
3. Anchor (§6.3 step 6), old: ``(7, 8, 9, or a trapped\n   signal)`` (as the text now reads: `(7, 8, 9, or a trapped` / `signal)`) new: ``(7, 8, 9, a trapped\n   signal, or the EXIT trap)``.
4. Fixture, anchor (mid-paragraph in the fake-gh bullet, one line break) ``` `FAKE_GH_DRAFT=1`: `release view`
  answers `isDraft: true`.``` -> append after it ``` `FAKE_GH_FAIL=1`: on its first `run list` call fake gh prints `fake gh: simulated failure` to stderr and exits 1.```
5. New offline row 70, after row 69:
   ```
   | 70 | cr_push_gh_fails_after_tag | `FAKE_GH_FAIL=1`; `--push ms-cli-v0.20.3`; then F-tk with `--push mnemonic-toolkit-v0.105.2` | 7 both times; stderr shows `cut-release: stopped after the tag push by a failing command (above); the tags are pushed and stay pushed: never delete or move them.` and the finish line (the toolkit run also the pins-PR line with (c)'s sha); the tag is on the bare remote; `<git-dir>/cut-release/follow-on.txt` exists; toolkit: local master = bare master = (b) |
   ```
Counts as in I-1 step 5. (Optional, not needed to close: say that a failing `gh` call inside a poll counts as a poll with no result. The EXIT trap alone gives the guarantee.)

### I-3 [fold, After clauses from r1's I-7; the fixture is pre-existing] No row can fail if `--dry-run` calls `push-via-staging.sh` (§9 rows 25, 26, 40, 48; fake-pvs)

**Scenario (measured).**
- The headline guarantee is "`--dry-run` makes no remote write at all", and §1 itself says the first remote write is PVS. The rows' After clauses check: tree clean, `gh.log` empty, `ls-remote` equals the pre-run output, HEAD unchanged, no local tag.
- In a dry run nothing is committed, so HEAD = origin tip when a faulty dry run calls the fake PVS. The default fake runs `git push origin HEAD:refs/heads/$1`, which is `Everything up-to-date`, rc 0, `ls-remote` unchanged (sandbox output above). In the resume row 26 there is not even a commit to restore. So rows 25, 26, 40 and 48 all pass.
- With the real PVS the same call pushes `ci/staging` and starts CI. This is the single most dangerous dry-run regression and the only one the suite cannot see. One unusual condition (a builder misplaces one guard), so no likelihood downgrade.

**Fix.**
1. Anchor (§9 fixtures), old: ``- **fake-pvs**: `FAKE_PVS=ok` (default) runs `git push origin HEAD:refs/heads/$1``` new: ``- **fake-pvs** first appends `$* <sha of HEAD>` to `$T/pvs.log`, which the harness empties before each row. `FAKE_PVS=ok` (default) runs `git push origin HEAD:refs/heads/$1```.
2. In each of rows 25, 26, 40 and 48 (the string occurs exactly four times), old: ``` `$T/gh.log` absent or empty;``` new: ``` `$T/gh.log` and `$T/pvs.log` absent or empty;```.

## Minor (one line each)

- M1 [pre-existing] §7 step 2 says "`git fetch` the head": `git fetch origin <branch>` does not bring the tag (measured: tags `[]`; plain `git fetch origin` does; `git merge-base --is-ancestor "<tag>^{}" ...` then rc 128, not the exit-9 refusal). Rows 46/47 run in the checkout that created the tag, so they cannot see it. Fix: "`git fetch --quiet --tags origin`". Conditions: the builder fetches only the head branch, and `--land-pr` runs in a checkout without the tag; fails closed.
- M2 [fold] Toolkit exit 5 after (b) landed: `follow-on.txt` is written only before the tag push and the `finish:` line only follows exits 7/8/9, so the by-hand recovery never lists the engrave-demo and reference moves; add to the second I-1 line "..., then the pins-PR line below, then the moves in <git-dir>/cut-release/follow-on.txt" and write the file with `pins-pr-body.md` before §6.3 step 1.
- M3 [fold] For a lib-only run `<bin tags>` is empty, so the finish line prints `cut-release.sh --verify ; once it shows 0 failed ...`: bare `--verify` checks all seven CLIs, five of which fail today (V3/V4), so the condition cannot hold; omit the `--verify` clause when there is no bin tag. Conditions: lib-only run and an exit 7 (mk-codec CI) or a signal.
- M4 [pre-existing, E4/E9 touched] §6.3 step 5's exit-9 text says "(c) is at refs/heads/release-pins-<N>" even when the `git push` (not `gh`) failed; the complete pins-PR line is printed right after it and works, so only the claim is wrong.
- M5 [fold] §6.3 step 4's exit-9 text says "commit the fix to that branch, then gh pr create ...": `gh pr create --head` explicitly skips pushing (`gh pr create --help`), so the fix commit never reaches the PR; add "push it" after "commit the fix to that branch". Conditions: step-4 exit 9 and an operator who takes the text literally.
- M6 [fold] Rows pin printed recovery text but run only one command (row 56's tag line): the 4a pins-PR line, the two exit-9 texts and the `finish:` line are never executed by a row (I ran the pins-PR line in `sandbox-b`: it works from both the exit-5 and the exit-7 state), row 56 does not assert that the pins-PR line follows or that local master stays at (c), and rows 43, 59, 61-64 do not assert the reset or finish line; extend row 60 to run the printed pins-PR line and assert bare `release-pins-0.105.2` = (c) and the `pr create` argv.
- M7 [fold] No row for step 4's "origin has every tag" and "mixed" outcomes or the peel-mismatch exit 6; the mixed outcome is unreachable under `--atomic`, the other two fail closed.
- M8 [pre-existing] `--trailer` goes "to `git commit --trailer`", but the toolkit makes two commits; say whether (c) carries it (cosmetic, no row).
- M9 [pre-existing] Fake gh "answers exactly the calls of §5.3, §5.4, §6.3 and §7": P9's `gh auth status` is §4 and `gh release download`'s argv is nowhere fixed, so a literal fake answers 99 to every `--push`, `--verify` and `--land-pr` row; row 33's "gh.log has no `fake gh: unexpected`" is vacuous unless the fake also logs that line. Fails loudly (red harness).
- M10 [pre-existing] "Rows 44-47 run on the F-tk state that row 42 leaves" but row 43 (`as 42 with FAKE_INSTALL_ASSETS=fail`) needs a fresh F-tk (a re-run after row 42 meets P15) and sits between them; rows 66/67 also need "row 42's state" again. Say that each row builds its own fixture directory and that rows 44-47, 66 and 67 first re-run row 42's setup.
- M11 [pre-existing; deepens F-709 M12] §6.2's "insert right after the `[installer-ahead]` line": the real `pinned-upstream.toml` has two comment lines containing `[installer-ahead]` (lines 95, 119) before the header (line 123), so an unanchored match misplaces the key; say "the line that is exactly `[installer-ahead]`". F-tk's file has no such comment and row 76 cannot see placement; the PR's CI would reject it (fails closed).
- M12 [fold] A Ctrl-C or crash inside the toolkit's PVS window leaves the checkout detached at (b) and no trap runs (4a starts at the tag push): P11 then P12a say "finish as cut-release.sh printed", nothing was printed, and if (b) had landed the P12a "drop it" advice discards (c) and P19 then refuses. Conditions: signal inside PVS and (b) already landed.
- M13 [pre-existing; the fold added more] Printed commands name `cut-release.sh` bare (`--verify`, `--land-pr 77`, P15 hint), but §2 says other repos reach it by absolute path, so they are not runnable as printed from a toolkit checkout; print the script's absolute path (the banner already has it).
- M14 [pre-existing] Network row 75 inserts `## ms-cli [Unreleased]` unconditionally; once the real CHANGELOG carries one, C2 fires and the row fails (row 76 handles the same case).
- M15 [pre-existing] §5.4 has no FAIL text for a failed download (`gh release download` under `set -e` aborts `--verify` with exit 1 instead of a FAIL line and exit 8).

## Nit (one line each)

- N1 [pre-existing] Tag regex accepts `ms-cli-v0.20.03`; P22 may pass it numerically and cargo rejects the manifest (`invalid leading zero in patch version number`, measured) with step 4's misleading "run 'cargo fetch'" text; reject leading zeros in the tag regex.
- N2 [pre-existing] Row 4 pins only line 1 of `--help`; the exit-code table is unpinned.
- N3 [pre-existing] Row 25 lists `would check <lib tag>` before `would verify <bin tag>`; §5.2 puts verify first (rows match by membership, so harmless).
- N4 [fold] Review labels `(I-2)`, `(I-4)`, `(I-5)`, `(I-6)` remain inside rows 51, 65, 67, 68 and mean nothing to the builder.

## Design concerns (ungraded)

- V6 runs the archive's binary even after V4/V5 FAIL or when the sums file is unsigned (rows 37, 63, 71 pin that); consider running it only when V5 passed and the sums file verified.
- The resume path tags `HEAD` (origin tip) after PVS re-stages it; if commits sit on top of the original release commit it tags the later tip. It still has manifest N, a matching heading and CI, so not graded.

## Deferred minors in touched text (F-709), re-graded

| item | fold touched | result |
|---|---|---|
| M3 (which refusal text prints on a bypass) | §5.3 step 3 | stays Minor; the toolkit's by-hand "finish" line now prints after a detected bypass although the script knows the answer. Suggest omitting that line when the bypass phrase matched. Not C/I: it is preceded by "If the output above shows a bypass, STOP", and nothing is tagged |
| M4 (resume path undecided) | row 55/26, recovery text | stays Minor; the undecided parts are cosmetic lines, not exit status |
| M11 (§8(a) stops before `deploy.sh`) | the `git commit` line | stays Minor |
| M12 (toml insert branch has no row) | none, see new M11 | stays Minor |
| M13 ("unchanged" is false when origin moved) | same lines | stays Minor; I-1 is a separate defect |
| M15 (re-stage after a bypass) | none | stays Minor; CI is read fresh from GitHub for the sha |
| M17 (restore-on-failure for `--push`) | 4a added a post-tag trap only | stays Minor; the gap before the tag is M12 |
| N1 ("drop the local commit", toolkit has two) | P12a text | stays Nit |
| N2 (`--help` exit 6) | step 4 now has an exit 6 where some tags are on origin | stays Nit, widened; an operator who retries is stopped by P15 |
| N3, N5 | P15 / rows 42, 46 | stay Nits |

## Run what you print: commands executed

| spec text | command (read-only unless marked sandbox) | output |
|---|---|---|
| §5.3 step 3, PVS | `git push origin HEAD:refs/heads/ci/staging` for a second release commit while an old one sits there (sandbox) | `! [rejected] HEAD -> ci/staging (non-fast-forward)`, rc 1 |
| PVS source | `OUT="$(false 2>&1)"; echo "$OUT"` under `set -euo pipefail` | exits rc 1 before the echo, so the delete line is never reached |
| 4a, I-1 by-hand tag | `git tag -a mnemonic-toolkit-v0.105.2 -m '...' <c sha> && git push origin refs/tags/...` (sandbox bare origin, (b) landed, master at (c)) | `* [new tag] mnemonic-toolkit-v0.105.2` |
| 4a pins-PR line | `git checkout --quiet --detach <c> && sh scripts/install-assets.test.sh && git checkout --quiet master && git push origin <c>:refs/heads/release-pins-0.105.2 && gh pr create ... --body-file <git-dir>/cut-release/pins-pr-body.md` (sandbox, stub gh) | from master at (c) and from master reset to (b): `[install-assets.test] OK`, `* [new branch] <c> -> release-pins-0.105.2`, stub prints `.../pull/77`; if the test fails the chain stops with HEAD detached |
| I-6 recipe | sed + `git commit -m ... -- demo/sh2/src/index.html demo/sh2/CLI_SPINE.md` with a modified `README.md` (sandbox) | 2 files committed; `M README.md` still uncommitted; a second paste: `no changes added to commit`, rc 1 |
| §7 | `gh pr view 123 --repo bg002h/mnemonic-toolkit --json state,isDraft,baseRefName,headRefName,headRefOid,mergeStateStatus,url,isCrossRepository` | `MERGED`, `isCrossRepository:false`, head `docs/bughunt-ms-manual` |
| §7 | `gh pr checks 123 ... --json name,bucket,link` | rc 0; `pass=30 skipping=2` |
| E9 | `gh pr list --repo bg002h/mnemonic-toolkit --head docs/bughunt-ms-manual` | empty (default state is open only; fine for a just-created PR); with `--state all` one `MERGED` PR |
| I-1 text | `gh run list --repo bg002h/mnemonic-toolkit --branch mnemonic-toolkit-v0.105.1` | 8 runs all success: changelog-check, fuzz-smoke, bitcoind-differential, release, man-pages, miniscript-fork-tripwire, sibling-pin-check, install-pin-check |
| step 5 flake text | `gh run rerun --help` | `--failed` and inherited `-R, --repo` exist |
| step 5 | `gh pr create --help` | `--body-file`, `--draft` exist; "Use `--head` to explicitly skip any forking or pushing behavior" |
| §7 step 2 | in a sandbox clone: `git fetch -q origin release-pins-0.105.2`, then `git fetch -q origin` | tags `[]` after the first, `[mnemonic-toolkit-v0.105.2]` after the second; `git merge-base --is-ancestor "<tag>^{}" ...` with the tag absent: rc 128 |
| §3 / branch protection | `gh api repos/bg002h/<r>/branches/<b>/protection` for 7 repos | 6 protected: `strict:false`, `enforce_admins:false`, `required_linear_history:false`, no required reviews, contexts engrave 2, toolkit 9, ms 6, md 2, mk 1, gui 5; mt `main` unprotected. All 7 allow merge, squash and rebase; `delete_branch_on_merge:false` |
| §6.3 | toolkit `release.yml`, `install-pin-check.yml`, `changelog-check.yml` | none checks that the tag commit is on master (claim holds); `install-pin-check` regex `mnemonic-toolkit-v[0-9]+\.[0-9]+\.[0-9]+` (F-709 M9) |
| §5.4 signing | `grep -c "minisign -S"` in each release workflow | 1 in every repo's release workflow today, so the old unsigned releases below are history |
| §5.4 V1-V6 (`bin/vcheck.sh` from r1, newest release per bin prefix) | gh release view/list/download, `minisign -Vm <s> -x <s>.minisig -P RWRUl0DY...`, awk + `sha256sum -c`, tar, `timeout 30 <bin> --version` | ms-cli-v0.20.2: 3 sums files signed, `Trusted comment: ms ms-cli-v0.20.2 SHA256SUMS.x86_64` (also aarch64, portable), `ms 0.20.2` (7 ok, row 69). mt-cli-v0.1.1: `Trusted comment: mt mt-cli-v0.1.1 SHA256SUMS.portable`, `mt 0.1.1` (5 ok, row 70). md-cli-v0.20.3: no `.minisig` for aarch64 and portable, `md 0.20.3` (4 ok, 2 failed, row 71). v0.12.0: `Signature key id in SHA256SUMS.minisig is CA39ECB257009A0F` / `but the key id in the public key is EF2B8D34D8409754`, `me 0.12.0` (row 72). toolkit-v0.105.1, mk-cli-v0.13.0, gui-v0.63.0: unsigned; every archive in exactly one sums file, `OK`, one file named `<bin>`, version correct |
| §2 | `dirname "$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")"`; `sha256sum` of the PVS copies; `gh api .../contents/scripts/push-via-staging.sh` | `/scratch/code/shibboleth`; `c12a644a676b` in engrave, toolkit, ms and md; mk, mt, gui `404` |
| §5.1/P-checks | `git ls-remote origin 'refs/tags/ms-cli-v0.20.2^{}'`; `--tags` on an absent tag; `cargo metadata --no-deps --format-version 1 --offline`; `cargo --version` | `62b4d9dc9035...^{}`; empty, rc 0; ms-cli -> ms-codec `=0.10.1`; rustup `1.85.0` vs `/usr/bin/cargo` `1.98.1`; `rust-toolchain.toml` channel `1.85.0` |
| §5.1 step 8 | `./demo/sh2/check-install-links.sh` (engrave master) | `OK: every advertised URL resolves and names the newest release of its kind.` |
| §6.3 step 4 | `sh scripts/install-assets.test.sh` (toolkit master) | `ok 32 of 32 mapped assets verified` ... `[install-assets.test] OK` |
| §6.2 | `check_cli_pins.py ...` on master; first `mnemonic-toolkit-v` in `install.sh`; `first_signed` line | `OK: cli-pin-consistency ... installer ahead: ms 0.20.2`; `mnemonic-toolkit-v0.105.1`; line 189 `        mnemonic)     echo "" ;;` |
| row 32 | recompute the repad on the real `--list` line | equals the spec's expected line |
| row 40 | tk_examples_b forms on the real files | gen.sh 5 lines, 0 X left; Examples.md 4 lines, 4 installer lines left |
| fold gates | `mech_checks.py`, `row49_check.py` on the head | PASS (76 rows, 68 offline; L1=3, L2=3; 5 controls fail) |
| fold fidelity | every `cut-release: ` text of the r1 report searched in the spec | 20 of 20 verbatim |
| not run (writes) | `cut-release.sh` (does not exist), PVS, tag/branch pushes against real remotes, `gh pr create`, `gh pr merge` | |
