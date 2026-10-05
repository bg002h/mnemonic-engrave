# E3a post-implementation execution review, round 2 (scoped re-review of the r1 fold)

Date: 2026-10-05. Reviewer: independent subagent. Fold under review: `git diff 70cb771 695f128`
(scripts, .github, plan, RUNBOOK, PICOTOOL_PIN, fixtures) on `claude/project-thread-eda5dt` at 695f128.
Prior report: `design/agent-reports/e3a-exec-review-r1.md` (0C / 1I / 9M / 6N). Implementer's account:
"Review r1 fold" in `design/agent-reports/e3a-impl-report.md` (checked, not trusted). Only this report was written.

## 0. What I re-ran

| gate | result |
|---|---|
| `scripts/test/run-e2e-otp.sh` (once, captured to a file) | **235 passed / 0 failed**, exit 0 (matches the implementer's count) |
| `scripts/test/picotool-argv-probe.sh` vs `/nix/store/5cxc3bjs...-picotool-2.3.1/bin/picotool`, no board | 58 ok, 53 shapes, PROBE PASS, exit 0. Every `otp set` shape now carries `0x000000`. Bare `picotool info` here: exit 249, "No accessible RP-series devices in BOOTSEL mode were found." |
| shellcheck 0.11.0, CI's command (`-x -S warning`, 5 files, from repo root) | clean, exit 0 |
| same at `-S info` on tool, R, probe | no SC1091, SC2178 or SC2179: the lib is now followed |
| `release.yml` parsed with PyYAML | valid; `jobs.test.permissions == {contents: read}` |
| probe with stub picotools (see 2.1) | aborts exit 2 on any non-249 / non-"No accessible" `info`, before any device argv |
| two hand-made mutants of the tool (line 308 CHIPID gate removed; line 346 `-c 1` noise trap removed), run through the new cases | both killed by exactly their case (`chipid mismatch -- exit 0, want 1`; `noisy -c1 -- exit 0, want 2`) |
| case 12's bench branch (never executed in the repo: no `BENCH_RUN`) run by me against fixtures built from the fake (stock capture + inject-copy bf0-copy3 / bf1-copy0 + check log + capture) | all 14 assertions pass, including `copies differ` in the logs and `RAW_VALUE=` in `0x048_named.txt` / `0x04b_named.txt`, and `REPLAY PASS` on both captures |

Not re-run: the 25-mutant harness (not committed) and R's old e2e (R changed by one shellcheck comment only).

## 1. Each r1 finding

| id | status | evidence |
|---|---|---|
| I1 | **fixed** | `picotool-argv-probe.sh:59-69`: `require_no_board` runs bare `info`, needs rc 249 and "No accessible", else prints ABORT and `exit 2`. Called at `:70` before anything else, at `:96` before every `probe "${argv[@]}"`, and at `:104` before the negative control. The only `probe` calls not preceded by it are `version -s`, two `otp list` (no device) and the bare `info` (read-only). Payloads are now `0x000000` (`otp-read.sh:182-185`). `PICOTOOL_PIN.md:82` and the probe header say to unplug. |
| M1 | **fixed** (see N3) | `refugium-otp.sh:769-770` requires the read-back to exist and be 4096 bytes; `:804` `tr ... || die2`. Cases `run-e2e-otp.sh:503-525` (SAVE_SHORT, SAVE_NO_FILE, a `tr` shim) pin the messages; M16-M18 claimed, and the probe-size message is pinned. |
| M2 | **CRIT1 half fixed**, white-label half knowingly open | `:235-238` rejects crit1 without bit 0 or with bit 2, exit 1; case 14 covers `0x000000` and `0x000005` with the message. |
| M3 | **fixed** | `:645` `exit 4` only on the `RESULT_OK=1 && TEST_ONLY=1` path; header `:33` documents it. Case 1 expects 4 plus the message plus unchanged state. See 2.3. |
| M4 | **fixed** | (1) CHIPID != `--ser`: case 4, exit 1, "reports CHIPID 1111222233334444", `same`, `no_argv ^otp\tset`. (2) NOISY_COPY_READS / NOISY_BARE_READS: case 3, exit 2, exact message, `same`. (3) read-back sizes: above. (4) ENABLE with CS0 0xb: case 2, exit 2, "CS0_SIZE must be 0xc". I confirmed the first two by mutation. |
| M5 | **fixed** | `heal_no` now takes a regex (`:323`); all 11 calls pass one; the no-write post-check case pins `REFUSED: post-check read`. Each keeps the `same` snapshot check. |
| M6 | **fixed** | case 4 uses a generated tree with `entries: []` (`:309-316`); `release.yml:109-120` treats `design/hardware/` as non-docs (note: PR events have an empty `github.event.before`, so PRs already ran in full). |
| M7 | **fixed** | `# shellcheck source-path=SCRIPTDIR` in the tool, R and the probe; `copies_hex` local renamed; CI shellcheck now also covers the probe and the e2e script. Verified clean, lib followed. |
| M8 | **fixed** | `release.yml:70-75` job-level `permissions: contents: read` on `test`; no step in the job needs more (checkout, git fetch, cargo, go, e2e, shellcheck; the demo link check only does `curl` GETs with the token). See 2.4. |
| M9 | **fixed, one part unexecutable in the repo** | plan R5/R7 add the `capture`; README lists the captures; case 12 requires `copies differ` and `RAW_VALUE=`; case 10 requires `copies differ`. The bench branch cannot run until `BENCH_RUN` exists, so I ran it on synthetic fixtures (section 0): it works. |
| N1 | header fixed, second half open (N1 below) | `:733` prints "check (no write was issued):"; `:711` "healing an unequal copy" still prints on a dry run. |
| N2 | fixed | RUNBOOK names `refugium-wallet` `IMPLEMENTATION_PLAN_mr_gui_v1.md` §9 item 14. |
| N3 | fixed | `slot_what`; case 1 asserts both texts and that rehearsal never says "fork key". |
| N4 | fixed | marker carries the parent's PID; see 2.2. |
| N5 | fixed | `-c 1` manifest value now comes from the named read (vote when clean, RAW_VALUE[0] otherwise; capture fails on WARNING without RAW_VALUE). Case 12 under COPIES_IGNORED with 0x803/0x003/0x003 gives `MISMATCH.*0x04b_c1`. |
| N6 | fixed | `tree8` line removed. |

## 2. Fold-introduced defect hunt

### 2.1 Probe cannot issue a device argv unless bare `info` just exited 249 with "No accessible"
Verified by reading and by three stubs: (a) `info` exit 0 "Program Information": ABORT, exit 2, argv log is a single `info`;
(b) `info` exit 1 with the text: ABORT, exit 2, single `info`; (c) a board appearing after three clean `info` calls:
the probe ran `version -s`, two `otp list` and one `info --ser ZZPROBE...` (read-only, and the immediately preceding gate had passed),
then aborted exit 2 with no `otp set`, `erase` or `load` issued. `require_no_board` runs in the main shell, so its `exit 2` ends the probe;
the trap still removes the temp dir. The check-then-use window is inherent (a board plugged in between two calls); each `otp set`
now follows a check by one `exec`, and the payload is `0x000000`, so the residual exposure is nil for OTP. Nothing found.

### 2.2 `--log` marker ($$ / $PPID)
`refugium-otp.sh:92-97`: the first run sets `REFUGIUM_OTP_LOGGING=$$` and re-runs `$0`; the re-run's `$PPID` is that `$$` because bash execs the
last simple command of the forked pipeline element directly (the implementer's claim; confirmed by the "one run, not two" case and by
`timeout` as parent). Loop analysis: the re-run sees marker == `$PPID`, skips the block, and does not re-export a new marker, so depth is at most 1.
If a future bash did not exec directly, the re-run would see a mismatch once more and re-exec once, giving a duplicated log, not a loop (the
grandchild's `$PPID` would equal the child's `$$`). An inherited value cannot silence `--log` unless it equals the live parent PID. The pipeline keeps `PIPESTATUS[0]`.
Cases cover exit 0 and 2, one transcript, and inherited `=1` for both. Nothing found.

### 2.3 TEST_ONLY exit 4 and no other exit change
The diff to `refugium-otp.sh` adds exactly one new exit path (`:645`, reachable only with `TEST_ONLY=1` and `RESULT_OK=1`) and two new `die1/die2`
refusals on already-refused classes (crit1 schema: exit 1 like every schema failure; read-back/`tr`: exit 2 like `condemned`). No `exit`/`die`
numbers on other paths changed. `TEST_ONLY=1` already refuses `--execute` and every write command (`:176-178`), so exit 4 cannot be reached after a write. Nothing found.

### 2.4 CI
`test` has no `continue-on-error`; both new steps are plain `run:` steps, so a non-zero exit fails the job, which is the required context. The
e2e exits 1 on any failed check (`:run-e2e-otp.sh` last lines); shellcheck exit propagates. The job-level `permissions` removes `id-token`,
`attestations` and `contents: write` from `test` only; `assemble` keeps the workflow-level grants and still `needs: test`. The probe job is unchanged
(`contents: read`, SHA-pinned nix action). No job gained a permission. Nothing found.

### 2.5 New cases assert message plus unchanged state
Yes for every refusal case added (`same`, plus `no_argv` where a write must not follow). The exceptions are correct: cases 8 (short read-back,
failed `tr`) and 6 (post-write read failure) run after a write by design, so they pin the message and the missing follow-up argv instead.

## 3. Findings

### Minor
None.

### Nit
- **E3A-X2-N1. The dry run still prints "healing an unequal copy".** `refugium-otp.sh:711` prints it before the `EXECUTE != 1` exit at `:717`, so an operator log
  from a dry run reads as though a heal was done. The r1 N1 named both messages; only the post-check header was changed. Fix: move the `say` after `confirm`, or word
  it "would heal" when `EXECUTE != 1`. The same applies to `:700` "copies equal E & ~T: writing T.".
- **E3A-X2-N2. Exit 4 is documented only in the script header.** The RUNBOOK exit-code table (`RUNBOOK_custom_boot_key.md:573-580`) and plan §3.2 (`...:205`, "never PASS") do not
  list it. It is a test-only path, so no operator sees it, but a caller keying on codes reads the RUNBOOK. Fix: one row ("4: check matched a TEST entry; only with
  REFUGIUM_OTP_RETAIL_JSON_TEST_ONLY").
- **E3A-X2-N3. The two new M1 refusals use `die2`, not `condemned`.** r1 asked for `condemned` on the `tr` failure. At `:770` the marker is already written at
  0x10000000 and the message ends "(refused)" without saying so; `:804` follows an erase that was issued but not verified. Exit is 2 either way, and the next step for
  both is a re-run of `erase-range`, so this is wording. Fix: route both through `condemned`, or add "a 4 KiB marker was written at 0x10000000 and is still there".
- **E3A-X2-N4. CI's shellcheck is version-dependent.** I ran 0.11.0; `ubuntu-latest` ships its own packaged shellcheck (not pinned), and `-x` now follows the lib, so
  analysis widened. A different version could add or drop a warning on the five files. Cheap hedge: `shellcheck --version` is already printed in the step; if the first CI run
  differs, pin it. Not verifiable here.

VERDICT: 0C / 0I / 0M / 4N
