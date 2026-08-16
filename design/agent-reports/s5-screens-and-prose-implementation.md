# S5 — screens and prose: implementation report

**Worktree** `/scratch/code/shibboleth/wt-s5`, branch `s5-multislot`.
**Base** `e7da39a` → **commit** `023505c` "S5: the screens an operator reads before putting a seed on steel".
Working tree clean; nothing outside `gui/` touched. `oracle/**`, `verifyMultisigLegs`,
the intersection and the md1 binding were not modified.

---

## 1. Every operator-facing string added or changed, verbatim

### 1.1 The EXPERIMENTAL warning (item 2) — `gui/multisig_build.go`, `multisigBuildExperimentalWarningBody()`

**Removed, verbatim:**

```
This device-authored multisig policy is NOT validated end-to-end. There is no
coordinator or hardware round-trip. You MUST verify the assembled descriptor and the
shown policy stub + per-slot fingerprints against your coordinator/wallet BEFORE funding.
The fingerprint choice changes the policy id.

Hold button to confirm.
```

**Added, verbatim** (`\n` shown as line breaks; the blank line before "Hold" is a real `\n\n`):

```
Nothing outside this device has checked this policy. Before you fund it, compare the keys you just reviewed, or the descriptor on the restore doc, against the same wallet in your coordinator.
A matching fingerprint is not that check: a card states its own, and nothing binds it to the key.
What settles it is restoring these plates in your coordinator and seeing your own first receive address.

Hold button to confirm.
```

The clause "The fingerprint choice changes the policy id" was **not deleted, it moved**: the
review screen already carries `Fingerprint choice changes the policy id, so match your
coordinator.`, which is where the choice is made and where an operator can still act on it.

### 1.2 The policy review (item 1) — `gui/multisig_build.go`, `buildReviewLines()`

**Added as the FIRST line** (prepended, per the census-NOTE precedent — this screen is
confirmable from any page):

```
Check each key below against your coordinator, or against the card it came from, before you continue.
```

**Per-slot lines changed.** Removed:

```
@0  fp 73c5da0a
@0  (no fp)
```

Added — the label, then the full base58 xpub in 20-character chunks:

```
@0, fingerprint 73c5da0a:
xpub6DkFAXWQ2dHxq2vat
rt9qyA3bXYU4ToWQwCHbf
5XB2mSTexcHZCeKS1VZYc
PoBd5X8yVcbXFHJR9R8UC
Vpt82VX1VhR28mCyxUFL4
r6KFrf
```

```
@0, no fingerprint:
xpub6DkFAXWQ2dHxq2vat
...
```

Unchanged and still present: the provenance lines, the §0.1a origin announcement,
`Policy stub: %x`, `Slots:`, `Fingerprints INCLUDED/OMITTED on every slot.`,
`Fingerprint choice changes the policy id, so match your coordinator.`

**New refusal**, shown when the keys cannot be mapped (title `Build Policy`):

```
Couldn't show the keys this policy holds, so it was not engraved. Build again; if it happens twice, rewrite the payload on the host with `me sysw pack`.
```

### 1.3 The abort screen (items 3 + 5) — `gui/bundle_flow.go`, `bundleAbortWarningText()`

**Removed, verbatim:**

```
Stopped at card 1 of 3 (ms1 share). A partial bundle can't be used - discard the engraved plate(s) and start the bundle over.
```

**Added — a set that carries a seed (any `cardMS1`):**

```
Stopped at card 1 of 3 (ms1 share). This set is not a usable backup yet.
To finish it, run this again and give the same answers: it cuts the same plates, byte for byte, so you only cut the ones you are missing.
If you throw any of it away instead, a plate with your seed on it must be DESTROYED, not binned: cut it up or grind the words off.
```

**Added — a set with no seed on it:**

```
Stopped at card 1 of 3 (md1 policy). This set is not a usable backup yet.
To finish it, run this again and give the same answers: it cuts the same plates, byte for byte, so you only cut the ones you are missing.
No plate in this set carries a seed.
```

The two are reconciled deliberately rather than left in tension: the primary instruction is
**finish the set** (the re-run property), and DESTROY is scoped to *plates the operator throws
away instead*. Saying only "destroy everything" would contradict the recovery the same screen
is there to offer.

### 1.4 The passphrase (item 4) — `gui/multisig_build_census.go`

**Engrave-mode row.** `buildFullModeLabel(passphrase bool)`:

| passphrase used | label |
| --- | --- |
| no | `Full (seed + keys)` — unchanged |
| yes | `Full (seed + keys, NOT passphrase)` |

Measured against the real row rather than judged by eye: `359 px` drawn on a `436 px` row at
`sh2DisplaySize` (`assertChoiceLabelFits`). `ChoiceScreen` uses `widget.Label`, which does not
wrap, so an over-wide row is drawn off the panel.

**Restore doc**, appended to the set inventory. Passphrase used:

```
A BIP-39 passphrase WAS used. It is not on these plates and cannot be recovered from them: nothing this device engraves carries a passphrase.
Without it, these plates do not reach the money. Keep it somewhere separate, and make sure whoever needs this backup can also get the passphrase.
```

No passphrase used:

```
No BIP-39 passphrase was used, so no passphrase is needed to spend from this wallet.
```

**A wording defect I caught and fixed mid-block, worth flagging:** my first draft read
*"the seed plate encodes the words only"* and *"These plates are the whole backup."* Both are
**false in watch-only mode**, which engraves no ms1 at all. Rephrased so the claim is about the
passphrase and stays true in both modes; what the set does and does not contain is the
inventory's job immediately above.

### 1.5 The verify's no-slot refusal (item 8, F-191) — `gui/multisig_verify.go`

**Removed, verbatim:**

```
That seed is not a cosigner of the read-back policy, so it cannot prove any of these plates.
```

**Added — three states**, `multisigVerifyNoSlotBody(passphraseTyped, provedInnocent bool)`:

*A passphrase was typed and the seed fills a slot without it (the only certain case):*

```
That seed IS a cosigner of this policy, but not with the passphrase you typed: this wallet's keys come from the seed with no passphrase. Your plates are fine. Try again and skip the passphrase.
```

*A passphrase was typed and the empty one does not match either:*

```
No slot matches that seed with the passphrase you typed. Check the passphrase before you doubt the plates: one wrong character derives a different wallet.
```

*No passphrase was typed:*

```
No slot matches that seed. If this wallet was built with a BIP-39 passphrase, add it and try again: without it the same words derive a different wallet.
```

The other two arms of that switch (`That seed is a cosigner, but none of its slots were
engraved in this run...` and `That seed's slots have already been checked...`) are **unchanged**.

### 1.6 F-182 (item 6) — titles only, no body text changed

`bundleEngrave(ctx, th, title string, cards)`. The end-of-engrave ms1 reminder now uses the
caller's title: `Engrave Bundle` (bundle flow), `Engrave Single-Sig`, `Engrave Multisig`,
`Build Policy`.

---

## 2. Verbatim RED output, per item

**A — the F-185 class check (`gui/modal_fits_test.go`)**

```
# seedhammer.com/gui [seedhammer.com/gui.test]
gui/modal_fits_test.go:249:10: undefined: multisigBuildExperimentalWarningBody
FAIL	seedhammer.com/gui [build failed]
FAIL
```

**B — the EXPERIMENTAL warning** (`TestExperimentalWarningStopsTeachingAnUncheckableCheck`), 6 failures:

```
multisig_build_prose_test.go:42: the warning still tells the operator to check "per-slot fingerprints against your", which is a check that cannot fail: fingerprints are omitted by default and self-declared when present.
multisig_build_prose_test.go:42: the warning still tells the operator to check "verify the assembled descriptor and the shown policy stub", which is a check that cannot fail: ...
multisig_build_prose_test.go:52: the warning never names "keys", so it does not ask for a comparison against an independent source:
multisig_build_prose_test.go:60: the warning never says that a matching fingerprint is not the check, so an operator who learned that ritual keeps performing it:
multisig_build_prose_test.go:70: the warning never names the real backstop ("restor"):
multisig_build_prose_test.go:70: the warning never names the real backstop ("first receive address"):
--- FAIL: TestExperimentalWarningStopsTeachingAnUncheckableCheck (0.07s)
```

A **second RED on the same item**, from the margin half of the class check — my first rewrite
was too long:

```
multisig_build_prose_test.go:96: the EXPERIMENTAL warning: 355 chars drawn in full, headroom 22 chars (margin 80)
multisig_build_prose_test.go:96: the EXPERIMENTAL warning fits today with only 22 characters to spare, under the 80-character margin. F-185's own fix failed exactly here: a +65-character edit still fit, so the screen could be re-broken without turning a test red. Shorten this body rather than lowering the margin.
--- FAIL: TestExperimentalWarningStopsTeachingAnUncheckableCheck (0.08s)
```

**C — the review shows the keys**

```
gui/multisig_build_prose_test.go:119:18: undefined: buildSlotKeyStrings
gui/multisig_build_prose_test.go:137:73: too many arguments in call to buildReviewLines
	have (md.MultisigScript, [4]byte, []md.SlotInfo, unknown type, bool, nil)
	want (md.MultisigScript, [4]byte, []md.SlotInfo, bool, []string)
gui/multisig_build_prose_test.go:188:18: undefined: buildSlotKeyStrings
gui/multisig_build_prose_test.go:197:87: too many arguments in call to buildReviewFlow
	have (*Context, *Colors, md.MultisigScript, [4]byte, []md.SlotInfo, unknown type, bool, nil)
	want (*Context, *Colors, md.MultisigScript, [4]byte, []md.SlotInfo, bool, []string)
gui/multisig_build_prose_test.go:255:15: undefined: buildSlotKeyStrings
gui/multisig_build_prose_test.go:261:15: undefined: buildSlotKeyStrings
FAIL	seedhammer.com/gui [build failed]
```

**D — the abort warning + F-182**

```
gui/bundle_abort_prose_test.go:36:12: undefined: bundleAbortWarningText
gui/bundle_abort_prose_test.go:37:12: undefined: bundleAbortWarningText
gui/bundle_abort_prose_test.go:91:10: undefined: bundleAbortWarningText
gui/bundle_abort_prose_test.go:131:11: undefined: bundleAbortWarningText
gui/bundle_abort_prose_test.go:144:5: undefined: bundleSetCarriesASecret
gui/bundle_abort_prose_test.go:147:6: undefined: bundleSetCarriesASecret
gui/bundle_abort_prose_test.go:152:6: undefined: bundleSetCarriesASecret
FAIL	seedhammer.com/gui [build failed]
```

**E — the passphrase**

```
gui/multisig_build_prose_test.go:315:11: undefined: buildFullModeLabel
gui/multisig_build_prose_test.go:316:10: undefined: buildFullModeLabel
gui/multisig_build_prose_test.go:356:55: too many arguments in call to buildPlateInventoryLines
	have ([]bundleCard, bool)
	want ([]bundleCard)
gui/multisig_build_prose_test.go:357:58: too many arguments in call to buildPlateInventoryLines
	have ([]bundleCard, bool)
	want ([]bundleCard)
gui/multisig_build_prose_test.go:412:9: reg.usesPassphrase undefined (type *seedRegistry has no field or method usesPassphrase)
gui/multisig_build_prose_test.go:418:10: reg.usesPassphrase undefined (type *seedRegistry has no field or method usesPassphrase)
gui/multisig_build_prose_test.go:429:9: reg.usesPassphrase undefined (type *seedRegistry has no field or method usesPassphrase)
FAIL	seedhammer.com/gui [build failed]
```

**F — F-191**

```
gui/multisig_verify_passphrase_test.go:35:13: undefined: multisigVerifyNoSlotBody
gui/multisig_verify_passphrase_test.go:39:12: undefined: multisigVerifyNoSlotBody
gui/multisig_verify_passphrase_test.go:42:13: undefined: multisigVerifyNoSlotBody
gui/multisig_verify_passphrase_test.go:138:6: undefined: multisigVerifySeedIsInnocent
gui/multisig_verify_passphrase_test.go:145:5: undefined: multisigVerifySeedIsInnocent
gui/multisig_verify_passphrase_test.go:152:5: undefined: multisigVerifySeedIsInnocent
FAIL	seedhammer.com/gui [build failed]
```

---

## 3. The F-185 class check, and its mutation proof

`gui/modal_fits_test.go`. It does **not** budget characters — capacity depends on how the words
*wrap*, not how many there are (measured: both modal shapes drew 588 normalized characters of
short-word filler in full, while F-185's real refusal was cut at ~500). It compares the **drawn
frame** against the **source string**, binary-searching the longest prefix that reached the glass
so the failure names the cut point rather than merely reporting one.

It carries the margin F-185 says its own fix lacked. `modalBodyMargin = 80`, and the number is
F-185's own: that entry measured a +65-character mutation still fitting on the screen it had
just pinned. `modalHeadroom` binary-searches the true headroom per body, so the margin is
enforced against a measurement rather than a hope.

**Mutation proof, and it RAN** (`TestModalFitCheckCatchesATruncatedBody`, verbatim):

```
=== RUN   TestModalFitCheckCatchesATruncatedBody
    modal_fits_test.go:258: unmutated: 347 chars drawn in full, headroom 146 chars
    modal_fits_test.go:271: mutated (+120 chars past headroom): the check went RED, reporting 490 of 613 characters drawn, cut after ..."ysteeliscutsoandtheoperatormustreadevery"
--- PASS: TestModalFitCheckCatchesATruncatedBody (0.11s)
```

The test asserts **both directions** and a third guard: (a) the real production body passes,
(b) grown past its measured headroom the check goes red, (c) the reported cut is *past the end of
the known-good body* — so a "0 of N drawn" blank-frame report cannot masquerade as a truncation.
`firstModalFrame` also holds every frame to `buildWalkRasterFloor`, so a blanked frame fails as
a blanking rather than as a false truncation.

Independent evidence the margin half is load-bearing: it turned red on my own first rewrite of
the EXPERIMENTAL warning at 22 characters of headroom (RED B, second block above).

**Applied to** the EXPERIMENTAL warning, both abort arms, all three F-191 refusals, the F-182
ms1 reminder, and the new unshowable-keys refusal. Final measured headroom, all ≥ the 80 margin:

```
the EXPERIMENTAL warning:                                    347 chars drawn in full, headroom 146
the abort warning for a set carrying a seed:                 272 chars drawn in full, headroom 262
the abort warning for a public-only set:                     198 chars drawn in full, headroom 339
the verify's no-slot refusal (passphrase proved innocent):   159 chars drawn in full, headroom 397
the verify's no-slot refusal (passphrase typed):             130 chars drawn in full, headroom 436
the verify's no-slot refusal (passphrase skipped):           125 chars drawn in full, headroom 436
the end-of-engrave ms1 reminder (F-182's screen):             71 chars drawn in full, headroom 476
the build's unshowable-keys refusal:                         125 chars drawn in full, headroom 436
```

**What it does NOT cover, stated rather than implied.** F-185's second open item stands: every
*other* long modal in the firmware is still unmeasured. The check now exists and applying it is a
one-line call, but sweeping the package is not this block's scope and would fold an unrelated
diff into it. **Filed for the controller.**

---

## 4. Two real defects the new tests found

Both were invisible to string assertions, which is the point of building the pixel-level check.

**(1) The production review's first page carries no key.** `TestBuildWalkTypedSeed` failed with
the drawn page-1 frame ending exactly at the slot label:

```
"PolicyReviewCheckeachkeybelowagainstyourcoordinator,oragainstthecarditcamefrom,beforeyoucontinue.Slots@1and@2filledfromthepayload(cards2and3of4,inpayloadorder).Yourkeyorigins:m/48h/0h/0h/2h,theBIP-48pathfornativesegwit.Policystub:06215ac0Slots:@0,nofingerprint:"
```

The header — provenance plus the §0.1a origin announcement — must stay on page one, because
plan §0.1 clause 3 puts an assumption announcement on the confirmation surface and this screen
is confirmable from any page. So the header cannot be moved to make room, and the keys
legitimately begin on page two. **Resolution:** the prepended instruction is asserted on page
one (it is what sends the operator to read them), the pager affordance is drawn whenever a
second page exists, and the walk now **pages through the review** and asserts the keys on the
pages it actually turned. This is a genuine ergonomic cost of the design and is recorded, not
papered over.

**(2) `TestBundleEngraveSetAbort` asserted on a display no device has.** It used
`newPlatform()`'s 240×240 default, which `gui_test.go`'s own comment calls "a fiction that no
shipped device has". The longer abort body drew **in full** on the real 480×320 panel and was
**cut mid-sentence** on the fiction, at `"...so you only cut the ones you are"`. Since the whole
subject of that test is a screen's reachability, it now lays out at `sh2DisplaySize`; the class
check gates the same body's length there with margin. Reasoning recorded in the test.

A third, smaller one: my first draft of `TestVerifyPassphraseArmIsDecidedByARederivation` used
`fixtureMasterC` as the "foreign" seed and the test correctly failed — **master C *is* Trace B's
cosigner card at @3**. Replaced with BIP-39's all-`zoo` vector, plus an `INCONCLUSIVE` guard that
*proves* the seed fills no slot rather than assuming it.

---

## 5. Design decisions worth a reviewer's attention

**The review shows the operator's OWN xpub strings, not the md1's reconstruction — and this is
load-bearing.** Measured during the block:

```
md1-reconstructed xpub for a slot: xpub6DXuQW1Q2JpZxsEnFKrPvDuiRMmQgU4fzHU1wsvM5EqgGAWRJ3cmw...
the same key as really derived:    xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeK...
```

md1 carries 32 B chain code ‖ 33 B pubkey per `@N` and **no parent fingerprint**, so
`expandedToDescriptor` builds `bip380.Key` with `ParentFingerprint: 0` and the base58 string
differs throughout. Displaying that form would have asked the operator to compare a string that
can never match their mk1 card or their coordinator's cosigner list — a violation of "every
comparison the device asks for must be one the operator can actually perform". So
`buildSlotKeyStrings` uses the assembled md1 **for the slot→key mapping** (65-byte match, drift-proof
against the bytes that go on steel) and displays **the input string** the operator holds.

**Consequence, and I could not fix it inside this block's scope:** the restore doc's
`desc.Encode()` still renders the parent-fingerprint-zero form, so the review screen and the
restore doc spell the same key differently. This is **pre-existing** — `expandedToDescriptor` is
shared with the supply path, which this plan does not own — and it is harmless for the use the
warning names (importing the descriptor derives the same wallet and the same addresses). It would
bite an operator attempting a character-by-character comparison *between the two device screens*.
**Filed for the controller as an observation.**

**`buildSlotKeyStrings` refuses rather than rendering a blank slot.** A slot it cannot map aborts
the engrave with a named screen. This is the §0.1 clause 2 direction: a review that silently omits
a slot's key is the original defect manufactured on purpose.

**The DESTROY/re-run tension is resolved, not ignored.** See §1.3.

**`reg.usesPassphrase()` is ANY, not ALL** — SPEC 4.1 makes the passphrase per-seed, so one
passphrased leg among three is enough to make the set incomplete. An explicitly bound *empty*
passphrase is correctly not a passphrase (`syswPassphraseFlowTitled` can return `("", true)`),
and that case is tested.

**F-191's certainty is scoped honestly.** The empty re-derivation settles exactly one case: the
policy was built with *no* passphrase and the operator typed one. It cannot distinguish
"engraved with passphrase X, typed Y" from a genuinely foreign seed — so that arm says so rather
than guessing, and even the *skipped* arm stops asserting the seed is foreign, since the wallet
may have been built with a passphrase the operator declined to re-enter. None of the three
recommends re-cutting steel.

---

## 6. Gate — verbatim, unpiped, redirected to a file with the true exit code echoed

```
$ nix develop --command go test ./... -count=1     (stdout+stderr → file)
TEST_TRUE_EXIT=0
ok=51  FAIL=0

$ nix develop --command gofmt -l ./                (stderr discarded; nix prints a
GOFMT_EXIT=0                                        dirty-tree warning there)
stdout lines: 0

$ nix develop --command go clean -cache
$ nix develop --command go vet ./...               (COLD GOCACHE)
VET_TRUE_EXIT=1
findings: 40 (excluding nix's dirty-tree warning line)
findings outside _test.go: 0
findings in any file this block added or edited: 0
unique files with findings:
  backup/backup_test.go
  backup/freetext_test.go
  bspline/bspline_test.go
  engrave/engrave_test.go
  gui/freetext_sizeproof_golden_test.go
  gui/op/draw_test.go
```

Matches the stated clean baseline exactly (exit 0 / 51 ok / 0 FAIL; gofmt empty; vet exit 1 with
40 findings, none outside `_test.go`). The gate output is in the commit message.

**On `go vet`:** the earlier per-package run also surfaced
`gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later`,
which is one of the 40 and is pre-existing.

---

## 7. Follow-ups for the controller (I could not write `FOLLOWUPS.md` from the worktree)

1. **F-185's remaining half.** The class check exists and is a one-line call, but only the
   screens this block touched are gated. Every other long modal in the firmware is still
   unmeasured. Owning phase: suggest a later S5 block or S6.
2. **The two xpub renderings.** The review screen shows the real base58 xpub; the restore doc's
   descriptor shows the parent-fingerprint-zero reconstruction of the same key. Pre-existing,
   shared with the supply path (`expandedToDescriptor`, `gui/md1_expand.go`), harmless for
   import but confusing for an on-device eyeball comparison. §5 above has the measured strings.
3. **The review's first page cannot show a key** while the §0.1 clause 3 header holds page one.
   Recorded in the walk test; a page-break-aware pager (keeping a slot's label and chunks
   together) would be a real improvement and touches `confirmReviewScreen`, shared code.
4. **Watch-only sets say nothing about the absent seed.** The passphrase lines are now
   mode-safe, and the inventory lists what was cut, but no line states outright that a
   watch-only set contains no seed. Adjacent to item 4's spirit; deliberately not scope-crept.

## 8. Not done / assumed

- **Nothing in the brief was left undone.** All eight items landed with tests.
- **Assumed** the F-182 titles for the four production callers from each flow's own `showError`
  titles: `Engrave Bundle`, `Engrave Single-Sig`, `Engrave Multisig`, `Build Policy`. The last
  is the only one this block owns; the other three are asserted at their call sites by
  `TestMs1ReminderIsTitledForTheProgramThatShowedIt` so a wrong guess is visible rather than silent.
- **Assumed** that "name the external-coordinator restore (S6) as the real backstop" means naming
  the *action* an operator can take today (restore the plates in a coordinator, check the first
  receive address) rather than naming a stage that has not shipped. Naming "S6" on screen would
  be spec language, which this block's constraints forbid.
- The emulator walk and the gate-record mint were out of scope and were not touched.
