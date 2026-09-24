# F-677 (ms + mk items) and F-670: implementation report

Date: 2026-09-24. Implementer: a single agent, with no subagents. Nothing was pushed, tagged or merged.

| repo | branch | base | HEAD |
|---|---|---|---|
| mnemonic-secret | `f677-help` | `f845bac` | `75f2168` |
| mnemonic-key | `f677-label` | `9ccd549` | `1b8c524` |

Toolchain: tests and clippy ran on the pinned 1.85.0 (with its toolchain bin prepended to PATH, and `clippy 0.1.85` confirmed). fmt ran on 1.95.0, which is what both repos' CI uses. `CARGO_TARGET_DIR` was `/scratch/code/shibboleth/{ms,mk}-worktrees/f677-target`.

## mnemonic-secret: `75f2168`

### 1. `ms encode --group-size` help
The code (`encode.rs` `emit_text`) applies `render_grouped` only to the stderr `engraving card:` line. Stdout always prints the canonical ms1, and `--out` and `--json` are always ungrouped. The old help said the flag grouped "the emitted ms1 string". The new help says it groups the engraving card only: stdout, `--out` and `--json` always carry the unbroken ms1, and the flag has no effect under `--no-engraving-card`.

### 2. Help EXAMPLES put the secret on argv
I checked every verb, not only `repair`. The guard (`argv_guard.rs`) refuses any value after `--phrase/--hex/--ms1/--passphrase/--hashlock-phrase` other than `-`, and any positional token shaped like an ms1, a phrase or hex. Examples that were refused, or that took a placeholder the guard would refuse:
- **encode**: `--phrase "…"` twice, and `--hex 000…`
- **decode**: `ms10entrs…` and `<ms1>` twice
- **inspect**: `<ms1>` twice. The `printf "ms10e…" | ms inspect -` line was not refused, but it wrote the card into shell history.
- **verify**: `<ms1>` in all three examples, plus `--phrase "…"` twice
- **repair**: `--ms1 ms10…` twice
- **split**: `--phrase "…"` twice, and `--hex 000…`
- **combine**: `<share1> <share2>` in all four examples

All of these now use `--in FILE`, `-` / `--phrase -` / `--hex -` with a redirect, or `--ms1 - <`. Two lines were added: `split … --out shares.txt` and `repair … --out repaired.ms1`. `encode` also gains a three-line note that the secret never goes on argv. The examples for hashlock, vectors, gui-schema and gen-man were already clean and are unchanged.

### 3. `ms split --out` advisory
The advisory was unconditional. It now uses encode's F-589 condition, `args.json || args.out.is_none()`, measured the same three ways as encode:
- `--out` in text mode: stdout is empty, so no warning.
- No `--out`: the shares are on stdout, so it warns.
- `--out --json`: the JSON carries `"shares"`, so it warns.

Other verbs: `decode` and `combine` have no `--out`. `repair --out` still prints the report, including the corrected chunk, on stdout (`repair.rs`, read from the code), so its unconditional advisory stays true. `hashlock` warns only under `--json`.

### 4. F-670: the md compose fragment
New card line:
`for md compose:  --wrapper wsh --path <your other paths> --path keyless,<kind>=<hex> --experimental --md-only`

Measured against the installed `md 0.20.2` for a key-less path:
- `tr` refuses it ("this build will not put a key-less path in taproot").
- `sh` and `sh-wsh` refuse it.
- `wsh` without `--experimental` exits 1.
- `wsh --experimental` without `--md-only` exits 1 ("Pass --md-only"). An accidental run of the M4 mutant binary reproduced this for all four kinds.

**Verified verbatim:** I took each printed fragment from the rebuilt `ms`, replaced only `<your other paths>` with `2of3`, and ran `md compose <fragment>`. It exits 0 for sha256, hash256, ripemd160 and hash160, and prints `wsh(or_d(multi(2,…),<kind>(<hex>)))`.

### Tests and gate
New file: `crates/ms-cli/tests/f677_help_matches_behaviour.rs`, with 5 tests.
- **`every_help_example_runs_and_none_puts_material_on_argv`** executes all 40 EXAMPLES lines. It covers every subcommand listed by `ms --help`, and every verb except `derive` must have examples. Each line runs in `bash -o pipefail` in a fresh tempdir with real fixture files. A `| jq F` tail is replaced by parsing the JSON and checking that `F` exists. A line fails on exit ≠ 0 (repair may exit 4) or on the guard's "on ARGV".
- One test for each of: the encode help text (plus the behaviour behind it), the repair private channel, the split advisory in all three modes, and the md fragment for all four kinds.

Mutations: one per item, each applied by an exact-match replace that asserts it matched, then the file was restored. All four were killed for the right reason:
- M1: restoring the old group-size doc fails the encode-help test ("the old wording is back").
- M2: putting a real ms1 on argv in a repair example fails the examples test ("refused by ms's own argv guard … argument 3 on ARGV") and the repair test.
- M3: making the split advisory unconditional fails ("warned … when stdout is EMPTY").
- M4: dropping `--md-only` from the fragment fails the fragment test.

Gate results:
- `cargo nextest run --locked --workspace`: 613/613, 11 skipped (baseline 608 + 5 new).
- `cargo clippy -p ms-cli` and `-p ms-codec`, both `--all-targets -D warnings`: clean.
- `cargo +1.95.0 fmt --all --check`: clean.
- `Cargo.lock` is unchanged, so `vendor-freshness.sh` does not apply.

CHANGELOG: a new `## ms-cli [Unreleased]` section with a `### Fixed` list. The version stays 0.19.0; the release is the next ms release, which owns F-670.

## mnemonic-key: `1b8c524`

**Decision: `0.14.0-dev`, not a release commit.**
- **Why 0.14.0:** `[Unreleased]` already holds three BREAKING entries (stdout is now the artifact, `--separator` is whitespace-only, an invalid artifact exits 1), so the pre-1.0 convention calls for a minor bump.
- **Why `-dev`:** descriptor-mnemonic used `0.2.0-dev` (`06f74719`). It sorts between 0.13.0 and 0.14.0 and can never equal a release.

Why main is not ready to release:
1. `crates/mk-codec/src` changed since the `mk-codec-v0.5.0` tag: the encode path-depth refusal (`47d7f97`), the csid_ext corpus (`58c8df4`) and a doc change. The codec CHANGELOG has no entry and the version is still 0.5.0, so a release needs a codec bump or release decision first.
2. `[Unreleased]` was missing three changes that had shipped to main: the P1 read-side chunk_set_id warning (`37a9524`), the P2 mint and repair warning (`1711228`), and the decode/verify correction note (`c1b56b9`). I added all three. They are written from the commit messages and checked against the test names in `csid_verification.rs`, `encode_repair_chunk_set_id_p2.rs` and `decode_verify_correction_note.rs`.
3. Cutting a release with BREAKING changes is the maintainer's call.

Changes:
- `crates/mk-cli/Cargo.toml` and `Cargo.lock` move 0.13.0 → 0.14.0-dev.
- The CHANGELOG `[Unreleased]` section now names "Next release: 0.14.0" and records the 83-commit gap (`git rev-list --count mk-cli-v0.13.0..9ccd549` = 83).
- The three missing entries above.
- New `tests/version_is_honest.rs`:
  - A release version needs a `## [X.Y.Z]` heading and an empty `[Unreleased]`.
  - A `-dev` version needs `[Unreleased]` to name its base, and that base must not already have a heading.
  - `mk --version` must equal `mk <CARGO_PKG_VERSION>`.

Mutations, both killed:
- Version set back to 0.13.0 (the original defect): "0.13.0 is a release version, but `## [Unreleased]` lists changes…".
- Version 0.14.0 with no heading: "no `## [0.14.0]` CHANGELOG heading".

Gate results:
- `cargo nextest run --locked --workspace`: 391/391 (baseline 389 + 2).
- `cargo clippy --workspace --all-targets -D warnings`: clean.
- `cargo +1.95.0 fmt --check`: clean.
- `ci/repro/vendor-freshness.sh`: OK.
- The built binary prints `mk 0.14.0-dev`.

**The version string the toolkit docs should cite:** mk-cli **`0.14.0`**, for example "(unreleased: mk-cli 0.14.0: --in)". A build from main prints `mk 0.14.0-dev`.

## Concerns
- **ms, F-670:** the fragment now prescribes `--wrapper wsh` and `--md-only`. `--md-only` means no coordinator md knows can sign the wallet. md prints that note itself, but the card does not say it. The fragment is a correct transcription of what md 0.20.2 requires, not an endorsement of a key-less path.
- **ms:** the conditional line for non-sha256 kinds ("requires `<kind>=` support in `md compose` … unknown option") is still true, but it is now noise: md 0.20.2 accepts all four kinds. I left it alone because it is outside F-670.
- **ms:** `split --group-size` has no effect under `--out` in text mode, and its help does not say so. This is Minor and I left it.
- **ms:** the toolkit manual (`43-ms.md`) carries none of the changed example strings (grep found none), and no flags changed, so no lockstep edit is needed.
- **mk:** `mk-codec` has the same defect class. Main's codec differs from the `mk-codec-v0.5.0` tag but still says 0.5.0. I did not bump it, because that would mean changing mk-cli's `version = "0.5.0"` requirement as well.
- **mk:** mk's CLAUDE.md says "Do NOT add a rust-toolchain.toml", but one exists, pinned to 1.85.0. This contradiction predates this work.
