# Composer S3 plan — R0 round 2, fold-verification

Independent reviewer, mechanical lens on fold `e4b3f804861f58a13e92e87ac93fd2f41522bb37`
(pre-fold `3820a6a16663fb4843123fe5f1b8f6cc8ea822c7`) against
`design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`. Question: did the round-1 fold fix every
round-1 finding exactly, and did it introduce a new defect? Not a fresh audit; round-1's own
VERIFIED items were re-verified only where the round-1 fold's diff touches them.

**Verdict: NOT GREEN.** Section 2's Critical (Task A11's fence) is correctly and fully restored,
proven by an independently-run `GATE_UNTIL` gate. Fidelity I-2's fix now reaches production.
Section 4's B5/B6/B9 closing steps and the five under-counted Expected lines are all fixed and
re-measured correct. Section 6's C-12, cell 6b and cell 8d now have real, catching regression
tests. But **three of the round-1 fold's own claimed regression guards are tests that cannot fail
for the defect they name** — journey I-6/F-458's new guard, tests-lens I-1's new guard, and
tests-lens C-9's claimed closure — each independently reproduced under the plan's own named
mutation. One further Important (journey I-5) has a real UI-driven test but the specific
round-1-named mutation remains a structural no-op, matching round-0/round-1's own established
finding (not a new defect). One new Minor: Task C1's own commit-message template still says
"three things" and lists three items after gaining a fourth.

## Method

Two lens copies, made once and mutated/reverted repeatedly (`diff -rq` against the source
confirmed byte-clean after the last revert):

- `/scratch/code/shibboleth/.s3-r2-lens/whole` — `cp -r` of `.plan-build-gate-go-s3/wired`
  (already fully wired: extraction + `handwire_s3.py`, no `--part-a`).
- `/scratch/code/shibboleth/.s3-r2-lens/partA-extract` — `cp -r` of
  `.plan-build-gate-go-s3-partA/seedhammer` (`GATE_UNTIL='^### Task B1'` extraction, NOT yet
  hand-wired), then `python3 handwire_s3.py --part-a` run once against my own copy (first run,
  not a re-run of an already-wired tree).

Go: `/nix/store/i77g9dmcd399rmxk8688qfr4g2wzgk37-go-1.26.7/bin/go`,
`CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local`, `TMPDIR=/scratch/code/shibboleth/.tmp`.

## 1. Section 2 (Critical) — VERIFIED

- **Task A11's fence is self-contained again.** Read the plan text (line 5582 fence) and the
  wired file (`partA-extract/gui/composer_flow.go`, 125 lines): no Part B symbol appears in code
  (only in comments explaining what it does *not* call); `md.Compose(st.list)` is called directly
  (line 56); ends at `composerEngraveTemplate` (defined in the same file). `composer_discard.go`
  is present in the Part-A-only file set (moved from Task B3, per plan line 2157/6692).
- **Task B11 still supplies the join.** Plan line 9486 (`Replace `gui/composer_flow.go`:`) and
  the whole-tree wired file both show the fully joined body (calls `composerKeySources`,
  `composerSeatFlow`, `composerConsentFlow`, `composerMintCards`, etc.).
- **The old weak test is gone.** `grep -c '^func TestComposerNoPayloadWalkEngravesAKeylessTemplate'`
  across both lens trees' `gui/*.go` = 1 each (single definition, in `composer_flow_test.go`,
  Task A11's own file — confirmed by `Files:` line). `TestComposerNoPayloadWalkReachesAKeylessTemplateThatDecodes`
  does not appear anywhere in the current plan text except the round-1 finding table describing
  its deletion. This one test discharges §12 item 3 in Part A and is written to also pass once
  Part B's seating step exists (confirmed: it PASSES in both lens trees).
- **Task A11 carries its own gate step and Expected.** Plan lines 5757–5790: Step 6
  `GATE_UNTIL='^### Task B1' ... scripts/plan-build-gate-go.sh`, then
  `handwire_s3.py --part-a ... go vet ./gui/ ; go test -run '^TestComposer' ./gui/`, Expected
  "23 files extracted, 47 `TestComposer*` top-level PASS".

**Proof, independently re-run (not the author's/controller's own logs — a fresh wiring and
build):**

| Check | Command | Result |
| --- | --- | --- |
| Part-A wiring | `handwire_s3.py --part-a` on my own copy | `10 file(s) wired, 0 mismatches` |
| Part-A file count | `ls gui/composer_*.go \| wc -l` | **23** files; none of `composer_sources.go`, `composer_seat.go`, `composer_review.go`, `composer_selfcheck.go`, `composer_engrave.go`, `composer_cards.go`, `composer_census.go`, `composer_join_test.go` present — exactly what the plan's Step 6 Expected requires |
| Part-A `go vet ./gui/` | | 2 pre-existing go1.25 `ArtifactDir` findings only |
| Part-A `go test -run '^TestComposer' -v ./gui/` | | **47** top-level PASS, **89** total RUN lines, **0** FAIL, `ok` — matches author report's "47 TestComposer* PASS" exactly |
| Whole-tree `go vet ./gui/` | | same 2 pre-existing findings, nothing else |
| Whole-tree `go test -run '^TestComposer' -v ./gui/` | | **110** top-level PASS, **205** total RUN lines (**95** sub-tests), **0** FAIL — matches Task C2's corrected Expected (110/95) and the fold commit message's "whole 205" exactly |
| Whole-tree `go test ./md/ ./mk/ ./sysw/` | | `ok` x3 |
| Whole-tree `gui-shard-test.sh ./gui/ 24` | | `RESULT: ok -- all 1168 tests ran across 24 shards`, partition verified exhaustive (1168==1168), 32s wall |
| DEAD-IN-PROD (`gui`), both trees | extracted the gate's own step-8 Python and ran it directly | whole: **1** (`composerDescriptorCeilingChars`) — matches the fold commit message's "DEAD-IN-PROD (gate step 8): 1" exactly. partA: 7 (expected — Part B's copy-table consumers don't exist yet; not a claim the plan makes about the partA tree) |

## 2. Fidelity I-2 — VERIFIED

`composerConsentFlow` (`composer_selfcheck.go:201`) calls `composerListedPaths(st.list)` and
passes the real `listed`/`keyPathNo` into `composerConsentLinesFor`, not `(nil, 0)`. The
parameterless `composerConsentLines(chunks)` wrapper is gone (0 matches).

**Mutation** (numbered the internal-key path as a leaf — removed the `if i == internal { continue }`
skip in `composerListedPaths`): `TestComposerConsentFlowNumbersPathsFromTheOperatorsList` **FAILS**:
`"INCONCLUSIVE: composerListedPaths gave listed=[1 2] keyPathNo=1 for a tr list whose first path
is the extracted internal key"`. Reverted, `diff -q` clean.

## 3. Journey I-6 / F-458 — NOT VERIFIED (test cannot fail for the named defect)

**The production fix is correct**, verified by direct reading of `composerLockEdit`'s date-entry
closure (`composer_lock.go:255-276`): `composerDateExists` is checked first (returns "that date
does not exist"), then the floor, then the ceiling — no tautology, no dead branch.

**But `composerLockEdit` has zero test callers anywhere in the tree**
(`grep -rn "composerLockEdit(" *.go` → only the production call site and the definition). The
claimed guard, `TestComposerDateCeilingAndImpossibleDateAreToldApart`, calls only the pure
functions `composerDateExists`/`composerDateToUnix` directly and re-derives the dispatch rule
inline — it never calls `composerLockEdit` or anything that reaches its closure.

**Mutation** (the plan's own named mutation — restored `if y > 2038 || u == 0` as the ceiling
test verbatim, inside `composerLockEdit`'s closure, leaving `composerDateExists`/
`composerDateToUnix` untouched): whole `go test -run '^TestComposer' ./gui/` → **`ok`, 0 FAIL**.
`TestComposerDateCeilingAndImpossibleDateAreToldApart` itself, re-run in isolation with `-v`,
**PASSES**, all 5 sub-tests green. Reverted, `diff -q` clean.

This is the exact defect class round-1's own report diagnosed for the *old* hex-entry test (a
test that recomputes the logic standalone instead of calling the function with the bug) — freshly
introduced here in a *new* round-1 guard for a *different* finding.

## 4. Section 3's six unguarded Importants, and section 6's C-9/C-12/6b/8d

| Item | Guard test | Mutation (plan's own, where named) | Result |
| --- | --- | --- | --- |
| journey I-1 | `TestComposerLockAndHashEditsAreNotGuardedByTheDiscardConfirm` | add `composerShapeGuard` to the Time-lock arm | **CAUGHT** — `"the time lock editor was never reached"` |
| journey I-2 / fidelity I-1 | `TestComposerInvariantIgnoresSeveralUnseatedSlots` | remove the `src < 0` skip in `composerInvariantViolation` | **CAUGHT** — `"three unseated slots are reported as colliding"` |
| journey I-5 / fidelity I-5 | `TestComposerBackInTheKeyEditorKeepsTheExistingKeySet` | drop the snapshot/restore in `composerPathEdit`'s Keys arm | **NOT CAUGHT, but a confirmed structural no-op, not a new defect** — `composerKeysEdit`'s decline paths (both count-pickers) never write to `Keys` before the function's single success-path assignment, so nothing needs restoring in the current call graph. This is exactly what round-0/round-1 already established for this mutation. The test is real (drives the actual UI through Back), just not effective for *this specific* named mutation. |
| fidelity I-4 / cell 6b | `TestComposerChangeTheScriptRowRewrapsAndDiscards` | nullify `composerApplyShapeEdit`'s discard at the call site (`if true \|\| ...`), call kept | **CAUGHT** — both assertions fail (`"a wrapper change kept its seats"`, `"the discarded source is still marked used"`) |
| fidelity I-9 | `TestComposerConsentRestatesTheHashRule` | delete the §8i block from `composerConsentLinesFor` | **CAUGHT** — hash rule absent from output |
| tests I-1 | `TestComposerHexEntryItselfTakesExactlySixtyFourCharacters` | `valid := len(frag) >= 63` in the real `composerHexEntry` | **NOT CAUGHT** — see below |
| tests C-12 | `TestComposerShortfallCountsSeatsNotSourcesOnAFixtureThatCanTellThemApart` | pass `len(st.sources)` instead of `composerAssignableSlots(st)` at the `composerShortfall` call site | **CAUGHT** — `"the shortfall screen counts SOURCES, not assignable seats"`; bonus: the reachability test also flags `composerAssignableSlots` as newly dead |
| cell 8d | `TestComposerMintCardsMintsOneCardPerSeatedSlot` | duplicate a card from the SECOND seated slot onward, using `composerCardFixture` (genuinely seats both slots) | **CAUGHT** — `"two seated slots produced 3 card(s), want exactly 2"` |
| tests C-9 | `TestComposerStubReshowSignalIsTheChunkSet` (claimed) + "both flow walks" | force `changed` always false in `composerFlow` | **NOT CAUGHT** — see below |

All CAUGHT rows: mutation applied, confirmed FAIL, reverted, `diff -q` against the source wired
tree confirmed clean before moving on.

### tests I-1 — NOT CAUGHT, root cause

`composerHexEntry` (`composer_hash.go:69`) has exactly one test caller
(`composer_gates_test.go:888`, this very test). The mutation was applied to the real function and
`go build ./gui/` succeeded. The whole `TestComposer*` suite stayed `ok`, **including this test
itself, re-run with `-v`: both sub-cases PASS**. Cause, confirmed by reading the function: after
`valid := len(frag) >= 63`, a 63-character fragment reaches `hex.DecodeString(frag)`, which
**rejects odd-length input independent of `valid`** — so the loop `continue`s on error and the
goroutine never returns within the test's frame-pump budget, leaving `ok` at its zero value
(`false`), which coincidentally equals the test's `want: false` for the 63-character case. This is
the identical "accidental safety net" round-1's own report named when it praised the *previous*
test's choice of 62 (even) over 63 (odd) as "closing the odd-length accidental safety net" — the
new, real-function-calling test walked back into using 63 as its "should fail" boundary and
reproduces the same masking. (The production code is still safe in practice — a second,
independent check, `len(raw) != 32` after decode, refuses any non-64-length input regardless of
`valid` — so no operator-facing guarantee actually breaks; only the claimed regression test does
not regress-test.)

### tests C-9 — NOT CAUGHT, root cause

`TestComposerStubReshowSignalIsTheChunkSet` is **not** one of the plan's own enumerated "twelve
round-1 guards" table (plan ~line 11116) — it is the same, unmodified, pre-round-1 test round-1's
own report already flagged: *"it recomputes the comparison standalone rather than exercising
composerFlow itself."* It declares its own local `changed` variable and never calls `composerFlow`
or anything containing the real line. The plan's claim that "both flow walks drive the screen it
feeds" is not backed by a walk test that asserts anything about the changed/unchanged text on a
*re-shown* stub screen — confirmed by grep: the only other direct call to `composerStubFlow` in
any test (`composer_stub_test.go:148`) passes a **hardcoded** `false` for `changed`, never the
computed value.

**Mutation:** `changed := false && shown != nil && !slices.Equal(shown, template)` in
`composer_flow.go`. `go vet ./gui/` clean (besides the 2 pre-existing findings); whole
`go test -run '^TestComposer' ./gui/` → **`ok`, 0 FAIL** — no test anywhere, walk or unit,
detects the regression. Reverted, `diff -q` clean.

## 5. Section 4 — B5/B6/B9 closing steps and the five Expected lines

All three tasks now reach `Step N: Run the tests` + `Step N+1: gofmt, commit` (previously absent).
Re-ran each task's own `Run:` command exactly as written:

| Task | Command | Plan's Expected | Measured |
| --- | --- | --- | --- |
| B5 | `-run '^TestComposerAssignableSlots\|^TestComposerSeatingComplete'` | `2` | **2** |
| B6 | `-run '^TestComposerSelfCheck\|^TestComposerConsentRefuses'` | `3` | **3** |
| B9 | `-run '^TestComposerDescriptorCeiling\|^TestComposerCensusLines'` | `2` PASS + two measured numbers | **2 PASS**; `concrete descriptor plate ceiling: 596 characters`; `C10's 688-character two-path wallet fits: false` — both exact |

3 of 3 spot-checked (exceeds the brief's "spot-check three"), all exact matches.

## 6. 5c/5f/5h

- **Task C2's counts** now read "110 top-level PASS, 0 FAIL", "95 sub-tests", "1168 across 24
  shards", each labelled "the count at plan time" rather than a threshold. All three match the
  independently-measured values in section 1's table above exactly (110/95/1168).
- **Two Produces lines**: Task B9's now reads
  `func composerCensusLines(params engrave.Params, cards []bundleCard) []string` — matches
  `composer_census.go:86` exactly. Task B2's now reads `srcIdx` — matches
  `composer_sources.go:291` exactly (`grep -n '^func composerCensusLines\|^func composerSeedAccountFor'`
  against the wired tree).
- **N-1's destination**: Task C1's Step 3 now reads "Fold the **four** spec changes" (was three)
  and item **(c1) §7a** exists, addressing the door/`ChoiceScreen.Lead` wording exactly as
  promised. Confirmed present at plan line ~11274.

**New Minor found here**: Task C1's own `git commit -s -F -` template (plan ~11296-11302),
immediately below the now-corrected Step 3 heading, still reads *"fold the three things S3
measured -- the paged capacities, the flag screens, the secret's plate form"* and its body lists
only those three, with no mention of the fourth (§7a) item this same fold added two paragraphs
above. Internal inconsistency within a single task, introduced by this fold. Cosmetic — a future
implementer following Step 3's prose would still make the edit; only the not-yet-run commit
message text would need a manual fix at execution time. **Severity: Minor.**

## 7. New-defect sweep

Walked all 48 hunks of `git diff 3820a6a..e4b3f804 -- design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`
(2581 lines) sequentially. Every hunk maps to a named round-1 finding: STATUS/round-1 table
addition; journey I-6/fidelity I-7 table-row corrections; the old pager-ink test deletion
(fidelity I-7); `composer_discard.go`'s move to Task A5 (§2 consequence); `composerDateExists`
extraction and the dispatch fix (journey I-6); Task A11's Produces/prose/fence restoration (§2);
`composerListedPaths`/`composerConsentLinesFor` wiring (fidelity I-2); B2's Expected-count and
Produces-line fixes (§4, §5f); B3's file move (§2 consequence); B5/B6/B9's closing steps (§4); B9's
Produces line (§5f); B11's test relocation and Expected-count fix (§2, §4); the 542-line Task C0
block of twelve round-1 guards (§3, §6); Task C1's four-item spec fold (§5h); Task C2's corrected
headline counts (§5c). No hunk was found that lacked a corresponding named finding.

**One pre-existing, not-newly-introduced inconsistency left untouched**: Task B11's Step 5
Expected line still says "the reachability test logs **two** named exemptions" — re-run
(`TestComposerEveryScreenFunctionHasAProductionCaller -v`), only **one** (`composerDescriptorCeilingChars`)
actually logs, because `composerDescriptorPlateFits` now has a real caller and is never even
checked against the exempt map. Round-1's own report already found and classified this exact
discrepancy ("both halves... explained by adjacent prose... Severity: Minor, pattern-level, no
hidden defect") and did not require it to be fixed; this fold correctly fixed the *count* half (5→4,
verified in section 5 above) and left the untouched exemption-wording half exactly where round-1
left it. Not counted as new.

**STATUS/Baselines**: current. "S2 HAS MERGED: fork `main` is `321acb56`" — confirmed
`git -C /scratch/code/shibboleth/seedhammer rev-parse --short=8 HEAD` = `321acb56`.

**Plan checks, all re-run independently:**

| Check | Result |
| --- | --- |
| `plan-cite-check.sh` (`CITE_FORK_ROOT=.../seedhammer`) | 241/241 resolved, 0 dangling, 0 ambiguous |
| `plan-glyph-check.sh` | 289 strings, 0 undrawable |
| `plan-table-check.sh` | 118 rows, 0 malformed |
| `plan-stepref-check.sh` | 0 step numbers in prose |
| `plan-staleness-check.sh <plan> <fork> 321acb56` | 144 unchanged, 0 drifted, 4 not-in-repo |

All exact matches to the fold commit message's own re-gate figures.

## Closing counts

- **Section 2 (Critical)**: **VERIFIED**, holds under an independently-run Part-A-only gate and a
  fresh whole-tree build (both zero mismatches on wiring, zero FAIL on test).
- **Fidelity I-2**: **VERIFIED**, holds under mutation.
- **Journey I-6 / F-458**: **NOT VERIFIED** — production fix correct, but its own named regression
  guard cannot fail under the exact mutation the plan itself specifies. **Important.**
- **Six unguarded Importants**: 4 of 6 now **CAUGHT** (journey I-1, journey I-2/fidelity I-1,
  fidelity I-4/cell 6b, fidelity I-9); 1 (**tests I-1**) **NOT CAUGHT**, Important, same defect
  class as journey I-6 above; 1 (**journey I-5**) has a real test but its named mutation is a
  confirmed structural no-op, matching round-0/round-1's own prior finding — not a new defect.
- **C-9**: **NOT CAUGHT** — the plan's claimed closure rests on an unmodified pre-round-1 test that
  never exercises `composerFlow`, and no walk test detects the mutation either. **Important.**
- **C-12, cell 6b, cell 8d**: all three **CAUGHT** under their named mutations.
- **New-defect sweep**: every hunk maps to a named finding; one new **Minor** (Task C1's own
  commit-message template omits its own fourth item); STATUS/Baselines current; all five plan
  checks clean and independently re-run.

**This fold does not close the R0 gate.** Three Importants remain open, all the identical shape:
a regression guard the plan's own text names a specific mutation for, which — reproduced exactly
— does not fail. In each case the underlying production behavior is correct (verified by direct
reading, independent of the test), so no operator-facing guarantee is currently broken; what is
missing is the regression protection the fold's own STATUS line and per-item table claim exists.
Recommend, before the next round: make journey I-6's guard call `composerLockEdit` itself (through
its real UI surface) rather than the pure date helpers; change tests I-1's boundary case from 63
to an even near-miss (e.g. 62, as the superseded test already knew to do) or assert on the
function's return within the same synctest step rather than relying on the zero-value default;
and give C-9 an actual test that drives `composerFlow` through a re-shown stub screen after a real
edit (the walk tests do not do this today, and the existing "pinning" test does not touch
`composerFlow` at all).
