# Plan B Phase B1 — R0 round 1 (sonnet), scoped fold re-review

**Artifact:** `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB1.md` after the
round-0 fold. **Primary object of review: `git diff 2b667f4..440f47a`** — exactly what
changed in response to round 0 and nothing else.

**Brief:** one question — did the fold fix each of the four findings, and did the fold
itself introduce a new defect. NOT a fresh audit. Round 0's "checked and found correct"
list was declared settled and off-limits. All 38 file:line + 5 pkg.Symbol citations were
declared already resolved by `plan-cite-gate.sh`.

**Verdict: 0C / 1I / 0M / 0N.**

---

## Report, verbatim

# Fold Re-Review — Plan B Phase B1, Round 1

## Fold-verification table

| # | round-0 finding | fixed? | notes |
|---|---|---|---|
| 1 | `layoutMainPager` arity change breaks both callers unnoted | FIXED | Both call sites now in Task 1c's table with the exact correct transforms; confirmed against current source (`gui/gui.go:1736`, `gui/text_program_test.go:79` are still the only two callers). Fold also caught a real adjacent gap round 0 missed (no test proves a 9th dot fits the panel) and added a width test that is concrete and non-redundant with Task 1d's separate dot-count assertion. |
| 2 | Task 4's label scheme can't distinguish mk1/md1 or reproduce §10.2.2 for multi-card wallets | PARTIAL | The new `HRP`/`CardIndex`/`CardTotal`/`PlateIndex`/`PlateTotal` fields and label rule are correct in isolation — I compiled `groupCards` against the real vector D and vector G strings (`seal/testdata/vectors.json`) and confirmed vector D's mk1/md1 each form exactly one card (reproducing `mk1 1/2`/`md1 2/3` literally) and vector G's mk1 forms three distinct cards (reproducing the `mk1 2/3 · 1/2` form). But Task 4a's premise — "populated by `AdmitSection` from the grouping it already computes" — doesn't match `AdmitSection`'s actual control flow. See new finding below. |
| 3 | `mdmkFlow` reuse drags in an NFC gatherer that hangs on chunked records | FIXED | The replacement composition (`validateMdmk` → `ChoiceScreen` → `NewEngraveScreen`) is not just plausible — it is the *exact* pattern already compiling and passing as `bundleEngrave` (`gui/bundle_flow.go:327-354`). The new regression test ("no path reaches `mk1GatherFlow`/`md1GatherFlow`") is concretely specified. F-76 is filed with the right owning phase and is consistent with the reversed Task 5 text. |
| 4 | `cmd/emu`'s flag-based `PayloadReader` has no flag mechanism to attach to | FIXED | Verified `cmd/emu/platform.go:1` is `//go:build js` and no `flag`/`os.Args` usage exists anywhere in the package. The fix (`return nil`) mirrors the file's own existing `NFCReader` pattern almost verbatim ("nil is a SUPPORTED value... gui checks it... offers Back-only where a scan would go" — `cmd/emu/platform.go:184-189`), and correctly notes Task 1d's tests go through `testPlatform`, not `cmd/emu`. |

## New findings

### [I] Task 4a's "grouping it already computes" doesn't exist where it's needed, and the obvious way to make it exist breaks an existing Phase A test
**Where:** Task 4a, "Surface it instead" paragraph and the `AdmittedRecord` field-addition block.
**Claim:** "Add to `AdmittedRecord`, populated by `AdmitSection` from the grouping it already computes" — framed as purely additive, with "Phase A's vector tests must still pass unchanged; if any needs editing, this change is wrong."
**Reality:** Read `seal/record.go:158-191`. `AdmitSection` builds `out` (the `AdmittedRecord` slice) in a per-record loop that runs pass 1 (lowercase) and pass 2 (classify + allow-list) — and only *after* that whole loop completes does it conditionally call `decodePublicSet(strs)` for `SectionPublic`, which internally calls `groupCards` and then discards `keys`/`groups`, returning only an `error`. So at the point `out` is built, `AdmitSection` has no grouping in hand at all; the grouping that does exist is transient, computed later, and never escapes `decodePublicSet`'s local scope.

More importantly, I compiled a probe against `record_test.go:180`'s `TestPublicSectionRefusesASecret`, which does `AdmitSection(bs([]string{mn}), SectionPublic)` for a BIP-39 mnemonic (`ClassMnemonic`, not `ClassMDMK`) and asserts `errors.Is(err, ErrRecordNotPermitted)`. `cardKey`'s `default` branch (`seal/record.go:290-292`) is documented as "Unreachable behind the allow-list" and returns `ErrUndecodableCardSet` instead — a *different* sentinel. If an implementer computes the card grouping before or independently of the classify+permitted filter (a natural reading of "surface the grouping" as a self-contained helper), a non-md1/mk1 public record hits `cardKey`'s fail-closed branch first and the function returns `ErrUndecodableCardSet` instead of `ErrRecordNotPermitted`, and `TestPublicSectionRefusesASecret` fails — directly violating the "vector tests must pass unchanged" promise Task 4a itself states.
**Consequence:** An implementer following Task 4a literally either (a) discovers mid-implementation that the described data flow doesn't exist and has to invent the restructuring themselves — exactly the "undocumented invention" round 0's finding 2 already objected to — or (b) picks the natural-looking but wrong order and silently breaks an existing green test, which either surfaces as an unplanned setup failure (violating the plan's own GREEN baseline) or, worse, gets "fixed" by loosening the test rather than the implementation.
**Fix:** State explicitly in Task 4a that the grouping must be computed **only after** the existing pass-1+2 loop has admitted every record (i.e., only over records already confirmed `ClassMDMK` — the only case that can reach card grouping in `SectionPublic`), and that it must reuse — not race — the same `groupCards` call `decodePublicSet` already makes at that point. Concretely: thread `keys`/`groups` out of `decodePublicSet` (change its return type, or have `AdmitSection` call `groupCards` once and pass the result into `decodePublicSet`) and backfill `out`'s new fields in a step after the per-record loop, never before or during it.

## Verdict

`VERDICT: 0C / 1I / 0M / 0N`

**Summary:** Three of round 0's four findings are cleanly fixed and hold up against the real source (`gui/gui.go`, `gui/bundle_flow.go`, `cmd/emu/platform.go`) and, for finding 2's label rule specifically, against real vector data (`seal/testdata/vectors.json` vectors D and G) via a compiled `groupCards` probe. Finding 2 is only partially closed: the *label rule* is right, but Task 4a's claim that the required grouping is already sitting in `AdmitSection`'s hands is false, and the natural way to make it true — computing the grouping before the classify/permitted filter — breaks `TestPublicSectionRefusesASecret` (`seal/record_test.go:180`) by changing which error a disallowed public record returns. This needs a round 2 fold on Task 4a specifically; nothing else in the fold needs revisiting.
