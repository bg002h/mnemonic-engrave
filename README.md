# Mnemonic Engrave (`me`)

Bridges the m-format constellation onto a [SeedHammer II](https://seedhammer.com) engraving machine: it turns the **public** backup strings — [`md1`](https://github.com/bg002h/descriptor-mnemonic) (wallet descriptor / policy) and [`mk1`](https://github.com/bg002h/mnemonic-key) (xpubs) — into an NFC NDEF payload the device can scan, and **refuses** the secret [`ms1`](https://github.com/bg002h/mnemonic-secret) so seed entropy never travels over RF.

> **Status: converter (`me`) implemented; firmware support pending.**
> The host-side converter validates a constellation string with the sibling
> codecs and emits a TLV-wrapped NDEF Text record for `md1`/`mk1`; `ms1` is
> refused (enter it by hand on the device's air-gapped CODEX32 keypad). The
> SeedHammer firmware changes that make the device recognize `md1`/`mk1` are a
> separate, future workstream. See
> [`design/SPEC_seedhammer_engrave.md`](design/SPEC_seedhammer_engrave.md) for
> the full, architect-reviewed design and
> [`design/FOLLOWUPS.md`](design/FOLLOWUPS.md) for open items.
>
> **If you intend to engrave a real seed via the encrypted sealed-payload path,
> read [Security limitation](#security-limitation--the-sealed-payload-wipe-is-incomplete)
> first — the wipe is incomplete and can be prevented from running.**

## Security limitation — the sealed-payload wipe is INCOMPLETE

**If you use the encrypted sealed-payload path to engrave a real seed, the
machine does not erase every copy of it, and under some conditions does not
erase anything at all.** This is a known, open, documented state of the build,
not a theory. Read this before you put a seed you care about into it.

The design intends that a decrypted secret is wiped as soon as its plate is cut
or skipped, with a 3-minute idle timer as a backstop. In the current build:

- **Not every copy is wiped.** The wipe removes the secret *record*. Getting a
  record onto a plate makes further copies — on the mnemonic engrave path, in
  the key-derivation working state, in the word-splitting and keyboard buffers,
  and in the uppercased string the plate's QR is built from. Those are not
  wiped.
- **~35 KB across ~81 reachable objects survives every wipe and has not been
  identified.** Nobody has established whether it holds seed material.
- **The idle wipe can silently never run.** The timer only fires when the
  machine is *idle*, and the idle clock is refreshed by **any** input event —
  including one that resolves to no actual input. A touch panel reporting
  spurious readings (protective film, moisture, debris, driver noise) keeps the
  machine permanently non-idle, so there is **no countdown, no wipe, and no
  indication that either was skipped**. Measured: 100,000 spurious touch polls
  over ~1000 s produced zero warnings and zero wipes, against a control that
  warned at 3:00.

**What actually protects you, therefore, is physical custody — not the wipe.**
This device is deliberately debuggable: `debug enable: 1` and
`secure debug enable: 1` are set, and BOOTSEL is not disabled, so anyone holding
the machine can read SRAM over SWD with no passphrase. Treat the wipe as a
convenience. **Power the machine down when you are done, and never leave it
unattended with a session open.**

Tracked as F-88, F-90, F-94, F-103, F-104 and F-109 in
[`design/FOLLOWUPS.md`](design/FOLLOWUPS.md), and stated normatively in
[`design/SPEC_encrypted_payload_delivery.md`](design/SPEC_encrypted_payload_delivery.md)
§2.2 item 16. All are scheduled to a **post-merge polish and hardening** phase
that runs *after* the current tag, by explicit decision — the tag is
`v0.0.0-g<sha>`, which marks a build rather than a product.

### How this got shipped open, recorded plainly

The encrypted-payload feature had a heavy review process: architect loops to
0 Critical / 0 Important on every design document, mutation testing of the test
suites, independent whole-diff execution reviews, and hardware validation on the
real machine. It found and closed real defects — a doubled wipe timer, a
rendered seed left in the frame buffer, plate geometry never zeroed after a cut,
an abandoned job's resume state.

It did not produce a complete wipe, and the specific gaps are worth naming
rather than generalising:

- **§10.2.4's idle wipe was designed in a single top-tier consult**
  ([`design/CONSULT_b2b_idle_timer_design.md`](design/CONSULT_b2b_idle_timer_design.md),
  2026-08-09), and the B2b implementation plan was written against it. Keying
  the timer on the session bracket rather than on a residency predicate was the
  right call and survives. But that design did not define what *idle* means at
  the level the code implements it — the clock is refreshed by raw event
  presence — and that is **F-103**, the defect that lets the wipe silently never
  run. A one-pass consult on a funds-safety control was thin for what it was
  carrying.
- **The same timer shipped a second defect that only hardware found.** The arm
  edge was processed one wakeup late, so the window ran at double its specified
  length — 6:00 instead of 3:00, deterministically. No host test caught it;
  three cycles on the machine did. Fixed.
- **Neither the spec nor the plan ever enumerated which copies of a decrypted
  secret exist.** The wipe was specified against the *record*. The copies made
  downstream of it — engrave path, KDF working state, word-split and keyboard
  buffers, the uppercased QR string — were discovered afterwards, one at a time,
  by review and by measurement. That is **F-88, F-90, F-94, F-104**, and the
  ~35 KB in **F-109** that still has no name.

Those items were found, written down, and then rescheduled rather than fixed,
and the whole-diff review that reads all three phases at once is deliberately
scheduled *after* this release. Review effort is not a substitute for the
remaining work, and this section exists so the gap is visible to anyone using
the build rather than only to whoever reads the follow-up file.

## What it does

- `md1` / `mk1` (public) → validate → NDEF Text record → write to an NFC tag / push from a phone → SeedHammer II scans → engrave.
- `ms1` (secret) → refused over NFC; type it on the device's CODEX32 keypad.

Validation is **per-string** (a single chunk of a multi-chunk card validates on its own) and **pristine-only** — a string that needed BCH error-correction is rejected rather than engraved.

## Usage

```sh
# Validate an md1/mk1 string from stdin and emit the NDEF bytes:
echo "md1yqpqqxqq8xtwhw4xwn4qh" | me --hex      # hex to stdout
echo "mk1..."                   | me --out wallet.ndef   # raw NDEF to a file

# ms1 is refused (exit 3), with on-device-entry guidance:
echo "ms1..."                   | me --stdout
```

Input is read from **stdin** (or `--in <file>`) — never a positional argument, so a secret can't leak into `ps`/shell history. NDEF bytes go to stdout (`--stdout` / `--hex` / `--base64`) or `--out <file>`; all human-readable text goes to **stderr**.

### Plate previews

`me bundle --preview <DIR>` renders each public plate to an SVG (or PNG, with
`--png`) via the `me-preview` sidecar. For safety the sidecar is discovered
**only alongside the `me` executable** — release archives ship the two together —
and `me` **does not search `$PATH`** for it, so a `me-preview` planted on `$PATH`
can never be handed your public payload or write into the preview directory. For a
non-standard install, point at the sidecar explicitly:

```sh
ME_PREVIEW_BIN=/path/to/me-preview me bundle --preview ./plates
```

If no sidecar is found, previews are skipped (a note is printed) and the manifest
is still emitted. A set-but-missing `ME_PREVIEW_BIN` is a hard error (exit 2).

## Verifying releases

Release archives (`mnemonic-engrave-<tag>-<os>-<arch>.tar.gz` / `.zip`) bundle
`me` + the `me-preview` sidecar and are published with a `SHA256SUMS` file that
is [minisign](https://jedisct1.github.io/minisign/)-signed (`SHA256SUMS.minisig`).
The signing public key is pinned here (and shipped in every archive as
`minisign.pub`):

```
untrusted comment: minisign public key CA39ECB257009A0F
RWQPmgBXsuw5yi8W0SfDr8KF+IqY/Z5U2p724emSODS1UPfJBP3agbKW
```

To verify a download:

```sh
# 1. Verify the checksum file's signature against the pinned public key:
minisign -Vm SHA256SUMS -P RWQPmgBXsuw5yi8W0SfDr8KF+IqY/Z5U2p724emSODS1UPfJBP3agbKW

# 2. Verify the binaries against the now-trusted checksums:
sha256sum -c SHA256SUMS --ignore-missing
```

(Step 1 is equivalent to `minisign -Vm SHA256SUMS -p minisign.pub` using the
bundled key file.) Key rotation is an explicit, auditable change to this section.

**Supported platforms (v0.3.0):** linux `amd64`/`arm64`, macOS `amd64`/`arm64`,
windows `amd64`. **windows/arm64 is not supported in v0.3.0** (no
GitHub-hosted runner; cross-MSVC is impractical).

## The constellation

- [`md-codec`](https://github.com/bg002h/descriptor-mnemonic) — wallet descriptors / policies (`md1`).
- [`mk-codec`](https://github.com/bg002h/mnemonic-key) — xpubs (`mk1`).
- [`ms-codec`](https://github.com/bg002h/mnemonic-secret) — secret entropy (`ms1`, BIP-93 codex32).
- **`mnemonic-engrave`** (this repo) — engrave the bundle onto SeedHammer II.

## Custom firmware tooling

`scripts/` also carries the tooling for running self-built firmware on a retail
SeedHammer II — `pico2-bootkey-rehearsal.sh` (rehearsal phases plus read-only
device checks) and `sign-firmware.sh` (signs a UF2 and proves the signature
offline before it reaches hardware), with a hardware-free regression harness in
`scripts/test/`.

The **procedure** those scripts implement is documented in the firmware fork:
[bg002h/seedhammer → `docs/custom-firmware.md`](https://github.com/bg002h/seedhammer/blob/main/docs/custom-firmware.md).
It burns your own boot key into an OTP slot and is **irreversible** — read it
before running anything here.

## License

Dual-licensed, at your option, under either the [MIT License](LICENSE) or the
[Unlicense](UNLICENSE) public-domain dedication — SPDX `MIT OR Unlicense`. Use
the Unlicense for maximal freedom, or MIT where a public-domain dedication
isn't accepted.
