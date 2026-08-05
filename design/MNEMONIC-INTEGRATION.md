# RECON — integrating the mnemonic constellation with the SeedHammer II

Status: **open question, no decision made.** Captured 2026-08-05 so the findings
below are not re-derived. The end goal is deliberately not settled — see §6.

## 1. The question

Move material produced by the `mnemonic-*` CLIs onto the SeedHammer II for
engraving, **without NFC** (this operator has no NFC hardware yet).

## 2. What the host side actually emits — measured, not assumed

`mnemonic bundle` writes to **both** streams, and the split matters:

| Stream | Content |
|---|---|
| **stdout** | the machine-readable `ms1:` / `mk1:` / `md1:` strings — the payload |
| **stderr** | a human-readable **engraving-card panel** — a rendering of the same data for a person to transcribe |

From `mnemonic bundle --help`:

> `--quiet` — *"Suppress the human-readable engraving-card panel on stderr. The
> stdout `ms1` / `mk1` / `md1` output is unchanged. Use for piping into other
> tooling"*

There is also `--json`: *"Emit a single JSON object on stdout instead of the
multi-line `ms1: … / mk1: … / md1: …` text form."*

**Correction to the initial framing:** the material to engrave is on **stdout**;
stderr carries the pretty panel. `--quiet` exists precisely to strip it when
piping. Other stderr traffic is warnings and deprecation notices
(`docs/manual/src/45-foreign-formats.md`, `crates/mnemonic-toolkit/src/mlock.rs`).

Constellation repos, one level up from this one at `/scratch/code/shibboleth/`:
`mnemonic-toolkit` (bin `mnemonic`), `mnemonic-secret` (ms1), `mnemonic-key`
(mk1), `descriptor-mnemonic` (md1), `mnemonic-gui`.

## 3. The transport — BOOTSEL flash blob

**USB carries power only while the firmware runs.** The only USB chip is an
`ap33772s` USB-PD controller on I2C (`cmd/controller/platform_sh2.go:211`); there
is no USB data interface. The one UART goes to the stepper drivers. The terminal
in `cmd/controller/debug_sh2.go` is behind a `debug` build tag and absent from
production builds.

The usable channel is **BOOTSEL**, where the mask ROM serves `picotool`:

```
flash mapped at:  0x10000000   (XIP — readable as memory, no flash driver)
image occupies:   0x100000f8 … 0x1012f900   ≈ 1.24 MB
flash size:       16 MB                      → ~14.7 MB free
```

```sh
mnemonic bundle … --quiet > payload.bin
picotool load -t bin -o 0x10800000 payload.bin   # device in BOOTSEL
```

Because flash is execute-in-place, the firmware reads that region by
dereferencing a pointer — **no flash driver, no erase, no write path.** That is
the key property: adding flash *writes* was the bricking risk in the discarded
secret-transfer design, and this needs none.

Firmware side: magic header + length + CRC (so "nothing loaded" is
distinguishable from "loaded", and a partial write is caught), plus an entry
point that reads it, shows the existing confirm screen, and engraves on approval.
Estimated 150–250 lines plus tests, touching no existing path.

**Workflow per payload:** hold button while plugging USB → `picotool load` →
unplug → power up on the machine's own supply → select on device.

**Zero-code alternative for one-offs:** put the text in a Go constant, rebuild,
sign, flash. No new firmware surface; costs a full build cycle per payload. The
blob only pays off if this is done repeatedly.

## 4. Properties and limits of the blob channel

- The region sits **outside the secure-boot signature**, so anything with BOOTSEL
  access can rewrite it. It is *displayed*, not *authenticated*. The confirm
  screen is the real defence: the operator reads exactly what will be cut.
- **It persists** until overwritten — a record of what was engraved, sitting in
  flash. Overwriting with zeros is one more `picotool load`.
- Firmware **cannot currently write its own flash** (no such path found), which
  is what keeps the verify-then-boot chain in §5 intact. Adding an erase would
  weaken it.

## 5. Attestation — what is and is not provable

- **The version on screen proves nothing.** `v0.0.0-g<sha> (UNLOCKED)` is drawn
  *by* the firmware; hostile firmware prints whatever it likes.
- **Secure boot proves something narrower but real:** the bootrom checks the
  signature at every boot against the OTP key hashes, so whatever runs is signed
  by SeedHammer's key or the operator's. Enforced by mask ROM.
- **The stored image can be verified:** `picotool verify <signed.uf2>` in BOOTSEL
  is trustworthy because the mask ROM does the reading — the firmware is not
  running to lie about itself.
- **Residual gap:** verify tells you what is *stored*; you then reboot and trust
  that what is stored is what runs, which secure boot enforces. There is no
  runtime challenge-response — RP2350 exposes no attestation key to query while
  the UI is up.

## 6. Open questions — decide before designing

1. **Which representation crosses into the device?** The raw
   `ms1`/`mk1`/`md1` strings, or the `--json` envelope? JSON is self-describing,
   so the firmware could distinguish an `md1` from an `mk1` and select the right
   plate layout automatically — the difference between "engrave this text" and
   "engrave this bundle".
2. **Is the free-text plate even the right target?** `mnemonic-engrave` already
   has `me bundle` and the Go sidecar reusing upstream SeedHammer curve math, and
   the SH2 has dedicated bundle/descriptor plate programs. Aiming the blob at
   those would give correct plate layouts instead of a wall of characters.
3. **Which record types are admissible?** `ms1` is **secret** material and the
   operator has explicitly ruled secrets off this channel (2026-08-05).
   `mk1`/`md1` are not secret. `me` already refuses `ms1` — precedent to follow,
   and the admission rule should be explicit rather than implied.
4. **Where does the split live** between the host CLI, `me`, and the firmware?
   The host could pre-render, or the firmware could interpret.

## 7. Discarded, with reasons

**Secret material over this channel.** Considered and dropped by the operator.
For the record, the blockers were real: the blob sits in dumpable flash, so it
would need AEAD; the key would have to come from a typed passphrase, so the KDF
becomes the security boundary; and the memory-hard KDFs want far more RAM than
the **16 KB stack** allows (`-stack-size 16kb` in `flake.nix`). PBKDF2 exists
on-device (`bip39/bip39.go:225`, 2048 rounds) but is weak against GPU attack.
Erasing after use would also require adding the flash-write path §4 relies on not
existing. For material that must stay secret, typing it or generating it
on-device remains strictly better.
