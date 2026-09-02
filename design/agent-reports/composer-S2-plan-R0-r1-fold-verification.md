# Composer S2 plan — R0 round 1, FOLD-VERIFICATION lens (mechanical)

**Artifact:** `design/IMPLEMENTATION_PLAN_composer_S2_fork_codec.md` (master `e2dc7e4`)
**Fold under review:** `e2dc7e4c8da670603db0f6eacfdc04fa668b874c` (`git diff c256391..e2dc7e4`)
**Inputs:** `composer-S2-plan-R0-r0-fidelity.md` (1C/3I/5M/2N), `composer-S2-plan-R0-r0-tests.md` (2C/1I/2M/1N)
**Reviewer:** independent (sonnet); did not author the fold. Scope: did the fold fix each R0 finding exactly, and did it introduce a new defect. Not a fresh audit.
**Method:** for every finding with an executable mutation, copied the target file to a `/tmp` backup, applied the exact mutation, ran the named test(s), recorded the result, then restored from backup and re-ran the full 3-package suite to confirm green before the next mutation. Toolchain: `/nix/store/i77g9dmcd399rmxk8688qfr4g2wzgk37-go-1.26.7/bin/go`, `CGO_ENABLED=0 GOFLAGS=-mod=mod GOPROXY=off GOTOOLCHAIN=local`. Working copy: `/scratch/code/shibboleth/.s2-r1-lens/seedhammer` (a mutable copy of the read-only gate scratch `/scratch/code/shibboleth/.plan-build-gate-go/seedhammer`). Baseline before any mutation: `go vet ./md/ ./mk/ ./sysw/` clean, `go test -count=1 ./md/ ./mk/ ./sysw/` → `ok` x3 (already machine-checked by the controller after the fold; reconfirmed here).

---

## Fidelity report findings

### C-1 — `Branch` cannot distinguish `older` from `after` — VERIFIED

Fold adds `Branch.Locks []Lock` (typed, via new `lockFromWire(tag, uint32) Lock` in `md/compose.go:142-153`) and the dedicated test `TestPolicyShapeDistinguishesOlderFromAfterAtTheSameOperand`.

- Mutation A — `lockFromWire` always returns `Lock{Kind: LockAfterHeight, Value: operand}`:
  ```
  --- FAIL: TestPolicyShapeSplitsAlternativesIntoBranches
  --- FAIL: TestPolicyShapeSplitsTheShippedOrCards
  --- FAIL: TestPolicyShapeDistinguishesOlderFromAfterAtTheSameOperand
  --- FAIL: TestPolicyShapeWalksAnEightPathChain
  --- FAIL: TestPolicyShapeTapLeavesCarryLocks
  --- PASS: TestPolicyShapeReportsAKeylessAlternativeHonestly   (its vector is already after-kind; correctly unaffected)
  --- FAIL: TestPolicyShapeSplitsAndOr
  ```
- Mutation B — only the bit-22 branch collapsed (`tagOlder` always returns `LockOlderBlocks`, dropping the `sequenceTypeFlag` check):
  ```
  --- PASS: .../keyed_compose_wsh_two_path_or_d
  --- FAIL: .../keyed_compose_wsh_single_head_or_i     <- exactly the row named in the brief
  --- PASS: .../keyed_compose_wsh_three_paths
  --- PASS: .../keyed_compose_wsh_hash_and_time
  --- PASS: .../keyed_compose_wsh_locked_head_or_i
  ```
Both mutations restored; `diff -q` against the read-only gate copy confirms byte-identical. **VERIFIED exactly as the brief specified** — the general mutation is caught broadly and the bit-22-specific mutation is caught precisely by the `single_head_or_i` subtest and nothing else.

### I-1 — `Branch` cannot distinguish `sortedmulti` from `multi` — VERIFIED

Fold adds `Branch.Sorted bool`, threading it through `branchOf`/`plainMulti` (now 4-return-value), and `TestPolicyShapeCarriesSortedForThresholds`.

Mutation: `plainMulti`'s sorted arm hardcoded to `true` (`return int(b.k), len(b.indices), true, true`):
```
--- FAIL: TestPolicyShapeCarriesSortedForThresholds
```
Restored, byte-identical. **VERIFIED.**

### I-2 — Task 6 fixture location/precondition — VERIFIED

- `**PRECONDITION for Task 6**` now stated in Baselines (plan line 13) AND restated at Task 6 Step 1 (line 2518) with a runnable check (`git -C .../mnemonic-engrave log -1 --format=%H -- crates/me-cli/testdata/record_class_vectors.json`).
- `a894e619` (the wrong/stale sha): **0 hits** in the plan (`grep -c`).
- `eed6b177...` present at every pin site: Consumes clause (2511), cp-Expected (2524), provenance JSON template (2545), commit message (3060) — 4/4.
- `45` (row count) present at every count site: Consumes clause (2511), cp-Expected (2524), provenance JSON (2546), `loadRecordClassRows` guard (2610), Step 4 heading (3044) — 5/5.
- Cross-checked against the real fixture in the scratch: `sha256sum sysw/testdata/record_class_vectors.json` → `eed6b177d1a3406a69c4a0102635f5d59c6412fa65e106f85b831c4736ac464e`; `python3 -c "len(json.load(...))"` → `45`. Matches the plan's pin exactly (not just internally consistent — externally correct). **VERIFIED.**

### I-3 — shipped `or_*` cards pinned — VERIFIED

`TestPolicyShapeSplitsTheShippedOrCards` (new) pins `keyed_wsh_or_b`, `keyed_wsh_or_d_degrading`, `keyed_wsh_timelock_hashlock`. Read all three `.template`/`.descriptor.json` pairs in the fork (`/scratch/code/shibboleth/seedhammer/md/testdata/vectors/`) and traced each policy tree by hand against the plan's pinned `[]Branch` expectations:
- `keyed_wsh_or_b` = `or_b(pk(@0), s:pk(@1))` → `[{Keys:1},{Keys:1}]` ✓.
- `keyed_wsh_or_d_degrading` = `or_d(multi(2,@0,@1), and_v(v:older(65535),pk(@2)))` → `[{K:2,N:2,Keys:2},{Keys:1,Timelock:true,Locks:older-blocks(65535)}]` ✓ (65535 has bit 22 clear → blocks, matches `LockOlderBlocks`).
- `keyed_wsh_timelock_hashlock` = `or_i(and_v(v:after(1e6),and_v(v:sha256(H),multi(2,@0,@1,@2))), and_v(v:older(65535),multi(1,@1,@2)))` → both multis sit under `and_v`/`v:` wrappers (not bare), so K/N correctly stay zero per the pre-existing "never claims a plain threshold it cannot see" rule; `Keys:3`/`Keys:2`, lock kinds `LockAfterHeight`(1e6 < 500M)/`LockOlderBlocks`(65535) ✓.
- Ran the test directly: `go test -run TestPolicyShapeSplitsTheShippedOrCards -v ./md/` → PASS, confirming the pinned values match the code's real output, not just hand-arithmetic.
**VERIFIED**, expectations correct and traceable to the vendored templates.

### M-1 — vector-side measurement cited — VERIFIED

Task 4's Produces text now cites `grep -l 'or_i\|or_d\|or_b\|or_c\|andor' md/testdata/vectors/*.template`. Reproduced against the fork: returns exactly `keyed_wsh_or_d_degrading.template`, `keyed_wsh_timelock_hashlock.template`, `keyed_wsh_or_b.template` (3 files, matching the plan's claim), and `grep -n "keyed_wsh_or_b\|keyed_wsh_or_d_degrading\|keyed_wsh_timelock_hashlock" md/policy_shape_test.go` → no match, confirming none of the three feeds the four pre-existing tests. **VERIFIED.**

### M-2 (fidelity) — change chain added, mutation count 20 — VERIFIED

`TestPkhWitnessScriptsReproduceRustsAddresses` now loops `chain := 0..1` (both receive and change) inside the existing `i := 0..1` loop, keyed off `rec.Chains[strconv.Itoa(int(chain))]`. Mutation: pk_h arm's `opEQUALVERIFY, opCHECKSIG` → `opEQUAL, opCHECKSIG` (`md/script_emit.go:197`):
```
go test -run TestPkhWitnessScriptsReproduceRustsAddresses ./md/ 2>&1 | grep -c 'rust:' → 20
```
Matches the plan's updated Step 5 text and commit-message text ("twenty" / `20`) exactly, and matches the fold commit's own claim ("5 pkh vectors x 2 chains x 2 indices = 20 addresses"). Restored, byte-identical. **VERIFIED.**

### M-3 — `Chunks()` → `ExpandWalletPolicyChunks` route named — VERIFIED

Task 2 Produces now reads: "for the §7c ... read the RESOLVED per-slot origins from the emitted chunks -- `Chunks()` → `ExpandWalletPolicyChunks(chunks)` (`md/expand.go:102`)". Confirmed in the fork: `md/expand.go:102` is exactly `func ExpandWalletPolicyChunks(strs []string) (Template, []ExpandedKey, error) {`. **VERIFIED.**

### M-4 — tag-coverage mirror tightened — VERIFIED

`TestComposeFamilyTagsAreCoveredTwice` (`md/compose_test.go:226-246`) now special-cases only `"spine:0"` (no-corpus exemption dropped) and asserts it appears **exactly once**. Mutation: duplicated `"spine:0"` into a second row's tag list (`keyed_compose_tr_unsorted_sole_leaf`):
```
--- FAIL: TestComposeFamilyTagsAreCoveredTwice
```
Restored, byte-identical. **VERIFIED** — the mirror now catches a duplicated singular tag, which it could not before.

### M-5 — presets filed as follow-up — VERIFIED

Plan's "What this stage does NOT do" section (line ~3183) now opens with the preset gap, naming the fix direction (Rust `--preset` + vector, then vendor) and cites "fidelity M-5". `design/FOLLOWUPS.md` contains `F-453 — composer-preset-vectors-missing`, owning phase "composer S3 — Rust first...", matching the plan's text. **VERIFIED**, filed as instructed rather than folded into a task (as the dispatch brief expected).

### N-1 — 1-based path refusals — VERIFIED

All four wrap sites in `ValidatePathList` (`md/compose.go:311,316,318,322`) now use `i+1` for `ErrComposeBadThreshold`, `ErrComposeLockOnlyPath`, `ErrComposeKeylessUnderTr`, `ErrComposeLockOutOfRange`. Grep-confirmed all four; no `i` (0-based) remnants. **VERIFIED.**

### N-2 — aliasing doc line — VERIFIED

`Composed`'s doc comment (`md/compose.go:215-217`) now states "A copy of a `Composed` shares the underlying descriptor: `Bind` on one keys them both (it is not copy-on-write)." **VERIFIED.**

---

## Tests report findings

### C-1 — pin test now scans the directory — VERIFIED

Added `isComposeVectorFile` + a `os.ReadDir("testdata/vectors")` pass in `TestComposeVectorsMatchTheirProvenancePin`. Reproduced the brief's exact scenario: copied in `md/testdata/vectors/compose_stray_extra.bytes.hex` (content `deadbeef`, referenced nowhere):
```
=== RUN   TestComposeVectorsMatchTheirProvenancePin
    compose_vectors_pin_test.go:118: compose_stray_extra.bytes.hex: in testdata/vectors but not in the provenance pin -- re-run scripts/vendor-compose-vectors.sh or remove it
--- FAIL: TestComposeVectorsMatchTheirProvenancePin
```
File removed after; `diff -rq` against the read-only gate's `md/testdata` tree confirms clean. **VERIFIED**, names the stray file exactly as the fix direction asked.

### C-2 — `AppendStubs` aliasing probe — VERIFIED

New `TestAppendStubsDoesNotShareTheInputsBackingArray` gives the input spare capacity and writes a sentinel through it after the call. Mutation: dropped the defensive copy (`out.Stubs = card.Stubs` instead of `make`+`append`, `mk/compose_stubs.go:10-12`):
```
=== RUN   TestAppendStubsDoesNotShareTheInputsBackingArray
    compose_stubs_test.go:57: appending to the input changed the result to [deadbeef ffffffff]: AppendStubs aliased the input's array
--- FAIL: TestAppendStubsDoesNotShareTheInputsBackingArray
```
Restored, byte-identical. **VERIFIED** — this is exactly the assertion the C-2 fix hypothesis called for (a genuine shared-backing-array probe, not a length/content check), and it fires with the exact named message.

### I-1 — `now:` digit-count unit test + fixture rows — VERIFIED

New `TestNowRecordDigitCountIsBoundedIndependentlyOfRange` plus fixture rows `now-seconds-eleven-digits` / `now-height-ten-digits` (confirmed present in the 45-row fixture). Mutation: `digitsInRange`'s length check `len(s) > maxDigits` → `len(s) > maxDigits+1` (`sysw/composer_records.go:117`):
```
=== RUN   TestNowRecordDigitCountIsBoundedIndependentlyOfRange
    composer_records_test.go:181: "01756684800" accepted: the digit count is not bounded
    composer_records_test.go:181: "00000000001" accepted: the digit count is not bounded
    composer_records_test.go:181: "1756684800,0499999999" accepted: the digit count is not bounded
    composer_records_test.go:181: "1756684800,0000000001" accepted: the digit count is not bounded
--- FAIL: TestNowRecordDigitCountIsBoundedIndependentlyOfRange
```
AND, under the same mutation, the two new fixture rows fail inside `TestComposerRecordsClassifyExactlyAsTheHost`:
```
composer_records_test.go:74: now-seconds-eleven-digits: Classify("now:...") = 12, want 0 (host's answer)
composer_records_test.go:74: now-height-ten-digits: Classify("now:...") = 12, want 0 (host's answer)
--- FAIL: TestComposerRecordsClassifyExactlyAsTheHost
```
Restored, byte-identical. **VERIFIED** — both the unit leg and the lockstep-fixture leg catch the mutation independently, as the fold commit message claims.

### M-1 (tests) — `bip32.Path.String()` citation — VERIFIED

Plan cites `bip32/bip32.go:20-35` (both at the Task 6 test comment, line 2655, and the Step 4 Expected text, line 3047). Confirmed: `func (p Path) String() string {` starts at line 20 and the closing brace is at line 35 in the fork's `bip32/bip32.go`. The stale `:103-110` citation is gone (0 hits). **VERIFIED.**

### M-2 (tests) — `gui-shard-test.sh` path — VERIFIED

Plan now reads "the sharded runner lives in mnemonic-engrave, not the fork: `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24` from the fork root". Confirmed the file exists at that absolute path (`-rwxr-xr-x`). **VERIFIED.**

---

## New-defect sweep

**Hunk-to-finding mapping** (`git diff c256391..e2dc7e4`, 717-line diff, ~46 discrete hunks): every hunk attributes to exactly one of the findings above — Baselines precondition line → I-2; `composeVectorNames` comment + `isComposeVectorFile` + directory scan → tests C-1; Task 2 Produces route sentence → M-3; `TestComposeFamilyTagsAreCoveredTwice` → M-4; `ErrCompose*` comment + `ValidatePathList` four `i+1` sites → N-1; `lockFromWire` + its doc comment → C-1; `Composed` doc comment → N-2; `strconv` import + chain-loop body + Step 4/5 text + commit message ("twenty") → M-2 (fidelity); Task 4 Files/Interfaces/Why text, `lk()` helper, all seven `TestPolicyShape*` test-body edits (`Locks`/`Sorted` fields), the two new tests (`...SplitsTheShippedOrCards`, `...DistinguishesOlderFromAfter...`, `...CarriesSortedForThresholds`), Step 2/3/4 text, commit message → C-1/I-1/I-3 jointly (the Task 4 fragment is one coherent change serving all three); `AppendStubs` test edits (weak assertion removed, new aliasing test added) + Step 4 text → tests C-2; Task 6 precondition/fixture/provenance/commit-message edits → I-2; `TestNowRecordDigitCountIsBoundedIndependentlyOfRange` + its adjoining comment → tests I-1; `bip32.Path.String` citation comment + Step 4 text → tests M-1; Task 9 gui-shard path → tests M-2; "What this stage does NOT do" preset paragraph → M-5; Type-consistency line → self-review update reflecting C-1/I-1.

One extra, in-scope addition not explicitly requested by any single finding: `TestLockCheckIsTheDeviceSideRangeGate` gained a round-trip loop (every legal `Lock` survives `operand()` → `lockFromWire()` unchanged). This is additional verification of the new `lockFromWire` function that C-1's fix introduced — it strengthens rather than contradicts anything, and does not constitute an unrelated or unrequested change. Not a defect.

**No hunk found that is unrelated to an R0 finding.**

**Fragment-vs-scratch check:** diffed the pristine fork's `md/policy_shape.go` (no `compose.go` exists yet in the pristine fork — Task 2 introduces it) against the scratch's hand-wired copy. The delta is exactly: `Branch.Locks/Sha256Digests/Sorted` fields, `plainMulti`'s 3rd return value, `branchOf`'s `br.Sorted` assignment, and the `collect` `tagAfter/tagOlder`/`tagSha256` arms writing `Locks`/`Sha256Digests`. Cross-checked this against the plan's Task 4 Step 3 fragment text (lines 2118-2246) verbatim: the `plainMulti` signature/return-value prose, the three `branchOf`-call-site replacements, and the `collect` arm replacement all match the scratch's actual compiled code character-for-character where quoted. (`splitBranches`/the or_*-splitting mechanism itself pre-dates this fold — introduced by the original Task 4 draft `b95df91`, not touched by `e2dc7e4` — confirmed by the fold diff not touching those lines.)

**Type consistency line:** updated to `Branch.Locks []Lock` / `Sha256Digests [][32]byte` / `Sorted bool` in Task 4 test and code (`Lock`, `LockKind`, `lockFromWire` from Task 2) — accurate: `lockFromWire` is indeed defined in `md/compose.go` (Task 2's file, confirmed at line 142), not `policy_shape.go`.

---

## Final state check

- All 6 mutated files (`md/compose.go`, `md/policy_shape.go`, `md/compose_test.go`, `md/script_emit.go`, `mk/compose_stubs.go`, `sysw/composer_records.go`) diffed byte-identical against the read-only gate copy after every restore.
- `md/testdata` tree diffed byte-identical (stray file removed cleanly).
- Final `gofmt -l md/ mk/ sysw/`: no output (clean). `go vet ./md/ ./mk/ ./sysw/`: clean. `go test -count=1 ./md/ ./mk/ ./sysw/`: `ok` x3.
- No plan text, worktree, or read-only scratch was modified by this review; only the disposable `/scratch/code/shibboleth/.s2-r1-lens/` copy and its `/tmp/*.bak` backups were touched.

## Closing counts

**14/14 findings VERIFIED** (10 fidelity: C-1, I-1, I-2, I-3, M-1, M-2, M-3, M-4, M-5, N-1, N-2 — 11 actually, see below — plus 5 tests: C-1, C-2, I-1, M-1, M-2). Exact tally: fidelity 1C + 3I + 5M + 2N = 11 findings, all VERIFIED; tests 2C + 1I + 2M = 5 findings, all VERIFIED (tests N-1 was already recorded as non-blocking/no-action-needed in R0 and required no fold). **0 findings folded incorrectly. 0 new defects found in the fold.** The fold is a faithful, complete response to both R0 reports.
