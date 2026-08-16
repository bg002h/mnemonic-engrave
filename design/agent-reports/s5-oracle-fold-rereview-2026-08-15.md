# S5.0 fold re-review — commit `92921ef` (`s5-oracle-block`, parent `5ed87c7`)

**Scope:** `git show 92921ef` only. Two questions: did the fold fix I-1 and I-2, and
did it introduce a new defect. No fresh audit of earlier S5.0 commits, the pin bump,
the ExpectKind, or any closed stage.

**Repo:** `/scratch/code/shibboleth/seedhammer-s5` · HEAD `92921efe9f38e2856eb6327964319bf3a0230176`

**Toolchain note (matters, and cost me a wrong first run):** the tree needs **Go 1.26**,
not the `go 1.25.10` in `go.mod`. `gui/freetext_sizeproof_golden_test.go:111` uses
`t.ArtifactDir`, a Go 1.26 `testing` API, so under go1.25.10 the `gui` package fails to
build and `./scripts/oracle-live.sh` exits 1 with `discovered 7 … but only 6 executed`.
All measurements below use `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`.

---

## VERDICT

| Severity | Count |
| --- | --- |
| Critical | 0 |
| **Important** | **2** |
| Minor | 2 |
| Nit | 3 |

Both Importants are in `scripts/oracle-live.sh`, both are in code the fold **added**,
and both were demonstrated to produce `live checks: PASS (exit 0)` with a live proof
that never executed — the exact defect class I-1 was filed about. The I-2 half of the
fold is sound; I found nothing against it.

---

## Q1 — did the fold fix I-1 and I-2?

### I-2 — **FIXED.** No findings.

The prefix bind at `oracle/expect.go:192-203` is correct, correctly placed, and proven
to be load-bearing.

- **Mutation M1** — neutered the check (`if false && !strings.HasPrefix(...)`) and ran
  `go test ./oracle/ -count=1 -run TestArtifactKindMustMatchItsBytes` → **exit 1**. The
  new test fails without the new code, so it is not a false PASS.
- Both of the reviewer's exact scenarios refuse on the real tree, naming the artifact:
  `built-policy-full` with an `"ms1"` holding an md1 string, and `built-policy-watch`
  with an `"mk1"` holding an ms1 string (`oracle/expect_test.go:1057`).
- The check does not reject honest work: `./scripts/oracle-live.sh` exits 0 with the
  live derivations flowing through `CheckArtifactShape` (`oracle/live_test.go:468,680,848`),
  and the committed-expectation sweep (`oracle/expect_test.go:202`) is green, so every
  committed artifact already satisfies `HasPrefix(String, Kind)`.
- Placement is right: `cmd/gaterecord/main.go` only ever sees **live-derived** artifacts,
  so the fabricated-record threat surface is the committed JSON, and that is exactly what
  `expect_test.go:202` sweeps with `CheckArtifactShape`.

### I-1 — **the demonstrated exploit is closed; the PROPERTY is not.**

The literal I-1 exploit no longer works. Renaming the gui live test to a fresh prefix
(`TestAssembledMd1MatchesThePrimaryByteForByte` → `TestMd1AssembledMatchesThePrimaryByteForByte`)
and running the script: `discovered 7 tagged test(s)`, the renamed test is discovered,
executed and PASSes, exit 0. Under the old hand-written allowlist that rename was the
silent-skip. **That is genuinely fixed.**

But the property the fold's own header now asserts —

> "A filter that silences a check, and a run that executes nothing, both fail loudly now."
> (`scripts/oracle-live.sh:71-72`)

— is **false**, via three doors, two of which I drove to a green verdict with a live
proof unexecuted. Those are Important-1 and Important-2 below. The third is Minor-1.

---

## FINDINGS

### **IMPORTANT-1** — `scripts/oracle-live.sh:130-131`

**The vacuity check compares a count of `=== RUN` LINES against a count of discovered
TEST NAMES. Those are different units, and a subtest's RUN line can pay for a top-level
test that never ran.**

```sh
ran=$(printf '%s\n' "$out" | grep -c '^=== RUN   Test')
if [ "$ran" -ne "$want" ]; then
```

`go test -v` prints `=== RUN   TestFoo/subtest` for subtests, and prints **no** RUN line
for `TestMain`. Three measured consequences:

**(a) FALSE GREEN — the one that matters.** Added one trivial `t.Run("probeSubtest", …)`
to `TestVendoredVectorsAreInSyncWithThePrimary`, then narrowed with a forwarded `-run`
that excludes only the gui byte-identity proof:

```
$ ./scripts/oracle-live.sh -run '^(TestLiveDerivation…|TestRealPins…|TestPinsAreCurrent…|
    TestBuiltPolicyDerivationMatchesTheS2Golden|TestBuiltPolicyDerivesDivergentOrigins|
    TestVendoredVectorsAreInSyncWithThePrimary)$'
EXIT=0
live checks: PASS (exit 0)
$ grep -c 'TestAssembledMd1MatchesThePrimaryByteForByte' <output>
0
```

6 top-level tests + 1 subtest = 7 RUN lines = `want` 7. **`TestAssembledMd1MatchesThePrimaryByteForByte`
— the md1-byte-identity-against-the-primary proof — did not run, appears nowhere in the
output, and the script reported PASS.** This is I-1 reproduced through I-1's own fix.

**(b) SPURIOUS RED on a normal edit.** With the subtest and *no* other change, all 7
tagged tests PASS and the script reports:

```
::error::discovered 7 tagged test(s) but only 8 executed --
live checks: FAIL (exit 1)
```

A fully green live run reported red. This is reachable by any future table-driven live
test — `t.Run` is used pervasively in this repo's untagged tests (13 subtests in
`TestCheckArtifactShapeRefusesTheWrongShape` alone). Whoever hits it will be tempted to
loosen the comparison, which is how (a) becomes reachable in the normal path.

**(c) SPURIOUS RED on `TestMain`.** Appended `func TestMain(m *testing.M) { os.Exit(m.Run()) }`
to the tagged sysw file → `discovered 8 tagged test(s) but only 7 executed`, and
`grep -c '=== RUN   TestMain'` = **0**. `TestMain` is counted by discovery and can never
be counted by the run.

**`-lt` is the wrong fix** (the brief floats it): under a subtest `ran` *inflates*, so
`-lt` would hide exactly case (a) as well as losing over-run detection.

**Minimal fix — compare the SETS of top-level names, not counts.** Validated (see below):

```sh
ran_names=$(printf '%s\n' "$out" | sed -n 's/^=== RUN   \(Test[A-Za-z0-9_]*\)$/\1/p' | sort -u)
missing=$(comm -23 <(printf '%s\n' "$tagged_tests") <(printf '%s\n' "$ran_names"))
if [ -n "$missing" ]; then
  echo "::error::discovered $want tagged test(s); these never executed:"
  printf '         %s\n' $missing
  rc=1
fi
```

The `$`-anchored `sed` drops `TestFoo/sub` (it has a `/`), so subtests stop counting.
Add `| grep -v '^TestMain$'` to the `tagged_tests` pipeline at line 114 for case (c).
Naming the missing test is strictly better than a count and costs nothing.

---

### **IMPORTANT-2** — `scripts/oracle-live.sh:109`

**The discovery grep matches the literal substring `go:build oraclelive`, so it cannot
see a tagged file whose constraint does not START with `oraclelive`. Such a file compiles
under the tag, is a real live test, is never discovered, is never run — and the script
says PASS.**

```sh
tagged_files=$(grep -rl 'go:build oraclelive' --include='*.go' ./oracle/ ./gui/ ./sysw/)
```

`//go:build linux && oraclelive` is the natural way to write a live test that needs a
Linux-only tool, and **`gofmt` does not normalise term order** (`gofmt -l` on such a file
returns empty), so nothing in the tree would ever flag it.

**Measured.** Created `oracle/zz_probe_e3_live_test.go`:

```go
//go:build linux && oraclelive

func TestProbeE3IsATaggedLiveTestTheDiscoveryGrepCannotSee(t *testing.T) {
	t.Fatal("PROBE E3: this test would have caught a defect, and it never ran")
}
```

- `grep -rl 'go:build oraclelive' … ` → lists **3** files, not 4. Invisible.
- `go vet -tags oraclelive ./oracle/` → exit 0. It compiles.
- `go test -tags oraclelive -list '.*' ./oracle/ | grep -c ProbeE3` → **1**. Go considers
  it a live test.
- `./scripts/oracle-live.sh` → **EXIT=0**, `discovered 7 tagged test(s)`,
  `live checks: PASS (exit 0)`, and `grep -c ProbeE3 <output>` = **0**.

A test that fails unconditionally never ran and the gate was green. This is a **new**
defect: pre-fold there was no discovery step at all, so this is a hole the fold created
while creating the mechanism.

**Minimal fix** — match the constraint, not a prefix:

```sh
tagged_files=$(grep -rlE '^//go:build\b.*\boraclelive\b' --include='*.go' ./oracle/ ./gui/ ./sysw/)
```

`\boraclelive\b` correctly rejects `oraclelivex`. Caveat worth a comment: it would also
match a negated `//go:build !oraclelive`, which does not exist today.

---

### MINOR-1 — `scripts/oracle-live.sh:94-99`, the `-update` branch

**The mint path still carries a hand-written, unanchored `-run` name and no vacuity check,
so it prints `mint: OK (exit 0)` having written nothing.** This is I-1's exact mechanism,
untouched, thirty lines above the fix, on the path the file's own header calls "the ONLY
code path that writes `gui/testdata/s2_md1_golden.expect.json`, which is what makes that
file trustworthy".

**Measured.** Renamed the gui live test with a fresh prefix (an unanchored `-run` still
matches a *suffix* rename, so that variant is not the hazard):

```
$ ./scripts/oracle-live.sh -update
testing: warning: no tests to run
PASS
ok  	seedhammer.com/gui	0.002s [no tests to run]

mint: OK (exit 0)
UPDATE_EXIT=0
```

`sha256sum gui/testdata/s2_md1_golden.expect.json` **unchanged** across the run.

Rated Minor rather than Important because the `-update` output is an *input* to the check
path: a stale golden makes the byte-identity test go RED, not green. The cost is an
operator who believes they re-minted, then reads that red as "the primary broke byte
identity" — a wrong and alarming diagnosis on the highest-stakes gate. Escalate to
Important if you weight that misdiagnosis higher than I did.

**Minimal fix:** anchor it and check it —
`-run '^TestAssembledMd1MatchesThePrimaryByteForByte$'`, then fail when the output
contains `no tests to run` (or reuse the set-diff against a one-element `tagged_tests`).

---

### MINOR-2 — `scripts/oracle-live.sh:109` vs `:124`

**Discovery is RECURSIVE (`grep -r`) over three roots; the run is NON-RECURSIVE
(`go test ./oracle/ ./gui/ ./sysw/`) over the same three. The two sets can differ.**

Measured: a tagged test at `oracle/zzsub/zz_live_test.go` →
`discovered 8 tagged test(s) but only 7 executed`, exit 1, and `grep -c ProbeE1` = 0.
Fail-loud, hence Minor. The mirror case is worse but pre-existing: a tagged test in any
package *outside* the three roots is invisible to discovery, to the run, and to CI's
compile check — the directory list is still a hand-maintained allowlist.

**Minimal fix — derive the run's package list from the discovered files**, which makes
the two sets equal by construction:

```sh
tagged_pkgs=$(printf '%s\n' "$tagged_files" | xargs -n1 dirname | sort -u)
out=$(CGO_ENABLED=0 go test -tags oraclelive -count=1 -v -run "^($filter)\$" $tagged_pkgs "$@" 2>&1)
```

---

### NIT-1 — `scripts/oracle-live.sh:114`
`grep -h '^func Test' $tagged_files` leaves `$tagged_files` unquoted, relying on word
splitting; a path containing a space or a glob character would break discovery silently.
No such path exists today. (Quoting is not the fix — an array or `xargs` is.)

### NIT-2 — `scripts/oracle-live.sh:124-126`
`out=$(… 2>&1)` buffers the entire run, so a live check now shows nothing until it
finishes, while the `-update` branch above still streams. Inconsistent, and it removes
progress feedback from the slow path.

### NIT-3 — `scripts/oracle-live.sh:74-76` (pre-existing, in the comment block the fold rewrote)
The header claims the tagged files are type-checked by
`go vet -tags oraclelive ./oracle/ ./gui/ ./sysw/`. The workflow actually runs
`CGO_ENABLED=0 go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/`
(`.github/workflows/test.yml:68`). Same effect, wrong command named; the same wrong claim
is repeated in `oracle/live_test.go:74`, `gui/multisig_build_oracle_live_test.go:25` and
`sysw/vendored_vectors_live_test.go:24`.

---

## THE FIVE SUSPICIONS, EACH SETTLED

### 1. Subtests inflate `ran` → **CONFIRMED** (Important-1)
Latent today (7 discovered / 7 ran; no tagged test has a subtest — `grep -n 't\.Run(' `
over the three tagged files returns nothing). Confirmed by adding one `t.Run` and running
the script: spurious FAIL at `discovered 7 … but only 8 executed` with every test green.
Worse than the brief supposed: combined with a forwarded `-run`, the extra RUN line pays
for a missing top-level test and the script reports **PASS** with the gui byte-identity
proof unexecuted. Also confirmed `TestMain` inflates `want` and never produces a RUN line.
Fix: set-difference on top-level names, plus a `TestMain` exclusion. **`-lt` is wrong** —
it hides the false-green case, not just the over-run case.

### 2. `HasPrefix(a.String, a.Kind)` with an empty `Kind` → **CLEAN**
Vacuous for `Kind: ""`, as suspected, but not reachable past the function: the sequence
loop requires every `Kind` to equal a member of `ArtifactKindsFor(k)`, and `""` never is.
Ran a scratch table of 8 hostile sets (temporary `oracle/zz_rereview_scratch_test.go`,
since deleted) — an empty-`Kind` artifact smuggling an `ms1` secret into a watch-only set
in **first, middle and last** position, all-empty `Kind`s at correct arity, empty `Kind`
in the ms1 slot of a full set, and `Kind:"ms1"` with `String:""`. **All 8 refused.** Seven
by the sequence check, one (the empty `String`) by the new prefix check.

No path consumes an artifact between the two checks — they are 1 line apart in the same
function, no early exit between them. On the wider question: `CheckFingerprintScope` and
`CompareCensus` **do** each accept an empty-`Kind` smuggler when called alone (measured),
but the committed-expectation surface runs all three independently over the same set
(`expect_test.go:115`, `:202`, `:275`), so `CheckArtifactShape` is never the skipped one,
and `cmd/gaterecord` only ever handles live-derived artifacts.

### 3. The discovery grep → **CONFIRMED** (Important-2 + Minor-2)
Enumerated the ways it can miss, and tested each rather than reasoning about it:
- **`//go:build linux && oraclelive` → MISSED, silently, with a PASS verdict.** Important-2.
- Tagged file in a subdirectory of a root → discovered but not run (false red). Minor-2.
- Tagged file outside the three roots → invisible to discovery, run and CI. Minor-2.
- `//go:build oraclelive && something` → found (substring matches). Fine.
- `func Test` not at column 0 → not reachable; `gofmt -l ./` is clean and gofmt puts
  top-level funcs at column 0.
- `TestMain` → found by discovery, never a RUN line. Rolled into Important-1(c).
- Run/discovery package-list drift → **yes**, and it is Minor-2.

### 4. `"$@"` after `-run` → **CLEAN** (no false-green door)
Ran the script once per flag and read the true exit code:

| forwarded flag | exit | RUN lines | verdict |
| --- | --- | --- | --- |
| `-json` | 1 | 0 | FAIL |
| `-list=.` | 1 | 0 | FAIL |
| `-count=0` | 1 | 0 | FAIL |
| `-count=2` | 1 | 14 | FAIL |
| `-run=TestRealPins…` | 1 | 1 | FAIL |
| `-short` | 0 | 7 | PASS |

Every flag that suppresses execution fails **loudly**; none defeats the count into a
green. `-short` is harmless: no tagged test calls `testing.Short()` and none calls
`t.Skip` (`grep -rn 't\.Skip\|testing\.Short()'` — the 8 hits are all untagged files, none
reachable from a tagged test). Also verified skip-masking directly: ran the script with
`HOME` pointed at an empty directory so the pinned oracles are absent → 7 RUN, **0 SKIP**,
exit 1. Absence is fatal, as the header claims, and the vacuity check does not mask it.
The only forwarded-flag issue is cosmetic false-RED (`-json`/`-list`/`-count=2`), a Nit
not worth a fix.

### 5. The fixture change in `arts()` → **CLEAN**, proven in both directions
`oracle/expect_test.go:506-522`. Because `String` is now *derived from* `Kind`, the prefix
check can never fire for an `arts()` fixture, so it cannot be silently carrying the
negative cases.

- **Mutation M1** (prefix check neutered): `TestCheckArtifactShapeRefusesTheWrongShape`
  still **PASSES**, all 13 subtests. So none of its 9 negative cases depends on the new
  check — they refuse for the same sequence reasons they always did.
- **Mutation M2** (`return nil` inserted before the sequence loop, prefix check intact):
  the test **FAILS**, and exactly the 7 sequence cases fail —
  `full, ms1 last` · `full, md1 before mk1` · `full, no ms1 at all` · `full, no md1 at all` ·
  `watch-only carrying an ms1` · `cosigner cards carrying an md1` · `full, mk1 after the md1s`.
  (`empty` and `unknown kind` still pass because they return before the loop.)

Making the fixtures realistic did not weaken what the test proves. It is still policing
the kind SEQUENCE, and only the sequence.

---

## VALIDATION OF THE RECOMMENDED FIXES

I applied Important-1 + Important-2 + Minor-2 to the script, measured all four scenarios,
then reverted. This is a machine-checked recommendation, not a suggestion:

| scenario | before the fix | after the fix |
| --- | --- | --- |
| pristine tree | `discovered 7`, PASS, exit 0 | `discovered 7`, PASS, exit 0 |
| `//go:build linux && oraclelive` probe | **PASS, probe never ran** | `discovered 8`, probe RUNs, FAIL |
| a subtest added | **FAIL: "only 8 executed"** | PASS, exit 0 |
| subtest + narrowed `-run` | **PASS, gui proof never ran** | FAIL, names `TestAssembledMd1MatchesThePrimaryByteForByte` |
| tagged test in a subdirectory | `discovered 8`, not run | discovered AND run |

---

## BASELINE — reproduced independently, unpiped, true exit codes

On the restored tree at `92921ef`, with go1.26.3:

```
go test ./... -count=1     exit 0    51 ok / 0 FAIL
gofmt -l ./                exit 0    0 files
go vet ./...               exit 1    40 findings, 0 outside _test.go
./scripts/oracle-live.sh   exit 0    discovered 7, ran 7
git status --porcelain     empty
git diff HEAD --stat       empty
```

Every number matches the fold commit's own claims exactly.

## READ-ONLY DISCIPLINE

Every mutation was reverted with `git checkout -- <path>` and every probe file removed.
One mutation had a side effect worth recording: `./scripts/oracle-live.sh -update` with a
*suffix*-renamed test still matched the unanchored `-run` and **rewrote**
`gui/testdata/s2_md1_golden.expect.json` (the sole diff was the `derived_at` timestamp,
`2026-08-15T19:27:27Z` → `2026-08-16T01:58:48Z`). Restored; the file's sha256 is back to
`c6d80c5da05e859dc0f65b732cfd1fd0241a86994b8409ce299264ddd483866d`.

Final state: `git status --porcelain` **empty**, `git diff HEAD --stat` **empty**,
HEAD still `92921ef`. `/scratch/code/shibboleth/seedhammer` was not touched.
