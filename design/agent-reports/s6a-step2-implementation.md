# S6a STEP 2 — implementation report

**Agent:** single implementer, step 2 of the §4.8 build order.
**Worktree:** `/scratch/code/shibboleth/wt-s6a`, branch `s6a-singlesig-truth`.
**Base:** `b8a23bf3dcf45f0b996bedf8b17f7141f092d282` (0 commits ahead, clean at start).
**Commit:** `c729176992f34f80de88a3d87b327fc09b14f0b9` — *"S6a step 2: the verify status line, as a pure function over a record"*.
**Not pushed. Not merged. Not rebased.**

---

## 0. BASELINE, measured before any edit

Run at `b8a23bf`, stdout and stderr to separate files:

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"
    cd /scratch/code/shibboleth/wt-s6a
    nix develop --command go test ./... -count=1

    EXIT=0
    stdout: 71 lines -- 51 `ok`, 20 `[no test files]`, 0 `FAIL`
    stderr: 0 bytes
    seedhammer.com/gui  ok  257.552s

**No pre-existing failure.** §6 of the plan asserts the same and it holds.

One pre-existing `go vet` finding, unrelated and inherited:

    nix develop --command go vet ./gui/
    EXIT=1
    stderr: gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)

`git diff --stat HEAD` was empty at the time (only two *untracked* new files), so
that file was never touched by this step. `.github/workflows/test.yml:63-67`
independently documents the same baseline — *"vet reports 40 pre-existing
findings in _test.go files across this tree (33 bezier.Point unkeyed literals, 7
testing.ArtifactDir)"* — and is why CI runs `go test`, not `go vet`, on this
tree. This is that baseline, scoped to one package.

---

## 1. FILES CHANGED

Two new files. **No existing file was modified** — step 2 has no callers by
design, so the blast radius is zero.

| file | lines | what |
| --- | --- | --- |
| `gui/verify_status.go` | 230 | `verifyStatus` + the four constants, `passRecord`, `verifyRecord`, `verifyStatusFor` (§4.7a's switch), `buildVerifyPassLine`, `buildVerifyStatusLine` |
| `gui/singlesig_truth_test.go` | 270 | T21, T22, T26 |

`gui/singlesig_truth_test.go` is the new file §5 names. `gui/verify_status.go` is
a new file the plan does not name a home for; it is shared by both single-sig and
multisig, so it is not filed under either.

---

## 2. THE FOUR STATUS LINES, EXACTLY AS EMITTED

**These were not transcribed from the source by hand.** A throwaway test in
package `gui` called `buildVerifyStatusLine` on the built package, printed each
result, and the output was captured and diffed against strings extracted from
§4.7c by `awk`. The throwaway test was then deleted (it is not in the commit).

    LINE 1  statusVerified, full mode (full=true, legs=1, suppliedCosigners=0)
    1 key plate was read back and matched what this run engraved. The ms1 secret you typed matched this seed.

    LINE 2  statusVerified, watch-only (full=false, legs=1, suppliedCosigners=0)
    1 key plate was read back and matched what this run engraved. No secret seed share was read back or compared.

    LINE 3  statusVerifiedOnRetry, full mode (pass + adverse)
    1 key plate was read back and matched what this run engraved. The ms1 secret you typed matched this seed. An earlier check did not pass; a later full check passed.

    LINE 4  statusVerifiedOnRetry, watch-only (pass + adverse)
    1 key plate was read back and matched what this run engraved. No secret seed share was read back or compared. An earlier check did not pass; a later full check passed.

    LINE 5  statusCheckDidNotPass (adverse, no pass)
    A verification check ran and did not pass: a comparison did not match, or a plate could not be read or accounted for. Do NOT rely on this backup until a full check passes. Check again with every plate this run engraved; if this repeats, engrave a fresh set.

    LINE 6  statusNotFullyChecked (the zero cell -- neither bit)
    These plates were not fully checked. Confirm they restore this wallet (master fingerprint below) before relying on this backup.

    LINE 7  statusVerified, multisig shape (full=true, legs=3, suppliedCosigners=2)
    3 key plates were read back and matched what this run engraved. The ms1 secret you typed matched this seed. Other cosigners' keys are taken as supplied.

    LINE 8  statusVerified, multisig shape (full=false, legs=2, suppliedCosigners=1)
    2 key plates were read back and matched what this run engraved. No secret seed share was read back or compared. Other cosigners' keys are taken as supplied.

### 2.1 Machine-check against §4.7c

Every string §4.7c states **verbatim** was extracted from the plan with `awk` +
`grep -o` and matched with `grep -F` against the emitted lines:

    MATCH   : An earlier check did not pass; a later full check passed.
    MATCH   : A verification check ran and did not pass: a comparison did not match, or a plate could not be read or accounted for. Do NOT rely on this backup until a full check passes. Check again with every plate this run engraved; if this repeats, engrave a fresh set.
    MATCH   : Other cosigners' keys are taken as supplied.
    MATCH   : These plates were not fully checked. Confirm they restore this wallet (master fingerprint below) before relying on this backup.

And the two **whole-line** literal cells were compared with `diff`, not `grep`:

    statusCheckDidNotPass: BYTE-EXACT
    statusNotFullyChecked: BYTE-EXACT

ASCII check on all eight emitted lines: `LC_ALL=C grep -c '[^ -~]'` → **0**. No
em dash, no smart quote, no ellipsis; every line draws in the body face
(F-78/F-151).

### 2.2 The pass lines are NOT in §4.7c, and this is FINDING 1 below

§4.7c gives two of the four cells as verbatim strings and the other two
**generatively**: *"generated from the pass record — names exactly the
comparisons this mode ran, states what was not read, and appends `Other
cosigners' keys are taken as supplied.` iff `rec.pass.suppliedCosigners > 0`"*.
So LINES 1–4 and 7–8 above are **authored at step 2**, against §4.7c's stated
obligations, not transcribed. The clause decomposition is:

| clause | entitled by | obligation it discharges |
| --- | --- | --- |
| `N key plate(s) was/were read back and matched what this run engraved.` | `passRecord.legs` | "names exactly the comparisons this mode ran" |
| `The ms1 secret you typed matched this seed.` | `passRecord.full == true` | same |
| `No secret seed share was read back or compared.` | `passRecord.full == false` | "states what was not read" |
| `Other cosigners' keys are taken as supplied.` | `passRecord.suppliedCosigners > 0` | §4.7c verbatim, §4.7b-seam's READ obligation |
| `An earlier check did not pass; a later full check passed.` | `verifyRecord.adverse` on the pass cell | §4.7c verbatim |

**A reviewer should read the pass-line wording as new text, not as a transcription
that can be diffed.** Everything else in §2 can be diffed.

---

## 3. TDD — RED, GREEN, MUTATION, per test

### 3.0 A note on what "red" is worth here

§5 says it plainly and it is honest to repeat it: *"A test of a function that
does not exist yet does not 'fail'; it does not COMPILE. T20, T21, T22 and T26
target `buildVerifyStatusLine`, `verifyRecord` and `verifyStatus`, which this
cycle introduces. 'Red' from a missing symbol proves nothing about the
assertion."*

The red phase is recorded below because it was run and it is the truth of what
happened, **but it is not the evidence.** The mutation checks in §3.4 are.

### 3.1 RED — all three tests, before `gui/verify_status.go` existed

    nix develop --command go test ./gui/ -run 'TestVerifyStatusZeroCellIsTheDefault|TestVerifyPassLineIsGeneratedPerMode|TestVerifyPassLineClausesAreEachBackedByARecord' -count=1
    EXIT=1

    --- stdout ---
    FAIL	seedhammer.com/gui [build failed]
    FAIL

    --- stderr ---
    # seedhammer.com/gui [seedhammer.com/gui.test]
    gui/singlesig_truth_test.go:58:17: undefined: verifyStatus
    gui/singlesig_truth_test.go:59:19: undefined: statusNotFullyChecked
    gui/singlesig_truth_test.go:67:12: undefined: verifyStatusFor
    gui/singlesig_truth_test.go:67:28: undefined: verifyRecord
    gui/singlesig_truth_test.go:67:52: undefined: statusNotFullyChecked
    gui/singlesig_truth_test.go:72:12: undefined: buildVerifyStatusLine
    gui/singlesig_truth_test.go:72:34: undefined: verifyRecord
    gui/singlesig_truth_test.go:79:11: undefined: passRecord
    gui/singlesig_truth_test.go:82:8: undefined: verifyRecord
    gui/singlesig_truth_test.go:83:8: undefined: verifyStatus
    gui/singlesig_truth_test.go:83:8: too many errors

This is the compile-red §5 warns proves nothing.

### 3.2 GREEN — after `gui/verify_status.go`

    nix develop --command go test ./gui/ -run '<the three>' -count=1 -v
    EXIT=0

    === RUN   TestVerifyStatusZeroCellIsTheDefault
    --- PASS: TestVerifyStatusZeroCellIsTheDefault (0.00s)
    === RUN   TestVerifyPassLineIsGeneratedPerMode
    --- PASS: TestVerifyPassLineIsGeneratedPerMode (0.00s)
    === RUN   TestVerifyPassLineClausesAreEachBackedByARecord
    --- PASS: TestVerifyPassLineClausesAreEachBackedByARecord (0.00s)
    PASS
    ok  	seedhammer.com/gui	0.037s
    stderr: empty

### 3.3 A DEFECT THE MUTATION CHECK FOUND, and review would not have

The **first** run of T21's mutation did not just fail — it **panicked**:

    --- FAIL: TestVerifyStatusZeroCellIsTheDefault (0.00s)
        singlesig_truth_test.go:68: an unwritten verifyRecord derived 3, want statusNotFullyChecked. ...
    panic: runtime error: invalid memory address or nil pointer dereference [recovered, repanicked]
    [signal SIGSEGV: segmentation violation code=0x1 addr=0x0 pc=0x728671]
        seedhammer.com/gui.buildVerifyStatusLine({0x0?, 0x1?})
            /scratch/code/shibboleth/wt-s6a/gui/verify_status.go:203 +0x91

`buildVerifyStatusLine` dereferenced `*rec.pass` on the two pass cells. Today
those cells are reachable only when `rec.pass != nil`, so the un-mutated code is
correct — **but the mutation is exactly the edit a future maintainer makes to
that switch**, and the failure mode it produced was a **SIGSEGV where the design
calls for the weakest line**. That is the opposite of "an omission can only move
the cell toward `statusNotFullyChecked`".

Fixed inline, with the guard stated as structural rather than incidental:

    if (status == statusVerified || status == statusVerifiedOnRetry) && rec.pass == nil {
        return verifyStatusNotFullyCheckedLine
    }

This makes P5(a) — *a claim with no record is not constructible* — hold at the
function boundary and not merely as a property of the switch above it. It is not
in the plan; it is a consequence of executing the plan's own named mutation.

**All three mutation checks in §3.4 were then re-run against the exact committed
source** (`sha256(gui/verify_status.go)[0:16] = 6bb31f020fdf9154`), and the
three tests were re-confirmed green against it (EXIT=0), so nothing below is
evidence about an earlier draft.

### 3.4 MUTATION CHECKS — the evidence

Each mutation is the one the test's own §5 row names. Each was applied by script
(`assert s.count(old)==1`, so the edit cannot silently miss), the grep confirming
the mutated line is in the file is shown, and each was reverted and `diff`-ed
back to the pre-mutation source afterwards (`CLEAN`).

**Proof the mutated line RAN, not merely that the edit landed:** in all three
cases the test's failure message quotes the *mutated output* — a value or a
string that only the mutated line can produce. A mutation that landed but never
executed cannot produce those bytes.

---

#### T21 — *the zero cell is the default*

Mutation (§5): *"make any other status the `default:` arm."*

    117:		return statusVerified // MUTATION T21: another status is the default arm

    EXIT=1
    --- FAIL: TestVerifyStatusZeroCellIsTheDefault (0.00s)
        singlesig_truth_test.go:68: an unwritten verifyRecord derived 3, want statusNotFullyChecked. The default arm is the zero cell; a path nobody classified must land there
        singlesig_truth_test.go:94: cell "no pass, no adverse (the zero cell)" derived 3, want 0
        singlesig_truth_test.go:97: cell "pass, no adverse" derived 3, which cell "no pass, no adverse (the zero cell)" already derived. Four cells must be four states, or the default arm has swallowed a named one
    FAIL
    FAIL	seedhammer.com/gui	0.002s
    stderr: empty

**Ran-proof:** `derived 3` is `statusVerified`'s ordinal. The only statement in
the program that can return `3` for `verifyRecord{}` is the mutated `default:`
arm.

**Verdict: T21 GENUINELY FAILS under its own mutation.**

---

#### T22 — *the pass line is generated per mode*

Mutation (§5): *"use a mode-blind literal."* The `if p.full { … } else { … }`
was replaced with an unconditional `verifyStatusMS1Clause`.

    179:	// MUTATION T22: mode-blind literal

    EXIT=1
    --- FAIL: TestVerifyPassLineIsGeneratedPerMode (0.00s)
        singlesig_truth_test.go:144: watch-only pass line
              got  "1 key plate was read back and matched what this run engraved. The ms1 secret you typed matched this seed."
              want "1 key plate was read back and matched what this run engraved. No secret seed share was read back or compared."
        singlesig_truth_test.go:153: the watch-only pass line claims an ms1 comparison. No ms1 was engraved, typed or compared on this run, and the pass record does not contain one:
              "1 key plate was read back and matched what this run engraved. The ms1 secret you typed matched this seed."
        singlesig_truth_test.go:158: both modes render the same pass line, so the line is mode-blind:
              "1 key plate was read back and matched what this run engraved. The ms1 secret you typed matched this seed."
    FAIL
    FAIL	seedhammer.com/gui	0.005s
    stderr: empty

**Ran-proof:** the ms1 clause appears in the `got` string for a record with
`full == false`. Only the mutated append can put it there.

**Verdict: T22 GENUINELY FAILS under its own mutation.** This is R9's C-1
reproduced and caught.

---

#### T26 — *every positive claim is named per mode (P6)*

Mutation (§5): *"add an unbacked clause to a pass line."* Appended `The
descriptor plate was read back and matched.` unconditionally — chosen because it
is **true of the code and unbacked by the record**, which is the hardest version
of this mutation, not a strawman.

    187:	// MUTATION T26: an unbacked clause -- no field of passRecord records this.

    EXIT=1
    --- FAIL: TestVerifyPassLineClausesAreEachBackedByARecord (0.00s)
        singlesig_truth_test.go:257: single-sig full pass line carries a clause no record entitles, or has lost one that is entitled
              got  "1 key plate was read back and matched what this run engraved. The ms1 secret you typed matched this seed. The descriptor plate was read back and matched."
              want "1 key plate was read back and matched what this run engraved. The ms1 secret you typed matched this seed."
              backed by [legs full]
        singlesig_truth_test.go:267: single-sig full retry line
              got  "... The descriptor plate was read back and matched. An earlier check did not pass; a later full check passed."
              want "... An earlier check did not pass; a later full check passed."
        singlesig_truth_test.go:257: single-sig watch-only pass line carries a clause no record entitles ...
        singlesig_truth_test.go:267: single-sig watch-only retry line ...
        singlesig_truth_test.go:257: multisig full, one leg, cosigners supplied pass line ...  backed by [legs full suppliedCosigners]
        singlesig_truth_test.go:267: multisig full, one leg, cosigners supplied retry line ...
        singlesig_truth_test.go:257: multisig watch-only, two legs, cosigners supplied pass line ...  backed by [legs full suppliedCosigners]
        singlesig_truth_test.go:267: multisig watch-only, two legs, cosigners supplied retry line ...
    FAIL
    FAIL	seedhammer.com/gui	0.007s
    stderr: empty

(Eight failures: four mode fixtures × {pass cell, retry cell}. The full text of
all eight was captured; the middle six are elided here only for length, and each
carries the same `got`/`want` shape with the added clause visible in `got`.)

**Ran-proof:** the added sentence is present in every `got`. And the `backed by`
list is the audit itself: `[legs full]` / `[legs full suppliedCosigners]` are the
`passRecord` fields naming the clauses that *are* entitled, so the failure says
in one line which clause has no name behind it.

**Verdict: T26 GENUINELY FAILS under its own mutation.**

**What T26 asserts beyond the whole-string compare**, so a reviewer can judge
whether it is really P6 and not a duplicate of T22:

- Every clause in its table names a `passRecord` field, and the field is
  confirmed to **exist by `reflect.TypeOf(passRecord{}).FieldByName`** — so a
  renamed or deleted field turns a "recorded observation" into a claim nothing
  records, and the test says so rather than a human noticing.
- A clause with an empty `backedBy` is an explicit failure.
- Each `entitled` closure reads **one** recorded field and nothing else —
  entitlement, never inference (§4.7g). None reads a verdict; there is no verdict
  in scope to read.
- Every clause is ASCII-checked against the body face's missing glyph set,
  mirroring `gui/multisig_build_prose_test.go:395`.
- The **retry cell** is asserted to be the pass line plus exactly one sentence,
  so an unbacked clause cannot hide in the arm the pass cell does not render.

---

## 4. FULL VALIDATION GATE — stdout and stderr separated throughout

`nix` prints `Git tree is dirty` on stderr, so nothing below was captured with
`2>&1` and no result is judged through a pipe.

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"
    cd /scratch/code/shibboleth/wt-s6a

| command | EXIT | stdout | stderr |
| --- | --- | --- | --- |
| `nix develop --command go build ./...` | **0** | 0 bytes | 0 bytes |
| `nix develop --command gofmt -l .` | **0** | 0 bytes | 0 bytes |
| `nix develop --command go test ./... -count=1` | **0** | 71 lines: **51 `ok`, 20 `[no test files]`, 0 `FAIL`** | **0 bytes** |
| `nix develop --command env CGO_ENABLED=0 go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/` | **0** | 3 × `ok … [no tests to run]` | 0 bytes |
| `nix develop --command ./scripts/test-32bit.sh` | **0** | `GOARCH=386 test: exit 0` / `GOARCH=arm build: exit 0` | 0 bytes |
| `nix develop --command ./cmd/emu/build.sh` | **0** | `built emu.wasm (9976125 bytes)…` | 0 bytes |
| `nix develop --command env GOOS=js GOARCH=wasm go vet ./cmd/emu/` | **0** | 0 bytes | 0 bytes |
| `nix develop --command tinygo build -size full -print-stacks -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller` | **0** | size/stack report | 0 bytes |

`seedhammer.com/gui  ok  188.265s`.

**Baseline-vs-final comparison, machine-made rather than eyeballed.** Both
`go test ./...` outputs were normalised (timings stripped), sorted and `diff`-ed:

    IDENTICAL package verdicts baseline vs final

**The whole suite is green, and nothing unrelated moved.** The only `go vet`
finding is the inherited `testing.ArtifactDir` one documented in §0 and in
`.github/workflows/test.yml` itself.

The last four rows are CI's gates (`.github/workflows/test.yml`) rather than
§6's, run because this change lands in `gui/`, which is in the emulator's *and*
the TinyGo device build's import graph — neither of which `go test ./...`
compiles.

---

## 5. THINGS I FOUND THAT CONTRADICT OR UNDER-SPECIFY THE PLAN

### FINDING 1 (design-level, needs a reviewer's eye) — §4.7c does not contain the pass lines it is the sole authority for

§4.7c says of itself: *"§4.7c IS THE SOLE AUTHORITY FOR WHAT THE BUILDER PRINTS,
and it must therefore carry every clause… Any future clause lands **here
first**."* But its `statusVerified` and `statusVerifiedOnRetry` rows are
**descriptions of a generation rule**, not text — only one clause (the cosigner
one) is verbatim there.

The consequence for this step is concrete: **half of what `buildVerifyStatusLine`
prints was authored here rather than transcribed**, so it cannot be reviewed by
diffing against the plan. §2.2 above lists every clause with the field that
entitles it, so a reviewer can at least audit the mapping.

This is not a request to change the design. It is the one place where a reviewer
must read new prose, and it should not be mistaken for text the plan already
blessed.

### FINDING 2 (documentation, harmless here) — §5's "two of six" paragraph is stale

`§5`'s *"NO SUBSTRING ASSERTIONS ON STATUS LINES"* block still describes the
pre-§4.7 **six**-line design, quotes lines that no longer exist
(`Plates VERIFIED: …`, `WARNING: … DISAGREED …`), and points at *"the §4.7d
table"* for the status strings — §4.7d is the membership-test table; §4.7c is the
strings table.

No harm done: the block's actual **rule** ("compare whole strings") is what
matters and it was followed. Every status assertion in
`gui/singlesig_truth_test.go` compares the entire string; the two `strings.Contains`
calls are used only to assert a clause is **absent**, or as a narrower second
statement about a line whose whole text is already pinned on the line above.
Flagged so a later fold does not resurrect the six-line wording from it.

### FINDING 3 (a real defect, in my own code, found by executing a plan-named mutation)

See §3.3. The nil-`pass` dereference. It is fixed and the fix is described in the
source at `gui/verify_status.go`. Recorded here because it is direct evidence for
the plan's own §5 preamble — *"a green suite proves nothing on its own"* — and
because it was **invisible to the green run and to reading the code**; only
running the mutation surfaced it.

### FINDING 4 (a judgement call the plan does not cover) — `legs == 0`

Nothing in §4.7 or the step-1 mapping says whether a pass record can carry
`legs == 0`. Today it cannot (single-sig writes the literal `1`; multisig writes
`len(legs)` at a site guarded by an empty-slot refusal). The prior art
`multisigVerifyOKMessage` keys its singular arm on `legs <= 1`, which would print
**"1 key plate"** for a zero count — a false claim (G1) if that state ever
becomes reachable.

I used `plateWord(p.legs, "key plate", "key plates")`, the existing helper, which
keys on `n == 1`. `legs == 0` therefore renders *"0 key plates were read back and
matched what this run engraved."* — truthful and weak, which is G2's direction.
**This is a deliberate divergence from the prior art's `<= 1`,** stated so it is
read as a choice.

### FINDING 5 (a clause deliberately NOT printed, and a reviewer may disagree)

**There is no descriptor-plate clause on the pass line, although the read-back
md1 IS compared** on both paths (`verifySingleSig` at
`gui/singlesig_verify.go:49`, and `bundle.Verify`'s md1 leg).

Reason: `passRecord` as specified by §4.7b-seam carries `full`, `legs`,
`suppliedCosigners` and nothing else, so **no recorded observation names a
descriptor comparison** — and P6 says *"a claim with no naming observation is
deleted."* Adding a field to say so would be authoring past §4.7b-seam's closed
struct and is exactly the NG1 increment §0.1 says to file rather than fold.

Filed here rather than fixed. If a reviewer decides the document should say the
descriptor plate was checked, that is a §4.7b-seam change (a new `passRecord`
field, written at both success returns), not a line-builder change.

### FINDING 6 (a small forward-compatibility fix for step 7's T25)

T25 (step 7) asserts *"the status derivation references neither `res` nor any
`verify*` constant."* My first draft's doc comment said *"Neither a
`multisigVerifyResult` nor any `verify*` constant appears in this function"* —
which is true, and which a source assertion searching for those names would
**match anyway, in the prose**. The comment was reworded to describe them without
naming them. `grep -n 'verifyComplete\|verifyIncomplete\|verifyFailed\|verifyAbandoned\|multisigVerifyResult'`
over both new files returns **nothing**.

Noted so step 7's implementer knows the file is already clean for that assertion.

---

## 6. WHAT I DELIBERATELY LEFT FOR A LATER STEP

Nothing here was blocked; all of it is out of step 2's scope, and doing any of it
now would make a later test pass vacuously — which the plan warns about by name.

| left undone | owning step |
| --- | --- |
| The `rec *verifyRecord` out-parameter on `singleSigVerifyFlow` / `multisigVerifyFlow`, and the `multisigVerifyFn` seam | 4–7 |
| The eleven single-sig record writes from `S6A_STEP1_EXIT_MAPPING.md` (2 adverse, 8 neither, 1 pass at the `:149` fall-through) | 7 |
| The multisig success write at `gui/multisig_verify.go:987`, and `countUncoveredPolicyKeys` (step 1's artifact (b)) — it belongs where `keys` and `covered` are in scope, and here it would be dead code | 7 |
| `restoreDocFlow` / `multisigRestoreDocFlow` gaining `status` + `extra`, and the three call sites | 4 |
| The twelve `multisigVerifyFlow` call sites, the four verbatim source assertions and the stub closure | 7 |
| **T20, T23, T24, T25, T27, T11, T7c** — each needs a rendered document through a production flow, or a multisig retry loop | 6–7 |
| §4.7f's scoping line under `statusCheckDidNotPass` — it is a **second** line on the page; `buildVerifyStatusLine` returns exactly one, per §4.7b-seam | 4 or 7, with T11 |
| `seedCapacity`, `buildSeedInventoryLines`, the census, the mode label, the abort gate | 3, 5 |

**I did not need a call site to test anything in scope.** T21, T22 and T26 are
all satisfiable as pure functions over a record, which is what §4.8 step 2 says
they are.

---

## 7. SELF-CHECK against the step-2 brief

| obligation | status |
| --- | --- |
| Work only in `/scratch/code/shibboleth/wt-s6a` | done — `/scratch/code/shibboleth/seedhammer` and `/scratch/code/shibboleth/mnemonic-engrave` untouched except this report |
| Baseline recorded before any change | §0 — EXIT=0, 51 ok, stderr 0 bytes |
| `verifyRecord`, `passRecord`, `buildVerifyStatusLine`, §4.7a's switch | done |
| §4.7c's four lines, verbatim where §4.7c is verbatim | done and machine-diffed — §2.1; the generated half is FINDING 1 |
| T21, T22, T26 only | done — no other test row written |
| No call sites, no flow changes, no record writes | done — `git show --stat` is two **new** files, zero modified |
| Tests first, red recorded | §3.1 (compile-red, and §3.0 says what it is worth) |
| Each test fails under its own named mutation, with the mutated line proven to have RUN | §3.4 — all three, each quoting mutated output |
| Whole suite green, nothing skipped, nothing papered over | §4 — EXIT=0, verdicts identical to baseline; the one inherited `vet` finding is named rather than hidden |
| Committed in the worktree, not pushed | `c729176992f34f80de88a3d87b327fc09b14f0b9`; `git status --short` clean |
| Types make step 1's success write expressible exactly as written | `passRecord{full: full, legs: 1, suppliedCosigners: 0}` compiles against these types as literally spelled in `S6A_STEP1_EXIT_MAPPING.md` §"The success write, in full" — field names, order and types all match |

**RESULT: DONE.** No blocking finding. Findings 1, 4 and 5 are judgement calls a
reviewer should look at before step 7 wires them to a document; none of them
blocks step 3.
