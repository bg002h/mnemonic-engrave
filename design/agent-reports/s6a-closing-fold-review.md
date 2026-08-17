# S6a CLOSING FOLD REVIEW
**Fold under review:** bd191dd (diff b2301d6..bd191dd) + plan fold 210afcb
**Prior report:** design/agent-reports/s6a-whole-diff-review.md (RED, 0C/1I)
**Branch:** s6a-singlesig-truth, vs main b8a23bf

## VERDICT: GREEN — 0 Critical, 0 Important (+ 3 already filed, unchanged: M-1, M-2-closed-below-is-now-covered-elsewhere-N/A, N-1)

I-1 is fixed, completely, with no surviving instance of the false phrasing
outside of prose that explains the correction. M-2's new test drives the real
functions through all four cells, is not tautological, and the controller's
own mutation run already showed it fails 3/4 subtests under the obvious wrong
implementation. The fold touches exactly two files and neither the eleven-exit
mapping, `suppliedCosigners`, the pass write, nor the scoping-line derivation
logic is in the diff. The deletion in I-1's remedy costs the single-sig reader
nothing measurable: the master fingerprint is the literal next line on that
document regardless of the parenthetical. No new finding.

---

## PART 1 — I-1 FIXED? AND THE INVARIANT vs THE OTHER LINES

**Fixed, and swept.** `[MECHANICAL]`

`grep -rn -i "fingerprint below" --include='*.go' .` inside `wt-s6a` returns
exactly **one** hit: `gui/verify_status.go:135`, which is inside the doc
comment on `verifyStatusNotFullyCheckedLine` explaining *why* the old
parenthetical was wrong ("An earlier draft said... FALSE of both multisig
documents... Whole-diff review I-1."). The shipped constant itself
(`:140-141`) reads `"...Confirm they restore this wallet before relying on
this backup."` — no parenthetical, no fingerprint reference.

`grep -n 't21ZeroCellLine' gui/singlesig_truth_test.go` shows the test-side
constant used at six sites (`:34, 78-79, 1320, 1634, 2166-2168`) — one
definition, no stale duplicate anywhere else asserting the old string.

Occurrences of the old phrase elsewhere in the repo are all in
`design/agent-reports/*.md` (step2/4/7 implementation logs and the whole-diff
review itself) and `design/IMPLEMENTATION_PLAN...md:979`'s own "an earlier
draft said" sentence — these are historical/explanatory prose about a defect
that existed, not shipped strings, and the plan's *authoritative* table row
(the one commit 210afcb changed) now carries the corrected line, verified
byte-exact against source in that same commit's message.

**The invariant tested against the other lines and the A–D clause table.**
The fold's own comment states the invariant: *"any artifact this line names
must exist on all three documents."* Checked every other rendered line in
`gui/verify_status.go` for a page-artifact reference:

- `verifyStatusDidNotPassLine` (`:147-150`) — names no artifact.
- `verifyStatusScopeLine` (`:175-176`) — "Everything **below**... the check
  **above**..." — these are positional, not content-specific: "above" is
  always the status line at index 0, "below" is always whatever
  `lines`/`extra` the calling flow supplies, and both slots are populated on
  all three flows unconditionally (`restoreDocScreen` is always called with
  `append(append(head, lines...), extra...)` in both `singlesig_restore.go:143`
  and `multisig_restore.go:114`). Not falsifiable the way "(master
  fingerprint below)" was, because it never names a specific field.
- Clause A (`:219-220`, plate count) — no artifact reference.
- Clause B / B2 (`:223/:225`) — no artifact reference.
- Clause C (`:228`, cosigner) — no artifact reference.
- `verifyStatusRetryClause` (`:153`) — no artifact reference.

I-1 was the only artifact-naming defect in this file; nothing else in the
clause table makes a claim about page layout, so the invariant holds
elsewhere by absence rather than by a second guard. `[MECHANICAL]`

## PART 2 — IS M-2's TEST HONEST?

`TestScopeLineRendersOnlyUnderCheckDidNotPass` (`gui/singlesig_truth_test.go`,
new, +45 lines) drives the real functions, not a hand-built string:

```go
status := buildVerifyStatusLine(tc.rec)
got := verifyStatusScopeLines(status)
```

Both calls are the shipped production functions — `buildVerifyStatusLine`
derives the status from a real `verifyRecord`, `verifyStatusScopeLines` is
the identity check under test. No status string is constructed by hand
anywhere in the test.

**All four cells are driven**, confirmed by constructing the four
`verifyRecord` fixtures and checking which `verifyStatus` cell each maps to
via `verifyStatusFor`'s own switch (`verify_status.go:108-119`):

| fixture | `pass` | `adverse` | cell | `want` scope |
| --- | --- | --- | --- | --- |
| `verifyRecord{}` | nil | false | zero (`statusNotFullyChecked`) | false |
| `full` | set | false | `statusVerified` | false |
| `retry` | set | true | `statusVerifiedOnRetry` | false |
| `adverse` | nil | true | `statusCheckDidNotPass` | true |

This is exactly the 2×2 the earlier review's M-2 finding named as
undriven for the zero cell — now covered.

**Not tautological.** `[MECHANICAL]`
- It could not pass if `verifyStatusScopeLines` were deleted: the file
  would fail to compile, which is a fail, not a vacuous pass.
- It could not pass if the guard were widened the way M-2 named
  (`status != ""` or `status == ""`): the controller's own mutation run
  (`if status != verifyStatusDidNotPassLine` → `if status == ""`) already
  measured **3 of 4 subtests FAIL**, each printing the wrongly-appearing
  scope line — reproduced from the brief's machine-verified table, not
  re-run here per the brief's instruction not to re-run settled checks.
- Every subtest branch (`tc.want` true and false) has an assertion
  (`t.Fatalf` on `len(got) != 1` or `len(got) != 0`); none of the four is a
  no-op case.

## PART 3 — REGRESSIONS AGAINST THE EARLIER CLEAN FINDINGS

`[MECHANICAL]` The fold's diffstat is:

```
 gui/singlesig_truth_test.go | 48 ++++++++++++++++++++++++++++++++++++++++++++-
 gui/verify_status.go        | 10 ++++++++--
```

Two files. In `gui/verify_status.go` the only change is the doc comment
block above `verifyStatusNotFullyCheckedLine` and that constant's string
literal — the function bodies (`verifyStatusFor`, `verifyStatusScopeLines`,
`buildVerifyPassLine`, `buildVerifyStatusLine`) are untouched, confirmed by
reading the full file: `verifyStatusScopeLines` at `:195-200` is byte-identical
to what the prior review quoted at its old line numbers (189-194).

None of the following files — where the earlier review's confirmed-clean
findings live — appear in this diff at all: `gui/singlesig_verify.go` (eleven
exits), `gui/multisig_verify.go` (`suppliedCosigners`, the pass write),
`gui/singlesig.go` / `gui/multisig.go` / `gui/multisig_build.go` (record
ordering, the three production call sites), `gui/singlesig_restore.go` /
`gui/multisig_restore.go` (scoping-line derivation call sites). A fold that
does not touch those files cannot have disturbed the invariants proven about
them.

## PART 4 — THE DELETION'S COST (judgement)

`[JUDGEMENT]` On the single-sig zero-cell document the status line renders at
slice index 0 with no scope line after it (zero cell never gets one — only
`statusCheckDidNotPass` does), and `singleSigRestoreLines` (`singlesig_restore.go:106-107`)
puts `"Master fp: %08x"` at the very next slice position. So on the one
document where the parenthetical was true, the fingerprint was never more
than one line away from the status regardless of whether the status names it
— the pointer was redundant with page layout, not the only route to the
fact. A stranger reading top to bottom hits the labelled fingerprint
immediately after the instruction to confirm the wallet, with or without the
word "below." I think the deletion was the right trade: adding a
document-shape parameter (a second constant or a threaded hint) to preserve
a one-word convenience on the single document, at the cost of a second
constant on a seam the code comments call load-bearing for staying single
(`verifyStatusScopeLines`'s own comment: "wired into two of the three and
forgotten on the third... is the exact shape that let this cycle's Critical
ship"), is the worse trade for a page a stranger uses to restore a wallet
years later. "Confirm they restore this wallet before relying on this
backup" remains actionable without the parenthetical: it tells the reader
what to do (attempt a restore) and against what (the wallet described below),
and the concrete artifact to check it against is still the immediately
following line on the one document where such an artifact exists.
