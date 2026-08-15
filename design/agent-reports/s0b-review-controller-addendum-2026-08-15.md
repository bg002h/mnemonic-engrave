# Controller addendum to the S0b execution review — C-1 reaches further than `oracle/`

**Date:** 2026-08-15
**Author:** controller (not the review agent)
**Extends:** `s0b-execution-review-2026-08-15.md` finding **C-1**
**Fork HEAD when measured:** `4b8488e`

The S0b review was correctly scoped to the S0b diff (`oracle/`, `cmd/emu/`). That
scope is why it could not see the following, and the following is the reason C-1
is worse than it reads.

---

## C-3 (Critical) — S2's md1 byte-identity gate skips by the SAME mechanism, in `gui/`

`gui/multisig_build_oracle_test.go:45-47` (`s2OracleMD`), reached by
`TestAssembledMd1MatchesThePrimaryByteForByte` at `:68`.

C-1 found `oracle/expect_test.go`'s `resolveBins` calling `t.Skipf` when the
pinned Rust oracles are absent. The same construct exists independently in
`gui/`, guarding **S2's headline deliverable** — the gate the S2 record describes
as *"the md1 byte-identity gate (6 chunks byte-identical to the pinned primary,
oracle resolved by binary hash)"*.

```go
// gui/multisig_build_oracle_test.go:45-47
if _, err := os.Stat(bin); err != nil {
    t.Skipf("the md oracle is not installed at %s; skipping the byte-identity gate", bin)
}
```

Its own doc comment states the rationale — *"a contributor without the Rust
toolchain should not see a red suite they cannot fix"* — which is a real concern
and the wrong remedy, for the same reason C-1 gives.

### Why it is unconditional in CI

`.github/workflows/test.yml` uses `runs-on: ubuntu-latest` with
`actions/checkout` and `actions/setup-go` only. **There is no Rust toolchain
step**, so `~/.cargo/bin/md` cannot exist and the skip fires on every push and
every pull request.

### Measured, both directions

```
--- with oracle present (real HOME) ---
=== RUN   TestAssembledMd1MatchesThePrimaryByteForByte
--- PASS: TestAssembledMd1MatchesThePrimaryByteForByte (0.07s)
ok  	seedhammer.com/gui	0.081s

--- with oracle ABSENT (fake HOME, real GOPATH) ---
=== RUN   TestAssembledMd1MatchesThePrimaryByteForByte
    multisig_build_oracle_test.go:69: the md oracle is not installed at
    /tmp/tmp.DNNejqGChO/.cargo/bin/md; skipping the byte-identity gate
--- SKIP: TestAssembledMd1MatchesThePrimaryByteForByte (0.00s)
ok  	seedhammer.com/gui	0.002s
```

Both report `ok` and exit 0. Only `-v` distinguishes them.

### What it means for S2, stated precisely

S2's GREEN was **real where it was measured** — on a machine with the pinned
oracles installed, the comparison ran and passed, and the record quotes its
result. What is false is the implied claim that this property is **enforced**.
The required status check — the one branch protection gates merges on — has
never executed this comparison, and cannot. So:

- S2's byte-identity result is a **local observation, not an enforced gate**.
- Any future regression in md1 assembly merges green, because the only test that
  would catch it does not run on the machine that decides.
- This does not reopen S2's *code*. It reopens the *status of S2's gate*, and the
  repair belongs to the S0b fold, because the mechanism and the fix are C-1's.

This is the same lesson `.github/workflows/test.yml` already learned once and
wrote down a few lines below the failing step — *"A gate whose instrument does
not compile is not a gate"* — applied to a gate whose instrument is not
installed.

### Fix, coupled — do not land half of it

Making absence fail is only safe **together with** giving CI the oracles;
fail-closed alone turns the required check permanently red. Whatever shape the
repair takes, it must satisfy: *the byte-identity comparison executes on the
machine whose verdict gates a merge*. Both skip sites move together —
`oracle/expect_test.go` and `gui/multisig_build_oracle_test.go` — or the fold
leaves one door open.

---

## Method note

Found by grepping `t.Skip` across the whole test surface rather than the reviewed
diff. The scoping rule that makes reviews proportional is also what let this sit
in a neighbouring package: **a defect class found in one package is a query to
run tree-wide**, and running it costs one grep.
