# Final review: constellation exit codes in mnemonic-refugium (mr-codec, `mr`, `mr-sitting`)

Reviewer: Fable 5.2, 2026-11-26. Independent round after the folded Opus review
(`design/agent-reports/exit-codes-impl-review-2026-11-26.md`, 0C/4I/7M/3N).

- Worktree: `/home/user/fx/mnemonic-refugium`, head `8cc88e4d5`, diff `5f60b0a61^..HEAD`.
- Spec: `/home/user/fx/mnemonic-engrave/design/SPEC_constellation_exit_codes.md` rev 6.
- No source was edited.

**Verdict: 0 Critical, 2 Important, 7 Minor, 9 Nit.** The repair shapes, the
signal paths and the ruling-4 moves of the seven error variants are sound as
measured. The two Importants are one funds-relevant classification (a verify
verdict reported as "fix your input") and three TRACE-row tests that still
assert the old exit code and cannot pass anywhere.

**One thing a person must clean up, caused by me:** my probe
`mr vectors --out /nonexistent/dir/x` succeeded and created `/nonexistent/dir/x`
(442 vector files). Removing it was blocked by the harness safety check and I
did not work around it. Please remove `/nonexistent` by hand.

## What I ran

- Built `mr` and `mr-sitting` (dev profile, debug info off, no `--release`).
- `mr repair` on all 116 mr1 string sets in the corpus, on main and testnet,
  plus crafted variants made with an independent Python checksum/header tool
  (`/tmp/claude-0/-home-user/b63c8a21-4ccb-540d-9c73-8450d18dd03a/scratchpad/mrbch.py`).
- `mr-sitting` driven on a pty with signals at the mainnet sentence prompt, at
  the first prompt and mid-session
  (`/tmp/claude-0/-home-user/b63c8a21-4ccb-540d-9c73-8450d18dd03a/scratchpad/drive.py`).
- Targeted tests only: codec 54 passed (`e105_`, `w14s1_hdr_cards`, `m8_chunk`,
  `m2_*`); cli 254 passed (`e105_`, `m11_units_repair`, `jf_items`,
  `m11_fold_stdout`, `pr45_build`); sitting integration 10 passed; sitting bin
  2 `e105` unit tests passed. No whole suite was run.
- `python3 scripts/trace-quote-check.py`: 0 new, 0 stale.
  `python3 scripts/trace-runs-check.py`: 0 new, 0 stale.
- `scripts/api-member-policy-spec-gate.sh` (hand-run gate): 61 of 66 pass.

Not run: full nextest, the exact-name phase-close run, regtest rows,
vendor-freshness, clippy, doc.

## Critical

None.

## Important

### I1. `mr verify` parentage verdicts exit 2 (INVALID) where they are verify's answer about the card (lens b)

`crates/mr-codec/src/app/verify/parentage.rs`:

- `:508` "the record's keys are seed L's own, not its child's ... mint again and engrave again"
- `:543` the `refused` closure, `Failure::invalid(mint_refused(..))` ("with seed L, this mint is refused ... mint again before any deposit")
- `:587`, `:601` ("at or below seed's floor ... before any deposit, mint again"), `:611`, `:623`

All six are `Failure::invalid`. Their neighbor `:536` ("does not match this
card and its record") is `Failure::no`, exit 1.

Scenario: the operator runs the parentage form of `mr verify` with a seed, a
record and the card. The card and record are well formed; verify's finding is
that this mint must not be funded. The exit is 2, which the legend and spec
ruling 4 define as "your input disagrees with itself, fix it and run again". A
script or GUI that separates 1 (the answer is no) from 2 (bad input) files a
do-not-deposit verdict under input errors. Ruling 4 puts a comparison against a
separate operator-given reference at 1, and seed, record and history are
exactly that here. Read from source; the code is pinned by
`crates/mr-cli/tests/pr45_build.rs:1193` (through `refused(&v, ..)`), which
passes at 2.

Fix: `Failure::no` at the verdict sites; keep 2 only where the inputs are
malformed or insufficient to judge. Move the `pr45_build` assertion and the
E105 table with it. If the spec thread holds that a refused mint is
self-disagreeing input, write that into ruling 4 as a named case, since `:536`
and `:543` then need a stated reason to differ.

### I2. Three TRACE-row tests still assert exit 1 for a typed TUI quit, and have never reached that assertion (lens e)

`crates/mr-sitting/tests/w1t_pty.rs:62`, `:143`, `:211`:
`assert_eq!(code, 1, ...)` in `ms_w1t_quit_before_handback`,
`ms_w1t_quit_after_handback`, `ms_w1t_quit_after_replaced`. A typed stop is now
130; `crates/mr-sitting/tests/tui.rs:159` asserts 130 for the same stop and
passes.

Measured: on this box all three fail earlier, on a timeout waiting for
ST-Q-01 / ST-Q-02 / ST-BK-04 at `crates/mr-sitting/tests/common/mod.rs:1177`,
and they fail the same way in the baseline log. The implementer report lists
them as environment failures. So the stale assertion was never executed, and
where ptys work the tests fail on the number instead. They are TRACE rows, so
the phase-close exact-name run is a FAIL either way.

Fix: assert 130, update the three TRACE rows, and run them by exact name on a
machine where the pty walk completes before the phase closes.

## Minor

### M1. A trailing unknown flag after a card argument exits 8 with a usage message

`crates/mr-cli/src/lib.rs:108` (`clap_exit_code` returns REFUSED for the
secret-on-argv swap before looking at the error kind; the flag is set in
`crates/mr-cli/src/common.rs:278`, `:363`, `:403`).

Measured: `mr decode garbage --bogus` and `mr repair garbage --bogus` exit 8
and print "error: unexpected argument at position 3 (not echoed)".
`mr decode --bogus garbage` exits 64. The code depends on argument order and
disagrees with its own message; spec R2 says usage first.

Fix: return REFUSED only when the reported error is the card parser's own
refusal; every other clap error kind maps to 64.

### M2. R2 ordering is wider than the "one exception" E105 records

Measured: `mr create 'garbage(' --network signet --height 1 --time 1 --no-other-cards-on-these-paths`
exits 2, while the same line with a valid template exits 64 ("create needs a
--role"). Usage is judged after the input is read at the in-app
`Failure::usage` sites: `crates/mr-codec/src/app/create.rs:1962`, `:1967` and
neighbors, `crates/mr-codec/src/app/verify.rs:399`. The Opus review's M5 was
answered by documenting one exception (AT33/AT34); the behavior is general.

Fix: either hoist the flag-presence checks ahead of input parsing, or restate
E105 as the general rule it is ("flags clap cannot judge are checked after the
inputs parse") and tell the spec thread R2 has that carve-out for `mr`.

### M3. mr-sitting plate ambiguity is 2; ruling 7 says 8

`crates/mr-sitting/src/verify.rs:292-299`: "this plate could be {list}: run
again with one of {sels}", code 2. The tool is declining a choice a flag would
make, which ruling 7 puts at REFUSED. Read from source.

Fix: 8, with a test row.

### M4. `MR_EXIT_RECORD` is left in the test helpers

`crates/mr-cli/tests/m11_common/mod.rs:89-110` (`exit_recorded`), used at
`crates/mr-cli/tests/fj1_common/mod.rs:933`, `w14s1_cli.rs:349`,
`w14s2s2_order.rs:217`. With the variable set, about 1,700 exit assertions
record instead of assert. The bulk re-pin was made by recording what the binary
did (1 to 2: 1239; 1 to 8: 236; 2 to 64: 172; 2 to 8: 44; 1 to 64: 34;
1 to 32: 22), so those rows prove stability from here on, not that each new
number is the right class. I1 is one that went through that way.

Fix: remove the switch (or make it panic at the end of the run), and have a
person sample the 1 to 2 group by message, since that is the group where a
verdict can hide as an input error.

### M5. Hand-run gate scripts and the API spec still carry the old numbers

- `scripts/api-member-policy-spec-gate.sh`, measured: 61 of 66 pass; five rows
  fail only on the exit code (`cli_deadline_180` 8 vs 1, `cli_records` 2 vs 1,
  `cli_untied_plates` 2 vs 1, `cli_compromised` 8 vs 1,
  `cli_kits_updated_none` 8 vs 1).
- `scripts/creator-warn-fix-gate.sh` stage c: `:230`, `:240` expect 2 (now 64),
  `:259` expects 1 (now 2). Read, not run.
- `scripts/api-first-epoch-spec-gate.sh`: of 57 quotes, 28 hold at base and 24
  at HEAD; 4 were broken by this diff.
- `design/SPEC_mr_codec_api.md` has about 69 exit lines with old values.

None of these runs in CI (`.github/workflows` has only `ci.yml` and
`trace.yml`), and FOLLOWUP `mr-exit-codes-spec-text` does not name the gates.

Fix: name the three scripts and the API spec in that FOLLOWUP, or move them in
this PR.

### M6. Repair API as a consumer sees it (lens d)

`crates/mr-codec/src/repair.rs`:

- `Repair::prints(decodes)` and `exit_code_with(decodes)` take the decode
  result as a caller-supplied bool. A consumer that passes the wrong value gets
  a confident wrong code; `exit_code(network)` is the safe form and the doc
  should say to prefer it.
- `repair_card` returns `Ok(Repair { strings, .. })` for results `mr`
  withholds. A consumer that shows `.strings` without asking `prints` displays
  what the CLI refuses to print.
- Public `repair::repair` with two or more strings takes its `Form` from the
  first header with no set check, so `exit_code` reports 0 or 16 for strings
  that are not one set; two non-chunk strings come back as `Form::Single` and
  22. `repair_card` is the checked entry; `repair` should be `pub(crate)` or
  documented as unchecked.
- `Outcome::code()` kept its signature, so a consumer's numeric comparisons
  compile unchanged and silently mean something else. refugium-wallet already
  pins mr-codec at `8cc88e4d` (`/home/user/fx/refugium-wallet/Cargo.toml:47`);
  its `is_path_refusal`
  (`/home/user/fx/refugium-wallet/crates/refugium-view/src/create/run.rs:273`)
  keys on the INVALID bit, with no typed discriminator. Any fold here moves
  that pin again, and if I1 is folded that helper should be rechecked.

### M7. Signal fallback can re-raise after a failed wipe

`crates/mr-sitting/src/wipe.rs:936-957`: the watcher wipes only
`if let Some(d) = dir`. If the session has already taken the RAM folder
(`crates/mr-sitting/src/session.rs:2157-2184`, `wipe_ram` takes `self.ram`) and
its own wipe fails after the grace period, the watcher has nothing to wipe,
keeps a non-32 code and raises the signal. R10 says a failed wipe is 32. Read
from source, not measured; it needs a wipe failure and a signal in the same
window.

Fix: have the session's wipe result reach the watcher (a shared flag set on
failure), and exit 32 when it is set.

## Nit

1. E105, the TRACE header and several test docs cite spec "rev 5" and say
   §7h's mainnet 8 is "for the spec thread to amend". Rev 6 already has 130 for
   the refused sentence and 32 for the held lock.
2. E105 typo: `mr1qpqq...` for `mr1qqpq...`.
3. Stale comments: `crates/mr-cli/tests/jf_items.rs:2083` ("still exits 5"),
   `crates/mr-codec/tests/m2_string.rs:695`.
4. About 39 rustdoc lines keep old numbers, for example
   `crates/mr-codec/src/error.rs:453` (G5 "exit 1, overridable"), `:468`
   (G11 "usage, exit 2"), `crates/mr-codec/src/app/common.rs:707`, `:739`,
   `:828`, `lost_or_stolen.rs:541`.
5. `crates/mr-codec/src/repair.rs:9-19` and `:289-293` (module and
   `repair_chunks` docs) still say a missing index fails the whole call and
   that repair does not decode the card.
6. `crates/mr-cli/src/repair.rs` `card_check` decodes twice (once for its
   lines, once through `rep.decodes`).
7. `crates/mr-cli/src/current.rs:291-296`: "no slot asked can be answered at
   this tip" is 8, with no flag that waives it. It reads more like 1.
8. Passphrase typed twice and not matching is 1 at three sites
   (`parentage.rs:127`, `hotrun.rs:498`, `crates/mr-cli/src/common.rs:1004`).
   The letter of ruling 4 says 2; the toolkit's `PassphraseMismatch` is also
   NO, so this is consistent across the constellation and is a question for
   the spec, not a defect here.
9. `crates/mr-sitting/src/wipe.rs:621` `on_signal` does not save and restore
   errno. Pre-existing.

The 10 (8+2) sums in `m12_fold_e45.rs` are correct under R1.

## Leftover from the Opus review, or still owed

- **Its M5** (usage ordering) was documented, not fixed, and understated; see M2.
- **Its M2** is partly done: untouched sitting TRACE rows keep old numbers
  behind a header note.
- **Its N2** (names) was kept as is.
- **Its I4** process gates are still open:
  - `ci/toolkit-manual.rev` is unchanged (`2634d7a8...`) although printed text
    changed; CLAUDE.md requires the bump before the PR is offered.
  - The phase-close exact-name nextest run with counts quoted against TRACE
    has not been done. All 17 E105 rows do have a same-named test.
  - The regtest `#[ignore]` rows have not been run.
  - No `agent-reports/whole-diff-mr-<phase>.md` exists for this phase.
  - The mr-sitting bin unit suite has never had a full clean run (the
    implementer's run was aborted).
  - Per-verb `--help` exit line (R11) is owed for `mr` and `mr-sitting`; only
    the top-level legend is there.
  - vendor-freshness was not rerun after the fold.

## Checked and sound

**Repair shapes (lens a), measured.** Every case matches the decision 6/7 table:

| Input | Exit | stdout |
|---|---|---|
| Complete set, clean | 0 | printed |
| Complete set, damaged | 16 | printed |
| Lone chunk, clean / damaged | 0 / 16 | printed, "not a card on its own" |
| 2 of 4 chunks, clean / damaged | 0 / 16 | printed, "not the whole card" |
| Duplicate, over-full, duplicate with gap, index out of range, disagreeing totals, different set ids, chunk mixed with non-chunk | 2 | nothing |
| Complete set with altered content (set-id mismatch) | 2 | nothing |
| Lone malformed header (index >= count, count 1, flag clear, version 3), clean / corrected | 2 / 22 | nothing / printed |
| Spec §3 vector | 22 | printed |
| Clean non-card | 2 | nothing |
| Five errors | 2 | nothing |
| No arguments | 64 | nothing |

Across the 116 corpus sets the codes are only 0, 2, 16 and 22, stdout agrees
with the code every time, and main and testnet give identical results.

**Signals (lens c), measured.** At the mainnet sentence prompt: SIGINT dies by
signal (130), SIGTERM 143, SIGHUP 129, typed junk or "stop" exits 130, EOF
exits 2; the lock file is gone each time. Mid-session SIGTERM and SIGINT die by
signal with the RAM folder empty and ST-END-03 printed. First-prompt SIGINT and
SIGHUP die by signal. `main.rs` calls `wipe::finished()` before
`wipe::reraise(code)`, and `kit.rs` `stopped()` raises only after its unlink.

**Ruling 4 (lens b).** The seven variants moved to INVALID are raised only in
create and upgrade paths, with one also reachable from parentage
(`history.rs:1027`). The variants kept at 1 in verify, member and address are
real card-against-reference comparisons. I1 is the exception, and it lives in
`Failure` call sites, not in `Error::exit_class`.

**Clap mapping.** Empty `--time ""` and `--digest ""` give 2; a flag with no
value, a bad `--network` and a bare unknown flag give 64; an md1 given as a
card gives 2.
