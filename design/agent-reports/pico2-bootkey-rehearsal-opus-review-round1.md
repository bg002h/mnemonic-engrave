# Round-1 fold review — Pico 2 boot-key rehearsal

*Persisted verbatim. Opus 5, round 1, 2026-07-26. Scope-limited to "did the fold
fix each round-0 finding, and did it introduce a new defect" — not a fresh audit.
Verdict: 0 Critical / 4 Important — RED.*

---

Verified the fold against picotool 2.2.0-a4's own OTP table (`share/picotool/rp2350_otp_contents.json`), its `otp get`/`seal`/`otp load` help, the binary's format/warning strings, `cmd/picosign/main.go`, and empirical bash/openssl tests.

**F1: FIXED** — `otp_field` reads the `field … = <hex>` line (correct per picotool's format strings, and `-n` = `--no-descriptions` exists and suppresses only descriptions), and phase 2 now asserts `16#$SB & 1 == 1` / `16#$KV == 1` with `die`; avoiding `VALUE` is also right, since `BOOT_FLAGS1` genuinely carries `KEY_INVALID` in the same row.

**F2: FIXED (structurally)** — the reject→burn→accept A/B is correct and phase 5 re-flashes the *same file* phase 3 produced; but the new phase 3 carries NEW-1 and NEW-2 below.

**F3: FIXED** — `read_slot` reads all 16 rows fail-closed, byte-swaps, and `verify_slot_or_die` compares against `key_hash()` before the SET-VALID confirm in both 1c and 4c; the runbook now points at the same code instead of asking for eyeballing.

**F4: FIXED** — CHIPID0-3 are `ecc=true, redundancy=1` in picotool's table, so `read_row -e` is the correct read; `require_board` re-checks the pin in phases 1-6 and the slot-0 tripwire is independent. `--ser` is unused, but with two boards in BOOTSEL picotool refuses to act at all, and a swap fails the CHIPID compare before any write.

**F5: FIXED** — `make_otp_json` scripts the jq reduction and asserts single-key / correct slot / 32 entries / no `crit1|boot_flags1` / byte-equal to the openssl hash before `otp load`. The final assertion also happens to close the `picotool seal` "will edit existing file if it exists" hazard on a stale `.seal-raw.json`.

**F6: FIXED** — page-lock reads with a die/STOP on non-zero in both phase 2 and runbook step 1, both explicitly labelled necessary-not-sufficient, and open item 4 corrected to name phase 4 as the first genuine sealed write. (Minor below re: `-e` on these rows.)

**F7: FIXED** — runbook open item 5 records `signature: verified` + `sha256(embedded uncompressed pubkey) == c8314536…319a473b` for v1.4.3, and correctly notes the fork's own zeroed signature is not counter-evidence.

**F8: FIXED** — `realpath` on both args; the seal branch now fires only on `missing SIGNATURE section|missing HASH_DEF item` and every other stderr dies with the image untouched; `BLOCKS -eq 2` and `signature: *verified` are asserted; `WORKDIR` is repo-anchored and the blinky `-o` is absolute.

**F9: FIXED** — no `placeholder.elf` remains; `build_blinky` runs before `picotool seal` in `make_otp_json` for both phases, and runbook step 2 seals the real `seedhammerii-<version>.uf2`.

**F10: FIXED** — `|| echo 0x0` is gone; `read_row` dies on a failed read, an unparseable `VALUE`, or a picotool WARNING (both `ECC IS INVALID` and `REDUNDANT ROWS AREN'T EQUAL` strings are caught), and all 16 rows of all four slots are checked.

**F11: PARTIAL — blocking.** (a) is covered by the F6 lock reads and (e) is documented in the script header/prerequisites (though phase 0 still cannot detect a Pico 2 W); but (b) host-vs-on-device sealing, (c) the raw redundant-row readback of 0x040-0x047 / 0x04b-0x04d, and (d) the "sign and flash the real 2.4 MB fork UF2" phase 4b are absent from the script, the runbook, *and* `design/FOLLOWUPS.md`. (d) is the substantive one: boot acceptance is still only ever demonstrated with a tiny TinyGo image. Either add 4b or file (b)/(c)/(d) against the hardware-bringup phase.

**F12: FIXED** — `WORKDIR` anchored to `REPO_ROOT`, keys generated only in phase 0, and phase 4 dies on a missing `my-key.pem` instead of regenerating.

**M3: FIXED** — `picosign extract` prints `%x` of the 64-byte sig and `openssl asn1parse` prints each INTEGER as exactly 64 hex digits (verified over 12 signatures incl. both 70- and 72-byte DER), so the comparison genuinely fires; picosign's DER path `FillBytes`es both halves, so a byte mismatch would be caught.

---

## New defects introduced by the fold

**NEW-1 — IMPORTANT — `scripts/pico2-bootkey-rehearsal.sh:406`.** Phase 3b runs `picosign sign -clear` on `blinky-unsigned.uf2`, a plain copy of the TinyGo build. `cmd/picosign/main.go:187` returns `missing SIGNATURE section` when `finfo.SignatureOffset == 0`, and the fold's own runbook (`design/RUNBOOK_custom_boot_key.md:219-220`) records that a freshly built blinky has no SIGNATURE section (it took `sign-firmware.sh`'s seal path). So under `--execute` phase 3 aborts at 3b after 3a has already passed — the negative control cannot be completed as written. The `-clear` is also unnecessary: the raw blinky is already unsigned.

**NEW-2 — IMPORTANT — `scripts/pico2-bootkey-rehearsal.sh:276-285`, consumed at `:398`, `:407`.** `flash_and_ask` is invoked as an `if` condition, which disables `set -e` for its whole body, so a failing `run picotool load --verify` (wrong/absent board, bad image, USB hiccup) falls through to the reboot and the prompt; the operator sees no blink — because nothing was flashed — answers `n`, and phase 3 prints `PASS: your-key-signed image was rejected, as it must be`. Verified empirically. This is the F2/F10 false-PASS class reintroduced on the very control that makes phases 5/6 meaningful; `flash_and_ask` needs to `die` on a load/reboot failure rather than fold it into the blink verdict.

**NEW-3 — IMPORTANT — `scripts/pico2-bootkey-rehearsal.sh:280`.** In dry-run `flash_and_ask` returns 0, i.e. "it blinked". So `--phase 3` (the default, unarmed mode) dies with `REJECTION FAILED … Secure boot is not being enforced. Do not continue, and do not trust phase 1's seal.`, while `--phase 5`/`--phase 6` print `PASS: ACCEPTED after the key burn` and `PASS: The signing chain and the OTP procedure are both proven on real silicon` having done nothing. The round-0 verdict's own precondition — "re-run the fixed script end-to-end in dry-run" — therefore produces one false FAIL and two false proofs. Dry-run should skip the verdict block, not answer it.

## Minor / Nit (non-blocking)

- `otp_field` (`:128-138`) omits `read_row`'s WARNING check, so `(WARNING - REDUNDANT ROWS AREN'T EQUAL)` on `CRIT1` (RBIT-8) or `BOOT_FLAGS1` (RBIT-3) is parsed silently — phase 2's seal assertion and phase 4d's `KEY_VALID == 0x3` can pass on an inconsistent redundant register.
- `read_row` uses `-e` on `PAGE{1,2}_LOCK{0,1}`, which picotool's table marks `ecc=false, redundancy=1, mask=0xffff7f/0xffff3f`; an actually-locked page will decode ECC-invalid, so `check_page_locks` dies with "picotool reported a warning" instead of "OTP pages are LOCKED", and bits 23:16 (the third lock copy) are masked away. Fails closed, wrong diagnosis. Same in `design/RUNBOOK_custom_boot_key.md:96-99`.
- `:339` — after an abort at phase 1's SET-VALID confirm, re-running phase 1 is impossible: `assert_stock_or_die` reports "slot 0 is not empty … Get a FRESH board" for a board that is merely half-way through.
- `scripts/sign-firmware.sh:131-142` — the M3 comparison silently degrades to an `info` line when either r or s has a zero high byte (asn1parse strips it, <64 digits); ~1 in 128 signatures.
- Runbook step 2's `jq -e 'has("crit1") or has("boot_flags1")' … && echo "REFUSE: would seal early"` only prints; the next pasted line loads regardless. Harmless only because the preceding `jq` cannot emit those keys.

## Verdict

**0 Critical / 4 Important — RED.** F1, F3-F10, F12 and M3 are correctly and verifiably fixed; F2's structure is right but its new phase 3 cannot run to completion (NEW-1) and can false-PASS (NEW-2); dry-run inverts three verdicts (NEW-3); F11 is only partly folded and its remainder is untracked.
