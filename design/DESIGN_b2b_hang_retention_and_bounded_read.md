# The post-wipe hang — a verified retention path, a bounded read, and what is still open

**Status:** design, R0 round 1 (round 0 folded). Written 2026-08-10.
**Round 0 review:** `design/agent-reports/2026-08-10-r0-hang-fix-round0.md` —
1 Critical, 6 Important, 7 Minor, 3 Nit. All folded here.
**Relationship to `DESIGN_b2b_payload_read_allocation.md`:** its fix C survives,
restructured. **Its fix B is NOT retired** — round 0's Critical showed the
argument for retiring it was unsound. See "What is still open".

## The defect, reproduced with numbers

| sequence on the machine | result |
| --- | --- |
| enter → exit Sealed Payload, repeatedly | works |
| full unlock → **normal** exit → re-enter | works |
| full unlock → **wipe** → re-enter | **hangs, deterministically** |

2026-08-10, with the idle probe attached, the last drawn frame read:

```
idle 0s w151 t7948 e138 - Pd459,252
```

`Pd459,252` is a **press**. The release never drew. So the flow rendered the
pressed checkmark and never called `ctx.Frame` again. The idle timer, the
screensaver and touch routing all stop together because all three live in
`runWithFlow`'s inner loop, which runs only inside `ctx.FrameCallback` — three
symptoms, **one fact**: the flow blocked inside the dispatch. The event loop,
the platform timer and the idle clock were all independently exonerated on
hardware the same day.

## A verified retention path in `op.Drawer` — real, and smaller than the hang

```go
type imageOp struct {
	src  any
	refs []any        // aliases the Buffer's refs array
	args []uint32     // aliases the Buffer's args array
}
```

`Drawer.Draw` (`gui/op/op.go:249`) begins `d.maskStack = d.maskStack[:0]`. That
truncates the **slice**; the backing array keeps every stale `frameOp`, and a
mark-sweep collector scans **whole allocated objects**, not up to `len`. So those
slice headers keep the arrays behind them alive. `Drawer.Reset()` clears only
`inputs`, and only by truncation. The drawer is allocated **outside** the session
loop (`gui/run_flow.go:42`), so it survives a wipe.

**This is real and worth fixing. It is NOT big enough to be the hang's cause,
and round 0's Critical is what established that.** The measured build
(`e969839`) already contained `ctx.B.Scrub()` — verified:

```
$ git show e969839:gui/run_flow.go | grep -n Scrub
245:			ctx.B.Scrub()
$ git show e969839:gui/op/buffer_len.go | grep -n clear
24:	clear(b.args[:cap(b.args)])
25:	clear(b.refs[:cap(b.refs)])
```

So **every slot of the abandoned buffer's `refs` array was already nil** at the
instant of the measurement. And `Drawer` stores no `*Buffer` and no `*Context` —
its fields are `maskStack`, `jumpStack`, `inputs`, `skipInputOps`, `text`, and
`draw` takes `buf *Buffer` as a **parameter** it never assigns (`op.go:251,261`).

Therefore the Drawer can retain **two allocations** — the `args []uint32` and
`refs []any` backing arrays, contents nil — plus interface copies in
`frameOp.op.src` and `inputOp.tag`, all of which round 0 resolved to
package-level `assets.*` images and pointer-free `*Clickable`s.

**Order of magnitude: ~24 KB in 2 objects, against a measured 214 KB in 1,567.**

| moment | in use | free | live allocs |
| --- | --- | --- | --- |
| fresh boot | 144 K | 301 K | 688 |
| unlock → **normal** exit | 144 K | 301 K | 688 |
| unlock → **wipe** | **358 K** | **87 K** | **2255** |

**The retainer of the other ~1,565 objects is unidentified.** Any claim that
fix D "owns" the hang is unsupported, and the earlier retirement of fix B rested
on exactly that claim.

### Not a secrecy bug

`imageOp.args` aliases the same array `Scrub` zeroes, so scrubbing reaches what
the alias can see. **`Scrub` stays regardless:** once an allocation is genuinely
freed, TinyGo may reissue it without zeroing.

## Fix D — release the drawer's references

```go
// Release drops every reference this drawer holds into a Buffer's backing
// arrays. Truncation does not: the collector scans whole allocated objects, so
// a stale frameOp beyond len keeps that session's args and refs arrays alive.
//
// Reslicing to cap before clear() is the point -- clear(d.maskStack) would zero
// only the live prefix and leave exactly the stale entries that leak.
func (d *Drawer) Release() {
	clear(d.maskStack[:cap(d.maskStack)])
	clear(d.inputs[:cap(d.inputs)])
	d.maskStack = d.maskStack[:0]
	d.inputs = d.inputs[:0]
	d.jumpStack = d.jumpStack[:0]
	d.text = nil
	d.skipInputOps = false
}
```

`jumpStack` is `[]ops` — three ints, no pointers — so truncation suffices; listed
to make the audit total.

### Altitude — answered (round 0 M6)

Production instantiates exactly **one** `op.Drawer` (`run_flow.go:42`), and
`run_flow.go:233`'s `if !wiping { return }` means **the wipe path is the only
path that abandons a Buffer while keeping the Drawer**. On a normal exit
`runWithFlow` returns and `d` is garbage with everything else. So the session
tail covers 100% of production abandonments **today**.

The invariant "a `Drawer` must not outlive the Buffers it drew" is undocumented
and unenforced in package `op`, and `Release` is a call a future author must
remember. **Decision: call `Release()` at the session tail AND make `Draw`
self-maintaining** — `clear` to cap at `op.go:249` and `op.go:257` — so the leak
is structurally unreachable for any new abandonment site. The recursion's
truncation at `op.go:369` must **not** be cleared: those entries alias the buffer
being drawn right now.

### Ordering — the reason corrected (round 0 M1)

The previous draft said `Scrub` must precede `Release` because "scrub zeroes the
secret while it is still reachable, then release makes it collectable." **That
reason is false.** `Release` frees nothing; `ctx.B` still holds its own headers
to the same two arrays and `ctx` is live across both calls, so nothing can be
collected in between and the two lines are behaviourally interchangeable. The
real constraint is that **both must follow the session's last `draw()`**, which
`run_flow.go:245` satisfies. Recorded because a false reason of this shape gets
copied into a security argument — and invites the opposite error, that `Release`
subsumes `Scrub`.

## Fix C — the bounded read, hoisted where tests can reach it

`RegionLen = 65_536` (`seal/wire.go:50`) is used as an **allocation size** while
the format's caps admit at most `52 + 8191 + 8191 + 16` = **16,450**.

**Round 0's I2: the previous draft put the bound inside `read_tinygo.go`, which
`seal/read.go`'s own header forbids** — and the draft quoted a *different*
comment from that file as its governing rule:

> *This file is UNTAGGED on purpose. The `RegionLen` bound lives here, in
> `clampRegion`, and is called by BOTH the host and the TinyGo implementation, so
> that a host `go test` can kill the unbounded-read mutant. A bound placed only
> inside `read_tinygo.go` is never compiled by `go test` and no automated test
> can reach it.*

So the bound goes in the **untagged** `read.go`, beside `clampRegion`, and both
readers call it:

```go
// boundBlob reports how many bytes of region the header declares, so a reader
// allocates the payload's real size instead of the region's.
//
// UNTAGGED and shared, for this file's own stated reason: a bound inside
// read_tinygo.go is never compiled by `go test`.
//
// The ORDER is the safety argument. ParseHeader is handed a HeaderLen-bounded
// slice -- a CONSTANT bound -- and rejects pub_len or ct_len above
// MaxSectionLen (wire.go:145,148). Only after it returns nil may either length
// be consulted for arithmetic: both are then proven <= 8191, so int() cannot
// reinterpret and the sum cannot wrap a 32-bit int. Reading them earlier
// reintroduces the negative-length overflow unlock_key.go:44-58 documents.
func boundBlob(region []byte) (int, error) {
	if len(region) < HeaderLen {
		return 0, ErrTooShort
	}
	h, err := ParseHeader(region[:HeaderLen])
	if err != nil {
		return 0, err
	}
	total := HeaderLen + int(h.PubLen) + int(h.CtLen)
	if h.Sealed() {
		total += TagLen
	}
	if total > len(region) {
		return 0, ErrTooShort
	}
	return total, nil
}
```

It returns a **length, not a slice**, so it allocates nothing and each reader
owns its own copy. `ParseHeader` retains nothing from `buf` — `Header`'s fields
are values (`[SaltLen]byte`, `[IVLen]byte`, `uint32`) — so handing it a subslice
of the XIP mapping is safe.

```go
func (XIPReader) Read() ([]byte, error) {
	region := unsafe.Slice((*byte)(unsafe.Pointer(uintptr(PayloadAddr))), clampRegion(RegionLen))
	if !hasMagic(region) {
		return nil, ErrNoPayload
	}
	n, err := boundBlob(region)
	if err != nil {
		return nil, err
	}
	out := make([]byte, n)
	copy(out, region[:n])
	return out, nil
}
```

`FileReader.Read` takes the same trim after reading the region, so **host and
device return the same length** — which the previous draft would have broken.

### The doc comment this invalidates (round 0 M3)

`read_tinygo.go:42-47` currently says *"nothing here may consult them"*. After
fix C that is false as written, and a future reader "restoring" the invariant
would delete the bound. It must be rewritten to state the real rule: *nothing may
consult them before `ParseHeader` has validated them.*

### Callers — corrected (round 0 I3)

- `gui/unlock_flow.go:38` — holds `blob`, zeroes it at exit via a closure. A
  shorter slice is still exactly what was read, so `clear(blob)` stays correct,
  and the comment's worry about "pinning all 65,536 bytes for the whole flow"
  shrinks to ≤16,450. **Strengthens F-79.**
- `cmd/sealread/main.go:102` — **not** "a host tool with no length assumption",
  as the previous draft claimed. Both halves were wrong: it is built
  `tinygo build -target pico2` (its own header), so it is the only **on-target**
  instrument that can observe fix C, and it **does** assert length at
  `main.go:112-123`, printing `OVER-LONG — clampRegion FAILED on target` when
  `len(b) > RegionLen`. After fix C that branch is unreachable for every
  header-valid payload, so the tool silently stops testing what it exists to
  test. It must be updated to print `len(b)` against `52+pub+ct(+16)`, and to
  check the region bound separately via `boundBlob`'s caller. Note also that a
  magic-present-but-header-invalid region — the XIP-aliasing case its own results
  section documents — now returns an error from `Read`, so its 8-byte dump and
  on-target `ParseHeader` diagnostic at `:127-134` would never run.

## What is still open — and the measurement that closes it

**Fix B is back on the table.** Round 0's Critical is that D cannot account for
the measurement, so "B is retired" was inferred from a claim that does not hold.

**Round 0's I1: "reachable vs merely uncollected" was closed without data.**
`mem.Mallocs - mem.Frees` counts what the runtime has **swept**, not what is
**reachable**; a readout taken without forcing a collection cannot tell them
apart. If the 1,567 objects were merely uncollected, D changes nothing, C's
smaller request happens to succeed, the hang looks fixed, and it returns on the
next feature that raises the watermark.

**The instrumented re-measurement, before any further design:**

1. Force `runtime.GC()` immediately before the heap readout in the diagnostic
   build. Without this the three-row table cannot distinguish the two cases.
2. Print `cap(ctx.B.args)` and `cap(ctx.B.refs)` at `run_flow.go:245`, so D's
   recoverable footprint is a number rather than an estimate.
3. Re-take the three rows with **fix D alone**.

**Falsifiable acceptance criterion**, stated in advance so the result cannot be
rationalised: *after one wipe, and again after three consecutive wipes, in-use
returns to ~144 K and live allocs to ~688.* **Three cycles, not one** — at 214 KB
stranded per wipe, a single-cycle pass proves nothing about the second. If D
alone does not meet it, D is a bystander, and B — or the still-unidentified
retainer — owns the hang.

## Tests that can fail

1. **White-box `Release` (`gui/op`).** Draw a frame, `Release()`, assert every
   element of `d.maskStack[:cap]` and `d.inputs[:cap]` is the zero value. Kills
   the `clear(d.maskStack)`-without-`[:cap]` mutant. **Must first assert
   `cap(d.maskStack) > 0` and `cap(d.inputs) > 0`** (round 0 M5) — on a frame
   with no mask and no input ops both caps are 0, `[:0]` is empty, and the
   assertion passes on an untouched Drawer.
2. **Reachability, end to end (`gui`).** A canary reachable only through session
   1's buffer, a wipe into session 2, then force collection and assert the
   finalizer ran. Round 0's I4 found a **false-PASS path and three construction
   traps**; all four constrain the test:
   - **Session 2 must draw FEWER mask ops than session 1**, and the test must
     assert that relationship. `Draw` re-appends from index 0, so a session-2
     frame with at least as many masks overwrites the canary's slot and the
     finalizer runs **without fix D**.
   - The canary must be an **`op.Mask` source or an `op.Input` tag** —
     `op.go:315-318` appends to `maskStack` only when `typ != opImage`, so an
     `op.Image`/`op.Color` canary never enters the structure under test.
   - It must **not** be a `*bitmap.Face`: `glyphImage`'s closure
     (`op/image.go:32-38`) assigns into a package-level object and pins it
     forever, so a glyph canary fails after the fix for an unrelated reason.
   - **Two `runtime.GC()` calls plus a channel with a timeout**, not one `GC()` —
     the finalizer goroutine must be scheduled, or the test is flaky-red.
3. **Bounded read (`seal`), against `boundBlob` directly** — reachable from host
   `go test` precisely because I2's hoist put it in the untagged file. For each
   vector assert `boundBlob(region) == 52 + PubLen + CtLen (+16 when sealed)`.
   For a hostile header with `pub_len = 0xFFFFFFFF`, assert
   `errors.Is(err, ErrPubLen)` and a zero length. **Not "must not allocate"**
   (round 0 I5): `ParseHeader`'s reject path is `fmt.Errorf` (`wire.go:146`) and
   allocates a formatted string and a `wrapError`. The overflow guard is proven
   by the **ordering**, not by an allocation count.
   `TestFileReaderNeverReturnsMoreThanTheRegion` asserts `len(got) == RegionLen`
   and **must be updated** to the bounded length — it is the test that would
   otherwise contradict fix C.
4. **The mutation row that matters** (round 0 I6): *delete `d.Release()` from
   `run_flow.go`*. Every existing host test stays green under it by construction,
   so its only killer is test 2 — which means test 2's false-PASS path must be
   closed before this row means anything. A second row: `clear(d.maskStack[:cap(d.maskStack)])`
   → `clear(d.maskStack)`, killed by test 1.

## Gate coverage — what is machine-verified

`plan-build-gate-go.sh` reaches these blocks at **TIER 2 (syntax) only**, so both
fixes were applied to a scratch fork and put through both toolchains. Round 0's
numbers are superseded by the re-run recorded in this fold's commit message,
because **the fold changed the code shape** (`boundBlob` hoisted into the untagged
`read.go`, called by both readers) and a fold re-earns the gate.

The TinyGo build is the one that matters for fix C: `read_tinygo.go` is behind
`//go:build tinygo`, so a host build never compiles it.

**Where review budget belongs — what tools still cannot reach:** whether the
instrumented re-measurement above is the right experiment; whether `boundBlob`'s
ordering survives a hostile header; whether test 2's false-PASS path is really
closed by the depth constraint; and the **unidentified retainer of ~1,565
objects**, which is now the phase's open question rather than a settled one.

## Lower-severity items recorded

- **N1** — `a.warnBuf` is a long-lived Buffer that is never `Scrub`ed. Its
  content is non-secret (`warningSubject()` returns constants), but a design that
  enumerates what survives a wipe should say so rather than leave it inferred.
- **N2** — package `op`'s five `*ImageHandle` scratch objects are package-level
  and outlive every session by construction; not a leak, worth naming in the same
  inventory.
