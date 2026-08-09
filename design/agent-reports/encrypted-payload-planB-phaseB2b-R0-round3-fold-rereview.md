# B2b R0 round 3 — fold re-review (opus, verbatim) — **GREEN**

**Artifact:** `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md` at `24e8376`
**Object of study:** `git diff 64108b5..24e8376` — the round-2 fold (two commits)
**Brief:** one question — *did the round-2 fold fix each finding, and did the fold
itself introduce a new defect?* Told that a clean result CLOSES the gate and
implementation begins, and not to manufacture findings.

**This review returned 0C/0I, so per the standing rule the R0 loop is CLOSED.**
No round 4. Its five Minors and four Nits were folded inline (`git show` the
next commit) rather than deferred, because three of them were cheap and one —
Minor 2 — told an implementer something untrue about code they were about to
write.

Notably, this reviewer **executed** the new panic path in a faithful minimal
reproduction rather than reasoning about it, and additionally type-checked the
whole test package under go1.26. That is what closing a gate on a control-flow
change should look like.

Persisted verbatim before folding.

**Controller's independent confirmation of the two machine-checkable Minors,
before folding:**

| claim | verified |
| --- | --- |
| **Minor 2** the "no `panic`/`recover`" constraint is false | `grep -rn "panic(" --include=*.go` excluding tests → **129** sites; the plan's own blocks contain **3** (`panic(err)` in `draw`, verbatim from the existing `Run` body, plus the new sentinel pair). Real `recover()` calls in non-test Go → **0** (the single grep hit, `backup/freetext.go:56`, is inside a comment). Round 2's wording scoped it to `recover`, which was correct. |
| **Minor 1** the gate-coverage line counts are stale | `plan-build-gate-go.sh` TIER-1 banner: `run_flow.go` **224** (plan said 223), `run_harness_test.go` **231** (plan said 210). |

---

I've read the round-2 report, the two-commit fold, and the current plan, and machine-checked the fold's own output rather than trusting the transcript. Verification I ran:

- `plan-mutation-anchors.py` on the plan → **15 unique, 0 BAD, 2 unresolved**, exit 0 (matches the plan's claim); instrumented it to confirm all **17** real mutation rows are graded (only the 4 header rows are skipped) and that exactly the 6 whole-file blocks are classified `whole`.
- `plan-cite-gate.sh` → every citation resolves. `.github/workflows/test.yml:29` is in fact the `-target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks` line. `gui/op/op.go:376` is `clear(b.refs)`.
- `plan-build-gate-go.sh` → TIER 1 fails only on the two documented `ctx.wipe` / `ctx.keepAwake` blind spots.
- Modelled the **shipped** config by hand (6 whole files + `Context` fields + `KeepAwake` + `Reset` clear + `Run` body replaced + `saver` import dropped), then applied step 6.0 too: `go build` OK; `go vet` under go1.26 with the `go` directive bumped → **exit 0, whole test package including `run_harness_test.go` type-checks clean**; `gofmt -l` empty.
- Built a faithful minimal reproduction of `runSession` → `runWithFlow` → `it` → `flow` to execute the new panic path.

## 1. VERDICT

**0C / 0I** (5 Minor, 4 Nit)

## 2. FINDING DISPOSITION

| finding | disposition |
| --- | --- |
| **N-I1** TinyGo row does not build | **FIXED** — row is now a citation of `test.yml:29`; verified that line carries all four dropped flags. |
| **N-I2** `boundedFlow` bounds the session, not the run | **FIXED** — I executed the mechanism. The panic propagates out of `flow` → `it` → `for content := range it` → `runWithFlow` → `runSession`'s recover with no `panicrangestate` interference (the panic is raised in *iterator* position, never inside a live `yield`, so the rangefunc state is `RF_READY`). Non-sentinel values re-panic correctly; `runtime.Goexit` yields `recover() == nil` and passes through. The mutant now dies as one `t.Errorf` instead of spinning. Step 3.2's new clause closes the "referenced by no step" half. |
| **N-I3** checker false-PASS + duplicate anchor + wrong file cell | **FIXED** — v2 resolves per-file, requires exactly one code span, and returns UNRESOLVED (not BAD) for fragments; reproduced 15/0/2. Row `:1061` and row `:1062` both now grade against the right file. |
| **Minors (group)** | **MOSTLY FIXED, 3 residual.** Fixed: `mustFinish`/`runSession` requirement + explicit `parked` assertion, session-head comment overclaim, step 6.0, Task 7's 5.2–5.3 claim, five/six, the stale singular vet sentence, `op.go:374`→`:376`, the `parked` naming nit. **Not fixed:** round-2 Minor 1 (`Dirty` fires twice per saver frame; steps 1.3/4.1 still never say how a test discriminates saver activation from a content draw); the dropped `a.idle.active = false` session-head row (line still pinned by nothing); the missing blank line before `### The release tag's precondition set`. |

## 3. NEW CRITICAL / NEW IMPORTANT

None.

Specifically closing the four risk areas the brief named: the panic/recover pair is sound (traced by execution, not assertion) — after a recovered `flowBound`, `runSession` returns `(drawn-so-far, parked=false)`, so `mustFinish` returns normally rather than `t.Fatalf`-ing, but `t.Errorf` has already failed the test, so there is **no false-PASS path**; the carve-out stays in `run_harness_test.go`; step 6.0's placement after `defer clear(m)` at `:250` is correct and the fragment compiles; Task 3 remains implementable without the removed snippet because Task 4's gated whole file carries the guard verbatim with its comment, and the stated reason for removing it (it made `if wiping {` match twice) is true — the old fence bound to `gui/run_flow.go` via the "all in `gui/run_flow.go`" anchor.

## 4. MINOR / NIT

**Minor**

1. `:1297-1298` — the gate-coverage line counts are **pre-fold**: `run_flow.go` is **224** lines (says 223) and `run_harness_test.go` is **231** (says 210). Measured with the plan's own extractor and confirmed by `plan-build-gate-go.sh`'s TIER-1 banner. The transcript's *conclusions* are true (I reproduced them), only the counts are stale — but they are the numbers a future reader would use to tell whether the gate ran against the current blocks.
2. `:145-151` — the rewritten constraint "**No `panic`/`recover` in NON-TEST code.** The fork has none and this plan adds none" is **false**: 129 `panic(` sites in non-test fork Go, and this plan's own `run_flow.go` block reproduces one verbatim (`panic(err)` in the `draw` closure). Round 2's wording scoped it to `recover` (0 sites), which was correct; restore that scope or the implementer is told the gated block violates the plan.
3. `:1172-1183` — step 6.0's snippet sits in a bare ``` fence, so **neither gate touches it** (`plan-build-gate-go.sh` saw 2 TIER-2 fragments, not 3), and the "what remains a reviewer's execution pass" list at `:1336-1339` was not extended to name it. I applied it by hand: build OK, vet clean, gofmt empty — so it is correct, just uncovered and unlisted.
4. `:1230-1233` — "Task 5's mutants live in 4.3's table" holds for **5.3 only**. 5.1's mutant (delete the `ctx.keepAwake` term) has no row and no stated exemption, so Task 7's "a list and not a judgement call" silently drops it. Fix: add `(ctx.keepAwake && !armed)` → `(false)`, killed by 5.1 — or exempt it in prose the way 5.2 is.
5. `:654` — the `if !wiping {` row is now a **different mutant**: `if false {` never exits the session loop (previously `return` unconditionally, which always exited). It is still killed, but by `maxRunFrames` → `mustFinish`'s `t.Fatalf` after ~100 000 ticks of real rendering, not by "SESSION 2 never drawn"; the "killed by" cell no longer says how. The restart property itself is still covered by the `break`→`return` row.

**Nit**

- `:1051` still says `break` occurs "5× in this plan's own blocks"; measured **6** lines across all blocks (5 in `run_flow.go`, of which one is a comment; 3 bare). `:1290` was corrected, `:1051` was not.
- `plan-mutation-anchors.py`'s `rows()` silently drops any 4-cell row whose first cell has no code span — a blind spot not in its stated NOT-COVERED list. Only the 4 header rows hit it today.
- Step 3.2's "every flow in Tasks 3–5's tests goes through `boundedFlow`" is a *test-authoring* rule filed under "Write the changes"; step 3.1 ("Tests first") is where the flows get written.
- The two UNRESOLVED rows are described at `:1281` as "cannot resolve until Task 6 adds the call site"; that is true of `unlockMnemonicParsedHook(m)` but not of `defer clear(m)`, which already exists at `unlock_session.go:250`.

## 5. READY TO IMPLEMENT?

Yes. Nothing here blocks an implementer or would lead them to write the wrong code: the six whole-file blocks are the single source, they compile and vet clean in the configuration that actually ships (including step 6.0's fragment, which I applied), every anchor is unique in the file its row names, and the one new piece of control flow — the `flowBound` panic/recover — behaves exactly as its comment claims when executed. The two places an implementer will have to supply something the plan does not: **step 1.3's saver assertion**, where `p.dirties` alone cannot distinguish saver activation from a content draw and the plan still names no discriminator (round-2 Minor 1, unfolded — round 2's suggestion was to correlate a `Dirty` with no following `onDraw`); and the 3-line delete for the `if wiping {` row, which is a natural-language instruction the Task 7 runner must special-case alongside the several "*delete the line*" rows. Neither is a gate. Minor 2 is the one I would fix before handing it over, since it is the only item that tells the implementer something untrue about code they are about to write.
