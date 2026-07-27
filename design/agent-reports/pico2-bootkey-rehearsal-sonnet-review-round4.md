# Round-4 fold verification — commit 642b7a3

*Persisted verbatim. Sonnet, round 4, 2026-07-26. Scope-limited to fold-verification
of the round-3 final-gate findings. Verdict: **0C/0I — GREEN**.*

---

Verified by reading the full diff (`git diff 642b7a3~1 642b7a3`) against the round-3 agenda, grepping the whole script for write-capable picotool subcommands, and a `bash -n` syntax check.

**C1** — FIXED. `--sh2-precheck`/`--sh2-verify-slot`/`--make-otp-json` reach only `sh2_require_seedhammer`, `otp_field`, `read_slot`, `check_page_locks`, `read_row_raw24`, `reject_rehearsal_key`, `verify_slot_or_die`, and (make-otp-json only) `build_blinky`+`picotool seal` (a local file transform, no device write). Grepped the whole file for `otp load`/`otp set`/`picotool load`: all three hits are inside phase 1/4, gated by `require_board`'s tripwire against SeedHammer's key — unreachable from any SH2 mode. All three mode arms `exit 0` before the `case "$PHASE"` dispatch, so there's no fallthrough into phase logic.

**I1** — FIXED. Runbook opens with the bash/fish preamble; verified `bash -n` passes on the script (irrelevant to this but confirms no syntax regression).

**I2** — FIXED. `sh2_require_seedhammer` pins CHIPID to `$WORKDIR/sh2-chipid.txt` on first call and re-checks it on every subsequent call (separate file from the rehearsal's `board-chipid.txt`, so no cross-contamination); re-confirms slot 0 == `SH_SIGNKEY_HASH` every time. A consumed rehearsal Pico's slot 0 holds `key_hash(factory-key.pem)`, never the real constant, so it fails this check immediately.

**I3** — FIXED. Grepped the runbook: the only remaining `-e` read is unrelated (`KEY_VALID` confirmation, not a page-lock row). Page-lock verification is now exclusively via `--sh2-precheck` → `check_page_locks` → `read_row_raw24` (no `-e`, full 24 bits).

**I4** — FIXED. Phase 6 now asserts `SECURE_BOOT_ENABLE&1==1` and `KEY_VALID==3` before proceeding, mirroring phases 4/5's precondition style.

**I5** — FIXED. Step 6 gained the do-NOT-burn-another-slot box; Recovery's "wrong hash burned" bullet is now explicitly scoped to an actual `--sh2-verify-slot` failure. No contradiction with step 3's own "if it mismatches, STOP → move to slot 2" text.

**I6** — FIXED. `ask_blink` loops until literal y/n (verified: `read -r a || die ...` then a `case` that only returns on exact matches, reprompting on `*`). 5b's verdict now comes from `bootsel_present` (`sleep 3; picotool info`), read under `if`, which is exempt from `set -e` — no unintended exit on a "device gone" (accepted) result.

**I7** — FIXED. `reject_rehearsal_key` hashes the supplied key and compares against `factory-key.pem`/`my-key.pem`/`third-party-key.pem` in `$WORKDIR`; called from both `--make-otp-json` and `--sh2-verify-slot` before any use. Runbook renamed the real key `sh2-boot-key.pem` and uses `$PWD`-absolute paths throughout steps 2/4/5/6.

**M1** — FIXED (reordered as suggested; the build-firmware call is duplicated in step 4 but that's harmless/idempotent and was the explicitly offered fix).
**M2** — FIXED (`$FW` replaced with `$PWD/seedhammerii-<version>.uf2`).
**M3** — FIXED (`KEY_INVALID` revoke command described, not printed, in step 7).
**M5** — FIXED (`--sh2-precheck` performs the raw 8-way CRIT1 / 3-way BOOT_FLAGS1 readback via `read_row_raw24`; `FOLLOWUPS.md`'s `bootkey-rehearsal-fidelity-residue` entry correctly marks (c) resolved, (b)/(e) still open and correctly described as such).
M4/M6 — correctly left untouched (accepted-as-is; diff confirms no changes to phase 1 resume logic or phase 3/4 ordering).

**Look-hardest items:** (1) confirmed no write path from any SH2 mode. (2) Runbook steps 1–3 are executable end to end: `REPO_ROOT`/`SEEDHAMMER_DIR` resolve via the script's own `$0`-relative `dirname`, not cwd, so the `../mnemonic-engrave/...` relative invocations work from `/scratch/code/shibboleth/seedhammer`; argument names match the parser exactly. (3) `set -e`/`pipefail` interactions in `bootsel_present`/`ask_blink`/`sh2_require_seedhammer` are all inside `if`/`case`/`||`-guarded contexts — no unintended early exit. (4) `--sh2-verify-slot` cannot pass without reading all 16 rows: `read_slot` loops i=0..15, `read_row` dies on any unparseable/WARNING row, and the final 64-hex-char compare is exact.

**New defect (non-blocking, Nit):** `design/RUNBOOK_custom_boot_key.md` Step 3's command block has no `cd` and relies on cwd carried over from Step 2 (unlike steps 4/5, which redundantly re-`cd`). If run out of session continuity the relative script path fails loudly ("No such file or directory") — fails safe, not silent, so not a scored finding.

**0C/0I (GREEN)**
