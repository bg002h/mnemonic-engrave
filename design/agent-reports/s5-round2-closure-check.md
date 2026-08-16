# S5 round-2 fold closure check — commit 2b9a128

Mechanical closure check only. Not an audit, not adversarial review. All commands
run for real, on the toolchain reached via `nix develop --command`, with stdout
and stderr captured to separate files throughout.

Repo checked: `/scratch/code/shibboleth/wt-s5`, branch `s5-multislot`, HEAD at
start and end: `2b9a12805f6b1b71d71137354971d34394b2364a`.

---

## Q1. Is the commit genuinely test-only?

Command:
```
git show --stat 2b9a128
git show 2b9a128 -- gui/multisig_verify.go
```

`git show --stat` file list (verbatim):
```
 gui/multisig_verify_report_test.go | 44 ++++++++++++++++++++++++++++++++++++++
 1 file changed, 44 insertions(+)
```

`git show 2b9a128 -- gui/multisig_verify.go` produced **no diff hunks at all** —
only the commit message, then nothing (no `diff --git` line, no `---`/`+++`
headers). That confirms `gui/multisig_verify.go` carries zero changes in this
commit.

**Verdict: CONFIRMED test-only.** Exactly one file changed
(`gui/multisig_verify_report_test.go`), 44 insertions, 0 deletions, 0 non-test
files touched.

---

## Q2. Does the new row exist and pass?

Command:
```
nix develop --command go test ./gui/ -run 'TestVerifyRetriesAfterACorrectableFirstSeed' -count=1 -v
```

Exit code: **0**

Four subtests, all PASS:

1. `TestVerifyRetriesAfterACorrectableFirstSeed/the_first_seed_fills_no_slot` — PASS
2. `TestVerifyRetriesAfterACorrectableFirstSeed/the_first_seed's_hand-typed_ms1_is_rejected` — PASS
3. `TestVerifyRetriesAfterACorrectableFirstSeed/the_first_seed's_hand-typed_ms1_is_a_k-of-n_share,_not_the_secret` — PASS (the new row)
4. `TestVerifyRetriesAfterACorrectableFirstSeed/Back_at_the_first_seed's_ms1_entry_still_abandons` — PASS

`--- PASS: TestVerifyRetriesAfterACorrectableFirstSeed (0.24s)`, `ok
seedhammer.com/gui 0.244s`.

**Verdict: CONFIRMED.** New row exists, four subtests total, all green.

---

## Q3. Does the new row FAIL when the arm it pins is reverted — and for the right reason?

Made a `cp -a` copy of the worktree to a scratch directory (not the real
worktree). In the copy only, edited `gui/multisig_verify.go` at the arm whose
`showError` message is `"That isn't a valid ms1 secret share."` (line 1017),
changing the `return "", false, true` immediately after it (line 1018) to
`return "", false, false`. Confirmed via `git diff` in the copy that only that
one line changed and the sibling arm (`"That isn't an ms1 secret share."`) was
untouched.

Command:
```
nix develop --command go test ./gui/ -run 'TestVerifyRetriesAfterACorrectableFirstSeed' -count=1 -v
```

Exit code: **1**

Subtest results:
- `the_first_seed_fills_no_slot` — PASS
- `the_first_seed's_hand-typed_ms1_is_rejected` — PASS
- `the_first_seed's_hand-typed_ms1_is_a_k-of-n_share,_not_the_secret` — **FAIL** (only failure)
- `Back_at_the_first_seed's_ms1_entry_still_abandons` — PASS

Only the new k-of-n row failed. Failure message (verbatim):

```
multisig_verify_report_test.go:538: a first-seed exit returned 4, want 1.
    a share IS an ms1 object, so the flow gets past the object check and refuses the PAYLOAD -- and typing the unshared secret instead is a remedy the operator can carry out, exactly like the row above.
    Both engrave callers loop on verifyIncomplete and verifyFailed only, so anything else is a dead end: the operator reads a remedy and is handed the restore document, which is headed "If any of them is missing, this backup is incomplete"
```

This is exactly the expected shape: verdict **4** (verifyAbandoned) returned
where verdict **1** (verifyIncomplete) was wanted. No timeout, no panic, no
frame-budget exhaustion, no missing-string mismatch — a clean, single assertion
failure on the return-value comparison.

**Verdict: CONFIRMED.** The new row fails when its arm is reverted, and fails
for the right reason.

---

## Q4. Is the arm isolation real?

Made a second, independent `cp -a` copy. In this copy, reverted the *other*
arm instead — the `showError` at line 1012 with message `"That isn't an ms1
secret share."` — changing its `return "", false, true` to `return "", false,
false`. Confirmed via `git diff` that only that line changed (the `"valid
ms1"` arm at line 1017-1018 was untouched).

Command:
```
nix develop --command go test ./gui/ -run 'TestVerifyRetriesAfterACorrectableFirstSeed' -count=1 -v
```

Exit code: **1**

Subtest results:
- `the_first_seed_fills_no_slot` — PASS
- `the_first_seed's_hand-typed_ms1_is_rejected` — **FAIL** (only failure; the pre-existing row)
- `the_first_seed's_hand-typed_ms1_is_a_k-of-n_share,_not_the_secret` — PASS (the new row)
- `Back_at_the_first_seed's_ms1_entry_still_abandons` — PASS

Failure message for the pre-existing row (verbatim, first two lines):
```
multisig_verify_report_test.go:538: a first-seed exit returned 4, want 1.
    the screen names the wrong object, which is an input the operator can correct, and re-typing it is the whole remedy.
```

This is the mirror image of Q3: reverting the *other* arm fails the
pre-existing row and leaves the new k-of-n row green. The two table rows pin
two distinct code arms, not the same one twice.

**Verdict: CONFIRMED.** Arm isolation is real.

Both scratch copies were deleted after use.

---

## Q5. Does the full build gate still pass? (on the real worktree, HEAD = 2b9a128)

All five commands run with stdout and stderr captured to **separate** files.

### `go test ./... -count=1`
Exit code: **0**
`ok` lines (stdout): **51**
`FAIL` lines (stdout): **0**
stderr: empty.

Matches expectation (51 ok, 0 FAIL).

### `gofmt -l ./`
Exit code: **0**
stdout: **0 lines** (no files listed)
stderr: empty.

Matches expectation (0 files).

### `go vet ./...` (cold `GOCACHE`, via `export GOCACHE=$(mktemp -d)`)
Exit code: **1**
stderr line count: **40**
stdout: empty.
Checked for the nix "Git tree is dirty" warning inside the vet stderr capture:
none present (`grep -i "dirty\|warning:"` on the vet stderr file returned
nothing — that warning appears on `nix develop`'s own stderr wrapper, not
inside the vet invocation's captured stream in this run).
Checked findings against `_test.go`: `grep -v '_test\.go:' <vet-stderr>`
returned **zero lines** — all 40 findings are inside `_test.go` files.

Matches expectation (exit 1, 40 findings, 0 outside `_test.go`).

### `./scripts/oracle-live.sh`
Exit code: **0**
Output: "discovered 7 tagged test(s) from source"; 7 `=== RUN` blocks
(`TestLiveDerivationReproducesEveryCommittedExpectation`,
`TestRealPinsResolveTheInstalledOracles`,
`TestPinsAreCurrentWithTheirPrimaries`,
`TestBuiltPolicyDerivationMatchesTheS2Golden`,
`TestBuiltPolicyDerivesDivergentOrigins`,
`TestAssembledMd1MatchesThePrimaryByteForByte`,
`TestVendoredVectorsAreInSyncWithThePrimary`), all 7 reporting `--- PASS`.
Final line: `live checks: PASS (exit 0)`.

Matches expectation (PASS, 7 discovered / 7 ran).

### `./cmd/emu/build.sh`
Exit code: **0**
stdout: `built emu.wasm (9976131 bytes); serve this directory and open
index.html`
Confirmed on disk: `cmd/emu/emu.wasm`, 9976131 bytes, matches the commit
message's claimed size exactly.

Matches expectation (exit 0).

**Verdict: CONFIRMED.** All five gate commands pass with the measured numbers
matching the commit message's claims.

---

## Q6. Is the worktree clean when finished?

Command: `git status --porcelain` in `/scratch/code/shibboleth/wt-s5` —
**empty output** (the `emu.wasm` build artifact is gitignored and does not
appear).

`git rev-parse HEAD` — `2b9a12805f6b1b71d71137354971d34394b2364a`, unchanged
from the start of this check.

Both `cp -a` scratch copies (Q3, Q4) were deleted after use; the scratch
`copies/` directory is empty.

**Verdict: CONFIRMED.** Worktree is clean, HEAD unmoved, no scratch copies
left behind.

---

## Summary

| Q | Verdict |
|---|---|
| Q1 | test-only, confirmed: 1 file, `gui/multisig_verify_report_test.go`, 44 insertions/0 deletions, `gui/multisig_verify.go` has zero diff |
| Q2 | new row present, 4/4 subtests PASS, exit 0 |
| Q3 | reverting the pinned arm fails ONLY the new row, for the right reason (verdict 4 vs wanted 1), no other failure mode |
| Q4 | reverting the other arm fails ONLY the pre-existing row, new row stays green — arm isolation is real |
| Q5 | build gate matches commit-message claims exactly: test 0/51 ok/0 FAIL, gofmt 0/0 files, vet 1/40 test-only, oracle-live 0/7-of-7, emu build 0/9976131 bytes |
| Q6 | worktree clean, HEAD unmoved at 2b9a128, scratch copies deleted |

CLOSURE: CONFIRMED
