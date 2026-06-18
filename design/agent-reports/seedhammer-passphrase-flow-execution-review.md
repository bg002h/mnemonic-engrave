<!--
Persisted verbatim from the opus-architect MANDATORY whole-diff execution review
of Slice 3 (BIP-39 passphrase flow + fingerprint-engrave choice).
Reviewer agentId: a68bdd5e0ac995f0d. Phase: post-implementation (non-deferrable).
Diff reviewed: feat/passphrase-flow e990f0b..3908fde (commits d2e458c, 3908fde).
Verdict: GREEN — 0 Critical / 0 Important. Cleared to merge.
Two non-blocking Minors (M-1 structurally-moot test note; M-2 pre-existing layout QA).
The text below is the agent's report exactly as returned; do not edit.
-->

# EXECUTION REVIEW — Slice 3 (passphrase flow) whole diff

**Reviewer:** opus architect (adversarial whole-diff execution review, read-only)
**Commit range:** `e990f0b..HEAD` (`HEAD` = `3908fde`); two commits — `d2e458c` (Task 1, threading) + `3908fde` (Task 2, passphrase flow + fingerprint choice)
**Base:** fork main `e990f0b`
**Worktree:** `/scratch/code/shibboleth/seedhammer-wt-passflow`, branch `feat/passphrase-flow`
**Date:** 2026-06-18
**Scope:** 156 insertions / 21 deletions across `gui/gui.go` and `gui/gui_test.go` only.

## Independently reproduced verification (in the worktree)

Go toolchain: `go version go1.26.4 linux/amd64` (`/home/bcg/.local/go/bin/go`).

```
$ go test -count=1 ./gui/ ./bip39/ ./backup/
ok  	seedhammer.com/gui	4.554s
ok  	seedhammer.com/bip39	0.019s
ok  	seedhammer.com/backup	0.064s

$ go test -count=1 -run 'TestMasterFingerprintPassphrase|TestPassphraseFlow|TestPassphraseFlowBack|TestEngraveFingerprintChoiceMapping' -v ./gui/
--- PASS: TestMasterFingerprintPassphrase (0.01s)
--- PASS: TestPassphraseFlow (0.00s)
--- PASS: TestPassphraseFlowBack (0.00s)
--- PASS: TestEngraveFingerprintChoiceMapping (0.00s)
ok  	seedhammer.com/gui	0.012s

$ go test -count=1 -run TestSeed -v ./backup/
--- PASS: TestSeed (0.00s)
    --- PASS: TestSeed/1-words-12 (0.05s)
    --- PASS: TestSeed/0-words-24 (0.07s)
ok  	seedhammer.com/backup	0.077s

$ go vet ./gui/        # exit 0, no output
$ gofmt -l gui/gui.go gui/gui_test.go   # empty, exit 0
$ git diff --name-only e990f0b..HEAD
gui/gui.go
gui/gui_test.go
$ git status --porcelain   # clean (no uncommitted changes)
$ git diff --stat e990f0b..HEAD -- backup/ bip39/ bip32/ gui/passphrase_keyboard.go   # empty (untouched)
```

The full `gui` suite was run twice (cache-busted) with no flake. The implementer's claimed green results are confirmed exactly.

## Focus-area findings

### 1. Security invariant (highest priority) — PASS

Traced the passphrase string end-to-end:
- `passphraseFlow` returns `kbd.Fragment` (`gui.go:504`), the ONLY read of the passphrase keyboard's fragment.
- It flows to `masterFingerprintFor(mnemonic, &chaincfg.MainNetParams, pass)` (`:1944`) → `deriveMasterKey(..., password)` (`:480`) → `bip39.MnemonicSeed(m, password)` (`:189`). `MnemonicSeed` (`bip39/bip39.go:217-226`) is pure PBKDF2 with salt `"mnemonic"+password`; it does not persist the password. The function's only escaping output is the `uint32` fingerprint.
- `engraveSeed`'s signature is now `engraveSeed(params engrave.Params, m bip39.Mnemonic, mfp uint32)` (`:455`) — it takes only the `uint32`. `backup.Seed` (`backup/backup.go:15-23`) has fields `Mnemonic []string, …, MasterFingerprint uint32, Font` — no passphrase field. `engraveSeed` builds `backup.Seed{… MasterFingerprint: mfp …}` (`:464-471`); the QR is `seedqr.QR(m)` (words-only, passphrase-independent). The passphrase string cannot reach the engrave artifact — this is a compile-time guarantee, not merely a runtime convention.
- No debug/log of the passphrase: `grep` for `Print*`/`log.`/`Sprint` intersected with `pass`/`password`/`Fragment` in `gui.go` matches ONLY the two fingerprint-choice labels (`:1952-1953`), which print `fmt.Sprintf("%.8X", mfp)` / `passFp` — the `uint32` hex, never the string. The on-screen readout in `passphraseFlow` is the keyboard's masked (`*`) readout (`passphrase_keyboard.go:343`).

No leak. ✓

### 2. Control-flow correctness of the `backupWalletFlow` rewrite — PASS

Full branch matrix of one iteration (`gui.go:1931-1973`), against base (`e990f0b:gui/gui.go:1888-1913`):
- `Confirm` false → `return`. ✓
- bare `masterFingerprintFor` err → `showSeedError` + `continue`. ✓
- `ppChoice.Choose` → Back `(_,false)` or Skip `(0,true)`: `ok && sel==1` is false → engrave bare. **Skip ≡ Back ≡ bare**, matching spec §4.3. ✓
- `ppChoice` → Add `(1,true)`: `passphraseFlow` Back `(_,false)` or empty-accept `("",true)` → `ok && pass!=""` false → engrave bare (**empty ≡ Skip**). ✓
- non-empty pass: `passFp` err → `showSeedError`+`continue`; `fpChoice` Back `(_,false)` → `continue` → re-Confirm (passphrase discarded — `pass` is block-scoped, keyboard re-created fresh); `(0,true)` → `mfp` stays bare; `(1,true)` → `mfp = passFp`. ✓
- **Exactly one engrave per iteration:** every path reaching `engraveSeed` (`:1965`) does so once; all early-outs are `continue`/`return` *before* `engraveSeed`. No double-engrave, no skipped-engrave. ✓
- **Index mapping:** choices `["No passphrase …", "Passphrase …"]` with `if sel == 1 { mfp = passFp }` — index 0 = bare (default, since fresh `ChoiceScreen.choice` is 0), index 1 = passphrase. Correct, no off-by-one. ✓ (Verified `ChoiceScreen.Choose` at `gui.go:1337-1387` returns `(s.choice, true)` on confirm, `(0,false)` on cancel.)
- **No stale state / no cross-iteration leak:** `ppChoice` and `fpChoice` are freshly allocated each iteration (`:1941`, `:1949`) so `choice` defaults to 0 (R0 C-2 correctly resolved); `mfp` is re-`:=`'d each iteration (`:1935`); `pass` is block-scoped and discarded on `continue`. ✓

### 3. Threading completeness & no behavior change on the no-passphrase path — PASS

All three signatures threaded (`deriveMasterKey` `:188`, `masterFingerprintFor` `:479`, `engraveSeed` `:455`). All call sites updated:
- `masterFingerprintFor` called at `:1935` (`""`) and `:1944` (`pass`).
- `deriveMasterKey` called at `:480` (from `masterFingerprintFor`) and the `Confirm` validity check at `:2132` correctly keeps `""` (validates the WORDS) — matches spec §4.1.
- No-passphrase path is byte-identical to base: base `engraveSeed` computed `masterFingerprintFor(m, &chaincfg.MainNetParams)` with `password=""`; new path computes `masterFingerprintFor(mnemonic, &chaincfg.MainNetParams, "")` and passes the resulting `mfp` to a render-identical `engraveSeed`. The `backup` `TestSeed` goldens derive `mfp` independently via `bip32.Fingerprint(pkey)` in `backup/backup_test.go:305,357` — fully decoupled from the gui diff — and stay green. ✓

### 4. The implementer's one deviation (`strings.ToUpper(w)` in `bip39FromWords`) — VERIFIED CORRECT, test-only

The fork's wordlist stores UPPERCASE labels: `const words = "ABANDONABILITYABLE…"` (`bip39/wordlist.go`). `ClosestWord` (`bip39/bip39.go:95-104`) binary-searches with `LabelFor(Word(i)) >= word` and confirms via `strings.HasPrefix(match, word)` where `match` is the uppercase label. A lowercase query (e.g. `"abandon"`) sorts lexicographically AFTER every uppercase label (ASCII lowercase > uppercase), so `sort.Search` returns `len(index)` → `(-1, false)` — i.e. lowercase would FAIL. Therefore `strings.ToUpper(w)` is REQUIRED for the helper to resolve words, the explanatory comment is accurate, and the deviation is confined to a test helper (`gui_test.go`), masking no production bug. ✓

### 5. Compile/vet/test reality — PASS

See the reproduced output above: `go test ./gui/ ./bip39/ ./backup/` green (cache-busted), `go vet ./gui/` clean (exit 0), `gofmt -l` empty, `git diff --name-only` is exactly the two gui files, working tree clean, `backup/`+`bip39/`+`bip32/`+`passphrase_keyboard.go` untouched. The `"fmt"` import was correctly added (`gui.go:4`); `"errors"`/`"strings"` retained. ✓

### 6. Test quality — PASS (with one MINOR observation)

- `TestMasterFingerprintPassphrase`: proves bare (`""`) and passphrase (`"TREZOR"`) fingerprints genuinely DIFFER (`if bare == pass { t.Errorf }`) on a real 12-word vector. Load-bearing — would catch a threading regression that dropped the password (the prime "no behavior change" risk inverted). ✓
- `TestPassphraseFlow` / `TestPassphraseFlowBack`: drive the real flow via pre-queued `ctx.Router` events; assert `("Ab1!", true)` on runes+Button3 and `("", false)` on Button1-Back. Because `PassphraseKeyboard` commits runes verbatim (`Fragment += string(key.r)`, "NO ToUpper — case preserved", `passphrase_keyboard.go:182`), the `"Ab1!"` assertion would catch any accidental case-folding or fragment corruption of the passphrase. Confirmed the direct-call (no-`runUI`) pattern is sound: with `FrameCallback` nil, `ctx.Frame` is a no-op (`gui.go:71-76`) and the flow terminates on the first-iteration `Clicked`. ✓
- `TestEngraveFingerprintChoiceMapping`: drives a fresh 2-row `ChoiceScreen` with `Down, Button3` and asserts `(1, true)` — exercises real Down-navigation + the exact index→fingerprint mapping used in the flow. ✓

MINOR observation (not a defect): no test asserts the passphrase string is *absent* from a built `Plate`/`backup.Seed`. That absence is, however, structurally guaranteed by `engraveSeed`'s signature (`mfp uint32` only — there is no string parameter into which a passphrase could flow), which is a stronger compile-time guarantee than any runtime assertion could provide. The test suite's chosen coverage (fingerprints differ + index mapping + flow I/O) is the right load-bearing set.

## Findings

### CRITICAL
None.

### IMPORTANT
None.

### MINOR
- **M-1 (test coverage, structurally moot):** No test asserts the passphrase string is absent from the engrave artifact; this is guaranteed at compile time by `engraveSeed(…, mfp uint32)` having no string parameter. No action required.
- **M-2 (pre-existing, carried by spec §6 R0 M-4):** The fingerprint-choice labels (`"No passphrase " + 8-hex` ≈ 23 chars) are a layout-QA item, not verified by an automated render-width assertion in this diff. The spec explicitly classed this as a non-correctness QA item; flagged here only for completeness.

## Verdict

**GREEN — 0 Critical / 0 Important. Cleared to merge.**

The diff faithfully implements SPEC §4.1–§4.3. The security invariant holds with a compile-time guarantee (passphrase string never reaches `backup.Seed`/engrave/QR/NFC/log). The `backupWalletFlow` rewrite has exactly-one-engrave-per-iteration, terminates on every branch, correct back-semantics (Skip≡Back≡empty≡bare; fp-choice-Back→re-Confirm with discarded passphrase), correct 0/1 index mapping, fresh per-iteration state, and no cross-iteration passphrase leak. The no-passphrase path is byte-identical to base, and the `backup` goldens (independently derived) stay green. The lone implementer deviation (`strings.ToUpper` in the test helper) is verified correct against the uppercase wordlist and `ClosestWord`'s uppercase binary search, and is test-only. Tests, `vet`, and `gofmt` independently reproduce green; the diff is confined to the two gui files. The two MINOR items are non-blocking.
