# SPEC — Hashlock H5: device polish (F-487, F-480, F-484, F-485, F-488)

**STATUS: R0 GREEN 2026-09-05 (0 Critical / 0 Important open).** Round 0 (spec `f6dd437`): fidelity + design (opus, `hashlock-H5-spec-R0-r0-fidelity.md`, 1C/5I/4M/4N), tests + citations (sonnet, `-tests.md`, 0C/3I/1M/2N), journey walk (opus, `-journey.md`, 0C/7I/5M/2N); fold `d36ede5`; r1 fold verification (sonnet) NOT GREEN 0C/4I/4M -- all four introduced by the fold (two numbers composed from other bodies' measurements, a §4.7 contradiction, an unlisted test site) -- folded at `44b1690` + `d206a2e`; r2 fold verification (sonnet, `hashlock-H5-spec-R0-r2-fold-verification.md`) **GREEN**. Lens-closure: fidelity, tests/citations, journey, fold-verification x2. **Amended by the PLAN's R0 round 0 (see `## Plan-round fold` at the end): the spec's R0 verdict is unchanged, but five normative items moved and six numbers were re-measured, because a plan-round lens is still a lens on this spec's text.** Every number in this spec is a measurement of the text as written. Base: seedhammer fork main `b9a9a30`
(H2 merged at c284484; H3 seam corpus at b9a9a30). Parent spec: `SPEC_hashlock_H2_device.md`
(GREEN 55ee7a4, H3-folded at 657f40f/8d5139d). Every citation below was measured at
`b9a9a30`; re-grep at plan time.

## §0. Why this stage exists

H2 shipped the phrase route; its post-implementation reviews and the ultracode lenses left
five defects that change what the operator sees or what the walk can prove, each ruled by
the operator on 2026-09-05 (FOLLOWUPS F-487, F-480, F-484, F-485, F-488). This stage folds
them into the H2 spec's sections and ships them as ONE fork leg. Out of scope: F-483
(secret handling, logged), F-489 (host, next `me` change), the idle-timer note.

## §1. F-487 -- the reconcile screen repeats the digest and count; the write-down asks for the digest

**Today** (`gui/composer_hashlock.go:62-84`): after HOLD assigns the hash, `showError(ctx,
th, "Hash lock", composerCopyHashlockReconcile())` draws *"Before you fund this wallet, run
ms hashlock with this phrase and method on the host and check the digest matches."*
(`gui/composer_copy.go:443-446`). The digest the operator is told to compare has just left
the screen with the confirm modal, and the write-down line (`composer_copy.go:419-420`)
asks for the phrase and the method only. The token IS shown once more later -- the Done
consent screen prints the same `first8..last8` (`composerDigestShort` ==
`hashlockFirst8Last8`, verified by the journey lens) -- and it is re-derivable by re-entering
the phrase route and retyping (about 10 s hardened); neither is where the check is asked.

**Normative.**

1. `composerCopyHashlockReconcile` takes `(first8last8, method string, chars int)` and
   returns exactly this, where `\n` marks the real line breaks (the renderer wraps the rest):
   ```
   hash  <first8>..<last8>\n
   method: <m>   chars: <n>\n
   Before you cut plates, run ms hashlock with this phrase and method on the host and check the digest matches. If they differ, do not fund this wallet: build it again.
   ```
   `<m>` is `hashlockMethod.String()` (`sha256` | `hardened`); the second line is spelled as
   the confirm modal spells it (`composer_copy.go:410-411`), so the two screens read alike
   and `chars: <n>` -- H2 §4.5's reconciliation field -- is present at the moment of
   reconciliation. The first sentence keeps the substring *"run ms hashlock with this phrase"*
   verbatim: `TestHashlockReconcileScreenIsReachableOnAMixedPolicy`
   (`gui/composer_hashlock_test.go:909`) and the walk (`walk_hashlock_phrase.js:318`) use it as
   their needle. **"Before you cut plates", not "before you fund this wallet"** (plan-round
   journey M-2): this screen is drawn inside `composerShapeFlow`, and the stub screen, seating
   and ~21 minutes per plate of engraving all follow it in the same `composerFlow` -- and the
   digest is IN the engraved md1, so a divergence found after the plates are cut costs every
   plate. Funding stays the deadline in the mismatch sentence; the first sentence names the
   threshold the operator is standing at. Measured on `errorScreenBody` at `sh2DisplaySize` for
   exactly this body (`hardened`, `chars: 100`; plan-round fold): 181 characters drawn in full,
   headroom 339 against the 80 margin. Drawn where it is drawn
   today: its own `showError` screen right after HOLD, for every phrase-set hash.
2. The confirm modal's write-down sentence becomes *"Write down this phrase, the method and
   this digest now."* -- the operator's second ruled remedy, as ruled, byte-identical. The
   sentence AFTER it becomes *"The phrase and method are not on this device."*, and the third
   is unchanged (*"Without both, this path can never be spent."*).

   **Why the second sentence had to move with the first** (plan-round journey I-1). The
   shipped H2 text was *"They are not on this device and not on your plates"*, true of a
   two-item list; adding "and this digest" to the list made it FALSE of the digest, which is
   compiled into the descriptor as `sha256(H)` (`md/compose.go`), written whole by the wire
   encoder, and engraved verbatim in the md1 this composer cuts -- so the one item that IS
   recoverable from the plates was being called absent from them, on the screen whose job is to
   define the backup burden. "Without both" also acquired a three-item antecedent, and the
   nearest-pair reading ("the method and this digest") makes the PHRASE look optional on the
   one screen that exists to stop an unspendable path. Naming the two subjects fixes both at
   once and leaves "both" one antecedent. Measured (plan-round fold): 343 characters drawn in
   full, headroom 107, identical to today's 107 -- headroom is a line budget, not a character
   budget (`gui/modal_fits_test.go:33-35`), and this edit adds no line. The longer repair
   *"Without the phrase and method, ..."* measured 361 drawn / headroom 64 and is NOT used;
   so did *"The phrase and method are not on this device and not on your plates"*, and both
   FAIL `TestConfirmScreensThisBlockTouchesAreDrawnInFull` outright (measured, plan-round
   fold) -- which is why the claim is SCOPED rather than merely lengthened.
3. Both bodies get their rows: the reconcile body in the error-body fit table
   (`assertModalBodyFits`, longest content = `hardened`, `chars: 100`) and in the copy table
   (`TestComposerCopyTableCoversEveryBody`); the confirm body's existing rows carry the new
   sentence (`composer_copy_test.go:130-135`, `modal_fits_test.go:388`).
4. H2 spec fold, part of this leg (the copy-verbatim test `TestComposerCopyIsVerbatimFromTheSpec`
   diffs the code against `SPEC_hashlock_H2_device.md`): §4.5's write-down sentence takes
   item 2's text and §4.5's post-HOLD reconcile clause quotes item 1's body (the reconcile
   text lives only in §4.5; §4.7 changes only through §2.5's two sentences). The
   toolkit manual's "On the SeedHammer II" section (`docs/manual/src/40-cli-reference/43-ms.md`)
   quotes the old reconcile sentence and is re-quoted in the same cycle (a toolkit docs
   commit, `make lint`), not left to a later stage (journey M-2).
5. The emulator walk asserts the reconcile screen carries the same `first8..last8` token
   AND the same `chars: <n>` the confirm modal carried (§4).

## §2. F-480 -- hash provenance is per digest, not one flag per policy

**Today**: `composerState.hashByPhrase bool` (`gui/composer_state.go:35-38`) is set on HOLD
(`composer_hashlock.go:70`), cleared only when NO path carries a hash
(`composerHashByPhraseSync`, `gui/composer_hash.go:177-199`, called from the `No hash lock`
row and, since a1fd139, from the Remove arm at `gui/composer_shape.go:356`), and read by
`composerCopyHashEveryPathFor` (`composer_copy.go:467-471`) to choose §8h's phrase form.
Replace a phrase-set hash on path 1 with a payload row while path 2 keeps a hex hash, and
Done still names a phrase the composition no longer has.

**Normative.**

1. `hashByPhrase` is REMOVED. In its place `composerState` carries
   `phraseDigests map[[32]byte]struct{}` -- the set of digests that were assigned through
   the phrase route in this composition. HOLD inserts `h` through one helper,
   `composerNotePhraseDigest(st, h)`, which allocates the map when it is nil: `composerState`
   is constructed as a zero-value literal at its ONE production site (`gui/composer_flow.go:34`)
   and in every test, and an assignment into a nil map panics (fidelity C-1, demonstrated).
   A test constructs the state exactly as `composerFlow` does and holds once. Nothing ever
   deletes from the set (a value set cannot go stale: a digest no path carries is simply
   never matched).
2. `composerAnyPathByPhrase(st) bool` is true iff some `p` in `st.list.Paths` has
   `p.Hash != nil` and `*p.Hash` is in `phraseDigests`. `composerCopyHashEveryPathFor`
   uses it. §8h's phrase form therefore names the phrase exactly when a CURRENT path's hash
   came from a phrase -- H2 §4.7's condition, now per digest.
3. `composerHashByPhraseSync` and both its call sites are deleted. Remove path, the
   `No hash lock` row, `Type 64 hex` and payload rows need no bookkeeping: they change
   `p.Hash`, and the predicate reads `p.Hash`. The six sites that reference the removed
   field today (`composer_state.go:35-38`, `composer_hashlock.go:70`, `composer_copy.go:469`,
   `composer_hash.go:177-199` incl. the `No hash lock` call, `composer_shape.go:356`, and
   `composer_hashlock_test.go:916` where `TestHashlockReconcileScreenIsReachableOnAMixedPolicy`
   asserts `st.hashByPhrase` -- it becomes `composerAnyPathByPhrase(st)`) and the
   two tests that mutate the deleted function (`TestRemovePathReSyncsHashByPhrase`; the
   `No hash lock` row test that names `composerHashByPhraseSync` in its MUTATION comment)
   are all listed in §6; `composer_copy_test.go:144`'s row keeps its section and drives
   `composerAnyPathByPhrase` through a state built for it.
4. Index identity is not used anywhere in this design (C16's lesson: "Remove path" splices
   the slice). Two paths sharing one phrase digest are both by-phrase; a path whose phrase
   digest is later replaced by the SAME digest typed as 64 hex is still by-phrase (the
   digest was derived here once; the backup burden is unchanged) -- state this in the code.

5. §8h's phrase form (`composerCopyHashEveryPathPhrase`, `composer_copy.go:461-466`) ends
   *"Back up every phrase and its method, and every preimage plate, separately."* -- "every"
   and "and", not "the" and "or": on a mixed wallet (one phrase path, one payload-row path)
   both backups are needed, one per path, and the old sentence offered a choice (journey I-3,
   constructed and quoted). Fit: 165 drawn (was 160), headroom unchanged at 378 (r1 verification).
   Copy-table row updated; H2 §4.7 folded to it.

   It OVERCOUNTS on an all-phrase wallet and on a phrase digest re-typed as 64 hex, neither of
   which has a preimage PLATE (plan-round journey M-4). That is the safe direction -- it asks
   the operator to look for a backup they do not have, where the reverse lets them stop looking
   for one they do -- and counting exactly would need three variants of this body. No change;
   the decision is RECORDED in a comment beside the sentence so it is not re-opened.

6. §8h's PLAIN form (`composerCopyHashEveryPath`, `composer_copy.go:169-173` at `b9a9a30`)
   ends *"Back up every preimage separately."* -- the same undercount, on the sibling body
   (plan-round journey I-2). Two paths can carry two DIFFERENT digests, which is two different
   preimages the operator must hold, and the shipped sentence named one; the wallet is the
   mixed wallet of item 5 with the phrase path replaced by a second plate, and
   `composerCopyHashEveryPathFor` -- rewritten by this stage -- is what chooses between the two
   forms, so leaving one counted and one not would make them disagree about what spending
   needs. Fit, measured on `errorScreenBody` at `sh2DisplaySize` (plan-round fold): 133 drawn
   (was 131), headroom unchanged at 397. Copy-table row updated; H2 §4.7's paragraph quoting
   this form folded to it, alongside item 5's blockquote.

## §3. F-484 -- the phrase screen's lead wraps inside the page band

**Today** (`gui/composer_hashlock.go:169-172`): the lead wraps at `dims.X-2*8` and is centred
on the whole panel; measured, 152 px of its ink lies inside the Back button's rectangle
(its empty margin; 0 px of glyph or chip lost) -- W-3's margin, spent. `composerPageLines`
(`gui/composer_paged.go:62-90`) already wraps inside `bandLeft = 8 .. bandRight = dims.X -
NavBtnPrimary.width - 8` for exactly this reason.

**Normative.**

1. The lead is laid out with the SAME band as `composerPageLines` (width `bandRight -
   bandLeft`, offset to `bandLeft`), not centred on the panel.
2. Geometry gate, a test at `sh2DisplaySize`: (a) no lead ink inside any nav button
   rectangle; (b) the readout budget (`kbd.MaxHeight - grid.Y - readoutGap`) stays >= one
   readout line (19 px) -- F-481 must not regress; (c) the lead is at most two lines.
3. If (c) fails with the narrower band at `sh2DisplaySize`, the lead becomes exactly
   *"This screen does the hashing. Use a phrase you have never used anywhere else."* and
   H2 §4.2 is folded to it; the plan records which branch was taken with the measured
   line count. No other copy change is permitted.

## §4. F-485 -- the walk proves hold order and stored-versus-displayed

**Today** (`cmd/emu/walk_hashlock_phrase.js`): the walk asserts the displayed tokens and the
reconcile sentence, picks the phrase row by INDEX (`chooseRow(0, ...)` at :232), and
recomputes `out.ok` from assertions that already threw (:325-329). It passes when the hash is
assigned before the hold, and when the stored digest differs from the displayed one (CI's
gui tests catch both; the walk does not).

**Normative.**

1. The seam. `composerState` is a local of `gui.composerFlow` (`gui/composer_flow.go:33-34`)
   with no path to `cmd/emu`, so the hook is a `gui` package variable on the sanctioned
   `passphraseWidgetHook` / `frame_hook.go` model: `//go:build !tinygo` file declaring
   `composerStateHook func() []*[32]byte` (nil in production builds of the host tooling; set
   by `composerFlow` for the composition's lifetime and cleared on exit) with a
   `//go:build tinygo` twin that carries nothing, exactly as `gui/frame_hook.go` /
   `gui/frame_hook_tinygo.go` do ("the firmware must not merely decline to use this hook, it
   must not carry it"). `cmd/emu`'s `//go:build js` glue publishes it as
   `window.shComposerPathHashes()` returning each path's hash as 64-hex or `null`. Doctrine
   note (fidelity M-2): a walk may READ state only to assert that what the screen shows equals
   what is stored; it never drives through a hook. Firmware size: the tinygo twin is empty,
   so the delta attributable to this hook is asserted to be NO MEASURABLE COST in §6's size
   gate -- measured subtractively on the stage tree against the same tree with the stub file
   and both call sites deleted, not inherited from frame_hook's number, and with a positive
   control proving edits to that stub do reach the image. **"No measurable cost", not
   "0 bytes"** (plan-round fidelity M-2): the pair measured exactly 0 on one tree and -16 B
   (the hook-LESS image 16 bytes larger) on a tree differing only in four operator-facing
   string literals, so an exact 0 is layout luck and asserting it would assert the noise.
   What is asserted is that the counterfactual is not larger than the shipped image by any
   amount the instrument can attribute to this hook.
2. The hardened trial: after `waitFor("Write down this phrase")` returns (the confirm modal
   is up) and BEFORE `hold(CONFIRM)`, read the hashes and assert the edited path's is `null`;
   after the hold assert its `first8..last8` equals the token the confirm modal displayed,
   then that it equals the corpus's full 64-hex `hardened` digest, and that the reconcile
   screen carries the same token and the same `chars: <n>` (§1.5).

   **The displayed token is READ OUT OF THE MODAL'S FRAME, and the comparison runs BEFORE the
   corpus check** (plan-round fidelity I-1 = journey I-7 unmet). Comparing the stored digest's
   abbreviation against a FILE CONSTANT that is the abbreviation of the constant the corpus
   check uses is a tautology: once the corpus check passes it cannot fail under any device
   behaviour, and §4.5(c) -- the run that exists to make it falsifiable -- trips the corpus
   assertion instead. So the walk parses `hash <first8>..<last8>` out of the frame it captured
   and compares against that; the corpus check stays afterwards as the oracle for what the
   value should have been. The two are then independent: a perturbed stored digest fails only
   the first, and a screen and a policy AGREEING on a digest the corpus does not hold fails
   only the second.
3. Row picking stays by INDEX with the landing assertion (`chooseRow(i, expect, label)`):
   `shTargets` exposes rectangles only -- `frameTargets` returns bare `image.Rectangle`s and
   `gui/screen.go:95-98` drops the tag on purpose -- so a label pick would need a second gui
   seam for no safety gain the landing assertion does not already give (fidelity I-2, tests
   I-1, journey I-5: the original §4.3 is withdrawn). F-485's index note is recorded as
   answered this way.
4. `out.ok` is set to `true` only after the last assertion; no recomputation. The `cmd/emu`
   guard that reads this shape (`TestWalkOkContainsNoDriverSuppliedPlateCount`) examines
   EVERY `x.ok =` assignment in a walk, not the first: a walk's verdict is its LAST one, and a
   guard reading the first passes `out.ok = false; ...; out.ok = out.plates === 3` and reports
   it as checked (plan-round fidelity I-2). Its log line states only what it measured -- that
   every right-hand side is a constant -- and not where the assignment sits (journey N-1). The
   guard's own blind spot carries a table test.
5. Three controller runs against emulator builds from the branch, each recorded WITH the
   assertion that failed: (a) unmutated -- PASS; (b) the HOLD assignment moved before the
   confirm -- must FAIL on §4.2's pre-hold `null` assertion; (c) the stored hash perturbed
   after assignment (one byte) -- must FAIL on the stored-equals-displayed assertion and on
   no earlier one (journey I-7: without (c) that assertion is never shown able to fail).
   **The tree is restored between runs and the restoration is CHECKED, not assumed**
   (plan-round journey M-1): `git checkout -- <file> && git diff --quiet <file>` before each
   mutation and after (c), with the emulator rebuilt after the final restore. With (b) still
   applied, (c) throws at the pre-hold assertion and never reaches the one it exists to
   exercise -- a contaminated run recorded as a FAIL, satisfying §7's "all three recorded"
   while leaving §4.5(c) exactly as unproven as before.

## §5. F-488 -- the unlock refusal says what to do next

**Today** (`gui/unlock_kdf.go:391`): *"Record N is a hashlock preimage, not a seed. This
payload cannot be unlocked here. Nothing was opened."*

**Normative.** The sentence gains one more: *"Remove that record -- and any others like it --
(records count from 0) on the host and seal the payload again."* The index is 0-based
(`seal/record.go:69`) and the device said so nowhere while `me` says it thirteen times; once
the number is an instruction to delete, a 1-based reading deletes the record ABOVE the plate
-- in the package's own fixture, a seed (journey I-4).

**"And any others like it" is the plural case** (plan-round journey M-3). `seal.AdmitSection`
returns on the FIRST refused record, so a payload carrying two preimage plates is refused once
per plate -- another ~31 s KDF and another host re-seal each round -- and THE INDEX MOVES,
because removing record 1 renumbers everything after it. Constructed on
`[seed, plate, seed, plate]`: round 1 names `Record 1`, round 2 names `Record 2`, round 3
admits. An operator applying round 2's number to their ORIGINAL listing deletes a seed. The
clause costs 22 characters and stays far inside the margin.

The body is shared by every `unlockRecordNoun` arm (`gui/unlock_kdf.go:390-393`), so the
change applies to all of them; fit re-measured in the error-body fit table at the longest noun
and a two-digit index (**175 drawn / headroom 378** with this sentence, plan-round fold; it
was 153/397 without the plural clause); `gui/unlock_preimage_test.go`'s frame assertion gains
the new sentence and the clause. Documentation only (journey M-5): the re-sealed payload has a
new passphrase; the manual's unlock section says so.

**ERRATUM (plan-round fidelity M-4): there is no copy-table row for this body, and there
cannot be.** An earlier draft of this section asked for "the copy-table row updated".
`unlockNotPermittedBody` is not a `composerCopy*` function, and `composerCopyTable()`'s AST
scan covers only those, so no such row exists. What stands in its place is the
`assertModalBodyFits(t, tc.name, errorScreenBody, body)` row inside
`TestUnlockNotPermittedBodyNamesTheRecordAndTheKind` at the longest noun and a two-digit
index -- the fit row IS the gate for this body, and the frame assertion is the gate for its
text.

## §6. Tests (each with the mutation that must fail it)

- §1: copy-table rows (reconcile, confirm); error-body fit row (reconcile, hardened, 100
  chars); flow test through HOLD reaches a frame carrying `3cf5d421..b70a4c12`,
  `method: hardened   chars: 28` and "If they differ" -- MUTATION: return the old
  one-sentence body -> fails on the token; MUTATION: drop the mismatch sentence -> fails.
  The confirm-modal test gains "and this digest" -- MUTATION: old write-down line -> fails.
  `TestHashlockReconcileScreenIsReachableOnAMixedPolicy` keeps its :909 needle by construction
  (the substring is kept) and has its :916 assertion rewritten to `composerAnyPathByPhrase(st)`
  in the plan's provenance task, the first to land (it does not compile otherwise); the walk's :318 needle stays -- run both.
- §2: a zero-value-state HOLD test (`composerState{}` as `composerFlow` builds it; one phrase
  route to HOLD; no panic; the digest is in the set) -- MUTATION: assign into the map without
  the nil check -> panics. `TestRemovePathReSyncsHashByPhrase` becomes the value-set test
  (remove the phrase path, add a hex-hashed one, Done draws the plain form -- MUTATION:
  predicate returns true when the set is non-empty -> fails); the `No hash lock` row test
  loses its `composerHashByPhraseSync` MUTATION comment and gains the predicate's; the
  interruption lens's edit-to-payload scenario; two paths one digest; same digest re-typed as
  hex (driven through a production re-assignment of the SAME digest, and asserting the
  pointer really changed, so the case is not indistinguishable from "one phrase path" --
  MUTATION: compare `p.Hash` pointers instead of the digest value -> fails, as do all four
  positive rows of the predicate table); the mixed-wallet banner test asserts "every phrase
  and its method, and every preimage plate" -- MUTATION: restore "or" -> fails; a two-plate
  banner test asserts the PLAIN form's "Back up every preimage separately." -- MUTATION:
  restore "Back the preimage up separately." -> fails, and so does the copy table.
- §3: the geometry test of §3.2 -- MUTATION: restore the panel-wide wrap -> (a) fails.
- §4: the three walk runs of §4.5, each recorded with the failing assertion's name; the
  `ok`-guard's own table test over synthetic assignment shapes -- MUTATION: read only the
  first `x.ok =` match -> the row whose offender is the LAST assignment fails.
- §5: frame assertion -- MUTATION: drop the new sentence -> fails; MUTATION: drop
  "(records count from 0)" -> fails; MUTATION: drop "-- and any others like it --" -> fails.
- Structural: the two helpers this stage inserts beside existing functions
  (`composerTextBand`, `composerFlowExit`) must not capture their neighbours' doc comments --
  a test over the four symbols requiring each to carry its OWN doc comment. MUTATION: put
  either helper back between a doc comment and the function it documents -> the captured
  symbol reports no doc comment and the capturing one reports the wrong text. (Deleting the
  blank line does NOT reproduce it: `go/ast` binds a comment group to the declaration after
  it, so the defect is the ORDER.)
- Whole gates: four packages; the 24 gui shards; gofmt; vet; firmware size with the delta
  stated for the STAGE against a named baseline and the hook's share asserted to be no
  measurable cost -- measured subtractively, with a positive control proving the measurement
  can move. NOT per change: the same unchanged hook measured 0 B on one tree and -16 B on
  a tree differing only in four string literals, so per-change attribution at this granularity
  reports layout noise as cost (plan-round fidelity M-2).
- Records: the fork comment at `gui/composer_copy.go:441` claims headroom 186; measured 107
  (tests M-1, journey N-1) -- corrected in the same commit as §1.

## §7. Acceptance

All three walk runs (§4.5) recorded by the controller with the failing assertion named;
the H2 spec folds of §1.4 and §2.5 and the toolkit manual re-quote landed; post-implementation
review; merge to fork main `--no-ff`; a signed image built (`sh2-flash -b`); the device walk
(H2 §8) stays ASSUMED at the operator's word until they run it.

## §8. Citations at `b9a9a30`

`gui/composer_hashlock.go:62-84` (HOLD, assignment, reconcile), `:169-172` (lead);
`gui/composer_copy.go:409-423` (confirm body), `:441-446` (reconcile + its stale comment),
`:458-473` (§8h forms); `gui/composer_state.go:35-38`; `gui/composer_hash.go:177-199`;
`gui/composer_shape.go:356` (Remove arm sync); `gui/composer_paged.go:62-90` (band);
`gui/unlock_kdf.go:390-393` (the shared body), `:395-425` (noun arms);
`gui/modal_fits_test.go:52` (`modalBodyMargin = 80`), `:33-35` (headroom is a line budget),
the reconcile body's row in the error-screen-body table at `:342` and the confirm-modal row at `:388`; `gui/composer_flow.go:33-34` (state construction);
`gui/frame_hook.go` / `gui/frame_hook_tinygo.go` (the seam model); `gui/screen.go:95-98`
(`frameTargets` drops the tag); `seal/record.go:69` (0-based index); `cmd/emu/walk_hashlock_phrase.js:74-76, 232,
286-329`.

## R0 round 0 folded here

*(A record of the SPEC's own round 0. Its numbers are as of that round; where the
plan round later re-measured a body, `## Plan-round fold` at the end of this file
carries the current figure and says which one it replaced.)*

Three lenses at `f6dd437`: fidelity (opus, 1C/5I/4M/4N), tests + citations (sonnet,
0C/3I/1M/2N), journey walk (opus, 0C/7I/5M/2N). Folded: fidelity **C-1** (nil map: the
`composerNotePhraseDigest` helper allocates; zero-value-state test), **I-1** (the ruled
write-down remedy restored -- it fits at headroom 107; the spec's own "Without all three"
repair was what cost 43 and is dropped), **I-2** = tests I-1 = journey I-5 (label pick
withdrawn: `shTargets` has no labels; index + landing assertion kept), **I-3** = tests I-2 =
journey I-6 (the seam is a `!tinygo` pair in `gui` on the frame_hook / passphraseWidgetHook
model; hook delta asserted 0 bytes), **I-4** (H2 §4.5/§4.7 folds are part of the leg), **I-5**
(pre-hold read pinned to the confirm-modal frame); tests **I-3** = journey M-1 (the needle
"run ms hashlock with this phrase" kept verbatim); journey **I-1** (mismatch sentence, headroom
320), **I-2** (`chars: <n>` on the reconcile screen), **I-3** (§8h "every ... and every ...",
mixed-wallet test + mutation), **I-4** ("(records count from 0)", headroom 397), **I-7** (third
walk run: perturbed stored hash must fail stored-equals-displayed, failing assertion
recorded). Minors folded: fidelity M-1 (five sites + two tests listed), M-2 (doctrine
sentence), M-3 (shared body, named table), M-4 (recoverability stated); journey M-2 (manual
re-quote in-cycle), M-3 (both mutating tests named), M-5 (documentation note); tests M-1 =
journey N-1 (the 186 comment corrected in the leg). Citations fixed: modalBodyMargin `:52`;
rows `:342`/`:388` (`:372` was not a row); `composer_copy.go:458-473`; unlock_kdf ranges.
Declined: journey M-4 (saying on the reconcile screen that the consent screen repeats the
token -- the token is on the screen itself now); journey N-2 (wording). Verified true and
kept: §3 exactly as written (152 px under Back, 0 with the band; the lead is 2 lines at 464
and at 411 px so §3.3's fallback never fires; readout budget unchanged at 209).

## R0 round 1 folded here

The r1 sonnet verification (`hashlock-H5-spec-R0-r1-fold-verification.md`) found the round-0
fold had composed two numbers from measurements of DIFFERENT bodies instead of re-measuring
the text it wrote: §1.1's body is 186 drawn / 339 headroom (not 205/320, which was the journey
lens's body before `chars:` and the new sentence); §2.5's sentence is 165 drawn (was 160),
headroom unchanged at 378 (not "shorter"). §1.4 no longer sends the reconcile clause to H2
§4.7 (only §2.5's phrase form changes §4.7). §2.3/§6 list the sixth `hashByPhrase` site,
`composer_hashlock_test.go:916`, whose assertion becomes the predicate (the test does not
compile otherwise). Lesson recorded: a fold that composes two remedies re-measures the
composed text; a number quoted from a report is only as good as the body it measured.

## Plan-round fold

The plan's R0 round 0 (three lenses on
`design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md` at engrave `0c2b13e`; reports
`hashlock-H5-plan-R0-r0-{fidelity,tests,journey}.md`, 0C total) reached back into this spec.
The spec's own R0 GREEN stands -- nothing below is a Critical or an Important against the
spec -- but a lens that finds a plan defect whose ROOT is a spec sentence has found a spec
defect, and the fold is recorded here rather than left in the plan.

**Every number below is a measurement of the text as it now reads**, taken by the fold on the
gated tree `/scratch/code/shibboleth/.tmp/h5-gate`, not carried over from a report: a number
quoted from a report is only as good as the body it measured (the r1 lesson, applied again).

| § | change | measured |
| --- | --- | --- |
| §1.1 | *"Before you fund this wallet, …"* -> *"Before you cut plates, …"*; the mismatch sentence keeps funding as the deadline. The plates are cut inside the same `composerFlow`, ~21 minutes each, and the digest is in the engraved md1 (journey M-2). | 181 drawn / headroom 339 (was 186/339) |
| §1.2 | the sentence after the ruled write-down line: *"They are not on this device and not on your plates."* -> *"The phrase and method are not on this device."*. The first sentence is byte-identical. "and this digest" had made the claim FALSE of the digest -- it IS on the plates -- and left "Without both" a three-item antecedent (journey I-1). | 343 drawn / headroom 107 (was 347/107); both rejected variants 361/64, and both FAIL the fit gate |
| §2.5 | the phrase form's overcount on the two pure wallets is RECORDED as a decision, not changed (journey M-4). | 165/378, unchanged |
| §2.6 (new) | §8h's PLAIN form ends *"Back up every preimage separately."* -- the same undercount on the sibling body (journey I-2). | 133 drawn / headroom 397 (was 131/397) |
| §4.1 | the hook's share is asserted as NO MEASURABLE COST, not "0 bytes" (fidelity M-2). | shipped 1,599,208 B; hook removed 1,599,224 B; second defer 1,599,304 B; `println` control 1,599,368 B; baseline `b9a9a30` 1,597,404 B |
| §4.2 | the stored-versus-displayed comparison reads the token OUT OF THE MODAL'S FRAME and runs BEFORE the corpus check; against a file constant it was a tautology and §4.5(c) tripped the corpus assertion instead (fidelity I-1 = journey I-7 unmet). | four scenarios replayed against the walk's own text: (a) passes, (b) pre-hold, (c) stored-vs-displayed, (d) corpus only |
| §4.4 | the `ok` guard reads EVERY assignment, and logs only what it checked (fidelity I-2, journey N-1). | the counterexample walk now FAILS the guard |
| §4.5 | the tree is restored between the three runs and the restoration is CHECKED (journey M-1). | — |
| §5 | *"Remove that record -- and any others like it -- (records count from 0) …"*; `AdmitSection` refuses one record at a time and the index moves between rounds (journey M-3). Plus an ERRATUM: no copy-table row for this body can exist, and the fit row is the gate (fidelity M-4). | 175 drawn / headroom 378 (was 153/397) |
| §6 | the firmware delta is stated for the STAGE, not per change, with the measurement that justifies it (fidelity M-2); the new tests and mutations are listed. | — |

Lesson recorded, alongside the r1 one: **a "0" that a whole-image build produces is not a
structural zero until something has moved around it.** §4.1's 0 B survived a spec round, two
fold verifications and a plan build gate, and fell to four string literals.
