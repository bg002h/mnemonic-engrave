# Final independent review: refugium-wallet exit-code implementation (2026-11-26)

Worktree `/home/user/fx/refugium-wallet`, branch `fx/refugium-wallet`, diff `ca70dd56..d0b86673`
(7c0fe9a4, 3e368726, c22a8c69, 4ed91ff6, d0b86673). Spec: `design/SPEC_constellation_exit_codes.md`
rev 6. mr-codec read at 9131377b in `/home/user/fx/mnemonic-refugium`. No source was edited.

**Verdict: 0 Critical, 1 Important, 2 Minor, 5 Nit.** The exit-code work itself is sound and
every Opus finding that was folded is fixed. The one Important is about what else the pin bump
carries, not about the exit codes.

## Critical

None.

## Important

### I1. The pin bump carries 158 mr master commits besides the exit codes, and triggers an open funds follow-up

**Where:** `Cargo.toml:43-47`, `Cargo.lock:2982-2984`; `crates/refugium-view/src/create/late.rs:517-544`
(`closing_rows`); `design/FOLLOWUPS.md:8908` (`rw-mrgui-card-check-row`, status=open, grade=I, kind=funds).

**What.** The old pin, 2c67cebf, is the merge of mr #218 (2026-11-21). mr master was at 25051a06
when the exit-code branch started. So `2c67cebf..9131377b` is 174 commits: 158 on master (mr #219
to #243, errata E93 to E104, the 0.1.0-alpha.2 release, md-codec 0.48.4 to 0.48.5) and 16 for E105.
`git diff --stat 2c67cebf..25051a06 -- crates/mr-codec/src` is 22 files, +1091/-209, none of it
exit codes. The exit-code spec's §7k assumed the wallet's pin was 25051a06 "today"; it was not.
Neither the implementer's report, the Cargo.toml comment ("Before it, 2c67cebf (master, the merge
of #218)"), the commit messages nor the Opus review mention this. The Opus review compared only
the four `wizard_texts` source regions.

**The consequence that matters.** E101 (mr #228, seat-swap verify) adds a line to every create
run's closing checklist (`guard::card_check_line`, pushed in `unengraved_lines_chain`,
`guard.rs:3346`; 16 mr golden files hold it as `  - \`mr\` cannot tell which seat a holder
belongs at: …`). The wallet's `closing_rows` turns every `  - ` line into a step 7 Part 2 row, and
step 7 is done only when every row is ticked (`late.rs:476`, `:1883`). So from this pin on,
mr-gui shows a new required tick whose text says:

- on a re-mint, run `mr verify MR1_CARD --network <net> --digest KIT_DIGEST`; and
- otherwise "read the role table this run prints against your own plan".

mr-gui offers no way to run the first, and step 5 shows mr's text only from the first `kit ` line
on, so the role table is not beside the row. That is exactly FOLLOWUPS `rw-mrgui-card-check-row`
("once rw pins mr-codec at the seat-swap-verify build; mr-gui shows mr's role table … beside that
row … re-bless the closing-row tests"). The follow-up is filed grade I, kind funds: the row is the
only guard against a seat swap, and a holder who cannot do what it says will tick it anyway.

**Why no test failed.** `create_6_9.rs:495-502` builds its expected rows from the same mr stdout,
so a new row passes silently. I read this; I did not render the screen.

**Other in-range behaviour I checked and found compatible:**
- `api::record_plates` gains refusals (a record whose fingerprint no seat at its path declares, a
  misbound record; mr #231). The wallet takes only `f.message` (`create/late.rs:27-36`,
  `finish.rs:58-62`) and shows a refusal. New refusals can now appear at step 6 and at Create's
  finish in Restore; that is by design but is new behaviour on this branch.
- `verify`'s "passed, but …" closing line was reworded after the prefix the wallet matches
  (`VERIFY_PASSED_BUT`); the prefix is unchanged.
- E102 (`spending.rs`, a time lock equal to `--time` is not reached), E96 to E100 (heir tests),
  E103 (`lost.rs`): mr text only; the wallet does not use `lost` and `eval.rs`, `tip.rs`, `hot.rs`,
  `export.rs`, `seatcheck.rs`, `walk.rs`, `par.rs` are unchanged.
- `sitting/sealed.rs` was reorganised (+`blob.rs`); `public_data_hash` and `format_hash` keep their
  signatures; `plan`, `handback`, `closing` are unchanged.
- Every mr text `create/refusal.rs` and `PATH_LINE_PREFIX` match on has the same occurrence count
  at both revisions.
- One copy each of mr-codec, md-codec, mk-codec in `Cargo.lock` (`scripts/codec-check.sh`'s rule).
- Not checked: what md-codec changed between 6cf6599a and c8bb97a5 (that revision range is not in
  the descriptor-mnemonic checkout here).

**Fix (one of):**
- (a) do `rw-mrgui-card-check-row` in this PR;
- (b) move the pin to 25051a06 in its own reviewed PR first (with that follow-up), then this one;
- (c) get Brian's ruling that the row may ship as mr prints it for now, and record it.

In every case: say in the Cargo.toml comment and the PR text that the pin crosses mr #219 to #243
(E93 to E104), and update the follow-up from "once rw pins" to "pinned at <rev>".

## Minor

### M1. Owed items are recorded only outside the repo

**Where:** `design/FOLLOWUPS.md` is not in the diff.

Owed after this branch, with no tracker line: the §7k erratum (only in UI spec [EXIT-1], "owed to
the spec thread"); Opus M3's merge gate and re-pin to mr master; the GUI manual lines (`mr-gui`
64/32/101, `--help`, the panel's states; toolkit `docs/manual-gui` lockstep); Brian's ratification
of the new internal text ([EXIT-1], a D24-class amendment); mr's `mr-exit-codes-refugium-wallet`
to close against this commit. They live in `/home/user/fx/impl-rw.md`, which is not in any repo.
**Fix:** one FOLLOWUPS line each.

### M2. The W2 spec still cites `mr-sitting`'s old usage code

**Where:** `design/SPEC_w2_mr_gui_sitting.md:402` "clap refuses them, exit 2".

At 9131377b `mr-sitting`'s usage is 64 (`crates/mr-sitting/src/main.rs:358`). [EXIT-1] covers the
UI spec's "code 1"/"code 2" of an `Outcome`, not the W2 spec. Nothing in the code depends on the
number (the wallet never passes those flags to `verify`). **Fix:** "exit 64", or a pointer to
[EXIT-1] in the W2 spec.

## Nit

- **N1.** `crates/refugium-view/src/restore/mod.rs:806-807`: the comment still says "fails closed,
  as a code other than 0 and 1". It is the usage state (64) now. M5's sweep missed it.
- **N2.** `crates/mr-gui/src/args.rs:88-90`: `--help`, `--version` and `--check-libs` return at
  once, so `mr-gui --help --bogus` and `mr-gui --version extra` exit 0 (measured), while
  `mr-gui --bogus --help` exits 64. R2 puts the usage check first. This predates the diff.
- **N3.** `launcher.rs:198-203`: a child that panics before its first frame is `StartFailure`, 32,
  though a panic is a bug; a child killed by a signal after its first frame is `Crash`, 101, with
  the crash text. Both fit R12's list, and the message text already says which; no change needed
  unless the spec thread wants a signal-killed child told apart.
- **N4.** Commit 7c0fe9a4 does not build alone (Opus N5), and this repo merges with merge
  commits, so it would reach `main`'s history. Squash 7c0fe9a4, 3e368726 and c22a8c69 before the
  PR is offered.
- **N5.** UI spec §8.3 near line 4803 still quotes "exit 1" for the sibling-question and path
  runs "at mr 922027fe". The cite is dated, so it is true as written; [EXIT-1] should name it or
  the runs should be re-measured at the pin (the implementer's question 7).

## 1. The Opus findings and the fold (d0b86673)

| Finding | State | How I know |
| --- | --- | --- |
| I1 status decoded is the terminal's | Fixed, option (a) | `git diff ca70dd56..HEAD -- sitting_spawn.rs` is 6 lines: the `killed by signal` wording and its doc. No probe, no `wait_within`, no `scheme_of` anywhere in `crates/` (grep). `exit_codes_early_status_stays_bare` passes (run). |
| M1 `amber()` unused | Fixed | `exit_status.rs` is gone, `lib.rs` drops the module, no reference remains (grep). `sixteen_alone_fails_closed` and `exit_codes_pinned_to_the_spec` pass (run). |
| M2 INVALID half untested | Fixed | `step3_path_line_only_under_invalid` (1, 8, 64 give `Other`; 2, 18 give `Paths`) and `w2_export_refusals` leg (2b). The first passes (run); the second I read (the test fails later at its read-only-folder leg, an environment failure). |
| M4 stdout failures | Fixed | Measured on the built binary: `--help >&-` 32, `>&- 2>&-` 32, `>/dev/full` 32, read-only stdout 32, reader gone (EPIPE) 0 for `--help` and `--version`, `<&-` 0, headless `--check-libs` 32 with stdout open or closed. `stdout_failure_adds_32` passes. |
| M5, N1 | Fixed, but one comment missed (my N1) | grep of `crates/*/src`. `take_result` uses `shown_code()`. |
| N2 | Comment only | `mr_panel.rs:254-255`. Right call: `STOPPED` is used nowhere in mr-codec outside `exit_codes.rs`, so no `api` call can return 130. |
| N3 | Fixed | `sitting_spawn.rs:111`, W2 spec `:261`. |

**New defects from the fold: none found.**
- The `.init_array` probe (`diagnostics.rs:73-93`, `main.rs:16-19`): fds are visited in order
  0, 1, 2, so each `/dev/null` open lands on the fd just found closed; `fcntl`, `open`, `close`
  and errno are safe before std's runtime starts; a failed open leaves the fd for std to plug as
  before. It runs in the UI and message children too, where it only does what std did. `libc` is
  a Linux-only dependency and every use is `cfg(target_os = "linux")`.
- `unwritable()` is consulted only by `to_stdout`, whose only callers are the four tool modes in
  `args.rs` and the Windows read-back. The child's record pipe (`protocol::send`) does not go
  through it, so a GUI started with stdout closed starts as before.
- The EPIPE exception matches R8 ("keeps the verb's own code").
- `exit_codes::add(code, IO)` keeps 64 and 101 as they are; `exit_after` is only called with 0
  or 32.
- All 20 `launcher::` integration tests pass, one per process, including
  `a_closed_stderr_does_not_stop_the_message`.

## 2. The Opus findings that were not folded

- **M3 (pin on an unmerged branch).** Rightly not changed in code: there is nothing to re-pin to
  until the mr branch merges. But it should have been folded as a FOLLOWUPS line (my M1). And see
  I1: the same pin move needs more than a re-pin.
- **Nit 2 (130 shown as internal).** The comment is enough; unreachable.
- **Nit 4 (`Scheme::of_help` token boundary).** Moot: the module was removed.
- **Nit 5 (non-building commit).** Should be squashed before the PR (my N4). Not a code fold.

## 3. Checked and sound

**Every place the wallet reads a code or class.**
- `Outcome::code()` / `Called::code`: `mr_panel.rs:274`, `create/run.rs:274,281,303`,
  `create/mod.rs:4031,4325`, `create/sitting.rs:1181,1321,1651`, `create/replay.rs:182`,
  `create/first_address.rs:205-238`, `balance/receive.rs:350-360`, `send/build.rs:279-284`,
  `action/job.rs:183-190,280`, `restore/check.rs:127,187`, `restore/mod.rs:201,797-812`. Each keys
  on 0 against not 0, on the INVALID bit through `Bits`, or goes through `CallState::of_code`.
- `CallState::of_code`: 0 Success; 1 to 63 Refused; 64 Usage; 101, 130, 65 to 125 and 128 up
  Internal. Every consumer of `Internal` fails closed (Restore's gate off with no reason,
  `lead_of` `None`, Balance and Send `Refused { shown }`).
- 16 or 4 alone, or 20: Refused, never success. Unreachable: only `repair.rs` sets them and the
  wallet has no call to it.
- `record_plates` and the root Sitting screen take the failure's message only.
- The sitting: `sitting_spawn.rs` reads the terminal's status as success or not, and prints the
  bare number or `killed by signal N`. The sitting's results come from the hand-back files, not
  from a status. Nothing else spawns `mr`, `mr-sitting` or any constellation binary.
- mr-sitting's ambiguity 8 and signal fallback 32, and parentage verdicts 1 (were 2 at 8cc88e4d):
  nothing keys on them; verify is read as 0 or not 0.

**mr-gui's own exits.** 0; 64 (bad argument, measured); 32 (no window system, `--check-libs`
failing, a failed screenshot, every renderer failing before its first frame, an unwritable
stdout); 101 (a crash after the first frame, or a panic in the launcher). No
`ExitCode::FAILURE`, `process::exit` or `abort` remains in `crates/*/src` (grep).

**The legend.** `mr_codec::app::exit_codes::LEGEND` equals the spec's R11 text, whitespace
normalised (compared by script against the spec file). `mr-gui --help` and `-h` print the usage,
a blank line, then the legend as the last text, on stdout, exit 0, stderr empty (measured and
tested).

**Wallet-safety invariants.** No change to result invalidation (`results()` and its epoch),
fixtures, amounts, descriptor membership or PSBT saving. Every address cross-check still needs
code 0 and an equal address before `Ready` or `Built`; a disagreement is still `Disagree`.

**Spec text against code.** UI spec §4.1.1's four states, the internal text, "Continue stays
disabled with no reason", §8.3's INVALID-bit wording and [EXIT-1]'s statements (no 16, bare
sitting status) all match the code. The usage text's "Refugium" against the code's "Refugium
Wallet" predates the diff. W2 spec §W2-6 matches `describe`.

## Measured and read

**Measured** (cached binaries from 21:49 to 21:53, built from the fold's tree; no source file is
newer than them and the tree is clean, so nothing was rebuilt):
- `target/debug/mr-gui` run directly for the exit codes listed under M4 and N2.
- refugium-view test binaries: `mr_panel` 21/21; `create_round` `step3_path*` 7/7 and
  `mint_refusal*` 2/2; `receive` 12/12; `create_6_9` `closing*` 3/3.
- mr-gui `gui_tests`, one test per process: the three `exit_codes::` tests,
  `w2_sitting_behaviour::exit_codes_early_status_stays_bare`, and all 20 `launcher::` tests pass.
  `version::audit_a_version_line_matches_this_checkout` fails only because the cached binary is
  stamped `4ed91ff6-dirty`; a rebuild at HEAD would stamp d0b86673. Not a finding.
- The legend comparison; `git` range counts and diffs in the mr checkout; text occurrence counts
  at 2c67cebf and 9131377b.

**Read only:** everything else, including the step 7 row in I1 (derived from mr's source and
goldens and the wallet's `closing_rows`, not rendered), `launcher_windows.rs`, `xvfb.rs`, the
regtest rows, the full suites (the implementer's logs were not re-run), and clippy/fmt.
