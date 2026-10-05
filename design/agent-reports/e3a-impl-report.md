# E3a implementation report (plan §8 step 2)

Plan: `design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md`, GREEN at `1982788`.
Branch `claude/project-thread-eda5dt`, built on `1982788`. Nothing was pushed.

No device was attached at any point, and no real OTP or flash write was attempted. Everything
device-facing ran against the fake (`scripts/test/fake-picotool`). The real picotool 2.3.1 binary was
used for two things only. The argv probe and R's old e2e signing step ran it with no board attached; the
probe's every device call carries an absent serial, so each one exits 249 before it touches a device.

## 1. What was built

| commit | content |
|---|---|
| `06dbc4e` | TDD, red. A row-level fake picotool (`fake_picotool.py`, with `fake_otp_state.py` and `capture_to_entry.py`). It enforces F14 argv order and exits 99 otherwise. It also models the §6.1 fault modes. The `run-e2e-otp.sh` cases are 1 to 14, plus 3c, 5d, 5e, 5f and 7b. The first run gave 18 passed and 191 failed. |
| `f2d712b` | `scripts/lib/otp-read.sh` is extracted from R. It holds the one argv builder (`pt_get_argv` / `pt_set_argv` and the rest; `PT_SER` is never inherited from the environment) and the `try_` readers. It also holds the `og_parse` / `og_join` row parser and `ecc16` / `vote`. R's readers become thin wrappers, and R's behaviour is unchanged (see §4). |
| `f453b14` | `scripts/refugium-otp.sh` covers check, capture, `disable-otp-boot`, `invalidate-spare-keys` (with heal), `erase-range` / `save-range`, `inject-copy`, and `--log`. Also: `design/hardware/retail-otp.json` (schema `refugium-retail-otp/1` with no entries), the transcripts fixture README, and fixes to two e2e test helpers (see §6). |
| `5ee122c` | R's §4 changes: `--expect-key-invalid 0\|c` (only with `--sh2-precheck` / `--sh2-verify-valid`), and picotool 2.2.0-a4 or 2.3.1 accepted. Comments now say "any 3 of 8", name 0x040, and record the `-c 1` artefact; shellcheck SC2010 and SC2006 are fixed. In R's old fake, `version -s` gives 2.3.1 and KI comes from state. `run-e2e.sh` gains five KI cases. |
| `6ffbd8d` | `flake.nix` and `flake.lock` pin nixpkgs `a7868a727837f3c09cee2ce0ca671c76b1589fed` (narHash `sha256-KgItSKML8Xvte0B7/uGnBDsYzSnnKHcOaiUbgWBXLXw=`). They add `packages.picotool` (2.3.1) and `devShells.otp`. Also: `design/PICOTOOL_PIN.md`. |
| `28bbf47` | CI. In `test (rust + go)`: the steps `refugium-otp e2e (fake picotool)` and `shellcheck (OTP scripts)`. A new job `picotool-probe` (`picotool argv probe`) has `permissions: contents: read` and uses `cachix/install-nix-action@13d8dd58da0234aa297dedd986986ccb8e7f3e24` (v31.11.1, checked with `git ls-remote`). It runs `scripts/test/picotool-argv-probe.sh` against `nix build .#picotool`. |
| `4d46f3a` | Docs: the RUNBOOK gains prerequisites and "Refugium steps (E3b)", HARDWARE_INVENTORY gains "Retail OTP rows", and FOLLOWUPS gains the F-619 correction and F-701 progress. F-704 and F-705 are untouched, and `followups-status.sh` passes. |
| `6cb0de5` | The fake's `COPIES_IGNORED` now prints the vote in the equal-copies shape under `-c`, as its docstring and §6.1 / F20 specify (see §3). |
| (this commit) | This report. |

Files: `.github/workflows/release.yml`, `design/FOLLOWUPS.md`, `design/HARDWARE_INVENTORY.md`,
`design/PICOTOOL_PIN.md`, `design/RUNBOOK_custom_boot_key.md`, `design/hardware/retail-otp.json`,
`flake.nix`, `flake.lock`, `scripts/lib/otp-read.sh`, `scripts/pico2-bootkey-rehearsal.sh`,
`scripts/refugium-otp.sh`, `scripts/test/capture_to_entry.py`, `scripts/test/fake-picotool`,
`scripts/test/fake_otp_state.py`, `scripts/test/fake_picotool.py`,
`scripts/test/fixtures/transcripts/README.md`, `scripts/test/picotool-argv-probe.sh`,
`scripts/test/run-e2e-otp.sh`, `scripts/test/run-e2e.sh`.

## 2. Test results, case by case

`scripts/test/run-e2e-otp.sh` at HEAD gave 208 passed, 0 failed, exit 0. Passing checks per case:

| case | 1 | 2 | 3 | 3c | 4 | 5 | 5d | 5e | 5f | 6 | 7 | 7b | 8 | 9 | 10 | 11 | 12 | 13 | 14 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ok | 7 | 25 | 24 | 1 | 22 | 43 | 1 | 3 | 7 | 11 | 4 | 4 | 21 | 4 | 9 | 6 | 4 | 4 | 8 |

R's `scripts/test/run-e2e.sh` needs the fork checkout and real signing. Its results are in §4: the
five new KI cases pass, and three phases fail on a picotool 2.3.1 signing error that predates this work.

## 3. Mutations (§6.3)

The harness is a scratch script, not committed. Each mutation is an exact-string edit, asserted to apply
exactly once, to a full copy of the tool, lib, fake and suite. `run-e2e-otp.sh` then runs on that copy;
all 12 run in parallel. The table is the full rerun at `6cb0de5`. Each row gives the red run: case id and failing assertion.

| # | mutation | named case | red run (case: failing assertion) |
|---|---|---|---|
| M01 | drop copy rule (a) | 3, copy 2 odd, `SUPPRESS_WARNING=1` | 12 failed, all case 3, e.g. `[3] BOOT_FLAGS0 copy 2 odd (SUPPRESS_WARNING=1) -- exit 0, want 2` (the same for copies 0, 1 and 2 of BF0, BF1, USB_BOOT_FLAGS, CRIT1 and CRIT0) |
| M02 | drop the `-c 1` read from (a) | 3, copy 0 odd, `SUPPRESS_WARNING=1` | 5 failed, all case 3, copy 0 only: `[3] BOOT_FLAGS0 copy 0 odd (SUPPRESS_WARNING=1) -- exit 0, want 2`, and the same for BF1, USB_BOOT_FLAGS, CRIT1 and CRIT0 |
| M03 | drop the WARNING trap (b) | 3c | `[3c] BOOT_FLAGS1 copy 0 odd (COPIES_IGNORED=1 SUPPRESS_RAW_VALUE=1) -- exit 0, want 2` |
| M04 | drop `--ser` from the builder (`otp get`) | `FAKE_REQUIRE_SER=1` (exit 99), case 13 | 174 failed across cases 1-14 except 9 (the suite runs with `FAKE_REQUIRE_SER=1`, so reads without `--ser` exit 99), e.g. `[2] rehearsal: CRIT0 not 0 -- exit 1, want 2`. The failures include case 13 |
| M05 | builder emits `-n` before `-c` | fake exit 99 | 173 failed across 1-14 (all except 13), e.g. `[1] rehearsal-shaped state: check PASS -- exit 1, want 0`. The fake exits 99 on the order |
| M06 | drop the post-write check | 6 `FAIL_READ_AFTER_WRITE` | `[6] FAIL_READ_AFTER_WRITE: exit 3 -- exit 0, want 3` |
| M07 | E replaced by copy 0 \| T in conditions 1-3 | 5d | `[5d] 0x903/0x103/0x103 refused -- exit 3, want 2` |
| M08 | drop condition 4 alone | 5, 0x803/0x003/0x003 under `COPIES_IGNORED=1` | `[5] 0x803/0x003/0x003 under COPIES_IGNORED refused -- exit 0, want 2`. Also red: `[5] 0x103/0x003/0x003 ... -- exit 3, want 2` and `[5e] 0x803/0x003/0x003 ... -- exit 0, want 2` |
| M09 | E's bit 11 fixed at 0 | 5f | `[5f] bit 11 in all copies: disable-otp-boot writes -- exit 2, want 0`, and `[5f] bit 11 still set in every copy -- got 0x000800 ..., want 0x002800 ...` |
| M10 | drop the RAW_VALUE cross-check (cond 4 and the equal-copy test) | 5, 0x103/0x003/0x003 under `COPIES_IGNORED=1` | `[5] 0x103/0x003/0x003 under COPIES_IGNORED refused -- exit 3, want 2`. Also red: `[5] 0x803/...` and `[5e] 0x803/...` |
| M11 | derived flag accepts any equal value | 5e | `[5e] KEY_INVALID 0x1 in all copies: disable-otp-boot refused -- exit 0, want 2` |
| M12 | identity gate skipped for erase | 7 | `[7] erase-range --profile rehearsal on a retail-shaped board -- exit 0, want 2`, and `[7] no erase/save/load argv recorded -- argv matching /^(erase\|save\|load)\t/ was issued` |

Every mutation is killed by its named case.

**Finding, fixed in `6cb0de5`.** In the first full run, M03, M08 and M10 **survived**. The cause was in
the fake, not in the tests or the tool. §6.1 defines `COPIES_IGNORED=1` as "`-c 1` returns the vote", and
the fake's own docstring says "in the equal-copies shape". F20 adds that RAW_VALUE never prints under
`-c`. The fake instead printed the vote *with* the WARNING and the RAW_VALUE list. The tool's `-c 1`
trap refuses any WARNING or RAW_VALUE on that read (see §7), so it refused cases 3c and 5's
`COPIES_IGNORED` cases before the targeted checks could run. Under that fake, those cases could not tell
the mutants from the original. With the fault mode as specified, the unmutated suite is still 208/0 and
all three mutants go red in their named cases. No test was weakened or skipped.

## 4. R's behaviour and R's old e2e

The fork `bg002h/seedhammer` at `d156a3e` was cloned into scratch. R's e2e then ran with the locked
nixpkgs' picotool 2.3.1, TinyGo and Go on PATH:

- At `1982788` (baseline) and at `f2d712b` (extraction only), the outcomes are identical: 40 passed and
  3 failed, with only random key hashes differing. **The extraction does not change R's behaviour.**
- At HEAD: 45 passed (including the new KI cases) and 3 failed, the same three.
- All three failures (phases 3, 5 and 6) have one cause. `scripts/sign-firmware.sh:104` runs
  `picotool seal --sign --clear`, which exits 248 with `ERROR: unknown sram end` on picotool 2.3.1 with
  the TinyGo blinky. Without `--clear`, `seal --sign` works and the result verifies.

**This is a plan-fact blocker for bench R0, and it is not fixed here.** It is out of §8 step 2's scope,
and the fix in sign-firmware's signing argv is a risk-set change. The plan assumed R runs unchanged on
2.3.1, and for `seal --clear` it does not. It is recorded in `design/PICOTOOL_PIN.md`, the RUNBOOK
prerequisites and F-701's progress note. Before R0 on the bench there are two choices: decide whether
`--clear` is needed (and how to drop it), or pin a picotool that accepts it.

## 5. Probe and shellcheck

- **The argv probe** ran against the real 2.3.1 binary (`nix build .#picotool` gave
  `/nix/store/5cxc3bjswwjpgw9i3sdlp5xw1y4v82hc-picotool-2.3.1`, and `version -s` printed `2.3.1`). With no
  board attached, all 53 builder shapes exit 249 with the absent serial in the message. `version`, the
  `otp list` fingerprint and bare `info` pass. The negative control `otp get -n -c 1 --ser S 0x04b` loses
  the serial, as F14 predicts, and the probe detects it. Result: 58 ok, `PROBE PASS`.
- **shellcheck `-x -S warning`** is clean on `refugium-otp.sh`, `lib/otp-read.sh`,
  `pico2-bootkey-rehearsal.sh`, `picotool-argv-probe.sh` and `run-e2e-otp.sh`.

## 6. Not run, and why

- **`nix develop .#otp` was not verified.** In this container, nix's build sandbox cannot use
  `/dev/null`, which is root-only for nixbld. `nix build .#picotool` worked. The devShell's packages were
  fetched from the same locked nixpkgs and put on PATH instead, and all runs above used them. The
  devShell needs to be entered once on a normal machine.
- **The CI jobs have not run.** Nothing was pushed. The probe job is not yet a required check; branch
  protection is the operator's to set.
- **No device steps were run.** R0 and the E3b steps are bench work.
- **Test fixes made during implementation.** In `run-e2e-otp.sh`, seven `expect` calls lacked the rc
  argument (2). The `rehrow` / `retrow` helpers now take `@S` and substitute the serial after `fresh`.
  Both were bugs in the red suite; neither loosened an assertion.

## 7. Plan ambiguities and the readings taken (the safer one in each case)

- A `-c 1` read that shows a WARNING or a RAW_VALUE counts as unreadable (F20 says neither prints under
  `-c`).
- Bare `info` (the no-board / two-board census) is exempt from `--ser`; every device call carries it.
  The probe checks both.
- The `COPIES_IGNORED` shape is the quiet vote (see §3).
- `SH2_BOOTKEY_FP` is copied into the lib. Case 14 asserts that it equals `sh2-flash`'s value.
- Capture paths are relative to the JSON and must sit under `captures/`.
- For a `_TEST_ONLY` entry, `check` exits 0 with `RESULT: TEST ENTRY` (never PASS). Write commands and
  any `--execute` with such an entry exit 1.
- Retail entries are required only for `check` and the two write commands, not for erase or save.
- `--rehearsal-slot1-key` is refused on commands that do not judge slot 1.
- `save-range` refuses to overwrite its output.
- An erase or save failure exits 2 ("condemned").
- These exit 1: a wrong confirmation, no board, two boards, and a CHIPID ≠ `--ser`.
- The post-write check uses the pre-derived flag.
- In the SLOT1 judgement, KEY_VALID is taken from the vote when BF1 is the write target.
- R's non-OTP picotool calls (`load --verify`, `reboot`, `info`, `seal`) stay off the builder, which is
  §3.0's scope for OTP calls.
- Mutation readings:
  - M04 drops `--ser` from `otp get` only (the read path every case exercises);
  - M10 drops the RAW check but keeps the WARNING check in the equal-copy test;
  - M11 is a two-line faithful mutant (the derived flag carries the raw value through to the
    post-check).
- emit.py does not generate this repo's `release.yml`, so the workflow was edited directly.
