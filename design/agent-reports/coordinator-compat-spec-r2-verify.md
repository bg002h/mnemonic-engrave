# r2 verification — `DESIGN_coordinator_compatibility.md` second fold

**Artifact:** `design/DESIGN_coordinator_compatibility.md` at `23c51d8e`.
**Input review:** `design/agent-reports/coordinator-compat-spec-opus.md`
(header says 4C/10I/5M/2N; its body carries **eleven** Importants — see N-N1).
**Fold under review:** `git diff 4772aa35..23c51d8e`, 226 insertions / 33
deletions, one file.
**Repos read:** `descriptor-mnemonic` at `b2c5d693`, `seedhammer` at `7b6f2fb`,
this repo at `e80ffead`. Brief:
`design/agent-briefs/coordinator-compat-spec-r2-verify.md`.

**New counts: 1 Critical / 7 Important / 5 Minor / 1 Nit.**

---

## Answers, first

**A. Did the fold close each finding?** Mostly, and this time the clauses were
propagated rather than the conclusions — the failure mode the fold's own
preamble names did not repeat at the level it repeated before. Tally:
**Criticals 3 FIXED / 1 PARTIAL. Importants 5 FIXED / 6 PARTIAL / 0
NOT-FIXED. Minors 1 FIXED / 4 NOT-FIXED. Nits 1 FIXED / 1 PARTIAL.** Every
PARTIAL is a *named clause of the fix that did not land*, not a disagreement
about the verdict.

**B. Is it implementable NOW? Still NO — but the shape has changed.** Three of
the four original blockers are genuinely closed: the template-only verdict is
defined (C-2), the step-1/step-2 circularity is broken by the `xtask` (I-1),
and the "no Go walk" retraction plus the `policy_shape.go` port at the head of
step 1 turns C-1's second half from a gap into costed work. What blocks an
implementer now is inside §1A, the wholly-new part:

1. **`fp_partition` still cannot be computed from the walk the design names.**
   The fork's `Branch` carries `Keys int` — a *count* of distinct placeholders
   (`md/policy_shape.go:245`, `br.Keys = len(keys)`) — and the index set is a
   function-local `map[uint8]struct{}` that is discarded. A verbatim port
   yields no per-path slot ids, so "per path, slots sharing a fingerprint" has
   no source. This is **N-C1**, and it is C-1's own defect relocated one level
   inside its remedy.
2. **`Skeleton` omits `root`**, the first of the two arguments of the rule
   template the same subsection quotes (**N-I1**). Liana class 1 is not
   computable from the struct as written.
3. **The key has no serialized form** (**N-I7**), and the conformance gate
   `chunks -> key == descriptor -> key` needs one.
4. **The document carries two incompatible spellings of the key** — §1A's rules
   versus §1(a)'s worked example at line 187 (I-2 and I-3, both PARTIAL). Two
   implementers reading different halves produce different keys.

**C. Did this fold introduce a defect?** Yes — several, all in new text, and
one is Critical (N-C1). The fold also left §3's staleness section asserting
facts that a commit already in this tree (`606ab180`) falsified, while adding
a table row that acknowledges the same commit (**N-I6**).

**Push-hardest question — the nine Liana classes.** 8 of 9 are computable from
`Skeleton` once `PolicyShape` is ported; class 1 is not (N-I1) and class 2 is
ambiguous between two fields that disagree (N-I2). Detail below.

**Push-hardest question — is `kind#class` unambiguous?** The *definition* is:
all four sub-decisions I-3 named are answered. The *document* is not, because
its only worked key example uses the retired `blocks#1` vocabulary. Fix line
187 and `kind#class` is reproducible; the key as a whole still is not, for
N-I7's reason.

---

## A. Per-finding verdicts

### Critical

**C-1 — `Skeleton` undefined, key uncomputable in Rust — PARTIAL.**
All three clauses of the prescribed fix are textually present:

| clause | landed? | where |
| --- | --- | --- |
| define `Skeleton` as a struct: template, per-path fp partition, whole-policy (xpub, derivation) partition, semantic decomposition | yes | lines 99-106 |
| put the `policy_shape.go` port back at the head of step 1 | yes | line 437 |
| retract "no Go walk needs porting" | yes | lines 194-215 |

What did not land is the thing the clauses were for: the per-path partition is
still not derivable. See **N-C1**. Marking this PARTIAL rather than FIXED is
deliberate — the finding was *"the key cannot be computed"*, and it still
cannot.

**C-2 — template-only verdict undefined — FIXED.** §1(a2) (lines 217-226)
states the rule (`Unproven { reason: KeysAbsent }`), preserves structural
refusals explicitly (*"A refusal that depends only on structure … still
stands"*), carries `keys_present: bool` on `Skeleton`, extends it to the
device's template-only consent, and step 2's gate names the case. The
mechanism by which an implementer decides *which* rules read key identity is
missing — that is **N-I3**, a new defect in the new text, not an unclosed
clause.

**C-3 — `renderer` reaches only `Imports` — FIXED.** `Refuses` gains
`renderer: Option<RendererId>`, `ImportsAltered` gains `renderer: RendererId`,
and lines 82-89 explain why `Refuses` alone is optional (rule-derived versus
measured) with the F-624 measurement as the motive. Both halves of the
finding — measured refusals and `ImportsAltered` — are addressed. §3's
mechanism 5 was not updated to match (**N-M2**, Minor).

**C-4 — `md descriptor` is a read path — FIXED, including the clause.** Lines
416-426 state the split, cite `validate.rs:449-458` as the precedent, and land
the safe reading in full: *"It prints the same notice on stderr and exits 0"*
— the "warn loudly on the read path" half, which is exactly the kind of clause
the first fold used to drop. Step 2's gate adds *"`md descriptor` NOT
refusing"*.

### Important

**I-1 — step 1's artifact built by step 2's binary — FIXED.** Line 440-445:
the generator is a step-1 `xtask`, `md shape-key` is a thin CLI over the same
library function, and a test asserts the two agree. Both sub-questions the
finding raised (the circularity, and where the table comes from) are answered.
§2's older sentence was not updated and now contradicts it — **N-M1**, Minor.

**I-2 — the key drops `/<0;1>/*` — PARTIAL.** The rule is stated and reasoned
(lines 145-149, *"`template` keeps `/<0;1>/*`"*, with the correct reason that
the key must agree with the evidence artifact it is looked up against). The
finding's closing instruction was *"Keeping it makes the design's illustration
wrong. Say which."* — the design said which and left the illustration wrong.
Line 187 still reads `multi(2,@0,@1,@2)`. Machine-checked:

    $ scripts/fold-propagation-check.sh design/DESIGN_coordinator_compatibility.md 'multi\(2,@0,@1,@2\)'
      LEFT   multi\(2,@0,@1,@2\)
               187:    wsh(or_d(multi(2,@0,@1,@2),or_i(pkh(@3),and_v(v:pkh(@4),older(blocks#1)))))

**I-3 — `kind#class` never defined — PARTIAL.** The definition (lines 155-162)
answers all four sub-decisions: vocabulary is the four `LockKind`
discriminants; the counter is per-kind over distinct values; base 10 from 1;
order is the canonical template's left-to-right traversal. The worked example
(`older(26280)`, `older(1000)`, `older(26280)` → `#1, #2, #1`) is
self-consistent. Verified against the fork: `LockKind` really is four-valued
(`md/compose.go:60-73`) and `lockFromWire` (`md/compose.go:217-232`) recovers
the kind from the BIP-68 bit-22 flag and the BIP-65 500,000,000 threshold, so
the vocabulary is derivable from the wire.

The PARTIAL is line 187 again: it spells the same component `blocks#1`, a
vocabulary §1A retired 27 lines earlier. The document therefore teaches two
key formats, and the one an implementer is most likely to copy is the worked
example.

**I-4 — absent fingerprint undefined — PARTIAL.** The main rule landed and is
right: lines 164-168, *"An absent fingerprint is its own singleton
partition"*, with md-codec's own sentinel ruling as the ground. Sub-clause (a)
landed: `key_partition` (whole-policy, `(xpub, derivation)`) is now a field.
Sub-clause **(b) did not land at all** — `expand_per_at_n` can fail
(`crates/md-codec/src/canonicalize.rs:445` returns `Result`;
`validate.rs:431-440` swallows the failure on purpose, *"a missing explicit
origin, a dead card"*), and the finding said `md shape-key` cannot swallow it.
`grep -i expand` over the design returns nothing. An implementer still has to
decide between erroring out (which collides with C-4's "`md descriptor` never
refuses") and emitting empty partitions (which gives a keyed card the same key
as a template-only one). Note the third state this creates:
`keys_present == true` **and** unexpandable is covered by neither
`keys_present` nor `UnprovenReason::KeysAbsent`.

**I-5 — `Unproven` carries no span and no reason — FIXED.** `Unproven {
reason: UnprovenReason, span: Option<Span> }`, `UnprovenReason` enumerated,
`Span` defined, and `Version`'s ordering source stated (*"ordered by the
registry's declared order"*) — all four things the finding asked for.

**I-6 — the "named class" is not named — PARTIAL.** The D1-D4 table landed
with resolutions, and D3 is anchored to the F-633 measurement. The finding's
last clause did not land: *"the coordinator's error string is mapped to a
reason class and compared"*, which the finding identified as *"the only thing
that makes the third row checkable."* `Reason { class: &'static str, cite }`
is a free string on our side and the evidence carries the coordinator's own
free-text message; nothing in §2 says how the two are brought into one
vocabulary. D3 is therefore a named class with no procedure.

**I-7 — `ImportsAltered`'s `as_read` — PARTIAL.** Two of the three questions
are answered well: `Description` is defined (line 128-132), and it is
**derived by the harness, never hand-authored**, with `ImportsAltered` firing
automatically — the architect's dropped clause, restored verbatim in spirit at
lines 290-297. The third is not: *in what space is the comparison made?*
`paths: Vec<String>` is *"the coordinator's rendering of each spend path"*,
and md's decomposition renders differently, so a naive field-wise comparison
marks **every** row `ImportsAltered`. `threshold: Option<(u8,u8)>` alone would
catch both measured instances (Liana's 2-of-4 on X24, Nunchuk's 0-of-3 on
F-626). Which fields participate is a materially different implementation and
the design does not say.

**I-8 — the none-case flag is never named — FIXED.** `--md-only` (line 419),
and the related sub-question is answered too: *"The flag is on `md compose`
alone"*. Verified free: `grep -rn "md-only" crates/` in `descriptor-mnemonic`
returns nothing. (Its relationship to the existing `md compose --experimental`
is unstated — **N-M5**, Minor.)

**I-9 — the rendered-template key cannot carry the internal-key kind —
PARTIAL.** `KeyPathKind { Nums, UnspendableXpub, Spendable, NotTaproot }` is
defined and `key_path` is on `Skeleton`. But the finding was about the **key**,
not the rule input: *"A text key cannot carry a marker that is not in the
text — so this needs a structured component, decided now."* §1(a) still
defines the key as the abstracted template plus the fp-partition, and says
nothing about `key_path`. The day F-449 lands, an unspendable-xpub internal key
renders `@i/<0;1>/*` — byte-identical to a spendable key path — so the two
still collide **in the key**, and every committed key for those shapes still
becomes wrong. The datum now exists; the lookup index still cannot carry it.
(Also **N-I2**: it is duplicated by `shape.KeyPath`, which is three-valued.)

**I-10 — the version a verdict names — FIXED, both clauses.** Lines 354-358
state the rule (*"the one the APPLICATION displays"*) **and** keep the library
revision as provenance on the evidence row, which was the second half of the
architect's I-6.

**I-11 — mechanisms assigned to no plan — FIXED as an assignment.** The table
(lines 362-369) maps all six. Mechanism 4 to plan 2, which resolves the
"plan 1 cannot close green" fork the finding raised, and the design says so
("Mechanism 4 being red on the day it lands is intended"). The row's *content*
is falsified by this tree — **N-I6**.

### Minor

| finding | verdict | evidence |
| --- | --- | --- |
| **M-1** doc-comment citation | **NOT-FIXED** | line 184 still cites `encode.rs:59-62`; those lines are verbatim doc-comment prose (`/// This delegates to [encode_payload]…`, measured at `b2c5d693`). The correct citation (`canonicalize.rs:168`) appears only in the retraction 15 lines later, so the load-bearing sentence still points at a doc comment. |
| **M-2** the key's blind spots | **NOT-FIXED** | `grep -i blind` returns nothing. |
| **M-3** gate against a sixth hand-written notice | **NOT-FIXED** | `grep -i composerCopy` returns nothing. |
| **M-4** digests get a kind but no class | **FIXED** | line 161-162 gives the reason: *"no coordinator measured distinguishes two digests."* A reasoned decline closes it. |
| **M-5** "self-reported version, never a label typed by the person running it" | **NOT-FIXED** | `grep -i self-report` returns nothing; mechanism 3 still asks only for the built source revision, which the finding already said is a different fact. |

### Nit

**N-1 — the status header — FIXED.** Replaced by *"Status: DRAFT, second
fold"*; nothing is now labelled "approved".

**N-2 — 1-based classes beside 0-based slot ids — PARTIAL.** The class base is
now explicit ("base 10, starting at 1") and slot ids are placeholder indices,
so the asymmetry is at least deliberate; the same two-line illustration still
shows both and never says why.

---

## C. New findings

### Critical

**N-C1 — the per-path fingerprint partition is still not computable from the
walk the fold names, because `Branch` carries a key COUNT and not the key
INDICES.**

*At section 5 step 1, an implementer must decide where per-path slot ids come
from, and the design's answer does not contain them.*

The fold's retraction is explicit about the source: *"The **per-path**
fingerprint partition needs a spend-path decomposition that exists **nowhere
in Rust** … the branch split lives only in the fork's Go
`md/policy_shape.go`"* — and step 1's first task is to port it. Measured
against that file at `7b6f2fb`:

    type Branch struct {
        K, N int
        Keys int          // <- a COUNT
        Timelock bool
        Hashlock bool
        Locks    []Lock
        Hashlocks []*HashLock
        Sorted bool
        Depth int
    }

and `branchOf` (line 239-245):

    keys := map[uint8]struct{}{}
    if !collect(n, &br, keys) { return Branch{}, false }
    br.Keys = len(keys)

The placeholder index set exists for exactly the duration of that function and
is then thrown away. `splitBranches` does split `or_b/or_c/or_d/or_i` into
alternatives, so the *paths* in the design's illustration are real — but
nothing on the resulting `Branch` says which `@N` belongs to which path. So a
faithful port produces the branch split and still cannot answer *"which slots
are in path 0"*, which is the whole input to `fp_partition`.

Two consequences the design does not cost:
- The port is not a port. It must **extend** `PolicyShape` with per-branch key
  indices — and under the Rust-primary rule the Go must then converge to the
  extended type, which is fork work the three-plan split places in plan 3.
- §1A says `PolicyShape` *"is the fork's existing `md/policy_shape.go`"* — an
  implementer told to port that file verbatim produces a `Skeleton` whose
  `fp_partition` cannot be filled, and the failure appears only when the
  conformance gate runs.

*Fix.* Say that the port adds a per-branch slot-index list (the `keys` map
`branchOf` already builds), and name the Go convergence as plan-3 scope.

### Important

**N-I1 — `Skeleton` omits `root`, the first argument of the rule template the
same subsection quotes; Liana class 1 is not computable from it.**

§1A argues correctly that `Skeleton` must be a struct, and quotes the
signature: *"`composerLianaOutsideModelClass`'s real signature is `(root
md.ScriptKind, shape md.PolicyShape)`"*. It then defines a struct carrying the
second argument and not the first. The fork says why this cannot be papered
over (`gui/composer_consent.go:362-364`): *"`root` is the DECODED wrapper …
because PolicyShape does not carry it: policyShape walks tagWsh and tagSh
identically, so a legacy sh wrapper is indistinguishable from wsh on
shape.Branches alone."* `root` comes from `Template.Root` (`md/md.go:1207`),
a different type entirely.

With `refuse: fn(&Skeleton) -> Option<Reason>`, class 1 (`legacy wrapper`) has
exactly one recovery: re-parse `template`'s leading token — which the same
subsection forbids four lines above (*"none of which is recoverable from a
template string"*). Add `root: ScriptKind` to `Skeleton`.

**N-I2 — two fields answer the internal-key question, they disagree, and the
ported rule reads the one that cannot see an unspendable xpub.**

`Skeleton` carries `key_path: KeyPathKind` (four-valued, the I-9 fix) *and*
`shape: PolicyShape`, whose own `KeyPath` is three-valued at `7b6f2fb`
(`md/policy_shape.go:32-41`: `KeyPathNone`, `KeyPathNUMS`,
`KeyPathSpendable` — no unspendable-xpub member). Liana class 2 is written
`if shape.KeyPath == md.KeyPathNUMS`, and class 7's unlocked-path count reads
`shape.KeyPath == md.KeyPathSpendable`. A rule ported verbatim therefore reads
`shape.KeyPath`, sees `Spendable` for an unspendable xpub, and I-9's
distinction is invisible at exactly the site the design added it for. The
design states no precedence and does not say whether the port widens
`PolicyShape.KeyPath` or leaves it narrow. Pick one field.

**N-I3 — §1(a2)'s template-only rule is not decidable from `RuleSet` as
typed.**

The rule is *"when `keys_present` is false, every coordinator **whose rule
reads a key-identity component** yields `Unproven { KeysAbsent }`"*, while a
purely structural refusal still stands. But `refuse` is an opaque
`fn(&Skeleton) -> Option<Reason>` — nothing declares its read-set, and a
function pointer cannot be introspected. An implementer must either annotate
each `RuleSet` (a field the type does not have), split `refuse` into
structural and identity halves, or tag each `Reason` with whether it used key
identity. Three materially different registries, and the wrong choice puts a
false `Imports` on `md compose`'s most common output — the outcome C-2 exists
to prevent.

**N-I4 — `key_partition` is new and the absence rule was not applied to it.**

The fold added `key_partition: Vec<Vec<u8>>` (*"whole-policy, slots sharing
(xpub, derivation)"*) and wrote the absence rule for **fingerprints only**.
The same question arises one field over: `ExpandedKey.xpub` is
`Option<[u8; 65]>` (`canonicalize.rs:361-363`), so slots can lack an xpub, and
grouping two absences asserts the same key-identity relation nobody measured —
the design's own words for why it refused to do that for fingerprints.
Separately, *"derivation"* is not resolved: `ExpandedKey` carries **both**
`origin_path` and `use_site_path`, and the two give different partitions for a
policy whose slots share a use-site but differ in origin. This is the
repo's "the defect class moves into the remedy" shape — the rule was written
for the instance the reviewer pointed at.

**N-I5 — no type carries the date that mechanism 1 and §3.1's mandated copy
require, and mechanism 1 is assigned to plan 1 (the types).**

Mechanism 1: *"Every verdict carries `(version verified, renderer, date)`"*,
and the copy it mandates is *"Liana 8.0 imports (2026-09); newer:
unmeasured"*. §1A defines `Verdict`, `Span`, `Version`, `RendererId` — and no
date anywhere. The plan-ownership table the same fold added assigns mechanism
1 to plan 1, i.e. to the very types that omit it. The device cannot print the
sentence §3.1 says it must.

**N-I6 — §3's staleness section asserts three things this tree already
falsified, and the fold added a row acknowledging the commit that falsified
them.**

`606ab180` (*"harness: commit the Liana harness and re-measure at v15.0 —
289/289 verdicts UNCHANGED (F-633)"*) is an **ancestor of the fold**
(`git merge-base --is-ancestor 606ab180 23c51d8e` → yes). Against the tree at
`e80ffead`:

| design text | measured |
| --- | --- |
| line 335-338: *"Today every harness lives in `/scratch/.tmp`, outside all three repos — only the consumer … is committed, never the producers."* | **False.** `git ls-files harnesses/` returns `harnesses/liana/{Cargo.toml,README.md,src/main.rs}`. Nunchuk's and Core's producers are still in `/scratch/code/shibboleth/.tmp`, so the claim is true only with a narrower quantifier. |
| line 491: *"**Liana must be re-measured at v15.0** (F-633)"*, under **"Open, and blocking a spec"** | **Done.** 289/289 verdicts identical; F-633's own entry records it and says the residue is that the notice is *unqualified*, plus a class-order revisit. |
| line 473-474: plan 2 *"is also the plan that closes F-633, because re-measuring Liana at v15.0 needs a committed harness"* | Already closed in that respect; the remaining F-633 work is §8x copy, which is not plan 2. |

And the new table row *"4 … **red today**: Liana 8.0 verified vs 15.0 known"*
now turns on an undefined term. Mechanism 4 fires when *"the newest known
release is more than one major past **every verified version**"*. Liana's
rules were read at v8.0 (`source_verified_at`), but its **evidence** now
exists at v15.0. If "verified" includes a measurement, the gate is green for
Liana and the design's flagship example of it firing is wrong; if it means
source-read only, it is red. `RuleSet` has both `span` and
`source_verified_at` and the design never says which one mechanism 4 reads.

Lastly, §3's premise paragraph still presents *"the import-time `InvalidPolicy`
refusal class no longer exists"* as the live hazard, with no mention that the
measurement came back 289/289 unchanged. That is the design's own staleness
lesson applied to everything except itself.

**N-I7 — the key's serialized form is undefined, and three `Skeleton`
components have no stated membership in it.**

`md shape-key` *"emits the key"*, and step 1's gate asserts `chunks -> key ==
descriptor -> key` for every evidence row — an equality over a string nobody
has specified. §1(a)'s illustration is prose, not a format:

    wsh(or_d(multi(2,@0,@1,@2),or_i(pkh(@3),and_v(v:pkh(@4),older(blocks#1)))))
    | fp-partition per path: [{0},{1},{2}] [{3}] [{4}]

Undecided: whether the key is one line or two; the separator; whether
`key_partition` (new this fold) and `key_path` (new this fold) are in the key
at all — I-9's defect is only fixed if `key_path` is; and the declared type
`fp_partition: Vec<Vec<u8>>` is **two levels deep** while the illustrated
value is three (paths × groups × slots), so the type as written cannot hold
the example beside it. Pick the serialization and make the illustration an
instance of it.

### Minor

**N-M1 — §2 and §5 step 1 now contradict each other on who computes the
table's keys.** Line 299-300: *"The generator computes keys by calling `md
shape-key`, never by reimplementing the canonicaliser in Python."* Line
440-442: *"**The table generator is a step-1 `xtask`, not `md shape-key`**"*.
The intent is reconcilable (both call one library function) but the sentences
are not, and §2's is the one stated as a rule.

**N-M2 — the C-3 fix did not propagate to the mechanism that implements it.**
§1 says a measured refusal is retired by the renderer gate *"exactly as it
retires a positive"*; §3 mechanism 5 still reads *"Any byte change retires
every Nunchuk **positive** until re-measured"*. An implementer building the
gate from §3 retires half of what §1 requires.

**N-M3 — `PolicyShape.Complete` is embedded and its contract is never
mentioned.** The fork's type says *"FALSE means the walk met a node it could
not classify, and the caller **MUST NOT** present any part of this summary"*.
`Skeleton` embeds `PolicyShape` unconditionally, no rule mentions `Complete`,
and `UnprovenReason` has no variant for it — so §1(a)'s promise that *"a
foreign md1 with an unfamiliar fragment falls to `Unproven`"* has no mechanism
on the rule side. A rule reading `shape.Branches` of an incomplete shape can
produce a confident refusal from a summary its own author forbade presenting.

**N-M4 — the digest kind vocabulary got the treatment locks got, minus the
enumeration.** I-3 was closed by naming all four `LockKind` values; the digest
rule names one example, `sha256(#)`. md-codec renders four
(`render.rs:199-202`: `sha256`, `hash256`, `ripemd160`, `hash160`) and the
fork's hashlock work ships all four. The retraction's phrase *"the two hash
renderers"* is accurate about the two helper *functions* and reads as a count
of kinds.

**N-M5 — `--md-only`'s relationship to `md compose --experimental` is
unstated.** `--experimental` already exists on that exact command
(`crates/md-cli/src/main.rs:303-305`, *"Admit key-less paths and
unsorted-where-sorted-was-legal, with a warning"*). A none-case policy built
from a key-less path needs both flags; the design says nothing about whether
they compose, which subsumes which, or what a user who passes only one sees.

### Nit

**N-N1 — the input review's own Important count is one short.** Its header and
this brief both say 10 Important; the body carries `I-1` … `I-11`. Machine
count: `grep -c '^### [CI]-'` → 15 (4 C + 11 I). A tally reported as "10 of 10
Importants closed" would silently drop one.

---

## What I checked and did not find wrong

- The `kind#class` definition itself is complete and derivable from the wire:
  `LockKind` is four-valued (`md/compose.go:60-73`) and `lockFromWire`
  (`:217-232`) recovers kind from BIP-68 bit 22 and the BIP-65 500,000,000
  threshold, so the four names are computable, not conventions.
- Liana classes 3-9 are all computable from `PolicyShape` as it stands:
  `!anyLock`, `hash`, `after`, `olderUnits`, `unlocked == 0`, the
  same-`older`-value pair (`Lock.Value` is carried, in operator units) and
  `unlocked >= 2` read only `Branches`, `Locks`, `Hashlocks` and `KeyPath`.
  `splitBranches` really does split `or_d`/`or_i` recursively, so a wsh policy
  yields the three paths the design's illustration shows.
- The absent-fingerprint rule is right and correctly grounded:
  `TlvSection.fingerprints` is `Option<Vec<(u8,[u8;4])>>` (`tlv.rs:27`) and
  `ExpandedKey.fingerprint` is `Option<[u8;4]>`, so both absence spellings
  exist and neither is a value.
- `--md-only` does not collide: `grep -rn "md-only" crates/` in
  `descriptor-mnemonic` at `b2c5d693` returns nothing.
- C-4's fix is correct on the merits — `validate.rs:449-458`'s recorded
  regression really is the precedent, and "refuse on mint, notice on read,
  exit 0" is the safe side of it.
- The three-plan split, the six operator rulings, the no-device-clock decision
  and the skeleton-key choice are untouched by this fold and were out of
  scope.

## Blind spots of this review

I built nothing: no `md-codec` compile, no prototype `Skeleton`, no harness
run. N-C1 is from reading `md/policy_shape.go` end to end at `7b6f2fb`, not
from a failed port. I did not re-derive the 56-shape matrix, the Core
boundary, F-624/F-626/F-627, or the `descriptor.cpp` citation — the brief
lists them as settled or controller-verified. I did not review plans 2 or 3,
which do not exist yet. Where I judged a fix "FIXED", I checked the clauses
the prior report named; a clause the prior report itself dropped from the
architect's text would not be visible to me.
