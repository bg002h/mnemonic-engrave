# hashlock H2 — refute pass (skeptic 2 of 2) on geometry lens C-1

**Subject:** the geometry lens's Critical C-1 claim — the `Hashlock phrase` screen draws
no readout at all on the shipped panel, masked and revealed, at every phrase length.
**Method:** reproduce independently in a fresh detached worktree at `17b3979`
(`/scratch/code/shibboleth/.tmp/h2-wf-refute-geometry-C-1-1`, removed after this run),
using probe test files that were written, run, and then deleted — nothing committed.

## Verdict: CONFIRMED

## Reproduction 1 — independent probe, not the lens's own test

Wrote a fresh probe (`gui/zz_refute_c1_test.go`, deleted after the run) that drives
`hashlockPhraseFlow` through `newPPHarness`/`ppHarness.start` (the same touch harness
`gui/passphrase_flow_test.go` uses), sets `kbd.Fragment` to a run of `"W"` at several
lengths and both `revealed` states, and checks the drawn frame text
(`uiContains`) for a 10-character run — the same technique
`TestPassphraseEntryFitsPanel` uses, so a false negative from `ExtractText` masking
occlusion cannot explain a miss (a 10-char literal run either is or isn't in the string).

```
go test ./gui/ -run 'TestZZRefuteHashlockReadoutEmpty' -v
```

```
n=1 revealed=false MaxHeight=201 content-has-run=false
n=5 revealed=false MaxHeight=201 content-has-run=false
n=20 revealed=false MaxHeight=201 content-has-run=false
n=70 revealed=false MaxHeight=201 content-has-run=false
n=100 revealed=false MaxHeight=201 content-has-run=false
n=1 revealed=true MaxHeight=201 content-has-run=false
n=5 revealed=true MaxHeight=201 content-has-run=false
n=20 revealed=true MaxHeight=201 content-has-run=false
n=20: NO 10-run of W in the frame -- readout IS empty (as claimed)
n=70 revealed=true MaxHeight=201 content-has-run=false
n=70: NO 10-run of W in the frame -- readout IS empty (as claimed)
n=100 revealed=true MaxHeight=201 content-has-run=false
n=100: NO 10-run of W in the frame -- readout IS empty (as claimed)
--- PASS: TestZZRefuteHashlockReadoutEmpty (0.02s)
```

`MaxHeight=201` at every length, matching the lens's number exactly. No run of typed
text appears in the frame at any length, masked or revealed.

## Reproduction 2 — `TestPassphraseEntryFitsPanel` ported verbatim onto `hashlockPhraseFlow`

The lens claims this exact port "goes red ... at every length." Ported it
(`gui/zz_refute_c1_ported_test.go`, deleted after the run), changing only the flow
under test (`hashlockPhraseFlow` instead of `passphraseEntryFlow`, no `dst`/`nil`
loader arg since the hashlock flow takes none):

```
go test ./gui/ -run 'TestZZRefuteHashlockFitsPanelPorted' -v
```

```
--- FAIL: TestZZRefuteHashlockFitsPanelPorted (0.01s)
    70 chars revealed: keyboard block is 209 tall but only 201 available -- overflows by 8
    70 chars revealed: readout shows no run of typed text; clamp emptied it.
      frame="qwertyuiopasdfghjklzxcvbnmABCspacehideThisscreendoesthathashingforyou.Useaphraseyouhaveneverusedanywhereelse.70/100Hashlockphrase"
    90 chars revealed: keyboard block is 209 tall but only 201 available -- overflows by 8
    90 chars revealed: readout shows no run of typed text; clamp emptied it. ...
    100 chars revealed: keyboard block is 209 tall but only 201 available -- overflows by 8
    100 chars revealed: readout shows no run of typed text; clamp emptied it. ...
    101 chars revealed: keyboard block is 209 tall but only 201 available -- overflows by 8
    101 chars revealed: readout shows no run of typed text; clamp emptied it. ...
```

This reproduces I-1's exact overflow number (209 tall vs 201 available, +8) alongside
C-1, at every one of the four lengths the sibling test checks. The captured `frame`
string shows title, lead, counter (`70/100`), and every key cap/label (including
`hide`, confirming `revealed=true` was honoured) — but **zero** `W` characters despite
`kbd.Fragment` holding 70–101 of them. This is not an artifact of the probe: the frame
contains everything else the layout draws, only the readout is missing.

Control, run unmodified on the branch as it ships:

```
go test ./gui/ -run 'TestPassphraseEntryFitsPanel' -v
--- PASS: TestPassphraseEntryFitsPanel (0.01s)
```

The sibling passphrase screen (no lead) passes the identical assertion. The divergence
is specific to `hashlockPhraseFlow`'s extra lead band, as claimed.

## Mechanism, checked against the branch's own source

`gui/theme.go:43`: `leadingSize = 44`. `gui/composer_hashlock.go:165-175` cuts, in
order: `leadingSize` (title, 44), `CutBottom(8)`, the lead band, then the counter band,
and hands the remainder to `kbd.MaxHeight`. `composerCopyHashlockPhraseLead()`
(`gui/composer_copy.go:367-370`) is:

```go
func composerCopyHashlockPhraseLead() string {
	return "This screen does that hashing for you. Use a phrase you have never " +
		"used anywhere else."
}
```

— two sentences, wrapping to two lines at the panel width, where SPEC_hashlock_H2_device.md
§4.2 specifies only the second sentence ("Use a phrase you have never used anywhere
else."). `gui/passphrase_keyboard.go:454-470`'s clamp (`avail := k.MaxHeight -
k.size[k.page].Y - readoutGap`) drops leading runes by binary search until the
remaining text's measured height fits `avail`; at `avail=11` against a 19px single-line
minimum, no non-empty suffix fits, so `shown` becomes `""` for every input. This
matches the lens's arithmetic (320 − 44 − 8 − 44 − 23 = 201; 201 − 182 − 8 = 11)
exactly — I did not need to re-derive it independently to confirm the readout is
empty (reproductions 1 and 2 above establish that directly), but the arithmetic the
lens gives for *why* also checks out against the cited line numbers.

## Independent corroboration already on the branch, not constructed by either lens

`cmd/emu/walk_hashlock_phrase.js:44-49` (the branch's own emulator walk, dated
2026-09-05, this build) states, unprompted by any reviewer:

> "the phrase screen has no usable readout either -- hashlockPhraseFlow gives the
> keyboard a MaxHeight that leaves no room for one, so PassphraseKeyboard.Layout
> clamps it away and `show` reveals nothing to read a character back from."

and the same file's header (:54-58) gives the 34px pitch / 8px dead gap the lens's
"why it matters" argument relies on. The implementation report
(`design/agent-reports/hashlock-H2-implementation-report.md:19, 221-224, 266`)
independently measured "no `*` at all" with reveal off **and** "the frame still
carries no characters" after tapping `show` (reveal on) — so the branch's own author
already found both states empty, not just the masked one; the lens's framing that it
"adds" the finding that the defect is larger than the inert `show` key is slightly
overstated as a point of novelty (the implementer's own report already states both
states are empty), though the lens's causal localization to the two-sentence lead
(vs. the implementer's more generic "the budget is under one line") is a genuine,
verified addition.

## Confirm-modal and reconcile-screen claims, checked

`composerCopyHashlockConfirm` (`gui/composer_copy.go:406-421`) confirmed to render
only `hash`, `method`, `chars`, optional relation/other-path lines, and "Write down
this phrase and the method now" — never the phrase bytes. In
`hashlockPhraseRoute` (`gui/composer_hashlock.go`), `st.list.Paths[idx].Hash = &d`
executes **before** the `showError(ctx, th, "Hash lock", composerCopyHashlockReconcile())`
call. `showError` (`gui/slip39_polish.go:36`) is `showModal` — a dismissible
informational modal with no verification step, no hold gesture, no gate against
proceeding. `composerCopyHashlockReconcile()`'s text ("Before you fund this wallet,
run ms hashlock with this phrase and method on the host and check the digest
matches") confirms it is advice, not an enforced check, and it fires strictly after
assignment.

## Severity

Both technical claims (readout empty at every length/state; overflow +8px; cause is
the two-line lead; only detector is post-assignment advisory text) are reproduced
directly, with commands and verbatim output, not merely argued. Against this task's
own severity rubric — Critical includes "a host/device digest divergence path" and
"lost operator work" — this qualifies on both counts: a mistyped character produces a
device-assigned digest that diverges from what the host would compute for the
operator's *intended* phrase, undetectable by the counter (which only reports
length, never key identity — true by construction, `len(kbd.Fragment)`), unechoed by
the readout, and caught (if at all) only by an optional, unenforced host cross-check
that the code allows the operator to skip entirely and that runs after the hash is
already assigned to the path. I therefore do not find grounds to downgrade below
Critical; the "narrower digest-or-counter" reading the dispatch brief offers as an
alternative is a legitimate but different rubric (does the digest itself get
truncated — no, 337/337; is the counter itself occluded — no, it draws), not a
refutation of the readout-echo claim, which is what C-1 is actually about.

## What was NOT re-derived (already settled, not re-checked here)

I-1's counter-occlusion-if-C-1-is-naively-fixed argument, M-1 (lead under the Back
button), and N-1 (240x240) were not independently re-measured; this refute pass was
scoped to C-1 as instructed. The digest-drawn-in-full claim (337/337, headroom 107)
and the whole-product maximum-body-length derivation were taken from the lens
report and not recomputed, since C-1's readout-emptiness claim does not depend on
them.

## Counts

**C-1 CONFIRMED.** No Important/Minor/Nit raised by this refute pass; one scope note
(above) on the lens's framing of what it "adds" relative to the implementer's own
F-481 disclosure, which does not change the verdict.
