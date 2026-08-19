# Changelog

All notable changes to `mnemonic-engrave` (`me`) are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2026-08-19

### Added

- `me bundle`'s checklist now **names the mk1 card each chunk plate belongs to**,
  so an operator cutting a 34-plate bundle can tell whose key is in front of
  them: `mk1 [aabbccdd/48'/0'/0'/2'] chunk 1/3`. Plates are emitted in
  `chunk_set_id` order, which is neither the order the cards were supplied in
  nor the order of the template's `@N` indices, so the bracket was previously
  the only missing link between a plate and a cosigner.
  - A `--privacy-preserving` card has no fingerprint by design and renders
    `[path 48'/0'/0'/2', no fingerprint]` — never a fabricated one.
  - Two *different* cards sharing one `(fingerprint, path)` — routine among
    privacy-preserving cosigners, who commonly all sit at the same path — take a
    trailing ` set 0x…` so their otherwise identical plates stay distinguishable.
    The scan is **per card**, not per plate.
- Manifest plates gained optional `card_fingerprint` / `card_path`, present on
  `mk1-chunk` plates only. Consumers of the Phase A schema are unaffected: both
  are omitted when absent.
- `scripts/build-preview.sh` builds the `me-preview` sidecar at the exact crate
  version, with `--check` to detect drift. The `-X main.version=` link flag
  previously existed only in release CI, so every version bump silently broke
  `me bundle --preview` from a checkout, and the fix was undiscoverable.

### Note

Releases 0.4.0, 0.5.0, 0.5.1 and 0.6.0 shipped without changelog entries; this
file jumps from 0.3.0 to 0.7.0. Their contents are recoverable from the git log
between the corresponding tags.

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
