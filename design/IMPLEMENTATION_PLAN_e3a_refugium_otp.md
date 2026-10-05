# IMPLEMENTATION PLAN — E3a: OTP tooling for Refugium, the rehearsal profile, and one picotool (F-701)

*Draft 1, 2026-10-05. Author: thread "mnemonic-engrave for Refugium". Baseline: mnemonic-engrave
`fde7841` (master after PR 6). Risk set (a) irreversible OTP writes and (b) keys: R0 to 0 C / 0 I
before code, a single implementer, then a mandatory adversarial review of the whole diff.*

Source of the ask: `bg002h/refugium-wallet` `design/IMPLEMENTATION_PLAN_mr_gui_v1.md` at `f929084`,
phase E3a (and E3b, which this plan prepares but does not run); UI spec
`design/SPEC_ui_refugium_wallet.md` §4.4 ("Expected OTP state", "Engraver provisioning sitting").
Recon, persisted verbatim: `design/agent-reports/e3a-recon-otp-facts.md` (cited **OF**),
`e3a-recon-picotool.md` (**PT**), `e3a-recon-repo-map.md` (**RM**). The rehearsal script
`scripts/pico2-bootkey-rehearsal.sh` is **R** below.

Build-gate coverage: this plan carries no ```rust blocks, so `scripts/plan-build-gate.sh` covers
nothing here. Everything below is bash, a Python test fake and docs; the gates are the e2e suite
(§6) and the bench rehearsal (§7), and neither has run yet. They are hypotheses until §8's
pre-dispatch check runs the e2e skeleton once.

## 0. Outcome

1. **One picotool**: 2.3.1 built against pico-sdk 2.3.1, provided by a small `flake.nix` in this
   repo (`packages.picotool`, `devShells.otp`). The fork's `nix develop` and the Sitting image (S1)
   take this flake as an input, so the version is set in one place. The scripts refuse any other
   picotool version.
2. **A new tool, `scripts/refugium-otp.sh`**, for the Refugium expected OTP state (UI spec §4.4):
   a read-only `check` against a profile, a read-only `capture` of the retail rows for H0, and two
   write steps, `disable-otp-boot` and `invalidate-spare-keys` (`KEY_INVALID` 0xC), each bound to a
   CHIPID with `--ser`, dry-run by default, typed confirmation naming the CHIPID.
3. **A flash-range helper** inside the same tool: `erase-range` and `save-range` with an explicit
   range from the profile, never picotool's guessed whole-flash size, plus an all-0xFF check.
4. **Two profiles**: `retail` (SeedHammer's key in slot 0, white-label and `FLASH_DEVINFO` rows
   compared with recorded per-revision values, 16 MB range) and `rehearsal` (the rehearsal factory
   key in slot 0, no white-label comparison, 4 MB range), each printing what it cannot prove.
5. **`KEY_INVALID` 0xC accepted** by R's `--sh2-verify-valid` when asked for (`--expect-key-invalid`).
6. **A row-level picotool fake in Python** and an e2e suite that needs no hardware, no real
   picotool and no TinyGo, run in CI.
7. **A bench procedure** for the 4 MB Pico 2 rehearsal (no per-board go-ahead, plan §9 item 5),
   including two injected unequal copies the check must refuse, and an E3b runbook section for
   SeedHammer #1 that runs only on Brian's typed go-ahead naming the CHIPID and the step.

Not in E3a: running E3b; any write to a SeedHammer; the key model (E4); the Rust/GUI provisioning
flow (lane S, which reuses these facts and the pinned picotool); the white-label values themselves
(H0 measures them with `capture`).

## 1. Facts this plan depends on (all from source, OF/PT; re-checked by the reviewer)

| id | fact | source |
|---|---|---|
| F1 | BOOT_FLAGS0 rows 0x048/0x049/0x04a; BOOT_FLAGS1 0x04b/0x04c/0x04d; USB_BOOT_FLAGS 0x059/0x05a/0x05b; all raw 24-bit RBIT-3 | OF 1c, 1d, 1h |
| F2 | DISABLE_OTP_BOOT = BOOT_FLAGS0 bit 13; ENABLE_OTP_BOOT bit 14; FLASH_DEVINFO_ENABLE bit 5 | OF 2a-2c |
| F3 | KEY_VALID BOOT_FLAGS1 bits 3:0; KEY_INVALID bits 11:8; effective valid = KEY_VALID & ~KEY_INVALID; 0xC invalidates slots 2 and 3 only | OF 2e, 2f, 5 |
| F4 | The boot ROM reads BOOT_FLAGS0/1 and USB_BOOT_FLAGS by per-bit 2-of-3 vote; CRIT1 SECURE_BOOT_ENABLE is any-3-of-8 | OF 3, 3b |
| F5 | DISABLE_OTP_BOOT supersedes ENABLE_OTP_BOOT and only skips OTP boot; flash boot and BOOTSEL are untouched | OF 4 |
| F6 | FLASH_DEVINFO row 0x054 (ECC); CS0_SIZE bits 11:8, `4 KiB << n`, 0xc = 16 MiB; with ENABLE set and a smaller CS0, BOOTSEL flash ops beyond it are refused | OF 1g, 7a, 7b |
| F7 | USB_WHITE_LABEL_ADDR row 0x05c (ECC), the row index of a 16-row ECC table; USB_BOOT_FLAGS bit 22 enables it; bits 0-15 are per-entry valid bits | OF 1i, UNVERIFIED note |
| F8 | Page locks: PAGE1_LOCK0/1 0xf82/0xf83, PAGE2_LOCK0/1 0xf84/0xf85; retail LOCK1 0x040404, LOCK0 0; picotool writes are refused if LOCK_S or LOCK_BL is non-zero or KEY_W is set | OF 1j, 1k, 6a-6e |
| F9 | `otp get` on a named redundant register votes per bit; when copies differ it prints `RAW_VALUE=` with every copy and `(WARNING - REDUNDANT ROWS AREN'T EQUAL)` | OF P1 |
| F10 | `-c N` overrides the copy count; with `-c 1` the value is one row's raw content (source; not yet seen on hardware) | OF P2 |
| F11 | `otp set` reads copy 0 only and writes the new value to all copies in one command, row by row; a copy holding a bit the new value lacks fails that row after earlier rows are burned | OF P3 |
| F12 | picotool 2.3.1 vs 2.2.0-a4: every name used here is unchanged; ECC `VALUE` prints 16-bit (`0x%04x`); a lone CHIPID1 read works; `seal --sign` output changes (EXTRA_SECURITY, VECTOR_TABLE) | PT |
| F13 | `erase`/`save` without `-r` guess the flash size from contents and refuse erased flash; with `-r` they use the range given | PT item 4; plan §3a |

Facts the implementer must establish from picotool 2.3.1 source **before** writing the parser or
the fake, citing `main.cpp:<line>` in a comment at each parser and fake arm (none is gated by a
build):
- G1. The exact text of every `otp get` line the tool parses (ROW, VALUE, RAW_VALUE, field,
  WARNING), for ECC, RBIT-3, RBIT-8 and plain raw rows, with and without `-r`, `-e`, `-c 1`.
- G2. How a raw write of **one** physical row is expressed (`otp set` with a numeric row and
  `-r`, or `otp load` with a raw-row JSON entry), and that it writes only that row.
- G3. The text and exit status of an `erase -r` / `save -r` that the boot ROM refuses.
- G4. How `--ser` is matched (uppercase CHIPID, PT item 5) and its exit status on a miss
  (HARDWARE_INVENTORY records 249 on 2.2.0-a4; confirm on 2.3.1).

## 2. The picotool pin (one place)

- New `flake.nix` + `flake.lock` at the repo root, nixpkgs **nixos-unstable** pinned by the lock
  (it carries picotool 2.3.1 and pico-sdk 2.3.1, PT §nixpkgs). Outputs:
  `packages.<system>.picotool` (unstable's picotool, with the mbedtls substitution PT names so
  `seal` and signature checks in `info` work, and the udev rule installed) and
  `devShells.<system>.otp` (picotool, openssl, jq, python3, bash, coreutils). Systems:
  `x86_64-linux`, `aarch64-linux`, `x86_64-darwin`, `aarch64-darwin`.
- `scripts/refugium-otp.sh` and R both call `require_pinned_picotool`: `picotool version` must
  print exactly 2.3.1 (format per G1). There is no override flag. R's existing bench users move to
  `nix develop .#otp` in this repo; the RUNBOOK's prerequisite section says so.
- `design/PICOTOOL_PIN.md`: the version, the nixpkgs rev, why 2.3.1 (F12), the one behaviour
  change that matters to signing (`seal --sign` output differs, so the fork's `picosign` is
  re-tested once, owned by the fork thread), and how the fork and the Sitting image consume the
  flake (`inputs.mnemonic-engrave.url = "github:bg002h/mnemonic-engrave?rev=…"`;
  `inputs.mnemonic-engrave.packages.${system}.picotool`). The fork thread is told the pin; its
  `nix develop` change is its own PR.
- R's format comment (R:166-171) and its `& 0xffff` masks are re-checked against G1; the CHIPID1
  workaround stays (harmless on 2.3.1).

## 3. `scripts/refugium-otp.sh`

A new script, not more modes in R: R's header promises its SH2 modes have no write path (R:48,
58-60, 149-152, 583-585; RM §7 gotchas), and that promise stays true for R. Shared code moves to
`scripts/lib/otp-read.sh` (sourced by both): `die/ok/info/hdr/warn`, `otp_field`, `read_rows`,
`read_row_raw24`, `read_slot`, `key_hash`, `chipid`, `check_page_locks`, `require_pinned_picotool`.
The move is mechanical and behaviour-preserving; R's existing e2e (sections A-E) is the check, run
under the new fake as well as the old (§6).

### 3.1 Command line

```
refugium-otp.sh check   --profile retail|rehearsal [--stage pre|post] [--expect-key-invalid 0|c]
                        [--rehearsal-key <factory-key.pem>] [--board-rev <id>]
refugium-otp.sh capture --out <file>                       # read-only dump for H0
refugium-otp.sh disable-otp-boot      --profile P --ser <CHIPID> [--execute]
refugium-otp.sh invalidate-spare-keys --profile P --ser <CHIPID> [--execute]
refugium-otp.sh erase-range --profile P --ser <CHIPID> [--execute]
refugium-otp.sh save-range  --profile P --ser <CHIPID> --out <file>
refugium-otp.sh inject-copy --profile rehearsal --ser <CHIPID> --row <0x049|0x04a|0x04b> --bits <hex> [--execute]
```

Rules for every command:
- `require_pinned_picotool`; exactly one RP2350 in BOOTSEL (`picotool info` lists one device);
  CHIPID read (CHIPID0-3) and spelled as picotool's `--ser` form (16 uppercase hex, CHIPID3..0).
- Every picotool call that touches a device passes `--ser <CHIPID>`, reads included, so a second
  board plugged in mid-run is never read or written.
- Write commands: `--ser` is required and must equal the CHIPID read; dry-run unless `--execute`;
  then a typed confirmation of the form `BURN <step> <CHIPID>`; a full `check --stage pre` must pass
  in the same run immediately before the write, and a full `check` of the expected post-state must
  pass immediately after. A failed post-check prints "do not use this board for seeds" and exits 3.
- Exit codes: 0 pass; 1 usage or tool error; 2 state refused (mismatch); 3 a write ran and its
  read-back failed. A row that cannot be read is a mismatch (exit 2), never a zero.
- Under `retail`, `inject-copy` is refused before any device access.

### 3.2 `check` — the expected state

Each row below is read and compared; the output lists every row with PASS/FAIL, then a RESULT
line naming the profile and stage. Nothing short-circuits after the identity rows, so one run
shows every mismatch (the GUI's "every refusal at once" rule).

| row(s) | how read | retail | rehearsal |
|---|---|---|---|
| CHIPID0-3 | one `read_rows` call | non-zero; equals `--ser` | same |
| CRIT1 ×8 (0x040-0x047) | raw, each copy | all 8 equal; SECURE_BOOT_ENABLE = 1 | same |
| slot 0 (BOOTKEY0_0..15) | `read_slot` | = `SH_SIGNKEY_HASH` | = hash of `--rehearsal-key`, and ≠ `SH_SIGNKEY_HASH` |
| slot 1 | `read_slot` | pre: empty or fork key; post: fork key (`846aa289…`, constant moved from `sh2-flash:142` into `otp-read.sh`) | pre: empty or the rehearsal "my-key"; post: my-key |
| slots 2, 3 | `read_slot` | all zero | all zero |
| BOOT_FLAGS1 ×3 | raw each copy + named vote + `-c 1` on 0x04b | copies equal; KEY_VALID 0x1 (pre, slot 1 empty) or 0x3; KEY_INVALID 0 or 0xC per `--expect-key-invalid` (default 0); KEY_INVALID bits 0-1 always 0 | same |
| BOOT_FLAGS0 ×3 | raw each copy + named vote + `-c 1` on 0x048 | copies equal; ENABLE_OTP_BOOT 0; DISABLE_OTP_BOOT 0 (pre) or 1 (post); every other bit equals the recorded retail value for `--board-rev` | copies equal; ENABLE_OTP_BOOT 0; DISABLE_OTP_BOOT per stage; other bits 0 (a stock Pico 2) |
| FLASH_DEVINFO (0x054) | ECC | equals recorded retail value; if FLASH_DEVINFO_ENABLE is set, CS0_SIZE must be 0xc | FLASH_DEVINFO_ENABLE 0 |
| USB_BOOT_FLAGS ×3, USB_WHITE_LABEL_ADDR, the 16-row table it points to | raw / ECC | copies equal; all equal the recorded retail values | **not compared**; listed under CANNOT PROVE |
| PAGE1/2_LOCK0/1 | raw | LOCK1 = 0x040404, LOCK0 = 0x000000 exactly | same values (a stock Pico 2 is expected to match; if it does not, the rehearsal stops and records it) |

Copy rule (F4, F9, F10): for each RBIT row the check requires (a) all raw copies equal, (b) the
named read printed no WARNING, (c) the named vote equals the expected value, and (d) `-c 1` on the
first row equals the first copy's raw read. (d) covers the case F-619 called blind, the first copy
being the odd one, and tests F10 on silicon (§7 step R6).

`--stage` defaults to `post` for `check`. `pre` is the state before the step being taken: the
write commands pass the right stage themselves.

CANNOT PROVE (printed, rehearsal only): slot 0 holds SeedHammer's key; the retail white-label and
USB_BOOT_FLAGS values; the flash map above 4 MB; on-device sealing as SeedHammer did it
(bootkey-rehearsal-fidelity-residue (b)). A rehearsal RESULT line reads
`RESULT: REHEARSAL PROFILE PASS — not a retail check`.

Retail values live in `design/hardware/retail-otp.json`, keyed by board revision (the string H0
reads from the white-label table), each entry citing the H0 capture file and date. With no entry
for the board's revision, a retail `check` refuses: "no recorded retail values for revision X;
run `capture` on a retail unit and record it (H0)". So a retail check cannot pass before H0.

### 3.3 Write steps

- `disable-otp-boot`: pre-check (`--stage pre`, DISABLE_OTP_BOOT 0 or already 1 with equal
  copies — an already-set board skips the write and goes to the post-check); `otp set -s
  BOOT_FLAGS0.DISABLE_OTP_BOOT 0x1 --ser`; post-check. If a copy holds a bit the new value lacks
  (F11) the pre-check has already refused it (copies unequal), so the partial-burn case never
  starts.
- `invalidate-spare-keys`: refused unless slots 2 and 3 are empty and KEY_VALID bits 2-3 are clear;
  `otp set -s BOOT_FLAGS1.KEY_INVALID 0xc --ser`; post-check with `--expect-key-invalid c`. The
  confirmation text says: "Slots 2 and 3 can never hold a key after this."
- Order on a real board follows UI spec §4.4 provisioning: precheck, erase, key (already done on
  #1-#3), image install, re-check, DISABLE_OTP_BOOT, optional KEY_INVALID, check. This tool does
  each step; it does not chain them (the Sitting image does, lane S).

### 3.4 Flash range

- Range from the profile: retail `0x10000000`-`0x11000000` (16 MB); rehearsal
  `0x10000000`-`0x10400000` (4 MB). Never `erase` or `save` without `-r` (F13).
- `erase-range`: `picotool erase -r <start> <end> --ser`; then `save-range` into a temp file and
  require its size equals the range and every byte is 0xFF (counted in bash with `tr -d '\377'`
  into a file and `stat`, not a pipe into `grep -q`, F-695). A refusal by the boot ROM (G3) prints
  "the boot ROM refused this range: treat the engraver as an unknown image (condemned)" and exits 2.
- `save-range`: same read, writes the file, prints its sha256.

### 3.5 `capture`

Read-only. Writes JSON with CHIPID, picotool version, and the raw value of every row in §3.2 plus
the full white-label table and the decoded revision string; refuses to overwrite `--out`. H0 runs
it on SeedHammer #1; its output becomes the `retail-otp.json` entry (§3.2) through a PR.

## 4. R changes

- Source `scripts/lib/otp-read.sh`; delete the moved definitions (mechanical).
- `--sh2-verify-valid` and `--sh2-precheck` take `--expect-key-invalid 0|c` (default 0); R:717 and
  R:829 compare with it. Bits 0-1 of KEY_INVALID must be 0 either way. No other behaviour change.
- R:753 comment fixed: CRIT1 SECURE_BOOT_ENABLE is any-3-of-8, not a majority (OF 3b).
- R:166-171 comment updated to 2.3.1's format (G1).
- `require_pinned_picotool` at start.

## 5. Docs

- `design/RUNBOOK_custom_boot_key.md`: prerequisites move to `nix develop .#otp`; a new section
  "Refugium steps (E3b)": `check --profile retail --stage pre`, `disable-otp-boot`, the optional
  `invalidate-spare-keys` (only if plan §9 item 14 allows it on a test board), and the
  expected-state check, each marked IRREVERSIBLE where it is, each needing Brian's typed go-ahead
  naming the CHIPID and the step. It states that no SeedHammer write happens before E4 is decided.
- `design/HARDWARE_INVENTORY.md`: a "Retail OTP rows" table per board revision (empty until H0),
  pointing at `design/hardware/retail-otp.json`.
- `design/FOLLOWUPS.md`: correct F-619's "`-c 1` is a no-op" with OF P2 (status line unchanged
  until §7 R6 settles it on silicon); F-701 progress.

## 6. Tests (no hardware, no real picotool, no TinyGo)

- `scripts/test/fake_picotool.py`: a row-level RP2350 simulator. State is a JSON file of raw rows
  (24-bit values, ECC rows stored as 16-bit data) plus a flash image file and a device list.
  It implements, with the exact text from G1-G4: `version`, `info` (device list, CHIPID serial),
  `otp get` (named fields with vote, RAW_VALUE and WARNING; numeric rows; `-r`, `-e`, `-c`),
  `otp set` (all copies, row by row, with F11's failure after earlier rows are burned), raw
  single-row writes per G2, `otp load` (the JSON R writes), `erase -r`/`save -r` (refused beyond
  FLASH_DEVINFO's CS0 size when ENABLE is set, and beyond the image size), `--ser` matching with
  G4's exit status. OTP bits only set: a write that would clear a bit fails. Any subcommand or
  flag it does not implement exits 99 with "fake-picotool: unimplemented", never 0.
- `scripts/test/run-e2e-otp.sh`, cases (each asserts exit code and a message regex):
  1. retail pre-check passes on a retail-shaped state (with a test `retail-otp.json` entry);
     rehearsal pre-check passes on a rehearsal-shaped state.
  2. each §3.2 row, one at a time, set wrong → `check` exits 2 naming that row; and two rows wrong
     → both named (no short-circuit).
  3. unequal copy injected in each of the three BOOT_FLAGS0 copies and each of the three
     BOOT_FLAGS1 copies (six cases, including the first, named copy) → refused; the same with a
     WARNING-only fake output (values equal, warning printed) → refused.
  4. `disable-otp-boot` and `invalidate-spare-keys`: dry-run writes nothing (state file unchanged,
     byte for byte); `--execute` with the right confirmation writes and the post-check passes;
     wrong confirmation, wrong `--ser`, two devices, retail profile without a revision entry → each
     refused with no write.
  5. a write whose read-back fails (fake told to drop one copy) → exit 3 and the "do not use"
     text.
  6. `erase-range`/`save-range`: explicit range passed (the fake records argv); a FLASH_DEVINFO
     CS0 of 8 MB under retail → erase refused → exit 2 with the condemned text; non-0xFF byte left
     after erase → exit 2.
  7. picotool version 2.2.0-a4 → refused before any device access.
  8. `inject-copy` under retail → refused before device access.
  9. R's sections A-E under the new fake, with R's real-picotool needs (`seal`, `info <file>`)
     served by a fixture UF2 so TinyGo is not needed; the old fake's run stays as is.
- Mutation checks, run once by the implementer and recorded in the report: remove the copy
  comparison, the WARNING trap, the `-c 1` comparison, the `--ser` pass-through, and the
  post-write check, one at a time; each must turn at least one case red.
- CI: a new job in `release.yml` (`otp tooling e2e`, ubuntu, bash + python3 + jq + openssl) that
  runs `run-e2e-otp.sh` and `shellcheck` on the new and changed scripts. The job name is new, so no
  required-check name changes.
- `nix flake check` and `nix build .#picotool` run by the implementer (nix is available in the
  cloud container) and on Brian's box at the bench.

## 7. Bench rehearsal (Pico 2, 4 MB, consumable; Brian at 13764k)

A fresh **plain** Pico 2 (not the W, RM residue (e)). No per-board go-ahead is needed (plan §9 item
5), but the procedure still binds every write to the board's CHIPID. Each step's full output is
saved under `rehearsal-work/e3a-<CHIPID>/` and summarised in `design/HARDWARE_RESULT_<date>_e3a.md`.

- R0. `nix develop .#otp`; `picotool version` shows 2.3.1.
- R1. R's phases 0-2 and 4 (existing): rehearsal keys in slots 0 and 1, secure boot on. Capture
  every `otp get` output once and diff its line formats against the fake's (G1); any difference
  stops the rehearsal and goes back to the implementer.
- R2. `check --profile rehearsal --stage pre`: PASS expected. Records the stock Pico 2's page
  locks, BOOT_FLAGS0 and FLASH_DEVINFO.
- R3. `erase-range`, then `save-range`: 4 MB, all 0xFF.
- R4. R's phase 5 (positive control): the my-key-signed blinky boots.
- R5. `inject-copy --row 0x04a --bits 0x002000` (DISABLE_OTP_BOOT in the third copy only).
  `check --stage pre` must refuse (copies unequal) and must show the vote as 0.
- R6. Read 0x048 with `-c 1` and 0x04a raw; record whether `-c 1` returned copy 0 alone (F10).
- R7. `disable-otp-boot --execute`. The set writes 0x2000 to all three copies; copy 3 already has
  it, so the write succeeds and heals the copy. Post-check PASS. R's phase 5 again: still boots
  (F5).
- R8. `inject-copy --row 0x04b --bits 0x000800` (KEY_INVALID for slot 3, first copy only, the
  named copy). `check --expect-key-invalid 0` must refuse, through the WARNING or the `-c 1`
  comparison.
- R9. `invalidate-spare-keys --execute`: heals the copy, post-check with `--expect-key-invalid c`
  PASS. Phase 5 again: still boots.
- R10. Optional negative control for the hardening: burn a third rehearsal key into slot 2 with R's
  `--make-otp-json` and `otp load`, set KEY_VALID bit 2, flash an image signed by that key: it
  must NOT boot (F3).
- R11. Final `check --profile rehearsal --expect-key-invalid c`: PASS, with the CANNOT PROVE list.

R5 and R8 are the plan's "injected unequal copy that the check refuses". Both injections are
subsets of the final values, so the board ends in the planned state. If R6 shows `-c 1` does not
isolate copy 0, the check keeps rule (d) only if R8 is still refused through another rule; if R8
passes the check, the rehearsal stops and the plan is re-opened.

## 8. Order of work

1. Pre-dispatch: establish G1-G4 from picotool 2.3.1 source; build `nix build .#picotool` once to
   confirm the flake shape works here.
2. Implementer (one agent, worktree, TDD): fake and e2e cases first (red), then `otp-read.sh`
   extraction (R's suites stay green), then `refugium-otp.sh`, R changes, flake, docs, CI job.
3. Adversarial review of the whole diff (opus), report to `design/agent-reports/`, fold, repeat
   until 0 C / 0 I.
4. PR; merge through the Merging PRs thread on Brian's per-PR go-ahead.
5. Bench rehearsal (§7) with Brian; result file; F-701's E3a part closes on its PASS.
6. E3b is a separate step: after E4 is decided and H0 has run `capture`, on Brian's typed
   go-ahead naming SeedHammer #1's CHIPID and each step.

## 9. Done when

- `run-e2e-otp.sh` passes locally and in CI, and every mutation in §6 turns a case red.
- R's existing e2e passes under both fakes.
- `nix build .#picotool` produces 2.3.1 on Linux (cloud) and on Brian's box.
- The §7 rehearsal passes on a Pico 2, including R5 and R8 refused, and its outputs are saved.
- No SeedHammer was written to.
