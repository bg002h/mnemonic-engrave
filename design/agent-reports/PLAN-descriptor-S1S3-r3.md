# R0 round 3 — `IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md` @ `a1b53e5`

**Target:** the plan at `a1b53e5` (195 lines) **plus the spec fold it is built
on**, `c4d5da9` (`SPEC_descriptor_input.md`, +43/−31). Both files verified
byte-unchanged since those commits (`git diff a1b53e5 HEAD --` on both →
empty; HEAD is `6d2c802`, a continuity-only commit).
**Question, as briefed:** did the two-commit fold close r2's fifteen findings,
and did the fold's own new text introduce defects. Proportional re-review, not
a fresh audit.
**Taken as settled, not re-derived:** r1's and r2's verified-TRUE tables;
everything measured in rounds 1–2; the spec's pre-amendment GREEN; the
overnight mandate.
**Reviewer:** independent context. Read-only on all three repos — nothing
modified, committed or pushed anywhere. This report is the only file written.
**Tools run:** `git`, `grep`, `awk`, `python3` (a fresh span sweep over §6),
`./scripts/plan-staleness-check.sh`, source reads of
`descriptor-mnemonic/crates/md-codec/src/{validate,encode,identity}.rs`.

---

## Counts

| severity | r2 disposition | NEW this round |
| --- | :-: | :-: |
| **Critical** | 2 → both FIXED **in the spec**, one NOT PROPAGATED to the plan | **1** |
| **Important** | 3 → all 3 FIXED | **5** |
| Minor | 7 → all 7 FIXED | 3 |
| Nit | 3 → 2 FIXED, 1 ACKNOWLEDGED | 2 |

**NOT GREEN — 1 Critical / 5 Important.**

The spec commit `c4d5da9` is a strong fold: every one of r2's five blockers is
closed *in the spec*, and I re-traced each one against the real tree rather
than checking for the presence of words. The conjunct-8 rewrite is correct —
the `(xpub, use-site)` predicate matches the primary's source exactly, the
clause-7/8 paste is repaired, the tally recomputes to 37/88/71 with no
arithmetic error at any of the three sites, §6 measures 35 rows, and the span
sweep over all 34 quoted texts finds zero internal identifiers.

**What did not happen is the second half of the same fold.** `a1b53e5` folded
the plan's *findings* but did not mirror the spec's *renumber and arithmetic*:
the plan still specifies 36 gate rows / 87 slots / a 70-row floor and calls
clause 8 a **pair**, and still cites "conjunct 7a" — a name that now returns
**zero hits** in the spec. Following P0.1 as written authors a vector file
**without the `wsh(multi(…))` twin row**, which is the exact row `c4d5da9`
added to close r2's NEW-C1, and P0.2 asserts the pre-amendment totals so the
harness certifies its absence green.

The other four blockers are in the fold's own new text: clause 8's third row
carries a column value that is wrong on one of the two paths it claims; §6's
new row merges two causes the primary explicitly separates and leaves one
variant's operator text unwritten; §5.4's DECIDED tier ruling justifies itself
with a fact the same commit measures false; and the plan's re-pinned staleness
gate examines zero citations.

**Ungated class, as the plan itself now says:** `grep -c '^```'` on the plan →
**0**. Four of this round's six blocking findings were found by *running*
something — the staleness script, a span sweep, a source read of the
validator, a citation extraction. None was findable by re-reading.

**Noted, not counted:** commit `6d2c802`'s message records that the controller
independently found "six plan-side misses of the r2 fold (spec renumber and
arithmetic not mirrored), held for the r3 fold as self-found." That is the same
class as C1 below. It does not close it — the artifact under review still
carries them, and the enumeration below is more complete (ten textual sites).

---

# Critical

## C1 — the spec's renumber and arithmetic were not mirrored into the plan: P0.1 authors 36 gate rows / "clause 8's impossible-wallet **pair**", so the `wsh(multi(…))` twin row — the row that closes r2's NEW-C1 — is never written, and P0.2 certifies the file green without it

**What the spec now says** (`c4d5da9`, all three sites re-derived from the text
as written, not from the commit message):

| site | spec @ `c4d5da9` | plan @ `a1b53e5` |
| --- | :-: | :-: |
| gate-tag minimum (spec 1668) | **37** | **36** (plan L29) |
| minima sum / tag-slots (spec 1651) | **88** | **87** (plan L26, L44) |
| physical-row floor (spec 1650) | **71** | **70** (plan L26, L44) |
| clause 8's cardinality (spec 1623–1631) | **trio** | **pair** (plan L30) |
| §6 row count at P3.1 (measured 35) | **35** | **34** (plan L161) |
| the conjunct's number | **8** | **7a** (plan L26, L101, L115, L130) |

**Machine-checked, from the text as written:**

- §7's gate clause list enumerates 15 + 6 + 2 + 4 + 1 + 3 + 3 + **3** = **37**
  rows (clause 7 restored to three tokens, clause 8 a trio). Matches the tag
  table's `gate` = 37 exactly.
- Minima sum: 4 + 15 + 14 + 1 + 5 + 3 + 3 + 6 + 37 = **88**. 88 − 17 = **71**.
  Both sites in the manifest prose agree. **The spec's arithmetic is exact.**
- §6 data rows: `awk` over 1360–1408 → 37 table lines − header − separator =
  **35**. P2.4's `== 35` is right; P3.1's "all 34 rows" is not.
- `grep -n '7a' SPEC_descriptor_input.md` → **0 hits**.

**Constructed failure.** P0.1 is the plan's first task and the cycle's whole
structuring decision ("the vector file is authored FIRST and is the failing
test the parser is built against"). Its text reads: *"per spec §7 AS AMENDED
(conjunct 7a): ≥ **70** physical rows … `gate` **36**, incl. clause 8's
impossible-wallet **pair**"*. An implementer authors 36 gate rows and, at
clause 8, two rows — the colliding-origin `sortedmulti` and the duplicate-slot
pair. **The colliding-origin `wsh(multi(…))` twin is not written.** P0.2 then
asserts *"87 slots, 70-row floor"*, both of which the 70-row/36-gate file
satisfies, and the P0 gate closes green.

That row is not decoration. It is the entire remedy for r2's NEW-C1 — the
`multi` twins exist **only** on the `--as md1` path, the one path that reaches
the published `md-codec` 0.42.0 lacking F-217/F-218 (r2's verified-TRUE 1 and
2). The spec's own words: *"the colliding-origin `wsh(multi(…))` twin (the
md1-only path — r2's NEW-C1)"*. Dropping it restores exactly the hole the
amendment was written to close, with a green harness on top.

**The other branch is no better.** An implementer who resolves the conflict
toward the spec (P0.1 does say "per spec §7 AS AMENDED") authors 71 rows and 37
gate rows — and then P0.2's *"87 slots"* assertion **reds against a correct
file**, because the minima now sum to 88. There is no reading of the plan under
which P0 closes green on a correct vector file.

**Compounding.** The plan header still reads *"**Spec:** … FINAL GREEN at
`b949d18`"*. `b949d18` predates both amendment commits, so a reader who
resolves "the spec" to the named revision gets a §4.7 with no conjunct 8 at
all. The re-pin note added by the fold moved the *staleness baseline* line and
left this one.

**What must change (fix not prescribed, but it is mechanical).** Ten sites:
L26 (×3 — "conjunct 7a", 70, 87), L29 (36), L30 ("pair" → trio, naming the
`multi` twin), L44 (×2 — 87, 70), L101 ("conjuncts 2–7 + conjunct 7a" →
the spec's "2–8"), L108 (36), L115 ("conjunct-7a"), L130 ("Conjunct 7a"),
L161 (34 → 35), plus the header's spec revision.

---

# Important

## I1 — P1.1 still instructs the implementer to refuse *"with the tree's refusal wording"*; `c4d5da9` changed that to *"refusing with §6's key-identity row"*, and r2's NEW-C2 measured that the tree's wording names no next action — a NORMATIVE §6 requirement

**Plan L101–103, verbatim:** *"`admit.rs` (the seven shapes + the `multi`
md1-path twins + conjuncts 2–7 **+ conjunct 7a's two impossible-wallet checks
(PLAN-r1 C1) with the tree's refusal wording — convergence with the Rust
primary**)"*. And plan L115, the P1 review brief: *"conjunct-7a wording vs the
tree's"* — the review is pointed at the wrong comparand too.

**Spec @ `c4d5da9`, conjunct 8, verbatim:** *"so `me` enforces both HOST-SIDE,
on both `--as` paths, **refusing with §6's key-identity row**."* The
"with the tree's own refusal wording" clause was deleted from the spec by this
very fold. The plan kept it.

**Why it is not a wording nit.** r2's NEW-C2 established, and I re-verified at
`validate.rs`, that the tree's messages are authoring-surface text for a
different tool and **name no next action**. §6's second paragraph is
NORMATIVE: *"every one of them **names a next action**."* §6's new row was
written precisely to supply one (*"Check the export: a duplicated cosigner line
carrying the wrong key is the usual cause."*).

**Constructed failure.** P1.1 is implemented as written: `admit.rs` refuses with
`Error::OriginKeyContradiction`'s tree text. The **P1 gate** — *"all P0-ignored
host assertions un-ignored and green"* — passes, because refusal *texts* are
P2.4's, not P1's. P2 then arrives and P2.4's key-identity test asserts §6's
text; it reds against P1's shipped wording, one phase late, with P2.2 and P2.3
stacked on top. The cheap repair at that point is to bend the test.

**Second half of the same finding: "conjunct 7a" is a dangling name.** Four
plan sites cite it; the spec returns zero hits. An implementer who greps for
their own instruction's referent finds nothing, and P1.1's *"conjuncts 2–7"*
does not match the spec's `multi` bridge, which now reads *"All other conjuncts
(**2–8**) apply to `multi` identically"*.

## I2 — clause 8's third row (the `multi` twin) is specified as `descriptor-refusal` citing conjunct 8, `refusal_row: key-identity`, **"on BOTH `--as` paths"** — but under `--as descriptor` a `multi` form gets conjunct 1's PERMANENT refusal, by a ruling the spec states three times

**Clause 8, verbatim (spec 1623–1631):**

> 8. **the impossible-wallet trio (conjunct 8, PLAN-r1's C1; r2)** — a
>    colliding-origin `wsh(sortedmulti(…))` …, a duplicate-`(xpub, use-site)`
>    slot pair, and the colliding-origin `wsh(multi(…))` twin (the md1-only
>    path — r2's NEW-C1): gate OPEN, `descriptor-refusal` citing conjunct 8
>    (`refusal_row: key-identity`), exit 3, **on BOTH `--as` paths**.

The sentence binds one outcome to all three rows. It is right for the two
`sortedmulti` rows. It is wrong for the `multi` twin on the `--as descriptor`
path.

**What the spec says elsewhere, measured, three sites:**

- §5.1 (line 936): *"conjunct 1's shape test included, so a `multi` form under
  `--as descriptor` gets **conjunct 1's permanent refusal instead, in every
  build** (r15's new-I3)."*
- §11 item 5, fifth case (line 1876): *"a `multi` form with explicit
  `--as descriptor` in the window — conjunct 1's permanent refusal, never the
  window text (r15's new-I3)."*
- §6 has a **dedicated row for it** at line 1374: *"`wsh(multi(…))` under
  `--as descriptor`"*, with its own text about `sortedmulti` vs `multi`.

§4.7 states no precedence among admission conjuncts. A colliding-origin
`wsh(multi(…))` under `--as descriptor` fails conjunct 1 **and** conjunct 8,
and the spec's only stated ruling for that shape/flag pair is conjunct 1's.

**Constructed failure.** P0.1 authors the row from clause 8's text:
`refusal_row: "key-identity"`, `outcome: descriptor-refusal`, `exit_code: 3`,
asserted on both paths. The Rust test runs
`me sysw pack --as descriptor --in <colliding-origin wsh(multi(…))>` and
asserts §6's key-identity text. Correct behaviour per r15's new-I3 prints §6
row 1374's text instead. The test reds against correct code — or the
implementer makes conjunct 8 preempt conjunct 1, which reverses r15's new-I3
and tells an operator to *"check the export for a duplicated cosigner line"*
when the real, permanent blocker is that a `Descriptor` record can never carry
`multi` at all. The second outcome is the worse one: it hides a permanent
refusal behind a repairable-looking one.

**What must be decided.** Either the clause splits the multi row's outcome per
path (`--as md1` → key-identity; `--as descriptor` → conjunct 1's row), or
§4.7 states a conjunct-precedence rule. The row's REQUIRED `refusal_row` field
cannot hold one value across both paths as written.

## I3 — §6's new key-identity row covers TWO distinct causes with one stated text; the duplicate-`(xpub, use-site)` variant's operator text is not written, and the text that IS written is factually false for that cause

**The row, verbatim (spec 1380):**

> two keys declaring **the same origin with different xpubs**, or one
> `(xpub, use-site)` pair in two slots | *"this wallet description contradicts
> itself: keys `@N` and `@M` both claim origin `<fp/path>` but name different
> keys — one origin identifies exactly one key, **so no wallet matches this
> description**. Check the export: a duplicated cosigner line carrying the
> wrong key is the usual cause."* **(The duplicate-pair variant substitutes its
> own sentence naming the two slots and the shared use-site.)**

**§6's own governing premise, first paragraph:** *"The device's parser has
exactly one message for eleven distinct causes (§4.1). **`me` has one per
cause.**"*

**The primary says the same thing, unprompted, in the doc comment the fold
read to get the predicate right** (`validate.rs`, `validate_no_duplicate_key_slots`):

> DISTINCT FROM [`validate_origin_key_consistency`], and the pair is easy to
> conflate: one origin bound to two DIFFERENT keys is IMPOSSIBLE and refused
> as malformed; one key in two slots is merely UNSAFE. **Separate errors,
> because one message explaining both would explain neither.**

**Two consequences, both constructed:**

**(a) The stated text is false for the duplicate cause.** *"no wallet matches
this description"* is true of an origin contradiction. It is false of a
duplicate slot: that wallet exists, derives, and may hold funds — the primary's
comment says *"The script is legal; the wallet is not what it looks like … one
key seated twice lets its holder produce two of the required signatures."* The
row's text never states that risk, which is the only reason the refusal exists.

**(b) The variant's text is unspecified, so a gate certifies implementer
wording.** Clause 8's second row (`duplicate-(xpub, use-site)`) carries
`refusal_row: key-identity`. P2.4 requires *"one named test per row, all 35
rows"* asserting the text; §11 item 4 requires *"a test that reaches it and
asserts the *text*"*. For this row the spec supplies no text — only *"substitutes
its own sentence"*. The implementer writes the sentence, the test binds to it,
and both gates report green. That is r2's NEW-C2 path B, surviving for one of
clause 8's three rows.

**(c) Knock-on, small but real:** the W5 paragraph says *"**The one exception**
is the single-key-wrapper row"*. There are now two rows whose printed text is
not the quoted text (this one and the window-substituted pair the plan already
handles), so P2.4's "verbatim" rule — which the fold correctly amended for the
two §5.3 rows — has an unnamed third case.

## I4 — §5.4's DECIDED FULL-tier ruling for conjunct-8 failures is justified by *"identification is what shows the operator the contradiction"*, which the SAME commit measures false; and the compare prompt it thereby keeps returns a **passing** check on a wallet that cannot exist

**The new text (spec 1176–1179):**

> a wallet that passes conjuncts 2–7 AND whose shape at least one `--as` path
> admits gets **(a conjunct-8 failure stays FULL-tier, DECIDED not accidental —
> its addresses derive, and identification is what shows the operator the
> contradiction; r2's NEW-M1)** the FULL block

**What FULL actually adds over PARTIAL** — §5.4 defines PARTIAL as *"the first
three lines plus the watch-only line — no `wallet-id:`, no `address 0:`, no
compare prompt"*. So FULL adds exactly three things: `wallet-id:`, `address 0:`,
and the compare prompt. The canonical descriptor — the line that *could* show
an operator a duplicated cosigner — is in the first three lines and prints in
**both** tiers. The ruling's stated benefit is therefore delivered by none of
the lines the ruling adds.

**And "identification shows the contradiction" is measurably false on the line
that matters.** Clause 8, written in the same commit, twenty-five lines away:

> **no address fields** — a colliding-origin wallet derives **byte-identical
> addresses to a clean control** (measured, r2's NEW-M5), so the refusal
> assertion itself is the row's witness

The primary's `validate_origin_key_consistency` doc comment says it a third
time: *"every address comparison — including cross-language conformance over
the whole corpus — passes identically whether the origins are right or
nonsense."*

**Constructed failure.** The operator pastes a colliding-origin
`wsh(sortedmulti(…))`. FULL-tier prints `address 0:` followed by *"compare
against your wallet software's first receive address before engraving."* They
do it. **It matches** — byte-identically, because the addresses derive from the
xpubs the descriptor carries, never from the origins it declares. The operator
has just executed the spec's own recommended verification and received a
confirmation on a wallet description that matches no wallet. The refusal then
fires. §5.4's stated reason for the PARTIAL tier is that *"a 'compare before
engraving' prompt would be a wrong instruction on every member"* of that class;
by that same test the prompt is wrong here, and this is the one class where the
comparison is *guaranteed* to pass for the wrong reason.

**What must be decided.** FULL may well be the right tier — the wallet-id does
range over origins and the canonical line is worth showing. But the ruling
needs a reason that survives its own commit's measurement, and the compare
prompt needs an explicit call: suppressed on conjunct-8 failures, or kept with
a sentence saying the comparison cannot discriminate here. Recorded as
Important rather than Minor because it is operator-facing, funds-adjacent, and
the r2-M1 remedy was *"state it, since the sentence enumerates rather than
generalises"* — the fold stated it and stated it wrong.

## I5 — the re-pinned staleness gate examines **zero** citations: the plan carries no `file:line` citations at all, so `plan-staleness-check.sh` returns 0/0/0 against any baseline, at all three phase gates

**The fold's remedy for r2's NEW-M2 (plan L9–13):** *"mnemonic-engrave:
**re-pinned at each phase gate to the spec's CURRENT tip**"*, and the gate text
at three places: *"**Staleness re-validation before P1 dispatch** (M4):
`scripts/plan-staleness-check.sh` against this plan's baselines."*

**Measured, just now:**

```
$ grep -nEo '[A-Za-z0-9_./-]+\.(rs|go|toml|json|md|sh):[0-9]+' \
    design/IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md
(no output)

$ ./scripts/plan-staleness-check.sh design/IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md . $(git rev-parse HEAD)
─── against . at 6d2c802… .. 6d2c802
─── unchanged: 0 ; DRIFTED: 0 ; not in this repo: 0
```

The plan cites no line numbers in any repo. The script's whole mechanism is
*"compares the line at a fixed NUMBER across two revisions"*; with zero
citations it has nothing to compare and reports clean by construction — for
every baseline, at every gate, forever.

**Two ways it cannot fail, stacked.** Even if citations were added, the fold's
own rule re-pins the baseline **at** each phase gate and runs the check **at**
that same gate, so `BASE` = `HEAD` and the script's `BASE..HEAD` window is
empty. r2's M2 correctly diagnosed a baseline aimed at the wrong tree; the
remedy replaced it with a baseline that cannot be wrong because it cannot be
different.

**Why this blocks.** The repo's severity rule keeps *"a gate that cannot fail"*
in the blocking set, and the project rule this gate implements — *"A PLAN'S
GREEN EXPIRES — review each phase plan IMMEDIATELY BEFORE dispatching its
implementer"* — describes the re-validation as a scoped **read** (*"what did the
last phase falsify here?"*), with the script as an optional mechanical aid.
The plan names only the script. So all three phase gates advance on a null
check.

**The irony is load-bearing, not rhetorical.** C1 above is precisely a
"what did the last commit falsify here" defect, in this plan, introduced
between r2 and r3 — and the instrument the plan names as its re-validation
gate could not have seen it.

---

# Minor

**M1 — the spec's status header claims a verification that had not happened
when it was written.** *"§7's gate its clause 8; found by PLAN-r1's C1,
corrected by PLAN-r2, **verified by the plan R0 rounds**."* At `c4d5da9` the
amendment had been *corrected by* r2 (which found 2C/3I in it) and verified by
nobody; this round is the first verification. The r20 half of the header is
**accurate** — I read `R0-descriptor-input-spec-r20-closure.md`: counts are
0C/0I and the verdict line is *"GREEN STANDS."* Claims-class, the 5-of-22
ungated class; state the amendment's verification in the past tense only once a
round has closed on it.

**M2 — the same status header does not mention that the amendment moved the
manifest arithmetic**, which is the one change most likely to falsify a
downstream artifact. It names §4.7, §6 and §7's gate; 36/87/70 → 37/88/71 is
what actually broke the plan (C1). A header that lists the *sections* touched
but not the *numbers* moved cannot serve the propagation check the operator
just made standing policy (`6d2c802`).

**M3 — r1/r2's reports are untracked.** `git status --porcelain` shows
`?? design/agent-reports/PLAN-descriptor-S1S3-r1.md` and `…-r2.md`, while both
fold commits are in history. The standing rule is persist-commit **then**
fold-commit, so that `git diff <report>..<fold>` means something; here the
folds landed and the reports never did. Cheap to fix retroactively (commit them
now, before folding this one), and worth fixing because this cycle is
accumulating exactly the "what changed in response to what" question those
diffs answer.

---

# Nit

**N1 — the §5.4 parenthetical is spliced between "gets" and "the FULL block".**
It parses, unlike r2's NEW-I3 clause-7 splice, but the tier sentence is the one
readers count conjuncts in and it now spans four lines around an interruption.

**N2 — `@N`/`@M` in the key-identity row are an md1-template notation on a row
that binds BOTH paths.** I checked before raising it and the convention has
precedent: the span sweep found `key `@N`` already inside the quotes at spec
1376 and 1395. Those two rows are §5.3 md1-split refusals, where the operator
is on the md1 path; the new row also fires under `--as descriptor`, where no
template exists and nothing in the operator's input carries an `@`. §6's other
positional convention is plain ordinal (*"key `N` is `tpub` … while key 0 is
`xpub`"*). Not a W5 violation and not worth a round; noted so it is a decision
rather than an inheritance.

---

# Disposition of r2's fifteen findings

| r2 | verdict | how re-traced |
| --- | :-: | --- |
| **NEW-C1** (`multi` outside conjunct 7a) | **FIXED in the spec; REOPENED by the plan** | Conjunct 1's bridge now reads *"All other conjuncts (**2–8**) apply to `multi` identically"* (spec 661) and clause 8's trio adds the colliding-origin `wsh(multi(…))` row explicitly. Traced the compose: a colliding-origin `wsh(multi(…))` under `--as md1` → conjunct 1 admits (md1 path) → bridge applies conjunct 8 → *"no two keys may declare the same `(fingerprint, origin path)` with DIFFERENT xpubs"* fails → refused before `md_codec::encode`, exactly as P2.2 requires. **The spec closes it.** The plan does not: P0.1 still says "pair", so the row never gets written — **C1**. And clause 8's own both-paths claim is wrong on the row it added — **I2** |
| **NEW-C2** (no §6 row for the refusal) | **FIXED** | §6 measures **35** data rows (`awk`, 1360–1408: 37 table lines − 2). The new row leads with the verdict (*"this wallet description contradicts itself"*) ✓, names a next action (*"Check the export: a duplicated cosigner line…"*) ✓, uses §6's `<…>` substitution convention for `<fp/path>` ✓. **Span sweep, fresh python over all 35 rows / 34 quoted spans: ZERO hits** for `§`, `F-\d`, `R0 `, `PLAN-`, `walk W`, `conjunct`, `NEW-[CIMN]`, `r\d+'s` inside any quote — W5's internal-identifier rule holds. `refusal_row: key-identity` now has a referent. P2.4's `== 35` matches the measured count. Residuals → **I3** (merged causes, unwritten variant) and **N2** |
| **NEW-I1** (`(xpub, origin)` vs `(xpub, use-site)`) | **FIXED** | Read the tree's predicate directly rather than re-running the encode probe: `crates/md-codec/src/validate.rs`, `validate_no_duplicate_key_slots` — `if xa == xb && a.use_site_path == b.use_site_path`. The spec now reads *"no two slots may carry the same `(xpub, use-site path)` pair … keyed on the USE SITE, not the origin"* and cites `0xbc4ce`. **Exact match with the primary.** The plan never carried the wrong predicate, so nothing to propagate here |
| **NEW-I2** (window substitution vs "verbatim") | **FIXED** | P2.4: *"'Verbatim' means WHAT THIS BUILD PRINTS: the two window-substituted rows are asserted in their SUBSTITUTED form per §5.3's normative substitution"*. Covers exactly the two annotated rows. (A third non-verbatim case now exists — I3(c)) |
| **NEW-I3** (clause 7/8 paste corruption) | **FIXED** | Clause 7 restored whole: three tokens, each with `gate_open`/`outcome`/`exit_code`; `` `[` alone `` has its verdict back. Clause 8 follows as its own bullet. **Tally recomputed from the text as written: 15+6+2+4+1+3+3+3 = 37**, matching the tag table. Minima sum 88, floor 71, both stated twice and both exact |
| NEW-M1 (§5.4's `2–7` accidental) | **ADDRESSED, defectively** | The ruling is now explicit and marked DECIDED — but its stated reason is false and the prompt it keeps mis-fires → **I4** |
| NEW-M2 (baseline predates the amendment) | **ADDRESSED in form, vacuous in effect** | Baseline line rewritten; spec status header now records the amendment (and the r20 verdict — accurate, M1). But the named instrument examines zero citations → **I5** |
| NEW-M3 (fork ritual for absent protection) | **FIXED** | Both sites: P0.3 *"Push: plain `git push -u origin seam/descriptor-vectors`"* with the measured 404 cited; P3.4 *"pushed plain (no protection on the fork)"* |
| NEW-M4 (`wallet_id`'s Go domain) | **FIXED** | P0.1 now scopes it: *"carried by MULTISIG rows at the device-default use-site only — the Go route's measured domain (`EncodeMultisig` hard-codes `<0;1>/*`, no single-sig arm)"* |
| NEW-M5 (`address_0` can't witness clause 8) | **FIXED, both sides** | Spec clause 8: *"**no address fields** … the refusal assertion itself is the row's witness"*. Plan P0.1: *"Clause-8 rows carry NO address fields"* with the measurement cited |
| NEW-M6 (fable consult unconditional) | **FIXED** | P1.0 applies both mandate tests in its own text: it GATES (phase-owned, not deferrable) and is not an unsettleable funds risk, with the reason given |
| NEW-M7 (unmerged fork branch unowned) | **FIXED** | `FOLLOWUPS.md:14649` — F-425, owning phase *"the operator's fork-merge decision, at or before S2"*, tags `#fork #vectors #ci`. S2 is a later phase than S1/S3, so the park is legitimate under the burndown rule. P3.4 names it |
| NEW-N1 (`7a.` not a list marker) | **FIXED** | §4.7's list markers now run `1.`…`8.` in order (2–8 verified by `awk`); conjunct 8 is a proper eighth item after item 7 |
| NEW-N2 (build gate is a no-op) | **ACKNOWLEDGED, still true** | `grep -c '^```'` on the plan → **0**. The plan now carries a "What the build gate does not cover here" section naming the ungated class and instructing reviewers to execute commands and resolve paths. That is the right response; the four executed checks in this round are the countermeasure working |
| NEW-N3 (P2.1's flag unplaced) | **FIXED** | *"**P2.1's flag skeleton → P2.2 → P2.3 → P2.1's window text → P2.4**"* |

---

# Verified TRUE this round — do not re-derive in round 4

| # | claim | how checked | verdict |
| --- | --- | --- | :-: |
| 1 | The primary's duplicate rule is `(xpub, use_site_path)` | `validate.rs`, function body read: `if xa == xb && a.use_site_path == b.use_site_path` | ✓ spec matches |
| 2 | §6 is **35** data rows | `awk` over 1360–1408: 37 `\|`-lines − header − separator | ✓ P2.4's 35 is right; P3.1's 34 is stale |
| 3 | No internal identifiers in any §6 quoted text | fresh python span sweep, 34 spans, regex over `§ F-\d R0 PLAN- walk W conjunct NEW- r\d's` → 0 hits | ✓ W5 holds |
| 4 | 3 of 35 §6 rows carry no `*"…"*` span (1364, 1365, 1370) | same sweep | ✓ pre-existing, was true at 34 too |
| 5 | Gate clause tally = **37** | enumerated from the clause text: 15+6+2+4+1+3+3+3 | ✓ matches the tag table |
| 6 | Minima sum = **88**, floor = **71** | 4+15+14+1+5+3+3+6+37; 88−17 | ✓ spec exact at all three sites |
| 7 | The tag table has 9 tags | counted | ✓ the plan's "9 tags" survives |
| 8 | `grep '7a'` on the spec → 0 hits | `grep -n` | ✓ the plan's four citations dangle |
| 9 | The plan carries **zero** `file:line` citations | `grep -nEo '…\.(rs\|go\|toml\|json\|md\|sh):[0-9]+'` → no output | ✓ I5's premise |
| 10 | `plan-staleness-check.sh` returns 0/0/0 | run against HEAD | ✓ gate cannot fail |
| 11 | The r20 closure report says 0C/0I and "GREEN STANDS" | report head read | ✓ header claim accurate |
| 12 | The tree's `encode` calls both new validators | `encode.rs:118` `validate_origin_key_consistency`, `:120` `validate_no_duplicate_key_slots` | ✓ |
| 13 | **`compute_wallet_policy_id` does NOT call the F-217/F-218 validators** | `identity.rs:186` body read — canonicalize → tree bytes → `expand_per_at_n` → records | ✓ so a conjunct-8 failure's `wallet-id:` line computes fine on either crate. **Do not chase an "it errors after F-424" branch — it does not** |
| 14 | §6 has a dedicated `wsh(multi(…))` under `--as descriptor` row | line 1374, read whole | ✓ I2's alternative referent |
| 15 | The `--as descriptor` + `multi` → conjunct-1 ruling is stated 3× | §5.1:936, §11 item 5:1876, §6:1374 | ✓ I2's premise |
| 16 | Plan and spec byte-unchanged since `a1b53e5`/`c4d5da9` | `git diff a1b53e5 HEAD --` on both files → empty; HEAD `6d2c802` is continuity-only | ✓ reviewed the right bytes |
| 17 | F-425 exists with an owning phase and tags | `FOLLOWUPS.md:14649` | ✓ |
| 18 | The plan carries 0 fenced code blocks | `grep -c '^```'` | ✓ NEW-N2 still true, now acknowledged |
| 19 | `@N` inside §6 quotes has precedent | span sweep: spec 1376 and 1395 | ✓ N2 is a nit, not a W5 finding |

---

# Verdict

**NOT GREEN — 1 Critical / 5 Important.**

The spec fold is right. Every one of r2's five blockers is genuinely closed in
`c4d5da9`, and the two that were hardest to get right — the `(xpub, use-site)`
predicate and the 37/88/71 cascade — are exact against the primary's source and
against the clause text respectively. I re-derived both rather than trusting the
commit message.

**Every blocking finding this round is a propagation defect of that fold, and
they share one shape: the amendment was completed in the spec and stopped at
the spec's edge.** The plan's arithmetic and the conjunct's *number* never
crossed over (C1, I1); clause 8's new third row inherited a claim written for
the two rows beside it (I2); §6's new row inherited a text written for one of
the two causes it covers (I3); §5.4's new ruling inherited a justification that
its own sibling paragraph measures false (I4); and the staleness gate that
exists to catch exactly this class was re-pinned into vacuity (I5).

This is the same class r2 named — *"a diff falsifies text it never touches"* —
one commit further on, and it is worth saying plainly that the operator's new
propagation-check directive (`6d2c802`) is aimed at precisely the right thing.
Applying it to *this* fold, before dispatching r4, is the cheapest possible
close: C1 and I1 are ten mechanical edits, and I2/I3/I4 are each one decision
plus a sentence.

**What round 4 should verify** is not the conjunct-8 design — that is settled
and correct — but whether the plan now says the same numbers and the same
conjunct name as the spec, whether clause 8's multi row states a per-path
outcome, whether the duplicate-slot refusal has written operator text, and
whether §5.4's ruling carries a reason that survives clause 8's own
measurement.
