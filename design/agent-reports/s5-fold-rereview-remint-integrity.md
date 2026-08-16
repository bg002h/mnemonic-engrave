# S5 fold re-review — round 1 (scope: fold diff `7da66bd..830aaf7` only)

**Reviewer stance:** independent adversarial re-review of the FOLD, not a fresh audit of the
10 commits under `7da66bd`. Two questions only: (1) did the fold CLOSE each finding it claims
to close, or MOVE it; (2) did the fold INTRODUCE a new defect. Special focus per brief: is the
re-minted `S5-trace-b` gate record honest, and does `walk_trace_b.js` actually throw.

**Verdict: 0 Critical / 0 Important. GREEN.** No finding survives. This is a legitimate clean
result, not padding — every claim below was checked against the real code or a real mutation,
not against the fold report's prose.

---

## LENS: is the re-minted gate record honest, and does the walk actually throw?

### (a) `git diff 7da66bd..830aaf7 -- oracle/gaterecords/` — real re-mint or hand edit?

Ran: `git diff 7da66bd..830aaf7 -- oracle/gaterecords/S5-trace-b.{expect,record,walk}.json`.

Changed fields: `derived_at`/`recorded_at` timestamps (09:53:56Z → 13:10:22Z), `elapsedSec`
(446 → 462), the walk's `sha256` in `record.json`, `keySourcesScreen` (gained the multi-account
notice sentence), `reviewScreen` (gained the per-slot origin enumeration), `claims`
(`multiAccountNotice: true` added). **Unchanged:** all 17 `digests`, all 17 `census.strings`
(the QR/text payloads that are actually cut into steel), the 51-entry `acts` sequence,
`payloadDigest`.

This is exactly what a genuine re-mint of the **same** underlying policy looks like: I-7 and
C-2 changed only what two UI screens say before the plates are cut, not the wallet policy or
keys, so the physical engraving content (and therefore its digests) is correctly unchanged.
Verified independently, not taken on faith:

* Recomputed the walk file's own sha256 and it matches what `record.json` claims:
  `python3 -c "import hashlib; print(hashlib.sha256(open('oracle/gaterecords/S5-trace-b.walk.json','rb').read()).hexdigest())"`
  → `3ac0f85d1dc53b67f08b98a95d9891031c86708698cae01d173da436bcf68d63`, byte-identical to
  `record.json`'s `walk.sha256`. A hand-typed record would need to hand-compute a correct
  SHA-256 of edited JSON, which is not what a hand edit looks like.
* Commit timestamp for `7dc49be` (the re-mint commit) is `2026-08-16 06:14:35 -0700` =
  `13:14:35Z`, ~4 minutes after the record's `derived_at` of `13:10:22Z` — consistent with the
  mint happening immediately before the commit, not backdated.
* `digests` come from `window.shToolpath.summary().digest` (`cmd/emu/walk_trace_b.js:430`) —
  the actual per-plate engraving toolpath hash — not from screen text, so their being unchanged
  while screen text changed is the *expected* signature of a genuine re-run over an unchanged
  policy, not evidence of staleness.

**Conclusion: consistent with a real re-mint.** No hand-edit signature found.

### (b) Artifact/digest counts — measured, not claimed

```
$ python3 -c "import json; w=json.load(open('oracle/gaterecords/S5-trace-b.walk.json')); \
  print(len(w['digests']), w['plateCount'], len(w['census']['strings']), w['elapsedSec'], \
  len(w['acts']), sum(1 for a in w['acts'] if a['screen']=='engrave-done'))"
17 17 17 462 51 17
$ python3 -c "import json; print(len(json.load(open('oracle/gaterecords/S5-trace-b.expect.json'))['artifacts']))"
17
```
17 digests, 17 plates, 17 census strings, 17 `engrave-done` acts, 17 `expect.json` artifacts,
462s elapsed. **Matches the fold report's claim exactly** — measured independently, not copied
from it.

### (c) Do the walk's new assertions actually throw?

Read `cmd/emu/walk_trace_b.js` diff directly (not the fold's description of it). Two additions:

1. `ORIGINS_EXPECTED` loop (around the "past the Policy Review" tap): for each expected origin,
   `if (!review.text.includes(squash(want))) { throw new Error(...) }`. This is a real throw,
   executed before the confirm tap — not a flag recorded into the returned object.
2. `claims.multiAccountNotice` is computed as a boolean (`sources.text.includes(squash(
   "multi-account wallet and is allowed"))`), then immediately: `if (!claims.multiAccountNotice)
   { throw new Error(...) }`. Also a real throw with a distinct predicate from the pre-existing
   `claims.multiAccount` (which only greps for `"account 1"` and was the round-0-identified
   false friend — confirmed still present as a separate, non-authoritative field).

Both failure paths exist in the source as unconditional `throw` statements, not `console.log`
or a recorded flag. I did not re-run a live browser walk to trigger these throws (out of budget
for a re-review scope), but the source-level evidence is unambiguous: these are not
dead/flag-only gates — the "I-9 class" the brief warns about does not apply here.

### (d) `TestS5GateHasARecord` — exists, runs, and fails on absence

```
$ nix develop --command go test ./oracle/... -run 'TestS5GateHasARecord|TestEveryRequiredStageHasAGateRecord|TestS0GateHasARecord' -v -count=1
--- PASS: TestS0GateHasARecord (0.00s)
--- PASS: TestEveryRequiredStageHasAGateRecord (0.00s)
--- PASS: TestS5GateHasARecord (0.00s)
```

Then, in a `cp -a` copy (never the frozen worktree) with all four `S5-trace-b.*` files deleted:

```
$ rm -f oracle/gaterecords/S5-trace-b.*
$ nix develop --command go test ./oracle/... -run 'TestS5GateHasARecord|TestEveryRequiredStageHasAGateRecord|TestS0GateHasARecord' -v -count=1
--- PASS: TestS0GateHasARecord (0.00s)
--- FAIL: TestEveryRequiredStageHasAGateRecord
    S5 has no gate record in gaterecords (stages present: [S0]).
--- FAIL: TestS5GateHasARecord
```

**Confirmed: the test genuinely fails on absence, reproducing round 0's I-9 repro exactly.**
Not run against the frozen `/scratch/code/shibboleth/wt-s5` tree (left untouched throughout).

### (e) Do the OTHER walks still reference screens/strings the fold changed?

`cmd/emu/walk_build_policy.js`, `walk_s3_nested.js`, `walk_s4_gate.js`, `walk_trace_a.js` were
grepped for every string the fold's production edits touched (origin-announcement text,
"Checked N of the M"/"Verify Incomplete"/"Verify Failed", "belong to a different seed",
"Full (seed + keys)", "checked against"):

* `walk_s3_nested.js:452` asserts `NEEDLE_NESTED_NOTE = "BIP-48 for nested segwit (script type
  1h)"` on the review screen. Read `buildOriginAnnouncement`'s `MultisigShWsh` case
  (`gui/multisig_build.go:1630-1636`): that literal substring is emitted unconditionally
  regardless of whether `base` is the scalar or the new enumerated form — unaffected.
* `walk_s3_nested.js` and `walk_s4_gate.js` both tap row 0 of "What to engrave?" via a
  positional tap with no text assertion on the label itself — unaffected by `buildFullModeLabel`
  now varying its text (and the build path already used `buildFullModeLabel` before this fold
  per the fold's own note; only the supply-path label changed here).
* `walk_s4_gate.js:448`'s `"checked against"` needle: `grep -n "checked against" gui/*.go` shows
  it lives in `gui/multisig_build_slots.go:675,696`, in `buildSlotSourceLines` — the fold's diff
  of that file (`git diff 7da66bd..830aaf7 -- gui/multisig_build_slots.go | grep "checked
  against"`) shows **no changes** to that text.
* `heldOriginSummary` (`gui/multisig_build.go:1685-1704`) returns the unchanged scalar
  `held[0].Path` whenever all held slots share one origin (`same == true`), which is
  `walk_s3_nested.js`'s and `walk_s4_gate.js`'s shape (single-master builds) — so their review
  screens read exactly as before.

No static reference to changed text was found broken. This is a static/textual check, not a
live re-run of these three walks in a browser (they carry no committed gate record and are not
invoked by `go test`, so nothing in the settled build gate exercises them); within that scope I
found no round-0-Critical-#4-class breakage. `go test ./... -count=1` was independently re-run
at HEAD `830aaf7` and confirms exit 0, 0 FAIL (see below), so nothing static-checkable broke.

---

## Verification beyond the lens: sampled findings for close-vs-move and new defects

Given the scope (fold diff only, no fresh audit), I mutation-tested a sample of the highest-risk
closures rather than re-deriving all 17. All mutations were performed in throwaway `cp -a`
copies under `/tmp/.../scratchpad/`, never in the frozen `/scratch/code/shibboleth/wt-s5` tree,
and every copy was discarded/restored after use.

* **C-1** (verify's "Checked N of M" false comparator). Mutated the fixed code back to skipping
  `verifyMultisigLegsPartial` before reporting Incomplete (the exact pre-fix shape). Result:
  `TestVerifyIncompleteDoesNotCallAForeignPlateChecked` **FAILs** with the foreign-plate-reported-
  as-checked screen text reproduced verbatim; `TestVerifyIncompleteReportsWhatTheComparatorMatched`
  still passes (as expected — it's the non-vacuity arm on a different axis). Confirms the fix is
  real and the pinning test is non-vacuous.
* **I-13** (watch-only verify claiming "secret verified"). Mutated `multisigVerifyOKMessage`'s
  single-leg arm back to unconditionally returning `multisigVerifyOKBody`. Result:
  `TestVerifyOKMessageClaimsASecretOnlyInFullMode` **FAILs**, reproducing exactly the false claim
  round 0 found. Confirms the fix is real.
* **C-2** fixture rebuild. Read `TestGateAcceptsSameSeedAtDistinctOrigins`
  (`gui/multisig_build_gate_test.go:221-274`) directly: it now builds its registry via
  `s5Registry(t, fixtureMasterA, fixtureMasterA)` (two real `reg.add()` calls) and takes its
  `sources` from the real `buildSlotSources(p, ids, ...)` — not a hand-built `[]slotSource{
  SeedID: 0, SeedID: 0}` literal. `s5Registry` (`gui/multisig_build_s5_test.go:38-54`) is
  confirmed to call `reg.add()` once per phrase. This genuinely closes the round-0
  "unrealistic-fixture" complaint rather than moving it.
* **I-6 / I-8 departures.** Read the actual code (not the review's prescribed fix) for both.
  I-6: `classifyCosignerSupply` (`gui/multisig_build_payload.go:204-227`) now returns
  `cosignerAutoFill` on `open == 0` up front, **and** `buildMultisigPolicyFlow`
  (`gui/multisig_build.go:96-127`) wraps the entire payload/gather/review/pick block in
  `if open > 0 { ... }` so a zero-demand build never enters `bundleGatherFlow` at all — this
  matches the fold's claim that the review's fix alone would have moved the dead end one screen
  later (into `bundleGatherFlow`'s own "no complete cards" refusal), and the actual fix avoids
  that. Ran `TestZeroDemandBuildIsNotRefusedForAPayloadItDoesNotNeed` and
  `TestBuildHoldingEverySlotReachesTheSeed` directly: both PASS. I-8: confirmed
  `grep -rn "holds exactly one seed\|A seed you entered" gui/` returns zero hits outside the
  test file's own comments describing what must NOT survive; read the replacement ruling text
  in `gui/multisig_build_census.go:75-90`, which states the multi-seed reality ("Every seed you
  entered -- this build can hold several -- stays in device memory...", "Do not leave a
  mid-build machine unattended") rather than the falsified singular premise.
* **I-9.** Independently mutation-tested per lens (d) above — genuinely fails on absence.
* **C-3, I-4, I-5, I-12** production wiring. Read `gui/multisig.go`'s two-line diff and
  `gui/multisig_build.go`'s parallel changes directly: `buildFullModeLabel(passphrase != "")`
  replaces the supply path's hard-coded literal; `bundleEngrave` now returns
  `bundleEngraveResult` and **both** callers (`gui/multisig.go:295`, `gui/multisig_build.go:400`)
  gate on `!= bundleEngraveDone`; the verify offer at both call sites is a `for {}` loop that
  only continues on `verifyIncomplete`/`verifyFailed` and re-offers via the shared
  `multisigVerifyRetryLead` string. All four are structurally symmetric between the build and
  supply paths, as claimed.
* **F-189.** `grep -rn "findUserSlot(" gui/*.go | grep -v _test.go` shows exactly one production
  call site, at the new 3-return signature; `gui/multisig_engrave.go`'s diff shows a clean
  deletion of `multisigEngraveCards` with no remaining callers.

None of the sampled mutations, greps, or direct reads found the fix "moved" rather than closed,
and none surfaced a defect the fold's own text introduced.

## Machine-checked at HEAD `830aaf7` (independently re-run by this reviewer, not taken from the fold's own report)

```
$ nix develop --command go test ./... -count=1
ok  <every package>, 0 FAIL   (one pre-existing, unrelated SKIP: TestIdleTimerUnderSH2ShapedEventLoop)
$ nix develop --command ./scripts/oracle-live.sh
live checks: PASS (exit 0)
```
Matches the SETTLED baseline in the brief; reconfirmed at the exact fold HEAD rather than
assumed to still hold.

## Not investigated (explicitly out of scope per brief)

* `gui/singlesig.go` defects (N-1/N-2 in the fold report; filed as F-197/F-198 in
  `design/FOLLOWUPS.md` — confirmed present by `grep -n "F-197\|F-198" design/FOLLOWUPS.md`,
  not otherwise read or evaluated, per instruction).
* The 10 commits under `7da66bd` (round-0 scope, not re-derived).
* R-1 (refuted, not reinstated — worktree confirmed clean throughout this review, never mutated).

## Findings

None. 0 Critical, 0 Important, 0 Minor filed by this review (Minors, if any, would not gate and
were not the focus of the sampled checks above).
