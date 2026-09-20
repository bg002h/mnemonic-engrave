# Architect's opinion — coordinator compatibility on the SH2

**Artifact:** `design/DESIGN_coordinator_compatibility.md` at `d92f90ad`
(sections 1-2 approved; 3-5 unwritten). **Brief:**
`design/agent-briefs/coordinator-compat-fable-architect.md`. Fable, dispatched
at the operator's explicit request. 2026-09-20.

Everything cited below was read or run this session: the fork at `7b6f2fb`,
descriptor-mnemonic at `b2c5d693`, the Liana checkout under
`/scratch/code/shibboleth/.tmp/fable-liana-src` (tags v8.0..v15.0), the
libnunchuk checkout under `.tmp/fable-nunchuk-lib` (`a7cfb49`, 2025-12-05),
Bitcoin Core under `/scratch/code/bitcoin`, and the seven committed evidence
files under `design/evidence/composer-fable-r0/`.

**Counts: 4 Critical / 7 Important / 4 Minor / 2 Nit.**

---

## Verdict, first

**The family of architecture is right. The instance is not yet safe.**

A registry of (coordinator × version span), refusals derived from the
coordinator's own source, positives only from a measured table, both joined by
a coordinator-independent canonical key — that is the correct shape for this
problem, and I would not replace it. It is also the shape the fork already
proved on one coordinator (`composerLianaOutsideModelClass`), so it is not
speculative.

What is wrong is that the design as written produces a false `Imports` on
steel by four independent routes, and each one is measured, not hypothetical:

1. the key cannot see **key-identity relations** that Liana refuses on (X11);
2. the verdict vocabulary cannot say **"accepts, but not as built"**, which two
   coordinators measurably do (X24 in Liana, F-626 in Nunchuk);
3. Nunchuk's acceptance is a **string round-trip against md's spelling**, so a
   shape-keyed table asserts positives for spellings nobody measured; and
4. the evidence **cannot be regenerated** — the harness programs live outside
   every repo — so the table the design calls "regenerable and diffable" is a
   snapshot frozen at Liana 8.0, on a day when Liana is at 15.0.

None of the six operator rulings is architecturally unworkable. Two of them
carry a consequence the design has not yet drawn (see "Rulings" at the end).

---

## Findings

### C-1 (Critical) — the shape key equates policies Liana distinguishes by key identity

`X11-wsh-samefp-in-path` (`--path 2of3 --path 1of1,older=26280`, two of the
2-of-3 keys sharing master fingerprint `73c5da0a`) is **refused** by Liana 8.0:

    Key '[73c5da0a/48'/0'/1'/2']xpub…/<0;1>/*' is derived from the same origin
    as another key present in the same spending path.

(`fable-liana-parse-out.jsonl`, variants `md`, `real-xpub`, `h-spelling`,
`regtest-tpub` all agree; source `analysis.rs:500-515`,
`LianaPolicyError::DuplicateOriginSamePath`.) Under the design's proposed key
content — wrapper, key-path kind, per-path k/n/sorted, lock kind, hash kind,
path count, lock-value equality classes — X11 has the **same key** as the
Liana-accepted `[2of3, 1of1 older]` family (`preset-kofn-recovery-wsh`, X16).
The evidence table therefore holds an `Imports{Liana}` hit for X11's key.

The device composes this shape: §8g ("SAME SEED, SAME PATH … Liana will refuse
it") is a *warning*, not a refusal, and §7d refuses only two slots resolving
to the *same xpub*. So an operator who seats one seed twice in a path, reads
§8g, continues, and reaches the consent would be told by the new registry
that Liana imports — with the old §8g line three screens earlier saying the
opposite. A false positive, on steel, from a shape the device offers.

"Never key material" is right. "Never key-identity *relations*" is wrong, and
the design has already accepted the distinction for locks: lock-value
equality classes are in the key precisely because Liana's "two paths with one
lock" turns on two values being equal, not on what they are. Fingerprint
equality within a path is the same idea, one level up.

**Fix.** Add to the key: (a) a per-path partition of key slots by master
fingerprint (an equality structure, e.g. `{0,1},{2}`, never the fingerprints
themselves); (b) a whole-policy partition by (xpub, derivation) for Liana's
`DuplicateKey`. State what the verdict is when this component is *unknown* —
a template-only consent, `md compose` without seated keys — because Liana's
verdict then depends on a fact not yet in hand. The honest output for that
state is a conditional ("imports unless one signer holds two keys in a
path"), or `Unproven`; it is not `Imports`.

### C-2 (Critical) — the verdict vocabulary cannot express "accepts, but not as built", and the design leaves rule-vs-evidence precedence undefined

Two measured instances, two coordinators:

- **X24** `[2-of-3, 1 key, 1 key + older(100)]`: Liana 8.0 **ACCEPTs** and
  re-reads it as `primary 2-of-4; rec older(100) 1 key`
  (`composer-fable-r0-liana-core.md:266`); its GUI can never spend the single
  unlocked key the device promised (lens 5 I-2). The evidence row says OK. The
  Go classifier's class 9 says "a second unlocked path". The design says
  `Imports` requires an evidence hit and rules produce only `Refuses` or
  `Unproven`; it does not say which wins when they disagree. If evidence wins,
  the device prints "Liana imports" — true, and exactly the sentence that
  puts the wrong wallet in Liana. If the rule wins, the conformance gate
  ("vectors pin every rule's verdict per shape") will flag the rule as
  contradicting the harness and someone will "fix" the rule.
- **F-626**: `wsh(multi(2,K,K,K))` imports into Nunchuk and derives matching
  addresses, but Nunchuk shows `MINISCRIPT 0-of-3` with one signing path
  rather than `MULTI_SIG 2-of-3`. OK in the matrix; not the wallet the
  operator built.

Both are the "success is not round-trip" class this repo has already named
for its own codec. Three verdicts cannot hold it.

**Fix.** (a) A fourth verdict, `ImportsAltered { how }`, stated like a
refusal (loud, always printed) — it is the more dangerous outcome because it
*looks* like success in the coordinator. (b) The harness must record more
than accept/refuse: for every accept it must compare the coordinator's
parsed policy against md's shape — Liana already prints
`primary 2-of-4`, Nunchuk already prints `MINISCRIPT 0-of-3`; the evidence
files hold these strings today and the matrix generator discards them. (c)
Rule-vs-evidence disagreement is a **table-build failure**, never a runtime
tie-break: one of the two is wrong, or the vocabulary is too small, and a
human resolves it before the table ships. See I-7 for the full contract.

### C-3 (Critical) — Nunchuk's acceptance is a string round-trip against md's spelling; a shape-keyed table asserts positives for spellings never measured

libnunchuk's `ParseDescriptors` (`src/descriptor.cpp:628-646`) parses the
descriptor into a `Wallet`, **re-renders it** for four of its five
`DescriptorPath` forms, and accepts only if one rendering is byte-equal (sans
checksum) to the input; otherwise `Failed to verify wallet descriptor`. Every
Nunchuk OK in the matrix is therefore a fact about **(shape × the exact text
`md descriptor` 0.17.0 emitted)**, not about shape alone. Measured
consequences already on file:

- F-624: the `--chain 1` spelling of *every* composed wallet is refused
  (30/30), the multipath and `--chain 0` spellings import (20/20 each) —
  same shape, three verdicts.
- Nunchuk report N-1: `H` hardened marker refused, `h` accepted, `'`
  accepted.
- F-627: the device's restore document spells hardened as `h`; the host
  spells `'`. The operator has two spellings of one wallet in hand.

The design's sentence "that independence is what lets a new coordinator
reuse the whole existing evidence table" is true for shape and false for
Nunchuk, whose verdict is not a function of shape. A future `md descriptor`
change — key ordering inside `multi`, origin formatting, the multipath form —
flips every Nunchuk positive while every shape key is unchanged and the table
keeps saying `Imports`. Nothing in the proposed gates would notice: the
conformance test round-trips *keys*, not renderings.

**Fix.** The evidence tuple is `(shape_key, coordinator, coordinator_version,
renderer)` where `renderer` = md-cli version + form (`multipath` / `chain0` /
`chain1`). A positive names the form: "Nunchuk 1.9: imports the multipath
form from `md descriptor`". The staleness gate re-renders every evidence
descriptor with the current `md` and fails on any byte difference — that is
the check that catches a spelling change before it silently retires the
positives.

### C-4 (Critical) — the evidence cannot be regenerated, so "regenerable and diffable" is an unmet claim and growth has no path

What is committed: seven harness **outputs** under
`design/evidence/composer-fable-r0/` and `scripts/importability-matrix.py`,
which formats them. What is not committed: every program that produced them.
The Liana harness is a Rust crate at `.tmp/fable-liana-harness/`, the parse
driver is `.tmp/fable-liana-parse.py`, the Nunchuk generator is
`.tmp/fable-nunchuk-gen.go` against a libnunchuk build in
`.tmp/fable-nunchuk-lib/`, the Core drivers are `.tmp/fable-liana-core*.py`
— and `/scratch/code/shibboleth/.tmp` is outside every repository
(`git check-ignore` reports it outside the tree). `design/POLICY_HARNESS.md`
documents a different harness (Rust vs device vs Core address agreement),
not these.

Ruling 6 says more versions and more coordinators are expected. Section 3
(staleness) has to say when to re-measure. Both require running the
measurement again, and there is no committed way to run it. The matrix
header's "Core 25.0" is also not a measured version string: the measurements
doc records that the binary was `Bitcoin Satellite version v0.2.4`.

**Fix.** Commit the harnesses as `scripts/measure-liana.sh`,
`scripts/measure-nunchuk.sh`, `scripts/measure-core.sh` (or one driver with a
coordinator argument), each pinning the coordinator source revision it builds
and writing the evidence files the matrix reads. Every evidence row records
the coordinator binary's **self-reported** version (`getnetworkinfo.version`,
`liana --version`, libnunchuk's version symbol), never a label typed by the
person running it. Until this lands, the design should say plainly that the
table is a one-time snapshot.

### I-1 (Important) — the Liana evidence is seven majors stale on the day the design was written, and the span model would hide it

Measured from the checkout's tags:

    v8.0  2024-11-08   src/descriptors/analysis.rs        8ea2a3589b35
    v9.0  2025-01-15   liana/src/descriptors/analysis.rs  48d142c6a1af
    v10.0 2025-04-03                                      e724a34692a4
    v11.0 2025-05-29                                      cf859782617a
    v12.0 2025-07-25                                      cf859782617a
    v13.0 2025-09-29                                      c4e4b0fc6213
    v14.0 2026-04-17                                      c4e4b0fc6213
    v15.0 2026-07-31                                      d03427e42236

The file the rules cite moved and changed content five times. Most of the
v8.0→v15.0 diff is the rust-miniscript `Threshold`/`Arc` migration, but one
change is semantic: `from_multipath_descriptor` at v15.0 calls `_new(…,
compile = false)` with the comment "We don't compile the policy as we assume
it compiles given we started with a descriptor" — so the `InvalidPolicy`
refusal class at *import* time no longer exists somewhere in 9..15. A shape
refused at 8.0 for a compiler limit may import at 15.0; the design's
`RuleSet { since: 8.0, until: None }` would print "Liana 8.0 and newer" from
source read once, at 8.0.

The same is true of every positive: `measured_at: [8.0]` is honest data, and
`Imports { span }` printing anything past 8.0 is an inference ruling 3
forbids.

**Fix.** A rule carries `source_verified_at: &[Version]` (distinct from the
evidence's measured versions); no verdict of either kind prints an open-ended
span; the right endpoint is the last version *verified*, and the copy says
"as of". "26 and newer" is not a sentence this device may print; "26.0-31.1"
is.

### I-2 (Important) — the consent screen makes the verdict skippable by construction, and section 4 must not inherit that placement

`composerConsentLinesFor` appends the coordinator notices (§8w, §8x, §8a)
after the script line, every path's head and lock-echo lines, the key-path
line and the id/stub lines (`composer_consent.go:157-283`). The surface is
`confirmReviewScreen`'s paged form, where `contBtn.Clicked → return true`
runs on every page (`multisig_build.go:1898`) and the pager is drawn only on
overflow (`:1937`). On any multi-path policy — three paths with lock echoes
is already ~12 lines before the first notice — the coordinator verdict is on
page two of a screen whose CONTINUE is live on page one. That is the
"paginated screen hides its overflow" class, applied to the one line whose
job is to stop an operator.

### I-3 (Important) — five hand-written copies of coordinator knowledge already ship, and the design retires none of them

§8a ("Bitcoin Core, Nunchuk and Liana all refuse it"), §8f ("Bitcoin Core
imports this form. Nunchuk cannot import a NUMS policy at all…"), §8g ("Liana
will refuse it"), §8w ("Nunchuk will refuse this wallet; Bitcoin Core imports
it"), §8x ("Bitcoin Core imports it") each state a coordinator verdict in
prose, from a fixed reading of the same sources the registry will encode.
§8x's "Bitcoin Core imports it" is already false for Core 25 on every `tr`
shape in its own class list. The design names "two implementations of one
key" as the defect class it exists to avoid and then leaves five
implementations of the rules on the consent.

**Fix.** The notice bodies become the registry's reason strings (they are
good strings); the consent prints from the registry; a gate asserts that no
`composerCopy*` body names a coordinator.

### I-4 (Important) — "Rust first" is stated, but the walk the key needs exists only in Go, and the design does not say which input the canonicaliser takes

md-codec has the compose-side input model (`compose::{Wrapper, Lock,
HashKind, SpendPath, PathList}`) and a canonical *payload*
(`encode.rs:54-81`). It has no decode-side structural summary: `PolicyShape`
— branch split over `or_*`/`andor`, the "one multi that accounts for every
key" K/N rule, lock kinds read from the wire band, the `Complete` honesty
contract — exists only in the fork's `md/policy_shape.go`. Section 5 must
name porting it (or, better, deriving a template skeleton from the canonical
payload — see the opinion below) as the **first** Rust task, with the Go
becoming a convergence port under a provenance pin.

It must also say the canonicaliser takes the **decoded md1**, never
`PathList`: the consent's own rule is "every line is derived from the decoded
md1, never from composerState", restored payloads have no `PathList`, and
`md shape-key` must accept both a chunk set and a descriptor so the evidence
table (descriptors) and the device (chunks) reach the key through one door.
The conformance test then asserts `descriptor → md1 → key` equals
`chunks → key` for every committed row.

### I-5 (Important) — the key must be ordered over paths and must not inherit the decoder's `sh`/`sh(wsh)` collapse

X24 and X25 differ only in path order (`[2of3, 1of1, 1of1+older]` vs
`[1of1, 2of3, 1of1+older]`); Liana accepts one and refuses the other
(lens 5 report `:484`). "Per-path k/n/sorted" does not say whether the key
is an ordered list or a multiset; it must be ordered, in wire order.

Nunchuk's `PREFIX_MATCHER` (`descriptor.cpp:589-596`) routes
`sh(wsh(sortedmulti(` and `sh(sortedmulti(` differently; the matrix measures
`plain-2of3-shwsh` and `plain-2of3-sh` as separate rows. The fork's decoded
`Template.Root == ScriptSh` is a three-way collapse (`composer_shape.go:175`:
bare `sh`, `sh(wsh)`, `sh(wpkh)`), which is why `composerLianaOutsideModelClass`
takes `root` as a separate argument. The Rust key must carry the wrapper as
four values, from the wire, not from that summary.

### I-6 (Important) — the version a verdict names must be one the operator can find

The matrix's Nunchuk column is "libnunchuk 2.1.1"; the operator's About box
shows the desktop application's version (the checkout on this box is
`nunchuk-desktop-1.9.54`). A consent line reading "Nunchuk 2.1.1" names a
number the operator cannot match to anything they have. `CoordinatorId`'s
version scheme must be the one the application displays, with the library
revision kept as provenance in the evidence row.

### I-7 (Important) — the rule/evidence meeting point is undefined; it should be a table build with three named failure modes

The split is sound. What is missing is where the two meet and what happens
when they disagree. The right place is the generated table's build, and the
cases are:

| rule says | evidence says | outcome |
| --- | --- | --- |
| Refuses(r) | OK | **build failure** — the rule is over-broad, or this is C-2's `ImportsAltered`; a human decides |
| — | REFUSE | allowed: a **measured refusal** at that version, non-generalising (Nunchuk's round-trip refusals have no shape rule and never will) |
| Refuses(r) | REFUSE(r') with r' ≠ r | **build warning** — the "first reason" precision is wrong; the coordinator's error string is mapped to a reason class and compared |
| Refuses(r) | REFUSE(r) | the rule is confirmed at that version |
| — | OK | `Imports` at that (version, renderer) |
| — | — | `Unproven` |

The third row is the gate the Go classifier's order-precision has never had:
`TestFableOutsideLianaModelNamesTheFirstClass` pins the order against the
*review's* reading, not against what Liana printed.

### M-1 (Minor) — lock-value equality classes hide push-size differences near script limits

`older(100)` pushes one byte, `older(65535)` three, `after(1700000000)`
five; two policies in one lock-value class can differ by a few script bytes.
Bounded by the composer's ceilings (20 keys per `wsh` multi, plate census),
so not a live defect — but the key's stated blind spots should list it,
because "same key, different verdict" is exactly what a future reviewer will
be asked to explain.

### M-2 (Minor) — the key-path kind should be three-way now, before F-449 orphans the table

F-449 is an open cycle to add a second NUMS spelling (unspendable xpub) to
the md1 wire because Liana requires it. Nunchuk's `ParseTrDescriptor` treats
`H_POINT` and `IsUnspendableXpub` alike on the key path (`:494-505`); Liana
refuses the raw form. A two-valued `KeyPathKind {NUMS, Spendable}` will be
split later; enumerate `{Spendable, NumsRaw, NumsXpub}` now so the committed
keys survive the wire change.

### M-3 (Minor) — name the `md` subcommands that print the verdict; `md descriptor` is the one that matters

Ruling 5 says `md` refuses the none case with a flag; the design says "md on
the host" without saying where. `md compose` is the obvious site, but
`md descriptor` is where the operator chooses the spelling (C-3, F-624) and
is the last thing they run before pasting into the coordinator. The verdict
belongs there too, naming the form.

### M-4 (Minor) — the Core boundary measurement is two releases, not six

Not an inference to print; a statement of what to measure first. The
tapscript-miniscript merge is `db283a6b6f` (`Merge bitcoin/bitcoin#27255`,
2023-10-08) and the first release tag containing it is `v26.0`. So the
measurement runbook is: the representative `tr`-with-miniscript-leaf
descriptor against 25.2 and 26.0, expecting the flip there, then the latest
release. Ruling 1 stands — the number printed comes from the run.

### N-1 (Nit) — `measured_at` sits on `RuleSet` but describes evidence

Move it to the evidence row (`measured_at`, `renderer`) and give `RuleSet`
`source_verified_at`. Two provenance fields for two sources of truth.

### N-2 (Nit) — `Unproven` needs a span

A coordinator's row is a list of `(span, verdict)` runs. "Liana 8.0: imports;
9.0-15.0: unmeasured" needs `Unproven` to carry the span it covers.

---

## Architect's opinion (judgement, not findings)

### 1. The shape key — key on the canonical template skeleton, not on a semantic summary

`PolicyShape` is the right *display* summary and the wrong *key*. It is a
semantic reading — branches, thresholds, lock kinds — and coordinators do not
parse semantics; they parse text. Liana lifts to a semantic policy and is
structure-insensitive; Nunchuk pattern-matches templates and re-renders
(structure- and spelling-sensitive); Core's `IsSane`/malleability checks are
structure-sensitive. Two md1 trees with identical `PolicyShape` but different
fragments (`or_i` vs `or_d`, `and_v` vs `and_b`) can differ in importability.
For device-composed policies this cannot happen because §5's lowering is
deterministic — but the design says the key is computed from the decoded
md1 so it applies to *any* payload, and there it is too coarse.

The key I would build is the **canonical template skeleton**: md-codec's
canonical payload (placeholders renumbered by first appearance —
`encode.rs:59-62` already does this) rendered as a template, with lock
values replaced by `kind#class` (`older(blocks#1)`, `after(time#2)`),
digests replaced by their kind (`sha256(#)`), and key origins erased; plus a
separate component for the key-identity partitions of C-1. Concretely:

    wsh(or_d(multi(2,@0,@1,@2),or_i(pkh(@3),and_v(v:pkh(@4),older(blocks#1)))))
    | fp-partition per path: [{0},{1},{2}] [{3}] [{4}]

Why this and not the field list in section 1:

- It is exact for what a coordinator parses, so evidence measured on one
  descriptor applies to another **only** when the coordinator would see the
  same thing. Too-coarse is structurally impossible.
- It is not too fine for the case that matters. Every device-composed and
  `md compose`d policy lowers to exactly one skeleton, so all 56 measured
  shapes stay matchable and every preset keeps its positive. A foreign md1
  with an unfamiliar fragment fails to `Unproven` — silence, which is the
  correct verdict for a policy nobody measured.
- Rust-first becomes cheap: the canonicaliser md-codec already owns *is* the
  key, and `md shape-key` is a formatter over it. No port of the Go walk is
  needed for the key (I-4's port is still needed for the display summary, but
  that is a display concern, not a normative one).
- The evidence rows already carry `template` in `fable-liana-shapes.json`;
  the generator's second output is a transcript of `md shape-key` over them,
  as section 2 asks.
- Ordered paths (I-5), the four-way wrapper (I-5), and the three-way key
  path (M-2) fall out of the wire for free instead of being fields to
  remember.

What is *in* the field list that should not be: nothing is harmful, but
"path count" and "per-path k/n" are derived from the skeleton, and listing
them as separate fields invites a second implementation that computes them
differently. What is *missing* from the field list, in order of cost: the
key-identity partitions (C-1, a false positive today); path order (I-5);
the wrapper's `sh`/`sh(wsh)` split (I-5); the renderer identity, which
belongs on the evidence tuple rather than the key (C-3).

### 2. Rules vs evidence — keep the split, make the table build the place they meet

The split is right because the two generalise differently: a source-derived
refusal is a claim about a program at the versions whose source was read; a
measured positive is a claim about one binary and one spelling. Neither can
stand in for the other. The failure mode is not that they disagree — they
*will* disagree, and X24 is the proof that a disagreement can be the most
valuable fact in the table — it is that nobody is told. I-7's table is the
contract: disagreement is a build failure with a named class, resolved by a
person, and the resolution is committed (a new verdict kind, a narrowed
rule, or a re-measurement). The generated `verdicts` table is the single
artifact both consumers read; rules and evidence are its inputs, never
consulted at runtime.

One more thing the split buys, which the design should say: **measured
refusals are admissible** (I-7 row 2). Ruling 3 says refusals are
rule-derived and always stated; that was written before Nunchuk's round-trip
check was understood. A refusal a real harness printed is at least as good as
a rule at that version; it just does not extend to other versions.

### 3. Growth — the span model holds if spans are closed and per-coordinator behaviour is a piecewise function

`Coordinator { rules: &[RuleSet] }` with `since`/`until` is fine for
non-monotonic behaviour: a coordinator that tightens at 9 and loosens at 12
is three `RuleSet`s, and the aggregator prints three runs. What breaks it is
open-ended spans (I-1) and versions the operator cannot see (I-6). The
model I would state in section 1's successor: **a coordinator's row is a
function from verified version to verdict, printed as its runs, each run
closed at both ends by a version that was actually verified.** Adding a
version is adding a point to that function; adding a coordinator is adding a
function. The display then has a bounded grammar per row regardless of how
many runs exist: worst run first, then "other versions: unmeasured".

Two growth hazards the design should name: coordinators that are libraries
with several front-ends (libnunchuk under desktop and mobile — which version
does the operator see?), and coordinators whose behaviour depends on a
setting rather than a version (Core's `-deprecatedrpc`, Liana's network).
Both are "version" in the model and neither is a number; a `Version` type
that is an opaque ordered label per coordinator, not a semver triple, keeps
the model honest.

### 4. Staleness (section 3) — what I would build

The premise to get right first: **truth about a version does not decay;
relevance does.** "Liana 8.0 imports this" is as true in 2030 as today,
because 8.0's binary is immutable. What changes is whether anyone runs 8.0.
So the design should not build a decay model ("refuse to claim anything
older than N"); it should build provenance and let the operator do the
matching. Concretely:

1. **Every verdict carries `(version verified, renderer, date)`, and the
   copy is always version-qualified.** The device never prints "Liana
   imports"; it prints "Liana 8.0 imports (2026-09); newer: unmeasured". The
   date is not decoration: a plate outlives releases, and a date is the one
   thing a future operator can judge against with no other context.
2. **No open-ended span, for either verdict kind** (I-1). "As of" is the
   only tense a positive may use. A refusal may say "every version checked
   through 15.0" when the source was read at 15.0.
3. **Committed, runnable harnesses** (C-4), each pinning the coordinator
   source revision it builds, producing the evidence files the matrix reads.
   Staleness is unmanageable without re-measurement, and re-measurement
   without a committed harness is a research project every time.
4. **A `KNOWN_RELEASES` file, maintained by hand**, listing each
   coordinator's newest known release, its date and the URL it was read
   from — and a build-time check that fails the firmware build (or degrades
   the row to `Unproven` with a stated reason) when the newest known release
   of a coordinator is more than one major past every verified version. This
   is the one gate that turns "we should re-measure" into a command that
   fails. Today it would fail on Liana (8.0 verified, 15.0 known) and on
   Nunchuk (libnunchuk checkout is 2025-12-05).
5. **A renderer gate** (C-3): re-render every evidence descriptor with the
   current `md` and diff. Any byte change retires every Nunchuk positive
   until re-measured — automatically, in the build, not by someone
   remembering that Nunchuk round-trips text.
6. **The evidence is a snapshot with a commit**, and re-measurement produces
   a diff of verdicts (section 2 already says this; it is the right artifact
   for review — keep it, and make the harness the thing that produces it).

What I would *not* build: a freshness threshold in days on the device.
The device cannot know the date reliably (§6b: "cannot tell the time"), and a
verdict that changes wording as the clock moves is a verdict nobody can
test.

### 5. The device surface (section 4) — bounded by ordering, not by count

The row list will grow without bound; the screen will not. Three principles
make that safe:

1. **The verdict rows go on the first page**, directly after the script
   line, before the paths. The paths are what the operator consents to, but
   the coordinator verdict is what changes whether they *should*. I-2 shows
   the current placement puts the verdict behind a live CONTINUE.
2. **Worst first, so overflow can only hide good news.** Order: `Refuses`,
   `ImportsAltered`, `Unproven`, `Imports`. If the registry ever holds more
   coordinators than the first page fits, what slides to page two is a
   positive, whose loss costs a convenience; a refusal that slid would be the
   W-2 class — a fact no hand can reach.
3. **Collapse the positives, never the negatives.** Above K rows, the
   `Imports` run becomes one line: "Imports: Liana 8.0, Core 26.0-31.1
   (+2 more)". Refusals are never collapsed; each names its reason.

Then the two gates this repo already knows how to write: assert the
first-frame row count against the registry size (the kind-picker lesson),
and measure the copy headroom by padding and bisecting rather than reasoning
about it.

The **none** case is its own confirm-to-proceed screen (ruling 2), before the
consent, in §8a's shape — bounded by construction because it has one body.
It should list the refusals with reasons and say what remains: "Only md can
rebuild this wallet, and md cannot sign."

One thing to resist: a summary line on the consent *and* a detail screen
elsewhere. That is two copies of the verdict, and I-3 is what two copies
become.

### 6. What belongs in section 5 (Rust-first sequencing and testing)

In order, each with its gate:

1. `md-codec`: the skeleton canonicaliser + `Verdict`/`ImportsAltered` types
   + the generated `verdicts` table, with vectors that pin every (shape,
   coordinator, version) cell. Gate: the conformance test (chunks → key ==
   descriptor → key for every evidence row) and the rule/evidence build
   (I-7).
2. `md-cli`: `md shape-key`, the verdict on `md compose` and
   `md descriptor` (naming the form), the none-case refusal with its flag
   (ruling 5). Gate: CLI vectors including the flag path and the
   template-only conditional (C-1's unknown component).
3. Harnesses committed (C-4) and run once to regenerate the evidence
   byte-identically from the current tree — the reproducibility proof.
4. Fork: the Go port of the canonicaliser with a provenance pin; the
   registry consumed from generated Go (not hand-ported rules); the consent
   rows and the none-case screen; retire the five hand-written notices
   (I-3). Gate: cross-language vectors — the same key and verdict from Rust
   and Go for every evidence row — because this repo has measured that
   887/887 green per-repo tests can hide a Go/Rust disagreement.
5. Emulator walk for the consent with the registry at N and at N+K
   coordinators (a synthetic registry), asserting first-page rows and that
   a refusal is never off the first page.

---

## Rulings — none unworkable; two carry a consequence

1. **Core as a version boundary** — workable; the measurement is two
   releases (M-4).
2. **None is confirm-to-proceed** — workable; bounded by construction as
   its own screen.
3. **Positive where measured, silent where not** — workable, but combined
   with ruling 6 it forces every positive to be version-and-renderer-pinned
   forever, and the copy to say "as of". The design has not yet drawn that
   consequence; I-1 and section 3 above draw it. Also: ruling 3's "refusals
   are rule-derived" should admit measured refusals (I-7).
4. **Rust first** — workable, and cheaper than the design implies if the key
   is the canonical skeleton (opinion §1); expensive if it is `PolicyShape`,
   because that walk exists only in Go (I-4).
5. **`md` refuses none with a flag** — workable; needs the subcommand named
   (M-3) and the template-only case defined (C-1).
6. **Designed for growth** — workable only once the harnesses are committed
   (C-4); without them the registry can grow rules but never evidence, and
   under ruling 3 that means it can grow refusals but never a positive.

---

## Blind spots of this review

I did not run any harness; every verdict cited is from the committed
evidence files and the reports. I did not read Liana v9..v14 source beyond
the blob ids and the v8.0→v15.0 diff, so "no semantic change other than
`compile=false`" is a claim about that diff, not a per-version audit. I did
not measure the consent screen's first-page line budget; I-2's "page two" is
from line counts, not from a frame.
