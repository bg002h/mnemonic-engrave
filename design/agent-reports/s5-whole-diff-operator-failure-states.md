# S5 whole-diff review — LENS: THE OPERATOR'S FAILURE STATES

> ## ⚠ REPAIR THE WORKTREE BEFORE ANYTHING ELSE — I damaged it
>
> `/scratch/code/shibboleth/wt-s5` is missing four tracked files from its working tree:
>
> ```
> $ git -C /scratch/code/shibboleth/wt-s5 status --porcelain
>  D oracle/gaterecords/S5-trace-b.expect.json
>  D oracle/gaterecords/S5-trace-b.inputs.json
>  D oracle/gaterecords/S5-trace-b.record.json
>  D oracle/gaterecords/S5-trace-b.walk.json
> ```
>
> **Repair:** `git -C /scratch/code/shibboleth/wt-s5 checkout -- oracle/gaterecords/`
>
> HEAD is still `7da66bd` on `s5-multislot`, the index is clean, nothing is staged, and no
> other file differs from HEAD — so the checkout restores the frozen state exactly.
>
> I caused this and I am reporting it rather than repairing it, because the brief forbids me
> any writing git operation on that tree. Probable mechanism, stated as a hypothesis: I made a
> `cp -a` copy of the worktree into the scratchpad to run a probe test. `cp -a` copied the
> worktree's `.git` **file**, which contains `gitdir: /scratch/code/shibboleth/.git/worktrees/wt-s5`,
> and that gitdir's `core.worktree` points back at the **original** path. So `nix develop`
> resolving the flake source inside the copy ran git against the original worktree, not the
> copy. The four files it removed are the only ones in the diff that are untracked by the Go
> build and therefore invisible to `go test` — nothing warned. Lesson for the next agent who
> needs a scratch copy of a worktree: delete or rewrite the copy's `.git` file first, or use
> `git archive | tar -x` instead of `cp -a`.


Artifact: `/scratch/code/shibboleth/wt-s5`, `git diff main..s5-multislot`, frozen at `7da66bd`.
Reviewer scope: **can a correct operator, or an interrupted one, be led to destroy a good
backup or trust a bad one?** Read-only on the frozen tree throughout; every runtime check
below was executed on a `cp -a` copy in the scratchpad, never on the worktree.

## What the lens cleared

These were traced through the actual screen strings and flow code and are **sound**:

* **Announce-before-first-cut is genuinely before the first cut, in code order, on both
  paths.** Build: `confirmReviewScreen("Plate Count", …)` at `gui/multisig_build.go:361`
  precedes `bundleEngrave` at `:364`. Supply: the collapse `NOTE:` is *prepended* to the
  census (`gui/multisig.go:259-267`) and `confirmReviewScreen("Plates To Cut", …)` at
  `:268` precedes `bundleEngrave` at `:272`. The multi-slot notice (`showNotice`,
  `gui/multisig.go:186`/`:193`) is drawn earlier still, before the engrave-mode pick. The
  collapse note leads the list rather than trailing it, which matters because
  `confirmReviewScreen` confirms from any page.
* **The EXPERIMENTAL warning is unskippable and precedes everything irreversible.**
  `multisigBuildExperimentalWarning` at `gui/multisig_build.go:326`, before the mode pick
  (`:340`), the tail (`:352`), the census (`:361`) and the engrave (`:364`). `ConfirmNo`/Back
  returns false and the caller returns. Its body no longer teaches the fingerprint ritual
  and it names a check that can actually fail.
* **The DESTROY arm is correct and correctly gated.** `bundleAbortWarningText`
  (`gui/bundle_flow.go:488`) says DESTROY *and names a method* only when
  `bundleSetCarriesASecret(cards)`; a public-only set says "No plate in this set carries a
  seed." Since `multisigEngraveCardsMulti` emits every ms1 first, the plate on the bench at
  almost any full-mode abort really is the seed plate. This is a real fix to text that was
  wrong on shipped code.
* **Both prose test files are real gates, not `len(s) > 0` theatre.** I read every assertion
  in `gui/bundle_abort_prose_test.go` and `gui/multisig_build_prose_test.go`. They assert
  named substrings *and* their negations (the retired wording must be gone), assert the two
  arms are *different strings* (so a one-string helper cannot pass), assert the drawn frame
  via `assertModalBodyFits`, and `TestModalFitCheckCatchesATruncatedBody` mutation-proves the
  fit primitive in both directions. Grepping the four newest prose/verify test files for
  weak-assertion shapes (`len(...) > 0`, `!= ""`) returned exactly one hit, and it is a
  `!= 0` on a *negative* expectation. No false gate found here.
* **The verify's incomplete state is reported as incomplete, not as a pass or a failure**
  (`gui/multisig_verify.go:679-686`), and the three `break`-not-`return` decisions are
  correct — a partial verify now reaches a verdict screen instead of walking out silently.

Everything below is what the lens did **not** clear.

---

## C1 (Critical) — On the SUPPLY path a passphrase build is labelled "Full (seed + keys)" and its restore document never mentions the passphrase

**Files:** `gui/multisig.go:204` (the label), `gui/multisig.go:302` (the restore doc),
`gui/multisig.go:141` (where the passphrase is taken).

S5 establishes, at length and correctly, that a BIP-39 passphrase is a required spending
factor that is never engraved, and that a set labelled "Full (seed + keys)" which omits it is
"the worst thing a backup can be: wrong AND trusted" (`gui/multisig_build_census.go:100-123`,
`gui/multisig_build_prose_test.go:337-352`). It then fixes exactly one of the two flows that
take a passphrase and cut a Full set.

The supply flow takes a passphrase:

```go
// gui/multisig.go:139-148
passphrase := ""
ppChoice := &ChoiceScreen{Title: "Passphrase", Lead: "Add a BIP-39 passphrase?", Choices: []string{"Skip", "Add passphrase"}}
if sel, ok := ppChoice.Choose(ctx, th); ok && sel == 1 {
	if pass, ok := syswPassphraseFlow(ctx, th); ok {
		passphrase = pass
```

and then hard-codes the label the build path was fixed for:

```go
// gui/multisig.go:201-205
modeChoice := &ChoiceScreen{
	Title:   "Engrave Mode",
	Lead:    "What to engrave?",
	Choices: []string{"Full (seed + keys)", "Watch-only (keys)"},
}
```

`buildFullModeLabel` is never called here — measured:

```
$ grep -rn "buildFullModeLabel" gui/
gui/multisig_build.go:340   ← the only production call site
gui/multisig_build_census.go:124,140
gui/multisig_build_prose_test.go:315,316
```

and the restore document is handed `nil` where the build path hands the inventory:

```
gui/multisig_build.go:415: multisigRestoreDocFlow(ctx, th, tpl, keys, buildPlateInventoryLines(cardsOut, usedPassphrase))
gui/multisig.go:302:       multisigRestoreDocFlow(ctx, th, tpl, keys, nil)
```

`buildPassphraseInventoryLines` reaches the operator *only* through that `extra` argument.
The prose test's own premise — "measured, the restore file contained zero occurrences of the
word" — is **still true after S5**:

```
$ grep -c -i passphrase gui/multisig_restore.go
0
```

**Failure scenario.** Operator picks Multisig → "Supply policy (md1)", scans a 2-of-3 policy,
enters their seed, picks "Add passphrase" and types one, picks **"Full (seed + keys)"**, and
cuts ms1 + mk1 + md1. The label told them the set is seed + keys. The restore document — the
artifact `buildPlateInventoryLines` exists because it "is read years later, alone, often by
someone who was not the operator" — says nothing about a passphrase in either direction. Five
years later the reader holds a complete-looking Full set that cannot reach the money, with no
statement anywhere on the device or on the steel that a third factor was ever in play. This is
F-132's shape, on the flow S5 *did* modify (F-188 changed its engrave rule and made it cut
more plates than before), and it is the exact defect the diff declares must not ship.

Yes, this is pre-existing rather than S5-introduced. It is still Critical: the merge is
immediately followed by engraving real wallet keys, and "Supply policy (md1)" is the first
row of the front-door `ChoiceScreen` (`gui/multisig.go:78`). The fix is two lines:
`buildFullModeLabel(passphrase != "")` at `:204` and `buildPlateInventoryLines(cardsOut,
passphrase != "")` at `:302`.

**Verified by:** reading all three sites; `grep -c -i passphrase gui/multisig_restore.go` → `0`;
`grep -rn buildFullModeLabel gui/` (single production call site); and running
`buildFullModeLabel(true)` → `"Full (seed + keys, NOT passphrase)"`, proving the correct string
exists and is simply not reachable from this flow.

---

## I1 (Important) — The abort screen's recovery instruction has no mechanism behind it: there is no way to skip a plate that is already cut

**Files:** `gui/bundle_flow.go:489-492` (the promise), `gui/bundle_flow.go:383-409` (the loop
that cannot keep it), `gui/gui.go:2955-3030` (`EngraveScreen.Engrave`).

The new abort text, machine-printed:

```
Stopped at card 1 of 6 (ms1 secret share 1 of 2). This set is not a usable backup yet.
To finish it, run this again and give the same answers: it cuts the same plates, byte for
byte, so you only cut the ones you are missing.
If you throw any of it away instead, a plate with your seed on it must be DESTROYED, not
binned: cut it up or grind the words off.
```

"you only cut the ones you are missing" is an instruction the device cannot obey.
`bundleEngrave` walks the plan strictly in order and offers exactly three ways out of any
one plate:

```go
// gui/bundle_flow.go:396-409
engraved := false
for !engraved {
	idx, ok := cs.Choose(ctx, th)
	if !ok {
		bundleAbortWarning(ctx, th, cards, p)   // aborts the WHOLE set
		return
	}
	if NewEngraveScreen(ctx, plates[idx]).Engrave(ctx, &engraveTheme) {
		engraved = true
	}
	// Back out of the engrave screen → re-show THIS plate's picker
}
```

The `Choices` are `labels` from `validateMdmk`, and those are exactly
`{"TEXT + QR", "TEXT ONLY", "QR ONLY"}` (`gui/gui.go:2299-2303`) — no skip row.
`EngraveScreen.Engrave` returns `true` only after the job reaches `engraveDone` **and** the
operator presses select (`gui/gui.go:2977-2986`); every other exit returns `false`, which
re-shows the same picker. Nothing records completed plates across runs: `notifyPlateEngraved`
is "a no-op in the firmware, which does not contain the interface" (`gui/gui.go:2979-2982`),
which is precisely why the abort text says the state is lost.

**Failure scenario.** Trace B, full mode, 9 plates. Power fails after plate 6, or the operator
simply runs out of blank steel at plate 7 (the census itself tells them to have that many
blanks ready, so this is the expected exhaustion point). They re-run, give the same answers,
reach `Card 1 of 6 | Plate 1 of 1` — the ms1 seed plate already sitting on the bench — and
their options are:

1. cut it again on a fresh blank, producing a **second seed plate**, which is the exact
   outcome `buildEngraveTail` refuses to produce on purpose ("a duplicate secret on steel with
   no recovery benefit", `gui/multisig_build_tail.go:70-72`), and which contradicts the plate
   census and the restore-doc inventory the same run will print;
2. re-run the identical job onto the already-engraved plate — plausible for a hammer, but
   named nowhere on the device or in the diff; or
3. press Back, which aborts the whole set and re-shows **the same message telling them to run
   it again**. That is a closed loop.

The sibling `multiPlateEngrave` still says "discard the partial plate(s) and start over"
(`gui/derive_xpub.go:523-527`), i.e. the honest description of what the machine actually
supports. S5 replaced that with a promise on the one screen an interrupted operator reaches,
and did not build the mechanism. Either add a "already cut — skip" row to the per-plate
`ChoiceScreen`, or say what the operator must actually do.

**Verified by:** reading `bundleEngrave`, `ChoiceScreen.Choose` (`gui/gui.go:1669-1680`),
`EngraveScreen.Engrave`, and `validateMdmk`'s label set; and by running
`bundleAbortWarningText(bundlePlate{cardIdx:1, cardTotal:6, label:"ms1 secret share 1 of 2"}, true)`
to print the promise verbatim.

---

## I2 (Important) — An abort does not propagate, so the verify offer and the restore-doc inventory run exactly as if the engrave had completed

**Files:** `gui/bundle_flow.go:376` (`func bundleEngrave(...)` — no return value),
`gui/bundle_flow.go:388` and `:401` (bare `return` after the abort modal),
`gui/multisig_build.go:364-415`, `gui/multisig.go:272-302`.

The new abort text's design rationale states as fact:

```
// gui/bundle_flow.go:481-483
// AND THIS IS WHERE IT HAS TO BE SAID. The restore document carries the set
// inventory, and it is printed at the end of a SUCCESSFUL run -- an operator
// whose engrave died never reaches it. This modal is the only screen they get.
```

That premise is false for every abort. `bundleEngrave` returns `void`; neither caller can
tell a completed run from an aborted one, and both walk straight on:

```
gui/multisig_build.go:364  bundleEngrave(ctx, th, "Build Policy", cardsOut)
gui/multisig_build.go:397  verifyChoice := …"Verify the engraved plates?"…
gui/multisig_build.go:415  multisigRestoreDocFlow(…, buildPlateInventoryLines(cardsOut, usedPassphrase))

gui/multisig.go:272        bundleEngrave(ctx, th, "Engrave Multisig", cardsOut)
gui/multisig.go:295        verifyChoice := …"Verify the engraved plates?"…
gui/multisig.go:302        multisigRestoreDocFlow(ctx, th, tpl, keys, nil)
```

**Failure scenario.** Trace B, full mode, 9 plates; the operator aborts at card 4 of 6 because
they have run out of blanks. They read "Bundle Incomplete / … This set is not a usable backup
yet." Two screens later the device asks **"Verify the engraved plates?"** — a verify that
cannot succeed, because the md1 card is emitted last and was never cut, so it dies at
`extractReadbackMd1AndMk1s` with "Read back one wallet-policy md1 AND the operator key
card(s) (mk1)." That reads as *your plates are unreadable*, not as *you never cut the md1*.
Then the restore document prints, carrying (machine-printed) the inventory:

```
["This backup is 1 plate:" "ms1: 1 plate (s)" "If any of them is missing, this backup is
 incomplete." "A BIP-39 passphrase WAS used. …"]
```

— i.e. for the real case, "This backup is 9 plates: ms1 secret share 1 of 2: 1 plate … If any
of them is missing, this backup is incomplete." That is the artifact the diff itself describes
as the thing "read years later, alone", printed as the last word of a run the device has just
told the operator produced no usable backup, with nothing on it distinguishing the two.

The whole reason the abort text was rewritten was that the operator's information ends at that
modal. It does not. Either thread an `ok bool` out of `bundleEngrave` and skip steps (10)/(11)
and (8)/(9) on an abort, or stop claiming the restore doc is unreachable and make the inventory
say which run it belongs to.

**Verified by:** reading the signature and both `return` sites in `bundleEngrave`; reading both
call sites and confirming no guard between the call and the subsequent screens; and running
`buildPlateInventoryLines(cards, true)` to print the inventory text verbatim.

---

## I3 (Important) — A watch-only verify tells the operator "secret verified" when no seed plate exists and no ms1 was ever typed

**File:** `gui/multisig_verify.go:714-717`.

```go
func multisigVerifyOKMessage(legs int, full bool) string {
	if legs <= 1 {
		return multisigVerifyOKBody   // "Operator key and secret verified. …"
	}
	if full { … }
	return fmt.Sprintf("All %d operator key plates verified. …", legs)
}
```

The multi-leg arms are `full`-aware, and the function's own doc comment states the rule:
"`full` is the mode, so a watch-only run does not claim a secret it never asked for"
(`gui/multisig_verify.go:709-710`). The single-leg arm ignores `full` entirely. Measured:

```
verifyOK(1 leg, watch-only) = "Operator key and secret verified. Other cosigners' keys are taken as supplied."
verifyOK(1 leg, full)       = "Operator key and secret verified. Other cosigners' keys are taken as supplied."
verifyOK(3 legs, watch-only)= "All 3 operator key plates verified. Other cosigners' keys are taken as supplied."
```

**Failure scenario.** Operator does the ordinary single-slot case — Build policy or Supply
policy, one held/matched slot — and picks **"Watch-only (keys)"**. No ms1 card is created
(`multisigEngraveCardsMulti` gets an empty `ms1s`), `multisigVerifyMS1Entry` is never called
(`gui/multisig_verify.go:589`, guarded by `if full`), and `bundle.Verify`'s ms1 leg is skipped
on both sides by presence semantics. They then verify, and the final screen says **"Operator
key and secret verified."** The device asserts it checked a secret that was never engraved,
never typed and never compared. `len(legs) == 1` is the *common* case, so this is the arm most
operators see. One line: `if legs <= 1 && full { return multisigVerifyOKBody }` plus a
watch-only single-leg string.

`TestMultisigVerifyNoticeIsHonest` (`gui/multisig_verify_test.go:171`) does not catch it — it
drives `showNotice` with the constant directly and never calls `multisigVerifyOKMessage`, so
the `full` parameter is untested at `legs <= 1`.

**Verified by:** running `multisigVerifyOKMessage(1, false)` and `(1, true)` on a copy of the
frozen tree under `nix develop` (output above); reading the `if full` guard at
`gui/multisig_verify.go:589` that proves no ms1 is requested in watch-only.

---

## I4 (Important) — The verify's "already checked / different seed" arm asserts a foreign seed where a same-seed passphrase divergence is equally likely (new site of the F-191 class)

**Files:** `gui/multisig_verify.go:566-573` (the three-arm switch),
`gui/multisig_build.go:194-201` + `gui/multisig_build.go:539-570` (the per-held-slot
passphrase prompt that creates the shape).

F-191's fix — `multisigVerifySeedIsInnocent` — is wired into **one** arm only, the
`len(slots) == 0` arm. The other two arms still state a conclusion the device cannot support:

```go
case len(everOwed) == 0:
	showError(…, "That seed is a cosigner, but none of its slots were engraved in this run. "+
		"The plates still outstanding belong to a different seed.")
default:
	showError(…, "That seed's slots have already been checked. The plates still "+
		"outstanding belong to a different seed.")
```

S5 makes the passphrase prompt **per held slot** (`buildSeedForSlot`, called once per
`p.SelfSlots` entry at `gui/multisig_build.go:194-201`; `Title: "Passphrase " + label`). There
is no confirm-entry, and nothing compares two registry entries that hold the *same words*.
`buildSlotSources` keys the account counter on `seed.MasterFP`, which is the fingerprint of the
`(seed, passphrase)` **pair** (`gui/multisig_build_slots.go:131-143`), so two entries of the
same words with different passphrases are two different masters as far as the flow is
concerned and both are assigned account 0.

**Failure scenario.** Operator holds @0 and @1 from one seed. They type the words for @0 and
the passphrase `hunter2`; they type the same words for @1 and mistype `huntre2`. Both slots get
account 0, both derive at `m/48'/0'/0'/2'`, the two keys differ so §4.1's duplicate refusal does
not fire, and the review screen prints only "@0 yours: derived from your seed for @0" / "@1
yours: derived from your seed for @1" — the account suffix is suppressed at account 0
(`gui/multisig_build_slots.go:601-608`), so nothing on the screen distinguishes them. Two mk1
plates are cut; the ms1 dedupe (correctly) cuts **one** seed plate, which encodes the words and
not either passphrase. At verify: seed + `hunter2` covers @0; the operator is asked for the next
seed; they type the same words and passphrase again and are told **"That seed's slots have
already been checked. The plates still outstanding belong to a different seed."** followed by
"Verify Incomplete: Checked 1 of the 2 key plates". The device has sent an operator holding the
only seed that exists to look for a second one, while the real cause — a passphrase divergence
on the same words — is the one explanation it never names, despite already owning the routine
that could distinguish it.

The two-line fix is the one F-191 already established: call
`multisigVerifySeedIsInnocent`-style logic (or simply compare this seed's words against the
already-covered seeds) before asserting "a different seed".

**Verified by:** reading the switch at `gui/multisig_verify.go:557-576` and confirming
`multisigVerifySeedIsInnocent` is called only in the `len(slots) == 0` arm; reading
`buildSeedForSlot` and confirming no confirm-entry and no cross-seed comparison; reading
`passphraseFlowTitled` (`gui/gui.go:694-720`) and confirming it returns `kbd.Fragment` on a
single OK press with no re-entry; reading `buildSlotSources`' `accounts[seed.MasterFP]` keying.

---

## Minor / Nit (recorded, does not gate)

* **M1 — the verify readback gather is titled "Engrave Bundle".**
  `gui/multisig_verify.go:455`: `bundleGatherFlow(ctx, th, "Engrave Bundle")` inside
  `multisigVerifyFlow`. The operator is presenting already-cut plates for checking and the
  screen — plus both of its "Done" refusals — names the engrave program. This is a new site of
  exactly the class `TestMs1ReminderIsTitledForTheProgramThatShowedIt` and S2's D-4 work
  address; that test's table covers `bundle_flow.go`, `singlesig.go`, `multisig.go` and
  `multisig_build.go` and does not look at `multisig_verify.go`. (Verbatim in `main` too, so
  not introduced here.) Suggested: `"Verify Bundle"`.
* **M2 — the abort at plate 1 issues DESTROY instructions for a plate that does not exist.**
  `gui/bundle_flow.go:493-495`. Backing out of the very first plate's variant picker cuts
  nothing, yet the modal reads "Stopped at card 1 of 6 … a plate with your seed on it must be
  DESTROYED". A `p.cardIdx == 1 && p.plateIdx == 1` guard would keep the DESTROY warning for
  the runs where it matters, which is §0.1's own corollary about warnings that get ignored.
* **M3 — the supply path's restore document carries no plate inventory at all.**
  `gui/multisig.go:302` passes `nil`. F-188 is what makes this bite: the supply path now cuts
  a plate per matched slot, so its set can be several plates, and its restore doc says nothing
  about how many — the same F-131/F-132 silence S4 fixed on the build path only.
* **M4 — a TEMPLATE-form build shows no restore document, so its passphrase statement exists
  only on the mode label.** `gui/multisig_build.go:412` (`if !template`). The engrave-mode row
  does say "NOT passphrase", so the operator is told at decision time; the artifact that
  outlives them is not written at all.
* **M5 — `assertModalBodyFits` coverage.** Already filed as F-192; noting only that I checked
  the two longest S5 strings outside its scope (the supply path's two multi-slot notices,
  `gui/multisig.go:186`/`:193`, ~270 and ~172 source characters) against the file's own
  measured capacity of 588 normalised characters for both modal shapes, and they are not close
  to the fold. F-192 is a real gap in the *gate*; I found no concrete truncation behind it.

## Not re-reported

F-189, F-190, F-191 (base case), F-192, F-193, F-194, F-195 — all encountered, all already
filed. I4 above is a new instance of the F-191 *class* at a different site (the `default` and
`len(everOwed) == 0` arms rather than the `len(slots) == 0` arm) and is reported as such.

## Settled negatives not re-derived

Per the brief: `go test ./... -count=1` (cold cache) green, `gofmt -l` empty, `go vet` at its
established 40-test-only-finding baseline, `oracle-live.sh` PASS, `emu.wasm` builds. None of
those was re-run and none is claimed as a finding.
