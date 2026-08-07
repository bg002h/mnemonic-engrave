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

   Defence during a session is **physical custody**, not cryptography. See
   §2.3's operating rule and the required idle-wipe in §12 item 8.

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

| Offset | Size | Field | Value / constraint |
| --- | --- | --- | --- |
| 0 | 8 | `magic` | ASCII `MNEMBLOB` |
| 8 | 1 | `version` | `0x01` |
| 9 | 1 | `kdf_id` | `0x01` = PBKDF2-HMAC-SHA256 |
| 10 | 1 | `aead_id` | `0x01` = AES-256-GCM |
| 11 | 1 | `payload_kind` | see §6.3 |
| 12 | 4 | `iterations` | u32, PBKDF2 iteration count |
| 16 | 16 | `salt` | fresh CSPRNG bytes, every encryption |
| 32 | 12 | `iv` | fresh CSPRNG bytes, every encryption |
| 44 | 4 | `ct_len` | u32, ciphertext length excluding tag |
| 48 | `ct_len` | `ciphertext` | AES-256-GCM ciphertext |
| 48+`ct_len` | 16 | `tag` | AES-256-GCM authentication tag |

Header length is 48 bytes. Total blob = `48 + ct_len + 16`.

**AAD = the full 48-byte header** (offsets 0..48). This binds version, algorithm
identifiers, payload kind, iteration count, salt, IV and length into the
authentication tag, so none can be tampered downward.

### 6.1 Blob presence

Erased flash reads `0xFF`. A blob is present iff the first 8 bytes equal
`MNEMBLOB`. Anything else — including all-`0xFF` — means "no payload", and the
feature stays invisible in the UI.

### 6.2 Parameter bounds — checked BEFORE any allocation or KDF work

This firmware runs with **no active watchdog** (`rp.WATCHDOG` appears only in
`rebootIntoBOOTSEL`). A hostile blob declaring a huge iteration count is a hang,
not an error message. Every field below is validated first, and any violation
fails closed with "payload unreadable":

- `version == 0x01`
- `kdf_id == 0x01`, `aead_id == 0x01`
- `payload_kind ∈ {0x01, 0x02, 0x03, 0x04}`. All four are admitted; `ms1` was
  signed off by the operator on 2026-08-07 (§12 item 6).
- `100_000 <= iterations <= 2_000_000`
- `0 < ct_len <= 8191` — one below the 8 KiB scan buffer, see below
- `48 + ct_len + 16 <= 65536` (fits the region)

The `ct_len` ceiling is 8191, not 8192, because `gui/scan.go:34` computes
`s.overflow = s.overflow || s.n == len(s.buf)` against an `8*1024` buffer:
overflow triggers when the buffer is exactly *full*. A payload of exactly 8192
bytes would pass every bound here, burn the full KDF, authenticate correctly,
and only then die in the classifier — a spec-legal blob that can never engrave.

The length arithmetic MUST be performed in unsigned arithmetic wider than 32
bits, or be otherwise overflow-checked. TinyGo's `int` is 32-bit on this target,
so `48 + ct_len + 16` evaluated natively wraps negative for `ct_len` near 2³² and
would pass a `<= 65536` test. A conforming implementation is protected by the
separate `ct_len <= 8191` check, but the region-fit check MUST NOT be relied on
alone — an implementation that "simplified" to it would admit a 4 GiB declared
length.

`ct_len` is authoritative. A UF2 block carries a fixed 256-byte payload, so the
written region is the blob followed by padding, followed by undefined sector
bytes. The device MUST bound the ciphertext by `ct_len` and MUST NOT infer
length from region contents.

### 6.3 Payload kinds — and which of these are actually secret

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

| Value | Kind | Secret? | Classifier route |
| --- | --- | --- | --- |
| `0x01` | UTF-8 constellation string, `md1` or `mk1` | no | codex32 |
| `0x02` | BIP-39 mnemonic, lowercase, single-space separated | **yes** | bip39 |
| `0x03` | `ms1` codex32 secret, standalone | **yes** | codex32 |
| `0x04` | bundle of constellation records — see §6.4 | **if it contains `ms1`** | per record |

All are forms `gui/scan.go`'s classifier already accepts, so no new engraving
path is required for any of them.

**`ms1` — standalone (`0x03`) or as a bundle record — is admitted.** This is a
deliberate reversal of the plaintext converter's refusal
(`crates/me-cli/src/lib.rs:59`), signed off by the operator 2026-08-07 (§12
item 6). The refusal was a property of the *plaintext* path, which had no
confidentiality; inside an authenticated encrypted envelope the objection does
not carry. The consequence is explicit and accepted: **the machine's flash holds
encrypted seed material, not merely an encrypted xpub** — see §2.2a.

### 6.4 Bundle container (`payload_kind = 0x04`) — NORMATIVE

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

The container is parsed **only after the GCM tag verifies**, so unlike the
header it is never parsed on unauthenticated input (§6.5).

Normative constraints, all checked before any record is acted on:

- **No trailing LF.** The last record is not followed by a separator. This makes
  the encoding canonical, which a test vector requires.
- **No CR.** A `0x0D` anywhere is a malformed bundle. CRLF is rejected, not
  tolerated.
- **No space or hyphen inside any record** (see above). A space-grouped record
  rejects the whole bundle.
- **No empty record.** This falls out of the rules above and independently
  rejects `\n\n`, a leading LF, and a trailing LF.
- **`1 <= record_count <= 24`.** See below — derived from the widget actually
  used, and sized to admit real multisig wallets.
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
bundle record MUST classify as `mdmkText` or a `codex32` secret; a BIP-39
mnemonic inside a bundle is rejected (that is `0x02`'s job, and mixing delivery
shapes buys nothing). An `ms1` record is admitted per §12 item 6.

If any record fails any check, **the entire bundle is rejected**. Partial
acceptance would leave the operator engraving an incomplete wallet backup while
believing it complete, which is the worst available outcome.

#### Why `record_count <= 24`, and why it is not 7

An earlier draft capped this at 7, derived from `ChoiceScreen`'s no-scroll limit.
**That was wrong on both the premise and the consequence**, and it is recorded
here because the error is instructive.

*The premise was wrong.* The fork already ships a paged, arbitrary-length card
list: `bundleReviewFlow` (`gui/bundle_flow.go:224`) uses
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
| **plaintext** (§6.4) | **after** the GCM tag verifies | produced by someone holding the passphrase |

`Open` returns an error without releasing plaintext on tag mismatch, so the
record container is parsed only on authenticated bytes. **That is why the bundle
container belongs in the plaintext and not in the header** — the opposite of
where a pre-authentication field would belong.

This is not a licence to trust plaintext blindly: §10.2.1's allow-list still
applies per record, because "authenticated" means "sealed by whoever knows the
passphrase", which is not the same as "safe".

## 7. Cryptographic construction — NORMATIVE

| Component | Choice | Rationale |
| --- | --- | --- |
| KDF | **PBKDF2-HMAC-SHA256**, iteration count per §7.1 | Already linked and exercised on-device by the SLIP-39 recovery path (`slip39/feistel.go:50`). Writing scrypt or Argon2 buys −0.8 to +3.6 bits at equal on-device wall clock — under one passphrase character — in exchange for new unaudited crypto on a funds path. Neither fits its own standard's recommended memory here: RFC 9106's *memory-constrained* Argon2id fallback is 64 MiB against ~452 KB free, and at ~256 KiB an RTX 4090's 72 MB L2 holds 288 concurrent working sets, so memory-hardness is paid for and not received. |
| AEAD | **AES-256-GCM** | Already linked (~52 KB, pulled in by `crypto/ecdsa`'s fips140 dependency, currently uncalled), so **zero marginal flash**. The payload is a few hundred bytes, so ChaCha20's software-speed advantage is irrelevant, and the threat is offline ciphertext attack rather than side-channel. |
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
me seal <payload> --out payload.uf2 [--iterations N]
```

**There is deliberately no `--addr` flag.** The target address is normative —
`0x10E00000`, fixed by §5 and read unconditionally by §10.1 — so any other value
produces a blob the device will never look at. Worse, §5's whole analysis exists
to keep the write clear of the signed image and inside physical flash: a
mis-specified address either overwrites the firmware directly or, past
`0x11000000`, **wraps to `0x10000000` and destroys it** (datasheet §5.5.2). The
flag would expose a destructive footgun with no legitimate use. If a test seam
is ever needed it MUST NOT be an operator-facing flag.

- Validates the input and sets `payload_kind` per §6.3. `md1`/`mk1` → `0x01`;
  a 12/15/18/21/24-word BIP-39 mnemonic with a valid checksum → `0x02`; `ms1` →
  `0x03` (admitted per §12 item 6, kept behind an explicit opt-in flag so that
  sealing a seed is never accidental); a record list → `0x04`.
- **Bundles (`0x04`).** Records are supplied as a list and joined with a single
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
- Generates the 12-word mnemonic, salt, and IV.
- Runs PBKDF2 and AES-256-GCM, assembles the §6 blob.
- Emits a `data`-family UF2 (`0xe48bff58`) targeting `0x10E00000`.
- Prints the 12 words to **stdout only**, never to a file, with a clear
  instruction to transcribe them and store them apart from the machine.
- Writes the `.uf2` with mode `0600`, matching `write_private` in
  `main.rs:375`.

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
2. Enter the existing 12-word BIP-39 entry flow.
3. Validate the BIP-39 checksum. Failure → "not a valid passphrase, check the
   words", return to entry. No KDF is run.
4. Run PBKDF2 with a progress indicator. This takes ~30 s and the screen must
   say so, or the operator will think the machine has hung.
5. AES-256-GCM open. **Tag mismatch → fail closed**, "wrong passphrase or
   damaged payload", return to entry. Never emit partial plaintext.
6. Classify the plaintext via `gui/scan.go`, then **allow-list the result** per
   §10.2.1 before acting on it. Anything outside the allow-list is "payload
   unreadable" — fail closed.
7. Wipe the derived key, the passphrase buffer, and PBKDF2 intermediates on
   every exit path, following the existing `wipeBytes` pattern
   (`gui/passphrase_flow.go:605`) and carrying the same honest caveat: TinyGo's
   GC may copy or retain, so this is defence in depth, not a guarantee.

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

| `payload_kind` | Permitted classification |
| --- | --- |
| `0x01` | `mdmkText` (via `codex32.ValidMD` / `ValidMK`) |
| `0x02` | a parsed BIP-39 mnemonic |
| `0x03` | a `codex32` secret (`ms1`) |
| `0x04` | **per record**: `mdmkText` or a `codex32` secret. A BIP-39 mnemonic inside a bundle is rejected. |

For `0x04` the allow-list runs **once per record**, and any single failure
rejects the whole bundle (§6.4).

Every other classification — explicitly including `debugCommand`,
`addressText`, and output descriptors — MUST be treated as "payload unreadable".
The check MUST be an allow-list, not a deny-list: a deny-list silently admits
whatever branch the classifier grows next.

**The classification MUST still be cross-checked against `payload_kind`**, even
though every kind is now admitted. The device routes on *content*, and
`codex32.New` (`codex32/codex32.go:98`) accepts secret shares, so a header byte
does not bind what actually engraves. `payload_kind` is a claim by the sealer,
not a guarantee.

The cross-check is no longer an `ms1` policy gate — it is a **mislabelling
detector**. A blob whose header says `0x01` (public card) but whose content is
an `ms1` secret is malformed, and the operator has been told they are engraving
a public card when they are about to cut a seed onto steel. Reject the mismatch:
the header and the content must agree about whether this plate is secret.

### 10.2.2 Bundle session — unlock once, cut many

A `0x04` bundle is engraved over one **session**: unlock once, then cut each
plate, swapping steel between them.

```
unlock ──► [plate list] ──► pick a record ──► engrave ──► back to list
               │                                              │
               │◄─────────────── mark cut ────────────────────┘
               │
               └──► [Lock] ──► wipe plaintext, return to main menu
```

- The plate list shows one entry per record, labelled by its **classified**
  type and index (`ms1`, `mk1 1/2`, `md1 2/3`), not by anything the sealer
  asserted.
- Records already cut this session are marked. The mark is a **convenience, not
  a guarantee** — it does not survive a power cut, and the UI must not imply it
  does.
- A **Lock** action wipes the plaintext and returns to the main menu. Leaving
  the bundle flow by any path — including Back — MUST wipe.
- Wiping follows the existing `wipeBytes` pattern with the same honest caveat as
  §10.2 step 7: TinyGo's GC may copy or retain, so this is defence in depth,
  not a guarantee.

#### The cost, stated plainly

This is a **real weakening of §2.1's first claim** and belongs in the threat
model, not a footnote. Single-record delivery holds plaintext for one engraving.
A six-plate bundle at roughly 21 minutes per plate holds decrypted seed material
in RAM for on the order of **two hours**, across plate swaps, with the machine
likely unattended for much of it.

Re-prompting per plate would shrink that window, at the cost of retyping twelve
words and eating the ~30 s KDF six times. The operator chose the session model
(2026-08-07); this records what was traded for it.

Mitigations that cost no re-typing:

- Wipe on **every** exit from the bundle flow, not only on Lock.
- An idle timeout that wipes and returns to the lock screen — value is §12 item 8.
- Never render a secret record's contents on the plate list; show type and index
  only.

### 10.3 UI constraints to respect

- `layoutNavigation` indexes a fixed `[3]int` (`gui/gui.go:1857`) — **a fourth
  nav affordance panics.** Back / Clear / OK is the whole budget. The bundle
  plate list must fit Back / Lock / OK exactly.
- `ChoiceScreen` does not scroll and draws over its own title past ~7 entries
  (`gui/gui.go:1455`, pinned by `gui/freetext_speed_test.go:31`). **This bounds
  layout variants per plate, NOT records per bundle** — the distinction that an
  earlier draft got wrong (§6.4). The bundle plate list MUST use
  `bundleReviewFlow`'s paged shape (`gui/bundle_flow.go:224`), which handles an
  arbitrary number of entries within the three-slot nav budget.

## 11. Testing

### 11.1 Rust (primary)

- The §11.4 canonical vector, byte-exact. This is the artefact the Go port is
  bound to.
- Round-trip seal/open.
- Passphrase normalisation (§8.1) byte-exactness.
- `payload_kind` classification per §6.3: `md1`/`mk1` → `0x01`, a checksum-valid
  BIP-39 mnemonic → `0x02`, `ms1` → `0x03`, a multi-record bundle → `0x04`. A
  mislabelled input is refused at seal time, not emitted.
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

- Decrypt **both** §11.4 vectors (A and B) to the expected plaintext. Vector B
  is not optional — it is the only test that catches a hardcoded iteration count.
- Every §6.2 bound violation fails closed *before* the KDF runs — asserted by
  timing or by instrumenting the KDF call, not merely by return value. The case
  set MUST include `ct_len = 0xFFFF_FFF0`, which a 32-bit native-`int` region-fit
  check would wrap negative and accept.
- `payload_kind` outside `{0x01, 0x02, 0x03, 0x04}` is rejected before the KDF.
- Tag mismatch yields no plaintext.
- BIP-39 checksum rejection happens without invoking the KDF.
- Absent/erased region (all `0xFF`) reports "no payload", not an error.
- **Classifier allow-list (§10.2.1).** A blob whose plaintext is
  `command: lock-boot` MUST be rejected as "payload unreadable", and
  `Platform.LockBoot` MUST NOT be reached — asserted with a fake/instrumented
  platform that fails the test if `LockBoot` is called, not by return value
  alone. Same for an `addressText` plaintext and an output descriptor.
- **Kind/content cross-check (mislabelling).** An `ms1` codex32 secret labelled
  `payload_kind = 0x01` MUST be rejected as malformed — not because `ms1` is
  forbidden (it is not), but because the header claims a public card while the
  content is a secret. The reverse — a `mdmkText` labelled `0x03` — MUST also be
  rejected.
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
  never on a return value. Carry the honest caveat from §10.2 step 7: this
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

### 11.4 Canonical test vector — `beef` / `bacon`

The normative cross-implementation vector. The Rust implementation produces it;
the Go port MUST decrypt it to the exact plaintext.

**The fixed salt and IV below exist ONLY because a test vector must be
deterministic. This is the sole exemption from §7.2. Production code MUST NOT
have a code path that accepts a caller-supplied salt or IV.**

| Field | Value |
| --- | --- |
| passphrase | `beef` × 12 (space separated) |
| plaintext | `bacon` × 24 (space separated), 143 bytes |
| `payload_kind` | `0x02` (BIP-39 mnemonic) |
| `iterations` | `100000` (the §6.2 minimum, so tests stay fast and legal) |
| `salt` | `beefbeefbeefbeefbeefbeefbeefbeef` |
| `iv` | `bac0bac0bac0bac0bac0bac0` |
| derived key | `615ad9b781b1ad6105d9dffb135d1bf17ebab286c560f26912ee815836e7ad1e` |
| tag | `84c39ba137f886a1a8ff835994aca24d` |
| blob length | 207 bytes (48 header + 143 ciphertext + 16 tag) |
| blob sha256 | `53d4991a41994089fbbe35e1c576335d8d6e82904ecd531257397d1780e16bb9` |

Full blob:

```
0000: 4d4e454d424c4f4201010102000186a0beefbeefbeefbeefbeefbeefbeefbeef
0020: bac0bac0bac0bac0bac0bac00000008f3d53f36eb6d5933d2fc5f6a555eca293
0040: 32fc4f611f238b42bb3cffdd6fff3e47b21e13649d104fe215b37f2b8454f777
0060: d478233321d9638e17c6b68a654abdd47ea80827e1b3c14c23c542ac291ca816
0080: e65b5a8498ba6311a6fe45b65a93651f9541ef4460c053a494b940a005c67842
00a0: 5bb2cf4aee8d47737ed527020643e7cbf59d3fca90e418a1e551cfedde831c84
00c0: c39ba137f886a1a8ff835994aca24d
```

Loadable form: one 512-byte UF2 block, `targetAddr=0x10E00000`,
`familyID=0xe48bff58`, `payloadSize=256`, sha256
`c58b684e6d206f599f4a3408e626534af0ce914aa157a93d9e05ab62cc2865fc`. **Payload
bytes beyond the 207-byte blob are `0x00`** — the sha256 above pins zero
padding, and `0xFF` padding will not match it.

#### Vector B — same inputs, `iterations = 100001`

Identical to vector A in every field except the iteration count. **MUST decrypt
successfully.** Its only purpose is to fail any implementation that hardcodes
the iteration count instead of reading it from the header.

| Field | Value |
| --- | --- |
| `iterations` | `100001` |
| derived key | `003800ae6cec47cd4b34bb264c6bbb1156d806516ad1ab88391e479d14d8776f` |
| tag | `ad86e19a59d82a8ca1de607e27450990` |
| blob sha256 | `edcba9c5125060a2ae35dc4e99b9d46030e3672409917e4bf12d95d81d15d4fe` |

```
0000: 4d4e454d424c4f4201010102000186a1beefbeefbeefbeefbeefbeefbeefbeef
0020: bac0bac0bac0bac0bac0bac00000008ff1151bdaafc0b11580b448b74e053b95
0040: 9cfefd9ade8ab990661772534a14bfc618a89dc35fe20bf2298bfa6ecf0b9e82
0060: 9733cc0f85c30bf7aa3bffddca85c1a627b4e0d78feefc66d638829429553979
0080: 3a714680d3882e9ed5debca68483241d364eb29af2e0846415603edbf158cfb8
00a0: 34c362053e61e33663f62c4984249d1d16e34e27b402bad982bf3e3c0546cbad
00c0: 86e19a59d82a8ca1de607e27450990
```

Confirmed by execution: a KDF hardcoded to 100000 is **rejected** by vector B,
while the correct key decrypts it.

Vectors A and B deliberately share a salt and IV. That is safe **because they
derive different keys** (different iteration counts), and GCM's nonce rule binds
per key. Holding salt and IV fixed is what isolates the iteration count as the
single variable, which is the vector's whole purpose.

#### Vector C — bundle, `payload_kind = 0x04`

Six **canonical** records from a real `bip84` bundle for the `bacon`×24 seed
(`mnemonic bundle --group-size 0`), LF-separated with no trailing LF.

| Field | Value |
| --- | --- |
| passphrase | `beef` × 12 (as A and B) |
| `payload_kind` | `0x04` |
| `iterations` | `100000` |
| `salt` | `beadbeadbeadbeadbeadbeadbeadbead` — **distinct from A/B** |
| `iv` | `cafecafecafecafecafecafe` — **distinct from A/B** |
| derived key | `19c78c5535ad24349f75fb6ca9a59c939ea885c126cc4909eb2cdc0c26add40e` |
| tag | `6d41656b6a84aca32e67ecb9b970cc5c` |
| plaintext | 472 bytes, sha256 `b0f68bacd6b9e91e22da2cb4b5cef0a6b367fda3159fd6a26f1e2724959a04e0` |
| blob | 536 bytes, sha256 `45c31f0096175da31cbc61a2e11a026b6766a2491da5a90291db1b7c829e2536` |

Header: `4d4e454d424c4f4201010104000186a0beadbeadbeadbeadbeadbeadbeadbeadcafecafecafecafecafecafe000001d8`

Records, in order:

| # | Type | Bytes | Classifies as |
| --- | --- | --- | --- |
| 0 | `ms1` | 75 | `codex32.String` (secret) |
| 1 | `mk1` | 111 | `mdmkText` |
| 2 | `mk1` | 80 | `mdmkText` |
| 3 | `md1` | 67 | `mdmkText` |
| 4 | `md1` | 67 | `mdmkText` |
| 5 | `md1` | 67 | `mdmkText` |

**An earlier draft of this vector used the space-grouped display form** —
records of 89/133/95/80/80/80 bytes totalling 562 — taken straight from
`mnemonic bundle`'s default `--group-size 5` output. Every one of those records
classifies as **unknown format** on the device, so the "positive" bundle test
could never have passed, and the nine negative cases built on it would have been
vacuous. It was caught at R0 round 3. The lesson is the standing one: never take
a value from how a tool *prints* it; trace it into the consumer that will parse
it. The canonical figures above were verified by running the real
`seedhammer.com/codex32` package against both forms.

**Vector C's salt and IV differ from A and B by necessity, not style.** With the
same passphrase and iteration count, reusing A's salt would derive A's exact key
— and with A's IV that is `(key, nonce)` reuse across two different plaintexts,
which under GCM leaks the authentication key and enables forgery. This was
caught during authoring: the first computed vector C did reuse both. The test
suite MUST assert that **no two shipped vectors share a `(derived key, iv)`
pair**; that assertion is what catches the same mistake in future vectors.

Vector C's record 0 is an `ms1` secret. Since §12 item 6 was signed off
(2026-08-07) this is a **positive** vector: all six records decrypt, classify,
and are engravable. Note that it therefore contains real — if publicly
known — seed material for the `bacon`×24 test seed, which is why that seed and
not a private one.

#### Why these words — both checksums are valid, and that is not obvious

Verified against `bip39/wordlist.txt` (2048 entries; `beef` = index 160,
`bacon` = index 138) with a checker self-tested against `abandon`×11+`about`,
`zoo`×11+`wrong`, `abandon`×23+`art` (accepted) and `abandon`×12 (rejected):

- `beef` × 12 — 128 bits entropy + 4-bit checksum `0000`. **Valid.** A 1-in-16
  coincidence.
- `bacon` × 24 — 256 bits entropy + 8-bit checksum `10001010`. **Valid.** A
  1-in-256 coincidence.

An all-identical mnemonic is normally checksum-invalid; `abandon`×12 is the
counter-example and is included in the test set as a negative case. Because both
of these are valid, they exercise the §10.2 step-3 checksum gate positively
rather than tripping it.

#### Required assertions

Positive: the blob decrypts to `bacon`×24 under `beef`×12.

Negative — each MUST fail, and the first two MUST fail *without invoking the
KDF*:

| Case | Expected |
| --- | --- |
| passphrase `abandon`×12 | rejected at the BIP-39 checksum, no KDF run |
| passphrase `beef`×11 + `bacon` | rejected at the BIP-39 checksum, no KDF run |
| `iterations` altered to `50000` in the header | AEAD tag mismatch (AAD binding) |
| any single ciphertext byte flipped | AEAD tag mismatch |
| `magic` altered | reported as "no payload", not as an error |

The `beef`×11 + `bacon` case is the important one: it is a valid-length
mnemonic of real words that differs from the passphrase in one position, and it
is checksum-**invalid** (confirmed). A checksum gate that passes it is broken.

The two "no KDF run" assertions MUST be enforced by instrumenting or timing the
KDF call. Asserting only on the return value passes over exactly the defect it
is meant to catch — an implementation that runs a 30-second KDF before checking
the checksum returns the same value.

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
6. **`ms1` admission — RESOLVED 2026-08-07, operator signed off. ADMITTED.**
   The plaintext converter refuses `ms1` by design
   (`crates/me-cli/src/lib.rs:59`). That refusal was a property of the
   *plaintext* path, which had no confidentiality; inside an authenticated
   encrypted envelope it does not carry. `0x03` and `ms1` bundle records are
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
   24, derived from `bundleReviewFlow`'s paged list (`gui/bundle_flow.go:224`).
   Remaining work is implementation, not decision: the plate list must use the
   paged shape, not `ChoiceScreen`.
8. **Session idle wipe (§10.2.2) — REQUIRED for v1, value unset.**
   Per §2.2 item 9 an open session is defended by physical custody alone, and
   the screensaver does not unwind the flow. A timeout that **wipes and returns
   to the lock screen** is therefore a v1 control, not an optional extra.
   The timer source is already identified and in use — `gui.go:2801`
   `idleTimeout = 3 * time.Minute`, driven by `time.Now()` and
   `ctx.WakeupAt`/`Platform.AppendEvents` in `Run`'s frame loop. Monotonic
   elapsed time is all this needs; no RTC is involved. **Only the value is
   open**, and it must account for legitimate mid-session pauses — a plate swap
   takes minutes, so 3 minutes is likely too aggressive to reuse verbatim.

## 13. Non-goals

- On-device encryption. The device decrypts only; `crypto/rand` panics there.
- A runtime USB data path. The bootrom does the transfer.
- Firmware flash writes of any kind.
- A partition table.
- Any device-side rate limiting, PIN counter, or lockout — §2.2 item 3 explains
  why these would be theatre.
- Replacing NFC ingest. This is an additional path, not a substitute.
