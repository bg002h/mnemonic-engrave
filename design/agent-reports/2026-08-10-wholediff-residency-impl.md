# Whole-diff execution review — B2b residency zeroing (F-107 / F-108)

- **Under review:** `/scratch/code/shibboleth/seedhammer-gate-orphan`, branch `b2b-residency`, commit `89235db65c79774d98b1ce5334b08ac036c103da`
- **Baseline:** `b2b` = `3de8aa1`
- **Scope:** implementation only. The design (`DESIGN_b2b_residency_zeroing.md`) was NOT re-reviewed.
- **Reviewer:** independent execution pass; author ≠ reviewer.
- **Worktree left untouched** (`git status --porcelain` empty at exit; HEAD still `89235db`). All mutations were run in a scratch copy.

---

## 1. Verdict

**RED — 0 Critical, 2 Important**

Both Importants are the same class: **a residency guarantee the commit claims, that no test in the tree can fail.** Neither is a behavioural defect in the shipped code — the code does the right thing today — and both are one-line reverts to the immediately preceding commit's text away from silently not doing it, with the whole `./gui/...` suite staying green. Demonstrated by mutation, not argued. If the gate owner grades "correct but unpinned" as Minor, this flips to GREEN; I grade them Important because both mutations restore *exactly* the pre-diff source line, and because the commit's own mutation table (eleven rows, all killed) reads as covering the diff when it does not cover these two claims.

No Critical was found. Specifically, the four wrong-plate / data-race hypotheses in the brief were each chased to a conclusion and each is clean — see §3 and §4.

---

## 2. Findings

### I1 (Important) — `unlockPassphraseFlow`'s `ctx.B.Scrub()` is unpinned; deleting it leaves the entire `./gui/...` suite green

**Where:** `gui/unlock_kdf.go:137-144`

**Failure scenario (demonstrated, not hypothesised).** In a scratch copy I reverted the defer to its pre-diff form:

```go
defer func() { ctx.wipe = prev }()
```

That is the *literal* text at `b2b`. Result:

```
$ go test ./gui/...
ok  	seedhammer.com/gui	49.474s
ok  	seedhammer.com/gui/op	1.642s
...   (all packages ok)
```

Nothing fails. The exposure the deleted line prevents is real and large — driving `unlockPassphraseFlow` to word entry and leaving by Back (the ordinary "give up" route), then reading the backing arrays:

| build | `ctx.B.Residue()` after the flow returned |
| --- | --- |
| HEAD (scrub present) | `args=0 refs=0` |
| scrub deleted | `args=906 refs=0` (caps `args=1216 refs=303`) |

906 non-zero `uint32`s is the §8 twelve-word passphrase's glyph stream — the thing that opens the sealed payload — recoverable verbatim and in index order, on the *common* path. The `unlockSecretSession` half of the same claim **is** pinned (`TestSecretSessionScrubsOnANormalExit`, and the author's mutation row reporting 495 surviving args). The passphrase half has no counterpart. `grep -l unlockPassphraseFlow gui/*_test.go` returns eight files; none reads `ctx.B`.

**Smallest fix.** One test, ~25 lines, mirroring `TestSecretSessionScrubsOnANormalExit`. Verified to discriminate (0 vs 906 above):

```go
func TestPassphraseFlowScrubsOnTheGiveUpRoute(t *testing.T) {
	pf := newPlatform()
	pf.display = sh2DisplaySize
	ctx := NewContext(pf)
	frame, _, quit := runUITouch(ctx, func() { unlockPassphraseFlow(ctx, &descriptorTheme) })
	defer quit()
	for i := 0; i < 400; i++ {
		if _, ok := frame(); !ok {
			break
		}
		click(&ctx.Router, Button1) // Back -> partial entry -> return nil,false
	}
	if args, refs := ctx.B.Residue(); args != 0 || refs != 0 {
		t.Errorf("after the passphrase flow gave up, the frame buffer still holds %d non-zero "+
			"args and %d non-nil refs -- op.Glyph wrote every typed rune into args", args, refs)
	}
}
```

(The loop bound must be a real exit condition, not a fixed count — `frame()` returning `!ok` is the flow having returned *with its defer run*, which is the assertion point.)

---

### I2 (Important) — `releaseResumeState`'s **wiring** is unpinned; unwiring it from `EngraveScreen.Engrave`'s defer leaves `./gui/` green

**Where:** `gui/gui.go:2715-2722`

**Failure scenario (demonstrated).** `TestReleaseResumeStateOnlyClearsAnAbandonedJob` builds `e := &engraveJob{}` directly and calls `e.releaseResumeState()` itself (`gui/residency_resume_test.go:115-122`). It proves the *function's* logic in both directions — which is genuinely the hard half, and it is well built — but nothing in the tree proves the function is ever **called**. Reverting `Engrave`'s defer to its pre-diff one-liner:

```go
defer s.job.Stop()
```

gives:

```
$ go test ./gui/
ok  	seedhammer.com/gui	54.304s
```

Every abandoned engrave job then keeps its `SafePointer.history` — plate geometry, i.e. the seed rendered as coordinates — resident for the Context's lifetime, and the suite says nothing. This is the headline of claim 5, and it is the one part of claim 5 that a reader of the mutation table would assume was covered by the row "drop `releaseResumeState`'s terminal guard" — that row exercises the function, not the call site.

**Smallest fix.** One test, ~30 lines. Verified: it **FAILs** with the defer unwired (`8 resume knots survive…`) and **PASSes** on HEAD.

```go
func TestEngraveReleasesResumeStateOnTheWayOut(t *testing.T) {
	pf := newPlatform()
	pf.display = sh2DisplaySize
	ctx := NewContext(pf)
	job := newEngraverJob(pf, nil, pf.EngraverParams().StepperConfig, 0)
	job.status.State = engraveDone // terminal: the goroutine has provably returned
	p := bezier.Pt(1234, 5678)
	for i := 0; i < 8; i++ {
		job.safePoint.Knot(bspline.Knot{Ctrl: p, T: 5})
	}
	if job.safePoint.HistoryLen() == 0 {
		t.Fatal("INCONCLUSIVE: no resume state to release")
	}
	scr := &EngraveScreen{duration: 1000, job: job}
	frame, _, quit := runUITouch(ctx, func() { scr.Engrave(ctx, &engraveTheme) })
	defer quit()
	for i := 0; i < 20; i++ {
		if _, ok := frame(); !ok {
			break
		}
		click(&ctx.Router, Button1) // Back on a non-running job -> break frames
	}
	if n := job.safePoint.HistoryLen(); n != 0 {
		t.Errorf("%d resume knots survive after EngraveScreen.Engrave returned on an "+
			"abandoned job -- releaseResumeState is not wired into the defer", n)
	}
}
```

`HistoryLen()` was added by this diff for exactly this purpose; it is currently read only by the unit test.

---

### M1 (Minor) — `SafePointer.Knot` is an unfunnelled `append`: the same outgrown-array class claim 1 fixes for `op.Buffer`, on the same data, in the same commit

**Where:** `engrave/engrave.go:1743-1745` (`s.history = append(s.history, k)`), against `ClearHistory` at `:1735-1741`.

`ClearHistory`'s godoc opens: *"zeroes **ALL** of this SafePointer's retained state."* It zeroes `s.history[:cap(s.history)]` — the **current** array. Every array `append` outgrew on the way there is orphaned holding a complete copy of the knots written into it, and nothing can reach it. That is verbatim the defect the diff's own `op.go:31-46` comment describes and that R0 round 0 graded Critical.

Measured on real seed plates (instrumented copy, `gui/engraveSeed` → `PlanEngraving`, counters in `SafePointer.Knot`):

| plate | driver model | knots | `history` reallocs | high-water cap | orphaned knots |
| --- | --- | --- | --- | --- | --- |
| 12-word | lockstep (`Progress(k.T)`) | 21,749 | 4 | 16 | 15 (~300 B) |
| 24-word | lockstep | 33,264 | 4 | 16 | 15 (~300 B) |
| 12-word | reports nothing (`Progress(0)`) | 21,749 | 22 | 27,136 | 92,755 (~1.9 MB) |
| 24-word | reports nothing | 33,264 | 23 | 34,304 | 119,891 (~2.4 MB) |

`stepper.Driver.Knot` (`stepper/stepper.go:88-104`) reports completed ticks as it flushes whole words, so the real machine sits near the lockstep row: **~15 orphaned knots, a fraction of one stroke.** That is why this is Minor rather than Important — but the godoc's "ALL" is false as written, and the *bound* is a property of the driver's flush granularity, not of this code.

**Smallest fix:** either funnel `SafePointer.Knot` the way `op.Buffer` was funnelled (detect `cap` change after the append, `clear` the old array), or soften the godoc to name what it does not reach and record the measured bound. The funnel is ~6 lines and removes the dependence on driver behaviour.

---

### M2 (Minor) — `releaseResumeState`'s safety premise is factually wrong; the conclusion survives, the premise does not

**Where:** `gui/engraver.go:134-136`

> "Safe to call only where a restart is impossible. **Start() has exactly one caller**, inside EngraveScreen.Engrave's own loop…"

Measured — `grep -rn '\.Start()' gui/*.go | grep -v _test.go`:

```
gui/gui.go:2753:					s.job.Start()
gui/engraver.go:179:		e.Start()
gui/qa.go:23:	e.Start()
```

Three. The conclusion still holds, for reasons the comment does not give:

- `gui/engraver.go:179` is `Status()`'s own restart, gated on `e.status.State == engraveRunning`. After `Engrave`'s defer, `Stop()` has already demoted any `engraveRunning` to `engraveStopping`, so that gate cannot be true — and `engrave_duration_test.go:147-149` shows the project already knows this path spawns real goroutines.
- `gui/qa.go:23` operates on a locally constructed job (`qaEngraveFlow`'s `e`), never an `EngraveScreen`'s.

This matters because `releaseResumeState` does **not** reset `e.nknots`: a restart after `ClearHistory` would skip `nknots` knots with an empty catch-up and an origin safe point — the wrong-plate outcome the whole terminal guard exists to prevent. The argument that this cannot happen is load-bearing, and it is currently resting on a claim that is false by `grep`.

**Smallest fix:** replace the sentence with the two-bullet argument above.

---

### M3 (Minor) — stale line citation introduced by this commit's own edit

`gui/engraver.go:121` and `gui/residency_resume_test.go:96` both cite `gui/gui.go:2747` for the operator's hold-to-resume. `gui/gui.go:2747` is `confirm.Start(ctx, confirmDelay)`; `s.job.Start()` is at **2753**. The citation was correct before the diff and was moved by the diff's own six-line insertion at `gui/gui.go:2715`. Two occurrences.

---

### N1 (Nit) — `engrave/residency_test.go:126`

> "Mutation this pins: delete the clear at the trim in `SafePointer.Resume`."

The trim, and the clear, are in `SafePointer.Progress` (`engrave/engrave.go:1697-1702`). `Resume` contains no clear. The test itself calls `sp.Progress(40)` before `sp.Resume(conf)` and is correct; only the naming is wrong.

### N2 (Nit) — funnel lint is whitespace-exact

`lintFile` matches the literal substrings `".args = append("` / `".refs = append("` (`gui/op/funnel_lint_test.go:89`). `b.args=append(b.args, x)` would not match. Unreachable in practice because `gofmt` normalises it, and the test's own header already declares its local-alias blind spot; noted only for completeness of the blind-spot list.

---

## 3. Per-claim verification

| # | Claim | Verdict | How it was established |
| --- | --- | --- | --- |
| 1 | `op.Buffer` funnel: `appendArgs`/`appendRefs` are the only growth routes; realloc detected via `cap(b.args) != cap(old) && cap(old) > 0`; outgrown array cleared immediately | **VERIFIED** | `grep` over every non-test `.go` in `gui/op` shows zero `.args = append` / `.refs = append` outside the two funnel functions; `draw.go` and `image.go` never name `b.args`/`b.refs` at all. The predicate is *exact*, not heuristic: Go's `append` changes `cap` **iff** it reallocates, and a realloc strictly increases it, so `!=` can neither miss nor false-positive; `cap(old) > 0` correctly excludes the nil-array first append. Zeroing the old array is safe because ops carry **indices** (`ops{start,end,refs}`), never slices — verified across `encodeOp`, `ParamImageMask`, `ensureLatest`, `newCompose`, `group.add/Op`. The only slice snapshot of the arrays is `Drawer.draw`'s `args := buf.args` / `refs := buf.refs` (`op.go:360-361`), and nothing appends between that snapshot and its last use: construction always completes before `ctx.Frame` → `FrameCallback` → `draw()` (`run_flow.go:57-79`, `:124`), and inside `draw` every `append` targets Drawer fields (`jumpStack`, `inputs`, `text`, `maskStack`), never the Buffer. Measured, real 24-word `SeedScreen` frame: 22 arrays / 7,919 entries zeroed; residue 0 after `Scrub`. |
| 2 | `ctx.B.Scrub()` added to `unlockSecretSession`'s and `unlockPassphraseFlow`'s defers; `run_flow.go:245` unchanged | **VERIFIED (behaviour) / 1 of 2 halves UNPINNED → I1** | "The defer runs strictly between frames" holds for **both** brackets, and this was checked for the passphrase bracket independently rather than inherited: `Context.Frame` (`gui.go:88-93`) calls `FrameCallback` synchronously, which `yield`s into `run_flow.go`'s range body, which calls `draw(content)` → `d.Draw` before returning — so when either defer runs, no op is pending draw. `unlockPassphraseFlow`'s caller (`unlockSealedFlow`, `unlock_kdf.go:409-413`) holds no `op.Op` across the return; it goes straight into `unlockAttemptOnce`. `d.inputs`/`d.maskStack` hold interface-value **copies**, not aliases into `b.refs`, so event routing after a `Scrub` is unaffected. `run_flow.go:245` confirmed unchanged in the diff. Test coverage: `unlockSecretSession` pinned; `unlockPassphraseFlow` **not** — see I1. |
| 3 | `planEngraving` defer clears `knotBuf[:cap]`, plus `spline[:cap]` if it reallocated | **VERIFIED; second clear confirmed dead (as documented)** | No caller retains a knot slice: `bspline.Measure` (`bspline.go:206`), `ProfileSpline`, `timeConstantPath`, `stepper.Driver.Knot`, `gui/qa.go`, `cmd/glyphtrace/flatten` and `internal/golden` all consume `bspline.Knot` **by value**; `engrave.DryRun` is dead code (no callers). Ranging the same `Curve` twice is safe because each invocation re-derives from `spline := knotBuf[:0]` and writes before it reads, and `ts timeScaler` is per-invocation — pinned by `TestPlanEngravingRematerialisesAfterZeroing`, and exercised in production by `toPlate` (`gui.go:2994-2995` measures, then the job ranges the same Curve) and by the re-present loops at `gui.go:2252`, `:2268`, `bundle_flow.go:351`. Measured: `spline` high-water cap is **exactly 100 == `cap(knotBuf)`** on both a real 12-word and a real 24-word plate, so the `if cap(spline) != cap(knotBuf)` branch never fires; deleting it leaves `./engrave/` and `./gui/` green. The comment already says this and says why it stays — no finding. |
| 4 | `splineResumer.Knot`: `defer clear(c)` on the catch-up array | **VERIFIED — aliasing is impossible by construction** | This was the brief's wrong-plate candidate and it is clean. `SafePointer.Resume` (`engrave.go:1664-1670`) starts from `make([]bspline.Knot, 0, len(s.history)+10)` — a fresh array — then only *appends* to it (`appendLine`, then `append(move, s.history...)`, which copies). No growth pattern can make the result alias `s.history`, because `make` never returns an array anything else holds and `append` only ever moves *away* from it. `clear(c)` clears `[0:len(c)]`; `[len:cap]` is untouched freshly-allocated memory, already zero. The `defer` (rather than a clear after the loop) is correct for the driver-error early return and is pinned by `TestSplineResumerZeroesTheCatchupArrayOnDriverError`; note it fires at **function** exit, not block exit, so `c` is still live for the loop that reads it. |
| 5 | `releaseResumeState` terminal-state-only, wired into `EngraveScreen.Engrave`'s defer after `Stop()` | **VERIFIED race-free / wiring UNPINNED → I2** | The guard is airtight for **every** terminal state, including `engraveStopped` reached via `Stop()`. Every write to `e.status.State` is on the UI goroutine — `Start` (`:108`), `Stop` (`:88`), `Status` (`:166-173`) — and `runEngraving` never touches it. The three terminal states are written *only* inside `Status()`'s `case err := <-e.errs`, and the goroutine body is `errs <- e.runEngraving(quit, progress)`: `runEngraving` is fully evaluated, defers included, **before** the send, so a terminal state establishes happens-before over every `e.safePoint.Knot`/`Progress` write. `Stop()` is not an exception — it writes `engraveStopping`, and only a subsequent `Status()` receive promotes that to `engraveStopped`; the double-Back path returns while still `engraveStopping` and correctly skips. `wipeGuard.armed()`'s `Status()` call (`wipe_guard.go:53`) is **not** a second goroutine: `run_flow`'s event loop is nested inside `ctx.Frame` via `FrameCallback`/`yield` (range-over-func is same-goroutine), and `g.job` is nilled by a defer that runs before any further frame is drawn (`unlock_session.go:222-226`, `:331-335`). `go test -race ./gui/` **passes** (483 s). Ordering `Stop()` before `releaseResumeState` is not load-bearing — both orders skip on `engraveRunning` — but it is the clearer of the two. |
| 6 | `ClearHistory()` zeroes `history` to cap, `safePoint`, `progress`, `completed`; new `HistoryLen()` | **VERIFIED for its callers; see M1 for what it misses** | `progress`, `completed` and `safePoint` are unexported and read only inside `SafePointer` (`Resume`, `Progress`) — no external observer exists. `engraveStatus.Completed` is a **separate** counter fed by the `progress` channel and is untouched by `ClearHistory`, so the countdown is unaffected. `history[:cap]` (not `[:len]`) is right, because `Progress`'s trim compacts and reslices. `HistoryLen()` is read-only and currently test-only. The one thing `ClearHistory` does not do is reach the arrays `append` outgrew (M1), which makes its "zeroes ALL" godoc an overclaim. |

---

## 4. Explicitly checked, no finding

- **Race on `e.safePoint` between the UI and engrave goroutines** — chased through every state transition; the terminal guard is a correct happens-before, and `go test -race ./gui/` is clean. `engrave`'s `-race` failures are pre-existing on untouched `b2b` (per brief; not re-checked).
- **`clear(c)` destroying live resume state via aliasing** — impossible by construction (claim 4). This was the highest-value wrong-plate hypothesis and it does not exist.
- **`planEngraving`'s defer breaking a second range** — pinned by test and confirmed by inspection of every `bspline.Curve` consumer in the tree, including the production re-present loops (`backupSeedStringFlow`, `descriptorFlow`, `bundle_flow`, `derive_xpub`, `unlock_platelist`) that each construct a fresh `EngraveScreen` over the **same** `Plate.Spline`.
- **`Scrub` invalidating a pending draw or the event router** — no. Drawing is synchronous inside `ctx.Frame`; `d.inputs` and `d.maskStack` hold copies, and `Drawer.Draw` clears `maskStack` to cap on entry anyway.
- **Ops holding slices into the Buffer across a reallocation** — no. Ops carry indices; the sole snapshot (`Drawer.draw`) cannot straddle an append.
- **`b.appendArgs(args...)` / `b.appendRefs(refs...)` where the caller's slice aliases the buffer** — the only external `ParamImageMask` call site (`gui.go:411`) passes freshly built literals, and `append` copies before `clear(old)` runs regardless, so even an aliasing caller would be safe.
- **Diff scope** — nothing outside the plan. `gui/gui.go` changes only `Engrave`'s defer (+6 lines); `gui/unlock_kdf.go` and `gui/unlock_session.go` only their existing defers; `gui/op/op.go` is mechanical routing plus the two new funnel functions; no existing test was modified; no unrelated refactor rode along.
- **Variadic-funnel allocation cost on TinyGo** — `vals` does not escape `append(b.args, vals...)`, so it stack-allocates; the reported flash delta (+1,280 B) and unchanged static RAM (60,584) are consistent with that. The new per-realloc `clear` costs 7,919 entries once on a 24-word frame and nothing per frame thereafter, since `Reset` preserves cap.
- **`TestSafePointerTrimZeroesTheTail`'s `t.Skip`** — confirmed **not** skipping: `go test -v` reports `--- PASS`, not `--- SKIP`. The inconclusive guard is live but does not fire, because `Progress(40)` drives two trims and leaves a 3-knot tail past `len` in an 8-cap array.
- **Vacuous assertions across the six new test files** — none found. Every zeroing test holds its own reference and reads the memory back rather than consulting bookkeeping (the `outgrown_test.go` header says so explicitly and it is true of the code); every `INCONCLUSIVE` guard I could reach was checked to be live; `TestSeedFrameReachesTheOutgrownArrayClass` is honest in its godoc about being structurally blind to the zeroing itself. `TestMeasureSeedFrameOrphans` asserts nothing and declares itself a measurement — correct as labelled. The two gaps are gaps in *which* claims have tests (I1, I2), not in the quality of the tests that exist.

---

## 5. Reproduction

```sh
# baseline / diff
git -C /scratch/code/shibboleth/seedhammer-b2b log --oneline -1          # 3de8aa1
git -C /scratch/code/shibboleth/seedhammer-gate-orphan diff b2b..HEAD

# I1: delete ctx.B.Scrub() from gui/unlock_kdf.go's defer (revert to `defer func() { ctx.wipe = prev }()`)
go test ./gui/...            # -> all ok  (mutation SURVIVES)

# I2: delete s.job.releaseResumeState() from gui/gui.go's Engrave defer (revert to `defer s.job.Stop()`)
go test ./gui/               # -> ok      (mutation SURVIVES)

# both proposed tests above kill their mutation and pass on HEAD.

# M1 / claim-3 measurement: counters in SafePointer.Knot + planEngraving, driven by
# gui.engraveSeed on validMnemonic(12) and validMnemonic(24).
```

Toolchain used throughout: `/nix/var/nix/profiles/default/bin/nix develop /scratch/code/shibboleth/seedhammer --command go …`
