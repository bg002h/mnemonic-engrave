# R0 round 13 — PROPORTIONAL re-review of the r12 fold

**Target:** `design/SPEC_descriptor_input.md` at `f5ebce4` ("spec: fold R0 r12 --
tier and follower decoupled; the precedence rule"). Tree clean at that commit
(`git status --short` empty; `f5ebce4` is HEAD).
**Source of truth for what was required:**
`design/agent-reports/R0-descriptor-input-spec-r12.md` (0C/1I/2M/1N).
**Scope:** the fold only — fidelity to r12's four findings, and defects the fold
itself introduced. Not a fresh audit. r1–r12 measured results, sweeps, the walk
log, the citation gate, all rulings and dispositions taken as settled.
**Reviewer:** independent context, opus tier. Read-only; nothing modified,
committed or pushed.
**Diff read in full:** `git show f5ebce4` — 43 changed lines in
`SPEC_descriptor_input.md` (+30/−13), one file. Every hunk dispositioned below.

---

## Counts — NEW findings

| severity | count |
| --- | :-: |
| **Critical** | **0** |
| **Important** | **2** |
| Minor | 3 |
| Nit | 1 |

**The spec does NOT re-close GREEN this round.** The fold's central move — the
decoupling of tier from follower — is correct and closes r12's construction A
cleanly. The precedence rule it added to close construction B is **new normative
logic that reached 2 of 5 sites** (new-I1) and is **keyed on the wrong predicate**
(new-I2): it discriminates on §4.7 *admission*, where the operator-facing question
— and the spec's own NORMATIVE invariant at §5.3 L1000–1001 — is *carriage in this
build*. The two differ on a non-empty class the spec names in three other places.

For the **fourth consecutive round the fold, not the artifact, is the defect
source.** This one is different in kind from r11's and r12's, though: those were
false claims written into justification prose. This one is a correct rule,
incompletely propagated and one predicate off. That is a narrower failure than
the last three, and both findings are single-clause edits.

---

## Disposition of r12's four

| r12 finding | verdict | evidence |
| --- | :-: | --- |
| **new-I1** (the follower partition stated as EXCLUSIVE; false in both directions; construction B changes an exit code with no precedence rule) | **PARTIALLY FIXED — construction A closed; construction B closed at 3 sites and survives at 3** | See the two re-runs below. Re-filed as r13's **new-I1**. |
| **new-M1** (`/0/1/*` bucketed as *underivable*; collides with §5.4's own `wallet-id:` bullet) | **PARTIALLY FIXED — M1a fixed, M1b untouched** | §5.4 L1082–1084 now reads *"and the UNMEASURED closed-set residue (deeper tails and the like — refused as unmeasured, not as underivable; R0 r12's new-M1a)"*, which matches §4.7 conjunct 7 L688–690 and §6 L1229 exactly. The bucket is right. **The collision r12 filed in the same finding is untouched:** `grep -n 'deeper tail'` returns **exactly two** hits — L1083 (the PARTIAL class list) and L1100 (the FULL-block `wallet-id:` bullet). Re-filed as r13's **new-M1**. |
| **new-M2** ("covers exactly §6's admission-refusal rows" is an `--as`-dependent class label) | **FIXED** | §5.4 L1078–1080: the gloss is now *"It covers the rows refused by the `--as`-independent conjuncts and the no-path-admits shapes"*. Both halves are `--as`-independent by construction — conjuncts 2–7 are flag-independent (§4.7 L652–697), and "no-path-admits" is quantified over both paths. **"exactly" is gone**, so the gloss no longer invites deriving the tier from a row inventory. The twelfth row r12 constructed (§6 L1214, `wsh(multi(…))` under `--as descriptor`) is no longer captured by the label, and the fold additionally names that row explicitly as FULL four lines above (L1061–1062). Closed. |
| **new-N1** (the fold's reflow left one 105-column line) | **FIXED** | Re-measured with `awk '{print length}'` over L1049–1080: **max 77**, min 44 (L1071). No line in the fold's region exceeds 77 in a file that hard-wraps at ~75. (Lines >90 elsewhere in the file — L98, 323, 470–477, 692 — are all pre-existing and outside this fold.) |

**Fidelity: 2 of 4 closed outright; 2 partially, both re-filed.**

### Re-run of r12's construction A — `wsh(multi(…))` under `--as descriptor`, full tier

**Result: the decoupled rule now describes it without contradiction. CLOSED.**

Tier: conjuncts 2–7 hold, the md1 path admits the shape (§4.7 conjunct 1
L640–651) → **FULL** (§5.4 L1054–1056). Follower: §4.7 conjunct 1 fails on the
`--as descriptor` path → the §6 L1214 admission refusal. The new text states
exactly this and names the row: L1057–1059 *"**The tier decides only WHICH LINES
print; what FOLLOWS the block is decided independently by §5's own logic** — any
tier may precede any follower"*, and L1061–1062 *"(A full-tier `wsh(multi(…))`
under `--as descriptor` meets a conjunct-1 admission refusal…)"*. The
contrapositive misreading r12 constructed ("admission refusals follow PARTIAL
blocks" → strip `wallet-id:`/`address 0:`/compare prompt from the operator being
told *"This wallet can still be engraved: `--as md1`"*) is no longer available:
the enumeration is of followers, not a partition, and the worked example is the
very input that broke the old sentence. Cross-checked against the `wallet-id:`
emission rule (L1099, *"Emitted only when the wallet HAS an md1 policy form"*) —
a `multi` with `<0;1>/*` has one, so the FULL block is fully printable. No
residue.

### Re-run of r12's construction B — `wsh(sortedmulti(0, K1, K2))`, `--as` omitted

**Result: the three sites the fold edited agree on 3. THREE OTHERS STILL SAY 2.**
Not closed → new-I1.

| site | says | agrees? |
| --- | --- | :-: |
| §5.4 L1065–1070 (new precedence rule) | no path admits → §4.7 admission refusal at **3** | ✅ |
| §6 L1205 (`--as` omitted row) | *"(input at least one path admits)"* → 2; *"For an input NO path admits, the §4.7 admission refusal fires directly at (3) instead"* | ✅ |
| §11 item 5 L1628–1631 | *"AT LEAST ONE PATH ADMITS exits **2**… with an input no path admits… directly at **3**"* | ✅ |
| **§5.1 L739** | *"Omitting it is a **usage** error, `EXIT_USAGE` (2), not a refusal."* | ❌ **2** |
| **§5.1 L806** | *"the input IS a descriptor and gets the "--as decides" block at `EXIT_USAGE` (2), which keeps §11's item 5 true for all four formats"* | ❌ **2** |
| **§6 L1235** | *"a multi-line BlueWallet or JSON file parses whole and gets the "--as decides" block instead"* | ❌ **2** |

Sweep method: `grep -n 'as decides\|choice block\|§5.1.s block\|§5.1.s text\|EXIT_USAGE'`
over the whole file — 10 hits, every one classified. The three ❌ rows are all of
them that speak to a descriptor input; the rest are the empty-file, whitespace,
and multi-operand rows, none of which reach admission.

---

# NEW findings

## new-I1 (Important) — the precedence qualifier reached 2 of the 5 sites that assert the `--as`-omitted exit code; the section that OWNS the behaviour is one of the three that still says 2

**Where.** §5.1 L739, §5.1 L806, §6 L1235 (table above).

The fold's own commit message enumerates its propagation list — *"6's row and 11
item 5 carry the qualifier"* — and the list is short by three. This is not a
stale echo in commentary; each of the three is a normative statement that an
implementer would reasonably treat as governing:

- **§5.1 L739 is the topic sentence of the section that owns `--as` omission.**
  An implementer writing the `--as`-omitted handler opens §5.1 — titled
  *"`--as` is required; there is no default and no fallback"* — and reads
  *"Omitting it is a usage error, `EXIT_USAGE` (2), not a refusal."* The
  qualifier lives in §5.4, a section about the identification block. Nothing in
  §5.1 points forward to it.
- **§5.1 L806 additionally carries a justification that the fold made false.**
  The clause *"which keeps §11's item 5 true for all four formats"* was true of
  the old item 5 and is not true of the new one: item 5 is now two cases, and
  the L806 sentence asserts the exit-2 case unconditionally for every input that
  parses whole. The fold edited item 5 out from under its own citation.
- **§6 L1235** repeats the unqualified claim inside the multi-record row, which
  is where a reader lands when reasoning about which of the two rows fires.

**Constructed failure.** A BlueWallet export with `Policy: 0 of 3` — or 21
cosigner lines (conjunct 3), or mixed `xpub`/`tpub` (conjunct 5) — saved as
`wallet.txt`, invoked `me sysw pack --in wallet.txt` with no `--as`. It is a
well-formed BlueWallet file, so it parses whole (§5.1's discriminator, measured
on the fork's own `sh` fixture) and no path admits it.

- By §5.1 L806 and §6 L1235: it *"gets the "--as decides" block"* at **2**.
- By §5.4 L1065–1070, §6 L1205 and §11 item 5: the §4.7 admission refusal fires
  at **3**.

This is r12's new-I1 verbatim, on the same input class, surviving at three sites
the fold did not visit. The operator-visible stake is unchanged and is why it
gates: for `sortedmulti(0, …)` the exit-3 text is §6 L1219 — *"threshold 0 means
NO signature is required: anyone who can see this script can spend from it… if it
already holds funds, treat them as at risk now"* — and the exit-2 text is a menu
asking them to pick a flag. Telling the holder of an anyone-can-spend wallet to
choose a plate format is materially worse than telling them nothing.

**Why Important and not Minor.** It is an exit-code contradiction, in normative
text, on a funds-relevant refusal, reachable from the highest-traffic entry point
in the spec. It is also the exact failure mode the constellation rule names —
*folds fail by incomplete propagation: the facts are right and the duplicates are
left.* Three one-clause edits close it.

## new-I2 (Important) — the choice block's new firing condition is keyed on ADMISSION, where the spec's own NORMATIVE invariant is CARRIAGE-IN-THIS-BUILD; three specified texts offer the operator a flag that refuses their file

**Where.** §5.4 L1069–1070 (new), §6 L1205 (new qualifier), §11 item 5 L1628–1630
(new) — all three say *"at least one path admits"*. Against §5.3 L1000–1001
(NORMATIVE, walk W11): ***"No refusal names a flag that refuses in the current
build."*** And against §5.1 L765: *"so the choice text itself never offers a dead
flag."*

**The predicate is the wrong one, and the spec's own vocabulary knows it.** §4.7
is *admission*; §5.3 is *"Two representable-in-md1 limits"* and the spec says
"md1-representable" throughout. The fold picked admission. The two sets differ,
and the spec names the difference in three places:

- §4.7 conjunct 7 L695–697: *"for a `multi` form, which has no `--as descriptor`
  path, carried by **NEITHER** path; §5.3's refusals say so rather than pointing
  at a flag that also refuses."*
- §5.3 L991–994: for a `multi` form *"the refusal states that no `me` path
  carries it."*
- §6 L1216: *"No `me` path engraves this descriptor this release."*

An input in that class **is admitted** (md1's shape set is a strict superset of
descriptor's, §4.7 conjunct 1) and therefore, by the new rule, gets the choice
block.

**Constructed failure A — a wallet no build will ever carry, handed a two-option
menu. Any build.** Input `wsh(multi(2, [fp/48h/0h/0h/2h]xpub…/0/*,
[fp2/…]xpub…/0/*))`, invoked `me sysw pack --in wallet.txt` with no `--as`.

- Admission: conjunct 1 ✅ (md1 path admits the `multi` twin); conjuncts 2–7 ✅
  — `/0/*` is `/i/*`, a member of conjunct 7's closed set L679.
- → *"at least one path admits"* is TRUE → §6 L1205 mandates §5.1's block at
  **`EXIT_USAGE` (2)**.
- §5.1's block offers `--as descriptor` (§6 L1214: refuses — device parser takes
  `sortedmulti`, not `multi`) and `--as md1` (§6 L1216's `multi`-form
  replacement: refuses — *"No `me` path engraves this descriptor this release"*).
- **Both offered flags refuse this file, in every build, forever.** §5.1 L765's
  claim *"the choice text itself never offers a dead flag"* is false for it, and
  §5.3 L1000–1001's invariant is violated. The block's own `--as md1` line even
  steers a `multi` holder toward the dead option: *"Carries policies --as
  descriptor cannot."*

**Constructed failure B — W11's own archival file, on the one invocation W11 did
not walk. The S3-only window, i.e. the first shipped build.** Input: the
Specter-era `{label, descriptor}` JSON with `/0/*` (`nonstandard/parse_test.go:22`,
the fork's own fixture; walk W11 establishes this user as *"the engraving tool's
core clientele"*), invoked with no `--as`.

- Admitted by both paths (conjunct 7 admits `/i/*`) → choice block at **2**.
- The block marks `--as descriptor (not available in this build)` per §5.1
  L763–764 (M6). The **only unmarked option is `--as md1`, which refuses this
  input** (§5.3(a), §6 L1216).
- So the block's live entry is dead for this file. W11 closed doors 1 and 2 for
  exactly this input and ruled *"No refusal may point at a flag that refuses in
  the CURRENT build"*; the fold has now positively specified door 3 to point at
  it. §5.1 L836–841 already holds the correct text for this operator — window
  refusal variant 2, *"No path in this build engraves this file. It loses nothing
  by waiting"* — reachable only by typing `--as descriptor` explicitly.

**Constructed failure C — the same conflation on the sibling invocation, plus an
unordered follower pair the fold's new sentence asserts is ordered.** Input
`wsh([fp/84h/0h/0h]xpub…/<0;1>/*)` — a bare key in a script slot, §6 L1230, **no
path admits it** — invoked `--as descriptor` in the S3 window.

- Two followers apply: §5.1 L817's window refusal (3) and §6 L1230's admission
  refusal (3). §5.4 L1057–1059 says the follower is *"decided independently by
  §5's own logic"* — but §5's logic does not decide this pair. §5.1 L817's window
  rule is unconditional on admissibility; §6's five-step cause rule L1166–1174
  *"ranks CASCADE (parse) failures only"* and explicitly says admission rows
  *"fire from their own checks — the rule never selects them"* (L1185–1188).
- If the window refusal wins, its variant selector (L827, *"decided by
  md1-representability"*) is a **binary with no arm for inadmissible inputs**: a
  `/<0;1>/*` path is md1-representable, so **variant 1** fires — *"Available now:
  --as md1 — me converts and packs in one step… nothing is lost by waiting"* —
  which is false twice over (md1 refuses `wsh(KEY)` on conjunct 1, and waiting
  never helps: no build ever packs this shape). §11 item 5's sibling test pins
  *"BOTH alternative variants"* against the same two-arm selector.

**Why Important.** A missing case in new normative logic, with an
operator-visible wrong outcome that is worse than silence on the journey rule's
own test — a menu implying a choice exists where none does, for an archival user
the walk identified as core clientele, reinstating a defect class the operator
personally ruled on at W11/W12. It also falsifies two sentences the spec states
absolutely (§5.3 L1000–1001, §5.1 L765). And §6 L1205's row is now inside a table
whose preamble binds every row to *"names only next actions executable in the
CURRENT build (walk W11)"* (L1176–1181) — the row the fold just edited is the one
that cannot satisfy it.

**Not prescribing the fix.** The mechanism already exists in three flavours — the
M6 inline mark (§5.1 L763), window substitution (§5.3 L996), and variant 2's
"no path in this build" text (§5.1 L836) — so the direction is a choice among
existing devices, not new machinery. What the spec must say is which predicate
governs the block: admission, or carriage-in-this-build. It currently says
admission in three places and carriage in two.

---

# Minor

**new-M1 — r12's new-M1b is untouched: "deeper tails" is now named EXPLICITLY in
both the PARTIAL class list and the FULL-block `wallet-id:` bullet, 17 lines
apart.** `grep -n 'deeper tail'` → exactly two hits. L1083 (fold-written): the
UNMEASURED closed-set residue, i.e. **PARTIAL** — which prints *"no `wallet-id:`,
no `address 0:`, no compare prompt"* (L1077–1078). L1100 (untouched): *"For a
wallet md1 cannot represent — (a)/(a″) shapes, **deeper tails** — the line is
instead: `wallet-id: none — … identify it by the checksum in the canonical line
and by address 0.`"* — a **FULL**-block bullet whose text refers the operator to
an `address 0:` line the PARTIAL block does not print. Since `/0/1/*` fails
conjunct 7 on **both** paths, the bullet's "deeper tails" clause is unreachable
under the tier rule: a dead clause pointing at a dead line. The fold fixed the
bucket label and left the collision the same finding named. Minor for r12's
reason and it holds: the rule (L1054–1057) is normative, stated first, and
decides the case correctly. (The `(a)/(a″) shapes` half of the same bullet is
live and correct — those inputs are admitted by the descriptor path, hence FULL.)

**new-M2 — the precedence rule's premise sentence is false as written, and is
contradicted four lines above it.** §5.4 L1065–1066: *"admission is
`--as`-independent, so for an input NO path admits…"*. Admission is **not**
`--as`-independent: §4.7 conjunct 1 L640–651 is the one conjunct that differs by
path (*"on the `--as md1` path ONLY … the three `multi` twins"*), and the fold's
own sentence at L1061–1062 relies on that difference. What is true is the weaker
pair the rule actually uses: conjuncts 2–7 are flag-independent, and
"no-path-admits" is flag-independent because it quantifies over both paths. The
operative predicates are correctly quantified, so the rule survives its premise —
but an implementer applying *"admission is `--as`-independent"* literally to
conjunct 1 refuses `wsh(multi(…))` on the md1 path, which is the capability
regression §4.7 conjunct 1 exists to prevent (r4's NEW-I1) and which §5.5 L1141
marks ✅. Third consecutive round in which justification prose does not track the
rule it justifies.

**new-M3 — the new paragraph was inserted INTO the tier paragraph, so the PARTIAL
block's normative line list now sits under an "`--as` omitted" lead-in.**
Measured: blank lines in §5.4 are at L1050, L1064, L1090 only, so the paragraph
opened by **"Precedence for `--as` omitted — the missing rule r12's contradiction
exposed:"** (L1065) runs to L1089 and swallows the multi-in-window parenthetical
(L1071–1075), the PARTIAL block's line list (L1076–1078), the whole class
inventory (L1078–1087), §5.3(b)'s label-warning note (L1087–1088) and the
*"`me` prints to stderr:"* lead-in (L1089) — none of which is scoped to `--as`
omission. A reader looking up "what does the PARTIAL block print" finds the
answer under a heading phrase that says it is about a flag being absent. Wrong
altitude, not wrong content; a paragraph break before L1076 is the whole fix.

---

# Nit

**new-N1 — the PARTIAL tier rule is now stated twice, in one paragraph pair.**
L1056–1057: *"a wallet NO path admits gets the PARTIAL block"* (fold-written).
L1076–1078: *"A wallet NO path admits — a conjunct failure — gets the **PARTIAL
block**: the first three lines plus…"* (pre-existing). They agree today. They are
21 lines apart with a normative rule between them, and the next fold touches this
paragraph. One of the two should carry the line list and the other should go.

---

# Verified in passing — recorded so a later round does not re-spend it

- **Both quoted-span sweeps are clean at `f5ebce4`:** 45 `*"…"*` spans extracted
  multi-line-aware; identifier sweep (`§\d|F-\d{3}|S[123]|R0|NEW-[A-Z]\d|new-[A-Z]\d|walk W\d|conjunct \d|EXIT_`)
  → **0 hits**; directive sweep (`substitut|placeholder|implementer|verbatim|directive|per §|editorial`)
  → **0 hits**. Identical to r12's measurement — the fold added only annotation
  text, none of it inside a quoted operator span. The new §6 L1205 clause
  (*"(R0 r12's new-I1 precedence rule)"*) sits outside the quotes, per walk W5.
- **§6 still has exactly 34 data rows**, measured at both `0170d7d` (pre-fold)
  and `f5ebce4` (post-fold). The fold added and removed none; it edited one row
  in place (L1205).
- **The r12 N1 rewrap is real:** L1049–1080 measure 40–77 columns, max 77.
- **Both walked journeys still compose correctly with the precedence rule.**
  Journey 1 (BlueWallet `sh` fixture, S3 window, `--as descriptor` explicit) →
  admitted by both paths, window refusal variant 1, unaffected. Journey 2
  (bare BIP-84 `zpub`, childless → §5.3(a′) materialises `<0;1>/*`) → admitted
  by both paths; with `--as` omitted the block fires at 2 with `--as md1` live
  and correct. Neither walked journey is broken by the fold. new-I2's failing
  inputs are the two the walk reached by *question* rather than by step (W11's
  archival `/0/*` file) or did not reach at all (the `multi` + `/0/*` class).
- **No cross-document copy of the `--as`-omitted claim exists.** `grep -rn` over
  `design/` (excluding `agent-reports/`) for `as decides` / `as. omitted` → 0
  hits outside the spec. No implementation plan for this spec exists yet, so
  new-I1's propagation is entirely spec-internal.
- **The FULL/PARTIAL partition is exhaustive and disjoint** as re-worded: FULL =
  (conjuncts 2–7) ∧ (∃path admits shape) ≡ ∃path fully-admits; PARTIAL =
  ¬∃path admits. A conjunct-3 failure with a both-path-admitted shape lands in
  PARTIAL correctly.
- **"No path admits" is unambiguous as to WHICH refusal text fires**, because
  md1's shape set is a strict superset of the descriptor path's (§4.7 conjunct
  1): any shape refused by both is refused by md1's set, so a single
  `--as`-independent §6 row exists for every no-path-admits shape. The
  precedence rule does not need a text-selection clause. (This is what keeps
  new-M2 a Minor.)

---

# What would re-close the round

new-I1 folded — the qualifier carried to §5.1 L739, §5.1 L806 (whose §11-item-5
citation also needs re-wording) and §6 L1235 — and new-I2 folded, by naming which
predicate governs the choice block and, if it is carriage, saying what the block
does for an input no shipped path can carry. The three Minors and the Nit are
single-clause edits and can ride along: re-bucket or retire L1100's "deeper
tails" clause, weaken L1065's premise to the conjuncts-2–7 form, break the
paragraph before L1076, and drop one of the two PARTIAL statements. Then a
re-review scoped to *"did the fold fix the two, and did it introduce a defect"*.

**What is closed and should not be re-opened:** the tier/follower decoupling and
construction A (fully closed, worked example in situ), the `--as`-independent
class label (r12's new-M2), the UNMEASURED bucket for deeper tails (r12's
new-M1a), the 105-column rewrap, both quoted-span sweeps, and the 34-row count.

One line for the cycle's record: **the fold enumerated its own propagation list
in the commit message and the list was short by three.** A fold that names where
its new rule must land is doing the right thing; the check that was missing is
the mechanical one — grep for every sentence asserting the value the rule
changes, before writing the list.

---

# What the spec's own text leaves open (carried forward unchanged; the round that closes it inherits this list)

**§9 residuals (7):** (1) nothing run on hardware; (2) the three admission-table
cells have never been exercised — *a gate that has never executed*; (3) change
addresses and testnet unmeasured in the `--as md1` address equality; (4) the
published `md-codec` 0.42.0 tarball not byte-compared to the tree; (5) TinyGo
compilation of a new `sysw.Classify` arm unchecked; (6) two negative claims with
named, narrower search scopes; (7) §6's refusal texts *"have not been walked with
the operator"* — still stated as open even though the walk reached refusal text
at W5/W11/W13; unchanged at `f5ebce4`, still due a scope update at the next fold
(flagged by r11 and r12).

**Parked with S2 (F-418, S1 → S3 → S2):** §11 item 1 (the `Descriptor` classify
round trip), §11 item 4's `--as descriptor`-only refusal rows, and §11 item 6 (a
`ClassDescriptor` record loaded and displayed on a real device, the discharge of
§9 item 2). All three need the device on the bench.

**Named follow-ups the spec defers to:** F-413 (host-side SLIP-132
normalisation), F-414 (descriptor + other records in one container), F-416
(`--in`'s contract note in `SPEC_systemwide_payloads` §5.6), F-417 (md1 wire
extension seam), F-422 (**RULING WANTED**, owning phase *"descriptor-input plan,
before S1 closes"* — only an interim status-quo ruling is recorded),
F-420/F-421 (cross-tool referrals, owning phase "with or after S1"), and F-423
(plate packing, fork-side, with S2).
