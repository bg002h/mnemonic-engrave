# H5 device polish — R0 round-1 fold verification

**Scope**: verify the round-0 fold of `design/SPEC_hashlock_H5_device_polish.md`, commit
`d36ede5` over `f6dd437`, against the three round-0 reports (`hashlock-H5-spec-R0-r0-{fidelity,journey,tests}.md`).
**Ground**: fork main `b9a9a30`, detached worktree `/scratch/code/shibboleth/.tmp/h5-r1`
(removed after use), Go 1.26.7 (`/scratch/code/shibboleth/.toolchain/go`). Read-only on both
repos; nothing committed; no sub-agents; no `.jsonl` read.

**Counts: 0 Critical / 4 Important / 4 Minor.**

---

## 1. Every C/I from the three reports — fixed, declined, or not

| Finding | Fold change | Verdict |
| --- | --- | --- |
| fidelity C-1 (nil-map panic on HOLD) | `composerNotePhraseDigest(st, h)` helper allocates on nil; zero-value-state test added (§6) | **FIXED** |
| fidelity I-1 (F-487 ruling(2) declined on a false measurement) | Ruling restored: write-down becomes "…and this digest now."; re-measured **347 drawn / headroom 107** (identical to today) | **FIXED** — re-measured myself: **347/107, exact match** |
| fidelity I-2 = tests I-1 = journey I-5 (§4.3 label pick impossible) | Withdrawn; index + existing `chooseRow(i, expect, label)` landing assertion (`waitFor(expect)`) kept as the safety mechanism | **FIXED** (sound decline — `chooseRow` already fails loud on a misdispatch via its post-tap `waitFor`, confirmed by reading `walk_hashlock_phrase.js:165-183`) |
| fidelity I-3 = tests I-2 = journey I-6 (§4.1 "no production path" false) | Seam named as a `!tinygo`/`tinygo` pair in `gui`, modelled on `gui/frame_hook.go`/`frame_hook_tinygo.go`; hook delta asserted 0 bytes | **FIXED** — confirmed the cited model files exist and match the described doctrine (`!tinygo` file that costs 0 bytes in the tinygo twin) |
| fidelity I-4 (§1 doesn't fold H2 §4.5) | New §1 item 4: "H2 spec fold... part of this leg" | **NOT CLEANLY FIXED** — see new finding R1-I-3 below: this new item misdirects part of itself to §4.7, contradicting §2 item 5 |
| fidelity I-5 (§4.2 pre-hold read not pinned to a moment) | Pinned: "after `waitFor(\"Write down this phrase\")` returns (the confirm modal is up) and BEFORE `hold(CONFIRM)`" | **FIXED** |
| tests I-3 = journey M-1 (needle "run ms hashlock with this phrase" would break) | Kept verbatim in the new reconcile body's first sentence | **FIXED** for the substring itself — but see new finding R1-I-4: the enclosing test does not compile |
| journey I-1 (mismatch consequence unstated) | "If they differ, do not fund this wallet: build it again." added | **FIXED** in substance; headroom claim is wrong — see R1-I-1 |
| journey I-2 (`chars: <n>` dropped from reconcile screen) | `method: <m>   chars: <n>` line added, spelled as the confirm modal spells it | **FIXED** |
| journey I-3 (§8h "or" on a mixed wallet) | "Back up every phrase and its method, and every preimage plate, separately." | **FIXED** in substance; "the new sentence is shorter" is false — see R1-I-2 |
| journey I-4 (0-based index unstated) | "(records count from 0)" added; re-measured **153 drawn / headroom 397** | **FIXED** — re-measured myself: **397, exact match** (both the Preimage noun and the tied `default` noun) |
| journey I-7 (stored-vs-displayed assertion has no mutation) | Third controller run (c): stored hash perturbed one byte after assignment, must fail on the stored-equals-displayed assertion and no earlier one | **FIXED** |
| fidelity M-1..M-4, journey M-2/M-3/M-5, tests M-1 (Minors) | All folded per the R0-paragraph's mapping; spot-checked M-1 (composer_flow.go:33-34), M-2 (doctrine sentence), tests M-1/journey N-1 (186→107 comment fix cited, not yet landed in source — correctly deferred to implementation) | **FIXED / correctly deferred** |
| journey M-4, N-2 (declined) | Declined with stated reasons (token now on-screen already; wording) | **DECLINED-OK** |

Everything the three reports raised has a fold response. The gap is not in *coverage* — it's
that three of the fold's own **new** additions (the mechanism used to fix I-4, journey I-1's
headroom number, journey I-3's "shorter" claim, and M-1's incomplete site list) introduce
fresh defects that no round-0 lens could have seen, because the text is new.

## 2. New defects introduced by the fold (not in any round-0 report)

### R1-I-1 — §1 item 1's headroom claim (205/320) is stale; the actual folded text measures 186/339

§1 item 1 cites "journey I-1/I-2" for **"205 characters drawn in full, headroom 320."** I
rebuilt the *exact* fenced body (`hash  b867db87..edbc96cb\nmethod: hardened   chars: 100\n`
+ the literal prose line, byte-identical — extracted programmatically from the spec file, not
retyped) and ran it through `assertModalBodyFits` on `errorScreenBody` at `sh2DisplaySize`:

```
FOLD reconcile body (hardened, chars: 100): 186 chars drawn in full, headroom 339 chars (margin 80)
```

**186/339, not 205/320.** I confirmed why: 205 is exactly what the *pre-fold* (f6dd437)
proposal measured — the body that still had "Write this digest beside the phrase and the
method." as a leading sentence and "run ms hashlock **with them**" (journey's own I-1
measurement, made before the fold rewrote the sentence and added `chars: <n>`). Reproduced
directly:

```
OLD-STYLE (journey I-1) normalized length: 205
```

The fold dropped that leading sentence, changed "with them" back to "with this phrase and
method," and added the `chars:` field — three edits that change the count — but reused
journey's number for the *old* text rather than re-measuring the *new* one it was writing.
The passing margin doesn't change (339 ≥ 80 either way), so this is not a funds-safety miss,
but it is a false number introduced in this fold, citing a source that no longer measures what
the citation claims.

**Fix for the next fold**: replace "205 characters drawn in full, headroom 320" with **186
drawn, headroom 339**.

### R1-I-2 — §2 item 5 claims the new §8h sentence is shorter; it is 5 characters longer

§2 item 5: *"Fit: 160 drawn today, headroom 378; the new sentence is shorter."* Measured both
bodies on `errorScreenBody`:

```
old sentence normalized len: 58   (today's full body: 160 drawn, headroom 378 -- matches spec)
new sentence normalized len: 63   (folded body: 165 drawn, headroom 378)
```

The **new sentence is 5 characters longer**, not shorter (63 vs 58 normalized; full body 165
vs 160 drawn). Headroom happens to measure identically (378 both ways) because the headroom
search is quantized to whole filler words, not characters — a coincidence, not evidence the
sentence shrank. The claim "the new sentence is shorter" is false; the substantive fit
conclusion (still comfortably clears the 80-char margin) is not affected.

**Fix for the next fold**: replace "the new sentence is shorter" with "165 drawn (was 160),
headroom unchanged at 378."

### R1-I-3 — §1 item 4 sends part of itself to H2 §4.7, contradicting §2 item 5

§1 item 4 (new, added to fold fidelity I-4): *"§4.5's write-down sentence takes item 2's
text; §4.5's post-HOLD reconcile clause **and §4.7** quote item 1's body."*

§2 item 5 (separately, correctly): *"...Copy-table row updated; **H2 §4.7 folded to it**"*
(§2's "every ... and every" text).

These name the **same H2 section** for **two different replacement texts**. I checked H2's
actual content: the reconcile sentence ("run ms hashlock with this... digest matches") occurs
exactly once, at `SPEC_hashlock_H2_device.md:272`, inside §4.5; the §8h banner sentence ("Back
up the phrase and its method, or the preimage plate...") occurs exactly once, at `:340`,
inside §4.7. §4.7 contains **no** reconcile-clause text to fold item 1's body into — that
content lives entirely in §4.5's own "drop order" paragraph. §1 item 4's mention of "§4.7" is
simply wrong; it duplicates and contradicts §2 item 5's own (correct) instruction for the same
section. A hostile implementer following §1 item 4 literally would try to write item 1's
reconcile text into H2 §4.7, directly conflicting with §2 item 5.

**Fix for the next fold**: drop "and §4.7" from §1 item 4; it belongs only to §2 item 5.

### R1-I-4 — the fold's own cited test does not compile once §2 removes `hashByPhrase`

§6's §1 bullet says: *"`TestHashlockReconcileScreenIsReachableOnAMixedPolicy` (:909) and the
walk's :318 needle stay green by construction (the substring is kept) — run them."*

`gui/composer_hashlock_test.go:909` is indeed `h.mustReach("run ms hashlock with this
phrase")` inside that function — but the **same function**, seven lines later, asserts:

```go
// gui/composer_hashlock_test.go:916
if !st.hashByPhrase {
    t.Fatal("the phrase route did not record that this hash was set by phrase")
}
```

§2 item 1 **removes** the `hashByPhrase` field entirely, replacing it with
`phraseDigests map[[32]byte]struct{}`. Once that lands, this line is a compile error — the
whole `gui` package test binary fails to build, not merely one assertion going red. The claim
"stays green by construction" is true only for the one substring at `:909`; it is false for
the function as a whole, which cannot even run.

This traces back to fidelity M-1, which listed this exact site
(`composer_hashlock_test.go:914, :916`) among "five sites that reference the removed field."
§2 item 3's list of what §6 must cover names the production sites and "the two tests that
mutate the deleted function" (`TestRemovePathReSyncsHashByPhrase`; the `No hash lock` row test
at `:704-720` that names `composerHashByPhraseSync` in a MUTATION comment) — but
`TestHashlockReconcileScreenIsReachableOnAMixedPolicy` is a **third**, distinct test function
(confirmed: `awk` on function boundaries places `:909`/`:916` inside
`TestHashlockReconcileScreenIsReachableOnAMixedPolicy` at `:882`, `:704/:719-720` inside
`TestComposerHashEditDispatchesByRowLabel` at `:644`, and `:1025/:1037-1038` inside
`TestRemovePathReSyncsHashByPhrase` at `:1016`) — and it is named nowhere as needing its
`st.hashByPhrase` assertion replaced (e.g. with `composerAnyPathByPhrase(st)` or a
`phraseDigests` membership check).

**Fix for the next fold**: add `TestHashlockReconcileScreenIsReachableOnAMixedPolicy`'s
`:916` assertion to §2 item 3's/§6's list of sites needing an update, and correct "stays green
by construction" to acknowledge the compile dependency on that update landing first.

## 3. Minor/Nit — citation looseness (does not block)

- **§8's "confirm rows at `:342` and `:388`"** — `:388` is a real confirm-modal-table row
  (`TestConfirmScreensThisBlockTouchesAreDrawnInFull`); `:342` is
  `composerCopyHashlockReconcile()`, a row in the **error-screen-body** table
  (`TestModalsThisBlockTouchesAreDrawnInFull`), not the confirm-modal table. Grouping both
  under "confirm rows" mislabels `:342`. The line number itself is correct (this is the same
  class of finding round 0 rated Nit for `:372`, not a new severity tier).
- **§2's items are numbered 1, 2, 3, 5, 4** in reading order (confirmed via the diff and the
  raw file: item "5. §8h's phrase form..." appears *before* item "4. Index identity...").
  Cosmetic — no content lost — but a hostile-implementer read stumbles here.
- **`composer_copy_test.go:130-137`**, cited for "the confirm body's existing rows carry the
  new sentence," spans the `composerCopyHashlockConfirm` row (`:130-135`, the one that
  changes) *and* the unrelated `composerCopyHashlockRelation` row (`:136-137`, unaffected).
- **`gui/unlock_kdf.go:391-393`** (in §5's body text) vs **`:388-393`** (in §8's citation
  table) cite two different ranges for what both call "the shared body" of the same function;
  `:391-393` omits the function's own signature line at `:390`.

## 4. Checks executed

- Re-measured, in the detached worktree, via `assertModalBodyFits`:
  - Baseline confirm modal: **336 drawn / headroom 107** (matches "today").
  - Fold-ruled confirm modal (item 2): **347 drawn / headroom 107** — matches spec exactly.
  - Declined longer repair: **361 drawn / headroom 64** (fails the 80 margin, confirming the
    decline; the exact drawn count differs slightly from the fold's terse commit-subject
    aside "351/64", but that number appears only in the commit message, not the spec body,
    and the substantive headroom-64/decline claim is correct).
  - Fold reconcile body (item 1): **186 drawn / headroom 339** — see R1-I-1.
  - §8h phrase form, baseline and fold: **160/378** and **165/378** — see R1-I-2.
  - §5 unlock refusal, fold (both tied nouns): **153 drawn / headroom 397** — matches spec
    exactly.
- Re-measured lead-ink-under-Back geometry directly (reproducing `composerPageLines`'s band
  arithmetic and `composer_hashlock.go`'s lead layout by hand against `rasterInk` +
  `navButtonRects`): **152 px today, 0 px with the band, band width 411 px
  (`sh2DisplaySize.X`(480) − `NavBtnPrimary`(53) − 8 − 8), lead height 44 px at both widths**
  (2 lines) — all match §3 and the fold's "verified true and kept" claim exactly.
- Re-grepped every new/changed citation: `gui/composer_flow.go:33-34`,
  `gui/frame_hook.go`/`frame_hook_tinygo.go`, `gui/screen.go:95-98`, `seal/record.go:69`,
  `gui/modal_fits_test.go:52`, `:342`, `:388`, `gui/composer_copy.go:388-473` range,
  `gui/unlock_kdf.go:388-425` range, `cmd/emu/walk_hashlock_phrase.js:74-76,232,286-329` — all
  exist at the cited (or near-cited, see §3 minors) lines at `b9a9a30`.
- Confirmed "records count from 0" appears 13 times in
  `crates/me-cli/src/main.rs` (13, exact).
- Confirmed the corpus anchor phrase "correct horse battery staple" is 28 characters,
  matching §6's "`chars: 28`" claim.
- Grepped for superseded phrasing: "Without all three", "with them", "Write this digest
  beside", "Nowhere later on the device shows the digest", the old citation lines
  (`:51`, `:395-415`, `:458-471`) — all absent from normative text (the one "Without all
  three" hit and the one ":372" hit are both inside the explanatory R0-fold paragraph,
  correctly describing what was declined/miscited, not left as live claims).
- Hostile-implementer read of the whole folded spec end to end.

## Closing

**0 Critical, 4 Important (R1-I-1..R1-I-4), 4 Minor.** The fold addressed every Critical and
Important from the three round-0 reports in substance — the underlying design decisions (nil
map helper, restored ruling, withdrawn label-pick, `!tinygo` seam, third walk run, 0-based
note, mismatch sentence, `chars:` field, "every...and every") are all sound and, where
numerically checkable, mostly correct. But this round's own new text introduces two false
measurement claims (R1-I-1, R1-I-2), one internal contradiction with §2 over which text H2
§4.7 receives (R1-I-3), and one incomplete fix that leaves a named, cited test unable to
compile while explicitly claiming it "stays green" (R1-I-4) — the same failure mode this
project's own standing rule warns about: a fold is authorship and re-earns the gate.

**NOT GREEN.** Fold R1-I-1..R1-I-4, then re-review.
