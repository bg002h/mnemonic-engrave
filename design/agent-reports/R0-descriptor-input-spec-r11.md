# R0 round 11 — PROPORTIONAL re-review of the r10 fold

**Target:** `design/SPEC_descriptor_input.md` at `e790f91` ("spec: fold R0 r10 --
the tier boundary, and the plate count corrected"). Tree HEAD at review time is
`e85f362` (F-423, `FOLLOWUPS.md` only — the spec file is unchanged from
`e790f91`; verified with `git show --stat`).
**Source of truth for what was required:**
`design/agent-reports/R0-descriptor-input-spec-r10.md` (0C/2I/5M/2N).
**Scope:** the fold only — fidelity to r10's nine findings, and defects the fold
itself introduced. Not a fresh audit. r1–r10 measured results, the walk log, the
citation gate, all rulings and dispositions taken as settled.
**Reviewer:** independent context, opus tier. Read-only against
`mnemonic-engrave`, `descriptor-mnemonic`, `seedhammer`. The whole fold diff
(`git diff 1a567ba e790f91`, 61 spec lines + 18 walk lines) was read; every hunk
is dispositioned below.

---

## Counts — NEW findings

| severity | count |
| --- | :-: |
| **Critical** | **0** |
| **Important** | **1** |
| Minor | 2 |
| Nit | 2 |

**The spec does NOT re-close GREEN this round.** One Important is open. r10's
nine are all closed — the fidelity half is clean, including both Importants,
and both closures were verified by recomputation rather than by presence. The
one Important is fold-introduced at the *same clause* that closed r10's new-N2:
the plural-path fix moved an implementer directive **inside** an operator-facing
quoted string. It is one clause wide.

---

## Disposition of r10's nine

| r10 finding | verdict | evidence |
| --- | :-: | --- |
| **new-I1** (tier boundary is `--as`-dependent; undetermined for every `multi` input) | **FIXED** | §5.4 L1049–1058. The predicate is now *"passes conjuncts 2–7 AND whose shape at least one `--as` path admits"* → FULL; *"a wallet NO path admits — a conjunct failure"* → PARTIAL. **This is total and decidable**, and the decomposition is exact rather than approximate: conjunct 1 is the *only* `--as`-dependent conjunct (§4.7 L640–657, "on the `--as md1` path ONLY … All other conjuncts (2–7) apply to `multi` identically"), and conjunct 7 states its own path-independence ("This conjunct gates BOTH `--as` values"). So `∃path: admits ⟺ (2–7 hold) ∧ (∃path: shape ok)` — the two tiers are complementary and exhaustive over "successful whole-input parse", with no third case. See the per-row trace below. |
| **new-I2** (§5.5's one-plate cell is false; 2 strings = 2 plates) | **FIXED** | §5.5 L1126 now reads **TWO plates**, with the mechanism (`one plate per STRING`), the pin (`plateTotal == len(strings)`) and the scope limit of the cited test. Re-measured, not transcribed: `bundlePlatePlan` (`seedhammer/gui/bundle_flow.go:386–402`) emits one `bundlePlate` per `c.strings` entry with `plateTotal: len(c.strings)`; the tree-wide assertion is `gui/bundle_engrave_test.go:38`; `TestBundlePlanSingleMD1OnePlate` (`:47`) feeds `singleMD1` → `loadVector(tb,"wpkh_basic")` (`gui/bundle_testdata_test.go:41–55`) → `md/testdata/vectors/wpkh_basic.phrase.txt`, which is **one line, 24 characters** — a single-string card, so the test cannot speak to a 2-string one. The walk log carries the correction (below), and the operator's direction became **F-423** (`e85f362`), so the honest fact produced a fix rather than only a retraction. |
| **new-M1** (`/**` identity cited to F-411; belongs to F-410) | **FIXED** | §5.3(a′) L954–956 now says F-410, with the correction named and `0x880c7` carried. Both citations in the walk's correction resolve: `descriptor-mnemonic@5465253b` = *"md-cli: accept BIP-388 `@i/**`…"*, and `crates/md-cli/tests/cli_bip388_double_wildcard.rs:1` = *"F-410 item 1"*. |
| **new-M2** (collapse evidence cited the OTHER divergence) | **FIXED — and correctly assembled** | §5.4 L1081–1087. Every element checks against r9's own probe output, not against the fold's paraphrase: `AltCountOutOfRange { got: 1 }` for `/0/*` (r9 report L93); the collapse target is the `/*` wallet, `0x9bf18` shared by `@0,@1` / `@0/*` / `@0/0/*` / `@0/1/*` (§5.3(a) L903–907); the address pair `bc1qadgf37z…` vs `bc1qu2cc6t7…` in §5.3(a)'s own table order; and `3bf32c0e…` is r9's literal printout **"id of the /* wallet"** (r9 report L116), i.e. the id of the collapsed encoding — the attribution the fold needed. The chain now reads error → collapse → shared id → divergent addresses → a different wallet's id above the compare prompt. |
| **new-M3** ("four of those rows" is five) | **FIXED as specified** | §5.4 L1057–1061 names five and labels the count. Machine-counted over §6's 34 table rows: the underivable class is exactly {mixed network (conjunct 5), single-key-wrapped multi (1), bare key in a script slot (1), hardened use-site (7), non-consecutive multipath (7)} = **5**, and r10's second gap (the conjunct-**3** key-count row, which is not a "threshold" row) is closed by the added *"and key-count rows"*. The residue r10 did not look for is **new-M1 below** — a different set of rows, not this count. |
| **new-M4** (verdict-first landed on (a) and not its (a″) twin) | **FIXED** | §6 L1213 now opens *"md1 cannot carry this wallet as written: key `@N` …"* — structurally identical to L1195's (a) row, which the spec binds it to. |
| **new-M5** (watch-only referent absent on refusal paths; tier reasoning did not reach it) | **FIXED, both halves** | §5.4 L1092–1096: printed in **BOTH** tiers, and the referent moved from *"this artifact"* to *"this wallet description"* — which exists whenever a parse succeeded, so the line is true on every refusal path. The partial tier's "share it accordingly" caveat is restored by the same edit (r10's converse half), and L1055 now says the partial block is *"the first three lines plus the watch-only line"*, agreeing with the bullet. |
| **new-N1** ("see below" points at nothing) | **FIXED** | §5.1 L764 is now `--as descriptor (not available in this build)` — and it matches the window refusal's own first line verbatim in substance. |
| **new-N2** (singular path slot vs a mixed (a)+(a″) input) | **FIXED — the plurality; the fix introduced new-I1** | §5.1 L836–839 now says *"each offending key and path substituted; a mixed input names all of them"*, which squares exactly with §6 L1195's clause *"A descriptor mixing an (a)-shaped and an (a″)-shaped key matches both this row and the next; **both fire**"*. The plurality defect is closed. Where the words were put is a new finding. |

**Fidelity: 9 of 9 closed.** No PARTIAL, no NOT FIXED.

### The per-row trace new-I1 asked for

All 34 §6 table rows enumerated mechanically; the tier is determined for every
one that reaches §5.4 (i.e. every post-parse row — §6 L1163–1166 states that
admission and §5.3 rows "fire from their own checks, after a successful parse").

| §6 row | conjunct | new-rule tier | right? |
| --- | :-: | :-: | :-: |
| `wsh(multi(…))` under `--as descriptor` (L1193) | 1, md1-path admits | **FULL** | ✅ derivable, spendable, md1-packable; the refusal names `--as md1`, so identification is exactly what this operator needs |
| `/0/*` under `--as md1` (L1195) | passes 2–7; §5.3(a) limit | **FULL** | ✅ §5.3 refusals are named full-tier followers; `wallet-id: none` fires by §5.4's own bullet |
| `<0;1>` under `--as md1` (L1213) | passes 2–7; §5.3(a″) | **FULL** | ✅ same |
| window refusal (L1189) | any admitted wallet | **FULL** | ✅ the `multi`-in-window case is named in the fold's own parenthetical; §11 item 5's test author can now determine the expected block |
| `--as` omitted (L1184) | any admitted wallet | **FULL** | ✅ and for an inadmissible wallet the tier is now *determined* (PARTIAL) rather than undefined — the second break r10 constructed |
| k > n (L1196), k < 1 (L1197) | 2 | PARTIAL | ✅ enumerated |
| too many keys (L1198) | 3 | PARTIAL | ✅ enumerated (added by this fold) |
| mixed network (L1199) | 5 | PARTIAL | ✅ enumerated |
| single-key-wrapped multi (L1208), bare key in script slot (L1209) | 1 | PARTIAL | ✅ enumerated |
| hardened use-site (L1210), non-consecutive multipath (L1211) | 7 | PARTIAL | ✅ enumerated |
| **any other use-site shape — `/0/1/*` (L1212)** | **7** | PARTIAL | tier ✅, **justification ✗ → new-M1** |
| **`ypub`/`upub`/… (L1200)** | **4** | PARTIAL | tier ✅, **justification ✗ → new-M1** |
| **`tr(sortedmulti(…))` (L1201)** | **1** | PARTIAL | tier ✅, **justification ✗ → new-M1** |

No row sits on the wrong side of the boundary. The rule is also compatible with
§7 L1300–1304, which independently disclaims `host_admits` as meaning *"some
`--as` succeeds"* — §5.4 uses the shape-level predicate, not `host_admits`, so
the two sections do not collide.

---

# NEW findings

## new-I1 — the plural-path fix put an IMPLEMENTER DIRECTIVE inside the operator-facing quoted string; it is the only such span in the file, and the r10 sweep cannot see it

**Where.** §5.1 L836–839, window refusal variant 2, rewritten by this fold to
close r10's new-N2:

> - input (a)/(a″)-shaped: *"--as md1 cannot carry this wallet either — key
>   `@N` uses `<path>` **(each offending key and path substituted; a mixed
>   input names all of them)**. No path in this build engraves this file. …"*

The bolded clause is **inside** the `*"…"*` span — verified mechanically, not
by eye: extracting all 45 quoted spans and searching them for
`substitut|placeholder|implementer|verbatim` returns **exactly one hit**, this
one. Every sibling site keeps the same directive *outside* the quote: §6 L1195
*"the offending key is named per §5.3's per-key rule"*, §6 L1213 *"key named per
R0 r4's NEW-M4"*, §5.3 *"Window substitution per §5.3."* And the text this fold
replaced used the placeholder convention correctly — `` `<the operator's own
path, substituted>` `` was delimited as a slot; the new wording is bare prose in
the message body.

**Why it is a defect and not a formatting quibble.** §6 L1167–1172 is NORMATIVE
about exactly this: *"Every quoted text below: leads with the verdict; contains
NO internal identifiers … (those live in the row's annotation, **outside the
quotes**)."* That rule is what makes the `*"…"*` span mean "what the operator
sees" — it is the convention r9's I5 was Important for breaking. So there are
two readings, with materially different stderr:

- **Reading A (the convention):** the span is verbatim, the operator is shown
  *"key `@N` uses `<path>` (each offending key and path substituted; a mixed
  input names all of them)."* — spec-ese in a refusal at steel-imminent stakes,
  in a build where this refusal is, per L841–842, *"the front door of the S3
  release"* that **both** walked journeys' first commands hit.
- **Reading B:** an implementer strips it as an editorial note.

**Constructed failure — the route to a shipped string is short.** §11 item 5
requires *"`--as descriptor` in a build where its path has not shipped exits 3
and prints §5.1's window refusal — BOTH alternative variants tested"*. A test
author writing the expected text from the spec has only the quote to copy from;
under reading A the parenthetical is pinned into the assertion, and the shipped
message then contains it permanently, because the test now guards it.

**Why the r10 sweep did not catch it.** The sweep regex is
identifier-shaped (`§ | F-\d{3} | S[123] | R0 | NEW-[A-Z]\d | walk W\d |
conjunct \d`). Re-run this round over all 45 spans, multi-line aware: **0 hits**
— genuinely still clean for that class. This is a *different* leak species —
editorial directives rather than identifiers — and the sweep is blind to it by
construction. A one-word widening (`substitut`) closes it.

**Not prescribing a fix,** but the shape every sibling uses is available: end
the quote after ``uses `<path>`.`` and carry the directive in the bullet's
trailing annotation, where §6 already puts the identical instruction.

---

# Minor

**new-M1 — the partial tier's row inventory is still not exhaustive, and the
fold's new "machine-counted" label now vouches for it.** §5.4 L1057–1061 asserts
*"**The rows it covers** describe wallets whose addresses are underivable (…
five rows, machine-counted …) or wallets unspendable or anyone-can-spend (the
threshold and key-count rows)"* — a completeness claim over the partial tier.
Machine-counted over §6's 34 rows, three post-parse rows land in the partial
tier and fall under **neither** named class:

- **`/0/1/*` / bare fixed index (L1212), conjunct 7.** Unambiguous: §6 L1163–1166
  places conjunct rows after a successful parse, so it reaches §5.4. §4.7
  conjunct 7 refuses it as **UNMEASURED**, not as underivable — the device's
  `parsePath` grammar accepts it. Grep `conjunct 7` in §6 → **3** rows; the
  enumeration names **2**. This is r10's own new-M3 finding, one row further on.
- **`ypub`/`upub`/`vpub`/`Upub`/`Vpub` (L1200), conjunct 4.** On the spec's own
  framing (§4.7 states version bytes as an *admission* conjunct, not a parse
  rule) this is post-parse; the wallet is neither underivable nor unspendable —
  a `ypub` BIP-49 wallet is a real wallet the device simply will not read.
- **`tr(sortedmulti(…))` (L1201), conjunct 1.** §4.7's closing paragraph lists it
  among the measured *admission* exclusions — the same class as the
  single-key-wrapped multi row, which the enumeration does name.

The **rule** is stated generally over "a wallet NO path admits" and is complete
and correct for all three (this is why it is Minor, not Important) — what is
false is the justification's claim to describe the rows it covers. The
"machine-counted" label makes it worse than r10 found it, because the label
now certifies an enumeration that is exhaustive only for the *underivable*
subclass it was measured against.

**new-M2 — the `multi`-in-window parenthetical over-claims for the one input
that most needs it.** §5.4 L1053–1056: *"(A `multi` input in the window is
full-tier: … the wallet is derivable, spendable, and **md1-packable**, and
stripping its identification would blind exactly the operator whose refusal says
**md1 is available**.)"* Both bolded claims are false for a `multi` whose
use-site path is `/0/*` or `<0;1>` — §5.3(a)'s own `multi` exception says that
wallet is carried by **neither** path, and §5.1's variant-2 refusal (the one
this same fold rewrote, three sentences up) tells that operator *"no path in
this build engraves this file"*. The **tier is still right** — conjuncts 2–7
hold and the md1 path admits the shape, so the rule returns FULL, which is the
same answer the non-`multi` `/0/*` case gets — so this is justification prose
that is true for the common case and stated as though it covered the class.
Worth one qualifier, because a later reader deriving the tier from the
parenthetical rather than the rule gets the wrong answer for that subcase.

---

# Nit

**new-N1 — the follower list is not partitioned by tier, so one combination
shows a follower on the wrong side.** §5.4 L1051–1053 attaches *"the
`--as`-omitted choice block"* to the FULL block's follower list. But
{inadmissible wallet, `--as` omitted} — e.g. `wsh(sortedmulti(0,…))` with no
flag — is PARTIAL by the rule and still gets the choice block, since §5.4 fires
on the parse and the flag check is what follows. The rule itself is not in
doubt: the sentence opens *"The tier is decided by what does NOT depend on the
flag"*, which forecloses reading the list as a trigger — that preamble is
precisely what makes this a Nit rather than a repeat of r10's new-I1. One word
("typically", or "before a pack or the `--as`-omitted choice block **when the
wallet is admitted**") settles it.

**new-N2 — the walk log's W14 is corrected 45 lines away with no marker at the
site, which is the exact path new-I2 travelled.** The appended corrections
(`WALK_descriptor_input_2026-08-28.md:520–536`) are accurate and both their
citations resolve. But in situ, W14 still reads *"and the one-plate hope is
TRUE, measured"* (L449), *"the fork's own `TestBundlePlanSingleMD1OnePlate` pins
one md1 card = exactly one plate"* (L472–473), and *"one md1 plate cut now"*
(L479) — three false statements with no forward pointer. That is not
hypothetical decay: **r9 read W14 in situ, took "measured TRUE" as settled, and
asked for it to be folded** — which is how the false cell reached §5.5 in the
first place. The spec side is clean (grepped: L1126 is the file's only
plate-count claim, and it is the corrected one). One bracketed marker at L449
and L472 costs nothing and closes the loop the corrections section opened.

---

# Verified in passing — recorded so a later round does not re-spend it

- **The identifier leak sweep is still clean.** All 45 `*"…"*` spans, matched
  across newlines, against r10's regex: **0 hits**. The fold changed two quoted
  texts (§5.1 variant 2, §6's (a″) row) and neither introduced an identifier.
- **The five-row underivable count is correct as a count**, verified by
  enumerating §6's rows and their conjunct annotations, not by reading.
- **`plateTotal == len(c.strings)` is the fork's rule, not an inference** —
  `bundlePlatePlan` builds it directly and `bundle_engrave_test.go:38` asserts
  it for every card in every plan, so no 2-string card can be one plate.
- **`wpkh_basic.phrase.txt` is a single 24-character md1 string** — the cited
  test's fixture, read (not named), confirming r10's diagnosis exactly.
- **r9's `3bf32c0e…` is labelled "id of the /* wallet" in r9's own probe
  output**, so the fold's new attribution of it to the (a) collapse is correct
  and not a second transcription of the (a′) pair.
- **F-423 exists and is scoped** (`FOLLOWUPS.md:14613`, commit `e85f362`), owning
  phase "with S2's firmware build", with the operator's verbatim direction and
  an explicit "do not guess the fit — measure it" step. The new-I2 correction
  therefore produced a scheduled fix, not just a retraction.
- **§8, §9, §10 and §11 are untouched by this fold**, and no citation the fold
  added resolves to a missing target.

---

# What would re-close the round

new-I1 folded — move the parenthetical out of the quoted span, as every sibling
site does — then a re-review scoped to *"did the fold fix the one, and did it
introduce a defect"*. The two Minors and two Nits are single-clause edits and
can ride along; widening the quoted-span sweep from identifiers to editorial
directives (`substitut|placeholder|verbatim`) is a one-word change to a check
that already exists.

**The fidelity half is closed and should not be re-opened:** all nine of r10's
findings are FIXED, both Importants were verified by recomputation (the plate
plan re-read from source, the collapse chain re-checked against r9's raw probe
output), the tier rule was proved total against all 34 §6 rows, and nothing in
this round comes from a question r10 did not ask. One line for the cycle's
record: **the r10 fold's only new Important sits at the clause that closed r10's
smallest finding** — a Nit-level fix, correctly diagnosed and correctly
executed as to substance, that broke a convention two sections away. That is the
third round running in which the fold, not the artifact, was the defect source,
and the second in which a mechanical sweep was clean because the defect was of a
species the sweep was not written for.

---

# What the spec's own text leaves open (for the round that does close it)

Not findings — the spec states these as open, and they survive this fold
unchanged.

**§9 residuals (7):** (1) nothing run on hardware; (2) the three admission-table
cells have never been exercised — *a gate that has never executed*; (3) change
addresses and testnet unmeasured in the `--as md1` address equality; (4) the
published `md-codec` 0.42.0 tarball not byte-compared to the tree; (5) TinyGo
compilation of a new `sysw.Classify` arm unchecked; (6) two negative claims with
named, narrower search scopes; (7) **§6's refusal texts "have not been walked
with the operator"** — note this item predates the walk (last touched at
`ff9a0f2`) and the walk *did* reach refusal text (W5, W11, W13), so it is at
minimum due a scope update when someone folds next.

**Parked with S2 (F-418, S1 → S3 → S2):** §11 item 1 (the `Descriptor`
classify round trip), §11 item 4's `--as descriptor`-only refusal rows, §11
item 6 (a `ClassDescriptor` record loaded and displayed on a real device —
the discharge of §9 item 2). All three need the device on the bench.

**Named follow-ups the spec defers to:** F-413 (host-side SLIP-132
normalisation), F-414 (descriptor + other records in one container), F-416
(`--in`'s contract note in `SPEC_systemwide_payloads` §5.6), F-417 (md1 wire
extension seam), F-422 (**RULING WANTED**, owning phase *"descriptor-input plan,
before S1 closes"* — an interim status-quo ruling is recorded, not a final one),
plus F-420/F-421 (cross-tool referrals, owning phase "with or after S1") and
F-423 (plate packing, fork-side, with S2).
