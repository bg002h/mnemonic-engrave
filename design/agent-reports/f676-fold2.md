# F-676 fold 2: the f675-677 review, folded

- **Implementer:** Opus 5.5
- **Date:** 2026-09-24
- **Worktree:** `/scratch/code/shibboleth/tk-worktrees/f675-677`, branch `f675-f677`
- **Commit:** `4eceab6b` "F-676 fold 2: fold the f675-677 review (I1, M1-M6, nits)". It sits on `d3f043e7`; nothing earlier is rewritten.
- **Not pushed, tagged or merged.**
- **Review folded:** `design/agent-reports/f675-677-toolkit-review.md` (0C/1I/6M/7N).

## Finding → action

| ID | What I did | Pinned by |
|---|---|---|
| **I1** (blocking) | Four changes, detailed below the table. | install-verify 15, 16; install-assets floor check; mutations I1a–I1f |
| **M2** (blocking) | Temp dir created up front, fails closed. Detail below. | install-verify 14 (both cases); mutations M2a, M2b |
| **M3** | Every listed survivor is pinned. Detail below. | see the M3 table |
| **M1** | BusyBox `wget` has no `--https-only`, so `fetch` uses it only when `wget --help` lists it. The URL is still https, and the checksum still gates every file. **Real run:** BusyBox-only PATH (BusyBox `sh`, `wget`, `tar`, `awk`, `sha256sum`, `mktemp`; no curl) installed mnemonic 0.104.0, md 0.20.3, ms 0.19.0 and mk 0.13.0 with rc=0. | not mutation-pinned (see "Not pinned") |
| **M4** | The musl GUI mapping is removed. A musl host builds the GUI from source, as before F-676. Reason documented in `asset_for`, `--help` and the manual. | install-verify 22 (musl row); mutation M4 |
| **M5** | After a real install, warns with bash/zsh/fish snippets if `$BIN_DIR` is not on `PATH`. | install-verify 28 (warns off-PATH, silent on-PATH); mutation M5 |
| **M6** | The missing-cargo error lists why each component needs a source build and ends "re-run with --exclude md,mnemonic-gui" (the actual list). | install-verify 29; mutation M6 |
| **N1** | A directory at `<root>/bin/<bin>` is refused and left untouched. | install-verify 26; mutation N1 |
| **N2** | The partial copy path is held in `PART`, and the EXIT trap removes it. | **not tested**: interrupting during `cp` is not deterministic |
| **N3** | `--root ""` and `--root=` exit 2. | install-verify 27; mutation N3 |
| **N4** | Kept the Windows GUI run-check skip. I can't run the Windows GUI to confirm `--version` output; being PE32+ console-subsystem makes it likely, not measured. The skip is scoped to the GUI only, and install-verify 23 pins that Windows CLI binaries are still run-checked. | install-verify 23; mutation X4 |
| **N5** | An asset listed with more than one distinct digest is refused. | install-verify 25; mutation N5 |
| **N6** | Not fixed. Editing cargo's `.crates.toml` from a shell script to evict a stale crates.io entry is invasive. The source path does now read `.crates.toml` to decide `--force`. Left as a known gap. | none |
| **N7** | Not fixed: `man-pages.yml:132` is pre-existing, outside this branch's diff, and a different cycle's drift class. | none |

### I1 detail

1. **glibc floors.** Measured on the pinned assets with `readelf -V`
   (arch-independent). The installer declares them in `glibc_floor`:

   | Binary | Floor |
   |---|---|
   | GUI x86_64 | 2.39 |
   | GUI aarch64 | 2.18 |
   | md x86_64 | 2.34 |
   | all other Linux assets | none (static) |

   A host whose glibc is below a floor now plans a source build before any
   download, with the note "needs glibc >= X; this host has Y". The host
   version comes from `getconf GNU_LIBC_VERSION`, or from the
   `MNEMONIC_INSTALL_GLIBC` override (the golden pins 2.39). The
   pre-install run check still backs this up when the glibc is unknown.
2. **Working recipe.** The refusal now prints
   `re-run with: --from-source --only <name>`. The source path passes
   `--force` when `<root>/bin/<bin>` exists and `<root>/.crates.toml` has no
   entry for the package. That is exactly the file cargo refused in the
   reviewer's repro. When cargo does track it, no `--force` is passed and
   cargo reports "already installed".
3. **Docs.** The floors are in `--help`, a manual table (naming Ubuntu 22.04,
   Debian 12 and Ubuntu 20.04) and the README.
4. **A bug found while walking I1.** `--root` containing a space was
   word-split on the source path. Measured: cargo installed into
   `…/src/bin` and failed on a crate named `root`. It now goes through
   `cargo_root` as one argument. Pinned by install-verify 30.

**Real I1 journeys:**
- **Simulated Ubuntu 22.04** (`MNEMONIC_INSTALL_GLIBC=2.35`, default
  all-five run, real cargo): rc=0. The four CLIs installed as binaries, and
  the GUI printed the floor note and was built from source (mnemonic-gui
  v0.59.0 at `#0390ce20`).
- **The reviewer's repro, under dash, into a root with a space:**
  - binary `--only mk,ms` → rc=0
  - `--from-source --only mk,ms` → rc=0, with "replacing … (--force)"
    printed for both; `mk 0.13.0` and `ms 0.19.0`; `.crates.toml` now
    tracks both; no stray `…/src`
  - a second `--from-source --only mk` → rc=0 and "Ignored package …
    already installed", with no `--force`

### M2 detail

- `make_tmp_root` runs once, before the install loop, and only when a
  download is planned. It tries `mktemp -d`, then
  `mktemp -d "${TMPDIR:-/tmp}/mnemonic-install.XXXXXX"`. It accepts only an
  absolute path to an existing directory; otherwise it prints "cannot create
  a temporary directory … Nothing was installed" and exits 1.
- `install_binary` re-checks `TMP_ROOT` (absolute and existing) and checks
  each step itself, including `rm`/`mkdir` via `if ! {…}`, because it runs
  with `set -e` off.
- `cleanup` only deletes an absolute `TMP_ROOT`.

Tests (install-verify 14):
- **Stub `mktemp` that always fails,** with `rm`/`mkdir` wrapped to log
  every call: refused, nothing installed, and **the rm/mkdir log is empty**.
- **The reviewer's trigger:** a `TMPDIR` that does not exist, with real GNU
  mktemp: refused, nothing installed.

Mutation M2b removes both guards. The log then shows
`rm -rf /mk; mkdir -p /mk/x`, which is the reviewer's defect, and the test
goes red. I ran it as a normal user; afterwards `/mk`, `/md` and `/ms` do
not exist.

### M3 detail

| Reviewer ID | Now pinned by |
|---|---|
| X14, X15 (platform mappings) | install-verify 22: the exact table of 8 platforms × 5 components, versions read from the pin table. Separately, install-assets re-measures every Linux asset's glibc floor, or its static-ness, with `readelf` against the real release (17 assets). |
| X5 (trap) | install-verify 21: SIGTERM mid-download → exit 143, next component never fetched |
| X1 (exactly-one) | install-verify 17 |
| X3 (exact version) | install-verify 18 (`mk 0.13.0-rc1` refused) |
| X9 (unpack failure) | install-verify 19 |
| X16 (Darwin detection) | install-verify 20 (arm64 and x86_64) |
| X4 (Windows run check) | install-verify 23 |
| X7 (`curl -f`) | the curl stub now honours `-f` |
| X13 (exact sums name) | install-verify 24 (decoy `old/<asset>` listed first) |
| R1, R1b (cc-validate residue) | cc-validate's `scan` moved verbatim into `residue-lib.sh` as `residue_scan`; cc-validate calls it. `residue.test.sh` drives it with cc-validate's own `DATE_RE`/`TIME_RE`: clean, heavy `/project`, one `/home/`, `__DATE__`, pinned-epoch-only, `__TIME__`-only (a warning) and missing-file cases. |
| VF-H (hermetic CARGO_HOME) | pinned rather than re-commented: `vendor-freshness.test.sh` runs the gate with a caller `CARGO_HOME` whose `config.toml` is malformed. It only passes if the gate's hermetic home is used. The comment now says what pins it. |

**Not pinned, with reasons:**
- **X6 (copy-then-rename becomes a direct copy):** there is no
  deterministic way to observe a torn copy.
- **X12 (`wget --https-only`):** covered by the real BusyBox run, not by a
  mutation. The harness stubs curl, and the wget branch now depends on
  `wget --help`.

## Mutations: 28/28 killed

Driver: `mutate.py` in the fold scratch dir, since deleted. Each mutation
copies `scripts/` or `ci/repro/`, asserts the pattern occurs once and the
text actually changed, runs the named test, and requires a non-zero exit.

- **VF-H** mutates `ci/repro/vendor-freshness.sh` in place, because that
  test copies the gate from the repo root. It restores the file
  byte-identical, and the script asserts this.
- **Killed:** I1a–I1f, M2a, M2b (combined with M2a), X1, X3, X4, X5, X7, X9,
  X13, X14, X15, X16, M4, M5, M6, N1, N3, N5, R1, R1b, R1c, VF-H.
- **Two mistakes in my own driver, both corrected before the counted run:**
  - the M2a pattern's indentation was wrong, so its assertion failed and the
    run stopped; fixed and re-run;
  - test 7's assertion assumed a fixed argument order; the `cargo_root`
    change moved `--root`, so I loosened it to check each flag
    independently.

## Gates, final tree

| Gate | Result |
|---|---|
| install-verify under bash, dash 0.5.13.5, BusyBox 1.37.0 `sh` and yash 2.61 | 36/36 each |
| install-msrv-guard and install-man-step under all four shells | 6/6 and 4/4 (shellcheck 0.11.0 clean, canary 39 pages) |
| install-assets | 32/32 mappings + 17/17 Linux glibc floors |
| Real install (dash-free, default `sh`) | all five at their pins into a spaced `--root`: mnemonic 0.104.0, md 0.20.3, ms 0.19.0, mk 0.13.0, mnemonic-gui 0.59.0; 79 man pages; `md encode --from-policy …` → `md1ypqqnq5fnf2uqqqqpyq8y3frmejdncm8` |
| Tamper refusal | one byte flipped in the GUI download → `REFUSED mnemonic-gui-v0.59.0-x86_64-linux.tar.gz: sha256 mismatch against SHA256SUMS`, "1 installed, 1 failed", rc=1, `bin/` holds only `mk` |
| Platform URLs, under dash with glibc pinned high | every `--dry-run` download URL for 7 platforms: **32/32 return 200 and are listed in a sums file, 0 bad** |
| Examples golden | regenerated; byte-identical to `d3f043e7`'s |
| Manual | `make audit` rc=0 |
| Doc CI checks | doc-flag-lint rc=0; doc-gate-guard self-test rc=0; quickstart `make lint` and `make verify-examples` rc=0 |
| sibling-pin-check | rc=0, 11 OK, 0 warnings |
| install-pin-check | self-pin `mnemonic-toolkit-v0.104.0` |
| Manual document builds | pdf/html rc=0 |
| ci/repro | `residue.test.sh` 21/21; `vendor-freshness.test.sh` 8/8 |
| Rust | fmt (1.95.0) rc=0; clippy `-D warnings` (1.85.0) rc=0; nextest 4057 passed, 20 skipped |

Platform detail: linux-x86_64-musl makes 3 downloads plus 2 source builds
(md, GUI), and linux-aarch64-musl makes 4 plus 1 (GUI). The count fell from
34 to 32 because the two musl GUI assets are no longer mapped.

**Two failures in my own harness, not the code.** The first manual audit
and quickstart verify-examples runs failed only because I pointed `*_BIN`
at a path containing a space, and the lint runs `eval $binv`. Re-run
through a space-free symlink, both passed. yash initially failed
install-verify 29, because my stripped no-cargo PATH lacked `[`/`echo`,
which yash resolves externally in non-POSIX mode. I added them to that
test's PATH.

## Concerns

- **Two items remain unmeasured.** N2 (partial-file cleanup on interrupt)
  is untested, and the Windows GUI `--version` behaviour (N4) is unverified.
- **The musl GUI now needs a Rust toolchain.** A musl desktop now needs
  cargo and rustc ≥ 1.88 for the GUI, which is the pre-F-676 behaviour.
  Without cargo, a default run on x86_64 musl stops with the `--exclude`
  hint instead of installing a GUI that cannot open a window.
- **New CI dependency.** `install-assets.test.sh` now downloads the 11
  distinct Linux assets per run (about 100 MB) and needs `readelf`. It is
  present on ubuntu-latest via binutils.
- **Pre-existing, outside this diff:** `ci/doc-gate-guard.test.sh` leaves
  its `mktemp -d` work dir in `/tmp` (no trap). My runs left three; I
  deleted them with `find -delete`.

## Housekeeping

`tk-worktrees/f676-fold2-scratch` (1.6 GB) was deleted with
`find -delete`, as were the three `/tmp/tmp.*` directories my
doc-gate-guard runs left. The worktree's `git status` is clean.
