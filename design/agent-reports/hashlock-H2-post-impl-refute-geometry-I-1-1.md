# hashlock H2 — post-implementation refute pass (skeptic 2/2), lens: geometry I-1

**Subject:** claim I-1 from `design/agent-reports/hashlock-H2-post-impl-lens-geometry.md`, against
fork `hashlock-h2` @ `17b3979`, in a fresh detached worktree
(`/scratch/code/shibboleth/.tmp/h2-wf-refute-geometry-I-1-1`, removed after the run).

**Claim under test:** `hashlockPhraseFlow` hands `kbd.MaxHeight = 201`, `PassphraseKeyboard.Layout`
returns a 209 px block, the block is bottom-aligned and drawn by `op.Layer` on top so an overflow
silently covers whatever is above it, and this is caught by no test — currently harmless only
because the readout is empty (0 px ink) so the 8 px overflow is a blank line box.

**Method:** independent reproduction, not re-reading. Wrote my own probe test files
(`gui/zz_refute_i1*_test.go`, never committed, deleted after the run) that drive the **real**
`hashlockPhraseFlow` screen through the harness the branch's own tests use
(`runComposerAddPath` + `hashlockKbdFor`), rather than trusting the lens's own probe files (which
no longer exist in this worktree — they were also never committed). Every number below is from a
fresh `go test` run in my own worktree.

---

## Verdict: CONFIRMED

### Reproduction 1 — the bound violation itself

`go test ./gui/ -run TestZZRefuteI1KeyboardBlockVsMaxHeight -v`, navigating the real path
`composerAddPath` → "A hash, no keys" → EXPERIMENTAL hold-confirm → "Type a hashlock phrase" →
row 0 → §8i rule modal → OK → "Hashlock phrase", then setting `kbd.Fragment` directly (the same
technique `TestPassphraseEntryFitsPanel` uses on the sibling screen) and reading
`kbd.MaxHeight` / `kbd.Layout(...)`:

```
n=0 MaxHeight=201 kbdsz=(340,209) overflow=8 hasRun(10 W)=false
n=20 MaxHeight=201 kbdsz=(340,209) overflow=8 hasRun(10 W)=false
n=70 MaxHeight=201 kbdsz=(340,209) overflow=8 hasRun(10 W)=false
n=90 MaxHeight=201 kbdsz=(340,209) overflow=8 hasRun(10 W)=false
n=100 MaxHeight=201 kbdsz=(340,209) overflow=8 hasRun(10 W)=false
n=101 MaxHeight=201 kbdsz=(340,209) overflow=8 hasRun(10 W)=false
```

Identical numbers to the reviewer's report (`MaxHeight=201`, `kbdsz=(340,209)`, overflow 8) at
every length tested, obtained from a probe I wrote myself against the harness, not the reviewer's
deleted files. `hasRun(10 W)=false` at every n independently reproduces C-1 (the readout never
shows a run of typed text) as the reason I-1 is silent today.

The spec line the reviewer cites is real:
`gui/passphrase_keyboard.go:53-61` (`MaxHeight bounds the whole block (readout + gap + grid) ...
since it is bottom-aligned and drawn by op.Layer ON TOP, the overflow silently covers whatever is
above it`), and the arithmetic behind the 209 traces directly in source:

- `gui/passphrase_keyboard.go:476`: `gridY := readoutSz.Y + readoutGap` (`readoutGap = 8`, line 448).
- `gui/passphrase_keyboard.go:534`: `combined := image.Pt(..., gridY+k.size[k.page].Y)`.
- `gui/text/text.go:56-78`, `Style.Measure`: `dims := image.Point{Y: asc.Ceil()}` then
  `dims.Y += m.Descent.Ceil()` at the end — this executes **regardless of whether any glyph was
  drawn**, so an empty string still measures a full line's height. Confirmed directly:
  `ctx.Styles.word.Measure(w, "%s", "").Y == 19` (`TestZZRefuteI1CouplingClaim` below).

So `19 (empty-readout floor) + 8 (gap) + 182 (grid) = 209`, against `kbd.MaxHeight = 201`
(`320 - 44 title - 8 bottom - 44 lead - 23 counter = 201`, `gui/composer_hashlock.go:165-175`).
The floor is irreducible: the binary-search clamp at `gui/passphrase_keyboard.go:458-470` can only
drop runes, and dropping every rune still leaves one line's worth of ascent+descent. When
`avail < 19` (here `avail = 11`), no `shown` value satisfies the budget, so the block *always*
exceeds `MaxHeight` on this screen — confirmed at 6 lengths spanning 0 to 101, not merely "at
n=100."

### Reproduction 2 — no test currently gates this screen

```
$ grep -n "MaxHeight\|kbdsz\|kbd.Layout" gui/composer_hashlock_test.go
(no output)
```

`TestPassphraseEntryFitsPanel` (`gui/passphrase_flow_test.go:1331`) is real, currently passes, and
does check exactly this bound — but only for `passphraseEntryFlow`:

```
$ go test ./gui/ -run TestPassphraseEntryFitsPanel -v
--- PASS: TestPassphraseEntryFitsPanel (0.01s)
```

Nothing analogous exists for `hashlockPhraseFlow`. My probe, run with the sibling test's own
assertion (`kbdsz.Y > kbd.MaxHeight` → fail), fails at every one of the 6 lengths tried — the
reviewer's claim that "ported verbatim, that test goes red on `hashlockPhraseFlow` at every
length" is exactly what reproduces.

### Reproduction 3 — the coupling claim ("harmless today, dangerous the moment C-1 is fixed naively")

`go test ./gui/ -run TestZZRefuteI1CouplingClaim -v`, measuring what an **unclamped** readout
would occupy at the same widths the flow uses (`k.size[page].X = 340`):

```
n=0 unclamped readout height=19 (one-line min for n=0 is 19)
n=20 unclamped readout height=19 (one-line min for n=0 is 19)
n=70 unclamped readout height=76 (one-line min for n=0 is 19)
n=100 unclamped readout height=95 (one-line min for n=0 is 19)
```

At n=100 an unclamped readout wraps to 95 px (5 lines), so a naive C-1 fix that simply widens or
removes the clamp — without also widening `MaxHeight` — would grow the block from 209 px to
`182+8+95=285` px, an 84 px overflow, not 8. That is *larger* than the reviewer's framing ("grows
into the counter band"), not smaller: at `MaxHeight=201` there is no slack anywhere in the
201 px budget for a multi-line readout, so restoring it without a budget change drives the block
well past the counter band and toward the lead. The direction of the coupling claim is confirmed;
if anything the reviewer understated the magnitude by illustrating only the counter-band case.

### Content-text corroboration (structural, not ink)

`go test ./gui/ -run TestZZRefuteI1ContentText -v` — `op.Drawer.ExtractText` output at n=0/100/101,
independent of the reviewer's own ink-based measurement:

```
n=0   content="...hidethisscreendoesthathashingforyou.Useaphraseyouhaveneverusedanywhereelse.0/100Hashlockphrase"
n=100 content="...hidethisscreendoesthathashingforyou.Useaphraseyouhaveneverusedanywhereelse.100/100Hashlockphrase"
n=101 content="...hidethisscreendoesthathashingforyou.Useaphraseyouhaveneverusedanywhereelse.101/100Hashlockphrase"
```

The lead and counter text are present in the extracted-text stream at every length (consistent
with, though not proof of, the reviewer's ink measurement that nothing is currently occluded); no
run of typed characters ever appears in the readout position (consistent with C-1). I did not
redo the reviewer's pixel-ink rasterisation (`rasterInk`) myself — that would re-derive a finding
already measured by a different, independent method (ExtractText vs ink), and the two agree on
the only thing that matters for I-1's "currently harmless" clause: the readout draws nothing, so
its 8 px overrun carries no ink.

---

## Assessment against the brief's severity ladder

I-1 is not a host/device digest divergence path, a lost-work event, a hash assigned before HOLD,
or a false-PASS test on a normative guarantee — it is a real, reproduced defect (an invariant the
sibling screen enforces and this screen violates), currently inert only because a *different*,
already-reported defect (C-1) happens to empty the exact 8 px that would otherwise carry ink.
That is squarely **Important** under the brief's definition ("a real defect, missing case, unsound
assumption"), and I found no basis to raise it to Critical on its own (it produces no wrong digest
and no data loss by itself) or to lower it to Minor (it is not wording/records — it is a
reproducible, always-triggered violation of a bound the codebase itself asserts elsewhere via a
named test).

**No overclaim found.** The reviewer scoped the "harmless today" clause correctly and, if
anything, the coupling claim is a *conservative* statement of the danger (Reproduction 3 above).

## Counts

**0 Critical / 0 Important / 0 Minor / 0 Nit new findings.** This is a refute pass on one existing
finding (I-1); verdict below.

**Verdict: CONFIRMED.** Reproduced independently, from my own probe against the real
`hashlockPhraseFlow` code path (not the reviewer's files), at 6 phrase lengths (0/20/70/90/100/101):
`kbd.MaxHeight=201`, `kbd.Layout(...)` block height `209`, overflow `8` at every length; no
committed test currently gates this screen's block against its bound; and the "currently harmless,
dangerous if naively fixed" framing holds and, by direct measurement, is if anything understated.
