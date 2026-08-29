# R0 round 4 — `IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md` @ `13188e3`

**Target:** the plan (208 lines) and the co-amended `SPEC_descriptor_input.md`
(1894 lines), both at `13188e3` — the single commit that folded PLAN-r3.
Verified I reviewed the right bytes: `git status --porcelain` → empty, `git diff
13188e3 HEAD --` on both files → empty, HEAD **is** `13188e3`.
**Question, as briefed:** did `13188e3` close PLAN-r3's eleven findings, and did
the fold's own new text introduce defects. Proportional re-review, not a fresh
audit.
**Taken as settled, not re-derived:** r1's, r2's and r3's verified-TRUE tables;
the conjunct-8 design; the published-crate gap; the overnight mandate.
**Reviewer:** independent context. Read-only on all three repos —
`mnemonic-engrave`, `descriptor-mnemonic` (`6864f377`), the fork (`main` =
`d402f18`, checkout at `0b656d7`). Nothing modified, committed or pushed
anywhere. This report is the only file written.
**Tools run:** `git`, `grep`, `awk`, `python3`, `sed`,
`./scripts/plan-cite-gate.sh`, `./scripts/plan-staleness-check.sh` (three
repos), `./scripts/plan-fold-sweep.sh` (both artifacts, explicit mode).

---

## Counts

| severity | r3 disposition | NEW this round |
| --- | :-: | :-: |
| **Critical** | 1 → **PARTIALLY FIXED** (9 of 10 sites + the header) | **0** |
| **Important** | 5 → 3 FIXED, 2 **PARTIALLY FIXED** | **4** |
| Minor | 3 → all 3 FIXED | 6 |
| Nit | 2 → both FIXED | 2 |

**NOT GREEN — 0 Critical / 4 Important.**

The fold is good work and the two hardest calls in it are right. §5.4's tier
re-decision to PARTIAL is correct and its stated reason now survives its own
sibling paragraph's measurement; §6's split into two rows matches what the
primary's own doc comment says about the two causes, and both new texts obey
every W5 rule I can check. The status header's new arithmetic claim —
*"34→37 gate / 85→88 slots / 68→71 floor"* — is **exactly right**, verified
against `9297a29^` and HEAD rather than against the commit message.

**Every one of this round's four blocking findings was found by running
something**, not by re-reading: the fold sweep (I2), the citation gate (I3), the
staleness gate against three repos (I3), and `git log` (I4). Two of them are the
plan's own named gates reporting red — one of which prints, in its own words,
*"fix before review"*.

**The shape has changed since r3.** r3's blockers were all one class ("the
amendment stopped at the spec's edge"). This round's are three classes: one
genuine text contradiction the fold created (I1), one site of r3's C1 that the
sweep reported clean and is not (I2), and two records/gate defects around the
fold rather than in it (I3, I4).

---

# Important

## NEW-I1 — §5.4's `multi`-in-the-window parenthetical still says FULL-tier, which contradicts the fold's own conjunct-8 → PARTIAL re-decision on exactly the row clause 8 added — and reinstates the compare-prompt-passes-on-an-impossible-wallet harm r3's I4 was folded to remove

**The fold's new rule (spec 1180–1186), verbatim:**

> A wallet NO path admits gets the PARTIAL block — including a conjunct-8
> failure, RE-DECIDED per PLAN-r3's I4 (reversing r2-M1's accidental FULL): its
> addresses derive but are byte-identical to a clean control (measured), so a
> compare prompt would PASS on an impossible wallet, actively reassuring …

**Fifteen lines later, untouched by the fold (spec 1195–1198), verbatim:**

> (A `multi` input in the window is FULL-tier — derivable, spendable,
> md1-packable when its use-site paths are md1-representable — and stripping its
> identification would blind the operator at the decision their refusal asks.

**The intersection is not hypothetical — it is clause 8's third row.** §7 clause
8 enumerates *"the colliding-origin `wsh(multi(…))` twin"*, and the same clause
rules that under `--as descriptor` it *"gets conjunct 1's permanent refusal
first"* — i.e. it is a `multi` input **in the window**.

**Every one of the parenthetical's own qualifiers is satisfied by that row**, so
the qualifiers do not rescue it:

- *derivable* — yes; 1182–1183 says so in the same paragraph (byte-identical to
  a clean control).
- *spendable* — yes; conjunct 8 is about key identity, not the threshold.
- *md1-packable when its use-site paths are md1-representable* — the clause-8
  rows are `<0;1>/*`-shaped; the paths are representable. (What refuses them is
  conjunct 8, which is not a path property.)

**Two rules, same paragraph, opposite answers for one enumerated vector row.**
By 1180 it is PARTIAL. By 1195 it is FULL. 1195 is the more *specific* rule — it
names `multi` inputs in the window by shape — so the natural reading picks it,
and the natural reading is the wrong one.

**Constructed failure.** P2.3 implements §5.4's two tiers. The operator pastes
the colliding-origin `wsh(multi(2, …/<0;1>/*))` from clause 8 and runs
`me sysw pack --as descriptor --in <file>`. Under 1195 the FULL block prints
`wallet-id:`, then `address 0:`, then *"compare against your wallet software's
first receive address before engraving."* The operator compares. **It matches**,
byte-identically — the addresses derive from the xpubs, never from the origins.
`me` has just handed a confirmation for a wallet description that matches no
wallet, and only then fires conjunct 1's refusal. That is the exact sequence
1181–1184 was written to eliminate, surviving on the one row of the three the
same fold added.

**What must be decided.** Either the parenthetical excepts conjunct-8 failures
(*"a `multi` input in the window that passes conjunct 8 is FULL-tier"*), or the
1180 rule states which wins. One clause either way. I do not prescribe the fix —
but note that both prior attempts to state this parenthetical are recorded at
1209 as having *"manufactured a false claim"*, so the safer edit is the narrowing
one at 1195, not a new general rule at 1180.

## NEW-I2 — the tenth site of r3's C1 was not folded: plan P3.1 still reads "all 34 rows", §6 now measures **36**, and the same plan asserts `== 36` twenty-four lines above. The controller's pre-review sweep reported old forms at zero; `plan-fold-sweep.sh` disagrees

**r3's C1 enumerated ten sites.** Nine landed. This is the tenth, verbatim from
r3: *"L161 (34 → 35)"*.

**Plan L163–164 @ `13188e3`, verbatim:**

> - **P3.1** §11's S3-bound items discharged and named (items 2, 3, 4 — all
>   34 rows —, 5); items 1/6 recorded as S2-parked (F-418).

**Measured, three ways:**

```
$ python3 — count table lines in §6, minus header and separator
table lines 38 ; data rows 36

$ ./scripts/plan-fold-sweep.sh design/IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md \
    --terms 'conjunct 7a' '87' '70' 'pair' 'b949d18' '35 rows' '34 rows' '36 gate'
STILL PRESENT  34 rows    1 occurrence(s)
                 164:  34 rows —, 5); items 1/6 recorded as S2-parked (F-418).
─── 2 term(s) still present.
```

(The other survivor, `pair` at L48, is the §7 overlap pair and is correct.)

**It is now stale by two, not one.** r3 asked for 34 → 35; the same fold then
split §6's key-identity row in two, so the right number is **36** — which the
plan already states at P2.4 (*"all 36 rows"*, *"asserts its own row-test count
== 36"*). One document, two counts, twenty-four lines apart.

**Constructed failure.** P3.1 is the acceptance ledger — the step that writes
"§11 items 2, 3, 4, 5 discharged" into the CHANGELOG, FOLLOWUPS and continuity
at P3.3. Nothing gates it: P2.4's `== 36` is a *test* assertion inside the test
file, and P3.1's count is prose an implementer transcribes. So the cycle ships a
record claiming §11 item 4 was discharged over 34 rows when there are 36, and
the two rows the amendment exists for — the key-identity pair — are the two the
number excludes. A future reader reconciling the ledger against §6 finds a
two-row gap and no way to tell whether the tests or the ledger is wrong.

**Second-order, and the reason I am not filing this as a Minor.** The
controller's brief states the pre-review sweep found *"old forms at zero"*. It
did not. A propagation sweep that reports clean while a named superseded form
survives is the failure mode the sweep exists to prevent, and it was the
controller's own evidence for closing a Critical. The command above is the one
that finds it, in under a second.

## NEW-I3 — the new load-bearing-anchors block cites `descriptor-mnemonic` `src/encode.rs:118`/`:120`; that path does not exist in that repo. The citation gate reports "2 unresolvable citation(s) -- fix before review", and the descriptor-mnemonic staleness gate examines **zero** citations — the block's own stated purpose, unmet for the one repo the cycle is about

**Plan L188–190, verbatim:**

> `descriptor-mnemonic` `src/encode.rs:118` and `src/encode.rs:120` (the two
> validator calls the published crate lacks) · … Each verified this cycle; the
> per-phase staleness re-check now has citations to examine.

**Measured.** `descriptor-mnemonic` has **no top-level `src/`**. It has three
`encode.rs` files:

```
./crates/md-codec/src/encode.rs
./crates/md-cli/src/cmd/encode.rs
./vendor/bitcoin/src/consensus/encode.rs
```

**The line CONTENT is right** — I resolved it at the pinned baseline
`6864f377` (= that repo's HEAD), `crates/md-codec/src/encode.rs`:

```
118:    crate::validate::validate_origin_key_consistency(d)?;
120:    crate::validate::validate_no_duplicate_key_slots(d)?;
```

So the claim *"the two validator calls"* is true. **The citation is not**: as
written it names no file in any of the three repos.

**Both named gates report it, and both were run before this review:**

```
$ ./scripts/plan-cite-gate.sh design/IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md
  ok    crates/me-cli/src/main.rs:335        | const EXIT_OK: i32 = 0;
  ok    crates/me-cli/src/sysw/mod.rs:205    | pub fn classify(record: &str) -> record::Class {
  ok    crates/me-cli/tests/codex32_seam.rs:60
  ok    crates/me-cli/tests/sysw_cli.rs:1928
  ok    md/encode_multisig.go:112 · md/walletpolicyid.go:138 · nonstandard/parse.go:36
  FAIL  src/encode.rs:118                    no such file
  FAIL  src/encode.rs:120                    no such file
== RESULT ==   2 unresolvable citation(s) -- fix before review

$ ./scripts/plan-staleness-check.sh <plan> /scratch/code/shibboleth/descriptor-mnemonic 6864f377
─── unchanged: 0 ; DRIFTED: 0 ; not in this repo: 9
```

**The framing in the brief does not hold.** The brief calls these *"cross-repo
descriptor-mnemonic citations"* — i.e. FAILs inherent to the gate's reach. They
are not: the **fork's three cross-repo Go citations resolve `ok`** in the same
run, and the staleness script resolves against whatever repo root it is given.
These two fail because the path is wrong, not because they are cross-repo. With
`crates/md-codec/` prefixed, the descriptor-mnemonic staleness gate goes from 0
live citations to 2. (The cite gate would still not resolve them — it knows two
repo roots — but that is a gate-coverage fact to state, not a wrong citation.)

**Why it blocks rather than being a typo.** Two of the repo's own blocking
categories, both self-inflicted:

1. **A red gate reaching a reviewer.** The gate's verdict line is literally
   *"fix before review"*, and the standing rule is that nothing machine-checkable
   reaches a reviewer unchecked. I am the second consecutive round to spend
   budget on the citation layer.
2. **A gate that cannot fail.** r3's I5 was Important precisely because the
   staleness check examined zero citations. The fold's remedy works for
   `mnemonic-engrave` (4 live) and the fork (3 live, and I confirmed a genuinely
   non-empty window: `d402f18 .. 0b656d7`, *"unchanged: 3"*). It delivers **zero**
   for `descriptor-mnemonic` — the repo holding F-217/F-218, the published-crate
   gap, and F-424, which is to say the only drift the conjunct-8 amendment
   exists to guard against.

## NEW-I4 — the fold commit's message is a **byte-identical copy** of an unrelated earlier commit's, describing eight findings this fold did not fold and asserting gate results this fold did not produce — while its cite gate is red

**Measured:**

```
$ git log --all --oneline --grep='per-key quantifiers'
13188e3 spec: fold R0 r3 -- per-key quantifiers, md1_admits, the discriminator
8409616 spec: fold R0 r3 -- per-key quantifiers, md1_admits, the discriminator

$ diff <(git log -1 --format=%B 8409616) <(git log -1 --format=%B 13188e3)
(no output — IDENTICAL)

$ git show --stat 8409616   → 1 file changed, +97 −39   (spec only)
$ git show --stat 13188e3   → 2 files changed, +49 −26  (plan + spec)
```

`8409616` is the **spec's own** R0 round-3 fold. `13188e3` is the **plan's**
R0 round-3 fold. They share a message describing the former: per-key
quantifiers, the `md1_admits` column, the whole-input discriminator, seven §6
rows joined to a table, `bip380.go:476,489`. **None of that is in `13188e3`'s
diff**, which is C1's ten sites, I1's P1.1 wording, I2's clause-8 scoping, I3's
row split, I4's tier re-decision, and the anchors block.

**The false claim that matters is the gate line**, carried over verbatim:

> Gates: cite gate resolves all Go citations (5 cross-repo Rust citations
> hand-verified …); sweep clean including the two patterns r3 caught …

For this fold, measured above: **the cite gate does not resolve** (2 FAILs,
NEW-I3) and **the sweep is not clean** (`34 rows` survives, NEW-I2). So the only
recorded gate evidence for this fold is another fold's green, and it is green
where this one is red.

**Why it blocks.** The repo's severity rule keeps *"defects in what a tool
claims to have done"* in the blocking set, and the constellation rule the commit
pair implements is explicit: the persist commit is the report, **the fold commit
carries the gate's output in its message**, so `git show <fold>` answers "what
changed in response to what, and what proved it". Here `git show 13188e3`
answers a different round's question with a different round's evidence. The diff
survives — `git diff 0d3e3a8..13188e3` is still authoritative and is what I
reviewed — so the damage is bounded to the record, which is why this is
Important and not Critical.

**Remedy is a records fix, not an authorship one.** `master` is unpushed in this
window, so the message can be rewritten in place; if the tip has since been
staged, a follow-up commit or `git notes` carrying the true fold summary and the
two gates' real output closes it. Nothing in the plan or spec text needs to
change for this finding.

---

# Minor

**NEW-M1 — §4.7 conjunct 8 still says "§6's key-identity row", singular, after
the fold split it into two.** Spec 723: *"so `me` enforces both HOST-SIDE, on
both `--as` paths, refusing with §6's key-identity row."* §6 now carries two
rows with two different texts (1385, 1386), §7 clause 8 names two slugs
(`key-identity`, `key-identity-duplicate`), and plan P1.1 says *"§6's TWO
key-identity rows"*. §4.7 is the NORMATIVE admission predicate and is the
section an implementer reads to build `admit.rs`; it is now the only site still
implying one message for both causes — the exact conflation the primary's doc
comment warns against (*"one message explaining both would explain neither"*).
Not Important because the plan's own instruction is right and P2.4's per-row
tests (36) force two messages, so a single-message implementation cannot close
the gate.

**NEW-M2 — F-424's FOLLOWUPS entry still cites "conjunct 7a", twice.**
`design/FOLLOWUPS.md:14642` and `:14645`: *"Interim: spec conjunct 7a has `me`
enforce both host-side"*, *"keep conjunct 7a's refusal (now double-enforced)"*.
`grep 'conjunct 7a'` returns **0 hits** in the spec and 0 in the plan, so the
entry P3.3 reconciles names a conjunct that no longer exists. Same propagation
class as r3's C1, one file outside the sweep's scope. Cheap: two tokens.

**NEW-M3 — §7 clause 8 says the `multi` twin's conjunct-8 refusal *"binds the
`--as md1` path ONLY"*, and never names the `--as`-omitted invocation — which is
the one the row's gate fields are asserted against.** §7's gate bullet:
*"Every row carries the four gate fields, asserted by the Rust test against the
real `--as`-omitted invocation."* With `--as` omitted, conjunct 1 admits `multi`
(the md1-path twins), conjunct 8 then refuses, neither path carries, and §5.4's
carriage rule fires the input's own refusal at exit 3 — so `refusal_row:
key-identity` is correct for that row. The clause's outcome sentence gets there
(*"Gate OPEN, `descriptor-refusal` …, exit 3"* is unscoped; only the *"on both
`--as` paths"* tail is scoped to the first two rows), which is why this is Minor
and not a blocker. But "ONLY" invites the opposite reading, under which the
`--as`-omitted case becomes `as-decides` at exit 2 — a two-option menu whose
options both refuse, which §5.4 names as *"the dead-flag defect §5.1's choice
block was ruled never to be"*. One clause naming the third invocation removes
the reading.

**NEW-M4 — the duplicate-slot row's new text fixes r3's falsity but still does
not state the risk the refusal exists for.** Spec 1386: *"keys `N` and `M` are
the same key at the same derivation — a threshold that needs the same key twice
is not the multisig this file describes. Remove the duplicate line, or supply
the missing cosigner's key."* This is a clean fix of I3(a) — *"no wallet
matches this description"* is gone, and it obeys every W5 rule (verdict first,
no internal identifiers, an executable next action). What it still omits is the
primary's own reason: *"one key seated twice lets its holder produce two of the
required signatures"* — a 2-of-3 that is really 1-of-2 for that holder. §6's
sibling rows do state their stakes (*"Funds sent to this wallet would be
unspendable"*; *"treat them as at risk now"*). Minor rather than Important
because the input is refused, so no funds are ever exposed by the omission —
only the operator's understanding of why.

**NEW-M5 — the mnemonic-engrave staleness baseline is still re-pinned and read
at the same moment, so its window can be empty.** Plan L10–15 keeps *"re-pinned
at each phase gate to the spec's CURRENT tip"*, and each gate says *"staleness
re-check before P<n+1> dispatch"*. Measured with BASE = HEAD: *"unchanged: 4 ;
DRIFTED: 0"* over an empty `13188e3..13188e3` window. If the re-pin happens at
gate *n* and the read at gate *n+1*, the window covers phase *n*'s own commits
and the gate is live — which is almost certainly the intent, and is why this is
Minor rather than a reopening of r3's I5. One clause fixes it: re-pin **after**
running the check, not before.

**NEW-M6 (pre-existing; recorded so it is not lost, outside this round's
proportional scope) — `refusal_row`'s slug vocabulary is defined for 2 slugs out
of 20-plus rows that need one.** `grep -n refusal_row` on the spec returns
exactly two lines: the schema definition (1526) and clause 8 (1636). The schema
says *"a slug naming the §6 row whose text the test asserts — the slug-to-text
binding lives with §11 item 4's per-row text tests"*, and gate clause 1 requires
eight `descriptor-refusal` rows *"each naming its §6 row"* with no naming
convention anywhere. P0.1's author invents ~20 slugs; P2.4's tests, written two
phases later by the same or another implementer, must match them exactly. Not
fold-introduced and not raised by r1/r2/r3, so it does not gate this round —
but it belongs in P0.1's brief, because P0 is where the vocabulary gets fixed.

---

# Nit

**N1 — the fold left two over-long lines that break the surrounding wrap.**
Spec 1186 (*"colliding lines. The PARTIAL block: the first three lines plus the
watch-only line — no `wallet-id:`, no"*) and plan L141 both run well past the
~76-column wrap both files otherwise hold. Cosmetic, but it is also the visual
signature of an inserted clause, which is where the next reader's eye should
NOT be drawn away from.

**N2 — §5.4's closing paragraph now generalises over a tier it does not
describe.** Spec 1268–1272: *"the operator can see — and CHECK, by one address
comparison — that the thing they are about to engrave is the wallet they meant,
even when this build's answer is a refusal."* PARTIAL-tier refusals print no
address, so the sentence has always been a FULL-tier claim; the fold widened
PARTIAL's membership without touching it. Pre-existing, and the tier rules 90
lines above are unambiguous.

---

# Disposition of r3's eleven findings

| r3 | verdict | how re-traced |
| --- | :-: | --- |
| **C1** (spec renumber/arithmetic not mirrored) | **PARTIALLY FIXED — 9 of 10 sites + the header** | Verified each site in the current text: L28–29 `conjunct 8` / **71** / **88**; L31 `gate` **37**; L32 **trio**; L46 *"88 slots, 71-row floor"*; L103 *"conjuncts 2–7 + conjunct 8's …"* (equivalent to the spec's 2–8 and complete — accepted); L111 **37** gate rows; L118 *"conjunct-8 wording"*; L134 *"Conjunct 8 refuses BEFORE encoding"*; header's `b949d18` removed and replaced with the CURRENT-tip rule. `grep 'conjunct 7a'` → 0 in both files. **L164's "34 rows" is untouched** → **NEW-I2**. I re-derived the arithmetic from the clause text rather than trusting r3: 15+6+2+4+1+3+3+3 = **37**; 4+15+14+1+5+3+3+6+37 = **88**; 88−17 = **71**; §6 = **36** data rows |
| **I1** (P1.1 pointed at the tree's wording) | **FIXED** | P1.1 now reads *"refusing with §6's TWO key-identity rows (the tree's text names no next action; §6's rules bind)"*, and the P1 review brief re-aimed to *"conjunct-8 wording vs §6's two key-identity rows"*. The dangling `7a` name is gone from all four sites |
| **I2** (clause 8's both-paths claim wrong for the `multi` twin) | **FIXED** | The fold took r3's first offered remedy — per-path scoping in the clause, rather than a new §4.7 precedence rule. Re-checked the three ruling sites it defers to and all three say `--as descriptor` explicitly: §5.1 (940), §6 (1379), §11 item 5 (1886). Residual → **NEW-M3** (the `--as`-omitted invocation, the one the gate fields assert against, is not named). Noted, not counted: the precedence now lives in §7 and §5.1, while §4.7 — which P1.1's `admit.rs` follows — states no precedence; §5.1's general ruling covers it |
| **I3** (one text for two causes; variant unwritten) | **FIXED** | §6 split into two rows (1385, 1386), each with its own text, `EXIT_REFUSED (3)`, both `--as` paths, and its own annotation. (a) *"no wallet matches this description"* now appears only on the origin-contradiction row, where it is true. (b) The duplicate variant's text is **written**, so no gate certifies implementer wording. (c) resolved by construction — a row with its own text does not substitute, so P2.4's "verbatim" rule keeps exactly the two §5.3 window cases. W5 re-checked on both new quotes: verdict first ✓, zero internal identifiers ✓, executable next action ✓. Residuals → **NEW-M1**, **NEW-M4** |
| **I4** (FULL-tier ruling justified by a fact the same commit measures false) | **FIXED** | The ruling is **reversed**, not re-argued: conjunct-8 failures are now PARTIAL, and the stated reason is the measurement itself (*"a compare prompt would PASS on an impossible wallet, actively reassuring"*). Composition checked on every §5.4 surface the brief named — **tier test**: FULL = *"passes conjuncts 2–8 AND … at least one `--as` path admits"*, PARTIAL = *"NO path admits"*; conjunct 8 is `--as`-independent, so a conjunct-8 failure lands in PARTIAL under both clauses — exhaustive and non-overlapping ✓. **Class description**: the enumeration does not list conjunct-8 failures, but *"The class is defined by the conjuncts, not by an inventory"* licenses it and 1181 states the membership explicitly ✓. **"a wrong instruction on every member"**: holds — for this class the comparison is guaranteed to pass for the wrong reason ✓. **Follower rules**: unchanged and still decoupled (*"any tier may precede any follower"*) ✓. **Identification lines**: PARTIAL prints no `wallet-id:`, which makes r3's verified-TRUE 13 (`compute_wallet_policy_id` does not call the validators) moot rather than load-bearing ✓. **§7's no-address rule**: consistent — PARTIAL-tier wallets carry no address line, clause-8 rows carry no address fields, and the Go/Rust address assertions only fire on fields a row carries ✓. **One surface disagrees** → **NEW-I1** |
| **I5** (staleness gate examines zero citations) | **PARTIALLY FIXED** | Materially better and measurably live in one repo: 7 citations added, all 7 resolving (`plan-cite-gate.sh` → `ok`), and the fork check runs a genuinely non-empty window (`d402f18 .. 0b656d7`, *"unchanged: 3 ; DRIFTED: 0"*) — the gate CAN fail now. Two residuals: `descriptor-mnemonic` still examines **0** because the two anchors name a path that does not exist → **NEW-I3**; and the re-pin/read ordering still permits BASE = HEAD → **NEW-M5** |
| **M1** (header claimed a verification that had not happened) | **FIXED** | Header now reads *"**under verification by the plan R0 rounds in flight** (past-tense only when a round closes on it)"* |
| **M2** (header omits the moved arithmetic) | **FIXED, and the numbers are exact** | Header: *"moving the manifest arithmetic **34→37 gate / 85→88 slots / 68→71 floor**, the numbers every downstream artifact must re-pin"*. Verified against the tree, not the message: `9297a29^` → `gate` **34**, *"at least **68** physical rows (the minima sum to **85** tag-slots"*; HEAD → **37**, **71**, **88**. All six numbers correct |
| **M3** (r1/r2 reports untracked) | **FIXED** | `0d3e3a8` persists r1, r2 **and** r3 (1523 lines, three files), and `git status --porcelain` is now empty. Late, and the commit subject owns it (*"LATE persist … process violation owned"*). Order is right for this round: persist `0d3e3a8` → fold `13188e3` |
| **N1** (§5.4 parenthetical spliced mid-sentence) | **FIXED** | The tier sentence reads straight through: *"…at least one `--as` path admits gets the FULL block — every line below."* The re-decision moved to its own clause on the PARTIAL side. New wrap residue → **N1** |
| **N2** (`@N`/`@M` on a both-paths row) | **FIXED** | Both new rows use plain ordinals — *"keys `N` and `M`"* — matching §6's other positional convention. `@M` → 0 hits in the spec; the surviving `@N` hits (1381, 1401) are the pre-existing §5.3 md1-split rows where r3 confirmed the template notation is correct |

---

# Verified TRUE this round — do not re-derive in round 5

| # | claim | how checked | verdict |
| --- | --- | --- | :-: |
| 1 | §6 is **36** data rows | python3 over 1365–1402: 38 table lines − header − separator | ✓ P2.4's 36 is right; P3.1's 34 is stale by two |
| 2 | Gate clause tally = **37** | enumerated from clause text: 15+6+2+4+1+3+3+3 | ✓ matches the tag table |
| 3 | Minima sum **88**, floor **71** | 4+15+14+1+5+3+3+6+37; 88−17 | ✓ exact at both manifest sites |
| 4 | Pre-amendment values were 34 / 85 / 68 | `git show 9297a29^:…` — tag table and manifest prose | ✓ the header's `34→37 / 85→88 / 68→71` is exact |
| 5 | `grep 'conjunct 7a'` → 0 in plan, 0 in spec | `grep -n` both files | ✓; 2 hits survive in `FOLLOWUPS.md` (NEW-M2) |
| 6 | `34 rows` survives at plan L164 | `plan-fold-sweep.sh --terms …` | ✓ NEW-I2 |
| 7 | Spec fold sweep is clean | `plan-fold-sweep.sh` on 7 superseded spec terms | ✓ no named superseded spec term survives |
| 8 | 7 of 9 plan citations resolve; 2 FAIL | `plan-cite-gate.sh` | ✓ verdict line: *"2 unresolvable citation(s) -- fix before review"* |
| 9 | `descriptor-mnemonic` has no top-level `src/`; three `encode.rs` files | `ls`, `find -name encode.rs` | ✓ NEW-I3's premise |
| 10 | `crates/md-codec/src/encode.rs:118`/`:120` **are** the two validator calls at `6864f377` | `sed -n '110,126p'` | ✓ the plan's claim is true; only the path is wrong |
| 11 | Staleness: me-cli 4 live / fork 3 live / descriptor-mnemonic **0** | `plan-staleness-check.sh` ×3 | ✓ fork window `d402f18..0b656d7` is non-empty and clean |
| 12 | Fork `main` **is** `d402f18` | `git log -1 main` in the fork | ✓ P0.3's baseline and cut-point are current; the checkout sits on `ship/tx-engraving` `0b656d7`, which P0.3 correctly forbids |
| 13 | All four me-cli anchors resolve to the lines implied | `sed -n` each | ✓ `classify`, `EXIT_OK`, the codex32 class assert, the `sysw_cli.rs:1928` operand-class test |
| 14 | All three fork anchors resolve **at `d402f18`**, not only at the checkout | `git show d402f18:<path>` ×3 | ✓ `OutputDescriptor`, `WalletPolicyIdChunks`, `EncodeMultisig` |
| 15 | `13188e3`'s message is byte-identical to `8409616`'s | `diff <(git log -1 --format=%B …) …` → empty | ✓ NEW-I4 |
| 16 | The plan carries **0** fenced code blocks | `grep -c '^```'` | ✓ the build gate remains a no-op here, as the plan's own section says |
| 17 | Plan and spec byte-unchanged since `13188e3`; tree clean | `git diff 13188e3 HEAD --` ×2 → empty; `git status --porcelain` → empty | ✓ reviewed the right bytes |
| 18 | `refusal_row` appears exactly twice in the spec | `grep -n refusal_row` | ✓ NEW-M6's premise |
| 19 | §5.4's FULL/PARTIAL tests are exhaustive and disjoint | read 1178–1195 against §4.7 conjunct 8's both-paths clause | ✓ the tier logic itself is sound; only 1195 disagrees |

---

# Verdict

**NOT GREEN — 0 Critical / 4 Important.**

The design questions r3 left open are **settled and settled correctly**. The
PARTIAL re-decision is the right call and is now argued from the measurement
rather than against it; the §6 split matches the primary's own separation of the
two causes; the arithmetic is exact at every site I recomputed. Nothing in this
round asks for a design change.

What is open is one contradiction the fold created and three things around it:

- **NEW-I1** is the only text defect, and it is the fold's own: §5.4 now decides
  the same input two ways, and the reading a reader will take is the one that
  prints a passing compare prompt on a wallet that cannot exist.
- **NEW-I2** is r3's C1, still open at its tenth site — and the sweep that was
  offered as evidence for closing it reports it in one command.
- **NEW-I3** and **NEW-I4** are gates and records, not authorship: a red citation
  gate whose own verdict says *"fix before review"*, a staleness gate that is
  still vacuous for the one repo the amendment guards, and a fold commit whose
  message belongs to a different fold and vouches for gates that are red on this
  one.

**Three of the four are one edit each** (a clause at spec 1195, a digit at plan
L164, a path prefix at plan L188–189); the fourth is a commit message. None
requires re-opening a decision, and none should need an opus round to confirm —
a sonnet fold-vs-findings pass with the three commands above re-run is the
proportional close.

**What P0's implementer would still lack even at GREEN** (recorded now so the
list is ready, not as findings):

1. **A `refusal_row` slug vocabulary** — NEW-M6. Two slugs are named for
   20-plus rows that need one, and P0.1 fixes them two phases before P2.4's
   tests must match them.
2. **`device_admits` for the three clause-8 rows.** §7 clause 8 states
   `host_admits=false`, `md1_admits=false` and no address fields, but not
   `device_admits` — which is a REQUIRED schema field. The author must measure it
   (the device's parser has no origin-consistency check, so `true` is the
   expected answer, but the plan's own rule is *"all measured values re-derived
   at authoring time … never transcribed from reports"*).
3. **Which §6 row the `multi` twin asserts under explicit `--as descriptor`.**
   Determined — spec 1379's row, per §5.1's ruling — but stated in §7 and §5.1
   rather than beside the vector row's `refusal_row` field, so the author has to
   assemble it. One sentence in P0.1 would spare them.
4. **The two clause-8 `sortedmulti` inputs themselves.** Neither the plan nor
   the spec pins a fixture; §7's precedent for the 16-key row is a **recorded
   construction** (r6's NEW-N1, after r2's unrecorded keys proved
   irreproducible). The same discipline should apply here, and P0.1 does not say
   so.
