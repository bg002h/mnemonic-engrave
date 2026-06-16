# Changelog

All notable changes to `mnemonic-engrave` (`me`) are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-06-16

### Added

- `me bundle`: validates a wallet backup's public md1/mk1 strings, proves
  chunk-set integrity (catches dropped/reordered/duplicate/foreign chunks),
  emits a JSON manifest + guided plate checklist. Refuses ms1.

## [0.1.0]

### Added

- `me`: convert a single public md1/mk1 constellation string into an NFC NDEF
  payload for SeedHammer II. Refuses the secret ms1.
