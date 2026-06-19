<!--
Persisted verbatim. opus-architect R0 GATE review of IMPLEMENTATION_PLAN_seedhammer_T2b_mk1_decode.md
(commit be541f4) BEFORE any code on the real branch. Reviewer agentId a1f09034fde1c8a29. Method:
materialized the plan task-by-task in a detached worktree off fork 4d02021 (go1.26.4), built, and ran
every test. Verdict: NOT GREEN — 0C/3I. ALL 7 parity vectors (V1-V7) decode to the exact embedded
xpub; R0-C1 chunk_index-0-based guard proven load-bearing (injecting +1 fails all multi-chunk
vectors); alloc gate (TestAllocs) intact; vet/gofmt clean; mdmkFlow old_string matched gui.go@4d02021
verbatim (no citation decay). The 3 Importants are mechanical Go compile defects in the plan's
test/import TEXT, not logic: I1 (Task 2 unused imports), I2 (Card has [][4]byte → not comparable, two
`!= Card{}` tests miscompile), I3 (gui import staging under-specified). Worktree removed; fork left at
4d02021 clean; nothing committed/merged/pushed. Disposition: folding I1/I2/I3 + M1 (add md1-no-Inspect
test) → re-dispatch R1; no logic changes. The text below is the agent's report verbatim.
-->

# R0 Gate Review — IMPLEMENTATION_PLAN_seedhammer_T2b_mk1_decode.md (commit be541f4)

## Verification Results (real command output)

I materialized the plan task-by-task in a detached worktree at fork base `4d02021` (go1.26.4) and ran every step.

**Baseline:** `go test ./codex32/ ./gui/ ./bip380/` → all `ok` (clean).

**Per-task red→green:**
- **Task 1** (`codex32.MKDataSymbols`): red = `undefined: MKDataSymbols`; green = `PASS`. Symbols it depends on (`ValidMK`, `splitHRP`, `feFromRune`, `mdmkShortSyms/LongSyms`, `mkRegularMinLen/MaxLen`, `mkLongMinLen/MaxLen`) all exist. ✅
- **Task 2** (`Header`/`ParseHeader`/`fiveBitToBytes`): red = `package mk does not exist`. **As written it does NOT compile** (6 unused imports — see I1). With the minimal import block (`errors`, `fmt`, `codex32`) → `PASS`, including the R0-C1 chunk_index-0-based guard (`TestParseHeader` c1 → index=1, total=2). ✅ after fix
- **Task 3** (`Decode`): red = `Decode undefined`. **The test as written does NOT compile** (`card != (Card{})` on a slice-containing struct — see I2). After fixing that single line: **all 7 parity vectors PASS with exact xpub match**, all 9 negatives reject with zero Card, reassembly order-independent. ✅ after fix
- **Task 4** (`mk1Gatherer`): red→green `PASS` (with minimal `strings`+`mk` imports per the plan's own note). ✅
- **Task 5** (`mk1DisplayFlow`): red→green `PASS`, including invariant 2.10 (xpub tail `1hM7vFrc` reached via paging) and Back-exits. ✅ (import staging caveat — see I3)
- **Task 6** (`mk1GatherFlow` + `mdmkFlow`): red = `mk1GatherFlow undefined`. **The gather test as written does NOT compile** (`card != (mk.Card{})` — same I2 class). After fix: `PASS` for both `TestMK1GatherFlowBackNoReader` and `TestMdmkFlowMK1ShowsInspect`. The `mdmkFlow` `old_string` matched the real `gui.go` at `4d02021` **verbatim** (lines 1928-1948 — no citation decay). ✅ after fix
- **Task 7:** `go build ./...` OK; `go test ./...` all `ok`; `go vet ./codex32/ ./mk/ ./gui/` clean; `gofmt -l` empty; `TestAllocs` **PASS** (invariant 2.6 intact).

**TestDecodeParity — all 7 xpubs match exactly:**
```
V1 m/48'/0'/0'/2'  → xpub6Den8YwXbKQvk…1hM7vFrc   PASS
V2 m/84'/0'/0'     → xpub6BmeGmRo4LosA…oCp2z6a    PASS
V3 m/48'/1'/0'/2'  → tpubDE2Qenmnf…CSz2dhS (testnet) PASS
V4 m/84'/0'/0' (no fp) → xpub6BmeGmSNQ…WBfWeHx    PASS
V5 m/9999'/1234'/56'/7' (0xFE) → xpub6Den8YxgJ…bLR3ZvB  PASS
V6 3-stub m/48'/0'/0'/2' → xpub6Den8Yxxy…RetAhFa   PASS
V7 10-comp no-fp → xpub6QwbHG5Nw…KQenNqk          PASS
```
No mismatches. **Independent corpus cross-check:** corpus SHA-256 = `ebd8f34d…d78ff` (matches pin), `family_token "mk-codec 0.2"`, `schema 2`. V1 `vectors[0].input` confirms `network=mainnet`, `origin_path=m/48'/0'/0'/2'`, `origin_fingerprint=aabbccdd`, `policy_id_stubs=["11223344"]`, exact xpub; `expected.strings` byte-identical to the plan's V1; `decoder_correction: "clean"`; canonical bytecode `040111223344aabbccdd05…0488b21e…` corroborates the wire layout the decoder parses.

**Load-bearing checks:** Injecting the `chunk_index + 1` bug made `TestParseHeader` + all multi-chunk parity vectors fail with "chunked header malformed" — the R0-C1 guard is real. `hdkeychain.NewExtendedKey(version, key, chainCode, parentFP []byte, depth uint8, childNum uint32, isPrivate bool)` matches the plan's positional call exactly; the fork's own `bip380.go:104` uses the identical arg order and the **raw** last-path u32 for `childNum` (confirms R0-M1).

**Cleanup:** worktree removed; fork left at `4d02021` (main), `git status` clean. Nothing committed/merged/pushed.

## Findings

### IMPORTANT

- **I1 — Task 2 `mk/mk.go` does not compile as written (6 unused imports).** Step 3's literal import block lists `bytes`, `crypto/sha256`, `encoding/hex`, `strings`, `btcec`, `hdkeychain`, but Task 2's code uses only `errors`, `fmt`, `codex32`. Go errors on every unused import, so Task 2's "Step 4: expect PASS" cannot pass. Unlike Task 4, Task 2 has **no** deferred-imports note. **Fix:** Task 2's import block must be `import ( "errors"; "fmt"; "seedhammer.com/codex32" )`, with Task 3 re-adding the full set when it appends the code that uses them. (Verified: with minimal imports Task 2 passes; with the full block it fails to build.)

- **I2 — `Card`/`mk.Card` is not comparable; two negative tests do not compile.** `Card` has a `Stubs [][4]byte` field, so `card != (Card{})` (Task 3 `TestDecodeNegative`, line ~498) and `card != (mk.Card{})` (Task 6 `TestMK1GatherFlowBackNoReader`, line ~1112) are hard compile errors (`struct containing [][4]byte cannot be compared`). This blocks the **entire** `mk` and `gui` test binaries — so even `TestDecodeParity` won't run until fixed. **Fix:** replace each with explicit field checks, e.g. `if card.Network != "" || card.Path != "" || card.Fingerprint != "" || card.Xpub != "" || len(card.Stubs) != 0 {`. (Verified: after this change both tests compile and pass.)

- **I3 — Import staging for `gui/mk1_inspect.go` is under-specified and the stated guidance miscompiles.** Task 4's note says "add the rest in Task 5," but `errors`/`io`/`log`/`time` are not used until Task 6's `mk1GatherFlow`; adding them in Task 5 yields `imported and not used` (verified). The correct, compiling staging is incremental per the code each task adds: Task 4 = `strings`+`mk`; Task 5 = +`fmt`,`image`,`assets`,`layout`,`op`,`widget`; Task 6 = +`errors`,`io`,`log`,`time`. **Fix:** state the exact import set for each of Tasks 4/5/6 explicitly, or fold Tasks 4-6 into a single GUI commit so intermediate-compile is moot. The final state compiles, builds, vets, and gofmts cleanly; this only affects the per-step TDD red/green checkpoints.

### MINOR

- **M1 — No execution test for the md1-no-Inspect path (§2.5/§2.9).** The plan adds `TestMdmkFlowMK1ShowsInspect` but no symmetric md1 frame test asserting "Inspect key" is absent and choice 0 engraves directly. The md1 branch (`isMK==false`) is byte-identical to the original (`title="Engrave"`, `choices=labels`, `idx=choice`), so a regression is structurally precluded, and existing `validateMdmk` md1 tests pass — but a one-line `mdmkFlow(ctx, …, mdmkText("md1…"))` + `!uiContains(content, "Inspect key")` assertion would make the invariant execution-review-checkable rather than argued. Non-blocking.

- **M2 — Task 3's red step is partly vestigial.** Step 2 ("expect FAIL: `Decode` undefined") is moot because the appended test won't compile at all (I2) until fixed; the implementer should fix I2 first, then observe the genuine red (`Decode` undefined) → green. Cosmetic ordering note.

## Verdict

**NOT GREEN — 0C / 3I** (I1, I2, I3)

All three are mechanical, low-risk transcription defects in the plan's test/import text — the underlying decode logic, GUI flows, wiring, and spec-coverage are **fully correct and parity-verified** (7/7 xpubs exact, R0-C1 guard load-bearing, alloc gate intact, vet/gofmt clean). Fold I1/I2/I3 (each fix is a verified one- or few-line change), re-persist this review, and re-dispatch; the plan should reach GREEN with no logic changes.
