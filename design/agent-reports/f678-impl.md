# F-678 implementer report — md x86_64 musl release leg

Author: implementer agent (Opus 5.5), 2026-09-24.
Repo: descriptor-mnemonic. Worktree: `/scratch/code/shibboleth/dm-worktrees/f678`.
Branch: `f678-musl` (pushed). Fix commit: `d2c8488a`, based on `origin/main` `0e5f31d9`.
Gate run: **36086241898** (workflow_dispatch, no tag). Every job is green except `repro-aarch64-musl`, which is skipped. The reason is under Q2.
Not done: no tag, no release upload, no merge, no push to main.

## Q1 — root cause

The jobs are in `.github/workflows/man-pages.yml` (confirmed): `resolve-miniscript-rev`, the `repro` caller (toolkit reusable workflow @`6e37b18e`), and the `musl-binaries` matrix.

**The defect.** `repro` passes `miniscript_rev` (read from Cargo.lock), so the toolkit gate builds with the **three-block** `[source]` config: crates-io, the miniscript git fork, and vendored-sources. The `musl-binaries` legs hard-coded the **two-block** form, which has no redirect for the git source. md pinned `miniscript` to a git rev in 5b4d20ad (2026-08-20). Commit `0ce18660` then fixed only the `repro` job, and the musl-binary legs were missed. The comments still read "md is fork-free → NO git-fork block". This is why `repro-x86_64-musl` passes and `musl-binary (x86_64)` fails.

**Why aarch64 went green anyway.** It was not built from vendor/. Before building, cross v0.2.5 runs a host-side `cargo metadata --format-version 1` with no `--offline`, no `--config` and no `--no-deps` (`cross-0.2.5/src/cargo.rs`, `cargo_metadata_with_args`; read from the crates.io tarball). That call fetches the git dependency over the network into the host `CARGO_HOME`, and cross mounts that directory into its container (`docker/shared.rs`: `CARGO_HOME=/cargo`). The inner `--offline` build then found `miniscript` in `/cargo/git/checkouts`. This is an inference from the mechanism: a two-block `--offline` build has nowhere else to get the git source. I did not observe it directly. The `Compiling miniscript v13.0.0 (https://…#ff4732e5)` line in the 0.20.3 log is **not** evidence either way, because the vendored build prints the same git URL (seen in run 36086241898).

**Local reproduction** (same failure, then the fix). I could not use the repro container digest locally: my user is not in the `docker` group, sudo needs a password, and the gh token lacks `read:packages`. I used the closest substitute: `unshare -rn` (only `lo`; `curl github.com` → exit 7), an empty `CARGO_HOME`, and the rustc 1.85.0 toolchain binaries invoked directly.
- Before, with the CI command verbatim (two-block): `error: failed to load source for dependency \`miniscript\` … can't checkout from 'https://github.com/rust-bitcoin/rust-miniscript': you are in the offline mode (--offline)`, EXIT=101.
- After, with the step body extracted from the edited YAML and fed the three-block args file: `Finished release`, EXIT=0, and a static-pie x86_64 binary. Afterwards the clean `CARGO_HOME` contains no `git/` directory. The dep-info for `miniscript-*.d` lists only `…/vendor/miniscript/src/*.rs`. (The local build used `CC_x86_64_unknown_linux_musl=gcc` because this box has no musl-gcc. That does not affect source resolution.)

## Q2 — why `repro-aarch64-musl` is skipped

The caller passes `run_aarch64: false`, and the reusable workflow gates that job on `if: ${{ inputs.run_aarch64 }}` (toolkit `6e37b18e`, reproducible-musl-build.yml line 547). This is deliberate and commented, because the QEMU byte-gate takes about 30–60 min. **Consequence: md's published aarch64 musl artifact is never byte-gated for reproducibility**, on a tag or on a dispatch. The only route is a manual dispatch of the toolkit workflow.

**Not fixed.** Flipping the flag is one line, but it adds 30–60 min to every release and every dispatch. Also, F-675 already reports that the toolkit's aarch64 remap-off negative check finds zero `/project` leakage, so this gate may be unable to fail. Turning it on would add a green of unknown meaning. This is a controller decision.

**A second, larger gap (fixed).** The `musl-binaries` job was tag-only. The workflow_dispatch "reproducibility gate" therefore never built either musl-binary leg, which is why F-678 was invisible until release day, twice.

## Q3 — the fix (`d2c8488a`, files: `.github/workflows/man-pages.yml`, `Cross.toml`)

- **`prep` step.** It derives the three-block config from `MINISCRIPT_REV` (the `resolve-miniscript-rev` output, now in `needs`) and writes one argument per line to `$RUNNER_TEMP/src-config.args`. Both legs `mapfile` that single file. The x86_64 leg passes the args into the container as `bash -c '…' bash "${SRC_CONFIG[@]}"`. The step fails closed on an empty rev, and on any `source = "git+…"` in Cargo.lock that the config does not redirect. It also sets `VER`: the tag version, or `dispatch-<sha12>` on a dispatch.
- **`vendored-miniscript` step (both legs).** It requires every `target/$TARGET/release/deps/miniscript-*.d` to reference `/vendor/miniscript/src/lib.rs`, and requires that no `.d` file references `/git/checkouts/`. This turns "built from vendor/" into a measurement. Without it, the aarch64 leg would stay green regardless, because of the host fetch.
- The job-level `if` now also admits `workflow_dispatch`. `ensure-release` and `upload` are tag-only.
- I corrected the comments that the change made false: the man-pages header, the musl-binaries block, and the Cross.toml "clean crates.io graph / NO git-fork stanza" note.

**Remaining limitation.** cross's host `cargo metadata` still hits the network on the aarch64 leg. The new step proves the compile no longer uses that fetch, but the host fetch itself still happens.

## Gates executed

| gate | command | result |
|---|---|---|
| actionlint | `actionlint .github/workflows/man-pages.yml` | clean |
| prep script, 4 cases (extracted from YAML) | branch ref / tag ref / `MINISCRIPT_REV=` / wrong rev | args file written, `VER=dispatch-0e5f31d95b42` / `VER=0.20.4` / rc=1 "empty" / rc=1 "git source … does not redirect" |
| local offline repro, before | `unshare -rn` + empty CARGO_HOME + CI two-block command | EXIT=101, `failed to load source for dependency miniscript` |
| local offline repro, after | the same, with the extracted x86_64 step body + args file | EXIT=0; tarball + SHA256SUMS produced; check step `OK` |
| check-step negatives | fake `.d` pointing at `/cargo/git/checkouts/…`; empty deps dir | rc=1 / rc=1 with the expected errors |
| CI dispatch | `gh workflow run man-pages.yml --ref f678-musl` → **run 36086241898** | success; see below |

Run 36086241898 job conclusions: `resolve-miniscript-rev` success, `man-pages` success (build steps guarded off), `build-container` success, `repro-x86_64-musl` success, `repro-substrate` success, **`musl-binary (x86_64)` success**, **`musl-binary (aarch64)` success**, `repro-aarch64-musl` skipped (`run_aarch64: false`, see Q2). Both legs printed `OK: miniscript compiled from vendor/ (1 dep-info file(s))`, and `ensure-release`/`upload` were `skipped` on both.
**The x86_64 leg's tarball sha256 `69c6cb2e96b5aa6bb551919482d478bb89acaa6439c5aaa846548732c062fdca` equals the hash `repro-x86_64-musl` measured** (`md-0.0.0-repro-x86_64-linux-musl.tar.gz`) in the same run. The published artifact is byte-identical to the gated one. aarch64 sha256: `95481f19…88f96`.

## Backfill for 0.20.3 (not executed — controller decision)

The 0.20.3 tag commit is `0e5f31d9`, the base of this branch. Cargo.lock and vendor/ are identical to the branch. Re-running run 36084793496 would not help, because it would run the unfixed workflow from the tag. A dispatch artifact is not the 0.20.3 artifact either: tar `--mtime` and `SOURCE_DATE_EPOCH` come from HEAD's commit time, which differs. So the build has to be done at the tag commit with the fixed recipe. It needs a host with docker and GHCR `read:packages` (this box has neither for my user):

```bash
git clone https://github.com/bg002h/descriptor-mnemonic md-bf && cd md-bf
git checkout descriptor-mnemonic-md-cli-v0.20.3            # 0e5f31d9
IMG=ghcr.io/bg002h/repro-musl-descriptor-mnemonic@sha256:3ffeeebd067b08092bb40883f82b1dbc6040d095ecc0bd89157118dacc3ccef5  # 0.20.3 run's image
R=ff4732e5f75aa555682343cb180fa72ee3e8e9d5; U="git+https://github.com/rust-bitcoin/rust-miniscript?rev=$R"
EPOCH=$(git show -s --format=%ct HEAD); M="-ffile-prefix-map=/build/src=/build -ffile-prefix-map=/cargo=/cargo"
docker run --rm --network=none -v "$PWD":/build/src -w /build/src \
  -e CARGO_HOME=/cargo -e RUSTUP_TOOLCHAIN=1.85.0 -e SOURCE_DATE_EPOCH="$EPOCH" -e LC_ALL=C -e TZ=UTC \
  -e CARGO_BUILD_RUSTFLAGS="--remap-path-prefix=/build/src=/build --remap-path-prefix=/cargo=/cargo" \
  -e CFLAGS="$M" -e CFLAGS_x86_64_unknown_linux_musl="$M" "$IMG" \
  bash -euo pipefail -c 'umask 022
    cargo build --locked --offline --release --target x86_64-unknown-linux-musl -p md-cli --bin md "$@"
    tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$SOURCE_DATE_EPOCH" \
      -cf - -C target/x86_64-unknown-linux-musl/release md | gzip -n -9 > md-0.20.3-x86_64-linux-musl.tar.gz
    sha256sum md-0.20.3-x86_64-linux-musl.tar.gz > SHA256SUMS.x86_64' bash \
  --config 'source.crates-io.replace-with="vendored-sources"' \
  --config "source.\"$U\".git=\"https://github.com/rust-bitcoin/rust-miniscript\"" \
  --config "source.\"$U\".rev=\"$R\"" \
  --config "source.\"$U\".replace-with=\"vendored-sources\"" \
  --config 'source.vendored-sources.directory="vendor"'
{ echo "artifact: md-0.20.3-x86_64-linux-musl.tar.gz"; echo "sha256:  $(cut -d' ' -f1 < SHA256SUMS.x86_64)"
  echo "source_commit: $(git rev-parse HEAD)"; echo "source_date_epoch: $EPOCH"; echo "container_image: $IMG"; } > PROVENANCE.x86_64.txt
gh release upload descriptor-mnemonic-md-cli-v0.20.3 --repo bg002h/descriptor-mnemonic \
  md-0.20.3-x86_64-linux-musl.tar.gz SHA256SUMS.x86_64 PROVENANCE.x86_64.txt
```

Alternatively, you could add a `release_tag` input to the dispatch path that checks out the tag and uploads. That is a larger change, and I did not make it.

## Concerns

1. **The shipped 0.20.3 aarch64 asset (and 0.15.0–0.20.2) was built from the host network fetch, not vendor/.** The source content is the same pinned git commit, so the binary is likely identical. Still, its provenance claim ("offline, committed vendor/") was false. Rebuilding it is optional and a controller call.
2. **Stale comments I left alone because they are out of scope:** the `vendor-freshness.yml` header ("this crate is fork-free … no miniscript git-fork stanza") and the `ci/repro/vendor-freshness.sh` header (lines 18–21) both contradict the three-block body of the script.
3. **The aarch64 repro byte-gate never runs for md** (Q2). The F-675 finding that its negative check may be unable to fail applies here too.
4. Dispatch runs now also build both musl legs, which adds about 3–4 min, and on dispatch they produce artifacts nobody uploads. That is intended.
5. The local reproduction used `unshare -rn` rather than the container digest, for the reasons given in Q1. The in-container proof is CI run 36086241898.
