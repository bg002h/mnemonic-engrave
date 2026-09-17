#!/usr/bin/env python3
"""Emit a release workflow for one constellation CLI repo.

One generator so the four files cannot drift. Everything repo-specific is a
parameter; the matrix and the build steps are byte-identical everywhere, and
they are the ones mnemonic-engrave already proves for this dependency tree.
"""
import sys

TARGETS = {
    "linux-x86_64":   ("ubuntu-latest",  "x86_64-unknown-linux-gnu",  "false", ""),
    "linux-aarch64":  ("ubuntu-latest",  "aarch64-unknown-linux-gnu", "true",  ""),
    "macos-x86_64":   ("macos-latest",   "x86_64-apple-darwin",       "false", ""),
    "macos-aarch64":  ("macos-latest",   "aarch64-apple-darwin",      "false", ""),
    "windows-x86_64": ("windows-latest", "x86_64-pc-windows-msvc",    "false", ".exe"),
}
ARCHIVE = {
    "linux-x86_64": ("linux", "amd64"), "linux-aarch64": ("linux", "arm64"),
    "macos-x86_64": ("macos", "amd64"), "macos-aarch64": ("macos", "arm64"),
    "windows-x86_64": ("windows", "amd64"),
}

def gen(*, bin_, pkg, branch, names, why_subset, smoke):
    matrix = "".join(
        f"""          - name: {n}
            os: {TARGETS[n][0]}
            target: {TARGETS[n][1]}
            use_cross: {TARGETS[n][2]}
"""
        for n in names
    )
    packs = "\n".join(
        f'          pack {TARGETS[n][1]:<28} {ARCHIVE[n][0]:<7} {ARCHIVE[n][1]:<5} "{TARGETS[n][3]}"'
        for n in names
    )
    return f"""# Release: build `{bin_}` for the platforms this repo does not already ship,
# then attach them to the tag's GitHub release.
#
# MODELLED ON `mnemonic-engrave/.github/workflows/release.yml`. That matrix is
# already proven for this dependency tree, and `descriptor-mnemonic` now runs the
# same one green on all five targets.
#
{why_subset}#
# COMPOSES with the other tag-triggered workflows in this repo rather than
# competing with them: it uses the same `gh release view || gh release create`
# then `gh release upload --clobber` idiom, so whichever workflow reaches the tag
# first creates the release and the rest attach their own assets.
#
# UNSIGNED FOR NOW (operator decision). Only `mnemonic-engrave` carries
# MINISIGN_SECRET_KEY. SHA256SUMS ships anyway, and VERIFY.txt says plainly that
# a checksum proves integrity and NOT origin. Add the secret and signing starts
# with no change here.
name: release

on:
  push:
    tags:
      - '*v[0-9]*'
    branches:
      - {branch}
      - 'ci/**'
  pull_request:
  workflow_dispatch:
    inputs:
      tag:
        description: 'Existing tag to build and BACKFILL binaries onto. Empty = just exercise the matrix.'
        required: false
        default: ''

permissions:
  contents: write
  id-token: write
  attestations: write

env:
  RUST_TOOLCHAIN: '1.85.0'
  FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: 'true'

jobs:
  test:
    name: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{{{ github.event.inputs.tag || github.ref }}}}
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{{{ env.RUST_TOOLCHAIN }}}}
      - name: cargo test
        run: cargo test --locked --workspace

  build:
    name: build {bin_} (${{{{ matrix.name }}}})
    runs-on: ${{{{ matrix.os }}}}
    strategy:
      fail-fast: false
      matrix:
        include:
{matrix}    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{{{ github.event.inputs.tag || github.ref }}}}

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{{{ env.RUST_TOOLCHAIN }}}}
          targets: ${{{{ matrix.target }}}}

      - name: Install cross
        if: matrix.use_cross
        run: cargo install cross --locked --version 0.2.5

      - name: Build (cross)
        if: matrix.use_cross
        env:
          RUSTFLAGS: --remap-path-prefix=${{{{ github.workspace }}}}=.
        run: cross build --release --locked --target ${{{{ matrix.target }}}} -p {pkg}

      - name: Build (cargo)
        if: ${{{{ !matrix.use_cross }}}}
        env:
          RUSTFLAGS: --remap-path-prefix=${{{{ github.workspace }}}}=.
        run: cargo build --release --locked --target ${{{{ matrix.target }}}} -p {pkg}

      - name: Smoke-test the binary
        # Only where the runner can execute what it just built. A cross build for
        # aarch64 cannot run on an x86_64 runner, and saying so beats a green job
        # that never ran its own artifact.
        if: ${{{{ !matrix.use_cross }}}}
        shell: bash
        run: |
          set -euo pipefail
          BIN="target/${{{{ matrix.target }}}}/release/{bin_}"
          [ "${{{{ runner.os }}}}" = "Windows" ] && BIN="$BIN.exe"
          "$BIN" --version
{smoke}
      - name: Stage binary
        shell: bash
        run: |
          set -euo pipefail
          mkdir -p dist
          if [ "${{{{ runner.os }}}}" = "Windows" ]; then
            cp "target/${{{{ matrix.target }}}}/release/{bin_}.exe" "dist/{bin_}-${{{{ matrix.target }}}}.exe"
          else
            cp "target/${{{{ matrix.target }}}}/release/{bin_}" "dist/{bin_}-${{{{ matrix.target }}}}"
          fi

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: {bin_}-${{{{ matrix.target }}}}
          path: dist/{bin_}-*
          if-no-files-found: error

  assemble:
    name: assemble + release
    needs: [test, build]
    if: startsWith(github.ref, 'refs/tags/') || github.event.inputs.tag != ''
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{{{ github.event.inputs.tag || github.ref }}}}

      - name: Resolve tag
        id: meta
        run: |
          set -euo pipefail
          TAG="${{{{ github.event.inputs.tag }}}}"
          [ -n "$TAG" ] || TAG="${{GITHUB_REF_NAME}}"
          echo "tag=$TAG" >> "$GITHUB_OUTPUT"

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Assemble archives
        run: |
          set -euo pipefail
          TAG="${{{{ steps.meta.outputs.tag }}}}"
          mkdir -p release
          cat > VERIFY.txt <<'EOF'
          Verifying this release
          ======================

          These binaries are NOT signed. Verify them against the checksums:

              sha256sum -c SHA256SUMS --ignore-missing

          A checksum file alone proves INTEGRITY, not ORIGIN. Signed releases
          are tracked separately.
          EOF

          pack() {{
            local target="$1" a_os="$2" a_arch="$3" ext="$4"
            local stage="stage-${{a_os}}-${{a_arch}}"
            mkdir -p "$stage"
            cp "artifacts/{bin_}-${{target}}/{bin_}-${{target}}${{ext}}" "$stage/{bin_}${{ext}}"
            cp VERIFY.txt "$stage/"
            [ -f LICENSE ] && cp LICENSE "$stage/" || true
            if [ "$ext" = ".exe" ]; then
              (cd "$stage" && zip -q "../release/{bin_}-${{TAG}}-${{a_os}}-${{a_arch}}.zip" ./*)
            else
              chmod +x "$stage/{bin_}"
              tar -czf "release/{bin_}-${{TAG}}-${{a_os}}-${{a_arch}}.tar.gz" -C "$stage" .
            fi
          }}

{packs}

          ls -l release

      - name: Write SHA256SUMS
        working-directory: release
        run: |
          set -euo pipefail
          # Named for the platform set THIS workflow produces, so it cannot race
          # the per-arch checksum files the musl workflow uploads to the same tag.
          sha256sum {bin_}-* > SHA256SUMS.portable
          cat SHA256SUMS.portable

      - name: Attest build provenance
        uses: actions/attest-build-provenance@v1
        with:
          subject-path: 'release/{bin_}-*'

      - name: Ensure a release exists, then upload
        env:
          GH_TOKEN: ${{{{ github.token }}}}
          TAG: ${{{{ steps.meta.outputs.tag }}}}
        run: |
          set -euo pipefail
          gh release view "$TAG" >/dev/null 2>&1 || \
            gh release create "$TAG" --title "$TAG" --generate-notes || true
          gh release upload "$TAG" release/{bin_}-*.tar.gz --clobber 2>/dev/null || true
          gh release upload "$TAG" release/{bin_}-*.zip --clobber 2>/dev/null || true
          gh release upload "$TAG" release/SHA256SUMS.portable --clobber
          gh release view "$TAG" --json assets -q '.assets[].name'
"""

if __name__ == "__main__":
    pass
