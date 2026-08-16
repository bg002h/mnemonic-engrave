# S6a R4 — cheap pre-review verification pass

Scope: `git diff eb9df42..HEAD -- design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
(142 lines, three commits: d78016e, ca889a9, 7021bcb), plus the surviving §4.7a
design it did not touch (established in eb9df42, still load-bearing for item 1
of the brief). Repo checked against: `/scratch/code/shibboleth/seedhammer`,
`main` @ `b8a23bf`.

## VERDICT: DIRTY — 0 false, 4 stale, 0 table/switch disagreements

All four "stale" findings are Important-class internal-consistency defects
(stale or wrong cross-references / contradictions), not false facts and not a
table/switch mismatch. **The switch and the table agree on all ten rows** —
this cycle's headline risk (the exact class that survived two prior rounds) is
CLEAN.

## §4.7a — SWITCH vs TABLE, ROW BY ROW

Traced by hand, executing the switch in
`design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md:705-718` over each row's
attempt sequence, tracking `sawDisagreement` (sticky, set only on
`verifyFailed`) and the final `res`.

| # | sequence | table says | switch produces | agree? |
| --- | --- | --- | --- | --- |
| 1 | `S` (skip) | `NOT VERIFIED` | loop never runs → `status` stays at its zero-value init `statusNotVerified` → `NOT VERIFIED` | yes |
| 2 | `complete` | `VERIFIED` | `sawDisagreement=false`; `res==complete` arm → `statusVerified` → `VERIFIED` | yes |
| 3 | `incomplete` then stop | `DID NOT COMPLETE` | `sawDisagreement=false`; default arm → `statusDidNotComplete` | yes |
| 4 | `refused`/`abandoned` | `DID NOT COMPLETE` | same as #3, default arm | yes |
| 5 | `incomplete` → `complete` | `VERIFIED` | iter1: default (`sawDisagreement` stays false, not failed); iter2: `res==complete` arm → `statusVerified`, overwriting iter1 | yes |
| 6 | `failed` then stop | `DISAGREED` | `sawDisagreement=true`; `sawDisagreement` arm → `statusDisagreed` | yes |
| 7 | `failed` → `abandoned` | `DISAGREED` | iter1: sticky set, `statusDisagreed`; iter2: `res=abandoned`, sticky still true, `sawDisagreement` arm → `statusDisagreed` (unchanged) | yes |
| 8 | `failed` → `incomplete` | `DISAGREED` | same pattern as #7 | yes |
| 9 | `failed` → `complete` | `VERIFIED on a repeat check` | iter1: sticky set; iter2: `res==complete && sawDisagreement` (both true) → `statusVerifiedOnRetry` | yes |
| 10 | `incomplete` → `failed` → `complete` | `VERIFIED on a repeat check` | iter1: default; iter2: sticky set, `statusDisagreed`; iter3: `res==complete && sawDisagreement` → `statusVerifiedOnRetry` | yes |

**0/10 disagreements.** This is the exact defect class (R1 C-1, R2 C-2/I-x,
R3 C-1/C-2) that survived five prior folds; the two-sticky-facts + switch
replacement is internally sound.

Two supporting claims checked separately, both TRUE:
- **"`status` is assigned only inside the loop body."** The only re-assignments
  of `status` after its zero-value declaration happen inside the four switch
  arms, which the pseudocode's own comment places "inside the existing offer
  loop." On Skip the loop body never runs, so `status` is left at its
  zero-value init (`statusNotVerified`) — consistent, not contradictory.
- **Single-sig has no retry loop.** Verified against
  `gui/singlesig.go:130-133` (fork): the verify offer is
  `if sel, ok := verifyChoice.Choose(...); ok && sel == 0 { singleSigVerifyFlow(...) }`
  — a one-shot conditional call, no loop, no re-offer. Matches the plan's
  characterization exactly.

## FACTUAL CLAIMS

| claim | TRUE/FALSE/UNVERIFIABLE | evidence |
| --- | --- | --- |
| `singleSigVerifyFlow` (`gui/singlesig_verify.go:65`) has eleven exit points and returns nothing | TRUE | Function signature has no return type. Counted 10 explicit `return` statements (lines 69,78,90,98,112,117,125,130,138,146) + 1 implicit fall-off-end after `showNotice` (success path) = 11. |
| `multisigRestoreDocFlow` has three call sites, one being `gui/multisig_nested_name_test.go:230`, passing `nil` | TRUE | `grep -rn "multisigRestoreDocFlow(" --include="*.go" .` → exactly 3 call sites (excl. the func def): `gui/multisig.go:361`, `gui/multisig_build.go:478`, `gui/multisig_nested_name_test.go:230`. Read the test: it does pass `nil` for `extra` and drives `runUITouchRaster` + `pumpUntil` + an ink-floor assertion — a real rendered document, not a stub. |
| `TestSeedResidencyRulingDescribesTheMultiSeedReality` asserts `"Every seed"` and guards the build path by accident | TRUE | `gui/multisig_build_prose_test.go:382-384`: `if !strings.Contains(ruling, "Every seed") { t.Errorf(...) }`. It also positively rejects the singular ("A seed you entered") at :377-381, so a mis-wire to the one-seed arm fails loudly, matching the claim. |
| §4.9's quote of `SPEC_seedhammer_T6a_singlesig_flagship.md:36` is verbatim | TRUE | Line 36 today: `restore doc (R0-M2): display-only + optional NFC; master fp + the concrete descriptor + first receive/change address (from-xpub *bip380.Descriptor, gui/md1_expand.go:60-77 + address.Receive/Change); greps clean of any xprv/private material.` The plan's quote elides the parenthetical with an honest `...` and matches word-for-word elsewhere. |
| §4.9's two grep gates behave as claimed, run today | TRUE (as forward-looking baselines) | `grep -c "verification status" design/SPEC_...md` → `0` (exit 1) today — consistent with the gate being a **post-update** check (`expect >= 1`), not yet met. `grep -c "xprv" design/SPEC_...md` → `3` (exit 0) — this is the baseline the "expect unchanged" clause protects. Neither gate is mis-described. |
| T3 "passes on the unfixed tree by design" | TRUE | `gui/singlesig.go:80` today is still the bare literal `"Full (seed + keys)"` (not yet wired to `buildFullModeLabel`, unlike `gui/multisig.go:217` and `gui/multisig_build.go:373`, which already use it). A bare (no-passphrase) single-sig run therefore cannot say "NOT passphrase" today — T3's non-vacuity assertion already holds pre-cycle, exactly as claimed. |
| T9, T13a, T13b, T14 target functions/types that do not yet exist | TRUE | `grep -rn "buildVerifyStatusLines\|verifyStatus\b\|statusNotVerified\|statusVerified\b" --include="*.go" .` → zero hits anywhere in the fork. |
| T9's row: "five §4.7c statuses" | TRUE | `verifyStatus` const block (§4.7c, plan lines 854-858) lists exactly 5 constants: `statusNotVerified, statusDidNotComplete, statusDisagreed, statusVerifiedOnRetry, statusVerified`. |
| Three false-comment sites (§4.7c) exist verbatim as quoted | TRUE (measured, not this fold's diff but load-bearing) | `gui/multisig_build.go:439`, `gui/multisig.go:321-322`, `gui/multisig_verify.go:78-79` all read as quoted; the type block at `gui/multisig_verify.go:88-100` has exactly 5 constants, confirming "doubly wrong." |

## STALE REFERENCES TO THE DELETED DESIGN

Greps run: `worstStatus`, `worst-seen`, `severity(`, `max(`, `incentive
invariant`, `five-state`/`5-state`/`five state`, `worst outcome` (case-insens.),
`rank`/`hierarchy`/`priority order`/`dominat`/`outrank`/`supersed`,
`accumulator`, `zero value.*seed`, `max over`.

All hits resolve to **historical, past-tense narrative** explicitly describing
why the lattice was deleted (e.g. "The R1 and R2 folds *built* a ranked
ordering...", "There is no `severity()`, no `max`, no seed", "The R2 fold
*asserted* an 'incentive invariant'...") — none assert the deleted mechanism as
current design. No dangling algorithmic residue found.

**One exception, flagged as a stale/contradictory finding (not caught by the
mechanical greps because it doesn't use the flagged vocabulary):**

- **§4.7a's own section header**, `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md:677`:
  `#### 4.7a THE STATUS IS THE WORST OUTCOME SEEN, NOT THE LAST ONE (R1 C-1)`.
  Unlike sibling headers 4.7b/4.7c (which correctly state the *current* design —
  verified true above), this header still asserts the pre-R3 "worst outcome"
  framing R3 explicitly deleted. It is **false of the current mechanism**: row 5
  (`incomplete → complete`) prints `VERIFIED` — the *best* outcome seen, not the
  worst — and the section's own body says so directly two paragraphs later: "An
  incomplete first attempt is **NOT an anomaly**, and the earlier design
  **wrongly treated it as one**." The header was written for the R1 fix and
  never updated across the R2/R3 rewrites that replaced its own mechanism.
  **Severity: Important** (stale claim contradicting the new design, per the
  brief's severity rule) — cosmetic in that no code or test reads the header,
  but it is the first line a future reviewer sees and it asserts exactly the
  model this cycle spent three rounds disproving.

## CROSS-REFERENCES

Extracted every `§n.n[a-z]` token and checked non-obvious ones against actual
section content (headers at
`grep -n "^#\{1,4\} " design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`).
Most resolve correctly, including all of this fold's own new material (§4.9's
self-references to §4.2/§4.3/§4.4/§4.7 all check out against those sections'
actual content; §3.1.7 exists and says what §4.9 says it says).

**Two wrong-section citations found** (both pre-existing, not touched by this
3-commit diff, but live in the current artifact — the "three false-comment
sites" table lives in **§4.7c** (plan lines 883-897, under the `#### 4.7c`
header at line 836), not §4.7b):

- Line 342 (§3.2): *"as does the type's own doc comment, §4.7b"* — should be
  §4.7c.
- Line 923 (§4.8 step 8): *"Correct the three false comments (§4.7b) + T8"* —
  should be §4.7c.
  (§4.7b's own content — the slice-index-0 / leading-parameter argument — is
  correctly cited at plan lines 188, 397, 416, 422, and 829/919. Those are
  fine.)

**One contradiction introduced by this fold itself (in scope), not a mis-citation
but a build-order/test-row mismatch:**

- This fold added (plan lines ~1010-1019): *"T10, T12, T13a and T13b CANNOT RUN
  ON THE SINGLE-SIG PATH... Those four rows must be driven on a **multisig**
  flow... Consequence for §4.8's build order: **step 7 is where these land**,
  and it needs a multisig walk."*
- But §4.8's build-order table (line 917, **untouched by this diff**) still
  reads: `2 | verifyStatus + buildVerifyStatusLines + T9, T13a, T13b, T14 |
  pure functions, no callers yet, fully unit-testable`.
- These two statements are in direct conflict: one says T13a/T13b are
  pure-function unit tests landing at step 2 with no callers; the other,
  written by this fold, says they require a live multisig retry loop and land
  at step 7 (needing steps 2, 4, 5 done first). The fold corrected the test-plan
  narrative (§5) but never propagated the correction back into §4.8's own step 2
  row — the same "corrected here, duplicate left standing" failure mode the plan
  itself names at line 424 for a different finding.
  **Severity: Important.** An implementer following step 2 literally would try
  to write T13a/T13b as isolated unit tests before any flow exists, which the
  newly-added paragraph says is impossible for the sequences they need to
  reach.

Everything else — step numbering 1-9, §4.8/§4.9 heading order, §5's references
back into §4.7a's ten-row table (T13a/T13b), §5.1(a)/(b)'s split, and the
§3.1/§3.2 numbered-list cross-references — resolves correctly.

## Notes on scope

- Did not re-derive or challenge any settled decision (ONE PIECE ruling,
  status-line design, cycle scope) — none of the above findings touch those.
- Did not re-audit `file:line` citations for existence (gate already ran,
  93/93, exit 0) — findings above are about what the cited/referenced text
  **means**, specifically wrong section pointers, not missing lines.
- The four Important findings are all "internal consistency" class, not "false
  fact" class and not "table/switch disagreement" class — the highest-risk
  check (§4.7a row-by-row) came back fully clean.
