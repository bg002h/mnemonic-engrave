# Mutation testing — Plan B Phase A (device, headless core)

**Date:** 2026-08-07
**Plan:** `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseA.md` @ `59ee89d`
**Code under test:** `seedhammer` fork, branch `feat/encrypted-payload-phaseA`,
worktree `/scratch/code/shibboleth/sh-wt-seal`, package `seal/` @ `c31f303`
(plus `adafd25`, which is a fix for one of the findings below).
**Toolchain:** `nix develop /scratch/code/shibboleth/seedhammer`, host `go1.26.3`.

A green suite proves little. Every row below was applied to the real source,
run, and **watched failing** — or watched surviving.

## Procedure

Driven by a script (`mutate.py`, scratchpad) so the discipline is a command
rather than a memory. Per mutant:

1. `shutil.copy2(file, file.mutbak)` — **a file copy. Never `git checkout`**,
   which has destroyed uncommitted work in this project.
2. Exact-string substitution, with the occurrence count asserted `== 1`
   **before** anything runs. A silently-failing edit reads exactly like a
   surviving mutation, so the file is re-read from disk afterwards and compared
   byte-for-byte against the intended mutant.
3. Run the named killer with `-count=1`.
4. Restore **from the copy**, `os.utime` (touch) the file, and re-run the same
   test to confirm the rebuild is green — otherwise the "restored" run is still
   the mutant. No `RESTORE-FAILED` row was produced.

A mutant that fails to compile is reported as `INVALID-MUTANT`, not as
`KILLED`: a compiler error is not evidence about the test suite.

## Results — 21 rows, 21 killed

| # | Mutant | Named killer | Verdict |
| --- | --- | --- | --- |
| M1 | `DeriveKey` ignores `iterations` (hardcoded 100000) | vector B | **KILLED** |
| M2 | §6.2 bound checks moved after the KDF | `TestBadHeaderNeverReachesTheKDF` | **KILLED** |
| M3 | unsealed-shape zero checks removed | `TestRejectsNonZeroCryptoFieldsWhenUnsealed` | **KILLED** |
| M4 | `sealed` byte dropped from the hash | pubhash literals + the D≠E inequality | **KILLED** |
| M5 | hash over a subset of the section (`input[1:]`) | pubhash literals + `TestEveryByteOfTheSectionAffectsTheHash` | **KILLED** |
| M6 | `public_record_count` dropped from the hash | pubhash literals | **KILLED** |
| M7a | pipeline AAD = header only | `TestReorderedPublicSectionFailsAtTheTag` | **KILLED** |
| M7b | `crypto.Open` truncates the AAD to 52 bytes | *(see below — the plan's named killer does NOT kill it)* | **KILLED**, by a different test |
| M8 | split before counting separators | `TestOverlongSectionRejectsBeforeSplitting` | **KILLED** |
| M9 | single-section record-count cap dropped | `TestRejectsMoreThan24Records` | **KILLED** |
| M10 | cross-section total cap dropped | `TestTotalRecordCapSpansBothSections` | **KILLED** |
| M11 | CR tolerated instead of refused | `TestRejectsACarriageReturnAnywhere` | **KILLED** |
| M12 | lowercase check removed | `TestRefusesAnUppercaseRecord` | **KILLED** |
| M13 | allow-list → deny-list | `TestPublicSectionRefusesDebugCommand` | **KILLED** |
| M14 | allow-list applied to `records[0]` only | `TestPublicSectionRefusesDebugCommand` | **KILLED** |
| M15 | md1 grouping by HRP alone (csid dropped) | vector G | **KILLED** |
| M15b | mk1 grouping by HRP alone (csid dropped) | vector G | **KILLED** |
| M16 | non-chunked dispatch arm removed | `TestDecodesTwoDistinctNonChunkedCards` | **KILLED** |
| M17 | decode step removed entirely | `TestRefusesBCHValidButUndecodable` | **KILLED** |
| M18 | md1 93-symbol codeword cap removed (`codex32/mdmk.go`) | `TestRefusesAnOverlongRecord` | **KILLED** |
| M19 | `clampRegion` stops clamping | the clampRegion host test | **KILLED** |

The `RegionLen` total-size check in `ParseHeader` is **deliberately absent** from
this table, per the plan: it is unreachable behind the section caps
(52+8191+8191+16 = 16 450 < 65 536), Rust has no test for it either, and it is
kept only as defence in depth against a future implementation that drops the
caps.

## The three rows that needed more than one run

### M8 — the allocation table, re-measured

The plan carries a measured table and requires re-measurement if the error
shape changes. **The error shape was NOT changed** — `SplitSection` returns the
preallocated sentinel `ErrTooManyRecords` with the record count as a separate
return value, exactly as specified. Both columns were re-measured anyway,
8191 LF bytes, `testing.AllocsPerRun(100, ...)`, Go 1.26.3 in the fork's dev
shell:

| | allocs |
| --- | --- |
| correct implementation | **0** (`allocs=0 n=8192 err=seal: too many records`) |
| split-first mutant | **1** (`SplitSection allocated 1 times on the reject path, want 0`) |

Exactly +1, matching the plan. The first attempt at this mutant was reported
`INVALID-MUTANT`, because replacing the scan with `bytes.Split` also needs the
`bytes` import; the mutant was re-run as a two-edit substitution and then
compiled and was killed.

This is the row where a return-value assertion would be a guaranteed false
PASS: 8191 LF bytes yields 8192 empty records, and both the correct scan and
the mutant reach "too many records" with the same error. Only the allocation
count discriminates, and only against **0** — `bytes.Split` performs exactly one
allocation and a correct scan performs zero, so a threshold of `<= 2` passes the
mutant.

### M7b — the plan names a killer that does not kill

The plan's row *"AAD = header only, public section dropped"* names two killers:
Task 4's flipped-public-byte test **and** Task 8's reorder test. Measured, that
is true for the pipeline-level mutant (M7a) and **false** for the crypto-level
one (M7b).

`TestOpenFailsOnAFlippedPublicSectionByte` builds the AAD itself and asserts
`Open` returns `ErrAuthentication`. If `Open` truncates the AAD internally, the
tag never verifies — for the flipped blob **and** for the untouched one — so the
test still sees an authentication error and **PASSES**. It passes for a reason
unrelated to what it is testing.

Scoped to the whole package the mutant dies immediately, on the positive path:

```
--- FAIL: TestOpenRoundTripsEveryEncryptedVector/D  vector must decrypt: seal: wrong passphrase, or this payload has been altered
--- FAIL: TestOpenRoundTripsEveryEncryptedVector/G  vector must decrypt: ...
--- FAIL: TestOpenDrivesEveryVector
```

So the suite is sound and the defect cannot ship. What is worth carrying
forward is the general shape: **a negative test that asserts only "an error came
back" cannot distinguish a mutant that breaks everything from correct code.**
The plan already applies exactly this reasoning to the reorder test (which is
why it forbids a byte flip there); the same caveat belongs on the AAD row.

### M19b — a survivor, now fixed

Not in the plan's table. Asked of the *caller* rather than the helper: what if
`FileReader.Read` simply stops calling `clampRegion`?

```
buf = buf[:clampRegion(n)]   ->   buf = buf[:n]
```

**SURVIVED** the entire `TestFileReader*` set, including
`TestFileReaderNeverReturnsMoreThanTheRegion`, whose whole purpose is to catch
an unbounded read. The buffer was `make([]byte, RegionLen)`, so `io.ReadFull`
could not overfill it: the bound came from the allocation, not from the clamp,
and the assertion was true for the wrong reason.

Fixed in `adafd25` by sizing the buffer at `RegionLen+1`, which makes the clamp
the thing that bounds the result. Re-measured:

```
before: SURVIVED   ok  seedhammer.com/seal  0.003s
after:  KILLED     Read returned 65537 bytes from a 262355-byte region, want exactly 65536
```

Note this is a **different** mutant from the plan's row, which is "clampRegion
stops clamping" — that one was killed by the clampRegion unit test both before
and after the fix.

**Left standing, for a reviewer:** `read_tinygo.go` calls
`clampRegion(RegionLen)`, a compile-time constant, so on the target path the
clamp is structurally a no-op — the read window is bounded because it is a
literal, not because it is clamped. That matches the plan (the bound lives in an
untagged, host-testable helper and both implementations route through it), but
the clamp is documentation there rather than enforcement. It would start
enforcing the moment the length became a variable, which is the point of putting
it in the shared file.

## Suite state after every restore

```
go test ./seal/ -count=1 -v   64 top-level PASS, 100 PASS including subtests, 0 FAIL
go test ./... -count=1        47 ok, exactly one failure:
                              FAIL seedhammer.com/cmd/kdfbench [setup failed]
                              "package machine is not in std" — TinyGo-only,
                              already unbuildable under host go before this work
gofmt -l seal/                silent
go vet ./seal/...             clean
git status --short            clean
```

No `.mutbak` file survived the run, and `git status` confirms no source file was
left mutated.
