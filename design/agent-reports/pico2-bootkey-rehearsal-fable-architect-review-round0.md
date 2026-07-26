# Adversarial review — Pico 2 boot-key rehearsal plan

*Persisted verbatim. Fable architect, round 0, 2026-07-26. Verdict: NOT safe to
execute as written (4 Critical / 8 Important); approach sound. Reviewed against
`scripts/pico2-bootkey-rehearsal.sh`, `design/RUNBOOK_custom_boot_key.md`,
`scripts/sign-firmware.sh`, `scripts/rehearsal-blinky/main.go`.*

---

**Artifacts reviewed:** `/scratch/code/shibboleth/mnemonic-engrave/scripts/pico2-bootkey-rehearsal.sh`, `/scratch/code/shibboleth/mnemonic-engrave/design/RUNBOOK_custom_boot_key.md`, `/scratch/code/shibboleth/mnemonic-engrave/scripts/sign-firmware.sh`, `/scratch/code/shibboleth/mnemonic-engrave/scripts/rehearsal-blinky/main.go`, cross-checked against the fork source (`driver/otp/otp.go`, `cmd/controller/platform_sh2.go`, `gui/gui.go`, `cmd/picosign/main.go`) and against the **installed picotool 2.2.0-a4 binary and its upstream source**. I ran picotool offline where possible (seal, help, otp list, strings on the binary) and executed picosign against the real firmware image to test claims empirically rather than by reading.

---

## F1 — CRITICAL: Phase 2's sealing assertion is a tautology; it passes on a board that was never sealed

- **Location:** `scripts/pico2-bootkey-rehearsal.sh:193` (also `:195`)
- **Finding:** `echo "$sb" | grep -q '1'` matches the literal `1` in the echoed register name `CRIT1` (and `BOOT_FLAGS1`), so the check passes unconditionally regardless of the value.
- **Evidence:** picotool's `otp get` output for a field selector prints the row header including `": " << reg->name` (i.e. the string `CRIT1`) before the `VALUE 0x…` line — verified in the picotool source (`otp_get_command::execute`), and the same format strings are present in the installed 2.2.0-a4 binary. So `$sb` always contains a `1`.
- **Concrete failure scenario:** Phase 1's `CRIT1.SECURE_BOOT_ENABLE` write fails or is aborted mid-phase → board is not sealed → phase 2 prints "PASS: SECURE_BOOT_ENABLE is set" anyway → every later phase draws conclusions from an unsealed board (see F2). The `KEY_VALID` check on line 195 is non-fatal (`|| info`) and cannot fail the phase at all. Phase 2 — "the assertion that our read of the SeedHammer II is correct" — asserts essentially nothing.
- **Fix:** Reuse phase 0's `hexval` parser (which is the correct pattern, and which phase 2 inexplicably does not use), assert `$((v)) & 1` for SECURE_BOOT_ENABLE and `$((v)) -eq 1` for KEY_VALID, `die` otherwise.

## F2 — CRITICAL: No negative control anywhere; the rehearsal can prove the signing chain "works" on a board where secure boot never engaged

- **Location:** `scripts/pico2-bootkey-rehearsal.sh:236-283` (phases 4-5; the phase 5 header at `:265` calls itself a "negative control" but is a second *positive* control)
- **Finding:** The rehearsal never once demonstrates that the sealed board **rejects** anything — not an unsigned image, not a wrong-key image. "LED blinks" is only evidence of signature acceptance if rejection has been shown to be possible.
- **Concrete failure scenario:** Combine with F1: sealing silently didn't take → phase 2 false-passes → phase 4 blinks (an unsealed RP2350 boots *any* image) → phase 5 blinks → operator records a fully green rehearsal, "sign chain proven end-to-end," and proceeds to burn the SeedHammer II's OTP on the strength of a test that never tested signature enforcement. This is precisely the "false proof of safety" the rehearsal exists to prevent.
- **Fix:** Add a retryable phase (zero OTP cost): (a) flash the picosign-`-clear`ed (unsigned) blinky — require NO blink and require the board to fall back to BOOTSEL; (b) flash a blinky signed with a third, never-burned key — require the same. Only after rejection is demonstrated does a blink in phases 4/5 mean anything. Rename phase 5 to "fallback (positive) control."

## F3 — CRITICAL: "Verify all 16 rows before setting the valid bit" is implemented as an unchecked printout, and read failures are swallowed

- **Location:** `scripts/pico2-bootkey-rehearsal.sh:175-176` (phase 1), `:225-227` (phase 3); `design/RUNBOOK_custom_boot_key.md:130-138` (step 2)
- **Finding:** The "verification" is `for i in $(seq 0 15); do picotool otp get -e "BOOTKEY1_$i" || true; done` — no expected value is computed or even displayed, nothing is compared, `|| true` silently skips failed reads, and the script proceeds directly to the `SET-VALID` confirm. The runbook additionally asks the operator to eyeball 16 byte-swapped ("low byte first") 2-byte rows against a 64-char hash string they must derive themselves.
- **Concrete failure scenario (the real device):** operator misreads one nibble across 16 byte-swapped rows — or one row's read fails and prints nothing thanks to `|| true` — types `SET-VALID-SLOT1`, `KEY_VALID 0x2` is burned → the slot is permanently valid for a wrong hash, the fork never boots from it, and one of three spare slots is gone. This is the single most safety-critical verification in the entire procedure and it is currently theater; worse, the rehearsal *trains* the operator that scrolling hex past their eyes counts as verification.
- **Fix:** Automate it: the expected 32-byte hash is available machine-readably (it is the `bootkey0`/`bootkey1` array in the seal-generated otp.json — schema verified empirically, see F6 — or recompute as SHA-256 of the uncompressed 64-byte X‖Y pubkey). Read back all 16 rows, parse the `VALUE` fields, byte-swap, reassemble, string-compare, `die` on any mismatch or failed read. Gate the `SET-VALID` confirm behind that assertion passing. Use the identical script on the SH2.

## F4 — CRITICAL: No device-identity pinning — the destructive phases will happily burn OTP on the wrong RP2350, including the SeedHammer II

- **Location:** `scripts/pico2-bootkey-rehearsal.sh` (all phases; no `--ser`/`--bus` filters, no board-ID capture anywhere)
- **Finding:** Nothing binds phases 1/3 to the board that passed phase 0. The SH2 is *also* an RP2350 in BOOTSEL; picotool supports `--ser` device filtering (verified in `picotool help otp set`) and the script never uses it.
- **Concrete failure scenario:** During the multi-day rehearsal the operator also performs runbook step 1 on the SH2 (as instructed — it's read-only). The SH2 is left in BOOTSEL; the Pico gets unplugged. `--phase 3 --execute` now talks to the SH2: the rehearsal's *throwaway* `my-key.pem` hash is burned into the real device's slot 1 and `KEY_VALID 0x2` set — a real, irreversible boot slot consumed by a junk key, and permanent loss of the on-screen secure-boot attestation (`isSecureBootEnabled()` requires `nvalid == 1`, `platform_sh2.go:738`, and this is forever). Phase 0's stock-board assertions would catch the SH2; phase 3 has no check at all.
- **Fix:** Phase 0 records the board's unique ID/serial into `$WORKDIR`; every subsequent phase asserts the same serial and additionally asserts slot 0 does NOT hold `c8314536…319a473b` before any write; pass `--ser <serial>` on every destructive picotool invocation.

## F5 — IMPORTANT: The manual otp.json edit gating `otp load` is never verified, and an unedited phase-1 json seals the board before any verification

- **Location:** `scripts/pico2-bootkey-rehearsal.sh:169-173`, `:219-223` (info messages only — the script does not pause for, inspect, or validate the edit); `design/RUNBOOK_custom_boot_key.md:126-131`
- **Evidence (verified empirically):** I ran `picotool seal --sign` on a copy of the real firmware; the generated json is `{"boot_flags1":{"key_valid":1},"bootkey0":[32 bytes],"crit1":{"secure_boot_enable":1}}`. In picotool's `process_otp_json`, the renamed key `bootkey1` IS accepted (fuzzy sequence matching → `BOOTKEY1_0`, 16 ECC rows) — so the rename instruction is viable — and JSON keys are processed alphabetically (`boot_flags1`, `bootkey0`, `crit1`).
- **Concrete failure scenarios:** (a) Phase 1, operator types `BURN-FACTORY` without editing: the single `otp load` writes key hash AND `key_valid` AND `secure_boot_enable` in one shot — the board is sealed with **zero** row verification, destroying the verify-then-valid ordering the plan itself preaches; if the hash rows misburned, the Pico is bricked earlier than the plan intends. (b) Phase 3 / real step 2, unedited json: the `bootkey0` array is written onto SeedHammer's occupied slot-0 ECC rows. The installed picotool/bootrom refuses ECC-row modification ("Attempted to modify OTP ECC row(s)" — string confirmed in the 2.2.0-a4 binary), so this *most likely* aborts cleanly after the benign same-value `boot_flags1` write — but the last line of defense for the sole valid boot slot of a bitcoin device should not be a bootrom error path the plan never mentions, and the JSON field path has no clear-bits guard for raw rows at all.
- **Fix:** Script the transformation (`jq 'del(.boot_flags1,.crit1) | {bootkey1: .bootkey0}'`), then hard-assert before load: exactly one top-level key, correct slot name, 32 integers, equal to the independently computed hash; refuse any json still containing `crit1`/`boot_flags1`.

## F6 — IMPORTANT: Phase 2 does not test what the runbook claims it tests, and the load-bearing writability assumption remains untestable before the first real write

- **Location:** `scripts/pico2-bootkey-rehearsal.sh:197-204`; `design/RUNBOOK_custom_boot_key.md:244-265` (open item 4), `:85-105` (step 1)
- **Finding:** Open item 4 says "Rehearsal phase 2 is precisely this test" for "a sealed device still accepts OTP **writes**." Phase 2 performs `picotool otp get -e BOOTKEY1_0` — a **read**. The first genuine sealed-board write is phase 3's own irreversible one. Worse, on the real SH2, runbook step 1's STOP condition "any OTP write returns not permitted" can never trigger because step 1 issues no writes, and factory page locks are invisible to the four reads it does perform. Note picotool's own source contains `// todo pre-check page lock` — `otp load`/`otp set` discover locks only at write time.
- **Concrete failure scenario:** SeedHammer's *factory provisioning* (which is not necessarily the published `lock-boot` code path — see F11a) locked OTP page 2. Step 1 passes cleanly, operator proceeds to step 2, and the first indication is a failed (or worse, partial) write during the irreversible step itself.
- **Fix:** (a) Correct the runbook claim. (b) Make the check real and read-only: read `PAGE1_LOCK0/1` (rows 0xf82/0xf83 — covers CRIT1/BOOT_FLAGS1) and `PAGE2_LOCK0/1` (0xf84/0xf85 — covers BOOTKEY0-3). I verified these selectors exist in the installed picotool's `otp list`. Add them to rehearsal phase 2 AND runbook step 1 with a STOP-if-nonzero condition. (c) Optionally add an `--execute` write probe to one sacrificial unused user row on the Pico to confirm the write path end-to-end.

## F7 — IMPORTANT: The "official firmware fallback" — the plan's only recovery path — is never verified against a real artifact

- **Location:** `design/RUNBOOK_custom_boot_key.md:225-243` (step 7 + Recovery); `scripts/pico2-bootkey-rehearsal.sh:264-283`
- **Finding:** Phase 5 proves the two-valid-slots *mechanism* with a stand-in key. Nobody has checked that SeedHammer's **published release UF2s** actually contain a valid signature by the slot-0 key. This is not hypothetical: the fork's own reproducible build ships with a **zeroed** signature (`picosign sign -clear`) — I confirmed `picotool info -a` on `seedhammerii-66d3121…uf2` reports `signature: incorrect`, value all-zeros. If official releases are likewise distributed unsigned (signed only at factory flash time), then "leave slot 0 valid and you can always fall back to official firmware" is false, and after any bad fork build the device has no recovery path at all.
- **Fix (offline, do before boards arrive):** Download an official `seedhammerii-vX.Y.Z.uf2`, require `picotool info -a` → `signature: verified`, and hash the embedded 64-byte pubkey (via `picosign extract`/`info`), requiring SHA-256 == `c8314536…319a473b` (`platform_sh2.go:70`). If this fails, the recovery section and step 7 must be rewritten before any SH2 OTP write.

## F8 — IMPORTANT: Phase 4 cannot run with default settings, and sign-firmware.sh breaks on relative paths with an ugly side effect

- **Location:** `scripts/pico2-bootkey-rehearsal.sh:37` (`WORKDIR=./rehearsal-work`, cwd-relative), `:244-249`; `scripts/sign-firmware.sh:51`, `:67-76`, `:115-117`
- **Finding:** (a) Phase 4 builds via `sh -c "cd '$REPO_ROOT/scripts/rehearsal-blinky' && tinygo build -o '$FW' …"` with `FW=./rehearsal-work/blinky.uf2` — the relative `-o` resolves under `rehearsal-blinky/`, whose `rehearsal-work/` doesn't exist; the build fails or lands in the wrong place, then the caller's `[ -f "$FW" ]` dies. (b) `sign-firmware.sh`'s `picosign()` runs after `cd "$SEEDHAMMER_DIR"`, so any relative image path fails — **empirically confirmed**: `picosign hash sealtest/fw.uf2` from the fork dir → `no such file or directory`. (c) Step 1 treats *any* `picosign hash` failure (path error, `go run` module fetch failure) as "no SIGNATURE section" (`2>/dev/null`), then re-seals the input **in place** with a throwaway key — producing exactly the 3-metadata-block double-sealed state the script warns about. (d) The only guard against that state, `BLOCKS`, is computed at `sign-firmware.sh:115` and **never compared to anything**; `signature: verified` is likewise never asserted (only `head -20` shown).
- **Concrete failure scenario:** Operator follows runbook step 5 verbatim (`…/sign-firmware.sh seedhammerii-<version>.uf2 my-key.pem` — relative path, `design/RUNBOOK_custom_boot_key.md:169-171`) → the build artifact is silently rewritten as throwaway-sealed, script dies at `picosign sign -clear`; a rerun then "succeeds" through steps 2-6 on a triple-block image whose block the bootrom verifies may not be the one the offline checks covered → refused boot on the SH2, recovery exercise under stress (and only survivable if F7 holds).
- **Fix:** `IMG=$(realpath "$1")`, `KEY=$(realpath "$2")` at entry; distinguish "missing SIGNATURE section" from other picosign errors instead of discarding stderr; assert `BLOCKS -eq 2` and grep-assert `signature: verified`; write the sealed recovery copy to a new file rather than mutating the input. In the rehearsal script, anchor `WORKDIR` to `$REPO_ROOT` (see also F10).

## F9 — IMPORTANT: `placeholder.elf` does not exist and cannot be an arbitrary file — the destructive phases are not executable as written

- **Location:** `scripts/pico2-bootkey-rehearsal.sh:166`, `:216`; `design/RUNBOOK_custom_boot_key.md:122`
- **Finding:** No `placeholder.elf` exists anywhere in either repo, and it cannot be a dummy: I verified `picotool seal` fails with `No metadata block found` on a plain binary (and `Can only sign to same file type` across types). The seal input must be a real RP2350 image.
- **Concrete failure scenario:** Phase 1 `--execute` dies mid-destructive-phase at the seal step; the operator improvises an input image and command line while armed and adjacent to OTP writes — improvisation inside a destructive phase is exactly the behavior this script exists to eliminate, and (on the real device in step 2) a mid-procedure improvised seal with the wrong key file yields a wrong otp.json one confirm away from a dead slot.
- **Fix:** Build the blinky first (move the build into phase 0 or a new prep phase) and use `blinky.uf2` (uf2→uf2) as the seal input in phases 1 and 3; in runbook step 2 use the built `seedhammerii-<version>.uf2`. Additionally assert the json's key array equals the SHA-256 of the key's uncompressed pubkey computed independently with openssl.

## F10 — IMPORTANT: Phase 0's stock-board assertion converts read failures into PASSes and checks 1 of 16 rows per slot

- **Location:** `scripts/pico2-bootkey-rehearsal.sh:144` (`|| echo 0x0`), `:143-147`
- **Finding:** A failed `picotool otp get -e BOOTKEY*_0` is substituted with `0x0` and asserted as "slot is empty" — a textbook false PASS, directly contradicting the fail-closed probe discipline at `:115-118`. Also only row `_0` of each 16-row slot is checked, so a partially-burned slot passes as empty.
- **Concrete failure scenario:** A USB hiccup or field-name drift makes the four reads fail → "PASS: all four boot-key slots are empty" on a previously-used board → the whole rehearsal runs from a wrong starting state and its conclusions are garbage.
- **Fix:** Remove `|| echo 0x0` (die on read failure); read all 16 rows per slot, or at minimum rows 0, 7, 15.

## F11 — IMPORTANT: Material fidelity gaps between the rehearsed condition and the retail SH2

- **Location:** `scripts/pico2-bootkey-rehearsal.sh:153-183` (phase 1) vs `cmd/controller/platform_sh2.go:510-518, 664-694`
- **Findings, each with what it could hide:**
  - (a) **Provenance of the seal.** The retail unit was sealed by whatever firmware/factory jig SeedHammer actually ran — not necessarily the published `lock-boot` path that was code-read. Page locks, `BOOT_FLAGS0` PICOBOOT-disable bits, or `KEY_INVALID` set by unpublished provisioning are invisible to the rehearsal and undetectable by the current step 1 reads. This is the one class of failure the Pico **cannot** de-risk; only the F6 read-only lock checks on the SH2 itself can.
  - (b) **Host-side vs on-device sealing.** Phase 1 seals via picotool JSON; the SH2 was sealed by on-device bootrom calls that also populated white-label strings and `USB_BOOT_FLAGS`. The rehearsal never writes OTP on a board with a populated user area. Low risk, but "the Pico accepted phase 3" does not fully entail "the SH2 will."
  - (c) **Redundant-row semantics.** CRIT1 is 8-way, BOOT_FLAGS1 3-way redundant (`otp.go:93-95`, `readOrRow(...,3)`); picotool's `otp set -s` writes `reg->redundancy` copies with set-bits semantics (verified in source — the runbook's `-s` claim is correct). Confirm on the Pico by raw-reading rows 0x040-0x047 and 0x04b-0x04d after phases 1/3.
  - (d) **Blinky is not the real image.** Boot acceptance is only ever tested with a tiny TinyGo image; the real 2.4 MB firmware has a different block structure (two metadata blocks as built — verified). Add a phase 4b: sign the actual fork UF2 with `my-key.pem`, flash to the Pico; acceptance signal = the board does **not** fall back to BOOTSEL after reboot (a rejected secure-boot image returns to BOOTSEL). It will then hang on missing hardware, which is fine and expected.
  - (e) The Pico 2 W spare gives no LED signal (correctly documented) but phase 0 does not detect a W → on the spare, "no blink" would be misdiagnosed as signature rejection.

## F12 — IMPORTANT: Cross-phase state lives in a cwd-relative WORKDIR with silent key regeneration inside a destructive phase

- **Location:** `scripts/pico2-bootkey-rehearsal.sh:37`, `:159-162`, `:211-214`
- **Concrete failure scenario:** Day 2, operator runs phase 3 again from a different directory: `./rehearsal-work` is empty there, so the script **silently generates a fresh `my-key.pem`** and burns the new hash — a second spare slot consumed, and two keys now in circulation that the operator will conflate (phase 4 then signs with whichever key the current cwd yields, and a non-blinking LED gets misdiagnosed as a broken sign chain).
- **Fix:** Anchor `WORKDIR` to `$REPO_ROOT`; never generate keys inside a destructive phase (require the file to pre-exist, generated in a dedicated safe step); record key fingerprints in phase 0/3 and assert them in phases 4/5.

## Minor findings

- **M1** `scripts/pico2-bootkey-rehearsal.sh:32-33` — the usage header shows phases 4/5 without `--execute`, but every meaningful command there is behind `run`; following the header verbatim does nothing real. `ok_done`'s dry-run wording keeps it honest, but fix the header.
- **M2** `scripts/pico2-bookey-rehearsal.sh:88-89` — prerequisites check only picotool/openssl; phases 4/5 also need `tinygo` and `go`.
- **M3** `scripts/sign-firmware.sh:107-111` — step 6 verifies the `sig.der` *file*, never the signature actually embedded in the image; a DER→raw conversion bug (r/s order, `FillBytes`) passes every offline check. `picosign extract` exists — compare it to the converted raw signature. Related open question I could not verify: whether the RP2350 bootrom rejects high-s ECDSA signatures (openssl does not normalize s). If phase 4 ever fails to blink, re-sign once (new nonce) before concluding the chain is broken, and note an intermittent pattern would point here.
- **M4** `scripts/pico2-bootkey-rehearsal.sh:231-232` — "Expect 0x3" after phase 3 is printed, not asserted (same fix as F1), and the `otp get` runs unguarded even in dry-run.
- **M5** `design/RUNBOOK_custom_boot_key.md:110-116` — the real `my-key.pem`, which gates firmware for a bitcoin-handling device, is generated unencrypted; use `-aes256` or state handling expectations.
- **M6** `design/RUNBOOK_custom_boot_key.md:132` — literal `...` inside a paste-executable command block (`BOOTKEY1_0 BOOTKEY1_1 ...`); write out all 16 selectors.

## External-facts audit

Verified against source or the installed tool: 4 boot-key slots and `KEY_VALID` bits 0-3 / `KEY_INVALID` bits 8-11 / `BOOTKEY0_0` at 0x080 (picotool `otp list` + `otp.go`); `otp set -s` = set-bits/OR with per-field redundancy (source + help); seal-generated otp.json schema and the `bootkey0→bootkey1` rename being accepted; `signKeyHash` constant, `LockBoot` flow, `driver/otp` never touching page locks / `DEBUG_DISABLE` / `KEY_INVALID` / `BOOT_FLAGS0`; the `(UNLOCKED)` suffix mechanics and that `FeatureSecureBoot` is cosmetic-only (single consumer at `gui/gui.go:2717`); re-triggering the NFC `lock-boot` after adding a second key is benign (exact-match early return in `AddBootKey`, idempotent white-label/CRIT1 writes); "no debug probe needed" is consistent with the code (all operations are PICOBOOT/USB) **contingent on retail behavior matching the published tree**. Unverifiable from the repo: actual retail SH2 OTP/lock state (mitigate via F6), whether official releases are signed (F7 — checkable offline now), bootrom low-s policy (M3). picotool behavior was verified against the develop-branch source cross-checked with strings in the shipped 2.2.0-a4 binary; minor line-level drift is possible.

---

## Verdict

**NOT safe to execute as written — but the approach is right, not misguided.** Rehearsing the irreversible chain on disposable identical silicon is exactly the correct de-risking strategy, and several of the plan's core claims held up under source verification (`otp set -s` semantics, slot arithmetic, the dual-key/`(UNLOCKED)` behavior, the JSON rename). But in its current state the rehearsal can return a fully green result without ever having engaged secure boot (F1 + F2), performs its most safety-critical check as an unverified printout (F3), has no defense against burning the wrong physical device — including the SeedHammer II itself (F4) — and cannot actually be run to completion (F8, F9).

Before the boards arrive: fix F1-F5 and F8-F10 in the script, do the F7 official-artifact signature check offline (it requires no hardware and could invalidate the entire recovery story), extend runbook step 1 with the PAGE1/PAGE2 lock reads (F6), correct open item 4's false claim about phase 2, and re-run the fixed script end-to-end in dry-run. Only then execute phases 0→5 — including the new negative-control phase — on the plain Pico 2.
