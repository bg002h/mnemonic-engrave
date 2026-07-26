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
  current workstation. ✅ **RESOLVED 2026-07-26** — Nix installed; `nix develop`
  in the fork provides picotool **2.2.0-a4**, TinyGo **0.41.1**, the RPi openocd
  fork, and Go 1.26.
  **Use the official nixos.org Nix, NOT `pacman -S nix`** — the CachyOS
  `cachyos-extra-v3/nix` build segfaults on every invocation (libstdc++ static
  init → glibc `__newlocale` → `operator delete[]` into `libmimalloc.so.3`).
  Same version 2.35.1 from upstream works fine.
- **A rehearsal board.** A plain **Pico 2** (~$5) — *not* a Pico 2 W as the
  primary: on the W the LED sits behind the CYW43 chip, so the rehearsal blinky
  gives no visible pass signal. Package (RP2350A vs B) is irrelevant here; the
  OTP layout is identical, and our firmware cannot run meaningfully on any Pico
  anyway. Do the whole OTP dance there first — a mistake costs $5 instead of a
  SeedHammer II boot slot. See `scripts/pico2-bootkey-rehearsal.sh`.
- **USB-C cable.** No debug probe is required; everything is PICOBOOT over USB.

> **Reproducible build confirmed (2026-07-26).** A local
> `env VERSION=66d3121691ecf325b35e44285c1b2e7cf5250cce nix run .#build-firmware`
> produced an image byte-identical to the CI artifact for the same commit —
> sha256 `0e576b452f641168a42c28339845a568e5a3ac920d670fb7bef024ccf4c4f9dc`.
> The firmware you sign is verifiably the artifact CI built from a readable
> commit, not something only your machine can produce.
>
> Note `VERSION` must be pinned to the commit SHA to match CI; `build-firmware`
> otherwise defaults to `git describe`, and the version string is compiled in via
> `-ldflags`. In fish use `env VERSION=… nix run …` (fish has no `VAR=val cmd`).

---

## Step 1 — Verify device state (READ ONLY, do this first)

Enter BOOTSEL: hold the white firmware button (underside of the control board,
near the hammerhead) while connecting USB.

```sh
picotool info
picotool otp get -n CRIT1.SECURE_BOOT_ENABLE
picotool otp get -n BOOT_FLAGS1.KEY_VALID
for i in $(seq 0 15); do picotool otp get -n -e "BOOTKEY0_$i"; done   # slot 0
for s in 1 2 3; do for i in $(seq 0 15); do picotool otp get -n -e "BOOTKEY${s}_$i"; done; done

# Are the OTP pages locked? This is the ONLY read-only way to learn whether a
# second boot key can ever be added. Page 1 covers CRIT1/BOOT_FLAGS1; page 2
# covers BOOTKEY0-3.
for l in PAGE1_LOCK0 PAGE1_LOCK1 PAGE2_LOCK0 PAGE2_LOCK1; do
  picotool otp get -n -e "$l"
done
```

Read field values from the `field ... = <hex>` line, **not** from `VALUE 0x…` —
`VALUE` is the whole 24-bit row and includes unrelated bits (e.g. `KEY_INVALID`
sits in the same row as `KEY_VALID`).

**Expected on a stock sealed unit:** `SECURE_BOOT_ENABLE` = 1; `KEY_VALID` = `0x1`
(slot 0 only); slot 0 holding `c8314536d6af61ac2e62e5991e3e4711629c54696ba8c4af08965a1d319a473b`
(each row is two bytes, **low byte first**, so byte-swap each row when
reassembling); slots 1–3 all zero; **all four page-lock rows zero**.

**STOP and reassess if:** more than one slot is valid, slots 1–3 are non-zero,
slot 0's hash does not match the constant at `platform_sh2.go:70`, or **any
page-lock row is non-zero** — a locked page means no further boot key can ever be
added to this device and the whole procedure is impossible.

> Note the page-lock check is *necessary but not sufficient*: picotool discovers
> most lock conditions only at write time (its own source carries a
> `// todo pre-check page lock`). The first genuine proof that a sealed device
> accepts a spare-slot write is the write itself, in step 2 — which is why the
> Pico rehearsal must be completed first.

## Step 2 — Generate your key and burn its hash (IRREVERSIBLE)

```sh
openssl ecparam -name secp256k1 -genkey -noout -out my-key.pem
# Or encrypted at rest -- this key gates firmware for a device holding your
# backups, and it must survive for the life of the machine:
#   openssl ec -in my-key.pem -aes256 -out my-key.enc.pem
```

Back `my-key.pem` up now — losing it means you can never sign another firmware
update for this device, and you'd be left with only SeedHammer's official
releases (which is why step 7 exists as a *don't*).

The value burned into OTP is SHA-256 of the **uncompressed 64-byte X‖Y** public
key — not the 33-byte compressed form. **Never hand-compute or hand-verify it.**
Compute it two independent ways and require them to agree:

```sh
# (a) independently, with openssl
openssl ec -in my-key.pem -pubout -conv_form uncompressed -outform DER \
  | tail -c 64 | sha256sum

# (b) what picotool will actually burn (seal needs a REAL RP2350 image --
#     there is no such thing as a 'placeholder.elf')
picotool seal --sign --quiet seedhammerii-<version>.uf2 /tmp/discard.uf2 \
  my-key.pem my-otp-raw.json
jq -r '.bootkey0[]' my-otp-raw.json | awk '{printf "%02x",$1}'; echo
```

Then reduce the json to *only* your slot — scripted, not hand-edited, and
asserted before it is ever loaded:

```sh
jq '{bootkey1: .bootkey0}' my-otp-raw.json > my-otp.json   # keep ONLY the key
jq -e 'has("crit1") or has("boot_flags1")' my-otp.json && echo "REFUSE: would seal early"
picotool otp load my-otp.json
```

Dropping `crit1`/`boot_flags1` is not cosmetic: loading the unedited file would
set the key hash **and** `KEY_VALID` **and** `SECURE_BOOT_ENABLE` in a single
shot, sealing the device before a single row had been verified.

**Then verify all 16 rows mechanically, before the valid bit.** Do not eyeball
byte-swapped hex. `scripts/pico2-bootkey-rehearsal.sh` contains `read_slot()` /
`verify_slot_or_die()`, which read all 16 rows, byte-swap, reassemble, and
compare against the openssl-derived hash — use that same code here. If it
mismatches, **stop**: the slot is not yet valid, so the device still boots
normally, and you can move to slot 2. You have three slots; treat each as one
attempt.

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

**Use the script — it proves the result rather than assuming it:**

```sh
cd /scratch/code/shibboleth/seedhammer
nix develop --command ../mnemonic-engrave/scripts/sign-firmware.sh \
    seedhammerii-<version>.uf2 my-key.pem
```

`scripts/sign-firmware.sh` runs the sequence above and then does two things the
raw commands cannot:

1. **Asserts the digest is unchanged** after embedding the signature. That is
   the ordering assumption made testable — if the digest moved, the signature
   would be inside the hashed region and *no* signature could ever verify.
2. **Verifies the signature against the digest with openssl**, offline. Combined
   with `picotool info -a` reporting `signature: verified`, you know the image is
   correctly signed *before* flashing, instead of learning it from a device that
   refuses to boot.

✅ **Validated 2026-07-26** end-to-end on both branches — a freshly built blinky
(no SIGNATURE section → seal path) and the real 2.4 MB firmware image (section
already present). Both reached `signature: verified`.

<details>
<summary>The underlying sequence, for reference</summary>

`picosign sign` writes pubkey+signature as one 128-byte region
(`cmd/picosign/main.go:119-165`), and the signed hash covers the block including
the public key but excluding the signature — hence: embed pubkey with a dummy
signature → hash → sign the hash → embed the real signature → expect exactly TWO
metadata blocks (three means the image was sealed twice; rebuild).

Note the hex helper: `nix develop` *prepends* to PATH rather than purging it, so
`xxd` in ad-hoc commands silently resolves to the host `/usr/bin/xxd` and is not
provided by the devshell. The script uses `od` from nix coreutils instead, so it
does not depend on host tooling.
</details>

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

1. ⬜ **Rehearse the entire flow on a plain Pico 2 first.** Non-negotiable.
   Boards ordered 2026-07-26 (one Pico 2 + one Pico 2 W as spare); run
   `scripts/pico2-bootkey-rehearsal.sh` phases 0→6 to completion. Remember the
   board is *consumed* — a full run burns 2 of its 4 slots and seals it.
   **Phase 3 (negative control) is not optional:** it proves the sealed board
   *rejects* an image signed with a not-yet-burned key. Only then does the same
   image booting in phase 5 — after the key burn, the single variable — prove
   anything. Skipping it means a board that was never really sealed would sail
   through with a green result.
2. ✅ **RESOLVED 2026-07-26 — picotool field names verified** against the
   installed **picotool 2.2.0-a4** (`picotool otp list`):
   `CRIT1.SECURE_BOOT_ENABLE` = bit 0; `BOOT_FLAGS1.KEY_VALID` = bits 0-3;
   `BOOT_FLAGS1.KEY_INVALID` = bits 8-11; `BOOTKEY0_0` at row `0x0080`
   (16 rows per key). `KEY_VALID` being 4 bits independently corroborates
   4 boot-key slots, matching `driver/otp/otp.go:15` `NumBootKeySlots = 4` and
   the `BOOTKEY0_0 = 0x080` constant at `otp.go:20`. The names used throughout
   this runbook are correct for this picotool version.
3. ⬜ Decide whether to build from the fork `main` merge commit `66d3121`
   (upstream v1.4.3 + our features, CI-green and reproducible) or to tag a fork
   release first.
4. ⬜ **Still unverified on real hardware:** that a sealed device accepts OTP
   writes to a spare slot. Our code read says yes (`driver/otp/` never writes
   page locks, `DEBUG_DISABLE`, or `KEY_INVALID`), and the community guide agrees
   — but this is the one assumption the whole procedure rests on and it has never
   been executed.
   **Correction (2026-07-26):** an earlier draft claimed rehearsal phase 2 was
   this test. It is not — phase 2 only *reads* (page locks + slot state), and a
   read cannot prove writability. The first genuine sealed-board write is
   rehearsal **phase 4**, which is itself irreversible. Page locks are a
   necessary-not-sufficient precheck; run them on the SH2 in step 1, and accept
   that the definitive answer only arrives at the moment of the real write.
5. ✅ **RESOLVED 2026-07-26 — the official-firmware fallback is real.** Verified
   offline against `seedhammerii-v1.4.3.uf2`: `picotool info -a` reports
   `signature: verified`, and SHA-256 of its embedded uncompressed pubkey is
   `c8314536…319a473b`, matching `signKeyHash` (`platform_sh2.go:70`); its
   X-coordinate also matches the flake's `copy-signature` constant. Official
   releases ship signed by the slot-0 key, so leaving slot 0 valid genuinely
   preserves a bootable recovery path. (Note our *own* build ships with a zeroed
   signature by design — `picosign sign -clear` — so it is not evidence either
   way about upstream releases.)
