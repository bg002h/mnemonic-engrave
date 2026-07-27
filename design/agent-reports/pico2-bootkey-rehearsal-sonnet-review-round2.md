# Round-2 fold review — Pico 2 boot-key rehearsal

*Persisted verbatim. Sonnet, round 2, 2026-07-26. Scope-limited to fold-verification
of the round-1 findings. Verdict: **0C/0I — GREEN**. Closes the review loop.*

---

Confirmed clean. This matches Minor "runbook's `jq -e` guard now aborts instead of printing" exactly. All items verified against the diff, the round-1 review, and cross-checked against the fork (`cmd/controller/platform_sh2.go`, `driver/otp/*.go`) for the F11(d) BOOTSEL-fallback assumption.

## Verdict per item

**NEW-1: FIXED.** Phase 3b now does `flash_image "$WORKDIR/blinky-unsigned.uf2"` directly (script lines 460-470) — no more `picosign sign -clear` on a no-SIGNATURE-section image; the copy from `blinky.uf2` is used as-is.

**NEW-2: FIXED.** `flash_image()` (lines 302-315) dies explicitly via `|| die` on both `picotool load --verify` and `picotool reboot` failure — this doesn't even rely on `set -e` propagation, so a masked failure is structurally impossible. `ask_blink()` (317-323) only ever returns yes/no/skip and is called as a separate, later statement.

**NEW-3: FIXED.** `ask_blink` returns `'skip'` unconditionally when `EXECUTE != 1` (line 319); every verdict `case` in phases 3/3b/5/5b/6 has a `skip)` arm that only prints, never dies/oks.

**F11(d): FIXED.** Phase 5b (536-552) signs/flashes the real fork UF2 and gates on "did BOOTSEL reappear" as a negative acceptance signal. Cross-checked against the fork: `cmd/controller/platform_sh2.go`'s `LockBoot()` (line 510) only calls `otp.EnableSecureBoot()` + `otp.AddBootKey()` — it never touches any BOOTSEL-disable OTP bit, and the rehearsal script's phase 1/4 mirror exactly that (`CRIT1.SECURE_BOOT_ENABLE`, `BOOT_FLAGS1.KEY_VALID`). So nothing in this procedure suppresses the RP2350's normal reject→BOOTSEL fallback, making the negative signal sound. The `FW_REAL` lookup guards `ls`-no-match under `pipefail` with `|| true`; the not-found branch just warns rather than failing the phase.

**Minor — otp_field WARNING check: FIXED.** Present at lines 132-136, matches `read_row`'s pattern.

**Minor — page-lock rows via read_row_raw24: FIXED.** Reads without `-e`, masks to the full 24 bits (lines 222-231), used by `check_page_locks`. It omits the WARNING check that `otp_field`/`read_row` have, but that's correct, not a gap: per round-1's own finding text these rows are `redundancy=1`, so the "REDUNDANT ROWS AREN'T EQUAL" class is structurally impossible there, and the ECC-invalid class is already avoided by dropping `-e`.

**Minor — phase 1 resume: FIXED.** The resume check (382-392) requires an exact slot-0 hash match *and* `KEY_VALID==0`; anything else (mismatched hash, partial state) falls through to the strict `assert_stock_or_die`, which fails closed. Verification (`verify_slot_or_die`) and the `SET-VALID` confirm are never skipped by the resume branch — only the burn (1a/1b) is skipped.

**Minor — sign-firmware.sh DER left-pad: FIXED.** `printf '%064s' "$half" | tr ' ' '0'` (line 136) correctly left-pads a short (e.g. 62-digit) half to 64 hex digits with zeros. The `${#RS} -eq 128` gate downstream still protects against the (much rarer, ~1/256) opposite case of an oversized 33-byte component, falling back to the pre-existing "info" degradation rather than a false compare — not a regression.

**Minor — runbook `jq -e` guard: FIXED.** Commit `893d3f9` (the follow-up edit) replaced `&& echo "REFUSE..."` with `if jq -e ...; then echo ...; exit 1; fi`, which now genuinely aborts before `picotool otp load`.

## New defects found in this round

None.

## 0C/0I (GREEN)
