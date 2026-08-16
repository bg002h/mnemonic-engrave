# S6a blind-spot pass 2 — is P4 dischargeable by a hand-built table?

**Question:** the design's central property (P4) requires a COMPLETE enumeration
of observations; hand-enumeration has failed three times. Is the property
enforceable at all — and if not, what replaces it?

**Inputs read:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md` §4.7
(§4.7a–§4.7f, properties, tables), `design/agent-reports/s6a-r7-adversarial.md`
in full. No fresh code audit, per the brief.

---

## IS P4 ENFORCEABLE

**No — not by a hand-built table — and the three failed rounds are the structure
of the obligation, not bad luck.** P4 quantifies over "every reachable outcome";
the table discharges it only if three conjuncts all hold: the enumeration is
complete, every row's W is correctly sized, and the code can make every
distinction the rows draw. Round 7 falsified each conjunct independently — C-2
(incomplete, and completeness had only ever been checked against paths reviewers
named), C-1 (a row *present* whose W was mis-sized, on the success side no
adverse-path analysis ever visits), C-3 (a distinction the table draws that the
runtime cannot). Worse than incompleteness is the table's failure *direction*:
it is a normative document sitting beside the code, and when it is wrong it
fails open — r7 measured that an observation with no row "silently classifies as
neither sticky fact" and inherits `DID NOT COMPLETE`, while the unexamined
success row inherited full-strength `VERIFIED`. An artifact that (a) requires an
unverifiable completeness and (b) *strengthens* the printed claim wherever it is
incomplete cannot be discharged by review; each round can only shrink the
residue somebody happened to name. One part was always mechanical, though: the
r7 reviewer produced the complete return-site sweep with `sed` in a single pass.
Enumerating the rows was never the hard part — *hand-maintaining* them was, and
per the project's own rule that is a tool's job. **Ruling: P4 survives as the
correctness criterion (r7 re-affirmed it after applying it further than the fold
did); what is unenforceable is its discharge method. Keep the property, replace
the mechanism.**

## WHAT REPLACES OR REPAIRS IT

**Invert the direction of derivation: the line is CONSTRUCTED from the evidence,
never selected by the outcome.** The verify flow records, at each site where it
learns something, an observation value carrying its provenance (plate-read vs
operator-typed) and its result — the same facts that site's own screen already
renders. The document's status and lines are then a total function of that
record: pass lines are assembled from, and scoped to, the positive entries, so a
sentence claiming a read that has no entry cannot be constructed (C-1 becomes
unwritable); adverse entries route on their *recorded* provenance, so nothing is
ever reconstructed downstream from an untyped error (C-3's gap dissolves — the
comparator knows which leg and which provenance while they are still in scope);
and an entry the mapping does not recognize takes the maximally hedged line, so
any future incompleteness *weakens* the document instead of strengthening it,
with a test asserting that default arm is unreachable today (C-2's class becomes
a red test instead of a reviewer find). The observation table survives with its
role changed: from normative source-of-truth to a **review projection checked
mechanically against the return-site sweep** — commit the sweep as a script,
per the plan-build-gate pattern, so row coverage is a command. Only the W column
remains human judgment, and a wrong W now mis-hedges rather than over-claims.
This also resolves the scope-line condition the way r7 required: expressed
against the class ("any adverse entry in the record"), not against two status
names, so adding a row can never leave a page unscoped.

## THE PROPERTY

**P5 — the line is constructed from recorded evidence, and enumeration failures
may only ever weaken it.** Three clauses, each independently testable:

- **(a) Positive claims are generated.** Every printed claim that a check
  occurred is generated from a recorded observation of that check, carrying that
  observation's provenance; a claim with no generating observation cannot be
  rendered. *(Catches C-1: no read-observation exists for the ms1 plate, so
  "each plate was read back" is not constructible.)*
- **(b) Classification at the point of knowledge.** Every distinction the
  status map draws is made at the code site where its distinguishing facts are
  values in scope, and recorded there — never reconstructed downstream from a
  verdict, an error value, or an error string. *(Catches C-3 and I-2: mk1-vs-ms1
  provenance exists inside the comparator and nowhere after it; a per-exit
  mapping is downstream reconstruction by definition.)*
- **(c) Monotone under omission.** An observation the mapping does not
  recognize maps to the line that makes the fewest claims — one true in every
  world — and a test asserts that default arm is unreachable from every known
  return path. Incompleteness of enumeration may only weaken the printed line,
  never strengthen it. *(Catches C-2: the unnamed `:701` and `:738` paths would
  have printed the hedged line, and the unreachability test would have flagged
  them the day they were written.)*

Under P5, **P4 becomes a theorem rather than an obligation**: a line constructed
only from recorded observations, with unrecognized evidence maximally hedged,
cannot assert a single member of a multi-world W. The completeness P4 needed is
split into the part a tool enforces (row coverage = the return-site sweep;
classifier exhaustiveness = the tested default arm) and the part a human still
judges (W per row), whose failure direction is now safe.

## THE STATUS SET

**Six survives as the reachable set; three things move within it.**
(1) The two pass lines reword to *scoped* form — "the key and descriptor plates
were read back and matched; the ms1 you typed matched this seed — no ms1 plate
was read" — mirroring `multisigVerifyOKMessage`. A fixed string per flow
remains legitimate because the checked-set is identical on every clean pass of
that flow; §4.7d row 1 loses "every plate". Scoping, not `statusUnaccounted`,
is the correct P4 discharge here (r7 M-5 is right: ambiguity is one way to be
true in every world; scope is the other, and the stronger one when the device
knows *which* thing it did not look at).
(2) Composition moves per C-2: `:701`, `:738`, and single-sig `:116` route to
`statusUnaccounted`; `:979` gets its benign `DID NOT COMPLETE` row; "incomplete"
leaves §4.7d row 4's enumeration.
(3) One **reserved default line** is added as P5(c)'s enforcement — maximally
hedged ("the device observed something it could not classify; treat these
plates as unchecked and confirm they restore before relying on this backup") —
but it is *scaffolding, not a seventh knowledge state*: a test asserts no
reachable path exercises it. `statusUnaccounted`'s line cannot serve as the
default, because it asserts two specific readings an arbitrary unrecognized
observation does not license. The claim "six, one per knowledge state — the
number is DERIVED" survives with its derivation now structural: the statuses are
the equivalence classes of the evidence record, plus one reserved arm whose
emptiness is continuously tested.

## WHY THE CODEBASE KEEPS BEING AHEAD

**Because the screens were authored under P5(b) and the plan was not — same
method failure, and the replacement fixes it structurally.** Every prior-art
answer the plan failed to adopt was written by someone standing at a return
site with the distinguishing facts in hand and one specific operator to
instruct; at that vantage you cannot avoid scope ("the ms1 **you typed**"),
ambiguity ("Either that plate was not presented, or…"), or declared limits ("IT
DOES NOT CLAIM THE SEED PLATES WERE COUNTED"). The plan authors the document
centrally, downstream of every site, summarizing over a table — precisely the
reconstruct-at-a-distance posture P5(b) forbids — so it keeps losing knowledge
the sites already hold and paying a review round to buy it back. Once screen
text and document line are two renderings of the *same recorded observation*,
the document inherits the screens' hard-won scoping by construction, and
"check each site's existing screen and doc comment before writing its row"
stops being review luck and becomes a mechanical part of producing the record.

## WHAT I AM ASSUMING

- The settled constraints hold and are compatible: the document always renders;
  P5(c)'s default is only reachable on paths no test knows, so it cannot create
  the skip-incentive gradient the r7 scope-line ruling protects (P1 is
  untouched: a clean pass still prints a pass line — a scoped one).
- Recording provenance at the comparison site is implementable without changing
  normative codec behavior — comparison *outcomes* gain identity, comparisons do
  not change. If giving `bundle/verify.go` typed errors turns out to engage the
  Rust-primary rule, P5(b) does not require typed errors specifically, only
  classification where the facts are in scope; a GUI-side comparator wrapper is
  an alternative site. Confirm before building, as r7's C-3 remedy already
  flags.
- r7's return-site sweep table is accurate as of fork `b8a23bf`; I did not
  re-run it (brief forbids a fresh audit). The committed sweep script replaces
  that trust with a command.
- Single-sig's eleven exits get rows from the same mechanical sweep, and I-2's
  "exit → observation → status" restatement is folded together with C-3, since
  both are P5(b) instances.
