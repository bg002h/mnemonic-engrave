# F-674 items 3 and 4: descriptor-mnemonic implementation report

Status: DONE. Both items are fixed and released on branch `f674-dm` as md-codec 0.48.2 and md-cli 0.20.2. Nothing is pushed, tagged or merged.

## HEADs

- descriptor-mnemonic `f674-dm` (base `6fc93083`):
  - `80feff1e`: item 3 (dates)
  - `1af993cf`: item 4 (network names)
  - `b662be1c`: release
- mnemonic-engrave `f674-records` (base `57123659`):
  - `b729bdb1`: `design/evidence/measured-at.json`
  - `89d34986`: F-674 status note on items 3 and 4
  - The parallel agent's `86d63288` and `f243bce9` sit between these two. I did not touch them.

## Item 3: measurement dates

**Root cause.** `scripts/vendor-coord-evidence.sh` dated each row by `git blame` committer time, rendered with `date -u`. That rule misdated three sources, not one:

| source | table said | actually measured (local, -0700) | where the date comes from |
|---|---|---|---|
| Core e2e `core-composer-tr.out` (16 rows) | 09-24 | 2026-09-23 | e2e-live-site-wallets.md:3 "Date 2026-09-23"; core-mainnet.txt mtime 19:29 |
| Liana 8.0 `fable-liana-parse-out.jsonl` (56 rows) | 09-20 | **2026-09-19** | `.tmp/fable-liana-parse-out.jsonl` mtime 23:22:54; report persisted 00:00 on 09-20 |
| Nunchuk `fable-nunchuk-harness-out.txt` (30 rows) | 09-20 | **2026-09-19** | harness out mtime 21:27:48; report persisted 21:35 |

The other sources were already right:
- Core boundary: coord-compat-core-boundary.md:5, "Measured 2026-09-23".
- Liana v15.0 re-measure: 09-20, file mtime 09:32.
- The F-640 v15 record: 09-23, after the plan went GREEN at 02:02.
- f449 stage 2 and stage 4, and both 1b probes: 09-23.

**Fix.** Engrave gets `design/evidence/measured-at.json`, with one entry per vendored outcome file (18 of them). Each entry holds `measured_at` and a `basis` saying where the date comes from. There is also a per-record override keyed by name, used once, for F-640's row.

It is a sidecar rather than a field inside each file. The files are raw harness output, and their own gates byte-compare them: `liana-live-gate.sh` diffs the expected file, and the stage 4 probes re-run byte-identical.

The vendor script now reads dates from the manifest and exits 1 in two cases:
- an input with no date;
- a manifest date that dates no row (a stale or mistyped entry).

The manifest is hashed into `evidence.meta.json`, so `--check` notices a date change. I vendored from a detached checkout at `b729bdb1`, so the meta names my commit and not the parallel agent's later HEAD. `--check` against the worktree's current HEAD `89d34986` reports fresh.

**What else changed.** With `measured_at` masked, `evidence.jsonl` and `table.rs` are identical to before. 54 cells change date: 16 go from 09-24 to 09-23, and 38 from 09-20 to 09-19.

**Tests** (both red first):
- `measured_at_is_the_date_each_source_states` (md-codec)
- `descriptor_dates_the_core_import_the_day_it_was_measured` (md-cli)

**Mutations:**
- Vendoring with the old blame script turns both tests red.
- Dropping the per-record lookup trips the script's "dates no row" guard. With the guard also disabled, the md-codec test goes red on `v15.jsonl:290`.
- Deleting an entry makes the script exit 1, naming the file.
- Everything was restored, compared byte-identical and touched.

## Item 4: network names

There were two sites. A sweep of every message that formats a network or `NetworkKind` found no third.
1. `decompose::check_network` printed `NetworkKind`, which is "testnet" for signet and regtest too. It now prints the network passed, via a new `parse::keys::network_name`.
2. `parse_key`'s version error said "expected testnet xpub version" under signet and regtest. It now names the network passed.

In the other direction, a tpub under mainnet was told to re-run with `--network testnet`. The key's version bytes can't tell testnet, signet and regtest apart, so the recipe now lists all three. This is the same class of defect; it changes a mainnet error message.

**Tests** (3, all red first):
- `network_mismatch_names_the_network_the_user_passed`
- `a_tpub_under_mainnet_offers_every_test_network`
- `rejects_xpub_naming_the_network_passed`

**Mutations.** There were three, each applied and each red:
- decompose names `want`'s kind again: 1 red;
- signet and regtest labelled "testnet": 2 red;
- the single-testnet recipe: 1 red.

## Release

md-codec 0.48.1 → 0.48.2 and md-cli 0.20.1 → 0.20.2. The exact pin `=0.48.2`, Cargo.lock (these two versions only) and both CHANGELOG entries are in one commit, `b662be1c`. `fuzz/Cargo.lock` still pins md-codec 0.47.0; it was already stale, and past releases did not touch it.

## Gate

Toolchain 1.85.0, `CARGO_TARGET_DIR=dm-worktrees/f674-target`, run on the tree of each commit:

| commit | nextest (`--workspace --all-features`) | doc, clippy, fmt |
|---|---|---|
| `80feff1e` | 1566 passed, 4 skipped | rc 0 |
| `1af993cf` | 1569 passed, 4 skipped | rc 0 |
| `b662be1c` | 1569 passed, 4 skipped | rc 0 |

- doc: `-D warnings`
- clippy: `--all-targets -D warnings`
- fmt: `--check`

## Release-build corpus diff

Release builds of `6fc93083` (md 0.20.1) and `b662be1c` (md 0.20.2) were run over the f672 corpus: 68 phrase vectors × 12 mainnet commands, 816 outputs. Each decompose was also run under signet and regtest, another 136 outputs. The script is `.tmp/f674rev/corpus.sh` and its output is `.tmp/f674rev/corpus.out`.

- **Mainnet:** 28 of 816 files differ. Every differing line is either a date (Core 29.4/31.1 moves 09-24 → 09-23; Nunchuk moves 09-20 → 09-19) or the Liana split described below.
- **Signet/regtest decompose:** 80 of 136 differ. Every one changes "--network says testnet" to "says signet" or "says regtest".
- **By hand:** `md encode --network regtest|signet` with an xpub now says "expected regtest|signet xpub version". Testnet is unchanged.

## Concerns

1. **Visible change beyond a date: the Liana verdict splits into two lines.** A verdict run merges consecutive versions only when their `At` is equal, and `At` includes `measured_at`. Liana 8.0 and 15.0 were both dated 09-20 by accident, so they merged into one line, "Liana 8.0-15.0: imports … (measured 2026-09-20)", which misdated 8.0. Now the output shows "Liana 8.0: … (2026-09-19)" and "Liana 15.0: … (2026-09-20)". That affects 6 mainnet corpus files, plus the X24 ImportsAltered case, whose test I updated to pin both runs.
   - The alternative is to merge across dates and print a date range. That changes `Verdict` (public API), so it is not a patch. I chose the split. Overrule it if you prefer the single line.
2. **Two dates rest on file mtimes, not on a stated date.** The Liana 8.0 and Nunchuk reports state no date, so the manifest cites the harness-output mtimes in `/scratch/code/shibboleth/.tmp`, which are ephemeral, plus the reports' commit times. The commit times alone already bound both runs to 09-19 local.
3. **The recipe change for a tpub under mainnet goes slightly past item 4's wording.** I included it because it is the same "assumes testnet" defect.
4. **Nothing is pushed.** The engrave branch interleaves my commits with the parallel agent's, and `**Status:** OPEN` is unchanged.
