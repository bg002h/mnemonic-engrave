# Fold — whole-diff-device-csid-review findings (1C/1I/4M/3N)

**Source review:** `design/agent-reports/whole-diff-device-csid-review.md`
(verbatim, unedited by this fold). **Trees touched:** fork worktree
`/scratch/code/shibboleth/sh-worktrees/dev-warn` (branch
`impl/device-csid-warning`, base `952712a`, code/tests) and
`mnemonic-engrave` main checkout (spec/README/FOLLOWUPS/report text). **No
commits made in either tree** (per dispatch brief); both left dirty.
Screenshot gate is CLOSED and untouched: every dev-warn production edit is a
`//` comment, verified below — no rendered string or the frozen marker form
changed.

## Per-finding disposition

**C1 (Critical) — census/restore-doc marker unreachable; a test + a
committed README both claimed it worked. All 4 parts done.**

1. README (`design/journeys/csid-tags/README.md`, mnemonic-engrave): the
   closing "(Build Policy only) the plate census / restore doc" clause is
   removed; "payload-cards" and the review-list marker (both reachable, both
   proven live) are kept, plus a new explicit sentence that the Build Policy
   census/restore doc never carries the marker on any reachable path,
   citing the review.
2. `TestBuildPlateCensusLinesMarksCSIDMismatch`'s docstring
   (`gui/csid_warning_test.go`, dev-warn) rewritten to state the
   helper-level-pin truth: it constructs the mismatched card directly, no
   production flow feeds either function a gathered card.
3. The two `csidMarker(c)` calls at `multisig_build_census.go:53,89`
   (dev-warn) are KEPT, each with a new doc-comment explaining they are
   defensive-only today (data-driven, zero cost, lights up for free if a
   future flow ever routes a gathered card there) and explicitly warning
   against "fixing" this by routing gathered cosigner cards into the
   restore doc.
4. Spec (`design/SPEC_device_csid_warning.md`): Contract 3's Build Policy
   bullet and the Acceptance row amended in place (dated 2026-09-01) to the
   reachability truth, cross-referencing the review and the new follow-up.
   FOLLOWUPS entry **F-447** (`device-csid-census-premise-gap`) filed,
   recording the host→device premise-transfer failure so a future cycle
   does not re-import it, and explicitly repeating the "do not route
   gathered cards into the inventory" warning.

**I1 (Important) — README tap procedure omits the chooser + double-Back,
silent-tap failure mode. Done, with one extension beyond the literal quote.**

Rewrote "Tap order" exactly per the report's sequence: tag1 → `mk1 key` /
"Choose action" chooser → confirm "Inspect key" → "Captured 1 of 2" → tag2
→ warning modal → dismiss → Back out of card display → Back again to home
→ only then tag3/tag4, plus the sentence that a tap from the chooser or
card display does nothing. **Extension:** while reading `gui/gui.go`'s
`mdmkFlow` to write this, confirmed the SAME chooser dispatch is
unconditional on card content — tapping tag3 (a fresh card, from home) also
shows the chooser and needs "Inspect key" confirmed, which the report's I1
prose doesn't spell out for the clean pair (it only walks the pinned pair
in detail) but which follows directly from the report's own call-graph
statement ("dispatches through `StartScreen.Flow` → `engraveObjectFlow` →
`mdmkFlow`"). Left uncorrected, step 4 would have reproduced I1's exact
failure mode (a silent, unexplained non-response) for the clean pair. Added
the same chooser-confirm instruction there.

**Note, not fixed (scope discipline):** the report's I1 text also flags
that "the same imprecision sits in the spec's Acceptance section" (the
on-device-acceptance paragraph, `SPEC_device_csid_warning.md` lines
~122-131, "after the SECOND tap … the first tap correctly shows only
capture progress"). The report's stated Remedy for I1 names only
`README.md`. Per "fix exactly what the report names," this fold left that
spec paragraph as-is. Flagging it here since it carries the identical gap
I1 already proved is load-bearing at the flash gate.

**M1 — done.** Added the strict byte-exact assertion to
`TestCSIDFixturePairIsWhatTheSpecClaims`: `csidMismatchWarningText(wantDeclared,
wantDerived) != pinned.WarningText` fails the test, exactly as the report's
remedy line specifies.

**M2 — done, both parts.**
- New `TestBundleGatherConsumersAreAccountedFor` (dev-warn): walks the real
  AST (`go/parser`/`go/ast`) of every non-test `gui/*.go` file, finds every
  top-level function that calls `bundleGatherFlow`/`bundleGatherFlowResume`,
  and asserts that set is exactly Contract 3's six named functions —
  `bundleFlow`/`walletPolicyFlow`/`buildMultisigPolicyFlow` must call
  `showBundleCSIDMismatchNotices`, `multisigVerifyFlow`/`singleSigVerifyFlow`
  must call `bundleCSIDNote`, `supplyMultisigPolicyFlow` must still call
  `extractSuppliedMd1`. AST-based rather than `funcBody`'s raw-string slice,
  so it is structurally immune to comment-blindness. **Mutation-verified
  live** (not part of the committed diff — reverted): adding a 7th caller
  elsewhere in `gui/*.go` fails it; removing the modal call from `bundleFlow`
  fails it too; `git status` confirmed clean after each revert.
- `TestMultisigVerifyFlowWiresCSIDNoteIntoVerdicts` patched in place: the
  `!= 2` count is now `< 2` (fails only if a wired site is LOST, not if a
  legitimate third one is added), and its `funcBody` source is now passed
  through a new `stripGoComments` helper (tokenizes with `go/scanner`,
  deletes only the byte ranges scanner classifies as `COMMENT`, keeps every
  other byte — including original spacing — untouched) before the
  `strings.Contains`/`strings.Count` checks.
  **RED→GREEN note:** the first version of `stripGoComments` reconstructed
  text by rejoining token literals with a single space each, which turned
  `bundleCSIDNote(cards)` into token-spaced text and broke this very test
  (`FAIL: ... no longer computes bundleCSIDNote(cards)`) — an own-goal
  caught by actually running the suite. Rewritten to delete comment byte
  ranges from the original string instead, preserving exact substrings;
  confirmed GREEN after the fix.

**M3 — done, "accept + document + test" (smallest change).** The report
frames this as non-blocking and offers a debounce latch only "if it
irritates at the gate" — not a requirement. Chose the smaller path:
(a) extended `showBundleCSIDMismatchNotices`'s doc comment (dev-warn,
`gui/bundle.go`) to state the re-fire-on-re-entry is intentional, name the
three loop-back call sites, and point at the new test; (b) added
`TestBundleFlowNoticeRefiresOnReviewBackReentry`, a **live** drive of
`bundleFlow`: gather the pinned card, dismiss the first notice, Back at
review (resumes the gather with the card still on the pile), Done again,
and assert the notice fires a SECOND time. No debounce latch was added — the
behaviour is now pinned as intentional rather than accidental, and the test's
own failure message tells a future implementer to update it (not delete it)
if a latch is ever added.
**RED→GREEN note:** the first version of this test asserted the first fire
immediately after preloading `ctx.syswBundleSeeds`, without ever clicking
"Done adding cards" — the preloaded seeds complete the card but the gather
screen still waits for the Done click before proceeding. Fixed by pumping to
`"mk1 keys: 1"` and clicking `Button3` before expecting the notice.

**M4 — done.** `TestBuildPolicyGatherSilentOnCleanTwinLive`'s post-Done
pump budget raised from 8 to 64 frames, matching the mismatch twin's
`pumpUntil(..., 64)` budget, with a comment explaining why the two must
match.

**N1 — done.** `design/agent-reports/impl-device-csid-warning.md`
Deviation 2's "5 sites total" corrected to 7, with the exact line list
(`singlesig_verify.go:225,229,231,235`, `multisig_verify.go:1171,1196,1208`)
— re-verified against the current dev-warn tree by `grep -n csidNote`
before writing (all seven lines confirmed present, matching exactly).

**N2 — done.** Same report's "gofmt -l on every touched/new file: empty"
line clarified: named as "touched/new file (not `gofmt -l gui/`)", plus a
parenthetical naming the three files a bare `gofmt -l gui/` DOES flag
(`transaction.go`, `transaction_golden_test.go`,
`transaction_txrecord_test.go`) so a future reader isn't surprised.
Independently re-verified in this environment: `gofmt -l gui/` on this
box's go1.26.4 toolchain flags exactly those three files, none touched by
this diff.

**N3 — done.** `README.md`'s regenerate snippet relabeled from ```` ```sh
```` to ```` ```bash ```` with a one-line note that it needs bash (fish
lacks herestrings); unchanged commands (already confirmed reproducible
under bash by the review).

## Build gate (dev-warn tree)

Go: `/home/bcg/.local/go/bin/go` (go1.26.4; `go` not on PATH in this
environment, matches the review's own toolchain note).

| check | result |
| --- | --- |
| `gofmt -l` on the 3 touched files (`bundle.go`, `multisig_build_census.go`, `csid_warning_test.go`) | empty |
| `go vet ./mk/... ./gui/...` | clean except 3 pre-existing `testing.ArtifactDir requires go1.26 or later` findings in files untouched by this diff (toolchain-version artifact, same class the review already documented) |
| `go test ./mk/...` | ok, 0.017s |
| `./gui/` via `scripts/gui-shard-test.sh ./gui/ 24` | **ok — all 1058 tests**, exhaustive 24-way partition asserted, ~24-25s wall (1056 baseline + 2 new: `TestBundleGatherConsumersAreAccountedFor`, `TestBundleFlowNoticeRefiresOnReviewBackReentry`; `grep -c '^func Test' gui/csid_warning_test.go` = 23, up from 21) |
| `git status --short` (dev-warn) | `M gui/bundle.go`, `M gui/csid_warning_test.go`, `M gui/multisig_build_census.go` — no other files, no commits |
| screenshot/marker-form freeze | verified: every production-file diff (`bundle.go`, `multisig_build_census.go`) is a `//` doc-comment addition only; no `fmt.Sprintf`/string-literal changed |

## mnemonic-engrave tree

`git status --short`: `M design/FOLLOWUPS.md`, `M design/SPEC_device_csid_warning.md`,
`M design/agent-reports/impl-device-csid-warning.md`,
`M design/journeys/csid-tags/README.md`. No commits.

## Findings not fixed as literally described

- **I1's spec-side twin** (see I1 disposition above): the report names the
  imprecision as present in both the README and the spec's Acceptance
  section, but its stated Remedy only edits the README. Left the spec
  paragraph untouched per "fix exactly what the report names"; flagged
  above and here in case the parent wants it folded into F-447 or a fresh
  follow-up.

Everything else in the report (C1's 4 parts, I1's README, M1-M4, N1-N3) was
fixed as described, with the two RED→GREEN test-authoring mistakes recorded
above (both caught by actually running the suite, both fixed before this
report was written).
