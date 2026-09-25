# F-676 implementer report (resume after the power outage)

- **Implementer:** Opus 5.5
- **Date:** 2026-09-24
- **Worktree:** `/scratch/code/shibboleth/tk-worktrees/f675-677`, branch `f675-f677`
- **Commit:** `8cf7759d` "F-676: install.sh installs pinned release binaries (sha256-verified), never crates.io". It sits on `9b04e874` / `327f9a16`, which are untouched. 22 files, +979/-311.
- **Not pushed, tagged or merged.**

## Verdict

I kept the design. `install.sh` installs each of the five components from the
prebuilt binary attached to its pinned GitHub release. Each download is checked
against that release's SHA-256 checksum file. `--from-source` builds the same
pinned tag with `cargo install --locked --git … --tag …`, and so does any
platform that has no release binary. Nothing installs from crates.io.

The design has one real gap, which I measured. **The md 0.20.2 release binary
is built without `--features cli-compiler`.** descriptor-mnemonic's
`release.yml:139` runs `cargo build --release --locked --target … -p md-cli`
with no features. As a result, `md encode --from-policy` refuses with
"requires the cli-compiler feature". The manual and quickstart mention
`--from-policy` 8 times, and the old installer passed that feature. I did not
switch md to source builds, because that would bring back the toolchain
requirement for every user. Instead the gap is stated in three places: the
installer prints a note after installing md, the manual chapter says it, and
the CHANGELOG records it. The recipe the note gives (`--from-source --only md`)
was run and produces a working `--from-policy`.

**The proper fix is upstream, and it is a controller decision.** It is a
one-line change to descriptor-mnemonic's release workflow to add
`--features cli-compiler`, followed by a new md release and a pin bump. After
that, delete the md arm of `binary_gap` in `install.sh`.

## What I kept from the draft, and what I changed

The draft was sound. I kept all of the following:
- the component and asset tables
- `SUMS_FILES` with first-exact-match semantics
- the fetch → verify → extract → exactly-one-binary → atomic-rename install
- the `--from-source` / `--from-git` handling, the no-binary fallback and `--root`
- the scoping of the MSRV guard to source builds of the GUI
- the draft's three test files and the `rust.yml` steps

Fixes and additions:
1. **Shellcheck would have turned CI red.** CI's ubuntu runner has
   shellcheck, and `install-man-step.test.sh` runs it on `install.sh`. The
   draft had three findings (SC2018/SC2019 on `tr 'A-Z'`, and SC2015 on the
   `cp && chmod && mv || …` chain). All are fixed. Checked with shellcheck
   0.11.0 from nix; the tool is not installed locally.
2. **Anchor-check would have turned CI red.** The draft removed the manual's
   `#manual-coverage` link but did not update
   `docs/manual/tests/anchor-dangler-baseline.txt`. I removed that entry.
3. **Libc detection.** The draft classified any host with `/lib/ld-musl-*` as
   musl. Debian's musl package installs that loader on glibc hosts, which
   would have given the GUI a musl build and pushed md into a source build.
   Detection now checks `getconf GNU_LIBC_VERSION` first.
4. **Signal handling.** The draft's `trap cleanup EXIT INT TERM` ran the
   cleanup on Ctrl-C and then kept going. EXIT now does the cleanup, and
   INT/TERM exit with 130/143.
5. **New: a pre-install run check.** After verification and extraction, the
   installer runs `<bin> --version` and refuses the binary unless the first
   line is exactly `<bin> <pin>`. The reason is measured: md's
   `linux-amd64` build is glibc-dynamic and needs `GLIBC_2.34`
   (`objdump -T`), so an older host would otherwise get an installed md that
   cannot run. The check is skipped only for the Windows GUI build, whose
   `--version` behaviour I could not verify. The dry-run output matches this.
6. **New: `binary_gap` note** for md's missing `cli-compiler` (see above).
7. **Examples golden determinism.** `gen.sh` now pins
   `MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu` and unsets `CARGO_HOME`. The
   draft's new default root reads `CARGO_HOME`, which GitHub runners set.
   Without this, the golden would have carried runner paths and the
   build host's platform. The prose and flag list are updated too.
8. **Docs the draft missed, each still describing the cargo or crates.io path:**
   - `README.md`, where the install section claimed `cargo install --git --tag`
     and that `cargo` was required
   - `crates/mnemonic-toolkit/README.md`
   - `docs/quickstart/src/20-singlesig/21-install.md`, which had tagless
     `cargo install --git` lines (those build master)
   - the three `docs/manual-gui/src/20-install/2{1,2,3}-*.md` pages, which
     said `cargo install --locked md-cli` / `ms-cli` / `mk-cli`, the F-676
     defect itself
   - "after `cargo install`" wording in manual chapters 41–44
   - two stale comment references in the workflows (`<bool>` field;
     `install.sh:44`)
9. **Manual chapter.** I replaced the draft's four hand-typed
   `cargo install … --tag` lines with the installer's own
   `--from-source --dry-run` command, which prints them. The reason: the
   toolkit line carries `mnemonic-toolkit-v0.104.0`, and sibling-pin-check
   excludes the toolkit, reporting only a `::warning::`. That line would have
   drifted silently at the next release. "Path A′" became a subsection, to
   avoid a prime character in a heading and to keep the "Path A or Path B"
   cross-reference valid.
10. **Tests.** The draft already added `install-verify.test.sh` with 9 cases.
    I added cases 10–13 (wrong version, unrunnable binary, the md note, libc
    detection) and made both new test files executable.
11. **CHANGELOG** `[Unreleased]` entry.

## Gates, with commands and output

**1. Harnesses, each proven able to fail.** On the final tree, each of
`sh scripts/install-{msrv-guard,man-step,verify,assets}.test.sh` returned
rc=0, with shellcheck on PATH:
```
  ok   shellcheck clean
  ok   39 pages, 0 *-help*.1 shadow pages
[install-verify.test] … 15 ok lines … OK
  ok   34 of 34 mapped assets verified
```

**Mutations.** The script is
`/scratch/code/shibboleth/tk-worktrees/f676-scratch/mutate.py`. It copies
`scripts/`, asserts that each pattern occurs exactly once and actually
changed, and requires the named test to go red. **Result: 16/16 killed.**

| # | Mutation | Test that caught it |
|---|---|---|
| M1 | sha-mismatch check removed | verify |
| M2 | no-checksum refusal removed | verify |
| M3 | `*name` sums form not honoured | verify |
| M4 | run check removed | verify |
| M5 | musl-loader-wins detection | verify |
| M6 | fallback note removed | verify |
| M7 | crates.io source path reintroduced | verify |
| M8 | `--from-source` ignored | verify |
| M9 | MSRV guard binds to the prebuilt GUI | msrv-guard |
| M10 | md glibc mapped to a non-existent asset | assets |
| M11 | md musl x86_64 given the glibc binary | assets |
| M12 | GUI Windows mapping dropped | assets |
| M13 | `SHA256SUMS.portable` dropped from the candidates | assets |
| M14 | install before verification | verify |
| M15 | md note removed | verify |
| M16 | md note printed for source builds | verify |

M6 survived on the first try. That first mutation only changed the `note:`
prefix and left the asserted text in place, so it was semantically inert. I
re-ran it as a real deletion and it was killed.

**2. Real end-to-end on this box (x86_64 glibc 2.44).** The command was
`sh scripts/install.sh --root …/e2e/final --man-dir …/e2e/finalman` (rc=0,
"5 installed"). Output:
```
mnemonic 0.104.0 / md 0.20.2 / ms 0.19.0 / mk 0.13.0 / mnemonic-gui 0.59.0
verified 7809bf4c… (SHA256SUMS.x86_64), 874594… (SHA256SUMS.portable), 7ef951…, ab2c8d…, 446bd6… (SHA256SUMS)
```
79 man pages were written (md 16, mk 11, mnemonic 39, ms 13). Two source
builds, both rc=0:
- `--from-source --only mk` → `mk-cli v0.13.0 (…?tag=mk-cli-v0.13.0#0feaaaa9)`, `mk 0.13.0`
- `--from-source --only md` → `md 0.20.2`, and `md encode --from-policy 'pk(@0)' --context segwitv0` → `md1yqqq3gfnmj0kk9j2lh0`

**3. Real refusal.** A `curl` wrapper fetched the real assets and then
flipped one byte in the mk tarball. The run was `--only ms,mk`. Result: rc=1,
"1 installed, 1 failed", and `bin/` contains only `ms`:
```
  error: REFUSED mk-0.13.0-x86_64-linux-musl.tar.gz: sha256 mismatch against SHA256SUMS.x86_64
         published ab2c8dad89f2fa9c2fc13880a0df5c8b293b4edce457a147332201c62b2e29da
         download  a91e365be7c08d280336afeb7e8d0b4c550e7571a58ea02279be5e30d9566d20
```

**4. Other platforms.** For each platform, every `download` URL printed by
`MNEMONIC_INSTALL_PLATFORM=<p> install.sh --dry-run` was resolved with
`curl -sIL` and looked up in the release's sums files. **All 34 URLs returned
200, and each is listed in a sums file** (0 bad):

| Platform | Downloads | Source builds |
|---|---|---|
| linux-x86_64-gnu | 5 | 0 |
| linux-x86_64-musl | 4 | 1 (md, as designed) |
| linux-aarch64-gnu | 5 | 0 |
| linux-aarch64-musl | 5 | 0 |
| macos-x86_64 | 5 | 0 |
| macos-aarch64 | 5 | 0 |
| windows-x86_64 | 5 | 0 |

In addition, I downloaded all 27 unique mapped assets. `sha256sum -c`
against the published sums reported OK for all 27. Each archive contains
exactly one `<bin>` or `<bin>.exe`, including the `./`-prefixed macOS tars
and the flat Windows zips.

Which sums file lists which asset, measured rather than inferred:
- mnemonic, ms, mk: `.x86_64` lists x86_64 musl; `.aarch64` lists aarch64
  musl; `.portable` lists macOS ×2 and Windows.
- md: `.aarch64` lists aarch64 musl; `.portable` lists linux-amd64,
  linux-arm64, macOS ×2 and Windows.
- GUI: a single `SHA256SUMS` lists all 7.

**5. Validation surface.** Every row below returned rc=0 or its equivalent.

| Gate | Command | Result |
|---|---|---|
| fmt | `rustup run 1.95.0 cargo fmt --all -- --check` | rc=0 |
| clippy | 1.85.0 toolchain bin prepended; `cargo clippy --locked --all-targets -- -D warnings` | rc=0 (`clippy 0.1.85`) |
| tests | `cargo nextest run --locked --workspace` | 4057 passed, 20 skipped |
| Examples golden | `gen.sh` regenerated, as `examples.yml` does | byte-identical to the committed file; also identical under a hostile environment (`CARGO_HOME`, `CARGO_INSTALL_ROOT`, `XDG_DATA_HOME` set, `MNEMONIC_INSTALL_PLATFORM=macos-aarch64`) |
| manual | `make audit` (CI's pinned binaries: md = source build with cli-compiler, ms/mk = release) | rc=0 |
| flag lint | `ci/doc-flag-lint.test.sh` | rc=0 |
| guard self-test | `ci/doc-gate-guard.test.sh` | rc=0 |
| quickstart | `make lint`; `make verify-examples` | rc=0; rc=0 |
| manual-gui | `make html`; `make lint` with `MANUAL_GUI_UPSTREAM_ROOT` = a clone of `mnemonic-gui-v0.57.0`, as CI does | rc=0; rc=0 |
| document builds | manual and quickstart `make pdf` / `make html` | all rc=0 |
| sibling-pin-check | step script extracted from the yml and run | "OK all sibling pins match" |
| install-pin-check | self-pin grep | `mnemonic-toolkit-v0.104.0` |
| g6-invariant | pin parse | `ms-cli-v0.19.0` |

Not run locally: miri, `musl-build-test` (cross/QEMU), the release-profile
mlock test, and the `lib-cross-platform` check. No Rust source changed.

## Concerns for the controller

- **The md `cli-compiler` gap** described above. It needs an upstream md
  release fix, filed Rust-primary in descriptor-mnemonic.
- **GUI pins disagree.** `install.sh` pins `mnemonic-gui-v0.59.0`, while v0.61.0
  exists (gui-pin-drift-check warns only) and `docs/manual-gui/pinned-upstream.toml`
  pins v0.57.0. I kept v0.59.0 as out of scope. All its assets verified.
- **Windows is claimed but unverified.** The installer's Windows path (Git
  Bash with `unzip`) has never run on Windows. Only its URLs and sums are
  verified. The GUI manual's Windows page now points users at it through Git
  Bash.
- **The GUI manual's own "install mnemonic-gui" paths** are still tagless
  `cargo install --git` (lines ~55 in the linux, macos and windows pages). I
  left them as GUI-owned, not F-676.
- **Manual now depends on a merge.** The manual's `--from-source --dry-run`
  recipe fetches `install.sh` from master, so it is only true after this
  merges. The same holds for the quickstart and README one-liners.
- **Checksums are not signatures.** They prove integrity against the
  release, not origin. This is stated in the installer, manual and README.

## Housekeeping

- **A stray-file slip in the controller's repo.** A denied `mkdir` meant my
  first sums download ran in the mnemonic-engrave repo root. That created
  five untracked directories (`mnemonic-toolkit/`, `descriptor-mnemonic/`,
  `mnemonic-secret/`, `mnemonic-key/`, `mnemonic-gui/`), all containing only
  SHA256SUMS files. I moved them to scratch at once, and `git status` there
  is back to its prior state.
- **The scratch directory was not deleted.**
  `/scratch/code/shibboleth/tk-worktrees/f676-scratch/` (515 MB: assets,
  E2E roots, source target dir, logs, `mutate.py`) is still there. The
  `block-rm-rf` hook refuses recursive deletion, so it needs a manual removal.
- **Papercut logged** in `/scratch/code/papercuts.md`: the Bash tool's shell
  is zsh, so an unquoted `$VAR` is not word-split.
