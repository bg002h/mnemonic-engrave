# SH2 final gate — operator-paste-path lens

*Persisted verbatim. 2026-08-03, final gate before the first irreversible OTP write on the real SeedHammer II. Combined verdict across the three lenses: 1 Critical / 8 Important. NOT safe to write until folded.*

---

## Summary

I walked the exact operator paste-path across `FIRMWARE-QUICKSTART.txt` section 3(a)-(g)/4 and the parallel steps in `design/RUNBOOK_custom_boot_key.md`, cross-checked every flag against `scripts/pico2-bootkey-rehearsal.sh`'s arg parser and `scripts/sign-firmware.sh`, and checked the build-artifact naming against `/scratch/code/shibboleth/seedhammer/flake.nix`.

**FIRMWARE-QUICKSTART.txt itself checks out clean** on every point I could verify: all `--sh2-precheck` / `--sh2-verify-slot` / `--sh2-verify-valid` / `--make-otp-json` invocations match the script's `case` parser exactly (`scripts/pico2-bootkey-rehearsal.sh:110-118`); the page-lock semantics text (`0x040404` expected, `LOCK_S`/`LOCK_BL` gate, `LOCK_NS` informational) matches `check_page_locks()` (`scripts/pico2-bootkey-rehearsal.sh:351-400`) verbatim; the `KEY_VALID` expected values (slot 1→3, 2→5, 3→9) match the `WANT=$(( 1 | (1 << SH2_SLOT) ))` check (`scripts/pico2-bootkey-rehearsal.sh:729`); section 4 correctly flashes `${FW%.uf2}.signed.uf2`, matching `sign-firmware.sh`'s actual output (`OUT="${IMG_IN%.uf2}.signed.uf2"`, `scripts/sign-firmware.sh:47`); and `signKeyHash` `c8314536…` matches `platform_sh2.go:70` exactly.

**The RUNBOOK, which sits open alongside it, has drifted** and I reported four findings (most severe first):

1. **Important** — RUNBOOK Step 6 (`design/RUNBOOK_custom_boot_key.md:358`) tells the operator to flash the *unsigned* build output, not the `.signed.uf2` Step 5 actually produces. This directly contradicts FIRMWARE-QUICKSTART.txt section 4 and `sign-firmware.sh`'s own printed "Flash with:" guidance — a genuine signed-artifact trap in the doc, at the last step of the procedure.
2. **Important** — RUNBOOK Step 4 (`design/RUNBOOK_custom_boot_key.md:302`) drops `env VERSION=$(git rev-parse HEAD)`, so per `flake.nix:96-109` the build falls back to a `git describe` filename that Step 5/6's hardcoded `seedhammerii-$(git rev-parse HEAD).uf2` won't match — a "no such file" right after the operator has already burned OTP.
3. **Important (PLAUSIBLE)** — Neither doc discloses that `--sh2-verify-slot`/`--sh2-verify-valid` — the read-only gates bracketing the two irreversible writes — are running their success path for the first time ever on this specific operator's machine; the runbook's phrasing (`design/RUNBOOK_custom_boot_key.md:168-170`) reads as broader reassurance than is earned, given all four hardware-found bugs today were in this same wrapper/parsing layer.
4. **Minor** — RUNBOOK's own "Open items" #1/#4 (`design/RUNBOOK_custom_boot_key.md:420,440`) are still marked unresolved despite `design/REHEARSAL_RESULT_2026-08-03.md` showing phases 0-6 passed today — stale but not command-blocking, and errs toward caution rather than false safety.

No script-internals were audited (per scope); all four findings are document-vs-tool or document-vs-document inconsistencies discoverable by reading the files, verified against `flake.nix` and the scripts as ground truth.