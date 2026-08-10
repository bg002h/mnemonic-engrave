# R0 round 3 — fold verification, DESIGN_b2b_residency_zeroing.md (F-107 / F-108)

**Reviewer:** independent architect agent (adversarial), 2026-08-10.
**Scope:** fold verification ONLY — (A) did the fold fix I-A, I-B and I-C, and
(B) did it introduce a new defect. **Not a fresh audit.** Round 2's
§"Explicitly checked, no finding" and §"What must be true to reach GREEN" were
honoured as settled and are not re-derived.
**Artifacts:** round 2 = `design/agent-reports/2026-08-10-r0-residency-round2.md`;
folded design = `design/DESIGN_b2b_residency_zeroing.md`; fold =
`899cd60..HEAD` (`b843cf4` design, `768fea5` register; `b226d8b` is an unrelated
F-106 hardware commit, correctly separated).
**Code read (read-only):** `/scratch/code/shibboleth/seedhammer-b2b` @ `3de8aa1`
and the implementation in `/scratch/code/shibboleth/seedhammer-gate-orphan`.
**Measurement:** all numbers below were produced by running something on a
throwaway **copy** of the gate worktree. Neither real tree was modified.

---

## Verdict: **GREEN (0C/0I)**

0 Critical · 0 Important · 5 Minor · 3 Nit.

All three Importants are genuinely closed, and I verified each one by execution
rather than by reading — including the two facts the whole resolution rests on,
which the design asserts but nothing pinned: that clearing `splineResumer`'s
catch-up array cannot reach `SafePointer.history` (it does not — `Resume`
returns a non-aliasing array, measured), and that zeroing `knotBuf` at
iterator-exit does not break the restart path (it does not — a re-range yields
the identical 3,863 knots, measured). §Gate coverage's uncomfortable claim
— *"deleting all four bodies would leave the suite just as green"* — is **true**;
I deleted all four and the suite passed.

The five Minors are all **records**, not code: a residual `ClearHistory` leaves
behind and does not name, two places where a shipped comment and a register entry
identify the skip/gap too narrowly, one stale `FOLLOWUPS.md` entry the fold
updated for F-111 but not for F-110, and one omission from §Gate coverage's own
"what is NOT built" list. This is the pattern this project has already named
(*records are the weak half*), and none of it gates.

---

## (A) Per-finding verdicts

| # | verdict | evidence |
| --- | --- | --- |
| **I-A** (`splineResumer.catchup` unreachable) | **RESOLVED** | `defer clear(c)` at `gui/engraver.go:265`, inside the `if c := s.catchup; c != nil` block. All four sub-questions checked, three by execution — see §I-A below. |
| **I-B** (no seam for `knotBuf`) | **RESOLVED** | `defer func(){…}()` at `engrave/engrave.go:1046-1051`, inside `planEngraving`'s closure. Verified on all three exit shapes and on the restart path by execution — see §I-B below. |
| **I-C** (comment claims the wipe covers resume state) | **RESOLVED** | `gui/engraver.go:126-130` now reads *"The wipe does NOT cover it either: that unwind is `ctx.B.Scrub()` + `Drawer.Release()` and reaches no engrave state. That hole is F-110, not a covered case."* Checked against `gui/run_flow.go:233-264`: after `if !wiping { return }` the unwind is exactly `ctx.B.Scrub()` (`:245`) and `d.Release()` (`:264`). Accurate, and F-110 is cross-referenced from the shipped comment as the finding required. One under-enumeration → **M3-2**. |
| **M-a** (fix misses more than `ErrTooLarge`) | **RESOLVED** | Design §"The paths where a plate is built and no cut ever happens" is now a three-row table whose middle row is the *ordinary* Back-before-cut path, explicitly labelled "an ordinary operator action, not a failure". `FOLLOWUPS.md` F-111 retitled *"unzeroed wherever a plate is built and no cut happens — SUBSUMED"*, with the widening reasoned out and "do not implement it separately". Register and design agree. |
| **M-b** (no test pins items (2)/(3)) | **RESOLVED** | §Tests rows **7**, **8** and **9** added, each with its mutation line (empty `releaseResumeState`; delete the tail-clear; delete `defer clear(c)`). Row 7 states exactly why row 5 was not enough ("*row 5 asserts the catch-up motion is unchanged, which passes identically whether the method zeroes anything or is a no-op*"). |
| **M-c** (lint reads one file) | **RESOLVED** | `gui/op/funnel_lint_test.go:31-45` now `os.ReadDir(".")` and lints every non-test `.go` in the package; `t.Fatal("INCONCLUSIVE: no package sources found")` if the walk finds nothing. The blind spot is stated in **both** the test comment (`:25-30`, local-alias form) and §Tests row 3. Mutation row executed (raw append in `buffer_len.go` → KILLED). |
| **N-a** ("the goroutine has provably returned") | **RESOLVED** | Design §"two structural facts" now says `runEngraving`, with the `Wakeup()` reason spelled out, and the **shipped** comment (`gui/engraver.go:124`) says "runEngraving has provably returned, with its defers complete" — the fix landed in the code too, not only in the prose. |
| **N-b** ("the abandoned Context") | **RESOLVED** | §Tests row 1 now says "*bound the flow, assert after the session defer has run and before any later frame reuses the buffer*" and names why "abandoned" was the wrong word. |
| **N-c** (unnamed read primitive) | **RESOLVED** | New threat-model paragraph names three candidate primitives (debug/SWD on a recovered board, a disclosure defect that does not replace the image, a service path that dumps RAM) and notes none requires defeating signed boot. |

**Counts:** RESOLVED 9 · PARTIAL 0 · NOT ADDRESSED 0 · REGRESSED 0.

### I-A in detail — the four sub-questions the brief asked

1. **Is the defer registered once, not per knot?** Yes. The block is guarded by
   `if c := s.catchup; c != nil` and its first statement is `s.catchup = nil`
   (`gui/engraver.go:255-256`), so it executes on the first `Knot` call of a run
   and never again. Executed: 1,000 `Knot` calls on one resumer, `catchup` nil,
   one clear.
2. **Does it fire on the early `return 0, err`?** Yes — a `defer` registered
   inside a block fires at *function* return, so both the loop-completion path
   and the driver-error path zero. Executed with a fake `Knotter` failing on the
   3rd catch-up knot: **0 non-zero knots left**.
3. **Does anything else still reach that array?** No. `e.catchup()` →
   `SafePointer.Resume` → `make([]bspline.Knot, 0, len(s.history)+10)`; the only
   reference is `splineResumer.catchup`, nilled before the clear. The knots go to
   `drv.Knot(k)` **by value**.
4. **Can `clear(c)` corrupt resume state — NC1 by another route?** **No, and this
   is the load-bearing fact nothing in the design pins.** `Resume` builds a fresh
   array and *copies* history into it (`move = append(move, s.history...)`), so it
   cannot alias. Executed: build a 12-knot history, `Resume`, `clear` the result,
   compare — history byte-identical. Also measured that `Resume`'s two-append
   sequence does **not** reallocate for histories of 0/1/5/12/40 knots
   (initial cap `len(history)+10`, final len `len(history)+8`), so it orphans no
   intermediate array either.

Residual on I-A: **N3-1** (the array is not zeroed if `Knot` is never called).

### I-B in detail — the four sub-questions the brief asked

1. **Correct on every exit?** Executed against a real 12-word plate
   (`String(constant.Font, 4mm, "ABANDON … ACCIDENT")`, 3,863 knots) with a
   caller-supplied 100-knot buffer:

   | exit shape | non-zero knots left in the caller's buffer |
   | --- | --- |
   | full range to completion | **0 / 100** |
   | `break` after 37 knots (the `!yield` early return) | **0 / 100** |
   | `bspline.Measure` only, no cut | **0 / 100** |

   (Round 2 measured **10 of 100** left on the never-cut path before the fix.)
2. **Does zeroing at iterator-exit break the restart path?** **No.** Ranged the
   same `Curve` twice and compared knot-for-knot: **identical over all 3,863
   knots**. The closure reopens with `spline := knotBuf[:0]` and rebuilds from the
   upstream `Engraving`, exactly as the design argues, and nothing in the loop
   reads an index it has not written in this range.
3. **Is the second clear right, or misleading?** Right, and correctly described
   as dead-today. The loop only ever reslices `spline[:0]`
   (`spline = appendLine(spline[:0], …)`, `spline = append(spline[:0], …)`), so
   `cap(spline)` can change **only** on a reallocation — the guard is an exact
   reallocation detector, not an approximation. Measured high-water cap = 100 =
   the initial cap, so it is dead code, and the design says so in the same breath.
4. **Would a multi-reallocation orphan intermediates?** Yes, and the design states
   it verbatim as a known limitation with the condition under which it becomes
   live. Correctly scoped.

---

## (B) New findings

### CRITICAL (0)

None. I specifically hunted the two ways this fold could have created a
wrong-plate path — `clear(c)` reaching `SafePointer.history`, and zeroing
`knotBuf` breaking a re-range — and executed both. Neither exists.

### IMPORTANT (0)

None.

### MINOR (5)

#### M3-1 — `ClearHistory` leaves `safePoint` and `progress` behind, and nothing names them as a residual

**Anchor:** `engrave/engrave.go:1714-1718` (written by this fold) and
`gui/engraver.go:115` — *"releaseResumeState zeroes **the job's resume state**"*.

`ClearHistory` zeroes `history[:cap]`, truncates, and resets `completed`. It does
**not** touch the other two fields of the same struct. Measured:

```
after ClearHistory: history non-zero=0/16  safePoint={X:1234 Y:5678}  progress=99  completed=0
```

`SafePointer.safePoint` is a `bezier.Point` on the engraved path — seed-derived
geometry by this design's own classification, and it is *resume* state by the
design's own split (it is what `Resume` moves the head back to).

**Failure scenario.** Operator cuts a plate, presses Back, the machine sits on
the plate list. `releaseResumeState` fires on `engraveDone`; every history knot
is zeroed; the last safe point survives in RAM until the job object is collected,
inside the exact window the threat model says the adversary operates in. Magnitude
is one control point — far below the ~10-knot residue round 0 and round 2 both
graded not-seed-recoverable, which is why this is Minor and not Important. What
makes it a finding at all is that the design's §"What this design does not cover"
lists four named residuals and this is not one of them, while the method's own
doc-comment asserts the broader claim.

**Smallest fix.** Two lines in `ClearHistory` (`s.safePoint = bezier.Point{}`,
`s.progress = 0`) — free, and it makes the method's name honest — *or*, if there
is a reason to keep the safe point, add it to §"What this design does not cover"
and narrow `releaseResumeState`'s first line to "the retained resume knots".

#### M3-2 — the shipped comment identifies the skip with the wipe; the double-Back skip is the common one

**Anchor:** `gui/engraver.go:126-127`, written by this fold: *"If the job is still
running — **Engrave returning because `ctx.Done`, i.e. the wipe** — this skips
rather than racing the goroutine."*

`releaseResumeState` skips on `default:`, i.e. on `engraveIdle`, `engraveRunning`
**and `engraveStopping`**. The `engraveStopping` case is not the wipe: it is the
double-Back path — Back while running (`gui/gui.go:2726` `Stop()`, screen stays),
then Back again before the goroutine's send is consumed, so `Status()` still
reports `engraveStopping` and `gui/gui.go:2723-2724` takes `break frames`.
`Engrave` returns, the screen is gone, and `SafePointer.history` is never zeroed.

Round 2's §"Explicitly checked" already blessed the *guard* on this path (it must
skip — the goroutine may be live), and I agree; the behaviour is correct. The
finding is the enumeration: `i.e. the wipe` reads as an identification, and this
is a shipped firmware comment on a funds path.

**Failure scenario.** An operator aborts a plate and leaves. Someone later greps
for why resume state survived, finds a comment saying the only skip is the wipe,
and looks in the wrong place — or scopes F-110's fix to the `ctx.Done` path and
leaves the more reachable one open.

**Smallest fix.** Change `i.e. the wipe` to `— the wipe (ctx.Done), or a Back
taken while the state is still engraveStopping —`.

#### M3-3 — `FOLLOWUPS.md`'s F-110 entry is now stale: it still describes the pre-I-A placement for `catchup`

**Anchor:** `design/FOLLOWUPS.md`, F-110: *"`SafePointer.history` **and
`splineResumer.catchup`** are resume state … The design zeroes **them** at
`EngraveScreen.Engrave`'s return, and only when the job is terminal"*, then gap 1
*"The wipe path skips it."*

That is the placement round 2's I-A killed. The fold moved `catchup` to
`defer clear(c)` in `splineResumer.Knot`, where it is zeroed **unconditionally**,
on the first resumed knot, terminal or not — verified by execution above. So on
the wipe path `catchup` is already zero, and F-110 gap 1 no longer applies to it;
only `history` is left. The fold rewrote F-111's register entry in its own commit
(`768fea5`) and did not revisit F-110's.

**Failure scenario.** F-110 is B2b-owned. Whoever picks it up reads the entry,
believes `catchup` is unzeroed on abandon, and adds a job-level sweep — which is
precisely what round 2's I-A proved zeroes a nil slice and delivers false
assurance, reintroducing the finding the fold just closed. This is the same
failure round 1's NN1 filed against F-108's own entry.

**Smallest fix.** Strike `splineResumer.catchup` from F-110's opening sentence
and from gap 1; add one line: *"`catchup` is closed by `defer clear(c)` in
`splineResumer.Knot`, unconditionally — this entry covers `SafePointer.history`
only."*

#### M3-4 — §Gate coverage's "What is NOT built" list omits that F-107's scrub placement is unimplemented

**Anchor:** design §Gate coverage, *"What is NOT built, and gates nothing yet"* —
three bullets: F-108's zeroing is untested, test rows 1 and 5–9 unwritten, and
the two comment corrections unapplied.

Measured on the gate worktree:

```
$ grep -rn "\.Scrub()" --include="*.go" . | grep -v _test.go
gui/engraver.go:128:// either: that unwind is ctx.B.Scrub() + Drawer.Release() and reaches no
gui/run_flow.go:245:			ctx.B.Scrub()
```

One non-test call site, unchanged from b2b. Neither bracket the design requires —
`unlockSecretSession`'s own defer, nor `gui/unlock_kdf.go:137`'s, which round 0's
I1 added and round 2 confirmed safe — has a `Scrub`. So the *entire* fix for
F-107's headline defect ("the rendered seed is scrubbed only on the wipe path")
is design-only, alongside its outgrown-array half which **is** built.

It is inferable — the build list enumerates "the op.Buffer funnel" and "all four
F-108 items" and nothing else — but the section exists to be explicit, and this
project's own rule is that a gate hiding its blind spot is worse than no gate.
Minor because the register and the design body are both correct and no reviewer
budget was misdirected; it is a completeness gap in one list.

**Smallest fix.** A fourth bullet: *"F-107's scrub placement (both brackets) is
not applied — `.Scrub()` still has exactly one non-test call site,
`gui/run_flow.go:245`. Only the outgrown-array half of F-107 is built."*

#### M3-5 — `SafePointer.ClearHistory` is exported on a funds path with no precondition in its doc

**Anchor:** `engrave/engrave.go:1708-1718`. The doc says what it zeroes and why
it zeroes to cap. It does not say **when it is legal to call**.

`releaseResumeState` carries the precondition ("Safe to call only where a restart
is impossible", `gui/engraver.go:132`), but that is the *caller*, in a different
package. The exported method is reachable from anywhere in the tree.

**Failure scenario, which is NC1 by a new route.** A future author adds a
`ClearHistory()` to `engraveJob.Stop()` — a natural-looking "release state when
the operator stops" — and the operator then holds to resume at `gui/gui.go:2747`.
`Resume` returns a move-to-safe-point line with an **empty** history, while
`runEngraving` still sets `skipKnots := e.nknots` and resumes at the knot where it
stopped. The head is parked at the safe point and the spline continues from a
point further along: the knots between are never cut. **Wrong plate**, from a
method whose doc gives no reason not to call it.

Not Important because no such caller exists and the one that does is guarded —
but the design's own thesis is *a copy of the seed that a comment says is
bounded, with nothing in the code enforcing the bound*, and this is an API with
no comment at all on the dangerous axis.

**Smallest fix.** Three sentences on `ClearHistory`: it destroys resume state;
calling it while a restart is still possible resumes from the wrong point and
cuts a wrong plate; the only legal caller is one that has proved
`Start()` can no longer run. (Enforcing it in code would need a
`SafePointer` state flag — not worth it here; the doc is.)

### NIT (3)

- **N3-1 — `defer clear(c)` misses the case where `Knot` is never called.** The
  clear lives in `splineResumer.Knot`, so a run that consumes zero knots leaves
  the array intact. Measured: `newSplineResumer` with an 8-knot catch-up and no
  `Knot` call → **8 of 8 non-zero**. Reachable only if a *resumed* run's
  `skipKnots := e.nknots` (`gui/engraver.go:197`) consumes every knot the spline
  yields — i.e. the operator presses Back during the final knot of a ~21-minute
  plate, then holds to resume. Sub-millisecond window, and the residue duplicates
  knots that `SafePointer.history` still holds and F-110 already owns, which is
  why this is a Nit rather than the completeness defect it looks like. Worth one
  clause in the code comment ("the array survives a resumed run that consumes no
  knots; see F-110") so the next reader does not read `defer clear(c)` as
  unconditional.
- **N3-2 — "Registered once per job" is one word off.** `gui/engraver.go:264`. A
  fresh `splineResumer` is built per `runEngraving` invocation
  (`gui/engraver.go:196`), so it is once per *run*, not per job — a job that
  restarts three times registers three clears, which is exactly the property the
  fix needed and the sentence slightly understates. The point it is making
  ("not per knot") is correct.
- **N3-3 — the design's own §"Follow-ups filed by this fold" still carries
  F-111's withdrawn scope.** It reads *"F-111 — `knotBuf` on the `ErrTooLarge`
  path (`unlock_session.go:191-193`)"* with no "subsumed" marker, while
  §"The paths where a plate is built and no cut ever happens" says F-111 "is
  therefore subsumed and should be closed as part of this design". `FOLLOWUPS.md`
  is correct, so the authoritative record is right and this is only the design's
  own summary list disagreeing with its body — the residue of round 1's central
  complaint, in one line.

---

## Worklist to GREEN

**Already GREEN.** Nothing blocks. The five Minors and three Nits are recorded
above with their smallest fixes; **M3-3** (stale F-110 register entry) and
**M3-1** (name or zero the `safePoint` residual) are the two worth folding in
this pass, because both are one-liners and both are the kind of stale record this
document exists to stop producing. Per the project's proportional re-review rule,
a comment/register fold of this shape does **not** re-trigger a gate.

---

## Explicitly checked, no finding

Recorded so a round 4, if one happens, does not re-derive them.

- **§Gate coverage's four-item honesty claim is TRUE, and I proved it by
  mutation.** Deleted all four bodies on a copy of the gate worktree —
  `planEngraving`'s `defer`, `SafePointer.ClearHistory`'s body,
  `defer clear(c)`, and the tail-clear at `engrave/engrave.go:1701` — and ran
  `go test ./gui/... ./engrave/ ./seal/`: **all packages ok** (`gui` 40.1s,
  `gui/op`, `gui/saver`, `gui/text`, `gui/widget`, `engrave`, `seal`). The
  section is not understating and not overstating: no test asserts any of the
  four buffers ends up zero, and it says exactly that.
- **The three written tests are where §Gate coverage says they are, and assert
  what it says.** `gui/op/outgrown_test.go` (rows 2 and the
  live-contents check), `gui/op/funnel_lint_test.go` (row 3, now directory-walking),
  `gui/orphan_measure_test.go` (row 4, with its own `INCONCLUSIVE` guard and its
  own admission that it does not verify the zeroing). Rows 1 and 5–9 are absent,
  as claimed.
- **`releaseResumeState`'s switch is correct and its `default` is fail-safe.**
  `engraveState` has exactly six values; the switch treats
  `engraveStopped`/`engraveDone`/`engraveFailed` as terminal. `engraveIdle` is
  skipped harmlessly — there is no assignment back to `engraveIdle` anywhere
  (`grep -n "State = engrave"` returns only Stopping/Running/Stopped/Done/Failed
  plus one test), so Idle implies never-started implies empty history.
  `engraveStopping` must be skipped (live writer). A future seventh state would
  fall to `default` and skip, which is safe against a race and open on residency —
  acceptable, and the direction a default should fail in.
- **`Engrave`'s defer ordering is correct and nothing observes the job between
  the two calls.** `gui/gui.go:2715-2721` is straight-line code with no frame and
  no yield. `Stop()` (`gui/engraver.go:85-92`) can only move
  `engraveRunning → engraveStopping`, which makes `releaseResumeState` skip; it is
  a no-op on every terminal state, so it can never turn a clearable state into a
  skipped one. Reversing the order would be equally safe — the ordering is
  conservative, not load-bearing.
- **No data race on the new path.** `e.status` is written only by
  `Stop`/`Start`/`Status`, all on the UI goroutine; `runEngraving` never touches
  it. `e.safePoint` is written by the engrave goroutine and read by
  `ClearHistory`, ordered by the `e.errs` receive that produced the terminal
  state. Confirmed empirically: `go test -race ./gui/` → **ok, 348s, clean**.
- **`engrave`'s `-race` failures are pre-existing and unrelated.**
  `TestConstantQR` shares one `math/rand.Rand` across parallel subtests
  (`engrave/engrave_test.go:54,73`). **Identical on untouched b2b `3de8aa1`** —
  I ran both. Test-harness only, no production code involved. Noted so nobody
  chases it as fallout from this fold.
- **The design's code blocks match the shipped code**, statement for statement:
  item (1) ↔ `engrave/engrave.go:1046-1051`; item (2) ↔ `gui/gui.go:2715-2721`
  plus `gui/engraver.go:134-141`; item (2b) ↔ `gui/engraver.go:265`; item (3) ↔
  `engrave/engrave.go:1701`. No divergence between what a reviewer read and what
  an implementer wrote.
- **The tail-clear at `engrave/engrave.go:1701` is correctly placed and correctly
  sliced.** It runs *before* `s.history = s.history[:rem]`, so `s.history[rem:cap]`
  still spans the full array; `k0`/`k1`/`k2` and `k` are value copies taken before
  it, so `s.safePoint = k0.Ctrl` on the next line is unaffected. `[rem:cap]` is
  dead by construction on every path, exactly as the design says.
- **The `op.Buffer` funnel's call-site rewrites are semantics-preserving.** Every
  converted site keeps its `len(b.args)` reads *before* the append
  (`ensureLatest`, `newCompose`, `group.add`, `group.Op`, `encodeOp`,
  `ParamImageMask`) and preserves append order. New exported `Zeroed()`/`Caps()`
  are read-only accessors; the counters are never reset by `Scrub`, which is
  correct for their stated purpose.
- **`Resume` does not orphan an intermediate array** for histories of 0, 1, 5, 12
  or 40 knots (measured: initial cap `len(history)+10`, final len
  `len(history)+8`, `reallocated=false` in every row), so `clear(c)` reaching only
  the final array costs nothing today.
- **Commit hygiene is correct.** `b843cf4` is the design fold alone, `768fea5` the
  register alone, and the unrelated F-106 hardware result is its own commit
  (`b226d8b`) — persist/fold/unrelated are three commits, not one.
- **Round 2's §"Explicitly checked, no finding" was taken as settled in full** and
  is not re-derived here: NC1's wrong-plate closure, restart-impossibility across
  all thirteen `Engrave` call sites, the reallocation detector's exactness, the
  aliasing argument for zero-at-reallocation, `unlockPassphraseFlow`'s bracket
  safety, and the citation spot-checks.

---

## Appendix — verification harness (reproduce verbatim)

Applied to **copies** of the gate worktree; neither real tree was modified.

```
cp -a /scratch/code/shibboleth/seedhammer-gate-orphan $SCRATCH/ver
```

`engrave/r3_verify_test.go` — knot-buffer exits and re-range identity;
`engrave/r3_resume_test.go` — `Resume` aliasing, `Resume` reallocation, and
`ClearHistory` residue; `gui/r3_resumer_test.go` — catch-up zeroing on the happy
path, the driver-error path, the once-not-per-knot property, and the never-called
probe. Run with the project toolchain:

```
nix develop /scratch/code/shibboleth/seedhammer --command go test ./engrave/ ./gui/ -run TestR3 -v
```

Output:

```
knots=3863                                   (full range)
re-range identical over 3863 knots
cap=100 knots=3863; caller buf non-zero after=0
Resume returned 17 knots, cap 22; history intact (12 knots)
history= 0  initial cap=10  final len= 8 cap=10  reallocated=false
history= 1  initial cap=11  final len= 9 cap=11  reallocated=false
history= 5  initial cap=15  final len=13 cap=15  reallocated=false
history=12  initial cap=22  final len=20 cap=22  reallocated=false
history=40  initial cap=50  final len=48 cap=50  reallocated=false
after ClearHistory: history non-zero=0/16  safePoint={X:1234 Y:5678}  progress=99  completed=0
non-zero knots left when Knot() is never called: 8 of 8
```

The four-body mutation (`$SCRATCH/mut`): delete `planEngraving`'s `defer`, empty
`ClearHistory`, delete `defer clear(c)`, delete the tail-clear, then

```
nix develop … --command go test ./gui/... ./engrave/ ./seal/
ok  seedhammer.com/gui 40.125s · gui/assets · gui/op · gui/saver · gui/text ·
ok  gui/widget · engrave 0.583s · seal 16.228s
```

— all green, confirming §Gate coverage's claim.
