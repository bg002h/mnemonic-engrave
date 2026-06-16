# mnemonic-engrave (`me`)

A host-side CLI that bridges the **m-format constellation** onto a [SeedHammer II](https://seedhammer.com) engraving machine. It turns the **public** backup strings — [`md1`](https://github.com/bg002h/descriptor-mnemonic) (wallet descriptor / policy) and [`mk1`](https://github.com/bg002h/mnemonic-key) (xpubs) — into an NFC NDEF Text payload the device can scan, and **refuses** the secret [`ms1`](https://github.com/bg002h/mnemonic-secret) so seed entropy never travels over RF.

> **Status:** the converter works today; the SeedHammer firmware changes that make a device recognize `md1`/`mk1` are a separate, in-flight workstream. See the [project repo](https://github.com/bg002h/mnemonic-engrave) for the full design and status.

## What it does

- `md1` / `mk1` (public) → validate (via the `md-codec`/`mk-codec` BCH checksums) → emit a TLV-wrapped NDEF Text record → write to an NFC tag / push from a phone → SeedHammer II scans → engrave.
- `ms1` (secret) → **refused** over NFC; type it on the device's air-gapped CODEX32 keypad.

Validation is **per-string** (a single chunk of a multi-chunk card validates on its own) and **pristine-only** — a string that needed BCH error-correction is rejected rather than engraved.

## Usage

```sh
# Validate an md1/mk1 string from stdin and emit the NDEF bytes:
echo "md1yqpqqxqq8xtwhw4xwn4qh" | me --hex            # hex to stdout
echo "mk1…"                     | me --out wallet.ndef # raw NDEF to a file
echo "md1…"                     | me --hex --echo      # also echo the validated string to stderr

# ms1 is refused (exit 3), with on-device-entry guidance:
echo "ms1…"                     | me --stdout
```

Input is read from **stdin** (or `--in <file>`) — never a positional argument, so a secret can't leak into `ps`/shell history. NDEF bytes go to stdout (`--stdout` / `--hex` / `--base64`) or `--out <file>`; all human-readable text goes to **stderr**.

## License

MIT.
