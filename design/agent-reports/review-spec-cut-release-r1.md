r1: 0C/7I/20M/5N

# Review r1: `design/SPEC_cut_release.md` (whole artifact), one Opus slice

- Artifact: `design/SPEC_cut_release.md` at `9de7d34a`, branch `spec/cut-release`, worktree `/scratch/code/shibboleth/me-worktrees/cut-release-spec`. Author: the Opus spec thread. This is round 1, so every finding is **[pre-existing]**. No fold exists yet, and there are no deferred minors to re-grade.
- Brief: `coordinator/briefs/review-spec-cut-release-r1.md`, following `review-brief-template.md`.
- Grades use RULES.md and its likelihood test.
- Nothing was pushed, tagged, merged or created. All probes ran in scratch clones under `/scratch/cache/review-cut-release-r1/` (clones, downloads, and the scripts `bin/vcheck.sh`, `bin/recipe-b.sh`, `bin/refs-d.sh`).

## The five questions, answered

1. **Wrong tag, wrong commit, wrong landing, bypass, false success.**
   - **What holds.**
     - The spec never puts a tag on anything but the release commit (the toolkit's (c)).
     - It lands a default branch only through push-via-staging.sh (PVS) or `gh pr merge --merge` with CLEAN required and no `--admin`.
     - It reports success only after V1-V6. I ran V1-V6 by hand on the newest release of all seven bin prefixes, and they give the right verdict for each (table in Q4).
     - All six protected default branches have `enforce_admins: false`. So PVS's bypass check and gh's own refusal of a BLOCKED merge are what stop an admin bypass, and the spec keeps both.
     - A dry run interrupted by Ctrl-C still restores files: bash runs the EXIT trap on SIGINT (measured).
   - **What breaks is at the failure points:**
     - I-1: the toolkit after (b) lands but PVS exits non-zero.
     - I-2: the toolkit's local master is left at (c).
     - I-3: every exit after the tag push, including Ctrl-C, the 90-minute waits and the atomic-push claim.
     - I-4: a mistyped version is tagged for good.
     - I-5: `--land-pr` can delete a branch it did not merge.
2. **Does `--dry-run` make zero remote writes? Yes, in every path.**
   - Its only remote I/O is reads: `git fetch` and `ls-remote` on origin, the sibling `git fetch`, and engrave's `check-install-links.sh`, which makes curl GETs.
   - It makes no gh call (P1 for the dry run has no gh), does not call PVS, and pushes nothing.
   - The gap is in the tests: rows 25, 26, 40 and 48 check only the bare remote's refs (I-7).
3. **Rows that cannot fail, and rules without rows.**
   - Four row groups cannot fail on the property they guard: row 46 (merge method), rows 25/26/48 and 40 (gh writes; HEAD for the toolkit), and row 11 (only the behind case).
   - Twelve safety rules have no row. Both lists are in I-7.
   - The low-risk uncovered refusals are M20.
4. **Run what you print.** Every read-only command the spec prints was executed (table at the end).
   - Every §3 claim checks out, as do rows 51-54's expected texts and counts, given M2's V4-skip reading.
   - The §8(a) recipe and the §8(b) sed check out.
5. **The six author decisions: keep all six.** Two are conditional on I-1 and I-4 (table below).

## Important

### I-1 [pre-existing] Toolkit: the "re-run the same command" advice fails twice once (b) has landed (§5.3 step 3, §6.3 step 1, P12, P19)

**Scenario.**
- A toolkit `--push` reaches PVS. PVS lands (b) on master, then exits non-zero. Three things can cause that: its bypass check (the race its own header measured on 2026-09-25), the `ci/staging` delete, or its final fetch.
- cut-release prints: `the release commit IS on origin/master ... otherwise re-run the same command: it finds the commit on master, re-stages it and tags.`
- But §6.3 step 1 has returned to local master, which is (c). So the re-run stops at P12, because HEAD (c) ≠ origin/master (b).
- If the operator then runs `git reset --hard origin/master`, the re-run stops at P19 ("toolkit already N"), and the untagged, unpushed (c) has been thrown away.

**Grade.** The printed recovery fails as printed, so this is Important.

**Fix.** Anchor (§5.3 step 3): ``cut-release: origin/<default> is unchanged. To retry, drop the local commit and run again: git reset --hard origin/<default>`.`` Append a new paragraph:

> For the toolkit, the IS-on-origin line is replaced by these two lines, because a re-run is refused (P12, then P19):
> `cut-release: the toolkit release commit (b) <12> IS on origin/master; its tag goes on (c) <12>, which is only on local master (do not reset it). If the output above shows a bypass, STOP and report it.`
> `cut-release: otherwise finish by hand: git tag -a mnemonic-toolkit-v<N> -m 'mnemonic-toolkit <N>' <c sha> && git push origin refs/tags/mnemonic-toolkit-v<N>; when its release, man-pages, install-pin-check and changelog-check runs are completed/success (gh run list --repo bg002h/mnemonic-toolkit --branch mnemonic-toolkit-v<N>): cut-release.sh --verify mnemonic-toolkit-v<N>, then the pins-PR line below.`
> followed by the pins-PR line of §5.3 step 4a (I-3).

Row: new 56.

### I-2 [pre-existing] Toolkit: local master is left at (c), and P12's advice then lands (c) outside the pins PR (§6.3 step 6, P12)

**Scenario.**
- After every successful toolkit release the spec says: "Local master stays at (c), one commit ahead of origin until the PR merges."
- Any `--dry-run` or `--push` in that checkout before the PR merges stops at P12, which says "(git pull --ff-only, or land your commits first)".
- The pull does nothing, because the checkout is ahead. "Land your commits first", read literally, means push-via-staging, and that lands (c) directly on master.
- CI stays green. I simulated (c) and ran `check_cli_pins.py`: rc 0, output `installer ahead: mnemonic 0.105.2`, with only the toml line added.
- So (c) reaches master without the GUI-manual `{#installer-ahead}` text that ruling 7 says the PR's author writes. The draft PR then has nothing left to merge.
- The same P12 text is also the only hint after an interrupted non-toolkit run: the local release commit is there and origin is unchanged. There it is harmless but incomplete, because the right action is the reset.

**Fix.**
1. Anchor (§6.3 step 6): `Local master stays at (c), one commit ahead of origin until the PR merges.` → `Then git checkout --quiet master && git reset --quiet --keep <b>: local master equals origin/master, and (c) stays reachable from the tag and release-pins-<N>. Every exit after the tag push (7, 8, 9, or a trapped signal) does the same reset first.`
2. Anchor: the P12 row. Replace it with two rows:
   - `| P12 | HEAD ≠ origin/<default>, and origin/<default> is not an ancestor of HEAD | (text unchanged) | 3 |`
   - `| P12a | origin/<default> is a proper ancestor of HEAD (HEAD is <k> commits ahead) | cut-release: HEAD <12> is <k> commit(s) ahead of origin/<default> <12>; the release commit must sit directly on the remote tip. A 'release: ...' commit from an interrupted cut-release.sh run: drop it (git reset --hard origin/<default>) and run again. A 'pins: installer -> ...' commit: finish as cut-release.sh printed when it stopped, and never land it without its pull request. Any other commit: land it first. | 3 |`

Rows: amend 42 (at exit, local master = bare master = (b)); new row 51.

### I-3 [pre-existing] After the tag push, no exit leaves a complete or runnable way to finish (§5.3 steps 4-6, §6.3 steps 4-5, P15)

The tags are irreversible, and from that point the script can wait up to 90 minutes per workflow. There are five gaps.

- **(a) Exits 7 and 8 lose the follow-on list.** They print "follow-on pins are NOT listed" and skip §8. Step 5's advice for a flake is `gh run rerun ... then cut-release.sh --verify <tag>`, and `--verify` prints no follow-on block. So after a successful rerun, nothing ever lists the engrave-demo or installer moves. A failed run is not hypothetical: md-cli-v0.20.3's `man-pages` run concluded `failure` (measured).
- **(b) Toolkit: the pins PR is never opened.** Exits 7, 8 and 9 all happen before §6.3 step 5. That PR is the only way (c), and with it the tag, reaches master (ruling 6), and no line says how to open it.
- **(c) An interrupt prints nothing.** A signal or crash during the waits leaves no message. The re-run stops at P15 ("already exists on origin") with no hint.
- **(d) Two exit-9 instructions fail.**
  - §6.3 step 4's exit 9 says "fix forward in a toolkit PR". A PR from master does not contain (c), so following it leaves the tag off master for good.
  - §6.3 step 5's printed `gh pr create ... --draft --title '...'` has no `--body`. gh 2.102's help says the body is prompted for unless `--body` is given, so a non-interactive agent's run is refused. Run interactively, the PR loses the owed block and the merge-commit warning.
- **(e) A failed atomic push is misreported.** It prints "(atomic; none is on origin)" without checking. That is false if origin accepted the push and only the client errored, or if another run pushed the same tag. The re-run's P15 then contradicts it.

**Fix.**

1. Anchor (§5.3 step 4): `If the push fails: delete the local tags (only those git ls-remote shows absent), print cut-release: pushing the tags failed (atomic; none is on origin): and git's output indented 3, then exit 6.` Replace with:

   > If the push fails, read `git ls-remote origin refs/tags/<t>^{}` for every tag.
   > - If every tag peels to `<sha>`, print `== the tag push reported an error, but origin has every tag on <12>; continuing` and continue.
   > - If no tag is on origin, delete the local tags and print `cut-release: pushing the tags failed (atomic; none is on origin):` with git's output indented 3. Then print `cut-release: re-run the same command; it finds the release commit on <default> and tags it` (for the toolkit, §5.3 step 3's two toolkit lines instead), and exit 6.
   > - Otherwise print `cut-release: after the failed push origin has <t> -> <x|absent>` (one line per tag), then `cut-release: STOP and report it`, and exit 6.

2. Insert after step 4:

   > **4a. Finishing lines.**
   > - Before the tag push, write the §8 block, computed from the pre-run state as a dry run prints it, to `<git-dir>/cut-release/follow-on.txt` (`<git-dir>` = `git rev-parse --absolute-git-dir`).
   > - The toolkit writes the §6.3 step 5 PR body to `<git-dir>/cut-release/pins-pr-body.md` before §6.3 step 1.
   > - From the tag push on, the script traps INT, TERM and HUP. The trap prints `cut-release: interrupted after the tag push; the tags are pushed and stay pushed: never delete or move them.` and exits 7.
   > - The trap, and every exit 7, 8 or 9, ends with:
   >   `cut-release: finish: cut-release.sh --verify <bin tags>; once it shows 0 failed, make the follow-on moves in <git-dir>/cut-release/follow-on.txt`
   > - For the toolkit, until §6.3 step 5 has opened the PR (and except after §6.3 step 4's exit 9), it then adds the pins-PR line:
   >   `cut-release: pins PR: git checkout --quiet --detach <c sha> && sh scripts/install-assets.test.sh && git checkout --quiet master && git push origin <c sha>:refs/heads/release-pins-<N> && gh pr create --repo bg002h/mnemonic-toolkit --base master --head release-pins-<N> --draft --title 'pins: installer -> mnemonic-toolkit-v<N>' --body-file <git-dir>/cut-release/pins-pr-body.md`
   > - The trap keys on a variable set after the tag push, never on `$?`. Measured: bash runs an EXIT trap after SIGINT with `$?` = 0.

3. Anchor (§5.3 step 6): `Exits 7 and 8 also print == follow-on pins are NOT listed: the release is not verified and skip §8.` → append `; they end with the step 4a finishing lines.`

4. Anchor (§6.3 step 4): `The tag is published; fix forward in a toolkit PR.` → `The tag is published. Fix forward on top of (c): git push origin <c sha>:refs/heads/release-pins-<N>, commit the fix to that branch, then gh pr create --repo bg002h/mnemonic-toolkit --base master --head release-pins-<N> --draft --title 'pins: installer -> mnemonic-toolkit-v<N>' --body-file <git-dir>/cut-release/pins-pr-body.md (a PR without (c) leaves the tag off master).`

5. Anchor (§6.3 step 5): the printed recovery `... --draft --title 'pins: installer -> mnemonic-toolkit-v<N>'` → append ` --body-file <git-dir>/cut-release/pins-pr-body.md`.

6. P15's on-origin text gains a second line: `cut-release: if an interrupted cut-release.sh --push pushed it: cut-release.sh --verify <tag>, then the moves in <git-dir>/cut-release/follow-on.txt`.

Rows: amend 36 (it also shows the finish line); new rows 57, 58, 59, 60.

### I-4 [pre-existing] A mistyped version is tagged for good, because nothing ties N to the next version (P17, P18; author decision 5)

**Scenario.**
- P17 accepts any N above the manifest's X, and P18 accepts any N above origin's newest tag.
- There is no prompt (decision 5), and the dry run is optional.
- `--push ms-cli-v0.30.3` (meant 0.20.3) passes every check. The manifest becomes 0.30.3, the heading becomes `## ms-cli [0.30.3]`, and the tag is pushed ("never delete or move them"). From then on, P18 refuses 0.20.3.
- Only C6 cross-checks N, and only in files that carry `**Next release: N**`. Today that is mk-cli alone.
- This does not fail closed: the result is a wrong release version, and it cannot be undone.

**Why the rule is relative to the manifest.** Historical tags skip versions relative to earlier tags (md-codec-v0.42.0 → 0.48.4, ms-codec-v0.9.0 → 0.10.1, descriptor-mnemonic-md-cli-v0.17.0 → 0.19.0; measured), because untagged versions were published in between. So the rule must compare N with the manifest's X, not with a tag.

**Fix.** Insert this row after the P18 row:

`| P22 | N ≠ X, and N is not a successor of X. With no '-' suffix on X, N is none of X.Y.(Z+1), X.(Y+1).0, (X+1).0.0. With a '-' suffix, triple(N) ≠ triple(X) | cut-release: <tag>: <crate> is <X> in <manifest>; the next version is one of <X.Y.Z+1> <X.Y+1.0> <X+1.0.0> (a skip is released by hand). For a '-' suffix: cut-release: <tag>: <crate> is <X> in <manifest>; the next version is <triple(X)> (a skip is released by hand) | 3 |`

Every N in rows 1-58 is already a successor, and rows 14/15 still fire P17/P18 first. Row: new 65.

### I-5 [pre-existing] `--land-pr` deletes `headRefName` on origin without checking that it is the branch it merged (§7 steps 1, 6)

**Scenario.**
- Step 6 runs `git push origin --delete <headRefName>` once `headRefOid` is on origin/<default>.
- Nothing checks that the head lives on origin. For a cross-repository PR (a fork's `patch-1`), headRefName names the fork's branch, and an unrelated origin branch with the same name is deleted.
- Nothing checks that origin's branch still points at `headRefOid`. A commit pushed to the branch after the merge is deleted unmerged.
- RULES.md: "Worktrees and unmerged branches still need his word". The 2026-10-14 exception covers only a PR's merged head branch.
- `gh pr view` offers `isCrossRepository` (measured on toolkit PR 123).
- This is data loss on origin, not a fail-closed refusal, so the likelihood test does not lower it.

**Fix.**
1. Anchor (§7 step 1): `--json state,isDraft,baseRefName,headRefName,headRefOid,mergeStateStatus,url` → add `,isCrossRepository`.
2. Anchor (§7 step 6): `Then git push origin --delete <headRefName>, allowed once ancestry holds (Brian 2026-10-14). Print == merged PR #<n> (<url>) as a merge commit; <headRefName> deleted, exit 0.` Replace with:

   > Then, only if `isCrossRepository` is false and `git ls-remote origin refs/heads/<headRefName>` prints `<headRefOid>`, run `git push origin --delete <headRefName>` (allowed once ancestry holds, Brian 2026-10-14) and print `== merged PR #<n> (<url>) as a merge commit; <headRefName> deleted`.
   > Otherwise print `== merged PR #<n> (<url>) as a merge commit; <headRefName> left in place (<reason>)`, where `<reason>` is `it is a fork's branch` or `origin's <headRefName> is <12>, not the merged <12>`.
   > If the delete fails, print `cut-release: merged, but deleting <headRefName> failed:` with git's output indented 3, and exit 9. Otherwise exit 0.

Row: new 67.

### I-6 [pre-existing] §8(a)'s printed `git commit -am` commits every tracked change in the engrave checkout (advice lens)

**Scenario.**
- In an engrave checkout with any tracked modification, `-am` commits that work under the demo message.
- The next printed line, `scripts/push-via-staging.sh master`, then lands it on master. PVS's tracked-files guard sees a clean tree because `-a` committed everything.

**The rest of the recipe is exact.**
- Replayed on the pre-bump files of the three real demo bumps (76b7a2d2 ms, 4a0afb3f md, e5c0a52f toolkit+ms), it is byte-identical to each commit.
- On today's files for ms 0.20.3 it changes 7 lines and leaves 0 stale.

**Fix.** Anchor: `     git commit -am 'demo/sh2: install block advertises <label> v<N>'` → `     git commit -m 'demo/sh2: install block advertises <label> v<N>' -- demo/sh2/src/index.html demo/sh2/CLI_SPINE.md`. Any other tracked change then stops PVS with "tracked files are modified".

Row: new 68.

### I-7 [pre-existing] §9 leaves safety rules without rows, and four row groups cannot fail on the property they guard (Q3)

**Rules without a row** that guard an irreversible step or a success claim:
- P12 when HEAD is ahead of origin
- P19 and P20
- PVS exiting 0 without `== OK:`
- PVS exiting non-zero with the release commit on origin, and the re-run it claims works
- the tag-push failure (exit 6)
- a run that never appears (exit 7)
- V1 draft or prerelease
- V4 trusted-comment mismatch
- V5 checksum mismatch
- V6 wrong version
- §7 merge state not CLEAN

**Rows that cannot fail:**
- **Row 46.** The fake `pr merge` does a `--no-ff` merge whatever flags it gets, so a script that passes `--squash` or `--rebase` still passes the row. On the real repos it would leave the tag off master; step 6 catches that only after the merge. All seven repos allow squash and rebase merges (measured).
- **Rows 25, 26, 48.** "Nothing was pushed" is checked only on the bare remote's refs, so a dry run that makes a gh write passes.
- **Row 40.** It checks only "tree clean". A toolkit dry run that makes commits (b) and (c) passes, and leaves master two commits ahead: §6.2 places those commits without saying "`--push` only".
- **Row 11.** It tests only the behind case. A P12 implemented as `merge-base --is-ancestor origin/<default> HEAD` passes it, and lets unpushed local commits ride the release commit through staging.

**Fix.**

1. **Amendments.**
   - The "After" clause of rows 25, 26, 40 and 48 adds: `$T/gh.log` absent or empty; the bare remote's `git ls-remote` output equals its pre-run output; HEAD unchanged; no local tag named in the command. Row 40 also adds: no bare `release-pins-*`.
   - Row 46 adds: gh.log has exactly one `pr merge` line, `pr merge 77 --repo bg002h/mnemonic-toolkit --merge --match-head-commit <c sha>`. The fake `pr merge` answers only that argv shape; any other prints `fake gh: unexpected: <args>` and exits 99.
   - §6.2 anchor `Commit (b) from the index` → `On --push only (at §5.3 step 1), commit (b) from the index`.

2. **Fixture additions.**
   - fake-pvs `silent`: pushes, prints nothing, exits 0.
   - fake-pvs `landed-fail`: pushes, prints `FATAL: origin/$1 is not <sha> after the push`, exits 1.
   - fake gh `FAKE_GH_KILL=TERM`: on the first `run list` call, runs `kill -TERM "$CUT_RELEASE_TEST_PID"`.
   - fake gh `FAKE_GH_DRAFT=1`: `release view` answers `isDraft: true`.
   - fake gh `FAKE_PR_MERGE_STATE=<s>`: `pr view` answers that merge state (default CLEAN).
   - fake gh `FAKE_PR_BRANCH_MOVES=1`: after merging, `pr merge` pushes one more commit to the head branch.
   - `mk_assets` env `MK_ASSETS_COMMENT=<c>`: sets the trusted comment.
   - `mk_assets` env `MK_ASSETS_TAMPER=1`: appends a byte to the archive after the sums are written.
   - `mk_assets` env `MK_ASSETS_BINVER=<v>`: the archived bin prints `<bin> <v>`.

3. **New offline rows 51-68.** Each finding's rows are listed with it. Rows 66 and 67 first rebuild row 42's state. If a fold drops a finding, it drops that finding's rows and recounts.

| # | row | setup → command | expect |
|---|---|---|---|
| 51 | cr_ahead_of_origin | one local commit, subject `release: ms-cli 0.20.3`, not pushed; `--dry-run ms-cli-v0.20.3` | 3; the P12a text with k = 1 (I-2) |
| 52 | cr_toolkit_already_bumped | F-tk with the crate at 0.105.2 (fx_commit); `--dry-run mnemonic-toolkit-v0.105.2` | 3; `cut-release: mnemonic-toolkit is already 0.105.2 in crates/mnemonic-toolkit/Cargo.toml; the toolkit release needs its version bump in commit (b), so release it by hand or restore the previous version` |
| 53 | cr_toolkit_selfpin_lags | F-tk with install.sh naming `mnemonic-toolkit-v0.105.0`; `--dry-run mnemonic-toolkit-v0.105.2` | 3; `cut-release: scripts/install.sh self-pins mnemonic-toolkit-v0.105.0 but Cargo.toml says 0.105.1; land the previous release's pins PR first` |
| 54 | cr_push_pvs_no_ok_line | `FAKE_PVS=silent`; `--push ms-cli-v0.20.3` | 5; `cut-release: push-via-staging.sh exited 0 without its '== OK:' line; not tagging`; no tag on the bare remote |
| 55 | cr_push_pvs_failed_after_landing | `FAKE_PVS=landed-fail`; `--push ms-cli-v0.20.3`; then the same command with `FAKE_PVS=ok` | first run: 5; `cut-release: push-via-staging.sh exited 1; no tag was pushed.` and `cut-release: the release commit IS on origin/master. If the output above shows a bypass, STOP and report it; otherwise re-run the same command: it finds the commit on master, re-stages it and tags.`; no tag. Second run: 0; `== ms-cli is already 0.20.3 (crates/ms-cli/Cargo.toml); no version edit`; the tag peels to the first run's commit; bare master unchanged by the second run |
| 56 | cr_toolkit_pvs_failed_after_landing | F-tk; `FAKE_PVS=landed-fail`; `--push mnemonic-toolkit-v0.105.2`; then run the printed `git tag -a … && git push …` | 5; the two I-1 lines with (b)'s and (c)'s `<12>`; `<git-dir>/cut-release/pins-pr-body.md` exists; no tag. After the printed commands run: the tag peels to (c) |
| 57 | cr_push_tag_push_fails | the bare remote gets a pre-receive hook that rejects `refs/tags/*`; `--push ms-cli-v0.20.3` | 6; `cut-release: pushing the tags failed (atomic; none is on origin):` and `cut-release: re-run the same command; it finds the release commit on master and tags it`; no local tag; bare master = the release commit |
| 58 | cr_push_run_never_appears | `FAKE_GH_WORKFLOWS=fuzz-smoke`; `--push ms-cli-v0.20.3` | 7; `cut-release: no 'release' run appeared for ms-cli-v0.20.3 within 10 minutes`; the finish line |
| 59 | cr_push_interrupted_after_tag | `FAKE_GH_KILL=TERM`; run as `bash -c 'export CUT_RELEASE_TEST_PID=$$; exec <script> --push ms-cli-v0.20.3'` | 7; stderr ends with the interrupted line and the finish line; line 1 of `<git-dir>/cut-release/follow-on.txt` is `== FOLLOW-ON: not done by this script; each is a commit or PR in the named repo`; the tag is on the bare remote |
| 60 | cr_toolkit_run_failed | F-tk; `FAKE_GH_CONCLUSION=failure`; `--push mnemonic-toolkit-v0.105.2` | 7; shows the pins-PR line with (c)'s sha; local master = bare master = (b); no bare `release-pins-0.105.2` |
| 61 | cr_push_release_draft | `FAKE_GH_DRAFT=1`; `--push ms-cli-v0.20.3` | 8; `   FAIL  release is a draft` |
| 62 | cr_push_trusted_comment | ms-cli-v0.20.3 assets with `MK_ASSETS_COMMENT='ms ms-cli-v0.20.2 SHA256SUMS.x86_64'` | 8; `   FAIL  SHA256SUMS.x86_64 trusted comment is 'ms ms-cli-v0.20.2 SHA256SUMS.x86_64', not '... ms-cli-v0.20.3 SHA256SUMS.x86_64'` |
| 63 | cr_push_checksum_mismatch | `MK_ASSETS_TAMPER=1` | 8; `   FAIL  ms-0.20.3-x86_64-linux-musl.tar.gz does not match SHA256SUMS.x86_64` |
| 64 | cr_push_wrong_binary_version | `MK_ASSETS_BINVER=0.20.2` | 8; `   FAIL  ms --version printed 'ms 0.20.2', not 'ms 0.20.3'` |
| 65 | cr_version_skip | `--dry-run ms-cli-v0.20.4` | 3; `cut-release: ms-cli-v0.20.4: ms-cli is 0.20.2 in crates/ms-cli/Cargo.toml; the next version is one of 0.20.3 0.21.0 1.0.0 (a skip is released by hand)` (I-4) |
| 66 | cr_land_pr_not_clean | row 42's state; PR ready, checks pass, `FAKE_PR_MERGE_STATE=BLOCKED`; `--land-pr 77` | 9; `cut-release: PR #77 merge state is BLOCKED, not CLEAN`; no `pr merge` in gh.log |
| 67 | cr_land_pr_branch_moved | as row 46, with `FAKE_PR_BRANCH_MOVES=1` | 0; `== merged PR #77 (<url>) as a merge commit; release-pins-0.105.2 left in place (origin's release-pins-0.105.2 is <12>, not the merged <12>)`; the bare branch still exists (I-5) |
| 68 | cr_follow_on_engrave_demo | `CUT_RELEASE_SIBLINGS=$T/sib`, holding an engrave clone whose two demo files are engrave master's ms lines, copied byte-exact as for F-tk, plus a modified tracked `README.md`; `--dry-run ms-cli-v0.20.3`; then, in `$T/sib/mnemonic-engrave`, run the printed (a) `sed` and `git commit` lines | 0; the commit changes exactly the two demo files, 7 lines; `README.md` is still modified and uncommitted; `grep -c 'ms-cli-v0.20.2'` is 0 in both files (I-6) |

4. **Counts.** The network rows become 69-76.
   - §9 `58 by default, 50 with --offline (rows 1-50)` → `76 by default, 68 with --offline (rows 1-68)`.
   - §10 step 5 `cut-release.test: 58 passed, 0 failed` → `cut-release.test: 76 passed, 0 failed`.

## Minor (one line each)

- M1: P2-P21 carry no mode column. `--verify` needs no checkout, and `--land-pr` needs only P1-P3 and P9. A builder that applies P10-P21 to `--verify` breaks rows 51-55 (fails closed).
- M2: V4's behaviour for a `SHA256SUMS*` with no `.minisig` is undecided (skip, which is what row 53's "4 ok, 2 failed" needs, measured), and so is whether V2's `--` line counts as ok.
- M3: §5.3 step 3 does not say which of the two refusal texts print, or in what order, when PVS exits non-zero and its output has the bypass line.
- M4: The resume path (X = N for every crate) is undecided: does step 7 check `<bin> --version`, and what does the Cargo.lock line print for 0 crates? Also: does `--push` print §5.2's would-lines with `PUSH:`?
- M5: A crate row with no CHANGELOG in a repo that has one (mnemonic-io-lib in engrave) gets `== CHANGELOG: none in mnemonic-engrave`, which is false. C7's handling of a D with no CHANGELOG is only implied.
- M6: §8(d)'s line format omits the `origin/<default>:` prefix that `git grep <rev>` prints (measured). The builder must be told to strip it.
- M7: §8(b) does not say whether P and X are regex-escaped, as (a) does. Run as printed (unescaped) it changes exactly 4 lines for ms, md and mk and 1 for gui today (measured).
- M8: P1 omits `curl` (engrave §5.1 step 8 and toolkit `install-assets.test.sh`), `readelf` (that test's glibc floors) and `timeout` (V6).
- M9: P20 matches `mnemonic-toolkit-v[0-9.]+`, while `install-pin-check.yml` matches `mnemonic-toolkit-v[0-9]+\.[0-9]+\.[0-9]+` (measured). A first mention ending in "." would refuse falsely, so use the workflow's regex.
- M10: Rows 56-58 must name the bare copy `<repo>.git`, or P3 refuses. The spec does not say so.
- M11: §8(a) stops at the commit and push. `check-install-links.sh`'s own FAIL text says "rebuild with demo/sh2/build.sh, and redeploy", so the live page stays stale until `demo/sh2/deploy.sh` runs, which the block does not list.
- M12: (c)'s toml branch "insert after `[installer-ahead]`" is today's real case (measured) but has no offline row. F-tk exercises only the append branch, and row 58 does not check the toml.
- M13: §5.3 step 3's "origin/<default> is unchanged" is false when PVS's master push was rejected because origin moved. The printed reset is still right; say "does not have the release commit".
- M14: §8(b)'s mirrors grep also lists GUI-tier pins that must not move: `manual-gui.yml:227` and `cross-tool-differential.yml:88` for md (measured). Only (d) says some references are deliberate.
- M15: On a re-run after a bypass, PVS re-stages a SHA that is already on master. Its `ci/staging` push is a no-op and it reads the old check runs, so the re-run tags with no new CI evidence. The STOP rests on the operator having read the previous output.
- M16: P7 runs only for `--push`. `--land-pr` performs an irreversible merge with uncommitted tooling unchecked.
- M17: Restore-on-failure by trap is stated only for `--dry-run`. A Ctrl-C during `--push`'s build leaves files modified; the next run's P13 lists them.
- M18: P18's "origin tag" could come from `ls-remote` or from local tags after the fetch; the spec does not say which. With local tags, a local-only tag would count.
- M19: `--land-pr` step 2 runs the release-pins check in any repo. A `release-pins-*` head outside the toolkit is refused with "does not contain mnemonic-toolkit-v<V>" (fails closed, but the message misleads).
- M20: Low-risk rules with no row (each fails closed):
  - usage: U3, U4, U5, U6, U8
  - preflight: P1, P2, P8, P9, P10, P15 local-only, P16
  - CHANGELOG: C2
  - the `-dev` version rule
  - toolkit: the (b) invariant exit 4, the (c) checks exit 4, a missing `mnemonic)` entry
  - §5.1: the step 4 and step 6 failures; step 8 (there is no engrave fixture)
  - §7: not OPEN, wrong base, no checks, checks timeout, merge failure, ancestry failure
  - §8: the (b) block and the 40-line cap

## Nit (one line each)

- N1: §5.3 step 3 says "drop the local commit", but the toolkit has two.
- N2: `--help`'s "6 tag push failed (no tag pushed)" is wrong for the peel-mismatch exit 6, where a tag is on origin.
- N3: P15 local-only says "if you made it by hand", but an interrupted cut-release.sh run leaves local tags too.
- N4: Row 49's "(its refs are refs/tags/<tag>)" cannot be checked by grep when the refs are in a variable.
- N5: Rows 35-39 and 42 omit their command and asset setup; they implicitly reuse row 34's and row 33's.

## Decisions the author flagged (Q5)

| # | decision | grade | reason |
|---|---|---|---|
| 1 | follow-on pins listed, not performed | keep | Performing them adds irreversible writes to other repos' default branches. The (a) recipe is byte-exact on three real bumps (measured), so printing it costs one paste. Fix `-am` (I-6). |
| 2 | toolkit pins PR always a draft owing prose | keep | Measured: `check_cli_pins.py` passes (rc 0, `installer ahead: mnemonic 0.105.2`) on a simulated (c) with only the toml line, while section 82 describes only ms. The draft is the only thing that enforces the owed text. |
| 3 | C7 refuses a CLI alone while its codec has unreleased entries | keep | The path dependency ships the codec's unreleased code inside the CLI binary, with neither its notes nor a codec tag. The refusal names both remedies. |
| 4 | toolkit already bumped refused (P19); `--list` repad | keep | The repad matches install.sh's `%-36s ` (install.sh:472; row 32's line is 37 wide, measured). P19 is what removes the toolkit's resume path, so its consequence is fixed in I-1, not by dropping P19. |
| 5 | no confirm prompt; test is a local gate only | keep, with I-4 | Agents run without a TTY, so a prompt would not protect them. I-4's successor rule is the typo guard the prompt would have been. The local-only test stays (release.yml is out of scope per Brian's ruling); see the design note. |
| 6 | table exclusions; io-lib as a lib row | keep | refugium has no release workflow; wc-codec and mt-codec have no prefix. io-lib has a prefix and no tag workflows, which matches its "none" cells. It is consumed from crates.io by md, mk and the toolkit (Cargo.toml lines, measured), and publishing it is out of scope. |

## Design concerns (ungraded)

- P7 checks only for uncommitted changes, so a committed but unmerged `cut-release.sh` on an engrave branch would run. Consider requiring the engrave checkout's HEAD to be an ancestor of its origin/master for `--push` and `--land-pr`.
- Every ms, md and toolkit release turns engrave's required `test (rust + go)` red until the §8(a) commit lands, because `check-install-links.sh` demands the newest release. The script prints the move but cannot gate on it.
- The offline rows run only locally (decision 5), so a later edit to `cut-release.sh` can regress unseen until someone runs the harness.

## Run what you print: commands executed (Q4)

| spec text | command (read-only) | output |
|---|---|---|
| §3 tag workflows | `on.push.tags` of every workflow on each default branch | Every waited name matches its row: engrave `release`; toolkit `changelog-check`, `install-pin-check`, `man-pages`, `release`; ms `man-release`, `release`; md `man-pages`, `release`; mk `CI`, `musl-binaries`, `release` (CI also on `mk-codec-v*`); mt `release`; gui `build`, `schema-mirror`. |
| §3 assets and §5.4 V1-V6 | `bin/vcheck.sh`: `gh release view/list`, `gh release download`, `minisign -Vm <s> -x <s>.minisig -P <key>`, awk + `sha256sum -c`, tar, `timeout 30 <bin> --version`, on the newest release of all 7 bin prefixes | **ms-cli-v0.20.2:** 14 assets; aarch64, portable and x86_64 all print `Signature and comment signature verified / Trusted comment: ms ms-cli-v0.20.2 SHA256SUMS.<x>`; the archive is in SHA256SUMS.x86_64 only, `OK`; `ms 0.20.2`. That is 7 ok (row 51). **mt-cli-v0.1.1:** `Trusted comment: mt mt-cli-v0.1.1 SHA256SUMS.portable`; `mt 0.1.1`; 5 ok (row 52). **descriptor-mnemonic-md-cli-v0.20.3:** no SHA256SUMS.aarch64.minisig and no SHA256SUMS.portable.minisig; `md 0.20.3`; 4 ok, 2 failed if V4 skips unsigned files (row 53). **v0.12.0:** `Signature key id in SHA256SUMS.minisig is CA39ECB257009A0F / but the key id in the public key is EF2B8D34D8409754`; `me 0.12.0` (row 54). **mnemonic-toolkit-v0.105.1 and mk-cli-v0.13.0:** no `.minisig` for aarch64, portable or x86_64. **mnemonic-gui-v0.63.0:** no SHA256SUMS.minisig. Every §3 archive is in exactly one sums file and checks OK, has one file named `<bin>`, and prints `<bin> <N>` (gui ran headless). V2 finds no missing names for any of them. Bare `--verify` today: 5 of 7 fail, so row 55 exits 8. |
| §5.4 V4 trusted comment | the `minisign -S ... -t` lines in every repo's release workflows | All end `<tag> <sums>`: engrave `mnemonic-engrave ${TAG} SHA256SUMS`; toolkit `mnemonic ${TAG} SHA256SUMS.portable` / `mnemonic ${REF_NAME} $SUMS`; ms `ms …`; md `md …`; mk `mk …`; mt `mt …`; gui `mnemonic-gui ${REF_NAME} SHA256SUMS`. |
| §5.3 step 5 | `gh run list --repo <slug> --branch <tag> --event push --limit 50 --json databaseId,name,workflowName,displayTitle,headSha,status,conclusion,url` for 9 tags | `name` equals the workflow name (displayTitle is the commit subject), and headSha is the peeled tag. ms-cli-v0.20.2: release, man-release, fuzz-smoke. toolkit-v0.105.1: the 4 waited plus fuzz-smoke, bitcoind-differential, miniscript-fork-tripwire, sibling-pin-check. **md-cli-v0.20.3: `man-pages completed/failure`.** ms-codec-v0.10.1 and mk-codec-v0.5.0 have separate per-tag runs. |
| §5.3 step 5 | `gh run view 41354946629 --repo bg002h/mnemonic-secret --json status,conclusion,url` | `{"conclusion":"success","status":"completed","url":".../runs/41354946629"}` |
| §5.3 step 4, P15 | `git ls-remote origin 'refs/tags/ms-cli-v0.20.2^{}'`; `git ls-remote --tags origin refs/tags/<absent>` | `62b4d9dc903566bc648dbf347bf571f9451ae817 refs/tags/ms-cli-v0.20.2^{}`; an absent tag gives empty output with rc 0 |
| §7 | `gh pr view 123 --repo bg002h/mnemonic-toolkit --json state,isDraft,baseRefName,headRefName,headRefOid,mergeStateStatus,url,isCrossRepository`; `gh pr checks 123 --json name,bucket,link` | All fields are present: `MERGED`, `isCrossRepository:false`. The checks call exits 0 with `pass=30 skipping=2`. |
| branch protection | `gh api repos/<slug>/branches/<default>/protection`, `repos/<slug>`, `.../rulesets` | On all 6 protected branches, `strict:false` and `enforce_admins:false`. mt's `main` is unprotected. Required contexts: engrave 2 (`test (rust + go)`, `picotool argv probe`), toolkit 9, ms 6, md 2, mk 1, gui 5. All 7 repos allow merge, squash and rebase. No rulesets exist. Engrave's `release.yml` runs on master and `ci/**` pushes with no path filter, and the toolkit's 9 contexts likewise have no push path filter (PVS route OK). |
| P14 | `rust-toolchain.toml`, `cargo --version` | `channel = "1.85.0"` in ms, md, mk, toolkit and mt; engrave and gui have none. `/usr/bin/cargo` is 1.98.1 and rustup's cargo is 1.85.0. |
| P8 | `sed -n 2p minisign.pub \| grep -cE '^RW[A-Za-z0-9+/]{54}$'` | 1 |
| P16, P21, §5.1 step 2 | `cargo metadata --no-deps --format-version 1 --offline` | ms-cli→ms-codec `=0.10.1`; md-cli→md-codec `=0.48.4`; xtask→md-codec `*`; mk-cli (0.14.0-dev)→mk-codec `^0.5.0`; mt-cli→mt-codec `^0.1.0`; mnemonic-engrave→mnemonic-io-lib `*`; toolkit→wc-codec `*`. Every pin is a one-line `{ path, version }`. With ms-cli's pin set to `=0.10.0`, metadata still succeeds (req `=0.10.0`), so row 22 reaches P21. |
| §5.1 steps 1, 2, 4, 5 | version edit (+ pin), `cargo update --workspace --offline`, `git diff --numstat Cargo.lock`, in scratch clones | md both crates + pin: 2/2; mk 0.14.0-dev→0.14.0: 1/1; engrave both crates: 2/2; gui: 1/1; mt: 1/1. Only `version =` lines change. |
| §6.1 | grep of the 0.105.1 forms | gen.sh lines 3, 50, 115, 132, 721; Examples.md lines 3, 20, 107, 121, 122, 125, 166, 1336 (as in F-tk); the `--list` GIT_TAG column is 37 wide; both README markers appear exactly once |
| §6.2, P20 | first `mnemonic-toolkit-v…` in install.sh; the `first_signed` line; `check_cli_pins.py` on master and on a simulated (c) | `52:mnemonic-toolkit-v0.105.1`; line 189 `        mnemonic)     echo "" ;;` matches. `check_cli_pins.py` prints `OK: cli-pin-consistency ... installer ahead: ms 0.20.2` on master and adds `installer ahead: mnemonic 0.105.2` on (c), rc 0 both times. |
| §6.3 step 4 | `sh scripts/install-assets.test.sh` (toolkit master) | `ok   32 of 32 mapped assets verified` / `[install-assets.test] OK` |
| §5.1 step 8 | `demo/sh2/check-install-links.sh` (engrave master, uses curl) | `OK: every advertised URL resolves and names the newest release of its kind.` |
| §8(a) | the recipe on the pre-bump files of 76b7a2d2, 4a0afb3f and e5c0a52f, and on today's files for ms 0.20.3 | byte-identical to all three real commits; today 4+3 lines change and 0 stale lines remain |
| §8(b) | `sed -i -E "/<P>\|'<bin> <X>'/ s#<X>#<N>#g"` on a copy of Examples.md | ms, md and mk: 4 lines each (the `--list` line plus the 3 installer lines); gui: 1 line |
| §8(b) mirrors | `git grep -n -F -- '--tag ms-cli-v0.20.2' .github/workflows docs/manual/src docs/quickstart/src` | `manual.yml:168`, `quickstart.yml:150`, `technical-manual.yml:181` |
| §8(d) | `git grep -n -w -F 'ms-cli-v0.20.2' origin/<default> -- <the printed pathspecs>` on 9 siblings | engrave 4 (demo), toolkit 9 (Examples.md, manual.yml, …), the other 7 0; each line is prefixed `origin/<default>:` |
| §2 | `dirname "$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")"` in the spec worktree; sha256 of each PVS copy | `/scratch/code/shibboleth`; `c12a644a` in engrave, toolkit, ms and md; no copy in mk, mt or gui |
| §5.1 EXIT trap | `timeout -s INT 1` on a `set -euo pipefail` script with an EXIT trap | the trap ran, and `$?` inside it was 0 |

Not run because each would write: cut-release.sh itself (it does not exist yet), push-via-staging.sh, any tag or branch push, `gh pr create` (its `--dry-run` "may still push git changes"), and `gh pr merge`.
