# `descriptor_seam_vectors.json` — the generator

`../../crates/me-cli/testdata/descriptor_seam_vectors.json` is DATA, and every
device-side and value column in it is a MEASUREMENT. This directory is how the
measurement is re-run, so the file's provenance is a command rather than a
claim. (Standing lesson: a reproduction path nobody re-runs rots while its
artifact keeps vouching for it.)

- `rows.py` — the 72 row DEFINITIONS. Host-side columns (`host_admits`,
  `md1_admits`, `format`, `covers`, the four gate fields) are AUTHORED from
  `design/SPEC_descriptor_input.md`. Nothing measured lives here.
- `gen.py` — runs the probes, fills every measured column, cross-checks the
  routes against each other, and refuses to write the file if any check fails.
- `goprobe/` — the device side: `nonstandard.OutputDescriptor`,
  `bip380.Descriptor.Encode`, `address.Receive/Supported`, `sysw.Classify`, and
  the fork's own `md.EncodeMultisig` → `WalletPolicyIdChunks`.
- `rsprobe/` — the PUBLISHED `md-codec 0.42` (the exact crate `me` links)
  reassembling an md1 set and computing `compute_wallet_policy_id`.
- `keytool-main.go` — the recorded key-material derivations: SLIP-132 version
  re-serialisation, and the 16/21 unhardened children of the `dc567276` fixture
  key. Run it in its own module (`go mod init keytool && go mod tidy`).

## Running it

```sh
cd scripts/descriptor-seam-vectors
$EDITOR goprobe/go.mod                     # point `replace` at your fork worktree
(cd rsprobe && cargo build --offline)
python3 gen.py ../../crates/me-cli/testdata/descriptor_seam_vectors.json
sha256sum ../../crates/me-cli/testdata/descriptor_seam_vectors.json
```

Then re-pin that sha256 in **both** tests — `crates/me-cli/tests/descriptor_seam.rs`
and the fork's `nonstandard/descriptor_seam_test.go` — and re-vendor the file to
`seedhammer/nonstandard/testdata/`. The two literals are what stop the copies
drifting.

`SEAM_GO` and `SEAM_MD` override the toolchain paths. `SEAM_MD` must be the
**debug** binary built from the `descriptor-mnemonic` tree: the installed
`~/.cargo/bin/md` is stale and does not have the `descriptor` subcommand (spec
§2's stale-binary trap).

## What the generator CHECKS, and therefore what a reviewer need not re-derive

It exits non-zero, writing nothing, on any of:

- a `host_admits=true` row whose `canonical` is missing, or is not a device
  **fixed point** (parse → re-encode → equal);
- an unexpected parse panic on a row not marked `device_probe`;
- an `address_N` that the device route and the md1 route disagree on;
- a `wallet_id` on which the `md` CLI, the published `md-codec 0.42` and the
  fork's Go `md` package do not all three agree — the F-212 gate, at
  generation time as well as in the two suites;
- an `md_descriptor_contains` pin absent from the real `md descriptor`
  read-back;
- a `want_wid` row with no md1 route to compute the Rust side from.

## Baselines this corpus was measured against

fork **0abbf81** (the `s2/descriptor-arm` worktree, carrying P3.1's `!= 4`
fingerprint guard and P3.4's `ypubVer` case — S2 measures two `device_admits`
booleans from it before either fix reaches fork `main`) ·
`descriptor-mnemonic` **6c4a56fd** (debug `md`) ·
published **md-codec 0.42.0** · Go **1.26.3**.
