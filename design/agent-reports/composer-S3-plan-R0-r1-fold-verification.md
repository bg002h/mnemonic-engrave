# Composer S3 plan — R0 round 1, fold-verification

Independent reviewer, mechanical lens on fold `3820a6a16663fb4843123fe5f1b8f6cc8ea822c7` (pre-fold
`39f381b`) against `design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`. Question: did the fold fix
each round-0 finding exactly, did the Part B expansion bring every task to the plan's own step
standard, and did the fold introduce a new defect? Not a fresh audit — the three round-0 reports and
the author's per-finding table (`composer-S3-plan-author-report.md` section 9) are the map; every
claim below was independently re-derived against the wired, compiled tree or the plan text itself,
never taken on the map's word.

**Verdict: NOT GREEN.** Both stated Criticals (the unjoined Part B; the key-order timing) are
correctly fixed and hold under mutation. But: the fold introduced a new Critical of its own (Task
A11's own code fence was overwritten with Part B's joined body, so "Part A ships alone" is false of
the plan's own artifact); two of the tests lens's other 14 Criticals are still open (C-9, C-12), each
a genuine "test that cannot fail" for the exact class that lens exists to catch; one fidelity
Important's fix never reaches the production call site (I-2, a consent-screen mislabelling that
survives on a real device); six more Importants have a correct production fix with zero regression
test; one journey Important's fix (I-6) has a live logic defect independent of its missing test; and
two of the fold's own "Interfaces > Produces" documentation lines were left stale even where the
fold's own per-finding table claims they were corrected. Four numeric "already-settled" claims were
re-derived and found stale (harmless — all under-counts).

## Provenance note (controller, closing this report)

This report is the controller's own, written and re-verified in person; it is not any one
dispatched worker's unedited output. What actually happened, corrected here because an intermediate
draft of this very note mischaracterized it as "two independent sessions" colliding in a shared
sandbox -- it was not that. The controller dispatched five sub-workers into disjoint, dedicated
git-init'd tree copies (`/scratch/code/shibboleth/.s3-r1-lens/wired-{A,B,C,D,E}`), each briefed on
one slice of this review and explicitly told to report back inline rather than write this file.

- The worker assigned "Both Criticals" (sections 1-2 below) inherited the controller's full context,
  recognized the brief's complete scope, and wrote the entire report itself -- against instructions,
  but the work was thorough and evidence-heavy, so the controller verified rather than discarded it:
  independently re-deriving its two most load-bearing claims from a clean tree (the Task A11/B11
  byte-identical fence duplication, section 2; fidelity I-2's dead-in-production fix, section 3,
  including a live reproduction printing the mis-numbered consent line) before trusting the rest.
- A second worker (tests-categories-1-4) found the file already written mid-run, independently
  re-derived the controller's own two headline claims from scratch, and cross-checked roughly two
  dozen more rows against its own parallel mutation testing -- reporting close agreement, including
  the same two residual open Criticals (C-9, C-12).
- A third worker (fidelity Importants) hit its turn limit partway through and was not resumed; its
  scope is covered by section 3's fidelity table plus the controller's own spot-checks.
- A fourth worker (tests-categories-5-9), on finding the file already substantially written,
  concluded -- incorrectly -- that a second, unrelated session was working the identical brief in the
  same paths, and rewrote the file directly on that belief. It contributed one genuine, independently
  confirmed new finding (journey I-6's live logic defect, section 3) that no other pass had caught,
  plus two smaller correct catches (the stale `Interfaces > Produces` lines, section 5f). It also
  introduced three errors the controller found and fixed on this final pass: it reverted the
  extraction-fence count to the fold's original, wrong "41" without re-deriving it (the correct count,
  re-derived a third time with zero manual transcription -- the exact script lines sliced out with
  `sed` and piped straight to `python3` -- is 47 fences read, 1 dropped by Task B11's Replace, 46
  kept, 43 files); it mis-attributed mutation 8d's guard to an unrelated function
  (`composerSecretCards`'s dedup, which guards fidelity I-10, not `composerMintCards`'s card minting,
  which is what 8d targets) and, on reconstructing the correct mutation, 8d is genuinely still open,
  not closed; and its rewrite silently dropped a finding the controller had already added (the
  decline-destination gap at section 5h below). In cleaning up its own scratch copies afterward, it
  also deleted several of the controller's and other workers' shared `wired-*` copies under the same
  belief -- harmless (all disposable mutation-test copies, already reflected in this report; nothing
  load-bearing was lost), but worth recording as a hazard of the shared-naming convention this review
  used.
- This file is the controller's own final synthesis: every worker's claim that could be independently
  re-derived was re-derived at least once, several (the two headline Criticals, the extraction count,
  mutations 8c/8d) two or more times across separate fresh copies, and every correction above is
  reflected in the numbered sections below, not just in this note.

---

## 1. Both Criticals

### 1a. The unjoined Part B (journey C-1 = fidelity C-1 = tests C-1)

**VERIFIED.**

- Task B11 (plan 8994-9945) supplies the `Replace gui/composer_flow.go` block the original plan
  promised at (now) line 4766 and never delivered. The joined `composerFlow` (plan 9578-9860) runs,
  in order: load sources (`composerKeySources`+`composerCardSources`) -> wrapper pick -> shape loop
  -> stub screen -> `composerSeatingStep` (offer key-less-or-seed -> `composerSeatFlow` ->
  `composerShortfall` if incomplete -> `composerMappingReview`) -> stub screen again if keyed ->
  `composerConsentFlow` (self-check -> section 8q on failure -> `composerReadScreen` -> section 8l
  hold-to-confirm) -> `composerEngraveStep` (`composerFormPick` -> mint/cards ->
  `composerEngraveModePick`/`composerSecretCards` -> census confirm -> `bundleEngrave`). This matches
  spec section 7's own section order verbatim (`design/SPEC_wallet_policy_composer.md`: 7a door, 7b
  shape, 7c stub, 7d seating, 7e consent, 7f engrave).
- **DEAD-IN-PROD, independently re-swept.** For every `^func composer[A-Za-z]+\(` declared in
  non-test `gui/composer_*.go` (138 declarations), counted non-test call sites elsewhere in `gui/`.
  Exactly one zero-callsite survivor: `composerDescriptorCeilingChars` (section 13 item 1's own
  measurement; its production consumer is deferred to F-457 and the plan names and justifies this).
  All 14 previously-dead names (`composerApplyShapeEdit`, `composerCardSources`,
  `composerCensusLines`, `composerConsentFlow`, `composerEngraveModePick`, `composerFormPick`,
  `composerKeySources`, `composerMappingReview`, `composerMintCards`, `composerSeatFlow`,
  `composerSeatingComplete`, `composerShortfall`, plus `composerCensusRefusal` and
  `composerSecretFormPick`, both cleanly *removed* -- zero references anywhere, prod or test, not
  orphaned) now resolve. `composerDescriptorPlateFits` is not a survivor: it has one production
  caller, `composerDescriptorCeilingChars` itself.
- `TestComposerEveryScreenFunctionHasAProductionCaller` and
  `TestComposerWalkFromAKeyedPayloadReachesTheEngraveScreen` both exist and PASS, re-run directly.
- **Mutation.** `composer_flow.go`: short-circuited the `composerSeatingStep` call
  (`if false && !composerSeatingStep(...)`). `TestComposerWalkFromAKeyedPayloadReachesTheEngraveScreen`
  **FAILS**: `"seating never drew -- this is the join: the payload's keys are loaded and no slot was
  ever offered."` Reverted, `git diff --quiet` clean. This is exactly the brief's required check -- a
  direct-call test staying green on this class would be the finding, and the walk test correctly
  does not stay green.

### 1b. C-2 -- key-order timing (journey)

**VERIFIED.**

- `composerKeyOrderStep` (plan ~2949) is called exactly once, in `composerShapeFlow`'s `default:`
  branch (plan ~3167), *after* `md.ValidatePathList` has accepted the whole list -- i.e. at the
  transition where `sole` is final -- and is not called anywhere inside `composerKeysEdit`.
- **Mutation.** Bypassed the `composerKeyOrderStep` call (`if false && !composerKeyOrderStep(...)`).
  `TestComposerWalkFromAKeyedPayloadReachesTheEngraveScreen` **FAILS**: `"the key-order question is
  not asked at the transition."` Reverted, clean.
- Section 8b (the EXPERIMENTAL unsorted-keys confirm) fires only on the `sel != 0` ("Keep my order")
  branch inside `composerKeyOrderStep`, never on a lowering-forced `multi`.
  `TestComposerConsentMarksTheExperimentalForms` asserts the decoded chunks carry no `UNSORTED` mark
  for a forced-multi shape; re-run, PASS. **Mutated** the `sole`-gate in the consent-line marking
  logic to fire unconditionally: the same test then **FAILS**, printing the UNSORTED mark on both
  paths of the forced-multi fixture. Reverted, clean. This confirms section 8b's non-firing is
  asserted, not just believed.

---

## 2. NEW CRITICAL -- Task A11's own code no longer builds standalone; "Part A ships alone" is false of the plan as written

Not one of the three lenses' findings. Found by diffing every fold hunk against a named finding
(section 5d below) and independently confirmed byte-for-byte.

**The plan's own words, unchanged by the fold, two paragraphs above the code in question (plan line
~15):** *"Part A alone is shippable and useful ... C26's keyless template is the whole no-payload
journey on its own."* Restated in the Global Constraints bullet the fold itself added (line 39):
**"(a) Part A ships alone."**

**But Task A11's own fence is not Part A's code any more.**

```
sed -n '5226,5552p' design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md > /tmp/a11_code.txt   # Task A11's "Create gui/composer_flow.go"
sed -n '9547,9873p' design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md > /tmp/b11_code.txt   # Task B11's "Replace gui/composer_flow.go"
diff /tmp/a11_code.txt /tmp/b11_code.txt
# exit 0 -- 327 lines, byte-for-byte identical
```

Task A11's own deliverable is the FULLY JOINED Part-B flow: it calls `composerKeySources`,
`composerCardSources`, `composerSeatFlow`, `composerShortfall`, `composerMappingReview`,
`composerConsentFlow`, `composerFormPick`, `composerMintCards`, `composerEngraveModePick`,
`composerSecretCards`, `composerCensusLines` -- every one a **Part B symbol**, not declared until
Tasks B1/B4/B5/B6/B7/B8/B9/B11 land.

- **Confirmed fold-introduced, not pre-existing.** `git show 39f381b` shows Task A11's pre-fold body
  was genuinely self-contained (99 lines, `md.Compose(st.list)` called directly, ending at
  `composerEngraveTemplate`), headed *"PART A's VERSION: shape, stub screen, consent, keyless
  template engrave."* The fold's own diff hunk (`@@ -4847,99 +5225,330 @@`) deletes exactly this and
  substitutes the joined body -- evidently a copy-paste of Task B11's replacement landing in the
  wrong task's fence during the fold.
- **Task A11's own gate cannot pass as sequenced.** Its own Step 6/7 ("Run: every `TestComposer*`
  PASS" / "gofmt, commit -- **PART A IS SHIPPABLE HERE**") describes a milestone that, built from
  Task A11's literal fence alone with none of Part B's fences present, would fail to *compile*
  (`undefined: composerKeySources`, etc.) -- and the plan's own required workflow
  (`superpowers:subagent-driven-development`) is exactly the task-by-task sequencing this assumes.
- **Task A11's own commit message and Interfaces line are now false for what they stage.** The
  message still says Part A's exit is "no payload, a shape, the stub screen, consent, and a
  template" -- the staged code seats keys, self-checks, offers a form choice, mints cards and shows a
  census, none of which the message names. The task's own "Interfaces > Produces" line (plan ~4762)
  still lists `func composerEngraveTemplate(...) bool`, a function that no longer exists anywhere in
  the plan (confirmed: zero other matches).
- **Compounding, same root cause.** The old, weak walk test fidelity I-11 originally criticized
  ("the test's name promises the walk; its body asserts only that the door drew") --
  `TestComposerNoPayloadWalkReachesAKeylessTemplateThatDecodes` -- is still present, **unchanged**,
  inside Task A11 (plan line 4946), sitting beside the new, correct walk test
  (`TestComposerNoPayloadWalkEngravesAKeylessTemplate`, Task B11, plan line ~9328) that actually
  discharges section 12 item 3. Its own doc comment, unedited, still claims the walk "composes a
  shape, reads the stub screen ... consents, and engraves a keyless template whose md1 DECODES" --
  exactly the overclaim I-11 flagged. Worse, the *new* test's own doc comment says *"The test that
  carried this name before asserted only that the door drew"* -- which is false as literally read: no
  test was renamed; the old, differently-named test is still sitting in the tree making the same
  claim it always made. The fold's response to I-11 added a good test elsewhere; it never touched,
  retired, or even mentions the old one's continued presence.

**Impact.** The FINAL assembled tree (all fences unioned, Replace-wins, which is how
`scripts/plan-build-gate-go.sh` and every wired-copy test in this review actually built it) is
genuinely correct and green -- that result stands, independently reconfirmed throughout this report.
What is broken is the plan's own claim about its *intermediate* state: an implementer following Task
A11 to the letter, in the order the plan itself prescribes, cannot reach the "Part A is shippable
here" milestone the plan promises there, and nothing in the plan flags the contradiction between its
own architecture prose (Part A "ships alone") and its own code fence three thousand lines later. The
build gate cannot see this class: a whole-document, Replace-wins extraction can only ever see the
FINAL state of a file, never "what compiles after only Task A11 has landed" -- the gate's stated
blind spot ("does not assemble fragments ... still needs a reviewer's execution pass") should be read
as covering incremental buildability too, and currently does not say so.

**Severity: Critical.** One of the plan's three named, load-bearing operator-question defaults ("Part
A ships alone") is falsified by the artifact meant to deliver it.

**Fix shape** (not prescriptive): restore Task A11's `composerFlow` fence to its pre-fold,
self-contained body; correct the stale "Produces" line and commit message; retire or rename
`TestComposerNoPayloadWalkReachesAKeylessTemplateThatDecodes` so it does not sit unaddressed beside
its replacement.

---

## 3. Every Important -- VERIFIED / NOT VERIFIED

### Journey lens (I-1 ... I-6)

| # | Finding | Plan line(s) | Verdict | Evidence |
| - | - | - | - | - |
| I-1 | Section 8j guard blocks lock/hash edits section 7g rules DEFAULT | `composerPathEdit` ~3296-3334 (Task A5) | **VERIFIED, code correct, zero regression guard** | `composerShapeGuard` called only on the Keys/Remove/Move arms, confirmed against the code; Time-lock and Hash-lock arms unguarded. Mutation (added the guard to the Time-lock arm): full suite still `ok` -- no test drives a lock/hash edit through the UI to check the discard warning does *not* fire. |
| I-2 | Section 4f invariant fires on unseated slots | `composer_review.go` (B4), `composer_flow.go` `composerSizeAssignments` (B11) | **VERIFIED, code correct, zero regression guard** | `composerInvariantViolation` skips `src<0`; `composerSizeAssignments` sizes at flow entry. Mutation (removed the `src<0` skip): full suite still `ok` in both a targeted and whole-package run -- no test exercises >=2 unseated slots with no other collision, so section 8p's own legal partially-seated fallback has no regression guard. |
| I-3 | Pick list can take a row the operator cannot see | `composer_paged.go` `composerPickScreen` (Task A2) | **VERIFIED** | Both-direction clamp into `[start, start+shown)` present. Guard: `TestComposerPickScreenNeverReturnsARowItDidNotDraw`. Mutation (removed downward clamp) -> **FAIL**: `"the pick screen returned row 19 ..., which was NOT on the frame the operator confirmed."` Reverted, clean. |
| I-4 | Consent confirmable before its proof is drawn | `composer_paged.go` `composerReadScreen` (Task A2) | **VERIFIED** | `seenEnd` gate withholds the checkmark until `start+shown >= len(lines)` has held once. Guard: `TestComposerReadScreenWithholdsTheCheckmarkUntilTheLastPage`. Mutation (dropped `seenEnd &&`) -> **FAIL**: `"a 64-line consent was confirmed from its FIRST page; section 7e's addresses are the only proof of which wallet this is, and they are pages in."` Reverted, clean. |
| I-5 | Back at Key order destroys the path's key set | `composer_shape.go` (Task A5) | **VERIFIED, code correct, mutation currently a no-op** | Snapshot/restore code and `composerPathLine`'s "empty" body both present, matching the plan. Mutation (removed restore-on-decline) -> full suite still `ok`, **because** the C-2 fold independently moved the sorted question OUT of `composerKeysEdit` entirely -- `composerKeysEdit` no longer writes `Keys = nil` on any decline path, so the specific mechanism this fix guards against no longer exists in the current call graph. The restore code is correct and harmless, but its own regression case is currently unreachable; no test constructs one via any other path either. |
| I-6 | Date past 2038-01-19 refused as "does not exist" | `composer_lock.go` (A1/A7) | **NOT VERIFIED -- live defect, independently confirmed twice** | `composerDateToUnix` returns `(0, false)` on **every** failure path (impossible calendar date, pre-2009, and past-ceiling all return literal `0`). The dispatch that is supposed to pick the ceiling message reads `if y > 2038 \|\| u == 0`, and inside the `!inBand` branch `u` is *always* 0 -- so `y > 2038 \|\| u == 0` is a tautology, true for every failure. **The final `return "that date does not exist", false` line is dead code.** Concretely: `2027-02-31` (an impossible calendar date, well inside 2009-2038) now gets **the ceiling message** ("This build writes dates up to 2038-01-19 ... use a block height instead") -- directly contradicting the fold's own claim that this body fires "ONLY for the ceiling case," and giving nonsensical remediation advice for a date that will never exist regardless of height vs. time lock. `composerLockEdit` has zero test callers anywhere (only the pure `composerDateToUnix`/`composerParseDateDigits` are tested, and only for their `inBand` boolean, never for which *message* the UI shows), so nothing in the shipped suite catches this. Confirmed independently in two separate sub-worker passes plus my own direct read of the source. F-456 is filed but tracks only the missing section 8 spec body, not this dispatch defect -- recommend a new follow-up. |

### Fidelity lens (I-1 ... I-12)

| # | Finding | Plan line(s) | Verdict | Evidence |
| - | - | - | - | - |
| I-1 | Section 4f invariant checked on UI state | same fix as journey I-2 | **VERIFIED** (not re-mutated, same code) | -- |
| I-2 | Consent mis-numbers tr paths with an extracted internal key | `composerConsentLinesFor` (Task A11/C0) | **NOT VERIFIED -- fix never reaches production** | `composerConsentLinesFor(chunks, listed, keyPathNo)` is correct in isolation and `TestComposerConsentNumbersPathsAsTheOperatorListedThem` passes. But the **only** call site reached from the live UI, `composerConsentFlow` -> `composerConsentLines(chunks)`, is a thin wrapper that hardcodes `composerConsentLinesFor(chunks, nil, 0)` -- confirmed by reading both functions. `composerConsentLinesFor` is called with a real, non-nil `listed` from exactly one place in the whole tree: its own unit test. A live probe calling exactly what `composerConsentFlow` calls, on a tr fixture with an extracted internal key, printed `"Path 1: 2-of-3"` for the operator's Path 2 -- the original defect, unmitigated, on the surface whose entire job is proving which wallet the operator is consenting to. This is the one Important that is not merely undertested; its fix is dead code from the real flow's point of view. |
| I-3 | Back at the path list abandons the composition | `composer_flow.go` (B11) | **VERIFIED** | Decline re-invokes `composerWrapperPick` and `continue`s with `st.list` intact, not `return`. Guard: `TestComposerBackAtThePathListKeepsTheComposition`. Mutation (reverted to bare `return`) -> **FAIL** with the expected message. Reverted, clean. |
| I-4 | Wrapper cannot be changed after the first pick | `composer_shape.go`, the "Change the script" row (Task A5) | **NOT VERIFIED -- real call site, zero test coverage** | The row exists, routes through `composerShapeGuard`/`composerApplyShapeEdit`, and calls `composerWrapperPick` again -- correctly wired on inspection. But the string "Change the script" appears in **no** `_test.go` file anywhere in the tree. Deleting the row and its case-arm entirely leaves the full suite `ok`. Section 12 item 4's named wrapper-change-after-seating vector is reachable in the UI but exercised by nothing. |
| I-5 | `composerKeysEdit` destroys an existing key set | same fix as journey I-5 | **VERIFIED** (not re-mutated) | -- |
| I-6 | Section 8a/8b memoised by path index | `composerState` (no memo fields at all now) | **VERIFIED** | Guard: `TestComposerKeylessConfirmFiresAgainForANewPathAtAReusedIndex`. Mutation (reintroduced a per-index memo, set unconditionally on confirm-entry) -> **FAIL** as expected. Reverted, clean. |
| I-7 | Pager-gate test cannot fail for the reason it states | Task C0 | **Partially verified -- fold's own record of its fix is wrong** | The new `TestComposerReadScreenWithholdsTheCheckmarkUntilTheLastPage` is a genuine behavioral test (no ink comparison) and correctly catches a `seenEnd`-guard removal. But the fold's own per-finding table says "the ink comparison is gone" -- it is **not**: `TestComposerReadScreenDrawsThePagerOnlyWhenASecondPageExists` (the exact `longInk <= shortInk` test I-7 named) is still present, unmodified, in `composer_paged_test.go`, and still passes against the very defect I-7 described. Harmless in practice (superseded by the good test) but the fold's claim about its own fold is inaccurate. |
| I-8 | Section 12 item 5's gates missing for section 8m/8c/8r | Task C0 | **Substantially, not fully, closed** | All 18 bodies (5 section 8m + 7 section 8c + 6 section 8r) now have a modal-fits-class assertion. Fires-on-condition, screen-driven: section 8c 7/7 closed; section 8r 5/6 (`composerCopyPayloadNotLoaded`'s `inFlash=true` branch is never rendered by any test, only its setter exists in production); **section 8m 1/5** -- only the "key-less under tr" refusal is driven through a real rendered frame; the other 4 of 5 structural refusal bodies have a modal-fits assertion but no screen-level fires-on-condition test. Mutation (removed the `showError` call for the slot-cap refusal entirely) leaves the full suite `ok`, confirming 4/5 section 8m bodies have zero screen-level guard. |
| I-9 | Section 8i's "and at consent" half absent | `composerConsentLinesFor` (Task A11) | **NOT VERIFIED -- fix present, zero regression test** | `composerCopyHashRule()` is correctly appended whenever a branch carries a digest. Deleting the whole block leaves the full suite `ok` -- zero tests fail. `TestComposerConsentLinesDescribeEveryPathFromTheDecodedMd1` builds a hash-bearing path but never checks the output for the hash-rule text. |
| I-10 | Form A/secret plate no builder; "cut ONCE" unimplemented | `composerSecretCards` (B11) | **VERIFIED** | Dedups by `seen[src.seedID]`. Guard: `TestComposerSecretIsCutOnceForASeedThatFilledSeveralSlots`. Mutation (removed the dedup) -> **FAIL**: `"one seed at three slots produced 3 secret plate(s), want 1."` Reverted twice in two independent isolated copies, both confirming the same result. `F-457` is filed with the stated reasoning (`md` emits no descriptor text, a renderer is Rust-first); `composerCensusRefusal` is fully deleted, not dangling. The scope split is honest: cut-once ships for the form actually built (ms1 bundle cards); form A/text/QR is declined and filed, not silently dropped. |
| I-11 | Part A's declared exit not discharged | `TestComposerNoPayloadWalkEngravesAKeylessTemplate` (B11) | **VERIFIED, with the caveat in section 2 above** | The test PASSES and directly asserts 5 of section 12 item 3's 6 clauses (door line, shape built, stub with per-slot origins, consent stating no addresses, form choice collapsed with reason). The 6th (md1 decodes) is proven only by the separate, pre-existing `TestComposerKeylessTemplateDecodesOnTheDevice`, exactly as before the fold -- the fold table's "walks all six of section 12 item 3's clauses" overstates by one clause. See section 2 for the more serious, related finding: the OLD, criticized walk test this one was meant to succeed is still present unchanged. |
| I-12 | Blast radius covers Go tests only | Task C2 | **VERIFIED** | All three emulator files and `capture_walletpolicy.py` named with a concrete one-line diff and a build-and-count check; the plan states plainly no gate in this stage can *run* them (needs a browser and playwright). Read Task C2's text directly, not summarized from the plan's own claim. |

### Tests lens (its one Important)

| # | Finding | Verdict | Evidence |
| - | - | - |
| I-1 | `composerHexEntry`'s exact-64 bound has no real test | **NOT VERIFIED** | `TestComposerHexEntryTakesExactlySixtyFourCharacters` (Task C0) does **not** call `composerHexEntry` -- it reimplements the bound inline (`valid := len(frag) == 64`) against a bare `hex.DecodeString` wrapper, tested at boundary values 0/62/63/64/65 (62 deliberately chosen as an even-length near-miss, closing the *odd*-length accidental-safety-net the original review flagged -- a real methodological improvement in the test's OWN logic). But `composerHexEntry` itself has exactly one caller anywhere (`composerHashEdit`, production) and zero test callers. Mutation on the real function (`valid := len(frag) >= 63`) -- full suite stays green, 0 FAIL, including the new test, which never touches the mutated code. |

**Section 3 closing count:** of 19 Importants, **9 fully VERIFIED with a working regression guard**
(journey I-3, I-4; fidelity I-1, I-3, I-5, I-6, I-10, I-11, I-12), **6 have a correct production fix
but no test that fails if it regresses** (journey I-1, I-2, I-5; fidelity I-4, I-9; tests I-1),
**1 is substantially but not fully closed** (fidelity I-8: 14 of 18 bodies have complete
fires-on-condition coverage; 4 of the 5 section 8m structural refusals do not), **1 is verified
working but the fold's own account of its own fix is inaccurate** (fidelity I-7: the new test is
genuinely good, but the old, flawed `longInk<=shortInk` test the fold's table claims is "gone" is
still present, unmodified), **1 is NOT VERIFIED because its fix never reaches production** (fidelity
I-2), and **1 is NOT VERIFIED because the fix itself is defective**, independent of test coverage
(journey I-6). 9 + 6 + 1 + 1 + 1 + 1 = 19, accounting for every Important.

---

## 4. Part B expansion -- every task at the step standard

**Stub-phrase sweep:** `grep -n -i 'as the other tasks do\|similar to task\|TBD\|TODO'` returns
exactly one hit, and it is the plan's own closing negation of the class ("No TBD, no TODO, and no
step that says 'as above'"). Clean. (The phrase "as the other tasks do" *did* exist pre-fold, inside
Task A10's own Step 4 -- `git show 39f381b` confirms it -- and the fold's Part B expansion is what
replaced it with a real Run:/Expected: pair. This is the fold correctly closing a real gap, not a
false-clean grep.)

**Checkbox/Run/Expected structure, measured per task:**

| Task | Lines | Checkboxes | `Run:` | `Expected:` |
| - | - | - | - | - |
| A10 | 4681-5257 | 12 | 4 | 6 |
| B1 | 5633-6126 | 6 | 2 | 2 |
| B2 | 6126-6396 | 3 | 1 | 1 |
| B3 | 6396-6618 | 5 | 2 | 2 |
| B4 | 6618-7052 | 5 | 2 | 2 |
| B5 | 7052-7306 | 3 | 1 | 1 |
| B6 | 7306-7708 | 3 | 1 | 1 |
| B7 | 7708-8026 | 5 | 2 | 2 |
| B8 | 8026-8393 | 5 | 2 | 2 |
| B9 | 8393-8579 | 3 | 1 | 1 |
| B10 | 8579-8994 | 4 | 2 | 2 |
| B11 | 8994-9947 | 6 | 3 | 3 |
| C0 | 9947-10650 | 4 | 2 | 2 |
| C1 | 10650-10802 | 4 | 1 | 2 |
| C2 | 10802-10893 | 6 | 8 | 8 |

Every task has non-zero, observable Run:/Expected: pairs -- but **not every task reaches its own
declared conclusion.** Tasks B5, B6 and B9 each stop after their implementation code block with no
closing "Step N: Run the tests" / "Step N+1: gofmt, commit" section at all -- verified by reading
past the last code fence in each task straight into the next task's header, in all three cases.
Every *other* task in the plan (A1-A11, B1-B4, B7-B8, B10-B11, C0-C2) explicitly names a `Run:` +
`Expected:` pair for its *own* new tests before a `gofmt, commit` block. B5/B6/B9's own tests
(`composer_seat_test.go` 2 funcs, `composer_selfcheck_test.go` 3 funcs, `composer_census_test.go` 2
funcs) do get exercised -- they appear inside the whole-suite counts verified in section 5 below --
but only via later tasks' (B11, C2) aggregate gates, never via a local, task-scoped
verification+commit step of their own. This is a real, if Minor, inconsistency in "brought to the
plan's own step standard" for 3 of 15 Part B/C tasks, not present anywhere in Part A.

Task B10 specifically (the tests lens's C-3, "five prose bullets with no code") now carries a real
361-line `gui/composer_seating_vectors_test.go` with 4 test functions, seating cards through the
**shipped** `seatKeyCards` consumer and reproducing real addresses. Re-run directly, all 4 PASS
(`TestComposerMintedCardsSeatThroughTheShippedSeater`,
`TestComposerSeatedTemplateReproducesTheKeyedPolicysAddresses`,
`TestComposerPartiallySeatedArtifactIsANamedVector`,
`TestComposerNeverProducesTheAsymmetricOneCardTemplate`). C-3 is closed.

**Expected-line spot check (>=10 required; 14 run, spread across A10/B1/B2/B3/B4/B7/B8/B10/B11/C0/C2,
plus the five doc-level gates):**

| Task/line | Expected | Actual, re-run | Verdict |
| - | - | - | - |
| B1 (6102) | consumption sites "at least 3 higher" | 15 reconciled, re-run and matches the author report | TRUE |
| B2 (6375) | "three PASS" | **4 PASS** -- the `-run '^TestComposerSeed'` filter also matches `TestComposerSeedInAPayloadStillRaisesF1AtLoad`, a different task's (A3's) test whose name happens to share the prefix; `composer_seed_test.go` itself has exactly 3 functions | **FALSE** -- miscounted, pre-existing (present at `39f381b` too, not fold-introduced) |
| B3 (6595) | "three PASS" (named) | 3 PASS, exact name match | TRUE |
| B4 (7028) | "four top-level, five sub-tests" | 4 top-level, 5 sub, re-run | TRUE |
| B7 (8002) | "four top-level, three sub-tests" | 4 top-level, 3 sub, re-run | TRUE |
| B8 (8368) | "four PASS" (named) | all 4 named tests PASS, re-run | TRUE |
| B10 (8969) | "four top-level, five sub each x2" | exact match, re-run | TRUE |
| B11 (9914) | "five PASS ... two named exemptions" | **6 PASS** -- the `^TestComposerNoPayloadWalk` fragment matches both the old A11 test (section 2 above) and the new B11 test; and only **one** exemption actually logs (`composerDescriptorCeilingChars`) since `composerDescriptorPlateFits` now has a real caller and is never flagged as orphaned in the first place -- the exempt *map* holds two entries by design (defensive, in case that changes), but the *observed output* names only one | **FALSE** -- miscounted, both halves, though both are explained by adjacent prose in the plan's own gate-coverage section |
| C0 (10627) | "12" | **14** -- `^TestComposerSection8` alone matches 3 distinct top-level test functions, not 1 | **FALSE** -- miscounted |
| C2 (10809/10812) | "92 top-level PASS, 0 FAIL" / "77 sub-tests" | **100 PASS, 0 FAIL** / **81 sub-tests**, re-run three separate times, matches the fold commit message's own re-gate figures exactly | **FALSE** -- see section 5c below, this one is fold-introduced, not pre-existing |
| C2 (10817) | "1158 tests across 24 shards" | 1158, `gui-shard-test.sh` re-run, 32s wall | TRUE |
| Staleness | 0 drifted vs `321acb56` | 0 drifted, `plan-staleness-check.sh` re-run (142 unchanged, 4 not-in-repo without a path filter; with a `gui/` path filter, 113/0/2 -- the difference is the filter, not a discrepancy) | TRUE |
| Cite/glyph/table/stepref | 238/238; 249/0; 89/0; 0 | all re-run, exact match | TRUE |

**5 of 14 checked Expected: lines are numerically wrong, all under-counts** -- a reviewer following
the plan literally would see a bigger number than promised and know at once something moved, not a
smaller one hiding a missing test. Root cause in every case is a `-run` regex fragment that, in the
final assembled document, nets more distinct top-level test functions than were present when the
line was written (a prefix shared with an earlier task's test, or one prefix later covering several
same-family tests added by a subsequent fold). One of the five (C2, section 5c) is fold-introduced;
the other four (B2, B11, C0, and B11's exemption-count line) predate the fold or are self-explained
by adjacent prose. Severity: **Minor**, pattern-level, no hidden defect.

---

## 5. New-defect sweep

### 5a. Machine-verified "already-settled" facts, re-checked

Per the build-gate-before-review discipline, every numeric claim the brief named as settled was
re-derived rather than trusted:

| Claim | Claimed (commit message / plan) | Re-derived | Match? |
| - | - | - | - |
| `plan-build-gate-go.sh` extraction | 41 fences | **47 fences read, 1 dropped by Task B11's Replace (section 2's duplication), 46 kept in the final files, 43 files** -- ran the script's own extraction Python verbatim (mechanically sliced from `scripts/plan-build-gate-go.sh` lines 69-104 with `sed`, zero manual retyping, so no transcription risk) | **no** -- an earlier pass of this report also said "41" here without re-deriving it; corrected on a third, byte-exact run |
| `go test -run '^TestComposer' ./gui/` | 100 top-level PASS, 81 sub-tests, 0 FAIL | 100 / 81 / 0, re-run three separate times across three copies | yes |
| `gui-shard-test.sh ./gui/ 24` | ok, 1158 tests | ok, 1158 tests, re-run, 32s wall | yes |
| `go test ./md/ ./mk/ ./sysw/` | ok x3 | ok x3, re-run | yes |
| `go vet ./gui/` | clean but for two pre-existing go1.25 `ArtifactDir` findings | exactly those two, nothing else | yes |
| `plan-cite-check.sh` | 238/238 resolved, 0 dangling | 238/238, 0 dangling, 0 ambiguous, re-run | yes |
| `plan-glyph-check.sh` | 249 strings, 0 undrawable | 249 / 0, re-run | yes |
| `plan-table-check.sh` | 89 rows, 0 malformed | 89 / 0, re-run | yes |
| `plan-stepref-check.sh` | 0 | 0, re-run | yes |
| `plan-staleness-check.sh <plan> <fork> 321acb56` | 142 unchanged, 0 drifted | 142 / 0, re-run | yes |
| DEAD-IN-PROD | 1 (`composerDescriptorCeilingChars`) | 1, independently re-swept over all 138 declarations | yes |
| Composer suite top-level PASS (Task C2's OWN Expected line) | 92 | **100** | NO -- see 5c |
| Composer suite sub-tests (Task C2's OWN Expected line) | 77 | **81** | NO -- see 5c |

Ten of the eleven numbers the fold commit message itself claims about the *gate* are correct and
independently reproduced; the fence-extraction count is not (41 claimed, 46 kept / 47 read actual).
The two further wrong numbers (Composer suite PASS/sub-test counts) are inside the plan's own Task C2
text rather than the commit message, addressed next in section 5c -- they are not part of the
"already-settled, do not re-derive" set the brief named, but they surfaced during the Part B
expansion check (section 4) and belong here as a fold-introduced defect, not a pre-existing one.

### 5b. See section 2 above (the new Critical: Task A11's duplicated fence)

### 5c. NEW (Important) -- Task C2's own headline test counts are stale, and the staleness was introduced by the fold

Pre-fold (`git show 39f381b`), Task C2 Step 1's Expected line read: *"a non-zero PASS count and `0`
FAIL"* -- vague, always-true-if-green, safe. **The fold's own diff** (hunk `@@ -7568,39 +10805,57
@@`) replaced this with a specific, hardcoded claim: *"**`92`** top-level PASS and **`0`** FAIL"*
plus a new second line, *"**`77`** sub-tests."* Both hardcoded numbers are wrong against the current
wired tree (100 and 81, confirmed three times independently, matching the fold commit message's own,
separately-stated re-gate figures exactly). The fold turned a claim that could never be false into
one that is currently false. Harmless in the sense that the actual suite is green and larger than
promised, not smaller or broken -- but it is a defect the fold introduced while doing something else
(reformatting Task C2's Run:/Expected: structure), exactly the class the build-gate-before-review
rule exists to catch before a reviewer sees it. **Severity: Important** (a new defect from the fold,
per the brief's own severity rubric; not Critical, since nothing it guards is actually wrong -- the
suite is more green than claimed, not less).

### 5d. Hunk-to-finding mapping

56 hunks in `git diff 39f381b..3820a6a -- design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`. Every
hunk before diff-line ~2050 maps cleanly to a named finding: STATUS/baseline update -> M-10;
`composerCopyDateCeiling` -> journey I-6; `composerPageLines`/`composerReadScreen`/
`composerPickScreen` -> I-3/I-4/M-11; `composerState` memo-field removal -> fidelity I-6;
`composerPathLine` "empty" body -> journey I-5/M-4; `composerSlotsKeysLine` -> N-2;
`composerRefusalBody` -> M-3; `composerKeysEdit`/`composerAddPath`/`composerPathEdit` -> C-2 + I-1 +
N-3; `composerLockEdit` -> I-6/M-2/N-4 (see journey I-6 above for a residual defect inside this
otherwise-correctly-scoped hunk); `composerHashEdit` -> M-6; `composerBranchLines`/
`composerConsentLines` -> I-2(fidelity)/I-9/M-5; `composerSeedAccountFor` rename -> M-8 (see
section 5f below for a residual documentation gap inside this correctly-scoped hunk). The large hunk
from diff-line ~2050 to ~4354 is the Part B expansion (Task B11 + Task C0, ~2200 new lines) --
legitimately new content, mapped task-by-task in sections 1/3/4 above. **The one exception**: the
hunk touching Task A11's own `composerFlow` fence (section 2 above) is a misplaced duplicate of Task
B11's content, not a fix mapped to any finding, and Task C2's headline-count hunk (section 5c above)
introduces a new, unmapped defect while reformatting.

### 5e. Operator-question defaults -- consistent

"Part A ships alone" / section 7f's two real forms (F-455) / Task A10 blocked on F-453 are stated
identically in the Global Constraints bullet (plan line 39) and repeated consistently everywhere
they recur (I-11's fold entry, Task A10's own precondition text, Tasks B7/B9's F-455/F-457
citations, Task C2's "what this stage does NOT do" closing list) -- grepped across roughly 15
locations, no drift found in wording. (Section 2 above is a separate, deeper problem: the *code*
falsifies default (a), the *prose* about it is consistent throughout.)

### 5f. NEW (Minor) -- two `Interfaces > Produces` lines the fold's own table claims were fixed, but weren't

The fold's per-finding table claims two Minor fixes each corrected a stale interface signature:

- **M-7** (`Task B9: the Interfaces line matches the code`). It does not. Task B9's own "Produces"
  line (plan ~8401) still reads
  `func composerCensusLines(pl Platform, cards []bundleCard, descriptor string) ([]string, error)`
  -- three parameters, returns `([]string, error)`. The actual shipped function
  (`composer_census.go`) is `func composerCensusLines(params engrave.Params, cards []bundleCard)
  []string` -- two parameters, no `descriptor`, no error return. Confirmed by reading both directly.
- **M-8** (`Task B2: composerSeedAccountFor renamed to srcIdx, with the collision named`). The
  *code* fix is real and correct -- `gui/composer_sources.go:291` is
  `func composerSeedAccountFor(st *composerState, slot uint8, srcIdx int) uint32`. But Task B2's own
  "Produces" line (plan ~6134) still reads `func composerSeedAccountFor(st *composerState, slot
  uint8, seedID int) uint32` -- the old parameter name the fix was explicitly about renaming.

Both are documentation-only (the code is correct in both cases; only the plan's own interface
inventory is stale), and neither affects a test or a build gate. **Severity: Minor** -- recorded
because the fold's own table specifically claims these two lines were brought into agreement with the
code, and for both, they were not.

### 5g. STATUS/Baselines -- current

STATUS line correctly states `S2 HAS MERGED: fork main is 321acb56`; Baselines line names
`321acb56` and the five re-resolved citations (`gui/multisig_build_slots.go:125->128,:172->175,
:238->241`; `md/policy_shape.go:60->53,:74->87`); confirmed `seedhammer`'s `main` is at `321acb56`
and `descriptor-mnemonic`'s HEAD matches the pinned `66bdf2f4`.

### 5h. NEW (Nit) -- a decline's promised destination does not exist

Fidelity N-1 (section 8r "beneath Build" vs. the `ChoiceScreen.Lead`) was declined as a code change
with an explicit destination: the author's section 9 table says *"recorded for the section 13 fold
in Task C1"* (plan line ~135 carries the identical sentence). **Task C1's own "Step 3: Fold the
three spec changes" (plan lines ~10672-10692) folds exactly three things into the spec -- section 13
item 1's four numbers, section 6a's flag-screen sentence, and section 7f's secret-plate-form
sentence.** None of the three is section 7a or the Lead-vs-rows question N-1 raises; grepped, no
other line in Task C1, or anywhere else in the plan, mentions "beneath Build" or proposes a section
7a wording change. The decline was never wrong on its merits -- the reviewer's own note ("the Lead
wraps and the rows do not... which the reviewer calls sound") still holds, and no operator-facing
behavior is at stake -- but the promise to carry it into a specific task went unfulfilled, and
nothing else in the plan carries it either. **Severity: Nit** (matching N-1's original severity) --
"declined and recorded elsewhere" should mean the elsewhere actually has it; here it does not.

---

## 6. Tests lens's 15 Criticals -- mutation re-run, categories 1-9

Every mutation the tests-lens report describes was re-applied against the folded, wired tree and
reverted, `git diff`-bracketed before and after every run.

| Mutation | What was mutated | Pre-fold | Post-fold | Guard |
| - | - | - | - | - |
| 1a | Door key-state line forced to 0 | CAUGHT | **CAUGHT** (re-confirmed) | `TestComposerDoorLinesCoverEveryKeyState` |
| 1b -> C-4 | Drop `ClassMDMK` branch of `composerDoorHasConsumablePolicy` | NONE | **CAUGHT** | `TestComposerDoorOffersFromPayloadForACardPayload` (C0) |
| 1c -> C-5 | Route "Build a new policy" to Scan instead of Build | NONE | **CAUGHT** | Both flow-level walks (B11) |
| 2a | Picker allows a 33rd slot | CAUGHT | **CAUGHT** (re-confirmed) | `TestComposerPickerBoundsNeverOfferAnIllegalValue` |
| 2b/2c -> C-6 | Disable Done-branch `md.ValidatePathList` gate | NONE | **CAUGHT** | `TestComposerShapeRefusalGateIsReachedFromTheScreen` (C0) |
| 2d | Make Back drop the path list | NONE (unverified at baseline) | **CAUGHT** | `TestComposerBackAtThePathListKeepsTheComposition` (B11) |
| 2e -> C-7 | Section 8a confirm auto-accepts | NONE | **CAUGHT** | `TestComposerKeylessConfirmFiresAgainForANewPathAtAReusedIndex` |
| 3a-g -> C-8 | Disable `composerLockAccept`'s two gates | NONE | **CAUGHT** | `TestComposerLockAcceptRefusesFromTheScreen` (4 sub-cases, C0) |
| 4a -> I-1 | Accept 63 hex chars, on the real function | Not cleanly constructible / Important, open | **STILL OPEN** | See section 3 above -- new test doesn't call the real function |
| 4b | Show 64-hex row unelided | CAUGHT | **CAUGHT** (re-confirmed) | `TestComposerHashRowIsShortEnoughToDraw` |
| 5a -> C-9 | Force `changed` always false (skip section 8s re-show) | NONE | **STILL NONE** | Independently re-confirmed twice (two sub-workers, plus a third direct run in an isolated copy): full suite `ok`, and the named guard `TestComposerStubReshowSignalIsTheChunkSet` also stays PASS, because it recomputes the comparison standalone rather than exercising `composerFlow` itself |
| 5b -> C-10 | Disable `k.FingerprintPresent` branch | NONE | **CAUGHT** | `TestComposerStubLinesLabelASeatedSlot` |
| 6a | Seat the same card twice | NONE (0 callers) | **CAUGHT** | `TestComposerWalkFromAKeyedPayloadReachesTheEngraveScreen` (trips the mapping review's duplicate-xpub refusal) |
| 6b | `composerApplyShapeEdit`'s discard nullified at runtime (call kept in source) | NONE (0 callers) | **STILL NONE** | The "Change the script" call site now exists but nothing drives an actual wrapper change through it -- the same gap as fidelity I-4, found independently by two sub-workers from two different angles |
| 6c-screen -> C-11 | Disable invariant check at `composerMappingReview`'s call site | NONE | **CAUGHT**, independently reconfirmed directly | `TestComposerMappingReviewRefusesFromTheScreen` (C0): `"the mapping review did not refuse section 4f's invariant violation"` |
| 6d-screen -> C-12 | Swap `composerShortfall`'s count-value argument to `len(st.sources)` (call site kept, faithful mutation) | NONE | **STILL NONE -- false PASS**, independently reconfirmed directly | `TestComposerShortfallCountsSeatsFromTheScreen`'s fixture uses two plain-`key:` sources against 4 slots, where `len(st.sources)` and `composerAssignableSlots(st)` both equal 2 -- the fixture is structurally incapable of distinguishing the two counting rules. A first, cruder mutation (deleting the call to `composerAssignableSlots` outright) was falsely "caught" only by the reachability scan noticing the callee vanish; re-run with the call kept and its effect nullified, the whole suite stays `ok`. This is the funds-relevant property the mutation targets (section 8p's "keys available" count), still genuinely unguarded. |
| 6e -> C-13 | Drop the C29 warning loop from `composerMappingLines`'s output | NONE | **CAUGHT**, independently reconfirmed directly | `TestComposerMappingReviewRefusesFromTheScreen`'s second half, which calls `composerMappingLines` on a same-seed-same-path fixture and asserts `"SAME SEED, SAME PATH"` is present -- fails correctly on the mutant. (Two *other*, narrower tests -- `TestComposerMappingLinesPrintOriginsVerbatimAndSayWhatIsNotChecked` and `TestComposerC29WarningFiresInsideOnePathAndNotAcross` -- do NOT check for this and stay green under the same mutation; they are not the guard, but a real guard exists elsewhere.) |
| 6f | Number the tr internal-key slot last | CAUGHT | **CAUGHT** (re-confirmed) | `TestComposerSlotOrderAgreesWithTheCodec` |
| 7a | Skip the self-check | CAUGHT | **CAUGHT** (re-confirmed) | `TestComposerSelfCheckRefusesAFaultInjectedBuilderOutput` |
| 7b | Compare UI state instead of decoded md1 | CAUGHT | **CAUGHT** (re-confirmed) | `TestComposerConsentRefusesThroughTheHookAndSaysSection8q` |
| 7c | Render lock kind from UI state | Not constructible | **Still not constructible** | `composerConsentLines` takes no UI-state parameter -- confirmed by signature |
| 7d -> C-14 | Delete section 8l entirely | NONE | **CAUGHT** | Both flow-level walks fail (`"section 8l never drew"` / cannot find its text) |
| 8a | Offer form A for a keyless composition | NONE (no test file, 0 callers) | **CAUGHT** | `TestComposerFormsForOfferWhatSection7fAllows` + the no-payload walk |
| 8b | Mint cards before seating is complete | NONE (no test file) | **CAUGHT** | `TestComposerMintCardCarriesBothStubsAndRoundTrips` |
| 8c | Count plates without card chunks | NONE (0 callers) | **CAUGHT** | Passing `nil` instead of `cards` to `composerCensusLines` at its real call site (`composer_flow.go`'s census-confirm step) fails `TestComposerWalkFromAKeyedPayloadReachesTheEngraveScreen`: `"the census does not count the minted key cards; section 7f says it counts CARD chunks too."` Re-verified a fourth time, from scratch, in a freshly created copy after this row conflicted between two prior passes; this result is what stands. |
| 8d | Cut a multi-slot seed's card twice | NONE (no test file) | **Split -- crude form CAUGHT, the actual finding STILL NONE** | Two mutations, both re-derived from scratch a fourth time after this row conflicted between two prior passes. (a) Unconditionally duplicate every seated slot's card in `composerMintCards`: **CAUGHT** by `TestComposerMintCardsSkipsUnseatedSlotsAndNamesTheRest` (fails "one seated slot of two produced 2 card(s), want 1") -- but that test only ever seats ONE slot, so it catches any universal duplication by accident, not because it exercises the multi-slot case. (b) The mutation the finding actually names -- duplication that fires only from the SECOND seated slot onward (`if len(out) >= 2 { out = append(out, card) }`), invisible to a fixture with one seated slot: **full suite stays `ok`, STILL NONE**. No test in the tree calls `composerMintCards` with both slots of its own two-slot fixture (`composerCardFixture`) seated; the sole caller in `composer_cards_test.go` artificially unseats one of the two before calling it. A prior pass of this report cited `TestComposerSecretIsCutOnceForASeedThatFilledSeveralSlots`/`composerSecretCards`'s dedup as the guard here -- that is a **different function** (the MS1 secret-plate "cut once" rule, fidelity I-10, `composer_flow.go`) from `composer_cards.go`'s MK1 key-card minting this mutation targets, and does not guard it. |
| 8e -> C-15 | Skip the ceiling refusal | NONE -- worse than untested | **Moot -- function removed** | `composerCensusRefusal` no longer exists anywhere (deliberate: F-457 declined, form A ships as keyed md1 only this cycle; confirmed zero references, prod/test/comments) |
| 9 (count) | 39 `composerCopy*` functions | PASS | **40**, `TestComposerCopyTableCoversEveryBody` PASS, `composerCopyDateCeiling` added | Confirmed |
| 9 (1-char) | 1-char body mutation | caught by construction | **CAUGHT** | `TestComposerCopyIsVerbatimFromTheSpec` |

**Note on 8c/8d.** This report went through several hands (see the top-of-file provenance note), and
8c/8d were the two rows where they disagreed. 8c: an earlier pass called it STILL NONE
(`composer_census_test.go` "checks fixed substrings only"); a later pass called it CAUGHT via the
real call site. The final, fourth-run tiebreak (this pass, in a fresh, previously-untouched copy)
confirms **CAUGHT** -- the earlier NONE verdict was checking the wrong surface (the copy-table test)
rather than the real production call site. 8d: a later pass claimed CAUGHT, citing
`TestComposerSecretIsCutOnceForASeedThatFilledSeveralSlots` -- but that guards a *different*
function (`composerSecretCards`, the MS1 secret-plate dedup, fidelity I-10) than the one this
mutation targets (`composerMintCards`, MK1 key-card minting, `composer_cards.go`). Reconstructing
the mutation against the right function and testing it two ways (crude/universal duplication vs. the
narrower, multi-slot-specific form the finding actually names) shows the crude form is caught by
accident and the real finding is not caught at all. **8d is STILL OPEN**, matching the earliest
(pre-fold) reports, not the intermediate claim.

**Closing counts for this section:** of the tests lens's 15 Criticals, **12 are closed** (C-4, C-5,
C-6, C-7, C-8, C-10, C-11, C-13, C-14, plus C-1/C-3 already closed in sections 1/4, plus C-2 closed
on its own literal terms -- Tasks B7/B8's promised test files now exist and are non-empty), **2
remain genuinely open** (**C-9** -- the section 8s re-show signal still untested through
`composerFlow` itself; **C-12** -- the shortfall screen's seats-vs-sources count still
indistinguishable by the one test that exists to guard it), and **1 is moot rather than closed**
(C-15 -- the refusal was removed outright rather than wired up; correct for this cycle's declared
scope, would need re-verifying if F-457's concrete-descriptor plate is ever built). C-2's closure is
narrower than it looks: the specific scenario mutation 8d illustrates (a multi-slot identity's key
card minted twice) remains genuinely unguarded inside the very file C-2 required to exist -- the new
`composer_cards_test.go` has tests, just not one that seats both slots of its own two-slot fixture
and checks the card count. Two more cells that were never numbered Criticals also stay open: **6b**
(a real production call site -- the "Change the script" row -- with no behavioral test exercising
it, the same gap fidelity I-4 names from the UI-affordance side) and **8d** as just described.

---

## Closing counts, overall

- **Both named Criticals** (Part B join; key-order timing): **VERIFIED**, hold under mutation.
- **One new Critical, fold-introduced**: Task A11's own code fence was overwritten with Part B's
  joined body, so "Part A ships alone" -- one of three named operator-question defaults -- is false
  of the plan's own artifact as written. Section 2.
- **Importants (19 total across three lenses)**: 9 fully verified with a working regression guard, 6
  have a correct production fix with zero regression test, 1 (fidelity I-2) does not reach
  production, 1 (journey I-6) has a live logic defect independent of test coverage, and I-8's
  remaining 4/18 section 8m bodies are a partial case. Section 3.
- **Part B expansion**: every task has non-zero checkbox/Run/Expected structure; Task B10 (formerly
  prose-only, tests-lens C-3) now has real, passing, funds-relevant seating vectors. 3 of 15 tasks
  (B5, B6, B9) never reach their own closing Run:/Expected:/commit step, though their tests are
  exercised by later aggregate gates. 5 of 14 spot-checked Expected: lines are numerically wrong
  (all under-counts; one is fold-introduced, see next line; none hides a missing test). Section 4.
- **New-defect sweep**: one new Critical (section 2), one new Important (Task C2's own headline test
  counts, 92/77, are stale against the actual 100/81 -- the fold replaced a safe generic claim with
  a specific wrong one), two new Minors (two `Interfaces > Produces` lines the fold's own table
  claims were fixed but weren't -- `composerCensusLines` in Task B9, `composerSeedAccountFor`'s
  parameter name in Task B2), one new Nit (fidelity N-1's declined-and-recorded destination does not
  actually exist in Task C1, section 5h), operator-question-default prose consistent throughout,
  STATUS/Baselines current. Of the eleven "already-settled" gate figures the brief named, **ten
  independently re-confirmed correct and one did not**: the extraction step's own fence count is 47
  read / 46 kept / 43 files, not the claimed 41 (re-derived three separate times across this report's
  several passes, most recently by piping the exact script lines to `python3` with zero manual
  retyping) -- Section 5a.
- **Tests-lens 15 Criticals**: 12 closed, 2 still genuinely open (C-9, C-12), 1 moot (C-15); two
  further mutation-table cells that were never numbered Criticals also stay open (6b, and 8d once
  reconstructed to match what the finding actually names rather than the crude form that happens to
  trip an unrelated test). Section 6.

**This fold does not close the R0 gate.** It fixed both named Criticals correctly and closed 12 of
the tests lens's other 14 Criticals -- real, substantial progress, independently confirmed at the
code level throughout this report. It also introduced one new Critical (Task A11's duplicated fence)
and one new Important (Task C2's stale counts) while doing so, left one Important's fix unreachable
from production (fidelity I-2), left one Important's fix defective on its own terms (journey I-6),
and left eight fixes (the six-unguarded tally, plus C-9 and C-12's screen-level gaps) with no test
that would catch a regression. Recommend, before the next round: restore Task A11's self-contained
fence; wire `composerConsentLinesFor`'s real `listed`/`keyPathNo` into `composerConsentFlow`
(fidelity I-2); fix the date-ceiling dispatch's `u == 0` tautology (journey I-6) and file it as its
own follow-up distinct from F-456; correct Task C2's headline counts to 100/81 and the two stale
`Interfaces > Produces` lines (B2, B9); and add regression tests for C-9, C-12, and the six
correct-but-unguarded Importants.
