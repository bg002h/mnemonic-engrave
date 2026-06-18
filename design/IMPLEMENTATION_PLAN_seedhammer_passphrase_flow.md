# SeedHammer Passphrase Flow + Fingerprint-Engrave Choice (Slice 3) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Optional on-device BIP-39 passphrase entry in the seed-backup flow: thread a `password` through seed/fingerprint derivation, compute both the bare and passphrase-protected master fingerprints, and let the user choose which to stamp on the engraved plate. The passphrase string is never engraved. Default (no passphrase) is byte-identical to today.

**Architecture:** **Task 1** = pure threading (add `password` to `deriveMasterKey`+`masterFingerprintFor`, all callers pass `""` → behavior-preserving, standalone-compilable). **Task 2** = the atomic flow feature: change `engraveSeed` to take the chosen `mfp uint32` (removing its internal fingerprint call), add `passphraseFlow` (consuming the Slice-2 `PassphraseKeyboard`) + a `showSeedError` helper, and rewrite `backupWalletFlow` to offer the optional passphrase + the 2-row fingerprint `ChoiceScreen`. `backup`/`bip39`/`bip32`/`PassphraseKeyboard` UNCHANGED; the fingerprint render path (`frontSideSeed`) untouched; no golden regen.

**Tech Stack:** Go/TinyGo. Host tests `/home/bcg/.local/go/bin/go test ./gui/... ./backup/...`.

**Base:** fork `main` `e990f0b`. Branch `feat/passphrase-flow`. Fork-side only; no upstream PR.

**Spec:** `design/SPEC_seedhammer_passphrase_flow.md` (R0 GREEN at R1 — the complete `backupWalletFlow`/`passphraseFlow`/`showSeedError` code there is R1-verified compile-accurate). **PLAN GATE:** must pass the opus plan R0 gate (0C/0I) before any code.

---

## File Structure

| File | Responsibility | Tasks |
|---|---|---|
| `gui/gui.go` *(modify)* | T1: `deriveMasterKey`+`masterFingerprintFor` gain `password`; 3 call sites pass `""`. T2: `engraveSeed(…, mfp uint32)`; `passphraseFlow`; `showSeedError`; rewrite `backupWalletFlow`; add `"fmt"` import. | 1,2 |
| `gui/*_test.go` *(modify)* | T1: both-fingerprints threading test. T2: `passphraseFlow` + fingerprint-choice tests; add `"fmt"` import (used by `TestEngraveFingerprintChoiceMapping`). | 1,2 |
| `backup/`, `bip39/`, `bip32/`, `gui/passphrase_keyboard.go` *(unchanged — must stay green)* | bare-fp golden `TestSeed*`; the keyboard widget. | guard |

**Commit hygiene:** explicit paths. Signed + DCO: `git commit -S -s` (fall back to `-s` if signing unavailable, say so).

---

## Task 0: Worktree + clean baseline

- [ ] **Step 1:** `cd /scratch/code/shibboleth/seedhammer && git worktree add /scratch/code/shibboleth/seedhammer-wt-passflow -b feat/passphrase-flow e990f0b && cd /scratch/code/shibboleth/seedhammer-wt-passflow && git config user.name "Brian Goss" && git config user.email "goss.brian@gmail.com"`
- [ ] **Step 2:** Baseline — `/home/bcg/.local/go/bin/go test ./gui/... ./backup/...` → PASS. If red, STOP.

---

## Task 1: S1 — thread `password` (behavior-preserving, standalone-compilable)

**Files:** modify `gui/gui.go`, `gui/gui_test.go`.

**Context:** add a `password string` param to `deriveMasterKey` (the single `bip39.MnemonicSeed(m, "")` at `gui.go:188`) and `masterFingerprintFor`, and update ALL THREE callers to pass `""` so behavior is unchanged and the package compiles standalone. (`engraveSeed`'s signature stays `(params, m)` for now — Task 2 changes it; here its internal call just gains a `""`.) Callers (verified): `masterFingerprintFor`'s internal `deriveMasterKey` (gui.go:483), the `SeedScreen.Confirm` validity check (gui.go:2071 — MUST stay `""`, validates the words), and `engraveSeed`'s `masterFingerprintFor` (gui.go:455).

- [ ] **Step 1: Write the failing test** — append to `gui/gui_test.go`

```go
func TestMasterFingerprintPassphrase(t *testing.T) {
	m := emptyBIP39Mnemonic(12)
	for i := range m {
		w, _ := bip39.ClosestWord(bip39.LabelFor(bip39.Word(i % int(bip39.NumWords))))
		_ = w
	}
	// Use a known-valid 12-word mnemonic from the test corpus (the abandon vector).
	mn := bip39FromWords(t, "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about")
	bare, err := masterFingerprintFor(mn, &chaincfg.MainNetParams, "")
	if err != nil {
		t.Fatalf("bare fp: %v", err)
	}
	pass, err := masterFingerprintFor(mn, &chaincfg.MainNetParams, "TREZOR")
	if err != nil {
		t.Fatalf("passphrase fp: %v", err)
	}
	if bare == pass {
		t.Errorf("bare and passphrase fingerprints must differ: both %08X", bare)
	}
}
```

> The helper to build a `bip39.Mnemonic` from a word string: if `gui_test.go` lacks one, add a tiny `bip39FromWords(t, s)` that splits `s` and maps each via `bip39.ClosestWord` (exact). (Check for an existing parser first — `emptyBIP39Mnemonic`/the word-flow tests may already have one; reuse it. Remove the dead `emptyBIP39Mnemonic` loop above if you use a direct builder.) The exact mnemonic-construction idiom is a small adaptation; keep it minimal and valid.

- [ ] **Step 2: Run to verify it fails** — `/home/bcg/.local/go/bin/go test ./gui/ -run TestMasterFingerprintPassphrase` → FAIL (`masterFingerprintFor` takes 2 args, not 3 — compile error).

- [ ] **Step 3: Thread `password`** — three edits in `gui/gui.go`:

(a) `deriveMasterKey` (gui.go:187-188):
```go
func deriveMasterKey(m bip39.Mnemonic, net *chaincfg.Params, password string) (*hdkeychain.ExtendedKey, bool) {
	seed := bip39.MnemonicSeed(m, password)
	...
```
(b) `masterFingerprintFor` (gui.go:482-483): add `, password string` to the signature; change its internal call to `deriveMasterKey(m, network, password)`.
(c) Update the other two callers to pass `""`:
  - `engraveSeed` (gui.go:455): `mfp, err := masterFingerprintFor(m, &chaincfg.MainNetParams, "")`.
  - `SeedScreen.Confirm` validity check (gui.go:2071): `deriveMasterKey(mnemonic, &chaincfg.MainNetParams, "")` (it validates the words — keep `""`).

- [ ] **Step 4: Run to verify it passes** — `/home/bcg/.local/go/bin/go test ./gui/... ./backup/...`
Expected: PASS — the new threading test passes (bare != "TREZOR" fp), and ALL existing tests stay green (behavior unchanged; bare fp identical). The `backup` golden `TestSeed*` are unaffected (they don't call these gui funcs).

- [ ] **Step 5: Commit**
```bash
git add gui/gui.go gui/gui_test.go
git commit -S -s -m "gui: thread password through deriveMasterKey/masterFingerprintFor (default \"\")"
```

---

## Task 2: S2/S3 — passphrase flow + fingerprint choice (atomic)

**Files:** modify `gui/gui.go`, `gui/gui_test.go`.

**Context:** atomic because changing `engraveSeed`'s signature to take `mfp uint32` forces its sole caller `backupWalletFlow` to change in the same commit. Adds `"fmt"` import, the `showSeedError` helper, `passphraseFlow`, and the `backupWalletFlow` rewrite — all from the R1-verified spec §4.1-4.3. The fingerprint render path (`frontSideSeed`) and `backup.Seed.MasterFingerprint` are unchanged.

- [ ] **Step 1: Write the failing tests** — append to `gui/gui_test.go`

```go
func TestPassphraseFlow(t *testing.T) {
	ctx := NewContext(newPlatform())
	frame, quit := runUI(ctx, func() {
		s, ok := passphraseFlow(ctx, &descriptorTheme)
		// stash the result for assertion via package vars or a closure:
		passphraseFlowResult = s
		passphraseFlowOK = ok
	})
	defer quit()
	runes(&ctx.Router, "Ab1!")
	click(&ctx.Router, Button3)
	// advance frames until the flow returns (Done) — pull a couple of frames
	for i := 0; i < 4; i++ {
		if _, ok := frame(); !ok {
			break
		}
	}
	if passphraseFlowResult != "Ab1!" || !passphraseFlowOK {
		t.Errorf("passphraseFlow = (%q, %v), want (\"Ab1!\", true)", passphraseFlowResult, passphraseFlowOK)
	}
}
```

> NOTE on driving a flow that RETURNS: `passphraseFlow` returns when OK/Back is clicked (it does not loop forever). The cleanest test harness: call `passphraseFlow` directly (NOT in `runUI`) after pre-queuing input, since `ctx.Frame` is a no-op without a `FrameCallback`. I.e.:
> ```go
> func TestPassphraseFlow(t *testing.T) {
> 	ctx := NewContext(newPlatform())
> 	runes(&ctx.Router, "Ab1!")
> 	click(&ctx.Router, Button3)
> 	s, ok := passphraseFlow(ctx, &descriptorTheme)
> 	if s != "Ab1!" || !ok { t.Errorf("= (%q,%v), want (\"Ab1!\",true)", s, ok) }
> }
> func TestPassphraseFlowBack(t *testing.T) {
> 	ctx := NewContext(newPlatform())
> 	click(&ctx.Router, Button1)
> 	if s, ok := passphraseFlow(ctx, &descriptorTheme); ok || s != "" {
> 		t.Errorf("back: = (%q,%v), want (\"\",false)", s, ok)
> 	}
> }
> ```
> Use the direct-call form (it mirrors `TestEngraveCodex32BackoutNotUnknown`'s direct `engraveObjectFlow` call). Drop the `runUI`/package-var version above.

- [ ] **Step 2: Run to verify it fails** — `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestPassphraseFlow'` → FAIL (`undefined: passphraseFlow`).

- [ ] **Step 3: Add the `"fmt"` import** to the import block of BOTH `gui/gui.go` AND `gui/gui_test.go` (both currently lack `"fmt"`; `gui.go` needs it for the fingerprint-choice labels in `backupWalletFlow`, and `gui_test.go` needs it for `fmt.Sprintf("%.8X", …)` in `TestEngraveFingerprintChoiceMapping` — Step 8).

- [ ] **Step 4: Add `passphraseFlow`** — append to `gui/gui.go` exactly the spec §4.2 body:

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
		kbdOp, kbdsz := kbd.Layout(ctx, th)
		kbdOp = kbdOp.Offset(content.S(kbdsz))
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

- [ ] **Step 5: Add `showSeedError`** — append to `gui/gui.go` exactly the spec §4.3 helper:

```go
func showSeedError(ctx *Context, th *Colors, ss *SeedScreen, mnemonic bip39.Mnemonic, err error) {
	errScr := NewErrorScreen(err)
	for !ctx.Done {
		dims := ctx.Platform.DisplaySize()
		d, dismissed := errScr.Layout(ctx, th, dims)
		if dismissed {
			return
		}
		main := ss.Draw(ctx, th, dims, mnemonic)
		ctx.Frame(op.Layer(d, main))
	}
}
```

- [ ] **Step 6: Change `engraveSeed` to take `mfp`** — in `gui/gui.go`, edit `engraveSeed` (gui.go:454):
  - signature: `func engraveSeed(params engrave.Params, m bip39.Mnemonic, mfp uint32) (Plate, error) {`
  - DELETE the first 4 lines (`mfp, err := masterFingerprintFor(m, &chaincfg.MainNetParams, ""); if err != nil { return Plate{}, err }`). Keep `qrc, err := qr.Encode(...)` as the first statement (it already declares `err`). The `MasterFingerprint: mfp` field now uses the passed param.

- [ ] **Step 7: Rewrite `backupWalletFlow`** — replace the entire current `backupWalletFlow` body (gui.go:1888 through its closing brace, including the inline error loop) with the R1-verified spec §4.3 version:

```go
func backupWalletFlow(ctx *Context, th *Colors, mnemonic bip39.Mnemonic) {
	ss := new(SeedScreen)
	for {
		if !ss.Confirm(ctx, th, mnemonic) {
			return
		}
		mfp, err := masterFingerprintFor(mnemonic, &chaincfg.MainNetParams, "") // bare
		if err != nil {
			showSeedError(ctx, th, ss, mnemonic, err)
			continue
		}
		ppChoice := &ChoiceScreen{Title: "Passphrase", Lead: "Add a BIP-39 passphrase?", Choices: []string{"Skip", "Add passphrase"}}
		if sel, ok := ppChoice.Choose(ctx, th); ok && sel == 1 {
			if pass, ok := passphraseFlow(ctx, th); ok && pass != "" {
				passFp, err := masterFingerprintFor(mnemonic, &chaincfg.MainNetParams, pass)
				if err != nil {
					showSeedError(ctx, th, ss, mnemonic, err)
					continue
				}
				fpChoice := &ChoiceScreen{
					Title: "Engrave fingerprint",
					Choices: []string{
						"No passphrase " + fmt.Sprintf("%.8X", mfp),
						"Passphrase " + fmt.Sprintf("%.8X", passFp),
					},
				}
				sel, ok := fpChoice.Choose(ctx, th)
				if !ok {
					continue
				}
				if sel == 1 {
					mfp = passFp
				}
			}
		}
		plate, err := engraveSeed(ctx.Platform.EngraverParams(), mnemonic, mfp)
		if err != nil {
			showSeedError(ctx, th, ss, mnemonic, err)
			continue
		}
		if NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme) {
			return
		}
	}
}
```

(If the current `backupWalletFlow` references a `qaProgram`/`backupWallet` branch or anything beyond the `for{Confirm; engraveSeed; Engrave}` + inline error loop, preserve it — but per the recon the body is exactly that shape; verify against the live source before replacing.)

- [ ] **Step 8: Add the fingerprint-choice test** — append to `gui/gui_test.go`

```go
func TestEngraveFingerprintChoiceMapping(t *testing.T) {
	// The fingerprint choice maps index 0 → bare, index 1 → passphrase.
	mn := bip39FromWords(t, "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about")
	bare, _ := masterFingerprintFor(mn, &chaincfg.MainNetParams, "")
	pass, _ := masterFingerprintFor(mn, &chaincfg.MainNetParams, "x")
	if bare == pass {
		t.Fatal("fingerprints unexpectedly equal")
	}
	// Drive a 2-row ChoiceScreen selecting index 1 (Down + Button3) and assert it returns 1.
	ctx := NewContext(newPlatform())
	cs := &ChoiceScreen{Title: "Engrave fingerprint", Choices: []string{
		"No passphrase " + fmt.Sprintf("%.8X", bare),
		"Passphrase " + fmt.Sprintf("%.8X", pass),
	}}
	click(&ctx.Router, Down, Button3)
	sel, ok := cs.Choose(ctx, &descriptorTheme)
	if !ok || sel != 1 {
		t.Errorf("Choose = (%d,%v), want (1,true)", sel, ok)
	}
}
```

> This asserts the choice-screen mechanism + the index→fingerprint mapping the rewritten `backupWalletFlow` relies on (driving the full `Confirm→…→engrave` E2E is heavy; this + `TestPassphraseFlow` + Task-1's threading test + the unchanged bare-path golden cover the slice). If `bip39FromWords` was added in Task 1, reuse it.

- [ ] **Step 9: Run the full suite + vet** — `/home/bcg/.local/go/bin/go test ./...` then `/home/bcg/.local/go/bin/go vet ./gui/...`
Expected: PASS — new tests + ALL guards (`TestWordKeyboardScreen`, `TestInputSeedCodex32`, SLIP-39, codex32, the passphrase-keyboard tests, and the bare-fp `backup` golden `TestSeed*`). vet clean (modulo the pre-existing `gui/op/draw_test.go` go1.26 note). gofmt clean.

- [ ] **Step 10: Commit**
```bash
git add gui/gui.go gui/gui_test.go
git commit -S -s -m "gui: optional passphrase entry + fingerprint-engrave choice in backup flow"
```

---

## Final: whole-diff adversarial execution review (mandatory)

Independent opus review over the whole diff vs `e990f0b`. Persist to `design/agent-reports/seedhammer-passphrase-flow-execution-review.md`; fold to clean.

Focus: the threading is complete (every `deriveMasterKey`/`masterFingerprintFor` caller updated; `Confirm` validity check stays `""`; `engraveSeed` sole caller updated); the no-passphrase path is byte-identical to today (bare fp, same engrave) — confirm the `backup` golden `TestSeed*` unchanged + no golden regen; both fingerprints genuinely differ for a non-empty passphrase; the `backupWalletFlow` control flow has exactly one engrave path per iteration, terminates, and the back-semantics match the spec (Skip≡Back≡bare; fp-Back→re-Confirm; empty→bare); `passphraseFlow` returns correctly (OK/Back/empty); the passphrase string never reaches `backup.Seed`/engrave/NFC; `fmt` is the only new import and is used; the fingerprint-choice labels fit the display (M-4 — verify or shorten); `frontSideSeed`/`backup.Seed`/`bip39`/`bip32`/`PassphraseKeyboard` untouched; signed+DCO.

Then **superpowers:finishing-a-development-branch** — no upstream PR: merge `feat/passphrase-flow` into fork `main` (no-ff, signed), push to `bg002h`.

---

## Self-Review (author)

- **Spec coverage:** S1 threading → Task 1; S2 `passphraseFlow` + S3 `backupWalletFlow`/`showSeedError`/fingerprint-choice → Task 2. The §8 resolved decisions (optional post-Confirm step default-skip; 2-row fingerprint ChoiceScreen bare-first; empty→skip; only `MasterFingerprint` changes; passphrase never engraved) all realized.
- **R0/R1 folds honored:** real `showSeedError` (not phantom); fresh `ChoiceScreen` per iteration; `"fmt"` import added; complete compilable `backupWalletFlow` (no invalid switch); explicit back-semantics; the `Confirm` validity check keeps `""`.
- **Type consistency:** `deriveMasterKey(m,net,password)`, `masterFingerprintFor(m,net,password)`, `engraveSeed(params,m,mfp)`, `passphraseFlow(ctx,th)→(string,bool)`, `showSeedError(ctx,th,ss,m,err)`, `ChoiceScreen.Choose→(int,bool)`, `fmt.Sprintf("%.8X",…)` all match the R1-verified spec + the extracted current bodies.
- **Atomicity:** Task 1 compiles standalone (all callers pass `""`, behavior unchanged); Task 2 is atomic (the `engraveSeed` signature change + its caller + the new helpers land together). The implementer must verify the live `backupWalletFlow` body before the wholesale replace (Step 7) and reuse/add `bip39FromWords` once.
