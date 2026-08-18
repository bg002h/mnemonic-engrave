# S6b P9 — fold of the failure-states review's three Important findings

Worktree: `/scratch/code/shibboleth/wt-s6b`, branch `s6b-pre-flash`. Two
commits: `15d8fed` (F1) and `511f7f3` (F2+F3), both on top of `c333e97`.
Source review, persisted verbatim: `design/agent-reports/s6b-failure-states-review.md`.

The review's own proposed fixes were flagged NOT authoritative (the prior
fold got three of four wrong). Each finding below was reproduced against the
real code before any fix was written, and each fix was chosen only after that
reproduction — not copied from the review's prose.

## F1 — aborted/rejected passphrase-plate steel leaves no destroy warning

**Reproduced.** `gui/s6b_p9_failure_states_test.go`,
`TestPassphrasePlateAbortAfterEngraveAttemptWarnsToDestroy`: drives the real
`engravePassphraseFlowPreloaded` through a COMPLETE cut (new helper
`engraveOnePlateThenReject`, same hold-to-confirm sequence as the existing
`engraveOnePlate`) that is then REJECTED at the accept screen (Back instead
of the checkmark — sequence (b) from the review; sequence (a), an abort
mid-cut, latches the identical bool at the identical call site and was not
separately walked, since the two collapse to the same `false` return from
`EngraveScreen.Engrave`). The test then backs all the way out through
Confirm → QR → Entry → out of the program and asserts a DESTROY-shaped modal
appears before the function returns.

**Failing test before the fix.** Confirmed by temporarily gating the new
warning call with `if false && attempted`: the test failed with
`"a COMPLETE passphrase plate was cut and then rejected, and backing out of
the program afterwards shows no destroy warning at all"` — reproducing the
review's claim exactly (no such screen exists pre-fix; the flow silently
returns `passphrasePlateNotCut`).

**Fix.** `gui/passphrase_flow.go`: a local `attempted bool` in
`engravePassphraseFlowPreloaded`, latched immediately *before*
`NewEngraveScreen(...).Engrave(...)` is called (not after — `Engrave`'s bool
return collapses "never started", "stopped mid-cut", and "cut in full but
rejected" into the same `false`, so the caller has no other signal). On the
one `notCut` exit reached while the operator is still at the device (backing
out of the entry step, whether on a first visit or a return trip after
reaching Engrave), a new dismissible `"Passphrase Plate"` modal
(`passphraseAbortWarningText`) fires:

> "This attempt is not counted anywhere as a backup. If any of the
> passphrase plate was cut, it must be DESTROYED, not binned: cut it up or
> grind the words off. It would carry the wallet's spending passphrase in
> the clear."

**Deviation from the review's literal prescription.** The review said "on
any notCut exit with it set" (including the `ctx.Done` fallthrough for
power-loss/walk-away). I did NOT add it there: `showModal`'s own loop guard
is `for !ctx.Done` (`gui/slip39_polish.go`), so a call reached only after
`ctx.Done` has already fired is a silent, unobservable no-op — there is also
no operator at the device to read it. This is exactly what the review's own
§3 item 5 ("traced and found SOUND") already established: mid-engrave power
loss produces no restore document that run, so nothing downstream
misrepresents a set that does not exist. Adding the call there would be dead
code proven by nothing. I refined the review's prescription rather than
reproducing it verbatim.

**Passing output after.** `TestPassphrasePlateAbortAfterEngraveAttemptWarnsToDestroy`
PASS (1.4s). Two supporting tests: `TestPassphraseAbortWarningTextFits`
(F-185-class modal-fit check via `assertModalBodyFits`: 179 chars drawn in
full, 397 chars headroom against the required 80-char margin) and
`TestPassphraseAbortWarningTextIsHedged` (pins that the wording is
conditional — "If any … was cut" — never an unconditional assertion that a
plate WAS cut, since the bool cannot distinguish those worlds).

**What the operator now sees.** After cutting (or attempting to cut) the
passphrase plate and then backing out all the way to the top of that
sub-program, a dismissible modal titled "Passphrase Plate" tells them the
attempt is not counted anywhere and, if any steel was cut, to destroy it —
mirroring the main set's existing `bundleAbortWarningText` modal in shape
and doctrine. Dismissing it continues the existing exit (back to the
passphrase-plate offer or on to the restore document, unchanged).
`passphrasePlateResult` is untouched (still `passphrasePlateNotCut`); GATE
6a's CUT-vs-OFFERED condition and the restore document's wording are
unchanged.

## F2 — the single-sig verify tail dead-ended on both adverse arms

**Reproduced.** Pre-fix, `singleSigVerifyFlow` was `void` and its only
caller (`engraveSingleSigFlow`) was a one-shot `if`. Reproducing required a
signature change (there is no way to observe "would the caller have
retried" without a return value to check), so the RED here is: the fix
could not be *written* without first changing the signature — confirmed by
building the retry-loop test against the old void signature, which does not
compile. This is a legitimate TDD red (the code cannot do what is being
asserted), not a proxy.

**Fix.** `gui/singlesig_verify.go`: `singleSigVerifyFlow` now returns
`bool` — "can the operator still act on this with what's in their hand" —
`true` only at its two ADVERSE return sites (an unreadable readback,
`gui/singlesig_verify.go:~163`; a disagreeing comparator, `~228`), `false`
everywhere else including the success return. `gui/singlesig.go`:
`engraveSingleSigFlow`'s verify offer is now a `for` loop, mirroring the
shape both multisig callers already use (`gui/multisig.go`,
`gui/multisig_build.go`), dispatching through a new `singleSigVerifyFn` test
seam (mirrors `multisigVerifyFn`) and re-using the existing
`multisigVerifyRetryLead` constant ("Not every plate is verified. Try
again?") — now drawn by three call sites; its doc comment updated
accordingly.

**The `statusVerifiedOnRetry` trace the review's brief demanded.**
`TestSingleSigVerifyRetryProducesAnHonestStatusVerifiedOnRetryLine` drives
the REAL `singleSigVerifyFlow` twice against one sticky `verifyRecord` — no
stub, no seam substitution for the function itself: first with a seed that
is NOT what the plates were engraved with (guaranteed comparator FAIL on
plates that are actually fine; `ret1 == true`), then with the correct seed
(clean PASS; `ret2 == false`). It asserts:

- `rec.adverse == true` (sticky, survived the retry — declared outside the
  loop, matching multisig's own reasoning)
- `rec.pass != nil`
- `verifyStatusFor(rec) == statusVerifiedOnRetry`
- `buildVerifyStatusLine(rec) == buildVerifyPassLine(*rec.pass) + " " + verifyStatusRetryClause`
  (identity with the function's own construction, not a hand-typed literal)
- the rendered line's actual content: `"1 key plate was read back and
  matched what this run engraved. No secret seed share was read back or
  compared. An earlier check did not pass; a later full check passed."`
  — correct leg count (1), correct watch-only no-ms1 clause, no multisig
  cosigner clause (single-sig has none), and the retry clause naming a real
  earlier failure this same test drove.

This is TRUE because `rec.pass` is written only at the flow's actual success
fall-through and `rec.adverse` is sticky by construction — the same
mechanism already proven correct on the multisig side; this test is the
proof it holds for single-sig now that the cell is reachable.

**One test-only obstacle found and fixed while reproducing:**
`bundleGatherFlow` consumes `ctx.syswBundleSeeds` on entry (sets it to
`nil`) — a one-shot payload. The retry test re-sets it between the two
attempts (the plates are still on the operator's bench; this mirrors a
second NFC tap). Not a production defect — `ctx.syswBundleSeeds` is a
harness/payload field, and the production retry loop re-runs
`bundleGatherFlow` itself each time, which re-reads whatever is on the
reader at that moment.

**Rejected shortcut, not a rejected fix.** I did not add a third,
expensive, full click-through walk test of the generic "VERIFY AGAIN" /
"CONTINUE" `ChoiceScreen` loop mechanics (mirroring multisig's stub-seam
walk, `s5AssertRetryLoop`). That mechanism is byte-identical to the already
mutation-tested multisig shape (same `for`, same two-choice re-offer, same
shared constant) — the mechanical source-needle test
(`TestSingleSigEngraveReOffersTheVerify`, proves the caller's source
actually dispatches through the seam, reads the result as the loop
condition, and draws the retry lead) plus the real behavioural test above
together cover both the wiring and the substantive correctness without
re-proving a mechanism this fold did not invent.

**What the operator now sees.** On a FAILED verify or an unreadable
readback, instead of silently moving on to the passphrase-plate offer and a
restore document that permanently says "did not pass," they see "Not every
plate is verified. Try again?" with two choices, "VERIFY AGAIN" and
"CONTINUE" — exactly what the multisig side already shows for the same
class of failure. Pressing VERIFY AGAIN re-runs the verify (re-type seed,
re-derive, re-present cards); CONTINUE proceeds as before. A later clean
pass after an earlier failure is recorded and reported honestly as
"verified on retry," not as a bare pass and not as "did not pass."

## F3 — the no-passphrase verify arm blamed the plates for an omitted passphrase

**Reproduced.** New subtest `"engraved with passphrase, omitted at verify"`
in `gui/singlesig_verify_failure_copy_test.go` (extends the existing F-204
gate test, since it is the same gate widened by one arm): derives a bench
bundle WITH a real passphrase (`"hunter2"`), then drives the real
`singleSigVerifyFlow` with the SAME seed but Skip at the verify's passphrase
prompt. **Confirmed RED** by temporarily reverting the fix to the original
two-way switch: both new assertions failed — the screen showed `"Check the
engraved plates."` (the false lead) and did not show the new sentence.
Restored, both pass.

**Fix.** `singleSigVerifyFlow` gains a fifth parameter,
`engravedWithPassphrase bool` — the caller's own `passphrase != ""` fact
from the engrave step, plumbed through (not a derivation input, only
failure copy — does not touch §7.4's independence rule). The failure-copy
`if` became a three-arm `switch`:

1. a passphrase was TYPED at verify → existing F-204 wording (suspect it
   first), unchanged.
2. no passphrase typed, but the engrave USED one → NEW: `"The read-back
   bundle does NOT match the seed. This set was engraved WITH a passphrase,
   and none was typed just now. Add the passphrase and try again before you
   doubt the plates."`
3. neither → original `"Check the engraved plates."`, left alone — it is
   true there, and the review's own framing agreed it should not be churned
   for its own sake.

**Scope, stated rather than silently left.** The review names a SECOND
trigger of the same false lead: re-picking a different wallet type
(purpose/script) at verify than the engrave used. The review's own
prescribed "smallest fix" — plumb one bool — does not cover that trigger,
and this commit does not either: closing it would require plumbing
purpose/script too (a materially bigger diff than "one bool"), and the
review did not prescribe that remedy. **This is a known residual gap**: a
wallet-type mismatch at verify can still produce the false "Check the
engraved plates." lead. Recorded here rather than silently dropped; a
follow-up would be the right vehicle if the operator wants it closed before
the flash.

**Passing output after.** `TestSingleSigVerifyFailedCopyConditionsOnPassphrase`
— all three subtests PASS, including the new one. The two pre-existing
subtests (and the two other pre-existing call sites in
`gui/singlesig_truth_test.go`) were updated to pass `false` for the new
parameter (all are no-passphrase-at-engrave scenarios).

**What the operator now sees.** Engrave with a passphrase, then at verify
press Skip (or otherwise omit it) instead of retyping it: the comparator
still fails (guaranteed — different derivation), but the screen no longer
tells them to doubt correct plates. It now says the set was engraved with a
passphrase and none was typed, and to add it and try again — landing
directly on F2's new retry loop, so that "try again" is now an action the
device can actually perform.

## Gate result

`go build ./gui/...` and `go vet ./gui/...` clean (ignoring pre-existing,
unrelated `testing.ArtifactDir requires go1.26` notices in
`freetext_sizeproof_golden_test.go` and `op/draw_test.go` — present on HEAD
before this fold, toolchain/version noise, not something this fold
touches).

`scripts/gui-shard-test.sh ./gui/ 6 20m`, run twice (once per commit's final
state): 868 top-level tests, exhaustive partition verified, 6 shards.
**Exactly one failure both times**: `TestGate51bMaxScrollAgreesWithVisibility`,
measured at **22/321 divergences in [239,260]** both times (identical
`bodysz.Y=239..260`, identical `maxScroll=1..22` values) — unchanged from
before this fold, matching the required expectation exactly. No other test
failed in any shard, either run. No golden test moved.

`go test $(go list ./... | grep -v '/gui$') -count=1 -timeout 20m`: every
non-gui package green.

## Commits

- `15d8fed` — F1 (passphrase_flow.go + its own tests in the new file).
  Verified to build and pass standalone (checked via `git stash` against
  the F2/F3 changes) — the commit message's hedge about this ("does not
  yet build in isolation") was written before that check and is INCORRECT;
  it does build and pass standalone. Noted here rather than amended, per
  the no-amend rule.
- `511f7f3` — F2+F3 combined (both land in the same function and the same
  switch statement, sharing the signature change; no natural intermediate
  state separates them without manufacturing a throwaway one).

Neither commit touches `DERIVED` wording, R-M's body, or body width 417.
GATE 5.1b's failure is unchanged (22/321 in [239,260], never adjusted).
