<!--
Persisted verbatim. opus-architect R0 GATE review of IMPLEMENTATION_PLAN_seedhammer_T2c_md1_decode.md
(commit 24b546f) BEFORE code. Reviewer agentId afadf8960d4b565a3. Method: materialized in a detached
worktree off 2fed9b6, transcribed Tasks 1/2/5, PORTED the cited md-codec 0.36.0 Rust to Go for Tasks
3/4 (readTag, 9-variant body, readVarint, paths, readNode, readTLV, decodePayload, 5 validators,
canonicalOrigin, symbolsToBytes, Decode, summarize), cross-checked vectors vs live tests/vectors/*,
built + ran everything. Verdict: NOT GREEN — 1C/0I. ALL 9 parity Templates match exactly; chunked-refuse
works; kiw correct; port faithful + implementable as written; build/vet/TestAllocs/mk1-path all GREEN.
The 1 Critical is a mechanical TEST-LITERAL bug (C1: TestMD1DisplayFlowPaging omits Renderable:true →
md1Summary renders the complex-refusal copy → "multisig" assertion fails). 2 MINORs: M1 gofmt artifacts
(auto-resolve; Task 6 gate catches), M2 useSitePath needs a hasMultipath bool to distinguish None from
Some([]) (plan struct sketch under-specified; reviewer added it during the port). Worktree removed; fork
clean at 2fed9b6; nothing committed/merged/pushed. Disposition: fold C1 + M1 + M2 → re-dispatch R1.
Text below verbatim (HTML entities un-escaped: &lt;→<, &gt;→>, &amp;→&).
-->

# R0 Gate Review — IMPLEMENTATION_PLAN_seedhammer_T2c_md1_decode.md (commit 24b546f)

## Method
I materialized the plan in a detached worktree at fork base `2fed9b6`, transcribed Tasks 1/2/5 verbatim, and **PORTED** the cited md-codec 0.36.0 Rust to Go for Tasks 3/4 (`readTag`, the 9-variant `body` interface + `node`, `readVarint`, origin/use-site paths, `readNode`, `readHeader`, the full `readTLV` with sparse-body scoping, `decodePayload`, all 5 validators, `canonicalOrigin`, `symbolsToBytes`, `Decode`, `summarize`). I cross-checked every embedded vector against the live `tests/vectors/*` files, built, and ran every test. Worktree removed; fork left clean at `2fed9b6`.

## Verification Results

| Check | Result |
|---|---|
| Task 1 `TestMDDataSymbols` | **GREEN** |
| Task 2 `TestBitReader` | **GREEN** |
| Task 3 `TestDecodePayloadAST` (all 9 AST subcases) | **GREEN** — incl. `deadbeef`/`cafebabe` FP extraction + `<2;3>` divergent use-site |
| Task 4 `TestDecodeParity` (9 single-string vectors) | **GREEN — all 9 Templates match exactly** (N, Root, Policy, K, M, per-key Index/Fingerprint/OriginPath/UseSite, Renderable) |
| Task 4 `TestDecodeChunkedRefused` | **GREEN** — `md1fz4aw…` → `ErrChunkedUnsupported` |
| `kiw` formula (n=1→0, 2→1, 3→2, 4→2, 5→3) | **correct** |
| chunked-flag (`syms[0]&1`): single=0, chunk=1, chunk line is BCH-valid md1 | **correct** |
| `validateXpubBytes` deferral | **consistent** — every corpus `descriptor.json` has `"pubkeys": null`; no-op for all; `md` deps = only `codex32`+stdlib (no btcec/bip380) |
| Structural rejects fire (version≠4, truncation, K>N, reserved tag) | **GREEN** (verified via scratch test; validators are live, not dead) |
| `go build ./...` | **GREEN** |
| `go vet ./codex32 ./md ./gui` | **GREEN** |
| `TestAllocs` | **GREEN** |
| `TestMdmkFlowMK1ShowsInspect` (mk1 path unchanged) | **GREEN** |
| Whole-module `go test ./...` | **1 failure**: `TestMD1DisplayFlowPaging` (see C1) |
| `gofmt -l` | **2 files** (`md/md.go`, `gui/mk1_inspect_test.go`) — see M1 |

`TestDecodeParity` all-9 match is the load-bearing proof, and it is GREEN. The port pattern is implementable exactly as written — every Go type/signature in Tasks 3/4 carried a faithful port with no guessing.

## Findings

### CRITICAL

**C1 — `TestMD1DisplayFlowPaging` test literal omits `Renderable: true`; the test as written cannot pass (Task 5, Step 1, plan lines 463-465).**
The test builds `md.Template{N: 2, Root: md.ScriptWsh, Policy: md.PolicyMulti, K: 2, M: 2, Keys: …}` and asserts the rendered output `uiContains("multisig")`. But `md1Summary` (plan lines 532-545) only emits the `policyLine` (which contains "multisig") **when `tpl.Renderable == true`**; otherwise it emits "Complex policy — cannot display safely." The literal leaves `Renderable` at its zero value `false`, so the screen renders the complex-refusal copy and the `"multisig"`/`"deadbeef"`/`"cafebabe"` assertion fails. I confirmed this is the sole cause: the entire rest of the module (all other GUI tests, `md`, `codex32`, every other package) is GREEN, and adding `Renderable: true` makes the full suite GREEN.
**Fix:** in the plan's Task 5 Step 1 `TestMD1DisplayFlowPaging`, set `Renderable: true` on the `tpl` literal:
`tpl := md.Template{N: 2, Root: md.ScriptWsh, Policy: md.PolicyMulti, K: 2, M: 2, Renderable: true, Keys: …}`

### IMPORTANT
None.

### MINOR

**M1 — gofmt artifacts in implementer-written code (not plan-supplied verbatim).** After transcription, `gofmt -l` flagged `md/md.go` (alignment of the nine `func (xBody) isBody() {}` receiver lines — gofmt column-aligns the trailing `{}`) and `gui/mk1_inspect_test.go` (trailing blank line left by deleting `TestMdmkFlowMD1NoInspect`). Both auto-resolve with `gofmt -w` and don't affect behavior. The plan's Task 6 already runs `gofmt -l` as a gate, so an implementer will catch these; worth a one-line note in Task 5 to gofmt after the `mk1_inspect_test.go` edit.

**M2 (observation, no fix needed) — `useSitePath` struct adds a `hasMultipath bool` the plan's sketch (`useSitePath{multipath []alternative; wildcardHardened bool}`) omits.** Rust models multipath as `Option<Vec<Alternative>>`; a bare `[]alternative` cannot distinguish `None` (bare `*`) from `Some([])`. I added `hasMultipath bool` to preserve the distinction, which `validateMultipathConsistency` and `useSiteString` both need. This is within the plan's "the Go below pins the signatures… port faithfully" latitude (the corpus never exercises bare-`*`, so parity passes either way), but the implementer must make this choice — the plan's struct sketch is slightly under-specified here. Not blocking.

## Verdict

`NOT GREEN — 1C/0I`

One mechanical fix (add `Renderable: true` to one test literal) takes the plan to fully GREEN: all 9 parity Templates match exactly, chunked-refuse works, the port is faithful and implementable as written, build/vet/gofmt/TestAllocs all pass, and the mk1 path is unchanged. Fold C1, re-dispatch.
