# SPEC — SeedHammer passphrase flow + fingerprint-engrave choice (Slice 3)

**Date:** 2026-06-18
**Target repo:** the SeedHammer II firmware fork `bg002h/seedhammer` (Go/TinyGo). Fork-side only — **no upstream PR**.
**Base:** fork `main` `e990f0b` (post-Slice-2 — `gui.PassphraseKeyboard` exists). Branch `feat/passphrase-flow` off `e990f0b`.
**Predecessors:** `design/cycle-prep-recon-passphrase-flow.md` (verified vs `e990f0b`), `design/RECON_seedhammer_input_ux.md` (3-slice decomposition; Slices 1+2 done).
**Slice boundary:** the FINAL input-UX slice — wires the Slice-2 `PassphraseKeyboard` into the seed-backup flow, threads a `password`, computes BOTH master fingerprints, and lets the user choose which to stamp. **The passphrase STRING is never engraved, never over NFC.**

---

## 1. Goal

Add **optional** on-device BIP-39 passphrase entry to the seed-backup flow. After confirming the seed words, the user may add a passphrase; the device then shows **both** the bare (passphrase="") and passphrase-protected master fingerprints and lets the user **choose which fingerprint to stamp** on the engraved plate — so the metal backup is labeled for the wallet it actually belongs to. The engraved **words and SeedQR are identical** regardless (they encode only the mnemonic); only the stamped `MasterFingerprint` differs. The default path (no passphrase) is byte-identical to today's behavior.

## 2. Scope

**In:**
- **S1 — thread `password`:** add a `password string` param to `deriveMasterKey` (the single `bip39.MnemonicSeed(m, "")` injection at `gui.go:188`) and `masterFingerprintFor`; have `engraveSeed` take the already-chosen `mfp uint32` (compute the fingerprint in the flow, pass the winner — `engraveSeed` stays passphrase-agnostic). The `SeedScreen.Confirm` validity check (`gui.go:2071`) keeps `password=""` (it validates the WORDS, not the passphrased wallet).
- **S2 — `passphraseFlow`:** a new flow consuming `PassphraseKeyboard`, returning `(passphrase string, ok bool)`.
- **S3 — `backupWalletFlow` wiring:** after `SeedScreen.Confirm`, an **optional** "passphrase?" `ChoiceScreen` (default = skip → today's bare path); on "add", run `passphraseFlow`; if the entered string is non-empty, compute both fingerprints and present a 2-row fingerprint `ChoiceScreen`; engrave with the chosen `mfp`.

**Out:** NFC passphrase entry; the `backup.SeedString` (md1/mk1 string-plate) path; changing the engrave RENDER (`frontSideSeed` is untouched — only the `uint32` value changes); persisting/zeroizing the passphrase beyond the existing posture; any change to the Slice-2 `PassphraseKeyboard` or the shared `Keyboard`.

**Files:** `gui/gui.go` (`deriveMasterKey`, `masterFingerprintFor`, `engraveSeed`, `backupWalletFlow`, + new `passphraseFlow` + the two `ChoiceScreen` uses), `gui/*_test.go`. `backup/`, `bip39/`, `bip32/`, `gui/passphrase_keyboard.go` UNCHANGED. The bare-fingerprint golden tests (`backup_test.go TestSeed*`) must stay green (they build `backup.Seed` directly with `password=""`).

## 3. Background (vs `e990f0b`)

- `deriveMasterKey(m bip39.Mnemonic, net *chaincfg.Params) (*hdkeychain.ExtendedKey, bool)` (`gui.go:187`) — `seed := bip39.MnemonicSeed(m, "")` at `:188` is THE injection point. Callers: `masterFingerprintFor` (`gui.go:483`), and `SeedScreen.Confirm`'s validity check (`gui.go:2071`).
- `masterFingerprintFor(m, net) (uint32, error)` (`gui.go:482`) → `deriveMasterKey` → `mk.ECPubKey()` → `bip32.Fingerprint` (4 bytes / `uint32`). Sole caller: `engraveSeed` (`gui.go:455`).
- `engraveSeed(params, m) (Plate, error)` (`gui.go:454`): computes `mfp` (`:455`), `qrc := qr.Encode(seedqr.QR(m), …)` (`:459`, **words-only, passphrase-independent**), builds `backup.Seed{… MasterFingerprint: mfp …}` (`:472`), `backup.EngraveSeed` → `toPlate`.
- The fingerprint is ALREADY stamped: `frontSideSeed` (`backup.go:181-188`) renders `fmt.Sprintf("%.8X", plate.MasterFingerprint)` when `!= 0`. **Render path untouched by this slice.**
- `bip39.MnemonicSeed(m, password string) []byte` (`bip39/bip39.go:217`): pure; salt `"mnemonic"+password`; empty vs non-empty → different seed → different fingerprint. Does not persist `password`.
- `backupWalletFlow` (`gui.go:1888`): `for { if !ss.Confirm(...) { return }; plate,_ := engraveSeed(...); … Engrave }`. **The seam is between `Confirm()==true` and `engraveSeed` (`gui.go:1891-1894`).**
- `PassphraseKeyboard` (Slice 2): `NewPassphraseKeyboard(ctx)`, `Update(ctx) bool`, `Layout(ctx,th) (op.Op, image.Point)` (combined readout+grid extent), `Clear()`, `Fragment string`. Consume like `inputCodex32Flow` but Layout already bundles the masked readout (no separate fragment box).
- `ChoiceScreen` (`gui.go:1282`): `Choose(ctx, th) (int, bool)` — single-select-of-N (Up/Down + Button3 confirm, Button1 back). Fields `Title`, `Lead`, `Choices []string`.

## 4. Design

### 4.1 S1 — thread `password` (minimal blast radius)

```go
func deriveMasterKey(m bip39.Mnemonic, net *chaincfg.Params, password string) (*hdkeychain.ExtendedKey, bool) {
	seed := bip39.MnemonicSeed(m, password) // was MnemonicSeed(m, "")
	…
}
func masterFingerprintFor(m bip39.Mnemonic, net *chaincfg.Params, password string) (uint32, error) {
	mk, ok := deriveMasterKey(m, net, password)
	…
}
func engraveSeed(params engrave.Params, m bip39.Mnemonic, mfp uint32) (Plate, error) {
	// no longer calls masterFingerprintFor; uses the passed mfp.
	qrc, err := qr.Encode(string(seedqr.QR(m)), qr.M) // unchanged (words-only)
	…
	seedDesc := backup.Seed{… MasterFingerprint: mfp …}
	…
}
```
Call-site updates: `SeedScreen.Confirm`'s validity check → `deriveMasterKey(m, net, "")` (validates the WORDS). `backupWalletFlow` computes the chosen `mfp` and passes it to `engraveSeed(params, m, mfp)`.

### 4.2 S2 — `passphraseFlow`

```go
// passphraseFlow lets the user enter a BIP-39 passphrase on the PassphraseKeyboard.
// Returns (passphrase, true) on accept (possibly ""), or ("", false) on Back.
func passphraseFlow(ctx *Context, th *Colors) (string, bool) {
	kbd := NewPassphraseKeyboard(ctx)
	backBtn := &Clickable{Button: Button1}
	okBtn := &Clickable{Button: Button3}
	for !ctx.Done {
		for kbd.Update(ctx) {
		}
		if backBtn.Clicked(ctx) {
			return "", false
		}
		if okBtn.Clicked(ctx) {
			return kbd.Fragment, true
		}
		dims := ctx.Platform.DisplaySize()
		screen := layout.Rectangle{Max: dims}
		_, content := screen.CutTop(leadingSize)
		content, _ = content.CutBottom(8)
		kbdOp, kbdsz := kbd.Layout(ctx, th)       // combined masked-readout + grid
		kbdOp = kbdOp.Offset(content.S(kbdsz))     // bottom-anchored
		nav, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack},
			{Clickable: okBtn, Style: StylePrimary, Icon: assets.IconCheckmark},
		}...)
		title, _ := layoutTitle(ctx, dims.X, th.Text, "Enter Passphrase")
		ctx.Frame(op.Layer(kbdOp, nav, title, op.Color(&ctx.B, th.Background)))
	}
	return "", false
}
```
(The masked readout is the keyboard's; OK is always enabled — an empty entry is allowed and treated by the caller as "no passphrase".)

### 4.3 S3 — `backupWalletFlow` wiring (the seam)

```go
func backupWalletFlow(ctx *Context, th *Colors, mnemonic bip39.Mnemonic) {
	ss := new(SeedScreen)
	for {
		if !ss.Confirm(ctx, th, mnemonic) {
			return
		}
		mfp, err := masterFingerprintFor(mnemonic, &chaincfg.MainNetParams, "") // bare
		if err != nil {
			showSeedErr(ctx, th, ss, mnemonic, err) // existing ErrorScreen pattern
			continue
		}
		// Optional passphrase + fingerprint choice.
		switch (&ChoiceScreen{Title: "Passphrase", Lead: "Add a BIP-39 passphrase?", Choices: []string{"Skip", "Add passphrase"}}).Choose(ctx, th); ... {
		// index 0 (Skip) or Back → keep the bare mfp.
		// index 1 (Add passphrase) →
		}
		// (see flow below)
		plate, err := engraveSeed(ctx.Platform.EngraverParams(), mnemonic, mfp)
		…
		if NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme) {
			return
		}
	}
}
```
Concrete flow (the plan nails the exact control structure; this is the design):
1. `ss.Confirm` true → compute `mfp = fp(m,"")`.
2. `ChoiceScreen{Choices: ["Skip", "Add passphrase"]}.Choose`. If `Back` (`!ok`) → loop (re-Confirm). If index 0 (Skip) → keep bare `mfp`, go to engrave.
3. If index 1 (Add passphrase): `pass, ok := passphraseFlow(ctx, th)`. If `!ok` (Back) → re-show the "Passphrase" choice (or loop to Confirm). If `pass == ""` → treat as Skip (keep bare `mfp`).
4. Else compute `passFp = fp(m, pass)`; show a 2-row `ChoiceScreen{Title:"Engrave fingerprint", Choices: ["No passphrase  " + hex(mfp), "Passphrase  " + hex(passFp)]}`. **Bare option first (index 0) = the safer default.** If `Back` → re-show passphrase entry. Else set `mfp` to the chosen fingerprint (`mfp` for index 0, `passFp` for index 1).
5. `engraveSeed(params, m, mfp)` → engrave. (`hex(x) = fmt.Sprintf("%.8X", x)` — matches the plate's render format.)

`fmt` IS already imported in `gui.go`? (Recon: `gui.go` imports `"strings"` but NOT `"fmt"`. So the `%.8X` formatting needs `"fmt"` — add it to `gui.go`'s imports, OR use `strconv`/the existing `engrave.String` path. The plan picks one; cleanest is to add `"fmt"`.)

## 5. Error handling / security

The passphrase lives only in `passphraseFlow`'s `PassphraseKeyboard.Fragment` and a local `pass string` during the flow — never written to `backup.Seed`, never engraved, never over NFC (the device is air-gapped; sessions are ephemeral). `bip39.MnemonicSeed` is pure (no persistence). Go strings aren't reliably zeroizable; this matches the existing posture (no special zeroization today). The fingerprint choice is purely a label on the plate — engraving the "wrong" fingerprint is recoverable (the words are correct) and the choice exists precisely to avoid that. Both `masterFingerprintFor` calls are PBKDF2-2048 (cheap on RP2350; the device already does one per backup + one in `Confirm`).

## 6. Testing (host: `go test ./gui/...`)

- **S1 (threading):** `masterFingerprintFor(m, &chaincfg.MainNetParams, "")` equals today's value AND differs from `masterFingerprintFor(m, …, "pass")` for a known mnemonic (the two fingerprints are distinct). `deriveMasterKey(m, net, "")` unchanged behavior. The bare-fp golden tests (`backup_test.go`) stay green.
- **S2 (`passphraseFlow`):** via `runUI` — drive `NewPassphraseKeyboard` with `runes("Ab1!")` + `click(Button3)` → returns `("Ab1!", true)`; `click(Button1)` → `("", false)`; empty + Button3 → `("", true)`.
- **S3 (wiring + choice):** the fingerprint-choice maps index→fingerprint correctly (index 0 → bare, index 1 → passphrase). Drive the 2-row `ChoiceScreen` (`Down` + `Button3`) and assert the chosen `mfp` is the passphrase fp; assert the readout shows both 8-hex fingerprints. An empty passphrase entered at step 3 → engrave path uses the bare `mfp` (choice skipped). Keep `TestWordKeyboardScreen`/`TestInputSeedCodex32`/SLIP-39/codex32/`TestSeed*` green. (Driving the full `Confirm→choice→passphrase→choice→engrave` E2E is heavy; component-test `passphraseFlow` + the fingerprint-choice + the `masterFingerprintFor` threading, and rely on build + the unchanged bare-path golden for the no-passphrase regression.)

## 7. Versioning / commits

Firmware `-ldflags`-injected (additive optional UX → a MINOR bump at the next tag; no source bump). Commits on `feat/passphrase-flow`, signed (SSH) + DCO, author Brian Goss. Fork-side; no upstream PR. Stage explicit paths.

## 8. Resolved decisions

- **Optional, post-Confirm step in `backupWalletFlow`** (default Skip = byte-identical to today) — recon recommendation; minimal blast radius; `SeedScreen` untouched.
- **Fingerprint choice = a 2-row `ChoiceScreen`**, bare option first (safer default), labels "No passphrase  <hex>" / "Passphrase  <hex>".
- **Empty entered passphrase → treated as Skip** (the two fingerprints would be identical; skip the degenerate choice, engrave bare).
- **Only `backup.Seed.MasterFingerprint` changes** to the chosen wallet's fp; words + SeedQR identical; passphrase string never engraved/over-NFC. Render path (`frontSideSeed`) untouched.
- **Threading is 3 signatures** (`deriveMasterKey`, `masterFingerprintFor`, `engraveSeed`); the `Confirm` validity check keeps `password=""`. Add `"fmt"` to `gui.go` imports for `%.8X` (or use the existing string path — plan decides).
- No `PassphraseKeyboard`/shared-`Keyboard`/`backup`/`bip39` changes.

## 9. Process note

Per project standard: this spec MUST pass the opus-architect **R0 gate to 0C/0I before any code** (fold → persist verbatim to `design/agent-reports/` → re-dispatch until GREEN). Then plan → plan R0 → single-implementer subagent TDD → mandatory whole-diff adversarial execution review. Proceeding autonomously (user directive). Completes the input-UX 3-slice arc (and "c and 2 & 3").
