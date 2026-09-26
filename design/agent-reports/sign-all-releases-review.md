# Adversarial review: minisign signing across six repos, plus installer verification

Brief: `design/briefs/sign-all-releases-review.md`. Reviewer: Opus 5.5, independent context. Date: 2026-09-26.

Branches reviewed: `sign-releases` at
- engrave 209e26ec
- toolkit 29170140
- ms 2dbac5b
- md 81945b5b
- mk 90fd9bf
- gui 96e9eee

Each was diffed against origin's default branch. Nothing was edited, committed, pushed, tagged or released. Scratch was deleted.

**Counts: 0 Critical, 2 Important, 5 Minor, 4 Nit.**

**Answer to the one question.**
- **CI side:** no. I found no path where a *release-tag* run of any of the 9 signing workflows uploads a checksum file unsigned, signed by the wrong key, or with a signature that does not cover the uploaded bytes, while reporting success.
- **Installer side:** yes. The installer *can* trust a checksum file whose signature was stripped from a signed release (I1). The mechanism that is supposed to prevent this, `first_signed`, is empty, and no gate forces it to be filled when the pins move onto signed releases.
- **Separately (I2):** after merge, over-broad tag filters will make the constellation key sign checksum files for binaries labelled with versions that were never released. This has already happened unsigned on `manual-gui-v1.4.0`.

---

## Important

### I1. The stripped-signature downgrade is open, and nothing gates closing it

**The mechanism.** Refusing a *missing* `.minisig` depends entirely on `first_signed <component>` in `scripts/install.sh`. All five entries are `""`. With an empty entry, a missing signature takes the "note" branch:
- `check_signature`: `note: $_cs is not signed (the $_cn $_cv release predates signed releases); checked by sha256 only`
- It returns 0 and the install proceeds.

**Consequence.** An attacker who can delete one release asset gets the pre-signing trust level back on a *signed* release, and the installer prints a false reason. Deleting a `.minisig` is well within the ability of anyone who can swap a tarball and its sums line, which is the threat signing exists to stop.

**Why no gate catches it:**
- `install-assets.test.sh` checks only one direction: "`first_signed` is filled ⇒ the pinned release has a `.minisig`".
- The converse is unchecked: "the pinned release has a `.minisig` ⇒ `first_signed` is filled and ≤ the pin".
- `first_signed` is referenced nowhere else except CHANGELOG prose (grep of `*.sh *.yml *.md *.py`).

**Where it bites first.**
- The toolkit's own `install-pin-check.yml` requires `install.sh` to pin the tag being released.
- So the first signed toolkit release (the next `mnemonic-toolkit-v*`) necessarily ships an `install.sh` that pins a signed release while `first_signed mnemonic` is `""`, unless someone remembers to fill it by hand in the release commit.
- md, ms, mk and gui follow at their next pin bumps.

**Reproduction.**
- Offline, this is exactly harness case 4 (`missing signature, no first-signed pin: installed with a note`).
- `sh scripts/install-signature.test.sh` passes it today. That behaviour is correct *only* while the pinned releases are genuinely unsigned.
- To see the gap: bump the mk pin to a release that has `.minisig` files, leave `first_signed mk` empty, and run `install-assets.test.sh`. It reports `ok` with `0 mapped assets checked for a published .minisig`, and the installer will accept that release with its `.minisig` deleted.

**Fix (small):** in `install-assets.test.sh`, for each mapped asset, if `$base/$sums.minisig` resolves, require that `first_signed "$n"` is non-empty and `version_ge "$ver" "$first"`.
- It then fails on the first pin bump onto a signed release, including the toolkit's self-pin.
- Alternatively, or as well, set `first_signed` in the same commit as the pin bump, and make the release checklist say so.

### I2. `release.yml`'s tag filter `'*v[0-9]*'` makes the constellation key vouch for mislabelled binaries

**The mechanism.** The generated `release.yml` (toolkit, ms, md, mk) fires on any tag matching `*v[0-9]*`, and `assemble` runs for any `refs/tags/`.

**Shown on real data.** In the toolkit, doc tags already trigger it:
- `gh release view manual-gui-v1.4.0 --repo bg002h/mnemonic-toolkit` lists `mnemonic-1.4.0-macos-amd64.tar.gz`, `mnemonic-1.4.0-macos-arm64.tar.gz`, `mnemonic-1.4.0-windows-amd64.zip` and `SHA256SUMS.portable` beside the manual PDFs.
- `gh run list … --workflow release.yml` shows the `manual-gui-v1.4.0` push run as `success`.
- The name comes from `VER="${TAG##*-v}"`, which gives `1.4.0`.
- The same trigger matches `manual-v*`, `quickstart-v*`, `tech-manual-v*`, `examples-v*` and `ultraquickstart-v*`.

**Same pattern elsewhere (latent: no such run found in their run history):**
- ms: `ms-codec-v*`, `mnemonic-secret-ms-codec-v*`
- md: `md-codec-v*`, `descriptor-mnemonic-md-codec-v*`
- mk: `mk-codec-v*`, `mnemonic-key-mk-codec-v*`

**What changes on merge.** The next such tag produces a `SHA256SUMS.portable` that is **minisign-signed with EF2B8D34D8409754**, vouching for `mnemonic-1.5.0-*` (or `ms-0.3.0-*`, and so on), a CLI version that does not exist.

The installer does not check the trusted comment, where the tag appears. So when a real CLI version with that number ships later, two different, authentically signed checksum files exist for the same asset name. An attacker able to replace release assets can then serve the doc-tag build under the CLI release, and the signature check passes.

**Assessment.** The trigger predates this branch. Signing is what turns it from a stray upload into an authenticated false claim.

**Fix:** narrow the tag filter to the CLI tag in `gen.py` and regenerate:
- toolkit: `mnemonic-toolkit-v*`
- ms: `ms-cli-v*`
- md: `descriptor-mnemonic-md-cli-v*`
- mk: `mk-cli-v*`

Alternatively, guard `assemble` with `startsWith(github.ref, 'refs/tags/<cli-prefix>')`. Separately, consider deleting the stray binaries from `manual-gui-v1.4.0`.

---

## Minor

### M1. No `concurrency:` group, and archive uploads stay `|| true`

**Concurrent runs.** A tag run and a backfill dispatch (`tag=<same>`) of `release.yml` can run at once. `gh release upload --clobber` deletes and then re-uploads each asset: `gh release upload --help` says "existing assets are deleted before new assets are uploaded. If the upload fails, the original assets will be lost."

So the two runs can interleave to leave `SHA256SUMS.portable` from run A beside `.minisig` from run B. Both runs report green, and the installer refuses (fail-closed).

**Silent archive failures.** `gh release upload "$TAG" release/<bin>-*.tar.gz --clobber 2>/dev/null || true` (and the same for `.zip`) can fail silently while the signed sums upload succeeds.
- On a re-run or backfill, the macOS and Windows builds are not reproducible.
- So a correctly signed sums file can then sit beside archives whose digests it does not list.
- CI is green and the installer refuses on digest.

**Fix:**
- Add `concurrency: { group: release-${{ github.event.inputs.tag || github.ref }}, cancel-in-progress: false }`.
- Drop `|| true` on the archive uploads.

### M2. Installer test gaps: component scope and the lower bound are unguarded

I ran 7 mutations of `scripts/install.sh`, each against a copy of `scripts/`, with application asserted by `cmp`. Each copy was run through `install-signature.test.sh` and through `install-verify.test.sh` in both modes.

| mutation | install-signature | install-verify (plain / FIRST_SIGNED=0.13.0) |
|---|---|---|
| `check_signature` call removed | **red**: cases 1–7, 10–13 | green / green |
| its return ignored (`\|\| true`) | **red**: 2 3 6 7 10 12 | green / green |
| `minisign -V` forced true | **red**: 2 3 12 13 | green / green |
| `first_signed` refusal disabled | **red**: 6 7 | green / green |
| component match ignored (`case "$_kc" in *)`) | **green (survives)** | green / green |
| lower version bound ignored | **green (survives)** | green / green |
| signature check moved after the digest compare | green (semantically inert: both still gate the install) | green / green |

**Findings from the table:**
- **Component scope.** If it broke, the retired key (`mnemonic-engrave||0.12.0|…`) would verify any mk, ms, md, toolkit or gui release ≤ 0.12.0 signed by it. That is exactly what the `signing_keys` comment says "can never" happen, and only a text grep of the shipped table guards it. Add a case: a key scoped to `mnemonic-engrave` signs the mk fixture, and the install must be refused.
- **Lower bound.** No shipped entry uses one yet, but the next rotation will. Add a case where the key is ranged `mk|999.0.0||` and the install is refused.
- **install-verify.** It never goes red on any signature mutation. That is by design (its stub signs every fixture on demand), but report 2's "the new run really verifies signatures" is not asserted by it. Only `install-signature.test.sh` gates signature behaviour.

**Baselines on the unmutated branch:**
- `install-signature.test.sh`: 14/14 ok (13 cases plus the shipped-table check).
- `install-verify.test.sh`: 47/47 plain, and 47/47 with `INSTALL_VERIFY_FIRST_SIGNED=0.13.0`.

### M3. No new-key signature can be verified outside CI (brief item 6 could not be run as specified)

- None of the proof runs uploads the signed checksum file or its `.minisig` as an artifact. I listed artifacts for engrave 36217201128, toolkit 36215620417 and 36215621875, and gui 36215630581: binaries only.
- No release carries a new-key signature yet.
- So nobody can yet run `minisign -V -P RWRUl0D…` against a real EF2B8D34D8409754 signature.

**Evidence available instead: the CI logs.**
- engrave 36217201128 "Verify SHA256SUMS.minisig against the pinned public key": `minisign -Vm SHA256SUMS -P RWRUl0DYNI0r72HYC0ou+T/7pHEf0km3a8RWHwqGwZmIEMWtiSd4k0B5` → `Signature and comment signature verified` / `Trusted comment: mnemonic-engrave sign-releases SHA256SUMS`.
- toolkit 36215621875: the same for `SHA256SUMS.x86_64` and `SHA256SUMS.aarch64`.

**What I verified myself:**
- Both pubkeys decode to the stated key ids: EF2B8D34D8409754 and CA39ECB257009A0F.
- engrave v0.12.0's real `SHA256SUMS.minisig` verifies with the retired key (`Trusted comment: mnemonic-engrave v0.12.0 SHA256SUMS`).
- The new key refuses it (`Signature key id … is CA39ECB257009A0F but the key id in the public key is EF2B8D34D8409754`, rc=1), so the retired-key range is right.

**Suggested:** have the dry-run path upload `release/SHA256SUMS*` as an artifact, so a rehearsal leaves something verifiable offline.

### M4. Engrave's key lives in four places with no consistency check

The pin is hard-coded in:
- `release.yml` (VERIFY.txt text and the verify step);
- `minisign.pub`, which ships inside every archive;
- the README.

Report 1 planned "a check that `minisign.pub` equals the pin". It is not in 209e26ec. Today all copies agree (counted: new key 2× `release.yml`, 1× `minisign.pub`, 2× README, 1× `gen.py`). A future rotation that misses one would ship a `minisign.pub` that disagrees with the verify step, and nothing would fail.

### M5. Tags on pre-merge commits run the old, unsigned workflows

A tag push runs the workflow files *at the tagged commit*. A tag on a commit that does not contain the merge (a maintenance branch, a late tag on an old commit) runs the pre-signing musl and GUI workflows: it uploads unsigned, with CI green.

- The old generated `release.yml` does sign when the secret exists, but verifies against the retired key, so it fails closed.
- The installer refuses such a release only once `first_signed` covers it (see I1).
- Classification: documentation only. The release checklist should say that tags must sit on a post-merge commit.

---

## Nit

- **N1.** The comments "The sums file and its signature go up in ONE call, so the release never carries a new SHA256SUMS… beside a stale signature" overstate it. gh deletes and then uploads per asset, so there is a window with the sums file and no signature. If the second upload fails, that becomes the persisted state (the job goes red). Reword it as "fails loudly; never leaves a stale pair".
- **N2.** GUI `build.yml` step `sha256sums` has no `pipefail`. A failing `sha256sum` inside `find | while … | sort` drops a line silently, and the smaller file is then signed. `[ -s SHA256SUMS ]` only checks that the file is non-empty. This fails closed at install time ("no checksum"). The problem predates this branch.
- **N3.** Every README and `verify-reproducibility.md` block uses `sha256sum -c`, including on the macOS and Windows rows, where that tool is usually absent. `shasum -a 256 -c` is the macOS spelling.
- **N4.** `--require-signature` does not cover the source-build fallback (a platform with no binary builds from the git tag with no signature). `--help` says so ("Source builds … are unaffected"). This is documentation only, noted because the flag's name suggests otherwise.

---

## Checked and found sound (so a re-review need not repeat it)

**Tag path vs dispatch path, all 9 workflows.**
- In each one, the condition that makes signing mandatory equals the condition under which the upload step runs:
  - toolkit/ms/md/mk `release.yml`: `publish`;
  - the musl callers: `IS_RELEASE` vs `startsWith(github.ref,'refs/tags/<cli>-v')`;
  - gui: `IS_RELEASE`;
  - engrave: `startsWith(github.ref,'refs/tags/v')`.
- Dispatch runs against a tag ref publish, and require the secret, consistently.
- `sign_dry_run` with `tag` publishes nothing.
- A missing secret on a publishing run exits 1 before any upload in that job.
- A failed sign exits 1 (engrave via `set -euo pipefail` on the pipe).
- Verify is a separate step that must succeed.
- No `continue-on-error` and no `|| true` on any sign, verify or sums/`.minisig` upload.
- The sums file and `.minisig` are uploaded in the same command in every `gh` workflow. gui and engrave publish them via softprops `files:`.
- Fork PRs never reach `assemble` or `musl-binaries`.
- Secrets are repo-level in all six repos (both names present, updated 03:29–03:30Z). No environment gates them; the toolkit's only environment is `github-pages`.

**Race.**
- Toolkit uploaders per tag family: `release.yml` → `SHA256SUMS.portable`; `man-pages.yml` → `SHA256SUMS.{x86_64,aarch64}`; the doc workflows → PDF/HTML only.
- No workflow regenerates or re-uploads another's checksum file.
- The only same-file race is concurrent runs of `release.yml` itself (M1).

**Reproducibility.**
- Signing is a host step after build+package.
- The `docker run --network=none` passes an explicit `-e` list with no MINISIGN variables.
- The cross step's env carries none, and the secrets are step-scoped to the sign step.
- The before/after `sha256sum` over tarball + sums + PROVENANCE is present and fails the job on any change.
- The reusable repro workflow is untouched and receives no secrets.

**Trust anchor.** Every verify step and every doc uses EF2B8D34D8409754. The retired key appears only in:
- engrave's README (the ≤ v0.12.0 note) and engrave `design/` history;
- toolkit `install.sh`'s ranged entry `mnemonic-engrave||0.12.0|…`, its test assertion, and CHANGELOG prose.

**Generator.**
- I regenerated all five outputs into scratch. toolkit, ms, md and mk are byte-identical (`cmp`) to their branch `release.yml`.
- The proof-run SHAs have zero workflow diff to the branch tips, except the toolkit, where the only difference is `rust.yml`.

**Installer ordering.** The signature is checked on the same local file (`$_w/$_sums`) that `_want` is read from, with no re-download, so there is no TOCTOU. A tampered sums file with the old `.minisig` is refused (case 2). A valid signature over a *different* file is the same event.

**Docs.**
- The README command sequence runs as written: throwaway-key sign, then `minisign -Vm SHA256SUMS.x86_64 -P <key>`, then `sha256sum -c … --ignore-missing` → OK.
- engrave's retired-key command verifies v0.12.0.
- File names in the tables match the workflows' `pack`, `sha256sum` and upload lines.

ready to merge: no (I1 and I2 are open; each fix is small: one gate direction in `install-assets.test.sh`, and one tag-filter change in `gen.py` plus regeneration).
