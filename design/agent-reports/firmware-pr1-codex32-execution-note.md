# Firmware PR1 — execution note (codex32 keypad)

- **Date:** 2026-06-16
- **Where:** fork `/scratch/code/shibboleth/seedhammer`, branch `feat/enable-codex32-input` off `upstream/main`; commit `a731087` (DCO `Signed-off-by`).
- **Process:** executed inline by the controller (one-line change + one test), verified by `go test` (go1.26.4). The plan-R0→R1 architect gate (GREEN) already provided the deep review; a separate two-stage subagent review is disproportionate for a one-line uncomment.

## What was done
- `gui/gui.go:1806`: moved `"CODEX32"` out of the comment in the `newInputFlow` menu `Choices`. One line.
- Added `gui/codex32_input_test.go` — `TestInputSeedCodex32` drives the Input Seed menu to the CODEX32 choice (`Down,Down,Button3`) and types a valid codex32 string (`runes` + `Button2`), asserting `newInputFlow` returns the `codex32.String` (compared uppercase — the keypad uppercases input).

## Verification (evidence)
- **Red phase (without the change):** test does NOT pass. Goroutine dump (`-timeout 15s`) showed it stuck in `inputWordsFlow` (`gui.go:1816→586`) — i.e. the menu navigation worked and reached a choice, but with only two choices the selection caps at "24 WORDS" and 24-word entry never completes on codex32 input, so the test **fails via the test timeout**. NOTE: this corrects the plan-R0/R1 prediction that it would *panic* on an uppercase "M" — in fact it hangs. The test comment was updated to say "fails via the test timeout," not "panics."
- **Green phase (with the change):** `--- PASS: TestInputSeedCodex32 (0.00s)` — the codex32 path terminates immediately.
- **No regression:** `go test ./gui/` → `ok seedhammer.com/gui 4.492s` (all existing tests + the new one).
- **Gates:** `go vet ./gui/` clean; `gofmt -l` clean.

## Known caveat (acceptable)
The test passes normally (feature present). If `CODEX32` were re-commented, the test would hang and fail via the Go test timeout (~10 min default) rather than a fast clean failure — a property of driving the full UI flow. Documented in the test comment; acceptable regression protection for a one-line menu entry.

## Status
Committed locally, all checks green. NOT pushed / no PR opened — awaiting user confirmation (outward-facing action to a third-party repo) and a decision on the DCO sign-off name (`bg` vs `Brian Goss`).
