# Independent verification — round 3 (targeted), the FOLD (`composer-s4e` @ `177b4906`)

Reviewer: independent fold-verification agent (sonnet). Base `70008da`, prior
tip `818220d8991e084ab6c8a4a3a6c44ebc7ff310a7` (round 2's DO-NOT-MERGE), fold
`177b490679228f25142f020e9b67851dcedd0fe8`, worktree
`/scratch/code/shibboleth/wt-composer-s4e` (left clean, `git status --short`
empty throughout; nothing committed). Every mutation ran in `cp -r` copies
under `/scratch/code/shibboleth/.s4e-*`, all removed at the end of the run. No
`.jsonl` read, no sub-agent spawned.

**VERDICT: MERGE. 0 Critical / 0 Important / 1 Minor (new) / 0 Nit.**

`git diff 818220d8..177b4906 --stat`:

```
 gui/composer_backleg_test.go | 184 +++++++++++++++++++++++++++++++++++++++++++
 gui/composer_discard.go      |  51 +++++++++---
 gui/composer_shape.go        |  34 ++++----
 3 files changed, 244 insertions(+), 25 deletions(-)
```

Exactly the three files the brief named. No hunk outside the fold.

---

## I-2 — **FIXED**

`composerEditCanRenumber` now takes `field composerShapeField` and clears only
that field, leaving the other as the operator left it. Reproduced end to end
through production `composerPathEdit`, real screens, on `177b4906`:

- **DECLINE** (the shipped `TestComposerHashEditOnAKeylessPathAsksBeforeItDiscards`,
  wsh `[{2-of-2}, {key-less + hash}]`, both slots seated, `Path 2` → `Hash
  lock` → `No hash lock`): §8j is drawn (`"CLEARS THE KEYS"` frame present),
  and declining (`Button1`) leaves **both** seats and the hash intact —
  verified by re-running the shipped test (`--- PASS`).
- **ACCEPT** (my own walk, `TestVerifyR3_I2_AcceptDiscardsBothSeats`, same
  shape, hold-to-confirm via `press(Button3)` + `time.Sleep(confirmDelay)`):
  both seats are discarded — the safe direction, as the brief asked me to
  confirm. `--- PASS`.

## I-3 — **FIXED**

tr, `[{1 key + hash}, {1 key}]`, slot @0 seated, a lock edit on path 0 (the
hash-carrying path): no confirm fires (`pumpUntil(frame, "CLEARS THE KEYS",
16)` returns `asked=false`), and the lock editor is reachable **to
completion** — not merely drawn: my walk (`TestVerifyR3_I3_DeclineLeavesLockEditable`)
proceeds into `"What kind of time lock?"`, selects `None` (a no-op value),
and the screen returns control to the path list with the signature
unchanged. `--- PASS`. (I also reproduced round 2's own hand-built shape and
`hashlock-gated` non-reachability note; both behave as round 2 described,
now correctly.)

---

## 3. THE FALSE-PASS HUNT — the main item

**Structural independence.** `composerEditCanRenumber` is called exactly
once inside `TestComposerEditCanRenumberIsExactOverEveryReachableShape`, only
to produce `got` for comparison against an independently-computed `truth`.
The `truth` computation never calls it — it sweeps `lockValues =
[nil, olderBlocks(26280), afterHeight(1000000)]` / `hashValues = [nil,
digestA, digestB]` directly against `list.Paths[idx]`, comparing
`composerShapeSignature` to `now` (the list's own current signature, not a
cleared variant). This is a different algorithm from the probe's
clear-vs-set comparison, not a restatement of it.

**Decisive experiment — restore round 2's probe.** I copied the worktree to
`/scratch/code/shibboleth/.s4e-verify3` and replaced
`composerEditCanRenumber`'s body with round 2's exact logic (`bare` clears
**both** Lock and Hash; `held` sets only Lock), keeping the 3-arg signature
so the shipped test still compiles (`field` ignored via `_ = field`). Result:

```
=== RUN   TestComposerEditCanRenumberIsExactOverEveryReachableShape
    composer_backleg_test.go:665: FALSE NEGATIVE: ... path 2 ... (r2 I-2).
    composer_backleg_test.go:674: FALSE POSITIVE: ... path 0 ... (r2 I-3).
    composer_backleg_test.go:688: checked 3708 (list, path, field) cases: 156 false negatives, 288 false positives
--- FAIL: TestComposerEditCanRenumberIsExactOverEveryReachableShape (0.04s)
=== RUN   TestComposerHashEditOnAKeylessPathAsksBeforeItDiscards
--- FAIL: TestComposerHashEditOnAKeylessPathAsksBeforeItDiscards (0.01s)
```

**156 false negatives / 288 false positives — the 288 matches the fold
commit message's own quoted figure exactly**, and both new tests fail hard
on the broken probe. Re-running unmodified (`wt-composer-s4e`, `177b4906`)
gives `0 false negatives, 0 false positives`, `--- PASS` both tests. **This
is not a false PASS: the test provably distinguishes the fixed probe from
the one it replaces.**

**Corpus composition.** The `variants` slice (7 entries) includes a
key-less path (`{Hash: digestA}`, comment: `"key-less: the shape I-2 was
hiding in"`), two- and three-of paths (K2N2, K2N3+lock — multi-key), and
combinations that place a hash-carrying path ahead of a bare single under
tr (I-3's shape). Crossed with 2 wrappers × (1,2,3)-path lists, `checked =
3708 ≥ 1000`. **Shrink-guard fires correctly**: in a copy, cutting
`variants` to one entry drops `checked` to 24 and the test fails with `"the
enumeration collapsed to 24 cases ... a shrinking corpus is the finding"`
rather than silently passing.

**Value-set narrowing — checked, and it does NOT narrow the census.** The
oracle's `lockValues`/`hashValues` are a subset of what `composerLockEdit`
(4 lock kinds: None, OlderBlocks, OlderUnits, AfterHeight/AfterTime) and
`composerHashEdit` can actually produce. I read `md/compose.go`:
`isBareSingle()` (line 175) is `Keys != nil && Keys.N == 1 && Hash == nil &&
Lock == nil` — it and `composerShapeSignature` (which only records
`Keys.N` per path plus `md.Compose`'s slot mapping) never inspect a Lock's
`Kind`/`Value` or a Hash's bytes, only their **nilness**. `lowerTr` (line
617) picks the internal key by `isBareSingle()`, first match, purely
positional. So any non-nil Lock or Hash value is interchangeable for
signature purposes; testing 2 non-nil values per field is redundant
coverage, not a gap. Not a defect.

## 4. Call sites — **VERIFIED, each swap caught by its own named test**

`composer_shape.go:335` (lock arm) and `:342` (hash arm) each pass their own
`composerShapeField` constant. Swapping one at a time in copies:

```
lock arm -> composerFieldHash:  --- FAIL: TestComposerLockEditUnderTrDiscardsTheSeatsItMoves
  "§8j did not fire before a lock edit that CAN renumber under tr; the arm is
   unguarded because §7d's premise ... is false for this wrapper."
hash arm -> composerFieldLock:  --- FAIL: TestComposerHashEditOnAKeylessPathAsksBeforeItDiscards
  "§8j was not asked before a hash edit that can empty this path and discard
   every seat."
```

As the brief predicted, the census itself did **not** catch either swap (it
tests the function, not the wiring) — the walked tests did, each with its
own message naming the defect.

## 5. M-2 / M-3

**M-3 — FIXED, verified by recomputation.** `composerMoveUp` (line 467-474)
still calls `composerDiscardAssignments` unconditionally — it does **not**
call `composerApplyShapeEdit`, confirmed by grepping every real call site of
`composerApplyShapeEdit` (7 total: `composer_flow.go:165`, `composer_shape.go`
lines 324/338/345/352/419/424 — Keys, Lock, Hash, Remove, AddPath, wrapper
row, Back leg). I independently recomputed the cited signature: a
`ComposeWsh` list `[{K1N1},{K1N1}]` swapped gives `before=after=
"w1/1,1,|0.0/1.0/"`, byte-identical to the comment's claim.

**M-2 — PARTIAL, not FIXED.** The original contradiction (comment said
lock/hash edits "moves none ... and is not guarded"; the fold guards them
under tr) is closed. But the rewrite at `composer_shape.go:375-377`
introduces a **new** false claim in the same sentence:

> "THE DISCARD RULE HAS ONE PLACE TO LIVE, and it is `composerApplyShapeEdit`:
> composerPathEdit's Keys, Remove and **Move** arms, composerAddPath, the
> wrapper row, the Back leg's composerStartStep, and ... the Lock and Hash
> arms too."

This lists the **Move** arm as one of the sites where the rule "is
`composerApplyShapeEdit`." It is not: `composerMoveUp` deliberately bypasses
`composerApplyShapeEdit` (see M-3, above), which `composerMoveUp`'s own
comment says explicitly three functions later, at line 451: *"It does not go
through `composerApplyShapeEdit`, and that is the fix rather than an
inconsistency (review r0 I-1)."* Removing "Move" from the list at line 376
would make it exactly match the 7 real call sites I enumerated. This is a
mechanically verifiable, self-contradicting pair of comments in the same
file, introduced by this fold's own M-2 remedy — not present before it (the
pre-fold text used "Keys, Remove and Move arms plus composerAddPath" only to
name *which arms can move slot numbering*, a different and true claim; the
rewrite repurposed the same list under a new, false claim about the
mechanism).

Per the operative severity rule ("wording = Minor/Nit"; this changes no
runtime behavior — `composerMoveUp`'s discard is unconditional and correct
either way, re-verified above) this is **Minor**, not Important, and does
not block. Recorded as a one-line fix: drop "and Move" from
`composer_shape.go:376`.

## 6. Diff scope — **VERIFIED.** Three files only (stat above); confirmed
nothing outside `gui/composer_discard.go`, `gui/composer_shape.go`,
`gui/composer_backleg_test.go`.

## 7. Gates, as CI runs them — **VERIFIED, every one reproduced**

| gate | claim | measured |
| --- | --- | --- |
| `gofmt -l cmd/` | clean | clean |
| `gofmt -l gui/` | 3 pre-existing | `transaction.go`, `transaction_golden_test.go`, `transaction_txrecord_test.go` — same 3, none of the fold's files |
| `go vet ./gui/ ./cmd/...` | 2 pre-existing `ArtifactDir` lines | exactly those 2 (`freetext_sizeproof_golden_test.go:111`, `transaction_golden_test.go:104`) |
| `go test ./...` | 0 FAIL | 54 `ok`, 0 `FAIL`; `gui` 166.535s |
| sharded gui (24) | 1201 (1199+2 new) | `partition verified exhaustive: 1201 == 1201`; `RESULT: ok -- all 1201 tests ran across 24 shards`; wall 31s |
| `./scripts/test-32bit.sh` | exit 0 both | `GOARCH=386 test: exit 0`, `GOARCH=arm build: exit 0` |
| `go build ./cmd/...` | exit 0 | exit 0 |
| firmware (tinygo) | `+1,424 B flash / +0 RAM` over `70008da`'s 1,581,204/62,800 | measured **1,582,628 B flash / 62,800 B RAM** = **+1,424 / +0**, exact match |

**Capture drivers**, `EMU` pointed at the fold worktree's `cmd/emu` (the other
three drivers hard-code a path to the persistent `/scratch/code/shibboleth/
seedhammer` checkout, which is on base `70008da` — not the fold — so I ran
them from a scratch copy of `design/journeys/` with `EMU` patched to the
fold worktree, everything else unchanged):

```
RC_capture_composer=0          "all legs matched the host" (--arm both, key-less template id
                                e0863d3ccac31a64...)
RC_capture_walletpolicy=0      MATCHED against the host (id 4e67c6fd..., 4 addresses)
RC_capture_seating=0           MATCHED against the host (id c8fe87cd..., 4 addresses)
RC_capture_tr_pathological=0   MATCHED against the host (id 590f3abc..., 4 addresses)
```

No journey regression.

---

## Every mutation run

| # | mutation | result |
| --- | --- | --- |
| 1 | restore round 2's probe body (ignore `field`, clear both Lock+Hash, vary only Lock) in a copy | census: 156 FN / 288 FP (288 matches fold commit's own quoted count); both new tests FAIL |
| 2 | shrink `variants` to 1 entry in a copy | `checked=24`, guard fires: `t.Fatalf("the enumeration collapsed to 24 cases...")` |
| 3 | swap lock-arm call site to pass `composerFieldHash` | `TestComposerLockEditUnderTrDiscardsTheSeatsItMoves` FAILs, own message |
| 4 | swap hash-arm call site to pass `composerFieldLock` | `TestComposerHashEditOnAKeylessPathAsksBeforeItDiscards` FAILs, own message |

All four caught by a named test with its own message; unmutated worktree:
`TestComposerEditCanRenumberIsExactOverEveryReachableShape` and
`TestComposerHashEditOnAKeylessPathAsksBeforeItDiscards` both `PASS`, and the
full `go test ./...` is 0 FAIL.

---

## Counts

| severity | n | |
| --- | --- | --- |
| Critical | 0 | — |
| Important | 0 | — |
| Minor | 1 (new) | M-2 is only PARTIAL: fixed the original contradiction, introduced a new one — `composer_shape.go:375-377` lists the Move arm as routed through `composerApplyShapeEdit`, contradicted by `composerMoveUp`'s own comment (`:451`) and by the code (7 real call sites, Move not among them). No behavior change; wording only. |
| Nit | 0 | N-1 confirmed filed as `F-471` (`design/FOLLOWUPS.md:15605`), not fixed inline, as the commit message states. |

**I-2: FIXED. I-3: FIXED. M-3: FIXED.** The false-PASS hunt is closed: the
new census is structurally independent of the probe, fails hard (156/288) on
round 2's exact probe restored, passes 0/0 on the fix, its corpus reaches
thousands of cases across the shapes that mattered, its shrink-guard is
provably live, and both call sites are pinned by named tests that fail on
either field swap. Every CI gate reproduced byte-for-byte against the fold's
own claims, firmware delta matches to the byte, and all four capture drivers
exit 0 with a byte-exact air-gap match.

**MERGE.** The one new Minor (M-2 partial) is a one-line wording fix with no
runtime effect and does not gate.

*F-470/F-471 not re-opened. No secret-handling defect observed.*
