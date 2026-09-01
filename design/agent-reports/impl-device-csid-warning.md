# Implementation — SH2 device: mk1 chunk-set-id mismatch warning

**Worktree:** `/scratch/code/shibboleth/sh-worktrees/dev-warn`, branch
`impl/device-csid-warning`, base `2337ed3` (seedhammer fork). Left dirty and
unstaged per the dispatch brief — no commits made.

**Spec:** `design/SPEC_device_csid_warning.md`, GREEN after 3 R0 rounds
(`design/agent-reports/R0-device-csid-warning-r{1,2,3}.md`).

## Diff summary

```
gui/bundle.go                  | 100 ++++++++++++++++++++++++++++++++++++++++-
gui/bundle_flow.go             |   4 +-
gui/mk1_inspect.go             |  22 +++++++++
gui/multisig_build.go          |   4 ++
gui/multisig_build_census.go   |   8 ++--
gui/multisig_build_payload.go  |   2 +-
gui/multisig_verify.go         |  12 +++--
gui/singlesig_verify.go        |  13 ++++--
gui/wallet_policy.go           |   2 +
mk/chunk_set_id_parity_test.go |  37 ++++++++++++---
mk/encode.go                   |  25 +++++++++++
11 files changed, 209 insertions(+), 20 deletions(-)
```
Plus two new, untracked files: `gui/csid_warning_test.go` (718 lines, 21 new
`Test*` funcs) and `cmd/emu/shots_csid_warning.js` (214 lines).

## Contract 1 — `mk.DerivedChunkSetID`

`mk/encode.go:64` — thin wrapper `top20(encodeBytecode(card))`; `Encode`
untouched. Extended the parity test (`mk/chunk_set_id_parity_test.go`): added
`Strings []string` to `csidCorpusRow`, and for every clean corpus row now
also asserts `DerivedChunkSetID(Decode(row.Strings))` == `derived_csid`, not
just `top20(canonical_bytecode_hex)`. RED (undefined symbol) → GREEN, 20/20
clean rows including the leading-zero row.

## Contract 2 — inspect flow

`gui/mk1_inspect.go`: `mk1Gatherer.chunked bool` (line 57), set from
`h.Chunked` at prime time (line 69) — never from `setID==0`/`total==1`
proxies. `decodeGathered` (line 264) gates the comparison on `g.chunked`,
calls `mk.DerivedChunkSetID`, and on mismatch shows a non-blocking
`showNotice` with the host-verbatim text before returning `(card, true)`.

RED→GREEN: `TestDecodeGatheredWarnsOnCSIDMismatch` (notice fires, body ==
corpus `warning_text` read from `mk/testdata/csid_ext_v0.1.json` at test
time — not transcribed, per the brief), `TestDecodeGatheredSilentOnCleanTwin`,
`TestMK1GathererChunkedFieldSetAtPrimeTime`,
`TestDecodeGatheredStillRefusesOnDecodeFailure` (regression: decode failure
path unchanged).

## Contract 3 — bundle-gatherer flow, all six consumers

`gui/bundle.go`: `bundleCard` gained `csidMismatch/declaredCSID/derivedCSID`
(line 49), computed once in `offerChunkedMK1` at set completion. Shared
helpers: `csidMismatchWarningText` (63, host-verbatim), `csidMarker` (78,
form `" [csid 12345!ef12f]"` — my proposal, screenshotted below),
`bundleCSIDNote` (91, the verify-readback line-marker),
`showBundleCSIDMismatchNotices` (115, the set-completion modal).

Per-surface wiring:
- **Engrave Bundle** (`bundle_flow.go:46`) / **Wallet Policy**
  (`wallet_policy.go:108`): `showBundleCSIDMismatchNotices` called right
  after gather returns; review-list marker at `bundle_flow.go:364`.
- **Build Policy** (`multisig_build.go:201`): same modal call, plus marker
  in `buildPlateCensusLines` and `buildPlateInventoryLines`
  (`multisig_build_census.go:53,89`) and `buildPayloadCardsLines`
  (`multisig_build_payload.go:295`).
- **Engrave Multisig**: no code change — `extractSuppliedMd1` already
  refuses any mk1 unconditionally before a card renders. Added
  `TestEngraveMultisigRefusesAnyMK1BeforeCSIDCouldMatter`, driven with a
  REAL mismatched corpus card (not a synthetic `bundleCard{}`), on top of
  the pre-existing `TestExtractSuppliedMd1`/"any mk1 present -> refuse"
  subtest (`gui/multisig_supply_test.go:42`, confirmed present, unmodified).
- **Verify readbacks** (`multisig_verify.go:793`, `singlesig_verify.go:153`):
  `bundleCSIDNote(cards)` computed once after gather, appended to the
  terminal verdict text — the PASS message and both comparator-FAIL
  messages in each flow — never a separate modal.

**Modal timing, resolved during implementation (not fully specified by the
R0 text):** the six consumers share ONE gather function
(`bundleGatherFlowResume`), so the notice cannot live inside the shared
loop without also firing for Multisig/verify. Resolved by having each of
the three "modal" callers invoke `showBundleCSIDMismatchNotices` once,
**after** their own gather call returns (i.e. at whole-gather-set
completion / "Done adding cards", not at the instant one card's chunks
complete mid-scan) — confirmed live in
`TestBuildPolicyGatherShowsCSIDMismatchNoticeLive`.

RED→GREEN, 16 new tests covering: `offerChunkedMK1` computing the fields
(mismatch + clean), `csidMarker`'s exact form, `bundleReviewFlow`'s marker,
both census-doc functions' markers, `buildPayloadCardsLines`'s marker, the
notice firing/silent (direct + **live end-to-end** through
`buildMultisigPolicyFlow` off a real systemwide payload, both mismatch and
clean-twin arms), `bundleCSIDNote` firing/silent, and a **live end-to-end**
`singleSigVerifyFlow` drive (typed seed → wallet-type → passphrase → NFC
readback via `ctx.syswBundleSeeds` → "Verify Failed") proving the marker
appears on the verdict screen and that dismissing it exits the flow (no
second modal). `multisigVerifyFlow`'s per-leg loop makes an equivalent live
drive disproportionate; covered instead by a mechanical wiring guard
(`TestMultisigVerifyFlowWiresCSIDNoteIntoVerdicts`, `funcBody`-idiom, mirrors
`TestGatherTitleReachesTheRefusalsToo`) — **this test caught a real
transcription miss**: my first `replace_all` edit only wired `csidNote` into
one of the two comparator-FAIL sites in `multisig_verify.go`; the guard
failed with "found 1, want 2" and was fixed before any review saw it.

## Contract 4 — single-string mk1

No code change. `clsSingleMK1Refuse` already pinned at
`gui/bundle_test.go:115-116` (`TestClassifySingleMK1Refuse`); cited, not
re-added.

## Mutation gate

Two independent mutations, each `&& false` on the comparison's boolean
condition: `gui/mk1_inspect.go:265` (inspect flow) and `gui/bundle.go:298`
(bundle-gatherer flow). Ran `go test ./gui/ -run 'CSID|CsID|Csid'` before,
during, after:

| state | PASS | FAIL |
| --- | --- | --- |
| baseline | 18 | 0 |
| both mutated | 8 | 10 |
| reverted | 18 | 0 (byte-identical PASS set to baseline) |

The 10 tests that failed under mutation span BOTH flows:
`TestDecodeGatheredWarnsOnCSIDMismatch` (inspect),
`TestOfferChunkedMK1ComputesCSIDMismatch`, `TestCSIDMarkerForm`,
`TestBundleReviewFlowMarksCSIDMismatch`,
`TestBuildPlateCensusLinesMarksCSIDMismatch`,
`TestBuildPayloadCardsLinesMarksCSIDMismatch`,
`TestShowBundleCSIDMismatchNoticesFiresOnMismatch`,
`TestBuildPolicyGatherShowsCSIDMismatchNoticeLive`,
`TestBundleCSIDNoteFiresOnMismatch`,
`TestSingleSigVerifyCSIDNoteOnFailureLive` (all bundle-gatherer-flow). Every
"silent on clean twin" test correctly stayed green throughout (the mutation
only removes detection, never introduces a false positive). Both mutations
reverted; `grep -rn MUTATION-GATE gui/ mk/` is empty.

## Screenshot deliverable

`cmd/emu/shots_csid_warning.js` (new driver, modeled on `shots_seating.js` /
`shots_operator.js`) presents an mk1 chunk **at the emulator's home screen**
(`gui/gui.go`'s `StartScreen.Flow` → `engraveObjectFlow` → `mdmkFlow`, the
real "tap a stray key card" door), walks "Inspect key", asserts the pinned
row's second chunk produces the warning modal and the clean twin's does not
(it does not just capture — it fails the run if either assertion is false),
then (best-effort) also walks Engrave Bundle to capture the review-list
marker.

Driven headlessly via `design/journeys/capture_csid_warning.py` (new,
modeled on `capture_seating.py`) using Python Playwright + Chromium
(pre-installed on this machine) against `emu.wasm` built from THIS
worktree (`--fork-dir` override — the other `capture_*.py` scripts assume a
sibling `../../../seedhammer` checkout, not a worktree). Ran successfully;
both assertions passed (`modalText` contains the host warning text,
`cleanSilent: true`); the bonus bundle-review shot also succeeded.

**One fix made during the run:** the driver's first pass got stuck at the
"mk1 key" chooser after the pinned card, because `mk1DisplayFlow`'s Back
returns to `mdmkFlow`'s own chooser loop, not to the carousel — a second
Back is required to reach the home screen. Caught by the walk itself
timing out, fixed, re-ran clean.

Saved to (both required by the brief):
- `/scratch/code/shibboleth/mnemonic-engrave/design/journeys/csid-warning-modal.png`
  — the Contract-2 warning modal, host text verbatim.
- `/scratch/code/shibboleth/mnemonic-engrave/design/journeys/csid-warning-bundle-review.png`
  — bonus: the bundle review list showing `1. mk1 key OK [csid 12345!ef12f]`.

Both viewed and legible: title "Inspect key" / body wraps cleanly across 5
lines in the first; "Bundle" / "1 cards verified:" / the marked line / card
summary in the second.

## Tag payloads

`design/journeys/csid-tags/` (new): four NDEF files via the host `me`
binary (no subcommand — its single-string converter), one per chunk of the
pinned/clean corpus rows, generated from the corpus's own `strings` arrays
(never hand-minted): `tag1-pinned-chunk0-of-2.ndef` (119 B),
`tag2-pinned-chunk1-of-2.ndef` (88 B), `tag3-clean-chunk0-of-2.ndef`
(119 B), `tag4-clean-chunk1-of-2.ndef` (88 B) — all mode 0600 (`--out`'s
default). `README.md` in the same directory names the tap order and
expected outcome per the spec's on-device acceptance section.

**Parse-back proof:** a scratch Go probe (`cmd/csidtagverify`, written, run,
then `rm -rf`'d — `git status` confirms zero residue) read each file back
through the fork's own `ndef.NewMessageReader` → `ndef.NewRecordReader`
chain and reproduced its source mk1 string byte-exact; the two 2-chunk sets
were then reassembled via `mk.Decode` and confirmed to (a) share identical
key content (`m/48h/0h/0h/2h`, same xpub) and (b) reproduce
declared=12345/derived=ef12f (mismatch=true) for the pinned set and
declared=derived=ef12f (mismatch=false) for the clean set — output:
`ALL CHECKS PASSED`.

## Gates

- `go test ./mk/...`: green (0.018–0.029s across runs).
- `./gui/` via `scripts/gui-shard-test.sh ./gui/ 24`: green, 1056/1056
  top-level tests (1049 `Test*` + 7 `Example`/`Fuzz`) across an
  **exhaustive, asserted** 24-way partition; ~22s wall. Baseline (stashed
  diff) was 1028 `Test*` funcs; this cycle added exactly 21 — matches
  `grep -c '^func Test' gui/csid_warning_test.go`.
- `gofmt -l` on every touched/new file: empty.
- `go vet ./mk/...`: clean. `go vet ./gui/`: 2 pre-existing findings
  (`testing.ArtifactDir requires go1.26 or later`) in
  `freetext_sizeproof_golden_test.go` / `transaction_golden_test.go` —
  neither touched by this diff; confirmed present on baseline `2337ed3` via
  `git stash` before/after (go1.26.4 toolchain vs. the repo's pinned
  `go 1.25.10`, an environment mismatch unrelated to this work).

## Deviations, and why

1. **Modal-firing granularity for the three interactive-gather surfaces**
   was left to the implementer by the spec text ("notice modal at set
   completion") and is resolved above (fires once, after the WHOLE gather
   session returns — not per-card mid-scan). This is the one design
   decision this diff makes that the spec did not pin exactly; it is
   directly verified live (`TestBuildPolicyGatherShowsCSIDMismatchNoticeLive`)
   rather than merely asserted.
2. **Verify-readback note scope** (Contract 3's "line-marker only"): applied
   to the PASS message and the comparator-FAIL messages in both flows
   (5 sites total), not to the pre-gather refusals (empty expectation /
   empty policy) or the mid-flow seed/passphrase retry screens in
   `multisigVerifyFlow`'s per-leg loop, since those refusals are about a
   DIFFERENT problem and occur before or independent of a specific card's
   content. This is a scope call, not a spec violation — Contract 3 names
   the two functions and "line-marker only, NO modal" as the shape, not an
   exhaustive site list.
3. **Marker form** `" [csid 12345!ef12f]"` is the implementer's proposal
   per the spec's own text ("frozen at the screenshot gate"); both required
   screenshots show it in place for the operator's approval.
4. Contract 4 required no new production code; the existing pins were
   verified current and cited rather than duplicated.

## Files touched (absolute paths)

- `/scratch/code/shibboleth/sh-worktrees/dev-warn/mk/encode.go`
- `/scratch/code/shibboleth/sh-worktrees/dev-warn/mk/chunk_set_id_parity_test.go`
- `/scratch/code/shibboleth/sh-worktrees/dev-warn/gui/mk1_inspect.go`
- `/scratch/code/shibboleth/sh-worktrees/dev-warn/gui/bundle.go`
- `/scratch/code/shibboleth/sh-worktrees/dev-warn/gui/bundle_flow.go`
- `/scratch/code/shibboleth/sh-worktrees/dev-warn/gui/wallet_policy.go`
- `/scratch/code/shibboleth/sh-worktrees/dev-warn/gui/multisig_build.go`
- `/scratch/code/shibboleth/sh-worktrees/dev-warn/gui/multisig_build_census.go`
- `/scratch/code/shibboleth/sh-worktrees/dev-warn/gui/multisig_build_payload.go`
- `/scratch/code/shibboleth/sh-worktrees/dev-warn/gui/multisig_verify.go`
- `/scratch/code/shibboleth/sh-worktrees/dev-warn/gui/singlesig_verify.go`
- `/scratch/code/shibboleth/sh-worktrees/dev-warn/gui/csid_warning_test.go` (new)
- `/scratch/code/shibboleth/sh-worktrees/dev-warn/cmd/emu/shots_csid_warning.js` (new)
- `/scratch/code/shibboleth/mnemonic-engrave/design/journeys/capture_csid_warning.py` (new)
- `/scratch/code/shibboleth/mnemonic-engrave/design/journeys/csid-warning-modal.png` (new)
- `/scratch/code/shibboleth/mnemonic-engrave/design/journeys/csid-warning-bundle-review.png` (new)
- `/scratch/code/shibboleth/mnemonic-engrave/design/journeys/csid-tags/` (new dir: 4 `.ndef` files + `README.md`)
