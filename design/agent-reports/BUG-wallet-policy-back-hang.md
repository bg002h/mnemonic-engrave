# BUG — "back hangs the device" on the pathological vault

Investigated in the headless sim against fork `main` **a0c1615** (which contains
F-76/F-437/the r1 fold). Repo left **byte-identical**: everything below ran via
`go test -overlay`, with the harness in the scratchpad
(`…/scratchpad/bug/zz_bug_test.go`, `overlay.json`, `overlay-pre.json`).

Fixture: `design/journeys/out/pathological/backup-strings.txt` (**36 records: 6
md1 chunks + 30 mk1 chunks = 1 policy card + 11 key cards**) packed with
`me sysw pack --no-passphrase --in … --out …`. `sysw.Open` reports 36 public /
0 secret; `me` emitted no "could not decode" warning, so every card is complete.

---

## DIAGNOSIS — neither a leak nor a livelock. A dismiss-only modal that ignores BACK.

**`showError` modals dismiss on Button3 (checkmark) ONLY. Button1 (back) does
nothing on them, forever.** `ErrorScreen.Layout` (`gui/gui.go:353-357`) sets
`s.ok.Button = Button3` and binds no other control; `showModal`
(`gui/slip39_polish.go:23-33`) loops on it until that one button fires.

The screen the operator names is exactly one of these:
`bundleAbortWarning` → `showError(ctx, th, "Bundle Incomplete", …)`
(**`gui/bundle_flow.go:689-692`**) — *"Stopped at card 1 of 12 (md1
descriptor). This set is not a usable backup yet…"*

**Measured** (`TestBugAbortModalIgnoresBack`): walked to the pre-engrave screen,
backed out to the modal, then pressed **BACK 30 times**:

```
ABORT MODAL: "Stoppedatcard1of12(md1descriptor).Thissetisnotausablebackupyet…"
AFTER 30 BACK PRESSES the screen is STILL: "Stoppedatcard1of12(md1descriptor)…"
Button3 dismissed it and walletPolicyFlow RETURNED
```

That is a complete, self-sufficient explanation of *"I'm hung on bundle
incomplete screen"*: the device is alive and redrawing, and the button the
operator is pressing is not wired to anything on that screen.

It is a **class**, not one screen: `gui` has **143** `showError`/`showNotice`
call sites, every one of them BACK-deaf.

### It is not new, and it is not F-76's

Pre-F-76 control (`TestBugPreF76DoorReachesTheGather`, `e456970`'s
`gui/wallet_policy.go` overlaid onto the current tree):

```
PRE-F76 DOOR ok=true: "Firstcardfromwhere?FROMPAYLOADENTERITInput"
PRE-F76 GATHER counted the card = false: "md1descriptors:0mk1keys:0"
PRE-F76 after Done: "Droppedanincompletecard:thepayloaddoesnotcarryallofits
                     chunks.Rewriteitonthehostwith`mesyswpack`toincludeit."
```

So on the old door this payload dead-ends **one screen earlier**, on *another*
`showError` modal with the identical BACK-deaf trap. F-76 did not create the
trap; it moved which modal the operator meets and made the engrave route
reachable at all for this wallet.

---

## THE CYCLE — walked 25 times, no hang, no state loss, no leak

`TestBugOperatorCycle`, one `Context` and one `syswSession` for the whole run
(as on the device). Each cycle: door → FROM PAYLOAD → gather → consent →
bundle review → engrave picker → **the pre-engrave screen** → back out
(back, back, then Button3 to clear "Bundle Incomplete") → flow returns.

**Every one of 25 cycles completed the full walk.** No cycle failed to count the
card, no cycle hung, and the flow returned after exactly 3 presses each time.

### 1. No state consumption bug

`session records=36 seeds=0` after every single cycle. `takeAll` filters, it does
not consume (`gui/sysw_session.go`), so `cardSet` hands out the same 36 records on
every pass, and `ctx.syswBundleSeeds` is correctly emptied by the gather and
re-filled by the next door. Pass 2 reached `md1 descriptors: 1 / mk1 keys: 11`
exactly as pass 1. **"Going back should lose nothing" holds on this route.**

The operator's "you'll go through bundle incomplete screen" is not a state
divergence: that modal is the *normal* terminus of backing out of an engrave, on
every pass including the first.

### 2. No per-cycle leak

Retained heap (`runtime.GC()` ×2, then `ReadMemStats`), `HeapAlloc`, from one
captured 25-cycle run (`…/scratchpad/bug/cycle25.log`):

| point | HeapAlloc | HeapObjects | delta vs cycle 1 |
| --- | --- | --- | --- |
| before any cycle | 649,680 | 1,473 | — |
| after cycle 1 | 1,682,048 | 1,512 | (**+1,032,368** vs baseline) |
| after cycle 2 | 1,682,064 | 1,513 | +16 |
| after cycle 3 | 1,687,752 | 1,521 | +5,704 |
| after cycle 5 | 1,688,568 | 1,527 | +6,520 |
| after cycle 10 | 1,690,696 | 1,535 | +8,648 |
| after cycle 15 | 1,692,392 | 1,542 | +10,344 |
| after cycle 20 | 1,694,024 | 1,550 | +11,976 |
| after cycle 25 | 1,694,904 | 1,556 | +12,856 |

**Cycles 2→25: +12,856 bytes over 24 cycles = 536 B/cycle**, and +44 objects
total — flat enough to be harness noise (a second run measured 760 B/cycle with
two cycles at **+0**). The cycle-1 step reproduces at ~1.03 MB in both runs.
Test result: `PASS`, zero failed assertions, so all 25 cycles reached every
screen including `md1 descriptors: 1 / mk1 keys: 11`.

### 3. The one-time +1.03 MB is HOST-ONLY — it does not exist on the device

`TestBugRetentionProfile`: cycle 1 retains **+1,021,120 bytes in only +22
objects**. `go tool pprof -inuse_space`:

```
1133.89kB 26.89%  github.com/decred/dcrd/dcrec/secp256k1/v4.init.init.func2.func3
       ← hdkeychain.(*ExtendedKey).Derive → ScalarBaseMultNonConst
                                          → scalarBaseMultNonConstFast
```

That is secp256k1's lazily-loaded base-point table: `bytePointTable
[32][256]JacobianPoint` = 32·256·3·(10×uint32) = **983,040 bytes**, decompressed
on first use behind a `sync.Once` and kept for the process lifetime
(`loadprecomputed.go:27-90`). Clearing `ctx.B` freed only 37,400 bytes, so the
op buffer is not the retainer.

**It is compiled out on the MCU.** The dispatch is build-tagged:

- `curve_precompute.go` — `//go:build !tinygo` → `scalarBaseMultNonConstFast` (the table)
- `curve_embedded.go` — `//go:build tinygo` → `scalarBaseMultNonConstSlow` (no table)

Corroborated by the device build's own size report: `secp256k1/v4` occupies
**801 B rodata / 200 B data / 140 B bss** in the pico-plus2 image. So this
number must **not** be carried to the 450 KB heap budget — it is an artifact of
measuring on x86.

With it subtracted there is nothing left to exhaust a heap: ~536 B/cycle of
harness noise. At that rate the 450 KB device heap would take on the order of
800 cycles, against the operator's observed 2-5 — the leak hypothesis does not
fit the observation by three orders of magnitude.

### 4. TinyGo on exhaustion panics; it does not spin

`-gc precise` uses `gc_blocks.go`, which calls `runtimePanicAt(…, "out of
memory")` (lines 407, 456). An OOM would abort, leaving whatever was last drawn
frozen on the panel — it would **not** keep redrawing a modal that responds to
one button. The observed symptom (a live screen ignoring one button) matches the
BACK-deaf modal; it does not match OOM.

---

## Not reproduced: the original single-back-press report

The first report ("at the Wallet policy info screen showing change addresses,
back hangs") did not reproduce in isolation, and the numbers say why it would
not:

- The Wallet Policy **consent** screen carries the address proof as lines
  (`Receive 0/1`, `Change 0/1` on page 2 of the paged screen). Building the whole
  consent surface for this 11-key miniscript vault — `ExpandWalletPolicyChunks`,
  `FormAwareIdChunks`, `complexAddressSource`'s probe **plus 4 derivations** —
  took **8.9 ms** on x86 (36 ms including the gather). Paging is **~1.2 ms**.
- BACK from that screen returned to the gather in **10.1 ms**, correctly, with
  the cards intact (`md1 descriptors: 1 / mk1 keys: 11`).

So the consent path is neither a freeze nor a re-derivation storm. The screen
titled *"Change addresses"* (`addressListFlow`) is **not reachable from Wallet
Policy for this wallet at all** — it hangs off `md1PolicyFlow` (Inspect) and off
`DescriptorScreen`, and this policy resolves to *"Complex policy - display
only."* before either. The operator was most likely on the consent screen's
change-address page, and what they read as a hang is the same BACK-deaf modal one
step later.

Derivation counts and x86 timings, for the record: consent = **1 probe + 4
address derivations** per build, 8.9 ms total; `addressListFlow.recompute()`
derives one address per line that fits and re-derives the whole page on
toggle/page, but back triggers **zero** re-derivation.

---

## MINIMAL FIX PROPOSALS (not implemented)

**D1 — the real defect. Make the dismiss-only modal accept BACK.** In
`ErrorScreen.Layout` (`gui/gui.go:353-357`) bind a second `Clickable` on
`Button1` that dismisses identically, and draw it in the nav row with
`assets.IconBack`. One screen, one meaning: on a dismiss-only modal, BACK and OK
are the same action, so wiring both costs nothing and removes a dead control.
Fixes all 143 sites at once. A test must press BACK on a `showError` and require
it to return — that is precisely the assertion `TestBugAbortModalIgnoresBack`
already encodes, inverted.

*Alternative if a bare `Button1` binding is considered too broad:* bind the
back **AltButton** on the existing `ok` clickable, which changes no layout.
Weaker, because the nav row still shows one glyph and the operator still cannot
see that BACK works.

**D2 — the honesty half (Minor).** "Bundle Incomplete" is a terminus that ends
the program, but nothing on it says so. The operator pressing BACK is asking to
*go back*, and the screen's own text ("Finish a set in one sitting, or start
over") implies a route it does not offer. Once D1 lands, add one closing line
naming what dismissing does (returns to the menu).

**Not proposed:** any allocation change, any `cardSet`/`takeAll` change, any
consent-screen caching. The measurements above do not support a leak, a
consumption bug, or a derivation-cost problem, and a fix aimed at those would be
aimed at nothing.

## What this does NOT establish

- No hardware run. Everything here is the host sim; the device may still hold a
  defect these measurements cannot see.
- The 25-cycle run drives one specific back-out route. A different route out of
  `bundleEngrave` (e.g. aborting mid-plate on real hardware, where
  `releaseResumeState` and the engraver are live) is unmeasured.
- Whether the operator's device was additionally short of memory is not
  answerable from x86; what *is* answerable is that the largest candidate
  retainer is `!tinygo`-only and the per-cycle retention is ~0.
