# R0 — IMPLEMENTATION_PLAN_descriptor_input_S2, round 7 (fold review)

**Artifact:** `design/IMPLEMENTATION_PLAN_descriptor_input_S2.md`, as folded at
`d3f59cc`.
**Round 6:** `design/agent-reports/R0-S2-plan-r6.md` (RED, 0C/1I/3M/1N),
persisted at `97fb400`, folded at `d3f59cc`. The reviewed text is exactly
`git diff 97fb400..d3f59cc -- design/IMPLEMENTATION_PLAN_descriptor_input_S2.md`
— **51 insertions, 18 deletions, 8 hunks, all read.**
**Trees:** mnemonic-engrave `d3f59cc`; seedhammer fork `main` @
`a5e29b44637d0657ab8f1ec603f1a375b0cc54cb`.
**Nothing in either repo was modified.** No fork patch was needed: every claim
below rests on plan/spec/source lines read at `HEAD` in both repos, one re-run
of `scripts/plan-cite-check.sh`, and r1–r6's settled tables.

**THE ONE QUESTION:** does the r6 fold resolve each of r6's five findings
(NEW-I1, M1, M2, M3, N1), and did the fold's edits introduce a new defect?

**Counts: 0 Critical / 0 Important / 1 Minor / 1 Nit — verdict GREEN.**

**All five findings are resolved, and NEW-I1 — the only one that was
Important — is resolved in one more place than r6 asked for.** r6's prescribed
fix named two sites (P0.1's member, invariant 1's arithmetic list) plus the
split rule's general qualification; the fold edited all three **and** P2.6's own
bullet (`plan:445-446`), which was the third leg of r6's constructed
counterexample. That counterexample is now unrunnable: its step 1 required
*"do not edit `comment.json` — P0.1 owns it to P2.7"*, and P0.1 (`plan:240-242`)
now says P2.6. Every one of r6's three Minors landed as asked, with all four
cited source sites verified line-exact against the tree, and N1's disposition
landed in the sweep's own survival-set sentence — the sentence r6 quoted —
rather than one section away.

The one Minor is an **incomplete propagation of M1**: the fold moved §7
requirement 3's device-column phrasing INTO P3.5 (`plan:670-671`) and ruled it
P3.5's in the authoritative inventory (`plan:219-224`), but did not remove it
from P2.7's amendment list (`plan:480-481`), so one member is now enumerated
under two owners. P0.1's declared precedence (`plan:209-211`) resolves it, and
the wrong branch produces the cross-repo doc transient r6 itself graded Minor —
so it records, and does not gate.

---

## Method

- Plan, `design/SPEC_descriptor_input.md`, `crates/me-cli/tests/descriptor_refusals.rs`,
  `crates/me-cli/src/descriptor/admit.rs`, `scripts/descriptor-seam-vectors/README.md`,
  and the fork's `nonstandard/descriptor_seam_test.go` read at the stated
  revisions, by line number, never from memory of an earlier round.
- Ownership overlap checked by **enumeration**, not by impression: every
  occurrence of `requirement` and of `defect 4` in the plan located with `grep -n`
  and each hit read in its surrounding member list.
- Manifest-copy census run as a whole-tree sweep in **both** repos
  (`tag-slot`, `7[12] (physical )?row`, `row DEFINITIONS`, `88 . 17`, `89 . 17`),
  then each live hit matched against an owner in the folded plan.
- `scripts/plan-cite-check.sh` re-run independently this round.
- r1–r6's verified tables taken as settled per the brief. The version-gap
  arithmetic (89/72/17/15/2/3/15/37/10), `gen.py:209`'s verbatim embed, the
  "nothing reads `_comment`, CI never runs `gen.py`" negative, and the four
  named source sites' semantics were **not** re-derived.

### Independent machine checks, this round

| check | command | result |
| --- | --- | --- |
| citations | `./scripts/plan-cite-check.sh design/IMPLEMENTATION_PLAN_descriptor_input_S2.md` | `citations resolved: 130 / 130 ; dangling: 0 ; ambiguous: 0` ✅ (matches the brief) |
| `requirement` mentions in plan | `grep -n` | 4 hits: `:219` (P0.1), `:412` (§7 requirement **4**, unrelated), `:480` (P2.7), `:671` (P3.5) → **two owners for requirement 3** |
| `defect 4` mentions in plan | `grep -n` | 4 hits: `:212` (P0.1 coarse list, no owner), `:472` (P2.7 file-half), `:564` (unrelated F-428 cite), `:672` (P3.5 device sentence) → **no double owner** |
| `comment.json` residue | `grep -n` | 7 hits; **zero** assign it to P2.7 (`:112`, `:241-242`, `:445-446`, `:485` all say P2.6/input; `:73`, `:109`, `:508` are payload/sweep text) ✅ |
| `"never feeds one to the parser"` in the spec | `grep -rn` | exactly **one** hit, `SPEC:390` — inside §4.2 (section starts `SPEC:324`), not §7 → Nit 1 |

---

## Fold vs r6's findings

| r6 finding | resolved? | evidence at `d3f59cc` |
| --- | --- | --- |
| **NEW-I1** — `comment.json` is a build INPUT to the sha-pinned vector file, and the fold assigned it to P2.7, the task that runs AFTER P2.6's single regeneration | **RESOLVED — in all three places r6 named, plus a fourth** | Invariant 1 (`plan:110-118`): *"**`comment.json` is a generator INPUT, not a description** (r6 NEW-I1: `gen.py:209` embeds it verbatim into the emitted file, so the pinned sha is a function of it) — it is edited AT P2.6 with `rows.py`, BEFORE `gen.py` runs, or the shipped file carries a NORMATIVE manifest that is false about itself with no test red (nothing reads `_comment`, and CI never runs `gen.py`)"*. P0.1 (`plan:240-242`): *"the three SPEC sites are owned by P2.7 per the r5 I2 split rule, while `comment.json`'s manifest block is a generator INPUT edited at P2.6 — r6 NEW-I1"*. The split rule (`plan:482-485`): *"a member that is a generator INPUT rather than a description is not routed by this rule at all: it is edited in the generating task itself, before the generator runs"* — verbatim the general qualification r6 asked for. **Fourth site, unasked:** P2.6's own bullet now reads *"`scripts/descriptor-seam-vectors/rows.py` + `comment.json` (a generator input, r6 NEW-I1) + JSON together"* (`plan:445-446`) — r6 named its absence there as one of the two reasons *"the wrong reading is the one the plan produces"*. Counterexample dead at step 1. |
| **M1** — the new ownership rule is not applied to two pre-existing P2.7 members | **RESOLVED for §4.2 defect 4; INCOMPLETELY PROPAGATED for §7 requirement 3 → Minor 1** | §4.2 defect 4 is split cleanly and the split is stated from both ends: P2.7 (`plan:471-475`) *"§4.2 defect 4's FILE-half clauses ("§7 marks these rows `device_probe`" — falsified by P2.6's marker retirement, so owned here; the "PANICS the Go parser" DEVICE sentence itself moves to P3.5, whose P3.1 fix is the falsifying diff — r6 M1)"*, and P3.5 (`plan:671-673`) *"§4.2 defect 4's "PANICS the Go parser" device sentence (P3.1 falsifies it; its file-half stays P2.7's)"*. Mutually consistent, no double owner. §7 requirement 3 was **added** to P3.5 (`plan:670-671`) and ruled P3.5's in P0.1 (`plan:222-224`), but `plan:480-481` still lists it among *"the host-truth spec amendments S2 forces"* at P2.7. |
| **M2** — the retag renames the row, and two by-name sites are outside the "COMPLETE" enumeration | **RESOLVED, and both cites verified line-exact** | P0.1 (`plan:242-250`) now carries *"the row-RENAME sites (r6 M2 …): `crates/me-cli/tests/descriptor_refusals.rs:463` (`vector_input` by name — reds loudly at P2.6 and is updated in the P2.6 commit; distinct from `:466`'s refusal-text pin three lines below, which stays P3.5's) and `crates/me-cli/src/descriptor/admit.rs:23` (a SILENT source comment citing the row by name — edited at P2.6 with the rename)"*. Verified in the tree: `descriptor_refusals.rs:463` is `&vector_input("neither/full-origin-ypub")` and `:466` is the `"the device admits exactly …"` string, three lines below, still P3.5's at `plan:669` — **not** double-owned. `admit.rs:23` is `` `parse_extended_key` (which is why `neither/full-origin-ypub` carries `` — a comment, silent on rename, exactly as described. |
| **M3** — `README.md:9` is falsified by the 72nd row, named nowhere, reached by no sweep term | **RESOLVED, with the finder problem stated** | P0.1 (`plan:250-252`): *"`scripts/descriptor-seam-vectors/README.md:9` ("the 71 row DEFINITIONS" — r6 M3: edited at P2.6 with the regeneration; no sweep term reaches it, so this named owner is its only finder)"*. Verified: `README.md:9` is `- \`rows.py\` — the 71 row DEFINITIONS. Host-side columns (\`host_admits\`,` — the count is on that line, and the fold's characterisation of the owner as its only finder is carried into the plan rather than left in the review. |
| **N1** — the new `tag-slots` sweep term hits a completed sibling plan with no stated disposition | **RESOLVED, in the sentence r6 quoted** | P2 gate (`plan:514-517`): *"the S3-parked phrasings must survive ONLY in `design/agent-reports/` and historical review text per P0's inventory; **completed sibling plans — e.g. the S1_S3 plan's manifest copy — are RECORDS, so sweep hits there are triaged, not amended, r6 N1**"*. r6 asked for it "in P0's inventory"; the fold put it in the survival-set clause r6 actually quoted, which is where the P2.6 implementer meets the hit. Equivalent or better; the example names the exact file r6 measured. |

---

## Checked, no defect — stated for the record

### Pressure point 1 — the §4.2 split is performable in two phases, and its residual transient is the class the plan already tolerates

The passage, measured (`SPEC:382-390`, §4.2 begins at `SPEC:324`):

```
382  4. **A fingerprint shorter than 4 bytes PANICS the Go parser.**
383-385  parseBlueWalletDescriptor checks only len(fp) > 4 … panics for fewer.
385-387  Measured: a 1-byte and a 3-byte fingerprint both panic … scan door.
387-389  `me` requires exactly 8 hex characters — matching bip380.ParseKey —
389-390  and §7 marks these rows `device_probe: "panic:parse"` so the Go test
         never feeds one to the parser.
```

The two halves occupy **disjoint line ranges**: the device claim is `382-387`,
the file clause is a trailing subordinate clause at `389-390` hanging off a
*host* sentence. Two amendments in two commits touch different lines; nothing
forces a re-edit of the other half. Performable as ruled.

The transient at P2 close is: `382-387` still says the Go parser panics — **true
of fork `main`, which does not carry P3.1 yet** — while `389-390` no longer
claims the `device_probe` marker, true of the engrave repo's regenerated copy.
Each half is true of its own subject. That is precisely the cross-repo skew
invariant 1 already declares (`plan:151-155`: *"between those commits the two
copies transiently differ"*), and it exists by design, because the vector file's
device booleans are measured on the P3 implementer's patched worktree.

Both alternatives are worse, which is the test the brief sets:

| option | state at P2 close | verdict |
| --- | --- | --- |
| **split as ruled** | spec's device sentence lags fork `main` correctly; file clause matches the engrave copy | tolerated class, already stated |
| both halves at P2.7 | spec asserts the panic is fixed while fork `main` still panics — spec describes a fork-side fix that does not exist | the exact shape r4 M2 moved five members for |
| both halves at P3.5 | the P2 gate closes with §4.2 asserting a `device_probe` marker the repo's own regenerated file no longer carries | the in-repo shape r5 I2 raised, which P2.7 exists to prevent (`plan:486-488`) |

No finding.

### Pressure point 2 — the manifest census: every live copy is owned

Swept both repos; records (`design/agent-reports/**`, prior-cycle reports)
excluded per the fold's own N1 disposition:

| live copy | measured site | owner in the folded plan |
| --- | --- | --- |
| SPEC §7, three sites | `SPEC:1610-1615`, `:1719-1723`, `:1728-1732` | P2.7 (`plan:240`) |
| `comment.json` manifest block | `comment.json:101-113` (`:111`, `:113` carry the arithmetic) | **P2.6** (`plan:112-118`, `:241-242`, `:445-446`) |
| the emitted file's own `_comment` | `crates/me-cli/testdata/descriptor_seam_vectors.json:112`, `:114` | P2.6, by generation from the above |
| engrave seam constants | `descriptor_seam.rs:62`, `:64`, `:373`, `:50-69`, `Pop :130-147` | P2.6, same commit (invariant 1) |
| generator README | `scripts/descriptor-seam-vectors/README.md:9` | P2.6 (`plan:250-252`) |
| S1_S3 plan's fourth copy | `IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md:33` | RECORD — triaged, not amended (`plan:516-517`) |
| fork's vector copy | `nonstandard/testdata/descriptor_seam_vectors.json:112`, `:114` | P3.3, byte-identical + pin |
| fork's population guards | `nonstandard/descriptor_seam_test.go:66-77`, `:157-159` | invariant 1's Go half, at P3.3 |

**Nothing unowned.** Two negatives worth stating with their scope: the fork's
Go test carries **no** per-tag minima table (`grep -n 'covers\|minima\|MANIFEST\|tag'`
returns one hit, the `Covers` struct field at `:59`), so the fork holds no fifth
manifest copy; and `rows.py`'s own docstring (`rows.py:2`) states no count, so it
is not a copy. Both negatives are as wide as those two files only.

### Pressure point 3 — the rest of the fold diff

The header hunk's counts match r6's own header (`0C/1I/3M/1N`) and its
characterisation (*"all 5 of r5's findings verified resolved"*, *"the one
Important was `comment.json`'s ownership"*) matches r6's fold-vs-findings table.
The P2.6 bullet, the P2-gate clause and the split-rule qualification introduce no
term that contradicts text elsewhere in the plan; the sweep-term paragraph
(`plan:505-509`) still describes `tag-slots` as reaching `comment.json`'s block,
which remains true and is about *finding*, not *owning*. `comment.json` has no
residual P2.7 assignment anywhere (`grep`, 7 hits, all consistent).

---

## MINOR

**M1 — §7 requirement 3's device-column phrasing is now enumerated under BOTH
P2.7 and P3.5; only P0.1's declared precedence separates them.** The fold
executed half of r6 M1's remedy — *"move them, or state the transient"* — by
adding the member to P3.5 and ruling it there in the authoritative inventory,
but left the P2.7 entry standing. Measured, all three at `HEAD`:

- **P0.1, authoritative** (`plan:219-224`): *"§7 requirement 3's "the Go test
  asserts the device column" phrasing (`design/SPEC_descriptor_input.md:1496-1498`
  — P3.3's derived rule reads the host column; the never-compare-implementations
  half survives; **owned by P3.5** per the split rule — P3.3, a fork commit in
  P3, is the falsifying diff, r6 M1)"*
- **P3.5** (`plan:670-671`): *"plus, per the same falsifying-diff rule (r6 M1):
  §7 requirement 3's device-column phrasing (P3.3 falsifies it)"*
- **P2.7** (`plan:477-481`), unchanged by the fold and still inside the comma
  list introduced by *"The host-truth spec amendments S2 forces:"*: *"§8's "S2
  is parked" sentence, §5.1's and §7's "after record classification fails"
  gate-trigger sentences (…), **§7 requirement 3's device-column phrasing.**"*

The `git diff` confirms the P2.7 occurrence is a **context line**, not an edit:
the hunk at `@@ -448,14 +469,20 @@` carries `implementer re-reads before touching
the pack path), §7 requirement` and `3's device-column phrasing. **The ownership
split rule (r5 I2): a` unprefixed.

**Constructed counterexample.** The P2.7 implementer works from P2.7's task
list — that is what a task list is for — and amends `SPEC:1496-1497` (*"The Rust
test asserts the host column; the Go test asserts the device column."*) to
P3.3's derived rule. The P2 gate closes: no test reads spec prose, the sweep
finds nothing wrong, everything is green. Ship state for the whole P2→P3 window:
the spec describes a Go test that fork `main` does not have, since the fork's
seam test updates at P3.3. Then the P3.5 implementer arrives at the same
sentence, finds it already amended, and makes a silent no-op — the second edit
never signals that the first happened a phase early.

**Why Minor, not Important.** Three reasons, and I checked each rather than
assuming: (i) the plan contains a determinate answer — P0.1 is declared *"the
COMPLETE enumeration — P2.7 and P3.5 defer to it, not the reverse"*
(`plan:209-211`) — so the contradiction is resolvable inside the document, with
the reason spelled out at the deciding site; (ii) the harm is a **cross-repo doc
transient**, the class r6 itself graded Minor for this same member and the class
invariant 1 already tolerates and states; (iii) no gate can fail on it — the P2
gate's spec sweep is term-based over the S3-parked phrasings and does not assert
requirement 3's tense.

**Fix, one deletion.** Strike *"§7 requirement 3's device-column phrasing"* from
`plan:480-481` — the sentence already ends cleanly at the preceding member — or,
if it is kept as a signpost, mark it *"(moved to P3.5, r6 M1)"* the way P2.7's
§4.2 entry marks its own device half. Nothing else moves: P0.1 and P3.5 are
already correct and agree.

---

## NIT

**N1 — the same §4.2 clause is covered by two P2.7 members, one of which files
it under §7 and attributes it to a P3 falsifier.** `plan:468-471` reads *"§7's
OWN text that invariant 1 falsifies (r2 I1): the `sysw_class` column definition
paragraph and the `device_probe`/panic-parse clauses ("**the Go test never feeds
one to the parser**" — false once P3.1 lands and the row's probe retires)"*, and
the fold's new member follows immediately at `plan:471-475` with *"§4.2 defect
4's FILE-half clauses … falsified by P2.6's marker retirement, so owned here"*.

Measured: `grep -rn "never feeds one to the parser" design/SPEC_descriptor_input.md`
returns **exactly one hit, `SPEC:390`** — the tail of the very clause the new
member owns, and inside §4.2 (which begins at `SPEC:324`), not §7. So the older
member quotes §4.2 text under a "§7's OWN text" heading, and gives its falsifier
as *"once P3.1 lands"*, while the new member three lines below rules the same
clause falsified by **P2.6**.

Harm is nil in practice — **both members land at P2.7**, so no ownership
conflict exists and the worst case is one implementer amending the same clause
twice as a no-op. It is a Nit rather than a Minor for exactly that reason, and
the older text is pre-existing (r2 I1 era), not something the fold wrote. But
the fold placed its new member directly adjacent to it, which is the moment to
merge them: fold the parenthetical into the new §4.2 member and let the "§7's
OWN text" member keep only what actually lives in §7 (`SPEC:1540`, `:1604-1605`,
both genuinely §7 and both genuinely P2.6-falsified).

---

## What this round did not find

No Critical and no Important. Specifically checked and clean: no member is
built wrong, no gate is made unmeetable, the NEW-I1 fix reaches every place r6
identified plus one more, all four newly cited source lines are exact, and the
manifest census closes with every live copy owned. The two records above are a
stale duplicate line and a stale parenthetical; neither changes what anyone
builds, and neither gates.

The R0 loop's finding counts across seven rounds are 20 → 10 → 9 → 4 → 5 → 5 →
2, with zero Criticals for four rounds and zero Importants for the first time.
**GREEN. The loop is closed** — re-dispatching for reassurance is the failure
mode the proportional re-review rule names, and the remaining Minor is a
one-line deletion that the S2 implementer's own P2.7 reading will meet with
P0.1's ruling three sections earlier.
