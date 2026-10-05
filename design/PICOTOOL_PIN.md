# The picotool pin

*Plan E3a §2 (`design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md`), F-701; the seal patch is
`design/IMPLEMENTATION_PLAN_e3a_picotool_seal_patch.md`.*

> **HOLD.** The `seal --clear` fix is in the toolchain but not yet proven on hardware. Until bench R4 boots a 2.3.1-sealed image, do not let 2.3.1 seal real SeedHammer firmware: an image `sign-firmware.sh` would seal itself (no SIGNATURE section yet) is sealed from the fork's shell (picotool 2.2.0-a4), and the fork does not move to this picotool. Signing an image the fork's build already sealed (R phase 5b, `sh2-flash`) works from either shell.

## What is pinned

| | |
|---|---|
| picotool | **2.3.1**, built against **pico-sdk 2.3.1**, plus `nix/patches/picotool-2.3.1-seal-clear-fix.patch` (two hunks; below) |
| source | nixpkgs-unstable, rev `a7868a727837f3c09cee2ce0ca671c76b1589fed` (locked in `flake.lock`) |
| store path (x86_64-linux), patched | `/nix/store/ri9krl08293dxl0w36765hcy45da1klp-picotool-2.3.1` — a local build (no binary-cache hit). Evaluated and built 2026-10-05 in the implementer's container (nix 2.34.6, `nix build` of this flake); CI's `picotool argv probe` job prints it on every run. It is input-addressed, so Brian's `nix build .#picotool` must produce this same path (bench R0) |
| store path (x86_64-linux), stock | `/nix/store/5cxc3bjswwjpgw9i3sdlp5xw1y4v82hc-picotool-2.3.1` (`packages.picotool-unpatched`, a binary-cache hit; CI's negative control only — never sign with it) |
| flake outputs | `packages.<system>.picotool` (also `default`), patched; `packages.<system>.picotool-unpatched`; `devShells.<system>.otp` with the patched picotool, openssl, jq, python3, xxd, tinygo and go |
| systems | x86_64-linux and aarch64-linux; x86_64-darwin and aarch64-darwin are listed but unverified |

nixpkgs already builds picotool against its own pico-sdk 2.3.1, substitutes mbedtls and installs the
udev rule. The flake's only change is `overrideAttrs` adding the seal patch to `patches` (empty in
nixpkgs at `a7868a72`); `version` stays 2.3.1, so `version -s` and the `otp list` fingerprint are
unchanged, and **only the store path tells patched from stock**. Evaluation fails if nixpkgs'
picotool is not 2.3.1, so a `flake.lock` bump cannot apply the patch to another version. The
nixpkgs input uses the
`git+https://github.com/NixOS/nixpkgs?ref=nixos-unstable&shallow=1` form because that is the form
that fetches through the sandbox's HTTPS proxy (plan §2).

```
export PATH=/nix/var/nix/profiles/default/bin:$PATH   # this box's shell does not source the nix profile
nix build .#picotool && result/bin/picotool version -s   # 2.3.1
readlink -f result     # must be the patched store path above
scripts/test/seal-clear-test.sh result/bin/picotool     # seal --sign --clear works and verifies
SEEDHAMMER_DIR=/path/to/seedhammer nix develop .#otp
```

## Who enforces it

- `scripts/refugium-otp.sh` refuses any picotool except 2.3.1. It also checks the `otp list`
  fingerprint of the SDK picotool was built with: `MAC0` must be present (row 0x062) and
  `BOOT_FLAGS0.DISABLE_BOOTSEL_EXEC2` absent (pico-sdk 2.3.1 removed it). There is no override.
- `scripts/pico2-bootkey-rehearsal.sh` (R) accepts **2.2.0-a4 or 2.3.1** and prints which, so its
  SeedHammer modes keep working from the fork's shell until the fork moves to this flake. Its
  readers mask ECC values to 16 bits, so both print formats parse the same.
- CI job `picotool argv probe` builds `.#picotool` from `flake.lock` and runs every argv shape the
  argv builder in `scripts/lib/otp-read.sh` emits against it (`scripts/test/picotool-argv-probe.sh`).
  It also builds `.#picotool-unpatched`, asserts the two store paths differ, and runs
  `scripts/test/seal-clear-test.sh` on the patched build and, with `--expect-fail`, on stock 2.3.1
  (which must exit 248). `assemble` needs this job, so a red seal test cannot be released. It is not
  yet a required check for merging (Brian's action, through the Merging PRs thread).
- `scripts/sign-firmware.sh` step 7 refuses an image whose `info -a` lacks
  `load map entry 0: Clear 0x20000000->0x20082000`, whatever picotool sealed it
  (`scripts/lib/seal-check.sh`; the refusal is exercised by the seal test).

## Why 2.3.1

- The ECC `otp get` `VALUE` is the 16-bit data (2.2.0-a4 printed the 24-bit re-encoding), so the
  parsers do not depend on a masking step.
- A lone `CHIPID1` read works. In 2.2.0-a4 a single row from a multi-row sequence could print
  nothing and exit 0.
- Register and field names are unchanged from 2.2.0-a4 for every row the tools read (plan F12).
- It is what nixpkgs-unstable ships against pico-sdk 2.3.1, so the fork, the Sitting image and this
  repo can share one build instead of three.

## What changed that matters: `seal`

- `seal --sign` in 2.3.1 adds an EXTRA_SECURITY item and a VECTOR_TABLE item to the image's
  metadata block (plan F12). The fork's `cmd/picosign` must accept 2.3.1's seal output. Bench step R0
  (plan §7) checks this before any write: build the blinky, run `sign-firmware.sh`, and require
  `picotool info -a` to report `signature: verified` and
  `load map entry 0: Clear 0x20000000->0x20082000` (`sign-firmware.sh` step 7 asserts both).
- **Stock 2.3.1 cannot `seal --sign --clear` a UF2 or BIN.** Measured 2026-10-05, no board attached:
  `sign-firmware.sh`'s throwaway-key seal (`picotool seal --sign --clear`) exits 248 with
  `ERROR: unknown sram end` on the TinyGo blinky, and the fork's own firmware seal fails the same way.
  R's old e2e (`scripts/test/run-e2e.sh`) failed phases 3, 5 and 6 on it; the pre-E3a tree failed
  the same three. Without `--clear` the seal works and verifies.
- **The cause is picotool, not the image** (`design/agent-reports/e3a-seal-clear-investigation.md`).
  An earlier version of this section put the fix in `sign-firmware.sh` or the blinky's link layout;
  that was wrong. Both images carry a valid IMAGE_DEF and the layout is irrelevant. Two upstream
  bugs, both from `raspberrypi/picotool` commit `3c743bd` ("Add support for pinning XIP SRAM
  (#298)", first released in 2.3.0), both still on `develop` at `ba3df40`:
  - **Bug A** (`main.cpp:5646` at 2.3.1, `:5807` on `develop`): `sign_guts_bin` passes
    `in.get_model()` — a file access whose model was never set, so `model_unknown` — instead of its
    `model` parameter. With `--clear`, `get_lm_hash_data` calls `sram_end()` on it
    (`bintool.cpp:863`) and fails (`model/model.h:124`), exit 248.
  - **Bug B** (`bintool.cpp:886`, both): the image is inserted *before* the Clear size word, so the
    digest is `bin ‖ size`; the bootrom hashes load-map order, `size ‖ bin`. Hidden behind A: with A
    fixed alone, picotool's own `--clear` signature comes out `incorrect`.
- **The fix: `nix/patches/picotool-2.3.1-seal-clear-fix.patch`**, applied by `flake.nix` (Brian's
  decision 2026-10-05, "Patch and report"). Hunk A passes `model`; hunk B appends the image after
  the size words. Why patch rather than keep 2.2.0-a4 for sealing or drop `--clear`: it keeps one
  picotool, and it keeps the signed load-map entry that makes the bootrom zero all of main SRAM
  before the image runs (investigation §3, §6). Calls without `--clear`/`--pin-xip-sram` are unchanged
  (measured: output differs from stock only in the randomized signature). The patched `--clear`
  output's load map is semantically identical to 2.2.0-a4's, and verifies under both versions
  before and after the fork's `picosign` re-sign (investigation §5).
- **The hold.** The `seal --clear` fix is in the toolchain but not yet proven on hardware. Until bench R4 boots a 2.3.1-sealed image, do not let 2.3.1 seal real SeedHammer firmware: an image `sign-firmware.sh` would seal itself (no SIGNATURE section yet) is sealed from the fork's shell (picotool 2.2.0-a4), and the fork does not move to this picotool. Signing an image the fork's build already sealed (R phase 5b, `sh2-flash`) works from either shell.
  What R4 proves when it passes: hunk A's output, re-signed by `picosign`, boots under the boot ROM
  with its EXTRA_SECURITY block. It does not prove hunk B (picosign recomputes the digest itself)
  and does not observe the SRAM wipe; those rest on the CI seal test, its tamper control and the
  one-time hunk-revert mutation run (`design/agent-reports/e3a-seal-patch-impl-report.md`). When R4
  passes, F-701 records it and this hold is lifted here, in the RUNBOOK and in F-701 together.
- **Upstream.** An issue text is drafted for Brian to post
  (`design/agent-reports/picotool-upstream-issue-draft.md`).
- **Dropping the patch** once an upstream release fixes both bugs: move `flake.lock` to a nixpkgs
  whose picotool carries the fix (following "Moving the pin" below), delete the patch and the
  `overrideAttrs`/version assertion in `flake.nix`, keep `seal-clear-test.sh` on `.#picotool`, and
  delete `packages.picotool-unpatched` with the `--expect-fail` CI step (stock would then pass, and
  the negative control would go red — which is the signal to do this). Record the new store path
  here.

## How the fork and the Sitting image consume it

```nix
inputs.mnemonic-engrave.url = "github:bg002h/mnemonic-engrave";   # or the git+https form
# ...
packages = [ inputs.mnemonic-engrave.packages.${system}.picotool ];
```

The fork's devshell change is its own PR in the fork thread. It is **not** a prerequisite of E3a: the
bench uses `.#otp` here, with `SEEDHAMMER_DIR` pointing at a fork checkout.

## Moving the pin

1. Move `flake.lock` (`nix flake lock --override-input nixpkgs 'git+https://github.com/NixOS/nixpkgs?ref=nixos-unstable&shallow=1&rev=<rev>'`).
2. Bump `PICOTOOL_PIN` in `scripts/lib/otp-read.sh`. The seal patch is pinned to 2.3.1: evaluation
   fails on any other version until the patch is re-derived against the new source (or dropped,
   above), and `seal-clear-test.sh` must pass on the result in both CI modes.
3. Re-check the `otp list` fingerprint and the G1 output layout (`design/agent-reports/e3a-g-facts.md`)
   against the new source.
4. Re-run the argv probe (with every board unplugged; it aborts if it sees one), `run-e2e-otp.sh`, R's old e2e and the bench transcript replay
   (`scripts/test/fixtures/transcripts/`).

A pin moved without these steps is a picotool nobody has checked.
