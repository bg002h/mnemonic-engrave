# Refute pass (skeptic 1/2) — lens `geometry`, finding I-1

**Subject:** fork `hashlock-h2` @ `17b3979`. **Scope:** I-1 only ("the keyboard
block is 209 px against its own 201 px `MaxHeight`... violated on the new
screen"), as quoted in the dispatch brief. Not re-reviewing C-1/M-1/N-1 or
anything else on the branch.

**Method:** independent reproduction. Wrote my own probe
(`gui/zz_refute_geom_test.go`, deleted after the run, nothing committed) that
does **not** reuse the reviewer's probe file — it drives `hashlockPhraseFlow`
directly through the existing `ppHarness` (`gui/passphrase_flow_test.go:48`,
which sets `p.display = sh2DisplaySize` = 480x320), reads back the `kbd`
widget, and calls `kbd.Layout` itself to get the real `kbdsz`.

## Verdict: CONFIRMED

The claimed numbers reproduce exactly, independently, at every length the
reviewer tested:

```
$ go test ./gui/ -run 'TestRefuteI1Geometry|TestRefuteI1ControlPassphraseScreen' -v
=== RUN   TestRefuteI1Geometry
    zz_refute_geom_test.go:25: n=0: MaxHeight=201 kbdsz=(340,209) overflow=8
    zz_refute_geom_test.go:25: n=20: MaxHeight=201 kbdsz=(340,209) overflow=8
    zz_refute_geom_test.go:25: n=70: MaxHeight=201 kbdsz=(340,209) overflow=8
    zz_refute_geom_test.go:25: n=90: MaxHeight=201 kbdsz=(340,209) overflow=8
    zz_refute_geom_test.go:25: n=100: MaxHeight=201 kbdsz=(340,209) overflow=8
    zz_refute_geom_test.go:25: n=101: MaxHeight=201 kbdsz=(340,209) overflow=8
--- FAIL: TestRefuteI1Geometry (0.01s)
=== RUN   TestRefuteI1ControlPassphraseScreen
    zz_refute_geom_test.go:51: control n=0: MaxHeight=245 kbdsz=(340,209) overflow=-36
    zz_refute_geom_test.go:51: control n=20: MaxHeight=245 kbdsz=(340,209) overflow=-36
    zz_refute_geom_test.go:51: control n=70: MaxHeight=245 kbdsz=(340,228) overflow=-17
    zz_refute_geom_test.go:51: control n=90: MaxHeight=245 kbdsz=(340,228) overflow=-17
    zz_refute_geom_test.go:51: control n=100: MaxHeight=245 kbdsz=(340,228) overflow=-17
    zz_refute_geom_test.go:51: control n=101: MaxHeight=245 kbdsz=(340,228) overflow=-17
--- PASS: TestRefuteI1ControlPassphraseScreen (0.01s)
```

Matches the reviewer's `MaxHeight=201 ... kbdsz=(340,209) ... overflow=8` to
the pixel, at every one of the six lengths, and confirms it is a *structural*
violation (identical regardless of `n`), not a fluke of one length. The
control probe — the exact same `PassphraseKeyboard.Layout`/`MaxHeight`
mechanism run against the sibling `passphraseEntryFlow` (`gui/passphrase_flow.go:141`,
no lead band) — never overflows (margin -36 to -17 px across the same six
lengths), so the defect is specific to `hashlockPhraseFlow`, not the shared
keyboard widget in general.

### Why it is structural, checked against the source (not just measured)

`gui/passphrase_keyboard.go:454-455`:
```go
if k.MaxHeight > 0 {
    avail := k.MaxHeight - k.size[k.page].Y - readoutGap
```
and `gui/widget/label.go:24-57` (`Labelwf`): for an empty string, `l.Next`
returns `false` on the first call, no glyph loop runs, and the function still
returns height `y + m.Descent.Ceil()` where `y` was initialized to
`m.Ascent.Ceil()` — i.e. **one line's ascent+descent, never zero**, even for
`""`. So the binary-search clamp at `gui/passphrase_keyboard.go:458-470` can
empty `shown` (and does, since `avail=11` here is below any renderable line),
but it cannot make the readout's own reported height `readoutSz.Y` reach zero.
The block's minimum possible height is therefore
`readoutSz.Y(empty) + readoutGap(8) + k.size[page].Y(182)` = `19+8+182=209`,
a floor that `MaxHeight=201` sits 8 px below **unconditionally**. That is
exactly why the reviewer's own table shows the same overflow at every length —
confirmed here independently rather than taken on their word.

### Cross-check against the invariant the sibling test already pins

`gui/passphrase_flow_test.go:1331` (`TestPassphraseEntryFitsPanel`) asserts
`kbdsz.Y > kbd.MaxHeight` is a failure on `passphraseEntryFlow`. Grepped
`gui/composer_hashlock_test.go` for any equivalent check against
`hashlockPhraseFlow`'s `kbd.MaxHeight`/`FitsPanel`: none exists (only a
`hashlockKbdFor` helper that captures the widget, never checks the bound). The
brief's framing — "the invariant `TestPassphraseEntryFitsPanel` exists to pin,
violated on the new screen [with] no sibling test" — is accurate: it is the
*same* invariant, on a *different* screen, currently gated by nothing.

### Severity

The brief's own rubric: Critical requires a digest-divergence path, lost
work, a hash before HOLD, or a false-PASS/false record; Important is a real
defect/missing case/unsound assumption. I-1 by itself is exactly the latter —
an unsound assumption in `hashlockPhraseFlow` (that its `content.Dy()` bound
holds for `PassphraseKeyboard`), currently inert only because the overflowing
8 px is provably unrendered ink (an empty text line, confirmed by the
`Labelwf` trace above, not merely "the report says so"). Nothing about I-1 in
isolation reaches Critical — no ink is lost today, so no digest divergence and
no lost operator work follow from *this* finding alone. **Important is the
right severity for I-1 as stated**, matching the reviewer's own call.
Whether the *pair* (I-1 + C-1, if C-1's fix reopens headroom) becomes Critical
is a C-1/coupling question, out of this pass's scope.

**No PARTIAL/REFUTED angle found.** I looked for one obvious out — that the
reviewer's `k.size[page].Y=182` might be page- or theme-dependent so real
device fonts land differently — but `newPPHarness`/`descriptorTheme` is the
same theme and default page (0, lowercase) the branch's own tests and the
emulator walk use, and the control probe used the identical harness/theme, so
this is not an artifact of a peculiar test setup.

## Counts

**0 Critical / 1 Important (confirmed) / 0 Minor / 0 Nit** for this scope.

- **I-1 (CONFIRMED, Important)** — `hashlockPhraseFlow`'s keyboard block is
  209 px tall against its own `kbd.MaxHeight=201`, an 8 px overflow that is
  structural (present at every phrase length 0/20/70/90/100/101, reproduced
  independently to the pixel) and specific to this screen (the sibling
  `passphraseEntryFlow`, same widget, same bound mechanism, never overflows
  across the same six lengths). No sibling panel-fit test exists for
  `hashlockPhraseFlow` today. Currently inert because the overflow is an
  empty text line's reported-but-unrendered height (`Labelwf` returns a
  non-zero height for `""`), not because the arithmetic is sound.
