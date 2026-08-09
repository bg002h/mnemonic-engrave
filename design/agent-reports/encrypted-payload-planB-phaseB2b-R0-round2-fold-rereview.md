# B2b R0 round 2 — fold re-review (opus, verbatim)

**Artifact:** `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md` at `3b67389`
**Object of study:** `git diff f54c91b..3b67389 -- design/ scripts/` — the round-1 fold
**Brief:** one question — *did the round-1 fold fix each finding, and did the fold
itself introduce a new defect?* Explicitly not a fresh audit; told a clean result
CLOSES the gate and not to manufacture findings.

Persisted verbatim before folding.

**Controller's independent confirmation of both machine-checkable findings,
before folding:**

| claim | verified |
| --- | --- |
| **N-I1** the TinyGo row the fold wrote does not build | ran it: `cmd/controller/platform_sh2.go:128:19: undefined: machine.GPIO30`. `pico2-w` is RP2350**A**; the SH2 is RP2350**B**. `.github/workflows/test.yml:29` reproduces `1310184 / 60584` exactly. |
| **N-I3** an anchor matches twice and the new checker reports it `ok` | measured: 6-tab `if armed {` occurs **2×** in `run_flow.go` (block lines 132, 178). The script graded `max(spans, key=len)`, which was the parenthetical *context*, not the anchor. Also confirmed `const wipeWarningDelay` occurs **0×** in `run_flow.go`, the file that row named. |

---

## 1. VERDICT

**0C / 3I** (+ 8 Minor, 3 Nit)

The fold's two riskiest structural moves are **correct** and I close them: the deleted tail `ctx.B.Reset()` is genuinely dead on the wipe path, and the removed armed-edge `a.idle.active = false` self-clears with no latch. All three Importants are in the fold's *new tooling and new verification rows* — the same failure mode as round 1.

## 2. FINDING DISPOSITION

| finding | disposition |
| --- | --- |
| fold-review **I1** (Task 4 reads `ctx.keepAwake`) | **FIXED** — `Context` field/`KeepAwake()`/`Reset()` clear moved to Task 4's fragment list; Task 5 part 1 rewritten; line 598's "after Tasks 3 and 4" → "3, 4 and 5"; the "byte-identical to today" claim now names its one deliberate exception. |
| fold-review **I2** (redundant armed-edge clear) | **FIXED** — line deleted, comment rewritten and now accurate (verified: `a.idle.start = now` → `idle` recomputes false → `a.idle.active` clears on the same tick; no path leaves it latched, incl. `keepAwake` and session restart). |
| fold-review **I3** (buffer rows unwritable; false `ctx.B.Reset()` comment) | **FIXED** — `gui/op/buffer_len.go` + `warnBufHook` make both rows killable (traced); tail `ctx.B.Reset()` deleted and the replacement comment is **true** (verified below). |
| fold-review **I4** (hoist `wiping := false` hangs below the cap) | **NOT FIXED** — `boundedFlow` bounds each *session*, not the *session loop*. See NEW IMPORTANT 2. |
| fold-review **I5** (1.3 / 4.1 "not armed" fail unconditionally) | **FIXED** — `runSession` → `(drawn, parked)`, `mustFinish`, `deadlinePlatform.Dirty`. The `t.Fatalf` correctly moved out of the live iterator. Steps 1.3/4.1 were not updated to name the entry point or the discriminator (Minor). |
| sweep **I1** (F-87 remedy unimplementable) | **FIXED** — `unlockMnemonicParsedHook` + mandatory fired-guard + a row that kills the guard. Verified it observes the same backing array on all three early returns. |
| sweep **I2** (386 row red at baseline) | **FIXED** — `CGO_ENABLED=0` restored, ~52 s noted. **But the row two lines above it was newly broken:** NEW IMPORTANT 1. |
| sweep **I3** (9/20 rows not mechanically appliable) | **PARTIALLY FIXED, INTRODUCED a duplicate anchor + a false-PASS gate** — see NEW IMPORTANT 3. §11.3's two procedural rules are now in Task 7 ✓. |
| sweep **I4** (three follow-ups parked on "own cycle") | **FIXED** — B2c named in the plan's coverage table and in all three FOLLOWUPS headings; register and plan agree (checked). |
| sweep **M1** (orphaned `saver` import) | **FIXED** — step 1.1 deletes it; the TIER-1-is-additive blind spot is named and the controller's transcript models the shipped config. |
| sweep **M3** ("Task 9", tag preconditions ×3) | **FIXED** — one checklist, "B2a-ii's Task 9" named, F-92/F-98/F-100/`ci/staging` folded in. |

## 3. NEW CRITICAL

None.

## 4. NEW IMPORTANT

### N-I1 — the green criterion's TinyGo row, which the fold rewrote to "the exact command", **does not build**

**Location:** plan `:117`.

The fold replaced the bare row with
`nix develop --command tinygo build -o /dev/null -target=pico2-w -size=short ./cmd/controller`.
I ran it at `a01b666`:

```
# seedhammer.com/cmd/controller
cmd/controller/platform_sh2.go:128:19: undefined: machine.GPIO30
exit 1
```

`pico2-w` is RP2350**A**; the SeedHammer II is RP2350**B** (`pico-plus2`). Even on the right target the numbers could not match: the row's stated baseline was produced with `-opt 2 -gc precise -scheduler tasks -stack-size 16kb`, all dropped (sweep M8 measured `-opt z` alone at 893676/69340 vs 1310184/60584).

The correct command is `.github/workflows/test.yml:29`, and both predecessor plans carry it verbatim (`…phaseB2a_i.md:136`, `…phaseB2a_ii.md:97`). I ran it:

```
   code  rodata    data     bss |   flash     ram | package
1034372  245368   30444   30140 | 1310184   60584 | total
```

— exactly the plan's baseline. This is sweep I2's defect class reintroduced by the fold that was fixing it, and worse: I2's row failed on a missing 32-bit cgo toolchain, this one fails to compile the firmware. Tasks **3.4 and 4.4 both require this device build**, so it blocks two task gates, not just the final green — and it is the phase's only RAM-budget signal, in the phase whose most expensive finding was a 228 KB buffer growth.

**Fix:** paste `.github/workflows/test.yml:29` verbatim, or cite that line. Do not restate it by hand.

### N-I2 — `boundedFlow` does not terminate the run for the one mutant it exists to catch; I4's hang is unfixed

**Location:** plan `:355-379` (`boundedFlow`), row `:605`.

Trace the mutant (`wiping := false` hoisted above `for {`):

1. Session 1 wipes; `wiping` stays true.
2. Session 2: fresh `ctx` (`Done == false`); `boundedFlow`'s closure starts at `n = 0`; every `ctx.Frame` → `yield(o)` → range body → `if wiping { continue }` → `yield` returns **true**, so `ctx.Done` never goes true and `runSession`'s `ticks` never increments (the outer `yield()` is below the guard).
3. At `n > maxRunFrames`: `t.Errorf` (does **not** Goexit), then `return`. The flow returns, `it` returns, the range ends.
4. `if !wiping { return }` — `wiping` is still true → **session 3**, and `boundedFlow` returns a fresh `n = 0`. Sessions repeat forever.

So the mutant still runs to `go test`'s 10-minute SIGQUIT, now emitting an unbounded stream of `Errorf`s. The fold's own comment states the requirement it misses — *"A HANG IS WORSE THAN A FAILURE, and Task 7 runs many mutants unattended."* Compounding it, `boundedFlow` is referenced by **no step** (only by row `:605`), so nothing requires a test to use it at all.

**Fix:** make the bound terminal rather than per-session. `t.Fatal` is barred (Goexit through a live iterator), so panic with a sentinel and recover it in `runSession`, which unwinds `runWithFlow` entirely and cannot re-enter the session loop:

```go
type flowBound struct{}
// in boundedFlow: if n > maxRunFrames { panic(flowBound{}) }
// in runSession:  defer func() { if r := recover(); r != nil {
//                     if _, ok := r.(flowBound); !ok { panic(r) }
//                     t.Errorf("test flow exceeded %d iterations without ctx.Done", maxRunFrames)
//                 } }()
```

Add a step requiring every Task 3 flow to be wrapped in `boundedFlow`.

### N-I3 — a mutation anchor matches **twice**, and the fold's new checker reports it `ok`: the gate has a false-PASS

**Location:** plan `:1010` (Task 4.3), `:1011`, `:1208-1218` (gate coverage); `scripts/plan-mutation-anchors.py:84`.

Row `:1010` is `| run_flow.go | ` + 6 tabs + `if armed {` (inside `if a.idle.active {`) `| if false { | the warning test …`. Measured in the plan's `run_flow.go` block:

```
'\t\t\t\t\t\tif armed {': 2   (block lines 132 and 178)
```

Line 132 is the **armed edge**; line 178 is the **warning branch** the row targets. The script passes it because `main()` takes `max(spans, key=len)` — the longest backticked span in the anchor cell — which here is the parenthetical *context* `if a.idle.active {` (1 match), not the anchor. A runner keyed on the first match mutates the armed edge instead, which the post-cut test already kills — reporting a kill while **never exercising the §10.2.4 warning branch**. That is precisely "a silently-failing `sed` reads exactly like a surviving mutation", relocated into the tool written to prevent it.

Same heuristic hides a second defect: row `:1011`'s file cell says `run_flow.go`, but `const wipeWarningDelay = 30 * time.Second` matches **0×** there and 1× in `wipe_warning.go` (which the row's own parenthetical admits). The script searches all fences concatenated, so it never compares the anchor against the file it names.

Consequently the gate-coverage claim at `:1208` — *"proves **every** mutation-table anchor matches exactly once — 16 unique, **0 duplicate**"* — is **false**.

**Fix:** (a) in the script, resolve each row against the fence for the file its first cell names, and check **every** span in the anchor cell rather than the longest (or take the first, and put context in the "killed by" column, never in the anchor cell); (b) disambiguate row `:1010` in the code the way `a.idle.start = now` was — a trailing comment on the warning branch's `if armed {`; (c) correct row `:1011`'s file cell to `wipe_warning.go`.

## 5. MINOR / NIT

- **Minor.** `Dirty` fires **twice per saver frame** (`saver.State.Draw` calls `newDraw` at `saver.go:328` and `:353`) and once per content frame via `draw()`. A raw `dirties` count is therefore neither a frame count nor saver-specific; steps 1.3 and 4.1 never say how a test discriminates (correlating a `Dirty` with no following `onDraw` is the available method).
- **Minor.** No step says which tests use `mustFinish` and which use `runSession` — it appears only in a code comment. Step 1.2's "a flow that returns immediately terminates `runSession`" now requires an explicit `parked == false` assertion or it passes vacuously.
- **Minor.** The fold dropped **both** `a.idle.active = false` mutation rows; the session-head line survives in the code, is now pinned by nothing, and its comment still claims a session inheriting it "would silently eat its first tap" — by the fold's own trace it eats at most that one tick's events.
- **Minor.** Task 7 claims to own "every anchored row in Tasks 2.3, 3.3, 4.3, **5.2–5.3** and 6.2", but 5.2 and 5.3 carry no rows; 5.2 is still the prose reordering the sweep asked to drop or re-express (5.3's mutant does have a row, in 4.3).
- **Minor.** The gate-coverage transcript says "the plan's **5** whole files" and "`gofmt -l <all five>`" three lines below "type-checks **six** whole-file blocks". Six is right (verified: 223/210/61/52/13/9 all match the fences).
- **Minor.** Stale singular left below the new text: "*That vet line* is the pre-existing baseline … and **the only finding**" now directly contradicts the sentence above it ("Both vet lines are the recorded baseline").
- **Minor.** `break` is stated as **5×** "in this plan's own blocks" (Task 4.3 preamble) and **6×** (gate coverage). Measured: 6 across all fences, 5 in `run_flow.go`.
- **Minor.** Task 6 has no step that adds the `unlockMnemonicParsedHook(m)` call site to `unlock_session.go:251`; it exists only in the seam file's doc comment and as a mutation-row anchor.
- **Nit.** `### The release tag's precondition set` has no blank line before it, immediately following a list item.
- **Nit.** `gui/op/op.go:374` is cited for `clear(b.refs)`; `:374` is `func (b *Buffer) Reset() {`, the `clear` is at `:376`.
- **Nit.** `runSession`'s `parked` means "hit the frame cap", not "the flow parked" — the discard-guard park (N-I2) never sets it. Worth saying, since the name invites the wrong reading.

---

**Verified clean, so this can close rather than loop** — the two items the brief flagged as the fold's highest risk:

- **Deleted tail `ctx.B.Reset()`.** Traced end to end on the wipe path against `a01b666`: at the wipe, `wiping = true; ctx.Done = true; break` exits only the inner loop, so the range body completes, `yield(o)` returns **true**, `FrameCallback` returns, and `Context.Frame`'s `c.B.Reset()` → `clear(b.refs)` runs **on the parked frame** — the twelve words. Every discard-guarded `Frame` during the unwind resets again (`continue` in a range-over-func body makes `yield` return true). Both fact-3 screens (`gui.go:2460`, `:2758`) end their loop bodies with `ctx.Frame` and build nothing after it; no `defer` in the flow appends ops. No unzeroed seed-derived refs reach the abandoned buffer. The replacement comment is accurate.
- **Removed armed-edge `a.idle.active = false`.** `a.idle.active` is recomputed from `a.idle.start` on every inner-loop pass; the armed edge's `a.idle.start = now` forces `idle == false` on that same pass, so the very next `if a.idle.active != idle` clears it. `keepAwake` refreshes the same clock (read before `ctx.Reset()`); the session head re-initialises `start`/`active`/`armed`. No latching path.
- Also verified: both buffer rows are genuinely killable through `warnBufHook`/`Buffer.Len()`; `unlockMnemonicParsedHook` at `:251` hands a test the same backing array `defer clear(m)` zeroes on all three early returns; the B2c re-assignment is self-consistent across plan and register.
