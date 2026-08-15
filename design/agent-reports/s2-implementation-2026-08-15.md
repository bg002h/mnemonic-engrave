# S2 implementation report — 2026-08-15

Sole implementer, solo, TDD, verified inline. Five commits in `seedhammer`, one
in `mnemonic-engrave`. **Nothing pushed.**

Repo: `/scratch/code/shibboleth/seedhammer`, branch `main`, from `ca2e14b`.

---

## What landed, in order, with SHAs

| # | SHA | item |
| --- | --- | --- |
| 1 | `dcd90a5` | **The §4.1 duplicate-key check — S2's FIRST landing**, before any work that completes an engrave |
| 2 | `101c8eb` | D-4, the gather title |
| 3 | `f712a81` | The interim foreign-origin refusal (spec M-E) |
| 4 | `189b173` | §0.1a — the review screen speaks the BIP-48 script-type origin |
| 5 | `3ea3ede` | `TestBuildWalkTypedSeed` + the whole-walk raster floor + the md1 byte-identity gate |
| — | `776844e` (mnemonic-engrave) | F-179 … F-182 |

`cmd/emu/emu.wasm` is gitignored, so the rebuilt emulator is not in any commit.

---

## 1 — the duplicate-key check (`dcd90a5`)

**Landed first, alone, and committed before anything else in the stage**, per the
sequencing ruling.

RED before GREEN, through the production path:

    go test ./gui/ -run TestS2RefusesDuplicateKeysBeforeS4
    assembleBuildPolicy accepted a policy whose slots @0 and @1 hold the SAME key

- Lives in `assembleBuildPolicy` (`gui/multisig_build.go`), after `all` is
  complete and before `md.EncodeMultisig`.
- Sentinel `errBuildDuplicateKey{SlotA, SlotB}`; the flow branches on
  `errors.As` and shows a named `"Duplicate key"` modal built from `p.SelfSlot`
  and `buildCosignerOrigins`.
- Comparison basis is `[32]byte` chain code + `[33]byte` pubkey array equality —
  literally the 65 bytes, since both are comparable arrays.

**The comparison basis is asserted in both directions, machine-checked, not
argued:** the delivered collision is byte-equal (`selfCC != a0.ChainCode ||
selfPK != a0.CompressedPubkey` fails the test if the payload ever stops
colliding), and A@0 vs A@1 differ on both components. The test also asserts that
A@0 and A@1 carry the **same non-empty master fingerprint**, which is what makes
"master fingerprint would refuse the legitimate wallet" a measurement rather than
a claim.

Tests: `TestS2RefusesDuplicateKeysBeforeS4` (5 subtests),
`TestBuildFlowRefusesDuplicateBeforeReview`,
`TestBuildFlowDuplicateNeverReachesReview`.

---

## 2 — D-4 (`101c8eb`)

RED first:

    the Build-policy cosigner gather is titled "Engrave Bundle" ... 
    "EngraveBundlemd1descriptors:0mk1keys:1Donewhenyouhavereviewedthese."

`bundleGatherFlow` takes a `title string`. Four callers pass `"Engrave Bundle"`
byte-unchanged; the Build path passes `buildCosignerGatherTitle = "Cosigner
Keys"`. The title is threaded to the two "Done" refusals inside the gatherer too,
one of which is reachable from Build.

Second effect, which is the one the walk needed: `"Cosigner Keys"` is
single-site in `gui/multisig_build.go`, is now pinned in
`cmd/emu/needle_test.go`'s `buildFlowNeedles`, and the walk asserts it — needle
count 6 → 7.

Two comments that had outlived their conditions were corrected rather than left:
`needle_test.go`'s decoy rationale and `walk_build_policy.js`'s header
measurement. Both keep the history and state the lesson.

---

## 3 — the foreign-origin refusal (`f712a81`)

`errBuildForeignOrigin{Slot, Declared}`, checked over the cards after the
duplicate check, refused with a `"Key origin mismatch"` modal quoting **both**
origins.

`originIsShared` compares **parsed path components**, not strings:
`m/48'/0'/0'/2'` and `m/48h/0h/0h/2h` are the same path and both are accepted. A
path that does not parse is not shared.

---

## 4 — the §0.1a announcement (`189b173`)

`buildOriginAnnouncement(script)`, placed with the provenance line above the
stub. Three distinct sentences; legacy `sh` cites **no** authority. `buildReviewLines`
and `buildReviewFlow` gained a `script md.MultisigScript` parameter (3 test call
sites updated).

---

## 5 — the walk, the floor and the byte gate (`3ea3ede`)

    Trace A completed: 9 plates engraved from a keyboard-typed seed

Nine is derived, not observed: 1 ms1 + 2 mk1 chunks + 6 md1 chunks, asserted
exactly. The tail (verify offer, restore doc) is driven too.

`typeWords(router, frame, phrase)` is the reusable driver; it emits the same
runes + Button3 a finger does.

**Raster floor, measured on `sh2DisplaySize`:**

    title + 1 nav button, no body      2259
    title + 2 nav buttons, no body     2693      (~= F-151's measured 2652)
    title + 3 nav buttons, no body     5482      <- worst blank
    ---------------------------------- floor 6000
    "How many keys (n)?"               6566      <- thinnest real screen
    ... up to "Payload cards"         17621

`raster_test.go`'s 4000 was **not** reused: it was calibrated on a two-button
modal and sits below the worst blank here. The first draft used 4000 and a
title-only frame passed it.

**Byte-identity gate:**

    md oracle resolved: commit 5a0a4f41017d71d47f70684c145702d4ca0c3aa9 by
      binary-sha256 (reports "md 0.13.0", matches pin: true)
    6 md1 chunk(s) byte-identical to the primary; policy stub 06215ac0

Two invocation facts measured, not guessed: `--group-size 0` and
`--force-chunked` (335 data symbols against an 80-symbol cap). The template is
asserted off the device's own decode before the comparison.

---

## THE FIND — an em-dash blanks its whole line

The raster floor caught this on its **first run**, on the EXPERIMENTAL warning:

    INK "EXPERIMENTAL"   4973      <- below the 5482 px BLANK frame
    after removing the em-dash:
    INK "EXPERIMENTAL"  18563

Cross-checked through `showError` with `runUITouchRaster`: the same body rasters
**7419** with a hyphen and **2652** with an em-dash — and 2652 is the exact figure
`gui/raster_test.go` records for F-151's shipped-blank body. **F-151's defect and
this one are the same defect.** F-78's "zero-pixel glyph" understates it: the
glyph takes its whole line with it.

Fixed on S2's own walk (the EXPERIMENTAL body and the review's fp line). 31 sites
remain, enumerated by script and filed as **F-179**, several of them refusals
about unencrypted secrets.

---

## Mutations run — every one compiled, every one proved to have executed

| # | mutation | result |
| --- | --- | --- |
| M1 | duplicate check `&& false` | 4 failures; flow test's last screen `PolicyReview … Policystub:28419203` — the duplicate policy reaching review |
| M2 | key the duplicate check on master fingerprint | Trace B's legitimate multi-account 2-of-3 refused |
| M3 | draw `buildReviewFlow` before the duplicate refusal | both flow tests red, last screen `PolicyReview … Policystub:00000000` |
| M4 | make `bundleFlow` pass the Build title | `TestSuppliedGatherKeepsItsTitle` red **and** `TestBuildFlowNeedlesHaveExactlyOneProductionSite` red ("Cosigner Keys" has 2 sites) |
| M5 | origin check `&& false` | 3 failures; last screen `PolicyReview … Policystub:809a05a7` — D-2's silent stamp verbatim |
| M6 | compare origin strings instead of parsed components | the apostrophe spelling of the shared origin refused |
| M7 | one announcement for all three templates | 3 subtests red, incl. "legacy P2SH claims a BIP-48 assignment that does not exist" |
| M8 | swap slots @1/@2 in the oracle invocation | `chunk 0 differs: primary md1fdj2wzsp… device md1fxrvxzsp…` |

Every mutant's failure message names a screen or a value only reachable by
executing the mutated line, which is the "prove the mutated line RAN" half.

---

## Judgment calls, and which rule decided each

1. **Check order: duplicate BEFORE foreign origin.** *Not in any ruling — I had
   to decide it.* On the delivered payload both fire on one build (self=masterA,
   cards A@0+A@1). §0.1 clause 2 decides: a repeated key degrades the quorum
   **invisibly**, a foreign origin mis-states a path that is **printed on every
   artifact**. Also the duplicate check outlives the stage and the origin one
   dies at S5. Consequence, and it preserves the fable ruling exactly: **default
   taps + payload seed still reach the Duplicate key screen.** Asserted in a test
   named for it.

2. **Gather title = "Cosigner Keys", a parameter not a rename.** D-4 names the
   gather; "Engrave Bundle" is *correct* for `bundleFlow`, so renaming the shared
   default would fix one screen by breaking four. Title-cased to match the other
   titles; single-site so the walk can anchor on it.

3. **Origin comparison over parsed components, not strings.** §0.1 — "defaults
   for spelling, never for stakes". Refusing `m/48'/0'/0'/2'` while accepting
   `m/48h/0h/0h/2h` would be a refusal over notation. Mutation-proved (M6).

4. **Path spelling in the announcement: the device's `h` form, not BIP-48's
   apostrophes.** That is what `mk.Decode` renders and what the operator will
   compare against their key card. One spelling on one machine.

5. **No em-dashes in any new operator text.** Measured, not inherited (see
   above). The plan's drafted modal text uses one; the "Duplicate key" heading is
   the modal **title** and the body carries the rest, which preserves the text
   without the glyph.

6. **Raster floor 6000, not `raster_test.go`'s 4000.** Measured both ways;
   4000 sits below the worst blank at this display size.

7. **The Go walk taps SKIP, USE, USE — not SKIP, SKIP.** Because the Go fixture
   roster and the emulator payload are in different orders (F-180). Found by
   running it.

---

## Emulator walk — what RAN and what did not

**RAN, live in the browser** (`nix develop --command ./cmd/emu/build.sh`, served
on a fresh port 8971, driven with Playwright):

    needlesProven: ["Supply or build a policy?", "Choose policy type",
                    "How many keys (n)?", "Which slot is your key?",
                    "Cosigner Keys", "Payload cards", "Use payload card"]
    presented: 0    cardsGathered: 4    openSlots: 2    selected: true
    gatherScreen: "CosignerKeysmd1descriptors:0mk1keys:4Scanacard,orDone."
    elapsedSec: 12    ok: true

`gatherScreen` is D-4 proved on the machine: the title reads **Cosigner Keys**.
`presented: 0` (F-174) holds.

**STOPPED, not shipped — the emulator typed-seed leg.** I attempted `typeWord`
over `shTap` live and could not derive key coordinates reliably: the key rects
come from font metrics at layout time, and blind probing is self-defeating
because the valid-key mask disables every key but the one legal next letter
(measured: `shTap(80,180)`→`Q`, then only `U` responds; a sweep for backspace
auto-completed word 1 instead). **I did not commit an unrun driver** — that is
the "a gate that has never executed is a hypothesis" rule. Filed as **F-181**
with the cheap fix identified (`op.Drawer` already resolves an input's rect for
`tapNavSlot`; exposing that to the walk bypasses nothing). Owned by S4, which
needs the same driver.

---

## Verification — exact output, true exit codes, no pipes judged

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"

    nix develop --command go test ./...
      TEST_EXIT=0    51 ok    0 FAIL

    nix develop --command go vet ./...
      6 findings, all `testing.ArtifactDir requires go1.26` — the baseline

    nix develop --command gofmt -l ./
      (no output)    exit 0

    nix develop --command tinygo build -size short -o /dev/null \
      -target pico-plus2 -stack-size 16kb -gc precise -opt 2 \
      -scheduler tasks ./cmd/controller
         code    data     bss |   flash     ram
      1323392   31176   30732 | 1354568   61908

**Flash: 1,354,568** — was 1,349,428, **+5,140**. Expected: S2 edits `gui/`.
Per-commit: 1352184 (+2756), 1352244 (+60), 1353816 (+1572), 1354568 (+752),
1354568 (+0).

Working tree clean at `3ea3ede`.

---

## Things a reviewer should look at hardest

1. **The check-order decision (judgment call 1)** is mine, not the plan's. It is
   load-bearing for which refusal an operator sees on the delivered payload.
2. **F-179 inverts F-78's characterisation**, and F-78's wording is quoted in
   several existing comments that are now imprecise.
3. **The raster floor's margins are tight** — 518 px below, 566 above. Both sides
   are asserted by a test, but a screen whose text shortens could go red.
4. **The `duplicateSlotPair` comparison is array equality**, which is correct
   only while `MultisigCosigner`'s ChainCode/CompressedPubkey stay fixed-size
   arrays. If either became a slice, `==` would stop compiling (good) — but a
   reviewer should confirm that is the failure mode.
5. **`FuzzAssembleBuildPolicy` builds every card from `selfXpub`**, so post-S2 it
   exercises mostly the duplicate refusal path. It only asserts "does not panic",
   so it still passes, but its coverage narrowed and nothing says so.
