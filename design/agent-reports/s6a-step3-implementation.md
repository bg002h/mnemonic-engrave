# S6a step 3 — implementation report

**Agent:** single implementer, step 3 of the §4.8 build order.
**Worktree:** `/scratch/code/shibboleth/wt-s6a`, branch `s6a-singlesig-truth`.
**Parent:** `c729176992f34f80de88a3d87b327fc09b14f0b9` (step 2).
**Commit:** `112d537f88b8a669f1f4a6529362b58d61f37c56`.
**Working tree after commit:** clean. Not pushed, not merged, not rebased.
`/scratch/code/shibboleth/seedhammer` untouched, still `b8a23bf`, clean.

---

## 1. Files changed

| file | ± | what |
| --- | --- | --- |
| `gui/multisig_build_census.go` | +183/−31 | `seedCapacity`, `buildSeedHandlingRuling`, `buildSeedInventoryLines`, new `buildPlateInventoryLines` signature + placement |
| `gui/multisig.go` | 1 line | SUPPLY-path call site → `seedCapacityOne` |
| `gui/multisig_build.go` | 1 line | BUILD-path call site → `seedCapacityMany` |
| `gui/multisig_build_prose_test.go` | 3 lines | 3 call sites → `seedCapacityMany` |
| `gui/multisig_build_perseed_passphrase_test.go` | 3 lines | 3 call sites → `seedCapacityMany` |
| `gui/singlesig_truth_test.go` | **+201/−0** | T4 and T7 **appended** |

Total: 6 files, 369 insertions, 31 deletions.

### A correction to the plan's test-file assumption

§5 says *"New file: `gui/singlesig_truth_test.go`"*. **That file already existed** —
step 2 created it and it holds T21, T22 and T26. Step 3's rows were **appended**,
not written over. `git diff --numstat` on that file reads `201  0` (201 added,
**zero** deleted); the only edit inside the pre-existing text is the import block
gaining `seedhammer.com/bundle`. All three step-2 tests were re-run and pass.

---

## 2. The call-site count — measured before and after

### Before (at `c729176`, tree clean)

```
$ grep -rn 'buildPlateInventoryLines(' --include='*.go' . | grep -v 'func buildPlateInventoryLines'
gui/multisig_build_prose_test.go:369:	ruling := strings.Join(buildPlateInventoryLines(cards, oneSeedPassphraseFact(false)), "\n")
gui/multisig_build_prose_test.go:424:	with := strings.Join(buildPlateInventoryLines(cards, oneSeedPassphraseFact(true)), "\n")
gui/multisig_build_prose_test.go:425:	without := strings.Join(buildPlateInventoryLines(cards, oneSeedPassphraseFact(false)), "\n")
gui/multisig_build.go:479:			buildPlateInventoryLines(cardsOut, reg.passphraseFacts()))
gui/multisig.go:362:		buildPlateInventoryLines(cardsOut, oneSeedPassphraseFact(passphrase != "")))
gui/multisig_build_perseed_passphrase_test.go:134:			doc := strings.Join(buildPlateInventoryLines(cards, facts), "\n")
gui/multisig_build_perseed_passphrase_test.go:246:	doc := strings.Join(buildPlateInventoryLines(cards, facts), "\n")
gui/multisig_build_perseed_passphrase_test.go:304:	doc := strings.Join(buildPlateInventoryLines(cards, facts), "\n")
```

**Count = 8.** Matches §4.3 and the dispatch brief exactly, both in count and in
file:line. It does **not** match §4.8's step-3 cell or §5.1(a)'s "six", which the
plan already records as a corrected stale count.

Capacity assignment, all 8: **7 × `seedCapacityMany`**, **1 × `seedCapacityOne`**
(`gui/multisig.go:362`, per §3.1.1 — the SUPPLY path holds one seed by
construction at `gui/multisig.go:355`).

### After (at `112d537`)

```
$ grep -rn 'buildPlateInventoryLines(' --include='*.go' . | grep -v 'func buildPlateInventoryLines'
gui/multisig_build_prose_test.go:369:	ruling := strings.Join(buildPlateInventoryLines(cards, oneSeedPassphraseFact(false), seedCapacityMany), "\n")
gui/multisig_build_prose_test.go:424:	with := strings.Join(buildPlateInventoryLines(cards, oneSeedPassphraseFact(true), seedCapacityMany), "\n")
gui/multisig_build_prose_test.go:425:	without := strings.Join(buildPlateInventoryLines(cards, oneSeedPassphraseFact(false), seedCapacityMany), "\n")
gui/multisig.go:362:		buildPlateInventoryLines(cardsOut, oneSeedPassphraseFact(passphrase != ""), seedCapacityOne))
gui/multisig_build_perseed_passphrase_test.go:134:			doc := strings.Join(buildPlateInventoryLines(cards, facts, seedCapacityMany), "\n")
gui/multisig_build_perseed_passphrase_test.go:246:	doc := strings.Join(buildPlateInventoryLines(cards, facts, seedCapacityMany), "\n")
gui/multisig_build_perseed_passphrase_test.go:304:	doc := strings.Join(buildPlateInventoryLines(cards, facts, seedCapacityMany), "\n")
gui/singlesig_truth_test.go:327:			buildPlateInventoryLines(cards, oneSeedPassphraseFact(false), capacity), "\n")
gui/singlesig_truth_test.go:424:	doc := strings.Join(buildPlateInventoryLines(
gui/singlesig_truth_test.go:461:				for _, line := range buildPlateInventoryLines(
gui/multisig_build.go:479:			buildPlateInventoryLines(cardsOut, reg.passphraseFacts(), seedCapacityMany))
```

**Count = 11** = the original 8 (all now carrying a capacity) + 3 new sites in the
step-3 tests. The three sites showing no `seedCapacity` token on their own line
are the two line-wrapped calls in the test file and the closure that takes
`capacity` as a parameter.

---

## 3. THE NINTH CALL SITE IS NOT IN THIS COMMIT — and cannot be

**This is the one place the implementation departs from the plan's build order,
and it is a plan self-contradiction rather than a judgement call.**

§4.8 step 3 and §5.1(a) both schedule a **new ninth** call site at
`gui/singlesig.go:136` to this step. That line is today:

```go
restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path)
```

and the inventory reaches the document only as an argument to `restoreDocFlow`.
`restoreDocFlow` (`gui/singlesig_restore.go:119`) does not accept one:
**§4.2 gives it `status string` and `extra []string` at step 4**, and §4.8 **step
5** separately owns *"Wire single-sig: label (§4.1), **inventory**, census (§4.6),
abort gate (§4.5)"*. So three sections of the plan schedule the same edit to three
different steps, and the two later ones are the consistent pair: a ninth call site
at step 3 is not compilable without step 4's signature change.

The dispatch brief independently forbids the signature change (*"No
`restoreDocFlow`/`multisigRestoreDocFlow` signature change (step 4)"*, *"No flow
changes"*), which resolves it the same way. **Step 3 lands the shared census only.**
The ninth site arrives with step 4/5, taking `seedCapacityOne`.

Recommended plan fix: §4.8 step 3's cell should read *"…updating all EIGHT
existing call sites"* and drop *"plus the new ninth"*; §5.1(a)'s ninth-site
sentence should say the site appears **at step 4/5**, not here.

---

## 4. The strings emitted, so a reviewer can diff them against §4.3/§4.4

Dumped by executing the shipped functions (temporary dump test, removed before
commit) — not transcribed from source.

### 4.1 The seed-handling ruling — `buildSeedHandlingRuling(capacity, seedOnPlates)`

**`seedCapacityMany`, `seedOnPlates=true`** — the BUILD path's full-mode arm:

> Seed handling: this build does not time out. Every seed you entered -- this build can hold several -- stays in device memory until the build ends, and on a full build the words are also on the plates as they are cut. Do not leave a mid-build machine unattended: the plates are the secret. Power the device off when you are done.

**BYTE-IDENTICAL to the shipped sentence.** §4.3 claims this; it is now *machine*
checked, by a whole-string `!=` in T7 against a literal transcribed from the plan,
not by eye. The multisig BUILD path's full-mode document does not churn.

**`seedCapacityMany`, `seedOnPlates=false`** — BUILD path, watch-only:

> Seed handling: this build does not time out. Every seed you entered -- this build can hold several -- stays in device memory until the build ends. Do not leave a mid-build machine unattended: it is still holding seed material. Power the device off when you are done.

**`seedCapacityOne`, `seedOnPlates=true`** — SUPPLY path full mode (and single-sig
full, once step 5 wires it):

> Seed handling: this build does not time out. The seed you entered -- this build holds exactly one -- stays in device memory until the build ends, and on a full build the words are also on the plates as they are cut. Do not leave a mid-build machine unattended: the plates are the secret. Power the device off when you are done.

**`seedCapacityOne`, `seedOnPlates=false`** — SUPPLY/single-sig watch-only:

> Seed handling: this build does not time out. The seed you entered -- this build holds exactly one -- stays in device memory until the build ends. Do not leave a mid-build machine unattended: it is still holding seed material. Power the device off when you are done.

All four match §4.3's assembly (`base` + subject + the two tails) character for
character, including the retained-on-purpose vestigial *"on a full build"*.

### 4.2 The seed statement — `buildSeedInventoryLines(cards)`

**Absence (no ms1 card — watch-only, any path):**

> Seed: this set contains NO seed. It is watch-only: it records the wallet, but it can never spend. If funds must be recovered, the seed words must come from somewhere else -- no plate in this set holds them.

**Presence, exactly one ms1 card:**

> Seed: this set contains YOUR seed, on the plate marked 'ms1 secret share'. Treat that plate as the secret itself.

**Presence, several ms1 cards:**

> Seed: this set contains YOUR seeds, on the plates marked 'ms1 secret share'. Treat each of those plates as the secret itself.

All three match §4.4 verbatim. ASCII only (`--`, straight quotes); the glyph guard
is asserted over the whole cross-product in T7.

### 4.3 Placement inside `buildPlateInventoryLines`

Exactly §4.4's order — verified by reading a rendered document out of a test
failure, not from the source:

```
This backup is 4 plates:
ms1 secret share: 1 plate (secret seed backup)
mk1 key: 2 plates (account key card)
md1 descriptor: 1 plate (wallet policy descriptor)
If any of them is missing, this backup is incomplete.
Seed: this set contains YOUR seed, on the plate marked 'ms1 secret share'. Treat that plate as the secret itself.
No BIP-39 passphrase was used, so no passphrase is needed to spend from this wallet.
Seed handling: this build does not time out. The seed you entered -- this build holds exactly one -- stays in device memory until the build ends, and on a full build the words are also on the plates as they are cut. Do not leave a mid-build machine unattended: the plates are the secret. Power the device off when you are done.
```

1 plate list + completeness claim → 2 seed statement → 3 passphrase statement →
4 ruling. ✔

### 4.4 What deliberately churns

Both changes are the ones §4.3 predicts, and both are corrections:

- **multisig SUPPLY, any mode** — moves onto the one-seed subject. Its document
  said *"this build can hold several"*, which is already false there.
- **any path, WATCH-ONLY** — loses the *"the words are also on the plates"* /
  *"the plates are the secret"* pair, which is false on every watch-only run.
- **Additionally**, F-195's seed statement is a new line on **every** document
  built through `buildPlateInventoryLines` — both multisig paths today, single-sig
  once step 5 wires it. That follows from §4.4's placement decision and is what
  §4.4's one-slot-multisig argument is about.

---

## 5. Tests — which rows land here, and which do not

### In this commit

- **T4** — *watch-only document contains the absence line; full contains the
  presence line.* §5 names it as the step-3 row.
  `TestRestoreDocSaysWhetherTheSetContainsASeed`.
- **T7** — *`seedCapacityOne` yields "The seed you entered" and not "Every seed";
  `seedCapacityMany` yields "Every seed"; every new operator string is
  ASCII-clean.* Pure functions over the ruling and the inventory, so it lands here.
  `TestSeedHandlingRulingIsKeyedOnCapacityAndOnThePlates`.

Both assert **through `buildPlateInventoryLines`** wherever a document-shaped
claim is made, rather than only on the arm-picking helpers underneath. A helper
that returns the right string and is never reached is the shape that let the
multisig instance of this defect ship. T4's single-sig fixtures come from
`singleSigEngraveCards`, so the arms are chosen by the card shapes that flow
actually cuts.

Two things T7 covers that §5.2 flags as otherwise uncovered:

- The **seedless fixture** is built on purpose — three existing tests run the
  glyph guard, and all three build it over ms1-**bearing** cards.
- The `seedOnPlates` axis is asserted in both directions, which is the R0 I-2
  half the first draft of §4.3 did not audit.

### Judged to belong to later steps, with reasons

| row | why not here |
| --- | --- |
| **T1, T3** | assert the engrave-mode label (`buildFullModeLabel` at `gui/singlesig.go:80`) — §4.1, step 5 |
| **T2** | needs the single-sig restore document to carry an inventory at all — that seam is step 4 + step 5. §5.2 also requires an engraver-driven walk and `s5PageForNeedle`, neither of which exists on this path yet |
| **T5** | the abort gate (§4.5) and the census press (§5.1b) — steps 5 and 6 |
| **T6** | the pre-engrave census (§4.6) — step 5 |
| **T7c** | explicitly step 6 per §4.8 and the brief: it drives **all three flows to their restore documents**, and the single-sig one does not exist until step 5 |
| **T8** | the three false comments (§4.7c) — step 8, deliberately last |
| **T11, T20, T23, T24, T25, T26(T27)** | verify-status wiring into flows — step 7. T21/T22/T26 already landed at step 2 and are untouched |

**§5.2's own note confirms the shape chosen here:** *"T4 at unit level proves
nothing about the single-sig document — that seam is carried by T2 alone. Fine as
designed."*

---

## 6. Red → green → mutation, per test

### 6.1 Red, phase 0 — the tests against the unchanged tree

As §5 predicts for rows targeting symbols this cycle introduces, this is a
**compile failure, not an assertion failure**, and it proves nothing on its own:

```
FAIL	seedhammer.com/gui [build failed]
gui/singlesig_truth_test.go:61:43: undefined: seedCapacity
gui/singlesig_truth_test.go:63:66: too many arguments in call to buildPlateInventoryLines
	have ([]bundleCard, []seedPassphraseFact, unknown type)
	want ([]bundleCard, []seedPassphraseFact)
gui/singlesig_truth_test.go:66:48: undefined: seedCapacityOne
...
gui/singlesig_truth_test.go:129:12: undefined: buildSeedHandlingRuling
```

### 6.2 Red, phase 1 — a GENUINE red for T4

Implementation was staged in two parts so T4 could have a real assertion failure.
With the capacity axis wired but **no seed statement**, T4 failed on three real
rendered documents and T7 passed:

```
--- PASS: TestSeedResidencyRulingDescribesTheMultiSeedReality (0.00s)
--- PASS: TestRestoreDocSaysThePassphraseIsNotOnThePlates (0.00s)
--- FAIL: TestRestoreDocSaysWhetherTheSetContainsASeed (0.00s)
    singlesig_truth_test.go:68: the watch-only single-sig document does not say the set contains NO seed. Silence is what a reader mistakes for a lost plate:
        want "Seed: this set contains NO seed. It is watch-only: it records the wallet, but it can never spend. If funds must be recovered, the seed words must come from somewhere else -- no plate in this set holds them."
        got:
        This backup is 3 plates:
        mk1 key: 2 plates (account key card)
        md1 descriptor: 1 plate (wallet policy descriptor)
        If any of them is missing, this backup is incomplete.
        No BIP-39 passphrase was used, so no passphrase is needed to spend from this wallet.
        Seed handling: this build does not time out. The seed you entered -- this build holds exactly one -- stays in device memory until the build ends. Do not leave a mid-build machine unattended: it is still holding seed material. Power the device off when you are done.
    singlesig_truth_test.go:79: the full single-sig document does not say which plate carries the seed: ...
    singlesig_truth_test.go:97: a set carrying two ms1 plates does not name them in the plural ...
--- PASS: TestSeedHandlingRulingIsKeyedOnCapacityAndOnThePlates (0.00s)
FAIL	seedhammer.com/gui	0.065s
```

### 6.3 Green

```
--- PASS: TestSeedResidencyRulingDescribesTheMultiSeedReality (0.00s)
--- PASS: TestRestoreDocSaysThePassphraseIsNotOnThePlates (0.00s)
--- PASS: TestVerifyStatusZeroCellIsTheDefault (0.00s)
--- PASS: TestVerifyPassLineIsGeneratedPerMode (0.00s)
--- PASS: TestVerifyPassLineClausesAreEachBackedByARecord (0.00s)
--- PASS: TestRestoreDocSaysWhetherTheSetContainsASeed (0.00s)
--- PASS: TestSeedHandlingRulingIsKeyedOnCapacityAndOnThePlates (0.00s)
ok  	seedhammer.com/gui	0.080s
```

The two step-2 verify-status tests and the three step-2 rows are in that list on
purpose: they prove the appended file did not disturb them.

### 6.4 Mutations — five applied, five failed, each proving the mutated line RAN

**Every mutation was reverted from a pristine copy taken before the first one, and
the final tree is byte-identical to the green tree.**

#### MUT-1 — T4's own named mutation: *swap the arms of `buildSeedInventoryLines`*

The absence and presence-singular **return blocks were exchanged**. `grep` after
the edit confirms the swap landed:

```
208:			"Seed: this set contains YOUR seed, on the plate marked 'ms1 secret " +
220:			"Seed: this set contains NO seed. It is watch-only: it records the " +
```

`TestRestoreDocSaysWhetherTheSetContainsASeed` **FAILED** (exit 1). **Proof the
mutated line ran, not merely that the edit landed** — the failure prints the
rendered documents with the arms exchanged:

```
    singlesig_truth_test.go:68: the watch-only single-sig document does not say the set contains NO seed...
        This backup is 3 plates:
        mk1 key: 2 plates (account key card)
        md1 descriptor: 1 plate (wallet policy descriptor)
        If any of them is missing, this backup is incomplete.
        Seed: this set contains YOUR seed, on the plate marked 'ms1 secret share'. Treat that plate as the secret itself.
    singlesig_truth_test.go:73: the watch-only single-sig document claims a seed is on these plates. No ms1 is engraved in watch-only mode: ...
    singlesig_truth_test.go:79: the full single-sig document does not say which plate carries the seed:
        This backup is 4 plates:
        ms1 secret share: 1 plate (secret seed backup)
        ...
        Seed: this set contains NO seed. It is watch-only: it records the wallet, but it can never spend...
```

A watch-only document asserting a seed plate exists, and a seed-bearing document
asserting there is none — the exact G1/G2 harm, caught.

#### MUT-2 — T4, plural arm: `if ms1s == 1` → `if ms1s >= 1`

Makes the plural arm unreachable. **FAILED**, and the printed document proves the
line ran — a two-ms1 set described in the singular:

```
    singlesig_truth_test.go:97: a set carrying two ms1 plates does not name them in the plural, so the document points at one plate while the set is two:
        This backup is 3 plates:
        ms1 secret share 1 of 2: 1 plate (seed)
        ms1 secret share 2 of 2: 1 plate (seed)
        mk1 key: 1 plate (key)
        If any of them is missing, this backup is incomplete.
        Seed: this set contains YOUR seed, on the plate marked 'ms1 secret share'. Treat that plate as the secret itself.
    singlesig_truth_test.go:102: a two-ms1 set is described as though it held a single seed plate: ...
```

#### MUT-3 — T7's named mutation: *swap the capacity arms*

`if capacity == seedCapacityMany` → `if capacity == seedCapacityOne`. **FAILED**,
and so did the **existing** guard `TestSeedResidencyRulingDescribesTheMultiSeedReality`
— which is exactly what §4.3 predicts and is the evidence it was **not** weakened:

```
--- FAIL: TestSeedResidencyRulingDescribesTheMultiSeedReality (0.00s)
    multisig_build_prose_test.go:383: the ruling does not say the machine holds EVERY seed entered:
        ... Seed handling: this build does not time out. The seed you entered -- this build holds exactly one -- ...
--- FAIL: TestSeedHandlingRulingIsKeyedOnCapacityAndOnThePlates (0.00s)
    singlesig_truth_test.go:130: the multi-seed, seed-bearing ruling is no longer byte-identical to the S5-reviewed sentence...
        want "Seed handling: this build does not time out. Every seed you entered -- this build can hold several -- ..."
        got  "Seed handling: this build does not time out. The seed you entered -- this build holds exactly one -- ..."
    singlesig_truth_test.go:137/140/145/149: (four more, both directions on both capacities)
```

#### MUT-4 — T7's second named mutation: *insert an em dash*

`"holding seed material. Power the device off…"` →
`"holding seed material — power the device off…"`. **FAILED**, four times — once
per cross-product cell that reaches the seedless arm, each printing the offending
line:

```
    singlesig_truth_test.go:200: an inventory line carries a glyph the body face lacks, so it does not draw:
        "Seed handling: ... it is still holding seed material — power the device off when you are done."
```

#### MUT-5 — the second axis: `if seedOnPlates` → `if true`

Not a plan-named row, applied because the `seedOnPlates` axis is R0 I-2's finding
and a test that cannot see it dropped would be a false PASS. **FAILED**, printing
the self-contradicting document the axis exists to prevent:

```
    singlesig_truth_test.go:163: a watch-only document says the plates are the secret, on a set whose own inventory says no plate in it holds the seed:
        ...
        Seed: this set contains NO seed. It is watch-only: it records the wallet, but it can never spend. If funds must be recovered, the seed words must come from somewhere else -- no plate in this set holds them.
        No BIP-39 passphrase was used, so no passphrase is needed to spend from this wallet.
        Seed handling: ... Do not leave a mid-build machine unattended: the plates are the secret. Power the device off when you are done.
    singlesig_truth_test.go:167: a watch-only document claims the words are on the plates. No ms1 is engraved on this run: ...
    singlesig_truth_test.go:175: the watch-only ruling dropped "still holding seed material"...
```

**5 of 5 mutations produced a red. No false PASS.**

---

## 7. The gate — streams separated

Streams kept in separate files throughout: `nix` writes
`warning: Git tree '/scratch/code/shibboleth/wt-s6a' is dirty` to **stderr**, and
a `2>&1` capture has corrupted counts twice in this project.

### Baseline, recorded at `c729176` with a clean tree, before any edit

```
go test ./... -count=1     exit 0
stdout: 51 × "ok", 0 × "FAIL"
stderr: (empty — the tree was clean, so nix printed nothing)
```

### Final, at `112d537`

```
go build ./...             exit 0   stdout empty; stderr = the dirty-tree warning only
go test ./... -count=1     exit 0   stdout: 51 × "ok", 0 × "FAIL"
                                    stderr = the dirty-tree warning only
gofmt -l gui/                       no output
seedhammer.com/gui                  281.256s   ok
```

**The package/status set is byte-identical to the baseline** — `diff` over the
`(package, status)` columns of the two stdout captures is empty. Nothing
unrelated broke, and nothing was skipped or weakened to get there.

### One pre-existing, unrelated red, reported not fixed

`go vet ./gui/` fails, and failed before this step:

```
gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
```

That file is **not touched by this commit** (`git show --name-only HEAD` does not
list it) and `go.mod`'s `go` directive is unchanged. `go test` runs only a subset
of vet and is unaffected — the suite is green. Flagged for whoever owns the
toolchain bump; **not** in step 3's scope, and not something to paper over.

---

## 8. Everything that contradicted the plan

1. **The ninth call site is scheduled to a step that cannot compile it.** §4.8
   step 3 and §5.1(a) place a new `gui/singlesig.go:136` site here; §4.2 gives
   `restoreDocFlow` the parameter it would need at **step 4**, and §4.8 **step 5**
   separately owns wiring the single-sig inventory. Resolved in favour of steps
   4/5. See §3 above for the recommended plan edit.
2. **`gui/singlesig_truth_test.go` is not a new file.** §5 calls it one; step 2
   created it. Appended, +201/−0.
3. **§4.8 step 3 and §5.1(a) still say "six" in prose around a corrected "eight".**
   Measured 8, as §4.3 and the brief say. Noted because the stale count is in the
   two places an implementer follows.
4. **§4.4's placement puts the seed statement on the multisig documents too**, not
   only single-sig. That is what the section intends (its one-slot-multisig
   argument only makes sense if multisig gets the line), but it is a visible change
   to two shipped documents and is called out here so no reviewer reads it as
   overreach.

Nothing else. §4.3's byte-identity claim, §4.4's three arms and their exact
wording, and §3.1.1's SUPPLY-path capacity all held exactly as written.

---

## 9. Left for later steps — nothing from step 3 deferred

Untouched, as the brief requires:

- **No flow changes.** `singleSigVerifyFlow`, `multisigVerifyFlow` and every
  verify wiring are byte-unchanged.
- **No `restoreDocFlow` / `multisigRestoreDocFlow` signature change** (step 4).
- **No verify-status wiring** (step 7). `gui/verify_status.go` is unmodified.
- **No `rec *verifyRecord` out-parameter** anywhere.
- The ninth call site, the mode label (§4.1), the pre-engrave census (§4.6), the
  abort gate (§4.5), the three walk repairs (§5.1b), T7c, the three false comments
  (§4.7c) and the spec update (§3.1.7) — all still open, on their own steps.

No follow-ups filed. No item owned by step 3 was carried past it.
