# Sign every release — fold 1 (response to `sign-all-releases-review.md`)

The review found 0C / 2I / 5M / 4N. This fold closes both Importants, all five Minors and three of the four Nits (N4 needed no change); the table below gives the status of each.

- All branches `sign-releases`, pushed. Nothing merged, tagged or pushed to a default branch.
- No `sign-releases` release exists in any of the six repos (checked).
- **Nothing was deleted** from `manual-gui-v1.4.0`; a recommendation is below.

## Branch tips

| repo | tip | fold commits |
|---|---|---|
| mnemonic-toolkit | e14b2302af | 9e3fabd5 workflows · 265366de installer I1/M2 · e14b2302 docs N3 |
| mnemonic-secret | d59d6a48d2 | 98c5ed3 workflows · d59d6a4 docs |
| descriptor-mnemonic | b15330de02 | 0b8156da workflows · b15330de docs |
| mnemonic-key | a49db3956a | 15cae25 workflows · a49db39 docs |
| mnemonic-gui | 2e2fae3e11 | 2e2fae3 build.yml + README |
| mnemonic-engrave | d69550548e | d6955054 generator + release.yml + pin check |

## Finding → change

| id | change | proof |
|---|---|---|
| **I1** | `install-assets.test.sh` now gates **both** directions via `signed_pin_gate` (see below). **Runtime fail-safe:** `fetch_sig` classifies the `.minisig` download, and only an HTTP **404** counts as missing; any other failure is refused, never read as "unsigned". **`first_signed`:** all five stay empty because **no signed release exists yet** (2026-09-26). The comment now says to fill each entry in the same commit as the pin bump, and the gate fails otherwise. | See "Proofs, I1" |
| **I2** | `gen.py`/`emit.py` take a per-repo `tag_prefix`: `mnemonic-toolkit-v`, `ms-cli-v`, `descriptor-mnemonic-md-cli-v`, `mk-cli-v`, `mt-cli-v`. The tag trigger is now `'<prefix>*'`, and a publishing backfill dispatch refuses any tag outside the prefix. Regenerated into toolkit, ms, md and mk. | Regeneration is byte-identical to each branch's `release.yml`. Workflow harness 108/108, including: a CLI-tag backfill publishes; a `manual-gui-v1.4.0` backfill is refused; a dry run of any tag is allowed. I checked every repo's tag families: all current CLI tags match. The only casualty is md's 3 legacy `md-cli-v*` tags, which can no longer be backfilled. |
| **M1** | Adds `concurrency:` to the generated `release.yml`, grouped by release tag (`release-<tag>`) for tag pushes and backfills. Other runs get `github.run_id`, so they never queue or cancel each other. `cancel-in-progress: false`. Archive uploads lose `2>/dev/null \|\| true`: every generated repo always has both `.tar.gz` (macOS) and `.zip` (Windows), so both globs match. The only remaining `\|\| true` is on `gh release create`, deliberately: another tag workflow may have created the release first. | actionlint clean |
| **M2** | New `install-signature.test.sh` cases: 14 (key scoped to `mnemonic-engrave` signs mk → refused), 15 (key `mk\|999.0.0\|\|` → refused), 16/17 (`.minisig` download fails: network / HTTP 500 → refused). | Mutations: *component match ignored* → case 14 fails; *lower bound ignored* → case 15 fails (the two the review saw survive); *download error treated as missing* → 16 and 17 fail; `check_signature` removed → 15 cases fail; verify forced true → 6 fail. One mutation survives and is **inert**: making the missing-branch test `!= ok` is unreachable for `error`, which returns earlier. |
| **M3** | Every dispatch or dry run uploads the signed checksums as a workflow artifact: `signed-sums-dry-run` from each `release.yml`, the GUI and engrave; `signed-sums-<arch>` from each musl leg. The upload is guarded by `signed == true` and uses `if-no-files-found: error`. | See "Proofs, M3" |
| **M4** | Engrave: `scripts/check-minisign-pin.sh` requires every key-shaped string in `release.yml`, `README.md` and `scripts/release-workflows/gen.py` to equal `minisign.pub` line 2, with minimum copy counts of 2, 2 and 1. The retired key is allowed only on a README line saying "retired". It runs in the `test` job ("signing key copies agree with minisign.pub"). | OK on the tree. 8/8 mutations caught against a copy: each of the 5 key copies changed; `minisign.pub` rotated alone; the retired key moved onto a verify line; `release.yml`'s copies deleted. An initial version had a broken format check that made every mutation "caught"; I found it, fixed it, and re-ran from a green baseline. Green in CI in run 36220168609. |
| **M5** | Documented in the header comment of every signing workflow (generated `release.yml`, the 4 musl callers, GUI and engrave): a tag runs the workflow files at the tagged commit, so tag only commits containing the signing workflows. Also in the toolkit CHANGELOG. | — |
| **N1** | "ONE call … never carries" comments reworded to "fails loudly rather than passing with a stale or missing `.minisig`" (generator and musl callers). | — |
| **N2** | GUI `sha256sums` step: `shell: bash` + `set -euo pipefail`. | actionlint; run 36220167240 green |
| **N3** | Every doc block now gives the macOS spelling `shasum -a 256 -c <file> --ignore-missing` (Perl `shasum` 6.04 supports `--ignore-missing`; checked locally). The GUI README also gives the Windows `Get-FileHash` route. | — |
| N4 | No change: `--help` already says source builds are unaffected by `--require-signature`. | — |

## Proofs, I1 (installer)

- **Gate rule, offline.** `scripts/install-assets-gate.test.sh` extracts `signed_pin_gate` from `install-assets.test.sh` and exercises it: **8/8**. Its mutations are all caught: empty `first_signed` accepted (the I1 hole), `first_signed` above the pin accepted, the old direction removed, the published flag ignored. Wired into `rust.yml`.
- **Gate, live.** `install-assets.test.sh` passes **32/32 mapped assets through the gate, in both directions**: no pinned release publishes a `.minisig` yet, and every `first_signed` entry is empty.
- **`fetch_sig`, against real GitHub:** an existing `.minisig` (engrave v0.12.0) → `ok`; mk-cli-v0.13.0's absent `.minisig` → `missing`; an unreachable host → `error`. Identical results through the **curl** path and the **wget** path.
- **Why a real 404 is still allowed before `first_signed`.** Controller item: "refuse if the release lists the `.minisig` but the download is missing it". The installer cannot list a release's assets without the rate-limited GitHub API. The stripped-signature case (a real 404) is therefore closed by the `first_signed` pin, which the gate now forces to be filled on the first signed pin.

## Proofs, M3 (dry-run artifacts)

- Every signed-sums artifact from the 10 proof runs was downloaded and verified **offline** with `minisign -V -P RWRUl0DYNI0r72HYC0ou+T/7pHEf0km3a8RWHwqGwZmIEMWtiSd4k0B5`: **14/14 files verified**.
- mk `SHA256SUMS.portable` (run 36220164206):
  - with the new key: `Signature and comment signature verified` / `Trusted comment: mk sign-releases SHA256SUMS.portable`;
  - the retired key refuses it (key id `EF2B8D34D8409754` vs `CA39ECB257009A0F`, rc=1);
  - a tampered copy fails (rc=1).

## Test results (final tree)

- **Toolkit:**
  - install-signature **18/18**;
  - install-verify **47/47**, both plain and with `INSTALL_VERIFY_FIRST_SIGNED=0.13.0`;
  - install-assets-gate **8/8**;
  - install-assets (live) OK;
  - install-msrv-guard OK;
  - shellcheck clean on `install.sh` and the new or changed tests;
  - the Examples `--dry-run` golden still matches.
- **Workflow harness:** 108/108 (extracted `run:` blocks from all 6 repos, throwaway keys).

## Proof runs (parallel, all non-publishing, all success)

In every run: the sign and verify steps succeeded; upload, release, attest and publish steps were skipped; the signed-sums artifact was present and verified offline.

| repo | workflow | run id |
|---|---|---|
| mnemonic-toolkit | release.yml (`sign_dry_run`) | 36220154979 |
| mnemonic-toolkit | man-pages.yml | 36220156546 |
| mnemonic-secret | release.yml (`sign_dry_run`) | 36220158131 |
| mnemonic-secret | man-release.yml | 36220159612 |
| descriptor-mnemonic | release.yml (`sign_dry_run`) | 36220161162 |
| descriptor-mnemonic | man-pages.yml | 36220162982 |
| mnemonic-key | release.yml (`sign_dry_run`) | 36220164206 |
| mnemonic-key | musl-binaries.yml | 36220165785 |
| mnemonic-gui | build.yml | 36220167240 |
| mnemonic-engrave | release.yml (also runs the new pin check) | 36220168609 |

**Toolkit full CI at e14b2302** (temporary `ci/sign-releases`, since deleted; the throwaway runs on 9efaa885 were cancelled), all success:

| workflow | run id |
|---|---|
| rust (all jobs; its installer steps below) | 36220180891 |
| examples | 36220180852 |
| manual | 36220180915 |
| manual-gui | 36220180845 |
| quickstart | 36220180889 |
| technical-manual | 36220180982 |
| sibling-pin-check | 36220180860 |
| release | 36220180953 |

The rust run's steps `install-verify`, `install-signature`, `install-assets signed-pin gate (offline)` and `install-assets check` all succeeded.

## `manual-gui-v1.4.0`: stray binaries (listed, not deleted)

I scanned every non-`mnemonic-toolkit-v*` release in bg002h/mnemonic-toolkit. Only this one carries CLI assets, all from run 36211422527 (2026-09-26T02:28Z, the old `*v[0-9]*` trigger):

| asset | bytes |
|---|---|
| `mnemonic-1.4.0-macos-amd64.tar.gz` | 5,810,446 |
| `mnemonic-1.4.0-macos-arm64.tar.gz` | 5,588,848 |
| `mnemonic-1.4.0-windows-amd64.zip` | 5,503,851 |
| `SHA256SUMS.portable` | 299 |

**Recommendation: delete all four.**
- They claim a `mnemonic` 1.4.0 that does not exist.
- They are unsigned.
- Nothing references them: a grep of toolkit, GUI and engrave HEAD for `mnemonic-1.4.0` and `manual-gui-v1.4.0/(mnemonic|SHA256SUMS)` found nothing, and the installer maps only `mnemonic-toolkit-v*`.
- If a real toolkit 1.4.0 ever ships, the same asset names would exist in two releases.
- Keep the manual's own assets (`gui_example.pdf`, `m-format-gui-manual.{html,pdf}`).

Command for the operator:

```sh
for a in mnemonic-1.4.0-macos-amd64.tar.gz mnemonic-1.4.0-macos-arm64.tar.gz mnemonic-1.4.0-windows-amd64.zip SHA256SUMS.portable; do
  gh release delete-asset manual-gui-v1.4.0 "$a" --repo bg002h/mnemonic-toolkit --yes
done
```

## Open items

- **`first_signed`:** fill each entry in the commit that bumps that component's pin onto its first signed release. The gate now fails if that is forgotten.
- **mnemonic-transaction:** unchanged. It needs its two secrets, then a regeneration (report 2). Its `tag_prefix` (`mt-cli-v`) is already set in `emit.py`.
- **Concurrency scope:** it is only on the generated `release.yml`. The musl callers, GUI and engrave have no backfill dispatch, so two runs for one tag can only come from a re-run.
