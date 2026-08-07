# R0 round 4 — `SPEC_encrypted_payload_delivery.md` (fold verification of round 3)

Reviewer: sonnet, scoped fold-verification pass.
Dispatched 2026-08-07.
Verdict: **0 Critical / 1 Important / 3 Minor / 0 Nit — GATE BLOCKED.**

## Fold table (round-3 findings)

| # | Fixed? | Note |
|---|---|---|
| C1 | YES | Canonical-record rule normative; SPACED-vs-CANON evidence independently reproduced byte-for-byte (SPACED len=80 `invalid character`; CANON len=67 ValidMD=true). Vector C recomputed from scratch — derived key, tag, header hex, plaintext sha256, blob sha256 all match. |
| I1 | YES | Cap 24; `bundleReviewFlow` paged-widget citation confirmed by reading `gui/bundle_flow.go:224`. Record counts re-measured: single-sig 6, 2-of-2 = 10, 2-of-3 = 15 — exact match. |
| I2 | YES | Wipe-on-every-exit assertion present and concretely specified; §11.3 row retargeted. |
| I3 | YES | §2.2a qualified; §2.2 item 9 added; §2.3 rule added; §12 item 8 promoted. One broken cross-ref introduced (Minor, below). |
| M1 | **PARTIAL** | Normative rule correct, but the named killer test cannot kill the mutant — see Important below. |
| M2 | YES | §9 `0x04` bullet added; `RefusedSecret` citations (`bundle.rs:104`, `:196`) confirmed; "pending" gone. |
| M3 | YES | `gui.go:2801 idleTimeout` citation confirmed; only the value left open. |
| N1 | YES | Row retargeted; duplicate case removed. |

## Findings

### [IMPORTANT] The 8191-LF-bytes test cannot kill the mutant it is mapped to
**Location:** §11.2 8191-LF-bytes bullet; §11.3 row "record count checked after splitting rather than before"
**Defect:** 8191 LF bytes splits into 8192 empty records under *either* implementation. A correct pre-split scan and a split-then-count mutant both reach `record_count > 24` and both reject, with the same reason. They are observationally identical to any return-value or error-message assertion; the only difference is a transient ~98 KB allocation, invisible to a host-run Go test.
**Failure scenario:** An implementation that splits first and validates afterward — exactly the defect M1 exists to prevent — passes the test, silently reintroducing the transient-allocation risk the fold claims to close.
**Fix:** Assert with `testing.AllocsPerRun` bounded to O(1) additional allocations, matching the "instrument it, don't trust the return value" pattern already used for the KDF-ordering bullets.

### [MINOR] Broken cross-reference: "§2.2 item 15" does not exist
**Location:** §2.2a. §2.2's list runs 1–9; the correct target is item 9, cited correctly in §2.3.
**Fix:** Change to "§2.2 item 9".

### [MINOR] "The last is the important one" now points at the wrong case
**Location:** §11.2 bundle-rejection case list. M1's fold appended the 8191-LF case to the end, so the trailing rationale — which describes the `lock-boot`-in-position-3 case — now attaches to the wrong item.
**Fix:** Reorder, or give each case its own inline rationale.

### [MINOR] §11.1 does not itemize a seal-time test for interior space/hyphen
**Location:** §11.1 canonical-encoding bullet. §9 requires `me seal` to refuse non-canonical records, and §6.4 makes it the central rule, but §11.1 names only trailing LF, CR and empty record. Defence-in-depth gap in host-side test naming; the device-side rejection is the real backstop.
**Fix:** Name the interior-space and hyphen cases explicitly, symmetric with the other three.

## VERDICT
Critical: 0   Important: 1   Minor: 3   Nit: 0
GATE: BLOCKED

CONFIDENCE: Verified by execution — vector C fully recomputed (all five values matched), all six canonical and all six space-grouped records classified against the real `seedhammer.com/codex32` package (temp files deleted, `git status` clean in both repos), multisig counts regenerated with the real CLI. Code citations confirmed by reading source. The Important and three Minor findings were found by close reading, not execution — they are documentation/test-design defects.

## Controller fold note (2026-08-07)

All four folded. The Important fix took option (a): §11.2 now requires
`testing.AllocsPerRun`-bounded assertion for the 8191-LF case, with the reason
stated inline ("a return-value assertion here is a guaranteed false PASS"), and
§11.3's row records the same. This is the third consecutive round in which the
finding was a test that could not fail rather than a defect in the construction.
