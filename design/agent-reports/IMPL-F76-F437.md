# IMPL-F76-F437 — the payload door, burned down

**Worktree** `/scratch/code/shibboleth/sh-worktrees/f76-payload-door`, branch
`f76/payload-door`, based on `e456970` (the code currently flashed).
**Not pushed. Report not committed.**

| commit | subject |
| --- | --- |
| `49173ea0efe71e0bf11fae47ea44f3c957de9312` | `gui: F-76 -- the payload door hands the gatherer the WHOLE card set` |
| `433d265647c8f1d42b3b1ec3a4aa561c10e63d0c` | `gui: F-437 -- the card doors say SCAN CARDS, not ENTER IT` |

Two commits, one per follow-up entry, each green on its own. `49173ea` carries
the door, the Inspect path and the message (F-76's own entry requires the
message to be corrected "in the same change that fixes the door"); `433d265`
carries only the label.

---

## The primable gatherer, in three sentences

`syswSession.cardSet(want)` is `takeAll` + `groupRecordsByCard` — the two lines
Build Policy has fed its gather with since S1 — moved into one method and given
to all four callers, so "which records make up the payload's cards" has exactly
one answer. On the engrave routes, `syswOfferCards` draws the same picker
`syswOffer` did and returns that whole set instead of the first record, which the
three doors assign to `ctx.syswBundleSeeds` so every chunk enters through the
same `bundleGatherer.offer()` a scanned card takes. On the Inspect route,
`syswPrimeCard(ctx, g)` feeds the same set through `md1Gatherer.offer` /
`mk1Gatherer.offer` (the `chunkSink` interface) **before** the NFC reader is
opened, and is **primed-only** — an unprimed gatherer adopts the first set it is
offered, so feeding one could let a payload card silently answer a question about
a different card.

**What did NOT change:** admission. The records travel the same `offer()`, so
dedup, chunk assembly, chunk-set-id matching and the BCH integrity gate are
byte-identical. This changes *how many* records reach a gatherer, never *how they
are admitted*. Each door still hard-codes `sysw.ClassMDMK` itself, so
`TestEverySyswConsumptionSiteNamesAnAdmittedClass` (§13 D7) can still reconcile
all four sites against §3.3.2 — `cardSet`/`syswOfferCards` take the class as a
parameter for exactly that reason, not for generality.

---

## Test evidence — every walk measurement, before and after

All measurements are rendered frames from headless-sim walks
(`runUI`/`pumpUntil`/`click`), in `gui/payload_door_walk_test.go` and
`gui/payload_door_label_test.go`. The **before** column is the RED run of those
same tests against unmodified `e456970` production code.

| walk | test | before (RED, verbatim frame) | after |
| --- | --- | --- | --- |
| J2 | `TestF76WalletPolicyCountsACompleteMd1CardFromThePayload` | `"WalletPolicymd1descriptors:0mk1keys:0Donewhenyouhavereviewedthese."` | reaches `md1 descriptors: 1` |
| J2BUNDLE | `TestF76BundleCountsACompleteMd1CardFromThePayload` | `"EngraveBundlemd1descriptors:0mk1keys:0..."` | reaches `md1 descriptors: 1` |
| FU2 | `TestF76BundleCountsACompleteMk1CardFromThePayload` | `"EngraveBundlemd1descriptors:0mk1keys:0..."` | reaches `mk1 keys: 1` |
| FU2 control | `TestF76BundleCountsTheSameMk1CardWhenSeededDirectly` | already green (1) | still green — the control that proves it was the DOOR, not the cards |
| F-76 original | `TestF76InspectDescriptorCompletesFromThePayload` | `"InspectdescriptorCaptured1of6.Scanthenextchunk."` | reaches `Engrave Descriptor`, never draws a scan screen |
| F-76 original | `TestF76InspectKeyCompletesFromThePayload` | `"InspectkeyCaptured1of2.Scanthenextchunk."` + `mk1GatherFlow` returned no card | returns the decoded card, draws **no** frame |
| FU1-NFC | `TestF76IncompletePayloadNamesBothRoutesOnAnNFCMachine` | `"Droppedanincompletecard.Scanallitschunkstoincludeit."` | names both routes: scan them **or** re-pack with `me sysw pack` |
| FU1 (no-NFC) | `TestF76IncompletePayloadGetsTheRepackAdvice` | already green — and now **true** rather than merely plausible | green |
| new | `TestF76CompletePayloadNeverSeesTheIncompleteRefusal` | `"...md1descriptors:0..."` (card never assembled) | Done proceeds to `N cards verified`, no refusal |
| FU3 / J2 | `TestF437CardDoorsDoNotPromiseTyping` (4 subtests) | `"Firstcardfromwhere?FROMPAYLOADENTERITInput"` ×3 and `"Walletpolicyfromwhere?FROMPAYLOADENTERITInput"` | all four draw `SCAN CARDS`, none draws `ENTER IT` |
| new | `TestF437KeyboardDoorsKeepEnterIt` | green | green — forbids the one-string shortcut inside `syswChoose` that would have lied at four honest doors |

### The message, all four arms

`bundlePendingMessage(hasReader, hasPayload)` is a pure function with a table
test (`TestF76PendingMessageNamesOnlyRoutesThatExist`) asserting each arm names
scanning **iff** there is a reader and a re-pack **iff** there is a payload:

- reader + payload → *"Dropped an incomplete card: some of its chunks are missing. Scan them, or re-pack the payload on the host with `me sysw pack`."*
- reader only → *"Dropped an incomplete card. Scan all its chunks to include it."* (unchanged; still correct)
- payload only (phase-1 hardware) → the shipped re-pack sentence, now **true**
- neither → *"…some of its chunks are missing, and this device has no card reader to scan them with."*

All four were added to `TestModalsThisBlockTouchesAreDrawnInFull` and measured
drawn in full: headroom **455 / 513 / 455 / 476** characters against the 80-char
F-185 margin.

### Fixtures

Real `me` containers, committed, each pinned by SHA-256 **and** by asserting the
opened records are byte-equal to this package's existing committed card
constants — a stronger binding than the hash, because it makes the container and
the expectation unable to drift apart:

```
me sysw pack --no-passphrase --in <records> --out gui/testdata/<name>.bin
```

| fixture | records | source constant |
| --- | --- | --- |
| `gui/testdata/f76_md1_card_payload.bin` (560 B) | 6 | `wshSortedmultiChunks` (`gui/md1_gather_test.go`) — the same 6-chunk card the walk used |
| `gui/testdata/f76_mk1_card_payload.bin` (244 B) | 2 | `mk1CardA` (`gui/bundle_testdata_test.go`) — byte-identical to the walk's FU2 chunks |
| `gui/testdata/f76_md1_partial_payload.bin` (481 B) | 5 | `wshSortedmultiChunks[:5]` — a payload GENUINELY short a chunk |

The partial container is deliberate. `me sysw pack` **warns** on it (`record N …
an md1/mk1 this tool could not decode; the device will treat it as a SECRET`)
and does not refuse, which is what makes "someone packs 5 of 6" a reachable
operator input and the corrected refusal worth keeping rather than deleting.

---

## The corrupted-chunk refusal (funds path)

`TestF76ACorruptedChunkInThePayloadIsStillRefused` corrupts one symbol of chunk 5
of the committed 6-chunk card and asserts three layers:

1. `(&md1Gatherer{}).offer(corrupt)` returns `gatherIgnored` — the same call an
   NFC scan makes. (`md.ParseChunkHeader` → `codex32.MDDataSymbols` → `ValidMD`,
   i.e. the BCH checksum.)
2. `sysw.Classify(corrupt) != sysw.ClassMDMK`, so it never becomes a record the
   door can hand over at all.
3. The whole door: five good chunks plus one corrupted one reach
   `md1 descriptors: 0` and Done prints the incomplete-card refusal — no partial
   card, and the operator is told.

Its **control** is `TestF76BundleCountsACompleteMd1CardFromThePayload`: the same
six chunks, one symbol apart, count 1. So the `0` is a measurement of the
corruption, not of a dead path. Priming goes **through** `offer()`, never around
it — there is no second insertion path.

**Two further primed-only proofs**, both new:

- `TestF76PrimingNeverSubstitutesACardForAnUnprimedGatherer` —
  **mutation-checked**: removing `!g.isPrimed()` from `syswPrimeCard` turns it
  red with *"the payload primed a gatherer that had identified no set"* and
  *"the payload COMPLETED an unprimed gatherer"*. Restored → green.
- `TestF76PrimingOnlyEverAddsToTheIdentifiedSet` — an md1 gatherer primed on an
  md1 chunk, fed a payload of a whole mk1 card, still holds exactly 1 chunk and
  is not complete.

---

## Two structural gates moved with the call shape (not weakened)

Both failed on the first full shard run and both were fixed by making the gate
track the new shape, with the reason in the test:

- `TestEveryNonSeamProgramReachesThePayload` matched the identifier `syswOffer`
  by **equality**, so it reported *"bundle_flow.go (Engrave Bundle) never calls
  syswOffer, so the payload cannot reach it"* about a door that plainly reaches
  it. Now prefix-matches `syswOffer*` — the rule its sibling oracle
  (`TestEverySyswConsumptionSiteNamesAnAdmittedClass`) already states in its own
  comment for the same reason.
- `TestTheBundleSeedIsBothWrittenAndRead` pinned the literal
  `ctx.syswBundleSeeds = []string{body}`. It now pins the whole-set write **and
  forbids the single-record shape by name**, which turns it into an F-76
  regression guard rather than a spelling check.

`TestEverySyswConsumptionSiteNamesAnAdmittedClass` needed **no** change once the
class was threaded through `cardSet`/`syswOfferCards` — that is why it is a
parameter.

---

## Gate tails (final tree, `433d265`)

```
gui shard (scripts/gui-shard-test.sh ./gui/ 24)
    1028 top-level tests
    partition verified exhaustive: 1028 == 1028
    === wall: 24s ===
    RESULT: ok -- all 1028 tests ran across 24 shards

go vet ./...            diff vs baseline: NO NEW FINDINGS
                        (baseline carries pre-existing unkeyed-fields +
                         testing.ArtifactDir go1.25/go1.26 notes)

gofmt -l                clean on every touched file
                        (gui/transaction.go, gui/transaction_golden_test.go,
                         gui/transaction_txrecord_test.go were already
                         unformatted at e456970 and are NOT touched here —
                         verified by stashing and re-running)

non-gui packages        go test <all but ./gui>: exit 0, 52 ok
S2 seam suite           ok seedhammer.com/nonstandard 0.026s
                        ok seedhammer.com/sysw        0.041s

tinygo (target pico-plus2, -opt 2 -gc precise -scheduler tasks -stack-size 16kb)
    exit 0
    1197684  269080   31612   30956 | 1498376   62568 | total
```

---

## Deviations and notes

1. **TDD order kept, commit order adjusted.** The tests were written and run RED
   first (every frame in the table above is from that run), but they are
   committed *with* their fix so no commit in the branch is red for `git bisect`.
2. **Behaviour change worth naming.** With the whole set delivered, a payload
   holding two md1 cards now reaches Wallet Policy's existing refusal *"Supply
   exactly one wallet policy (md1) card."*, and one holding an md1 plus mk1
   cards reaches Engrave Multisig's *"Supply exactly one wallet-policy md1 (and
   no key cards)."* Before the fix those payloads produced the (false)
   incomplete-card message instead. Both refusals are pre-existing, accurate,
   and strictly more informative than what they replace.
3. **Not touched, deliberately:** `bundleGatherScreen.feedback`'s
   `bundleRefusedSingleMK1` string (*"Incomplete key card: the payload is missing
   some of its chunks."*) is the same claim-shape as the message F-76 names, but
   it fires on a **non-chunked single mk1**, where it is correct. Left alone to
   keep the diff to the entry's scope; flagging it here rather than silently
   widening.
4. **`bundleDoneEmpty`'s message** was likewise left alone: it fires only with
   zero cards *and* nothing pending, where both its arms are still true.
5. **No hardware, no plate.** Every walk stops at a rendered screen. Nothing here
   has been exercised on the real SeedHammer II.
