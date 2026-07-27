# Lens 3 — failure-injection review

*Persisted verbatim. opus, 2026-07-26 three-lens fan-out (round 5). NOTHING FOLDED YET.*

---

I have everything I need. picotool 2.2.0-a4 source was available at `/tmp/picotool`, so the write-path claims below are read from source rather than assumed.

---

# Failure-injection review — quickstart, runbook, rehearsal script, sign-firmware

## C1 — Critical — Nothing verifies SH2 state *after* the writes; the final irreversible write is gated by a raw picotool call that ignores picotool's own warning, and `--sh2-precheck` hard-refuses the device from step (d) onward

**`FIRMWARE-QUICKSTART.txt:117-119`**, `design/RUNBOOK_custom_boot_key.md:209-211`, `scripts/pico2-bootkey-rehearsal.sh:452-466`

The whole review series built a fail-closed reader — `otp_field()` (script:157-159) *dies* on any picotool WARNING. Every pre-write check uses it. The post-write check does not: quickstart 3f and runbook step 3 end with a bare `picotool otp get -n BOOT_FLAGS1.KEY_VALID  # expect field = 3`.

`otp set` writes `reg->redundancy` rows in **one** PICOBOOT command (`main.cpp:9700-9704`: `wRowCount = 3`, rows `0x04b-0x04d`). An interruption inside that command leaves 1 or 2 of 3 copies programmed.

- **2-of-3 written.** picotool's vote is `sets[b] >= clears[b]` (`main.cpp:9245`) → prints `field ... = 3` **and** `(WARNING - REDUNDANT ROWS AREN'T EQUAL)`. The card's `expect field = 3` **passes**. Operator records success; the machine works; BOOT_FLAGS1 redundancy is permanently degraded on the one bit whose failure means "stops trusting your key".
- **1-of-3 written.** Vote → `field = 1`. The fork firmware will not boot. The correct remedy — re-run the identical `otp set -s`, which is OR-only and safe to repeat — appears nowhere. The card's TWO RULES #2 says the opposite: *"If firmware does not boot but step (e) passed, the hash is PROVEN correct. It is a signing problem."* Step (e) proved the **hash rows**; it says nothing about the **valid bit**. The operator re-signs indefinitely against a cause they have been told to rule out.

Compounding: from step (d) onward the operator has **no working survey tool**. `--sh2-precheck` dies at `KEY_VALID is 0x3, expected exactly 0x1 ... STOP` (script:453) and at `spare slot 1 is NOT empty ... this device has been modified before. STOP` (script:463) — both now true *by design*. This is the SH2 analogue of round 3's M4 that the brief asked me to look for, and the consequence is not $5: an operator who re-runs the precheck after an interruption is told their machine has been tampered with.

**Fix.** Add a read-only `--sh2-verify-valid N --key K`: `sh2_require_seedhammer` → `otp_field BOOT_FLAGS1.KEY_VALID` (fails closed on WARNING) equals `0x1 | (1<<N)` → `KEY_INVALID == 0` → `verify_slot_or_die N K` → `read_row_raw24` on `0x04b/0x04c/0x04d` requiring all three **equal**. Make quickstart 3f and runbook step 3 call it. Add the sentence "a short KEY_VALID means the redundant write was interrupted — re-run the identical `otp set -s`; it only sets bits." Teach `--sh2-precheck`'s two failure messages to name the mid-procedure state instead of implying tampering.

## I1 — Important — `expect field = 3` is hard-coded to slot 1, but the card's own failure path sends the operator to slot 2

**`FIRMWARE-QUICKSTART.txt:113-119`**

The runbook qualifies it — `# expect field = 3 (slot 0 AND slot 1)` (`:210`). The card drops the qualifier while keeping `# slot 2 = 0x4, slot 3 = 0x8` on the line above, and 3e says "If this fails, STOP and use slot 2".

Operator's (e) fails → moves to slot 2 → `otp set -s BOOT_FLAGS1.KEY_VALID 0x4` → picotool reports `field = 5` → the card says expect 3. They now believe the most dangerous write in the procedure just failed. The plausible improvisation is to re-run with `0x2`, which permanently validates **slot 1 — the slot that just failed verification**. Not destructive (a garbage slot matches no image, slot 0 still boots official firmware), but it is an irreversible write the entire document exists to prevent, taken because the card's expected value was wrong.

Separately, 3c/3d/3e all hard-code `1` with no variable, so the documented slot-2 fallback requires hand-editing three commands under stress — the improvisation channel round 3's C1 was written to close, reopened in the artifact that is actually pasted from.

**Fix.** `S=1; BIT=$((1<<S))` and `expect field = $((1|BIT))`, or write the slot-2/slot-3 sequences out verbatim. At minimum restore the runbook's parenthetical.

## I2 — Important — `--make-otp-json --slot 0` is accepted, prints PASS, and produces a json targeting SeedHammer's own slot

**`scripts/pico2-bootkey-rehearsal.sh:106,109,433,437,490,495`** — `SH2_SLOT` is never range-checked anywhere.

`--make-otp-json --key K --slot 0 --out F` emits `{"bootkey0": [...]}` and passes **all five** of `make_otp_json`'s assertions (single key, correct slot name, 32 bytes, no crit1/boot_flags1, matches openssl) → prints `PASS: otp json validated`. The safety apparatus actively blesses it. Quickstart 3d is then a bare `picotool otp load $PWD/my-otp.json`.

I traced what actually happens. `process_otp_json` for an array value does **no** read-back, **no** "cannot clear bits" check, and **no** "cannot modify ECC row" check — it calls `con.otp_write` directly (`main.cpp:9120`) and returns before the verify loop (`main.cpp:9445`, *"Return now, don't do rest of function"*). The only protection is in the bootrom, which returns `PICOBOOT_UNSUPPORTED_MODIFICATION` for an already-programmed ECC row (`main.cpp:8952`). Slot 0's 16 rows are all programmed, so the first row is rejected and picotool aborts. **This is not a brick** — round 3's destruction inventory survives. But the operator gets an unexplained hard failure inside the irreversible step, and `--sh2-verify-slot 0 --key sh2-boot-key.pem` then tells them, verbatim, `SLOT 0 READBACK MISMATCH ... this one is permanently unusable` — i.e. that SeedHammer's production slot, the entire recovery path, is dead.

**Cannot verify from this repo:** whether the bootrom rejects the 16-row batch *before* programming any row, or per-row as it walks. I have assumed the safe reading. If it is per-row and any row of slot 0 could take bits, this is Critical, not Important — the guard costs three lines either way.

**Fix.** `case "$SH2_SLOT" in 1|2|3) ;; *) die "slot must be 1, 2 or 3 — slot 0 is SeedHammer's production key and must never be written" ;; esac` in both modes, and refuse to emit a json keyed `bootkey0`.

## I3 — Important — the 16-row `otp load` is unverified and an interruption inside it is unrecoverable; neither document says the window exists

**`FIRMWARE-QUICKSTART.txt:110-112`**, `design/RUNBOOK_custom_boot_key.md:180-184`

Answering the lens question directly: interruption during `picotool otp load` leaves rows `0..k` programmed and `k+1..15` zero. `--sh2-verify-slot` **correctly catches this** — good. But:

1. The load itself reports nothing. The JSON path returns before the read-back-and-compare that the BIN path and the command's own doc string ("into OTP and verify") provide (`main.cpp:9430-9445`). There is no "Verified OK", and its absence is not flagged as meaningful anywhere.
2. **Re-running `picotool otp load` cannot repair it.** The bootrom rejects a write to an already-programmed ECC row regardless of whether the value is identical (host-side mirror: `main.cpp:9692`, `if (old_raw_value && otp_cmd.bEcc) fail("Cannot modify OTP ECC row(s)")`). So the operator's natural reflex — "it errored, try again" — fails, and `verify_slot_or_die`'s "permanently unusable" is literally true.
3. Therefore one USB glitch during a ~1-second write costs a spare slot outright, and **nothing in either document mentions this** or asks the operator to stabilise the link for it. Quickstart `00.` covers udev rules and charge-only cables but never says: short cable direct to the host, no hub, laptop on AC with suspend disabled, don't touch the bench during (d) and (f). The rehearsal never exercises the state, so its cost is discovered on the real machine.

**Fix.** State the window in both docs with the "re-running the load will not help" fact. Add a pre-write stability checklist. Have `verify_slot_or_die` distinguish *read ⊂ expected* ("interrupted write — slot consumed, move to the next") from *conflicting bits* ("wrong key or wrong json — check before spending another slot"), so the operator gets the true cause.

## I4 — Important — a **dry run** of phase 3 silently un-signs the phase-5 A/B image and turns the positive control into a false FAIL

**`scripts/pico2-bootkey-rehearsal.sh:623`** — the `cp` is not wrapped in `run`, unlike the `sign_image` three lines below it (`:627`). Phase 5's only check on that artifact is `[ -f ... ]` (`:693`).

Quickstart line 90 explicitly invites the trigger: *"Drop --execute on any phase for a dry run."* Operator runs phase 3 `--execute` (green), phase 4 `--execute` (**slot 1 burned**), then re-runs `--phase 3` without `--execute` to re-read what it does. The unguarded `cp` overwrites `blinky-mykey.uf2` with the **unsigned** blinky. Phase 5 flashes it, the bootrom correctly rejects an unsigned image, and the script dies:

> `Your key is burned and valid, but your signed image still will not boot. Something in the signing chain is wrong -- investigate before touching the SeedHammer II.`

A false FAIL of the load-bearing positive control, on a board whose OTP is already spent, instructing the operator not to proceed — caused by an action the card documents as read-only. It fails safe, but it aborts a correct procedure, and the die text's re-sign advice cannot help because phase 5 never re-signs.

**Fix.** Wrap the `cp`s at `:623`, `:624`, `:754` in `run`. In phase 5, assert the image is *actually signed by `my-key.pem`* (`picotool info -a` reporting `signature: verified` plus a pubkey match) rather than merely present — the comment at `:693` claims "so this is a true A/B" and existence does not establish that.

## I5 — Important — `sign-firmware.sh` mutates its input in place and will re-sign SeedHammer's official release with your key, destroying your local recovery image

**`scripts/sign-firmware.sh:82,97,109`** — every step writes to `$IMG`; there is no output argument and no backup. The "SIGNATURE section already present" branch (`:75-76`) accepts an image already signed by SeedHammer and overwrites its pubkey without comment.

The fork's `.gitignore` is `seedhammerii-*.uf2` (verified), so the official `seedhammerii-v1.4.3.uf2` downloaded to verify the recovery path (runbook open item 5) and the fork build sit side by side, untracked, in the same directory. **Quickstart step 4 writes the filename as the literal placeholder `seedhammerii-<version>.uf2` — twice — so it must be hand-completed or tab-completed**, while step 0 uses a different form (`seedhammerii-$(git rev-parse HEAD).uf2`). One tab-complete onto the release destroys the only local copy of the recovery image, silently, ending with a green `PASS: ... the signature is proven valid offline`.

Cost: recoverable only by re-downloading from seedhammer.com — i.e. discovered exactly when the machine is already in a bad state and you needed the file.

**Fix.** (a) Write to `<img>.signed.uf2` (or take an output path) and never touch the input. (b) In the already-present branch, extract the embedded pubkey and `die` if `sha256(uncompressed)` == `c8314536…319a473b`: *"this is an official SeedHammer release — signing it would destroy your recovery image."* (c) In the quickstart, use `$(git rev-parse HEAD)` consistently instead of a `<version>` placeholder. Also worth one line: after signing, the artifact's sha256 no longer matches CI, so the step-0 reproducibility proof cannot be re-demonstrated on the file you flash.

## I6 — Important — the SH2 identity pin and the rehearsal-key guard both live in `rehearsal-work/`, which every document teaches the operator to throw away — and both print PASS when they verify nothing

**`scripts/pico2-bootkey-rehearsal.sh:66,402-411,417-428`**

`rehearsal-work/` is gitignored (so `git clean -xdf` removes it), the script header calls the board "CONSUMED", the quickstart says the same, and `reject_rehearsal_key`'s own comment (`:414-416`) calls the directory "documented as disposable". So deletion is expected behaviour, not an accident. Three consequences:

- **CHIPID pin (`:402-411`).** If `sh2-chipid.txt` is missing when `--sh2-verify-slot` runs, the `else` branch re-pins and prints `PASS: pinned SeedHammer II CHIPID …` instead of `PASS: device identity matches the one pinned at --sh2-precheck`. Round 3's I2 binding degrades to nothing; the only signal is different PASS wording. Bounded by the slot-0 tripwire (only a real SH2 gets this far), so it matters only with two SeedHammers — but it is a check reporting green having checked nothing.
- **`reject_rehearsal_key` (`:417-428`).** It `continue`s over every missing `.pem`, then unconditionally prints `PASS: key is not one of the rehearsal keys`. Once `rehearsal-work/` is gone, round 3's I7 guard is entirely vacuous while still printing green.
- **Pico mirror.** Deleting `rehearsal-work/` mid-rehearsal is a hard dead end: `require_board` dies *"no pinned board -- run phase 0 first"* (`:209`), and phase 0 then dies in `assert_stock_or_die` — *"board already sealed. Get a FRESH board."* (`:229`). The keys are gone too, so the board is scrap and the rehearsal restarts on new hardware. A truncated write to `board-chipid.txt` (disk full) produces the same dead end via a `WRONG BOARD` on the right board.

**Fix.** Move SH2 state to a separate `sh2-state/` documented as **keep**; make `reject_rehearsal_key` die (or loudly warn) when it finds no keys to compare against; make `--sh2-verify-slot` *require* an existing pin rather than creating one.

## M1 — Minor — `--make-otp-json` silently depends on tinygo and a cached blinky, and fails with a message about a blinky

`scripts/pico2-bootkey-rehearsal.sh:123-127` gates the tinygo/go check on `PHASE` being `0|3|4|5|6`; in SH2 modes `PHASE` is empty, so it never runs — yet `make_otp_json` → `build_blinky` → `tinygo` (`:270-277`). With `rehearsal-work/` deleted (see I6) and the operator outside the devshell, quickstart step 3c dies with `blinky build failed` in the middle of the SeedHammer procedure. It also insists `$SEEDHAMMER_DIR` exist (`:128-129`) though the mode never uses it. **Fix:** add the toolchain check for this mode; better, use the fork's already-built UF2 as the `picotool seal` input so the mode has no toolchain dependency at all.

## M2 — Minor — after an *accepted* image the board leaves BOOTSEL, and nothing tells the operator to put it back

Phase 5a's blink means the image is running, so the board is not enumerable; 5b's `flash_image` (`:326-338`) then dies `FLASH FAILED ... (board connected? in BOOTSEL? image valid?)`. Same between phases 5 and 6. `scripts/rehearsal-blinky/main.go` has no USB at all, so `picotool load` cannot reset it. $0 and fails closed — but the die text offers "image valid?" as a cause at exactly the moment the operator is judging whether the 2.4 MB image is acceptable (F11(d)), an easy misread into "the real firmware failed". **Fix:** one `info` line before 5b and at the end of phase 5.

## M3 — Minor — `bootsel_present` fails **open** on slow USB re-enumeration

`scripts/pico2-bootkey-rehearsal.sh:364-368` — `sleep 3; picotool info`. If a *rejected* board takes longer than 3 s to re-enumerate (hubs, some xHCI stacks), `picotool info` fails, the function returns `no`, and 5b prints `PASS: real firmware accepted by the bootrom` — a false PASS on the only check covering the real image. The inverse (another RP2350 attached) fails closed, though the quickstart never says to unplug the SH2 *during the rehearsal* (only the reverse, at line 95). **Fix:** poll ~15 s requiring sustained absence (e.g. 5 consecutive misses), and print the raw result so the verdict is auditable.

## M4 — Minor — the machine's only signing key is created inside the fork's git worktree

`FIRMWARE-QUICKSTART.txt:104` and `design/RUNBOOK_custom_boot_key.md:149` both use `$PWD/sh2-boot-key.pem` with `$PWD` = the fork. The fork's `.gitignore` is only `seedhammerii-*.uf2` and `_artifacts` — I verified `git check-ignore` does **not** match `sh2-boot-key.pem` there. So it appears in `git status`, dies to `git clean -fdx`, and is one `git add -A` from being committed. (The `mnemonic-engrave` repo ignores `*-key.pem`; the fork does not.) The quickstart's backup instruction is the words "and back up your key" in a heading — no destination, no verification, no fingerprint to record. **Fix:** create it outside both worktrees, add `*-key.pem` to the fork's `.gitignore`, and make step (b) an explicit backup + a printed `key_hash` fingerprint to write down.

## Nits

- `scripts/pico2-bootkey-rehearsal.sh:96` — `read -r reply` under `set -e`: EOF (closed terminal, piped stdin) exits the phase silently with no message. Fails closed; cosmetic.
- `scripts/pico2-bootkey-rehearsal.sh:271` — `build_blinky` runs `tinygo` and writes to `$WORKDIR` during a `--dry-run` phase 0/3.

## Could not be verified from this repo

Stated plainly rather than assumed fine: whether the bootrom rejects a 16-row ECC batch before or after programming earlier rows (bounds I2's severity); the bootrom's high-`s` ECDSA policy (open since round 0); picotool's two-device refusal (asserted round 1, never re-verified); whether TinyGo's `pico2` target exposes any picotool reset interface (M2 assumes not); and silicon-stepping equivalence between a retail Pico 2 and the SH2's RP2350. No hardware exists yet; nothing here was executed.

---

**Verdict: not ready to execute — C1 (no post-write verification mode; the last irreversible write is checked by a bare picotool call that passes while picotool is printing a WARNING, and `--sh2-precheck` refuses the device from step (d) onward) plus I1–I6 must land first; round 3's lesson repeated itself in the quickstart, which is the artifact that will actually be pasted from.**