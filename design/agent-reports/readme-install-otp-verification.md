# README install/OTP-burn section — verification and completion

Scope: `/scratch/code/shibboleth/seedhammer/README.md`, the `### Installing
*this fork*` subsection added in commit `310cc78` (flagged DRAFT/unverified),
plus the `## About this fork` feature list. Verified against
`docs/custom-firmware.md` (authoritative, 470 lines), the real scripts in
`mnemonic-engrave/scripts/`, `flake.nix`, and `git log`.

## VERDICT

23 machine-checkable claims checked. 0 were factually wrong. 2 pre-existing
non-DRAFT issues found and fixed while in the file: a stale feature count in
the "About this fork" intro sentence, and one missing shipped feature (BIP-39
passphrase support) omitted from that list.

## CLAIMS CHECKED

| Claim | How verified | Result | What I changed |
|---|---|---|---|
| `--phase 0..6` are real, spelled correctly | `Read scripts/pico2-bootkey-rehearsal.sh` lines 108-121 (arg parser) and 847-1172 (case "$PHASE" in 0)..6)) | PASS | none |
| `--execute` is real | line 111: `--execute) EXECUTE=1; shift ;;` | PASS | none |
| `--sh2-precheck` is real | line 112 + case branch at line 680 | PASS | none |
| `--sh2-verify-slot` is real, takes slot as the very next token | line 113: `SH2_SLOT="${2:-}"; shift 2`; case branch line 817 | PASS | none |
| `--sh2-verify-valid` is real, same shape | line 114: `SH2_SLOT="${2:-}"; shift 2`; case branch line 759 | PASS | none |
| `--make-otp-json` is real | line 115; case branch line 667 | PASS | none |
| `--key` is real | line 116 | PASS | none |
| `--slot` is real | line 117 | PASS | none |
| `--out` is real | line 118 | PASS | none |
| README's `$R --sh2-verify-slot 1 --key ...` argument shape matches script | grep of usage lines 50, 124, 819-820 — slot is a bare positional value after the flag, exactly as README shows | PASS | none |
| `sign-firmware.sh <image>.uf2 <key.pem>` argument order | `Read scripts/sign-firmware.sh` line 36: `IMG="${1:-}"; KEY="${2:-}"` | PASS | none |
| Output name `<image>.signed.uf2` | line 50: `OUT="${OUT:-${IMG_IN%.uf2}.signed.uf2}"` | PASS | none |
| `nix run .#build-firmware` attribute exists | `grep -n build-firmware flake.nix` → defined line 93 | PASS | none |
| `build-firmware` output is UNSIGNED | flake.nix lines 118-123: seals with a dummy key then runs `picosign sign -clear` to strip pubkey+signature | PASS | none |
| §3 cross-ref (udev rule, recovery-image download) | docs/custom-firmware.md `## 3. Toolchain` contains both the udev-rule block and "Download your recovery image *first*" subsection | PASS | none |
| §4 cross-ref (generate key) | docs `## 4. Generate your key` | PASS | none |
| §2 cross-ref (Pico 2 rehearsal) | docs `## 2. Rehearse on a $5 board first` | PASS | none |
| §5, §6 cross-ref (OTP burn step) | docs `## 5. Read-only reconnaissance` (precheck/verify-slot) + `## 6. The two irreversible writes` (make-otp-json/otp load/otp set/verify-valid) — README's step 4 block spans exactly this content | PASS | none |
| §7 cross-ref (build/sign/flash, and the pubkey-link confirmation) | docs `## 7. Build, sign, flash`, incl. "One extra check before you flash" subsection | PASS | none |
| `picotool otp load ~/.sh2/my-otp.json` | `grep -n 'picotool '` both files, README:136 vs docs:312 — identical string | PASS | none |
| `picotool otp set -s BOOT_FLAGS1.KEY_VALID 0x2` | README:138 vs docs:334 — identical string | PASS | none |
| `picotool load --verify <image>.signed.uf2` | README:161 vs docs:399 — identical string | PASS | none |
| Relative links resolve (`docs/custom-firmware.md`, `LICENSE`) | `ls -la docs/custom-firmware.md LICENSE` — both exist | PASS | none |
| Markdown structure (code fences, headings) | fence count = 18 (even/balanced); heading levels unchanged by this edit | PASS | none |

Internal-consistency facts also cross-checked and found consistent between
README and docs/custom-firmware.md: "three free OTP slots" (4 total, slot 0
factory), `(UNLOCKED)` meaning (two valid keys, not a fault), WRITE 1 = burn
hash / WRITE 2 = mark valid, and which steps are reversible vs. permanent.

Commit hashes `dbb187a` / `e3c0c21` cited in the "About this fork" intro were
also confirmed via `git show --stat` to be the actual CODEX32 and md1/mk1
merge commits, matching their PR-status descriptions.

## FEATURE LIST

`git log upstream/main..main --oneline` plus the merge-commit trail were used
to enumerate fork-native user-visible features. Found a fourth shipped,
top-level, user-reachable feature not in the "About this fork" list:

- **BIP-39 Password** — merged as `f6cb8d3` ("Engrave BIP-39 Password — the
  passphrase plate program"), building on `20fa4c4` (passphrase-flow) and
  `e990f0b` (passphrase keyboard widget). It is a genuine seventh top-level
  program (confirmed via `gui/gui.go:1967` — `titleTxt = "BIP-39 Password"`,
  and commit `36cf02e` "the seventh program — BIP-39 Password, second in the
  menu"). Confirmed absent from `upstream/main:gui/gui.go` (no "password"
  hits) and confirmed fork-only from the merge commit message for
  `e990f0b`: "Fork-side only; no upstream PR." No PR number exists, so per
  instructions the PR clause states "no upstream PR" rather than inventing one.

Added as a new bullet in the same style/register as the existing three
(CODEX32, md1/mk1, SLIP-39 recovery): what it does (own plate, two optional
fingerprint fields, opt-in QR carrying only the passphrase), why a user would
want it (passphrase-protected wallet, backed up on steel not memorized), and
the PR-status clause.

Also corrected, in the same intro sentence I was editing: it claimed "two
additive features, merged as `dbb187a` and `e3c0c21`" while the list already
held three bullets before this pass (the SLIP-39 bullet was added later, in
commit `8574227`, without updating the count) and now holds four. Reworded to
"a growing set of additive features — the first two merged as `dbb187a` and
`e3c0c21`" so the sentence no longer misstates the count while still crediting
the two hashes that are genuinely verifiable as the fork's founding merges.
This was a pre-existing inaccuracy unrelated to the DRAFT install section, but
directly adjacent to the edit and would have become more wrong (3→4 bullets
under a "two" claim) if left untouched.

## UNRESOLVED

None. Every claim in scope was checked against source and confirmed true; no
claim was deleted.
