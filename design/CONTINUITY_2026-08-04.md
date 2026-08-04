# Continuity — SeedHammer II fork, 2026-08-04

Handoff for resuming after a context clear. Written at a deliberate boundary:
three releases published, `main` in sync, nothing half-finished. **The next step
needs the user's hands (flash + engrave), not more analysis.**

---

## 1. Where everything is

| | |
|---|---|
| Fork repo | `/scratch/code/shibboleth/seedhammer`, branch `main` @ `cd2cbd5`, **in sync with `origin/main`** |
| Remote | `git@github.com:bg002h/seedhammer.git` — **PUBLIC** |
| Planning docs | `/scratch/code/shibboleth/mnemonic-engrave/design/` (this file, `FOLLOWUPS.md`, `SPEC_*`, `IMPLEMENTATION_PLAN_*`, `agent-reports/`) |
| Boot key | `~/.sh2/sh2-boot-key.pem`, mode 600, **outside all repos**. Its fingerprint is in `~/.sh2/sh2-boot-key.fingerprint` — read it from there, never paste it into a repo file |
| Recovery image | `~/.sh2/recovery/seedhammerii-v1.4.3.uf2` |
| `picotool` | `/nix/store/j7pl045ik6yb73zvq3n9a52j85d2qnig-picotool-2.2.0-a4/bin/picotool` (not on PATH) |

Six stale worktrees exist (`seedhammer-wt-*`); every branch is merged into
`main` and they can be removed with `git worktree remove` whenever convenient.

## 2. Hardware state

- The user's **retail SeedHammer II boots their own firmware**. Their secp256k1
  boot key was burned into **OTP slot 1** on 2026-08-03 — one-time, permanent.
- `CRIT1 = 0x01` (secure boot on). `BOOT_FLAGS1 = 0x03` — slots 0 **and** 1
  `KEY_VALID`, none invalidated.
- **Slot 0 is the recovery path and must never be invalidated.** It holds
  SeedHammer's production key (`c8314536…`), which is why the official v1.4.3
  image can always be re-flashed.
- **The device is currently running `v0.0.0-gf6cb8d3`** — the ORIGINAL glyphs.
  None of the font work below has been flashed.

## 3. What shipped

Three GitHub releases, all deliberately at **v0.0.0** (see §5):

| Tag | Contains |
|---|---|
| `fork-v0.0.0-gcd2cbd5` | + `rnm` appended to the FONTPROOF! pattern. **Never flashed or run — source-verified only** |
| `fork-v0.0.0-gab44d72` | + `f` crossbar y=4→4.5, asterisk 8-arm r=1 → 6-arm r=2. Booted OK |
| `fork-v0.0.0-gf6cb8d3` | The **Engrave BIP-39 Password** feature. Booted OK, `(UNLOCKED)` |

**Engrave BIP-39 Password** — new top-level program, second of seven. Passphrase
(≤100 chars, full printable ASCII, non-ASCII refused), two optional fingerprint
fields, opt-in QR. The QR encodes **only the passphrase**, never the
fingerprints. Spaces engrave as a visible space mark (`0x1F`). Timing is
quantized **per engrave run**, so a k-part glyph costs k units and content stays
non-leaking at row granularity. `FONTPROOF!` typed into any of the three fields
offers to load the test pattern (98 runes: the 95-char sweep + `rnm`) plus
`DEAD BEEF` / `CAFE BABE`.

Gates passed: fable whole-feature review (0C/3I → folded → clean), opus
execution review of FONTPROOF! (0C/0I, 3 Minor, two folded one filed). Verbatim
reports in `design/agent-reports/`.

## 4. What is OPEN

**O1 — hardware legibility. The single most important open item.**
Most of the 96 glyphs have **never been cut into metal**. One plate has been
read, which produced exactly two fixes (`f`, `*`) — both now in `main` but
**not yet flashed**. Still unread: the descenders `g/j/p/q/y`, the `rn`/`m`
pair (`rnm` was appended to the pattern *for* this check), and the confusable
groups `0O`, `1lI|`, `5S`, `2Z`, `adoq`.

**O4** — footer/legend wording measured at 3 mm.

Filed in `design/FOLLOWUPS.md` with owning phases:
- `seedhammer-passphrase-qr-quiet-zone` (→ O1) — ~1.75 mm vs the ISO 4-module
  convention. **O1 Plate A is exactly this worst case**; if its scan fails,
  suspect the quiet zone before module size.
- `seedhammer-passphrase-space-underscore-compensating-swap` (→ O1, UX only)
- `seedhammer-plate-carries-only-secret-prefix` (→ next engraving cycle) —
  test-gap only; shipped code is correct.
- `seedhammer-fontproof-guide-line` (→ doc pass)
- `seedhammer-fuzzconstantqr-never-reaches-ecc-l-dim37`
- **No golden covers `*` or lowercase `f`.** A full-charset golden over the
  FONTPROOF! pattern would pin every glyph at once — the obvious next safety net,
  not yet built.

**Fable run 2 is unspent**, held for the pre-engrave gate.

## 5. Conventions that are easy to get wrong

- **Version is permanently `v0.0.0-g<sha>`.** Never a bare semver: the fork
  inherits upstream's `v1.4.x` tags and anything resembling one reads as an
  official SeedHammer release. Release tags are `fork-v0.0.0-g<sha>` — the exact
  string the device shows. A `fork-v0.1.0` tag was created and **deliberately
  retagged away** for introducing a second, conflicting number.
- **The boot key must never be committed.** `flake.nix` contains an upstream
  `dummy_pem` whose PEM header trips naive secret greps — it is not the boot key.
- **`-update` on an existing golden is forbidden** (one documented exception:
  three `slip39-*`). New goldens are fine.
- **The signed `.uf2` boots only on this user's device.** Publish it for
  reproducibility; the **unsigned** build is what others want.
- **Judge a flash on machine power, not USB.** `monitorPowerSupply` demands
  20–28 V before the LCD initialises, so on a laptop cable the device drops back
  to BOOTSEL and looks like a failed boot. Success = home screen + `(UNLOCKED)`.
- Rust-primary rule does not apply to the font (fork-native, no Rust counterpart).

## 6. Traps that have each cost real time

- **Check `$?`, never grep for FAIL.** A build error goes to a different stream;
  `go test | grep FAIL || echo GREEN` printed GREEN on a broken build once, and a
  `grep | head` secret scan silently reported clean.
- **Mutate to verify a test.** Seven false-passing tests were found this way in
  this feature and **none by reading**. Break the code; if the suite stays green,
  the test is decoration.
- **`op.Drawer.ExtractText` collects runes regardless of occlusion**, so no
  text-based test can see an overdrawn label. Measure rectangles.
- **`uiContains` lowercases AND strips spaces from its needle** — a screen titled
  "Font Proof" is matched by a readout containing `FONTPROOF!`. This has made
  negative assertions vacuously green three separate times.
- **A rendered space inks nothing**, so `ExtractText` yields `_=SPACE`, never
  `_ = SPACE`.
- **The font is a B-spline.** SVG points are control points, not the drawn curve;
  collinear midpoints and 180° reversals change the shape. **Always render
  through `cmd/vectorfont -dump`** before believing a glyph edit.
- **`vectorfont` writes `constant.go`/`constant.bin` into the current
  directory.** Run font experiments in a throwaway worktree — it once left a
  variant font in the repo root.
- **`runeDuration` pads EVERY glyph to the longest run.** One fat glyph taxes the
  whole plate. Currently **572245, set by `#`**, pinned by
  `TestPassphraseRuneDurationPin`. A 6-arm asterisk was chosen over 8-arm
  precisely because 8-arm r=2 cost +11.7% on every character.
- **Nix:** `nix develop --command …` from the repo root. Expanding `$PATH` in the
  outer shell clobbers the dev shell's.

## 7. Resume here

**Everything below is blocked on the user, not on analysis.**

1. **Flash `gcd2cbd5`** so the font fixes reach metal. Requires the user to hold
   the button while plugging USB to reach BOOTSEL. Then:
   ```sh
   cd /scratch/code/shibboleth/seedhammer
   PT=/nix/store/j7pl045ik6yb73zvq3n9a52j85d2qnig-picotool-2.2.0-a4/bin/picotool
   $PT load --verify seedhammerii-v0.0.0-gcd2cbd5.signed.uf2
   ```
   (Artifacts already built and signed, in the repo root, gitignored.) Then
   unplug USB and power from the machine's own supply.
2. **Run O1**: `FONTPROOF!` into any of the three fields → Plate A. Read back the
   descenders, `rn`/`m`, and the confusable groups. Scan the QR — that answers
   the quiet-zone follow-up for free.
3. **Fold whatever the plate says** into `font/constant/constant.svg`. The loop
   is established: render variants via `vectorfont -dump`, price them against
   `runeDuration`, let the user choose, then regenerate + full suite + no golden
   moved.
4. **Before the first engrave of a real passphrase**, spend fable run 2.

To rebuild/sign/release a later iteration:
```sh
nix run .#build-firmware                     # -> seedhammerii-v0.0.0-g<sha>.uf2
/scratch/code/shibboleth/mnemonic-engrave/scripts/sign-firmware.sh <img>.uf2 ~/.sh2/sh2-boot-key.pem
# then ALWAYS verify the key link before flashing:
$PT info -a <img>.signed.uf2 | awk '/[Pp]ublic key:/{print $NF}' | head -1 | xxd -r -p | sha256sum
# must equal:  awk '{print $1}' ~/.sh2/sh2-boot-key.fingerprint
gh release create fork-v0.0.0-g<sha> --repo bg002h/seedhammer ...
```
Note `gh` resolves to **upstream** `seedhammer/seedhammer` without
`--repo bg002h/seedhammer`. Always pass it.
