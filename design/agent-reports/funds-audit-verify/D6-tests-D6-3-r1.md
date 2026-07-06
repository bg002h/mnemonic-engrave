# Adversarial verification — D6-3 (round 1)

**Finding:** D6-3 (important) — "Only one golden .ndef vector (24-char md1); no
mk1/long/glyph golden and round-trips use me's own decoder (symmetric-bug blind)."
**Location:** `crates/me-cli/tests/golden.rs:4`
**Verdict:** NOT fully refuted on the descriptive facts, but **funds-safety severity
overstated → downgrade to `low`.** `refuted=false`, `adjustedSeverity=low`, confidence high.

---

## 1. What the code actually shows (facts the finding gets right)

- `crates/me-cli/tests/golden.rs` — one test, one 24-char md1
  (`md1yqpqqxqq8xtwhw4xwn4qh`), one 32-byte vector `vectors/md1-short.ndef`.
  Hexdump confirms `03 1d D1 01 19 54 00 <24 text bytes> FE` = 32 bytes. **CONFIRMED.**
- Only one `.ndef` vector exists under `tests/vectors/`. The other fixture,
  `bundle-md1-mk1.json`, is a **manifest** JSON (not an NDEF byte anchor). So: no mk1
  `.ndef`, no long/boundary `.ndef`, no full-alphabet glyph `.ndef`. **CONFIRMED.**
- The two in-crate round-trip tests decode with `me`'s **own** decoder:
  - `lib::converts_md1_to_ndef` (`lib.rs:75`) → `ndef::decode_text_tlv`.
  - `ndef::round_trips` (`ndef.rs:126`) → `decode_text_tlv`.
  A symmetric encoder+decoder bug passes both. **CONFIRMED.**

So the descriptive test-gap claims are accurate; I cannot say the code differs from what
is described, hence `refuted=false` rather than `true`.

## 2. Where the finding's headline overstates (funds-safety lens)

**(a) "Only the single golden is an independent byte anchor" is imprecise.** There are
**two** independent literal-byte anchors plus a real third-party decoder:
- `ndef::encodes_expected_bytes` (`ndef.rs:110-120`) hand-writes the full expected TLV
  vector for `"md1q"` (`03 09 D1 01 05 54 00 6d 64 31 71 FE`) and asserts equality — an
  independent anchor for the entire record structure (header, type-len, payload-len math,
  `T`, status, verbatim text copy, terminator, TLV framing). Not decoder-based.
- `golden.rs` — external file anchor for the 24-char md1.
- `cross_lang.rs::rust_ndef_parses_in_seedhammer_go_reader` decodes the md1 NDEF with
  **SeedHammer's real `nfc/ndef` Go reader** (a genuinely independent decoder, not `me`'s)
  and asserts round-trip. This directly rebuts "symmetric-bug blind" for the md1 path when
  Go is present. (Its skip-when-Go-absent weakness is a *separate* finding, D6-2.)

**(b) The encoder has no per-glyph code surface, so "corrupts a specific bech32 glyph"
has nothing to mutate.** `text_record` (`ndef.rs:30-43`) copies the string with a single
`out.extend_from_slice(text.as_bytes())` — a verbatim bulk byte copy, charset-agnostic,
no per-character branch. The finder **admits this themselves** in P4: "since
`encode_text_tlv` is charset-agnostic (raw `str` byte copy)…". A mutation that corrupts
one specific glyph would have to *introduce entirely new per-char logic*, not mutate
existing code — that is not the "encoder mutation" class the failure scenario invokes.

**(c) mk1 and md1 traverse the identical encode path.** `convert` (`lib.rs:56-64`):
`classify → refuse ms → validate(fmt, s) → encode_text_tlv(s)`. There is **no md1/mk1
branch in the NDEF layer**; format only affects `validate` (which the golden does not
pin). An added mk1 golden would exercise the *same* bytes-copy code with different literal
text — real but low marginal coverage.

**(d) The one genuinely code-grounded residual is length/boundary, already filed as
D6-4.** The only length-dependent logic (`payload_len as u8`, `message.len() as u8`,
`>= 0xFF` / `> u8::MAX` guards) is the "mk1-length string" half of the failure scenario —
and that boundary is separately captured as **D6-4 (moderate)**. D6-3 should not double-
count it.

## 3. Reachability of the failure scenario

No current input produces a wrong-but-accepted plate. The scenario is a *hypothetical
future mutation*. For the charset half there is no code surface (verbatim copy); for the
length half, the residual is D6-4. The symmetric-bug concern is mitigated by two
independent literal anchors and the real Go device-reader for the covered md1 string.
No concrete funds-loss path is demonstrable from the current code.

## 4. Severity judgment

The finding is a legitimate **test-hygiene** observation (the golden corpus is genuinely
thin: no mk1/long/boundary/glyph vector, and the two in-crate round-trips are decoder-
symmetric). But at the **funds-safety** bar the verifier is asked to apply — "would it
really produce a wrong-but-accepted plate / lost funds?" — the answer is no, given:
charset-agnostic verbatim-copy encoder (no glyph code surface), two independent literal
byte anchors, a real SeedHammer Go-reader differential for the md1 path, and the length
residual already owned by D6-4. That is materially below "important."

**Downgrade to `low`.** Adding an mk1 golden, a long/boundary golden, and an
independent-decoder round-trip (e.g. reuse the Go harness) is worthwhile hardening, but it
closes a low-funds-impact gap, not an important one.

---

**Structured verdict:** `refuted=false`, `adjustedSeverity=low`, `confidence=high`.
