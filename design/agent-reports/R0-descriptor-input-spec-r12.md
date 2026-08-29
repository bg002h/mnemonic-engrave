# R0 round 12 — PROPORTIONAL re-review of the r11 fold

**Target:** `design/SPEC_descriptor_input.md` at `b535916` ("spec: fold R0 r11 --
the directive leaves the quote; counts retired"). Tree is clean at that commit
(`git status --short` empty; `b535916` is HEAD).
**Source of truth for what was required:**
`design/agent-reports/R0-descriptor-input-spec-r11.md` (0C/1I/2M/2N).
**Scope:** the five-edit fold only — fidelity to r11's five findings, and defects
the fold itself introduced. Not a fresh audit. r1–r11 measured results, sweeps,
the walk log, the citation gate, all rulings and dispositions taken as settled.
**Reviewer:** independent context, opus tier. Read-only.
**Diff read in full:** `git show b535916` — 30 changed lines in
`SPEC_descriptor_input.md` (+21/−12) and 3 added lines in
`WALK_descriptor_input_2026-08-28.md`. Every hunk is dispositioned below.

---

## Counts — NEW findings

| severity | count |
| --- | :-: |
| **Critical** | **0** |
| **Important** | **1** |
| Minor | 2 |
| Nit | 1 |

**The spec does NOT re-close GREEN this round.** One Important is open, and it
is fold-introduced at the edit that answered r11's *smallest* finding — the
same shape as the previous two rounds. r11's five are all addressed as text;
four are cleanly closed, and the fifth (new-N1, a Nit asking for one hedging
word) was answered with an **exclusivity claim** that is falsifiable in both
directions and contradicts §6 and §11 on an exit code.

---

## Disposition of r11's five

| r11 finding | verdict | evidence |
| --- | :-: | --- |
| **new-I1** (implementer directive inside the operator-facing quoted span) | **FIXED** | §5.1 L836–841. The `*"…"*` span now ends at `` ships."* `` and the directive sits outside it, in the bullet's trailing parenthetical — the shape every sibling site uses. **Re-run mechanically, not by eye:** all **45** `*"…"*` spans extracted multi-line-aware and swept for `substitut\|placeholder\|implementer\|verbatim\|directive\|per §\|editorial` → **0 hits** (r11 measured exactly one, this one). The identifier sweep r10 introduced was re-run over the same 45 spans (`§\d\|F-\d{3}\|S[123]\|R0\|NEW-[A-Z]\d\|new-[A-Z]\d\|walk W\d\|conjunct \d\|EXIT_`) → **0 hits**, so the fold did not trade one leak species for another. **The mixed-input case §6 requires is covered:** §6 L1201 states *"A descriptor mixing an (a)-shaped and an (a″)-shaped key matches both this row and the next; both fire"*, and the new directive reads *"Each offending key and path is substituted; a mixed input repeats the key clause per offender"* — which names every offender and is strictly more determinate than r11's "names all of them" (it says how the message grows). |
| **new-M1** (partial-tier inventory not exhaustive; the "machine-counted" label vouched for it) | **FIXED — count retired, all cited rows covered; two new wording defects** | §5.4 L1064–1071. The count and the "machine-counted" label are gone and the class is now stated as the conjuncts' business. **Re-measured:** §6 has **34** table rows; **11** carry a §4.7-conjunct or §4.3 admission citation (L1202, L1203 conjunct 2; L1204 conjunct 3; L1205 conjunct 5; L1206, L1207 §4.3; L1214, L1215 conjunct 1; L1216, L1217, L1218 conjunct 7). All **11** map onto a named class in the new sentence, including r11's three orphans — `ypub`/… → *"the refused version bytes"*, `tr(sortedmulti(…))` → *"and shapes"*, `/0/1/*` → *"deeper tails"*. The exhaustiveness ask is closed. What the new wording introduced is **r12's new-M1** (`/0/1/*` bucketed as *underivable*, which §4.7 contradicts, and which collides with §5.4's own `wallet-id:` bullet) and **r12's new-M2** (the "exactly §6's admission-refusal rows" gloss is `--as`-dependent). |
| **new-M2** (`multi`-in-window parenthetical over-claims md1-packability) | **FIXED** | §5.4 L1058–1062. *"md1-packable"* is now qualified *"when its use-site paths are md1-representable"*, and the false consequent (*"whose refusal says md1 is available"*) is replaced by *"either way, stripping its identification would blind the operator at the decision the refusal asks of them"* — which is true for both subcases, including the `/0/*` `multi` whose §5.1 variant-2 refusal says *"No path in this build engraves this file."* The tier assignment the parenthetical justifies is unchanged and still correct (conjuncts 2–7 hold, the md1 path admits the shape → FULL). |
| **new-N1** (follower list not partitioned by tier) | **PARTITIONED AS ASKED — and the partition is the source of r12's new-I1** | §5.4 L1056–1058. The list is now split. But it was split with an exclusivity claim — *"the PARTIAL block's **one follower** is a §4.7 admission refusal"* — where r11 asked for a hedge, and the claim is false in both directions (see new-I1). r11's own construction ({inadmissible wallet, `--as` omitted}) is now not merely ambiguous but decided the other way from §6 L1190 and §11 item 5. **Not closed.** |
| **new-N2** (W14 corrected 45 lines away, no marker in situ) | **FIXED** | `WALK_descriptor_input_2026-08-28.md:451–453`. The erratum sits inside the W14 heading block, immediately above the body, and its content matches the corrections section item 1 verbatim in substance (*"one plate per STRING"*, *"the keyed card is TWO plates"*). **The section it points at exists:** `# Corrections to this log (R0 r10, 2026-08-28)` at **L526**, and the pointer's wording ("below") is directionally right (451 → 526). The two in-body claims r11 also named (L472–473, L479) remain uncorrected in situ, ~20–28 lines under the marker; a reader entering at the heading is covered, a reader arriving by grep on "one-plate" is not. That is the residue of a Nit and I do not re-file it. |

**Fidelity: 4 of 5 closed; 1 (new-N1) addressed in a direction that opens an
Important.**

---

# NEW findings

## new-I1 (Important) — the follower partition is stated as EXCLUSIVE and is false in both directions; one of the two cases changes an exit code, and nothing in the spec resolves it

**Where.** §5.4 L1056–1058, written by this fold to close r11's new-N1:

> the **FULL block**, whose followers are: a pack, the `--as`-omitted choice
> block, §5.1's window refusal, and §5.3's refusals; **the PARTIAL block's one
> follower is a §4.7 admission refusal** (partitioned per R0 r11's new-N1).

r11 rated the un-partitioned list a **Nit** for one stated reason: the preamble
*"The tier is decided by what does NOT depend on the flag"* forecloses reading
the list as a trigger, so the list was descriptive. The fold converted it into a
partition with an "one follower" quantifier. A quantifier is falsifiable, and
this one falsifies twice.

**Constructed failure A — a FULL-tier wallet whose follower is PARTIAL's
exclusive one.** Input `wsh(multi(2, [fp/48h/0h/0h/2h]xpub…/<0;1>/*,
[fp2/…]xpub…/<0;1>/*))`, invoked `me sysw pack --as descriptor` in a build where
the descriptor path HAS shipped (S2; §11 item 4 explicitly schedules the
`--as descriptor`-only rows there, so this is a specified invocation, not a
hypothetical).

- **Tier: FULL.** Conjuncts 2–7 hold; the md1 path admits the shape (§4.7
  conjunct 1, L640–651, "on the `--as md1` path ONLY … the three `multi`
  twins"). r11's own per-row trace dispositions this exact row **FULL**, and
  that disposition is settled.
- **Follower: a §4.7 admission refusal.** §4.7 conjunct 1 L647–651: *"Under
  `--as descriptor` … the shape conjunct remains the seven forms"* — so `multi`
  fails **conjunct 1** on that path. The §6 row that fires is L1199
  (`wsh(multi(…))` under `--as descriptor`), and §5.5's own row for the same
  cell cites §4.7 conjunct 1 for the md1 side and the device refusal for the
  other.
- **The collision:** an implementer applying the new sentence contrapositively
  ("admission refusals follow PARTIAL blocks") strips `wallet-id:`,
  `address 0:` and the compare prompt from precisely the operator being told
  *"This wallet can still be engraved: `--as md1` …"* — the harm r9's I1 and
  walk W13 exist to prevent. The fold's own rescuing parenthetical covers only
  the **window** case (*"A `multi` input in the window is full-tier: its refusal
  is the window's, not admission's"*); it does not reach the shipped-build
  `--as descriptor` case, where the refusal **is** admission's.

**Constructed failure B — a PARTIAL-tier wallet whose follower is a FULL-listed
one, and the two spellings differ in EXIT CODE.** Input
`wsh(sortedmulti(0, K1, K2))` — r11's own construction — invoked with **no
`--as`**.

- **Tier: PARTIAL** (conjunct 2 fails on both paths).
- **§5.4 as folded:** PARTIAL's *one* follower is a §4.7 admission refusal →
  `EXIT_REFUSED` **(3)**, with §6 L1203's threshold-0 text.
- **§6 L1190:** *"**`--as` omitted** | §5.1's text. **`EXIT_USAGE` (2)**, not 3
  — nothing was refused, a choice was not made."*
- **§11 item 5:** *"`--as` omitted with a descriptor input exits **2** and
  prints §5.1's block."* — unconditional; an inadmissible descriptor is still a
  descriptor input (it parses; §5.4 fires *"on EVERY successful whole-input
  parse"*).

So the spec now mandates **two different exit codes and two different stderr
bodies for one invocation**, and there is **no precedence rule anywhere to pick
between them**: grepping the whole file for `precedence|order of checks|in this
order|before the flag|flag check` returns exactly one hit, §4's L275, which is
about cascade *diagnostics* and says explicitly *"precedence does not decide
admission"*. §11 item 4 requires a test asserting the **text** of every §6
refusal; the author of the `--as`-omitted row's test picks a fixture, and if it
is an inadmissible wallet the two spec sections demand opposite assertions.

*(A third, weaker falsification sits two sentences later: L1072–1073, *"§5.3(b)'s
label warning, where it applies, follows the block"* — a second follower for any
tier where it applies. I do not lean on it, because "where it applies" may scope
it to pack paths.)*

**Why this is Important and not the Nit it replaced.** r11's Nit was "a list
that could be misread"; this is a normative quantifier that is false, that
decides an interaction the spec had left open, and that decides it against two
other sections neither of which the fold touched. It is not resolvable by
reading harder — the fold either meant to move admission ahead of the flag check
(a real normative change that §6 L1190 and §11 item 5 must then carry) or it
meant the list to stay descriptive (in which case the quantifier must go). Both
are one-clause edits; the spec must say which.

**Not prescribing the fix.** Either direction closes it, but the direction is a
design decision with an operator-visible consequence: refusing an
anyone-can-spend wallet outright (3) is arguably better than offering a choice
between two flags that both refuse it — that is walk W11's own rule, *"no
refusal may point at a path that refuses in the CURRENT build"* — and if that is
the intent, §6 L1190 and §11 item 5 are the places it has to be stated.

---

# Minor

**new-M1 — `/0/1/*` is bucketed as *underivable*, which §4.7 contradicts, and
naming it in the PARTIAL class collides with §5.4's own `wallet-id:` bullet
twenty lines below.** §5.4 L1065–1068 now reads *"It covers exactly §6's
admission-refusal rows — **the underivable wallets** (mixed network, hardened
use-site, non-consecutive multipath, the wrapped and script-slot shapes,
**deeper tails**)…"*.

- **The bucket is wrong on the spec's own evidence.** §4.7 conjunct 7 L688–690:
  *"Everything else in `parsePath`'s grammar (a bare fixed index, multi-component
  tails like `/0/1/*`) is refused as **UNMEASURED**, per the closed-set rule."*
  §6 L1218 says the same in operator language: *"outside the set the device is
  **measured to handle**."* The device's `parsePath` accepts it; nothing in the
  spec claims its addresses are underivable. r11 named this row and this reason
  explicitly, and the fold moved it from "unlisted" to "listed in the wrong
  class" — which is worse for a later reader, because the class list is now the
  only thing that speaks to it.
- **The collision.** §5.4 L1084–1087 (untouched by the fold): *"For a wallet md1
  cannot represent — (a)/(a″) shapes, **deeper tails** — the line is instead:
  `wallet-id: none — …identify it by the checksum in the canonical line and by
  address 0.`"* That is a **FULL-block** bullet. The fold now places deeper tails
  in the PARTIAL class, which prints *"no `wallet-id:`, no `address 0:`"*. The
  same section therefore says a `/0/1/*` wallet both prints `wallet-id: none`
  (referring the operator to an `address 0:` line that is not printed) and
  prints no `wallet-id:` at all. Since deeper tails fail conjunct 7 on **both**
  paths, the bullet's "deeper tails" clause is unreachable under the tier rule —
  a dead clause pointing at a dead line. Half of this predates the fold (r10's
  tier rule already implied it, and r11's trace put L1218 in PARTIAL without
  noticing the bullet); what the fold added is the explicit naming that makes
  the contradiction assertible from one section without deriving anything.
- Minor rather than Important because the **rule** — *"a wallet NO path admits …
  gets the PARTIAL block"* — is normative, stated first, and decides the case
  correctly; only the gloss and the stale bullet clause disagree with it.

**new-M2 — "covers exactly §6's admission-refusal rows" re-introduces an
`--as`-dependent label as the class descriptor, which is what r10's new-I1
removed from this boundary.** The rule above it is `--as`-independent by
construction ("passes conjuncts 2–7 AND whose shape at least one `--as` path
admits"). "§6's admission-refusal rows" is not: §6 L1199 (`wsh(multi(…))` under
`--as descriptor`) **is** a §4.7 conjunct-1 admission refusal for that
invocation (L647–651), and r11's settled trace makes that row **FULL**. Read by
citation the gloss is exhaustive and true over the 11 annotated rows (measured
above); read by substance it captures a twelfth row that the rule excludes. The
"exactly" is what does the damage — it invites a reader to derive the tier from
the row inventory instead of from the conjunct predicate, which is the failure
mode r10 spent an Important closing. Same species as r11's new-M1, one wording
generation on: the justification prose does not track the rule it justifies.

---

# Nit

**new-N1 — the fold's reflow left one line at 105 columns in a file that hard-wraps at ~75.** L1058 (`follower is a §4.7 admission refusal (partitioned per R0 r11's new-N1). (A `multi` input in the window is`) is 105 chars; L1051–1057 and L1059–1071 measure 53–77. (L1073, 93 chars, is pre-existing context and not this fold's.) Measured with `awk '{print length}'`, not by eye. Cosmetic only — but it is the visible signature of a paragraph edited four times without a re-wrap, and the next fold touches this same paragraph.

---

# Verified in passing — recorded so a later round does not re-spend it

- **Both quoted-span sweeps are clean at `b535916`:** 45 spans, 0 directive
  hits, 0 identifier hits. The widening r11 recommended (`substitut|placeholder|
  verbatim`) was applied here and finds nothing.
- **The reflowed variant-2 quote is still W5-compliant.** It leads with the
  verdict (*"--as md1 cannot carry this wallet either — key `@N` uses
  `<path>`."*), carries no internal identifier, and names only current-build
  actions. The reflow changed line breaks and the closing punctuation position
  only; the sentence order is unchanged.
- **§11 item 5 still matches §5.1.** It requires *"BOTH alternative variants
  tested (an md1-representable input, and an (a)/(a″)-shaped one)"* and §5.1's
  two bullets are labelled exactly *"input md1-representable"* and *"input
  (a)/(a″)-shaped"*. The fold did not disturb the pairing.
- **The retired count leaves no orphan.** Grepped the whole spec for
  `machine-counted|five rows`: **0 hits**. Tier vocabulary
  (`PARTIAL|FULL block|full-tier|TWO tiers`) appears **only** in §5.4
  L1052–1063, so no other section carried a copy of the inventory that the fold
  would have had to update.
- **No stale copy of the pre-fold variant-2 wording survives.** Grepped
  `design/` (excluding `agent-reports/`) for `names all of them` → 0 hits;
  `cannot carry this wallet either` → the one §5.1 site.
- **The walk erratum's target exists and its content is faithful** — `#
  Corrections to this log (R0 r10, 2026-08-28)` at L526, item 1, which the
  erratum paraphrases without changing a fact.
- **§6 still has 34 rows**; the fold added and removed none.

---

# What would re-close the round

new-I1 folded — either drop the exclusivity quantifier and restore the list to
the descriptive reading r11 measured as safe, or make the precedence explicit and
carry it into §6 L1190 and §11 item 5 — then a re-review scoped to *"did the fold
fix the one, and did it introduce a defect"*. The two Minors are single-clause
edits (drop or re-bucket "deeper tails", and reconcile or retire the
`wallet-id:` bullet's "deeper tails" clause; soften "exactly" to a
non-`--as`-dependent phrasing) and can ride along with the Nit's re-wrap.

**What is closed and should not be re-opened:** the quoted-span convention
(both sweeps clean, mechanically re-run), the multi-in-window qualifier, the
retirement of the row count, and the walk erratum. Four of r11's five are done.
One line for the cycle's record: **for the third consecutive round the fold —
not the artifact — is the defect source, and for the second consecutive round
the Important sits at the clause that answered the round's SMALLEST finding.**
r11's new-N1 was a Nit whose recommended fix was one hedging word; the fold
chose a stronger, cleaner-reading sentence instead, and a stronger sentence is a
bigger target. A Nit fix that upgrades prose from descriptive to normative is
not a Nit fix.

---

# What the spec's own text leaves open (unchanged by this fold, listed so the
# round that does close it inherits the list)

**§9 residuals (7):** (1) nothing run on hardware; (2) the three admission-table
cells have never been exercised — *a gate that has never executed*; (3) change
addresses and testnet unmeasured in the `--as md1` address equality; (4) the
published `md-codec` 0.42.0 tarball not byte-compared to the tree; (5) TinyGo
compilation of a new `sysw.Classify` arm unchecked; (6) two negative claims with
named, narrower search scopes; (7) §6's refusal texts *"have not been walked with
the operator"* — still stated as open even though the walk reached refusal text
at W5/W11/W13, so it remains due a scope update at the next fold (r11 flagged
this; it is unchanged at `b535916`).

**Parked with S2 (F-418, S1 → S3 → S2):** §11 item 1 (the `Descriptor` classify
round trip), §11 item 4's `--as descriptor`-only refusal rows — which is where
new-I1's construction A lands — and §11 item 6 (a `ClassDescriptor` record
loaded and displayed on a real device, the discharge of §9 item 2). All three
need the device on the bench.

**Named follow-ups the spec defers to:** F-413 (host-side SLIP-132
normalisation), F-414 (descriptor + other records in one container), F-416
(`--in`'s contract note in `SPEC_systemwide_payloads` §5.6), F-417 (md1 wire
extension seam), F-422 (**RULING WANTED**, owning phase *"descriptor-input plan,
before S1 closes"* — only an interim status-quo ruling is recorded),
F-420/F-421 (cross-tool referrals, owning phase "with or after S1"), and F-423
(plate packing, fork-side, with S2).
