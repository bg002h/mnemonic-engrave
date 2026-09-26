# Sign every release's checksums — implementation report

Brief: `design/briefs/sign-all-releases.md`, plus two controller messages (key rotation; parallelise the proofs).
Date: 2026-09-26. Author: implementer agent (Opus 5.5).

## Outcome

- **Five repos proven with the new key.** mnemonic-toolkit, mnemonic-secret, descriptor-mnemonic, mnemonic-key and mnemonic-gui now sign every checksum file they publish. Non-publishing dispatch runs signed and verified with the **rotated** key (EF2B8D34D8409754) in all nine workflows, and uploaded nothing.
- **Installer.** It verifies signatures against a range-scoped trusted-key table, with 14 offline tests and seven mutations, all caught. The first real-runner CI run is green.
- **Engrave is not finished.** Its trust-anchor switch to the new key (`minisign.pub`, the README key block, the pinned key in `release.yml`) was **denied by the permission classifier** ("Security Weaken"), and I did not retry it. So engrave's `release.yml` on `sign-releases` still verifies against the **old** key, and its new-key proof run has **not** been done. See "Blocked" below.
- No default branch pushed, nothing merged or tagged, and no release created. I checked that no `sign-releases` release exists in any of the six repos.

## Branches (all `sign-releases`, pushed; worktrees under `/scratch/code/shibboleth/<prefix>-worktrees/sign`)

| repo | worktree | tip | commits |
|---|---|---|---|
| mnemonic-toolkit | tk-worktrees/sign | 9efaa88544 | a912a6fd workflows · 5a8b429d installer · b3affa91 rust.yml paths · 899249af key · 9efaa885 installer key table |
| mnemonic-secret | ms-worktrees/sign | 9362fa88bd | 1f44385 workflows · 9362fa8 key |
| descriptor-mnemonic | dm-worktrees/sign | 4582a8d9ef | 9bcae9e2 workflows · 4582a8d9 key |
| mnemonic-key | mk-worktrees/sign | 13d5b22a0b | 648fc95 workflows · 13d5b22 key |
| mnemonic-gui | gui-worktrees/sign | 0f41eb1084 | 1365fbb build.yml · 0f41eb1 key |
| mnemonic-engrave | me-worktrees/sign | 7864c81a56 | 412def40 release.yml policy + dispatch · 7864c81a generator |

## What signs what, and where

| repo | file signed | workflow / job | signature |
|---|---|---|---|
| toolkit, ms, md, mk | `SHA256SUMS.portable` (macOS/Windows, plus md's Linux gnu) | `release.yml` / `assemble` | `SHA256SUMS.portable.minisig` |
| toolkit | `SHA256SUMS.x86_64`, `SHA256SUMS.aarch64` (reproducible musl) | `man-pages.yml` / `musl-binaries` | `.minisig` per arch |
| ms | same | `man-release.yml` / `musl-binaries` | same |
| md | same | `man-pages.yml` / `musl-binaries` | same |
| mk | same | `musl-binaries.yml` / `musl-binaries` | same |
| gui | `SHA256SUMS` (all 7 targets) | `build.yml` / `release` | `artifacts/SHA256SUMS.minisig`, published by the existing `artifacts/**/*` glob |
| engrave | `SHA256SUMS` | `release.yml` / `assemble` (unchanged mechanism) | `SHA256SUMS.minisig` |

**Where the reproducible Linux signing lives, and why.** It is in each caller's own `musl-binaries` job, **not** in the reusable `reproducible-musl-build.yml`. The reusable workflow only gates reproducibility; it never writes or uploads the published `SHA256SUMS.<arch>`. Each caller's `musl-binaries` job does both. So no secrets are passed to the reusable workflow, and its SHA pin in ms/md/mk (`@4120af85…`) is untouched.

Signing is a **host** step after build+package and before `ensure-release`, so the secret never enters the `--network=none` container or the cross container. The step hashes the tarball, `SHA256SUMS.<arch>` and `PROVENANCE.<arch>.txt` before and after signing, and fails if a byte moved. The repro gate runs in separate jobs and hashes nothing this step writes.

**Single-sourced public key.** Each workflow holds the key exactly once:
- the `MINISIGN_PUBKEY` env var in each `release.yml`;
- one step env in each musl caller and in the GUI.

I counted the occurrences mechanically: 1 per file across the 9 workflow files, plus `gen.py`. In install.sh the key lives in the one `signing_keys` table.

## Tag-vs-dispatch behaviour (same in every repo)

| run | secret present, sign + verify OK | secret missing | sign or verify fails |
|---|---|---|---|
| release tag push (or `release.yml` backfill dispatch with `tag`: it uploads, so it counts as publishing) | signs, verifies, uploads sums + `.minisig` **in one call** | **job FAILS** with an explicit error, before anything from that job uploads | **job FAILS** before upload |
| `release.yml` dispatch with `sign_dry_run: true` | signs, verifies, **uploads nothing, no attestation** | warning, unsigned, uploads nothing | job fails |
| musl callers / GUI / engrave `workflow_dispatch` | signs, verifies; `ensure-release` / `upload` / `github-release` / attest are tag-only, so skipped | warning, unsigned, nothing published | job fails |

Further details:
- An encrypted key with an empty password fails with a hint naming `MINISIGN_SECRET_KEY_PASSWORD`.
- `release.yml` used to upload the `.minisig` with `|| true`. It now fails if the signature is missing or the upload fails.
- Scope of a failure: on a tag, the failing job uploads nothing. Other tag-triggered workflows on the same tag (the man tarball, the other workflow's assets) can still attach their own files, so the release is red and incomplete, not silently unsigned.
- New triggers:
  - toolkit `man-pages.yml`: `workflow_dispatch`, with the man-tarball job made tag-only;
  - ms and mk: the `musl-binaries` job now runs on dispatch (md already did);
  - GUI `build.yml` and engrave `release.yml`: `workflow_dispatch`.

## Proof runs — the rotated key (EF2B8D34D8409754), all non-publishing

I checked every run step by step through the jobs API:
- **signed**: the sign step succeeded;
- **verified**: the verify step succeeded (it runs only when signing happened);
- **nothing published**: upload / `ensure-release` / attest / `github-release` were **skipped**.

| repo | workflow | run id | legs signed + verified |
|---|---|---|---|
| mnemonic-toolkit | release.yml (`sign_dry_run`) | 36215620417 | SHA256SUMS.portable |
| mnemonic-toolkit | man-pages.yml | 36215621875 | SHA256SUMS.x86_64, SHA256SUMS.aarch64 (repro gate green too) |
| mnemonic-secret | release.yml (`sign_dry_run`) | 36215623161 | SHA256SUMS.portable |
| mnemonic-secret | man-release.yml | 36215624391 | x86_64, aarch64 |
| descriptor-mnemonic | release.yml (`sign_dry_run`) | 36215625533 | SHA256SUMS.portable |
| descriptor-mnemonic | man-pages.yml | 36215626780 | x86_64, aarch64 |
| mnemonic-key | release.yml (`sign_dry_run`) | 36215628013 | SHA256SUMS.portable |
| mnemonic-key | musl-binaries.yml | 36215629411 | x86_64, aarch64 |
| mnemonic-gui | build.yml | 36215630581 | SHA256SUMS |
| mnemonic-engrave | release.yml | 36214464144 | SHA256SUMS: sign + verify success, attest/publish skipped. **This ran at 03:19Z with the PRE-rotation engrave secrets and the old pinned key.** |

**Fact for the controller.** The operator rotated because the old key's password was lost, but run 36214464144 signed successfully with engrave's pre-rotation `MINISIGN_SECRET_KEY` and `MINISIGN_SECRET_KEY_PASSWORD` (engrave's secrets were updated at 03:29–03:30Z). So the old secret and password stored in engrave were working until then. That does not change the rotation; it only means "lost" was not true of the GitHub copy.

Toolkit full CI on the final SHA 9efaa885 (via a temporary `ci/sign-releases` ref, since deleted): all green. That covers rust (13/13 jobs, including the new `install-signature harness (offline)` step on a real runner), examples (the golden gate), manual, manual-gui, quickstart, technical-manual, sibling-pin-check and release.

## Installer (`scripts/install.sh`)

- **Trusted keys:** `signing_keys` is one table of `<component>|<first>|<last>|<key>` lines, with `*` meaning every component and an empty bound meaning open:
  - `*|||RWRUl0DYNI0r72HYC0ou+T/7pHEf0km3a8RWHwqGwZmIEMWtiSd4k0B5`: the new key, for everything.
  - `mnemonic-engrave||0.12.0|RWQPmgBXsuw5yi8W0SfDr8KF+IqY/Z5U2p724emSODS1UPfJBP3agbKW`: the old key, only for the engrave releases it signed. install.sh does not install engrave today, so this entry is a record that keeps the old key from verifying anything else. A future rotation adds a line and bounds the old one.
- **`trusted_keys <name> <version>`** selects the keys covering that release. `check_signature` accepts the signature if **any** of them verifies it, and refuses if none does (including when no key covers the release).
- **Order of checks:** the `SHA256SUMS*` file is only searched for the asset name. Its `.minisig` is verified **before** any digest in it is compared.
- **Missing signature:**
  - refused when the pin is at or after `first_signed <component>`;
  - refused under `--require-signature`;
  - otherwise installs with a note.
  - The `first_signed` table holds **bare versions** (the pin gates grep for the first `<name>-v<x.y.z>`). All five entries are **empty**, because no signed releases exist yet. Fill each one when that component's first signed release ships.
- **minisign absent:** one note up front, then sha256 only. With `--require-signature` it is an error before anything is downloaded. The note is not printed in `--dry-run`, which keeps the Examples golden independent of the host.
- **Dry-run:** one new static line per binary ("check that SHA256SUMS file's minisign signature (if minisign is installed); refuse a bad one"). The golden was updated, and its `--list` and `--dry-run` blocks match a regeneration under gen.sh's pinned environment; the CI examples gate is green.
- **Docs:** `--help` (SOURCE, the new OPTIONS entry, REQUIREMENTS), README (install and "Verifying your download" origin check), the manual chapter `docs/manual/src/20-quickstart/21-install.md` (with `minisign` added to `.cspell.json`), and CHANGELOG [Unreleased].
- **Tests:** `scripts/install-signature.test.sh`, offline. It generates throwaway keypairs with `minisign -G -W`, rewrites the table in a copy of the installer, and asserts every rewrite applied. **14/14 pass:**
  1. good signature;
  2. asset **and** its sums line swapped with the old `.minisig` kept: refused on the signature;
  3. signature by another key;
  4. missing, no pin: note;
  5. missing, before the pin;
  6. missing, at the pin: refused;
  7. missing, after the pin: refused;
  8. minisign absent (a PATH farm without it): said **once** across two components;
  9. absent + `--require-signature`: nothing downloaded;
  10. missing + `--require-signature`;
  11. good + `--require-signature`;
  12. key pinned only for other versions: refused;
  13. second, version-ranged key verifies;
  - plus a check of the shipped table.
- **Mutations, all caught:**

  | mutation | cases that fail |
  |---|---|
  | `check_signature` call removed | 11 cases |
  | `version_ge` inverted | 6, 13 |
  | require-refusal removed | 10 |
  | verify forced true | 2, 3, 12, 13 |
  | absent-note suppressed | 8 |
  | upper range bound ignored | 12 |
  | only the first key tried | 13 |

- **Also:**
  - `install-assets.test.sh` (network) now asserts that a pinned release carries `<sums>.minisig` once `first_signed` says it should. It is a no-op today (0 checked); a direct probe confirmed `mk-cli-v0.13.0/SHA256SUMS.x86_64.minisig` is a 404, so the check would fire.
  - `install-verify` and `install-msrv-guard` still pass.
  - shellcheck 0.11.0 is clean on install.sh and on the new test.

## Build gate on the workflows (before any CI)

- actionlint is clean on every changed workflow.
- I extracted every new sign/verify/policy `run:` block from the YAML and executed it against throwaway keys: 100/100 checks passed.
  - Cases: secret present; absent on a tag; absent on dispatch; encrypted key without and with its password; wrong pinned key; tampered sums; tarball missing; publish/dry-run resolution.
  - It also confirmed that the tarball, sums file and provenance stay byte-identical across signing, and that the temp key file is gone afterwards.
- The engrave generator (`scripts/release-workflows/gen.py` + `emit.py`) was regenerated into a scratch dir. It is **byte-identical** to the toolkit, ms, md and mk `release.yml` on their branches.
  - Before this work md had already drifted from the generator by 9 lines (F-676's `--features cli-compiler` and the `--from-policy` smoke check). Those are now generator inputs, so regenerating no longer regresses md.

## Blocked — needs the user

- **Engrave trust anchor.** The classifier denied the edit to `minisign.pub` (new key, id EF2B8D34D8409754), the README's "Verifying releases" key block (new key, plus an old-key note for ≤ v0.12.0), and engrave `release.yml`, where I would have moved the key into one `env: MINISIGN_PUBKEY`, substituted it into VERIFY.txt, and added a check that `minisign.pub` equals the pin. I did not retry.
  - Until someone makes that change, engrave's `sign-releases` `release.yml` verifies against the **old** key. With the new secrets its dispatch proof would fail at verify, so I did **not** run it.
  - A classifier-denied search also means I could not grep engrave for tests referencing `minisign.pub`. Run engrave's suite after that edit.
  - After the edit: `gh workflow run release.yml --repo bg002h/mnemonic-engrave --ref sign-releases`.

## Findings and follow-ups (not done here)

1. **`rust.yml` never ran the installer harnesses on an installer-only push.** Its `paths:` filter named only `crates/`, `Cargo.*` and `rust.yml`. Fixed in b3affa91 by adding `scripts/install.sh` and `scripts/install-*.test.sh`. Pre-existing gap; it surfaced because `rust.yml` did not start on the first ci/ push.
2. **mnemonic-transaction** carries the same dormant signing step but has no signing secret. It is outside the brief's six repos and I did not touch it. `emit.py` now generates the signing policy, so regenerating mt's `release.yml` would make its next tag **fail** until mt gets the secrets. Its committed file lags the generator until then.
3. **`install-verify.test.sh` will break when `first_signed` for mk is filled in.** Its fixtures have no `.minisig`, so a pin at or after the entry is refused (correctly). The fix is to have `fresh_release` sign with a throwaway key and rewrite the table, the way `install-signature.test.sh` does. Owner: whoever fills in the first `first_signed` entry.
4. **Not updated:** `docs/verify-reproducibility.md` and the ms/md/mk/GUI READMEs still describe `sha256sum -c` only. They should add the `minisign -Vm … -P <new key>` line once the first signed releases exist.
5. The md commit message says the musl legs "already built on workflow_dispatch". True, but they now also sign and verify there.
6. Secret handling (non-gating per the 2026-08-27 ruling): the key is written to a `mktemp` file (0600) and removed by an EXIT trap or an `always()` step. It never enters a container.
