# R0 architect review — SPEC_sizeproof.md — round 4 (fold check) — **GREEN**

Two independent lanes dispatched 2026-08-05 against the R4 spec @ `615ad26`,
scoped to "did the fold fix each round-3 finding, and did it introduce a new
defect". Opus design-adversarial + sonnet mechanical, merged by an independent
opus synthesiser. Rounds 0-2 declared closed in the brief; the measured numbers
declared settled. Persisted VERBATIM.

VERDICT: **GREEN — 0 Critical / 0 Important.** Both lanes GREEN
independently (opus 0C/0I + 6 non-gating, sonnet 0C/0I + 0 findings).
**This closes the R0 gate.** The six items below are documentation precision
and are folded inline, not by another round.

---

## MERGED GATE VERDICT (synthesiser)

VERDICT: GREEN — 6 findings

### 1. Minor — §3.1 — the blockquote restating `ftPlan.Blocks`

**The blockquote "`Blocks` emits `min(parts, runs)` blocks … It collapses to ONE block only when the text has exactly one '\n'-part" is false for the shipped two-run plan, and §3.1 routes that wording into `freetext_flow.go:107`'s corrected doc comment.**

*Failure:* Verified myself by probe (now deleted). `ftPlanBoth` is `[{sh, Blocks: 4}, {const, Blocks: 0}]` with `ftProofBothSplit == 4` (freetext_proof.go:200-206). It emits ONE block at 2, 3 and 4 parts — not `min(parts, 2) == 2` — because the non-final run's `n > len(parts)` clamp (freetext_flow.go:116-117) makes run 0 swallow everything and `break` at :127. The shipped test at gui/freetext_flow_test.go:1134-1141 already pins that collapse, so the spec's summary contradicts a green test. The generalisation holds only for plans whose every run declares 1 (i.e. the two ladders). NOT promoted to Important: nothing normative rests on it — the predicate is on the part count, and §3.1's measured six-run table is correct — but this is precisely the class of "described from a doc comment rather than from the code" error the R4 preamble names as the root cause of every Important since round 1, and the wrong text is on a direct path into the source comment that misled R2.

*Fix:* State the rule the code implements: each non-final run takes `min(Blocks, remaining)` parts, the walk stops when parts run out, and the final run takes whatever is left — so `len(out) == len(Runs)` only once every non-final run's declared share is satisfied (`parts >= len(Runs)` for an all-1 plan, `parts >= ftProofBothSplit+1` for `ftPlanBoth`). Keep the measured table as the normative statement.

### 2. Minor — §3 (site table) vs §5 (`ftProofOutcomeFor`)

**The §3 table, introduced as the complete list of sites that must change, has no row for `ftProofOutcomeFor` (`gui/freetext_proof.go:525`) — the function that hardcodes `Footer: ftProofFooter` for every non-Sizeable proof, including both ladders. The mechanism by which it learns to suppress the footer is never specified.**

*Failure:* Confirmed at source: freetext_proof.go:525 is `return ftProofOutcome{Text: p.For(useQR), Title: p.Title, Footer: ftProofFooter, Plan: p.Plan}`. §3's table row is for the `ftProofOutcome` STRUCT at :511 ("carries the plan … not a single rung"), and the `proofPreview` row only removes the preview's own literal at preview.go:130 — routing it through a resolver that still returns `ftProofFooter`. §5 does mandate the behaviour and does name the site ("must not inherit `ftProofFooter` — not from `ftProofOutcomeFor`"), which is why this is not Important: an implementer who ships the footer gets a hard failure at §7.10 (FitSized refuses — §5 measures FRONT short by 3.200 mm, BACK by 1.600 mm), a visible refusal, not wrong steel. But the discriminator is left to invent: the only new field §3 introduces is `Side string`. Related wart, same site: `ftProofReplaces` (:538) builds ", Footer becomes " + out.Footer + ".", rendering "Footer becomes ." on the consent screen when the footer is empty.

*Fix:* Add a §3 row for `ftProofOutcomeFor` (:521-526) stating the footer is per-proof; name the mechanism (e.g. `ftProof` gains `Footer string` defaulting to `ftProofFooter`, empty for the two ladder entries); and say what `ftProofReplaces` prints when the footer is empty.

### 3. Minor — §2.1 anchor table vs §2.1 prose / §2.3 / §7.7(c)

**`EngraveFitted` is listed in the anchor table as a caller that chooses `anchorY = params.I(outerMargin)`, while the same section, §2.3 and §7.7(c) all require it to build no placement at all and read `f.qrAt`.**

*Failure:* Two paragraphs below the table: "`EngraveFitted` cannot: it receives only `Fitted` … so with no field to read it would have to call `qrPlaceAt` again — the second derivation this section exists to abolish"; §2.3 gives it `qrAt`; §7.7(c) asserts "neither engraver computes a y of its own". Two normative statements about the same function disagree. NOT Important: §2.1.1's `QR != nil ⇒ !Mixed` guard makes the dangerous branch (`params.F(f.SizeMM)` with `SizeMM == 0`, the `qrLines` divide-by-zero this spec calls R0's C5) unreachable, and on a uniform plate the recomputed placement is identical, so the consequence is a redundant second derivation rather than a wrong y — and §7.7(c) would still pass. The defect is in the document, not in the resulting plate.

*Fix:* Split the table into placement PRODUCERS (`fitBlocksAt`, `FitSized` (nil), `AdmissibleBlocks`, `rowFaces`/`MaxCharsAtBlocks`, `EngraveFreeText`, `EngraveText`) and placement CONSUMERS (`EngraveFitted`, which reads `f.qrAt` and never calls `qrPlaceAt`).

### 4. Minor — §1 and §8 vs §3 and §4

**§1 ("The firmware gains no concept of a side") and §8's non-goal ("Any notion of plate sides in firmware") contradict §3's `ftProof` gains `Side string` and §4's requirement that the prompt name the side from that field.**

*Failure:* §4 is safety-motivated — both sides prove identical faces, so `Plan.Name()` gives near-identical prompts and "a mis-pick is engraved where a mistype is refused" (R0's I5). A reader taking §8's non-goal literally drops the `Side` field. NOT Important, for two reasons I checked: (a) §3's table row and §4 both state the requirement explicitly and unambiguously, so dropping it means overriding an explicit requirement with a one-line non-goal; (b) even in that case the prompt is not actually ambiguous — `ftProofReplaces` already prints "Title becomes " + out.Title, and the two ladder titles are `FRONT 5.0+3.8` and `BACK 4.4+3.4+3.0`, so the side is named regardless. The two statements are reconcilable (a label string is not a model of a two-sided plate — no pairing, no flip, no ordering) but the document never says so.

*Fix:* Narrow the non-goal to what is meant: §8 "Any RELATIONSHIP between the two sides in firmware — no pairing, no flip prompt, no ordering. `ftProof.Side` is a label the prompt prints (§4), nothing more." Mirror it in §1.

### 5. Nit — §2.7 / §2.3 — `FitSized`'s own `Mixed` and `SizeMM`

**No clause assigns `FitSized` the job of computing `Mixed` and `SizeMM`; §2.3 spells the population rules out in full for `fitBlocksAt` and `EngraveFreeText` only.**

*Failure:* §2.3 does define `Mixed` as a general property ("true when `Sizes`, `TitleSizeMM` … and `FooterSizeMM` … are not all the same value"), so this is derivable, and every ladder composition is genuinely mixed so the flow never exercises the degenerate case. But `FitSized` is a public entry point and an all-3.0 composition with a 3.0 title is legal, whereupon an implementer who hardcoded `Mixed: true, SizeMM: 0` puts "0.0mm" on the readout — the defect §3 bolds.

*Fix:* One sentence in §2.7: "`FitSized` sets `Sizes` from the resolved per-block sizes, computes `Mixed` by §2.3's rule, and sets `SizeMM` to the common value when `!Mixed` and 0 otherwise."

### 6. Nit — §3.1 `declaredParts` / §7.13

**`declaredParts` is numerically identical to `len(p.Runs)` for both ladders, so no §7.13 case can distinguish the specified predicate from a `len(Runs)`-based one; and `ftPlan.Blocks`' single-run early return (`freetext_flow.go:113-115`) bypasses the split entirely, so a hypothetical one-run sized plan would never have its sizes cleared.**

*Failure:* FRONT is 4 runs x `Blocks: 1` and BACK 6 x 1, so `sum(Blocks) == len(Runs)` for every fixture — a mutant computing `len(strings.Split(text,"\n")) == len(p.Runs)` passes 6/5/7/1 identically, and §7.19's mutation pass cannot catch it either. Both residues bite only a future sized plan with a run declaring `Blocks > 1` or a single sized run; §3.1 states the constraint ("A plan whose runs carry sizes must declare `Blocks` on every run") but nothing pins it. Note this does NOT weaken the round-3 I1 fix: I confirmed by probe that §7.13(c) at 7 parts gives `len(out) == 6 == len(Runs)`, so the block-count predicate the fold replaced does fail that case.

*Fix:* Add to §7.13 a case over a synthetic sized plan with runs `[2,1,1]` (`declaredParts == 4 != len(Runs) == 3`), asserting sizes survive at 4 parts and clear at 3 and 5, plus a well-formedness assertion that every run of every sized plan declares `Blocks >= 1` and that a sized plan has more than one run.

### Notes

MERGE RESULT: GREEN — 0 Critical / 0 Important. Both lanes returned GREEN independently (opus 0C/0I + 6 non-gating; sonnet 0C/0I + 0 findings). No overlap to dedupe — sonnet filed nothing — so the six surviving items are opus's, each re-verified by me against source rather than taken on report. Nothing was dropped as unreal or out of scope; nothing was promoted. I explicitly considered and REJECTED promoting two of them: the §3-table omission of `ftProofOutcomeFor` (M2) fails hard at §7.10 rather than reaching steel, and the §1/§8-vs-§4 contradiction (M4) is doubly guarded — §4 states the requirement explicitly and `ftProofReplaces` already prints the side-bearing title regardless. I also declined to manufacture an Important out of "§7 never pins the §4 side-naming prompt", for the same reason.

(a) §3.1's SHAPE TABLE — REPRODUCED, third independent time. My own throwaway probe (gui/zzmerge_probe_test.go, since deleted) replayed the real `(*ftPlan).Blocks` over a six-run Blocks:1 plan: parts→blocks = 1→1, 2→2, 5→5, 6→6, 7→6, 8→6, with `len(out)==len(Runs)` false at 1/2/5 and true at 6/7/8. Block contents matched the table's prose verbatim: at 7 parts blocks 0-4 hold one part each and block 5 holds "f|g"; at 8 parts block 5 holds "f|g|h". The four-run FRONT plan behaves identically (4→4 eq, 5→4 eq). The table is exact, and the part-count predicate is exact rather than merely adequate: at `parts == declaredParts` no non-final run's clamp fires and the final run's forced `n` equals its declared 1, so block i == part i in run i's face AND size; below it the walk stops early, above it the final run over-absorbs — both give `parts != declaredParts`. Deletion AND insertion are both caught, which is what round 3's I1 demanded.

(b) FOLD-COVERAGE GAPS — none. All nine round-3 items (0C/2I/4M/3N) are folded and I spot-checked the load-bearing ones at source: I1 → §3.1's part-count predicate plus §7.13's fourth shape (c); I2 → §2.1's bolded `AdmissibleBlocks` row and §7.20's `useQR = true` case; M1 → the error/panic split in §2.1.1 now matching §7.7(d); M3 → the `ftProofLoader` row and the "ladder's rung is 0" paragraph, which I confirmed at freetext_proof.go:670 (`*size = out.SizeMM`, 0 for non-Sizeable, so the un-edited ladder cannot trip §3's non-zero-size-plus-sized-blocks error); N1 → three `backup.Fitted{}` literals, grep-confirmed at gui/freetext_flow_test.go:564/893/928 exactly; N3 → cmd/plateview/main.go:98-103 confirmed to return the literal "fixed layout". No round-3 item is unaddressed and the fold introduced no new defect.

(c) WHAT THE CONTROLLER MUST DECIDE — three implementation choices the spec leaves open, none of which blocks starting work: (1) HOW `ftProofOutcomeFor` suppresses the footer for the two ladders (per-proof `Footer` field vs branching on `Side`), and what `ftProofReplaces` prints when the footer is empty; (2) what `FitSized` writes to `Mixed`/`SizeMM` in the degenerate all-one-size case; (3) whether to widen §7.13 with a `[2,1,1]` synthetic plan so `declaredParts` is actually distinguished from `len(Runs)`. The ladder `Text` constants themselves are NOT an open decision — §1's 4x95/6x95 and §1.1's per-block budgets force them to the 95-rune sweep repeated 4 and 6 times joined by "\n", and §7.3/§7.4 pin them.

(d) REPO CLEAN — YES. `git status --short` in /scratch/code/shibboleth/seedhammer was empty on entry and empty on exit, both at 3c3a2ad; no tracked file modified. Sonnet's operational flag is RECONCILED and is NOT an open item: the untracked `backup/zz_probe_test.go` it saw was the opus lane's probe, running concurrently in the same worktree; opus's own report accounts for creating and deleting `backup/zz_probe_test.go` and `gui/zz_probe_test.go`, and neither exists now. Sonnet was right to leave it alone under the parallel-agent-isolation rule. Standing lesson for the next multi-lane gate: two read-only reviewers sharing one worktree is fine, but two reviewers WRITING probe files into it is not — give each lane its own worktree, or the exit-clean check becomes unreadable.

(e) READY FOR IMPLEMENTATION — YES, in my judgement. The gate has run five times (rounds 0-4) with three zero-Critical rounds running, and R4 closes both of round 3's Importants without opening a new one. Every measured number is now backed by four-plus independent reproductions, and the one number new in R4 is confirmed by three. The six residual items are documentation precision — a false generalisation in a blockquote, a missing table row for a behaviour mandated elsewhere, two internal wording contradictions, and two unpinned future-proofing gaps — none of which changes what gets cut into steel and none of which an implementer would have to resolve before starting. GREEN closes the loop; folding the six Minors/Nits should be an inline edit, not another review round.

---

## Lane 1 — design-level adversarial (opus)

VERDICT: GREEN — 6 findings

### 1. Minor — §3.1 (the blockquote restating `ftPlan.Blocks`)

**The blockquote "`Blocks` emits `min(parts, runs)` blocks … It collapses to ONE block only when the text has exactly one '\n'-part" is false for the shipped two-run plans, and §3.1 tells the implementer to write it into `freetext_flow.go:107`'s corrected doc comment.**

*Failure:* Measured (temporary probe, now deleted): `ftPlanBoth` (runs `[{sh, Blocks: 4}, {const, Blocks: 0}]`, `ftProofBothSplit == 4`) emits ONE block at 2, 3 and 4 parts, not `min(parts, 2) == 2`; the non-final run's `n > len(parts)` clamp (lines 116-117) makes it swallow everything and `break` at line 127. The existing test at `gui/freetext_flow_test.go:1134-1141` already pins that collapse. So the generalisation holds only for plans whose runs each declare 1 (i.e. the two ladders), and the second sentence is falsified by any two-run plan edited below its split. Nothing normative rests on it — the predicate is on the part count and §3.1's measured six-run table is correct (reproduced exactly: parts 1,2,5,6,7,8 -> 1,2,5,6,6,6 blocks) — but §3.1 closes with "`freetext_flow.go:107`'s doc comment … is corrected as part of this change", so the wrong summary is on a direct path into the source comment that misled R2 in the first place.

*Fix:* State the rule the code implements: each non-final run takes `min(Blocks, remaining)` parts and the walk stops when the parts run out; the final run takes whatever is left. So `len(out) == len(Runs)` only once every non-final run's declared share is satisfied — `parts >= len(Runs)` for an all-1 plan, `parts >= ftProofBothSplit+1` for `ftPlanBoth`. Keep the measured table as the normative statement.

### 2. Minor — §3 table vs §5 (`ftProofOutcomeFor`)

**The §3 table, introduced as the complete list of sites that must change, omits `ftProofOutcomeFor` (`gui/freetext_proof.go:521-526`), the function that actually hardcodes `Footer: ftProofFooter` for every non-Sizeable proof; the mechanism by which it learns to suppress the footer for a ladder is never specified.**

*Failure:* §5 mandates an empty footer on both ladder plates (a footer refuses FRONT by 3.200 mm and BACK by 1.600 mm) and names the site — "must not inherit `ftProofFooter` — not from `ftProofOutcomeFor`" — but the §3 row is for the `ftProofOutcome` STRUCT at :511 ("carries the plan … not a single rung"), not for the resolver's fallback `return ftProofOutcome{…, Footer: ftProofFooter, …}`. An implementer working the table alone ships the footer and `FitSized` refuses both plates (visible refusal, not wrong steel). The only discriminator the spec introduces is the new `Side string`; whether to branch on that or add a per-proof `Footer` field is left to invent. Related wart: `ftProofReplaces` builds ", Footer becomes " + out.Footer + ".", rendering "Footer becomes ." on the consent screen when the footer is empty.

*Fix:* Add a §3 row for `ftProofOutcomeFor` (:521) stating the footer is per-proof, name the mechanism (e.g. `ftProof` gains `Footer string`, defaulting to `ftProofFooter`, empty for the two ladder entries), and say what `ftProofReplaces` prints when the footer is empty.

### 3. Minor — §2.1 (the anchor table) vs §2.1 prose / §2.3 / §7.7(c)

**`EngraveFitted` is listed in the anchor table as a caller that chooses `anchorY = params.I(outerMargin)`, but the same section, §2.3 and §7.7(c) all require it never to build a placement at all — it must read `f.qrAt`.**

*Failure:* Two paragraphs below the table: "`EngraveFitted` cannot: it receives only `Fitted` … so with no field to read it would have to call `qrPlaceAt` again — the second derivation this section exists to abolish"; §2.3 gives it `qrAt`; §7.7(c) asserts "neither engraver computes a y of its own". An implementer taking the table at face value writes `qrPlaceAt(…, params.F(f.SizeMM), margin)` inside `EngraveFitted` — exactly where §2.5 has just removed `fontSize`, so on a `Mixed` plate it reaches `params.F(0) == 0` and the `qrLines` integer divide-by-zero this spec calls R0's C5. §2.1.1's `QR != nil => !Mixed` guard makes that unreachable, so the consequence is a redundant second derivation on uniform plates rather than a panic — but the table and the prose disagree about the same function.

*Fix:* Split the table into placement PRODUCERS (`fitBlocksAt`, `FitSized` (nil), `AdmissibleBlocks`, `rowFaces`/`MaxCharsAtBlocks`, `EngraveFreeText`, `EngraveText`) and placement CONSUMERS (`EngraveFitted`, which reads `f.qrAt` and calls `qrPlaceAt` never).

### 4. Minor — §1 and §8 vs §3 and §4

**§1 ("The firmware gains no concept of a side") and §8's non-goal ("Any notion of plate sides in firmware") contradict §3's `ftProof` gains `Side string` and §4's requirement that the prompt name the side from that field.**

*Failure:* §4 is load-bearing — both sides prove identical faces, so `Plan.Name()` gives near-identical prompts and "a mis-pick is engraved where a mistype is refused". A reader who takes §8's non-goal literally drops the `Side` field and falls back to the plan name, re-admitting exactly the R0-I5 hazard §4 was written to close. The two statements are reconcilable (a label string is not a model of a two-sided plate: no pairing, no flip, no ordering), but the document never says so.

*Fix:* Narrow the non-goal to what is meant, e.g. §8: "Any RELATIONSHIP between the two sides in firmware — no pairing, no flip prompt, no ordering. `ftProof.Side` is a label the prompt prints (§4), nothing more." Mirror it in §1.

### 5. Nit — §2.7 / §2.3 (`FitSized`'s own `Mixed` and `SizeMM`)

**§2.7 never says what `FitSized` writes to `Mixed` and `SizeMM`; §2.3 defines the invariant but assigns the work to nobody.**

*Failure:* §2.3 spells out the population rules for `fitBlocksAt` and `EngraveFreeText` in full, and §2.7 lists `FitSized`'s validation and its nil `QR`/`qrAt`, but no clause says `FitSized` computes `Mixed` from "`Sizes`, `TitleSizeMM` and `FooterSizeMM` are not all the same value" and sets `SizeMM` to that common value when `!Mixed`. It is derivable from `// valid only when !Mixed`, and every ladder composition is genuinely mixed so the flow never exercises the degenerate case — but `FitSized` is a public entry point and a caller passing all-3.0 blocks with a 3.0 title is legal, whereupon an implementer who hardcoded `Mixed: true, SizeMM: 0` puts "0.0mm" on the readout, the defect §3 bolds.

*Fix:* One sentence in §2.7: "`FitSized` sets `Sizes` from the resolved per-block sizes, computes `Mixed` by §2.3's rule, and sets `SizeMM` to the common value when `!Mixed` and 0 otherwise."

### 6. Nit — §3.1 `declaredParts` / §7.13

**`declaredParts` (sum of every run's `Blocks`) is numerically identical to `len(p.Runs)` for both ladders, so none of §7.13's four shapes can distinguish the specified predicate from the wrong one, and §7.19's mutation pass cannot either.**

*Failure:* FRONT is 4 runs x `Blocks: 1` and BACK 6 x 1, so `sum(Blocks) == len(Runs)` for every fixture. A mutant computing `len(strings.Split(text,"\n")) == len(p.Runs)` passes 6/5/7/1 parts identically. The distinction only bites a future sized plan with a run declaring `Blocks > 1`, at which point the wrong predicate silently accepts a mis-shaped edit. §3.1 states the constraint ("A plan whose runs carry sizes must declare `Blocks` on every run") but nothing pins it.

*Fix:* Add to §7.13 a case over a synthetic sized plan with runs `[2,1,1]` (`declaredParts == 4 != len(Runs) == 3`), asserting sizes survive at 4 parts and are cleared at 3 and 5 — plus an assertion that every run of every sized plan declares `Blocks >= 1`.

### Notes

VERDICT GREEN — 0 Critical / 0 Important. Six non-gating items above. Repo /scratch/code/shibboleth/seedhammer checked with `git status --short` on entry and exit: clean at 3c3a2ad both times; two temporary probe files (backup/zz_probe_test.go, gui/zz_probe_test.go) were added, run and deleted; no tracked file modified.

WHAT I ATTACKED AND WHAT HELD.

(1) §3.1's new predicate — HOLDS, and it is exact, not merely adequate. Measured the one new number in R4: the BACK plan's six runs give parts 1,2,5,6,7,8 -> blocks 1,2,5,6,6,6, with len(out)==len(Runs) true at 6, 7 and 8 — §3.1's table reproduces verbatim, including "run 5 never emitted" at 5 parts and the absorb at 7/8. The loop's forced final absorb (line 116) does NOT defeat a part-count predicate: at parts == declaredParts every non-final run's n = r.Blocks is satisfiable (n > len(parts) never fires) and the final run's forced n = len(parts) equals its declared 1, so 6 parts implies block i = part i in run i's face AND size — structurally the loaded ladder. Below 6 the walk stops early (fewer blocks); above 6 the final run over-absorbs; both give parts != 6. Deletion and insertion are both caught, which is what round 3's I1 asked for. Edge cases: Split("") = 1 part -> cleared -> uniform auto-fit of an empty text (WrapText("") returns one empty line, measured); leading or trailing "\n" gives 7 parts -> cleared; five bare newlines gives 6 parts and IS kept as six empty blocks — but that is the decided policy ("exact part count keeps the ladder"), the plate is blank under a title and the readout says so, not a new hole. ftPlanBoth/ftBothPlanFor(n) have declaredParts == 4 against 9 actual parts, so the predicate is false for them on every shipped text and clearing SizeMM is a genuine no-op — they carry no sizes. Only residue, unreachable by any part-count predicate: two compensating edits (join one boundary, split another) keep 6 parts and shift content across runs. That is a CONTENT change, not a shape change, and content edits at exact shape are the decided policy; not filed.

(2) §2.1's anchor table — enumeration is COMPLETE. Exhaustive grep: wrapBlocks callers are exactly fitBlocksAt (fit.go:227), AdmissibleBlocks (:280), rowFaces (:302); textLayout callers are exactly wrapBlocks (:150), faceLayouts.at (:327, removed by §2.6) and EngraveText (backup.go:359). With MaxCharsAtBlocks (which loses faceLayouts) and EngraveFreeText (§2.3) that is the whole producer set, and all six appear. params.I(outerMargin) for AdmissibleBlocks is right, and R4's "term for term" claim verifies: with anchorY = margin and start = margin + F(size), block-relative row j has y = margin + (1+j)*fontSize, so qrTop <= y < qrBottom reduces exactly to holeLines <= 1+j < holeLines+qrLines, i.e. today's lay.at(1+i); the screw-hole predicate reduces identically in BOTH halves, including the baseY+(i+1)*fontSize term, and stays exact across block boundaries because the running y advances by len(l)*fontSize at a uniform size.

(3) §2.1.1's third guard as an error return — SAFE, and safer than the spec claims. §2.1.1 measured only 3.0 mm; I measured all six rungs and drove real fits. Largest module count each rung tolerates: 101 (6.0/5.0/4.4), 105 (3.8/3.4/3.0). Worst qrAt.Bottom any text FitBlocks actually accepts produces: 51.0 mm at 6.0, 53.0 at 5.0, 55.8 at 4.4, 56.2 at 3.8, 64.2 at 3.4, 66.0 at 3.0 — against an 82.0 mm limit, i.e. 16.0-31.0 mm of slack; same picture through FitBlocksAt with title+footer at every rung (worst 63.0 mm). So the new error can never make FitBlocks' rung walk `continue` past a rung it takes today, never reaches ErrTooLarge, and cannot move a golden. ftBothAt's drop-ladder `continue` is unaffected for the same reason.

(4) §3's table — NO FOURTH MISSING SITE of substance. Grepped every non-test reader of SizeMM, every Fitted literal, every wrapBlocks/textLayout caller and every %.1fmm. Everything lands in the table or is already covered: Fit (fit.go:251) is the single-face wrapper (uniform, SizeMM valid); preview.go:167 is covered by the "gui.Preview gains Sizes" row feeding sizeLabel's range; freetext_proof.go:286/332 take an explicit size argument and belong to BOTHPROOF. §2.1's claim that the only Fitted literals outside package backup are gui/freetext_flow_test.go:564, 893, 928 is exactly right. The one omission is ftProofOutcomeFor's hardcoded footer — specified in §5, absent from the table — filed Minor rather than Important because the behaviour IS mandated and the failure mode is a visible refusal, not steel.

(5) Whole-document consistency — the pairs I found are the two Minors above (§2.1's table vs its own prose on EngraveFitted; §1/§8's "no concept of a side" vs §3/§4's Side field). Everything arithmetic cross-checks: §1's 82.0 = 85-3 and the 3.600/2.400 spares against 78.400/79.600; §1.1's y-ranges against the stated row counts and rungs on both sides; §1.2's untitled figures (76.600 -> 5.400; the front's five-row sh@5.0 -> 2.400); §5's title lengths (13, 16) and inset spans (34-2*4 = 26 at sh 3.8, 44-2*4 = 36 at sh 3.0, from holeChars = ceil(44800/charWidth)); §2.4's start/limit branches against §2.5's footer anchor. Every cited line number spot-checked and accurate: freetext_flow.go:107/116-117/146/204/218/485/643-651, freetext_proof.go:367/511/531/538/656-672, fit.go:150/213/262/269/280/297/302/316-331/344, freetext.go:34-39/42-46/81-85/103-113, backup.go:359/383-385/390-393, preview.go:111-132/130/149, plateview/main.go:98-103. (freetext.go:71 reads .at(row) with row := start + i rather than the spec's literal .at(start + i) — same expression, not worth a finding.)

(6) Implementability of SIZEPROOF!BACK end to end — trigger match -> ftProofOffer/ftProofReplaces -> ftProofLoader (clears useQR via NeedsWholePlate, writes rung 0) -> ftEvaluate -> plan.Blocks stamps six sizes (6 == declaredParts) -> ftFitAt routes to FitSized (sized blocks tested first, rung 0) -> start = margin + F(3.0), limit = plateHeight - margin, running y, Mixed, TitleFace = blocks[0].Face = sh -> confirm screen off Sizes -> ftBuildPlate -> EngraveFitted walks y, title at margin at 3.0 in TitleFace, per-row at(0) layouts -> toPlate (79.600 <= 82.000). The only decisions an implementer must invent are the three filed above: how ftProofOutcomeFor suppresses the footer, what FitSized writes to Mixed/SizeMM, and the ladder Text constants themselves — which §1/§1.1 force to be ftProofSweep (measured: exactly 95 runes) repeated 4 and 6 times joined by "\n", but which the spec never writes down. None of the three blocks implementation.

This gate has now returned zero Criticals three rounds running and the R4 fold closes both of round 3's Importants without opening a new one. GREEN closes the loop.

---

## Lane 2 — mechanical fold-vs-findings (sonnet)

VERDICT: GREEN — 0 findings

*(no findings)*

### Notes

Independent mechanical re-review of R4 (design/SPEC_sizeproof.md) against seedhammer main@3c3a2ad. Result: 0 Critical / 0 Important. Round-3's RED (0C/2I/4M/3N) is fully folded and no new defect was introduced.

SCOPE A — fold coverage of round-3's 9 items, each checked with the spec's own text quoted and cross-checked against live source:
- I1 (edit-path predicate blind to insertions; §7.13 said THREE shapes) — ADDRESSED. §3.1 replaced the block-count predicate with a part-count predicate: 'clears SizeMM on every block it emits unless len(strings.Split(text,"\n")) == declaredParts // sum of every run's Blocks'. §7.13 now enumerates FOUR shapes, explicitly adding '(c) 7 parts — an inserted newline — reverts, and does NOT cut a six-rung plate with every band one run late and sh@3.0 absent'.
- I2 (AdmissibleBlocks missing from §2.1's anchorY table; §7.20 had no QR case) — ADDRESSED. §2.1's table gained a bolded AdmissibleBlocks row (anchorY = params.I(outerMargin)) plus a paragraph explaining start ≠ anchorY for this one caller. §7.20 now says 'with useQR = true as well as false' and states which half of the test each guards.
- M1 (§2.1.1 third guard: refused-at-fit vs panic vs §7.7(d)) — ADDRESSED. Table now reads 'fitBlocksAt/FitSized, error return; re-asserted in EngraveFitted as a defensive panic'; §7.7(d) matches this split exactly.
- M2 (fitBlocksAt never told to populate Sizes/TitleSizeMM/FooterSizeMM/qrAt) — ADDRESSED. §2.3 adds a full paragraph naming fitBlocksAt (fit.go:224-241) with the exact fill rule.
- M3 (ladder SizeMM==0 not stated; no ftProofLoader row) — ADDRESSED. §3's table gained an ftProofLoader (freetext_proof.go:656-672) row plus the 'ladder's rung is 0' paragraph citing freetext_proof.go:670.
- M4 (§7.20 pinned AdmissibleBlocks only) — ADDRESSED, folded together with I2: item 20 now also pins MaxCharsAtBlocks/rowFaces for a mixed-block composition with a boundary on a screw-hole row.
- N1 ('two' GUI fixtures should be three) — ADDRESSED, text now reads 'three GUI test fixtures' (gui/freetext_flow_test.go:564,893,928 — grep confirms exactly 3 backup.Fitted{} literals at those lines).
- N2 (footerY vs 'limit is the only name') — ADDRESSED. §2.4 now says 'limit is the name of the budget's lower end' (not 'the only name') and defines footerY as the footer branch's own top-y name; §2.5 uses 'footerY (§2.4)' consistently with this.
- N3 (sizeLabel prints 'fixed layout' not '0.0mm') — ADDRESSED, §3's table and prose both now say the plateview zero branch prints 'fixed layout', matching cmd/plateview/main.go:98-103 verbatim (grep-confirmed: `if p.SizeMM == 0 { return "fixed layout" }`).

SCOPE B — the one new measurement (§3.1's ftPlan.Blocks shape table). Wrote a throwaway probe in package gui (gui/zzprobe_sizeproof_r4_test.go, deleted after use) replaying the real (*ftPlan).Blocks against a 6-run Blocks:1-each plan with placeholder parts. Result reproduces the spec's table exactly: parts→blocks = 1→1, 2→2, 5→5, 6→6, 7→6, 8→6; len(out)==len(Runs) is false at 1/2/5 and true at 6/7/8. At 7 parts, blocks[0..4] each carried exactly one part and block[5] carried two ('p\np') — confirming 'runs 0-4 take one part each and run 5 absorbs two' verbatim. At 8 parts block[5] carried three parts, consistent with 'run 5 absorbs three'.

SCOPE C — citations. Checked every file:line R4 added/changed against live source: freetext_flow.go:107-135 (Blocks func, exact), :116-117 (exact), :195 (exact), :643-651 (ftBuildPlate — func header at 643, 651 is its last statement; closing brace is actually line 652, one past the cited range — a defensible trim, not misleading, not flagged); fit.go:224-241 (exact), :262-281 (exact), :269 (exact); freetext_proof.go:656-672 (exact), :670 (exact); plateview/main.go:98-103 (exact, includes closing brace); freetext_flow_test.go:564/893/928 (exact, three literals confirmed by grep). No stale citations found.

SCOPE D — test-plan falsifiability. 7(d) matches the corrected §2.1.1 error/panic split and would fail if the third guard were implemented only as a panic in EngraveFitted. 13's four cases include (c), the exact case that catches round-3's I1 (confirmed via my own Scope-B trace: a len(out)==len(Runs)-only predicate is blind to it; the spec's part-count predicate is not). 20 pins AdmissibleBlocks' linesUsed/ok with useQR true and false, plus a MaxCharsAtBlocks/rowFaces companion pin at a face-boundary screw-hole row.

SCOPE E — arithmetic cross-checks: §1's 78.400/79.600/3.600/2.400 reproduce identically in §1.1 and §5; 78.400+3.600=82.000=79.600+2.400, consistent with 'limit is 82.000mm'. §1.1's per-row budgets match §1.2's restatements exactly (including the untitled sh@5.0 [20 20 26 26 26] and the untitled-back 5.400mm-spare figures, 82.000-76.600=5.400). 0.119mm appears identically in the preamble and §2.6. No disagreeing pair found.

OPERATIONAL FLAG (not a spec finding — please reconcile): git status --short in /scratch/code/shibboleth/seedhammer was empty on entry. On exit it shows one untracked file I did NOT create: backup/zz_probe_test.go (TestProbeQRBottomGuard, TestProbeEmptyBlocks — content that directly probes §2.1.1's third-guard reachability, squarely inside this same review's territory). Its mtime falls inside my session window and it did not exist at my entry check, so it was almost certainly written by a concurrent process/agent sharing this worktree — not a round-3 leftover (round 3's own reports name different filenames, both confirmed deleted). Per the parallel-agent-isolation rule I did not delete or modify it since I cannot confirm it isn't another lane's in-progress work; I only removed my own probe (gui/zzprobe_sizeproof_r4_test.go). The controller should identify and reconcile with whichever other lane is running against /scratch/code/shibboleth/seedhammer right now, and ensure that lane cleans up its own file before this gate is considered to have left the repo clean.

Overall: this closes the SIZEPROOF! R0 gate at 0C/0I on the content itself. The only open item is the untracked file above, a workspace-hygiene/concurrency matter for the controller, not a defect in the spec.
