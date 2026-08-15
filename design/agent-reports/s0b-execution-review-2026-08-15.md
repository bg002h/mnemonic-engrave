# S0b execution review — independent adversarial false-PASS hunt

**Date:** 2026-08-15
**Target:** SeedHammer fork `/scratch/code/shibboleth/seedhammer`, branch `main`, HEAD `4b8488efd0c094ee5ae635740bbbd359e38b5994`
**Scope:** the S0b diff only — `8345b0e`, `c94c135`, `94e8085`
**The one question:** does the S0b scaffolding contain a defect that would let a LATER stage's gate PASS WRONGLY?

**Verdict: 2 Critical, 2 Important, 4 Minor, 2 Nit. The loop does not close.**

Both Criticals are the same shape from opposite ends: the derived-census /
oracle byte comparison that S0b built **works** — I re-proved all three of its
mutation gates go red — but it is **wired to nothing that a later stage will
produce**, and it **skips silently** in the environment the repo's own required
check runs in. The mechanism is real; its reach is one committed file on one
machine.

Working tree left clean: `git status --porcelain` empty, HEAD unchanged at
`4b8488e`, final `go test ./...` exit 0 / 51 ok / 0 FAIL. Every mutation below
was reverted with `git checkout --` and re-verified green.

---

## CRITICAL

### C-1 — The whole derived-census gate SKIPS when the oracle binaries are absent, which is always the case in CI, and the suite still reports `ok` and exit 0

`oracle/expect_test.go:17-32` (`resolveBins`), reached by `oracle/expect_test.go:49` (`loadS0`)
`.github/workflows/test.yml:13`

`resolveBins` calls `t.Skipf` when any pinned binary is missing from
`~/.cargo/bin`. Every census test funnels through `loadS0`, so the skip takes
out **the gate and all three of its mutation proofs at once**:
`TestS0CensusMatchesTheDerivedExpectation`,
`TestS0PlateCountIsDerivedFromTheInputs`,
`TestS0FingerprintsComeFromTheRecordedSeeds`,
`TestCompareCensusCatchesAMutatedString`,
`TestCompareCensusCatchesAShortCensus`,
`TestCompareCensusCatchesReorderedPlates`.

CI runs `CGO_ENABLED=0 go test ./...` on `ubuntu-latest` with `actions/setup-go`
and no Rust toolchain, so `~/.cargo/bin/md` does not exist there and the skip is
**unconditional in CI**, on every push and every PR.

The in-code justification is itself false: *"The gate that makes absence fail is
`TestS0GateHasARecord`, which does not need the binaries"* (`expect_test.go:16`).
`TestS0GateHasARecord` (`oracle/record_test.go:359`) checks only that a **record
file exists** for stage S0. It says nothing about whether the byte comparison
ran, and it cannot: it never touches `CompareCensus`.

**False-PASS scenario.** S2's and S5's gate is "the engraved md1/mk1 is
byte-identical to the primary toolchain", and the evidence line every stage
commit carries is `go test ./... N ok 0 FAIL`. In CI — the required check — that
line is satisfied with the comparison and all three of its mutation proofs
skipped, so the gate reports GREEN having compared nothing. Same thing happens
locally the moment a contributor lacks the Rust toolchain, or the maintainer's
binaries move (`cargo install --root`, a rustup relocation, a rename).

**Evidence — run:**

```
$ nix develop --command sh -c 'HOME=$SP/fakehome … go test ./oracle/ -count=1 -v \
    -run "TestS0Census|TestCompareCensusCatches|TestS0PlateCount"'
=== RUN   TestS0CensusMatchesTheDerivedExpectation
    expect_test.go:75: oracle "md" is not installed at …/fakehome/.cargo/bin; skipping the derived-census gate
--- SKIP: TestS0CensusMatchesTheDerivedExpectation (0.00s)
--- SKIP: TestS0PlateCountIsDerivedFromTheInputs (0.00s)
--- SKIP: TestCompareCensusCatchesAMutatedString (0.00s)
--- SKIP: TestCompareCensusCatchesAShortCensus (0.00s)
--- SKIP: TestCompareCensusCatchesReorderedPlates (0.00s)
PASS
ok  	seedhammer.com/oracle	0.004s
EXIT_CODE=0
```

and the full suite in the same environment:

```
$ … CGO_ENABLED=0 go test ./... -count=1
FULL_SUITE_EXIT=0
--- FAIL count ---  0
--- ok lines ---   51
--- oracle line --- ok  	seedhammer.com/oracle	0.055s
```

51 ok, 0 FAIL, exit 0 — the exact evidence line the stage commits quote, with
the byte-identity gate silent.

**Minimal fix.** Make absence fail, the way `TestS0GateHasARecord` already does
for the record: default to `t.Fatalf` when a pinned oracle is missing, and let a
contributor opt out explicitly (`ORACLES_OPTIONAL=1`) rather than the gate
opting out on their behalf. Independently, install the three oracles in the
`Test` workflow so the gate has run before a merge. A skip that is invisible
without `-v` is not a gate.

---

### C-2 — The comparison is hardwired to one record produced by the walk S0b exists to REPLACE; nothing applies it to any record a later stage will produce. A fabricated gate record passes the entire suite.

`oracle/expect_test.go:50,54` — `LoadRecord(… "S0-trace-a.record.json")`, `LoadInputsFile(… "S0-trace-a.inputs.json")`
`cmd/gaterecord/main.go` — mints records and never derives

`CompareCensus` and `DeriveExpected` have **no callers outside
`oracle/expect_test.go`**:

```
$ grep -rn "CompareCensus" .    # non-comment hits
oracle/expect_test.go:76,121,133,153,167,175
oracle/expect.go:179            # the definition
$ grep -rn "DeriveExpected" .
oracle/expect_test.go:65,183
oracle/expect.go:125            # the definition
```

`cmd/gaterecord` — the only tool that writes a gate record — resolves the three
oracles and then never asks them what the census should have been.
`oracle.VerifyRecord` (`oracle/record.go:313-372`) checks the record against the
**walk file beside it**, which is a closed loop: both halves come from the same
run, so a consistent pair says nothing about whether the right thing was
engraved.

Two consequences, and the second is the one that bites later stages:

1. **The one record the gate covers came from the wrong walk.**
   `oracle/gaterecords/S0-trace-a.walk.json` carries the key `gathered`, which
   only `walk_trace_a.js` emits — the **Engrave Bundle** driver. So the derived
   census has never once been applied to a run of `walk_build_policy.js`, the
   Build-policy flow S0b was written to reach. F-169's core complaint
   ("five gates named a flow no walk had entered") now has a sibling: the byte
   comparison names a flow no *record* has come from.

2. **A later stage's record is not compared at all.** I fabricated one:

```
$ go run ./cmd/gaterecord -stage S9 -walk $SP/fake.walk.json \
      -inputs $SP/fake.inputs.json -base S9-fake
wrote oracle/gaterecords/S9-fake.record.json
      oracle/gaterecords/S9-fake.walk.json
```

whose census is six invented strings `md1FAKE0 … md1FAKE5`. Then:

```
$ CGO_ENABLED=0 go test ./... -count=1
FULL_SUITE_EXIT=0
--- FAIL count --- 0

$ go test ./cmd/emu/ ./oracle/ -v -run "TestGateRecord…|TestEveryGateRecord…|TestS0GateHasARecord|TestS0Census"
    gaterecord_anchor_test.go:79: S9-fake.record.json plate 0 is not an mk1 (md1FAKE0…) — not anchored by the payload
    …plate 1…2…3…4…5 (all six)
    gaterecord_anchor_test.go:97: anchored 6 engraved mk1 string(s) across 2 record(s) to the payload's own chunks
--- PASS: TestGateRecordStringsAreRecordsOfTheCardsPayload (0.00s)
    expect_test.go:79: derived 6 artifact(s) from the recorded inputs; all matched the engraved census
--- PASS: TestS0CensusMatchesTheDerivedExpectation (0.02s)
    record_test.go:374: S0 gate records: [S0-trace-a.record.json]
--- PASS: TestS0GateHasARecord (0.00s)
    record_test.go:395: verified 2 gate record(s): [S0-trace-a.record.json S9-fake.record.json]
--- PASS: TestEveryGateRecordOnDiskVerifies (0.00s)
ok  	seedhammer.com/cmd/emu
ok  	seedhammer.com/oracle
```

`TestEveryGateRecordOnDiskVerifies` reports **"verified 2 gate record(s)"** about
a census of pure fiction. `TestGateRecordStringsAreRecordsOfTheCardsPayload`
logs all six as unanchored and still passes, because its `matched != 0` floor is
satisfied by S0's six real mk1s — one good record permanently launders every
later bad one. The derived comparison never opens the file.

(Fabricated files removed; `git status --porcelain` empty afterwards.)

**False-PASS scenario.** S3/S4/S5 each engrave **md1 policy chunks**, which the
payload anchor test explicitly cannot check ("md1 and ms1 are PRODUCED by a
build, not supplied by the payload",
`cmd/emu/gaterecord_anchor_test.go:74-80`). A stage lands `S3-*.record.json`
holding six md1 chunks built from the wrong xpub / wrong stub / wrong slot
order; `go test ./...` is green, the record "verifies", and the stage's stated
byte-identity gate is satisfied by a comparison that was never performed on it.

**Minimal fix.** Replace the hardcoded filename with a loop over
`oracle.Records(GateRecordsDir)`, require every record to carry an `expect`
block, and fail — not skip — on one that does not. Then make `cmd/gaterecord`
refuse to write a record whose census does not equal the derived expectation, so
the comparison is a property of *minting* a record rather than of remembering to
extend a test.

---

## IMPORTANT

### I-1 — The walk's own `ok` still compares the census against a caller-supplied `plates`. F-170 was fixed in Go and left standing in JavaScript.

`cmd/emu/walk_build_policy.js:299` (`plates = 9`), `:532` (`census.strings.length === plates`)
`cmd/emu/walk_trace_a.js:151` (`plates = 6`), `:274`

F-170's stated target was *"`census.strings.length === plates`, with `plates` a
parameter defaulting to 6"*. That line is still there, verbatim, in the walk that
produced S0's record:

```js
// walk_trace_a.js:151,274
export async function run({ pace, plates = 6, perPlateDigest = true, …
    ok: census.strings.length === plates && census.unattributed === 0,
```

and reproduced in the new driver with a hand-derived 9 ("1 ms1 + 2 mk1 chunks +
6 md1 chunks for a full 2-of-3 wsh build" — a count computed by a human in a doc
comment, which is the shape the project's own rule forbids).

Of the six terms in `walk_build_policy.js`'s `ok`, five are observed from the
emulator (`proven` is pushed only after a successful `shScreen()` match;
`presentedAtEnd` from `shNFC.presented()`; `cardsGathered` parsed off the gather
screen; `selected` from a needle actually appearing; `census.unattributed` from
the recorder). **`plates` is the only driver-supplied term, and it is the one
that stands in for content.**

**False-PASS scenario.** A later stage runs `w.run({ engrave: true, plates: 9 })`,
the flow cuts nine **wrong** strings, `ok === true`, and a stage whose gate is
"the emulator walk returned `ok: true`" goes green about content nothing
compared. Today that is only closed by the record + derived census — which is
C-2's hole.

**Minimal fix.** A walk cannot derive, so `ok` should stop implying content:
drop `census.strings.length === plates` from `ok` and report the count as data,
making the gate the record plus the derived comparison. Or pass the derived
strings *in* and compare them in the walk.

---

### I-2 — The two halves of the needle gate are joined by a comment, not by a check

`cmd/emu/walk_build_policy.js:56-70` (the `NEEDLE_*` constants)
`cmd/emu/needle_test.go:42-68` (`buildFlowNeedles`)

The design is explicit that neither half is a gate alone: *"needle_test.go pins
them; walk_build_policy.js asserts the needle appeared. Neither half is a gate
alone."* The JS says *"Keep in sync with cmd/emu/needle_test.go's
buildFlowNeedles, which is what proves 'unique'."*

Nothing enforces that. No Go file reads the driver:

```
$ grep -rn "walk_build_policy" .   # excluding the file itself
cmd/emu/walk_build_policy.js:4:   //   const w = await import("./walk_build_policy.js");
```

— one self-reference in its own doc comment, and no other hit in the tree.

**False-PASS scenario.** A stage author edits a `NEEDLE_*` literal in the JS to
a string with multiple production sites — `"First card from where?"` (2 sites)
or `"Which md1?"` (2 sites), both of which the walk already calls `waitFor` on
elsewhere. `needle_test.go` still passes, because it validates its own untouched
list. The walk still returns `needlesProven` of length 7 and `ok: true`. The
walk's central claim — *this is the Build-policy flow and not Engrave Bundle* —
is now false while green. That is F-169 recurring through the one seam the fix
did not close, and it is trivially machine-checkable.

**Minimal fix.** A Go test that extracts `export const NEEDLE_\w+ = "(...)"`
from `walk_build_policy.js` and requires every literal to appear in
`buildFlowNeedles` — the same read-the-source-off-disk trick `needle_test.go`
already uses for `gui/`.

---

## MINOR

### M-1 — `oracle/pins.json` lags the primary, and nothing in the gate can tell

`oracle/pins.json:38-44`

Measured against the primary checkouts:

| oracle | pinned commit | primary HEAD | drift |
| --- | --- | --- | --- |
| `md` | `5a0a4f41` | `5a0a4f41` | none (at HEAD) |
| `mk` | `a38a908e` | `3462157` | 1 commit — docs only |
| `ms` | `ddfa497` (0.15.0) | `de593ca` | 3 commits, incl. `98e1f6a` **feat(ms-cli 0.16.0)** |

```
$ git -C mnemonic-secret log --oneline ddfa497..HEAD
de593ca style(ms-cli): rustfmt mlock.rs under the pinned 1.95.0 toolchain
ef57a51 test(ms-codec): honour parity_smoke's own documented version skip
98e1f6a feat(ms-cli 0.16.0): accept a bare `--template bip48` and say so loudly
```

This confirms F-177 (the `ms` pin lags the settled 0.16.0). **Answering the
probe's second half: a stale pin PASSES silently.** Resolution in `resolveBins`
compares the installed binary's SHA-256 against the recorded hash and nothing
else; both currently match (`sha256sum ~/.cargo/bin/{md,mk,ms}` equals the three
pinned digests, `--version` reports `md 0.13.0` / `mk 0.13.0` / `ms 0.15.0`), so
everything is green. There is no check anywhere that asks whether the pinned
commit is still what the primary repo says.

Currently benign — 0.16.0 is additive (it accepts a bare `--template bip48`;
the gate passes `bip48-p2wsh` explicitly) and the `mk` delta is documentation —
which is why this is Minor rather than Important. The false-PASS is the next
one: a normative change lands in a primary encoder, the pin is not moved, and
"byte-identical to the primary toolchain" stays green against a superseded
encoder. Note the fail-closed direction is already right: a **rebuilt** binary
changes its hash and `resolveBins` `t.Fatalf`s.

**Fix.** Put the primary checkout path in `pins.json` beside `repo`, and report
(loudly, even if it cannot fail on a machine without the checkout) the distance
between the pinned commit and that repo's HEAD.

### M-2 — `CompareCensus` returns nil for N empty strings against N empty artifacts

`oracle/expect.go:179-207`

The vacuity probe was the highest-value item in the brief; here is the whole
table, run:

```
nil,nil                      -> REFUSED   (nothing was compared, so this check passed by checking nothing)
empty,empty                  -> REFUSED   (same)
nil,empty                    -> REFUSED   (same)
empty,nil                    -> REFUSED   (same)
empty want, non-empty got    -> REFUSED   (the inputs require 0 engraved string(s); the census holds 1)
non-empty want, empty got    -> REFUSED   (the inputs require 1 engraved string(s); the census holds 0)
one empty string each        -> *** RETURNED nil (passed) ***
six empty strings each       -> *** RETURNED nil (passed) ***
```

The `n == 0` guard is genuinely sound and catches every nil/empty/length-mismatch
shape. The one hole is **content-empty, count-nonzero**: six `Artifact{String:""}`
against six `""` compares six times, matches six times, and returns nil.

Unreachable today — `mkEncode` (`oracle/expect.go:334-350`) skips blank lines,
refuses any line without an `mk1` prefix, and refuses a zero-chunk result — so
`DeriveExpected` cannot emit an empty `String`. Reachable by a future deriver
(`md encode`, `ms encode`) that is less careful. Note `ParseWalk` would *not*
stop it from the other side: it checks `len(w.Census.Strings) == 0`, not whether
the strings are non-empty.

**Fix.** Refuse an empty `want[i].String` or `got[i]` inside the loop.

### M-3 — `productionSites` counts FILES, not sites, and only at `gui/` depth 1

`cmd/emu/needle_test.go:158-189`

The helper returns one entry per file (`out = append(out, "gui/"+name)` after a
whole-file `strings.Contains`), so two identical needle literals inside
`gui/multisig_build.go` would read as *"exactly 1 production site"* — which is
not what the test's failure message claims to have measured. It also skips
directories, so a needle rendered from `gui/widget`, `gui/text` or another
package counts as zero sites.

No live defect: I measured every needle across the whole tree, and each of the
seven occurs exactly once in production, in the file `needle_test.go` names, with
nothing outside `gui/*.go`. And `gui/multisig_build.go` holds only build-flow
functions today (`grep -n "^func " gui/multisig_build.go`), so file ≈ flow holds.
Recording it because the guard's stated claim is stronger than its measurement,
and both assumptions are the kind that decay silently.

### M-4 — The inputs file is bound to the record only by the comparison succeeding

`oracle/record.go:313-372` (`VerifyRecord`), `oracle/expect_test.go:47-70` (`loadS0`)

`loadS0` takes the seed **words** from `S0-trace-a.inputs.json` and the
**origins** from `rec.Inputs`, and nothing ever compares `inf.Tuple()` against
`rec.Inputs`. The record's attested `template`, `n`, `k`, `slot_order`,
`fp_choice` and per-seed `digest` can all drift from the inputs file that the
expectation is derived from, and no test notices.

Words are the exception and they are covered — mutation D below proves a word
change goes red. Everything else is unchecked. Minor because the load-bearing
field is the one that is bound; recorded because a record whose tuple has
drifted from the file the gate derived from is an attestation about a run that
did not happen.

**Fix.** One line in `loadS0` (or better, in `VerifyRecord` given an inputs
path): require `inf.Tuple()` to equal `rec.Inputs`.

---

## NIT

### N-1 — `samePath` normalisation is lowercase-only and unanchored

`oracle/expect.go:253-258` — `strings.ReplaceAll(strings.TrimSpace(s), "h", "'")`
replaces every `h` anywhere in the string and does not handle `H`. An origin
written `m/48H/0H/0H/2H` fails to compare (fail-closed, so safe today, and
`pathRe` accepts only `'` or lowercase `h` anyway).

### N-2 — `squash()` removes all whitespace on both sides, and `ExtractText` emits runes in draw order

`cmd/emu/walk_build_policy.js:37`, `gui/op/op.go:617-628`. Text is collected in
reverse draw order and then reversed, so adjacency in the extracted string
reflects draw order rather than layout; with all whitespace stripped, a needle
could in principle be satisfied by two unrelated drawn strings that happen to
abut. No live instance; the seven needles are long and distinctive.

---

## PROBES THAT CAME BACK CLEAN

Stated explicitly, with what was run.

**Probe 2 — is the census actually DERIVED, or a constant wearing a function
signature? CLEAN.** It is genuinely derived, and I proved it two ways.

First, I reproduced plate 0 and 1 by hand from the primary toolchain:

```
$ ~/.cargo/bin/ms derive --phrase "abandon … about" --template bip48-p2wsh --account 0 --network mainnet --json
{"master_fingerprint":"73c5da0a","account_path":"m/48'/0'/0'/2'","account_xpub":"xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf",…}
$ ~/.cargo/bin/mk encode --xpub xpub6DkFA… --origin-path "m/48'/0'/0'/2'" \
      --origin-fingerprint 73c5da0a --policy-id-stub 5b48af35 --group-size 0
mk1qpd8cwpqqsq4kj90x4eutks2q5zg3vs7rnefw94m5rru59s2su80aw2q4wgdpapgfl4pkhsdyytkwl5z8lphut2hvvpp5av5muuc0cmfrjw2
mk1qpd8cwpp806lhaeh6reknylagmwyjycf8044xtt9flsdlkvt6f6cthyl995lpm5zlrp6yv6kc36tw
```

byte-identical to census plates 0 and 1 in `S0-trace-a.record.json`.

Second — the decisive one — **MUTATION D (mine): change a seed's WORDS in the
inputs file.** A constant would not notice; a derivation must:

```
$ # masterB's words replaced with another valid BIP-39 vector
$ go test ./oracle/ -run TestS0CensusMatchesTheDerivedExpectation
--- FAIL: TestS0CensusMatchesTheDerivedExpectation (0.07s)
        plate 2 (mk1, payload:masterB (card B@0)) differs:
          expected mk1qpu27cpqqsq4kj90x5lkxknrq5zg3vs7aecl33fc6wcqy5dyvwc0l6d4jhfpw5nxf3dzx8yflshmfa544wgvmnthm5psuw2k5q0st8pxdyn4
          engraved mk1qpmm93pqqsq4kj90xkux3r03q5zg3vs7llvu2xd8x2rk7av9gmew82jq5zap9302ynhp37ggd6z5u4emag0zr8gh9upnj5stuxqtzdn4uxkd
        plate 3 (mk1, payload:masterB (card B@0)) differs: …
FAIL	seedhammer.com/oracle
```

Both of masterB's plates changed and only masterB's. `plates = 6` is dead on the
Go side: the count falls out of `len(out)`. Restored, green.

**Probe 3 — needle single-site-ness, RE-MEASURED. CLEAN.** Every needle the walk
actually asserts, counted today with `grep -rnF … gui/ | grep -v _test.go`:

| needle | sites | file |
| --- | --- | --- |
| `Choose policy type` | 1 | `gui/multisig_build.go:396` |
| `How many keys (n)?` | 1 | `gui/multisig_build.go:472` |
| `Which slot is your key?` | 1 | `gui/multisig_build.go:490` |
| `Cosigner Keys` | 1 | `gui/multisig_build.go:524` |
| `Supply or build a policy?` | 1 | `gui/multisig.go:44` |
| `Payload cards` | 1 | `gui/multisig_build_payload.go:296` |
| `Use payload card` | 1 | `gui/multisig_build_payload.go:325` |
| `Which md1?` (decoy) | 2 | `gui/singlesig.go:95`, `gui/multisig_build.go:213` |
| `First card from where?` (decoy) | 2 | `gui/multisig.go:76`, `gui/bundle_flow.go:25` |
| `Engrave Bundle` (decoy) | ≥1 | 6 files |

All seven single-site, in the claimed files; both pinned decoys at their pinned
counts. **The line numbers in the plan have drifted** — `Choose policy type` is
at `:396` not `:300`, `How many keys (n)?` at `:472` not `:376`, `Which slot is
your key?` at `:490` not `:394` — cosmetic only, since the test matches by
content. The remaining exposure is I-2 (nothing binds the JS list to the pinned
list) and M-3 (the counter measures files).

**Probe 4 — can `shNFC.presented() == 0` be zero because the mechanism is
absent? NO. Proven RED live in the browser.** Built `emu.wasm`
(`nix develop --command ./cmd/emu/build.sh`, 9 806 687 bytes), served
`cmd/emu` on a fresh port 8791, drove the real walk:

```js
> await import("./walk_build_policy.js").then(w => w.run())
{ ok: true,
  needlesProven: ["Supply or build a policy?","Choose policy type","How many keys (n)?",
                  "Which slot is your key?","Cosigner Keys","Payload cards","Use payload card"],
  presented: 0, cardsGathered: 4, openSlots: 2, selected: true,
  decisions: ["use:1","use:2"], elapsedSec: 12, carouselHops: 9,
  gatherScreen: "CosignerKeysmd1descriptors:0mk1keys:4Scanacard,orDone." }
```

then mutated the condition in the live session:

```
before:        0
control:       PASS (no throw)
afterPresent:  1
mutated:       THREW: after present: 1 record(s) crossed the NFC reader; a stage-gate run must present ZERO…
afterClear:    1
laundered:     THREW: after clear: 1 record(s) crossed the NFC reader…
```

The assertion goes red on a single presentation, and `clear()` does **not**
launder it back to green. The stale-wasm guard is real too: `assertNoNFC` throws
a named error when `shNFC.presented` is not a function, and a missing `shNFC`
entirely raises a `TypeError` — both fail-closed. `presentedCount` is
incremented in exactly one place (`nfc.go:101`, inside `set`), which is the only
route a record has into the reader.

**Probe 5 — does `shScreen()` read the RENDERED frame or the model's intended
text? RENDERED. CLEAN, and structurally different from `uiContains`.**
`screenRecorder.Frame` calls `op.Drawer.ExtractText` (`gui/op/op.go:617`), and
the rune is appended only after the glyph survives clipping:

```go
// gui/op/op.go:416-427
clip := state.clip.Intersect(dst.Bounds())
if clip.Empty() { break }
…
if d.text != nil {
    for _, m := range maskStack {
        switch img := m.op.materialize(0).(type) {
        case *glyph:
            d.text = append(d.text, img.r)
```

So a line that did not draw is absent from `shScreen()` exactly as it is absent
from the panel, and a walk's `waitFor` on it times out — RED, which is the safe
direction. The walk's matcher is also strictly stricter than `uiContains`:
`squash()` (`walk_build_policy.js:37`) normalises whitespace on **both** sides,
where `uiContains` (`gui/gui_test.go:527-532`) strips spaces from the needle only
and lowercases both — the asymmetry F-179 exploits. S0b's screen-reading path
does not inherit that blindness. (Residual: N-2.)

**Probe 6 — oracle resolution by name? CLEAN.** Every invocation is
`exec.Command(bin, …)` with `bin` an absolute path built by
`filepath.Join(home, ".cargo", "bin", name)` (`oracle/expect_test.go:40-44`,
`cmd/gaterecord/main.go:133`), and `exec.Command` does not use a shell, so the
`md` → `mkdir -p` alias hazard cannot fire. The only bare-name exec in the
package is `git` in `gitState` (`oracle/oracle.go:270,274`), which is not an
oracle. Also checked the `94e8085` refusal: `mk encode`'s
`note: stdout is watch-only …` goes to **STDERR**, verified by splitting the
streams, so the non-`mk1` refusal is not tripped today — and if that note ever
moves to stdout the gate refuses rather than adopting it, which is the correct
direction. Pin staleness is M-1.

**Probe 7 — the three claimed mutation proofs, re-applied by me. ALL THREE
PROVEN.**

*Mutation 1 — one expected string.* Flipped the last character of census plate 2
in `S0-trace-a.record.json`:

```
--- FAIL: TestS0CensusMatchesTheDerivedExpectation (0.02s)
    plate 2 (mk1, payload:masterB (card B@0)) differs:
      expected mk1qpmm93p…tzdn4uxkd
      engraved mk1qpmm93p…tzdn4uxkq
FAIL	seedhammer.com/oracle	0.026s
```
Both strings printed in full, plate named. Restored → `ok`.

*Mutation 2 — one plate digest.* `55b380890d0b2dff` → `…fe`:

```
--- FAIL: TestEveryGateRecordOnDiskVerifies (0.00s)
    the embedded plate digests are not the walk's:
     embedded [… 55b380890d0b2dfe …]
     walk     [… 55b380890d0b2dff …]
FAIL	seedhammer.com/oracle	0.026s
```
Restored → clean.

*Mutation 3 — one needle.* Appended a **real, compiling** second production site
to `gui/singlesig.go`:

```go
func mutantSecondSite() *ChoiceScreen {
    return &ChoiceScreen{Title: "Template", Lead: "Choose policy type", Choices: multisigScriptChoices()}
}
```

The mutant compiles — `go build ./gui/` → `BUILD_EXIT=0`, so the red below is
the gate firing and not the compiler:

```
--- FAIL: TestBuildFlowNeedlesHaveExactlyOneProductionSite (0.02s)
    needle_test.go:100: needle "Choose policy type" has 2 production site(s), want exactly 1:
          gui/multisig_build.go
          gui/singlesig.go
        a walk anchoring on this cannot prove which flow it is in
FAIL	seedhammer.com/cmd/emu	0.022s
```

Restored → `ok  seedhammer.com/cmd/emu`. All three mutations landed in code that
ran (each produced its refusal message from the mutated value itself, not merely
from an edit having occurred), and `git status --porcelain` is empty afterwards.

**Probe 8 — outcome binding. Mostly clean; one hole, which is I-1.** Five of the
six terms in `walk_build_policy.js`'s `ok` are read back from the emulator; only
`plates` is driver-supplied. `raceFor` correctly refuses to treat a timeout as
"the other arm happened". The engrave tail stops on an unrecognised screen
(`acts.push({act:"STALLED"}); break`) rather than tapping past it, and a stall
then makes `ok` false via the short census. Two S2-era fields (`decisions`,
`gatherTextIsNotEvidence`) are recorded without being asserted, which is
deliberate and documented as such.

**Probe 9 — could a stale record vouch for a walk that never ran? Mostly no.**
`VerifyRecord` binds the record to the walk file by SHA-256 (mutation 2 proves
the digest arm is live), and `ParseWalk` (`oracle/record.go:154-186`) is a
genuinely strict gate: it refuses `ok:false`, an empty census, any
`unattributed != 0`, `announced < len(strings)`, a digest count that disagrees
with the string count, an empty payload digest and any blank digest.
`TestS0GateHasARecord` never skips. The gaps are C-2 (a record can be minted
with any census at all, and only S0's is ever compared) and M-4 (the inputs file
is not bound to the record's tuple).

---

## HYGIENE

- No edits survive. `git status --porcelain` → empty; `git rev-parse HEAD` →
  `4b8488efd0c094ee5ae635740bbbd359e38b5994`.
- Final baseline re-run at pristine HEAD: `go test ./...` exit 0, 51 `ok`,
  0 `FAIL`; `gofmt -l` clean.
- `cmd/emu/emu.wasm` was rebuilt; confirmed gitignored
  (`git check-ignore -v cmd/emu/emu.wasm` → `cmd/emu/.gitignore:1:emu.wasm`).
- Exit codes were read directly, never inferred from a summary of piped output.
