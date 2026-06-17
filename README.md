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

## License

Dual-licensed, at your option, under either the [MIT License](LICENSE) or the
[Unlicense](UNLICENSE) public-domain dedication — SPDX `MIT OR Unlicense`. Use
the Unlicense for maximal freedom, or MIT where a public-domain dedication
isn't accepted.
