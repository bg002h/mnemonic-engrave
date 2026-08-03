# Post-hardware-contact review — pico2-bootkey-rehearsal (commit `cde9a5a`)

*Persisted verbatim. Fable, round 6, 2026-08-03. Called after first contact with
real RP2350 silicon surfaced three bugs in the reader/guard layer.
Verdict: **GREEN to burn the Pico (phase 1)**; NOT yet green for the SeedHammer
write. 3 Important / 5 Minor-Nit.*

---

All verification is done. I have read the full script, the fix commit, the picotool 2.2.0-a4 source (cloned at the exact tag, commit `25aa087`), its OTP table, the fork's OTP driver, both operator documents, the e2e harness, and the prior round-5 lens reports; I also ran live bash experiments reproducing the subshell semantics in question.

## Verification of the three fixes

**`check_page_locks` is correct. Verified against picotool's own table and picotool source.**

- Field masks (`OTP_DATA_PAGE1_LOCK1`/`PAGE2_LOCK1`): `LOCK_S` mask 0x3, `LOCK_NS` mask 0xC, `LOCK_BL` mask 0x30, `R1` bits 15:8, `R2` bits 23:16. The script's decode (`maj&0x3`, `(maj>>2)&0x3`, `(maj>>4)&0x3`) matches exactly.
- The 3-way bitwise majority vote matches the encoding the table specifies, so a disagreeing-copies device is judged the way the chip will actually behave.
- Gating only on `LOCK_S==0` and `LOCK_BL==0` is correct and complete: `LOCK_S=1` = "readable by Secure software but can not be written"; `LOCK_BL` is the field bootloaders honor for user writes. `LOCK_NS` correctly does not gate — retail `0x04` (`LOCK_NS=1`) is precisely NS-read-only. Not permissive-wrong, not strict-wrong.
- Page coverage complete: BOOTKEY0-3 occupy rows 0x080-0x0bf (page 2); CRIT1/BOOT_FLAGS1 in page 1. Both checked.
- Reading lock rows without `-e` at 24 bits is right (`ecc=false, redundancy=1`); no warning is possible on that path.

**Bug 2's real mechanism is now known, and it vindicates the batch fix.** picotool `main.cpp:7641`: `uint32_t last_reg_row = 1; // invalid` — the sentinel is row 0x001, which IS CHIPID1. Any query whose *first* printed row is row 1 skips the whole ROW/VALUE block. Not a general "sequence member" failure; exactly one row, when it comes first. Batched `CHIPID0..3` can never present row 1 first, and the count assertion catches recurrence.

**Output ordering: safe for every current call site, but the stated contract is false.** `filter_otp` returns `std::map<std::pair<row,mask>, otp_match>` — output is always ascending row order regardless of request order, with silent dedup. All three callers request distinct ascending rows, so Nth-VALUE-to-Nth-selector holds, and is empirically proven on the SH2. But the comment describes a contract picotool does not have.

## Findings

**Important — `:848, 862, 928, 1010` (with `:457`) — bug class 3 is still open in `ask_blink`: its `die` is reachable only inside a `case "$(...)"` word, and the case has no `*)` default, so a dead verdict is silently skipped and the phase ends green.**
Reproduced empirically. Run `--phase 3 --execute` with stdin closed or exhausted. `read -r` hits EOF, `die` exits only the subshell, the case word is empty, **no pattern matches, execution falls through both 3a and 3b verdicts**, and the phase prints the `ok_done` "PASS: prove the sealed board rejects untrusted images…" banner having proven nothing. A false negative-control green — the exact proof structure the A/B depends on. The e2e harness feeds finite heredoc input and would also sail through this hole rather than catch it.
Fix: have `ask_blink` set a global and call it as a plain command; add `*) die` defaults to all five case statements.

**Important — `:639-646` — the sh2-precheck's redundant-row readback asserts nothing, continues past a failed read, and states a false assurance.**
(a) `printf ... "$(read_row_raw24 "$r")"` — a `die` inside a command substitution used as a printf *argument* does not stop the script (reproduced: `CRIT1 copy 0x041 = 0x` then continues to the green RESULT banner). (b) The claim "picotool prints a WARNING on any disagreement" is wrong: raw row-number reads resolve with `m.reg == nullptr`, take the no-redundancy path, and can never print a redundancy WARNING. The operator is invited not to compare 11 values the script could compare itself. On the observed device all copies agree, so this did not bite today.
Fix: read the copies into variables via plain assignments (set -e then closes the die path), assert all-equal within each group, delete the false sentence. `--sh2-verify-valid` already does exactly this for BOOT_FLAGS1.

**Important — `design/RUNBOOK_custom_boot_key.md:149-152` and `FIRMWARE-QUICKSTART.txt:201-203` — both operator documents still teach the pre-`cde9a5a` gate: "all four page-lock rows zero … Non-zero = this procedure is impossible on this device."**
The operator cross-checks the script's PASS against the runbook, sees a direct contradiction on the retail device's actual value, and either "fixes" the script back toward the wrong gate or loses trust in the tool at the worst moment. The runbook currently declares the real SeedHammer II state disqualifying. Must land before the SH2 procedure; does not gate the Pico phases.

**Minor — `:207-233`** — `read_rows`'s positional mapping is only safe because all callers request distinct ascending rows; the comment states an order-preserving contract picotool does not have. A future caller passing rows out of order gets silently permuted values with a passing count check. Fix: parse the `ROW 0x%04x` lines alongside VALUE and verify the Nth row matches the Nth selector.

**Minor — `scripts/test/fake-picotool:54-60`** — the fake serves multi-selector reads as N independent single-row calls, so the e2e cannot reproduce the row-0x001 sentinel bug, the sorted output, or any batching behavior — the regression harness is blind to the exact bug class that just bit on hardware.

**Minor — `:361-366`** — the `PAGE*_LOCK0` gate is still a bare all-zero equality over structured fields. `NO_KEY_STATE=1` with `KEY_R=KEY_W=0` is harmless but would STOP the procedure. Failure direction is safe, but it is the same shape as the bug that nearly killed the project.

**Minor — `:191-205`** — `read_row` is now dead code. Fail-closed but single-row, i.e. the shape the project just learned to distrust. Remove it, or it becomes the helper a future edit reaches for.

**Nit — `:251-254`** — `key_hash` on an unreadable PEM silently yields sha256 of the empty string (`e3b0c4…`). Every current caller fails closed downstream, but it is a deterministic-wrong-value producer in a funds-adjacent path. (Also `misses` in `bootsel_present` is assigned and never read.)

Sweep results: every other `$(...)` call of a die-bearing function is a plain top-level assignment, and those DO halt under `set -e` (verified empirically). `sign-firmware.sh` and `run-e2e.sh` are clean of all three classes. `otp_field`'s WARNING gate is confirmed against picotool source.

## Question 3 — probability of another bug in phases 1-6

The three bugs were all in the read/parse layer, which has now met real silicon end-to-end (~70 rows, including a nonzero 16-row slot that reassembled to exactly `signKeyHash` — validating batching, ordering, byte order and ECC masking on real data). What has never met silicon: the write path (`otp load` of a 16-row ECC slot, `otp set -s` on an RBIT-3 row and a crit row), the flash/reboot path, and the bootrom accept/reject observation.

**Probability that phases 1-6 surface at least one more surprise: high — roughly 50-70%.** Most likely: **phase 1**, the first real `otp load`/`otp set` (picotool's own source still carries `// todo pre-check page lock` at write time). Second: **phase 5b's** USB-enumeration verdict, given this host has shown enumeration flakiness.

Does it fail safe? **Phase 1: yes, structurally.** Every write is followed by an independent read-back gate before the next irreversible step, the reader those gates use is the hardware-validated one, and the worst case is a consumed expendable board. **The probability of a surprise that produces a FALSE GREEN — the only outcome that matters for the SeedHammer — is low, single-digit percent**, concentrated in the `ask_blink` case hole and 5b's fail-open.

## Question 4 — safe to proceed to `--phase 1 --execute` on the Pico?

**Yes — proceed.** Phase 1 is the correct next step precisely because it exercises the never-tested write path where the next bug most plausibly lives, on hardware budgeted to be consumed. Conditions:

1. Run interactively from a real terminal (the `ask_blink` hole is unreachable with live stdin; `confirm`'s `read` dies on EOF under set -e).
2. Fix the `ask_blink`/case-default finding **before relying on phase 3's verdict**.
3. Fix the precheck redundancy assertion and the two stale documents **before the SH2 procedure**.
4. Do not treat a green rehearsal as licensing the SH2 write until docs and tool agree — the gate discipline that caught bug 1 only works if the checklist and the tool agree.

## Question 5 — observed device state

- **The USB white-label "discrepancy" is expected and dissolves on inspection.** `writeOTPValues()` (`platform_sh2.go:666-694`) writes the volume label ("SHII"), SCSI vendor/product, info_uf2 and redirect strings — it never writes `INDEX_USB_DEVICE_MANUFACTURER_STRDEF`/`PRODUCT_STRDEF`. A correctly provisioned retail unit still enumerates in BOOTSEL as "Raspberry Pi"/"RP2350 Boot". No plan change needed.
- **No OTP-write-at-boot hazard from the fork firmware:** `LockBoot()` is reachable only via the NFC `lock-boot` debug command, never on normal startup; `AddBootKey` returns the existing slot on exact match. Worth one runbook sentence: never send `lock-boot` to the unit.
- **`error -71`**: transient descriptor-read failure that recovered — normal USB noise, but direct evidence enumeration can glitch on this host, which is the failure mode 5b fails open on. Use a direct root port and short cable for verdict-bearing steps.
- The CHIPID/USB-serial cross-check is an independent confirmation of row order and byte order on real data — worth recording in the runbook.
- `CRIT1 = 0x000001` also confirms `DEBUG_DISABLE` is **not** set — one of the runbook's listed disqualifiers, now positively cleared.

## Could not be verified from the repo

Real-silicon behavior of `otp load` under interruption and of `otp set -s` on crit rows (phase 1 is the test); the bootrom's high-`s` policy (phase 5 is the test); whether the SH2's RP2350 stepping differs from a retail Pico 2 in any way that matters to the write path.

**Verdict: GREEN to burn the Pico (phase 1), with findings 1-3 owned before their respective gates — finding 1 (ask_blink) before phase 3's verdict is trusted, findings 2-3 (precheck assertion, docs) before the SeedHammer procedure. Not yet green for the SeedHammer write.**
