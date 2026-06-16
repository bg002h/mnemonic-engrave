# Firmware PR1 — re-enable on-device CODEX32 entry — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Re-enable the SeedHammer II on-device CODEX32 keypad so a user can hand-enter an `ms1` (BIP-93 codex32) secret on the air-gapped device — the secure path the `me` converter points users to instead of sending `ms1` over NFC.

**Architecture:** A one-line change in `seedhammer`'s `gui/gui.go` — uncomment `"CODEX32"` in the `newInputFlow` menu `Choices`. The handler (`case 2:` → `inputCodex32Flow`) and the keypad flow are already fully implemented and wired; only the menu entry was commented out (with a `// TODO: re-enable`). Verified by a host test that drives the menu to the CODEX32 choice and enters a valid codex32 string. Submitted as a DCO-signed PR to upstream `seedhammer/seedhammer` `main`.

**Tech Stack:** Go 1.25+ (host `go test`; `~/.local/go/bin/go` is go1.26.4). Work happens in the fork clone at `/scratch/code/shibboleth/seedhammer` (origin = `bg002h/seedhammer`, upstream = `seedhammer/seedhammer`, default branch `main`).

> **Design source:** `design/SPEC_seedhammer_engrave.md` §7 PR1 (architect R-loop GREEN). This plan implements that PR. **Per the iterative-architect-review standard, this plan-doc must pass its own architect R0 gate (0C/0I) before any code.** This planning artifact lives in the `mnemonic-engrave` repo; the actual code change lives in the `seedhammer` fork (kept free of our planning docs so the PR is clean).
>
> **Note on `go` PATH:** prefix `go`/`cargo` commands with `export PATH="$HOME/.local/go/bin:$PATH"` (Go is a user-space install).
>
> **Plan status:** architect gate **GREEN** (plan-R0 → plan-R1, 0C/0I; reports in `design/agent-reports/firmware-pr1-codex32-plan-R{0,1}-review.md`). Eligible for execution.

---

## File Structure

| File | Change |
|---|---|
| `seedhammer/gui/gui.go` (`newInputFlow`, ~line 1806) | Uncomment `"CODEX32"` in the menu `Choices`. One line. |
| `seedhammer/gui/codex32_input_test.go` (new) | Host test driving `newInputFlow` → select CODEX32 → enter a valid `ms1`, asserting a `codex32.String` is returned. |

No other files change. `inputCodex32Flow` (`gui/gui.go:623`) and `case 2:` (`gui/gui.go:1820-1821`) already exist — do **not** modify them.

---

## Task 1: Working branch in the fork

**Files:** none (git setup)

- [ ] **Step 1: Create the PR branch off upstream main**

```bash
cd /scratch/code/shibboleth/seedhammer
git fetch upstream
git checkout -b feat/enable-codex32-input upstream/main
git branch --show-current   # => feat/enable-codex32-input
```
Expected: branch created at `upstream/main`'s tip.

- [ ] **Step 2: Confirm the target line is unchanged on main**

Run: `grep -n 'CODEX32' gui/gui.go`
Expected: a line `Choices: []string{"12 WORDS", "24 WORDS" /* , "CODEX32", "SLIP-39" */},` (around line 1806). If the surrounding text differs from what Task 2 edits, adapt the edit to the actual current text (report the deviation).

---

## Task 2: Test-first, then the one-line enable

**Files:**
- Create: `seedhammer/gui/codex32_input_test.go`
- Modify: `seedhammer/gui/gui.go` (`newInputFlow` `Choices`)

- [ ] **Step 1: Write the failing test**

Create `seedhammer/gui/codex32_input_test.go`:
```go
package gui

import (
	"strings"
	"testing"

	"seedhammer.com/codex32"
)

// Drives the "Input Seed" menu to the CODEX32 choice (index 2) and enters a
// valid codex32 string on the keypad, asserting newInputFlow returns it.
//
// Without "CODEX32" in the menu this is RED: with only two choices the Down-key
// selection caps at index 1 ("24 WORDS"), so the run enters 24-word BIP-39
// input; the keypad uppercases each rune, and an uppercase "M" is not a valid
// BIP-39 fragment, so the flow panics (gui.go ~873) — either way it never
// returns a codex32.String. With "CODEX32" added, index 2 routes to
// inputCodex32Flow and returns the entered codex32 string.
//
// NOTE: the keypad stores typed runes UPPERCASE, so the returned string is the
// uppercase form of what we type; compare against strings.ToUpper(share).
func TestInputSeedCodex32(t *testing.T) {
	// A valid "ms" codex32 string from the codex32 package's own test corpus.
	const share = "ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw"

	ctx := NewContext(newPlatform())
	// Menu: move the selection 0 -> 2 (CODEX32) with two Down presses, confirm
	// with Button3 (the ChoiceScreen "choose" button).
	click(&ctx.Router, Down, Down, Button3)
	// Keypad: type the share, then confirm with Button2 (OK).
	runes(&ctx.Router, share)
	click(&ctx.Router, Button2)

	obj, ok := newInputFlow(ctx, &descriptorTheme)
	if !ok {
		t.Fatal("newInputFlow did not return a value")
	}
	s, isCodex := obj.(codex32.String)
	if !isCodex {
		t.Fatalf("newInputFlow returned %T, want codex32.String", obj)
	}
	want := strings.ToUpper(share) // keypad uppercases typed runes
	if got := s.String(); got != want {
		t.Errorf("codex32 entry returned %q, want %q", got, want)
	}
}
```

- [ ] **Step 2: Run the test to verify it FAILS (CODEX32 not yet in the menu)**

Run: `export PATH="$HOME/.local/go/bin:$PATH" && cd /scratch/code/shibboleth/seedhammer && go test ./gui/ -run TestInputSeedCodex32 -v`
Expected: FAIL (red). Without `"CODEX32"` the selection caps at "24 WORDS", so the typed (uppercased) string drives 24-word BIP-39 entry and **panics** — uppercase "M" is not a valid BIP-39 fragment prefix (`panic("invalid fragment")`, `gui/gui.go:873`). Either way the run does not return a `codex32.String`. The test MUST be red here before the change.

> If instead it fails to *compile* or the event sequence doesn't move the selection as expected, this is the one compile/iteration point: adjust the input injection (`Down`/`Button3`/`Button2` ordering, or use `Center` instead of `Button3`) against the actual `ChoiceScreen.Choose` / `inputCodex32Flow` behavior until the test fails for the RIGHT reason (wrong return type), not a harness mismatch. Do not change production code to make it pass yet.

- [ ] **Step 3: Make the one-line change**

In `seedhammer/gui/gui.go`, in `newInputFlow`, change:
```go
			Choices: []string{"12 WORDS", "24 WORDS" /* , "CODEX32", "SLIP-39" */},
```
to:
```go
			Choices: []string{"12 WORDS", "24 WORDS", "CODEX32" /* , "SLIP-39" */},
```
(Only `"CODEX32"` moves out of the comment; SLIP-39 stays commented — its `case 3:` is still disabled.)

- [ ] **Step 4: Run the test to verify it PASSES**

Run: `export PATH="$HOME/.local/go/bin:$PATH" && cd /scratch/code/shibboleth/seedhammer && go test ./gui/ -run TestInputSeedCodex32 -v`
Expected: PASS — `newInputFlow` returns a `codex32.String` whose `String()` equals `strings.ToUpper(share)` (the keypad uppercases typed input).

---

## Task 3: Full verification gate

**Files:** none

- [ ] **Step 1: Full gui package tests (no regressions)**

Run: `export PATH="$HOME/.local/go/bin:$PATH" && cd /scratch/code/shibboleth/seedhammer && go test ./gui/`
Expected: `ok  seedhammer.com/gui` — all existing tests (TestScan, TestEngraveScreen, TestWordKeyboardScreen, …) plus the new one pass.

- [ ] **Step 2: Vet + host build of the non-device packages**

Run: `export PATH="$HOME/.local/go/bin:$PATH" && cd /scratch/code/shibboleth/seedhammer && go vet ./gui/ && go build ./gui/`
Expected: no vet warnings, clean build. (Device-only packages under `driver/` and `cmd/controller` are `//go:build tinygo`-gated and excluded from host build — do not attempt to build them here.)

- [ ] **Step 3: Confirm gofmt cleanliness**

Run: `export PATH="$HOME/.local/go/bin:$PATH" && cd /scratch/code/shibboleth/seedhammer && gofmt -l gui/gui.go gui/codex32_input_test.go`
Expected: no output (both files are gofmt-clean).

---

## Task 4: Commit (DCO) and open the PR

**Files:** none

- [ ] **Step 1: Commit with DCO sign-off**

Upstream requires the Developer Certificate of Origin — every commit MUST be signed off.
```bash
cd /scratch/code/shibboleth/seedhammer
git add gui/gui.go gui/codex32_input_test.go
git commit -s -m "gui: re-enable on-device CODEX32 seed entry

Uncomment the CODEX32 choice in the Input Seed menu; the handler
(inputCodex32Flow) and keypad were already implemented and wired behind a
'// TODO: re-enable'. Adds a host test driving the menu to the CODEX32 choice
and entering a valid codex32 string."
```
(Do NOT add a Co-Authored-By trailer on the upstream PR commit; the DCO `Signed-off-by` line is the required attestation. Verify it landed: `git log -1 | grep Signed-off-by`.)

- [ ] **Step 2: Push to the fork**

Run: `cd /scratch/code/shibboleth/seedhammer && git push -u origin feat/enable-codex32-input`
Expected: branch pushed to `bg002h/seedhammer`.

- [ ] **Step 3: Open the PR to upstream**

```bash
cd /scratch/code/shibboleth/seedhammer
gh pr create --repo seedhammer/seedhammer --base main --head bg002h:feat/enable-codex32-input \
  --title "gui: re-enable on-device CODEX32 seed entry" \
  --body "$(cat <<'EOF'
## Summary
- Re-enables the `CODEX32` choice in the **Input Seed** menu (`newInputFlow`). The handler (`case 2:` → `inputCodex32Flow`) and the keypad flow are already implemented and were sitting behind a `// TODO: re-enable`. This is the only change to production code (one line).
- Adds `gui/codex32_input_test.go`, a host test that drives the menu to the CODEX32 choice and enters a valid codex32 string, asserting `newInputFlow` returns the `codex32.String`.

## Why
Lets users hand-enter a BIP-93 codex32 secret on the air-gapped device instead of importing it over NFC — the secure path for secret material.

## Test Plan
- [ ] `go test ./gui/` passes (incl. the new `TestInputSeedCodex32` and existing flows).
- [ ] `go vet ./gui/` and `go build ./gui/` clean.
EOF
)"
```
Expected: PR URL printed. Report it.

> **Confirm before running Step 3:** opening a PR against upstream `seedhammer/seedhammer` is outward-facing. The controller will confirm with the user before this step executes.

---

## Self-Review

- **Design coverage (§7 PR1):** uncomment CODEX32 → Task 2 Step 3 ✓; verified reachable + functional → Task 2 test ✓; DCO PR to upstream → Task 4 ✓.
- **Placeholder scan:** none. The one flagged iteration point (Task 2 Step 2, event-injection tuning) is an explicit "make it fail for the right reason" step, not a behavioral TBD; the test code and the production change are fully given.
- **Type consistency:** `newInputFlow(ctx, th) (any, bool)`; codex32 case returns `codex32.String`; test asserts `obj.(codex32.String)` and `s.String()` — matches `gui/gui.go:623` + `codex32/codex32.go:390`.
- **Does the test actually test the change?** Yes — without "CODEX32" in `Choices`, two `Down` presses cap selection at index 1 ("24 WORDS") → 24-word entry; the keypad uppercases typed runes and uppercase "M" is not a valid BIP-39 fragment, so the flow panics (`gui/gui.go:873`) → the test is red. With the uncomment, index 2 → `inputCodex32Flow` → returns the (uppercased) `codex32.String`, compared against `strings.ToUpper(share)`. Passes only after the change. (Verified by plan-R0: the keypad-uppercasing and the panic path were confirmed against source.)

## Open items to confirm during execution
- The exact event sequence to select choice index 2 and confirm (`Down, Down, Button3` vs `Center`) — tune in Task 2 Step 2 until the test fails for the right reason.
- The codex32 vector `ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw` is from the codex32 package's own tests; if it does not validate via `codex32.New`, pick another from `codex32/codex32_test.go`.
- Whether the on-screen keypad accepts every character of the chosen vector (all are bech32 + digits; the vector avoids `b/i/o`).
