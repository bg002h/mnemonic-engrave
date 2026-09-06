# REPORT — H6 spec, R0 round 0 FOLD

**Artifact folded:** `design/SPEC_hashlock_H6_preimage_plates.md` (engrave master
`a0f832d0`, working tree). **STATUS** set to `DRAFT -- R0 round 0 folded; r1 fold
verification pending`; the fold's own records are in the spec's new **§16**.

**Inputs:** `hashlock-H6-spec-R0-r0-fidelity.md` (4C/7I/7M/3N),
`-journey.md` (1C/7I/7M/2N), `-tests.md` (0C/3I/1M/1N), all read in full; the
brief's C-1 ruling **as corrected mid-task by the controller** (the Rust encoder
already exists — no new Rust-first deliverable; the Go port is the gap).

**Method.** Every measurable claim was re-run in a detached fork worktree at
`fb0dd04` (`/scratch/code/shibboleth/.tmp/h6-fold`, Go 1.26.7), calling the
fork's own functions. §7.2 items 1–3 and a `case 2` arm were applied **to the
scratch worktree only** so the raise could actually be executed; nothing was
committed, and citations were re-checked against **pristine `fb0dd04`**
(`git show`) because the patch shifted line numbers. Both worktrees removed.
Only the spec was edited. No sub-agents. No `.jsonl` read.

---

## 1. Criticals — change and measurement

### fidelity C-1 = tests I-3 — the raise omits `constantTimeQRModules`
**Change:** §7.2 gains a normative item 4 (an arm per version, the fuzzing
protocol named from the v5 entry's own provenance, the fold's floor tabulated,
v7 flagged as under-converged) and a re-written item 5 that separates the budget
proof from the regression guard. §11.3 splits the single row into two and says
which is which.

**Measured.** Reproduced the failure first: with items 1–3 applied and nothing
else, `constantTimeQRModules` returns **0** at 41/45/49/53 and every raised
version errors `too many dims N QR modules for constant time engraving`. Then
fuzzed §8.6-shaped payloads, sampled per dim across the phrase lengths that reach
each version, 24 workers:

| dim | v | samples | observed max | max/dim² | `findPath` errors |
| --- | --- | --- | --- | --- | --- |
| 41 | 6 | 216,000 | **813** | 0.4836 | 0 |
| 45 | 7 | 216,000 | **945** | 0.4667 | 0 |
| 49 | 8 | 216,000 | **1161** | 0.4835 | 0 |
| 53 | 9 | 216,000 | **1369** | 0.4874 | 0 |

against shipped ratios 0.3764/0.4176/0.4590/0.4977/0.4850 at 21/25/29/33/37.
**These are a FLOOR, not the answer, and the spec says so**: the fidelity lens's
independent 4,000-payload sample saw 785 and 1281 where 216,000 saw 813 and 1369,
so the ceiling was still climbing — a plan-time entry below the floor is proof of
an under-converged run, which is a checkable gate. v7's ratio sits below its
neighbours (trend-implied ≈982 vs observed 945), which is exactly the signal the
shipped v5 comment used to justify a buffer of 20 rather than 5.

Also cited: the fork **already says this is the shape of the work** —
`engrave/engrave_test.go:696-707` records that the bound "was derived by fuzzing,
not by this test" and that a raise means "constantTimeQRModules needs a v6 entry
derived by fuzzing. Do NOT fall back to the non-constant-time engrave.QR".

### fidelity C-2 — QR scale 2 panics `ConstantQRCmd.Engrave`
**Change:** **scale 2 is KEPT and the engraver learns it** — new §7.3 (normative
`case 2` arm with its geometric contract) and §7.4 (the raise and the arm are one
deliverable). §6.5 consequence 1 states the dependency and why re-fitting is
unavailable.

**Why not re-fit:** measured, scale 3 at 53 modules is 47.70 mm; at 3.0 mm the
worst case totals 79.70 mm against a 65 mm budget. Fitting it needs the text
block down to 5 rows, and the 100-char phrase (3 rows) plus the method line
(2 rows) is already 5 with **zero** locator rows — and the locator is ruling A5.
There is no rung at which the phrase form carries a scale-3 QR.

**Measured (implemented and run).** `centerOf` is scale-3-specific — it puts the
scale-2 centre half a stroke off the cell centre — so the arm must be asymmetric
like `case 4`'s, path `[-sw, 0]` on both axes:

```
scale=2 p={0 0} cmds=5 ink x=[0,3840] want [0,3840] OK | y=[0,3840] OK
scale=2 p={1 0} cmds=5 ink x=[3840,7680] want [3840,7680] OK
scale=2 p={5 7} cmds=5 ink x=[19200,23040] OK | y=[26880,30720] OK
scale=3 p={5 7} cmds=5 ink x=[28800,34560] OK (control)
```

3,840 units = **0.6 mm**, the cell exactly, no neighbour overlap, and **5
commands per module — the same constant as scale 3**, which is what preserves the
constant-time argument. §11.3 asserts both the count and the extent.

### fidelity C-3 — `--pack-preimage` has no stated place in classification
**Change:** §3.2 makes it **ADMISSION, never classification**, in four normative
points: classification unconditional on both sides; `Admission.pack_preimage`
plus a second `admit_check` rule (a new `SyswError` arm, not a `Class::Unknown`,
because the record is understood and simply not asked for); **`decide_sealing`
byte-unchanged**; one corpus truth per row.

**Read, not assumed:** `decide_sealing` (`main.rs:2353-2410`) filters with
`sysw::classify`; it is called at `:1696` **without** the `admission` value that
exists at `:1478`; `classify`/`classify_with` at `sysw/mod.rs:246`/`:252`;
`admit_check` at `:463-470`; the lockstep test at
`sysw/composer_records_test.go:64-75` compares corpus rows (generated with
strict `classify`) against the device's unconditional `Classify`.

**I declined the lens's own remedy** (thread `admission` into `classify_with` and
`decide_sealing`) and said why in the spec: it adds a second call site to keep in
step — the exact failure the shipped `--expect` comment records at
`main.rs:1488-1494` ("a false refusal carrying a false message, on the funds
path") — and forces two answers per corpus row. Gating admission needs neither.

### fidelity C-4 = journey I-1 — the review has no input budget
**Change:** §5.3 becomes two steps — **(A)** a `composerPickScreen` per held
digest, **(B)** the read-only `confirmReviewScreen` census reporting the choices.
Back contract stated. The `show` toggle is **dropped**, not relocated.

**Measured.** `confirmReviewScreen` (`gui/multisig_build.go:1895`) binds all three
buttons and its rows are `widget.Labelw` ops, not `Clickable`s.
`composerPickScreen` (`gui/composer_paged.go:278`) has row touch targets, a
cursor, Button3-takes/Button1-declines, cap 24 (`:243`). Pick rows measured in the
411 px band: 128 / 138 / 180 / 196 px — **all one line**. The `show` affordance is
`{label: "show", action: ppReveal}` (`gui/passphrase_keyboard.go:141`), a
**keyboard-grid key** that **latches** (`k.revealed = !k.revealed`, `:221`) — it
is neither placeable nor "held", and item 7's own argument (the plate preview is
the screen that shows the phrase) settles it.

### journey C-1 — no ms1 preimage encoder → new §3.5
**Change per the corrected ruling:** §3.1 cites the existing Rust encoder and
§3.5 specifies the **Go** wrapper `codex32.EncodeMS1Preimage(x [32]byte)` over
`NewSeed`, id fixed to `hash`, with three tests.

**Verified, not taken on trust:** `ms_codec::encode` (`encode.rs:16`), its
`tag != payload.kind().single_tag()` refusal (`:24-31`),
`PayloadKind::Preimage → Tag::HASH` (`payload.rs:23-28`), the 0x03 prefix
(`envelope.rs:248`, `:261`), and its use at `hashlock.rs:298`. So the Rust
primary **structurally cannot** emit a preimage under `entr`.

**Measured on the Go side:**
```
encoded = ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c
corpus  = (identical)   len=75  id="hash"  IsPreimage=true  round-trip=true
NewSeed("ms",0,"entr",'s',[0x03||X])  -> NOT REFUSED
  = ms10entrsqw46...  IsPreimage=true  IsPreimagePlate=false
```
**The third line is the finding:** the Go primitive has no equivalent of the Rust
refusal, so a mistagged plate is one parameter away and §4.3 would refuse it on
the way back in — an operator's only backup of a spend secret, on steel, that no
tool reads. Hence the fixed id and the assert-on-output test.

---

## 2. Importants — all 17 folded

Fidelity **I-1** (§8.4 rewritten: both arms are composer screens,
`bundleAbortWarningText` unchanged — its `:610-616` comment prohibits the
variadic tail the plumbing would need). **I-2 + M-2** (§6.3 re-justified on LINE
COUNT via the shipped two-line record at `backup/passphrase.go:250-253`; §11.4's
mutation replaced with a placement one — measured, locator rows are 6/24/**27**
/29/30 chars against a 32-char cap, so **all fit** and the old mutation could not
fail). **I-3** (§10.2: no guard, no test; the property is by construction, one
call site inside `hashlockPhraseRoute`). **I-4** (§10.1 to the present tense of
what is HELD; re-measured **185/360** and **186/360**). **I-5** (§4.3 corrected
and given a three-id partition table; §8.1.2 becomes the *existing*
`PreimagePlate` body rewritten — `entr` never reaches it). **I-6 = tests I-2**
(see §3 below). **I-7 = journey I-2** (§5.2's four-part derive rule).

Journey **I-3** (§3.3 item 3 extended to `phrase:` with both hand-build errors
named; §8.2.3 gains a phrase body). **I-4** (§8.1.1's last sentence conditional on
id `hash`; §8.1.2 gains the 1-in-256 sentence). **I-5** (§10.3 states md1 **and**
mk1 with F-132 as the reason — verified `bundlePlateMark` excludes only `cardMS1`
and `composerMintCards` emits `cardMK1`; §11.5's row now names mk1, without which
every assertion passed either way). **I-6** (new §8.8, measured **165/397**).
**I-7** (§5.4 tables both abort windows; §8.4b measured **86/476**).
Tests **I-1** (§6.5's 5.0 mm advance → **3.333 mm**; measured 21,333 units).

---

## 3. Measurements that REPLACED spec numbers

| body | spec claimed | measured | note |
| --- | --- | --- | --- |
| §8.1.2 as written | 165 / 397 | **frame blanks** (5,004 ink px, floor 6,000) | the `§` glyph |
| §8.1.2 ASCII | — | **236 / 320** | + collision sentence **340 / 223** |
| §8.4a alone | 288 / 244 | **75 / 476** | claimed value unobtainable |
| §8.4b (new) | — | **86 / 476** | journey I-7's arm |
| §8.8 (new) | — | **165 / 397** | |
| §10.1 arm A / B | 153 / 378, 159 / 378 | **185 / 360, 186 / 360** | of the *corrected* wording |
| §6.5 advance @5.0 mm | 3.435 mm | **3.333 mm** | 21,333 units |
| `(preimage in payload)` row | — | 50 ch / 348 px / **2 lines** → replaced by `(in payload)`, 41 ch / **343 px / 1 line** | |

**Reproduced unchanged** (so the fold did not disturb them): §8.3 107/455, §8.5
126/378, §9 204/302 and 184/378; the whole §7.1 ceiling table and ECC-L
thresholds; **§7.2's alignment table, independently re-derived from the encoder
bitmap for v2–v9, every row exact**; §6.5's chars/line 19/23/26/31/34/39 and
lines/plate 13/15/17/20/23/26; the QR envelopes 31.80 / 47.70 mm; the 65 mm
budget and every FITS/OVER verdict.

**§8's ASCII rule** now carries its mechanism (`font/bitmap/bitmap.go:33` —
an unrenderable rune blanks the **entire** frame while `ExtractText` still reports
the text) and a host/device split, with §11.5 asserting ASCII mechanically over
the modal table. Verified: **0 non-ASCII characters in any device copy string**;
the 8 remaining are host stderr lines, which are exempt and already shipped that
way.

---

## 4. Declines, each with a true reason

1. **fidelity C-3's prescribed remedy** — adds a second call site (the `--expect`
   defect class, quoted from the shipped comment) and two corpus answers per row.
   The defect is folded; the remedy differs.
2. **journey C-1 option (a) as worded** — would have specified Rust work already
   shipped. Corrected per the controller; the Go half is folded in full.
3. **journey I-1's `show`-toggle alternatives** (Center-press latch) — no spare
   input, and item 7's own argument says the review should not show the phrase.
   Secret-handling, non-gating (2026-08-27).
4. **journey M-1's grouping** — measured: grouping in tens is **8 rows vs 4** at
   6.0 mm and does not fit; would force 5.0 mm and change every golden. The
   ungrouped rendering is stated explicitly instead.
5. **journey M-2's fourth QR line** — declined in §8.6 rule 5 with reasoning (the
   plate's job is tool-independent reproduction; `hardened` is already the CLI
   default), so it reads as a decision, not a foreclosure.
6. **fidelity M-7's "measure and accept"** — the 374 px overlap is pre-existing
   (the shipped completeness line is **459 px**), but adding five more rows is the
   W-3 class; the block is routed through `composerPageLines` instead.
7. **tests M-1** — explicitly not a finding against the spec; moot now that
   §8.4's arms are measured alone. Recorded in §8.4.

---

## 5. What I could NOT make consistent — named in the spec's §16

1. **The four `constantTimeQRModules` values are not settled here.** My floor is
   demonstrably not converged (4,000 samples → 785/1281; 216,000 → 813/1369). The
   spec specifies the protocol, the per-version record, and the floor as a gate,
   and leaves the four runs as plan-time work. **A plan that pastes a number
   without a fuzzing run is not following this spec.**
2. **§12 item 8's phone scan is unrun and unrunnable at spec time.** 53 modules at
   0.6 mm on steel has no precedent on either axis and the SH2 has no camera. The
   spec makes the scan a **gate on offering the QR at all** and states the
   fallback (text-only phrase form), so a negative result is a decision rather
   than a re-plan.
3. **Two design questions were pushed to §13 rather than answered**: a CLI
   producer for `phrase:` records (a wire form with no writer, mitigated by the
   new phrase-orphan warning), and what a *payload* phrase should be told in place
   of the reconcile screen.

---

## 6. Verification run on the folded text

- **171 line-bearing citations, 171 resolve in range**, checked against
  pristine `fb0dd04` / engrave `a0f832d0` / ms `504ff46`. Every **new** citation
  was additionally content-checked (the cited line says what the spec claims);
  **6 drifted citations were corrected**: `freetext_test.go:75→109`,
  `composer_engrave.go:40-61→80`, `composer_door.go:93-97→86-90`,
  `composer_flow.go:388→381`, `freetext.go:11-19→backup.go:111`, plus
  `hashlock.rs:186→184` and `:353→352` found by my own content check of citations
  I had just added.
- **Propagation sweep** for superseded phrasing caught four misses inside my own
  fold — including the §6.5 table cell, whose edit had been silently rolled back
  by an assertion failure in the same script. All fixed and re-verified.
- **0 non-ASCII in device copy**; host bodies labelled and exempt.
- Working tree: **only** `design/SPEC_hashlock_H6_preimage_plates.md` modified.
  Nothing committed. Both scratch worktrees removed.
