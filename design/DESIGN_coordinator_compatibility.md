# DESIGN — which wallet coordinators will take this policy

**Status: DRAFT, third fold. Nothing in this document is still "as approved" —
sections 1-2 were approved and have since been revised twice under review, so
treat the whole of it as current draft rather than reaching for which
sentences carried an approval (r3 N-1).** Reviewed four times, all folded here: the fable architect
(`…-fable-architect.md`, 4C/7I/4M/2N); an opus spec review with an
implementability walk (`…-spec-opus.md`, 4C/**11**I/5M/2N — the body has
eleven Importants, not the ten its own header claimed); a re-review of the
second fold (`…-spec-r2-verify.md`, 1C/7I/5M/1N new); and r4
(`…-spec-r4.md`, 0C/2I/2M/1N new), whose judgement was that a fifth design
round would be the wrong instrument and whose recommended sequence this
revision follows.
Brainstorm 2026-09-20. Not a spec yet; plan 1a and plan 1b are both planned
straight from this document.

**Fourth fold (2026-09-23), before plan 1b.** Every change is tagged with its
source: **[recon]** = `design/agent-reports/coord-compat-1b-recon.md`
(measured at descriptor-mnemonic `d269c556`, fork `2c9eed3`);
**[core-boundary]** = `design/agent-reports/coord-compat-core-boundary.md` and
`design/evidence/coord-compat-core-boundary/`; **[R-1]**, **[R-2]** = the
controller rulings in `design/CONTINUITY_coord_compat_1b.md`; **[plan 1b]** =
measured while authoring `IMPLEMENTATION_PLAN_coordinator_compat_1b_verdicts.md`
against a scratch build of `d269c556`. Citations below are re-measured at
`d269c556` (descriptor-mnemonic) and `2c9eed3` (fork) unless a line says
otherwise.

The architect's verdict was *"the family of architecture is right; the
instance is not yet safe"*, and it still stands — the registry, the
source-derived refusals, the measured-only positives and the
coordinator-independent key are unchanged.

**What the second review found, and it was mostly the first fold's fault.**
That fold folded the architect's *conclusions* and dropped a named clause of
his *fix* in seven places, which is this repo's documented
incomplete-propagation failure. It also introduced a defect of its own: it
promoted an aside — that the `PolicyShape` port is "a display concern, not a
normative one" — into normative text, deleting from the plan the exact walk
the design's own rule template consumes. **The claim "no Go walk needs porting
for the normative path" is retracted below**; it was the one line a plan would
have been costed from.

This fold therefore propagates *clauses*, not conclusions, and defines every
type the data model names — five of them appeared exactly once each, inside
the struct that used them.

## The ask

The SH2 composer should tell the operator which wallet coordinators
(Nunchuk, Liana, Bitcoin Core) will import the policy they have just built:
all three, a named subset, or none — and "none" must be loud.

## Operator rulings (settled; do not re-litigate)

1. **Core is reported as a VERSION BOUNDARY**, not a yes/no. Measured over 56
   shapes, Core 25.0 accepts 39 and Core 31.1 accepts 54 — fifteen shapes
   differ, mostly tapscript miniscript — and the device cannot know which Core
   the operator runs. The exact boundary release **must be measured**, not
   inferred from the 25-vs-31 gap. **MEASURED [core-boundary]: 26.0.** On
   official release binaries (sha256 and guix.sigs verified) 24.2, 25.0 and
   25.2 refuse all 15 discriminating `tr` shapes and 26.0, 26.2, 27.2, 28.4,
   29.4, 30.3 and 31.1 accept all 15 with md's addresses; no release lies
   between 25.2 and 26.0. The "Core 25.0" of the 56-shape matrix identifies
   itself as `Bitcoin Satellite v0.2.4`, not Core; its refusals agree with the
   genuine 25.0/25.2 binaries. A second, SEPARATE boundary: the `<0;1>`
   multipath spelling is refused by `getdescriptorinfo` through 28.4 and
   parsed from 29.4 (not bracketed tighter).
2. **The "none" case is a loud confirm-to-proceed**, not a refusal. A policy no
   coordinator imports is still restorable by `md`, and §8a already handles
   exactly this for key-less paths as a confirm-to-proceed. The advanced choice
   stays available; the operator cannot claim they were not told.
3. **Positive where measured, silent where not.** Three verdicts per
   coordinator: a refusal (rule-derived, always stated), an import (stated ONLY
   where a measured shape backs it), or silence. The device never asserts a
   positive it cannot support.
4. **Rust first.** The classifier is normative behaviour, so it lands in
   `md-codec` with test vectors, and the fork's Go port is the downstream
   convergence. One rule set, two consumers: `md` on the host and the SH2
   consent screen.
5. **`md` refuses the "none" case** and names an explicit flag to proceed
   (exit non-zero). No TTY prompt: this project has already shipped a tool that
   blocked on a TTY and looked like a hang.
6. **Designed for growth**: more versions of each coordinator, and more
   coordinators, are expected.

## Controller rulings before plan 1b (settled; do not re-litigate)

Recorded in `design/CONTINUITY_coord_compat_1b.md`, 2026-09-23.

- **R-1, the Nunchuk leaf order.** The SkeletonKey stays
  coordinator-independent. Nunchuk's kind-1 cell is a source-derived,
  KEY-DEPENDENT rule: it accepts only if the card's leaf pubkeys are already in
  sorted, unique order (libnunchuk `a7cfb49` `src/descriptor.cpp`: the
  `GetUnspendableXpub` re-derivation at `:689-712` and the re-render check at
  `:640-648`). A template has no keys, so it is `Unproven { KeysAbsent }`.
  Cost if wrong: evidence keyed by skeleton alone would need a key-order field
  added later. **How the rule stays inside ruling 3:** the rule only REFUSES
  (unsorted); "accept iff sorted" is carried by a measured positive for the
  sorted card, not asserted by the rule.
- **R-2, evidence transport.** Evidence recorded in mnemonic-engrave is
  VENDORED into descriptor-mnemonic by a script with a freshness check (the
  `vendor-liana-evidence.sh` pattern), and the md-codec xtask builds the table
  from the vendored copy. Cost if wrong: a second vendoring path to maintain.

## Section 1 — the data model (REVISED after review)

The unit is not a coordinator; it is a **coordinator at a version range**.

```rust
struct Coordinator { id: CoordinatorId, name: &'static str, rules: &'static [RuleSet] }

struct RuleSet {
    span: Span,                          // both ends a version actually verified
    source_verified_at: &'static [Version],
    reads: ReadSet,                      // which Skeleton fields this rule may touch
    refuse: fn(&Skeleton) -> Option<Reason>,
}

/// Declared, because §1(a2)'s template-only rule is otherwise undecidable: you
/// cannot ask an opaque `fn` pointer whether it reads key identity (r4).
bitflags ReadSet { STRUCTURE, KEY_IDENTITY, LOCK_VALUES, DIGESTS }

enum Verdict {
    // `renderer` is on EVERY MEASURED verdict, not on `Imports` alone (r2 C-3).
    Refuses { reason: Reason, span: Span, renderer: Option<RendererId> },
    ImportsAltered { as_read: Description, span: Span, renderer: RendererId },
    Imports { span: Span, renderer: RendererId },
    Unproven { reason: UnprovenReason, span: Option<Span> },
}
```

`renderer` is `Option` on `Refuses` alone, and the option is the whole point:
`None` means the refusal is **rule-derived** (a claim about source at the
versions read, spelling-independent); `Some` means **measured**, and then
§3's renderer gate retires it exactly as it retires a positive. Nunchuk's
refusals are entirely renderer-dependent — F-624 measured the `--chain 1`
spelling refused 30/30 while multipath imported 20/20, same shape — so a
measured refusal with no renderer would print a claim the evidence does not
support.

**Core's verdict is renderer-dependent too [recon §3b, core-boundary].** The
paragraph above presented renderer dependence as Nunchuk's property. Core
26.0-28.4 import the per-chain spelling of a policy and refuse its `<0;1>`
spelling with `Key path value '<0;1>' is not a valid uint32`, so a Core
verdict names its form as well. Plan 1b's measurement found the order too
[plan 1b]: Core parses key expressions before the script, so on 24.2-25.2 a
multipath `tr` miniscript is refused for its `<0;1>`, not for its miniscript
— a spelling refusal is checked FIRST.

### 1A. Types — normative

The review found `Skeleton`, `Span`, `RendererId`, `Description` and
`CoordinatorId` each named once and never defined; an implementer cannot start
without them.

```rust
/// Everything a rule may read. Computed from a DECODED md1 only.
struct Skeleton {
    root: ScriptKind,          // the fork's SIX: Wpkh|Pkh|Sh|Wsh|Tr|ShWpkh
    inner_wsh: bool,           // sh(wsh) is NOT a ScriptKind value (r4 NEW-I1)
    template: String,          // canonical @i template, use-site KEPT (r2 I-2)
    shape: PolicyShape,        // the semantic decomposition; carries KeyPath
    fp_partition: Vec<Vec<Vec<u8>>>,  // [path][group][slot]: slots sharing a fingerprint
    key_partition: Vec<Vec<u8>>,      // whole-policy [group][slot]: same (xpub, origin_path)
    keys_present: bool,        // false for a template-only payload (r2 C-2)
    // [R-1] key MATERIAL, NOT in the key: for `tr`, whether the tap-leaf key
    // occurrences' pubkeys, in wire order, are strictly ascending (sorted and
    // unique). None when not `tr` or no keys. Nunchuk's Liana-key rule reads it.
    leaf_keys_ascending: Option<bool>,
    // NO key_path field: it would duplicate shape.KeyPath, and a ported rule
    // reading the duplicate is how I-9 re-opens. One field, extended below.
}

struct Span { since: Version, until: Version }   // both ends VERIFIED, closed
struct Version(&'static str);   // opaque, ordered by the registry's declared order
struct CoordinatorId(&'static str);
struct RendererId { tool: &'static str, version: &'static str, form: Form }

/// §3.1 requires every VERDICT to carry a date. It lives on the evidence row
/// and is copied onto the verdict — NOT on `RuleSet` beside
/// `source_verified_at`, which is a different fact about a different act
/// (when source was read, versus when a binary was run) (r4 NEW-M1).
/// ISO 8601 date, no clock: the device cannot tell the time (§6b) and never
/// compares this to "now" — it is printed for a human to judge.
struct MeasuredAt(&'static str);

/// One measured cell. The date and the renderer live here and ride onto the
/// verdict the table build produces.
struct EvidenceRow {
    key: SkeletonKey,
    coordinator: CoordinatorId,
    version: Version,          // the APPLICATION's version (§3)
    library_rev: &'static str, // provenance; what the binary self-reported
    renderer: RendererId,
    measured_at: MeasuredAt,
    outcome: MeasuredOutcome,  // Imported | Refused(String) | ImportedAltered(Description)
}
enum UnprovenReason { NoEvidence, KeysAbsent, OutsideEveryVerifiedSpan }

/// Which rendering produced the descriptor a measurement was taken on.
enum Form { Multipath, Chain0, Chain1 }

/// A rule's refusal class: the coordinator's own order-of-refusal name, plus
/// the source that establishes it. `cite` is what makes a rule auditable.
struct Reason { class: &'static str, cite: &'static str }

/// The internal key. **CORRECTION (F-449 stage 1b, Task 10):** this was
/// THREE-valued when drafted, on the claim that an unspendable xpub is NOT a
/// variant here — that recognising one means re-deriving a specific
/// coordinator's own function, so the distinction lives in a rule, not in
/// the key. Stage 1b (shipped, `crates/md-codec/src/policy_shape.rs`) made
/// that claim false for Liana's convention specifically: it gave that
/// convention its own wire discriminant (a version-8 kind bit), so for any
/// descriptor that carries it the distinction now lives in the key. The
/// reasoning is unchanged for `Xpub` itself — a `key_index` the wire gives
/// no discriminant for remains exactly as ambiguous as before. See the port
/// note above, and `KeyPathKind::LianaUnspendable`'s doc in the shipped code
/// for the design reversal in full.
/// `Xpub`, not `Spendable`: this walk verifies nothing about spendability.
enum KeyPathKind { NotTaproot, Nums, Xpub, LianaUnspendable }

/// The coordinator's OWN parsed reading, recorded by the harness — never
/// hand-authored (r2 I-7).
struct Description {
    wallet_kind: String,       // e.g. "MULTI_SIG", "MINISCRIPT", "Liana"
    threshold: Option<(u8, u8)>,
    paths: Vec<String>,        // the coordinator's rendering of each spend path
}
```

`PolicyShape` is the one type this design does **not** define: it is the
fork's existing `md/policy_shape.go`, and step 1's first task is porting it.

**`Skeleton` is a struct, not a string**, because the rules need the semantic
decomposition and a rendered template cannot give it back without a second
parse. `composerLianaOutsideModelClass`'s real signature is
`(root md.ScriptKind, shape md.PolicyShape)` and its body reads
`shape.KeyPath`, `shape.Branches`, `b.Locks[].Kind/.Value`, `b.Hashlocks` —
none of which is recoverable from a template string.

**`template` keeps `/<0;1>/*`** (r2 I-2): both `descriptor_to_template` and the
committed evidence's `template` field carry the use-site, and dropping it in
the key would make the key disagree with the artifact it is looked up against.
The `--chain 0`/`--chain 1` *form* is correctly a renderer property, not a key
property — one decoded card renders both.

**`root` is in the struct because the rule template's first argument is
`root`** (r3). The design quotes
`composerLianaOutsideModelClass(root md.ScriptKind, shape md.PolicyShape)` two
paragraphs above and then defined a `Skeleton` without it — Liana class 1,
"legacy wrapper", is uncomputable without it. The fork's own comment says why
it is separate: *"PolicyShape does not carry it: policyShape walks tagWsh and
tagSh identically."*

**THE PORT IS A PORT PLUS TWO NAMED EXTENSIONS** (r3 C-1 remainder). A verbatim
port of `policy_shape.go` yields a `Skeleton` whose central field cannot be
filled, because:

1. **`Branch` keeps a COUNT, not the slots.** `branchOf`
   (`md/policy_shape.go:239-245` when written; `:265-271` at fork `2c9eed3`
   [recon]) builds `keys := map[uint8]struct{}{}` — the
   placeholder indices the branch references — then writes `br.Keys =
   len(keys)` and **discards the map**. `fp_partition` needs *which* slots,
   per path. So the ported `Branch` must retain the index set.
2. **`KeyPathKind` kept THREE values through plan 1a; stage 1b gave it a
   fourth.** An earlier draft said the ported enum "gains a fourth value" for
   an unspendable xpub. Right that the distinction matters — Nunchuk treats
   it as a different wallet and F-449 records it — **wrong about where it
   lived, measured so during plan 1a:**
   `Body::Tr { is_nums, key_index, tree }` (then `crates/md-codec/src/tree.rs:49-57`;
   at `d269c556` it is `Body::Tr { internal_key: InternalKey, tree }`,
   `tree.rs:69`, with `enum InternalKey { Slot(u8), NumsPoint, LianaUnspendable }`
   at `tree.rs:23-30` [recon])
   made `is_nums` the only internal-key discriminant on the md1 wire, and an
   unspendable xpub was an ordinary `key_index`, structurally identical to a
   spendable one. **CORRECTION (F-449 stage 1b, Task 10): that is no longer
   the wire's shape.** Liana's `unspendable_internal_key(desc)` **derives**
   the key from the descriptor rather than being a constant to match, but
   stage 1b taught the wire a second kind bit at version 8
   (`crate::tree::InternalKey::LianaUnspendable`) precisely so that, for a
   descriptor carrying it, recognising Liana's derivation no longer requires
   re-deriving a coordinator's own function — the computation happens once,
   at encode time, and the wire carries its result. The ported enum is
   `NotTaproot | Nums | Xpub | LianaUnspendable` — `Xpub`, not the fork's
   `Spendable`, because the walk verifies nothing about spendability — and
   the coordinator-rule reasoning now scopes to `Xpub` alone: a `key_index`
   the wire gives no discriminant for is exactly as ambiguous as before, and
   a rule-vs-evidence disagreement over THAT case still surfaces as a D1/D2
   build failure.

Both extensions land in **Rust**, and this is the rare direction: `policy_shape.go`
is fork-native code with no Rust counterpart, so porting it *makes* Rust
primary for it. The fork's Go converges afterwards if it wants the new fields;
nothing in the fork needs them today.

**`kind#class` is defined** (r2 I-3): *kind* is the `LockKind` discriminant
(`after-height`, `after-time`, `older-blocks`, `older-units`); *class* is a
counter over **distinct values of that kind within this policy**, base 10,
starting at 1, assigned in the canonical template's own left-to-right
traversal order. `older(26280)` then `older(1000)` then `older(26280)` renders
`older(older-blocks#1)`, `older(older-blocks#2)`, `older(older-blocks#1)`.
Digests render **with a class too** — `sha256(#1)` — symmetric with locks
(r3 M-4). The asymmetry in an earlier draft was unexplained and wrong for the
same reason locks carry one: two hashlock branches committing to the *same*
digest is a different wallet from two committing to different ones, and the
device composes hashlock paths. Class numbering is per kind, as for locks.

**`key_partition` groups by `(xpub bytes, origin_path)`** — "derivation" in the
architect's clause means the origin path, the only derivation a decoded md1
carries — and **an absent xpub is its own singleton**, by the same argument as
the fingerprint rule below. It exists for Liana's `DuplicateKey`, which is a
whole-policy relation, where `fp_partition` is per-path and exists for
`DuplicateOriginSamePath`. Two relations, two partitions; collapsing them
would make one of the two refusals uncomputable.

**An absent fingerprint is its own singleton partition** (r2 I-4). md-codec has
already ruled on this question for itself: `[0,0,0,0]` is the ABSENT sentinel,
not a value. Two slots whose fingerprint is absent are **not** known to share a
signer, so they never join a partition — grouping them would assert a
key-identity relation nobody measured, which is C-1's failure in miniature.

**Adding a version** is a new `RuleSet`; **adding a wallet** is a new
`Coordinator`. Neither touches the classifier or the display. A coordinator's
row is **a function from verified version to verdict, printed as its runs** —
non-monotonic behaviour (tightens at 9, loosens at 12) is just three rule sets.

Five things the review forced, each closing a measured route to a false
positive:

**(a) The key is the canonical template SKELETON, not a semantic summary.**
`PolicyShape` is the right *display* summary and the wrong key: it is a
semantic reading, and coordinators do not parse semantics, they parse text.
Two md1 trees with identical `PolicyShape` but different fragments (`or_i` vs
`or_d`) can differ in importability. The key is md-codec's **existing**
canonical payload — placeholders renumbered by first appearance
(`crates/md-codec/src/canonicalize.rs:159` [recon]) — rendered as a template with lock values and digests alike replaced by
`kind#class`, and origins erased (one spelling; an earlier draft said "digests
by their kind" here and `sha256(#1)` thirty-five lines later — r4 NEW-I2):

    wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),
             or_i(pkh(@3/<0;1>/*),
                  and_v(v:pkh(@4/<0;1>/*),older(older-blocks#1)))))
    | fp-partition per path: [[{0}],[{1}],[{2}]] [[{3}]] [[{4}]]

The use-site `/<0;1>/*` is KEPT and the lock reads `older-blocks#1`, not
`blocks#1` — an earlier draft of this example spelled both the other way,
which left the document carrying two incompatible key spellings (r3). The
partition nests three deep, `[path][group][slot]`, matching the type.

**The key's blind spots, listed** (r3 M-2) — *"same key, different verdict" is
exactly what a future reviewer will be asked to explain*, so the list is part
of the design rather than folklore:

- **Lock-value classes hide push-size differences.** `older(100)` is one byte,
  `older(65535)` three, `after(1700000000)` five. Two policies sharing a key
  can sit on opposite sides of a script-size or op-count limit, and a
  coordinator that refuses on that limit would refuse one and accept the other.
- **Digest classes hide the digest.** Two policies with the same structure and
  different preimages share a key; no coordinator measured distinguishes them,
  but a future one checking a known-preimage list would.
- **The key is renderer-independent by construction**, which is why the
  renderer rides on the evidence tuple. A coordinator that parses text — all
  three measured do — can disagree across spellings of one key.
- **The key hides the ORDER of the leaf keys [recon §3c].** Measured with
  libnunchuk `a7cfb49` (Nunchuk 2.1.1's pin): two kind-1 cards with
  byte-identical SkeletonKeys, differing only in the order of their leaf
  pubkeys, are REFUSED and ACCEPTED respectively. Nunchuk re-derives Liana's
  key over its signers' pubkeys sorted and deduplicated
  (`src/descriptor.cpp:689-712`, `std::sort` + `std::unique` at `:700-701`)
  and refuses unless the re-render equals the input (`:646`); Liana hashes
  them in wire order with duplicates. **Ruled [R-1]:** the key stays
  coordinator-independent; the Nunchuk cell is a KEY-DEPENDENT, source-derived
  rule — refuse unless the leaf pubkeys are already strictly ascending — read
  from `Skeleton::leaf_keys_ascending`, a non-key field. A template has no
  keys, so it is `Unproven { KeysAbsent }` for Nunchuk. The positive for a
  sorted card is still measured-only (the recon's probe, committed as
  evidence by plan 1b).

Any of these turning into a real disagreement surfaces as a **build
failure**, not as a silent wrong verdict. That is the containment — but only
with a fifth class [recon §1c, R-1]: D1-D4 below are all RULE vs EVIDENCE, and
two evidence rows with one key and opposite outcomes had no class. **D5
evidence-conflict** is that class; a key-dependent rule (R-1) is how such a
pair is resolved, because the rule explains one row and the other is then the
only cell.

**THE KEY'S MEMBERSHIP, settled here because a plan may not choose it** (r4).
`SkeletonKey` — the thing hashed, printed by `md shape-key`, and used to key
the evidence table — is exactly:

    root + inner_wsh + template + fp_partition + key_partition + key_path_kind

and **nothing else**. Specifically:

- **`key_partition` IS in the key.** It distinguishes wallets Liana refuses on
  `DuplicateKey`, and a measurement taken on a policy where two slots share an
  xpub does not apply to one where they do not.
- **The internal-key kind IS in the key.** An earlier draft defined a
  serialization without it, which would have made a NUMS policy and an
  unspendable-xpub policy share a key — the distinction Nunchuk makes and F-449
  records (r4).
- **`keys_present` is NOT in the key.** It selects the *verdict* (§1(a2)), it
  does not make a template-only card a different policy from the same card
  seated.
- **A "path" for `tr` is a taptree LEAF, plus the key path as path 0 when the
  internal key is a real key slot (`InternalKey::Slot`) — NOT merely "not
  NUMS" [recon]: a Liana key (wire kind 1) is not NUMS and pushes NO path 0,
  because an unspendable key spends nothing (`policy_shape.rs:301`, gated on
  `InternalKey::Slot`; Liana v15.0's evidence agrees).** **CORRECTION (plan 1a, Task 4):** an earlier
  draft said `walkTapTree` "already defines this decomposition and the port
  inherits it". That is factually wrong — the Go appends one branch per **leaf
  only**, and the key-path branch does not exist there. It is an ADDITION the
  Rust port makes deliberately, implemented at `policy_shape.rs`'s `Tag::Tr`
  arm, gated on `InternalKey::Slot` (it was `!is_nums` before F-449 stage 1b
  [recon]), pushed before `walk_tap_tree` so it lands at index
  0. It matters because the next plan's rules read `shape.branches`, and a
  spendable internal key IS an unlocked spend path: a rule counting unlocked
  paths would undercount every such taproot.

**The key's serialized form** is THREE `U+001F`-separated fields [recon]: the
template; the partitions (`fp_partition` then `key_partition`, each a
self-delimiting bracket expression, rendered as `[path][group][slot]` with
slots ascending, groups ordered by their lowest slot, and paths in template
traversal order); and the key-path kind (`NotTaproot`, `Nums`, `Xpub`,
`LianaUnspendable`). `skeleton.rs:313-319` at `d269c556` is the one spelling;
an earlier draft of this paragraph named two fields and omitted the third.
That serialization is the thing hashed, the thing `md shape-key` prints and
the thing the evidence table is keyed by — one spelling, defined once.

**A template's key is NOT its seated card's key [plan 1b].** 1a's partitions
render EMPTY for a card with no key TLV (`[[][][]][]`, the module doc of
`skeleton.rs`), not as singletons, so the template-only card and the same card
seated have different keys — measured on X24. The key-membership clause below
("`keys_present` ... does not make a template-only card a different policy")
is therefore true of the KEY MEMBERSHIP and false of the rendered key. It is
harmless for verdicts, because (a2) already forbids a template any positive
whatever the key, and plan 1b pins that by a test that clears only
`keys_present` on a seated Skeleton (keeping its key) and asserts no positive.

Slot ids are **0-based** (they are `@i` placeholder indices, and `@0` is a real
placeholder); equality classes are **1-based** (they are a counter, and `#0`
would read as "no class"). The two bases differ on purpose and the key's
grammar says so, because an earlier draft left a reader to guess (r3 N-2).

Too-coarse becomes structurally impossible, every measured shape stays
matchable, and a foreign md1 with an unfamiliar fragment falls to `Unproven` —
silence, the correct verdict for a policy nobody measured.

**RETRACTED: "no Go walk needs porting for the normative path."** That
sentence was in the first fold and it is false — it was the one line a plan
would have been costed from. Measured (r2 C-1, and independently by the
controller at `b6e20412`):

- **Genuinely already owned:** `render::descriptor_to_template(&Descriptor)`
  (`crates/md-codec/src/render.rs:146` at `d269c556` [recon]) takes the **decoded** descriptor, emits
  `@i` placeholders and **erases origins**; placeholder renumbering is real
  (`canonicalize::canonicalize_placeholder_indices`, `crates/md-codec/src/canonicalize.rs:159` [recon]).
- **Not owned:** lock values and digests render **literally**
  (`crates/md-codec/src/render.rs:280` and `:299` at `d269c556`, and the two hash renderers at `:426`/`:466` [recon]; plan 1a shipped the abstracting mode beside them) — the key needs a
  rendering MODE that abstracts them. The **per-path** fingerprint partition
  needs a spend-path decomposition that exists **nowhere in Rust**:
  `compose::SpendPath` is the *input* model, `md decompose`'s `Occurrence`
  carries no path index, and the branch split lives only in the fork's Go
  `md/policy_shape.go`.

So **porting `policy_shape.go` to Rust is the first task of step 1**, not a
display concern deferred to the fork. Computing the partition from the
compose-side `PathList` instead is rejected: it would be a second
implementation of one key — the defect §2 exists to forbid — and a restored
card has no `PathList` at all.

**(a2) A template-only payload is `Unproven`, never `Imports` (r2 C-2).**
`md compose` is keyless *by construction* — measured this session, it prints
`note: stdout is a keyless descriptor template (no keys)` — so the key's
identity components are **always** unknown on the most-used command in plan 1.
The rule: when `keys_present` is false, every coordinator whose rule reads a
key-identity component yields `Unproven { reason: KeysAbsent }`. A refusal
that depends only on structure (wrapper, lock kind, hash presence) **still
stands** — it is sound without keys — so a template-only policy can be refused
but can never be claimed to import. The device's template-only consent is the
same state on steel.

**(a3) A card whose keys will not expand is `Unproven`, and the build says so**
(r4). `ExpandWalletPolicyChunks` can fail — a hardened wildcard, a hardened
multipath alternative, an exotic range — and on that card no partition can be
computed, so no key exists. The verdict for every coordinator is
`Unproven { reason: NoEvidence }`, the device prints no coordinator row at all
rather than an empty one, and `md` prints the expansion error it already has.
This is distinct from template-only (a2), where the structure is known and only
identity is absent.

**(b) The fingerprint partition is part of the key.** Liana refuses on a
key-IDENTITY relation (X11, `DuplicateOriginSamePath`) that a structural key
erases — and the device composes that shape today under a §8g *warning*. Two
policies identical in structure but differing in which slots share a seed are
different keys.

**(c) A fourth verdict: `ImportsAltered` — "accepts, but not as built".**
Measured twice: Liana re-reads X24 as a 2-of-4, Nunchuk shows an unsorted
`multi` as `MINISCRIPT 0-of-3` (F-626). Both are "yes" in a three-verdict
model and neither is the wallet the operator built. This is the verdict that
says so.

**(d) The renderer is on the evidence tuple, not the key.** Nunchuk's
acceptance is a **byte round-trip against `md descriptor`'s exact spelling**
(`descriptor.cpp:628-646`) — which is F-624 and F-627 landing directly on this
design. A positive is therefore a claim about `(shape, coordinator, version,
renderer)`; change the renderer's output by a byte and every Nunchuk positive
is retired until re-measured.

**(e) Spans are closed at both ends, and `Version` is an opaque ordered
label.** `until: None` prints an inference past the last verified version,
which ruling 3 forbids — so **"26 and newer" is not a sentence this device may
print; "26.0-31.1" is.** And a coordinator's "version" is not always a semver
triple: libnunchuk is a library under several front-ends, and Core's behaviour
can turn on `-deprecatedrpc` rather than a release. An opaque ordered label per
coordinator keeps the model honest about both.

## Section 2 — rules and the evidence pipeline (REVISED after review)

**A rule is a predicate over the skeleton written in the coordinator's own
order of refusal**, each clause citing the source that establishes it, and it
returns the FIRST class that applies — which is what makes the reason named the
one the coordinator would actually hit first. `composerLianaOutsideModelClass`
(fork `gui/composer_consent.go:396` at `2c9eed3`, was `:381` [recon]) is the
working template; it moves to Rust and the Go becomes the convergence port.

**Three traps in that move [recon §1d].**

- **Double count.** The Go adds one unlocked path for a spendable key path
  (`if shape.KeyPath == md.KeyPathSpendable { unlocked++ }`,
  `composer_consent.go:423`) because the Go walk has no key-path branch. The
  Rust walk already carries that branch as an unlocked `Branch` 0, so a
  verbatim port counts a spendable key path TWICE. The Rust rule counts
  branches only. Plan 3's cross-language vectors must allow for the Go and
  Rust `branches` lengths differing on every `tr` policy with a real key path.
- **Kind-1 rulings carry over.** Class 2 fires on NUMS only and the unlocked
  count adds a real key only, so a Liana key is neither (stage-4 F2,
  `TestComposerLianaClassRulings`).
- **A second host-side Liana classifier already ships:** md-cli's
  `liana_refuse_or_warn` (`crates/md-cli/src/cmd/compose.rs:638-666`), which
  warns on a hashlock and on every-path-timelocked under `--unspendable liana`
  and which F-644 asks to widen. Plan 1b REPLACES its two Liana warnings with
  the registry's verdict rather than adding a third copy; its SPEC §6 refusal
  and its "has no effect" warning are md's own rules and stay.

**Liana checks older-units before `after` [plan 1b].** The Go's classes 5
(`after`) and 6 (older in time units) are in the wrong order for a policy
carrying both: Liana v8.0 and v15.0 refuse `mixed-lock-bases-{wsh,tr}` with
*"Timelock value '4194404' isn't valid or safe to use"* — the units check —
and the Go names "an absolute lock". The Rust order is the measured one (the
table build reports three D3s with the Go order); the fork converges in plan 3.

**Class 9 is narrower in Rust than in the Go [plan 1b, plan 1b R0 I-1].**
The Go names every second unlocked path. Liana lifts the policy, and
rust-miniscript's `normalized()` flattens every 1-of-n into bare keys, which
Liana folds into the primary path: a second unlocked path that is a single key
(X24) or a 1-of-n (R0's measured `wsh(or_d(multi(2,A,B),or_i(multi(1,C,D),
and_v(v:pkh(E),older(100)))))`, imported by v15.0 as 2-of-4) is IMPORTED, read
as altered (`ImportsAltered`). A second unlocked path that stays a threshold,
`k >= 2` (X25, X26), is refused. The Rust clause refuses exactly `k >= 2`; a
plan draft keyed on "more than one key" and falsely refused the 1-of-n case.

**Liana's relative-lock class reads the lock VALUE [plan 1b R0 M-1].**
`csv_check` accepts only a u16 block count, so `older` in blocks above 65535 is
refused like one in time units (measured: `older(70000)`, *"Timelock value
'70000' isn't valid or safe to use"*). The key abstracts lock values, so this
must be a rule, not a cell: the clause declares `LOCK_VALUES` in its `ReadSet`,
as does "two paths with one lock".

**Measured refusals are admissible.** Ruling 3 said refusals are rule-derived;
that was written before Nunchuk's round-trip check was understood. A refusal a
real harness printed is at least as good as a rule at that version — it simply
does not extend to other versions.

**Rules and evidence meet at a table BUILD, never at runtime.** The generated
`verdicts` table is the single artifact both consumers read. A disagreement is
**a build failure**, resolved by a person and committed. Disagreements are not
a flaw in the split; X24 is proof that a disagreement can be the most valuable
row in the table. The flaw would be nobody being told.

**The classes are named** (r2 I-6), because "a named class" that names none is
not a contract:

| class | rule says | evidence says | resolution |
| --- | --- | --- | --- |
| **D1 false-refusal** | refuses | imported | narrow the rule, or re-read the source at that version |
| **D2 missed-refusal** | admits | refused | widen the rule; the measured refusal stands meanwhile |
| **D3 reason-drift** | refuses for X | refused for Y | re-attribute the class; the verdict is unaffected |
| **D4 orphan-evidence** | no rule spans it | any | add a `RuleSet` for that version, or drop the row |
| **D5 evidence-conflict** [recon, R-1] | — | two rows, one key/coordinator/version/form, opposite outcomes | a key-dependent rule that explains one row (R-1's shape), or drop a row |

**D3 is not hypothetical:** F-633 measured Liana v15 refusing the two key-less
shapes for *"All spend paths must require a signature"* while our classifier
names *"a hash lock"* — same verdict, different reason. Today nothing would
classify that; under D3 it is a build failure with an obvious resolution.

**D3 only compares messages that NAME a class [recon].** Liana's refusal text
for EVERY kind-1 refusal in evidence is one generic string, *"Descriptor is not
compatible with a Liana spending policy."* (live gate 4 of 4 plus the control;
probes 5 of 5), and CONTINUITY_f449_stage2 records it covers at least three
causes. A generic message cannot be attributed, so D3 is inapplicable to it:
the verdict is checked, the reason is not. Each coordinator declares the
messages that do name a class; everything else is generic.

**`Description` is DERIVED, never hand-authored** (r2 I-7). It is the
coordinator's own parsed reading of the policy, recorded by the harness —
which is the architect's clause the first fold dropped: *the harness must
record the coordinator's parsed policy*. So every harness emits, alongside its
verdict, the structure the coordinator inferred (Liana's primary/recovery
paths; Nunchuk's wallet type and `m`-of-`n`), and `ImportsAltered` fires
**automatically** wherever that reading differs from the policy as built. A
hand-marked field would fire only where a human remembered.

**The shape key has exactly one implementation.** The generator computes keys
by calling `md shape-key`, never by reimplementing the canonicaliser in Python.
Two implementations of one key drift, and the failure is silent in the worst
direction: positives quietly stop firing, or fire on the wrong shape.

**Gates:** vectors pin every `(shape, coordinator, version)` cell; a
conformance test asserts `chunks -> key == descriptor -> key` for every
evidence row; and the rule/evidence build fails on an unresolved disagreement.
**"Every evidence row" means the evidence, not the vector corpus [recon §1b]:**
plan 1a's gate ran over 48 vendored vectors, which contain 0 of the 11 Liana
live-gate rows and 1 of the 56 matrix shapes, and md-codec had no library
route from descriptor text to a key (the walker was test-private). Plan 1b
promotes that walker into md-codec and runs the gate over the vendored
evidence rows themselves.

**Evidence transport [R-2].** Evidence is recorded in mnemonic-engrave, where
the harnesses live, and VENDORED into descriptor-mnemonic by a script with a
freshness check — the `vendor-liana-evidence.sh` pattern. The table build
reads the vendored copy only, so a step-1 gate never depends on another
repository being checked out.

## Section 3 — staleness (FIRST-CLASS, at the operator's direction)

**The premise, and it is the thing to get right: truth about a version does not
decay; relevance does.** "Liana 8.0 imports this" is as true in 2030 as today,
because 8.0's binary is immutable. What changes is whether anyone still runs
8.0. So this design does **not** build a decay model — no "refuse to claim
anything older than N". It builds **provenance**, and lets the operator do the
matching.

That premise is not theoretical here. Measured from the Liana checkout's own
tags: our rules were read at **v8.0 (2024-11-08)**; Liana ships **v15.0
(2026-07-31)**. Seven majors, the file moved, its content changed five times,
and one change is semantic — at v15.0 the import path passes
`/* compile = */ false`, so the import-time `InvalidPolicy` refusal class no
longer exists. The device is shipping a present-tense claim about Liana today
from a 21-month-old reading (**F-633**). **Half-fixed [recon]:** §8x now says
"Liana (as of v15.0)" (`gui/composer_copy.go:316`), but every Core claim on the
device is still unversioned ("Bitcoin Core imports it"), and the Core boundary
now measured makes those claims false for Core 24.2-25.2.

Six mechanisms:

1. **Every verdict carries `(version verified, renderer, date)`, and the copy
   is always version-qualified.** The device never prints "Liana imports"; it
   prints "Liana 8.0 imports (2026-09); newer: unmeasured". The date is not
   decoration — a plate outlives releases, and a date is the one thing a future
   operator can judge against with no other context.
2. **No open-ended span, for either verdict kind.** "As of" is the only tense a
   positive may use.
3. **Committed, runnable harnesses**, each pinning the coordinator source
   revision it builds **and recording the binary's own self-reported version —
   never a label typed by the person running it** (r3 M-5). The motive is
   measured: `bitcoind --version` on this box prints
   `Bitcoin Satellite version v0.2.4` for what the matrix calls "Core 25.0".
   A hand-typed label is how a verdict comes to name a release nobody ran. **Liana's is now committed** (`harnesses/liana/`,
   `606ab180`) and was used to re-measure at v15.0 — and since F-449 stage 2
   it is re-run as a gate, `scripts/liana-live-gate.sh`, which pins the tag and
   commit and diffs against a committed expectation. **Core's is now committed
   as a probe [recon, core-boundary]:** `design/evidence/coord-compat-core-boundary/probe.py`
   run against official release binaries (and, earlier, the one-shot
   `design/evidence/f449-stage4/core-kofn-import.sh` on a dev build, which the
   release-binary run supersedes). Nunchuk's still lives only in
   `/scratch/code/shibboleth/.tmp/fable-nunchuk-lib/build/fableharness`. Re-measurement without a
   committed harness is a research project every time, and under ruling 6 that
   means the registry can grow rules but never evidence for those two:
   refusals forever, positives never.
4. **A hand-maintained `KNOWN_RELEASES` file** — each coordinator's newest
   known release, its date, the URL it was read from — and a build-time check
   that fails, or degrades the row to `Unproven` with a stated reason, when the
   newest known release is more than one major past every verified version.
   **This is the one gate that turns "we should re-measure" into a command that
   fails.** ~~It would fail today on Liana (8.0 verified, 15.0 known) and on
   Nunchuk.~~ **Not on Liana any more [recon]:** v15.0 is verified (289/289,
   and the live gate pins `v15.0`/`4684d5cb`), and `git ls-remote --tags`
   finds no newer release tag. **Nunchuk is unknown:** libnunchuk upstream
   HEAD `33f7dc3` is 204 commits past `a7cfb49` and `src/descriptor.cpp`
   changed (`ParseTrDescriptor` key-path handling, musig;
   `GetUnspendableXpub` and the round-trip check are untouched), but whether
   the APP moved past 2.1.1 was not measured.
5. **A renderer gate:** re-render every evidence descriptor with the current
   `md` and diff. Any byte change retires every Nunchuk positive until
   re-measured — automatically, in the build, not by someone remembering that
   Nunchuk round-trips text.
6. **Evidence is a snapshot with a commit**, and re-measurement produces a diff
   of verdicts — the artifact a reviewer needs.

**The version a verdict names is the one the APPLICATION displays** (r2 I-10),
not the library revision. An operator can read "Nunchuk 2.1.1" off their own
About screen; they cannot read `libnunchuk a7cfb498`. The library revision is
kept as **provenance on the evidence row**, so the chain from a printed
version to the bytes that produced it stays intact.

**Which plan owns each mechanism** (r2 I-11) — the first fold assigned none:

| mechanism | plan |
| --- | --- |
| 1 provenance in every verdict | 1 (md-codec) |
| 2 no open-ended span | 1 (md-codec) |
| 3 committed harnesses | 2 — **Liana's is done** (`harnesses/liana/`) |
| 4 `KNOWN_RELEASES` + freshness gate | 2 — no longer red for Liana (15.0 verified, 15.0 newest) [recon]; Nunchuk's app version unmeasured |
| 5 renderer gate | 2 |
| 6 evidence snapshot + verdict diff | 2 |

Mechanism 4 being red on the day it lands was intended: it is the gate that
turns "we should re-measure" into a command that fails, and F-633 is the
proof it would have fired. Whether it is red when it lands now turns on
Nunchuk alone [recon].

**What this design deliberately does NOT build:** a freshness threshold in days
evaluated on the device. The device cannot tell the time (§6b says so in its
own copy), and a verdict whose wording changes as a clock moves is a verdict
nobody can test.

## Section 4 — the two surfaces

### The SH2 consent screen

Bounded by **ordering**, not by count — the registry grows, the screen does
not.

1. **The verdict rows go on the first page**, directly after the script line
   and before the paths. The paths are what the operator consents to; the
   coordinator verdict is what changes whether they *should*. The current
   placement puts it behind a CONTINUE that is live on every page
   (`gui/multisig_build.go:1898`), which makes it skippable by construction.
2. **Worst first, so overflow can only hide good news.** Order: `Refuses`,
   `ImportsAltered`, `Unproven`, `Imports`. If the registry ever exceeds the
   first page, what slides to page two is a positive, whose loss costs a
   convenience. A refusal that slid would be the W-2 class — a fact no hand can
   reach.
3. **Collapse the positives, never the negatives.** Above K rows the `Imports`
   run becomes one line ("Imports: Liana 8.0, Core 26.0-31.1 (+2 more)").
   Refusals are never collapsed; each names its reason.

**One copy of the verdict, not two.** No summary line on the consent *plus* a
detail screen elsewhere — hand-written coordinator notices already ship, and
this design retires them rather than adding another.

**The inventory, recounted [recon §2, recounted mechanically by plan 1b].**
"Five (§8a, §8f, §8x and two more)" was already ambiguous at `7b6f2fb`, where
SEVEN `composerCopy*` bodies named a coordinator. At `2c9eed3`, scanning the
string literals (comments stripped) of every non-test function in `gui/*.go`
and `md/*.go` (137 files), **19 functions** mention a coordinator:

- **11 carry a coordinator VERDICT:** ten `composerCopy*` bodies —
  `KeylessPath` :94, `NUMS` :199, `LianaKeyPath` :220, `UnspendableRowNUMS`
  :241, `UnspendableRowLiana` :247, `MixedLockBases` :284,
  `OutsideLianaModel` :314, `SameSeedThreshold` :324, `SameSeedBelow` :332,
  `DuplicateKeys` :633 (all `gui/composer_copy.go`) — plus
  `composerUnspendableDropCause` (`gui/composer_unspendable.go:125`, literal
  at :141), which is not a `composerCopy*`.
- **4 name only the KEY KIND** "Liana key": `composerCopyLianaKeyDropped` :254,
  `composerCopyLianaUnmet` :264, `md1KeyPathLine` (`gui/md1_inspect.go:202`),
  `policySummaryLines` (`gui/template_engrave.go:151`).
- **4 use the generic word** "coordinator": `composerCopyNothingChecked` :460,
  `multisigBuildExperimentalWarningBody` (`gui/multisig_build.go:880`),
  `buildReviewLines` (`:1632`), `templateConsentLines`
  (`gui/template_engrave.go:63`).

**A gate asserts that no verdict body names a coordinator** (r3 M-3) — the
bodies become reason strings and the consent prints from the registry, but
without the gate another hand-written notice grows exactly as these did,
unnoticed. As worded ("no `composerCopy*` body") it would FIRE on the two
key-kind bodies and MISS `composerUnspendableDropCause` [recon]; plan 3's gate
needs a key-kind exemption and must scan beyond `composerCopy*`.

**Gates this repo already knows how to write:** assert the first-frame row
count against the registry size (the kind-picker lesson — a paginated screen
hides its overflow), and measure copy headroom by padding and bisecting rather
than by reasoning about it.

### `md` on the host

`md shape-key` emits the key. `md compose` and `md descriptor` both print the
verdict, naming the form.

**Only `md compose` refuses; `md descriptor` never does (r2 C-4).** The two
commands are not the same kind of act. `md compose` **mints** a policy — that
is the moment ruling 2's loud stop belongs, and it exits non-zero naming
**`--md-only`** (r2 I-8) as the flag that proceeds. `md descriptor` **reads**
an existing card, usually one already engraved: refusing there would deny an
operator the descriptor for a wallet they already hold, which is a regression
`crates/md-codec/src/validate.rs:444-453` (at `d269c556` [recon]) documents in its own words. It prints the same notice on
stderr and exits 0.

The flag is on `md compose` alone, because it is the only command that can
refuse.

The none case on the device is **its own confirm-to-proceed screen** before the
consent, in §8a's shape — bounded by construction because it has one body. It
lists the refusals with reasons and says what remains: *"Only md can rebuild
this wallet, and md cannot sign."*

## Section 5 — Rust-first sequencing

**The gate for every step in every plan is four commands, not three:**
`cargo test`, **`cargo doc --workspace --no-deps --document-private-items
--all-features`**, `cargo clippy --all-targets -- -D warnings`, `cargo fmt
--check`. Plan 1a shipped with `cargo doc` missing from its gate, and the
omission reached `main` — the CI workflow went red on it while both REQUIRED
contexts passed, so the push ritual reported "no bypass" and was right to.
Seven reviews inherited the blind spot because each was scoped to the plan's
own gate. A required-contexts check is not a green build.

Each step with its gate. Nothing starts in the fork.

1. **`md-codec`** — **first task: port `policy_shape.go`'s branch split to
   Rust** (r2 C-1); then the abstracting render mode, the `Skeleton` builder,
   the four `Verdict` kinds, and the generated `verdicts` table with vectors
   pinning every `(shape, coordinator, version)` cell. **The table generator
   is a step-1 `xtask`, not `md shape-key`** (r2 I-1) — step 1's gate needs
   the table, so it cannot be built by a step-2 binary. **No `xtask` exists
   yet [recon]:** plan 1a put the one key transcript in
   `crates/md-codec/examples/dump_skeleton_keys.rs`; plan 1b creates
   `crates/xtask` (`cargo xtask verdicts`); `md shape-key` in
   step 2 is a thin CLI over the same library function, and a test asserts
   the two agree. *Gate:* the conformance test (`chunks -> key ==
   descriptor -> key` for every evidence row) and the rule/evidence build.
2. **`md-cli`** — `md shape-key`; the verdict on `md compose` and
   `md descriptor`; `md compose`'s none-case refusal and `--md-only`.
   *Gate:* CLI vectors including the flag path, the template-only case
   (`Unproven { KeysAbsent }`), and `md descriptor` NOT refusing.
3. **Harnesses committed**, then run once to regenerate the evidence
   byte-identically from the current tree — the reproducibility proof.
4. **Fork** — the Go port of the canonicaliser with a provenance pin; the
   registry consumed from *generated* Go, not hand-ported rules; the consent
   rows; the none-case screen; retire the five hand-written notices. *Gate:*
   cross-language vectors, the same key and verdict from Rust and Go for every
   evidence row — because this repo has measured that 887/887 green per-repo
   tests can hide a Go/Rust disagreement.
5. **Emulator walk** for the consent at N and at N+K coordinators (a synthetic
   registry), asserting first-page rows and that a refusal is never off the
   first page.

## Scope — this is more than one implementation plan

Section 5's five steps are sequenced but they are not one plan. The natural
split, each with its own gate and its own review:

1. **md-codec + md-cli** (steps 1-2) — the canonicaliser, the verdict types,
   the generated table, `md shape-key`, the CLI verdict and the none-case
   refusal. Self-contained in the primary repo, and the only part ruling 4
   forces to come first.
2. **The harnesses** (step 3) — committing and re-running them. Independent of
   both other plans, blocks nothing in plan 1, and blocks *everything* about
   growth. It is also the plan that closes F-633, because re-measuring Liana at
   v15.0 needs a committed harness to be repeatable.
3. **The fork** (steps 4-5) — the Go port, the consent rows, the none-case
   screen, retiring the five notices, the emulator walk. Cannot start until
   plan 1 lands, per the Rust-primary rule.

Writing one plan for all three would produce a document whose later half is
written against a tree that does not exist yet — the staleness this cycle has
already measured in a plan's own citations.

## Settled since the third fold

- **The Core boundary release — MEASURED [core-boundary]: 26.0** (ruling 1
  above). Every release 24.2-25.2 refuses the representative `tr` +
  tapscript-miniscript descriptor with `Miniscript expressions can only be used
  in wsh`; every release 26.0-31.1 imports it with md's receive and change
  addresses. The "one capability" claim holds across releases: all 15
  discriminating shapes flip together, at 26.0, and the `wsh` control is
  accepted by all ten releases. F-449's kind-1 descriptor first imports at
  26.0, with receive/change 0..2 equal to Liana v15.0's, replacing the
  v30.99 dev-build-only evidence. The `<0;1>` spelling is a second boundary,
  between 28.4 and 29.4. Not measured: point releases between the ones
  listed, release candidates, spending, and the GUI.
- **The Nunchuk leaf-order question [recon §3c] — ruled [R-1]** (§1A's blind
  spots).
- **How evidence in mnemonic-engrave reaches a table built inside md-codec
  [recon §6] — ruled [R-2]** (§2's evidence transport).

## Open, and blocking a spec
- ~~Liana must be re-measured at v15.0~~ — **DONE** (`606ab180`): 289/289
  verdicts identical to v8.0, 73/73 accepted policies identical in inferred
  policy and addresses, 10 refusals changed message only. F-633 stays open on
  the unqualified present tense, which §3 fixes, not on a wrong verdict.
- **`K`, the collapse threshold**, is a measurement against the real frame, not
  a number to pick here. (A fork measurement; it does not block plan 1b.)
- **Nunchuk's own unspendable form (PR-1746) has no md1 encoding [recon §4].**
  Its real internal key is an origin-less xpub; `md decompose` reads it as a
  plain `Slot`, so it reaches the classifier as `KeyPathKind::Xpub` plus an
  unlocked branch 0 — a spendable-looking key path. Encoding it is a normative
  wire change (risk set (c)) that would spend the last usable wire version.
  A future wire question, not in plans 1b, 2 or 3; meanwhile no rule may emit
  a Nunchuk positive for an `Xpub` internal key on the assumption that it
  spends, and the evidence build marks such rows unkeyable.

## Measured context this design rests on

`design/IMPORTABILITY_composer_shapes.md` — 56 shapes (30 device-composed,
26 `md compose`), generated from `design/evidence/composer-fable-r0/`.
Totals: Nunchuk OK 20 of 30 measured; Liana OK 17 of 56; Core 25.0 OK 39 of 56;
Core 31.1 OK 54 of 56. (The "Core 25.0" binary was Bitcoin Satellite v0.2.4
[core-boundary]; `design/evidence/coord-compat-core-boundary/` is the Core
evidence plan 1b vendors.) Every preset is in the measured set, so for
preset-built policies — the common case — a measured positive can fire.
