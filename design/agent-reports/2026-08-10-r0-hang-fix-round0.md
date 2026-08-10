# R0 round 0 — `DESIGN_b2b_hang_retention_and_bounded_read.md`

**Reviewer:** independent architect agent (opus), 2026-08-10.
**Artifact:** `design/DESIGN_b2b_hang_retention_and_bounded_read.md`
**Code read:** `seedhammer-b2b` @ `6b828cf` (branch `b2b`), plus `e969839`
(`b2b-heapprobe`, the build that produced the 358 K / 2255 measurement).
**Verdict:** **1 Critical, 6 Important, 7 Minor, 3 Nit.** Not GREEN.

The single Critical is not in either fix's code. Both fixes are, as far as I can
tell, correct code. The Critical is that **the design's root-cause claim is
refuted by the design's own build**, and fix B was retired on the strength of it.

---

## Q1 — Fix C's ordering and arithmetic

**No Critical and no Important on the arithmetic or the ordering.** Stated
explicitly because the design nominated this as "what R0 should attack hardest";
it survives.

Each sub-question, answered against the source:

**Is there any input `ParseHeader` accepts for which `total` misbehaves on a
32-bit `int`?** No. `ParseHeader` returns `err == nil` only past
`wire.go:145` (`pubLen > MaxSectionLen` → reject) and `wire.go:148` (`ctLen >
MaxSectionLen` → reject), and `Header.PubLen`/`CtLen` are assigned from exactly
those two locals at `wire.go:205`. Nothing else in the package constructs a
`Header` from a blob. So on a nil error both are ≤ 8191, `int(h.PubLen)` and
`int(h.CtLen)` are exact widenings on a 32-bit `int`, and
`52 + 8191 + 8191 + 16 = 16450` is the supremum — three orders below `MaxInt32`.
The four reachable shapes all behave: `pub>0,ct=0` → `52+pub`; `pub=0,ct>0` →
`52+ct+16`; both > 0 → full sum; both 0 → `ErrEmpty` before any arithmetic
(`wire.go:151`).

**Does any line consult an attacker-controlled length before `ParseHeader`
returns nil?** No. Stage 1 reads `region[:8]` (via `hasMagic`) and
`region[:HeaderLen]` — both bounded by package constants. `clampRegion(RegionLen)`
is the only length that reaches `unsafe.Slice`. The first read of `h.PubLen` is
after the `if err != nil { return }`. The `int(uint32)` reinterpretation that
`unlock_key.go:44-58` documents (and reproduces under `GOARCH=386`) is not
reachable here.

**Does the new `Read` change behaviour for any LEGAL payload?** No, on the
device. Downstream, `Inspect` computes the identical `end`
(`open.go:130-135`) and requires `len(blob) >= end`; the new `Read` returns
`len(blob) == end` exactly. `Unlock` (`open.go:190-199`) and `UnlockWithKey`
(`unlock_key.go:70-81`) recompute the same offsets and slice `blob[:split]` /
`blob[split:end]`, both in range at equality. `unlock_flow.go:58`'s
`clear(blob)` and `:110`'s `clear(blob)` still zero exactly what was read.
Verified the formula against every vector rather than trusting it:

```
A len=211  pub=0    ct=143  expected=211  MATCH
B len=211  pub=0    ct=143  expected=211  MATCH
C len=540  pub=0    ct=472  expected=540  MATCH
D len=539  pub=396  ct=75   expected=539  MATCH
E len=448  pub=396  ct=0    expected=448  MATCH
F len=1421 pub=0    ct=1353 expected=1421 MATCH
G len=1420 pub=1125 ct=227  expected=1420 MATCH
```

**Is `total > len(region)` the right check, and is `ErrTooShort` the right
error?** The check is unreachable and the sentinel is wrong. See **M2**.

**Does the two-stage form break `hasMagic`'s contract or the F-79 residency
argument?** Neither. `hasMagic` still receives the full 65,536-byte mapping and
its `len(b) >= len(Magic)` guard is untouched; `Probe` is not modified. §10.1's
deliberate Probe-coarser-than-Read asymmetry (`read.go:30-37`) is preserved:
magic-present-but-header-invalid still probes true and still reaches
"Payload unreadable" — the message merely originates in `Read` now instead of
`Inspect`, and `unlock_flow.go:39-45` maps both to the same string. F-79 is
strengthened, and by more than the design claims (see **N3**).

Where fix C *is* wrong is not in its logic but in **where it lives** (I2), what
it silently does to the only instrument that can observe it (I3), and its doc
comment (M3).

---

## Q2 — Is the root-cause claim true, and is the sweep complete?

### (a) Does a truncated `maskStack` retain `imageOp` values holding live slice headers?

**Yes — the mechanism is real, and I verified it rather than taking it.**
`frameOp` is appended in exactly one place, `gui/op/op.go:317`, inside the
`opImage, opMask` case and only on the `typ != opImage` branch. The value
appended is built at `:312`:

```go
iop := imageOp{src: rargs[0], args: oargs, refs: rargs[1:]}
```

with `oargs = args[r.end-1-nargs : r.end-1]` over `args := buf.args` (`:264`,
`:276`) and `rargs = refs[r.refs-nrefs : r.refs]` over `refs := buf.refs`
(`:265`, `:277`). Both are slice headers into the Buffer's backing arrays.
`Draw` truncates at `:249` and the recursion truncates at `:369`, so at the
session tail `len(d.maskStack) == 0` while `cap` is the all-frames high-water,
every slot holding a `frameOp` from some earlier frame. A mark-sweep collector
follows the whole object. The design's mechanism is correct as stated.

One correction the design should carry: **only `MaskOp`s ever enter
`maskStack`** (`op.go:315-318`). `op.Image`/`op.Color` ops never do — their
`fop` is a function local. This matters for test 2 (see **I4**).

### (b) Is `clear(s[:cap(s)])` correct and sufficient?

Correct. `s[:cap(s)]` is legal for any slice including nil (`cap == 0`), and
`clear` on a slice zeroes every element, so `frameOp{pos, imageOp{src, refs,
args}}` and `inputOp{bounds, tag}` are fully nilled. `jumpStack []ops` is
`struct{start, end, refs int}` — pointer-free, truncation is genuinely
sufficient, and the design is right to list it for audit completeness rather
than as a leak. `d.text` is nil in production (`ExtractText` is the only
setter and every caller of it is in a `_test.go`).

Sufficient **for the Drawer**. Not sufficient to explain the measurement — C1.

### (c) Does any other long-lived object alias a Buffer the same way?

I checked the sweep rather than trusting it, and I did it type-first rather than
object-first, which is the stronger argument and one the design does not make:

- `imageOp` and `inputOp` are the **only** two types in the tree that hold a
  slice into a `Buffer`'s arrays. Grep over `gui/op/`: they are declared at
  `op.go:508` and `:571`, constructed at `:306` and `:312`, and stored in exactly
  two places — `d.inputs` (`:310`) and `d.maskStack` (`:317`). Nowhere else.
- The other route to a whole Buffer is a stored `op.Op`/`op.MaskOp` (they carry
  `buf *Buffer`). Repo-wide, three struct fields hold one: `richText.Content`
  (`gui.go:208`), `ftConfirmView.Content` (`freetext_flow.go:1286`) and
  `Choice.W` (`gui.go:1456`). All three are per-frame or per-screen and die with
  the flow on the unwind.
- **`op.Drawer` therefore is the only aliaser**, and production holds exactly
  **one** instance: `gui/run_flow.go:42`. Every other `new(op.Drawer)` in the
  tree is in a `_test.go` (measured: 14 sites, all test files). That is a
  stronger result than the design's object-by-object sweep and it should replace
  it.

**What the sweep missed** (none of it changes the fix; all of it changes the
argument):

- Package `op` itself is never swept. Five package-level `*ImageHandle`s
  (`op/image.go:32,40,48,60`; `gui.go:438`) each retain `refs[0]` and derived
  state across every wipe — `glyphImage` keeps `img.face`, `img.g` **and
  `img.r`, the last rune drawn**. Neither `Scrub` nor `Release` reaches them.
  Magnitude is one rune plus package assets, so it is a Nit (**N2**) — but it is
  precisely the class the sweep exists to enumerate.
- `a.warnBuf` is a second, long-lived `op.Buffer` that is only ever `Reset()`
  (`run_flow.go:208`), never `Scrub()`ed (**N1**).
- The design says `frameOp.op.args` "aliases *the same array* `ctx.B.Scrub()`
  zeroes". True for `args` and `refs`. **`src` and `tag` are copies of interface
  values, not aliases** — `Scrub` nils the array slot, not the copy. The
  conclusion ("not a secrecy bug") still holds today, and I verified it site by
  site, but the argument as written does not establish it (**M4**).

### The root-cause claim itself — **CRITICAL**

**C1. Critical.** *Design §"Root cause: a slice-truncation leak in `op.Drawer`",
and the measurement table it rests on.*

The design attributes 214 KB and 1,567 live objects to the Drawer's stale
`frameOp`s. **The build that produced those numbers already had
`ctx.B.Scrub()`.** Verified, not assumed:

```
$ git merge-base --is-ancestor e8e78f0 e969839 && echo YES
YES
$ git show e969839:gui/run_flow.go | grep -n Scrub
245:			ctx.B.Scrub()
$ git show e969839:gui/op/buffer_len.go   # Scrub
	clear(b.args[:cap(b.args)])
	clear(b.refs[:cap(b.refs)])
```

`e969839` is `DIAGNOSTIC: heap readout on the start screen (branch
b2b-heapprobe)`, dated 2026-08-09, sitting directly on top of `484ceb9` →
`e8e78f0` (the Scrub commit). `HARDWARE_RESULT_2026-08-09_phaseB2b.md:257` names
that build as the source of the table.

So at the instant of the measurement, **every slot of the abandoned Buffer's
`refs` backing array was already nil** (`clear(b.refs[:cap(b.refs)])`; the Scrub
commit measured it as "refs scrubbed 511/511"). Consequently:

- The Drawer holds **no `*Buffer` and no `*Context`** — check the field list:
  `maskStack []frameOp`, `jumpStack []ops`, `inputs []inputOp`, `skipInputOps
  bool`, `text []rune`. `d.draw` takes `buf *Buffer` as a *parameter* and never
  stores it.
- Its stale `imageOp.refs` slices therefore retain the refs **array**, whose
  contents are nil. They cannot transitively hold "records, splines, glyph refs"
  — which is exactly what `HARDWARE_RESULT_2026-08-09_phaseB2b.md:299-302` says
  the 1,567-object signature means.
- What is left is: **two allocations** (the `args []uint32` and `refs []any`
  backing arrays) plus the interface-value copies in `frameOp.op.src` and
  `inputOp.tag`. I resolved every production call site: all 8 `op.Input` sites
  (`unlock_platelist.go:152`, `passphrase_keyboard.go:495`, `gui.go:1385, 1567,
  1913, 2027, 2028, 2537`) pass a `*Clickable` — a 5-field pointer-free struct
  (`widget.go:7-16`); all 11 `op.Mask` sites pass a package-level `assets.*`
  image or a package-level `*ImageHandle`.

**Concrete failure scenario:** the design retires fix B ("Retired.") and declares
the hang owned. Fix D lands, recovers on the order of `cap(args)*4 +
cap(refs)*8` bytes in **2 objects**, the real retainer of the other ~1,565 is
untouched, and the machine hangs again on a funds path — after the operator has
been told it is fixed, and with the one option that would have addressed it
deleted from the design.

Order-of-magnitude check, using the repo's own measured number: `cap(refs)` was
511 → ~4 KB on the 32-bit target. `cap(args)` is a *single frame's* high-water
(`Context.Frame` resets `ctx.B` per frame, `gui.go:88`; the 228 KB accumulation
`run_flow.go:22-28` describes was the parked-warning bug that `warnBuf` fixed) —
a 24-word `SeedScreen` frame is a few thousand `uint32`. Call it 24 KB in 2
objects, against 214 KB in 1,567.

**Smallest correct fix (to the design, not the code):** do not retire fix B on
this argument. Either (i) instrument it — print `cap(ctx.B.args)` and
`cap(ctx.B.refs)` at `run_flow.go:245`, and re-take the three-row table with fix
D alone; if in-use does not return to ~144 K and allocs to ~688, D is a
bystander and B is still on the table — or (ii) restate D as what it provably is:
a correct, cheap hygiene fix for a real retention path of bounded size, with the
hang's cause still open. D is worth landing either way. What is not sound is the
inference from D to "fix B is retired".

---

## Q3 — Altitude

**Is the session tail the right place?** Today, yes, and for a better reason
than the design gives: production instantiates exactly one `op.Drawer`
(`gui/run_flow.go:42`), and `run_flow.go:233` (`if !wiping { return }`) means the
only path that abandons a Buffer while keeping the Drawer is the wipe path. On a
normal exit `runWithFlow` returns and `d` is garbage with everything else. So the
call site covers 100% of production abandonments.

**Should it instead be called wherever a Buffer is abandoned, or inside `Draw`?**
The design poses this (§"What R0 should attack" item 3) and never answers it —
**M6**. Arguing from the code: the invariant "a `Drawer` must not outlive the
Buffers it drew" is undocumented in package `op` and unenforced, and `Release` is
a method a caller must remember. Making `Draw` self-maintaining —
`clear(d.maskStack[:cap(d.maskStack)])` at `op.go:249` and the same for `inputs`
at `op.go:257` — costs one clear of ≤ cap entries per frame (cap is the frame
high-water, tens of entries, against a full-screen redraw) and makes the leak
structurally unreachable for any future caller or any new abandonment site. The
recursion's truncation at `op.go:369` does **not** need it: those entries alias
the buffer currently being drawn. I am recording this as the altitude answer,
not proposing it as scope.

**The ordering claim — `Scrub` before `Release`.** The order is fine. **The
stated reason is false** (**M1**). `Release` frees nothing; it drops the
Drawer's references. `ctx.B` still holds its own slice headers to the same two
arrays, and `ctx` is used by the very next statement, so the arrays are
unconditionally reachable across both calls and no collector can reclaim them in
between. Reversing the two lines is behaviourally identical. Nothing else depends
on the order. The real ordering constraint — both must follow the session's last
`draw()` — is satisfied at `run_flow.go:245`.

This matters because the false reason is the kind of sentence that gets copied
into a commit message and then into a security argument. It also invites the
opposite error: a reader who believes `Release` frees may conclude `Release`
subsumes `Scrub`.

---

## Q4 — Can the proposed tests fail?

**Test 1 (white-box `Release`).** Would fail before / pass after in the trivial
sense (`Release` does not exist pre-fix), and it does kill the named
`clear(d.maskStack)`-without-`[:cap]` mutant, because at the assertion point
`len` is 0 and the mutant clears nothing. **But it is vacuous unless it first
asserts `cap(d.maskStack) > 0` and `cap(d.inputs) > 0`** — a frame with no mask
ops and no input ops gives `cap == 0`, `[:0]` is empty, and "every element is the
zero value" passes on an untouched Drawer. **M5.**

**Test 2 (finalizer / `runtime.GC`) — the one to scrutinise.** It proves
*reachability*, which is the right property, and reachability is genuinely not
collector-specific, so the design's host-GC argument is sound. Everything else
about it is not. **I4.** Four concrete problems:

1. **False-PASS path — the important one.** The canary sits in
   `d.maskStack`'s backing array at some index *i*. `Draw` re-appends from index
   0 (`op.go:249`), so **session 2's first frame with ≥ i+1 mask ops overwrites
   the canary's slot**, the finalizer runs, and the test passes *without fix D*.
   Real frames draw dozens of masks. Unless the test controls the mask counts
   (session 1 deep, session 2 shallow) and asserts the depth relationship, this
   is a test that passes on the unfixed code — B2a-ii's exact defect class.
2. **The canary cannot be an `op.Image` source.** `op.go:315-318` appends to
   `maskStack` only when `typ != opImage`. A canary injected via
   `op.Image`/`op.Color` never enters the structure under test. It must be an
   `op.Mask` source or an `op.Input` tag.
3. **The canary must not be a `*bitmap.Face`.** `glyphImage`'s closure
   (`op/image.go:32-38`) assigns `img.face = refs[0]` into a **package-level**
   object, pinning it forever. An `op.Glyph`-based canary fails after the fix for
   a reason unrelated to D. `op.Mask(b, img)` is safe — its `materialize` returns
   `src` directly (`op.go:554-565`).
4. **A single `runtime.GC()` is not a deterministic finalizer trigger.** Go
   requires the finalizer goroutine to be scheduled; the idiom is two `GC()`
   calls plus a channel with a timeout. As written the test is flaky-red after
   the fix.

**Test 3 (bounded read).** The formula is right (verified against all seven
vectors, table above). The test as specified **cannot be run**, and one of its
assertions **cannot pass**:

- Where it runs: `read_tinygo.go` is `//go:build tinygo`, so `go test ./seal/`
  never compiles `XIPReader.Read`. Against `FileReader` — which fix C does not
  change — `len(Read())` is the clamped region, so the assertion fails today and
  after the fix, and directly contradicts the existing
  `TestFileReaderNeverReturnsMoreThanTheRegion` (`seal/read_test.go`), which
  asserts `len(got) == RegionLen`. **I2.**
- **"must not allocate" is false.** `ParseHeader`'s reject path for
  `pub_len = 0xFFFFFFFF` is `fmt.Errorf("%w: %d exceeds %d", ErrPubLen, ...)`
  (`wire.go:146`), which allocates a formatted string and a `wrapError`. The
  overflow guard is proven by the *ordering*, not by an allocation count.
  **I5.** Smallest fix: assert `errors.Is(err, ErrPubLen)` and that the returned
  slice is nil; if an allocation bound is wanted, bound it at "no allocation ≥
  `MaxSectionLen`", not zero.

**Test 4 ("mutation rows for both").** Not a test — it names no mutant and no
killer. **I6.** The mutant that matters is *delete `d.Release()` from
`run_flow.go`*, and every existing host test stays green under it by
construction: the leak is invisible to `go test` except through test 2, which
I4 shows has a false-PASS path. So today the design has **no row that can catch
a revert of fix D**.

---

## Q5 — What a funds path cannot be silent about

**I1. Important.** *Design §"The defect, reproduced with numbers" / §"Why B is no
longer needed".* `HARDWARE_RESULT_2026-08-09_phaseB2b.md:310-312` ends on the
open question: *"Whether those 1,567 objects are **reachable** … or merely
**uncollected** (garbage the GC has not run on)."* The design closes it in favour
of "reachable" without any new data. `mem.Mallocs - mem.Frees` counts what the
runtime has swept, not what is reachable; a readout taken without forcing a
collection cannot distinguish the two. **Failure scenario:** the objects were
merely uncollected, D changes nothing, C's smaller request happens to succeed,
the hang appears fixed, and it returns on the next feature that raises the
watermark. **Smallest fix:** force a `runtime.GC()` immediately before the heap
readout in the diagnostic build and re-take the three rows; and state a
falsifiable post-fix acceptance criterion — *"after one wipe, and after three
consecutive wipes, in-use returns to 144 K / 688 allocs"*. Three cycles, not one:
at 214 KB stranded per wipe a single-cycle pass proves nothing about the second.

**I2. Important.** *Design §"Fix C — the bounded read".* Fix C puts the entire
new bound inside `read_tinygo.go`. `seal/read.go:5-14` states the governing rule
in the file's own header: *"The `RegionLen` bound lives here, in `clampRegion`,
and is called by BOTH the host and the TinyGo implementation… A bound placed only
inside `read_tinygo.go` is never compiled by `go test` and no automated test can
reach it."* Fix C does the thing that comment forbids, and the design quotes the
*other* comment in the same file as its constraint while never mentioning this
one or `read_host.go` at all. Consequences: (a) no host test can ever reach fix
C; (b) `FileReader` keeps returning the clamped region, so **host and device
Readers now differ in returned length** and every host GUI/seal test exercises a
65,536-byte blob while the device gets ≤ 1,421; (c) test 3 is unrunnable (Q4).
**Smallest fix:** hoist stages 1-3 into an untagged helper in `read.go` —
`func boundBlob(region []byte) ([]byte, error)` — called by both
`XIPReader.Read` and `FileReader.Read`; test the helper directly; update
`TestFileReaderNeverReturnsMoreThanTheRegion` to assert the bounded length. That
is the same shape `clampRegion` already uses and it costs nothing.

**I3. Important.** *Design §"Callers", second bullet.* The design says
"`cmd/sealread/main.go:102` — host tool, no length assumption." **Both halves are
wrong.** `cmd/sealread` calls `seal.XIPReader{}.Read()` (line 102) and is built
`tinygo build -target pico2` (its own header, lines 39-40) — it is an
**on-target** tool, and given I2 it is the *only* instrument that can observe
fix C at all. And it does make a length assertion, `main.go:112-123`:

```go
ok := "OK"
if len(b) > seal.RegionLen {
    ok = "OVER-LONG — clampRegion FAILED on target"
}
fmt.Printf("sealread: read %d bytes at %#08x (bound %d) %s\n", ...)
```

with the comment *"which the host test cannot observe — read_tinygo.go is behind
//go:build tinygo."* **Failure scenario:** after fix C, `len(b)` is `total`
(211 for vector A), so the region-clamp branch is unreachable for every
header-valid payload and the tool silently stops testing the thing it exists to
test; and a magic-present-but-header-invalid region (exactly the XIP-aliasing
case `main.go`'s own result section documents at 0x10E00000 on a 4 MB Pico) now
returns an error from `Read`, so the `first 8 bytes` dump and the on-target
`ParseHeader` diagnostic at `:127-134` never run. **Smallest fix:** in the design's
Callers section, correct the characterisation, and say what `sealread` must
print after fix C — `len(b)` against `52+pub+ct(+16)`, with the region bound
checked separately via the untagged helper from I2.

Also silent, at lower severity: **M3** (the `Read` doc comment at
`read_tinygo.go:42-47` — *"nothing here may consult them"* — becomes false, and
the design quotes it as the governing rule without updating it; a future reader
"restoring" the invariant deletes the bound); **N1** (`a.warnBuf` is a long-lived
Buffer never `Scrub`ed — content is non-secret, `warningSubject()` returns
constants, but a design that enumerates what survives a wipe should say so);
**N2** (package `op`'s five `*ImageHandle` scratch objects).

---

## Findings, indexed

| # | Sev | Anchor | One line |
| --- | --- | --- | --- |
| C1 | **Critical** | design §"Root cause"; `e969839`, `op/buffer_len.go:23-28`, `run_flow.go:245` | `Scrub` was already in the measured build, so the Drawer can retain at most 2 nil-contents arrays — it cannot account for 214 KB / 1,567 objects, and fix B was retired on that inference |
| I1 | Important | design §"The defect…" / §"Why B is no longer needed" | "reachable vs merely uncollected" closed without data; no forced `runtime.GC()` before the readout; no falsifiable post-fix acceptance criterion |
| I2 | Important | design §"Fix C"; `seal/read.go:5-14`, `read_host.go`, `read_test.go` | the bound lives in the `//go:build tinygo` file, which `read.go`'s own header forbids; no host test can reach it; host/device now return different lengths |
| I3 | Important | design §"Callers"; `cmd/sealread/main.go:102,112-123` | mischaracterised as a host tool with no length assumption; it is the only on-target instrument and fix C degrades it silently |
| I4 | Important | design §"Tests that can fail" item 2; `op/op.go:249,315-318`, `op/image.go:32-38` | finalizer test has a false-PASS path (session 2 overwrites the canary's slot) plus three construction traps |
| I5 | Important | design §"Tests…" item 3; `wire.go:146` | "must not allocate" cannot pass — `ParseHeader`'s reject path is `fmt.Errorf` |
| I6 | Important | design §"Tests…" item 4 | "mutation rows" names no mutant; nothing can catch a revert of `d.Release()` |
| M1 | Minor | design §"Fix D", the ordering note | "Reversed, the array could be freed" is false — `Release` frees nothing and `ctx.B` is live across both statements |
| M2 | Minor | design §"Fix C", `total > len(region)` | unreachable (`clampRegion(RegionLen)==RegionLen`, total ≤ 16450) and returns `ErrTooShort` for what `wire.go:78,201` names `ErrTooLarge` |
| M3 | Minor | `seal/read_tinygo.go:42-47` | the `Read` doc comment becomes false and is not updated |
| M4 | Minor | design §"Why this is a memory bug and not a secrecy bug" | argument covers the aliases (`args`, `refs`) and not the interface-value copies (`src`, `tag`) that `Scrub` cannot reach; conclusion holds today — verified across all 8 `op.Input` and 11 `op.Mask` production sites — but is not established |
| M5 | Minor | design §"Tests…" item 1 | vacuous unless it asserts `cap(d.maskStack) > 0` and `cap(d.inputs) > 0` first |
| M6 | Minor | design §"What R0 should attack" item 3 | the altitude question the design raises is never answered; the `op`-level invariant is undocumented and unenforced |
| M7 | Minor | design §"Root cause", "permanently reachable … for the process's lifetime" | overstated — a stale slot is reclaimed the moment a later frame reaches that depth; retention is high-water-bounded and transient (this is also what creates I4) |
| N1 | Nit | `run_flow.go:29,208` | `a.warnBuf` is a long-lived Buffer only ever `Reset()`, never `Scrub()`ed; omitted from the enumeration |
| N2 | Nit | `op/image.go:32,40,48,60`; `gui.go:438` | five package-level `*ImageHandle`s retain `refs[0]` across every wipe, `glyphImage` also the last rune drawn; package `op` is never swept |
| N3 | Nit | design §"Fix C" | the ≤16,450 figure is the format's worst case; the largest real vector is **1,421 bytes**, so fix C's actual reduction is 46× not 4× — worth stating, it strengthens F-79 |

## What I did not find

- **No Critical or Important in fix C's arithmetic or ordering.** The two-stage
  form is sound against a hostile header on a 32-bit target.
- **No Critical or Important in fix D's code.** `Release` is correct and, for
  the `Drawer`, complete.
- **No second Buffer-aliasing retainer.** `imageOp` and `inputOp` are the only
  types that alias a Buffer's arrays, they are stored in exactly two fields, both
  on `Drawer`, and production holds exactly one `Drawer`. The sweep's conclusion
  is right; its method (object-by-object) is weaker than the type-level argument
  above and should be replaced by it.
