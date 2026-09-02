# Composer S3 plan — R0 round 0, FIDELITY + DESIGN lens

Reviewer: independent architect (did not author the plan).
Artifact: `design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md` (master, 7638 lines).
Spec: `design/SPEC_wallet_policy_composer.md` §4–§9, §12, §13.
Read against: `/scratch/code/shibboleth/.plan-build-gate-go-s3/wired` (READ-ONLY; the
gate's scratch copy with every extracted block plus the six hand-wired fragments),
and the fork at `/scratch/code/shibboleth/seedhammer`.

**The one question, answered: NO on both halves.** The plan transcribes §8 almost
perfectly and reasons well about the screens, but **Part B is never joined to the
flow** — thirteen of its entry points have zero non-test call sites in the gate's
own wired copy — so seating, the mapping review, §7e's self-check, §8q, the
engrave form choice, card minting and the census are all unreachable from
`walletPolicyFlow`. The plan names a `Replace gui/composer_flow.go` anchor that it
never supplies. Beneath that, the state model cannot do three things §7b/§7d
require, one refusal fires on a shape §7f explicitly supports, and one paging gate
cannot fail.

Counts: **1 Critical / 12 Important / 11 Minor / 3 Nit.**

Nothing settled in the brief was re-derived: the build gate's output, the citation
resolution (222/222) and the glyph/table checks are taken as given. Every claim
below was measured in the wired copy or in the plan text, with the command shown.

---

## What is CLEAN (stated so the fold does not re-derive it)

These were machine-checked by this review, not assumed.

**§8 verbatim (lens 1, the copy half) — clean.** I extracted every blockquote
group from spec §8 and every `verbatim` cell from `composerCopyTable()` and
compared them under whitespace normalisation:

```
spec blockquote groups: 41       plan table rows: 39
spec bodies with NO exact match in the plan table: 4
  -> all four are §8n's HOST lines (me sysw pack stderr), which are S1's, not the fork's
plan rows whose text is not a verbatim §8 blockquote: 2
  -> composerCopyLockEchoBlocks  ("N blocks (about D days)", §6b's table form)
  -> composerCopyPackedHeightBound (§8c's date body with §6b's "the packed height was H")
     — both are documented in the plan as derived, at plan:236-240 and :263-272.
```

So **every fixed §8 body the fork draws is word-for-word the spec's.** Do not
spend another round on that.

- `grep -c '^func composerCopy' gui/composer_copy.go` = **39**;
  `grep -c '^\t\t{"composerCopy' gui/composer_copy_test.go` = **39**; the AST scan
  in `TestComposerCopyTableCoversEveryBody` is a real structural gate.
- `addrProofPerChain` = **2** (`gui/wallet_policy.go:257`), matching §7e's
  "receive and change addresses 0..1".
- `md.Branch.Locks` carries the **operator-unit** value — `lockFromWire`
  (`md/compose.go:145`) strips `sequenceTypeFlag` — so `composerSelfCheck`'s
  `b.Locks[0].Value != p.Lock.Value` comparison is sound, and the state, echoes and
  tests agree on units throughout.
- `TestComposerWalletPolicyAdmitsTheComposerClasses`' loop
  `progBackupWallet..progTransaction` covers all ten programs
  (`gui/sysw_admit.go:19-28`) — the "admitted at Wallet Policy alone" assertion is
  exhaustive, and it checks refusals as well as additions.
- `composerSlotOrder` is cross-checked against `md.Composed.Slots()`
  (`TestComposerSlotOrderAgreesWithTheCodec`) rather than assumed — the right
  shape for §5's numbering, and the tr "first single-key path not first-listed"
  case is in its table.
- Paging reachability: `composerPageLines` always draws the first row, so `shown ≥ 1`
  and forward paging terminates; the pager gate `start > 0 || shown < len(lines)`
  is present in both paged screens; `composerPickScreen`'s Down reaches
  `len(lines)-1` and re-pages on `sel >= start+shown`. The stub test walks pages to
  the end and fails on a stall. Lens 3's reachability question is answered.
- §13 item 1: one measure site (`composerPageLines`), one measurement task (C1)
  that prints all four numbers and folds them into the spec with the gate output in
  the commit. Correct construction.

---

## C-1 (Critical) — Part B is written but never called; the plan's own `Replace` anchor does not exist

**Plan lines:** 4437 ("Part B replaces `gui/composer_flow.go` wholesale (the gate's
``Replace `gui/composer_flow.go` `` anchor)"), 4869 (`func composerFlow`, Part A's
version, the only one in the plan), 1946 (`walletPolicyFlow` calls it).

**Spec:** §7d, §7e, §7f, §9 items 7, 9, 10; §12 items 4, 6, 9.

**Reproduction.**

```
$ grep -n 'Replace `' design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md
54:   ... (the sentence describing the gate's anchors)      <- the only hit
```

There is no `Replace gui/composer_flow.go` block anywhere in the plan. In the
gate's own wired copy:

```
$ cd /scratch/code/shibboleth/.plan-build-gate-go-s3/wired
$ for f in composerSeatFlow composerMappingReview composerConsentFlow \
           composerFormPick composerMintCards composerCensusLines \
           composerSecretFormPick composerEngraveModePick composerShortfall \
           composerApplyShapeEdit composerKeySources composerCardSources \
           composerSeatingComplete; do
    echo "$f: $(grep -rn "$f(" gui/*.go | grep -v _test.go | grep -v "func $f" | wc -l)"
  done
composerSeatFlow: 0          composerMappingReview: 0     composerConsentFlow: 0
composerFormPick: 0          composerMintCards: 0         composerCensusLines: 0
composerSecretFormPick: 0    composerEngraveModePick: 0   composerShortfall: 0
composerApplyShapeEdit: 0    composerKeySources: 0        composerCardSources: 0
composerSeatingComplete: 0
```

`composerSelfCheck` has one non-test caller — `composerConsentFlow` — which itself
has none. `composerMintCard`'s one caller is `composerMintCards`, which has none.

`gui/composer_flow.go` in the wired copy is Part A's function verbatim:
wrapper → `composerShapeFlow` → `md.Compose` → `composerStubFlow` →
`composerConsentLines` → `composerReadScreen` → §8l → `composerEngraveTemplate`.
It never populates `st.sources`, never calls `composerSeatFlow`, and never runs the
self-check.

**Consequences, each a spec clause unmet in the shipped binary:**

1. **§7d seating is unreachable.** An operator with a payload of `key:` records
    reads the door's "Keys loaded: 4", chooses Build, and is never offered a slot.
2. **§7e's self-check never runs.** §7e states the check unconditionally — it is
    the mechanism that stops "a builder defect in the shape, the seating, the
    origins, the fingerprints or the use-site" reaching steel as a reviewed wallet.
    §8q is dead copy on a shipped device.
3. **§7f's form choice, card minting and census never run.** `composerEngraveTemplate`
    cuts one md1 with no form choice and no card census.
4. **§7b's live line is always wrong.** `composerSlotsKeysLine` reads
    `st.sources`, which is empty for the whole flow, so the path-list screen prints
    `keys available: 0` whatever the payload holds.
5. **§7d's discard rule is prose-only.** B3 (plan:5974) says *"Wire
    `composerApplyShapeEdit` around the two edit calls in `composerShapeFlow`"* and
    supplies no code. The gate's extractor only assembles ```go blocks, so the wired
    version has never been compiled, let alone run.

**Why the gate could not catch this.** Go does not error on unused package-scope
functions — only on unused imports and local variables. The plan's gate line
(plan:7638) correctly names what it does not cover, but "an unreachable feature"
is not on that list and should be: this is the exact class recorded in memory as
*"plans list components and omit the call that joins them; six green stages shipped
an inert feature."*

**Hypothesis (not prescriptive).** Part B needs its own
``Replace `gui/composer_flow.go` `` block giving the full seated flow — sources
gathered before the shape (so §7b's line is right), seating between the stub screen
and consent, `composerMappingReview`, `composerConsentFlow`, `composerFormPick`,
`composerMintCards`, `composerCensusLines` — plus a Go walk test that drives it from
the door to the census. Without a flow-level test, the same defect recurs silently;
a `grep` assertion that every `composer*Flow`/`composer*Pick` has a non-test caller
would be a cheap structural gate.

---

## Important

### I-1 — the §4f invariant fires on unseated slots, refusing a shape §7f supports

**Plan:** 6194-6220 (`composerInvariantViolation`), 6359-6363 (mapping review),
6921 (self-check). **Spec:** §4f invariant, §7f partially-seated form, §8p, §8v,
§12 item 6.

`composerInvariantViolation` groups `st.assigned` by `composerOriginKey(a.origin)`.
An **unseated** slot is `composerAssignment{src: -1}` with `origin == nil`, so its
key is `""`. Two or more unseated slots therefore land in one bucket, neither has a
fingerprint, and the function returns `true`.

Reproduction: any partially seated composition with ≥2 unfilled slots — precisely
§8p's fallback — is refused at the mapping review with §8v ("Two keys declare the
same origin and not both carry a fingerprint"), and, once C-1 is fixed, fails
`composerSelfCheck` and shows §8q ("The policy on this device does not match what
you built") on a build that is entirely correct.

The check is also being run on the **wrong data**: §4f says the invariant holds on
the *produced template*, where unseated slots take "the LOWEST account not already
declared", and §7e re-checks it *on the decoded md1*. `composerInvariantViolation`
reads composer UI state, where those origins do not yet exist.

Untested in either direction: `TestComposerInvariantRefusesTwoSlotsAtOneOrigin…`
uses two seated assignments only; `TestComposerRefusesTwoSlotsResolvingToTheSameXpub`
exercises `{src:-1},{src:-1}` against the *xpub* check, not this one.

Hypothesis: skip `src < 0` entries in the UI-state pass, and run the real invariant
over `md.ExpandWalletPolicyChunks(chunks)`' declared origins inside `composerSelfCheck`.

### I-2 — the consent screen mis-numbers paths for a taproot policy with an extracted internal key

**Plan:** 4743-4744 (`composerBranchLines`: `head := fmt.Sprintf("Path %d: ", idx+1)`),
4792-4794 (the loop over `shape.Branches`). **Spec:** §7d ("'Path N' … is the
OPERATOR's listed path index, never an emitted leaf index"), §7e ("per path in
listed order").

Measured: `md.PolicyShape.Branches` is *leaves* — the taproot internal key is
reported through `KeyPath`, not as a `Branch` (`md/policy_shape.go:43-45`, `:99-110`).
`composerLeafPaths` (plan:6809) knows this and removes the extracted path; the
consent renderer does not.

For `tr` with `Paths = [P1: 1-of-1, P2: 2-of-3]`, §5 extracts P1 as the internal key.
`shape.Branches` then holds one entry, and the consent prints **"Path 1: 2-of-3"** —
the operator's Path *2*. Meanwhile `composerSeatPrompt` for slot @1 says
"Slot @1, Path 2 key 1 of 3". The two screens disagree about which path is which on
the surface that consents to steel, and the key-path line names no listed path at all.

Hypothesis: pass the operator's path index alongside each branch (the mapping
`composerLeafPaths` already computes) rather than the branch ordinal.

### I-3 — Back at the path list abandons the whole composition; §7b's rule has no test anywhere

**Plan:** 4886-4888 (`if !composerShapeFlow(...) { return }`), 2892-2896
(`composerShapeFlow` returns false on Back at the pick screen). **Spec:** §7b
("Back preserves everything — 'going back should lose nothing'"), §12 item 5's
condition tests.

`composerShapeFlow` returns `false` whenever `composerPickScreen` declines, and
`composerFlow` then `return`s — dropping the wrapper, every path, every lock, every
digest and every confirm already given. §7b's own step is "Wrapper → preset or
blank → paths", so Back from the path list should land on the wrapper pick with the
list intact.

Second half, equally load-bearing: **no test in the plan drives Back and asserts
state survived.** The brief asks whether each Back behaviour "is asserted by a test
that would fail if Back lost state"; grepping the wired copy, the answer is no for
every one of them — `TestComposerDiscardIsSilentWithNothingSeated` and
`TestComposerShapeSignature…` are unit tests over predicates, not walks.

### I-4 — the wrapper cannot be changed after the initial pick, so §12 item 4's wrapper vector is unreachable

**Plan:** 4878-4883 (`composerWrapperPick` runs once, outside the loop), 2878-2884
(the path-list rows are paths + "Add a spend path" + "Done"). **Spec:** §7g row
"shape | edits the wrapper … after a slot was assigned | WARNING before the edit;
assignments discarded (§8j)"; §12 item 4 "a path-count edit **AND a wrapper change**
after a slot was assigned (discard)".

There is no UI affordance for changing the wrapper. `composerShapeSignature`
correctly includes it (plan:5932-5943) and `TestComposerShapeSignature…` asserts it
moves the signature — but the operator can never trigger that path, so §12 item 4's
named vector cannot be produced through the flow, and §7g's row is unimplementable.
A signature test is not the acceptance §12 item 4 asks for.

### I-5 — `composerKeysEdit` destroys an existing key set when the operator Backs out of the sorted choice

**Plan:** 2754, 2762 (`st.list.Paths[idx].Keys = nil`). **Spec:** §7b Back rule.

`composerKeysEdit` writes `st.list.Paths[idx].Keys = set` **before** the "Key order"
ChoiceScreen, then on Back (`!ok`) or on a declined §8b confirm sets `Keys = nil`
and returns false. `composerPathEdit` (plan:2833-2836) ignores that return value.

So: a path that already held a 2-of-3; the operator re-enters Keys, re-picks 2 of 3,
then Backs at "Key order" — the path now has **no key set at all**. It either
becomes a lock-only path (refused by §4e at Done, with §8m line 2 blaming the
operator for something the UI did) or silently a hash-only path. Restoring the
previous value, not `nil`, is the correct decline.

### I-6 — the §8a/§8b confirms are keyed by path index and go stale on removal, so an unskippable confirm can be skipped

**Plan:** 2812-2818 (`st.keylessConfirmed[idx]`), 2759-2765
(`st.unsortedConfirmed[idx]`), 2838-2840 (`Remove path` splices the slice).
**Spec:** §7b, §8a, §8b, C16 ("confirm-to-proceed, neither is dismissible").

`composerAddPath` appends at `idx := len(st.list.Paths)` and records the confirm
under that index; `composerPathEdit`'s "Remove path" splices the slice and leaves
both maps untouched. Reproduction: add path 1 as key-less, confirm §8a
(`keylessConfirmed[0] = true`); remove path 1; add a **new** key-less path — it is
again index 0, `keylessConfirmed[0]` is still true, and the EXPERIMENTAL
confirm-to-proceed **does not fire**. The same construction skips §8b for a new key
set at a reused index.

An unskippable confirm that can be skipped is the defect class §12 item 4 exists
for, and no test removes a path.

### I-7 — the pager-gate test cannot fail for the reason it states

**Plan:** 789 (`if longInk <= shortInk`), 741-790
(`TestComposerReadScreenDrawsThePagerOnlyWhenASecondPageExists`). **Spec:** §7c,
§7e, §9 items 6 and 7, §12 item 5 ("the variable-length screens … are asserted by
PAGING capacity").

The test compares total frame ink between a **1-line** screen and a **64-line**
screen and concludes "the pager icon should make the two-page frame strictly
heavier". Measured semantics: `ink()` is a lit-pixel count for the whole frame
(`gui/raster_test.go:24`, `assertFrameHasBody` at `:80`), so the difference is
dominated by roughly ten drawn body rows. The assertion passes if the pager is
always drawn, and passes if it is never drawn. The one behaviour the plan inherits
from `confirmReviewScreen` — "a control that is present and inert teaches the
operator that controls here may be inert" — is therefore unasserted.

Hypothesis: hold the body constant and vary only whether a second page exists (the
same line count at two display heights, or a one-page and two-page body of equal
first-page content), or assert on the drawn nav-button count rather than on ink.

### I-8 — §12 item 5's gates are missing for §8m entirely, and for §8c and §8r

**Spec:** §12 item 5 ("the modal-fits assertion … on every §8 body and every new
screen; plus a fires-on-condition test for each of … §8m … §8r …"); the plan's own
Global Constraint at plan:40-41 ("Three copy gates per §8 body, plus one condition
test").

Measured in the wired copy — every `assertModalBodyFits` call site enumerated, then
each `composerCopy*` function checked for one:

| §8 section | bodies | modal-fits | driven onto a frame |
| --- | --- | --- | --- |
| §8m (five structural refusals) | 5 | **none** | **none** |
| §8c (echoes + bound lines) | 7 | **none** | none (string-level only) |
| §8r (door key-state lines) | 6 | **none** | one line, in the door walk |
| everything else | 21 | yes | yes |

The five §8m bodies are drawn through `showError`, i.e. exactly the
`errorScreenBody` surface `assertModalBodyFits` measures, so there is no
"paged screen" carve-out for them. `TestComposerShapeRefusalsActuallyRefuse`
(plan:2088-2124) checks `composerRefusalBody`'s *mapping*; nothing renders a
refusal. §8m line 5 (the 33rd slot) is reached in `composerKeysEdit` at plan:2718
and never exercised.

### I-9 — §8i's "and at consent" half is absent

**Spec:** §6c ("At entry **and at consent** the device states the 32-byte rule");
§8i's own heading, "Hashlock entry rule (at entry and at consent)". **Plan:**
4771-4839 (`composerConsentLines`) contains no `composerCopyHashRule()` call;
`TestComposerHashRuleIsStatedAtEntry` (plan:3775) asserts entry only.

A composition with a hashlock consents without ever restating the rule whose whole
purpose is to prevent an unspendable wallet — "the reference wallet's own README
records months of exactly that" (§6c).

### I-10 — §7f's form A and the secret plate have no builder, and the "cut ONCE" rule has no implementation

**Plan:** Task B7 (6960-7125) delivers `composerFormsFor`, three labels and two
pickers, and nothing that produces a plate. **Spec:** §7f, §9 item 10.

Missing, each a named §7f/§9 clause:

- **Form A** ("plain-text plates, QR plates, or keyed md1 strings"): nothing builds
  the concrete descriptor text or the QR plates "via the transaction program's plate
  machinery" (§9 item 10). `composerCensusRefusal` (plan:7372) takes a
  `descriptor string` that nothing produces, and `composerDescriptorPlateFits` /
  `composerDescriptorCeilingChars` have no caller outside tests.
- **The secret in Full mode**: `composerSecretFormPick` returns a
  `composerSecretForm` that nothing consumes; neither `engraveSeed` nor
  `engraveCodex32` is ever called.
- **"a seed that filled several slots is cut ONCE"** (§7f, verbatim): no dedup
  exists and no test asserts it. With one seed at three slots the plan has no
  mechanism that would prevent three secret plates.

### I-11 — Part A's declared standalone exit (§12 item 3) is not discharged, and Part A alone breaks §7e and §7f

**Plan:** 80 ("Part A's exit is spec §12 item 3"), 4617-4643
(`TestComposerNoPayloadWalkReachesAKeylessTemplateThatDecodes`).

The test's name promises the walk; its body asserts **only that the door drew**
("(1) THE DOOR…"). It never picks a wrapper, never adds a path, never reaches the
stub screen, never consents, never engraves. §12 item 3 requires "door line present,
shape, stub screen with per-slot expected origins, consent stating no addresses,
form choice collapsed, keyless-template engrave whose md1 decodes". Only the last
clause is covered, and by a separate artifact-level test
(`TestComposerKeylessTemplateDecodesOnTheDevice`), not by a walk.

Two further §12 item 3 clauses cannot be satisfied by Part A as written:

- **"form choice collapsed"** — Part A's `composerEngraveTemplate` (plan:4930) has
  no form choice, so §7f's "the choice collapses to 'template only' and says so" is
  not said.
- **§7e's self-check** is unconditional in the spec and absent from Part A's consent
  path (see C-1). Shipping Part A alone ships a consent surface §7e does not
  sanction.

Consistency with the controller's "Part A ships alone" default: it holds for the
**no-payload** journey and only that one. With a payload loaded, Part A's door
states "Keys loaded: N" (§8r) and then offers a Build that can never use them — the
door promises a capability the shipped half does not have.

### I-12 — the blast-radius table covers Go tests only; the door hangs three emulator walks

**Plan:** 4949-4986 (the five shipped Go tests, correctly measured and updated).
**Missing:** the emulator.

Measured in the wired copy:

```
$ grep -rln -i "wallet *policy" cmd/emu/*.js
cmd/emu/shots_seating.js      cmd/emu/shots_tr_pathological.js      cmd/emu/shots_walletpolicy.js
```

All three share the same entry (`shots_seating.js:143-146`):

```js
// (3) Enter it. The gather screen is what it opens on.
await tap(CONFIRM);
await waitFor("md1descriptors:0");
```

With the door in front of `walletPolicyFlow`, `CONFIRM` opens the door and
`md1descriptors:0` never draws; all three time out. `shots_walletpolicy.js` is
driven by `design/journeys/capture_walletpolicy.py` in this repo, so the journey
capture breaks too. The plan's gate line explicitly says it "does not cover … the
emulator", which is exactly why these belong in the blast-radius table with their
door step, as the five Go walks got.

Also in this class, lower stakes: `cmd/emu/shots_seating.js:211-215` waits on the
literal `"-ID:"`. After the A9 relabel `templateConsentLines` prints no `-ID:` at all
(measured: the only remaining sources are `md/template_id.go:152` and comments), so
whether that walk still finds its needle depends on which consent screen it lands
on. Worth confirming when the three walks are updated.

---

## Minor

- **M-1 (plan:2780-2785)** `composerAddPath` refuses a 9th path with an ad-hoc
  string ("This wallet already has %d spend paths, which is the most this build
  writes."). §4e rules that refusal "at the picker (the picker does not offer the
  value)", and §11 requires every refusal's copy to be a §8 blockquote or a quoted
  string in its table so the glyph and modal-fits gates cover it. This one is
  neither. Either stop offering "Add a spend path" at `md.ComposeMaxPaths`, or give
  the string a §8 home.
- **M-2 (plan:4756-4757)** The consent renders locks with `composerLockShort` — the
  path-list *row* form — where §7e asks for "its lock kind and value in operator
  units (§6b echo form)". The operator sees "1000 blocks" instead of §8c's
  "1000 blocks (about 6.9 days)", and "2027-03-01" instead of "2027-03-01 00:00 UTC".
- **M-3 (plan:4884, 4901-4919)** `edited` is set on *any* Back out of the stub,
  consent or §8l screens, not on an actual edit. Returning to the path list and
  pressing Done unchanged re-shows the stub screen carrying §8s's "The shape changed,
  so this id changed. Cards minted with the old stub will not seat here." — false, and
  it invites the operator to discard a correct written-down stub. Compare the
  signature instead (`composerShapeSignature` already exists).
- **M-4 (plan:3651)** The date pad's echo appends the raw Unix operand
  (`" (" + strconv.FormatUint(uint64(u), 10) + ")"`). §6b's whole design is that
  "the operator never types a raw operand"; §8c's date body is
  "2027-03-01 00:00 UTC" and nothing else.
- **M-5** §7e and §9 item 9 name `confirmReviewScreen`'s paged form as the consent
  surface. Part B's `composerConsentFlow` uses it (plan:6950); Part A's flow uses
  `composerReadScreen` (plan:4910). Two consent paths on two surfaces with different
  Back semantics is one decision described twice.
- **M-6 (plan:3952)** `composerHashEdit` shows §8i unconditionally on entry,
  including when the operator's next choice is "No hash lock" — a modal in front of a
  clear.
- **M-7 (plan:7235 vs 7381)** The B9 Interfaces line declares
  `composerCensusLines(pl Platform, cards []bundleCard, descriptor string)`; the code
  declares `composerCensusLines(params engrave.Params, cards []bundleCard)`. The
  `descriptor` argument — the one the ceiling refusal needs — is absent from the real
  signature, which is why `composerCensusRefusal` has no caller.
- **M-8 (plan:5713)** `composerSeedAccountFor(st, slot uint8, seedID int)` indexes
  `st.sources[seedID]` — it is a **source index**, not `composerSource.seedID`.
  `composerSeedDerive` passes `srcIdx`. The name collides with a real field of a
  different meaning on the same struct.
- **M-9** §7b's step is "Wrapper → **preset or blank** → paths". `composerFlow` goes
  wrapper → `composerShapeFlow` with no preset-or-blank screen and no call site for
  `composerPresetPick`, so A10 has nowhere to land when F-453 ships. Consistent with
  the controller's "blank-shape first" default, but the wiring point should be named
  now so the blocked task is a fill-in rather than a redesign.
- **M-10 (plan:1)** The STATUS line's `FORK_REPO=/scratch/code/shibboleth/wt-composer-s2`
  no longer exists (measured 2026-09-02: `git worktree list` does not list it), and
  **S2 has merged** — fork `main` is `321acb5` ("merge: composer S2 …"), while the
  plan's staleness baseline is still `169073c`. PRECONDITION 1 is now satisfied; the
  re-validation the plan mandates should be run against `321acb5`, not `169073c`.
- **M-11 (plan:885-889)** `composerPageLines` increments `shown` and *then* breaks on
  `y > contentBottom`, so the last counted row may extend past the content box. The
  reported per-frame capacity is the number §13 item 1 records, so it can be one
  greater than the number of fully drawn rows.

## Nit

- **N-1** §7a: "**Beneath** Build the door states the key state." The plan puts
  §8r's lines in the `ChoiceScreen.Lead`, i.e. above the choices (plan:1927-1935).
  The plan's reason is sound (the Lead wraps, rows do not) — worth a spec fold rather
  than a code change.
- **N-2 (plan:1013-1019)** After paging, `composerPickScreen` clamps `sel` upward
  only (`if sel < start`), never on `sel >= start+shown`, so the cursor can sit off
  the drawn page until the next Up/Down.
- **N-3** §8n's four host lines are correctly absent from the fork's copy table
  (they are `me sysw pack` stderr), but the plan's scope statement never says so;
  §12 item 5 lists "§8n (host)" among the bodies needing a fires-on-condition test,
  and a reader checking coverage has to work out that S1 owns it.

---

## Lens-by-lens summary

| lens | verdict |
| --- | --- |
| 1. Spec fidelity, clause by clause | §8 copy **clean** (machine-diffed, 37/37 in-scope bodies verbatim). Clause coverage: gaps at §6c/§8i-at-consent (I-9), §7f form A + secret plate + cut-once (I-10), §12 item 4's wrapper vector (I-4), §12 item 5's §8m/§8c/§8r gates (I-8). |
| 2. State and Back | **Fails.** Back at the path list abandons everything (I-3); the sorted-choice Back nils an existing key set (I-5); confirm maps go stale on removal (I-6); the changed-id line can be false (M-3); §7d's discard/keep is prose-only wiring (C-1.5) and no test drives Back at all (I-3). |
| 3. Paging honesty | **Mostly sound.** Per-frame measurement at render time, one measure site, pager gate present, last page reachable, §13 numbers recorded by C1. Two defects: the pager test cannot fail (I-7) and `shown` can over-count by one (M-11). |
| 4. Seating correctness | Numbering, per-master ordinal accounts and the scrub seam are right and cross-checked. The §4f invariant refuses correct partially-seated builds (I-1); everything else is unreachable (C-1). |
| 5. Consent from the decoded md1 | The derivation and the fault-injection hook are **exactly right** in construction — and the check never runs (C-1). Lock kinds come from `Branch.Locks`, not UI state (verified). Path numbering is wrong under tr (I-2); §8i missing (I-9). |
| 6. Engrave forms | `composerFormsFor` matches §7f's three states; F-455 is a good, honest resolution of the three-vs-two secret forms. But no builder exists for form A or the secret plate, and no cut-once rule (I-10); nothing is wired (C-1). |
| 7. Blast radius | Go tests: **excellent** — five broken walks found by a whole-package sharded run, each updated with its route, none deleted, and the table-driven one guarded per row. Emulator: **missing entirely** (I-12). The oracle widening (B1 step 4) is a genuine improvement that closes three pre-existing unchecked sites. |
| 8. TinyGo fitness | No new module dependencies (verified: the import blocks name only `gui`-internal and already-imported packages; `encoding/json` is test-only). Per-frame allocation in `composerPageLines` matches `confirmReviewScreen`'s pattern; `composerReadScreen`/`composerPickScreen` add one `append` per frame — acceptable, worth a comment. Line-building is hoisted out of the draw loop everywhere I checked. The firmware size delta **is** measured (C2 step 4), with the right reasoning that a zero delta would itself be a defect. No finding. |

## What I did not review

Spec correctness (closed under its own R0); the build gate's own results; citation
resolution; the Rust/host halves (S1, S2); the presets task beyond its precondition
and its wiring point; the emulator journey (§12 item 2, S4's, and the plan correctly
does not claim it).
