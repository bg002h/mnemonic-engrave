# Verify verdict — D1-2 (adversarial verifier #0)

- Finding: D1-2 — Stale md-codec 0.36 pin admits over-length (>93-symbol, out-of-domain) single md1 that the current codec rejects.
- Cited location: `crates/me-cli/src/validate.rs:43` (`md_codec::codex32::unwrap_string`), secondary `crates/me-cli/src/bundle.rs:132`.
- Verdict: **CONFIRMED (not refuted).** Severity **moderate** (unchanged — honest and appropriately conservative).
- Confidence: **high** (reproduced end-to-end against the actual prebuilt `me` binary and against both codec versions from source).

## What the finding claims
md-codec 0.36's `unwrap_string` has only a lower-length bound and no upper bound, so a residue-0 codeword longer than 93 symbols (BCH(93,80,8) domain max) is accepted. md-codec 0.40 added a fail-closed `StringSymbolCountOutOfRange` guard. `me` links 0.36, so `me convert`/`me bundle` admit an out-of-domain single md1 that the current codec rejects.

## Evidence gathered

### 1. Pin is real
- `crates/me-cli/Cargo.toml:22` → `md-codec = "0.36"`.
- `Cargo.lock:309-312` → `md-codec 0.36.0` (checksum `75b1bfb7…`). mk-codec pinned `0.4.0`.

### 2. Source inspection — the guard is present in 0.40, absent in 0.36
- `md-codec-0.36.0/src/codex32.rs` `unwrap_string` (lines 113-161): after BCH-verify, the ONLY length check is the lower bound `if symbols.len() < REGULAR_CHECKSUM_SYMBOLS` (line 151). No upper bound. `wrap_payload` (line 67) likewise has no data-symbol cap.
- `md-codec-0.40.0/src/codex32.rs` `unwrap_string` (line 139): adds `if symbols.len() > REGULAR_CODE_SYMBOLS_MAX { return Err(Error::StringSymbolCountOutOfRange { symbols, max }) }` (lines 174-177) BEFORE the BCH-verify path, with `REGULAR_CODE_SYMBOLS_MAX = 80 + 13 = 93` (lines 25-33). `wrap_payload` also gained a `data_symbols.len() > REGULAR_DATA_SYMBOLS_MAX` cap (lines 89-92). 0.40 even ships regression tests `unwrap_string_rejects_clean_over_93_symbol_string` / `unwrap_string_accepts_exactly_93_symbol_codeword`. This is a deliberate, known-fixed validation defect, exactly as the finding describes.

### 3. Cross-version probe (scratch crate outside the repo, `/var/tmp/overlen-probe`, depends on md-codec =0.36.0 and =0.40.0)
Built an 81-data-symbol (94-total) md1 with 0.36's own `wrap_payload`:
```
built md1: 97 chars, 94 symbols
0.36 unwrap_string: ACCEPTED (residue==0, no upper bound)
0.40 unwrap_string: REJECTED: StringSymbolCountOutOfRange { symbols: 94, max: 93 }
```

### 4. End-to-end against the actual prebuilt `me` binary (`target/debug/me`)
Fed the identical 97-char / 94-symbol string on stdin:
```
$ printf '%s' "$OVERLEN" | ./target/debug/me --hex
0366d1016254006d6431356b6a...357a667171367979686d75336a38fe   <- exit=0
```
`me convert` reports success and emits an NDEF payload embedding the over-length md1 (`6d6431 35...` = "md15…"). No other layer catches it: classify only checks the HRP; `validate()` only calls `unwrap_string`; NDEF cap is 254 bytes (97 < 254); `PLATE_TEXT_BUDGET`=300 (97 < 300). The over-length, structurally out-of-domain string is accepted and would be engraved with a green "valid".

### 5. Location accuracy
- `validate.rs:43` calls `md_codec::codex32::unwrap_string(s)` on the convert path — accurate.
- `bundle.rs:132` calls the same `md_codec::codex32::unwrap_string(s)` on the md1 bundle path — accurate.

## Refutation attempts (all failed)
- "Another layer prevents it": No. classify/NDEF-cap/plate-budget all pass a 97-char string; validate() delegates solely to `unwrap_string`. Reproduced exit 0 with NDEF emitted.
- "Not reachable": The string is constructible with 0.36's OWN `wrap_payload` (no cap), and once constructed it passes `me` verbatim. Reachability of the *input* is the only caveat (see below), not of the code path.
- "Severity inflated": The finder already downgraded from critical to moderate precisely because normal constellation tooling always chunks payloads and never emits a >93-symbol single md1, so the malicious/mis-encoded input is abnormal. That reasoning is sound and honestly disclosed.

## Severity assessment (kept at moderate)
- Real contract violation: `validate.rs`'s module doc promises "a corrupted string is never engraved" and `me`'s job is to refuse invalid/out-of-domain input. Past 93 symbols the BCH check has no error-detection guarantee (β aliases at the code-length boundary), so the "validated" claim is genuinely false in that regime — reproduced.
- Downgrade pressure (why not critical): reaching it requires a hand-crafted / mis-encoded over-length md1; the constellation encoder chunks and never produces one, and random corruption producing a residue-0 over-length word is astronomically unlikely. The concrete "unreadable backup" harm is also partly contingent on a *future* SeedHammer md decoder carrying the same I1 guard (a current 0.36-era decoder would round-trip it).
- Upgrade pressure (why not low): the repo's own provenance-pin / Rust-primary discipline treats stale-pin admission drift as in-scope, and the upstream codec explicitly fail-closed this exact case ("so a non-correcting decode cannot accept an out-of-domain payload"). Shipping the pre-fix codec is a real regression against the current codec contract.
- Net: moderate is the honest rating. Not adjusting.

## Conclusion
The finding is concretely substantiated at the cited location by source and by a working end-to-end probe against the shipped binary. refuted = false; severity moderate.
