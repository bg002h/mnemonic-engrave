# E3a post-implementation adversarial execution review, round 1

Date: 2026-10-05. Reviewer: independent subagent (opus). Diff under review: `git diff 1982788 70cb771`
(9 commits) on branch `claude/project-thread-eda5dt`. Plan: `design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md`
at `1982788` (draft 5, GREEN). Facts: `design/agent-reports/e3a-g-facts.md`. Implementer report:
`design/agent-reports/e3a-impl-report.md`.

## 0. What I re-ran myself

| gate | result |
|---|---|
| `scripts/test/run-e2e-otp.sh` (fake picotool) | **208 passed / 0 failed**, exit 0, 6 m 24 s wall |
| `scripts/test/picotool-argv-probe.sh` against `/nix/store/5cxc3bjs…-picotool-2.3.1/bin/picotool`, no board | **58 ok, 53 shapes, PROBE PASS**, exit 0 |
| `shellcheck -x -S warning` on `refugium-otp.sh`, `lib/otp-read.sh`, R (shellcheck 0.11.0) | clean, but see M7: the lib is not followed |
| `cachix/install-nix-action@13d8dd58…` | `git ls-remote` shows that SHA is `refs/tags/v31.11.1` |
| R's version gate on 2.2.0-a4 (`/nix/store/j7pl045i…-picotool-2.2.0-a4`) | `version -s` prints `2.2.0-a4`, exit 0, so R's `2.2.0-a4\|2.3.1` gate works from the fork's shell |
| Fake output vs G-facts §2.2 | fake `otp get -n` for (a) equal, (b) BF1 0x803/0x003/0x003, (c) `-c 1`, (d) CRIT1 8 copies with copy 5 odd (124-character line, no space before `(WARNING`), (e) unnamed 0x04a: **byte-identical** to the literal blocks (`cat -A`) |
| `og_parse` on the G-facts literal blocks | accepts (b), (d), (g) the flipping note with a wrapped warning, (g) equal flipped copies, (e), (f) `-r`, (f) ECC-invalid at width 4, and an 8-copy CRIT1 with a flipping note. It refuses empty output ("expected one ROW header"). All values, WARNING and RAW lists parse correctly |
| M08 (drop heal condition 4), re-run by hand | the original refuses 0x803/0x003/0x003 under `COPIES_IGNORED=1` with exit 2; the mutant exits 0. Killed |
| No-write branch, post-check failure | with `FAIL_GET_AFTER=N/2` the failure does land in the post-check (`REFUSED: post-check read`), exit 2. The case is not vacuous |
| CHIPID ≠ `--ser` (hand-built state: serial 66D3…, CHIPID 1111…) | exit 1, "reports CHIPID 1111222233334444". The gate works, but no case covers it (M4) |

**Not re-run.** R's old e2e (`scripts/test/run-e2e.sh`) needs the fork checkout and real signing, and
the `sign-firmware.sh` 2.3.1 seal failure is out of scope. I did not re-run the full 12-mutation
harness: it is not committed. M08 was re-run as above, and I checked the others by reading the
code against the named cases.

## 1. Verified sound (no finding)

- **Heal rule vs plan §3.3** (`refugium-otp.sh:665-703`). The branch order is: equal and = E → no
  write, post-check exits 2 on failure; equal and = E & ~T → write; unequal → conditions 1-4 →
  write; anything else → exit 2. Conditions 1-3 use the per-copy reads (`-c 1` and the bare reads).
  Condition 4 requires RAW_VALUE and `raw_matches` copy for copy. E comes from the expected state:
  the entry's bits or 0, plus T, plus bit 11 only when every copy agrees (`:669-680`). It is never
  computed from copy 0.
- **Every write argv is constant.** `otp set -s BOOT_FLAGS0.DISABLE_OTP_BOOT 0x1 --ser S`;
  `otp set -s BOOT_FLAGS1.KEY_INVALID 0xc --ser S`; `otp set -s 0x04a 0x002000 --ser S`;
  `otp set -c 1 -s 0x04b <copy0|0x800> --ser S`; `erase -r 0x10000000 <0x11000000|0x10400000> --ser S`;
  `load -v <marker>.bin -o 0x10000000 --ser S`. The tool never emits `otp load`. Every write runs
  only after the single-device gate, the CHIPID = `--ser` check, the identity gate, `--execute`
  and the typed `BURN <step> <CHIPID>`. A dry run exits 0 before `confirm`.
- **Exit 3 discipline.** `die3` is reachable only after `pt_exec` of `otp set`. `cmd_postcheck 1`
  maps a gate, read or judge failure to exit 3, and `cmd_postcheck 0` maps the same failures to
  exit 2. The `--log` re-exec keeps `PIPESTATUS[0]`. Nothing in the tool or lib uses
  `local x=$(…)` with a reader, `|| true`, or a reader inside `$(…)`. ROW_NAMES/ROW_RESULTS live
  in the main shell.
- **Compared-row set.** The required set comes from the profile and the matched entry
  (`:582-592`). A missing row is FAIL, and a row compared twice is TOOL_ERR.
- **R's SH2 modes still have no write path.** The only `otp set`/`otp load` calls run in phases 1
  and 4 through `run` and the builder. `:694`/`:702` are message text. The KEY_INVALID 0|c
  handling matches plan §4.
- **COPIES_IGNORED change (6cb0de5).** The quiet vote under `-c` matches plan §6.1 ("`-c 1`
  returns the vote"). It is also the only shape under which plan case 3c ("only the WARNING can
  refuse") can mean anything. It makes the fake *harder* for the tool, not easier, so I accept
  it. It does leave one trap untested (M4).
- **Fake fidelity** on every path the tool uses. The `otp get` layout, wrapping and glued CRIT
  warning match G-facts. `otp set` computes the value from copy 0 and fails a later superset row
  only after the earlier rows have burned (F11/F19/F21, exit 248). Flash is refused past CS0
  (157), and erase works sector by sector. `load` models the guess_flash_size refusal. `--ser`
  miss is 249 with the wrapped text; two boards give 248, and bare `info` prints "Multiple…" with
  exit 0 (F23). The argv-order enforcement is strict (exit 99).
- **CI.** The two new steps sit inside the required `test (rust + go)` job and fail it on a
  non-zero exit. The probe job has `permissions: contents: read`, and the nix action is pinned by
  a full SHA that I verified.

## 2. Findings

### Important

**E3A-X-I1. The argv probe can write OTP and erase flash on an attached board, under exactly the
regression it exists to detect.**
Evidence: `scripts/test/picotool-argv-probe.sh:33-81` runs every line of
`pt_print_argvs` (`scripts/lib/otp-read.sh:167-188`) against the *real* picotool. That includes
16 `otp set` variants writing `0x000803` to row `0x04b` (`-c 1`, `-r`, `-e`, `-s` combinations),
`erase -r 0x10000000 0x10001000`, `load -v probe.bin -o 0x10000000` and `otp load probe.json`. The
script's only safety is that `--ser ZZPROBE00000000` matches no board. Nothing enforces "no board
attached":

- the bare `info` line inside the loop only counts a FAIL and carries on;
- the header just *says* "with no board attached".

The probe exists to catch the case where `--ser` stops being parsed (F14). In that case each
`otp set` lands on whatever RP2350 is in BOOTSEL. On a SeedHammer (KEY_VALID 0x3), the `-s`
variant burns KEY_INVALID bit 11, which revokes slot 3 for good. That is the KEY_INVALID 0x8
state `disable-otp-boot` then refuses. The `erase` shape wipes the first sector of the
firmware. The likeliest moment for such a regression is moving the pin, and
`design/PICOTOOL_PIN.md:82` tells the operator to "Re-run the argv probe" at that step, on the
bench box, where a board may well be plugged in. CI has no board, so CI is safe; the local runs
the plan and the pin doc prescribe are not.

Fix:
- before any other call, run bare `picotool info` and require exit 249 and
  `No accessible RP-series devices`, else abort with "unplug every RP2040/RP2350 board";
- print the same requirement in the header and in PICOTOOL_PIN step 4;
- secondary: make the payloads as inert as possible, e.g. `otp set … 0x04b 0x000000` (with `-s`
  this rewrites copy 0's own value), and probe a sector the tool never relies on.

### Minor

**E3A-X-M1. Two local I/O failures read as success in the flash path.**
- `refugium-otp.sh:791`: `tr -d '\377' < "$f" > "$TMPD/left.bin"` has no exit check. If the
  write of `left.bin` fails (a full tmpfs, for example), `left.bin` can be shorter than the true
  count, possibly empty. `left=0` then prints "erased and verified (every byte 0xFF)".
- `:758`: the alias probe compares `sha256sum < "$got"` without checking that `$got` exists and is
  4096 bytes. A missing or short read-back reads as "no copy of the marker". Real picotool writes
  the exact range, so this needs a local failure, but it fails open.

Fix: `tr … > left.bin || condemned "…"`, and require `stat -c%s "$got"` = 4096 before comparing.

**E3A-X-M2. The retail check does not enforce the plan's stated CRIT1 invariants, and the schema
does not check that entries are internally consistent.**
Plan §3.2's table says CRIT0/CRIT1 equal the recorded entry "(SECURE_BOOT_ENABLE 1,
DEBUG_DISABLE clear)". The tool compares only to the entry (`refugium-otp.sh:480-481`), and
`load_retail` (`:204-248`) does not check `crit1 & 1 == 1` or `crit1 & 4 == 0`. An entry captured
from, or mistyped as, a non-secure unit would make `check --profile retail` PASS a board with
secure boot off. The schema also does not check that `white_label_rows` is consistent with
`usb_boot_flags` bit 22 and `usb_white_label_addr`. Fix: assert these in `load_retail` (exit 1).

**E3A-X-M3. A `_TEST_ONLY` `check` exits 0.**
`refugium-otp.sh:610` with `:634`: the RESULT line says "TEST ENTRY — not a retail check", but the
exit code is 0, which the tool's own table and the RUNBOOK define as "pass". Any caller that keys
on the exit code (the Sitting image, a script) reads it as a pass. This is the implementer's
reading in report §7. The plan's intent ("never PASS") is better served by a distinct non-zero
exit, e.g. 2, or exit 1 since it is an environment condition.

**E3A-X-M4. Some gates are not killed by any case.**
1. The CHIPID = `--ser` comparison (`:299`). Every fixture has serial = CHIPID; case 4's "`--ser`
   mismatch" uses a serial that matches no board, so the `info --ser` gate refuses first. I
   confirmed the gate works by hand (§0), but deleting `:299` keeps the suite green. This is the
   F15 white-label-serial case.
2. The `-c 1` / bare-read WARNING and RAW traps in `read_group` (`:337`, `:341`). After 6cb0de5
   the fake never prints either under `-c`, so deleting them survives.
3. The read-back size check (`:789`) and M1's missing checks.
4. Retail "ENABLE set ⇒ CS0_SIZE = 0xc" (`:541`). The fixture entry always has CS0 0xc.

Fix: a fake state with serial ≠ CHIPID (expect exit 1 "reports CHIPID"); a fake fault mode
printing the noisy F-619 shape under `-c` (expect exit 2); an entry with ENABLE set and CS0 0xb
(expect FAIL FLASH_DEVINFO).

**E3A-X-M5. Some refusal cases assert only the exit code and an unchanged state, with no
message.**
`run-e2e-otp.sh:323` (`heal_no`, 11 cases across 5, 5d, 5e, 5f) and `:369` pass `""` as the
regex. Plan §6.2's header requires "exit code, a message regex". An exit-2 refusal from the wrong
gate would pass. Today the mutation runs show the cases are reached, but nothing pins *why*. Fix:
regexes such as `heal refused: copy .* outside E`, `RAW_VALUE=.* disagrees`, `bit 11`,
`KEY_INVALID is 0x1`, `post-check read`.

**E3A-X-M6. CI skips the OTP e2e for retail-value PRs, and case 4 will break once the first
entry lands.**
- The docs-only detector (`release.yml:92-111`, regex `^(design/|[^/]+\.md$)`) classes
  `design/hardware/retail-otp.json` and `design/hardware/captures/*` as docs. A PR that changes
  only the retail values the tool gates on therefore skips both new steps.
- `run-e2e-otp.sh:275` runs the repo's live `$TOOL` against the repo's live `retail-otp.json`
  and expects "no recorded retail values".

So the H0 PR that adds SeedHammer #1's entry merges green (steps skipped), and the next code PR
goes red on case 4. Fix:
- case 4 uses a generated tree whose JSON has `entries: []`;
- exclude `design/hardware/` from docs-only, or add an always-on step that runs the tool's schema
  check on the committed JSON.

**E3A-X-M7. CI's `shellcheck -x` does not follow the shared library.**
`# shellcheck source=lib/otp-read.sh` (`refugium-otp.sh:37`; R `:90`) resolves relative to the
cwd, and the CI step runs from the repo root. shellcheck reports
`SC1091 (info): Not following: lib/otp-read.sh`, which `-S warning` hides, so the cross-file
analysis the plan's `-x` was meant to buy is not happening. With `-P SCRIPTDIR` the lib is
followed and shellcheck raises SC2178/SC2179 at `refugium-otp.sh:347`: a false positive from the
lib's `local -a o` against `copies_hex`'s string `o`. Fix: add
`# shellcheck source-path=SCRIPTDIR` and rename that local.

**E3A-X-M8. The new steps run with write tokens.**
`release.yml:43-46` grants `contents: write`, `id-token: write` and `attestations: write`
workflow-wide. The `test` job has no job-level `permissions`, so `run-e2e-otp.sh` and shellcheck
run with a write token and OIDC on pushes. This is pre-existing for the job, but the review lens
says nothing new should hold write tokens. Fix: job-level `permissions: contents: read` on `test`
(nothing in it writes).

**E3A-X-M9. The bench evidence will never exercise the safety-critical parse on silicon.**
- R1's capture runs on a stock board, so its committed transcripts never contain a RAW_VALUE or
  WARNING line.
- Case 12's R5/R7 assertions (`run-e2e-otp.sh:535-538`) grep only `FAIL +BOOT_FLAGS0/1`. That
  also matches `unreadable: unexpected line…`, which is what a real-silicon parse failure would
  print.

R6 would then refuse with fail-closed exit 2, so this is not a false PASS on a write. But the
plan's "R5 must refuse naming BOOT_FLAGS0 copies" would look green for the wrong reason. Fix:
- add a `capture` (or raw `otp get` transcripts) after R5 and after R7 to the bench fixtures;
- grep `copies differ` in the R5/R7 logs.

### Nit

- **E3A-X-N1.** `cmd_postcheck 0` prints the header "post-write check:" (`:722`) although no write
  was issued. "healing an unequal copy" also prints on a dry run. Neither changes behaviour, but
  both read wrongly in an operator log.
- **E3A-X-N2.** RUNBOOK `:565` says "only if plan §9 item 14 allows it". In this repo "plan"
  reads as the E3a plan, whose §9 has no item 14. Name `refugium-wallet`
  `IMPLEMENTATION_PLAN_mr_gui_v1.md` §9 item 14.
- **E3A-X-N3.** Under rehearsal, SLOT1's PASS text reads "fork key" (`:505`), but the slot holds
  `--rehearsal-slot1-key`.
- **E3A-X-N4.** If `REFUGIUM_OTP_LOGGING` is already set in the operator's environment, `--log` is
  silently ignored (`:89`). Use a unique value, such as the parent's PID, rather than a bare
  presence check.
- **E3A-X-N5.** The capture manifest's `-c 1` expected value is the same `-c 1` read
  (`:938-939`). That replay row is circular and proves only the parser.
- **E3A-X-N6.** `run-e2e-otp.sh:451` builds `$TMP/tree8`, which is never used.

## 3. Checks on the implementer's §7 readings

- The `-c 1` WARNING or RAW reading counts as unreadable. Safe; but see M4.2.
- Bare `info` is exempt from `--ser`. Correct and necessary (F23); the probe and case 13 both
  account for it.
- The COPIES_IGNORED quiet shape. Accepted (§1).
- `_TEST_ONLY` `check` exits 0. Disagree: M3.
- Retail entries are required only for check and the writes, not for erase/save. This matches
  plan §3.4 (identity gate only).
- `--rehearsal-slot1-key` is refused on other commands. Stricter than the plan; harmless.
- Exit 1 for a wrong confirmation, no board, two boards, or CHIPID ≠ `--ser`. Consistent with the
  RUNBOOK table; nothing is written on any of them.
- The post-check uses the pre-derived flag, and SLOT1's KEY_VALID comes from the vote when BF1 is
  the target. Both are safe because heal condition 2 holds every copy's KEY_VALID at E's 0x3, and
  any refused heal exits 2 before the write.
- R's non-OTP calls (`load --verify`, `reboot`, `seal`, `info`) stay off the builder. This
  narrows plan §3.0 ("every picotool call"). It is harmless because R passes no `--ser` anyway.
  Noted, not raised.

## 4. Lens summary

1. **False PASS:** none on an irreversible step. Two fail-open local-I/O paths (M1), one retail
   invariant gap (M2), and the exit code of a test-only path (M3).
2. **Unintended burn:** the tool is clean. The probe is not (I1).
3. **Fake fidelity:** faithful on every path used. 6cb0de5 conforms to the plan.
4. **Test vacuity:** M4, M5 and M9; case 12 is vacuous by design until BENCH_RUN exists.
5. **R:** no regression found. The SH2 modes still have no write path, and 2.2.0-a4 `version -s`
   is verified.
6. **CI:** the gates run and fail the job; the probe is read-only; the SHA is verified. See M6, M7
   and M8.
7. **Docs:** the RUNBOOK E3b flags match the tool, and every irreversible step requires Brian's
   typed go-ahead naming the CHIPID and the step. PICOTOOL_PIN's "re-run the argv probe" feeds I1.
   N2.

VERDICT: 0C / 1I / 9M / 6N
