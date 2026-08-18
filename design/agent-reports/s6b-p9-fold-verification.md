# S6b P9 fold verification — failure-states review's 3 Important findings

Fold-verification pass, not a fresh audit. Scope: `15d8fed` (F1) and `511f7f3`
(F2+F3) on `s6b-pre-flash` in `/scratch/code/shibboleth/wt-s6b`, folding
`design/agent-reports/s6b-failure-states-review.md` against its own account
in `design/agent-reports/s6b-p9-failure-states-fold.md`. Rulings checked:
`design/REQUIREMENTS_s6b_pre_flash_cycle.md` §2bis (R-C, R-D, R-N; R-M, R-E
also consulted for the "did not break" checks).

## 1. F1 / F2 / F3 disposition

### F1 — CLOSED

Claim: after an aborted/rejected passphrase-plate engrave, the operator is
warned to destroy any cut steel, and the modal's own `ctx.Done`-fallthrough
scoping argument holds.

Traced `engravePassphraseFlowPreloaded` (`gui/passphrase_flow.go:803-917`)
as a step machine (`ppPLStepEntry/QR/Confirm/Engrave`). The **only**
`return passphrasePlateNotCut` reachable while the operator is still at the
device is the one at the `ppPLStepEntry`/`!ok` branch (line ~840) — every
other loop-back (`step -= 2; break`) stays inside `for !ctx.Done` and never
returns directly; the bottom-of-function `return passphrasePlateNotCut` after
the loop is reached only once `ctx.Done` is already true. So there is exactly
one reachable notCut exit, and it is the one gated by `if attempted {
passphraseAbortWarning(ctx, th) }`.

Verified the `ctx.Done` no-op argument directly: `showModal`
(`gui/slip39_polish.go:23-33`) is `for !ctx.Done { ... }` — a call made after
`ctx.Done` is already true executes zero loop iterations and returns
immediately, no frame drawn, no dismiss possible or needed. The fold's
scoping is correct, not a hole: adding the call at the bottom-of-function
fallthrough would be dead code by construction.

Ran the tests myself (not just read them):
```
TestPassphrasePlateAbortAfterEngraveAttemptWarnsToDestroy   PASS (0.59s)
TestPassphraseAbortWarningTextFits    PASS — 179 chars drawn, 397 chars headroom (margin 80)
TestPassphraseAbortWarningTextIsHedged PASS
```
Text is conditional ("If any … was cut … must be DESTROYED, not binned"),
never asserts a cut unconditionally. `passphrasePlateResult` is unchanged
(still `passphrasePlateNotCut`) — GATE 6a's CUT-vs-OFFERED condition is
untouched.

**F1: CLOSED.**

### F2 — CLOSED

Claim: the single-sig verify tail now re-offers on both adverse arms, and a
retried pass renders an honest `statusVerifiedOnRetry` line.

Structural check: `singleSigVerifyFlow` now returns `bool`, `true` only at
its two adverse sites (unreadable readback, disagreeing comparator — both
confirmed by reading `gui/singlesig_verify.go:150-226`), `false` at every
other exit including the success fall-through. `engraveSingleSigFlow`'s
verify offer (`gui/singlesig.go:205-225`) is a `for {}` loop, structurally
byte-identical in shape to both pre-existing multisig callers
(`gui/multisig.go:338-350`, `gui/multisig_build.go:452-467`: same
`lead/choices` swap to `multisigVerifyRetryLead` + `{"VERIFY AGAIN",
"CONTINUE"}`, same `if !ok || sel != 0 { break }` / re-offer condition) —
confirmed by direct comparison, not by trusting the fold's "mirrors" claim.

**The critical check, driven independently.** Ran
`TestSingleSigVerifyRetryProducesAnHonestStatusVerifiedOnRetryLine` myself:
```
go test ./gui/... -run TestSingleSigVerifyRetryProducesAnHonestStatusVerifiedOnRetryLine -v -count=1
--- PASS: TestSingleSigVerifyRetryProducesAnHonestStatusVerifiedOnRetryLine (0.10s)
```
The rendered `statusVerifiedOnRetry` line the test drove and logged:

> "1 key plate was read back and matched what this run engraved. No secret
> seed share was read back or compared. An earlier check did not pass; a
> later full check passed."

This is a real fail-then-pass sequence against one sticky `verifyRecord`: a
wrong seed FAILs first (`ret1==true`), the correct seed PASSes second
(`ret2==false`), and the line is true of what the test's own run actually
did (1 key plate, watch-only so no ms1 clause, no multisig cosigner clause,
the retry clause naming a real earlier failure). `rec.adverse` is sticky
(declared outside the caller's loop) and `rec.pass` is written only at the
one success fall-through — confirmed by reading, matching the multisig
mechanism this reuses. Not overclaimed.

Also ran `TestSingleSigEngraveReOffersTheVerify` (wiring/source-needle
proof) — PASS.

**F2: CLOSED.**

### F3 — CLOSED, with a stated residual gap that is genuinely out of scope

Claim: the no-passphrase verify arm now blames an omitted passphrase, not
the plates, when the engrave used one; the wallet-type-mismatch trigger is
left open and disclosed, not silently dropped.

Read the three-arm switch (`gui/singlesig_verify.go:193-217`): `passphrase
!= ""` (typed at verify) → F-204's original wording; `engravedWithPassphrase`
(caller's own `passphrase != ""` fact from the engrave, plumbed as a new
5th parameter) with no verify-time passphrase → new sentence naming the
omission; else → original "Check the engraved plates." Confirmed
`engravedWithPassphrase` at the call site (`gui/singlesig.go:223`,
`passphrase != ""`) is the **same `passphrase` variable** used to derive the
bundle at engrave time (declared line 69, set line 77, fed to
`deriveSingleSigBundle` at line 112) — not the verify-scoped local of the
same name inside `singleSigVerifyFlow`. This is the correct fact, correctly
plumbed.

Ran `TestSingleSigVerifyFailedCopyConditionsOnPassphrase` (all 3 subtests,
including the new "engraved with passphrase, omitted at verify"):
```
--- PASS: TestSingleSigVerifyFailedCopyConditionsOnPassphrase (0.17s)
    --- PASS: .../passphrase_entered (0.05s)
    --- PASS: .../no_passphrase (0.04s)
    --- PASS: .../engraved_with_passphrase,_omitted_at_verify (0.05s)
```
The new subtest asserts the false lead ("Check the engraved plates.") is
ABSENT and the new sentence IS present, driven against a real bundle
derived with a real passphrase, correct plates, verify-time Skip.

**Residual gap, confirmed real and out of scope.** The wallet-type-mismatch
trigger (re-picking a different purpose/script at verify than the engrave
used) still falls into the same `else` arm and would still show "Check the
engraved plates." on correct plates. This requires plumbing `purpose`/
`script` too — genuinely a larger diff than the one-bool fix the review
itself prescribed as F3's "smallest fix," and the review's own text never
asked for that second trigger to be closed by this fix. The fold states the
gap explicitly (in both the commit message and the fold report) rather than
letting it pass silently. This is a disclosed, correctly-scoped deferral,
not an incomplete fix of what F3 asked for.

**F3: CLOSED** (fix matches what F3 required; the disclosed gap is a
different, out-of-scope trigger of the same symptom, not a partial fix of
F3 itself).

## 2. `statusVerifiedOnRetry` line — driven independently

See F2 above. Quoted again for the record, exactly as logged by my own run
of `TestSingleSigVerifyRetryProducesAnHonestStatusVerifiedOnRetryLine`:

> "1 key plate was read back and matched what this run engraved. No secret
> seed share was read back or compared. An earlier check did not pass; a
> later full check passed."

True of the sequence the test drove. Not overclaimed.

## 3. New finding

**N1 (Minor, non-gating) — F3's new failure-copy arm has no committed
modal-fit regression test, unlike its two sibling arms in the same
switch.** `gui/s6b_modal_fit_sweep_test.go` (P6/GATE 4) explicitly
enumerates and `assertModalBodyFits`-checks the two pre-existing arms of
this exact switch ("F-204 'passphrase entered' arm", "F-204 'no passphrase'
arm") by its own stated methodology: a `showError`-shaped body with bytes
new to a cycle is a GATED candidate. P9's third arm ("This set was engraved
WITH a passphrase, and none was typed just now. Add the passphrase and try
again before you doubt the plates.") is new production text on the exact
same funds-critical verify-failure path and meets that file's own stated
gating criteria, but was not added to that table or covered by any other
fit assertion.

I verified this is a coverage gap, not a live defect: I wrote a scratch
test (not committed, removed after) calling `assertModalBodyFits` directly
on the literal string, and it **passed** — "146 chars drawn in full,
headroom 418 chars (margin 80)". The text draws in full; nothing is cut off
today. Recorded as Minor because it is a real inconsistency with this
fold's own project convention (every sibling arm in this switch is
regression-tested this way) on a funds-critical screen, but it gates
nothing: the string is proven to fit, and a future edit that grows it would
have no automatic guard, which is the only actual risk. Smallest fix, for a
follow-up: add one entry to `TestS6bModalFitSweep`'s table.

No other new defect found in the diff (loop shape, return semantics, the
switch's three arms, `attempted`'s latch point, or the test-seam wiring).

## 4. Invariants

- **GATE 5.1b unchanged.** Ran `scripts/gui-shard-test.sh ./gui/ 6 20m`
  myself from the worktree: 868 top-level tests enumerated, partition
  verified exhaustive, 6 shards. Exactly one failure, in shard 3:
  `TestGate51bMaxScrollAgreesWithVisibility`, **22/321 divergences in
  bodysz.Y=[239,260]** (verified from the shard's captured stdout, values
  `maxScroll=1..22` at `bodysz.Y=239..260`), identical to the fold report's
  claim and matching R-E's documented, accepted consequence
  (`REQUIREMENTS_s6b_pre_flash_cycle.md` §R-E/"THE CONSEQUENCE R-E FORCES ON
  F-208"). All other 5 shards passed; all `err*.txt` files empty (0 bytes).
  **CONFIRMED — did not change, did not newly pass.**
- **Body width 417; `POLICY <8 hex> DERIVED` footer (R-N); R-M's modal
  body; no golden moved.** `git diff c333e97..HEAD --name-only` touches
  only `gui/multisig_verify.go` (comment-only, on
  `multisigVerifyRetryLead`, not `multisigVerifyNoSlotBody` — R-M's body),
  `gui/passphrase_flow.go`, `gui/singlesig.go`, `gui/singlesig_verify.go`,
  and 3 test files. The 417-width constant (`gui/gui.go:508-510`) and the
  footer-format code (`backup/passphrase.go`,
  `gui/multisig_build_census.go`, `md/template_id.go`) are untouched; no
  file matching `testdata`/`.golden` appears in the diff. **CONFIRMED.**
- **R-C survives** — the preloaded-passphrase entry screen
  (`ppPLStepEntry`) is structurally unchanged by this fold except for the
  new `attempted`-gated warning call on its exit; no re-type is introduced
  or required anywhere in the diff. **CONFIRMED.**
- **No new dead-end.** F2's retry loop actually re-invokes
  `singleSigVerifyFn`, which re-asks for seed/wallet-type/passphrase from
  scratch — "VERIFY AGAIN" is a real, working action, confirmed by the
  behavioural retry test passing end-to-end. F3's new copy ("Add the
  passphrase and try again") lands directly on that same loop: pressing
  VERIFY AGAIN after an F3-triggered failure re-prompts for the passphrase
  the operator can now supply. **CONFIRMED.**

## 5. Gate result

```
go build ./gui/...                      clean
go vet ./gui/...                        clean (pre-existing go1.26 ArtifactDir notices only, unrelated)
gui-shard-test.sh ./gui/ 6 20m          868 tests, exhaustive partition, exactly 1 expected failure
go test (all non-gui packages)          all green
```

No Critical, no Important. One new Minor (N1, non-gating, verified harmless
today). F1/F2/F3 all CLOSED.

GREEN 0C/0I
