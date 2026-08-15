# Design judgment — did "remove all judgment" cause the failures, and what replaces it?

Reviewer: fable (independent design judgment, 2026-08-15). Scope per brief:
philosophy and consequences only; no code audit, no re-review of S0b, no new
gates. Every plan/spec citation below was read in the current files; two facts
were machine-checked fresh for this report and are marked as such.

---

## 1. VERDICT

**Judgment is required, and this cycle is the controlled experiment that proves
it.** The plan's attempt to specify judgment away did not remove it — it
relocated it to places where the product goal was not in the room: twice to
operator escalations (settled fact 3), once to an unreviewed unilateral call in
an upstream release (the `bip48` refusal), and once to a queue (F-175, three
options, none chosen). The mechanism is measurable: the operator's own criterion
— "permissive on input, expressive on output, speaking loudly when common
assumptions must be made" — **appears nowhere in the 1101-line plan or the
842-line spec** (`grep -in "permissive\|expressive"` over both, exit 1,
machine-checked for this report). The plan removed judgment by removing the
criterion judgment would need, so when choices surfaced anyway they were decided
by the nearest house rule — one-armed tests, fail-closed, refuse-by-name —
instead of by the goal. What CAN be specified away, and what this plan
specified away brilliantly, is **trust in evidence**: oracles pinned by commit,
mechanisms seen to fail, single-site needles. What cannot is **design choice
under information that only execution produces** — and the two upstream
releases ("neither knowable from the plan", settled fact 2) are the proof that
no thickness of specification substitutes for running the thing. The
replacement is not "more judgment everywhere"; it is: machinery for evidence,
judgment for design, the product goal written into the artifact as the named
tiebreaker, and an explicit boundary (section 4) that says where permissiveness
must stop.

---

## 2. WHAT THE PLAN GOT WRONG

Ranked by cost to the operator.

### W1 — It pre-ruled a refusal that made the product goal unreachable (F-173)

**What it did.** The spec offered two arms: "more matching cards than open
slots gets a selection step or a named refusal" (SPEC P0 item 4, line 647). The
plan picked the refusal and wrote the test to pin it:
`TestBuildRefusesMoreCardsThanOpenSlots — named refusal, not a fall-through`
(plan line 598-599, original wording preserved in the re-scope note). Combined
with S1's specified unconditional feed, the delivered payload's four cards made
the build "refuse for every n except 5" (F-173, FOLLOWUPS.md:5854-5855) — S2's
gate unsatisfiable, Trace A, the plan's own flagship acceptance journey,
unreachable.

**What it cost.** An operator escalation mid-cycle, a re-scope of three tests
and a standing ruling folded into §1 — and the tell: the ruling that came back
("Available key count could be 0 to n", plan lines 56-62) was **wider and more
permissive than either option escalated**. The product goal, which the plan
never carried, decided in five words what the specification machinery had spent
a round converting into two narrow workarounds.

**What should have been done.** When a spec offers refuse-or-permit and the
arms differ in what the user can do, the goal picks the arm; test shape picks
only the phrasing. The plan chose refusal because a named refusal makes a
clean one-armed test — the same reasoning it states openly at S2 test 3 ("this
plan picks REFUSE, so the test's name matches its body and the assertion has
one arm", line 668-669). Testability is a real value; it is never a reason to
choose *which behavior the product has*.

### W2 — It wrote five acceptance gates on a mechanism that did not exist

**What it did.** "Every stage's gate includes the §4.5 emulator walk" (plan
line 212), written for S1-S5 while no walk could reach the flow, the census was
a literal, and nothing invoked the pinned oracles. The plan's own blind-spot
section now concedes it: "every 'by test and by emulator walk' clause from S1
on is a hypothesis until that stage writes its own walk" (line 1089), and its
own S0 preamble concedes the cause: "Round-0-through-3 of this plan named S0's
deliverables without opening any of them" (lines 241-242).

**What it cost.** S0b — an unplanned stage (settled fact 1), two primary-repo
releases (settled fact 2), and most of the cycle to date, with zero user-visible
capability shipped. The connection to the question under review: exhaustive
written acceptance *felt like* judgment removed from "is this stage done," but a
specified gate is a hypothesis until executed (settled fact 5, the repo's own
lesson), and the sheer confidence of the prose is what deferred the run. A
200-line plan with one executed walk would have found F-168-F-172 on day one; an
1100-line plan with none found them in review round after review round had
already been paid for.

**What should have been done.** Execute one gate before writing five on it —
which is what S0b retroactively is. The residue is F-175: the record machinery
built to remove judgment from stage closure cannot represent the plan's own S1
(FOLLOWUPS.md:5947-5951). Ruled in section 5.

### W3 — It ruled a fact: the mk1 "property of the format" that was a bug

**What it did.** §1a's comparison table excludes the mk1 chunk-set id because
"the primary randomizes it by design; this is a ruled property of the format,
not a test-time convenience" (line 110), and disposes of the fix: "A
`--chunk-set-id` flag on `mk encode` would restore full mk1 string identity.
File it, do not build it" (lines 113-114).

**What it cost.** Execution proved the randomization violated mk SPEC §2.5 and
was fixed as a conformance bug (mk-codec 0.5.0, settled fact 2; continuity
lines 65-79). The "designed" property was a defect; the filed flag was the
wrong remedy for a mis-modeled cause; the plan's two-part mk1 relation was a
comparison weakened to accommodate a bug. The real blocker still cost a
mid-stage upstream release — the ruling bought nothing and asserted something
false in a ruling's typography.

**What should have been done.** A plan may rule the *relation it will accept*;
a claim about *another system's design intent* is a fact, not a ruling, and
gets the plan's own gate-coverage treatment (§1a: "5 of 22 ungated facts were
false" — this was a sixth, wearing bold italics). The distinction is
mechanical: if a sentence could be falsified by reading someone else's spec or
running someone else's binary, it may not be ruled, only cited and checked.

### W4 — It legislates per-failure, and the law is displacing the product

**What it did.** Every discovered defect adds inline law to the artifact: "Do
not edit this table in place next time either. Re-derive it" (line 517-518);
the D4 rescope essay (lines 300-330); the S3 gate-scope essay (lines 753-773);
the F-164 vocabulary-drift essay (lines 477-491). The plan is 1101 lines; the
actual S1-S5 implementation deltas are a small fraction of them.

**What it cost.** Cycle time and attention, paid by the operator. Every future
implementer reads the archaeology to find the work. F-175 shows the compliance
machinery now generating its own open decisions — an artifact-free stage cannot
satisfy the record system built to verify it, and resolving *that* is queued as
operator judgment. Meanwhile the user still cannot do the thing: the product
goal sits at the end of a queue behind the plan's self-defense.

**What should have been done.** Evidence rules compound and should accrete
(they are section 3's success). Design pre-rulings and post-mortem essays do
not: corrections belong in FOLLOWUPS and continuity, and the plan should be
read per-stage, not re-earned per-read. Freeze recommended in section 5.

*(A fifth candidate — the `bip48` refusal — is a symptom of W1's pattern
exported upstream; it is ruled as the worked example in section 4.)*

---

## 3. WHAT THE PLAN GOT RIGHT

Honest, and substantial — the judgment-removal effort bought real things in the
one domain where removal is possible:

- **The evidence machinery works, and it paid twice in one week.** The
  byte-identity criterion plus the Rust-first adjudication rule (plan lines
  178-181) routed two genuine upstream defects — mk nondeterminism, the missing
  seed-to-multisig-xpub oracle — to fixes in the primary repos with test
  vectors (settled fact 2). Both were invisible to every review round; only an
  unbendable byte gate could force them out. "Remove judgment from whether
  evidence counts" is the correct instinct, fully vindicated.
- **The seen-to-fail discipline is the best rule in the document.** "A
  mechanism that has not failed here does not leave this stage" (S0b gate,
  lines 570-571). Cheap, mechanical, and it converts false-PASS from a review
  finding into an impossibility.
- **Jurisdiction rulings are judgment-removal done right.** The Adjudication
  paragraph (who settles a divergence), spec R-3 (whose origin wins), and the
  cardinality ruling's stated reason — "leaving the choice to the stage would
  have left §4.5's 'every mk1' undefined, and the spec is the document that
  rules" (SPEC §4.1a item 2). Specifying *who decides* a class of question is
  durable; specifying *the answer* before execution is where W1 and W3 came from.
- **One design pre-ruling in the plan is exactly correct and shows the
  standard:** S5's assembly-and-tail-are-one-stage constraint (lines 33-36).
  Both arms were analyzed, one ships C2 on steel, the asymmetry is stated. And
  S5 test 5 (lines 910-914) picks the arm that is simultaneously more testable
  and better for the user — when those coincide, pre-ruling is free. Contrast
  S2 test 3, where testability alone decided.
- **The blind-spot section (§5) and the escalations themselves.** The plan
  states what its gates cannot see, and when reality outran the specification
  it escalated rather than plowing on — twice, with both rulings recorded and
  greppable. A judgment-heavy plan would have made those calls silently and
  divergently.

---

## 4. THE PERMISSIVENESS RULE

**"Defaults for spelling, never for stakes — and every default is printed."**

When an input underdetermines the output, run four clauses in order:

1. **Authority.** Is there a default the governing standard states, or a
   convention effectively universal in deployed practice? If neither: REFUSE,
   listing the explicit forms. A tool never invents a default — permissiveness
   is applying someone else's decision, not making one.
2. **Auditability — the funds-safety boundary, and it is already in the spec.**
   Trace where the assumption lands. If it is printed, engraved, or displayed
   in artifacts the operator keeps — paths, script types, plate counts — a
   wrong assumption is detectable by reading the output, so it is eligible to
   assume. If a wrong assumption would be **invisible in every artifact**, the
   boundary bites: SPEC §4.1 states it verbatim for the per-seed passphrase
   ("a wrong binding is invisible in every engraved artifact", lines 289-291)
   and correctly makes it REFUSE-shaped. Generalized: **permissiveness stops
   exactly where a wrong assumption could not be detected by reading everything
   the device outputs.** Also on the refuse side of this line, permanently:
   duplicate keys in the final slot set (quorum degradation an operator cannot
   see — announcement cannot cure harm that accrues to an attacker, SPEC §4.1)
   and a failed `both`-slot derivation (proceeding loudly would engrave a claim
   the device knows is false, SPEC §4.3).
3. **Reversibility.** Upstream of an irreversible act (steel, OTP, broadcast),
   the assumption must be announced on the confirmation surface itself, not in
   scrollback. If the flow cannot announce at the decision point: refuse.
4. **If all three pass: accept, apply the authority's default, and announce
   unmissably** — the assumption, its source, the result, and the explicit
   spelling that overrides it.

**Corollary that decides the plan's two-armed cases:** when the spec offers
refuse-or-permit and clause 2 does not bite, the permissive arm is the
product's arm. Test shape may then choose the phrasing — a one-armed assertion
*of the permissive behavior* — but never the arm.

### Worked example — the `bip48` ruling

The refusal's recorded rationale: "two script types are registered; guessing
would put a cosigner key at a path nobody chose" (CONTINUITY_2026-08-14g.md:
86-88). Run the clauses:

1. **Authority: yes, verbatim.** BIP-48: *"The recommended default for wallets
   is pay to witness script hash `m/48'/0'/0'/2'`."* (Fetched from
   bitcoin/bips master for this report.) Applying a standard's stated default
   is not guessing — the rationale is wrong on its own terms. And the
   operator's goal statement names *this exact case* — "assuming BIP-assigned
   origin paths" — as the archetype of an assumption to make loudly.
2. **Auditability: maximal.** The script type and full path are printed with
   the derived xpub, carried in the mk1's origin path, and shown on the restore
   doc; a wrong default additionally fails loudly at coordination time, before
   funds exist.
3. **Reversibility:** `ms derive` is host-side, upstream of nothing
   irreversible.

**Ruling: the refusal is the wrong call.** Bare `bip48` should derive p2wsh at
script_type `2'`, announce "assumed BIP-48 recommended default: p2wsh
(m/48'/coin'/acct'/2'); use bip48-p2sh-p2wsh for nested" in the output itself,
and keep both explicit forms. Mitigation credit: the refusal lists both
spellings, so recovery is one retry — the cheap end of the wrong pattern. The
pattern is what matters: the call was made mid-stage, upstream, with no rule in
hand, and it defaulted to the plan's house posture (refuse, one-armed) because
the product goal was written down nowhere it could be consulted. That is
precisely how "remove all judgment" degrades a product — judgment happens
anyway, minus the criterion.

---

## 5. WHAT TO CHANGE NOW (S1 onward)

**Change the plan:**

1. **Paste the operator's goal sentence into plan §0 verbatim and name it the
   tiebreaker** for every refuse-vs-permit choice from S1 on. One edit;
   grep-verified absent today. This is the highest-leverage line in this report.
2. **Rule F-175 before S1 code, and rule it (b) — recordless with the
   substitute NAMED.** The plan already half-answers it: "S1 ends at a screen…
   the census is inert, so that walk asserts on `shScreen()` text at a named
   screen instead" (lines 217-219). The gap is only that the record machinery
   cannot represent that. Note also that S1's gate has an engrave arm ("either
   the flow completes an engrave, or D-1 reproduces", lines 651-652) — so
   F-175 bites only the D-1 arm. Edit: on that arm, S1 is recordless and its
   evidence is the committed walk script, the single-site needle, the
   `shNFC.present == 0` assertion, and the captured failing test — stated in
   the plan so the stage cannot pass with neither record nor substitute.
   Option (a)'s schema bump waits for a second artifact-free stage; option (c)
   re-couples S1 to S2 and undoes the plan's own staging. Reconcile the
   preamble/gate tension in the same edit.
3. **Freeze the plan's growth.** From S1 on, discoveries go to FOLLOWUPS and
   continuity; the plan is edited only when a gate or ruling changes. The
   implementer's read is the stage section plus the standing rules, not the
   archaeology.
4. **One-pass sweep of S1-S5's named refusals against the rule** (a read, not a
   rewrite): the `0..n` rework already fixed the largest; S1 test 7's
   "name the host route" is the rule's clause 4 done right — keep; S2 test 3's
   interim REFUSE stands (the alternative was *silent* origin-stamping, which
   clause 2 forbids), but its recorded reason should be the exposure, not the
   test shape; S4's FAIL screens are on the correct side of the boundary — keep.

**Change the product:**

5. **Reverse the bare-`bip48` refusal in ms-cli's next release** per the worked
   example: accept, derive p2wsh, announce the assumption and its BIP-48 source
   in the output, keep both explicit forms. Rust-first with a test vector per
   the standing rule — and the test asserts the *announcement*, not just the
   derivation, because the announcement is the load-bearing half.
6. **Recognize S5's review-screen work (lines 977-983) as goal
   implementation, not just defect repair.** Showing the per-slot keys and
   rewriting the EXPERIMENTAL warning is the "expressive on output" half of the
   operator's sentence, already in the plan under another name. Treat it as
   non-negotiable scope, not polish.

**Keep unchanged:**

7. **All evidence machinery, at full strength.** Oracle-by-commit,
   seen-to-fail mutations, single-site needles, fail-closed records (modulo
   item 2). This is where the judgment-removal instinct is correct, and it paid
   for itself twice this week.
