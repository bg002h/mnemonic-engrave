# REVIEW-F76-F437-r1 — adversarial pre-merge review of the payload-door burn-down

**Target** `git diff e456970..433d265` on branch `f76/payload-door` in
`/scratch/code/shibboleth/sh-worktrees/f76-payload-door`
(`49173ea` the primable card-set door, `433d265` the SCAN CARDS relabel).
**Under review as evidence** `design/agent-reports/IMPL-F76-F437.md` @ `2ccd1a6`,
and the `### F-76` (both widenings) / `### F-437` entries in `design/FOLLOWUPS.md`.

**Verdict: RED — 0 Critical / 2 Important / 3 Minor / 3 Nit.**

**No funds-safety, correctness or validation defect was found.** Every claim in
the brief's Critical band was constructed and refuted: no cross-card material can
reach a gatherer, no payload card answers another card's question, and the
payload path's validation is byte-for-byte the NFC path's. Both Importants are
**records-level** — one is a test that cannot fail for the reason it and the
report state, the other is an operator route the report presents as fixed that
does not exist. Neither requires a production-code change to close, and neither
makes the shipped behaviour wrong. On the diff's own subject the work holds.

Modified nothing. Worktree left byte-identical (verified, §8).

---

## 1. Suites, run once, by me

| gate | command | result |
| --- | --- | --- |
| non-gui packages | `go test $(go list ./... \| grep -v '/gui$')` | **exit 0**, `52 ok` (72 packages listed, 52 with tests) |
| gui shard | `scripts/gui-shard-test.sh ./gui/ 24` | **exit 0** — `RESULT: ok -- all 1028 tests ran across 24 shards`, `=== wall: 25s ===`, 0 `FAIL` lines |
| TinyGo device build | `nix develop --command tinygo build -size full -print-stacks -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller` | **exit 0** |
| `gofmt -l` (11 touched files) | — | empty |
| `go vet ./...` | — | exit 0; every finding is the pre-existing baseline (`unkeyed fields` in `bspline_test.go`, `testing.ArtifactDir` go1.25/go1.26). **Zero findings in any touched file** (grep over the vet output for `bundle_flow\|md1_gather\|mk1_inspect\|multisig.go\|multisig_build_payload\|sysw_session\|wallet_policy\|payload_door\|modal_fits\|sysw_programs` → no hits). |

TinyGo size totals reproduce the report's line exactly:

```
1197684  269080   31612   30956 | 1498376   62568 | total
```

Fixture pins verified independently:

```
59eb99f7b60ff1526d31dcc27d156ee838950d97496dcc30b7c556d90b1c87a3  gui/testdata/f76_md1_card_payload.bin   (560 B)
03aa3113272d92fae73ee0dd5c30baedf0a90a3021cf6d49cdc413977f98c412  gui/testdata/f76_mk1_card_payload.bin   (244 B)
0875699344b48c3b4e26a8df053ee62dcff06b2b9c67e53bb09dc5a2415a514c  gui/testdata/f76_md1_partial_payload.bin (481 B)
```

All three match the constants in `gui/payload_door_walk_test.go:61-67` and the
sizes in the report.

The four new `bundlePendingMessage` rows really run:

```
--- PASS: TestModalsThisBlockTouchesAreDrawnInFull/the_incomplete-card_refusal,_reader_+_payload
--- PASS: TestModalsThisBlockTouchesAreDrawnInFull/the_incomplete-card_refusal,_reader_only
--- PASS: TestModalsThisBlockTouchesAreDrawnInFull/the_incomplete-card_refusal,_payload_only
--- PASS: TestModalsThisBlockTouchesAreDrawnInFull/the_incomplete-card_refusal,_neither
```

Structural gates:

```
TestEverySyswConsumptionSiteNamesAnAdmittedClass  PASS  "10 consumption sites reconciled against §3.3.2"
TestEveryNonSeamProgramReachesThePayload          PASS
TestTheBundleSeedIsBothWrittenAndRead             PASS
```

---

## 2. Mutations reproduced

All run via `go test -overlay` against a mutated copy in scratch; the worktree
was never written to.

**M1 (the implementer's) — remove `!g.isPrimed()` from `syswPrimeCard`.**
Reproduced verbatim:

```
--- FAIL: TestF76PrimingNeverSubstitutesACardForAnUnprimedGatherer (0.00s)
    payload_door_walk_test.go:470: the payload primed a gatherer that had identified no set; a payload card would then answer a question about a different one
    payload_door_walk_test.go:474: the payload COMPLETED an unprimed gatherer
```

**M2 (the implementer's) — revert the door to a single record**
(`ctx.syswBundleSeeds = []string{bodies[0]}` in `bundle_flow.go`):

```
--- FAIL: TestF76BundleCountsACompleteMd1CardFromThePayload
--- FAIL: TestF76BundleCountsACompleteMk1CardFromThePayload
--- FAIL: TestF76CompletePayloadNeverSeesTheIncompleteRefusal
```

(`TestTheBundleSeedIsBothWrittenAndRead` did not fire here, but that is an
artefact of my method, not a gap: it reads `bundle_flow.go` from disk with
`os.ReadFile`, which the `-overlay` mutation does not reach.)

**M3 (mine) — the exact "one-string shortcut" the report says is forbidden:**
`syswAltEnter = "ENTER IT"` → `"SCAN CARDS"`. **The whole 1028-test gui suite
stays green.** See Important I1.

**M4 (mine) — `syswPrimeCard` consumes `sysw.ClassMnemonic` instead of
`ClassMDMK`:**

```
--- FAIL: TestF76InspectDescriptorCompletesFromThePayload
--- FAIL: TestF76InspectKeyCompletesFromThePayload
```

Caught — behaviourally, not by the §13 D7 oracle. See Nit N1.

---

## 3. Brief item 1 — the primed-only guard, cross-card, constructed

I did **not** reuse the implementer's cross-card test. Its foreign card is an
`mk1` set against an `md1` gatherer, which `md.ParseChunkHeader` rejects outright
(`gatherIgnored`) — the easy case. I built the hard one: a payload holding **two
different md1 cards plus an mk1 card**, using the package's own distinct-csid
fixtures.

Characterised first, so nothing below rests on a name:

```
md1CardA: 6 records, chunked=true csid=0x2d950 total=6   (wsh(sortedmulti) 2-of-3)
md1CardB: 1 record,  chunked=true csid=0x157ae total=1   (wsh_multi_chunked)
mk1CardA: 2 records
payload = 9 ClassMDMK records
```

| probe | construction | result |
| --- | --- | --- |
| md1 primed on A | `g.offer(A[0])`, `syswPrimeCard` over the 9-record payload | completes with **exactly A's six chunks**; `md.DecodeChunks(g.collected())` deep-equals `md.DecodeChunks(A)`; no chunk of B present |
| md1 primed on B | same payload | holds exactly B's one record; A's six never join |
| mk1 primed on A | payload = mk1 A + mk1 B + md1 A | completes with **exactly mk1 A's two chunks**; none of mk1 B's |
| **Inspect, framewise** | `md1GatherFlow(ctx, th, A[0])` run twice — once with a payload holding **only A**, once with the 9-record mixed payload — capturing every rendered frame | **24 frames, byte-identical between the two runs.** First frame `"Type2-of-3multisigScriptSegwit(P2WSH)EngraveDescriptor"`. No scan screen. |

That last one is the direct answer to *"Inspect on card A never shows card B's
chunks"*: with foreign cards in the payload the flow renders **exactly** what it
renders without them, frame for frame.

**Mechanism, read rather than assumed.** `md1Gatherer.offer` (`gui/md1_gather.go:28-50`)
and `mk1Gatherer.offer` (`gui/mk1_inspect.go:51-68`) both answer `gatherForeign`
to any `!h.Chunked || h.ChunkSetID != g.setID || h.TotalChunks != g.total` once
primed. `syswPrimeCard` refuses an unprimed sink, so the adopt-the-first-set
hazard is unreachable — mutation-confirmed (M1).

**Defence in depth beyond the gatherer**, which matters because `ChunkSetID` is
only 20 bits: `md/chunk.go:288-291` re-derives the csid from the *decoded*
descriptor and returns `ErrChunkSetIDMismatch` if it disagrees, and
`gatheredDescriptorFlow` surfaces that as `"Chunks don't match - mixed or
tampered set."` (`gui/md1_gather.go:166`). A csid collision therefore cannot
deliver a wrong descriptor to a plate; it produces a refusal.

### The mixed case the report flags as changed behaviour

Both refusals are **pre-existing and byte-identical** at `e456970`:

```
$ git grep -c -F "Supply exactly one wallet policy (md1) card." e456970 -- gui/
e456970:gui/wallet_policy.go:1
$ git grep -c -F "Supply exactly one wallet policy (md1) card." 433d265 -- gui/
433d265:gui/wallet_policy.go:1
$ git grep -c -F "Supply exactly one wallet-policy md1 (and no key cards)." e456970 -- gui/
e456970:gui/multisig.go:1
$ git grep -c -F "Supply exactly one wallet-policy md1 (and no key cards)." 433d265 -- gui/
433d265:gui/multisig.go:1
```

Walked:

- Wallet Policy, payload = A + B → `md1 descriptors: 2` → Done →
  `"Supplyexactlyonewalletpolicy(md1)card.WalletPolicy"`, and **not** the
  incomplete-card message.
- Engrave Multisig, payload = A + B + mk1 → Done →
  `"Supplyexactlyonewallet-policymd1(andnokeycards).EngraveMultisig"`, no
  incomplete-card message.
- Engrave Bundle, same payload →
  `"EngraveBundlemd1descriptors:2mk1keys:1Donewhenyouhavereviewedthese."`

### State pollution after the refused attempt — clean

`takeAll` (`gui/sysw_session.go:167-178`) is **read-only**; it does not consume.
`bundleGatherFlowResume` sets `ctx.syswBundleSeeds = nil` after feeding them
(`gui/bundle_flow.go:216`). Measured after driving the two-card refusal to
completion:

- `ctx.sysw.cardSet(sysw.ClassMDMK)` still returns all 7 records, `ok=true`.
- `ctx.syswBundleSeeds` is `nil`.
- A one-card payload then reaches the consent screen:
  `"WalletPolicyPolicy-ID:a67e07d16b2500fde6c557a76c7390f6Type:P2WSH2-of-3multisig(sorted)@05a0804e3…"`
  with no refusal.

Declining the door (Back) leaves nothing behind:
`"EngraveBundlemd1descriptors:0mk1keys:0…"`.

---

## 4. Brief item 2 — validation parity with the NFC path

Read side by side first: `syswPrimeCard` calls `g.offer(r)` — the **same method**
the scanner loop calls at `gui/md1_gather.go:104` and `gui/mk1_inspect.go` — and
there is no other insertion path into either gatherer (grep: `offer(` has no
non-`_test` caller that bypasses it). Then measured, six chunk shapes, each fed
**by payload** and **by scan** into two independent gatherers primed identically:

| shape | scan → set / complete | payload → set / complete | agree |
| --- | --- | --- | --- |
| in order | `[0 1 2 3 4 5]` / true | `[0 1 2 3 4 5]` / true | ✓ |
| reversed | `[0 1 2 3 4 5]` / true | `[0 1 2 3 4 5]` / true | ✓ |
| duplicate chunk appended | `[0 1 2 3 4 5]` / true | `[0 1 2 3 4 5]` / true | ✓ |
| a **different card** spliced into the set | `[0 1 2 3 4 5]` / true | `[0 1 2 3 4 5]` / true | ✓ |
| **corrupted BCH checksum** on chunk 5 | `[0 1 2 3 4]` / **false** | `[0 1 2 3 4]` / **false** | ✓ |
| chunk 4 missing | `[0 1 2 3 5]` / false | `[0 1 2 3 5]` / false | ✓ |

`setID` and `total` identical in every row. The corrupted chunk is refused at
**three** independent layers, all re-measured by me:

1. `(&md1Gatherer{}).offer(corrupt)` → `gatherIgnored`;
2. a **primed** gatherer also answers `gatherIgnored` (my addition — the report
   only tested the unprimed call);
3. `sysw.Classify(corrupt) != sysw.ClassMDMK`, so it never becomes a record the
   door could hand over;
   and `md.DecodeChunks(set-with-corrupt-chunk)` errors.

**Validation is not weaker than the NFC path. It is the same code, and it
measures the same.**

---

## 5. Brief item 3 — the truthful message, and the boundaries

- **Genuinely short a chunk** (`f76_md1_partial_payload.bin`, 5 of 6): both arms
  reproduce — reader-less gets the re-pack sentence, NFC gets both routes. The
  advice is now **followable**: the payload really is short a chunk, and
  `me sysw pack` warns-but-does-not-refuse on that input, so the operator input
  is reachable.
- **Single-record card** (`md1CardB`, `chunked=true total=1`): no regression.
  Engrave Bundle counts `md1 descriptors: 1` from the payload and Done proceeds
  to `"Bundle1cardsverified:1.md1descriptorOKP2WSH3-of-3multisig"` — no
  incomplete-card message.
- **Complete payload never sees the refusal**: reproduced.
- **`bundlePendingMessage` table**: all four arms drawn in full (§1).

### A near-finding that resolved clean: `hasPayload` skips the `compared` gate

`bundleGatherFlowResume` computes `hasPayload: ctx.sysw != nil && ctx.sysw.has(sysw.ClassMDMK)`,
and `has()` deliberately ignores `compared` while `takeAll` enforces it. I
constructed an uncompared session directly and confirmed the asymmetry is real:
`has()==true`, `cardSet()` returns `ok=false`, the door draws, FROM PAYLOAD
yields `md1 descriptors: 0` silently, and Done prints
`"No complete cards. Pack them on the host with 'me sysw pack' and load the payload again."`
— advice that cannot be followed.

**Not a finding: the state is unreachable in production.** `gui/sysw_load.go:203`
drops the session outright when the comparison is declined —
`ctx.sysw = nil; showError(…, "Digest not compared.\nNothing was loaded.")` —
under the operator ruling of 2026-08-13, whose recorded reason is exactly this
`has`/`take` asymmetry. `syswSession.load` has one non-test call site
(`gui/sysw_load.go:190`). Recorded here so a future change that resurrects an
uncompared session knows what it re-opens.

---

## 6. Brief item 4 — the relabel

Enumerated every `syswOffer*` / `syswChoose` call site (grep, non-test) and
traced each **decline arm** to what it actually reaches:

| site | class | label | decline arm reaches |
| --- | --- | --- | --- |
| `bundle_flow.go:34` | MDMK | **SCAN CARDS** | `bundleGatherFlowResume` — NFC card gather ✓ |
| `multisig.go:99` | MDMK | **SCAN CARDS** | `bundleGatherFlow` — NFC card gather ✓ |
| `wallet_policy.go:45` | MDMK | **SCAN CARDS** | descriptor offer, then `bundleGatherFlowResume` ✓ |
| `wallet_policy.go:47` | Descriptor | **SCAN CARDS** | `bundleGatherFlowResume` ✓ (F-437's own subject) |
| `passphrase_flow.go:662` | Passphrase | ENTER IT | `ppStepEntry` → `passphraseEntryFlow` — keyboard ✓ |
| `sysw_source.go:103` | Passphrase | ENTER IT | `passphraseFlowTitled(ctx, th, "Enter Passphrase")` — keyboard ✓ |
| `freetext_flow.go:1496` | FreeText | ENTER IT | `ftTextEntryFlow` — keyboard ✓ |
| `gui.go:2790` | Mnemonic | ENTER IT | `Input Seed` menu → `inputWordsFlow` — keyboard ✓ |
| `gui.go:2806` | Codex32Secret | ENTER IT | `Input Seed` menu → `inputCodex32Flow` — keyboard ✓ |

Four SCAN CARDS doors, all genuinely NFC card gathers. Five ENTER IT doors, all
genuinely keyboards. **The keyboard was proven to open**, not inferred: walking
the mnemonic door and declining reaches
`"Choosenumberofwords12WORDS24WORDSInputSeed"`.

**No other door carries the mislabel.** Tree-wide grep for
`ENTER IT|TYPE IT|SCAN CARDS|SCAN IT` over non-test `.go` finds only the two
constants, one comment, and `gui/derive_xpub.go:286-287` — `syswSeedPickerTitled`'s
own `{"TYPE IT", srcTyped}` row with a `SCAN` row gated on `FeatureNFC`, which is
a keyboard picker and correctly labelled.

**But the guard the report cites for this is inert — see Important I1.**

---

## 7. Brief item 5 — everything the diff touches

14 files, all accounted for:

```
M gui/bundle_flow.go               F-76 door + hasPayload + bundlePendingMessage
M gui/md1_gather.go                isPrimed + syswPrimeCard call
M gui/mk1_inspect.go               isPrimed + syswPrimeCard call
M gui/modal_fits_test.go           four new message rows
M gui/multisig.go                  door
M gui/multisig_build_payload.go    takeAll+groupRecordsByCard -> cardSet
M gui/sysw_programs_test.go        two structural gates re-shaped
M gui/sysw_session.go              cardSet, syswChoose, alt labels, syswOfferAlt,
                                   syswOfferCards, chunkSink, syswPrimeCard
M gui/wallet_policy.go             both doors
A gui/payload_door_label_test.go   F-437 walks
A gui/payload_door_walk_test.go    F-76 walks
A gui/testdata/f76_*.bin           three fixtures
```

- `multisig_build_payload.go` is the only edit outside the two entries' literal
  scope. The report gives its reason and the reason checks out: `cardSet` **is**
  `takeAll` + `groupRecordsByCard` verbatim (`gui/sysw_session.go:215-221`), so
  `buildCosignerSource` is unchanged in behaviour, and the Build path's tests are
  inside the green 1028.
- `wallet_policy.go:47`'s move from `syswOffer` to `syswOfferAlt` keeps the same
  title (`"Input"` — `syswOffer` delegated to `syswOfferTitled(…, "Input", …)`),
  so only the second choice's label changed.
- **The S2 seam surface is byte-identical.** `git diff --stat e456970..433d265 --
  nonstandard/ sysw/` is empty, and no file matching `*classif*` appears in
  `git diff --name-status`.

---

## 8. Worktree

```
$ git status --porcelain      # (empty)
$ git rev-parse HEAD          433d265647c8f1d42b3b1ec3a4aa561c10e63d0c
$ git rev-parse HEAD^{tree}   f323083e0003322a4b7db8105a2d8809f8a55438
$ ls gui/zz_*                 no matches
```

All probes ran via `go test -overlay` with the probe and mutated sources held in
the scratchpad. Nothing was written to the worktree, nothing committed, nothing
pushed.

---

# FINDINGS

## Critical — none

Every constructed attack in the Critical band was refuted with a measurement:
cross-card isolation (§3), same-validation-as-NFC (§4), no state pollution (§3),
no single-record regression (§5). Priming goes **through** `offer()`, there is no
second insertion path, and the csid is re-derived from the decoded descriptor
downstream of the gatherer.

---

## Important

### I1 — `TestF437KeyboardDoorsKeepEnterIt` cannot fail for the reason it and the report state; the five keyboard doors are guarded by nothing

**The claim.** The report's evidence table:

> `TestF437KeyboardDoorsKeepEnterIt` … green — **forbids the one-string shortcut
> inside `syswChoose`** that would have lied at four honest doors

and the test's own comment (`gui/payload_door_label_test.go:91-94`):

> The classes that really DO reach a keyboard keep ENTER IT … Without this the
> rename could have been made by changing **one string in `syswChoose`**, which
> would then lie in the other direction at four honest doors.

**The measurement.** I applied exactly that shortcut —
`syswAltEnter = "ENTER IT"` → `"SCAN CARDS"` in `gui/sysw_session.go:233` — and
ran the **whole** gui package:

```
M3 applied: syswAltEnter -> SCAN CARDS
EXIT=0
ok  	seedhammer.com/gui	147.044s
```

**Green. 1028 tests, nothing fires.**

**Why.** The test walks `seedEntryFlow`, which routes to
`syswSeedPickerTitled` (`gui/derive_xpub.go:267-296`) — a picker that builds its
own rows `{"FROM PAYLOAD"}`, `{"TYPE IT"}`, `{"SCAN"}` and **never calls
`syswChoose`, `syswOfferAlt` or `syswOfferTitled`**. The frame it actually draws:

```
"Wherefrom?FROMPAYLOADTYPEITInputSeed"
draws ENTER IT: false
draws TYPE IT : true
```

So the assertion at `gui/payload_door_label_test.go:105`

```go
if !uiContains(got, "ENTER IT") && !uiContains(got, "TYPE IT") {
```

has a **dead first disjunct**: only the `TYPE IT` arm can ever match, and
`TYPE IT` is a string `syswAltEnter` does not control.

**Consequence.** None of the five doors that really do use `syswAltEnter` —
`passphrase_flow.go:662`, `sysw_source.go:103`, `freetext_flow.go:1496`,
`gui.go:2790`, `gui.go:2806` — is asserted by any test in the tree (grep for
`ENTER IT` in `gui/*_test.go`: the only occurrences are this file's). A future
edit to the shared constant reintroduces F-437 in mirror image at five honest
doors, silently.

**Severity.** This is the "a gate that cannot fail" class, which
`/scratch/code/CLAUDE.md` lists as still blocking. **The shipped behaviour is
correct** — I proved the keyboard opens (§6) — so the fix is test-only: assert
against a door that actually uses the constant (e.g. `engraveTextFlowFrom` or
`engravePassphraseFlowFrom`), or drop the `TYPE IT` disjunct, or pin
`syswAltEnter == "ENTER IT"` directly. Any of those makes M3 red.

---

### I2 — F-76's original scope is presented as closed, but no operator route reaches Inspect on a payload-sourced card

**The claim.** The report presents two rows as "F-76 original":

> `TestF76InspectDescriptorCompletesFromThePayload` … reaches `Engrave
> Descriptor`, never draws a scan screen
> `TestF76InspectKeyCompletesFromThePayload` … returns the decoded card

under the heading

> **F-76's original scope.** … so Inspect on a chunked record — the ordinary case
> — **stranded the operator** on a scan-waiting screen whose only exit was Back.

**The call graph, traced end to end.**

1. `"Inspect key"` / `"Inspect descriptor"` exist in exactly one place:
   `mdmkFlow` (`gui/gui.go:2657,2660`). Tree-wide grep for `Inspect` finds no
   other entry.
2. `mdmkFlow` has exactly **one** non-test caller: `engraveObjectFlow`
   (`gui/gui.go:2505`), whose own comment reads *"this switch is only ever
   reached from a scan"*.
3. `engraveObjectFlow` has exactly **one** non-test caller: `gui/gui.go:2096`,
   fed by `obj := act.scan` (`gui/gui.go:2049`) — an NFC tag, or `newInputFlow`'s
   `bip39.Mnemonic`, which is not an `mdmkText`.
4. The payload's own routes offer no Inspect. `syswPayloadMenu` offers
   `ENGRAVE TRANSACTION / LOAD AGAIN / UNLOAD`. The sealed-payload record list
   routes to `unlockEngraveFlow` (`gui/unlock_platelist.go:113`), a
   `ChoiceScreen{Lead: "Choose engraving"}` with **no Inspect entry** — and
   `unlock_platelist.go:189-201` says so explicitly, citing F-76 as the reason.

**Therefore an operator holding a payload cannot reach `md1GatherFlow` or
`mk1GatherFlow` for a payload record.** The two tests call those functions
directly with `first = wshSortedmultiChunks[0]` — the position a **scanned**
chunk occupies. The RED baselines quoted for them
(`"InspectdescriptorCaptured1of6.Scanthenextchunk."`) are measurements of a
synthetic call, not of an operator path.

**What the change does deliver, reachably, and it is real:** an operator who taps
**one** chunk of a card the payload also carries now gets the remaining chunks
from the payload instead of tapping the rest. That is a genuine improvement, it
is safe (§3, §4), and `gui/md1_gather.go:81-84` describes it accurately.

**Severity.** Not a safety defect and not a stranding regression — the strand the
entry describes was already prevented by the routing that deliberately keeps
`mdmkFlow` away from payload records. It is Important because the merge decision
rests on this report, and closing F-76 on this evidence loses the residue: the
Inspect **entry point** for payload records still does not exist, and the F-76
entry's *"implement it once, for Inspect AND the two engrave programs"* is half
met. Closing it costs a corrected F-76 note (and, if the capability is still
wanted, a follow-up owning the platelist entry) — **no production-code change,
and no reason to hold the engrave half.**

---

## Minor

### M1 — the three card doors still say "First card from where?" while now handing over *every* card

`syswOfferCards` delivers the whole card set, but the lead at all three MDMK
doors is unchanged: `"First card from where?"`. Measured: one FROM PAYLOAD tap on
a 9-record payload produces
`"EngraveBundlemd1descriptors:2mk1keys:1Donewhenyouhavereviewedthese."`. This is
the same honesty class F-437 exists for, in a string the diff did not touch.
Mitigated, which is why it is Minor: the very next screen shows the count, and
`bundleReviewFlow` plus the consent screen stand between it and any plate.

### M2 — `SCAN CARDS` is drawn unconditionally, including where there is no reader

`syswChoose` draws the alt label with no `FeatureNFC` test, unlike
`syswSeedPickerTitled`, which gates its `SCAN` row on it
(`gui/derive_xpub.go:287`). Measured on a platform reporting no NFC:
`"Firstcardfromwhere?FROMPAYLOADSCANCARDSInput"` with
`Features().Has(FeatureNFC)=false`. **Reaches no operator today** —
`cmd/controller/platform_sh2.go:313` sets `FeatureNFC` unconditionally and
`gui/bundle_flow.go:113-116` records why ("the SH2's ST25R3916 is soldered to
every board") — so this is a latent inconsistency, not a live lie.

### M3 — the priming path discloses no source (§3.3.3 F3)

The four doors ask the operator explicitly. `syswPrimeCard` does not: an operator
who taps one chunk has the set silently completed from the payload, and no screen
says the payload contributed. Every other non-typed entry draws
`syswSourceAccept`. The material is csid-bound and re-verified downstream
(`md/chunk.go:288-291`), and the payload is `[compared]`-authenticated before
`takeAll` will release anything, so nothing wrong can arrive this way — which is
why it is Minor rather than Important. Related, and **non-gating by the operator
ruling of 2026-08-27**: a ClassMDMK record marked `unconfirmed` counts as secret
for §3.3.3 flags, and this path raises none.

---

## Nit

### N1 — `syswPrimeCard` is a payload-consumption site §13 D7's oracle cannot see

`TestEverySyswConsumptionSiteNamesAnAdmittedClass` skips `sysw_session.go` by
name (`gui/sysw_admit_oracle_test.go:120`) and matches only `syswOffer*` calls
and the selector `take` — so `syswPrimeCard`'s hard-coded
`ctx.sysw.cardSet(sysw.ClassMDMK)` is outside it twice over. Nor does `mdmkFlow`'s
Inspect path have a `syswProgram` row in `admitted` at all. **The report's own
claim is narrowly true** (all four *doors* are still reconciled — the oracle logs
`10 consumption sites`). Nit rather than Important because the class **is** pinned
behaviourally: M4 (`ClassMDMK` → `ClassMnemonic`) fails two tests.

### N2 — `gui/unlock_platelist.go:189-201`'s doc comment is now half-false

It still explains that `mk1GatherFlow` / `md1GatherFlow` "prime a FRESH gatherer
with only the single string handed to them" and then wait for physical tags. They
no longer do — they prime from the payload first. Its *conclusion* (that
`unlockEngraveFlow` does not reuse `mdmkFlow`) remains correct, and the comment is
the load-bearing record of I2, so it wants a correction rather than a deletion.
Classic "a diff falsifies text it never touches".

### N3 — the report's characterisation of `bundleRefusedSingleMK1` overstates its correctness

The report defers it correctly (out of scope, flagged rather than silently
widened) but says it fires "on a non-chunked single mk1, **where it is correct**".
Measured: `feedback` is reached only from the scan loop
(`scr.msg = scr.feedback(scr.g.offer(scan.Object))`; the payload seeds are offered
without capturing feedback), so its `!hasReader` arm — *"the payload is missing
some of its chunks"* — is **unreachable**, and its reachable arm tells the
operator to *"scan all its chunks"* about a card that has none. Pre-existing,
untouched by this diff, correctly out of scope; recorded so the deferral carries
the right description.

---

## What I checked and found clean (so a re-review does not re-derive it)

- Cross-card isolation, md1 and mk1, constructed with two distinct-csid md1 cards
  plus an mk1 card — including a frame-for-frame Inspect comparison.
- Payload/scan parity across six chunk shapes; corrupted-chunk refusal at three
  layers plus a primed-gatherer fourth.
- Both "supply exactly one" refusals byte-identical at `e456970`.
- `takeAll` is non-consuming; `syswBundleSeeds` nil'd after use; a one-card
  payload works after the refusal path.
- Single-record (`total=1`) card: no regression at the bundle door.
- Nil session / no payload: `syswPrimeCard` is a no-op, no offer drawn, gather
  unchanged.
- Uncompared session unreachable (`gui/sysw_load.go:203`).
- All four `bundlePendingMessage` arms drawn in full.
- S2 seam (`nonstandard/`, `sysw/`) byte-identical; no classifier file touched.
- `multisig_build_payload.go`'s refactor is behaviour-preserving by construction.
- Fixture SHA-256s and sizes match their pins.
- Suites: non-gui 52 ok / exit 0; gui 1028/1028 across 24 shards; TinyGo exit 0
  with the report's exact size totals; gofmt clean; vet clean on every touched
  file.
- Mutations M1 and M2 reproduced exactly as reported.

**Nothing here is a reason to hold the merge on funds, correctness or validation
grounds.** The two Importants are both about what the record claims, and both
close without touching production code.
