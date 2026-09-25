# F-679 fold 2 — mnemonic-gui `f679-pins` @ `5d9709c` (ms 0.19.1)

**Outcome:** one commit, `5d9709c`, on top of `69e3adb`. It pins ms 0.19.1 and adds a real-binary `ms verify` test that uses a phrase plus a positional ms1. The test passes on 0.19.1 and fails on 0.19.0 with I1's exact error. CI is green: build run **36105111445**, all jobs passed and `release` was skipped as it only runs on tags; schema-mirror run **36105111454** passed. In that run the new test executed against ms built from `ms-cli-v0.19.1` and passed. Nothing is tagged or merged.

## Binaries

| CLI | Source | Version |
|---|---|---|
| mnemonic, md, mk | toolkit installer v0.104.0, `--no-gui --no-man`, scratch root | 0.104.0 / 0.20.3 / 0.13.0 |
| ms | `gh release download ms-cli-v0.19.1`, asset `ms-0.19.1-x86_64-linux-musl.tar.gz` | 0.19.1 |

- **Checksum:** sha256 `92a45a92d8e8995cacff838c38f8930219b757f1752572eff4cfdbe839b2eb55`. This matches the line in the release's `SHA256SUMS.x86_64`.
- The installer's ms 0.19.0 binary was kept aside, only to show the new test fails on it.

## Pin changes (every place the ms pin lives)

- `pinned-upstream.toml`: `[ms].tag` changes from `ms-cli-v0.19.0` to **`ms-cli-v0.19.1`**, and the comment now records why.
- `src/schema/ms.rs`: `pinned_version` changes from `"ms 0.19.0"` to **`"ms 0.19.1"`**.
- `README.md`: the ms `cargo install … --tag` line now reads `ms-cli-v0.19.1`. `readme_pin_coherence` passes.
- `CHANGELOG.md`, 0.62.0 entry:
  - the headline now names **`ms` 0.19.1**;
  - the pin sentence now reads `ms v0.16.0 → v0.19.1`;
  - a new bullet explains the I1 fix and that the mirror did not change.
- **Left unchanged:** comments of the form `// F-679 (ms-cli v0.19.0): …`. They record which release introduced a surface, and that is still true.

## Mirror and snapshots: nothing moved, and measured as such

- `ms gui-schema` from 0.19.1 is **byte-identical** (`cmp`) to the output from 0.19.0.
- I compared `--help` for all 8 mirrored ms verbs. The flag-token sets are identical; only the EXAMPLES text changed. For example, `verify` now teaches `--in card.ms1 --phrase - < phrase.txt`.
- No schema-mirror expectation, form PNG or tutorial artifact changed. The form-snapshot suite and the tutorial harness are both green.

## New test: `real_ms_verify_phrase_and_positional_ms1`

- **File:** `tests/f679_argv_secret_admission.rs`.
- **What it does:** puts the phrase and the ms1 card in the GUI's secret widgets, builds the Run argv (which includes `--allow-argv-secret` and `-- <ms1>`), runs it against `MS_BIN`, and expects exit 0.

| MS_BIN | Result |
|---|---|
| ms 0.19.1 (release asset) | **pass** |
| ms 0.19.0 (installer's release binary) | **FAIL**: `error: cannot read both ms1 and --phrase from stdin`, exit 1. This is I1's exact failure. |

## Gates run on `5d9709c`

| Gate | Result |
|---|---|
| `cargo nextest run --locked`, `*_BIN` = the four binaries above | **710 passed, 6 skipped**. The skips are the same 6 as before: 3 proptest finders, 2 tutorial harness tests run separately, `manual_anchor_coverage`. |
| clippy `--all-targets -D warnings`; `--no-default-features -D warnings` | clean |
| MSRV `rustup run 1.88.0 cargo check --locked` | clean |
| Tutorial harness `--include-ignored`, release mnemonic | 12 passed |
| `GUI_SNAPSHOTS=1` form snapshots | 2 passed, no PNG changed |
| rustfmt on the touched test file | clean |
| CI | build 36105111445, all jobs passed; schema-mirror 36105111454, passed. The `ci/f679-pins` trigger ref was deleted afterwards. |

## Concerns

- **The toolkit installer (v0.104.0) still pins ms 0.19.0.** When the controller updates the installer to pin GUI v0.62.0, it should move ms to 0.19.1 in the same change. Otherwise the installer ships the ms that breaks the GUI's verify-with-phrase flow.
- **Scratch:** `/scratch/code/shibboleth/f679-scratch/` was removed with `find … -delete`.
