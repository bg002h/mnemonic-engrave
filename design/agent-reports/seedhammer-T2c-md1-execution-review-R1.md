<!--
Persisted verbatim. opus-architect R1 re-dispatch of the MANDATORY whole-diff execution review, after
the author folded the R0 (0C/1I) findings into commit 1e376d9. Reviewer agentId a7dd09390927019bb.
Verdict: GREEN 0C/0I. All three folds CLOSED with re-run + re-fuzz (122.3M execs, 0 panics) + independent
md-codec 0.36.0 `md` CLI provenance verification of the 4 FOLD-C wire strings (decoded to exactly the
asserted structures). IMPORTANT-1 closed (TestDecodeNegative 9/9 by errors.Is category, no panic, no
partial Template; TestDecodeRenderableClassification 4/4 decoding REAL wires incl. tr(NUMS,sortedmulti_a)
→ Renderable=false + is_nums cursor; provenance test 3/3). MINOR-1 closed (availBits guard; the exact R0
panic input → errTruncated; no over-restriction). MINOR-2 closed (classifyPolicy: tr renderable only
when !isNums && tree==nil; any tr-with-tree → PolicyComplex/Renderable=false; dead multiAPolicy removed;
tr_keyonly still renderable — no regression). Whole-module go test ./... zero failures; vet/gofmt/build/
TestAllocs clean. One residual MINOR: harmless pre-existing dead policyLine enum arms in gui (non-blocking,
optional future cleanup). Worktree left as-is at 1e376d9, nothing merged/pushed. Disposition: GREEN —
merge no-ff → push bg002h. Text below verbatim (entities un-escaped: &lt;→<, &gt;→>, &amp;→&).
-->

# ADVERSARIAL WHOLE-DIFF EXECUTION REVIEW (R1 re-dispatch) — feat/md1-decode-display @ 1e376d9

## Context
R1 re-dispatch after the author folded R0 (NOT GREEN — 0C/1I) findings into commit `1e376d9` (+461/−18 across `md/{md.go,bits.go,bits_test.go,md_test.go}`). Worktree `seedhammer-wt-t2c-md1`, base `2fed9b6`, 6 commits. Re-run + re-fuzz, implementer report NOT trusted. go1.26.4.

## Fold verification

**IMPORTANT-1 (coverage gap) — CLOSED.** Three new test funcs ship and pass:
- `TestDecodeNegative` (md_test.go:152) — 9 white-box subtests, all asserting by `errors.Is` category (NOT string-equality, per spec §6 line 104), each with panic-recovery + a partial-Template assertion (`d != nil` on error fails). Categories covered: wire-version≠4 (`errWireVersion`), `0x3F` extension tag + `0x24` reserved tag (`errTagOutOfRange`), non-canonical root multi (`errOperatorContext`), K>N (`errKGreaterThanN`), depth>128 via 130 nested `sh` (`errDepthExceeded`), truncation-after-header (`errTruncated`), placeholder-not-referenced (`errPlaceholderNotReferenced`), placeholder-first-occurrence-out-of-order (`errPlaceholderOrder`). All 9 PASS.
- `TestDecodeRenderableClassification` (md_test.go:331) — decodes 4 REAL md-codec-encoded md1 wire strings via the public `Decode(s)` (NOT hand-built Template literals): wsh(and_v)→Renderable=false/PolicyComplex; explicit-origin sh(multi)→Renderable=true/PolicyMulti 2-of-2; elided sh(multi)→`errMissingExplicitOrigin` reject; tr(NUMS,sortedmulti_a(2,@0,@1,@2))→Renderable=false/PolicyComplex while decoding 3 keys (the is_nums variable-width cursor). All 4 PASS.
- `TestDecodeRenderableBytecodeProvenance` (md_test.go:450) — re-decodes the recorded hex payloads; all 3 PASS.

**MINOR-1 (bit-reader guard) — CLOSED.** `bits.go` adds `availBits() = min(remaining(), len(bytes)*8 - bitPos)` (clamped ≥0); `read()` now checks `availBits() < count → errTruncated` (was `remaining() < count`). `TestBitReaderLimitExceedsSlice` (bits_test.go:45) proves the exact R0 panic input `newBitReader([]byte{},64).read(5)` → `errTruncated` (no panic, recover-wrapped), AND that `availBits` does NOT over-restrict: `newBitReader([]byte{0xFF},64).read(8)` returns `0xFF` then `read(1)`→`errTruncated`. Existing `TestBitReader` (bitLimit=3<8) still green → legitimate `bitLimit<len*8` reads unaffected. PASS.

**MINOR-2 (elevated, tr-with-tree) — CLOSED.** `classifyPolicy` (md.go:1243) tagTr arm now returns `PolicySingle` ONLY when `!b.isNums && b.tree == nil`; ANY tr with a tree falls through to `PolicyComplex`/Renderable=false. `multiPolicy` (md.go:1283) matches only `tagMulti`/`tagSortedMulti` — the dead `multiAPolicy` helper is removed (grep: gone) and no decode path produces `PolicyMultiA`/`PolicySortedMultiA`. The tr(NUMS,sortedmulti_a) subcase confirms Renderable=false while still decoding 3 keys; `tr_keyonly` (corpus, tree=nil) STILL Renderable=true single-key (`TestDecodeParity/tr_keyonly` PASS — no regression).

**Independent provenance check (md-codec 0.36.0 `md` CLI, NOT trusting the embedded strings):** all 4 FOLD-C strings decode via `md decode`/`--json` to exactly the asserted structures — wsh(and_v(v:pk(@0),older(144))) n=1 origin m/48'/0'/0'/2'; sh(multi(2,@0,@1)) n=2 origin m/48'/0'/0'/1'; elided sh(multi)→"non-canonical wrapper requires explicit origin for @0"; tr(50929b74…ac0, sortedmulti_a(2,@0,@1,@2)) n=3 is_nums=true origin m/48'/0'/0'/2'. The recorded hex bytecodes match. Origins and N/root/policy all match the test assertions verbatim.

## Regression + fuzz

- `go test -count=1 ./...` (whole module) → every package `ok`, ZERO failures (md 0.003s, gui 5.547s, mk, codex32, all GUI/engrave/stepper).
- Named load-bearing tests verbose: `TestDecodeParity` (all 9 subvectors), `TestDecodeChunkedRefused`, `TestDecodePayloadAST`, `TestDecodeNegative` (9/9), `TestDecodeRenderableClassification` (4/4), `TestDecodeRenderableBytecodeProvenance` (3/3), `TestMdmkFlowMK1ShowsInspect`, `TestMdmkFlowMD1ShowsInspect`, `TestAllocs`, `TestMD1DisplayFlowPaging`, `TestMD1DisplayFlowComplexRefuses` — all PASS.
- `go build ./...` OK; `go vet ./codex32/ ./md/ ./gui/` clean; `gofmt -l codex32/ md/ gui/` empty.
- **Re-fuzz of the fold-touched hot paths (scratch harness, run then deleted; ~122.3M total execs, 0 panics, 0 crashers):**
  - `FuzzDecodeScratch` (public Decode, full pipeline incl. tightened classifyPolicy): ~30.0M execs, 0 panics
  - `FuzzDecodePayloadValidatedScratch` (constrained bitLen∈[0,len*8], decodePayload+validators+summarize): ~34.0M execs, 0 panics
  - `FuzzDecodePayloadUnconstrainedScratch` (deliberately bitLen>len*8 — hammers FOLD-B availBits): ~26.9M execs, 0 panics — proves the guard converts OOB→errTruncated, never panic
  - `FuzzBitReaderScratch` (read() with arbitrary count+bitLimit): ~31.4M execs, 0 panics
  - Confirmed availBits does NOT wrong-truncate valid decodes (9 parity vectors green + the explicit FOLD-B success-read assertion).

## Findings

- CRITICAL: none.
- IMPORTANT: none.
- MINOR (non-blocking, informational): `gui/md1_inspect.go:44-47` `policyLine` retains dead `case md.PolicyMultiA`/`PolicySortedMultiA` arms. These are now unreachable — `classifyPolicy` never produces those kinds, and `md1Summary` only calls `policyLine` when `tpl.Renderable`. Harmless dead code; predates the fold (the fold was correctly scoped to `md/` only). Optional future cleanup, not a defect.

## Scope / provenance / hygiene
- `git diff --stat 2fed9b6..HEAD` = only the 11 manifest files (+2453/−22). Fold commit `1e376d9` touched ONLY `md/{bits.go,bits_test.go,md.go,md_test.go}` — nothing outside.
- 6th commit: SSH-signed (`gpgsig -----BEGIN SSH SIGNATURE-----` blob present; local `sig=N` is only the missing `allowedSignersFile` verifier), DCO `Signed-off-by: Brian Goss`, `Co-Authored-By: Claude Opus 4.8`, authored Brian Goss.
- Scratch fuzz file deleted; no `md/testdata/` corpus leaked; `git status` clean; HEAD unchanged at `1e376d9`. No merge/push/commit/branch modification performed; worktree left as-is.

## Verdict

**GREEN — 0 Critical / 0 Important**

(All three R0 findings closed with confirming re-run + re-fuzz evidence and independent md-codec CLI provenance verification of the FOLD-C wire strings. The decode logic remains a faithful, panic-free port; the previously-missing reject/classify/is_nums-cursor regression guards now ship and pass. The one residual MINOR is harmless pre-existing dead GUI code, non-blocking.)
