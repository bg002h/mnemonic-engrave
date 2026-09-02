# Changelog

All notable changes to `mnemonic-engrave` (`me`) are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `me sysw pack`: three new record classes for the SeedHammer II composer —
  `key:` (a cosigner `[fingerprint/path]xpub`), `hash:` (a 32-byte sha256
  digest), `now:` (the pack time and optional height). Bodies are lowercase
  hex; a malformed body is refused with its own line, before any passphrase is
  printed. At most one `now:` per payload. When the records include a `key:`
  or `hash:` record and no `now:`, `pack` appends the pack time as a trailing
  `now:`; `--now` forces that onto any payload, `--no-now` suppresses it, and
  a supplied `now:` always wins — payloads without a composer record pack
  byte-identically to before. A sealed payload that gets a `now:` therefore
  carries its pack time in cleartext: the class is public by design, because
  the device reads the bound before any passphrase. `me sysw show` prints the
  three.
  `testdata/record_class_vectors.json` is the lockstep fixture (45 rows, one
  per §6a rule) the device's classifier is measured against (composer spec §12
  item 8).

- **The descriptor seam (S1+S3).** `me sysw pack --as md1` reads a wallet
  descriptor — a concrete descriptor, a BlueWallet `Key: value` export, a
  JSON `{"descriptor": …}` wrapper, or a bare extended key — as one whole
  document from `--in`/stdin, admits it through the §4 cascade measured
  against the SeedHammer device's own parser plus the §4.7 eight-conjunct
  predicate, and packs the BIP-388 decomposition as md1 text cards. Before
  packing it prints the §5.4 identification block: wallet id, receive
  address 0 (derived key-by-key, verified against an independent BIP-32
  oracle and the device on 91 constructed wallets), and a watch-only owner
  line. Every refusal is one of §6's rows, each with a named test
  asserting its text; `--as descriptor` shipped parked behind the window
  refusal (un-parked by the S2 entry below). The vector file is pinned
  byte-identical in the fork, and the cross-language gate runs for real
  under `ME_REQUIRE_GO=1`.
- `me`'s top-level NDEF converter now refers descriptor-shaped input to
  `me sysw pack --as <descriptor|md1>` instead of dead-ending (F-421).

- **`--as descriptor` end to end (S2, F-418).** `me sysw pack --as
  descriptor` packs §5.2's canonical re-encoded descriptor as ONE
  `Descriptor` record — admission first, the `multi` refusal permanent,
  the §5.4 identification block printed on every path. The pack path's
  §5.1 gate now keys on IDENTIFICATION rather than classification
  failure, so the choice block survives the new classifier arm;
  `sysw::classify` gains the `Descriptor` arm (delegating to the §5.2
  predicate, `host_admits`) and `--expect descriptor` accepts either
  carrier. `me sysw show` reports each `Descriptor` record with the
  §5.4 block (previously silent). §6's table is 35 rows post-S2 (the
  window row retired with the build state that produced it); the vector
  file regenerated ONCE — 72 rows, the `sysw_class` sample column
  replaced by an exhaustive derived classification rule asserted in
  both languages, sha `e7a4160c…` pinned byte-identical in the fork.
  Device side (fork `s2/descriptor-arm`): `sysw.Classify` gains the
  same predicate (§4 cascade narrowings + §4.7 conjuncts, ASCII-edge
  parity, key-material-scoped version check; 187-case parity probe, 0
  divergences), Wallet Policy consumes a `Descriptor` record straight
  to the descriptor screen (first execution of that admission cell,
  sim-walked), the short-fingerprint parse panic is fixed as Rust
  convergence, and F-426's `ypub` scan-door case lands (the sysw
  classifier stays host-exact; host widening is F-426's later cycle).
  F-423: bundle plates PACK — a card's strings share plates up to the
  measured fit (keyed single-sig 2→1, full 2-of-3 build 9→4; packed
  plates are TEXT ONLY, F-433).

### Acceptance (spec §11)

Items 1-5 are discharged by the suites (item 4's rows at the post-S2
count of 35; item 5's five-case matrix on the full-build truth table).
**Item 6 discharged 2026-08-29 on hardware**: firmware `bga0c1615`
flashed, a `--as descriptor` payload loaded and digest-authenticated,
and the Engrave Descriptor screen displayed the correct wallet with
the correct receive address, confirmed by the operator's eyes. F-423's
physical test plate remains its own follow-up.

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
