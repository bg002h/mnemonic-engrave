# S6a — scope and design decisions (operator stand-in)

Date: 2026-08-16. Role: operator stand-in on four scoping/design decisions for the
S6a pre-hardware software burndown. Not a review; no code was audited. All facts
taken as given from the controller's measured list at fork `main` (b8a23bf).

Standing directive applied throughout: **"permissive on input, expressive on
output, and loudly declare assumptions we make to fulfill user requests."**

---

## Q1 — Cycle scope

**DECISION:** (b) — S6a = F-198 + F-197 + F-195, one plan, one review surface;
F-199 gets its own cycle (S6b) that still closes before the flash, and that
cycle opens by answering its design question, not by coding.

**WHY:** All three of F-198/F-197/F-195 are the same sentence said three ways —
"the single-sig path never learned what S4/S5 taught the multisig paths" — and
land in the same file and the same fifteen lines. Splitting them means a
reviewer reads that function three times in three cycles and the operator waits
through three gates to get one coherent restore document; bundling them means
the Critical is not actually slowed, because F-197 and F-195 add no open design
questions — they are settled fixes riding the same diff. F-199 is the opposite
case: a different file, a different concern, and its own follow-up says it
"needs a decision, not a reflex" — bundling an open design question with a
Critical is how the Critical ends up waiting. Excluding F-199 from S6a is a
scheduling choice only: it remains S6-owned and gating, and the flash does not
happen until S6b closes GREEN. What (a) gets wrong: it ships F-198's label fix
while leaving the restore document with no inventory at all (fact 2 is broader
than F-198), so the operator holding steel years later still reads a document
that cannot tell them whether the set is complete.

---

## Q2 — The unfiled pre-engrave census gap (fact 6)

**DECISION:** Include it in the S6a plan **now, before the R0 gate**, as a named
scope item — not filed, not folded in later.

**WHY:** The "text nobody has read" hazard is about additions that enter a cycle
*after* review — folded in around the gate. Putting the census call in the plan
before R0 is the exact opposite: it enters *through* the gate and gets the same
review as everything else. The alternative — filing it and leaving the path
silent — means the very cycle whose purpose is a hardware flash sends an
operator to commit a 2- or 3-plate cut, minutes of irreversible machine time per
plate, with no count on the screen, on the one path this cycle is already
editing, using a function that already exists and is already called twice. That
is the same defect family (the flow is silent about the set) at the machine
instead of on the document. "Expressive on output" decides this: the census is
one call, the plan grows by one line, and the review surface it joins is the one
already being reviewed.

---

## Q3 — The false sentence ("this build can hold several")

**DECISION:** (a) — make the shared seed-handling ruling count-derived, in the
existing `plateWord`-style house pattern, keyed on the path's seed capacity.
The multisig arm is byte-identical to today's text; only the one-seed arm is new.

**WHY:** (c) is out because the ruling's warnings — the plates are the secret,
do not leave a mid-build machine unattended, power off when done — bind the
single-sig operator exactly as hard as the multisig one; dropping the ruling
trades a false clause for a missing warning. (b) is out because two independent
prose strings stating the same house ruling is precisely how folds fail by
incomplete propagation: the next edit fixes one and strands the other, and the
stranded one is on steel-adjacent paper read years later. (a) keeps one function
true on every path, matches the pattern the file already uses so it never prints
"1 plates", and keeps the multisig documents unchanged so nothing churns.

**TEXT:**

Seed capacity == 1 (single-sig):

> Seed handling: this build does not time out. The seed you entered -- this build holds exactly one -- stays in device memory until the build ends, and on a full build the words are also on the plates as they are cut. Do not leave a mid-build machine unattended: the plates are the secret. Power the device off when you are done.

Seed capacity > 1 (both multisig paths) — unchanged from today, verbatim:

> Seed handling: this build does not time out. Every seed you entered -- this build can hold several -- stays in device memory until the build ends, and on a full build the words are also on the plates as they are cut. Do not leave a mid-build machine unattended: the plates are the secret. Power the device off when you are done.

Selector note (design, not code): key the arm on what the path *can hold*, not
on how many seeds were entered this run — a multisig build that happened to see
one seed still truthfully "can hold several", and two otherwise-identical
documents should not disagree because of runtime happenstance.

---

## Q4 — F-195's shape: which arm(s) get the explicit seed statement

**DECISION:** Both arms speak. The absence arm is the load-bearing new line; the
presence arm is one short line that converts the inventory's `ms1` entry into
its consequence, and is count-derived so it stays true on single-sig and both
multisig paths.

**WHY:** The test behind BOTH ARMS SPEAK is whether a reader holding one
document, years later, with no other document to compare against, can
distinguish "absent" from "this document never mentions it". An inventory line
is an artifact of listing, not a declaration: the reader of a full-set document
sees `ms1 secret share: 1 plate` and must already know what ms1 means and must
already know that a watch-only document would have said something — knowledge
that lives in other documents. Speaking the presence arm removes both
dependencies for one line of text, converts jargon into consequence ("treat that
plate as the secret itself"), and keeps the house rule exception-free — a rule
with no exceptions is the only kind that survives future edits, because a later
reformat of the inventory cannot silently delete the only place presence was
ever stated. On the absence arm, the line must do the one thing silence cannot:
tell the reader whether to keep hunting for a seed and where spending authority
actually lives. Deliberately excluded from the presence arm: any claim that the
seed plates *alone* suffice to spend — true on single-sig, false on a k-of-n
multisig set — and any passphrase claim, which is the passphrase lines' job.

**TEXT:**

Presence arm, one seed (single-sig full):

> Seed: this set CONTAINS the seed. The plate marked 'ms1 secret share' in the inventory above is the seed backup -- treat that plate as the secret itself.

Presence arm, several seeds (multisig full):

> Seed: this set CONTAINS seeds. Each plate marked 'ms1 secret share' in the inventory above is a seed backup -- treat each one as the secret itself.

Absence arm (watch-only, all paths):

> Seed: this set contains NO seed. It is watch-only: these plates can rebuild the wallet's addresses but can never spend. If funds must be recovered, the seed must come from somewhere else -- no plate in this set holds it.

---

## WHAT I AM ASSUMING

1. **The flash gate is unmoved by the cycle split.** S6a and the F-199 cycle
   (S6b) both close GREEN before the hardware flash; putting F-199 in its own
   cycle is scheduling, never deferral past its owning phase. If schedule
   pressure forces a choice, the flash waits.
2. **F-199's cycle opens with a decision pass** — the operator (or an explicit
   operator stand-in) answers its open design question before any fix is
   written, per its own follow-up's "a decision, not a reflex".
3. **Q2's inclusion is conditional on timing:** the census item enters the S6a
   plan before R0 review. If for any reason it arrived mid-cycle instead, it
   should be filed, not folded — the whole argument for including it is that it
   gets reviewed.
4. **The seed-handling arm is keyed on path capacity** (single-sig = exactly
   one; multisig = several), not on the runtime seed count, so identical builds
   print identical documents.
5. **The single-sig restore document will use the shared census functions**, not
   bespoke prose — that is why every string above was written to be true on all
   three paths, and why fixing fact 5 is a prerequisite inside S6a rather than
   separate work.
6. **Watch-only single-sig still holds a seed in device memory during the
   build** (it must, to derive the xpub), so the seed-handling ruling's
   "stays in device memory" clause is true there too. This was not in the
   measured-facts list; if it is false, the one-seed ruling text needs a second
   look before it ships.
7. **The inventory label `ms1 secret share` is stable and shared** across paths;
   the Q4 presence-arm text names it, so a future relabel must propagate into
   these lines (a grep-able coupling, and deliberate — the sentence pointing at
   the inventory is what prevents the two from contradicting each other).
8. **Multisig documents do not churn:** the capacity>1 arm in Q3 is byte-identical
   to today's text, so S5's already-reviewed multisig output is unchanged by S6a.
