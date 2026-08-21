# BURNDOWN — the FOLLOWUPS ledger, 2026-08-20

Operator-standing plan (fable, per "ask fable for followup burndown plan").
Scope: schedule, not audit. Inputs are the controller's measured facts
(163 entries; 126 not marked CLOSED; 16 self-declared-resolved; 110 genuinely
open; duplicate numbers F-109 and F-120) plus a headings-only sweep of the
ledger. No code was read.

---

## 1. THE HEADLINE

The ledger is not 110 items of debt — it is roughly **three real work queues
wrapped in a bookkeeping problem**. About a third of the "open" mass is either
already resolved in its own title (16 entries), owned by a phase that has since
**shipped** (the SPEC_multisig_build_repair S0–S5/P0–P2 items, and — most
alarmingly — **2 entries still labelled "B2b — CRITICAL, gates the phase"** for
a phase that closed months ago), or is a historical note that was never a task.
Another third is correctly parked on phases that have not arrived yet (release
tag, systemwide payloads, polish/v0.0.1, journeys) and needs no touch now. The
27 ownerless entries are a filing defect, not a queue: each needs a two-minute
routing decision, not work. Fixing the ledger means one mechanical close-out
session, one reconciliation session against the shipped phases, one triage
sitting for the ownerless block — and then the genuinely-open remainder rides
its owning phases exactly as the standing rule says, with the live tr/wsh
cycle absorbing F-214/215/216/218 where they are already owned. No dedicated
"burndown project" beyond that; the rule maintains itself at phase boundaries.

**One caveat that outranks the whole plan:** the 2 open "B2b — CRITICAL"
entries (the F-106/F-107/F-108 family) must be verified FIRST. Almost
certainly they are fixed-but-unrecorded — records have historically been the
weak half — but if either is genuinely unfixed, it is a Critical carried past
a gate and jumps the queue ahead of everything below, including the live
cycle.

---

## 2. IMMEDIATE — mechanical, no engineering (one session)

Order within the session:

**2.0 Verify the two B2b CRITICALs.** Grep the fix commits / current source for
each. Expected outcome: mark CLOSED with the commit SHA in the heading. If not
fixed: stop, escalate to the operator, do not proceed with the rest of this
plan.

**2.1 Close the 16 self-declared-resolved entries.** Every entry whose own
title says WITHDRAWN / FIXED / ANSWERED / DONE / RULED / SUBSUMED / MISFILED /
DOWNGRADED-to-nothing gets `CLOSED` added to the heading, per the ledger's own
convention ("status lives in the heading", because status gets *counted* more
than read). Keep bodies and nested entries verbatim. Zero judgment calls; if
one turns out to be only *partially* resolved (e.g. an F-151-style "(1) DONE;
(2)+(3) deferred"), split the heading status, do not close it.

**2.2 Repair the duplicate F-numbers.** F-109 and F-120 each have two entries.
Spot-check shows both second entries (at ~L3631 and ~L3656) are audit-round
*addenda about the same subject* as the originals, not distinct items. Repair:
demote each second entry to a nested `####` addendum under the original
heading, cross-referenced by date — the convention already blesses nested
entries as the reasoning record. If either turns out on reading to be a
genuinely different item, assign it the next free number (F-219+) instead.
Never reuse a number; numbers are cited in commits and reports.

**2.3 Close the record-only entries.** Entries that are notes, not tasks:
F-72 (heading says "historical note, do NOT rewrite"), F-83 ("ACCEPTED
LIMITATION, not a follow-up — operator ruling"), F-111 ("SUBSUMED by the
F-108 design" — close pointing at F-108). Mark CLOSED-AS-RECORD /
CLOSED-SUBSUMED. The text stays; only the "open" bit changes.

Commit this session as its own commit ("ledger reconciliation — no code
changed"). Expected effect: 126 non-CLOSED drops to roughly 100–105, and every
remaining open heading is a real task.

---

## 3. BATCHES

Named batches; each is a scheduling unit, not a review.

### Batch A — OVERDUE RECONCILE (SPEC_multisig_build_repair residue)
**What:** every open entry owned by a multisig-build-repair stage that has
shipped — the P0/P1/P2 items (F-157/158/159/160), the S1–S5 "gating" items
(F-169, F-170, F-171, F-174, F-175, F-180, F-182, F-185, F-189, F-190), and
the strays (F-166 "post-S0 / its own cycle", F-167/168 "folded at close").
**Why together:** identical question for each — *the owning stage closed
green; was this item satisfied on the way, or carried past the gate?* Items
marked "gating" for stages that passed were, by definition of the gate, either
done (close with evidence pointer) or the gate was hollow (re-own explicitly
by operator ruling — an overdue item may not silently re-park itself).
**Size:** ~15 items, expected mostly closures; one session of grepping stage
records and walk scripts. This is bookkeeping with teeth, still not
engineering.

### Batch B — OWNERLESS TRIAGE (the 27)
**What:** the 27 entries with no owning phase.
**Why together:** it is one *kind* of thinking — routing — and doing it in one
sitting keeps the routing consistent.
**The rule, applied per item in ≤2 minutes, in this precedence order:**
1. Title says resolved → should have been in the §2.1 pass; close.
2. Test-infrastructure of any weight → **polish / v0.0.1** (the operator's
   2026-08-12 ruling already decides this class; F-149/F-153-shaped items).
3. Touches funds, seeds, keys, addresses, or normative codec behavior →
   assign to the **live tr/wsh cycle's next open stage** or the **pre-tag
   gate** (Batch C), whichever comes first. Risk-set items do not float.
4. Docs / wording / naming / cross-repo manual → **ownerless residue,
   batches to the end** — the one legitimate ownerless state, and say so in
   the heading so the next sweep skips it.
5. None of the above and nobody has cited it since it was filed →
   **retire**: CLOSED-RETIRED, one line of why. Carrying a dead item costs a
   read every sweep, forever, and re-filing is cheap if it ever bites.
**Size:** 27 decisions, one sitting. Expected split from the headings: a
third to polish/v0.0.1, a handful to the cycle/pre-tag, a third to legitimate
residue, and a real retire tail.

### Batch C — THE PRE-TAG GATE (release-blocking set)
**What:** everything owned "before the release tag": F-85, F-92, F-98, F-101,
F-102, F-112, F-113, F-115, F-116 (plus whatever Batch B routes here).
**Why together:** they share a trigger, not a subsystem — none is due until a
tag is proposed, and then all of them are due at once. Convert this set into
the literal pre-tag checklist: the tag proposal is not GREEN while any member
is open.
**Size:** ~9 items, mixed engineering; burned in the tag-prep phase, not now.
**Do not** start these early "to get ahead" — they are correctly parked.

### Batch D — RESIDUE & WIPE (post-merge polish and hardening)
**What:** the seed-residue / zeroing family: F-88, F-90, F-104, F-109
(downgraded Minor), F-110, F-120, F-122, plus F-143 (flash verification).
**Why together:** one subsystem (memory hygiene on the engrave paths), one
kind of thinking (enumerate copies, prove the wipe), and several entries
explicitly cross-reference each other. Doing them separately re-pays the
context cost of the residue model every time.
**Size:** 6–8 items, real engineering, Minor-severity by operator downgrade.
**When:** the "post-merge polish and hardening" phase never got a slot on the
calendar — that is why these look overdue. Give it one: schedule Batch D as
its own short cycle immediately after tr/wsh Stage 6 closes, before tag prep.

### Batch E — OPERATOR JOURNEYS (13 items)
**What:** F-127, F-128, F-130–F-134, F-136, F-137, F-139, F-140, F-147,
F-156 — the `#mnemonic` journeys/doc family.
**Why together:** they are the journeys phase; the `/journeys` skill is the
resume point and 7 of 10 programs still lack journeys, so the phase is live
work anyway. These burn as that phase's intake list, not as a separate chore.
**Size:** 13 items, mostly doc/CLI-truth work, some (F-130, F-137) touching
codec behavior — those obey the Rust-primary rule when picked up.

### Batch F — PARKED FUTURE CYCLES (no action now, confirm the park)
**What:** systemwide payloads (F-123, F-125, F-145-remainder, F-148, F-155),
"needs its own brainstorm/R0" items (F-150, F-152, F-196, F-124), next
`#seedhammer` cycle (F-211), key-&-password custody (F-205), post-release
features (F-117, F-118), and the long ~35-phase one-item tail.
**Why together:** the only action is to confirm each parks on a phase that is
*still plausible*. A one-item phase that no roadmap mentions anymore is a
retire candidate under Batch B rule 5.
**Size:** a 20-minute sweep during Batch B's sitting, zero engineering.

### Batch G — END RESIDUE (cross-cutting Minors/Nits)
**What:** the "none — batches to the end" set: F-184, F-193, F-194, F-200,
F-201, F-203, plus F-71/F-75/F-82-style residue and whatever Batch B adds.
**When:** v0.0.1 polish phase, as the standing rule already says. Untouched
until then.

---

## 4. ORDER — interleaved with the live cycle

1. **§2.0 first**: verify the two B2b CRITICALs. Everything below assumes
   they close as fixed-unrecorded.
2. **Immediate session** (§2.1–2.3): the 16 closures, the duplicate-number
   repair, the record-only closures. One commit.
3. **Ask the operator for the Stage 6b ruling now** (QR-series wire shape;
   what may cross a display channel). It gates 6b work and costs the operator
   one message; asking early means the answer is waiting when 6b starts.
4. **Batch A** (overdue reconcile) — next working session. It is the last
   bookkeeping pass, and it must precede Batch B so the ownerless triage
   routes against a truthful board.
5. **Batches B + F in one sitting** — the 27 routing decisions plus the
   parked-phase confirmation sweep. After this, zero ownerless-without-reason
   entries exist. Ledger is now *structurally* healthy; remaining work rides
   phases.
6. **Live cycle resumes as the main line — the burndown does not preempt it:**
   - **F-216** (Stage 5 — keyless template + mk1 cards shows no addresses):
     the slot-mapping rule is an unmade decision, so it starts with an
     operator ruling, then lands in Stage 5's scope.
   - **F-215** (Stage 6 — shape guard refuses legitimate engraves): first
     engineering item of Stage 6; it blocks a real engrave today, which makes
     it the sharpest open item in the ledger.
   - **F-218** (Stage 6 — duplicate xpubs accepted by `md encode`):
     funds-safety, normative admission behavior → **Rust `md` first with a
     test vector, then the Go convergence port**, per the Rust-primary rule.
   - **F-214** (Stage 6 — tap leaves the device cannot derive): the refusal
     is correct; this is a capability gap. Park it at Stage 6's tail —
     implement only if Stage 6's scope ruling includes it, else re-own to a
     named future cycle at Stage 6 close (never back to ownerless).
   - **Stage 6b** (QR-series transport) once its ruling arrives, carrying its
     assigned items (F-192, F-199, F-204, F-206, F-208 are already owned by
     S6b — burn them inside it, per the per-phase rule).
   - **Stage 6c** (BSMS / Nunchuk / Sparrow export) after 6b. 6d stays
     deferred.
7. **Batch D** (residue & wipe) as its own short cycle after Stage 6 closes.
8. **Batch C** (pre-tag gate) when a release tag is proposed — it *is* the
   tag checklist.
9. **Batch E** (journeys) whenever the journeys phase next resumes; **Batch G**
   at v0.0.1 polish.

Steps 1–5 are the actual "burndown": roughly three sessions of bookkeeping
and routing, no new engineering. Everything after is normal phase discipline.

---

## 5. STOPPING RULE — what healthy looks like, measurably

The ledger is healthy when all of the following hold, each checkable by grep:

1. **Zero self-declared-resolved-but-open**: no non-CLOSED heading contains
   WITHDRAWN / FIXED / ANSWERED / DONE / RULED / SUBSUMED / MISFILED /
   RETIRED.
2. **Zero duplicate F-numbers** among headings.
3. **Zero overdue items**: every open entry's owning phase is a phase that
   has not yet closed. (Operational test: at each phase entry, the standing
   reconcile sweep finds nothing whose owner already passed.)
4. **Zero unexplained ownerless entries**: every "no owning phase" heading
   says `ownerless residue — batches to the end` explicitly, or it is a
   filing defect to fix on sight.
5. **No Critical/Important severity on any open entry** whose owning phase
   is not the live one.

Explicitly NOT part of health: **the open count.** 60 open items correctly
parked on real future phases is a healthy ledger; 10 items with one stale
CRITICAL label is not. Do not schedule burndown sessions to drive the number
down.

Maintenance thereafter is what the standing rule already prescribes, plus one
addition: the **phase-entry reconcile sweep** (existing rule) catches overdue
items, and a **15-minute ledger check at each cycle close** (new, cheap)
catches conditions 1, 2 and 4 before they accumulate. If those two checks stay
clean for two consecutive cycles, this plan is complete and no successor
burndown is needed.

---

*Filed by the burndown-planning agent, 2026-08-20. Spot-checks performed:
ledger header/conventions, the non-CLOSED heading list with owning phases, and
the four F-109/F-120 entry sites. No entry bodies beyond those were read; no
code was read.*
