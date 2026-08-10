# R0 round 2 — fold verification, DESIGN_b2b_residency_zeroing.md (F-107 / F-108)

**Reviewer:** independent architect agent (adversarial), 2026-08-10.
**Scope:** fold verification ONLY — (A) did the fold resolve each round-1 finding,
(B) did the fold introduce a new Critical or Important. Not a fresh audit.
**Artifacts:** round 1 = `design/agent-reports/2026-08-10-r0-residency-round1.md`;
folded design = `design/DESIGN_b2b_residency_zeroing.md`; fold =
`3e78295..2a70944` (design + `FOLLOWUPS.md`).
**Code read (read-only):** `/scratch/code/shibboleth/seedhammer-b2b` @ `3de8aa1`
(branch `b2b`) and the implemented funnel in
`/scratch/code/shibboleth/seedhammer-gate-orphan`.
**Measurement:** one instrumented run on a scratch **copy** of the tree
(Appendix A). No file in either real tree was modified.

**Verdict: RED — 0 Critical, 3 Important.**

The fold is a genuine sweep. All twenty round-1 verdicts land, the document no
longer argues with itself anywhere I could find, and **NC1's Critical is really
closed** — I traced the operator's Back-then-resume against the code and the
resolution holds, including the "did the wrong-plate path merely move" question,
which it did not. The threat model is honest and does support the trade. The
departure from round 1's retention design is the right call and NM1/NM2 are
legitimately dissolved rather than evaded.

The three Importants are all in the **F-108 resolution's implementability**, the
part §Gate coverage correctly flags as design-only. Two of the three buffers the
resolution names **cannot be reached from the place the design puts the zeroing**,
and the resolution's proposed shipped comment asserts a wipe-path guarantee that
the design's own F-110 says does not exist. None is a wrong-plate path; all three
are "an implementer following this text must invent, or will silently deliver
less than the register records".

---

# (A) Per-finding verdicts — all 20

## Round-0 findings (as re-verdicted by round 1)

| # | verdict | justification (design section) |
| --- | --- | --- |
| **C1** | **RESOLVED** | §"The mechanism: one funnel, and zero at the reallocation" replaces prediction with detect-after-append; both funnels are pasted in full, implemented, and mutation-killed. Round 0's two outstanding halves are both done: a test that **can see** the class exists (`gui/op/outgrown_test.go:18-48`, `:55-79` hold their own reference and read the memory rather than consulting `Zeroed()`), and §Tests row 4 states in terms that `Residue()` is *structurally blind* to the class — "stop citing it as the witness" is now written down rather than implied. |
| **I1** | **RESOLVED** | Unchanged from round 1. §"The scrub's position" puts the scrub on **both** brackets, naming `gui/unlock_kdf.go:137`. Verified the defer is still there and is ordinary flow code. |
| **I2** | **RESOLVED** | §"RE-RESOLVED after round 0" stands, and the contradiction round 1 flagged is gone: §"What R0 should attack" #2 is now struck as **WITHDRAWN** with the reason ("the premise is false and the fold withdrew it"). |
| **I3** | **RESOLVED** | §F-108 now states the real invariant with the code quoted (`gui/gui.go:544-547`, verified verbatim) and the forward-looking prohibition — "*any future derivation that captures `rec` or `m` directly breaks this and must not be added*". The laziness/re-entrancy fact round 1 wanted is recorded where it is load-bearing, in resolution item (1) ("a re-range **recomputes** the buffer"). |
| **I4** | **RESOLVED** | §"The scrub's position" keeps `:245` and explains the difference in positions; §"What R0 should attack" #4 is struck, with the offending "idempotent, so this is about clarity" sentence explicitly named as "the sentence a future fold would have read before deleting `:245`". The one-line `:245` comment is required and listed in §Gate coverage as not yet applied. |
| **I5** | **RESOLVED** | The finalizer/lifetime canary is deleted, and the deletion is recorded with its reason ("*whose stated premise was 'since there are no bytes to check'*"), so it cannot come back by accident. |
| **I6** | **RESOLVED** | §"Threat model (round 1, I6)" is new. It names the adversary, the window, three exclusions with reasons, and — the part that matters — derives the trade from it: "*memory that is merely freed and zeroed is strictly better than memory retained for a later scrub*". See §B, N-c for one tension that does not change the verdict. |
| **M1** | **RESOLVED** | §"The Drawer, and why it is not in scope here" is new, names `Release`/`maskStack`/the `src` interface-value copy, cites `gui/op/op.go:262`/`:292` and `gui/run_flow.go:264` (all three verified), and says what changed under zero-at-reallocation. |
| **M2** | **RESOLVED** | §"What this design does not cover" item 3 lists all six legacy flows with file:line, states plainly "**This design does not fix them**", and **F-112 is filed** in `FOLLOWUPS.md` with an owning phase. |
| **M3** | **RESOLVED** | §"The `ErrTooLarge` path, which the cut-end fix cannot reach" is now a first-class section, not a paragraph inside a block labelled dead. **F-111 filed.** (Incomplete in scope — see M-a below — but M3 itself is answered.) |
| **M4** | **RESOLVED** | §Tests row 1 now specifies the assertion point: "*after the session defer has run and before any later frame reuses the buffer*", with `TestWipeScrubsTheAbandonedFrameBuffer` named as the shape. One wording nit at N-b. |
| **N1** | **RESOLVED** | Unchanged from round 1. |
| **N2** | **RESOLVED** | §F-108 now cites `unlock_session.go:204` and says why ("*round 1, N2: `:195-203` is the comment, `:204` is the statement*"). Verified: `clear(rec)` is line 204. |
| **N3** | **RESOLVED** | §F-108 shows the grep and its `(no matches)` output rather than asserting "three matches are two comments and a constructor". |

## Round-1 findings

| # | verdict | justification (design section) |
| --- | --- | --- |
| **NC1** | **RESOLVED** | §"The ordering hazard — resolved by lifetime, not by one instant" splits cut state from resume state and moves items (2)/(3) off the goroutine exit. **Traced against the code and it holds:** the first Back while running takes `gui/gui.go:2722-2726` (`st.State == engraveRunning` → `Stop()`), the screen **stays**, and hold-to-resume at `:2747` restarts with `history` intact. `Engrave` returns only on a Back taken while the state is already non-running (`:2723-2724` `break frames`), at which point restart is impossible. The wrong-plate path is closed, and it did **not** move — see §"Explicitly checked" for the restart-impossibility proof over all 11 call sites. |
| **NI1** | **RESOLVED** | The funnel is structural (`cap(b.args) != cap(old)` after the append), implemented, and backed by `TestBufferGrowthIsFunnelled`. The detector is exact — verified, no false positive or negative. Round 1's "`n` disappears" is delivered. One coverage boundary at M-c. |
| **NI2** | **DISSOLVED** | Retention is gone, so "~2×" and "until the next Scrub" no longer exist as claims. The replacement is measured, not projected (35.5 KB retained vs freed, 21.2 KB current), and the F-109 collision round 1 demanded be named is now the **stated reason** for the departure. The design also corrects round 1's own Appendix A ("*the idealised doubling series predicts 3072 … and omits `refs` entirely*") — which is right; my own measurement of `cap args=3392` matches the design, not the series. |
| **NI3** | **RESOLVED** | §Gate coverage is rewritten against the design that exists, and its "What is NOT built" block says the thing round 1 said stopped a reviewer asking: "**F-108's zeroing is unimplemented … This is the part the Critical was in, and it is the part a reviewer must read as design rather than as verified code.**" That sentence is what put me on the F-108 resolution, which is where all three of my Importants are. |
| **NM1** | **DISSOLVED — legitimately** | There is no orphan list, so there is nothing to truncate-vs-clear. Not evasion: the design states the dissolution explicitly, names it as a dissolution rather than an answer, and adds "*If a reviewer reinstates retention, both findings come back with it*" — which is exactly the treatment that keeps a dissolved finding from being lost. |
| **NM2** | **DISSOLVED — legitimately** | Same, and strictly stronger than answering it: an outgrown `refs` array is `clear`ed at the reallocation (`gui/op/op.go:78-84`), so its referents drop **immediately**, rather than at a `Scrub` that on the six legacy flows never comes. Verified in the implementation and pinned by `TestAppendZeroesTheRefsArrayItOutgrows`. |
| **NM3** | **RESOLVED** | The `gui.go:2651-2656` anchor is replaced by `:2715`/`:2726`/`:2747` in the design **and** in `FOLLOWUPS.md` (the fold diff corrects the register entry's copy too), and the design requires the shipped `unlock_session.go:200` comment to be corrected in the same change. |
| **NM4** | **RESOLVED** | No "Superseded framing" block survives; `grep -i superseded` over the design returns nothing. M3's substance is promoted into its own section. |
| **NN1** | **RESOLVED** | `FOLLOWUPS.md:1759-1789` is rewritten in the fold commit: the withdrawn framing is struck, the RE-SCOPING is itself withdrawn, and the bad `:2651-2656` anchor is corrected in place. |
| **NN2** | **RESOLVED** | Both funnels are pasted in full. `Scrub`/`Residue` are not pasted, which is fine — they are unchanged from `b2b` apart from a comment, and the design says so ("*`Scrub` still zeroes the current arrays*"). |

**Counts:** RESOLVED 17 · DISSOLVED 3 · PARTIAL 0 · NOT ADDRESSED 0 · REGRESSED 0.

Round 1's central complaint — "the fold rewrote two sections and left four
arguing with them" — is **fully answered**. I looked for a surviving
contradiction in §Tests, §Gate coverage, §What R0 should attack and §Follow-ups
and found none.

---

# (B) New findings

## CRITICAL (0)

None. I specifically hunted the failure mode round 1 found — a wrong-plate path
created rather than removed by the fix — and could not produce one. The trace and
the restart-impossibility proof are in §"Explicitly checked, no finding".

## IMPORTANT (3)

### I-A — item (2) cannot reach `splineResumer.catchup`: the job holds no handle on the resumer, and the field is nilled at the first resumed knot

**Anchor:** §"The resolution, split three ways" item 2 — "*with
`releaseResumeState` zeroing `e.safePoint.history` and **the resumer's `catchup`**
when `e.status.State` is terminal*" — and the three-buffer table row
"`splineResumer.catchup` | `gui/engraver.go:222` | never zeroed".

**Two independent reasons it reaches nothing.**

1. **No handle.** `splineResumer` is constructed at `gui/engraver.go:168` as
   `res := newSplineResumer(drv, e.catchup())` — a **local inside
   `runEngraving`**. `engraveJob` (`gui/engraver.go:15-30`) has fields `pl,
   spline, conf, opts, quit, errs, progress, lock, status, nknots, safePoint`
   and nothing else. `newSplineResumer` has exactly one non-test caller
   (measured). A method on the job cannot name the resumer.
2. **The reference is dropped before any terminal state.**
   `gui/engraver.go:226-228`:

   ```go
   func (s *splineResumer) Knot(k bspline.Knot) (completed uint, cerr error) {
       if c := s.catchup; c != nil {
           s.catchup = nil
   ```

   The field is nilled on the **first knot of the resumed run**, so by the time
   `Status()` ever reports `engraveStopped`/`engraveDone`/`engraveFailed`, the
   array is already unreachable garbage. Even with a handle, `releaseResumeState`
   would zero a nil slice.

**Failure scenario.** Operator Backs mid-cut, holds to resume, lets the plate
finish, Backs out. `releaseResumeState` fires on `engraveDone` and zeroes
`e.safePoint.history` — and reaches nothing at all for `catchup`. Each restart
allocated a fresh array in `SafePointer.Resume` (`engrave/engrave.go:1643`
`make([]bspline.Knot, 0, len(s.history)+10)`) holding the move-to-safe-point line
**plus a full copy of the history knots** — seed-derived geometry, by this
design's own classification — and every one of them is unreachable and unzeroed
for the machine's uptime. The design records `catchup` as closed by item (2).

**Smallest correct fix.** Move the clear to where the buffer is dead by
construction, the same shape as item (3): `clear(c)` in `splineResumer.Knot`
immediately **after** the fast-forward loop and before `Knot` returns. That needs
no ordering argument on any path, reaches every restart's array rather than the
last, and lets `catchup` come out of `releaseResumeState` entirely — which leaves
item (2) owning exactly one buffer and makes its terminal-only guard simpler to
justify.

### I-B — item (1) has no seam: nothing outside package `engrave` can reach `knotBuf`

**Anchor:** §"The resolution, split three ways" item 1 — "*`knotBuf` — zero
**inside the engrave goroutine**, before the send*" — resting on
§"RE-RESOLVED"'s "*`planEngraving(knotBuf, conf, e)` already exists as a
caller-supplies-the-buffer seam, with a doc comment saying so*".

That sentence is true **only inside package `engrave`**, and the design uses it
to justify a change in package `gui`. Measured:

- `engrave/engrave.go:1016-1021` — `PlanEngraving` allocates `knotBuf` itself and
  returns only `bspline.Curve`, a closure. Nothing else escapes.
- `engrave/engrave.go:1025` — `func planEngraving(` is **unexported**. Its only
  non-test callers are `PlanEngraving` itself; every external construction goes
  through `engrave.PlanEngraving` (`gui/gui.go:2988`, `gui/qa.go:19`,
  `cmd/controller/engraver.go:196,211`, `cmd/glyphtrace`, `engrave.go:1232`).
- `Plate` (`gui/gui.go:2994-2998`) carries `Duration`, `Spline`, `Conf`. No
  buffer. `newEngraverJob` (`gui/engraver.go:64`) copies `plate.Spline` into
  `e.spline` and nothing more.

So the engrave goroutine (`gui/engraver.go:109-112`) has no expression that names
the array item (1) tells it to zero.

**Failure scenario.** An implementer reaches item (1), finds no handle, and takes
one of two bad branches: (a) invents an exported API change in `engrave` — a
shared funds-path package — that this gate never reviewed and that the design
does not scope; or (b) quietly drops item (1), leaving F-108's **central** buffer
unzeroed while the design, `FOLLOWUPS.md`'s rewritten F-108 entry ("*a real
defect with a real patch*") and §10.2.2's inventory all record it as fixed. That
is the shape this whole document exists to prevent.

**Smallest correct fix — name the seam, and the cheapest one needs no API change
at all.** Put the clear inside the closure, in `planEngraving`:

```go
return func(yield func(bspline.Knot) bool) {
    var ts timeScaler
    start := bspline.Knot{}
    spline := knotBuf[:0]
    defer clear(spline[:cap(spline)])
```

It fires at the end of **every** range on **every** exit — including the
`!yield` early return and including `bspline.Measure`'s build-time pass at
`gui/gui.go:2989`. Because `spline := knotBuf[:0]` reopens the buffer on the next
range, the design's own safety argument for item (1) carries over unchanged. It
also dissolves F-111 and M-a below, which the goroutine-exit placement
structurally cannot reach. If the goroutine-exit placement is kept for a reason,
the design must state **which exported seam is added to `engrave`** and re-cost
it, because that is an API change to shared funds-path code.

### I-C — the proposed shipped comment says the wipe covers resume state; the design's own F-110 says it does not

**Anchor:** §"The resolution, split three ways" item 2's code block, which the
design presents as text to paste into `gui/gui.go`:

> *Terminal-only: … If the job is still running -- Engrave returning because
> ctx.Done, i.e. the wipe -- skip it and **let the wipe do its work**, rather
> than race the goroutine.*

**The wipe does no such work.** The unwind is `gui/run_flow.go:233-264` and it is
exactly two calls: `ctx.B.Scrub()` (`:245`) and `d.Release()` (`:264`). Nothing
in the tree touches `SafePointer.history`, and the design proves this itself,
two sections earlier, with the grep it quotes:

```
$ grep -rn "clear(" --include='*.go' gui/ | grep -v _test.go | grep -i "spline\|plate\|knot"
(no matches)
```

The design is honest about the gap fifteen lines below, in §"Residual, named
rather than hidden", and files it as F-110 gap 1. But the residual paragraph
stays in `mnemonic-engrave`; **the comment ships in the firmware.**

**Failure scenario.** F-110 is scheduled to B2b. Someone triaging it — or a later
audit of §10.2.2's wipe-by-any-route guarantee — greps the wipe path, finds a
comment stating that the wipe handles resume state, and closes or de-prioritises
it. Seed-derived geometry then stays resident across a §10.2.4 idle wipe, which
is the *one* window this design's own threat model says the adversary operates
in ("*the operator walks away, and the machine must not still hold the seed*").
This is verbatim the class the document's second sentence names: **a copy of the
seed that a comment says is bounded, with nothing in the code enforcing the
bound.**

**"Rather than race the goroutine" is also a false dichotomy.** Once `Engrave`
has returned the screen is gone, so the resume state is provably dead — the
design proves exactly this for the terminal case. The non-terminal case needs no
race and no wait: `releaseResumeState` can set a flag the goroutine's own exit
path checks, so the goroutine zeroes `e.safePoint` after its last read. That
closes the wipe hole instead of deferring it.

**Smallest correct fix.** Either close it with the goroutine-exit handoff, or
change the comment to say what is true — *the wipe does not reach this either;
see F-110* — and cross-reference F-110 from the comment so a future grep lands on
the open item rather than on a false assurance.

## MINOR (3)

### M-a — the cut-end fix misses more than `ErrTooLarge`: an engrave screen Backed out of before the cut starts

§"The `ErrTooLarge` path" and F-111 name one path,
`gui/unlock_session.go:191-193`. But `toPlate` fills `knotBuf` at build time for
**every** plate (`gui/gui.go:2988-2989`), and `EngraveScreen.Engrave` returns on
the first Back while the state is still `engraveIdle`
(`gui/gui.go:2721-2725`: `st.State != engraveRunning` → `break frames`). On that
path there is no goroutine and no send, so item (1) cannot fire; and
`releaseResumeState`'s terminal-only guard also skips `engraveIdle`. "Insert a
blank plate… hold button to start" then Back is an ordinary operator action, not
an error path.

Measured residue is ~10 non-zero knots of the final stroke (Appendix A; round 0
measured 9 and graded that magnitude not seed-recoverable), which is why this is
Minor. But F-111 as **filed** will be implemented as an `ErrTooLarge`-only patch
and leave the ordinary path open. Fix: widen F-111 to "every path where the plate
is built and no cut completes", or adopt I-B's in-closure `defer clear`, which
covers all of them for free.

### M-b — no proposed test pins items (2) or (3); deleting `releaseResumeState` entirely leaves every row green

§Tests rows 5 and 6 cover NC1's regression and item (1). Nothing covers items (2)
or (3):

- **Row 5 asserts the catch-up motion is *unchanged*.** It fails against the
  *previous* draft, which is what it is for — but it passes identically whether
  `releaseResumeState` zeroes anything or is a no-op.
- **Nothing asserts `SafePointer.history` IS zeroed** at `Engrave`'s terminal
  return.
- **Nothing asserts the `history` tail-clear** at
  `engrave/engrave.go:1675-1676`, which the design calls "free, and always safe"
  and says "should land regardless of the rest".

§Gate coverage says "Test rows 1, 5 and 6 are not written" without noticing that
rows for (2) and (3) do not **exist**. On a project whose standing rule is that a
green suite proves little until a mutation kills it, two thirds of F-108's
resolution has no mutation row. Fix: add two rows with their mutation lines —
(a) drive to `engraveStopped`, exit via Back, assert `history[:cap]` all-zero,
mutant = empty `releaseResumeState`; (b) after a trim, assert
`history[rem:cap]` all-zero, mutant = delete the tail-clear.

### M-c — `TestBufferGrowthIsFunnelled` lints one file out of four, and the design does not state the boundary

`gui/op/funnel_lint_test.go:24` hard-codes `const src = "op.go"`. But `b.args`
and `b.refs` are package-visible: `buffer_len.go`, `draw.go` and `image.go` can
append to them and the lint would not see it. The pattern also cannot see a
local-alias form (`a := b.args; a = append(a, x); b.args = a`).

Measured today, nothing else appends —
`grep -rn '\.args = append(\|\.refs = append(' --include="*.go" .` over the
gate-orphan tree returns only the two funnel bodies (plus the lint's own string
literals) — so this is future-proofing, not a live hole. But §Tests row 3 sells
the lint as the structural guarantee ("*a lint, because no behavioural test can
catch this*") without stating what it does not cover, and this project's own rule
is that a gate hiding its blind spot is worse than no gate. Fix: walk the package
directory instead of one filename, and say in the test comment what the pattern
cannot see.

## NIT (3)

- **N-a.** "*a terminal state IS the receive on `e.errs`, so the goroutine has
  **provably returned***" overstates it. `errs` is buffered with cap 1
  (`gui/engraver.go:100`) and the body is `defer e.pl.Wakeup(); errs <-
  e.runEngraving(...)` (`:109-112`), so after the receive the goroutine may still
  be in `Wakeup()`. **The safety conclusion is unaffected** — the send is the
  happens-before edge and `runEngraving`'s defers (including `d.Close()` at
  `:160-165`) complete before the value is sent — but the sentence as written
  licenses a future author to add work after the send. Say "`runEngraving` has
  provably returned".
- **N-b.** §Tests row 1 calls the normal-exit Context "*the **abandoned**
  Context*", which §F-107 spends a paragraph proving it is not ("*the flow
  walking back to the start screen **with the same `Context` and the same
  `op.Buffer`***"). Only the harness abandons it, by bounding the flow at the
  session return. Round 0's M4 asked the design to "say which"; say that the flow
  is bounded so the session return ends the run, rather than borrowing the wipe
  test's word for it.
- **N-c.** The threat model's in-scope read primitive is unstated and is in mild
  tension with its own exclusion: in scope is an attacker "*able to run code on
  it or read its RAM*", out of scope is "*a compromised firmware image — signed
  boot is the control*". Signed boot is largely the control against running code
  too, so the in-scope adversary needs a named primitive (a debug port, a
  firmware defect, a service path) or the exclusion needs narrowing. **The trade
  survives either way** — zeroing is strictly better under every candidate
  adversary — which is why this is a Nit and I6 is RESOLVED.

---

# What must be true to reach GREEN

1. **I-A** — `splineResumer.catchup` given a placement that can actually reach
   it. Recommended: `clear(c)` inside `splineResumer.Knot` after the fast-forward
   loop, and `catchup` removed from `releaseResumeState`.
2. **I-B** — the seam by which `knotBuf` is zeroed named explicitly.
   Recommended: `defer clear(spline[:cap(spline)])` inside `planEngraving`'s
   closure, which needs no API change and subsumes F-111 and M-a. If the
   goroutine-exit placement is kept, state the exported `engrave` API being added
   and re-cost it.
3. **I-C** — the "let the wipe do its work" comment corrected to state that the
   wipe does not reach resume state either, with F-110 cross-referenced from the
   comment; or the wipe hole closed via a goroutine-exit handoff.

Minors M-a/M-b/M-c and Nits N-a/N-b/N-c are recorded and do not gate. M-b in
particular is worth folding in the same pass, since it is two test rows and it is
what would catch a botched I-A or I-B at implementation time instead of at the
next review.

**Re-review scope for round 3:** did the fold fix I-A, I-B and I-C, and did it
introduce a new defect. Everything below is settled — do not re-derive it.

---

# Explicitly checked, no finding

Recorded so round 3 does not re-derive them.

- **NC1's wrong-plate path is genuinely closed, and it did not move.** Traced:
  first Back while running → `gui/gui.go:2722` `s.job.Status()` reports
  `engraveRunning` → `:2726` `Stop()` → `engraveStopping`; the screen **stays**
  and `Engrave` does not return; the operator holds select → `:2747`
  `s.job.Start()` restarts with `history` intact, because `releaseResumeState`
  never ran. `Engrave` returns only on a Back taken while the state is already
  non-running (`:2723-2724` `break frames`). The terminal-only guard is therefore
  the right guard, and it also correctly skips the double-Back case that returns
  in `engraveStopping` with the goroutine still live — a case `gui/wipe_guard.go`
  already documents as reachable.
- **"Restart is impossible once `Engrave` returns" holds for the whole tree.**
  Every non-test `Engrave` call site constructs a **fresh** `EngraveScreen`, and
  therefore a fresh `engraveJob` with a fresh `SafePointer`, per call:
  `gui/passphrase_flow.go:661`, `gui/bundle_flow.go:351`,
  `gui/derive_xpub.go:286`, `gui/bip85.go:337`, `gui/freetext_flow.go:1569`,
  `gui/unlock_platelist.go:233`, `gui/gui.go:2188`, `:2235`, `:2252`, `:2268`,
  `gui/slip39_polish.go:507`, `gui/unlock_session.go:206-211` and `:315-320`.
  None reuses a screen across calls, including the `for { if …Engrave { return } }`
  loops. Note for the record that this is a property of the **call sites**, not
  of `Engrave` — a future flow that hoists `NewEngraveScreen` out of its loop
  would reintroduce NC1. Worth one sentence in the design; not filed as a finding
  because no such site exists.
- **`engraveJob` registered on the wipe guard cannot restart after `Engrave`
  returns.** `gui/unlock_session.go:207-210` sets `g.job` and defers `g.job =
  nil`; `wipeGuard.armed()` calls `Status()`, whose `e.Start()` at
  `gui/engraver.go:148` is inert (confirmed independently: `Start()` early-returns
  while `errs != nil`, and the only assignment of `engraveRunning` is `Start()`
  itself at `:108`); and the window between `Engrave`'s return and `g.job = nil`
  is frame-free straight-line code.
- **The reallocation detector is exact.** `cap(b.args) != cap(old) && cap(old) > 0`
  has no false positive and no false negative: Go's `append` replaces the backing
  array only when it increases `cap`, never returns a different array at equal
  `cap`, and returns the identical slice for `len(vals) == 0`. The `cap(old) > 0`
  guard correctly skips the nil-to-first-array case, where there is nothing to
  zero.
- **`append` cannot be handed a slice that aliases the Buffer.** Every `refs`
  argument reaching `encodeOp`/`ParamImageMask` inside package `op` is a fresh
  `[]any{…}` literal (`gui/op/op.go:125,132,139,147,156,170,186,212`) and every
  `args` argument is literal `uint32`s. So zeroing the outgrown array can never
  zero a live source.
- **Only `op.go` and `buffer_len.go` touch `b.args`/`b.refs` at all**; `draw.go`
  and `image.go` do not. (This is what makes M-c a future-proofing point rather
  than a live hole.)
- **The outgrown-array class does NOT apply to `knotBuf`.** I instrumented
  `planEngraving` on a scratch copy (Appendix A): base cap 100, **max cap seen
  100, zero reallocations** over 3863 yielded knots. `computeSCurve` returns a
  fixed `[7]scurvePhase` (`engrave/engrave.go:838`), so `appendLine`'s per-call
  knot count is bounded at 7 and `spline` cannot outgrow a 100-entry buffer on
  the line path. Zeroing `knotBuf[:cap]` therefore reaches everything — no funnel
  is needed there. (Contrast `SafePointer.history`, which the design correctly
  identifies as carrying the class, in F-110 gap 2.)
- **Item (1) has no concurrent reader.** `bspline.Measure`'s only firmware call
  site is `gui/gui.go:2989`, at build time; `EngraveScreen.draw` (`:2832`) and
  `drawNav` (`:2883`) read `s.job.Status()` and `s.duration` and never range the
  spline. Two engrave goroutines cannot coexist (`Start()` is gated on
  `errs != nil`).
- **§What R0 should attack #3 — `unlockPassphraseFlow`'s bracket is SAFE.** The
  fold left this open for R0; answering it: `gui/unlock_kdf.go:137`'s defer is
  ordinary flow code, i.e. not lexically inside a `ctx.Frame` call, so round 0's
  §(a) argument applies unchanged — `ctx.B` has `len 0` and holds only
  already-drawn content when it fires. No op built into `ctx.B` can be pending a
  draw there. **Close this item.**
- **Every citation spot-checked for SEMANTICS, not just resolution**, and all say
  what the design says they say: `gui/gui.go:1612` `for !ctx.Done`;
  `cmd/controller/main.go:34` `for range gui.Run(p, ver) {}`;
  `bip39/bip39.go:79-87` `LabelFor` returns `words[start:end]` of the static
  wordlist (so §"does not cover" item 2's "selection, not copied plaintext" is
  right); `gui/gui.go:544-547` `words := make([]string, len(m))`;
  `gui/gui.go:2988-2989`; `gui/op/op.go:49-52` `ops{start,end,refs}`, `:262`
  `clear(d.maskStack[:cap…])`, `:292` `Release`, `:307-308` the `args`/`refs`
  snapshot, `:614-618` `imageOp{src, refs, args}`, `:470-473` `Buffer.Reset`
  quoted verbatim; `gui/run_flow.go:245`, `:264`; `gui/engraver.go:96-100`,
  `:131-144`, `:168`, `:222`; `engrave/engrave.go:1029`, `:1146`, `:1642-1648`,
  `:1675-1676`, `:1683`; `gui/unlock_session.go:191-193`, `:204`, `:239`, `:276`;
  `gui/unlock_kdf.go:137`.
- **The three implemented tests are honest about what they do and do not pin.**
  `gui/op/outgrown_test.go:18-48` and `:55-79` hold their own reference and read
  the memory rather than consulting `Zeroed()` for the verdict;
  `gui/orphan_measure_test.go:64-95` says in its own comment that it does **not**
  verify the zeroing and fails `INCONCLUSIVE` if the frame never reallocated.
  §Tests rows 2/3/4 describe them accurately, including row 4's admission that it
  still passes with `clear` deleted.
- **The aliasing argument for zero-at-reallocation holds.** Aliases into
  `b.args`/`b.refs` are created only inside `Drawer.draw` (`oargs`, `rargs`,
  `imageOp.args/refs` at `gui/op/op.go:355`), all consumed within the same
  traversal; `inputOp.tag` is an interface-value **copy**, not an alias, so
  `d.inputs` survives a scrub; stale `frameOp`s in `d.maskStack` are cleared to
  cap at the next `Draw` entry (`:262`) before anything is read. Nothing appends
  during a draw. The design's four bullets are correct and complete as far as I
  could push them.
- **The threat model supports the trade it is used for.** "Every buffer that
  persists past the session is in scope regardless of how briefly it was written"
  supports the funnel; "memory freed and zeroed is strictly better than memory
  retained" supports the departure from retention; and the F-107 defect (read the
  words, press Back, machine sits on the start screen holding them) falls squarely
  inside the in-scope window rather than in either exclusion. Excluding cold-boot
  / SRAM remanence is defensible **as written**, because the exclusion is scoped
  to a residual the design cannot bound and says so, rather than waved away.

---

# Appendix A — the `knotBuf` reallocation measurement (reproduce verbatim)

Applied to a **copy** of the tree; neither real tree was modified.

```
cp -a /scratch/code/shibboleth/seedhammer-b2b $SCRATCH/shb2b
```

`engrave/probe_scratch.go` records the backing-array pointer of `knotBuf` at the
top of `planEngraving`'s closure and compares it against `spline`'s at every
yield batch; `engrave/engrave.go` gains two probe calls and nothing else.

```go
func TestProbeKnotBufReallocation(t *testing.T) {
	s := String(constant.Font, 40*mm/10, "ABANDON ABILITY ABLE ABOUT ABOVE ABSENT "+
		"ABSORB ABSTRACT ABSURD ABUSE ACCESS ACCIDENT")
	e := Engraving(s.Engrave)
	buf := make([]bspline.Knot, 0, 100)
	spline := planEngraving(buf, conf, e)
	n := 0
	for range spline {
		n++
	}
	...
}
```

Output (go1.26.3 via `nix develop`):

```
=== RUN   TestProbeKnotBufReallocation
    knots yielded: 3863
    base cap=100  maxCap seen=100  reallocation-observations=0  distinct orphan arrays=0
    non-zero entries left in the CALLER's knotBuf after the cut: 10 of 100
--- PASS
```

Two conclusions used above: `knotBuf` does **not** carry the outgrown-array class
(so item (1)'s `clear(knotBuf[:cap])` is complete), and the residue on a
never-cut plate is the same ~10 knots round 0 measured — the magnitude behind
M-a's Minor grading.
