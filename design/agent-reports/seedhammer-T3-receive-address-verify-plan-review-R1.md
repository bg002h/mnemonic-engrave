<!--
Persisted verbatim. opus-architect R1 re-dispatch of the T3 plan R0 gate, after folding R0 (1C/3I).
PLAN commit c8ae3b0. Reviewer agentId ab2556ef11e10070f. Method: re-materialized + implemented + built +
ran the folded plan in a throwaway worktree off d334861. Verdict: GREEN 0C/0I. All folds CLOSED with
build+test evidence: C1 (runVerify one-shot non-blocking Verifying frame + self-contained result loop →
TestRunVerifyResult passes all 3, no hang), I1 (showMessage eliminated), I2 (updated
TestDescriptorConfirmAddressAffordance drives Button2,Button3 through the Show/Verify ChoiceScreen;
TestAllocs intact), I3 (TestFindPropagatesDerivationError <5;7> passes), M1 (xpubs pkg var, TestFind 7/7),
M2 (scanOnce drive-loop + tvXpub), M3 (btcaddr alias compiles), M4 (NewAddressKeyboard revealed after
Clear, case preserved), M5 (no addressText case in engraveObjectFlow). Whole-suite go test ./... green;
vet/gofmt/build clean; TestScan 8/8 still pass (no recognition regression). One non-blocking MINOR (R1-M1):
the Find code block wrote bare DecodeAddress (undefined in package address — must be address.DecodeAddress);
prose was correct; FIXED in the GREEN commit. Worktree removed; fork clean at d334861; nothing
committed/pushed. Disposition: GREEN — cleared to single-implementer TDD. Text below verbatim (entities
un-escaped: &lt;→<, &gt;→>, &amp;→&).
-->

# R1 RE-DISPATCH — IMPLEMENTATION_PLAN_seedhammer_T3_receive_address_verify.md (`c8ae3b0`)

## Method
Re-materialized the folded plan task-by-task into a throwaway detached worktree off `d334861` (go1.26.4), transcribed every full-Go part verbatim, mirrored the cited scaffolding flows (`descriptorAddressFlow`, `passphraseFlow`, `mk1GatherFlow`), built, and ran every test. Worktree removed; fork clean at `d334861`; nothing committed/pushed.

## Fold verification

| Fold | Status | Evidence |
|---|---|---|
| **C1** (runVerify one-shot non-blocking "Verifying…" + self-contained Back-able result loop, no `showMessage`) | **CLOSED** | `TestRunVerifyResult` PASSES all 3 subcases (match→"Receive", not-found→"Not found", invalid→"Invalid"). The one-shot `ctx.Frame` renders, then `address.Find` runs synchronously, then the result loop renders — flow does NOT hang. All cited primitives resolve and compile: `layoutTitle`, `layoutNavigation`, `widget.Labelw`, `op.Layer`, `op.Color`, `leadingSize` (theme.go:43), `assets.IconBack`. Capture-then-assert (no per-frame Back) works. |
| **I1** (`showMessage` eliminated) | **CLOSED** | No `showMessage` reference anywhere; `runVerify` is self-contained. Builds clean. |
| **I2** (update `TestDescriptorConfirmAddressAffordance`; ChoiceScreen wiring; TestAllocs) | **CLOSED** | Updated test drives `click(Button2, Button3)` (open Show/Verify ChoiceScreen → select choice 0 "Show addresses") → address view opens → PASS. `TestDescriptorConfirmAddressAffordanceUnsupported` still PASS. `TestAllocs` PASS (ChoiceScreen alloc stays inside the click branch; per-frame nav literal untouched). |
| **I3** (`TestFindPropagatesDerivationError`, `<5;7>` range) | **CLOSED** | PASS — propagated derivation error, not silent not-found. |
| **M1** (hoist `xpubs` to pkg var) | **CLOSED** | `TestFind` compiles + PASS (all 7 subcases). |
| **M2** (`scanOnce` drive-loop + real `tvXpub`) | **CLOSED** | `TestScanRecognizesAddress` PASS: address→`addressText`; `wpkh(`+tvXpub+`)` NOT misrecognized. |
| **M3** (`btcaddr` import alias) | **CLOSED** | `btcaddr "github.com/btcsuite/btcd/address/v2"` + `chaincfg/v2` compile in `gui/scan.go`; mainnet-then-testnet branches work. |
| **M4** (`NewAddressKeyboard` sets `revealed=true` AFTER `NewPassphraseKeyboard`) | **CLOSED** | `TestTypeAddressCasePreserved` PASS: `bc1Q3` preserved, unmasked. Confirmed `Clear()` (passphrase_keyboard.go:164) sets `revealed=false`, so post-construction set is required. |
| **M5** (no `addressText` case in `engraveObjectFlow`) | **CLOSED** | `engraveObjectFlow` cases are only `*bip380.Descriptor`/`mdmkText`; default returns false. Top-level address scan harmlessly ignored. |

## Regression
Whole-suite `go test ./...` GREEN (address, gui, and all other packages — no FAIL). `go vet ./address/ ./gui/` clean. `gofmt -l address/ gui/` empty. `go build ./...` OK. `TestAllocs` PASS isolated (`-count=1`). Pre-existing `TestScan` (8 subcases) still PASS — the new address branch did not regress descriptor/codex32/md1/mk1 recognition (a descriptor is still recognized as a Descriptor, before the address probe).

## Findings

**MINOR (R1-M1, non-blocking):** The plan's literal `Find` body writes `want, derr := DecodeAddress(candidate, net)` **unqualified**, which does NOT compile — within package `address`, the btcd parser is imported under its own name `address`, so the local package has no unqualified `DecodeAddress` (verified: `address/address.go:61:16: undefined: DecodeAddress`). The plan's own Source-of-truth prose correctly states the call is `address.DecodeAddress(addr, net)`, so an implementer following the prose resolves it correctly (which I did). This is a code-block-vs-prose inconsistency in the plan text, not a logic defect; recommend the author qualify the code-block line as `address.DecodeAddress(...)` for a clean copy-paste, but it does not block GREEN (the intended, prose-specified form builds and all tests pass).

No CRITICAL or IMPORTANT findings. All R0 findings (C1/I1/I2/I3/M1–M5) are closed with build+test evidence; no new compile error, contradiction, or regression introduced by the folds.

## Verdict
`GREEN — 0 Critical / 0 Important`

Worktree `/scratch/code/shibboleth/seedhammer-wt-t3-r1check` was removed; fork left clean at `d334861` (HEAD unchanged, working tree clean); nothing committed, merged, or pushed.
