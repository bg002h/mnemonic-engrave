# Changelog

All notable changes to `mnemonic-engrave` (`me`) are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-06-16

### Added

- `me bundle --preview <dir>` (+ `--png`): faithful host-side SVG plate previews
  via the `me-preview` Go sidecar (pins SeedHammer v1.4.2 via a git submodule;
  replicates the SH2 engrave params + exact cubic-Bézier strokes). Renders only
  the public plates (md1 + mk1 chunks) — the ms1 secret plate is never rendered.
  The sidecar is version-checked against `me` (`me-preview --version` must match
  the crate version) and degrades gracefully when absent (manifest + checklist
  still emitted).
- Signed cross-platform release archives (minisign): `.github/workflows/release.yml`
  builds linux/macOS/windows `amd64` + linux/macOS `arm64` (windows/arm64
  unsupported), bundles `me` + `me-preview` + `minisign.pub` + `THIRD_PARTY_LICENSES`,
  and attaches a minisign-signed `SHA256SUMS`. See the README "Verifying releases"
  section.

## [0.2.0] - 2026-06-16

### Added

- `me bundle`: validates a wallet backup's public md1/mk1 strings, proves
  chunk-set integrity (catches dropped/reordered/duplicate/foreign chunks),
  emits a JSON manifest + guided plate checklist. Refuses ms1.

## [0.1.0]

### Added

- `me`: convert a single public md1/mk1 constellation string into an NFC NDEF
  payload for SeedHammer II. Refuses the secret ms1.
