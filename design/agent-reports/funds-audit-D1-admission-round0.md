# Funds-safety audit — Dimension D1 (input admission & validation)

Repo: `/scratch/code/shibboleth/mnemonic-engrave` · Rust CLI `me` (`crates/me-cli`)
Scope: `classify.rs`, `validate.rs`, `lib.rs`, `main.rs` (+ `bundle.rs`, `ndef.rs`, `preview.rs`, `manifest.rs` as they touch admission).
Codecs pinned: `md-codec 0.36.0`, `mk-codec 0.4.0` (Cargo.lock). Auditor cross-checked crate source in `~/.cargo/registry`.

Method: read every entry path (`convert`, `bundle`, `bundle --preview`) down to the `md_codec` / `mk_codec` call; traced error propagation; verified codec API contracts against crate source; built `me` and ran adversarial probes; built two scratch crates outside the repo to construct edge-case strings and cross-check them against a current codec (`md-codec 0.40.0`).

Bottom line: the **ms1 refusal is robust**, the **mk1 path is strict and sound**, and **no reorder/drop/dup/truncate** was found. Two real gaps, both **md1-only**, both **moderate**:
- **D1-1** — md1 admits internal codex32 visual separators (`-`, ASCII/Unicode whitespace, newline) that the codec strips before BCH-verify, then `me` engraves them **verbatim**: the emitted bytes are not covered by the checksum that "validated" the string.
- **D1-2** — the stale `md-codec 0.36` pin lacks the later over-length admission guard; `me` accepts a 94-symbol (over-domain) single md1 that the current codec rejects as `StringSymbolCountOutOfRange`.

---

## FINDING D1-1 (moderate) — md1 visual-separators/whitespace pass validation but are engraved verbatim (validated form ≠ emitted form)

**Where:** `crates/me-cli/src/lib.rs:56-64` (`convert`) → `validate::validate` (`validate.rs:43`, `md_codec::codex32::unwrap_string`) → `ndef::encode_text_tlv(s)` (`lib.rs:63`). Same shape on the bundle single-md1 path (`bundle.rs:131-151` → `manifest`/preview render the verbatim `s`).

**Root cause:** `md_codec::codex32::unwrap_string` (registry `md-codec-0.36.0/src/codex32.rs:131-141`) **skips** any `c.is_whitespace() || c == '-'` character before computing the BCH checksum ("tolerate visual separators per D11"). So the BCH check is performed on the *stripped* symbol sequence. But `me` validates and then encodes the **raw trimmed input** `s` — `lib.rs:57` does only `input.trim()` (outer whitespace), and `ndef::encode_text_tlv(s)` at `lib.rs:63` embeds `s` verbatim into the NDEF Text record. The characters the codec stripped are therefore engraved but were **never covered by the checksum** that reported the string "valid".

`validate.rs`'s own module doc claims the opposite guarantee: *"Confirms HRP + BCH checksum so a corrupted string is never engraved."* This finding shows bytes outside the checksum's coverage do get engraved.

**Proof (built `target/debug/me`):**
```
$ printf 'md1yqpqqxqq8xt-whw4xwn4qh' | me --hex   # exit 0
031ed1011a54006d643179717071717871713878742d7768773478776e347168fe
                                            ^^ 0x2d = '-' embedded in NDEF text
$ printf 'md1yqpqqxqq8xt whw4xwn4qh' | me --hex   # exit 0  (0x20 space embedded)
$ printf 'md1yqpqqxqq8xt\nwhw4xwn4qh' | me --hex  # exit 0  (0x0a newline embedded)
```
All three report success and emit an NDEF payload containing the separator/whitespace/newline byte. The `mk1` path is **not** affected — `mk_codec::string_layer::bch::decode_string` (registry `mk-codec-0.4.0/.../bch.rs:667-676`) rejects any non-alphabet char, so `mk1` with a `-` or space fails with `InvalidChar` (verified: exit 4). This gap is **md1-exclusive**.

**Failure scenario (funds-relevant):** A user's md1 policy string is stored/transmitted through a channel that inserts a soft-wrap hyphen or a mid-string newline (mail clients, PDFs, hand-written codex32 grouping). `me convert --in policy.txt` reports "validated" and engraves the plate **with** the stray separator. The BCH checksum bound the stripped form, not the engraved bytes, so `me` gave no warning. At recovery, if the coordinator/restore tool (or an updated SeedHammer md decoder) does not strip the separator exactly as `md-codec` does, the plate is undecodable — a silently unreadable backup. The newline sub-case is worse: a newline/control char cannot be faithfully engraved at all, so the resulting plate matches no canonical string. Note `me bundle` is line-oriented (`bundle.rs:178-182` splits on `.lines()`), so newlines are only reachable through the single-string `convert` path; dashes/spaces reach both.

**Not a critical:** the emitted string equals what the *user typed* (no substitution/reorder/drop), and `-` is a legal BIP-93 codex32 visual separator, so within the constellation's own tolerant parsers it round-trips. The exposure is the checksum-coverage gap plus non-strippable whitespace/newline.

**Fix direction:** engrave the **canonical form the codec actually validated** — i.e. after a successful `unwrap_string`, re-derive the canonical string (or strip `-`/whitespace from `s`) and encode that, so what is checksummed is exactly what is engraved; or reject any md1 whose raw chars differ from its canonical char set. Do not encode the raw input on the md1 path.

**Regression test (add to `lib.rs` tests):**
```rust
#[test]
fn md1_visual_separators_not_engraved_verbatim() {
    // md-codec strips '-'/whitespace before BCH-verify; me must not engrave bytes
    // the checksum never covered — either reject, or engrave the canonical form.
    for bad in ["md1yqpqqxqq8xt-whw4xwn4qh",
                "md1yqpqqxqq8xt whw4xwn4qh",
                "md1yqpqqxqq8xt\nwhw4xwn4qh"] {
        match convert(bad) {
            Err(_) => {}                       // acceptable: reject non-canonical
            Ok(ndef) => {
                let text = ndef::decode_text_tlv(&ndef).unwrap();
                assert!(!text.contains(['-', ' ', '\n', '\t']),
                    "engraved payload carries a non-checksummed separator: {text:?}");
            }
        }
    }
}
```
(Fails today: `convert` returns `Ok` and the decoded text contains the separator.)

---

## FINDING D1-2 (moderate) — stale `md-codec 0.36` pin admits over-length (out-of-domain) single md1 that the current codec rejects

**Where:** `crates/me-cli/src/validate.rs:43` (`md_codec::codex32::unwrap_string` on the `convert` path) and `bundle.rs:132` (same call on the bundle md1 path). Version pinned in `crates/me-cli/Cargo.toml` (`md-codec = "0.36"`) / `Cargo.lock` (`0.36.0`).

**Root cause:** `md-codec 0.36`'s `unwrap_string` (registry `md-codec-0.36.0/src/codex32.rs:113-161`) has **only a lower-bound** length check (`symbols.len() < REGULAR_CHECKSUM_SYMBOLS`) and no upper bound. Later md-codec releases add an explicit fail-closed guard — from `md-codec-0.40.0/src/codex32.rs`:
```
// cycle-4 I1 (§5.2.3): reject an over-93-symbol codeword BEFORE the
// length-agnostic BCH verify. A clean (residue==0) over-length word is
// BCH-verifiable but structurally out-of-domain for the regular code
// (β has order 93) ... fail-closed so a non-correcting decode cannot
// accept an out-of-domain payload.
if symbols.len() > REGULAR_CODE_SYMBOLS_MAX { return Err(StringSymbolCountOutOfRange{..}); }
```
i.e. this is a *known-fixed validation defect*: the codex32 regular code is BCH(93,80,8); beyond 93 symbols the generator `β` aliases and the BCH check no longer guarantees error detection. `me` links the pre-fix `0.36`, so its md1 admission is laxer than the current codec contract. (`mk-codec 0.4.0` is **not** affected — `decode_string` calls `bch_code_for_length(data_part.len())` and returns `InvalidStringLength` for any non-exact length; `bch.rs:663-664`.)

**Proof (scratch crates outside the repo):** built an 81-data-symbol (94-total) md1 with `md-codec 0.36`'s own `wrap_payload` (which also lacks the cap), then fed the identical string to both `me` and a `md-codec 0.40` checker:
```
string (97 chars, 94 symbols): md15kj6tfd9...5zfqq6yyhmu3j8
$ printf '%s' "$OVERLEN" | me --hex        -> exit 0 (emits NDEF)   # md-codec 0.36
0.40 unwrap_string          -> REJECTED: StringSymbolCountOutOfRange { symbols: 94, max: 93 }
```

**Failure scenario (funds-relevant):** `me`'s contract is to refuse invalid/corrupted input. A hand-crafted or mis-encoded single md1 longer than 93 symbols is structurally out-of-domain, yet `me` admits it, reports "validated", and emits/engraves it. Because the code is undefined past 93 symbols, the BCH check gives no detection guarantee in this regime, and an updated SeedHammer md decoder (carrying the same I1 guard) would reject the plate on readback — an unreadable backup produced with a green "valid" from `me`. Requires abnormal input (the constellation always chunks payloads that would exceed a single string), which is why this is moderate rather than critical, but it is exactly the "stale pin with known-fixed validation behavior" class the audit calls in-scope.

**Fix direction:** either bump `md-codec` to a release carrying the H6/I1 caps (mind the provenance-pin rule — verify encode/identity bytes are unchanged), or add an `me`-side guard rejecting any md1 whose codex32 symbol count exceeds 93 (data+checksum) before trusting `unwrap_string`.

**Regression test (add to `lib.rs`/`validate.rs` tests):**
```rust
#[test]
fn rejects_over_length_single_md1() {
    // 81 data + 13 checksum = 94 symbols > 93 (BCH(93,80,8) domain). Out-of-domain;
    // current md-codec rejects with StringSymbolCountOutOfRange.
    let bits = 81 * 5;
    let bytes = vec![0xA5u8; (bits + 7) / 8];
    let overlen = md_codec::codex32::wrap_payload(&bytes, bits).unwrap();
    assert!(convert(&overlen).is_err(),
        "me must refuse an over-93-symbol single md1 string");
}
```
(Fails today: `convert` returns `Ok`.)

---

## Checked and found SOUND (negative results)

1. **Checksum enforced on every entry path.** `convert` → `validate::validate` before `encode_text_tlv` (`lib.rs:62-63`); `bundle` → `parse_line` calls `validate::validate` before any reassembly (`bundle.rs:101`), then `md_codec::chunk::reassemble` / `mk_codec::decode` on the grouped set; `bundle --preview` reuses `run_bundle` then only renders already-validated public `plate.string`s (`main.rs:271-294`). No path reaches `encode`/render/manifest without a prior `validate` returning `Ok`. Every codec error is propagated via `map_err`/`?` — none is `unwrap_or`'d, defaulted, or swallowed (`validate.rs:43-52`, `bundle.rs:96-167`).

2. **mk1 non-pristine refusal is real.** `validate` rejects `decode_string` results with `corrections_applied != 0` (`validate.rs:46-51`); `DecodedString.corrections_applied` is authoritative in `mk-codec 0.4.0` (`bch.rs:396-484,686`). A single flipped mk1 symbol is BCH-*correctable* but `me` refuses it (probe: exit 4, `MkCorrected`). md1 uses `unwrap_string`, a pure verify (no correction), so any corruption is rejected outright (probe: `md1zzzzzzzz` → exit 4).

3. **ms1 refusal is robust.** `classify` trims + lowercases the HRP (`classify.rs:41-51`); `convert` refuses `Format::Ms` **before** validation (`lib.rs:59-61`); `run_bundle` does a classify-only ms1 pre-scan over *every* line before validating any (`bundle.rs:188-192`), and `parse_line` refuses ms1 again (`bundle.rs:97-99`). Probed refused (exit 3): lowercase `ms1…`, all-uppercase `MS1…`, whitespace-padded `  ms1…  `, dashed `ms1-…`, mixed-case `Ms1Qqpq`, and ms1 anywhere in a bundle. An ms1 secret **cannot** be reclassified as md/mk: its HRP is `ms`, and md/mk validation independently enforce HRP `md`/`mk` (`codex32.rs:124`, `bch.rs:659-661`), so ms material cannot leak onto an md/mk emit path.

4. **Classification first-`1` vs codec last-`1` does not create a bypass.** `classify` uses `find('1')` (first) while `md-codec` checks `starts_with("md1")` and `mk-codec` uses `rfind` (last) — but the codex32 data alphabet contains no `1`, so a valid md/mk/ms string has exactly one `1` and all three agree. Any second `1` (e.g. a smuggled `…ms1…` tail) is a non-alphabet char that fails the codec char-decode (`md1zzz…1…` → `InvalidChar`). No misclassification admits ms or a wrong HRP.

5. **Normalization does not silently alter the payload the user validated.** The only mutation is `str::trim` of the whole input (`lib.rs:57`, `bundle.rs:95/180`), which removes outer whitespace that was never part of the bech32 token; HRP lowercasing in `classify` is match-only and does not rewrite `s`. The exact `s` that is validated is the exact `s` that is encoded (`lib.rs:62-63`) — no reorder/drop/dup/truncate. (The one wrinkle is internal separators/whitespace, D1-1 above, where the *codec* — not `me` — mutates before checksumming.)

6. **No success-on-failure and no truncation.** Every malformed input probed exits non-zero with no stdout payload: empty/whitespace/`no separator`/`leading 1`/unknown HRP → exit 4; bad md1/mk1 checksum/length → exit 4. NDEF encoding caps at 254 text bytes and returns `TooLong` (→ exit 4) rather than truncating (`ndef.rs:30-42`); a valid single md1 (≤93 chars) / mk1 (≤~108 chars) is well under the cap, so no valid string is truncated.

7. **`exceeds_plate_budget` warning is inert but harmless.** `PLATE_TEXT_BUDGET = 300` (`lib.rs:46`) exceeds the NDEF 254-byte text cap, so any `convert` input long enough to trigger the warning already fails NDEF encoding with `TooLong` (exit 4) before the warning prints (`main.rs:99,119-133`). Dead-ish code, not a funds issue. `bundle` does not budget-check, but each plate is a bounded single chunk and the preview sidecar / device backstop with `ErrTooLarge`.

## Areas explicitly NOT deep-audited here (belong to sibling dimensions)
- Chunk-set completeness/consistency semantics of `md_codec::chunk::reassemble` and `mk_codec::decode`, and the single-vs-chunked md1 discriminator in `bundle.rs:132-167` (the `ChunkHeader::read` DEVIATION) — integrity/reassembly dimension.
- Secret-scrub / argv-env-tempfile leakage (the `Zeroizing` handling in `main.rs`, preview image residue) — the secret-exposure dimension.
- The Go preview sidecar's own layout/QR/glyph mapping and preview-vs-device faithfulness — the sidecar dimension.

## FOLLOWUPS cross-check
Read `design/FOLLOWUPS.md`. Neither D1-1 nor D1-2 is an already-filed item; the closest is `me-decode-text-tlv-comment` (a resolved doc-only nit about `decode_text_tlv` scope), which is unrelated to admission. No `knownFollowup` overlap.
