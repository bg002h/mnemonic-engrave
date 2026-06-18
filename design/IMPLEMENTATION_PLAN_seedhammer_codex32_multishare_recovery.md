# SeedHammer CODEX32 Multi-Share Recovery (Cycle B) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let the device reconstruct a codex32 unshared secret from k entered shares and engrave it verbatim (Option A), reusing the existing codex32 engrave path. Closes the only correctness gap (`Interpolate` had zero `gui` callers).

**Architecture:** **B1** (codex32 pkg): `ConsistentShares([]String) error` (incremental cross-share consistency, no count check — mirrors `Interpolate`'s pass-1) + extend `Describe` to label the cross-share sentinels. **B2/B3** (gui): convert `confirmCodex32Flow` from `bool` to an action enum {Back, Engrave, Recover} (Recover offered only for a share, index ≠ S) + title branch; add `recoverCodex32Flow` (collect shares 2..k, "Share i of k", eager `ConsistentShares` validation, then `Interpolate(shares,'S')`); refactor `engraveObjectFlow`'s `case codex32.String:` into an `engraveCodex32` loop that re-confirms a recovered secret then engraves verbatim. **No new crypto; `Interpolate`/`Split()`/`mdmk.go` untouched.**

**Tech Stack:** Go/TinyGo. Host tests with `/home/bcg/.local/go/bin/go`. GUI harness: `runUI`+`ExtractText`+`uiContains`, `runes`/`click`, `descriptorTheme`, `newPlatform()`. codex32 tests are white-box (`package codex32`).

**Base:** fork `main` `bf7f811` (post-Cycle-A1). Branch `feat/codex32-multishare-recovery`. Fork-side only; **no upstream PR**.

**Spec:** `design/SPEC_seedhammer_codex32_multishare_recovery.md` (R0 GREEN at R1 — `design/agent-reports/seedhammer-codex32-multishare-spec-review-R{0,1}.md`).

**PLAN GATE:** this plan must pass the opus-architect R0 gate (0C/0I) before any code.

**Build order:** B1a (Task 1) → B1b (Task 2) → B2/B3 (Task 3, atomic — the `confirmCodex32Flow` signature change couples the GUI pieces).

---

## File Structure

| File | Responsibility | Tasks |
|---|---|---|
| `codex32/polish.go` *(modify)* | Add `ConsistentShares`; extend `Describe` with 6 cross-share labels. | 1,2 |
| `codex32/polish_test.go` *(modify)* | `TestConsistentShares`; update + extend `TestDescribe`. | 1,2 |
| `gui/codex32_polish.go` *(modify)* | `codex32ConfirmAction` enum; `confirmCodex32Flow` bool→action + title branch + Recover button; `showCodex32Error`; `recoverCodex32Flow`. | 3 |
| `gui/gui.go` *(modify)* | `inputCodex32Flow` gains a `title` param; `newInputFlow` call updated; `engraveObjectFlow case codex32.String:` → `engraveCodex32`. | 3 |
| `gui/codex32_polish_test.go` *(modify)* | Update `TestConfirmCodex32Share` (note text changed) + `codex32Frame` (title arg); add `TestConfirmCodex32Action*`, `TestRecoverCodex32`, `TestRecoverCodex32Mismatch`. | 3 |
| `codex32/codex32.go`, `codex32/mdmk.go`, `gui/codex32_input_test.go` *(unchanged — must stay green)* | `Interpolate`/`Split()`/`partsInner` crypto; mdmk; `TestInputSeedCodex32`. | guard |

**Commit hygiene:** explicit paths (no `git add -A`). Commits signed + DCO: `git commit -S -s -m "…"` (fall back to `-s` if SSH signing unavailable, and say so).

---

## Task 0: Isolated worktree + clean baseline

**Files:** none.

- [ ] **Step 1: Create the worktree off fork main**

```bash
cd /scratch/code/shibboleth/seedhammer
git rev-parse --short HEAD                 # expect bf7f811
git worktree add /scratch/code/shibboleth/seedhammer-wt-recovery -b feat/codex32-multishare-recovery bf7f811
cd /scratch/code/shibboleth/seedhammer-wt-recovery
git config user.name "Brian Goss"
git config user.email "goss.brian@gmail.com"
```

- [ ] **Step 2: Verify clean baseline** — Run: `/home/bcg/.local/go/bin/go test ./codex32/... ./gui/...`
Expected: PASS. If red, STOP and report.

---

## Task 1: B1a — `codex32.ConsistentShares`

**Files:** Modify `codex32/polish.go`; modify `codex32/polish_test.go`.

**Context:** `Interpolate` (`codex32.go:185`) checks share consistency in a pass-1 loop (length → hrp → threshold → id, sentinels `errMismatched{Length,HRP,Threshold,ID}`) then repeated-index later. It returns `errInsufficientShares` for < k shares, so the GUI cannot use it to validate a *partial* set. `ConsistentShares` does the consistency subset (no count check). `parts()` (`codex32.go:175`) **panics on a non-`New`-valid String** — hence the precondition. `parts.shareIdx` is a comparable `fe`. `ConsistentShares` lives in `codex32/polish.go` (same package → may call `s.parts()` and read `s.s`).

- [ ] **Step 1: Write the failing test** — append to `codex32/polish_test.go`

```go
func TestConsistentShares(t *testing.T) {
	mk := func(s string) String {
		v, err := New(s)
		if err != nil {
			t.Fatalf("New(%s): %v", s, err)
		}
		return v
	}
	// BIP-93 vector-2 shares: threshold 2, id NAME, indices A and C.
	a := mk("MS12NAMEA320ZYXWVUTSRQPNMLKJHGFEDCAXRPP870HKKQRM")
	c := mk("MS12NAMECACDEFGHJKLMNPQRSTUVWXYZ023FTR2GDZMPY6PN")
	// vector-3 share: threshold 3, id CASH (a field mismatch vs the vector-2 set).
	cash := mk("MS13CASHA320ZYXWVUTSRQPNMLKJHGFEDCA2A8D0ZEHN8A0T")

	if err := ConsistentShares(nil); err != nil {
		t.Errorf("nil set: %v, want nil", err)
	}
	if err := ConsistentShares([]String{a}); err != nil {
		t.Errorf("single share: %v, want nil", err)
	}
	if err := ConsistentShares([]String{a, c}); err != nil {
		t.Errorf("consistent pair: %v, want nil", err)
	}
	if err := ConsistentShares([]String{a, a}); !errors.Is(err, errRepeatedIndex) {
		t.Errorf("repeated index: %v, want errRepeatedIndex", err)
	}
	if err := ConsistentShares([]String{a, cash}); !errors.Is(err, errMismatchedThreshold) {
		t.Errorf("threshold mismatch: %v, want errMismatchedThreshold", err)
	}
}
```

- [ ] **Step 2: Run to verify it fails** — Run: `/home/bcg/.local/go/bin/go test ./codex32/ -run TestConsistentShares`
Expected: FAIL — `undefined: ConsistentShares`.

- [ ] **Step 3: Implement `ConsistentShares`** — append to `codex32/polish.go`

```go
// ConsistentShares reports whether a set of codex32 shares can belong to one
// recovery set: all share the same HRP, threshold, identifier, and total length,
// and all share indices are distinct. It does NOT require the set to be complete
// (k shares) — use it to validate shares as they are collected. Returns the same
// sentinels Interpolate uses (errMismatched{Length,HRP,Threshold,ID},
// errRepeatedIndex), so Describe maps them. A set of 0 or 1 share is consistent.
//
// Each share MUST already be New-valid: ConsistentShares calls the unexported
// parts(), which PANICS on a malformed String. Callers must only pass strings
// that passed New without error (the keypad gates the OK button on New==nil).
func ConsistentShares(shares []String) error {
	if len(shares) <= 1 {
		return nil
	}
	s0 := shares[0].parts()
	for _, share := range shares {
		p := share.parts()
		switch {
		case len(shares[0].s) != len(share.s):
			return errMismatchedLength
		case s0.hrp != p.hrp:
			return errMismatchedHRP
		case s0.threshold != p.threshold:
			return errMismatchedThreshold
		case s0.id != p.id:
			return errMismatchedID
		}
	}
	seen := make(map[fe]bool, len(shares))
	for _, share := range shares {
		idx := share.parts().shareIdx
		if seen[idx] {
			return errRepeatedIndex
		}
		seen[idx] = true
	}
	return nil
}
```

- [ ] **Step 4: Run to verify it passes** — Run: `/home/bcg/.local/go/bin/go test ./codex32/ -run TestConsistentShares`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add codex32/polish.go codex32/polish_test.go
git commit -S -s -m "codex32: add ConsistentShares for incremental multi-share validation"
```

---

## Task 2: B1b — extend `Describe` with cross-share labels

**Files:** Modify `codex32/polish.go`; modify `codex32/polish_test.go`.

**Context:** `Describe` currently returns "invalid" for the cross-share sentinels. The existing `TestDescribe` asserts `{errInsufficientShares, "invalid"}` (`polish_test.go:46`) — that row MUST change (R1-1).

- [ ] **Step 1: Update the failing test** — in `codex32/polish_test.go`, replace the `errInsufficientShares` row and add the cross-share rows. Change the `sentinels` slice in `TestDescribe` so these rows read:

```go
		{errIncompleteGroup, "incomplete group"},
		{errMismatchedLength, "shares differ in length"},
		{errMismatchedHRP, "mismatched type"},
		{errMismatchedThreshold, "mismatched threshold"},
		{errMismatchedID, "mismatched id"},
		{errRepeatedIndex, "repeated share"},
		{errInsufficientShares, "need more shares"},
		{errors.New("other"), "invalid"},
```

(i.e. delete the old `{errInsufficientShares, "invalid"}, // Interpolate-only → fallback` line and insert the six cross-share rows before the `{errors.New("other"), "invalid"}` row.)

- [ ] **Step 2: Run to verify it fails** — Run: `/home/bcg/.local/go/bin/go test ./codex32/ -run TestDescribe`
Expected: FAIL — e.g. `Describe(mismatched threshold) = "invalid", want "mismatched threshold"`.

- [ ] **Step 3: Extend `Describe`** — in `codex32/polish.go`, insert these six cases into the `Describe` switch, immediately before the `default:` clause:

```go
	case errors.Is(err, errMismatchedLength):
		return "shares differ in length"
	case errors.Is(err, errMismatchedHRP):
		return "mismatched type"
	case errors.Is(err, errMismatchedThreshold):
		return "mismatched threshold"
	case errors.Is(err, errMismatchedID):
		return "mismatched id"
	case errors.Is(err, errRepeatedIndex):
		return "repeated share"
	case errors.Is(err, errInsufficientShares):
		return "need more shares"
```

- [ ] **Step 4: Run to verify it passes** — Run: `/home/bcg/.local/go/bin/go test ./codex32/...`
Expected: PASS (all codex32 tests).

- [ ] **Step 5: Commit**

```bash
git add codex32/polish.go codex32/polish_test.go
git commit -S -s -m "codex32: map cross-share sentinels in Describe for recovery UI"
```

---

## Task 3: B2/B3 — the GUI multi-share recovery flow (atomic)

**Files:** Modify `gui/codex32_polish.go`, `gui/gui.go`, `gui/codex32_polish_test.go`.

**Context:** This task is atomic because converting `confirmCodex32Flow` from `bool` to an action enum breaks its sole production caller (`engraveObjectFlow case codex32.String:`, `gui.go:1841`) until that caller is refactored into `engraveCodex32`. Do all of B2/B3 together so the package compiles. The A1 tests `TestConfirmCodex32Unshared`/`TestConfirmCodex32Share` call `confirmCodex32Flow` as a bare statement (return discarded) inside `runUI` — source-compatible with the signature change; but `TestConfirmCodex32Share` asserts the share note text, which this task changes (update it). `inputCodex32Flow` gains a `title` param (to show "Share i of k"); its only callers are `newInputFlow` (`gui.go:2009`) and the `codex32Frame` test helper.

### 3a — `inputCodex32Flow` title parameter

- [ ] **Step 1: Add the `title` param** — in `gui/gui.go`, change the signature and the title-render line of `inputCodex32Flow`:

Signature (`gui.go:672`):
```go
func inputCodex32Flow(ctx *Context, th *Colors, title string) (codex32.String, bool) {
```
Title line (currently `title, _ := layoutTitle(ctx, dims.X, th.Text, "Input Codex32 Share")`) — rename the local to avoid shadowing the param and use the param:
```go
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, title)
```
and update the frame layer to use `titleOp` in place of the old `title` op (the `op.Layer(... title ...)` becomes `op.Layer(... titleOp ...)`).

- [ ] **Step 2: Update `newInputFlow`'s call** — in `gui/gui.go` (`case 2:`, `gui.go:2009`):
```go
		case 2:
			s, ok := inputCodex32Flow(ctx, th, "Input Codex32 Share")
			if ok {
				return s, true
			}
```

- [ ] **Step 3: Update the `codex32Frame` test helper** — in `gui/codex32_polish_test.go`, change its `inputCodex32Flow` call:
```go
		inputCodex32Flow(ctx, &descriptorTheme, "Input Codex32 Share")
```

- [ ] **Step 4: Build-check** — Run: `/home/bcg/.local/go/bin/go build ./gui/`
Expected: PASS (no other callers of `inputCodex32Flow`).

### 3b — confirm action enum + Recover + title branch

- [ ] **Step 5: Write failing confirm-action tests** — append to `gui/codex32_polish_test.go`

```go
func TestConfirmCodex32ShareOffersRecover(t *testing.T) {
	s, err := codex32.New("MS12NAMEA320ZYXWVUTSRQPNMLKJHGFEDCAXRPP870HKKQRM")
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Button2) // Recover
	if got := confirmCodex32Flow(ctx, &descriptorTheme, s); got != codex32Recover {
		t.Errorf("share + Button2 → %v, want codex32Recover", got)
	}
}

func TestConfirmCodex32UnsharedNoRecover(t *testing.T) {
	s, err := codex32.New("ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw")
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx := NewContext(newPlatform())
	click(&ctx.Router, Button2, Button3) // Button2 must be inert for an unshared secret
	if got := confirmCodex32Flow(ctx, &descriptorTheme, s); got != codex32Engrave {
		t.Errorf("unshared + Button2,Button3 → %v, want codex32Engrave (Button2 ignored)", got)
	}
}
```

- [ ] **Step 6: Run to verify they fail** — Run: `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestConfirmCodex32ShareOffersRecover|TestConfirmCodex32UnsharedNoRecover'`
Expected: FAIL — `undefined: codex32Recover`/`codex32Engrave` (compile error) and the `bool` return type.

- [ ] **Step 7: Convert `confirmCodex32Flow` to the action enum** — in `gui/codex32_polish.go`, add the enum and rewrite the function:

```go
// codex32ConfirmAction is the result of the pre-engrave codex32 confirm screen.
type codex32ConfirmAction int

const (
	codex32Back    codex32ConfirmAction = iota // Button1
	codex32Engrave                             // Button3
	codex32Recover                             // Button2 — only offered for a share (index != S)
)

// confirmCodex32Flow shows a pre-engrave review of a (New-valid) codex32 string.
// For an unshared secret it offers Back/Engrave; for a share (index != S) it also
// offers Recover (reconstruct the secret from k shares). It branches display on
// the RAW ParsePrefix fields (NOT Split(), which remaps an unshared secret's
// threshold 0→1). The codex32 string is engraved verbatim.
func confirmCodex32Flow(ctx *Context, th *Colors, scan codex32.String) codex32ConfirmAction {
	f, _ := codex32.ParsePrefix(scan.String()) // scan is New-valid → no error
	title := "Confirm Codex32 Share"
	lines := []string{"id " + strings.ToUpper(f.Identifier)}
	if f.Unshared {
		title = "Confirm Codex32 Secret"
		lines = append(lines, "Unshared secret (S)")
	} else {
		lines = append(lines,
			"Share "+strings.ToUpper(string(f.ShareIndex))+" of a k-of-n set",
			"Engrave this share, or Recover the secret",
		)
	}
	lines = append(lines, fmt.Sprintf("%d chars", len(scan.String())))

	backBtn := &Clickable{Button: Button1}
	recoverBtn := &Clickable{Button: Button2}
	engraveBtn := &Clickable{Button: Button3, AltButton: Center}
	for !ctx.Done {
		if backBtn.Clicked(ctx) {
			return codex32Back
		}
		// Always drain Button2 — even for an unshared secret, where Recover is not
		// offered — so an unconsumed event cannot block the router queue head in a
		// direct-call (non-runUI) context. Act on it only for a share. (R0 C1)
		recoverClicked := recoverBtn.Clicked(ctx)
		if !f.Unshared && recoverClicked {
			return codex32Recover
		}
		if engraveBtn.Clicked(ctx) {
			return codex32Engrave
		}
		dims := ctx.Platform.DisplaySize()
		navBtns := []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack},
		}
		if !f.Unshared {
			navBtns = append(navBtns, NavButton{Clickable: recoverBtn, Style: StyleSecondary, Icon: assets.IconRight})
		}
		navBtns = append(navBtns, NavButton{Clickable: engraveBtn, Style: StylePrimary, Icon: assets.IconHammer})
		nav, _ := layoutNavigation(&ctx.B, th, dims, navBtns...)
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, title)

		screen := layout.Rectangle{Max: dims}
		_, content := screen.CutTop(leadingSize)
		content, _ = content.CutBottom(leadingSize)
		body := make([]op.Op, 0, len(lines))
		y := content.Min.Y + 8
		for _, ln := range lines {
			lbl, sz := widget.Labelw(&ctx.B, ctx.Styles.body, dims.X-2*8, th.Text, ln)
			body = append(body, lbl.Offset(image.Pt((dims.X-sz.X)/2, y)))
			y += sz.Y + 6
		}
		frameOps := append([]op.Op{nav, titleOp}, body...)
		frameOps = append(frameOps, op.Color(&ctx.B, th.Background))
		ctx.Frame(op.Layer(frameOps...))
	}
	return codex32Back
}
```

### 3c — `showCodex32Error` + `recoverCodex32Flow`

- [ ] **Step 8: Add the error modal helper + recovery flow** — append to `gui/codex32_polish.go`

```go
// showCodex32Error displays a dismissible error modal (Button3 dismisses) over a
// blank background; returns when dismissed or ctx.Done.
func showCodex32Error(ctx *Context, th *Colors, msg string) {
	errScr := &ErrorScreen{Title: "Invalid share", Body: msg}
	for !ctx.Done {
		dims := ctx.Platform.DisplaySize()
		d, dismissed := errScr.Layout(ctx, th, dims)
		if dismissed {
			return
		}
		ctx.Frame(op.Layer(d, op.Color(&ctx.B, th.Background)))
	}
}

// recoverCodex32Flow collects shares 2..k (k = the first share's threshold),
// validating each against the set as it is added, then reconstructs the unshared
// secret via Interpolate(shares,'S'). Returns (secret, true) on success, or
// (_, false) if the user backs out or recovery fails.
func recoverCodex32Flow(ctx *Context, th *Colors, first codex32.String) (codex32.String, bool) {
	f, _ := codex32.ParsePrefix(first.String())
	if !f.ThresholdKnown || f.Threshold < 2 { // unreachable for a New-valid share; defensive
		return codex32.String{}, false
	}
	k := f.Threshold
	id := strings.ToUpper(f.Identifier)
	shares := []codex32.String{first}
	for len(shares) < k {
		title := fmt.Sprintf("Share %d of %d · id %s", len(shares)+1, k, id)
		cand, ok := inputCodex32Flow(ctx, th, title)
		if !ok {
			return codex32.String{}, false // Back exits recovery
		}
		pf, _ := codex32.ParsePrefix(cand.String())
		if pf.Unshared {
			showCodex32Error(ctx, th, "enter a share, not the secret")
			continue
		}
		if err := codex32.ConsistentShares(append(shares, cand)); err != nil {
			showCodex32Error(ctx, th, codex32.Describe(err))
			continue
		}
		shares = append(shares, cand)
	}
	secret, err := codex32.Interpolate(shares, 'S')
	if err != nil { // defense-in-depth; should not happen after ConsistentShares + exactly k
		showCodex32Error(ctx, th, codex32.Describe(err))
		return codex32.String{}, false
	}
	return secret, true
}
```

Note: `append(shares, cand)` for the `ConsistentShares` check passes a temporary slice; `cand` is appended to `shares` only on success. (`append` may reuse `shares`'s backing array, but `shares` is not re-read after the check fails, so this is safe; on success we re-`append` and assign.)

### 3d — `engraveCodex32` + wire into `engraveObjectFlow`

- [ ] **Step 9: Write the recovery integration tests** — append to `gui/codex32_polish_test.go`

```go
func TestRecoverCodex32(t *testing.T) {
	shareA, err := codex32.New("MS12NAMEA320ZYXWVUTSRQPNMLKJHGFEDCAXRPP870HKKQRM")
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx := NewContext(newPlatform())
	// k=2: enter the second share (C) and accept it.
	runes(&ctx.Router, "MS12NAMECACDEFGHJKLMNPQRSTUVWXYZ023FTR2GDZMPY6PN")
	click(&ctx.Router, Button3)
	secret, ok := recoverCodex32Flow(ctx, &descriptorTheme, shareA)
	if !ok {
		t.Fatal("recoverCodex32Flow did not recover")
	}
	const want = "MS12NAMES6XQGUZTTXKEQNJSJZV4JV3NZ5K3KWGSPHUH6EVW"
	if got := secret.String(); got != want {
		t.Errorf("recovered %q, want %q", got, want)
	}
}

func TestRecoverCodex32Mismatch(t *testing.T) {
	shareA, err := codex32.New("MS12NAMEA320ZYXWVUTSRQPNMLKJHGFEDCAXRPP870HKKQRM")
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx := NewContext(newPlatform())
	frame, quit := runUI(ctx, func() { recoverCodex32Flow(ctx, &descriptorTheme, shareA) })
	defer quit()
	// Enter a share from a DIFFERENT set (threshold 3, id CASH) and accept it.
	runes(&ctx.Router, "MS13CASHA320ZYXWVUTSRQPNMLKJHGFEDCA2A8D0ZEHN8A0T")
	click(&ctx.Router, Button3)
	var content string
	for i := 0; i < 8; i++ {
		c, ok := frame()
		if !ok {
			break
		}
		content = c
		if uiContains(content, "mismatched") {
			break
		}
	}
	if !uiContains(content, "mismatched") {
		t.Errorf("expected a mismatch error; got %q", content)
	}
}
```

- [ ] **Step 10: Run to verify they fail** — Run: `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestRecoverCodex32'`
Expected: FAIL — `recoverCodex32Flow` exists (Step 8) but `engraveCodex32` and the `engraveObjectFlow` rewrite are not done; these tests call `recoverCodex32Flow` directly so they should actually PASS once Step 8 compiles. **If Step 8 compiled, run them now and expect PASS** (they validate the Step-8 code). Proceed to Step 11 regardless.

- [ ] **Step 11: Refactor `engraveObjectFlow` into `engraveCodex32`** — in `gui/gui.go`, replace the entire `case codex32.String:` block (`gui.go:1841-1854`) with:

```go
	case codex32.String:
		return engraveCodex32(ctx, th, scan)
```

and add the helper (place it in `gui/codex32_polish.go`, appended after `recoverCodex32Flow`). **First add to `gui/codex32_polish.go`'s import block (R0 I1 — `engraveCodex32` needs them; the other codex32 helpers do not):**

```go
	"seedhammer.com/backup"
	"seedhammer.com/font/constant"
```

then append:

```go
// engraveCodex32 confirms a codex32 string and engraves it verbatim. A share may
// instead be recovered into the unshared secret, which is then re-confirmed and
// engraved. Returns true (recognized/handled) in all terminal cases — Back is a
// deliberate decline, NOT "Unknown format".
func engraveCodex32(ctx *Context, th *Colors, scan codex32.String) bool {
	for {
		switch confirmCodex32Flow(ctx, th, scan) {
		case codex32Back:
			return true
		case codex32Recover:
			secret, ok := recoverCodex32Flow(ctx, th, scan)
			if !ok {
				continue // back to the original share's confirm
			}
			scan = secret // recovered unshared secret; loop re-confirms it (no Recover offered for S)
			continue
		case codex32Engrave:
			id, _, _ := scan.Split()
			s := backup.SeedString{Title: id, Seed: scan.String(), Font: constant.Font}
			backupSeedStringFlow(ctx, th, s)
			return true
		}
	}
}
```

### 3e — fix the A1 test the note change breaks

- [ ] **Step 12: Update `TestConfirmCodex32Share`** — in `gui/codex32_polish_test.go`, the share confirm note changed from "engraves THIS share, not a recovered seed" to "Engrave this share, or Recover the secret". Update its assertions:

```go
	if !uiContains(c, "Share A") {
		t.Errorf("share: want \"Share A\"; got %q", c)
	}
	if !uiContains(c, "Recover the secret") {
		t.Errorf("share note: want recover affordance; got %q", c)
	}
```

(`TestConfirmCodex32Unshared` is unaffected — it asserts "Unshared secret"/"id TEST", not the title; the new "Confirm Codex32 Secret" title does not break it. These two tests discard `confirmCodex32Flow`'s return, so the enum signature compiles.)

- [ ] **Step 13: Run the full gui + codex32 suites + guards** — Run: `/home/bcg/.local/go/bin/go test ./codex32/... ./gui/...`
Expected: PASS — incl. `TestRecoverCodex32`, `TestRecoverCodex32Mismatch`, `TestConfirmCodex32*`, and the unchanged `TestInputSeedCodex32`, `TestWordKeyboardScreen`, `TestEngraveCodex32BackoutNotUnknown`, `TestCodex32FlowReadout`.

- [ ] **Step 14: vet** — Run: `/home/bcg/.local/go/bin/go vet ./codex32/... ./gui/...`
Expected: clean (the pre-existing `gui/op/draw_test.go` go1.26 `testing.ArtifactDir` note is unrelated and not from this cycle).

- [ ] **Step 15: Commit**

```bash
git add gui/codex32_polish.go gui/gui.go gui/codex32_polish_test.go
git commit -S -s -m "gui(codex32): multi-share recovery — collect k shares, Interpolate, engrave"
```

---

## Final: whole-diff adversarial execution review (mandatory, non-deferrable)

Dispatch an independent opus adversarial execution review over the entire diff vs `bf7f811`. Persist verbatim to `design/agent-reports/seedhammer-codex32-multishare-execution-review.md`. Fold any Critical/Important and re-review until clean.

Review focus:
- `ConsistentShares` cannot panic on any `New`-valid input; its sentinel order matches `Interpolate`'s; 0/1-share sets return nil; it never accepts an inconsistent set.
- `Describe`'s new labels are correct and the `TestDescribe` rows were updated (no stale "invalid").
- `confirmCodex32Flow` action enum: Recover offered iff `!Unshared`; Button2 inert for unshared; title branches; the A1 callers/tests still pass; the note change is reflected in the test.
- `recoverCodex32Flow`: exactly-k loop terminates; the `ThresholdKnown`/`k≥2` guard never blocks a real share; eager `ConsistentShares` + unshared-rejection wired with the right messages; `append(shares, cand)` aliasing is safe; `Interpolate` defense-in-depth handled.
- `engraveCodex32`: loop terminates (recovered S → no Recover); Back returns `true` (not "Unknown format"); recovered secret engraved verbatim via the unchanged `backupSeedStringFlow`; `Split()` used only for `id`.
- A recovered LONG secret (125–127) engraves via the existing path (M2 — note/observe; add a backup-level long-code test if cheap).
- Scope: `Interpolate`/`Split()`/`codex32.go` crypto + `mdmk.go` untouched; only the 5 intended files changed; `go test ./...` green; signed + DCO.

Then **superpowers:finishing-a-development-branch** — per the post-#36 directive, **no upstream PR**: merge `feat/codex32-multishare-recovery` into fork `main` locally (no-ff, signed) and push to `bg002h`.

---

## Self-Review (author)

- **Spec coverage:** B1a→Task 1, B1b→Task 2, B2/B3→Task 3. The §8 resolved decisions (Option A, branch existing flow, exactly-k, eager-validate, Interpolate untouched) are all realized; Option B / SLIP-39 correctly excluded.
- **Placeholder scan:** every step has real code + exact `-run` commands with expected FAIL/PASS. The one judgment step (Step 10: tests calling `recoverCodex32Flow` directly may already pass after Step 8) is explained, not a placeholder.
- **Type consistency:** `codex32ConfirmAction`/`codex32Back`/`codex32Engrave`/`codex32Recover`, `ConsistentShares`, `recoverCodex32Flow`, `engraveCodex32`, `showCodex32Error`, the `inputCodex32Flow(ctx,th,title)` signature, and the `Fields`/`Describe` symbols all match across tasks and the extracted source.
- **Atomicity:** Task 3 bundles the signature-coupled GUI changes so the package always compiles; the only cross-task dependency (Tasks 1→2 in `polish.go`, Task 3 consuming both) is forward-only.
- **R1-1 captured:** Task 2 Step 1 explicitly rewrites the `errInsufficientShares` row from "invalid" to "need more shares".
