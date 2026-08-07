# Plan B Phase A — R0 round 2 (scoped re-review of the round-1 fold, verbatim)

- **Date:** 2026-08-07
- **Reviewer:** independent opus architect, read-only
- **Artifact:** `IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseA.md` @ `b1dbaf5`
- **Scope given:** two questions only — did the fold close round 1's findings,
  and did the fold introduce a new defect. Fresh-audit scope creep forbidden.
- **Round 1:** `REVIEW_planB_phaseA_R0_round1.md` @ `6d6d234` (3C, 6I, 6M, 1N)

## VERDICT: 0 Critical, 1 Important, 2 Minor, 1 Nit

**All 16 round-1 findings CLOSED.** The single Important is a contradiction the
FOLD authored inside its own new Task 5 — not a regression of anything round 1
filed. The two items the brief flagged as most likely to break (the reorder
tamper, the grouping key) both hold up against fork source.

Controller's independent reproduction of the Important, before folding —
`testing.AllocsPerRun(100, …)` over 8191 LF bytes, Go 1.26.3 in the fork's dev
shell:

```
sentinel CORRECT  allocs = 0        sentinel MUTANT  allocs = 1
struct   CORRECT  allocs = 1        fmt     CORRECT  allocs = 3
```

Confirms the collision exactly: only the preallocated-sentinel shape makes
`== 0` a true discriminator, and the struct-error shape puts a CORRECT
implementation at the same value the sentinel MUTANT produces.

---

## Round-1 closure

**Critical** — C1 `CLOSED` (Task 5 exists with every required element;
`container.rs:10-11` verified as `MAX_RECORDS`/`MAX_RECORD_LEN`). C2 `CLOSED`
(three named passes, case check first, both sections, mutation row;
`record.rs:64-70` verified). C3 `CLOSED` (exporter inside `mod tests`, command
changed, Global Constraint forbids widening the seam; `mod.rs:150` and `:122`
verified).

**Important** — I1 `CLOSED` (key is `(hrp, chunked, csid, uniq)`, byte-for-byte
the Rust semantics at `record.rs:113`/`:124-125`; `md/chunk.go:195`, `:66` and
`mk/mk.go:75-76` all mean what the plan claims). I2 `CLOSED` **and
checkable-verified** — `md.Reassemble` sorts by `ChunkIndex` before the gap check
(`md/chunk.go:257-268`) and `mk.reassemble` slots by `ChunkIndex`
(`mk/mk.go:186-201`, doc at `:146` "any order"), so a permutation of vector D's
5 public records still decodes and reaches the tag with a changed AAD. The
replacement negative is live, not dead. I3 `CLOSED` (both alternatives folded).
I4 `CLOSED` (injectable seam, call-count-0 assertion, row retargeted).
I5 `CLOSED` (index 2 of 6, empty-slice assertion, two rows; `gui/scan.go:57`,
`gui/gui.go:1672`, `platform_sh2.go:545` all correct). I6 `CLOSED`.

**Minors + Nit** — all `CLOSED`. M2's pinning table independently re-measured and
found correct (blob length asserted at `mod.rs:291,332,354,366,428,453`, not for
B; §6.6 hash only D/E at `pubhash.rs:69-80` and G at `mod.rs:414-418`).
M3's arithmetic checks out (52+8191+8191+16 = 16450).

**Task renumbering is clean**: all 20 `Task N` references resolve to the right
heading, and no reference points at a thing produced later.

---

## New findings

```
[Important] Task 5's two requirements collide: `AllocsPerRun == 0` is
            unreachable for any error that names the record count
Where:   Task 5, Step 1 — `TestOverlongSectionRejectsBeforeSplitting`; and the
         mutation row "split before counting separators"
Claim:   Task 5 requires (a) "The too-many-records error MUST be distinguishable
         from 'payload unreadable', naming the count and the cap" and (b) the
         test "must assert `testing.AllocsPerRun(...) == 0`". For the signature
         the task fixes — `SplitSection(b []byte) ([]string, error)` — no
         implementation satisfies both. Constructing an error that carries the
         count allocates, so a CORRECT pre-split scan reports 1 or 3, not 0. The
         implementer meets a red test on correct code, and the two exits are
         (i) drop the count from the error (violates (a) and §11.2's
         record-count-naming rule) or (ii) relax the threshold — which is
         precisely the false PASS the spec spends two paragraphs forbidding,
         because at `<= 1` or `<= 3` the split-first mutant passes.
Proof:   Measured, Go 1.26.3 in the fork's dev shell, 8191 LF bytes,
         `testing.AllocsPerRun(100, ...)`:
           preallocated sentinel      correct 0   mutant 1
           `&CountError{n, max}`      correct 1   mutant 2
           `fmt.Errorf(count, cap)`   correct 3   mutant 4
         The struct-error row is the sharp one: a correct implementation and the
         mutant BOTH sit at a value the other shape also produces, so only the
         sentinel shape makes `== 0` a true discriminator. SPEC :1470-1479 fixes
         the number at 0 ("The whole point of this assertion is the difference
         between 0 and 1"); :1487-1489 independently requires the
         record-count-naming error. container.rs:23-26 is the Rust message the
         plan is porting, and it is formatted.
Fix:     In Task 5 Step 1, split the two: make the too-many-records return a
         package-level preallocated sentinel (`ErrTooManyRecords`) and carry the
         count out of band — `SplitSection(b []byte) (recs []string, n int, err
         error)` — so the count-naming message is composed by the caller
         (Task 8), which is also where the cross-section total is known. Then
         state the measured numbers in the plan.
```

```
[Minor] `seal/read_host.go` carries no build constraint, so it also compiles
        under TinyGo alongside `read_tinygo.go`
Claim:  Go derives implicit build constraints only from `_GOOS`, `_GOARCH` and
        `_GOOS_GOARCH` filename suffixes; `_host` is none of these, so the file
        is unconditionally included and the firmware build fails on
        redeclaration. Host `go test` is unaffected, so Phase A stays green and
        the break surfaces only at Task 7 Step 4.
Fix:    Say `seal/read_host.go` carries `//go:build !tinygo`.
```

```
[Minor] The cross-section total record cap is required but no test binds it
Claim:  Task 5 correctly delegates `len(public)+len(secret) <= 24` to the caller,
        but Task 8 names no test for it and the mutation row points at Task 5's
        25-record test, which exercises only the single-section path. A payload
        of 20 public + 5 secret is 25 total and passes every named test with the
        caller check deleted.
Proof:  SPEC :640-644 makes the cap normative "across both sections together";
        container.rs:1-3 assigns it to the caller.
Fix:    Add one line to Task 8's negatives: 20 public / 5 secret is refused with
        the record-count-naming error; retarget the mutation row at both tests.
```

```
[Nit] Two fixture citations point inside the function rather than at it
Claim:  `mod.rs:256`/`:263` are cited as `bacon24()`/`bip84()`; the functions
        start at 253 and 262. Both land inside the intended fixture so the claim
        holds, but the cite gate cannot catch this class.
Fix:    `:253` and `:262`.
```

`VERDICT: 0 Critical, 1 Important, 2 Minor, 1 Nit`
