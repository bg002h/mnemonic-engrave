# S6a step 4 — implementation report

**Worktree** `/scratch/code/shibboleth/wt-s6a`, branch `s6a-singlesig-truth`
**Parent** `112d537f88b8a669f1f4a6529362b58d61f37c56` (step 3)
**Commit** `7d317ffa937a29eaf6072b028ed916c130c6afb5`
**Scope** §4.8 step 4 only — `restoreDocFlow` and `multisigRestoreDocFlow` gain
`status` + `extra`, all four call sites threaded. Signature change and nothing
else. Not pushed, not merged, not rebased.
`/scratch/code/shibboleth/seedhammer` was not touched.

---

## 1. The call-site count — measured before and after

### The plan's own grep is wrong, and it under-reports by three

The brief supplied this command:

    grep -rn 'restoreDocFlow(' --include='*.go' . | grep -v 'func restoreDocFlow\|func multisigRestoreDocFlow'

Run at `112d537` it returns **ONE** line:

    gui/singlesig.go:136:	restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path)

It is **case-sensitive**, and `multisigRestoreDocFlow` carries a capital **R**.
The three multisig sites are invisible to it. Adding `-i` gives the real count:

    $ grep -rni 'restoreDocFlow(' --include='*.go' . \
        | grep -v 'func restoreDocFlow\|func multisigRestoreDocFlow'
    gui/multisig.go:361:	multisigRestoreDocFlow(ctx, th, tpl, keys,
    gui/multisig_nested_name_test.go:230:		multisigRestoreDocFlow(ctx, &descriptorTheme, tpl, keys, nil)
    gui/singlesig.go:136:	restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path)
    gui/multisig_build.go:478:		multisigRestoreDocFlow(ctx, th, tpl, keys,

**BEFORE = 4** (3 production + 1 test), matching the brief's table exactly,
including `gui/multisig_nested_name_test.go:230`, which passed `nil` and would
have been left at the old arity — a package-wide compile error, not a soft
failure. All four line numbers in the brief were re-verified and were correct.

The function declarations, also measured:

    gui/multisig_restore.go:100:func multisigRestoreDocFlow(ctx *Context, th *Colors, tpl md.Template, keys []md.ExpandedKey, extra []string) {
    gui/singlesig_restore.go:119:func restoreDocFlow(ctx *Context, th *Colors, xpub string, masterFP, parentFP uint32, script md.ScriptKind, path bip32.Path) {

### After

    $ grep -rni 'restoreDocFlow(' --include='*.go' . \
        | grep -v 'func restoreDocFlow\|func multisigRestoreDocFlow'
    gui/singlesig_truth_test.go:569:			restoreDocFlow(ctx, &descriptorTheme, knownAccountXpub84, knownMasterFP, pfp,
    gui/singlesig_truth_test.go:581:			multisigRestoreDocFlow(ctx, &descriptorTheme, tpl, keys, statusNeedle, extra)
    gui/multisig.go:367:	multisigRestoreDocFlow(ctx, th, tpl, keys,
    gui/singlesig.go:149:	restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path,
    gui/multisig_build.go:484:		multisigRestoreDocFlow(ctx, th, tpl, keys,
    gui/multisig_nested_name_test.go:233:		multisigRestoreDocFlow(ctx, &descriptorTheme, tpl, keys, verifyStatusNotFullyCheckedLine, nil)

**AFTER = 6** — the same four, plus the **two new call sites in this step's own
test**. Every one is at the new arity; the package compiles.

---

## 2. Files changed

| file | what |
| --- | --- |
| `gui/singlesig_restore.go` | `restoreDocFlow` signature + composition, +14 lines of comment |
| `gui/multisig_restore.go` | `multisigRestoreDocFlow` signature + composition, +7 lines of comment |
| `gui/singlesig.go` | call site + the placeholder rationale |
| `gui/multisig.go` | call site + the placeholder rationale |
| `gui/multisig_build.go` | call site + the placeholder rationale |
| `gui/multisig_nested_name_test.go` | the fourth call site, updated from `nil` |
| `gui/singlesig_truth_test.go` | **appended** (step 2 created it, step 3 appended); 180 added, 0 deleted |

`7 files changed, 236 insertions(+), 6 deletions(-)`.

---

## 3. The new signatures, verbatim

    func restoreDocFlow(ctx *Context, th *Colors, xpub string, masterFP, parentFP uint32, script md.ScriptKind, path bip32.Path, status string, extra []string) {

    func multisigRestoreDocFlow(ctx *Context, th *Colors, tpl md.Template, keys []md.ExpandedKey, status string, extra []string) {

Both compose identically, and this line is byte-identical in the two files:

    	restoreDocScreen(ctx, th, append(append([]string{status}, lines...), extra...))

This is §4.2's shape transcribed, not re-derived. `status` is "leading" in the
**slice**, not in the parameter list — §4.2's own sample call puts it
second-to-last, immediately before `extra`, and both functions follow that.

Incidental improvement, worth recording because it is a behaviour change nobody
asked for: the old multisig form was `append(lines, extra...)`, which can write
into `lines`' backing array. The new form always starts from a fresh
one-element slice, so neither flow can now alias the slice its line builder
returned.

---

## 4. What each of the four sites passes, and why it fails safe

**The rule this step had to satisfy:** the parameters have no real values yet, so
whatever is passed now is a claim in its own right for the length of steps 5–7.
If step 5 and step 7 never arrived, the document must say **less** than the
truth, never more (S6a G2 — omission must WEAKEN).

| # | site | `status` | `extra` |
| --- | --- | --- | --- |
| 1 | `gui/singlesig.go:149` | `verifyStatusNotFullyCheckedLine` | `nil` |
| 2 | `gui/multisig.go:367` | `verifyStatusNotFullyCheckedLine` | unchanged — `buildPlateInventoryLines(cardsOut, oneSeedPassphraseFact(passphrase != ""), seedCapacityOne)` |
| 3 | `gui/multisig_build.go:484` | `verifyStatusNotFullyCheckedLine` | unchanged — `buildPlateInventoryLines(cardsOut, reg.passphraseFacts(), seedCapacityMany)` |
| 4 | `gui/multisig_nested_name_test.go:233` | `verifyStatusNotFullyCheckedLine` | `nil` (unchanged) |

### Why `verifyStatusNotFullyCheckedLine` and not something else

1. **It is the weakest of the four §4.7c lines.** *"These plates were not fully
   checked. Confirm they restore this wallet (master fingerprint below) before
   relying on this backup."* It asks the reader to confirm and vouches for
   nothing.
2. **It is true today.** No flow records a verify bit until step 7, so every run
   in the tree right now genuinely is a run about which nothing was recorded.
   This is not a placeholder standing in for a fact — it *is* the fact.
3. **It is byte-identical to what the builder renders for a zero record**, and
   that is asserted, not assumed:
   `buildVerifyStatusLine(verifyRecord{}) == verifyStatusNotFullyCheckedLine`.
   So step 7's substitution of `buildVerifyStatusLine(rec)` for these literals is
   a **no-op on any run that recorded nothing**, and can only ever *strengthen* a
   document that earned it. A drifted placeholder would have made step 7 silently
   change every skipped-verify document, and nothing would have noticed.
4. **It does not call `buildVerifyStatusLine` from a flow** — the brief forbids
   that until step 7, so the constant is named directly. The constant and the
   call are pinned equal by test, which buys the equivalence without the wiring.

### The alternative, and why it is the one thing this step could have got wrong

`""` compiles, still occupies slice index 0, and **draws nothing**. The document
would be silent about its verification — and silence is precisely what
`gui/verify_status.go`'s own header says reads as a pass to the stranger who
finds the plates years later. That is an omission that **strengthens** a claim,
the single failure direction G2 forbids. It is also invisible: no compiler
catches it, and no rendering assertion catches it either, because an empty label
inks nothing. That is why the placeholder has a test of its own rather than a
comment.

### Why `extra` is `nil` at site 1

The single-sig inventory is **step 5's** (§5.1: the ninth
`buildPlateInventoryLines` call site is step 5, not step 3 and not step 4). `nil`
means the document says nothing about the set — which is exactly what it says
today, and which is weaker than saying something wrong about it. Sites 2 and 3
keep the inventories step 3 gave them, unchanged.

---

## 5. Which §5 rows land here — none, and why

I judged that **no row of the plan's §5 test table lands at step 4**, and the
build-order table agrees: step 4's cell names no tests, while steps 2, 3, 5, 6, 7
and 8 all name theirs.

| row | owning step | why not here |
| --- | --- | --- |
| T1, T2, T3, T4(doc half), T5, T6 | 5 (+6) | need the single-sig label / inventory / census / abort wiring, none of which exists |
| T4 (unit), T7 | 3 | landed |
| T7c | 6 | needs the three walks repaired past the census screen |
| T8 | 8 | the comment sweep |
| **T11** | **7** | asserts index 0 **through a production flow** *and* the §4.7f scope line under `statusCheckDidNotPass`. No flow records a status, and the scope line does not exist |
| **T20** | **7** | each of the four §4.7c cells rendering **its own** line on all three documents. Today every document renders the same zero-cell line, so all four cells would be "asserted" by one string |
| **T23, T24** | **7** | stickiness and the retry path — need a multisig retry, which needs the record wired into the flow |
| **T25** | **7** | "no verdict is read" through the wired derivation |
| **T27** | **7** | the path axis on a rendered pass line, plus a fixture with `suppliedCosigners > 0` |
| T21, T22, T26 | 2 | landed |

Writing any of T11/T20/T23/T24/T25/T27 now would have produced a test that
passes without reaching the sequence it names — the exact vacuity §5 warns about
for T5 and T27.

### What I wrote instead, and why it is not those rows

Step 4 changes something real and observable *today*: **where the two new
parameters land in the document**. That is testable without a real status, so it
is tested, in two rows appended to `gui/singlesig_truth_test.go`:

**`TestRestoreDocPutsTheStatusFirstAndTheInventoryLast`** — drives **both**
`restoreDocFlow` and `multisigRestoreDocFlow` under `runUI` and asserts, on page
1 of each: the `status` argument is present, it is drawn **before** the
document's own first line (`Master fp:` / `Type:`), and the `extra` argument is
**not** on page 1.

The fixture statuses are synthetic (`ZZSTATUSZZ`, `ZZINVENTORYZZ`) **on
purpose**: position is the claim, and the real status line wraps to most of page
1 on this pager, at which point "did it come first" cannot be read off a single
frame at all. The real line's placement on a real document is T11's job at step
7. This test is deliberately *below* T11 — it asserts on the two restore-doc
functions, not through `engraveSingleSigFlow` — and it does not discharge it.

**`TestRestoreDocStatusPlaceholderCannotStrengthenTheDocument`** — (a) the
placeholder equals `buildVerifyStatusLine(verifyRecord{})`, and (b) all three
production files name it in **code**.

Row (b) is expected to go **red at step 7**, when those literals become
`buildVerifyStatusLine(rec)`. That is deliberate: it forces step 7 to remove the
step-4 scaffold consciously rather than leave a stale guard passing over changed
code. Its failure message says so.

---

## 6. TDD — red, green, and four mutations

### Baseline, measured before any edit (at `112d537`, clean tree)

    go build ./...            exit 0
    go test ./... -count=1    exit 0   51 ok   0 FAIL   stderr empty

### RED — before the fix

The signature landed first with the composition in the **wrong** shape
(`append(append(lines, extra...), status)` — the trailing-parameter form the
round-1 fold specified), so the red is about the assertion and not about a
missing symbol, which §5 explicitly says proves nothing:

    === RUN   TestRestoreDocPutsTheStatusFirstAndTheInventoryLast/single-sig
        single-sig: page 1 of the restore document does not carry the status at all.
        A trailing parameter cannot reach slice index 0, and a status the reader has
        to page to is one the reader does not have
          page 1 "RestoreDocMasterfp:73c5da0aDescriptor:wpkh([73c5da0a/84h/0h/0h]xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcu"
    === RUN   TestRestoreDocPutsTheStatusFirstAndTheInventoryLast/multisig
        multisig: page 1 ... does not carry the status at all
          page 1 "RestoreDocType:P2WSH2-of-3multisig(sorted)Descriptor:wsh(sortedmulti(2,xp"
    --- FAIL: TestRestoreDocPutsTheStatusFirstAndTheInventoryLast (0.01s)

A second, genuine red followed on the fixed tree: the needle `"Master fp:"`
matched nothing, because the space glyph inks nothing and
`op.Drawer.ExtractText` never sees one. The test now normalises the way
`uiContains` does.

### GREEN — after `append(append([]string{status}, lines...), extra...)`

    --- PASS: TestRestoreDocNestedNameIsActuallyDrawn (0.01s)
    --- PASS: TestRestoreDocPutsTheStatusFirstAndTheInventoryLast (0.00s)
        --- PASS: .../single-sig (0.00s)
        --- PASS: .../multisig (0.00s)
    --- PASS: TestRestoreDocStatusPlaceholderCannotStrengthenTheDocument (0.00s)
    ok  	seedhammer.com/gui	0.024s

### The four mutations — every one failed, and the output proves the mutated line ran

**M1 — the status rides in the trailing parameter** (T11's own named mutation,
and the shape the round-1 fold specified). Both flows changed to
`append(append(lines, extra...), status)`:

    FAIL, both sub-tests.
    page 1 "RestoreDocMasterfp:73c5da0aDescriptor:wpkh(..." -- no status anywhere on it.

*Ran:* the printed page 1 is the mutated composition — the status is absent from
a page that previously opened with it.

**M2 — `extra` hoisted to index 1**, i.e. both ends present but the inventory
above the wallet (`append(append([]string{status}, extra...), lines...)`):

    FAIL, both sub-tests: "the set inventory is on PAGE 1"
    page 1 "RestoreDocZZSTATUSZZZZINVENTORYZZMasterfp:73c5da0aDescriptor:wpkh([73c5da0a/84h/0"
    page 1 "RestoreDocZZSTATUSZZZZINVENTORYZZType:P2WSH2-of-3multisig(sorted)"

*Ran:* `ZZINVENTORYZZ` is visibly rendered between the status and the document's
first line — the mutation's exact effect, printed.

**M3 — `buildVerifyStatusLine`'s fall-through returns the did-not-pass line**
instead of the zero-cell line:

    FAIL: the step-4 placeholder is not what an unrecorded verify renders, so wiring
    the record in at step 7 would silently change every document that recorded nothing
      placeholder "These plates were not fully checked. Confirm they restore this wallet (master fingerprint below) before relying on this backup."
      zero cell   "A verification check ran and did not pass: a comparison did not match, or a plate could not be read or accounted for. Do NOT rely on this backup until a full check passes. Check again with every plate this run engraved; if this repeats, engrave a fresh set."

*Ran:* the "zero cell" value printed is the mutated return, verbatim.

**M4 — one production call site's status set to `""`** (`gui/singlesig.go`):

    FAIL: gui/singlesig.go reaches a restore document but names no status placeholder.
    A restore-doc call site whose status is "" renders a blank first line, and a
    document silent about its verification is one a stranger reads as verified

*Ran:* the message names the mutated file, and only that file was mutated.

### M4 exposed a FALSE PASS in the row written to catch it

Row (b) was first written as `readGuiFile(t, file)` + `strings.Contains`, over
raw source. All three call sites carry a comment explaining the placeholder **by
name**, so gutting the code leaves the justification behind and the search still
matches. Measured, with M4 applied and the search unstripped:

    === RUN   TestRestoreDocStatusPlaceholderCannotStrengthenTheDocument
    --- PASS: TestRestoreDocStatusPlaceholderCannotStrengthenTheDocument (0.00s)
    ok  	seedhammer.com/gui	0.006s

**PASS, with the defect present.** The row now strips `//` line comments before
searching, and then M4 fails as shown above. This was found by *running* the
mutation, not by reading the test — the third time in this cycle that executing a
thing beat reading it. Over-stripping can only ever produce a spurious red, never
a false green, so the failure direction of the helper itself is safe.

---

## 7. Full-suite gate — streams separated

`nix` writes `Git tree is dirty` to **stderr**; a `2>&1` capture has corrupted
counts twice in this project, so every stream below was captured to its own file.

    $ nix develop --command go build ./...          exit 0   stdout empty
    $ nix develop --command go test ./... -count=1  exit 0   51 ok   0 FAIL
    $ nix develop --command gofmt -l gui/           exit 0   no output
    $ nix develop --command ./cmd/emu/build.sh      exit 0   built emu.wasm (9980340 bytes)
    $ nix develop --command go vet ./...            exit 1   40 findings, ALL test-only

`go vet`'s exit 1 with **40 test-only findings** is the plan's own declared clean
baseline (§6). Non-test findings: **0**, measured. The set is the shipped
`bezier.Point` unkeyed-literal noise plus the `testing.ArtifactDir requires
go1.26` group, which includes the known `gui/freetext_sizeproof_golden_test.go`
one the brief named. Nothing was skipped or weakened.

**The package/status set is byte-identical to the baseline** measured at
`112d537` before any edit:

    $ diff <(awk '{print $1, $2}' base-test.out) <(awk '{print $1, $2}' s4-test.out)
    (no output)

The emulator was built explicitly because `go test` does not compile it.

---

## 8. Contradictions with the plan, and things worth carrying forward

1. **The plan's own grep for the call sites is case-sensitive and reports 1 of
   4.** §1.8 quotes `grep -rn "restoreDocFlow(" --include="*.go" gui/` and then
   asserts *"the definition at `gui/singlesig_restore.go:119`, and
   `gui/singlesig.go:136`. Nothing else."* That is a **true statement about a
   command that cannot see `multisigRestoreDocFlow`**, presented as a blast-radius
   measurement. §1.8 is labelled as a fact about `restoreDocFlow` alone, so it is
   not wrong — but §4.2 then repeats *"Blast radius is one production call site
   and zero test call sites (§1.8)"* directly under a code sample that changes
   **both** functions. **Any later step re-running that grep to check its own work
   will get 1 and believe it.** It should be `grep -rni`, or anchored on
   `RestoreDocFlow(`. This is the same defect class as the "three"/"four" and
   "six"/"eight" counts the plan has already corrected twice: a count measured
   with the wrong instrument, then propagated.

2. **§1.8's "zero test call sites" is false as a statement about the change**,
   and the brief already knew it — `gui/multisig_nested_name_test.go:230` is a
   test call site of the *other* function. Updated, not left.

3. **Prepending a status line moves every restore document down by one wrapped
   label, and that is a real hazard for step 7, not for step 4.** With the
   synthetic short status this step's test uses, nothing shifted. With the
   **real** line — 122 characters, wrapping to several rows — page 1 of the
   restore document may hold *little but the status*. Two shipped single-frame
   assertions sit directly in that blast radius and pass **today** only because
   nothing production-side passes a long status yet:
   - `gui/multisig_nested_name_test.go:235` — `pumpUntil(frame, "P2SH-P2WSH", 64)`.
     `pumpUntil` only pumps frames, it never presses, so this is a **page-1**
     assertion. It passes at step 4 (measured: ink 13280 px, floor 6000) because
     the placeholder line still leaves `Type:` and the name on page 1.
   - `gui/multisig_supply_passphrase_test.go:243` — `pumpUntil(frame, "Descriptor:", 96)`,
     the anchor for reaching the supply-path restore document. Same shape.

   Neither needed repair here. **Both should be re-checked at step 7**, when the
   status becomes a real multi-row line, and the repair if they break is a paging
   press (`s5PageForNeedle`, `gui/multisig_build_s5_flow_test.go:119`) — never a
   weakened needle. Flagging rather than pre-fixing: at step 4 there is nothing
   to fix, and a speculative repair would have to be re-justified anyway.

4. **`§4.7f`'s scope line does not exist and is not owed by this step.** It
   renders only under `statusCheckDidNotPass`, which no flow can produce yet.
   Step 7.

---

## 9. Left for later steps — nothing pulled forward

Confirmed absent from this commit:

- **No `rec *verifyRecord` out-parameter** on `singleSigVerifyFlow` or
  `multisigVerifyFlow`; neither signature moved. (step 7)
- **No `buildVerifyStatusLine` call from any flow** — the three production sites
  name the constant. (step 7)
- **No single-sig label, inventory, census or abort wiring** (§4.1/4.5/4.6), and
  **no ninth `buildPlateInventoryLines` call site**: `gui/singlesig.go` passes
  `nil` for `extra`. (step 5)
- **No walk repairs** — the three walks §5.1b names are untouched and still
  green, because nothing yet stops them. (step 6)
- **T11, T20, T23, T24, T25, T27 not written.** (step 7)
- **No comment corrections** (§4.8b). (step 8)
- **No spec edit.** (step 9)

Nothing in the threading pushed toward a later step's work, so there was nothing
to stop and report on that axis. The one thing that *would* have — the
single-sig inventory — was already correctly scheduled to step 5 by step 3's
implementer, and `extra` accepts `nil` without it.
