# S6a STEP 7 — implementation report

**Worktree** `/scratch/code/shibboleth/wt-s6a`, branch `s6a-singlesig-truth`,
parent `1a35b663f47ef48a1ea9ac6fb03337196c76a3e2` (steps 5+6),
**committed as `bf453671b82ed066785e1bde1bc064660a25ccea`**, worktree clean, not
pushed, not merged, not rebased. Nothing outside that worktree was modified
except this file; `/scratch/code/shibboleth/seedhammer` is still `main` at
`b8a23bf3dcf45f0b996bedf8b17f7141f092d282` with 0 dirty files.

Scope: §4.8 step 7 — wire the verify status into all three flows, plus **T11,
T20, T23, T24, T25, T27**, updating all twelve call sites plus the stub (§5.1).

---

## 0. Files changed

    $ git diff --stat HEAD
     gui/multisig.go                        |   21 +-
     gui/multisig_build.go                  |   21 +-
     gui/multisig_engrave_tail_walk_test.go |    2 +-
     gui/multisig_restore.go                |    3 +-
     gui/multisig_supply_multislot_test.go  |    4 +-
     gui/multisig_verify.go                 |   72 +-
     gui/multisig_verify_flow_test.go       |   26 +-
     gui/multisig_verify_policy_test.go     |    2 +-
     gui/multisig_verify_report_test.go     |   10 +-
     gui/singlesig.go                       |   29 +-
     gui/singlesig_restore.go               |    3 +-
     gui/singlesig_truth_test.go            | 1226 ++++++++++++++++++++++++++---
     gui/singlesig_verify.go                |   56 +-
     gui/verify_status.go                   |   37 +
     14 files changed, 1376 insertions(+), 136 deletions(-)

Production code changed in seven files; the whole production diff, comments
stripped, is 40 lines. Everything else is tests.

---

## 1. THE FOUR COUNTS — measured before and after

### BEFORE (at `1a35b66`, before any edit)

**(a) Eight direct `multisigVerifyFlow(` call sites.**

    $ grep -rn 'multisigVerifyFlow(' --include='*.go' . | grep -v 'func multisigVerifyFlow'
    gui/multisig_verify_report_test.go:38:		res = multisigVerifyFlow(ctx, &descriptorTheme, ms1 != "", expected, engravedMd1)
    gui/multisig_verify_report_test.go:348:		res = multisigVerifyFlow(ctx, &descriptorTheme, full, expected, engravedMd1)
    gui/multisig_verify_report_test.go:576:		res = multisigVerifyFlow(ctx, &descriptorTheme, true /* FULL */, expected, engravedMd1)
    gui/multisig_supply_multislot_test.go:271:	frame, quit := runUI(ctx, func() { multisigVerifyFlow(ctx, &descriptorTheme, false, expected, engravedMd1) })
    gui/multisig_verify_flow_test.go:118:		multisigVerifyFlow(ctx, &descriptorTheme, false, expected, engravedMd1)
    gui/multisig_verify_flow_test.go:224:	frame, quit := runUI(ctx, func() { multisigVerifyFlow(ctx, &descriptorTheme, false, nil, md1) })
    gui/multisig_verify_flow_test.go:250:		multisigVerifyFlow(ctx, &descriptorTheme, false, []int{slot}, nil)
    gui/multisig_verify_policy_test.go:177:		multisigVerifyFlow(ctx, &descriptorTheme, false, expected, engravedMd1)
    COUNT: 8

**Exactly the eight the brief named, at exactly the lines it named. CONFIRMED.**

**(b) Four source assertions.**

    $ grep -rn 'multisigVerifyFn(ctx, th, full, engravedSlots' --include='*_test.go' .
    gui/multisig_verify_flow_test.go:373:	if !strings.Contains(body, "multisigVerifyFn(ctx, th, full, engravedSlots, engraveMd1)") {
    gui/multisig_verify_flow_test.go:394:	if !strings.Contains(body, "multisigVerifyFn(ctx, th, full, engravedSlots, suppliedMd1)") {
    gui/multisig_verify_report_test.go:1079:			"multisigVerifyFn(ctx, th, full, engravedSlots, suppliedMd1)"},
    gui/multisig_verify_report_test.go:1081:			"multisigVerifyFn(ctx, th, full, engravedSlots, engraveMd1)"},

**Four, at the named lines. CONFIRMED.**

**(c) One stub closure** — `gui/multisig_engrave_tail_walk_test.go:105`
(`multisigVerifyFn = func(...)`). CONFIRMED.

**(d) One `singleSigVerifyFlow` call site.**

    $ grep -rn 'singleSigVerifyFlow(' --include='*.go' .
    gui/singlesig_verify.go:65:func singleSigVerifyFlow(ctx *Context, th *Colors, full, template bool) {
    gui/singlesig.go:184:		singleSigVerifyFlow(ctx, th, full, template)

**One call site, at `gui/singlesig.go:184` — the brief's re-anchored line, not
the plan's `:132`. CONFIRMED. No stub, no indirection, no test callers.**

8 + 4 = **twelve**, plus the stub, plus single-sig's one. All four counts
matched the brief exactly; nothing was reported back for a decision.

### AFTER

    $ grep -rn 'multisigVerifyFlow(' --include='*.go' . | grep -v 'func multisigVerifyFlow'
    gui/multisig_verify_report_test.go:38:		res = multisigVerifyFlow(ctx, &descriptorTheme, ms1 != "", expected, engravedMd1, &verifyRecord{})
    gui/multisig_verify_report_test.go:348:		res = multisigVerifyFlow(ctx, &descriptorTheme, full, expected, engravedMd1, &verifyRecord{})
    gui/multisig_verify_report_test.go:576:		res = multisigVerifyFlow(ctx, &descriptorTheme, true /* FULL */, expected, engravedMd1, &verifyRecord{})
    gui/multisig_supply_multislot_test.go:272:		multisigVerifyFlow(ctx, &descriptorTheme, false, expected, engravedMd1, &verifyRecord{})
    gui/multisig_verify_flow_test.go:132:		multisigVerifyFlow(ctx, &descriptorTheme, false, expected, engravedMd1, rec)
    gui/multisig_verify_flow_test.go:239:		multisigVerifyFlow(ctx, &descriptorTheme, false, nil, md1, &verifyRecord{})
    gui/multisig_verify_flow_test.go:266:		multisigVerifyFlow(ctx, &descriptorTheme, false, []int{slot}, nil, &verifyRecord{})
    gui/multisig_verify_policy_test.go:177:		multisigVerifyFlow(ctx, &descriptorTheme, false, expected, engravedMd1, &verifyRecord{})
    gui/singlesig_truth_test.go:1863:		multisigVerifyFlow(ctx, &descriptorTheme, true /* FULL */, []int{idx}, md1, &rec)
    gui/singlesig_truth_test.go:2198:		multisigVerifyFlow(ctx, &descriptorTheme, false, nil, md1, &rec)
    COUNT: 10

**The count is 10, not 8, and the two extra sites are MINE** — both in
`gui/singlesig_truth_test.go`, added by step 7's own tests (T27's real-flow
fixture and the multisig record-mapping row). All eight pre-existing sites were
updated; none was deleted or weakened. `gui/multisig_verify_flow_test.go:132`
now passes the caller's `rec` because `s5DriveVerify` gained a
record-carrying variant (`s5DriveVerifyRec`) that T27 needs; the old
zero-argument entry point is preserved as a one-line wrapper, so **its own call
sites are untouched.**

    $ grep -rn 'multisigVerifyFn(ctx, th, full, engravedSlots' --include='*.go' .
    gui/multisig_verify_report_test.go:1079:			"multisigVerifyFn(ctx, th, full, engravedSlots, suppliedMd1, &rec)"},
    gui/multisig_verify_report_test.go:1081:			"multisigVerifyFn(ctx, th, full, engravedSlots, engraveMd1, &rec)"},
    gui/multisig_build.go:460:			res := multisigVerifyFn(ctx, th, full, engravedSlots, engraveMd1, &rec)
    gui/multisig_verify_flow_test.go:389:	if !strings.Contains(body, "multisigVerifyFn(ctx, th, full, engravedSlots, engraveMd1, &rec)") {
    gui/multisig_verify_flow_test.go:410:	if !strings.Contains(body, "multisigVerifyFn(ctx, th, full, engravedSlots, suppliedMd1, &rec)") {
    gui/multisig.go:345:		res := multisigVerifyFn(ctx, th, full, engravedSlots, suppliedMd1, &rec)

**All four source assertions carry the NEW call verbatim, including the closing
paren, and each still matches its production site byte for byte. None was
relaxed to a substring** — the needles are longer than before, not shorter.

    $ grep -rn 'singleSigVerifyFlow(' --include='*.go' .
    gui/singlesig.go:191:		singleSigVerifyFlow(ctx, th, full, template, &rec)
    gui/singlesig_verify.go:89:func singleSigVerifyFlow(ctx *Context, th *Colors, full, template bool, rec *verifyRecord) {
    gui/singlesig_truth_test.go:1908:		singleSigVerifyFlow(ctx, &descriptorTheme, true /* FULL */, false, &rec)
    gui/singlesig_truth_test.go:2049:		singleSigVerifyFlow(ctx, &descriptorTheme, false /* watch-only */, false, &rec)

The two extra single-sig sites are also mine: step 7's tests are **the first
executing callers `singleSigVerifyFlow` has ever had**, which is discussed in §7.

The stub closure gained the parameter:

    gui/multisig_engrave_tail_walk_test.go:105:	multisigVerifyFn = func(ctx *Context, th *Colors, full bool, expectedSlots []int,
    gui/multisig_engrave_tail_walk_test.go:106:		engravedMd1 []string, rec *verifyRecord,
    gui/multisig_engrave_tail_walk_test.go:107:	) multisigVerifyResult {

---

## 2. The new signatures

    gui/multisig_verify.go:674  func countUncoveredPolicyKeys(keys []md.ExpandedKey, covered map[int]bool) int
    gui/multisig_verify.go:697  func multisigVerifyFlow(ctx *Context, th *Colors, full bool, expectedSlots []int, engravedMd1 []string, rec *verifyRecord) multisigVerifyResult
    gui/singlesig_verify.go:89  func singleSigVerifyFlow(ctx *Context, th *Colors, full, template bool, rec *verifyRecord)
    gui/verify_status.go:189    func verifyStatusScopeLines(status string) []string

**Unchanged, deliberately:** `multisigVerifyFlow`'s verdict return, both restore-doc
flow signatures (`restoreDocFlow`, `multisigRestoreDocFlow` still take
`status string, extra []string` exactly as step 4 landed them), and
`buildVerifyStatusLine(rec verifyRecord) string`.

Both flows normalise a nil record at entry (`if rec == nil { rec = &verifyRecord{} }`).
Every caller passes one, so this is unreachable today; it is there because the
alternative failure is a **nil dereference on the device, mid-verify, on the
paths that are already the bad news.** It cannot change behaviour for a correct
caller. Flagged here as the one line in the diff nothing in the plan asked for.

---

## 3. THE ELEVEN-EXIT MAPPING AS SHIPPED — line by line against step 1's table

`gui/singlesig_verify.go` was untouched by steps 2-6, so step 1's line numbers
were still correct when I started; the writes below moved them. Both columns are
measured, not quoted:

    $ awk 'NR>=95 && NR<=203 && /return$|rec.adverse = true|rec.pass = &passRecord/{printf "%d\t%s\n", NR, $0}' gui/singlesig_verify.go
    97		return
    106		return
    118		return
    126			return
    140		return
    150		rec.adverse = true
    152		return
    160			return
    165			return
    173			return
    181		rec.adverse = true
    183		return
    198	rec.pass = &passRecord{

| # | step 1 says (`b8a23bf`) | now | class | writes | shipped |
| --- | --- | --- | --- | --- | --- |
| 1 | `:69` Back at the seed keyboard | `:97` | benign | NEITHER | bare `return` — **no write** |
| 2 | `:78` Back at the purpose/script picker | `:106` | benign | NEITHER | bare `return` — **no write** |
| 3 | `:90` re-typed seed will not derive | `:118` | benign | NEITHER | bare `return` — **no write** |
| 4 | `:98` template bundle will not build | `:126` | benign | NEITHER | bare `return` — **no write** |
| 5 | `:112` Back/Done at the gather | `:140` | benign | NEITHER | bare `return` — **no write** |
| 6 | `:117` readback not accounted for | `:150`/`:152` | **ADVERSE** | `adverseRecorded` | `rec.adverse = true` |
| 7 | `:125` Back at the ms1 keyboard | `:160` | benign | NEITHER | bare `return` — **no write** |
| 8 | `:130` typed object is not an ms1 | `:165` | benign | NEITHER | bare `return` — **no write** |
| 9 | `:138` ms1 will not decode | `:173` | benign | NEITHER | bare `return` — **no write** |
| 10 | `:146` comparator disagreed | `:181`/`:183` | **ADVERSE** | `adverseRecorded` | `rec.adverse = true` |
| 11 | `:149` **fall-through**, the only success exit | `:198` | benign (not adverse) | `fullPassRecorded` | `rec.pass = &passRecord{...}` before the closing brace, after the `showNotice` |

    $ grep -c 'rec.adverse = true' gui/singlesig_verify.go   → 2
    $ grep -c 'rec.pass = &passRecord{' gui/singlesig_verify.go → 1

**2 adverse / 8 zero-cell / 1 pass = 11.** Identical to step 1's table, row for
row. **No exit names a `verifyStatus`**; the cell is derived once, downstream, by
`verifyStatusFor`. `statusVerifiedOnRetry` remains unreachable from inside the
flow: both adverse sites are terminal `return`s and the single call site is a
one-shot `if` (`gui/singlesig.go:191`), so nothing writes adverse and then
reaches `:198`.

The pass write, verbatim as shipped:

    rec.pass = &passRecord{
        full:              full,
        legs:              1,
        suppliedCosigners: 0,
    }

### The multisig side, for symmetry

Six adverse writes, at §4.7b's six adverse sites inside the flow
(`extractReadbackMd1AndMk1s` refusal, foreign md1, md1 will not decode, plate
count mismatch, `verifyMultisigLegsPartial` mismatch, `verifyMultisigLegs`
mismatch), and one pass write at the success return. Measured:

    $ grep -c 'rec.adverse = true' gui/multisig_verify.go    → 6
    $ grep -c 'rec.pass = &passRecord{' gui/multisig_verify.go → 1

The nine benign sites (`:670`, `:680`, `:696`, `:794`, `:897`, `:938`, `:940`,
`:979` in `main` numbering, plus the loop exits) write neither boolean.

---

## 4. `suppliedCosigners` — the expression as shipped

    // gui/multisig_verify.go:674
    func countUncoveredPolicyKeys(keys []md.ExpandedKey, covered map[int]bool) int {
        n := 0
        for i := range keys {
            if !covered[i] {
                n++
            }
        }
        return n
    }

used at the success return:

    rec.pass = &passRecord{
        full:              full,
        legs:              len(legs),
        suppliedCosigners: countUncoveredPolicyKeys(keys, covered),
    }

**Step 1's direction is preserved: it iterates the KEYS and asks whether each is
covered, never `len(keys) - len(covered)`.** A stray or out-of-range entry in
`covered` can then only make the count LARGER — which renders a clause saying
*less* was checked — and can never shrink it, which would hide an unchecked key.
Single-sig writes the literal `0`, by construction rather than by omission.

---

## 5. THE RENDERED STATUS LINE, PER FLOW, PER MODE

Produced by a throwaway `TestZZDumpS7StatusLines` run against the shipped
builder and deleted afterwards (`git status` clean of it). Diff these against
§4.7c's clause table A-D.

| flow / mode | status | line |
| --- | --- | --- |
| any flow, skip or benign exit | `statusNotFullyChecked` | `These plates were not fully checked. Confirm they restore this wallet (master fingerprint below) before relying on this backup.` |
| any flow, adverse, no pass | `statusCheckDidNotPass` | `A verification check ran and did not pass: a comparison did not match, or a plate could not be read or accounted for. Do NOT rely on this backup until a full check passes. Check again with every plate this run engraved; if this repeats, engrave a fresh set.` |
| single-sig, FULL pass | `statusVerified` | `1 key plate was read back and matched what this run engraved. The ms1 secret you typed matched this seed.` |
| single-sig, WATCH-ONLY pass | `statusVerified` | `1 key plate was read back and matched what this run engraved. No secret seed share was read back or compared.` |
| single-sig, FULL pass after an adverse | `statusVerifiedOnRetry` | `1 key plate was read back and matched what this run engraved. The ms1 secret you typed matched this seed. An earlier check did not pass; a later full check passed.` |
| single-sig, WATCH-ONLY pass after an adverse | `statusVerifiedOnRetry` | `1 key plate was read back and matched what this run engraved. No secret seed share was read back or compared. An earlier check did not pass; a later full check passed.` |
| multisig supply, 1 leg / 3 supplied, FULL | `statusVerified` | `1 key plate was read back and matched what this run engraved. The ms1 secret you typed matched this seed. Other cosigners' keys are taken as supplied.` |
| multisig supply, 1 leg / 3 supplied, WATCH-ONLY | `statusVerified` | `1 key plate was read back and matched what this run engraved. No secret seed share was read back or compared. Other cosigners' keys are taken as supplied.` |
| multisig build, 2 legs / 1 supplied, FULL | `statusVerified` | `2 key plates were read back and matched what this run engraved. The ms1 secret you typed matched this seed. Other cosigners' keys are taken as supplied.` |
| multisig build, 2 legs / 1 supplied, WATCH-ONLY | `statusVerified` | `2 key plates were read back and matched what this run engraved. No secret seed share was read back or compared. Other cosigners' keys are taken as supplied.` |
| multisig build, 2 legs / 1 supplied, FULL after an adverse | `statusVerifiedOnRetry` | `2 key plates were read back and matched what this run engraved. The ms1 secret you typed matched this seed. Other cosigners' keys are taken as supplied. An earlier check did not pass; a later full check passed.` |
| multisig self-build, 3 legs / 0 supplied, FULL | `statusVerified` | `3 key plates were read back and matched what this run engraved. The ms1 secret you typed matched this seed.` |

Checked against §4.7c: **A** always (with `plate`/`plates` and `was`/`were` both
agreeing with N — see rows 1-leg vs 2-leg); **B** iff `full`; **B2** iff not
`full`; **C** iff `suppliedCosigners > 0` (present on the supply and build rows,
absent on every single-sig row and on the self-build row where the operator
holds every slot); **D** iff `statusVerifiedOnRetry`; order A, B/B2, C, D,
joined by one space. No line carries an em dash or a smart quote.

The §4.7f scoping line renders under `statusCheckDidNotPass` and nowhere else
(the dump prints `scope: []` for all eleven other rows):

    Everything below describes what this run INTENDED to engrave. Until the check above is resolved, do not assume the plates match it.

### The one design decision step 7 had to make, because §4.7f owns no mechanism

§4.7f requires the scoping line "immediately after the status line", and §4.2's
signature — landed at step 4, pinned by a shipped test — carries a single
`status string`. Nothing in the plan says who builds the scope line.

**It is derived INSIDE the two restore-doc flows** from the status string
itself, by `verifyStatusScopeLines`, rather than passed in as a new parameter:

    head := append([]string{status}, verifyStatusScopeLines(status)...)
    restoreDocScreen(ctx, th, append(append(head, lines...), extra...))

Reason: three production flows reach a restore document, and a parameter can be
omitted at one call site while the compiler stays happy — which is the exact
"wired into two of the three and forgotten on the third" shape this cycle exists
to close. A line the document builds for itself cannot be forgotten. The test is
identity against a package constant (`status != verifyStatusDidNotPassLine`), not
inference from a verdict, and the constant and the comparison move together in
any rewording. **No signature changed, so step 4's four call sites and its test
are untouched.** If a reviewer prefers an explicit parameter, that is a
mechanical change to two functions and four call sites.

---

## 6. THE TESTS

### 6.0 What "red" means here, stated plainly

§5 rules that "FAIL against the unfixed tree" is wrong for these rows: a test of
a function or arity that does not exist yet does not fail, it does not compile.
So the standard applied is §5's own — **mutate the specific behaviour the row
names, on an otherwise-complete tree, and watch that row go red** — and every
row below carries the mutation, the command, and the real failure text. The
failure messages name the mutated line's effect, which is what proves the
mutated line RAN rather than merely landing.

The only genuine pre-implementation red available was the compile break from the
arity change, and it is not evidence about any assertion.

### 6.1 Green, all rows, on the shipped tree

    $ nix develop --command go test ./gui/ -run '<the step-7 rows>' -count=1 -v
    --- PASS: TestRestoreDocStatusIsBuiltFromTheRecordOnEveryFlow (0.00s)
        singlesig_truth_test.go:1267: the verified watch-only single-sig run cut 5 plate(s)
        singlesig_truth_test.go:1573: the supply run cut 7 plate(s)
        singlesig_truth_test.go:1613: the build run cut 9 plate(s)
    --- PASS: TestEveryFlowsRestoreDocumentSaysWhatItCheckedAndWhatItHolds (132.58s)
        --- PASS: .../single-sig (32.74s)
        --- PASS: .../multisig-supply (47.30s)
        --- PASS: .../multisig-build (52.55s)
    --- PASS: TestVerifyStatusCellsRenderFourDistinctLines (0.00s)
    --- PASS: TestVerifyStatusDerivationReadsNoVerdict (0.00s)
        singlesig_truth_test.go:1986: multisig: 4 policy keys, 1 leg(s) verified, 3 taken as supplied
    --- PASS: TestVerifyPassLineNamesCosignersOnlyWhereThereAreSome (0.17s)
    --- PASS: TestSingleSigVerifyRecordsWhatItObserved (0.29s)
        --- PASS: .../readback-not-accounted-for (0.15s)
        --- PASS: .../comparator-disagreed (0.14s)
        --- PASS: .../back-before-any-plate-is-read (0.00s)
    --- PASS: TestMultisigVerifyRecordsWhatItObserved (0.07s)
        --- PASS: .../comparator-disagreed (0.05s)
        --- PASS: .../refused-before-any-plate-is-read (0.02s)
    ok  	seedhammer.com/gui	133.151s

### 6.2 Where each row lives

| row | test | walk |
| --- | --- | --- |
| T20(a) four cells, byte-exact, distinct, non-empty | `TestVerifyStatusCellsRenderFourDistinctLines` | none |
| T20(b) flow 1 of 3 + T11 scope-absent | `TestEveryFlowsRestoreDocumentSaysWhatItCheckedAndWhatItHolds/single-sig` | single-sig, REAL verify, watch-only, 5 plates |
| T20(b) flow 2 of 3 + T11 scope-present + T23 | `.../multisig-supply` | supply 2-of-2, stub, 7 plates |
| T20(b) flow 3 of 3 + T24 | `.../multisig-build` | build 2-of-3, stub, 9 plates |
| T25 | `TestVerifyStatusDerivationReadsNoVerdict` | none |
| T27 | `TestVerifyPassLineNamesCosignersOnlyWhereThereAreSome` | two REAL verify flows |
| the eleven-exit mapping (added; see §8) | `TestSingleSigVerifyRecordsWhatItObserved` | three REAL single-sig verifies |
| the multisig record wiring (added; see §8) | `TestMultisigVerifyRecordsWhatItObserved` | two REAL multisig verifies |

**T20 asserts on all three production flows, and each carries a DIFFERENT cell**
(`statusVerified`, `statusCheckDidNotPass`, `statusVerifiedOnRetry`). That is
deliberate: three arms all expecting the zero cell would pass on the pre-step-7
tree, where all three call sites passed that literal.

**T11 is asserted through production flows, on both arms**, and "slice index 0"
is observed as "the status is drawn before the first line the document itself
contributes" (`Master fp:` / `Type:`) across the document's pages in page order.

**T23 and T24 run on multisig retries only**, per §5's ruling that single-sig
has no retry loop.

### 6.3 THE CONSOLIDATION — and why it is not a weakening

The first draft gave every row its own walk: **seven walks, 435s**, and the gui
package **died at Go's default 10-minute timeout, mid-engrave, with every
assertion passing** (`panic: test timed out after 10m0s`, `FAIL
seedhammer.com/gui 600.013s`). A second draft merged to three walks (127s) and
finished at **577.6s — 96% of the budget**, which is a latent red, not a pass.

The shipped structure folds step 7's document rows into the three walks step 6
already ran (`TestSeedHandlingRulingMatchesEachPathsCapacity`, T7c, 129.23s),
renaming it `TestEveryFlowsRestoreDocumentSaysWhatItCheckedAndWhatItHolds`
because leaving the old name on it would have made the NAME the false comment.
Measured: **129.23s before, 132.58s after** — step 7's document rows cost 3.4s of
walk time, and T7c kept every one of its assertions.

**No row lost an assertion, and every row still fails against its own mutation
(§6.4).** The rename is the visible half of the trade; it is greppable in the
diff, and the new name's doc comment carries the row map.

### 6.4 MUTATION CHECKS — fourteen, every one RED

Each was applied to the shipped tree, run scoped, and reverted; the tree was
diffed against byte-exact backups after every batch (`restoration checked`, no
`DIFFERS`).

| # | mutation | row | result |
| --- | --- | --- | --- |
| M1 | `statusCheckDidNotPass` returns the zero-cell line (two cells, one string) | T20(a) | **RED** — *"statusCheckDidNotPass and statusNotFullyChecked render the SAME line, so two cells of the 2x2 are indistinguishable on the document"* |
| M2 | `buildVerifyStatusLine` returns `""` (the status as SILENCE) | T20(a)+(b) | **RED** — *"statusCheckDidNotPass renders the empty string"* and *"the single-sig restore document does not carry its status line"* |
| M3 | the status passed via the TRAILING `extra` parameter, on both doc flows | T11 | **RED** — *"the single-sig restore document draws its status AFTER \"Master fp:\", so it is not at slice index 0"* |
| M4 | `verifyStatusScopeLines` always returns nil | T11 | **RED** — *"reports a check that RAN AND DID NOT PASS and does not scope the page beneath it"* |
| M5 | `verifyStatusScopeLines` always returns the line | T11 | **RED** — *"carries the scoping line under a status that is not \"a check ran and did not pass\""* |
| M6 | the record cleared before each attempt in `gui/multisig.go` (last-wins) | T23 | **RED** — *"carries statusNotFullyChecked as well as statusCheckDidNotPass"* |
| M7 | `verifyStatusFor`'s `pass != nil` arm moved above `pass != nil && adverse` (two-state collapse) | T24 | **RED** — *"the multisig build restore document does not carry its status line"* (the retry sentence is gone) |
| M8 | `res := verifyComplete` + a `case res == verifyComplete` pass arm in `verifyStatusFor` | T25 | **RED** — *"gui/verify_status.go names verifyComplete"*, *"reads \" res \""*, plus two wrong cells |
| M9 | `suppliedCosigners` left unwritten in the multisig pass record | T27 | **RED** — *"the multisig fixture covered every one of its 4 policy keys, so suppliedCosigners is 0 and the cosigner clause renders nowhere"* |
| M10 | the single-sig pass write at the fall-through deleted | mapping | **RED** — *"the single-sig verify passed on screen and recorded no pass"* |
| M11 | both single-sig `rec.adverse = true` writes deleted | mapping | **RED** — both adverse arms |
| M12 | `rec.adverse = true` added at the BENIGN `:97` exit (Back at the seed keyboard) | mapping | **RED** — *"pressing Back at the seed keyboard recorded something ADVERSE"* + *"a benign exit does not land in the zero cell"* |
| M13 | all six multisig `rec.adverse = true` writes deleted | mapping | **RED** — comparator arm |
| M14 | `rec.adverse = true` added at the BENIGN empty-expectation refusal | mapping | **RED** — *"a refusal taken BEFORE the gather recorded something adverse"* |

M12 and M14 are the pair the plan's R16 I-1 exists for: they are the mutations
that make a benign exit CLAIM a check ran, and both are caught. Without them the
adverse rows would be satisfiable by "set adverse everywhere".

### 6.5 The two false-PASS traps the brief named

- **Step 4's raw-source trap.** Every source assertion in this step
  (`TestVerifyStatusDerivationReadsNoVerdict`,
  `TestRestoreDocStatusIsBuiltFromTheRecordOnEveryFlow`) reads through
  `s6aCodeOf`, which **strips comments**. This mattered: `gui/verify_status.go`'s
  own comment says the derivation names no verdict "deliberately not even in this
  comment", and M8 was caught by both the code search AND the driven cells.
- **Step 5's truncation trap.** Every document assertion runs on
  `p.display = sh2DisplaySize`, and the negative assertions
  (`s6aAssertOneStatusLine`'s "no other cell", T11's "no scope line", T27's "no
  cosigner clause") are all paired with a positive assertion of the same string
  somewhere else in the step, so a needle that could never appear would show up
  as a failing positive.

---

## 7. T27 NON-VACUITY — the fixture, named and measured

T27's multisig half is driven by **`s6aMultisigFullOneSlotVerify`**, built on the
shipped **`s5TraceBEngraved(t, true)`** fixture: a run that engraved ONE key
plate (`expected = {idx}`, master A's own slot) of a policy the readback expands
to **four keys**. The flow covers one slot, so three policy keys are never
covered by a verified leg.

Measured, logged by the test itself so a reader of the output does not have to
take the comment's word for it:

    singlesig_truth_test.go:1986: multisig: 4 policy keys, 1 leg(s) verified, 3 taken as supplied
      "1 key plate was read back and matched what this run engraved. The ms1 secret you typed matched this seed. Other cosigners' keys are taken as supplied."
    singlesig_truth_test.go:1988: single-sig: 1 leg verified, 0 taken as supplied
      "1 key plate was read back and matched what this run engraved. The ms1 secret you typed matched this seed."

The test **asserts** the non-vacuity rather than assuming it: `suppliedCosigners
<= 0` is a `t.Fatalf` naming the self-multisig degeneracy the plan warns about,
and it is exactly what M9 trips.

**Both halves are FULL mode and both record `{full: true, legs: 1}`** — which is
R12's C-1 example verbatim: two runs whose records are identical except for the
path axis, whose truthful lines differ. The final assertion is the equation
`multisigLine == singleSigLine + " " + cosignerClause`, so if the path axis were
dropped the two would be identical and the row fails on the difference itself.

**Both records come from REAL flows**, not stubs: `multisigVerifyFlow` driven
through gather → seed → passphrase → typed ms1 → comparator → `Verify OK`, and
`singleSigVerifyFlow` driven through the same shape. No stub can tell you whether
`countUncoveredPolicyKeys` is right, which is why this row refuses one.

---

## 8. Two tests the plan does not list, and why they are here

`TestSingleSigVerifyRecordsWhatItObserved` and
`TestMultisigVerifyRecordsWhatItObserved` are not rows of §5.

**Without them, eight of the nine record writes in this step are dead under
mutation.** Every other multisig assertion drives the record through the
`multisigVerifyFn` stub, and every other single-sig assertion observes only the
success exit — so deleting all six multisig `rec.adverse = true` writes, or both
single-sig ones, left the entire suite green when I tried it (M11, M13). A line
no mutation reaches is a line nothing tests, and these are the lines that decide
whether a failed verify's document says *a check ran and did not pass* or the
weaker *not fully checked*.

Each is cheap (0.29s and 0.07s), drives the REAL flows, and covers all three
classes: an adverse readback-accounting exit, an adverse comparator exit, and a
BENIGN exit that must write neither bit. They cost 0.4s of the package budget.

---

## 9. VALIDATION GATE — full suite, streams separated

    $ nix develop --command go build ./...                     → BUILD=0
    $ nix develop --command gofmt -l .                          → GOFMT=0, 0 bytes of stdout
    $ nix develop --command ./cmd/emu/build.sh                  → EMU=0
                                                                  built emu.wasm (9991127 bytes)
    $ nix develop --command go test ./... -count=1 > out 2> err → TEST_EXIT=0
      stderr (61 bytes): warning: Git tree '/scratch/code/shibboleth/wt-s6a' is dirty
      non-ok lines in stdout: (none)
      ok  	seedhammer.com/gui	507.037s

    $ (run again, unchanged tree)                              → TEST_EXIT=0
      stderr (61 bytes): the same nix warning
      non-ok lines in stdout: (none)
      ok  	seedhammer.com/gui	447.143s

**Green twice, exit 0 both times, and stderr carries only nix's dirty-tree
warning.** `go vet ./...` still fails only on the known, pre-existing
`gui/freetext_sizeproof_golden_test.go:111: testing.ArtifactDir requires go1.26
or later (file is go1.25)`; left alone as instructed.

**Baseline for comparison, taken on this same worktree before any edit:**

    ok  	seedhammer.com/gui	401.344s     TEST_EXIT=0, stderr 0 bytes

### THE TIMEOUT IS THE BINDING CONSTRAINT AND IT IS NOW A REPORTABLE RISK

`go test` applies a **default 10-minute timeout per package binary**, and CI runs
`CGO_ENABLED=0 go test ./...` with no `-timeout` override
(`.github/workflows/test.yml:44`). The gui package is a single binary. So the
package has a hard 600s ceiling, and:

- scoped measurement says step 7 added **~4s** of walk time (T7c 129.23s →
  132.58s) plus ~0.6s of unit rows;
- whole-suite measurement of the SAME unchanged tree, twice, says **507.0s** and
  **447.1s**, against a **401.3s** pre-step baseline. The 60s spread between two
  identical runs is larger than the change itself, so the honest statement is
  that this machine's run-to-run variance is ±30s and step 7's contribution is
  inside the noise. (The intermediate three-walk draft measured 577.6s, also
  ~50s above its scoped prediction, which is the same variance.)

Either way the package now sits at **75%-85% of a hard ceiling** that a slower
CI runner will reach first, and it sat at 67% before. **This is a finding for the
controller, not a blocker for this step:** the gate is green on two independent
runs, but the next test anybody adds to `gui` should measure the package total
first, and steps 8 and 9 should treat 600s as the budget rather than a distant
limit. The first draft of this step's tests proved it is reachable.

---

## 10. Anything contradicting the plan

1. **§4.7f owns no mechanism for the scoping line.** §4.2 fixes the doc-flow
   signature at `status string, extra []string` (landed and test-pinned at step
   4), §4.7c is "the sole authority for what the builder prints" and the builder
   returns exactly one line, and §4.7f then requires a second line "immediately
   after the status line". Nothing says who builds it. Resolved as described in
   §5 above (derived inside the two doc flows, zero signature churn); flagged
   because it is the one place step 7 had to choose.

2. **`scripts/verify-returnsite-sweep.sh` carries a note that this step makes
   false.** Its declared blind spot says single-sig's exits "become visible and
   the count must jump" once `singleSigVerifyFlow` "gains a verdict type
   (build-order step 1)". §4.7b-seam explicitly gives it an **out-parameter and
   no verdict**, so the sweep will keep reporting 15/0 for ever and its stated
   expectation can never be met. It scans the fork, not this worktree, so nothing
   broke; it belongs on step 8's false-comment sweep. (The script lives in the
   design repo, which I did not modify.)

3. **The plan's line citations for `gui/singlesig.go` are stale**, as the brief
   said: `:132`/`:131`/`:136` are `:184`/`:183`/`:212` on this branch, and after
   step 7 they are `:191`/`:190`/`:219`. `gui/singlesig_verify.go`'s eleven exits
   were exactly where step 1 said, and moved only by my own writes (§3).

4. **§4.8 step 7's call-site count is right and incomplete in one direction.**
   All twelve sites plus the stub were the ones to update, and they were; the
   grep now returns ten `multisigVerifyFlow(` sites because step 7's own tests
   add two. Recorded so a later reader diffing counts does not read it as drift.

---

## 11. Things I thought of and did NOT build (NG1)

Each of these would report something truer about the device's epistemic state,
and each is out of scope by default under §0.1's guard. Filed, not folded.

1. **A descriptor-plate clause.** The read-back md1 IS compared on both flows,
   and the pass line does not say so. §4.7c kills this explicitly; I did not
   resurrect it, and `passRecord` still carries no field that would back it.
2. **Distinguishing the two adverse single-sig exits on the document.** `:150`
   (could not account for the plates) and `:181` (the comparator disagreed) are
   different facts and both render `statusCheckDidNotPass`. That is §4.7e's
   deliberate loss and NG2's "diagnosis has a reader" — the screens already tell
   the operator which one happened, at the machine, at verify time.
3. **Recording WHICH attempt failed on a retry.** `statusVerifiedOnRetry` says an
   earlier check did not pass; the record could carry a count. NG1.
4. **A `suppliedCosigners` clause naming the slots.** The count is known and the
   slot indices are in scope at the success return. It would put policy topology
   on the document. NG1, and arguably NG2.
5. **Reporting `template` on the pass line.** `singleSigVerifyFlow` takes
   `template` and the record does not carry it; a template engrave's pass line is
   identical to a full-policy one's. Step 1 ruled `legs: 1` correct in both forms
   and the plan carries no template clause. NG1.
6. **Making the doc flows take the record instead of the built string.** It would
   make "the status is built from the record" unforgeable rather than asserted by
   a source search. It changes a step-4 signature and breaks a shipped test, for
   a property M2/M3 already catch behaviourally.
7. **A nil-record panic instead of the nil normalisation.** A panic would be
   louder about a caller that forgot the record; it would also crash the device
   mid-verify. I chose the quiet normalisation and am flagging it (§2) rather
   than deciding it silently.

---

## 12. What a reviewer should check first

1. **§5's clause table against §5 of this report** — the twelve rendered lines
   are pasted verbatim; every one is a byte comparison, not a description.
2. **The eleven-exit table in §3** — three columns are measured output, and the
   classification is step 1's, unchanged.
3. **The scope-line mechanism (§5, item 1 of §10)** — the one decision the plan
   did not make.
4. **The consolidation in §6.3** — whether folding step 7's document rows into
   T7c's walks is an acceptable trade for staying inside the package timeout, and
   whether the rename is the right way to say so.
