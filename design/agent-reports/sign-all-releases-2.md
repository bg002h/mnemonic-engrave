# Sign every release — leftovers round (report 2)

Follows `design/agent-reports/sign-all-releases.md`. These are the controller's four leftover items, all on the `sign-releases` branches, all pushed. Nothing was merged, tagged or pushed to a default branch, and no release was created (checked).

## Branch tips

| repo | tip | this round |
|---|---|---|
| mnemonic-toolkit | 2917014084 | a30efb7f install-verify signed fixtures (+ rust.yml order) · 29170140 docs |
| mnemonic-secret | 2dbac5b240 | 2dbac5b docs |
| descriptor-mnemonic | 81945b5b34 | 81945b5b docs |
| mnemonic-key | 90fd9bfcf7 | 90fd9bf docs |
| mnemonic-gui | 96e9eee511 | 96e9eee README |
| mnemonic-engrave | 209e26ec0c | (controller's trust-anchor commit; no change from me) |

## 1. `install-verify.test.sh` survives a filled `first_signed`

**Approach.** I did not touch the ~20 fixture writers. Instead:
- The stub `curl` answers a request for `<sums>.minisig` by signing that fixture sums file on demand with a **throwaway** key generated in the test (`minisign -G -W`).
- The installer under test is a copy whose `signing_keys` `*|||` line trusts only that key. The rewrite is asserted, so a no-op substitution cannot pass.
- Every fixture is therefore a correctly signed release, whatever a case does to the sums file.
- `INSTALL_VERIFY_FIRST_SIGNED=<ver>` fills mk's `first_signed` in the copy.
- Without minisign, the stub serves no signatures and the installer skips the check, which is the old behaviour.
- `rust.yml` now installs minisign **before** this harness, so CI takes the signed path.

**Proof:**

| run | result |
|---|---|
| plain | 47/47 ok |
| `INSTALL_VERIFY_FIRST_SIGNED=0.13.0` (mk's current pin, i.e. "at the first signed version") | 47/47 ok |
| **control**: the previous version of the test against an install.sh copy with `mk) echo "0.13.0"` | fails **13** cases |

The control shows the old fixtures would have broken, and that the new run really verifies signatures (a missing or bad one would be refused).

- shellcheck `-S warning` is clean. It also fixes the old file's one warning (`CDPATH=`).
- **CI on 29170140**: rust.yml run **36217954576**, success, 0 non-success jobs. Its steps "Install minisign", "install-verify harness", "install-signature harness" and "install-assets check" all succeeded.
- The toolkit's other workflows on 29170140 are green too: sibling-pin-check, technical-manual, examples, manual-gui, release, quickstart, manual.
- **Side finding.** A branch *creation* push does not start path-filtered workflows here, so rust.yml did not run on the first push of a recreated `ci/` ref. To get a run, I pushed the ref to 9efaa885 and then forward to 29170140, cancelling the throwaway runs on 9efaa885. The `ci/sign-releases` ref is deleted again.
  - This correction matters: in report 1, the "rust.yml never runs on installer-only pushes" finding was drawn from a creation push. The fix itself stands: `paths:` genuinely lacked `scripts/install.sh` and `scripts/install-*.test.sh`.

## 2. Docs: signature verification with the new key

Each changed doc gives:
- the pinned key `RWRUl0DYNI0r72HYC0ou+T/7pHEf0km3a8RWHwqGwZmIEMWtiSd4k0B5` (id `EF2B8D34D8409754`);
- the exact commands `minisign -Vm <sums> -P <key>` then `sha256sum -c <sums> --ignore-missing`;
- a table of which checksum file and `.minisig` go with each platform;
- a note that releases from before signing have no `.minisig`.

| file | `.minisig` files described |
|---|---|
| toolkit `README.md` (replaces report 1's short Origin block), `docs/verify-reproducibility.md` §6 | musl Linux → `SHA256SUMS.x86_64.minisig` / `SHA256SUMS.aarch64.minisig`; macOS/Windows → `SHA256SUMS.portable.minisig` |
| ms `README.md`, `docs/verify-reproducibility.md` §6 | same |
| mk `README.md`, `docs/verify-reproducibility.md` §6 | same |
| md `README.md`, `docs/verify-reproducibility.md` §6 | musl Linux → `SHA256SUMS.<arch>.minisig`; glibc Linux (`md-<ver>-linux-amd64/arm64`), macOS, Windows → `SHA256SUMS.portable.minisig` |
| gui `README.md` (Install section) | one `SHA256SUMS.minisig` covering all seven archives |

- ms, md and mk each carry their own `verify-reproducibility.md`, so I updated those alongside the READMEs.
- No gate lints these files: I checked every repo's workflows. The command syntax is the one the CI verify steps run.
- The asset names follow install.sh's `asset_for` and the generator's `pack` lines.

## 3. mnemonic-transaction — not regenerated; what it needs

State now:
- `bg002h/mnemonic-transaction` has **no secrets** (`gh secret list` is empty).
- Its default branch is `main` @ e8fd044.
- Its committed `release.yml` is the old generator output: a dormant signing step that skips when the secret is absent, which is unsigned today.

Regenerating would change 85 lines, bringing in the signing policy, the `sign_dry_run` input, the new pinned key and the single-call sums + `.minisig` upload. It would make any tag **fail** until the secrets exist. To enable signing:

1. **Operator:** set both secrets in mnemonic-transaction (the same key as the other six):
   `MINISIGN_SECRET_KEY` and `MINISIGN_SECRET_KEY_PASSWORD`.
2. **Regenerate** from engrave's generator (engrave `sign-releases` ≥ 7864c81a, or master after merge):
   `cd mnemonic-engrave/scripts/release-workflows && python3 emit.py`
   - `emit.py` rewrites `/scratch/code/shibboleth/<repo>/.github/workflows/release.yml` in **all five** checkouts (md, ms, mk, mt, toolkit).
   - Keep only the mt change, or run it after the other four branches have merged. The other four's outputs are byte-identical to their `sign-releases` branches, so for them it is a no-op once merged. I re-checked this against the current generator: generator == branch for toolkit, ms, md and mk.
3. **Commit** on an mt branch. Then prove it with `gh workflow run release.yml --repo bg002h/mnemonic-transaction --ref <branch> -f sign_dry_run=true`: expect the Sign and Verify steps successful, and Attest / Ensure-release skipped.
4. mt has no musl pipeline, so `SHA256SUMS.portable` is the only file it signs. Its README would also want the verify block from item 2.

## 4. Engrave run 36217201128 — confirmed

- Run: `workflow_dispatch` on `sign-releases` at **209e26ec**, which pins the new key twice in `release.yml` (VERIFY.txt text and the verify step) and the old key zero times (counted with `git show … | grep -c`). Conclusion **success**.
- `assemble` steps:

  | step | conclusion |
  |---|---|
  | Install minisign | success |
  | Write key | success |
  | Sign SHA256SUMS with minisign | success |
  | Verify SHA256SUMS.minisig against the pinned public key | success |
  | Remove key | success |
  | Dispatch rehearsal -- nothing published | success |
  | Attest build provenance | **skipped** |
  | Publish GitHub release | **skipped** |

- `gh release view sign-releases`: no such release.

**All six repos are now proven with the new key.** Non-publishing runs:

| repo | run ids |
|---|---|
| mnemonic-toolkit | 36215620417, 36215621875 |
| mnemonic-secret | 36215623161, 36215624391 |
| descriptor-mnemonic | 36215625533, 36215626780 |
| mnemonic-key | 36215628013, 36215629411 |
| mnemonic-gui | 36215630581 |
| mnemonic-engrave | 36217201128 |

## Still open (unchanged from report 1)

- `first_signed` in install.sh stays empty until each component's first signed release ships. Filling it is now safe for `install-verify.test.sh`.
