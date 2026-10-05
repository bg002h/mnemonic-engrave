# `blinky.uf2` — the seal-test input

`scripts/test/seal-clear-test.sh` seals this image with a throwaway key. It is
the unsealed TinyGo blinky that R (`scripts/pico2-bootkey-rehearsal.sh`) builds in
`build_blinky`, and the input image of the investigation
(`design/agent-reports/e3a-seal-clear-investigation.md`, which calls it
`blinky.uf2`).

| field | value |
|---|---|
| sha256 | `e48659d727367cd2c23bffa6ad4b56d7388f83c9f9e3a6b4998c28431f8454a9` (checked by the test before anything else) |
| size | 47,104 bytes (92 UF2 blocks) |
| family | `rp2350-arm-s` |
| IMAGE_DEF | Metadata Block 1 at `0x100000f8`: EXE, ARM, Secure, RP2350 (`ffffded3 10210142 000001ff <link> ab123579`); no load map, no SIGNATURE |
| source | `scripts/rehearsal-blinky/` in this repo (`main.go`, `go.mod`; last changed in `fa8bba6`); no fork commit is involved |
| compiler | TinyGo 0.42.0, `/nix/store/0fpy7i24qwyhv5vdj49gvrzgaa94hy4s-tinygo-0.42.0` — the `tinygo` in `devShells.otp` at nixpkgs `a7868a72` (`flake.lock`). `tinygo version` prints `tinygo version 0.42.0 linux/amd64 (using go version go1.27.1 and LLVM version 22.1.8)` |
| command | R's exact `build_blinky` command, run in `scripts/rehearsal-blinky/`: `tinygo build -o blinky.uf2 -target pico2 -opt 2 .` |

**Reproduced byte-for-byte** on 2026-10-05 (x86_64-linux): running that command
with that TinyGo from this tree gave the same sha256. A different TinyGo or LLVM
will most likely give a different image; the test pins the bytes, not the build,
so re-derive the sha256 in the test only together with a reason in the commit.

It carries no key and no secret: it has never been sealed or signed, and the test
generates a fresh secp256k1 key per run.
