# The post-wipe hang — root cause, and the two fixes it actually needs

**Status:** design, pre-R0. Written 2026-08-10.
**Supersedes** the options analysis in `DESIGN_b2b_payload_read_allocation.md`:
its **fix B is retired** (see "Why B is no longer needed"). Its fix C survives
with its ordering constraint intact.
**Owns:** the hang recorded in `HARDWARE_RESULT_2026-08-09_phaseB2b.md`,
reproduced again 2026-08-10 with an instrument attached.

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
`runWithFlow`'s inner loop, and that loop only runs inside `ctx.FrameCallback` —
so three symptoms are **one fact**: the flow blocked inside the dispatch.

The same session had already proved the event loop healthy: the §10.2.4 window
opened and wiped on schedule with `e` unchanged across 3:30.

## Root cause: a slice-truncation leak in `op.Drawer`

Found by the retention recon agent, then verified line by line against the
source.

```go
type imageOp struct {
	src  any
	refs []any        // aliases the Buffer's refs array
	args []uint32     // aliases the Buffer's args array
}

type frameOp struct {
	pos image.Point
	op  imageOp
}
```

`Drawer.Draw` (`gui/op/op.go:249`) begins:

```go
d.maskStack = d.maskStack[:0]
```

That truncates the **slice**. The backing array keeps every stale `frameOp`, and
each one holds slice headers pointing into the buffer of the session that drew
it. A mark-sweep collector scans **whole allocated objects**, not up to `len`, so
those pointers are still followed and the arrays behind them can never be freed.

`Drawer.Reset()` (`gui/op/op.go:256`) clears only `inputs`, and only by
truncation:

```go
func (d *Drawer) Reset() {
	d.inputs = d.inputs[:0]
	d.skipInputOps = false
}
```

**The drawer is deliberately allocated outside the session loop**
(`gui/run_flow.go:42`, `d := new(op.Drawer)`), so it survives a wipe — and pins
the buffer of the session just abandoned.

**This explains the asymmetry exactly.** A normal exit *reuses* the Context, so
the drawer's stale headers point at a buffer still in use and nothing leaks. A
wipe builds a **fresh** Context, and the old buffer becomes unreachable to the
program but permanently reachable to the collector.

The measurement matches:

| moment | in use | free | live allocs |
| --- | --- | --- | --- |
| fresh boot | 144 K | 301 K | 688 |
| unlock → **normal** exit | 144 K | 301 K | 688 |
| unlock → **wipe** | **358 K** | **87 K** | **2255** |

One stale `frameOp` is enough. `maskStack`'s capacity is irrelevant to the leak.

### Why this is a memory bug and not a secrecy bug

`imageOp.args` aliases *the same array* `ctx.B.Scrub()` zeroes, so scrubbing
still reaches the bytes the alias can see. **`Scrub` stays** regardless: once the
allocation is genuinely freed, TinyGo may hand it out again without zeroing, so
zero-then-release remains the correct order and the correct pair.

## Fix D — release the drawer's references at the session boundary

New method in `gui/op`:

```go
// Release drops every reference this drawer still holds into a Buffer's backing
// arrays. Truncation does not do it: the collector scans whole allocated
// objects, so a stale frameOp beyond len keeps that session's args and refs
// arrays alive for the process's lifetime.
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

`jumpStack` is `[]ops`, three ints and no pointers, so truncation suffices; it is
listed to make the audit total rather than because it leaks.

Called at the session tail in `gui/run_flow.go`, immediately after the existing
scrub:

```go
ctx.B.Scrub()
d.Release()
```

**Order is deliberate:** scrub zeroes the secret while it is still reachable,
then release makes it collectable. Reversed, the array could be freed and reused
with the twelve words still in it.

## Fix C — the bounded read

`seal/read_tinygo.go` asks for the flash **region size** as an **allocation
size**:

```go
out := make([]byte, len(region))   // 65,536, unconditionally
```

`RegionLen = 65_536` (`seal/wire.go:50`), while the format's own caps admit at
most `HeaderLen + 2*MaxSectionLen + TagLen` = `52 + 8191 + 8191 + 16` = **16,450**
— verified against the constants, not read off the comment. So the code demands
**4× more, contiguous, than anything legal can occupy**, on a non-moving
collector.

```go
func (XIPReader) Read() ([]byte, error) {
	region := unsafe.Slice((*byte)(unsafe.Pointer(uintptr(PayloadAddr))), clampRegion(RegionLen))
	if !hasMagic(region) {
		return nil, ErrNoPayload
	}
	// STAGE 1. HeaderLen is a CONSTANT, so nothing attacker-controlled bounds
	// this copy. clampRegion already guarantees the length, but a region
	// shorter than a header is checked rather than assumed.
	if len(region) < HeaderLen {
		return nil, ErrTooShort
	}
	var hdr [HeaderLen]byte
	copy(hdr[:], region[:HeaderLen])

	// STAGE 2. ParseHeader validates PubLen and CtLen against MaxSectionLen
	// (seal/wire.go:145-151) BEFORE either is consulted for arithmetic. This
	// ordering is the whole safety argument -- see below.
	h, err := ParseHeader(hdr[:])
	if err != nil {
		return nil, err
	}

	// STAGE 3. Safe in int now, and only now: both lengths are proven <= 8191,
	// so the sum cannot exceed 16,450 and cannot wrap a 32-bit int.
	total := HeaderLen + int(h.PubLen) + int(h.CtLen)
	if h.Sealed() {
		total += TagLen
	}
	if total > len(region) {
		return nil, ErrTooShort
	}
	out := make([]byte, total)
	copy(out, region[:total])
	return out, nil
}
```

### The constraint this must respect, and why it is satisfied

`seal/read_tinygo.go`'s own comment states the rule:

> *The bound is the region constant alone. The header's own lengths are
> attacker-controlled and are checked by `ParseHeader`, but that happens AFTER
> this read, so nothing here may consult them.*

The two-stage form **honours** it rather than evading it: stage 1 is bounded by a
constant; the attacker-controlled lengths are consulted only in stage 3, after
`ParseHeader` has rejected anything above `MaxSectionLen`. Verified in the
source:

```go
if pubLen > MaxSectionLen { return Header{}, ... }   // seal/wire.go:145
if ctLen  > MaxSectionLen { return Header{}, ... }   // seal/wire.go:148
```

Any rearrangement that reads the lengths before that check reintroduces the
`int(uint32)` reinterpretation `unlock_key.go:44-58` documents, where a length
goes **negative** on a 32-bit target. **This is what R0 should attack hardest.**

### Callers

Two, both checked:

- `gui/unlock_flow.go:38` — takes `blob`, and zeroes it at exit via a closure. A
  shorter slice is still exactly what was read, so `clear(blob)` stays correct.
  The existing comment there worries about "pinning all 65,536 bytes for the
  whole flow"; fix C reduces that to ≤16,450 and **strengthens F-79's argument**.
- `cmd/sealread/main.go:102` — host tool, no length assumption.

## Why B is no longer needed

`DESIGN_b2b_payload_read_allocation.md` proposed reusing the `Context` across
sessions. With the retainer identified, B addresses a **symptom of D**: the
garbage exists because the drawer pins it, not because a fresh Context is
inherently wasteful.

B was also the invasive option. It would have required resetting `Done`,
`Wakeup`, `keepAwake` and `Router.pointer.pressedTag` by hand, and it would have
weakened the property B2b chose deliberately — *"a fresh Context, not a scrubbed
one"* — which is wipe hygiene, not an accident. **Retired.**

## Tests that can fail

The hang itself was never host-reproducible. **The retention is**, and that is
the point of naming the mechanism.

1. **White-box, deterministic — `gui/op`.** Draw a frame, then `Release()`, then
   assert every element of `d.maskStack[:cap]` and `d.inputs[:cap]` is the zero
   value. Kills the `clear(d.maskStack)`-without-`[:cap]` mutant, which is the
   easy way to write this wrong.
2. **Reachability, end to end — `gui`.** Attach a `runtime.SetFinalizer` to an
   object reachable only through the session-1 buffer's `refs`, drive
   `runWithFlow` through a wipe into session 2, then `runtime.GC()` and assert
   the finalizer ran. **Before fix D this must fail**, which is what makes it a
   test rather than a decoration. Host GC, not TinyGo's — it proves
   *reachability*, which is the property, and reachability is not
   collector-specific.
3. **Bounded read — `seal`.** For each vector, assert `len(Read()) == 52 +
   PubLen + CtLen (+16 when sealed)` and that the bytes equal the region prefix.
   Plus a hostile header whose `pub_len` is `0xFFFFFFFF`: `Read` must return
   `ParseHeader`'s error and **must not allocate**, which is the overflow guard.
4. **Mutation rows** for both, since B2a-ii's dominant defect class was tests
   that could not fail.

## What R0 should attack

1. **Fix C's ordering.** Does any path consult `PubLen`/`CtLen` before
   `ParseHeader` returns nil? Does `total` overflow on a 32-bit `int` for any
   input `ParseHeader` accepts?
2. **Is the session boundary the right altitude for `Release()`?** If any *other*
   long-lived object aliases a `Buffer` the same way, fixing it at the session
   tail is treating one instance of a class. The agent swept
   `EventRouter.pointer.pressedTag`, 14 `*Hook` vars, five NFC goroutines, the
   engrave job and the `a` struct and ruled each out — **that sweep is the thing
   to check**, since a missed retainer leaves the bug with a smaller footprint
   and no symptom until the heap is tight again.
3. **Does `Release()` on the wipe path only leave the leak alive on the normal
   path?** A normal exit reuses the Context, so nothing leaks — but that is an
   argument about today's code, not an invariant. Should `Release` be called
   whenever a Buffer is abandoned, wherever that happens?
4. **Is fix C sufficient alone, making D optional?** No — D is the correctness
   fix; C reduces the demand. But the converse deserves scrutiny: with D alone,
   is a 64 KiB contiguous run reliable at 301 KiB free? "Probably" is not the
   standard on a funds path.
5. **`d.text = nil` vs truncation** — `[]rune` holds no pointers, so this is
   about the allocation, not references. Is dropping it correct given
   `ExtractText` is test-only?

## Gate coverage — what is machine-verified, and what is not

`plan-build-gate-go.sh` reaches these blocks at **TIER 2 only** (7/8 parse; the
eighth is a two-line quotation from `wire.go` with no file anchor), because they
are modifications rather than whole new files. **TIER 2 proves syntax, never
semantics** — so both fixes were additionally applied to a scratch copy of the
fork and put through both toolchains:

```
$ go build ./gui/... ./seal/...                      (clean)
$ go test ./gui/op/ ./seal/
  ok  seedhammer.com/gui/op   1.218s
  ok  seedhammer.com/seal    25.400s
$ tinygo build -target pico-plus2 -gc precise … ./cmd/controller
     code    data     bss |   flash     ram
  1283228   30444   30140 | 1313672   60584
```

The TinyGo build is the one that matters for fix C: `seal/read_tinygo.go` is
behind `//go:build tinygo`, so a host build never compiles it at all. It now
type-checks, and the firmware comes out **256 bytes SMALLER** than b2b's
1,313,672 vs 1,313,928 — the bounded read is not a size cost.

**So the reviewer does not need to be a compiler.** What remains genuinely
un-machine-checked, and is where review budget belongs:

- whether the **ordering** in fix C is sound against a hostile header (a type
  checker cannot see an overflow that only a validated bound prevents);
- whether the retention **sweep was complete**, i.e. whether any other
  long-lived object aliases a `Buffer`;
- whether the tests proposed below **can actually fail**;
- whether the session tail is the right **altitude** for `Release()`.

Resolved against the source and quoted from it, not paraphrased: `RegionLen`,
`HeaderLen`, `MaxSectionLen`, `TagLen`, the two `ParseHeader` cap lines, both
`Read` callers, and the `imageOp`/`frameOp`/`Drawer` field lists.
