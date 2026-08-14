# Propagation check — fold `bdb6655..07e9dba`, IMPLEMENTATION_PLAN_multisig_build_repair.md

**Verdict: CLEAN.** Both corrections propagated to every site the prior fold-check
named, no new duplicate-claim defect found on a broader sweep, and the plan is
buildable S0→S6 as written.

Reviewer: independent context, 2026-08-13. Fork checked out at `a10d00795be44f2db5b9bf41f400f83b87c06d0a`
(matches brief). Both repos clean before and after (`git status --short` empty in
both `mnemonic-engrave` and `seedhammer`; no files left in scratchpad).

---

## 1. Consistency sweep

### Claim 1 — TYPED-ONLY count = 9, distributed `multisig.go×4, bip85.go×2, singlesig.go×2, multisig_build.go×1`, none in verify flows

Ground truth: `grep -rn "TYPED-ONLY" --include='*.go' .` in `seedhammer@a10d007` →
**9 hits**: `gui/multisig.go` (17, 24, 60, 102) ×4, `gui/bip85.go` (264, 270) ×2,
`gui/singlesig.go` (18, 32) ×2, `gui/multisig_build.go` (67) ×1. Confirmed
`seedEntryFlowTypedOnly` (the identifier) is what `multisig_verify.go` and
`singlesig_verify.go` call — zero hits of the literal phrase `TYPED-ONLY` in either.

| location | text | count stated | consistent? |
| --- | --- | --- | --- |
| line 26, stage table | "all 9 stale `TYPED-ONLY` comments die" | 9 | yes |
| line 315, S3 Implementation | "Delete or correct **all 9** `TYPED-ONLY` occurrences … `gui/multisig.go` ×4, `gui/bip85.go` ×2, `gui/singlesig.go` ×2, `gui/multisig_build.go` ×1 … none in the verify flows" | 9, full breakdown | yes, matches ground truth exactly |
| line 328, S3 Gate | "Measured: there are **9** occurrences across 4 files (`gui/multisig.go` ×4, `gui/bip85.go` ×2, `gui/singlesig.go` ×2, `gui/multisig_build.go` ×1) … none in the verify flows" | 9, same breakdown | yes |

Remaining occurrences of "four" (`grep -ni '\bfour\b'`): line 83 (unrelated —
"four host-side defects F-127/128/130/140"), line 316 and 322 ("not counted from
the spec's four cited sites" / "the spec cites four because four are the
*doc-comment* sites it analysed") — both are explicit historical attributions to
what the *spec* said, not a live count restated as current fact. No surviving
sentence implies 4 is the count to act on.

**Result: consistent everywhere. Implementer following the S3 Implementation
bullet word-for-word deletes/corrects all 9 sites; the Gate's `grep … returns 0`
is then satisfiable.** This is exactly what the prior fold-check's Critical
Finding 1 required and it is now fixed at all three sites it named.

### Claim 2 — BIP-382 → BIP-383 for `address_test.go` fixtures

| location | text | live citation or historical note? | consistent? |
| --- | --- | --- | --- |
| line 95, BIP table | `**383** — wsh(multi(…))/wsh(sortedmulti(…)) vectors — scriptPubKey` | live | correct, matches source (BIP-383 has `multi(`, 0 address strings, confirmed by prior fold-check §1.1) |
| lines 101-107, §1a correction paragraph | "previous version cited BIP-382 … that is BIP-383; 382 is `wsh()` alone and contains no `multi(`" | historical note, explicitly framed as a correction | correct — legitimately mentions 382 to record what was wrong |
| lines 179-183, S0 deliverable 3 | "replace them with **BIP-383** scriptPubKey vectors (compared at scriptPubKey, per §1a — **not** BIP-382, which publishes no addresses; an earlier draft said 382 here and the correction two sections above did not reach this line)" | historical note + corrected live instruction | **now fixed** — was the site the prior fold-check's Important Finding 2 flagged as still saying "replace … with BIP-382 vectors" |
| lines 210-212, S0 test list | `TestBip383WshMultiScriptPubKeyMatchesPublishedVectors` … "Not addresses: 383 does not publish them" | live | correct |

`grep -n "382"` → only lines 101-103 and 181-182, both historical/corrective
notes, never a live "do X with 382" instruction. `grep -n "383"` → lines 95, 102,
180, 210, 212, all consistent with the corrected relation.

**Result: consistent. Finding 2's dead branch is gone — S0 deliverable 3 no
longer asks for something impossible.**

### Generalised sweep for a third undiscovered propagation defect

Checked every candidate area named in the brief plus a full re-read of the
499-line file:

- **mk1 comparison relation** (`canonical_payload_bytes` vs `mk verify`) — stated
  once, in the §1a oracle table (line 75); referenced once more at line 451
  ("each mk1 must satisfy the two-part mk1 relation") with no conflicting
  restatement elsewhere. Consistent.
- **md 0.36→0.42 drift** — "ZERO byte drift across all 30 vectors" (line 63) is
  the only quantitative drift claim in the document; S0 deliverable 4 (line
  186-193) characterises the re-pin as "coverage catch-up, not a correctness
  repair" consistent with zero drift. No surviving sentence claims drift was
  found. Consistent.
- **S4-before-S5 ordering** — stage table (lines 27-28) lists S4 before S5;
  narrative ruling at lines 37-41 agrees and is the only ordering statement in
  the document. Consistent.
- **"ms1 presence" wording** — appears once (line 452), explicitly marked as
  superseded ("was this plan's earlier wording and it was a defect"); the S5
  gate's live requirement is byte-for-byte comparison (lines 448-451), matching
  test 5's mutation-check. No other passage still asserts mere presence
  suffices. Consistent.
- **BIP-32/BIP-48** — correction (lines 104-107, "BIP-48 publishes no vectors at
  all") appears once; remaining `m/48'` mentions elsewhere in the doc (S2, S4,
  S5) are path notation or the "BIP-48 account component" terminology, not
  claims that a published vector exists for it. No contradiction.

No third instance of a corrected-in-one-place / stale-in-another claim found.

---

## 2. Buildability answer

**Yes.** A competent implementer can start S0 today and work through S6 without
hitting a contradiction, an unbuildable gate, or a claim the plan elsewhere
refutes. Both defects the prior fold-check identified (Critical: S3's Gate vs.
its own Implementation bullet and the stage table; Important: S0 deliverable 3's
dead BIP-382 branch) are fully propagated in this fold, verified against the
fork's actual source tree, not just re-read from the plan's own prose.

## 3. Findings

None. 0 Critical, 0 Important, 0 Minor beyond what prior rounds already recorded
and scoped out (the M-2 mk-codec-pin note, already marked out-of-scope by the
inherited-fact audit, untouched by this fold, not re-raised here).
