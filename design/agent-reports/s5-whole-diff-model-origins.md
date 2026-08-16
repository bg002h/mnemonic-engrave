# S5 whole-diff review — LENS: THE MODEL (SelfSlots, divergent origins, the REMOVED refusal)

Artifact: `/scratch/code/shibboleth/wt-s5`, frozen at `7da66bd`, `git diff main..s5-multislot`.
Tree verified untouched at exit: `git status --porcelain` empty, `git rev-parse --short HEAD` = `7da66bd`.

Method note. Every finding below was reproduced by RUNNING code, not by reading it. Because the
tree is read-only I took a byte copy to
`/tmp/claude-1000/.../scratchpad/wt5copy` (`cp -a`, 18M) and ran probe tests there under
`nix develop /scratch/code/shibboleth/wt-s5 --command go test ./gui/ -run TestProbe... -v -count=1`,
so the toolchain and the flake are the frozen tree's own. Nothing was written to `wt-s5`.

---

## The lens question, answered first

**Did widening `SelfSlot -> SelfSlots` and removing S2's interim foreign-origin refusal open a hole?**

The removal itself is sound. I established the exact refused→accepted delta and it is safe:

* The removed check was `originIsShared(cosigners[gi].Path, multisigSharedOrigin())`, run in
  `assembleBuildPolicy` over every gathered card (old `gui/multisig_build.go:790-805`, now deleted).
  It refused any card not declaring `m/48'/0'/0'/2'`.
* What is now accepted is exactly: **a cosigner card declaring any parseable, non-empty,
  ≤15-component BIP-32 path**. I confirmed each remaining gate by reading the code it lands in:
  `bip32.ParsePath` (`bip32/bip32.go:86`) rejects unparseable; `md.EncodeMultisig`
  (`md/encode_multisig.go:104-110`) rejects a zero-component origin in divergent mode and
  `md/encode.go:90-93` rejects >15 components with `errPathDepth`. So there is **no silent
  truncation path**: the 4-bit depth field is guarded on the write side before it is written.
* The card's origin was always operator-supplied and always went onto steel; pre-S5 it was simply
  constrained to one value. The recorded origin is still the card's own declaration, so the
  removal widens the accepted set without lowering the trust level of any recorded field.

**Ordering claim — verified in code, and it half-holds.** `duplicateSlotPair` (`gui/multisig_build.go:1023`)
runs at `gui/multisig_build.go:1278` *before* `commonOrigin`/`md.EncodeMultisig` and before
`errBuildEmptyOrigin`, so §4.1 does still outrank the empty-origin refusal. But `cosignerFromCard`
now calls `bip32.ParsePath` inside the slot-fill loop, which runs *before* the duplicate check — so
for an unparseable card path the ruling is silently reversed. Filed as M1 below, with a RUN.

**`OriginDivergent` from ALL origins?** Yes. `commonOrigin` (`gui/multisig_build.go:1348`) compares
`all[0].Origin` against every element of `all[1:]`, by parsed component, not by string. Correct.

**`derivedSlotOrigin` template coverage.** `md.MultisigScript` has exactly three values
(`md/encode_multisig.go:27-29`). `multisigScriptTypeComponent` covers all three: `MultisigShWsh -> 1`,
`MultisigWsh -> 2`, `MultisigSh -> 2` (default arm). No unhandled template. The `script` argument is
threaded from `p.Script` into every one of the four call sites (`buildSlotGate`, `buildSelfKeys`,
`buildEngraveTail`, `buildOriginAnnouncement`) — no site retains a script-blind copy.

**Single-slot case after the widening.** Correct: one held slot always gets account 0 from
`nextAccount`, so `derivedSlotOrigin(MultisigWsh, 0)` == `multisigSharedOrigin()` and `commonOrigin`
still selects `OriginShared` when the cards agree, preserving S2's byte-identity golden.

**The hole is not in the removal.** It is in the widening's *bookkeeping identity* — the same
SeedID-vs-secret-identity confusion this cycle already found once, at a second site — and in three
surfaces that still describe a single-slot world. Those are C1, I1, I2, I3 below.

---

## C1 (CRITICAL) — SPEC 4.3 row 5's multi-account notice is DEAD in the real flow, and the review screen labels ONE seed as TWO

**File:** `gui/multisig_build_slots.go:377` (`bound := map[int][]binding{}`, keyed on `s.SeedID`),
with `gui/multisig_build.go:195-201` and `gui/multisig_build.go:554`.

**The defect.** `buildSlotGate` groups a seed's slot bindings by `s.SeedID` and emits the row-5
notice only when one SeedID holds ≥2 bindings at distinct origins. But `buildMultisigPolicyFlow`
calls `buildSeedForSlot` **once per held slot** (`gui/multisig_build.go:196-201`) and
`buildSeedForSlot` calls `reg.add(...)` **unconditionally** (`gui/multisig_build.go:554`), so an
operator typing the SAME words for two held slots gets **two distinct SeedIDs**. Every held slot
therefore has a unique SeedID, `len(bs)` is always 1, and the notice can never fire.

This is the exact class of this cycle's Critical #1 (a dedupe keyed on `SeedID` while the flow
registers one entry per held slot) at a *different site*. `buildEngraveTail` was fixed by re-keying
on the ms1 string, and `buildSlotSources` was written keyed on `MasterFP` precisely because the
authors knew SeedIDs differ per slot (`gui/multisig_build.go:430-437` says so in words) — but the
gate's notice grouping was not re-keyed.

It is also this cycle's Critical #2: the only test guarding the notice,
`TestGateAcceptsSameSeedAtDistinctOrigins` (`gui/multisig_build_gate_test.go:207-229`), constructs
`{Kind: slotFromBoth, SeedID: 0}` and `{Kind: slotFromSeed, SeedID: 0, Account: 1}` — **a registry
the flow cannot produce**. The Trace B walk does not catch it either: `cmd/emu/walk_trace_b.js:616`
asserts the *account* line, never the notice.

**Verified — RUN.** Probe reproducing the registry the flow actually builds:

```
registry: 2 entries, seedIDs=[0 1], masterFPs=73c5da0a/73c5da0a
source @0: kind=1 seedID=0 account=0 card=-1
source @1: kind=1 seedID=1 account=1 card=-1
source @2: kind=0 seedID=-1 account=0 card=0
buildSlotGate -> notices=[] err=<nil>
=== Key sources review, as the operator reads it ===
  Where each key comes from:
  @0  yours: derived from your seed for @0
  @1  yours: derived from your seed for @1, account 1
  @2  a cosigner: payload card 1, taken as supplied
  No slot claims to be both a seed and a card here, so nothing was cross-checked. ...
```

Both entries carry the **same master fingerprint `73c5da0a`** — the device has computed that they
are one secret — and the screen the operator reads before assembly says **"your seed for @0"** and
**"your seed for @1"**, which reads as two different seeds.

**Concrete failure scenario (funds).** A 2-of-3 built on this device: the operator holds @0 and @1
and types one seed for both; @2 is a third party's cosigner card. The tail correctly engraves ONE
ms1 and TWO mk1s. Nothing on any screen states that @0 and @1 stand on a single secret. The plate
census (RUN) says only:

```
This engraves 12 plates.
ms1 secret share: 1 plate (secret seed backup)
mk1 key 1 of 2: 2 plates (account key card)
mk1 key 2 of 2: 3 plates (account key card)
md1 descriptor: 6 plates (wallet policy descriptor)
```

Years later the ms1 plate is lost. The operator, believing they hold two independent keys, expects
one of theirs plus the cosigner to reach 2-of-3. In truth both of their keys are gone (mk1 plates
are public keys and cannot sign), one key remains, and **the wallet is permanently unspendable**.
The announcement designed to prevent exactly this — "Slots @0 and @1 all come from …, at different
key origins. That is a multi-account wallet and is allowed." (`gui/multisig_build_slots.go:468-470`)
— is unreachable production code.

**Asymmetry worth noting:** the SUPPLY path, which merely *discovers* this shape, has a loud live
notice (`gui/multisig.go:180-196`). The BUILD path, which *creates* it, is silent.

**Fix direction (not prescriptive):** key `bound` on the registered pair's `MasterFP` — the same key
`buildSlotSources`' `accounts` map already uses — and re-point the guarding test at a registry built
the way `buildSeedForSlot` builds one.

---

## I1 (IMPORTANT) — holding EVERY slot dead-ends on a self-contradictory refusal

**File:** `gui/multisig_build.go:91` (`open := p.N - len(p.SelfSlots)`) into
`gui/multisig_build_payload.go:204-213`.

**The defect.** S5's multi-select picker (`multisigSelfSlotPickFlow`, `gui/multisig_build.go:508`)
lets the operator hold all `n` slots — the single-person multisig this stage exists for. Then
`open == 0`: no cosigner cards are needed at all. But `classifyCosignerSupply` returns
`cosignerRefuse` whenever `state != cosignerSourceLoaded`, *regardless of `open`*, and
`buildSupplyRefusal` renders the no-payload row. Pre-S5 this was unreachable because `open` was
always `p.N - 1 >= 1`.

The comment at `gui/multisig_build.go:46` — "The @S picker always sets exactly one held slot, so
this cannot fire today" — is the stale premise that hid it; it was true at S5.A/B and false from
S5.C onward.

**Verified — RUN (whole flow, through the screens).** 2-of-2, hold @0 then "YES, ONE MORE" (@1 is
auto-added as the single remaining slot), no payload:

```
--- Template ---   Choosepolicytypewsh(nativesegwit)sh(wsh)...
--- Cosigners ---  Howmanykeys(n)?2345
--- Threshold ---  Requiredsignatures(kof2)?12
=== SCREEN AFTER THE PICKERS ===
Nopayloadisloaded,andthispolicyneedsnocosignerkeycards.Thisdevicehasnocardreader:
packthecardsonthehostwith`mesyswpack`,loadthepayload,thenbuild.
flow returned: false
```

And the classification table at `open == 0`:

```
state=loaded(0)      have=0 open=0 -> autoFill
state=noPayload(1)   have=0 open=0 -> REFUSE: "No payload is loaded, and this policy needs no cosigner key cards. ..."
state=uncompared(2)  have=0 open=0 -> REFUSE: "The loaded payload has not been checked yet. ..."
```

**Concrete failure scenario.** An operator building a 2-of-2 they hold entirely, entering both seeds
on the keyboard (no payload is needed for anything — `seedEntryFlow` offers the keyboard), is
refused with a message that simultaneously says the policy needs **no** cosigner key cards and that
they must go load a payload of cosigner key cards on a host. There is no forward route on the screen;
the only workaround is to load an unrelated compared payload so `state` becomes `cosignerSourceLoaded`,
which nothing tells them. Trace B does not catch it: Trace B is 3 held of **n=4**, so `open == 1`.

---

## I2 (IMPORTANT) — the §0.1a origin announcement states ONE origin for a build that derived at several, on the last review before the engrave

**File:** `gui/multisig_build.go:1520` (`base := derivedSlotOrigin(script, 0)`), rendered into
`buildReviewLines` at `gui/multisig_build.go:1043`.

**The defect.** `buildOriginAnnouncement` hardcodes **account 0**. On a multi-slot build the device
derives the operator's keys at accounts 0, 1, … , so the sentence is false for every held slot past
the first. `buildReviewLines` prints each slot's key in full but prints **no per-slot origin**, so
the review screen contains no correct statement of where @1's key lives.

**Verified — RUN, and then confirmed against the repo's own minted gate record.** Probe over two
held slots from one master (accounts 0 and 1):

```
assembled md1 slot @0 origin = m/48h/0h/0h/2h
assembled md1 slot @1 origin = m/48h/0h/1h/2h     <-- what the device actually stamped
assembled md1 slot @2 origin = m/48h/0h/0h/2h
=== buildOriginAnnouncement ===
  Your key origins: m/48h/0h/0h/2h, the BIP-48 path for native segwit.
```

The same wrong sentence is in the S5.D gate record for the flagship shape. Extracted verbatim from
`oracle/gaterecords/S5-trace-b.walk.json` (whitespace squashed as the record stores it):

```
...Yourkeyorigins:m/48h/0h/0h/2h,theBIP-48pathfornativesegwit.Policystub:70fb9d6eSlots:@0,nofingerprint:PolicyReview...
```

while the same record's own oracle command lines carry
`--origin-path m/48'/0'/0'/2'` **and** `--origin-path m/48'/0'/1'/2'`
(`oracle/gaterecords/S5-trace-b.expect.json`). The walk taps straight past this screen
(`cmd/emu/walk_trace_b.js:357  // past the Policy Review`) and asserts nothing on it, so the wrong
announcement is minted into a green gate record.

**Concrete failure scenario.** The screen immediately after this one instructs the operator to
"compare the keys you just reviewed … against the same wallet in your coordinator". An operator who
takes the announced origin at face value enters `m/48'/0'/0'/2'` for both of their keys in the
coordinator, derives the wrong key for @1, and concludes the *device* produced a wrong key — or, in
the other direction, mis-registers the wallet in the coordinator at a path the plate does not carry.
§0.1a's stated requirement is that the device "says WHICH derivation path it stamped on **every**
slot"; for the shape S5 exists to build, it does not.

(The account *number* is disclosed one screen earlier on "Key sources" — "@1 yours: derived from
your seed for @1, account 1" — so the information exists in the flow. The defect is that the Policy
Review's origin sentence is affirmatively wrong, not that the fact is unobtainable.)

---

## I3 (IMPORTANT) — the registry's own no-idle-limit justification is now false, and S5 is the phase it was scheduled to

**File:** `gui/multisig_build_census.go:58-63`.

**The defect.** The comment that justifies having no wipe bound on the seed registry reads, verbatim:

> "The registry today holds exactly one seed, which is what the shipped flow already held, so an
> idle limit would buy no reduction in exposure over the state of the tree. **S5 multiplies the
> masters in it; the bound is filed to be re-decided there, when it would actually change something.**"

S5 is this diff, and it does exactly what the comment predicted: `buildSeedForSlot` now runs once
per held slot, so the registry holds up to `n` live `bip39.Mnemonic` buffers (Trace B: three entries
across two masters), and the build it brackets grew from a few plates to twelve. The premise
("holds exactly one seed") is false as of `7910e00`, the conclusion drawn from it ("would buy no
reduction in exposure") no longer follows, and the scheduled re-decision does not appear anywhere in
`main..s5-multislot` — `buildPlateInventoryLines` still emits the unchanged non-wiping ruling.

**Verified — RUN.** The same probe as C1 shows the registry holding two live entries
(`registry: 2 entries, seedIDs=[0 1]`), and the restore-doc inventory it feeds still says:

```
Seed handling: this build does not time out. A seed you entered stays in device memory until the
build ends, like the rest of the payload surface. Power the device off when you are done.
```

**Concrete failure scenario.** An operator part-way through a 12-plate Trace B build walks away
between plates. Where the shipped flow left one seed live, this build leaves **two masters' word
lists** live in RAM for the hours the engrave takes, on a device whose only mitigation is the
operator remembering to power it off. That is a security-posture change the phase owns and the
project's own rule ("an item scheduled *to* a phase is not deferrable past its owning phase") makes
non-deferrable at this gate. Either the bound gets re-decided or the comment's premise gets
corrected and the deferral re-scheduled explicitly — but it cannot silently carry a stale
justification across the merge.

---

## M1 (MINOR) — S2's "duplicate outranks the origin refusal" ruling is reversed for an unparseable card path

**File:** `gui/multisig_build.go:1166-1169` (`cosignerFromCard` now calls `bip32.ParsePath`), inside
the fill loop at `gui/multisig_build.go:1256`, which precedes `duplicateSlotPair` at
`gui/multisig_build.go:1278`.

S2 ruled the ordering explicitly and said so: "It is also the check that OUTLIVES this stage: the
origin refusal disappears at S5, §4.1's never does. Asserted, not left to the reading order."
S5 moved origin *parsing* into the fill loop, upstream of the duplicate check.

**Verified — RUN:**

```
duplicate + unparseable-origin -> *errors.errorString  bip32: invalid path element: "notanumber"
duplicate only                 -> gui.errBuildDuplicateKey  multisig build: slots @0 and @1 hold the same key
```

The first case reaches the operator as the generic "Couldn't assemble the wallet policy." (device
failure) rather than the named "Duplicate key" modal, so a payload that is *both* a duplicate set and
carries a malformed path sends the operator to fix the wrong input. Nothing is engraved on either
arm, which is why this is Minor and not Important. Neither existing test covers the combination:
`TestBuildRecordsTheCardsOwnOrigin/"a duplicate outranks anything the encoder would say"` uses two
parseable cards, and `.../"an unreadable declared origin is refused"` uses a non-duplicate set.

---

## M2 (MINOR) — the restore doc's descriptor carries no key origins, which S5 made load-bearing

**File:** `gui/multisig_restore.go:59-62` (untouched by this diff — `git diff --stat main..s5-multislot -- gui/multisig_restore.go` is empty).

**Verified — RUN.** For the two-held-slots-one-master build, the restore doc renders:

```
Descriptor:
wsh(sortedmulti(2,xpub6DXuQW1Q2JpZwAoNcK.../<0;1>/*,xpub6DXuQW1Q2JpZwtNiGZ.../<0;1>/*,xpub6DXuQW1Q2JpZxfSWqC.../<0;1>/*))#pe4dfssa
```

No `[fingerprint/48h/0h/1h/2h]` prefixes anywhere. Pre-S5 every slot shared one announced origin and
the omission cost little; post-S5 the same document describes a policy whose slots sit at genuinely
different paths, including two accounts of one master, and states none of them. The origins are
still on steel — the md1 carries them per slot (proved above by `ExpandWalletPolicyChunks`) and each
mk1 carries its own — so this is recoverable and therefore Minor, not a backup-loss finding. Flagged
because the restore doc is explicitly the artifact "read years later, alone, often by someone who was
not the operator", and it is now the only kept surface that has *lost* information relative to the
plates it describes.

---

## N1 (NIT) — three comments still assert a single-select picker

* `gui/multisig_build_slots.go:40-44` — "What has NOT moved is the @S PICKER, which is still
  single-select, so a build driven through the screens produces exactly one held slot and the
  multi-slot shapes are exercised by tests driving the model directly." False since `4b10319`.
* `gui/multisig_build.go:46` — "The @S picker always sets exactly one held slot, so this cannot fire
  today." False since `4b10319`, and it is the premise that hid I1.
* `gui/multisig_build.go:890` — "false means @SelfSlot is filled …", the retired singular field name.

Grep used: `grep -n "always sets exactly one\|still single-select\|@SelfSlot\|p.SelfSlot\b\|selfSlot" gui/*.go | grep -v _test`.

---

## Checked and found SOUND (recorded so the next reviewer does not re-derive)

* `commonOrigin` walks every cosigner, compares parsed components, and is spelling-insensitive.
* `emptyOriginSlot` + the declared-spelling walk correctly skips held slots when attributing the
  depth-0 refusal to a card.
* `writeOriginPath` guards `>15` components before writing the 4-bit depth field — no truncation,
  no bit-stream corruption from a deep card. `readOriginPath`/`readPathDecl` round-trip divergent
  paths (confirmed by RUN: assembled md1 decoded back to `m/48h/0h/0h/2h`, `m/48h/0h/1h/2h`,
  `m/48h/0h/0h/2h`).
* `buildSlotSources`' account numbering keyed on `MasterFP`: ascending-slot deterministic, aligned
  with `p.SelfSlots` (sorted ascending by `slices.Sort`) and with `seedIDs` by the caller's loop.
  Two masters colliding on a 4-byte fingerprint would mis-number accounts but produces *different*
  keys, so it fails safe; two identical masters getting one account would produce a duplicate key
  and hit §4.1. Both directions safe.
* `buildSlotSources`' `gi` walk, `assembleBuildPolicy`'s `gi` walk and `buildCosignerOrigins`' `gi`
  walk skip the same slots in the same order — no card→slot desync in the widened model.
* `duplicateSlotPair` still compares only `ChainCode`+`CompressedPubkey`; divergent origins cannot
  hide a repeated key.
* `deriveAccountXpub` handles an arbitrary-length path and scrubs each intermediate; the
  serialize-before-`Zero()` ordering (R0-C1) is intact.
* `buildEngraveTail`'s `both` arm derives at the card's declared origin, which the gate has already
  proved the seed reaches — engrave origin and declared origin cannot come apart.
* The verify re-derives at `keys[s].OriginPath` from the readback md1, which is the same per-slot
  origin the engrave stamped — consistent under divergence.
* The one-question-for-all-held-slots assumption in `buildSelfSourceFlow` does fail loudly: a held
  slot bound to a card that is not the operator's produces `errBuildSeedKeyMismatch` naming that
  slot (traced through `buildSlotSources` → `buildSlotGate`).
