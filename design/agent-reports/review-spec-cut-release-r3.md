r3: 0C/0I/6M/3N

# Review r3: `design/SPEC_cut_release.md` at 4869cbb7 (fold 313c637a), last-round hybrid, one Sonnet xhigh slice

- Brief: `coordinator/briefs/review-spec-cut-release-r3.md`. Worktree `/scratch/code/shibboleth/me-worktrees/cut-release-spec` (branch `spec/cut-release`). Scope: the fold diff in full depth, plus every read-only command the whole spec prints. Nothing was pushed, tagged, merged or created on any remote; all probes ran under `/scratch/cache/review-cut-release-r3/` (scripts `bin/`, outputs `out/`, plants `plant/`, sandboxes `sb-*`).
- **Outcome: 0 Critical, 0 Important.** I-1, I-2 and I-3 are closed: nothing in the folded text can push a wrong tag, tag a wrong commit, land a branch it should not, or make a remote write under `--dry-run`. Six Minors and three Nits remain; every one fails closed, none changes a tag or a landing. Both fold-flagged open points are real but Minor (M4) and Nit (N1).
- Method note: the `/scratch/cache/cut-release/fold-r2/` scripts reproduce PASS on the head (mech: rows 1-78, 70 offline + 8 network, counts 78/70/78; row 49: L1=4, L2=3, 6 controls caught). I did not rely on them: I planted faults (Q4), wrote an independent exit-code cross-check (`bin/exit_cross.py`, 17 quoted §4 texts, 0 mismatches) and measured bash behaviour of the trap design in real processes.

## Q1. I-1, I-2, I-3: states walked

| state | what the folded text does | verdict |
|---|---|---|
| I-1a no `ci/staging` on origin | P23 silent | ok |
| I-1b stale S (unrelated or descendant of the tip), `--push` or `--dry-run` | exit 3 before any change; prints the text and the delete command; after `git push origin --delete ci/staging` P23 passes and the staging push succeeds (sandbox, printed command run literally) | ok |
| I-1c S = HEAD (PVS landed the commit, then died before `--delete`; or bypass-kept) | allowed; PVS's staging push is a no-op (resume path) | ok, no row (M2) |
| I-1d S = proper ancestor of HEAD (old release's staging kept, master since moved) | refused, though a fast-forward would succeed (sandbox `sb-anc`: `a24a40f..5a70c93  HEAD -> ci/staging`) | false text, fails closed, remedy works: M2 |
| I-1e Ctrl-C in PVS's CI wait, then re-run | P12a (drop the commit) then P23 (delete) then success: two refusals, both name a working command | ok |
| I-1f Ctrl-C in the toolkit PVS window | detached at (b): P11, P12a, then P23; the P12a "finish as printed" gap is F-710 M12 | unchanged |
| I-2a `set -e` abort (failing `gh`) during the wait | EXIT trap: "stopped" line, reset (toolkit), finishing lines, exit 7; tags stay | ok (sketch rc 7, one "stopped" line) |
| I-2b explicit exit 7/8/9 | sets the third variable; EXIT trap silent; exit code kept (sketch exit 8: rc 8, 0 "stopped" lines) | ok |
| I-2c normal DONE | DONE variable set first; EXIT trap silent; rc 0 | ok |
| I-2d INT/TERM/HUP | signal trap exits 7; EXIT trap silent. Real SIGINT to a process group (python `killpg`): `trap ... INT` then EXIT trap sees rc=7; with no INT trap rc=0, so the spec's "$? = 0" measurement holds | ok |
| I-2e re-run after any of the above | non-toolkit: P15 hint. Toolkit: master was reset to (b), so P12 passes and P15 refuses with the hint | ok |
| I-2f exit 6 after the flag is set (peel mismatch in step 4's "Afterwards") | guard lists only 7, 8, 9: EXIT trap rewrites exit 6 to 7 (`bin/trap_probe.sh exit6_after_push`: rc=7, "stopped" + finish printed after the exit-6 text) | M1 |
| I-2g abort inside the toolkit reset in the trap | `set -e` aborts the trap: rc=128, no finish line (`sb-reset`) | M6 |
| I-3 `--dry-run` that calls the PVS seam | `ls-remote` unchanged (old rows pass); `$T/pvs.log` non-empty (new clause fails the row) (`sb-pvs`) | ok |

## Q2. Can rows 69, 70 and the changed rows 25, 26, 40, 48, 49 fail?

| row | rule-break tried | caught? |
|---|---|---|
| 69 | P23 missing (`--push` and `--dry-run` legs both exit 0) | yes, exit code |
| 69 | wrong exit / wrong text / `<12>` taken from HEAD / refusal after the commit / PVS called | yes (exit, text, `<12>` = the unrelated commit, HEAD unchanged and tree clean, `pvs.log`) |
| 69 | P23 only on one mode | yes (the row runs both) |
| 69 | S = HEAD refused (resume exemption broken) | **no row** (M2) |
| 70 | no EXIT trap (exit 1), exit 8 instead of 7, no finish line, no master reset, no pins-PR line | yes (exit 7, both stderr lines, `follow-on.txt`, local master = bare master = (b)) |
| 70 | trap fires on DONE or overrides exit 8/9 | yes, by rows 33/42 (exit 0) and 37-39, 61-64, 43 (exits 8/9) |
| 70 | builder retries a failing `gh` instead of aborting | yes (exit 0 or a different text) |
| 70 | toolkit leg under a "first call" fake | would fail for the wrong reason: M3 |
| 25/26/40/48 | dry run calls the PVS seam | yes (`pvs.log` non-empty); also tag/branch push (`ls-remote`), `gh` call (`gh.log`) |
| 49 | `git push` bypasses: `\|\| die`, `printf; git push --tags`, force spliced into a text, 4th plain push, comment naming it | yes (L2 count or L1 forbidden char); the pre-fold 3-prefix rule rejects the P23 line, so the edit is needed |

Row 49's wording was closed by hand (inline python, not saved): the four prefixes in the spec's row 49 equal the prefixes of the four `git push` operator texts (lines 189, 327, 351, 479), each used once; §2 says "four".

## Q3. The two points the fold left open

1. **EXIT trap and step 6's "== follow-on pins are NOT listed" line: undecided, changes one stdout line, no row sees it. Minor (M4).** Literal §5.3 step 6 ("Exits 7 and 8 also print ...") says yes; the EXIT-trap sentence lists its output without it. Decide **no**: an abort (and a signal) can come after a clean verify, in the §8 block, where "the release is not verified" would be false; the finishing lines are true in every state. Fix text below (M4).
2. **"The trap, and every exit 7, 8 or 9" with two traps: no effect on printed text, exit status or a row. Nit (N1).** The EXIT-trap sentence itself says it prints the finishing lines, and row 59 pins the signal trap's, so both readings print the same.

## Q4. Gate scripts, planted faults (`plant/`, `bin/plant_mech.py`, `bin/plant_r49.py`)

| plant | script | result |
|---|---|---|
| `as 76` -> `as 79` (row 77); `rows 28-32` -> `28-99` | mech | FAIL (reference to a missing row) |
| row 69 or 70 exit `1` (undefined); exit lead removed | mech | FAIL |
| row 51 cites `P99`; `P12a` definition renamed | mech | FAIL (not defined) |
| count sentence 77; step 5 `76 passed` | mech | FAIL |
| row 69 or 70 exit changed to a *defined but wrong* code (5, 8) | mech | **PASS** (not caught) |
| row 69 operator text altered; `P23` row deleted while row 69 still quotes it | mech | **PASS**, `unmatched` rises 1 -> 2 (check 5 prints, never fails) |
| backtick, `"`, `$(` inside the P23 or pins-PR text; P23 prefix changed | row49 | FAIL (the prefix plant crashes rc 1) |
| spec's own row 49 loses the P23 prefix, loses the `--mirror`/`HEAD:` bans, says L2 = 4; §2 says "three" | row49 | **PASS** (the script uses its own hard-coded rule, not the spec's wording) |

All four faults the brief names FAIL. The gate is not a false PASS, but it has blind spots (M5). `bin/exit_cross.py` catches the defined-but-wrong exit code (row 69 -> 5, row 65 -> 9 both flagged).

## Q5. Run what you print: executed (outputs in `out/`)

| spec text | command | result |
|---|---|---|
| P23 | `git ls-remote origin refs/heads/ci/staging` on the 7 real repos | rc 0, empty on all 7 |
| P23 printed fix | the spec's text extracted, `git push origin --delete ci/staging` run in a sandbox with a stale ref | `- [deleted] ci/staging`; P23 then silent |
| P15 / P18 | `git ls-remote --tags origin refs/tags/ms-cli-v0.20.2` (+ `^{}`), absent tag | `993dd1da...`, peeled `62b4d9dc...`; absent: empty rc 0; newest ms-cli tags 0.20.1, 0.20.2 |
| §2 default | `dirname "$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")"` | `/scratch/code/shibboleth` |
| P14/P16 | `cargo --version` (system 1.98.1; rustup 1.97.0-nightly), `rust-toolchain.toml` 1.85.0, `cargo metadata --no-deps --format-version 1 --offline` | ms-cli 0.20.2 -> ms-codec `=0.10.1` |
| §5.3 step 5 | `gh run list --repo bg002h/mnemonic-secret --branch ms-cli-v0.20.2 --event push --limit 50 --json ...`, `gh run view` | fuzz-smoke, release, man-release all completed/success |
| §3 grounding | runs per latest tag, 7 prefixes plus libs | toolkit 8 runs, ms-cli 3, ms-codec fuzz-smoke only, md-cli 4, mk-cli CI+fuzz-smoke+musl-binaries, mk-codec-v0.5.0 CI+fuzz-smoke, mt release, gui build+schema-mirror: all agree with §3 |
| §7 | `gh pr view 123 ... --json ...`, `gh pr checks 123 ... --json name,bucket,link` | MERGED, isCrossRepository false; rc 0, pass=30 skipping=2 |
| §7 step 6, E9 | `git ls-remote origin refs/heads/release-pins-0.105.1`; `gh pr list --head docs/bughunt-ms-manual --state all` | empty rc 0; one MERGED PR |
| §5.4 V1-V6 | `bin/vcheck.sh` (r1) on ms-cli-v0.20.2, mt-cli-v0.1.1, md-cli-v0.20.3, v0.12.0, toolkit-v0.105.1, mk-cli-v0.13.0, gui-v0.63.0 | rows 71-75 reproduce: ms 3 sums signed + `ms 0.20.2`; mt `mt 0.1.1`; md no `.minisig` for aarch64/portable + `md 0.20.3`; engrave `Signature key id ... CA39ECB257009A0F` + `me 0.12.0`; toolkit, mk, gui unsigned (so 5 of 7 releases fail bare `--verify` today) |
| §8 | `git grep -n -w -F 'ms-cli-v0.20.2' origin/master -- . ':!design' ...` in toolkit; `--tag` mirrors grep; zero-hit case | 9 hits / 3 mirror lines; zero hits give rc 1 (a builder must not let it abort under `set -e`; no row has a sibling with zero hits, harmless: the block runs before the tag push too) |
| §8(a), §6.2, §6.3 | `./demo/sh2/check-install-links.sh`; `check_cli_pins.py`; `sh scripts/install-assets.test.sh` | `OK: every advertised URL resolves ...`; `OK: cli-pin-consistency ... installer ahead: ms 0.20.2`; `32 of 32 ... [install-assets.test] OK` |
| help claims | `gh run rerun --help`, `gh pr create --help` | `--failed`, `--repo`; `--body-file`, `--draft`, `--head` exist |
| §3 / §2 | `gh api .../branches/<b>/protection` x7; `sha256sum` of the PVS copies | 6 protected (contexts 2/9/6/2/1/5), mt `main` 404; `c12a644a676b` on engrave, toolkit, ms, md; mk, mt, gui have none |
| bash syntax | `bash -n` on the commands inside P23, the pins-PR line, the toolkit by-hand line, the §6.3 step 4 text (placeholders filled) | rc 0 each |
| not run | every write (tag push, `pr create`, `pr merge`, branch delete, `git -C <sib> fetch` on real checkouts), `cargo build` | |

## Q6. Advice lens on the added or changed operator texts

- P23: followed literally in states I-1a to I-1e. `git push origin --delete ci/staging` runs as printed from any table repo, deletes only that branch, cannot tag or land. In I-1d the sentence "cannot push a new release commit over it" is false (M2); the command is still harmless.
- EXIT-trap line and finishing lines: the "stopped ... (above)" text is accurate for an abort during the wait and misleading after a clean verify only if step 6's line were also printed (M4). The `cut-release.sh --verify <bin tags>` line is the pre-existing bare-name form (F-710 M13) and, for a lib-only run, the empty-`<bin tags>` form (F-710 M3); the pins-PR line was executed by r2 from both the exit-5 and exit-7 states and is unchanged.
- Row 70's and the fake's texts are harness-only.

## Minors

- **M1 [fold; text is the r2 reviewer's I-2 fix] The EXIT trap's guard omits exit 6, and the instant of "the variable set after the tag push" is undecided.** Measured: set right after `git push --atomic` returns, step 4's post-push `exit 6` (peel mismatch) becomes exit 7 with a "stopped by a failing command" line (`out/trap-exit6_after_push.err`). Conditions: a remote tag that peels differently or reads empty right after an atomic push, and the builder choosing that instant. Fails closed, the exit-6 text still prints first. Fix, anchor 4a bullet 1, old `and no exit 7, 8 or 9 and no signal trap has begun (a third variable they set).` new `and no exit 6, 7, 8 or 9 and no signal trap has begun (a third variable they set; step 4's exit 6 after the push sets it too, so the EXIT trap never changes an exit 6).` and, same bullet, after `and also EXIT.` insert ` The variable set after the tag push is set the moment git push --atomic returns 0, or when step 4 prints the "continuing" line.`
- **M2 [fold; r2 reviewer's text] P23 refuses when S is a proper ancestor of HEAD (state I-1d), prints a false reason, and the S = HEAD exemption has no row.** Measured in `sb-anc`. Conditions: a kept/bypass staging from an earlier release, and master moved past it before the next release. Fix: P23 condition old `prints a sha S, and S is not the sha of \`HEAD\`` new `prints a sha S, S is not the sha of \`HEAD\`, and \`git merge-base --is-ancestor <S> HEAD\` exits non-zero (an S unknown locally counts as not an ancestor)`; after the §4 table add `P23: <12> is the first 12 characters of S. S = HEAD (the resume path) and S behind HEAD (an old staging a fast-forward covers) are allowed.`; row 69 setup old `pushes an unrelated commit to` new `pushes a commit that is not an ancestor of the bare master tip to`, and append to its expect `; then, with ci/staging set to the sha of HEAD and then to its parent, --dry-run ms-cli-v0.20.3 exits 0 both times with $T/pvs.log absent or empty`.
- **M3 [fold; r2 reviewer's text] `FAKE_GH_FAIL=1` "on its first `run list` call" is ambiguous in row 70, which runs two cut-release processes.** A gh.log- or marker-derived "first" fails only the ms leg; the toolkit leg then exits 0 and the row goes red for a fixture reason. Test-only. Fix, old `on its first \`run list\` call fake gh prints` new `on every \`run list\` call fake gh prints`, and append `(the script makes no \`run list\` call before the tag push)`.
- **M4 [fold; the open point 1] Whether the traps print step 6's NOT-listed line.** Conditions for a wrong claim: an abort or signal after a clean verify (the §8 block) and a builder printing it. Fix: 4a bullet 1, old `(for the toolkit after the §6.3 step 6 reset), and exits 7.` new the same followed by ` Neither trap prints step 6's \`== follow-on pins are NOT listed\` line: an abort can come after a clean verify, and the finishing lines are true in every state.`; §5.3 step 6, old `Exits 7 and 8 also print` new `Exits 7 and 8 from steps 5 and 6 (not the traps' exit 7) also print`.
- **M5 [fold; gate tooling] Gate blind spots (Q4).** `mech_checks.py` check 5 (quoted text matches a template) prints but never fails; a defined-but-wrong exit code and a deleted-but-quoted refusal pass; `row49_check.py` ignores the spec's row 49 and §2 wording; none of the scripts is committed. Fix: `fail()` on any unmatched text except the known row-59 header; add `bin/exit_cross.py`; read row 49's prefixes and counts from the spec; commit the scripts under `scripts/` with a "does not cover" line (CLAUDE.md pattern).
- **M6 [pre-existing r1 text, a path added by the fold] A failing toolkit reset inside an exit path is undecided; under `set -e` it aborts the exit path.** Measured (`sb-reset`): an uncommitted edit to `install.sh` makes `git reset --keep` fail, the trap returns rc 128 and prints no finishing line. Conditions: toolkit run, any post-tag exit, tracked edits to a file that differs between (b) and (c) during the wait. Fix, §6.3 step 6, old `does the same reset first.` new the same followed by ` A failing reset is ignored: its git error shows, and the exit code and the finishing lines are unchanged.`

## Nits

- N1 [fold; open point 2] 4a bullet 2, old `- The trap, and every exit 7, 8 or 9, ends with:` new `- The signal trap, the EXIT trap and every exit 7, 8 or 9 end with:`.
- N2 [fold] `--help` line `7 a release run failed` no longer covers the signal and EXIT traps' exit 7: new `7 a release run failed or the script stopped after the tag push` (row 4 pins only line 1).
- N3 [fold] The fold report says "no deviation", but the r2 fix's P23 parenthetical (`<12>` = first 12 of S; S = HEAD is the resume path) is absent from the spec. Folded into M2's fix.

## Design concerns (ungraded)

- After a trap, `--verify` checks release assets only, so the finishing line does not send the operator to the tag's other workflow runs (`gh run list --repo <slug> --branch <tag>`) that the aborted wait never judged.
- The toolkit pins-PR line exists only in the terminal output of the run that stopped; a re-run meets P15, whose hint and `follow-on.txt` do not carry it. Writing it into `follow-on.txt` would survive a closed terminal.

## Q7. Fold accounting for 313c637a (Sonnet xhigh author; fix texts by the Sonnet xhigh r2 reviewer)

Of 0C/0I/6M/3N: **5M and 3N come from the fold**; 1M (M6) is pre-existing text. Of the eight, M1, M2, M3 and N2 sit in the r2 reviewer's prescribed texts (applied verbatim: 11 of 12 fix-text blocks machine-matched, the 12th lacks the P23 parenthetical, N3); M4 and N1 are the two points the fold author flagged; M5 is the author's gate adaptation; N3 is the author's omission. FOLLOWUPS: F-710 is byte-identical to the r2 review's 15 M, 4 N and 2 concerns (machine-compared); the F-709 pointer sentence moves +2 correctly.

## Q8. Deferred minors in text the fold changed (re-graded)

None becomes C or I. F-709 M1 (no mode column): P23 carries its mode, stays Minor. F-709 M15 (re-stage after a bypass): P23's S = HEAD exemption allows that state, stays Minor. F-709 M17 (trap stated for `--dry-run` only): the new post-tag EXIT trap leaves the pre-tag gap, stays Minor. F-709 N2 and F-710 M7 (exit 6 wording, no row for exit-6 outcomes): now coupled with M1, stay Minor/Nit. F-709 N4 (row 49 refs): stays Nit. F-710 M3 (empty `<bin tags>`): the EXIT trap makes it reachable on a lib-only abort, needs two conditions, stays Minor. F-710 M6: row 70 now asserts the reset and finish line for the EXIT path only, stays Minor. F-710 M12: P23 adds a third refusal to that recovery path, stays Minor. F-710 M13 (bare `cut-release.sh`): text unchanged, reach widened by the EXIT trap; r2's Minor grade stands.
