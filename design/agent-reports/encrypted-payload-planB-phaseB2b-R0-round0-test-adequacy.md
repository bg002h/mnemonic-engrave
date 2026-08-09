# B2b R0 round 0 — test-adequacy lens (sonnet, verbatim)

**Artifact:** `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md` at `7678b5e`
**Fork:** `seedhammer` at `a01b666`
**Brief:** one question — *can the tests this plan specifies actually FAIL, and would
the mutation checks actually be killed?* Design explicitly out of scope (reviewed
separately). Told not to re-derive: that all plan Go type-checks, and that every
citation resolves.

Persisted verbatim before folding. HTML entities from the transport layer
(`&amp;&amp;`, `&gt;`) are rendered back to `&&` and `>`; nothing else is altered.

---

# Test-Adequacy Review: IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md

## VERDICT

**2 Critical / 3 Important**

## CRITICAL

### C1 — Task 3's own specified tests cannot be constructed; the wipe trigger it claims to add lives entirely in Task 4

Plan lines 156–159 define the task boundary precisely by subtraction: *"Task 1's version is that file minus the session loop, the discard guard, and the whole `a.wipe` block."* By elimination, **Task 3's own code = session loop + discard guard only** — it explicitly excludes "the whole `a.wipe` block."

But Task 3's own numbered list (lines 371–416) claims a third change: *"3. The wipe itself, in the event loop — `wiping = true; ctx.Done = true; break`."* That exact statement only appears in Task 4's given code, nested inside `if armed && now.Sub(wipeAt) >= 0 { ... }` (Task 4 block, lines 642–646) — and `armed`, `warnAt`, `wipeAt`, and `a.wipe` are unambiguously part of "the whole `a.wipe` block" Task 3 is defined to lack (confirmed by tracing every other use of `a.wipe`/`armed`/`warnAt`/`wipeAt` in the Task 4 block — all of it is new in Task 4).

Consequence for testability: `wiping` is a plain local variable inside `runWithFlow`'s closure (`wiping := false`, declared fresh each session-loop iteration) with **no external seam**. Nothing outside `run_flow.go` can set it. If Task 3's code truly lacks the `a.wipe` block, there is **no reachable code path in Task 3's own commit that ever sets `wiping = true`** — meaning:
- Step 3.1 ("setting the wipe condition makes the flow return... Run restarts") cannot be written against Task-3-only code.
- All four rows of Task 3.3's mutation table (`break`→`return`, discard-guard deleted, "continue" removed, wiping-never-cleared) target code that, per the plan's own accounting, doesn't exist until Task 4.

This isn't a design critique of the unwind — it's that the plan's own task-by-task/one-commit-per-task discipline (Global Constraints, "one commit per task"; step 3.4's `go test ./gui/ ... commit`) is internally inconsistent with Task 3 also containing "the wipe itself," and as written the tests it promises for that task cannot exist.

**Minimal fix:** either (a) explicitly spell out a minimal, Task-3-only trigger (distinct from Task 4's `armed`/`warnAt`/`wipeAt`) that Task 3's tests exercise and Task 4 later replaces, or (b) merge Tasks 3 and 4 into one task/commit and drop the "Tasks 1–3 change no operator-visible behaviour" framing for Task 3, or (c) state plainly that Task 3's own commit is verified by regression only (`go test ./gui/` unchanged) and defer all restart/discard-guard assertions to Task 4, where the trigger actually exists.

### C2 — The harness has no deadline/iteration cap: the plausible failure mode for several mutants is a hang, not a failure

`runSession` (Task 1b) does `for range runWithFlow(...) {}` with no `context.Context`, no wall-clock safety timeout, and no frame cap. Trace of what happens once `a.idle.active` becomes (or stays) true with no real input queued: the inner loop (Task-4 code, lines 663–669 / pre-B2b code gui.go:3007–3013) draws the saver, calls `ctx.WakeupAt(now.Add(40*time.Millisecond))`, and `continue`s — this **never returns control to the flow**, so the flow stays parked inside its `ctx.Frame` call indefinitely. Under `synctest`, `time.Sleep(40ms)` resolves near-instantly (fake clock), so this becomes a real, unbounded busy loop bounded only by `go test`'s default 10-minute binary timeout.

Concretely, this is not hypothetical:
- **Task 1, step 1.2's own smoke test**, as literally described ("a flow that loops `for !ctx.Done` drawing a label per tick"), has no termination condition at all — nothing in Task 1's version of `run_flow.go` ever sets `ctx.Done`, and the harness's synthetic consumer never stops ranging (empty `for range runWithFlow(...) {}` body in `runSession`, so `yield()` never returns false). This test, if implemented exactly as prose describes, hangs by construction until saver-park, forever.
- **Task 1, step 1.3** ("prove the deadline is honoured") is the one test whose entire purpose is to catch `deadlinePlatform.AppendEvents` NOT actually honoring the deadline — but if that defect is present (no `time.Sleep`, so no synctest clock advance), the resulting behavior is a tight zero-time busy loop, i.e., the foundational self-check hangs instead of failing on exactly the defect it exists to catch.
- **Task 5's stated mutant** ("remove the `KeepAwake` call") causes a long derivation to trip the saver and park (per the plan's own claim) — which, per the trace above, parks the flow permanently with no way to un-idle in a test with no real touches. "does not trip the saver and completes" (5.1) has no fallback: if the mutant fires, the test hangs rather than reporting "saver tripped."

Per the review brief's own framing: *"A test that hangs is worse than one that fails."* None of the plan's tasks specify a bounded-failure mechanism.

**Minimal fix:** give `runSession` (or each `synctest.Test` wrapper) an explicit iteration/frame cap or a real-wall-clock watchdog goroutine that calls `t.Fatal` with the `drawn` slice printed, so a stuck flow reports "exceeded N frames, likely parked" instead of a bare `go test` SIGQUIT dump. This is especially important given Task 7 commits an automated mutation runner (F-96) expected to run many mutants unattended — one hanging mutant currently costs up to the full test timeout instead of milliseconds.

## IMPORTANT

### I1 — The restart test's assertion content is unspecified, and the natural implementation would false-PASS on the `break→return` mutant

Step 3.1 says "assert the restart by extracted content, never by frame count," correctly avoiding the class of bug that already caused a false PASS in this feature — but doesn't say *which content* distinguishes "restarted" from "never restarted." If the test's flow is the obvious thing ("a flow that parks in `ctx.Frame`" drawing a constant label, e.g. "PARKED"), that label is drawn on the very first tick, well before the wipe fires. Under the `break`→`return` mutant (a full GUI exit, no second session, per my trace of Go's range-over-func return propagation matching the plan's own claim), `drawn` still contains "PARKED" from the pre-wipe session — so `assertDrawn(t, drawn, "PARKED")` passes identically whether or not the restart actually happened.

**Fix:** the restart test's flow must draw something that only appears on a *second* entry (a closure-captured entry counter rendered into the frame, e.g. `"SESSION %d"`), and the assertion must check for the second-session marker specifically, not mere presence of any label from the flow.

### I2 — Mutation-table row "saver gate `&& !armed` removed" is attributed to the wrong test and would survive it as named

Task 4.3's table pins this mutant to "the warning test — the saver covers the warning." Tracing the code (Task 4 block, lines 617–621, 640–676): `a.idle.start` and `a.wipe.origin` are refreshed identically (`if len(evts) > 0 { a.idle.start = now; a.wipe.origin = now }`), and diverge *only* via the armed false→true edge reset (line 636–638, `a.wipe.origin = now` without touching `a.idle.start`). In a simple single continuous armed session with no cut (the natural reading of "armed + no input → warning at 3:00"), `idleWakeup == warnAt` exactly, and the `armed && now.Sub(warnAt) >= 0` check (line 647) textually precedes and pre-empts the idle/saver check (line 663) on the very same tick — so `a.idle.active` never becomes true before `warnAt` in that scenario, and the `&& !armed` guard's presence or absence has **zero observable effect** on that specific test. The mutant's actual discriminator is the stale-`a.idle.start`-after-a-cut scenario, i.e. the post-cut test (already named for a different row).

**Fix:** re-attribute this row to the post-cut test, and when writing it, explicitly assert no saver frame appears between cut-end and `warnAt`.

### I3 — No test named for the warning's countdown text or its `secs < 0` clamp

`wipeWarningOp` (Task 4 block) computes `secs := int(remaining.Seconds() + 0.5)` and clamps negative values to 0, then renders "It will be erased in %d seconds." None of Task 4.3's mutation rows exercise this number — a sign flip in the `Sub`, an off-by-one in rounding, or deletion of the `secs < 0` clamp would produce a plausible but wrong countdown (e.g., "-3 seconds" or a frozen/wrong value) with no test catching it, since the named tests check only for presence of the warning, not its numeric content. `assertDrawn` can check for substrings like "erased in 30 seconds" at a specific, synctest-controlled instant — worth adding explicitly.

## MINOR / NIT

- Task 3.3's row "the session loop's `continue` removed" doesn't correspond to a literal `continue` keyword in Task 4's given code (the session loop uses `if !wiping { return }` with implicit fallthrough, not an explicit `continue`); the mutation likely means "hoist `wiping := false` or unconditionally return," but as worded it isn't directly machine-appliable by an AST-based mutator — relevant to Task 7's automated runner (F-96).
- Same imprecision affects "wiping never cleared" (line 432): `wiping` is freshly re-declared each session-loop iteration by construction, so there's no "clear" step to delete; the intended mutation is presumably hoisting the declaration outside the loop.
- `deadlinePlatform.AppendEvents`'s zero-`deadline` edge case (first tick, before any `WakeupAt` call) relies on `time.Until(time.Time{})` being very negative so the `d > 0` sleep is skipped — correct, but undocumented; a reader unfamiliar with the zero-Time behavior could plausibly "fix" this into a bug.

## WHAT I CHECKED AND FOUND ADEQUATE

- Confirmed against the real fork at `a01b666`: `Context`/`NewContext`/`WakeupAt`/`Reset` (gui.go:64–102), `Run`'s pre-B2b body verbatim (gui.go:2934–3020) matches the plan's fact 4 and Task 1's described starting point exactly, `EventRouter.Next`/`Reset` (event.go:266–294), `PointerEvent{Pressed,Entered,Pos}` and its `.Event()` (event.go:90–94, 193).
- Traced Go's range-over-func semantics for this specific code shape (a `return` inside a nested plain `for{}` within `for content := range it {...}` propagates out through the whole `runWithFlow` closure; a `break` targeting only the inner `for{}` does not) — confirms the plan's own claims in fact 1, fact 3, and the `break`-not-`return` rationale are technically correct.
- Verified `uiContains`/`ExtractText`'s "whitespace-insensitive" behavior is real and intentional (glyph-by-glyph rune collection in `gui/op/op.go:311–334` doesn't preserve inter-word spacing, so both sides of the comparison end up space-stripped) by cross-checking existing multi-word usages (`gui/codex32_polish_test.go`) — not a defect, despite initially looking like one.
- Confirmed the "derivation parks under the saver" mechanism Task 5 relies on is real (traced that once `a.idle.active` is true and not armed, the inner loop never returns control to the flow) — the *test* for it is what lacks a timeout (C2), not the underlying claim.
- `assertDrawn`/`drawnContains` correctly fail (not false-PASS) on `drawn == nil` / empty slices.
- `tap()`'s press+release pair correctly makes `len(evts) > 0` true regardless of hit-testing outcome, matching §10.2.4's "any touch refreshes the clock, dismissal-only" behavior.
- Task 2's tests (guard install/uninstall, `armed()` state) are concrete, white-box, and not subject to the false-PASS patterns hunted here.
