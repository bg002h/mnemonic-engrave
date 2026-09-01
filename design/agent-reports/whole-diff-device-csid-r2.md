# Whole-diff device-csid fold re-check (r2) — scoped, not a fresh audit

**Scope (per dispatch brief):** did the fold at `c04d4d1` (dev-warn tree)
and `66cc1c0` (mnemonic-engrave tree) discharge each finding of
`design/agent-reports/whole-diff-device-csid-review.md` (1C/1I/4M/3N), and
does either fold introduce a new defect? The review's cleared list
("Explicitly cleared" section) is settled and not re-derived. The
operator-approved frozen wording/marker form is re-confirmed unchanged.

**VERDICT: 0 Critical / 0 Important — SHIPS.** Every finding is FIXED as
described (one Important carries a documented, consistent, out-of-band
extension by the controller, verified below). No new defect found. All
gates green.

---

## Per-finding disposition

**C1 (Critical) — FIXED, all 4 parts, verified against the actual diffs
(not the fold report's prose).**
1. README closing clause corrected — quoted, `design/journeys/csid-tags/README.md:85-88`:
   *"The Build Policy plate census and restore doc do NOT carry this marker
   on any reachable path"* — matches the review's exact remedy #1.
2. Test docstring rewritten — `gui/csid_warning_test.go:397-411` (dev-warn,
   commit `c04d4d1`): *"TestBuildPlateCensusLinesMarksCSIDMismatch is a
   HELPER-LEVEL PIN... NO PRODUCTION FLOW FEEDS THESE TWO FUNCTIONS A
   GATHERED CARD"* — matches remedy #2 verbatim in substance.
3. `csidMarker(c)` calls KEPT at `multisig_build_census.go:53,89` with new
   doc comments: *"csidMarker(c) BELOW IS DEFENSIVE, NOT LIVE TODAY"* and
   an explicit warning not to "fix" this by routing gathered cards into the
   restore doc — matches remedy #3's chosen branch exactly.
4. Spec Contract 3 + Acceptance amended in place, dated 2026-09-01, quoting
   the review by name; **F-447** (`device-csid-census-premise-gap`) filed
   in `design/FOLLOWUPS.md`, records the host→device premise-transfer
   failure and repeats the "do not route gathered cards" warning — matches
   remedy #4.

**I1 (Important) — FIXED in README as literally specified, PLUS a
controller extension to the spec beyond the fold report's own claim; the
extension is verified CONSISTENT, not contradictory.**

README `design/journeys/csid-tags/README.md` "Tap order" now reads exactly
per the remedy: chooser appears on tag1 → confirm "Inspect key" → "Captured
1 of 2" → tag2 → modal → dismiss → Back (card display) → Back again (home)
→ only then tag3/tag4, plus the silent-tap sentence. The fold **also**
applied the same chooser-confirm instruction to the clean pair (step 4),
an extension the implementer flagged as necessary to avoid reproducing I1's
exact failure mode for the second pair — reasonable and correctly scoped.

**Discrepancy surfaced by the dispatch brief, checked directly:** the fold
report (`impl-device-csid-fold.md`) states under I1 *"this fold left that
spec paragraph as-is"* regarding the spec's on-device Acceptance section.
The actual commit `66cc1c0` diff of `SPEC_device_csid_warning.md` **does**
amend that paragraph — it adds *"CONFIRMING the 'mk1 key / Choose action'
chooser into Inspect key on the first tap"* and *"Dismiss the modal, Back
out of the card display, Back AGAIN to the home screen (a tag tapped from
the chooser or card display does nothing at all — do not read that as a
dead build)"*, word-for-word tracking I1's remedy text. So the fold report
is stale relative to the commit it sits in (the controller edited the spec
paragraph *after* the report was written, one step beyond the report's
literal scope, per the dispatch brief's framing) — **not a contradiction**:
the spec's new wording matches the corrected README's wording and the
review's I1 remedy exactly; both documents now agree, and neither claims
anything the other doesn't. This is a report/commit staleness note, not a
gating defect — recorded here so a future reader isn't misled by the fold
report's own "left as-is" line.

**M1 — FIXED, test run and PASSES.**
`TestCSIDFixturePairIsWhatTheSpecClaims` (`gui/csid_warning_test.go:126-134`)
now asserts `csidMismatchWarningText(wantDeclared, wantDerived) !=
pinned.WarningText` as a strict byte-exact check, exactly the remedy line.
Verified: `go test ./gui/ -run TestCSIDFixturePairIsWhatTheSpecClaims -v` →
PASS.

**M2 — FIXED, both parts, test run and MUTATED live.**
`TestBundleGatherConsumersAreAccountedFor` (`gui/csid_warning_test.go`) is
AST-based (`go/parser`/`go/ast`), enumerates every top-level function in
non-test `gui/*.go` calling `bundleGatherFlow`/`bundleGatherFlowResume`,
and checks the set against the six named Contract-3 consumers.
- Ran clean: PASS.
- **Mutation 1** — appended a real 7th consumer function
  (`fakeSeventhConsumerForMutationTest`, correctly calling
  `bundleGatherFlow(ctx, th, "fake")`) to `gui/bundle.go`: the guard
  **FAILED** with the exact expected message ("...is not on this guard's
  named list of six audited consumers..."). Reverted; `diff` against the
  pre-mutation file byte-identical, `git status` clean.
- **Mutation 2** — deleted the `"supplyMultisigPolicyFlow": {exempt:
  "extractSuppliedMd1"}` line from `gui/csid_warning_test.go`: the guard
  **FAILED** (same message, naming `supplyMultisigPolicyFlow` as the
  spurious 7th). Reverted; byte-identical, `git status` clean.
- Comment-blindness fix: `TestMultisigVerifyFlowWiresCSIDNoteIntoVerdicts`
  now runs its `funcBody` output through `stripGoComments` (tokenizes with
  `go/scanner`, deletes only COMMENT byte ranges, preserves every other
  byte including spacing) before the `strings.Contains`/`strings.Count`
  checks — confirmed in the diff, ran clean (PASS).
- Brittle-count fix: the older wiring test's `!= 2` changed to `< 2`
  (fails only if a wired site is lost, not if a legitimate third
  comparator-FAIL site is added) — confirmed in the diff, ran clean
  (PASS).

**M3 — FIXED, "accept + document + test," test run and PASSES.**
`showBundleCSIDMismatchNotices`'s doc comment (`gui/bundle.go:112-124`)
states the re-fire-on-re-entry is intentional and names the three loop-back
call sites. New live test `TestBundleFlowNoticeRefiresOnReviewBackReentry`
drives `bundleFlow` end to end (gather → Done → first fire → dismiss →
Back at review, re-enters gather with the card still on the pile → Done
again → second fire) and asserts the notice fires twice. Ran individually:
PASS.

**M4 — FIXED, verified by diff and by test run.**
`TestBuildPolicyGatherSilentOnCleanTwinLive`'s post-Done pump budget raised
from 8 to 64 frames with a comment tying it to the mismatch twin's budget.
Confirmed in the diff; ran individually: PASS.

**N1 — FIXED.** Implementer report Deviation 2 now reads "7 sites total,
corrected 2026-09-01... `singlesig_verify.go:225,229,231,235`,
`multisig_verify.go:1171,1196,1208`" — matches the review's count and line
list exactly.

**N2 — FIXED.** Same report's gofmt line now says "on every touched/new
file (named explicitly, not `gofmt -l gui/`)" and names the three files a
bare `gofmt -l gui/` flags. Independently re-verified in this environment
(go1.26.4): `gofmt -l gui/ mk/` returns exactly `gui/transaction.go`,
`gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go` — none
touched by this diff, matching the documented set precisely.

**N3 — FIXED.** README regenerate snippet relabeled ```` ```sh ```` →
```` ```bash ```` with a note that fish lacks herestrings; commands
unchanged (already confirmed reproducible under bash by the r1 review).

---

## Freeze check — wording/marker form UNCHANGED

`git -C /scratch/code/shibboleth/sh-worktrees/dev-warn diff 952712a..c04d4d1 -- gui/`:
touches exactly three files (`bundle.go`, `csid_warning_test.go`,
`multisig_build_census.go`). Filtered every `+`/`-` line in the two
**production** files (`bundle.go`, `multisig_build_census.go`) for
anything that is not a `//` comment or blank line: **zero matches** — every
changed line in production code is a doc comment. Also grepped the full
`gui/` diff for `csid %` (the `[csid X!Y]` marker's format verb) and any
`Sprintf`/`Fprintf` call in the two production files: **zero matches**. No
rendered warning string or marker format changed. The operator-approved
frozen wording (spec amended 2026-09-01, "Screenshots perfect!") is intact.

## New-defect sweep

None found. The two mutation probes above are the adversarial check this
re-review is scoped to run (per the M2 remedy's own suggestion); both
demonstrated the guard actually gates rather than passing vacuously. No
other code path in either fold commit does anything but add comments,
tests, and prose.

## Gates (both trees)

| check | result |
| --- | --- |
| `git -C dev-warn diff 952712a..c04d4d1 -- gui/` non-comment production lines | 0 |
| `gofmt -l` (touched files) | empty |
| `gofmt -l gui/ mk/` (repo-wide) | 3 pre-existing files, none touched by this diff — matches N2 |
| `go vet ./mk/... ./gui/...` | clean except 3 pre-existing `testing.ArtifactDir` toolchain-version findings, none touched by this diff |
| `go test ./mk/ ./gui/` (literal) | **ok** — `mk` cached-ok, `gui` ok in 164.457s |
| `scripts/gui-shard-test.sh ./gui/ 24` | **ok — all 1058 tests**, exhaustive 24-way partition asserted, 46s wall |
| named M1/M2/M3/M4/C1-docstring tests, individually | all PASS |
| M2 mutation 1 (fake 7th consumer, `bundle.go`) | guard FAILS as expected; reverted byte-identical, `git status` clean |
| M2 mutation 2 (drop exempt-list entry, `csid_warning_test.go`) | guard FAILS as expected; reverted byte-identical, `git status` clean |
| `grep -c '^func Test' gui/csid_warning_test.go` | 23, matches fold report |
| both trees `git status --short` | clean (both folds are committed: `c04d4d1`, `66cc1c0`) |

**0 Critical / 0 Important. SHIPS.**
