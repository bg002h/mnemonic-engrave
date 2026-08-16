# S5 fold — RE-REVIEW round 2 — INCOMPLETE PROPAGATION sweep

**Artifact:** `git diff 830aaf7..s5-multislot` on `s5-multislot` (worktree
`/scratch/code/shibboleth/wt-s5`, HEAD `6088487`, tree clean, READ-ONLY). Three
commits: `da4fa98` (B1), `750296f` (B2, B3), `6088487` (B4, B5) — the fold of
round 1's five Important findings.

**Question asked of this round, and the only one:** did this fold leave any
stale reference behind — an old string, signature, or comment describing
behaviour the fold changed, anywhere in the tree (source, tests, `cmd/emu/*.js`
walks, `oracle/gaterecords/`, docs)?

**This is not a fresh audit.** Nothing under `830aaf7` was re-opened. B1..B5,
R-1, I-8, `gui/singlesig.go` (F-197/F-198), F-199, F-200, F-201, and the gate
record's already-verified no-re-mint are all treated as settled per the brief
and are not re-derived here.

---

# VERDICT: **GREEN — 0 Critical, 0 Important**

No stale reference survived any of the four greps below. Every changed string,
signature, and behavioural comment in this fold's diff was traced to its
consumers and every consumer agrees with the new form.

---

## Method and results

### (a) Old-form greps, each positive-controlled

For every string/signature this fold changed, grepped the whole tree for the
superseded form. Positive control run first to prove the pattern works.

```
$ grep -rn "type the remaining seed" .
gui/multisig_verify_report_test.go:220:  if uiContains(last, "type the remaining seed") {
```
The one hit is `TestVerifyIncompleteReportsWhatTheComparatorMatched` asserting
the banned phrase is **absent** from the drawn frame — an intentional negative
assertion pinning B2, not a stale reference. `TestVerifyIncompleteInstructionCanBeObeyed`
bans the same phrase a second, independent way (`strings.Contains` on the pure
string, `gui/multisig_verify_report_test.go:266`). No production or comment
site uses the old phrasing to describe current behaviour.

```
$ grep -rln "Verify the engraved plates" .          # positive control
cmd/emu/walk_s4_gate.js
cmd/emu/walk_trace_b.js
cmd/emu/walk_s3_nested.js
gui/multisig_verify_report_test.go
gui/bundle_flow.go
gui/multisig.go
gui/multisig_build.go
gui/multisig_supply_passphrase_test.go
gui/multisig_build_walk_test.go
gui/multisig_engrave_tail_walk_test.go
```
Confirms the grep methodology finds real hits against a string this fold did
**not** change (8 files). Against that control:

```
$ grep -rln "VERIFY AGAIN\|type the remaining seed\|Verify Incomplete\|No slot matches\|isn't an ms1" oracle/gaterecords/
(no output)
$ grep -rln "VERIFY AGAIN\|type the remaining seed\|Verify Incomplete\|No slot matches\|isn't an ms1" cmd/emu/*.js
(no output)
```
Zero hits in gate records and emulator walks for every screen string this fold
touched — this is absence, not a broken grep (the control above proves the
grep runs).

```
$ grep -rn "multisigVerifyFlow(" --include="*.go" .
```
All 9 hits are in `_test.go` files (calling the flow directly, which is
correct — those tests exercise the function itself, not the dispatch seam) plus
the one declaration line. **Zero** production call sites still call
`multisigVerifyFlow` directly; both (`gui/multisig.go:336`,
`gui/multisig_build.go:452`) go through `multisigVerifyFn`, confirmed by
`git diff` above.

```
$ grep -rn "s, ok := multisigVerifyMS1Entry\|, ok := multisigVerifyMS1Entry" .
(no output)
```
No caller still destructures the old two-return signature. The one production
call site (`gui/multisig_verify.go:868`) uses `s, ok, rejected :=`, matching the
new `(canonical string, ok, rejected bool)` signature at `:1004`.

### (b) `strings.Contains`-over-source tests: does the needle still exist

Every source-grep (`funcBody`) test touched by or adjacent to this fold was
checked against the current source:

| test | needle | present at |
| --- | --- | --- |
| `TestBothEngraveFlowsReOfferTheVerify` (`gui/multisig_verify_report_test.go:1035,1037`) | `multisigVerifyFn(ctx, th, full, engravedSlots, suppliedMd1/engraveMd1)` | `gui/multisig.go:336`, `gui/multisig_build.go:452` |
| `TestBuildPassesTheTailsSlotsToTheVerify` (`gui/multisig_verify_flow_test.go:373`) | `multisigVerifyFn(ctx, th, full, engravedSlots, engraveMd1)` | `gui/multisig_build.go:452` |
| `TestSupplyPassesTheEngravedPolicyToTheVerify` (`gui/multisig_verify_flow_test.go:394`) | `multisigVerifyFn(ctx, th, full, engravedSlots, suppliedMd1)` | `gui/multisig.go:336` |
| `TestBothEngraveFlowsGateOnACompletedSet` (`gui/multisig_verify_report_test.go:896,898`, unchanged by this fold) | `bundleEngrave(ctx, th, "Engrave Multisig"/"Build Policy", cardsOut) != bundleEngraveDone` | `gui/multisig.go`, `gui/multisig_build.go:402` — untouched by this fold, confirmed still present |

All four still name a string that exists. The three that this fold edited
(row 1–3) were updated in the same commit (`6088487`) that introduced
`multisigVerifyFn` — this is the fold correctly propagating its own rename into
every needle that referenced the old call, including the two
(`TestBuildPassesTheTailsSlotsToTheVerify`,
`TestSupplyPassesTheEngravedPolicyToTheVerify`) that round 1's own report did
not name (the fold's own log records finding these by grepping the superseded
phrasing after landing the seam; re-verified here independently and confirmed
correct).

### (c) Comments asserting an invariant the fold changed

Checked every comment touching code this fold modified:

- `gui/multisig_build_census.go:143-148` (`len(seeds) < 2` early return) — the
  comment "ONE SEED: the shipped two lines, unchanged" is accurate post-B1: the
  parameter is FACT count (post-merge), and B1's merge is exactly what makes
  `len(seeds)==1` correctly represent one secret regardless of how many slots
  it was entered at. Verified by reading `TestRestoreDocMergesOneSeedHeldAtTwoSlots`
  and `TestRestoreDocNamesEveryPassphrasedSeed`'s B1 row, which both exercise
  this arm and pass.
- `gui/multisig.go:304-309`, `gui/multisig_build.go:411-415` — prose describing
  `multisigVerifyFlow`'s behaviour ("multisigVerifyFlow has another caller",
  "multisigVerifyFlow reads back EVERY engraved key plate"). These describe the
  algorithm the seam still dispatches to in production (`var multisigVerifyFn =
  multisigVerifyFlow`, never reassigned outside tests) — still true, not stale.
- `gui/multisig_verify.go:990` (`multisigVerifyMS1Entry`'s one-line summary,
  "hand-types ONE ms1 and returns its canonical string") is terse about the new
  third return value, but the paragraph immediately below it (`:997-1003`)
  documents `rejected` in full. Not a stale assertion — a Nit at most, not
  reported as a finding (does not misdescribe current behaviour, just
  under-summarizes it in the one-liner).
- `gui/multisig_verify.go:848-865` (the `correctable = true` unconditional set
  after the no-slot/covered-seed switch) — comment claims "ALL THREE ARMS
  PRESCRIBE A REMEDY". Traced both arms: `multisigVerifyNoSlotBody` (`:151`)
  always ends with a remedy sentence; `multisigVerifyCoveredSeedBody` (`:516`)
  always ends with either "Try again and skip the passphrase" or a statement
  that the outstanding plates are from different words/passphrase — read as
  informational rather than an imperative in the non-bareWordsMatch arm, but it
  still names an actionable next step (type a different seed) rather than
  dead-ending, so `correctable=true` is directionally safe here: it only ever
  *adds* a retry offer, never *removes* one, so a borderline classification
  costs an extra "try again" screen, not a lost verification. No production
  code path is left in a state neither the comment nor the tests describe.

### (d) Walk files and gate records vs. every altered screen string

Already covered under (a) — zero hits for the five changed screen substrings
across `oracle/gaterecords/` and `cmd/emu/*.js`, positive-controlled against an
unchanged string that legitimately hits 3 walk files. `S5-trace-b.walk.json`'s
`restoreDoc` field takes the pre-existing "No BIP-39 passphrase was used" arm
regardless of B1 (it has one typed seed with no passphrase), so B1's merge
cannot move it — this is restated from the fold's own log and re-confirmed by
running the same grep independently rather than trusted from the log.

---

## Additional checks run (not part of the four greps, but the same lens)

- `multisigVerifyMS1Entry` has exactly one production caller
  (`gui/multisig_verify.go:868`); confirmed by grep, no second site missed the
  signature change.
- No test in the package calls `t.Parallel()` (`grep -c "t.Parallel()"
  gui/*_test.go` → 0), so the new package-level `multisigVerifyFn` var being
  swapped in/out by `s5StubVerifyFn`'s `t.Cleanup` restore has no concurrent-
  mutation exposure across tests.
- `gui/multisig_verify_policy_test.go` is untouched by this fold (`git diff
  830aaf7..s5-multislot -- gui/multisig_verify_policy_test.go` is empty) — the
  Minor-5 dead needle round 1 recorded there is pre-existing and out of this
  fold's scope, not re-derived or re-reported.
- `gui/multisig_verify.go:698-702` (`verifyRefused` on a missing/short readback,
  F-199/N-3's site) is confirmed **unchanged** by this fold (`git diff` shows no
  touch to that block; `git show 830aaf7:gui/multisig_verify.go` already had
  `multisigVerifyNoExpectationBody`/`verifyRefused` at the equivalent lines).
  Not re-reported, per the brief.

---

## Findings

**None.** Every grep in (a) and (d) came back either empty (against a proven-
working pattern) or hit only intentional negative-assertions; every
`strings.Contains`-over-source test in (b) still names a string present in the
current tree; every behaviour-describing comment checked in (c) still matches
the code beneath it.

*Round 2. Lens: incomplete propagation. 0 Critical, 0 Important, 0 Minor/Nit
newly filed. Gate: **GREEN**.*
