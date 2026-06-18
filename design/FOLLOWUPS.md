# FOLLOWUPS — mnemonic-engrave

Low/nit items deferred from architect reviews (per the iterative-architect-review standard: Critical/Important fixed inline; low/nit recorded here). Promote to a cycle when convenient.

## Open

> These are **cycle-sized** items (bigger than architect-review nits) — each warrants its own brainstorm → spec → plan → R0 → implement pass when picked up.

- **`seedhammer-upstream-prs-tracking`** — Track the two open upstream PRs to `seedhammer/seedhammer`: **#34** (re-enable on-device CODEX32 entry) and **#35** (BCH-validated md1/mk1 engraving). Respond to maintainer feedback; mirror any requested changes back. **If declined or stalled:** pursue the fork-fallback — stand up a `seedhammer-fork` sibling repo and document the "Set custom boot key" path (program a 2nd RP2350 OTP boot-key slot via picotool to run own-signed firmware on a locked SH2; "Advanced · irreversible" — per https://gangleri42.github.io/seedhammer/).

- **`seedhammer-slip39-hwsha`** — **PENDED 2026-06-18 (user direction — "pend the cipher accelerator for now").** Do not pick up until the user revives it; when resumed, the entry point is the Phase-0 hardware-benchmark spike below (cheapest unlock: a ~$5 RP2350 board — Pico 2 / Pico Plus 2 — NOT a SeedHammer II; can be prepped as a flash-and-read harness). — Add an RP2350 **hardware-SHA-256** path for SLIP-39's PBKDF2-HMAC-SHA256 Feistel round function (TinyGo uses pure-Go `crypto/sha256` today; high-iteration-exponent recovery is slow — e=15 ≈ 5–8.5 h; e=0/1 ≈ 0.5–1.9 s is fine). **Cycle-prep recon DONE 2026-06-18** (`design/cycle-prep-recon-slip39-hwsha.md`, 4-agent fan-out vs the RP2350 datasheet + pinned TinyGo SVD): **feasibility = GO** (firmware runs Secure state; SHA default ACCESSCTRL = Secure-Privileged-allowed; `LOCK_SHA_256` is a cooperative bootrom mutex off by default — NOT a lock-out; TinyGo pre-declares `rp.SHA256`). **BUT the performance win is UNPROVEN** — the hardware can't load an arbitrary IV, so a drop-in `hash.Hash` forfeits HMAC's marshal fast-path (~2× block count) and per-block CPU polling is slow; a naive drop-in may negate/invert the win. The real path is a **bespoke hw HMAC-PBKDF2 loop** that **must be benchmarked on real RP2350 first** (Phase-0 spike gates whether to build the cycle), and the value is bounded to **rare high-e backups**. **Priority LOW** — parked behind the benchmark spike. (See the cycle-prep doc for the full register/throughput facts + must-handles: no watchdog → bound polling; single shared block + two-distinct-HMAC-hashes → mutex/confine; register secret-scrub; build-tag host oracle + on-device golden.)

## Resolved

### `seedhammer-slip39-recovery-trezor-routing` — DONE 2026-06-18 (fork `main` `bc63caa`)
Shipped the two-way post-recovery fork (the rescope of `-verbatim-hex`; architect consult
`design/agent-reports/seedhammer-slip39-verbatim-hex-design-consult.md` found verbatim-hex is a
non-restorable artifact — won't-build). `engraveRecoveredSLIP39`'s one-way acknowledgement is now
a `ChoiceScreen`: **"BIP-39 seed"** (this toolkit / from a phrase → fingerprint check →
`backupWalletFlow`) vs **"Engrave shares"** (Trezor / other SLIP-39 wallet → `engraveSLIP39Verbatim`
on the share, **no** BIP-39 fingerprint). Removes the dead-end for non-constellation backups +
a README doc line. Gated: consult → plan R0→R1 GREEN → single-implementer TDD → whole-diff
execution review GREEN (0C/0I); reviews `design/agent-reports/seedhammer-slip39-trezor-routing-*`.

### `seedhammer-slip39-cycleC-all-lengths` — RESOLVED-BY-D2 2026-06-18
Cycle D Phase D1 widened `slip39.ParseShare` to accept all valid SLIP-39 share lengths
({20,23,27,30,33} words → {16,20,24,28,32} B; dropped `errUnsupportedSize`/`wordsShort`/
`wordsLong`), and Phase D2 added a **word-count picker** to the menu `case 3:` single-share
entry (`inputSLIP39Flow` gained a variable length). So the single-share verbatim entry+engrave
path now accepts all lengths, not just 20-word/128-bit — exactly this followup's ask. Shipped
on fork `main` `9db3fd2`.

### `seedhammer-slip39-recovery` (Cycle D) — DONE 2026-06-18 (fork `main` `9db3fd2`)
On-device SLIP-0039 secret recovery. **D1** (`f0092d5`): in-tree Go port of
`mnemonic_toolkit::slip39` — GF(256) field, Lagrange, 4-round Feistel decrypt, two-level
`Combine`, share-value extraction; no `math/big`; TDD vs official vectors + Rust-`split`-
generated intermediate-length fixtures. **D2** (`9db3fd2`): GUI recover flow — Recover button,
all-length entry, two-level roster + `selectForCombine`, optional SLIP-39 passphrase, the
entropy-interpretation hold-to-confirm + always-on fingerprint display, engrave via
`backupWalletFlow`. Full gated pipeline (spec R0→R1 + 4-lens architect panel; D1 plan R0→R2,
D2 plan R0→R1; both impl + whole-diff execution review GREEN 0C/0I). Reviews:
`design/agent-reports/seedhammer-slip39-recovery-*`. Two follow-ons filed above
(`-verbatim-hex`, `-hwsha`).

### `me-bundle-preview-sidecar` — Phase B DONE 2026-06-16 (v0.3.0)
Shipped the faithful host-side **plate preview** + the signed cross-platform release-CI. The `me-preview` (Go) sidecar (`preview/`) pins **UPSTREAM seedhammer v1.4.2** via a git submodule (`third_party/seedhammer` @ `713aee2`) and renders ONLY a validated public string → `engrave.Engraving` → SVG (optional `--png`):
- **B1 (sidecar/trust split) — DONE.** `preview/go.mod` imports `backup`+`engrave` directly (not `gui`); `seedhammer.com v0.0.0` sentinel + local `replace` (the `firmware/ndef-roundtrip/` pattern); not blocked on PR #35. The sidecar has no secrets and no network; `me` excludes ms1 from rendering.
- **B2 (faithfulness) — DONE.** Replicated `validateMdmk` layout: `backup.EngraveText`, QR via `qr.Encode(s, qr.L)`, `qrScale = 3`, modes TEXT+QR / TEXT / QR-only; replicated SH2 `engrave.Params` with a geometry-golden drift-guard; **exact cubic-Bézier SVG** (mirrors seedhammer's own `internal/golden` renderer — single `<path>`, B-spline G1 continuity preserved). Fidelity target = exact (not approximate).
- **B3 (delivery/version binding) — DONE.** `me bundle --preview <dir>` locates `me-preview` beside itself / on `$PATH`, checks `me-preview --version` against `CARGO_PKG_VERSION` (mismatch → exit 2, never a silent stale render), and degrades gracefully when absent (manifest + checklist still emitted, exit 0). `.github/workflows/release.yml` builds all targets (windows/arm64 omitted), assembles per-platform archives (`me` + `me-preview` + `minisign.pub` + `THIRD_PARTY_LICENSES` + verify note), and minisign-signs `SHA256SUMS`. A Rust↔Go cross-lang round-trip test (`crates/me-cli/tests/preview_cross_lang.rs`) builds the real sidecar and asserts one SVG per public plate, none for ms1.

`me` → **v0.3.0**. Spec `design/SPEC_me_bundle_phaseB_preview.md`; plan `design/IMPLEMENTATION_PLAN_me_bundle_phaseB_preview.md` (both R0/R1 GREEN). **Maintainer prerequisite — DONE:** the minisign keypair was generated (`minisign -G`); the public key is committed (`minisign.pub`, in README); the secret key + password are set as GitHub Secrets `MINISIGN_SECRET_KEY` / `MINISIGN_SECRET_KEY_PASSWORD` (never committed).

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
