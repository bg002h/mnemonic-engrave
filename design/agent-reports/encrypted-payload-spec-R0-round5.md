# R0 round 5 — `SPEC_encrypted_payload_delivery.md` (close-out)

Reviewer: sonnet, tightly-scoped close-out of round 4's four fixes.
Dispatched 2026-08-07.
Verdict: **0 Critical / 0 Important / 1 Minor / 0 Nit — GATE PASS.**

This closes the R0 loop. Per project standard a re-review returning 0C/0I closes
the gate; no further rounds.

## Fold table

| # | Fixed? | Note |
|---|---|---|
| I1 | YES | §11.2 now states "rejection alone does not test anything here", names the pre-split-vs-split-then-count ambiguity, and requires `testing.AllocsPerRun` instead of a return-value check. §11.3's row matches. One residual imprecision — see Minor. |
| M1 | YES | §2.2a reads "§2.2 item 9"; all `§2.2 item N` refs across the doc (1, 3, 4, 9) resolve against the real 9-item list with matching content. |
| M2 | YES | The rejection-case list now ends with `command: lock-boot` in position 3 of 6, with its rationale attached; the 8191-LF case moved to its own bullet. |
| M3 | YES | §11.1 enumerates 7 seal-time refusals; matches §6.4 and §9 item-for-item. |

## Findings

### [MINOR] "O(1) additional allocations" is imprecise enough to admit a false PASS
**Location:** §11.2 pre-split bound scan bullet; §11.3 corresponding row
**Defect:** The spec required the assertion be "bounded to O(1) additional allocations" without a concrete threshold. `bytes.Split` — the natural split-then-count mutant — performs exactly **one** heap allocation regardless of N (a single `make([][]byte, n)`; the slices point into the existing buffer). A correct pre-split byte-counting scan performs **zero**. Both are literally O(1), so `allocs <= 2` satisfies the wording while passing the mutant.
**Failure scenario:** An implementer writes `if allocs := testing.AllocsPerRun(...); allocs > 2 { t.Fatal(...) }`, satisfying the letter of "O(1) bounded". A `bytes.Split`-based mutant shows `allocs == 1`, passes, and ships — reintroducing the exact defect the bullet exists to catch, with every visible check green.
**Fix:** State a concrete numeric bound instead of "O(1)". Note this does not change the direction of the fix — moving from return-value to allocation-count instrumentation is correct and real progress; it only tightens a threshold that round 4's own proposed fix text left unspecified and the fold carried forward verbatim.

## Verified clean

- §11.3's mutant→killer table (15 rows) checked row by row: **every named test
  exists** in §11.1, §11.2 or §11.4. No dangling rows, despite the table gaining
  rows in rounds 3 and 4.
- The three constraint lists (§6.4, §9, §11.1) agree item-for-item on trailing
  LF, CR, empty record, interior space, hyphen, the 24-record cap and the
  512-byte per-record cap.
- All `§2.2 item N` cross-references resolve.

## VERDICT
Critical: 0   Important: 0   Minor: 1   Nit: 0
GATE: **PASS**

## Controller fold note (2026-08-07)

The Minor was folded inline rather than deferred, since it is the same
false-PASS class the whole loop has been chasing and the fix is one paragraph.
§11.2 now states the bound is **0 additional allocations**, explicitly forbids
writing it as "O(1)", and records why: `bytes.Split` allocates exactly once and
a correct scan allocates zero, so the entire assertion turns on the difference
between 0 and 1. Being a wording fix to an already-passing gate, this does not
re-trigger a round.
