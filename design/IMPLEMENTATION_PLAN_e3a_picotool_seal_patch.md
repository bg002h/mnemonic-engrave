# IMPLEMENTATION PLAN — E3a follow-up: carry the picotool 2.3.1 `seal --clear` fix in the flake (F-701)

*Draft 1, 2026-10-05. Author: thread "mnemonic-engrave for Refugium". Baseline: mnemonic-engrave
`39ee550` (master after PR 11 and PR 13). Risk set (a): this changes the firmware signing toolchain.
R0 to 0 C / 0 I before code, a single implementer, then an adversarial review of the whole diff.*

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
  built (TinyGo version, target, R's command) and its sha256. It carries no key and no secret.
- **`scripts/test/seal-clear-test.sh <picotool> [--expect-fail]`.** In a temp dir: generate a
  throwaway secp256k1 key with `openssl ecparam`; run `picotool seal --sign --clear --quiet
  blinky.uf2 out.uf2 key.pem`; require exit 0; run `picotool info -a out.uf2`; require
  `signature:           verified` and `load map entry 0:    Clear 0x20000000->0x20082000` (measured
  2026-10-05 on a native build of patched 2.3.1 against INV's `blinky.uf2`; the script cites the
  2.3.1 source line that prints each). Also seal **without** `--clear` and require it
  verifies (P4 regression guard). With `--expect-fail` (the unpatched build) require exit 248 and
  "unknown sram end" on the `--clear` seal, proving the test can tell patched from unpatched. The
  fixture's sha256 is checked first.
- **CI.** In the `picotool argv probe` job (already builds `.#picotool` with nix, `contents: read`):
  build `.#picotool-unpatched` too, then run `seal-clear-test.sh` on both. The probe's no-board gate
  is not needed here (seal and `info` on a file touch no device), but the script runs `info -a` with
  a file argument only.
- **Docs.** `design/PICOTOOL_PIN.md`: replace "no overlay" with the patch, why, how to drop it when
  upstream fixes both bugs, and that the store path changes; fix the seal section (cause is picotool,
  not the image). F-701 progress: blocker resolved in the toolchain; bench R0 and R4 remain the
  hardware proof. RUNBOOK prerequisites unchanged (`nix develop .#otp`).
- **Upstream issue draft** `design/agent-reports/picotool-upstream-issue-draft.md`: title, repro
  (any TinyGo or SDK UF2 with an IMAGE_DEF, `seal --sign --clear`), both bugs with line numbers, the
  patch, and the verification. Delivered to Brian as an editable draft; not posted by Claude.

## 3. Tests and gates

- `seal-clear-test.sh` passes on `.#picotool` and passes in `--expect-fail` mode on
  `.#picotool-unpatched`, in CI.
- Mutation (run once by the implementer, recorded): revert each hunk alone. Reverting hunk A → the
  patched test fails with 248. Reverting hunk B → `signature: verified` is missing. Both red runs
  shown in the report. (Native cmake builds of the mutants are acceptable where nix cannot build
  here; say which.)
- `run-e2e-otp.sh` and the argv probe still pass (the OTP tool is unaffected by the seal path).
- Bench (unchanged §7 of the E3a plan): R0 runs `sign-firmware.sh` on the blinky with the patched
  picotool and requires `signature: verified` after `picosign`; R4's phase 5 boots that image on the
  Pico 2 (the first hardware boot of a 2.3.1-sealed, EXTRA_SECURITY image).
- The fork does not move to this picotool until R4 has booted on hardware; the fork thread is told.

## 4. Done when

- CI green with both seal-test runs; `nix build .#picotool` on Brian's box prints `2.3.1`.
- The upstream draft is with Brian.
- Bench R0 and R4 pass (closes with the E3a bench rehearsal, not this PR).
