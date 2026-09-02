# Composer Stage 2 (fork codec) — implementation report

**Implementer:** single agent, executing `design/IMPLEMENTATION_PLAN_composer_S2_fork_codec.md`
at mnemonic-engrave master `38e3ed13eb0d903ae2d24e64edc830a9484dcc6e` (the S2 GREEN revision).

**Worktree:** `/scratch/code/shibboleth/wt-composer-s2`, branch `composer-s2`, forked from
seedhammer fork `169073c` (the plan's baseline). Nothing pushed, tagged, published or flashed.
The main fork checkout, mnemonic-engrave and mnemonic-secret were not modified (this report is
the single file written outside the worktree, as the brief directs).

**Outcome: Tasks 1–8 complete and green. Task 9 Step 1 is RED and I STOPPED there.**
Three PRE-EXISTING `gui/` tests fail on this branch. The brief forbids editing a pre-existing
test, so no fix was attempted. Two of the three are deliberate tripwires firing exactly as
their comments say they should; the third is a genuine WRONG-ADDRESS defect in the fork's
shipped tapscript emitter, exposed for the first time by the vendored corpus. Details in
**Task 9** and **Findings** below.

```
$ git log --oneline 169073c..HEAD
fa52bb3 gui: tie Multisig Build's script-type table to the composer's (tr = 3') (composer S2 task 7)
7ac35dc sysw: key:/hash:/now: record classes, lockstep with the host's 45-row fixture (composer S2 task 6)
be298ef md, mk: ComposerStubs (template stub, plus the keyed policy's after seating) and AppendStubs for re-minted cards (composer S2 task 5)
99bcc9b md: PolicyShape splits or_*/andor into one Branch per alternative, carrying typed Locks, sha256 digests and Sorted (composer S2 task 4)
301ce78 md: pk_h emitter arm in both script contexts, pinned to Rust's addresses for five pkh vectors (composer S2 task 3)
33fedc5 md: the composer's tree builder -- Compose/ComposeWith, FIXED lowering, 4f origins, byte parity with all 28 family vectors (composer S2 task 2)
79fa1de md: vendor the composer's 26-vector corpus with a provenance pin (composer S2 task 1)
```

Every commit is `-s` signed-off (author Brian Goss), carries the plan's message verbatim and the
two trailer lines. `gofmt -l` was clean on every touched file before each commit;
`go vet ./md/ ./mk/ ./sysw/` exits 0 at the tip.

Preconditions verified before starting, not assumed:

```
$ sha256sum crates/me-cli/testdata/record_class_vectors.json      # mnemonic-engrave master
eed6b177d1a3406a69c4a0102635f5d59c6412fa65e106f85b831c4736ac464e   (45 rows)
$ git -C mnemonic-engrave log -1 --format=%H -- crates/me-cli/testdata/record_class_vectors.json
5720e3c0747f72e7c6a6225b2993db9d0d40d24e   (ancestor of master 38e3ed13 — Task 6 precondition MET)
$ git -C descriptor-mnemonic rev-parse HEAD
66bdf2f47e7fc703d5fb09120122b3e98cab5528   (clean tree)
```

---

## Task 1 — vendor the compose corpus with a provenance pin  → commit 79fa1de

**Step 2 (red).**
```
--- FAIL: TestComposeVectorsMatchTheirProvenancePin (0.00s)
    compose_vectors_pin_test.go:78: INCONCLUSIVE: no provenance pin at testdata/compose_vectors.provenance.json: open testdata/compose_vectors.provenance.json: no such file or directory
--- FAIL: TestEveryKeyedComposeVectorHasAConformanceRecord (0.00s)
    compose_vectors_pin_test.go:132: keyed_compose_sh_sole: stat testdata/vectors/keyed_compose_sh_sole.conformance.json: no such file or directory
```
Matches the Expected line.

**Step 3 (vendoring).**
```
$ scripts/vendor-compose-vectors.sh /scratch/code/shibboleth/descriptor-mnemonic
vendored 126 files, 26 vectors, primary 66bdf2f47e7f
$ ls md/testdata/vectors | grep -cE '^(keyed_)?compose_'
126
```
Byte-identical to the Expected line. The pin JSON is generated; nothing was pasted by hand.

**Step 4 (green).** Three top-level tests PASS, no FAIL lines, and the keyed conformance gate
grew as predicted — measured, not described:
```
top-level PASS: 3
TestKeyedConformanceAgreesWithRust sub-tests: 36   (22 of them keyed_compose_*)
FAIL lines: 0
ok  	seedhammer.com/md	0.013s
```
14 → 36 exactly as the plan says. Every one of the 22 new keyed vectors agrees with Rust's ids
before a line of builder code existed.

**Deviation:** none.

---

## Task 2 — the `md` builder  → commit 33fedc5

**Step 2 (red).**
```
# seedhammer.com/md [seedhammer.com/md.test]
md/compose_test.go:27:21: undefined: SpendPath
md/compose_test.go:28:21: undefined: SpendPath
md/compose_test.go:29:12: undefined: SpendPath
md/compose_test.go:29:25: undefined: Lock
```
**Deviation (cosmetic).** The Expected line says `undefined: Compose` (and the other new names);
the compiler reports the first undefined symbol in file order, `SpendPath`. Same failure class.

**Step 2a (the `.descriptor.json` loader).** Applied exactly as written: the `Hash256Body` /
`Hash160Body` arms and the `buildTLV` pubkey arm now read hex STRINGS, and `hexBody` was
appended. Confirmed against the Expected line:
```
$ go test -count=1 ./md/          # with compose_test.go held aside, see deviation
ok  	seedhammer.com/md	0.021s
```
**Deviation (plan ordering, not content).** The Expected line for Step 2a — "*Then run
`CGO_ENABLED=0 go test -count=1 ./md/ 2>&1 | tail -2`: the existing tests do not reach these
arms and stay `ok`*" — **cannot be run as written**: `md/compose_test.go` was created in Step 1
and does not compile until Step 3, so the package build fails first
(`FAIL seedhammer.com/md [build failed]`). To perform the check the plan actually intends, I
moved `md/compose_test.go` out of the package directory, ran the command (result above), and
moved it straight back. Nothing was deleted or edited. The check is real: the pre-existing md
tests stay `ok` under the loader change.

**Step 4 (green).**
```
exit=0
(no FAIL lines)
TestComposeReproducesEveryVectorByteForByte sub-tests PASS: 28
ok  	seedhammer.com/md	0.006s
```
All 28 family rows — 26 vendored vectors byte- and chunk-identical, plus the two `no-corpus`
keyless-wsh shapes against the chunk-set literals. No tree, pathDecl, useSite, payload-byte or
chunk-string differed anywhere; no lowering rule had to be reconsidered.

**Step 5.** `gofmt -l md/` silent; `ok seedhammer.com/md 0.063s`.

**Deviation (staging).** The plan's commit block stages only `md/compose.go md/compose_test.go`,
but Task 2's own Step 2a modifies `md/testdata_test.go` (and the task's **Files** header lists
it as Modified). I staged all three explicitly, so the loader fix travels with the task that
required it rather than being left dirty across later commits.

---

## Task 3 — the `pk_h` emitter arm  → commit 301ce78

**Step 2 (red).** All five oracle sub-tests plus both unit tests fail:
```
--- FAIL: TestPkhWitnessScriptsReproduceRustsAddresses/keyed_compose_wsh_two_path_or_d
    compose_pkh_emit_test.go:103: EmitWitnessScriptChunks(chain 0 index 0): md: script fragment not supported for emission
--- FAIL: TestPkhScriptDependsOnTheKey
--- FAIL: TestPkhTapLeafEmitsTheHash160Form
    compose_pkh_emit_test.go:125: md: script fragment not supported for emission
```
**Deviation (wording only, and the plan anticipated it).** The Expected line quotes
`md: script emission unsupported`; the package spells `ErrScriptUnsupported` as
`md: script fragment not supported for emission`. The plan explicitly deferred to "the
`ErrScriptUnsupported` text as the package spells it".

The tap test failed for the SAME reason as the wsh ones (the emitter's default arm), not for a
construction problem — so the plan's fallback ("if the TAP test fails for a different reason …
fix the TEST's construction") was **not** needed: `split`, `Reassemble`, `EmitTapLeavesChunks`
and `TapLeafScript.Script` all took the hand-built tree as written.

**Step 4 (green).**
```
--- PASS: TestPkhWitnessScriptsReproduceRustsAddresses (0.01s)   [5 sub-tests, all PASS]
--- PASS: TestPkhScriptDependsOnTheKey (0.00s)
--- PASS: TestPkhTapLeafEmitsTheHash160Form (0.00s)
ok  	seedhammer.com/md	0.016s
```
Twenty addresses (chains 0 and 1, indices 0 and 1, five vectors) equal Rust's.

**Step 5 — the opcode mutation, count as required.**
```
$ # opEQUALVERIFY -> opEQUAL in the NEW arm only
$ go test -count=1 -run 'TestPkhWitnessScriptsReproduceRustsAddresses' ./md/ 2>&1 | grep -c 'rust:'
20
```
**The count is 20** — every address moved. Reverted; `git diff --stat` afterwards showed only
`md/script_emit.go | 25 +++…` (the intended arm), and Step 4 re-ran green.

---

## Task 4 — `PolicyShape` splits alternatives  → commit 99bcc9b

**Step 2 (red).**
```
md/compose_shape_test.go:34:30: unknown field Locks in struct literal of type Branch
```
Matches the Expected compile error.

**Step 4 (green).** All twelve `TestPolicyShape*` tests pass — the eight new ones AND the four
pre-existing ones, which were not touched:
```
--- PASS: TestPolicyShapeSplitsAlternativesIntoBranches            [5 sub-tests]
--- PASS: TestPolicyShapeSplitsTheShippedOrCards                   [3 sub-tests]
--- PASS: TestPolicyShapeDistinguishesOlderFromAfterAtTheSameOperand
--- PASS: TestPolicyShapeCarriesSortedForThresholds
--- PASS: TestPolicyShapeWalksAnEightPathChain
--- PASS: TestPolicyShapeTapLeavesCarryLocks
--- PASS: TestPolicyShapeReportsAKeylessAlternativeHonestly
--- PASS: TestPolicyShapeSplitsAndOr
--- PASS: TestPolicyShapeDescribesRealCards                        (pre-existing)
--- PASS: TestPolicyShapeNeverClaimsAPlainThresholdItCannotSee     (pre-existing)
--- PASS: TestPolicyShapeRefusesAnUnknownTag                       (pre-existing)
--- PASS: TestPolicyShapeReportsEveryLeafOfADeepTree               (pre-existing)
ok  	seedhammer.com/md	0.002s
```
`git diff --stat` before staging showed `md/policy_shape.go` alone (90 insertions, 19 deletions)
— `md/policy_shape_test.go` was not edited. The three shipped `or_*` cards now summarise as
2, 2 and 2 spend paths, pinned by `TestPolicyShapeSplitsTheShippedOrCards`.

**Deviation:** none.

---

## Task 5 — `md.ComposerStubs`, `mk.AppendStubs`  → commit be298ef

**Step 2 (red).** `undefined: ComposerStubs` / `undefined: AppendStubs`, as Expected.

**Step 4 (green).**
```
--- PASS: TestComposerStubsAreTheTwoIdsFirstFourBytes (0.00s)     ok  seedhammer.com/md
--- PASS: TestAppendStubsPreservesExistingAndAddsEachOnce (0.00s)
--- PASS: TestAppendStubsDoesNotShareTheInputsBackingArray (0.00s) ok  seedhammer.com/mk
```
The two stubs equal the first eight hex characters of Rust's `wallet_descriptor_template_id`
and `wallet_policy_id`.

**The Expected line's aliasing claim was machine-checked, not taken on trust.** Dropping the
defensive copy (`out.Stubs = card.Stubs`) makes the probe fail with exactly the predicted
message, and it passes again on revert:
```
--- FAIL: TestAppendStubsDoesNotShareTheInputsBackingArray (0.00s)
    compose_stubs_test.go:57: appending to the input changed the result to [deadbeef ffffffff]: AppendStubs aliased the input's array
… (reverted) …
ok  	seedhammer.com/mk	0.001s
```

**Deviation:** none.

---

## Task 6 — `sysw` composer record classes  → commit 7ac35dc

Fixture vendored and verified in place: `sha256 eed6b177…464e`, 45 rows. The provenance pin's
two angle-bracket fields were filled from the two named commands with full 40-character SHAs:
`commit 38e3ed13eb0d903ae2d24e64edc830a9484dcc6e`,
`file_commit 5720e3c0747f72e7c6a6225b2993db9d0d40d24e`, `repo_clean_when_recorded true`
(mnemonic-engrave's tree had 0 modified paths at vendoring time).

**Step 2 (red).** `sysw/composer_records_test.go:24:9: undefined: ClassKey`, as Expected.

**Step 4 (green) — every one of the 45 rows agrees with the host, and the pre-existing
classification gates still pass:**
```
--- PASS: TestClassifyMatchesTheRustPrimary (0.00s)                 (pre-existing)
--- PASS: TestClassifyRejectsMs1RustWouldRefuse (0.00s)             (pre-existing)
--- PASS: TestClassifyMtAndTxRecordsMatchesTheRustPrimary (0.00s)   (pre-existing)
--- PASS: TestComposerRecordsClassifyExactlyAsTheHost (0.00s)
--- PASS: TestComposerRecordParsersReturnTheHostsValues (0.00s)
--- PASS: TestComposerClassesArePrefixMatchedAndNotSecret (0.00s)
--- PASS: TestNowRecordDigitCountIsBoundedIndependentlyOfRange (0.00s)
--- PASS: TestKeyRecordPathGrammarMatchesTheHost (0.00s)
ok  	seedhammer.com/sysw	0.002s
```
No lockstep disagreement on any row; `bip32.Path.String()` renders `m/48h/0h/0h/2h` as the plan
measured. Whole package: `ok seedhammer.com/sysw 0.113s`.

**Deviation (additive only).** I added `TestNowRecordDigit` to the Step 4 `-run` filter, which
the plan's filter omits although the test is part of Step 1. Superset of what was asked.

---

## Task 7 — the taproot `3'` origin tie  → commit fa52bb3

```
$ go test -count=1 -run 'TestComposerOriginTableAgreesWithMultisigBuild' ./gui/
ok  	seedhammer.com/gui	0.002s
```
Passes at once, as the plan says it should (the arm lives in `md`; this task's product is the
tie). The doc comment was added after the `(buildOriginAnnouncement).` sentence.

**Deviation / side effect worth recording — `GOFLAGS=-mod=mod` rewrites `go.mod`.** The first
command that pulled `seedhammer.com/address` into the build graph (this task's `./gui/` run)
made Go promote `github.com/btcsuite/btcd/chainhash/v2 v2.0.0` from the `// indirect` block to
the direct `require` block, because `address/taproot_script_path.go:12` imports it directly and
the shipped `go.mod` marks it indirect. That is a pre-existing metadata inaccuracy in the fork,
not a consequence of any code in this plan. I reverted `go.mod` (`git checkout go.mod`) and ran
every subsequent command in Go's DEFAULT `-mod=readonly`, which is what CI uses
(`.github/workflows/test.yml:75` runs a bare `CGO_ENABLED=0 go test -timeout 20m ./...`).
Readonly builds and tests fine offline here, so nothing was lost. **`go.mod` and `go.sum` are
unchanged on this branch.**

---

## Task 8 — firmware build and size delta

**Step 1.**
```
$ nix run .#build-firmware
Built seedhammerii-v0.0.0-bgfa52bb3.uf2        (exit 0)
```

**Step 2.** `tinygo` is not on PATH outside the dev shell, so the size build ran as
`nix develop --command tinygo build -size short -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`:
```
   code    data     bss |   flash     ram
1475184   31636   30956 | 1506820   62592
```

**I did not take the baseline on trust — I measured it.** With the tree clean I detached to
`169073c` inside the same worktree, ran the identical command, and returned to `composer-s2`:
```
   code    data     bss |   flash     ram
1472016   31636   30956 | 1503652   62592          (fork 169073c)
```
That reproduces the plan's baseline **1,503,652 B flash / 62,592 B RAM exactly.**

| | flash | RAM |
|---|---|---|
| baseline `169073c` | 1,503,652 B | 62,592 B |
| tip `fa52bb3` | 1,506,820 B | 62,592 B |
| **delta** | **+3,168 B (+0.21 %)** | **0 B** |

**Deviation from the Expected line, and it needs the sentence the plan asks for.** The plan
expects "ZERO or near it … a non-zero delta … means something is already reachable". It is
non-zero, and two things are indeed already reachable — neither of them the builder:

1. `md/script_emit.go` now imports `github.com/btcsuite/btcd/address/v2` for `Hash160`, and the
   new `case tagPkH:` sits inside `emitFragment`, which the shipped GUI already calls. At the
   baseline no non-test file in `md/` imported that package
   (`git grep -l 'btcsuite/btcd/address/v2' 169073c -- 'md/*.go'` is empty; at the tip it is
   `md/script_emit.go`).
2. `sysw.Classify` — reachable from the shipped scan door — now calls `classifyComposer`, so
   `ParseKeyRecord` and its `hdkeychain.NewKeyFromString` are live device code. At the baseline
   only `sysw/descriptor.go` used `hdkeychain`; at the tip `sysw/composer_records.go` does too.

`md.Compose` and the `sysw` parsers' public surface are otherwise unreferenced by
`cmd/controller` until Stage 3, as the plan predicted. RAM is unmoved.

---

## Task 9 — whole-repository gates  → **STOPPED, RED**

### Step 1 — `CGO_ENABLED=0 go test -timeout 20m ./...` (CI's command)

**53 packages `ok`; one package FAILs.** `md`, `mk`, `sysw` and `address` are all green
(`ok seedhammer.com/md 0.054s`, `mk 0.028s`, `sysw 0.051s`, `address 0.066s`; 540 PASS lines
across the three plan packages). The failure is `seedhammer.com/gui`, three tests, **all three
pre-existing and none of them edited by me:**

```
FAIL	seedhammer.com/gui	162.671s
--- FAIL: TestEveryKeyedVectorReachesAnAddress (0.13s)
    --- FAIL: TestEveryKeyedVectorReachesAnAddress/keyed_compose_tr_nums_three_leaves (0.00s)
        policy_address_test.go:148: chain 0 index 0 via complex:
             got  bc1pzzy3pcnzmsgsq8t2jv0yt234gray8mzctjqkcmle4p8xy7c0pcwqe5nq3j
             want bc1pxv80xfntj9qjhljsr3j5v0xvppwltvrq8qwp9jkdpd96q33ay6lssl42jw (rust)
--- FAIL: TestPkhTapLeafGapIsPinnedByShape (0.00s)
    policy_address_test.go:411: THE GAP IS CLOSED: this port now derives bc1pl3kq0susghpu43eszll3ym5wjq9uva3enmwj49akc7v5rr68aw9qjz8aja for a pkh tap leaf, matching Rust. Convert this to a positive test, as its predecessor was.
--- FAIL: TestWalletPolicyConsentNeverHidesTheAbsenceOfAddresses (0.00s)
    wallet_policy_test.go:109: an underivable KEYED policy does not say so: … Receive 0: bc1pl3kq0su… 
    wallet_policy_test.go:117: a policy this device cannot derive shows an address label "Receive 0:" …
    wallet_policy_test.go:117: a policy this device cannot derive shows an address label "Change 0:" …
```

I stopped here rather than touching them, per the brief ("NO pre-existing test may be edited; if
one fails, stop and record it"). Task 9 Step 3 (this report) is the only step after it.

**Attribution is measured, not guessed.** I ran the three tests at the baseline and at each of
the seven commits with a `-run` filter:

| revision | task | result |
|---|---|---|
| `169073c` | baseline | **ok** — all three green |
| `79fa1de` | T1 vendor corpus | FAIL: `TestEveryKeyedVectorReachesAnAddress` |
| `33fedc5` | T2 builder | same one |
| `301ce78` | T3 `pk_h` arm | **+** `TestPkhTapLeafGapIsPinnedByShape`, `TestWalletPolicyConsentNeverHidesTheAbsenceOfAddresses` |
| `99bcc9b` … `fa52bb3` | T4–T7 | unchanged (same three) |

So: **Task 1 alone turns the address gate red; Task 3 alone turns the two tripwires red.**

### Step 2 — the remaining CI gates (all green)

```
$ scripts/test-32bit.sh                → exit 0
GOARCH=386 test:  exit 0
GOARCH=arm build: exit 0
$ go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/
ok  	seedhammer.com/oracle	0.001s [no tests to run]
ok  	seedhammer.com/gui	0.003s [no tests to run]
ok  	seedhammer.com/sysw	0.002s [no tests to run]
$ GOOS=js GOARCH=wasm go vet ./cmd/emu/     → exit 0, no output
$ gofmt -l md/ mk/ sysw/ gui/ scripts/
gui/transaction.go
gui/transaction_golden_test.go
gui/transaction_txrecord_test.go
```
**Deviation from "gofmt prints nothing".** Those three files are unformatted **at the baseline
too** — I re-ran `gofmt -l` detached at `169073c` and got the identical three names. The diff is
stray blank lines (`gui/transaction.go:191`, two blank lines to remove). Nothing on this branch
touched them, and CI has no gofmt job (`grep -rn 'gofmt' .github/workflows/*.yml` is empty), so
this is a pre-existing repo nit, not a regression. I left them alone: they are outside the plan.

---

## Findings

### F-1 (Critical, PRE-EXISTING defect, exposed by this branch) — `v:multi_a` emits the wrong tapscript, so the device shows a WRONG taproot address

`keyed_compose_tr_nums_three_leaves` derives
`bc1pzzy3pcnzmsgsq8t2jv0yt234gray8mzctjqkcmle4p8xy7c0pcwqe5nq3j` where Rust derives
`bc1pxv80xfntj9qjhljsr3j5v0xvppwltvrq8qwp9jkdpd96q33ay6lssl42jw`. Its third leaf is
`and_v(v:multi_a(2,@2,@3),after(2))` — the first VERIFY-WRAPPED `multi_a` any vector in this
repo has ever contained.

The cause is in `md/script_emit.go`, not in anything this plan wrote. The `case tagVerify:` arm
folds the wrapped fragment's final opcode into its VERIFY form for exactly three opcodes:

```go
	switch (*out)[len(*out)-1] {
	case opCHECKSIG:
		(*out)[len(*out)-1] = opCHECKSIGVERIFY
	case opCHECKMULTISIG:
		(*out)[len(*out)-1] = opCHECKMULTISIGVER
	case opEQUAL:
		(*out)[len(*out)-1] = opEQUALVERIFY
	default:
		*out = append(*out, opVERIFY)
	}
```

`multi_a` ends in `opNUMEQUAL` (`md/script_emit.go:546`, constant at `:51`, `0x9c`), which is not
in that table, so `v:multi_a(…)` emits `… OP_NUMEQUAL OP_VERIFY` where miniscript emits
`… OP_NUMEQUALVERIFY` (`0x9d` — a constant this file does not even define, which is itself
evidence the case was never handled). A different leaf script is a different leaf hash, a
different taproot output key, and a different address. The arm's own comment states the stakes:
"*Appending instead produces a longer script that still parses and hashes to a different
address.*"

Why nothing caught it before: no shipped vector verify-wraps a `multi_a`. `keyed_tr_pathological`
has `and_v(v:older(65535),multi_a(2,…))` — the lock is wrapped, the `multi_a` is last and bare.
`keyed_compose_tr_extracted_later_four_paths` has the same three-leaf right spine and passes,
because its leaves end in `pk`, not `multi_a`.

Ownership: this is the fork's Go emitter disagreeing with the Rust primary, which is the oracle
here, so under the Rust-primary rule it is a Go-only convergence fix (add the
`opNUMEQUAL → opNUMEQUALVERIFY` arm) — **but the mandatory check is that the same defect is
looked for in the Rust primary first, and I did not do that.** It is out of this plan's scope
(the plan names "no address derivation for taproot script trees in `address/`" as explicitly NOT
in Stage 2), and the fix would land in `md/script_emit.go`'s pre-existing arm plus a new
positive test — neither of which the brief lets me author here.

### F-2 (Important, PLAN GAP) — the plan states the vendored corpus is not address-checked; a second glob address-checks it

Task 1's **Interfaces** says only "*the keyed conformance glob (`md/conformance_keyed_test.go:44`,
`keyed_*.conformance.json`) picks up the 22 keyed vectors with no code change*", and the
self-review says the composer's tr vectors are "*id-checked by the keyed conformance gate, not
address-checked*". There is a **second** glob over the same directory:

```go
// gui/policy_address_test.go:83
paths, err := filepath.Glob(filepath.Join("..", "md", "testdata", "vectors", "keyed_*.conformance.json"))
```

`TestEveryKeyedVectorReachesAnAddress` enrols every vendored keyed vector in a device-side
address gate against Rust's addresses. Vendoring alone (commit 79fa1de, before any builder code)
therefore turned it red — 7 sub-tests at that point:
`keyed_compose_tr_nums_three_leaves` on the wrong address above, and six pkh-bearing wsh vectors
reaching **no** address route at all ("*reaches NO address route — an operator sees "display
only"*"). Task 3's `pk_h` arm fixed those six; only F-1 remains at the tip.

This is worth calling a plan gap rather than an implementation problem: the plan reasoned about
one glob and there are two, so it predicted a green tree that could not be green.

### F-3 (Important, EXPECTED CONSEQUENCE, needs a decision the implementer may not take) — Task 3 closes a pinned gap, and two shipped tests are its tripwires

`gui/policy_address_test.go`'s `TestPkhTapLeafGapIsPinnedByShape` is a deliberate
"pin the gap, don't fail forever" test. Its own comment says it should fail exactly now:

> PINNED BY SHAPE: when the emitter grows `pk_h`, this FAILS saying the gap is closed rather
> than going quiet.

and its failure message prescribes the remedy: "*Convert this to a positive test, as its
predecessor was*". The derived address MATCHES Rust
(`bc1pl3kq0susghpu43eszll3ym5wjq9uva3enmwj49akc7v5rr68aw9qjz8aja`), so this is confirmation that
Task 3 is CORRECT, not a defect.

`gui/wallet_policy_test.go`'s `TestWalletPolicyConsentNeverHidesTheAbsenceOfAddresses` fails for
the same reason at one remove: it uses the `gap_tr_leaf_pkh` fixture as its example of "a keyed
policy this device cannot derive", and the device can now derive it. It needs a different
underivable fixture, or the assertion re-aimed.

Both fixes are edits to pre-existing tests, which the brief forbids me. Neither is a wrong
result; both are gates whose premise this stage retired. **Whoever folds this should decide
them deliberately** — F-1 must be fixed on its own merits regardless.

---

## Everything I decided, could not do, or stopped on

1. **Stopped at Task 9 Step 1** with the whole-tree gate RED — three pre-existing `gui/` tests
   (F-1, F-3). No pre-existing test was edited, no fix attempted. Task 9 Step 2's other four
   gates were run anyway (all green) so the record is complete.
2. **Ran `go test` in Go's default `-mod=readonly` from Task 7 onward** instead of the brief's
   `GOFLAGS=-mod=mod`, because `-mod=mod` silently rewrote `go.mod`'s indirect block (Task 7
   deviation above). Readonly is what CI uses. `go.mod`/`go.sum` are untouched on the branch.
3. **Moved `md/compose_test.go` aside briefly** to run Task 2 Step 2a's Expected check, which is
   unrunnable in the order the plan gives (Task 2 deviation above). File restored, unmodified.
4. **Staged `md/testdata_test.go` with Task 2** although the plan's `git add` line omits it.
5. **Measured, rather than trusted, three of the plan's own numbers**: the 1,503,652 B / 62,592 B
   firmware baseline (reproduced exactly at `169073c`), the 14 → 36 conformance sub-test growth,
   and the claim that dropping `AppendStubs`'s defensive copy makes the aliasing probe fail.
   All three held.
6. **Did not touch** `mnemonic-engrave`, `mnemonic-secret` or the main fork checkout beyond
   `git worktree add` and this report; nothing pushed, tagged, published or flashed.
7. **Left the three pre-existing `gofmt -l` findings in `gui/` alone** (baseline-identical, no CI
   gofmt job, outside the plan).
