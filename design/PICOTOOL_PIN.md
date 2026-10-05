# The picotool pin

*Plan E3a §2 (`design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md`), F-701.*

## What is pinned

| | |
|---|---|
| picotool | **2.3.1**, built against **pico-sdk 2.3.1** |
| source | nixpkgs-unstable, rev `a7868a727837f3c09cee2ce0ca671c76b1589fed` (locked in `flake.lock`) |
| store path (x86_64-linux) | `/nix/store/5cxc3bjswwjpgw9i3sdlp5xw1y4v82hc-picotool-2.3.1` (a binary-cache hit; no local build) |
| flake outputs | `packages.<system>.picotool` (also `default`); `devShells.<system>.otp` with picotool, openssl, jq, python3, xxd, tinygo and go |
| systems | x86_64-linux and aarch64-linux; x86_64-darwin and aarch64-darwin are listed but unverified |

nixpkgs already builds picotool against its own pico-sdk 2.3.1, substitutes mbedtls and installs the
udev rule, so the flake carries no overlay. The nixpkgs input uses the
`git+https://github.com/NixOS/nixpkgs?ref=nixos-unstable&shallow=1` form because that is the form
that fetches through the sandbox's HTTPS proxy (plan §2).

```
export PATH=/nix/var/nix/profiles/default/bin:$PATH   # this box's shell does not source the nix profile
nix build .#picotool && result/bin/picotool version -s   # 2.3.1
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
  `picotool info -a` to report `signature: verified`.
- **Measured 2026-10-05, no board attached: `sign-firmware.sh` fails on 2.3.1.**
  - Its throwaway-key seal is `picotool seal --sign --clear` (`sign-firmware.sh:104`). On the
    TinyGo blinky this exits 248 with `ERROR: unknown sram end`.
  - The same command without `--clear` (the form R's `make_otp_json` uses) succeeds, and
    `info -a` then reports `signature: verified`.
  - The failure shows up in R's old e2e (`scripts/test/run-e2e.sh`) as phases 3, 5 and 6.
  - The baseline tree (`1982788`, before E3a) fails the same three phases on 2.3.1, so E3a did not
    introduce it.
  - Until it is resolved, R0 cannot pass, and the fork should not move to this pin for signing.
  - The fix belongs to `sign-firmware.sh` or the blinky's link layout, and E3a does not change
    either. It is recorded in F-701.

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
2. Bump `PICOTOOL_PIN` in `scripts/lib/otp-read.sh`.
3. Re-check the `otp list` fingerprint and the G1 output layout (`design/agent-reports/e3a-g-facts.md`)
   against the new source.
4. Re-run the argv probe, `run-e2e-otp.sh`, R's old e2e and the bench transcript replay
   (`scripts/test/fixtures/transcripts/`).

A pin moved without these steps is a picotool nobody has checked.
