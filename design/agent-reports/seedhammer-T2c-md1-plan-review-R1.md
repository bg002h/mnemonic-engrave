<!--
Persisted verbatim. opus-architect R1 GATE re-review of IMPLEMENTATION_PLAN_seedhammer_T2c_md1_decode.md
(folded, commit 3d60559). Reviewer agentId a8dc1c0b906dcef56. Verdict: GREEN 0C/0I. Re-materialized +
re-PORTED the cited md-codec 0.36.0 Rust to Go (with the hasMultipath-augmented useSitePath), built +
ran. C1 CLOSED (Renderable:true → TestMD1DisplayFlowPaging passes, "multisig"/"deadbeef"/"cafebabe"
render) and NOT over-corrected (TestMD1DisplayFlowComplexRefuses still passes); M2 CLOSED (useSitePath
ports cleanly, 9/9 parity exact incl. <2;3>/* divergent use-site); M1 CLOSED (gofmt -w → gofmt -l empty).
Whole-module go test ./... exit 0, zero failures; build/vet/TestAllocs/mk1-path GREEN; no drift. Reviewer
re-confirmed elided origins surface OriginPath:"m" (never the canonical implied path) — the parity test
catches mis-substitution. Worktree removed; fork clean at 2fed9b6; nothing committed/merged/pushed.
Disposition: GREEN — cleared to single-implementer TDD. Text below verbatim (entities un-escaped:
&lt;→<, &gt;→>, &amp;→&).
-->

# R1 Gate Re-Review — IMPLEMENTATION_PLAN_seedhammer_T2c_md1_decode.md (folded, commit `3d60559`)

## Method
Re-materialized the folded plan in a detached worktree off fork base `2fed9b6`. Transcribed Tasks 1/2/5 verbatim; **re-PORTED** the cited md-codec 0.36.0 Rust to Go for Tasks 3/4 (`readTag`, the 9-variant `body` interface + `node`, `readVarint`, origin/use-site paths with the now-`hasMultipath`-augmented `useSitePath`, `readNode`, `readHeader`, the full `readTLV` with sparse-body bit-limit scoping, `decodePayload`, the 5 validators, `canonicalOrigin`, `symbolsToBytes`, `Decode`, `summarize`). Cross-checked every embedded vector against the live `tests/vectors/*` files. Built and ran everything. Worktree removed; fork clean at `2fed9b6`.

## Fold verification
- **C1 — CLOSED.** With the folded `Renderable: true` on the `TestMD1DisplayFlowPaging` literal, the test PASSES: `md1Summary` emits the `policyLine` ("2-of-2 multisig (multi)") and the per-key `deadbeef`/`cafebabe` lines, so `uiContains("multisig")`, `"deadbeef"`, `"cafebabe"` all hold. The whole-module `go test ./...` is GREEN (exit 0).
- **C1 not over-corrected — CONFIRMED.** `TestMD1DisplayFlowComplexRefuses` (the `Renderable: false` case) still PASSES — `md1Summary` emits "Complex policy — cannot display safely." and the test asserts the refusal copy.
- **M2 — CLOSED.** The `useSitePath{hasMultipath bool; multipath []alternative; wildcardHardened bool}` struct ports cleanly. `TestDecodeParity` matches all 9 Templates exactly, including the `<2;3>/*` divergent use-site (`wsh_divergent_paths` via its use-site-override TLV → `useSiteString`) and `<0;1>/*` shared default. `useSiteString` correctly gates the `<…>` group on `hasMultipath`.
- **M1 — CLOSED.** Before the Task-5 `gofmt -w`, `gofmt -l` flagged `md/md.go` (the `var (…)` sentinel-block / receiver-line alignment); after `gofmt -w`, `gofmt -l codex32/ md/ gui/` is empty. (Note: the `gui/mk1_inspect_test.go` artifact R0 saw did not reproduce — deleting the whole `TestMdmkFlowMD1NoInspect` function left no trailing blank line — so the Task-5 gofmt step is sufficient and harmless either way.)

## Drift check
Whole-module `go test ./...` → **exit 0, every package `ok`, zero failures**. Re-confirmed each named regression target GREEN: `TestDecodeParity` (9/9), `TestDecodeChunkedRefused` (→ `ErrChunkedUnsupported`), `TestDecodePayloadAST` (9/9 AST subcases), `TestMDDataSymbols`, `TestBitReader`, `TestMdmkFlowMK1ShowsInspect` (mk1 path unchanged), plus the new `TestMdmkFlowMD1ShowsInspect`/`TestHasMDPrefix`. `go build ./...` GREEN, `go vet ./codex32 ./md ./gui` GREEN, `TestAllocs` GREEN. No new compile error, contradiction, or test regression introduced by the folds.

One materialization note (NOT a plan defect, no finding): the R0-cleared parity fact holds — the renderable shapes surface `OriginPath: "m"` for elided origins (the decoded `path_decl`, per spec §4.2 R0-I3), **never** the canonical implied path. `canonicalOrigin` remains live solely as the gate inside `validateExplicitOriginRequired`. My first port draft briefly mis-substituted the canonical path and the parity test caught it immediately; the plan's literals (`m`) are correct and the corrected summarizer matches all 9. This re-confirms the test literals are the right GREEN proof.

## Findings
- **CRITICAL:** None.
- **IMPORTANT:** None.
- **MINOR:** None.

## Verdict
`GREEN — 0 Critical / 0 Important`

The worktree `/scratch/code/shibboleth/seedhammer-wt-t2c-r1check` was removed; the fork is left clean at `2fed9b6` with nothing committed, merged, or pushed.
