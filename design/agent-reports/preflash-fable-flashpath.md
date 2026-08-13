# FINAL-GATE REVIEW — the flash operation itself (fable, 2026-08-11)

**Lens:** the act of putting firmware and payload onto the SeedHammer II and
getting back out. Not the code that will run.

**Verdict: GO**, with two procedural conditions (I1, I2 below). No Critical
findings: nothing in this path as written can permanently destroy the machine
or its key material. The dangerous class is OTP writes, and this path contains
none — verified by reading every line of both scripts and the flake's
build-firmware: no `picotool otp set/load/write` anywhere in the flash path.

---

## 0. The brief's premise was stale — and the correction is good news

The brief stated the device is NOT attached. **It is.** `lsusb` shows
`2e8a:000f Raspberry Pi RP2350 Boot` (bus 001 dev 020), in BOOTSEL, right now.
Because `2e8a:000f` is the *bootrom's* identity, not the machine's, I
identified it read-only: `picotool otp get CHIPID0..3` returned
`f55c / 45ab / 83b7 / 77c4` — exactly the recorded SH2 CHIPID
`f55c45ab83b777c4` (`~/.sh2/SEED_HAMMER_OTP_SLOT_USAGE.txt`). The consumed
Pico 2 rehearsal board is `bf2ff20ad60f66d3`; this is not it.

**The attached BOOTSEL device is the SeedHammer II itself.** Every read below
against "the device" is against the real target. Nothing in this review wrote
to it (reads only: `otp get`, `help`, `version`; the one `sh2-flash` run was
`--dry-run` and demonstrably wrote nothing).

## 1. What was machine-checked this session (do not re-derive)

| Claim | How checked | Result |
| --- | --- | --- |
| Slot 0 valid on silicon | `picotool otp get BOOT_FLAGS1.KEY_VALID/.KEY_INVALID` on the live device | `KEY_VALID=0x3`, `KEY_INVALID=0x0` — slots 0 AND 1 valid, nothing revoked |
| Recovery image real | `picotool info -a ~/.sh2/recovery/seedhammerii-v1.4.3.uf2` | `signature: verified`; embedded pubkey sha256 = `c8314536…` = documented slot-0 production key |
| Signing key is the burned key | sha256 of uncompressed X‖Y pubkey from `~/.sh2/sh2-boot-key.pem` vs `~/.sh2/sh2-boot-key.fingerprint` | both `846aa289f2f3…d64cabb4` — a default-key run WILL boot |
| Build ships unsigned | `seedhammer/flake.nix:114-116` | seals with a dummy PEM then `picosign sign -clear` zeroes pubkey+sig |
| picotool BIN default offset | `picotool help load` in the devshell (v2.2.0-a4) | **`default 0x10000000`** — see I2 |
| Sample region image | `od` + `wc -c` + `target/release/me sysw show` (me 0.5.1) | 65536 bytes exactly, `MNEMSYSW` at offset 0, digest `616f 88f5 bb98 2e84 eb3d 0b5a f3d3 8777` (matches F-141's recorded value) |
| Region read at boot? | grep of `syswOffer`/`SyswReader` call sites in fork HEAD (`afa7ac7`) | read **on demand from menu flows only**, never at boot — a poison payload cannot wedge boot |
| `--dry-run` writes nothing | executed it + audited every write path | all writes route through `run()`/`devshell()`, both gated on `DRY` |

## 2. Q1 — What, concretely, is unrecoverable? (the enumeration)

BOOTSEL is mask ROM plus a physical button. **No flash write, at any address,
with any content, can remove it.** The only ways to lose it are OTP
(`BOOT_FLAGS0` BOOTSEL-disable bits, CRIT flags), and this path never writes
OTP. So:

| Failure | BOOTSEL saves it? | Recovery |
| --- | --- | --- |
| Unsigned build output flashed (the classic) | **Yes** | re-enter BOOTSEL, flash the `.signed.uf2` |
| Signed with a valid-but-wrong key (I1) | **Yes** | BOOTSEL, re-sign with `~/.sh2/sh2-boot-key.pem` |
| Payload bin loaded without `-o` → firmware head overwritten (I2) | **Yes** | BOOTSEL, re-run sh2-flash |
| Torn write (cable yank mid-load) — firmware or payload | **Yes** | BOOTSEL, reload; `--verify` would have failed anyway; device digest check catches a torn region |
| Wrong offset up to and past `0x11000000` (wrap destroys firmware start, RP2350 §5.5.2) | **Yes** | BOOTSEL, reflash |
| Fork firmware boots but is broken/unusable | **Yes** | BOOTSEL, flash `~/.sh2/recovery/seedhammerii-v1.4.3.uf2` → boots via slot 0 |
| **Any `picotool otp` write** (KEY_INVALID on slots 0+1, garbage in slots 2/3 then revocation, BOOTSEL-disable flags) | **NO — this is the whole unrecoverable class** | none; and it is NOT in this path |
| Loss of `~/.sh2/sh2-boot-key.pem` (host side) | machine still boots official via slot 0 | slot 1 unusable forever; slots 2/3 spare; **back the key up** |
| Loss of the recovery image (host side) | n/a | guarded: `sign-firmware.sh` step 0b refuses to re-sign it by keyhash and never mutates input; re-downloadable regardless |
| Mistyped payload offset `0x10E00000` → sealed payload overwritten | boots fine | data loss only, re-packable from host source |

**Slot-0 claim verified rather than trusted** (row 1–2 of §1): the recovery
image exists on disk, its signature verifies offline, its key hashes to the
slot-0 hash, and the live device says slot 0 is valid and unrevoked. That
recovery path is real end-to-end.

## 3. Q2 — Can the payload write hit the wrong address or length?

The map (spec §4, measured constraints; region constants confirmed in
`seedhammer/sysw/wire.go:27`):

```
0x10000000  firmware (ends 0x10135300; picotool touches through 0x10136000)
0x10136000 … 0x10CFFFFF   ~12 MiB unprogrammed
0x10D00000 – 0x10D10000   SYSTEMWIDE region (this write, 16 × 4 KiB sectors, aligned)
0x10D10000 … 0x10DFFFFF   1 MiB clearance — deliberate
0x10E00000 – 0x10E10000   Sealed Payload region
0x10FFF000               top sector, must stay clear
0x11000000               end; past it a write WRAPS to 0x10000000 (datasheet 5.5.2)
```

- Correct invocation (64 KiB at `0x10D00000`) erases and writes exactly its own
  16 sectors; nothest neighbor in either direction is ~1 MiB away. No
  collateral is reachable by an off-by-one; it takes a wrong *megabyte*.
- **OTP is not flash.** No `picotool load`, to any address, can reach OTP or
  the key material. The wrong-address worst case is firmware (recoverable) or
  the sealed payload region (host-recoverable data).
- **Verification of landing:** `--verify` reads back the range written. The
  end-to-end check that it landed where the *reader* looks is the device
  displaying the container digest at load (spec §4.2/§5.4), which must equal
  `616f 88f5 bb98 2e84 eb3d 0b5a f3d3 8777` — the number `me sysw pack` /
  `me sysw show` prints on the host. Use both.
- The two real traps are procedural, not in the map — see I2.

## 4. Q3 — Is signing safe to repeat, and does it fail loudly?

Yes, with one gap (I1). `sign-firmware.sh`:

- **Never mutates its input** (copies to `.signed.uf2`); re-running regenerates
  the output from the input — idempotent, freely repeatable, and its header
  says truthfully that nothing in it touches OTP.
- **Fails loudly at every hinge:** missing key/image die; non-secp256k1 key
  dies; "no SIGNATURE section" is *distinguished* from other picosign errors
  (no silent re-seal); digest-unchanged assertion (step 5) proves the sig sits
  outside the hashed region; offline openssl verify (step 6); embedded-bytes
  vs DER r‖s comparison; picotool structural check demanding exactly 2
  metadata blocks and `signature: verified`.
- `sh2-flash` then independently refuses to flash anything picotool won't
  vouch for (line 213), and flashes `$SIGNED`, never `$IMAGE`. A
  `.signed.uf2` input is flashed as-is, not double-signed.
- Missing key → `sh2-flash` dies **in preflight**, before build, before the
  prompt. Correct.
- **The gap (I1):** a syntactically valid but *wrong* secp256k1 key passes
  every one of those checks — they prove internal consistency, not that the
  key is in *your* slot — and yields exactly the dark-screen-plus-BOOTSEL the
  script's own header warns about. The fork's guide
  (`docs/custom-firmware.md`, "One extra check before you flash") mandates a
  pubkey-hash-vs-burned-fingerprint comparison; **sh2-flash does not implement
  it.** Today it cannot bite (default key verified == burned fingerprint), but
  `-k`/`SH2_KEY` overrides re-open it.

## 5. Q4 — Ordering

- **Data-independent.** picotool erases only the sectors it writes; the
  firmware image ends ~12.6 MiB below the region; a firmware flash preserves
  the payload and vice versa (EPD measured this for 0x10E00000; the geometry
  is identical here).
- **No boot-wedging state exists.** The region is read on demand from menu
  flows (verified, §1); absent/garbage/wrong-magic regions make `syswOffer`
  fall back to typed entry. Old firmware + new payload = payload invisible
  (confusion, not damage). There is no state that cannot be corrected from
  BOOTSEL, and none that traps the UI.
- **Recommended order: firmware first, judge the boot on machine power, then
  payload.** Not for safety — for diagnosis: it isolates the signing chain
  from the payload write, so a dark screen has one suspect. The payload write
  afterward costs one more BOOTSEL entry and ~2 seconds.

## 6. Q5 — Is `--dry-run` honest?

**About writes, yes — verified by execution and by code audit.** Every writing
step (`worktree add`, build, sign, `picotool load`, `reboot`) routes through
`run()`/`devshell()`, both of which print instead of executing under `DRY=1`.
Executed today it printed the full would-run ladder and wrote nothing.

Three honesty wrinkles, all Minor:
- The **lsusb device check is not gated on DRY** (line 230): with no device a
  dry run *dies* at "Device" and never previews the flash steps (writes
  nothing, shows less than it claims). With a device it prints a live-looking
  `OK RP2350 present in BOOTSEL` — which is how this review discovered the
  device was attached.
- Dry-run skips the picotool-reachability preflight (gated `DRY=0`), so a dry
  run can pass where a real run dies in preflight.
- Dry-run with `--pick`/non-HEAD: `BUILD_DIR` is never created, so the
  dirty-tree warning fires falsely and `git log` errors to stderr. Cosmetic.

## 7. Findings

**Critical — none.** The path as written cannot permanently destroy the
machine or its key material. (A clean row is a real result: the unrecoverable
class is OTP writes, and there are none here.)

**Important:**

- **I1 — sh2-flash omits the guide's key-vs-fingerprint check.** A valid
  secp256k1 key that is not the burned key passes sign-firmware.sh entirely
  and picotool reports `signature: verified`; the bootrom then rejects it:
  dark screen + BOOTSEL, the exact misdiagnosis this script exists to prevent.
  Recoverable, hence Important. Mitigated today (default key verified against
  `~/.sh2/sh2-boot-key.fingerprint`, hash `846aa289…`). Fix: before the
  confirm prompt, hash the embedded pubkey of `$SIGNED` (uncompressed X‖Y,
  `xxd -r -p | sha256sum`) and require equality with the fingerprint file.
  ~3 lines; both artifacts already on disk.
- **I2 — the payload write is hand-typed, and every default is wrong.**
  (a) picotool v2.2.0-a4, measured: a BIN load without `-o` goes to
  **`0x10000000`** — the region image lands on the firmware vector table;
  no-boot + BOOTSEL; misdiagnosis shape again. (b) The only greppable
  `-t bin -o` command in this repo's docs is `design/MNEMONIC-INTEGRATION.md:56`
  with the **superseded address `0x10800000`** — copy-paste writes 64 KiB to
  flash nothing reads, and the payload is silently "absent". (c)
  `me sysw pack --region` prints "write it at 0x10D00000" but not the command.
  Fix: give sh2-flash a payload mode (it already has the BOOTSEL check,
  confirm prompt, and devshell plumbing), or at minimum make `--region` print
  the exact `picotool load … -t bin -o 0x10D00000 --verify` line and mark the
  stale doc superseded.

**Minor:**

- M1 — dry-run device-check gating and preflight skip (§6).
- M2 — `sh2-flash` identifies the target only as "some RP2350 in BOOTSEL";
  SH2 vs bench Pico 2 is indistinguishable to it. picotool fails loudly on
  multiple attached devices, and the CHIPID check exists (`picotool otp get
  CHIPID0..3`) but only as manual discipline. Today: exactly one device,
  identified as the SH2.
- M3 — stale `me` 0.3.0 at `~/.cargo/bin/me` (no `sysw` subcommand; fails
  loudly). Use `target/release/me` (0.5.1, verified).
- M4 — newest-uf2-by-mtime selection (sh2-flash line 179) could flash a stale
  build if the flake ever changes its output location while exiting 0. The
  version string on the boot screen is the operator's cross-check.

## 8. GO / NO-GO and the sequence

**GO** — conditional on typing the payload offset explicitly (I2) and not
overriding `-k` (I1). Every artifact in the chain was machine-verified
consistent this session; slot-0 recovery is proven end-to-end; the attached
BOOTSEL device is confirmed to be the SH2.

```sh
# 0. Target identity (done this session; repeat if the device was replugged):
cd /scratch/code/shibboleth/seedhammer
nix develop --command picotool otp get CHIPID0 CHIPID1 CHIPID2 CHIPID3
#    expect f55c / 45ab / 83b7 / 77c4  (= SH2 f55c45ab83b777c4)

# 1. Build + sign, stopping before the device:
/scratch/code/shibboleth/mnemonic-engrave/scripts/sh2-flash --build-only

# 2. The check sh2-flash omits (I1) — bind the image to the burned slot:
nix develop --command bash -c \
  'picotool info -a <IMAGE>.signed.uf2 | awk "/[Pp]ublic key:/{print \$NF}" | head -1 \
   | xxd -r -p | sha256sum'
#    MUST equal ~/.sh2/sh2-boot-key.fingerprint (846aa289f2f3…d64cabb4). Mismatch = STOP.

# 3. Flash the signed artifact (script re-verifies the signature, prompts, loads --verify, reboots):
/scratch/code/shibboleth/mnemonic-engrave/scripts/sh2-flash <IMAGE>.signed.uf2

# 4. JUDGE THE BOOT ON MACHINE POWER. Unplug USB, machine PSU, expect (UNLOCKED).
#    Dark screen on laptop USB means nothing (PD check runs before LCD init).
#    If it does not boot on machine power: do NOT touch OTP; recovery is
#    BOOTSEL + re-sign, or slot 0:
#    nix develop --command picotool load --verify ~/.sh2/recovery/seedhammerii-v1.4.3.uf2

# 5. Payload. Note the digest on the host first:
/scratch/code/shibboleth/mnemonic-engrave/target/release/me sysw show /tmp/claude-1000/hello-world-region.bin
#    digest: 616f 88f5 bb98 2e84 eb3d 0b5a f3d3 8777

# 6. Re-enter BOOTSEL (hold button, plug in), then — OFFSET TYPED, VERIFY ON:
cd /scratch/code/shibboleth/seedhammer
nix develop --command picotool load --verify /tmp/claude-1000/hello-world-region.bin -t bin -o 0x10D00000
nix develop --command picotool reboot

# 7. On machine power: open a flow that offers the payload; the device MUST
#    display the same digest as step 5. Same number = the write landed where
#    the reader looks. Different or absent = wrong offset or torn write —
#    harmless, return to step 6.
```

Reviewed: `scripts/sh2-flash`, `scripts/sign-firmware.sh` (whole files),
`seedhammer/flake.nix` build-firmware, `seedhammer/sysw/` reader,
`SPEC_systemwide_payloads.md` §4–5, `RUNBOOK_custom_boot_key.md` step 6,
`docs/custom-firmware.md`, live device OTP (read-only), picotool v2.2.0-a4
behavior, and the sample region image.
