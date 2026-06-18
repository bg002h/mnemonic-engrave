# cycle-prep recon — 2026-06-18 — Slice 3: passphrase flow + both-fingerprints verify + which-fingerprint-to-engrave

**Tree at recon time:** `e990f0b` ("Merge feat/passphrase-keyboard: passphrase keyboard widget (Slice 2)")
**Local branch:** `main`
**Sync state:** clean working tree; no `origin/master` ref exists in this fork (trunk is `main`). All citations verified against the `e990f0b` working tree, which equals `HEAD`.
**Build:** `go build ./...` → clean. Go at `/home/bcg/.local/go/bin/go`.

Verdict: **clean — minor drift only.** The earlier RECON_seedhammer_input_ux.md line numbers are mostly accurate or off by ≤10; no structural errors. PassphraseKeyboard is built+tested but **not yet consumed by any flow** (confirmed Slice 3 is the wiring slice).

---

## 1. Seed-backup + fingerprint plumbing (verified vs e990f0b)

### `deriveMasterKey` — **ACCURATE** (`gui/gui.go:187`, recon said ~187)
```go
func deriveMasterKey(m bip39.Mnemonic, net *chaincfg.Params) (*hdkeychain.ExtendedKey, bool) {
	seed := bip39.MnemonicSeed(m, "")
	mk, err := hdkeychain.NewMaster(seed, net)
	...
	return mk, err == nil
}
```

### The hardwired-`""` injection point — **ACCURATE** (`gui/gui.go:188`, recon said gui.go:188)
`seed := bip39.MnemonicSeed(m, "")` — this is THE single injection point inside `deriveMasterKey`. Because `masterFingerprintFor` (and the validity check at `gui.go:2071`) both route through `deriveMasterKey`, threading a `password` only needs to reach this one line. There is a **second** identical hardwire in `genSeed` test helper (`backup/backup_test.go:347`) and `fillDescriptor` (`gui/gui_test.go:299`) — both test-only, both pass `""`.

### `bip39.MnemonicSeed` — **ACCURATE** (`bip39/bip39.go:217`, recon said 217)
```go
func MnemonicSeed(m Mnemonic, password string) []byte {
	var sentence []byte
	for i, w := range m { ... }  // lowercased word labels, space-joined
	return pbkdf2.Key(sentence, []byte("mnemonic"+password), 2048, 64, sha512.New)
}
```
Pure function: no global state, no persistence of `password`. Salt is `"mnemonic"+password` (BIP-39 exact). **Confirms §3:** empty passphrase → salt `"mnemonic"`; non-empty → `"mnemonic"+pw` → different 64-byte seed → different master key → different fingerprint. 2048 PBKDF2-SHA512 rounds per call.

### `masterFingerprintFor` — **DRIFTED-by-0/structurally-same** (`gui/gui.go:482`, recon said ~482 — exact)
```go
func masterFingerprintFor(m bip39.Mnemonic, network *chaincfg.Params) (uint32, error) {
	mk, ok := deriveMasterKey(m, network)
	if !ok { return 0, errors.New("failed to derive mnemonic master key") }
	pkey, err := mk.ECPubKey()
	if err != nil { return 0, err }
	return bip32.Fingerprint(pkey), nil
}
```
**Call sites (ALL):** exactly one — `gui/gui.go:455` inside `engraveSeed`.

### `bip32.Fingerprint` — **ACCURATE** (`bip32/bip32.go:38`)
```go
// Fingerprint is the first 4 bytes of the RIPEMD160(SHA256(pkey)).
func Fingerprint(pkey *secp256k1.PublicKey) uint32 {
	mfp := address.Hash160(pkey.SerializeCompressed())[:4]
	return binary.BigEndian.Uint32(mfp)
}
```
**Format = 4 bytes / `uint32`.** Rendered as 8 uppercase hex via `%.8X` (below).

### `engraveSeed` sets `MasterFingerprint` — **DRIFTED-by-~1** (`gui/gui.go:454-479`, recon said ~455-479)
```go
func engraveSeed(params engrave.Params, m bip39.Mnemonic) (Plate, error) {
	mfp, err := masterFingerprintFor(m, &chaincfg.MainNetParams)
	...
	qrc, err := qr.Encode(string(seedqr.QR(m)), qr.M)   // QR = WORDS ONLY, passphrase-independent
	...
	seedDesc := backup.Seed{ ..., MasterFingerprint: mfp, ... }  // gui.go:472
	seedSide, err := backup.EngraveSeed(params, seedDesc)
	...
}
```

### The fingerprint IS already engraved today — **CONFIRMED**
`backup.Seed.MasterFingerprint uint32` (`backup/backup.go:21`). Rendered in `frontSideSeed` (`backup/backup.go:161`), guarded `MasterFingerprint != 0`:
```go
// backup/backup.go:181-188
if plate.MasterFingerprint != 0 {
	mfp := fmt.Sprintf("%.8X", plate.MasterFingerprint)   // 8 uppercase hex
	offy := (plateDims.Y-col1Height)/2 - metaMargin
	mfpStr := engrave.String(plate.Font, params.F(plateSmallFontSize), mfp)
	mfpszX, mfpszY := mfpStr.Measure()
	t.Offset((plateDims.X-mfpszX)/2, offy-mfpszY)
	mfpStr.Engrave(t.Yield)
}
```
`EngraveSeed` (`backup/backup.go:62`) just calls `frontSideSeed`. So today's bare seed already stamps `fp(password="")` at the top-center of the plate. **Slice 3 changes only the `uint32` value placed in this field** — render path is untouched.
(`backup.SeedString` has the identical field+render at `backup.go:28` / `backup.go:119-126` — the `md1`/`mk1` string-plate path; out of scope but mirrors the pattern.)

### Whole flow + the seam — **ACCURATE/DRIFTED-by-~-29**
- `backupWalletFlow` at **`gui/gui.go:1888`** (recon said ~1917 — **DRIFTED-by-29**, moved UP):
```go
func backupWalletFlow(ctx *Context, th *Colors, mnemonic bip39.Mnemonic) {
	ss := new(SeedScreen)
	for {
		if !ss.Confirm(ctx, th, mnemonic) { return }
		plate, err := engraveSeed(ctx.Platform.EngraverParams(), mnemonic)
		... // error screen
		completed := NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)
		if completed { return }
	}
}
```
- `SeedScreen.Confirm` at **`gui/gui.go:2001`** (recon said ~2014 — **DRIFTED-by-13**). Returns `bool`. The `return true` is at `gui.go:2078` after seed-validity + `deriveMasterKey` succeed.
- `NewEngraveScreen(...).Engrave` at `gui.go:2309`/`2321`.
- Entry point: `engraveObjectFlow` → `case bip39.Mnemonic:` → `backupWalletFlow` (`gui/gui.go:1808-1809`).

**The seam is explicit and clean:** `backupWalletFlow` line 1891-1894, between `ss.Confirm()==true` and `engraveSeed(...)`. A passphrase-entry step + fingerprint-choice step slot here naturally, since `engraveSeed` is the function that computes the fingerprint and builds the plate. There is no existing "after confirm, before engrave" step today — `Confirm`→`engraveSeed`→`Engrave` is direct.

### `SeedScreen` struct — **ACCURATE** (`gui/gui.go:1996`)
```go
type SeedScreen struct {
	selected int
	words    []Clickable
}
```
`Confirm`/`Draw` take `(ctx, th, [dims,] mnemonic)`. **No slot today** for a fingerprint or passphrase action — its nav is fixed: Back / Edit / (Confirm when complete) (`gui.go:2100-2108`). Adding a passphrase action would mean either a new nav button on `SeedScreen` OR (cleaner) a separate post-Confirm step in `backupWalletFlow`.

---

## 2. PassphraseKeyboard API (Slice 2, to consume) — `gui/passphrase_keyboard.go`

Exported surface (all **present, ACCURATE**):
- `type PassphraseKeyboard struct { Fragment string; ... }` (`:47`) — `Fragment` is the only exported field; holds the case-preserved cleartext passphrase.
- `func NewPassphraseKeyboard(ctx *Context) *PassphraseKeyboard` (`:62`) — builds 3 pages (lower/UPPER/symbols), self-`Clear()`s.
- `func (k *PassphraseKeyboard) Update(ctx *Context) bool` (`:200`) — same drain idiom as shared Keyboard; returns true when a key committed. Handles clicks, D-pad nav, hardware-rune input (cross-page, case-sensitive). Has reveal (show/hide mask) + page-cycle + space + backspace.
- `func (k *PassphraseKeyboard) Layout(ctx *Context, th *Colors) (op.Op, image.Point)` (`:339`) — renders masked (`*`×len) or revealed readout ABOVE the grid; **returns COMBINED extent (readout+grid)** (`:402-404`), confirmed.
- `func (k *PassphraseKeyboard) Clear()` (`:161`) — resets Fragment="", page=0, revealed=false, cursor to center.
- Also exported: `func (k *PassphraseKeyboard) Valid(key ppKey) bool` (`:171`) — but `ppKey` is unexported, so not externally callable; internal use only.

**Consumption pattern to mirror** = `inputCodex32Flow` (`gui/gui.go:672-753`):
```go
kbd := newCodex32Keyboard(ctx)       // → NewPassphraseKeyboard(ctx)
backBtn, okBtn := ...
for !ctx.Done {
	for kbd.Update(ctx) {}            // drain
	... // per-frame derive/validate from kbd.Fragment
	if backBtn.Clicked(ctx) { break }
	if ok && okBtn.Clicked(ctx) { return kbd.Fragment, true }
	dims := ctx.Platform.DisplaySize()
	kbdOp, kbdsz := kbd.Layout(ctx, th)         // COMBINED extent
	kbdOp = kbdOp.Offset(content.S(kbdsz))      // position bottom-anchored
	... // title, nav, etc.
	ctx.Frame(op.Layer(frameOps...))
}
return "", false
```
A passphrase flow returns `(string, bool)`. The `inputCodex32Flow` shape is a 1:1 template — but note PassphraseKeyboard's `Layout` already bundles the readout, so the consuming flow does NOT need the separate fragment-box that `inputCodex32Flow` hand-rolls (gui.go:700-715).

---

## 3. Fingerprint computation for BOTH passphrases — **CONFIRMED**

- After threading a `password` param through `deriveMasterKey`→`masterFingerprintFor`, computing both is **two calls with different `password`**: `masterFingerprintFor(m, "")` and `masterFingerprintFor(m, entered)`. The chaincfg/hdkeychain usage (`hdkeychain.NewMaster(seed, net)` → `mk.ECPubKey()` → `bip32.Fingerprint`) is identical for both; only the PBKDF2 salt differs.
- **Cost:** each is one PBKDF2-SHA512 @ 2048 rounds + one secp256k1 pubkey derive + Hash160. Two calls = 4096 rounds total. Acceptable on RP2350 (the device already does one such derive per engrave today, and one more in the `Confirm` validity check at `gui.go:2071`).
- **Format:** `uint32` (4 bytes), displayed as 8 uppercase hex (`%.8X`).
- **BIP-39 semantics — CONFIRMED from source** (`bip39.go:225`): salt `"mnemonic"+password`. Empty vs non-empty → different seeds → different fingerprints.
- **Empty-passphrase edge case:** if the entered passphrase is empty, `fp("")==fp("")` — the two fingerprints are identical, the choice is degenerate. Recommendation surfaced in §4: treat an empty entered passphrase as "no passphrase" → skip the fingerprint-choice step and fall straight through to today's behavior.

---

## 4. Design space + decisions needing the user

**Proposed flow (in `backupWalletFlow`, the clean seam at gui.go:1891-1894):**
```
SeedScreen.Confirm() == true
  → ChoiceScreen "Add passphrase?"  [No (default) / Yes]   ← OPTIONAL, default No
       No  → engraveSeed(params, mnemonic)            (today's path, fp(""))
       Yes → passphraseFlow() → entered string
              if entered == ""  → treat as No (fall through)
              else:
                fpBare = masterFingerprintFor(m, "")
                fpPass = masterFingerprintFor(m, entered)
                → ChoiceScreen "Engrave which fingerprint?"
                     ["No passphrase  XXXXXXXX" / "Passphrase  YYYYYYYY"]
                → engraveSeed(params, mnemonic, chosenFp)
  → NewEngraveScreen(...).Engrave()
```

**Decisions the user must make:**

1. **Passphrase-entry placement (recommend: optional post-Confirm step in `backupWalletFlow`).** The seam is clean and keeps `SeedScreen` untouched. Alternative (new top-level menu path) is heavier and not needed. **Recommend OPTIONAL, default = no passphrase = byte-identical to today.** Decision needed: confirm this placement vs. adding a passphrase nav-button onto `SeedScreen` itself.

2. **Fingerprint-choice UX (recommend: a 2-row `ChoiceScreen`).** `ChoiceScreen` (gui.go:1282, `Choose() (int, bool)`) already does single-select-of-N with up/down + confirm — perfect fit. Show both 8-hex fingerprints labelled (e.g. "No passphrase" / "With passphrase"). Decision needed: exact labels + whether to also show the seedqr/words preview, and whether the bare option should be first (safer default) or the passphrase option.

3. **What gets stamped — CONFIRMED scope.** Only `backup.Seed.MasterFingerprint` (the existing `uint32`) changes to the chosen wallet's fp. **The engraved WORDS and the seedqr QR are byte-identical regardless of passphrase** (`seedqr.QR(m)` and the word list take only `m`, never `password` — verified gui.go:459/463). The passphrase STRING is never placed in `backup.Seed`, never engraved, never over NFC. **CONFIRMED.**

4. **Security posture — CONFIRMED.** `MnemonicSeed` is a pure function and does not persist `password` (bip39.go:217). The passphrase lives only in `PassphraseKeyboard.Fragment` (RAM) and any local `password string` during the flow; device is air-gapped/ephemeral (reboots between sessions). Optional `kbd.Clear()` on exit zeroizes `Fragment`. Decision (minor): whether to explicitly clear the local `password` after fingerprint computation — note Go strings aren't easily zeroizable, matching the existing posture (no special zeroization today).

5. **Threading scope (minimal, ~3 signatures):**
   - `deriveMasterKey(m, net)` → add `password string` (the ONE injection at gui.go:188).
   - `masterFingerprintFor(m, net)` → add `password string` (forwards to deriveMasterKey).
   - `engraveSeed(params, m)` → either add `password`/`chosenFp` OR pass the pre-computed `uint32`. **Recommend: pass the already-chosen `mfp uint32` into `engraveSeed`** (compute both fps in the flow, pass the winner) — avoids `engraveSeed` re-deriving and keeps it agnostic.
   - Call-site blast radius: `deriveMasterKey` has 3 callers (gui.go:483 via masterFingerprintFor, gui.go:188 self, gui.go:2071 the Confirm validity check — this last must keep `password=""` since it validates the WORDS, not the passphrased wallet). `masterFingerprintFor` has 1 caller (gui.go:455). All in-package (`gui`).

**Lockstep / test impact:** **none breaking.**
- `backup.Seed.MasterFingerprint uint32` field signature is **unchanged** — the `backup` package (and its golden-plate `TestSeed`, backup_test.go:180, via `genSeed` which builds `Seed` directly with `mfp` for `password=""`) is untouched. **No golden plate regenerates.**
- `bip39.MnemonicSeed` signature already takes `password` (since Slice 0) — no signature change there.
- `MnemonicSeed(m, "")` test helpers (gui_test.go:299, backup_test.go:347, gui_test.go:259) keep `""` and keep asserting today's bare fingerprints — threading the gui-layer `password` param defaults them to `""`, so they don't change.
- bip39 `TestVectors` does NOT exercise the seed-bytes/password path at all (only parse/entropy/checksum) — no assertion to break.
- New tests needed (TDD): a passphrase-keyboard-consuming-flow test + a both-fingerprints-differ assertion + empty-passphrase-skips-choice assertion.

**Sizing / SemVer / PR:**
- Rough LoC: ~80-150 lines in `gui/gui.go` (one passphrase flow ~50-70 LoC mirroring inputCodex32Flow, one "add passphrase?" choice, one fingerprint-choice screen, ~3 signature threads) + tests. No new package.
- SemVer: device firmware versioned via `-ldflags '-X main.Version=...'` (`cmd/controller/main.go:14`). Additive optional UX feature → **MINOR** bump (e.g. next `v1.x.0`). Latest tags: v1.4.2.
- **No upstream PR** (per CLAUDE.md: fork-only feature, `bg002h/seedhammer` kept clean; this is Slice 3 of the fork's input-UX work).

**Cross-cutting:** (1) RECON_seedhammer_input_ux.md line numbers have drifted ≤29 lines post-Slice-2 — re-cite against `e990f0b`. (2) The `backup.SeedString` md1/mk1 string-plate path has the identical `MasterFingerprint` field+render but is out of Slice-3 scope (it's the engrave-from-constellation-string path, not the seed-backup path). (3) No `origin/master`; cite `main @ e990f0b`.

**Recommended next gate:** This recon FEEDS the mandatory R0 gate. Before any code: write the brainstorm spec / IMPLEMENTATION_PLAN, run it through the opus architect R0 loop to 0C/0I (persist each round verbatim to `design/agent-reports/`), then implement TDD in a worktree. The two genuinely user-facing decisions to lock in the brainstorm are **(1) passphrase-step placement** and **(2) the fingerprint-choice screen labels/order** — flagged above.
