# Hardware inventory — RP2350 boards on this bench

**Why this exists.** Chipids were scattered across four documents and a Go doc
comment (`CONTINUITY_2026-08-07.md:64`, `CONTINUITY_2026-08-07b.md:60`,
`IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseA.md:515`,
`cmd/sealread/main.go:18`), with no single place to check "which board is this?"
On 2026-08-07 **two RP2350s were in BOOTSEL simultaneously** — exactly the state
`cmd/sealread`'s checklist says to resolve before flashing — and identifying them
meant reading `picotool` output against notes in three files.

`tinygo flash` and `picotool load` target whatever is in BOOTSEL. Confirm the
chipid **before** every flash:

```
lsusb | grep 2e8a:000f          # expect exactly ONE line
picotool info -a                # or: picotool info -a --bus N --address M
```

---

## The boards

| chipid | board | part | flash | secure boot | notes |
| --- | --- | --- | --- | --- | --- |
| `0x77c483b745abf55c` | **SeedHammer II** | RP2350**B**, QFN80, rev A4 | 16 MB | **1** — own key | the real machine |
| `0x66d3d60ff20abf2f` | Pico 2 (rehearsal) | RP2350A, QFN60 | 4 MB | 1 — rehearsal key | boot-key rehearsal, 2026-08-03 |
| `0xb3d19289d3ec3f0e` | **Pico 2 W** | RP2350A, QFN60, rev A2 | 4 MB | **0** | blank; WiFi; LED differs — see below |

### `0x77c483b745abf55c` — SeedHammer II

Measured 2026-08-07 via `picotool info -a`:

```
revision: A4   package: QFN80   secure boot: 1   debug enable: 1
image type: ARM Secure          signature: verified
```

Its OTP trusts `~/.sh2/sh2-boot-key.pem`. Verified independently on 2026-08-07
by hashing the pubkey `picotool` read off the device:

```
sha256(X‖Y) = 846aa289f2f317e55ff03f90555132302842cff2f68ee45712834a25d64cabb4
```

which matches the value recorded in `CONTINUITY_2026-08-07b.md:87`, and matches
the compressed pubkey `sh2-flash` embeds when signing
(`03` ‖ `3b73217292f1205a44c0e1907d1c074256ca8b67f6c6ef21e662092b204bcc20`). So an
image built by `sh2-flash` will boot on this machine — provable offline, before
flashing.

**QFN80 means RP2350B.** §7.1's PBKDF2 rate of 9,715 it/s was measured on an
RP2350**A**; the rate on this part is still unmeasured (SPEC §12 residual, and
continuity §5 item 2). Grab it on the next flash trip.

**This is the only board that can close F-73**, which needs the payload region at
`0x10E00000` — 14 MB in — to actually exist.

### `0x66d3d60ff20abf2f` — Pico 2, boot-key rehearsal

Its OTP holds the **rehearsal** keys from `mnemonic-engrave/rehearsal-work/`
(X‖Y sha256 `17644fb6…`), **not** the SH2 key — the SH2 key is unrelated and this
board does not trust it. Used for the 2026-08-03 rehearsal and for Phase A
Task 7's on-silicon read.

**4 MB, so `0x10E00000` does not exist on it** and an XIP read there aliases to
`0x10200000`. This is the trap recorded in `CONTINUITY_2026-08-07b.md` §3: a
`sealread` run on this board reports "no payload at 0x10e00000 — CLEAN state",
which is a correct-*looking* answer from the wrong address. Do not cite it as
evidence about the normative region.

### `0xb3d19289d3ec3f0e` — Pico 2 W

Measured 2026-08-07 while it was in BOOTSEL alongside the SH2:

```
revision: A2   package: QFN60   secure boot: 0   debug enable: 1
Metadata Blocks: none            (blank — no image programmed)
```

**Not previously recorded anywhere**, and easy to mistake for the rehearsal Pico
— same form factor, same 4 MB, adjacent on the bench. Two things distinguish it
in ways that matter:

- **`secure boot: 0`.** Unlike both other boards, this one has no secure-boot
  fuse burned and no image. **It will accept and run anything, signed or not.**
  Convenient for scratch work; also means a signed-image test proves nothing here.
- **It is a *W*: it has WiFi, and its LED is not in the same place as a Pico 2's**
  (operator report, 2026-08-07). The electrically important form of that
  difference — on Pico W-class boards the onboard LED hangs off the CYW43
  wireless chip rather than an RP2350 GPIO, so `machine.LED` blink code written
  for a Pico 2 does not light it — is the standard arrangement for the part but
  **has not been verified on this specific board**. Verify before relying on a
  blink as a liveness signal; a dark LED here is not evidence that firmware
  failed to boot.

**Does NOT qualify for F-73.** `pico2-w` is `{"inherits": ["pico2"]}` — 4 MB,
same as the rehearsal Pico. Closing F-73 needs a 16 MB RP2350**B**: a Pico
Plus 2 (`__flash_size=16M`, which is why the fork's build target is
`pico-plus2`) or the SH2.

---

## Flashing

Always `~/bin/sh/sh2-flash` (→ `scripts/sh2-flash`), never `picotool` by hand:
the fork's build ships **unsigned** — `build-firmware` zeroes the pubkey and
signature — so the build output and the flashable artifact are different files,
and flashing the wrong one looks exactly like a signature rejection.

`--build-only` proves the whole build-and-sign path without touching a device.

**The signed artifact is not byte-reproducible.** ECDSA draws a fresh nonce per
signature, so two builds of identical input produce different signature values
and different sha256s. Do not use the signed `.uf2`'s hash as a build identity.
