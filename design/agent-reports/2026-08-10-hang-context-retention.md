# Reachability sweep: what survives a §10.2.4 wipe's abandoned `*gui.Context`

Repo: `/scratch/code/shibboleth/seedhammer-b2b`, branch `b2b`, HEAD `6b828cf` at
time of review. Read-only static analysis — no code changed, nothing executed
against the device or the test suite. Byte figures below are order-of-magnitude
plausibility arguments from the code's own shapes (e.g. keyboard key counts),
not measurements; I did not instrument or run anything.

## TL;DR

The session loop (`gui/run_flow.go`) allocates `d := new(op.Drawer)` **once,
outside** the `for {}` session loop (line 42), specifically so it survives a
wipe (comment at lines 43-45). That survival is intentional and, on its own,
cheap. The defect is that two of `op.Drawer`'s fields — `maskStack` and
`inputs` — are only ever **re-sliced to length 0** between frames
(`gui/op/op.go:249,257`), never `clear()`-ed. Both fields are populated with
values that hold **slice-header aliases directly into `ctx.B`'s backing
arrays** (`gui/op/op.go:264-265,276-277,311-318`). A slice header keeps its
*entire backing array* reachable for a precise (non-conservative) GC — the
scanner walks the full allocated capacity of the array, not just the current
`len` — which is the exact mechanism this codebase already had to work around
once, for `op.Buffer` itself (`Reset()` vs `Scrub()`, `gui/op/op.go:374` vs
`gui/op/buffer_len.go:23`, and the `Residue()` helper that explicitly scans
`[:cap(...)]`). `op.Drawer.maskStack`/`.inputs` never got the same treatment.

Net effect: after a wipe, `ctx.B.Scrub()` (`gui/run_flow.go:245`) zeroes the
*content* of the abandoned Context's buffer (so no secret bytes leak through
this path), but it cannot free the *backing arrays themselves*, because
`d.maskStack`'s tail entries — built from the abandoned session's last frame,
before Reset/Scrub ran — still hold slice headers pointing at them. Those
arrays stay allocated, sized to the biggest single frame the abandoned session
ever drew, until some *future* session draws a frame with at least as many
mask ops, overwriting the same slice indices. On a normal (non-wipe) exit this
doesn't happen because `runWithFlow`'s whole closure returns and `d` — along
with everything it aliases — is dropped in one shot, which is consistent with
the given fact that a normal exit returns the heap to boot state byte-for-byte
and a wipe does not.

This is the top-ranked candidate below. Everything else swept either dies with
`ctx` (no external handle exists), is a test-only hook that's nil in
production, or is a goroutine that is synchronously joined before the
enclosing flow function returns.

## Candidate retainer table

| # | file:line | what it is | reachability chain | VERDICT |
|---|-----------|------------|---------------------|---------|
| 1 | `gui/run_flow.go:42` (`d := new(op.Drawer)`); `gui/op/op.go:33-41` (`Drawer.maskStack []frameOp`); `gui/op/op.go:249` (`d.maskStack = d.maskStack[:0]` in `Draw`); `gui/op/op.go:264-265,276-277,311-318` (`args := buf.args; refs := buf.refs; oargs := args[...]; rargs := refs[...]; iop := imageOp{src: rargs[0], args: oargs, refs: rargs[1:]}; d.maskStack = append(d.maskStack, fop)`) | `op.Drawer`, a struct allocated **outside** the session loop so it survives a wipe by design. `maskStack` is a `[]frameOp`, and `frameOp.op` (`imageOp`) carries `args []uint32` / `refs []any` that are **subslices of `buf.args`/`buf.refs`** — i.e. of whatever `*op.Buffer` (`&ctx.B`) was drawn last, not copies. | `d` is reachable for the whole life of `runWithFlow`'s closure (every session). Each `Draw()` call truncates `maskStack` to length 0 but does not `clear()` it, so the backing array — and every `imageOp.args`/`.refs` slice header stored in it beyond the *new* frame's entry count — survives untouched. Those slice headers point into the **abandoned session's `ctx.B` backing arrays**. `ctx.B.Scrub()` (`run_flow.go:245`) zeroes that memory's *content* (shared, not copied, so no divergent copy exists) but cannot free the *allocation*, because `d.maskStack` still references it. The array stays live until a later frame with ≥ as many mask ops overwrites those same indices. Content is scrubbed (no secret leak via this specific path), but the raw byte allocation is not reclaimed — this is a structural analog of the very bug `Buffer.Scrub()` was written to fix, one level up, on a field that never got the same fix. | **RETAINS** — highest-plausible-impact. Frames with many `opMask` ops (e.g. the passphrase keyboard, `gui/passphrase_keyboard.go:292,495` — 26+ keys per page, each producing at least one `Glyph`/`Mask` op via `op.Input`+`op.Compose(Color,Mask)`, or the 12-word `SeedScreen`, `gui/gui.go:2537`) size this retention to whatever the largest frame the abandoned session ever drew required. |
| 2 | `gui/run_flow.go:42`; `gui/op/op.go:33-41` (`Drawer.inputs []inputOp`); `gui/op/op.go:256-259` (`Reset()`: `d.inputs = d.inputs[:0]`); `gui/op/op.go:302-310` (`inputOp{tag: rargs[0], bounds: state.clip}; d.inputs = append(d.inputs, iop)`) | Same `d`, different field. `inputs` holds `inputOp{bounds, tag}` where `tag op.Tag` (`any`) is whatever the screen passed to `op.Input(&ctx.B, tag)` — in this codebase, always a pointer to a small UI widget struct (`*Clickable`, `*ppKey.clk`, etc. — see `gui/unlock_platelist.go:152`, `gui/gui.go:1385,1567,2537`, `gui/passphrase_keyboard.go:495`), never seed/secret material directly. | `d.Reset()` runs at the top of every `draw()` call (`run_flow.go:89`) and only re-slices `inputs` to length 0 — same never-`clear()`-ed pattern as #1. Stale `inputOp` entries from the abandoned session's last frame keep whatever small widget struct their `tag` points at reachable (and, transitively, that widget's own backing slice, e.g. `SeedScreen.words []Clickable` or a keyboard's `pages[page][i][j]` grid — but not the mnemonic itself, which `Confirm` receives as a parameter, not a field on the tagged struct). | **RETAINS**, but modest — bounded by the interactive-element count of the busiest screen (tens of small widget structs, not buffer-sized data). Distinct from #1: the pointed-to objects here are small and UI-only, not the scrubbed seed buffer. |
| 3 | `gui/event.go:10-17` (`EventRouter.pointer.pressedTag op.Tag`); `gui/event.go:296-331` (`Events()`); `gui/gui.go:64-82` (`Context.Router EventRouter`, a **value** field) | Package hint explicitly calls this out: "holds an `op.Tag` from a frame that no longer exists." True within a session (handled: `event.go:307-309` nils it out the next tick if `d.TagBounds` can't find the tag anymore) — but `Router` is embedded **by value** inside `Context`, not stored anywhere else. | No code stores `&ctx.Router` or copies `ctx.Router` outside the `Context` it belongs to (checked: no package-level `*Context`/`*EventRouter` vars, no hook captures it — see #6). So `Router`, and `pointer.pressedTag` inside it, die with `ctx` itself. They cannot outlive the abandoned Context because nothing external points at them — the abandoned-frame staleness the comment worries about is a **within-session** hazard already guarded by the nil-fallback, not a cross-wipe one. | **RULED OUT** as an external retainer — it's part of the abandoned `ctx`'s own graph, not a root pointing back into it. (Same reasoning applies to `EventRouter.filters`/`.events`, which have the identical re-slice-without-clear pattern at `event.go:290,292,274` — internal to `ctx`, irrelevant to what survives the wipe.) |
| 4 | `gui/*.go` — 14 package-level `var …Hook func(...)` declarations (`bip85.go:249,254`; `unlock_platelist.go:30`; `multisig_build.go:36`; `multisig.go:33`; `freetext_flow.go:1415,1429`; `passphrase_flow.go:25,518,531`; `wipe_warning.go:40`; `wipe_guard.go:35`; `unlock_mnemonic_seam.go:13`; `unlock_kdf.go:60,72,80`; `singlesig.go:29`; `unlock_session.go:40,48`) | Test-only observation seams, by their own doc comments ("nil in production"). | Grepped every assignment (`Hook = `) across non-`_test.go` files: the only production-code hit is `cmd/controller/debug_sh2.go`, which sets `initHook`, a separate hook of type `func(events chan<- gui.Event)` (`platform_sh2.go:239`) — it hands the platform's own event-injection channel to a debug REPL, never a `*Context`, `*op.Buffer`, or anything gui-package-internal. That file is gated `//go:build tinygo && rp && debug` — not the default production build. | **RULED OUT** for the production build. If a `debug` build is what was measured, re-check `initHook`/`dbgInit` (`debug_sh2.go:87`) specifically — but even there, the captured `output chan<- gui.Event` only ever carries `gui.RuneEvent`/`gui.ButtonEvent` values (ints), never a Context or buffer reference. |
| 5 | `gui/bundle_flow.go:107`; `gui/md1_gather.go:96`; `gui/mk1_inspect.go:172`; `gui/gui.go:1707` (`StartScreen.Flow`); `gui/verify_address.go:88` (`scanAddressFlow`) | Five structurally identical NFC-scanner goroutines (`go func() { s := new(scanner); for { ...s.Scan(r)... } }()`), one per flow that offers a scan. | Each captures only `r` (the `io.ReadCloser` from `ctx.Platform.NFCReader()`), a local `scans` channel, and `closer`/`closed` synchronization channels — never `ctx` or `ctx.B`. Each site's enclosing function has `defer func() { close(closer); r.Close(); <-closed }()` immediately after spawning, which **blocks the function's return** until the goroutine observes `closer` and signals `closed`. Since the wipe unwind relies on every deferred `clear`/`Wipe` running (established fact), this defer runs too, and the flow function cannot return — hence cannot let `ctx` become unreachable — until the scanner goroutine has fully exited. | **RULED OUT** as a retainer of the abandoned Context (never captures it) — at most a bounded, self-clearing rendezvous delay if `s.Scan(r)` is mid-call when `closer` closes (the goroutine only checks `closer` between `Scan` calls), not a permanent leak. |
| 6 | `gui/engraver.go:15-30` (`engraveJob`), `:95-113` (`Start()`, `go func(){ errs <- e.runEngraving(...) }()`); `gui/gui.go:2699-2712` (`EngraveScreen{job *engraveJob}`); `gui/wipe_guard.go:15-27,49-60` (`wipeGuard.job`, `armed()`) | The engrave goroutine and the job object driving it. `wipeGuard.armed()` is what actually gates §10.2.4's timer. | `armed()` (`wipe_guard.go:53-58`) returns `false` whenever `g.job.Status().State` is `engraveRunning` or `engraveStopping` — i.e. **the wipe timer cannot arm while a job's goroutine is live**, by construction. `EngraveScreen.Engrave` (`gui.go:2714-2778`) also `defer s.job.Stop()`s (line 2715) on every return path, including a `ctx.Done`-triggered unwind. `engraveJob` itself holds only `pl Platform`, `spline`, `conf`, and channels — no reference to `Context`, `ctx.B`, or `EventRouter` regardless. | **RULED OUT** — architecturally cannot be alive+armed simultaneously, and holds no Context reference even transiently. |
| 7 | `cmd/controller/platform_sh2.go:274` (`go home()`), `:488` (`monitorPowerSupply`'s goroutine); `cmd/controller/debug_sh2.go:87` (`dbgInit`, debug-build only) | Platform/hardware-level goroutines, all spawned once at `Init()` / boot, for the process lifetime. | `home` closes over `p *Platform` only (calls `p.Engraver(true)`); `monitorPowerSupply`'s loop closes over `d *ap33772s.Device`, `interrupts`, `voltage`. `Platform` (`platform_sh2.go:43-66`) has no `gui.Context`, `op.Buffer`, or `EventRouter` field — it's pure hardware state (LCD, touch, NFC device handles, display buffers). | **RULED OUT** — never touches gui-package session state at all. |
| 8 | `gui/run_flow.go:19-36` (`a := struct{ mask, warnBuf, idle, armed }{}`) | The struct explicitly called out in the given facts as deliberately surviving a wipe (comment at lines 43-45). | Declared inside `runWithFlow`'s **outer** closure, outside the `for {}` session loop but inside the returned iterator — so it survives across repeated wipes within one `Run()` invocation, and is dropped only when the whole iterator returns (matching the normal-exit boot-state-reset fact). `a.warnBuf` is `Reset()` (not `Scrub()`) each tick (`run_flow.go:208`), so — like #1/#2 — its backing array's cap doesn't shrink, but it's a **separate, dedicated buffer** that never aliases `ctx.B`; it never references the abandoned Context. | **RULED OUT** as a retainer of the abandoned Context specifically (this is a known, accepted, independent baseline cost — not part of the "still references the abandoned Context" question). Flagged only for completeness since it's a plausible *separate* contributor to raw KB if the wipe scenario measured passed through the warning countdown (which any real §10.2.4 wipe does) while a hypothetical "normal exit" baseline never shows the warning at all. |

## Ranking of RETAINS findings by plausible memory impact

1. **`d.maskStack` aliasing into `ctx.B.args`/`ctx.B.refs`** (table row 1) — sized
   to the abandoned session's single largest frame's mask-op count; this is the
   only candidate that can plausibly account for tens-to-hundreds of KB, matching
   the reported ~214 KB delta. It is also the only candidate that directly
   explains the elevated **live-object count** (1567), since every stale
   `frameOp`/`imageOp` entry beyond the new frame's length is itself one or more
   live heap-scanned objects/slice-headers on top of the arrays it pins.
2. **`d.inputs` aliasing small UI-widget pointers** (table row 2) — real, but
   bounded by widget count on the busiest screen; not buffer-sized.
3. Everything else in the table: ruled out — either scoped to `ctx` itself (dies
   with it), a nil-in-production test hook, or a goroutine that is synchronously
   joined (or structurally barred from running) before the flow function that
   would let `ctx` go unreachable actually returns.

## What I did not verify

No instrumentation, build, or test run was performed (task was read-only static
analysis). The `gui/wipe_inventory_audit_test.go` / `gui/run_reentry_test.go`
harnesses construct their own **fresh, per-test** `op.Drawer` (e.g.
`wipe_inventory_audit_test.go:160`), which does not reproduce the module-scope,
across-the-loop `d` that production's `runWithFlow` uses — so the existing test
suite's residency assertions do not currently exercise the `d.maskStack`/
`d.inputs` mechanism identified above at all, which is consistent with this
being unmeasured territory rather than a known-and-accepted cost.
