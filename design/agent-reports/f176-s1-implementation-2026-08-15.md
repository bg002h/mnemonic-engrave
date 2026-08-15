# F-176 + S1 implementation report (2026-08-15)

Brief: (1) fix F-176 in the primary `descriptor-mnemonic` by adding
`md encode --origin @i=<path>`; (2) implement S1 in the fork. Work solo, TDD,
verify inline, commit but do not push.

Outcome in one line: **phase 1 STOPPED on a false premise and F-176 was
withdrawn; phase 2 (S1) landed complete, with a green emulator walk — and S1's
gate hit a THIRD outcome the plan does not have an arm for.**

---

## PHASE 1 — STOPPED. F-176's premise is false, measured.

**No code was written in `descriptor-mnemonic`. The checkout is clean at
`5a0a4f41`, untouched.**

F-176 (and the brief) state: *"`md encode` cannot author per-key origins, so a
divergent-origin md1 cannot be produced by the primary at all."* That is wrong.
`md encode` authors per-key origins today — not through a flag, through the
**template placeholder syntax**, which the audit's three probes never tried.

Reproduction, against the pinned oracle (`md 0.13.0`, commit `5a0a4f41`, the
exact `oracle/pins.json` pin):

    md encode "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))" \
      --key "@0=xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf" \
      --key "@1=xpub6EAMBJLn1jiquajTsNRkZXU1oKnA4WJMNvcz4FRR4QmFKdfHxJVvfRLoysWfcc16AMTR4CoMD8UNjvs9JtbsLeuLwpTczgq8zuuERnp8YZF" \
      --group-size 0 --force-chunked --json

emits 4 chunks; `md decode --json` returns
`path_decl {tag: "Divergent", data: ["m/48'/0'/0'/2'", "m/48'/0'/1'/2'"]}`
with each path on the right slot; `md verify … --template … --key @0= --key @1=`
over the same four strings prints `OK`, exit 0.

**Stronger than "satisfiable" — it is already byte-identical.** The fork's
`md.EncodeMultisig` in `OriginDivergent` mode over the same two xpubs and the
same two origins emits the **same four strings, character for character**
(`stub=4cb7f1a8`, 4/4 chunks; run as a scratch test in `gui`, since deleted).
So S5's byte-comparison gate holds **today**, with the pinned binary, needing no
upstream change, no release, and no re-pin. The recorded decode-equivalence
fallback is moot.

Why the entry was filed: three probes of the wrong surface (`--key` with a
bracketed origin; a concrete-key descriptor; `--help` for a flag) plus one probe
of a template that carried no inline origins read as consensus. **Absence of a
FLAG was mistaken for absence of the CAPABILITY** — while `make_path_decl`
(`crates/md-cli/src/parse/template.rs:495-510`) emits `Divergent` whenever the
inline per-`@i` origins differ, and `emit_pathless_advisory`'s doc-comment
(`crates/md-cli/src/cmd/encode.rs:180-183`) names the feature in prose.

The two design questions the brief asked me to settle are already answered by
the shipped mechanism, both measured:

- **`--path` + inline divergent origins:** `--path` wins and flattens to
  `Shared`, documented on the flag itself (`crates/md-cli/src/main.rs:93-95`).
- **Partial specification** (origin on `@0`, none on `@1`): accepted; the
  unspecified `@i` gets a depth-0 empty origin and the encoder emits the
  pathless advisory on stderr.

**Judgment call, and the rule that decided it.** The brief's STOP condition —
*"If a premise the plan states turns out to be false when you MEASURE it, stop
and report rather than improvising around it"* — plus "don't gold-plate". With
the gap gone, `--origin @i=` is pure ergonomic sugar that still costs a version
bump, a CHANGELOG entry, and a **cross-repo manual lockstep**
(`mnemonic-toolkit docs/manual/src/40-cli-reference/42-md.md`, gated by
`tests/lint.sh flag-coverage`). Landing a cross-repo release chain on a
justification that had just evaporated is the improvisation the rule forbids, so
I stopped and recorded instead.

**Landed:** `mnemonic-engrave` `f51d8cc` — *"followups: F-176 WITHDRAWN"*.
F-176 is marked withdrawn with the reproduction verbatim, the byte-identity
result, the two measured behaviours above, the reason the false premise formed,
and an explicit *"do not re-file it as gating"*. The original entry is preserved
below the correction.

---

## PHASE 2 — S1 landed. Fork commit `3a0ac6e`.

*"The payload supplies the whole cosigner set."*

### What changed

| file | what |
| --- | --- |
| `gui/sysw_session.go` | `takeAll(class)` — every record of a class in payload record order, inheriting `take`'s `!loaded \|\| !compared` refusal. Returns `(nil, true)` for "compared, none present" so a caller can tell that from "not allowed to look" |
| `gui/multisig_build_payload.go` *(new)* | the whole S1 surface: `buildCosignerSource` (the seam), `buildCosignerSupply`, `mk1CosignerCards`, `classifyCosignerSupply`, `buildSupplyRefusal`, `buildCosignerPickFlow`, `buildCosignerOrigins`, `buildProvenanceLines` |
| `gui/multisig_build.go` | the `syswOffer` single-card seeding replaced by the whole-set feed + resolution; `buildReviewLines`/`buildReviewFlow` grew a `provenance` parameter |
| `gui/gui.go`, `gui/bundle_flow.go`, `gui/multisig.go` | `ctx.syswBundleSeed string` → `syswBundleSeeds []string`, fed to `offer()` in record order |
| `cmd/emu/walk_build_policy.js` | the payload leg + the selection leg; two new needles |
| `cmd/emu/needle_test.go` | two new single-site needles pinned; the `"First card from where?"` decoy re-pinned 3 → 2 |

### The eight plan tests, all written before the code

1. `TestSyswTakeAllYieldsEveryMDMKRecord` — three records yield all three, in
   order; `take` still first-only; "compared but empty" ≠ refusal.
2. `TestSyswTakeAllRefusesBeforeCompared` — **mutation-checked in place**: the
   sub-test runs the guard-free record loop and proves it *would* have returned
   the card, so the refusal cannot pass because the session was empty.
3. `TestBuildGathersEveryCosignerFromPayload` — n=3, two multi-chunk cards,
   `NFCReader() == nil` asserted as the INCONCLUSIVE guard so "zero scans" is
   structural.
4. `TestBuildIgnoresMd1RecordsInThePayload` — md1 **first** in the record list;
   asserts the md1 reaches the tally (so the test is not vacuous), that
   `buildCosignerCards` really would refuse it, and that the flow reaches seed
   entry anyway.
5. `TestBuildSlotOrderIsPayloadRecordOrder` — self at **@1** and a
   **non-contiguous** selection (cards 1 and 3), asserted on `md.SlotInfo`
   fingerprints from the encoder's own output, then on the review lines.
6. `TestBuildRefusesMoreCardsThanOpenSlots` — **re-scoped**: the feed classifies
   4-for-2 as `cosignerSelect`, not a refusal; the exact-count refusal still
   exists on the assembled set and is proved to be a backstop selection can
   never trip.
7. `TestUnderSupplyRefusalNamesTheHostRoute` — six rows incl. **zero cards**,
   no-payload and not-compared; each asserts `me sysw pack` is named, that the
   word "scan" never appears, and that the count reads grammatically.
8. `TestPayloadCardCountIsIndependentOfN` — n ∈ 2..5 × cards 0..n =
   **18 cells, 8 assembled, 10 refused by name, 0 fell through**. The mutation
   is **run**: restoring the feed-side exact-count refusal flips 2 of 5 n=3
   rows, and the delivered payload's own cell (4 cards, 2 slots) is asserted to
   be non-refusing live and refusing under the mutant.

Plus two I added because the ruling needed them:
`TestBuildOverSupplySelectionIsWalkable` (the flagship cell through the real
screens) and `TestBoundedSelectionCannotEndShort` (skip cards 1 and 2 of 4 for
2 slots → the remaining two are taken without asking, in record order).

Fixtures (`multisig_build_payload_testdata_test.go`) are derived through the
device's own path exactly as `cmd/buildpayloadcards` does — real BIP-48 depth-4
xpubs, real fingerprints, multi-chunk sets — and are guarded by their own test
for distinct csid and distinct xpub. `mk1CardA`/`mk1CardB` were deliberately
**not** reused: they share one xpub, so a policy built from them carries
duplicate keys and slot order is unobservable.

### Judgment calls, and which rule decided each

1. **Refuse BEFORE the gather, not after.** With zero or too few cards the
   shared `bundleGatherFlow` traps the operator on *"No complete cards yet —
   scan a card's chunks first"*, which phase-1 hardware cannot obey. Decided by
   audit row 2 + §0.1 clause 1: the refusal stays, but it must name a route that
   exists. Cost: `buildCosignerSupply` assembles the payload's records once for
   the pre-check and `bundleGatherFlow` assembles them again — one pure function
   over one input, run twice, documented as such. The alternative was a second
   insertion path, which `gui/bundle_flow.go:100-103` forbids.
2. **The source picker is gone; the seam is not.** `buildCosignerSource` is now
   the single place answering "where does a cosigner key come from", with the
   payload as phase 1's only answer. Decided by the plan's §5.1 note. A
   one-option `Input` screen was dropped as a tap that teaches nothing.
3. **Selection is a per-card USE/SKIP crawl, not a slot assignment.** Decided by
   audit row 1's *"preserving payload record order among the selected (no
   reorder)"* — a picker that let the operator assign cards to slots would be a
   second way to decide policy identity, and only one of them is announced. The
   crawl is bounded twice: it stops at `open`, and when the remaining cards equal
   the remaining slots it takes them without asking.
4. **The announcement goes FIRST on the Policy Review, above the stub.** Decided
   by §0.1 clause 3 (announce on the confirmation surface itself). Auto-fill
   announces too — it is still an assumption — while an empty origin list
   announces nothing, per §0.1's corollary.
5. **Three refusal states, not one.** No-payload / not-compared / too-few get
   different texts. "Collapsing them sends two of three operators to the wrong
   place."
6. **Two existing tests changed, both because behaviour changed by ruling, and
   both keep their property.** `TestBuildFlow_GatherBeforeSeed` grew two arms
   (no-payload now refuses naming the host route; with a payload the gather runs
   and Back still types no seed). `TestMultisigTakesItsFirstCardFromThePayload`
   split: supplied-policy keeps its picker, built-policy asserts the picker is
   **gone** and the cards reach the tally anyway.

### Two things only RUNNING found

- **`SyswReader()` is resolved once, by the caller of `syswLoadFlow`.** The boot
  offer is on screen before any script runs, so its reader is chosen before
  `shSysw` can speak. The first walk attempt loaded the RECORDS blob and refused
  for want of cosigner cards — a correct refusal on a machine that really held
  no cards. The driver now skips the boot offer and loads from the `Load Payload`
  carousel entry. Written into the script.
- **The em-dash draws as nothing** in `poppins.Regular16` — the F-78 `·` defect
  again. The picker's lead read `"Card1of4useit?"` on screen. Changed to
  `"Use payload card 2 of 4?"`. **Note for a later cleanup:** existing operator
  strings already contain em-dashes (e.g. `multisigBuildExperimentalWarning`'s
  body), so they render with a missing word-break today. Pre-existing, cosmetic,
  not S1's to fix — but worth a Minor.
- Also found by running: the cards payload carries a `ClassMnemonic`, so
  §3.3.3's F1 flag fires a **Payload Warnings** screen plus a KEEP/UNLOAD
  choice between the digest and the carousel. The driver now waits for both.

### The emulator walk — RUN, green

`walk_build_policy.js` driven in a real browser against a freshly built
`emu.wasm` on a fresh port. Verbatim result:

    {
      "elapsedSec": 13, "carouselHops": 9,
      "params": {"n":3,"k":2,"selfSlot":0,"includeFp":false,"use":2},
      "needlesProven": ["Supply or build a policy?","Choose policy type",
                        "How many keys (n)?","Which slot is your key?",
                        "Payload cards","Use payload card"],
      "presented": 0, "cardsGathered": 4, "openSlots": 2, "selected": true,
      "gatherScreen": "EngraveBundlemd1descriptors:0mk1keys:4Scanacard,orDone.",
      "screen": "Wherefrom?FROMPAYLOADTYPEITSCANInputSeed",
      "gatherTextIsNotEvidence": true, "ok": true
    }

Four payload cards in the gather, **zero records across the NFC reader**,
over-supply narrowed by bounded selection, six single-site needles proven.
F-175's D-1-arm substitute is therefore satisfied on three of its four items:
the walk script, the single-site needle (`needle_test.go` green), and
`shNFC.presented() === 0`. **No gate record was emitted** — F-175 rules S1
recordless on this arm and `ParseWalk` refuses an empty census by design.

---

## ⚠ STOPPED ON — S1's gate has a THIRD outcome, and it is gating for S2

The plan's S1 gate is *"either the flow completes an engrave, or D-1 reproduces
and is captured as a failing test"*. **Neither fired.** S1 does not engrave (plan
§3 preamble; giving it a tail was explicitly rejected), and **D-1 did not
reproduce.**

Measured, by hand-driving the same live session past where the walk stops. Every
screen drew; none was blank:

    Input Seed (Where from? -> FROM PAYLOAD)
    Input Seed: Source: the systemwide payload
    Add a BIP-39 passphrase? (Skip)
    Policy Review — Slots @1 and @2 filled from the payload (cards 1 and 2 of 4,
      in payload order). Policy stub: 4c3c96f1 Slots: @0 (no fp) @1 …
    Which md1? Full policy md1 / Template-only md1
    EXPERIMENTAL (hold to confirm)
    Engrave Mode — What to engrave? Full (seed + keys) / Watch-only (keys)
    Choose engraving  TEXT+QR / TEXT ONLY / QR ONLY   Card 1 of 3 | Plate 1 of 1

The flow is drivable from the template picker to the first engrave screen, fed
entirely by the payload. That line is also the live proof that the §0.1
announcement reaches the confirmation surface.

SPEC P1 anticipates exactly this and says to record it and **name what was not
exercised**, so I filed **F-178** (`mnemonic-engrave` `11ddcd0`) naming five:
the engrave itself, **hardware** (where D-1 was field-observed), the
NFC-scanned card source (excluded by F-174), the **typed-seed** source (this run
took the seed from the payload), and every parameter shape but n=3/k=2/@0/wsh.

**Gating consequence, and it is S2's:** S2's test 1 is *"the D-1 reproduction
from S1, promoted to a regression test — it MUST fail on the unfixed code"*.
There is no reproduction to promote. S2 must reproduce D-1 in one of those
shapes or record that it could not; it may not treat test 1 as discharged.

---

## Verification — run, with true exit codes, never through a pipe

    $ nix develop --command go test ./...
    TEST_EXIT=0    51 ok, 0 FAIL          (baseline: 51 ok, 0 fail — unchanged)

    $ nix develop --command go vet ./...
    VET_EXIT=1     6 findings, ALL `testing.ArtifactDir requires go1.26 or
                   later (file is go1.25)` — the pre-existing baseline, 6 of 6.
                   Zero non-ArtifactDir findings.

    $ nix develop --command gofmt -l ./
    (no output)    clean

    $ nix develop --command tinygo build -size short -o /dev/null \
        -target pico-plus2 -stack-size 16kb -gc precise -opt 2 \
        -scheduler tasks ./cmd/controller
       code    data     bss |   flash     ram
    1315740   31176   30732 | 1346916   61908

    flash 1346916, was 1342468 → +4448 bytes. Expected: S1 edits gui/, which is
    reachable from the firmware. The device build SUCCEEDS.

Test counts were taken by `grep -c '^ok'` / `grep -c '^FAIL'` over a saved file
plus the command's own exit code, not by summing a piped summary.

## Commits

| repo | sha | what |
| --- | --- | --- |
| `mnemonic-engrave` | `f51d8cc` | F-176 WITHDRAWN — the premise is false, with the reproduction and the byte-identity result |
| `seedhammer` | `3a0ac6e` | S1: the payload supplies the whole cosigner set |
| `mnemonic-engrave` | `11ddcd0` | F-178 — S1 found no D-1; the five unexercised shapes named |

`descriptor-mnemonic` is **clean and untouched** at `5a0a4f41`. Nothing pushed.

## Not done, deliberately

- `md encode --origin @i=` — sugar, not gating; see phase 1.
- No re-pin of `oracle/pins.json`, no gate-record re-anchor (batched with S2 per
  F-177), and no gate record for S1 (F-175).
- The shared `bundleGatherFlow` still says "scan" in its own two in-gather
  messages. S1's pre-check makes both unreachable from the Build path when the
  payload cannot fill the slots, but a payload with *enough* cards plus one
  orphan chunk still shows *"Dropped an incomplete card — scan all its chunks"*.
  It is a warning, not a dead end, and the string is shared with Engrave Bundle
  whose D-4 text work is **S2's**. Flagged, not fixed.
