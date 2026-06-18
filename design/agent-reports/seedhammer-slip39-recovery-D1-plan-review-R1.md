<!--
Persisted verbatim. opus-architect R1 re-dispatch of the Cycle D D1 plan R0 gate
(@ 24cb9e1). Reviewer agentId a75fe71856204a537. Verdict: NOT GREEN 0C/1I — the fold's
intent was right and the crypto untouched, but stale residue survived the restructure
(I1-residue lines 398-400 reinstating inline literals; M2/M3 residue; stale Task 6 header).
Disposition: all four swept. The text below is the agent's report exactly as returned; do not edit.
-->

# R1 GATE REVIEW — Cycle D D1 plan (SLIP-39 crypto port)

**Reviewer:** opus architect (adversarial R1 re-dispatch of the R0 gate, read-only)
**Plan (folded):** `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_seedhammer_slip39_recovery_D1.md` @ `24cb9e1`
**R0 followed up:** `design/agent-reports/seedhammer-slip39-recovery-D1-plan-review-R0.md` (0C/1I + M1–M6)
**Spec:** `SPEC_seedhammer_slip39_recovery.md` §0/§4/§7 (R1 GREEN)
**Base:** fork `main` `20fa4c4`; oracle `mnemonic-toolkit/.../slip39/*.rs`
**Date:** 2026-06-18

## Scope of this review
The crypto port (gf256/lagrange/feistel/combine/share value-extraction) was **execution-proven correct at R0** against all official positive vectors (incl. 256-bit/33-word and extendable), the full negative corpus, and panic-safety. I re-verified that the fold did **not** touch that code, and focused on (a) correctness of the I1 + M1–M6 folds and (b) no-drift. I did not re-execute the crypto.

## Verification Results

**Crypto port UNCHANGED by the fold (no drift into proven code) — CONFIRMED.**
Task 1 (`gfMul(0x53,0xCA)==0x01`, generator 3, `init` loop, `gfInv (255-log)%255`), Task 2 (`interpolateAt`/`interpolateSecretAt`, `secretIndex=255`/`digestIndex=254`/`digestLen=4`), Task 4 (round order `i=3..0`, `l[j]^=f[j]`+swap, output `r||l`, salt `"shamir"||be16(id)`/`nil`, `itersPerRound=(10000<<e)/4`), and Task 5 (`Combine`/`recoverSecret`/`ConsistentShares`/`wipe`, six cross-share sentinels, `subtle.ConstantTimeCompare` digest gate) are byte-for-byte the R0-executed code. `decodeValue` (Task 3 Step 3) is unchanged. Confirmed clean.

**I1 — partially folded; the inline-literal CLASS is re-opened by stale residue.**
- Task 0 now front-loads `testdata/slip39_vectors.json` (VERBATIM upstream, 4-tuple `[desc,[mnemonics],master_hex,xprv]`, indices 0,3,17,35,42,1,4,5,9,12,13) + the loader helpers `loadVectors`/`vectorShares`/`vectorShare`/`vectorSecretHex` in `slip39/vectors_test.go` (lines 58–113). The `slip39Vector` struct, `os.ReadFile("testdata/slip39_vectors.json")`, and `[][]json.RawMessage` parse shape match the real vectors.json. CONFIRMED.
- The fabricated `"shadow pistol … mustard …"` literal is **gone** (`grep mustard|mustang|"shadow pistol"` → no hits). CONFIRMED.
- Task 3 Step 1 `TestParseShareExtractsValue`/`TestParseShareGroupThresholdExceedsCount` now load via `vectorShare(t,3,0)` (line 318), `vectorShare(t,35,0)` (line 327), `vectorShare(t,9,0)` (line 336) with NO inline literal, using `errors.Is` (line 337). CONFIRMED.
- Self-review adds the no-inline-literal grep guard (lines 772–775: "`grep` the test files for hand-typed 20+-word strings and `[0-9a-f]{32}` literals → none"). CONFIRMED.
- **BUT (drift) — Task 3 Step 4, lines 398–400** still reads: *"(Provide `vectorShare`/`errorsIs` helpers — Task 6 adds the JSON loader; for this task, **inline the idx-3 and idx-9 mnemonics as string literals if Task 6 hasn't landed yet**, then dedupe in Task 6.)"* This is the I1 transcription-risk class reinstated verbatim, in an actionable step body, and it contradicts (a) the Task 0 I1-fold note (lines 54–56: "no crypto mnemonic or secret literal is ever hand-typed"), (b) the Step 1 directive directly above it (line 314: "NO inline mnemonic literals"), and (c) the front-loading premise (the loader is now Task 0, not "Task 6 hasn't landed yet"). An implementer reading Step 4 is told to hand-type the idx-3 mnemonic — exactly the fabrication path that produced the R0 blocker.

**M1 — orphaned consts: FOLDED.** Task 3 Step 3 (line 359) explicitly: "Also delete the now-unused `wordsShort`/`wordsLong` consts (`share.go:31-32`) — plan-R0 M1." CONFIRMED.

**M2 — `errorsIs` wrapper: NOT FULLY FOLDED.** The test body uses `errors.Is` directly (line 337) and the Task 0 note says "no `errorsIs` wrapper" (line 55), but **`errorsIs` survives as a to-be-provided helper at line 398 ("Provide `vectorShare`/`errorsIs` helpers") and line 723 (Task 6 Files header lists "`...hexEq`/`errorsIs`")**. M2 said to eliminate it. Two dangling references remain.

**M3 — delete (not flip) the 256-bit-reject test: FOLDED in the body, STALE in the checklist.** Task 3 Step 1 NOTE (lines 343–346) correctly says **DELETE** (not flip-to-expect-nil) both the `errUnsupportedSize` 33-word assertion and the `Describe {errUnsupportedSize,"256-bit not supported"}` case. **BUT the self-review checklist line 780 still reads "`share_test.go` 256-bit-rejection assertions inverted"** — the exact loose "invert" wording R0/M3 flagged, now contradicting its own Step-1 NOTE. Stale.

**M4 — explicit `{20,23,27,30,33}` gate as canonical: FOLDED.** Task 3 Step 3 item 3 (line 358): "Replace the `switch len(fields)` length gate with the **explicit** accepted set (M4 — canonical form): accept word count ∈ {20,23,27,30,33}". The "equivalently" alternative is gone. CONFIRMED.

**M5 — var-block gofmt alignment:** self-heals under the per-task `gofmt -l` guard (R0 declared non-blocking). Not re-checked; acceptable.

**M6 — fixture-regen wording: FOLDED.** Task 6 Step 2 (lines 729–739) now says the wedge "lives in the CLI layer (`MNEMONIC_SLIP39_TEST_RNG` + `MNEMONIC_SLIP39_TEST_IDENTIFIER`)", documents `slip39/testdata/GEN.md`, and states "`extendable` is hardcoded `false` on the CLI wedge path, so ext=1 coverage comes from official idx 42 (Step 3), not fixtures." CONFIRMED.

**Helper composition / ordering — one drift item.**
- `hexEq` is defined in `combine_test.go` (line 112, Task 5) and used there (line 517). Task 0 note correctly points to it. CONFIRMED. **But the Task 6 Files header (line 723) wrongly lists `hexEq` among helpers to create in `vectors_test.go`** — contradicting line 112 and Step 1 below it (lines 725–727: helpers "were created in Task 0… nothing to copy here").
- `errors` importability in `share_test.go`: the test uses `errors.Is` (line 337); the plan does not show the `share_test.go` import block, but `errors` is a stdlib import the implementer adds — no ordering hazard.
- **`vectorShare`-before-defined ordering: CLEAN.** Task 0 (front-loaded, before Task 3) defines `vectorShare`/`vectorShares`/`vectorSecretHex`/`loadVectors`. No task references a helper before its definition — **except** the line-398/399 residue, which tells the implementer the loader is not yet available ("if Task 6 hasn't landed yet"), reviving the pre-fold ordering assumption the front-load was meant to kill.

## Findings

### CRITICAL
None. The crypto is execution-proven and untouched by the fold; no precondition/panic-safety regression.

### IMPORTANT

**I1-residue — Task 3 Step 4 (lines 398–400) re-opens the exact inline-literal class R0/I1 closed.** The fold front-loaded the loader into Task 0 and rewrote Step 1, but Step 4's parenthetical still instructs: *"inline the idx-3 and idx-9 mnemonics as string literals if Task 6 hasn't landed yet."* This is the literal fabrication path that produced the R0 blocker, now self-contradicting the same task's Step 1 and Task 0 note. An implementer following Step 4 verbatim would hand-type the idx-3 mnemonic — re-creating the I1 defect. The I1 fold is not complete at the class level until this residue is removed.
**Required fix:** Delete the parenthetical at lines 398–400 entirely (the helpers exist from Task 0; there is no "Task 6 hasn't landed" case). Replace Step 4 with the plain run/vet/gofmt instruction. Remove the `errorsIs` mention while doing so.

### MINOR
- **M2-residue:** `errorsIs` survives at line 398 and line 723. Delete both (M2 = use `errors.Is` directly; the test body already does). 
- **M3-residue (self-review):** line 780 "256-bit-rejection assertions inverted" contradicts the Step-1 NOTE's correct "DELETE". Change "inverted" → "the `errUnsupportedSize` test + `Describe` case deleted".
- **Task 6 Files header (line 723) is stale:** it lists creating `vectors_test.go` and helpers `vectorShare`/`vectorShares`/`hexEq`/`errorsIs` as Task 6 work, but Task 0 creates `vectors_test.go` + loader helpers and `hexEq` lives in `combine_test.go` (Task 5). Contradicts Step 1 immediately below (lines 725–727). Rewrite the header to: testdata/loaders are Task 0, `hexEq` is Task 5; Task 6 only adds `slip39_fixtures.json` + the round-trip/negative/panic/scrub tests.

## Verdict

**NOT GREEN — 0 Critical / 1 Important.**

The fold's *intent* is correct and the crypto is untouched, but the fold was applied to the new/rewritten regions (Task 0, Task 3 Step 1/3) without sweeping the **stale residue** the restructure left behind. One residue item (I1-residue, lines 398–400) re-instates the precise inline-literal transcription path that was the original R0 blocker, inside an actionable step that contradicts its own task — that is Important, not cosmetic. The three minors (M2/M3 residue + the stale Task 6 header) are internal contradictions that should be cleaned in the same pass.

**Required fixes before GREEN:**
1. **I1-residue** — Delete the lines 398–400 parenthetical ("inline the idx-3 and idx-9 mnemonics as string literals if Task 6 hasn't landed yet"); the loader is front-loaded in Task 0. Make Step 4 a plain run/vet/gofmt step. (Important)
2. **M2-residue** — Remove `errorsIs` at lines 398 and 723. (Minor)
3. **M3-residue** — Self-review line 780: "inverted" → "deleted". (Minor)
4. **Task 6 header** — Line 723: stop listing Task-0/Task-5 helpers as Task 6 work; scope Task 6 to fixtures + tests. (Minor)

Fold these four (all confined to Task 3 Step 4, Task 6 Step header, and the self-review — no crypto-code touch), persist this review verbatim to `design/agent-reports/`, and re-dispatch for the GREEN re-confirmation.
