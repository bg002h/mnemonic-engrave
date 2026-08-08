# Plan B Phase B1 — R0 round 0 (sonnet)

**Artifact under review:** `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB1.md`
at `ffa2d29`, plus the §10.3 "Back IS Lock" amendment at `5b31211`.

**Reviewer:** independent sonnet agent. **Author did not review its own work.**

**Brief:** one question — can a single implementer execute this without a
question or a defect. Scope forbade auditing §10, Phase A, or the rest of §10.3.
The brief listed all 26 `file:line` + 5 `pkg.Symbol` citations as already
resolved by `scripts/plan-cite-gate.sh`, and pointed the budget at the plan's
stated blind spot: **`plan-build-gate.sh` is Rust-only, so every Go fragment in
this plan had been compiled by nothing.**

**Verdict: 0C / 4I / 0M / 0N.**

---

## Report, verbatim

### I Task 1c changes `layoutMainPager`'s signature but never updates either of its two call sites
**Where:** Task 1c, table row for `gui/gui.go:1938`
**Claim:** "`const npages = int(bip85Derive) + 1` → parameter `lastNav program`" — i.e. `layoutMainPager` gains a `lastNav` parameter, listed as one of the "four boundary sites… machine-located via grep, not recalled."
**Reality:** `layoutMainPager(buf *op.Buffer, th *Colors, page program)` has exactly two call sites in the repo, neither in the plan's tables:
```
gui/gui.go:1736:      inner, sz := layoutMainPager(&ctx.B, th, m.prog)
gui/text_program_test.go:79:  _, sz := layoutMainPager(&ctx.B, &descriptorTheme, backupWallet)
```
The second is an *existing* test (`TestStartScreenFitsAtEightPagerDots`), part of today's green suite. Neither call passes a fourth argument, and grep across the whole repo (not just `gui.go`) turns up no others — this is the complete set.
**Consequence:** Changing the function's arity without updating both callers is a straight `go build`/`go test ./gui/...` failure — two call sites, one of them a pre-existing test the plan never proposes touching. Per the plan's own Global Constraints ("`go test ./...` GREEN means exactly TWO setup failures… Any third failure is a regression"), this would register as an unplanned build break the implementer has to diagnose and fix ad hoc. It's exactly the class of defect the plan itself says Go fragments are unchecked for (F-74).
**Fix:** Add both call sites to Task 1c's table: `gui/gui.go:1736` → `layoutMainPager(&ctx.B, th, m.prog, m.lastNav)`; `gui/text_program_test.go:79` → pass `bip85Derive` as the fourth argument (behaviorally correct, since that test only exercises the no-payload state). Note also that `text_program_test.go:8-11`'s comment ("bip85Derive must stay the last navigable program") becomes stale once B1 ships and should be updated in the same commit.

### I Task 4's plate-list labeling scheme cannot produce the labels §10.2.2 requires
**Where:** Task 4, point 2 ("Entries are labelled from `AdmittedRecord.Class` and a 1-based index within that class — `mk1 1/2`, `md1 2/3`")
**Claim:** The label (specifically distinguishing "mk1" from "md1") comes from `AdmittedRecord.Class`.
**Reality:** `seal/record.go`'s `Classification` enum has exactly one value covering both: `ClassMDMK`. `Classify()` returns it identically for `codex32.ValidMD(s) || codex32.ValidMK(s)` — there is no `ClassMD`/`ClassMK` split, and `AdmittedRecord{Record []byte; Class Classification}` carries no other field that distinguishes them. `AdmitSection` returns one `AdmittedRecord` **per raw record** (not per decoded card), so `p.Public` is a flat list of records all tagged `ClassMDMK`. Separately, for a payload with several same-HRP cards (Vector G: "`mk1` ×6 (three cards, 2 chunks each)"), a flat "index within that class" conflates three distinct cosigner cards into one running counter (`mk1 1/6` … `mk1 6/6`), unlike the fork's own existing precedent for exactly this kind of label — `bundlePlatePlan` (`gui/bundle_flow.go:300`) computes `cardIdx`/`cardTotal`/`plateIdx`/`plateTotal` from a pre-grouped `bundleCard` structure, which `seal.AdmittedRecord` has no equivalent of.
**Consequence:** As specified, Task 4 cannot even tell "mk1" from "md1" from the data it says to use, let alone reproduce §10.2.2's normative example format for a multi-cosigner wallet (Vector G — the exact vector Task 4's own paging test uses). The implementer either invents undocumented re-classification/re-grouping logic mid-implementation, or ships labels that don't match the NORMATIVE example. Task 4's own test ("Labels come from Class: construct a payload whose records classify as mk1 and md1…") doesn't specify a vector with multiple same-HRP cards, so this can ship untested.
**Fix:** Specify how md1 is distinguished from mk1 for the label (e.g. re-derive via `codex32.ValidMD`/`ValidMK` on `r.Record` at the UI layer), and specify whether the index is per-card-chunk or per-class-flat; if per-card, specify how the plate list re-groups `p.Public` into cards (duplicating or exposing `seal`'s internal `groupCards`/`cardKey`). Extend Task 4's label test to Vector G's multi-card mk1 case.

### I Task 5's reuse of `mdmkFlow` drags in an NFC chunk-gatherer that hangs on any chunked record
**Where:** Task 5 ("Reuse `mdmkFlow`… it already offers TEXT+QR / TEXT / QR-ONLY and the `md1`/`mk1` inspect paths")
**Claim:** Reusing `mdmkFlow` verbatim on a payload's `AdmittedRecord` is presented as a benefit, explicitly including "the md1/mk1 inspect paths."
**Reality:** `mdmkFlow` (`gui/gui.go:2024`) prepends an "Inspect key"/"Inspect descriptor" choice that calls `mk1GatherFlow`/`md1GatherFlow` (`gui/mk1_inspect.go:156`, `gui/md1_gather.go:79`). Both prime a **fresh gatherer** with only the one string passed in (`g.offer(first)`), and if that alone isn't `complete()` — true for any card that's chunked, which is the spec's own examples: single-sig's md1 alone is 3 records, Vector G's `md1` is one 6-chunk card and `mk1` is three 2-chunk cards — they open `ctx.Platform.NFCReader()` and block waiting for the operator to tap the **remaining physical NFC tags**. A payload-derived record has no NFC tag to supply those chunks; the payload already holds every chunk in `p.Public`, but the gatherer has no access to them. Selecting "Inspect" on any chunked payload record leaves the operator on a scan-waiting screen with no way to view the card's own data (recoverable only via that screen's own Back, since `ChoiceScreen`'s cancel button and this gatherer's Back are separate, non-wiping, and architecturally fine — that specific nav-budget question checks out).
**Consequence:** A dead-end interaction on the common case (chunked cards), not a data-loss or funds-safety issue, but a real functional regression the plan explicitly endorses as a reused benefit without qualification. Task 5's own test ("selecting an entry reaches the engrave screen… returning… lands back on the plate list") only exercises the plain engrave path, never the Inspect branch, so this ships untested.
**Fix:** Either strip the "Inspect" choice for B1 (call `validateMdmk` + `ChoiceScreen` + `NewEngraveScreen` directly, as `bundleEngrave` does, rather than the whole of `mdmkFlow`), or explicitly test and document that Inspect is out of scope / disabled for payload-sourced records.

### I `cmd/emu/platform.go`'s `PayloadReader` "from a flag" has no flag mechanism to attach to
**Where:** Task 1a, table row for `cmd/emu/platform.go:189`
**Claim:** `seal.FileReader{Path: …} from a flag, else nil` — described as adjacent, small work, and "not skip[pable]" because it's what makes B1 "drivable from `seal/testdata/vectors.json` without hardware."
**Reality:** `cmd/emu` is a `//go:build js` browser build (`GOOS=js GOARCH=wasm go build`, per `cmd/emu/build.sh`) invoked via `wasm_exec.js` inside a static HTML page — there is no `flag.Parse()` or `os.Args`-based CLI anywhere in the package, and none of the existing `syscall/js` usage in `platform.go` reads a URL query string or any other external input channel. `os.Args` under the default browser-hosted `wasm_exec.js` glue carries no real argv.
**Consequence:** A `flag.String(...)` call would compile and run without error but never receive a value in the browser — the FileReader path would silently never activate, defeating the plan's own stated purpose for including it, with no build/test failure to reveal the gap. Note this doesn't block Task 1d's actual automated tests, which drive B1 through `testPlatform` (correctly specified as "a settable field, default nil"), not through `cmd/emu` — so the blast radius is the emulator affordance only, not the test suite.

---

**Checked and found correct** (no finding): the `seal.Reader`/`XIPReader`/`FileReader` build-tag split (`tinygo` vs `!tinygo`) is sound and creates no cycle — `seal.Reader` itself is untagged in `read.go`, and `cmd/emu` is confirmed built via plain `go build GOOS=js` (not TinyGo), so `!tinygo`'s `FileReader` is exactly what's available there; `cmd/controller/platform_sh2.go`'s `tinygo && rp` tag correctly gets `XIPReader`. The append-vs-insert argument in "A deliberate departure…" is verified against the real `layoutMainPager`/`layoutMainPlates` dot-fill and wrap logic and holds up. The compile-time guard's arithmetic (`qaProgram - unlockPayload` = 1, matching today's `qaProgram - bip85Derive` = 1) is correct. `showError`'s signature, `ConfirmWarningScreen`'s fields/return values, `seal.Opener.Inspect`'s error behavior (including `ErrTooManyRecords` wrapping via `%w`), and `len(p.Public)` matching the §6.6 hash's `public_record_count` all check out against source. The "Back IS Lock" three-slot budget does not conflict with nested screens' own independent nav bars (`ChoiceScreen`, `mk1GatherFlow`'s own Back) — each screen owns its own `layoutNavigation` call, so a non-wiping Back one level down is architecturally unproblematic in B1 (nothing to wipe regardless).

VERDICT: 0C / 4I / 0M / 0N
