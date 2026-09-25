# F-679 toolkit review — installer pins + GUI manual → v0.62.0 + F-681

- **Reviewer:** Opus 5.5 (independent of the implementer)
- **Date:** 2026-09-25
- **Target:** worktree `/scratch/code/shibboleth/tk-worktrees/f679-681`, branch `f679-681`, `origin/master..8ed3494f` = `65ba3c22` (F-681), `c39ea361` (installer pins), `8ed3494f` (GUI manual)
- **Question:** can this ship? Does the installer install the right versions, and does what the GUI manual says match what the pinned binaries do?
- **Method:** a real install from this branch into a scratch root. Every claim below was **run** against the installed pinned binaries. Gates were run in a scratch clone at `8ed3494f`, not in the worktree. Scratch (2.3 GB) was deleted with `find -delete`. The worktree is untouched (clean at `8ed3494f`).

## Verdict

**0 Critical / 1 Important / 3 Minor.**

The installer, the tutorial rewrite and every gate are correct. The one Important is a pin-bump propagation miss, and the fold is mechanical: about 15 version sentences that still name old CLI tags, one of which gives a remedy that would break the GUI.

## 1. Measured false-claim rate (item 1)

I sampled **73 behavioural claims** (exit codes, refusal messages, flag effects, output shapes) and ran each one. They came from `restore`, `convert`, `bundle`, `export-wallet`, `derive-child`, `slip39 split`, `final-word`, `repair`, `ms {encode,split,combine,verify,decode,inspect,repair,derive}`, `md {encode,address}`, `mk {address,repair}`, §14 secret handling, and the J4 tutorial steps.

| Class | False / sampled |
|---|---|
| Any mismatch, including byte-exact message wording | **11 / 73 (15%)** |
| Behaviour is wrong: the outcome, exit code or mechanism differs | **3 / 73 (4%)**: M-1, M-2, and `ms split` K-range (exit 1 at runtime, not a value-parser refusal) |
| Tells the user a **dangerous** thing is refused when it isn't | **0** (M-1 and M-2 claim refusals of benign inputs) |
| "This manual is pinned to X" sentences and banner versions | **15 / 15 false** (see I-1) |

**True, and worth noting, because the pinned build is what the reader gets:**
- restore exits 1, 2, 4 and 0-with-"(overridden)".
- The convert `xpub→mk1`, one-way and advisory strings are byte-exact.
- The `ms`/`mnemonic` repair `--ms1` path exits 4 with the byte-exact UNVERIFIED advisory.
- `mk repair` on an incomplete group exits 5 with its advisory; `mnemonic repair --mk1` exits 4.
- The ms hex error strings, the `K<=N<=31` refusal, and the `--in`-alongside-positional refusal.
- `--in` needs no `--allow-argv-secret`.
- The ms verify exit 4 and stdin messages.
- Hyphen/comma are refused by md and ms. The default grouping is 5 for md/ms and 0 for mnemonic.
- The export-wallet refusals, including the range and timestamp value-parsers and specter missing-info.
- The derive-child rsa, length, wif and dice refusals, and the slip39 1-of-1, xpub and word-count refusals.
- The final-word count, unknown-word and entropy refusals.
- The mk address exits 64 and 2.

## Important

### I-1 — The pin bump left the manual naming old CLI tags. The install page this diff rewrote points readers to them, and one remedy breaks the GUI.

`8ed3494f` moves `pinned-upstream.toml` to toolkit v0.104.0 / md v0.20.3 / ms v0.19.1 / mk v0.13.0. The prose still names older tags.

**Made false by this bump** (each was true at the v0.57.0 pins):
- `src/40-mnemonic/4i-repair.md:117-119,149`: "**This manual is pinned to `toolkit v0.75.0`** … a build at the manual's own pinned tag reports exit `5`". The pinned 0.104.0 exits **4** (run: `mnemonic repair --allow-argv-secret --ms1 ms10entrsqqqqqqqqqqqzqqqqqqqqqqqqqqqqcj9sxraq34v7f` → rc=4 plus UNVERIFIED). The same holds for the incomplete `--mk1` group (rc=4).
- `src/60-ms/6a-repair.md:97-99,124-125`: "pinned to `ms-cli v0.13.0` … exits `5`". ms 0.19.1 exits **4**.
- `src/70-mk/79-repair.md:104-109`: "pinned to `mk-cli v0.11.0` — PRE-fix … a complete-group reassembly failure (blessed as a confident fix …)". mk 0.13.0 has the fix.
- `src/50-md/51-overview.md:9-11`, `src/60-ms/61-overview.md:9-11,53`, `src/70-mk/71-overview.md:10-12,56`: "pinned upstream version is `md-cli v0.11.0` / `ms-cli v0.13.0` / `mk-cli v0.11.0`", plus the banner format `Pinned: ms 0.13.0`. The built HTML carries `Pinned: ms 0.13.0` twice.

**False before this bump, and now load-bearing:**
- `src/10-foundations/12-relation-to-cli.md:37-53`: the table says toolkit `v0.13.0`, md `v0.5.0`, ms `v0.2.1`, mk `v0.3.1`, then "**Re-install the CLI at the pinned tag** to resolve".
- The same list appears at `src/80-troubleshooting/82-binary-and-launch.md:27-30`, `src/10-foundations/11-what-is-mnemonic-gui.md:12`, `src/30-tour/31-first-launch.md:32-35` and `src/30-tour/33-help-icons-and-deep-links.md:37`.

**Why this is Important, not Minor.** The install pages this diff rewrote (`20-install/21-linux.md:36-37`, and the matching lines on the macOS and Windows pages) say "The GUI pins specific tags of these (see chapter 12); install at or above the pinned tags."
- Chapter 12's "at or above ms v0.2.1" admits **ms 0.19.0**. The implementer's own control measured that 0.19.0 breaks the GUI's `ms verify --phrase` (`cannot read both ms1 and --phrase from stdin`, rc=1), which is the defect this cycle exists to fix.
- Chapter 12's remedy, "Re-install the CLI at the pinned tag", sends a user who sees a mismatched banner to **toolkit v0.13.0**. `--allow-argv-secret` first appears at `mnemonic-toolkit-v0.104.0` (`git tag --contains` on the first commit adding it). The GUI adds that flag on every secret-bearing Run (§14, `secret-argv-opt-in`). So after following the remedy, every secret-bearing GUI run fails with clap's unknown-argument error. This is **inferred**: I did not install 0.13.0.

Following the manual's own advice ends somewhere worse than telling the user nothing.

**Reproduce:**
```sh
grep -rn 'pinned to\|Pinned: m[sdk] 0\.\|v0\.13\.0\|v0\.2\.1\|v0\.3\.1\|v0\.5\.0' docs/manual-gui/src/{10-foundations,30-tour,40-mnemonic/4i-repair.md,50-md/51-overview.md,60-ms,70-mk,80-troubleshooting}
```

**Fold:**
- Rewrite each "this manual is pinned to X … exits 5" paragraph to state the pinned behaviour: exit 4 for `mnemonic repair --ms1` and `ms repair`; mk 0.13.0 rejects a complete group at exit 2 and keeps exit 5 plus the advisory for an incomplete group.
- Replace the chapter 12 and chapter 82 tables and lists, and the three overview banners, with the four `*-tag-implied` values.
- Optionally, have `lint.sh` compare these sentences against `pinned-upstream.toml`, so the next bump cannot miss them. The GUI install-page pin is also ungated, as the implementer noted.

## Minor

### M-1 — `restore --md1` with a depth-2 tap tree is documented as refused (exit 2); it succeeds.

The claim is at `src/40-mnemonic/4d-restore.md:482` and `:623`. I bundled the v0.62.0 fixture `tests/tutorial/fixtures/taproot-4leaf.desc` (`mnemonic bundle --network mainnet --descriptor "$(cat taproot-4leaf.desc)"` gives 25 md1 chunks). `mnemonic restore --md1 …×25` then returned **rc=0** with a four-leaf `tr(…,{{…},{…}})`. Its first receive address is `bc1p6yc7kzttzsafprr6hwsaefuyqxvee4j48zdrqt4kl9ers68mhcestwvn66`. `export-wallet --format bitcoin-core-addresses` gives that same address for both the original and the restored descriptor. This is a same-tool consistency check, not an independent derivation.

The claim is not dangerous, because the refused input now works. But it contradicts the J4 rewrite's own premise that PR-#953 is fixed.

Toolkit-side follow-up: the released `mnemonic restore --help` for `--md1` still says "depth-≥2 taproot are refused".

### M-2 — `bundle --descriptor` with `--account != 0` is documented as refused; the CLI accepts it for non-canonical descriptors.

The claim is at `src/40-mnemonic/42-bundle.md:41,230,374-376,645`. Run: `mnemonic bundle --network mainnet --descriptor "wpkh([73c5da0a/84'/0'/0']xpub6CatWdiZ…PW6V/<0;1>/*)" --account 1` gives rc=0, with origin `m/84'/0'/0'`.

The guard has been canonicity-gated since v0.19.0 (`crates/mnemonic-toolkit/src/cmd/bundle.rs:292-299,1422-1429`). The GUI cannot reach this case: `gui_schema.rs:579-596` pins `--account` to 0 whenever `--descriptor` is present. The page should say the refusal applies to canonical descriptors only, and that the GUI coerces `--account` to 0.

### M-3 — Byte-exact message drift (wrong wording only; the refusals still happen)

- `60-ms/63-encode.md:270`: the actual clap group is `<--phrase <PHRASE>|--hex <HEX>|--in <FILE>>` (exit 64).
- `50-md/53-encode.md` (runtime pre-check row): the actual text is `encode: TEMPLATE required (on argv, via --in FILE, or use --from-policy with cli-compiler)`.
- `50-md/59-address.md` (first row): not a runtime pre-check. The actual refusal is the clap group `<PHRASES|--template <TEMPLATE>|--from-mk1 <STRING>...>`.
- `40-mnemonic/42-bundle.md`:
  - `--descriptor` with `--descriptor-file` hits clap `conflicts_with` (exit 64), not the quoted `mode_text` string.
  - The `--passphrase` conflict text differs: the actual is `the argument '--passphrase <PASSPHRASE>' cannot be used with '--passphrase-stdin'`.
  - The `--passphrase` advisory is quoted with "— use --passphrase-stdin"; the actual text is "— pipe via --passphrase-stdin".
- `40-mnemonic/46-derive-child.md`: the unknown-node list omits `seedqr`.
- `60-ms/68-combine.md:97`: "`threshold not passed`" — the actual is `not enough shares: have 1, need 2`.
- `60-ms/67-split.md:231`: K outside `2..=9` exits 1 at runtime (`invalid threshold 1; K-of-N shares require k in 2..=9`), not a value-parser refusal.

## Item 2 — Chapter 50 and the J1/J3/J5 prose: all correct

- **Corpus:** 148/148 tutorial artifacts and 61/61 gallery PNGs are byte-identical to `git archive mnemonic-gui-v0.62.0` (tag object `3b37143b`, which peels to `9f569e1`, the commit the implementer cited).
- **J4 step 14:** `mnemonic export-wallet --descriptor "$(cat taproot-4leaf.desc)" --format descriptor` gives rc=0. Its stdout and stderr are **byte-identical** to `tut-j4-14-depth2-export.{stdout,stderr}.txt`.
  - The prose matches the transcript: `#trqmzhua`; `Kint [73c5da0a/84'/0'/4']`; the `after`+hashlock pair and the `older` pair; each leaf is `and_v(…, multi_a(…))`; stderr is only the watch-only note.
  - The intro's "one refusal" is true: `tut-j4-19-bsms-unsupported` is J4's only `refusal_step` in the tag's manifest, and it reproduces with rc=2 and the same message.
  - The built HTML carries `trqmzhua` and no `depth2-refusal` text.
- **J1:** the `tut-j1-01` stdout cards are unbroken; the stderr order and warnings match the prose.
- **J3:** I decoded all 11 xpub pairs in canonicalise versus restore. The key and chain code are identical, depth 3 and the child number are kept, and the parent fingerprint is `00000000`. That is exactly what the new prose says.
- **J5:** `#284ufd99` is present in `tut-j5-23-restore-descriptor.stdout.txt`.

## Item 3 — Installer pins: correct

- `sh scripts/install.sh --root <scratch> --no-man`: rc=0, "5 installed". The versions are mnemonic 0.104.0, md 0.20.3, ms 0.19.1, mk 0.13.0 and mnemonic-gui 0.62.0 (sha `2f10ee0b…`, verified against `SHA256SUMS`).
- `ms encode --allow-argv-secret --phrase "abandon ×11 about"` gives `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f`. `ms verify --allow-argv-secret --phrase "<P>" <ms1>` gives `OK: round-trip valid (12 words, language=english)`, rc=0. Without the flag it refuses before parsing (rc=1), as §14 says.
- **Floors (`readelf -V`):** the GUI x86_64 installed binary and the aarch64 asset (SHA256SUMS OK) both need GLIBC_2.18, and md x86_64 needs GLIBC_2.34. That matches `glibc_floor()` (install.sh:125-131), `--help`, the README and the CLI manual's floor table.
- **The GUI install pages are consistent with the installer:**
  - `--tag mnemonic-gui-v0.62.0` appears three times.
  - MSRV 1.88 matches `rust-version = "1.88"` at the tag.
  - `--no-gui` and `--only mnemonic-gui` exist in `install.sh --help`.
  - The Path C asset names match.
  - The sentence about the source fallback below glibc 2.18 or on musl is correct.
- **F-681:** `--dry-run --root R` sends man pages to `~/.local/share/man/man1`. It uses `$XDG_DATA_HOME/man/man1` when that is set, and `--man-dir D` when given. That matches the new sentence.

## Item 4 — Gates: all green

| Gate | Result |
|---|---|
| Examples golden (`EXAMPLES_BIN_DIR`=scratch root, released 0.104.0) | regenerated == committed (`diff` empty) |
| sibling-pin-check (workflow script run verbatim) | rc=0, "OK all sibling pins match" |
| `docs/manual` `make audit` (`*_BIN`=scratch root) | rc=0 (anchor-check: 9 danglers = baseline) |
| `docs/quickstart` `make lint` / `verify-examples` | rc=0 / rc=0 |
| manual-gui `make html` / `lint` / `verify-examples` | rc=0 / 12/12 OK (1009 anchors, 128 outlines, 61+50+98 corpora) / 17/17 |
| manual-gui `verify-examples-gui` (gui-render built from the tag) | 61/61, no secret leak |
| manual-gui `make pdf` / `gui-example-pdf` / `gui-example-html` | rc=0 ×3 |
| install-verify / msrv-guard / man-step / install-assets, bash **and** dash 0.5.13.5 | all OK ×8 |
| shellcheck 0.11.0 | `install.sh` clean. `*.test.sh` has 11 warnings, **identical at base `77a0d172`** (not in CI; unchanged by this diff) |

## Observations outside the diff (not graded)

- **The installer's printed `man -M` recipe doesn't work** (pre-existing, `642300b7`, `install.sh:930`). It prints `man -M "<MAN_DIR>"`, where `MAN_DIR` is the `…/man/man1` directory. `man -M …/man1 mk` gives "No manual entry for mk", while `man -M …/man mk` works. The comment above it says "The `-M` fallback is always correct".
- **`final-word --from phrase=@env:VAR` does not resolve `@env:`.** It treats the literal string as the phrase ("got 1 words") and fires the argv warning. The manual makes no `@env` claim for final-word.
- **`slip39 split --from phrase=@env:VAR` fires the "secret material on argv" warning** even though it does resolve the variable. This is a secret-handling class issue, so it never gates.

ready to ship: no
