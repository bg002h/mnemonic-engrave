# F-694 pin step — manual-gui v1.4.0 against mnemonic-gui v0.63.0 (agent report)

Date: 2026-09-25. Branch `f694-manual-gui` in
`/scratch/code/shibboleth/tk-worktrees/f694`. Not tagged, not merged, not
pushed.

## To publish

- **Tag name:** `manual-gui-v1.4.0`
- **Commit to tag:** `cfa293a3c949ce39b522e57d0357dbaede49cbb3` (branch tip)
- Previous manual tag: `manual-gui-v1.3.1`. The version is a minor bump
  under the manual's convention (new chapters plus a GUI pin); it also
  carries the v0.62.0 pin, which was never tagged.

## Commits added in this step

| SHA | What |
|---|---|
| `27307a86` | Merge of origin/master `9e56fe7a`. The only file that changed was `scripts/push-via-staging.sh`; the rest of 0.105.1 / F-693 / F-695 was already in the base `af5cc1a5`. |
| `feb74d67` | Installer: `scripts/install.sh` GUI pin → `mnemonic-gui-v0.63.0`, Examples golden regenerated (one line: the GUI row of `install.sh --list`), toolkit CHANGELOG `[Unreleased]` entry. No other mirror names the GUI tag: sibling-pin-check scans `cargo install --git … --tag` lines in workflows, docs/manual and docs/quickstart, and none is for mnemonic-gui. The GUI manual's install pages move in the next commit. |
| `cfa293a3` | manual-gui v1.4.0 pin step (details below). |

Caveat: `feb74d67` on its own fails `check_cli_pins.py`, because the
installer says v0.63.0 while the toml still says v0.62.0. The tip is
green; only the tip needs to pass CI.

## What the manual pin changed (`cfa293a3`)

- **`pinned-upstream.toml`:** GUI pin → v0.63.0. Toolkit → v0.105.1 and ms
  → v0.20.1; md v0.20.3 and mk v0.13.0 are unchanged. These were verified
  against the tag's own `pinned-upstream.toml`. The `[installer-ahead]`
  table is deleted and a history comment added.
- **§82:** the pinned list is updated and the *installer is ahead*
  section is deleted. The version-floor paragraph now says the secret
  channels were measured against exactly these binaries.
- **Old version numbers:**
  - 16 lines now name the new pins: §11, §12 (including the table and
    its secrets-on-argv paragraph), §21–23 `cargo install` tags, §31,
    §33, §61, the tutorial frontmatter and orientation.
  - 5 lines are genuine history ("since toolkit 0.104.0", the v0.62.0
    release-history entry) and were added to
    `tests/cli-version-history.txt`.
  - Two references to the installer-ahead section (§12, §21) are
    removed.
- **Render count 61 → 66:**
  - Makefile: `EXPECTED_GUI_RENDER_COUNT` and its comments.
  - `tests/lint.sh`: the default count.
  - `manual-gui.yml`: the embed-census step and comments, and the
    `verify-examples` install tier (toolkit v0.105.1, ms v0.20.1).
- **Corpus re-sync from the tag:**
  - 50 tutorial PNGs changed (they show the new `Pinned:` labels).
  - Already byte-identical, so no change: the 66 form PNGs, the 95
    tutorial transcripts, and the 66 `.gui` renders regenerated with the
    tag's `gui-render`.
- **Transcript:** `transcripts/4e-xpub-search-passphrase-advisory.err`
  regenerated with the 0.105.1 advisory text.
- **§14 "Since mnemonic-gui v0.63.0" confirmed:** the GUI CHANGELOG's
  0.63.0 entry is where the secret channels ship, and the first tag that
  contains `d35033a` is v0.63.0.
- **§14 corrected for v0.63.0.** The tag's `channel_table.json` differs
  from master `3ca1d60`: the 16 passphrase/password inputs now use the
  rule `strip-one-trailing-newline`. Measured on the tag:
  - `$VAR` = `hunter2\n` into `restore --passphrase` is now **accepted**;
    before, it was refused as `value-ends-in-newline`. Two trailing
    newlines are still refused, and so is a seed variable ending in a
    newline (seeds are taken verbatim).
  - Copy of a passphrase supplied as `$VAR` is now
    `--passphrase @env:VAR` with the comment `# --passphrase: read by the
    CLI from $VAR`. The `printf … |` pipe now appears only where the CLI
    has no `@env:` for that input, for example an xpub-search
    `--phrase`.
  - The refusals table and the Copy paragraph in §14 are rewritten to
    match.
- **The `••••` leftover:** 16 mnemonic-tab pages plus §11, §12 and §41
  now say "a private reference on Linux, `••••` on macOS/Windows". Three
  worked-example Preview blocks now show the real Linux preview:
  - inspect and repair: `--ms1 -` / `← stdin via `-``;
  - ms-shares combine: `--share @env:MNEMONIC_GUI_S0 …`.

  The remaining `••••` mentions are the macOS/Windows cases, the md
  `xprv` exception (masked on every OS), masked input fields, and
  release-history entries.
- **CHANGELOG:** `[1.4.0] - 2026-09-25`, plus a new section in the §94
  release history.

## How claims were re-checked

I wrote a scratch kittest/planner/runner probe against a `git archive`
of the v0.63.0 tag and ran it with the installer's binaries:
- all 17 restore-field cases: the exact Preview and binding lines, each
  refusal's text, and whether Run is disabled;
- the text of both dialogs, the tour's convert dialog, and the Copy texts;
- the interim path on macOS and Windows;
- hashlock: `(choose)` blocking Run, the notes, and its channels;
- the md `xprv` "reveals secret" labels;
- the channels for the ms positional, import-wallet, verify-bundle,
  xpub-search ×3, silent-payment, electrum-decrypt, nostr,
  seedqr-encode/-decode, ms-shares split/combine and addresses (every one
  goes privately on Linux);
- the help anchors;
- a real run: a GUI-resolved `@env:` gives the same fingerprint as a
  typed passphrase (`b4e3f5ed`).

## Gates (all run at the tip)

`MANUAL_GUI_UPSTREAM_ROOT` pointed at a clean v0.63.0 archive; the `*_BIN`
variables pointed at a scratch `install.sh` install.
- `make lint`: 13/13 OK. gui-schema-coverage 1089 anchors / 66
  subcommands; outline-coverage 140; figures 66/66; tutorial figures and
  transcripts 50 + 95; cli-pin-consistency OK with no installer-ahead
  entries.
- `make html`: 66 embeds. `make pdf`: 0 missing glyphs.
  `gui-example-html`: 50 embeds. `gui-example-pdf`: OK.
- `verify-examples-gui`: 66/66. `verify-examples` (0.105.1 tier): 17/17.
- `docs/manual` `make audit`: OK (62 transcripts; anchor baseline
  unchanged).
- The sibling-pin-check workflow steps, run locally: OK.
- Installer (in `feb74d67`): install-verify, install-msrv-guard and
  install-man-step OK under bash and dash; install-assets 32/32 mappings
  and 17/17 glibc floors; shellcheck clean.
- Examples golden: `gen.sh` with a source-built mnemonic 0.105.1; the diff
  is exactly the GUI row.
- **Real install:** `sh scripts/install.sh --root <scratch> --no-man`
  with the GUI. It installed from the release asset
  `mnemonic-gui-v0.63.0-x86_64-linux.tar.gz` and reports mnemonic 0.105.1,
  md 0.20.3, ms 0.20.1, mk 0.13.0, **mnemonic-gui 0.63.0**.

## The 20 help-icon anchors to verify on the live site

These are produced by the v0.63.0 GUI's `help::url` over the five new
forms, with `widget.rs::needs_help_icon`'s rule. All are under
`https://bg002h.github.io/mnemonic-toolkit/manual-gui/`:

```
#md-compose
#md-compose-wrapper
#md-compose-path
#md-compose-unspendable
#md-shape-key
#md-descriptor
#md-descriptor-key
#md-descriptor-fingerprint
#md-descriptor-from-mk1
#md-descriptor-seat
#md-descriptor-network
#md-descriptor-emit
#md-descriptor-separator
#md-decompose
#md-decompose-emit
#md-decompose-network
#ms-hashlock
#ms-hashlock-kind
#ms-hashlock-method
#ms-hashlock-separator
```

Check: fetch the published `index.html` and grep `id="<anchor>"` for each.
In the local build all 20 are present, along with all 80 lint-required
anchors for these forms.

## Side effects

The scratch directory `/scratch/code/shibboleth/f694-pin-scratch/` was
deleted with `find … -delete`. This covered the GUI archives, the target
dirs, the CLI/GUI install and the probe. The installer was run with
`--no-man`, so it wrote nothing outside scratch.
