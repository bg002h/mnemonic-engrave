# Adversarial review: mnemonic-toolkit branch `f675-f677`

- **Reviewer:** Opus 5.5 (independent context)
- **Date:** 2026-09-24
- **Tree:** `/scratch/code/shibboleth/tk-worktrees/f675-677`, HEAD `d3f043e7` (4 commits over origin/master: `327f9a16`, `9b04e874`, `8cf7759d`, `d3f043e7`)
- **Question:** can this branch ship? Above all, can the installer put a binary on a user's machine that the pinned release did not publish, while reporting success?

## Answer to the main question

**No such path found.** The binary path fails closed in every case I built: a digest mismatch, a sums file that does not list the asset or names it differently (`./name`, CRLF), no sums file, a truncated download or a 404, zero binaries, two binaries, a symlink named `mk`, `..` traversal, a symlink escape, a wrong version, the right version with a non-zero exit, and a `-rc1` suffix. Each one exits non-zero with nothing installed. Ctrl-C during the download or the copy never leaves a `mk` behind.

Every platform mapping is correct today. I downloaded all 34 mapped assets and ran `file` on each binary: every one has the right OS and architecture, and every Linux CLI is static musl. Two things can report success without installing the release binary:

- the source fallback, which is designed and settled;
- a pre-existing directory at `<root>/bin/<bin>` (N1), which is a Nit.

**What blocks shipping is one Important usability defect (I1), not safety.** The default one-liner refuses the GUI on the common LTS distros, and the fix the installer prints then fails.

## Critical

None.

## Important

### I1. On glibc < 2.39 the GUI is refused, and the installer's printed fix then fails for everything the same run installed

**The GUI needs a newer glibc than anyone checked.** The x86_64 glibc GUI asset needs **GLIBC_2.39**. I measured this with `objdump -T` on `mnemonic-gui-v0.59.0-x86_64-linux.tar.gz`; the implementer measured only md's 2.34. The pre-install run check therefore refuses the GUI on:
- Debian 12 (glibc 2.36)
- Ubuntu 22.04 (2.35)
- RHEL 9 (2.34)

On glibc < 2.34 it refuses md as well: Debian 11, Ubuntu 20.04, RHEL 8.

**The default install exits 1 on those systems.** The README's default one-liner installs all five. After the refusal it prints "4 installed, 1 failed" and exits 1. The refusal text says:

```
(it does not run on this host, or is not the pinned version;
--from-source builds the pinned tag instead)
```

**Following that advice fails.** It fails for every component the first run already installed. Reproduced in `--root <scratch>/src-root`:

```
$ dash scripts/install.sh --only mk --no-man --root R            # binary install, rc=0
$ dash scripts/install.sh --only mk --no-man --root R --from-source
install  mk (source: mk-cli-v0.13.0)
error: binary `mk` already exists in destination
Add --force to overwrite
  FAILED
0 installed, 1 failed.                                            # rc=1
```

cargo does not track a file the binary path copied into place, so it refuses to overwrite it. The quickstart's "add `--from-source` to that command" breaks the same way for anyone who ran the binary command first.

- **Classification:** refusal with a wrong remedy. It is loud and fail-closed, so no funds or safety risk.
- **Fix, small:**
  - make the refusal print `--from-source --force --only <name>`;
  - say in the manual and README that the prebuilt Linux GUI needs glibc ≥ 2.39, and md ≥ 2.34.

## Minor

### M1. With BusyBox `wget` and no `curl`, every download fails as "download failed"

`wget -q --https-only` is not a BusyBox option:

```
$ busybox wget -q --https-only -O x https://github.com/
wget: unrecognized option '--https-only'   (rc=1)
```

Stock Alpine has only BusyBox wget, and Alpine is the musl target. The documented one-liner needs `curl` to fetch the script at all, so only a user who got the script another way reaches this. It fails closed, but the message points at the network rather than the flag.

### M2. An unusable `TMPDIR` makes the working directory `/<component>`, run with `rm -rf` and `mkdir -p` at the filesystem root

`install_binary` runs in an `elif` condition, so `set -e` is off inside it. The `mktemp -d -t mnemonic-install` fallback is BSD-only syntax: GNU mktemp fails with "too few X's". Reproduction:

```
$ TMPDIR=/nonexistent-dir dash -x scripts/install.sh --only mk ...
+ mktemp -d
+ mktemp -d -t mnemonic-install
mktemp: too few X's in template 'mnemonic-install'
+ TMP_ROOT=
+ _w=/mk
+ rm -rf /mk
+ mkdir -p /mk/x
```

As a normal user this just fails. As root (sudo, or a container) it deletes and then recreates `/mnemonic`, `/md`, `/ms`, `/mk` and `/mnemonic-gui`, and leaves them behind, because `cleanup` skips an empty `TMP_ROOT`. Fix: refuse when `TMP_ROOT` is empty, and use a portable fallback such as `mktemp -d "${TMPDIR:-/tmp}/mnemonic-install.XXXXXX"`.

### M3. Tests that cannot fail: 14 mutations survived

Method: each mutation was applied to a copy of `scripts/` or `ci/repro/`, the pattern was asserted to occur exactly once, and the named test was run with dash as `sh`. The drivers were `review-f676-scratch/mutate.py` and `mutate_repro.py`, since deleted.

**Survived:**

| # | Mutation | Test run | Result |
|---|---|---|---|
| X1 | the "exactly one binary" count check is removed | install-verify | SURVIVED: no case has two binaries. Two binaries are still refused further down, where the run check fails. |
| X3 | the version check becomes a prefix match | install-verify | SURVIVED: a binary printing `mk 0.13.0-rc1` would pass |
| X4 | the Windows run-check skip is widened to every Windows binary | install-verify | SURVIVED: Windows is untested |
| X5 | `trap cleanup EXIT INT TERM` restored: the draft bug the implementer fixed as #4 | install-verify | SURVIVED: no signal test pins it |
| X6 | the atomic copy-then-rename becomes a direct copy | install-verify | SURVIVED |
| X7, X12 | `curl -f` dropped; `wget --https-only` dropped | install-verify | SURVIVED: the stub ignores flags. The checksum still catches a 404 body. |
| X9 | an unpack failure is ignored (`tar … \|\| true`) | install-verify | SURVIVED: no bad-archive case |
| X13 | sums matched by substring instead of exact name | install-verify | SURVIVED |
| X14 | macOS arm64 CLIs mapped to the amd64 asset | install-assets (network) | SURVIVED |
| X15 | x86_64 GUI glibc and musl assets swapped | install-assets (network) | SURVIVED |
| X16 | Darwin detection hard-coded to x86_64 | install-verify | SURVIVED: detection is tested for Linux only |
| R1, R1b | `cc-validate.sh` residue check inverted, or PATHS residue ignored | residue.test | SURVIVED: `residue.test.sh` never invokes `cc-validate.sh`. The commit calls this site the false-GREEN one, which is the more dangerous direction. |
| VF-H | the hermetic `CARGO_HOME` removed from vendor-freshness check (1) | vendor-freshness.test | SURVIVED: 7/7 still pass |

Notes on the survivors:
- **X14 and X15 are the brief's "wrong mapping that still passes its checksum".** `install-assets.test.sh` proves only that the asset exists and one sums file lists it. It never checks that the asset suits the platform. The run check would not catch either swap: Rosetta runs amd64 on arm64, and the static musl GUI runs `--version` on glibc. The current table is correct (measured by `file`), so this is a gate gap, not a live defect.
- **VF-H contradicts a comment.** `vendor-freshness.sh` says "The mutation tests … pin this", but they do not. Every `Cargo.lock` git source now gets a stanza, and an ungrounded source fails before check (1), so the hermetic home is defence in depth that no test can observe.

**Killed:**
- X10: `--version` stderr dropped
- X11: exit 0 on failure
- R2: `residue_hits` swallows a grep error
- R3: remap-off back to the racy `grep | head | grep -q`; the heavy cases went red
- R4: exclude pattern ignored

### M4. On musl hosts the installed GUI cannot open an X11 display, yet it counts as installed

The musl GUI asset is static-pie. Measured on the real asset:

```
$ env -u WAYLAND_DISPLAY DISPLAY=:987 ./mnemonic-gui
Error: WinitEventLoop(... XNotSupported(LibraryOpenError(... "opening library failed (Dynamic loading not supported)" ...)))
```

It passes `--version`, so the installer reports it installed. The old installer built the GUI from source on musl, which gave a dynamic musl build. This is an upstream artifact and a niche platform, but it is a regression for Alpine or Void X11 desktops.

### M5. Nothing warns when `<root>/bin` is not on PATH

The new default audience has no Rust toolchain, so `~/.cargo/bin` is usually not on their `PATH`. The closing "verify: mnemonic --version" then gives "command not found". The docs say to add it to `PATH`, but the installer does not check, for example with `case ":$PATH:" in *":$BIN_DIR:"*`.

### M6. On x86_64 musl without cargo, nothing installs

The early cargo check exits 1 before installing any of the other four components ("md must be built from source … cargo is not on PATH"), and the message does not suggest `--exclude md`. This is adjacent to F-678.

## Nit

- **N1.** If `<root>/bin/mk` is a directory, `mv -f` moves the file into it, as `bin/mk/.mk.partial.<pid>`. The run still prints "installed …/bin/mk" and exits 0.
- **N2.** Ctrl-C during the copy leaves a hidden `<root>/bin/.mk.partial.<pid>`. It exits 130 and the temp dir is cleaned.
- **N3.** `--root ""` (an empty variable) silently falls back to `~/.cargo`.
- **N4.** The Windows GUI binary is a console-subsystem PE32+ (`file`), so the reason given for skipping the run check ("output unverified") is probably unnecessary.
- **N5.** When a sums file lists the asset twice with different digests, the first entry wins silently. It fails closed if that first entry is wrong.
- **N6.** A binary install leaves cargo's `.crates.toml` untouched, so a previous crates.io `md-cli` entry stays. `cargo install --list` then misreports it, and `cargo install-update -a` could later replace the pinned binary with the older crates.io one.
- **N7.** Pre-existing, not in this diff: `man-pages.yml:132` still hard-codes `miniscript_rev`, which is the drift class `repro-drift.yml` just fixed.

## Verified, and holding

- **shellcheck 0.11.0** on `scripts/install.sh`: clean.
- **Harnesses** under dash (`sh` → dash on PATH, so `install.sh` itself runs under dash), BusyBox sh and yash: `install-verify.test.sh`, `install-msrv-guard.test.sh` and `install-man-step.test.sh` all OK. `install-assets.test.sh` gave 34/34.
- **Real end-to-end install** under dash, `--root` with a space in the path: 5 installed at mnemonic 0.104.0, md 0.20.3, ms 0.19.0, mk 0.13.0 and mnemonic-gui 0.59.0, with digests matching the implementer's; 79 man pages; the temp dir removed. `md encode --from-policy 'pk(@0)'` works, which confirms cli-compiler.
- **BusyBox userland** with curl (BusyBox tar, awk, find, sha256sum, mktemp; no getconf or ldd): 5 installed.
- **Fold claims re-run:**
  - md-codec grounding: 395 of 396 blob hashes match `cf35d61a:crates/md-codec`, and the only difference is `Cargo.toml`. That commit is tag `descriptor-mnemonic-md-cli-v0.19.0`.
  - The `repro-drift.yml` `pin` step, extracted and run: it emits miniscript `ff4732e5` plus descriptor-mnemonic `cf35d61a`.
  - `vendor-freshness.test.sh` 7/7; `residue.test.sh` 14/14. The old shape gave a false "no residue" in 20 of 20 runs here.
  - `ci/doc-gate-guard.test.sh` 88/0.
  - Every line of the Examples golden's install output reproduces from `install.sh`.
- **Rust:** `cargo nextest run --locked --workspace` gives 4057 passed, 20 skipped. The new `verify-bundle --help` text for `--accept-search-time` is correct, and the flag is routed into restore's context (`verify_bundle.rs:960`).
- **`git status`** in the worktree is clean after restoring the one in-place mutation (VF-H). Scratch is deleted with `find -delete`.

ready to ship: no
