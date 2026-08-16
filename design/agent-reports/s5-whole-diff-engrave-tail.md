# S5 whole-diff review — LENS: THE ENGRAVE TAIL and THE DEDUPE

Artifact: `/scratch/code/shibboleth/wt-s5`, `git diff main..s5-multislot`, frozen at `7da66bd`.
Reviewer context: read-only on the worktree; every measurement below was taken either
read-only on the worktree or on a private copy minted with
`git archive 7da66bd | tar -x -C <scratch>` (so the bytes are the commit's, not the
working tree's).

The one question: **does the tail cut exactly the right plates, exactly once, in the
required order?**

---

## PROCESS ALERT — the "frozen" tree was written to during this review

At one point mid-review `git -C /scratch/code/shibboleth/wt-s5 status --short` reported:

```
 M gui/multisig_verify.go
?? gui/probe_open0_skeptic_test.go
```

and `gui/multisig_verify.go:596` held a mutation (`declared and not used: ms1Readback`,
i.e. an `ms1Readback` assignment removed — the shape of a "verify-ms1-leg-dropped"
mutant). A `cp -a` of the tree taken at that instant did not compile. The modification
was reverted a few minutes later and the tree is clean again; `git diff --stat HEAD --
gui/multisig_build_tail.go gui/multisig_supply_tail.go gui/multisig_engrave.go
gui/multisig_build.go gui/multisig.go gui/multisig_build_census.go
gui/multisig_build_slots.go` was empty, so nothing I read was contaminated.

Raised because a concurrent writer on a tree declared frozen silently invalidates any
test result any reviewer takes from it. Recommend future parallel gates copy out of
`git archive <sha>` rather than reading the shared worktree.

(Also observed: the shared scratchpad already contains a sibling lens's 28-mutation
campaign, `results.txt` / `mutlist.txt`. My Finding 1 below independently reproduces what
that campaign recorded as `M14-build-both-uses-derived-origin SURVIVED`. Treat as one
finding, not two.)

---

## FINDINGS

### 1. IMPORTANT — the `both`-slot engrave origin is not pinned by any test, on the exact branch S5 newly opened

**File:** `gui/multisig_build_tail.go:92-99` (the `slotFromBoth` arm of `buildEngraveTail`)

S5's headline claim is "one mk1 per held slot AT THAT SLOT'S OWN ORIGIN", and for a
`both` slot "that slot's own origin" is the CARD's declared path (SPEC M-B makes the card
authoritative), not `derivedSlotOrigin(script, s.Account)`. The shipped code is CORRECT.
**Nothing in the suite can see it go wrong.**

Mutation applied to a pristine `git archive 7da66bd` copy:

```go
-		o, err := bip32.ParsePath(cards[s.Card].Path)
+		_, err := bip32.ParsePath(cards[s.Card].Path)
 		if err != nil {
 			return nil, nil, nil, errBuildUnreadableCard{Slot: slot}
 		}
-		origin = o
+		origin = derivedSlotOrigin(script, s.Account)
```

RUN: `nix develop --command go test ./gui/ ./oracle/ -count=1` → **exit 0**
(baseline on the same copy: exit 0, `ok seedhammer.com/gui 70.5s`, `ok seedhammer.com/oracle`).
Log: `<scratch>/tail-lens/out-T6.txt`.

**The shape is reachable through the shipped screens, with the delivered payload.** The
payload roster carries `A@1` at `m/48h/0h/1h/2h` (roster index 3). An operator holding
master A picks `SelfFromCard`, asserts that card at `@0`, and the seed↔key gate PASSES —
it derives at the CARD's origin, so a non-default account is admitted by design now that
S5 deleted S2's foreign-origin refusal. Measured (`zzreach_test.go`, run on the pristine
copy):

```
card path = m/48h/0h/1h/2h
gate err = <nil> notices = []
policy slot @0 origin=m/48h/0h/1h/2h
policy slot @1 origin=m/48h/0h/0h/2h
REACHABLE: slots=[0] engraved mk1 path=m/48h/0h/1h/2h
           (card says m/48h/0h/1h/2h, derivedSlotOrigin would be m/48h/0h/0h/2h)
```

**Concrete failure scenario if this ever regresses (and nothing would say so):** with the
mutation applied, that same input engraves

```
ENGRAVED AT m/48h/0h/0h/2h, the card declares m/48h/0h/1h/2h
engraved xpub xpub6DkFAXWQ2dHxq2va != card xpub xpub6DzhyrnFFYQ1HimD
```

— a key plate carrying a key the assembled md1 does not seat at that slot, at a path the
policy does not declare. The verify that would catch it is behind a
`Verify now / Skip` ChoiceScreen (`gui/multisig_build.go:396-401`), so on Skip the wrong
plate is the operator's only record of a slot they can no longer prove.

The `derived` arm IS pinned (the sibling campaign's `M12-scripttype-not-template-aware`
is killed by 3 tests, and I confirmed sh(wsh)→`1'` end-to-end below). It is specifically
the `both` arm — the arm whose whole reason for existing is that S5 now admits origins
the device did not choose — that no test reaches.

**Fix shape:** one test asserting `mk.Decode(leg.MK1).Path == cards[s.Card].Path` and
`.Xpub == cards[s.Card].Xpub` for a `both` slot whose card is at a NON-default account.
`dupTestCard(t, 3)` is already that card. It is ~15 lines and it kills the mutant.

---

### 2. IMPORTANT — the restore document collapses N per-seed passphrases into one boolean, and can silently omit a required spending factor

**Files:** `gui/multisig_build_slots.go:230` (`usesPassphrase`), `gui/multisig_build.go:339`,
`gui/multisig_build_census.go:108-131` (`buildPassphraseInventoryLines`),
`gui/multisig_build_census.go:140-145` (`buildFullModeLabel`).

SPEC 4.1, quoted in S5's own code (`gui/multisig_build.go:529-541`), makes the
`(seed, passphrase)` PAIR the derivation unit and asks the passphrase PER SEED, precisely
because "one flow-global passphrase applied to N seeds would mint keys the operator can
only re-derive with a pairing they never chose".

S5 then routes that per-seed model through `reg.usesPassphrase()`, which is `any()`:

```go
func (r *seedRegistry) usesPassphrase() bool {
	for _, s := range r.seeds {
		if s.Passphrase != "" {
			return true
		}
	}
	return false
}
```

That single bool is the ONLY passphrase signal reaching the engrave-mode label and the
restore document. `usesPassphrase`, `buildPassphraseInventoryLines` and
`buildFullModeLabel` are all NEW in this diff (`git log -S` → `023505c`, "S5: the screens
an operator reads before putting a seed on steel"); `main` has zero occurrences of either
symbol. So this is S5's own text, not inherited.

**Measured** on the pristine copy (`zzpp_test.go`): two held slots, the SAME twelve words
entered twice (which is what `buildSeedForSlot` does — it calls `reg.add` unconditionally,
once per held slot), with passphrases `"alpha"` and `"beta"`:

```
leg 0 slot=@0 path=m/48h/0h/0h/2h fp=8aaa4f4b ms1="ms10entrsqqqq…34v7f"
leg 1 slot=@1 path=m/48h/0h/0h/2h fp=d70ed067 ms1=""
ms1 plates cut = 1 for 2 required passphrases
usesPassphrase() = true (ONE bool for 2 registered pairs)
restore doc: This backup is 11 plates:
restore doc: ms1 secret share: 1 plate (secret seed backup)
restore doc: mk1 key 1 of 2: 2 plates (account key card)
restore doc: mk1 key 2 of 2: 2 plates (account key card)
restore doc: md1 descriptor: 6 plates (wallet policy descriptor)
restore doc: If any of them is missing, this backup is incomplete.
restore doc: A BIP-39 passphrase WAS used. It is not on these plates and cannot be
             recovered from them: nothing this device engraves carries a passphrase.
restore doc: Without it, these plates do not reach the money. Keep it somewhere
             separate, and make sure whoever needs this backup can also get the
             passphrase.
engrave-mode label: "Full (seed + keys, NOT passphrase)"
```

Note the singulars: *"A BIP-39 passphrase"*, *"It is not on these plates"*, *"Without it"*,
*"Keep it somewhere separate"*, *"the passphrase"*. The build requires **two** distinct
passphrases. The plate set is complete and the document explicitly vouches for it
("If any of them is missing, this backup is incomplete") — and there is nothing anywhere,
on steel or on that page, from which a reader could learn that a SECOND passphrase exists.

**Concrete failure scenario.** 3-of-4 build (Trace B's shape), operator holds three slots,
two of them from one word set under two different passphrases. They follow the document,
record "the passphrase", store the plates. Years later the recoverer has words₁, words₂,
one passphrase, and two `mk1` plates that both declare `m/48h/0h/0h/2h` and differ only in
a master fingerprint they cannot compute without already knowing the missing passphrase.
Two legs of three recover; the wallet is 3-of-4; the funds are unreachable. Silent, and
the backup asserts it is complete throughout.

The ms1 dedupe itself is NOT the defect — `ms1` encodes the words, both slots share them,
and a second identical seed plate would be pure risk. The defect is that the document
describing that collapsed set reports a flow-global passphrase for a per-seed model.

The milder and much more common form is the same bug: two DIFFERENT masters, one
passphrased and one not, cuts `ms1 secret share 1 of 2` / `2 of 2` with no fingerprint and
no attribution, and the document cannot say which of the two plates the passphrase belongs
to.

Note also that the "Key sources" review screen (`buildSlotSourceLines`,
`gui/multisig_build_slots.go:579-628`) is no help: in this shape it prints
`@0 yours: derived from your seed for @0` / `@1 yours: derived from your seed for @1`,
with no account suffix on either (both get account 0, because `nextAccount` keys on
MasterFP and a passphrase changes the MasterFP) and no passphrase column.

**Fix shape:** make the passphrase line count and attribute — e.g. "2 of the 3 seeds in
this build used a BIP-39 passphrase, and they are NOT the same passphrase", plus a
per-`ms1`-plate marker. The registry already holds everything needed
(`registeredSeed.Label`, `.Passphrase`, `.MasterFP`).

---

### 3. MINOR — two comments in this diff assert a single-select picker that this same diff replaced

**Files:** `gui/multisig_build_slots.go:40-44`, `gui/multisig_build.go:869`

`gui/multisig_build_slots.go:40-44`:

> "What has NOT moved is the @S PICKER, which is still single-select, so a build driven
> through the screens produces exactly one held slot and the multi-slot shapes are
> exercised by tests driving the model directly. The multi-select screen, and the
> multi-seed entry it implies, belongs to the block after this one."

`gui/multisig_build.go:869`:

> "…ascending and distinct; the picker produces exactly one today and the model accepts
> several."

Both were true at `7910e00` (S5 A+B) and were falsified by `4b10319` (S5 C) in the SAME
branch: `multisigSelfSlotPickFlow` (`gui/multisig_build.go:808-840`) loops
"Do you hold another slot? YES, ONE MORE" until the operator says no, and
`gui/multisig_build.go:944-955` is wired to it and says so ("A SET, from S5's multi-select
picker"). So the file contains both claims.

Not a behaviour defect — filed because these are **reachability** claims, and a reachability
claim is exactly the kind a reviewer takes as a given. Reading `multisig_build_slots.go:40`
alone, Finding 2's whole scenario ("the operator enters two seeds") looks like a shape the
screens cannot produce.

---

### 4. NIT — the engraved plate labels never name the slot

**File:** `gui/multisig_engrave.go:78-83` (`numberedLabel`)

The device labels multi-plate sets `mk1 key 1 of 3` … `3 of 3`. The oracle's own labels for
the same set are `(mk1 key, slot 0)` … `(mk1 key, slot 2)`
(`oracle/gaterecords/S5-trace-b.expect.json`). The slot index is available at the call site
(`buildEngraveTail` returns `slots` parallel to `mk1s`) and is what the census, the restore
inventory and every verify message speak in. Usually recoverable by decoding the plate's
origin path — except in Finding 2's shape, where two plates declare the same path.

---

## WHAT I VERIFIED AS SOUND (negative results, so a later round does not re-derive them)

* **Emission order satisfies the consecutive-run contract, and the contract is enforced
  toolchain-free.** `multisigEngraveCardsMulti` (`gui/multisig_engrave.go:47-72`) appends
  every `ms1`, then every `mk1`, then the `md1`, from ONE emitter shared by both tails —
  there is no second emitter to drift. `oracle.ArtifactKindsFor`
  (`oracle/expect.go:145-156`) declares `built-policy-full = [ms1 mk1 md1]` /
  `built-policy-watch = [mk1 md1]`; `oracle.CheckArtifactShape` (`:169-227`) requires
  non-empty consecutive runs in that order, nothing outside them, and (I-2) that each
  artifact's string carries its own kind as a prefix. `oracle.CompareCensus`
  (`oracle/expect.go:851-879`) is index-by-index and byte-exact, and
  `TestEveryGateRecordCensusMatchesItsCommittedExpectation`
  (`oracle/expect_test.go:112-125`) runs it over every record on disk with no oracle
  binary and no skip path. Measured from `S5-trace-b.expect.json`: 17 artifacts, indices
  0-1 `ms1`, 2-8 `mk1`, 9-16 `md1` — 2/7/8, consecutive, in order; the walk census
  (`S5-trace-b.record.json`) holds the same 17 strings in the same order, and 17 plate
  digests. Watch-only drops the `ms1` run and still matches its kind's declaration.
* **The build tail needs no mk1 dedupe, and the guard it relies on is pinned.**
  `duplicateSlotPair` (`gui/multisig_build.go:1024-1035`) compares
  chainCode‖compressedPubkey and IGNORES origin, so it refuses ANY two slots holding the
  same key whether or not the origins agree — which is strictly stronger than "no two
  identical mk1s". It runs inside `assembleBuildPolicy` (`:1278`), and
  `buildEngraveTail`'s only production call site is `gui/multisig_build.go:351`, after it.
  (`grep -rn 'buildEngraveTail' --include='*.go' | grep -v _test` → one call site.) The
  sibling campaign's `M11-dup-slot-check-off` is killed by 9 tests.
  I confirmed the tail has no independent defence: feeding it two `both` sources pointing
  at one card yields two byte-identical `mk1` cards labelled "1 of 2"/"2 of 2" — but
  `buildSlotSources` assigns each non-derived slot a distinct card index, so that state
  cannot be built by the flow, and any two cards carrying one key die at `duplicateSlotPair`.
* **The ms1 dedupe is fail-safe in both directions and is pinned.** Keyed on the engraved
  `ms1` STRING (`gui/multisig_build_tail.go:85,117-126`): two equal strings encode
  identical entropy, so it can never drop a distinct seed's only plate; and it fires for
  the shape the flow actually builds (one registry entry per held slot, so `SeedID` would
  not work). Mutation `if false && engraved[b.MS1]` → **exit 1**, killed by
  `TestFullModeEngravesMs1ForEveryMaster` ("engraved 3 ms1 plate(s), want 2") and
  `TestVerifyCoversEveryMastersSecret`.
* **The supply tail's mk1 dedupe and its obligation collapse are pinned together.** Moving
  `slots = append(slots, s)` ABOVE the `cut[mk1Key]` check (so the obligation names a slot
  no plate exists for) → **exit 1**, killed by `TestSupplyTailCollapsesByteIdenticalPlates`
  ("obligation list is [0 1], want [0]"), `TestSupplyDuplicateSlotVerifiesItsOwnOutput`
  ("Read back 1 key plate, but this run engraved 2"), and
  `TestSupplyFlowAnnouncesTheCollapseBeforeTheFirstCut`. The join key
  `strings.Join(b.MK1, "|")` is injective (`|` is not in the bech32 charset).
* **The announcement precedes the first cut on the supply path, and its predicate is the
  tail's own return.** `gui/multisig.go:250-263` prepends the collapse NOTE when
  `len(engravedSlots) < len(slots)`, first line of a screen that is confirmable from any
  page, before `bundleEngrave`. The pre-mode notice (`gui/multisig.go:185-199`) states no
  count, correctly — and `multisigSlotsShareAKey` cannot disagree with the tail, because
  `allUserSlots` (`gui/multisig_match.go:76-95`) matches all 65 bytes of
  `ExpandedKey.Xpub`, so two MATCHED slots at one origin necessarily carry one xpub.
* **The count shown equals the count cut, structurally.**
  `buildPlateCensusLines`/`buildPlateInventoryLines` both call `bundlePlatePlan`
  (`gui/bundle_flow.go:348-363`), the same function `bundleEngrave` loops
  (`gui/bundle_flow.go:382`). Mutating the total to `len(cards)` does not even COMPILE
  (`declared and not used: plan`) — the drift is closed by construction, not by a test.
  `numberedLabel` is pinned: `i+1` → `i` is killed by
  `TestSupplyFlowEngravesAPlatePerMatchedSlot`.
* **sh(wsh) → `1'` is template-aware end-to-end.** `derivedSlotOrigin` +
  `multisigScriptTypeComponent` (`gui/multisig_build_slots.go:103-122`); measured, an
  `md.MultisigShWsh` build declares `m/48h/0h/0h/1h` at the held slot in the assembled
  policy AND engraves `mk1 path=m/48h/0h/0h/1h`. The cosigner slot correctly keeps its
  card's `…/2h` (divergent origins).
* **One master at two accounts is right.** Measured: 2 held slots on master A →
  `slots=[0 1]`, policy origins `m/48h/0h/0h/2h` and `m/48h/0h/1h/2h`, plates
  `[ms1 secret share] [mk1 key 1 of 2] [mk1 key 2 of 2] [md1 descriptor]`, one seed plate,
  census "This engraves 12 plates".
* **A seed that fills zero slots is refused on both paths.** Build:
  `errBuildNoHeldSlot` — "multisig build: no slot is held by this device"
  (`gui/multisig_build_tail.go:131-133`), measured. Supply: refused one screen earlier at
  `len(slots) == 0` (`gui/multisig.go:163-166`), so `errSupplyNoMatchedSlot` is a backstop.
* **Secret-first ordering is handled at the abort.** Full mode cuts the `ms1` first;
  `bundleAbortWarningText` (`gui/bundle_flow.go:488-498`) says DESTROY-not-bin only when
  `bundleSetCarriesASecret`, and tells the operator the re-run is byte-identical so only
  missing plates need cutting. Correct for multi-`ms1` sets.

## Commands run

```
# on the frozen worktree (read-only)
git -C /scratch/code/shibboleth/wt-s5 log --oneline main..s5-multislot
git -C /scratch/code/shibboleth/wt-s5 diff --stat HEAD -- <the 7 files above>   # empty
python3 -c "...json.load('oracle/gaterecords/S5-trace-b.expect.json')..."       # 17 artifacts, 2/7/8
python3 -c "...json.load('oracle/gaterecords/S5-trace-b.record.json')..."       # 17 census strings, 17 digests

# on a private copy: git archive 7da66bd | tar -x -C <scratch>/tail-lens/tree
export PATH="/nix/var/nix/profiles/default/bin:$PATH"
nix develop --command go test ./gui/ ./oracle/ -count=1        # BASELINE exit 0
# per mutation: apply, run the same command, revert
#   T1 numberedLabel i+1 -> i                    KILLED (1 test)
#   T3 census counts cards not plates            KILLED (does not compile)
#   T5 supply obligation appended before dedupe  KILLED (3 tests)
#   T6 both-slot origin -> derivedSlotOrigin     SURVIVED (exit 0)   <- Finding 1
#   (build-tail ms1 dedupe disabled              KILLED, 2 tests, on an earlier copy)
nix develop --command go test ./gui/ -run 'TestZZ...' -v -count=1   # the 3 probes quoted above
```

Probe sources and mutation logs: `<scratch>/tail-lens/` (`mut.py`, `out-T*.txt`,
`tree/gui/zz*_test.go`). Nothing was written to `/scratch/code/shibboleth/wt-s5`.
