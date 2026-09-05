# Changelog

All notable changes to `mnemonic-engrave` (`me`) are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.1] - 2026-09-04

### Added

- **Hashlock H0** (`SPEC_ms_hashlock` §9): a kind-`0x03` hashlock PREIMAGE
  plate is pinned as *never a seed record* on the host. `me sysw pack` now
  NAMES the kind — "is a hashlock PREIMAGE plate (kind 0x03), not a seed
  record" — instead of misdiagnosing it as "outside the profile" and telling
  the operator to re-encode a spend secret as entropy
  (`UnknownReason::PreimagePlate`, `seal::record::preimage_plate`). **`me seal`
  says the same thing**, via a new `RecordError::PreimagePlate` — before, the
  second verb stopped at the raw codec string `reserved-prefix byte was 0x03,
  expected 0x00`, with no kind name and, in a multi-record seal, no way to tell
  which record. H0 shipped
  against ms-codec 0.7, where the codec's prefix gate refused the string; H1b
  moved the refusal onto the codec's success path and the tripwire test
  `tests/preimage_plate_is_not_a_seed.rs` now asserts the named variant.
- Five rows in the shared seam corpus `testdata/codex32_seam_vectors.json`
  (now 13 rows: 2 both / 6 device-only / 5 neither), vendored byte-identical
  into the SeedHammer fork and pinned by sha256 in both suites:
  `preimage-plate-0x03`, `preimage-shape-entr-id` (the shape under the wrong
  id — the kind is the prefix byte, not the id), and two `device_admits: true`
  controls whose payloads begin `0x03`, `bip93-plain-payload-0x03` and
  `bip93-share-payload-0x03`, which the device guard must NOT touch. The fifth,
  `bip93-plain-33-byte-payload-0x03`, pins the deliberate COLLISION: a plain
  BIP-93 secret whose 33-byte seed begins `0x03` is indistinguishable from a
  preimage plate and is refused on both sides (`me` refuses it too, so this is
  convergence). 16-, 20-, 24-, 28- and 32-byte seeds and every share are
  untouched. The S2 pre-capture `testdata/record_corpus_pre_s2.json` grows
  33 → 38 with the same records, all `Unknown`.
- **Hashlock H1b** (`SPEC_ms_hashlock` §9, follow-up F-473): the `ms-codec`
  pin moves `0.7` → `0.8`, so `me` reads the RELEASED hashlock wire. At `0.8`
  the codec DECODES a kind-`0x03` string as `Payload::Preimage`, so the refusal
  H0 shipped is now on the codec's **success** path rather than an accident of
  the old pin: `validate_record` answers `RecordError::PreimagePlate` for a
  decoded preimage and never `RecordKind::Ms`. `Payload` is `#[non_exhaustive]`,
  and the wildcard arm **refuses** — a payload kind a future `ms-codec` minor
  adds cannot be placed as a seed until `me` has decided what it is, and the
  compiler cannot warn about it. `seal::record::preimage_plate` is now
  pin-independent and keyed on the device's own SHAPE (an `ms`-HRP, unshared,
  codex32-valid single with a 33-byte payload whose first byte is `0x03`, the
  same test as the fork's `codex32.IsPreimage`), so a `0x03` single under any
  id — or with a wrong X length the codec can name (`PreimageLengthMismatch`,
  which it reaches only when the string length sits in the profile's length
  sets, i.e. X ∈ {16, 17, 20, 21, 24, 25, 28, 29, 32, 33}) — is named a preimage
  plate on both host verbs rather than falling through to "outside the
  profile". Any other X (18, 19, 22, …) is outside the profile's length sets and
  is refused as exactly that (post-impl M-1).
- An **id/kind mismatch** (`SPEC_ms_hashlock` §1 rule 2, ruling L24) — an `ms1`
  single whose 4-character id and kind byte disagree — is diagnosed as what it
  is on both verbs (`UnknownReason::TagKindMismatch`,
  `RecordError::TagKindMismatch`, `seal::record::id_kind_mismatch`) instead of
  as "outside the profile". It is refused and never read by either field: a
  damaged or forged plate is re-encoded from the source, not edited.

### Changed

- `me sysw pack`: a `key:` record whose origin path has a `+`-signed component
  (`[fp/+48'/0'/0'/2']xpub…`) is now refused like any other malformed path.
  rust-bitcoin's path parser tolerated the sign; the SeedHammer device's does
  not, and the lockstep fixture (`testdata/record_class_vectors.json`, now 47
  rows) pins both that and an unhardened component of 2^31 on both sides
  (composer Stage 2 whole-diff review, C-1/I-1).

## [0.8.0] - 2026-09-02

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
