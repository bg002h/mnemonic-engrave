# E3a recon: map of the existing OTP / boot-key tooling (mnemonic-engrave)

Recon only. Nothing in the repo was modified except this file. All line numbers are
for the files as found on disk 2026-10-05 (`scripts/pico2-bootkey-rehearsal.sh` is
1218 lines; call it **R** below). Facts about RP2350 row addresses / bit positions
for BOOT_FLAGS0 are NOT in this repo anywhere (grep for `BOOT_FLAGS0`, `DISABLE_OTP_BOOT`,
`ENABLE_OTP_BOOT`, `FLASH_DEVINFO`, `WHITE_LABEL` over scripts/, design/RUNBOOK*,
HARDWARE_INVENTORY, agent-reports, FIRMWARE-QUICKSTART returns nothing relevant); they
must be taken from `picotool otp list` / pico-sdk `otp_data.h`, not from memory.

Plan inputs read: `refugium-wallet/design/IMPLEMENTATION_PLAN_mr_gui_v1.md:343-393`
(E3a at 360-373), `SPEC_ui_refugium_wallet.md` §4.4 (263-420; "Expected OTP state"
~l.331-360, provisioning steps 1-6 ~l.368-398, Advanced hardening ~l.400-410).

## 0. Inventory of files

| file | role |
|---|---|
| `scripts/pico2-bootkey-rehearsal.sh` (R) | the one script: Pico phases 0-6 + three read-only SH2 modes + `--make-otp-json` |
| `scripts/test/fake-picotool` (85 lines) | stateful OTP simulator for the e2e harness |
| `scripts/test/run-e2e.sh` (251 lines) | the only test harness for R (offline of hardware, not offline of tools) |
| `scripts/sh2-flash` (345 lines) | build+sign+flash; contains NO OTP reads/writes (only `picotool version`, `info -a <file>`, `load --verify`, `reboot`) |
| `scripts/sign-firmware.sh` | signing; picotool use: `seal --sign --clear` (:104), `info -a` (:85, :173) |
| `scripts/rehearsal-blinky/{main.go,go.mod}` | TinyGo LED image sealed/flashed by phases |
| `design/RUNBOOK_custom_boot_key.md` (590 lines) | operator procedure; mirrors R's SH2 modes |
| `design/HARDWARE_INVENTORY.md` (163 lines) | board table, per-board retail state |
| `.github/workflows/release.yml` | only workflow; no shellcheck, no bats, no run-e2e (see §4) |
| `FIRMWARE-QUICKSTART.txt` | operator card, section 0a mentions precheck (not read in full) |

There is no `flake.nix` in this repo: picotool comes from `nix develop` in the
SEPARATE seedhammer fork (R:129, R:3-60 header, runbook l.16, l.70-77: picotool
2.2.0-a4, TinyGo 0.41.1). The picotool pin that E3a "owns" therefore lives outside
this repo; E3a here can only assert/record the version, not set it.

## 1. R: modes, flags, step order

### 1.1 Argument parsing (R:107-126)
`--phase N` (R:110), `--execute` (R:111), `--sh2-precheck` (R:112),
`--sh2-verify-slot N` (R:113), `--sh2-verify-valid N` (R:114), `--make-otp-json` (R:115),
`--key` (R:116), `--slot` (R:117), `--out` (R:118); anything else dies (R:119).
Env: `WORKDIR` (R:67, default `$REPO_ROOT/rehearsal-work`), `SH2_DIR` (R:73, default
`$REPO_ROOT/sh2-state`), `SEEDHAMMER_DIR` (R:74), `ALLOW_UNCHECKED_KEY` (R:635),
`ACCEPT_BLINKY_ONLY` (R:1135). No profile flag exists. `set -euo pipefail` (R:62).

Tool preflight: picotool/openssl/sha256sum always (R:128-130); tinygo+go for phases
0,3,4,5,6 (R:132-136); tinygo+jq for `--make-otp-json` (R:140-144); `SEEDHAMMER_DIR`
must exist (R:145-146). Banner: read-only mode vs dry-run (R:151-155).

### 1.2 Modes (dispatch `case "$MODE"` R:681-889; `case "$PHASE"` R:891-1218)
- `--make-otp-json` (R:682-693): no device. `require_spare_slot`, `reject_rehearsal_key`, `make_otp_json`.
- `--sh2-precheck` (R:695-772), order: `sh2_require_seedhammer` (R:698) -> SECURE_BOOT_ENABLE==1
  (R:699-702) -> KEY_VALID==1 exactly (R:704-714) -> KEY_INVALID==0 (R:716-718) -> slots 1-3 all
  zero (R:720-732) -> `check_page_locks` (R:734) -> CRIT1 x8 raw compare (R:746-756) ->
  BOOT_FLAGS1 x3 raw compare (R:759-767) -> RESULT (R:769-772).
- `--sh2-verify-valid N --key` (R:774-859): `require_spare_slot`, `sh2_require_seedhammer`,
  `reject_rehearsal_key`, `verify_slot_or_die` (R:784), then KEY_VALID == `1|(1<<N)` with
  extra/missing-bit diagnosis (R:790-825), KEY_INVALID==0 (R:828-830), BOOT_FLAGS1 A/B/C
  copy compare (R:846-853), RESULT.
- `--sh2-verify-slot N --key` (R:861-885): refuses if slot already valid (R:871-878), then `verify_slot_or_die`.
- Phase 0 (R:893-930): `require_rp2350_bootsel`, print SB/KV, `assert_stock_or_die` (R:904), `check_page_locks` (R:905), pin CHIPID to `$WORKDIR/board-chipid.txt` (R:908-910), generate 3 keys factory/my/third-party (R:913-921), `build_blinky`.
- Phase 1 (R:932-973) DESTRUCTIVE: `require_board`; resume branch (R:942-952); `make_otp_json` slot 0 (R:956); `confirm BURN-FACTORY` + `otp load` (R:959-960); verify 16 rows (R:964); `confirm SET-VALID`; `otp set -s BOOT_FLAGS1.KEY_VALID 0x1` (R:969) and `otp set -s CRIT1.SECURE_BOOT_ENABLE 0x1` (R:970).
- Phase 2 (R:975-995): read-only sealed-state check; SB, KV==1, `verify_slot_or_die 0`, `check_page_locks`.
- Phase 3 (R:997-1056): negative control; flash my-key-signed + unsigned images; `bootsel_present` + `ask_blink`.
- Phase 4 (R:1058-1090) DESTRUCTIVE: KV==1 precondition (R:1064-1065), `check_page_locks`, `make_otp_json` slot 1, `confirm BURN-MY-KEY`, `otp load` (R:1074), `verify_slot_or_die 1` (R:1077), `confirm SET-VALID-SLOT1`, `otp set -s BOOT_FLAGS1.KEY_VALID 0x2` (R:1082), read back ==3 (R:1084-1086).
- Phase 5 (R:1092-1179): positive control + 5b real firmware (R:1134-1177).
- Phase 6 (R:1181-1213): fallback control.

Dry-run is the default; `--execute` arms writes (R:94-97 `run()`, R:99-105 `confirm()`); SH2 modes have no armed variant (R:149-152) and contain no write path (R:583-585).

## 2. Every picotool call and how its output is parsed

Parsers are all fail-closed (R:159-164).

| call | where | parse |
|---|---|---|
| `picotool otp get -n <field-selector>` | `otp_field` R:174-193 (only for `CRIT1.SECURE_BOOT_ENABLE`, `BOOT_FLAGS1.KEY_VALID`, `BOOT_FLAGS1.KEY_INVALID`) | dies on any output containing `warning` (R:184, pure-bash glob per F-695), takes the LAST `field ...` line, strips to the text after `=`, lowercase bare hex (R:187-191). This is the ECC/interpreted, majority-voted read. |
| `picotool otp get -n -e <rows...>` (one call for all rows) | `read_rows` R:208-240 (CHIPID0-3, BOOTKEY{n}_{0..15}) | warning trap R:212-214; extracts every `VALUE 0x...` line, masks to 16 bits (R:215-217); asserts count == requested (R:218-220); asserts the echoed `ROW 0x....: OTP_DATA_<NAME>` names equal the request order (R:226-239) because picotool sorts and dedups. NOTE the name regex needs `OTP_DATA_` + name, so only NAMED rows can be fed to `read_rows`; a numeric selector (`0x048`) would fail the name check. |
| `picotool otp get -n <row>` (no `-e`, single row) | `read_row_raw24` R:365-385 | warning trap R:377-379 (added by F-619); last `VALUE 0x...` line, mask 24 bits, printed `%06x` (R:380-384). Used for PAGE{1,2}_LOCK{0,1} (R:402-403) and numeric rows `0x040-0x047`, `0x04b-0x04d` (R:746, 759, 846). |
| `picotool otp load <json>` | R:960, R:1074 (via `run`) | not parsed (exit status only; the operator-facing echo is documented as pre-write input in runbook l.324-338) |
| `picotool otp set -s FIELD VALUE` | R:969, 970, 1082 | exit status only; then `otp_field` read-back (R:1084). The `--sh2-verify-valid` messages print `otp set -s` recipes (R:814, 822). |
| `picotool seal --sign --quiet <uf2> <out> <key> <json>` | `make_otp_json` R:462 | JSON file via jq (R:465-478) |
| `picotool info` (device) | `bootsel_present` R:555 | exit status only, 15 x 1s poll |
| `picotool info -a <file>` | R:1104, R:1110 | `public key:` line (R:1105), `signature: *verified` (R:1111) |
| `picotool load --verify` / `reboot` | `flash_image` R:509, 512 | exit status |
| `picotool erase`, `save`, `verify` | **not used anywhere in R or sh2-flash** | n/a (see §7) |

No call passes `--ser` (board binding for writes is documented only as an operator rule, runbook l.286-292; R binds by CHIPID read before writing, R:326-343). R never passes `-r` or `-c`; `-c 1` was measured to be a no-op (FOLLOWUPS F-619, ~l.18846).

## 3. How state is verified today

- Field-level, ECC/majority reads: SECURE_BOOT_ENABLE, KEY_VALID, KEY_INVALID via `otp_field` (R:174). One interpreted read per field, so the 3 BOOT_FLAGS1 copies are NOT seen individually here. The sole protection against a degraded redundant row is picotool's `WARNING` string (F-619; comment R:834-845).
- Slots: 16 rows each through `read_rows -e` (R:242-253), byte-swapped (R:251), compared to the openssl hash (R:482-493).
- Raw (no `-e`), full 24 bits: page locks (R:387-440; decoded with a bit-wise majority, disagreement is only a `warn` R:411-412), CRIT1 rows 0x040-0x047 (8 copies, R:746-756, equality enforced), BOOT_FLAGS1 rows 0x04b/0x04c/0x04d (3 copies, equality enforced only in `--sh2-verify-valid`, R:846-853, and in `--sh2-precheck`, R:759-767). Known weakness: 0x04b resolves to the NAMED row and is majority-voted by picotool while 0x04c/0x04d print bare (R:373-376, R:834-845), so the copy compare is blind when 0x04b is the odd one out; the warning trap is the real guarantee.
- Rows NOT read anywhere today: BOOT_FLAGS0 (any copy), FLASH_DEVINFO, USB_BOOT_FLAGS, USB_WHITE_LABEL_ADDR, white-label structure, ENABLE_OTP_BOOT, DEBUG_DISABLE.
- Identity: CHIPID0-3 via `read_rows` (R:308-324) re-assembled word-reversed (runbook l.142-145 explains the spelling); all-zero CHIPID is refused (R:316-321). Pico phases pin to `$WORKDIR/board-chipid.txt` (R:326-343); SH2 modes pin to `$SH2_DIR/sh2-chipid.txt` created ONLY by `--sh2-precheck` (R:597-628). Slot-0 tripwire: Pico phases require slot 0 != SeedHammer hash (R:338-342); SH2 modes require slot 0 == `SH_SIGNKEY_HASH` (R:81, R:601).
- Page locks are interpreted as: LOCK1 rows: LOCK_S/LOCK_BL must be 0, LOCK_NS informational (R:415-423); LOCK0 rows: KEY_R/KEY_W must be 0, NO_KEY_STATE informational (R:425-436). Retail values per inventory: `PAGE1/2_LOCK0 = 0x000000`, `PAGE1/2_LOCK1 = 0x040404` (HARDWARE_INVENTORY l.88).

## 4. How it is tested today; can it run without hardware?

- Harness: `scripts/test/run-e2e.sh` drives R phases 0-6 and all SH2 modes against `scripts/test/fake-picotool` (symlinked as `picotool` at run-e2e.sh:45-47, state file `$OTPSTATE`, run-e2e.sh:85/118/131/212). Sections: A phases 0-6 happy path (:84-109), B refusals (:112-127), C SH2 modes on a simulated retail unit (:130-153), D verify-valid (:157-218, includes DEGRADE_BOOT_FLAGS1 :172-176 and DEGRADE_CRIT1 :212-218), E key/curve guards (:220-245). Assertion helpers `run_phase` (:54-76, exit code + optional `EXPECT_OUT` regex) and `state_is` (:78-81). Result summary :247-251. Runbook cites it as "39 offline checks" (runbook l.157) and the final-gate report as 28/28; the current count is not recorded in-repo (not run here).
- Hardware-free? Only partly. The fake handles OTP reads/writes, `load`, `reboot`, `info` (device). But `seal` and `info <file>.uf2` delegate to the REAL picotool (fake-picotool:23, 26; run-e2e.sh:37), and the e2e needs `tinygo` for the blinky (run-e2e.sh:36; R:132-136). Neither picotool nor tinygo is installed in this sandbox (`which` returned only jq and openssl), so the harness cannot run here; it is meant to run via `nix develop` in the fork (run-e2e.sh:19-21).
- Not in CI: `.github/workflows/release.yml` has no shellcheck, no bats, no e2e step (grep for shellcheck|bats|e2e returned nothing; `scripts/lint-gate.sh` also has no match). `R` carries `# shellcheck disable=SC2086` (R:248) but no one runs shellcheck on it in CI. No bats tests. `scripts/followups-status.sh` gates FOLLOWUPS.md only.
- fake-picotool limitations that bear directly on E3a (fake-picotool line refs):
  - State is just SB, KV, KI(never settable), SLOT{n}, BOOTED, PENDING_OK (:16, :48-49). `otp set` handles only `BOOT_FLAGS1.KEY_VALID` and `CRIT1.SECURE_BOOT_ENABLE` (:47-50); any other field is silently accepted as a no-op ("OTP set" printed).
  - `BOOT_FLAGS1.KEY_INVALID` is hard-wired to 0 (:65), so KEY_INVALID 0xC cannot be simulated.
  - Row emitter prints row address `0` and name only (`emit_row`, :19); numeric selectors served: 0x04b-0x04d (KV, with DEGRADE_BOOT_FLAGS1 making ONLY 0x04d differ, :70-73), 0x040-0x047 (SB, DEGRADE_CRIT1 only 0x047, :74-77). Everything else falls through to `emit_row 0 "$sel"` (:85). `PAGE*` always returns 0 (:78) so the retail 0x040404 pattern is never exercised.
  - No `BOOT_FLAGS0`, `FLASH_DEVINFO*`, white-label, `erase`, `save`, `verify` handling: `*) exit 0` at :34 swallows every unknown subcommand silently, which would turn an erase/save test into a vacuous pass unless the fake is extended to fail loudly.
  - No `field ...` line for BOOT_FLAGS0 fields; `otp_field` would die "could not parse a field value".
  - Does not model picotool's WARNING output at all; degradation is modelled by differing raw VALUEs, which exercises R:846-853 but not the R:184/212/377 warning traps (those were mutation-tested by hand per F-619, not in the harness).
- Past hand tests not in the harness: F-619 stubbed-warning mutation (FOLLOWUPS ~l.18852).

## 5. Expected-state constants and where they live

| constant | value | location |
|---|---|---|
| SeedHammer slot-0 hash `SH_SIGNKEY_HASH` | `c8314536d6af61ac2e62e5991e3e4711629c54696ba8c4af08965a1d319a473b` | R:81 (also run-e2e.sh:28 duplicate; runbook l.36-37, fork `platform_sh2.go:70`) |
| fork slot-1 key fingerprint `SH2_BOOTKEY_FP` | `846aa289f2f317e55ff03f90555132302842cff2f68ee45712834a25d64cabb4` | `scripts/sh2-flash:142` (NOT in R; HARDWARE_INVENTORY l.44) |
| KEY_VALID retail | `0x1` (slot 0 only) | R:705 (precheck), R:984, R:1065 |
| KEY_VALID after fork key | `1 | (1<<slot)` -> 3/5/9 | R:790, R:1085, R:1098, R:1195 |
| KEY_INVALID | must be 0 | R:717, R:829 (**hard `-eq 0`; this is the E3a "0xC accepted" change**) |
| SECURE_BOOT_ENABLE | bit0 must be 1 on SH2 / sealed Pico | R:700, R:982, R:1191 |
| slot-empty | 64 zeros | R:355, R:721 |
| page-lock decoding and thresholds | LOCK_S/LOCK_BL==0, KEY_R/KEY_W==0 | R:415-436 (no recorded-value comparison; retail 0x040404 / 0 only in prose R:396-400, runbook l.207-217, inventory l.88) |
| CRIT1 copy rows | `0x040..0x047` (8) | R:746 |
| BOOT_FLAGS1 copy rows | `0x04b 0x04c 0x04d` (3) | R:759, R:846 |
| slot numbering / spare slots | slots 1-3 spare; slot 0 forbidden | `require_spare_slot` R:588-595 |
| picotool field names | `CRIT1.SECURE_BOOT_ENABLE`, `BOOT_FLAGS1.KEY_VALID` (bits 0-3), `BOOT_FLAGS1.KEY_INVALID` (bits 8-11), `BOOTKEY0_0` row 0x0080, 16 rows/key | runbook l.554-561; R:969-970, R:1082 |
| retail per-board white-label values | **not recorded anywhere in this repo** (HARDWARE_INVENTORY has no white-label rows; the SH2-vs-Pico report places `USB_BOOT_FLAGS`/`WHITE_LABEL_ADDR` "around 0x059-0x05c" and strings at >= 0x0c0, an approximation, from `sh2-final-gate-lens-sh2-vs-pico-delta.md`) | E3a/H0 must record them |
| boards and chipids | SH2 #1 `0x77c483b745abf55c`, #2 `0x09f50bf63e8d6f46`, #3 `0xdb2010f935ed25b8`, rehearsal Pico 2 `0x66d3d60ff20abf2f` (rehearsal result note says `bf2ff20ad60f66d3`, word-reversed spelling), Pico 2 W `0xb3d19289d3ec3f0e` | HARDWARE_INVENTORY l.25-29 |
| rehearsal profile facts | Pico 2 is 4 MB (inventory l.28, l.114-118); 0x10E00000 aliases on it | inventory |

## 6. Open items, known gaps, still-OPEN findings bearing on E3a

FOLLOWUPS.md (status gate `scripts/followups-status.sh` passes: "OK", 28 uncited closures pinned):
- **F-701 `mr-gui-e3-otp-tooling`** (~l.20395-20412), Status OPEN: the E3a/E3b entry itself. Done-when: whole sequence on 4 MB Pico 2 under rehearsal profile including an injected unequal copy the check refuses.
- **F-143** (~l.4856) Status OPEN: `sh2-flash` compares the key to a RECORDED fingerprint (`SH2_BOOTKEY_FP`, sh2-flash:142), not live OTP; the stronger form (read BOOTKEY1_0..15 from the attached unit) is available but unbuilt. Relevant because E3a's rehearsal profile and any slot-1 key model (E4) change what "the expected fingerprint" is.
- **F-65** (~l.933) Status OPEN: no backup of `~/.sh2/sh2-boot-key.pem` (one copy only); related `seedhammer-bootkey-24word-backup` (~l.94). Bears on any plan that adds irreversible steps.
- `bootkey-rehearsal-fidelity-residue` (FOLLOWUPS l.128): (c) resolved; **(b) host-vs-on-device sealing remains open** (rehearsal seals via picotool JSON, retail SH2 was sealed on-device and also populated white-label strings and `USB_BOOT_FLAGS`, so a Pico never has a populated user area) and **(e) Pico 2 W undetectable by phase 0** remains open (doc-only). (b) is exactly the gap the plan's "rehearsal profile, stating the rows it cannot prove" must name.
- Known-weak spot (closed but structural, F-619 ~l.18844-18858): copy equality via numeric rows is blind when the NAMED row (0x04b) is the odd one; `-c 1` is a measured no-op so there is NO picotool flag that reads one physical copy independently. E3a's "raw reads of all three copies of each boot-flag row" inherits this: for BOOT_FLAGS0 the same asymmetry should be expected (first row named, siblings bare); the injected-unequal-copy test must therefore corrupt a copy that is NOT the named one AND a case where the named one is the odd one, and the plan should say what it will do about the latter (rely on the warning trap, as F-619 concluded).
- Review findings from the agent-reports that were noted as not folded / residual:
  - `pico2-bootkey-rehearsal-fable-final-gate-round3.md` M4: Pico phase 1 interrupted between `KEY_VALID` and `SECURE_BOOT_ENABLE` writes (R:969-970) strands the board ("FRESH board" at R:349/351); resume branch (R:942-952) only handles KV==0. Not marked fixed in what I read.
  - `sh2-final-gate-lens-sh2-only-code.md` M3: a half-programmed ECC row yields a tool-shaped "warning" die (R:212-214) instead of "slot is spent" guidance; M4: `sign-firmware.sh:87` uses `xxd` without preflight.
  - `sh2-final-gate-lens-sh2-vs-pico-delta.md`: SH2 PD front end / `error -71` makes the one-shot `otp load` window more interruption-prone; runbook l.317-323 recommends a non-PD 5 V source.
  - `new-board-otp-burn-delta-review.md` I-1 / journey-review I-2: irreversible commands need `--ser`; this is runbook-only (l.286-292, 304) and **R has no `--ser` support or check**, which matters for any E3a write of DISABLE_OTP_BOOT (SPEC §4.4 "Every OTP write is bound to [the CHIPID]").
  - `bootkey-round5-lens-failure-injection.md` M3: `bootsel_present` fail-open on slow enumeration (addressed by polling, R:549-562).
  - `preflash-fable-*` reports concern the flash path/Go/Rust, not OTP; not read in depth.
- Runbook says "`DEBUG_DISABLE` ... never written" (l.48-50) and Step 7 forbids revoking slot 0 (l.514-522): **KEY_INVALID 0xC revokes slots 2 and 3 only (bits 2,3)**, which does not conflict with Step 7 but the runbook has no text for it, and the runbook deliberately omits the KEY_INVALID command (l.516-519; round-3 M3). Adding it back needs a deliberate design choice (SPEC §4.4 says KEY_INVALID and DISABLE_OTP_BOOT "are in no runbook yet").
- Existing precheck vs SPEC precheck deltas (R:695-772 vs SPEC step 1): R requires KEY_VALID == 1 exactly and KEY_INVALID == 0 and slots 1-3 empty; the SPEC allows slot 1 empty OR holding the fork key, KEY_VALID in {1, 3}, requires `ENABLE_OTP_BOOT` clear and `KEY_INVALID` clear for slots 0 and 1 only (not whole-field), plus FLASH_DEVINFO and white-label rows. `--sh2-precheck` today REFUSES an SH2 that is already provisioned (R:704-713 message points to `--sh2-verify-valid`); boards #1-#3 are in that state, so the E3b precheck cannot be the current `--sh2-precheck` unchanged.

## 7. Concrete extension points for each E3a item

(Edits in R unless stated. `read_rows` cannot take numeric selectors because of its name check at R:227-239; E3a needs either a numeric-capable reader or `read_row_raw24`.)

1. **DISABLE_OTP_BOOT write + read-back in all three BOOT_FLAGS0 copies.**
   R has no write path in SH2 modes by design (R:58-60, R:583-585) and its only OTP-write modes are Pico phases. Options:
   - Pico side: new phase 7 (after R:1213, before the `*)` at R:1215, and widen the `die` string R:122 and R:1216 "0..6") modelled on phase 4 (R:1058-1090): `require_board`, preconditions, `confirm DISABLE-OTP-BOOT`, `run picotool otp set -s BOOT_FLAGS0.DISABLE_OTP_BOOT 0x1`, then read-back via a new `verify_boot_flags_copies` helper (three raw reads + field read; die on warning or inequality). Field name must be confirmed with `picotool otp list` on the pinned picotool (runbook l.554-561 shows how the existing names were verified).
   - SH2 side: either keep read-only and add a new `--sh2-verify-disable-otp-boot` mode (parallel to `--sh2-verify-valid`, R:774-859), leaving the write in the runbook; or introduce the first SH2 write mode. The current contract (R:58-60) says SH2 modes contain no write path, so a write mode is a policy change, not just an edit; the plan's E3b requires "Brian's typed go-ahead naming the board's CHIPID" so a `--ser`-bound, `confirm`-gated write would be needed. New helpers: `bound_picotool()` adding `--ser <CHIPID uppercase picotool spelling>` (note the two CHIPID spellings, runbook l.142-145, R:308-324 yields word-reversed).
   - Runbook: new step between Step 3 and Step 4 (or after Step 6) plus Step 7 text.
2. **KEY_INVALID 0xC accepted in the rehearsal script and `--sh2-verify-valid`.**
   - `--sh2-verify-valid`: R:828-830 (`-eq 0` -> accept `0` or mask `0xC`; recommended: derive an expected value from a new `--expect-key-invalid {0|c}` arg or from the profile, rather than accepting both silently, so a wrong 0xC cannot pass). Mirror in `--sh2-precheck` R:716-718 (precheck must still require "clear for slots 0 and 1", bits 0-1, SPEC step 1) and add KEY_VALID/KEY_INVALID cross-check (a valid bit whose invalid bit is set is ignored by the bootrom; datasheet text quoted in `bootkey-round5-lens-external-facts.md` item 4). Note KEY_VALID checks at R:790 compute `WANT` assuming slots 0 and N only, which stays correct.
   - Pico phases: phase 4 (R:1084-1086) and phase 6/5 preconditions assume KV; add a new phase (e.g. 8) that sets `BOOT_FLAGS1.KEY_INVALID 0xC` on slots 2/3 of the Pico (those slots are empty on the Pico: R:353-357 confirms), then reads back all three BOOT_FLAGS1 copies. Rehearse "slot 2/3 key burned later cannot be made valid" if wanted.
   - `fake-picotool:65` must become stateful (new `KI` state is already initialised at :16 but never settable; `set` case at :47-50 needs a KEY_INVALID arm and `get` at :65 must read `$(get KI)`; field line must show `bits 8-11`).
3. **Precheck (page locks, ENABLE_OTP_BOOT clear, FLASH_DEVINFO_ENABLE and FLASH_DEVINFO, white-label rows vs recorded retail values).**
   - Page locks: reuse `check_page_locks` (R:387-440); the SPEC wants "values the plan fixes from the runbook (retail LOCK1 0x040404, LOCK0 0)" i.e. an equality comparison to recorded values, which R does not do today (R:411-412 only warns on copy disagreement; threshold-only decoding). Add exact-value assertion behind the profile (retail profile: `0x040404`/`0x000000`; keep the current threshold logic for rehearsal profile).
   - ENABLE_OTP_BOOT / FLASH_DEVINFO_ENABLE / FLASH_DEVINFO: new reads in a new function (e.g. `check_boot_flags0`) called from `sh2-precheck` after R:734; use `otp_field BOOT_FLAGS0.ENABLE_OTP_BOOT` (R:174, field-level) and `read_row_raw24` for FLASH_DEVINFO.
   - White-label rows: new constants block near R:78-81 (or a separate `scripts/sh2-retail-otp.conf`/data file) keyed by board revision; the retail values do not exist in the repo and must first be captured by a raw read of a retail unit (SPEC says "added to mnemonic-engrave's hardware inventory before this check ships") -> HARDWARE_INVENTORY.md needs a new per-revision table. Row addresses must come from `picotool otp list` (the delta review's "around 0x059-0x05c / >=0x0c0" is approximate). White-label strings live in the ECC/user area with an address pointer (USB_WHITE_LABEL_ADDR), so reading them requires a pointer-following helper; `read_rows` (names only) won't do and `read_row_raw24` is single-row. Expect a new multi-row reader.
   - "Any OTP row the check cannot read counts as a mismatch": R's readers already `die` on read failure (R:176, 210, 382); keep that but add the explicit page-2-read-lock wording.
   - SPEC precheck also allows slot 1 empty OR fork key, KEY_VALID in {1,3}: current `--sh2-precheck` refuses an already-provisioned unit (R:704-713, R:720-731), so either relax by flag or add a separate `--sh2-precheck-provisioned`.
4. **Raw reads of all three copies of each boot-flag row.**
   Existing pattern to copy: R:758-767 and R:846-853 (BOOT_FLAGS1). Factor into `read_copies <label> <row...>` returning an array and dying on any inequality or warning, then call for BOOT_FLAGS0 (rows to be confirmed with `picotool otp list`; do not assume), BOOT_FLAGS1 (0x04b-0x04d), and optionally CRIT1 (0x040-0x047). Mind F-619: first row is named so the vote is not independent of copies 2/3; keep the WARNING trap as the primary guard (R:377-379). Today's code is duplicated twice (R:758-767, R:846-853) plus CRIT1 (R:745-756); a shared helper would also let the profile and tests drive it.
5. **Explicit-range erase and save.**
   Nothing exists in R, `sh2-flash`, or sign-firmware. Closest prior art: `picotool erase -r 0x10E00000 0x10E10000` wipe and the "`picotool save` before and after" readback experiment recorded in FOLLOWUPS ~l.11340-11400 (also `CONSULT-P0-row4-f259-refusal.md:31,45`), and the hint that pipes/`/dev/stdin` silently read nothing (FOLLOWUPS ~l.11360-11395: picotool sizes input with `fstat`). A 4 MB Pico cannot erase a 16 MB range (the SPEC says a range the boot ROM refuses counts as an unknown image), so the rehearsal profile cannot prove the 16 MB behaviour and must say so. New helpers `erase_flash_range <start> <end>` and `save_flash_range <start> <end> <file>` with: explicit `-r` start/end (never whole-chip), post-check via `picotool verify` or hash of `save` output, exit-code capture without pipes (F-695 rule, comment at R:109, R:188). They need new fake-picotool arms (it must fail loudly on unknown subcommands rather than `exit 0`, fake-picotool:34). Also a possible new standalone mode `--sh2-erase-range`/`--sh2-save-range` in R, or a new script; given the plan places these in the Sitting image's flow (SPEC provisioning step 2), putting them in their own small script `scripts/flash-range.sh` (new file) keeps R's "no write path in SH2 modes" invariant (R:58-60).
6. **Rehearsal profile.**
   No concept exists; Pico phases are implicitly "rehearsal" (slot 0 holds `factory-key` from phase 1, R:942-970; the tripwire R:338-342 forbids the SH hash) and SH2 modes are implicitly "retail" (R:601). The cleanest edit: a `--profile {retail|rehearsal}` flag next to R:115-118 (default retail), with:
   - `sh2_require_seedhammer` (R:597-628) accepting `slot0 == factory-key hash from $WORKDIR/factory-key.pem` under rehearsal (inverse of the tripwire; keep the CHIPID pin; keep the Pico out of the real-SH2 path: under rehearsal REQUIRE slot0 != `SH_SIGNKEY_HASH`),
   - white-label comparison skipped under rehearsal AND printed as an explicit "CANNOT PROVE" list (slot 0 identity as SeedHammer's, white-label values, flash map above 4 MB; plus, per FOLLOWUPS residue (b), host-vs-on-device sealing), emitted in RESULT (R:769-772, 855-859) so a green run cannot be mistaken for the retail check,
   - a `PROFILE` string printed in the banner (R:151-155) and in every PASS RESULT.
   The e2e harness has the Pico-as-SH2 case only through phases; adding rehearsal-profile SH2 modes requires fake state with `SLOT0=<factory hash>`.
7. **Test: injected unequal copy refused.**
   Pattern exists: `DEGRADE_BOOT_FLAGS1=1` (fake-picotool:72, run-e2e.sh:172-176 with `EXPECT_OUT="redundant copies DISAGREE"`) and `DEGRADE_CRIT1=1` (fake-picotool:76, run-e2e.sh:212-218). For E3a add `DEGRADE_BOOT_FLAGS0=<copy-index>` in fake-picotool (so each of the 3 copies, including the NAMED first one, can be the odd one) and run_phase cases that expect refusal for each index; plus a WARNING-injection variant (`DEGRADE_WARN=1` emitting `(WARNING - REDUNDANT ROWS AREN'T EQUAL)` with identical VALUEs) so the traps at R:184/212/377 are covered in the harness for the first time (today they are tested only by the hand mutation in F-619). The fake also needs `BOOT_FLAGS0` state (DISABLE_OTP_BOOT), KEY_INVALID state, FLASH_DEVINFO and white-label rows, nonzero PAGE lock values (0x040404) and `erase`/`save` handlers.
   Hardware-side "done when" (4 MB Pico 2 under rehearsal profile) needs a real injected unequal copy: on silicon an unequal copy can only be produced by programming one raw copy row (OTP bits cannot be cleared), i.e. a deliberate partial write to a rehearsal Pico, which needs its own phase or a runbook note; the fake cannot prove picotool's real warning text.

### What needs a NEW file (vs an edit)

- Edits: `scripts/pico2-bootkey-rehearsal.sh` (profile flag, new constants, helpers, precheck, KEY_INVALID, DISABLE_OTP_BOOT phase/mode), `scripts/test/fake-picotool`, `scripts/test/run-e2e.sh`, `design/RUNBOOK_custom_boot_key.md` (new steps + Step 7 text), `design/HARDWARE_INVENTORY.md` (retail white-label / FLASH_DEVINFO / BOOT_FLAGS0 values per board revision, which must be measured from a retail unit first), `design/FOLLOWUPS.md` (F-701 status when closed; `scripts/followups-status.sh` gate).
- New files likely needed: (a) a data file or section for per-revision retail OTP values (could be a block in R, but the SPEC wants a record "in mnemonic-engrave's hardware inventory" -> inventory table plus a machine-readable copy for the script); (b) a flash-range erase/save helper or modes (see 5); (c) a pin record for picotool version (e.g. `design/PICOTOOL_PIN.md` or a check in R: `picotool version` compare), since there is no flake in this repo; the fork's `flake.nix` (outside this repo) is where the pin actually lands; (d) a CI hook if the e2e is to gate: nothing in `.github/workflows/release.yml` runs it, and CI has no picotool/tinygo, so either extend fake-picotool to also fake `seal`/`info -a` (removing the dependency on the real binary), or add a nix-based job; this is a decision for the plan, not an edit. (e) New hardware-result note for the Pico rehearsal (convention: `design/REHEARSAL_RESULT_2026-08-03.md` is cited by runbook l.541 but I did not confirm the file exists in this repo; `ls design` listing shown was truncated at 150 entries).

### Gotchas worth carrying into the plan

- `read_rows` name regex (R:227) rejects numeric selectors; BOOT_FLAGS0 copy rows need `read_row_raw24`.
- Field-level reads are majority-voted; only raw numeric reads expose copies, and the first row is named (F-619). A picotool upgrade (2.3.1 vs 2.2.0-a4) can change which row is named, the `ROW`/`field` line formats (R:166-171 is derived from 2.2.0 `main.cpp`), the `-e` semantics, `otp set` copy behaviour and erase semantics; all parsers must be re-measured on the pinned version (R:166 comment documents the source of the format).
- R's banner and docs say SH2 modes are read-only (R:48-60, R:149-152, runbook l.199-200 "contains no otp load, no otp set"); any write mode contradicts those statements in at least five places (R:48, 58-60, 149-152, 583-585; runbook l.199).
- `die` inside `$(...)` exits only the subshell (R:204-206, R:521-523): new helpers must set globals (as `read_rows`/`read_slot` do) or use plain assignments, not nested substitutions in test arguments (R:659-662).
- Pipelines into `grep -q` under `pipefail` are banned (F-695; R:181-183, R:1109).
- The two CHIPID spellings (word-reversed R form vs picotool/`--ser` uppercase form) must not be mixed (runbook l.142-145; inventory l.91-105).
