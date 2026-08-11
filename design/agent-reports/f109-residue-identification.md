# F-109 — identifying the residue that survives every wipe

Agent report, 2026-08-11. Fork at `823499c` (`main`). Read-only on both repos;
all instrumentation lived in a throwaway `git worktree` under `/tmp` and is
discarded.

**Bottom line.** The residue is now ~64% named by object, and the security
question is answered with paired controls: **no secret-derived bytes survive a
wipe** — not the plaintext records, not the passphrase in either of its two
in-memory representations, not the derived AES key — in any live heap object,
any goroutine stack, or the data/bss segments. The single largest survivor,
~9.5 KB of the ~35 K, is the **§10.2.4 warning's own frame buffer**, and its
retained contents decode to the warning's compile-time-constant text. The
second largest, ~12.5 KB, is the **display mask**, which `runWithFlow`
deliberately keeps across a wipe and which the device probe counted only because
the probe reads *before* the first frame of the boot session and *after* the
first frame of every later one.

---

## 1. The table

Sizes are for the **RP2350 device** (32-bit: `int`/pointer = 4 B, interface =
8 B, slice header = 12 B), block-rounded to TinyGo's 16-byte allocation
granularity (`wordsPerBlock = 4`, `bytesPerBlock = 4 × sizeof(uintptr)`,
`tinygo-0.41.1/share/tinygo/src/runtime/gc_blocks.go:44-46`). The **capacities**
were measured on host Go by driving the real `uiFlow` through three real
unlock + real §10.2.4 timer-wipe cycles at the real panel size (480×320) with
the real fonts — see §3.

| # | object / class | type | allocation site | what holds the reference | device bytes | secret-bearing? | evidence |
|---|---|---|---|---|---|---|---|
| 1 | display mask pixels | `[]uint8` (inside `*image.Alpha`) | `gui/run_flow.go:172` `image.NewAlpha` | the closure struct `a` in `runWithFlow` — allocated **above** the session loop precisely so "a wipe must not reallocate the mask" (`gui/run_flow.go:43-45`) | **12,480** (480 × 26; + 32 for the `image.Alpha` header = 2 objects) | **no** — write-only alpha scratch for `drawMask`; overwritten every chunk of every frame | chunk geometry computed from `cmd/controller/platform_sh2.go:277-280` (`framebuffer = 320/6*480*2 = 50,880`; `buffers[i] = make([][2]byte, 12,720)`) and `:646` (`nrows = 12,720/480 = 26`) |
| 2 | `a.warnBuf.args` | `[]uint32` | `gui/op/op.go:69` `appendArgs` ← `gui/wipe_warning.go:71` `widget.Labelwf` | `a.warnBuf`, the warning's own buffer (`gui/run_flow.go:29`) | **5,376** (cap 1344 × 4) | **no** — *measured*: the retained words decode to `"WIPING SECRET DATA … This machine holds decrypted seed material and has been idle. … It will be erased in 1 seconds. … Touch the screen to keep it."`, i.e. `wipe_warning.go:28-29,70-72` verbatim plus a countdown integer | §3.3 |
| 3 | `a.warnBuf.refs` | `[]any` | `gui/op/op.go:79` `appendRefs` ← `gui/wipe_warning.go:71` | `a.warnBuf` | **4,096** (cap 511 × 8) | **no** — *measured*: the 511 slots hold only `*op.ImageHandle` ×287, `*bitmap.Face` ×143, `nil` ×81. Both are package-level singletons over `//go:embed` flash data (`gui/assets/embed.go`, `font/bitmap/bitmap.go:103-115` returns a *slice of the embedded font*, allocating nothing). **Zero references to any session object.** | §3.3 |
| 4 | `d.inputs` | `[]inputOp` (`{image.Rectangle; Tag}`) | `gui/op/op.go:406` ← `gui/gui.go:801` `inputWordsFlow` (the word keyboard is the high-water frame) | `d *op.Drawer`, allocated above the session loop (`gui/run_flow.go:42`) | **896** (cap 37 × 24) | **no** — `Drawer.Release` zeroes it *to capacity* on the wipe path (`gui/op/op.go:345-355`) | §3.1, §3.2 |
| 5 | `d.jumpStack` | `[]ops` (`{start,end,refs int}`) | `gui/op/op.go:389` | `d` | **96** (cap 8 × 12) | **no** — three ints, no pointers, no text | §3.2 |
| 6 | `d.maskStack` | `[]frameOp` | `gui/op/op.go:409` | `d` | **48** (cap 1 × 40) | **no** — zeroed to capacity by `Release` | §3.2 |
| — | **identified total** | | | | **23,024 B in 7 objects** | | |
| 7 | **~12 K in ~74 objects: UNIDENTIFIED** | mixed, small (~166 B mean) | — | — | ~12,000 | **unknown, but bounded** — the live-memory scan (§4) finds none of the four secret canaries anywhere in this set | §5 |

Two further classes were considered and are **not** in the residue, but are worth
recording because the reasoning is what closes them:

| class | verdict | why |
|---|---|---|
| the abandoned session `ctx.B` (device-measured caps `2048/512` = 12,288 B) | **not secret even if retained** | `Buffer.Scrub` clears **to capacity**, both arrays (`gui/op/buffer_len.go:23-33`), and it runs at `gui/run_flow.go:326` before the Context is dropped. `TestWipeScrubsTheAbandonedFrameBuffer` (`gui/run_flow_scrub_test.go`) asserts `Residue() == (0,0)` afterwards. This is the one buffer that provably *did* hold rendered secret glyphs, and it is zeroed. |
| the 64 KiB payload region | **not in the residue** | `XIPReader.Read` allocates `boundBlob(region)` bytes, not `RegionLen` (`seal/read_tinygo.go:48-62`, `seal/read.go:83-103`) — ≤ 16,450 by §6.2, and the vectors are ~1 KB. It is `clear`ed then `nil`ed before the session (`gui/unlock_flow.go:110-111`), and the deferred clear is a closure so it does not pin the array. A retained copy would show as 64 K, not 35 K. (Note: `gui/gui.go:1587-1590`'s comment still says `Read` "allocates the whole 65,536-byte region"; that is stale.) |

---

## 2. Reproducing the measurement, and one correction to how it was read

The device numbers in F-109 come from `b2b-heapprobe2` (`64dbf6c`), which prints
`m.Mallocs - m.Frees` after a forced `runtime.GC()`. I verified against the
shipping toolchain's source what those fields mean:

- `tinygo-0.41.1 .../runtime/gc_blocks.go:842` sets `m.Frees = gcMallocs - liveHeads`,
  so **`Mallocs - Frees` is exactly the live-object count**. The "81 objects" is
  a real object count, not a sweep artefact.
- `:827-828` sets `HeapInuse = liveBlocks × bytesPerBlock` with `bytesPerBlock = 16`
  on a 32-bit target. So 35 K over 81 objects is a mean of ~442 B/object — the
  residue is a handful of **large buffers**, not 81 small structs. That is what
  led me to the four backing arrays above rather than to a leak of many records.

**Correction — the baseline reading is systematically low.** `heapLine()` rides
`StartScreen.Version` (`gui/gui.go:1603`), which `uiFlow` constructs *before it
ever calls `ctx.Frame`*. So at **baseline** the readout is taken before the first
`draw()` of the machine's life — before `a.mask` exists and before the Drawer's
arrays have grown. At every **post-wipe** readout, `a` and `d` survive from the
previous session and are fully grown. Rows 1 and 4-6 of the table therefore
appear in the baseline→wipe-1 delta **as an artefact of where the probe reads**,
not because a wipe failed to reclaim them. That is ~13.5 KB of the ~35 K, or
about 39%, attributable to probe placement alone.

`git show 64dbf6c` also fixes the criterion in advance as "MUST return to
~144 K"; the failure to return is real, but ~39% of the shortfall was never
reclaimable in the first place.

---

## 3. How I measured, and on what

**Target: host Go 1.26.3, `GOOS=linux GOARCH=amd64`, `CGO_ENABLED=0`.**
Not TinyGo, not the device. Pointers are 8 bytes; every device byte figure above
is the measured *element capacity* multiplied by the **32-bit** element size,
stated per row. Nothing is carried across as a byte count.

I could not measure on the TinyGo toolchain at all. `tinygo test -gc=precise ./gui/`
fails to compile, exit 1, for three independent reasons (recorded because it is
the reason a device-equivalent number is not in this report):

```
gui/freetext_sizeproof_golden_test.go:111:13: t.ArtifactDir undefined   (TinyGo 0.41.1 ships a go1.25 stdlib; the repo needs go1.26)
gui/freetext_sizeproof_test.go:1586:15: undefined: BuildPreview          (build-tag split)
gui/unlock_program_test.go:148:14: undefined: seal.FileReader            (build-tag split, same shape as F-92)
```

The harness is the existing `gui/run_reentry_test.go` driver — `runWithFlow` +
the **real `uiFlow`** + a real sealed payload + real 12-word passphrase entry +
the **real §10.2.4 timer wipe** (park on Cut/Skip, warning at 3:00, wipe at
3:30), three cycles, under `testing/synctest`. Vectors A (bare-mnemonic secret),
D (public section + hash screen) and F (15 ms1 secrets) were all driven.

### 3.1 Instrument 1 — allocation-site attribution

`runtime.MemProfileRate = 1`, then at every start screen: `runtime.GC()` twice,
`runtime.MemProfile`, and **stream every in-use record to disk immediately**,
retaining nothing. (The first draft kept snapshots in memory; its own maps and
interned stack strings then *dominated* the delta it was measuring — +100 KB of
instrument against +22 KB of subject. That draft's numbers are discarded.)

Result, vector A, three real timer-wipe cycles, host Go:

| | baseline | wipe 1 | wipe 2 | wipe 3 |
|---|---|---|---|---|
| HeapInuse | 1,515,520 | 1,622,016 | 1,654,784 | 1,630,208 |
| live objects | 1,909 | 1,977 | 1,996 | 2,009 |

`+36,032 B / +68 objects` on cycle 1, then near-flat. Same *shape* as the device
(+35 K / +81, then +2, then +0) though the composition differs: ~15 K of the host
delta is Go's own scheduler (`runtime.allocm`/`malg`/`makeProfStackFP`, new Ms)
which does not exist on the device, and the host lacks the device's 12.5 K mask.

The top firmware-attributable rows:

```
+8192 B  +1 obj  op.(*Buffer).appendRefs(op.go:79) <- op.encodeOp(op.go:484) <- op.Color(op.go:125)
                 <- widget.Labelwf(label.go:49) <- gui.wipeWarningOp(wipe_warning.go:71)
                 <- gui.runWithFlow.func1-range1(run_flow.go:298)
+5376 B  +1 obj  op.(*Buffer).appendArgs(op.go:69) <- op.encodeOp(op.go:482) <- op.offsetOp(op.go:493)
                 <- widget.Labelwf(label.go:51) <- gui.wipeWarningOp(wipe_warning.go:71)
+1792 B  +1 obj  op.(*Drawer).draw(op.go:406) <- op.(*Drawer).Draw(op.go:318)
                 <- gui.runWithFlow.func1.3(run_flow.go:183) <- gui.inputWordsFlow(gui.go:801)
```

(8192 = 512 × 16, the host size of `[]any`; the same array is 4,096 B on device.)

### 3.2 Instrument 2 — capacities of everything that survives by design

A diagnostic seam in the worktree hands `runWithFlow`'s cross-session state
(`d`, `a.warnBuf`, `a.mask`, the fresh `ctx.B`) to a probe at the top of each
session — i.e. at baseline and immediately after every wipe. Capacity, not
length, because `Reset`/`Release` re-slice and clear but never shrink.

```
A/s0  Drawer{maskStack=0 jumpStack=0 inputs=0  text=0}  warnBuf{args=0    refs=0}
A/s1  Drawer{maskStack=1 jumpStack=8 inputs=37 text=0}  warnBuf{args=1344 refs=511}
A/s2  Drawer{maskStack=1 jumpStack=8 inputs=37 text=0}  warnBuf{args=1344 refs=511}
A/s3  Drawer{maskStack=1 jumpStack=8 inputs=37 text=0}  warnBuf{args=1344 refs=511}
```

Identical for vectors A, D and F. **It plateaus after cycle 1 and is
byte-identical thereafter** — the same signature F-109 recorded on hardware, and
the expected signature of high-water-mark buffers rather than a leak.

Cross-check: instrument 1 and instrument 2 were built independently and name the
same three objects at the same sizes (1344 × 4 = 5,376; 512 × 16 = 8,192 host;
37 × 48 = 1,776 → 1,792 host size class).

Two control runs matter here: with the wipe **forced** via `wipeNowHook` instead
of the real timer, `warnBuf` stays at `args=0 refs=0` and the retained total
falls from 10,489 to 1,025 device bytes. The warning path *is* the residue.
`d.text` is 0 in production (`ExtractText` is test-only and `Release` nils it).

### 3.3 What the largest survivor actually holds — decoded, not described

Reading `a.warnBuf`'s backing arrays to capacity after the wipe:

```
warnBuf.args printable-rune content (260 of 1344 words):
  "W.I.PING SECRET DATAThis m.a.c.h.i.n.e. .s.t.i.ll holds decrypted seed material
   and has b.e.e.n. .i.d.l.e. It will be erased in 1 seconds. Touch the screen to keep it."
warnBuf.refs types: map[*bitmap.Face:143 *op.ImageHandle:287 <nil>:81]
```

(The interleaved single characters are `op` header words that happen to land in
printable ASCII; 1,084 of the 1,344 words are non-rune-valued headers and
offsets.) That string is `gui/wipe_warning.go:28` and `:70-72` verbatim. It is a
compile-time constant plus one integer. There is nothing operator-supplied in it,
and `warningSubject()` chooses between two constants.

---

## 4. Does any of it hold secret-derived bytes? — with the controls

`runtime/debug.WriteHeapDump` writes the **contents** of every live heap object,
every goroutine stack frame, and the data/bss segments. Six dumps are taken per
run, each after two forced `runtime.GC()`s, and scanned for four secret needles.
Every needle is held by the test XOR-0x5A and the plaintext is reconstructed
**only after the last dump is on disk**, so a hit can never be the test's own copy.

Controls, because an unpaired negative is worthless:

- **MECHANISM (must be non-zero everywhere):** a 32-byte heap object deliberately
  retained by a package var. *(First attempt used a static composite literal and
  scored 0 in every dump — the compiler had put it in rodata, which a heap dump
  does not contain. That is exactly the false negative this control exists to
  catch; it was fixed to `make` + write.)*
- **DOMAIN (must exceed baseline at the instant each secret is live):** dumps
  taken inside the `newDeriver` seam (passphrase live), inside `unlockKeyHook`
  (AES key live), and inside `unlockSecretHook("offered")` (plaintext record live).
- **NEGATIVE:** a pattern allocated in a `//go:noinline` function and dropped.

Vector A (bare-mnemonic secret), host Go, real timer wipes:

| needle | baseline | passphrase-live | key-live | record-live | **after wipe 1** | **after wipe 3** |
|---|---|---|---|---|---|---|
| plaintext secret record | 0 | 0 | 0 | **1** | **0** | **0** |
| passphrase (string form) | 0 | **1** | **1** | 0 | **0** | **0** |
| typed passphrase (`[]bip39.Word`) | 0 | **1** | **1** | 0 | **0** | **0** |
| derived AES-256 key | 0 | 0 | **1** | 0 | **0** | **0** |
| SYNTH (mechanism control) | 1 | 1 | 1 | 1 | 1 | 1 |
| DEAD (negative control) | 0 | 0 | 0 | 0 | 0 | 0 |

Vector F, which carries **fifteen** secret ms1 records, gives the same shape:
each of the 15 records scores exactly 1 at `record-live` and **0** at both
post-wipe dumps. Vector D likewise. Full tables in the run logs.

Reading:

- The scan **does** find firmware-held secrets when they exist — all four
  needles light up at their own live instant, and nowhere else. The negative
  below is therefore a measurement, not an absence of looking.
- Exactly **one** copy of each record exists at `record-live`, which incidentally
  confirms the decrypted section buffer is already zeroed by then (`AdmitSection`
  copies, `seal/record.go:207`).
- After the wipe: **zero occurrences of any secret, in any live heap object, in
  any goroutine stack, in data or bss.** Including the `[]bip39.Word` passphrase
  buffer, which this test *still holds a reference to* — it reads as zeros
  because `clear(m)` ran, so the result does not depend on unreachability.

**Device caveat, stated plainly.** This is host Go, whose GC scans stacks
precisely. TinyGo's `-gc precise` is precise for *heap objects* only: stack and
register scanning is **conservative**
(`.../runtime/gc_stack_raw.go:20-31`, "This implementation is conservative … Also,
it assumes a descending stack"; and `gc_precise.go:79-82` falls back to
`scanConservative` for unknown layouts). So the device can retain objects the
host frees. What that changes and what it does not:

- It can make an object **reachable** that host Go collects → so the device's 81
  may include stale-root retention my host run cannot see. This is my leading
  hypothesis for part of §5's residue.
- It cannot make a **zeroed** buffer non-zero. Every wipe in this path is a
  `clear()`, which is GC-independent by construction (the same argument F-92's
  investigation records), and the buffers that ever held rendered secrets —
  `ctx.B`, `d.maskStack`, `d.inputs` — are zeroed *to capacity* before
  abandonment. So conservative retention of them yields zeros, not secrets.

---

## 5. What I could not determine, and why

1. **~12 KB in ~74 objects is still unnamed.** 23,024 B in 7 objects are named
   with certainty; the device delta is 35 K ± 2 K (the probe prints truncated
   kilobytes). The remainder averages ~166 B/object, which is a many-small-objects
   profile — strings, closures, channels, small buffers — not another big array.
   I could not name them because **no off-device instrument reaches them**: host
   Go's composition differs (it has ~15 K of Go-scheduler M/g allocations the
   device does not, and lacks the device's mask), and `tinygo test ./gui/` does
   not compile (§3). Two specific unverified hypotheses, both consistent with a
   one-time plateau:
   - **TinyGo's `sync.Pool` never releases.** `.../src/sync/pool.go` is a plain
     slice: `Put` appends, nothing ever trims. Every `fmt` call on the unlock path
     (`unlockSecretLabel`, `unlockHashBody`, `log.Printf`, `showError`) parks a
     `*pp` and its grown buffer there permanently. Small, plateauing, and
     invisible to host Go, which drains its pools at every GC. **Not secret on
     this path** — the widget text layout uses a non-allocating custom formatter
     (`gui/text/text.go:80-90`), and its only `fmt.Sprintf` is under `if false`
     for vet — but see item 3.
   - **Conservative retention of the previous session's `Context`** by a dead
     stack slot or callee-saved register in `runWithFlow`'s frame. That would pin
     `ctx.B`'s 12,288 B of arrays, which the device already measured at
     `buf 2048/512` — but they are Scrubbed to capacity, so this costs bytes and
     not secrecy.

2. **Nothing here was measured on hardware.** I had no SeedHammer II. The
   capacities in §3.2 come from the real panel geometry and the real fonts, so
   they should transfer, but they are host measurements.

3. **The cut path is out of scope and untested here.** F-109's three cycles
   walked away at Cut/Skip, so `bip39.Parse`, `engraveSeed`, `backup.SeedString`
   and `plate.Spline` never ran. My runs match that. Whether TinyGo's
   non-releasing `sync.Pool` retains a `fmt` buffer containing seed words on the
   *cut* path is a real, unanswered question — it is adjacent to F-88 and F-83
   and should be asked there, not folded into F-109.

4. **`saver.State` is unexercised.** During an armed window the warning branch
   preempts the screensaver, so the saver never ran in any of my cycles or in the
   hardware ones. If an operator lets the machine idle *unarmed* at the start
   screen before taking a reading, the saver's own allocations enter the number.
   Its state is a value inside the closure struct (`gui/saver/saver.go:15-39`,
   `snakeBuf` is an inline array), so it should not add heap objects, but I did
   not drive it.

---

## 6. Recommendation

**F-109 is a scary-sounding non-issue on the residency question, and a small
Minor on memory hygiene. It is not a Critical and not an Important.**

Justification, from the evidence rather than from the entry's tone:

- The entry is open because the objects were unnamed, and its own gate is
  "identify the ~81 objects … say whether it can hold seed material". 7 objects
  and 64% of the bytes are now named with allocation sites and holders. The
  largest is the warning buffer, whose retained content was *decoded* and is a
  compile-time constant; the second largest is the display mask, which is
  write-only alpha scratch that `runWithFlow` retains **on purpose**.
- The operator's framing — *"for all we know that missing 35 K is unwiped secret
  data"* — was the right question and now has a measured answer: a live-memory
  content scan with a working mechanism control, three domain positive controls
  and a negative control finds **zero** occurrences of the plaintext records, the
  passphrase in either representation, or the derived key after a wipe, across
  three vectors including the 15-secret one.
- The ~12 KB that remains unnamed is bounded by the same scan: whatever those
  objects are, they do not contain the secrets. Bytes without secrets is a
  hygiene item, and it plateaus, so there is no exhaustion path.

Suggested disposition:

1. **Downgrade F-109 to Minor** and record the identification above. The
   §10.2.4 guarantee is about secrets, and the secrets are gone.
2. **Fix the measurement, not the memory** — if a future device probe is run,
   move `heapLine()` to *after* the first `ctx.Frame` of the boot session, or
   take the baseline on the second start-screen visit. ~13.5 KB of the "missing"
   35 K is the probe reading before the first frame ever drawn.
3. **Optional, ~9.5 KB, one line:** `a.warnBuf.Scrub()` beside the existing
   `ctx.B.Scrub()` at `gui/run_flow.go:326` would zero the warning buffer at
   abandonment. It buys no secrecy (the content is a constant) but it makes the
   largest survivor's *audit* trivially total, and it is the only line in this
   report that would change behaviour. Not required.
4. **File separately, do not fold into F-109:** whether TinyGo's non-releasing
   `sync.Pool` (`sync/pool.go`) retains a `fmt`-formatted copy of seed material on
   the **cut** path. That is a different path, a different phase, and a real
   question.
5. **Fix the stale comment** at `gui/gui.go:1587-1590`, which still says
   `XIPReader.Read` "allocates the whole 65,536-byte region"; `seal/read_tinygo.go:48-62`
   allocates `boundBlob(region)` — ≤ 16,450 by §6.2.

---

## Appendix — commands, so this is repeatable

```sh
git worktree add /tmp/f109wt 823499c --detach          # never the checked-out trees
# add gui/f109_*_diag_test.go + gui/op/f109caps.go + one probe seam in run_flow.go
export PATH=/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin:$PATH
CGO_ENABLED=0 go test ./gui/ -run TestF109ResidueByAllocationSite   # exit 0
CGO_ENABLED=0 go test ./gui/ -run TestF109RetainedCapacities        # exit 0
CGO_ENABLED=0 go test ./gui/ -run TestF109SecretBytesInLiveMemory   # exit 0, vectors A/D/F
tinygo test -gc=precise ./gui/                                       # exit 1, see §3
```

Exit statuses were read from `$?` on the command itself, never through a pipe.
