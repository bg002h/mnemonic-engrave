# RUNBOOK — running fork firmware on a locked SeedHammer II

**Status:** drafted 2026-07-26, **not yet executed on hardware.**
**Applies to:** a retail SeedHammer II with secure boot sealed to SeedHammer's
signing key (confirmed 2026-07-26 — the device's home screen does *not* show the
`(UNLOCKED)` suffix).

> ### Run every command block here inside `bash`, not fish
>
> Your login shell is fish, where `for … do … done` and `if … then … fi` are
> **hard syntax errors** — a pasted block executes *nothing*, and a partially
> pasted one can run a destructive command with its guard silently missing.
> Start each session with:
>
> ```sh
> cd /scratch/code/shibboleth/seedhammer && nix develop   # this drops you into bash
> ```
>
> The `--sh2-*` invocations below are single commands and are fish-safe, but the
> habit should be uniform.

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
- **udev rule, numbered BELOW 73.** picotool needs USB access:
  `/etc/udev/rules.d/60-picotool.rules` containing
  `SUBSYSTEM=="usb", ATTRS{idVendor}=="2e8a", MODE="0660", TAG+="uaccess"`.
  The number is load-bearing: systemd's `73-seat-late.rules` is what converts
  `TAG+="uaccess"` into an ACL for your session, and udev processes files in
  sort order — a `99-` file tags the device too late and the ACL never appears.
  The symptom is picotool saying the device *"appears to be in BOOTSEL mode, but
  picotool was unable to connect"*. Verify with `getfacl` on the device node:
  you should see a `user:<you>:rw-` entry.

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

> **Honest status of these gates.** `--sh2-precheck` has been run against your
> actual SeedHammer and passed. `--sh2-verify-slot` and `--sh2-verify-valid`
> have **not** — they refuse any board without SeedHammer's key in slot 0, so the
> Pico could not exercise them, and they need a burned spare slot that does not
> exist yet. Their underlying reader (`read_slot` / `verify_slot_or_die`) *is*
> hardware-proven, via rehearsal phases 1c and 4c, and the wrappers are covered
> by 39 offline checks — but their success path will run for the first time on
> your machine. Every failure mode they have is a `die` costing zero OTP, so the
> risk is being stalled at a gate, not being misled past one.
>
> **Free hardening before step 2:** run
> `--sh2-verify-slot 1 --key ~/.sh2/sh2-boot-key.pem` against the still-pristine
> device. It must die with `SLOT 1 READBACK MISMATCH` showing all-zeros against
> your hash. That exercises the entire wrapper chain on the real machine, with
> only the final comparison differing from the post-burn success path. (Ignore
> its "permanently unusable" wording in this context — nothing has been burned.)

> **Run this the day you get the machine — not the day you burn OTP.** It writes
> nothing, and it is the only way to learn, short of an irreversible write,
> whether your retail unit carries OTP page locks or other factory provisioning
> that would make this whole procedure impossible. It is also the first time the
> OTP parsers meet real picotool output rather than stubs derived from the same
> source reading they were written from — they fail closed, so a mismatch is a
> loud error, not a wrong answer. See `FIRMWARE-QUICKSTART.txt` section 0a.
> Re-run it again immediately before step 2, so the state you act on is the
> state you just read.

Enter BOOTSEL: hold the button on the control board while connecting USB. (The
manual calls it the firmware button; on the unit used here, 2026-08-03, it is
physically labelled **reset**. Only the moment of power-up matters — there is no
need to keep holding it.)

**Confirm you are actually in BOOTSEL rather than trusting the label:** `lsusb`
must show `2e8a:000f ... RP2350 Boot`, and `picotool info` must report
`target chip: RP2350`. That device ID *is* the bootrom's BOOTSEL identity —
normally-booted firmware cannot present it, and OTP is unreadable outside
BOOTSEL. Both succeeding is proof.

**Unplug every other RP2350 board first** — especially the consumed rehearsal
Pico. A wrong-but-plausible device answers these reads with exactly the values
you expect to see.

```sh
cd /scratch/code/shibboleth/seedhammer
nix develop --command \
  ../mnemonic-engrave/scripts/pico2-bootkey-rehearsal.sh --sh2-precheck
```

`--sh2-precheck` is **read-only** — it contains no `otp load`, no `otp set`, no
`picotool load`. It asserts, and STOPs on any failure:

- slot 0 holds `c8314536…319a473b` (`signKeyHash`, `platform_sh2.go:70`) — i.e.
  this really is a SeedHammer II, and the official-firmware recovery path is intact
- it **pins this device's CHIPID**, which step 2 and step 3 re-check
- `SECURE_BOOT_ENABLE` = 1, `KEY_VALID` = `0x1` (slot 0 only), `KEY_INVALID` = 0
- spare slots 1–3 empty across **all 16 rows each**
- OTP page **permissions** allow Secure writes. These rows are read **without
  `-e`** at full 24 bits (they are `ecc=false`, and an ECC read would mask away
  the third lock copy). **They are not expected to be zero:** a retail
  SeedHammer II reads `PAGE1_LOCK1 = PAGE2_LOCK1 = 0x040404` — a byte replicated
  3-way, decoding to `LOCK_S=0` ("page is fully accessible by Secure software"),
  `LOCK_BL=0` ("bootloader permits user reads and writes") and `LOCK_NS=1`
  (Non-secure restricted to reads). picotool writes as Secure software, so that
  state is fully writable. Only `LOCK_S`/`LOCK_BL` non-zero, or `KEY_R`/`KEY_W`
  set in the `LOCK0` rows, disqualifies the device. *(An earlier draft demanded
  all-zero and would have declared a working retail unit permanently unusable —
  caught on real hardware 2026-08-03.)*
- raw redundant-row readback of `CRIT1` (×8) and `BOOT_FLAGS1` (×3), so a partial
  write is visible

> **Never send the NFC `lock-boot` debug command to this unit.** It is the
> factory provisioning path (`gui/gui.go:1626` → `LockBoot()`). It cannot waste
> a boot-key slot — `AddBootKey` returns the existing slot on an exact match —
> but it burns white-label OTP rows for no benefit.

It is the same `read_slot` / `verify_slot_or_die` / `check_page_locks` code the
Pico rehearsal exercises — deliberately, so the tool used here is the tool the
rehearsal proved. **Do not hand-verify byte-swapped hex.**

> The page-lock check is *necessary but not sufficient*: picotool discovers most
> lock conditions only at write time (its own source carries a
> `// todo pre-check page lock`). The first genuine proof that a sealed device
> accepts a spare-slot write is the write itself, in step 2 — which is why the
> Pico rehearsal must be completed first.

## Step 2 — Generate your key and burn its hash (IRREVERSIBLE)

**Name the real key distinctly from any rehearsal key.** The rehearsal generates
its own `my-key.pem` inside `rehearsal-work/`, a directory this project documents
as disposable — burning that key's hash here would strand the slot the moment it
is deleted. The SH2 tooling refuses any key matching a rehearsal key, but don't
rely on that; use a different name and back it up.

```sh
mkdir -p ~/.sh2 && chmod 700 ~/.sh2
openssl ecparam -name secp256k1 -genkey -noout -out ~/.sh2/sh2-boot-key.pem
chmod 600 ~/.sh2/sh2-boot-key.pem
# Encrypted at rest is better -- this key gates firmware for a device holding
# your backups and must survive for the life of the machine:
#   openssl ec -in ~/.sh2/sh2-boot-key.pem -aes256 -out ~/.sh2/sh2-boot-key.enc.pem
```

> **Not inside any git repo.** The seedhammer fork's `.gitignore` covers only
> `seedhammerii-*.uf2` and `_artifacts`, and its remote is **public**. A key
> generated there is one `git add -A` from being published — and a published
> boot key that is permanently valid on your engraver cannot be un-burned.
> Record its fingerprint (`openssl ec -in ~/.sh2/sh2-boot-key.pem -pubout
> -conv_form uncompressed -outform DER | tail -c 64 | sha256sum`) so you can
> always tell it apart from a rehearsal key.

Back it up **now**, off this machine. Losing it means you can never sign another
firmware update for this device and are left with only SeedHammer's official
releases (which is why step 7 exists as a *don't*).

Build the firmware **now**, before the irreversible steps — `picotool seal` needs
a real RP2350 image, and you do not want to discover a missing file mid-procedure:

```sh
env VERSION=$(git rev-parse HEAD) nix run .#build-firmware
```

Generate and validate the OTP json (this touches **no device**):

```sh
nix develop --command ../mnemonic-engrave/scripts/pico2-bootkey-rehearsal.sh \
  --make-otp-json --key ~/.sh2/sh2-boot-key.pem --slot 1 --out ~/.sh2/my-otp.json
```

That asserts, before the file can ever be loaded: exactly one top-level key, the
correct slot, 32 entries, no `crit1`/`boot_flags1`, and byte-equality with an
independently openssl-derived hash. Dropping `crit1`/`boot_flags1` is not
cosmetic — loading the raw `picotool seal` output would set the key hash **and**
`KEY_VALID` **and** `SECURE_BOOT_ENABLE` in one shot, sealing the device before a
single row had been verified.

Now the first irreversible write:

```sh
picotool otp load ~/.sh2/my-otp.json
```

> **This is the one command with an unrecoverable interruption window.** It
> programs 16 rows; an interruption leaves rows `0..k` burned and the rest
> blank, and **re-running it cannot repair that** — the bootrom refuses any
> write to an already-programmed ECC row, identical value or not. That slot is
> then spent. `--sh2-verify-slot` below detects the state correctly, but the
> only remedy is the next free slot.
>
> So before running it: short cable straight into the machine, no hub, laptop on
> AC with sleep disabled, and don't touch the bench until step 3 completes.
>
> **Prefer a non-PD 5 V source for the burn** — a USB-A port with an A-to-C
> cable. The SeedHammer puts an AP33772S USB-PD sink between the connector and
> the system (`platform_sh2.go:211`); PD renegotiation or a hard reset can drop
> VBUS with no firmware involvement while the chip sits in BOOTSEL. A USB-A
> source has no PD state machine at all, and reproduces the electrical
> conditions under which the Pico rehearsal succeeded. This device already
> logged one `device descriptor read/64, error -71` during enumeration.
> Note also that `otp load` prints no "verified" confirmation of its own — the
> absence of output is not success, which is why step 3 is mandatory.

## Step 3 — Verify, then mark the slot valid (IRREVERSIBLE)

**The verification is the gate. It is mechanical, and it is not optional.**

```sh
cd /scratch/code/shibboleth/seedhammer
nix develop --command ../mnemonic-engrave/scripts/pico2-bootkey-rehearsal.sh \
  --sh2-verify-slot 1 --key ~/.sh2/sh2-boot-key.pem
```

Read-only. It re-checks the pinned CHIPID (so a different board cannot answer for
your SeedHammer), re-confirms slot 0 still holds SeedHammer's key, then reads all
16 rows of slot 1, byte-swaps, reassembles, and requires an exact match against
the openssl-derived hash. It is the same code the Pico rehearsal proves in phases
1c and 4c.

**If it mismatches, STOP.** The slot is not yet valid, so the device still boots
normally — move to slot 2 (`--slot 2`, then `0x4` below). You have three slots;
treat each as one attempt.

Only once it passes:

```sh
picotool otp set -s BOOT_FLAGS1.KEY_VALID 0x2   # slot 1 (slot 2 = 0x4, slot 3 = 0x8)
```

Then **close the loop with the read-only post-write check** — not a bare
`picotool otp get`:

```sh
cd /scratch/code/shibboleth/seedhammer
nix develop --command ../mnemonic-engrave/scripts/pico2-bootkey-rehearsal.sh \
  --sh2-verify-valid 1 --key ~/.sh2/sh2-boot-key.pem
```

`otp set` writes all three redundant `BOOT_FLAGS1` copies in **one** PICOBOOT
command. An interruption inside it can leave 2 of 3 programmed — at which point
picotool's majority vote prints **the value you expected** alongside a
`(WARNING - REDUNDANT ROWS AREN'T EQUAL)`, so a bare `otp get` passes while the
redundancy protecting "trust your key" is permanently degraded.
`--sh2-verify-valid` fails closed on that warning, requires `KEY_VALID` to be
exactly `0x1 | slot-bit` (slot 1 → `3`, slot 2 → `5`, slot 3 → `9`), requires
`KEY_INVALID == 0`, and requires all three copies to agree.

**If `KEY_VALID` reads low, the write was interrupted.** Re-run the identical
`otp set -s` command — it only ever *sets* bits, so repeating it is safe and
costs nothing. Do **not** burn another slot, and do **not** start re-signing:
step 3's readback already proved your hash is correct.

Use `otp set -s` (OR-in), never `otp load`, for this field — `otp load` attempts
to clear bits and will fail.

## Step 4 — Build (retryable)

```sh
cd /scratch/code/shibboleth/seedhammer
env VERSION=$(git rev-parse HEAD) nix run .#build-firmware
# → seedhammerii-$(git rev-parse HEAD).uf2
# VERSION must be pinned: without it build-firmware falls back to `git describe`
# and produces a filename steps 5 and 6 will not find.
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
    $PWD/seedhammerii-$(git rev-parse HEAD).uf2 ~/.sh2/sh2-boot-key.pem
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
FW=$PWD/seedhammerii-$(git rev-parse HEAD).uf2
picotool load --verify ${FW%.uf2}.signed.uf2      # the SIGNED file, not the input
picotool reboot
```

> ### Judge the boot on MACHINE power, never on the laptop
>
> `Init()` runs `monitorPowerSupply` **before** it configures the LCD
> (`cmd/controller/platform_sh2.go:224-251`). That code demands a 20–28 V USB-PD
> contract and, on failure, calls `rebootIntoBOOTSEL()`
> (`platform_sh2.go:428-441`) — deliberately, for exactly the "plugged into a
> flashing computer" case.
>
> So on a laptop port that cannot source 20 V, **correctly signed firmware that
> the bootrom ACCEPTED still gives you a dark screen and a device that
> re-enumerates as `RP2350 Boot`** — pixel-identical to a signature rejection.
>
> After `picotool reboot`: **unplug from the computer and power the machine from
> its normal supply before judging anything.** A dark screen while tethered means
> nothing. This also means the Pico rehearsal's "still enumerable = rejected"
> reasoning does **not** transfer to the SeedHammer.

**Expected result (on machine power):** normal startup screen, with `(UNLOCKED)`
appended to the version line.

> ### If it does NOT boot — do NOT burn another slot
>
> Step 3's readback already **proved** the burned hash is correct, so a boot
> failure here is a **signing or image** problem, not a slot problem. Those are
> retryable at zero OTP cost:
>
> 0. **Did you flash the `.signed.uf2`?** `sign-firmware.sh` never modifies its
>    input — it writes a new file. Flashing the build output directly flashes an
>    image whose signature `build-firmware` deliberately zeroed, which cannot
>    boot on a sealed device. And **did you judge it on machine power?** See the
>    box above.
> 1. Re-run step 5 and re-flash. **Note what this is NOT:** the RP2350 bootrom
>    performs no ECDSA canonicality check — verified against
>    `raspberrypi/pico-bootrom-rp2350` (`arm8_sig.c` into the pinned `sweet-b`
>    commit, whose only scalar test is `0 < k < n`). Roughly half of all openssl
>    signatures are high-`s` (measured: 53.2% of 500) and they verify fine. A
>    boot failure here is **never** a high-`s` artifact — look at real causes:
>    wrong key, wrong image, a corrupted UF2, or the metadata-block count.
> 2. Re-flash and reboot.
> 3. Official SeedHammer firmware remains bootable throughout — flash a release
>    UF2 to get a working machine back while you debug.
>
> The Recovery section's "move to the next free slot" advice applies **only** when
> step 3's verification actually failed. Burning a second slot for a boot failure
> that step 3 passed will consume your spares without fixing anything.

**That suffix is expected and is not a failure.** `isSecureBootEnabled()`
(`platform_sh2.go:712-741`) returns true only when secure boot is on **and**
SeedHammer's key is the *sole* valid key (`nvalid == 1`). With two valid keys the
condition is false, so `FeatureSecureBoot` is cleared and `gui/gui.go:2717-2719`
appends the suffix. The device is still enforcing signature checks — it now
simply trusts two keys instead of one. You permanently lose the on-screen
attestation as a quick integrity indicator; budget for that.

## Step 7 — DO NOT revoke SeedHammer's key

The community guide offers a final `otp set` on `BOOT_FLAGS1.KEY_INVALID` to
revoke slot 0. **Don't** — and the exact command is deliberately not written out
here, because it is the single most destructive line in this document and there
is no reason for it to sit in your paste buffer. Leaving slot 0 valid keeps
official SeedHammer releases bootable, which is your only recovery path if a fork
build ever fails to start.
Revoking is permanent and removes that path.

---

## Recovery

- **Fork firmware won't boot / bad signature** → re-enter BOOTSEL and flash an
  official `seedhammerii-vX.Y.Z.uf2`. Works as long as step 7 was respected.
- **Wrong hash burned into a slot** — *i.e. `--sh2-verify-slot` actually FAILED* →
  that slot is dead forever. Move to the next free slot (you start with three).
  The device is unaffected until a valid bit is set. **This does not apply to a
  boot failure after a passing verification** — see step 6, which is a re-signable
  problem and must not cost you a slot.
- **Lost `sh2-boot-key.pem`** → you can no longer sign updates for your slot.
  Official firmware still boots. Burn a fresh key into the next slot.

## Open items to resolve before executing

1. ✅ **DONE 2026-08-03 — the Pico 2 rehearsal was executed and passed, phases
   0→6.** See `design/REHEARSAL_RESULT_2026-08-03.md`. Board CHIPID
   `bf2ff20ad60f66d3`, consumed as designed. The reject→accept A/B held on real
   silicon, the real 2.4 MB firmware was accepted, and the factory key still
   booted afterwards. *(Original text follows.)* Rehearse the entire flow on a
   plain Pico 2 first. Non-negotiable.
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
4. ✅ **RESOLVED 2026-08-03 on real silicon.** A sealed board DOES accept OTP
   writes to a spare slot: rehearsal phase 4 burned slot 1 on a board sealed in
   phase 1, `KEY_VALID` went 0x1 → 0x3, and the 16-row readback matched. This was
   the load-bearing assumption of the entire procedure and it is now demonstrated
   rather than inferred — on a Pico, not on the SeedHammer, but the SeedHammer's
   page permissions were confirmed identical by `--sh2-precheck`.
   *(Original text follows.)* Still unverified on real hardware: that a sealed
   device accepts OTP writes to a spare slot. Our code read says yes (`driver/otp/` never writes
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
