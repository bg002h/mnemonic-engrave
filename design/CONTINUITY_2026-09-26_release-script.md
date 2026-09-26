# CONTINUITY — 2026-09-26: next up is a release script for every repo

Written before an operator reboot. Everything below is pushed; nothing was
in flight.

## Where things stand (all shipped, CI green, no bypass)

- **Signing, all seven repos.** One minisign key (id `EF2B8D34D8409754`,
  pubkey `RWRUl0DYNI0r72HYC0ou+T/7pHEf0km3a8RWHwqGwZmIEMWtiSd4k0B5`) signs every
  release's checksum files: engrave, toolkit, ms, md, mk, gui, mnemonic-transaction.
  The password is in the operator's Proton Pass. Secrets
  `MINISIGN_SECRET_KEY`/`_PASSWORD` are set in all seven. Release tags fail
  rather than ship unsigned. Each release workflow has a `sign_dry_run` dispatch
  mode that signs + verifies and publishes nothing. The retired key
  `CA39ECB257009A0F` is valid only for engrave <= v0.12.0.
- **First signed release:** `mt-cli-v0.1.1` (mnemonic-transaction `40e5809`).
  The published `SHA256SUMS.portable.minisig` was verified locally with the new
  key, and the linux-amd64 binary was checksummed and run (`mt 0.1.1`).
- **Installer** (toolkit `scripts/install.sh`) always checks SHA256SUMS, and
  checks minisign signatures when `minisign` is installed. `--require-signature`
  makes that mandatory. Per-component `first_signed` entries are EMPTY because no
  installer-pinned tool has a signed release yet. `install-assets.test.sh` forces
  the entry to be filled when a pin moves onto a signed release.
- CI runs on cargo nextest (F-696), and the aarch64 legs run on native ARM runners.
- Current releases: mnemonic-toolkit 0.105.1, ms-cli 0.20.1, md-cli 0.20.3,
  mk-cli 0.13.0, mnemonic-gui 0.63.0, manual-gui v1.4.0, me v0.12.0, mt-cli 0.1.1.

## The next task (operator asked, then said "hold off" until after reboot)

**A release script for all repos** (e.g. `scripts/cut-release.sh <repo> <version>`,
living in engrave next to `push-via-staging.sh`, which is repo-agnostic and
byte-identical in 5 repos). What cutting `mt-cli-v0.1.1` by hand took:

1. Bump the CLI crate's version (`crates/<x>-cli/Cargo.toml`), then
   `cargo update --workspace --offline` to update Cargo.lock. Build it and
   check `--version`.
2. CHANGELOG: move `[Unreleased]` under the new version, where the repo has one
   (mt has no CHANGELOG).
3. Commit, then `push-via-staging.sh` (CI green, no bypass). **Freeze** the
   default branch through step 5.
4. `git tag -a <prefix><ver>` on that exact commit and push the tag. Prefixes:
   `mnemonic-toolkit-v`, `ms-cli-v`, `descriptor-mnemonic-md-cli-v`,
   `mk-cli-v`, `mt-cli-v`, `mnemonic-gui-v`, engrave `v`.
5. Wait on every run for the tag by run id (never `pgrep -f`). Then download
   `SHA256SUMS*` + `.minisig` and verify them with `minisign -V -P <pubkey>`;
   checksum and run the linux binary.
6. Follow-on pins, the part that's easy to forget:
   - an md, ms or toolkit tag turns engrave CI red until
     `demo/sh2/src/index.html` + `CLI_SPINE.md` name it;
   - a new tool version usually wants the toolkit installer pin moved, with the
     Examples golden regenerated in the same commit, the GUI-manual
     `check_cli_pins` and the sibling-pin mirrors;
   - the first signed release of an installer-pinned tool must fill its
     `first_signed` entry.
7. **Toolkit is special:** its own tag must go on the commit whose `install.sh`
   pins that tag (install-pin-check). So: a release commit without the pins,
   then a separate pin commit, which gets the tag and lands via PR. See
   `design/agent-reports/release-ms020-tk0105.md` for the exact sequence.

Tagging is irreversible, so per CLAUDE.md the script is risk-set work: a short
spec, an R0 review, then the implementation and a review. It must support a
dry run that does everything short of pushing a tag.

Also stale: `scripts/verify-releases.sh` has hard-coded old tags and checks
checksums only. Fold it into the new script's verify step or refresh it
(signatures included).

## Open follow-ups touched this session

F-682 (Refugium idea), F-697 (fetch_sig redirect label), F-698 (CI build
caching; the toolkit's release.yml re-runs the whole suite), plus older ones in
`design/FOLLOWUPS.md`.

## How to resume

Run `/resume-release-script`, or say "resume the release script from
design/CONTINUITY_2026-09-26_release-script.md".
