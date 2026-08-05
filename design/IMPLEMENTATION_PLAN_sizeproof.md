# IMPLEMENTATION PLAN — `SIZEPROOF!`

Status: **P0, awaiting R0.** Author: controller session, 2026-08-05.
Spec: `design/SPEC_sizeproof.md` @ R5, **R0 GREEN (0C/0I) at round 4**.

This is a **sequencing** document. It adds no design: every "what" lives in the
spec, and a disagreement between the two is a defect in this file. What it fixes
is the ORDER, the phase boundaries, and which of the spec's 20 test items gates
which phase — so that no phase can close on a suite that could not have caught
the thing that phase changed.

---

## 0. Standing constraints

- **Repo:** the fork, `/scratch/code/shibboleth/seedhammer`, branched off `main`
  @ `6d57681` (which now carries the `PASSPROOF!` rename).
- **Worktree:** one, dedicated. `seedhammer-wt-sizeproof`, branch `sizeproof`.
  The primary checkout stays free — a second agent compiling `gui` against a
  half-written `backup` produces `[setup failed]` that reads exactly like a real
  failure.
- **One implementer.** Not parallel re-implementations. The controller folds
  small post-review fixes inline in the worktree rather than spawning agents.
- **TDD per phase:** the phase's gating tests are written and RED before its
  implementation, and the phase closes when they are green **and the whole suite
  is green** — not just the slice the phase touched.
- **`-update` is never run.** A moved golden is a finding, not a baseline
  refresh. This is the load-bearing property of the entire change: the spec's
  claim is that the general path reproduces the special case exactly, and the
  goldens are the only thing that can falsify it.
- **Review:** opus + sonnet two-lane on the whole diff after P5, per the
  operator's 2026-08-05 choice. **fable stays unspent** — nothing here is
  irreversible, funds-touching, or normative-codec; the OTP write is long done.

## 1. Phases

Ordered so the goldens stay green at every boundary. Each phase is a commit.

### P1 — the data carriers. No behaviour change.

`Block.SizeMM` (§2.2); `Fitted` gains `Mixed`, `Sizes`, `TitleSizeMM`,
`FooterSizeMM`, `qrAt` (§2.3); **both** legacy constructors populate them —
`fitBlocksAt` and `EngraveFreeText` (§2.3). Guards added: `len(Sizes) !=
len(Lines)`, the size/string invariant, and §2.1.1's two panics.

Nothing reads the new fields yet. `EngraveFitted` still walks rows.

**Gate:** §7.1 (every golden, byte for byte) · §7.15 (the `Sizes` guard panics).

*Why first:* it is pure addition, so a moved golden here means the constructors
were filled wrong, with nothing else in the diff to hide behind.

### P2 — the QR placement. Extraction, not redesign.

`qrPlacement` + `qrPlaceAt` (§2.1); `textLayout` takes a `*qrPlacement` and
`lineLayout` carries `qrTop`/`qrBottom`; `EngraveFitted` reads `f.qrAt` and
`EngraveText` its own local. Every producer keeps the anchor it has today
(§2.1's producer table) — **including `AdmissibleBlocks`, whose `anchorY` is
`outerMargin` while §2.4 will give it a different `start`.**

Row indexing is UNCHANGED in this phase. The band is expressed in absolute y and
is measured to agree with the row-index predicate on every row at every rung, so
this phase should be behaviour-identical by construction.

**Gate:** §7.1 · §7.7(a) the band equivalence at all six rungs · §7.7(c) both
engravers read the stored placement · §7.7(d) the `Bottom` check is an ERROR
from the fit, not a panic.

### P3 — the y budget and the running y.

`wrapBlocks` takes `sizes`, `qrp`, and device-unit `start`/`limit` (§2.4), with
**both existing caller translations** — `AdmissibleBlocks` →
`params.I(outerMargin) + params.F(size)`, `rowFaces` → `params.I(outerMargin)`.
`limit` is read off the footer. `EngraveFitted` accumulates y and reads `at(0)`
per row (§2.5). `faceLayouts` is deleted (§2.6).

**Gate:** §7.1 · §7.2 rows are cut at the sizes claimed · §7.8 footer and last
body row disjoint on a MIXED plate with a 3.8 mm footer · §7.11 the unbounded
callers still report untruncated counts · **§7.20 `AdmissibleBlocks`' verdict
does not move, with `useQR` true and false, plus the `MaxCharsAtBlocks` pin.**

*This is the phase that broke twice in review.* Both round-2's I2 and round-3's
I2 live here, and both were invisible to the compiler and to every golden —
`int` to `int`, right shape, wrong meaning. §7.20 is the only thing that fails.

### P4 — `FitSized`, and mixed plates become real.

`FitSized` (§2.7) with its validation, its `Mixed`/`SizeMM` computation, and nil
`QR`/`qrAt`.

**Gate:** §7.5 a block inside the top band is inset, one below it is not · §7.6
the same face at two sizes on one plate · §7.9 `EngraveFitted` on a `Mixed`
plate does not panic · §7.14 the size/string invariant refuses, and the
no-footer path gives `limit == plateHeight - margin`.

### P5 — the GUI surface, the triggers, and the previews.

Every row of §3's table; `ftPlan.Blocks`' part-count predicate and its corrected
doc comment (§3.1); the `SIZEPROOF!FRONT`/`BACK` entries with `TextQR: ""` and
their `Side` (§4); `ftProofOutcomeFor`'s per-proof footer; `fittedPreviewAt`
through `ftFitAt`; `cmd/plateview`'s two entries.

**Gate:** §7.10 end-to-end through `freetextPlateHook` · **§7.13 all four edit
shapes plus the synthetic `[2,1,1]` plan** · §7.16 both proofs need the whole
plate · §7.17 the confirm screen names the rungs, never `0.0mm`, and fits by
measured rectangles · §7.18 the preview's per-row sizes equal the device's.

### P6 — the composition pins and the mutation pass.

§7.3's two per-block tables (face, size, row count, per-row budgets, y-range,
total) · §7.4 the character set per `(rung, face)` · §7.12 the titles.
Then §7.19: **mutate every test added in P1-P6 and confirm each notices.**

*Why last:* §7.3 is what makes the back's 2.400 mm of spare safe, and it can
only be written against the real composition, which does not exist until P5.

## 2. What closes the phase gate

A phase is green when its listed items pass **and** `go build ./...` and
`go test ./...` are green — the whole surface, not the slice. A red suite is
itself a blocking finding.

Between phases, reconcile follow-ups: anything filed against a phase is burned
down in or before that phase, not batched to the end.

## 3. After P6

1. Two-lane adversarial review over the whole diff (opus + sonnet), R0 =
   plan correctness. Non-deferrable — it is what catches implementation-introduced
   regressions TDD misses. Persist verbatim to `design/agent-reports/`.
2. Merge to fork `main`.
3. Cut a release carrying **both** this and the un-flashed BIP-39 password work
   already on `main`, so one flash gets the operator everything
   (operator's choice, 2026-08-05).
4. The operator flashes and engraves. Reading the plates is its own cycle.

## 4. Out of scope here

Everything in the spec's §8, plus: this plan does not schedule the glyph test
coverage sweep, the `f` top-bar steel verdict, or the stale worktree cleanup.
Those are open items in `CONTINUITY_2026-08-05.md` §6 with no owning phase in
this change.
