# S1 execution review — `3a0ac6e` "S1: the payload supplies the whole cosigner set"

Independent adversarial post-implementation gate. Reviewer did not see the
implementation conversation. Repo: `/scratch/code/shibboleth/seedhammer` @
`3a0ac6e` (1,621 insertions, 15 files). Date 2026-08-15.

Question answered: **does the commit do what S1 required, and what did TDD
miss?**

---

## VERDICT

**3 Important, 0 Critical — BLOCKED.**

S1's substance is there and is built *to* the rulings, not around them. The
`0..n` ruling is implemented, `classifyCosignerSupply` is genuinely total, the
`takeAll` `[compared]` guard is real and mutation-proven, the md1 filter works,
under-supply names `me sysw pack` and never says "scan", and the announcement
lands above the stub on the confirmation surface. Thirteen of thirteen
mutations I applied to the *primitives* were killed by the suite.

What blocks is elsewhere. **Every mutation I applied to the FLOW's wiring
survived `go test ./...` with exit 0** — including one that fills every cosigner
slot with the same payload card, and one that deletes the §0.1 announcement
outright. Separately, a ruled spec item (P0 item 6) is unimplemented on the
stage's default arm, and the "payload record order" contract that three files
state absolutely is false for an input the format admits.

None of the three is a wrong result in the shipped code. All three are the
class this project's own standards name: the components are green and the
joint is untested, a record states more than the code holds, and a ruled
deliverable did not land.

---

## CRITICAL

None.

---

## IMPORTANT

### I1 — the flow-level wiring from selection → assembled policy → announcement is asserted by nothing; four defect-injecting mutations survive the FULL suite

`gui/multisig_build.go:76-105,151`

The picker's output, the card→slot mapping, and the §0.1 announcement are each
unit-tested against **hand-built inputs**, and nothing asserts that the flow
passes the real ones. Measured by injecting each mutation into the worktree and
running `nix develop --command go test ./... -count=1`:

| # | mutation | site | result |
| --- | --- | --- | --- |
| M14 | discard the operator's selection, use the first `open` cards | `multisig_build.go:90` | **SURVIVED, full suite exit 0** |
| M15 | `picked = append(picked, mk1s[0])` — every cosigner slot gets payload card 1, i.e. a policy with **duplicate keys** | `multisig_build.go:96-98` | **SURVIVED, full suite exit 0** |
| M16 | `buildCosignerOrigins(p.N, 0, chosen)` — announce the wrong slots | `multisig_build.go:105` | SURVIVED (S1 subset) |
| M18 | pass `nil` provenance — the §0.1 announcement vanishes from the review screen | `multisig_build.go:151` | **SURVIVED, full suite exit 0** |

Reproduction for M15 (the worst): replace

```go
	for _, i := range chosen {
		picked = append(picked, mk1s[i])
	}
```

with `for range chosen { picked = append(picked, mk1s[0]) }`, then
`go test ./...` → `exit 0`, no FAIL lines. On the auto-fill arm this mints a
2-of-3 whose @1 and @2 are the same key; with fingerprints omitted (the
default) the review screen renders `@0 (no fp) / @1 (no fp) / @2 (no fp)` and
the operator cannot see it.

**Why the existing tests do not catch it, measured rather than guessed:**

- `TestBuildSlotOrderIsPayloadRecordOrder` (`multisig_build_payload_test.go:170`)
  never runs the flow. It calls `assembleBuildPolicy`, `buildCosignerOrigins`
  and `buildReviewLines` directly with a hand-written `chosen := []int{0, 2}`.
- `TestBuildOverSupplySelectionIsWalkable` walks USE(1) / SKIP(2) / USE(3)
  through the real screens and then asserts only `pumpUntil(frame, "Seed", 32)`.
  It never checks **which** cards were used, so a flow that ignores the taps
  passes it.
- `TestBoundedSelectionCannotEndShort` exercises `buildCosignerPickFlow` in
  isolation, with the caller replaced by the test.

Plan S1 test 5 requires "asserts `@N` assignment follows payload record order,
**and that the review screen shows it**". That holds for the components and not
for the product. The brief's own question — "does bounded selection preserve
payload record order end to end, **including after a deselect/reselect**" —
has no test at all: no test performs a deselect/reselect, and no test observes
the assembled policy after any selection made through the UI.

This is the "can a user do the thing" failure mode in test form: six green
components, and the call that joins them unasserted.

**Fix (one test, not a redesign):** continue
`TestBuildOverSupplySelectionIsWalkable` past seed entry to the Policy Review
and assert (a) the announcement names **cards 1 and 3** — the ones the walk
actually chose, not 1 and 2 — and (b) with `IncludeFp` on, the per-slot
fingerprints on that screen are fixture 0's and fixture 2's. Both M14 and M15
die to (a); M16 and M18 die to (a) as well.

---

### I2 — spec P0 item 6 is unimplemented: the gather screen still reads "Scan a card, or Done." and on the auto-fill arm it is the ONLY screen

`gui/bundle_flow.go:89` (`bundleGatherScreen.tally()`), unchanged by this commit.

Plan §3 S1, implementation bullet 4: *"The gather screen becomes a **review of
what the payload supplied** (spec P0 item 6). Title fixed in S2 with the rest of
D-4."* Spec P0 item 6: *"The gather screen becomes a review of what the payload
supplied, not a 'Scan a card' prompt. **Ruled here, not deferred**."*

Only the **title** is S2's — D-4 is defined in `SPEC §2.2` as *"the Build-policy
cosigner gather is titled 'Engrave Bundle'"*, and plan S2's file table names
`gui/bundle_flow.go` for the title alone. The prompt text is S1's.

Measured by driving the real flow (`n=3`, exactly two payload cards, the
EQUAL-COUNT / auto-fill arm), at the default test platform size and again at
`sh2DisplaySize` (480×320):

```
AUTO-FILL ARM, the screen the operator meets after the pickers:
  "EngraveBundle md1descriptors:0 mk1keys:2 Scanacard,orDone."
next screen after Done: "Choosenumberofwords 12WORDS 24WORDS InputSeed"
```

On the auto-fill arm that is the **entire** review of what the payload
supplied: a card count and an instruction to scan a card, on hardware with no
reader. This is the stage's own motivating defect — *"an instruction nobody
could follow"* — surviving on the arm the stage reaches by default. The new
`Payload cards` review screen exists only inside `buildCosignerPickFlow`, i.e.
only on **over**-supply.

(The over-supply arm is fine, and I checked the paging worry: at 480×320 the
whole 7-line list including the four `N. mainnet | m/48h/0h/0h/2h | fp …`
identity lines fits on page 1. Only the small default test-platform display
truncates it.)

Related, same class, also reachable from Build: `gui/bundle_flow.go:135`
shows *"Dropped an incomplete card — scan all its chunks to include it."* A
payload holding ≥`open` complete cards **plus** a half chunk set classifies as
auto-fill/select (measured: `supply=2 incomplete=true classify(open=2)=1`), so
the Build path reaches `bundleDonePending` and prints it. Plan S1 test 7's
"no phase-1 refusal may say scan" assertion only covers `buildSupplyRefusal`,
so it does not see this string.

---

### I3 — "payload record order" is COMPLETION order; it diverges on an interleaved payload, and the operator-facing line asserts it unconditionally

`gui/multisig_build_payload.go:90-95` and `:305`;
`gui/bundle_flow.go:104-107`; `gui/sysw_session.go:126-149`.

`buildCosignerSupply` returns `mk1CosignerCards(g.cards)`, and
`bundleGatherer.cards` is documented as *"completed + verified, **in completion
order**"* (`gui/bundle.go:122`). Completion order equals record order only for a
payload whose chunk sets are contiguous.

Measured (records `A1, B1, B2, A2` — card A's first chunk arrives first):

```
supply[0] summary = "mainnet | m/48h/0h/0h/2h | fp b8688df1"   <- card B
supply[1] summary = "mainnet | m/48h/0h/0h/2h | fp 73c5da0a"   <- card A
```

So card **B** takes the lower slot. `@N` order is identity-bearing —
`md/encode_multisig.go:13-21`: *"Two callers supplying the same N keys in
DIFFERENT orders mint DIFFERENT, both valid, md1 cards with DIFFERENT
WalletPolicyId"* — and spec P0 item 5 rules *"Slot order is payload record
order."*

Three sites state the contract as absolute and are therefore wrong:

- `gui/sysw_session.go:135` — "nothing downstream may reorder them"
- `gui/bundle_flow.go:105` — "Fed in PAYLOAD RECORD ORDER and never reordered"
- `gui/multisig_build_payload.go:305` — prints "**in payload order**" to the
  operator, unconditionally, on the confirmation surface

`buildCosignerSupply`'s own doc comment does hedge ("which for a contiguous
payload is record order"), which means the divergence was seen and then
asserted anyway one layer up. With fingerprints OMITTED — the default — the
review screen shows `(no fp)` on every slot, so a wrong order is invisible in
every artifact the operator gets: that is exactly §0.1 clause 2's refuse side,
and here it is being announced as true instead.

Reachability: `me sysw pack` (`crates/me-cli/src/sysw/mod.rs:157-195`) preserves
caller order among public records, so interleaving depends on the order the
operator hands chunk files to `pack`. It is not the common shape, but the
format admits it and nothing rejects it.

**Fix (~5 lines):** carry each completed card's first-chunk record index and
sort by it before returning, or feed `offer()` grouped by `chunk_set_id` in
first-appearance order. Add the interleaved fixture as a test — the two-card
case above is sufficient.

---

## MINOR / NITS

**N1 — the flow's *consumption* of the total classifier is not itself total.**
`gui/multisig_build.go:76-94` pre-seeds `chosen` with **all** indices and then
handles `cosignerAutoFill` as the switch's *implicit default* (no `case`). The
function is total; its call site is not. A fourth outcome, or any future
disagreement between the two classify calls, silently takes the all-cards
default rather than being caught. Make it `case cosignerAutoFill:` (even empty)
plus a `default:` refusal — that is the shape the ruling's own comment asks for.

**N2 — a dead refusal arm with a self-contradictory message.**
`gui/multisig_build.go:99-103`: `picked` is all-`cardMK1` (filtered at :73) and
`len(picked) == open` always (both arms guarantee it), and every card in it
already survived `mk.Decode` inside `offerChunkedMK1`. So this arm is
unreachable — and if it were reached it would print *"The payload holds 2
cosigner key cards; this policy needs 2 cosigner key cards. Rewrite the payload
…"*, two equal counts and a wrong remedy.

**N3 — Back on the card picker abandons the whole Build flow.**
`gui/multisig_build_payload.go:242` returns `(nil, false)` → the flow returns,
discarding all five picked parameters. The comment says "Back abandons, as it
does on every other picker", but `buildParamPickFlow` steps back exactly one
stage. The gather does abandon, so it is defensible — just not what the comment
claims.

**N4 — the walk's selection loop has no per-card post-condition.**
`cmd/emu/walk_build_policy.js`, the `for (let taken = 0; taken < use; taken++)`
loop taps CONFIRM `use` times without a `waitFor` between taps. The same file's
`choose()` helper exists precisely because *"a wrong row does NOT fail loudly on
its own"*. A stray or early-auto-taken card would put a tap on the next screen.
Cheap to fix: `await waitFor(\`Use payload card ${taken + 1} of\`)` inside the
loop.

**N5 — duplicate keys reach the assembled set. CONFIRMED, and NOT S1's.**
The brief asked whether the duplicate refusal "survives" selection. Measured
answer: **there is no such refusal anywhere in the code**, and the plan already
knows — S2 test 4 `TestS2RefusesDuplicateKeysBeforeS4` says verbatim *"No
duplicate check exists anywhere in the code today"*. It has an owning phase
(S2, before any engrave completes), so it does not block S1. Recording the
reproduction so S2 does not have to re-derive it:

- `assembleBuildPolicy(p{N:3,K:2,SelfSlot:0}, selfXpub, selfFP, []mk.Card{cA, cA})`
  returns **no error** and a valid md1 (stub `cdb77b0d`).
- The **delivered** payload makes it a default-tap outcome, not a contrived one.
  `cmd/buildpayloadcards/main.go:53-58` packs `A@0, A@1, B@0, C@0` **plus
  masterA's mnemonic** (for S4's `both` slot). `A@0` is masterA at
  `m/48'/0'/0'/2'`, which is exactly `multisigSharedOrigin()`
  (`gui/multisig_build.go:463-466`). So: take the self seed from the payload
  (masterA) and accept the picker's default for card 1 — `USE THIS CARD` is row
  0 — and @0 and @1 hold the identical key. Review screen, verbatim from the
  probe:

  ```
  Slots @1 and @2 filled from the payload (cards 1 and 2 of 2, in payload order).
  Policy stub: f4fec97d
  Slots:
  @0  (no fp)
  @1  (no fp)
  @2  (no fp)
  ```

  `sortedmulti(2, K, K, X)` is spendable by K alone, and nothing on that screen
  says so. Worth S2 knowing the collision is in the shipped fixture.

**N6 — no test binds `takeAll`'s order to the flow.** M2 (reverse `takeAll`'s
output) was killed only by `TestSyswTakeAllYieldsEveryMDMKRecord`, the unit
test. The flow-level order test does not see it, for I1's reason. Fixing I1
fixes this too.

---

## Rulings check, one by one

| # | ruling | verdict |
| --- | --- | --- |
| 1 | `0..n`; no stage assumes `n-1`; exact-count on the ASSEMBLED set | **MET.** `classifyCosignerSupply` (`multisig_build_payload.go:138-147`) is the only count comparison; probed total over `state × have × open` — `state != loaded, have < open → refuse`, `have == open → autoFill`, `default → select`, no input falls through. `buildCosignerCards` unchanged. M3 (restore the feed-side exact-count refusal) turns 4 matrix cells red. |
| 2 | over-supply → bounded selection, record order, review announces which card filled which slot | **MET in the code, UNASSERTED at the flow level** — see I1; and the order claim is conditional — see I3. |
| 3 | under-supply/zero stays a refusal, names `me sysw pack`, never "scan" | **MET** for `buildSupplyRefusal` (6-row table, all rows assert the absence of "scan"; M9 kills it). Leaks at `bundle_flow.go:135` — see I2. |
| 4 | `takeAll` before `[compared]` refuses | **MET.** Guard inherited verbatim; M1 (drop `!s.compared`) is killed by `TestSyswTakeAllRefusesBeforeCompared/loaded_but_not_compared`. The test also runs the guard-free loop in place and proves it *would* have returned the card, so the refusal is not passing on an empty session. |
| 5 | §0.1 announcement on the confirmation surface; explicit choices announce nothing | **MET in behaviour** (observed in F-178's hand-drive and by `buildReviewLines` unit test; `buildProvenanceLines(nil, 0)` returns nil). **UNASSERTED at the flow level** — M18 deletes it and the full suite stays green. |
| 6 | F-175: S1 is recordless on the D-1 arm | **MET.** No gate record expected or produced; the four-part substitute is present — walk script, single-site needle test, `presented === 0`, and F-178 recording that D-1 did not reproduce. |

**Is `classifyCosignerSupply` actually total?** Yes — I could not construct a
falling-through tuple. The `switch` has no reachable gap over the integers, and
the second call site hardcodes `cosignerSourceLoaded` legitimately (the first
gate already established it). The totality hole is one layer up, at the call
site — N1.

**Is the walk's payload-sourcing claim non-circular?** Yes, by construction, and
I verified the construction rather than the transcript. `presentedCount`
(`cmd/emu/nfc.go:55-99`) increments **only** in `nfcSource.set`, i.e. only from
`window.shNFC.set()`; it has no reset by deliberate design; and
`cmd/emu/nfc_presented_test.go` asserts it reaches 2 after two presentations, so
it is not hardcoded. `SyswReader()` (`cmd/emu/platform.go:313`) returns
`embeddedSyswReader`, a different type that never touches `nfcSource`, so
loading the payload cannot move the counter. `bundleGatherFlow` has exactly two
paths into `scr.g.offer` — the `ctx.syswBundleSeeds` loop and the scanner
channel. So `cardsGathered: 4` with `presented: 0` can only mean the payload
fed them. The claim holds.

**Does the diff explain why D-1 no longer reproduces?** Partly, and I would not
let S2 lean on it. Pre-S1 a reader-less machine could gather at most the one
`syswOffer` seed, so for `n ≥ 3` the flow could never satisfy
`buildCosignerCards(cards, p.N-1)`; pressing Done with zero cards showed *"No
complete cards yet — scan a card's chunks first"* and left the operator in the
gather with only Back. That is a genuine dead end and S1 removes it. But D-1 as
filed is a **blank screen** — a rendering failure — and this diff touches no
drawing code. So I read the non-reproduction as consistent with S1 having fixed
a *different* dead end, and I would weight S2's search toward F-178's shapes 2
(hardware) and 4 (typed seed), not treat the payload path as cleared.

---

## WHAT I RAN

All commands with `export PATH="/nix/var/nix/profiles/default/bin:$PATH"`, from
`/scratch/code/shibboleth/seedhammer`, exit codes read directly (never through a
pipe).

```
nix develop --command go test ./...          → TRUE_EXIT=0, 51 "ok", 0 FAIL/panic lines
nix develop --command go vet ./...           → VET_EXIT=1, 6 findings, ALL the
                                                pre-existing testing.ArtifactDir
                                                go1.26-vs-go1.25 baseline
nix develop --command gofmt -l ./            → GOFMT_EXIT=0, no files listed
nix develop --command tinygo build -size full -print-stacks -o /dev/null \
  -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks \
  ./cmd/controller                           → TINYGO_EXIT=0
                                                flash 1346916, ram 61908
                                                (matches the commit message exactly)
```

Test-name inventory of the S1 subset (18 matrix cells present and running):
`TestPayloadCardCountIsIndependentOfN/n={2..5}/cards={0..n}` = 18 subtests, plus
`TestSyswTakeAll{Yields,Refuses}`, `TestBuildGathersEveryCosignerFromPayload`,
`TestBuildIgnoresMd1RecordsInThePayload`, `TestBuildSlotOrderIsPayloadRecordOrder`,
`TestBuildRefusesMoreCardsThanOpenSlots`, `TestUnderSupplyRefusalNamesTheHostRoute`
(6 rows), `TestBuildOverSupplySelectionIsWalkable`,
`TestBoundedSelectionCannotEndShort`, `TestCosignerCardFixturesAreDistinctAndComplete`,
`TestBuildFlow_GatherBeforeSeed` (2 arms),
`TestMultisigTakesItsFirstCardFromThePayload` (2 arms),
`TestBuildFlowNeedlesHaveExactlyOneProductionSite`, `TestDecoyNeedlesAreStillAmbiguous`.

**Mutation matrix — 18 mutations, applied in a throwaway `git worktree` and
reverted.** A mutation that did not compile was not counted as a proof; none
failed to compile.

| # | mutation | outcome |
| --- | --- | --- |
| M1 | `takeAll` drops the `[compared]` guard | killed |
| M2 | `takeAll` reverses record order | killed (unit test only — N6) |
| M3 | restore the `n-1` exact-count refusal on the FEED | killed (4 matrix cells + 2 tests) |
| M4 | md1 filter removed | killed |
| M5 | provenance dropped from `buildReviewLines` | killed |
| M6 | picker returns cards in reverse selection order | killed |
| M7 | gather feeds only the FIRST payload seed (the n-1 regression) | killed (5 tests) |
| M8 | off-by-one in the announced card number | killed |
| M9 | refusal names a card reader instead of the host route | killed (10 rows) |
| M10 | `autoFill` / `select` swapped | killed (8 matrix cells) |
| M11 | the PRE-GATHER refusal removed | killed |
| M12 | seeds never cleared after the gather consumes them | killed (both arms) |
| M13 | bounded-selection auto-take short-circuit removed | killed |
| **M14** | **flow discards the operator's selection** | **SURVIVED `go test ./...`** |
| **M15** | **every cosigner slot gets payload card 1 (duplicate keys)** | **SURVIVED `go test ./...`** |
| **M16** | **flow announces the wrong slots (`selfSlot` → 0)** | **SURVIVED** |
| **M18** | **flow passes NO provenance — announcement deleted** | **SURVIVED `go test ./...`** |
| M17 | (probe, not a mutation) interleaved payload ordering | reordered — see I3 |

13 killed / 4 survived. Every survivor is on the same seam: the flow's wiring.

**Behavioural probes** (throwaway `_test.go` files in the worktree, removed
after): interleaved-chunk ordering (I3); duplicate-cosigner acceptance and
self-key-duplication acceptance (N5); auto-fill-arm and over-supply-arm screen
text at both the default and `sh2DisplaySize` displays (I2); payload-card list
paging at 480×320; pending-chunk-set classification.

Worktree removed; `git status --porcelain` on the repo is empty and
`git worktree list` shows only the main tree at `3a0ac6e`.

---

## WHAT I COULD NOT CHECK

Named explicitly, because an unchecked area named is worth more than a clean
bill implying coverage I did not have.

1. **The emulator walk itself was not executed.** `walk_build_policy.js` runs in
   a browser against `emu.wasm`; I have no browser and did not build or serve
   the wasm. Its reported output (`needlesProven` ×6, `presented: 0`,
   `cardsGathered: 4`, `openSlots: 2`, `selected: true`, `ok: true`) is the
   implementer's transcript and I take it on trust. What I *did* verify
   independently is the thing that makes it meaningful — that
   `presented === 0` is non-circular (see the Rulings check) and that the two
   new needles are single-site (`TestBuildFlowNeedlesHaveExactlyOneProductionSite`
   passes, and the decoy count for `"First card from where?"` was correctly
   dropped 3 → 2).
2. **No hardware.** Nothing was run on a physical SH2.
3. **The drawing layer.** Every screen assertion I made — mine and the suite's —
   is `shScreen()` text. Per F-151 a text assertion cannot see a body that fails
   to raster. I2's "the auto-fill arm shows only the scan prompt" is a text
   finding; I did not raster it.
4. **`me sysw pack`'s real-world record ordering.** I read
   `crates/me-cli/src/sysw/mod.rs:157-195` and confirmed caller order is
   preserved among public records, but I did not enumerate how the CLI collects
   files, so I cannot say how often I3's interleaving actually occurs in the
   field — only that nothing rejects it.
5. **S0b, the needle gate and the oracle comparison** were not re-reviewed
   (settled per the brief) beyond running their tests as part of `go test ./...`.
6. **`sysw.MDMKUnconfirmed` / `[mdmk-decode]`.** `takeAll` ignores the
   `unconfirmed` flag. I confirmed `take` ignores it too and that the flag's only
   consumers are the load-screen flag surfaces (`gui/sysw_load.go:236,258-278`),
   so this is consistent and not a regression — but I did not analyse whether a
   cosigner set built from records flagged `unconfirmed` deserves its own
   warning. That may be worth a follow-up question rather than a finding.
7. **k-of-n shapes beyond the walked one.** `n=3, k=2, @0, fp omitted, wsh` is
   the only shape any walk drove; the 18-cell matrix covers `n ∈ 2..5` at the
   classifier level only, not through the UI.

---

## Recommended disposition

- **I1** — one test extension, in `TestBuildOverSupplySelectionIsWalkable`.
  Blocks: it is the assertion that makes S1's own deliverable regression-proof,
  and it kills all four surviving mutations.
- **I2** — S1 owns it by the plan's own bullet and the spec's "ruled here, not
  deferred". Smallest compliant shape: give the Build path its payload-supplied
  review on **both** arms (show the `Payload cards` screen before auto-fill too),
  or make `tally()`'s third line source-aware. The title stays S2's.
- **I3** — order the assembled cards by first-chunk record index, add the
  interleaved fixture, and correct the three absolute comments. If the operator
  prefers to rule that interleaved payloads are out of scope, then the
  unconditional "in payload order" line and the two "never reordered" comments
  must still be corrected, because a record that overstates is the defect class
  this cycle keeps paying for.
- **N1–N4, N6** — fix inline or file with owning phases. **N5** is already
  S2-owned; the reproduction above should be pasted into that follow-up so S2
  does not re-derive it.
