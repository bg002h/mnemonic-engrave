# Spec-stage architect review — R1 — `mnemonic-engrave` / SeedHammer II engrave

- **Stage:** brainstorm/spec R1 (re-dispatch after R0 fold)
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect`
- **Spec under review:** `design/SPEC_seedhammer_engrave.md` (post-R0-fold)
- **Verdict:** **NOT-GREEN** — 0 Critical, 1 Important open, 3 Minor. (Folded below; R2 required to confirm GREEN.)

---

## VERBATIM REVIEW OUTPUT

I now have all the evidence. Let me verify one final thing: the spec §6 claim that SR=1 single-byte payload length "covers ~112-byte mk1 payloads." Max mk1 data part is 108 chars; with `01 54 00` overhead, payload = 1 (status) + up to ~111 text bytes = ~112, which fits in one byte (max 255). Confirmed. Also confirming the NDEF reader: `ndef.go:189` checks `header&(0b1<<7)` (bit7 = UTF-16); status `0x00` has bit7=0 → UTF-8, langLen=0. Confirmed correct.

I have enough to deliver the review.

---

# R1 Architect Review — `SPEC_seedhammer_engrave.md`

## 1. Per-finding status (R0 items)

**[Critical] NDEF framing** — **RESOLVED.** §6 byte layout (spec:86-99) verifies against `ndef.go`:
- TLV `03 <len> … FE`: matches `ndefType=0x03` / `termType=0xfe` (`ndef.go:252-253`), with 1-byte len and `FF`+2-byte BE escape (`ndef.go:71-80`). Correct.
- Record header `D1`: MB|ME|SR|TNF=001 = `0x80|0x40|0x10|0x01` = `0xD1`. Matches flag consts (`ndef.go:255-261`). CF=0, IL=0 correct.
- `01 <plen> 54 00`: tlen=1 read at `ndef.go:157-164`; wellKnown `'T'` at `:183`; status byte read at `:184`, bit7=UTF-16 test `:189`, langLen=`header&0b111111` `:194`. Status `0x00` ⇒ UTF-8, zero lang. Correct.
- SR=1 single-byte payload (`:138-145`): max mk1 data-part 108 chars ⇒ payload = 1+~111 = ~112 < 255. The "~112-byte mk1 payloads" claim (R0) holds.
- T4T-vs-passive split (spec:102): `poller.go:83` (`NewMessageReader`, TLV) vs `poller.go:85-88` (bare to `NewRecordReader`). Confirmed exactly.

**[Important] mk1 per-string reframe** — **RESOLVED, and sound.** Stress-tested:
- (a) Each chunk carries its **own** BCH checksum: `encode_5bit_to_string` appends a per-string 13/15-char checksum (`bch.rs:512-548`); `decode_string` validates one string in isolation (`bch.rs:645-690`). A single chunk validates without its siblings. Confirmed.
- (b) Ordering survives independent engraving: `chunk_index`/`total_chunks`/`chunk_set_id` live **inside the BCH-covered 5-bit data part** (`header.rs:67-101`), i.e. inside the verbatim engraved string — not as separate plate metadata. Recovery re-parses them from the string. Nothing lost.
- (c) "One NDEF record per string" is unambiguous: one string → one Text record (MB=ME=1).

**[Important] opaque-text bypass** — **RESOLVED.** (a) `engine` is genuinely generic over `generator`/`residue`/`target` (`checksum.go:11-18`); `newShortChecksum`/`newLongChecksum` are just constructors (`:29-68`); `isValid()` compares residue==target (`:72-74`). A new md/mk constructor is feasible. (b/c) md/mk constants exist and per-chunk validation is correct: `MD_REGULAR_CONST=0x0815c07747a3392e7`, HRP "md" (`md-codec/bch.rs:17`); `MK_REGULAR_CONST`/`MK_LONG_CONST`, HRP "mk" (`mk-codec/consts.rs:18,21`).

**[Important] argv exposure** — **RESOLVED.** §3 (spec:48) and §5 (spec:73,76,80) mandate stdin/`--in` only, binary→stdout, human→stderr. No residual argv path.

**[Important] upstream-PR viability** — **RESOLVED (judgment).** PR1/PR2 split (spec:107-117) + fork fallback (§14:159-161). BCH-validated framing reusing their own `engine` is a plausible upstream ask; risk honestly recorded.

**[Important/Minor] HRP false-positive / fit / version / zeroize** — **RESOLVED.** Case-insensitive + BCH (spec:115); plate fit deferred to HW (§10.2:134); parity test pins drift (§9:128, §11:139); zeroize caveat stated (spec:80).

## 2. New findings (fold-induced drift)

**[Important] — Representation mismatch glossed in §7/§9.** The spec says the Go `engine` is reused "by adding constructors with the md/mk generator + target residues sourced from `consts.rs`" (spec:114). But the two sides use **incompatible internal representations**: SeedHammer's `engine` stores `generator`/`target` as **GF32 coefficient vectors** (`[]fe`, `checksum.go:11-18,31-46`), whereas the Rust codecs store a **packed-u128 polymod** (`GEN_REGULAR: [u128;5]`, target as a single u128, `mk-codec/bch.rs:173-179,296`). You cannot copy `MK_REGULAR_CONST=0x1062…` into a `[]fe target`; it must be re-expressed as 13/15 GF32 elements. The "sourced from consts.rs … same rigor" framing understates real porting work and is exactly where the BCH **parity test (§9:128)** earns its keep. Recommend §7 explicitly note the residue must be converted u128→`[]fe` (13 elements regular / 15 long), and that the parity test is the gate. Not blocking, but a silent landmine for the implementer.

**[Minor] — `consts.rs` over-generalized.** §7 (spec:114) and §11 (spec:139) say residues come from "the Rust codecs' `consts.rs`." True for mk (`mk-codec/src/consts.rs`), but **md-codec has no `consts.rs`** — `MD_REGULAR_CONST`/`GEN_REGULAR` live in `md-codec/src/bch.rs:7-21`. Fix the path reference so the parity-test author looks in the right file.

**[Minor] — mk1 long-code coverage unstated for Go port.** md1 is regular-code only (`md-codec/bch.rs:1` "regular code only"), but mk1 needs **both** regular and long (`mk-codec/bch.rs:323-347`). §7 says "single or chunked string" but never states the Go verifier must implement **two** mk generators/targets (13- and 15-char). An implementer reading §7 could ship regular-only and silently reject every long-code mk1 chunk as "unknown format." Worth one explicit sentence.

**[Minor] — cross_chunk_hash not mentioned as recovery-time-only.** `split_into_chunks` appends `SHA-256(bytecode)[0..4]` across the set (`chunk.rs:66-70`); it is verified only at reassembly, NOT per-chunk. §4/§7's "device does not reassemble" is therefore correct, but the spec never notes that per-chunk BCH alone cannot detect a dropped/reordered chunk — that safety net is deferred to the (non-goal) bundle layer. A user engraving chunks today gets no missing-chunk detection on-device. Acceptable for v1 given §2 defers bundles, but the gap deserves a one-line callout in §4.

## 3. Verdict

**NOT-GREEN** — 0 Critical, 1 Important open:

- **[Important]** §7/§9: GF32-vector vs packed-u128 representation mismatch is glossed; the residue/generator must be converted to `[]fe`, and §9's parity test is the actual correctness gate. Add an explicit porting note.

All R0 Critical/Important items are genuinely resolved. The three Minors (consts.rs path, mk long-code coverage, cross-chunk-hash callout) are documentation fixes, not design holes. Once the §7 porting note lands, this goes GREEN.

---

## FOLD DISPOSITION (added by main session; not part of verbatim review)

| Finding | Sev | Disposition |
|---|---|---|
| Representation mismatch (GF32 `[]fe` vs packed-u128) glossed | **Important** | FOLDED inline → §7 PR2 (porting note: convert residue/generator u128→`[]fe`, 13 regular / 15 long; parity test §9 is the gate). |
| `consts.rs` path over-generalized (md consts in `bch.rs`) | Minor | FOLDED inline → §7 + §11 path references corrected. |
| mk1 needs BOTH regular + long generators in Go | Minor | FOLDED inline → §7 (Go verifier implements md=regular-only, mk=regular+long). |
| cross_chunk_hash recovery-time-only / no on-device missing-chunk detection | Minor | FOLDED inline → §4 one-line callout. |

All four folded inline (factual spec-accuracy fixes, not deferrable nits). R2 re-dispatch follows to confirm GREEN.
