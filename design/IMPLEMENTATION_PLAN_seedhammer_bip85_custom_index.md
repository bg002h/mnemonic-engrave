# BIP-85 custom (typed) hardened index — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the bounded `0..9` BIP-85 child-index `ChoiceScreen` with a typed decimal index entry over the full hardened range `0 .. 2^31-1`, with a width-safe validator AND a defense-in-depth upper-bound guard inside `deriveBip85Child` that closes the silent `uint32`-overflow → unhardened-element bug on the 64-bit host.

**Architecture:** Firmware-only, all in `gui/bip85.go` + `gui/bip85_test.go`. Add `parseBip85Index` (parses via `strconv.ParseUint(s,10,64)`, rejects `>2147483647`, accepts leading zeros). Clone `typeAddressFlow` → `bip85IndexEntryFlow` (reuses `NewAddressKeyboard`, error+re-prompt on parse failure). Harden `deriveBip85Child` with an upper-bound guard before the `uint32(index)+h` cast. Wire the flow into `bip85ParamPickFlow`, retiring `bip85IndexChoices`. App/lang/word-count pickers unchanged; output (child words + SeedQR + child bare fp) unchanged.

**Tech Stack:** Go 1.26 (host `amd64`, `strconv.IntSize==64` — where the truncation bug is live), `testing/synctest`, `seedhammer.com/{bip39,bip85}`, btcd `hdkeychain`. Run Go via `export PATH=$PATH:/home/bcg/.local/go/bin`.

---

## Verified facts (do not re-derive)

All cited LIVE against fork `/scratch/code/shibboleth/seedhammer`, branch `main`, HEAD `8459654` (`84596549228116ac`). Re-anchor any line that has drifted at impl time before editing.

- **Host is 64-bit (`go env GOARCH` → `amd64`).** The silent truncation is LIVE here — REPRODUCED at this HEAD: `deriveBip85Child(abandonAboutMnemonic(),"",12,1<<31)` and `…,1<<31+1` both return `err=nil` with a non-nil child (an UNHARDENED element, off-spec, no error). This is the bug Task 2 fixes.
- **High-index golden (PROBE-VERIFIED at HEAD `8459654`, two independent paths):**
  - In-tree `deriveBip85Child(abandonAboutMnemonic(),"",12,2147483647)` → `jewel solution patient quarter elite grace quarter dinosaur taste parent dial clump`.
  - biptool (`bip32.ParsePath`, SEPARATE code path): master xprv `xprv9s21ZrQH143K3GJpoapnV8SFfukcVBSfeCficPSGfubmSFDxo1kuHnLisriDvSnRRuL2Qrg5ggqHKNVpxR86QEC8w35uxmGoggxtQTPvfUu`, `m/83696968h/39h/0h/12h/2147483647h` → byte-identical to the above. `…/2147483648h` → `biptool: bip32: path element out of range`. **STALENESS GUARD: re-run this probe at impl time (Task 4 includes the exact command); a derive-library bump could move it.**
  - Index 0 → `prosper short ramp prepare exchange stove life snack client enough purpose fold`; index 1 → `sing slogan bar group gauge sphere rescue fossil loyal vital model desert` (both match the SHIPPED goldens `gui/bip85_test.go:31` / `:79` — the typed path does not change existing children).
- **BIP-85 / BIP-32 hardened max = `2^31-1` = `2147483647`** = `hdkeychain.HardenedKeyStart - 1`. `hdkeychain.HardenedKeyStart = 0x80000000 = 2147483648 = 2^31`. Cross-checked vs the authoritative `bip-0085.mediawiki` (index hardened, no sub-range beyond the BIP-32 max), `bip32.ParsePathElement`, and biptool's rejection of `2147483648h`.
- **`deriveBip85Child` truncation site** — `gui/bip85.go:32` signature `deriveBip85Child(m bip39.Mnemonic, passphrase string, words, index int) (bip39.Mnemonic, error)`; guards ONLY `index < 0` at `:36-38` (`fmt.Errorf("bip85: invalid index: %d", index)`); `uint32(index)+h` at `:54` is the cast — the upper-bound guard MUST precede it. `const h = hdkeychain.HardenedKeyStart` is at `:40`.
- **Keyboard API:** `typeAddressFlow` (`gui/verify_address.go:44-71`) — `kbd := NewAddressKeyboard(ctx)`; `backBtn := &Clickable{Button: Button1}`; `okBtn := &Clickable{Button: Button3}`; `for !ctx.Done { for kbd.Update(ctx) {} ; if backBtn.Clicked(ctx) { return "", false }; if okBtn.Clicked(ctx) { return kbd.Fragment, true } … ctx.Frame(…) }`. `NewAddressKeyboard(ctx) *PassphraseKeyboard` (`gui/passphrase_keyboard.go:133-137`) = revealed (cleartext; the index is public). `kbd.Fragment` (`:48`) is the typed string; the keyboard commits ANY rune (digits live on `ppPageSymbols`, `:21`) so **digit-only enforcement is the validator's job** (`commit` `:189-206`; cross-page rune handler `:247-258`).
- **Test idiom:** flow tests use `runUI(ctx, ui)` → `(frame func()(string,bool), quit func())` (`gui/gui_test.go:467`). Drive the keyboard via `runes(&ctx.Router, "…")` (`gui/event_test.go:68`) then `for kbd.Update` consumes them; press OK via `click(&ctx.Router, Button3)` and Back via `click(&ctx.Router, Button1)` (`gui/event_test.go:42`). `TestTypeAddressCasePreserved` (`gui/verify_address_test.go:36-50`) is the CLOSEST shipped keyboard-entry flow test — mirror it exactly. `synctest.Test(t, func(t){…})` wraps multi-frame flow tests (`gui/bip85_test.go:172`, `:207`).
- **m\*-free / no-lockstep:** `gui/bip85.go` imports only `errors`,`fmt`, btcd `hdkeychain`/`chaincfg`, `seedhammer.com/{bip39,bip85,engrave,gui/assets,gui/op}` — no `md`/`mk`/`codex32`. No new program → no enum / t5-M1 compile-guard / nav-test / 8-site lockstep edit; no `me` CLI flag / `schema_mirror` / `docs/manual` mirror / SemVer bump. This task adds `strconv` to the import set (Task 1).

---

## File structure

- **Modify** `gui/bip85.go`: add `import "strconv"`; add `parseBip85Index`; add `bip85IndexEntryFlow`; add the upper-bound guard in `deriveBip85Child`; rewrite the index stage of `bip85ParamPickFlow`; remove `var bip85IndexChoices`.
- **Modify** `gui/bip85_test.go`: add `parseBip85Index` unit tests; add `deriveBip85Child` upper-bound tests; add the `bip85IndexEntryFlow` synctest flow test; add the high-index derive golden + index-0-unchanged tests; re-pin `TestBip85ParamBounds`; re-point the index step of `TestBip85DeriveFlow_ScrubsBothMnemonics`; tighten `FuzzDeriveBip85Child`.

No other files. No new files.

---

## Task 0: Worktree + baseline

**Files:** none (setup only).

- [ ] **Step 1: Create the worktree off current `main`**

```bash
cd /scratch/code/shibboleth/seedhammer
git worktree add -b feat/bip85-custom-index ../seedhammer-bip85-custom-index main
cd ../seedhammer-bip85-custom-index
git rev-parse HEAD   # Expected: 84596549228116ac1600350c00f03be021ce4814 (or current main if it advanced)
```

Expected: `Preparing worktree (new branch 'feat/bip85-custom-index')`.

- [ ] **Step 2: Confirm Go + arch + baseline green**

```bash
cd /scratch/code/shibboleth/seedhammer-bip85-custom-index
export PATH=$PATH:/home/bcg/.local/go/bin
go version          # Expected: go1.26.x
go env GOARCH       # Expected: amd64 (64-bit host; the truncation bug is live here)
go test ./gui/... ./bip85/...
```

Expected: `ok seedhammer.com/gui`, `ok seedhammer.com/bip85` (assorted `[no test files]` lines are fine). NO failures.

> All subsequent commands run from `/scratch/code/shibboleth/seedhammer-bip85-custom-index` with `export PATH=$PATH:/home/bcg/.local/go/bin` already applied in the shell.

---

## Task 1: `parseBip85Index` validator + unit tests

**Files:**
- Modify: `gui/bip85.go` (add `import "strconv"`; add `parseBip85Index`)
- Test: `gui/bip85_test.go` (add `TestParseBip85Index`)

- [ ] **Step 1: Write the failing test**

Add to `gui/bip85_test.go` (append at end of file):

```go
// TestParseBip85Index pins the width-safe typed-index validator: it parses base-10
// via strconv.ParseUint (never a bare int), accepts leading zeros, and rejects
// anything > 2^31-1 (the BIP-85 hardened max), non-[0-9] runes, signs, whitespace,
// 0x, and empty input. The >2^31-1 reject is the validator's job, NOT a length cap
// (R0-M2): "9999999999" is 10 digits but still out of range.
func TestParseBip85Index(t *testing.T) {
	ok := []struct {
		in   string
		want int
	}{
		{"0", 0},
		{"7", 7},
		{"007", 7},          // leading zeros ACCEPTED (R0 adjudication #1)
		{"1000000", 1000000},
		{"2147483647", 2147483647}, // = 2^31-1, the boundary, ACCEPTED
	}
	for _, tc := range ok {
		got, err := parseBip85Index(tc.in)
		if err != nil {
			t.Fatalf("parseBip85Index(%q): unexpected error %v", tc.in, err)
		}
		if got != tc.want {
			t.Fatalf("parseBip85Index(%q) = %d, want %d", tc.in, got, tc.want)
		}
	}
	bad := []string{
		"",            // empty
		"12a",         // trailing letter
		"a12",         // leading letter
		"-1",          // sign
		"+1",          // sign
		" 1",          // leading whitespace
		"1 ",          // trailing whitespace
		"0x10",        // hex prefix
		"1.0",         // decimal point
		"2147483648",  // = 2^31, first out-of-range value
		"9999999999",  // 10 digits but > 2^31-1 (range, not length, is the authority)
		"9223372036854775808", // > 2^63, ParseUint(…,64) itself overflows
	}
	for _, in := range bad {
		if got, err := parseBip85Index(in); err == nil {
			t.Fatalf("parseBip85Index(%q) = %d, want an error", in, got)
		}
	}
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `go test ./gui/ -run TestParseBip85Index -v`
Expected: FAIL — `undefined: parseBip85Index` (compile error).

- [ ] **Step 3: Write minimal implementation**

In `gui/bip85.go`, add `strconv` to the import block (between `fmt` and the blank line before the btcd import):

```go
import (
	"errors"
	"fmt"
	"strconv"

	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip39"
	"seedhammer.com/bip85"
	"seedhammer.com/engrave"
	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/op"
)
```

Then add `parseBip85Index` immediately above `deriveBip85Child` (i.e. just before the `// deriveBip85Child re-creates …` doc comment at `gui/bip85.go:22`):

```go
// bip85MaxIndex is the BIP-85 / BIP-32 hardened-child ceiling: an un-hardened
// index in [0, 2^31-1]. It equals hdkeychain.HardenedKeyStart-1; biptool rejects
// anything >= HardenedKeyStart with "bip32: path element out of range".
const bip85MaxIndex = hdkeychain.HardenedKeyStart - 1 // = 2147483647 = 2^31-1

// parseBip85Index parses a typed decimal child index, WIDTH-SAFE on every target.
// It uses strconv.ParseUint(s,10,64) — NEVER a bare int — so a value > the 64-bit
// host's int is still caught, not wrapped. It rejects empty input, any non-[0-9]
// rune (sign, whitespace, '.', "0x", letters — all typeable on the keyboard), and
// any value > 2^31-1 (the hardened max). Leading zeros are accepted ("007" -> 7),
// matching base-10 ParseUint. The returned value is guaranteed in [0, 2^31-1], so
// it fits an int on every target.
func parseBip85Index(s string) (int, error) {
	if s == "" {
		return 0, errors.New("bip85: empty index")
	}
	v, err := strconv.ParseUint(s, 10, 64) // base 10; rejects sign/whitespace/0x/letters/overflow
	if err != nil {
		return 0, fmt.Errorf("bip85: invalid index %q", s)
	}
	if v > bip85MaxIndex {
		return 0, fmt.Errorf("bip85: index %s exceeds the maximum %d", s, bip85MaxIndex)
	}
	return int(v), nil // safe: v <= 2^31-1
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `go test ./gui/ -run TestParseBip85Index -v`
Expected: PASS — `--- PASS: TestParseBip85Index`.

- [ ] **Step 5: Build sanity**

Run: `go build ./...`
Expected: exit 0 (no output). Confirms `strconv` is used and the package still compiles.

- [ ] **Step 6: Commit**

```bash
git add gui/bip85.go gui/bip85_test.go
git commit -S -s -m "feat(bip85): width-safe typed-index validator parseBip85Index

Parse via strconv.ParseUint base-10 (never a bare int); reject empty,
non-[0-9], sign/whitespace/0x, and > 2^31-1 (the BIP-85 hardened max).
Accept leading zeros. The range reject is the authority, not a length cap.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

> Confirm the commit is signed + DCO-trailed: `git log -1 --show-signature --format='%G? %an %ae'` (Expected: a good-sig marker `G`, author `Brian Goss`, and a `Signed-off-by:` line in the body via `-s`). If `git config user.name`/`user.email` are not already `Brian Goss <goss.brian@gmail.com>`, set them before committing.

---

## Task 2: `deriveBip85Child` upper-bound guard + tests

**Files:**
- Modify: `gui/bip85.go` (add the upper-bound guard in `deriveBip85Child`, before `uint32(index)+h`)
- Test: `gui/bip85_test.go` (add `TestDeriveBip85Child_RejectsHighIndex`, `TestDeriveBip85Child_HighIndexGolden`)

- [ ] **Step 1: Write the failing tests**

Add to `gui/bip85_test.go`:

```go
// TestDeriveBip85Child_RejectsHighIndex pins the defense-in-depth upper-bound
// guard: an index > 2^31-1 MUST error, never silently truncate. On this 64-bit
// host, 1<<31 and 1<<31+1 fit an int and would otherwise wrap through
// uint32(index)+h into an UNHARDENED element with no error (the R0-reproduced
// bug). The lower bound (-1) still errors with its distinct message (R0-M3).
func TestDeriveBip85Child_RejectsHighIndex(t *testing.T) {
	for _, idx := range []int{1 << 31, 1<<31 + 1} { // 2147483648, 2147483649
		if _, err := deriveBip85Child(abandonAboutMnemonic(), "", 12, idx); err == nil {
			t.Fatalf("index=%d: expected an error (silent uint32 truncation), got nil", idx)
		}
	}
	// Lower bound still errors (retained).
	if _, err := deriveBip85Child(abandonAboutMnemonic(), "", 12, -1); err == nil {
		t.Fatal("index=-1: expected an error, got nil")
	}
}

// TestDeriveBip85Child_HighIndexGolden pins the boundary child at index 2^31-1.
// PROBE-VERIFIED at HEAD 8459654 two independent ways (in-tree derive + biptool's
// bip32.ParsePath path); re-probe-verify at impl time (Task 4 has the command).
// Index 0 stays byte-unchanged vs the shipped golden (typed path is additive).
func TestDeriveBip85Child_HighIndexGolden(t *testing.T) {
	child, err := deriveBip85Child(abandonAboutMnemonic(), "", 12, 2147483647) // = 2^31-1
	if err != nil {
		t.Fatalf("index=2147483647: %v", err)
	}
	const want = "jewel solution patient quarter elite grace quarter dinosaur taste parent dial clump"
	if got := child.String(); got != want {
		t.Fatalf("high-index child mismatch:\n got %q\nwant %q", got, want)
	}
	if len(child) != 12 || !child.Valid() {
		t.Fatalf("high-index child: %d words, valid=%v", len(child), child.Valid())
	}
	// Index 0 unchanged vs the shipped golden.
	c0, err := deriveBip85Child(abandonAboutMnemonic(), "", 12, 0)
	if err != nil {
		t.Fatalf("index=0: %v", err)
	}
	const want0 = "prosper short ramp prepare exchange stove life snack client enough purpose fold"
	if got := c0.String(); got != want0 {
		t.Fatalf("index-0 child changed:\n got %q\nwant %q", got, want0)
	}
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `go test ./gui/ -run 'TestDeriveBip85Child_RejectsHighIndex|TestDeriveBip85Child_HighIndexGolden' -v`
Expected: `TestDeriveBip85Child_RejectsHighIndex` FAILS — `index=2147483648: expected an error …, got nil` (the unguarded truncation). `TestDeriveBip85Child_HighIndexGolden` PASSES already (the derive math is correct; only the guard is missing) — that is expected and fine.

- [ ] **Step 3: Write minimal implementation**

In `gui/bip85.go`, in `deriveBip85Child`, add the upper-bound guard immediately AFTER the existing `index < 0` guard (`gui/bip85.go:36-38`), i.e. before `const h = hdkeychain.HardenedKeyStart`:

```go
	if index < 0 {
		return nil, fmt.Errorf("bip85: invalid index: %d", index)
	}
	if index > bip85MaxIndex {
		// Defense-in-depth (independent of the picker's parseBip85Index): a 64-bit
		// host int > 2^31-1 would otherwise be silently truncated/wrapped by the
		// uint32(index)+h cast below into a different/UNHARDENED element. Reject it.
		return nil, fmt.Errorf("bip85: index %d exceeds the maximum %d", index, bip85MaxIndex)
	}
```

(The `uint32(index)+h` cast and the rest of the body are unchanged. `bip85MaxIndex` was defined in Task 1.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `go test ./gui/ -run 'TestDeriveBip85Child_RejectsHighIndex|TestDeriveBip85Child_HighIndexGolden|TestDeriveBip85Child_RejectsNegativeIndex' -v`
Expected: all PASS.

- [ ] **Step 5: Run the existing index-0 goldens (no regression)**

Run: `go test ./gui/ -run 'TestDeriveBip85Child_AbandonGoldens|TestDeriveBip85Child_CanonicalVector|TestDeriveBip85Child_IndexVaries' -v`
Expected: all PASS (index 0/1 children unchanged).

- [ ] **Step 6: Commit**

```bash
git add gui/bip85.go gui/bip85_test.go
git commit -S -s -m "fix(bip85): reject index > 2^31-1 in deriveBip85Child

Defense-in-depth guard before uint32(index)+h: on a 64-bit host an index
> 2^31-1 was silently truncated/wrapped into an unhardened element with no
error (off-spec child). Distinct message from the lower-bound guard.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: `bip85IndexEntryFlow` (keyboard clone) + flow test

**Files:**
- Modify: `gui/bip85.go` (add `bip85IndexEntryFlow`)
- Test: `gui/bip85_test.go` (add `TestBip85IndexEntryFlow`)

- [ ] **Step 1: Write the failing test**

Add to `gui/bip85_test.go` (mirrors `TestTypeAddressCasePreserved`, `gui/verify_address_test.go:36-50`, wrapped in `synctest` since it has an error-re-prompt cycle):

```go
// TestBip85IndexEntryFlow drives the typed child-index keyboard flow: a valid
// decimal returns (idx,true); a non-numeric/empty Fragment shows an error and
// RE-PROMPTS (no abort, no silent 0); Back returns (0,false). Mirrors
// TestTypeAddressCasePreserved (runes -> kbd.Update; Button3=OK, Button1=Back).
func TestBip85IndexEntryFlow(t *testing.T) {
	// Valid high index -> (2147483647, true).
	t.Run("valid_high_index", func(t *testing.T) {
		ctx := NewContext(newPlatform())
		var idx int
		var ok bool
		frame, quit := runUI(ctx, func() { idx, ok = bip85IndexEntryFlow(ctx, &descriptorTheme) })
		defer quit()
		frame()
		runes(&ctx.Router, "2147483647")
		frame()
		click(&ctx.Router, Button3) // OK
		frame()
		if !ok || idx != 2147483647 {
			t.Fatalf("typed 2147483647 -> (%d,%v); want (2147483647,true)", idx, ok)
		}
	})

	// Back from an empty keyboard -> (0,false).
	t.Run("back_exits", func(t *testing.T) {
		ctx := NewContext(newPlatform())
		var idx int
		var ok bool
		frame, quit := runUI(ctx, func() { idx, ok = bip85IndexEntryFlow(ctx, &descriptorTheme) })
		defer quit()
		frame()
		click(&ctx.Router, Button1) // Back
		frame()
		if ok || idx != 0 {
			t.Fatalf("Back -> (%d,%v); want (0,false)", idx, ok)
		}
	})

	// Non-numeric input on OK -> error + re-prompt (NOT a silent 0, NOT an abort).
	// After the error, the flow loops back to the keyboard; clearing the bad runes
	// and typing a valid index then succeeds.
	t.Run("nonnumeric_reprompts_then_valid", func(t *testing.T) {
		synctest.Test(t, func(t *testing.T) {
			ctx := NewContext(newPlatform())
			var idx int
			var ok bool
			done := false
			frame, quit := runUI(ctx, func() {
				idx, ok = bip85IndexEntryFlow(ctx, &descriptorTheme)
				done = true
			})
			defer quit()
			frame()
			runes(&ctx.Router, "abc")
			frame()
			click(&ctx.Router, Button3) // OK on a non-numeric Fragment -> error screen
			// The flow must NOT have returned: it shows the error, then re-prompts.
			if done {
				t.Fatal("flow returned on a non-numeric index; want error + re-prompt")
			}
			// Dismiss the error screen (Back/OK), then type a valid index and confirm.
			if c, ok := pumpUntil(frame, "index", 16); !ok {
				t.Fatalf("error screen not shown after non-numeric input; got %q", c)
			}
			click(&ctx.Router, Button3) // dismiss showError -> back to the keyboard
			frame()
			// Backspace the 3 stale runes, then type a valid index.
			runes(&ctx.Router, "5")
			frame()
			click(&ctx.Router, Button3) // OK
			for i := 0; i < 16 && !done; i++ {
				frame()
			}
			if !done {
				t.Fatal("flow did not return after a valid re-entry")
			}
			if !ok {
				t.Fatalf("re-entered a valid index -> ok=false; want true (idx=%d)", idx)
			}
		})
	})
}
```

> Note: the `"5"` re-entry asserts `ok==true` (it appends to the prior `"abc5"`? No — `showError` re-enters a FRESH `bip85IndexEntryFlow` loop iteration but the SAME keyboard instance retains `Fragment`). The implementation in Step 3 calls `kbd.Clear()` after a parse error so the re-prompt starts empty; the test relies on that (typing `"5"` then OK yields `idx=5,ok=true`). If the impl chose NOT to clear, the test would see `"abc5"` and re-error — so Step 3 MUST clear on parse error.

- [ ] **Step 2: Run test to verify it fails**

Run: `go test ./gui/ -run TestBip85IndexEntryFlow -v`
Expected: FAIL — `undefined: bip85IndexEntryFlow` (compile error).

- [ ] **Step 3: Write minimal implementation**

In `gui/bip85.go`, add `bip85IndexEntryFlow` immediately after `bip85ParamPickFlow` (this clones `typeAddressFlow`; it is wired into the picker in Task 4):

```go
// bip85IndexEntryFlow lets the operator TYPE the child index on a cleartext
// keyboard (the index is public, not a secret). It clones typeAddressFlow
// (gui/verify_address.go:44-71): Back (Button1) -> (0,false); OK (Button3) parses
// kbd.Fragment via parseBip85Index. On a parse error it shows the message and
// RE-PROMPTS (clears the keyboard, re-loops) — it NEVER returns a silent 0 and
// NEVER aborts. Only a valid index in [0,2^31-1] returns (idx,true).
func bip85IndexEntryFlow(ctx *Context, th *Colors) (int, bool) {
	kbd := NewAddressKeyboard(ctx)
	backBtn := &Clickable{Button: Button1}
	okBtn := &Clickable{Button: Button3}
	for !ctx.Done {
		for kbd.Update(ctx) {
		}
		if backBtn.Clicked(ctx) {
			return 0, false
		}
		if okBtn.Clicked(ctx) {
			idx, err := parseBip85Index(kbd.Fragment)
			if err != nil {
				showError(ctx, th, "Child index", "Enter a whole number 0 to 2147483647.")
				kbd.Clear()
				continue
			}
			return idx, true
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
		title, _ := layoutTitle(ctx, dims.X, th.Text, "Child index")
		ctx.Frame(op.Layer(kbdOp, nav, title, op.Color(&ctx.B, th.Background)))
	}
	return 0, false
}
```

Add `"seedhammer.com/gui/layout"` to the import block (it is referenced by `layout.Rectangle`, mirroring `verify_address.go`):

```go
import (
	"errors"
	"fmt"
	"strconv"

	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip39"
	"seedhammer.com/bip85"
	"seedhammer.com/engrave"
	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/layout"
	"seedhammer.com/gui/op"
)
```

> `showError`, `Clickable`, `Button1`/`Button3`, `layoutNavigation`, `layoutTitle`, `NavButton`, `StyleSecondary`/`StylePrimary`, `assets.IconBack`/`IconCheckmark`, `leadingSize`, `op.Layer`/`op.Color` are all in-package symbols used by `typeAddressFlow` / `showError` callers — no new external import beyond `layout`.

- [ ] **Step 4: Run test to verify it passes**

Run: `go test ./gui/ -run TestBip85IndexEntryFlow -v`
Expected: PASS — all three sub-tests.

> If the `nonnumeric_reprompts_then_valid` sub-test stalls at `pumpUntil(frame, "index", 16)`, confirm `showError`'s title text contains "index" (it does — title is `"Child index"`). If `showError` uses a hold-to-dismiss instead of a click, adjust the dismiss step to match the shipped `showError` contract — re-read `showError` at impl time and mirror `TestChildSeedWarningAbort`'s dismiss idiom (`gui/bip85_test.go:171-198`) if needed.

- [ ] **Step 5: Commit**

```bash
git add gui/bip85.go gui/bip85_test.go
git commit -S -s -m "feat(bip85): typed child-index entry flow (keyboard clone)

bip85IndexEntryFlow clones typeAddressFlow over NewAddressKeyboard
(cleartext; the index is public). OK parses via parseBip85Index; a parse
error shows a message and re-prompts (no silent 0, no abort). Back -> (0,false).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Wire into `bip85ParamPickFlow` + re-pin bounds + flow re-point

**Files:**
- Modify: `gui/bip85.go` (replace the index `ChoiceScreen` in `bip85ParamPickFlow`; remove `bip85IndexChoices`)
- Test: `gui/bip85_test.go` (re-pin `TestBip85ParamBounds`; re-point `TestBip85DeriveFlow_ScrubsBothMnemonics` index step)

- [ ] **Step 0: Re-probe-verify the high-index golden (STALENESS GUARD)**

Run (from the worktree, `PATH` set):

```bash
export XPRV="xprv9s21ZrQH143K3GJpoapnV8SFfukcVBSfeCficPSGfubmSFDxo1kuHnLisriDvSnRRuL2Qrg5ggqHKNVpxR86QEC8w35uxmGoggxtQTPvfUu"
printf '%s' "$XPRV" | go run ./cmd/biptool derive -path "m/83696968h/39h/0h/12h/2147483647h" bip39 -words 12
printf '%s' "$XPRV" | go run ./cmd/biptool derive -path "m/83696968h/39h/0h/12h/2147483648h" bip39 -words 12
```

Expected:
```
jewel solution patient quarter elite grace quarter dinosaur taste parent dial clump
biptool: bip32: path element out of range: "2147483648"
```

If the first line differs from the golden pinned in Task 2 (`TestDeriveBip85Child_HighIndexGolden`), STOP — a derive-library bump moved it; update the golden in Task 2's test to the new value and re-run Task 2 before proceeding. (Verified byte-identical at HEAD 8459654.)

- [ ] **Step 1: Re-pin the bounds test (the typed-entry contract)**

Replace the body of `TestBip85ParamBounds` (`gui/bip85_test.go:138-163`) — the index axis is no longer the enumerated `bip85IndexChoices` slice, it is "any value `parseBip85Index` accepts":

```go
// TestBip85ParamBounds asserts the picker's parameter contract: word count is
// exactly {12,18,24}, and the typed index is whatever parseBip85Index accepts —
// 0 and 2^31-1 in range, 2^31 / non-numeric rejected (the typed-entry contract
// replaces the retired bounded 0..9 ChoiceScreen). Every (words, representative
// accepted-index) pair derives a valid child.
func TestBip85ParamBounds(t *testing.T) {
	if len(bip85WordChoices) != 3 ||
		bip85WordChoices[0] != 12 || bip85WordChoices[1] != 18 || bip85WordChoices[2] != 24 {
		t.Fatalf("bip85WordChoices = %v, want [12 18 24]", bip85WordChoices)
	}
	// The index axis is the validator's contract, not an enumerated slice.
	for _, s := range []string{"0", "2147483647"} { // boundaries, both accepted
		if _, err := parseBip85Index(s); err != nil {
			t.Fatalf("parseBip85Index(%q) rejected an in-range index: %v", s, err)
		}
	}
	for _, s := range []string{"2147483648", "abc", "-1", ""} { // out of range / non-numeric
		if _, err := parseBip85Index(s); err == nil {
			t.Fatalf("parseBip85Index(%q) accepted an out-of-contract index", s)
		}
	}
	// Every (words, representative accepted-index) pair derives a valid child.
	for _, w := range bip85WordChoices {
		for _, idx := range []int{0, 1, 9, 1000000, 2147483647} {
			child, err := deriveBip85Child(abandonAboutMnemonic(), "", w, idx)
			if err != nil {
				t.Fatalf("words=%d idx=%d: %v", w, idx, err)
			}
			if len(child) != w || !child.Valid() {
				t.Fatalf("words=%d idx=%d: bad child (%d words, valid=%v)", w, idx, len(child), child.Valid())
			}
		}
	}
}
```

- [ ] **Step 2: Run the re-pinned bounds test to verify it fails**

Run: `go test ./gui/ -run TestBip85ParamBounds -v`
Expected: PASS already (it only references `parseBip85Index`, `bip85WordChoices`, `deriveBip85Child` — all present). The OLD enumeration is now gone, so `bip85IndexChoices` is referenced ONLY by `bip85ParamPickFlow`. (Removing it is Step 3.)

- [ ] **Step 3: Wire the typed flow into the picker + remove `bip85IndexChoices`**

In `gui/bip85.go`, replace the index `ChoiceScreen` block in `bip85ParamPickFlow` (`gui/bip85.go:128-137`):

Old:
```go
		idxCS := &ChoiceScreen{
			Title:   "Child Seed",
			Lead:    "Child index",
			Choices: []string{"0", "1", "2", "3", "4", "5", "6", "7", "8", "9"},
		}
		isel, iok := idxCS.Choose(ctx, th)
		if !iok {
			continue // Back from index -> re-pick the word count.
		}
		return bip85WordChoices[wsel], bip85IndexChoices[isel], true
```

New:
```go
		index, iok := bip85IndexEntryFlow(ctx, th)
		if !iok {
			continue // Back from index -> re-pick the word count.
		}
		return bip85WordChoices[wsel], index, true
```

Then remove the now-unused `bip85IndexChoices` declaration (`gui/bip85.go:112`):

Old:
```go
var bip85WordChoices = []int{12, 18, 24}
var bip85IndexChoices = []int{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
```

New:
```go
var bip85WordChoices = []int{12, 18, 24}
```

Also update the doc comment above `bip85WordChoices` (`gui/bip85.go:106-110`) so it no longer claims a bounded `0..9` index (replace the `bip85IndexChoices` sentence; keep the word-count rationale):

Old:
```go
// bip85WordChoices / bip85IndexChoices are the picker's in-spec, validated-by-
// construction bounds (R0-I-B): word count = biptool's {12,18,24}; index is a
// bounded small set 0..9 (no free-form numeric entry — there is no reusable
// numeric-entry widget; a larger index space is a FOLLOWUP). The application is
// FIXED to BIP-39 (the only engrave-as-words-faithful BIP-39 app).
var bip85WordChoices = []int{12, 18, 24}
```

New:
```go
// bip85WordChoices is the picker's in-spec, validated-by-construction word-count
// set: biptool's {12,18,24}. The child index is now TYPED (bip85IndexEntryFlow +
// parseBip85Index, range [0,2^31-1]) rather than a bounded ChoiceScreen. The
// application is FIXED to BIP-39 (the only engrave-as-words-faithful BIP-85 app).
var bip85WordChoices = []int{12, 18, 24}
```

> Confirm no other reference to `bip85IndexChoices` survives:
> `grep -rn bip85IndexChoices gui/` → Expected: NO matches (exit 1).

- [ ] **Step 4: Re-point the full-flow index step**

In `TestBip85DeriveFlow_ScrubsBothMnemonics` (`gui/bip85_test.go:236-239`), the index step currently uses `chooseEntry(frame, &ctx.Router, 0)` (a ChoiceScreen idiom). Replace ONLY the child-index step (keep the word-count `chooseEntry`) with the typed-entry drive:

Old:
```go
		// Param picker: word count = 12 (index 0), child index = 0 (index 0).
		// chooseEntry queues the Down presses, pumps a frame, confirms, pumps again.
		chooseEntry(frame, &ctx.Router, 0) // word count 12
		chooseEntry(frame, &ctx.Router, 0) // child index 0
```

New:
```go
		// Param picker: word count = 12 (ChoiceScreen, index 0), then TYPE the child
		// index "0" on the keyboard and press OK (the index is now typed, not chosen).
		chooseEntry(frame, &ctx.Router, 0) // word count 12 (ChoiceScreen)
		runes(&ctx.Router, "0")            // child index 0 (typed)
		frame()
		click(&ctx.Router, Button3) // OK on the index keyboard
		frame()
```

- [ ] **Step 5: Run the picker + flow tests to verify they pass**

Run: `go test ./gui/ -run 'TestBip85ParamBounds|TestBip85DeriveFlow_ScrubsBothMnemonics|TestBip85IndexEntryFlow' -v`
Expected: all PASS. (The full-flow test now drives the typed index; the two-secret scrub assertions stay green — I-4.)

- [ ] **Step 6: Build + the whole bip85 surface**

```bash
go build ./...                 # Expected: exit 0
go test ./gui/ -run 'Bip85|DeriveBip85Child|ParseBip85Index' -v
```
Expected: all PASS; no `bip85IndexChoices` undefined error.

- [ ] **Step 7: Commit**

```bash
git add gui/bip85.go gui/bip85_test.go
git commit -S -s -m "feat(bip85): use typed index entry in the picker; retire 0..9 ChoiceScreen

bip85ParamPickFlow now calls bip85IndexEntryFlow for the child index
(full 0..2^31-1 range); bip85IndexChoices removed. Re-pin
TestBip85ParamBounds to the validator contract and re-point the full-flow
test's index step to the keyboard.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Fuzz tightening + no-regression sweep

**Files:**
- Modify: `gui/bip85_test.go` (tighten `FuzzDeriveBip85Child`, seed the corpus)

- [ ] **Step 1: Tighten the fuzz success assertion + seed the corpus**

Replace `FuzzDeriveBip85Child` (`gui/bip85_test.go:293-314`):

Old corpus + assertion:
```go
	f.Add(12, 0)
	f.Add(18, 5)
	f.Add(24, 9)
	f.Add(15, 0)  // out-of-spec word count
	f.Add(12, -1) // negative index
	f.Add(0, 0)
	f.Fuzz(func(t *testing.T, words, index int) {
		// Must not panic. Errors are fine for out-of-spec inputs.
		child, err := deriveBip85Child(abandonAboutMnemonic(), "", words, index)
		if err != nil {
			return
		}
		// On success the inputs were in-spec; the child must be valid.
		if !validBip85Words(words) || index < 0 {
			t.Fatalf("deriveBip85Child accepted out-of-spec words=%d index=%d", words, index)
		}
		if len(child) != words || !child.Valid() {
			t.Fatalf("words=%d index=%d: invalid child (%d words, valid=%v)", words, index, len(child), child.Valid())
		}
	})
```

New:
```go
	f.Add(12, 0)
	f.Add(18, 5)
	f.Add(24, 9)
	f.Add(15, 0)             // out-of-spec word count
	f.Add(12, -1)            // negative index
	f.Add(0, 0)
	f.Add(12, 1<<31)         // = 2147483648: would wrap uint32 -> unhardened element 0 (R0-M1)
	f.Add(12, 1<<31+1)       // = 2147483649: would wrap to unhardened element 1 (R0-M1)
	f.Add(12, 2147483647)    // = 2^31-1: the accepted boundary
	f.Fuzz(func(t *testing.T, words, index int) {
		// Must not panic. Errors are fine for out-of-spec inputs.
		child, err := deriveBip85Child(abandonAboutMnemonic(), "", words, index)
		if err != nil {
			return
		}
		// On success the inputs were in-spec; the child must be valid AND the index
		// must be a valid hardened index (0..2^31-1) — accepting index>2^31-1 means
		// the uint32 truncation guard failed (R0-M1).
		if !validBip85Words(words) || index < 0 || index > bip85MaxIndex {
			t.Fatalf("deriveBip85Child accepted out-of-spec words=%d index=%d", words, index)
		}
		if len(child) != words || !child.Valid() {
			t.Fatalf("words=%d index=%d: invalid child (%d words, valid=%v)", words, index, len(child), child.Valid())
		}
	})
```

- [ ] **Step 2: Run the seeded fuzz corpus (deterministic, no fuzzing time)**

Run: `go test ./gui/ -run 'FuzzDeriveBip85Child/' -v`
Expected: PASS — the seed corpus (incl. `1<<31` / `1<<31+1`) all derive→error, so the tightened assertion never fires on them.

- [ ] **Step 3: Short fuzz pass (smoke, finds no truncation)**

Run: `go test ./gui/ -run '^$' -fuzz 'FuzzDeriveBip85Child$' -fuzztime 20s`
Expected: `--- PASS` / `elapsed: …; no new interesting inputs` (no failing corpus written to `testdata/fuzz/`). If a crasher appears, the guard is incomplete — fix before proceeding.

- [ ] **Step 4: Full no-regression sweep**

```bash
go build ./...                              # Expected: exit 0
go vet ./gui/                               # Expected: exit 0 (the pre-existing gui/op/draw_test.go:176 note is OUT OF SCOPE — vet ./gui/ does not cover gui/op)
go test ./gui/ -run 'TestAllocs'           # Expected: PASS (no new heap allocs on the hot path)
go test ./gui/... ./bip85/...              # Expected: ok seedhammer.com/gui, ok seedhammer.com/bip85
```

> If any T7b sibling test (`TestEngraveBip85Child_UsesChildFP`, `TestChildSeedWarningAbort`, the index-0 goldens) fails, STOP and reconcile — the typed-index change must be additive (I-1/I-4).

- [ ] **Step 5: Commit**

```bash
git add gui/bip85_test.go
git commit -S -s -m "test(bip85): tighten FuzzDeriveBip85Child for the >2^31-1 guard

Fail the success path on index > 2^31-1 and seed the corpus with 1<<31
and 1<<31+1 (the two uint32-wrap cases). No-regression sweep green.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Self-Review

**1. Spec coverage (invariants I-1..I-4 + the 3 Minors):**

- **I-1 (derivation faithful):** Task 2 pins the high-index golden (`m/83696968'/39'/0'/12'/2147483647'` → `jewel solution …`) and index-0-unchanged; Task 4 re-probe-verifies it two ways before pinning. The walk/truncation are unchanged; only the index axis widens. COVERED.
- **I-2 (Critical — width-safe parse + no silent truncation):** Task 1 (`parseBip85Index` via `strconv.ParseUint(…,10,64)`, rejects `>2^31-1`) + Task 2 (the in-`deriveBip85Child` upper-bound guard before `uint32(index)+h`). Both guards independent of `int` width. Task 5 fuzz locks it on `1<<31`/`1<<31+1`. The R0-reproduced bug (`2^31`/`2^31+1` → unhardened element, no error) is the explicit Task 2 failing test. COVERED.
- **I-3 (m\*-free + firmware-only):** only `gui/bip85.go` + its test change; one new stdlib import (`strconv`) + one in-repo import (`gui/layout`, already used by the clone source); no `md`/`mk`/`codex32`; no new program/enum/lockstep/CLI-flag/SemVer. COVERED.
- **I-4 (security spine unchanged):** the index is public (no new secret, no scrub-buffer touch); `TestBip85DeriveFlow_ScrubsBothMnemonics` two-secret scrub assertions stay green after the index step is re-pointed (Task 4 keeps the rest of the flow byte-unchanged). COVERED.
- **R0-M1 (fuzz):** Task 5 tightens `:307` to fail on `index>2^31-1` and seeds `1<<31` AND `1<<31+1`. COVERED.
- **R0-M2 (length cap ≠ range authority):** Task 1 rejects `"9999999999"` (10 digits, in length but out of range) — the validator, not a length cap, is the authority. No early length cap is added (it was optional; omitted to keep the validator the single authority). COVERED.
- **R0-M3 (distinct upper-bound message):** lower bound keeps `"bip85: invalid index: %d"` (`gui/bip85.go:37`); the new upper bound uses `"bip85: index %d exceeds the maximum %d"` — distinct, so diagnostics tell them apart. COVERED.
- **Leading-zero adjudication (#1, ACCEPT):** Task 1 accepts `"007"`→7 (base-10 `ParseUint`). COVERED.
- **Empty-Fragment adjudication (#2, re-prompt not silent 0):** Task 1 rejects `""`; Task 3 re-prompts on any parse error incl. empty (no silent 0, no abort). COVERED.

**2. Placeholder scan:** No `TBD`/`TODO`/`add appropriate …`/`similar to Task N`. Every code step has full code; every run step has the exact command + expected output. The one forward reference (`bip85IndexEntryFlow` is added in Task 3, wired in Task 4) is intentional and explicit. PASS.

**3. Type/signature consistency:**
- `parseBip85Index(s string) (int, error)` — defined Task 1, called in Tasks 3 (`bip85IndexEntryFlow`) and 4 (`TestBip85ParamBounds`). Consistent.
- `bip85IndexEntryFlow(ctx *Context, th *Colors) (int, bool)` — defined Task 3, called in Task 4 (`bip85ParamPickFlow` + the flow test). Consistent.
- `bip85MaxIndex` (untyped const = `hdkeychain.HardenedKeyStart - 1`) — defined Task 1, used in Tasks 1/2/5. Compares cleanly against `int index` (untyped const) and `uint64 v` (untyped const ≤ `2^31-1` fits). Consistent.
- `bip85WordChoices` retained; `bip85IndexChoices` removed in Task 4 (grep-confirmed unused). Consistent.
- Keyboard idiom (`runes` + `for kbd.Update`, `click(Button3)`=OK, `click(Button1)`=Back) matches `TestTypeAddressCasePreserved`. Consistent.

No gaps found.

---

## Execution Handoff

Plan complete and saved to `design/IMPLEMENTATION_PLAN_seedhammer_bip85_custom_index.md`. Per the project R0 gate, this plan MUST pass an opus architect R0 review to 0 Critical / 0 Important before ANY code. After GREEN: a SINGLE implementer executes the tasks in the `feat/bip85-custom-index` worktree (TDD, signed + DCO commits, author Brian Goss), followed by the mandatory whole-diff adversarial execution review, then merge no-ff → push `bg002h`.
