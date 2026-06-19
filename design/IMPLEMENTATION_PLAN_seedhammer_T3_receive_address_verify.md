# T3 — receive-address verification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`. TDD: test → red → implement → green → commit.

**Goal:** Confirm "does THIS descriptor control THIS address?" — input a candidate address (NFC scan OR typed), gap-limit-scan the descriptor's receive+change ranges, report match + chain + index. Read-only.

**Architecture:** A headless `address.Find` (the load-bearing, panic-safe match core) + a GUI verify flow off `DescriptorScreen` (Show-addresses / Verify-address `ChoiceScreen` → Scan or Type → "Verifying…" → result). Candidate normalized + validated by the in-tree `btcd/address/v2.DecodeAddress`; compared by canonical string to `address.Receive`/`Change` output.

**Tech stack:** Go (host test) / TinyGo. Deps already in `address` (btcd `address/v2`, `chaincfg/v2`). Go: `/home/bcg/.local/go/bin/go` (go1.26.4).

**Spec:** `design/SPEC_seedhammer_T3_receive_address_verify.md` (GREEN, `f654d63`). **Base:** fork `d334861`. Input method = **Both (scan + typed)** (user choice).

---

## Source-of-truth facts (R0/R1-verified vs `d334861`)
- `address.Receive/Change(desc,i) (string,error)` return canonical `addr.String()`; `Supported` = `!errors.Is(Receive(desc,0), errUnsupported)`. **`addressAt`/`Receive`/`Supported` PANIC on a keyless descriptor** (`desc.Keys[0]`, no guard) → `Find` MUST guard `len(desc.Keys)==0` first.
- Within package `address`, the btcd parser is `address.DecodeAddress(addr, net) (Address,error)` (the pkg imports `btcd/address/v2` under its own name); `Address.String()`/`IsForNet`. `DecodeAddress(...).String()` canonicalizes (lowercases uppercase bech32) → matches `Receive`/`Change`.
- `derivePubKey`: default children `<0;1>/*`; `RangeDerivation` requires `End==Index+1` (else error → propagate); a range/wildcard-less path derives the SAME address for all `(chain,index)` (degenerate → report `(0,0)`).
- `DescriptorScreen.Confirm` (`gui.go:2348`): `backBtn=B1/addrBtn=B2/confirmBtn=B3`; `supported := address.Supported(desc)` HOISTED (alloc gate); the `addrBtn.Clicked(ctx) && supported` branch (currently → `descriptorAddressFlow`) is the ChoiceScreen insertion point; the per-frame `[]NavButton{...}` is a FIXED literal (must stay 0-alloc). No Button4 exists.
- `descriptorAddressFlow` (`gui/address_polish.go`): measure-and-advance display idiom (`ctx.Styles.body.Measure`, page by `shown`); `const addrMaxIndex = 49`.
- `passphraseFlow` (`gui.go:495`): keyboard-driver loop (`NewPassphraseKeyboard`, `for kbd.Update(ctx)`, B1 back / B3 ok, `kbd.Layout`, returns `kbd.Fragment`). `PassphraseKeyboard.Fragment` is case-preserved (NO ToUpper); `Layout` MASKS with `*` unless `revealed` (`passphrase_keyboard.go:341-344`) — must unmask for a public address.
- `scanner.Scan` (`gui/scan.go`) dispatch chain ends `else { return nil, errScanUnknownFormat }`; add a `DecodeAddress` branch + `addressText` type. Scanner-shell idiom: `mk1GatherFlow` (`gui/mk1_inspect.go:156`).
- Test harness: `runUI`/`(*op.Drawer).ExtractText`/`uiContains`, `click`/`press`/`runes` (take `*EventRouter` → `&ctx.Router`), `testPlatform.NFCReader()==nil` (scan goroutine never starts in tests → test via `Find` headless + `runVerify`-with-candidate + keyboard `runes` drive + headless `scanner.Scan`), alloc gate = `StartScreen.Flow`+`DescriptorScreen.Confirm` only.

---

## File manifest
- **Modify** `address/address.go` (+ `address_test.go`) — `Find`, sentinels `ErrUnsupported`/`ErrAddrUnparseable`/`ErrAddrWrongNetwork`, const `addrFindMaxGap`.
- **Create** `gui/verify_address.go` (+ `verify_address_test.go`) — `verifyAddressFlow`, `runVerify`, result screen, the Scan/Type input flows, the unmasked address keyboard.
- **Modify** `gui/passphrase_keyboard.go` — an unmasked-keyboard affordance (a `revealed`-true constructor/variant for public addresses).
- **Modify** `gui/scan.go` (+ test) — `DecodeAddress` recognition branch + `addressText` type.
- **Modify** `gui/gui.go` (`DescriptorScreen.Confirm`) — the Show/Verify `ChoiceScreen` in the `addrBtn` click branch.

---

## Task 0: Worktree + baseline
- [ ] **Step 1:** `cd /scratch/code/shibboleth/seedhammer && git worktree add -b feat/verify-address ../seedhammer-wt-t3-verify d334861 && cd ../seedhammer-wt-t3-verify`
- [ ] **Step 2:** `/home/bcg/.local/go/bin/go test ./address/ ./gui/` → PASS (baseline).

---

## Task 1: `address.Find` (headless core — load-bearing)

**Files:** Modify `address/address.go`, `address/address_test.go`.

- [ ] **Step 1: Write failing tests** — append to `address/address_test.go` (reuse the file's `xpubs` + `nonstandard.OutputDescriptor` pattern; addresses verbatim from the existing table):
```go
func TestFind(t *testing.T) {
	wpkh, err := nonstandard.OutputDescriptor([]byte("wpkh(" + xpubs[0] + ")"))
	if err != nil { t.Fatal(err) }
	multi, err := nonstandard.OutputDescriptor([]byte("wsh(sortedmulti(1," + xpubs[0] + "/1234/<5;6>/*))"))
	if err != nil { t.Fatal(err) }

	cases := []struct {
		name        string
		desc        string // built below
		cand        string
		wantChain   int
		wantIndex   uint32
		wantFound   bool
		wantErrIs   error // nil = no error
	}{
		{"wpkh receive[2]", "wpkh", "bc1qkwl5qpx6k93cqmnygn6kgucgka8q3z4kur2nm8", 0, 2, true, nil},
		{"wpkh change[1]", "wpkh", "bc1qvwlscfgdmtkna074wylrvqly4w6nlpklsmyx7x", 1, 1, true, nil},
		{"wpkh foreign", "wpkh", "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4", 0, 0, false, nil},
		{"multi receive[0]", "multi", "bc1qt77623mmw4lnsewlmt9cs60yvxpwks540ygtzkakdf8xaa4ahsvqcma0k0", 0, 0, true, nil},
		{"multi change[1]", "multi", "bc1qwh9lhlgx9an4kz3s9qtrfm3xyvms84lkjy4paflg408vswjq4zcqx2xzlp", 1, 1, true, nil},
		{"unparseable", "wpkh", "not-an-address", 0, 0, false, ErrAddrUnparseable},
		{"wrong network (testnet bech32)", "wpkh", "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx", 0, 0, false, ErrAddrWrongNetwork},
	}
	pick := func(n string) *bip380.Descriptor { if n == "multi" { return multi }; return wpkh }
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			ch, idx, found, err := Find(pick(c.desc), c.cand, 20)
			if c.wantErrIs != nil {
				if !errors.Is(err, c.wantErrIs) { t.Fatalf("err=%v want %v", err, c.wantErrIs) }
				return
			}
			if err != nil { t.Fatalf("unexpected err: %v", err) }
			if found != c.wantFound || (found && (ch != c.wantChain || idx != c.wantIndex)) {
				t.Fatalf("got (chain=%d idx=%d found=%v) want (%d %d %v)", ch, idx, found, c.wantChain, c.wantIndex, c.wantFound)
			}
		})
	}
}

func TestFindKeylessNoPanic(t *testing.T) { // R0-I1: must error, never panic
	defer func() { if r := recover(); r != nil { t.Fatalf("Find panicked on keyless descriptor: %v", r) } }()
	keyless := &bip380.Descriptor{Type: bip380.Singlesig, Script: bip380.P2WPKH} // no Keys
	_, _, found, err := Find(keyless, "bc1qkwl5qpx6k93cqmnygn6kgucgka8q3z4kur2nm8", 20)
	if found || !errors.Is(err, ErrUnsupported) { t.Fatalf("keyless: found=%v err=%v want ErrUnsupported", found, err) }
}

func TestFindPropagatesDerivationError(t *testing.T) { // R0-I2/§2.1b: don't swallow as a non-match
	// A range element where End != Index+1 makes derivePubKey return
	// "unsupported range path element"; Find must propagate it, not silently
	// compare "" == want and report not-found.
	desc, err := nonstandard.OutputDescriptor([]byte("wpkh(" + xpubs[0] + "/1234/<5;7>/*)"))
	if err != nil { t.Fatal(err) }
	_, _, found, ferr := Find(desc, "bc1qkwl5qpx6k93cqmnygn6kgucgka8q3z4kur2nm8", 20)
	if found || ferr == nil {
		t.Fatalf("want propagated derivation error, got found=%v err=%v", found, ferr)
	}
}
```
(`errors` is already imported by `address_test.go`; if not, add it. `nonstandard`/`bip380` already used. **R0-M1:** the `xpubs` slice is function-local to `TestAddresses` — HOIST it to a package-level `var xpubs = []string{...}` so `TestFind`/`TestFindPropagatesDerivationError` can reference it.)
- [ ] **Step 2: Run — expect FAIL** (`Find`/`ErrUnsupported` undefined): `/home/bcg/.local/go/bin/go test ./address/ -run TestFind 2>&1 | tail`
- [ ] **Step 3: Implement** — add to `address/address.go`:
```go
// Errors returned by Find. ErrUnsupported is the exported counterpart to the
// internal errUnsupported, returned for a keyless descriptor (which addressAt
// would otherwise panic on).
var (
	ErrUnsupported      = errors.New("address: unsupported descriptor")
	ErrAddrUnparseable  = errors.New("address: candidate is not a valid address")
	ErrAddrWrongNetwork = errors.New("address: candidate is for a different network")
)

// addrFindMaxGap bounds the per-chain gap scan (mirrors gui.addrMaxIndex+1;
// defined here because package address cannot import package gui).
const addrFindMaxGap uint32 = 50

// Find scans the descriptor's receive then change ranges [0,gap) for an address
// equal to candidate. chain is 0 (receive) or 1 (change). Panic-safe / total:
// returns a typed error (never panics) for a keyless/unsupported descriptor, an
// unparseable candidate, or a wrong-network candidate; propagates any per-index
// derivation error rather than masking it as a non-match.
func Find(desc *bip380.Descriptor, candidate string, gap uint32) (chain int, index uint32, found bool, err error) {
	if len(desc.Keys) == 0 { // R0-I1: guard before desc.Keys[0]/Supported (both panic on keyless).
		return 0, 0, false, ErrUnsupported
	}
	if gap == 0 || gap > addrFindMaxGap {
		gap = addrFindMaxGap
	}
	net := desc.Keys[0].Network
	want, derr := DecodeAddress(candidate, net)
	if derr != nil {
		return 0, 0, false, ErrAddrUnparseable
	}
	if !want.IsForNet(net) {
		return 0, 0, false, ErrAddrWrongNetwork
	}
	wantStr := want.String()
	for i := uint32(0); i < gap; i++ {
		got, e := Receive(desc, i) // R0-I2: propagate, don't compare "" silently.
		if e != nil {
			return 0, 0, false, e
		}
		if got == wantStr {
			return 0, i, true, nil
		}
	}
	for i := uint32(0); i < gap; i++ {
		got, e := Change(desc, i)
		if e != nil {
			return 0, 0, false, e
		}
		if got == wantStr {
			return 1, i, true, nil
		}
	}
	return 0, 0, false, nil
}
```
- [ ] **Step 4: Run — expect PASS:** `/home/bcg/.local/go/bin/go test ./address/ -run TestFind -v`
- [ ] **Step 5: Commit:**
```bash
git add address/address.go address/address_test.go
git -c commit.gpgsign=true commit -S -s --author="Brian Goss <goss.brian@gmail.com>" \
  -m "address: Find — gap-limit receive/change address match (panic-safe) (T3)" \
  -m "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```
(All task commits use this signed + DCO + author + trailer form.)

---

## Task 2: GUI verify flow + result + DescriptorScreen wiring

**Files:** Create `gui/verify_address.go`, `gui/verify_address_test.go`; modify `gui/gui.go`.

- [ ] **Step 1: Write failing tests** — `gui/verify_address_test.go`:
```go
package gui

import (
	"strings"
	"testing"
)

func TestRunVerifyResult(t *testing.T) {
	desc := loadTestDesc(t, descWPKH) // address_polish_test.go helper + const
	// Drive runVerify directly with a candidate (bypasses NFC, NFCReader()==nil).
	cases := []struct{ name, cand, want string }{
		{"match receive", "bc1qkwl5qpx6k93cqmnygn6kgucgka8q3z4kur2nm8", "Receive"},
		{"not found", "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4", "Not found"},
		{"invalid", "not-an-address", "Invalid"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			ctx := NewContext(newPlatform())
			frame, quit := runUI(ctx, func() { runVerify(ctx, &descriptorTheme, desc, c.cand) })
			defer quit() // abandons the result loop; do NOT click Back per-frame (it would dismiss before the result renders — R0-C1)
			var all strings.Builder
			for i := 0; i < 4; i++ { // frame 1 = "Verifying…", frame 2+ = result (loops until Back)
				content, ok := frame()
				if !ok { break }
				all.WriteString(content)
			}
			if !uiContains(all.String(), c.want) {
				t.Errorf("verify(%q): want %q; got %q", c.cand, c.want, all.String())
			}
		})
	}
}
```
- [ ] **Step 2: Run — expect FAIL** (`runVerify` undefined): `/home/bcg/.local/go/bin/go test ./gui/ -run TestRunVerifyResult 2>&1 | tail`
- [ ] **Step 3a: Implement** `gui/verify_address.go` (the result core; Scan/Type input added in Task 3). Self-contained — mirrors `descriptorAddressFlow`'s nav/measure idiom; NO `showMessage` helper (it doesn't exist, and a blocking `showError`-style modal as the "Verifying…" frame would hang and never run `Find` — R0-C1/I1):
```go
package gui

import (
	"errors"
	"fmt"
	"image"

	"seedhammer.com/address"
	"seedhammer.com/bip380"
	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
)

// runVerify shows a ONE-SHOT non-blocking "Verifying…" frame, runs address.Find
// ONCE (outside any loop — a multisig gap scan can block for seconds on RP2350,
// R0-M3), then displays the result in a Back-able screen. Read-only: no
// engrave/NFC/mutation. The "Verifying…" frame is a single ctx.Frame (NOT a
// blocking showError-style modal — R0-C1), so Find actually runs after it.
func runVerify(ctx *Context, th *Colors, desc *bip380.Descriptor, candidate string) {
	dims := ctx.Platform.DisplaySize()
	{ // one-shot progress frame, then compute
		title, _ := layoutTitle(ctx, dims.X, th.Text, "Verifying…")
		ctx.Frame(op.Layer(title, op.Color(&ctx.B, th.Background)))
	}
	chain, index, found, err := address.Find(desc, candidate, 20)
	var body string
	switch {
	case errors.Is(err, address.ErrAddrUnparseable):
		body = "Invalid address."
	case errors.Is(err, address.ErrAddrWrongNetwork):
		body = "Address is for a different network."
	case err != nil:
		body = "Can't verify this address."
	case !found:
		body = "Not found in the first 20 receive or change addresses."
	default:
		chainName := "receive"
		if chain == 1 {
			chainName = "change"
		}
		// Degenerate range/wildcard-less descriptor reports (0,0); phrase plainly.
		body = fmt.Sprintf("Match: %s address #%d. Controlled by this descriptor.", chainName, index)
	}
	// Back-able result screen (mirror descriptorAddressFlow's nav/render).
	backBtn := &Clickable{Button: Button1}
	lineWidth := dims.X - 2*8
	for !ctx.Done {
		if backBtn.Clicked(ctx) {
			return
		}
		lbl, sz := widget.Labelw(&ctx.B, ctx.Styles.body, lineWidth, th.Text, body)
		bodyOp := lbl.Offset(image.Pt((dims.X-sz.X)/2, leadingSize+16))
		title, _ := layoutTitle(ctx, dims.X, th.Text, "Verify address")
		nav, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack},
		}...)
		ctx.Frame(op.Layer(nav, title, bodyOp, op.Color(&ctx.B, th.Background)))
	}
}
```
- [ ] **Step 3b: Modify `DescriptorScreen.Confirm`** in `gui/gui.go` — replace the `addrBtn.Clicked` branch:
```go
		if addrBtn.Clicked(ctx) && supported {
			descriptorAddressFlow(ctx, th, s.Descriptor)
			continue
		}
```
with a Show/Verify choice (allocates only inside the click branch — per-frame layout untouched, 0-alloc gate safe):
```go
		if addrBtn.Clicked(ctx) && supported {
			cs := &ChoiceScreen{Title: "Addresses", Lead: "Choose", Choices: []string{"Show addresses", "Verify an address"}}
			switch choice, ok := cs.Choose(ctx, th); {
			case !ok:
			case choice == 0:
				descriptorAddressFlow(ctx, th, s.Descriptor)
			default:
				verifyAddressFlow(ctx, th, s.Descriptor)
			}
			continue
		}
```
- [ ] **Step 3c:** add a temporary stub `func verifyAddressFlow(ctx *Context, th *Colors, desc *bip380.Descriptor) { runVerify(ctx, th, desc, "") }` in `gui/verify_address.go` so the build compiles this task; Task 3 replaces it with the real Scan/Type flow.
- [ ] **Step 3d: UPDATE the existing T1 test `TestDescriptorConfirmAddressAffordance`** (`gui/address_polish_test.go`, R0-I2). The Show/Verify `ChoiceScreen` now interposes one selection before the address view: the test drives `click(Button2)` and expects the address view directly — it MUST be updated to also select "Show addresses" in the ChoiceScreen (drive the ChoiceScreen's choose button, e.g. `click(Button3)` for the default choice 0), then assert the address view opens. Run `go test ./gui/ -run TestDescriptorConfirmAddressAffordance` → PASS. (This reconciles spec §2.6: the show-addresses path is preserved but now reached via the Button2 Show/Verify choice — one extra step, intended per §4.2.)
- [ ] **Step 4: Run — expect PASS** + build: `/home/bcg/.local/go/bin/go build ./... && /home/bcg/.local/go/bin/go test ./gui/ -run 'TestRunVerifyResult|TestAllocs' -v`
- [ ] **Step 5: Commit:** `git add gui/verify_address.go gui/verify_address_test.go gui/gui.go` then signed commit `gui: verify-address result flow + DescriptorScreen Show/Verify choice (T3)`.

---

## Task 3: Address input — typed keyboard + scan recognizer

**Files:** modify `gui/passphrase_keyboard.go`, `gui/scan.go`, `gui/verify_address.go`; append to `gui/verify_address_test.go`, add `gui/scan_test.go` if absent.

- [ ] **Step 1: Write failing tests** — append to `gui/verify_address_test.go`:
```go
func TestTypeAddressCasePreserved(t *testing.T) {
	ctx := NewContext(newPlatform())
	var got string
	var ok bool
	frame, quit := runUI(ctx, func() { got, ok = typeAddressFlow(ctx, &descriptorTheme) })
	defer quit()
	frame()
	runes(&ctx.Router, "bc1Q3") // mixed case must be preserved (NOT uppercased)
	frame()
	click(&ctx.Router, Button3) // OK
	frame()
	if !ok || got != "bc1Q3" {
		t.Fatalf("typed = %q ok=%v; want bc1Q3 (case preserved)", got, ok)
	}
}
```
And a scan-recognizer test in `gui/scan_test.go` (R0-M2: `Scan` returns `errScanInProgress` on the first data-available read, so DRIVE IT IN A LOOP until a non-in-progress result; use the real `tvXpub` from `address_polish_test.go` for the negative case, not a placeholder):
```go
func scanOnce(t *testing.T, s string) (any, error) {
	t.Helper()
	sc := new(scanner)
	r := strings.NewReader(s)
	for {
		obj, err := sc.Scan(r)
		if errors.Is(err, errScanInProgress) {
			continue
		}
		return obj, err
	}
}

func TestScanRecognizesAddress(t *testing.T) {
	obj, err := scanOnce(t, "bc1qkwl5qpx6k93cqmnygn6kgucgka8q3z4kur2nm8")
	if err != nil { t.Fatalf("scan: %v", err) }
	if _, isAddr := obj.(addressText); !isAddr {
		t.Fatalf("scanned object = %T, want addressText", obj)
	}
	// A descriptor must NOT be misrecognized as an address (address branch is
	// AFTER the descriptor probe in the dispatch chain).
	obj2, _ := scanOnce(t, "wpkh("+tvXpub+")")
	if _, isAddr := obj2.(addressText); isAddr {
		t.Fatal("descriptor misrecognized as addressText")
	}
}
```
- [ ] **Step 2: Run — expect FAIL** (`typeAddressFlow`/`addressText` undefined).
- [ ] **Step 3a: Unmasked keyboard** — `gui/passphrase_keyboard.go`: add `func NewAddressKeyboard(ctx *Context) *PassphraseKeyboard { k := NewPassphraseKeyboard(ctx); k.revealed = true; return k }`. **The `k.revealed = true` MUST come AFTER `NewPassphraseKeyboard` (R0-M4)** — its trailing `Clear()` sets `revealed=false` (`passphrase_keyboard.go:164`), so setting it in/before construction would be reset. The `Layout` masking branch (`:341-344`) then shows `k.Fragment` directly. (Minimal change; `PassphraseKeyboard` otherwise intact → `passphraseFlow` unaffected.)
- [ ] **Step 3b: `typeAddressFlow`** in `gui/verify_address.go` — mirror `passphraseFlow` (B1 back / B3 ok, `for kbd.Update(ctx)`, `kbd.Layout`) but with `NewAddressKeyboard`, title "Enter address", returning `(kbd.Fragment, ok)`. (Validity is checked downstream by `address.Find` → "Invalid address"; optionally gate OK on `DecodeAddress` parsing for live feedback.)
- [ ] **Step 3c: scan recognizer** — `gui/scan.go`: add `type addressText string` and, in `Scan`'s dispatch chain (AFTER the descriptor/codex32 probes, before the final `else`), a branch. **(R0-M3: in package `gui` the name `address` already refers to `seedhammer.com/address` (`gui.go:23`), which has NO `DecodeAddress` — so import the btcd parser under an ALIAS:** `btcaddr "github.com/btcsuite/btcd/address/v2"` + `"github.com/btcsuite/btcd/chaincfg/v2"`.)
```go
	} else if _, aerr := btcaddr.DecodeAddress(string(buf), &chaincfg.MainNetParams); aerr == nil {
		return addressText(buf), nil
	} else if _, aerr := btcaddr.DecodeAddress(string(buf), &chaincfg.TestNet3Params); aerr == nil {
		return addressText(buf), nil
```
(Mainnet then testnet; the descriptor's own network re-check in `address.Find` remains authoritative.) **`engraveObjectFlow` gets NO `addressText` case (R0-M5)** → a top-level address scan falls to `default → unknown format` (unchanged). The `addressText` value is consumed only by the verify flow's scanner-shell.
- [ ] **Step 3d: real `verifyAddressFlow`** — replace the Task-2 stub: a `ChoiceScreen` "Scan" / "Type"; Scan → `scanAddressFlow` (the `mk1GatherFlow` scanner-shell idiom, accepting the first `addressText`, Back exits) → candidate; Type → `typeAddressFlow` → candidate; then `runVerify(ctx, th, desc, candidate)`.
- [ ] **Step 4: Run — expect PASS** + no regress:
```
/home/bcg/.local/go/bin/go build ./...
/home/bcg/.local/go/bin/go test ./gui/ ./address/ -v 2>&1 | tail -40
/home/bcg/.local/go/bin/go test -run TestAllocs ./gui/
/home/bcg/.local/go/bin/gofmt -l address/ gui/
```
- [ ] **Step 5: Commit:** `git add gui/passphrase_keyboard.go gui/scan.go gui/verify_address.go gui/verify_address_test.go gui/scan_test.go` then signed commit `gui: address input for verify — unmasked typed keyboard + scan recognizer (T3)`.

---

## Task 4: Full verification
- [ ] **Step 1:** `/home/bcg/.local/go/bin/go test ./... && /home/bcg/.local/go/bin/go vet ./address/ ./gui/ && /home/bcg/.local/go/bin/gofmt -l address/ gui/` (empty) and `/home/bcg/.local/go/bin/go test -count=1 -run TestAllocs ./gui/` (PASS).
- [ ] **Step 2 (CI):** TinyGo firmware build (`./cmd/controller`) compiles `address`+`gui` — confirm in CI (pure derivation + existing widgets; no new heavy deps).

---

## Done criteria
- `address.Find`: correct receive/change match (singlesig+multisig), no false positive, no false negative in [0,20); keyless → `ErrUnsupported` no panic; derivation errors propagated; unparseable/wrong-network → typed errors.
- Verify flow: Show/Verify choice; Scan + Type both feed `runVerify`; result shows chain/index match, not-found, invalid, or wrong-network; "Verifying…" frame before the scan; typed entry case-preserved + unmasked.
- No regression: descriptor show-addresses + engrave unchanged; `engraveObjectFlow` no `addressText` case; alloc gate passes; vet/gofmt clean.

## Self-review (vs spec)
- §2.1/2.1a/2.1b Find correctness+panic-safe+error-prop → Task 1 (`TestFind`/`TestFindKeylessNoPanic` + the error-propagation path). §2.2 canonical compare → `DecodeAddress(...).String()` vs `Receive/Change`. §2.3 read-only → Tasks 2/3 (no engrave/NFC-write). §2.4 case-preserving → Task 3 (`TestTypeAddressCasePreserved`, unmasked). §2.5 alloc gate → ChoiceScreen inside click branch; `TestAllocs`. §2.6 no regression → show-addresses preserved as choice 0; `engraveObjectFlow` no addressText case. §2.7 bounded gap → `addrFindMaxGap`/20. §2.8 no secrets → public, no scrub. R0-M2 degenerate path → `runVerify` phrasing. R0-M3 Verifying frame → `runVerify`. R0-M5 → scan §3c note.
- Type names consistent (`address.Find`, `address.ErrUnsupported`/`ErrAddrUnparseable`/`ErrAddrWrongNetwork`, `addrFindMaxGap`, `verifyAddressFlow`/`runVerify`/`typeAddressFlow`/`scanAddressFlow`, `addressText`, `NewAddressKeyboard`).
- **R0 gate next:** opus-architect, materialize + build/run (`address.Find` parity + typed case-preservation are the proofs). Fold → persist verbatim → re-dispatch until GREEN.
