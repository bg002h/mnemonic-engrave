# R0 REVIEW — IMPLEMENTATION_PLAN_s6b.md (scheduling review)

**Reviewer:** independent pass (opus-tier), scoped to ORDER and OWNERSHIP only,
per the dispatch brief. Design not re-opened. Source checked against fork
`bg002h/seedhammer` `main` = `b1479a1b38f6b045d27443764c858906e4e6e122` (clean,
`git rev-parse HEAD` confirmed at review time).

**Scope note.** Gate *coverage* (27 spec gates / 27 scheduled) was already
machine-checked by the controller and is not redone here. This review spends
its budget on (1) whether the plan's three claimed dependencies are real and
complete, and (2) whether every spec requirement/prohibition — not just every
numbered gate — has a phase owner that can actually satisfy it when its turn
comes.

---

## 1. Dependency claims

| # | claim | verdict | evidence |
| --- | --- | --- | --- |
| 1 | **§1.1 before §1.2/§1.3** (`Title`/`Footer` must exist on `backup.Text` before anything conditions them) | **SOUND** | `backup/backup.go:32-40` — `Text` today carries only `Paragraphs`, `Font`, `FontSize`. No `Title`/`Footer` field exists yet, confirming the premise; §1.2/§1.3's conditioning genuinely has nothing to condition until §1.1 lands. Fully internal to P2, no cross-phase exposure. |
| 2 | **P3 before P4** (§6's condition reads a result only P3's flow returns; `engravePassphraseFlowFrom` returns nothing today) | **SOUND** | `gui/passphrase_flow.go:617` — `func engravePassphraseFlowFrom(ctx *Context, th *Colors, body []byte, src syswSource) {` has no return type, confirmed. The offer-insertion site itself (spec §2.6, "between the verify offer and `restoreDocFlow`") sits inside spec §2, which the plan's own phase table maps wholly to P3 — so P3, not P4, is unambiguously the phase that inserts the call and captures its boolean. P4 (§6/GATE 6a) then consumes that value at the `restoreDocFlow` call. If attempted out of order, P4's GATE 6a test cannot even compile/pass — the dependency is self-enforcing, not silent. |
| 3 | **P1 before P6** (R-M's body must be in the tree before the sweep) | **SOUND, but not a distinguishing constraint** — see Minor finding M1 below. `gui/multisig_verify.go:882` shows `multisigVerifyNoSlotBody` is rendered via `showError(...)`, the same modal class `TestModalsThisBlockTouchesAreDrawnInFull` (`gui/modal_fits_test.go:297`) already sweeps by table entry — so it genuinely is the kind of body GATE 4 must cover. |
| — | **P5 order-free** (R-I: arrows cost no body width, so §4 stays valid whenever arrows land) | **SOUND** | `gui/gui.go:403-416` — `bodyClip`/`maxScroll`/`scrollFadeDist` are pure geometry, untouched by any modal *content* string; plan §3 makes "must not change body width from 417" an explicit P5 prohibition, which is exactly what keeps GATE 4's measurements valid regardless of P5's position. Spec §5.3 also states explicitly that GATE 4's op-tree comparison *cannot* see the arrow/glyph overlap problem — that's why GATE 5.3 exists as a separate, P5-owned pixel check rather than folding into P6. No missed coupling. |

**No missed dependency found.** I specifically checked for a hidden coupling
between P2 and P3 (both touch `gui/singlesig.go`, at line-adjacent but
disjoint regions: P2's marking call is upstream of the verify offer, P3's new
offer sits between the existing verify offer and `restoreDocFlow` — confirmed
by reading the current file, no overlapping edit region), between P2 and P4
(disjoint: `bundlePlate`'s new `kind` field vs. GATE 6.1's deliberate
non-participation in `bundlePlatePlan`), and for golden-file collisions across
phases (`text-*-shards-1.bin` / `passphrase-*.bin` / `sizeproof-*.bin` are
disjoint mechanisms — `backup.Text` vs `backup.Passphrase` vs size-proof —
so no phase's golden churn is order-sensitive relative to another's).

`bundleEngrave`'s four callers were counted directly against source and match
the plan's "touches four callers" claim exactly:
`gui/bundle_flow.go:39`, `gui/multisig_build.go:402`, `gui/multisig.go:291`,
`gui/singlesig.go:177` (only the last passes non-empty values). This confirms
P2's blast radius as scheduled.

---

## 2. Unowned or mis-owned requirements/prohibitions

Checked every spec sentence that reads as normative but is not itself a
numbered gate, against the plan's phase→spec mapping (§1) and its per-phase
"must not" list (§3). None were found unowned:

- **Spec §2.1.3**, *"No package-level variable and no field on `Context`"*
  (for the preloaded passphrase) — **not** in the plan's P3 "must not" list,
  but P3 owns spec §2 wholesale per the plan's own phase table, and the plan
  states up front it "does not restate... the spec" (line 9) — the §3 lists
  are explicitly a non-exhaustive "boundaries a reasonable implementer would
  otherwise cross" supplement, not a checklist substituting for the spec.
  Owned, just not called out. Not a finding — see Minor M2 for a cheap
  strengthening suggestion.
- **Spec R-K's boundary**, *"a change that touches sealed-payload secret
  handling is out of scope for R-K"* — trivially respected: no phase's file
  set (`gui/singlesig.go`, `gui/passphrase_flow.go`, `gui/multisig_verify.go`,
  `gui/gui.go`, `backup/*.go`) touches sealed-payload code at all. No owner
  needed.
- **`gui/unlock_platelist.go:222`**, listed "no" in spec §1.3's call-path
  table but **absent from the plan's P2 "must not" bullet** — checked and
  confirmed **not a gap**: that call site invokes `validateMdmk` **directly**
  (`gui/unlock_platelist.go:222`), not through `bundleEngrave`. Since P2's
  mechanism only grows `bundleEngrave`'s signature (spec §1.3's "grows two
  string parameters"), this site is structurally outside the marking
  mechanism's reach — it cannot be marked regardless of any prohibition, so
  none was needed. (If `validateMdmk`'s own signature changes too, this site
  needs a trivial `"", ""` compile-fix, which is not "marking" and is
  self-enforcing via the compiler, not something an implementer could get
  wrong silently.)
- **The stale doc comment** (`gui/singlesig_derive.go:28`, spec §8) — plan §5
  correctly defers only the *commit-within-P3* granularity, not the phase; the
  spec's "fix it in the same commit [as §2.4]" pins it to P3 regardless.
  Correctly out of the plan's scope to re-decide.
- **Spec §1.3's prohibitions** ("a variadic tail is prohibited," "shared state
  on `Context`... is prohibited") — both explicitly present, verbatim in
  effect, in the plan's P2 "must not" list. Owned.

**Conclusion: no requirement or prohibition found with no owner, or with the
wrong owner.**

---

## 3. Phases that could be merged or dropped

None. Checked three candidates the KEEP-IT-SIMPLE framing invites:

- **P2 + P3** (both "plate marking"-flavored) — rejected: their golden-churn
  contracts are opposite by design (P2 asserts **zero** churn; P3 **expects**
  four files to move and re-records them in-commit). Merging would blur
  exactly the assertion GATE 1.1 exists to make crisp — R-G's "a moved byte is
  a finding" only works if nothing else in the same commit is expected to
  move bytes.
- **P6 folded per-phase instead of once at the end** — rejected: GATE 4's own
  mechanism (`TestModalsThisBlockTouchesAreDrawnInFull`, a single growing
  table) is naturally a one-shot sweep; R-I's decoupling is what makes running
  it exactly once, last, both correct and cheaper than re-sweeping after every
  phase. The plan already found the minimal shape here.
- **P5 dropped/deferred** — not available: F-208 is explicitly in the spec's
  §0 "In" scope, alongside F-199/F-204/F-206/F-192 and the plate marking.

**The six-phase split is already minimal for this scope.**

---

## 4. Findings

**M1 (Minor).** The "P1 before P6" dependency, while factually correct (§1
verified above), is not actually a distinguishing ordering constraint: P6 is
scheduled last in every case, so it trivially follows P1 (and P2–P5) by
construction regardless of which phase's content it needs to see. Listing it
alongside the two *genuinely* order-determining dependencies (§1.1→§1.2/§1.3
within P2; P3→P4) slightly overstates its selectivity. **Smallest fix:**
reword the "three real dependencies" framing to distinguish "P6 is last
because it must see everything" from the two intra/inter-phase couplings that
actually force a specific relative order — or simply note that P6's position
is fixed by its own job description, not by a dependency edge. Does not gate;
optional wording-only fold.

**M2 (Minor).** Spec §2.1.3's "no package-level variable, no `Context` field"
prohibition for the preloaded passphrase is not echoed in plan §3's P3 "must
not" list, unlike its P2 sibling (variadic tail / shared `Context` state,
which *is* echoed). Both are the same class of hazard (a secret-adjacent value
escaping its flow) and P2's list already sets the precedent of calling it out
explicitly. **Smallest fix:** add one clause to P3's "must not" bullet:
"...must not stash the preloaded passphrase, seed FP, or combined FP on a
package-level variable or `Context` field." One line; does not change scope,
only brings P3's explicit list to parity with P2's.

Neither finding blocks implementation or risks a redo/silent gap — both are
optional strengthenings of an already-owned area.

---

## Verdict

**GREEN 0C/0I**
