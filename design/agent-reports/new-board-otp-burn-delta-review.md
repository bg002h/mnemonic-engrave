VERDICT: GO

Critical: 0 / Important: 1 / Minor: 3

Reviewer: independent adversarial delta review, 2026-09-17.
Question answered: *is anything about THIS burn, on THIS new board, different in a
way that matters from the burn already proven on `f55c45ab83b777c4` (2026-08-03)?*

**Answer: one thing is, and it is not the payload or the board — it is that the
identity defence the runbook relies on has been silently halved by the existence
of a second SeedHammer II.** Everything else I could attack came back clean, and
three independent measurements corroborate that this is the same part, same
factory provisioning, and same key. GO once I-1's one-flag remedy is typed in.

---

## I-1 (Important) — the two irreversible commands are the only steps with NO device binding, and for the first time a second device satisfies every non-CHIPID identity check

**Claim.** `picotool otp load` and `picotool otp set -s` are typed by the operator
outside the script and carry no device selector; on 2026-08-03 that was safe
because exactly one SeedHammer II existed in the world, and it is no longer safe
now that two do.

**Reproduction / measurement.**

1. The script contains no write path at all — by design. `scripts/pico2-bootkey-rehearsal.sh:555-557`:
   *"They invert the tripwire (they REQUIRE SeedHammer's key in slot 0) and contain
   no write path whatsoever -- no otp load, no otp set, no picotool load. The
   irreversible writes stay in the operator's hands, in the runbook."*
   So `sh2_require_seedhammer`'s CHIPID pin (`:594-599`) gates **only** the three
   read-only `--sh2-*` modes. The two writes pass through none of it.

2. The runbook's stated defence against a wrong device is the slot-0 tripwire plus
   "unplug every other RP2350 board" (`design/RUNBOOK_custom_boot_key.md`, step 1:
   *"A wrong-but-plausible device answers these reads with exactly the values you
   expect to see."*). That text was written when the only plausible impostor was
   the consumed rehearsal Pico, which **fails** the slot-0 check.
   The old SH2 `f55c45ab83b777c4` **passes** it: per
   `~/.sh2/SEED_HAMMER_OTP_SLOT_USAGE.txt`, its slot 0 holds
   `c8314536d6af61ac2e62e5991e3e4711629c54696ba8c4af08965a1d319a473b` — the same
   constant the script compares against (`pico2-bootkey-rehearsal.sh:81`).
   **CHIPID is now the sole discriminator between the two boards, and the
   irreversible commands do not check it.**

3. picotool 2.2.0-a4 *does* offer the binding, on both write verbs. Measured:
   `picotool help otp load` → `SYNOPSIS: picotool otp load [-r] [-e] [-s <row>]
   [-i <filename>] <filename> [-t <type>] [device-selection]`, with
   `--ser <ser>  Filter by serial number` under *Target device selection*.
   `picotool help otp set` lists the identical block.

4. The serial to bind to, measured passively from sysfs (no device command run):
   `/sys/bus/usb/devices/1-10/` → `idProduct=000f serial=09F50BF63E8D6F46
   product="RP2350 Boot"` — the only `2e8a:000f` present, and exactly the
   word-reversed form of the pinned CHIPID `6f463e8d0bf609f5`
   (`sh2-state/sh2-chipid.txt`), i.e. the same CHIPID↔serial relationship the
   old board's record documents (`f55c45ab83b777c4` ↔ `77C483B745ABF55C`).

**What this can and cannot do.**
- It **cannot** produce a false GREEN. If the wrong board is addressed, both
  `--sh2-verify-slot 1` and `--sh2-verify-valid 1` die `WRONG DEVICE` at
  `:595`. I tried to construct a path past it and could not.
- It **cannot** damage the old board: its slot 1 already holds this identical
  hash, and `otp set -s` is set-bits-only against a bit already set.
- It **can** waste a slot on the *correct* board. An `otp load` that errors
  because it hit the wrong board is textually indistinguishable from the
  runbook's own step-2 warning — *"an interruption leaves rows 0..k burned and
  the rest blank, and re-running it cannot repair that. That slot is then
  spent... the only remedy is the next free slot."* An operator following that
  advice burns slot 2 for nothing. That outcome is worse than saying nothing,
  which is what makes this a finding rather than an observation.

**Minimal remedy — one flag per command, zero risk, no re-review needed.**

```sh
picotool otp load ~/.sh2/otp-6f463e8d0bf609f5.json --ser 09F50BF63E8D6F46
# then, only after --sh2-verify-slot 1 passes:
picotool otp set -s BOOT_FLAGS1.KEY_VALID 0x2 --ser 09F50BF63E8D6F46
```

Device selection is documented as **trailing** in both synopses — keep it last.
The flag is safe to add unconditionally: a mistyped serial makes picotool find
*no* device and error out, it can never select a different one. This closes I-1
mechanically; it does not need another review round.

---

## M-1 (Minor) — `~/.sh2/SEED_HAMMER_OTP_SLOT_USAGE.txt` is a single-device record that this burn makes actively wrong

Its header says *"SLOT 1 BURNED, VERIFIED AND VALID (2026-08-03). The OTP work on
this machine is COMPLETE"*, and its DEVICE IDENTITY block says
*"CHIPID f55c45ab83b777c4 ... If a device presents different values, IT IS NOT
THIS MACHINE."* After this burn there are two machines, and a future operator
reading that line against the new board would conclude it is the wrong device.
**Does not block the burn** (nothing reads this file programmatically — verified:
no reference to it in `pico2-bootkey-rehearsal.sh`). Remedy: after the burn,
re-title it per-device or add a second DEVICE IDENTITY block for
`6f463e8d0bf609f5`. Post-burn work.

## M-2 (Minor) — the runbook hardcodes `~/.sh2/my-otp.json`, which is now the *old* board's filename

Steps 2 and 3 say `--out ~/.sh2/my-otp.json` and `picotool otp load
~/.sh2/my-otp.json`. The operator will use `otp-6f463e8d0bf609f5.json`. Measured:
`diff` of the two files reports **no difference** (both 297 bytes), so a literal
paste of the runbook line loads identical bytes and is harmless. Recorded only so
nobody "fixes" an apparent discrepancy mid-procedure.

## M-3 (Minor) — the runbook shows the write commands bare, but `picotool` exists only inside `nix develop`

Steps 2 and 3 print `picotool otp load ...` with no `nix develop --command`
wrapper, unlike every `--sh2-*` invocation around them. Measured: `which picotool`
→ not found; a filesystem-wide `find` returns exactly one binary,
`/nix/store/j7pl045ik6yb73zvq3n9a52j85d2qnig-picotool-2.2.0-a4/bin/picotool`
(v2.2.0-a4 — the version the runbook's resolved open-item #2 pins). Failure mode
is `command not found`, i.e. a stall, not damage. Run the burn from inside
`nix develop`, as on 2026-08-03.

---

## WHAT I CHECKED AND FOUND CLEAN

**1. The payload (asked: is it correct, and is byte-identity suspicious?).**
`diff ~/.sh2/otp-6f463e8d0bf609f5.json ~/.sh2/my-otp.json` → **identical**, 297
bytes each. This is **correct, not suspicious**: an RP2350 boot-key slot stores
SHA-256 of the uncompressed 64-byte X||Y pubkey and nothing board-specific, so
the same key at the same slot *must* produce the same file. Had they differed,
that would have been the alarm. Verified independently, not trusted:
- `openssl ec -in ~/.sh2/sh2-boot-key.pem -pubout -conv_form uncompressed
  -outform DER | tail -c 64 | sha256sum` →
  `846aa289f2f317e55ff03f90555132302842cff2f68ee45712834a25d64cabb4`
- `jq -r '.bootkey1[]' ... | awk '{printf "%02x",$1}'` → **the same 64 hex chars**
- `~/.sh2/sh2-boot-key.fingerprint` → the same value
- curve is `ASN1 OID: secp256k1` (the script's `key_hash` rejects anything else
  at `:266-271`, for exactly the "every EC curve yields a plausible 64-byte
  slice" reason)
- single top-level key `bootkey1`, 32 entries, no `crit1`/`boot_flags1`.

**2. The signed firmware really is signed by the key about to be burned** — proven
directly, not inferred from "it booted on the old board" (which would have been
weak: the old board trusts two keys). `picotool info -a
seedhammerii-v0.0.0-bgf5b068f.signed.uf2` reports `signature: verified`, exactly
**two** metadata blocks (three would mean double-sealed), and embeds
`public key: 3B73...93AD`. sha256 of those 64 bytes as printed =
`846aa289f2f317e55ff03f90555132302842cff2f68ee45712834a25d64cabb4` — **an exact
match for the hash going into slot 1.** File sha256 confirms
`7fb899cb4ea3a9fbcfc8f3f0299fa53d9c850eed40ce53a10845a0acebeaaf0c`.

**3. Carried-over state between boards (asked: stale pins, cached keys, hardcoded
chipid, first-run-only branches).** All clean:
- `sh2-state/sh2-chipid.txt` = `6f463e8d0bf609f5` — the **new** board. The old
  pin is parked at `sh2-state/board-f55c45ab83b777c4.chipid` where nothing reads
  it (the script only ever opens `$SH2_DIR/sh2-chipid.txt`).
- `sh2-state/` is **gitignored** (`.gitignore:28`) and `git ls-files sh2-state/`
  is empty — so no `git checkout`/`git clean` can restore the old board's pin
  under a later run. (`git clean -xdf` would *delete* it, which the script
  handles by refusing to run, not by re-pinning — see next item.)
- The one first-run-only branch is the pin-create at `:600-602`, and it is
  fenced to `--sh2-precheck` alone; every other mode `die`s if the pin is
  missing (`:604-610`) precisely so a later mode cannot downgrade a
  wrong-device check into a first-time pin and still print PASS. Re-pinning a
  *burned* board is additionally impossible because `--sh2-precheck` requires
  `KEY_VALID == 0x1` exactly.
- No CHIPID is hardcoded anywhere: `grep -nE '[0-9a-f]{16}'` over the script
  returns only the all-zeros rejection literal at `:313` and the slot-0
  `SH_SIGNKEY_HASH` at `:81`.
- `$WORKDIR/board-chipid.txt` belongs to the rehearsal `--phase` path
  (`require_board`, `:323-337`) and is not consulted by any `--sh2-*` mode.
- The cached `rehearsal-work/blinky.uf2` reused by `build_blinky` (`:431`) does
  not affect the payload: `make_otp_json` asserts byte-equality against the
  openssl-derived hash regardless of which image was sealed (`:459-463`).
- All three rehearsal keys are present (`factory-key.pem`, `my-key.pem`,
  `third-party-key.pem`, Aug 3), so `reject_rehearsal_key` compares against a
  full set rather than dying or, worse, passing against nothing (`:655-664`).

**4. False-PASS construction against `--sh2-verify-slot` / `--sh2-verify-valid` —
I tried and could not build one.** The paths I attacked:
- *Vacuous equality in the BOOT_FLAGS1 three-copy check.* `A="$(read_row_raw24
  0x04b)"; B=...; C=...` would compare three empty strings as equal if
  `read_row_raw24` could return empty. It cannot: it `die`s on a failed read and
  on an unparseable VALUE line (`:363-368`). I also **executed** the errexit
  question rather than reasoning about it — `bash -c 'set -euo pipefail; f() {
  exit 1; }; A="$(f)"; B="$(f)"; echo REACHED'` exits 1 without printing, so a
  `die` inside the substitution does halt the script.
- *Parser-level.* `read_rows` (`:204-240`) asserts the parsed row **count**
  equals the requested count *and* that picotool's echoed row **names** match
  the requested selectors positionally — so picotool's ascending sort + silent
  dedup cannot shift a value into the wrong position. `otp_field` and
  `read_row_raw24` both `die` on any `WARNING` in combined stdout+stderr, which
  is what makes a degraded 2-of-3 redundant write fail closed instead of
  majority-voting to the expected answer.
- *Empirically.* The negative control already run on this board reached
  `SLOT 1 READBACK MISMATCH` after clearing the chipid pin and the
  not-yet-valid gate — i.e. the entire wrapper chain executed here, and only
  the final string comparison separates it from the post-burn success path.
- The only remaining way to pass is `SLOT_HEX == sha256(X||Y)` for the same pem
  file used by `--make-otp-json` and by `sign-firmware.sh`, which item 2 above
  shows is the key the flashed image is signed with. A "wrong key" would be
  consistently wrong across all three and would still boot.

**5. `0x2` and `otp set -s` semantics — resolved against the real tool, not the
prose.** `picotool otp list` prints `field KEY_VALID (bits 0-3)`, so slot 1's bit
is `1<<1 = 0x2`. `picotool help otp set` documents `-s, --set-bits  Set bits
only` — it **ORs in**, it does not overwrite. That matches the script's
expectation `WANT=$(( 1 | (1 << SH2_SLOT) ))` = `0x3` for slot 1 (`:766`), the
2026-08-03 observed `KEY_VALID 0x1 -> 0x3`, and the recovery advice that
re-running the identical `otp set -s` after an interrupted write is free.
Row layout also confirmed from the tool: `ROW 0x0090: OTP_DATA_BOOTKEY1_0 (Part
1/16)` … `ROW 0x009f: OTP_DATA_BOOTKEY1_15 (Part 16/16)`.

**6. Page permissions cover both pages this burn touches.** BOOTKEY1 rows
`0x090-0x09f` fall in OTP page 2; `BOOT_FLAGS1` row `0x04b` falls in page 1.
`check_page_locks` reads **all four** of `PAGE1_LOCK0/1`, `PAGE2_LOCK0/1`, and
the settled readings (`LOCK0 = 0x000000`, `LOCK1 = 0x040404` → `LOCK_S=0`,
`LOCK_BL=0`, `KEY_R=0`, `KEY_W=0`) mean both pages are Secure-writable. No gap.

**7. This is the same class of hardware as the proven board — third, independent
corroboration.** Beyond the slot-0 hash and the `signKeyHash` check already
settled, I read the BOOTSEL mass-storage SCSI inquiry passively from sysfs:
`vendor=[SH      ] model=[SHII            ] rev=[5   ]`. That is byte-for-byte
the string the old board's record documents (*"SH  SHII  5"*), proving (a) the
factory `writeOTPValues()` white-label provisioning ran on this board too, and
(b) `boardVersion()`'s `SYSINFO.PACKAGE_SEL == "5"` → RP235xB, the same package
as the proven unit. This is a genuine, factory-provisioned SeedHammer II control
board, not a bare RP2350.

**8. Single device on the bus, confirmed independently of the settled fact.** The
sysfs sweep for `idVendor == 2e8a` returns exactly one entry, `1-10`, serial
`09F50BF63E8D6F46`.

**9. Recovery path intact on the new board.** Slot 0 holds SeedHammer's
production key hash, `KEY_INVALID == 0`, and `~/.sh2/recovery/` holds the
official v1.4.3 image signed by that key — which is board-independent. Runbook
step 7 ("do not revoke slot 0") is respected by this procedure; neither command
touches `KEY_INVALID`.

### Out of scope, per the brief
Not re-audited: the runbook as a whole, the codec, `sh2-flash`, the post-burn
build/sign/flash steps (retryable at zero OTP cost), and the security consequence
that two machines now trust one key (intended by the operator, stated in the
brief). No device write was performed; every picotool invocation was
`version`, `help`, `otp list`, or a file-only `info -a`.
