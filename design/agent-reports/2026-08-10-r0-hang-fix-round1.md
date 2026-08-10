# R0 round 1 — fold verification of `DESIGN_b2b_hang_retention_and_bounded_read.md`

**Role:** verify the FOLD only (`git diff 175cdd5..4c373d9`), not a fresh audit.
**Two questions:** (A) did the fold resolve each round-0 finding; (B) did the
fold introduce a NEW Critical/Important. Severity: Critical / Important / Minor
/ Nit, exactly.

**Artifacts:** round-0 report
`design/agent-reports/2026-08-10-r0-hang-fix-round0.md`; folded design
`design/DESIGN_b2b_hang_retention_and_bounded_read.md` at `4c373d9`; diff
`175cdd5..4c373d9` (242 insertions / 225 deletions, one file). Fold commit
message (`4c373d9`) carries the re-run build gate: `go build ./gui/... ./seal/...`
clean; `go test ./gui/op/ ./seal/ ./gui/` ok/ok/ok; `tinygo -target pico-plus2`
1,313,688 bytes flash vs b2b's 1,313,928 (240 smaller); FileReader-trim breaks
exactly `TestFileReaderNeverReturnsMoreThanTheRegion`, `./gui/` still ok. These
numbers were given as already machine-checked and are not re-derived here.

---

## (A) Per-finding verdicts

| # | Sev | Verdict | Basis |
| --- | --- | --- | --- |
| C1 | Critical | **RESOLVED** | Design title/status line and body now say fix B is "NOT retired" / "back on the table"; the whole `## Why B is no longer needed` section (the section that rested on the false inference) is deleted outright, not softened. The retraction is explicit and specific: "Any claim that fix D 'owns' the hang is unsupported, and the earlier retirement of fix B rested on exactly that claim." Same `e969839` Scrub evidence is now presented as *why D cannot be the cause* rather than as proof it is. A falsifiable acceptance criterion is stated in advance ("§What is still open"): in-use returns to ~144 K / ~688 allocs after 1 wipe **and** after 3 consecutive wipes; explicit "if D alone does not meet it, D is a bystander." This is genuinely falsifiable (concrete numbers, concrete procedure, concrete failure rule), not rhetorical. |
| I1 | Important | **RESOLVED** (at the design level; measurement itself is deferred, and the design says so) | "reachable vs merely uncollected" is no longer closed in favour of "reachable" — the design now calls the retainer "unidentified" and states the exact remedy round 0 asked for: force `runtime.GC()` before the readout, print `cap(ctx.B.args)`/`cap(ctx.B.refs)` at `run_flow.go:245`, re-take the three rows with fix D alone, plus the falsifiable criterion above. The actual re-measurement hasn't happened yet, but the design doesn't hide that — it names it as "the phase's open question" and puts the instrumented re-measurement first in "What is still open," which is the correct level of resolution for a design artifact (round 0's own "smallest fix" included "state a falsifiable acceptance criterion," which this does verbatim). |
| I2 | Important | **RESOLVED** | `boundBlob(region []byte) (int, error)` is hoisted into the untagged `seal/read.go`, quoting `read.go`'s own governing comment ("a bound placed only inside `read_tinygo.go` is never compiled by `go test`") as the reason. Both `XIPReader.Read` and (per prose + the fold commit's confirmed test run) `FileReader.Read` call it, so host and device now agree on returned length — the previous draft's asymmetry is gone. |
| I3 | Important | **RESOLVED** | "Callers" section explicitly retracts both wrong halves of the old claim: `cmd/sealread` is now correctly called the only on-target instrument (`tinygo build -target pico2`), its `len(b) > RegionLen` assertion at `main.go:112-123` is named, and the design states what it must be changed to print (`len(b)` against `52+pub+ct(+16)`, region bound checked separately). It also now flags the magic-present/header-invalid on-target diagnostic path going dead — a consequence round 0 raised. |
| I4 | Important | **RESOLVED** | All four sub-findings addressed in "Tests that can fail" item 2: (1) false-PASS path closed by requiring session 2 to draw fewer masks than session 1 **and asserting that relationship**; (2) canary constrained to an `op.Mask` source or `op.Input` tag, explicitly excluding `op.Image`/`op.Color` with the `op.go:315-318` citation; (3) canary must not be a `*bitmap.Face`, citing `glyphImage`'s package-level pin; (4) two `runtime.GC()` calls plus a channel-with-timeout instead of one. |
| I5 | Important | **RESOLVED** | "Must not allocate" dropped. New assertion: `errors.Is(err, ErrPubLen)` and a zero-length result, with the reasoning restated correctly — "the overflow guard is proven by the ordering, not by an allocation count," citing `wire.go:146`'s `fmt.Errorf` allocation. |
| I6 | Important | **RESOLVED** | Test 4 now names the mutant explicitly: "delete `d.Release()` from `run_flow.go`," notes every existing host test stays green under it by construction so only test 2 (with its false-PASS path now closed per I4) can kill it, and adds a second named row (`clear(d.maskStack[:cap])` → `clear(d.maskStack)`, killed by test 1). |
| M1 | Minor | **RESOLVED** | New subsection "Ordering — the reason corrected (round 0 M1)" states the old reason was false, explains why (`Release` frees nothing, `ctx.B` holds its own headers and is live across both calls, nothing is collectible in between), and restates the real constraint (both calls must follow the session's last `draw()`). |
| I? / M2 | Minor | **NOT ADDRESSED** | `boundBlob`'s `total > len(region)` branch still returns `ErrTooShort` (confirmed at the shown code, and matches `wire.go:67`'s own sentinel table where the analogous over-large condition in `ParseHeader` uses `ErrTooLarge`, `wire.go:78,202`). Round 0 flagged this branch as unreachable (given `clampRegion(RegionLen) == RegionLen` and `total ≤ 16450 < 65536`) and mis-sentineled. No text anywhere in the fold discusses it — not fixed, not acknowledged as deferred. |
| M3 | Minor | **RESOLVED** | Dedicated subsection "The doc comment this invalidates (round 0 M3)" states plainly that `read_tinygo.go:42-47`'s "nothing here may consult them" becomes false after fix C, and gives the corrected rule verbatim: "nothing may consult them before `ParseHeader` has validated them." (The shown `XIPReader.Read` code block itself doesn't carry an inline doc comment reflecting this — a presentation gap, not a substantive one, since the corrected wording is stated in prose immediately above.) |
| M4 | Minor | **NOT ADDRESSED** | "Not a secrecy bug" section is unchanged in substance from the pre-fold draft (confirmed by diff and by direct comparison): still argues only from `imageOp.args`/`refs` aliasing the array `Scrub` zeroes, still does not mention that `frameOp.op.src` and `inputOp.tag` are interface-value **copies**, not aliases, that `Scrub` cannot reach — and does not cite round 0's site-by-site verification (8 `op.Input`, 11 `op.Mask` sites) that established the conclusion holds despite the gap in the argument. |
| M5 | Minor | **RESOLVED** | Test 1 now states explicitly: "Must first assert `cap(d.maskStack) > 0` and `cap(d.inputs) > 0`," with the exact vacuous-pass scenario named (a frame with no mask/input ops gives `cap == 0`). |
| M6 | Minor | **RESOLVED** | New subsection "Altitude — answered (round 0 M6)" gives an explicit decision: call `Release()` at the session tail **and** make the natural truncation points self-maintaining (`clear` to cap at `op.go:249` inside `Draw`, and at `op.go:257` inside `Reset`), with the recursion's `op.go:369` truncation explicitly excluded and why (those entries alias the buffer being drawn right now). |
| M7 | Minor | **RESOLVED** | `Release`'s doc comment dropped the overstated "arrays alive for the process's lifetime," leaving "arrays alive." — matches round 0's correction that retention is high-water-bounded and transient, not permanent. Verified against the diff: this exact clause was removed and nothing replaced it with an equally overstated claim. |
| N1 | Nit | **RESOLVED** | Now recorded verbatim in "Lower-severity items recorded": `a.warnBuf` is long-lived and only `Reset()`, never `Scrub()`ed; content is non-secret, stated explicitly rather than left inferable. |
| N2 | Nit | **RESOLVED** | Now recorded verbatim in the same section: package `op`'s five `*ImageHandle` scratch objects are package-level and outlive every session by construction. |
| N3 | Nit | **NOT ADDRESSED** | No mention anywhere in the fold of the ≤16,450 worst-case vs. 1,421-byte largest-real-vector distinction, or the 46× (vs. stated 4×) actual reduction fix C achieves. Grepped for "46", "1,421", "4×" — no hits. Silently dropped, not contested. |

**Tally: 14 RESOLVED, 0 PARTIAL, 3 NOT ADDRESSED (M2, M4, N3), 0 REGRESSED.**
All three NOT ADDRESSED items are Minor/Nit — none block. Every Critical and
every Important (C1, I1–I6) is RESOLVED.

---

## (B) New Critical/Important introduced by the fold

**None found.** The three flagged risk areas were traced against the actual
pre-fix source (the fix isn't applied to this checkout; `seedhammer-b2b`@`6b828cf`
is pre-fold, matching what the design describes changing):

1. **`boundBlob(region[:HeaderLen])` handed a subslice of the XIP mapping.**
   Safe. `ParseHeader` (`seal/wire.go:120-206`) takes `buf []byte` as a
   parameter, never stores it, and `Header`'s only fields are value types
   (`uint32`, `[SaltLen]byte`, `[IVLen]byte`); the two places it captures bytes
   from `buf` are `copy(h.Salt[:], buf[16:32])` and `copy(h.IV[:], buf[32:44])`
   — copies into arrays, not retained slices. Confirmed no slice field exists
   on `Header` to alias into. The design's claim ("`ParseHeader` retains nothing
   from `buf`... so handing it a subslice of the XIP mapping is safe") is
   correct as verified against the source, not merely asserted.

2. **Self-maintaining `Draw` clearing `maskStack` to cap on every frame —
   correctness and cost; does it break the recursion's save/restore at
   `op.go:369`?** No regression. Traced `gui/op/op.go:245-260` (top-level
   `Draw`) against `:261-370` (the recursive `draw` helper): the clear-to-cap
   at `Draw`'s entry (`:249`) runs exactly once, as the *first* statement,
   strictly before `d.jumpStack` is seeded and before the recursive `d.draw`
   call begins. The recursion's own push/pop discipline (`origMaskStackLen :=
   len(d.maskStack)` at entry, `d.maskStack = d.maskStack[:origMaskStackLen]`
   at `:369`) is purely length-based — append and truncate only, never a read
   of any index `>= len(d.maskStack)`. So content beyond `len` (up to `cap`)
   is write-only from the recursion's point of view; clearing it before the
   recursion starts cannot change what the recursion observes. Cost: `cap`
   only grows (never shrinks) and reflects the frame's own historical
   high-water — round 0 independently estimated this at "tens of entries" for
   a typical screen; clearing tens of interface-slot writes once per `Draw`
   call is negligible next to a full-screen redraw on the target. Confirmed
   via source, not merely by trusting the design's claim.

3. **`Reset` also clearing `inputs` to cap — does it break `skipInputOps`
   reuse?** This was the reviewer brief's strongest candidate, and it is where
   the most work went. Traced the actual call graph in
   `gui/run_flow.go:88-176`: `d.Reset()` runs at line 89, as the *first*
   statement of the `draw` closure, which is called once per logical frame
   (twice in the source — the normal content path at `:124` and the
   wipe-warning path at `:209` — both call the same closure). Inside that
   closure, `d.Draw(fb, a.mask, content)` (`:105`) runs **once per framebuffer
   chunk** inside a `for { fb, ok := pl.NextChunk(); ...; d.Draw(...) }` loop
   (`:94-106`) — so `Draw` (and therefore `skipInputOps`) fires multiple times
   per single logical frame, one per chunk. `skipInputOps` exists precisely so
   chunk 2..N's redraw of the *same* op tree does not re-append the same input
   regions chunk 1 already recorded (`op.go:302-310`): chunk 1 runs with
   `skipInputOps == false` (just reset), populates `d.inputs`, then `Draw`
   sets `skipInputOps = true` (`op.go:253`) before returning; chunks 2..N see
   `skipInputOps == true` and skip the `opInput` append. `d.inputs` is then
   read later in the **same** frame, after the chunk loop completes, by
   `ctx.Router.Events(d, evts...)` (`run_flow.go:175`, consuming it via
   `d.TagBounds`/`d.Hit`, `op.go:536-550`, both of which iterate `d.inputs` by
   `len` only). The next frame's `draw()` call re-invokes `d.Reset()`
   (`:89`) *before* its own first `Draw`/chunk call. So the entire
   `skipInputOps` reuse window is bounded strictly within one
   `Reset()`-to-next-`Reset()` span — `Reset` is never called between chunks
   of the same frame — and nothing reads `d.inputs` from an *earlier* frame's
   window. Clearing the backing array to cap inside `Reset` only zeroes
   content that is already dead by construction (the previous frame's
   `Reset`-to-`Reset` window has already closed), exactly mirroring the
   `maskStack` pattern round 0 validated. **No regression.**

   One presentation nit worth naming (not scored, since it changes no
   behavior as specified): the design's decision text bundles both changes
   under "make `Draw` self-maintaining — clear to cap at `op.go:249` and
   `op.go:257`," but line 257 is inside `Reset`, a different method with a
   different call site and different timing guarantee than `Draw` itself. As
   *written* (by line number) it is correct and this is what was traced above.
   A future implementer who takes the "Draw self-maintaining" framing
   literally rather than the cited line number, and moves the `inputs` clear
   into `Draw` itself (run once per chunk instead of once per frame), *would*
   break `skipInputOps`: chunk 2's `Draw` call would wipe chunk 1's already-
   collected `d.inputs` before `Router.Events` ever reads it, since
   `skipInputOps` prevents chunk 2 from repopulating. This ambiguity
   originates in round 0's own Q3 answer (which proposed the same "op.go:249
   and the same for inputs at op.go:257" framing) and the fold adopted it
   verbatim rather than sharpening it — worth a one-line tighten
   ("clear `maskStack` at `Draw`'s entry; clear `inputs` at `Reset`'s entry")
   before implementation, but not a defect in the design as written.

4. **`FileReader` returning a trimmed slice.** Per the already-machine-checked
   fact in the brief (host+device length agreement breaks exactly
   `TestFileReaderNeverReturnsMoreThanTheRegion`, `./gui/` still green) and the
   design's own text ("`TestFileReaderNeverReturnsMoreThanTheRegion`... must be
   updated to the bounded length — it is the test that would otherwise
   contradict fix C"), the design correctly identifies the one affected test
   and no other caller assumption is disturbed. Not re-derived, per
   instructions; nothing in the fold's text contradicts or overstates this.

**No new Critical or Important.**

---

## Summary

- Per-finding verdicts: **14 RESOLVED / 0 PARTIAL / 3 NOT ADDRESSED (M2, M4, N3 — all Minor/Nit) / 0 REGRESSED.**
- Every Critical and Important from round 0 (C1, I1–I6) is RESOLVED.
- New Critical/Important introduced by the fold: **0.**
- Not blocking: the design is GREEN on the two gating questions this review
  covers. The three NOT ADDRESSED Minors/Nits (M2's wrong `ErrTooShort`
  sentinel on an unreachable branch, M4's incomplete secrecy argument, N3's
  46×-not-4× framing) and the one presentation nit on the `Draw`/`Reset`
  phrasing are worth a cheap follow-up fold but do not gate implementation.
