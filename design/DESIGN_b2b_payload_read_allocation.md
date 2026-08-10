# Design note — the payload read's 64 KiB allocation, and the post-wipe hang

**Status:** proposal, pre-R0. Written 2026-08-09 from a hardware Critical.
**Owns:** the B2b hang recorded in `HARDWARE_RESULT_2026-08-09_phaseB2b.md`.

## The defect, established by experiment

| sequence on the real machine | result |
| --- | --- |
| enter → exit Sealed Payload, repeatedly | works |
| full unlock → **normal** exit → re-enter | works |
| full unlock → **wipe** → re-enter | **hangs, deterministically** |

The hang is total: no redraw, no touch, no screensaver at 3:00, no §10.2.4 timer.
Those three share one goroutine, so all of them stopping is a single fact — the
flow stopped calling `ctx.Frame`. The **pressed-button frame drew**, so the click
was processed and `Run` had not returned; the failure is the *next* thing, which
is `XIPReader.Read`'s allocation.

**A wipe is required.** That rules out the NFC-shutdown deadlock as primary — that
defer runs on every program entry and completed many times — and rules out a
single blob alloc/free cycle, which entering and exiting already performs.

## Why a wipe and not a normal exit

A normal exit **reuses the `Context`**. A wipe **abandons it and builds a fresh
one**, stranding:

- the old `ctx.B`, grown across a whole session including the rendered seed screen
- `a.warnBuf`, grown during the 30-second warning
- `a.mask`

Then re-entry asks for **64 KiB contiguous** under `-gc precise`, which is
**non-moving**. On ~440 KiB of heap, after a session's churn, that is not a
reliable request.

**This is B2b's own design decision.** The plan says: *"A fresh `Context`, not a
scrubbed one … a wipe is rare enough that the allocation is irrelevant."* Round 0
accepted it. The reasoning weighed the allocation's **cost** and never considered
the **garbage it strands**, nor that a non-moving collector plus a large
contiguous demand makes fragmentation a **correctness** problem.

## The number that reframes the fix

`seal/wire.go:192` already records the format's own arithmetic:

> the largest total they admit is **52 + 8191 + 8191 + 16 = 16,450** — well under
> 65536

`XIPReader.Read` allocates **65,536** unconditionally. That is **~4× more than
anything the format can legally contain.** The contiguous demand is not justified
by the payload; it is the flash **region size** being used as an **allocation
size**.

## Options

### A — allocate the 64 KiB once at boot and reuse it

Removes the contiguous allocation permanently.

- **For:** simplest to reason about; wipe becomes a `clear()` at a fixed address,
  which is *more* deterministic than today's abandoned heap buffer.
- **Against:** reserves 64 KiB of ~440 KiB heap for the process lifetime, to hold
  a payload that can never exceed 16,450 bytes. Cuts against F-79, whose whole
  argument is that the region must not stay resident.

### B — reuse the `Context` across sessions

Reset it in place instead of allocating a fresh one, including the
`Router.pointer` field that motivated the fresh allocation in the first place.

- **For:** smallest change; directly removes the garbage this wipe strands.
- **Against:** treats *this* trigger, not the class. The 64 KiB demand stays
  exposed to any other fragmentation — a long session, a big plate, a future
  feature.

### C — bounded read *(recommended)*

Two stages:

1. `hasMagic` on the XIP region, as today.
2. Copy the **fixed 52-byte header** (`HeaderLen`) out of XIP and `ParseHeader`
   it — which is what **validates** `pub_len` and `ct_len` in `uint64` arithmetic
   against `MaxSectionLen`.
3. Allocate exactly `HeaderLen + pub_len + ct_len + TagLen` — **≤16,450** — and
   copy that.

- **For:** attacks the root. A ≤16 KiB allocation is far likelier to succeed
  against a fragmented heap than 64 KiB, and it holds no more than the payload
  actually is. No static reservation, no change to residency lifetime, and F-79's
  argument is preserved intact.
- **Against:** touches §6.2-adjacent parsing on a funds path, so it needs the full
  gate. And the ordering is load-bearing — see below.

## The constraint C must respect

`seal/read_tinygo.go`'s own comment is explicit:

> *"The bound is the region constant alone. The header's own lengths are
> attacker-controlled and are checked by `ParseHeader`, but that happens AFTER
> this read, so nothing here may consult them."*

C does **not** violate that, but only if it is a genuine two-stage read:
stage 2's copy is bounded by `HeaderLen`, a **constant**; the attacker-controlled
lengths are consulted **only after `ParseHeader` has validated them**, in
`uint64`, against `MaxSectionLen`. Any implementation that reads the lengths
before validating them reintroduces exactly the overflow `unlock_key.go:44-58`
documents — where `int(uint32)` **reinterprets** on a 32-bit target and makes a
length negative.

**This is the part an R0 reviewer should attack hardest.**

## Recommendation

**C, and consider B alongside it.** C removes the class; B removes the specific
garbage the wipe strands and is nearly free. A is the fallback if C's ordering
cannot be made safe, and it should not be chosen merely because it is easy — it
is the only option that makes residency *worse*.

## Open questions for R0

1. Does anything else depend on `Read()` returning a full-region slice? The
   comment says the caller keeps the bytes across a flash write; a shorter slice
   is still a copy, but callers should be checked rather than assumed.
2. Is `hasMagic` safe to run on the XIP region before the bounded copy? It reads
   a fixed 8 bytes and looks safe, but it is the one thing touching XIP directly.
3. Should the wipe path also drop `a.warnBuf` and reset `a.mask`, or is B enough?
4. **Does the fix want a test that can fail?** The host cannot reproduce the OOM
   (gigabytes of RAM). The honest answer may be that this is verified on hardware
   only — in which case say so in the plan rather than inventing a host test that
   proves nothing. See `…-hang-repro.md`, which established the host's limits.
