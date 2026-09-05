# hashlock H2 — refute pass (skeptic 1 of 2) on lens `geometry`'s C-1

**Subject:** fork `hashlock-h2` @ `17b3979`, detached worktree
`/scratch/code/shibboleth/.tmp/h2-wf-refute-geometry-C-1-0` (removed after this run).
**Claim under test (verbatim, abridged):** the `Hashlock phrase` screen draws NO
readout at all on `sh2DisplaySize` (480x320) — masked and revealed, at every
phrase length — so a ≤100-char secret is typed with zero echo; the only
detector (§4.5's host cross-check) runs after the hash is assigned to the
path.
**Verdict: CONFIRMED.** Reproduced independently, twice, by two different
methods; every cited line and number checks out; no counter-evidence found.
Nothing here was committed; both probe files were deleted before the worktree
was removed.

---

## Reproduction 1 — standalone arithmetic, `TestRefuteHashlockPhraseReadoutInk`

A fresh `PassphraseKeyboard`, `MaxHeight` set by hand to 201 (the value the
report claims `hashlockPhraseFlow` computes at `sh2DisplaySize` with the
shipped lead), `.Layout()` called directly, ink counted in the band above the
fixed-height key grid (`kbdsz.Y - kbd.size[kbd.page].Y`), both `revealed`
states, three lengths.

```
go test ./gui/ -run 'TestRefuteHashlockPhraseReadoutInk' -v
=== RUN   TestRefuteHashlockPhraseReadoutInk
    zz_refute_geometry_test.go:48: n=1 revealed=false: kbdsz=(340,209) gridH=182 readoutH=27 strip-ink=0 px
    zz_refute_geometry_test.go:48: n=1 revealed=true: kbdsz=(340,209) gridH=182 readoutH=27 strip-ink=0 px
    zz_refute_geometry_test.go:48: n=20 revealed=false: kbdsz=(340,209) gridH=182 readoutH=27 strip-ink=0 px
    zz_refute_geometry_test.go:48: n=20 revealed=true: kbdsz=(340,209) gridH=182 readoutH=27 strip-ink=0 px
    zz_refute_geometry_test.go:48: n=100 revealed=false: kbdsz=(340,209) gridH=182 readoutH=27 strip-ink=0 px
    zz_refute_geometry_test.go:48: n=100 revealed=true: kbdsz=(340,209) gridH=182 readoutH=27 strip-ink=0 px
--- PASS: TestRefuteHashlockPhraseReadoutInk (0.00s)
```

Control, same keyboard, `MaxHeight=245` (the pre-existing passphrase screen's
budget):

```
go test ./gui/ -run 'TestRefuteHashlockPhraseReadoutInkControlWideBudget' -v
    zz_refute_geometry_test.go:70: CONTROL n=1: kbdsz=(340,209) gridH=182 readoutH=27 strip-ink=26 px
    zz_refute_geometry_test.go:70: CONTROL n=20: kbdsz=(340,209) gridH=182 readoutH=27 strip-ink=520 px
    zz_refute_geometry_test.go:70: CONTROL n=100: kbdsz=(340,228) gridH=182 readoutH=46 strip-ink=2600 px
--- PASS
```

0 px at every length under the claimed budget, non-zero (26–2600 px) under a
wider one — same keyboard, same code path, only the budget differs. (My
`readoutH` band is 8 px wider than the lens's own 19 px figure, because I
included `readoutGap`; it does not change the result — ink is still exactly 0.)

## Reproduction 2 — live, touch-driven route, `TestRefuteHashlockPhraseScreenRealMaxHeightAndInk`

This does not hand-set `MaxHeight`. It drives `composerAddPath` →
`composerHashEdit` → `hashlockPhraseRoute` → `hashlockPhraseFlow` on the real
touch harness (`runComposerAddPath`, which sets `p.display = sh2DisplaySize`),
types the phrase on the real keyboard via `tapPassphraseKey`, then reads back
the **live** `kbd.MaxHeight` the running screen computed, and re-invokes that
same `kbd`'s own `.Layout` (state untouched) to isolate the readout band.

```
go test ./gui/ -run 'TestRefuteHashlockPhraseScreenRealMaxHeightAndInk' -v
    zz_refute_e2e_test.go:50: phrase-len=1 revealed=false LIVE kbd.MaxHeight=201 kbdsz=(340,209) gridH=182 readoutH=27 strip-ink=0 px
    zz_refute_e2e_test.go:50: phrase-len=1 revealed=true LIVE kbd.MaxHeight=201 kbdsz=(340,209) gridH=182 readoutH=27 strip-ink=0 px
    zz_refute_e2e_test.go:50: phrase-len=28 revealed=false LIVE kbd.MaxHeight=201 kbdsz=(340,209) gridH=182 readoutH=27 strip-ink=0 px
    zz_refute_e2e_test.go:50: phrase-len=28 revealed=true LIVE kbd.MaxHeight=201 kbdsz=(340,209) gridH=182 readoutH=27 strip-ink=0 px
--- PASS: TestRefuteHashlockPhraseScreenRealMaxHeightAndInk (2.04s)
```

`kbd.MaxHeight == 201` was **measured off the live screen**, not asserted by
me — this closes the gap between "arithmetic says 201" and "the shipped code
actually computes 201" using the real route (real `screen.CutTop`/`CutBottom`,
real lead/counter layout, real keyboard instance), at phrase length 1 and at
the 28-character corpus anchor phrase (`"correct horse battery staple"`,
`hashlock/testdata/hashlock-v0.8.json` row 0), in both `revealed` states. Ink
in the readout strip is 0 in all four cases.

## Mechanism, checked line by line

`gui/passphrase_keyboard.go` (unchanged by this branch):

```
455:		avail := k.MaxHeight - k.size[k.page].Y - readoutGap
```

`k.size[k.page].Y` is fixed per page at construction (`gui/passphrase_keyboard.go:192`,
`k.size[p] = image.Pt(maxw, y-margin)`), independent of `MaxHeight` — confirmed:
`gridH=182` is identical across every `n` and every `MaxHeight` value tested
above, including the control at `MaxHeight=245`. `avail = 201-182-8 = 11`.

`gui/text/text.go:56-75`, `Style.Measure`: for the empty string the glyph loop
never executes (`l.Next` returns `ok=false` immediately), so `dims.Y =
asc.Ceil() + descent.Ceil()` — a **positive baseline height**, not 0 — which is
exactly why the reported `readoutH`/`readoutH=19` figure is constant across
every length in the lens's log **including length 0**: `Measure("")` still
returns a one-line height. Since `11 < 19`, `Measure(...).Y > avail` is true
for *every* non-empty suffix `r[mid:]` the binary search at
`gui/passphrase_keyboard.go:458-470` can construct (down to a single
character), so the search converges to `lo == len(r)` and `shown =
string(r[len(r):]) == ""` regardless of what was typed. This is not a
boundary-length edge case; it is unconditional under this budget.

`gui/composer_hashlock.go:165-175` (the phrase screen, this branch):

```go
_, content := screen.CutTop(leadingSize)     // 320 - 44 = 276
content, _ = content.CutBottom(8)            // 276 - 8 = 268
leadOp, leadSz := widget.Labelw(..., composerCopyHashlockPhraseLead())
leadBand, content := content.CutTop(leadSz.Y) // - lead height
...
counterBand, content := content.CutTop(cntsz.Y) // - counter height
kbd.MaxHeight = content.Dy()
```

`leadingSize = 44` (`gui/theme.go:43`, confirmed by grep). Live measurement
above gives `content.Dy() == 201`, matching `320 - 44 - 8 - 44 - 23`.

## The causal claim (fold pushed the lead from one line to two)

`SPEC_hashlock_H2_device.md:190` (§4.2), verbatim:

```
Title **`Hashlock phrase`**, lead (journey I-2): *"Use a phrase you have never
used anywhere else."*
```

Shipped copy, `gui/composer_copy.go:367-369`, verbatim:

```go
func composerCopyHashlockPhraseLead() string {
	return "This screen does that hashing for you. Use a phrase you have never " +
		"used anywhere else."
}
```

The spec's own lead is the single sentence; the shipped lead prepends a second
one. `grep` confirms the spec's sentence is a strict suffix of the shipped
string (both end `...never used anywhere else."`), so this is an addition, not
a rewording. The doc-comment immediately above the function (r0-journey-I-5)
independently states the reason: answering the §8i rule modal the operator
sees immediately before this screen. The lens's claim that the two-sentence
lead is what pushes `MaxHeight` from a value with slack (one line = 44 px
lead → budget 32 px, fits) to one without (two lines = 44 px lead is what's
shipped when the SECOND sentence wraps to a second line at 480px width →
budget 11 px) is consistent with everything measured here; I did not
independently re-derive the one-line-lead counterfactual (the lens's own
`zz_lens_leadsize_test.go`, already deleted), but the shipped numbers
(`leadSz.Y` consuming enough of the 268 px content band to leave only 201) are
independently confirmed above without relying on that file.

## Detection order — hash assigned before the only check, and that check is advisory

`gui/composer_hashlock.go` (confirm loop), the assignment happens first, the
reconcile screen after:

```go
if composerConfirmScreen(ctx, th, "Hash lock", composerConfirmBody(body)) {
	d := h
	st.list.Paths[idx].Hash = &d          // <-- assigned here
	st.hashByPhrase = true
	showError(ctx, th, "Hash lock", composerCopyHashlockReconcile())  // <-- advice, after
	return hashlockAssigned
}
```

`composerCopyHashlockReconcile()` (`gui/composer_copy.go:443-446`), verbatim:

```
"Before you fund this wallet, run ms hashlock with this phrase and method on
the host and check the digest matches."
```

`showError` is `showModal` (`gui/slip39_polish.go:36-37`), documented as "a
dismissible error modal" — a notice, not a gate; nothing inspects whether the
operator actually ran the host command, and nothing blocks composition from
proceeding to engraving regardless. `composerCopyHashlockConfirm`
(`gui/composer_copy.go:406-421`) — the modal shown **before** HOLD, i.e.
before assignment — prints `hash`, `method`, `chars`, the relation/other-path
lines, and "Write down this phrase and the method now" — the word "phrase"
appears only in that instruction, never followed by the phrase's own bytes.
Confirmed by reading the format string directly; there is no third argument
carrying phrase text.

## Independent corroborating evidence not in the lens's own report

The implementation report the branch already carries
(`design/agent-reports/hashlock-H2-implementation-report.md`) documents a real
occurrence of exactly the failure mode the claim warns about, during
implementation itself (D11, lines ~250-260): the walk's author, deliberately
typing a *known* phrase via script (not blind), "mistyped 8 of 28 characters"
on the first attempt because there is no readout to check against, and it was
"caught immediately" only **because the walk asserts the corpus digest** — a
check no real operator's self-chosen phrase has available. This is
first-party evidence that the zero-echo condition reproduced above is not a
theoretical risk: it already produced a silent divergence once, on this exact
branch, caught only by a test-only oracle that does not exist outside a test.
The same report states plainly (line ~19): *"I did not argue it as gating...
its severity is a reviewer's call, not mine,"* confirming the claim's
characterization that severity was explicitly deferred, not asserted by the
implementer.

## On the "downgrade to Important" hint

The brief's own severity rubric (handed to me, not the lens's) lists as
Critical: *"a host/device digest divergence path... or a false claim in a
record."* The reproduced mechanism is exactly that shape: the operator's
*intended* phrase and the *actually-typed* phrase can diverge with zero
on-device feedback in either masked or revealed mode; the resulting digest is
what gets assigned to the spending path and then engraved; the one detector is
optional, informational copy, shown only after assignment, with nothing
enforcing that the operator ever runs it. D11 above shows this is not
speculative. I also checked whether the spec's own N-2 note ("the keyboard's
reveal (`show`) key is inherited as-is: secret-handling, non-gating") could be
read as already covering this and pre-authorizing a downgrade — it does not:
N-2 is about *which* show/hide policy to inherit (a secrecy-semantics
question), not about the readout rendering **zero pixels in both states**,
which is a rendering defect, not a deliberate secrecy choice, and is not
covered by the constellation's "secret-handling defects never gate" rule
either (that rule is about a secret leaking somewhere it shouldn't, not about
a display showing nothing at all in every mode). I did not find a basis in the
branch, spec, or plan to downgrade this to Important; I report it as
independently reproduced and, under the rubric I was given, Critical.

## What I did not find

No counter-evidence anywhere on the route: no other screen echoes the phrase,
no build-gate or CI check catches the empty readout (confirmed absent:
`grep -n TestPassphraseEntryFitsPanel` has no sibling for
`hashlockPhraseFlow`), and no spec or plan line requires or waives a readout
on this specific screen beyond the N-2 note addressed above.

---

## Counts

**0 Critical / 0 Important / 0 Minor / 0 Nit filed by this pass** — this is a
refute-only pass with one target claim, not a fresh review. On that one claim:
**CONFIRMED**, reproduced by two independent methods against the live branch,
mechanism traced to source, causal fold verified against the spec text, and
detection-order/advisory-only status of the sole mitigant confirmed by
reading the code. No basis found for REFUTED or for downgrading to Important.
