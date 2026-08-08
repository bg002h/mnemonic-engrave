# SPEC — encrypted payload delivery to the SeedHammer II

Status: **DRAFT, pre-R0.** No code until this passes an architect R0 review at
0 Critical / 0 Important.

Risk classification: **full risk set** — funds/seed material, a new normative
wire format, and work spanning both `mnemonic-engrave` (Rust) and the
`bg002h/seedhammer` fork (Go/TinyGo).

## 1. Purpose

Deliver constellation strings (`md1`/`mk1`) from `mnemonic-engrave` to a
SeedHammer II **over a wire rather than NFC**, with the payload encrypted at
rest, so that the machine's persistent state is ciphertext and nothing else.

The operator loads an encrypted blob into a reserved region of the machine's
internal flash while it is in BOOTSEL. On the next normal boot the application
finds the blob, asks for a passphrase on the touchscreen, decrypts into RAM, and
feeds the plaintext into the engraving path that already exists. Cutting power
destroys the plaintext; the ciphertext remains.

### 1.1 What is genuinely new

Delivering constellation strings already works today over NFC, including
multi-chunk reassembly of long `md1`/`mk1` strings (`nfc/poller/poller.go:41` →
`gui/scan.go:28`). Three things are new:

1. the payload is encrypted,
2. it arrives over USB instead of NFC,
3. it persists on the device between power cycles.

Everything downstream of `gui/scan.go`'s classifier is unchanged.

## 2. Threat model

### 2.1 What this defends against

- **Plaintext seed material at rest on the engraver.** After this feature, the
  machine's persistent state contains ciphertext only.
- **Casual inspection of the machine.** Powering it on reveals nothing without
  the passphrase.
- **Loss of the machine alone, or the passphrase alone.** Recovering the seed
  requires both. This is the feature's real security value: it splits the seed
  into two artefacts that must both be captured.

### 2.2 What this does NOT defend against

This list is normative and belongs in operator documentation, not only here.

1. **A weak passphrase — completely.** Total strength is
   *passphrase entropy + ~20 bits* from the KDF. A human-chosen passphrase is
   worth 25–35 bits and falls to a single rented GPU in minutes. §7 therefore
   mandates a generated passphrase, and the CLI MUST NOT accept a user-supplied
   one.
2. **Anyone who steals the machine obtaining the ciphertext.** Confirmed
   empirically on this device (§3): `debug enable: 1`, `secure debug enable: 1`,
   and BOOTSEL is not disabled, so `picotool save -a` extracts all of flash over
   USB. No soldering, no expertise. **Treat the ciphertext as published.**
3. **Any rate limit, PIN counter, or lockout.** An attacker never executes our
   firmware; they read the flash and attack offline. RP2350 has no secure
   element, and OTP bits only go 0→1, so they cannot back a resettable counter.
4. **Tampering with the blob.** The reserved region lies outside the signed
   image's `LOAD_MAP`, so secure boot gives it no integrity. An attacker with
   brief physical access can *replace* the blob as easily as read it. This is
   why §6 requires AEAD and fail-closed behaviour.
5. **A patient attacker.** Flash contents do not expire; cracking hardware gets
   cheaper. A passphrase that buys 20 years today buys less later.
6. **Coercion.** No duress passphrase, no decoy payload.
7. **A substituted or compromised engraver.** No attestation to the operator of
   which firmware is running.
8. **Loss of the passphrase.** No recovery path, no oracle. The blob is gone.
9. **An open bundle session. The passphrase protects nothing during it.**
   While a session is unlocked (§10.2.2) the decrypted records — including
   `ms1` — are live in SRAM. §3 measured `debug enable: 1` and
   `secure debug enable: 1` on this device, so **SWD reads SRAM directly**: no
   passphrase, no KDF, no offline attack needed. §2.1's "recovering the seed
   requires both" is false for this window, and §2.2a's statement holds only for
   a powered-down machine.

   This is not an exotic state. §10.2.2 holds plaintext for roughly two hours
   across a six-plate bundle. The screensaver does **not** help: `gui.go:2801`
   `idleTimeout = 3 * time.Minute` suspends event routing and draws an overlay
   but does not unwind the flow, so plaintext stays live behind a blanked
   screen — the most likely physical state of an unattended mid-session machine,
   and the one that looks safest.

   **Narrowed by §10.2.2**: the seed record is wiped as soon as its plate is
   cut or skipped, so this window is now the first plate — roughly 21 minutes —
   not the whole session. After that RAM holds public records only. The §10.2.4
   idle timer guards precisely this window and is absent afterwards.

   Defence during that window is **physical custody**, not cryptography. See
   §2.3's operating rule.

10. **An encrypted payload can be DOWNGRADED to an unencrypted one.** An
    attacker with the write access §2.2 item 4 already concedes can strip the
    ciphertext and tag, zero the crypto fields, set `ct_len = 0`, and produce a
    blob the device accepts with no passphrase prompt. The AAD binding of §6.1a
    protects public records against **modification**, not against **removal of
    the encryption** — the guarantee is escapable by deleting the tag rather
    than by defeating it, and no AEAD can prevent that.

    The gain is real in both directions. Alone it is **seed suppression**: the
    secret plates silently cease to exist, and where the blob is the seed's only
    copy the operator engraves the public cards and believes the backup
    complete — §6.4's own "worst available outcome", delivered by an attacker
    rather than a parser bug. Combined with a weak public-data hash it is worse,
    because it moves the payload out of the regime where substitution is 2⁻¹²⁸
    and into the regime where the hash is the only barrier.

    **What detects it:** §6.6's `sealed` byte, which makes a stripped payload
    display a different hash, and §10.2.3's warning naming the case explicitly.
    Both require the operator to have recorded the hash **for every payload,
    including encrypted ones** — which is why §9 prints it for every payload with a
    public section, and §6.6 says to record it.

11. **A public-only payload is not authenticated at all.** With `ct_len == 0`
    there is no key, therefore no tag (§6.1a). Anyone with brief physical access
    can replace the blob wholesale, and the device cannot tell. The BCH checksum
    catches corruption, not substitution: a well-formed `mk1` encoding an
    attacker's xpub passes every structural check, and the operator engraves a
    steel backup of a wallet they do not control.

    What stands in its place is the §6.6 hash — an out-of-band check that works
    **only if the operator actually compares it** — plus the §10.2.3 warning, the
    fingerprint already shown by `gui/mk1_inspect.go:90`, and the mk1↔md1
    template binding (`gui/template_engrave.go:16`).

    Scale it honestly: **the existing NFC path has the same weakness**, since a
    swapped tag does the same thing and carries no warning at all. This is not a
    regression against today. It *is* weaker than an encrypted payload, which is
    why `me seal` encrypts by default and plaintext is an explicit opt-in.

### 2.2a What admitting `ms1` changed (operator sign-off, 2026-08-07)

Before this decision the envelope could carry only public constellation data —
an xpub and a wallet policy. A stolen machine would have leaked privacy, not
funds. §12 item 6 admits `ms1`, and bundles carry it by construction.

**So the machine's flash now holds encrypted seed material.** Combined with §2.2
item 2 — the ciphertext leaves over a USB cable via BOOTSEL, no soldering, no
expertise — the honest statement is:

> Anyone who steals this machine **while it is powered down** obtains an
> offline-attackable ciphertext of your seed. Its entire defence is the 128-bit
> generated passphrase and the ~20 bits the KDF adds. Nothing else stands
> behind it.

**The "powered down" qualifier is load-bearing and was missing from an earlier
draft.** See §2.2 item 9: during an open bundle session the plaintext is in
SRAM and the passphrase protects nothing.

That is a defensible position *only* because §8 mandates a generated 12-word
passphrase and forbids a user-chosen one. It would not be defensible under a
memorable passphrase, and §8's prohibition is therefore load-bearing rather than
advisory — the CLI MUST NOT provide any path to supply one.

The reason this is nonetheless a reasonable trade: the alternative delivery
routes for a seed card are typing 24 words on a touchscreen or sending them in
the clear over NFC. Neither is better, and the second is worse.

### 2.3 The operating rule that follows

**The passphrase must never be stored with the machine.** The entire security
argument collapses if both artefacts sit in the same place.

And, once bundles exist (§10.2.2): **Lock before leaving the machine — at every
plate swap, not only at the end.** A blanked screen is not a locked machine; per
§2.2 item 9 the screensaver leaves plaintext live in SRAM, where SWD reads it
without the passphrase.

## 3. Hardware facts — measured, not assumed

Measured 2026-08-06 on the operator's SeedHammer II via
`picotool info -a` (picotool 2.2.0-a4, device in BOOTSEL):

| Fact | Value |
| --- | --- |
| target chip | RP2350, package QFN80 (RP2350B) |
| **revision** | **A4** |
| **flash size** | **16384K (16 MB)** |
| chipid | `0x77c483b745abf55c` |
| secure boot | 1 |
| **debug enable / secure debug enable** | **1 / 1** — readback is available |
| last booted partition | **none** — no partition table present |
| image extent | `Load 0x10000000->0x10135300` |
| signature | verified |

Revision **A4** is newer than the A2/A3 steppings that errata E10 (partition
table + UF2) and E18 (`FLASH_PARTITION_SLOT_SIZE` bricking) are documented
against. This design avoids partition tables entirely and never programs
`FLASH_PARTITION_SLOT_SIZE`, so neither applies regardless.

### 3.1 Empirical validation of the transport (2026-08-06)

The datasheet is self-contradictory on whether a UF2 written with no partition
table honours `targetAddr` (§5.5.1 "stored at the address they specify" vs
§5.5.3 "always downloaded to the start of flash"). This was settled by
experiment rather than by reading:

1. Backed up `0x10000000`–`0x10140000` → sha256 `4767f695…`
2. Built a 4096-byte distinctive payload; converted with
   `picotool uf2 convert test.bin test.uf2 -o 0x10E00000 --family data`.
   Verified block fields: `magic0=0x0A324655`, `magic1=0x9E5D5157`,
   `flags=0x2000`, `payloadSize=256`, `familyID=0xe48bff58`,
   `magicEnd=0x0AB16F30`, `targetAddr` ascending from `0x10E00000`.
3. `picotool load --verify test.uf2` → OK
4. Read back `0x10E00000` → **byte-identical to the payload**
5. Read back the firmware region → **sha256 unchanged**, signature still verified
6. `picotool erase -r 0x10E00000 0x10E01000` → region returns to all-`0xFF`,
   firmware still unchanged

**Conclusions.** Secure boot with a burned boot key gates *booting*, not
*writing* — the bootrom's UF2 download path has no signature concept (its
complete abort-reason enumeration at datasheet §5.6.4.11 contains no
signature-related reason). Writing a `data`-family blob at a high address is
safe, does not disturb the signed image, and is fully reversible with
`picotool erase -r`.

**Not validated:** the MSD drag-and-drop path. `/dev/sdc` (confirmed as the
machine: `ID_VENDOR=SH`, `ID_MODEL=SHII`, serial embedding the chipid) was not
writable without root and udisks would not mount it. §9 therefore mandates
`picotool load`. Drag-and-drop MUST NOT be documented as supported until tested.

## 4. Architecture

```
HOST (Rust, `me seal`)                DEVICE (Go/TinyGo)
──────────────────────                ──────────────────
constellation string
  │
  ├─ generate 12-word BIP-39 mnemonic (OS CSPRNG)
  ├─ generate 16-byte salt, 12-byte IV (OS CSPRNG)
  ├─ PBKDF2-HMAC-SHA256 → 32-byte key
  ├─ AES-256-GCM seal
  └─ emit blob → data-family UF2 @ 0x10E00000
         │
         │  picotool load --verify   (machine in BOOTSEL, laptop power)
         ▼
   flash 0x10E00000 ────────────────►  XIP read at boot
                                          │
                                          ├─ magic present? → offer unlock
                                          ├─ 12 words typed on EXISTING seed flow
                                          ├─ BIP-39 checksum → typo caught in ~1s
                                          ├─ PBKDF2 (~30s, progress shown)
                                          ├─ AES-256-GCM open → FAIL CLOSED
                                          └─ plaintext → gui/scan.go classifier
                                                  │
                                                  ▼  (unchanged existing path)
                                              engraving
```

Power is the reason the two steps are separate: `monitorPowerSupply`
requires a 20–28 V USB-PD contract before `Init()` configures the LCD
(`cmd/controller/platform_sh2.go:463`), so the app cannot run on laptop power.
BOOTSEL does run on bus power. Load on the laptop, then move the cable to the
PD supply to boot and engrave.

### 4.1 Properties this buys

- **The firmware never writes flash.** Read-only XIP. No erase driver, no
  write-while-XIP hazard, no wear handling, no `crypto/rand` on device.
- **No new USB stack.** The bootrom performs the transfer while the app is not
  running. There is no runtime wired-ingest path in this firmware and this
  design does not add one.
- **Wipe is a host command**, `picotool erase -r`, validated in §3.1.
- **Cache coherency is a non-issue.** The bootrom invalidates the XIP cache on
  boot (datasheet §4.4.1), and the app never reads a region written in the same
  power cycle.

## 5. Flash region

| Property | Value |
| --- | --- |
| Base | `0x10E00000` |
| Size | 64 KiB (16 × 4 KiB sectors) |
| Extent | `0x10E00000`–`0x10E10000` |

Constraints satisfied:

- 4 KiB sector aligned, whole number of sectors, shares no sector with anything.
- Far above the image end (`0x10135300`) and above the sector `picotool load`
  touches when reflashing firmware (`0x10136000`). Leaves ~12.8 MB of firmware
  growth headroom against the current 1.24 MB.
- Far below the top sector (`0x10FFF000`), so a future `--abs-block` (default
  `0x10ffff00`) cannot clobber it.
- Entirely within the measured 16 MB and below the `0x11000000` CS0 watermark.
  **This matters:** writes past physical flash wrap to `0x10000000` and destroy
  the firmware (datasheet §5.5.2).

`picotool load` of new firmware preserves the region — it erases only the
sectors it writes and does read-modify-write on partial ones. Verified in
picotool 2.2.0 `main.cpp` and confirmed empirically in §3.1.

## 6. Wire format — NORMATIVE

Per the project's Rust-primary rule, this format is defined and implemented
**first in Rust with test vectors**; the Go decrypt is a behaviour-faithful port
and may never lead.

All multi-byte integers are **big-endian**.

A payload carries a **public section**, an **encrypted section**, or both.

| Offset | Size | Field | Value / constraint |
| --- | --- | --- | --- |
| 0 | 8 | `magic` | ASCII `MNEMBLOB` |
| 8 | 1 | `version` | `0x01` |
| 9 | 1 | `kdf_id` | `0x01` = PBKDF2-HMAC-SHA256; **`0x00` when nothing is encrypted** |
| 10 | 1 | `aead_id` | `0x01` = AES-256-GCM; **`0x00` when nothing is encrypted** |
| 11 | 1 | `reserved` | `0x00` |
| 12 | 4 | `iterations` | u32; **`0` when nothing is encrypted** |
| 16 | 16 | `salt` | fresh CSPRNG bytes per encryption; **all-zero when nothing is encrypted** |
| 32 | 12 | `iv` | fresh CSPRNG bytes per encryption; **all-zero when nothing is encrypted** |
| 44 | 4 | `pub_len` | u32, length of the public section |
| 48 | 4 | `ct_len` | u32, ciphertext length excluding tag; `0` when nothing is encrypted |
| 52 | `pub_len` | `public records` | **cleartext**, LF-joined per §6.4 |
| 52+`pub_len` | `ct_len` | `ciphertext` | AES-256-GCM ciphertext |
| 52+`pub_len`+`ct_len` | 16 | `tag` | **present only when `ct_len > 0`** |

Header length is 52 bytes.

- Encrypted (`ct_len > 0`): total = `52 + pub_len + ct_len + 16`.
- Unencrypted (`ct_len == 0`): total = `52 + pub_len`, **and there is no tag**.

### 6.1 Blob presence

Erased flash reads `0xFF`. A blob is present iff the first 8 bytes equal
`MNEMBLOB`. Anything else — including all-`0xFF` — means "no payload", and the
feature stays invisible in the UI.

### 6.1a AAD — the public records are authenticated, not encrypted

**AAD = the header AND the public section**, i.e. bytes `[0, 52 + pub_len)`.

This is what "Associated Data" in AEAD is for, and it is the whole reason the
public section can travel in the clear without becoming a funds-loss path. §2.2
item 4 concedes the blob is attacker-**writable** — it lies outside the signed
image's `LOAD_MAP`. Without the AAD binding, an attacker with brief physical
access could swap a `mk1` for one encoding *their* xpub, and the operator would
engrave a steel backup of a wallet they do not control. With it, altering one
byte of a public record fails the tag exactly as altering ciphertext does.
Verified by execution against vector D.

It costs nothing: no second key, no extra primitive, no extra pass.

**But it only works when something is encrypted**, because that is what makes a
key exist. A payload with `ct_len == 0` has no key, therefore no tag, therefore
no authentication. See §2.2 items 10 and 11, and §6.6, for what stands in its place.

The AAD also binds version, algorithm identifiers, iteration count, salt, IV and
both lengths, so none can be tampered downward.

### 6.2 Parameter bounds — checked BEFORE any allocation or KDF work

This firmware runs with **no active watchdog** (`rp.WATCHDOG` appears only in
`rebootIntoBOOTSEL`). A hostile blob declaring a huge iteration count is a hang,
not an error message. Every field below is validated first, and any violation
fails closed with "payload unreadable":

- `version == 0x01`
- `reserved == 0x00`
- **If `ct_len > 0`** (something is encrypted):
  `kdf_id == 0x01`, `aead_id == 0x01`, `100_000 <= iterations <= 2_000_000`,
  `0 < ct_len <= 8191`.
- **If `ct_len == 0`** (nothing is encrypted): `kdf_id == 0x00`,
  `aead_id == 0x00`, `iterations == 0`, and `salt` and `iv` **all-zero**.
  Any non-zero value in those fields is a malformed blob and MUST be rejected —
  they are meaningless without a key, and accepting junk there would let an
  attacker stage a downgrade that a later version might honour.
- `0 <= pub_len <= 8191`
- `pub_len + ct_len > 0` — an empty payload is malformed
- `52 + pub_len + ct_len + (16 if ct_len > 0 else 0) <= 65536` (fits the region)

The `ct_len` and `pub_len` ceilings are 8191, not 8192, because `gui/scan.go:34` computes
`s.overflow = s.overflow || s.n == len(s.buf)` against an `8*1024` buffer:
overflow triggers when the buffer is exactly *full*. A payload of exactly 8192
bytes would pass every bound here, burn the full KDF, authenticate correctly,
and only then die in the classifier — a spec-legal blob that can never engrave.

The length arithmetic MUST be performed in unsigned arithmetic wider than 32
bits, or be otherwise overflow-checked. TinyGo's `int` is 32-bit on this target,
so `52 + pub_len + ct_len + 16` evaluated natively wraps negative when either
length is near 2³² and would pass a `<= 65536` test. A conforming implementation
is protected by the separate `<= 8191` checks, but the region-fit check MUST NOT
be relied on alone — an implementation that "simplified" to it would admit a
4 GiB declared length.

`pub_len` and `ct_len` are authoritative. A UF2 block carries a fixed 256-byte payload, so the
written region is the blob followed by padding, followed by undefined sector
bytes. The device MUST bound each section by its declared
length and MUST NOT infer length from region contents.

### 6.3 Which section a record belongs in — and why there is no `payload_kind`

An earlier draft carried a `payload_kind` byte describing the whole payload.
That stopped working the moment one blob could hold both public and encrypted
content, so the byte is gone (offset 11 is now `reserved`). **Section placement
carries the meaning instead, and it does so more strongly.**

The rule is one line:

> **A record in the PUBLIC section MUST NOT classify as `ms1` or as a BIP-39
> mnemonic, and MUST additionally DECODE.**

Enforced on the **classified content**, never on anything the sealer asserts.

**Be precise about what this does and does not prevent.** An earlier draft
claimed it "prevents shipping seed material in the clear at all". That is false,
and the spec contradicted itself — §12 item 6 already stated the weaker, true
position. `ValidMD`/`ValidMK` (`codex32/mdmk.go:124,136`) are **pure BCH
verifiers**: they check the HRP and the checksum and never decode the payload.
The checksum is publicly computable — the fork ships the generator
(`MDChecksumSymbols`, `AssembleMD1`). So arbitrary bytes wrap into a record that
classifies as `mdmkText`. Verified by execution against the real packages:

```
32 bytes of entropy → md1qqqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8qarc0sdmjzeptm5fdk0
  ValidMD = true   →  scanner.Scan(...) = mdmkText
  codex32.New = "invalid checksum"  →  not a secret, so the rule never fires
  MDDataSymbols round-trips the entropy exactly
```

And `mdmkFlow` (`gui/gui.go:2024`) engraves `mdmkText` **verbatim**; `md.Decode`
runs only on the optional Inspect branch. So a non-conforming sealer — which
§10.2.1 explicitly refuses to assume away — could place secret bytes in the
cleartext section, where `picotool save` reaches them with no passphrase at all.

**Hence the DECODE requirement — and it is per CARD SET, not per record.**
Constellation records are **chunks**. Verified against the real crates and the
real fork packages:

```
md1 single chunk  → "chunk set incomplete: got 1 chunks, expected 3"
mk1 single chunk  → "received 1 chunks, header declares total_chunks = 2"
md1 all 3 chunks  → md_codec::reassemble(&set)  → Ok
mk1 both chunks   → mk_codec::decode(&set)      → Ok
smuggled entropy  → md_codec::reassemble(&[s])  → "wire-format version mismatch"
```

A per-record decode is not merely stricter — it is **impossible**, and would
reject every legitimate payload.

#### The grouping key is `(HRP, chunk_set_id)` — NORMATIVE

**Not the HRP alone.** This is the trap, and it is invisible in vectors D and E.
A 2-of-3 `wsh-sortedmulti` wallet has **three separate `mk1` cards** — one per
cosigner — and **one `md1` card chunked six ways**. Measured, not assumed: the 12
public records group into four cards (`md1` csid 841149 ×6 chunks; `mk1` csids
153720 / 153721 / 153723, ×2 chunks each). An earlier draft said "three md1
cards", generalising the cosigner count onto both halves. Grouping all six `mk1`
records into one HRP group and reassembling gives:

```
mk1 CARD 0 alone (2 chunks)        → Ok
mk1 ALL SIX as one HRP group       → "received 6 chunks, header declares 2"
md1 across two wallets             → "chunk set inconsistent (version/csid/count)"
```

So HRP grouping **rejects every multisig wallet** — the exact shape §6.4's
"why not 7" section commits to admitting, and the flagship mixed payload
(`ms1` encrypted, `mk1`+`md1` public). Vectors D and E each carry one card per
HRP and vector F is `pub_len = 0`, so an HRP-grouping implementation passes all
of them. **Vector G exists to close that.**

The key is the 20-bit `chunk_set_id` in the chunk header. **Both accessors below
are verified operational on a raw, not-yet-grouped record**, which is what this
requires:

| | md1 | mk1 |
| --- | --- | --- |
| **device** | `md.ParseChunkHeader(s)` | `mk.ParseHeader(s)` |
| **host** | `codex32::unwrap_string` → `bitstream::BitReader::new` → `ChunkHeader::read` → `.chunk_set_id` | `string_layer::decode_string` → `StringLayerHeader::from_5bit_symbols` → `Chunked { chunk_set_id, .. }` |

**Not `md_codec::chunk::derive_chunk_set_id`.** An earlier draft of this
amendment cited it, and it cannot do the job: its signature is
`derive_chunk_set_id(id: &Md1EncodingId) -> u32`, and an `Md1EncodingId` is only
obtainable from `compute_md1_encoding_id(&Descriptor)` — i.e. *after* a group has
already been reassembled and decoded. It is also md1-only, with no mk1
counterpart. Following that citation gives a type error, not a working grouping
key.

Note `StringLayerHeader` is `#[non_exhaustive]`, so a wildcard match arm is
mandatory on the host; it MUST fail closed, since an unrecognised header variant
on this path must never be silently grouped with anything. (The three `mk1` cards of a
2-of-3 return 153720 / 153721 / 153723 — **measured on both sides and in
agreement**: `mk.ParseHeader` on the device and
`StringLayerHeader::from_5bit_symbols` on the host return the same values for the
same records. An earlier draft printed 852310 / 852311 / 852308, copied out of a
review report rather than measured.) Every record MUST land in exactly one
group with no leftovers, and every group MUST reassemble **and** decode.

#### Non-chunked records

The chunked flag is in the same header. A record that is **not** chunked is its
own card and decodes via the single-string path (`md_codec::decode_md1_string` /
`md.Decode`); a chunked record joins its `chunk_set_id` group and is
reassembled. Dispatching on the flag is required because neither path handles
both forms — `reassemble` on a non-chunked md1 gives
`wire-format version mismatch: got 2, expected 4`, and `decode`/`md.Decode` on a
chunked one gives `chunked md1 not supported`. `md_codec::encode_md1_string` is
a public API that emits the non-chunked form, so this is reachable.

Note the last line: the §6.3 smuggling example is caught by this, which is the
point.

**No dependency bump is required.** An earlier draft of this amendment claimed
the records carry "md1 wire version 9" and that `me-cli` MUST move from md-codec
0.40 to 0.42. **That was false and is recorded here so it is not reintroduced.**
The records carry version **4** (`md.ParseChunkHeader` →
`{Version:4 Chunked:true ChunkSetID:398802 TotalChunks:3}`), and 0.40
reassembles all of them, including a 6-chunk multisig card. The `9` came from a
single call — `decode_md1_string` (the SINGLE-STRING API) on a CHUNKED record —
and is `0b01001`: version 4 with the chunked flag, misread as a 5-bit version.
The granularity diagnosis was right; the version skew attached to it was the
same error misread a second time. Note also that the device's Go port is
provenance-pinned to md-codec **0.36.0**, so a host-only bump would widen a real
host/device gap for no demonstrated reason.

The encrypted section may carry anything — `ms1`, `mk1`, `md1`, a BIP-39
mnemonic — since it is confidential by construction.

Generating the cards for a real seed makes the distinction plain:

```
ms1  (entropy, BCH-checksummed)   SECRET  — this is the seed
mk1  (xpub + origin)              PUBLIC
md1  (wallet policy)              PUBLIC
```

**`md1` and `mk1` carry public data.** An xpub and a wallet policy leak privacy
if published, but they do not spend coins. Encrypting them is defence in depth,
not protection of key material. The secret half of the constellation is `ms1`,
and a raw BIP-39 mnemonic is equally secret.

This is the feature's real justification: **the encrypted envelope is what makes
it defensible to deliver secret material over this path at all.** The existing
plaintext converter refuses `ms1` (`crates/me-cli/src/lib.rs:59`) precisely
because that path has no confidentiality. Inside this envelope, that objection
does not apply — the blob is ciphertext from the moment it leaves the host.

| Record | Secret? | May ride in the PUBLIC section? | Classifier route |
| --- | --- | --- | --- |
| `md1` — wallet policy | no | yes | `codex32.ValidMD` → `mdmkText` |
| `mk1` — xpub + origin | no | yes | `codex32.ValidMK` → `mdmkText` |
| `ms1` — codex32 secret | **yes** | **NO — refuse** | `codex32.New` → secret |
| BIP-39 mnemonic | **yes** | **NO — refuse** | `bip39.Parse` |

All are forms `gui/scan.go`'s classifier already accepts, so no new engraving
path is required for any of them.

**`ms1` inside the encrypted section is admitted**, a deliberate reversal of the
plaintext converter's refusal (`crates/me-cli/src/lib.rs:59`), signed off by the
operator 2026-08-07 (§12 item 6). The refusal was a property of the *plaintext*
path, which had no confidentiality; inside an authenticated encrypted envelope
the objection does not carry. The consequence is explicit and accepted: **the
machine's flash holds encrypted seed material, not merely an encrypted xpub** —
see §2.2a.

Note the two halves of that decision are now separable, which is the point of
the split: `ms1` is admitted **encrypted** and refused **in the clear**.

### 6.4 Record container — NORMATIVE

Both sections use the same encoding. Everything below applies to the public
section and the encrypted section alike, and the `1..24` record cap is over the
**total** across both.

A bundle carries the several cards of one wallet in a single blob, so the
operator unlocks once and then cuts every plate.

**Encoding: records separated by a single LF (`0x0A`). Nothing else.**

```
record[0] LF record[1] LF … LF record[n-1]
```

LF is safe as a separator because no constellation string contains a newline.

**Every record MUST be the canonical, unbroken string — no interior spaces, no
hyphens, no grouping of any kind.** This is normative and it is the rule most
likely to be got wrong, because it is not how the strings are *displayed*:
`mnemonic bundle` defaults to `--group-size 5` and prints
`md1fv 9wjpq pqpm6 …`, which is a display form. The device rejects it outright —
`codex32`'s `inputChar` has no mapping for `0x20`, so `New` returns
`invalid character` and both `ValidMD` and `ValidMK` return false. Measured
against the real package:

```
SPACED len= 80  New_err=codex32: invalid character  ValidMD=false ValidMK=false
CANON  len= 67  New_err=invalid checksum            ValidMD=true  ValidMK=false
```

**Do not "fix" this by stripping whitespace on the device.** The primary Rust
implementation already refuses interior spaces and hyphens
(`refuses_noncanonical_md1_interior_space` / `_interior_dash`) for a reason that
applies with full force here: `mdmkFlow`/`bundleEngrave` engrave the string
**verbatim**, so a stripped-then-engraved plate would carry separator characters
the BCH checksum never covered, while `unwrap_string` leniently strips
whitespace on the way back in. A scratch or mis-strike that alters a separator
would then be silently absorbed rather than detected — on what may be the
operator's only copy of that card. Refuse at seal time; refuse on device.

The **encrypted** container is parsed only after the GCM tag verifies. The
**public** container is not: §10.2 step 2 splits and classifies it before any tag
check, and in the unencrypted shape there is no tag at all. An earlier draft said
the container "is never parsed on unauthenticated input", which was true in v1
and is false here for half the payload — and it is precisely the sentence that
would license writing the splitter as trusted-input code. Every constraint in
this section binds the public section identically (§6.5).

Normative constraints, all checked before any record is acted on:

- **No trailing LF.** The last record is not followed by a separator. This makes
  the encoding canonical, which a test vector requires.
- **No CR.** A `0x0D` anywhere is a malformed bundle. CRLF is rejected, not
  tolerated.
- **No space or hyphen inside any record** (see above). A space-grouped record
  rejects the whole payload.
- **All-lowercase.** The validators accept a consistently-uppercased string
  (`engine.setCase`, `checksum.go:132`; `verifyMDMK` folds case on the HRP), so
  without this the same wallet has two spec-legal encodings — and therefore two
  different §6.6 hashes. Verified: `md1qqqsyqcyq5rq…` and `MD1QQQSYQCYQ5RQ…` both
  return `ValidMD = true` and hash differently. This is not hypothetical: the
  device's own keyboard-entry path emits **uppercase**
  (`gui/codex32_input_test.go:62`). An operator re-deriving with `me hash` from
  their engraved cards would then see a mismatch on an untampered payload — and
  learn that mismatches are normal, which disarms the single control §6.6 exists
  to provide. Lowercase is what `mnemonic bundle --group-size 0` emits. Pinned
  here at §6.4, not inside §6.6, so the engraved artefact and the hash agree by
  construction.
- **No empty record.** This falls out of the rules above and independently
  rejects `\n\n`, a leading LF, and a trailing LF.
- **`1 <= record_count <= 24`**, counted across **both sections together**. See
  below — derived from the widget actually used, and sized to admit real multisig
  wallets. **Note this is NOT §6.6's `public_record_count`**, which counts the
  public section only; vector D is 5 public of 6 total and the two produce
  different digests.
- **Public-section records MUST additionally group, reassemble and DECODE as
  CARD SETS** (§6.3) — grouped by `(HRP, chunk_set_id)`, never by HRP alone.
  Classification is not sufficient: `ValidMD`/`ValidMK` never open the payload.
  **Do not cite `md.Decode` here** — it takes a single string and refuses
  chunked input (`md/md.go:1231`, "refuses chunked md1"), so calling it per
  record rejects every legitimate payload including vectors D, E and G.
- **Each record `1..512` bytes.** The longest record in a real bip84 bundle is
  **111 bytes** (canonical); 512 is headroom, not a target.
- Total still bounded by `ct_len <= 8191` (§6.2). A canonical six-record bip84
  bundle is **472 bytes**; a 2-of-3 multisig bundle of 15 records is well under
  the ceiling.

**Count the `0x0A` separators and reject `record_count > 24` BEFORE splitting.**
A plaintext of 8191 LF bytes satisfies `ct_len <= 8191` and is only caught by the
record-count and empty-record rules; an implementation that splits first
materialises ~8192 slice headers (~98 KB on a 32-bit target), a fifth of the free
heap, transiently. Bound each record's length during the same scan.

**Every record is classified and allow-listed independently** per §10.2.1. A
record MUST classify as `mdmkText` or a `codex32` secret; a BIP-39 mnemonic as a
record is rejected. **In the PUBLIC section a record MUST classify as `mdmkText`
only** — anything classifying as a secret rejects the whole payload (§6.3). An
`ms1` record in the ENCRYPTED section is admitted per §12 item 6.

If any record fails any check, **the entire payload is rejected**. Partial
acceptance would leave the operator engraving an incomplete wallet backup while
believing it complete, which is the worst available outcome.

#### Why `record_count <= 24`, and why it is not 7

An earlier draft capped this at 7, derived from `ChoiceScreen`'s no-scroll limit.
**That was wrong on both the premise and the consequence**, and it is recorded
here because the error is instructive.

*The premise was wrong.* The fork already ships a paged, arbitrary-length card
list: `bundleReviewFlow` (`gui/bundle_flow.go:227`) uses
`pageBtn := &Clickable{Button: Button2}` to page through any number of cards,
inside the three-slot nav budget (Back / Page / OK). `ChoiceScreen`'s ~7-entry
ceiling binds *layout variants per plate* in `bundleEngrave`, not records per
bundle. The claim that "a scrolling list widget does not exist today" was false.

*The consequence was not an edge case.* Measured with the installed CLI:

| wallet | records |
| --- | --- |
| single-sig `bip84` (12- or 24-word) | 6 |
| **2-of-2 `wsh-sortedmulti`** | **10** |
| **2-of-3 `wsh-sortedmulti`** | **15** |

The smallest multisig that exists is 10 records. A cap of 7 would have rejected
**every multisig wallet** — the entire multisig product surface — while reading
in the spec like a rare-case limitation.

24 admits 2-of-3 with headroom and stays far inside the `ct_len` ceiling. The
plate list MUST use the paged shape (`bundleReviewFlow`'s), not `ChoiceScreen`.
Note §10.2.2 makes Back and Lock the same action, which frees the middle nav slot
for Page.

**The device MUST distinguish "too many records (N, max 24)" from "payload
unreadable".** `record_count` is authenticated plaintext, so naming it leaks
nothing to anyone without the passphrase — and §6.2 uses "payload unreadable" for
a corrupt or *tampered* blob, which per §2.2 item 4 the operator has been taught
to read as "someone replaced my blob". Conflating a too-large wallet with an
attack would send them chasing a compromise that did not happen.

### 6.5 Where structure may live, and why

A rule worth stating once, because it is easy to get backwards:

| | parsed | trust |
| --- | --- | --- |
| **header** (§6) | **before** authentication — it must be, it holds the salt and iteration count | hostile input by construction; bound-check everything (§6.2) |
| **public section** (§6.4) | **before** authentication, always (§10.2 step 2) — and when `ct_len == 0`, never authenticated at all | **hostile by construction.** §2.2 item 4: these bytes are attacker-writable. Every §6.4 constraint binds here identically, including the pre-split separator scan |
| **encrypted plaintext** (§6.4) | **after** the GCM tag verifies | produced by someone holding the passphrase |

`Open` returns an error without releasing plaintext on tag mismatch, so the
record container is parsed only on authenticated bytes. **That is why the bundle
container belongs in the plaintext and not in the header** — the opposite of
where a pre-authentication field would belong.

This is not a licence to trust plaintext blindly: §10.2.1's allow-list still
applies per record, because "authenticated" means "sealed by whoever knows the
passphrase", which is not the same as "safe".
### 6.6 The fixed public-data hash — NORMATIVE

When a payload has no encrypted section it has no key, so nothing authenticates
it (§6.1a). What stands in place of a tag is an **out-of-band check the attacker
does not control**: a hash of the public data that the operator compares against
a value they recorded themselves.

```
sealed  = 0x01 if ct_len > 0, else 0x00
input   = the public section exactly as it appears on the wire —
          canonical LOWERCASE records, LF-joined, no trailing LF (§6.4)

digest  = SHA-256( "MNEMBLOB/pub/v1" ‖ 0x00 ‖ sealed ‖ public_record_count(u8) ‖ input )
display = first 16 bytes, lowercase hex, in 8 groups of 4

        a26e d22b b747 dfd0 2367 06ad 14c1 9679
```

Four things in that construction are load-bearing. Each closes a specific
finding from R0 v2 round 1, and none is decoration.

**1. `sealed` — this is what makes the downgrade visible.**
An earlier draft hashed only the records, making the value deliberately
"independent of whether anything is encrypted at all". That property was chosen
so the operator could record one value forever. It was also **exactly the
blindness an attacker needs**: strip the ciphertext and tag, zero the crypto
fields, set `ct_len = 0`, and you have a payload that satisfies every §6.2 rule,
prompts for no passphrase, and displays *the same hash the operator recorded*.
The one integrity value they hold could not see the one transformation that
destroys integrity. Binding `sealed` into the digest means a stripped payload
shows a **different** number. Vectors D and E pin this: same five public records,
and the hashes now differ.

**2. `public_record_count` — so a removed record is visible**, not merely a
changed one. **This is the count of records in the PUBLIC section, NOT §6.4's
`1..24` cap, which is the total across both sections.** Vector D is exactly where
the two diverge — 5 public records, 6 total — and they produce different digests
(`a26ed22b…` for 5, `c7e152ae…` for 6). A host counting totals and a device
counting public records would disagree on every mixed payload, producing a
mismatch on an untampered blob and teaching the operator that mismatches are
normal.

**3. The domain-separation label**, so this digest can never collide with any
other SHA-256 use in the system.

**4. 128 bits, not 64.** An earlier draft displayed 64 and argued it was "out of
reach", on a cost model of **one child key derivation per candidate**. That model
was wrong. The attacker fixes a single xpub and grinds on fields that are not
cryptographically bound to it and that no one checks:

- the **origin path and parent fingerprint** in each `mk1` — arbitrary indices,
  arbitrary 4-byte fingerprint; descriptors never verify origin metadata against
  the key. Unbounded free bits at zero elliptic-curve cost.
- **record order**, and ordering the grindable record *last* enables SHA-256
  midstate reuse, cutting ~7 compressions per candidate to ~2.
- slack in the `md1` policy encoding — `ValidMD` is a pure BCH verifier with no
  upper length bracket, so a record may run to §6.4's 512-byte cap.

So a candidate costs **one to two SHA-256 compressions**, not a derivation. At
2⁶⁴ that is ≈2.4×10⁵ GPU-hours — **$60k–$250k of rented GPU, weeks on a thousand
cards.** Inside budget for a seed backup whose machine the attacker has already
handled. The argument this spec makes against 32 bits — *it would look like
verification while being defeatable* — applied verbatim at 64.

128 bits removes the parameter from the threat model rather than making it
expensive, and it is the same transcription effort already asked of the operator
for the 12-word passphrase, which is itself 128 bits.

**It is COMPUTED, never STORED. There is deliberately no hash field on the
wire.** A hash carried inside the payload is worthless: an attacker who rewrites
the records rewrites the hash beside them, and the device displays a value that
matches the tampered data perfectly. The check works only because the device
derives the number from the bytes it is about to engrave, and the operator's copy
lives somewhere the attacker cannot reach.

**Why the content and not the file.** The blob's bytes change on every seal
(fresh salt and IV), so a hash over the file would be a new value to write down
each time — and a check nobody can perform from memory is a check nobody
performs. Hashing the canonical records instead keeps the value **fixed** across
salt, IV, iteration count and passphrase. It deliberately does **not** stay fixed
across the sealed/unsealed shape; see point 1.

It is order-sensitive, because record order is plate order and that is content.

**Displayed whenever `pub_len > 0`, sealed or not.** When a tag exists the hash
still answers *which wallet is this?* — but the operator MUST record it for every
payload, including fully-encrypted ones, because that recorded value is what
detects the downgrade. When `pub_len == 0` **nothing is displayed**: the digest
of an empty record set is a constant, and showing the same number on every
fully-encrypted payload would teach the operator it is furniture.

`me hash --sealed|--unsealed <records...>` re-derives the value from the
operator's own cards, with no passphrase, no seal operation and no original file.
The shape selector is required and the operator always knows which to pass —
they either hold a 12-word passphrase for that blob or they do not.


## 7. Cryptographic construction — NORMATIVE

| Component | Choice | Rationale |
| --- | --- | --- |
| KDF | **PBKDF2-HMAC-SHA256**, iteration count per §7.1 | Already linked and exercised on-device by the SLIP-39 recovery path (`slip39/feistel.go:50`). Writing scrypt or Argon2 buys −0.8 to +3.6 bits at equal on-device wall clock — under one passphrase character — in exchange for new unaudited crypto on a funds path. Neither fits its own standard's recommended memory here: RFC 9106's *memory-constrained* Argon2id fallback is 64 MiB against ~452 KB free, and at ~256 KiB an RTX 4090's 72 MB L2 holds 288 concurrent working sets, so memory-hardness is paid for and not received. |
| AEAD | **AES-256-GCM** | Already resident, so **marginal flash is ~1.6 KB** — see the corrected measurement below. The payload is a few hundred bytes, so ChaCha20's software-speed advantage is irrelevant, and the threat is offline ciphertext attack rather than side-channel. |

**Correction, 2026-08-07 — the original rationale here was wrong, and measured
numbers now replace it.** This row previously read "~52 KB, pulled in by
`crypto/ecdsa`'s fips140 dependency … zero marginal flash". Both halves were
unverified and both are false as stated:

- **`crypto/ecdsa` is not the mechanism.** Go 1.25.10's `crypto/ecdsa` never
  imports `crypto/internal/fips140/aes`, and in the real `cmd/controller` build
  it contributes **0 bytes of code** — dead-code-eliminated to an 8-byte stub.
  The actual path is `seedhammer.com/bip39 → crypto/rand →
  crypto/internal/fips140/drbg → .../fips140/aes` (Go 1.24+'s `crypto/rand` is an
  AES-CTR-DRBG), plus the FIPS self-test registry: `aes/gcm/cast.go`'s `init()`
  registers a `fips140.CAST` closure guarded by a runtime `Enabled` var, not a
  compile-time constant, so the code stays link-reachable while never running.
- **"Zero marginal flash" overstates it.** Measured by A/B TinyGo builds at
  `-target pico-plus2 -opt 2`: AES/GCM object code already resident ≈ **20.2 KB**
  (not 52 KB), and making it *callable* from application code — importing
  `crypto/aes` + `crypto/cipher`, which are **absent from today's build** —
  costs ≈ **1.6 KB**. Negligible against 16 MiB of flash at 1.21 MiB current
  usage (~7.6%), but not literally free.

**The conclusion is unchanged and the choice stands**; only the reasoning behind
it was wrong. Note also that the ~64 KB the FIPS registry conscripts on first
PBKDF2-SHA256 use is **already paid** by the existing bip39/slip39 code, so it is
not a cost of this feature. And there is **no hardware SHA acceleration in play**:
nothing in the repo touches the RP2350's SHA256 peripheral, and 32-bit ARM falls
to `sha256block_noasm.go`'s pure-Go `blockGeneric` — which is what the measured
9,715 iterations/sec of §7.1 already reflects.
| Key length | 32 bytes | AES-256 |
| Salt | 16 bytes, fresh per encryption | NIST SP 800-132 §5.1 requires ≥128 random bits |
| IV | 12 bytes, fresh per encryption | SP 800-38D §8.2.2 RBG construction |
| Tag comparison | `crypto/subtle` | Already linked; constant time |

### 7.1 Iteration count

Target **~30 seconds** on device. **Default: 300,000 iterations** → 30.9 s.

**MEASURED 2026-08-07 on real RP2350 silicon: 9,715 PBKDF2-HMAC-SHA256
iterations/second** (dkLen=32, 150 MHz, TinyGo with the firmware's own build
flags: `-stack-size 16kb -gc precise -opt 2 -scheduler tasks`). Harness:
`cmd/kdfbench` in the fork.

| iters | dkLen | elapsed | iters/sec |
| --- | --- | --- | --- |
| 10,000 | 16 | 1.025 s | 9,759 |
| 50,000 | 16 | 5.022 s | 9,957 |
| 100,000 | 16 | 10.303 s | 9,705 |
| 200,000 | 16 | 20.346 s | 9,830 |
| 10,000 | 32 | 1.032 s | 9,685 |
| 50,000 | 32 | 5.139 s | 9,730 |
| 100,000 | 32 | 10.303 s | 9,706 |
| 200,000 | 32 | 20.586 s | 9,715 |

**The estimate this replaces was 15,000/s — high by 1.54×.** An earlier draft
defaulted to 450,000 on that basis, which is **46.3 s** on device, not the 30 s
it claimed. Where the estimate came from is worth recording: an
`≈` on a *range* in a "responsiveness" section
(`design/SPEC_seedhammer_slip39_recovery.md:273`, repeated at
`design/FOLLOWUPS.md:59`), phrased in "SHA-256 blocks" while the derivation read
it as iterations. Nobody had timed it.

Two things the measurement checks rather than assumes:

- **dkLen 16 vs 32 costs the same** — 9,830 vs 9,715/s, a 1.2% difference. Both
  are one PBKDF2 block, so the SLIP-39 path (dkLen 8/16) is a valid anchor for
  our 32-byte key. This was an assumption in the derivation; it now has data.
- **The rate is linear** across a 20× range (1.0 s to 20.6 s), 9,705–9,957/s, so
  it is not distorted by fixed overhead at either end.

Consequences for §6.2's bounds: the floor of 100,000 is 10.3 s and the ceiling of
2,000,000 is 205.9 s — long, but bounded, which is what the no-watchdog argument
requires.

**Residual caveat:** measured on an RP2350**A** (Pico 2, QFN60); the SeedHammer II
is an RP2350**B** (QFN80). Same core, same 150 MHz, and PBKDF2 is compute-bound
with a working set of a few hundred bytes, so the figure should transfer — but
confirm it on the machine during Plan B before release.

The value travels in the blob, so host and device need not agree at compile time.

### 7.2 The one-key-one-message invariant

GCM's nonce-uniqueness requirement (SP 800-38D §8: *"if even one IV is ever
repeated, then the implementation may be vulnerable to the forgery attacks"*) is
satisfied **structurally, not procedurally**: a fresh random salt per encryption
yields a fresh key, so every key encrypts exactly one message. A requirement
about repeats under a given key cannot be violated by a key used once.

**Therefore `salt` and `iv` are write-once.** Re-encrypting an edited payload
MUST generate both afresh. The following are prohibited and each is a silent,
unrecoverable failure:

- reusing a salt to "keep the blob's identity stable across edits";
- deriving salt or IV from the plaintext, a device serial, or a counter to make
  output reproducible;
- restoring a blob from backup alongside a live blob that shares its salt.

There is no legitimate reason for `me seal` to accept a caller-supplied salt or
IV, and it MUST NOT expose a flag to do so.

### 7.3 Randomness

All randomness is generated **host-side in Rust** from the OS CSPRNG.

This is not merely convenient — it is required. TinyGo's `crypto/rand` has no
`Reader` for the `rp2350` target and calling it panics `"no rng"` at runtime.
The device only ever decrypts, so it needs no randomness. Any future on-device
encryption would first require a driver for the unused hardware TRNG at
`0x400f0000`.

## 8. The passphrase

**A 12-word BIP-39 mnemonic, generated by the host. 128 bits of entropy.**

- The CLI generates it from the OS CSPRNG. It **MUST NOT** accept a
  user-supplied passphrase — see §2.2 item 1.
- The BIP-39 checksum lets the device reject most typos in about a second,
  before committing to a ~30-second KDF. Without it, a typo costs the full KDF
  and then an indistinguishable tag failure.
- The device reuses the **existing 12-word seed-entry flow unmodified**
  (`gui/gui.go:968` `NewKeyboard` with `wordKeys`, gated live by
  `updateValidBIP39Keys`). No new keyboard code, and the operator already knows
  the interaction.
- The mnemonic is used **as a passphrase only**. It is never treated as seed
  entropy and never derives a wallet. The UI must not imply otherwise.

Precedent: `age`'s CLI reaches the same conclusion, generating 10 BIP-39 words
rather than letting the user choose (`cmd/age/age.go`).

### 8.1 Normalisation

The passphrase input to PBKDF2 is the 12 words, lowercase, single-space
separated, no leading or trailing space, UTF-8, no NFKD step required (the
English BIP-39 list is ASCII). Host and device MUST produce byte-identical
input; this is covered by a test vector.

## 9. Host side — `me seal`

New subcommand in `crates/me-cli`.

```
me seal <payload>... --out payload.uf2 [--iterations N] [--plaintext <record>]... [--seal-secret]
```

**`--seal-secret` is required to encrypt seed material** — an `ms1` record or a
BIP-39 mnemonic (the two forms of the same secret; `classify` needs a bech32 `1`
separator, so a bare mnemonic does not present as `ms1`). Without it, `me seal`
exits `EXIT_REFUSED` and writes nothing.

This is a **best-effort anti-footgun, not a security boundary** (operator
decision, 2026-08-07). Sealing seed material is a supported operation — §12
item 6 admits it deliberately. The flag exists so it is never done by *accident*,
e.g. pasting a whole `mnemonic bundle` output without reading it. The check is
correspondingly cheap and makes no claim to catch every conceivable encoding of a
seed; anyone who means to seal one simply passes the flag. **Do not grow it into
something that claims to be a control**, and do not let the device rely on it —
§10.2.1's allow-list is where the device's guarantees live.

**There is deliberately no `--addr` flag.** The target address is normative —
`0x10E00000`, fixed by §5 and read unconditionally by §10.1 — so any other value
produces a blob the device will never look at. Worse, §5's whole analysis exists
to keep the write clear of the signed image and inside physical flash: a
mis-specified address either overwrites the firmware directly or, past
`0x11000000`, **wraps to `0x10000000` and destroys it** (datasheet §5.5.2). The
flag would expose a destructive footgun with no legitimate use. If a test seam
is ever needed it MUST NOT be an operator-facing flag.

- Validates every record (canonical form, lowercase, BCH checksum) and decides
  its section. Records destined for the **public** section MUST additionally
  group by `(HRP, chunk_set_id)`, reassemble and decode (§6.3) — `me seal`
  refuses a set that does not, so a blob the device will reject never leaves the
  host. Note `--plaintext` is a per-record flag while the check is per card set,
  so the grouping happens after all `--plaintext` records are collected.
  **By default every record is encrypted.** `--plaintext <record>` places a
  record in the public section instead; `me seal` MUST refuse to place an `ms1`
  or a BIP-39 mnemonic there (§6.3).
- Prints the §6.6 public-data hash whenever `pub_len > 0`, **exactly as the
  device will display it** — hash, record count and sealed/unsealed — and
  instructs the operator to record that whole line, for every payload including
  fully-encrypted ones. The shape is part of the recorded artefact: the shape is redundant
  confirmation — the hex alone already differs between shapes, because `sealed`
  is inside the digest — but it tells the operator which `me hash` flag to pass
  when re-deriving.
- Writes the `.uf2` with mode `0600`, matching `write_private` (`main.rs:375`).

A sibling subcommand re-derives the hash with no passphrase, no seal operation
and no original file, so the expected value can be regenerated months later:

```
me hash --unsealed <record>...  →  70f3 e35a acf7 47db c40f 8376 91aa 61e0
me hash --sealed   <record>...  →  a26e d22b b747 dfd0 2367 06ad 14c1 9679
```

It applies §6.4's canonical checks and refuses a non-canonical record rather than
hashing something the device would reject.
- **Bundles.** Records are supplied as a list and joined with a single
  LF, no trailing LF. `me seal` MUST enforce **every** §6.4 constraint at seal
  time and refuse rather than emit: canonical unbroken records (no interior
  space or hyphen), no CR, no empty record, `1..24` records naming the count and
  cap in the error, each record `1..512` bytes. A bundle the device will reject
  must never leave the host.
  Note `me bundle` cannot be the input path unchanged — `bundle.rs` returns
  `BundleError::RefusedSecret` on any `ms1` line, which §6.4 bundles require.
  Reconciling that refusal with §12 item 6's admission is implementation work,
  and the refusal must be lifted *only* on the sealed path, never on the
  plaintext one.
- **When an encrypted section exists**: generates the 12-word mnemonic, salt and
  IV. **When `ct_len == 0`**: generates no passphrase, prints none, and emits
  `kdf_id = aead_id = 0`, `iterations = 0`, all-zero `salt` and `iv` per §6.2.
  Without this split the CLI would print twelve words for a payload that carries
  no ciphertext — leaving the operator storing a passphrase that protects
  nothing and believing a payload is encrypted when it is not, which is the
  false belief the §6.6 downgrade exploits.
- Runs PBKDF2 and AES-256-GCM, assembles the §6 blob.
- Emits a `data`-family UF2 (`0xe48bff58`) targeting `0x10E00000`.
- Prints the 12 words to **stderr only**, never to a file, with a clear
  instruction to transcribe them and store them apart from the machine.

Loading is a separate, explicit operator step:

```
picotool load --verify payload.uf2      # machine in BOOTSEL, laptop power
```

Wipe:

```
picotool erase -r 0x10E00000 0x10E10000
```

Both are validated in §3.1. Drag-and-drop MUST NOT be documented until tested.

### 9.1 UF2 emission

Either `picotool uf2 convert` or direct emission is acceptable; the fork already
has a `uf2` package. Required block fields, all confirmed against a real
artefact in §3.1: `magic0=0x0A324655`, `magic1=0x9E5D5157`, `flags=0x2000`
(familyID present, and nothing else — in particular not `0x1`
not-main-flash), `payloadSize=256` on every block, `targetAddr` 256-byte aligned
and ascending, exact `numBlocks`, `familyID=0xe48bff58`,
`magicEnd=0x0AB16F30`. Single family per file. **Do not pass `--abs-block`.**

**Payload bytes beyond the blob are `0x00`.** A UF2 block always carries a
256-byte payload, so the final block is padded; §11.4's UF2 sha256 pins zero
padding and will not match `0xFF` padding. The device is unaffected either way
because it bounds every read by `ct_len` (§6.2), but the vector is only
reproducible with the padding byte stated.

## 10. Device side

### 10.1 Detection

At GUI start, read 8 bytes at `0x10E00000` via XIP and compare to `MNEMBLOB`.
Present → the unlock entry point appears in the menu. Absent → the feature is
invisible.

The repo has no precedent for a bare absolute-address read; the closest shapes
are the cgo fixed-address dereference in `driver/otp/otp_rp2350.go:13` and the
`unsafe.Slice` over a peripheral address in `driver/dma/dma_rp2.go:70`. Which
form TinyGo 0.41.1 compiles correctly here is an implementation-time question,
settled by a test, not a design question.

### 10.2 Unlock flow

1. Parse and bound-check the header per §6.2. Any violation → "payload
   unreadable", stop.
2. Split the public section into records and allow-list each per §10.2.1. **Any
   record classifying as a secret rejects the whole payload** (§6.3) — this is
   what stops a seed reaching steel in the clear. Then **group the records by
   `(HRP, chunk_set_id)`, and reassemble and decode every group**; a leftover
   record, or any group that fails, rejects the payload. Non-chunked records are
   their own card and take the single-string decode path (§6.3).
3. **If `pub_len > 0`**, compute the §6.6 public-data hash and display it with
   the record count and the sealed/unsealed shape. **Computed from the records
   just parsed — never read from the payload.** If `pub_len == 0`, display
   nothing: the digest of an empty record set is a constant, and showing the
   same number on every fully-encrypted payload would teach the operator it is
   furniture.
4. **If `ct_len == 0`, stop here: no passphrase is prompted.** Show the
   unauthenticated warning of §10.2.3 with the hash, require an explicit
   confirmation, then go to the plate list. Steps 5–8 are skipped entirely.
5. Enter the existing 12-word BIP-39 entry flow.
6. Validate the BIP-39 checksum. Failure → "not a valid passphrase, check the
   words", return to entry. No KDF is run.
7. Run PBKDF2 with a progress indicator. This takes ~31 s (§7.1) and the screen
   must say so, or the operator will think the machine has hung.
8. AES-256-GCM open over `AAD = header ‖ public section` (§6.1a). **Tag mismatch
   → fail closed**, "wrong passphrase or damaged payload", return to entry.
   Never emit partial plaintext. Note this same check is what authenticates the
   public records, so a tampered public card fails here too — and because that
   is ~31 s after the hash was displayed at step 3, the message MUST offer both
   readings: *"wrong passphrase, or this payload has been altered — compare the
   hash above against the value you recorded"*, keeping the hash on screen
   through the retry loop. Reporting only "wrong passphrase" invites the operator
   to retype three times and conclude the blob is corrupt, losing the one signal
   §2.2 item 4 exists to raise.
9. Split the decrypted section into records and allow-list each per §10.2.1.
10. Wipe the derived key, the passphrase buffer, and PBKDF2 intermediates on
    every exit path, following the existing `wipeBytes` pattern
    (`gui/passphrase_flow.go:605`) and carrying the same honest caveat: TinyGo's
    GC may copy or retain, so this is defence in depth, not a guarantee.

**The passphrase prompt is conditional on `ct_len > 0` and nothing else.** A
public-only payload must never ask for one — there is no key to derive and
prompting would train the operator to type twelve words at a screen that cannot
check them.

### 10.2.1 The classifier allow-list — NORMATIVE, and load-bearing

The classifier's acceptance surface is **wider than the three §6.3 payload
kinds**, and one of the extra branches is irreversible:

```go
// gui/scan.go
const cmdPrefix = "command: "
if bytes.HasPrefix(buf, []byte(cmdPrefix)) {
        cmd := debugCommand{string(buf[len(cmdPrefix):])}
        return cmd, nil
}
// gui/gui.go:1668
case "lock-boot":
        if err := ctx.Platform.LockBoot(); err != nil {   // OTP writes + CPUReset
```

`LockBoot()` performs `writeOTPValues()`, `otp.EnableSecureBoot()` and
`machine.CPUReset()` (`cmd/controller/platform_sh2.go:545`). **A decrypted
plaintext of `command: lock-boot` would burn OTP fuses.** The classifier also
accepts output descriptors and mainnet/testnet Bitcoin addresses.

It is not sufficient to rely on §9's host-side validation to keep these out.
**The wire format is normative and public, so the device MUST NOT assume the
blob was produced by a conforming `me`.** A third-party or defective sealer is
inside the threat model; recall from §2.2 item 4 that the blob is also
attacker-*writable*, being outside the signed image's `LOAD_MAP`.

Therefore the unlock flow MUST accept only these classifier results:

| Section | Permitted classification |
| --- | --- |
| public | `mdmkText` (via `codex32.ValidMD` / `ValidMK`) **AND the records must group by `(HRP, chunk_set_id)` and every group must reassemble and DECODE** (§6.3). A failure in any group rejects the payload. **Not `md.Decode` per record** — that API refuses chunked input and would reject every valid payload. |
| encrypted | `mdmkText`, a `codex32` secret (`ms1`), or a parsed BIP-39 mnemonic |

**The decode step is not optional and not belt-and-braces.** `ValidMD`/`ValidMK`
are pure BCH verifiers that never open the payload, and the fork ships the
checksum generator — so arbitrary bytes wrap into a record that classifies as
`mdmkText` (§6.3, verified by execution). Without the decode, a defective or
third-party sealer can put seed entropy in the cleartext section, where
`picotool save` reaches it with no passphrase and `mdmkFlow` engraves it
verbatim.

The allow-list runs **once per record**. The DECODE step is separate and runs
**once per card group** (§6.3) — the two passes are distinct and neither
substitutes for the other. Any single failure in either rejects the whole
payload (§6.4).

Every other classification — explicitly including `debugCommand`,
`addressText`, and output descriptors — MUST be treated as "payload unreadable".
The check MUST be an allow-list, not a deny-list: a deny-list silently admits
whatever branch the classifier grows next.

**Secrecy is decided by section placement, checked against classified content.**
`codex32.New` (`codex32/codex32.go:98`) accepts secret shares, so nothing the
sealer asserts binds what actually engraves — the device must look at the
content. A record in the public section that classifies as a secret is
malformed, and rejecting it is what stops a seed being shipped in the clear.

### 10.2.2 Session lifecycle — every secret record first, then wiped

A multi-record payload is engraved over one **session**: unlock once, then cut
each plate, swapping steel between them.

**Every record that classifies as a secret is offered FIRST, consecutively, and
each is wiped as its plate leaves the screen — by any route.**

```
unlock ──► [ secret plate 1 of N ] ──┬── Cut       ──► engrave ──┐
                                     ├── Skip                     │
                                     ├── Cancel / fail mid-plate  ├──► WIPE that record
                                     └── error                    ┘
                                              │
                                    (repeat for every secret record)
                                              │
                                              ▼
                         [ plate list: mk1/md1 only, no secret resident ]
                                              │
                                              └──► [ Lock ] ──► wipe all, main menu
```

**Plural, deliberately.** An earlier draft was written throughout in the
singular — *the* `ms1` plate — which is wrong for every multisig wallet that
exists. Measured with the real CLI:

| wallet | `ms1` | `mk1` | `md1` | total |
| --- | --- | --- | --- | --- |
| single-sig `bip84` | 1 | 2 | 3 | 6 |
| **2-of-2 `wsh-sortedmulti`** | **2** | 4 | 4 | 10 |
| **2-of-3 `wsh-sortedmulti`** | **3** | 6 | 6 | 15 |

Under the singular reading the second and third seed cards would never be
offered: the operator engraves one of three, the plate list shows only `mk1`/`md1`
so nothing looks missing, and they store an **incomplete backup of a 2-of-3
wallet believing it complete** — §6.4's own "worst available outcome". Under the
charitable reading the remaining secrets stay resident with §10.2.4's timer
already disabled, which is strictly worse than the design this replaced. §6.4
cites that same 15-record bundle as the reason the cap is 24, so the two sections
were in direct contradiction.

- The operator may **Cut** or **Skip** each secret plate. Either way that record
  leaves RAM before the next one is offered.
- **A cancelled or failed engrave wipes the record too.** Aborting mid-plate to
  re-seat shifted steel is the machine's most ordinary recovery, and keying the
  wipe on *completion* would leave the seed resident with the timer already
  disabled — the exact unguarded state this section exists to prevent. Re-cutting
  needs a fresh unlock; that is the price and it is deliberate.
- The plate list labels each entry by its **classified** type and index
  (`mk1 1/2`, `md1 2/3`), never by anything the sealer asserted, and never
  renders a secret record's contents.
- Records already cut this session are marked. The mark is a **convenience, not
  a guarantee** — it does not survive a power cut and the UI must not imply it
  does.
- Leaving the session by **any** path — Lock, Back, an error, `ctx.Done` — MUST
  wipe everything. Same `wipeBytes` caveat as §10.2 step 10.

#### What this costs

Holding every record for a whole session would mean decrypted **seed** material
resident in SRAM for hours, largely unattended — and §2.2 item 9 makes that live,
because `debug enable: 1` (measured, §3) lets SWD read SRAM with no passphrase.
Cutting the secrets first collapses that to the first *N* plates: ~21 minutes for
single-sig, ~63 for a 2-of-3. The remaining plates carry nothing worth stealing.

The cost is that the operator loses free choice of plate order for the seed
cards, and a later re-cut needs twelve words and a 31-second KDF. Deliberate
(operator decision, 2026-08-07).

### 10.2.3 The unauthenticated-payload warning — NORMATIVE

Shown when and only when `ct_len == 0`. The operator must confirm before the
plate list appears.

```
  ⚠  THIS PAYLOAD IS NOT AUTHENTICATED

  It carries no encrypted data, so there is no key
  and nothing proves it is the payload you sent.
  Anyone with physical access could have replaced it.

  Public data hash (5 records, UNSEALED):
        70f3 e35a acf7 47db c40f 8376 91aa 61e0

  Compare this with the value you recorded.

  If you sealed this payload with a passphrase, the
  encrypted part has been REMOVED. Do not continue.

        [ Matches — continue ]     [ Cancel ]
```

The wording must not overstate what the hash proves. It is an out-of-band check
that works **only if the operator actually compares it** against a value they
hold independently; it is not a signature and the device cannot verify it.

### 10.2.4 Idle wipe — keyed on RESIDENCY, NORMATIVE

§10.2.2 changes what a timeout is for. Once every secret record is gone, RAM
holds public data, so a timer there would guard an xpub while still firing
during the legitimate multi-minute pauses of a plate swap — the fastest way to
teach an operator to disable a control.

The timer is therefore keyed on **whether any secret record is resident**, never
on which button was last pressed:

| Condition | Timer | Rationale |
| --- | --- | --- |
| **any** secret record resident, not actively engraving | **3 min**, 30 s warning | The operator has just typed twelve words; they are standing there. Reuses the existing `idleTimeout` value (`gui/gui.go:2801`). |
| actively engraving, any plate | **paused** | Never wipe mid-plate, needle down. A plate is ~21 min of untouched screen and that is not idleness. |
| **no** secret record resident | **none** | Public data only. Nothing to protect. |

Keying on residency rather than on a Cut/Skip press is what makes the aborted
engrave safe: cancel a secret plate mid-cut and the record is wiped (§10.2.2),
so the third row becomes true because the secret is *actually gone* — not because
a button was pressed.

The warning wakes the screen and any touch resets it, so a present operator is
never wiped out and an absent one is.

**The timer source is already in use and needs no new machinery**:
`gui/gui.go:2801` `idleTimeout = 3 * time.Minute`, driven by `time.Now()` and
`ctx.WakeupAt`/`Platform.AppendEvents` in `Run`'s frame loop. Monotonic elapsed
time is all this needs; no RTC is involved.

**What it does not do.** Per §2.2 item 9 the attack is physical access plus an
SWD probe. Against someone who has both and is waiting, three minutes versus
thirty is close to noise — they need only the window to exist. This is a backstop
against *forgetting*. The controls carrying real weight are physical custody,
Lock being one tap away, and the secrets being gone within the first *N* plates.

### 10.3 UI constraints to respect

- `layoutNavigation` indexes a fixed `[3]int` (`gui/gui.go:1857`) — **a fourth
  nav affordance panics.** Three slots is the whole budget.
- `ChoiceScreen` does not scroll and draws over its own title past ~7 entries
  (`gui/gui.go:1455`, pinned by `gui/freetext_speed_test.go:31`). **This bounds
  layout variants per plate, NOT records per bundle** — the distinction that an
  earlier draft got wrong (§6.4). The bundle plate list MUST use
  `bundleReviewFlow`'s paged shape (`gui/bundle_flow.go:227`), which handles an
  arbitrary number of entries within the three-slot nav budget.

#### The plate list's three slots are Back / Page / OK — and Back IS Lock

**Amended 2026-08-07 (Phase B1 design). The two bullets above contradicted each
other and this resolves it.** An earlier draft of the first bullet required the
plate list to "fit Back / Lock / OK exactly" while the second required
`bundleReviewFlow`'s paged shape — and that shape spends a slot on Page
(`gui/bundle_flow.go:275`: Back / Page / OK). Back + Lock + Page + OK is
**four** affordances into `ys := [3]int{…}` indexed by `int(clk.Button - Button1)`,
which is the panic the first bullet exists to prevent. Both requirements could
not be met as written.

The resolution is not to pick one. **Leaving the session by any route already
wipes everything** (§10.2.2: "Lock, Back, an error, `ctx.Done`"), so a Lock
button distinct from Back is a second control performing one action. The plate
list's nav is therefore:

| slot | affordance | meaning |
| --- | --- | --- |
| `Button1` | **Back — and this is Lock** | wipe everything, return to the main menu |
| `Button2` | Page | advance the paged list, wrapping |
| `Button3` | OK | engrave the selected plate |

This is strictly stronger than a separate Lock. With two exits the guarantee
depends on the operator choosing the wiping one; with Back *being* the wipe,
**every** exit from the plate list wipes, and there is no path an operator can
take that leaves a secret resident. The label shown to the operator should read
as leaving the session, not as stepping back one screen.

§10.2.2's flow diagram is unchanged in meaning — its `[ Lock ]` edge is this
Back.

## 11. Testing

### 11.1 Rust (primary)

- The §11.4 canonical vector, byte-exact. This is the artefact the Go port is
  bound to.
- Round-trip seal/open.
- Passphrase normalisation (§8.1) byte-exactness.
- Section placement per §6.3: `me seal` refuses to put an `ms1` or a BIP-39
  mnemonic in the public section, refuses a record that does not DECODE there,
  and refuses a non-canonical or uppercase record anywhere.
- **The §6.6 hash, asserted as LITERALS.** D (sealed, 5 public records) =
  `a26ed22bb747dfd0236706ad14c19679`; E (unsealed, same 5 records) =
  `70f3e35aacf747dbc40f837691aa61e0`. **They MUST DIFFER** — that inequality is
  the downgrade detector (§6.6 point 1).
  An earlier draft required the opposite here ("D and E, which MUST agree"),
  which is exactly the property the `sealed` byte was added to destroy. The only
  construction satisfying that agreement is one that omits `sealed`, so an
  implementer building this suite from the old bullet would have deleted the fix
  and shipped green.
- **Stability, correctly scoped:** the same records **and the same shape**,
  sealed twice with different salts, IVs and iteration counts, MUST yield the
  same hash. That is the pin; agreement across *shapes* is not.
- Rejection of every §6.2 bound violation.
- UF2 emission field-by-field against §9.1.
- **Freshness (kills the frozen-salt mutant).** Two `seal` invocations of the
  *same* plaintext MUST yield a different salt, IV, passphrase, and ciphertext.
  Without this test, an implementation that froze the salt — the precise defect
  §11.4's warning box exists to prevent — passes the entire rest of the suite,
  because both the round-trip test and the fixed-salt canonical vector are
  satisfied by a constant salt.
- **No `(derived key, iv)` pair is shared by any two shipped vectors.** Computed
  over the whole vector set, not asserted by inspection. Two vectors sharing
  both would be GCM nonce reuse in the project's own test data — the exact error
  caught while authoring vector C.
- **Bundle container encoding is canonical (§6.4).** Sealing the same record
  list twice yields identical plaintext, and each of these is **refused at seal
  time rather than emitted**: a trailing LF · a CR · an empty record · **a
  record containing an interior space** · **a record containing a hyphen** ·
  more than 24 records · a record over 512 bytes.
  The space and hyphen cases are the ones that shipped broken in an earlier
  draft of vector C (§11.4). The device rejects them too, but a host that emits
  a blob the device will refuse is its own defect — and host-side refusal is
  where the operator gets an actionable message instead of "payload unreadable"
  after a 30-second KDF.
- **Vector B (kills the hardcoded-iterations mutant).** §11.4 vector B is
  identical to vector A except `iterations = 100001`, and MUST decrypt
  successfully. An implementation that hardcodes the iteration count rather than
  reading it from the header fails this and passes everything else — including
  the altered-to-50000 negative case, which mismatches on AAD alone regardless
  of which count the KDF used.

### 11.2 Go (port, bound to the Rust vectors)

- Decrypt vectors **A, B, C and D** to their expected records, and parse **E**.
  Vector B is not optional — it is the only test that catches a hardcoded
  iteration count. D and E are the only device-side coverage of the mixed and
  public-only shapes.
- **Vector E reaches the plate list with the keyboard flow NEVER ENTERED** —
  asserted by instrumenting the prompt entry point, not by return value. A
  scripted fake platform will happily feed twelve words into a prompt that should
  not exist and still reach the plate list, so a return-value assertion reports
  PASS over exactly the defect (§10.2's stated harm: prompting "would train the
  operator to type twelve words at a screen that cannot check them").
- **The §6.6 hash, literal.** D displays `a26ed22bb747dfd0236706ad14c19679`; E
  displays `70f3e35aacf747dbc40f837691aa61e0`. Asserted as literals, not merely
  as differing from each other.
- **Every secret record is offered before any public plate**, and each is zeroed
  after its plate leaves the screen — including on the **cancelled** and
  **failed** paths, asserted on the buffers via a fake platform.
- **The idle timer is paused during engraving** and armed whenever any secret
  record is resident, asserted on the timer state rather than by waiting.
- **Vector F: all three secret records are offered consecutively, before any
  public plate, and each is zeroed before the next is offered.** The only
  multi-secret coverage in the suite; a singular implementation passes A–E.
- **A BCH-valid but UNDECODABLE `md1` in the public section rejects the payload**
  (§10.2.1). Construct it the way §6.3 documents — wrap arbitrary bytes and
  append a correct checksum with the fork's own generator — and assert nothing
  was engraved.
- **An uppercase record is refused** (§6.4). `MD1QQQ…` passes `ValidMD`, so
  without this the same wallet has two spec-legal hashes.
- **E-shape hash sensitivity.** Flip the first byte, then the last byte, of
  vector E's public section and assert the **displayed hash changes**. E has no
  tag, so this is the only test that discriminates a hash covering the whole
  section from one covering a subset — the AAD flips in §11.4 fire on the tag
  regardless of what the hash reads.
- **A public section of 8191 LF bytes** with `ct_len == 0` is rejected on the
  separator count before any split allocates — asserted with `testing.AllocsPerRun`
  bounded to **0** additional allocations. This path needs no passphrase and no
  KDF to reach, unlike the plaintext-container case.
- Every §6.2 bound violation fails closed *before* the KDF runs — asserted by
  timing or by instrumenting the KDF call, not merely by return value. The case
  set MUST include `ct_len = 0xFFFF_FFF0`, which a 32-bit native-`int` region-fit
  check would wrap negative and accept.
- The §6.2 unencrypted-shape rules are enforced: with `ct_len == 0`, any non-zero
  `kdf_id`, `aead_id`, `iterations`, `salt` or `iv` is rejected.
- Tag mismatch yields no plaintext.
- BIP-39 checksum rejection happens without invoking the KDF.
- Absent/erased region (all `0xFF`) reports "no payload", not an error.
- **Classifier allow-list (§10.2.1).** A blob whose plaintext is
  `command: lock-boot` MUST be rejected as "payload unreadable", and
  `Platform.LockBoot` MUST NOT be reached — asserted with a fake/instrumented
  platform that fails the test if `LockBoot` is called, not by return value
  alone. Same for an `addressText` plaintext and an output descriptor.
- **Secret-in-the-clear refusal.** An `ms1` record placed in the PUBLIC section
  MUST reject the whole payload, and the test MUST confirm nothing was engraved.
  This is the single most important negative case in the suite: it is what stops
  a seed reaching steel unencrypted.
- **Bundle container (§6.4).** Vector C splits into exactly 6 records with the
  stated types and lengths. Each of these MUST reject the **whole** bundle, and
  the test MUST confirm no record was engraved:
  trailing LF · a CR anywhere · an empty record (`\n\n`) · a leading LF ·
  25 records · a 513-byte record · a space-grouped record · a hyphenated
  record · a BIP-39 mnemonic as a record · a `command: lock-boot` record in
  **position 3 of 6**.
  That last case is the load-bearing one: it proves the allow-list runs **per
  record** rather than on the first only. A deny-list, or a loop that checks
  record 0 and then trusts the rest, engraves records 0-2 and only then meets
  the command — so the test MUST also assert that **nothing was engraved**.
- **Pre-split bound scan, asserted by allocation count.** A plaintext of 8191 LF
  bytes MUST be rejected. Note carefully that **rejection alone does not test
  anything here**: 8191 LF bytes yields 8192 empty records, and a correct
  pre-split separator scan and a split-then-count implementation both reach
  `record_count > 24` and both reject, with the same reason. The *only*
  observable difference is the ~8192 transient slice headers the mutant
  allocates. So this case MUST be asserted with `testing.AllocsPerRun` (or
  equivalent) — the same "instrument it, don't trust the return value" rule the
  KDF-ordering assertions use. A return-value assertion here is a guaranteed
  false PASS.

  **The bound MUST be a concrete number, and that number is 0 additional
  allocations. Do NOT write it as "O(1)".** `bytes.Split` performs exactly
  *one* allocation regardless of record count (a single `make([][]byte, n)`;
  the slices themselves point into the existing buffer), while a correct
  `bytes.Count`-style scan performs *zero*. Both are "O(1)", so a threshold of
  `allocs <= 2` — a faithful reading of an O(1) requirement — passes the mutant
  and ships the defect with every check green. The whole point of this
  assertion is the difference between 0 and 1.
- **Vector C is a positive test.** It decrypts to 6 canonical records, classified
  in order as `ms1`, `mk1`, `mk1`, `md1`, `md1`, `md1`, and all six are
  engravable. The test MUST assert the classification of each record, not merely
  that the bundle parsed — a bundle that split correctly but classified nothing
  would otherwise pass.
- **Space-grouped records reject the whole bundle (§6.4).** Vector C with its
  records in `--group-size 5` display form MUST be rejected, and the test MUST
  confirm no record was engraved. This is the exact defect that shipped in an
  earlier draft of vector C.
- **Wipe on every exit path.** After leaving a bundle session by **each** exit —
  Lock, Back, an error path, and `ctx.Done` — the plaintext record buffer, the
  derived key, and the passphrase buffer MUST read as zeroed. Asserted **on the
  buffers themselves**, via a fake platform that drives the flow to each exit —
  never on a return value. Carry the honest caveat from §10.2 step 10: this
  covers the buffers the flow owns, not copies TinyGo's GC may have made.
- **Too many records is not "unreadable".** A 25-record bundle MUST report a
  distinct, record-count-naming error (§6.4), not the string used for a corrupt
  or tampered blob.

### 11.3 Mutation testing — mandatory

Per project standard, a green suite proves little. Break the code and confirm a
test notices.

**Every mutant below MUST name the test that kills it.** A mutant with no named
killer is a gap in the suite, not a passing result — R0 round 1 found exactly
this: two of the original five mutants survived the entire specified test set.

| Mutant | Killed by |
| --- | --- |
| tag verification made unconditional-pass | §11.4 negative: flipped ciphertext byte |
| bound checks removed | §11.2 bound-violation cases, incl. `ct_len = 0xFFFF_FFF0` |
| BIP-39 checksum check removed | §11.4 negative: `beef`×11 + `bacon` |
| **salt reused across two seals** | **§11.1 freshness test** — nothing else sees it |
| **iteration count read as a constant** | **§11.4 vector B** — nothing else sees it |
| classifier allow-list weakened to a deny-list | §11.2 `command: lock-boot` case |
| KDF run before the checksum gate | §11.2 timing/instrumentation assertion |
| allow-list applied to the first record only | §11.2 `command: lock-boot` in position 3 of 6 |
| bad record dropped instead of rejecting the bundle | §11.2 bundle-**rejection** assertion (not the "no record engraved" one — under this mutant the bundle is *accepted* with the bad record silently dropped, so rejection is what does the work) |
| trailing LF tolerated on parse | §11.1 canonical-encoding test |
| whitespace stripped from records before classifying | §11.2 space-grouped-bundle rejection |
| **two vectors share a `(key, iv)` pair** | **§11.1 pair-uniqueness assertion** |
| **wipe omitted on the Back exit path (wipe on Lock only)** | **§11.2 wipe-on-every-exit assertion** |
| **public section left out of the AAD** | **§11.4 negative: flip a byte of D's public section** — nothing else notices, and the failure mode is an engraved backup of an attacker's wallet |
| **the §6.6 hash computed over a subset of the public section** (first record, or `pub_len - 1` bytes) | **§11.1/§11.2 literal-value assertion.** NOT the tag-mismatch flips — those fire on the AAD regardless of what the hash covers, so they do not discriminate |
| **the §6.6 hash unchanged by a public-section edit** | §11.2 E-shape negative: flip the first byte and the last byte of E's public section (no tag exists there) and assert the DISPLAYED HASH changes |
| **only the first secret record offered** | §11.2 vector F offer-order assertion — nothing else in the suite has more than one secret |
| **decode check removed from the public section** | §11.2 BCH-valid-but-undecodable `md1` negative |
| **uppercase record accepted** | §11.1/§11.2 uppercase refusal case |
| **`sealed` omitted from the hash input** | **vectors D and E must DIFFER** — the downgrade detector |
| **`public_record_count` omitted from the hash input** | §11.1/§11.2 literal-value assertion. NOT "a 4-record variant hashes differently" — LF-joined records are already injective over the record list, so that test passes under the mutant too (verified) |
| `ms1` accepted in the public section | §11.2 secret-in-the-clear refusal |
| passphrase prompted when `ct_len == 0` | §11.2 vector E parses with no prompt |
| `ms1` not wiped after its plate | §11.2 post-plate buffer assertion (§10.2.2) |
| idle timer runs during engraving | §11.2 timer-paused assertion (§10.2.4) |
| record count checked after splitting rather than before | §11.2 8191-LF-bytes case, **asserted on allocation count** — the return value is identical under both, so a rejection-only assertion is a guaranteed false PASS |
| `me seal` emits a record with an interior space or hyphen | §11.1 canonical-record seal-time refusal |

The two bolded rows are the ones that survived round 1. Note in particular why
the altered-iterations negative case does *not* kill the hardcoded-count mutant:
changing the header changes the AAD, so the tag mismatches regardless of which
iteration count the KDF actually used. Only a vector that must *succeed* at a
different count discriminates.

Procedural rules: assert the substitution matched before running the test (a
silently-failing `sed` reads exactly like a surviving mutation), and restore
from a **file copy**, never `git checkout`.

### 11.4 Canonical test vectors — `beef` / `bacon`

The normative cross-implementation vectors. The Rust implementation produces
them; the Go port MUST reproduce them byte-exactly.

**The fixed salts and IVs below exist ONLY because a test vector must be
deterministic. This is the sole exemption from §7.2. Production code MUST NOT
have a code path that accepts a caller-supplied salt or IV.**

Shared inputs: passphrase `beef` × 12 (a checksum-valid 12-word BIP-39 mnemonic
— see "Why these words" below). Records are the canonical, unbroken forms from
`mnemonic bundle --group-size 0` for the `bacon`×24 seed.

| | A | B | C | D | E |
| --- | --- | --- | --- | --- | --- |
| shape | all encrypted | all encrypted | all encrypted | **mixed** | **public only** |
| public records | — | — | — | 5 (mk1×2, md1×3) | 5 (mk1×2, md1×3) |
| encrypted records | 1 (bacon×24) | 1 (bacon×24) | 6 (full bundle) | 1 (`ms1`) | — |
| `pub_len` | 0 | 0 | 0 | 396 | 396 |
| `ct_len` | 143 | 143 | 472 | 75 | **0** |
| `iterations` | 100000 | **100001** | 100000 | 100000 | **0** |
| `salt` | `beef`×8 | `beef`×8 | `bead`×8 | `d00d`×8 | all-zero |
| `iv` | `bac0`×6 | `bac0`×6 | `cafe`×6 | `f00d`×6 | all-zero |
| blob length | 211 | 211 | 540 | 539 | **448** |
| tag present | yes | yes | yes | yes | **no** |

Derived keys, tags, and blob digests:

| | derived key | tag | blob sha256 |
| --- | --- | --- | --- |
| A | `615ad9b781b1ad6105d9dffb135d1bf17ebab286c560f26912ee815836e7ad1e` | `4c425808fc389298761c3905166bea40` | `6707c20e7967e80e4cd4cb6dbe05e681d56c722320aa8213886c05a31e94def0` |
| B | `003800ae6cec47cd4b34bb264c6bbb1156d806516ad1ab88391e479d14d8776f` | `cf761a295fd66eaeffe235090cba3cbb` | `25fc2eaf950c9455497dc18eea6a93f5a54463a471cd15a4f8f327d13c7fea4c` |
| C | `19c78c5535ad24349f75fb6ca9a59c939ea885c126cc4909eb2cdc0c26add40e` | `be5bfc2beaf3d91995f5d526e755b505` | `272f45e8ee30c95fdb1804ca54a9ec4b1d8c1358967d88c76312c0f725973ffc` |
| D | `ac975af49a59f691723d559ed9130bd84df744048776fe1a15905468c7f60a06` | `d971935b5091822833206dd0d70b2b8f` | `6332e2d674322b2af656677cb550754b1ec7691f3df14895a807297712cdcd6a` |
| E | *(none — no key exists)* | *(none)* | `39b21ef010540d16967bba954bac6e94a888b2811b65df2e829402dc68d1c132` |

Headers (52 bytes each):

```
A  4d4e454d424c4f4201010100000186a0beefbeefbeefbeefbeefbeefbeefbeef
   bac0bac0bac0bac0bac0bac0000000000000008f
B  4d4e454d424c4f4201010100000186a1beefbeefbeefbeefbeefbeefbeefbeef
   bac0bac0bac0bac0bac0bac0000000000000008f
C  4d4e454d424c4f4201010100000186a0beadbeadbeadbeadbeadbeadbeadbead
   cafecafecafecafecafecafe00000000000001d8
D  4d4e454d424c4f4201010100000186a0d00dd00dd00dd00dd00dd00dd00dd00d
   f00df00df00df00df00df00d0000018c0000004b
E  4d4e454d424c4f4201000000000000000000000000000000000000000000000000
   00000000000000000000000000018c00000000
```

#### The fixed public-data hash — the property vectors D and E exist to pin

Both D and E carry the same five public cards. D encrypts an `ms1` alongside
them; E encrypts nothing at all. Their salts, IVs, iteration counts, keys, tags
and blob digests all differ — and the §6.6 hash is identical:

```
D (sealed)      a26e d22b b747 dfd0 2367 06ad 14c1 9679
E (public-only) 70f3 e35a acf7 47db c40f 8376 91aa 61e0

raw D  a26ed22bb747dfd0236706ad14c19679
raw E  70f3e35aacf747dbc40f837691aa61e0
```

**They MUST DIFFER**, and that inequality is the whole point. D and E carry
byte-identical public records; the only difference is that D is sealed. An
earlier draft made the hash invariant across that difference and pinned D ≡ E as
"the fixed requirement" — which was exactly the blindness the downgrade needs.
Under §6.6's `sealed` byte, stripping the ciphertext changes the displayed value.

**The equality test that replaced it.** A *stability* pin is still needed, and it
must not be satisfiable by hashing the wrong bytes: two payloads sharing records
**and shape** but differing in salt, IV and iteration count MUST agree. Vectors A
and B are that pair on the encrypted side; for the public hash, seal D twice with
different salts and assert the hash is unchanged.

**Why the literal value must be asserted, not just agreement.** The old D ≡ E
test was satisfied by *any deterministic function of any subset* of those bytes,
because the two public sections were identical. Demonstrated by execution against
the old construction: hashing only the first record, or `pub[:-1]`, both passed
it. §11.1 and §11.2 MUST therefore assert the **literal** values above, and
§11.4's negatives MUST include flipping the **first** and **last** byte of the
public section — subset and off-by-one mutants survive an agreement test and die
on those.

#### Vector F — 2-of-3 multisig, THREE secret records

Without this, a singular implementation of §10.2.2 passes the entire suite: A/B
carry one BIP-39 mnemonic, C has one `ms1` among six records, D has one, E has
none. Nothing discriminates plural from singular, which is precisely the defect
§10.2.2 was rewritten to fix.

A real 2-of-3 `wsh-sortedmulti` bundle, all records encrypted:

| Field | Value |
| --- | --- |
| records | **15** — `ms1` ×3 (indices 0,1,2), `mk1` ×6, `md1` ×6 |
| record lengths | 75, 75, 75, 111, 93, 111, 93, 111, 93, 85, 85, 85, 85, 85, 77 |
| `pub_len` / `ct_len` | 0 / 1353 |
| `salt` / `iv` | `f00d`×8 / `beef`×6 |
| `iterations` | 100000 |
| derived key | `d9bdc86754e222898f0c1dfa7f63b209b5851c11c9a0019e7e45568cf0ad7019` |
| tag | `660202c75c2ff0fe05bfced46e2b7cdf` |
| blob | 1421 bytes, sha256 `97e059ac91596da711a70197b20a7fec1edbe7992eba6c51751ef062596f1cb6` |

Header: `4d4e454d424c4f4201010100000186a0f00df00df00df00df00df00df00df00dbeefbeefbeefbeefbeefbeef0000000000000549`

Required of F specifically: **all three secret records are offered consecutively,
before any public plate, and each is zeroed before the next is offered.** An
implementation that offers only the first passes A–E and fails only here.

#### Vector G — 2-of-3 MIXED, a public section spanning FOUR cards

The vector that makes the grouping key testable. D and E carry **one card per
HRP**, so an implementation grouping by HRP alone passes them; F is
`pub_len = 0`. G is the first payload whose public section holds several cards
of the same HRP (three `mk1`), and an HRP-grouping implementation fails it with
`received 6 chunks, header declares total_chunks = 2`.

| Field | Value |
| --- | --- |
| public | **12 records** in **four cards** — `mk1` ×6 (three cards, 2 chunks each), `md1` ×6 (ONE card, 6 chunks) |
| encrypted | 3 × `ms1` |
| `pub_len` / `ct_len` | 1125 / 227 |
| `salt` / `iv` | `abcd`×8 / `1234`×6 |
| `iterations` | 100000 |
| derived key | `13a9867c197f242a577fd4c782ae09435bfdf4d4bd61c25db20e93e55988fc89` |
| tag | `6712131b90654967eae853bad65fd5af` |
| blob | 1420 bytes, sha256 `483fb482ac7aef0da3fec638de183f8f3bfb35e1b6c0ec4f5b274ec0409908f1` |
| §6.6 hash | `be11 7b56 9cc4 cd6e b47d 32b6 fd32 ccb8` |

Header: `4d4e454d424c4f4201010100000186a0abcdabcdabcdabcdabcdabcdabcdabcd12341234123412341234123400000465000000e3`

Required of G specifically: **the 12 public records group into four card sets,
every one reassembles and decodes, and no record is left over.** An
implementation grouping by HRP alone rejects this payload.

#### Required assertions

Positive: each vector round-trips to its exact records. E parses with **no
passphrase prompt at all**.

Negative — each MUST fail:

| Case | Expected |
| --- | --- |
| passphrase `abandon`×12 | rejected at the BIP-39 checksum, **no KDF run** |
| passphrase `beef`×11 + `bacon` | rejected at the BIP-39 checksum, **no KDF run** |
| `iterations` altered `100000` → `100002` on vector A | AEAD tag mismatch (AAD binding). **Not `50000`** — that is rejected by §6.2's floor before any tag work, so it proves nothing about the AAD |
| any ciphertext byte flipped | AEAD tag mismatch |
| **any byte of D's public section flipped** | **AEAD tag mismatch** — this is what proves the AAD covers the cleartext section; verified by execution |
| an `ms1` record placed in the public section | rejected, nothing engraved (§6.3) |
| E with a non-zero `salt`, `iv`, `iterations`, `kdf_id` or `aead_id` | rejected as malformed (§6.2) |
| `magic` altered | reported as "no payload", not as an error |

The `beef`×11 + `bacon` case is the important passphrase one: a valid-length
mnemonic of real words differing in one position, and checksum-**invalid**. A
gate that passes it is broken.

The two "no KDF run" assertions MUST be enforced by instrumenting or timing the
KDF call. Asserting only on the return value passes over exactly the defect it
is meant to catch — an implementation that runs a 31-second KDF before checking
the checksum returns the same value.

#### Why these words — both checksums are valid, and that is not obvious

Verified against `bip39/wordlist.txt` (2048 entries; `beef` = index 160,
`bacon` = index 138) with a checker self-tested against `abandon`×11+`about`,
`zoo`×11+`wrong`, `abandon`×23+`art` (accepted) and `abandon`×12 (rejected):

- `beef` × 12 — 128 bits entropy + 4-bit checksum `0000`. **Valid.** 1-in-16.
- `bacon` × 24 — 256 bits entropy + 8-bit checksum `10001010`. **Valid.** 1-in-256.

An all-identical mnemonic is normally checksum-invalid; `abandon`×12 is the
counter-example and is in the test set as a negative case.

#### Records must be canonical — the round-3 Critical

An earlier draft built these vectors from `mnemonic bundle`'s **display** form
(`--group-size 5`), records of 89/133/95/80/80/80 bytes. Every one of those
classifies as **unknown format** on the device — `codex32`'s `inputChar` has no
mapping for `0x20`:

```
SPACED len= 80  New_err=codex32: invalid character  ValidMD=false ValidMK=false
CANON  len= 67  New_err=invalid checksum            ValidMD=true  ValidMK=false
```

Canonical lengths are **75, 111, 80, 67, 67, 67**. Regenerate with
`--group-size 0`. Never take a value from how a tool *prints* it; trace it into
the consumer that will parse it.

### 11.5 Hardware

- Measure the real PBKDF2 rate and fix §7.1's iteration count.
- End-to-end: seal on host → `picotool load` → boot on PD power → unlock →
  engrave.
- Confirm firmware reflash preserves the blob.
- Confirm `picotool erase -r` wipes it and the app then reports "no payload".

## 12. Open items — must close before or during R0

1. **Iteration count — RESOLVED 2026-08-07 by measurement.** 9,715
   iterations/sec on real RP2350 silicon; default set to **300,000** (30.9 s).
   The prior 450,000 came from an estimate that was high by 1.54× and would have
   meant 46 s. See §7.1 for the full table and the residual RP2350A-vs-B caveat,
   which is the only part still owed — confirm on the machine during Plan B.
2. **Multi-record payloads — RESOLVED 2026-08-07, bundles are in scope.**
   Operator chose the LF-separated container (§6.4) and the unlock-once session
   model (§10.2.2) over one-blob-per-card. This adds a container parser, a plate
   list, and roughly two hours of plaintext residency per bundle — see §10.2.2's
   cost paragraph, which is a genuine weakening of §2.1.
3. **MSD drag-and-drop** (§3.1) — untested. Either test it or keep it
   undocumented.
4. **Recorded security decision.** Closing the readback hole
   (`CRIT1.DEBUG_DISABLE`, `BOOT_FLAGS0.DISABLE_BOOTSEL_*`) is **incompatible**
   with this design, which needs BOOTSEL as its permanent provisioning channel.
   Erratum RP2350-E20's cleanest mitigation is thereby foreclosed. This should
   be an explicit, recorded decision rather than an accident of the current OTP
   state.
5. **XIP read form** (§10.1) — settled by a test at implementation time.
6. **`ms1` admission — RESOLVED 2026-08-07, operator signed off. ADMITTED behind
   `--seal-secret`** (amended 2026-08-07; see §9 for the flag, which also covers
   a bare BIP-39 mnemonic and is a best-effort guard, not a control).
   The plaintext converter refuses `ms1` by design
   (`crates/me-cli/src/lib.rs:59`). That refusal was a property of the
   *plaintext* path, which had no confidentiality; inside an authenticated
   encrypted envelope it does not carry. `ms1` records in the encrypted section are
   admitted.

   The accepted consequence, recorded in §2.2a: **the machine's flash holds
   encrypted seed material, and a stolen machine yields an offline-attackable
   ciphertext of the seed.** Its whole defence is the generated 128-bit
   passphrase plus ~20 KDF bits. This makes §8's prohibition on user-chosen
   passphrases load-bearing rather than advisory.

   The kind/content cross-check (§10.2.1) survives this decision but changes
   purpose: it is now a mislabelling detector, not a policy gate.
7. **Bundles larger than 24 records — RESOLVED 2026-08-07 by re-derivation.**
   An earlier draft capped this at 7 on a false premise (that no paged list
   widget existed) with a consequence that would have rejected *every* multisig
   wallet — measured at 10 records for 2-of-2 and 15 for 2-of-3. The cap is now
   24, derived from `bundleReviewFlow`'s paged list (`gui/bundle_flow.go:227`).
   Remaining work is implementation, not decision: the plate list must use the
   paged shape, not `ChoiceScreen`.
8. **Session idle wipe — RESOLVED 2026-08-07 as a two-phase timer (§10.2.4).**
   The question dissolved once `ms1` is wiped after its plate (§10.2.2): the
   long tail of a session holds public data only, so there is nothing for a
   timeout to guard there. What remains is the narrow window before the seed
   plate is cut or skipped, which takes a **3-minute** timer with a 30-second
   warning — the operator has just typed twelve words and is standing at the
   machine. Paused during engraving; absent afterwards. The timer source was
   already in use (`gui/gui.go:2801`).
9. **Public-only payloads are unauthenticated** (§2.2 item 11). Accepted with
   the §10.2.3 warning and the §6.6 hash; `me seal` encrypts by default so
   plaintext is a deliberate opt-in. Revisit only if a signing story ever exists
   for this path.

## 13. Non-goals

- On-device encryption. The device decrypts only; `crypto/rand` panics there.
- A runtime USB data path. The bootrom does the transfer.
- Firmware flash writes of any kind.
- A partition table.
- Any device-side rate limiting, PIN counter, or lockout — §2.2 item 3 explains
  why these would be theatre.
- Replacing NFC ingest. This is an additional path, not a substitute.
