# S5 whole-diff review — LENS: FIXTURE REALISM

Reviewer: independent agent (opus), read-only on `/scratch/code/shibboleth/wt-s5` @ `7da66bd`.
Diff under review: `main..s5-multislot` (10 commits, 57 files, +8873/-607).
Date: 2026-08-16.

**The one question:** is any guarding test built on a fixture the production flow could never
construct?

**Answer: yes — one, and it is Critical #2's shape at a second site.** The consequence is that
a SPEC 4.3 row-5 operator announcement is measurably dead in the shipped flow, and 51 green
packages plus the Trace B emulator walk did not notice.

---

## Method and scope

For each of the seven new S5 test files I traced every hand-built fixture back to the
production constructor that would have to produce it, field by field.

**What I found NOT to be a problem, stated so it is not re-derived.** The seven new files are,
with one exception, built *through* production code rather than around it, and the S5 authors
were plainly reacting to Critical #1/#2:

| fixture | built by | verdict |
| --- | --- | --- |
| `s5TraceB` (`multisig_build_s5_test.go:64`) | `s5Registry` (one entry **per held slot**) → `buildSlotSources` → `buildSelfKeys` | realistic; this is the repaired fixture |
| `s5TraceBEngraved` (`multisig_verify_legs_test.go:32`) | `assembleBuildPolicy` + `buildEngraveTail` | realistic |
| `s5OneSlotReadback` (`multisig_verify_flow_test.go:44`) | as above, expectation `{@0}` | realistic — reachable as a build with `SelfSlots={0}` that picks payload card A@1 for @1 (`duplicateSlotPair` refuses only *identical* keys, so A@0/A@1 is admitted) |
| `s5PolicyPair` (`multisig_verify_policy_test.go:39`) | `assembleBuildPolicy` twice + `buildEngraveTail` | realistic |
| `s5DuplicateSlotMd1` (`multisig_supply_dupslot_test.go:53`) | `md.EncodeMultisig` direct | realistic **and correctly justified**: the BUILD path refuses it (`duplicateSlotPair`), the SUPPLY path admits a foreign coordinator's policy by design |
| `s5PickSlots` / `TestSelfSlotSetReachesParams` | drives `multisigSelfSlotPickFlow` / `buildParamPickFlow` | realistic |
| supply drives (`s5DriveSupply`, `s5DriveVerify`, `s5DriveVerifyTwoSeeds`) | real `runUI` + real gatherer via `ctx.syswBundleSeeds` | realistic |

Struct-literal fixtures that set fields production sets differently but that **nothing reads**
(`slotFromSeed` sources with `Card: 0` where production sets `-1`; `slotFromCard` sources with
`SeedID: 0` where production sets `-1`) were checked against every consumer
(`buildSlotGate`, `buildEngraveTail`, `buildSelfKeys`) and are inert. Not reported.

Already filed (F-189 … F-195): seen, not re-reported.

---

## FINDING 1 — IMPORTANT

### `buildSlotGate`'s multi-account notice is keyed on `SeedID`, so it never fires in the shipped flow; its only guard is a fixture that shares a `SeedID` across two held slots — a registry the flow cannot build

**File:** `gui/multisig_build_slots.go:377`, `:388-392`, `:425-432` (production)
**Guarding fixture:** `gui/multisig_build_gate_test.go:210-214`
(`TestGateAcceptsSameSeedAtDistinctOrigins`)

#### The fixture

```go
sources := []slotSource{
    {Kind: slotFromBoth, SeedID: 0, Card: 0},    // origin m/48h/0h/0h/2h
    {Kind: slotFromSeed, SeedID: 0, Account: 1}, // origin m/48h/0h/1h/2h
    {Kind: slotFromCard, Card: 0},               // somebody else
}
```

Two held slots share **`SeedID: 0`**.

#### Why the flow cannot produce it

`buildMultisigPolicyFlow` registers **one registry entry per held slot**:

- `gui/multisig_build.go:195-202` — `for _, slot := range p.SelfSlots { id, ok := buildSeedForSlot(...); seedIDs = append(seedIDs, id) }`
- `gui/multisig_build.go:554` — `buildSeedForSlot` calls `reg.add(...)` **unconditionally**, so
  typing the *same words* twice yields two entries and two different ids.
- `gui/multisig_build.go:442-449` — `buildSlotSources` sets `SeedID: seedIDs[hi]`, `hi` being the
  held-slot ordinal, so two held slots are **distinct by construction**.

This is the identical fact `buildEngraveTail`'s own comment records at
`gui/multisig_build_tail.go:64-70` as the cause of Critical #1 ("a SeedID-keyed dedupe therefore
never fires for the shape the product actually builds"). The fix was applied to the tail and
**not swept into `buildSlotGate`, which has the same keying.**

#### The consequence

`buildSlotGate` groups its bindings in `bound[s.SeedID]` and emits SPEC 4.3 row 5's notice only
when `len(bs) >= 2` (`gui/multisig_build_slots.go:442-472`). Under the flow's real shape every
group has exactly one binding, so the notice is **never emitted**.

The notice is the *only* surface anywhere in the flow that tells the operator two of their slots
come from one master. The Key-sources review labels them "your seed for @0" / "your seed for @1"
(`buildSlotSourceLines`, `gui/multisig_build_slots.go:600-612`), which reads as two different
seeds. So an operator who types one master for two held slots of a 3-of-4 — deliberately, or by
mistake — is told nothing, and their wallet needs two independent parties where its k-of-n
implies three.

#### Concrete failing state, and how I verified it

Input: Trace B exactly as the plan specifies it and `walk_trace_b.js` drives it — n=4, k=3,
`SelfSlots={0,1,2}`, master A typed for @0 and @1, master B for @2, card C@0 at @3.

**(a) Measured against the frozen tree's *own committed walk record*** — the real flow, on the
emulator, cutting all 17 plates. `oracle/gaterecords/S5-trace-b.walk.json`, `keySourcesScreen`:

```
KeysourcesWhereeachkeycomesfrom:@0yours:derivedfromyourseedfor@0
@1yours:derivedfromyourseedfor@1,account1@2yours:derivedfromyourseedfor@2
@3acosigner:payloadcard4,takenassuppliedNoslotclaimstobebothaseedandacardhere,
sonothingwascross-checked.Thecosignerkeysaretakenassupplied.
```

No notice. `buildSlotSourceLines` appends `notices...` immediately before that closing sentence
(`gui/multisig_build_slots.go:617`), so if any had been produced they would be on this screen.

**(b) Executed.** I copied the frozen tree to scratch (the frozen tree was not written to) and
ran a probe calling the production `buildSlotGate` over `s5TraceB`'s production-built sources:

```
source @0 kind=1 seedID=0 account=0 card=-1
source @1 kind=1 seedID=1 account=1 card=-1      <- SAME master, DIFFERENT SeedID
source @2 kind=1 seedID=2 account=0 card=-1
source @3 kind=0 seedID=-1 account=0 card=0
PROBE notices for the REAL Trace B shape: 0 []
PROBE notices for the SHARED-SeedID (unreachable) shape: 1
  ["Slots @0 and @1 all come from your seed, at different key origins.
    That is a multi-account wallet and is allowed."]
```

**(c) Mutation — the mechanism is unpinned.** Replacing the entire notice loop with
`return nil, nil` and running `go test ./gui/ -count=1` failed **exactly one assertion**:

```
multisig_build_gate_test.go:222: got 0 notice(s) [], want exactly 1:
  proceeding SILENTLY on a shape the spec calls out is half the requirement
```

i.e. the only thing standing behind the mechanism is the impossible fixture. That is the
headline process lesson of this cycle verbatim: *an unpinned fix is indistinguishable from an
inert one* — here, an unpinned **mechanism** is indistinguishable from an inert one.

**(d) The fix is small and safe, and I proved it.** Re-keying `bound`/`order` on the registered
seed's `MasterFP` (which `seedRegistry.add` already captures,
`gui/multisig_build_slots.go:169-180`) instead of on `SeedID`:

```
PROBE notices for the REAL Trace B shape: 1 ["Slots @0 and @1 all come from your seed, ...]
```

and the **whole `gui` suite stays green** (`ok seedhammer.com/gui 85.021s`), including the
existing `TestGateAcceptsSameSeedAtDistinctOrigins`. Nothing else depends on the `SeedID`
keying.

#### What the repair owes

Fixing the production keying alone leaves the fixture lying. The fixture must be rebuilt through
`buildSlotSources` over a registry holding **one entry per held slot** (the pattern
`s5Registry`/`s5TraceB` already establish), or the same class lands here a third time. A test
asserting `len(notices) == 1` on Trace B's *production* sources is the assertion that could not
have passed before this fix.

---

## FINDING 2 — IMPORTANT

### S5 fixed "Full (seed + keys)" lying about the passphrase on the BUILD path only; the SUPPLY path, which S5 rewrote, still labels a passphrase build "Full (seed + keys)" and its restore doc never mentions a passphrase at all

**Files:** `gui/multisig.go:204` (mode label), `gui/multisig.go:302` (restore doc), against
`gui/multisig_build_census.go:124-145` (`buildFullModeLabel`) and `:83-121`
(`buildPassphraseInventoryLines`).

S5 states the rule itself, at `gui/multisig_build_census.go:126-128`:

> "Full (seed + keys)" is correct for a build with no passphrase and is a LIE for one with a
> passphrase

and at `gui/multisig_build_slots.go:219`:

> A set labelled "Full (seed + keys)" that silently omits a spending factor is F-132's shape: a
> backup that is both wrong and trusted.

`supplyMultisigPolicyFlow` accepts a BIP-39 passphrase (`gui/multisig.go:147`,
`syswPassphraseFlow`) and threads it into `supplyEngraveTail`, which engraves an ms1 encoding the
**words only**. It then:

- offers the mode picker with the hardcoded literal `"Full (seed + keys)"` (`gui/multisig.go:204`)
  — measured: `buildFullModeLabel` has exactly one caller, `gui/multisig_build.go:340`;
- shows the restore document with `extra == nil` (`gui/multisig.go:302`), and
  `multisigRestoreDocFlow` appends only `extra` (`gui/multisig_restore.go:93-100`) — measured:
  `grep -n passphrase gui/multisig_restore.go` returns **nothing**.

`buildPlateInventoryLines` / `buildPassphraseInventoryLines` likewise have exactly one caller
(`gui/multisig_build.go:415`).

**Concrete failing scenario.** Operator supplies a coordinator's md1, types their seed, adds a
BIP-39 passphrase, chooses row 0 labelled "Full (seed + keys)". The device cuts an ms1 + one mk1
per matched slot + the md1 and prints a restore document. Every plate is correct. Nothing on any
screen or on the kept document states that a required spending factor is absent from the set. In
five years the person holding that steel has a "full" backup that does not reach the money — and,
per `buildPassphraseInventoryLines`' own reasoning, cannot distinguish that from a complete one.

**Why it is in scope for this diff rather than "pre-existing".** Before S5 neither path said it;
S5 *created* the asymmetry, in the same stage that rewrote this exact function's engrave tail,
its census, and its verify hand-off. The two helpers it needs already exist and are pure. It is a
two-line change (`buildFullModeLabel(passphrase != "")` and passing
`buildPassphraseInventoryLines(passphrase != "")` — or the full
`buildPlateInventoryLines(cardsOut, ...)` — into `multisigRestoreDocFlow`).

Adjacent, **not** reported as a separate finding because it is outside this diff: `gui/singlesig.go:80`
carries the same hardcoded literal. Worth a follow-up with an owning phase.

---

## FINDING 3 — MINOR

### Two production comments assert the @S picker is single-select; it has been multi-select since commit `4b10319`, and one of them licenses exactly the fixture practice that caused this cycle's Critical #2

**Files:** `gui/multisig_build_slots.go:40-44`, `gui/multisig_build.go:46`.

`gui/multisig_build_slots.go:40`:

> What has NOT moved is the @S PICKER, which is still single-select, so a build driven through
> the screens produces exactly one held slot and **the multi-slot shapes are exercised by tests
> driving the model directly.** The multi-select screen … belongs to the block after this one.

False on the frozen tree: `multisigSelfSlotPickFlow` (`gui/multisig_build.go:808-855`) is the
multi-select picker, `buildParamPickFlow` calls it at `:948`, `TestSelfSlotPickerTakesTraceB`
picks {0,1,2}, and `cmd/emu/walk_trace_b.js:517-534` drives it on the machine.

`gui/multisig_build.go:46`: "The @S picker always sets exactly one held slot, so this cannot fire
today." The guard's *rationale* still holds (the first pick is mandatory, so the set is never
empty — which `buildParamPickFlow` states correctly at `:942-946`), but the stated reason is now
wrong.

The first one matters more than a typo: it tells the next author that hand-built model-level
fixtures are the sanctioned way to cover multi-slot shapes. That is the practice that produced
Finding 1 and Critical #2. ("Comments outlive their conditions.")

---

## What this lens did NOT reach

Stated so the next reviewer's budget is not spent here and is not falsely reassured:

- I did not audit the oracle gate-record derivation (`S5-trace-b.expect.json` args vs. what the
  device encodes) — that is an oracle-provenance lens, not this one. I did confirm the record's
  census holds 2 ms1 + 7 mk1 + 8 md1 = 17 strings, matching the claim.
- I did not attempt a mutation sweep over the verify comparator; I mutated only the one mechanism
  Finding 1 concerns.
- **Multi-slot `both` (`SelfFromCard == true` with `len(SelfSlots) > 1`) has no test at all**, in
  Go or in any walk. I traced it by hand through `buildSlotSources` → `buildSlotGate` →
  `buildSelfKeys` (empty) → `assembleBuildPolicy` (`len(cosigners) == p.N`) → `buildEngraveTail`
  and found no defect, but that is reasoning, not execution. Worth a follow-up; the shape is
  reachable through the screens today (`buildSelfSourceFlow` explicitly grows a plural
  "YES, CHECK THE CARDS" row for it, `gui/multisig_build_slots.go:519-540`).
