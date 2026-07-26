# RUNBOOK — running fork firmware on a locked SeedHammer II

**Status:** drafted 2026-07-26, **not yet executed on hardware.**
**Applies to:** a retail SeedHammer II with secure boot sealed to SeedHammer's
signing key (confirmed 2026-07-26 — the device's home screen does *not* show the
`(UNLOCKED)` suffix).

> **Read this first.** Steps 1–3 write to **one-time-programmable** memory. They
> cannot be undone, reverted, or cleared — not by SeedHammer, not by Raspberry
> Pi, not by anyone. Steps 4–6 (build, sign, flash) touch only flash and are
> freely retryable. Keep that boundary in mind: **the only irreversible part of
> this procedure is burning your key hash and setting its valid bit.**

---

## 0. Why this is necessary

Retail units are sealed at the factory by an NFC-delivered `lock-boot` debug
command (`gui/gui.go:1624-1631`), which runs `Platform.LockBoot()`
(`cmd/controller/platform_sh2.go:510-518`):

1. `writeOTPValues()` — writes white-label strings and `AddBootKey(signKeyHash)`
   where `signKeyHash = c8314536…319a473b` (`platform_sh2.go:70`);
2. `otp.EnableSecureBoot()` — sets `CRIT1.SECURE_BOOT_ENABLE`;
3. reboots.

After that the RP2350 bootrom will only start images signed by a key whose
SHA-256 appears in a *valid* boot-key slot. Our unsigned build (the flake's
`build-firmware` ends with `picosign sign -clear`, zeroing pubkey+signature)
will not boot.

**The opening this relies on:** RP2350 has **four** boot-key slots
(`driver/otp/otp.go:15`, `NumBootKeySlots = 4`); SeedHammer occupies one, so
**three remain writable**. Critically, `driver/otp/` **never** writes OTP page
locks, `DEBUG_DISABLE`, `KEY_INVALID`, or any BOOTSEL-disable flag — the only
call sites are `EnableSecureBoot`, `AddBootKey`, and the white-label strings.
So PICOBOOT/OTP access over plain USB stays open on a sealed unit. *(Verified by
reading the fork tree, 2026-07-26. Re-verify on your actual device in step 1
before writing anything.)*

## Source caveat

The command sequence below is adapted from a **third-party community guide** —
[Gangleri42/seedhammer `docs/howto-bootkey-and-signing.md`](https://github.com/Gangleri42/seedhammer/blob/main/docs/howto-bootkey-and-signing.md).
It is **not** official SeedHammer documentation; seedhammer.com's firmware-upgrade
page does not mention secure boot at all. Our own R0 review flagged this class of
claim as "unknowable for retail units without physical access"
(`design/agent-reports/seedhammer-engrave-spec-R0-review.md:58`). You now have
the access — **step 1 is the empirical check that replaces that assumption.**
Do not skip it.

---

## Prerequisites

- **Toolchain.** None of `nix`, `go`, `tinygo`, `picotool` are installed on the
  current workstation (checked 2026-07-26). Nix is the gate — the flake pins
  TinyGo 0.41.1, picotool, the RPi openocd fork, and Go. Install Nix with flakes,
  then `nix develop` in the fork gives you everything.
- **A rehearsal board.** A Pico 2 or Pico Plus 2 (~$5). The firmware targets
  `-target pico-plus2` — *the same silicon*. Do the whole OTP dance there first.
  A mistake costs $5 instead of a SeedHammer II boot slot.
- **USB-C cable.** No debug probe is required; everything is PICOBOOT over USB.

---

## Step 1 — Verify device state (READ ONLY, do this first)

Enter BOOTSEL: hold the white firmware button (underside of the control board,
near the hammerhead) while connecting USB.

```sh
picotool info
picotool otp get CRIT1.SECURE_BOOT_ENABLE
picotool otp get BOOT_FLAGS1.KEY_VALID
for k in BOOTKEY0_0 BOOTKEY1_0 BOOTKEY2_0 BOOTKEY3_0; do
  echo "--- $k ---"; picotool otp get -e "$k"
done
```

**Expected on a stock sealed unit:** `SECURE_BOOT_ENABLE` set; `KEY_VALID` = `0x1`
(slot 0 only); slot 0 holding `c8314536d6af61ac2e62e5991e3e4711629c54696ba8c4af08965a1d319a473b`;
slots 1–3 all zero.

**STOP and reassess if:** more than one slot is valid, slots 1–3 are non-zero, any
OTP write returns "not permitted", or slot 0's hash does not match the constant at
`platform_sh2.go:70`. Any of those invalidates the assumptions above.

## Step 2 — Generate your key and burn its hash (IRREVERSIBLE)

```sh
openssl ecparam -name secp256k1 -genkey -noout -out my-key.pem
openssl ec -in my-key.pem -pubout -out my-pubkey.pem
```

Back `my-key.pem` up now — losing it means you can never sign another firmware
update for this device, and you'd be left with only SeedHammer's official
releases (which is why step 6 exists as a *don't*).

Have picotool compute the hash — **never hand-compute it.** It must be SHA-256 of
the **uncompressed 64-byte X‖Y** public key, not the 33-byte compressed form.

```sh
picotool seal --sign placeholder.elf discard.elf my-key.pem my-otp.json
rm discard.elf
```

Edit `my-otp.json` down to *only* your slot, renaming `bootkey0` → `bootkey1`.
Delete the `boot_flags1` and `crit1` entries — the valid bit comes later, after
verification.

```sh
picotool otp load my-otp.json
picotool otp get -e BOOTKEY1_0 BOOTKEY1_1 ... BOOTKEY1_15   # verify all 16 rows
```

**Verify every row before proceeding.** Each row holds two bytes, low byte first.
If any row disagrees, **stop** — the slot is not yet marked valid, so the device
still boots official firmware normally, and you can move to slot 2 instead. You
have three slots; treat each as one attempt.

## Step 3 — Mark the slot valid (IRREVERSIBLE)

```sh
picotool otp set -s BOOT_FLAGS1.KEY_VALID 0x2   # slot 1 (slot 2 = 0x4, slot 3 = 0x8)
picotool otp get BOOT_FLAGS1.KEY_VALID          # expect 0x3 — slot 0 AND slot 1
```

Use `otp set -s` (OR-in), never `otp load`, for this field — `otp load` attempts
to clear bits and will fail.

## Step 4 — Build (retryable)

```sh
cd /scratch/code/shibboleth/seedhammer
nix run .#build-firmware        # → seedhammerii-<version>.uf2
```

## Step 5 — Sign with your key (retryable)

**Ordering matters.** The signed hash covers the block including the public key
but excluding the signature, so the real pubkey must be embedded *before* the
hash is computed. `picosign sign` writes pubkey+signature as one 128-byte region
(`cmd/picosign/main.go:119-165`), so the sequence is: embed pubkey with a dummy
signature → hash → sign the hash → embed the real signature.

```sh
FW=seedhammerii-<version>.uf2
PUB=$(openssl ec -in my-key.pem -pubout -conv_form compressed -outform DER \
      | tail -c 33 | xxd -p -c 33)

# 5a. Embed the real pubkey with a placeholder signature.
go run seedhammer.com/cmd/picosign sign -pubkey "$PUB" \
   -sig "$(printf '0%.0s' {1..128})" "$FW"

# 5b. Extract the digest (raw bytes on stdout) and sign it.
go run seedhammer.com/cmd/picosign hash "$FW" > digest.bin
openssl pkeyutl -sign -inkey my-key.pem -in digest.bin -out sig.der

# 5c. Embed the real signature.
go run seedhammer.com/cmd/picosign sign -pubkey "$PUB" \
   -sig "$(xxd -p -c 256 sig.der)" -sigfmt der "$FW"

picotool info -a "$FW"    # expect exactly TWO metadata blocks
```

Three metadata blocks means the image got sealed twice — rebuild from step 4.

Nothing here touches OTP. Get it wrong as many times as you need.

## Step 6 — Flash

```sh
picotool load --verify "$FW"
picotool reboot
```

**Expected result:** normal startup screen, with `(UNLOCKED)` appended to the
version line.

**That suffix is expected and is not a failure.** `isSecureBootEnabled()`
(`platform_sh2.go:712-741`) returns true only when secure boot is on **and**
SeedHammer's key is the *sole* valid key (`nvalid == 1`). With two valid keys the
condition is false, so `FeatureSecureBoot` is cleared and `gui/gui.go:2717-2719`
appends the suffix. The device is still enforcing signature checks — it now
simply trusts two keys instead of one. You permanently lose the on-screen
attestation as a quick integrity indicator; budget for that.

## Step 7 — DO NOT revoke SeedHammer's key

The community guide offers a final `picotool otp set -s BOOT_FLAGS1.KEY_INVALID 0x1`
to disable slot 0. **Don't.** Leaving it valid keeps official SeedHammer releases
bootable, which is your only recovery path if a fork build ever fails to start.
Revoking is permanent and removes that path.

---

## Recovery

- **Fork firmware won't boot / bad signature** → re-enter BOOTSEL and flash an
  official `seedhammerii-vX.Y.Z.uf2`. Works as long as step 7 was respected.
- **Wrong hash burned into a slot** → that slot is dead forever. Move to the next
  free slot (you start with three). The device is unaffected until a valid bit is
  set.
- **Lost `my-key.pem`** → you can no longer sign updates for your slot. Official
  firmware still boots. Burn a fresh key into the next slot.

## Open items to resolve before executing

1. **Rehearse the entire flow on a Pico 2 / Pico Plus 2 first.** Non-negotiable.
2. **Re-verify** the community guide's `picotool` invocations against the RP2350
   datasheet and your installed picotool version — flag/field names have shifted
   across picotool releases.
3. Decide whether to build from the fork `main` merge commit `66d3121`
   (upstream v1.4.3 + our features) or to tag a fork release first.
