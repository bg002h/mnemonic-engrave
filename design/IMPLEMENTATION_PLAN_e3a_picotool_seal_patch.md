# IMPLEMENTATION PLAN — E3a follow-up: carry the picotool 2.3.1 `seal --clear` fix in the flake (F-701)

*Draft 2, 2026-10-05. Author: thread "mnemonic-engrave for Refugium". Baseline: mnemonic-engrave
`39ee550` (master after PR 11 and PR 13). Risk set (a): this changes the firmware signing toolchain.
R0 to 0 C / 0 I before code, a single implementer, then an adversarial review of the whole diff.*

Draft 2 folds R0 round 1 (`design/agent-reports/e3a-seal-patch-plan-r0.md`, 0 C / 2 I / 6 M / 4 N;
fold table §5). The reviewer re-verified the patch against source and the bootrom, recomputed the
digest independently, and confirmed the nix override mechanics at `a7868a72` (`patches = []`,
`version` stays 2.3.1, the patch applies without fuzz).

Decision: Brian chose "Patch and report" on 2026-10-05 (decision card in the thread). Evidence:
`design/agent-reports/e3a-seal-clear-investigation.md` (**INV**). The candidate patch is
`design/patches/picotool-2.3.1-seal-clear-fix.patch`.

Build-gate coverage: no ```rust blocks; `scripts/plan-build-gate.sh` covers nothing. The gates are the
CI seal test (§3), the nix build in CI (§3), R's old e2e at bench R0, and a hardware boot at bench R4.

## 0. Outcome

1. `packages.picotool` in `flake.nix` is picotool 2.3.1 from the same nixpkgs rev (`a7868a72`) with
   the two-hunk patch applied. `version -s` still prints `2.3.1`, so `refugium-otp.sh`'s pin check and
   the `otp list` fingerprint are unchanged.
2. `picotool seal --sign --clear` works on UF2 input and produces a load map whose signature verifies.
3. CI proves (1) and (2) on every change, with a negative control on the unpatched build.
4. An upstream issue text is drafted for Brian to post (outward-facing: he posts it, not Claude).

## 1. Facts (from INV, checked against picotool `2.3.1` source at `2041936`)

| id | fact | source |
|---|---|---|
| P1 | Bug A: `sign_guts_bin` passes `in.get_model()` (a `model_unknown`) instead of its `model` parameter; with `--clear`, `get_lm_hash_data` calls `sram_end()` on it → `ERROR_NOT_POSSIBLE` → exit 248 "unknown sram end" | INV §1, `main.cpp:5646`, `bintool.cpp:863`, `model/model.h:124` |
| P2 | Bug B: the BIN `get_lm_hash_data` inserts the image **before** the clear-size word (`to_hash.insert(to_hash.begin(), bin…)`), so the digest is `bin ‖ size`; the bootrom hashes load-map order `[Clear, Load]` = `size ‖ bin`. Fixing A alone gives `signature: incorrect` | INV §1, `bintool.cpp:886`, bootrom `varm_blocks.c:973-997` |
| P3 | Both bugs come from upstream `3c743bd` (first released 2.3.0); still present on `develop` `ba3df40` | INV §1, §4(c) |
| P4 | ELF input is unaffected (`sign_guts_elf` passes the real model, `main.cpp:5574`); calls without `--clear`/`--pin-xip-sram` and with no prior load map never reach the changed code, and their output differs from stock only in the random signature bytes | INV §1, §4(b) (measured) |
| P5 | `--clear` adds a signed load-map entry `Clear 0x20000000→0x20082000`; the bootrom zeroes main SRAM before the image runs; nothing else clears it on a normal boot | INV §3 |
| P6 | Patched output verifies under picotool 2.3.1 and 2.2.0-a4, before and after the fork's `picosign` re-sign; its load map is semantically identical to 2.2.0-a4 `--clear` output (same entries, same block address, one relative storage-offset word shifted 8 bytes, resolving to the same `0x10000000`) | INV §5 (measured on native builds) |
| P7 | Nix's build sandbox cannot run in the cloud container (`/dev/null: Permission denied`); the CI probe job runs `nix build .#picotool` on a hosted runner | INV; E3a impl report |

## 2. Changes

- **Patch location.** Move the patch to `nix/patches/picotool-2.3.1-seal-clear-fix.patch` (a path the
  CI docs-only rule classes as code, unlike `design/`), with a header comment naming upstream
  `3c743bd`, the two hunks, and INV. Delete the `design/patches/` copy so there is one source.
- **`flake.nix`.** `picotool = pkgs.picotool.overrideAttrs (old: { patches = (old.patches or []) ++
  [ ./nix/patches/picotool-2.3.1-seal-clear-fix.patch ]; });` used for `packages.picotool`,
  `packages.default` and `devShells.otp`. Add `packages.picotool-unpatched = pkgs.picotool;` for the
  negative control only (not used by any shell). An assertion in the flake fails evaluation if
  `pkgs.picotool.version != "2.3.1"`, so a `flake.lock` bump cannot silently apply the patch to a
  different version.
- **Fixture.** `scripts/test/fixtures/seal/blinky.uf2`: the unsealed TinyGo blinky R's `build_blinky`
  makes (INV's input image; IMAGE_DEF EXE/ARM/Secure/RP2350), with a `README.md` stating how it was
  built (TinyGo version and source, target, R's exact command, fork commit if any) and its sha256 (N3). It carries no key and no secret.
- **`scripts/test/seal-clear-test.sh <picotool> [--expect-fail]`.** In a temp dir: generate a
  throwaway secp256k1 key with `openssl ecparam`; run `picotool seal --sign --clear --quiet
  blinky.uf2 out.uf2 key.pem`; require exit 0; run `picotool info -a out.uf2`; require
  `signature:           verified` and `load map entry 0:    Clear 0x20000000->0x20082000` (measured
  2026-10-05 on a native build of patched 2.3.1 against INV's `blinky.uf2`; the script cites the
  2.3.1 source line that prints each). Also seal **without** `--clear` and require it
  verifies (P4 regression guard). With `--expect-fail` (the unpatched build) require exit 248 and
  "unknown sram end" on the `--clear` seal, proving the test can tell patched from unpatched. The
  fixture's sha256 is checked first. Greps are anchored to whole lines, and `signature:` must appear
  only as `verified` (the line prints twice in `info -a`; N1). **Tamper control (SP-M3):** XOR 0x01 into
  byte 32 of the sealed `out.uf2` (block 0's first payload byte, target `0x10000000`, inside the hashed
  range; skipped under `--expect-fail`; SP2-N1) and require
  `info -a` to print `signature:           incorrect`, so the check can tell a correct signature from
  a wrong one. Hunk B is otherwise proven only by the one-time mutation run (§3).
- **CI.** In the `picotool argv probe` job (already builds `.#picotool` with nix, `contents: read`):
  build with explicit out-links, `nix build .#picotool -o pt-patched` and `nix build
  .#picotool-unpatched -o pt-stock`, assert the two store paths differ, print the patched one, and
  run the argv probe and `seal-clear-test.sh` on `pt-patched/bin/picotool`, and `seal-clear-test.sh
  --expect-fail` on `pt-stock/bin/picotool` (SP-M2). The scripts run `info -a` with a file argument
  only; no device is touched.
- **The test must gate (SP-I1).** The probe job is not a required check today. This PR adds
  `picotool-probe` (the job id) to `assemble`'s `needs:` so a red seal test can never be released,
  and before merge the thread asks Brian, through Merging PRs 2, to make `picotool argv probe` a
  required status check. Until he has, the PR body records the green probe run on the final head by
  link, and the merge ask names that run.
- **Docs, with the hardware hold written in (SP-I2).** Patched and stock both print `2.3.1`, so no
  tool can enforce the hold; the words must. `design/PICOTOOL_PIN.md`, the RUNBOOK prerequisites note
  (`RUNBOOK_custom_boot_key.md:85-88`) and F-701 each say, in these words: "The `seal --clear` fix is
  in the toolchain but not yet proven on hardware. Until bench R4 boots a 2.3.1-sealed image, do not
  let 2.3.1 seal real SeedHammer firmware: an image `sign-firmware.sh` would seal itself (no SIGNATURE
  section yet) is sealed from the fork's shell (picotool 2.2.0-a4), and the fork does not move to
  this picotool. Signing an image the fork's build already sealed (R phase 5b, `sh2-flash`) works
  from either shell." The RUNBOOK's "the SH2 steps below work from either shell" sentence is amended
  to match (SP2-M1). PICOTOOL_PIN.md also gains: the patch, why, the two upstream bugs, how to drop
  the patch once upstream fixes both, the patched store path as built in CI and on Brian's box
  (SP-M5), and the seal section corrected (the cause is picotool, not the image). Every reference to
  `design/patches/` (FOLLOWUPS F-701, PICOTOOL_PIN.md) is updated to `nix/patches/` (SP-M6).
- **`sign-firmware.sh` checks the Clear entry (SP-M1).** After the final signature check it also
  requires a line matching `^[[:space:]]*load map entry 0:[[:space:]]+Clear 0x20000000->0x20082000$`
  in the captured `picotool info -a` output (matched with a here-string, not a pipe, F-695), else it
  refuses (an image sealed elsewhere without `--clear`, or one whose existing load map made
  picotool ignore `--clear`, would otherwise pass as verified with no SRAM wipe). R's old e2e must
  still pass with this check, and `seal-clear-test.sh` exercises the refusal: it seals the fixture
  without `--clear`, runs `sign-firmware.sh`'s check on it (or the script with a stub picosign), and
  requires the refusal (SP2-M2).
- **Upstream issue draft** `design/agent-reports/picotool-upstream-issue-draft.md`: title, repro
  (any TinyGo or SDK UF2 with an IMAGE_DEF, `seal --sign --clear`), both bugs with line numbers at
  `2.3.1` **and** at `develop@ba3df40` (`main.cpp:5807`, `bintool.cpp:886`), every affected path
  (UF2/BIN `--clear`, `--pin-xip-sram`, `--hash --clear`; ELF unaffected), the patch, and the
  verification (SP-M4). Delivered to Brian as an editable draft; not posted by Claude.

## 3. Tests and gates

- `seal-clear-test.sh` passes on `.#picotool` and passes in `--expect-fail` mode on
  `.#picotool-unpatched`, in CI.
- Mutation (run once by the implementer, recorded): revert each hunk alone. Reverting hunk A → the
  patched test fails with 248. Reverting hunk B → `signature: verified` is missing. Both red runs
  shown in the report. (Native cmake builds of the mutants are acceptable where nix cannot build
  here; say which.)
- `run-e2e-otp.sh` and the argv probe still pass (the OTP tool is unaffected by the seal path).
- `seal-clear-test.sh` is added to the CI shellcheck list (N4).
- Bench (E3a plan §7): R0 runs `sign-firmware.sh` on the blinky with the patched picotool and
  requires `signature: verified` and the Clear entry after `picosign` (the E3a plan's R0 line is
  updated to name the Clear entry); R4's phase 5 boots that image on the Pico 2. What R4 proves (N2):
  hunk A's output, re-signed by `picosign`, boots under the boot ROM with its EXTRA_SECURITY block.
  It does not prove hunk B (picosign recomputes the digest itself) and does not observe the SRAM
  wipe; those rest on the CI test, the tamper control and the mutation run.
- The fork does not move to this picotool until R4 has booted on hardware; the fork thread is told.

## 4. Done when

- CI green with both seal-test runs; `nix build .#picotool` on Brian's box prints `2.3.1`.
- The upstream draft is with Brian.
- Bench R0 and R4 pass (closes with the E3a bench rehearsal, not this PR).

## 5. Fold table (R0 round 1)

| finding | fold |
|---|---|
| SP-I1 seal test gates nothing | `picotool-probe` in `assemble.needs`; required-check ask before merge; green run linked in the PR §2 |
| SP-I2 hardware hold not written down | exact hold wording in PICOTOOL_PIN, RUNBOOK:85-88 and F-701 §2 |
| SP-M1 Clear entry never checked | `sign-firmware.sh` requires it; bench R0 names it §2, §3 |
| SP-M2 out-links | explicit `-o`, store paths asserted different §2 |
| SP-M3 hunk B in CI | tamper control §2; mutation run stays §3 |
| SP-M4 issue line numbers | `develop@ba3df40` lines and all affected paths §2 |
| SP-M5 2.3.1 does not prove patched | patched store path recorded in PICOTOOL_PIN §2 |
| SP-M6 stale paths | `design/patches/` references updated §2 |
| N1 anchored grep | §2 |
| N2 what R4 proves | §3 |
| N3 fixture provenance | §2 |
| N4 shellcheck list | §3 |

Round 2 (`e3a-seal-patch-plan-r1.md`, Sonnet fold check): 0 C / 0 I / 2 M / 1 N, which closes the R0
gate. SP2-M1 hold scoped to images `sign-firmware.sh` seals itself, RUNBOOK sentence amended; SP2-M2
whitespace-tolerant here-string match and a tested refusal; SP2-N1 tamper recipe fixed.
