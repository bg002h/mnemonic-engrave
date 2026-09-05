# R0 round 0 — tests + citations lens — `SPEC_hashlock_H5_device_polish.md`

**Reviewer**: sonnet tier, independent tests+citations lens. **Base**: fork main
`b9a9a3039797639ee80a4cedb9225a033f3580b4` (confirmed via `git rev-parse`), engrave
master `f6dd437` (per spec header; not independently re-verified — engrave-side only).
**Worktree**: `/scratch/code/shibboleth/.tmp/h5-lens-tests` (detached at `b9a9a30`, Go
`/scratch/code/shibboleth/.toolchain/go/bin/go` 1.26.7), removed after use along with two
throwaway measurement test files (`h5_lens_band_test.go`, `h5_lens_scratch_test.go` —
never committed). One question: is every file:line/number/mechanism the spec cites true
at `b9a9a30`, and can every §6 test/mutation be executed as stated and fail on what it
names?

## 1. Citation table (§8 + section bodies)

| Cite | True/False | Evidence |
|---|---|---|
| `gui/composer_hashlock.go:62-84` (HOLD, assignment, reconcile) | TRUE | `:70 st.hashByPhrase = true`, `:82 showError(ctx, th, "Hash lock", composerCopyHashlockReconcile())`, `:83 return hashlockAssigned` |
| `gui/composer_hashlock.go:169-172` (lead) | TRUE | `:169-170 widget.Labelw(&ctx.B, ctx.Styles.lead, dims.X-2*8, th.Text, composerCopyHashlockPhraseLead())`; `:172 leadOp.Offset(leadBand.N(leadSz))` — `Rectangle.N` (`gui/layout/layout.go:41-46`) centres on `r`'s full width, confirming "centred on the whole panel" |
| `gui/composer_copy.go:409-423` (confirm body) | TRUE | `composerCopyHashlockConfirm` spans exactly 409-423 |
| `gui/composer_copy.go:443-446` (reconcile) | TRUE | `composerCopyHashlockReconcile` spans exactly 443-446, current 0-arg signature confirmed |
| `gui/composer_copy.go:458-471` (§8h forms) | MOSTLY — see N-2 | `composerCopyHashEveryPathPhrase` 461-466 is inside range; `composerCopyHashEveryPathFor` starts 468 but its own closing brace is at `:473`, 2 lines past the cited end |
| `gui/composer_state.go:35-38` | TRUE | `hashByPhrase bool` field + doc comment, exact range |
| `gui/composer_hash.go:177-199` | TRUE | `composerHashByPhraseSync` spans exactly 177-199 |
| `gui/composer_shape.go:356` (Remove arm sync) | TRUE | `:356 composerHashByPhraseSync(st)` inside the Remove-path callback |
| `gui/composer_paged.go:62-90` (band) | TRUE | `:87-90` `bandMargin=8; bandLeft=8; bandRight=dims.X-NavBtnPrimary.width-8; lineWidth=bandRight-bandLeft` matches spec verbatim |
| `gui/unlock_kdf.go:395-415` (noun) | PARTIAL — see I-2 | `unlockRecordNoun` actually spans `:404-424`; the cited range stops mid-`switch`, before the `ClassDebugCommand`, `ClassMDMK` and `default` cases (`:418-423`) — the `default` case ("not a format this machine reads", 32 chars) **ties for longest** with the Preimage case and is exercised by `gui/unlock_preimage_test.go`'s "record this machine does not read at all" row |
| `gui/unlock_kdf.go:391` (arm's sentence) | TRUE | `fmt.Sprintf("Record %d is %s. This payload cannot be unlocked here. Nothing was opened.", ...)` at exactly `:391` |
| `gui/modal_fits_test.go:51` (`modalBodyMargin = 80`) | OFF BY ONE — N-1 | actual line is `:52` (`const modalBodyMargin = 80`) |
| `gui/modal_fits_test.go` rows at `:342,372,388` | PARTIAL — see N-3 | `:342` (`composerCopyHashlockReconcile(),`) and `:388` (`composerCopyHashlockConfirm("b867db87...`) are real table rows; `:372` is a **doc-comment line** ("`// composerCopyHashlockReconcile instead.`"), not a table row |
| `cmd/emu/walk_hashlock_phrase.js:74-76` | TRUE | `ANCHOR`, `ANCHOR_SHA_H`, `ANCHOR_HARD_H` corpus constants, exact lines |
| `cmd/emu/walk_hashlock_phrase.js:232` | TRUE | `await chooseRow(0, "32-byte value", "Type a hashlock phrase");` exact line |
| `cmd/emu/walk_hashlock_phrase.js:286-329` | TRUE | Block runs exactly 286-329; `out.ok` (326-329) recomputes from `squash().includes()` checks whose underlying strings were already passed through `must()`/`mustNot()` (both of which `throw` on failure, confirmed by reading their bodies at `:123-147`) — so by the time :326 runs, every check it repeats has already succeeded once. "Recomputes from assertions that already threw" is literally true. |
| `hashlockMethod.String()` → `sha256`/`hardened` | TRUE | `gui/composer_hashlock.go:36-41` |
| Fold report `hashlock-H2-plan-R0-r0-fold-report.md` exists | TRUE | present at `design/agent-reports/` |
| Commit `a1fd139` (Remove-arm sync, "since a1fd139") | TRUE | `a1fd1398f`, message and diff match (`gui/composer_shape.go` among files touched) |

**Numeric spot-checks not already covered above**: modal-fit margin arithmetic in §1.2
("about 107 ... costs more than the 27 to spare") is internally consistent: 107 − 80
(margin) = 27. Confirmed against the measured value below.

## 2. Modal-fit headroom (item 2)

Ran `TestConfirmScreensThisBlockTouchesAreDrawnInFull` at `b9a9a30` (unmodified):

```
the hashlock confirm modal, longest variant (H2 §4.5): 336 chars drawn in full, headroom 107 chars (margin 80)
```

Matches the spec's "about 107" exactly.

Built the PROPOSED §1.1 reconcile body (`hash <first8>..<last8>` / `method: hardened` /
"Write this digest beside the phrase and the method. Before you fund this wallet, run ms
hashlock with them on the host and check the digest matches.") and ran it through
`assertModalBodyFits` on `errorScreenBody` (the same `showError → ErrorScreen` path §1.3
names):

```
H5 proposed reconcile body (method: hardened): 159 chars drawn in full, headroom 360 chars (margin 80)
```

**Fits, with 360 ≥ 80 headroom — comfortably clears the gate.**

Also measured §5's proposed unlock-refusal sentence (longest tied noun "a hashlock
preimage, not a seed", two-digit index):

```
H5 proposed unlock refusal (§5, longest noun, 2-digit index): 134 chars drawn in full, headroom 418 chars (margin 80)
```

Also fits comfortably. Neither proposed body is at risk under the modal-fit gate.

**Stray fact, not a spec citation but worth flagging (see M-1)**: `gui/composer_copy.go:441`'s
own doc comment claims "keeps the confirm modal's measured headroom (186) intact" — the
actual measured headroom (above, and reproducibly) is 107, not 186. Pre-existing, not
introduced by this spec, but H5 rewrites this exact function.

## 3. §3 band-width layout (item 3)

At `sh2DisplaySize` (480×320): `bandLeft=8`, `bandRight = 480 − NavBtnPrimary.width(53) − 8
= 419`, band width `411`. Today's width is `dims.X−2*8 = 464`.

Laid out the CURRENT lead text ("This screen does that hashing for you. Use a phrase you
have never used anywhere else.") with `ctx.Styles.lead` at both widths (throwaway test,
replicating `composer_hashlock.go:163-179`'s arithmetic exactly, plus a second test
printing the actual wrapped lines via `text.Layout` directly):

```
TODAY  width=464: line 1 "This screen does that hashing for you. Use a phrase you"
                  line 2 "have never used anywhere else."
BAND   width=411: line 1 "This screen does that hashing for you. Use a phrase"
                  line 2 "you have never used anywhere else."
```

Both wrap to **exactly 2 lines** (leadSz.Y = 44px at both widths). Downstream readout
budget (`kbd.MaxHeight − grid.Y − readoutGap`, `passphrase_keyboard.go:455`'s own
expression): **19px at both widths** — meets the `>= 19` (one readout line) floor exactly,
with no regression from banding.

**Conclusion for §3.3's branch**: the fallback (shorter, single-purpose lead text) is
**NOT triggered** — (c) "at most two lines" holds at the narrower band width with the
existing H2 §4.2 copy unchanged. A plan/implementer following this spec correctly stays on
the "no other copy change permitted" branch. I did not independently re-derive the "Today"
claim's cited pixel figure ("152 px of ink lies inside the Back button's rectangle") — out
of this brief's 7 numbered items, and it is a previously-settled (W-3) fact this spec
inherits rather than a new claim of its own.

## 4. Emulator `shTargets` / composer-state reachability (item 4) — I-1, I-2

**`shTargets()` does NOT expose row labels.** `cmd/emu/screen_js.go:65-75` returns, per
target, only `{x, y, w, h, cx, cy}` — pure geometry. `frameTargets`
(`cmd/emu/screen.go:92-115`) computes these from `d.Hit(...)`, and **discards the hit
tag**: `_, r, ok := d.Hit(image.Pt(x, y))` (`:97`). No text/label field exists anywhere in
this pipeline. Confirmed by reading `cmd/emu/walk_hashlock_phrase.js:165-181`'s
`chooseRow(i, expect, label, settle)`: `i` is a bare **index** into
`window.shTargets()`; the `label` parameter is used **only inside error-message template
strings** (`` `choosing ${label} (row ${i} of ${n})...` ``), never as a lookup key.

The spec's own citation for this ("H2 §5's own rule for production",
`SPEC_hashlock_H2_device.md:350-365`) is a **different mechanism entirely**: H2 §5 is a
Go-side rule about `composerHashEdit` dispatching on a struct of *named row indices*
(`payloadRows`/`phraseRow`/`hexRow`/`noneRow`) instead of a bare `switch sel`, so an
inserted row can't silently misdispatch — it says nothing about, and supplies no mechanism
for, reading an on-screen text label from JS.

**No existing seam carries `composerState` (or path hashes) out to a `Platform`/JS hook.**
`composerState` (`gui/composer_state.go:26`) is created and lives **only as the local
variable `st` inside `composerFlow`** (`gui/composer_flow.go:33-34`) — never stored on
`Context` (`gui/gui.go:64-93`, no composer-related field), `Platform`, or anywhere
reachable from outside that one function's stack frame. The two existing precedents for a
js-only "read the flow's internals" seam are `gui.FrameAware` (`gui/frame_hook.go:49-79`,
wired from the *generic* run loop at `gui/run_flow.go:262`) and `gui.PlateAware`
(`gui/plate_hook.go:32-55`, wired from the Engraver) — both `!tinygo`-gated optional
interfaces, but both hung on something OTHER than `composerFlow`. Implementing §4.1's
`window.shComposerPathHashes()` therefore requires a **new** hook of the same shape (most
likely a new optional interface on `Platform`, invoked from inside `composerFlow` where
`st` is in scope) that neither exists today nor is named by the spec's normative text or
its §8 citations.

Both are logged as Important findings below (I-1, I-2): the spec states these as if
straightforward JS glue against an existing API, but a plan built on it would discover,
mid-implementation, that two separate pieces of new Go-side plumbing are required — one of
which duplicates infrastructure this repo has already built twice (`FrameAware`,
`PlateAware`) and could reasonably follow the same pattern, but the spec doesn't say so.

## 5. `gui/unlock_preimage_test.go` (item 5)

Exists, 142 lines. `TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable` drives the
real flow and asserts `uiContains(got, "not a seed")` and `uiContains(got, "Nothing was
opened")` on the drawn frame (`:59-63`). `TestUnlockNotPermittedBodyNamesTheRecordAndTheKind`
table-tests `unlockNotPermittedBody` directly over 4 rows including one that exercises the
`default` switch case outside the spec's cited `:395-415` range (`"a record this machine
does not read at all"`, expecting `"not a format this machine reads"`, `:112-116`) — this
is the concrete instance of the truncated-citation finding above (I confirmed it's an
actually-tested case, not merely a theoretical one). Both tests ran green at `b9a9a30`.

## 6. Per-§6-mutation table

| §6 item | Mechanism exists today? | Test that fails, and how I know |
|---|---|---|
| §1 copy-table row; fit row (hardened) | `composerCopyHashlockReconcile` is 0-arg today (`composer_copy.go:443`); new signature doesn't exist | Not executable as literally stated. Measured the PROPOSED body directly (§2 above): fits with 360 headroom, so once implemented the fit row will pass; "return the old one-sentence body" mutation would fail on the missing `first8last8`/`method:` tokens, which the old body structurally cannot contain. |
| §1 flow test reaches frame with `3cf5d421..b70a4c12`, `method: hardened`, "Write this digest" | Not yet — reconcile fn is 0-arg | Not executable today. **Collateral risk (I-3, new)**: the EXISTING flow test `TestHashlockReconcileScreenIsReachableOnAMixedPolicy` (`gui/composer_hashlock_test.go:909`, confirmed green today) asserts `h.mustReach("run ms hashlock with this phrase")`. The proposed §1.1 body's second sentence reads "run ms hashlock **with them**" — the literal substring `"run ms hashlock with this phrase"` is gone. Implementing §1.1 without touching this existing assertion turns a currently-green test red. The walk's own `waitFor("run ms hashlock with this phrase", ...)` (`cmd/emu/walk_hashlock_phrase.js:318`) has the identical exposure, but the walk is already being substantially rewritten per §4 so it is less likely to be missed. |
| §2 `TestRemovePathReSyncsHashByPhrase` becomes the value-set test | Test exists TODAY at `gui/composer_hashlock_test.go:1016-1035` and is GREEN (ran it: `--- PASS`) against the CURRENT `st.hashByPhrase` bool. `composerAnyPathByPhrase`/`phraseDigests` do not exist yet. | Confirms the spec's premise (this test exists, to "become" something else) is TRUE. The described mutation ("predicate returns true when the set is non-empty") targets code that doesn't exist yet — not executable today. |
| §2 interruption edit-to-payload / two-paths-one-digest / same-digest-as-hex | None of these scenarios has an existing test | Proposed, new; not executable today (depends on `phraseDigests`). |
| §3 geometry test, MUTATION: restore panel-wide wrap → (a) fails | The specific lead-geometry test doesn't exist yet; the PRECEDENT pattern (`TestComposerPagedLinesNeverDrawUnderTheNavButtons`, `gui/composer_paged_geometry_test.go:199`, and its shared raster helpers `inkUnderNavOps`/`rasterInk`) exists and is directly reusable | Not executable today for the lead specifically. Independently confirmed (§3 above) that the CURRENT (would-be "restored") centered layout is exactly what the cited W-3 defect describes, and that the proposed banded layout keeps the same 2-line/19px readout outcome — consistent with the mutation description. |
| §4 the two walk runs | `shComposerPathHashes` doesn't exist (I-2); `chooseRow` is index-only (I-1) | Not executable today; no browser/playwright run performed (out of scope for this lens per the brief — Go-only worktree). |
| §5 frame assertion, MUTATION: drop the new sentence → fails | `unlockNotPermittedBody` doesn't have the new sentence yet | Not executable today. Measured the PROPOSED body (§2 above): fits with 418 headroom. Once added, `TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable` (`:59-63`) and `TestUnlockNotPermittedBodyNamesTheRecordAndTheKind`'s `want` lists are the natural place for the new assertion; dropping the sentence would fail whichever `want` entry names it. |
| Whole gates (4 packages, 24 gui shards, gofmt, vet, firmware size) | N/A — CI-mechanics restated, not a mutation | Not applicable to run in isolation here; `go vet ./gui/...` in this worktree shows 3 pre-existing `go1.26`-vs-`go1.25` file-version warnings unrelated to this spec (not introduced by anything reviewed here). |

## 7. Walk-line spot-check (item 7)

`walk_hashlock_phrase.js:74-76, 232, 286-329` — all TRUE, see citation table above and §7
detail under "Today" verification.

## Findings

### I-1 — §4.3's "chosen by LABEL ... from shTargets" describes a capability that doesn't exist
`shTargets()` (`cmd/emu/screen_js.go:65-75`, `cmd/emu/screen.go:92-115`) returns pure
geometry; `frameTargets` discards the `Hit()` tag entirely (`screen.go:97`). The walk's
`chooseRow`'s `label` argument (`walk_hashlock_phrase.js:165-181`) is decorative
(error-message text only), never a lookup key — row selection is 100% index-based today.
The spec's cited precedent, "H2 §5's own rule for production"
(`SPEC_hashlock_H2_device.md:350-365`), is a Go-side named-struct-dispatch rule unrelated to
the emulator API and supplies no such mechanism. Implementing §4.3 literally requires new,
unscoped engineering (extracting per-target text, e.g. by running `ExtractText` scoped to
each target's own rectangle) that neither §4 nor §8 names.

### I-2 — §4.1's `window.shComposerPathHashes()` has no reachable seam to hang off
`composerState` (`gui/composer_state.go:26`) exists only as the local variable `st` inside
`composerFlow` (`gui/composer_flow.go:33-34`); `Context` (`gui/gui.go:64-93`) carries no
composer-related field. The two existing js-only "read the flow's internals" precedents,
`gui.FrameAware` (`gui/frame_hook.go:49-79`) and `gui.PlateAware`
(`gui/plate_hook.go:32-55`), are both wired from the *generic* run loop / engraver, not from
`composerFlow`. A new hook of the same shape is needed and is neither described nor cited.
(Ironically, `composer_flow.go`'s own top-of-file doc comment is a warning about precisely
this failure class — "plans list components and omit the call that joins them" — for a
different defect in this same file.)

### I-3 — Implementing §1.1 verbatim breaks a currently-green test, uncounted in §6
`gui/composer_hashlock_test.go:909` (`TestHashlockReconcileScreenIsReachableOnAMixedPolicy`,
confirmed GREEN at `b9a9a30`) asserts `h.mustReach("run ms hashlock with this phrase")`.
§1.1's proposed body's second sentence is "...run ms hashlock **with them** on the host...",
which does not contain that substring. §6's own mutation table does not name this test as
needing an update, so an implementer who touches only `composerCopyHashlockReconcile`
without touching this line breaks a passing test as a side effect rather than by design.
(The walk's `waitFor("run ms hashlock with this phrase", ...)` at
`walk_hashlock_phrase.js:318` has the same exposure but is far less likely to be missed
since §4 already rewrites the walk substantially.)

### N-1 — `modalBodyMargin = 80` is at `:52`, not `:51` (off-by-one)
`gui/modal_fits_test.go:52`. Nit per the brief's own rule (off-by-one).

### N-2 — `composer_copy.go:458-471` truncates `composerCopyHashEveryPathFor` by 2 lines
The function's closing brace is at `:473`; the cited range ends at `:471`, cutting the
trailing `return composerCopyHashEveryPath()` and `}`. No hidden case (the cut lines are the
unremarkable default branch); wording-level, not a wrong fact.

### N-3 — `modal_fits_test.go` "rows at :342,372,388" — one of three isn't a row
`:342` and `:388` are real test-table entries; `:372` is a doc-comment line ("`//
composerCopyHashlockReconcile instead.`"), not a table row. The line number itself is real
and does carry relevant context (the declined-suggestion reasoning for §1.2's design), so
this is a mischaracterization rather than a wrong line number.

### M-1 — Stale headroom number in a fork source comment this spec's own change touches
`gui/composer_copy.go:441`'s doc comment claims "keeps the confirm modal's measured
headroom (186) intact"; the actual measured headroom (reproducible, see §2 above) is 107.
Pre-existing in the fork, not a spec citation, but H5 rewrites this exact function so an
implementer will see the wrong number in the neighboring comment.

## Closing counts

- Citations checked: 20 (table in §1, plus the modal-fit and band-width numeric claims).
- TRUE: 15. PARTIAL/truncated: 3 (N-2, I-2's related citation gap, unlock_kdf.go noun range
  feeding I-2's table). OFF-BY-ONE: 1 (N-1). MISCHARACTERIZED: 1 (N-3).
- **Important: 3** (I-1, I-2, I-3). **Minor: 1** (M-1). **Nit: 2** (N-1, N-2/N-3 combined as
  wording). **Critical: 0.**
- All measured fit/geometry numbers this spec makes forward-looking claims about (proposed
  reconcile body, proposed unlock-refusal sentence, band-width lead line count and readout
  budget) **hold** — no Important finding from the numbers themselves, only from two
  unscoped mechanism gaps (I-1, I-2) and one uncounted test-breakage (I-3).
