# Adversarial verification — D1-1 (md1 visual-separators/whitespace/newline engraved verbatim, outside checksum coverage)

Verifier: adversarial #0. Goal: refute if not concretely substantiable. **Verdict: CONFIRMED (not refuted).** Severity **moderate** upheld. Confidence **high**.

## Claim under test
`md_codec::codex32::unwrap_string` strips `-`/whitespace before the BCH check, but `convert` (lib.rs) validates then encodes the *raw trimmed* input `s`, so separator/whitespace/newline bytes are engraved despite never being covered by the checksum that reported the string valid. md1-only; mk1 rejects these chars.

## Code trace (verified against actual source)
- `crates/me-cli/src/lib.rs:56-64` `convert`: `let s = input.trim();` (line 57) → `classify` → refuse `Ms` → `validate::validate(fmt, s)` (line 62) → `ndef::encode_text_tlv(s)` (line 63). The **exact same `s`** that is validated is the one encoded. Only outer whitespace is stripped (`str::trim`); internal chars are untouched. **Confirmed.**
- `crates/me-cli/src/validate.rs:43` md path: `md_codec::codex32::unwrap_string(s).map(|_| ())` — a pure verify; its return value (canonical/stripped payload) is **discarded** (`.map(|_| ())`), so no canonicalization is fed back to the encoder. **Confirmed.**
- `~/.cargo/registry/.../md-codec-0.36.0/src/codex32.rs` `unwrap_string`, step 2 "Char-to-symbol decode (tolerate visual separators per D11)":
  ```rust
  for c in symbols_str.chars() {
      if c.is_whitespace() || c == '-' { continue; }   // stripped BEFORE BCH-verify
      let sym = char_to_symbol(c).ok_or(...)?;
      symbols.push(sym);
  }
  // 3. BCH-verify on `symbols` (the stripped sequence)
  ```
  So the BCH residue is computed over the stripped symbol stream, not over `s`. **Confirmed.** Pin is `md-codec 0.36.0` (Cargo.lock).
- `crates/me-cli/src/ndef.rs:60-62` `encode_text_tlv` → `text_record(text)` embeds `text.as_bytes()` verbatim into the NDEF Text record. No filtering. **Confirmed.**
- `validate.rs:1-2` module doc: *"Confirms HRP + BCH checksum so a corrupted string is never engraved."* The emitted form (with stray byte) is not the checksummed form — the stated guarantee is defeated for these bytes. **Confirmed.**

## Probe (built `target/debug/me`, ran the finder's exact inputs)
```
printf 'md1yqpqqxqq8xt-whw4xwn4qh' | me --hex  -> exit 0  text=b'md1yqpqqxqq8xt-whw4xwn4qh'  (0x2d embedded)
printf 'md1yqpqqxqq8xt whw4xwn4qh' | me --hex  -> exit 0  text=b'md1yqpqqxqq8xt whw4xwn4qh'  (0x20 embedded)
printf 'md1yqpqqxqq8xt\nwhw4xwn4qh'| me --hex  -> exit 0  text=b'md1yqpqqxqq8xt\nwhw4xwn4qh' (0x0a embedded)
printf 'md1yqpqqxqq8xtwhw4xwn4qh'  | me --hex  -> exit 0  text=b'md1yqpqqxqq8xtwhw4xwn4qh'   (clean, 1 byte shorter: TLV len 0x1d vs 0x1e)
printf 'mk1qpzry-9x8gf2tv'         | me --hex  -> exit 4  "invalid mk1 string: invalid character - at position 5"
```
Decoding the NDEF payloads confirms the stray byte is carried into the emitted Text record (tlvlen 0x1e for the three tainted cases vs 0x1d for the clean canonical). The mk1 dash case rejects (exit 4), confirming md1-exclusivity as claimed.

Every element of the finding reproduces exactly: cited location behaves as described; failure scenario is reachable with inputs that *pass* validation (exit 0); no downstream layer (validate, ndef, main) intercepts it.

## Refutation attempts (all failed)
- *Does another layer strip/canonicalize before encode?* No. `validate` discards the codec's canonical output (`.map(|_| ())`); `encode_text_tlv` embeds `s` verbatim. No canonicalization anywhere between `trim` and the NDEF bytes.
- *Is the input actually invalid (so refusal, not admission)?* No — `unwrap_string` returns `Ok` because it strips the separators before BCH-verify; `me` exits 0.
- *Is the pin wrong / does a newer codec behave differently on this path?* Irrelevant to D1-1: even the newest `unwrap_string` still tolerates `-`/whitespace (that is the intended BIP-93 behavior); the gap is on the `me` side (encoding raw `s`), not the codec's.

## Severity assessment (moderate — honest, upheld)
- **Not critical:** `me` engraves the user's *own* bytes — no substitution/reorder/drop. For `-`, ASCII space, and tabs, these are legal BIP-93 codex32 visual separators; any conformant md decoder (including md-codec itself) strips them and recovers the correct payload, so the common case round-trips and funds remain recoverable.
- **Above low:** it defeats a *stated* safety guarantee (validated form ≠ emitted form; checksum-coverage gap), and there are two concrete adverse paths: (a) a non-tolerant downstream reader / updated device decoder that does not strip exactly like md-codec → unreadable plate reported "valid"; (b) the newline/control-char sub-case, which cannot be faithfully engraved (no glyph) and yields either a canonical-by-accident drop or a tofu/placeholder corruption depending on the engraver — non-deterministic and unverified. Newline is reachable only via single-string `convert` (bundle splits on `.lines()`), narrowing but not eliminating it.

Moderate is defensible and honest. It is arguably at the generous edge (the dominant dash/whitespace cases round-trip through spec-conformant parsers), but the combination of a violated documented guarantee plus the non-engravable-char path keeps it above "low." No adjustment.

## Verdict
refuted = **false** (CONFIRMED). confidence = **high**. adjustedSeverity = none (moderate upheld).
