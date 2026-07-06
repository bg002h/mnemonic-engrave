# Funds-safety audit — Dimension D2: NDEF encoding & manifest integrity

Auditor: D2 finder (round 0). Repo: `/scratch/code/shibboleth/mnemonic-engrave`.
Scope: `crates/me-cli/src/ndef.rs`, `crates/me-cli/src/manifest.rs`,
`firmware/ndef-roundtrip/main.go`, plus the callers that turn a validated md1/mk1
string into emitted bytes (`lib.rs::convert`, `bundle.rs::run_bundle`,
`main.rs`), and the *actual* machine consumer (`third_party/seedhammer/nfc/ndef`,
`nfc/poller`, `gui/scan.go`).

Verdict: **the core D2 surface is sound.** I found no path by which the emitted
NDEF/manifest differs from the validated input, no silent-truncation at the
short-record/255 boundary, and no invalid input admitted on the NDEF path. The
three findings below are all **integrity/coverage gaps around** the funds-safety
guarantee (test breadth, build hermeticity, write atomicity), **not** active
corruption bugs. Severities: 2 moderate, 1 low. No Critical/Important.

---

## What `me` emits, field by field, vs the NFC Forum NDEF 1.0 spec

`text_record(text)` (`ndef.rs:30`) builds one well-known Text record:

| byte | value | meaning |
|------|-------|---------|
| 0 | `0xD1` | header: MB=1 ME=1 CF=0 SR=1 IL=0, TNF=001 (well-known) |
| 1 | `0x01` | TYPE_LENGTH = 1 |
| 2 | `payload_len` (1 byte, SR) | = 1 (status) + text.len() |
| 3 | `0x54` `'T'` | TYPE |
| 4 | `0x00` | status: bit7=0 (UTF-8), bit6=0 (RFU), lang-len=0 |
| 5.. | text bytes | UTF-8 text |

`tlv_wrap(msg)` (`ndef.rs:47`) wraps it in the NFC-Forum Type-2/5 memory TLV:
`[0x03, msg.len(), msg..., 0xFE]` (NDEF-message TLV + terminator).

This is **spec-correct** against the NFC Forum NDEF record layout (verified against
the authoritative field definitions: TNF in bits 2–0, flags MB/ME/CF/SR/IL in bits
7–3, 1-byte TYPE_LENGTH, SR ⇒ 1-byte PAYLOAD_LENGTH; and the well-known Text
record's status byte with bit7=UTF-16 flag, bits5–0=language length). The status
byte `0x00` = UTF-8 + empty language is the minimal legal Text payload. Golden
vector `crates/me-cli/tests/vectors/md1-short.ndef` is byte-correct
(`03 1d d1 01 19 54 00 <24 text bytes> fe`; `0x1d`=29=5+24, `0x19`=25=1+24), and the
real `me --hex` reproduces it exactly (`031dd1011954006d64...fe`, verified live).

### The 255-byte short-record boundary — the silent-truncation class — is CLOSED

Two guards bound the output so the SR/TLV single-byte length forms can never
overflow:

- `text_record` (`ndef.rs:32`): rejects `payload_len = 1+text.len() > 255`
  → `text.len() >= 255` refused.
- `tlv_wrap` (`ndef.rs:48`): rejects `msg.len() = 5+text.len() >= 0xFF`
  → `text.len() >= 250` refused.

The tighter guard (`tlv_wrap`) is the binding one: **max accepted `text.len()` =
249**, giving `payload_len = 250` (fits one SR byte) and `msg.len() = 254` (fits
one TLV length byte, strictly < the `0xFF` escape). So `me` **never** reaches the
3-byte TLV length escape (`0x03 0xFF hi lo`) nor the 32-bit non-SR record form —
the two constructs that produce the classic silent wrong-length. It fails **closed**
(returns `NdefError::TooLong`, exit 4) rather than truncating.

**Probe (against the pinned submodule reader `third_party/seedhammer/nfc/ndef`):**
I replicated `me`'s exact byte layout and round-tripped every `text.len()` in
`0..=300` (pseudo-random bech32 content + all-`q` extreme) through the *real*
SeedHammer `MessageReader`+`RecordReader`:

```
me encoder REJECTS text.len() >= 250 (first rejected length)
max ACCEPTED text.len() = 249
ALL round-trips through the real seedhammer reader MATCHED for every accepted length.
NEGATIVE CONTROL (flip text byte): got="ld1..." err=<nil> equalToOrig=false
```

Every accepted length round-trips byte-exactly; the negative control (flip one
text byte) proves the reader/harness is a genuine oracle, not a no-op.

### The refuse-at-249 path is actually UNREACHABLE for valid input (fails safe)

`convert` (`lib.rs:56`) runs `validate()` **before** `encode_text_tlv()`. A valid
md1/mk1 is far shorter than 249 chars:

- **md1 single string**: `SINGLE_STRING_PAYLOAD_BIT_LIMIT = 64*5` in
  `md-codec 0.36.0 chunk.rs:225` ⇒ HRP `md1`(3) + ≤64 data + 13 checksum =
  **≤ 80 chars**. Larger descriptors are *chunked*, so each md1 chunk is also small.
- **mk1 string**: `mk-codec 0.4.0 error.rs` caps the data part at the
  "long-code maximum (108)" 5-bit symbols ⇒ HRP `mk1`(3) + ≤108 = **≤ 111 chars**.

So no valid single md1/mk1 approaches 250 chars: `encode_text_tlv`'s `TooLong`
branch and `exceeds_plate_budget`'s 300-char warning are pure defense-in-depth,
never triggered by admissible input. There is **no false refusal** of a valid
one-plate string, and no reachable path to the length boundary. Confirmed against
the codec source (authoritative), not the design docs.

### The reader also handles the long forms `me` never emits (reader is not the weak link)

For completeness I fed the pinned reader hand-built records using the **32-bit
non-SR payload length** AND the **`0xFF` 3-byte TLV length escape**, payloads
200/255/256/300/1000 bytes:

```
longForm L=200:  match=true  gotLen=200
longForm L=255:  match=true  gotLen=255
longForm L=256:  match=true  gotLen=256
longForm L=300:  match=true  gotLen=300
longForm L=1000: match=true  gotLen=1000
```

All round-trip. Even a hypothetical future encoder that emitted long form would be
read correctly — silent truncation is impossible on this transport in either
direction within the reachable and well-beyond range.

---

## Manifest faithfully describes the emitted payload (no re-encode divergence)

`me bundle` (`bundle.rs::run_bundle`) does **not** re-encode: it proves each chunk
set complete/consistent via `md_codec::chunk::reassemble` / `mk_codec::decode`
(`bundle.rs:246`, `:273`) — which catch dropped/reordered/duplicate/foreign chunks
(covered by the `dropped_/reordered_/duplicate_/cross_chunk_/foreign_mismatched_`
tests) — but the `PlateEntry.string` it emits is the **verbatim trimmed validated
input** (`s.clone()`, `bundle.rs:234/261/283`), the same bytes `validate()`
checked. `(index, string)` are carried as a paired tuple through the
`sort_by_key(index)` (`bundle.rs:244/271`), so a chunk can never be labelled with
another chunk's index. The preview path (`main.rs::wire_previews` →
`preview::render_plate`) renders that **same** `plate.string`, so preview and
bundle cannot diverge on the string. The manifest carries no separately-computed
hash/length that could drift from the strings it lists. `SetEntry.total =
chunks.len() as u8` is safe (`MAX_CHUNKS` = 32 md / 64 mk ≤ 255). Trim handling is
consistent across classify/validate/encode (all operate on the same trimmed `s`),
so no "validate one string, engrave another" seam. **Sound.**

`ms1` refusal on the NDEF path is enforced three times over: `convert` (`lib.rs:59`),
`run_bundle`'s classify-only pre-scan before any BCH work (`bundle.rs:188`), and
`parse_line` (`bundle.rs:97`); `wire_previews` skips `PlateKind::Ms1` so no secret
is ever handed to the sidecar (`main.rs:272`). Output files only ever contain
**public** md1/mk1 NDEF, so even a world-readable `--out`/preview is not a secret
leak (no funds-safety exposure via D2 outputs).

---

## `firmware/ndef-roundtrip` is a faithful decoder (real consumer code)

The harness (`main.go`) decodes with SeedHammer's own `ndef.NewMessageReader` +
`NewRecordReader` — the **exact** package the device's `nfc/poller` (`poller.go:83,
88`) feeds into `gui/scan.go::Scan`. It accumulates in a short-read loop
(`main.go:23`, appends `buf[:n]` then breaks on `io.EOF`), matching the scanner's
own accumulate-until-EOF pattern (`scan.go:30-48`). Because it uses the real
consumer's parser (not a mirror of `me`'s encoder), a compensating encoder bug
cannot cancel out — a wrong byte layout would fail to parse or parse to a different
string (demonstrated by the negative control above). It is the correct oracle. The
device's `scan.go` then routes the recovered string to md1/mk1 admission in the
**fork** (`bg002h/seedhammer`); the pinned upstream v1.4.2 submodule here routes to
`bip39.Parse`/`nonstandard.OutputDescriptor`/`codex32.New`, i.e. the fork adds the
md1/mk1 arms — but the **NDEF transport layer** (what D2 owns) is shared and
verified faithful.

---

## Findings

### D2-1 (moderate) — NDEF cross-language round-trip + golden cover only ONE 23-char md1 vector

`crates/me-cli/tests/cross_lang.rs:21` round-trips a single fixed input
(`"md1yqpqqxqq8xtwhw4xwn4qh"`) through the Go reader, and
`crates/me-cli/tests/golden.rs:6` byte-pins the same single vector. There is no
length sweep, no boundary case (max-accepted 249 / first-rejected 250), and no
`mk1` vector. The transport is currently correct (I proved it across `0..=300`),
but a **length-dependent regression** — e.g. someone "optimising" the length
guards, switching to the 3-byte TLV form, or an off-by-one at the SR boundary —
would sail through CI. This is the funds-safety cross-language anchor; it should
exercise the edges it is meant to protect.

- Failure scenario: a future edit changes `tlv_wrap`'s guard from `>= 0xFF` to
  `> 0xFF` (accepting `msg.len()==255`), so a 250-char text emits a TLV whose
  1-byte length field wraps/misparses. The single 23-char test still passes; a
  wrong/truncated plate ships.
- Suggested test: parametrised round-trip through the Go harness for
  `text.len() ∈ {0,1,63,64,80,111,248,249}` and an assertion that 250 is
  rejected with `TooLong`; plus one real `mk1` vector. Byte-pin the max-length
  accepted output as a second golden.

### D2-2 (moderate) — ndef-roundtrip harness builds against an out-of-repo, unpinned copy, not the submodule

`firmware/ndef-roundtrip/go.mod:7` is
`replace seedhammer.com => ../../../seedhammer-ref-v1.4.2` — a directory **outside
the repo**, untracked and unpinned, resolving to
`/scratch/code/shibboleth/seedhammer-ref-v1.4.2`. The repo already vendors the
pinned machine code as the submodule `third_party/seedhammer` (v1.4.2 @ `713aee2`),
and the sibling `preview/go.mod:12` correctly uses `../third_party/seedhammer`.
Two problems: (a) **hermeticity/reproducibility** — on a fresh clone the sibling is
absent, so `cross_lang.rs` (which only *skips* when `go` is missing, not when the
build fails) hard-fails its `assert!(out.status.success())`; (b) **oracle
integrity** — the safety round-trip validates against a copy that can silently
**drift** from the v1.4.2 pin the firmware and preview actually build. I diffed the
two `nfc/ndef/ndef.go` files: currently **byte-identical**, so no active
divergence — but the harness should point at the submodule so the funds-safety
oracle is authoritative and self-contained.

- Failure scenario: the external `seedhammer-ref-v1.4.2` is updated to a newer
  SeedHammer with a changed NDEF reader (or deleted). The round-trip test now
  either validates `me` against the wrong parser (false green) or fails to build
  for everyone without that sibling — while the shipped firmware/preview use the
  unchanged pinned submodule.
- Suggested test / fix: change the `replace` to `../../third_party/seedhammer`
  (matching preview), add a CI job that runs the cross-language test in a clean
  checkout with only the submodule initialised, and assert the harness's resolved
  `seedhammer.com` path is inside the repo.

### D2-3 (low) — non-atomic writes of the NDEF `--out` file and the `--manifest` file

`main.rs:140` (`std::fs::write(path, &bytes)` for `me convert --out`) and
`main.rs:205` (`std::fs::write(path, json...)` for `me bundle --manifest`) truncate
then write with no temp-file + `rename` and no `fsync`. A crash / `ENOSPC` /
`SIGKILL` mid-write leaves a truncated file that can look complete. Impact is low:
the NDEF file is < 260 bytes (a partial write is improbable and a truncated NDEF is
caught on device read / round-trip), and the manifest is advisory. Recorded for the
"silent partial failure" dimension.

- Failure scenario: disk fills while writing `--out backup.ndef`; a partial record
  is left; a script that only checks the exit code (which is non-zero here, so
  low real risk) or a user who ignores stderr writes the truncated tag.
- Suggested test: fault-inject a writer that errors after N bytes and assert the
  target path does not exist / is unchanged (i.e. write-to-temp-then-rename).

---

## Areas checked and found SOUND (negative results, for coverage)

- NDEF record field encoding vs NFC Forum NDEF 1.0 (TNF/MB/ME/SR/CF/IL,
  TYPE_LENGTH, SR PAYLOAD_LENGTH, Text status byte): correct; golden byte-pinned
  and reproduced live.
- Short-record / 255-byte boundary & >255 silent-truncation class: closed by two
  correct guards (`ndef.rs:32,48`); max emitted `msg.len()`=254 < `0xFF` escape;
  probe-proven across `0..=300`.
- Reachable md1/mk1 sizes: md1 ≤ 80 chars, mk1 ≤ 111 chars (from codec source) —
  far below 249, so no false refusal and the boundary is unreachable by valid
  input; the `TooLong`/plate-budget paths are pure defense-in-depth.
- Round-trip through the real pinned SeedHammer reader: byte-exact for every
  accepted length; negative control detects a single flipped text byte.
- Reader handling of the long forms `me` never emits (32-bit non-SR + `0xFF` TLV
  escape): correct up to 1000-byte payloads.
- Manifest faithfulness: strings are verbatim validated input, no re-encode, no
  index/string desync, preview renders the same string, no drifting hash/length.
- ms1 refusal on every NDEF/bundle/preview path (`lib.rs:59`, `bundle.rs:97,188`,
  `main.rs:272`); public-only outputs (no secret exposure via D2 output files).
- Harness (`firmware/ndef-roundtrip/main.go`) is the real consumer's parser
  (`nfc/ndef` via `poller`→`scan.go`), short-read-loop correct, a genuine oracle.
- No relevant open item duplicated: `me-go-harness-shortread-loop` (FOLLOWUPS,
  Resolved) is already fixed; `seedhammer-nfc-secret-refusal` (WON'T FIX) is a
  device-side ms1-over-NFC policy item, not an `me` NDEF-encoding concern.

## Reproduction

Probe module (outside the repo):
`/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/0afb8cf0-bb79-4c70-8970-96a68909972d/scratchpad/ndefprobe`
— `go.mod` replaces `seedhammer.com` with the **pinned submodule**
`third_party/seedhammer`; `main.go` replicates `me`'s encoder and sweeps
`text.len() 0..=300` through the real reader; `escape_test.go` exercises the long
forms. Live checks run: `me --hex` == golden; `me --stdout | (cd
firmware/ndef-roundtrip && go run .)` == input; `cargo test --test cross_lang`
green.
