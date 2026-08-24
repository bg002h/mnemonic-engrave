# R9 — Fold verification of the R8 cycle (mechanical check, not a fresh audit)

Verifies fold commits `9353bdd, 67561f6, 1b95ebf, 34e90f4, a89b539, 136eb66, 327af69,
30d256a, 7da5e76` (plus superseded `e5c6b5e`) against `design/agent-reports/
R8-plan-spec-coverage.md` (5C/13I), `R8-plan-gates-and-fork.md` (6C/11I), and the
operator-substituting `R8-fable-ruling-nums-defence.md`. Two questions only: did the
fold fix each finding it claims to, and did it introduce a new defect. Structure,
numbering and citation existence are already machine-verified
(`spec-structure-check.sh` → STRUCTURE OK, 17 sections, 58 cross-refs;
`plan-cite-check.sh` → 0 dangling on both docs) and not re-checked here.

**Result: every Critical is closed (fixed, dissolved-by-ruling, or WON'T-FIXed by the
operator). Of 24 unique Importants, 20 are FIXED, 1 is SUPERSEDED, 3 are PARTIAL
(concrete sub-items the fold's own commit message claimed were folded but were not).
The fold also introduced three new Important-severity defects — all stale-arithmetic
or uncorrected-claim residue from find-and-replace edits that did not recompute or
did not propagate to every site.** None is Critical: none changes what gets built or
produces wrong bytes: they are false/contradictory numbers and an uncorrected false
security claim sitting in the document.

---

## Section A — finding-by-finding

### Criticals — `R8-plan-spec-coverage.md`

| id | status | evidence |
| --- | --- | --- |
| C-1 (P0 gate exits 4) | **FIXED** | Plan P0 gate is now `cargo build --locked`, clippy `-D warnings`, `fmt --check`; nextest explicitly excluded with the exit-4 measurement recorded (lines 148–167) |
| C-2 (P1 wrong port source / bit-packer mismatch) | **SUPERSEDED** | a89b539's per-field alignment removes the bit-packer dependency entirely — plan line 199: "no bit packer: every field is a whole number of 5-bit symbols." Commit message states explicitly: "This also dissolves R8 coverage C-2" |
| C-3 (`PLATE n OF m` printed, §0a deletes it) | **FIXED** | `PLATE n OF m` deleted from the spec (9353bdd, 67561f6); plan line 269-271 lists five fields "(Six until `PLATE n OF m` was deleted…)" |
| C-4 (P4 gate demands identical rendering, unsatisfiable) | **FIXED** | Plan P4 gate rewritten: agree where the row table predicts agreement, differ exactly where it predicts difference (lines 391–415) |
| C-5 (no test for wrong HRP/version/plain count) | **RULED WON'T FIX** | Operator ruling, plan §2a (lines 497–525): the pinned byte-exact vector fails on all three by construction; BCH-protected header means a miscut index self-corrects under `t=4`; recorded, not silently dropped |

### Criticals — `R8-plan-gates-and-fork.md`

| id | status | evidence |
| --- | --- | --- |
| C1 (= coverage C-1) | **FIXED** | same as above |
| C2 (cross-format negative can't fail on the mistake it exists to catch) | **FIXED** | 34e90f4 deletes the test, replaces with pinned vector + drift test + 4 `assert_ne!` lines (plan lines 95–121); spec §12.22 carries the same correction |
| C3 (defense aimed at wrong neighbour — mk not md) | **FIXED** | plan lines 109–115: `assert_ne!` against **both** siblings' constants and domain strings, hardcoded literals, explicitly because "the mistake that actually gets made is against the crate you have open" (mk) |
| C4 (`mt verify` gated by nothing) | **FIXED** | P3 now has a `verify` deliverable (structural-only, margin report, re-derivation failure path, `--transaction`) and 4 gates (lines 319–375) |
| C5 (nothing gates that BCH correction ever corrects) | **FIXED** | P1 test 4, "a correction test that CORRECTS": mutate 1–4 symbols, assert repair; mutate 5, assert refusal (lines 223–227) |
| C6 (vectors self-generated, no external anchor) | **FIXED** | spec-authored, independently-derived pinned byte-exact vector required "before `mt-codec`'s first commit" (plan lines 97–102) |

**All 10 unique R8 Criticals closed** (the ledger in `327af69`'s message — 11 raw → 10
unique, P0's gate found by both lenses — checks out against the above).

### Importants — `R8-plan-spec-coverage.md`

| id | status | evidence |
| --- | --- | --- |
| I-1 (module list is md's, mis-attributed to mk; omits pipeline.rs) | **FIXED** | plan lines 172–184, list corrected from `ls`, `pipeline.rs` added |
| I-2 (verify's margin report unowned) | **FIXED** | P3 deliverable, lines 329–341 |
| I-3 (3 mandatory encode-time stderr blocks unowned) | **PARTIAL** | P2 lists BEARER warning + correction-coverage block + "verify the STEEL" block (lines 272–279) — but the *third* block the finding actually named, §6a's **encode-shaped no-node warning** ("The transaction may already be unspendable… Consider re-running with a node before cutting," spec line 1919–1920), is not in P2 or P4's deliverable. BEARER warning is a real, ruled block but was not one of I-3's three; it was substituted for the missing one, not added to it. Grep confirms: `unspendable`/`Consider re-running` do not appear in the plan at all |
| I-4 (3 ruled CLI behaviours unowned: `--transaction`, `--quiet`, TTY line) | **PARTIAL** | `--transaction` fixed (P3, lines 347–348, via the C4 fix). `--quiet` and the TTY welcome line: **zero** hits in the plan. The fold's own commit message mislabels this as `"I-4/I-11"` and folds only I-11's content (1-based numbering) — a bookkeeping conflation between two unrelated findings that left I-4's other two sub-items untouched |
| I-5 (§10.10 input table: FROM/TO/values/node location unowned) | **FIXED** | P2 deliverable, lines 254–259 |
| I-6 (§6a's refusal outside §8 numbering, invisible to exhaustiveness gate) | **PARTIAL** | The *gate mechanism* is fixed — `refusals.toml` is a manual list, not a §8-numbering parse, so it is no longer structurally blind (plan lines 456–459 name §6a explicitly as the motivating example). But the finding's "smallest change" ("name it explicitly in P4 or P5's deliverable") is not done — P4's bullet list (lines 382–385) has no mismatch-refusal item |
| I-7 (P5 gate can't close: §8.2/§8.8 aren't refusals, only §8.7/§8.7c excluded) | **FIXED** | Explicit list replaces numbering-parse; "Not implemented: §8.7 and §8.7c" (line 441); §8.7b confirmed still live in spec (line 3609), correctly not excluded |
| I-8 (no fixture corpus for P2/P5) | **FIXED** | P2 deliverable, "the FIXTURE CORPUS" (lines 287–293) — generic ("one PSBT per refusal that must fire") rather than the original's itemised list, but the phase-ownership gap is closed |
| I-9 (§12.13 ×3, §12.10 ×1 cited, neither exists) | **FIXED** | Verified by grep: zero live hits for `§12.13` or `§12.10` in either document; all `§10.13`/`§10.10` citations resolve |
| I-10 ("§10.10 spellings only" false — exit codes, refusal format also open) | **FIXED** | plan lines 555–574, explicit correction with the false quote reproduced and retracted |
| I-11 (1-based chunk numbering unowned) | **FIXED** | P2 deliverable, lines 261–264 |
| I-12 (spec lives in a different repo than the gates that read it) | **FIXED** | P0 deliverable copies the spec + vector into `mnemonic-transaction` with source SHA recorded (lines 132–136) |
| I-13 (P2's gate stops passing once P5 lands) | **FIXED** | Explicit two-kinds-of-vector callout (lines 298–305) |

### Importants — `R8-plan-gates-and-fork.md`

| id | status | evidence |
| --- | --- | --- |
| I1 (header restatement drops version/chunked values) | **SUPERSEDED** | Header fully rewritten with explicit values (`version = 0b00001`, no `chunked` bit at all) — plan lines 194–203 |
| I2 (chunking test can't distinguish balanced/filled) | **FIXED** | P1 test 5, explicit boundary property (lines 228–235) |
| I3 (header.rs ported from mk, widths differ, no header-layout test) | **PARTIAL/SUPERSEDED** | Core premise (bit-packer port mismatch) dissolved by per-field alignment (no bit packer needed at all). But the specific ask — a header round-trip test at the field boundaries (max `count`/`index`/`chunk_set_id`) — is not present in P1's 5 listed tests; not tracked anywhere as a gap |
| I4 (P4 gate fails on conformant code, omits `verify`) | **FIXED** | P4 gate rewritten (= coverage C-4 fix); `verify` separately gated in P3 |
| I5 (P4 gate passes vacuously offline) | **FIXED** | "Gate, run BOTH with node fixtures and offline" + explicit rationale (lines 391–400) |
| I6 (mutation discipline is prose, no mechanism) | **FIXED** | `scripts/mutate-refusals.sh` described, including the vacuous-control guard (lines 428–439) |
| I7 (exhaustiveness script has no well-defined input) | **FIXED** | = coverage I-7 fix |
| I8 (§12.13 dangling) | **FIXED** | = coverage I-9 |
| I9 (P1 built on an item the plan calls non-blocking-open) | **FIXED** | plan lines 566–570, explicit reconciliation |
| I10 (shared-crate premise stale — retired, not deferred) | **FIXED** | plan lines 59–73, retirement note quoted and reasoned through |
| I11 (no provenance pin; no 3-way defect-check obligation) | **NOT FIXED** | Zero hits for "forked from", "fork date", "provenance pin" anywhere in the plan. Not mentioned in either fold commit's list of what was worked. Genuinely dropped |
| I12 (no gate runs the whole surface; CI narrower than sibling — nextest skips doctests) | **PARTIAL** | The `-p mt-codec` scoping problem is fixed ("From P1 onward the gate is the WHOLE validation surface," lines 152–156). The doctest sub-point is not: zero hits for "doctest" / `--doc` / `cargo test --workspace` anywhere in the plan, so `///` examples remain uncompiled by any gate, exactly as the finding said |
| I13 (P6 journeys undefined, placeholder cross-ref) | **FIXED** | Three named journeys A/B/C with explicit assertions (lines 475–491) |
| I14 (§10.14 "before implementation" vs. "does not block P0-P2") | **FIXED** | plan lines 545–551, reconciled |

### Minor / Nit (non-gating, spot-checked only)

M-2 (live-node smoke test unscheduled) — **fixed** (plan line 417, explicit non-gating
checklist item). M-1 (§10.20 no owning phase) and M-3 (P6 has no "Tests first" line) —
**not addressed**; both remain exactly as R8 found them. Not gating; not chased
further per the brief's scope.

### The fable ruling (`R8-fable-ruling-nums-defence.md`)

All four "order of work" items were checked against the plan/spec:

1. **"Fold §10.13(a): replace the false cross-verification sentence"** — **NOT
   DONE at the cited location.** See Section B, defect 3: the fold corrected §12.22
   (the historical appendix) but left §10.13(a) itself — the normative "RULED, ready
   to build" section — carrying the same false sentence, uncorrected.
2. Reference script + pinned vector landing in the spec before first commit — plan
   requires this (lines 97–102); the vector/script are not yet materialized (this is
   pre-implementation, expected).
3. D2's four `assert_ne!` lines — **done** (plan lines 109–115, matches D2 exactly,
   hardcoded literals against both siblings).
4. Strike the cross-format negative — **done** (34e90f4, confirmed nowhere else in
   either doc as a live requirement).

---

## Section B — new defects introduced by the fold

All three are stale-arithmetic or uncorrected-claim residue from a89b539's and
34e90f4's edits: a value was updated at one site and not recomputed or not
propagated to every site carrying it. None changes wire format or implementation
behavior; all are false or self-contradicting statements a reader/implementer would
encounter.

### B-1 (Important) — §0's summary table understates the real ceiling by ~8x

`design/SPEC_mt_v0_1.md:24`:

    | **`mt encode`** | ... | raw signed transaction (§3b) | **32,768 chunks / 164 KB** |

`164 KB` ≈ `163,840` bytes = the **old** 12-bit-field ceiling (`4,096 × 40`). The
current ceiling, stated correctly everywhere else in the document (lines 71, 1246,
1498, 2781, 3609), is `32,768 chunks = 1,310,720 bytes` ≈ 1,280 KB. `git log -p` on
this line confirms the mechanism: a89b539's diff changed `4,096 chunks / 164 KB` →
`32,768 chunks / 164 KB` — the chunk count was updated, the byte figure was not
recomputed. This is line 24 of a 3,729-line document — the first table a reader
sees — understating the real capacity by roughly 8×.

### B-2 (Important) — a stale sub-heading and a stale headroom ratio, from the same edit

`design/SPEC_mt_v0_1.md:1250-1254`:

    > **Why 12, and the bound is Bitcoin's rather than ours.** Operator ruling
    > 2026-08-23. ... That is **2,500 chunks**; 32,768 covers it with 1.6x headroom.

Two residues in one paragraph, same mechanism as B-1 (confirmed via `git log -p`:
a89b539 changed only the literal `4,096` → `32,768` on this line, per its own commit
message's residual grep for "4,096" / "163,840"):

- The heading still says **"Why 12"** although the field is now 15 bits.
- **"1.6x headroom" is arithmetically wrong for 32,768.** `32768 / 2500 = 13.11`.
  `1.6x` is the *old* ratio (`4096 / 2500 = 1.64`), for the 12-bit field the
  heading is still titled after. The correct figure, `13.1x`, is stated
  correctly in a89b539's own commit message ("13.1x headroom where 12 bits gave
  1.6x") but never landed in the spec text itself — the commit's fix touched the
  chunk-count number and missed the headroom-ratio number three words later in
  the same sentence.

### B-3 (Important) — §10.13(a), the normative section, still asserts the false claim that §12.22 retracts

`design/SPEC_mt_v0_1.md:3155-3158`, inside "13. `mt1`'s own encoding, NUMS constant
and content id — **RULED, ready to build**":

    `MD_REGULAR_CONST` is hardcoded into checksum create and verify
    (`crates/md-codec/src/bch.rs`). Every constellation format gets its own;
    without a distinct one **an `mt1` chunk would verify as a valid `md1` chunk**,
    which for a bearer plate sitting in a drawer beside `md1` plates is a real
    hazard, not a theoretical one.

This is the exact sentence 34e90f4's commit message says was found false and fixed
("THE SPEC WAS ASSERTING SOMETHING FALSE... Verified at source before folding"), and
which the fable ruling's order-of-work item 1 explicitly named as the site to fix
("Fold §10.13(a): replace the false cross-verification sentence with the real
rationale... One paragraph, no new scope"). The correction was in fact written —
but at **§12.22** (line 3699: `"CORRECTION 2026-08-23 — this entry said the
consequence was that 'mt1 chunks verify as md1 chunks', and that is FALSE..."`),
the historical appendix, not at §10.13(a), the live normative section the fable
ruling named and that P1's implementer actually reads ("RULED, ready to build" —
the plan cites `§10.13 a` directly, e.g. plan lines 194, 200). The document now
contradicts itself: read top-down from §10.13 and the false claim stands
unqualified; read the appendix at §12.22 and it is retracted. Grep across both
documents confirms this is the only surviving live instance — the plan (line 83)
and §12.22 both carry the corrected version.

**Why these three are Important, not Critical:** none produces wrong bytes or
changes what an implementer builds — B-1/B-2 are capacity-description errors in
rationale/summary text, not in the normative field-width table (which is correct
everywhere it's load-bearing), and B-3 is a security-rationale claim, not the
security mechanism itself (the HRP-mixing that actually protects the format is
correctly described in the plan and in §12.22). But all three are false or
self-contradicting statements a reviewer or implementer could reasonably rely on,
in a document whose own review history is built on "comments outlive their
conditions" and "a stale pin hides the next defect" as named failure classes.

---

## Section C — checked and correct, not re-derived

- **Arithmetic, recomputed independently, matches the document everywhere except
  B-1/B-2 above**: `5+20+15+15=55` bits `=11` symbols; invariant `5+20+15=40`
  bits `=8` symbols; index `15` bits `=3` symbols; `2^15=32768`; `32768×40=
  1,310,720`; elision saves `8×(n−1)` — `32`/`104`/`496` for 5/14/63 strings,
  matching the doc exactly; net vs. the 50-bit layout for 14 strings:
  `104 − 14 = 90`, matches the doc's "90 characters cheaper."
- **Legend arithmetic**: 5 fields, 152 characters, 6 lines — consistent across
  every live citation (lines 1657, 1758, 2636); the superseded "six fields / 164
  ch / 7 lines" figures survive only inside their own retractions, each read and
  confirmed to say so explicitly.
- **Grep sweep for the brief's named stale terms** — `49`, `50 bits`/`50-bit`,
  `4,096`, `163,840`, `chunked(1)`, `version(4)`, 12-bit count/index, "first 7
  characters": every live hit is either a false-positive substring match (`2,498`
  contains `49`; `496` contains `49`) or sits inside an explicit retraction/
  quotation (plan lines 205–206: *"This block said '49 bits...' until
  2026-08-23"*; spec line 1233 quoting the pre-R3 37-bit layout as history). No
  surviving *live* claim of any of these values, other than B-1/B-2/B-3 above
  which are a different residue class (derived numbers, not the literal strings
  the brief listed) — which is exactly why they weren't caught by the fold's own
  grep-based self-checks.
- **§8.7b status**: confirmed still live in v0.1 (spec line 3609, "the
  1,310,720 B ceiling stands, and §8.7b refuses past it"); P5's "Not implemented:
  §8.7 and §8.7c" correctly excludes only the QR-deferred siblings, not §8.7b —
  no contradiction.
- **§10.13 / §12.22 numbering**: §10.13 is correctly the open-but-RULED section
  P1 is built from (plan explicitly reconciles this at lines 566–570); §12.22 is
  correctly the settled appendix entry for the NUMS domain string. No §12.N
  citation found pointing at an item that is still open, or vice versa, anywhere
  in either document.
- **§10.4 citation** (plan line 256, the free-text TO label) resolves to a
  "SETTLED — reasoning in §12.4" stub, consistent with this spec's own established
  convention (§10.1/2/3/9 are the same "number preserved, MOVED/SETTLED pointer"
  shape) — not a miscitation.
- **The fold's own gate output claims** (spec-structure-check / plan-cite-check
  pass/fail counts quoted in each commit message) were not re-run since the brief
  states they're already machine-verified; spot-checked only that the final state
  (0 dangling, `§12.13`/`§12.10` gone) matches what the last commit claims.

---

**Counts.** Criticals: 10/10 unique closed (5 FIXED, 1 SUPERSEDED, 1 WON'T-FIX-ruled
from the coverage lens's 5; 5 FIXED, 1 already-counted from the gates-and-fork
lens's 6). Importants: 24 unique, 20 FIXED, 1 SUPERSEDED, 3 PARTIAL, 1 NOT FIXED.
New defects: 3 (all Important, all B-1/B-2/B-3 above).

Report written by the R9 fold-verification pass as its final action, per the
standing agent-persistence rule. Read-only: neither document was edited.
