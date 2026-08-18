# S6b P1 implementation report — the verify tail

Worktree: `/scratch/code/shibboleth/wt-s6b`, branch `s6b-pre-flash`, off fork
`bg002h/seedhammer` `main` = `b1479a1`. Scope: spec
`design/SPEC_s6b_pre_flash_cycle.md` §3 (F-199, F-204, R-M, F-206), gates
3.1/3.2/3.2a/3.3 only.

Three commits, in the worktree (not pushed, not merged):

| commit | covers |
| --- | --- |
| `c95dd2341d759c848c603deaaef4a847f048a423` | F-199 (gate 3.1) + R-M (gate 3.2a) — both in `gui/multisig_verify.go` |
| `3539d4b51d2d41ab5b8591246d6a6b07961fa96c` | F-204 (gate 3.2) — `gui/singlesig_verify.go` |
| `2c18a6f5c72240c0b52cf0d0287d4d42724b999f` | F-206 (gate 3.3) — `gui/verify_status.go` |

F-199 and R-M land in one commit rather than two: both touch
`gui/multisig_verify.go`, their edits are interleaved (a `git add -p` split
went wrong once — mid-session I mis-assigned two of eight hunks between the
intended commits, caught it by inspecting `git diff --cached` rather than
trusting the interactive session, and reset and re-grouped by file instead of
risking a second manual hunk split). The commit message states both gates and
both TDD sequences separately.

## Honesty note, up front

Per the coordinator's explicit ask: for every assertion below I say whether I
personally watched it flip red→green (genuine TDD) or whether it was green
before my diff and stayed green after (a regression pin, not new coverage).
Nothing below is reconstructed after the fact — every "before" transcript
quoted here is copy-pasted from the actual tool output produced when I
reverted the relevant source line(s) and re-ran, in this same session.

## Files changed

- `gui/multisig_verify.go` — F-199 (`:753`'s return value, the enum doc block,
  a defensive comment at the `:854` site) and R-M (the `provedInnocent` arm's
  body and its doc comment, plus two historical comments elsewhere in the file
  that quoted the superseded wording).
- `gui/multisig_verify_refusal_test.go` (new) — gate 3.1.
- `gui/multisig_verify_provedinnocent_test.go` (new) — gate 3.2a.
- `gui/multisig_verify_passphrase_test.go` — folded: an F-191 test
  (`TestVerifyNamesThePassphraseBeforeCondemningTheSeed`) pinned the
  `provedInnocent` arm's OLD wording verbatim ("is a cosigner") and went red
  under R-M's new text; updated to pin the new text's own affirmative claim
  ("match this seed") instead.
- `gui/singlesig_verify.go` — F-204's conditional copy.
- `gui/singlesig_verify_failure_copy_test.go` (new) — gate 3.2.
- `gui/verify_status.go` — F-206's count-free clause.
- `gui/verify_status_ms1_clause_test.go` (new) — gate 3.3.
- `gui/singlesig_truth_test.go` — folded: `t22MS1Clause`, a test-local
  transcription of the production constant (its own doc comment: "so a
  mutation to the production text has to be made twice to go unnoticed"),
  updated to match F-206's new text.

`git status --porcelain -- '*testdata*'` is empty throughout — no golden was
touched. Nothing outside `gui/` changed; the `me` CLI and `md`/`mk`/`ms1` wire
format are untouched.

## Gate 3.1 — F-199, `verifyRefused` re-offers at exactly one site

**The four sites, census (unchanged from the spec's own table, re-confirmed
against the worktree before editing):**

| line (pre-edit) | trigger | correctable | this fold |
| --- | --- | --- | --- |
| 715/717 | `len(expectedSlots) == 0` | no | untouched, still `verifyRefused` |
| 725/727 | `len(engravedMd1) == 0` | no | untouched, still `verifyRefused` |
| 745/753 | `extractReadbackMd1AndMk1s` fails | **yes** | **now `verifyIncomplete`** |
| 851/854 | `verifyFreshSlots` → `ferr != nil` | no (unreachable in-process) | untouched, still `verifyRefused` |

**Why `verifyIncomplete` at `:753`, not a new verdict or the `correctable`-flag
mechanism the follow-up suggested (the coordinator asked for this in my own
words):**

The filed follow-up (F-199) proposed reusing the `correctable` local the
function already declares for the seed-entry and ms1-entry breaks, on the
theory that the same local "would cover this site." I considered three
options:

1. **Route through `rec` (the `*verifyRecord` out-parameter).** Rejected
   immediately: `verifyRecord`'s own doc comment says it exists so flows can
   "keep their verdict return unchanged," and reusing it for control-flow
   signalling contradicts that separation on purpose, not by oversight.
2. **A new sixth verdict constant** (e.g. `verifyRefusedCorrectable`), with
   both caller loop conditions widened to include it. This is the
   textbook-clean option, and I rejected it for two reasons. First, it costs
   surface: it requires editing `gui/multisig_build.go` and `gui/multisig.go`,
   which this phase's own plan scopes as "self-contained copy + control flow,
   no new mechanism" — a fix confined to `multisig_verify.go` alone is a
   strictly smaller, more auditable diff for identical behaviour. Second, the
   type's doc comment is explicit and hard-won about being "FIVE OUTCOMES, NOT
   A BOOL" with a documented history of a miscounted headline costing a review
   round (`"THE HEADLINE SAID 'FOUR' UNTIL S6a STEP 8"`) — adding a sixth
   invites exactly that class of mistake again for no behavioural gain over
   option 3.
3. **Return `verifyIncomplete` directly** (what I did). At `:753`, `legs` is
   provably always empty — the seed-typing loop has not started yet, and
   `correctable` has not been set. The function's OWN merge point 250-odd
   lines later already computes exactly this: `if len(legs) == 0 { if
   correctable { return verifyIncomplete } else { return verifyAbandoned } }`.
   Falling through to that point from `:753` — which is what the follow-up's
   suggested mechanism amounts to — would require hoisting `legs`, `covered`,
   `typed` and `correctable`'s declarations above the readback checks and
   restructuring an early return into a long fall-through, a materially bigger
   and riskier diff for a value that is **statically determined already**:
   `correctable=true` with `len(legs)==0` always evaluates to
   `verifyIncomplete`. Writing `return verifyIncomplete` directly is that same
   computation with the dead branches removed, not a shortcut around it.

The one thing option 3 costs is honesty debt on `verifyIncomplete`'s own doc
comment, which said "what was compared MATCHED" — untrue at `:753`, where
nothing was compared. I paid that down by widening the comment (`gui/multisig_verify.go`,
both on the type-level "FIVE OUTCOMES" paragraph and on the constant itself)
to state both cases truthfully, and by adding an explanatory comment at
`:753` and at `:854` naming which of the four sites is which — satisfying the
prohibition's own text ("the gate must say which arm is which").

**Test, gate 3.1 core fix — `TestVerifyReoffersOnAnUnaccountableReadback`,**
driven directly against `multisigVerifyFlow` with a readback carrying the mk1
plate but no md1 card (`extractReadbackMd1AndMk1s`'s `len(md1)==0` arm).

BEFORE (source reverted to `return verifyRefused` at the `:753` site;
confirmed the sed target was the right line before reading this):

```
=== RUN   TestVerifyReoffersOnAnUnaccountableReadback
    multisig_verify_refusal_test.go:97: the readback-accounting failure returned 3, want verifyIncomplete (1). F-199 (S6b spec §3.1) requires THIS SPECIFIC site to re-offer, and the two caller loops only re-offer on verifyIncomplete/verifyFailed -- a verdict of verifyRefused here leaves this correctable failure un-retryable
--- FAIL: TestVerifyReoffersOnAnUnaccountableReadback (0.11s)
FAIL
```

AFTER:

```
=== RUN   TestVerifyReoffersOnAnUnaccountableReadback
--- PASS: TestVerifyReoffersOnAnUnaccountableReadback (0.03s)
PASS
```

Genuine red→green, watched directly.

**The other two gate-3.1 tests are regression pins, not TDD — stated plainly
because they never went red:**

- `TestExpectedSlotsNeverReassignedInVerifyFlow` (source assertion for `:854`
  — greps `multisig_verify.go` with comments stripped for `expectedSlots =` /
  `expectedSlots :=` and finds neither). The code it checks was never touched
  by this fold; it was true before my diff and is true after. PASS throughout.
- `TestVerifyRefusedIsNotInTheCallerLoopCondition` (source assertion — greps
  `multisig_build.go` and `multisig.go` for the loop condition
  `if res != verifyIncomplete && res != verifyFailed {` verbatim). Neither
  file was touched by this fold. PASS throughout.

**Why these are source assertions rather than the behavioural walk the gate's
own wording ("717/727 behavioural non-loop") suggests — a deliberate,
disclosed deviation:** I wrote the behavioural version first. It drives
`supplyMultisigPolicyFlow` and `buildMultisigPolicyFlow` through a full engrave
(via the `s5StubVerifyFn` seam, mirroring `TestBothEngraveFlowsDriveTheRetryLoop`)
and asserts the retry lead never draws on a `verifyRefused` stub. It PASSED on
both call sites, confirming the behaviour is correct:

```
--- PASS: TestVerifyRefusedDoesNotReoffer (119.06s)
    --- PASS: TestVerifyRefusedDoesNotReoffer/supply (73.68s)
    --- PASS: TestVerifyRefusedDoesNotReoffer/build (45.38s)
```

119s is the problem. The runbook measures the `gui` package at 429–507s
against Go's 600s per-package default and warns explicitly that a previous
cycle blew through that ceiling with every assertion passing. Adding 119s on
top of a baseline that can run as high as 507s risked reproducing exactly
that failure. I removed the walk and replaced the "does NOT loop" half with a
source assertion pinning the caller condition's literal text — a precise,
direct encoding of the actual prohibition ("verifyRefused must not be added to
that condition"), reasoned to be sound because the condition is one Boolean
expression neither caller computes dynamically, and because
`TestBothEngraveFlowsDriveTheRetryLoop` already exercises the identical inline
code behaviourally for the direction that does need to loop. This is a
judgment call a reviewer may want to revisit; the 119s walk is preserved in
this report's history (this paragraph) even though it is not in the diff.

## Gate 3.2 — F-204, single-sig failed-verify copy is conditional on the passphrase

**Test — `TestSingleSigVerifyFailedCopyConditionsOnPassphrase`,** two
subtests off one shared watch-only bench bundle (`abandonAboutPhrase()`, no
passphrase): "passphrase entered" re-types the same seed but adds a
passphrase at re-verify (mints a different wallet, so the comparator
disagrees against the bench plates); "no passphrase" re-types a different
seed (`fixtureMasterB`, premise-checked by
`TestFixtureMasterBFillsADifferentSingleSigWallet` to actually derive a
different wallet at the shared default path) with no passphrase.

BEFORE (`gui/singlesig_verify.go` reverted via `git stash`):

```
=== RUN   TestSingleSigVerifyFailedCopyConditionsOnPassphrase
=== RUN   TestSingleSigVerifyFailedCopyConditionsOnPassphrase/passphrase_entered
    singlesig_verify_failure_copy_test.go:78: a FAILED verify with a passphrase entered must suspect the passphrase before the plates (S6b spec §3.2); got "Theread-backbundledoesNOTmatchtheseed.Checktheengravedplates.VerifyFailed"
    singlesig_verify_failure_copy_test.go:82: the plates-first wording is a false lead once a passphrase was entered -- the plates may be perfect and the passphrase wrong; got "Theread-backbundledoesNOTmatchtheseed.Checktheengravedplates.VerifyFailed"
=== RUN   TestSingleSigVerifyFailedCopyConditionsOnPassphrase/no_passphrase
--- FAIL: TestSingleSigVerifyFailedCopyConditionsOnPassphrase (0.11s)
    --- FAIL: TestSingleSigVerifyFailedCopyConditionsOnPassphrase/passphrase_entered (0.05s)
    --- PASS: TestSingleSigVerifyFailedCopyConditionsOnPassphrase/no_passphrase (0.04s)
FAIL
```

AFTER:

```
=== RUN   TestSingleSigVerifyFailedCopyConditionsOnPassphrase
=== RUN   TestSingleSigVerifyFailedCopyConditionsOnPassphrase/passphrase_entered
=== RUN   TestSingleSigVerifyFailedCopyConditionsOnPassphrase/no_passphrase
--- PASS: TestSingleSigVerifyFailedCopyConditionsOnPassphrase (0.11s)
    --- PASS: TestSingleSigVerifyFailedCopyConditionsOnPassphrase/passphrase_entered (0.05s)
    --- PASS: TestSingleSigVerifyFailedCopyConditionsOnPassphrase/no_passphrase (0.04s)
```

**Honesty note:** only the `passphrase_entered` subtest is genuine red→green
— it exercises the arm the fix changes. `no_passphrase` PASSED even against
the unmodified source, above, because that arm's wording is untouched by
design (spec: "no passphrase → that wording would be a false lead; the copy
says something true of that case" — the shipped "Check the engraved plates."
was already true of that case, so I left it alone rather than churn it). It
is a regression pin confirming the conditional split didn't accidentally
touch the arm it shouldn't, not a second red→green.

## Gate 3.2a — R-M, the `provedInnocent` arm's body

**Tests — `TestProvedInnocentBodyIsRMsAdoptedWording`** (exact byte match
against the 251-char verbatim text from spec §3.2a/REQUIREMENTS §2bis) and
**`TestProvedInnocentBodyDoesNotClaimAPassphraseIsRequired`** (absence of the
forbidden "necessary to use the key" claim and the struck "skip the
passphrase" advice; absence of an em dash; presence of the required "not a
passphrase-protected wallet" statement).

BEFORE (`gui/multisig_verify.go` reverted via `git stash`):

```
=== RUN   TestProvedInnocentBodyIsRMsAdoptedWording
    multisig_verify_provedinnocent_test.go:32: multisigVerifyNoSlotBody(true, true) =
        "That seed IS a cosigner of this policy, but not with the passphrase you typed: this wallet's keys come from the seed with no passphrase. Your plates are fine. Try again and skip the passphrase."
        want (R-M's adopted wording, REQUIREMENTS §2bis, verbatim)
        "These plates match this seed with NO passphrase. This is not a passphrase-protected wallet. If you meant to use one, these plates are not that wallet: try the password again. If you continue without a passphrase, these plates are complete as they are."
--- FAIL: TestProvedInnocentBodyIsRMsAdoptedWording (0.00s)
=== RUN   TestProvedInnocentBodyDoesNotClaimAPassphraseIsRequired
    multisig_verify_provedinnocent_test.go:53: the provedInnocent body still carries the struck "skip the passphrase" advice (R-M struck it as a procedural workaround that buries the finding): "That seed IS a cosigner of this policy, but not with the passphrase you typed: this wallet's keys come from the seed with no passphrase. Your plates are fine. Try again and skip the passphrase."
    multisig_verify_provedinnocent_test.go:61: the provedInnocent body does not state outright that this is NOT a passphrase-protected wallet, which is the fact R-M requires: "That seed IS a cosigner of this policy, but not with the passphrase you typed: this wallet's keys come from the seed with no passphrase. Your plates are fine. Try again and skip the passphrase."
--- FAIL: TestProvedInnocentBodyDoesNotClaimAPassphraseIsRequired (0.00s)
FAIL
```

AFTER:

```
=== RUN   TestProvedInnocentBodyIsRMsAdoptedWording
--- PASS: TestProvedInnocentBodyIsRMsAdoptedWording (0.00s)
=== RUN   TestProvedInnocentBodyDoesNotClaimAPassphraseIsRequired
--- PASS: TestProvedInnocentBodyDoesNotClaimAPassphraseIsRequired (0.00s)
```

Genuine red→green on both, watched directly.

**`TestProvedInnocentBodyPassesTheModalFitClassCheck`** (the F-185 class
check, one call: `assertModalBodyFits(t, ..., errorScreenBody, body)`) PASSED
both before and after — the old text also fit inside the budget (159 chars
drawn, 397 headroom, pre-fix; 209 chars drawn, 360 headroom, post-fix; margin
80). Regression pin, not new coverage. Full sweep coverage over every modal
this cycle touches belongs to P6 per the plan; this call is P1's own use of
the same one-line check spec §4 says already exists, on the one body this
phase introduces.

**Pre-existing test folded — `TestVerifyNamesThePassphraseBeforeCondemningTheSeed`
(F-191, `gui/multisig_verify_passphrase_test.go`).** Discovered by running the
broader regression sweep after implementing R-M, not by TDD against a gate:
this test transcribed the OLD `provedInnocent` text's "is a cosigner" claim
and went red once R-M's new wording shipped.

```
=== RUN   TestVerifyNamesThePassphraseBeforeCondemningTheSeed
    multisig_verify_passphrase_test.go:68: the certain arm does not tell the operator their seed IS in the policy, which is the one fact the re-derivation proved:
        These plates match this seed with NO passphrase. This is not a passphrase-protected wallet. If you meant to use one, these plates are not that wallet: try the password again. If you continue without a passphrase, these plates are complete as they are.
--- FAIL: TestVerifyNamesThePassphraseBeforeCondemningTheSeed (0.11s)
```

I updated the assertion to pin the new text's own affirmative claim ("match
this seed") in place of the superseded "is a cosigner", keeping the test's
original intent (the certain arm alone makes an affirmative claim; the unsure
and skipped arms must not) intact:

```
=== RUN   TestVerifyNamesThePassphraseBeforeCondemningTheSeed
--- PASS: TestVerifyNamesThePassphraseBeforeCondemningTheSeed (0.41s)
```

I also checked, rather than assumed, that `multisigVerifyCoveredSeedBody` —
a DIFFERENT function with its own independent "Try again and skip the
passphrase" text, surfaced by grep during this check — is out of R-M's scope
(spec §3.2a names only `multisigVerifyNoSlotBody`) and untouched; its test
(`TestVerifyCoveredSeedBodyDoesNotAssertAForeignSeed`) passes unaffected.

I also fixed two stale doc comments in `gui/multisig_verify.go` that quoted
the old `provedInnocent` wording verbatim while explaining unrelated design
decisions (the `correctable` local's rationale, and the "ALL THREE ARMS
PRESCRIBE A REMEDY" note) — comments, not assertions, so nothing to fail/pass,
but left uncorrected they would mislead the next reader searching for text
that no longer exists.

## Gate 3.3 — F-206, the ms1 pass-line clause is count-free

**Test — `TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts`,** flow-level per
the spec's own requirement (a `passRecord` literal cannot distinguish "1 seed
filling 2 legs" from "2 seeds filling 2 legs" — the case that kills F-206's
own filed pluralisation remedy). Drives the real `multisigVerifyFlow` in full
mode, typing seeds one at a time, over one shared Trace B fixture
(`s5TraceBEngraved(t, true)`: master A at slots @0/@1 different origins,
master B at @2; measured 3 mk1 plates, 2 distinct ms1 plates since
`buildEngraveTail` dedupes on the ms1 string value):

| case | expectedSlots | seeds typed | legs |
| --- | --- | --- | --- |
| 1 seed / 1 leg | {0} | A | 1 |
| 1 seed / 2 legs | {0,1} | A (fills both) | 2 |
| 2 seeds / 2 legs | {0,2} | A, then B | 2 |

BEFORE (`gui/verify_status.go` reverted via `git stash`):

```
=== RUN   TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts
=== RUN   TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts/1_seed_1_leg
    verify_status_ms1_clause_test.go:106: [1 seed 1 leg] pass line does not carry the count-free ms1 clause: "1 key plate was read back and matched what this run engraved. The ms1 secret you typed matched this seed. Other cosigners' keys are taken as supplied."
    verify_status_ms1_clause_test.go:118: [1 seed 1 leg] pass line carries "The ms1 secret you typed matched this seed.", which F-206 (S6b spec §3.3) removes: ...
=== RUN   TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts/1_seed_2_legs
    verify_status_ms1_clause_test.go:106: [1 seed 2 legs] pass line does not carry the count-free ms1 clause: "2 key plates were read back and matched what this run engraved. The ms1 secret you typed matched this seed. Other cosigners' keys are taken as supplied."
=== RUN   TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts/2_seeds_2_legs
    verify_status_ms1_clause_test.go:106: [2 seeds 2 legs] pass line does not carry the count-free ms1 clause: "2 key plates were read back and matched what this run engraved. The ms1 secret you typed matched this seed. Other cosigners' keys are taken as supplied."
--- FAIL: TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts (0.26s)
    --- FAIL: TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts/1_seed_1_leg (0.07s)
    --- FAIL: TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts/1_seed_2_legs (0.07s)
    --- FAIL: TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts/2_seeds_2_legs (0.08s)
FAIL
```

AFTER:

```
=== RUN   TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts
=== RUN   TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts/1_seed_1_leg
=== RUN   TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts/1_seed_2_legs
=== RUN   TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts/2_seeds_2_legs
--- PASS: TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts (0.24s)
    --- PASS: TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts/1_seed_1_leg (0.03s)
    --- PASS: TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts/1_seed_2_legs (0.05s)
    --- PASS: TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts/2_seeds_2_legs (0.13s)
```

All three subtests genuine red→green, watched directly. This is the case
that specifically kills the naive pluralisation remedy: at "1 seed 2 legs" a
pluralised clause would ALSO have been wrong (one seed typed one ms1, not
several) — the count-free replacement is the only wording that is true in
all three rows at once, and the test asserts the old text's absence in every
row, not just its presence being replaced in some of them.

The `t22MS1Clause` fold in `gui/singlesig_truth_test.go` is a transcription
update, not a gate — it broke for the same reason as any other place quoting
the old string, and I updated it rather than leave a red suite.

## Full-suite result

Toolchain: `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin`. Run
once, at the very end, stdout and stderr to separate files:

```
go test ./... -count=1 \
  1> /tmp/s6b_p1_fullsuite_stdout.txt \
  2> /tmp/s6b_p1_fullsuite_stderr.txt
```

Exit code 0. `seedhammer.com/gui` — the package the runbook warns can run
429–507s against Go's 600s per-package ceiling — finished at **397.892s**,
comfortably inside that range and nowhere near the ceiling; my added tests
are all sub-second unit/direct-flow tests (the one heavyweight walk I wrote
was removed, see gate 3.1 above). 51 packages report `ok`, the rest report
`[no test files]`; `grep -i 'FAIL\|panic'` over both files matches nothing.
`/tmp/s6b_p1_fullsuite_stderr.txt` is 0 bytes.

`go vet ./...`: the two pre-existing failures named in the brief
(`gui/freetext_sizeproof_golden_test.go:111`, `gui/op/draw_test.go:176`, both
`testing.ArtifactDir requires go1.26 or later`) are present, plus the SAME
class of failure in four other files this phase never touched
(`gui/op/draw_test.go` already counted; also `bspline/bspline_test.go`,
`engrave/engrave_test.go`, `backup/backup_test.go`,
`backup/freetext_test.go`) and unrelated `unkeyed fields` warnings in
`bspline/bspline_test.go`. None of my nine changed/new files appear anywhere
in the vet output.

`go build ./...` and `gofmt -l gui/`: clean.

## Prohibitions, checked against the diff

- `verifyRefused` is not widened: only `:753`'s return value changed; the
  other three sites are byte-identical to `main`. Confirmed by the source
  assertion (`TestVerifyRefusedIsNotInTheCallerLoopCondition`) and by reading
  the commit diff.
- `:854` carries a source assertion, not a behavioural test, with a comment at
  the site stating which of the four arms it is.
- F-204's copy is conditional (an `if passphrase != ""` branch), not a string
  swap; both arms are tested, and the no-passphrase arm's text is byte-for-byte
  the original.
- R-M's body is the spec's exact 251-character text — asserted byte-for-byte
  in `TestProvedInnocentBodyIsRMsAdoptedWording`, not paraphrased. No em dash
  (checked programmatically, both in the Go source and in the rendered
  string). "A passphrase will be necessary to use the key" is asserted absent.
- F-206 does not pluralise over `passRecord.legs`; the replacement clause is
  the exact count-free text from spec §3.3, and gate 3.3's test specifically
  exercises the case (1 seed / 2 legs) where a plural would have been wrong.
- `multisigVerifyNoSlotBody`'s doc comment is updated in the same commit as
  the body it describes.
- No golden moved (`git status --porcelain -- '*testdata*'` empty throughout).

## What I could not do / spec gaps I worked around

Nothing in P1's scope was blocked. Two implementation-level choices the spec
left open, both already covered above in context, summarized here:

1. **The `:753` mechanism** (verdict reuse vs. a new verdict vs. the
   follow-up's `correctable`-flag fall-through) — spec states the required
   BEHAVIOUR (only `:753` re-offers) but not the Go-level mechanism. Resolved
   as `return verifyIncomplete` directly; full reasoning above.
2. **F-204's exact wording** for the passphrase-entered arm — spec gives
   direction ("suspect the passphrase before the plates") citing the multisig
   sibling's phrasing, not an exact string, and explicitly leaves the
   no-passphrase arm's wording unsettled (§8). I modelled the passphrase-entered
   arm closely on the multisig sibling's own established sentence and left the
   no-passphrase arm's shipped text untouched, since spec only requires it stay
   true of that case.

One process deviation, disclosed rather than hidden: gate 3.1's "717/727
behavioural non-loop" requirement is satisfied by a source assertion instead
of a behavioural walk, for the suite-time-budget reason detailed under gate
3.1. The behavioural version was written and confirmed passing before being
removed from the diff.
