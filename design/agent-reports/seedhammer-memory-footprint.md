# SeedHammer II firmware — peak dynamic heap footprint (static upper-bound analysis)

**Repo:** `/scratch/code/shibboleth/seedhammer` @ `main` (`59abd64`)
**Target:** RP2350, 520 KB on-chip SRAM = **532,480 B** total. PSRAM present on Pico Plus 2 but **unused** (no PSRAM init in firmware).
**Build:** `tinygo build -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller` (`flake.nix:80`, `.github/workflows/test.yml:29`).
**Method:** static source analysis with allocation-site evidence. This is an **upper-bound estimate**, not a measured GC high-water mark — see the honesty caveat (§7).

---

## 0. Established static baseline (authoritative, from CI `-size full` @ `59abd64`)

| Item | Bytes | Source |
|---|---:|---|
| Flash (text+rodata) | 1,194,632 | CI `-size full` |
| **Static RAM (.data + .bss)** | **58,728** | CI `-size full` |
| Main task stack (reserved) | 16,384 | `-stack-size 16kb` |
| Total SRAM | 532,480 | RP2350 datasheet (520 KB) |
| **⇒ GC heap available** | **≈ 457,368 B (≈ 446.6 KB)** | 532,480 − 58,728 − 16,384 |

The remaining sections size what the firmware allocates **on the GC heap** at runtime, against this ≈446 KB budget. (The 16 KB main stack is treated as already excluded per the baseline; see §2 note on the `-scheduler tasks` subtlety.)

---

## 1. Persistent heap (allocated once at init / first-engraver-open, kept for the session)

All under build tag `tinygo && rp`.

| # | Allocation | File:line | Type / length | Bytes |
|---|---|---|---|---:|
| 1 | Display framebuffer, half A | `cmd/controller/platform_sh2.go:244` | `make([][2]byte, 12720)` | 25,440 |
| 2 | Display framebuffer, half B | `cmd/controller/platform_sh2.go:244` | `make([][2]byte, 12720)` | 25,440 |
| 3 | Engraver DMA ring buffer | `driver/mjolnir2/mjolnir2.go:90` | `make([]uint32, 3200)` | 12,800 |
| 4 | Engraver DMA ctrl word | `driver/mjolnir2/mjolnir2.go:91` | `new(uint32)` | 4 |
| 5 | NFC Type-5 transceiver buffer | `nfc/type5` (sized `st25r3916.FIFOSize`=510), via `platform_sh2.go:286` | `make([]byte, 510)` | 510 |
| 6 | NFC poller bufio reader | `nfc/poller` (256 B), via `platform_sh2.go:530` | `bufio.NewReaderSize(_,256)` | 256 |
| 7 | Misc init channels (`wakeups`, `touch.ints`, `interrupts`, I2C mux) | `platform_sh2.go:217,266,443,651` | small `chan` | ~40 |
| | **Persistent driver/NFC/framebuffer subtotal** | | | **≈ 64,490** |

**Framebuffer detail (confirmed):** `const framebuffer = lcdHeight/6 * lcdWidth * 2 = ⌊320/6⌋·480·2 = 53·480·2 = 50,880 B` (`platform_sh2.go:241`). It is allocated **once at init**, split across `p.display.buffers[2]` at `:244` as `make([][2]byte, framebuffer/2/2)` → 12,720 `[2]byte` elements (25,440 B) per buffer, **50,880 B total**. It is **persistent** (a double-buffered DMA scan-out target, kept alive across frames via `runtime.KeepAlive(d)` at `:629`), **not** re-`make`'d per frame. The two buffers ping-pong (`NextChunk`, `:621`).

**Engraver DMA buffer detail (verified by computation):** `bufferWords = bufDur·TicksPerSecond / (stepsPerWord·1s)` with `bufDur=100ms`, `TicksPerSecond=topSpeed=30·mm=30·6400=192,000` steps/s, `stepsPerWord=32/pinBits=32/5=6` ⇒ `bufferWords = 3200` ⇒ `3200·4 = 12,800 B`. Allocated in `Configure` when the engraver is opened (at init via `go home()` and on first `Engraver()` call); held for the session.

### 1a. GUI render scratch buffers (persistent once grown; resident under live render)

These are grown to a high-water mark and reused (sliced back to length 0), so they behave as persistent once first sized:

| # | Allocation | File:line | Worst-case bytes |
|---|---|---|---:|
| 8 | Mask framebuffer (alpha) for chunk compositing | `gui/gui.go:2741` `image.NewAlpha(fbdims)` | 12,480 (26 rows · 480 · 1 B; sized to the per-chunk dims = `cap(buffers[0])/width` = 12720/480 = 26 rows) |
| 9 | QR/UR scan input buffer | `gui/scan.go:30` `make([]byte, 8*1024)` | 8,192 |
| 10 | `op.Buffer` op-list high-water (`args []uint32` + `refs []any`) | `gui/op/op.go:28`, reset each frame at `gui/gui.go:77` (`c.B.Reset()`) | ≈ 25,000 typical / **≈ 55,000 worst** (≈1500 glyph-ops · (5·4 B args + 2·8 B refs)) |
| | **Render-scratch subtotal (worst)** | | **≈ 75,700** |

The op-list is a **retained-mode display list** reset to length-0 every frame (`Buffer.Reset`, `op/op.go:374`) — capacity persists at its high-water mark. The mask fb grows only if a chunk is larger than seen before. There is **no full-screen RGBA image**: screens composite directly into the rgb565 framebuffer via the op-list (`Drawer.draw`, `op/op.go:261`). The engrave-preview screen (`EngraveScreen`, `gui/gui.go:2514`) draws entirely through the op-list — **no dedicated preview image buffer**.

**Persistent + render-scratch (worst): ≈ 64,490 + 75,700 ≈ 140,200 B (≈137 KB).**

---

## 2. Goroutine stacks (the largest uncertainty)

`go` statements in the firmware (confirmed by grep over `cmd/controller`, `gui`, `driver`, `nfc`):

| Goroutine | File:line | Lifetime | Indirect/interface calls in tree? |
|---|---|---|---|
| `go home()` | `platform_sh2.go:238` | startup → homing done | yes (engraver via `gui.Engraver` iface, I2C `Tx`) |
| power-supply monitor | `platform_sh2.go:452` | **always-on** | yes (I2C `Tx` via `multiplexI2C` iface) |
| engraver job | `gui/engraver.go:88` | during engraving | yes (`Knotter.Knot` iface, `iter.Seq` closures) |
| address-verify scan | `gui/verify_address.go:88` | during that flow | yes (`io.Reader`) |
| mk1 inspect scan | `gui/mk1_inspect.go:172` | during that flow | yes (`io.Reader`) |
| bundle gather scan | `gui/bundle_flow.go:107` | during that flow | yes (`io.Reader`) |
| md1 gather scan | `gui/md1_gather.go:96` | during that flow | yes (`io.Reader`) |
| start-screen NFC scan | `gui/gui.go:1560` | during scan | yes (`io.ReadCloser.Read`) |
| stdin reader | `cmd/controller/debug_sh2.go:87` | session | **`debug` build tag only — not in production** |
| main | runtime | session | — |

**Peak concurrent (production):** the GUI runs **one flow at a time** (the scan/gather/engrave goroutines are mutually exclusive). Steady-state peak = **main + power-monitor + one flow goroutine = 3 goroutines**. (`home()` runs only transiently at startup, before user flows.)

**Per-goroutine stack size — authoritative mechanism (TinyGo):**
- Under `-scheduler tasks`, each goroutine stack is **heap-allocated from the GC heap** at spawn.
- TinyGo determines the size at compile time via DWARF stack-size analysis when the call tree is bounded (`AutomaticStackSize`). `-opt 2` enables this.
- **When the tree contains an indirect/interface call the analyzer cannot bound, it falls back to `config.StackSize()`** = `Options.StackSize` if set, else the target `DefaultStackSize` (`compileopts/config.go`). The build passes `-stack-size 16kb`, so **`Options.StackSize = 16,384`** — this is the fallback per spawned goroutine.
- **Every** firmware goroutine's tree contains an interface/indirect call (I2C `Tx`, `io.Reader.Read`, `Knotter.Knot`, `iter.Seq` yield-closures), so the analyzer is **likely to hit the 16 KB fallback** for them rather than auto-size small. This is the single largest source of uncertainty in the whole estimate.

| Scenario | extra goroutines (excl. main) | per-stack | subtotal |
|---|---|---|---:|
| Optimistic (auto-sized) | 2 (power + one flow) | ~2 KB | ~4,096 B |
| **Conservative (16 KB fallback)** | 2 | 16,384 | **32,768 B** |

> **Subtlety / double-count note:** under `-scheduler tasks` the *main* goroutine is itself a heap-allocated task whose stack is `-stack-size` (16 KB). The §0 baseline already excluded 16 KB as "main stack reserved," so the main task's 16 KB is accounted for there and **not** re-counted here. Only the 2 *additional* live goroutines are charged against the heap above. If on-device instrumentation shows the main task stack also comes from the GC heap pool (not a separately-reserved region), add one more 16 KB — included in the worst-case margin below.

**Sources:** TinyGo [`compileopts.Config.StackSize`/`AutomaticStackSize`](https://pkg.go.dev/github.com/tinygo-org/tinygo/compileopts), [`compiler/goroutine.go`](https://github.com/tinygo-org/tinygo/blob/release/compiler/goroutine.go), [`stacksize` package](https://pkg.go.dev/github.com/tinygo-org/tinygo/stacksize), [TinyGo issue #2000](https://github.com/tinygo-org/tinygo/issues/2000).

---

## 3. Transient peak working set per heavy flow (one at a time, under live GUI render)

Each flow below runs while §1 + §1a + §2 are resident. **They are mutually exclusive** (the device gathers/verifies *or* encodes QRs *or* recovers a secret *or* engraves — never simultaneously), so the worst-case peak takes the **single largest** flow.

### 3.1 Multisig bundle gather + verify + md1 decode — **DOMINANT, ≈ 40–50 KB**
Max cosigners: wire format allows `N≤32` (5-bit + 1) but BIP-380 / GUI practice caps at **N=15**; the §5 11-key general-miniscript wallet is the cited stress case. Largest terms (all file:line in `md/`, `bundle/`, `gui/`):

| Term | File:line | Bytes (N=15) |
|---|---|---:|
| `split()` re-encoding buffers (template strip / re-chunk) | `md/chunk.go:122–175` | ~10,000 |
| Chunk reassembly payloads | `md/chunk.go:216,233` | ~6,000 |
| **Two coexisting descriptor ASTs** (derived + readback) held simultaneously in `bundle.Verify` | `bundle/verify.go:32–107`; tree at `md/md.go:844`; `idxPub`/`idxOrigin`/`idxUseSite` slices at `md/md.go:767,800,693` | ~7,900 × 2 = 15,800 |
| `bundleCard.strings` accumulation across cards | `gui/bundle.go:156` | ~4,800 |
| mk.Card / ms1 entropy / misc | `mk/mk.go:148`, `bundle/verify.go:143` | ~3,000 |
| **Subtotal (conservative)** | | **≈ 40,000–50,000** |

Peak coexistence is during `bundle.Verify`, which holds **two full descriptor ASTs at once** plus the re-encode scratch. Use **50,000 B** as the bound.

### 3.2 QR / SeedQR encode — **≈ 11–12 KB**
The encoder (`github.com/seedhammer/kortschak-qr@v0.3.2`) is reachable only up to **QR version 4 (33×33)**: `engrave.ConstantQR` rejects `dim>33` at `engrave/engrave.go:408`, so larger versions never survive past encode (version 40's ~125 KB pixel grid is **unreachable**). Largest payload = 80 data bytes (level L). Peak during `NewPlan`/`Encode`:

| Term | File:line (module cache) | Bytes |
|---|---|---:|
| Pixel grid `[][]Pixel` 33×33 | `coding/qr.go:649` | 4,356 |
| merged `bits[]` | `coding/qr.go:817` | 3,228 |
| `data[]` | `coding/qr.go:790` | 2,560 |
| Code bitmap (40×33, 1-bit packed) | `coding/qr.go:570` | 1,320 |
| `check[]` + misc | `coding/qr.go:794` | ~700 |
| **Subtotal** | | **≈ 12,300** |

### 3.3 SLIP-39 recovery — **≈ 1–2 KB (algorithmically tiny)**
Worst case 16 shares × 33 words, 1 group (SLIP-39 caps: 16 groups, 16 shares/group). The combine/Lagrange/Feistel/PBKDF2 working set is **≈ 940 B** (`slip39/combine.go:70,80,109,122,140,144`; `lagrange.go:41,42`; `feistel.go:44,45,49,50`), the three phases (member interpolation, group interpolation, Feistel decrypt) are **sequential not simultaneous**, and secret buffers are `wipe()`d. Conservative ceiling with allocator overhead ≈ **2 KB**. (The GUI-held *roster* of decoded shares for the on-screen list is the larger term but is bounded by the per-share `Value` 16–32 B × ≤16 shares ≈ 512 B + mnemonic strings.)

### 3.4 md1 descriptor decode/expand (11-key general miniscript)
Subsumed by 3.1 (same `md.Reassemble` → descriptor-tree path). `make([]ExpandedKey, 0, d.n)` at `md/expand.go:84` and the node tree at `md/md.go:844` are the terms; counted in 3.1.

### 3.5 Engrave execution
The whole engrave pipeline is **lazy/streamed**: `engrave.Engraving = iter.Seq[Command]`, `bspline.Curve = iter.Seq[Knot]` — the plate plan is **never materialized**. `PlanEngraving` uses a bounded `knotBuf` of `maxSplineKnots=100` (`engrave/engrave.go:976–978`). The engrave-QR plan is ≤547 `qrMove` (1 byte each, `engrave/engrave.go:325,361,431`) + a few 264-byte bitmaps (`newBitmap`, `:716–722`). The gonum LP/Simplex spline optimizer (`bspline/optimize.go:305,324`, several KB matrices) is **only reached from `cmd/vectorfont` (host tool) and `gui/qa.go` (QA mode)** — **not the production engrave path**. Production engrave transient ≈ **≤5 KB** (DMA buffer is already counted as persistent in §1).

**Dominant transient = §3.1 multisig ≈ 50,000 B.**

---

## 4. Worst-case peak heap

**Peak = persistent (§1) + render-scratch (§1a) + extra goroutine stacks (§2) + single largest transient flow (§3.1), all coexisting.**

| Component | Bytes |
|---|---:|
| Persistent driver/NFC/framebuffer (§1) | 64,490 |
| GUI render scratch high-water (§1a) | 75,700 |
| Extra goroutine stacks (§2) | 4,096 (optimistic) … 32,768 (16 KB fallback) |
| Dominant transient — multisig verify (§3.1) | 50,000 |
| **PEAK** | **≈ 194,300 B (≈190 KB)** optimistic … **≈ 222,900 B (≈218 KB)** conservative |

### Headroom

| Scenario | Peak | vs ≈446 KB heap | vs 520 KB SRAM (info) |
|---|---:|---:|---:|
| Optimistic (auto-sized goroutine stacks) | ≈ 190 KB | **+257 KB free (~58%)** | +330 KB |
| **Conservative (16 KB goroutine-stack fallback)** | **≈ 218 KB** | **+229 KB free (~51%)** | +302 KB |

Even with **every** assumption pessimistic (worst-case op-list high-water, two 16 KB goroutine stacks, full multisig verify peak), the firmware uses **roughly half** of the ≈446 KB heap, leaving **≈229 KB headroom**. There is comfortable margin.

### Dominant terms (conservative peak ≈218 KB)
1. Display framebuffers — 50,880 B (23%)
2. Multisig verify transient (two descriptor ASTs + re-encode) — ~50,000 B (23%)
3. `op.Buffer` op-list high-water — ~55,000 B worst (25%)
4. Goroutine stacks (16 KB fallback ×2) — 32,768 B (15%)
5. Engraver DMA ring + mask fb + scan buffer — ~33,500 B (15%)

---

## 5. Honesty caveat & largest uncertainties

This is a **static upper-bound analysis**, not a measured GC high-water mark. The exact peak depends on allocation/collection **timing** and TinyGo's conservative mark-sweep GC behavior, which can only be pinned by on-device `runtime.MemStats` (`HeapInuse`, `Mallocs−Frees`) — the firmware already has a hook: `runtimeStats.Dump` reads `runtime.ReadMemStats` in debug builds (`gui/gui.go:2792`). **Recommend enabling that and reading `HeapInuse` during a live multisig-verify-under-render and an engrave to validate.**

**Largest uncertainties, in order:**
1. **Goroutine stack sizing (±28 KB).** Whether TinyGo auto-sizes the firmware's goroutines small or falls back to the 16 KB `-stack-size` per goroutine. Because every goroutine tree contains an interface/indirect call (I2C `Tx`, `io.Reader.Read`, `Knotter.Knot`, `iter.Seq` closures), the **16 KB fallback is the likely case** — this dominates the optimistic/conservative spread. Verifiable with `tinygo build -print-stacks` / the `.tinygo_stacksizes` ELF section.
2. **`op.Buffer` op-list high-water** — modeled at ~25 KB typical / ~55 KB worst from a glyph-count estimate; actual depends on the densest screen. Measurable directly via `MemStats` on the busiest screen.
3. **GC fragmentation / collection headroom** — a conservative GC may want headroom above the live set to avoid thrashing; not a hard allocation but reduces effective free space. Not modeled here.
4. **Multisig verify ASTs** — the "two coexisting descriptor trees" and re-encode scratch in `bundle.Verify` were sized by node-count estimate at N=15; the 11-key miniscript tree node count is the soft spot.

**Bottom line:** persistent ≈ **64.5 KB**; extra goroutine stacks **4 KB (optimistic) – 33 KB (conservative)**; dominant transient = **multisig verify ≈ 50 KB** running under the live GUI render (≈76 KB scratch high-water); **worst-case peak ≈ 218 KB**, leaving **≈229 KB (≈51%) headroom** against the ≈446 KB heap and ≈302 KB against the full 520 KB SRAM.

---

## UPDATE — goroutine-stack uncertainty RESOLVED via `tinygo -print-stacks` (CI commit `ed8fcb1`, run `27921102488`)

`-print-stacks` was added to the CI `tinygo-device-build` step. TinyGo's stack analysis could **NOT bound a single firmware goroutine** — every entry point reports `recursive, runtime.runtimePanicAt may call itself` (the panic path is treated as self-recursive, which defeats bounding). Entry points listed: `Reset_Handler`, `runtime.run$1`, `main.Init$1`, `(*main.Platform).monitorPowerSupply$2`, `(*gui.StartScreen).Flow$2`, `(*gui.engraveJob).Start$1`, `gui.bundleGatherFlow$2`, `gui.md1GatherFlow$2`, `gui.mk1GatherFlow$2`, `gui.scanAddressFlow$2`. (Also visible: a fixed **4096 B C stack** in `.bss`.)

**Consequence:** every concurrent goroutine takes the full **16 KB `-stack-size` fallback** — the **4 KB optimistic case is ruled out; the 33 KB conservative case is the operative one.** The gather/scan flows are mutually exclusive (one active at a time) and `engraveJob` is engrave-only, so peak concurrency stays ~3 live goroutines (main + power monitor + one flow) ⇒ goroutine-stack term ≈ **48 KB total** (~32 KB beyond the main-stack baseline already in §0).

**Net:** the **conservative ≈218 KB worst-case is confirmed** as the operative figure (not the 190 KB optimistic). Headroom **≈229 KB (≈51%) vs the ≈446 KB heap** stands.

**Optional optimization (only if RAM ever tightens — NOT needed at 51% headroom):** the 16 KB-per-goroutine cost is entirely the unbounded-panic-path fallback; making `runtime.runtimePanicAt` non-recursive to TinyGo's analyzer (or lowering `-stack-size` after measuring real high-water via the `.tinygo_stacksizes` section / on-device `runtime.ReadMemStats`) would reclaim ~10 KB/goroutine. Deferred — the margin doesn't warrant it.
