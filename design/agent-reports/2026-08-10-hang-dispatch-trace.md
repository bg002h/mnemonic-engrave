# Hang dispatch trace: checkmark press on "Sealed Payload" -> next `ctx.Frame`

Repo: `/scratch/code/shibboleth/seedhammer-b2b`, branch `b2b` (read-only trace).

Scope: every step executed from the checkmark click on the "Sealed Payload"
carousel entry up to the next `ctx.Frame` call, tagged ALLOCATES / BLOCKS /
NEITHER, followed by a ranked shortlist of freeze candidates. Facts already
established by the caller (deterministic wipe-then-reenter repro; pressed
checkmark frame drew; idle/screensaver/touch all live inside
`runWithFlow`'s inner loop and stopped together, i.e. the flow stopped
calling `ctx.Frame`; heap 144K/301K free @ 688 allocs fresh boot, unchanged
after a normal exit, 358K/87K free @ 2255 allocs after a wipe;
`seal.XIPReader.Read` allocates 65536 bytes unconditionally against a legal
cap of 16,450; a wipe abandons `*gui.Context` and the session loop in
`gui/run_flow.go` builds a fresh one) are NOT re-derived here.

## 1. Ordered step list, checkmark -> next `ctx.Frame`

All line numbers are current source, branch `b2b`.

| # | file:line | step |
|---|-----------|------|
| 1 | `gui/gui.go:1758-1759` | `if selectBtn.Clicked(ctx) { return startScreenAction{prog: m.prog}, true }` — checkmark click detected inside `StartScreen.Flow`'s `for !ctx.Done` loop; function returns. |
| 2 | `gui/gui.go:1698-1704` | Deferred cleanup registered at `Flow` entry (only when `ctx.Platform.NFCReader()` is non-nil) now runs, in order: `close(closer)` (1702), `r.Close()` (1703, `*nfc/poller.Poller.Close`), `<-closed` (1704). Runs **before** `Flow`'s caller (`uiFlow`) sees the return value. |
| 2a | `nfc/poller/poller.go:92-99` | Inside `r.Close()`: `select { case p.reading <- struct{}{}: default: p.d.Interrupt(); p.reading <- struct{}{} }` then `p.d.Close()`. If the poll goroutine's `Read` currently holds the `reading` token (blocked inside `p.d.Detect()`/`p.poll()`, i.e. mid hardware I/O), this branch calls `Interrupt()` and then **blocks** sending to `p.reading` until that in-flight `Read` returns and its `defer func(){ <-p.reading }()` (poller.go:52-54) drains the token. |
| 2b | `gui/gui.go:1707-1739` | The background poll goroutine (started at `Flow` entry) loop body: top-of-loop non-blocking `select` on `closer`; then `s.Scan(r)` -> `poller.Poller.Read` (poller.go:50-90), which internally may call `p.d.Detect()` / `p.poll()` (blocking hardware I/O over the NFC transceiver). Only after this call returns does the loop re-check `closer` and `close(closed)` (gui.go:1712) so step 2's `<-closed` can proceed. |
| 3 | `gui/gui.go:1613-1614, 1638-1639` | Back in `uiFlow`'s `for !ctx.Done` loop: `act, ok := s.Flow(ctx, th)`; `ok==true`, `act.scan==nil`, `switch act.prog { case unlockPayload: unlockPayloadFlow(ctx, th, payloadReader) }`. |
| 4 | `gui/unlock_flow.go:26, 34` | `unlockPayloadFlow` entry; `r == nil` guard is false (payload was probed present at gui.go:1600), falls through. |
| 5 | `gui/unlock_flow.go:38` | `blob, err := r.Read()` — dispatches to `seal.XIPReader.Read`. |
| 5a | `seal/read_tinygo.go:49` | `region := unsafe.Slice(...)` over XIP flash — a mapping, not a copy (no allocation, per F-79 comment). |
| 5b | `seal/read_tinygo.go:50-52` | `hasMagic(region)` check (cheap byte compare; already known true, this is a re-probe). |
| 5c | `seal/read_tinygo.go:56` | **`out := make([]byte, len(region))`** — `len(region)==RegionLen==65536` (`seal/wire.go:50`). One unconditional 65,536-byte allocation, unconditionally sized regardless of the payload's actual declared length. |
| 5d | `seal/read_tinygo.go:57` | `copy(out, region)` — 65,536-byte memcpy out of XIP into `out`. |
| 6 | `gui/unlock_flow.go:38-39` | `err != nil` check on the `Read` result — false on this path (payload present). |
| 7 | `gui/unlock_flow.go:58` | `defer func() { clear(blob) }()` registered (closure form, not `defer clear(blob)` — see in-code comment re F-79/I1). |
| 8 | `gui/unlock_flow.go:63-64` | `var o seal.Opener; p, err := o.Inspect(blob)`. |
| 8a | `seal/open.go:119` (`ParseHeader`, `seal/wire.go:127`) | Parses/bound-checks the 44-byte header; small, fixed-size work. |
| 8b | `seal/open.go:143-153` | If `h.PubLen > 0`: `SplitSection` (`seal/container.go:49`) + `AdmitSection` (`seal/record.go:204`), which does one `append([]byte(nil), r...)` copy per admitted public record (`seal/record.go:227`) — bounded in total by the section's declared `PubLen` (≤8191 per §6.2, and ≤16,450 combined with the whole payload per the wire.go:190-193 comment already cited by the caller). Many small copies, not one large contiguous request. |
| 8c | `seal/open.go:159-165` | `strs := make([]string, len(recs))` + per-record `string(r)` conversions (further small copies), then `PublicDataHash(strs, ...)` — CPU-bound hashing, no I/O. |
| 9 | `gui/unlock_flow.go:65-79` | `err != nil` check on `Inspect` — false on this path. |
| 10 | `gui/unlock_flow.go:85` | `defer p.Wipe()` registered. |
| 11 | `gui/unlock_flow.go:91-93` | **Branch on `p.HasHash`** (true iff `PubLen>0`). If true: `showNotice(ctx, th, "Public Data Hash", unlockHashBody(p))` -> `showModal` (`gui/slip39_polish.go:23-33`) -> loop -> **`ctx.Frame(...)` at `gui/slip39_polish.go:31`**. This is the next `ctx.Frame` call for any Sealed Payload that also carries a public section. |
| 12 | `gui/unlock_flow.go:97-100` | `if p.Header.Sealed() { if !unlockSealedFlow(ctx, th, blob, p) { return } ... }` — `Header.Sealed()` is `CtLen > 0` (`seal/wire.go:95`); true for the entry under test (it is the "Sealed Payload" carousel item). |
| 13 | `gui/unlock_kdf.go:396, 400` | `unlockSealedFlow` entry; first statement is `unlockPassphraseNotice(ctx, th)`. |
| 14 | `gui/unlock_kdf.go:91-96` | `unlockPassphraseNotice` -> `showNotice(ctx, th, unlockTitle, "Enter the 12-word passphrase...")` -> `showModal`. |
| 15 | `gui/slip39_polish.go:23-31` | `showModal`'s `for !ctx.Done` loop builds `ErrorScreen` content and calls **`ctx.Frame(op.Layer(d, op.Color(...)))` at line 31**. |

**Where the trace actually terminates depends on `p.HasHash` (step 11):**
- If the sealed payload also has a nonempty public section (`PubLen>0`, `HasHash==true`): the next `ctx.Frame` is reached at step 11 (`gui/slip39_polish.go:31`, via the "Public Data Hash" notice), *before* `unlockSealedFlow` is even entered.
- If the sealed payload is secret-only (`PubLen==0`, `HasHash==false` — called out in `gui/unlock_flow.go:132-138` as "plausibly the most common real use of this feature"): step 11 is skipped entirely, and the next `ctx.Frame` is reached at step 15, via `unlockPassphraseNotice`'s "Enter the 12-word passphrase" notice.

In both cases, everything strictly before that first `ctx.Frame` call is identical (steps 1-10, 12-14 minus whichever notice fires first), so the candidate freeze points below are the same set regardless of which branch the specific repro payload takes.

## 2. Tags

| step | tag |
|---|---|
| 1 (click detect / return) | NEITHER |
| 2 `close(closer)` | NEITHER |
| 2 `r.Close()` (incl. 2a) | **BLOCKS on: channel send `p.reading <- struct{}{}` (poller.go:97), gated on a goroutine handshake; plus whatever `p.d.Close()`/`p.d.Interrupt()` do at the hardware driver level (not visible in this repo — the concrete `Device` implementation for the real NFC chip lives outside the paths searched; see §4)** |
| 2 `<-closed` | **BLOCKS on: channel recv, waiting for the poll goroutine to observe `closer` and `close(closed)`** |
| 2b poll goroutine `s.Scan(r)`/`p.d.Detect()`/`p.poll()` | **BLOCKS on: hardware I/O (NFC transceiver), duration/interruptibility not visible in this repo (see §4)** |
| 3 dispatch to `unlockPayloadFlow` | NEITHER |
| 4 nil guard | NEITHER |
| 5a `unsafe.Slice` mapping | NEITHER (explicitly zero-allocation per source comment) |
| 5b `hasMagic` | NEITHER |
| 5c `make([]byte, 65536)` | **ALLOCATES 65,536 bytes, one contiguous allocation** |
| 5d `copy(out, region)` | NEITHER (CPU-bound memcpy of 65,536 bytes; not I/O, not a channel/mutex wait) |
| 6 err check | NEITHER |
| 7 `defer clear(blob)` registration | NEITHER |
| 8a `ParseHeader` | NEITHER (fixed ~44-byte parse) |
| 8b `SplitSection`+`AdmitSection` | ALLOCATES — bounded, ≤16,450 bytes total across many small per-record copies (not one large contiguous request) |
| 8c hash-prep + `PublicDataHash` | ALLOCATES small (`[]string` + per-record string copies) + NEITHER (hashing is CPU-bound) |
| 9 err check | NEITHER |
| 10 `defer p.Wipe()` registration | NEITHER |
| 11 conditional `showNotice`→`showModal`→`ctx.Frame` | terminal (reaches the target `ctx.Frame`) when `HasHash==true` |
| 12 `Sealed()` branch | NEITHER |
| 13-14 `unlockSealedFlow`→`unlockPassphraseNotice`→`showNotice` | NEITHER until... |
| 15 `showModal`'s `ctx.Frame` | terminal (reaches the target `ctx.Frame`) when `HasHash==false` |

No `sync.Mutex`, no other channel operations, and no goroutines were found anywhere in the `seal` package (`grep` for `chan `/`sync\.`/`go func` in `seal/*.go` returned nothing) — every step from 4 through 15 is single-threaded, synchronous Go/CPU work or a bounded allocation. The only blocking primitives (channel sends/receives, a goroutine join) on the whole path are in step 2/2a/2b, all inside `StartScreen.Flow`'s NFC-reader teardown, which runs *before* `unlockPayloadFlow` is ever entered.

## 3. Ranked shortlist — most likely silent-freeze cause first

**#1 — `seal/read_tinygo.go:56`, `out := make([]byte, 65536)` inside `XIPReader.Read` (step 5c).**
This is the single largest, single most contiguous allocation anywhere on the path, and it is the *only* step whose resource profile lines up with the measured heap delta: fresh boot and a normal exit both show 301K free / 688 live allocs, but post-wipe shows 87K free / 2255 live allocs — 3.3x more live allocations packed into far less free space. The build is confirmed `-gc precise` (`.github/workflows/test.yml:29: tinygo build ... -gc precise ...`), TinyGo's precise collector is non-moving/non-compacting, so a 65,536-byte request needs one *contiguous* free run, not merely 65,536 free bytes in aggregate. 87K of free space fragmented behind 2255 live objects (vs. 301K behind 688) is exactly the condition under which a nominally-satisfiable request can fail to find a contiguous span — triggering GC and, if that doesn't produce a contiguous 64KB gap, an allocator retry/stall with no code path in this repo that would panic, print, or reboot on the way (this repo has no watchdog-triggering panic handler visible on this trace, and the caller's own facts state no panic/no reboot were observed). This is the only candidate for which the "only after a wipe" gating is actually *explained* by evidence already in hand, rather than merely consistent with it.

**#2 — `nfc/poller/poller.go:92-99`, `Poller.Close()`'s `p.reading <- struct{}{}` send, and the surrounding `close(closer); r.Close(); <-closed` handshake at `gui/gui.go:1701-1704` (step 2/2a/2b).**
This is textbook "goroutine handshake around an NFC reader": `Close()` must interrupt an in-flight blocking hardware `Read` via `p.d.Interrupt()` and then wait for that `Read` to actually return before it can proceed, and the *outer* `gui.go` defer then waits a second time (`<-closed`) for the poll goroutine to notice `closer` was closed. Two sequential blocking waits, both gated on the assumption that `Interrupt()` reliably unblocks whatever hardware call is in flight. If it does not — e.g. the NFC transceiver is wedged in a state `Interrupt()` doesn't reach — this hangs with no panic, no reboot, and (per the established facts) exactly the symptom of "the flow stopped calling `ctx.Frame`," since this all runs *before* `uiFlow` ever dispatches to `unlockPayloadFlow`. Ranked below #1 only because nothing on this trace ties it specifically to *wipe* state: the same `NFCReader()`/`Flow()`/deferred-close sequence executes on every StartScreen exit, including the "enter/exit repeatedly" and "unlock then normal exit" cases the caller says do NOT hang, and the underlying hardware driver instance (`p.nfc` on `cmd/controller/platform_sh2.go`'s `Platform`) is not rebuilt by a wipe — only `*gui.Context` is (`gui/run_flow.go:47`'s session loop). Absent a source-level reason the NFC chip's own internal state would differ post-wipe, this candidate's likelihood rests on hardware behavior this repo doesn't show (see §4).

**#3 — `seal/open.go:143-165` (`SplitSection`/`AdmitSection`/hash-prep allocations, step 8b/8c).**
These allocate, but each individual allocation is small (bounded by §6.2's per-section caps, ≤16,450 bytes total across many pieces, not one contiguous block), and they are ordinary Go/TinyGo heap traffic of the same shape the "enter/exit repeatedly" and "normal unlock" cases already exercise successfully. Nothing distinguishes their post-wipe behavior from their pre-wipe behavior on this trace; listed for completeness, not because the facts point here.

## 4. What could NOT be determined from source

- **What `p.d.Interrupt()` / `p.d.Close()` / `p.d.Detect()` actually do at the hardware level**, and whether `Interrupt()` is guaranteed to unblock an in-flight `Read` on the real ft6x36/NFC transceiver driver. The `Device` interface (`nfc/poller/poller.go:15-23`) is implemented by whatever concrete driver `cmd/controller` wires up as `p.nfc`; that implementation was not located within the paths searched for this trace (this trace stayed inside `gui/`, `seal/`, and `nfc/poller/` per the one-question scope) and its blocking/interruption semantics are therefore unverified here.
- **Whether TinyGo's `-gc precise` allocator, on failing to find a contiguous 65,536-byte run, spins/stalls silently versus panics/aborts.** This is TinyGo runtime behavior, not something in this repo's source; the build flag (`-gc precise`) is confirmed present, but the allocator's actual failure-path behavior under fragmentation was not traced beyond this repo's boundary.
- **Whether `p.HasHash` is true or false for the specific payload used in the hardware repro**, which determines whether the terminal `ctx.Frame` in §1 is reached via step 11 (`gui/unlock_flow.go:92`) or step 15 (`gui/unlock_kdf.go:92` -> `gui/slip39_polish.go:31`). Both branches share the identical set of pre-`ctx.Frame` steps (1-10, 12-14 minus the skipped notice), so this does not change the ranked shortlist, only which exact line the trace would have ended on had it not hung.
