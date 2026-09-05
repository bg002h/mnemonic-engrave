# R0 round 1 — fold-verification report for `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md`

**Scope.** Independent verification of the round-0 fold, commit `f60c2df` over `02abee6`
(`git diff 02abee6..f60c2df -- design/IMPLEMENTATION_PLAN_hashlock_H2_device.md`, 1341
insertions / 180 deletions, read in full). Contract: the deduplicated CONFIRMED (15) +
PARTIAL (1) list in `design/agent-reports/hashlock-H2-plan-R0-r0-refute.md` §3 (16 distinct
defects), plus its three severity disputes and the fold author's three declines
(`design/agent-reports/hashlock-H2-plan-R0-r0-fold-report.md`).

**Method.** Read-only on `mnemonic-engrave`, `seedhammer` (main `c4a64fc`, confirmed via
`git rev-parse HEAD`, clean porcelain), and `/scratch/code/shibboleth/.tmp/h2-gate` (never
written). Own scratch: `/scratch/code/shibboleth/.tmp/h2-r1`, a fresh `cp -a` of `h2-gate`
made for this review; Go `/scratch/code/shibboleth/.toolchain/go/bin/go` 1.26.7. Every
mutation below was applied in `h2-r1`, run, and reverted; each restoration was diff-checked
byte-identical to `h2-gate` before moving to the next. Nothing committed; no sub-agents; no
`.jsonl` read.

---

## 1. The two Criticals — RED executed independently, then GREEN

### Critical 1 — hardened derivation stalls under the screensaver (adversarial C-1)

Removed only `ctx.WakeupAt(time.Now())` from `hashlockDeriveFlow`'s frame closure (kept
`KeepAwake`), ran `TestHashlockDeriveKeepsAwakeUnderTheScreensaver`:

```
--- FAIL: TestHashlockDeriveKeepsAwakeUnderTheScreensaver (0.05s)
    composer_hashlock_test.go:835: the derivation took 9h57m1s of device time; at a 1s
    tick floor and 200 frames it should take about 3m20s. A frame that omits
    ctx.WakeupAt(time.Now()) waits out Run's idle deadline (3 min) instead of the next
    500-iteration slice
```

Restored, then removed only `ctx.KeepAwake()` (kept `WakeupAt`):

```
--- FAIL: TestHashlockDeriveKeepsAwakeUnderTheScreensaver (0.13s)
    composer_hashlock_test.go:825: Run exceeded 100000 ticks without terminating -- flow
    is probably parked (screensaver?). 180 frames drawn, last = "89%About21secondsleft.Deriving"
```

Both outputs match the fold report's §1.1 quotes byte-for-byte. Restored the file (`diff`
against `h2-gate` = identical), re-ran: `PASS (0.05s)`.

### Critical 2 — decoder off-by-one / vacuous digest test (adversarial C-2 = fidelity I-6 = tests C-1)

Baseline: `TestDecodeMS1PreimageIsShapeExact` PASS. Applied `copy(preimage[:], d[:32])` in
place of `d[1:]`:

```
--- FAIL: TestDecodeMS1PreimageIsShapeExact (0.00s)
    mspayload_test.go:185: preimage = 03ababab...abab, want the corpus's preimage_hex
    abababab...abab
```

Matches the fold report's quote exactly. Restored (`diff` = identical), re-ran: `PASS`.
Independently recomputed the corpus's `digest` in Python (`sha256(preimage_hex)` =
`9a2db2e2…885`, matching `hashlock-v0.8.json`'s `kind[0].digest` and the fold's quoted
`TestKindRowPreimageDigest` RED output exactly) — the corpus, not just the code, is
internally consistent.

Both Criticals: RED reproduces, restoration verified byte-identical to `h2-gate`, GREEN
confirmed. **FIXED, as claimed.**

## 2. The three declined remedies — measurements re-executed, not accepted on the fold's word

**tests I-6, "delete the redundant `TrimSpace`" — DECLINED.** Removed
`strings.TrimSpace(s)` from `IsMS1Shaped` and ran `TestIsMS1ShapedTrimsWhatTheStripLoopCannot`:

```
--- FAIL: TestIsMS1ShapedTrimsWhatTheStripLoopCannot
    hashlock_test.go:320: "\v" + the plate is not ms1-shaped ...
    (same for \f, U+0085, U+00A0, U+2003, leading and trailing -- 10 lines)
```

Matches the fold's quoted count and characters exactly. Confirmed the strip loop in
`IsMS1Shaped` skips only `' ' '\t' '\n' '\r' '-' ','`, a strict subset of what `TrimSpace`
removes. Confirmed the Rust-primary citation directly:
`mnemonic-secret/crates/ms-cli/src/argv_guard.rs:148-149` (repo at `504ff46a`) reads exactly
`pub(crate) fn looks_like_ms1(raw: &str) -> bool { is_ms1_shaped(&raw.trim().to_ascii_lowercase()) }`.
The decline's premise is TRUE, executed. Restored, re-ran: `PASS`.

**journey I-2, "re-run §4.5's drop order; the line goes back if it now fits" — DECLINED.**
Built my own probe test (not committed, deleted after use) against the CURRENT post-fold
tree measuring `modalHeadroom` under both `errorScreenBody` and `confirmWarningBody` for
all three `TestConfirmScreensThisBlockTouchesAreDrawnInFull` bodies plus the reconciliation
screen, the §8h form and the ms1-plate refusal:

```
hardened warning wrapped: errorScreenBody drawn=189 headroom=302 | confirmWarningBody drawn=189 headroom=302 | delta=0
sha256 warning wrapped: drawn=226 headroom=302 | drawn=226 headroom=302 | delta=0
confirm modal longest variant: drawn=337 headroom=107 | drawn=337 headroom=107 | delta=0
reconciliation screen: drawn=94 headroom=455 | drawn=94 headroom=455 | delta=0
HASH ON EVERY PATH phrase-route form: drawn=160 headroom=378 | drawn=160 headroom=378 | delta=0
ms1-plate refusal: drawn=91 headroom=476 | drawn=91 headroom=476 | delta=0
```

Zero delta on all six, and the numbers match the fold's own Task 4 Step 1 table exactly
(337/107, 189/302, 226/302, 91/476, 94/455, 160/378) — this is an independent
re-measurement against the tree as it stands NOW, not a repeat of the refute pass's
pre-fold numbers. The decline is correct: re-measuring on the other renderer reproduces the
identical numbers, so the drop order would not change.

**Per-path hash provenance — DEFERRED to H3.** Not a numeric measurement; verified the
reasoning by reading source. `composerHashByPhraseSync` is called only from
`composerHashEdit`'s `noneRow` arm (`gui/composer_hash.go:237`, gated tree), matching the
stated remedy exactly. `composerCopyHashEveryPathPhrase()`'s actual text — read directly —
is *"Back up the phrase and its method, or the preimage plate, separately"*, confirming the
stated safe-direction claim (names both artifacts, so a stale-true flag over-instructs
rather than under-instructs). Reasoning holds.

## 3. Full diff read; citations re-grepped against the fork

Read the entire 1943-line diff. Every one of the 16 items' code changes, and the two H3
spec-departure record items, matches the fold report's own description of it (cross-checked
against `h2-gate` source directly, not against the report's prose). Confirmed by direct
`sed`/sourcing against `seedhammer` at `c4a64fc`:

| citation | verified |
| --- | --- |
| `gui/composer_state.go:239` (`composerEveryPathHashed`) | exact |
| `gui/gui.go:595-600` (`warningBodyClip`) | exact |
| `gui/gui.go:3584` (`idleTimeout`) | exact |
| `gui/gui.go:110,119` (`WakeupAt`/`KeepAwake`) | exact |
| `gui/unlock_kdf.go:334-335` (F-93 wakeup) | exact |
| `gui/run_flow.go:401-406` (screensaver `continue`) | exact (span) |
| `gui/run_harness_test.go:58,183,220` | exact |
| `gui/run_flow_test.go:671` | exact |
| `gui/modal_fits_test.go:108` (`modalRenderer`) | exact |
| `gui/composer_shape.go:269-272` (creation-time delete) | exact |
| `md/compose.go:299` (`ValidatePathList` start) | exact (function runs to 340, cited range 299-334 undershoots the close but the claim — no clause compares two paths' `Hash` — holds over the whole function) |
| `ms-cli/src/argv_guard.rs:148-149` | exact |

**Two Nit-level citation imprecisions found, neither Important:**
1. `run_flow.go:350-351` is cited for a two-line quote (`` effectiveInput(evts, &a.pressed) ||
   (ctx.keepAwake && !armed) ``) that actually spans source lines 349-350, one line short.
2. The plan's own "Build gate folded here" section claims its embedded
   `h2-plan-blocks-vs-tree.sh` output was "re-captured after the last edit and diffed
   against a fresh run, so the line numbers... are current, not stale." A live re-run
   (below) reproduces all 26 PASS lines and both non-`bash`/`go` unheadered-block lines
   exactly, but the LAST unheadered-block line differs: the plan's embedded text says
   `2929`, a fresh run says `2930`. Informational list only (not part of the PASS/FAIL
   verdict); the claim of currentness is technically false for this one line.

No contradiction found elsewhere: grepped for stale phrasing left behind by the fold
(`42 → 51` unqualified, `254 drawn`/`headroom 262`, `1213` unqualified, `nine more bodies`) —
every instance found is inside the historical "twelve fixes" narrative and is explicitly
annotated superseded (`**Now eleven rows and 42 → 53**`, `**SUPERSEDED IN PART...**`,
`(**1220 after the R0 round 0 fold**...)`). No silent propagation failure.

## 4. Whole-suite and gate numbers — re-executed, not read off the report

```
$ go test -count=1 ./hashlock/... ./codex32/... ./seal/... ./sysw/...
ok  	seedhammer.com/hashlock	0.231s
ok  	seedhammer.com/codex32	0.002s
ok  	seedhammer.com/seal	11.878s
ok  	seedhammer.com/sysw	0.041s

$ scripts/gui-shard-test.sh ./gui/ 24
    1220 top-level tests
    partition verified exhaustive: 1220 == 1220
RESULT: ok -- all 1220 tests ran across 24 shards      (wall 26s)

$ go vet ./hashlock/... ./codex32/... ./seal/... ./sysw/... ./gui/
gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later
gui/transaction_golden_test.go:104:13: testing.ArtifactDir requires go1.26 or later

$ gofmt -l hashlock/ codex32/ seal/ sysw/ gui/*.go
gui/transaction.go
gui/transaction_golden_test.go
gui/transaction_txrecord_test.go

$ scripts/h2-plan-blocks-vs-tree.sh   (default plan + h2-gate)
26 blocks checked, 0 FAIL
```

All match the fold's claims exactly (1220/1220 exhaustive, both pre-existing vet
complaints, all three pre-existing gofmt files, 26/0 FAIL). `hashlock` package test count
independently counted via `go test -list`: 9 (was 6). `composerCopy*` function count in
`gui/composer_copy.go`: 53, matching the `declared != 53` literal;
`TestComposerCopyTableCoversEveryBody` PASSES.

## 5. Severity disputes

All three resolutions read as stated and are internally consistent with cited project
policy (secret-handling exemption does not apply here; "false PASS" stays blocking):
(a) decoder/digest gap → Critical, supported by two independently-executed false-PASS
reproductions (fold's and this review's); (b) `hashByPhrase` never cleared → Minor,
supported by the direct copy-text read above; (c) `Deriving` zero-state lead → Important,
a normative-copy argument that is consistent with project severity language.

---

## 6. Table

| # | finding | fold change | verdict |
| --- | --- | --- | --- |
| 1 | derivation stalls under screensaver (C) | wakeup pair + `TestHashlockDeriveKeepsAwakeUnderTheScreensaver` | **FIXED** (RED/GREEN executed independently) |
| 2 | decoder off-by-one / vacuous digest test (C) | corpus `Digest` field + corpus-driven decoder test | **FIXED** (RED/GREEN executed independently) |
| 3 | reconciliation line unreachable, mixed policy | own post-HOLD `showError`; §8h reverted to §4.7 text | **FIXED** |
| 4 | `hashByPhrase` never cleared/verified | `composerHashByPhraseSync` in `noneRow` arm + assignment test | **FIXED** |
| 5 | `Type 64 hex` Back untested, false claim | `TestHashlockHexRowBackKeepsThePath`; false sentence corrected | **FIXED** |
| 6 | `Deriving` zero-state lead unreachable | pure `hashlockDerivingLead` + hoisted zero-state frame | **FIXED** |
| 7 | C-4 regression mis-attributed | comment corrected + `TestComposerHashEditDispatchesByRowLabel` | **FIXED** |
| 8 | fit-gate renderer mismatch (PARTIAL) | new `TestConfirmScreensThisBlockTouchesAreDrawnInFull` via `confirmWarningBody` | **FIXED** |
| 9 | relation line no-match untested | 3-case parameterised `TestHashlockConfirmRelationLine` | **FIXED** |
| 10 | two paths, two phrases, no cross-check | `hashlockOtherPathLine` + new relation line + test | **FIXED** |
| 11 | §8i modal confusing ahead of phrase route | phrase lead copy answers the modal | **FIXED** |
| 12 | Task 3 stub under-specified | stub shown in full, compiled | **FIXED** |
| 13 | Task 4 RED claim wrong | Expected line corrected to runtime failure | **FIXED** |
| 14 | `DeriveHardened` abandon contract untested | `TestDeriveHardenedAbandonsWhenProgressSaysStop` | **FIXED** |
| 15 | `minMS1Len` boundary untested | `TestIsMS1ShapedMinLengthBoundary`, literal 47/48 | **FIXED** |
| 16 | `TrimSpace` untested-as-such | `TestIsMS1ShapedTrimsWhatTheStripLoopCannot`; remedy declined | **FIXED**, decline **DECLINED-OK** (verified) |
| decline | journey I-2 re-measure suggestion | declined | **DECLINED-OK** (verified independently) |
| decline | per-path provenance | deferred to H3 | **DECLINED-OK** (reasoning verified) |

**Closing counts:** 16/16 confirmed/partial findings fixed as claimed; mapping paragraphs
honest; 3/3 declines verified correct on independent measurement/inspection; 0 confirmed
findings left unfixed; 0 new Important defects or contradictions found; 2 new Nit-level
citation imprecisions found (one off-by-one line citation, one stale line number in an
informational "not covered" list) — neither changes a conclusion or leaves a defect
unfixed.

## GREEN

Closes R0 round 1 for this plan.
