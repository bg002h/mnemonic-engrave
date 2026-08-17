# S6a step 8 — mechanical comment re-sweep

Repo: `/scratch/code/shibboleth/wt-s6a`, branch `s6a-singlesig-truth`, HEAD `bf453671b82ed066785e1bde1bc064660a25ccea` ("S6a step 7: the document says what the verify observed, on all three flows").

Scope: read-only mechanical sweep of the ten named production files for comments carrying a checkable factual claim (count, call-site, "returns X on Y", "only"/"every"/"both"/"no caller does Z") that the code contradicts. Design, style, naming, test coverage and code correctness are explicitly out of scope. The three already-known findings (bundle_flow.go ~535 "both engraving callers", multisig_verify.go ~78 "FOUR OUTCOMES", bundle_flow.go ~126 "(nil,false) on Back / an empty bundle") were re-checked only for *additional* falsity beyond what is already recorded — none was found, so they are not repeated below.

All ten files were read in full (5291 lines total: `wc -l` on the ten files). Every finding below was resolved against the current worktree with the quoted command and its literal output — nothing was judged from a symbol name, a doc comment, or another comment's say-so.

## Findings

### 1. `gui/multisig_build.go:873` — "three walk drivers" undercounts by one

**Comment (lines 871-876):**
```
// THE FIRST SCREEN IS THE SHIPPED ONE, CHARACTER FOR CHARACTER. "Which slot is
// your key?" is a pinned walk needle with exactly one production site
// (cmd/emu/needle_test.go), three walk drivers anchor on it, and its default row
// is @0. So accepting every default produces {@0}: the pre-S5 single-select
// behaviour unchanged, which is what keeps every existing test and walk meaning
// what it meant.
```

**Claim:** exactly three walk-driver scripts anchor on the `"Which slot is your key?"` needle.

**Evidence it is false:**
```
$ grep -rl 'NEEDLE_SLOT = "Which slot is your key?"' cmd/emu/*.js
cmd/emu/walk_s3_nested.js
cmd/emu/walk_build_policy.js
cmd/emu/walk_trace_b.js
cmd/emu/walk_s4_gate.js
$ grep -rl 'NEEDLE_SLOT = "Which slot is your key?"' cmd/emu/*.js | wc -l
4
```
All four are independent, self-described walk drivers (each file's own header: "The BUILD-POLICY driver...", "S3's walk: build a NESTED-SEGWIT sh(wsh) policy...", "TRACE B: the flagship...", "S4's walk: DRIVE THE SEED<->KEY GATE..."), and each actively uses its `NEEDLE_SLOT` constant (`choose(...)` + `proven.push(NEEDLE_SLOT)`), confirmed for all four:
```
$ grep -n "NEEDLE_SLOT" cmd/emu/walk_build_policy.js cmd/emu/walk_s3_nested.js cmd/emu/walk_trace_b.js cmd/emu/walk_s4_gate.js
... (each file: one `export const NEEDLE_SLOT = ...` line plus a `choose(k - 1, n, NEEDLE_SLOT, ...)` and `proven.push(NEEDLE_SLOT)` line)
```
The "exactly one production site" half of the sentence is correct (verified separately: `"Which slot is your key?"` occurs in production Go source only at `gui/multisig_build.go:889`, the `ChoiceScreen.Lead`). Only the walk-driver count is wrong: it is four, not three.

**Confidence:** certain.

### 2. `gui/bundle_flow.go:377` — wrong line number for `deriveXpubFlow`'s `multiPlateEngrave` call

**Comment (lines 365-368):**
```
// bundleEngrave is the Phase-3 guided verbatim engrave. It is a SIBLING of
// multiPlateEngrave (R0-M2: Go has no default params; deriveXpubFlow's call site
// at derive_xpub.go:162 stays BYTE-UNCHANGED), reusing the same per-plate
// validateMdmk + ChoiceScreen + NewEngraveScreen machinery. ...
```

**Claim:** `deriveXpubFlow`'s call site of `multiPlateEngrave` is at `gui/derive_xpub.go:162`.

**Evidence it is false:**
```
$ grep -n "multiPlateEngrave" gui/derive_xpub.go
390:		multiPlateEngrave(ctx, th, strs)
491:func multiPlateEngrave(ctx *Context, th *Colors, strs []string) {

$ grep -n "^func deriveXpubFlow" gui/derive_xpub.go
330:func deriveXpubFlow(ctx *Context, th *Colors) {
```
`deriveXpubFlow`'s actual call to `multiPlateEngrave` is at line 390 (within the function that starts at 330), not 162. Line 162 belongs to a different function entirely:
```
$ awk 'NR<=162 && /^func /{last=NR": "$0} END{print last}' gui/derive_xpub.go
149: func seedEntryFlowTypedOnlyTitled(ctx *Context, th *Colors, title, wordPrefix string) (bip39.Mnemonic, bool) {
```
Line 162 is inside `seedEntryFlowTypedOnlyTitled`'s body, unrelated to `multiPlateEngrave` or `deriveXpubFlow`.

**Confidence:** certain.

### 3. `gui/multisig.go:34` and `gui/singlesig.go:25` — wrong line number for `seedEntryFlowTypedOnly` (same false citation, mirrored in both files)

**Comment (identical text, multisig.go lines 30-36 / singlesig.go lines 20-27):**
```
//     seedEntryFlow is the SOURCE PICKER (systemwide payload / keyboard / scan,
//     gui/derive_xpub.go:88) ...
//     seedEntryFlowTypedOnly (gui/derive_xpub.go:124), which the VERIFY flows
//     call so a payload-sourced secret is never compared against itself (§7.4).
```

**Claim:** `seedEntryFlowTypedOnly` is at `gui/derive_xpub.go:124`.

**Evidence it is false:**
```
$ grep -n "^func seedEntryFlowTypedOnly" gui/derive_xpub.go
140:func seedEntryFlowTypedOnly(ctx *Context, th *Colors) (bip39.Mnemonic, bool) {
149:func seedEntryFlowTypedOnlyTitled(ctx *Context, th *Colors, title, wordPrefix string) (bip39.Mnemonic, bool) {

$ sed -n '124p' gui/derive_xpub.go
		// sources rather than dropping the operator out of seed entry.
```
Line 124 is a comment inside the *previous* function, `seedEntryFlow` (whose own citation, `gui/derive_xpub.go:88`, is correct — confirmed `func seedEntryFlow` is exactly at line 88). `seedEntryFlowTypedOnly` is actually declared 16 lines later, at 140. Both `gui/multisig.go:34` and `gui/singlesig.go:25` carry the identical wrong citation.

**Confidence:** certain.

### 4. `gui/multisig_restore.go:44` — self-referential line citation points at prose, not at `desc4Display`'s call site

**Comment (lines 43-51):**
```
	// SPEC §4.4 says of the three sh-rooted names that "the restore document is
	// the one that matters most", and §2.2 D-3 names gui/multisig_restore.go:51
	// as the call site that matters. MEASURED 2026-08-15, running S3's own gate
	// for the first time: that call site is desc4Display, which sits on the
	// display-only branch ABOVE, and a full-policy build never reaches it. ...
```

**Claim:** `gui/multisig_restore.go:51`, in the current file, is the `desc4Display` call site on the display-only branch above this comment.

**Evidence it is false:**
```
$ grep -n "desc4Display(" gui/multisig_restore.go
29:		lines = append(lines, chunkString(desc4Display(tpl), 20)...)
64:	lines = append(lines, chunkString(desc4Display(tpl), 20)...)
79:func desc4Display(tpl md.Template) string {

$ sed -n '51p' gui/multisig_restore.go
	// the nested-segwit NAME on the restore doc) with nothing to read.
```
The display-only branch's `desc4Display` call (the one the comment says line 51 names) is actually at line 29. Line 51 in the current file is itself a line of this same comment's prose ("...with nothing to read."), not code. The comment's own "MEASURED 2026-08-15" verification clause asserts the citation resolves to `desc4Display`; in the current file it resolves to nothing (a comment line).

**Confidence:** certain (the file's own text names what line 51 is supposed to be and it demonstrably is not that).

## Checked and confirmed TRUE (no finding) — notable claims verified for completeness

For transparency, these are the higher-value checkable claims that were resolved against the code and found accurate, so they are not repeated as findings:

- `bundleGatherFlow` has five call sites, one of them `bundleFlow` (bundle_flow.go:132) — confirmed via grep (bundle_flow.go, multisig.go, singlesig_verify.go, multisig_build.go, multisig_verify.go).
- `bundleEngrave` is called from "the bundle flow, single-sig, the supplied-md1 path and Build alike" (bundle_flow.go:378-380) — confirmed 4 call sites in exactly those four files.
- `bundle_flow.go`'s "both engraving callers" self-corrected text at line ~536 ("three callers carry a post-engrave tail... All three tail-carriers now gate") matches the actual 3-vs-1 split found by grep (already-settled item, re-verified, no additional falsity).
- `multisigVerifyResult` has FIVE constants (already-settled item, re-verified: `verifyComplete/verifyIncomplete/verifyFailed/verifyRefused/verifyAbandoned`).
- `multisigVerifyNoExpectationBody` is "ONE STRING, TWO SITES" — confirmed exactly 2 uses (multisig_verify.go:710, 847).
- `multisigVerifyRetryLead` is used by "both engraving callers" — confirmed exactly 2 uses (multisig.go:349, multisig_build.go:464).
- `multisigVerifyFn` is dispatched through by "BOTH engrave callers" — confirmed exactly 2 call sites (multisig.go:345, multisig_build.go:460).
- `multisigRestoreDocFlow` — "BOTH ENGRAVING CALLERS PASS ONE" (a non-nil `extra`) — confirmed exactly 2 callers (multisig.go:374, multisig_build.go:491), neither passes nil.
- `classifyCosignerSupply`'s switch is exhaustive over its 3 outcomes (`cosignerRefuse`/`cosignerAutoFill`/`cosignerSelect`) — confirmed by enumerating the const block and the switch's case list.
- `singleSigVerifyFlow` — "ELEVEN EXITS... Ten explicit returns plus the fall-through," "TWO OF THE ELEVEN ARE ADVERSE and eight write NEITHER bit" — confirmed: exactly 10 `return` statements in the function body, exactly 2 `rec.adverse = true` sites (10 - 2 = 8 "neither" exits + 1 fall-through pass = 11).
- `multisigVerifyOKMessage`'s four arms all end "Other cosigners' keys are taken as supplied." — confirmed by reading all four return statements (lines ~1123-1132).
- `singleSigEngraveCards` — "full = 3 cards incl. the secret ms1; watch-only = 2 cards" — confirmed by reading the function body.
- `extractSuppliedMd1`'s refusal text "Supply exactly one wallet-policy md1 (and no key cards)" — confirmed matches the function's actual gating (count != 1 refuses; any cardMK1/cardMS1 refuses).
- "the T5 gather path never produces a cardMS1" — confirmed: `cardMS1` is constructed only in `singlesig_engrave.go:24` and `multisig_engrave.go:36` (engrave-construction paths), never inside `bundleGatherer.offer`/`offerChunkedMK1`/`offerChunkedMD1`/`offerStandaloneMD1`.
- Several other cross-file line citations were spot-checked and are accurate: `gui/derive_xpub.go:88` (seedEntryFlow's own func line), `gui/singlesig_restore.go:118` (inside restoreDocFlow's doc comment), `gui/md1_expand.go:36-49` (expandedToDescriptor's nil-descriptor returns), `gui/ms1_decode.go:29` (`defer wipeBytes(entropy)`), `md/encode_multisig.go:96-106` (both OriginShared/OriginDivergent empty-origin refusals), `gui/singlesig_verify.go:49` (verifySingleSig's own func line), `bundle/verify.go:71-79` (ms1 presence-mismatch logic).

## Files swept and found clean (beyond the findings above)

All ten files were read in full and their checkable claims resolved against the code; no further falsities were found beyond items 1-4:

- `gui/bundle_flow.go` (562 lines) — clean except finding #2.
- `gui/multisig.go` (402 lines) — clean except finding #3 (multisig.go half).
- `gui/multisig_build.go` (1797 lines) — clean except finding #1.
- `gui/multisig_build_census.go` (390 lines) — clean, no checkable-and-false claims found.
- `gui/multisig_restore.go` (115 lines) — clean except finding #4.
- `gui/multisig_verify.go` (1133 lines) — clean; this file carries the already-settled "FOUR OUTCOMES" defect (not repeated here) but no further falsity was found.
- `gui/singlesig.go` (224 lines) — clean except finding #3 (singlesig.go half).
- `gui/singlesig_restore.go` (199 lines) — clean, no checkable-and-false claims found.
- `gui/singlesig_verify.go` (203 lines) — clean, no checkable-and-false claims found (the "ELEVEN EXITS" claim was checked and is TRUE).
- `gui/verify_status.go` (267 lines) — clean, no checkable-and-false claims found ("THREE production flows reach a restore document" was checked and is TRUE: `restoreDocFlow` × 1 caller + `multisigRestoreDocFlow` × 2 callers = 3).

## Summary

4 findings, all "certain" confidence, all stale/wrong line-number or count citations (not design or style issues):

1. `gui/multisig_build.go:873` — "three walk drivers anchor on it" should be four.
2. `gui/bundle_flow.go:377` — "derive_xpub.go:162" should be 390 (line 162 is a different function).
3. `gui/multisig.go:34` and `gui/singlesig.go:25` (identical mirrored text) — "gui/derive_xpub.go:124" should be 140.
4. `gui/multisig_restore.go:44` — "gui/multisig_restore.go:51" should be 29 (the display-only branch's actual `desc4Display` call).
