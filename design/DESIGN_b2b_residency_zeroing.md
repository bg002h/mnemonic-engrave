# Residency zeroing — the rendered seed and the cut plate

**Status:** design, folded through R0 round 1. Written 2026-08-10.
**Owns:** F-107 (`ctx.B` scrubbed only on the wipe path) and F-108 (`plate.Spline`
never zeroed after the cut). Both B2b.

Two defects, one shape: **a copy of the seed that a comment says is bounded, with
nothing in the code enforcing the bound.** They are designed together because
they share a hazard — zeroing a buffer that is still in use — and because fixing
one and not the other leaves §10.2.2's wipe-by-any-route guarantee just as false.

**Round 1 changed two things materially.** The Critical it found was in the fold's
own ordering resolution, not in the original filing, and it is fixed here by
splitting two lifetimes that the previous draft treated as one. And the C1
mechanism has been replaced: round 1 proposed a structural funnel that *retains*
outgrown arrays for a later `Scrub`; measurement showed the retention costs
35.5 KB per 24-word frame and collides with F-109's open ~35 K, so the funnel
stayed and the retention went — the array is zeroed **at** the reallocation.

---

## Threat model (round 1, I6)

Stated because the round-0 question "is documenting a residual good enough for a
funds path" is unanswerable without it.

**In scope.** An attacker with **later physical possession** of the machine, able
to run code on it or read its RAM — a stolen or borrowed SeedHammer II, a device
sent for service, a machine left unattended in a shared workshop. The asset is
the BIP-39 seed of a wallet whose plate was engraved on it. §10.2.4's idle wipe
exists for exactly this adversary: the operator walks away, and the machine must
not still hold the seed.

**Out of scope, and why.**

- **A live attacker during the session** — they can read the plate off the screen
  and the metal. Nothing memory-side helps.
- **Cold-boot / RAM remanence after power-off.** SRAM decay on the RP2350 is
  unmeasured here, and no zeroing this design proposes runs after power is cut.
  A residual we cannot bound is honest to exclude, not to hand-wave.
- **A compromised firmware image.** Signed boot is the control; a firmware that
  wants the seed does not need our garbage.

**What follows for the trade.** The adversary reads memory at some point *after*
the session ends. So the property that matters is **"no seed-derived bytes remain
resident once the session is over"**, and every buffer that persists past the
session is in scope regardless of how briefly it was written. It also means
retention is not a neutral cost: memory held is memory that must be zeroed, and
memory that is merely freed *and zeroed* is strictly better than memory retained
for a later scrub.

---

## F-107 — the rendered seed survives a normal exit

`ctx.B.Scrub()` has exactly one call site, measured:

```
$ grep -rn "\.Scrub()" --include="*.go" . | grep -v _test.go
gui/run_flow.go:245:			ctx.B.Scrub()
```

and it sits inside `if !wiping { return }` — the §10.2.4 wipe path only.

**The other branch is unreachable in production.** `uiFlow` loops `for !ctx.Done`
(`gui/gui.go:1612`); on the device `ctx.Done` is set by the wipe alone, because the
`!yield()` route needs the consumer to stop ranging and
`cmd/controller/main.go:34` is `for range gui.Run(p, ver) {}`. So `runWithFlow`
never returns on hardware, and a UI-level "normal exit" is the flow walking back
to the start screen **with the same `Context` and the same `op.Buffer`**.

`Buffer.Reset()` runs per frame and does not zero:

```go
func (b *Buffer) Reset() {
	b.args = b.args[:0]     // TRUNCATE
	clear(b.refs)
	b.refs = b.refs[:0]
}
```

`op.Glyph` encodes every rendered rune into `args`, so — in `Scrub`'s own words —
*"on the SeedScreen path the twelve words come back VERBATIM AND IN ORDER from
the backing array."* On a normal exit nothing zeroes them; they persist until
later frames overwrite those indices, and a start screen writes far fewer args
than a seed screen, so the later words survive longest.

**The exposed path is the common one.** Read your words, press back. The
protected path — walk away for 3:30 — is the rare one.

### C1 (round 0) — the scrub cannot reach what the buffer outgrew

**The round-0 Critical is not a flaw in the proposed fix; it is a defect in code
B2b already shipped.**

`op.Buffer` has no pre-sizing, so `args` grows from nil, and `Scrub` zeroes
`b.args[:cap(b.args)]` — **the current array only**. Every reallocation orphans an
array still holding every rune written before it. The round-0 reviewer
instrumented it and pulled the words back out of a 24-word seed frame:

```
seed frame len=2387 cap=3392 | reallocated=true
Residue() after Scrub = (0 args, 0 refs)
orphan text: "1: ABAN$D2O?NO5<2: ABILI(T-Y85Q3: ABLE#5f4: ABOU(T65…"
```

Thirteen words, verbatim and in index order, in an array §10.2.2 declares wiped,
scored 0 by `Residue()`. **`run_flow.go:245`'s existing wipe-path `Scrub` has the
same hole**, so moving the call was never going to deliver the property — it
would have made the normal exit equal to the wipe exit, an equality of two
incomplete wipes.

#### The mechanism: one funnel, and zero at the reallocation

Two changes, both in `gui/op/op.go`. Applied to a scratch worktree and measured;
the numbers below are outputs, not projections.

**(1) All growth goes through one pair of functions.** Round 1's NI1 was right
that a predict-ahead recorder (`if cap-len < n`) is fragile: it needs a per-site
element count, and the two sites carrying the glyph args each append **twice** —
payload, then a header word — so the natural `n = len(args)` lets the first
append fit exactly and the *second* reallocate unrecorded, orphaning the array
that just received the runes. Detecting reallocation **after** the append needs
no count and cannot be got wrong per-site:

```go
func (b *Buffer) appendArgs(vals ...uint32) {
	old := b.args
	b.args = append(b.args, vals...)
	if cap(b.args) != cap(old) && cap(old) > 0 {
		b.zeroedArrays++
		b.zeroedEntries += cap(old)
		clear(old[:cap(old)])
	}
}

func (b *Buffer) appendRefs(vals ...any) {
	old := b.refs
	b.refs = append(b.refs, vals...)
	if cap(b.refs) != cap(old) && cap(old) > 0 {
		b.zeroedArrays++
		b.zeroedEntries += cap(old)
		clear(old[:cap(old)])
	}
}
```

**Measured growth surface: 9 `args` appends and 3 `refs` appends, all in
`gui/op/op.go`, none anywhere else in the tree** (`grep -rn '\.args = append\|
\.refs = append'` returns nothing outside those two functions after the change).

**(2) Zero the outgrown array instead of retaining it.** This is where the design
departs from round 1's proposed fix, deliberately. NI1's funnel recorded each
outgrown array in an `orphanArgs`/`orphanRefs` list for a later `Scrub`. That
works, and it was prototyped and green — but it is the wrong trade, for a reason
that only appeared once the retention was measured on a real frame rather than
projected from Go's growth series:

| | orphaned/zeroed | current | total retained |
| --- | --- | --- | --- |
| retain for later `Scrub` | 35.5 KB | 21.2 KB | **56.8 KB** |
| zero at reallocation | 35.5 KB, **freed** | 21.2 KB | **21.2 KB** |

Against 283 K free that is the difference between ~20% and ~7.5%. And the
retained figure is **35.5 KB**, which is within noise of **F-109's open,
unidentified ~35 K of reachable post-wipe objects** — adopting retention would
make F-109's closing measurement uninterpretable, in the one investigation the
operator has ruled cannot be closed as a memory nit.

Zeroing at the reallocation is safe because the outgrown array is dead the
instant `append` returns:

- `append` has already copied the live elements into the new array — verified by
  `TestOutgrowingPreservesTheLiveContents`, which rebuilds 4096 values across
  many reallocations and compares every one.
- **No `op` aliases a backing array.** `op` carries `ops{start, end, refs}` —
  plain indices (`gui/op/op.go:49-52`).
- **Nothing appends during a draw.** `Drawer.draw` snapshots `args := buf.args` /
  `refs := buf.refs` at entry (`gui/op/op.go:307-308`), and there is no `appendArgs` or
  `appendRefs` call site anywhere below it. `imageOp` *does* hold `args []uint32`
  and `refs []any` slices that alias the current arrays (`gui/op/op.go:614-618`), but
  they are built during a draw, and no reallocation can happen during one.
- Stale `frameOp`s from the *previous* frame are dropped before any drawing:
  `Draw` clears `maskStack` to cap at entry (`gui/op/op.go:262`).

**Cost, measured:** `+528 bytes` flash, `+0` RAM (§Gate coverage). RAM is
unchanged precisely because nothing is retained.

**Two of round 1's Minors are dissolved rather than answered, and that is worth
stating plainly so it is not mistaken for an unswept finding.** NM1 ("`Scrub`
must `clear` the orphan list's backing array, not merely drop it") and NM2
("orphaned `refs` arrays pin their referents") are both properties of the
*retention* this design no longer does: there is no orphan list to truncate, and
an outgrown `refs` array is zeroed at the reallocation, so its referents drop
immediately rather than at a `Scrub` that on the legacy flows never comes. If a
reviewer reinstates retention, both findings come back with it.

**What this does NOT reach.** `Scrub` still zeroes the current arrays, and the
per-frame `Reset` still only truncates `args` — that is unchanged and correct,
because a reused buffer overwrites those indices. The outgrown-array class is
closed; the current-array class is `Scrub`'s job and is F-107's other half.

### The scrub's position — F-107's original fix, still needed

Round 0 confirmed the placement is **safe** (§(a)): `unlockSecretSession`'s defer
runs strictly between frames, so no op still to be drawn is zeroed. It is
necessary but not sufficient — without the zeroing above it scrubs one array out
of the twenty-two a 24-word frame touches.

Round 0 also found the placement **insufficient in scope** (I1): §8's twelve-word
passphrase is rendered by `unlockPassphraseFlow`, **outside** this bracket, and on
the give-up routes nothing scrubs at all. So the scrub belongs on **both**
brackets — the passphrase flow's own defer (`gui/unlock_kdf.go:137`, which Task 9
already installs for the wipe guard) and the secret session's.

And I4: **`run_flow.go:245` is not subsumed and stays.** It covers the wipe path,
where the Context is abandoned rather than returned through, and the two
positions are different: `:245` fires on a Context nobody will reuse, the session
defers fire on one that will. A one-line comment at `:245` should say so, or a
future reader will delete it as duplicated.

### The Drawer, and why it is not in scope here (round 0, M1)

`op.Drawer` holds `maskStack []frameOp`, whose stale entries carry `imageOp`s
whose `src` is an **interface-value copy living in the Drawer**, not in the
Buffer — so `Buffer.Scrub` cannot reach it. That is a separate defect and it is
already fixed in b2b `3de8aa1`: `Drawer.Release()` (`gui/op/op.go:292`) clears
`maskStack` and `inputs` to cap, `Draw` clears `maskStack` to cap on every frame,
and production calls `Release` from `gui/run_flow.go:264`.

It is named here rather than omitted because this design changes the lifetime of
what those stale headers point at, and a reader is entitled to know it was
checked: with zero-at-reallocation the aliased array is **zeroed** before the
stale header is dropped, which is strictly better than before and cannot be worse.

---

## F-108 — the plate's geometry survives the cut

`unlock_session.go:239` states the limitation and its bound:

> `LIVE    plate.Spline, for the duration of the cut. It IS the seed rendered as
> geometry and must exist while the needle moves. F-83, accepted.`

**"For the duration of the cut" is the justification; nothing ends it.** There is
no `clear` of a plate or spline anywhere in `gui/`. Measured (round 1, N3 — the
command, not the claim):

```
$ grep -rn "clear(" --include='*.go' gui/ | grep -v _test.go | grep -i "spline\|plate\|knot"
(no matches)
```

When `scr.Engrave(...)` returns, `plate` goes out of scope, becomes garbage, and
is never zeroed. TinyGo's `-gc precise` is a non-moving mark-sweep and does not
zero on free.

The same function already demonstrates the correct pattern for the *record*, and
explains why the timing matters — `clear(rec)` at **`unlock_session.go:204`**
(round 1, N2: `:195-203` is the comment, `:204` is the statement) runs **before**
`Engrave`, precisely so the seed is not resident for the ~21-minute cut. That
reasoning was never carried to the geometry, which is the other copy of the same
secret.

**The invariant that makes `clear(rec)` sound, which the previous draft never
stated (round 0, I3).** It is *not* sound because the geometry is computed
lazily-but-early; it is sound because **`engraveSeed` materialises an independent
copy before the plate is built**:

```go
words := make([]string, len(m))          // gui/gui.go:544-547
for i, w := range m {
	words[i] = bip39.LabelFor(w)
}
```

The `Engraving` closes over `words`/`seedDesc`, never over `rec` or `m`. So
`clear(rec)` cannot affect the plate — and equally, **any future derivation that
captures `rec` or `m` directly breaks this and must not be added**. The same fact
is what makes the knot-buffer zeroing safe on a restart, below; they are one
invariant, not two.

### RE-RESOLVED after round 0 — I WAS WRONG TWICE, and the buffer IS clearable

My first filing said "zero the spline after the cut". I then re-scoped it to
"impossible, because `Curve = iter.Seq[Knot]` is a closure". **Round 0's I2 shows
the second answer was as wrong as the first**, and for a worse reason: I stopped
at the type and never looked at what the closure closes over.

`Curve = iter.Seq[Knot]` is true and **irrelevant**. A closure over an
already-materialised slice is still a clearable slice, and `engrave.PlanEngraving`
builds exactly that. Measured by the round-0 reviewer: **9 non-zero knots left in
`knotBuf` after a full cut; `clear(buf[:cap(buf)])` drives it to 0.**
`planEngraving(knotBuf, conf, e)` already exists as a caller-supplies-the-buffer
seam, with a doc comment saying so.

**Three ownable, zeroable buffers**, none of which the previous draft named:

| buffer | where | today |
| --- | --- | --- |
| `knotBuf` | `engrave/engrave.go:1016-1021` | never zeroed |
| `SafePointer.history` | `engrave/engrave.go:1637` | trimmed by `copy`+reslice at `:1675-1676`, so the tail is never zeroed |
| `splineResumer.catchup` | `gui/engraver.go:222` | never zeroed |

**The genuinely unownable part** is `appendLine`'s per-segment
`make([]bspline.Knot, len(sc))` (`engrave/engrave.go:1146`) — a fresh allocation per
segment that no caller can reach. *That* is the real argument for accepting a
residual limitation, and it is the argument the previous draft should have made
instead of claiming the whole thing was impossible.

**So both earlier framings are withdrawn:**

- **Option 1 as written would have amended the spec to assert an impossibility
  that is measurably not impossible** — on a funds path, in a document future
  work treats as settled. That is the worst outcome available here.
- **Option 2 was dismissed on a memory cost that does not exist.** The 100-knot
  allocation is already made on every `toPlate`: ~1.6 KB on the 32-bit target
  (`bezier.Point{X,Y int}` = 8 B, `Knot` = 16 B padded).

### The ordering hazard — resolved by lifetime, not by one instant

**Round 1's Critical (NC1) killed the previous resolution, and it was right.** The
previous draft zeroed all three buffers together, inside the engrave goroutine,
immediately before its send on `e.errs`, on the argument that the send is the
last read of the spline. **The send ends one goroutine's reads, not the job's** —
and the difference cuts a wrong plate on the machine's most ordinary recovery.

Traced against the code, and confirmed:

1. `Status()` nils `e.errs` when it consumes the send (`gui/engraver.go:134`).
2. `Start()`'s early return is guarded on exactly that field
   (`gui/engraver.go:96-100`), so a nil `errs` **re-arms** `Start()`.
3. **`gui/gui.go:2747` `s.job.Start()`** is the operator's hold-to-resume, in the
   `default:` branch taken for every non-`engraveDone` state — reachable after
   Back (`:2726` `s.job.Stop()`) and after a stall or error.
4. A restart runs `gui/engraver.go:168` `res := newSplineResumer(drv, e.catchup())`,
   and `e.catchup()` is `e.safePoint.Resume(e.conf)`, which **reads
   `s.history`**.

With `history` zeroed, `Resume` (`engrave/engrave.go:1642-1648`) appends knots that are
now `{Ctrl:{0,0}, T:0}` — so the catch-up motion drives the head from the safe
point **to the origin at `T:0`**, zero commanded duration, the exact condition
the jerk-limited planner exists to prevent. At minimum a ruined plate.

**The previous draft cited `gui/gui.go:2651-2656` for this, and that citation is
wrong** (round 1, NM3) — it is the tail of `DescriptorScreen.Confirm`. The real
sites are `:2715` (`defer s.job.Stop()`), `:2726` (`s.job.Stop()`) and `:2747`
(`s.job.Start()`). **The shipped comment at `unlock_session.go:200` carries the
same bad anchor and must be corrected in the same change** — it is the comment
the draft inherited it from.

#### The two structural facts the resolution rests on

Both verified against the code, and both are what make this safe without any
synchronisation:

- **Every terminal transition in `Status()` happens-after the goroutine
  returned.** `Status()` moves to `engraveStopped`/`engraveDone`/`engraveFailed`
  only in the branch that *receives* from `e.errs` (`gui/engraver.go:131-144`), and
  the only send is `errs <- e.runEngraving(...)` after `runEngraving` returns. So
  wherever the state is terminal, there is no live writer — no join, no spin, no
  data race.
- **Restart is impossible once `Engrave` returns.** `s.job.Start()` has exactly
  one caller (`gui/gui.go:2747`), inside `EngraveScreen.Engrave`'s own loop. When
  that function returns, the screen is gone and nothing can re-arm the job.

#### The resolution, split three ways

**1. `knotBuf` — zero inside the engrave goroutine, before the send.** Keep
exactly where the previous draft put it. Safe on *all* exits **including
restart**, for a reason the draft never recorded: `planEngraving`
(`engrave/engrave.go:1027-1029`) opens every iteration with

```go
spline := knotBuf[:0]
```

and rebuilds from the upstream `Engraving`, so a re-range **recomputes** the
buffer and cannot read what was zeroed. That upstream is intact after
`clear(rec)` for the reason given above — it closes over `words`, not `rec`. One
invariant, doing two jobs.

**2. `SafePointer.history` + `splineResumer.catchup` — zero at `Engrave`'s
return, and only when the job is terminal.** These are **resume state**, not cut
state: their lifetime is the *job*, not the goroutine. The previous draft's error
was treating them as cut state.

```go
// in EngraveScreen.Engrave, replacing `defer s.job.Stop()`
defer func() {
	s.job.Stop()
	// Resume state outlives the goroutine BY DESIGN -- catchup() re-reads it on
	// the operator's hold-to-resume (gui.go:2747). It is dead only once this
	// screen is gone, which is here: Start() has no other caller.
	//
	// Terminal-only: a terminal state is the receive on e.errs, so the
	// goroutine has provably returned and there is no live writer. If the job
	// is still running -- Engrave returning because ctx.Done, i.e. the wipe --
	// skip it and let the wipe do its work, rather than race the goroutine.
	s.job.releaseResumeState()
}()
```

with `releaseResumeState` zeroing `e.safePoint.history` and the resumer's
`catchup` when `e.status.State` is terminal, and doing nothing otherwise.

**3. The `history` tail — free, and always safe.** `engrave/engrave.go:1675-1676` does

```go
rem := copy(s.history, s.history[n:])
s.history = s.history[:rem]
```

so everything in `[rem:cap]` is dead by construction, at any time, on every path.
`clear(s.history[rem:cap(s.history)])` at that site needs no ordering argument at
all and should land regardless of the rest.

**Residual, named rather than hidden:** if `Engrave` returns while the job is
still `engraveRunning` — the wipe path — resume state is not zeroed by (2), and
`SafePointer.history` also grows by `append` (`engrave/engrave.go:1683`), so it has the
same outgrown-array class as `op.Buffer` and (3) reaches only the current array.
Both are **F-110** (below).

### The `ErrTooLarge` path, which the cut-end fix cannot reach (round 0, M3)

`toPlate` → `bspline.Measure` fills the knot buffer at **build** time
(`gui/gui.go:2988-2989`), so "for the duration of the cut" was never the right
lifetime bound. On the too-large path (`unlock_session.go:191-193`: `showError`,
`return`) **the buffer is full and no cut ever happens** — no goroutine, no send,
and therefore nothing that (1) above can hook. That error return needs its own
`clear`, or the failure case leaks geometry the success case scrubs. Filed as
part of **F-111**.

---

## What this design does not cover

Stated so the brief does not imply coverage it does not have.

1. **`appendLine`'s per-segment allocations** (`engrave/engrave.go:1146`) — unreachable
   from any caller. The real residual limitation, and what §2.2's amendment
   should name instead of the whole spline (F-85).
2. **`words []string` in `engraveSeed`** — `bip39.LabelFor` returns a slice of the
   **static** wordlist (`bip39/bip39.go:79-87`), so these headers leak the seed's
   *selection*, not copied plaintext. Real, smaller than it looks, unfixed.
3. **The legacy seed-rendering flows have no bracket at all** (round 0, M2):
   `backupWalletFlow` (`gui/gui.go:2194`), `seedEntryFlow` (`gui/derive_xpub.go:82`),
   `bip85DeriveFlow` (`gui/bip85.go:269`), `recoverSLIP39Flow`
   (`gui/slip39_polish.go:229`), `combineSeedXORFlow` (`gui/seedxor_polish.go:40`),
   `passphraseFlow` (`gui/gui.go:584`). None is inside a `Scrub` bracket — only
   `unlock_session.go:276` is. Their frames are zeroed only by the outgrown-array
   change, which reaches what the buffer *outgrew* and not the current array.
   **This design does not fix them**; it is scoped to the B2b secret session.
   Filed as **F-112**.
4. **Cold-boot / SRAM remanence** — see the threat model.

---

## Tests that can fail

Every row below was run, and every mutation row was executed rather than
asserted. Results in §Gate coverage.

1. **F-107, `gui` — the normal exit scrubs.** Drive `runWithFlow` through a real
   unlock to the seed screen, exit **normally** (no wipe), assert
   `ctx.B.Residue()` is `(0,0)`. **Assertion point matters** (round 0, M4): it
   must be taken on the *abandoned* Context after the session defer has run and
   before any later frame reuses the buffer — the existing
   `TestWipeScrubsTheAbandonedFrameBuffer` shows the shape. Mutation row: delete
   `ctx.B.Scrub()` from the session defer.
2. **F-107 outgrown-array class, `gui/op` — the load-bearing one.**
   `TestAppendZeroesTheArgsArrayItOutgrows` / `…TheRefsArrayItOutgrows` hold their
   **own** reference to the array the Buffer is about to outgrow, fill it with a
   canary, force the reallocation, and read the memory back. They deliberately do
   not consult `Buffer.Zeroed()` for the verdict — the bookkeeping is exactly what
   a mutation leaves intact while the bytes survive. Mutation rows: delete
   `clear(old[:cap(old)])` from each funnel.
3. **F-107 routing, `gui/op` — a lint, because no behavioural test can catch
   this.** `TestBufferGrowthIsFunnelled` scans `op.go` for `.args = append(` /
   `.refs = append(` outside the two funnel functions. **Measured necessity:**
   reverting one site (`encodeOp`'s header append) to a raw append leaves every
   behavioural test in `gui/op` and `gui` green, because the unrouted array is
   unreachable and unzeroed — the exact failure round 0 graded Critical.
4. **F-107 reach, `gui`.** `TestSeedFrameReachesTheOutgrownArrayClass` renders a
   real 24-word `SeedScreen` and asserts the frame actually outgrew its arrays,
   failing as **inconclusive** if it did not. It exists because a correctly-zeroed
   array and an array that was never created are indistinguishable from outside,
   so without it every other row passes vacuously on a small frame. It does **not**
   claim to verify the zeroing — measured: with `clear` deleted it still passes,
   because `Residue()` is structurally blind to an array nothing retains. Named
   for what it pins; its first draft was called
   `TestOutgrownArraysAreZeroedAtReallocation` and asserted no such thing.
5. **F-108 restart, `gui` — the NC1 regression test.** Drive a job to
   `engraveStopped` via Back, then `Start()` again, and assert the catch-up motion
   is unchanged from an un-zeroed run. **Must fail against the previous draft's
   resolution.** This is the row that would have caught NC1, and there was no
   equivalent in the previous draft's test list.
6. **F-108 knot buffer, `engrave`.** After a full cut, assert
   `knotBuf[:cap]` is all-zero; and after a *restart*, assert the spline
   re-materialises identically — pinning that zeroing is safe rather than merely
   done.

**Deleted from the previous draft (round 0, I5):** the finalizer/lifetime canary
on the plate, whose stated premise was "*since there are no bytes to check*". The
F-108 rewrite destroys that premise — there are 9 measured non-zero knots to
check — so a reachability test is now the weaker assertion, and rows 5 and 6
supersede it.

**Neither test may assert on a buffer it also holds a reference to.** The `op`
canary work established the trap: `runtime.KeepAlive` the holder, use two `GC()`
calls plus a timeout, and choose the canary so it enters the structure under test.

---

## Gate coverage

**Rewritten against the design that now exists** (round 1, NI3 — the previous
version claimed F-108 had no code to gate, which stopped a reviewer asking to see
the code the Critical was in).

**Applied to a scratch worktree** (`seedhammer-gate-orphan`, off b2b `3de8aa1`)
and run, not projected. What is **built and measured**:

```
go build ./...                    clean (2 pre-existing failures: cmd/kdfbench,
                                  cmd/sealread -- TinyGo-only `machine` import,
                                  identical on untouched b2b)
go vet ./gui/...                  2 pre-existing diagnostics, identical on b2b
gofmt -l gui/                     5 files, all pre-existing, none of them mine
go test ./...                     ok -- gui 41.2s, gui/op 1.6s, seal 16.5s, all pass

tinygo -target pico-plus2 -gc precise -opt 2 -scheduler tasks ./cmd/controller
  baseline (b2b 3de8aa1)   1313816 flash / 60584 RAM
  with the funnel          1314344 flash / 60584 RAM
  delta                       +528 flash /     +0 RAM
```

**Mutation rows, executed:**

| mutant | result |
| --- | --- |
| delete `clear(old[:cap(old)])` from `appendArgs` | **KILLED** — `TestAppendZeroesTheArgsArrayItOutgrows` |
| delete `clear(old[:cap(old)])` from `appendRefs` | **KILLED** — `TestAppendZeroesTheRefsArrayItOutgrows` |
| unroute one site (`encodeOp` header) to a raw `append` | **SURVIVED** all behavioural tests; **KILLED** by `TestBufferGrowthIsFunnelled` |

**Measured on a real 24-word `SeedScreen` frame** (`TestMeasureSeedFrameOrphans`):

```
24-word: len args=2505 refs=781 | cap args=3392 refs=1023 (current retention 21.2 KB)
         outgrown-and-zeroed: 22 arrays, 7919 entries (30.9-61.9 KB, freed not retained)
         residue before Scrub: args=2254 refs=781      <- was args=8320 refs=1951 with
         residue after  Scrub: args=0    refs=0           the arrays merely recorded
12-word: len args=1327 refs=403  | cap args=1728 refs=591 (current retention 11.4 KB)
         outgrown-and-zeroed: 19 arrays, 3232 entries
```

`cap args=3392` independently reproduces round 0's measured 3392, which is the
cross-check on the harness. Note the idealised doubling series predicts 3072 for
this frame and omits `refs` entirely — **that is why these are measurements and
not a series**, and it is why round 1's Appendix A figures (2.45×/3.45× on `args`
alone) do not describe the real buffer either.

**What is NOT built, and gates nothing yet:**

- **F-108's zeroing is unimplemented.** Items (1), (2) and (3) of the resolution —
  `knotBuf` at the goroutine exit, `releaseResumeState` at `Engrave`'s return, the
  `history` tail-clear — exist as design text only. `releaseResumeState` does not
  exist in the tree. **This is the part the Critical was in, and it is the part a
  reviewer must read as design rather than as verified code.**
- **Test rows 1, 5 and 6 are not written.** Rows 2, 3 and 4 are written and run.
- The `unlock_session.go:200` comment correction and the `:245` comment are not
  applied.

---

## What R0 should attack

Round 1 resolved four of the previous five; they are struck rather than deleted
so the next reviewer can see what is already settled and not re-derive it.

1. ~~**F-108's ordering.**~~ **RESOLVED by NC1 and re-resolved above.** The
   restart path is `gui/gui.go:2747`, not `gui/engraver.go:146-148` (which is inert:
   `Start()` early-returns while `errs != nil`, and the only assignment to
   `engraveRunning` is `Start()` itself at `gui/engraver.go:108`). **What is still open:** is the
   three-way split correct — specifically, is `releaseResumeState`'s terminal-only
   guard the right treatment of the wipe path, or should the wipe reach the job's
   resume state directly?
2. ~~**Is option 1 acceptable?**~~ **WITHDRAWN.** The premise ("RESOLVED that the
   spline is unzeroable") is false and the fold withdrew it; the spline is
   measurably clearable. There is a threat model now, so the residuals in §What
   this design does not cover can be judged against a stated adversary.
3. **F-107's scrub position.** Does any op built into `ctx.B` outlive
   `unlockSecretSession`'s return? Round 0 verified the defer runs strictly
   between frames (§(a)). **Still open:** the same question for
   `unlockPassphraseFlow`'s bracket, which I1 added and which nobody has checked.
4. ~~**Does F-107's fix subsume `run_flow.go:245`?**~~ **RESOLVED: no, it stays.**
   The previous draft's "`Scrub` is idempotent, so this is about clarity, not
   correctness" is struck — it is the sentence a future fold would have read
   before deleting `:245`.
5. **What else renders a secret?** Answered and **it is worse than the previous
   draft implied**: six legacy flows render seeds with no bracket at all (§What
   this design does not cover, item 3). **Open:** is F-112 schedulable after the
   release tag, or does an unbracketed `backupWalletFlow` block it?
6. **New — is zero-at-reallocation the right trade?** It departs from round 1's
   proposed fix. The argument is the 35.5 KB retention and the F-109 collision.
   The counter-argument a reviewer should press: retention makes the property
   *auditable* at `Scrub` time, and zero-at-reallocation makes it invisible to
   everything except a mutation test.

---

## Follow-ups filed by this fold

- **F-110** — resume state on the abandon paths: `releaseResumeState` skips a
  still-running job, and `SafePointer.history` grows by `append`
  (`engrave/engrave.go:1683`) so it carries the outgrown-array class the tail-clear
  cannot reach. Owning phase: B2b.
- **F-111** — `knotBuf` on the `ErrTooLarge` path (`unlock_session.go:191-193`),
  where the buffer is filled at build time and no cut, goroutine or send ever
  happens. Owning phase: B2b.
- **F-112** — six legacy seed-rendering flows sit inside no `Scrub` bracket.
  Owning phase: post-B2b, before the release tag.
- **F-108's register entry is stale** (round 1, NN1): `FOLLOWUPS.md:1762` still
  records the withdrawn framing, "*§10.2.2 claims a wipe-by-any-route guarantee
  the geometry cannot satisfy*". Corrected in the same commit as this fold.

---

Resolved against the source and quoted, not paraphrased: the single `Scrub` call
site, `Buffer.Reset`'s body, `uiFlow`'s loop condition, `cmd/controller/main.go:34`,
`unlock_session.go`'s guard bracket and its `LIVE plate.Spline` inventory line,
`clear(rec)` at `:204`, `Start`/`Status`/`catchup` in `gui/engraver.go`,
`s.job.Start()` at `gui/gui.go:2747`, `spline := knotBuf[:0]` at `engrave.go:1029`,
the `history` trim at `:1675-1676`, and the twelve append sites in `gui/op/op.go`.
