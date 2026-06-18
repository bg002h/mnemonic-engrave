<!--
Persisted verbatim. opus-architect MANDATORY whole-diff execution review of the SLIP-39
Trezor-routing fork. Reviewer agentId a7c958c937348741e. Diff feat/slip39-trezor-routing
9db3fd2..8574227 (2 commits). Verdict: GREEN — 0C/0I, cleared to merge. Independently re-ran
tests/vet/gofmt/build; verified the fork routing (no seed to backupWalletFlow without the
fork+fingerprint; verbatim arm has no BIP-39 fingerprint), C1 build-safety (only local ack
removed), the test-hang fix is sound (drives engrave to completion, not ctx.Done; assertions
load-bearing incl. the GIFT discriminator), no production loop changed, no-hang, scope clean.
The text below is the agent's report exactly as returned; do not edit.
-->

# EXECUTION REVIEW — SLIP-39 Trezor-routing whole diff

**Reviewer:** opus architect (independent adversarial whole-diff execution review)
**Commit range:** `9db3fd2..HEAD` (2 commits: `643d15f` feat, `8574227` docs)
**Base:** fork `main` `9db3fd2` (Cycle D shipped — trusted)
**Worktree / branch:** `/scratch/code/shibboleth/seedhammer-wt-slip39-trezor` @ `feat/slip39-trezor-routing`
**Date:** 2026-06-18
**Scope:** READ-ONLY. `README.md` (+7), `gui/slip39_polish.go` (+56/-31 net), `gui/slip39_polish_test.go` (+142). 174 insertions.

## Reproduced verification (tails)

```
$ go test -count=1 ./gui/ ./slip39/ ./bip39/
ok  seedhammer.com/gui     7.592s
ok  seedhammer.com/slip39  0.039s
ok  seedhammer.com/bip39   0.028s

$ go vet ./gui/           → exit 0 (no output)
$ gofmt -l gui/           → (empty — nothing unformatted)
$ go build ./...          → exit 0 (no output)

$ git diff --name-only 9db3fd2..HEAD
README.md
gui/slip39_polish.go
gui/slip39_polish_test.go            (exactly the 3 files)

$ go test -count=1 -run 'SLIP39|Recover' -v ./gui/
… all PASS, incl:
--- PASS: TestEngraveSLIP39RecoverToBackup       (0.02s)
--- PASS: TestEngraveSLIP39RecoverForkVerbatim   (2.20s)
--- PASS: TestEngraveSLIP39RecoverForkBack        (0.01s)
--- PASS: TestEngraveSLIP39BackoutRecognized      (0.00s)
--- PASS: TestConfirmSLIP39LoneButton2NoHang      (0.00s)
PASS  ok seedhammer.com/gui  2.269s
```

Green and clean on all independent reproductions.

## Per-focus findings

### 1. Fork logic — the seed can never reach `backupWalletFlow` without the fork + fingerprint
The only routing into `engraveRecoveredSLIP39` is `engraveSLIP39` (slip39_polish.go:372), passing `scan` and `m`; grep confirms no other caller (the only other matches are comments and the test). Inside the fork (lines 388-428):
- `!ok` (Back) → `return false` → caller `continue`s to the original confirm (recognized, no engrave). Verified `ChoiceScreen.Choose` returns `(0, false)` on Button1/cancel (gui.go:1344-1345, 1386).
- `sel == 1` → `engraveSLIP39Verbatim(ctx, th, scan); return true` — **no** `masterFingerprintFor`, **no** `confirmSLIP39Fingerprint`, **no** `backupWalletFlow`. The verbatim arm never shows a BIP-39 fingerprint.
- `sel == 0` (default) → `masterFingerprintFor` → `confirmSLIP39Fingerprint` (decline → `return false`) → `backupWalletFlow`. `backupWalletFlow` is reachable only through the fingerprint gate. Confirmed.

No defect.

### 2. C1 (build-safety) — only the local `ack` literal removed
The diff removes the `ack := &ConfirmWarningScreen{…}` literal and its `holdToConfirm` call *inside the old function body only*. Independently confirmed all three symbols survive and are still used:
- `holdToConfirm` — defined slip39_polish.go:338, **still called** at line 261 (the §5.6 high-iteration-exponent gate in `recoverSLIP39Flow`).
- `ConfirmWarningScreen` — **still constructed** at line 256 (that same gate) and gui.go:2083.
- `assets` import (line 12) — **still used** at lines 116/119/121/259/451/452 (`assets.IconBack/IconRight/IconHammer/IconInfo`).

`go vet`/`gofmt -l`/`go build` all clean → no dangling/unused symbol or import. No defect.

### 3. C2 + test-hang fix
(a) **No production loop changed.** `git diff` lands no line inside `engraveSLIP39Verbatim`'s `for { if NewEngraveScreen(ctx, plate).Engrave(...) { return } }` body (slip39_polish.go:492-496); diff body-match count = 0. Byte-identical to base.
(b) **Test fix is sound, masks nothing.** `TestEngraveSLIP39RecoverForkVerbatim` wires `p.engraver = newEngraver()` and drives the engrave to completion — `click(Button3,Button3,Button3)` → `press(Button3)` → `frame()` → `time.Sleep(confirmDelay)` → loop on `e.closes`/`p.wakeups` → `click(Button3)` → `synctest.Wait()` → assert `frame()` `!ok`. This is a faithful mirror of `TestEngraveScreen` (gui_test.go:256-273), the established pattern. The loop terminates because the engrave *completes* (`e.closes` fires), not because `ctx.Done` is set — so it exercises the real success path rather than masking the spin.
(c) **Assertions are real, not tautological.** Positive: `pump("Insert a blank plate", 128)` — that literal is the EngraveScreen body (gui.go:2468), proving the verbatim path reaches the EngraveScreen. Negatives accumulate *every* frame into `seen` (not just up to the anchor): `!uiContains(all, "Recovered Fingerprint")` (the unique title of `confirmSLIP39Fingerprint`, slip39_polish.go:454) and `!uiContains(all, "GIFT")`. The "GIFT" discriminator is genuinely load-bearing: `TestEngraveSLIP39RecoverToBackup` proves the *same* `slip39Vec3` fixture renders "GIFT" on the BIP-39 SeedScreen (line 394), and I verified neither `slip39Vec3` share string contains "gift" — so if the verbatim arm wrongly fell into `backupWalletFlow`, "GIFT" would appear and the test would fail. `uiContains` is case-insensitive (gui_test.go:481), so render-case is irrelevant.
(d) **Spin-once-Done is pre-existing and benign; not worsened.** Confirmed the identical `for { … .Engrave(ctx,…) }` pattern in `mdmkFlow` (gui.go:1918), `backupWalletFlow` (1931/1986), and `descriptorFlow`'s engrave path. This change adds no new instance and modifies no production loop.

No defect.

### 4. I1 / UX
Both choice strings — `"BIP-39 seed"`, `"Engrave shares"` — are short and rendered via `widget.Label` (single-line, gui.go:1408); the explanation lives in the width-wrapped `Lead` (`widget.Labelw`, gui.go:1392). The diff comments correctly document this Label-vs-Labelw distinction. Default index is 0 = "BIP-39 seed" (`s.choice` zero-inits; `Choose` returns it absent any Down). Labels are far under the 480px (here 240px test display, real 480px) single-line budget. No defect.

### 5. Button2-drain / no-hang
`ChoiceScreen.Choose` handles its own navigation (Button1 cancel, Button3 choose, Up/Down via `InputTracker`, pointer per choice) and registers **no** `ButtonFilter(Button2)`. `EventRouter.Next` is head-only (event.go:266-279), but `Reset()` (called once per frame via `ctx.Reset()`→gui.go:95, in the loop at gui.go:2627) discards any head event matching none of the frame's accumulated filters — so a queued Button2 at the head is dropped each frame, never blocking Button3. This is the identical, already-shipped mechanism the SLIP-39 passphrase `ppChoice` relies on. No direct-call screen in the new flow leaves a queued Button2 stuck. No Cycle-B footgun. No defect.

### 6. Safety / scope
- No secret logged; grep for added `SeedString|hex.|log.|fmt.Print|passphrase=|secret` in the diff → empty. No hex/`SeedString` artifact (the consult's won't-build).
- `engraveSLIP39Verbatim` / `confirmSLIP39Fingerprint` / `backupWalletFlow` / `masterFingerprintFor` bodies unchanged (diff touches only comments and the new fork call site).
- Diff scope is exactly the 3 declared files; no `backup/` or other package touched.
- README line accurate: button label "Engrave shares" matches `Choices[1]`; BIP-39-default behavior correctly described; mnemonic-toolkit off-device path noted.
- Existing guards green: `TestConfirmSLIP39*`, `TestRecoverSLIP39*`, `TestEngraveSLIP39BackoutRecognized`, codex32/BIP-39/backup all PASS.

No defect.

## Findings

- **CRITICAL:** none
- **IMPORTANT:** none
- **MINOR:** none

## Verdict

**GREEN — 0 Critical / 0 Important. Cleared to merge.**
