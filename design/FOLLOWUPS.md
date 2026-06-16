# FOLLOWUPS — mnemonic-engrave

Low/nit items deferred from architect reviews (per the iterative-architect-review standard: Critical/Important fixed inline; low/nit recorded here). Promote to a cycle when convenient.

## Open

> These are **cycle-sized** items (bigger than architect-review nits) — each warrants its own brainstorm → spec → plan → R0 → implement pass when picked up.

- **`me-crates-io-publish`** — Publish `me` to crates.io. First: verify the bin/crate name is free (`cargo search me` / crates.io); if `me` is taken, ship as `mnemonic-engrave` with `me` as a local alias only. Then a first versioned release (0.1.0/0.1.1) with a `CHANGELOG.md`, the `[package]` metadata (`repository`, `homepage`, `keywords`, `categories`), and `cargo publish --dry-run` green. Depends on the published `md-codec 0.36` / `mk-codec 0.4`. SemVer: initial release.

- **`me-bundle-preview-layer`** — The deferred host-side **bundle orchestration** (v1 non-goal in `design/SPEC_seedhammer_engrave.md` §2). A wallet backup = a *set* of plates: `md1` policy + `mk1` xpub chunk(s) + `ms1` secret (typed on-device, never via the tool). Build a manifest + guided per-plate workflow ("plate 1/N: md1 — push via NFC & engrave; … ms1 — type on device") and optionally a faithful plate preview (could reuse SeedHammer's Go `engrave`/`backup` libs host-side). Larger feature; its own spec→plan→R0 cycle. Honors the per-string model (a multi-chunk `mk1` = multiple plates).

- **`firmware-deferred-formal-reviews`** — Run the formal opus-architect **subagent** reviews that were done as *inline self-reviews* because Agent-API dispatch was failing (500/529) for the back half of the 2026-06-16 session: (a) PR2 (#35) final whole-diff review (`codex32/mdmk.go` + gui scanner/engrave); (b) the converter-polish plan-R0 (`design/agent-reports/me-converter-polish-plan-R0-selfreview.md`). Do once subagent dispatch recovers; fold any C/I, persist verbatim to `design/agent-reports/`.

- **`seedhammer-upstream-prs-tracking`** — Track the two open upstream PRs to `seedhammer/seedhammer`: **#34** (re-enable on-device CODEX32 entry) and **#35** (BCH-validated md1/mk1 engraving). Respond to maintainer feedback; mirror any requested changes back. **If declined or stalled:** pursue the fork-fallback — stand up a `seedhammer-fork` sibling repo and document the "Set custom boot key" path (program a 2nd RP2350 OTP boot-key slot via picotool to run own-signed firmware on a locked SH2; "Advanced · irreversible" — per https://gangleri42.github.io/seedhammer/).

## Resolved

### Converter (`me`) polish cycle — RESOLVED 2026-06-16 (commit `5086119`)
All five nits from the converter execution review (`design/agent-reports/me-converter-execution-review.md`) were cleared in one PATCH cycle (spec `design/SPEC_me_converter_polish.md`, plan `design/IMPLEMENTATION_PLAN_me_converter_polish.md`):

- **`me-in-stdin-intermediate-zeroize`** — input now read into a `Zeroizing<String>`, scrubbed on drop (`main.rs`).
- **`me-validate-ms-unreachable`** — `panic!` → `unreachable!("ms1 is refused before validation")` (`validate.rs`).
- **`me-decode-text-tlv-comment`** — `decode_text_tlv` now documents its intentional 1-byte-TLV / no-terminator-check scope (`ndef.rs`).
- **`me-canonical-string-stderr`** — reconciled via an opt-in `--echo` flag (prints the validated string to stderr on success); spec §5 amended to match (`main.rs`, `cli.rs`, `SPEC_seedhammer_engrave.md`).
- **`me-go-harness-shortread-loop`** — the harness now reads the NDEF record in a short-read loop (`firmware/ndef-roundtrip/main.go`).
