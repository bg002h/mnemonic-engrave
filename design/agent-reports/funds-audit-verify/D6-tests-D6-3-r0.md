# Adversarial verification — D6-3 (verifier #0)

Finding: "Only one golden .ndef vector (24-char md1); no mk1/long/glyph golden and
round-trips use me's own decoder (symmetric-bug blind)."
Location: `crates/me-cli/tests/golden.rs:4`. Claimed severity: important (funds).

Verdict: **REFUTED** as an important funds-safety finding. Residual value is at most a
**low** defense-in-depth test-hardening nicety. Confidence: high.

---

## What the surface facts actually are (verified)

- `golden.rs` (read): pins `convert("md1yqpqqxqq8xtwhw4xwn4qh")` == `vectors/md1-short.ndef`.
  One 24-char md1. TRUE.
- `ls tests/vectors/` → only `md1-short.ndef` (+ `bundle-md1-mk1.json`). No mk1/long/boundary
  `.ndef` golden. TRUE. `xxd` confirms the golden decodes as `03 1d D1 01 19 54 00 <24 md1
  bytes> FE` — a genuine, correct 24-char md1 TLV.
- `lib::converts_md1_to_ndef` (lib.rs:75) decodes with `ndef::decode_text_tlv`. TRUE self-RT.
- `ndef::round_trips` (ndef.rs:126) decodes with `decode_text_tlv`. TRUE self-RT. (Its input
  `"mk1qpzry9x8gf2tv"` is actually mk1-shaped and adds z,r,9,g,f,2,v + the 'k' HRP char — so
  even the "no mk1 anything" framing is slightly soft, though it remains a self-round-trip.)

So the plain observations are largely accurate. The problem is the two *load-bearing* claims.

## Load-bearing claim #1 is FALSE: "Only the single golden is an independent byte anchor"

There are **three** independent anchors over the very same encoder, not one:

1. `golden.rs` — `convert(24-char md1)` == pinned file bytes.
2. `ndef::encodes_expected_bytes` (ndef.rs:110) — `encode_text_tlv("md1q")` == a **hardcoded
   expected byte vector** (`0x03,0x09,0xD1,0x01,0x05,0x54,0x00,'m','d','1','q',0xFE`). This is
   a second, fully independent byte anchor — it does NOT round-trip through me's decoder.
3. `cross_lang.rs` — `convert(md1)` is parsed by **SeedHammer's real Go `nfc/ndef` reader**
   (`firmware/ndef-roundtrip`) and must round-trip to the exact input. Independent decoder,
   the actual device parser. (Only when `go` is present — that gating is the separate D6-2.)

The finder's own §1 table even lists `encodes_expected_bytes` as "Hard byte anchor #1", which
contradicts D6-3's "only the single golden" wording.

## Load-bearing claim #2 (the failure scenario) is NOT reachable in the cited encoder

The encoder is charset- and format-agnostic. `convert` (lib.rs:56-63) trims, classifies,
validates, then calls one shared `ndef::encode_text_tlv(s)` for **both** md1 and mk1.
`text_record` (ndef.rs:30-42) emits fixed header bytes then
`out.extend_from_slice(text.as_bytes())` — a raw, positional, per-byte memcpy of the whole
string (ndef.rs:41). There is **no** per-character, per-glyph, per-HRP, or per-format branch
anywhere in the NDEF path (grep for glyph/charset/qpzry finds nothing but the test's own input).

Consequences for the stated scenario:

- **"corrupts a specific bech32 glyph"** — no code path exists that could selectively corrupt
  one glyph and not another; the copy is byte-uniform. The finder itself admits this in §5 P4:
  "`encode_text_tlv` is charset-agnostic (raw str byte copy)" — which directly refutes D6-3's
  own scenario. A per-glyph corruption can only live in the **Go font/render** path, which is a
  *different layer* covered by D6-5, not the NDEF encoder that `golden.rs` anchors.
- **"an mk1-length string"** — md1 and mk1 traverse the identical encoder; no distinct branch.
  The only length-dependent branch is the `TooLong` boundary (~254 bytes: `payload_len > 255`
  / `message.len() >= 0xFF`), and mk1 single strings are far below it. That boundary is a
  separate finding (D6-4).

Any *real* single-point mutation of `text_record`/`tlv_wrap` (drop a byte, offset, wrong
length byte) manifests on the 24-char md1 golden **and** on `encodes_expected_bytes` — the
finder's own M1/M3 spot-checks confirm both are CAUGHT. There is no demonstrated single-point
encoder mutation that yields a wrong mk1/glyph NDEF while simultaneously passing the golden,
the hardcoded-byte anchor, and the self round-trip. The concrete "wrong plate emitted and
accepted" outcome is therefore not substantiated for this encoder.

## Severity assessment

The honest residual is: "it would be nice to add mk1/long/boundary golden `.ndef` vectors and
an independent-decoder round-trip corpus" — defense-in-depth. That is a **low**-severity
test-hardening suggestion. It is not an important funds-safety gap because:
- no reachable wrong-but-accepted-plate path is shown (encoder is a raw byte copy with two
  independent byte anchors + a real-device-reader differential for the primary md1);
- the genuinely adjacent risks it gestures at (TLV/short-record boundary; preview glyph
  render) are already separately filed as D6-4 and D6-5.

## Verdict

refuted = true (as stated at "important" funds severity). adjustedSeverity = low
(residual defense-in-depth corpus expansion only). confidence = high.
