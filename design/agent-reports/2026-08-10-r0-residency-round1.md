# R0 round 1 — fold verification, DESIGN_b2b_residency_zeroing.md (F-107 / F-108)

**Reviewer:** independent architect agent (adversarial), 2026-08-10.
**Scope:** fold verification ONLY — (A) did the fold resolve each round-0 finding,
(B) did the fold introduce a new Critical or Important. Not a fresh audit.
**Artifacts:** round 0 = `design/agent-reports/2026-08-10-r0-residency-round0.md`;
folded design = `design/DESIGN_b2b_residency_zeroing.md`; fold =
`17a748a..HEAD` (`fdec087` + `1163007`, both touching that one file and nothing
else).
**Code read (read-only):** `/scratch/code/shibboleth/seedhammer-b2b` @ `3de8aa1`
(branch `b2b`).

**Verdict: RED — 1 Critical, 3 Important new. Not GREEN.**

The F-108 rewrite is the strongest part of the fold and is genuinely
first-rate: it withdraws two false premises, names the three ownable buffers,
and re-costs option 2 against an allocation that already exists. The Critical is
not in that rewrite. It is in the **new** ordering resolution added by `1163007`,
which the design itself asked R0 to judge — and the answer is the bad one.

Everything numeric below was measured. The one number I computed rather than
read is in §B.2 and the program is reproduced in Appendix A.

---

# (A) Per-finding verdicts

| # | verdict | justification (design section cited) |
| --- | --- | --- |
| **C1** | **PARTIAL** | §"C1 (round 0)" replaces the fix with a real, different, correct mechanism — `orphanArgs`/`orphanRefs` recorded by `growArgs`/`growRefs`, cleared by `Scrub`, **counted by `Residue`** — which also retires round 0's "`Residue()` cannot witness this class". The substantive half is done. Two requirements are not in the document: round 0 required "**plus a test that can see orphaned arrays**", and §"Tests that can fail" is byte-identical to round 0's (test 1 still reads as the pre-C1 test); and the per-site `n` is unspecified, which is where the mechanism is fragile — see **NI1**. |
| **I1** | **RESOLVED** | The false universal ("*the seed is only ever rendered inside the secret session*") is **deleted**, and §"The scrub's position" now says the scrub belongs on **both** brackets, naming `unlockPassphraseFlow`'s own defer. Verified the defer exists to hang it on: `gui/unlock_kdf.go:137` `defer func() { ctx.wipe = prev }()`. |
| **I2** | **PARTIAL** | §"RE-RESOLVED after round 0" is a correct, measured rewrite: `Curve = iter.Seq[Knot]` declared irrelevant, the three ownable buffers tabulated, `appendLine`'s per-segment `make` named as the real residual, option 2 re-costed at ~1.6 KB already-allocated. **But** §"What R0 should attack" #2 is unchanged and still reads "*RESOLVED that the spline is unzeroable*" — the document withdraws and asserts the same proposition. One strike-through closes this and I4 together. |
| **I3** | **PARTIAL** | The false sentence ("*the geometry was computed into the closure beforehand, which is why that early clear is sound*") went out with the whole section — good. I3's required **replacement** did not go in: nowhere does the design state the correct invariant (`clear(rec)`/`clear(m)` is safe **only** because `engraveSeed`/`EngraveSeedString` materialise independent copies before the closure is built, and any new derivation that captures `rec`/`m` breaks it). §F-108 lines 136-140 still hold `clear(rec)`-before-`Engrave` up as "the correct pattern" with no caveat. The measured laziness fact (upstream invocations 0→1→2) is recorded nowhere — and it is precisely the fact that makes **NC1** below possible. |
| **I4** | **PARTIAL** | §"The scrub's position" now answers it correctly: "*I4: `run_flow.go:245` is **not** subsumed and stays.*" **But** §"What R0 should attack" #4 survives verbatim — "*`Scrub` is idempotent, so this is a question about clarity, not correctness*" — which is the exact sentence I4 said a future fold would read before deleting `:245`. The one-line comment at `:245` naming what it uniquely covers is also not mentioned. |
| **I5** | **NOT ADDRESSED** | §"Tests that can fail" #3 (the finalizer/lifetime canary) is present **verbatim**, including "*since there are no bytes to check*" — a premise the fold's own F-108 rewrite destroys. `git diff 17a748a..HEAD` touches no line of the tests section. |
| **I6** | **NOT ADDRESSED** | No threat model. `grep -in "threat\|adversar\|attacker\|powered\|cold boot"` over the folded design returns **zero** matches. §"What R0 should attack" #2's question ("is option 1 acceptable for a funds path") therefore remains unanswerable as written, and the fold has now made it a *live* question again by proposing real code. |
| **M1** | **NOT ADDRESSED** | No mention of `d.Release()`, the Drawer, `maskStack`, or `glyphImage` anywhere in the design (`grep` returns zero). It matters *more* after the fold, not less: retained orphans mean the Drawer's stale `frameOp` slice headers now alias **reachable** arrays rather than garbage. (The aliasing is still safe — see §B.1 — but the design still does not say it checked.) |
| **M2** | **PARTIAL** | The **denial** is gone (deleted with the old §"The fix"), which was the blocking half. The two other halves are not done: the design nowhere states that `backupWalletFlow`/`SeedScreen.Confirm`, `seedEntryFlow`, `bip85DeriveFlow`, `recoverSLIP39Flow`, `combineSeedXORFlow`, `passphraseFlow` render seeds with no bracket at all, and no follow-up was filed — both fold commits touch `design/DESIGN_b2b_residency_zeroing.md` and nothing else, `FOLLOWUPS.md` is untouched. This also leaves §B.2's "until the next Scrub" ungrounded. |
| **M3** | **PARTIAL** | Acknowledged — but **only** inside the trailing paragraph the document itself labels "**Superseded framing**" (see **NM4**), i.e. M3's substance is parked in a block flagged as dead text. And the re-scoped fix ("*zero … at the end of the cut*") structurally cannot cover the path M3 identified: `toPlate` → `bspline.Measure` fills `knotBuf` at **build** time, so on `ErrTooLarge` (`unlock_session.go:191-193`, `showError`, `return`) the buffer is full and **no cut ever happens**, hence no goroutine, hence no zeroing. |
| **M4** | **NOT ADDRESSED** | Test section unchanged; test 1's assertion point is still unspecified, and the obvious end-of-run point still fails with the fix. |
| **N1** | **RESOLVED** | "*Same position in the frame cycle as the existing `run_flow.go:245` call*" was deleted with the old fix block, and the replacement distinguishes the two positions correctly: "`:245` … *covers the wipe path, where the Context is abandoned rather than returned through*". |
| **N2** | **NOT ADDRESSED** | Line 137 still cites `unlock_session.go:195-203` for `clear(rec)`. Re-confirmed against the tree: `:195-203` is the comment, `clear(rec)` is **line 204**. |
| **N3** | **NOT ADDRESSED** | Line 132 still asserts "*the three matches are two comments and a constructor*" with no command shown, in the sentence that carries F-108's negative result. |

**Counts:** RESOLVED 2 · PARTIAL 6 · NOT ADDRESSED 6 · REGRESSED 0.

Note the shape: **every Important the fold engaged with, it engaged with well**;
what it did not do is sweep the parts of the document the findings did not
anchor into. Six of the fourteen verdicts (I2, I4, I5, I6, M4, plus half of C1)
are "the fold rewrote §F-107 and §F-108 and left §Tests, §What R0 should attack
and §Gate coverage exactly as they were", so the document now argues with
itself in four places. That is one editing pass, not four rounds.

---

# (B) New findings

## CRITICAL (1)

### NC1 — zeroing the resume state before the send on `e.errs` cuts a wrong plate on the operator's ordinary Back-then-resume

**Anchor:** design §"The ordering hazard — proposed resolution", the sentence
"*Anything zeroed before that send is provably no longer read by the loop, on
every exit — completion, `Stop()`, and error alike, since all three converge on
that send*", combined with §"Re-scoped fix" ("*zero `knotBuf`,
`SafePointer.history` and `splineResumer.catchup` at the end of the cut*").
Code: `gui/engraver.go:96-114` (`Start`), `:126-150` (`Status`), `:78-81`
(`catchup`), `:160-184` (`runEngraving`), `engrave/engrave.go:1642-1648`
(`SafePointer.Resume`); `gui/gui.go:2714-2779` (`EngraveScreen.Engrave`).

**The design asks R0 to judge exactly this, and states the bar itself: "*A
restart that re-reads a zeroed buffer would cut a wrong plate, which is worse
than leaving the bytes resident.*" Plainly: yes, this design can do that.**

`e.errs` is the last read of the spline **for one goroutine run**. It is not the
last read *of the job*. `Status()` sets `e.errs = nil` when it consumes the send
(`engraver.go:132`), and `Start()`'s early return is guarded on exactly that
field (`engraver.go:97-100`), so a nil `errs` re-arms `Start()` — and
`EngraveScreen.Engrave` calls it: **`gui/gui.go:2747` `s.job.Start()`**, behind
the hold-to-confirm on the select button, in the `default:` branch that is taken
for every non-`engraveDone` state.

**Fully reachable by an ordinary operator sequence, traced:**

1. Job running. Operator presses Back → `gui.go:2726` `s.job.Stop()` → state
   `engraveStopping`, `close(quit)`.
2. `reportProgress` returns false → `runEngraving` returns → **the goroutine
   would zero here** → `errs <- nil`.
3. Next `Status()` → state `engraveStopped`, `e.errs = nil`.
4. Operator holds select → `gui.go:2747` `s.job.Start()` → new goroutine.
5. `runEngraving:168` `res := newSplineResumer(drv, e.catchup())` →
   `e.catchup()` = `e.safePoint.Resume(e.conf)` → **reads `s.history`**, which
   step 2 zeroed.

The stall/error exit is the same shape and is the *more* common one:
`runEngraving` returns a non-nil `err` → `engraveFailed` → same `default:`
branch → same `Start()`. Resume-after-stall is the whole reason
`SafePointer`/`nknots`/`splineResumer` exist.

**What a zeroed `history` produces.** `Resume` (engrave.go:1642-1648) is
`appendLine(move, conf, false, bezier.Point{}, s.safePoint)` followed by
`append(move, s.history...)`. `safePoint` is a `bezier.Point` value field and
survives; the history knots become `{Ctrl:{0,0}, T:0, Engrave:false}`. So the
catch-up motion drives the head from the safe point **to the origin**, at `T:0`
— zero commanded duration, i.e. maximum step rate, the precise condition the
jerk-limited planner exists to prevent — and only then does the main spline
resume from knot `e.nknots` with its absolute control points. At minimum a
ruined plate; with lost steps, every remaining stroke is offset and the plate is
cut wrong.

**The shipped code already names this path as the one that must not break.**
`gui/unlock_session.go:198-203`: "*Back while running does NOT return … so the
abort-mid-plate path §10.2.2 calls **the machine's most ordinary recovery***".
The design quotes that phrase — "*turning the machine's most ordinary recovery
into a ruined plate*" — as the hazard it is avoiding, and then proposes the
resolution that causes it.

**Why the fold missed it, mechanically:** the design's own citation for the
ordering hazard, `gui.go:2651-2656`, is wrong (see **NM3**). It points at
`DescriptorScreen.Confirm`'s tail. Resolving it would have landed the author in
`EngraveScreen.Engrave` at `:2715`/`:2726`/**`:2747`**, with `s.job.Start()`
twenty-one lines below `s.job.Stop()`.

**Smallest correct fix — split the two lifetimes; do not abandon the resolution.**
The resolution is right for one of the three buffers and wrong for the other two:

- **`knotBuf` — keep it exactly where the design puts it.** Verified safe on
  *all* exits including restart, and for a reason the design should record:
  `planEngraving` (`engrave.go:1027-1029`) opens every iteration with
  `spline := knotBuf[:0]` and rebuilds from the upstream `Engraving`, so a
  re-range recomputes the buffer from scratch and cannot read what was zeroed.
  Zeroing before the send introduces no data race either: the send is the
  happens-before edge that gates `Start()`.
- **`SafePointer.history` + `splineResumer.catchup` are RESUME STATE, not cut
  state.** Their lifetime is the *job*, not the goroutine. They may only be
  zeroed where the job is provably abandoned — the `engraveDone` transition in
  `Status()` (which cannot restart: the restart requires `State ==
  engraveRunning`), and `EngraveScreen.Engrave`'s return. Not on `Stop()`, not
  on error.
- **One piece is free and needs no ordering argument at all**, and is worth
  splitting out because it is the half of round 0's I2 that carries no risk:
  the `history` **tail** beyond the trim. `engrave.go:1675-1676` does
  `rem := copy(s.history, s.history[n:]); s.history = s.history[:rem]`, so
  everything in `[rem:cap]` is dead by construction. `clear(s.history[rem:cap(s.history)])`
  at that site is unconditionally safe, at any time, on every path.

Then re-state the invariant the design is actually asserting, because it is not
the one it wrote: **"the send on `e.errs` ends this goroutine's reads, not the
job's."**

---

## IMPORTANT (3)

### NI1 — the orphan recorder's correctness is 12 hand-computed `n` values with no structural guarantee, and the two sites carrying the glyph args each perform TWO appends

**Anchor:** design §"C1 (round 0)", the `growArgs` block and the claim
"*routing them through the recorder is a local change and **not a discipline
anyone must remember***". Code: `gui/op/op.go`.

The mechanism is right. `if cap(b.args)-len(b.args) < n && cap(b.args) > 0` is
an *exact* predictor of reallocation — for **one** append of **exactly** `n`
elements. The design specifies no `n` at any site, and the sites do not have
that shape. Measured, the twelve appends the design counts:

| site | `args` appends | `refs` appends |
| --- | --- | --- |
| `ParamImageMask` op.go:144-147 | 2 (`args...`, then header) | 2 (`img`, then `refs...`) |
| `ensureLatest` op.go:195 | 1 (4 words) | — |
| `encodeOp` op.go:429-431 | 2 (`args...`, then header) | 1 |
| `newCompose` op.go:458 | 1 (4 words) + calls `encodeOp` | — |
| `group.add` op.go:495, 497 | 2, the first conditional on `!g.discontinuous` | — |
| `group.Op` op.go:510 | 1 (3 words), conditional | — |

`encodeOp` is the universal path — **every** op, including `op.Glyph`, goes
through it. Its natural reading, `growArgs(len(args))`, is **off by one**: the
first append then fits exactly, and the *second* (the one-word header)
reallocates with no orphan recorded — orphaning the array that just received
`args...`, i.e. the runes. That is C1, reintroduced silently, on the seed path.
Correct is `1+len(args)`; `ParamImageMask` needs `1+len(args)` and
`1+len(refs)`; `group.add`'s branch needs 1 or 4 depending on
`g.discontinuous`.

**Would a test notice?** For `encodeOp`, yes — any realistic frame hits it, and
post-fold `Residue()` counts orphans. For `ParamImageMask`, `group.add`'s
discontinuous branch, `group.Op` and `newCompose`, **no**: a frame that does not
take those branches leaves the site unexercised and the leak invisible. The
prototype's mutation row (delete `Scrub`'s orphan-clearing → FAIL) proves the
*recorder and scrubber work*; it proves nothing about whether every site is
routed. And the failure mode is the one round 0 graded Critical: `Residue()`
reports 0 while the array holds the words.

**Smallest correct fix — make it structural instead of arithmetic.** Detect
reallocation *after* the append, in a single funnel, so `n` disappears:

```go
func (b *Buffer) appendArgs(vals ...uint32) {
	old := b.args
	b.args = append(b.args, vals...)
	if cap(b.args) != cap(old) && cap(old) > 0 {
		b.orphanArgs = append(b.orphanArgs, old[:cap(old)])
	}
}
```

Unconditionally correct for any count, any number of appends, any branch — and
it turns "did I route this site" into a grep for `b.args = append(` outside the
funnel, which is a lint, not a discipline. Then the design's sentence becomes
true instead of aspirational. If prediction-before is kept for a reason, the
design must state the `n` for all twelve sites and add the grep as a test.

### NI2 — the memory bound is wrong (measured 2.7×–3.5×, not "~2×"), and "until the next Scrub" is not a bound on the paths where no Scrub runs

**Anchor:** design §"C1 (round 0)", §"Cost": "*the orphaned arrays stay
reachable **until the next Scrub** … bounded by the doubling series, **~2× the
high-water mark**.*"

Both halves are false, and the design's own next paragraph already says so.

**"Doubling series" is not how Go grows a slice.** `growslice` doubles only
below 256 elements and then grows by ~1.25×, so the series is longer and sums to
more. Measured (Appendix A, go1.26.3):

```
args 24-word seed   target=2387  final cap=3072   orphan arrays=10  orphan entries=5272   orph/final=1.72x  total/final=2.72x
args prototype      target=10240 final cap=10240  orphan arrays=14  orphan entries=25048  orph/final=2.45x  total/final=3.45x
refs high-water     target=511   final cap=512    orphan arrays=6   orphan entries=504    orph/final=0.98x  total/final=1.98x
   args caps: [8 16 32 64 128 256 512 864 1344 2048 3072 4096 5440 7168 10240]
```

The prototype row reproduces the design's own cited figure — "*16 orphaned
arrays holding 25,054 non-zero words*" at cap 10,240 — to within the warm-start
offset. **That figure is 2.45× in orphans alone and 3.45× total**, quoted two
paragraphs below the sentence claiming ~2×. On the real device path the args
high-water is the 24-word seed frame: **~21 KB of orphans + 12 KB current ≈ 33 KB**,
plus ~4 KB of refs. Against 283 K free that is ~12% — affordable, but the
design's justification for the trade is the number, and the number is out by
1.4–1.7×.

**"Until the next Scrub" is not a bound where no Scrub exists.** After this
design the `Scrub` sites are `run_flow.go:245` (wipe only), the secret-session
defer and the passphrase-flow defer. On M2's entire legacy surface —
`backupWalletFlow`, `seedEntryFlow`, `bip85DeriveFlow`, `recoverSLIP39Flow`,
`combineSeedXORFlow`, `passphraseFlow` — **none of the three ever fires**, so
the retention is for the machine's uptime, not until the next Scrub.

The retention *is* bounded (once `cap` reaches the largest frame, no further
reallocation, hence no further orphans) — say that, since it is the reassuring
part and the design does not currently earn it.

**One interaction the design must name.** F-109 is open and gating B2b: ~35 K in
~81 **reachable** objects surviving every wipe, unidentified, with the operator's
ruling that "bounded ≠ safe" until the objects are named. This design
deliberately adds ~33 KB of reachable, seed-derived arrays to exactly that
population. F-109's closing measurement will attribute them to the mystery
unless the design says up front what it is adding and why.

### NI3 — §"Gate coverage" now describes a design that no longer exists, in the one section whose sole job is truthful coverage

**Anchor:** design §"Gate coverage", unchanged by the fold: "***F-108 no longer
has code to gate** — its remedy is a spec amendment, not a patch. Stated so the
brief does not imply coverage that does not exist.*" and "***F-107's fix IS
gated***" with `go build` / `go test` / `tinygo … 1313768 flash / 60584 ram`.

Both statements are now false:

- **F-108 has code.** The fold's re-scoped fix zeroes three buffers and changes
  the engraver's goroutine exit ordering. That is a patch, not a spec amendment
  — and it is where **NC1** lives. A reviewer who reads this section does not
  ask to see F-108's code, which is exactly what happened.
- **F-107's quoted gate output is for the superseded one-line-defer fix.** The
  fix is now two new `Buffer` fields, two new methods, twelve call-site changes
  and modified `Scrub`/`Residue`. The `tinygo` flash/RAM figures in particular
  cannot survive that unchanged, and the fold did not re-run them.

Per the project's own rule a gate that hides its blind spot is worse than no
gate; this one now advertises coverage in the wrong direction. Rewrite it
against what actually exists: state that the orphan mechanism is prototyped
(`go build ./gui/...` clean, `go test ./gui/op/ ./gui/ ./seal/` ok, mutation row
kills), state the tinygo numbers are stale pending a re-run, and state that
F-108's zeroing is **not** built at all.

---

## MINOR (4)

### NM1 — `Scrub` must `clear` the orphan list's backing array, not merely "drop" it

Design: "*`Scrub` then clears the orphans as well as the current arrays and
**drops the list***". If that is implemented as `b.orphanArgs = b.orphanArgs[:0]`,
the stale slice headers past `len` keep every orphaned array **reachable for the
buffer's lifetime**, so the retention in NI2 becomes permanent rather than
transient. This is precisely the class the repo already documented and fixed
eight lines away — `gui/op/op.go:250-260`: "*clear to CAP, not just truncate.
The backing array outlives the slice, and a mark-sweep collector scans whole
allocated objects*". Bounded (~14 headers, the same arrays), and the arrays are
zeroed by then, so it is memory hygiene rather than secrecy — but say
`clear(b.orphanArgs[:cap(b.orphanArgs)])` explicitly, in a design whose entire
subject is that truncation is not clearing.

### NM2 — orphaned `refs` arrays pin their referents, and the cost paragraph counts only bytes

`Buffer.Reset` does `clear(b.refs)` every frame **precisely** so the interface
values drop their referents. An orphaned `refs` array is never Reset-cleared, so
it pins whatever it held at the instant of reallocation until a `Scrub` that may
never come (NI2). Measured high-water is 512 refs → ~504 pinned entries. In
practice most are package globals (`glyphImage`, assets, faces), so the
transitive cost is small — which is the reason to state it rather than leave a
reader to work it out on a 283 K-free device.

### NM3 — `gui.go:2651-2656` is the wrong anchor, is cited twice by the fold, and resolving it is what would have surfaced NC1

`gui/gui.go:2651-2656` is the tail of `DescriptorScreen.Confirm` and the head of
`DescriptorScreen.Draw`; there is no `Stop()` in it. The real sites are
`gui.go:2715` (`defer s.job.Stop()`), `:2726` (`s.job.Stop()`) and — the one that
matters — **`:2747` (`s.job.Start()`)**. The fold inherited the bad anchor from
the shipped comment at `gui/unlock_session.go:200`, which carries the same drift
and should be corrected in the same pass. Same class as round 0's N2, graded
Minor rather than Nit only because it is the load-bearing citation of the new
section and it concealed a Critical.

### NM4 — the trailing "Superseded framing" paragraph contradicts the section immediately above it, and is the only place M3 is acknowledged

Lines 228-234 restate the ordering hazard as "**unchanged and now the whole
difficulty**", forty lines after the section that resolves it. Labelled
"Superseded framing", so a reader is told to discount it — yet M3's entire
acknowledgment ("*`bspline.Measure` fills the knot buffer at build time, so 'for
the duration of the cut' was never the right lifetime bound*") lives only
inside it. Delete the duplicated hazard text; promote the M3 sentence into the
re-scoped fix, where it forces the open question NC1's fix must also answer:
**what zeroes `knotBuf` on the `ErrTooLarge` path, where the buffer is filled at
build time and no cut, no goroutine and no send ever happen?**

---

## NIT (2)

- **NN1.** `design/FOLLOWUPS.md:1762` still records F-108's withdrawn framing —
  "*F-108 becomes: §10.2.2 claims a wipe-by-any-route guarantee the geometry
  cannot satisfy*". Neither fold commit touched `FOLLOWUPS.md`, so the follow-up
  register and the design now disagree about whether the geometry is zeroable.
  Records have been the wronger half here before.
- **NN2.** The C1 code block shows one helper of the four changed functions and
  elides the rest with `...`, while the brief states the whole thing was
  prototyped and mutation-tested. Paste the prototyped `growRefs`, `Scrub` and
  `Residue` — a reviewer cannot check `n` (NI1) or the "drops the list" wording
  (NM1) against an ellipsis.

---

## Explicitly checked, no finding

Recorded so the next round does not re-derive them.

- **Zeroing an orphan cannot corrupt a frame still being drawn.** All appends
  happen during layout; `Drawer.Draw` walks the buffer afterwards and takes
  `oargs`/`rargs` from the **current** array only, so no `imageOp` header ever
  aliases an orphan of the frame it belongs to. Stale headers from the previous
  frame are dropped at the next `Draw` (`op.go:262` clears `maskStack` to cap);
  `inputOp.tag` is an interface-value copy, not an alias. Orphans are strictly
  older than the current array, and round 0 §(a) already proved the current
  array is safe to zero at the scrub point — so the orphans are safe a fortiori.
  The `d.Release()` question (M1) is unchanged by orphan retention.
- **No data race is introduced by zeroing inside the engrave goroutine.** The
  buffered send on `e.errs` is the happens-before edge that gates `Status()`
  nilling `errs` and hence `Start()`; nothing else touches `e.safePoint`,
  `e.nknots` or `knotBuf` from the UI goroutine.
- **The spline has exactly two readers in firmware:** `gui/engraver.go:170`
  (the cut) and `gui/gui.go:2988-2989` (`bspline.Measure` at build time). The
  other `PlanEngraving` sites are `cmd/` tools and `gui/qa.go`, which builds its
  own plan from `qaPlan` and never resumes.
- **`unlock_kdf.go:137`'s defer exists** and is a valid hanging point for I1's
  scrub.
- **Test 1's citation of `Residue()` is now correct**, because the fold extends
  `Residue` to count orphans — round 0's "stop citing it as the witness" is
  retired by construction, not by editing.

---

## What must be true to reach GREEN

1. **NC1**: `SafePointer.history`/`splineResumer.catchup` moved off the goroutine
   exit onto a provably-abandoned point; `knotBuf` stays where it is, with the
   `spline := knotBuf[:0]` reason written down; the `history` tail-clear split
   out at `engrave.go:1675-1676` as the free, always-safe piece.
2. **NI1**: the recorder made structural (funnel + detect-after), or all twelve
   `n` values stated with a grep test.
3. **NI2**: the cost restated at the measured 2.7×–3.5× / ~33 KB, "until the next
   Scrub" corrected for the no-Scrub paths, and the F-109 interaction named.
4. **NI3**: §"Gate coverage" rewritten against the design that now exists.
5. **The stale sections swept** — this closes I2, I4, I5, M4 and the second half
   of C1 in one pass: strike or answer §"What R0 should attack" (#2 and #4 assert
   what the fold withdrew), delete test 3 (I5), fix test 1's assertion point
   (M4), and add the orphan-visible test (C1).
6. **I3, I6, M1, M2, M3, N2, N3** folded as round 0 specified.

---

# Appendix A — the growth measurement (reproduce verbatim)

Nothing in the tree was modified. go1.26.3 from the Nix store
(`/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`).

```go
package main

import "fmt"

func series(target int) (steps []int) {
	var s []uint32
	prev := 0
	for len(s) < target {
		s = append(s, 0)
		if cap(s) != prev {
			if prev > 0 {
				steps = append(steps, prev)
			}
			prev = cap(s)
		}
	}
	steps = append(steps, prev) // final (current) cap
	return
}

func report(name string, target int, wordBytes int) {
	st := series(target)
	final := st[len(st)-1]
	orph := 0
	for _, c := range st[:len(st)-1] {
		orph += c
	}
	fmt.Printf("%-22s target=%-6d final cap=%-6d orphan arrays=%-3d orphan entries=%-7d ratio orph/final=%.2fx  total/final=%.2fx  orphan bytes(%dB/entry)=%d\n",
		name, target, final, len(st)-1, orph, float64(orph)/float64(final), float64(orph+final)/float64(final), wordBytes, orph*wordBytes)
	fmt.Printf("   caps: %v\n", st)
}

func main() {
	report("args 24-word seed", 2387, 4)
	report("args prototype", 10240, 4)
	report("refs high-water", 511, 8)
}
```

Output:

```
args 24-word seed      target=2387   final cap=3072   orphan arrays=10  orphan entries=5272    ratio orph/final=1.72x  total/final=2.72x  orphan bytes(4B/entry)=21088
   caps: [8 16 32 64 128 256 512 864 1344 2048 3072]
args prototype         target=10240  final cap=10240  orphan arrays=14  orphan entries=25048   ratio orph/final=2.45x  total/final=3.45x  orphan bytes(4B/entry)=100192
   caps: [8 16 32 64 128 256 512 864 1344 2048 3072 4096 5440 7168 10240]
refs high-water        target=511    final cap=512    orphan arrays=6   orphan entries=504     ratio orph/final=0.98x  total/final=1.98x  orphan bytes(8B/entry)=4032
   caps: [8 16 32 64 128 256 512]
```

The `args prototype` row is the check on the design's own quoted measurement
(16 arrays / 25,054 words at cap 10,240): the small delta is the prototype's
warm start at cap 128 and its multi-element appends. Either figure refutes
"~2×".
