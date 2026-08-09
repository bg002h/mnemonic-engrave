# B2b R0 round 1 — fold re-review (opus, verbatim)

**Artifact:** `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md` at `da225c0`
**Object of study:** `git diff 1d8b3a2..da225c0` — the round-0 fold
**Fork:** `seedhammer` at `a01b666`
**Brief:** one question — *did the fold fix each round-0 finding, and did the fold
itself introduce a new defect?* Explicitly **not** a fresh audit. Told not to
re-derive the type-check, the citations, round 0's blessed facts, or F-99.

Persisted verbatim before folding. HTML entities from the transport layer
(`&amp;&amp;`, `&gt;`, `&lt;`) are rendered back to `&&`, `>` and `<`; nothing
else is altered.

**Controller's independent confirmation of the load-bearing claims, before
folding** (each read from the source at `a01b666`, not taken from this report):

| claim | verified |
| --- | --- |
| `saver.State.Draw(screen)` calls `screen.Dirty(dr)` — a real same-goroutine seam | `gui/saver/saver.go:311`, `:315` |
| `kdfStepIterations = 500` | `gui/unlock_kdf.go:26` |
| `newDeriver` is `var newDeriver = seal.NewDeriver`, returning a **concrete** `*seal.Deriver` — so no fake deriver is possible, only a chosen iteration count | `gui/unlock_kdf.go:51` |
| `op.Buffer` is `{args []uint32; refs []any}`, both unexported, **no accessor** — package `gui` cannot observe its size | `gui/op/op.go:28-31` |
| `seal.MaxIterations = 2_000_000` | `seal/wire.go:37` |
| `Context.Frame` runs `c.B.Reset()` **after** the callback | `gui/gui.go:75-80` |

---

## 1. VERDICT

**0C / 5I** (+ 6 Minor, 1 Nit)

The fold's two structural moves — the dedicated `a.warnBuf` and the merged clock — are **correct**. Every finding below is in the fold's *justifications and verification rows*, not in its runtime arithmetic. I walked every `WakeupAt` path, every write to `a.idle.start`, and the reordered `ctx.Reset()`: **the machine cannot sleep past a deadline, spin hot, wipe without a warning, or wipe mid-cut**, and unarmed behaviour is unchanged for Tasks 1–4.

---

## 2. ROUND-0 FINDING DISPOSITION

| finding | disposition |
| --- | --- |
| **A-C1** warning into `ctx.B` | **FIXED BUT INTRODUCED I3** — `a.warnBuf` + `Reset()` before each build bounds it at one frame; `wipeWarningOp`'s inlined `layoutTitlef` is faithful (verified against `gui/gui.go:1865`: same `width-2*16`, same `margin`, body still lands at `titleRect.Max.Y+margin` = y 52). But all three mutation rows meant to *prove* it are unwritable, and the `ctx.B.Reset()` comment is false. |
| **A-I1** `idle.active` latched | **FIXED BUT INTRODUCED I2** — `a.idle.start = now` at the armed edge is the actual fix and is correct. The companion `a.idle.active = false` is redundant; its comment claims otherwise and its mutation row cannot be killed. |
| **A-I2** Task 5 unspecified/self-passing | **FIXED BUT INTRODUCED I1** — three parts, `tickFloor`, three mutants, `&& !armed` correctly normative. But the `Run` term now lives in Task 4's file while Task 5 adds the field. |
| **B-C1** no reachable writer of `wiping` | **FIXED** — `wipeNowHook` is genuinely reachable, nil in production, and matches the package idiom (I confirmed 14 package-level `…Hook` vars in non-test `gui/*.go`). |
| **B-C2** no frame cap | **PARTIALLY FIXED** — `maxRunFrames` does convert Task 5's mutant into a kill (18 000 ticks to `idleTimeout`, then 82 000 saver ticks → `t.Fatalf`; arithmetic checks out). But one of the fold's *own new* mutants hangs **below** the cap (I4), and two named healthy-path tests now fail unconditionally (I5). |
| **B-I1** restart false-PASS | **FIXED** — `"SESSION %d"` / assert `"SESSION 2"` discriminates `break`→`return` and the discard-guard deletion. |
| **B-I2** `&& !armed` row | **FIXED (dissolved)** — the `if armed { … } else` structure is sound: `now ≥ wipeAt ⇒ now ≥ idleWakeup ⇒ idle`, so nesting the wipe inside `if a.idle.active` cannot lose it. "Delete the `if armed`" is killed (saver parks → `t.Fatalf`). |
| **B-I3** countdown untested | **FIXED** — `"erased in 30 seconds"` / `"erased in 15 seconds"` land exactly on synctest's clock. |

---

## 3. NEW CRITICAL

None.

---

## 4. NEW IMPORTANT

### I1 — Task 4's file uses `ctx.keepAwake`, but Task 5 is what adds the field: **Task 4's commit does not compile**

**Location:** plan line 728 (`if len(evts) > 0 || (ctx.keepAwake && !armed)`) vs. line 874 (Task 5 part 1, "`Context` gains `keepAwake bool`, a `KeepAwake()` setter, and `Reset()` clears it").

Task 4's only named fragment is line 808 (`Run`'s one-line delegation). Under the plan's own Global Constraint "one commit per task", step **4.4** (`go test ./gui/`, TinyGo device build, commit) runs against a `run_flow.go` referencing an undefined field → `ctx.keepAwake undefined`. Task 4 cannot go green.

Two false statements ride along:
- Line 598: "this is its state **after Tasks 3 and 4**" — it is the state after Task **5**.
- Line 822, in the section that explicitly tells a reviewer to *confirm* rather than re-derive: "**When `armed` is false the event loop is byte-identical to today**, including the saver covering a running 21-minute cut." False once `keepAwake` is live — `unlockDerive` runs unarmed, so a >3 min derivation no longer trips the saver. That is F-93's intended change, but this line asserts the opposite about the same file.

**Fix:** move Task 5 part 1 (field + `KeepAwake()` + `Reset()` clear) into Task 4's fragment list; correct lines 598 and 822 to say the block is post-Task-5 and that the `keepAwake` term is a deliberate unarmed-saver change.

### I2 — the armed-edge `a.idle.active = false` is redundant; its comment claims a guarantee it does not provide, and its mutation row cannot be killed

**Location:** plan lines 734–741 and mutation row line 849.

The comment says: *"Clearing `active` is not cosmetic — it gates Router.Events below, so leaving it latched makes the plate-done screen look live while silently eating the operator's first tap."*

That describes the **pre-fold** code. Trace the folded code with the line deleted, at cut end (tick T):

1. `a.idle.start = now` (still present, from the edge).
2. `ctx.Reset()`; `if !a.idle.active` → latched true → `Router.Events` skipped on T. `evts` is empty on T — the edge fires because `Status()` flipped, not because of a touch.
3. `idleWakeup = now+3 min` → `idle = false` → `a.idle.active != idle` → **`a.idle.active = false`**.
4. `ctx.WakeupAt(idleWakeup)`, `break` → the flow redraws the plate-done screen.
5. The operator's tap arrives many ticks later: `!a.idle.active` is true → routed against a freshly drawn `d`. **Tap delivered.**

A-I1's symptom does not reproduce. The deleted line's only observable effect is on events arriving in the *same tick* as the edge — which row 849's "post-cut **tap** test" cannot produce (`onDraw` is not called on that tick). **The mutant survives.**

**Fix:** either drop the line and the row, or keep the line and re-target the row to a test that queues the tap from inside the fake job's `Status()` at the flip — and correct the comment to say what it actually buys (routing on the edge tick), not the A-I1 symptom, which `a.idle.start = now` already fixes.

### I3 — all three A-C1 buffer mutation rows name tests that cannot be written, and one is a false PASS regardless

**Location:** mutation rows at plan lines 520, 854, 855; comment at lines 799–801.

- Row 854 (`delete a.warnBuf.Reset()` → "a test asserting the buffer does not grow across warning ticks")
- Row 855 (`&a.warnBuf` → `&ctx.B` → "the same buffer-growth test — this is C1 restored")
- Row 520 (`delete ctx.B.Reset()` → "a test asserting the abandoned buffer's `refs` are zeroed")

`a` is an anonymous struct local to `runWithFlow`'s closure and `ctx` is a local inside the session loop. **Neither is reachable from a test.** The only observation channel is `onDraw(op.Op)`, and `op.Op` is `struct{ op }` with `op.buf` unexported in package `op` (`gui/op/op.go:20-22`) — package `gui` cannot read it. Nor is the growth observable through extracted text: the ops' `{start,end}` ranges stay correct whichever buffer backs them, so `drawn` is byte-identical under mutant 855. This is round-0 B-C1's defect class, reintroduced by A-C1's own verification rows.

Row 520 additionally fails on its own terms. The comment at 799–801 says the tail `ctx.B.Reset()` is *"The ONLY scrubbing the abandoned Context gets … over the last frame drawn, which on the SeedScreen path is the twelve words."* **False.** `Context.Frame` is

```go
func (c *Context) Frame(op op.Op) {
	if f := c.FrameCallback; f != nil { f(op) }
	c.B.Reset()
}
```

(`gui/gui.go:75-80`). The wipe uses `break`, so the range body completes normally, `yield(o)` returns **true**, the callback returns, and `c.B.Reset()` — `clear(b.refs)` at `gui/op/op.go:374-378` — **runs on exactly that last frame**. It runs again after every discard-guarded `Frame` during the unwind. By the time the tail line executes, `refs` is already zeroed; with the harness's own test flow (`for !ctx.Done { build; ctx.Frame(op) }`) nothing appends afterwards, so **deleting the line changes nothing observable**. Round 0's Minor was wrong on this point and the fold promoted it into a comment and a gating row.

**Fix:** add a real seam (a `wipeNowHook`-style package-level `warnBufSizeHook func(args, refs int)`, or hand `runWithFlow` an optional state observer) so the three rows have a host; and correct the 799–801 comment to what the line actually does — scrub any residue built *after* the final `Frame` — or drop it as dead defence.

### I4 — the fold's own new row "hoist `wiping := false`" **hangs**, and `maxRunFrames` structurally cannot catch it

**Location:** mutation row at plan line 519; the cap at lines 251–257 / 282–299.

`ticks++` lives in `runSession`'s range body, which is driven by `yield()` at the **top of the inner event loop** (plan line 708). The discard guard (`if wiping { continue }`, line 698) is the **first** statement of the range body and skips the inner loop entirely.

With `wiping` hoisted above the session loop: session 1 wipes; session 2 starts with `wiping` still true; every `ctx.Frame` from the flow hits `continue`; `yield()` is never called; `ticks` never increments; `draw()` never runs. The test flow's `for !ctx.Done { … }` spins on a fresh `Context` with `Done == false`. Result: a **CPU-bound infinite loop with zero ticks and zero fake-time advance** — a SIGQUIT at `go test`'s 10-minute timeout, which is precisely the outcome B-C2 was filed to eliminate, and Task 7.2 runs these unattended.

**Fix (one line, and it also hardens every other row):** make the plan's test flows self-limiting — `for !ctx.Done { if n++; n > cap { return }; ctx.Frame(op) }`. The flow is test-supplied and is the thing spinning, so bounding it is the only cap that survives the discard guard.

### I5 — step 1.3 and step 4.1's "not armed" bullet cannot pass under the new frame cap: the unarmed saver parks the flow and `runSession` unconditionally `t.Fatalf`s

**Location:** plan lines 331–334 (step 1.3), 837 (step 4.1 "not armed"), 846 (row `armed` hardcoded true); cap at 292–299.

Step 1.3 requires "a test that sleeps past `idleTimeout` … **must observe `Run`'s saver activate**". Once `a.idle.active` is true and unarmed, the saver branch `continue`s forever and the flow never regains control — so the flow can never return, `runSession` runs to `ticks > maxRunFrames`, and `t.Fatalf` fires. There is **no un-park seam**: `onDraw` is only called from `draw()` (plan line 687), and the saver bypasses it entirely (`a.idle.state.Draw(pl)` writes straight to the platform via `Dirty`/`NextChunk`). A tap queued from an earlier content frame is drained by `AppendEvents` on the next call and prevents the saver from activating at all.

Same for step 4.1's "not armed" bullet: to kill row 846 (`armed` hardcoded true) it must run past 3:30 unarmed, which parks it. So the foundational self-check the whole harness rests on, and one of Task 4's five named tests plus its mutation row, all fail unconditionally. (Pre-fold they hung; the cap converted the symptom, not the cause.)

**Fix:** either give `runSession` a `maxFrames int` parameter that *returns* `drawn` instead of `Fatal`ing when a park is expected (then "not armed" asserts `!drawnContains(drawn, "WIPING SECRET DATA")` and 1.3 asserts the flow stopped producing content), or give `deadlinePlatform` a `Dirty` override that queues a tap after N saver frames — `saver.State.Draw` calls `screen.Dirty` (`gui/saver/saver.go:310-312`), so that is a real same-goroutine injection point.

---

## 5. MINOR / NIT

- **Minor.** Row 848's symptom is wrong: deleting `a.idle.start = now` from the armed edge leaves the clock ~21 min stale, so `now ≥ wipeAt` immediately — that is an **instant wipe with no warning**, a §10.2.4 violation, not the "instant warning at cut end" the row states. (The mutant is still killed by the post-cut test.)
- **Minor.** Lines 811–821 were not folded: line 814 names **`a.wipe.origin`**, a field the fold deleted, and lists "two sources" where `a.idle.start` now has three (`len(evts)>0`, the armed edge, `ctx.keepAwake && !armed`). Line 820 says a tap "refreshes `origin`" — same dead field — and attributes the tap's harmlessness to `d`/tags when the actual reason is that `if !a.idle.active` skips `Router.Events` entirely. This is the section that tells reviewers what to *confirm*.
- **Minor.** Line 159 still defines Task 1's file by subtracting "the whole **`a.wipe` block**" — a token that no longer exists — and omits `wipeNowHook` (Task 3) and `ctx.keepAwake` (Task 5), both of which Task 1's pure move must also lack.
- **Minor.** Step 5.1's "the `newDeriver` seam supplying a **fake deriver**, so slices are instant" is not achievable: `newDeriver` is `seal.NewDeriver` returning concrete `*seal.Deriver` (`seal/pbkdf2.go:37,85`) — a test can only choose the iteration count. Each frame costs `kdfStepIterations = 500` real PBKDF2 iterations (`gui/unlock_kdf.go:26`), so ~18 000 ticks = **9 000 000 iterations**, 4.5× `seal.MaxIterations`. The test is feasible only by raising `tickFloor` (e.g. 1 s → 180 ticks → 90 000 iterations) — which the plan does name as a knob, so say so here.
- **Minor.** The armed edge clears `a.idle.active` *before* the `if !a.idle.active` routing gate, so an event landing on that exact tick is routed against a `d` last filled before the saver activated (~21 min earlier, and a different `EngraveScreen` state). Moving the clear below the gate preserves today's dismissal semantics and still fixes A-I1.
- **Minor.** The `secs < 0` clamp (line 566) is unreachable from `Run`: `wipeAt.Sub(now)` is only evaluated after `now.Sub(wipeAt) >= 0` has been ruled out. Row 853 is killable only by a direct unit call of `wipeWarningOp` with a negative `remaining` — worth stating in the row.
- **Nit.** `wipeNowHook` is declared in Task **2**'s whole-file `wipe_guard.go` but is Task **3**'s mechanism, and its doc comment explains Task 3. Harmless (an unused package-level var compiles, so 2.2's "suite otherwise unchanged" holds), but it is filed one task early.
