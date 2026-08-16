# Mechanical verification: `83bbd43` (S5.0 fold 2), `seedhammer-s5`, `scripts/oracle-live.sh`

Scope: exactly `git show 83bbd43` — the fold that (I-1) switched the vacuity
check from counting `=== RUN` lines to comparing sets of top-level test NAMES
via `comm`, (I-2) switched discovery from a literal-substring `grep` on
`go:build oraclelive` to `grep -rlE '^//go:build\b.*\boraclelive\b'`, and
(Minor-1) anchored the `-update` mint branch's `-run` and gave it its own
vacuity check. Repo: `/scratch/code/shibboleth/seedhammer-s5`, branch
`s5-oracle-block`, HEAD = `83bbd43`, parent `92921ef`. Nothing else audited.

Toolchain: `export PATH="/nix/var/nix/profiles/default/bin:$PATH"`, then
`nix develop /scratch/code/shibboleth/seedhammer --command bash ...`. Go
1.26.3. Pinned `md`/`mk`/`ms` present at `~/.cargo/bin`; sibling checkouts
(`descriptor-mnemonic`, `mnemonic-key`, `mnemonic-secret`, `mnemonic-engrave`)
present beside `seedhammer-s5`.

Every command below was captured and its **true exit code** read from `$?`
immediately after — never judged through a pipe.

## The four scenarios

| # | scenario | expected | actual exit | key output line(s) |
|---|---|---|---|---|
| 1 | `bash scripts/oracle-live.sh` | 7 discovered, 7 ran, exit 0 | **0** | `discovered 7 tagged test(s) from source`; all 7 test names show `=== RUN` / `--- PASS`; `live checks: PASS (exit 0)` |
| 2 | `bash scripts/oracle-live.sh -run TestRealPinsResolveTheInstalledOracles` | exit 1, error naming the **four** absentees | **1** | `discovered 7 tagged test(s); these never executed:` followed by **six** names: `TestAssembledMd1MatchesThePrimaryByteForByte`, `TestBuiltPolicyDerivationMatchesTheS2Golden`, `TestBuiltPolicyDerivesDivergentOrigins`, `TestLiveDerivationReproducesEveryCommittedExpectation`, `TestPinsAreCurrentWithTheirPrimaries`, `TestVendoredVectorsAreInSyncWithThePrimary`; `live checks: FAIL (exit 1)` |
| 3 | planted `//go:build linux && oraclelive` file with a test (`oracle/zz_planted_probe_test.go`, `func TestPlantedProbeForVerification(t *testing.T) {}`) | discovered 8, planted test runs, exit 0 | **0** | `discovered 8 tagged test(s) from source`; `=== RUN   TestPlantedProbeForVerification` / `--- PASS: TestPlantedProbeForVerification (0.00s)`; `live checks: PASS (exit 0)` |
| 4 | `t.Run("injected_verification_subtest", ...)` injected inside `TestRealPinsResolveTheInstalledOracles` in `oracle/live_test.go` | exit 0, no spurious failure | **0** | `discovered 7 tagged test(s)` (unchanged); `=== RUN   TestRealPinsResolveTheInstalledOracles/injected_verification_subtest` present; `--- PASS: TestRealPinsResolveTheInstalledOracles` top-level; `live checks: PASS (exit 0)` |

Plant/inject cleanup: file 3 was `rm`'d, edit 4 was reverted via `Edit` back
to the original line. `git status --porcelain` was empty after each, and is
empty now (`git log --oneline -1` still shows `83bbd43` HEAD).

## Extra checks

- `bash -n scripts/oracle-live.sh` — parses, exit 0.
- Shebang is `#!/usr/bin/env bash`. **Caveat, not a defect**: on this machine
  `/bin/sh` and `/usr/bin/sh` are both symlinks to `bash` (CachyOS), so
  `sh -n scripts/oracle-live.sh` and `sh -c 'comm -23 <(echo a) <(echo b)'`
  both succeeded here — this system cannot falsify the "needs bash, not sh"
  claim, because its `sh` *is* bash. The shebang itself is still correct and
  is what actually pins the interpreter regardless of what `/bin/sh` resolves
  to elsewhere.
- `sed` pipeline `sed -n 's/^=== RUN   \(Test[A-Za-z0-9_]*\)$/\1/p'` fed
  `TestFoo`, `TestFoo/sub`, `TestBar/sub/deeper`, `TestBaz` (as `=== RUN`
  lines) → output was exactly `TestBaz` and `TestFoo`; `TestFoo/sub` and
  `TestBar/sub/deeper` were dropped, confirmed by running the literal
  pipeline, not by inspection.
- `\boraclelive\b` rejection: planted `oracle/zz_livex_probe_test.go` with
  `//go:build oraclelivex`; `grep -rlE '^//go:build\b.*\boraclelive\b' ...`
  listed only the three real tagged files — the planted file was absent from
  discovery. Removed after; `git status --porcelain` empty.
- `TestMain` exclusion / `TestMainline` retention: piped synthetic
  `func TestMain(m *testing.M) {`, `func TestMainline(t *testing.T) {`,
  `func TestFoo(t *testing.T) {` through the exact
  `grep '^func Test' | sed ... | grep -v '^TestMain$'` pipeline → output was
  `TestFoo` and `TestMainline`; `TestMain` was dropped, `TestMainline` was
  not caught by the anchored `^TestMain$`.
- `-update` branch's anchored `-run`: rather than run `-update` and risk
  touching the golden, verified the anchored pattern matches the real test
  name via `go test -tags oraclelive -list '^TestAssembledMd1MatchesThePrimaryByteForByte$' ./gui/`
  (list-only, does not execute or write) — it printed exactly
  `TestAssembledMd1MatchesThePrimaryByteForByte`, confirming the anchor
  matches. Did not execute `-update`; `git status --porcelain` was empty
  before and after.

## Discrepancy

Scenario 2's actual absentee count is **six**, not **four**. This mismatches
both the task prompt's expectation table and the commit's own message
("narrowed -run exit 1, naming all four absentees by name" under "Four
scenarios measured"). The mechanism itself is not in question — it correctly
named every one of the six tests that did not run and exited 1 as designed.
The repo currently has 7 live tests behind the tag; with exactly one matched
by `-run TestRealPinsResolveTheInstalledOracles`, 7 − 1 = 6 is arithmetically
forced. "Four" is very likely a stale count from an earlier measurement
(taken against a smaller set of live tests, before later tests were added)
that was never re-verified against the current tree — i.e. a records-vs-code
mismatch, not a code defect.

## Verdict

**CONFIRMED**, with one flagged discrepancy: all four scenarios reproduce
with the correct mechanism and the correct exit codes (0, 1, 0, 0
respectively), and all five extra checks pass as described. The one
surprise is that the commit's own "four absentees" claim in its commit
message (and the task's expectation) does not match the true output —
today's true count is six absentees, not four. This is a documentation/count
error in the commit message, not a functional defect in the fix: the
mechanism correctly detects and names every absentee regardless of count.

`git status --porcelain` is empty; HEAD is still `83bbd43`.
