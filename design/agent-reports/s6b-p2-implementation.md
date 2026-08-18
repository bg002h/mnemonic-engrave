# S6b P2 implementation — plate marking (spec §1)

**Worktree:** `/scratch/code/shibboleth/wt-s6b`, branch `s6b-pre-flash`.
**Commits:** `a0c4bd6` (backup-package mechanism), `a855a6e` (gui-package
wiring), both on top of P1's `2c18a6f`.

Scope: spec `SPEC_s6b_pre_flash_cycle.md` §1 only. Gates 1.1, 1.2, 1.2a,
1.2b, 1.3.

---

## What changed and where

**`backup/backup.go`** (commit `a0c4bd6`):

- `Text` gains two optional fields, `Title, Footer string`.
- `EngraveText` renders them through a new `centerRow` closure — the same
  arithmetic `EngraveFitted`'s `centerInset` uses (`textLayout`'s
  `holeChars*charWidth`), applied at `plate.fontMM()` and `plate.Font`.
  Title renders at plate row 0 (`offy := params.I(outerMargin)`); if a title
  is present the body's starting `offy` advances by one row. Footer renders
  at the LAST plate row, anchored from the bottom via the existing
  `footerRowY(params, plate.fontMM())` helper (already used by
  `EngraveFitted`), not appended after the body.
- `s == ""` is a no-op inside `centerRow` — no `Yield` call at all — which
  is what keeps an unmarked plate byte-identical to before these fields
  existed.

**`gui/gui.go`**: `validateMdmk(pl Platform, s string) (...)` becomes
`validateMdmk(pl Platform, s, title, footer string) (...)`. It plumbs
`title`/`footer` straight into every variant's `backup.Text{}` literal
(TEXT+QR, TEXT ONLY, QR ONLY) and evaluates no predicate itself.

**`gui/bundle_flow.go`**: `bundlePlate` gains a `kind bundleCardKind`
field, populated in `bundlePlatePlan` from `bundleCard.kind`. New helper
`bundlePlateMark(kind, title, footer) (string, string)` returns `"", ""`
for `cardMS1`, the caller's values verbatim otherwise. `bundleEngrave`
grows two trailing params `markTitle, markFooter string`, calling
`bundlePlateMark` per plate before each `validateMdmk` call.

**`gui/singlesig.go`**: new helper `singleSigPlateMark(full, hasPassphrase
bool, masterFP uint32) (title, footer string)` — the only place that
computes non-empty marking. `full` is R-A's predicate ("the set contains a
seed") for this flow. `masterFP` is already the combined fingerprint when
a passphrase was entered (from `deriveSingleSigBundle` at line 107) and the
bare-seed one otherwise, so P2 needs no extra derivation. Called right
before the existing `bundleEngrave(ctx, th, "Engrave Single-Sig", cards,
...)` call at line ~190 (was :177 pre-P1).

**Every other call site** (`mdmkFlow` in gui.go, `unlock_platelist.go:222`,
`multiPlateEngrave` in derive_xpub.go:494, `bundleFlow`'s call in
bundle_flow.go, `multisig.go:291`, `multisig_build.go:402`) passes `"",
""` explicitly — Go has no default parameters, and the spec prohibits a
variadic tail or shared state as a shortcut.

**Test call sites updated to compile**: `gui/mdmk_gui_test.go`,
`gui/engraved_hook_test.go`, `gui/bundle_engrave_test.go`,
`gui/singlesig_engrave_test.go` — all pass `"", ""`.

**New test file**: `gui/s6b_plate_marking_test.go` (gates 1.2, 1.3, plus a
supplementary `singleSigPlateMark` pin).

---

## Per-gate TDD evidence

### GATE 1.1 — empty title/footer, goldens don't move

Not a new test: the existing `TestText` (backup/backup_test.go) already
constructs `Text{Paragraphs: ..., Font: sh.Font}` with no `Title`/`Footer`
and compares against the frozen goldens. **This is a regression pin, green
throughout — not red→green.** Ran before implementing (PASS, using the
pre-P2 struct) and after (PASS, using the post-P2 struct with the new
fields left at their zero value):

```
=== RUN   TestText
--- PASS: TestText (0.01s)
    --- PASS: TestText/1-shards-1 (0.03s)
    --- PASS: TestText/2-shards-1 (0.05s)
    --- PASS: TestText/0-shards-1 (0.06s)
PASS
ok  	seedhammer.com/backup	0.078s
```

`git status --porcelain backup/testdata/` reported nothing after every run
in this cycle, including the final whole-repo gate.

### GATE 1.2b — title is row 0, footer is the last row

New test `TestTextTitleFooterAreAbsoluteRows` (backup/engravetext_test.go).

**RED** (captured by `git stash push -- backup/backup.go`, reverting the
struct/EngraveText change, then running the new tests):

```
backup/engravetext_test.go:272:65: unknown field Title in struct literal of type Text
backup/engravetext_test.go:278:66: unknown field Footer in struct literal of type Text
backup/engravetext_test.go:286:69: unknown field Title in struct literal of type Text
backup/engravetext_test.go:292:70: unknown field Footer in struct literal of type Text
backup/engravetext_test.go:320:61: unknown field Title in struct literal of type Text
backup/engravetext_test.go:320:76: unknown field Footer in struct literal of type Text
backup/engravetext_test.go:328:68: unknown field Title in struct literal of type Text
FAIL	seedhammer.com/backup [build failed]
```

**GREEN** (`git stash pop` to restore, then rerun):

```
=== RUN   TestTextTitleFooterAreAbsoluteRows
--- PASS: TestTextTitleFooterAreAbsoluteRows (0.00s)
```

### GATE 1.2a — the title/footer budget, layout-based

New test `TestTextTitleFooterBudget` (same file, same RED capture above).

**GREEN, first pass, with the spec's cited budget (25):**

```
=== RUN   TestTextTitleFooterBudget
    engravetext_test.go:329: a 26-character title fits the screw-hole-free span; the 25-character budget has stopped binding
--- FAIL: TestTextTitleFooterBudget (0.00s)
```

**FINDING.** The spec's cited 25-character figure
(`SPIKE_s6b_q2_results.md` §3c) is a raw-string-width measurement, and that
spike document itself states the caveat: *"raw width UNDER-reports... the
implementation's gate must be the layout-based form, not this one."* Gate
1.2a's own wording requires exactly that layout-based form. Bisecting
through `EngraveText`/`textLayout` (not raw width) at `prodParams` finds
the real threshold is **28** (28 'W's fit, 29 do not):

```
n=25 fits=true
n=26 fits=true
n=27 fits=true
n=28 fits=true
n=29 fits=false
```

I updated the test to assert the measured 28, with the discrepancy
documented in the test's own doc comment rather than silently reconciled
with the spike. **This changes no gate outcome**: every title/footer S6b
introduces this cycle is ≤18 characters, well inside either number. Final
run:

```
=== RUN   TestTextTitleFooterBudget
--- PASS: TestTextTitleFooterBudget (0.00s)
```

### GATE 1.2 — the marking renders in every offered variant

New test `TestValidateMdmkMarkingRendersInEveryVariant`
(gui/s6b_plate_marking_test.go).

**RED** (`go vet ./gui/` before `validateMdmk`'s signature changed):

```
vet: gui/s6b_plate_marking_test.go:28:55: too many arguments in call to validateMdmk
	have (*testPlatform, string, string, string)
	want (Platform, string)
```

**GREEN** (after implementing gui.go/bundle_flow.go/singlesig.go):

```
=== RUN   TestValidateMdmkMarkingRendersInEveryVariant
--- PASS: TestValidateMdmkMarkingRendersInEveryVariant (0.01s)
```

For a representative md1 and mk1 string, this asserts (a) the offered
variant *label set* is identical between an empty-marking call and a
`"PASSWORD REQUIRED"`/`"COMB FP: FC60 C6DF"` call (so a band cannot
silently drop a variant — R-F's required gate), (b) `"QR ONLY"` is present
in that set, and (c) for every offered variant, the marked plate's ink
bounds (`bspline.Measure(pl.Spline).Bounds`) differ from the unmarked
plate's — i.e., the title/footer actually rendered, QR ONLY included.

### GATE 1.3 — only the single-sig engrave marks

Same RED capture as gate 1.2 (same new file, same `go vet` failure before
`validateMdmk`/`bundleEngrave` changed signature).

Two tests, both **GREEN** after implementation:

```
=== RUN   TestBundlePlateMarkSuppressesMS1
--- PASS: TestBundlePlateMarkSuppressesMS1 (0.00s)
=== RUN   TestOnlySingleSigMarksPlates
--- PASS: TestOnlySingleSigMarksPlates (0.00s)
```

`TestBundlePlateMarkSuppressesMS1` is the mechanism, unit-level:
`bundlePlateMark` returns the caller's title/footer verbatim for
`cardMK1`/`cardMD1` and `"", ""` for `cardMS1`.

`TestOnlySingleSigMarksPlates` is the SOURCE fact, same idiom as the
package's existing `TestBothEngraveFlowsGateOnACompletedSet` and
`TestMs1ReminderIsTitledForTheProgramThatShowedIt`: every call path named
in spec §1.3's table except `gui/singlesig.go`'s passes `"", ""`
verbatim in source, and `singlesig.go`'s does not.

**Supplementary, not a named gate but closes a real coverage gap**:
`TestSingleSigPlateMark` pins the exact spec §1.2 strings for all four
`(full, hasPassphrase)` combinations directly against `singleSigPlateMark`
— `PASSWORD REQUIRED`/`COMB FP: ...` for full+passphrase, no
title/`SEED FP: ...` for full+bare, `"", ""` for watch-only either way.
Mutation-checked: temporarily changed the full+passphrase footer from
`"COMB FP: "` to `"SEED FP: "` and reran — the test failed as expected
(`got ("PASSWORD REQUIRED", "SEED FP: FC60 C6DF"), want ("PASSWORD
REQUIRED", "COMB FP: FC60 C6DF")`) — then reverted and reconfirmed GREEN.

---

## The three source-string tests spec §1.3 named

Spec: *"Three tests assert the call text as a SOURCE STRING and must be
updated in the same commit: `gui/multisig_verify_report_test.go:940` and
`:942`, and `gui/bundle_abort_prose_test.go:258`."*

- `gui/multisig_verify_report_test.go:940`/`:942`
  (`TestBothEngraveFlowsGateOnACompletedSet`): confirmed necessary. These
  assert the literal call text `if bundleEngrave(ctx, th, "Engrave
  Multisig"/"Build Policy", cardsOut) != bundleEngraveDone {` via
  `strings.Contains`. Updated both to include the new trailing `, "",
  ""`. **RED before the fix** (ran after implementing everything else, to
  confirm the spec's claim mechanically rather than trust it):

  ```
  multisig_verify_report_test.go:949: multisig.go does not stop when the engrave is ABORTED.
      want the call gated as:
        if bundleEngrave(ctx, th, "Engrave Multisig", cardsOut) != bundleEngraveDone {
  multisig_verify_report_test.go:949: multisig_build.go does not stop when the engrave is ABORTED.
      want the call gated as:
        if bundleEngrave(ctx, th, "Build Policy", cardsOut) != bundleEngraveDone {
  --- FAIL: TestBothEngraveFlowsGateOnACompletedSet (0.00s)
  ```

  **GREEN after the fix.**

- `gui/bundle_abort_prose_test.go:258` (inside
  `TestMs1ReminderIsTitledForTheProgramThatShowedIt`): **did NOT need an
  edit.** Its check is `strings.Index(src, "bundleEngrave(ctx, th, ")`
  followed by a `strings.Contains` over a 120-character window — a prefix
  match that tolerates appended trailing arguments. Ran it unmodified
  (before touching any other test) and it PASSED. **Minor spec
  inaccuracy**, noted for the record; not acted on further since there is
  no functional gap — the test still exercises exactly what it always did.

---

## Full-suite gate

`export PATH="/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin:$PATH"`,
then `go test ./... -count=1`, stdout and stderr captured to separate
files, blocking wait (not poll-and-pause) until completion.

**stderr: empty (0 bytes).**

**stdout**: 71 package result lines, every one `ok` or `? ... [no test
files]`. Zero lines matching `^--- FAIL`. Relevant lines:

```
ok  	seedhammer.com/backup	3.488s
ok  	seedhammer.com/gui	437.048s
```

437.048s is inside Go's 600s per-package default (about 73%), consistent
with the plan's stated 429–507s range and P1's removal of a 119s
behavioural walk for exactly this reason.

`git status --porcelain` after the run showed no changes under
`backup/testdata/` or `gui/testdata/` — confirms gate 1.1/R-G held through
the whole-suite run, not just the scoped one.

**Pre-existing, not-mine `go vet` failures** (go1.26 `t.ArtifactDir()` on
go1.25-tagged files), confirmed via `git stash` (present identically before
any of my changes):

- `gui/freetext_sizeproof_golden_test.go:111`
- `gui/op/draw_test.go:176`

— both as documented in the dispatch brief. **Additionally found** (not
previously named in the runbook I was given, same class, also confirmed
pre-existing via the same `git stash` check): `backup/backup_test.go:393`
and `backup/freetext_test.go:240`. Not touched; recorded here since the
brief's coverage statement named only two and there are in fact four of
this class repo-wide.

---

## What I could not do / spec discrepancies found

1. **Gate 1.2a's cited budget (25) does not match the layout-based
   measurement (28).** See above — a genuine finding, predicted by the
   spike's own stated method caveat, changes no outcome, fixed by using the
   measured value with the discrepancy documented in-place.
2. **One of the three "must be updated" source-string tests
   (`bundle_abort_prose_test.go:258`) did not need an edit.** Confirmed
   empirically; a minor inaccuracy in the spec's citation, not a defect.

Nothing else in spec §1 was ambiguous or contradicted by what I found in
the fork's current source. The `me` CLI was not touched, and no
`md1`/`mk1`/`ms1` encode/decode path was touched — this is plate layout
and text only, confirmed by `git diff --stat` (only `backup/backup.go` and
`gui/*.go`/`gui/*_test.go` files changed).

---

## Commits

```
a855a6e S6b P2 (1.2/1.3): condition the marking one frame above validateMdmk
a0c4bd6 S6b P2 (1.1/1.2a/1.2b): backup.Text gains optional Title/Footer rows
```

Both carry their gate output in the commit message per this repo's
build-gate convention. Working tree is clean on `s6b-pre-flash`.
