# F-694 part 2 — GUI manual: five new forms + secret channels (agent report)

Date: 2026-09-25. Brief: `design/briefs/f694-manual.md`.
Toolkit worktree `/scratch/code/shibboleth/tk-worktrees/f694`, branch
`f694-manual-gui` off origin master `af5cc1a5`. Written against
mnemonic-gui master `3ca1d604`. Not pushed, merged or tagged. Pin step
NOT done (waits for the GUI release).

## Commits (branch `f694-manual-gui`)

| SHA | What |
|---|---|
| `de4d60c1` | `extract_gui_schema.py`: resolve `&str` consts inside a variant slice (hard error on an unknown name; skips `//` comments). It silently dropped `ms hashlock --kind`'s `HASHLOCK_KIND_ALL`. Inventory at v0.62.0 byte-identical before/after; at master +1 variant. |
| `26941173` | Corpus re-sync to master: 5 `.gui` renders (master's `gui-render`), 5 gallery PNGs (byte copies), tutorial 9 PNGs + 5 stderr transcripts moved, 3 empty stderr transcripts deleted (manifest dropped them), J1/J2/orientation prose rewritten for the private-channel runs. |
| `74296693` | Five chapters: `src/50-md/5C-compose.md`, `5D-shape-key.md`, `5E-descriptor.md`, `5F-decompose.md`, `src/60-ms/6c-hashlock.md`; gallery sections (752/753), counts (750, 51, 61 overviews), index-table rows, flag-index entries, 3 cspell words. |
| `7dc78fa9` | Secret channels section in §14 (`#secret-channels` + 6 subsections); Defense 2 and `#secret-argv-opt-in` rewritten (argv flag = interim path only); §32 tour modal, §84 limitations; per-form sweep (see below); F-687 advisory texts. |
| `fce38204` | CHANGELOG `[Unreleased]` entry. |

## Anchors

Derived from the GUI's own code: a scratch test in a `git archive` of
master called `help::url::manual_url_for_subcommand` /
`manual_url_for_flag` (base `MANUAL_BASE_URL` =
`https://bg002h.github.io/mnemonic-toolkit/manual-gui/`) over the five
schemas, with `form/widget.rs::needs_help_icon`'s predicate copied
verbatim (Dropdown | NodeValueComposite | TaggedOrIndexed | repeating;
plus `--slot` when `allows_slots` — none of the five). Variant anchors
are never produced by a help icon (`manual_url_for_variant` has no
caller).

**GUI-produced (20) — all 20 present in the built HTML:**

| Form | Anchors |
|---|---|
| md compose | `md-compose`, `md-compose-wrapper`, `md-compose-path`, `md-compose-unspendable` |
| md shape-key | `md-shape-key` |
| md descriptor | `md-descriptor`, `-key`, `-fingerprint`, `-from-mk1`, `-seat`, `-network`, `-emit`, `-separator` |
| md decompose | `md-decompose`, `md-decompose-emit`, `md-decompose-network` |
| ms hashlock | `ms-hashlock`, `ms-hashlock-kind`, `ms-hashlock-method`, `ms-hashlock-separator` |

**Lint-required schema anchors for the five forms (sub + every flag +
every variant): 80, 0 missing** (compose 15, shape-key 2, descriptor 25,
decompose 14, hashlock 24; includes the `(none)`/`(choose)` empties
`md-compose-unspendable-`, `md-descriptor-emit-`, `ms-hashlock-kind-`,
and `ms-hashlock-kind-all-kinds-lookup-only`). Whole manual:
gui-schema-coverage 1089 anchors / 66 subcommands; outline-coverage 140.

Not documented (consistent with existing pages): `--allow-argv-secret`
on hashlock (spread-const flag the extractor skips; no widget).

## Behavioural claims — how each was checked

GUI claims: a scratch kittest/planner probe (never committed) against
the master archive, `cargo test --test zz_f694_manual_probe`:

- **Window (kittest, `MnemonicGuiApp::new_headless`)**, `restore
  --passphrase` typed into the real form: Preview line and binding lines
  (`--from phrase= ← env MNEMONIC_GUI_S0 (typed)`, `--passphrase ← stdin
  via --passphrase-stdin + '\r\n' (typed|value of $VAR)`); confirm dialog
  first line "This invocation sends these secrets privately to
  mnemonic:" with argv + Secrets; each refusal's exact "Run refused —
  …" text and Run disabled for: `-`, unset/empty/lower-case/reserved
  `@env:`, variable ending `\n`, variable holding `@env:OTHER`, ` - `,
  `@ENV:X`, `@environment`, `U+200B -`, `U+FF0D`, typed `\r`, NUL;
  `-lead` accepted on Linux. Copy (Windows) disabled on a stdin binding.
- **md descriptor `--key` with xprv**: buttons "Copy command (POSIX) —
  reveals secret", `copy_reveals_secret = true`, Copy text contains the
  xprv, dialog "This invocation passes secret-bearing arguments to md:"
  with `--key ••••`. Persistence/positional claims quoted from
  `persistence.rs:114-122,140-148` and `secrets.rs:197-203`.
- **ms hashlock**: `(choose)` → "Run disabled — choose a --kind first…",
  Copy disabled with same reason; all-kinds note; `--json` note; Copy
  text; channels per source (phrase → `--hashlock-phrase-stdin`, hex →
  `--hex -`, plate → `-- -`); macOS `-lead` → `value-starts-with-dash`.
- **Interim path**: `channels::plan(..., "macos"|"windows")`:
  `--allow-argv-secret`, masked argv, `← argv + --allow-argv-secret
  (interim)`. Dialog prefix choice quoted from `app_window.rs:1180-1184`.
- **Copy** texts (typed EnvRef `IFS= read -rs`, typed stdin comment,
  `$MY_PW` → `printf '%s\r\n' "$MY_PW" |`, multi-line typed → disabled
  with MULTILINE_TOOLTIP) via `copy::copy_commands`.
- **Real runs through the GUI's planner + runner** (`plan_for_run` +
  `runner::run_plan`, installer CLIs): all worked examples (compose ×4,
  shape-key, descriptor ×2, decompose ×2, hashlock ×4); GUI-resolved
  `@env:F694_MY_PW` = typed = `b4e3f5ed`.
- ms positional (inspect/decode/verify/derive → `-- -`, `-` refused),
  import-wallet (`--ms1 @env:MNEMONIC_GUI_S0`), xpub-search ×3,
  silent-payment, electrum-decrypt channels, via `channels::plan`.
- Env hygiene / fd numbering quoted from `runner.rs:244-286`,
  `channels/mod.rs:658`; policy from `channel_policy.json`.

CLI claims (installer's mnemonic 0.105.1, ms 0.20.1, md 0.20.3; run
directly): `-`/`@env:`/`--passphrase-stdin` = `b4e3f5ed`; empty stdin /
empty env warnings (exact text) → `73c5da0a`; unset env error; exactly
one line ending stripped (`TREZOR\n\n` → `48efb44f`); literal argv
refused, and with `--allow-argv-secret` the new advisory text (for
`--passphrase` in convert/bundle/slip39 combine and `--bip38-passphrase`);
terminal prompt "Enter passphrase:" and paste drain with masked preview
under `script(1)` (mnemonic and ms). `--bip38-passphrase` /
`--decrypt-password` `-`/`@env:` taken from the CLIs' `--help` only (not
run). All md/ms refusal rows in the new pages were run.

## Per-form sweep (in `7dc78fa9`)

Statements that were false against master and are corrected: restore
`--from`/`--passphrase`/`--passphrase-stdin`/advisories; addresses
`--from`/`--passphrase`; verify-bundle `--from`; import-wallet §9.3 and
its walkthrough/refusals; ms inspect/decode/verify/derive positional `-`
and "argv shows --allow-argv-secret"; ms verify `--phrase -`; worked
examples in xpub-search ×3, silent-payment, electrum-decrypt that said
to tick a `*-stdin` toggle (rendered `[disabled]` in the renders); md
overview "modal never fires" (xprv exception); inline `--passphrase` /
`--bip38-passphrase` advisory rows → 0.105.1 text.

**Residue (Minor, not done):** many per-form pages still say the secret
"renders as `••••`" in the modal (true on macOS/Windows; on Linux the
argv shows a reference instead). Not wrong-in-effect (nothing is
revealed), so left; §14 Defense 2 now states both. ms overview still says
inspect/decode "do not accept any `secret: true` schema flag" (their
positional is secret) — pre-existing, untouched.

## Gates (all run; `MANUAL_GUI_UPSTREAM_ROOT` = clean `git archive` of
master at `/scratch/.../gui-clean`, `*_BIN` = scratch installer install)

- `make lint EXPECTED_GUI_RENDER_COUNT=66`: 13/13 OK.
- `make html`: exit 0, embed census 66 (`src="data:image/png"`).
- `make pdf`: exit 0, no missing glyphs (a U+FF0D was replaced).
- `make gui-example-html` (50/50 embeds), `make gui-example-pdf`: OK.
- `make verify-examples-gui GUI_RENDER_BIN=<master gui-render>
  EXPECTED_GUI_RENDER_COUNT=66`: 66/66, no leak.
- `docs/manual make audit` (installer bins): OK (62 transcripts; anchor
  baseline unchanged).
- Informational, not a brief gate: manual-gui `make verify-examples`
  with the 0.105.1 bins FAILS on one transcript,
  `transcripts/4e-xpub-search-passphrase-advisory.err` line 2 (old
  `--passphrase` advisory text). The page includes only line 1. Left for
  the pin step, since CI's tier is still 0.104.0.

Note: the brief's count override is on the command line; the Makefile
default and CI still say 61 (they move with the pin). Until the pin
step, CI on this branch will be red against v0.62.0 (figures, renders,
tutorial corpus and anchors all follow master) — by design.

## Pending pin step (after mnemonic-gui v0.63.0 is released)

Verify first what the tag pins (expected toolkit 0.105.1, md 0.20.3, ms
0.20.1, mk 0.13.0). Then:

1. `pinned-upstream.toml`: `[mnemonic-gui] tag` → `mnemonic-gui-v0.63.0`;
   `toolkit-tag-implied` → v0.105.1, `ms-cli-tag-implied` → v0.20.1 (md,
   mk unchanged if verified); delete `[installer-ahead]`; add a
   v0.62.0 → v0.63.0 history comment.
2. `src/80-troubleshooting/82-binary-and-launch.md`: delete the
   `{#installer-ahead}` section (§82 lag note) and update lines 25-36
   (pinned list; "Below `mnemonic` 0.104.0 … `ms` 0.19.1").
3. The other 21 off-pin prose lines `check_cli_pins.py` will flag
   (simulated): §11:13-14, §12:42/44, §31:32/36, §33:37, 42-bundle:56,
   44-convert:55, 61-overview:10/11/57, §82:43/44, 94-release-history:
   12-13 (history — add to `cli-version-history.txt` or reword),
   tutorial 00-frontmatter:20, 10-ch0:21/69, 50-j4:26.
4. `Makefile` `EXPECTED_GUI_RENDER_COUNT` 61 → 66 (+ comment);
   `tests/lint.sh` default `:-61`; `.github/workflows/manual-gui.yml`
   comments/steps at 157-211 (toolkit v0.104.0 → v0.105.1, ms v0.19.1 →
   v0.20.1), 229, and the embed census 351-361 (61 → 66).
5. Re-sync corpora from the v0.63.0 tag (its `Pinned:` labels change,
   so expect most PNGs to move): `figures/gui/`, `transcripts/gui/`
   (tag's `gui-render`), `figures/tutorial/`, `transcripts/tutorial/`;
   re-check any tutorial prose quoting "Pinned: mnemonic 0.104.0".
6. Regenerate `transcripts/4e-xpub-search-passphrase-advisory.err`
   (and any other `verify-examples` drift) at the new tier.
7. §14 says "Since mnemonic-gui v0.63.0" — confirm the release number.
8. CHANGELOG: move `[Unreleased]` to the new `manual-gui-v*` version;
   94-release-history row; tag `manual-gui-v*` (publishes via
   `manual-gui.yml`); then fetch the live site and check the 20
   help-icon anchors above resolve (F-694 close criterion).

## Side effects

- `scripts/install.sh --root <scratch>` also installed man pages to
  `~/.local/share/man/man1` (installer behaviour; same versions as the
  installer's pins).
- Scratch `/scratch/code/shibboleth/f694-manual-scratch/` (GUI archives,
  target dir, CLI install, probe test, logs) deleted with `find … -delete`.
