# S1 fold re-review — `93ad031`

Scoped re-review of the fold of `s1-execution-review-2026-08-15.md` (3 Important,
0 Critical). One question only: **did the fold fix each finding, and did it
introduce a new defect?** Not a fresh audit. `0..n`, F-175, F-178 and N5
(duplicate keys, S2-owned) were not re-opened.

Reviewer: same independent context as the original review. Fold report
(`5027fcc`) read, then checked rather than believed.

---

## VERDICT

**1 Important, 0 Critical.** Not closed.

All three findings are **fixed in behaviour** and I verified each by execution,
not by reading the call graph. The four mutations that survived the first review
all die now. The one Important is a **new defect the fold introduced in its own
fix for I3**: the grouping call is guarded at the function, not at the seam, so
dropping it from `buildCosignerSource` leaves the full suite green — the exact
component-vs-seam gap that I1 was filed for, reproduced one layer over.

It is a three-line test change and I have already written and run the
replacement. Two Minors accompany it, one of which the coordinator asked me to
judge explicitly.

---

## Per finding

### I1 — the flow's wiring is asserted ✅ **FIXED**

Re-applied all four original survivors myself, at the fold's new line numbers,
each against the **full** suite (`go test ./... -count=1`, exit code read
directly, throwaway worktree):

| # | mutation | site | before | after |
| --- | --- | --- | --- | --- |
| M14 | discard the operator's selection | `multisig_build.go:103` | SURVIVED exit 0 | **KILLED** |
| M15 | every cosigner slot gets card 1 (duplicate keys) | `:123` | SURVIVED exit 0 | **KILLED** |
| M16 | announce the wrong slots (`selfSlot`→0) | `:137` | SURVIVED | **KILLED** |
| M18b | announcement deleted from the flow | `:183` | SURVIVED exit 0 | **KILLED** |

All four die to `TestBuildOverSupplySelectionIsWalkable`. I read the test rather
than trusting the name: it drives a **non-contiguous** USE(1)/SKIP(2)/USE(3),
self at **@1**, fingerprints **INCLUDED**, pages the review with
`readReviewPages`, and asserts positively (`cards 1 and 3 of 4`, `Slots @0 and
@2`, fixtures 0 and 2's fingerprints present, `selfFP` present) **and
negatively** (`cards 1 and 2 of 4` absent, skipped fixture 1's fingerprint
absent). The negative assertions are what make M14/M15 observable, and the
uniqueness guard is correctly scoped to the four keys the test names — I checked
the stated reason and it holds: fixtures 0 and 3 share masterA's fingerprint by
design, matching the delivered payload's own `A@0`/`A@1` collision.

The fourth-master story checks out too: `testSeedPhrase` is masterB = fixture 1,
the skipped card, so the absence check really was unsatisfiable. Caught by
failing on correct code, which is the useful direction.

**N6 also confirmed fixed:** M2f (`takeAll` reverses record order) now dies to
**two** production tests — `TestSyswTakeAllYieldsEveryMDMKRecord` and
`TestBuildOverSupplySelectionIsWalkable` — where before it died only to the unit
test.

### I2 — SPEC P0 item 6 on the default arm ✅ **FIXED in substance**; the second half is inert — see Minor F2

**(a) The ruled item lands, measured on screen at `sh2DisplaySize` (480×320),
auto-fill arm (n=3, exactly 2 cards):**

```
gather:  "EngraveBundle md1descriptors:0 mk1keys:2 Donewhenyouhavereviewedthese."
review:  "Payloadcards  Thepayloadsupplied2cosignerkeycards.
          Thispolicyhas2openslot(s).
          1.mainnet|m/48h/0h/0h/2h|fp73c5da0a
          2.mainnet|m/48h/0h/0h/2h|fpb8688df1
          Allofthemfilltheopenslots,inthisorder."
```

Before the fold this arm had no such screen at all. M19 (auto-fill arm loses its
review) is **KILLED** by `TestBuildIgnoresMd1RecordsInThePayload`. The claim that
this does not re-introduce selection on the auto-fill arm is correct — `selecting`
only changes the closing line, and `buildCosignerPickFlow` is still reached only
from `case cosignerSelect`.

**(b)** The `FeatureNFC` keying works where the bit is false, but no shipping
platform makes it false. Full detail in Minor F2 below; it does not block,
because (a) is what P0 item 6 rules and (a) landed.

### I3 — record order ✅ **behaviour FIXED**, ❌ **the fix is not guarded at the seam** → Important F1

Everything the fold claims about the *behaviour* checks out, by execution:

- **Permutation on every input I could construct.** I ran grouping over
  interleaved, duplicate records, a card whose chunks never complete, `md1 +
  junk + cards`, empty, single, and reversed-within-card. Multiset equality held
  in all 7; within-card chunk order was left alone. Key construction is sound —
  `mk:`/`md:`/`solo:` prefixes cannot collide, and every record lands in exactly
  one group, so it cannot drop or invent.
- **The seam does group.** `buildCosignerSource` over `A1 B1 B2 A2` yields card
  **A** first (`fp 73c5da0a`), where the raw feed yielded card B.
- **No other consumer reads the ungrouped feed.** `takeAll` has exactly one
  caller (`buildCosignerSource`), which has exactly one caller
  (`buildMultisigPolicyFlow`). Verified by grep over the whole repo.
- **The INCONCLUSIVE guard is real.** `TestInterleavedPayloadStillAssemblesInRecordOrder`
  Fatals if the raw feed stops putting card B first, so the fix cannot pass by
  the defect vanishing. I confirmed the raw arm still reproduces.
- **The three comments now name where the guarantee is obtained** — I read all
  three (`sysw_session.go:137-146`, `bundle_flow.go:127-136`,
  `multisig_build_payload.go:150-155`) and each says what its own layer can and
  cannot promise, and points at `groupRecordsByCard`.

---

## IMPORTANT

### F1 — I3's fix is guarded at the FUNCTION, not at the SEAM; dropping the call leaves the full suite green

`gui/multisig_build_payload.go:74` (the call) and
`gui/multisig_build_payload_test.go:188-280` (the guard).

The fold reports "M20 — `groupRecordsByCard` becomes the identity → FAIL,
suite exit 1". That claim is **true**, and I reproduced it: mutating the function
*body* to `return records` kills
`TestInterleavedPayloadStillAssemblesInRecordOrder`.

But the mutation that matters is the other one, and the fold did not run it:

```
### M20-seam: gui/multisig_build_payload.go:74
-	return groupRecordsByCard(records), cosignerSourceLoaded
+	return records, cosignerSourceLoaded
    → *** SURVIVED full suite (go test ./... exit 0) ***
```

Cause, measured: every call in that test is `groupRecordsByCard(interleaved)`
applied **by the test itself** (lines 233, 250, 268, 277). The test never routes
records through `buildCosignerSource`, so the only thing binding the fix to the
production path is one unasserted line.

This is the **same shape as I1**, reproduced inside I1's own fold: a correct
component with a fully-tested contract, and the call that joins it to the flow
asserted by nothing. Its consequence is not cosmetic — dropping that call
restores I3 exactly: an interleaved payload assigns `@N` in completion order, the
review still announces "in payload order", and with fingerprints omitted (the
default) every slot renders `(no fp)`, so it is invisible in every artifact. And
`buildCosignerSource` is documented at `:48-58` as the seam **the later NFC plan
is expected to reopen** — the likeliest future edit site in this file.

**Fix — three lines, already written and passing.** Drive the seam instead of the
function. This exact probe passed against `93ad031` and goes red under M20-seam:

```go
ctx := NewContext(newPlatform())
ctx.sysw = sessionHolding(interleaved...)          // A1 B1 B2 A2
got, state := buildCosignerSource(ctx)             // <- the seam, not the helper
// ... then assert card A leads through buildCosignerSupply(got), as today
```

**Verification command for the fold:** re-run M20-seam above; it must go red.
That single check discharges F1.

---

## MINOR

### F2 — the `FeatureNFC` keying is inert on every shipping platform, and the comment justifying it states a false premise

*The coordinator asked directly: honest boundary, or hole? **Hole, and larger
than the report names.***

The report's limitation says the no-reader wording cannot be exercised in the
emulator "because the emulator HAS a reader". True, and it omits the bigger half.
There are exactly **three** `Features()` implementations in the repo:

| platform | value | takes the no-reader branch? |
| --- | --- | --- |
| `cmd/controller/platform_sh2.go:568` — the real SeedHammer II | `p.feats`, which gets `\|= gui.FeatureNFC` **unconditionally** at `:313` | **no** |
| `cmd/emu/platform.go:343` | `return gui.FeatureNFC` | **no** |
| `gui/gui_test.go:430` — the Go test platform | settable | yes |

`platform_sh2.go:306-313` says why, in terms: *"The ST25R3916 is soldered to
every board, so the reader is unconditional here."* So on the machine this stage
exists for, the gather still reads **"Scan a card, or Done."**, still says *"No
complete cards yet — scan a card's chunks first."*, and still says *"Dropped an
incomplete card — scan all its chunks to include it."* The branch is taken by the
Go test platform and by nothing else that exists.

The premise is wrong too, and the fold has now written it into the code.
`gui/bundle_flow.go:83-86` newly asserts *"phase-1 hardware has no reader, so on
the machine this stage exists for, the gather instructed an action the operator
could not perform."* SPEC §3.1 is titled **"Dropping NFC removes the hardest
stage, not a feature"**, and its "no reader" facts are about the *host test
platform* — spec line 175: *"The host test platform has no reader either."* Phase
1 drops NFC from **scope**, not from the board. That is a new comment stating a
condition that is false, which is the class this project has been burnt by
repeatedly.

**Why this is Minor and not Important:** no wrong behaviour ships. On a machine
with a reader the scanner really is live in the gather
(`startScanner(ctx, ctx.Platform.NFCReader())`, unchanged) and a scanned card
really is accepted — the flow's own comment at `multisig_build.go:74-75` says so
— therefore "Scan a card, or Done." is **true** on the SH2. And P0 item 6's
ruled substance is delivered by I2(a), which does land on every platform.

**But the operator should decide the residue**, because it turns on a ruling I
was told not to re-litigate. If the intent is *"the Build-policy gather must
never say 'Scan a card', regardless of the machine"*, then the predicate is
wrong — the property is flow-scoped, not machine-scoped — and this becomes
Important. If the intent is *"say it only where scanning works"*, the code is
right and only the comment at `bundle_flow.go:83-86` needs correcting to match
SPEC §3.1.

Either way I recommend correcting that comment now: it is a false claim about
hardware sitting directly above a branch, and the S1 commit message carries the
same sentence.

### F3 — the "absence over every string the screen can produce" guard reaches 2 of the 4 conditioned sites

`gui/bundle_flow_test.go:42-88`.

`TestGatherScreenNeverSaysScanWithoutAReader` iterates `tally()` and every
`feedback()` status — but the two Done-gate messages are inline string literals
inside `bundleGatherFlow` (`:158-181`) and are unreachable from either. Measured:

```
### M21 no-reader PENDING-DROP reverted to "scan all its chunks to include it."
    → *** SURVIVED full suite (exit 0) ***
### M22 no-reader EMPTY-DONE reverted to "scan a card's chunks first."
    → *** SURVIVED full suite (exit 0) ***
```

The test's own log line claims *"the closing tally line and the two Done-gate
errors are the three sites; all are keyed on FeatureNFC"* — but the two Done-gate
errors are precisely what it cannot see. (There are also four conditioned sites,
not three: `tally()`, `feedback(bundleRefusedSingleMK1)`, and the two `showError`
literals.) Cheap fix: hoist the two messages into small reader-aware helpers and
add them to the `every` sweep, so the absence assertion covers what it claims.

---

## NEW DEFECTS INTRODUCED BY THE FOLD

- **F1** (Important) — the seam-shaped hole in I3's own regression guard. New
  with the fold: the code it fails to protect did not exist before.
- **F2** (Minor) — a new comment at `gui/bundle_flow.go:83-86` asserting a
  hardware fact that `cmd/controller/platform_sh2.go:313` and SPEC §3.1
  contradict.
- Nothing else. I re-checked the restructured flow (`multisig_build.go:76-131`):
  the now-exhaustive switch is correct, `chosen` is assigned on every
  non-returning arm, `chosen, ok =` is a plain assignment against the
  pre-declared `var chosen []int`, the auto-fill review runs before `picked` is
  built, and N2's replacement message ("Couldn't read the cosigner key cards from
  the payload.") is right for the only failure that can actually reach it.
  N3 and N4 are correct as described.

---

## WHAT I RAN

```
nix develop --command go test ./...      → TEST_EXIT=0, 51 ok, 0 FAIL/panic
nix develop --command go vet ./...       → VET_EXIT=1, 6 findings, ALL ArtifactDir
                                            (grep -v ArtifactDir returned nothing)
nix develop --command gofmt -l ./        → GOFMT_EXIT=0, no files
nix develop --command tinygo build -size full -print-stacks -o /dev/null \
  -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks \
  ./cmd/controller                       → TINYGO_EXIT=0, flash 1349428, ram 61908
```

Every number the fold report claims, **confirmed**: 51/0, the 6-finding
ArtifactDir baseline with zero non-baseline findings, gofmt clean, and flash
**1,349,428** exactly (+2,512 over S1's 1,346,916).

**Mutations — 10 applied, full suite each, exit codes read directly, throwaway
worktree, removed afterwards. A mutation that did not compile was not counted.**

| # | mutation | result |
| --- | --- | --- |
| M14 | discard the operator's selection | KILLED |
| M15 | every cosigner slot gets card 1 | KILLED |
| M16 | announce the wrong slots | KILLED |
| M18b | announcement deleted from the flow | KILLED |
| M19 | auto-fill arm loses its payload review | KILLED |
| M20-impl | `groupRecordsByCard` **body** → identity (the fold's own form) | KILLED |
| **M20-seam** | **grouping call dropped from `buildCosignerSource`** | **SURVIVED** |
| **M21** | **no-reader pending-drop says "scan"** | **SURVIVED** |
| **M22** | **no-reader empty-Done says "scan"** | **SURVIVED** |
| M2f | `takeAll` reverses record order (N6) | KILLED by 2 tests |

7 killed / 3 survived. The fold reported 7/7 — its seven are all genuinely
killed; the three survivors are mutations it did not apply.

**Behavioural probes** (throwaway `_test.go`, removed): auto-fill-arm screens at
480×320; grouping-is-a-permutation over 7 adversarial inputs; the seam's grouped
output. Repo verified clean afterwards (`git status --porcelain` empty,
`git worktree list` shows only the main tree at `93ad031`).

---

## WHAT I COULD NOT CHECK

1. **The emulator walk was again not executed.** No browser, no wasm build. The
   fold claims a re-run on a rebuilt `emu.wasm` (6 needles, `presented: 0`,
   `cardsGathered: 4`, `selected: true`). I take it on trust, as before. Its
   reported `gatherScreen` still reading "Scan a card, or Done." is **consistent
   with the code I read** — `cmd/emu/platform.go:343` reports `FeatureNFC` — so
   that detail at least corroborates rather than contradicts.
2. **No hardware.** The F2 conclusion about the real SH2 is read from
   `cmd/controller/platform_sh2.go:306-313`, not observed on a board.
3. **Rastering.** All screen evidence here is `shScreen()`-style text; per F-151
   a text assertion cannot see a body that fails to draw. The auto-fill review's
   *content* is verified, its *drawing* is not.
4. **The four other `bundleGatherFlow` callers** (`bundleFlow`,
   `supplyMultisigPolicyFlow`, `multisig_verify`, `singlesig_verify`) now inherit
   the reader-aware strings. I confirmed the change is behaviour-preserving where
   `FeatureNFC` is set — which, per F2, is every shipping platform — so their
   on-screen text is unchanged in practice. I did not walk those four flows.
5. **Out of scope by instruction and not examined:** `0..n`, F-175, F-178, N5.

---

## Disposition

- **F1** is the only blocker and is a three-line test change; the replacement
  probe is written above and passes. Discharge it by re-running M20-seam and
  confirming red. This does not need a further full round from me.
- **F2** needs an operator decision on the predicate, plus a comment correction
  either way.
- **F3** is a cheap widening of an existing test.
