# Verdict — D6-4 (adversarial verify #0)

**Finding:** NDEF TLV/short-record length boundary is entirely untested; a boundary
mutation emits a 0xFF length byte the real device misparses.
**Location:** `crates/me-cli/src/ndef.rs:48`
**Verdict: CONFIRMED (not refuted). Severity: moderate (unchanged). Confidence: high.**

## What the finding claims
1. `tlv_wrap` guards `message.len() >= 0xFF` (line 48); max valid text = 249 chars.
2. `rejects_oversize` exercises the `text_record` payload_len guard, NOT the `tlv_wrap`
   boundary; no test lives near 249–255.
3. Mutation M2 (`>= 0xFF` → `> 0xFF`) lets a 255-byte message wrap with a 1-byte TLV
   length of `0xFF`.
4. SeedHammer's reader (`third_party/seedhammer/nfc/ndef/ndef.go:73`) treats `0xFF` as
   the 2-byte-length escape → reads the record header `D1 01` as a big-endian length
   `0xD101` → total misparse.

## Evidence gathered

### Cited code is exactly as claimed
`ndef.rs:48` reads `if message.len() >= 0xFF { return Err(NdefError::TooLong(...)) }`.
`text_record` (line 31) guards `payload_len > u8::MAX` where `payload_len = 1 + text.len()`.
The emitted message length = 4 fixed header bytes + payload_len = `5 + text.len()`
(confirmed by the `encodes_expected_bytes` golden: "md1q" → TLV len 0x09 = 5+4).

### Boundary arithmetic — probe (outside repo)
`msg_len = 5 + text.len()`; current guard rejects `msg_len >= 255`:

```
text=248: msg_len=253  current[>=0xFF]=emit 0xfd   mutated[>0xFF]=emit 0xfd
text=249: msg_len=254  current[>=0xFF]=emit 0xfe   mutated[>0xFF]=emit 0xfe   <- max valid text
text=250: msg_len=255  current[>=0xFF]=REJECT      mutated[>0xFF]=emit 0xff   <- MUTANT LEAKS
text=251: msg_len=256  current[>=0xFF]=REJECT      mutated[>0xFF]=REJECT
text=254: msg_len=259  current[>=0xFF]=REJECT      mutated[>0xFF]=REJECT
text=255: text_record REJECTS
```

So the mutation M2 produces a `0xFF` length byte at **exactly** `text.len()==250`
(msg_len 255). At that length `text_record` succeeds (payload_len=251, not >255), so
`tlv_wrap`'s guard is the **sole** load-bearing check — no redundancy at the boundary.

### `rejects_oversize` does not cover it — confirmed
`rejects_oversize` calls `text_record("a".repeat(255))`; payload_len=256 > 255, so it
trips the **text_record** guard and returns before `tlv_wrap` is ever reached. Grep of
`crates/me-cli/src` + `tests/` for length literals near the boundary finds only
`repeat(255)` (text_record guard) and `repeat(400)` (advisory plate-budget). No test
constructs a 249/250/254-byte case, and no test asserts the emitted TLV length byte.
The `tlv_wrap` boundary is genuinely unpinned.

### Device misparse mechanism — confirmed against source
`third_party/seedhammer/nfc/ndef/ndef.go` reads a 1-byte length then:
`length8 := buf[0]; length := int(length8); if length8 == 0xff { /* 2-byte length */
buf = r.scratch[:2]; ...; length = int(binary.BigEndian.Uint16(buf)) }`.
A `0xFF` length byte is therefore taken as the escape and the next two bytes (the NDEF
record header `0xD1 0x01`) are read as `0xD101` (53505) → total misparse. The finding's
device-side claim is accurate.

### Upstream reachability
`convert()` (lib.rs:56) applies classify + validate then calls `encode_text_tlv` with
**no length cap**; `exceeds_plate_budget` is advisory and is not invoked on the convert
path. A valid ~250-char md1/mk1 string flows straight into `encode_text_tlv`. The
finding is explicitly mutation/refactor-conditional and honestly states the shipped
code is currently safe (250+ → TooLong → exit 4).

## Refutation attempts (all failed)
- **"Another layer catches it"**: at the exact trigger (text.len()==250) `text_record`
  succeeds; `tlv_wrap` is the only guard. No redundancy. Rejected.
- **"rejects_oversize already covers the boundary"**: it hits the text_record guard at
  255, never reaching tlv_wrap. Rejected.
- **"Reader tolerates 0xFF"**: source shows the opposite — 0xFF is the escape sentinel.
  Rejected.
- **"Not reachable"**: no upstream length cap on the convert path. Rejected.

## Severity assessment
Moderate is honest and internally consistent with the finder's calibration (peer of the
other moderate, D6-5). It is a test-adequacy gap (not a live bug): the trigger is a
single exact length (250) plus a specific mutation direction, and D6-1 means CI runs no
tests anyway. But the consequence if a regression slips is a wrong-but-accepted plate
(device engraves a truncated/garbage descriptor the user believes was validated) — a
genuine funds-relevant outcome. A one-line boundary test (assert 249→len byte 0xFE valid;
250→TooLong) would close it. No downgrade warranted.

**Conclusion: CONFIRMED, moderate, high confidence.**
