# FOLLOWUPS — mnemonic-engrave

Low/nit items deferred from architect reviews (per the iterative-architect-review standard: Critical/Important fixed inline; low/nit recorded here). Promote to a cycle when convenient.

## Open

> These are **cycle-sized** items (bigger than architect-review nits) — each warrants its own brainstorm → spec → plan → R0 → implement pass when picked up.

- **`me-bundle-preview-sidecar`** (Phase B → v0.3.0) — The deferred faithful host-side **plate preview**, split out of `me-bundle-preview-layer` (Phase A shipped the pure-Rust orchestration core; see Resolved). A `me-preview` (Go) sidecar renders ONLY a validated public string + plate mode → `engrave.Engraving` → image; `me` (Rust) does all validation; the sidecar has no secrets and no network. Carries `DESIGN_me_bundle_preview.md` §B in full:
  - **B1 — sidecar & trust split / upstream pin:** `go.mod` pins **UPSTREAM seedhammer v1.4.2** (`backup.EngraveText`/`backup.Text`/`backup.Paragraph`/`font/sh`/`engrave.Params` are all pre-existing upstream; the sidecar imports `backup`+`engrave` directly, NOT `gui`, so it is **NOT blocked on PR #35**). Mirrors the `firmware/ndef-roundtrip/` replace pattern.
  - **B2 — faithfulness contract (resolves R0 I-3 + m-5):** replicate `validateMdmk`'s exact layout — `backup.EngraveText`, QR via **`qr.Encode(s, qr.L)`** (error-correction level **L**, not M), **`qrScale = 3`**, modes TEXT+QR / TEXT / QR-only (any deviation makes the preview QR differ from the engraved QR). SVG primary (walk the `Command` stream via exported `AsKnot()`/`AsDelay()`); **B-spline `ControlPoint` knots (multiplicity ≠ 3) must be interpolated, not drawn as line segments** (m-5), or fonts mis-render — the Phase-B spec must declare the fidelity target (exact B-spline vs documented-approximate). Optional `--png`.
  - **B3 — delivery & version binding (resolves R0 I-4):** bundled per-platform signed release archive (`me` + `me-preview` + `SHA256SUMS` + signature); cross-platform CI matrix; no runtime network. `me` locates `me-preview` beside itself / on `$PATH` and **checks `me-preview --version` against the expected pin before invoking it** — mismatch → clear warning/refusal (never a silent stale-layout render); absent with `--preview` → graceful degrade (manifest+checklist still emitted).
  - Gets its own brainstorm → spec → plan → R0 cycle when picked up. UX enhancement, not a safety feature.

- **`seedhammer-upstream-prs-tracking`** — Track the two open upstream PRs to `seedhammer/seedhammer`: **#34** (re-enable on-device CODEX32 entry) and **#35** (BCH-validated md1/mk1 engraving). Respond to maintainer feedback; mirror any requested changes back. **If declined or stalled:** pursue the fork-fallback — stand up a `seedhammer-fork` sibling repo and document the "Set custom boot key" path (program a 2nd RP2350 OTP boot-key slot via picotool to run own-signed firmware on a locked SH2; "Advanced · irreversible" — per https://gangleri42.github.io/seedhammer/).

## Resolved

### `me-bundle-preview-layer` — Phase A DONE 2026-06-16
Shipped the pure-Rust **bundle orchestration core** (`me bundle`): reads newline-separated public md1/mk1 strings (stdin/`--in`), classifies + ms1-early-refuses, per-string pristine-validates, groups by `chunk_set_id`, and proves each chunk set complete/consistent (catches dropped/reordered/duplicate/foreign chunks via `mk_codec::decode` / `md_codec::chunk::reassemble`). Emits a JSON manifest (stdout/`--manifest`) + a guided per-plate checklist (stderr); refuses ms1 (exit 3). `me` → **v0.2.0**. Spec `design/SPEC_me_bundle_phaseA.md` (R0/R1 GREEN); plan `design/IMPLEMENTATION_PLAN_me_bundle_phaseA.md`. The faithful **preview sidecar** is split out as the new Phase-B `me-bundle-preview-sidecar` item (see Open) carrying `DESIGN_me_bundle_preview.md` §B (R0 findings I-3/I-4/m-5 + the upstream-v1.4.2 pin).

### Deferred formal subagent reviews — RESOLVED 2026-06-16
Both formal opus-architect **subagent** reviews deferred during the 2026-06-16 Agent-API outage (which had forced inline self-reviews) were run after agents recovered:
- **(a) PR2 (#35) final whole-diff review — DONE.** Caught 1 Important (md1/mk1 lowercase-only) + 3 Minor the inline self-review missed; folded in seedhammer `6ab12c0` (PR #35 updated), R1 **GREEN** (`design/agent-reports/firmware-pr2-mdmk-final-review-R{0,1}.md`).
- **(b) converter-polish diff (`5086119`) review — DONE.** R0 caught 1 Important (I-1: with `--echo`, the input was copied into an un-zeroized heap `String` *before* `convert()`, so `--echo --in <ms1-file>` left the secret un-scrubbed on the ms1-refusal path — defeating nit 4's defense-in-depth) + 1 Nit (N-1: echo test lacked a stdout-purity assertion). Folded: `echo_line` now built only when `cli.echo && result.is_ok()` and wrapped in `Zeroizing<String>`; echo test now asserts stdout stays binary-only. R1 **GREEN** (`design/agent-reports/me-converter-polish-final-review-R{0,1}.md`).

### Converter (`me`) polish cycle — RESOLVED 2026-06-16 (commit `5086119`)
All five nits from the converter execution review (`design/agent-reports/me-converter-execution-review.md`) were cleared in one PATCH cycle (spec `design/SPEC_me_converter_polish.md`, plan `design/IMPLEMENTATION_PLAN_me_converter_polish.md`):

- **`me-in-stdin-intermediate-zeroize`** — input now read into a `Zeroizing<String>`, scrubbed on drop (`main.rs`).
- **`me-validate-ms-unreachable`** — `panic!` → `unreachable!("ms1 is refused before validation")` (`validate.rs`).
- **`me-decode-text-tlv-comment`** — `decode_text_tlv` now documents its intentional 1-byte-TLV / no-terminator-check scope (`ndef.rs`).
- **`me-canonical-string-stderr`** — reconciled via an opt-in `--echo` flag (prints the validated string to stderr on success); spec §5 amended to match (`main.rs`, `cli.rs`, `SPEC_seedhammer_engrave.md`).
- **`me-go-harness-shortread-loop`** — the harness now reads the NDEF record in a short-read loop (`firmware/ndef-roundtrip/main.go`).

### crates.io publish — RESOLVED 2026-06-16
- **`me-crates-io-publish`** — **`mnemonic-engrave` v0.1.0 published** to crates.io (<https://crates.io/crates/mnemonic-engrave>; `cargo install mnemonic-engrave` → the `me` binary). Added publish metadata (`repository`/`homepage`/`keywords`/`categories`) + a crate-local `README.md` (`9ad758c`); dry-run green; uploaded with a `publish-new`-scoped token. Future versions: bump `version` and `cargo publish` (needs `publish-update` scope).
