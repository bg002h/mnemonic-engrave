# Composer S3 plan — R0 round 3, TARGETED fold-verification

Independent reviewer, targeted lens on fold `de66796a9d420844aa2959ad7728f26ac636b0a6`
(pre-fold `e4b3f80`) against `design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`. One
question: do the three regression guards round 2 found unable to fail now fail under the
plan's own named mutation, was the Minor folded, and did nothing else move? Not a fresh
audit — round 2's other VERIFIED items (Section 2 Critical, fidelity I-2, C-12, cell 6b,
cell 8d) are taken as settled and were not re-derived.

**Verdict: GREEN. All four items hold.** The three replacement guards each fail under the
plan's own named mutation, byte-for-byte matching the Expected output the fold pastes into
the plan text. The Minor (Task C1's commit template) is folded correctly, now naming and
describing all four spec-fold items. All eight hunks of the fold diff map to these four
items or their necessary bookkeeping consequences; nothing else moved. Every re-run gate
and plan check reproduces the fold commit message's own figures exactly.

## Method

Copied (not re-wired — `handwire_s3.py` refuses to run twice on an already-wired tree) the
two scratches the controller's re-gate already built from this fold:
`/scratch/code/shibboleth/.plan-build-gate-go-s3/wired` → `.s3-r3-lens/whole`,
`.../.plan-build-gate-go-s3-partA/wired` → `.s3-r3-lens/partA` (`diff -rq` confirmed
byte-identical copies before mutating). Go:
`/nix/store/i77g9dmcd399rmxk8688qfr4g2wzgk37-go-1.26.7/bin/go`,
`CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local`, `-mod=readonly`,
`TMPDIR=/scratch/code/shibboleth/.tmp`. Both scratch trees deleted after verification.

## 1. Journey I-6 / F-458 — VERIFIED

Applied the plan's named mutation to `gui/composer_lock.go`'s date closure (captured `u`
from `composerDateToUnix` and replaced the `composerDateExists`/floor/exists dispatch with
`if y > 2038 || u == 0 { return composerCopyDateCeiling(), false }` first, per the plan's
own "MUTATION: restore `if y > 2038 || u == 0` as the ceiling test" comment).

`go test -run '^TestComposerLockEditTellsAnImpossibleDateFromThePastCeilingDate' -v ./gui/`:

```
--- FAIL: TestComposerLockEditTellsAnImpossibleDateFromThePastCeilingDate
    --- FAIL: .../an_impossible_date_inside_the_band
        the date pad does not say "that date does not exist" for 20270231.
        ...the date pad says "up to 2038-01-19" for 20270231, which is the WRONG message...
    --- PASS: .../a_real_date_past_the_ceiling
    --- FAIL: .../a_real_date_below_the_floor
        the date pad does not say "before 2009" for 20081231.
```

Matches the plan's pasted Expected block exactly (same two failing sub-cases, same wording,
same third sub-case passing by coincidence). Reverted; `diff -q` against the source wired
tree clean.

## 2. Tests-lens I-1 — VERIFIED

Applied the named mutation to `gui/composer_hash.go` (`valid := len(frag) == 64` →
`valid := len(frag) >= 63`).

`go test -run '^TestComposerHexEntryItselfRefusesAnythingButSixtyFourCharacters' -v ./gui/`:

```
--- FAIL: TestComposerHexEntryItselfRefusesAnythingButSixtyFourCharacters
    --- PASS: .../sixty-two_hex_characters
    --- FAIL: .../sixty-three_hex_characters
        composerHexEntry ACCEPTED 63 characters (the exact length the named mutation
        would admit): it reached the decode branch, which only a valid-length fragment
        does.
        Frame: "Thatisnota32-bytedigest.Hashlock"
```

Matches the plan's pasted Expected block exactly (same frame text, same message).
Reverted; `diff -q` clean.

## 3. C-9 — VERIFIED

Applied the named mutation to `gui/composer_flow.go`
(`changed := shown != nil && !slices.Equal(shown, template)` →
`changed := false && shown != nil && !slices.Equal(shown, template)`).

`go vet ./gui/`: only the two pre-existing `ArtifactDir` (go1.25 vs go1.26) findings.
`go test -run '^TestComposerFlowReShowsTheStubScreenOnlyAfterARealEdit' -v ./gui/`:

```
--- FAIL: TestComposerFlowReShowsTheStubScreenOnlyAfterARealEdit
    --- PASS: .../back_out_and_Done_again_with_no_edit
    --- FAIL: .../back_out,_change_a_key_count,_Done_again
        the stub screen was re-shown after a real key-count change and does NOT carry
        §8s: a card minted with the old stub will not seat here, and the operator is
        not told.
```

Matches the plan's pasted Expected block exactly. Reverted; `diff -q` clean.

Full-tree `diff -rq` against `/scratch/code/shibboleth/.plan-build-gate-go-s3/wired` after
all three mutations applied-and-reverted in sequence: clean (no output). Whole
`go test -run '^TestComposer' ./gui/` after all reverts: `ok`.

## 4. The Minor — FOLDED

Task C1's commit template (plan line ~11613) now reads *"fold the four things S3
measured -- the paged capacities, the door's key state, the flag screens, the secret's
plate form"*, with a body paragraph for each of the four, including the previously-missing
§7a/door's-key-state item. Confirmed present and consistent with Step 3's "Fold the
**four** spec changes" and item **(c1) §7a** (plan line ~11593), which was already
verified present in round 2. No inconsistency remains between the heading, the numbered
items, and the commit template.

## 5. Nothing else moved

**Hunk sweep** (8 hunks, `git diff e4b3f80..de66796a9d420844aa2959ad7728f26ac636b0a6`):

| Hunk | Content | Maps to |
| --- | --- | --- |
| 1 | STATUS line rewrite | documents the round-2 fold as a whole (items 1-4); not independent scope |
| 2 | New "R0 round 2" table | documents the fold-location of items 1-4 plus two correctly-non-refolded items (I-5 structural no-op, B11 exemption wording) already classified by round 1/2; documentation, not a fifth item |
| 3 | `composer_gates_test.go`: hex-entry test replaced, lock-edit test added | items 1, 2 |
| 4 | `composer_flow.go`-driving test added + `composerFlowDone` helper | item 3 |
| 5 | Task C0 table rows renamed/added, count 12→14 | bookkeeping consequence of items 1-3 |
| 6 | New Step 5 "apply each guard's mutation, see it FAIL"; renumbers old Step 5→6 | operationalizes the shared root cause named across items 1-3; not independent scope |
| 7 | Task C1 commit template, three→four things | item 4 |
| 8 | Task C2 headline counts 110→112, 95→100, 1168→1170 | bookkeeping consequence of items 1-3 (net +2 top-level tests) |

Every hunk maps to one of the four items or a necessary, machine-verified bookkeeping
consequence of them. No hunk touches Section 2, fidelity I-2, C-12, cell 6b, cell 8d, or
any other round-2-VERIFIED item.

**Part-A-only gate, re-run fresh (not the pre-wired scratch):**
`GATE_UNTIL='^### Task B1' SCRATCH=/scratch/code/shibboleth/.s3-r3-lens-partA
scripts/plan-build-gate-go.sh design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md` →
23 files extracted (0 dropped beyond the plan's own count); `go test -run '^TestComposer'
./gui/` on the unwired extraction FAILS (expected — Part A's fragments aren't wired yet).
`handwire_s3.py --part-a .` on this fresh copy → `10 file(s) wired, 0 mismatches`. Then:
`go vet ./gui/` clean but for the two pre-existing `ArtifactDir` findings; `go test -run
'^TestComposer' -v ./gui/` → `ok`, **47** top-level PASS, **89** total PASS lines
(`grep -cE -- '--- PASS'`), **0** FAIL.

**Whole-tree measurements** (on the copy of `.plan-build-gate-go-s3/wired`):
`go test -run '^TestComposer' -v ./gui/` → `ok`, **112** top-level PASS, **100** sub-test
PASS, **212** total PASS lines, **0** FAIL — exact match to Task C2's corrected headline
(112/100) and the fold commit message's "whole 212". `gui-shard-test.sh ./gui/ 24` →
`RESULT: ok -- all 1170 tests ran across 24 shards`, partition exhaustive, 33s wall — exact
match. DEAD-IN-PROD (gate step 8's own extracted Python, run directly against the wired
`gui` tree): **1** (`composerDescriptorCeilingChars`) — exact match to the fold commit
message's "DEAD-IN-PROD: 1".

**Plan checks, all re-run independently:**

| Check | Result | Fold commit message |
| --- | --- | --- |
| `plan-glyph-check.sh` | 299 strings, 0 undrawable | match |
| `plan-stepref-check.sh` | 0 step numbers in prose | match |
| `plan-table-check.sh` | 126 rows, 0 malformed | match |
| `plan-cite-check.sh` (`CITE_FORK_ROOT=.../seedhammer`) | 241/241 resolved, 0 dangling, 0 ambiguous | match |
| `plan-staleness-check.sh <plan> <fork> 321acb56` | 144 unchanged, 0 DRIFTED, 4 not-in-repo | match |

All five exact matches to the fold commit message's own re-gate figures.

## Closing counts

- **Journey I-6 / F-458**: **VERIFIED** — the replacement guard drives `composerLockEdit`'s
  real screen and fails under the named mutation with the pasted output.
- **Tests I-1**: **VERIFIED** — the replacement guard drives the real `composerHexEntry`
  screen and fails under the named mutation with the pasted output.
- **C-9**: **VERIFIED** — the replacement guard drives `composerFlow` itself and fails
  under the named mutation with the pasted output.
- **The Minor**: **FOLDED** — Task C1's commit template now names and describes all four
  spec-fold items, consistent with Step 3's heading and item list.
- **Hunk sweep**: all 8 hunks map to the four items or their measured bookkeeping
  consequences; no other round-2-VERIFIED section touched.
- **Gates**: Part-A-only (23 extracted, 10 wired/0 mismatches, 89 PASS/0 FAIL) and whole
  (212 PASS/0 FAIL, 1170/1170 sharded, DEAD-IN-PROD 1) both reproduce the fold commit
  message's figures exactly. All five plan checks clean and exact.

**This closes the R0 gate for this plan: 0 Critical / 0 Important.** No further round is
recommended on this question — the three guards this round targeted are real guards now,
the Minor is fixed, and the fold diff carries no untracked scope.
