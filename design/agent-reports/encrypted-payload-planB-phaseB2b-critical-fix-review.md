# B2b — independent review of the `ctx.Done` clobber fix (`b27af1f`)

**Reviewer:** independent agent (opus), 2026-08-09. **Scope:** `b27af1f` only —
is the fix correct, is the clobber class present elsewhere, and what else in the
plan was authored on the wrong mental model. Everything else (timer arithmetic,
warning, harness, Tasks 5–8, follow-up register, style) was excluded by the brief
and was not re-audited. Go type-checking and citation resolution were taken as
already machine-verified and not re-derived.

**All evidence below was executed.** Working copy of the b2b worktree at
`$SCRATCH/b2b-review` (`cp -a` of `/scratch/code/shibboleth/seedhammer-b2b`,
which carries Tasks 1–2 committed plus Task 3's uncommitted attempt), with
`b27af1f`'s `FrameCallback` form patched in. The user's worktree was **not**
modified. Toolchain `go1.26.4`.

---

## 1. VERDICT

**1 Critical / 2 Important.**

The **fix itself is correct and complete** — verified by exhaustive execution, not
argument. The Critical is not in the fix; it is the thing the fix *invalidated*
and the plan did not follow through on: **the discard guard is now unreachable
dead code, and three of the plan's claims plus one of the three tests the
implementer already wrote depend on it being live.** The guard-deletion mutant —
row 2 of Task 3's own mutation table — **survives**, measured.

Runtime behaviour is safe either way. Nothing here says the firmware wipes
incorrectly. What it says is that Task 3 as currently written **cannot close green
against its own definition of done**, and that one of its tests is a false-PASS on
the seed-wipe unwind — the exact class that produced this phase's last two
Criticals.

---

## 2. IS THE FIX CORRECT

### 2.1 The four-row truth table, re-derived independently

Standalone program modelling `FrameCallback` in all three forms (fixed, old `||`,
naive operand swap), run:

```
== FIXED (b27af1f) ==
case                             | Done after | yield called | verdict
wipe sets Done during yield      | true       | true         | PASS
consumer stops ranging           | true       | true         | PASS
ordinary frame                   | false      | true         | PASS
Frame after Done already true    | true       | false        | PASS

== OLD (gui.go:2949) ==
wipe sets Done during yield      | false      | true         | FAIL (want Done=true)
consumer stops ranging           | true       | true         | PASS
ordinary frame                   | false      | true         | PASS
Frame after Done already true    | true       | false        | PASS

== NAIVE SWAP  ctx.Done = !yield(o) || ctx.Done ==
wipe sets Done during yield      | true       | true         | PASS
consumer stops ranging           | true       | true         | PASS
ordinary frame                   | false      | true         | PASS
Frame after Done already true    | true       | true         | FAIL (want yield NOT called)
```

The controller's table reproduces exactly, including that the operand swap passes
three rows and loses the fourth.

### 2.2 Exhaustive 8-case difference between old and new

Over the full input space `(Done_before) x (mutated during yield) x (yield result)`:

```
Done_in  mutate   yieldRet | OLD Done_out   NEW Done_out   | OLD yld  NEW yld
false    false    false    | true           true           | 1        1
false    false    true     | false          false          | 1        1
false    true     false    | true           true           | 1        1
false    true     true     | false          true           | 1        1   <<< DIFFERS
true     *        *        | true           true           | 0        0   (all four rows)

cases where OLD and NEW differ: 1 of 8
```

**This is the completeness argument.** The fix is a strict minimal refinement: it
is bit-identical to the pre-existing line on 7 of 8 inputs, and the 8th is the
bug. No existing caller can regress, because no existing caller sees a different
value or a different number of `yield` calls. The new form also **never writes
`false`** to `ctx.Done`, which is the invariant the wipe needs (`Done` is
monotone within a session; a fresh `*Context` per session restores it).

### 2.3 Against the real code

Fix applied to the copy; `gofmt -l` empty, `go build ./gui/ ./gui/op/` OK.
The three tests the implementer wrote and could not pass:

```
--- PASS: TestRunWipeUnwindsAndRestartsTheFlow (0.00s)
--- PASS: TestRunDiscardGuardSwallowsExtraFrameAfterWipe (0.00s)
--- PASS: TestRunTwoWipesEachRestartCleanly (0.00s)
```

Full package suite: `ok seedhammer.com/gui 51.3s`.

Mutation run against those three tests (each mutant applied alone, then reverted):

| mutant | result |
| --- | --- |
| `break // unwind, never exit` → `return` | **killed** (all 3 fail) |
| `if !wiping {` → `if false {` | **killed** (all 3 fail, via the cap at ~10 s each) |
| hoist `wiping := false` above `for {` | **killed** (all 3 fail) |
| revert `FrameCallback` to the `\|\|` form (the Critical) | **killed** (all 3 fail) |
| **delete the 3-line `if wiping { continue }`** | **SURVIVES — all 3 PASS** |

### 2.4 Re-entrancy / nested screens — does the early return break a caller?

**No.** Three independent checks:

- **Behavioural delta is nil.** Per §2.2, in every case where the new code
  early-returns without calling `yield`, the old `||` **also** short-circuited and
  did not call `yield`. Nothing about post-`Done` `Frame` calls changed.
- **`Context.Frame`'s buffer scrub is untouched.** `gui/gui.go:77-82` is
  `if f := c.FrameCallback; f != nil { f(op) }` followed by `c.B.Reset()`. The
  early return exits the *closure*, not `Frame`, so `c.B.Reset()` — and therefore
  `clear(b.refs)` (`gui/op/op.go:374-378`) — still runs on every post-`Done`
  frame. The plan's tail comment's conclusion holds (see M-1 for its wording).
- **No caller can busy-spin.** AST scan (`go/parser`, not grep) over every
  non-test file in `gui/`: **54 `ctx.Frame` call sites; 0 whose innermost
  enclosing loop is anything other than `for !ctx.Done`.** The only two sites with
  no enclosing loop at all are one-shot frames that fall through immediately
  (`gui/slip39_polish.go:220` `showSLIP39Message`, `gui/verify_address.go:177`
  `runVerify`'s "Verifying…" progress frame). Every loop that draws re-tests
  `ctx.Done` each iteration, so the early return always terminates the loop rather
  than starving it.

### 2.5 One thing the fix is doing that is worth writing down

Row 4 is **not cosmetic and not merely an optimisation.** Executed:

```go
it := func(yield func(int) bool) { ok := yield(1); yield(2) }
for v := range it { break }
// panic: range function continued iteration after function for loop body returned false
```

On the *consumer-stops-ranging* path `yield` has genuinely returned `false`, and
calling it again is a hard panic — on a watchdog-less device. The `||`
short-circuit was silently preventing that; the early return now is. So fact 3's
retraction ("it does NOT panic") is right, but the reason is *because of this
line*: the panic is prevented, not impossible. Anyone who later "simplifies"
`FrameCallback` back to a single assignment re-introduces both defects at once.

**Conclusion: the fix is correct, minimal, and complete as a fix. 0 findings
against `b27af1f`'s code change.**

---

## 3. OTHER INSTANCES OF THE CLASS

**None found in production code.** Where I looked, and what each turned out to be:

**Mechanical sweep** — every non-test `.go` file in the fork, for an assignment
whose RHS contains both a call and a `||`/`&&`:

| site | verdict |
| --- | --- |
| `gui/run_flow.go:49` (working tree) / `:32` (committed) | **the known one.** Fixed by `b27af1f` |
| `gui/scan.go:34` `s.overflow = s.overflow \|\| s.n == len(s.buf)` | safe — RHS contains **no call**, so nothing can mutate between read and write |
| `cmd/controller/engraver.go:284` `e.sdiagAvailable = e.sdiagAvailable \|\| !S_DIAG.Get()` | structurally identical shape, **cannot bite**: `S_DIAG` is a `machine.Pin` (`cmd/controller/platform_sh2.go:102`) and `Pin.Get()` is a GPIO read with no path to `e` |
| `cmd/controller/platform_sh2.go:780` `ours = ours \|\| bytes.Equal(...)` | safe — `bytes.Equal` is pure; `ours` is a local |
| `codex32/codex32.go:266` `isUpper = isUpper && unicode.IsUpper(c)` | safe — pure callee |
| `engrave/engrave.go` ×10, `cont = cont && yield(...)` / `&& DelayMove(yield, ...)` | **the closest analogue** — a consumer-supplied `yield` combined with a pre-read flag. Safe: `cont` is a plain function-local `bool`, no closure in those functions captures it, and `yield` is supplied by the *caller* of the iterator, so it holds no reference to `cont`. The pattern is also monotone-downward there (`false && …` stays false), which is the intended stop semantics |

**Targeted sweep of the unwind path and `Run`'s inner loop** (the brief's specific
ask). Every assignment in the gated `run_flow.go` block, checked for a callee that
can reach the LHS:

- `evts = pl.AppendEvents(wakeup, evts[:0])` — `evts` is a local slice header; the
  callee gets a copy and returns a new one. Not the class.
- `a.idle.start = now`, `a.armed = armed`, `a.idle.active = idle` — RHS is a
  pre-computed value, no call. `a` is closure-captured but no callee reaches it.
- `armed := ctx.wipe.armed()` — `:=`, not a read-modify-write.
- `ctx.WakeupAt(...)` / `Context.Reset` (`gui/gui.go:93-105`) — `Reset` writes the
  zero value *first*, then conditionally overwrites from `c.Router.Reset()`.
  `EventRouter` holds no `*Context` (`gui/event.go`, struct is
  `{events, filters, pointer}`), so it cannot write back. Not a read-then-restore.
- `ctx.Router.Events(d, evts...)` — no assignment, and again no `*Context` handle,
  so **no event handler can set `ctx.Done` mid-call**.

**`gui/run_flow.go:98` (`if ctx.Done || !yield()`) — confirmed safe**, for two
independent reasons:

1. It is a **test, not an assignment**: there is no write-back, so nothing can be
   discarded. Even in the hypothetical where the outer `yield()` set `ctx.Done`
   and returned `true`, the flag survives and the *next* iteration of the same
   inner `for` re-reads it and returns — one tick late, never lost.
2. It cannot arise anyway. Both production consumers have an **empty** range body
   — `cmd/controller/main.go:34` and `cmd/emu/main.go:33` are both
   `for range gui.Run(…) {}` — and the outer `yield` is `func() bool` with no
   arguments, so neither production nor test code has any handle on `ctx` from
   inside it.

The three test harnesses that share the shape (`gui/gui_test.go:510`,
`gui/unlock_session_test.go:748`, `gui/start_screen_touch_test.go:38`) were
re-checked: none sets `Done` mid-yield. Confirming the controller's count.

---

## 4. CLAIMS BUILT ON THE OLD MODEL

### C-1 (Critical) — the discard guard is dead code, and four artefacts still say it is load-bearing

**Measured, three ways:**

1. **Deleting `if wiping { continue }` kills nothing.** With the fix applied, the
   three Task 3 tests all still PASS with the guard removed (§2.3 table, last row).
2. **The guard is never executed at all.** Replacing its body with
   `panic("DISCARD-GUARD REACHED")` and running the *entire* `./gui/` suite:
   `ok seedhammer.com/gui 51.7s`. Zero hits, package-wide. (I had first tried
   `println` — a passing `go test` swallows the child's stderr, so that result was
   discarded and re-run as a panic, which cannot be swallowed.)
3. **It is provable, not just unobserved.** `wiping = true` is set at exactly two
   sites, each of which sets `ctx.Done = true` on the adjacent line; nothing sets
   `Done` back to `false` within a session (`Context.Reset` does not touch it —
   `gui/gui.go:99-105`); and `Done == true` now makes `FrameCallback` return
   before `yield`. So `wiping == true` ⟹ no further `yield` ⟹ **no further range
   body iteration** ⟹ the guard's condition is never evaluated as true. The guard
   is only reachable *under mutation* (the hoisted-`wiping` mutant, where session 2
   starts with `wiping` already true).

**What was authored on the old model and is now false:**

| # | location | the claim | status |
| --- | --- | --- | --- |
| a | plan L61-68, **fact 3 as corrected in `b27af1f`** | "It does NOT panic: `FrameCallback` returns early once `Done` is true, so `yield` is never called after it returned false. … **without the discard guard the extra frame reaches `if ctx.Done \|\| !yield() { return }` in the inner loop and executes the `return`**" | **self-contradictory.** If `yield` is never called, no range-body iteration exists to reach the inner loop. The fold corrected the *panic* half and left the *hazard* half standing on the model it had just retracted |
| b | plan L638-643, Task 3 change 2 | "**Required, not defensive** (fact 3). … without the guard that iteration reaches … and executes the `return`, **converting the wipe into a full GUI exit**" | **false.** There is no such iteration. Measured: guard deleted, wipe still restarts the session |
| c | plan L689, **mutation row 2** | anchor `if wiping {` → delete; "must be killed by … the wipe becomes a GUI exit, so `\"SESSION 2\"` never drawn" | **false anchor — the mutant survives.** Task 7's runner would report it surviving, and Task 3 cannot close green against its own 3.3 table |
| d | plan L917-921, the in-code comment in the gated block | "The DISCARD GUARD. … Without this, that frame reaches `if ctx.Done … return` below and the wipe becomes a full GUI exit — the machine loses its UI because the timer worked." | **false**, and it ships in the firmware source |
| e | `gui/run_flow_test.go` (uncommitted), `TestRunDiscardGuardSwallowsExtraFrameAfterWipe` | asserts `"EXTRA FRAME"` is never drawn, under a name and a doc comment attributing that to the guard | **false-PASS.** It passes because `FrameCallback` early-returns, and it passes identically with the guard deleted. A test named for a mechanism, asserting a property that mechanism does not provide |

Item (e) is why this is graded Critical rather than Important: a green test on the
seed-wipe unwind that cannot fail when its stated mechanism is removed is the
same class as the false-GREEN assertion the 2026-07-26 boot-key review caught, and
this project's own memory (`mutation-testing-finds-false-passes`) treats it that
way.

**Recommended disposition** (the choice is the author's; both are consistent):

- **Keep the guard as declared defence-in-depth.** Then (a)-(d) must be rewritten
  to say *"unreachable in the correct program; it exists so that a future wipe
  path which sets `wiping` without setting `Done` still cannot draw"*, mutation
  row (c) must be **deleted** (it is unkillable), and test (e) must be renamed and
  re-commented to assert what it actually proves — that `FrameCallback`'s early
  return discards post-`Done` frames.
- **Or delete the guard.** Measured to be a no-op. Note this also removes the
  hoisted-`wiping` mutant's kill mechanism as currently described (see M-4).

Either way, **the plan must not ship a mutation row whose mutant survives.**

### C-1 corollary — what *does* discard the post-`Done` frames

For the record, since the plan will need this sentence: it is
`FrameCallback`'s `if ctx.Done { return }`, which drops the op without drawing
while still letting `Context.Frame` run `c.B.Reset()`. Same visible outcome the
guard was written for, one level up, and it is the only version that also
survives the consumer-stop path (§2.5).

### I-1 (Important) — fact 2 says the fix is already in Task 1's committed code. It is not.

Plan L51-56 now reads: *"Task 1's version of `run_flow.go` therefore differs from
`gui.go` here, and it is the ONE line in the move that is not verbatim."*

Measured against the shipped commits:

```
$ git show fbe31ab:gui/run_flow.go | grep -n ctx.Done     # Task 1
32:				ctx.Done = ctx.Done || !yield(op)
$ git show aa704b6:gui/run_flow.go | grep -n ctx.Done     # Task 2, = HEAD
32:				ctx.Done = ctx.Done || !yield(op)
```

Task 1's move **was** verbatim, and Task 2 inherited it. The corrected form exists
only inside Task 4's gated whole-file block. Compounding it, **Task 3's own body
still says "Three changes, all in `gui/run_flow.go`"** (L613) and enumerates the
session loop, the discard guard and the wipe — the `FrameCallback` correction is
**not one of them**. An implementer working Task 3's numbered steps has no
instruction to touch it.

Fix: state that the fix is a **Task 3 edit to already-committed Task 1 code**, and
add it as change **4** in Task 3's list (or renumber to four changes). If it is
meant to land as its own commit ahead of Task 3, say that instead — but say
something, because right now the one line that four rounds missed is the one line
no task is told to write.

### I-2 (Important) — no mutation row anchors the fix itself

Task 3's 3.3 table has four rows; none of them is the Critical. The single most
expensive defect in this phase would not be re-checked by Task 7's runner.

It is writable and it kills — measured (§2.3): reverting `FrameCallback` to the
`||` form fails **all three** Task 3 tests. Suggested row, anchored on a line that
`plan-mutation-anchors.py` can match uniquely:

| file | anchor (unique) | → replace with | must be killed by |
| --- | --- | --- | --- |
| `run_flow.go` | `					if !yield(o) {` | `					if ctx.Done = ctx.Done \|\| !yield(o); false {` | all three Task 3 tests — the wipe's `ctx.Done` is discarded, `"SESSION 2"` is never drawn |

(Any mechanically-appliable spelling of "revert to the `||` form" is fine; the
point is that the row exists and that the plan records the measured kill.)

---

## 5. MINOR / NIT

- **M-1** (plan L1038-1045, tail comment) — "*`clear(b.refs)` … runs on the last
  frame drawn — then again **after every discard-guarded Frame** during the
  unwind*". The **conclusion is correct** (the abandoned `Context`'s buffer is
  zeroed by the time the session loops) but the named mechanism is wrong: those
  frames are not discard-guarded, they are early-returned inside `FrameCallback`,
  and the scrub survives only because `c.B.Reset()` sits *outside* the callback in
  `Context.Frame`. Re-word — this is a funds-relevant scrub and the reason it
  works should be the reason written down.
- **M-2** (plan L80-81 and the table row at L91) — "the session loop and discard
  guard make `ctx.Done` survivable". It is the `FrameCallback` fix that makes
  `Done` survivable; the session loop makes it *non-terminal*. Same substitution
  as I-1, in the phase-boundary section.
- **M-3** (plan L642-643) — "Note it also skips the *inner* loop, which is what
  keeps `ctx.Done` from being re-read as an exit." Moot under the fix: no
  iteration occurs, so there is nothing to skip.
- **M-4** (plan L680-682, L411-427, L487-489) — the `boundedFlow` rationale
  ("*the discard guard means `runSession`'s tick counter never advances*") is
  **still correct** and I verified the hoisted-`wiping` mutant is killed. But it is
  now the *only* remaining reason the guard exists in the source, and it is a
  property of a **mutant**, not of the program. If C-1 is resolved by deleting the
  guard, this rationale and `boundedFlow`'s doc comment need re-deriving (the
  hoist mutant would then spin in the session loop rather than in the guard —
  `boundedFlow` still catches it, but for a different reason).
- **N-1** — fact 3's "cannot happen" is worth strengthening to "cannot happen
  **because** `FrameCallback` returns before `yield`", per §2.5: the
  range-over-func panic is real and demonstrable on the consumer-stop path, and
  this line is the only thing preventing it. As written, a future reader could
  conclude the panic was never a risk and delete the guard clause.
- **N-2** — Task 3's step 3.2 says "Write the changes" and defers the whole-file
  form to Task 4's block. That indirection is what let the `FrameCallback` line
  fall between the two tasks (I-1). Consider a one-line pointer in 3.2.

---

## Reproduction

```sh
cp -a /scratch/code/shibboleth/seedhammer-b2b $S/b2b-review   # Tasks 1-2 committed
                                                              # + Task 3 uncommitted
# patch FrameCallback to b27af1f's form, then:
go build ./gui/ ./gui/op/ && gofmt -l gui/run_flow.go
go test ./gui/ -run 'TestRunWipe|TestRunDiscardGuard|TestRunTwoWipes' -v -count=1
go test ./gui/ -count=1                                       # full suite, 51s
# mutants: apply one, run the three tests, revert
# dead-code proof: replace the guard body with panic(), run the FULL suite
```

Standalone truth-table and exhaustive-difference programs, the AST scan of all 54
`ctx.Frame` call sites, and the range-over-func panic repro are under
`$SCRATCH/{tt,tt2,astscan,panicrepro}`.
