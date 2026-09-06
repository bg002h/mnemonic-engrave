# REPORT — H6 spec R0 round 1: fold verification (sonnet, independent)

**Artifact:** `design/SPEC_hashlock_H6_preimage_plates.md`, fold commit `4881474f`
over `a0f832d0`. **Inputs read in full:** `hashlock-H6-spec-R0-r0-fidelity.md`
(4C/7I/7M/3N), `-journey.md` (1C/7I/7M/2N), `-tests.md` (0C/3I/1M/1N),
`-fold-report.md`. **Ground:** fork `fb0dd04` (detached worktree
`/scratch/code/shibboleth/.tmp/h6-r1`, Go `/scratch/code/shibboleth/.toolchain/go/bin/go
1.26.7`, removed after the run), engrave master (checked out in place, HEAD
`f6723516`), mnemonic-secret `504ff46`. Read-only on the engrave repo (git status
clean throughout); execution only in the detached fork worktree; nothing
committed there either — `engrave.go` was patched for measurement and restored
from a `.orig` backup before the worktree was removed. No sub-agents. No
`.jsonl` read.

**Method.** Rather than trust the fold's own numbers, every measurable claim
below was re-derived from scratch: a scale-2 `engraveModule` arm and §7.2 items
1–3 were applied to the worktree (matching what the fold report says it did),
and independent Go test files called the fork's real functions
(`qr.Encode`, `ConstantQR`, `constantTimeQRModules`, `backup.CharsPerLine`/
`fixedCharWidth`, `assertModalBodyFits`/`errorScreenBody`) — never a
reimplementation of the fold's arithmetic.

---

## 1. Per-finding table

| finding | fold change | verdict |
| --- | --- | --- |
| fidelity C-1 = tests I-3 (raise omits `constantTimeQRModules`) | §7.2 item 4: normative fuzzing arm, floor tabulated, v7 flagged under-converged; §11.3 splits budget/regression rows | **FIXED** — reproduced the pre-fix failure verbatim (`too many dims 41/45/49/53 QR modules...`); re-fuzzed independently, see §2 below |
| fidelity C-2 (scale-2 panics `engraveModule`) | New §7.3 `case 2` arm (asymmetric, 5 commands/module) + §7.4 (raise and arm are one deliverable) | **FIXED** — independently implemented and measured; extents match the spec's three worked examples to the unit (§2) |
| fidelity C-3 (`--pack-preimage` has no stated classification/admission split) | §3.2: admission-only, four normative points; `decide_sealing` unchanged; declines the lens's own remedy with a named reason | **FIXED** — every cited line (`main.rs:2353`, `:1696`, `:1478`; `sysw/mod.rs:246`, `:252`, `:463-470`) read and matches verbatim in the actual engrave-repo source |
| fidelity C-4 = journey I-1 (review screen has no input budget) | §5.3 splits into a `composerPickScreen` PICK step + read-only census; `show` toggle dropped, not relocated | **FIXED** — the two-step structure, back contract, and the four measured row widths (128/138/180/196 px) are stated exactly as the fold report claims; `show`'s latch mechanism is correctly described |
| journey C-1 (no ms1 preimage encoder for device-derived material) | New §3.5 `codex32.EncodeMS1Preimage`, Go-only (Rust encoder already exists, per the controller's correction); fixed id `hash`, output-side assertion | **FIXED** — §3.5's code comment and measured corpus round-trip are consistent with the journey report's own counterexample; new §12 item 3a closes the acceptance gap the journey report named (nothing else in §12 cut a device-derived string-form plate) |
| fidelity I-1 (§8.4's clause dead code) | §8.4 rewritten: two composer-screen arms, `bundleAbortWarningText` unchanged | **FIXED** |
| fidelity I-2 + M-2 (band mutation can't fail; width justification false) | §6.3 re-justified on line count; §11.4 mutation replaced with a placement one | **FIXED** — locator row char counts (6/24/27/29/30 vs 32-char cap) read as stated |
| fidelity I-3 (§10.2 guards an unreachable screen) | §10.2 rewritten: no guard, no test, reason recorded | **FIXED** |
| fidelity I-4 (§10.1 asserts an undecided plate cut) | Both arms moved to present tense of what is HELD | **FIXED** — both bodies re-measured, see §2 (185/360, 186/360 exact) |
| fidelity I-5 (`preimage_plate` sentence false; untestable "any other id" row) | §4.3 corrected, three-id partition table added | **FIXED** — table's `entr` row (`TagKindMismatch`, shipped, unchanged) vs `hash`/other-id rows is internally consistent and matches `decode.rs`'s rule as described in the R0 reports |
| fidelity I-6 = tests I-2 (non-ASCII device copy; two unreproducible numbers) | §8 preamble states the blanking mechanism + host/device split; every device body re-measured ASCII | **FIXED** — grepped every blockquote line in the spec for non-ASCII: all 8 hits are in HOST-labelled §8.1.1/§8.2.x bodies (explicitly exempt); zero hits in any DEVICE-labelled body (§8.3, §8.4a/b, §8.5, §8.8, §9, §10.1); re-measured all 7 changed device/HOST-measured bodies, all exact (§2) |
| fidelity I-7 = journey I-2 (flow can't compute a phrase digest) | §5.2 gains the four-part derive rule (list without deriving, derive once on pick behind the countdown, flow-local, print from it) | **FIXED** |
| journey I-3 (orphan check misses `phrase:` records) | §3.3 item 3 + §8.2.3 extended to both carriers, two hand-build errors named | **FIXED** |
| journey I-4 (three refusals, none naming the collision) | §8.1.1 conditional on id `hash`; §8.1.2 gains the 1-in-256 sentence | **FIXED** — re-measured both variants, exact (§2) |
| journey I-5 (marking reaches mk1 too, untestable) | §10.3 states md1 AND mk1 with F-132 as reason; §11.5 names the mk1 assertion as required | **FIXED** |
| journey I-6 (Password refusal invisible) | New §8.8 modal + §11.5 row + §12 item 7 | **FIXED** — re-measured, exact (§2) |
| journey I-7 (new cut order opens a second, uncovered abort window) | §5.4 tables both windows; §8.4b is the new arm | **FIXED** — re-measured, exact (§2) |
| tests I-1 = fidelity M-3 (5.0 mm advance wrong) | §6.5 corrected to 3.333 mm | **FIXED** — re-derived from `fixedCharWidth` independently: 21,333 units = 3.3333 mm, exact (§2) |
| fidelity N-2 (two live `IsPreimage` call sites omitted) | §14 now lists SIX call sites including `unlock_session.go:197` and `seal/record.go:260` | **FIXED** — both cited lines exist and read as claimed |
| journey's closing note (4th falsified shipped record, `hashlock.rs:352`) | Folded into §0 as item 4 | **FIXED** — `crates/ms-cli/src/cmd/hashlock.rs:352` read verbatim in the mnemonic-secret repo, matches exactly |

All 5 Criticals and all 17 Importants from the three round-0 reports are accounted for above: FIXED in every case (no DECLINED-OK or NOT-FIXED verdict was needed for a C/I). The two declines recorded in §16 (fidelity C-3's remedy, journey C-1's Rust-first framing) are declines of a *suggested remedy*, not of the underlying finding — both underlying defects are folded, as the table above shows, and both declines give a true, checkable reason (re-verified against the same cited lines).

---

## 2. Measurements re-derived from scratch (not the fold's arithmetic)

**QR module-count / version thresholds (§7.1).** `qr.Encode` at `qr.L` on
byte-mode strings of length 194/135/122/107/106/192/155/154/134/36 bytes
produced dim 53/45/41/41/37/49/49/45/41/29 — every one exactly the version
§7.1's threshold table claims.

**Alignment-ring centres (§7.2), independently scanned from the bitmap** (not
from the table this probe copied into `bitmapForQRStatic`): a from-scratch 5×5
ring detector over `qr.Code.Black(x,y)` found

```
v6 (dim 41): [34 34]
v7 (dim 45): [22 6] [6 22] [22 22] [38 22] [22 38] [38 38]
v8 (dim 49): [24 6] [6 24] [24 24] [42 24] [24 42] [42 42]
v9 (dim 53): [26 6] [6 26] [26 26] [46 26] [26 46] [46 46]
```
— an exact match to §7.2's table, every row.

**Scale-2 arm geometry (§7.3).** Implemented `case 2` in `engraveModule`
(asymmetric square, matching the spec's stated envelope) and measured directly
against `Command`'s raw args (same-package test, no reimplementation of
`centerOf`):

```
p={0 0} cmds=5 x=[0,3840] y=[0,3840]
p={1 0} cmds=5 x=[3840,7680] y=[0,3840]
p={5 7} cmds=5 x=[19200,23040] y=[26880,30720]
adjacent cells touch exactly at 3840, no gap/overlap
```
Exact match to the spec's three worked examples and to the "5 commands per
module" / "no neighbour overlap" claims.

**Constant-time budget floor (§7.2 item 4), independently fuzzed, parallel
across cores, smaller sample than the fold's 216,000/dim:**

| dim | v | my samples | my observed max | spec's floor (216k) | at/under floor? |
| --- | --- | --- | --- | --- | --- |
| 41 | 6 | 30,000 | 795 | 813 | yes |
| 45 | 7 | 30,000 | 928 | 945 | yes |
| 49 | 8 | 60,000 (varied phrase length across the whole valid range) | **1168** | 1161 | **no — 7 over** |
| 53 | 9 | 30,000 (phrase length correctly bounded to 99–100 chars by §3.1's cap) | 1335 | 1369 | yes |

Zero `findPath` errors across every sample, at every dim, consistent with the
spec's "0 in 864,000." **The v8/dim-49 result is a real, reproducible
discrepancy against the fold's stated number** (re-run with a larger, more
varied sample, not a fluke of one seed) — but it does **not** falsify a claim
the spec makes: §7.2 item 4 and §16 explicitly and repeatedly say these four
numbers are "a FLOOR, not the answer," "demonstrably not converged," and "not a
number to paste," citing their own smaller-sample-found-less-than-larger-sample
evidence for exactly this shape of result. An independent, differently-sampled
run finding a value **above** one of the four floors is what "not converged"
predicts, not a contradiction of it. Classified as a **confirmation of the
spec's own caveat**, not a new defect — but it is worth restating loudly for
whoever runs the real fuzzing campaign: **the v8 entry needs the largest margin
of the four**, more than v7's already-flagged one.

**Device-body headroom, all 7 numbers the fold introduced or changed
(`assertModalBodyFits`/`errorScreenBody`, literal spec text, `{i}` placeholder
un-substituted exactly as the spec's own template writes it):**

| body | spec claims | measured | match |
| --- | --- | --- | --- |
| §8.1.2 ASCII (no collision sentence) | 236/320 | 236/320 | exact |
| §8.1.2 + collision sentence | 340/223 | 340/223 | exact |
| §8.4a `composerAbortNoPreimage` | 75/476 | 75/476 | exact |
| §8.4b `composerAbortPreimageCut` | 86/476 | 86/476 | exact |
| §8.8 Password-program notice | 165/397 | 165/397 | exact |
| §10.1 arm A (plain, present tense) | 185/360 | 185/360 | exact |
| §10.1 arm B (phrase, present tense) | 186/360 | 186/360 | exact |

(Note: the drawn-character count only matched once the `{i}` placeholder was
left as the literal three characters `{i}` — exactly as the spec's own §8.1.1/
§8.1.2 template text is written — rather than substituted with an example index
digit. Substituting `"0"` for `{i}` reproduces the headroom exactly but the
drawn count 2 characters low, which is arithmetic, not a spec defect.)

**§6.5 advance-at-5.0mm correction**, re-derived via `fixedCharWidth`/
`CharsPerLine`/`LinesPerPlate` with `sh2.Params()` and `constant.Font` (not the
fold's own numbers): `21,333 units = 3.3333 mm` at 5.0 mm — matches the fold's
correction exactly, and the withdrawn `3.435 mm` is confirmed wrong. All six
rungs' chars/line and lines/plate values reproduced exactly (19/23/26/31/34/39
and 13/15/17/20/23/26).

**Citation spot-check** of the 6 citations the fold report says it corrected
(`freetext_test.go:109`, `composer_engrave.go:80`, `composer_door.go:86-90`,
`composer_flow.go:381`, `backup.go:111`, plus `hashlock.rs:184`/`:352` in
mnemonic-secret): all 7 read exactly as the spec now cites them, verified by
reading the actual lines in both repos.

---

## 3. Propagation sweep (superseded numbers/phrases named in the brief)

Grepped the folded spec for every string the brief named:

| string | hits | classification |
| --- | --- | --- |
| `3.435` | 2 (both in §6.5's own "was X in the first draft, does not reproduce ... /3.435 = 23.28, not whole" correction sentence) | **historical, correctly labelled** |
| `205` (the "205 taps" W-2 measurement) | 1, in §5.3's `confirmReviewScreen` unimplementability argument, attributed to W-2 as prior evidence | **live but correctly used** — not a superseded H6 number, a citation of an unrelated prior finding used as supporting evidence; unchanged by the fold and not claimed to be an H6 measurement |
| `165/397` | 2: once in §16's Criticals table (`"the claimed 165/397 is withdrawn"`, about §8.1.2), once as §8.8's own genuine, current, re-measured value | **both correctly historical/live** — the withdrawn instance is explicit about being withdrawn; the live instance is a different body (§8.8, not §8.1.2) and is independently confirmed true in §2 above |
| `288/244` | 2, both in explicit "the claimed 288/244 is withdrawn" sentences (§8.4 body text and §16 table) | **historical, correctly labelled** |
| `v7` | 8 hits, all in §7.1/§7.2/§16's live, current, correctly-value text (45 modules, the six-ring table, the under-converged-entry flag) — **zero** hits of the superseded "v7 = 53 modules" claim outside the sentence that names and corrects it (`"The ruling names 'v7 = 53 modules'; that is v9."`) | **correctly corrected, one hit is the explicit correction itself** |
| `cannot unlock` | 1, inside the explicit "**CORRECTION, measured** ... The ruling's premise for warning 4 -- *'the device cannot unlock it...'* -- does not hold" sentence | **historical, correctly labelled as a corrected premise** |
| `by label` | 1, in §11.5's *"`Which hash?` by LABEL with 0, 1 and 2 records..."* | **not a superseded phrase at all** — this describes a real, already-shipped label-keyed dispatch mechanism (label vs. index-keyed default), referenced positively as the pattern a mutation would break; nothing to withdraw |
| `§` / `—` in device copy | see §1's I-6 row above | **zero live hits in any DEVICE body; all live hits are in HOST-labelled, explicitly-exempt bodies** |

No superseded number or phrase leaked into the spec as live, uncaveated text.

---

## 4. Hostile-implementer read (check 4)

Read §3–§8 as an implementer building from the folded text alone, watching for
a sentence two implementers would read differently. Nothing new found beyond
what round 0 already flagged as Minor/Nit (e.g. fidelity M-4's wrap rule, now
stated; N-1's three-vs-four band literals, now corrected). §3.2's admission/
classification split, §4.3's three-id table, and §5.3's two-step review are all
stated with enough mechanism (function names, line numbers, exact predicates)
that an implementer has one reading, not several. §7.2 item 4 is explicit that
the four budget numbers are NOT to be pasted without a fresh fuzzing run —
correctly pre-empting the one place an implementer might otherwise take a
plan-time shortcut.

**One observation, not a defect:** §7.2 item 4's own text singles out v7 as
"the suspect entry" needing the largest margin; my independent re-fuzz (§2
above) found v8, not v7, as the dim whose stated floor a differently-distributed
smaller run could exceed. This doesn't make either flag wrong — v7's signal
(ratio below trend) and v8's (an independent run finding above its floor) are
different kinds of evidence for the same underlying fact (none of the four are
converged) — but a plan-time fuzzing task should not read "v7 is flagged" as
license to under-margin v8.

---

## 5. New-defect findings

None. No new false number, no new false citation, no new contradiction with
`design/CONTINUITY_composer_2026-09-01.md`'s "H6 BRAINSTORM" / "Group A
rulings" entries was found (spot-checked §3.4's D1 correction, §4.3's ruling
A7, and §7's ruling A1 against the spec's own citations of those rulings in
§15; all consistent).

---

## 6. Closing counts

- **17 Importants + 5 Criticals from round 0: 22/22 FIXED**, 0 DECLINED-OK
  needed at C/I severity (the 2 declines are of suggested remedies only, with
  the underlying defect folded), 0 NOT FIXED.
- **New Important-or-above findings this round: 0.**
- **One reproducible, reportable observation** (v8's constant-time floor
  exceeded by an independent, larger/varied-sample re-fuzz) that confirms —
  rather than contradicts — the spec's own explicit "not converged, not a
  number to paste" caveat; logged in §2/§4 for whoever runs the real fuzzing
  campaign, not gating.
- Propagation sweep: 0 superseded numbers/phrases live as current claims.
- Citation spot-check: 7/7 corrected citations verified true against real
  source.

## GREEN

R0 for `design/SPEC_hashlock_H6_preimage_plates.md` closes at fold `4881474f`.
