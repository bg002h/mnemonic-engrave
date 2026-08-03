# SH2 final gate — sh2-only-code lens

*Persisted verbatim. 2026-08-03, final gate before the first irreversible OTP write on the real SeedHammer II. Combined verdict across the three lenses: 1 Critical / 8 Important. NOT safe to write until folded.*

---

## Verdict

The SH2 modes are, on the whole, well built — and three of the four things this lens was most worried about check out under execution. But there is **one Critical false-PASS** on the gate immediately before the irreversible write, and four Important issues. Details, then the clean list.

I executed `--sh2-verify-slot` and `--sh2-verify-valid` against `scripts/test/fake-picotool` for every case below; every claim marked *(executed)* is a real observed output, not a reading.

---

### C1 — Critical — `key_hash` accepts **any** EC curve, so `--make-otp-json` and `--sh2-verify-slot` green-light a key that can never boot anything

`scripts/pico2-bootkey-rehearsal.sh:254-269` (used at `:425` in `make_otp_json`, `:449` in `verify_slot_or_die`)

`key_hash` runs `openssl ec -pubout -conv_form uncompressed -outform DER | tail -c 64` and validates only that the result is 128 hex chars. It never checks the curve. Every EC curve satisfies that check:

```
secp256k1: hexlen=128    prime256v1: hexlen=128
secp384r1: hexlen=128    secp521r1:  hexlen=128
```

For P-384/P-521 `tail -c 64` slices the *middle* of a 96/132-byte point — the hash isn't even a faithful identity for the key.

**Failure scenario (executed).** Operator's `openssl ecparam` line is mistyped, pasted from a generic ECDSA tutorial, or `~/.sh2/sh2-boot-key.pem` gets swapped with an unrelated EC key. `--make-otp-json` prints *"otp json validated: single key, slot 1, 32 bytes, matches openssl-derived hash"*. `picotool otp load` burns it. Then:

```
  PASS: Slot 1 matches .../p256.pem across all 16 rows.
  ..  ONLY NOW is it safe to set the valid bit (runbook step 3).
```

(identical output for the P-384 key). Operator runs `otp set -s BOOT_FLAGS1.KEY_VALID 0x2` — both irreversible writes done. The curve check exists at `scripts/sign-firmware.sh:75-77` (`"key is not secp256k1 -- RP2350 secure boot requires secp256k1"`) but that is **step 5**, after both burns. A spare slot is permanently spent on a hash no signature can ever match, and every gate said PASS.

**Fix.** Move the curve assertion to the front of `key_hash` (it is one line, already written at `sign-firmware.sh:75`):
```sh
openssl ec -in "$1" -noout -text 2>/dev/null | grep -qi 'ASN1 OID: secp256k1' \
  || die "key_hash: $1 is not secp256k1 -- RP2350 secure boot requires secp256k1"
```
and validate the *full* DER point length rather than blind `tail -c 64` (a secp256k1 uncompressed point is exactly 65 bytes `04||X||Y`).

---

### I1 — Important — `reject_rehearsal_key` fails open two ways, and one of them is hardware-bug class (3) recurring

`scripts/pico2-bootkey-rehearsal.sh:604-621`

**(a) `:607` — `die` inside `$(...)` inside a `[ ]` test.** `[ "$h" != "$(key_hash "$p")" ] || die` — when `key_hash` dies, only the subshell exits, the substitution yields `""`, the `!=` is true, and the `||` short-circuits. `set -e` does not help here: command-substitution failure only propagates when the substitution *is* the command, not when it is an argument to `[`. This is exactly the "`die` unreachable inside a substitution" class found on hardware — `read_rows`/`ask_blink`/`read_row_raw24` were all converted to globals or bare assignments for this reason; this one was missed.

*Executed:* with `rehearsal-work/my-key.pem` truncated to zero bytes and the SH2 key being a byte-identical copy of it:
```
  PASS: key is not one of the 3 rehearsal key(s) on disk
  PASS: Slot 1 matches .../moved-key.pem across all 16 rows.
```
It says **3** — it counted a key it never actually compared against.

**(b) `:605` — a missing rehearsal key is silently skipped.** `[ -f "$p" ] || continue` runs before `seen=$((seen+1))`, so `seen` just drops. *Executed*, with `my-key.pem` moved out and the SH2 key being that same key:
```
  PASS: key is not one of the 2 rehearsal key(s) on disk
```
The `seen -gt 0` guard at `:614` was written to stop exactly this ("Printing PASS after comparing against nothing is exactly the false-green this guard exists to prevent") but it only catches *zero*, not *fewer than expected*.

**Failure scenario.** Post-rehearsal tidy-up moves or truncates `rehearsal-work/my-key.pem`; the operator (reasonably, since it worked) reuses the rehearsal key for the SH2. The guard prints PASS. That key's only remaining copy lives wherever the tidy-up left it, in a tree the project documents as disposable — and the slot is burned permanently.

**Fix.** Hash each rehearsal key into a variable via a bare assignment (so `set -e` closes the die path), and require the full set:
```sh
for p in ...; do [ -f "$p" ] || die "missing rehearsal key $p -- cannot check"; ph="$(key_hash "$p")"; ...; done
[ "$seen" -eq 3 ] || die ...
```
Also reject any `--key` whose `realpath` is under `$WORKDIR` — a *newly generated* key inside `rehearsal-work/` is not one of the three and passes today.

---

### I2 — Important — `--sh2-verify-valid` misdiagnoses *extra* KEY_VALID bits as an interrupted write, and prescribes another irreversible write

`scripts/pico2-bootkey-rehearsal.sh:733-742`

The arithmetic is correct (`WANT` = 3/5/9 for slots 1/2/3 — verified by execution). The problem is the single `-ne` comparison: it cannot distinguish *missing* bits (the recoverable interrupted-write case the message describes) from *unexpected extra* bits, and the message only explains the former.

**Failure scenario (executed).** Slot 1's valid bit gets set on an unburned slot (operator pastes step 3's two commands in the wrong order — see I3 below), then slot 2 is burned and validated correctly. The device is now genuinely fine: slot 2 holds the key, its valid bit is set, SeedHammer's slot 0 is intact. `--sh2-verify-valid 2` says:

```
  PASS: slot 2 readback matches the expected hash across all 16 rows
FAIL: KEY_VALID is 0x7, expected 0x5 (slot 0 + slot 2).
If it reads LOWER than expected, the redundant write was INTERRUPTED partway.
... re-run the identical
    picotool otp set -s BOOT_FLAGS1.KEY_VALID 0x4
```

The advice is an OTP write that cannot possibly change anything, on a device that is already correct, and the check can **never** pass no matter how many times it is followed. The operator's only remaining reads are "the tool is broken" or "burn slot 3".

**Fix.** Split the branches:
```sh
MISSING=$(( WANT & ~0x$KV )); EXTRA=$(( 0x$KV & ~WANT ))
[ "$MISSING" -eq 0 ] || die "<interrupted-write text, re-run otp set -s>"
[ "$EXTRA"   -eq 0 ] || die "KEY_VALID has bit(s) you never asked for (0x$KV vs 0x$(printf %x $WANT)).
Do NOT run another otp set. Find out which slot that bit refers to first."
```

---

### I3 — Important — `--sh2-verify-slot` does not check whether the valid bit is already set, and its failure text asserts that it isn't

`scripts/pico2-bootkey-rehearsal.sh:766-779`, message at `:451-455`

`--sh2-verify-slot` reads only slot 0, the CHIPID, and the 16 hash rows. It never reads `BOOT_FLAGS1`. Its mismatch message states as fact: *"The slot is not yet valid, so the board still boots normally."*

**Failure scenario (executed).** Quickstart section 3 puts `(e)` verify and `(f)` `otp set` adjacent; a double-paste, a scrollback re-run, or an operator who reads `(f)`'s slot table before running `(e)` sets the valid bit on a slot that is empty or half-burned. Running `--sh2-verify-slot 1` then prints:

```
FAIL: SLOT 1 READBACK MISMATCH -- DO NOT SET THE VALID BIT.
The slot is not yet valid, so the board still boots normally. Use the next
free slot instead; this one is permanently unusable.
```

Both sentences are false. The operator moves to slot 2 believing the device is untouched, and then hits the permanently-unpassable I2 loop. The device is not bricked (the bootrom cannot match an all-zero hash, and slot 0 stays valid), but its OTP is in a state nobody accounted for and no tool will confirm.

**Fix.** In `--sh2-verify-slot`, read `BOOT_FLAGS1.KEY_VALID` before `verify_slot_or_die` and refuse if `(KV >> slot) & 1` is already set, directing the operator to `--sh2-verify-valid` instead — the same shape as the mid-procedure guidance `--sh2-precheck` already carries at `:648-656`.

---

### I4 — Important — `--sh2-verify-valid` has **zero** coverage: not on hardware, and not in `run-e2e.sh` either

`scripts/test/run-e2e.sh:112-139`

`grep -rn 'sh2-verify-valid'` across the repo returns the script itself, the runbook, the quickstart, and two agent reports — **no test**. The 28/28 e2e green does not touch this mode at all. It is the gate on the *second* irreversible write, and I2 above is a defect it would have caught immediately.

Note also `scripts/test/fake-picotool:74` (`emit_row 0 "$sel"` fallthrough): raw rows `0x04b/0x04c/0x04d` return `0x000000` regardless of `KV`, so simply adding a happy-path case would leave the 3-copy redundancy check vacuous — which is why my run printed `all three BOOT_FLAGS1 copies agree (0x000000)` on a device with `KV=3`.

**Fix.** Add e2e cases: pass for slots 1/2/3; fail on KV-low; fail on KV-extra (I2); fail on `KI≠0`; fail on a wrong key. Teach `fake-picotool` to derive `0x04b/0x04c/0x04d` from `KV`, with an env knob to make one copy disagree so the degraded-redundancy branch is actually exercised.

---

### Minors

- **M1 — `make_otp_json` leaves a failed/truncated json at `--out`.** `:429-443`: `jq … > "$out"` happens *before* the five assertions, and nothing removes `$out` when one dies. Quickstart `(c)`→`(d)` are adjacent commands; an operator who scrolls past the FAIL pastes `picotool otp load` on an unvalidated file. **Fix:** write to `"$out.tmp"`, `mv` only after the last assertion.
- **M2 — the CHIPID pin still lives inside the repo.** `:67-73` moves `SH2_DIR` out of `rehearsal-work/` citing `git clean -xdf` — but `sh2-state/` is itself gitignored (`.gitignore:29`) and untracked, so `git clean -xdf` deletes it too. Fail-closed (`--sh2-verify-slot` dies with a correct message, and re-running `--sh2-precheck` re-pins before it errors on the burned slot — *executed*), but the recovery re-pins TOFU from whatever is connected at that moment. **Fix:** default `SH2_DIR` to `~/.sh2`, where the runbook already puts the key.
- **M3 — a half-programmed ECC row surfaces as a tool-shaped error, not "this slot is spent".** Verified against picotool `main.cpp:9234-9237`: an ECC-invalid row prints `(WARNING - ECC IS INVALID)`, which `read_rows:208-210` correctly turns into `die "picotool reported a warning reading: … (redundant rows disagree or ECC invalid)"`. Correct behavior, wrong message: the runbook (step 3 / Recovery) only teaches the `READBACK MISMATCH → move to slot 2` branch, so the operator sees an unfamiliar tool error and is likely to replug and retry rather than move on. **Fix:** catch the ECC-warning case in `read_slot` and emit the slot-is-spent guidance.
- **M4 — `sign-firmware.sh:87` uses `xxd`** — the exact host-tool hazard `pico2-bootkey-rehearsal.sh:264-265` and the runbook's own note (line 345-348) document avoiding, and it is not in the preflight at `:59-61`. Under `pipefail` a missing `xxd` aborts the script *silently* (no `die`, no message). Only reachable when the input image already carries an embedded pubkey, which is why the rehearsal never hit it. **Fix:** use the `od`-based path, or add `command -v xxd` to the preflight.

### Nit

- `:814` — `printf … "$(key_hash "$WORKDIR/$k.pem")"` is the second (and only other) swallowed-`die` substitution; phase 0, informational print only.

---

## What this lens found **clean** (each verified, not assumed)

- **`WANT=$(( 1 | (1 << SH2_SLOT) ))` is correct for slots 2 and 3, not just 1.** Executed: slot 2 → `0x5` PASS, slot 3 → `0x9` PASS. `require_spare_slot` runs before the shift, so no arithmetic-injection surface.
- **The hex parsing matches picotool's actual contract.** `main.cpp:9279` emits `" = %x\n"` — bare lowercase hex, one line per matching field — exactly what `otp_field:183-187` parses.
- **The 3-copy `BOOT_FLAGS1` comparison is genuinely sound, and the script's stated rationale for it is correct.** I checked the claim that raw row-number reads bypass redundancy: `filter_otp` (`main.cpp:7590-7595`) passes `nullptr` for the register on an absolute row number, so `redundancy` stays at its `-1` default, `main.cpp:9257-9260` takes the plain path, and picotool prints the *individual* row — not the majority vote, and it cannot emit a disagreement warning. Comparing `0x04b/0x04c/0x04d` really does detect a degraded write. (Had this resolved to the `BOOT_FLAGS1` register instead, `read_row_raw24` — which does *not* grep for `WARNING` — would have returned the majority vote for `0x04b` and produced a false "all three copies agree". It doesn't.)
- **Lens item 2's "never run on hardware" risk is smaller than it looks.** `--sh2-verify-valid`'s device reads are `otp_field BOOT_FLAGS1.KEY_VALID`, `otp_field BOOT_FLAGS1.KEY_INVALID`, `read_row_raw24 0x04b/0x04c/0x04d`, and `read_slot` — **all four already ran on the real SH2** during `--sh2-precheck` (which reads both fields and those exact three raw rows). What is untested on silicon is the arithmetic and the messages, which is what I exercised in the simulator.
- **Partially-written and empty slots fail closed.** Executed: 8-of-16 rows burned → `SLOT 1 READBACK MISMATCH` with both hashes printed; all-zero slot → same. No false pass.
- **`reject_rehearsal_key` compares by hash, not filename.** Executed: `rehearsal-work/my-key.pem` copied to `rehearsal-work/sh2-boot-key.pem` is still correctly refused (`REFUSING: … is a REHEARSAL key (my-key.pem)`). The concern that renaming defeats it is unfounded.
- **`--make-otp-json`'s five assertions hold.** If `picotool seal` ever stopped emitting `bootkey0`, `.bootkey0` → `null` → `jq length` → `0` → dies at the 32-byte check. The `has("crit1") or has("boot_flags1")` assertion is structurally vacuous (the transform can only produce one key) but harmless.
- **The CHIPID-pin multi-day story works.** Executed: pin deleted mid-procedure → `--sh2-verify-slot` dies fail-closed → `--sh2-precheck` re-pins *before* it errors on the burned slot → `--sh2-verify-slot` then works. A second SeedHammer or the consumed Pico is rejected (slot-0 tripwire fires before pinning).
- **No recurrence of hardware bug classes (1), (2), or (4) in the SH2 modes.** No `picotool info` program-metadata dependence anywhere in the SH2 paths (`bootsel_present` uses it only as a presence probe, and phase 5's `info -a` runs against a *file*). No single-row picotool query in these modes that `--sh2-precheck` has not already run on the real SH2. No assertion over a structured lock field treated as simple equality — `check_page_locks:365-402` decodes and majority-votes, which is what hardware forced.
- **The two manual irreversible commands fail closed on a stale Pico.** picotool errors `Multiple RP-series devices in BOOTSEL mode found` (`main.cpp:4691`), so an unplugged-Pico mistake aborts the write rather than misdirecting it.

## Cannot be verified from the repo

- Whether the RP2350 bootrom treats a *valid-but-all-zero* boot-key slot (the I3 out-of-order state) as merely unmatchable or as something worse. Nothing in the fork's `driver/otp/` or the runbook covers it, and the rehearsal never produced that state. I have assumed "unmatchable, harmless" throughout; if that assumption matters to the decision, it needs the datasheet, not this repo.
- Whether `otp set -s` on `BOOT_FLAGS1` can partially program in a way that reads back as *more* bits than requested. I treated "extra bits" as operator-caused only.

## Recommendation

Fix **C1** before touching the machine — it is one grep line, it defends the file that gets burned, and today `--sh2-verify-slot` will say "ONLY NOW is it safe to set the valid bit" for a key the RP2350 can never use. **I1** and **I2/I3** are each a few lines and all sit on the irreversible path. **I4** is what would have caught I2 for free.