# Lens 1 — operator-error review (incl. FIRMWARE-QUICKSTART.txt)

*Persisted verbatim. opus, 2026-07-26 three-lens fan-out (round 5). NOTHING FOLDED YET.*

---

Reviewed: `FIRMWARE-QUICKSTART.txt` (145 lines, side-by-side with `design/RUNBOOK_custom_boot_key.md`), the SH2 modes and phase bodies of `scripts/pico2-bootkey-rehearsal.sh`, `scripts/sign-firmware.sh`, round-3 and round-4 reports. Shell claims were tested empirically (fish 4.8.1, bash) rather than reasoned about.

---

## C1 — CRITICAL: both documents tell the operator to generate the device's permanent boot private key inside a git working tree with a public remote, which does not ignore `*.pem`

**`FIRMWARE-QUICKSTART.txt:104`** (and `design/RUNBOOK_custom_boot_key.md:149`)

Every `$PWD` in section 3 resolves to `/scratch/code/shibboleth/seedhammer` (preamble line 8). Verified in that repo:

```
origin   git@github.com:bg002h/seedhammer.git      # public fork
upstream git@github.com:seedhammer/seedhammer.git
.gitignore = "seedhammerii-*.uf2" + "_artifacts"   # that is the whole file
$ git check-ignore -v sh2-boot-key.pem my-otp.json  -> no match (both tracked-able)
```

Round 3's I7 fixed the key's *name*; nobody checked its *location*. `sh2-boot-key.pem` lands as an untracked file in a repo whose stated workflow is "upstream PRs branch off `upstream/main`" — a repo that gets committed to and pushed routinely, sometimes by agents.

**Failure scenario:** operator finishes section 4 at 2am, machine boots, they tidy up — `git status` in the fork shows `sh2-boot-key.pem`, `my-otp.json`, and their firmware changes; `git add -A && git commit && git push` (or any agent doing the same). The secp256k1 private key that is now a *permanently valid boot key* for a machine that engraves seed backups is on GitHub. OTP cannot be un-burned. The remedy is burning one of the two remaining slots for a fresh key — and the leaked key stays valid forever unless they touch `KEY_INVALID`, the one field both documents deliberately refuse to write out.

**Fix:** generate outside every repo and use the absolute path everywhere in both docs — `mkdir -p ~/.sh2 && chmod 700 ~/.sh2; openssl ecparam … -out ~/.sh2/sh2-boot-key.pem && chmod 600 ~/.sh2/sh2-boot-key.pem` — replacing all four `$PWD/sh2-boot-key.pem` occurrences (quickstart 104/108/115/126, runbook 149/170/193/236) and `$PWD/my-otp.json`. Belt and braces: add `*.pem` and `*-otp.json` to the fork's `.gitignore`.

---

## C2 — CRITICAL: the quickstart demotes the mandatory precheck to "Survey" and mis-describes what it checks, so a genuine STOP will read as a tool false-positive

**`FIRMWARE-QUICKSTART.txt:100-101`** vs **`RUNBOOK:99-137`**

The card says: `a) Survey (read-only; refuses anything that isn't a SeedHammer II)`. That is one of six assertions. Absent from the card entirely: `SECURE_BOOT_ENABLE == 1`, `KEY_VALID == 0x1`, `KEY_INVALID == 0`, **spare slots 1-3 empty across all 16 rows each**, and **page locks clear** — the only read-only writability gate on the whole procedure (script lines 588-604, 262-274). Also absent: any statement that a failure means STOP. The runbook says "READ ONLY, do this first" and "It asserts, and STOPs on any failure". The card says "Survey".

**Failure scenario:** `--sh2-precheck` dies with `spare slot 1 is NOT empty (0x…) — this device has been modified before. STOP.` The operator's card told them this tool's job is to "refuse anything that isn't a SeedHammer II". They know it *is* a SeedHammer II. At 2am that reads as a tool bug, not a device fact. They skip (a) and run (c)→(d). `picotool otp load` ORs into a partially-written slot; (e) then mismatches; slot 1 is irreversibly gone having proved nothing, and they repeat on slot 2. Identical shape if `PAGE1_LOCK0` is non-zero — the one condition that makes the whole procedure impossible and that only (a) can detect before a write. This is round-3 C1 recurring one artifact downstream: the safety apparatus exists and works, and the document the operator actually holds does not tell them it is load-bearing.

**Fix:** retitle (a) `MANDATORY GATE — run first, read every line`, list the five assertions in one line each, and add: *"Any FAIL here is a fact about your device, not a tool bug. Do not run (b)-(f). Go read the runbook."* Also state that (a) is what pins the CHIPID that (e) later re-checks — skipping it silently downgrades (e)'s wrong-device check to a first-time pin (script lines 407-417, `else` branch).

---

## I1 — IMPORTANT: "(e) fails → use slot 2" is not an executable procedure, and (f)'s verification value is wrong on that path

**`FIRMWARE-QUICKSTART.txt:113-119`**

The card's abort branch is four words: *"STOP and use slot 2"*. It does not say to re-run (c) with `--slot 2` and a new `--out`, then (d), then (e) with `--sh2-verify-slot 2`. The only slot-2-shaped thing on the card is the inline comment on the irreversible line: `# slot 2 = 0x4`.

**Failure scenario:** (e) mismatches. The operator, told to "use slot 2", reaches for the only slot-2 instruction visible — line 118 — and types `picotool otp set -s BOOT_FLAGS1.KEY_VALID 0x4`, permanently validating a slot that is **all zeros**. Two of three spares now gone, nothing burned, nothing signable. Compounding it: line 119's `# expect field = 3` is only correct for slot 1. On the slot-2 path the correct answer is `5`; an operator who did burn slot 2 correctly reads `5`, believes the write failed, and starts "fixing" it.

**Fix:** replace the abort branch with the literal loop — *"Re-run (c) with `--slot 2 --out ~/.sh2/my-otp-slot2.json`, then (d) with that file, then (e) with `--sh2-verify-slot 2`, then (f) with `0x4`. You have three attempts total."* — and make (f)'s expectation slot-dependent: `slot 1 -> 3, slot 2 -> 5, slot 3 -> 9`.

---

## I2 — IMPORTANT: the udev block is the one block in the document that cannot be safely pasted in *either* shell, and it has already misfired on this machine

**`FIRMWARE-QUICKSTART.txt:29-32`**

Tested, not assumed:

- **bash** — the heredoc terminator is indented (`    EOF`), so it never matches. Reproduced verbatim: `warning: here-document at line 1 delimited by end-of-file (wanted 'EOF')`, **exit status 0**, the rules file written containing `    EOF` and `    udevadm control --reload-rules && udevadm trigger`, and the reload/trigger **never runs**.
- **fish** — `t.fish (line 1): Expected a string, but found a redirection` → exit 127, nothing runs.

And the evidence that this already fired: `/scratch/code/shibboleth/mnemonic-engrave/` currently contains five **root-owned, 1-byte** files named `SUBSYSTEM==usb,`, `ATTRS{idVendor}==2e8a,`, `MODE=0660,`, `TAG+=uaccess`, `EOF` — the signature of `sudo tee` receiving the rule tokens as filenames — dated `Jul 26 18:40`, one minute after `FIRMWARE-QUICKSTART.txt` (18:39). `/etc/udev/rules.d/99-picotool.rules` is correct now, but its mtime is 18:41, i.e. hand-repaired a minute later.

**Failure scenario:** the bash variant is the dangerous one because it exits 0. The operator sees no error, replugs, `picotool info` still fails (rules never reloaded, file full of junk). The card's next suggestion is sudo — and the card itself warns (lines 37-42) that sudo leaves root-owned state in `rehearsal-work/` that "breaks partway through — possibly between two OTP writes". The silent failure steers the operator directly at the workaround the same section forbids.

**Fix:** un-indent the heredoc body and terminator to column 0 (or use `<<-EOF` with tabs), and print it as a single unambiguous line instead:
`printf 'SUBSYSTEM=="usb", ATTRS{idVendor}=="2e8a", MODE="0660", TAG+="uaccess"\n' | sudo tee /etc/udev/rules.d/99-picotool.rules` — one line, no heredoc, safe in fish and bash. Then delete the five stray files from the repo root before someone commits them.

---

## I3 — IMPORTANT: nothing in §2 says the rehearsal must PASS, and phase 5b silently skips itself into a green result

**`FIRMWARE-QUICKSTART.txt:75-90`**, script `:551-573`

The card lists phases 0-6 as a checklist and never says *"if any phase FAILs, do not proceed to section 3."* That sentence is the entire justification for owning a Pico. Worse, phase 5b — the only proof that the 2.4 MB real firmware (not just a 20 KB blinky) is accepted, i.e. follow-up F11(d) — is conditional on `ls "$SEEDHAMMER_DIR"/seedhammerii-*.uf2` finding a file. If it doesn't, the script prints two `warn` lines and then `ok_done`s the phase anyway. The card's only hint that phase 5 needs a built image is the parenthetical `(+5b real firmware)`; it never says to build one first, and §0 — which produces it — is labelled `[safe, anytime]`, which reads as optional.

**Failure scenario:** operator skips §0 (or cleaned the artifact — `rehearsal-work/` does not exist on this machine today, and the built uf2 is a gitignored file that any `git clean -xdf` removes). Phase 5 blinks, prints PASS, two yellow warns scroll off. Operator records a green rehearsal and proceeds to the irreversible SH2 write with the small-image-only acceptance proof — the exact "green that didn't cover what mattered" pattern round 3 was written about.

**Fix:** add to §2: *"Run §0 first — phase 5b needs `seedhammerii-*.uf2` in the fork dir or it silently skips the real-firmware proof."* and *"If any phase FAILS or 5b reports 'skipping', the rehearsal is not green. Do not touch the engraver."*

---

## I4 — IMPORTANT: the recovery path both documents rest on is not staged, and neither document says to stage it

**`FIRMWARE-QUICKSTART.txt:140-142`**, `RUNBOOK:323-324`

"you can always flash a stock UF2 to get a working machine back" / "flash an official `seedhammerii-vX.Y.Z.uf2`". I searched: no official SeedHammer UF2 exists anywhere on this machine — not in either repo, not in `~/Downloads`. The only `.uf2` present is the locally built `seedhammerii-66d3121….uf2` plus two 2-block test fixtures. The v1.4.3 image whose signature round-4's settled facts verified is gone.

**Failure scenario:** section 4 flashes, the machine does not boot. The safety net the whole risk assessment is priced on now requires downloading and integrity-checking a firmware image, at 2am, under stress, on the machine whose engraver is bricked-looking — with a bad-download or wrong-version outcome landing squarely in the "it still won't boot" panic that I5/round-3 identified as the second-slot-burning path.

**Fix:** make it a prerequisite in both docs, above the irreversible section: *"Before step (d): download `seedhammerii-v1.4.3.uf2`, verify `sha256(embedded pubkey) == c8314536…319a473b` with `picotool info -a`, and keep it in `~/.sh2/`. This is your only recovery path; have it on disk before you need it."*

---

## I5 — IMPORTANT: section 3 is entirely `$PWD`-relative with its only `cd` ninety lines earlier, across a procedure explicitly spanning days

**`FIRMWARE-QUICKSTART.txt:98-119`** (7 × `$PWD`, plus `R=../mnemonic-engrave/…`) vs the runbook, which reprints `cd /scratch/code/shibboleth/seedhammer` inside step 1, step 2 *and* step 3 (`:109, :148, :191`).

`$R` and `picotool otp load $PWD/my-otp.json` fail loudly from the wrong directory, which is fine. The unsafe one is **(b) and (c)**, which *create* files at `$PWD` and therefore succeed anywhere.

**Failure scenario:** operator does (a)-(c) one night; next session opens a terminal (cwd `~`, or `cd`s into `mnemonic-engrave` to re-read the runbook), resumes at (b) because they can't remember whether the key exists, sees no `sh2-boot-key.pem` in the current directory, and re-runs line 104 — generating a **second, different key**. They burn (d) the hash of key B, then sign in §4 with whichever key their shell history reaches first. If that's key A: firmware won't boot, the card says "the hash is PROVEN correct, it's a signing problem, re-sign" (line 143) — which is true and yet will never fix it, because the mismatch is between two files with the same name in two directories.

**Fix:** hardcode absolute paths (`~/.sh2/sh2-boot-key.pem`, `~/.sh2/my-otp.json`) throughout section 3 — which C1's fix does anyway — and add at the top of section 3: *"Every path below is absolute on purpose. Never re-run (b) — if the key file is missing, you have lost it; stop and read the runbook's Recovery section."*

---

## Minor

- **`FIRMWARE-QUICKSTART.txt:126,127`** — literal `<version>` placeholder, inconsistent with §0 line 56 in the same document, which correctly uses `$(git rev-parse HEAD)`. In bash `seedhammerii-<version>.uf2` tokenizes as redirections: creates a junk file named `.uf2`, errors `version: No such file or directory`, command never runs. Fails safe but wastes a cycle at the worst hour. Round 3's M2 was only half-folded — `RUNBOOK:236` and `:276` still carry the same placeholder. Fix: use `$(git rev-parse HEAD)` in both docs.
- **`RUNBOOK:213-214` absent from the quickstart** — "Use `otp set -s` (OR-in), never `otp load`, for this field — `otp load` attempts to clear bits and will fail." Omitted from the card next to the second irreversible write.
- **`FIRMWARE-QUICKSTART.txt:125-127`** — neither document says `sign-firmware.sh` rewrites the UF2 **in place** (`sign-firmware.sh:82` `cp "$WORK/sealed.uf2" "$IMG"`, `:97,:109` `picosign sign … "$IMG"`). An operator who re-runs §0's `sha256sum` after signing gets a mismatch against CI, concludes the build is corrupt, rebuilds over the signed image, and flashes an unsigned one.
- **`FIRMWARE-QUICKSTART.txt:125`** — §4 wraps the call in `nix develop --command` although the preamble (lines 6-8) says you are already in the devshell, while §2/§3 use bare `$R`. The inconsistency invites the inverse error: running `$R --sh2-precheck` *outside* the devshell → `picotool not found` → improvisation with a host/sudo picotool at the gate.
- **`FIRMWARE-QUICKSTART.txt:78`** ("The board is CONSUMED") — `rehearsal-work/` is framed as spent junk, but it holds `sh2-chipid.txt` (the wrong-device pin used by (e)) and the three rehearsal keys that `reject_rehearsal_key` compares against (`script:436-448`). Deleting it makes **both** protections no-ops silently — the key check `continue`s over missing files, the CHIPID check takes the "pin it fresh" branch. Neither doc says to keep the directory until section 3 is complete.
- **`FIRMWARE-QUICKSTART.txt:103-105`** — the runbook's encrypted-at-rest suggestion (`RUNBOOK:150-152`) and its emphatic "Back it up **now**, off this machine" (`:155`) are compressed to "Make and back up your key" with no method and no off-machine requirement.
- **`FIRMWARE-QUICKSTART.txt:107`** — "(touches no device…)" is accurate, but `--make-otp-json` calls `build_blinky`, which shells out to **tinygo** when `rehearsal-work/blinky.uf2` is absent (it is absent right now). SH2 modes skip the tinygo/go precheck (`script:122-126` gates on `$PHASE`, empty in SH2 modes), so a missing toolchain surfaces as `blinky build failed` mid-section-3.

## Nit

- **`FIRMWARE-QUICKSTART.txt:132`** — "checks the embedded bytes match" is stated unconditionally; `sign-firmware.sh:138-146` skips that comparison and prints `could not normalise DER…` when the DER halves don't normalise to 128 hex chars.
- **`FIRMWARE-QUICKSTART.txt:118`** — the comment lists `slot 2 = 0x4, slot 3 = 0x8` but never labels `0x2` as slot 1; `RUNBOOK:209` does.

## Answers to the specific questions

1. **Divergence.** Commands, flags, key filenames and slot numbers agree between the two documents — that part is clean. The divergences are all *subtractive* (the card drops guards: I1, I2's precheck contents in C2, Minor items 2/6) or *anchoring* (the runbook repeats `cd` per step, the card does not — I5). One item exists only in the card and is therefore wholly unreviewed until now: the udev/permissions section (I2). Where they disagree on correctness, the **runbook** is right in every case except `<version>`, where both are wrong and §0 of the card is right.
2. **Omitted guards.** C2 (the five precheck assertions and their STOP semantics), I1 (the slot-2 recovery loop), I3 ("all phases must pass"), Minor 2 (`otp set -s`, never `otp load`). C2 and I1 can each cause an irreversible mistake — a consumed spare slot — that the runbook would have prevented.
3. **Ambiguity at the irreversible steps.** (d) is unambiguous *if* (a) was run; C2 is exactly the argument that (a) can reasonably be skipped or dismissed. (f) is unambiguous on the happy path and ambiguous on the abort path, where it can validate an empty slot (I1). Wrong-key and wrong-slot are both reachable (I5, I1); running twice is harmless; running out of order is caught by the script.
4. **Copy-paste hazards.** `<version>` (Minor 1), the heredoc (I2), `$PWD` drift (I5). No dangerous-default prompts remain — round 4's `ask_blink` reprompts until a literal `y`/`n` and 5b is machine-judged.
5. **fish/bash.** The bash requirement is stated clearly at lines 6-10 and repeated implicitly by the `R=` idiom. Tested on the actual fish here (4.8.1): `$(…)` and `.#build-firmware` both work, so §0/§1 are fish-safe; `R=…` and multi-line blocks fail loudly and harmlessly. The single block that is unsafe to paste is the udev heredoc — and it is unsafe in **bash** as well, silently (I2). So the fish warning is adequate and the actual trap is elsewhere.

**Not verifiable from the repo:** nothing here was exercised on hardware. picotool's behaviour when `otp load` targets a partially-written slot (C2's scenario) is asserted from the docs' own claims, not confirmed. I did not audit script internals beyond what the documents claim about them.

**Verdict:** the quickstart is not safe to paste from as written — C1 (boot key generated inside a public-remote git tree, in both documents) and C2 (the mandatory precheck presented as an optional "Survey" whose real STOP conditions are unlisted) must be fixed before either document is used on the engraver.