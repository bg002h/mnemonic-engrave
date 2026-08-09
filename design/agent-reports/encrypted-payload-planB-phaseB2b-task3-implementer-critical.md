# B2b Task 3 — implementer STOP: a Critical in the GREEN plan (verbatim)

**Found by:** the Tasks 2–3 implementer (sonnet), 2026-08-09, while executing a
plan that had passed **four** independent R0 rounds at 0C/0I.
**Status:** Task 2 committed and green (`aa704b6`). **Task 3 stopped, not
committed.** The implementer was instructed that a GREEN plan's defects come back
to the controller rather than being worked around, and it did exactly that.

**This invalidates the plan's GREEN status.** The defect is in the normative
unwind mechanism — the thing the whole phase exists to build — and it is not
confined to Task 3: Task 4's warning-expiry wipe sets `ctx.Done` the same way.

## Controller's independent verification, before folding

I did not take this on trust. Minimal repro, run:

```go
c := &ctx{}
yield := func() bool {
    c.Done = true   // the wipe block: wiping = true; ctx.Done = true; break
    return true     // `break` lets the range body complete, so yield reports "keep going"
}
c.Done = c.Done || !yield()
// after the assignment: ctx.Done = false    <-- WANT true
```

`ctx.Done` is **false** after the assignment. The left operand is read *before*
`yield` is called, so a `Done` set from inside the call is discarded when the
assignment writes back `staleFalse || !true`.

Second repro, the property any fix must PRESERVE:

```
yield called when Done already true: false
```

The `||` short-circuits, so once `Done` is true `yield` is never called again.

**Instances, measured:** one production site, `gui/run_flow.go:49`. `:98` is
`if ctx.Done || !yield()` — a test, not an assignment, so it cannot clobber.
Three test harnesses (`gui_test.go:510`, `unlock_session_test.go:748`,
`start_screen_touch_test.go:38`) share the shape but nothing sets `Done` mid-yield
in them.

## The implementer's report, verbatim

Task 3 — STOPPED. Found a real defect in the plan, not implemented further, not committed, per your explicit instruction to stop rather than work around a wrong plan.

The defect: the plan's pre-existing (Task-1-inherited, unchanged) line

```go
ctx.FrameCallback = func(o op.Op) {
    ctx.Done = ctx.Done || !yield(o)
}
```

self-clobbers any `ctx.Done = true` set from *inside* the current `yield(o)` call — which is exactly what Task 3's wipe trigger does (`wiping = true; ctx.Done = true; break // unwind, never exit`, executed deep inside the `yield(o)` call chain via `ctx.Frame`). Go evaluates the `||`'s left operand (`ctx.Done`) *before* calling `yield(o)`; since `break` lets the range body complete normally, `yield(o)` returns `true`, so `!yield(o) == false`, and the assignment becomes `ctx.Done = false(stale) || false = false` — wiping out the flag the wipe block just set.

Verified two ways:
1. Isolated minimal repro (`a = a || f()` where `f` sets `a=true` and returns `true`): printed `c.Done after assignment: false`.
2. A `println` placed directly at the wipe site in `run_flow.go` showed `wiping= true ctx.Done= true` at the point of the trigger, then `ctx.Done=false` immediately after `ctx.Frame` returns to the flow — reproduced with all three of my new tests (which all failed the same way: `boundedFlow`'s `n>100000` panic → "wiping stuck true?").

This isn't a test-harness artifact — the production consumer (`for range gui.Run(p, ver) {}` in `cmd/controller/main.go`) also never stops ranging, so `yield(o)` returns `true` there too, and the same clobber would occur on real hardware: the wipe would never actually persist `ctx.Done`. It also isn't unique to Task 3's `wipeNowHook` trigger — Task 4's warning-expiry wipe (`wipeAt` block) sets `ctx.Done` the identical way, so this blocks the whole unwind mechanism, not just my slice of it.

Current repo state (`/scratch/code/shibboleth/seedhammer-b2b`, branch `b2b`): Task 2 committed at `aa704b6`. Task 3's attempt is left **uncommitted** in the working tree (`gui/run_flow.go`, `gui/run_flow_test.go`) for inspection — `go build ./gui/` is clean, but the three new tests (`TestRunWipeUnwindsAndRestartsTheFlow`, `TestRunDiscardGuardSwallowsExtraFrameAfterWipe`, `TestRunTwoWipesEachRestartCleanly`) fail deterministically (not hanging) for this reason; the rest of the existing `./gui/` suite is unaffected (52s, only those 3 fail). I did not run the TinyGo device build since Task 3 isn't green.

I did not attempt a fix — that's a design change to normative unwind logic and belongs in front of the plan's architect/R0 loop, not invented by the implementer.

## Why four review rounds missed it

Everyone — me included — read `ctx.Done = ctx.Done || !yield(op)` as "Done is
sticky". It **is** sticky against being cleared by a later `false`. It is not
sticky against a mutation that happens *during* the call, because the left
operand was already read. Round 0's design lens explicitly blessed "the unwind is
sound" after walking every nested loop on the secret-session path to confirm each
exits on `ctx.Done` — the right check, one level too high. Nobody traced the
assignment's own evaluation order.

It is also the one line nobody thought to question, because Task 1 moved it
**verbatim** from `gui/gui.go:2949` and the plan's own fact 2 recorded it as
pre-existing, established behaviour. Inherited code inside a moved block reads as
settled.

## The fix, and the four cases it must satisfy

```go
if ctx.Done {
    return
}
if !yield(o) {
    ctx.Done = true
}
```

Executed truth table:

| case | Done after | yield called | required |
| --- | --- | --- | --- |
| wipe sets Done during yield | **true** | yes | true — this is the bug |
| consumer stops ranging | true | yes | true |
| ordinary frame | false | yes | false |
| Frame after Done is already true | true | **no** | true, and yield must NOT be called |

The last row is the property the `||` short-circuit was silently providing. A
naive `ctx.Done = !yield(o) || ctx.Done` (operand swap) would fix the clobber and
**lose** it.
