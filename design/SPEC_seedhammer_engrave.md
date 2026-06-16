# Design — `mnemonic-engrave`: engraving the constellation on SeedHammer II

- **Status:** Architect R-loop GREEN (R0→R3, 0C/0I, see `design/agent-reports/`); awaiting user spec review before plan-writing.
- **Date:** 2026-06-16
- **Author:** Brian Goss (with Claude)
- **Reference firmware:** SeedHammer II `v1.4.2` (`/scratch/code/shibboleth/seedhammer-ref-v1.4.2`), Go/TinyGo, RP2350, public domain (Unlicense) + DCO.
- **Sibling crates:** `descriptor-mnemonic` (`md-codec`, `md1`), `mnemonic-key` (`mk-codec`, `mk1`), `mnemonic-secret` (`ms-codec`, `ms1`).

## 1. Background & goal

The **m-format constellation** is a family of Rust codecs that encode Bitcoin wallet artifacts as bech32-style strings designed to be hand-transcribed / engraved on steel:

- `md1…` — wallet descriptor / spending policy (miniscript template + derivation + cosigner xpubs). Uses a **local fork of BIP-93's BCH** with HRP-mixed per-format target residues (**not** codex32).
- `mk1…` — xpubs. Same local HRP-mixed BCH (**not** codex32).
- `ms1…` — **secret** (BIP-39 entropy). This one **is** BIP-93 codex32 directly (HRP `ms`, threshold `0`, id `entr`, share `S`), via Poelstra's `rust-codex32`.

**Goal:** make a SeedHammer II engrave these three strings **verbatim**, while keeping the secret (`ms1`) entirely off-RF. This project (`mnemonic-engrave`) is the sibling that bridges constellation output → the engraver.

### Why the obvious paths don't work as-is (recon summary)

- The SeedHammer II's **only data input is NFC** (ST25R3916). No camera, no SD, no USB-data; USB is firmware-flash only. The touchscreen allows manual BIP-39 word entry and a codex32 entry flow.
- The on-device scanner (`gui/scan.go:26-73`) is a **format whitelist** — BIP-39 → BIP-380 descriptor → codex32 — and drops anything else as "unknown format" **before** the engrave step. There is **no "engrave arbitrary text" path**.
- Our `md1`/`mk1` use a non-codex32 BCH residue, so the stock firmware rejects them. `ms1` **is** codex32 and is accepted by the stock parser, but we deliberately do **not** send it over RF.

## 2. Goals / non-goals

**v1 goals**
- A host-side Rust CLI that converts a single constellation string into an NFC-writable payload (for the **public** strings `md1`/`mk1`), and **refuses** `ms1`.
- A SeedHammer firmware change that (a) exposes air-gapped on-device `ms1` (codex32) entry, and (b) recognizes and **BCH-validates** `md1`/`mk1` and engraves them as text/QR.
- An end-to-end path: public strings via NFC; secret string typed on-device.

**v1 non-goals (explicitly deferred)**
- Bundle manifest / guided multi-plate ("plate N of M") workflow.
- Plate-preview rendering on the host.
- Direct USB-NFC tag writing from the tool (phone-based for v1).
- String *generation* — that stays in the existing CLIs / `mnemonic-toolkit`. `mnemonic-engrave` **consumes** already-produced constellation strings.

## 3. Security model (the spine)

| String | Sensitivity | Transport to device | Engraves as |
|---|---|---|---|
| `md1` (policy/template) | public | NFC NDEF Text record (phone writes a tag / pushes to machine) | text + QR plate |
| `mk1` (xpubs) | public | NFC NDEF Text record | text + QR plate |
| `ms1` (**secret** entropy) | **secret** | **never over RF** — hand-typed on the air-gapped touchscreen | codex32 seed plate |

**Rationale:** `ms1` encodes raw seed entropy; transmitting it over RF (NFC) exposes it to eavesdropping and defeats the air-gap. `md1`/`mk1` are public-key/policy artifacts — transporting them over NFC is equivalent to scanning a QR and carries only the usual wallet-privacy (not theft) risk.

The converter **refuses `ms1`** (non-zero exit) with a message explaining the RF risk and directing the user to on-device entry. Input is **stdin/file only — never argv** (argv leaks into `ps`, `/proc`, and shell history; even a *refused* secret must never reach it).

## 4. Architecture

Two independent deliverables:

```
existing CLIs / mnemonic-toolkit ──► md1, mk1, ms1   (one string, or one chunk, at a time)

  md1 / mk1 ─► `me` validates (md-codec/mk-codec) ─► NDEF Text (TLV-wrapped)
              ─► phone writes/pushes over NFC ─► (patched) scanner BCH-validates ─► text+QR plate

  ms1       ─► `me` REFUSES (RF risk) ─► user selects "CODEX32" on device
              ─► types ms1 (live-validated by codex32.New) ─► codex32 seed plate
```

### Per-string model & chunking

`mk1` (and large `md1`) cards are **chunked** into multiple self-contained bech32 strings (`md-codec/src/chunk.rs`, `mk-codec/src/string_layer/header.rs` — `SingleString` vs `Chunked` headers carrying `chunk_set_id`/`total_chunks`/`chunk_index`/`cross_chunk_hash`). **Each chunk is a complete bech32 string with its own BCH checksum.**

The engraver does **not** reassemble chunks — reassembly is a **recovery-time** job for the Rust tooling. A multi-chunk card is simply **multiple plates** (a "bundle"). The v1 per-string converter therefore needs no chunk awareness: feed it one string (one chunk) → one NDEF → one plate. The deferred bundle layer is what sequences "card = N chunks = N plates."

The `chunk_index`/`total_chunks`/`chunk_set_id` live **inside** the BCH-covered data part of each string (`mk-codec/src/string_layer/header.rs`), so engraving a chunk verbatim preserves everything reassembly needs. Note, though, that per-chunk BCH validation **cannot detect a dropped or reordered chunk** — the `cross_chunk_hash` that guards set integrity is checked only at reassembly (`chunk.rs`), so missing-chunk detection is **deferred to the (non-goal v1) bundle layer**. v1 engraves each chunk independently with no on-device set-completeness check.

## 5. Component A — Rust converter `mnemonic-engrave` (binary `me`)

- **Crate:** `crates/me-cli` in this repo, family layout. Binary installed as `mnemonic-engrave` with short alias `me`.
- **Input:** exactly one constellation string via **stdin** or `--in <file>` (never a positional argv secret).
- **Validation:** depend on `md-codec` / `mk-codec` / `ms-codec` to **fully parse and verify** the string — classify HRP, verify the BCH checksum, and reliably identify `ms1`. A malformed `md1`/`mk1` is rejected before it can be engraved.
- **Behavior by HRP:**
  - `md1` / `mk1`: write the NDEF message bytes (§6) to `--out <file>` (default) or stdout via `--stdout`; the canonical validated string (for pasting into a phone NFC-writer app) and any guidance go to **stderr**, so binary output and human text never collide on the same stream. Optional `--hex` / `--base64` make the NDEF safe to print on stdout directly.
  - `ms1`: **refuse**, exit non-zero, print the RF-risk explanation + on-device-entry instructions.
  - unknown HRP / failed checksum: error naming the problem.
- **Length guard:** warn when a string is long enough to risk overflowing the plate with a QR (firmware still backstops via `ErrTooLarge`).
- **Hygiene:** offline tool; reuse the family `mlock`/`zeroize` pattern for in-memory buffers (note: this cannot scrub OS-held argv copies — hence stdin-only).

## 6. NDEF wire format (committed contract)

The converter emits an **NFC Forum Type-2/Type-5 TLV-wrapped NDEF message** containing one **well-known Text record**, matching what `nfc/ndef/ndef.go` parses:

```
TLV wrapper:   03 <len> <NDEF message…> FE
                │   │                     └ terminator TLV
                │   └ length (1 byte if <255; else FF + 2-byte big-endian)
                └ NDEF-message TLV type

NDEF message (single record):
  D1            header: MB=1 ME=1 CF=0 SR=1 IL=0 TNF=001(well-known)
  01            type length = 1
  <plen>        payload length (1 byte, SR=1)  = 1 + len(text)
  54            type = 'T' (Text)
  00            status byte: bit7=0 (UTF-8), lang-code length = 0
  <text…>       UTF-8 payload = the verbatim md1/mk1 string
```

- Empty language code (`status = 0x00`) — `ndef.go:193-200` skips `langLen` bytes; zero is simplest and maximally compatible.
- The **T4T phone-push** path presents the NDEF message without the TLV wrapper (`poller.go:85-88` feeds the emulator output straight to `NewRecordReader`); the **passive-tag** path expects the TLV wrapper (`poller.go:83-84` via `NewMessageReader`). Commodity phone NFC-writer apps write the TLV-wrapped form to passive tags by default, so the converter emits the TLV-wrapped form as canonical. A `--no-tlv` flag MAY emit the bare message for direct T4T tooling.
- A **golden test vector** (input string → exact bytes) is committed and cross-checked against SeedHammer's Go reader (§9).

## 7. Component B — SeedHammer firmware patches

Two **separate** upstream PRs (DCO-signed), so the safe one lands independently of the contentious one. A small private fork is the fallback if the second is declined.

### PR 1 — expose on-device `ms1` (codex32) entry  *(trivial, ship first)*
- `gui/gui.go:1806`: uncomment `"CODEX32"` in the `newInputFlow` menu. `case 2:` → `inputCodex32Flow` (`gui/gui.go:623`) is already fully wired and functional; the keyboard covers the full bech32 alphabet and live-validates via `codex32.New`. `ms1` is accepted (HRP `ms`, threshold 0 ⇒ share `S`, length 48–93). Engraves via the existing codex32 seed-string plate (`backup.EngraveSeedString`, which uppercases — fine, bech32 is case-insensitive).
- One-line, zero new failure modes; clearly intended (`// TODO: re-enable`).

### PR 2 — recognize & BCH-validate `md1`/`mk1`, engrave as text/QR  *(the real ask)*
- **Go BCH verifier for md1/mk1:** reuse SeedHammer's generic checksum `engine` (`codex32/checksum.go`, fields `generator`/`residue`/`target`) by adding constructors for the md/mk generator + target residues. This validates each (single or chunked) string with the same rigor as codex32 — **no opaque text engraving**.
  - **Representation conversion (the real porting work — do not gloss):** the md/mk constants live in the Rust codecs as a **packed-`u128` polymod** — md target+generator in `md-codec/src/bch.rs:7-21` (`MD_REGULAR_CONST`, `GEN_REGULAR`); mk targets in `mk-codec/src/consts.rs:18,21` (`MK_REGULAR_CONST`, `MK_LONG_CONST`) with generators in `mk-codec/src/string_layer/bch.rs:173` (`GEN_REGULAR`, `GEN_LONG`). SeedHammer's `engine` instead stores `generator`/`target` as **GF(32) coefficient vectors** (`[]fe`). The constants must be **re-expressed** u128 → `[]fe` (13 elements for a regular code, 15 for a long code); they cannot be copied across verbatim. The §9 **BCH parity test is the correctness gate** for this conversion.
  - **Code coverage:** the Go verifier implements **md1 = regular code only** (`md-codec/src/bch.rs:1`) and **mk1 = both regular (13-symbol) and long (15-symbol)** codes (`mk-codec/src/string_layer/bch.rs:318-347`). Shipping mk1 regular-only would silently reject every long-code mk1 chunk as "unknown format."
- **Scanner branch** (`gui/scan.go`): after the existing cascade, detect the `md1`/`mk1` HRP (case-insensitive), run the matching BCH verifier; on success route the raw string as engravable text to the existing `backup.EngraveText`/`Paragraph` path, offering the same **TEXT+QR / TEXT / QR-ONLY** choice descriptors get. On checksum failure, fall through to "unknown format" (no garbage engraved).
- **Per-string only** — no on-device chunk reassembly (reassembly is recovery-time, in Rust). Each chunk is its own plate.
- Reuses the existing `toPlate`/`ErrTooLarge` fit check; no new failure modes there.

## 8. Error handling

- **Converter:** invalid checksum → error naming the string; `ms1` → refuse + security note + on-device instructions; unknown HRP → error; over-long → warn.
- **Firmware:** BCH-fail → "unknown format" (unchanged UX); over-large plate → existing `ErrTooLarge`.

## 9. Testing

- **Converter unit tests:** known `md1`/`mk1`/`ms1` vectors (reuse sibling test vectors); golden NDEF bytes; negatives (`ms1` refused, corrupt rejected, unknown HRP).
- **Cross-language NDEF round-trip (anchor test):** bytes emitted by the Rust converter, parsed by SeedHammer's Go `nfc/ndef` reader, must round-trip to the exact input string. Proves both halves agree on the wire format (incl. the TLV wrapper).
- **BCH parity test:** the Go md1/mk1 verifier and the Rust `md-codec`/`mk-codec` checksum agree on a shared vector set (validates the ported generator/target constants).
- **Firmware Go tests:** route `md1`/`mk1` chunks through the scan path to a text plate via the existing golden-engrave harness; assert corrupt strings are rejected; assert the `CODEX32` menu entry is reachable.

## 10. Verify on real hardware (cannot be settled from source)

1. **Secure-boot / OTP status of a retail unit** — does it accept a self-built unsigned UF2 in BOOTSEL mode? (Decides whether the fork-fallback is even viable.)
2. **Plate fit:** an ~111-char `mk1` chunk **with** a QR on the 85×85 mm plate.
3. **End-to-end:** `me` → phone NFC-write → patched device → engraved `md1`/`mk1` plate; and on-device `ms1` entry → codex32 plate.

## 11. Version coupling & invariants

- Pin the sibling codec crate versions; the **BCH parity test** (§9) guards against silent generator/residue drift between the Rust codecs and the ported Go verifier (the source-of-truth constants are `md-codec/src/bch.rs` and `mk-codec/src/{consts.rs,string_layer/bch.rs}` — note md-codec has no `consts.rs`, and mk-codec's BCH lives under `string_layer/`).
- If this tool grows a user-facing CLI surface, honor the family **manual-mirror invariant** (mirror flags into `mnemonic-toolkit/docs/manual/`); a future manual chapter lives under `docs/`.

## 12. Project layout

```
mnemonic-engrave/
  crates/me-cli/        # Rust converter (binary `mnemonic-engrave`, alias `me`)
  firmware/             # Go patch series + notes for the two upstream PRs
  docs/                 # this spec; future manual chapter
```

## 13. Phasing

1. Converter `md1`/`mk1` → NDEF + the NDEF round-trip + BCH parity tests.
2. Firmware **PR 1** (CODEX32 uncomment) — ship/upstream first.
3. Firmware **PR 2** (BCH-validated md1/mk1 engrave) — upstream; fork fallback.
4. Hardware verification (§10).
5. (Later) bundle manifest + guided multi-plate workflow; host-side plate preview.

## 14. Open risks

- **Upstream may decline PR 2** (three non-BIP formats in their firmware). Mitigated by the BCH-validated framing and the fork fallback — but the fork's viability depends on the secure-boot finding (§10.1).
- **NDEF delivery UX** depends on which phone NFC-writer app the user uses; the TLV-wrapped canonical form targets the common case, with `--no-tlv` for T4T tooling.
- **Sibling codec drift** — guarded by the parity test but requires discipline on version bumps.
