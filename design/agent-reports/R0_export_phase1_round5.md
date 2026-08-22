# R0 re-review, round 5 — PLAN_wallet_file_export.md Phase 1 fold (7dbc00f..c2e56a1) + grid follow-on (c2e56a1..2d7bc0f)

- **Reviewed:** 2026-08-22. Scope strictly: (1) did the round-4 fold answer
  R4-1, R4-2, R4-3; (2) did the fold plus the grid follow-on introduce a new
  defect, with the grid checked cell-by-cell against the rulings it restates.
  Not a fresh audit; no settled ruling reopened, and none moves below.
- **Verdict: 0 Critical / 0 Important / 2 Minor / 2 Nit — the gate goes GREEN.**
  The trend 1C/4I → 0C/4I → 0C/2I → 0C/2I → **0C/0I** has converged. The
  round-4 fold was subtractive where round 4 said to subtract (the ungated
  wording is deleted, not replaced), and both prescribed remedies were
  transcribed near-verbatim — no softening, no drift in the normative text.
  The four residual findings are two documentation Minors and two Nits; none
  changes what an implementer builds, none mandates a falsehood, and the
  acceptance set is satisfiable in every bullet.
- Checked this round (against the diffs and the current doc at `2d7bc0f`, not
  recalled):
  - R4-1's prescribed acceptance bullet vs the folded bullet: **verbatim**
    modulo punctuation ("`--allow sigless-branch` on `--template`/`--slot`
    emits the did-not-fire note — a consequence of the uniform gate, asserted
    as such; `--allow <other>` emits the unenforced-rule note. Same wordings
    as `--descriptor`, because no arm is ungated.").
  - R4-2's prescribed mechanism sentence vs the folded ruling blockquote:
    **verbatim in substance** — helper at the two named sites (`run`,
    `run_from_import_json`), each arm's canonical descriptor, `AllowSet`,
    `cmd/restore.rs:2496`/`:2801` explicitly out of scope, "Phase 1 makes no
    behaviour change to `restore`", own-decision-own-release-note. The
    parse-site enumeration bullet ("all three arms … annotated with which arm
    each serves — that annotation is what makes 'ONLY admission point'
    assertable rather than asserted") is R4's sentence word-for-word. The
    `:524`-lenient consequence appears in both the ruling and acceptance.
  - The adjacent-door honesty R4-2 required is present: the measured
    `restore --md1 --format bitcoin-core` flagless exit-0 (2694 bytes) is
    stated in the fold section next to the explicit out-of-scope ruling — the
    door stays open by choice, not by silence.
  - The grid's six cells, each against the rulings and R4's own reachable
    matrix: **all six are true.** Row 1 descriptor/import = {fired warning,
    did-not-fire note}, both reachable on each; row 1 template/slot =
    did-not-fire only (builder invariant — fired is unconstructible there);
    row 2 = unenforced-rule note on all arms. Grid reading-bullets 2–4 are
    faithful restatements of the consequence-not-exemption sentence, the
    R3-2 parenthetical rule, and the R4-1 deletion. Bullet 1 is not (M5-1).
  - Every remaining occurrence of "ungated" in the doc: the acceptance carries
    only "because no arm is ungated" (a negation); the R4-1 narrative and grid
    bullet 4 record the deletion; the sole live-looking remnant is the round-3
    definition list (N5-1).
  - Pairwise composition across the full set the brief named — (b)
    sigless-only × uniform gate; gate-helper-two-sites × ONLY-admission-point
    × no-arm-routes-around (arms 1-2 at site 1, arm 3 at site 2: covered);
    `restore`-excluded × never-silent (restore has no `--allow`, so no silent
    flag exists there); `:524`-lenient × flagless-refusal-on-tr baseline (the
    RCW's tr carries the sigless leaf, so the per-leaf gate fires and the
    baseline holds post-leniency); `:524`-lenient × export-with-flag-tr
    (satisfiable per R4's emitter check); five-value vocabulary × grid row 2
    label; grid acceptance bullet (6 cells + row-2 column identity) × the
    R4-1 bullet; parenthetical-never-printed × both did-not-fire cells (the
    rule ran on every arm under the uniform gate, so the parenthetical is
    true wherever printed). **All compose.** The two places composition
    strains are exactly M5-1 and M5-2 below.

## Ledger — the three round-4 findings

| ID | answered? |
| --- | --- |
| R4-1 | **Yes, fully.** The ungated-path note is deleted rather than reworded; the acceptance bullet is R4's own sentence; the grid has no ungated cell and says why. Residue is only the un-struck definition site in the round-3 narrative (N5-1). |
| R4-2 | **Yes, fully.** Locus replaced by mechanism against real code; restore's two constructors named and excluded; `:524` leniency stated as ruling and acceptance; parse-site enumeration extended to all three arms with the arm annotation. Residue: the leniency's own second behaviour change is not named in the release-note bullet (M5-2). |
| R4-3 | **Yes in substance.** The bullet is wsh-scoped and the unsatisfiable tr half is gone; the parenthetical explains why. The transcription compressed "remain categorically refused by Fix-α regardless of `--allow`" to "tr refuses regardless of the flag" — mechanism name and preservation phrasing dropped (N5-2), substance intact. |

---

## M5-1 (Minor, GRID) — "The columns are identical by construction" is falsified by the grid's own row 1, and its edit-check is unsound as stated

**Text attacked:** grid reading-bullet 1 — *"**The columns are identical by
construction.** That is the point of topology (B) and the check on it: if any
future edit makes a column differ, the uniform gate has been broken
somewhere."* — and the section header's *"the arm dimension is deliberately
degenerate"*.

**Why it is wrong.** One line above the bullet, the columns are not
identical: row 1's `--template`/`--slot` cell is "did-not-fire note" while
the other two columns are "fired warning, **or** did-not-fire note". That
difference is *correct* — the fired case is unconstructible on builder output
(the settled invariant) — so the columns differ today with the uniform gate
intact, and the bullet's contrapositive check ("column differs ⇒ gate
broken") reports a phantom break against the truthful grid it sits under. A
second leg: even columns 1 and 2 are identical only at the note-wording
level — for a tr input with the flag, the `--descriptor` arm exports with the
fired warning (post-`:524`-leniency) while the import arm categorically
refuses (the R4-3 parenthetical's own fact) — so "identical" is true of
wordings, which is exactly what R4 ruled ("Same wordings as `--descriptor`"),
and not of columns. The document's own acceptance already concedes all of
this: the new test bullet scopes column identity to **row 2 only** ("a test
that the three columns are identical for row 2"). So the grid section makes a
claim the rulings never made, the table one line up falsifies, and the
acceptance quietly narrows. This is the R3-2/R4-1 defect *family* (a
confident sentence false beside a true ruling) but not their severity: it is
plan commentary, not a printed note; no acceptance bullet mandates anything
false; an implementer who operationalises it gets a red test on a correct
implementation (fail-closed, machine-caught), not a false green. Hence Minor.

**Fix (reproduce, not assert):** scope the bullet to what the table shows —
the *wordings* are arm-independent and row 2's columns are identical; row 1's
template/slot cell differs by reachability alone (the builder invariant, not
a gate difference). The sound edit-checks are: row 2's columns ever differ ⇒
uniform gate broken; row 1's descriptor/import cells ever differ ⇒ uniform
gate broken; template/slot ever shows the fired warning ⇒ builder invariant
broken. Drop or qualify "deliberately degenerate" the same way.

## M5-2 (Minor, NEW — re: the R4-2 fold) — the `:524` leniency is a second operator-visible behaviour change, and the release-note bullet stayed singular

**Text attacked:** acceptance — *"**A release-note line for the behaviour
change.**"* — beside the new bullet *"**The `--descriptor` intake parse at
`:524` becomes lenient**, so a tr form reaches the gate at all."*

**Why it is wrong.** Per R4's machine-checked ledger (the premise of its own
remedy: leniency is needed "so a tr form can reach the gate **at all**", and
keeping it strict "fails the export-with-flag baseline"), no tr form survives
`:524` today — `export-wallet --descriptor <tr>` refuses at intake regardless
of content. After the fold's ruling, a **non-sigless** tr descriptor passes
the lenient intake, violates no enforced rule (uniform enforcement is
`sigless-branch` only, by the settled (b) ruling), and exports at exit 0 —
the emission side carries no taproot refusal (R4's ledger). So Phase 1 ships
two distinct operator-visible changes on this surface: (1) the headline —
sigless wsh flagless-export becomes refuse-or-waive; (2) tr descriptors that
were categorically refused at intake become admitted. The release-note bullet
names "the behaviour change", singular, and was written for (1); nothing in
the plan names (2) as release-notable, on the surface whose constraint
section explicitly polices release-note wording. Both implementers ship the
same product either way — the defect is documentation completeness, hence
Minor, not a topology or product-divergence defect.

**Fix:** pluralise the bullet and name both lines: the wsh-hole closure
(refusal + waiver) and the tr intake widening at `:524` (previously refused
forms now admitted, gated by `sigless-branch` alone).

## N5-1 (Nit, re: R4-1) — the deleted ungated-path wording still reads as live at its definition site

The round-3 section still says *"Two export-side wordings, hung on the
existing export-wording acceptance bullet:"* and lists **ungated path** with
its full quoted sentence, un-struck. Supersession is explicit and named
downstream (the R4-1 narrative quotes the sentence and rules it false; grid
bullet 4 says "R4-1 deleted it"), which matches this document's chronological-
ledger convention — but this is the one site where a dead wording still
carries normative framing ("hung on the … acceptance bullet"). Fix: mark the
list item deleted at the definition site (e.g. "~~ungated path~~ — DELETED by
R4-1: under (B) there is no ungated path").

## N5-2 (Nit, re: R4-3) — the transcription dropped "remain … by Fix-α"

R4-3's prescribed clause was *"taproot envelopes remain categorically refused
by Fix-α regardless of `--allow`"*; the folded parenthetical says *"tr
refuses regardless of the flag"*. The wsh scoping — the actual fix — landed
fully, and the compressed clause is true. What was lost: the mechanism name
(Fix-α, which R4-3 named precisely so nobody "fixes" a red tr test by
relaxing it) and "remain", the preservation commitment. Combined with
"asserted to be the ONLY admission point", an aggressive uniformity reading
could treat the import arm's categorical tr refusal as a competing admission
point to remove; the enumeration bullet's shape (sites *enumerated and
annotated*, with only `:524` mandated lenient) already contradicts that
reading, so this stays a Nit. Fix: restore the two dropped words — "tr
envelopes **remain** categorically refused **by Fix-α** regardless of the
flag" — and, if wanted at zero cost, assert it (a flagged-tr-envelope refusal
test) so "regardless of the flag" is a test rather than a parenthetical.

---

**Gate: GREEN — 0 Critical / 0 Important / 2 Minor / 2 Nit.** The round-4
fold did the thing rounds 3 and 4 each failed to do one round earlier: it
adopted the reviewer's remedies *and* checked them against the topology they
sit beside — verbatim where the wording was load-bearing, subtractive where
the fix was a deletion. The grid's six cells are all true and three of its
four reading-bullets are faithful; the one that is not (M5-1) overstates a
claim the acceptance already scopes correctly, so no implementer builds from
it. Nothing here blocks: record the two Minors and two Nits with an owning
phase (all four are one-sentence edits to the plan text, natural to fold into
the Phase-1 PR's plan-sync commit) and **close the loop**. Per the standing
closure rule this is lens-closure: the questions this round was asked — did
the fold answer, is the grid true, does the set compose — have no more
answers. Phases 2-5, the wallet, and the journeys were not reviewed, by
brief.
