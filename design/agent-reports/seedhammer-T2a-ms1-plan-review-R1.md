<!--
Persisted verbatim. opus-architect R1 gate of the T2a ms1-decode plan after folding R0
(IMPLEMENTATION_PLAN_seedhammer_T2a_ms1_decode.md @ 188331e). Reviewer agentId af017a9ecc86481cd.
Verdict: GREEN 0C/0I (3 minor, all non-blocking). The reviewer applied the folded plan in a
throwaway off 68e6ead and RAN it: TestConfirmCodex32UnsharedNoRecover PASSES in 0.00s (the R0 45s
hang is gone — the DecodeMS1-success gate makes showSecret=false for the non-m-format ms10tests…
secret → Button2 inert → engrave); TestConfirmShowSecretGate + all new TestMS1Decode*/TestDecodeMS1*
+ the 24-word paging test pass; the Step-4 edits matched codex32_polish.go:98-122 verbatim (I-1
closed); paging genuinely spans 5 pages for 24 words (last word reachable only via paging — M-1
closed); vet+gofmt+TestAllocs clean; scope/secrecy intact; spec §2.7 matches. Minors: M-a the
24-word test's mid-word want-keys collapse on zero-entropy ABANDON×23 (only ART requires paging) —
correctly proves last-word-via-paging but the mid assertion is redundant (optionally match full
"N LABEL" lines); M-b showError text dead-end (gate ensures decode succeeded); M-c TinyGo CI-deferred.
Disposition: GREEN — proceed to single-implementer TDD (note M-a tightening + M-b to the implementer).
The text below is the agent's report exactly as returned; do not edit.
-->

# R1 GATE REVIEW — T2a ms1 decode (plan)

**Plan (folded):** `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_seedhammer_T2a_ms1_decode.md` (committed `188331e`)
**Prior R0:** `design/agent-reports/seedhammer-T2a-ms1-plan-review-R0.md` (NOT GREEN 1C/1I)
**Method:** Materialized the folded plan's exact code in a throwaway detached worktree off fork `68e6ead` (Go 1.26.4 at `/home/bcg/.local/go/bin/go`), applied the Step-4 edits via literal string match, built, and ran every relevant test plus targeted diagnostics. Per the brief I did NOT re-verify the R0-cleared items (decoder math, 5 parity vectors, panic-guard, secrecy/reuse/scope) — only the folds. Worktree removed; fork untouched at `68e6ead`, clean tree. Nothing committed/merged/pushed.

## Verification Results

**I-1 (anchors textually applicable) — CONFIRMED.** All three Step-4 edits applied via exact-string match against the real `gui/codex32_polish.go`:
- (a) probe inserted after the 3 Clickable decls (`:98-100`), before `for !ctx.Done {` (`:101`).
- (b) recover-click `old_string` matched `:108-111` verbatim (`recoverClicked := recoverBtn.Clicked(ctx)` / `if !f.Unshared && recoverClicked { return codex32Recover }`).
- (c) nav-append `old_string` matched `:116-122` verbatim (multi-line `navBtns := []NavButton{...}` + `if !f.Unshared { append }`).
Each Edit succeeded on first try → the plan's "Replace the EXACT block" quotes are literally applicable; no citation decay.

**Build:** `go build ./...` → BUILD_EXIT=0. `assets.IconInfo` resolves.

**C-1 fix — the load-bearing re-check. `go test -timeout 90s ./gui/ ./codex32/ ./bip39/` → all `ok` (gui 11.09s, codex32 0.008s, bip39 0.107s), no hang, no timeout.** Verbose:
```
--- PASS: TestConfirmCodex32Unshared (0.00s)
--- PASS: TestConfirmCodex32Share (0.00s)
--- PASS: TestConfirmCodex32ShareOffersRecover (0.00s)
--- PASS: TestConfirmCodex32UnsharedNoRecover (0.00s)   <-- the R0 hang; now passes instantly
--- PASS: TestRecoverCodex32 (0.00s)
--- PASS: TestRecoverCodex32Mismatch (0.00s)
--- PASS: TestMS1DecodeFlowEnglishWords (0.00s)
--- PASS: TestMS1DecodeFlowNonEnglish (0.01s)
--- PASS: TestConfirmShowSecretGate (0.00s)
--- PASS: TestMS1DecodeFlowPaging24Words (0.02s)
--- PASS: TestDecodeMS1Parity (all 5 subcases incl. mnem-japanese16)
--- PASS: TestDecodeMS1Refusal
```
**`TestConfirmCodex32UnsharedNoRecover` PASSES in 0.00s (vs the R0's 45s-timeout hang).** Mechanism re-confirmed: `ms10tests…` → `Seed()[0]=0x31` → `DecodeMS1` returns `errMSBadPrefix` → `showSecret = f.Unshared && (msErr==nil) = false` → Button2 inert → `click(Button2,Button3)` → `codex32Engrave`. `TestConfirmShowSecretGate` (decodable `entr`) opens the decode view. Share path intact.

**M-1 — paging genuinely exercised.** Instrumented partition over the 24-word secret at 240×240 (content height 152px): `total lines=24`, `page 1 = 5 lines (0..4)`, `pages needed=5`. Index 23 (last) only on page 5, reachable solely by paging; the test clicks Button3 across 40 frames and asserts the last word appears — passes, proving gap-free forward advance.

**Drift / no-regression:** `go vet ./codex32/ ./gui/` clean. `gofmt -l` silent. `TestAllocs` PASS (confirmCodex32Flow not alloc-gated). `git status` = exactly the 5 manifest files (1 modified +15/-4, 4 new); md1/mk1 + engrave untouched. The probe `_, _, _, msErr := codex32.DecodeMS1(scan)` discards prefix/lang/entropy → no secret retained. Spec §2.7 matches the plan's refined gate verbatim.

## Findings

### CRITICAL — none. C-1 resolved (gated showSecret → Button2 inert for the non-m-format secret → existing test passes, no hang).
### IMPORTANT — none. I-1 resolved (Step-4 blocks match `codex32_polish.go:108-111`/`:116-122` exactly).

### MINOR
- **M-a (M-1 mid-word assertion vacuous; non-blocking):** the 24-word zero-entropy mnemonic is `ABANDON×23 + ART`; the `want` map keyed by bare label collapses indices 0 and 11 to `"ABANDON"` (satisfied by page 1). The test genuinely proves *paging reaches the last word* (only `"ART"`, index 23/page 5, requires paging — catches a "stops early" regression), but a mid-page skip wouldn't be caught and the plan prose overstates ("first/mid/last positions"). The plan's own Note (line 357) offers the fix: match the full `"N LABEL"` line so the index prefix disambiguates. Cosmetic/test-strength; paging-to-end is verified.
- **M-b (carried R0 M-2):** `showError` body surfaces the Go sentinel; now dead-end (the gate ensures `ms1DecodeFlow` is entered only when `DecodeMS1` already succeeded). Cosmetic.
- **M-c (carried):** TinyGo not local; device build CI-deferred. `go build ./...` passes; pure byte-slice decode + existing primitives. Confirm in CI before merge.

## Verdict
C-1 holds (existing test passes instantly, no hang; shares get Recover; the decodable secret opens the view). I-1 anchors match verbatim, edits applied cleanly. Paging genuinely exercised (5 pages/24 words). All decoder/display/regression/alloc tests pass; vet+gofmt clean; scope/secrecy intact; §2.7 matches. The three remaining items are minors (one a test-strength nit the plan already documents a fix for).

**GREEN — 0 Critical / 0 Important.**
