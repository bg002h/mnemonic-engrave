# r4 — `DESIGN_coordinator_compatibility.md` third fold

**Artifact:** `design/DESIGN_coordinator_compatibility.md` at `3b374675`
(unchanged at `8b33e840`; `git diff 3b374675 HEAD -- <file>` is empty).
**Input review:** `design/agent-reports/coordinator-compat-spec-r2-verify.md`
("r3": 1C / 7I / 5M / 1N new, implementable **NO**).
**Fold under review:** `git diff 4882d2af..3b374675`, 114 insertions /
19 deletions, one file, spanning commits `78f0c541` and `3b374675`.
**Repos read:** `descriptor-mnemonic` `b2c5d693`, `seedhammer` `7b6f2fb`, this
repo `8b33e840`. All three trees left clean; nothing tracked was modified.
**Brief:** `design/agent-briefs/coordinator-compat-spec-r4.md`.

**New counts: 0 Critical / 2 Important / 2 Minor / 1 Nit.**

---

## Answers, first

**A. Did the fold close r3's findings, clause by clause?**

| r3 | FIXED | PARTIAL | NOT FIXED |
| --- | --- | --- | --- |
| Critical (1) | 0 | 1 | 0 |
| Important (7) | 3 | 3 | 1 |
| Minor (5) | 0 | 0 | **5** |
| Nit (1) | 0 | 0 | 1 |

Separately, the seven **carried-over** items from r2 that r3 had marked
NOT-FIXED — r2's five Minors and both Nits — are **7 of 7 FIXED**, and well
fixed: the blind-spot list, the `composerCopy*` gate, the self-reported-version
clause with its `bitcoind --version` measurement, the digest class, the
`canonicalize.rs:168` citation (verified: that line is
`pub fn canonicalize_placeholder_indices`), the status header, and the
0-based/1-based note.

That pairing is the story of this fold. The commit message reads *"fold the r3
Minors and Nits"*, and it folded **r2's** Minors and Nits — the ones r3 reported
on — while r3's own five new Minors and one new Nit were not touched at all.
`grep` confirms each: `experimental` 0 hits, `Complete` 0 hits (the one match is
the word "incomplete" in the preamble), `hash256`/`ripemd160`/`hash160` 0 hits,
and both halves of the §2-vs-§5 contradiction still stand verbatim at lines
381-382 and 532. This is a scoping slip, not the clause-dropping failure the
brief asked me to hunt — but it is why the Minor column is 0 of 5.

On the clause-dropping failure itself: **it did not repeat in the Criticals and
Importants at the rate it did before, but it did repeat.** Three of the seven
Importants are PARTIAL for a named, identifiable clause, and one of my two new
Importants is the fold re-creating — one paragraph away, in the same
subsection — the exact defect it fixed in the adjacent sentence.

**B. Is it implementable NOW? Still NO.** r3's four blockers: `fp_partition`'s
derivation is **closed** (the port now names the extension and the `keys` map it
comes from), `root` is **closed as a field** but its stated domain is wrong
(NEW-I1), the two key spellings are **closed** (machine-checked: no bare
`blocks#1`, no `multi(2,@0,@1,@2)`), and the serialized form is **half closed**
— a separator and an ordering exist, membership does not. There is not a fifth
blocker; walking §5 steps 1-2 as an implementer who may invent nothing, there
are **seven**, listed in section B below. They are individually smaller than
r3's, and none is Critical.

**Design or plan?** Section D gives the judgement. Short version: **three of the
seven are design, four are plan, and the five Minors are neither — they are a
propagation sweep a command should do.** A fourth design-review round is the
wrong instrument, and I say so with reasons rather than as a hedge.

**C. Did this fold introduce a defect?** Yes, two Important ones, both in new
text, both machine-findable without a reviewer.

---

## A. Per-finding verdicts

### Critical

**N-C1 — per-path fingerprint partition not computable from the walk the fold
names — PARTIAL.** The blocker is closed; one clause of the fix is not.

| clause | landed? | where |
| --- | --- | --- |
| say that the port adds a per-branch slot-index list (the `keys` map `branchOf` already builds) | **yes** | lines 170-178, "THE PORT IS A PORT PLUS TWO NAMED EXTENSIONS", naming `md/policy_shape.go:239-245` and `br.Keys = len(keys)` and **discards the map** |
| name the Go convergence as plan-3 scope | **no** | contradicted at lines 186-188 |

The finding's stated wrong outcome — *"at section 5 step 1, an implementer must
decide where per-path slot ids come from, and the design's answer does not
contain them"* — is genuinely gone. An implementer now knows to retain the index
set.

What did not land is the second clause, and the fold did not merely omit it, it
wrote the opposite: *"The fork's Go converges afterwards **if it wants** the new
fields; **nothing in the fork needs them today**."* "Today" is load-bearing and
true, and misleading. §5 step 4 puts *"the registry consumed from **generated**
Go"* on the device, and a `RuleSet`'s `refuse` is
`fn(&Skeleton) -> Option<Reason>` — so the fork must **build a `Skeleton`**,
which means the fork needs both extensions the moment plan 3 starts. The design
itself says `fp_partition` exists for Liana's `DuplicateOriginSamePath` and
`key_partition` for `DuplicateKey`, neither of which is in
`composerLianaOutsideModelClass` today, so the registry the device consumes will
read fields the fork's `PolicyShape` cannot supply. §5 step 4's scope list still
reads *"the Go port of the canonicaliser"* and never mentions the extended
`PolicyShape`.

*Consequence:* plan 3 gets dispatched under-costed by exactly the item r3 asked
to have named. Not a wrong verdict — a stale plan. Keeping this PARTIAL rather
than FIXED because the clause was named and the fold answered it with "if it
wants".

### Important

**N-I1 — `Skeleton` omits `root` — FIXED as a field, and the fix introduced a
new Important.** `root: ScriptKind` is on the struct (line 104) with the fork's
own justification quoted at lines 162-168 (*"policyShape walks tagWsh and tagSh
identically"*). Liana class 1 is now computable in principle. The **domain** the
fold gave the field is wrong in both directions — that is **NEW-I1**, a new
defect in new text, not an unclosed clause.

**N-I2 — two fields answer the internal-key question — FIXED.** Cleanly. The
duplicate `key_path` is gone, with the reason recorded in the struct itself:
*"NO key_path field: it would duplicate shape.KeyPath, and a ported rule reading
the duplicate is how I-9 re-opens. One field, extended below."* Extension 2
(lines 179-183) widens the ported enum to four values. The precedence question
the finding raised is dissolved rather than answered, which is the better fix.
Verified the rename is faithful: the fork's `KeyPathNone` is documented as
*"not a taproot policy"* (`md/policy_shape.go:34`), so the design's `NotTaproot`
is the same member under a clearer name.

**N-I3 — §1(a2)'s template-only rule is not decidable from `RuleSet` as typed —
NOT FIXED.** `RuleSet` gained exactly one field this fold (`measured_at`);
`refuse: fn(&Skeleton) -> Option<Reason>` is unchanged, `Reason` is unchanged,
and §1(a2)'s text is unchanged. `grep -i 'read-set'` returns nothing. The
finding's three candidate registries — annotate each `RuleSet`, split `refuse`
into structural and identity halves, tag each `Reason` — are all still open, and
they are materially different implementations. The wrong pick still puts a false
`Imports` on `md compose`'s most common output, which is the outcome r2's C-2
exists to prevent.

**N-I4 — `key_partition`'s absence rule and "derivation" — FIXED, both
clauses.** Lines 201-208: *"groups by `(xpub bytes, origin_path)`"* with the
reason (*"the only derivation a decoded md1 carries"*), and *"an absent xpub is
its own singleton, by the same argument as the fingerprint rule below"*. Both
the sub-clause about absence and the `origin_path`-vs-`use_site_path`
disambiguation landed, and the fold added the reason the two partitions cannot
be collapsed. Verified free: `ExpandedKey` really does carry both `origin_path`
and `use_site_path` and an `Option<[u8; 65]>` xpub
(`crates/md-codec/src/canonicalize.rs:352-364`).

**N-I5 — no type carries the date mechanism 1 requires — PARTIAL.** `MeasuredAt`
exists (line 122) and its doc comment is good on the no-clock point. But the
field landed on **`RuleSet`**, and mechanism 1 says *"Every **verdict** carries
`(version verified, renderer, date)`"*. All four `Verdict` variants still carry
no date:

    Refuses { reason, span, renderer: Option<RendererId> }
    ImportsAltered { as_read, span, renderer }
    Imports { span, renderer }
    Unproven { reason, span: Option<Span> }

The fold's own doc comment states the requirement it then satisfies on a
different type: *"§3.1 requires every verdict to carry a DATE, and no type held
one."* A measured positive (`Imports`) comes from the evidence table, not from a
`RuleSet`, so §3.1's mandated copy — *"Liana 8.0 imports (2026-09)"* — still has
no field to read its date from. There is no evidence-row type in §1A at all,
though §3 refers to *"the evidence row"* four times. Mechanism 1 is assigned to
plan 1.

**N-I6 — §3's staleness section asserts facts this tree falsified — PARTIAL, 2
of 5 clauses.**

| clause | verdict | evidence |
| --- | --- | --- |
| harnesses all in `/scratch/.tmp` | **FIXED** | lines 421-426 now say Liana's is committed at `harnesses/liana/` (`606ab180`) and narrow the quantifier to *"Nunchuk's and Core's"*. `git ls-files harnesses/` returns the three Liana files. |
| *"Liana must be re-measured at v15.0"* under **Open, and blocking a spec** | **FIXED** | struck through, with the 289/289 result and the residue stated. |
| plan 2 *"closes F-633"* | **NOT FIXED** | lines 564-566 still say it, now hedged with *"to be repeatable"*. r3's point stands: the remaining F-633 work is §8x copy, and this repo's HEAD is doing exactly that §8x work right now (`eee58770`, `f23c740a`). |
| mechanism 4's *"every verified version"* is undefined between `span` and `source_verified_at` | **NOT FIXED** | line 430 is unchanged; `RuleSet` still carries both and the design still never says which one the freshness gate reads. The flagship "red today on Liana" example turns on it. |
| §3's premise still presents the `InvalidPolicy` hazard with no mention of 289/289 | **NOT FIXED** | lines 399-405 unchanged. The 289/289 fact lives only in the "Open" section 180 lines later. `grep -c '289/289'` → 1. |

**N-I7 — the key's serialized form is undefined — PARTIAL.** Real progress, and
three named clauses missing.

Landed: the separator (*"the template, a `U+001F` separator, then the
partitions"*), the ordering (*"slots ascending, groups ordered by their lowest
slot, and paths in template traversal order"*), and the type/illustration depth
mismatch — `fp_partition` is now `Vec<Vec<Vec<u8>>>` and the illustration is
three deep (`[[{0}],[{1}],[{2}]] [[{3}]] [[{4}]]`).

Not landed:

1. **Membership of `key_partition`.** *"the partitions"* is plural and never
   resolved. `key_partition` is `Vec<Vec<u8>>` — `[group][slot]` — so *"rendered
   as `[path][group][slot]`"* literally describes only `fp_partition`, and only
   one separator is specified, so a second partition has no place to go. The
   illustration shows only the fp-partition. Step 1's gate is a **string
   equality** (`chunks -> key == descriptor -> key`); an implementer cannot
   produce the string.
2. **Membership of the internal-key kind** — the finding's clause *"I-9's defect
   is only fixed if `key_path` is [in the key]"*. It is not; the field is gone
   from `Skeleton` entirely. See "carried over" below.
3. **"Make the illustration an instance of it."** The illustration still uses a
   `| fp-partition per path:` label and a newline, not `U+001F`, and wraps the
   template across three indented lines with no statement that the wrapping is
   cosmetic.

A fourth, unraised by r3 and arising from the new ordering rule: *"paths in
template traversal order"* has no defined domain for `tr`. The fork's
`shape.Branches` holds tapscript **leaves**; a spendable internal key is counted
as an unlocked path by the Liana rule (`gui/composer_consent.go:377-380`) but is
not a `Branch`. Whether the internal key's slot is a path of its own, joins
another, or is absent from `fp_partition` is undecided — and all 15 of the
shapes that discriminate the Core boundary are `tr`.

### Minor — 0 of 5

| r3 finding | verdict | machine check at `3b374675` |
| --- | --- | --- |
| **N-M1** §2 vs §5 contradict on who computes the table's keys | **NOT FIXED** | line 382 *"by calling `md shape-key`, never by reimplementing the canonicaliser in Python"*; line 532 *"**The table generator is a step-1 `xtask`, not `md shape-key`**"*. Both verbatim. |
| **N-M2** C-3 did not propagate to mechanism 5 | **NOT FIXED** | line 435 still *"Any byte change retires every Nunchuk **positive** until re-measured"*, against line 89's *"retires it exactly as it retires a positive"*. |
| **N-M3** `PolicyShape.Complete` never mentioned | **NOT FIXED** | `grep -i complete` → 1 hit, the word "incomplete" at line 21. `UnprovenReason` still has no variant for it, so §1(a)'s promise that an unfamiliar fragment *"falls to `Unproven`"* has no mechanism. The fork's contract is emphatic (`md/policy_shape.go:20-22`: *"it sets Complete=false and the caller must show nothing"*). |
| **N-M4** digest kind vocabulary not enumerated | **NOT FIXED** | `grep -c 'hash256\|ripemd160\|hash160'` → 0. md-codec renders four (`render.rs:199-202`), the design names `sha256` only. |
| **N-M5** `--md-only` vs `md compose --experimental` | **NOT FIXED** | `grep -ci experimental` → 0. `--experimental` is on that exact command (`crates/md-cli/src/main.rs:304-306`, *"Admit key-less paths and unsorted-where-sorted-was-legal, with a warning"*), and a none-case policy built from a key-less path needs both. |

### Nit

**N-N1 — the input review's Important count is one short — NOT FIXED.** The
header still reads `4C/10I/5M/2N` for the opus report. Machine count at
`b2c5d693`-era tree: `grep -c '^### [CI]-'` on
`coordinator-compat-spec-opus.md` → **15**, of which `I-1` … `I-11`. See
NEW-N1 for what else that line now gets wrong.

### Carried over from r2, still open after three folds

Neither of these was re-raised as a new finding by r3 — both were r2 PARTIALs —
so they sit outside the tally above. They are both plan-1 blockers.

- **r2 I-4 sub-clause (b): `expand_per_at_n` can fail, and `md shape-key` has no
  rule for it.** `grep -i expand` over the design → **0 hits**, unchanged for
  three folds. Verified: `expand_per_at_n` returns `Result`
  (`crates/md-codec/src/canonicalize.rs:445`) and propagates
  `Error::MissingExplicitOrigin` and `Error::DivergentPathCountMismatch`;
  `validate_origin_key_consistency` swallows it **on purpose**
  (`crates/md-codec/src/validate.rs:431-440`: *"expand_per_at_n fails for its
  own reasons — a missing explicit origin, a dead card"*). The third state r3
  named is still covered by nothing: `keys_present == true` **and** unexpandable
  is neither `KeysAbsent` nor any other `UnprovenReason`, and erroring out
  collides with C-4's *"`md descriptor` never refuses"*.
- **r2 I-9 residual: the key still cannot carry the internal-key kind.** This
  fold settled the *rule input* (N-I2, well) and, by defining the serialized key
  as template + partitions, settled that the *key* does not carry it. Today this
  is latent, not live: md-codec renders the NUMS H-point **literally**
  (`render.rs:87-88`, `NUMS_H_POINT_X_ONLY_HEX`), so NUMS is distinguishable in
  the template. The collision arms when F-449 lands — an unspendable **xpub**
  internal key is a placeholder, rendering `@i/<0;1>/*`, byte-identical to a
  spendable one. F-449 is OPEN and owned by *"its own constellation cycle in
  descriptor-mnemonic"*. Every key committed to the evidence table before then
  becomes wrong on that day. The key's new "blind spots, listed" section — which
  exists precisely so *"same key, different verdict"* is not folklore — does not
  list this one, the only blind spot the design has a named follow-up for.

  Note also that the containment claim under that list is too strong:
  *"Any of these turning into a real disagreement surfaces as a **D1/D2 build
  failure**."* D1-D4 are all **rule-vs-evidence** classes. Two *evidence* rows
  colliding on one key with different verdicts fits none of them, and the table
  is keyed by the key, so the build has no defined behaviour for it.

---

## B. Implementable now? NO — seven blockers in §5 steps 1-2

Walking as an implementer who may invent nothing:

1. **Write `enum ScriptKind`** — cannot. NEW-I1: the stated domain is
   `wsh | sh | sh(wsh) | tr`; one of those four is not a value of the type the
   design cites, and three real values are missing.
2. **Write the abstracting render mode** — cannot, for digests. NEW-I2: the
   document gives two spellings, `sha256(#)` and `sha256(#1)`, 35 lines apart,
   and N-M4 leaves three of the four digest kinds unnamed.
3. **Build a `Skeleton` from a card whose keys will not expand** — cannot. r2
   I-4(b), open three folds; no rule, no `UnprovenReason` variant, and the two
   available answers contradict C-4.
4. **Emit the key** — cannot. N-I7 clause 1: is `key_partition` in the string,
   and after which separator? This is the string step 1's gate compares.
5. **Fill `fp_partition` for a `tr` policy** — cannot. N-I7's fourth clause: the
   internal key is not a `Branch` and has no stated home.
6. **Construct a `Verdict`** — cannot satisfy mechanism 1, which plan 1 owns. No
   variant carries a date; `MeasuredAt` is on `RuleSet`; no evidence-row type
   exists.
7. **Implement §1(a2)'s template-only rule**, whose gate is in step 2 — cannot.
   N-I3: three materially different registry shapes, none chosen.

Blockers 1, 2 and 3 are the ones that stop the first hour of work.

---

## C. New findings

### Important

**NEW-I1 — the `root` field added to close N-I1 declares a domain that
contradicts the type it cites, in both directions.**

*State:* an implementer at §5 step 1 writes the `Skeleton` struct. *Wrong
outcome:* two, below.

Line 104: `root: ScriptKind,          // wsh | sh | sh(wsh) | tr`. The design
never defines `ScriptKind` as a Rust type — that trailing comment is the only
statement of its domain — and it cites the fork's `md.ScriptKind` twice (lines
152, 164). Measured at `7b6f2fb`, `md/md.go:1165-1180`:

    type ScriptKind int
    const (
        ScriptWpkh ScriptKind = iota
        ScriptPkh
        ScriptSh
        ScriptWsh
        ScriptTr
        ScriptShWpkh
    )

**(a) `sh(wsh)` is not a `ScriptKind` value.** Nested segwit is
`Root == ScriptSh` plus a separate bool, `Template.InnerWsh`
(`md/md.go:1212-1219`), whose own comment says: *"It distinguishes a
nested-segwit P2SH-P2WSH from a bare legacy P2SH multisig — both summarize to
ScriptSh+PolicySortedMulti, but they hash to DIFFERENT addresses, so a consumer
… MUST use this to pick P2SH_P2WSH vs P2SH and never verify one against the
other."* An implementer writes `enum ScriptKind { Wsh, Sh, ShWsh, Tr }`; Liana
class 1 is `if root == md.ScriptSh` (`gui/composer_consent.go:388`), whose
comment states the collapse is deliberate — *"ScriptSh covers BOTH bare sh and
sh(wsh)"*. With a separate `ShWsh` variant, class 1 stops firing for a
`sh(wsh(sortedmulti))` policy and it falls through to class 3, *"no locked
path"*: right verdict, wrong reason class, and under D3 the design says *"the
verdict is unaffected"* — so it is mis-attributed silently.

**(b) `wpkh`, `pkh` and `sh(wpkh)` are missing entirely**, and md1 carries all
three. Rust: `Tag::Wpkh` and `Tag::Pkh` exist in `crates/md-codec/src/tree.rs`.
Go: `EncodeSingleSig(..., script ScriptKind)` (`md/encode_singlesig.go:36`), and
its fuzz test walks the whole domain — `ScriptKind(scriptRaw % 6)` with the
comment *"6 ScriptKind values incl. ScriptShWpkh"*
(`md/encode_singlesig_fuzz_test.go:35`). `md descriptor` reads single-sig cards.
So the `Skeleton` builder has no representable `root` for them, and step 2's own
gate — *"`md descriptor` NOT refusing"* — cannot pass.

*Fix.* Either mirror the six values plus `InnerWsh`/`InnerWpkh`, or state the
collapse rule explicitly (`Sh` covers bare sh and sh(wsh), as the fork's Liana
class 1 does) and say which single-sig roots are in the domain. Do not leave the
domain in a comment.

**NEW-I2 — the key's *definitional* sentence still says digests carry no class,
35 lines after the fold decided they do; the document again teaches two key
spellings.**

*State:* an implementer at §5 step 1 builds the abstracting render mode.
*Wrong outcome:* two renderings of one key.

- Line 196 (**new this fold**, closing r2 M-4): *"Digests render **with a class
  too** — `sha256(#1)` — symmetric with locks."*
- Line 231 (**untouched**), inside §1(a), the paragraph that *defines* the key:
  *"rendered as a template with lock values replaced by `kind#class`, **digests
  by their kind**, and origins erased."*

This is precisely the defect r3 reported as two incompatible key spellings — and
this fold fixed the lock half of it in the sentence immediately below the
illustration (*"the lock reads `older-blocks#1`, not `blocks#1`"*, line 238)
while re-creating it for digests in the same subsection. §1(a) is the sentence
an implementer reads to build the renderer; §1A's is the one a reviewer reads.

Step 1's gate does not catch it: `chunks -> key == descriptor -> key` is an
equality between two calls of the *same* function, so it passes under either
spelling. What breaks is the `xtask`-built table versus `md shape-key`'s output
versus any hand-written vector — silently, in the direction §2 names: *"positives
quietly stop firing, or fire on the wrong shape."*

### Minor

**NEW-M1 — `measured_at` on `RuleSet` names a fact that type cannot hold.** A
`RuleSet` is the **rule-derived** side: it already carries
`source_verified_at: &'static [Version]`, the versions whose *source* was read.
A measurement date belongs to an evidence row. Putting both on one struct, one
named `source_verified_at` and one `measured_at`, invites exactly the confusion
N-I6's fourth clause is about — mechanism 4's undefined *"every verified
version"*. Pick a home for the date and make the two names say which fact each
holds.

**NEW-M2 — two load-bearing citations omit their crate path, and one resolves to
nothing as written.** Line 514 cites `validate.rs:449-458`; there is no
`crates/md-cli/src/validate.rs` in `descriptor-mnemonic` at `b2c5d693` — the
file is `crates/md-codec/src/validate.rs`. Line 230 cites `canonicalize.rs:168`
the same way. **Both are correct once the path is supplied** — I verified
`validate.rs:449-458` is the measured regression the design invokes (*"an
already-engraved card of that shape stopped being READABLE, which is a far worse
outcome than the advisory this check exists to give"*) and `canonicalize.rs:168`
is `pub fn canonicalize_placeholder_indices`. The defect is that a reader
following either as written finds nothing, and both are cited as the *ground* of
a decision (C-4's split, and the key's renumbering).

### Nit

**NEW-N1 — the provenance header is wrong in three ways after this fold.** Line
6 still reads *"Reviewed **twice**"* while the status line one line above reads
*"third fold"*; the opus report is still cited as `4C/10I/5M/2N` (machine count
**15** findings, `I-1`…`I-11`, so 4C/**11**I — r3's N-N1, unclosed); and
`coordinator-compat-spec-r2-verify.md`, the review this entire fold responds to,
is **not cited anywhere in the document**. Every finding in the fold is tagged
`(r3)` or `(r3 M-4)` with no entry telling a reader what `r3` is.

---

## D. Design or plan? — the judgement

**A fourth design-review round is the wrong instrument. Settle three sentences
in the design, run two commands, then write plan 1.**

The residue splits three ways, and only one third of it is design work.

**Design (3 items, ~four sentences — an authoring pass, not a round).** These
share one property: the plan cannot freely choose, because a wrong pick is
either expensive to reverse or invisible to every gate.

1. **The key's membership.** Is `key_partition` in the serialized key, and is
   the internal-key kind? (N-I7 clauses 1-2, plus the `tr` path-domain question,
   plus r2 I-9.) This is the one thing that must be settled **before** plan 1
   ships, because every evidence row is committed against the key and a format
   change retires the whole table. It is also where the design's containment
   claim needs one more sentence: D1-D4 are rule-vs-evidence classes and have no
   member for two evidence rows colliding on one key.
2. **The digest spelling** (NEW-I2) — same reason, one word.
3. **The unexpandable-keys state** (r2 I-4(b), three folds open) — one
   `UnprovenReason` variant and one sentence on `md descriptor`'s exit code.
   Cheap, and it collides with a stated ruling, which is why the plan may not
   pick unilaterally.

**Plan (4 items).** For each of these the design already states the *invariant*;
what is missing is a mechanism, and a plan review can do the thing a design
review cannot — ask whether the chosen mechanism has a gate that can actually
fail.

- **N-I3's registry shape.** The invariant is written (*"a refusal that depends
  only on structure still stands"*) and is testable. Which of the three
  mechanisms implements it is engineering.
- **Where the date lives, and the evidence-row type** (N-I5). Mechanism 1 is
  already a commitment; this is typing.
- **Mechanism 4's `span` vs `source_verified_at`** (N-I6 clause 4) — falls out
  of the previous item.
- **`--md-only` vs `--experimental`** (N-M5) — a CLI surface decision with a
  vector.

**Neither — a command (the five Minors, N-I6's clauses 3 and 5, both Nits).**
These are unpropagated clauses and missing enumerations. `N-M1` is two sentences
in one file that contradict; `N-M2` is one word (*positive* → *positive or
refusal*); `N-M4` is four names that `grep -n 'Tag::Sha256' -A 3
crates/md-codec/src/render.rs` prints; `NEW-I1` is one `grep` against
`md/md.go`; `NEW-M2` is `test -f`. **Not one of my five new findings required
design judgement to spot.** Two came from grepping the fork for a type the
design cites, one from diffing two sentences in the same subsection, one from
`find`, one from `grep -c`.

**Why that settles it.** The yield of the thing a review round is *for* has
fallen monotonically — r1 4C, r2 4C/11I, r3 1C/7I, r4 0C/2I — while the share of
findings that are propagation failures has risen to nearly all of them. This is
the repo's own documented shape: *three occurrences is a wrong shape — stop
fixing instances, add the boundary, state the rule at it.* This document has now
had two-spellings-of-the-key found in three consecutive rounds (locks in r3,
digests in r4) and a clause-not-propagated-to-§3-or-§5 in all three. Paying opus
rates to find the fourth instance one at a time is the thing
`machine-checkable claims get machine-checked, never reviewed` exists to
prevent, and this repo already owns the tool: `scripts/fold-propagation-check.sh`
found the `multi(2,@0,@1,@2)` instance for r3 in one command.

**So, concretely, instead of r5:**

1. One authoring pass settling the three design items above.
2. `scripts/fold-propagation-check.sh` over the fold's own changed vocabulary
   (`sha256(#`, `positive until re-measured`, `md shape-key` vs `xtask`), plus a
   claim-check resolving **every type name and file:line the design cites**
   against `descriptor-mnemonic` `b2c5d693` and `seedhammer` `7b6f2fb`. That
   pass alone closes NEW-I1, NEW-I2, NEW-M2, N-M1, N-M2, N-M4 and N-M5.
3. Write plan 1, and let its R0 gate resolve N-I3, the date's home, the
   evidence-row type and `--md-only`.

**The one thing not to do** is skip to the plan with the key's format unsettled.
Everything else in the residue is cheap to fix late; the key is the single
artifact that becomes expensive the moment plan 1 commits an evidence table
against it.

---

## What I checked and did not find wrong

- `canonicalize.rs:168` is `pub fn canonicalize_placeholder_indices` — r2 M-1's
  fix is correct on the merits (only the crate path is missing).
- `validate.rs:449-458` really is the engraved-card-unreadable regression C-4
  invokes, in its own words.
- The `KeyPathNone` → `NotTaproot` rename is faithful: the fork documents it as
  *"not a taproot policy"* (`md/policy_shape.go:34`).
- `(xpub bytes, origin_path)` is the right resolution of "derivation":
  `ExpandedKey` carries `origin_path`, `use_site_path`, `fingerprint:
  Option<[u8;4]>` and `xpub: Option<[u8;65]>` (`canonicalize.rs:352-364`), so
  both absence spellings exist and neither is a value.
- The two-key-spellings defect r3 raised **is** closed: `grep 'blocks#1'` →
  4 hits, all `older-blocks#1` except the one inside the explicit retraction;
  `grep 'multi(2,@0,@1,@2)'` → 0.
- Mechanism 3's new claims are factually right: `git ls-files harnesses/`
  returns the three Liana files; Nunchuk's and Core's are not in the repo.
- NUMS is distinguishable in the template today (`render.rs:87-88` emits
  `NUMS_H_POINT_X_ONLY_HEX`), which is why I rated the I-9 residual Important
  rather than Critical.
- The six operator rulings, the three-plan split, the skeleton-key choice, the
  no-device-clock decision and the Core boundary are untouched by this fold and
  were out of scope.

## Blind spots of this review

I built nothing: no `md-codec` compile, no prototype `Skeleton`, no harness run,
no `xtask`. NEW-I1 and NEW-I2 come from reading `md/md.go`,
`gui/composer_consent.go` and the design end to end, not from a failed build. I
did not re-derive the 56-shape matrix, the Core boundary, F-624/F-626/F-627, the
`descriptor.cpp` citation, or the 289/289 re-measurement — the brief lists them
as settled or controller-verified. I did not review plans 1-3, which do not
exist. Where I marked a fix FIXED I checked the clauses **r3** named; a clause
r3 itself dropped from the opus report would not be visible to me, and r3's own
N-N1 says that report has one more Important than anybody has been counting.
