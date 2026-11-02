# SPEC: `cut-release.sh`, one release script for every constellation repo

Status: DRAFT for R0 review (2026-11-01). Author: Opus spec thread.
Builder: Sonnet. The builder decides only how the code is written. Every
operator-visible text, refusal, exit code and test row is fixed here. On any
point this document does not decide, the builder STOPs and asks.

Sources: `design/CONTINUITY_2026-09-26_release-script.md` (the seven hand
steps), `design/agent-reports/release-ms020-tk0105.md` (toolkit two-commit
precedent, in THIS repo), the coordinator's
`agent-reports/release-ms-0.20.2.md` (the 2026-11-01 hand release, 55 min,
317k tokens), and the coordinator's rulings of 2026-11-01 (§1).

## 1. Rulings this spec implements (settled; do not re-derive)

1. **No prose at release time.** The release notes are written and reviewed
   in the fix PR, under the crate's `[Unreleased]` heading (§3 column U). The
   script refuses when that section is empty, and it moves the section under
   the new version heading itself. The script never writes prose.
2. **No local suite.** The release commit holds only versions, the lockfile,
   the CHANGELOG heading move and (toolkit only) version markers. CI on that
   exact SHA, through `push-via-staging.sh`, is the gate. The local checks are
   `cargo build --locked --workspace`, a `Cargo.lock` diff limited to the
   bumped crates' version lines, and `--version` of the built binary.
3. **`push-via-staging.sh` is called, never re-implemented.** It discovers
   the required contexts itself (ms has six, not the four its CLAUDE.md
   names). No context list appears anywhere in this script.
4. **One commit can carry several tags** (ms 2026-11-01: `ms-codec-v0.10.1`
   and `ms-cli-v0.20.2` both on `62b4d9d`).
5. **Tags are pushed by full ref, never `--tags`.** Runs are waited on by
   run id. A "Bypassed rule violations" line is a failure. Each `SHA256SUMS*`
   is verified with `minisign -V -P <pubkey>`, and the linux binary is
   checksummed and run.
6. **The toolkit tag goes on the commit whose `install.sh` self-pins it**
   (`install-pin-check.yml`), so a toolkit release is two commits (§6).
7. **Holder-facing GUI-manual text is never written by the script.** The
   toolkit pins commit lands by a PR whose author writes that text (§6.4).
8. **`pr-merge-check.sh` and `review-class.sh` are refugium-only.** They are
   not gates here. `--land-pr` (§7) is the merge-readiness check.
9. **Out of scope:** crates.io publish, flashing, any change to any
   workflow file, documentation tags (`manual-v`, `manual-gui-v`,
   `tech-manual-v`, `quickstart-v`, `ultraquickstart-v`, `examples-v`),
   `mnemonic-refugium` (no release workflow), secondary lockfiles
   (`fuzz/Cargo.lock` and the like are left alone).

**Dry-run reading (decided here).** "Dry run does everything short of
pushing a tag" is read as "everything short of the first remote write". The
first remote write is `push-via-staging.sh`, and that call lands the default
branch, because the script has no stage-only mode. So `--dry-run` makes no
remote write at all. The staging CI signal comes in the real run, where a
red result stops before any tag (exit 5) and the run can be retried.

## 2. Interface

Location: `mnemonic-engrave/scripts/cut-release.sh`, bash, one copy. Other
repos reach it by absolute path. Run it from inside a checkout of the repo
the tags belong to. It calls `push-via-staging.sh` from its own directory
(`$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")`). That engrave copy is
byte-identical to the ones in toolkit, ms, md, refugium and seedhammer;
mk, mt and gui have none. It reads the public key from `../minisign.pub`,
line 2. Like push-via-staging, it uses the target repo's `origin`.

```
usage: cut-release.sh --dry-run [--trailer 'Key: value']... <tag>...
       cut-release.sh --push    [--trailer 'Key: value']... <tag>...
       cut-release.sh --verify  [<tag>...]
       cut-release.sh --land-pr <number>

Run --dry-run, --push and --land-pr from inside a checkout of the repo the
tags belong to. The release notes must already be under the crate's
[Unreleased] heading on the default branch; the script never writes prose.
--dry-run changes nothing remote and leaves tracked files as they were.
--push commits, lands the commit through push-via-staging.sh, pushes the
tags by full ref, waits for their release workflows, verifies the published
release and lists the follow-on pin moves.
--verify checks published releases (no tag: the newest release of every CLI).
--land-pr merges a pull request once every check is green.
Exit: 0 ok, 2 usage, 3 refused before any change, 4 a local check failed,
5 staging failed (no tag pushed), 6 tag push failed (no tag pushed),
7 a release run failed or the script stopped after the tag push, 8 verification failed, 9 toolkit pins or PR step failed.
```

`--help` prints exactly that block to stdout and exits 0. Exactly one of the
four mode flags is required. `--trailer` is repeatable, goes only with
`--dry-run`/`--push`, and is passed to `git commit --trailer`.

Environment overrides, all optional:

| variable | default | purpose |
|---|---|---|
| `CUT_RELEASE_SIBLINGS` | parent of the main checkout: `dirname "$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")"` | where the sibling clones live (follow-on scan, §8) |
| `CUT_RELEASE_DATE` | `date +%F` | date in the new CHANGELOG heading. It must match `^[0-9]{4}-[0-9]{2}-[0-9]{2}$`, else exit 2 `cut-release: CUT_RELEASE_DATE must be YYYY-MM-DD` |
| `CUT_RELEASE_POLL_SECONDS` | `15` | sleep between polls |
| `CUT_RELEASE_CACHE` | `${XDG_CACHE_HOME:-$HOME/.cache}/cut-release` | downloads go to `<cache>/<tag>/`, which is deleted and recreated per verify (the delete is guarded, §5.4). It must be an absolute path other than `/`, else exit 2 `cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is '<cache>')`, checked right after `CUT_RELEASE_DATE`, in every mode, before any other work |
| `CUT_RELEASE_PUSH_VIA_STAGING` | `<script dir>/push-via-staging.sh` | test seam |
| `CUT_RELEASE_PUBKEY_FILE` | `<script dir>/../minisign.pub` | test seam |

Streams. Progress lines (`== `), `DRY RUN: ` lines and verify lines go to
stdout. Every refusal starts with `cut-release: ` and goes to stderr.

Code shape. The script is `set -euo pipefail` functions behind a main guard
(`if [ "${BASH_SOURCE[0]}" = "$0" ]; then main "$@"; fi`), so the test can
source it. It uses here-strings, never `echo | grep -q` (F-695). The four
operator texts that contain the words `git push` (§5.3 step 3's second
toolkit line, the §5.3 step 4a pins-PR line, §6.3 step 4's exit-9 text and
P23) are each written as one line of a here-document that starts in column 1
with `cut-release: ` and contains no `"`, `$(` or backtick. Every other
`git push` in the script is a command, and no comment names it (row 49).
These
functions have fixed names and signatures, because rows 28-32 call them:

- `lock_diff_check <old-lock> <new-lock> <crate>...`
- `changelog_promote <file> <U-line> <heading-line>`
- `tk_examples_b <file> <X> <N>`
- `tk_examples_c <file> <X> <N>`

## 3. The release table (the only per-repo knowledge in the script)

The repo is the basename of `git remote get-url origin`, minus `.git`. The
GitHub slug is always `bg002h/<repo>`. A tag is `<prefix><N>`, where `N`
matches `^[0-9]+\.[0-9]+\.[0-9]+$` and the prefix is exactly one of the
repo's prefixes.

| repo (default) | crate | manifest | prefix | bin | CHANGELOG, U (the `[Unreleased]` line), H (the release heading) | waited tag workflows | linux archive / sums file |
|---|---|---|---|---|---|---|---|
| mnemonic-engrave (master) | mnemonic-engrave | crates/me-cli/Cargo.toml | `v` | me | crates/me-cli/CHANGELOG.md, `## [Unreleased]`, `## [N] - DATE` | release | `mnemonic-engrave-vN-linux-amd64.tar.gz` / SHA256SUMS |
| | mnemonic-io-lib | crates/mnemonic-io-lib/Cargo.toml | `mnemonic-io-lib-v` | (lib) | none | none | none |
| mnemonic-toolkit (master) | mnemonic-toolkit | crates/mnemonic-toolkit/Cargo.toml | `mnemonic-toolkit-v` | mnemonic | CHANGELOG.md, `## [Unreleased]`, `## mnemonic-toolkit [N] — DATE` | release man-pages install-pin-check changelog-check | `mnemonic-N-x86_64-linux-musl.tar.gz` / SHA256SUMS.x86_64 |
| mnemonic-secret (master) | ms-cli | crates/ms-cli/Cargo.toml | `ms-cli-v` | ms | CHANGELOG.md, `## ms-cli [Unreleased]`, `## ms-cli [N] — DATE` | release man-release | `ms-N-x86_64-linux-musl.tar.gz` / SHA256SUMS.x86_64 |
| | ms-codec | crates/ms-codec/Cargo.toml | `ms-codec-v` | (lib) | CHANGELOG.md, `## ms-codec [Unreleased]`, `## ms-codec [N] — DATE` | none | none |
| descriptor-mnemonic (main) | md-cli | crates/md-cli/Cargo.toml | `descriptor-mnemonic-md-cli-v` | md | CHANGELOG.md, `## md-cli [Unreleased]`, `## md-cli [N] — DATE` | release man-pages | `md-N-linux-amd64.tar.gz` / SHA256SUMS.portable |
| | md-codec | crates/md-codec/Cargo.toml | `md-codec-v` | (lib) | CHANGELOG.md, `## md-codec [Unreleased]`, `## md-codec [N] — DATE` | none | none |
| mnemonic-key (main) | mk-cli | crates/mk-cli/Cargo.toml | `mk-cli-v` | mk | crates/mk-cli/CHANGELOG.md, `## [Unreleased]`, `## [N] — DATE` | release musl-binaries CI | `mk-N-x86_64-linux-musl.tar.gz` / SHA256SUMS.x86_64 |
| | mk-codec | crates/mk-codec/Cargo.toml | `mk-codec-v` | (lib) | crates/mk-codec/CHANGELOG.md, `## [Unreleased]`, `## [N] — DATE` | CI | none |
| mnemonic-transaction (main) | mt-cli | crates/mt-cli/Cargo.toml | `mt-cli-v` | mt | none (the repo has no CHANGELOG) | release | `mt-N-linux-amd64.tar.gz` / SHA256SUMS.portable |
| mnemonic-gui (master) | mnemonic-gui | Cargo.toml | `mnemonic-gui-v` | mnemonic-gui | CHANGELOG.md, `## Unreleased`, `## mnemonic-gui [N] — DATE` | build schema-mirror | `mnemonic-gui-vN-x86_64-linux.tar.gz` / SHA256SUMS |

`—` is U+2014 with one space on each side. Engrave alone uses ` - `, as its
file already does. A crate with no prefix (wc-codec, mt-codec) cannot be
released by the script. Other workflows also fire on tags (fuzz-smoke,
sibling-pin-check, bitcoind-differential, ...). They are printed, not
waited on (§5.3). In shared files (ms, md), U names the crate. The fix PR
writes `## ms-cli [Unreleased]` / `## ms-codec [Unreleased]` above every
release heading. Today's stale lower `## Unreleased` (ms) and
`## [Unreleased]` (md) lines are not U, so they are ignored.

Grounding (run 2026-11-01): the workflow names are the `name:` fields of
every workflow whose `on.push.tags` matches the prefix, on each default
branch. Archive names, sums files and `<bin> --version` output were taken
from the latest release of each prefix: every binary prints exactly
`<bin> <version>` on a headless run. Each trusted comment ends
`<tag> <sumsfile>`, e.g. `ms ms-cli-v0.20.2 SHA256SUMS.x86_64`.

## 4. Preflight (exit 3 unless noted; nothing is changed). Checks run in this order.

| # | condition | exact stderr | exit |
|---|---|---|---|
| U1 | not exactly one mode flag | `cut-release: give exactly one of --dry-run, --push, --verify, --land-pr (see --help)` | 2 |
| U2 | `--dry-run`/`--push` with no tag | `cut-release: name at least one tag` | 2 |
| U3 | `--trailer` value lacks `: ` | `cut-release: --trailer needs 'Key: value', got '<arg>'` | 2 |
| U4 | `--trailer` with `--verify`/`--land-pr` | `cut-release: --trailer goes only with --dry-run or --push` | 2 |
| U5 | unknown option | `cut-release: unknown option '<arg>'` | 2 |
| U6 | `--land-pr` without exactly one all-digits argument | `cut-release: --land-pr takes one PR number` | 2 |
| U7 | `--verify` with a lib-row tag | `cut-release: <tag> is a library tag; it has no release assets to verify` | 2 |
| U8 | `--verify` with a prefix in no row | `cut-release: '<tag>' is not a release tag of any repo in the table` | 2 |
| P1 | tool missing. `--dry-run`: git cargo jq sha256sum tar sed awk (+ python3 for the toolkit). `--push`: those + gh minisign. `--verify`: gh minisign jq sha256sum tar. `--land-pr`: git gh jq | `cut-release: '<tool>' is not on PATH` | 3 |
| P2 | not in a git work tree | `cut-release: not inside a git checkout; run it from the repo you are releasing` | 3 |
| P3 | repo not in §3 | `cut-release: origin is '<url>'; '<repo>' is not in the release table` | 3 |
| P4 | tag lacks a semver tail | `cut-release: '<tag>' does not end in a MAJOR.MINOR.PATCH version` | 3 |
| P5 | prefix not this repo's | `cut-release: '<tag>' is not a release tag of <repo> (its prefixes: <p> <p>)` | 3 |
| P6 | two tags for one crate | `cut-release: two tags for <crate>: <t1> <t2>` | 3 |
| P7 | `--push`: `scripts/cut-release.sh`, `scripts/push-via-staging.sh` or `minisign.pub` modified in the script's own checkout (one line per file) | `cut-release: <file> in <engrave toplevel> has uncommitted changes; a release runs only committed tooling` | 3 |
| P8 | `--push`/`--verify`: key file line 2 not `^RW[A-Za-z0-9+/]{54}$` | `cut-release: no minisign public key at <path> line 2` | 3 |
| P9 | `--push`/`--verify`/`--land-pr`: `gh auth status` fails | `cut-release: gh is not logged in (gh auth login)` | 3 |
| P10 | `git fetch --quiet --tags origin` fails | `cut-release: git fetch origin failed:` + git's stderr, each line indented 3 spaces | 3 |
| P11 | current branch is not the default | `cut-release: on branch '<b>'; release from <default> (git switch <default>)` | 3 |
| P12 | `HEAD` ≠ `origin/<default>`, and `origin/<default>` is not an ancestor of `HEAD` | `cut-release: HEAD <12> is not origin/<default> <12>; the release commit must sit directly on the remote tip (git pull --ff-only, or land your commits first)` | 3 |
| P12a | `origin/<default>` is a proper ancestor of `HEAD` (HEAD is <k> commits ahead) | `cut-release: HEAD <12> is <k> commit(s) ahead of origin/<default> <12>; the release commit must sit directly on the remote tip. A 'release: ...' commit from an interrupted cut-release.sh run: drop it (git reset --hard origin/<default>) and run again. A 'pins: installer -> ...' commit: finish as cut-release.sh printed when it stopped, and never land it without its pull request. Any other commit: land it first.` | 3 |
| P13 | `git status --porcelain --untracked-files=no` non-empty | `cut-release: tracked files are modified; the release commit carries only what cut-release.sh writes. Commit or restore them first:` then `   <path>` per file | 3 |
| P14 | `rust-toolchain.toml` has `channel = "<c>"`, `<c>` numeric, and `cargo --version` does not start with `cargo <c> ` | `cut-release: cargo --version is '<out>' but rust-toolchain.toml pins <c>; put rustup's cargo first on PATH (PATH=$HOME/.cargo/bin:$PATH)` | 3 |
| P15 | tag on origin (`git ls-remote --tags origin refs/tags/<tag>`) / only local | `cut-release: tag <tag> already exists on origin` then `cut-release: if an interrupted cut-release.sh --push pushed it: cut-release.sh --verify <tag>, then the moves in <git-dir>/cut-release/follow-on.txt` / `cut-release: tag <tag> already exists locally but not on origin; delete it if you made it by hand: git tag -d <tag>` | 3 |
| P16 | `cargo metadata --no-deps --format-version 1 --offline` fails | `cut-release: cargo metadata failed:` + its stderr indented 3 | 3 |
| P17 | N not newer than the manifest version X (rule below) | `cut-release: <tag>: <crate> is <X> in <manifest>; <N> is not newer` | 3 |
| P18 | the newest origin tag `^<prefix>[0-9]+\.[0-9]+\.[0-9]+$` (version L) is not older than N | `cut-release: <tag>: origin already has <prefix><L>, which is not older than <N>` | 3 |
| P22 | N ≠ X, and N is not a successor of X. With no '-' suffix on X, N is none of X.Y.(Z+1), X.(Y+1).0, (X+1).0.0. With a '-' suffix, triple(N) ≠ triple(X) | `cut-release: <tag>: <crate> is <X> in <manifest>; the next version is one of <X.Y.Z+1> <X.Y+1.0> <X+1.0.0> (a skip is released by hand)`. For a '-' suffix: `cut-release: <tag>: <crate> is <X> in <manifest>; the next version is <triple(X)> (a skip is released by hand)` | 3 |
| P19 | toolkit, X = N | `cut-release: mnemonic-toolkit is already <N> in crates/mnemonic-toolkit/Cargo.toml; the toolkit release needs its version bump in commit (b), so release it by hand or restore the previous version` | 3 |
| P20 | toolkit, the first `mnemonic-toolkit-v[0-9.]+` in `scripts/install.sh` is not `mnemonic-toolkit-v<X>` | `cut-release: scripts/install.sh self-pins mnemonic-toolkit-v<Y> but Cargo.toml says <X>; land the previous release's pins PR first` | 3 |
| P21 | a workspace member's path dependency on a bumped crate has a metadata `req` other than `=X`, `^X` or `*` | `cut-release: <manifest> pins <crate> at '<req>'; expected '=<X>', '^<X>' or no version (fix the pin by hand first)` | 3 |
| P23 | `--dry-run`/`--push`: `git ls-remote origin refs/heads/ci/staging` prints a sha S, S is not the sha of `HEAD`, and `git merge-base --is-ancestor <S> HEAD` exits non-zero (an S unknown locally counts as not an ancestor) | `cut-release: origin has ci/staging at <12>; push-via-staging.sh cannot push a new release commit over it (an interrupted or failed staging run left it). If no staging run is in progress, delete it, then run again: git push origin --delete ci/staging` | 3 |
| C1-C7 | CHANGELOG, below | below | 3 |

P23: <12> is the first 12 characters of S. S = HEAD (the resume path) and S behind HEAD (an old staging a fast-forward covers) are allowed.

**Version rule (P17).** Compare the numeric triples. N is newer than X when
triple(N) > triple(X), or when the triples are equal and X has a `-`
pre-release suffix (`0.14.0-dev` → `0.14.0`). When N = X exactly, there is
no version edit. This is the resume path (§5.1). For the toolkit it is P19.

**CHANGELOG rule** (per crate with a CHANGELOG; F, U, H from §3). A line is
"U" when it equals U after stripping trailing whitespace. It is "H" when it
equals the H prefix (H up to and including `[N]`, e.g. `## ms-cli [0.20.3]`,
`## [0.14.0]`), or starts with that prefix followed by a space. An *own
release heading* is a line matching the crate's H with any version:
`^## ms-cli \[[0-9]+\.[0-9]+\.[0-9]+\]` for ms-cli,
`^## \[[0-9]+\.[0-9]+\.[0-9]+\]` for a `## [N]` file. Other crates'
headings in a shared file never count. A *body* is the lines after a
heading, up to the next line starting `## ` or EOF. It is *empty* when
every line is blank. `<n>` in C3, C4 and C6 is U's line number.

| # | state | exact stderr |
|---|---|---|
| C2 | more than one U line / more than one H line | `cut-release: <F> has <k> '<U>' lines; keep one` / `cut-release: <F> has <k> '<H-prefix>' headings; keep one` |
| C5 | U body non-empty AND an H line exists | `cut-release: <F> has both '<H-prefix>' and entries under '<U>'` |
| C3 | an own release heading is above U (the first one is quoted) | `cut-release: <F>:<n>: '<U>' sits below '<line>'; it must be above every release heading` |
| C6 | U body's first non-blank line starts `**Next release: ` but not `**Next release: <N>**` | `cut-release: <F>:<n>: '<U>' says 'Next release: <V>', not <N>` (V = text between `: ` and `**`) |
| C4 | U present, body empty, no H line | `cut-release: <F>:<n>: the '<U>' section is empty; nothing to release for <crate>` |
| C1 | no U line and no H line | `cut-release: <F> has no '<U>' section; the fix PR writes the release notes there` |
| C7 | a crate this run releases depends by path on workspace crate D, D is not in this run, and D's U body is non-empty | `cut-release: <D> has entries under '<U of D>' and <crate> builds it by path; tag <D> in this same run (<D prefix>MAJOR.MINOR.PATCH) or move those entries` |

U with a non-empty body and no H line is promoted (§5.1). If U is absent or
empty and exactly one H line exists, the heading is already in place. That
is the resume path: no CHANGELOG edit, and the run prints
`== CHANGELOG: <F> already has '<H-prefix>'`.

## 5. Flow (non-toolkit; the toolkit differs only where §6 says so)

### 5.1 Local phase (after preflight, before any commit)

From here on, a failure restores every tracked file the script touched
(`git restore --source=HEAD --staged --worktree -- <files>`), prints
`cut-release: tracked files restored; nothing was committed or pushed` as
its last line, and exits 4. A dry run always restores at the end, by an
EXIT trap.

1. For each tag in argument order: if X ≠ N, replace the manifest's single
   `[package]` line `version = "X"` with `version = "N"`. Print
   `== <crate> <X> -> <N> (<manifest>)`. When X = N, print
   `== <crate> is already <N> (<manifest>); no version edit`.
2. Pins: for each member path-dependency on a bumped crate with req `=X`
   or `^X`, replace `version = "=X"` (or `version = "X"`) on that
   manifest's line starting `<crate> = {` with N, keeping the `=`. Print
   `== pin: <manifest> <crate> <old> -> <new>` (`=0.10.1 -> =0.10.2`, or
   `0.1.0 -> 0.1.1`). `*` (no version) is left alone, silently.
3. CHANGELOG: `changelog_promote F U "H-prefix<sep>DATE"`. The output is the
   lines before U unchanged, then U, a blank line, the new heading, the U
   body, then the rest unchanged. The body drops the `**Next release: N**`
   paragraph, if present: that runs from that line to the line before the
   next blank line, and the blank line after it goes too. Print
   `== CHANGELOG: <F> '<U>' -> '<new heading>'`. A repo with no CHANGELOG
   prints `== CHANGELOG: none in <repo>`.
4. `cargo update --workspace --offline` (log at
   `$(git rev-parse --git-dir)/cut-release/update.log`). On failure:
   `cut-release: cargo update --workspace --offline failed (log <path>); run 'cargo fetch' once online, then retry`.
5. `lock_diff_check <HEAD:Cargo.lock> Cargo.lock <bumped crates>`. It
   passes when `git diff --numstat` reports k added = k removed = the
   number of bumped crates, and the new lock has each crate's
   `[[package]] name` at N. It prints
   `== Cargo.lock: <k> version line(s) changed (<crates, tag order, space-separated>)`.
   Otherwise:
   `cut-release: Cargo.lock changed beyond the bumped crates' version lines (<k> line(s) changed, <e> expected):`
   followed by the `-`/`+` lines, indented 3.
6. `cargo build --locked --workspace` (log `.../build.log`). On failure:
   `cut-release: cargo build --locked --workspace failed (log <path>); last 20 lines:`
   followed by the tail.
7. For each bumped crate with a bin: `<target_directory>/debug/<bin> --version`
   (target_directory from cargo metadata). The first line must equal
   `<bin> <N>`. Print `== built: <bin> --version = '<bin> <N>'`. Otherwise:
   `cut-release: <bin> --version printed '<out>', not '<bin> <N>'`.
8. Engrave only: run `demo/sh2/check-install-links.sh`. On exit 0, print
   `== engrave: demo/sh2/check-install-links.sh OK`. Otherwise:
   `cut-release: demo/sh2/check-install-links.sh failed; land the engrave demo follow-on for the sibling release it names first:`
   followed by the last 15 lines. (Engrave's required `test (rust + go)` runs
   it, so staging would fail anyway.)

### 5.2 Dry-run output (then restore; exit 0)

```
DRY RUN: would commit <k> file(s): <paths, LC_ALL=C sorted, space-separated>
DRY RUN: would run push-via-staging.sh <default>
DRY RUN: would tag <tag> on the release commit and push refs/tags/<tag>      (one per tag)
DRY RUN: would wait for <tag> workflows: <names>|none                         (one per tag)
DRY RUN: would verify <tag>: <archive> in <sums>, every SHA256SUMS* signed, '<bin> <N>'   (bin tags)
DRY RUN: would check <tag> points at the release commit                       (lib tags)
<follow-on block, §8>
== DRY RUN OK: nothing was pushed; tracked files are as they were
```

When nothing changed (the resume path), the first line reads
`DRY RUN: nothing to commit; would stage HEAD <12> as it is`. The banner
before all of this is two lines:
`== cut-release DRY RUN: <repo> (<default>) -> <tags>` and
`== script: <abs path> (mnemonic-engrave <12>)`. `--push` prints `PUSH` in
place of `DRY RUN`, then a third line:
`== PUSH: these tags cannot be taken back once pushed: <tags>`.

### 5.3 Remote phase (`--push` only)

1. Commit, if anything changed: `git add` exactly the files touched, then
   `git commit -F <msg>` plus `--trailer` per flag. The message:
   ```
   release: <crate> <N>[ + <crate> <N>]...

   Cut by cut-release.sh (mnemonic-engrave <12>). Versions and the CHANGELOG
   heading only: the gate is CI on this exact commit (push-via-staging.sh),
   not a local suite.

   Files: <paths, space-separated>
   Local checks: cargo build --locked --workspace OK; Cargo.lock <k> version line(s); <bin> --version = '<bin> <N>'
   ```
   (one `--version` clause per bin, `; `-joined; the clause is omitted for
   lib-only runs). Print `== commit <12> <subject>`.
2. `== FREEZE <slug> <default>: no commits or merges until this script prints FREEZE LIFTED`.
3. Run push-via-staging with `<default>` as its argument, its output teed
   to stdout and captured. Exit 5 when: it exits non-zero; or its output
   matches `bypassed rule violations` (case-insensitive) —
   `cut-release: push-via-staging.sh output contains 'Bypassed rule violations'; not tagging. STOP and report it.`;
   or it exits 0 without a line starting `== OK: ` —
   `cut-release: push-via-staging.sh exited 0 without its '== OK:' line; not tagging`.
   On a non-zero exit, print
   `cut-release: push-via-staging.sh exited <n>; no tag was pushed.`, fetch,
   and add exactly one of two lines. If `origin/<default>` is the release
   commit:
   `cut-release: the release commit IS on origin/<default>. If the output above shows a bypass, STOP and report it; otherwise re-run the same command: it finds the commit on <default>, re-stages it and tags.`
   If not:
   `cut-release: origin/<default> is unchanged. To retry, drop the local commit and run again: git reset --hard origin/<default>`.
   For the toolkit, the IS-on-origin line is replaced by these two lines,
   because a re-run is refused (P12a, then P19):
   `cut-release: the toolkit release commit (b) <12> IS on origin/master; its tag goes on (c) <12>, which is only on local master (do not reset it). If the output above shows a bypass, STOP and report it.`
   `cut-release: otherwise finish by hand: git tag -a mnemonic-toolkit-v<N> -m 'mnemonic-toolkit <N>' <c sha> && git push origin refs/tags/mnemonic-toolkit-v<N>; when its release, man-pages, install-pin-check and changelog-check runs are completed/success (gh run list --repo bg002h/mnemonic-toolkit --branch mnemonic-toolkit-v<N>): cut-release.sh --verify mnemonic-toolkit-v<N>, then the pins-PR line below.`
   They are followed by the pins-PR line of §5.3 step 4a.
   On success: `== FREEZE LIFTED`. Any exit between the FREEZE line and
   that point prints `== FREEZE LIFTED` before its refusal lines. The
   freeze covers only the push-via-staging window: tags go on an explicit
   SHA, so later commits to the branch do no harm.
4. Tags: `git tag -a <tag> -m '<crate> <N>' <sha>` for each, then one
   `git push --atomic origin refs/tags/<t1> [refs/tags/<t2>]`.
   If the push fails, read `git ls-remote origin refs/tags/<t>^{}` for every tag.
   - If every tag peels to `<sha>`, print `== the tag push reported an error, but origin has every tag on <12>; continuing` and continue.
   - If no tag is on origin, delete the local tags and print `cut-release: pushing the tags failed (atomic; none is on origin):` with git's output indented 3. Then print `cut-release: re-run the same command; it finds the release commit on <default> and tags it` (for the toolkit, §5.3 step 3's two toolkit lines instead), and exit 6.
   - Otherwise print `cut-release: after the failed push origin has <t> -> <x|absent>` (one line per tag), then `cut-release: STOP and report it`, and exit 6.

   Afterwards, each
   `git ls-remote origin refs/tags/<t>^{}` must equal `<sha>`. If not:
   `cut-release: origin's <t> peels to <x>, not <sha>; STOP and report it`,
   exit 6. Print `== tag <t> -> <12> pushed`.
4a. **Finishing lines.**
   - Before the tag push, write the §8 block, computed from the pre-run state as a dry run prints it, to `<git-dir>/cut-release/follow-on.txt` (`<git-dir>` = `git rev-parse --absolute-git-dir`).
   - The toolkit writes the §6.3 step 5 PR body to `<git-dir>/cut-release/pins-pr-body.md` before §6.3 step 1.
   - From the tag push on, the script traps INT, TERM and HUP, and also EXIT. The variable set after the tag push is set the moment git push --atomic returns 0, or when step 4 prints the "continuing" line. The signal trap prints `cut-release: interrupted after the tag push; the tags are pushed and stay pushed: never delete or move them.` and exits 7. The EXIT trap acts only when the variable set after the tag push is set, the variable set just before the `== DONE` line is unset, and no exit 6, 7, 8 or 9 and no signal trap has begun (a third variable they set; step 4's exit 6 after the push sets it too, so the EXIT trap never changes an exit 6). That case is a `set -e` abort, for example a failing `gh`, `git ls-remote` or `jq` call. It prints `cut-release: stopped after the tag push by a failing command (above); the tags are pushed and stay pushed: never delete or move them.`, then the finishing lines below (for the toolkit after the §6.3 step 6 reset), and exits 7. Neither trap prints step 6's `== follow-on pins are NOT listed` line: an abort can come after a clean verify, and the finishing lines are true in every state.
   - The signal trap, the EXIT trap and every exit 7, 8 or 9 end with:
     `cut-release: finish: cut-release.sh --verify <bin tags>; once it shows 0 failed, make the follow-on moves in <git-dir>/cut-release/follow-on.txt`
   - For the toolkit, until §6.3 step 5 has opened the PR (and except after §6.3 step 4's exit 9), it then adds the pins-PR line:
     `cut-release: pins PR: git checkout --quiet --detach <c sha> && sh scripts/install-assets.test.sh && git checkout --quiet master && git push origin <c sha>:refs/heads/release-pins-<N> && gh pr create --repo bg002h/mnemonic-toolkit --base master --head release-pins-<N> --draft --title 'pins: installer -> mnemonic-toolkit-v<N>' --body-file <git-dir>/cut-release/pins-pr-body.md`
   - The traps key on variables set after the tag push, never on `$?`. Measured: bash runs an EXIT trap after SIGINT with `$?` = 0.
5. Wait for each tag's §3 workflows, by run id. Poll
   `gh run list --repo <slug> --branch <tag> --event push --limit 50 --json databaseId,name,headSha,status,conclusion,url`
   until a run with that `name` and `headSha` = sha exists. Allow at most 40
   polls; then
   `cut-release: no '<w>' run appeared for <tag> within 10 minutes`. Print
   `== waiting for <tag>: <w> run <id>`. Then poll
   `gh run view <id> --repo <slug> --json status,conclusion,url` until
   `status` = `completed`. Allow at most 360 polls; then
   `cut-release: run <id> (<w>, <tag>) did not finish within 90 minutes: <url>`.
   A `conclusion` other than `success` gives
   `cut-release: run <id> (<w>, <tag>) concluded '<c>': <url>`. Each of the
   three adds:
   `cut-release: the tags are pushed and stay pushed: never delete or move them.`
   and
   `cut-release: a flake: gh run rerun <id> --failed --repo <slug>, then cut-release.sh --verify <tag>. A real defect: STOP; the fix is a new patch release.`
   then exits 7. On success: `== run <id> <w> (<tag>) success`. After all
   waits, each other run on the tag gets one line:
   `   not waited: <name> run <id> <status>/<conclusion|->`.
6. Verify (§5.4) each bin tag. A lib tag gets `   ok    <tag> peels to <12>`
   (already checked in step 4). Any FAIL gives
   `cut-release: <tag> did not verify (above); the tags stay pushed. STOP and report it.`
   and exit 8. Exits 7 and 8 from steps 5 and 6 (not the traps' exit 7) also print
   `== follow-on pins are NOT listed: the release is not verified` and skip §8; they end with the step 4a finishing lines.
7. The §8 follow-on block, then `== DONE: <tags> published and verified`,
   exit 0.

### 5.4 Verify (one tag; shared by `--push` and `--verify`)

**Cache directory.** Before V1, verify deletes and recreates `<cache>/<tag>`
(`<cache>` is `CUT_RELEASE_CACHE` or its default). It first checks, in this
order, and on the first failure prints one line to stderr, exits 2 and deletes
nothing: (1) `<cache>` is an absolute path other than `/` (a run of slashes
counts as `/`): `cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is '<cache>')`;
(2) `<tag>` has no `/` and is not `.` or `..`: `cut-release: '<tag>' cannot name a cache directory: it has a '/' or is '.' or '..'`;
(3) the directory is exactly `<cache>/<tag>` and lies strictly below
`<cache>`: `cut-release: refusing to delete '<dir>': it is not exactly '<cache>/<tag>' strictly below the cache`.
The delete is `rm -rf -- "${d:?}"`. Check (1) has already run in `main`
before any change, so no new exit appears after the tag push. Checks (2) and
(3) cannot fail from the command line (a tag reaches this point only after its
prefix matched a table row, no prefix has a `/`, and a tag ends in
`MAJOR.MINOR.PATCH`); they guard the function itself, and rows 74 and 75 call
it directly.

The section opens with `== verify <tag>`. Every check runs, even after a
FAIL. Each prints `   ok    <text>` or `   FAIL  <text>`:

| check | ok text | FAIL text(s) |
|---|---|---|
| V1 `gh release view <tag> --repo <slug> --json isDraft,isPrerelease,assets` | `release is published (not draft, not prerelease), <n> assets` | `release not found` / `release is a draft` / `release is a prerelease` |
| V2 previous = the newest non-draft, non-prerelease release with this prefix and a lower version (`gh release list --repo <slug> --limit 100 --json tagName,isDraft,isPrerelease`), version PV. Each of its asset names, with literal PV replaced by N, must be present | `every asset of <prev> is present (names with <PV> -> <N>)` (none older: `   --    no earlier <prefix> release to compare against`) | `missing vs <prev>: <name>` (one per name) |
| V3 every asset `SHA256SUMS*` not ending `.minisig` has `<name>.minisig` | (no line; V4 reports) | `no <sums>.minisig asset (unsigned release)` |
| V4 download all `SHA256SUMS*` and the archive; in the download dir run `minisign -Vm <sums> -x <sums>.minisig -P <key>` | `<sums> signed: <its 'Trusted comment: ...' line>` | `<sums>.minisig does not verify:` + minisign output indented 9 / `<sums> trusted comment is '<c>', not '... <tag> <sums>'` |
| V5 the archive appears (awk `$2 == name`) in exactly one sums file, and `sha256sum -c` of that line passes | `<archive> matches <sums>` | `<archive> is not listed in any SHA256SUMS* file` / `<archive> is listed in <k> SHA256SUMS* files` / `<archive> does not match <sums>` |
| V6 extract; exactly one regular file named `<bin>`; `timeout 30 <bin> --version`; the first line = `<bin> <N>` | `RAN <bin> --version -> <bin> <N>` | `<archive> has <k> files named '<bin>'` / `<bin> --version printed '<out>', not '<bin> <N>'` |

The section closes with `== verify <tag>: <ok> ok, <fail> failed`. Bare
`--verify` covers the newest non-draft, non-prerelease release of each
bin prefix (`v`, `mnemonic-toolkit-v`, `ms-cli-v`,
`descriptor-mnemonic-md-cli-v`, `mk-cli-v`, `mt-cli-v`, `mnemonic-gui-v`).
`--verify` ends with `== verify: <k> release(s), <f> failed` and exits 8 if
f > 0. An unsigned release is a FAIL; there is no allow-unsigned flag.
This absorbs `scripts/verify-releases.sh`, which is deleted (§10).

## 6. Toolkit: two commits

Let X be the current version, N the new one, T = `mnemonic-toolkit-v<N>`,
P = `mnemonic-toolkit-v<X>`, and G = `[manual-gui] toolkit-tag-implied`
from `docs/manual-gui/pinned-upstream.toml` (the GUI's tier).

### 6.1 Commit (b): the release commit (on master through staging)

These are §5.1 steps 1-7, plus:
- the `<!-- toolkit-version: X -->` marker → N in `README.md` and
  `crates/mnemonic-toolkit/README.md` (each must hold it exactly once);
- `tk_examples_b` on `.examples-build/gen.sh` and `.examples-build/Examples.md`.
  This replaces X with N only inside these literal forms:
  `` `mnemonic` vX ``, `` `mnemonic` **vX** ``, `mnemonic-toolkit vX`,
  `"mnemonic X"`, `expected mnemonic X,`, and a whole line `mnemonic X`.
- Invariant: afterwards gen.sh contains no X (matched as
  `(^|[^0-9.])X([^0-9]|$)`, dots literal). Every X left in Examples.md must
  be on an installer line: a line containing `mnemonic-toolkit-vX` or
  `'mnemonic X'`. There must be at least one such line. Otherwise exit 4:
  `cut-release: toolkit (b) left <k> unexpected line(s) naming <X>:` + those
  lines. Print
  `== toolkit (b): README markers 2, .examples-build/gen.sh <k> line(s), Examples.md <k> line(s); <k> installer line(s) left for (c)`.
- Stage (b)'s files with `git add`. (c)'s edits then go to the work tree
  only, so all checks run before either commit.

Measured on master today (X 0.105.1): gen.sh 5 lines, Examples.md 4 lines
plus 4 installer lines. This is exactly precedent `b0801244`/`1402d547`.

### 6.2 Commit (c): the pins commit (tagged; master by PR)

- `scripts/install.sh`: P → T. It must be the only occurrence of P.
- `first_signed()`: if the line matching
  `^\s*mnemonic\)\s+echo "" ;;$` exists, set it to `echo "<N>" ;;`. If the
  entry is already non-empty, leave it. If no `mnemonic)` entry exists,
  exit 4:
  `cut-release: cannot find the 'mnemonic)' entry of first_signed() in scripts/install.sh`.
- `tk_examples_c` on Examples.md. On each installer line, X → N. A `--list`
  line (it matches `^mnemonic +mnemonic-toolkit +`) keeps its `GIT_TAG`
  column 37 characters wide (`%-36s` plus a space): the spaces after T
  become `max(1, 37 - length(T))`.
- `docs/manual-gui/pinned-upstream.toml`: replace the value of an existing
  `toolkit-tag-implied` line under `[installer-ahead]`. Else insert
  `toolkit-tag-implied = "<T>"` right after the `[installer-ahead]` line.
  Else append `\n[installer-ahead]\ntoolkit-tag-implied = "<T>"\n`.
- Checks (exit 4 on failure, `cut-release: toolkit (c): <what> failed:`):
  the first `mnemonic-toolkit-v[0-9.]+` in install.sh is T (the
  install-pin-check logic); install.sh has no P; Examples.md has no X.
- Informational, never a failure: run
  `python3 docs/manual-gui/tests/check_cli_pins.py --manual-dir docs/manual-gui --install-sh scripts/install.sh`.
  Print `== toolkit (c): check_cli_pins.py OK` or
  `== toolkit (c): check_cli_pins.py fails on (c); the pins PR owes the GUI manual's {#installer-ahead} section (listed below)`.
- Print
  `== toolkit (c): install.sh self-pin <P> -> <T>; first_signed mnemonic <"" -> "N"|already "V">; Examples.md <k> installer line(s); [installer-ahead] toolkit-tag-implied = "<T>"`.
- On `--push` only (at §5.3 step 1), commit (b) from the index (message §5.3, subject
  `release: mnemonic-toolkit <N> (installer self-pin follows in the pins commit)`).
  Then `git add` (c)'s three files and commit (c):
  ```
  pins: installer -> mnemonic-toolkit-v<N>

  Cut by cut-release.sh (mnemonic-engrave <12>). The tag mnemonic-toolkit-v<N>
  goes on this commit (install-pin-check). It reaches master by pull request,
  after the release verifies; that PR owes the GUI manual's {#installer-ahead}
  text.

  Files: .examples-build/Examples.md docs/manual-gui/pinned-upstream.toml scripts/install.sh
  Local checks: install.sh self-pin = tag; no mnemonic-toolkit-v<X> left in install.sh; Examples golden names no <X>; check_cli_pins.py <OK|fails, prose owed>
  ```

### 6.3 Remote order

1. `git checkout --quiet --detach <b>`, push-via-staging `master` (§5.3
   step 3), then `git checkout --quiet master` on every path. The FREEZE
   covers only this window.
2. Tag (c) with T (§5.3 step 4). (c) is not on master. Neither release
   workflow checks for that (precedent).
3. Wait (§5.3 step 5), then verify T (§5.4).
4. Run `sh scripts/install-assets.test.sh` at (c). Its last stdout line
   must be `[install-assets.test] OK`. Otherwise exit 9:
   `cut-release: scripts/install-assets.test.sh failed on (c) <12>; the pins PR was not opened. The tag is published. Fix forward on top of (c): git push origin <c sha>:refs/heads/release-pins-<N>, commit the fix to that branch, then gh pr create --repo bg002h/mnemonic-toolkit --base master --head release-pins-<N> --draft --title 'pins: installer -> mnemonic-toolkit-v<N>' --body-file <git-dir>/cut-release/pins-pr-body.md (a PR without (c) leaves the tag off master).`
5. `git push origin <c>:refs/heads/release-pins-<N>`, then
   `gh pr create --repo bg002h/mnemonic-toolkit --base master --head release-pins-<N> --draft --title 'pins: installer -> mnemonic-toolkit-v<N>' --body-file <git-dir>/cut-release/pins-pr-body.md`.
   The body is the §6.4 owed block, followed by
   `Tagged commit: <c sha>. Merge with a merge commit only: squash or rebase would leave the tag off master.`
   If this fails, exit 9:
   `cut-release: could not open the pins PR (above); (c) is at refs/heads/release-pins-<N>; open it by hand: gh pr create --repo bg002h/mnemonic-toolkit --base master --head release-pins-<N> --draft --title 'pins: installer -> mnemonic-toolkit-v<N>' --body-file <git-dir>/cut-release/pins-pr-body.md`.
   On success, `<url>` is the last line `gh pr create` printed on stdout and
   `<number>` is the digits after its final `/pull/`. If that line does not
   match `^https://github\.com/bg002h/mnemonic-toolkit/pull/[0-9]+$`, exit 9:
   `cut-release: gh pr create exited 0 but printed no pull request URL; find the PR: gh pr list --repo bg002h/mnemonic-toolkit --head release-pins-<N>`.
6. §8 block, then
   `== toolkit: pins PR <url> is open as a draft; it owes the text above. Its author marks it ready, then: cut-release.sh --land-pr <number>`
   then `== DONE: <T> published and verified`, exit 0. Just before the `== DONE` line it runs `git checkout --quiet master && git reset --quiet --keep <b>`: local master
   equals origin/master, and (c) stays reachable from the tag and
   `release-pins-<N>`. Every exit after the tag push (7, 8, 9, a trapped
   signal, or the EXIT trap) does the same reset first. A failing reset is ignored: its git error shows, and the exit code and the finishing lines are unchanged.

A toolkit dry run prints these lines, in place of §5.2's:

```
DRY RUN: would commit (b) <k> file(s): <paths>
DRY RUN: would commit (c) <k> file(s): <paths>
DRY RUN: would run push-via-staging.sh master for (b)
DRY RUN: would tag <T> on commit (c) and push refs/tags/<T>
DRY RUN: would wait for <T> workflows: release man-pages install-pin-check changelog-check
DRY RUN: would verify <T>: mnemonic-<N>-x86_64-linux-musl.tar.gz in SHA256SUMS.x86_64, every SHA256SUMS* signed, 'mnemonic <N>'
DRY RUN: would run sh scripts/install-assets.test.sh on (c), push (c) to refs/heads/release-pins-<N> and open a draft PR against master
```

These are followed by the §8 block and the `DRY RUN OK` line.

### 6.4 The owed block (in the PR body and in §8)

```
   toolkit pins PR (owed before it can merge; holder-facing text, so the PR's author writes it):
     docs/manual-gui/src/80-troubleshooting/82-binary-and-launch.md {#installer-ahead}: name <T> as installed ahead of the GUI's <G> and say what a GUI user sees differently (precedent 1402d547; create the section as in af986ee4 if it is missing)
     docs/manual-gui/CHANGELOG.md: one line for the declaration
     re-check: python3 docs/manual-gui/tests/check_cli_pins.py --manual-dir docs/manual-gui --install-sh scripts/install.sh
     land it: cut-release.sh --land-pr <PR number>   (from a mnemonic-toolkit checkout)
```

`<PR number>`, here and in §8 (b), is printed as it stands: the PR it names
does not exist yet when the text is written (the PR body, `follow-on.txt`,
a dry run). On `--push`, the §6.3 step 6 line that follows §8 names the
pins PR's number.

## 7. `--land-pr <n>` (merge-readiness check; any table repo)

Run it inside a checkout of the repo; the slug comes from origin. It does
not need a clean tree or the default branch.

1. `gh pr view <n> --repo <slug> --json state,isDraft,baseRefName,headRefName,headRefOid,mergeStateStatus,url,isCrossRepository`.
   Exit 3 with `cut-release: PR #<n> is <STATE>, not OPEN`,
   `cut-release: PR #<n> is a draft; its author marks it ready (gh pr ready <n>) once the owed edits are in`,
   or `cut-release: PR #<n> targets '<b>', not <default>`.
2. If `headRefName` is `release-pins-<V>`: `git fetch` the head, and
   `mnemonic-toolkit-v<V>^{}` must be an ancestor of `headRefOid`. Else exit
   9: `cut-release: PR #<n> does not contain mnemonic-toolkit-v<V> (<12>); merging it would leave the tag off <default>`.
3. Poll `gh pr checks <n> --repo <slug> --json name,bucket,link`, reading
   stdout whatever the exit status (gh exits 8 while checks are pending).
   Wait for a non-empty list (40 polls; else exit 9
   `cut-release: no checks appeared on PR #<n> within 10 minutes`), then
   for no `pending` bucket (240 polls; else exit 9
   `cut-release: PR #<n> checks did not finish within 60 minutes`). Every
   bucket must be `pass` or `skipping`. Else, one line per check,
   `cut-release: PR #<n> check '<name>' is <bucket>: <link>`, then
   `cut-release: fix it on the PR branch, then run cut-release.sh --land-pr <n> again`,
   exit 9.
4. Re-read `mergeStateStatus`; while it is `UNKNOWN`, re-read, at most 10
   polls. It must be `CLEAN`, else exit 9
   `cut-release: PR #<n> merge state is <s>, not CLEAN`.
5. `gh pr merge <n> --repo <slug> --merge --match-head-commit <headRefOid>`.
   On failure, exit 9 with `cut-release: gh pr merge failed:` + output.
6. Fetch; `git merge-base --is-ancestor <headRefOid> origin/<default>`, else
   exit 9
   `cut-release: <12> is not on origin/<default> after the merge; STOP and report it`.
   Then, only if `isCrossRepository` is false and
   `git ls-remote origin refs/heads/<headRefName>` prints `<headRefOid>`, run
   `git push origin --delete <headRefName>` (allowed once ancestry holds,
   Brian 2026-10-14) and print
   `== merged PR #<n> (<url>) as a merge commit; <headRefName> deleted`.
   Otherwise print
   `== merged PR #<n> (<url>) as a merge commit; <headRefName> left in place (<reason>)`,
   where `<reason>` is `it is a fork's branch` or
   `origin's <headRefName> is <12>, not the merged <12>`.
   If the delete fails, print
   `cut-release: merged, but deleting <headRefName> failed:` with git's output
   indented 3, and exit 9. Otherwise exit 0.

## 8. Follow-on block (printed, never performed)

The script never edits another repo. For each pushed tag, it prints what
the sibling repos owe. Each sibling is `$CUT_RELEASE_SIBLINGS/<name>` for
the seven §3 repos plus `mnemonic-refugium` (master) and `seedhammer`
(main). The scan runs `git -C <dir> fetch --quiet origin` (a failure is
ignored; the stale ref is used), then reads `origin/<default>`, never the
work tree. Let P be the newest pre-run origin tag of the prefix, X its
version, and N, T the new ones.

```
== FOLLOW-ON: not done by this script; each is a commit or PR in the named repo
-- <T> (previous <P>)
```

Then, in this order:

(a) For `ms-cli-v`, `descriptor-mnemonic-md-cli-v`, `mnemonic-toolkit-v`.
Stem/label: `ms-`/`ms-cli`, `md-`/`md-cli`, `mnemonic-`/`mnemonic-toolkit`.
The block ends with `scripts/push-via-staging.sh master`. Regex metachars
in P and X are escaped with `\`. This recipe was executed on a copy of
today's files for ms 0.20.1→0.20.2: 7 lines changed, no stale line left,
and `check-install-links.sh` printed OK.

```
   engrave demo (its CI's required 'test (rust + go)' fails until this lands):
     cd <S>/mnemonic-engrave && git switch master && git pull --ff-only
     sed -i 's#\b<P>\b#<T>#g; s#\b<stem><X>-#<stem><N>-#g' demo/sh2/src/index.html demo/sh2/CLI_SPINE.md
     ./demo/sh2/check-install-links.sh
     git commit -m 'demo/sh2: install block advertises <label> v<N>' -- demo/sh2/src/index.html demo/sh2/CLI_SPINE.md
     scripts/push-via-staging.sh master
```

(b) For `ms-cli-v` (short `ms`), `descriptor-mnemonic-md-cli-v` (`md`),
`mk-cli-v` (`mk`), `mnemonic-gui-v` (`mnemonic-gui`). The `first_signed`
line is printed only when that entry on the sibling's `origin/master`
`scripts/install.sh` is `""`. A missing sibling prints
`first_signed: <S>/mnemonic-toolkit not found; check 'first_signed <short>' by hand`
instead. The repad note is printed only when length(T) ≠ length(P). The
Examples sed was executed on today's golden for ms 0.20.2→0.20.3: exactly
the 4 ms installer lines changed.

```
   toolkit installer pin (a PR in <S>/mnemonic-toolkit from origin/master):
     scripts/install.sh component_info: <P> -> <T>
     scripts/install.sh first_signed <short>: "" -> "<N>" (this release is signed)
     mirrors: git grep -n -F -- '--tag <P>' .github/workflows docs/manual/src docs/quickstart/src
     Examples golden: sed -i -E "/<P>|'<bin> <X>'/ s#<X>#<N>#g" .examples-build/Examples.md
     Examples golden: <T> and <P> differ in length; keep the --list GIT_TAG column 37 wide
     GUI manual: python3 docs/manual-gui/tests/check_cli_pins.py --manual-dir docs/manual-gui --install-sh scripts/install.sh
       if it fails: [installer-ahead] <key> = "<T>" in pinned-upstream.toml, plus {#installer-ahead} text (holder-facing: the PR's author writes it; precedent 99310923)
     gates: sh scripts/install-assets.test.sh; then land it: cut-release.sh --land-pr <PR number>
```

`<key>` is `ms-cli-tag-implied`, `md-cli-tag-implied`,
`mk-cli-tag-implied`, or (gui) the `[mnemonic-gui] tag`. For gui the line
reads `if it fails: the GUI tag cannot run ahead; the GUI manual re-pins to <T> (its [mnemonic-gui] and [manual-gui] tables)`.

(c) For the toolkit's own tag: the §6.4 block.

(d) Always:

```
   references to <P> on each sibling's default branch (history paths excluded; some name an older tier on purpose):
     <repo>:<path>:<line>: <text, cut to 160 chars>
```

This is `git grep -n -w -F '<P>' origin/<default> -- . ':!design' ':!vendor' ':!**/CHANGELOG.md' ':!**/agent-reports/**' ':!**/*release-history*' ':!**/cli-version-history.txt' ':!**/Cargo.lock'`.
For prefix `v` the pattern is `mnemonic-engrave/releases/download/<P>/`
(fixed, no `-w`). The block is capped at 40 lines; beyond that it prints
`     ... <k> more: git -C <dir> grep -n -w -F '<P>' origin/<default> -- <same pathspecs>`.
A missing sibling prints
`     <repo>: not found at <S>/<repo>; check it by hand`. Zero hits print
`     (none)`.

A dry run prints the same block, computed from the pre-run state.

## 9. Test rows (`scripts/test/cut-release.test.sh`)

The harness prints `ok <row>` or `FAIL <row>: <why>` per row and ends with
`cut-release.test: <p> passed, <f> failed`. It exits 0 iff f = 0 and the
number of rows run equals the expected count: 85 by default, 77 with
`--offline` (rows 1-77). A row "shows L" when the named stream has L as a whole line.
`<12>`, `<sha>`, `<c sha>` (commit (c)'s full sha) and `<git-dir>`
(`git rev-parse --absolute-git-dir` in the fixture checkout) in expected
text are computed by the harness.

**Harness setup.** `T=${XDG_CACHE_HOME:-$HOME/.cache}/cut-release-test/run-$$`,
never under /tmp. T is deleted on full success. Every recursive delete in the
harness and its fixtures is `rm -rf -- "${VAR:?}"`. Deleting `T` itself (at
the start and at the end) is refused unless `T` matches
`*/cut-release-test/run-*`. Deleting any other path (`mk_eng`'s argument, the
paths `fresh` clears, `FXDIR`, `FXBARE`, the assets directory) is refused
unless it lies strictly below `T` and has no `..` component. A refusal deletes
nothing, prints `cut-release.test: refusing to delete '<path>': <why>` to
stderr and exits 2. Fake gh's merge scratch clone is deleted only when it
lies below `T`, else fake gh prints `fake gh: refusing to delete '<dir>': it is not below <T>` to stderr and exits 1.
Rows 76 and 77 test this. `PATH=$T/bin:$HOME/.cargo/bin:$PATH`.
The rest of the environment: `CUT_RELEASE_SIBLINGS=$T/none` (an empty dir),
`CUT_RELEASE_DATE=2026-11-01`, `CUT_RELEASE_POLL_SECONDS=0`,
`CUT_RELEASE_CACHE=$T/cache`, `CARGO_TARGET_DIR=$T/target`,
`CUT_RELEASE_PUSH_VIA_STAGING=$T/bin/fake-pvs`,
`CUT_RELEASE_PUBKEY_FILE=$T/key/pub` (made by `minisign -G -W -p $T/key/pub -s $T/key/sec`),
and the git identity `Test <t@example.invalid>`. Network rows unset the
two test seams, so they use the real key and push-via-staging. Each
fixture's origin is the bare repo `$T/remote/<repo>.git` (HEAD = the §3
default). Every fixture change is committed and pushed (helper
`fx_commit <msg>`), so the tree is clean and HEAD = origin.

**Fixtures.**
- **F-ms** (mnemonic-secret, master). `Cargo.toml` is a workspace with
  `crates/ms-codec` (lib 0.10.1, empty `lib.rs`) and `crates/ms-cli` (0.20.2,
  `[[bin]] name = "ms"`, `ms-codec = { path = "../ms-codec", version = "=0.10.1" }`,
  `main.rs` printing `ms {CARGO_PKG_VERSION}`). Lock from
  `cargo generate-lockfile --offline`. Tags `ms-cli-v0.20.2` and
  `ms-codec-v0.10.1`. `CHANGELOG.md` lines 1-13:
  `# Changelog`, ``, `## ms-cli [Unreleased]`, ``, `- a fix.`, ``,
  `## ms-cli [0.20.2] — 2026-11-01`, ``, `- old.`, ``,
  `## ms-codec [0.10.1] — 2026-11-01`, ``, `- old.`.
- **F-mt** (mnemonic-transaction, main): mt-codec 0.1.0 (lib); mt-cli 0.1.1
  (`bin mt`, `mt-codec = { path = "../mt-codec", version = "0.1.0" }`); no
  CHANGELOG; tag `mt-cli-v0.1.1`.
- **F-tk** (mnemonic-toolkit, master): crate mnemonic-toolkit 0.105.1 (`bin
  mnemonic`); both READMEs `<!-- toolkit-version: 0.105.1 -->`. `gen.sh`
  holds toolkit master's lines 3, 50, 115, 132 and 721; `Examples.md`
  holds master's lines 3, 20, 107, 121, 122, 125, 166 and 1336 (copied
  byte-exact at harness-writing time). `scripts/install.sh` holds
  `component_info` with the one `mnemonic)` arm naming
  `mnemonic-toolkit-v0.105.1`, and `first_signed` with
  `        mnemonic)     echo "" ;;`. `docs/manual-gui/pinned-upstream.toml`
  is `[manual-gui]` + `toolkit-tag-implied = "mnemonic-toolkit-v0.105.1"`.
  The stub `docs/manual-gui/tests/check_cli_pins.py` prints `OK: stub` and
  exits 0 when `[manual-gui]` equals install.sh's pin, or when
  `[installer-ahead]` equals it and some file under `docs/manual-gui/src`
  holds `{#installer-ahead}`. Else it prints
  `  pinned-upstream.toml [manual-gui] toolkit-tag-implied = mnemonic-toolkit-v0.105.1, but install.sh pins <pin>`
  and exits 1. The stub `scripts/install-assets.test.sh` prints
  `[install-assets.test] OK`, or `[install-assets.test] FAILED` and exits 1
  when `FAKE_INSTALL_ASSETS=fail`. `CHANGELOG.md`: `# Changelog`, ``,
  `## [Unreleased]`, ``, `- a toolkit fix.`, ``,
  `## mnemonic-toolkit [0.105.1] — 2026-09-25`, ``, `- old.`. Tag
  `mnemonic-toolkit-v0.105.1`.
- **fake-pvs** first appends `$* <sha of HEAD>` to `$T/pvs.log`, which the harness empties
  before each row. `FAKE_PVS=ok` (default) runs `git push origin HEAD:refs/heads/$1`
  and prints `== OK: <sha> is on $1, CI green, no bypass`. `red` prints
  `FATAL: CI is not green for <sha> -- NOT pushing $1.` and exits 1.
  `bypass` pushes, prints `remote: Bypassed rule violations for refs/heads/$1:`
  then the OK line, and exits 0. `silent` pushes, prints nothing, and exits 0.
  `landed-fail` pushes, prints `FATAL: origin/$1 is not <sha> after the push`,
  and exits 1.
- **fake gh** logs every call to `$T/gh.log`, which the harness empties
  before each row, and answers exactly the calls
  of §5.3, §5.4, §6.3 and §7. Any other call prints
  `fake gh: unexpected: <args>` and exits 99. Runs: one per name in
  `FAKE_GH_WORKFLOWS` (default: the tag's §3 list plus `fuzz-smoke`), with
  ids 1001…, `headSha` from the bare remote, `status` completed, and
  `conclusion` `$FAKE_GH_CONCLUSION` (default success). Releases come from
  `$T/assets/<tag>/`. That dir is built by `mk_assets` from the archive
  (an executable `<bin>` shell script printing `<bin> <N>`), the §3 sums
  file, and `<sums>.minisig` signed with `$T/key/sec` and trusted comment
  `<bin> <tag> <sums>`. `pr create` prints
  `https://github.com/bg002h/mnemonic-toolkit/pull/77`. `pr view` and
  `pr checks` answer from `FAKE_PR_*` env; `pr view` always answers
  `"isCrossRepository": false`. `pr merge` does
  `git merge --no-ff` of the head in a scratch clone of the bare remote,
  then pushes and deletes the clone (guarded as in Harness setup).
  `FAKE_GH_KILL=TERM`: on the first `run list` call, runs
  `kill -TERM "$CUT_RELEASE_TEST_PID"`. `FAKE_GH_DRAFT=1`: `release view`
  answers `isDraft: true`. `FAKE_GH_FAIL=1`: on every `run list` call fake gh prints `fake gh: simulated failure` to stderr and exits 1 (the script makes no `run list` call before the tag push).
  `FAKE_PR_MERGE_STATE=<s>`: `pr view` answers that
  merge state (default CLEAN). `FAKE_PR_BRANCH_MOVES=1`: after merging,
  `pr merge` pushes one more commit to the head branch. `mk_assets` env
  `MK_ASSETS_COMMENT=<c>` sets the trusted comment; `MK_ASSETS_TAMPER=1`
  appends a byte to the archive after the sums are written;
  `MK_ASSETS_BINVER=<v>` makes the archived bin print `<bin> <v>`.

Offline rows (F-ms unless named). Each row is a command, then its exit
code and what it shows:

| # | row | setup → command | expect |
|---|---|---|---|
| 1 | cr_usage_no_mode | `cut-release.sh ms-cli-v0.20.3` | 2; stderr U1 text |
| 2 | cr_usage_two_modes | `--dry-run --push ms-cli-v0.20.3` | 2; U1 |
| 3 | cr_usage_no_tags | `--dry-run` | 2; U2 |
| 4 | cr_help | `--help` | 0; stdout line 1 = `usage: cut-release.sh --dry-run [--trailer 'Key: value']... <tag>...` |
| 5 | cr_verify_lib_tag | `--verify ms-codec-v0.10.1` | 2; `cut-release: ms-codec-v0.10.1 is a library tag; it has no release assets to verify` |
| 6 | cr_not_in_table | a bare-origin repo named mnemonic-refugium; `--dry-run mr-cli-v0.1.0` | 3; `cut-release: origin is '<T>/remote/mnemonic-refugium.git'; 'mnemonic-refugium' is not in the release table` |
| 7 | cr_wrong_prefix | `--dry-run mk-cli-v0.14.0` | 3; `cut-release: 'mk-cli-v0.14.0' is not a release tag of mnemonic-secret (its prefixes: ms-cli-v ms-codec-v)` |
| 8 | cr_not_semver | `--dry-run ms-cli-v0.21` | 3; `cut-release: 'ms-cli-v0.21' does not end in a MAJOR.MINOR.PATCH version` |
| 9 | cr_duplicate_crate | `--dry-run ms-cli-v0.20.3 ms-cli-v0.20.4` | 3; `cut-release: two tags for ms-cli: ms-cli-v0.20.3 ms-cli-v0.20.4` |
| 10 | cr_not_default_branch | `git switch -c topic` | 3; `cut-release: on branch 'topic'; release from master (git switch master)` |
| 11 | cr_behind_origin | push one commit to the bare remote from a second clone | 3; `cut-release: HEAD <12> is not origin/master <12>; the release commit must sit directly on the remote tip (git pull --ff-only, or land your commits first)` |
| 12 | cr_dirty_tracked | append to `crates/ms-cli/src/main.rs`, no commit | 3; P13 line, then `   crates/ms-cli/src/main.rs` |
| 13 | cr_tag_on_origin | second clone pushes annotated `ms-cli-v0.20.3` | 3; `cut-release: tag ms-cli-v0.20.3 already exists on origin` |
| 14 | cr_not_newer_than_cargo | `--dry-run ms-cli-v0.20.1` | 3; `cut-release: ms-cli-v0.20.1: ms-cli is 0.20.2 in crates/ms-cli/Cargo.toml; 0.20.1 is not newer` |
| 15 | cr_not_newer_than_tag | origin gets tag `ms-cli-v0.20.5`; `--dry-run ms-cli-v0.20.4` | 3; `cut-release: ms-cli-v0.20.4: origin already has ms-cli-v0.20.5, which is not older than 0.20.4` |
| 16 | cr_no_unreleased | delete lines 3-6 | 3; `cut-release: CHANGELOG.md has no '## ms-cli [Unreleased]' section; the fix PR writes the release notes there` |
| 17 | cr_unreleased_empty | delete line 5 | 3; `cut-release: CHANGELOG.md:3: the '## ms-cli [Unreleased]' section is empty; nothing to release for ms-cli` |
| 18 | cr_unreleased_below_release | move lines 3-6 below line 10 | 3; `cut-release: CHANGELOG.md:7: '## ms-cli [Unreleased]' sits below '## ms-cli [0.20.2] — 2026-11-01'; it must be above every release heading` |
| 19 | cr_unreleased_and_heading | insert `## ms-cli [0.20.3] — 2026-11-01`, ``, `- x.`, `` above line 7 | 3; `cut-release: CHANGELOG.md has both '## ms-cli [0.20.3]' and entries under '## ms-cli [Unreleased]'` |
| 20 | cr_next_release_mismatch | line 5 becomes `**Next release: 0.21.0** — x.` | 3; `cut-release: CHANGELOG.md:3: '## ms-cli [Unreleased]' says 'Next release: 0.21.0', not 0.20.3` |
| 21 | cr_codec_entries_unreleased | insert `## ms-codec [Unreleased]`, ``, `- a codec fix.`, `` above line 7; `--dry-run ms-cli-v0.20.3` | 3; `cut-release: ms-codec has entries under '## ms-codec [Unreleased]' and ms-cli builds it by path; tag ms-codec in this same run (ms-codec-vMAJOR.MINOR.PATCH) or move those entries` |
| 22 | cr_stale_pin | ms-cli pin `=0.10.0`; codec U with entries; `--dry-run ms-codec-v0.10.2` | 3; `cut-release: crates/ms-cli/Cargo.toml pins ms-codec at '=0.10.0'; expected '=0.10.1', '^0.10.1' or no version (fix the pin by hand first)` |
| 23 | cr_toolchain_mismatch | add `rust-toolchain.toml` with `[toolchain]` + `channel = "1.85.0"`; put first on PATH a fake `cargo` that prints `cargo 9.9.9 (fake)` for `--version` | 3; `cut-release: cargo --version is 'cargo 9.9.9 (fake)' but rust-toolchain.toml pins 1.85.0; put rustup's cargo first on PATH (PATH=$HOME/.cargo/bin:$PATH)` |
| 24 | cr_push_tooling_dirty | copy cut-release.sh, push-via-staging.sh and minisign.pub into a committed scratch repo `$T/eng`; modify `$T/eng/minisign.pub`; run `$T/eng/scripts/cut-release.sh --push ms-cli-v0.20.3` | 3; `cut-release: minisign.pub in <T>/eng has uncommitted changes; a release runs only committed tooling` |
| 25 | cr_dry_run_ms_two_tags | codec U with entries (as row 21); `--dry-run ms-codec-v0.10.2 ms-cli-v0.20.3` | 0. Shows: `== ms-codec 0.10.1 -> 0.10.2 (crates/ms-codec/Cargo.toml)`; `== ms-cli 0.20.2 -> 0.20.3 (crates/ms-cli/Cargo.toml)`; `== pin: crates/ms-cli/Cargo.toml ms-codec =0.10.1 -> =0.10.2`; `== CHANGELOG: CHANGELOG.md '## ms-codec [Unreleased]' -> '## ms-codec [0.10.2] — 2026-11-01'`; `== CHANGELOG: CHANGELOG.md '## ms-cli [Unreleased]' -> '## ms-cli [0.20.3] — 2026-11-01'`; `== Cargo.lock: 2 version line(s) changed (ms-codec ms-cli)`; `== built: ms --version = 'ms 0.20.3'`; `DRY RUN: would commit 4 file(s): CHANGELOG.md Cargo.lock crates/ms-cli/Cargo.toml crates/ms-codec/Cargo.toml`; `DRY RUN: would run push-via-staging.sh master`; `DRY RUN: would tag ms-codec-v0.10.2 on the release commit and push refs/tags/ms-codec-v0.10.2`; `DRY RUN: would wait for ms-codec-v0.10.2 workflows: none`; `DRY RUN: would wait for ms-cli-v0.20.3 workflows: release man-release`; `DRY RUN: would check ms-codec-v0.10.2 points at the release commit`; `DRY RUN: would verify ms-cli-v0.20.3: ms-0.20.3-x86_64-linux-musl.tar.gz in SHA256SUMS.x86_64, every SHA256SUMS* signed, 'ms 0.20.3'`; `     mnemonic-engrave: not found at <T>/none/mnemonic-engrave; check it by hand`; `== DRY RUN OK: nothing was pushed; tracked files are as they were`. After: `git status --porcelain` empty; `$T/gh.log` and `$T/pvs.log` absent or empty; the bare remote's `git ls-remote` output equals its pre-run output; HEAD unchanged; no local tag named in the command |
| 26 | cr_dry_run_resume | commit ms-cli 0.20.3 + the promoted CHANGELOG by hand (lock updated); `--dry-run ms-cli-v0.20.3` | 0; `== ms-cli is already 0.20.3 (crates/ms-cli/Cargo.toml); no version edit`, `== CHANGELOG: CHANGELOG.md already has '## ms-cli [0.20.3]'`, `DRY RUN: nothing to commit; would stage HEAD <12> as it is`. After: `$T/gh.log` and `$T/pvs.log` absent or empty; the bare remote's `git ls-remote` output equals its pre-run output; HEAD unchanged; no local tag named in the command |
| 27 | cr_wrong_version_output | main.rs prints `ms 9.9.9`; `--dry-run ms-cli-v0.20.3` | 4; `cut-release: ms --version printed 'ms 9.9.9', not 'ms 0.20.3'`, last line `cut-release: tracked files restored; nothing was committed or pushed`; tree clean |
| 28 | cr_fn_lock_diff_check | source the script; old lock vs a copy with ms-cli's version line and one dependency line changed | rc 1; `cut-release: Cargo.lock changed beyond the bumped crates' version lines (2 line(s) changed, 1 expected):` |
| 29 | cr_fn_promote_plain | `changelog_promote` on `## Unreleased`,``,`- x.`,``,`## mnemonic-gui [0.63.0] — 2026-09-25` with `## mnemonic-gui [0.64.0] — 2026-11-01` | the file is exactly `## Unreleased`,``,`## mnemonic-gui [0.64.0] — 2026-11-01`,``,`- x.`,``,`## mnemonic-gui [0.63.0] — 2026-09-25` |
| 30 | cr_fn_promote_next_release | `## [Unreleased]`,``,`**Next release: 0.14.0** — a.`,`b.`,``,`### Changed`,`- y`,``,`## [0.13.0] — 2026-08-19` with `## [0.14.0] — 2026-11-01` | exactly `## [Unreleased]`,``,`## [0.14.0] — 2026-11-01`,``,`### Changed`,`- y`,``,`## [0.13.0] — 2026-08-19` |
| 31 | cr_fn_tk_examples | `tk_examples_b`, then `tk_examples_c`, X=0.105.1 N=0.105.2, on F-tk's gen.sh and Examples.md | after b: gen.sh 5 lines changed, 0 X left; Examples 4 changed, 4 X lines left, each containing `mnemonic-toolkit-v0.105.1` or `'mnemonic 0.105.1'`. After c: 0 X left, 8 changed in total |
| 32 | cr_fn_tk_repad | `tk_examples_c` X=0.105.1 N=0.1000.0 | the --list line = `mnemonic        mnemonic-toolkit     (none)         mnemonic-toolkit-v0.1000.0           mnemonic-0.1000.0-x86_64-linux-musl.tar.gz` |
| 33 | cr_push_ms_happy | codec U with entries; `--push --trailer 'Co-Authored-By: Test <t@example.invalid>' ms-codec-v0.10.2 ms-cli-v0.20.3`; assets for ms-cli-v0.20.2 (prev) and ms-cli-v0.20.3 | 0; bare master = the release commit, subject `release: ms-codec 0.10.2 + ms-cli 0.20.3`, trailer present; both tags annotated and peeling to it; the committed CHANGELOG lines 3-9 are `## ms-cli [Unreleased]`,``,`## ms-cli [0.20.3] — 2026-11-01`,``,`- a fix.`,``,`## ms-codec [Unreleased]`; shows `   ok    RAN ms --version -> ms 0.20.3`, `   not waited: fuzz-smoke run 1003 completed/success`, `== FREEZE LIFTED`, `== DONE: ms-codec-v0.10.2 ms-cli-v0.20.3 published and verified`; gh.log has no `fake gh: unexpected` |
| 34 | cr_push_staging_red | `FAKE_PVS=red`, `--push ms-cli-v0.20.3` | 5; `cut-release: push-via-staging.sh exited 1; no tag was pushed.` and `cut-release: origin/master is unchanged. To retry, drop the local commit and run again: git reset --hard origin/master`; no tag on the bare remote |
| 35 | cr_push_bypass_line | `FAKE_PVS=bypass` | 5; `cut-release: push-via-staging.sh output contains 'Bypassed rule violations'; not tagging. STOP and report it.`; no tag on the bare remote |
| 36 | cr_push_run_failed | `FAKE_GH_CONCLUSION=failure` | 7; `cut-release: run 1001 (release, ms-cli-v0.20.3) concluded 'failure': <url>`, `cut-release: the tags are pushed and stay pushed: never delete or move them.`, `== follow-on pins are NOT listed: the release is not verified`; the tag is on the bare remote; it also shows the finish line |
| 37 | cr_push_bad_signature | ms-cli-v0.20.3 signed with a second key | 8; a line starting `   FAIL  SHA256SUMS.x86_64.minisig does not verify:` |
| 38 | cr_push_missing_minisig | no `.minisig` in assets | 8; `   FAIL  no SHA256SUMS.x86_64.minisig asset (unsigned release)` |
| 39 | cr_push_missing_asset | prev has `ms-0.20.2-macos-arm64.tar.gz`, new lacks `ms-0.20.3-macos-arm64.tar.gz` | 8; `   FAIL  missing vs ms-cli-v0.20.2: ms-0.20.3-macos-arm64.tar.gz` |
| 40 | cr_toolkit_dry_run | F-tk; `--dry-run mnemonic-toolkit-v0.105.2` | 0; `== toolkit (b): README markers 2, .examples-build/gen.sh 5 line(s), Examples.md 4 line(s); 4 installer line(s) left for (c)`, `== toolkit (c): install.sh self-pin mnemonic-toolkit-v0.105.1 -> mnemonic-toolkit-v0.105.2; first_signed mnemonic "" -> "0.105.2"; Examples.md 4 installer line(s); [installer-ahead] toolkit-tag-implied = "mnemonic-toolkit-v0.105.2"`, `== toolkit (c): check_cli_pins.py fails on (c); the pins PR owes the GUI manual's {#installer-ahead} section (listed below)`, `DRY RUN: would commit (b) 7 file(s): .examples-build/Examples.md .examples-build/gen.sh CHANGELOG.md Cargo.lock README.md crates/mnemonic-toolkit/Cargo.toml crates/mnemonic-toolkit/README.md`, `DRY RUN: would commit (c) 3 file(s): .examples-build/Examples.md docs/manual-gui/pinned-upstream.toml scripts/install.sh`; tree clean. After: `$T/gh.log` and `$T/pvs.log` absent or empty; the bare remote's `git ls-remote` output equals its pre-run output; HEAD unchanged; no local tag named in the command; no bare `release-pins-*` |
| 41 | cr_toolkit_pins_ok | F-tk plus a committed `docs/manual-gui/src/82.md` holding `{#installer-ahead}` | 0; `== toolkit (c): check_cli_pins.py OK` |
| 42 | cr_toolkit_push_happy | F-tk; `--push mnemonic-toolkit-v0.105.2` | 0; bare master = (b); (c)'s parent = (b); the tag peels to (c); bare branch `release-pins-0.105.2` = (c); gh.log has `pr create` with `--draft` and `--body-file <git-dir>/cut-release/pins-pr-body.md`, and that file contains `land it: cut-release.sh --land-pr <PR number>`; shows `== toolkit: pins PR https://github.com/bg002h/mnemonic-toolkit/pull/77 is open as a draft; it owes the text above. Its author marks it ready, then: cut-release.sh --land-pr 77`; at exit, local master = bare master = (b) |
| 43 | cr_toolkit_install_assets_fail | as 42 with `FAKE_INSTALL_ASSETS=fail` | 9; `cut-release: scripts/install-assets.test.sh failed on (c) <12>; the pins PR was not opened. The tag is published. Fix forward on top of (c): git push origin <c sha>:refs/heads/release-pins-0.105.2, commit the fix to that branch, then gh pr create --repo bg002h/mnemonic-toolkit --base master --head release-pins-0.105.2 --draft --title 'pins: installer -> mnemonic-toolkit-v0.105.2' --body-file <git-dir>/cut-release/pins-pr-body.md (a PR without (c) leaves the tag off master).`; no `pr create` in gh.log |
| 44 | cr_land_pr_draft | `FAKE_PR_DRAFT=true`; `--land-pr 77` | 3; `cut-release: PR #77 is a draft; its author marks it ready (gh pr ready 77) once the owed edits are in` |
| 45 | cr_land_pr_failed_check | one check bucket `fail` named `manual-gui` | 9; `cut-release: PR #77 check 'manual-gui' is fail: <link>` |
| 46 | cr_land_pr_happy | after row 42; PR ready, checks pass/skipping, state CLEAN | 0; bare master is a merge commit with (c) as a parent; branch `release-pins-0.105.2` gone; `== merged PR #77 (<url>) as a merge commit; release-pins-0.105.2 deleted`; gh.log has exactly one `pr merge` line, `pr merge 77 --repo bg002h/mnemonic-toolkit --merge --match-head-commit <c sha>`. The fake `pr merge` answers only that argv shape; any other prints `fake gh: unexpected: <args>` and exits 99 |
| 47 | cr_land_pr_tag_not_in_pr | head = a commit not containing the tag | 9; `cut-release: PR #77 does not contain mnemonic-toolkit-v0.105.2 (<12>); merging it would leave the tag off master` |
| 48 | cr_dry_run_mt | F-mt; `--dry-run mt-cli-v0.1.2` | 0; `== CHANGELOG: none in mnemonic-transaction`, `== Cargo.lock: 1 version line(s) changed (mt-cli)`, `DRY RUN: would commit 2 file(s): Cargo.lock crates/mt-cli/Cargo.toml`, `DRY RUN: would wait for mt-cli-v0.1.2 workflows: release`. After: `$T/gh.log` and `$T/pvs.log` absent or empty; the bare remote's `git ls-remote` output equals its pre-run output; HEAD unchanged; no local tag named in the command |
| 49 | cr_static_push | `grep -nE '\bgit push\b' scripts/cut-release.sh`; L1 = its output lines that match `^[0-9]+:cut-release: `, L2 = the other lines | L2: exactly 3 lines. One contains `--atomic origin` (its refs are `refs/tags/<tag>`), one contains `refs/heads/release-pins-`, one contains `origin --delete`. None contains `--tags`, `--force`, ` -f `, `HEAD:` or `--mirror` (default-branch pushes happen only inside push-via-staging). L1: after its `<n>:`, every line starts with `cut-release: otherwise finish by hand: `, `cut-release: pins PR: `, `cut-release: scripts/install-assets.test.sh failed on (c) ` or `cut-release: origin has ci/staging at `, and no line contains `"`, `$(` or a backtick (§2 code shape) |
| 50 | cr_bash_n | `bash -n` on the script and the harness | rc 0 |
| 51 | cr_ahead_of_origin | one local commit, subject `release: ms-cli 0.20.3`, not pushed; `--dry-run ms-cli-v0.20.3` | 3; the P12a text with k = 1 (I-2) |
| 52 | cr_toolkit_already_bumped | F-tk with the crate at 0.105.2 (fx_commit); `--dry-run mnemonic-toolkit-v0.105.2` | 3; `cut-release: mnemonic-toolkit is already 0.105.2 in crates/mnemonic-toolkit/Cargo.toml; the toolkit release needs its version bump in commit (b), so release it by hand or restore the previous version` |
| 53 | cr_toolkit_selfpin_lags | F-tk with install.sh naming `mnemonic-toolkit-v0.105.0`; `--dry-run mnemonic-toolkit-v0.105.2` | 3; `cut-release: scripts/install.sh self-pins mnemonic-toolkit-v0.105.0 but Cargo.toml says 0.105.1; land the previous release's pins PR first` |
| 54 | cr_push_pvs_no_ok_line | `FAKE_PVS=silent`; `--push ms-cli-v0.20.3` | 5; `cut-release: push-via-staging.sh exited 0 without its '== OK:' line; not tagging`; no tag on the bare remote |
| 55 | cr_push_pvs_failed_after_landing | `FAKE_PVS=landed-fail`; `--push ms-cli-v0.20.3`; then the same command with `FAKE_PVS=ok` | first run: 5; `cut-release: push-via-staging.sh exited 1; no tag was pushed.` and `cut-release: the release commit IS on origin/master. If the output above shows a bypass, STOP and report it; otherwise re-run the same command: it finds the commit on master, re-stages it and tags.`; no tag. Second run: 0; `== ms-cli is already 0.20.3 (crates/ms-cli/Cargo.toml); no version edit`; the tag peels to the first run's commit; bare master unchanged by the second run |
| 56 | cr_toolkit_pvs_failed_after_landing | F-tk; `FAKE_PVS=landed-fail`; `--push mnemonic-toolkit-v0.105.2`; then run the printed `git tag -a … && git push …` | 5; the two I-1 lines with (b)'s and (c)'s `<12>`; `<git-dir>/cut-release/pins-pr-body.md` exists; no tag. After the printed commands run: the tag peels to (c) |
| 57 | cr_push_tag_push_fails | the bare remote gets a pre-receive hook that rejects `refs/tags/*`; `--push ms-cli-v0.20.3` | 6; `cut-release: pushing the tags failed (atomic; none is on origin):` and `cut-release: re-run the same command; it finds the release commit on master and tags it`; no local tag; bare master = the release commit |
| 58 | cr_push_run_never_appears | `FAKE_GH_WORKFLOWS=fuzz-smoke`; `--push ms-cli-v0.20.3` | 7; `cut-release: no 'release' run appeared for ms-cli-v0.20.3 within 10 minutes`; the finish line |
| 59 | cr_push_interrupted_after_tag | `FAKE_GH_KILL=TERM`; run as `bash -c 'export CUT_RELEASE_TEST_PID=$$; exec <script> --push ms-cli-v0.20.3'` | 7; stderr ends with the interrupted line and the finish line; line 1 of `<git-dir>/cut-release/follow-on.txt` is `== FOLLOW-ON: not done by this script; each is a commit or PR in the named repo`; the tag is on the bare remote |
| 60 | cr_toolkit_run_failed | F-tk; `FAKE_GH_CONCLUSION=failure`; `--push mnemonic-toolkit-v0.105.2` | 7; shows the pins-PR line with (c)'s sha; local master = bare master = (b); no bare `release-pins-0.105.2` |
| 61 | cr_push_release_draft | `FAKE_GH_DRAFT=1`; `--push ms-cli-v0.20.3` | 8; `   FAIL  release is a draft` |
| 62 | cr_push_trusted_comment | ms-cli-v0.20.3 assets with `MK_ASSETS_COMMENT='ms ms-cli-v0.20.2 SHA256SUMS.x86_64'` | 8; `   FAIL  SHA256SUMS.x86_64 trusted comment is 'ms ms-cli-v0.20.2 SHA256SUMS.x86_64', not '... ms-cli-v0.20.3 SHA256SUMS.x86_64'` |
| 63 | cr_push_checksum_mismatch | `MK_ASSETS_TAMPER=1` | 8; `   FAIL  ms-0.20.3-x86_64-linux-musl.tar.gz does not match SHA256SUMS.x86_64` |
| 64 | cr_push_wrong_binary_version | `MK_ASSETS_BINVER=0.20.2` | 8; `   FAIL  ms --version printed 'ms 0.20.2', not 'ms 0.20.3'` |
| 65 | cr_version_skip | `--dry-run ms-cli-v0.20.4` | 3; `cut-release: ms-cli-v0.20.4: ms-cli is 0.20.2 in crates/ms-cli/Cargo.toml; the next version is one of 0.20.3 0.21.0 1.0.0 (a skip is released by hand)` (I-4) |
| 66 | cr_land_pr_not_clean | row 42's state; PR ready, checks pass, `FAKE_PR_MERGE_STATE=BLOCKED`; `--land-pr 77` | 9; `cut-release: PR #77 merge state is BLOCKED, not CLEAN`; no `pr merge` in gh.log |
| 67 | cr_land_pr_branch_moved | as row 46, with `FAKE_PR_BRANCH_MOVES=1` | 0; `== merged PR #77 (<url>) as a merge commit; release-pins-0.105.2 left in place (origin's release-pins-0.105.2 is <12>, not the merged <12>)`; the bare branch still exists (I-5) |
| 68 | cr_follow_on_engrave_demo | `CUT_RELEASE_SIBLINGS=$T/sib`, holding an engrave clone whose two demo files are engrave master's ms lines, copied byte-exact as for F-tk, plus a modified tracked `README.md`; `--dry-run ms-cli-v0.20.3`; then, in `$T/sib/mnemonic-engrave`, run the printed (a) `sed` and `git commit` lines | 0; the commit changes exactly the two demo files, 7 lines; `README.md` is still modified and uncommitted; `grep -c 'ms-cli-v0.20.2'` is 0 in both files (I-6) |
| 69 | cr_stale_staging | a second clone pushes a commit that is not an ancestor of the bare master tip to `refs/heads/ci/staging` of the bare remote; `--push ms-cli-v0.20.3`, then `--dry-run ms-cli-v0.20.3` | 3 both times; `cut-release: origin has ci/staging at <12>; push-via-staging.sh cannot push a new release commit over it (an interrupted or failed staging run left it). If no staging run is in progress, delete it, then run again: git push origin --delete ci/staging` (`<12>` = the unrelated commit); no tag on the bare remote; bare master unchanged; HEAD unchanged; tree clean; `$T/pvs.log` absent or empty; then, with ci/staging set to the sha of HEAD and then to its parent, --dry-run ms-cli-v0.20.3 exits 0 both times with $T/pvs.log absent or empty |
| 70 | cr_push_gh_fails_after_tag | `FAKE_GH_FAIL=1`; `--push ms-cli-v0.20.3`; then F-tk with `--push mnemonic-toolkit-v0.105.2` | 7 both times; stderr shows `cut-release: stopped after the tag push by a failing command (above); the tags are pushed and stay pushed: never delete or move them.` and the finish line (the toolkit run also the pins-PR line with (c)'s sha); the tag is on the bare remote; `<git-dir>/cut-release/follow-on.txt` exists; toolkit: local master = bare master = (b) |
| 71 | cr_cache_root_slash | `CUT_RELEASE_CACHE=/`; `--verify ms-cli-v0.20.2`, run from `$T/none` | 2; `cut-release: CUT_RELEASE_CACHE must be an absolute path other than / (the cache directory is '/')`; stdout has no line starting `== verify`; `$T/gh.log` absent or empty |
| 72 | cr_cache_root_relative | `CUT_RELEASE_CACHE=cache`; `--verify ms-cli-v0.20.2`, run from `$T/none`; then F-ms ready to push, as row 36 (`cr_push_run_failed`) sets it up, `CUT_RELEASE_CACHE=cache`, `--push ms-cli-v0.20.3` | 2; the same line with `'cache'`; stdout has no line starting `== verify`; `$T/none/cache` does not exist; the push: 2, the same line, nothing changed (tree clean, HEAD and the bare remote unchanged, gh.log and pvs.log empty, no local or remote tag `ms-cli-v0.20.3`) |
| 73 | cr_cache_root_empty | source the script, `set +e`, `CR_CACHE=''`, call `verify_tag ms-cli-v0.20.2` (the command line cannot reach this: `${CUT_RELEASE_CACHE:-...}` turns an empty variable into the default) | 2; the same line with `''`; stdout has no line starting `== verify` |
| 74 | cr_cache_tag_slash | `$T/cache` and `$T/escape/ms-cli-v0.20.2/keep` exist. Source the script, `set +e`, call `verify_tag '../escape/ms-cli-v0.20.2'`; then `cache_guard <tag> "$CR_CACHE/<tag>"` for the tags `.`, `..` and `a/b`; then the command line `--verify 'x/ms-cli-v0.20.2'` | verify_tag: 2; `cut-release: '../escape/ms-cli-v0.20.2' cannot name a cache directory: it has a '/' or is '.' or '..'`; `$T/escape/ms-cli-v0.20.2/keep` still exists. Each cache_guard: 2 and that line for its tag. Command line: 2; `cut-release: 'x/ms-cli-v0.20.2' is not a release tag of any repo in the table` (U8; it never reaches verify_tag) |
| 75 | cr_cache_dir_not_below | source the script, `set +e`; `cache_guard ms-cli-v0.20.2 <dir>` for <dir> = `$T/cache`, `$T/cache/other`, `$T`, `$T/cache/ms-cli-v0.20.2/x`; then `cache_guard '' $T/cache/` (an empty tag, which only check (3) catches); then `cache_guard ms-cli-v0.20.2 $T/cache/ms-cli-v0.20.2` | the first four: 2; `cut-release: refusing to delete '<dir>': it is not exactly '<T>/cache/ms-cli-v0.20.2' strictly below the cache`. The empty tag: 2; `cut-release: refusing to delete '<T>/cache/': it is not exactly '<T>/cache/' strictly below the cache`. The last: 0 and nothing on stderr |
| 76 | cr_harness_rm_guards | `rm_T` and `rm_below_T` called in subshells. A scratch directory `<cache-root>/guard.XXXXXX` (made by `mktemp -d`; it matches no `*/cut-release-test/run-*`) holds a file `keep`; `$T/guard/cut-release-test/run-1` holds `sub`; `$T/guard/outside` exists | with `T` = the scratch directory: `rm_T`, and `rm_below_T <scratch>/x`: 2; `cut-release.test: refusing to delete '<scratch>': T is not */cut-release-test/run-*`; `keep` survives. `rm_T` with `T` empty: 2, the same line with `''`. With `T` = `run-1`: `rm_below_T` of `outside`, of `run-1` itself and of `''`: 2; `cut-release.test: refusing to delete '<path>': it is not strictly below <run-1>`; of `run-1/sub/../../outside`: 2; `cut-release.test: refusing to delete '<path>': it has a '..' component`; `outside` and `sub` survive. `rm_below_T run-1/sub`: 0, `sub` gone. `rm_T`: 0, `run-1` gone. The scratch directory is removed at the end of the row with `rm -f` and `rmdir` |
| 77 | cr_static_rm | `grep -nE '(^|[^[:alnum:]_])rm +(-[A-Za-z]*[rR][A-Za-z]*\|--recursive)( \|$)'` over `scripts/cut-release.sh`, the harness, `fixtures/cut-release/fake-gh` and `fake-pvs`, comment lines dropped | exactly 4 lines (1 in the script, 2 in the harness, 1 in fake-gh), each matching `rm -<flags> -- "${NAME:?}"` |

Network rows (skipped by `--offline`; need gh auth and rustup's cargo
first on PATH; the real key and push-via-staging):

| # | row | command | expect |
|---|---|---|---|
| 78 | crn_verify_ms | `--verify ms-cli-v0.20.2` | 0; `   ok    SHA256SUMS.x86_64 signed: Trusted comment: ms ms-cli-v0.20.2 SHA256SUMS.x86_64` (same for aarch64, portable), `   ok    every asset of ms-cli-v0.20.1 is present (names with 0.20.1 -> 0.20.2)`, `   ok    ms-0.20.2-x86_64-linux-musl.tar.gz matches SHA256SUMS.x86_64`, `   ok    RAN ms --version -> ms 0.20.2`, `== verify ms-cli-v0.20.2: 7 ok, 0 failed` |
| 79 | crn_verify_mt | `--verify mt-cli-v0.1.1` | 0; `   ok    SHA256SUMS.portable signed: Trusted comment: mt mt-cli-v0.1.1 SHA256SUMS.portable`, `== verify mt-cli-v0.1.1: 5 ok, 0 failed` |
| 80 | crn_verify_md_unsigned | `--verify descriptor-mnemonic-md-cli-v0.20.3` | 8; `   FAIL  no SHA256SUMS.aarch64.minisig asset (unsigned release)`, `   FAIL  no SHA256SUMS.portable.minisig asset (unsigned release)`, `   ok    RAN md --version -> md 0.20.3`, `== verify descriptor-mnemonic-md-cli-v0.20.3: 4 ok, 2 failed` |
| 81 | crn_verify_engrave_retired | `--verify v0.12.0` | 8; `   FAIL  SHA256SUMS.minisig does not verify:` then a line containing `Signature key id in SHA256SUMS.minisig is CA39ECB257009A0F`; `   ok    RAN me --version -> me 0.12.0` |
| 82 | crn_verify_all | `--verify` | exit 0 iff the last line is `== verify: 7 release(s), 0 failed`; otherwise 8; exactly 7 `== verify <tag>:` lines |
| 83 | crn_real_mt_dry | clone `https://github.com/bg002h/mnemonic-transaction` (`--reference-if-able $CUT_RELEASE_SIBLINGS_REAL/mnemonic-transaction`); make a bare copy the clone's origin; `cargo fetch --locked`; X from the manifest, N = X's triple with patch+1 (X's triple if X has `-`); `--dry-run mt-cli-v<N>` | 0; `== mt-cli <X> -> <N> (crates/mt-cli/Cargo.toml)`, `== built: mt --version = 'mt <N>'`, `== DRY RUN OK: nothing was pushed; tracked files are as they were`; tree clean |
| 84 | crn_real_ms_dry | as 76 for mnemonic-secret; in the clone, insert `## ms-codec [Unreleased]`,``,`- test.`,`` and `## ms-cli [Unreleased]`,``,`- test.`,`` above the first `## ` line, commit, push to the bare copy; `--dry-run ms-codec-v<Nc> ms-cli-v<N>` | 0; `== pin: crates/ms-cli/Cargo.toml ms-codec =<Xc> -> =<Nc>`, `== Cargo.lock: 2 version line(s) changed (ms-codec ms-cli)`, `== built: ms --version = 'ms <N>'`; tree clean |
| 85 | crn_real_tk_dry | as 76 for mnemonic-toolkit; if CHANGELOG's `## [Unreleased]` body is empty, add `- test.` under it and commit; `--dry-run mnemonic-toolkit-v<N>` | 0; shows `== toolkit (b): README markers 2,` (prefix), and exactly one of the two `== toolkit (c): check_cli_pins.py` lines; tree clean |

`CUT_RELEASE_SIBLINGS_REAL` defaults to `/scratch/code/shibboleth` and is
used only for `--reference-if-able`. Fake gh run ids are `1000 + position`
in that tag's workflow list, so ids repeat across tags. That does no harm,
because `run view` answers every id with `FAKE_GH_CONCLUSION`. Rows 44-47
run on the F-tk state that row 42 leaves, in table order. Row 28 compares
with `git diff --no-index --numstat`; k = max(added, removed), e = the
number of crates passed.

## 10. Build

1. Add `scripts/cut-release.sh` (mode 755).
2. Add `scripts/test/cut-release.test.sh` and
   `scripts/test/fixtures/cut-release/` (fake-pvs, fake gh, the two stubs).
3. Delete `scripts/verify-releases.sh` (nothing references it outside
   `design/`).
4. No change to any workflow, to `push-via-staging.sh`, or to any other
   repo. Engrave's CI does not run the new test: that would mean changing
   `release.yml`, which is out of scope. The test is a local gate.
5. Gate: `bash scripts/test/cut-release.test.sh` →
   `cut-release.test: 85 passed, 0 failed` (with gh logged in and
   `$HOME/.cargo/bin` first on PATH), plus engrave's existing CI green on
   the PR. shellcheck is not on the build machine, so it is not a gate;
   row 50 runs `bash -n`.
