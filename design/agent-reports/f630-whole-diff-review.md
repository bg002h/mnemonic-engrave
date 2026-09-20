# F-630 whole-diff adversarial execution review

**Reviewer:** independent agent; did not write this code or its plan.
**Diff under review:** fork worktree `/scratch/code/shibboleth/sh-worktrees/f630`,
branch `f630-xpub-header-sync`, `git diff 95716e97..a126070`
(`da35965` T1, `ea3c825` T2, `a126070` T3).
**Question answered:** what did the implementation get wrong that the plan and
TDD could not catch.

Every claim below was reproduced in the worktree. Mutations were applied to a
filesystem snapshot copy and restored from it; the worktree was left at
`a126070` with `git status` clean and `git diff a126070` empty, and re-verified
green afterwards: `go test -count=1 ./md/` → `ok 0.154s`, and
`scripts/gui-shard-test.sh ./gui/ 24` → `RESULT: ok -- all 1374 tests ran across
24 shards`.

**Counts: 0 Critical / 1 Important / 3 Minor / 4 Nit.**

---

## I-1 — The pinned tier is EXAMINED only while its vendored twin exists. When the twin goes, the gate silently stops checking the fork's own fixtures and reports `0 fail`.

**Severity: Important.** Latent, not live: the tree is correct today. It
activates on exactly the event the pins were created for.

### Reproduction

```
$ rm -f md/testdata/vectors/keyed_tr_multi_a.*
$ go test ./md/ -run TestKeyedConformanceDescriptorsAgreeWithTheirTemplates -v
    conformance_keyed_test.go:279: descriptor gate: 45 of 45 vectors pass
                                   (43 correct-header, 2 pinned-legacy), 0 fail
PASS
ok  	seedhammer.com/md	0.049s
```

`md/testdata/forkbuilt/keyed_tr_multi_a.conformance.json` and
`keyed_tr_multi_a.md1.txt` are still on disk, untouched, and still named in
`forkbuiltRecordPins`. The gate passes, the pinned tier silently drops from 3 to
2, and **the membership assertion — the clause this plan invested most in — says
nothing**, because it asserts that the pin FILE EXISTS, never that the pin was
EXAMINED.

### Mechanism

- `eachKeyedVector` (`md/vector_fixtures_test.go:62-76`, and its gui twin
  `gui/vector_fixtures_test.go:90-104`) globs `testdata/vectors/` **only**. Its
  own comment states the design: *"The glob is over the VENDORED directory
  because that is what defines corpus membership."* But `forkbuiltRecordPins`
  (`md/conformance_keyed_test.go:207-211`) is a **second** definition of
  membership, and the glob does not include it.
- The membership assertion (`md/conformance_keyed_test.go:237-254`) compares
  `forkbuiltPinnedRecordSet` (a directory glob) against `forkbuiltRecordPins`.
  Both sets still contain `keyed_tr_multi_a` after the deletion, so both loops
  are silent.
- The acceptance measurement — *"43 correct-header plus 3 pinned-legacy"*, which
  the plan states as T3's acceptance — is a `t.Logf` at
  `md/conformance_keyed_test.go:279`. Nothing asserts `legacyPassed == 3`, or
  that any particular number of vectors ran. The only floor is
  `passed+failed == 0`, which `eachKeyedVector`'s own fatal already precludes.
  CI runs `CGO_ENABLED=0 go test -timeout 20m ./...`
  (`.github/workflows/test.yml:75`) with no `-v`, so that log line is never even
  printed in CI.

### Why this is not academic

F-529's stated premise is *"a re-vendor would DELETE these three vectors"*. That
is the event the whole T2 pinning exercise exists to survive. Today the
deletion is loud — verified:

```
$ go test ./md/            # with keyed_tr_multi_a.* removed from vectors/
--- FAIL: TestComposeVectorsMatchTheirProvenancePin
    pinned file missing: open testdata/vectors/keyed_tr_multi_a.bytes.hex: ...
--- FAIL: TestEveryKeyedComposeVectorHasAConformanceRecord
    keyed_tr_multi_a: stat testdata/vectors/keyed_tr_multi_a.conformance.json: ...
```

— but **the prescribed remedy for those two failures is the thing that opens the
hole**: re-run `scripts/vendor-compose-vectors.sh` (which regenerates the
provenance pin from whatever the primary now ships) and delete the name from
`composeVectorNames`. After that human action, both of those tests are green
again, the pins are orphaned, and nothing objects. Note also that T3 is what
*created* this loud backstop, by adding the 14 names to `composeVectorNames`;
before this cycle the deletion was silent everywhere.

### Blast radius beyond the descriptor gate

Every keyed cross-language gate iterates the same vendored glob and drops the
names with it:

| gate | iterates |
| --- | --- |
| `md/TestKeyedConformanceAgreesWithRust` | `eachKeyedVector(t,"keyed_*")` |
| `md/TestKeyedConformanceDescriptorsAgreeWithTheirTemplates` | `eachKeyedVector(t,"keyed_*")` |
| `gui/TestEveryKeyedVectorReachesAnAddress` | `eachKeyedVector(t,"keyed_*")` |
| `gui/TestTaprootScriptPathMatchesRust` | `eachKeyedVector(t,"keyed_tr_*")` |
| `gui/TestWshWitnessScriptHashesToRustsAddress` | `eachKeyedVector(t,"keyed_wsh_*")` |

`gui/TestEveryKeyedVectorReachesAnAddress` is the sharpest of these: its
`refusedByPolicy` map (`gui/policy_address_test.go:140-151`) names all three
F-529 vectors and carries no "every listed name was seen" assertion, so those
three deliberate-refusal entries become stale exemptions that nothing checks —
the exact shape the surrounding comments spend twenty lines forbidding for
`stillUnsupported`.

What still covers the pins in that state: only tests that name them literally
(`md/duplicate_keys_test.go`, `md/f533_pinned_vectors_test.go`'s two shape
tests, `gui/composer_flow_test.go`), and **all of those read the CARD pins
only**. The three RECORD pins — the artifacts T2 created, whose custodian the
fork now is — would be read by nothing at all.
`TestPinnedVectorsAgreeWithTheVendoredCorpusOrWithARecordedMove` `t.Skip`s in
that state by design (`md/f533_pinned_vectors_test.go:140-142`).

### Why the plan and TDD could not catch it

Both tiers coexist today, so every count is right, every assertion fires, and
every test is green. The defect is a *missing thing at a moment* — the moment
the pin becomes load-bearing — not a wrong thing in a section.

### Remedy sketch (secondary to the proof)

Iterate the union of `eachKeyedVector(t, "keyed_*")` and `forkbuiltRecordPins`,
and turn the acceptance line into an assertion:
`legacyPassed == len(forkbuiltRecordPins)`. Both are a few lines and neither
changes a verdict today.

---

## M-1 — A pinned conformance record can be edited into a DIFFERENT POLICY undetectably. The identical edit on the vendored tier reds on sha256.

### Reproduction

In `md/testdata/forkbuilt/keyed_tr_multi_a.conformance.json`, change
`multi_a(2,` → `multi_a(1,` in **both** `template` **and** both
`chains[].descriptor` (BIP-380 checksums recomputed):

```
template: tr(@0/48'/0'/0'/2'/<0;1>/*,multi_a(1,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))
$ go test ./md/                                   ok  0.136s
$ go test ./gui/ -run 'ReachesAnAddress|TaprootScriptPathMatchesRust|
    WshWitnessScriptHashes|ConsentWarnsOnDuplicate|EveryAddressSurface|
    TheTwoAddressRoutes'                          ok  0.163s
```

The record now describes a 1-of-2 taproot policy while the card it is paired
with encodes 2-of-2, and every gate is green.

**Control — the same consistent edit on a vendored record:**

```
$ # vectors/keyed_wsh_multi_2of3.conformance.json, multi(2, -> multi(1, in
$ # template AND both descriptors
--- FAIL: TestComposeVectorsMatchTheirProvenancePin
    keyed_wsh_multi_2of3.conformance.json: sha256 644a3dd157c5d3…,
    pin says e2f4f631523aef24…
```

### Why

D1′ compares `chains[].descriptor` against `template` — **two fields of one
file**. On the vendored tier that file is itself sha256-pinned in
`compose_vectors.provenance.json`, so a both-sides edit is caught by the pin.
Nothing hashes `md/testdata/forkbuilt/*.conformance.json`. The record's ids and
addresses *are* bound to the card, but its `template`/`descriptor` pair is not:
`WalletDescriptorTemplateId` is computed from the CARD and compared to the
record's id FIELD, never to `rec.Template`.

This is the plan's own r5 I-1 observation (*"mutating `template` alone leaves
the whole surface green"*) surviving into the pinned tier, where D1′ — justified
in the plan as *"the primary's source of truth against the primary's rendering
of it — input against output"* — degenerates to self-consistency, because on
this tier the fork authors both sides.

### What is NOT wrong

D5g's *"only the header arm relaxes, nothing else does"* is true clause by
clause. Single-sided edits to a pinned record are caught, verified in both
directions:

| pinned-record mutation | result |
| --- | --- |
| descriptor only, `/0/*` → `/7/*` (checksum fixed) | `chain 0: the descriptor does not reduce to the record's own template` |
| `template` only, `multi_a(2,` → `multi_a(1,` | same, on both chains |

What is weaker than the plan claims is the *tier* comparison: the plan says
these three *"need more scrutiny than the vendored tier, not less"*; measured,
they have strictly less — no provenance hash, no upstream input to compare
against, and no re-vendor that would ever correct them.

---

## M-2 — The CARD pin tier's membership is inferred from a file's existence, which is precisely what the plan forbade for the record tier.

`vectorChunksFor` (`md/vector_fixtures_test.go:85-102`) and `loadVectorChunks`
(`gui/vector_fixtures_test.go:65-84`) both prefer
`md/testdata/forkbuilt/<name>.md1.txt` on existence alone. Nothing enumerates
that directory: `allPinnedVectors()` (`md/f533_pinned_vectors_test.go:56-58`)
and `dupSeatVectors` (`md/dup_seat_fixture_test.go:34`) are hand-declared lists,
and `forkbuiltPinnedRecordSet` globs `*.conformance.json` only.

**Reproduction — a smuggled card pin is accepted in silence:**

```
$ cp md/testdata/vectors/keyed_wsh_or_b.phrase.txt \
     md/testdata/forkbuilt/keyed_wsh_or_b.md1.txt
$ go test -count=1 ./md/                                    ok  0.059s
$ go test -count=1 ./gui/ -run 'ReachesAnAddress|Taproot…'  ok  0.163s
```

Both packages now read that file in preference to the vendored card for every
`keyed_wsh_or_b` test, and no assertion mentions it. It is inert today (the
bytes match), and becomes load-bearing at the next re-vendor — at which point
the pairing split does red loudly, so the failure mode is loud rather than
silent. But the plan's argument for the record tier applies verbatim and was not
applied here: *"a fourth pinned record is a deliberate, visible act or it is a
bug."*

Verified positively for the record tier, which does have the assertion, in both
directions the plan predicted:

- pinning a 4th record that carries a **defective** (legacy-shaped) header →
  the plan's own M17: gate reads 46/46, membership `t.Errorf` fires.
- pinning a 4th record that carries a **correct** header → verified here, BOTH
  clauses fire:
  `testdata/forkbuilt/keyed_wsh_multi_2of3.conformance.json is pinned but is not
  one of the 3 names…` and `chain 0: pinned record's header is depth 4 child
  2147483650 under 4 origin component(s)…`

That assertion also closes the gui side, which has no membership check of its
own — `gui/loadVectorRecord` prefers the same directory on existence alone, and
md's assertion is what makes a "green it by pinning it" move visible there too.

---

## M-3 — The implementer's "every new assertion proven able to fail" is not literally true: D1's parent-fingerprint clause has no mutation in the 23-row table.

`md/conformance_keyed_test.go:405-408` asserts `parsed.parentFP == 0` for every
descriptor key on **both** tiers. It is an independent sub-clause: D2a/D2b match
on (chain code ‖ compressed pubkey), which a parent-fingerprint change does not
touch, and D1′ replaces the whole key with `@N`, so neither sees it. No row of
the implementation report's mutation table exercises it (M1 fires depth+child
only).

Verified independently that it fires, and fires **alone**: re-encoding every
descriptor xpub of `keyed_wsh_multi_2of3` with parent fingerprint `deadbeef`
(checksums recomputed) produces six lines, all of them this clause, and no
D1/D1′/D1″/D2/D3 line:

```
chain 0: [73c5da0a/48'/0'/0'/2'] xpub carries parent fingerprint deadbeef,
         want 0 — the parent point is not on the md1 wire
…
descriptor gate: 45 of 46 vectors pass (42 correct-header, 3 pinned-legacy), 1 fail
```

So this is a records defect, not a code defect — but the table is the artifact a
future reader will trust, and its header over-claims.

---

## Nits

**N-1 — the "no longer diverges" clause is strictly dominated and can never fire
alone.** `md/conformance_keyed_test.go:445-449` reports `%s no longer diverges
(both sides say %s)` when `goPath == bracketPath`. Clause 2 above it pins the
record's bracket to `[73c5da0a]`, so `bracketPath` is `bip32.Path(nil).String()`
== `"m"` (`bip32/bip32.go:20-35`). Clause 3 therefore requires `goPath == "m"`,
which necessarily violates clause 1's pin (`"m/84h/0h/0h"` / `"m/86h/0h/0h"`) —
so every state that fires clause 3 already fires clause 1, with a message that
names the actual change. Not a coverage gap: nothing goes unreported. Recorded
because the brief asked whether each new clause can fail.

**N-2 — `gui/loadVectorChunks` lacks md's empty-pin guard.**
`md/vectorChunksFor` fatals with *"the pin for %s carries no md1 strings"*
(`md/vector_fixtures_test.go:88-90`); the gui twin
(`gui/vector_fixtures_test.go:65-84`) returns an empty slice for a present but
contentless `forkbuilt/<name>.md1.txt`, with no guard and no fallback to the
vendored card. The two loaders are otherwise documented as the same rule.

**N-3 — the elided-origin bracket pin is documented as `verbatim and complete`
but is case-normalised.** `md/conformance_keyed_test.go:359` lowercases the
fingerprint before it is spliced back for the comparison at `:440`, so
`[73C5DA0A]` would satisfy a pin written `[73c5da0a]`. Separately, D1′'s
`strict` reduction writes `@N` + `originText` and drops the bracket fingerprint
entirely, so a fingerprint case flip is invisible to it as well (the path
hardening spelling, which is what D1′'s strictness was argued for, IS preserved
verbatim). Cosmetic today; `keys[].index`-level correctness is unaffected.

**N-4 — a duplicated `keys[].index` defeats the ambiguity refusal in one
direction.** `recByIndex[k.Index] = parsed.material`
(`md/conformance_keyed_test.go:297`) silently overwrites, so two `keys[]`
entries sharing an index collapse to one before `slotsByMaterial` ever sees
them. The refusal catches duplicated *material*, not duplicated *index*.
Unreachable from the primary's emitter.

---

## Verified clean — do not re-derive

Checked adversarially and found sound; recorded so the controller does not
spend another round on them.

- **All ten routed record reads are correctly paired.** A repo-wide grep for
  `conformance.json` leaves only error strings, the two fixtures files, and the
  one deliberate exception (`md/f533_pinned_vectors_test.go:156`, which reads
  both tiers because comparing them is its job and says so). Every site that
  unmarshals a record takes its card from a pin-preferring loader — verified by
  reading all 12 `loadVectorRecord`/`vectorRecordFor` call sites and their
  neighbouring card loads. No read was missed: `gui/bundle_testdata_test.go:41`
  and `gui/md1_gather_test.go:29` read a vendored `.phrase.txt` directly but
  read no record, which is exactly what D5b permits; `md/compose_shape_test.go`
  and `md/compose_test.go` keep `loadPhraseChunks` for the same reason.
- **`vectorChunksFor` was moved verbatim** out of `md/duplicate_keys_test.go` —
  the 28 deleted lines are the function and its doc comment, nothing else.
- **A vendored record cannot be routed into the legacy arm.** `legacy :=
  pinned[name]` (a glob of `forkbuilt/*.conformance.json`) and
  `vectorRecordFor`'s pin preference read the same path, so the flag and the
  bytes cannot disagree — and in the one state where they could (the file
  globbable but unreadable) the legacy arm fails closed on the correct-header
  record.
- **The legacy arm binds everything except depth/child.** Material is bound to
  `keys[]` (D2a) and to the Go expansion of the same card (D2b); the bracket
  fingerprint to `ExpandedKey.Fingerprint` (D2c); the bracket path to
  `ExpandedKey.OriginPath` (D3) and, verbatim, to the template (D1′); the
  checksum to `bip380.ValidChecksum` (D1″). `parentFP == 0` and `len(comps) > 0`
  are asserted in the legacy arm too. Apart from M-1's both-sides case, no
  single-field defect I could construct reached it.
- **`bip380`'s change is exactly the rename.** Two hunks, the doc comment and
  the identifier, plus the single call site at `bip380/bip380.go:273`. `grep -rn
  validChecksum` over the tree returns nothing.
- **The vendor script does what T3 says.** `^(keyed_|compose_)` minus
  `^compose_refusal_` selects **246 of 308** files in the primary at
  `b2c5d693`, and the sorted selection is **byte-for-byte identical** to the
  246 names in `md/testdata/compose_vectors.provenance.json` (`vectors: 50`,
  `commit b2c5d69384327265da239dc13502727c8f9645fb`). The refusal pin carries
  exactly `compose_refusal_keyless_cap.json`, so `isComposeVectorFile`'s
  exclusion is single-valued. No `keyed_compose_refusal_*` exists upstream, so
  the two exclusions (prefix in the script, pin-membership in the test) agree
  today.
- **`isComposeVectorFile`'s widened directory scan has no false positives.**
  `comm` against the primary shows **zero** fork-only files in
  `md/testdata/vectors/` whose name begins `keyed_` (the fork-only fixtures are
  `gap_*`, `keyless_*`, `seat_*`, `multisig_*`, `singlesig_*`), and no stray
  `compose_*`/`keyed_*` file is unpinned.
- **The three fork-authored conformance records** (`gap_tr_leaf_and_v`,
  `gap_tr_leaf_pkh`, `seat_same_origin_two_masters`) carry **no** `descriptor`
  field, so the gate's `keyed_*` scope leaves nothing uncovered there.
- **D6's `primaryMovedTo` arm is live, not dead.** All three pins now differ
  from their vendored cards (md5 of the md1 lines differs for each), so the
  recorded-move branch executes rather than short-circuiting at
  `slices.Equal`.
- **The ambiguity refusal is a well-placed guard, not a latent false RED.** Two
  slots can only collide on the 65 bytes if the same account key is seated
  twice, which is BIP-forbidden key reuse; the legitimate "one card fills two
  slots" shape (`seat_same_origin_two_masters`,
  `wsh(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/0'/2'/<0;1>/*))`) carries two
  **distinct** xpubs — verified from its `keys[]`.
- **The gate runs in CI.** `.github/workflows/test.yml:75` is
  `CGO_ENABLED=0 go test -timeout 20m ./...`.
- **The re-vendor retired no assertion.** `md/compose_shape_test.go`'s
  `keyed_wsh_timelock_hashlock` row (the one site still on the vendored card for
  a pinned name) asserts the same two branches against the reuse-free policy and
  is not vacuous; the duplicate-shape assertions all moved to the pin.
