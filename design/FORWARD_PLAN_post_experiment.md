# FORWARD PLAN — transaction engraving, post-experiment

**Written 2026-08-25 by the architect agent, at the operator's direction, after
`design/EXPERIMENT_plan_vs_brief_2026-08-25.md`.** This is a decision document,
not a review. The operator's standard binds it: **reasonable-effort funds
safety** — shippable, falsifiable, converging.

Facts below marked *(verified this session)* were measured by this agent
against the repos and binaries, not transcribed from the experiment write-up.

---

## 1. The base is ARM B, with named grafts from A

**Arm B (`exp/tx-brief-driven`) is the base.** It proved the requirement the
brief actually stated — its QR symbols round-trip through an independent
mainstream decoder (ZXing, reverse scan order, byte-identical), and a
1,027-byte payload packed by the Rust binary is decoded by the Go device code —
while arm A never decoded a symbol it produced. It did this in 40% of A's code,
with **no new wire container**: two record classes inside the existing sysw
container, so there is no 75-byte framing to maintain in two languages and no
carried identifier that can disagree with the body. It already computes the
semantic confirmation **on the device** (structural tx parse + the
txid↔chunk_set_id binding at 2^-20), which is exactly what the 2026-08-25b
ruling requires and what A still owed to P3. It verified the firmware links
under TinyGo on target. And it left `mt` untouched, honoring both the brief and
the Rust-primary discipline. A's real value — it knows what "done" means — is
portable: its acceptance surface, and roughly six concrete mechanisms listed in
§4/P1, graft onto B without dragging the MTX1 framing with them. The framing
itself, and the 66 occurrences of `wtxid` it exists to carry, do not survive
(§3). One consequence is accepted knowingly: A's 25-vector shared fixture and
its Python generator die with the framing; B's conformance rests on the
mt1 corpus generator, mirrored Rust-answer fixtures, and the cross-language
payload test, which is the same discipline with fewer artifacts.

## 2. The funds-safety bar, made concrete

This feature engraves an **already-signed** transaction. It never signs, never
derives, never touches seed-class records (the program's admission row is
`{ClassMt, ClassTx}` only). The funds-relevant failure class is therefore
narrow: **an artifact that passes every check and is worthless — or
dangerous — in steel.** Reasonable effort means every check below exists with
a test that can fail; it does not mean proving the transaction is valid.

**Checks that MUST exist (and where they stand):**

1. **Signature-presence predicate** (§3) — every input carries a non-empty
   scriptSig or ≥1 witness item. Host: refuse, exit 4, override
   `--allow-unsigned-inputs` that names the failing input indices. Device:
   loud flag + mandatory legend substitution, consistent with the rulings.
   *NEW — the only new admission rule this plan adds.*
2. **Strict admission, no BCH correction ever on the pack/engrave path** —
   a corrected string engraved verbatim cuts the damage into steel. *(B has:
   D5, both sides agree by construction.)*
3. **Semantic confirmation of chunk sets, computed ON DEVICE** — structural
   parse (canonical compactSize, whole buffer consumed) + set-id binding.
   Closes the C3 smuggling channel per ruling 2026-08-25b. *(B has, both
   sides, conformance-tested with Rust answers verbatim.)*
4. **Independent decode proof for every QR class emitted** — the ZXing
   round-trip in the suite now; a phone scan of engraved steel at S0 (P4).
   *(B has the first; the second is the one gate only hardware can run.)*
5. **Cross-language seam tests** — a Rust-packed payload decoded by Go, and
   fixtures generated independently of the code under test. *(B has.)*
6. **txid truth at every surface** — full display-order txid on the review
   screen, on the legend, and recomputable post-cut (any QR scanner, or a
   chunk string hand-typed into `mt`). *(B has screens/legend; P1 adds the
   `mt inspect` raw subject; P4 proves the physical path.)*
7. **Bearer posture** — raw hex never accepted on argv; review screen states
   "anyone holding the plates can broadcast"; no echo before any refusal.
   *(B has.)*
8. **Rulings conformance** — incomplete sets pack loudly and engrave;
   `not_a_transaction` engraves under a substituted, un-overridable legend.
   *(B diverges today — it refuses unconfirmed sets and classes them SECRET;
   P0 folds it to the rulings.)*
9. **Never emit unvalidated geometry** — 0.9/0.6 mm modules only; 0.3 mm and
   sub-3.0 mm text stay out of the emitter until a plate validates them.
   *(B has.)*

**What we are choosing NOT to guard, and why that is acceptable:**

- **Signature VALIDITY.** Checking signatures requires prevout scripts and
  amounts the offline device does not have and can never fetch. A transaction
  with garbage witness bytes of plausible structure passes. The signing wallet
  is the authority on validity; our artifact-side guarantee is *presence and
  structure*, stated on the review screen. Guarding this would make the device
  a verifier it cannot be — this is the line "reasonable effort" draws.
- **Witness tampering with intact structure, pre-pack.** The retired wtxid
  field could not see this either (it was self-computed from the same bytes at
  pack time). Post-pack, BCH checksums on chunks and QR ECC already cover
  transit corruption.
- **Fee/amount/recipient sanity.** The wallet showed the user these before
  signing. Not our concern.
- **An adversary holding the machine or forging plates.** Physical custody is
  assumed, as everywhere else in this product.
- **Honest empty/empty inputs** (P2A anchor spends and similar exotica) —
  false-positives of check 1; the override flag names them and the operator
  decides. Documented, not designed around.
- **Regenerability of the payload** — ruled 2026-08-25: report loudly, pack.

## 3. E17 versus the signature predicate: REPLACE

**The finding is correct, and I verified it independently** *(verified this
session, corpus vector `2dcf2b97…`)*: the honest 222-byte body parses as
segwit with witness items `[2]`; its stripped 113-byte form has the identical
txid, `segwit=false`, scriptSig lengths `[0]`, and **fails** the predicate
"every input has a non-empty scriptSig OR a witness item" while the honest
body passes. The plan's accepted-cost sentence — *"not a gap any field can
close"* — is false as written because it quantifies over **carried fields**;
the discriminator is a **predicate over the body**, and an honest signed
legacy transaction has non-empty scriptSigs, so a both-empty input is not
"an honest witness-free transaction". Ten rounds missed it; the word
`scriptSig` appears nowhere in the spec or the plan *(verified: 0 hits)*.

**Ruling: the predicate replaces E17 and the carried `wtxid`.** In the B base
there is no framing, so there is nothing to remove — the predicate is an
addition, implemented identically in `sysw/tx.rs` and `mt.ParseTx`, with the
stripped corpus vector as its RED test. It catches **both** stripping cases,
including the self-consistent one A shipped at exit 0 and the plan declared
unpreventable.

Two residuals, named so the retirement is reasoned and not amnesiac:

- **Evasion by stuffing:** a non-empty garbage scriptSig defeats the
  predicate — but changes the txid, so the artifact stops impersonating the
  honest transaction. The impersonation class (same txid, no signatures) is
  closed.
- **What E17 alone caught:** witness bytes replaced *between pack and verify*
  with structure intact. That span is BCH-protected already, and E17's
  self-computed wtxid never saw pre-pack tampering. Accepted under §2.

## 4. Sequenced work to shippable

Each phase lands on master green before the next; P1/P2 may run concurrently
(disjoint surfaces: code vs documents).

**P0 — Land the base; fold to the rulings; add the predicate.**
Merge `exp/tx-brief-driven` (both repos; `mt` untouched). Then, in the same
phase because none of it is optional: (a) incomplete chunk sets pack loudly
AND engrave under a substituted legend — B currently refuses them at the plate
and classes them SECRET, both contrary to the rulings; fold to non-secret +
flagged + mandatory substitution, host warning naming **every** missing index;
(b) `not_a_transaction` engraves under the un-overridable substituted legend;
(c) the signature predicate, both sides, RED-tested with the stripped vector;
(d) CI: a deploy key + side-by-side checkout for the private
`mnemonic-transaction` repo, because B's path dependency otherwise breaks
every workflow *(verified: workflows check out this repo only)*.
**Done when:** each ruling's behavior has a test; the stripped body exits 4 at
the host and flags on device; full suites + CI green on master.

**P1 — Graft the A items that survive the format.** Six, each with its test:
section cap 8191 → 32,734 on BOTH sides with A's cross-repo formula test
(lifts B's ~3.5 KB ceiling — its top listed limitation — and makes the 8 KB
pathological case deliverable); the exit-code vocabulary (2/3/4 split);
R11′'s two messages (no-payload vs no-transaction-in-payload); legend cut
LAST with an emission-order test ("an unsigned plate is an unfinished
plate"); `capgate` run once over B's `saCapacity`; `mt inspect` gains the raw
subject (small `mt` PR — the post-cut verify step needs a tool that can run).
Operator to/from labels ride as ordinary `text:` records — documentation, no
format change.
**Done when:** each graft lands with its test; the delivery ceiling row in the
docs states the new number.

**P2 — Reconcile the shipped code against the R0-GREEN spec; retire the plan.**
The spec is mostly framing-agnostic *(verified: one MTX1/wtxid hit in 1,775
lines)*, so its walk-derived R-rules and refusal lists still bind. Walk it
against B item by item; classify every requirement **met / met-differently
(record why) / not-met (schedule to P3-P5) / superseded-by-ruling**. Output is
ONE acceptance sheet (~200 lines, `design/ACCEPTANCE_engrave_transaction.md`):
live rules, vector list, refusal/flag coverage table, the P3/P4 gates. That
sheet is the normative surface going forward. §5 disposes of the plan.
**Done when:** every spec requirement is classified and the not-mets have an
owning phase.

**P3 — The UI walk and the journey.** B's own top gap: no end-to-end
screen-sequence test. Add the `runUITouch` walk of the full program — choice →
review → plan-confirm → engrave loop — for TEXT and QR paths, **including the
legend-substitution screens P0 added**; golden images; an emulator journey
doc; then a live journey walk WITH the operator (the highest-yield review this
project has measured — divergences classified refusal/warning/default/not-our-
concern/doc-only).
**Done when:** the walk drives a complete simulated engrave both paths, and
the operator walk's findings are folded or filed with owners.

**P4 — S0, the hardware session (the machine is ~a week out; nothing above
waits for it).** One batched cut list: legend/text legibility at the chosen
faces; engraved QR at 0.9 mm and 0.6 mm modules scanned by ≥2 phone apps; one
full small-transaction plate each path; post-cut verify exactly as the screen
instructs — phone scan → txid matches; one chunk string hand-typed into `mt`
→ verifies. 0.3 mm stays never-emitted unless it validates here.
**Done when:** an engraved plate round-trips through a phone scanner to the
correct txid. **This gate cannot be simulated and the release does not ship
before it** — the feature's entire output is physical.

**P5 — Ship.** One whole-diff independent review (opus, scoped to the merged
feature, brief states what §2's gates already machine-verified); CHANGELOG;
the `ci/staging` push ritual; tag. **`cargo publish mt-codec` stays deferred**:
it is irreversible, nothing in this release needs it (the deploy key covers
CI), and it becomes its own gated act when `mt`'s API is worth freezing.
**Done when:** the review closes 0C/0I and the tag's workflow is green.

## 5. The 2,062-line plan: RETIRE it

Mark `IMPLEMENTATION_PLAN_P1_me_container.md` superseded at the top, pointing
here and at the acceptance sheet; git keeps the history; the sixteen agent
reports stay where they are. Do not shrink it to the ~600 useful lines —
those lines describe MTX1, and MTX1 is not shipping. What actually carries
forward: **the acceptance sheet (P2) carries "done"**; the spec's R-rules
carry the requirements; the handful of surviving mechanisms are P1 work
items with tests, which outlive any document. The experiment's lesson is that
prose guidance the implementer doesn't act on is cost — so keep the artifacts
that are executable (vectors, gates, tests) and let the narrative go.

## 6. The review regime going forward

Ten rounds produced a document whose implementer used ~30% of it, and the
sharpest defect in it (§3) was found by *running the case*, not by an
eleventh reading. The regime that replaces it:

1. **Enumerate the lenses up front; one round each; stop when the lenses are
   spent.** For risk-set work: correctness (opus), adversarial funds-safety
   (opus), and a journey walk with the operator. Sonnet verifies folds. No
   round N+1 because round N found something — a fold re-triggers review only
   if it changed logic.
2. **Demonstration gates replace reading rounds.** Every requirement of the
   form "X works" carries a named executable proof (*decode what you produce;
   run the gate you wrote; the stripped vector goes RED*), and the proof must
   have RUN before any reviewer is engaged. Both arms' verdicts agree here:
   what B did right was pressure-to-demonstrate, and what ten rounds missed
   was findable by one measurement. Closure-is-lens-closure already binds; add
   its corollary — **a review round may not be spent on anything a vector
   could have caught.**
3. **Plans cap at the density A measured useful**: current rules only, the
   wire/behavior tables, vectors, near-miss pairs, and the one-paragraph shape
   of the code. Archaeology lives in agent-reports; struck material is
   deleted, not struck through. A plan line earns its place iff it changes
   what a competent implementer would otherwise do.
4. **Implementation stays one agent, TDD, machine gates inline** (unchanged),
   with the one mandatory whole-diff review at the end (unchanged).

This keeps the two things the record shows actually finding defects —
independent adversarial review in small doses, and executed vectors — and
drops the thing the record shows not paying: iterated re-reading of a
lengthening document.

---

*Files referenced: `design/EXPERIMENT_plan_vs_brief_2026-08-25.md`,
`design/FOLLOWUPS.md` (rulings 2026-08-25 / 2026-08-25b),
`design/SPEC_engrave_transaction.md`,
`design/IMPLEMENTATION_PLAN_P1_me_container.md`,
`_experiment/{A,B}/mnemonic-engrave/IMPLEMENTATION_LOG_{A,B}.md`,
`mnemonic-transaction/crates/mt-codec/src/test_vectors/mt1_v1.json` (predicate
verification input).*
