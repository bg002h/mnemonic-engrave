# F-679 toolkit fold 1: stale CLI pins in the GUI manual (I-1), with a gate; the 11 false claims (M-1, M-2, M-3)

- **Implementer:** Opus 5.5
- **Date:** 2026-09-25
- **Worktree:** `/scratch/code/shibboleth/tk-worktrees/f679-681`, branch `f679-681`
- **Commit:** `473bca17`, on top of `8ed3494f`. 32 files; two new: `docs/manual-gui/tests/check_cli_pins.py` and `tests/cli-version-history.txt`.
- **Not pushed, merged or tagged.**
- **Review folded:** `design/agent-reports/f679-toolkit-review.md` (0C/1I/3M).

All corrected claims were run against a fresh install of the pinned
binaries into a scratch root: mnemonic 0.104.0, md 0.20.3, ms 0.19.1,
mk 0.13.0 and mnemonic-gui 0.62.0.

## I-1: every stale pin sentence fixed

| Place | Before | Now |
|---|---|---|
| `10-foundations/12-relation-to-cli.md` "Version pinning" (gains anchor `#version-pinning`) | table: toolkit v0.13.0, md v0.5.0, ms v0.2.1, mk v0.3.1, plus a crates.io column and the remedy "Re-install the CLI at the pinned tag" | New table (details below) |
| `10-foundations/11-what-is-mnemonic-gui.md` | the same four old tags | the four pins, linking to the minimums |
| `80-troubleshooting/82-binary-and-launch.md` | the old tags; "Drift is allowed" | the pins; the two measured failure classes below the minimums; "re-run the installer" |
| `30-tour/31-first-launch.md` | `Pinned: mnemonic 0.13.0`, said to be "the runtime `--version` … the GUI reads from each CLI binary at launch" | `Pinned: mnemonic 0.104.0`, the release the GUI was **built against**; the GUI does not read the installed version, so compare it yourself. **The old claim was also false.** |
| `30-tour/33-help-icons-and-deep-links.md` | `Pinned: mnemonic 0.13.0` | `Pinned: mnemonic 0.104.0` |
| `50-md/51-overview.md`, `60-ms/61-overview.md`, `70-mk/71-overview.md` | md-cli v0.11.0 / ms-cli v0.13.0 / mk-cli v0.11.0, and their banners | v0.20.3 / v0.19.1 / v0.13.0, and matching banners |
| `40-mnemonic/4i-repair.md` | "pinned to `toolkit v0.75.0` … exit 5" (twice) | the pinned 0.104.0 behaviour: `--ms1` substitution and incomplete `--mk1` group both exit 4 with UNVERIFIED |
| `60-ms/6a-repair.md` | "pinned to `ms-cli v0.13.0` … exit 5" (twice) | the pinned 0.19.1 exits 4 |
| `70-mk/79-repair.md` | "pinned to `mk-cli v0.11.0` — PRE-fix" | 0.13.0: incomplete group exits 5 with UNVERIFIED; a complete group that fails reassembly exits 2 |
| `20-install/21-linux.md` | "see chapter 12; install at or above the pinned tags" | the installer installs exactly these tags; otherwise stay at or above them, linked to `#version-pinning` |
| `tutorial/10-ch0-orientation.md` (×2) | `Pinned: mnemonic 0.75.0`, "the version … the GUI spawns" | 0.104.0, "the release the GUI was built against" (the capture harness's exact-tier check is real: `tests/tutorial/mod.rs::version_matches`) |
| `tutorial/00-frontmatter.md` | `mnemonic 0.75.0` | `mnemonic 0.104.0` (**found by the gate**) |
| `70-mk/76-vectors.md` | "40 fixtures at `mk-cli v0.3.1`" | measured: 41 fixtures at mk-cli v0.13.0, `family_token` `mk-codec 0.5` (**found by the gate**) |
| `10-foundations/14-secret-handling.md` | "Since toolkit 0.104.0 and ms 0.19" (my own fold-0 line, imprecise) | "The pinned `mnemonic` and `ms` (see Version pinning)" (**found by the gate**) |
| `90-appendices/94-release-history.md` | top entry "Unreleased — GUI pin v0.53.0, in progress" | a v0.62.0 entry added; the v0.53.0 entry is no longer marked unreleased |

The new chapter-12 table has three columns: CLI, pinned tag, and the oldest
release that works with this GUI.
- **toolkit 0.104.0.** The GUI adds `--allow-argv-secret` to secret-bearing
  runs, and older `mnemonic` rejects it. The review established that the
  flag first ships in v0.104.0.
- **ms 0.19.1.** On 0.19.0, `ms verify <ms1> --phrase` fails. This was
  measured last round.
- **md and mk: the pinned tag.** Older releases lack flags the form
  offers. That is from the schema diff; I did not bisect the exact
  release.

It also states that the GUI does not check the installed versions, gives
the `--version` commands, and names the remedy: re-run `install.sh`, or
`install.sh --only <cli>`. The crates.io column is removed and replaced by
"do not install from crates.io".

## The gate: lint phase 13, `cli-pin-consistency`

`docs/manual-gui/tests/check_cli_pins.py` runs in two places:
- `make lint` (phase 13 of 13; the step labels in `lint.sh` are
  renumbered);
- a new step in `sibling-pin-check.yml`, which runs on **every push**. An
  installer-only pin bump would skip the GUI manual's own lint job, which
  is path-filtered, so this second place is what catches it.

It checks three things:
1. **Pin file matches the installer.** The four CLI tags and the GUI tag
   in `pinned-upstream.toml` must equal `scripts/install.sh`'s pins.
2. **Prose names only the pins.** Every CLI version the GUI manual's
   prose names must equal that CLI's pin. This covers forms like
   `mnemonic-toolkit-v…`, `toolkit v…`, `md-cli v…`, `ms …`, and
   `Pinned: <cli> …`, across `src/**/*.md` and `tutorial/*.md`. The one
   exception is a line listed verbatim in `tests/cli-version-history.txt`.
3. **No stale history entries.** Every history entry must still match a
   line.

The history list holds **30 lines, each read and confirmed to be history**:
- "since toolkit v0.80.0";
- "as of `ms-cli v0.14.0`";
- "(ms-cli v0.7.0+)";
- the word-card and glossary "toolkit v0.74.0";
- `md --force-long-code` "no-op since md-cli v0.12.0";
- the release-history appendix.

On a pin bump, every sentence that stated the old pin fails. The author
must either update it or consciously list it as history.

**Proved red**, each run through the new script:

| Input | Result |
|---|---|
| Tree at `8ed3494f` with only the new script and history file dropped in | rc=1, 36 findings: 35 off-pin mentions covering every site the review listed plus the frontmatter, orientation, vectors count and §14; and 1 stale history entry, a line that exists only in the new text |
| Current tree, `install.sh` ms pin bumped to 0.19.2, manual untouched | rc=1, 8 findings: the toml mismatch plus each sentence stating ms 0.19.1 |
| Current tree, `pinned-upstream.toml` GUI tag set to v0.61.0 | rc=1, 1 finding |
| Current tree, unchanged | rc=0: `pinned-upstream.toml == install.sh (mnemonic 0.104.0, md 0.20.3, ms 0.19.1, mk 0.13.0, mnemonic-gui-v0.62.0); 67 version mention(s) in 99 files, 30 history line(s)` |

## M-1, M-2, M-3: the 11 claims, each run

**M-1: `restore --md1` with a depth-2 tap tree restores.** I bundled the
tag's `tests/tutorial/fixtures/taproot-4leaf.desc` into 25 md1 chunks.
`mnemonic restore --md1 …×25` returns rc=0, a `tr([73c5da0a/84'/0'/4']…)`
descriptor, and first receive address `bc1p6yc7kzttzsafprr6hwsaefuyqxvee4j48zdrqt4kl9ers68mhcestwvn66`,
the same as the review. Both places in `4d-restore.md` now refuse only the
`@-in-both` shape, and the page notes that the CLI's own `--help` still
says depth-≥2 is refused.

**M-2: `bundle --descriptor` with `--account != 0` is shape-dependent.**
The measured rule is more precise than the review's:

| Descriptor | `--account 1` gives |
|---|---|
| `wpkh(@0/<0;1>/*)` + slot | rc=2, `--account != 0 is meaningful only with --template; …` |
| `wsh(sortedmulti(2,@0/…,@1/…))` + slots | rc=2, same message |
| `wsh(pk(@0/<0;1>/*))` (non-canonical) | rc=0; the account is used in the inferred default origin `m/48'/0'/1'/2'` |
| literal key with its own origin, `wpkh([73c5da0a/84'/0'/0']xpub…/<0;1>/*)` | rc=0; its own origin `m/84'/0'/0'` is kept |
| `--descriptor-file` with `wsh(pk(…))` | rc=0 |

The toolkit's GUI-schema conditional (`cmd/gui_schema.rs`) pins `--account`
to 0 whenever `--descriptor` is set. `42-bundle.md` is updated in its
outline bullet, its `--descriptor` section, its `--account` section (a
three-case list) and its refusal row.

**M-3: wording and exit codes.** All runs were on the pinned binaries.

| Page | Now says (measured) |
|---|---|
| `60-ms/63-encode.md` | clap group `<--phrase <PHRASE>\|--hex <HEX>\|--in <FILE>>`, exit 64 |
| `50-md/53-encode.md` (row and prose) | `md: encode: TEMPLATE required (on argv, via --in FILE, or use --from-policy with cli-compiler)`, exit 2 |
| `50-md/59-address.md` | clap group `<PHRASES\|--template <TEMPLATE>\|--from-mk1 <STRING>...>`, exit 2 |
| `42-bundle.md`, `--descriptor` with `--descriptor-file` | clap conflict text, exit 64 |
| `42-bundle.md`, `--passphrase` with `--passphrase-stdin` | `the argument '--passphrase <PASSPHRASE>' cannot be used with '--passphrase-stdin'`, exit 64. The GUI's run carries `--allow-argv-secret`; without it the argv-secret refusal fires first, exit 2. |
| `42-bundle.md`, inline `--passphrase` advisory | "— pipe via --passphrase-stdin" |
| `46-derive-child.md` | the node list now includes `seedqr` |
| `60-ms/68-combine.md` | `error: not enough shares: have <n>, need <K>`, exit 1 |
| `60-ms/67-split.md` | K outside `2..=9`: exit 1 at run time, `invalid threshold <K>; K-of-N shares require k in 2..=9` |

## Gates, as run on `473bca17`'s tree

**GUI manual:**

| Gate | Result |
|---|---|
| `make html` | rc=0 |
| `make lint` (against a v0.62.0 clone) | **13/13**: schema 1009, outlines 128, figures 61/61, tutorial 50 + 98, xref, cli-pin-consistency. The first run failed cspell on my word "behaviour"; fixed to "behavior". |
| `verify-examples` | 17/17 |
| `verify-examples-gui` (the tag's `gui-render`, rebuilt) | 61/61 |
| `pdf`, `gui-example-pdf`, `gui-example-html` | rc=0 |
| CI embed censuses | 61 and 50 |

**CLI manual, quickstart and doc CI:**
- CLI manual `make audit`, pdf and html: rc=0.
- doc-flag-lint rc=0; doc-gate-guard 88/0.
- Quickstart `lint` and `verify-examples`: rc=0.

**Installer:**
- Under bash and dash: install-verify 47/47, msrv-guard 6/6, man-step 4/4,
  install-assets 59 ok.
- shellcheck 0.11.0 on `install.sh`: clean.

**Pins, golden and Rust:**
- sibling-pin-check, both steps extracted from the yml and run: 11 OK,
  0 warnings; cli-pin-consistency OK.
- Examples golden: unchanged.
- fmt (1.95.0) rc=0; clippy `-D warnings` (1.85.0) rc=0; nextest 4057/4057.

## Not done, and notes

- **The CLI manual (`docs/manual`) is outside the new gate's scope**, per
  the brief ("GUI-manual sentence"). Its sibling tags are already gated by
  sibling-pin-check.
- **Toolkit-side follow-ups the fold surfaced; they live in the CLIs, not
  the manual:**
  - `mnemonic restore --help` for `--md1` still says "depth-≥2 taproot
    are refused". Measured: they are not refused.
  - `ms 0.19.1 encode|split --help` still lists `hyphen|comma`, which the
    binary refuses (noted last round).
  - The review's out-of-diff observations stand as it filed them:
    - the installer's printed `man -M …/man1` recipe;
    - `final-word` does not resolve `@env:`;
    - slip39's `@env:` path warns about argv.
- **md/mk minimums:** chapter 12 says "the pinned tag" for md and mk
  because I did not bisect which older release first carries each flag.
  "Older releases lack flags the form offers" comes from the schema diff
  against v0.57.0, not from a bisection.

## Housekeeping

Scratch `/scratch/code/shibboleth/f679-fold1-scratch/` (651 MB: the pinned
install root, a v0.62.0 clone, the `gui-render` build, the old-tree export
for the red proof, and logs) was deleted with `find -delete`. There are no
`/tmp` leftovers, and the worktree is clean.
