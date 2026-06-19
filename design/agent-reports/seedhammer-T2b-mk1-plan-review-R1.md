<!--
Persisted verbatim. opus-architect R1 GATE re-review of IMPLEMENTATION_PLAN_seedhammer_T2b_mk1_decode.md
(folded, commit f58963f). Reviewer agentId a9f32db8fc10e7448. Verdict: GREEN 0C/0I. Re-materialized the
folded plan task-by-task in a detached worktree off 4d02021 (go1.26.4) and ran every checkpoint incl.
the new Step-3a import-expansion sub-steps. All 3 R0 Importants (I1 unused imports, I2 non-comparable
Card, I3 import staging) verified CLOSED — each task compiles at its checkpoint (go build clean after
Tasks 4/5/6), each red step fails only on the intended undefined symbol; M1 (TestMdmkFlowMD1NoInspect)
passes. No drift: all 7 parity xpubs exact, negatives reject, chunk_index=1 guard intact, full go test
./... green, vet/gofmt clean, TestAllocs passes, file manifest exact. Worktree removed; fork left at
4d02021 clean; nothing committed/merged/pushed. Disposition: GREEN — cleared to single-implementer TDD.
The text below is the agent's report verbatim.
-->

# R1 Gate Review — IMPLEMENTATION_PLAN_seedhammer_T2b_mk1_decode.md (folded, `f58963f`)

Re-materialized the folded plan task-by-task in a detached worktree off fork `4d02021` (go1.26.4) and ran every "Run" checkpoint, including the new Step-3a import-expansion sub-steps. Baseline (`go test ./codex32/ ./gui/ ./bip380/`) was clean before applying any task.

## Fold verification

- **I1 — CLOSED.** Task 2's `mk/mk.go` was written with the minimal import block (`errors`, `fmt`, `seedhammer.com/codex32` only). It compiles with no unused-import error and Task 2 Step 4 (`TestParseHeader`/`TestFiveBitToBytes`) PASSES — including the R0-C1 chunk_index-0-based guard (c1 → index=1, total=2). Task 3 Step 3a then expands the block to the full set (`bytes`, `crypto/sha256`, `encoding/hex`, `errors`, `fmt`, `strings`, `btcec`, `hdkeychain`, `codex32`); the appended `Decode` code compiles and all of `go test ./mk/` passes. The R0 defect (6 unused imports in Task 2) is gone.

- **I2 — CLOSED.** Both negative tests now use explicit field-equality checks instead of `!= Card{}`. The red checkpoints prove the comparability error is gone: Task 3 Step 2 failed with *only* `undefined: Decode` (no "struct containing [][4]byte cannot be compared"); Task 6 Step 2 failed with *only* `undefined: mk1GatherFlow`. After implementation, `TestDecodeNegative` (all 9 cases) and `TestMK1GatherFlowBackNoReader` both COMPILE and PASS.

- **I3 — CLOSED.** `gui/mk1_inspect.go` compiles cleanly at every task checkpoint, verified with `go build ./...` (exit 0) after each of Tasks 4, 5, 6:
  - End of Task 4: imports `strings`+`mk` only → build clean, gatherer tests pass.
  - End of Task 5 (after Step 3a adds `fmt`/`image`/`assets`/`layout`/`op`/`widget`): build clean, display tests pass incl. invariant 2.10 (xpub tail `1hM7vFrc` reached via paging).
  - End of Task 6 (after Step 3a adds `errors`/`io`/`log`/`time`): build clean, all gui tests pass. No "imported and not used" at any stage.

- **M1 — PASSES.** `TestMdmkFlowMD1NoInspect` compiles and PASSES: an md1-prefixed literal drives the `isMK==false` branch and the chooser does NOT contain "Inspect key". (`TestMdmkFlowMK1ShowsInspect` confirms the mk1 branch does.)

## Drift check

Every task compiled at its checkpoint with the exact code/import blocks given; each red step failed only on the intended undefined symbol, each green step passed. The `mdmkFlow` `old_string` (Task 6 Step 3c) matched `gui.go` at `4d02021` verbatim — no citation decay. Final state:
- `go build ./...` → exit 0
- `go test ./...` → all `ok`, NO FAILURES across the whole repo
- `go vet ./codex32/ ./mk/ ./gui/` → clean (exit 0)
- `gofmt -l codex32/ mk/ gui/` → empty (clean)
- `TestAllocs` → PASS (alloc gate / invariant 2.6 intact)
- File manifest matches exactly: created `codex32/mkdata.go(+test)`, `mk/mk.go(+test)`, `gui/mk1_inspect.go(+test)`; modified only `gui/gui.go`. No stray changes.

**TestDecodeParity — all 7 PASS with exact xpub match:**
```
V1 m/48'/0'/0'/2'        PASS
V2 m/84'/0'/0'           PASS
V3 m/48'/1'/0'/2' (tpub) PASS
V4 m/84'/0'/0' (no fp)   PASS
V5 m/9999'/1234'/56'/7'  PASS
V6 3-stub m/48'/0'/0'/2' PASS
V7 10-comp no-fp         PASS
```
Negatives (5 corpus + 4 constructed) all reject with zero Card; reassembly order-independent; header chunk_index=1 guard intact.

## Findings

CRITICAL: none. IMPORTANT: none. MINOR: none. The three R0 Importants are folded correctly and the M1 test is present and green; no fold introduced a new compile error, contradiction, or regression.

## Verdict

`GREEN — 0 Critical / 0 Important`

The worktree `/scratch/code/shibboleth/seedhammer-wt-t2b-r1check` was removed (`--force`, exit 0); the fork is left at `4d02021` (main) with a clean `git status`. Nothing was committed, merged, or pushed.
