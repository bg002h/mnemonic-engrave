# H5 device polish — R0 round 0, fidelity + design lens

Artifact: `design/SPEC_hashlock_H5_device_polish.md` (engrave `f6dd437`, spec committed at
`f6dd437`; brief at `design/agent-briefs/hashlock-H5-spec-R0-r0-fidelity-brief.md`).
Parent: `design/SPEC_hashlock_H2_device.md`. Rulings: `design/FOLLOWUPS.md` F-487, F-480,
F-484, F-485, F-488. Fork base: `seedhammer` main `b9a9a30`, executed in a detached
worktree at `/scratch/code/shibboleth/.tmp/h5-lens-fidelity` (removed), Go 1.26.7 from
`/scratch/code/shibboleth/.toolchain/go`.

ONE QUESTION: implemented literally, does each section fix exactly the follow-up it names
without contradicting the H2 spec or the fork, and is every claim about the fork true?

**Counts: 1 Critical / 5 Important / 4 Minor / 4 Nit.**

---

## Measurements taken (so the next round need not re-derive them)

Every number below was produced by a scratch test in the worktree, not read off a document.

| claim | source | measured |
| --- | --- | --- |
| confirm modal headroom, longest legal variant | §1.2 "about 107" | **107** (`assertModalBodyFits`, `confirmWarningBody`, 336 drawn) — TRUE, and matches `design/agent-reports/hashlock-H2-plan-R0-r0-fold-report.md:114` |
| `hashlockMethod.String()` | §1.1 | `"sha256"` / `"hardened"` (`gui/composer_hashlock.go:36-41`) — TRUE |
| `3cf5d421..b70a4c12` is the hardened anchor digest | §6 | TRUE (`hashlock/testdata/hashlock-v0.8.json:13` `hardened_h`; `cmd/emu/walk_hashlock_phrase.js:76`) |
| lead ink inside a nav button rectangle today | §3 / F-484 "152 px" | **152** exactly (lead size (440,44) at (20,44); nav rects (427,44)-(480,97), (427,133)-(480,186), (427,223)-(480,276)) — TRUE |
| lead ink inside a nav rect with §3.1's band | §3.2(a) | **0** |
| band width at `sh2DisplaySize` | §3.1 | `NavBtnPrimary` = 53x53, so band = 8..419 = **411 px** (panel wrap is 464 px) |
| lead line count at 411 px | §3.2(c) "at most two lines" | **2 lines** (44 px; a one-line control measures 23 px, `lead.LineHeight()` = 21) — the two-line rule HOLDS with the CURRENT copy, so **§3.3's fallback does NOT fire** |
| readout budget after §3.1 | §3.2(b) | `kbd.MaxHeight` = **209 at both wraps**, unchanged; F-481 cannot regress because `leadSz.Y` is 44 either way |
| new reconcile body fit (§1.1, `method: hardened`) | §1.3 | 159 drawn, **headroom 360** — fits with room |
| z-order / width of the digest on an error screen | brief | none: `warningBodyClip` (`gui/gui.go:595-604`) is x 6..423, width 417; the nav column starts at x=427. `showError` → `showModal` → `ErrorScreen` (`gui/slip39_polish.go:23-38`), the same body the fit gate's `errorScreenBody` renders |
| §5's new sentence, longest noun + two-digit index | §5 | 134 drawn, **headroom 418** (both `"a hashlock preimage, not a seed"` and `"not a format this machine reads"` normalise to 26 chars, so they tie) — fits |
| "Nowhere later on the device shows the digest" | §1 | TRUE in the composer: `hashlockFirst8Last8` has exactly ONE call site (`gui/composer_hashlock.go:65`). But see I-1: the digest IS engraved — `md/compose.go:403-404` lowers `p.Hash` into the descriptor as `sha256(H)` |
| §2's citations | §2 | all exact: `gui/composer_state.go:35-38`, `gui/composer_hash.go:177-199` (+ call site `:237`), `gui/composer_shape.go:356`, `gui/composer_copy.go:467-471` |
| a map field on `composerState` is safe | §2.1 | yes — no production code copies `composerState` by value or compares it (`composerApplyShapeEdit`, `gui/composer_discard.go:144-156`, mutates through the pointer) |

---

### C-1 — §2.1's `phraseDigests` has no initialisation, and the one production construction site leaves it nil

**Section:** §2.1 (`composerState` carries `phraseDigests map[[32]byte]struct{}` … "HOLD inserts `h`").

**Counterexample.** `composerState` is built in exactly one production place:

```
$ grep -rn "composerState{" --include=*.go . | grep -v _test
gui/composer_flow.go:34:	st := &composerState{reg: &seedRegistry{}, bound: composerBoundFrom(ctx.sysw)}
```

No keyed field for `phraseDigests`, and §2 states no place where the map is made. Implemented
literally — "HOLD inserts `h`" as `st.phraseDigests[h] = struct{}{}` — that is an assignment to
a nil map:

```
$ cat /tmp/h5nil/main.go
type composerState struct{ phraseDigests map[[32]byte]struct{} }
func main() {
	st := &composerState{}              // gui/composer_flow.go:34, verbatim shape
	var h [32]byte
	defer func() { fmt.Println("PANIC:", recover()) }()
	st.phraseDigests[h] = struct{}{}    // H5 SPEC §2.1: "HOLD inserts h"
}
$ go run .
PANIC: assignment to entry in nil map
```

The panic fires **at HOLD**, i.e. after the operator has typed up to 100 characters of phrase and
waited out a ~10 s hardened derivation, and `composerState` is RAM (H2 §4.4: "A power loss ends
the composition"), so the whole composition is lost. The predecessor field was a `bool`, which has
a usable zero value; the replacement does not. Every one of the ~20 `&composerState{…}` literals
in `gui/*_test.go` has the same shape, so the tests would not catch it either — they would panic
only on the paths that reach HOLD.

**SUGGESTION.** Make §2.1 normative about the lifetime, not only the type: either say
`composerFlow` builds the state with `phraseDigests: map[[32]byte]struct{}{}`, or say HOLD
lazily creates it before the first insert. Naming the construction site (`gui/composer_flow.go:34`)
in the spec is what makes the plan's implementer unable to miss it.

---

### I-1 — §1.2 declines half of F-487's explicit ruling on a measurement that does not hold for the ruled edit

**Section:** §1.2 ("The confirm modal's body (`composerCopyHashlockConfirm`) is UNCHANGED … 'and
this digest' plus a repaired 'Without both' clause costs more than the 27 to spare.").

F-487's ruling is not one remedy, it is two, and it says so:

> **RULING 2026-09-05 (operator, walked live: "I agree with your recommendation").**
> Both: (1) the reconcile screen repeats `hash <first8>..<last8>  method: <m>` above its
> sentence …; (2) the confirm modal's write-down line becomes "Write down this phrase, the
> method and this digest now." (about 16 characters into a modal with 107 characters of
> headroom -- re-measure with `assertModalBodyFits`).

§1 ships (1) and refuses (2). **Measured, the ruled edit passes the gate unchanged:**

```
today                                                normalised 336
today: 336 chars drawn in full, headroom 107 chars (margin 80)

F-487 ruling (2) VERBATIM, 'Without both' untouched  normalised 347 (delta +11)
...: 347 chars drawn in full, headroom 107 chars (margin 80)          <-- PASSES

F-487 ruling (2) + repaired clause                   normalised 351 (delta +15)
...: 351 chars drawn in full, headroom 64 chars (margin 80)           <-- FAILS
```

(`assertModalBodyFits` on `confirmWarningBody`, `composerConfirmBody(...)` wrapped exactly as
production draws it, `sh2DisplaySize`.) Headroom after the operator's own sentence is **107 —
identical to today**. It is only the spec's *own* addition, "Without all three", that costs the
gate.

The arithmetic frame is also unsound. §1.2 treats headroom as a character budget ("costs more
than the 27 to spare"); `gui/modal_fits_test.go:33-35` says exactly the opposite in its own words:
"It is not a character budget: capacity depends on how the words WRAP, not on how many there
are." The measurements above show it: **+11 characters cost 0 headroom, +15 cost 43.**

There *is* a good reason to be wary of ruling (2), and §1.2 does not give it: the sentence the
digest would join reads "They are not on this device and not on your plates", and the digest is on
both — it lives in `st.list.Paths[idx].Hash` and `md/compose.go:403-404` lowers it into the
engraved descriptor as `sha256(H)`. That is a truth problem, not a fit problem, and it is the
argument that would let the operator re-rule (2) knowingly.

**SUGGESTION.** Either implement F-487 (2) as ruled (it fits), or take the departure back to the
operator with the two measurements above and the "on your plates" observation. A draft spec
should not close a ruled remedy on a number that is false of the ruled text.

---

### I-2 — §4.3 picks the row "by LABEL … from `shTargets`", and `shTargets` exposes no labels

**Section:** §4.3.

`shTargets` returns rectangles and nothing else (`cmd/emu/screen_js.go:65-78`):

```go
out = append(out, map[string]any{
    "x": r.Min.X, "y": r.Min.Y,
    "w": r.Dx(), "h": r.Dy(),
    "cx": (r.Min.X + r.Max.X) / 2,
    "cy": (r.Min.Y + r.Max.Y) / 2,
})
```

It is built from `frameTargets` (`cmd/emu/screen.go:92-119`), which walks one vertical line
calling `d.Hit` and dedupes by rectangle — deliberately keeping no tag, "because a tag is a live
pointer into GUI state". There is no text on a target. The walk's existing helper already reflects
this: `chooseRow(i, expect, label, settle)` (`cmd/emu/walk_hashlock_phrase.js:165-183`) selects
`targets[i]` **by index** and uses `label` only in the two error strings.

So §4.3, implemented literally, is not implementable with the API §4.1 authorises. §4.1 adds
exactly one new emulator entry point (`window.shComposerPathHashes()`) and it is not this one.
The parenthetical "(H2 §5's own rule for production)" is also a mis-transfer: H2 §5 is about the
**Go** switch dispatching on named row indices in a struct (`payloadRows`/`phraseRow`/`hexRow`/
`noneRow`, `gui/composer_hash.go:216-240`), not about a JS walk resolving a label to a row.

**SUGGESTION.** Either drop §4.3 to what the API can do (assert the frame text contains
`Type a hashlock phrase` and that the target count equals the expected row count before tapping
row *i*, so an inserted row turns the walk red rather than silently moving it), or make §4 declare
the second emulator change it actually needs — a label on each target — and say what it costs
`frameTargets` to produce one without retaining a tag.

---

### I-3 — §4.1's "no production code path" is false: the composer state is unreachable from `cmd/emu`

**Section:** §4.1 ("The emulator exposes `window.shComposerPathHashes()` … (`cmd/emu`, js build
only; no production code path)").

`composerState` is unexported, and the composition's `st` is a **function-local** in the `gui`
package:

```
gui/composer_flow.go:33: func composerFlow(ctx *Context, th *Colors) {
gui/composer_flow.go:34: 	st := &composerState{reg: &seedRegistry{}, bound: composerBoundFrom(ctx.sysw)}
```

`cmd/emu` is `package main`. There is no exported accessor, no package-level handle, and no
existing seam that carries composer state out of `gui`. So the hook cannot be confined to
`cmd/emu`: it needs a seam **in `gui`**, which is the package the firmware is built from.

The fork already has the pattern and writes down its price. `gui/frame_hook.go` is
`//go:build !tinygo` with a `//go:build tinygo` twin, precisely so "the firmware must not merely
decline to use this hook, it must not carry it"; `gui/frame_hook_tinygo.go` records that its
per-frame call costs **zero** bytes, and then records that `plate_hook`'s equivalent per-job call
costs **486,697 bytes** — "written down rather than assumed in either direction".

**SUGGESTION.** Say in §4.1 that the seam is a `!tinygo` pair in `gui` following
`gui/frame_hook.go` / `gui/frame_hook_tinygo.go` (including the one-line registration in
`composerFlow`), that `cmd/emu` only publishes it to JS, and that §6's firmware-size gate must
state the delta for *this* hook rather than inheriting frame_hook's zero.

---

### I-4 — §1 changes the reconcile copy without folding H2 §4.5, while §3.3 folds H2 §4.2 for the same class of change

**Section:** §1.1 / §1.3 (contrast §3.3: "H2 §4.2 is folded to it").

The shipped reconcile string is pinned to the H2 spec by a test that diffs the two:

```
gui/composer_copy_test.go:140: {"composerCopyHashlockReconcile", "H2-4.5", composerCopyHashlockReconcile(),
gui/composer_copy_test.go:141:   "Before you fund this wallet, run ms hashlock with this phrase and method on the host and check the digest matches."},
```

`TestComposerCopyIsVerbatimFromTheSpec` ("compares every shipped string with SPEC §8 word for
word", `gui/composer_copy_test.go:146-158`) is the gate, and the row's section column names
**H2 §4.5** — whose block still carries that sentence verbatim (`SPEC_hashlock_H2_device.md:272-274`).
§1 rewrites the string and never folds H2 §4.5, so after H5 the `verbatim` column would hold text
that appears in no spec, and the gate would compare the code against itself.

Two smaller errors ride along in §1.3. "The reconcile body gets its own row in the modal-fit
table … and a row in the copy table" — **both rows already exist** and must be *updated*, not
added: `gui/modal_fits_test.go:340-343` and `gui/composer_copy_test.go:140-141`. And the copy
table's named test, `TestComposerCopyTableCoversEveryBody`, is only the coverage scan (it parses
`composer_copy.go` and requires a row per `composerCopy*` decl); the gate that H5's copy change
actually has to satisfy is `TestComposerCopyIsVerbatimFromTheSpec`.

**SUGGESTION.** Give §1 the fold clause §3.3 has: "H2 §4.5's reconciliation block is folded to
this text." Then say the two existing rows are updated (with their file:line), and name
`TestComposerCopyIsVerbatimFromTheSpec` as the gate the fold has to keep green.

---

### I-5 — §4.2's pre-hold read is not pinned to a moment, so §4.5's mutation gate need not fail

**Section:** §4.2 with §4.5.

§4.5 requires a second run "against a build with the HOLD assignment moved before the confirm
(mutation): the walk must FAIL on §4.2's pre-hold assertion". That only follows if the read
happens **while the confirm modal is up**. §4.2 says only "reads the hashes BEFORE the hold" — and
the walk has several earlier moments that satisfy those words: `Which hash?`
(`walk_hashlock_phrase.js:284`), the phrase screen (`:234`), the method pick (`:237`). A read taken
at any of them is `null` under both the honest build and the mutation, the mutation run passes,
and the gate proves nothing.

(The read itself is sound at the confirm modal: the GUI goroutine is parked in
`composerConfirmScreen`'s frame loop with the state quiescent, so a `js.FuncOf` callback sees a
settled `st`.)

**SUGGESTION.** Pin it: "after `waitFor("Write down this phrase")` returns and before
`hold(CONFIRM)`, `shComposerPathHashes()[idx]` is `null`." That sentence is what makes §4.5's
mutation run able to fail.

---

### M-1 — §2 and §6 name one test to rewrite; five sites reference the removed field

**Section:** §2.3, §6.

Removing `hashByPhrase` breaks, beyond `TestRemovePathReSyncsHashByPhrase`:

```
gui/composer_copy_test.go:144: composerCopyHashEveryPathFor(&composerState{hashByPhrase: true})
gui/composer_hashlock_test.go:704, :719-720   (the "last hash cleared" test)
gui/composer_hashlock_test.go:914, :916       (MUTATION: delete `st.hashByPhrase = true`)
gui/composer_hashlock_test.go:1025, :1037-1038
```

`gui/composer_copy_test.go:144` is the one that matters for design, not just for compilation: it is
the copy table's row for `composerCopyHashEveryPathFor`, and under §2 it needs a *composition*
(a path list plus a seeded digest set) rather than a one-field literal — which changes what that
row proves.

**SUGGESTION.** List the five sites in §6 and say what `composer_copy_test.go:144`'s row becomes.

---

### M-2 — §4.1 reads state no eye can see, which departs from the emulator's own written doctrine

**Section:** §4.1.

`cmd/emu/screen.go:76-83` states the reading rule: "IT IS A READING PRIMITIVE … It injects no
event, reaches no flow, and lets a walk do NOTHING a hand could not -- it says where the targets
are, which is what the operator's eyes do." `shComposerPathHashes()` reports the **stored** digest,
which no operator eye can reach — that is the entire point of F-485's stored-vs-displayed
assertion, and it is right, but it is a new category alongside `shScreen`/`shTargets`.

**SUGGESTION.** One sentence in §4.1 recording the departure and its bound ("a walk may READ
composition state it cannot see, never drive it"), placed where `screen.go`/`walk_js.go` state the
doctrine, so the next hook does not cite this one as precedent for driving.

---

### M-3 — §5 changes a body shared by eight nouns, and does not say which table gets the fit row

**Section:** §5.

`unlockNotPermittedBody` (`gui/unlock_kdf.go:390-393`) is the refusal for **every** class
`unlockRecordNoun` (`:404-425`) names — codex32 secret, BIP-39 mnemonic, output descriptor,
bitcoin address, debug command, md1/mk1 card, the `default`, and the preimage arm F-488 is about.
Adding "Remove that record on the host and seal the payload again." changes all of them. That is
almost certainly right (the advice holds for each), but the spec should say so, because §5 is
written as if it edited "the arm's" copy.

Measured, the new sentence fits at the longest noun with a two-digit index (134 drawn, headroom
418, `errorScreenBody`). §5 says "Fit re-measured (`assertModalBodyFits`…)" but names no table;
`unlockNotPermittedBody` has **no** fit row today.

**SUGGESTION.** Say the body is shared by all `unlockRecordNoun` arms, and name the table the row
joins (`TestModalsThisBlockTouchesAreDrawnInFull`, `gui/modal_fits_test.go:302-353`) with the
argument `unlockNotPermittedBody(&seal.RecordNotPermittedError{Index: 99, …})`.

---

### M-4 — the digest is still one tap from gone

**Section:** §1.

§1 puts the digest on a `showError` screen, which dismisses on Back *or* OK
(`gui/gui.go:400-410`, F-440). An operator who taps through before writing has no way back to it
except re-entering the phrase route on that path, re-typing the phrase and re-deriving (~10 s
hardened) to read the confirm modal again. That is recoverable, so it is not blocking — but it is
the same shape as F-487 itself, one screen later, and the spec does not say it was considered.

**SUGGESTION.** One line in §1 recording that the digest remains re-derivable by re-entering the
phrase route, so a future reader knows the case was weighed rather than missed.

---

### N-1 — `modalBodyMargin` is at `gui/modal_fits_test.go:52`, cited as `:51` twice

§1.2 and §8. Line 51 is the comment "// that does."; line 52 is `const modalBodyMargin = 80`.

### N-2 — §8's "rows at `:342,372,388`" — `:372` is not a row

`gui/modal_fits_test.go:372` is a comment line inside `TestConfirmScreensThisBlockTouchesAreDrawnInFull`'s
doc comment. The hashlock rows are `:340-343` (reconcile) and `:344-347` (HASH ON EVERY PATH) in
the `showError` table, and `:378-381`, `:382-385`, `:386-390` in the confirm table. `:342` and
`:388` are correct.

### N-3 — §8's `gui/unlock_kdf.go:395-415` (noun)

`unlockRecordNoun` is at `:404-425`; `:395` is the start of its doc comment. The range stops nine
lines short of the function it names.

### N-4 — §1.1's block does not say which line breaks are real

The fenced block shows four lines; the confirm body's precedent (`composer_copy.go:410-412`, real
`\n` after the hash and method lines, prose wrapped by the widget) settles it, and
`normalizeDrawn` makes the gates indifferent — but "with these exact strings" invites a literal
reading in which the prose carries a hard break too.

---

## Closing counts

**1 Critical / 5 Important / 4 Minor / 4 Nit.**

Not blocking, and worth keeping: §3 is the strongest section in the spec. Its two load-bearing
numbers (152 px of lead ink under Back; the band from `composerPageLines`) reproduce exactly, its
(c) gate holds with the current copy so §3.3's fallback never fires, and its (b) gate cannot
regress because the lead's height is 44 px at both wraps. §5 is correct and fits. §2's citations
are all exact and its value-set design is a genuine improvement on F-480's own "per-path array"
sketch — a value set has no splice discipline to get wrong, which is the C16 lesson §2.4 names.
