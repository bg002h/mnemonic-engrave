# Verdict — D2-1 (adversarial verifier #0)

**Finding:** D2-1 (moderate) — "NDEF cross-language round-trip + golden cover only
one 23-char md1 vector (no boundary/length-sweep/mk1)."
**Location cited:** `crates/me-cli/tests/cross_lang.rs:21` (+ `golden.rs:6`).

**Verdict: REFUTED as a funds-safety finding.** Confidence: high.

The literal observation (the two *anchor* test files are thin) is factually true,
but the moderate severity rests entirely on a failure scenario that is **not
reachable with inputs that pass upstream validation**. As a funds-safety item it
does not stand; at most it is a low-value defense-in-depth test-hygiene nit.

---

## What is literally true (the observation)

- `cross_lang.rs:21` round-trips a single fixed input `"md1yqpqqxqq8xtwhw4xwn4qh"`
  through the Go reader. Confirmed. (Note: this input is **24** text bytes, not 23
  — golden vector is `03 1d d1 01 19 54 00 <24 bytes> fe`, `0x19`=25=1+24 — a minor
  mislabel in the finding, immaterial.)
- `golden.rs:6` byte-pins that same one vector. Confirmed.
- Those two files contain no length sweep, no 249/250 SR-boundary case, no mk1
  vector. Confirmed.

So the coverage-breadth statement about *those two files* is accurate.

## Why the funds-safety claim is not substantiated

The finding's severity hangs on ONE concrete failure scenario (its own words):

> a future edit changes `tlv_wrap`'s guard from `>= 0xFF` to `> 0xFF` (accepting
> `msg.len()==255`), so a 250-char text emits a TLV whose 1-byte length field
> wraps/misparses; the single 23-char test still passes; a wrong/truncated plate
> ships.

The record layout is `msg.len() = 5 + text.len()` (header+typelen+plen+type+status
+text, `ndef.rs:36-41`). Relaxing `tlv_wrap` (`ndef.rs:48`) from `>= 0xFF` to
`> 0xFF` newly accepts exactly `msg.len() == 255`, i.e. **`text.len() == 250`**,
which emits a `03 FF ..` TLV — the 0xFF 3-byte-length escape — and would misparse.
That mechanism is real. **But `text.len() == 250` is unreachable by valid input:**

1. **Validation runs before encoding.** `convert()` (`lib.rs:62-63`):
   ```
   validate::validate(fmt, s).map_err(ConvertError::Validate)?;
   ndef::encode_text_tlv(s).map_err(ConvertError::Ndef)
   ```
   Any string that is not an admissible md1/mk1 is rejected *before* the encoder is
   ever called.

2. **Admissible md1/mk1 is far shorter than 250.** `validate()` delegates to
   `md_codec::codex32::unwrap_string` (Md) and `mk_codec::string_layer::decode_string`
   (Mk) (`validate.rs:43-51`). The finder's own round-0 report verified against the
   codec source that a valid single md1 string is ≤ ~80 chars and a valid mk1 is
   ≤ ~111 chars (chunked descriptors → each chunk is a separate short string). No
   admissible single string approaches 250. `encode_text_tlv` is only ever fed one
   validated string (sole external caller `main.rs:101` → `convert`; the bundle path
   carries the verbatim per-chunk string, it does not concatenate before encoding).

Therefore, even if a future edit relaxed the `tlv_wrap` guard exactly as described,
**no valid md1/mk1 input could produce the 250-char text needed to trigger it** —
`validate()` rejects such input first. The corrupting path is doubly dead: the
input is not a valid md1/mk1 (rejected at validate), and it never reaches the
boundary the regression would open. No wrong-but-accepted plate can ship from valid
input. The finder itself states this in its own report ("The refuse-at-249 path is
actually UNREACHABLE for valid input") and proved the transport correct across
`0..=300`.

The broader hedge ("*a* length-dependent regression would pass CI") is a generic
test-breadth argument, not a demonstrated funds-safety defect. Most length-
independent encoder regressions (wrong header, off-by-one in plen/typelen, terminator)
would corrupt the pinned 24-char golden and the round-trip too, and are caught. A
regression that specifically spares length 24 yet corrupts another *reachable*
length (0..~111) is contrived and unnamed; the only concrete scenario offered lives
at 250, behind the validation cap.

## Existing coverage is also broader than the finding implies

Beyond the two anchor files, the encoder path already has:
- `ndef.rs::round_trips` — encode+decode round-trip on an **mk1** string
  (`"mk1qpzry9x8gf2tv"`), refuting "no mk1 vector" for the encoder itself.
- `ndef.rs::encodes_expected_bytes` — byte-exact encode assertion.
- `ndef.rs::rejects_oversize` — `text_record` rejects `text.len()==255` (`TooLong`).
- `lib.rs::converts_md1_to_ndef` (round-trip), `refuses_ms1`, `flags_plate_overflow_risk`.
- `validate.rs::accepts_valid_mk1`.

The specific tlv-boundary (249 accept / 250 reject) is not directly unit-pinned —
that is the only genuinely missing edge, and it guards an unreachable input.

## Severity assessment

Funds-safety impact: **none reachable.** No path from a valid md1/mk1 to a
wrong-but-accepted or truncated plate exists, with or without the hypothesized
regression, because validation caps admissible length ~2.2x below the boundary and
runs first. Adding boundary + mk1 golden tests is a reasonable low-value
defense-in-depth improvement, but it is not a moderate funds-safety gap. For a
funds-safety audit this is refuted.

## Notes on method

- Could not run `cargo test` live: the session task tmpfs was full (ENOSPC on
  `/tmp/claude-1000/.../tasks`), unrelated to the repo. Verdict rests on direct
  source reading (`ndef.rs`, `lib.rs`, `validate.rs`, both test files) plus the
  finder's own reproduced `0..=300` probe, which I did not need to re-run to
  establish reachability. The reachability conclusion depends only on the confirmed
  validate-before-encode ordering and the codec length caps, both source-verified.

**refuted = true; confidence = high.**
