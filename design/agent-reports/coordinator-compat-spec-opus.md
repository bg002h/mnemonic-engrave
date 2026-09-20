# Opus spec review — `DESIGN_coordinator_compatibility.md` at `4c6c5224`

**Brief:** `design/agent-briefs/coordinator-compat-spec-opus.md`. Opus,
2026-09-20. Repos read: `descriptor-mnemonic` at `b2c5d693`, `seedhammer` at
`7b6f2fb`, this repo at `5da35923`. `md` 0.17.0 (matches the repo's md-cli
version) was **run**; every quoted output below is from this session.

**Counts: 4 Critical / 10 Important / 5 Minor / 2 Nit.**

---

## Answers, first

**A. Is the design sound?** The architecture is sound and the fold is faithful
at the level of *verdicts*. All four Criticals have a corresponding revision
((a)-(e) in section 1), the architect's five opinion sections became sections
3-5, and the three-plan split is new, correct, and load-bearing.

The fold's failure mode is specific and repeats three times: **it folded the
architect's conclusion and dropped a named clause of his fix.**

| Architect's fix | Folded | Dropped clause |
| --- | --- | --- |
| C-1 | fp-partition per path | *"State what the verdict is when this component is unknown — a template-only consent, `md compose` without seated keys"* → my **C-2** |
| C-2 | the `ImportsAltered` variant | *"the harness must record more than accept/refuse … compare the coordinator's parsed policy against md's shape"* → my **I-7** |
| C-3 | renderer on the evidence tuple | the renderer reaches only `Imports`, while §2 admits measured refusals and §1(c) adds a fourth verdict → my **C-3** |
| N-2 | — | `Unproven` still carries no span, and §3.4/§4 now print things that need one → my **I-5** |
| I-3 | the five notices are retired | the gate that stops a sixth (*"no `composerCopy*` body names a coordinator"*) → **M-3** |
| I-6 | the hazard is named in (e) | the rule — *"the version scheme must be the one the application displays"* — is not → **I-10** |
| C-4 | harnesses committed | *"the coordinator binary's self-reported version, never a label typed by the person running it"* → **M-5** |
| M-1 | — | the key's blind-spot list → **M-2** |
| M-2 | — | the three-way key-path kind, which the rendered-template key makes *harder*, not free → **I-9** |

**And the fold introduced one new defect.** It promoted the architect's
*opinion* §1 into normative design text — including his aside that I-4's
PolicyShape port *"is still needed for the display summary, but that is a
display concern, not a normative one"*. That is false: the design's own
working rule template, `composerLianaOutsideModelClass`, is a **normative
refusal rule** and its signature is `(root md.ScriptKind, shape
md.PolicyShape)`. The fold deleted from the plan the exact walk its rules
consume, and then wrote *"no Go walk needs porting for the normative path"*
as a cost saving. See **C-1**.

**B. Is it implementable as written? NO.** An implementer who may invent
nothing cannot start section 5 step 1. The blocking gaps, in the order they
are hit:

1. `Skeleton` — the type the whole data model hangs off — is named exactly
   once, in `refuse: fn(&Skeleton) -> Option<Reason>`, and never defined
   (**C-1**). So are `RendererId`, `Description` and `CoordinatorId`: one
   occurrence each, all inside the struct that uses them.
2. The key the design illustrates **cannot be computed in Rust today**. Its
   fp-partition is *per path*, and there is no spend-path decomposition
   anywhere on the Rust decode side (**C-1**).
3. Half the payloads the verdict must cover — every `md compose` product —
   have no key identity at all, and the design never says what the verdict is
   (**C-2**).
4. The step that builds the table needs the CLI that the *next* step builds
   (**I-1**).

Everything after that is decidable but under-determined: ten places where two
readings give materially different code, listed below.

---

## Critical

### C-1 — the key the design illustrates cannot be computed in md-codec, and `Skeleton` is undefined

*At section 5 step 1, an implementer must decide what type `Skeleton` is and
where the per-path decomposition comes from, and the design does not say.*

The design's central cost claim is section 1(a): *"the canonicaliser md-codec
already owns **is** the key, so `md shape-key` is a formatter over it and no
Go walk needs porting for the normative path."* Measured against the source,
that claim is **half true, and the false half is the normative half**.

**What is genuinely already owned.** `md_codec::render::descriptor_to_template(&Descriptor)
-> Result<String, RenderError>` (`crates/md-codec/src/render.rs:52`) exists,
takes the **decoded** `Descriptor` (so I-4's "never `PathList`" is satisfied
for this half), erases origins, and emits `@i` placeholders. Placeholder
renumbering is real: `canonicalize::canonicalize_placeholder_indices`
(`crates/md-codec/src/canonicalize.rs:168`).

**What is not.** The design's key has a second component — *"fp-partition per
path: `[{0},{1},{2}] [{3}] [{4}]`"*. **"Per path" requires a spend-path
decomposition that exists nowhere in Rust:**

- `compose::SpendPath` (`crates/md-codec/src/compose/mod.rs:274`) is the
  **input** model — the thing you build a card *from*. Grep for any
  decode-side equivalent returns nothing.
- `md decompose`'s `Occurrence` (`crates/md-cli/src/decompose/walk.rs:55`)
  carries `display`, `record`, `origin_path_text`, `use_site`, `xpub`,
  `origin` — **no path or branch index**.
- The branch split lives only in the fork's Go `md/policy_shape.go` (438
  lines), exactly as the architect's I-4 said.

Measured, and this is the sharpest form of the problem: `md compose --json`
**does** emit a per-slot path index —

    "slots": [ {"index":0,"ordinal":0,"path":0}, … {"index":3,"ordinal":0,"path":1}, … ]

— but it is derived from the `PathList` the operator typed, not from a decoded
md1. So an implementer has exactly two readings:

- **(a) compute the partition from the compose-side `PathList`.** Then the key
  is computed by one code path at mint and a different one from a restored
  card — which is precisely the *"two implementations of one key drift, and
  the failure is silent in the worst direction"* defect section 2 exists to
  forbid. And restored payloads have no `PathList` at all.
- **(b) port `policy_shape.go`'s branch split to Rust.** Correct — and it is
  the work the fold deleted from the plan while calling ruling 4 *cheaper*.

**The rules make this worse, not better.** `refuse: fn(&Skeleton) ->
Option<Reason>`, and the design names `composerLianaOutsideModelClass` (fork
`gui/composer_consent.go:381`) as *"the working template; it moves to Rust"*.
Its real signature is:

    func composerLianaOutsideModelClass(root md.ScriptKind, shape md.PolicyShape) string

and its body reads `shape.KeyPath` (`KeyPathNUMS` / `KeyPathSpendable`),
`shape.Branches`, `b.Locks` with `l.Kind` ∈ {`LockAfterHeight`,
`LockAfterTime`, `LockOlderUnits`, `LockOlderBlocks`} and `l.Value`, and
`b.Hashlocks`. **None of that is recoverable from a rendered template string
without re-parsing it into a semantic policy** — which is the same walk again.
The fork's own comment says why `root` is passed separately: *"PolicyShape
does not carry it: policyShape walks tagWsh and tagSh identically."*

So `Skeleton` is either (i) a `String` — and then nine of the nine Liana
classes must be re-derived by parsing that string, a second semantic walk; or
(ii) a struct carrying the template *plus* a PolicyShape-equivalent — and then
the port the fold deleted is step 1's first task. These are different crates.

**Fix.** Define `Skeleton` as a struct in the design: the canonical template
string, the per-path fingerprint partition, the whole-policy (xpub,
derivation) partition, and the semantic decomposition the rules read. Put the
`policy_shape.go` port back at the head of step 1, and retract the "no Go walk
needs porting" sentence — it is the one line of the fold that a plan would be
costed from.

### C-2 — the template-only verdict is undefined, and template-only is the *default* product of `md compose`

*At section 5 step 2, an implementer must decide what `md compose` prints when
there are no keys to partition, and the design does not say.*

The architect's C-1 fix ended: *"State what the verdict is when this component
is unknown — a template-only consent, `md compose` without seated keys …
The honest output for that state is a conditional … or `Unproven`; it is not
`Imports`."* The fold folded the partition and dropped that sentence. Section
5 step 2's gate says *"CLI vectors including the flag path and the
template-only case"* — the design knows the case exists and still never says
what it produces.

This is not a corner. **`md compose` is keyless by construction.** Run this
session:

    $ md compose --wrapper wsh --path 2of3 --path 1of1 --path 1of1,older=26280
    wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,…),or_i(pkh(@3/…),and_v(v:pkh(@4/…),older(26280)))))
    note: stdout is a keyless descriptor template (no keys)

`crates/md-cli/src/cmd/compose.rs:5` states the contract: *"Not to be confused
with `crate::seat::compose`, which SEATS keys into an existing keyless card;
this module builds the card's policy from a path list."* And md-codec has a
first-class mode for it: `Descriptor::is_wallet_policy()` is false when
`tlv.pubkeys` is absent or empty (`encode.rs:51`).

So section 4's *"`md compose` … print the verdict"* asks for a verdict on a
payload whose key-identity component is **always** unknown. Three readings —
`Unproven`, a conditional string, or "compute the structural half and let
`Imports` fire" — and the third puts a false `Imports` on the most-used
command in plan 1. The device's template-only consent (§8a's key-less paths)
is the same state on steel.

### C-3 — `renderer` reaches only `Imports`, so §3.5's retirement guarantee cannot fire on the two verdicts that need it most

*At section 5 step 1, an implementer must decide where the renderer lives on a
measured refusal and on `ImportsAltered`, and the design does not say.*

Section 1(d) is right that a positive is a claim about `(shape, coordinator,
version, renderer)`, and §3.5 states the guarantee: *"Any byte change retires
every Nunchuk positive until re-measured — automatically, in the build."* But
the `Verdict` enum gives `renderer: RendererId` to **`Imports` alone**:

    Refuses { reason: Reason, span: Span },
    ImportsAltered { as_read: Description, span: Span },
    Imports { span: Span, renderer: RendererId },

Two things the fold itself added are renderer-dependent and have nowhere to
put it:

- **Measured refusals**, admitted by section 2 (*"A refusal a real harness
  printed is at least as good as a rule at that version"*). Nunchuk's are
  *entirely* renderer-dependent — F-624 measured the `--chain 1` spelling
  refused 30/30 while the multipath and `--chain 0` spellings imported 20/20
  each, same shape. A `Refuses` row with no renderer prints **"Nunchuk refuses
  this"** for a wallet Nunchuk imports in the form `md descriptor` emits by
  default. That is a wrong statement on steel, in the direction that costs the
  operator a working import.
- **`ImportsAltered`**, whose two measured instances include F-626 — a
  *Nunchuk* row, on the byte-round-tripping coordinator. After a renderer
  change the build cannot retire it, so the device keeps asserting "Nunchuk
  1.9 imports this, as MINISCRIPT 0-of-3" for a spelling nobody measured.

Either the renderer belongs on every measured verdict, or §3.5's guarantee is
unmet as written. An implementer cannot pick.

### C-4 — `md descriptor` is a READ path over an existing card, and the design does not say whether the none-case refusal applies to it

*At section 5 step 2, an implementer must decide whether the none-case refusal
fires on `md descriptor`, and the design does not say.*

Section 4 runs the two together: *"`md compose` and `md descriptor` print the
verdict, naming the form. The **none** case refuses with exit non-zero and
names the flag that proceeds."* Step 2 repeats the pairing: *"the verdict on
`md compose` and `md descriptor`; the none-case refusal and its flag."* The
subject of "refuses" is `md`, unqualified.

But the two commands are opposite contracts. `md compose` **mints** a policy —
refusing a new card nobody has engraved is ruling 5 working as intended.
`md descriptor` **reads** one: it decodes an md1 and prints the concrete
descriptor (`crates/md-cli/src/cmd/descriptor.rs:245`, `println!("{rendered}")`).
Under the reading where the refusal applies there, an operator who already
engraved a none-case policy finds that the tool which is supposed to restore
it exits non-zero and demands a flag.

This repo has already paid for that exact mistake and wrote the lesson into
the source. `crates/md-codec/src/validate.rs:449-458`, on
`OriginKeyContradiction`:

> MEASURED: `mnemonic bundle` emits `[00000000/m]` for a WIF slot … And
> because `chunk::reassemble` recomputes the encoding id via `encode_payload`,
> that refusal reached DECODE — an already-engraved card of that shape stopped
> being READABLE, which is a far worse outcome than the advisory this check
> exists to give.

That is why `Admission::SkipPolicy` exists (`encode.rs:105-112`, *"THIS EXISTS
BECAUSE THE ENCODE-SIDE REFUSALS WERE REACHING DECODE"*). The design must say
which side of that line the none-case refusal sits on. The safe reading is
"refuse on the mint path, warn loudly on the read path" — but it is a reading,
not a statement, and the wrong one is a restore regression.

---

## Important

### I-1 — step 1's artifact is generated by step 2's binary

*At section 5 step 1, an implementer must decide how the `verdicts` table is
generated before `md shape-key` exists, and the design does not say.*

Section 2: *"The generator computes keys by calling `md shape-key`, never by
reimplementing the canonicaliser in Python."* Section 5 step 1 is md-codec and
includes *"the generated `verdicts` table"*; `md shape-key` is step 2. Step
1's gate (*"the conformance test … for every evidence row and the
rule/evidence build"*) needs the table. The order as written cannot run.

Compounding it: the design never says **where the table lives or how it is
built** — a `build.rs` codegen (which would need the CLI at build time, making
the circularity structural), or a committed generated `.rs` regenerated by a
script with a CI freshness check. Materially different repos.

### I-2 — the illustrated key drops `/<0;1>/*`, which both the renderer and the committed evidence keep

*At section 5 step 1, an implementer must decide whether the use-site path is
in the key, and the design shows one answer while the machinery shows another.*

Measured this session, same policy as the design's own illustration:

    $ md compose --wrapper wsh --path 2of3 --path 1of1 --path 1of1,older=26280 --json
    "template": "wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),
                 or_i(pkh(@3/<0;1>/*),and_v(v:pkh(@4/<0;1>/*),older(26280)))))"

The design's key:

    wsh(or_d(multi(2,@0,@1,@2),or_i(pkh(@3),and_v(v:pkh(@4),older(blocks#1)))))

The committed evidence agrees with the machinery, not the design:
`design/evidence/composer-fable-r0/fable-liana-shapes.json` carries
`"template": "tr(@0/<0;1>/*)"`.

This is not cosmetic. The use-site path is **wire data with a large space**:
`UseSitePath { multipath: Option<Vec<Alternative>>, wildcard_hardened: bool }`
with `MAX_ALT_COUNT = 9` and arbitrary values and hardened flags
(`use_site_path.rs:58-68`), plus per-`@N` overrides in the TLV. Erasing it
gives a `<2;3>/*` or `/*h` payload the key — and therefore the measured
positive — of the `<0;1>/*` payload, against coordinators that parse text.
Keeping it makes the design's illustration wrong. Say which.

(The `--chain 0/1` *form* is correctly a renderer property, not a key
property: `descriptor.rs:199-201` collapses at render time from the same
decoded card. That half of (d) is right.)

### I-3 — `kind#class` is never defined; four independent sub-decisions

*At section 5 step 1, an implementer must decide the kind vocabulary, the class
counter's scope, its base, and the traversal that assigns it — and the design
defines none of them.*

`older(blocks#1)`, `after(time#2)` are the only two examples in the design.

1. **Kind vocabulary.** Two values from the fragment name (`older`/`after`,
   redundant with the text) or four semantic ones? The fork's `l.Kind` has
   four — `LockOlderBlocks`, `LockOlderUnits`, `LockAfterHeight`,
   `LockAfterTime` — and the blocks-vs-units and height-vs-time splits are
   **BIP-68/BIP-65 flag bits inside the value**, which md-codec renders
   verbatim (`render.rs:159`, `write!(out, "older({v})")`). The four-value
   reading is the one Liana's classes 5/6 need; the design shows `blocks` and
   `time` and never enumerates the other two.
2. **Counter scope.** Per-kind or shared? The two examples are `#1` and `#2`,
   which implies **shared** — but they are two fragments in two different
   sentences, so it is an inference, and per-kind would give `#1`/`#1`.
3. **Base.** `#1` is 1-based; the fp-partition in the same two-line example is
   `{0},{1},{2}`, 0-based.
4. **Order.** First appearance in which traversal — render order, wire order,
   or sorted by value? Different orders give different keys for the same
   policy, and the evidence table and the device would have to agree by luck.

A divergence here surfaces only in step 4's cross-language vectors, i.e. two
plans late.

### I-4 — the fp-partition's treatment of an absent fingerprint is undefined, and md-codec has already ruled on the same question for itself

*At section 5 step 1, an implementer must decide how slots with no fingerprint
partition, and the design does not say — while the adjacent md-codec check
says the opposite of the obvious reading.*

`TlvSection.fingerprints` is `Option<Vec<(u8, [u8;4])>>` (`tlv.rs:28`) —
**optional and sparse**, so a payload can carry fingerprints for some slots
and not others. And `[0,0,0,0]` is not a value, it is a sentinel.
`validate.rs:421-459` states the rule and why:

> `[0u8; 4]` is the ABSENT sentinel, not a master. It is what a producer
> writes when there IS no master to name … so two slots carrying it are two
> ABSENCES.

The naive reading — partition by fingerprint bytes — groups every zero-
fingerprint slot into one class. Measured consequence of the same mistake next
door: *"`mnemonic bundle` emits `[00000000/m]` for a WIF slot, since a WIF has
no master and no path, so a legal 2-of-2 of two DISTINCT WIFs was refused."*
Under the design's key, that same grouping makes Liana's
`DuplicateOriginSamePath` rule fire on a policy Liana accepts — a false
`Refuses`, and a key that no longer matches the measured row.

Two further sub-decisions in the same place: (a) the architect's C-1 also
asked for a **whole-policy partition by (xpub, derivation)** for Liana's
`DuplicateKey`; the fold kept only the per-path fingerprint one. (b)
`expand_per_at_n` — the function that yields `(idx, fingerprint, origin_path,
xpub)` — **can fail**, and `validate.rs:439` deliberately swallows that
(*"If the keys cannot be expanded there is no contradiction to prove"*).
`md shape-key` cannot swallow it: it must return something. What?

### I-5 — `Unproven` carries no span and no reason, and three places in the fold now print both

*At section 5 step 1, an implementer must decide `Unproven`'s payload, and the
design's own copy requires more than the enum provides.*

`Unproven,` is a unit variant. But §3.1's copy is *"Liana 8.0 imports
(2026-09); newer: unmeasured"* — the "newer" needs a span; §3.4 degrades a row
*"to `Unproven` with a stated reason"* — needs a reason; §4.2 orders
`Unproven` as its own row class — needs something to print. This was the
architect's N-2 (a Nit when `Unproven` was a corner); the fold's sections 3
and 4 made it load-bearing and it is now an Important.

Same shape, smaller: `Span` is used by three variants and never defined, and
§1(e) makes `Version` *"an opaque ordered label per coordinator"* — so the
ordering `Span` needs has no stated source.

### I-6 — the "named class" of a rule/evidence disagreement is not named

*At section 5 step 1, an implementer must enumerate the build-failure classes,
and the design names none of them.*

Section 2: *"A disagreement between a rule and a measurement is **a build
failure with a named class**."* The architect's I-7 supplied the table — six
rows, three outcomes: `Refuses(r)`/OK → build **failure**; `—`/REFUSE →
**allowed**, a measured refusal; `Refuses(r)`/`REFUSE(r')` with `r' ≠ r` →
build **warning**. The fold kept the sentence and dropped the table, and also
dropped its last mechanism — *"the coordinator's error string is mapped to a
reason class and compared"*, which is the only thing that makes the third row
checkable.

Note the third row is not hypothetical: F-633 measured that Liana v15 names a
*different reason* than our classifier for the two key-less shapes ("a hash
lock" vs the missing signature), same verdict. That is exactly a row-3
warning, and today nothing would classify it.

### I-7 — `ImportsAltered { as_read: Description }` has no type, no producer, and no derivation rule

*At section 5 step 1, an implementer must decide what `Description` is and how
a row becomes `Imports` rather than `ImportsAltered`, and the design does not
say.*

`Description` appears once in the whole document. Three questions, none
answered:

1. **What is it?** A free string, a structured comparison, or a per-coordinator
   type? The as-read material exists in the evidence but in **incomparable
   formats**: Liana's accepted rows are structured JSON (`"primary":{"keys":
   [...],"kind":"single"}, "recovery":[{"older":26280,…}]`), Nunchuk's are a
   text dump (`wallet_type=SINGLE_SIG address_type=TAPROOT template=DEFAULT
   m=1 n=1`). One type cannot hold both without a per-coordinator adapter the
   design never mentions.
2. **Who writes it?** The brief's worry is right — a hand-authored field in a
   generated table is a maintenance trap, and there is nothing in the design
   that makes it derived.
3. **How is the discrimination made?** Deciding X24 is `ImportsAltered` means
   comparing Liana's `primary 2-of-4` against *md's own* path decomposition —
   C-1's missing walk, a third time. The architect's C-2(b) asked for exactly
   this and it is the clause the fold dropped.

Without (3), an implementer's only option is to hand-mark rows, and then
`ImportsAltered` fires only where a human remembered.

### I-8 — the none-case flag is never named

*At section 5 step 2, an implementer must invent the flag's name, and the
design does not name it.*

Probe answered directly: all four mentions say "the flag" or "an explicit
flag" (design lines 39, 238, 255-256); ruling 5 says *"names an explicit flag
to proceed"* and the design never does. md-cli's existing override family is
`--experimental` (`cmd/decode.rs`), `--force-chunked` (`cmd/gui_schema.rs`),
`--force-long-code` (`cmd/encode.rs`) — a convention, but three plausible
names for this one, and a CLI surface is not changeable after it ships.

Related, same step: the design does not say whether the flag is shared between
`md compose` and `md descriptor` (see C-4) or per-command.

### I-9 — the rendered-template key cannot carry the internal-key KIND, and F-449 is already filed

*At section 5 step 1, an implementer must decide how the key distinguishes an
unspendable-xpub key path from a spendable one, and the design does not say.*

The architect's M-2 asked for `{Spendable, NumsRaw, NumsXpub}` *"now so the
committed keys survive the wire change"*, and reasoned it *"falls out of the
wire for free"* under a skeleton key. It does not. Today `is_nums` renders as
the literal H-point hex (`render.rs:87-89`, `nums.rs:12`), so NumsRaw is
visible in the text. F-449 — an open, filed cycle — adds *"a second
internal-key KIND on the md1 wire"* (`design/FOLLOWUPS.md:15684`), the
unspendable xpub. That form renders as `@i/<0;1>/*`: **byte-identical to a
spendable key path**.

The verdict turns on exactly that distinction. The device's own §8f says so:
*"Nunchuk cannot import a NUMS policy at all … Liana and BIP-388 signers need
an unspendable xpub instead (see F-449), which is a different wallet with
different addresses."* A rendered-template key collides a Nunchuk-refused
shape with a Nunchuk-accepted one the day F-449 lands, and every committed key
for those shapes silently becomes wrong. A text key cannot carry a marker that
is not in the text — so this needs a structured component, decided now.

### I-10 — the version a verdict names is still not the one the operator can see

*At section 5 step 1, an implementer must decide `CoordinatorId`'s version
scheme, and the design names the hazard without stating the rule.*

§1(e) says *"libnunchuk is a library under several front-ends"* and concludes
only that `Version` is an opaque ordered label. The architect's I-6 asked for
a **rule**: *"`CoordinatorId`'s version scheme must be the one the application
displays, with the library revision kept as provenance in the evidence row."*
Naming a risk is not managing it: the matrix's column is "libnunchuk 2.1.1"
and the desktop app on this box is `nunchuk-desktop-1.9.54`, so a consent line
reading "Nunchuk 2.1.1 imports" names a number no operator can match — and
§3.1 makes version-qualification mandatory in the copy, which is what turns an
unmatched number from a footnote into the sentence.

### I-11 — section 3's six mechanisms are assigned to no plan, and mechanism 4 is red today

*At the plan split, an implementer must decide which plan owns each staleness
mechanism, and the design does not say.*

Section 3 lists six mechanisms; the "Scope" section splits section 5's five
steps into three plans and never maps the six onto them. Mechanism 3
(harnesses) is plainly plan 2. Mechanism 5 (the renderer gate) needs md-cli
(plan 1) *and* the evidence (plan 2). Mechanism 4 (`KNOWN_RELEASES`) is
unassigned — and the design says of it: *"It would fail today on Liana (8.0
verified, 15.0 known) and on Nunchuk."*

So if mechanism 4 is inside plan 1's *"rule/evidence build"* gate, **plan 1
cannot close green on day one**. If it is not, plan 1 ships a table that plan 2
immediately reds. The design asserts plan 2 *"blocks nothing in plan 1"*; that
is true only under the second reading, and the design does not say which.

---

## Minor

**M-1 — the one code citation under the central claim points at a doc
comment.** Section 1(a) cites `encode.rs:59-62` as the canonicaliser. Those
lines are prose inside `canonical_payload_bytes`'s doc comment (*"This
delegates to `encode_payload`, which canonicalizes BIP 388 placeholder
ordering internally"*). The code is
`canonicalize.rs:168 canonicalize_placeholder_indices`. Under this repo's own
rule — never describe code from its doc comment — the design's load-bearing
citation is the doc comment.

**M-2 — the key's blind spots are not listed.** The architect's M-1
(lock-value classes hide push-size differences near script limits: `older(100)`
one byte, `older(65535)` three, `after(1700000000)` five) was filed with the
explicit reason *"the key's stated blind spots should list it, because 'same
key, different verdict' is exactly what a future reviewer will be asked to
explain."* The design lists none.

**M-3 — the gate that stops a sixth hand-written notice is missing.** I-3's
fix had three parts; the fold took two (bodies become reason strings, consent
prints from the registry) and dropped *"a gate asserts that no `composerCopy*`
body names a coordinator"*. Five copies grew unnoticed once.

**M-4 — digests get a kind but no equality class, and locks get both.** The
design writes `sha256(#)` with no class number while locks carry `#n`. Two
hashlock branches committing to the *same* digest versus different digests is
the same equality question that justified lock classes, and the asymmetry is
unexplained. (The device composes hashlock paths.)

**M-5 — "self-reported version" was dropped.** C-4's fix required every
evidence row to record the binary's self-reported version *"never a label typed
by the person running it"*, with the measured motive that the matrix's "Core
25.0" came from a binary reporting `Bitcoin Satellite version v0.2.4`. Section
3 mechanism 3 asks harnesses to pin the *source revision they build* — a
different fact, and it does not close this.

---

## Nit

**N-1** — the status header (*"Sections 1-2 approved, then REVISED"*) leaves
"approved" attached to text that has since changed substantially; a reader
reaching for what was approved cannot tell which sentences those are.

**N-2** — within the same two-line illustrated key, lock classes are 1-based
(`blocks#1`) and slot ids are 0-based (`{0},{1},{2}`).

---

## What I checked and did not find wrong

- `descriptor_to_template` takes the **decoded** `Descriptor` and erases
  origins — I-4's "never `PathList`" and "origins erased" are satisfied for the
  structural half of the key, and `md shape-key` really can be a thin
  formatter over it *for that half*.
- The `--chain` form is genuinely a render-time choice from one decoded card
  (`descriptor.rs:199-201`), so putting the renderer on the evidence tuple
  rather than the key (§1(d)) is right.
- Ordered paths (I-5) do fall out of a rendered template for free.
- Section 2's "measured refusals are admissible" correctly amends ruling 3.
- The three-plan split is sound and the stated reason for it — a plan's later
  half written against a tree that does not exist — is this repo's own
  measured staleness lesson, correctly applied.
- The decision *not* to build a device-side freshness clock is right and
  correctly grounded in §6b.

## Blind spots of this review

I did not run any coordinator harness; every coordinator fact is from the
committed evidence, the fork source, and the architect's report. I did not
build `md-codec` or write a prototype canonicaliser — C-1's "cannot be
computed" is from reading every decode-side type in md-codec and md-cli and
from `md compose --json`'s output, not from a failed compile. I did not review
plans 2 or 3, per the brief. I did not re-derive the 56-shape matrix totals,
the Core boundary, or F-633, which the brief lists as settled.
