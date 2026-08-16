# S6a R6 — cheap pre-review verification pass

**Fold checked:** `git diff 3d6ceb6..HEAD -- design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
**Plan:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
**Code:** `/scratch/code/shibboleth/seedhammer` @ `main` = `b8a23bf`
**Answers to:** `design/agent-reports/s6a-r5-adversarial.md` (1C/4I), `design/agent-reports/s6a-blindspot-pass.md`

## VERDICT: DIRTY — 1 false, 3 stale, 4 table/switch disagreements

The new prose (§4.7d, the six-status table, P4) is well-formed and internally
sound. The defect is that **the fold changed the frame in prose only** — the
mechanical artifacts that a reader/implementer would actually build from
(the switch pseudocode, the status-type enum, the outcome→line table, the
promised P4 test fixture) were never touched, so the plan now asserts a
six-status design and ships five-status machinery underneath it. This is the
same "propagate to the whole file" failure class the brief exists to catch,
and it is severe: an implementer following §4.7a/§4.7c literally builds the
old five-status system, not the one §4.7d just specified.

---

## CODE CLAIMS

1. **`errVerifyLegHasNoPlate` returns before any comparison runs — TRUE.**
   `gui/multisig_verify.go:392-395`, inside `verifyMultisigLegsPartial`:
   `verifyClaimPlate` fails → `return nil, errVerifyLegHasNoPlate{Slot: l.Slot}`
   at line 394, **before** `verifyMultisig(...)` is called at line 397. Confirmed
   by direct read.

2. **Three shipped tests pin the retry-loop condition — TRUE, and exhaustive.**
   `grep -rn "res != verifyFailed\|== verifyFailed" --include="*_test.go" .`
   over the whole tree returns exactly three hits: `gui/multisig_verify_report_test.go:166`
   (`if res != verifyFailed`), `:759` (same), `:1093`
   (`strings.Contains(body, "res != verifyIncomplete && res != verifyFailed")`).
   `:1093` is confirmed to be a **source-text** assertion (`strings.Contains` on
   the file body), matching the plan's claim exactly. No other test in the tree
   references this condition — the "three, and only three" claim is CLEAN.

3. **The `:42-48` / `:717-724` "device cannot tell" claim — FALSE.** §4.7d
   (line 770-772) says: *"the sting is that the screens already speak this
   frame and only the document lacked it (`gui/multisig_verify.go:42-48`,
   `:717-724`)."* Read both ranges directly:
   - `:42-48` is the doc comment on `multisigVerifyForeignPolicyBody`: *"what
     actually happened is that the steel in their hands belongs to a different
     wallet"* — a **confident, single-world** diagnosis, the opposite of
     ambiguity framing.
   - `:717-724` is the `!slices.Equal(readbackMd1, engravedMd1)` check itself,
     preceded by a comment (`:703-716`) arguing the check is **decisive**:
     *"nothing further this flow does can produce a true answer about
     it"* — again confident, not ambiguous.
   The actual "device cannot tell" text — *"Either that plate was not
   presented, or it is not the one this run cut"* — lives at
   `gui/multisig_verify.go:427-429`, inside `multisigVerifyFailureText`'s
   `errVerifyLegHasNoPlate` arm, a location the plan does not cite here. This
   is the plan's stated motivating evidence for the whole reframe, and it
   points at the wrong lines. **Important** (motivational/rhetorical, not
   itself something an implementer executes, but it is the plan's own cited
   proof that "the codebase already had the idea").

## SUBSTRING MEASUREMENT

Extracted the five §4.7d table lines plus the new line quoted at line 795 and
tested programmatically:

| line | contains VERIFIED | contains DISAGREED |
| --- | --- | --- |
| `Plates VERIFIED: ...` | yes | no |
| `Plates VERIFIED on a repeat check...` | yes | yes |
| `Plates NOT VERIFIED. ...` | yes | no |
| `Plate verification DID NOT COMPLETE. ...` | no | no |
| `WARNING: ... DISAGREED ...` | no | yes |
| `Some plates could not be checked against this run...` (new) | no | no |

**3 of 6 contain `VERIFIED`, 2 of 6 contain `DISAGREED`** — matches the plan's
claim exactly. CLEAN. (Caveat, see below: this "six-line table" does not
actually exist anywhere in the document as a table — the count is correct only
because I assembled the sixth line myself from prose at line 795.)

## §4.7a — SWITCH vs TABLE, ROW BY ROW

Re-executed the printed switch (lines 738-743) against all twelve ENUMERATED
rows (908-934) by hand. **All twelve rows are correctly derived from the
switch as printed** — no arithmetic disagreement between the switch and the
table sitting under it. Row-by-row: `S`→NOT VERIFIED, `complete`→VERIFIED,
`incomplete`(stop)→DID NOT COMPLETE, `refused`/`abandoned`→DID NOT COMPLETE,
`incomplete→complete`→VERIFIED, `mismatch`(stop)→DISAGREED,
`mismatch→abandoned`→DISAGREED, `mismatch→incomplete`→DISAGREED,
`mismatch→complete`→repeat-check, `incomplete→mismatch→complete`→repeat-check,
`failed`(stop)→DID NOT COMPLETE, `failed→complete`→VERIFIED. Every one checks
out against `sawDisagreement`'s single-boolean semantics.

**But the switch itself was never updated for six statuses, so it and the
table under it are both stale together.** §4.7d (~30 lines above, same
subsection) declares: *"Mechanically: one more sticky fact —
`sawDisagreement` and `sawUnaccounted`, with explicit switch-arm order,
`DISAGREED` outranking `UNACCOUNTED`"* (line 797-798). The actual switch block
at §4.7a (lines 730-743) still has exactly the pre-fold code: two variables
(`status`, `sawDisagreement`), a four-arm switch, no `sawUnaccounted`, no
`UNACCOUNTED` case. `grep -n sawUnaccounted` over the whole file returns only
the one prose mention at line 797 — the variable is never defined. **This is
the highest-value finding**, and it is worse than a switch/table mismatch: an
implementer who builds §4.7a as printed does not implement status 6 at all.
**Critical.**

Three more artifacts in the same cluster, all promised by §4.7d's prose but
never built:

- **§4.7c's `verifyStatus` type (lines 991-1013) still declares exactly five
  constants** (`statusNotVerified`, `statusDidNotComplete`, `statusDisagreed`,
  `statusVerifiedOnRetry`, `statusVerified`) — no sixth constant for
  `PLATES UNACCOUNTED FOR`. Build-order step 1 and step 2 both depend on this
  type. **Critical.**
- **§4.7d's outcome→line table (lines 936-946) still has five rows**, not six
  — no `PLATES UNACCOUNTED FOR` row, despite T13a (line 1173) instructing
  implementers to assert "against the **six-line table in §4.7d**" and the
  substring-measurement prose (line 1192) saying "measured over the six
  lines." No six-row table exists anywhere in the document; T13a's fixture
  reference is to something that was never written. **Critical.**
- **T15 (line 1176) claims "the §4.7d **observation table**... one row per
  return path / error / comparand provenance, each naming its world-set W and
  its line" is the test fixture.** No such table exists anywhere in the
  document — `grep -n "world-set W\|comparand provenance"` finds only the two
  prose mentions of the *rule* (lines 833, 873) and the one enforcement-clause
  mention (line 903); none is a table. T15 is the test meant to enforce P4,
  the plan's central new property, and it cites a fixture that was never
  built. As specified, T15 cannot be implemented. **Critical.**

## STALE REFERENCES TO THE FIVE-STATUS / P3 DESIGN

- **T9 (line 1169): "each of the five §4.7c statuses renders its own line"**
  — stale; should be six now that status 6 exists. (Consistent with the fact
  that §4.7c's type block itself is also still five, so this isn't a
  standalone transcription error, but it is a claim a reader takes at face
  value.) **Important.**
- **"THE TWO PROPERTIES THIS MUST SATISFY" heading (line 855) vs. its own
  body.** The heading says two; the body directly under it enumerates **three**
  properties at identical formatting/weight — P1 (line 861), P2 (line 866),
  and P4 (line 872, replacing the removed P3). Nothing in the section marks P4
  as outside "the two" the heading counts. This is exactly the class the brief
  warned about — a heading asserting what the body no longer matches.
  **Important.**
- **Build-order step 7 (line 1090) lists T9, T10, T11, T12, T13a, T13b but
  omits T15 and T15b** — the two tests this very fold introduced for P4 and
  status 6. No other step mentions them either (`grep -n "T15\b\|T15b\b"`
  returns only their §5 table rows). Since steps 5+6+7 are the plan's own
  atomic-commit claim, and T15/T15b are unscheduled, nothing in §4.8 forces
  them to land in the same commit as the behavior they test — reopening
  exactly the gap the atomicity rule (§4.8) exists to close. **Important.**
- Checked and found **not** stale: `P3` appears only twice, both as "P4
  replaces/subsumes P3" — no orphaned "P3 forbids/requires" language survives.
  No "four-arm switch" or "a warning must be earned" phrase survives outside
  the deliberately-retained "old §4.7d" measurement subsection, which is
  explicitly labeled superseded. `verifyMismatch` is still used as the direct
  trigger for `sawDisagreement` in the (stale) switch — consistent with the
  switch's staleness already flagged above, not a separate defect. The
  five-site table (814-820) and "two of five"/"five sites loop" language
  (812, 834, 839) are the *old 4.7d measurement*, explicitly kept and labeled
  "SUPERSEDED AS A FIX, RETAINED AS A MEASUREMENT" — correctly scoped, not
  stale.

## THE NEW LINE

`Some plates could not be checked against this run. Either a plate was not
presented, or it is not one this run cut. Present every plate this run
engraved and check again; if this repeats, re-cut the set.`

Checked programmatically: **ASCII-clean** (no characters above ordinal 127),
**does not contain `VERIFIED`**, **does not contain `DISAGREED`**. CLEAN.

---

## Summary for the dispatcher

DIRTY. 1 false claim (the `:42-48`/`:717-724` "cannot tell" citation — the
real text is at `:427-429`). 3 stale references (T9's "five", the "TWO
PROPERTIES" heading vs. 3-property body, build order omitting T15/T15b). 4
Critical table/switch-class disagreements, all one cluster: the §4.7a switch,
the §4.7c type enum, and the §4.7d outcome-line table were never updated from
five statuses to six despite §4.7d's prose declaring the six-status design
immediately above them, and T15 cites a "§4.7d observation table" fixture
that does not exist. Substring measurement (3/6 VERIFIED, 2/6 DISAGREED) and
the new status-6 line (ASCII, no VERIFIED/DISAGREED substring) are both
CLEAN. No arithmetic disagreement was found between the switch as printed and
the twelve-row table under it — both are stale together, not stale against
each other.
