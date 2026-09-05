# SPEC — Hashlock H5: device polish (F-487, F-480, F-484, F-485, F-488)

**STATUS: DRAFT 2026-09-05 -- R0 round 0 pending.** Base: seedhammer fork main `b9a9a30`
(H2 merged at c284484; H3 seam corpus at b9a9a30). Parent spec: `SPEC_hashlock_H2_device.md`
(GREEN 55ee7a4, H3-folded at 657f40f/8d5139d). Every citation below was measured at
`b9a9a30`; re-grep at plan time.

## §0. Why this stage exists

H2 shipped the phrase route; its post-implementation reviews and the ultracode lenses left
five defects that change what the operator sees or what the walk can prove, each ruled by
the operator on 2026-09-05 (FOLLOWUPS F-487, F-480, F-484, F-485, F-488). This stage folds
them into the H2 spec's sections and ships them as ONE fork leg. Out of scope: F-483
(secret handling, logged), F-489 (host, next `me` change), the idle-timer note.

## §1. F-487 -- the reconcile screen repeats the digest; the write-down asks for it

**Today** (`gui/composer_hashlock.go:62-84`): after HOLD assigns the hash, `showError(ctx,
th, "Hash lock", composerCopyHashlockReconcile())` draws *"Before you fund this wallet, run
ms hashlock with this phrase and method on the host and check the digest matches."*
(`gui/composer_copy.go:443-446`). The digest the operator is told to compare has just left
the screen with the confirm modal, and the write-down line (`composer_copy.go:419-420`)
asks for the phrase and the method only. Nowhere later on the device shows the digest.

**Normative.**

1. `composerCopyHashlockReconcile` takes `(first8last8, method string)` and returns, in
   this order and with these exact strings:
   ```
   hash  <first8>..<last8>
   method: <m>
   Write this digest beside the phrase and the method. Before you fund this
   wallet, run ms hashlock with them on the host and check the digest matches.
   ```
   (`<m>` is `hashlockMethod.String()`: `sha256` or `hardened`, as the confirm modal
   prints it.) It is drawn where it is drawn today, on its own `showError` screen right
   after HOLD, for every phrase-set hash (H2 §4.5 as H3-folded).
2. The confirm modal's body (`composerCopyHashlockConfirm`) is UNCHANGED. Reason, measured:
   the modal-fit gate requires 80 normalised characters of headroom (`modalBodyMargin`,
   `gui/modal_fits_test.go:51`) and the longest legal confirm body has about 107 after the
   R0 round-0 drop order (fold report `hashlock-H2-plan-R0-r0-fold-report.md`); "and this
   digest" plus a repaired "Without both" clause costs more than the 27 to spare. The
   operator's second remedy (the digest on the paper) is therefore met on the reconcile
   screen, which has the room, at the moment the digest is shown again.
3. The reconcile body gets its own row in the modal-fit table for the longest legal content
   (`method: hardened`), measured by `assertModalBodyFits` on the error-screen renderer
   (`showError` -> `ErrorScreen`), and a row in the copy table (`TestComposerCopyTableCoversEveryBody`).
4. The emulator walk asserts the reconcile screen carries the SAME `first8..last8` token
   the confirm modal carried (§4 below).

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
   the phrase route in this composition. HOLD inserts `h`. Nothing ever deletes from it
   (a value set cannot go stale: a digest no path carries is simply never matched).
2. `composerAnyPathByPhrase(st) bool` is true iff some `p` in `st.list.Paths` has
   `p.Hash != nil` and `*p.Hash` is in `phraseDigests`. `composerCopyHashEveryPathFor`
   uses it. §8h's phrase form therefore names the phrase exactly when a CURRENT path's hash
   came from a phrase -- H2 §4.7's condition, now per digest.
3. `composerHashByPhraseSync` and both its call sites are deleted. Remove path, the
   `No hash lock` row, `Type 64 hex` and payload rows need no bookkeeping: they change
   `p.Hash`, and the predicate reads `p.Hash`.
4. Index identity is not used anywhere in this design (C16's lesson: "Remove path" splices
   the slice). Two paths sharing one phrase digest are both by-phrase; a path whose phrase
   digest is later replaced by the SAME digest typed as 64 hex is still by-phrase (the
   digest was derived here once; the backup burden is unchanged) -- state this in the code.

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

1. The emulator exposes `window.shComposerPathHashes()` returning the current composition's
   per-path hash as 64-hex or `null` (`cmd/emu`, js build only; no production code path).
2. The walk, in the hardened trial: reads the hashes BEFORE the hold and asserts the edited
   path's is `null`; after the hold asserts it equals the corpus's full 64-hex
   `hardened` digest AND that its `first8..last8` equals the token the confirm modal
   displayed AND the token the reconcile screen displays (§1.4).
3. The phrase row is chosen by LABEL (`Type a hashlock phrase`) from `shTargets`, never by
   index (H2 §5's own rule for production).
4. `out.ok` is set to `true` only after the last assertion; no recomputation.
5. The controller runs the walk against the branch build (fresh port, playwright) AND
   against a build with the HOLD assignment moved before the confirm (mutation): the walk
   must FAIL on §4.2's pre-hold assertion. Both runs are recorded.

## §5. F-488 -- the unlock refusal says what to do next

**Today** (`gui/unlock_kdf.go:391`): *"Record N is a hashlock preimage, not a seed. This
payload cannot be unlocked here. Nothing was opened."*

**Normative.** The sentence gains one more: *"Remove that record on the host and seal the
payload again."* Fit re-measured (`assertModalBodyFits`, longest noun and a two-digit
index); the copy-table row updated; `gui/unlock_preimage_test.go`'s frame assertion gains
the new sentence.

## §6. Tests (each with the mutation that must fail it)

- §1: copy-table row; fit row (hardened); flow test through HOLD reaches a frame carrying
  `3cf5d421..b70a4c12`, `method: hardened` and "Write this digest" -- MUTATION: return the
  old one-sentence body -> fails on the token.
- §2: `TestRemovePathReSyncsHashByPhrase` becomes the value-set test (remove the phrase
  path, add a hex-hashed one, Done draws the plain form -- MUTATION: predicate returns true
  when the set is non-empty -> fails); the interruption lens's edit-to-payload scenario;
  two paths one digest; same digest re-typed as hex.
- §3: the geometry test of §3.2 -- MUTATION: restore the panel-wide wrap -> (a) fails.
- §4: the two walk runs of §4.5.
- §5: frame assertion -- MUTATION: drop the new sentence -> fails.
- Whole gates: four packages; the 24 gui shards; gofmt; vet; firmware size (delta stated).

## §7. Acceptance

Both walk runs (§4.5) recorded by the controller; post-implementation review; merge to
fork main `--no-ff`; a signed image built (`sh2-flash -b`); the device walk (H2 §8) stays
ASSUMED at the operator's word until they run it.

## §8. Citations at `b9a9a30`

`gui/composer_hashlock.go:62-84` (HOLD, assignment, reconcile), `:169-172` (lead);
`gui/composer_copy.go:409-423` (confirm body), `:443-446` (reconcile), `:458-471` (§8h
forms); `gui/composer_state.go:35-38`; `gui/composer_hash.go:177-199`;
`gui/composer_shape.go:356` (Remove arm sync); `gui/composer_paged.go:62-90` (band);
`gui/unlock_kdf.go:395-415` (noun), `:391` (the arm's sentence); `gui/modal_fits_test.go:51`
(`modalBodyMargin = 80`), rows at `:342,372,388`; `cmd/emu/walk_hashlock_phrase.js:74-76, 232,
286-329`.
