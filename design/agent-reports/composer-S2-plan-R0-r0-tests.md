# Composer S2 plan — R0 round 0, TESTS lens

Reviewer: independent (sonnet), mutation-testing against a mutable copy of the
plan-build-gate scratch tree
(`/scratch/code/shibboleth/.plan-build-gate-go/seedhammer`, read-only source,
copied to `/scratch/code/shibboleth/.s2-tests-lens/seedhammer` for mutation).
Toolchain: `/nix/store/i77g9dmcd399rmxk8688qfr4g2wzgk37-go-1.26.7/bin/go`,
`CGO_ENABLED=0 GOFLAGS=-mod=mod GOPROXY=off GOTOOLCHAIN=local`. Baseline
confirmed green before and after every mutation round: `go test -count=1
./md/ ./mk/ ./sysw/` → `ok` ×3.

Scope: **can every named test actually FAIL when the code it guards is
wrong, and is every "Expected:" line specific and true.** Design/spec
correctness is out of scope (per brief) and was not re-audited.

## Method

For each named mutation: copy the target file to a backup, apply the exact
mutation, run the named test(s), record PASS/FAIL, restore from backup,
verify the whole three-package suite is green again before the next
mutation. All restores verified byte-identical to the original
(`diff -q` against the untouched scratch backups) at the end of the run.

## Mutation table

### 1. `md/compose.go`

| # | Mutation | Plan lines | Test that failed |
| - | - | - | - |
| 1a | `wshChain`: swap `tagOrD`/`tagOrI` choice | 1273–1290 | `TestComposeReproducesEveryVectorByteForByte` (12+ sub-tests: every `or_d`/`or_i` vector's tree diverged) |
| 1b | `numberSlots`: drop the `first`-goes-first ordering, always listed order | 1292–1320 | `TestComposeNumbersSlotsByFirstAppearance` (exact expected order); also `TestComposeReproducesEveryVectorByteForByte/keyed_compose_tr_extracted_later_four_paths` and `.../keyed_compose_tr_three_paths_extracted_later` |
| 1c | `resolveOrigins`: unseated accounts start at 1, not 0 | 1356–1422 (loop at ~1378) | `TestComposeReproducesEveryVectorByteForByte` (9 sub-tests, every default-origin vector) and `TestComposeWithFillsTheLowestFreeAccount` |
| 1d | `isBareMulti`: accept `N>=1` instead of `N>=2` | 1026–1028 | `TestComposeRefusesWhatThePrimaryRefuses` ("sh single key" case: `err = <nil>, want ErrComposeLegacyWrapperShape`) and 4 `TestComposeReproducesEveryVectorByteForByte` sub-tests |
| 1e | `keyLeaf`: flip `sortedLegal` (`if sortedLegal && ks.Sorted` → `if !sortedLegal && ks.Sorted`) | 1226–1237 | `TestComposeReproducesEveryVectorByteForByte` (17 sub-tests: every sole-bare-multi and sole-tap-leaf vector) |
| 1f | `Lock.operand`, `LockOlderBlocks` arm: `> 0xffff` → `>= 0xffff` | 976–1001 (line ~980) | `TestLockCheckIsTheDeviceSideRangeGate` (65535 boundary now rejected) and `TestComposeReproducesEveryVectorByteForByte/keyed_compose_tr_extracted_first` |
| 1g | `Bind`: skip the fingerprints-map branch entirely | 1100–1140 | `TestComposeReproducesEveryVectorByteForByte` (21 of 22 keyed sub-tests — every keyed vector whose descriptor.json carries a fingerprint TLV) |

All seven of §1's prescribed mutations are caught, several by more than one
independent test. No finding here.

### 2. `md/script_emit.go`, `pk_h` arm (lines ~1741–1763)

| # | Mutation | Test that failed |
| - | - | - |
| 2a | `opEQUALVERIFY` → `opEQUAL` | `TestPkhWitnessScriptsReproduceRustsAddresses` (all 5 sub-tests) and `TestPkhTapLeafEmitsTheHash160Form`. `TestPkhScriptDependsOnTheKey` stays PASS (by design — Step 5's own comment says this test is for the hash-of-key claim, not the opcode). Hand-mutation count reproduced: **10** `rust:` lines (matches plan Step 5's claimed count exactly — see "Task 3 Step 5" below). |
| 2b | Push the key itself instead of its `Hash160` | `TestPkhWitnessScriptsReproduceRustsAddresses` and `TestPkhTapLeafEmitsTheHash160Form` FAIL; `TestPkhScriptDependsOnTheKey` still PASSES (same reason as 2a — it only checks address-changes-with-key and length-invariance, both still true when the push is a fixed-size key instead of a fixed-size hash) |
| 2c | In tap context (`e.tap`), push `k[:20]` instead of `Hash160(k)` | `TestPkhTapLeafEmitsTheHash160Form` FAILS; `TestPkhWitnessScriptsReproduceRustsAddresses` and `TestPkhScriptDependsOnTheKey` stay PASS (correctly — they only exercise the non-tap arm, `e.tap == false`) |

All three caught. **N-1 (Nit, not blocking):** `TestPkhTapLeafEmitsTheHash160Form`
builds its expectation with `btcaddr.Hash160(xonly)` — literally the same
library call the arm under test invokes — so it is self-consistent rather
than independently oracled (unlike `TestPkhWitnessScriptsReproduceRustsAddresses`,
which is checked against Rust-derived addresses with no shared code path).
This *does* matter in principle (a test using the SUT's own primitive to
build its own expectation can miss a bug that infects both sides identically),
but empirically it is not vacuous here: mutations 2a, 2b and 2c all changed
the tap arm's *structure* (opcode, what gets pushed, whether hashing happens
at all) rather than the choice of hash function, and all three were caught by
this test. `btcaddr.Hash160` itself is an unmodified third-party primitive,
not code this stage writes, so verifying the SUT calls it correctly on the
right input with the right opcodes is a legitimate (if not maximally
independent) check. Recorded for visibility, does not gate.

**Task 3 Step 5 claim, verified:** with mutation 2a applied, `go test -run
'TestPkhWitnessScriptsReproduceRustsAddresses' ./md/ 2>&1 | grep -c 'rust:'`
→ **10**. The plan's claimed count is correct.

### 3. `md/policy_shape.go`

| # | Mutation | Plan lines | Test that failed |
| - | - | - | - |
| 3a | `splitBranches`, `tagAndOr` arm: treat as two children (X, Z), drop Y | 2024–2040 | `TestPolicyShapeSplitsAndOr` only (no vendored vector emits `andor`, as the plan's own comment says; the four pre-existing `TestPolicyShape*` tests and the three other new tests are correctly unaffected) |
| 3b | `splitBranches`, `or_*` arm: don't recurse into the right child (`branchOf` instead of `splitBranches`) | 2010–2023 | `TestPolicyShapeSplitsAlternativesIntoBranches` and `TestPolicyShapeWalksAnEightPathChain`. `TestPolicyShapeReportsAKeylessAlternativeHonestly` stays PASS — verified this is benign: its vector's `or_d` has only 2 paths, so the right child is already a leaf and `branchOf`/`splitBranches` agree there; the two-vector tests above catch the general case (nesting depth ≥ 2). |
| 3c | `collect`, `tagSha256` arm: set `Hashlock` but never append to `Sha256Digests` | 2071–2076 | `TestPolicyShapeSplitsAlternativesIntoBranches` and `TestPolicyShapeReportsAKeylessAlternativeHonestly` |

All three caught. No finding.

### 4. `sysw/composer_records.go`

| # | Mutation | Plan lines | Test that failed |
| - | - | - | - |
| 4a | `unhexLower`: also accept `A`–`F` | 2660–2677 | `TestComposerRecordsClassifyExactlyAsTheHost` and `TestKeyRecordPathGrammarMatchesTheHost` (confirmed via the `73C5DA0A/...` bad-case row) |
| 4b | `ParseKeyRecord`: accept depth 5 | 2813–2816 | `TestComposerRecordsClassifyExactlyAsTheHost` only — traced to fixture row **`key-depth-5-refused`** (`Classify(...) = ClassKey, want ClassUnknown`). `TestKeyRecordPathGrammarMatchesTheHost` and the parser-value test do not exercise depth 5, so this rule has exactly one line of coverage, from the fixture; that line does catch it. |
| 4c | `ParseKeyRecord`: drop the `origin[last] == ek.ChildIndex()` check | 2820–2822 | `TestComposerRecordsClassifyExactlyAsTheHost` and `TestKeyRecordPathGrammarMatchesTheHost` (`.../48'/0'/0'/3'` bad-case row) |
| 4d | `digitsInRange`: allow `maxDigits+1` characters | 2694–2712 | **NONE — see I-1 below.** |
| 4e | `parseOriginPath`: accept `H` as a hardening marker (trim it locally and normalize to lowercase `h` before calling `bip32.ParsePathElement`) | 2742–2772 | `TestComposerRecordsClassifyExactlyAsTheHost` and `TestKeyRecordPathGrammarMatchesTheHost` |

4a/4b/4c/4e caught (4b by exactly one fixture row, still catches it). 4d is
a genuine coverage gap (Important) — see finding I-1.

### 5. `md/compose_vectors_pin_test.go` (pin mechanics, plan lines 65–184)

| Mutation | Test that failed |
| - | - |
| Flip one hex character in `keyed_compose_wsh_sole_sortedmulti.bytes.hex` | `TestComposeVectorsMatchTheirProvenancePin` (sha256 mismatch) |
| Remove `"keyed_compose_wsh_unsorted_sole"` from `composeVectorNames` | `TestComposeVectorsMatchTheirProvenancePin` (both the `Vectors != len(...)` count check and the "pinned file whose vector is not named here" stray check) |
| Add a stray file `compose_stray_extra.bytes.hex` to `md/testdata/vectors/` (not in the provenance JSON, not in `composeVectorNames`) | **NONE — the whole `./md/` package stays `ok`. See C-1 below.** |

### 6. `mk/compose_stubs.go`

| Mutation | Test that failed |
| - | - |
| Drop the dedup loop in `AppendStubs` | `TestAppendStubsPreservesExistingAndAddsEachOnce` |
| Drop the defensive copy (`out.Stubs = card.Stubs` instead of `make`+`append`) | **NONE — the whole `./mk/` package stays `ok`. See C-2 below.** |

## Findings

### I-1 (Important): `now:` record's digit-COUNT bound has zero test coverage — a spec §6a rule with no test that catches its violation

Spec `SPEC_wallet_policy_composer.md` §6a states the rule explicitly: `now:`
"MUST match `^[0-9]{1,10}(,[0-9]{1,9})?$`" — a length bound independent of
the numeric-range bound stated in the same sentence. Plan lines 2714–2740
port this as `digitsInRange(s, maxDigits, lo, hi)`, called with
`maxDigits=10` for seconds and `maxDigits=9` for height (plan lines
2727, 2733).

Mutating the length check from `len(s) > maxDigits` to `len(s) >
maxDigits+1` (i.e., silently allowing one extra digit) is **not caught by
any test the plan specifies**, and the whole `./sysw/` package (40/40
fixture rows) stays green. Verified with a concrete example, not just an
absence of failure: the record `now:` + hex(`"01756684800"`) — 11 ASCII
digits, leading zero, decoding to the in-range value 1,756,684,800 — is
accepted by `ParseNowRecord` and classified `ClassNow` under the mutation.
None of the 40 vendored fixture rows exercises an in-range value with an
over-length digit string (checked: `now-seconds-2^31` and
`now-zero-seconds` both test out-of-*range* values, which the separate `v <
lo || v > hi` check catches regardless of digit count; there is no
`now-seconds-too-many-digits`-shaped row, nor an equivalent for `key:`'s
depth-vs-path-length rule beyond the depth bound itself).

Classified Important rather than Critical per this review's own rubric:
there is no single assertion here that "cannot fail" (a false-PASS-path /
test-that-cannot-fail defect) — there is simply no test at all for this
sub-rule, which is the Important category ("a spec §6a/§4c/§5 rule with no
test that catches its violation"). It still gates: the rule is stated
explicitly by name in the spec (§6a's regex) and restated as load-bearing in
the plan's own doc comment (plan line ~2715: "hex of
`<seconds>[,<height>]`, seconds 1..=2^31-1 (10 digits at most)..."), and a
violation of it produces a silent false PASS across the entire test surface
this task ships.

Fix direction (not prescriptive per the review's own rule against
prescribing fixes): add at least one fixture-shaped row (or a plan-level Go
test) with an in-range value padded to `maxDigits+1` digits via a leading
zero, for both `seconds` and `height`.

### C-1 (Critical): the compose-vector pin test doesn't scan the vectors directory — a stray file is invisible, contradicting the test's own doc comment

`md/compose_vectors_pin_test.go` (plan lines 103–106) states as its own
rationale: "checked against the pin below so a file copied in without a
name here (or a name with no file) fails rather than silently asserting
nothing." Empirically this is **false** for the "file copied in" half of the
claim. `TestComposeVectorsMatchTheirProvenancePin` (plan lines 139–169)
never calls `os.ReadDir` (or equivalent) on `testdata/vectors/`; it only
iterates `p.Files` (the list *inside* the provenance JSON) and
`composeVectorNames`. A file placed in `md/testdata/vectors/` that is
**not referenced by the provenance JSON at all** is invisible to this test
by construction — there is no code path that could ever see it. Verified:
adding `md/testdata/vectors/compose_stray_extra.bytes.hex` (content
`deadbeef\n`, referenced nowhere) leaves the entire `./md/` package
(all tests, not just the pin test) at `ok`.

This matters because the brief's own §5 explicitly asks "add a stray file —
does it fail?" as a designed check on this exact test, and the test's own
inline comment asserts the behavior the mutation just disproved. It is a
narrower gap than it may first appear — a stray file that also gets an
entry added to the provenance JSON's `files` list (with a correct sha256)
*would* be caught, since it would then appear in the `seen` map with no
matching `composeVectorNames` entry — but "copy a file in" without touching
the JSON, the literal scenario the comment describes, passes silently.

Fix direction: have the test additionally `os.ReadDir("testdata/vectors")`
and assert every entry's base name (up to the first `.`) is in
`composeVectorNames`.

### C-2 (Critical): `AppendStubs`'s "input not mutated" test cannot detect the absence of a defensive copy, by Go slice semantics

`mk/compose_stubs_test.go`'s `TestAppendStubsPreservesExistingAndAddsEachOnce`
(plan lines 2187–2216) asserts, after calling `AppendStubs(card, tmpl, pol,
tmpl)`, that `reflect.DeepEqual(card.Stubs, [][4]byte{existing})` still
holds — intending to prove the input wasn't mutated. Removing the defensive
copy entirely (`out.Stubs = card.Stubs` instead of `make([][4]byte, 0,
...)` + `append`) leaves this assertion **passing**, and the whole `./mk/`
package (14/14 tests) stays `ok`.

Mechanism, verified: `card.Stubs` is a Go slice *header*, passed to
`AppendStubs` by value. `append`ing to `out.Stubs` inside the function can
never change the caller's `card.Stubs` **length** field regardless of
aliasing, and `AppendStubs` never writes to an index *within* the original
length (it only appends past it) — so the only way this assertion could
ever fail is if the function shrank or reordered the existing elements,
which no plausible implementation of "append some stubs" would do. The test
is therefore structurally incapable of detecting whether `AppendStubs`
defensively copies its input or aliases it, independent of any accidental
behavior of Go's `append` reallocation (which happened not to matter here
either, since the input slice literal has cap==len==1 and grows via
reallocation regardless).

This is squarely a "test that cannot fail" per the review's severity
definition — the assertion exists, reads as if it tests the stated
no-mutation contract, and cannot observe a violation of it as constructed.

Fix direction: to make this observable, build the input with spare
capacity (e.g., `existing := make([][4]byte, 1, 4); existing[0] = ...`) and,
after calling `AppendStubs`, `append` a **different** sentinel to the
*original* `card.Stubs` slice and check it doesn't reappear read back
through `got.Stubs` (or vice versa) — a genuine shared-backing-array probe,
not a length/content-in-range check.

### M-1 (Minor): stale line citation for `bip32.Path.String()`

Plan line 2832 cites `bip32/bip32.go:103-110` for the claim that
`bip32.Path.String()` renders `m/48h/0h/0h/2h`. In the actual file (scratch
copy, matching the fork), `String()` is at lines 20–35; lines 102–114 hold
a separate, near-duplicate method `Encode()` with the same suffix logic.
The **behavioral claim is true** (verified: `String()` writes `'h'` for
every hardened component, no other spelling), only the citation is off by
~80 lines — likely drift from an earlier version of the file. Non-blocking.

### M-2 (Minor): Task 9's `gui-shard-test.sh` alternative doesn't exist at the path given

Plan line 2963: "`./gui/` runs here as CI runs it; `scripts/gui-shard-test.sh
./gui/ 24` is the faster local equivalent — either, but at least one." Run
from the fork root (`/scratch/code/shibboleth/seedhammer`, the cwd every
other Task 9 command in this plan assumes — `scripts/test-32bit.sh` does
exist there and was run successfully), `scripts/gui-shard-test.sh` **does
not exist**. The script lives only in `mnemonic-engrave/scripts/
gui-shard-test.sh` (this repo, not the fork). Confirmed by `find` on both
trees.

This doesn't block Task 9's gate — Step 1's primary command (`go test
-timeout 20m ./...`) already covers `./gui/` and was verified runnable
(other flags/tags checked below) — but the "faster local equivalent"
parenthetical is a broken pointer as written. Fix direction: either name
the correct relative path from the fork root
(`../mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`) or drop the
"either" framing.

### N-1 (Nit): see §2's table entry above (pk_h tap test is self-consistent, not independently oracled) — recorded there, non-blocking.

## Task 8 / Task 9 command-runnability audit (from the fork root, nix firmware build NOT run per instructions)

- Task 8 Step 1 (`nix run .#build-firmware`): target confirmed to exist,
  `flake.nix:93`. Not run (slow, per instructions).
- Task 8 Step 2 (`tinygo build -size short -target pico-plus2 -stack-size
  16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`): flags are
  **byte-identical** to the flake's canonical `tinygo-flags` string
  (`flake.nix:80`: `-target pico-plus2 -stack-size 16kb -gc precise -opt 2
  -scheduler tasks`); CI's own build (`.github/workflows/test.yml:135`)
  uses `-size full` instead of `-size short` — a verbosity difference only.
  `tinygo` itself is not installed in this shell (`which tinygo` → not
  found), consistent with it being a nix-shell-provided tool; not run.
  Baseline numbers (1,503,652 B flash / 62,592 B RAM) are stated in the
  plan (line 34) and are not independently re-derivable without the build.
- Task 9 Step 1 (`go test -timeout 20m ./...`): standard, not run in full
  (large `gui` package, would cost real wall time in this review); the
  command syntax is unremarkable.
- Task 9 Step 2, line 1 (`scripts/test-32bit.sh`): **exists and runs
  successfully** from the fork root — verified (`ok seedhammer.com/sysw`,
  `GOARCH=386 test: exit 0`, `GOARCH=arm build: exit 0`).
- Task 9 Step 2, line 2 (`go test -tags oraclelive -run '^$' ./oracle/
  ./gui/ ./sysw/`): **runs successfully** — verified, all three packages
  `ok ... [no tests to run]` (the `-run '^$'` pattern deliberately matches
  no test names, so this is a build-tag compile check, and it compiles).
- Task 9 Step 2, line 3 (`GOOS=js GOARCH=wasm go vet ./cmd/emu/`):
  **runs successfully**, no output (clean vet).
- Task 9 Step 2, line 4 (`gofmt -l md/ mk/ sysw/ gui/ scripts/`): not
  separately verified beyond the gofmt checks already run per-task above
  (all clean throughout this review).
- The one broken reference found: M-2 above.

## Expected-line audit

Every `Expected:` line in the plan (44 occurrences) was read. Machine-checked
where the post-implementation state made it checkable without reverting the
implementation:

| Plan line | Claim | Verified |
| - | - | - |
| 246 | `vendored 126 files, 26 vectors, primary 66bdf2f47e7f` | TRUE — provenance JSON: `commit: 66bdf2f47e7fc703d5fb09120122b3e98cab5528` (prefix matches), `vectors: 26`, `files: 126` |
| 251 | `TestKeyedConformanceAgreesWithRust` has 36 sub-tests (14 before) | TRUE — `go test -v -run ...` lists 36 sub-tests; independently: 14 non-compose + 22 compose `keyed_*.conformance.json` files = 36 |
| 1508 | "All 28 family rows byte- and chunk-identical" | TRUE — `TestComposeReproducesEveryVectorByteForByte` runs exactly 28 sub-tests, all pass at baseline |
| 1768 | "ten receive addresses across five vectors" | TRUE — `TestPkhWitnessScriptsReproduceRustsAddresses` runs 5 sub-tests × 2 addresses = 10 |
| 1772 (Step 5's "10") | Hand-mutation address-move count | TRUE — reproduced exactly, see §2 above |
| 2293 | Stub test PASS, stubs equal first-8-hex of Rust ids | TRUE — `TestComposerStubsAreTheTwoIdsFirstFourBytes` passes |
| 2329 | fixture sha256 `a894e619...` | TRUE — `sha256sum` of the vendored file matches exactly |
| 2832 | `bip32.Path.String()` renders `m/48h/0h/0h/2h`; cites `bip32/bip32.go:103-110` | Behavior TRUE; **citation stale** (M-1) |
| 2973 (family tag coverage aside, plan's own claim) | `grep -c 'Or\|andor' md/policy_shape_test.go` = 0 | TRUE |

No `Expected:` line was found to be vague ("should work", "all pass" with
no count) — every one names a specific string, count, or exit condition.
No `Expected:` line was found to be **false** against the scratch copy.

## Closing counts

- **7/7** compose.go mutations caught.
- **3/3** script_emit.go pk_h mutations caught (1 Nit on test independence).
- **3/3** policy_shape.go mutations caught.
- **4/5** composer_records.go mutations caught (1 Important: digit-count bound has no test at all).
- **2/3** pin-test scenarios caught (1 Critical: stray-file blind spot — an assertion exists and cannot fail).
- **1/2** mk/compose_stubs.go mutations caught (1 Critical: copy-vs-alias untestable by the given assertion).
- Every checkable `Expected:` line verified TRUE; none vague, none false.

**C-1, C-2 / I-1 / M-1, M-2 / N-1** → **2 Critical, 1 Important, 2 Minor, 1 Nit.**
