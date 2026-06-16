# Spec-stage architect review — R0 — `mnemonic-engrave` / SeedHammer II engrave

- **Stage:** brainstorm/spec R0 (pre-plan, pre-implementation)
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect` (adversarial brief: verify load-bearing claims against source, find holes, challenge the upstream-PR assumption)
- **Spec under review (at time of R0):** the brainstorm-converged design (now `design/SPEC_seedhammer_engrave.md`)
- **Verdict:** SOUND-WITH-CHANGES. All Critical/Important findings folded into the spec (see disposition at end). **R1 required** to confirm convergence to 0C/0I per the family iterative-architect-review standard.

---

## VERBATIM REVIEW OUTPUT

I now have enough information to write the full adversarial review.

---

### 1. Verified / Refuted Claims

**Claim 2 — NDEF path: VERIFIED WITH IMPORTANT NUANCE**

The NDEF reader at `nfc/ndef/ndef.go:175-201` (lines 182-201 in the actual file) correctly: checks TNF=WellKnown (line 175-179), reads the `'T'` type byte, reads the status byte, rejects UTF-16 (bit 7 of status byte, line 189), discards `langLen` language bytes (lines 193-200), and returns the UTF-8 body.

However, the claim is imprecise about the NDEF framing layer. The `MessageReader` at lines 29-85 expects **TLV-wrapped** NDEF messages (Type=0x03, the NDEF TLV block), with a null-terminated TLV stream. This is the NDEF over T2T (Type 2 Tag) / T5T (Type 5) memory layout — it is NOT plain NDEF message bytes. For the Type 4 tag emulator path (phone pushes data to device), the `type4.go` implementation exposes a raw NDEF file, and `poller.go:85-88` feeds the Type 4 emulator output directly through `ndef.NewRecordReader` WITHOUT the `MessageReader` TLV wrapping layer, but it DOES pass passive tag reads through `NewMessageReader` first (lines 83-84). The converter's NDEF output format must match one of these two paths — this is the principal framing risk.

For a **phone-as-writer pushing to the device acting as T4T emulator**: the phone writes raw NDEF message bytes (MB/ME flags, type length, payload length, etc.) directly into the NDEF file. The `poller.go` reads the emulator's output via `bufr` → `NewRecordReader`, skipping the TLV wrapper. A converter that emits a raw NDEF message (with correct record framing) to be NFC-written is correct for this path.

For a **passive tag that the device reads**: the device reads T2T/T5T memory, which requires the TLV wrapper (0x03, length, NDEF message bytes, 0xFE terminator) per the NFC Forum specs. The converter outputting only an NDEF message without TLV wrappers would be **silently ignored** on passive tag reads, since `MessageReader` would see no recognized TLV block type.

**Verdict**: The claim is correct about what the Text record must look like, but the NDEF-message-vs-TLV distinction is unaddressed in the design. The converter must document which path it targets and emit accordingly. This is a real defect in the proposal.

**Claim 3 — Scanner parser order, md1/mk1 rejection: VERIFIED WITH A HIDDEN RISK**

Scan order confirmed at `gui/scan.go:59-69`: `bip39.Parse` → `nonstandard.OutputDescriptor` (→ `bip380.Parse`) → `codex32.New`.

Whether md1/mk1 strings reach `codex32.New` and fail there:
- `bip39.Parse` splits on spaces. md1/mk1 strings contain no spaces; the first "word" (the whole string) is not in the BIP-39 word list. Fails immediately. Safe.
- `nonstandard.OutputDescriptor` → `bip380.Parse` looks for a `(` character in the string (line 280 of `bip380.go`). md1/mk1 strings use only bech32 alphabet characters `[qpzry9x8gf2tvdw0s3jn54khce6mua7l]` plus the separator `1`. No `(` is present. Fails at `bip380.go:289`. Then `parseBlueWalletDescriptor` fails (no `\n` or `: ` separators). Then `json.Unmarshal` fails. Then `ParseKey` is tried — this attempts to parse an xpub. An xpub starts with `xpub` or `tpub`, not `mk1` or `md1`. Fails. Safe.
- `codex32.New` at `codex32.go:98`: length check first. md1 single-string: 91+3 = ~94+ chars (HRP `md` + sep `1` + data). The codex32 length check is `48 ≤ len ≤ 93` for short code, `125 ≤ len ≤ 127` for long code. **md1 and mk1 single-string encodings are 91–111 chars total** (HRP `md`/`mk` = 2 + 1 separator + data part 14–108 chars). A mk1 chunk with a long-code data part of 93 chars would be 3+93=96 total chars, which is **outside both the 48-93 short range AND the 125-127 long range** of codex32.New. It would fail at `errInvalidLength`. But a mk1 regular-code string (data part 14-93 chars → total string 17-96 chars) with a data part at exactly the right length (48-93) could have the correct *total length* to pass the codex32 length gate.

**Specifically**: a regular-code mk1 string has data part 14-93 chars, so total length = 3 + 14-93 = 17-96. Lengths 48-93 of the total string would be data-part lengths 45-90, all within the regular-code valid range for codex32. Such strings pass the length gate. They then go through `check.inputHRP("mk")` (the `splitHRP` splits on first `'1'`, giving HRP=`mk`), then `check.inputData(data_part)`. All bech32 characters pass `feFromRune`. Then `check.isValid()` fails because the BCH checksum was computed with a different target residue (`MK_REGULAR_CONST` vs codex32's `SECRETSHARE32` target). The checksum validation fails. So `codex32.New` correctly rejects md1/mk1.

However: a longer md1/mk1 string (total length 48-93 as noted) will spend CPU computing the BCH checksum before rejecting. More importantly, by the probability of a random BCH collision, the probability is negligible. **This is safe**.

There is no path to a `*bip380.Descriptor` false-positive. Claim 3 is confirmed.

**Claim 4 — codex32.New accepts ms1, threshold 0 enforces share index S: VERIFIED**

`codex32.go:98-124`: length gate OK (ms1 = `ms1` + 45-90 chars = 48-93 chars, valid). `splitHRP` yields HRP=`ms`, data starts with threshold char. At `partsInner` line 169: `if ret.threshold == 0 && ret.shareIdx != feS { return nil, errInvalidShareIndex }`. BIP-93 ms1 has threshold `0` and share index `S`, so this passes. Confirmed.

For `inputCodex32Flow` at `gui/gui.go:623-682`: the keyboard alphabet at line 624 is `"1234567890\nqwertyup\nasdfghjk\nlzxcvnm"`. [Reviewer worked through the rows and confirmed all 32 bech32 characters plus digits are represented; the keyboard supports the full bech32 alphabet for codex32 entry. `inputCodex32Flow` is fully implemented.]

**Critical sub-claim: uncommenting the string suffices.** At `gui/gui.go:1806`: `Choices: []string{"12 WORDS", "24 WORDS" /* , "CODEX32", "SLIP-39" */}`. And at `case 2:` (line 1820): the codex32 flow is fully wired to `inputCodex32Flow`. Uncommenting `"CODEX32"` restores choice index 2, which already routes correctly. The `case 3:` block (SLIP-39) is also commented out; the structure remains consistent. Claim 5 is **confirmed** — uncommenting the menu string is sufficient.

**Important flag**: `backup.EngraveSeedString` at `backup/backup.go:76` calls `strings.ToUpper(plate.Seed)` before QR encoding. So ms1 would be uppercased and engraved uppercase. The codex32 spec (BIP-93) accepts either case. Confirmed correct for ms1 (bech32 is case-insensitive).

**Claim 6 — Portable Go / Unlicense / RP2350 secure boot: PARTIALLY VERIFIABLE**

The `cmd/controller` gating is confirmed by examining the file tree. The Unlicense claim cannot be fully verified from code alone but is consistent with repository headers. The OTP fusing observation is accurate per `driver/otp/otp_rp2350.go` but unknowable for retail units without physical access.

---

### 2. Architectural Risks (Ranked by Severity)

**CRITICAL: NDEF Framing Boundary — The Design Has No Concrete Answer**

The proposed design says emit "an NDEF message with one well-known Text record as raw .ndef bytes." This is under-specified and will cause silent failures. The device reads NFC via two distinct paths in `poller.go`:

Path A (passive tag): `poller.go:83-84` wraps raw tag memory in `ndef.NewMessageReader` (the TLV reader) before `NewRecordReader`. The TLV reader (`ndef.go:36-85`) expects the NFC Forum T2T/T5T memory layout: `0x03` (NDEF TLV type), a length byte (or 3-byte extended), the raw NDEF message bytes, then `0xFE` (terminator). Raw NDEF message bytes without this TLV wrapper will be skipped/failed.

Path B (phone→T4T emulator): `poller.go:85-88` skips the TLV layer and feeds emulator output directly to `NewRecordReader`. The T4T NDEF file contains only the raw NDEF message (with a 2-byte length prefix that `type4.go:264-272` strips). Raw NDEF message bytes are correct for this path.

**Consequence**: the converter's `.ndef`/hex/base64 output is only correct for Path B. Writing those bytes to a passive sticker silently yields "Unknown format." **Recommendation**: emit the **TLV-wrapped form** by default (what commodity NFC writer apps produce for passive tags); document the byte format with a test vector.

**HIGH: mk1 Is Always Multi-String — The NDEF Design Only Handles One String**

From `mnemonic-key/crates/mk-codec/src/string_layer/pipeline.rs` (comments/tests around line 164-168): "no realistic mk1 card fits in a single string — SingleString remains reachable only through hand-constructed sub-card test inputs." A typical mk1 card produces **2 chunked strings**. The design ("one NDEF Text record") handles only single-string mk1, which never occurs in practice. The firmware `gui/scan.go` processes the buffer as a single string with no multi-part reassembly. Either mk1 multi-chunk needs firmware-side assembly, a different delivery strategy, or the design must explain chunk combination.

**HIGH: The Firmware Patch Adds an Opaque Bypass — No Format Validation at Scan Time**

Routing md1/mk1 to `backup.EngraveText` as opaque text performs no BCH verification: a bit-flipped or truncated string is engraved verbatim; the engraved QR encodes a corrupted string. The existing paths all parse-and-validate before engraving. The patch breaks this invariant.

**MEDIUM: False-Positive Risk in HRP-Prefix Recognition**

Any payload beginning `md1`/`mk1` (case?) would route to engrave instead of dropping as unknown. Must be case-insensitive; the proposal didn't specify.

**MEDIUM: Plate Capacity for mk1 Chunk Strings**

`EngraveSeedString` (`backup/backup.go:96-158`) engraves 10 chars/line, 16 lines col1, plus a QR. A long-code chunk is 111 chars; QR ~version 6-7. Likely fits on 85×85mm but needs physical measurement.

**MEDIUM: Version Coupling Between mnemonic-engrave and Sibling Crates**

Converter depends on md-codec/mk-codec/ms-codec. Any BCH/HRP/chunking change requires coordinated updates. Needs a concrete versioning policy.

**LOW: Scanner Buffer Is 8KB** — multi-record mk1 (~207-222 bytes) is well within 8KB. Not a real risk.

---

### 3. Security Review

**argv vs stdin: A Real Exposure** — passing ms1 via argv exposes it in `/proc/pid/cmdline`, `ps aux`, shell history, clipboard/monitoring agents. The refuse-ms1 mitigation happens AFTER the string is already in argv. mlock/zeroize cannot scrub OS-held argv copies. **Mitigation: never accept secrets via argv; stdin only.**

**ms1 Detection Safety** — zeroize on a plain `String` doesn't scrub argv copies. Low residual risk for a refused ms1, but should not be documented as "safe."

**md1/mk1 Threat Model: Confirmed Correct** — descriptor + xpubs are public; NFC transport is equivalent to QR scanning. Sound.

**Phone as NFC Intermediary** — a compromised phone exposes md1/mk1 (public-key artifacts → wallet-privacy loss, not theft). Correctly a non-goal vs ms1.

**No Privilege Escalation Path** — firmware exposes no network/USB-data interface; the patch adds no remote attack surface.

---

### 4. Upstream-PR Viability

**Skeptical Assessment: Low Probability of Acceptance As Described.** "Engrave opaque non-standard text" is antithetical to SeedHammer's validate-before-engrave philosophy. Maintainers would prefer the firmware either properly decode/validate md1/mk1 or refuse cleanly. The `"CODEX32"` uncomment is much more likely to be accepted (trivially safe, clearly intended).

**More Palatable Framing**: implement a BCH verifier for the md1/mk1 target constants in Go (~150 lines) on top of the existing GF32 machinery in `codex32/checksum.go` + `codex32/gf32.go` (same polynomial, different target residue). Pitch: "expose codex32 entry + add BCH-validated md1/mk1 engrave" — both items with proper validation.

**Fallback Cost**: private fork is trivially maintainable for this change (self-contained `gui/scan.go` + routing). Must track upstream security patches. Low overhead, non-zero supply-chain risk.

---

### 5. Simpler / Alternative Approaches

**A)** Use UR/animated-QR — no camera on device; path doesn't exist. Rejected.
**B)** Translate md1 → standard BIP-380 descriptor (no md1 firmware patch) — loses compact encoding, longer strings, doesn't help mk1, defeats the constellation. Rejected.
**C)** Implement md1/mk1 BCH verification in Go (configurable target residues) — proper validation, not opaque text. The right long-term approach.
**D)** Stateful multi-scan assembly (like `nonstandard.Decoder` for animated QR) for mk1 chunks — clean but non-trivial.

---

### 6. Concrete Recommendations (before writing the spec)

1. **Resolve the NDEF framing ambiguity now** — use the **T2T TLV-wrapped form** (`0x03, length, NDEF message, 0xFE`); document with a test vector.
2. **Redesign mk1 delivery** — mk1 is 2+ strings; design the multi-chunk delivery (multi-record NDEF + firmware reassembly, or sequential multi-scan with reassembly) before writing the converter.
3. **Add BCH validation at the firmware for md1/mk1** (or reject opaque-text). Use the HRP-mixed target constants (MD/MK), not codex32's target.
4. **Prohibit argv input** — stdin only; document why.
5. **`"CODEX32"` uncomment as a separate PR, submitted first.**
6. **Test the plate layout with a real mk1 string before committing the design** (physical).
7. **Verify HRP-prefix detection handles uppercase** (case-insensitive); md1/mk1 are lowercase but `EngraveSeedString` uppercases.

---

## FOLD DISPOSITION (added by main session; not part of verbatim review)

Severity mapping to family C/I/M: CRITICAL→Critical; HIGH→Important; MEDIUM→Important/Minor; LOW→Minor.

| # | Finding | Sev | Disposition |
|---|---|---|---|
| 2.1 | NDEF framing ambiguity (TLV vs raw) | **Critical** | FOLDED — spec §6 commits to TLV-wrapped T2T form as canonical, documents T4T `--no-tlv` variant, mandates a golden byte test vector. |
| 2.2 | mk1 always multi-string | **Important** | FOLDED/REFRAMED — spec §4 "Per-string model & chunking": engraving is per-string, no on-device reassembly; multi-chunk card = multiple plates (bundle). v1 per-string converter already fits. (Architect's "redesign delivery" partially declined: reassembly is recovery-time, not engrave-time — documented rationale.) **Flag for R1 to confirm this reframe is sound.** |
| 2.3 | Opaque bypass / no scan-time validation | **Important** | FOLDED — spec §7 PR2: Go BCH verifier reusing the generic `engine` (generator/residue/target) with md/mk residues; BCH-fail → "unknown format". |
| 2.4 | HRP false-positive recognition | **Important/Minor** | FOLDED — spec §7: case-insensitive HRP detect + BCH validation rejects non-matching strings. |
| 2.5 | Plate capacity mk1+QR | **Minor** | FOLDED as hardware-verify item — spec §10.2. |
| 2.6 | Sibling crate version coupling | **Important/Minor** | FOLDED — spec §11: pin versions + BCH parity test guards drift. |
| 2.7 | 8KB buffer | Minor | No action (not a risk); noted. |
| 3.1 | argv secret exposure | **Important** | FOLDED — spec §3 + §5: stdin/file only, never argv; stderr for human text. |
| 3.2 | zeroize false-sense | Minor | FOLDED — spec §5 notes mlock cannot scrub argv → stdin-only is the real mitigation. |
| 4 | Upstream-PR viability low | **Important** | FOLDED — spec §7 two-PR split (CODEX32 first), BCH-validated framing; §14 risk + fork fallback. |
| 5C | Adopt BCH-validation approach | — | ADOPTED — spec §7 PR2. |
| 5D | Multi-scan reassembly | — | OUT OF SCOPE v1 — per-string model; reassembly is recovery-time (Rust). |
| 6.1–6.7 | Recommendations | — | All folded (see rows above + §6/§10 of spec). |

**Net:** 0 findings left open in the spec; reframe at 2.2 is the item R1 must stress-test. Per standard, R1 (re-dispatch on the revised spec) is required to reach GREEN.
