# F-679 toolkit side: pin mnemonic-gui v0.62.0 and ms 0.19.1

- **Implementer:** Opus 5.5
- **Date:** 2026-09-25
- **Worktree:** `/scratch/code/shibboleth/tk-worktrees/f679-681`, branch `f679-681`, on top of `65ba3c22` (F-681)
- **Not pushed, merged or tagged.**

## Result

Two commits:

| Commit | What |
|---|---|
| `c39ea361` | Installer: pins `mnemonic-gui-v0.62.0` and `ms-cli-v0.19.1`; the GUI's x86_64 glibc floor drops 2.39 → 2.18; mirrors, golden and CHANGELOG follow. |
| `8ed3494f` | GUI manual pinned to v0.62.0: corpora re-synced, J4 step 14 rewritten, 39 new schema anchors documented, argv-secret prose, install pages pinned. |

I split them because the installer change is what un-breaks the GUI's
verify flow and stands on its own. The GUI manual bump was far larger than
the brief assumed (see "Scope" below).

## 1. Pins (`c39ea361`)

Changes:
- **`install.sh`:** `ms-cli-v0.19.1` and `mnemonic-gui-v0.62.0`.
- **ms mirrors** that sibling-pin-check reads: `manual.yml`,
  `quickstart.yml`, `technical-manual.yml`. sibling-pin-check: rc=0, 11
  OK, 0 warnings.
- **Examples golden** regenerated in the same commit. The diff is exactly
  the ms and GUI `--list` and `--dry-run` lines.
- **The manual's `--from-source --dry-run` recipe** is a command, not a
  capture; it prints the pins at run time, so it needed no edit. No other
  `ms-cli-v0.19.0` or GUI-tag mirror exists in `docs/manual` or
  `docs/quickstart`.
- **Deliberately unchanged:** `install-verify.test.sh`'s fixture
  `.crates.toml` rows still mention `ms-cli 0.19.0`. They are realistic
  cargo-record data, not pins.

## 2. GUI assets and glibc floor

Measured on the published v0.62.0 release:
- **Assets:** x86_64/aarch64 linux and linux-musl, x86_64/aarch64 macos,
  x86_64 windows.zip, and a single `SHA256SUMS`. The installer's existing
  GUI mapping already matches these names.
- **Checksums:** `sha256sum -c` on the two Linux glibc assets: OK.
- **Floors:** `readelf -V` (and `objdump -T`) give max **GLIBC_2.18** for
  **both** x86_64 and aarch64 glibc builds.
- **Installer floor table:** GUI x86_64 2.39 → **2.18**; aarch64 is
  already 2.18. `--help`, the manual's floor table (now naming Ubuntu
  20.04 and RHEL 8 as the hosts where md, not the GUI, builds from
  source) and the README follow.
- **install-verify floor cases** moved: GUI 2.17 → source, 2.18 → binary.
  The note and getconf cases now use md at glibc 2.33, since the GUI
  no longer falls back at 2.35.
- **install-assets:** 32/32 mappings, and 17/17 Linux floors re-measured,
  with both GUI floors at 2.18.

## 3. GUI manual (`8ed3494f`)

**Pin.** `pinned-upstream.toml` → `mnemonic-gui-v0.62.0`. It is **not
pin-neutral**: the tag's own `pinned-upstream.toml` pins toolkit v0.104.0,
md v0.20.3, ms v0.19.1 and mk v0.13.0 (verified on the clone at
`9f569e1`). So the four `*-tag-implied` fields and the `manual-gui.yml`
`verify-examples` CLI tier move too. That job had also lagged at toolkit
v0.74.0 against the file's v0.75.0.

**Corpora**, copied from the tag and measured against v0.57.0:

| Corpus | Changed | How |
|---|---|---|
| Gallery PNGs | 19 of 61 | copied from the tag |
| `.gui` renders | 23 of 61 | regenerated with the tag's `gui-render` (`cargo install --tag mnemonic-gui-v0.62.0 --no-default-features --bin gui-render`) |
| Tutorial figures | 48 of 50 | copied from the tag |
| Tutorial transcripts | 17 of 98 | copied from the tag |

Step 14's two figures and three transcripts are renamed from
`tut-j4-14-depth2-refusal` to `tut-j4-14-depth2-export`.

**`tutorial/50-j4-taproot-twin.md` rewritten.** Step 14 now shows the
four-leaf depth-2 tree `{{T1,T2},{T3,T4}}` under `Kint` exporting: exit 0,
`…#trqmzhua`, and a stderr watch-only note. It explains why: toolkit
0.104.0's `rust-miniscript` pin carries PR-#953, per the GUI's own
`tests/tutorial/manifest.rs` comment and CHANGELOG. The intro no longer
counts it as a refusal, and step 15 introduces the depth-1 twin the rest
of the journey uses.

**Other tutorial prose that had gone false.** Every checksum the tutorial
cites was checked against the new transcripts; all 6 are present.
- **J1:** cards now print unbroken (`--group-size` defaults to `0`).
- **J3:** the restore note described a "depth-0 `xpub661My…`"
  re-serialisation. Decoding both xpubs shows the key and chain code are
  identical; the restore now sets depth 3 and the child number from the
  origin, and leaves the parent fingerprint at `00000000`. I also removed
  the "unlike the depth-2 tree" sentence.
- **J5:** the cited checksum was `#yjp7hj7w`; the transcript now shows
  `#284ufd99`.

**39 new schema anchors**, plus the outline fixes. Each section is written
from the pinned binaries' own `--help` text and the GUI CHANGELOG's
conditionals:
- **md:** `--in` on inspect/decode/verify/bytecode/repair (with new
  outlines); `encode --in/--out/--experimental`;
  `verify --path/--experimental`;
  `address --path/--from-mk1/--from-mk1-file/--seat/--experimental`.
- **mk:** `encode --chunk-set-id`.
- **mnemonic:** `export-wallet --allow` (5 variant sections) and `--count`;
  `bitcoin-core-addresses` on export-wallet and restore (and "eight" /
  "eleven values" corrected to twelve); `restore --recalibrate-threads`;
  `(none)` on `xpub-search address-of-xpub --address-type/--network`.
- **ms:** `--in` on all 8 material verbs, `--out` on encode/repair/split,
  and a new ms inspect outline.

**Separators.** The 14 orphan `hyphen`/`comma` sections are removed from
mnemonic, md and ms. Measured: `ms 0.19.1` refuses both ("no longer
offered … an already-engraved hyphen- or comma-grouped card still
DECODES"), even though its own `--help` still lists them. mk keeps all
three. The four `mnemonic` group-size defaults now read 0; measured, md,
ms and mk still default to 5. The ms encode example now shows the grouped
form on the stderr engraving card (`lines="1-4"`).

**Secrets on argv.** A new §14 subsection explains that the CLIs refuse a
secret on argv unless `--allow-argv-secret` is given, and that the GUI
adds it on Run, shows it in the modal, and never puts it in Copy command.
Evidence: the GUI's `form/invocation.rs::admit_argv_secret_for_run` and
the J1 modal screenshot, which shows `--allow-argv-secret` in the argv and
not in the Preview. Related changes:
- The `ms` `.cmd` transcripts now use that Run argv.
- Three `ms` steps (inspect, decode, verify-bare) said "no run-confirm
  modal". That was already wrong before this bump:
  `secrets::should_confirm_run` fires on a non-empty secret positional,
  and has since v0.34.0.
- I measured that with the flag the CLI still prints its
  `warning: secret material on argv …`, so the existing refusal-table rows
  quoting that warning stay true.

**Install pages.** The three tagless
`cargo install --git …/mnemonic-gui` lines are pinned to
`--tag mnemonic-gui-v0.62.0`. The stated GUI MSRV is corrected from 1.85
to 1.88, as v0.62.0's `Cargo.toml` declares. Path C now points at
`install.sh --only mnemonic-gui`, and "verify against the release notes"
now says `SHA256SUMS`.

**CHANGELOG:** entries in both the toolkit and GUI-manual CHANGELOGs.

## 4. Gates, as run

**Installer harnesses and real install:**
- **Harnesses under bash and dash:** install-verify 47/47, msrv-guard 6/6,
  man-step 4/4 (39 pages, 0 help shadows), install-assets 32/32 + 17/17.
  shellcheck 0.11.0 is clean.
- **Real install** into a scratch `--root` with `--man-dir`: rc=0,
  "5 installed", 79 man pages. Versions: mnemonic 0.104.0, md 0.20.3,
  ms 0.19.1 (`92a45a92…`, `SHA256SUMS.x86_64`), mk 0.13.0, and
  **mnemonic-gui 0.62.0** (`2f10ee0b…`, `SHA256SUMS`).
- **`ms verify` through the installed ms**, using "abandon ×11 about"
  (encoded to `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f`):

  | Case | Result |
  |---|---|
  | ms 0.19.1, `ms verify --allow-argv-secret --phrase "<P>" <ms1>` | `OK: round-trip valid (12 words, language=english)`, rc=0 |
  | Control: ms 0.19.0 asset, same command | `error: cannot read both ms1 and --phrase from stdin`, rc=1 |
  | ms 0.19.1, wrong phrase | `error: phrase mismatch`, rc=4 |

**CLI manual and toolkit** (`*_BIN` pointed at the scratch root's pinned
binaries, mnemonic at `target/debug`):
- `make audit` rc=0; doc-flag-lint rc=0; doc-gate-guard 88/0.
- Quickstart `make lint` and `make verify-examples` rc=0.
- Manual pdf/html rc=0.
- Examples golden: regenerated; unchanged on the second commit.
- sibling-pin-check 11 OK, 0 warnings; install-pin-check
  `mnemonic-toolkit-v0.104.0`.
- fmt (1.95.0) rc=0; clippy `-D warnings` (1.85.0) rc=0; nextest 4057/4057
  (run on both commits).

**GUI manual**, run against the v0.62.0 clone and the new CLI tier:
- `make html` rc=0.
- `make lint` 12/12: schema coverage 1009 anchors; outlines 128; figures
  61/61; tutorial figures 50/50; transcripts 98/98; tutorial-xref,
  markdownlint, cspell and lychee all OK.
- `make verify-examples` 17/17; `make verify-examples-gui` 61/61 with no
  secret leak.
- `make pdf`, `gui-example-pdf` and `gui-example-html` rc=0; CI's embed
  censuses 61/61 and 50/50.

## 5. CHANGELOG at v0.104.0 (note only; not fixed)

**Yes, it still leaves shipped changes under `[Unreleased]`.** The released
`mnemonic 0.104.0` binary already has both of these, and both still sit
under `## mnemonic-toolkit [Unreleased]`:
- `bundle --group-size` `[default: 0]` (the P3 CLI-uniformity
  "--group-size now defaults to 0" item);
- `export-wallet --format bitcoin-core-addresses` (the "Phase 1b" item).

My two F-679 entries go under `[Unreleased]` too, correctly, since they
are not released.

## Scope, and what is not done

- **The GUI manual bump was a documentation refresh, not a pin change.**
  The brief expected a pin bump plus one chapter. Against v0.62.0 the lint
  failed 6 of its 12 phases, and every ms worked example was refused by
  the new argv rule. I did the whole thing rather than narrow the task. It
  was machine-gated wherever the gates reach.
- **Not audited: GUI-manual prose outside the gates.** The gates prove
  anchors, outlines, corpora and transcripts. They do not prove every
  sentence in about 90 pages still describes toolkit 0.104.0's behaviour.
  I fixed every stale statement my targeted greps found:
  - grouping and separator defaults
  - argv-secret wording and "no modal" claims
  - xpub headers and checksums
  - MSRV and install paths

  A sweep of the other chapters against the 0.75 → 0.104 CLI delta
  (refusal tables, exit codes) has **not** been done. A reviewer lens is
  worth spending there.
- **GUI install-page pins are ungated.** sibling-pin-check scans
  `docs/manual` and `docs/quickstart`, not `docs/manual-gui`. The manual-gui
  `verify-examples` tier installs are multi-line, so it cannot see them
  either. I didn't add the GUI manual to the scan: its GUI tag can
  legitimately differ from the installer's.
- **ms-side finding (not ours to fix):** `ms 0.19.1 encode|split --help`
  still says `space|hyphen|comma`, but the binary refuses `hyphen` and
  `comma`.
- **The musl GUI is still not mapped.** The installer builds it from source
  on musl (M4, F-676 fold 2). v0.62.0 still ships static musl assets, and I
  did not re-test whether they can open a display.

## Housekeeping

- Scratch `/scratch/code/shibboleth/f679-tk-scratch/` (846 MB: the GUI
  clones, the `gui-render` build, the scratch install root, logs) was
  deleted with `find -delete`. No `/tmp` leftovers.
- The worktree is clean.
