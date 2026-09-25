# F-676 fold 1: md pin 0.20.3, cli-compiler caveat removed, GUI pin decision

- **Implementer:** Opus 5.5
- **Date:** 2026-09-24
- **Worktree:** `/scratch/code/shibboleth/tk-worktrees/f675-677`, branch `f675-f677`
- **Commit:** `d3f043e7` "F-676 fold 1: md pin 0.20.2 -> 0.20.3; drop the cli-compiler caveat". It sits on `8cf7759d`, which is unchanged.
- **Not pushed, tagged or merged.**

## 1. md pin 0.20.2 → 0.20.3

I first checked the release myself:
`gh release view descriptor-mnemonic-md-cli-v0.20.3` lists the same asset
shape as 0.20.2. It has `SHA256SUMS.{aarch64,portable}` and no
`SHA256SUMS.x86_64` or x86_64-musl asset (that gap is F-678).

**The tag moved everywhere sibling-pin-check scans.** The installer is not
the only place it lives: four workflows carried it too, and the check
requires them to equal the installer's pin.
- `scripts/install.sh`
- `.github/workflows/manual.yml`
- `.github/workflows/quickstart.yml`
- `.github/workflows/technical-manual.yml`
- `.github/workflows/cross-tool-differential.yml`

Other strings that moved:
- `ci/doc-flag-lint.test.sh` R3 fixture, `(unreleased: md-cli after 0.20.3: …)`.
  It has to name the pinned version, or the mutation lands on the
  "does not name the pinned release" path instead of the stale-marker path it
  is meant to test.
- The manual's crates.io comparison: "the 0.20.3 documented here".
- The CHANGELOG pin list.
- `.examples-build/Examples.md`, regenerated in the same commit. The golden
  diff is exactly the md tag and asset lines plus the removed note.

The manual's `--from-source --dry-run` line prints its tags at run time, so
it follows the pin with no edit.

**Remaining "0.20.2" strings, and why they stay:**
- `CHANGELOG.md:167`, the line that records the bump.
- `vendor/bitcoin` (×2) and `vendor/secp256k1`: these are
  `bitcoinconsensus 0.20.2-0.5.0` and the secp256k1 0.20.2 changelog, and
  have nothing to do with md.
- `design/` has no hits.

## 2. cli-compiler caveat removed

Removed from:
- `install.sh`: the `binary_gap` function, its two call sites, and the help
  text lines
- the harness: `install-verify.test.sh` case 13
- `docs/manual/src/20-quickstart/21-install.md`: the paragraph
- `CHANGELOG.md`: the bullet, replaced with the pin-bump rationale

A grep for `binary_gap`, `cli-compiler feature` and `without the cli-compiler`
across scripts, manual, quickstart, README, CHANGELOG and gen.sh finds
nothing.

**Proof through the installed binary.** I ran it against
`<scratch>/root/bin/md` as installed by `install.sh`:
```
$ root/bin/md --version
md 0.20.3
$ root/bin/md encode --from-policy 'or(pk(@0),and(pk(@1),older(144)))' --context segwitv0 --group-size 0
md1ypqqnq5fnf2uqqqqpyq8y3frmejdncm8
(rc=0)
```
This is the same string the controller measured.

## 3. GUI pin: kept at `mnemonic-gui-v0.59.0`

**Where v0.59.0 came from.** It was set in `de140a08` (2026-07-12) with the
message "fix(install.sh): GUI pin v0.58.0 → v0.59.0 (lockstep — badge now
released)". The draft and I both inherited it and did not re-choose it.

**Why I did not move to v0.61.0.** This repo's records do not show v0.61.0
working with these CLI versions:
- **Its own pins are older.** `git show mnemonic-gui-v0.61.0:pinned-upstream.toml`
  pins toolkit v0.97.0, md-cli v0.11.0, ms-cli v0.13.0 and mk-cli v0.11.0.
  v0.59.0 pins toolkit v0.75.0 and the same three older CLI tags.
- **The only in-repo record of v0.61.0 running** is
  `design/agent-reports/install-2026-08-31-constellation.md`, which installed
  it via `cargo install --tag mnemonic-gui-v0.61.0` and saw
  `--version → mnemonic-gui 0.61.0`. That was next to md 0.14.0, ms 0.16.0
  and mnemonic 0.97.0, not the current md 0.20.3, ms 0.19.0 and
  mnemonic 0.104.0. It is also a `--version` smoke test, not a compatibility
  run.
- **No other evidence.** `design/FOLLOWUPS.md` and
  `docs/manual-gui/pinned-upstream.toml` (which pins v0.57.0) say nothing
  about v0.60 or v0.61 compatibility.

**Caveat on keeping v0.59.0.** Its own compatibility with these CLIs is not
evidenced either. It is simply the pin this repo already shipped. What is
measured is that its binary is verified against the published digest, and it
installs and runs `--version` at 0.59.0.

The three-way disagreement (installer v0.59.0, latest release v0.61.0, GUI
manual v0.57.0) is still open. It needs a GUI-side compatibility check before
any bump. I did not touch the GUI-owned manual pages.

## 4. Gates re-run on the fold

| Gate | Command | Result |
|---|---|---|
| Harnesses (shellcheck 0.11.0 on PATH) | `sh scripts/install-{msrv-guard,man-step,verify,assets}.test.sh` | all rc=0; shellcheck clean; man-step canary 39 pages / 0 help pages; install-verify 14 ok (13 + 1 banner); install-assets 34/34 |
| Mutations | `mutate.py` from the implementation report, minus M15/M16, whose target `binary_gap` no longer exists | **14/14 killed** |
| Real install | `sh scripts/install.sh --root <scratch>/root --man-dir <scratch>/man` | rc=0, 5 installed, 79 man pages; versions below |
| Tamper refusal | `curl` wrapper flips one byte of `md-*.tar.gz`; `--only md,mk` | rc=0 for this check (details below) |
| Platform URLs and sums | for 7 platforms, every `download` URL from `--dry-run` | **34/34 return 200 and are listed in a sums file, 0 bad** |
| md 0.20.3 assets | all 6 downloaded; `sha256sum -c` against both sums files | 6/6 OK; each archive holds exactly one `md` / `md.exe` |
| Examples golden | regenerated and committed in `d3f043e7` | diff limited to md 0.20.3 lines and the removed note |
| Manual | `make audit` (md/ms/mk = the installed release binaries, mnemonic = `target/debug`) | rc=0 |
| Doc CI checks | `ci/doc-flag-lint.test.sh`; `ci/doc-gate-guard.test.sh`; quickstart `make lint` and `make verify-examples` | all rc=0 |
| sibling-pin-check | step script extracted from the yml | rc=0, 11 OK lines, no warnings |
| install-pin-check | self-pin | `mnemonic-toolkit-v0.104.0` |
| Rust | `rustup run 1.95.0 cargo fmt --all -- --check`; 1.85.0 `cargo clippy --locked --all-targets -- -D warnings`; `cargo nextest run --locked --workspace` | rc=0; rc=0 (clippy 0.1.85); 4057 passed, 20 skipped |

Real install results:
- `mnemonic` 0.104.0, verified against `SHA256SUMS.x86_64`
- `md` 0.20.3, `31b1e6f0…e65017` against `SHA256SUMS.portable`
- `ms` 0.19.0 and `mk` 0.13.0, against `SHA256SUMS.x86_64`
- `mnemonic-gui` 0.59.0, against `SHA256SUMS`

Tamper refusal: the installer printed the following, then reported
"1 installed, 1 failed", and `bin/` contains only `mk`.
```
error: REFUSED md-0.20.3-linux-amd64.tar.gz: sha256 mismatch against SHA256SUMS.portable
       published 31b1e6f04e1e829929444489b8343fe638a7fd8d5523eaea756d393b60e65017
       download  13382f01f2e09828b3c590580298aabb9828da54ecb72bfcdcf5527607f66d79
```

Platform results:

| Platform | Downloads | Source builds |
|---|---|---|
| linux-x86_64-gnu | 5 | 0 |
| linux-x86_64-musl | 4 | 1 (md, as designed; F-678) |
| linux-aarch64-gnu | 5 | 0 |
| linux-aarch64-musl | 5 | 0 |
| macos-x86_64 | 5 | 0 |
| macos-aarch64 | 5 | 0 |
| windows-x86_64 | 5 | 0 |

## Housekeeping

Both scratch directories were removed with `find <dir> -delete`: the old
`tk-worktrees/f676-scratch` (515 MB) and this fold's
`tk-worktrees/f676-fold1-scratch` (81 MB). `ls` confirms neither exists. The
worktree's `git status` is clean.
