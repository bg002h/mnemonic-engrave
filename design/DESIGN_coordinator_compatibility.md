# DESIGN — which wallet coordinators will take this policy

**Status: DRAFT. Sections 1-2 approved, then REVISED under the fable architect
review (4C/7I/4M/2N, `design/agent-reports/coordinator-compat-fable-architect.md`).
Sections 3-5 written and awaiting approval.** Brainstorm in progress
2026-09-20. Not a spec yet.

The architect's verdict was *"the family of architecture is right; the instance
is not yet safe"* — the registry, the source-derived refusals, the
measured-only positives and the coordinator-independent key all stand. Four
measured routes to a false `Imports` on steel did not, and the revisions below
are what closes them.

## The ask

The SH2 composer should tell the operator which wallet coordinators
(Nunchuk, Liana, Bitcoin Core) will import the policy they have just built:
all three, a named subset, or none — and "none" must be loud.

## Operator rulings (settled; do not re-litigate)

1. **Core is reported as a VERSION BOUNDARY**, not a yes/no. Measured over 56
   shapes, Core 25.0 accepts 39 and Core 31.1 accepts 54 — fifteen shapes
   differ, mostly tapscript miniscript — and the device cannot know which Core
   the operator runs. The exact boundary release **must be measured**, not
   inferred from the 25-vs-31 gap.
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

## Section 1 — the data model (REVISED after review)

The unit is not a coordinator; it is a **coordinator at a version range**.

```rust
struct Coordinator { id: CoordinatorId, name: &'static str, rules: &'static [RuleSet] }

struct RuleSet {
    since: Version,                      // inclusive, a version actually verified
    until: Version,                      // inclusive, a version actually verified
    source_verified_at: &'static [Version],
    refuse: fn(&Skeleton) -> Option<Reason>,
}

enum Verdict {
    Refuses { reason: Reason, span: Span },
    ImportsAltered { as_read: Description, span: Span },
    Imports { span: Span, renderer: RendererId },
    Unproven,
}
```

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
(`encode.rs:59-62`) — rendered as a template with lock values replaced by
`kind#class`, digests by their kind, and origins erased:

    wsh(or_d(multi(2,@0,@1,@2),or_i(pkh(@3),and_v(v:pkh(@4),older(blocks#1)))))
    | fp-partition per path: [{0},{1},{2}] [{3}] [{4}]

Too-coarse becomes structurally impossible, every measured shape stays
matchable, and a foreign md1 with an unfamiliar fragment falls to `Unproven` —
silence, the correct verdict for a policy nobody measured. It also makes
ruling 4 *cheaper*: the canonicaliser md-codec already owns **is** the key, so
`md shape-key` is a formatter over it and no Go walk needs porting for the
normative path.

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
(fork `gui/composer_consent.go:381`) is the working template; it moves to Rust
and the Go becomes the convergence port.

**Measured refusals are admissible.** Ruling 3 said refusals are rule-derived;
that was written before Nunchuk's round-trip check was understood. A refusal a
real harness printed is at least as good as a rule at that version — it simply
does not extend to other versions.

**Rules and evidence meet at a table BUILD, never at runtime.** The generated
`verdicts` table is the single artifact both consumers read. A disagreement
between a rule and a measurement is **a build failure with a named class**,
resolved by a person and committed — as a new verdict kind, a narrowed rule, or
a re-measurement. Disagreements are not a flaw in the split; X24 is proof that
a disagreement can be the most valuable row in the table. The flaw would be
nobody being told.

**The shape key has exactly one implementation.** The generator computes keys
by calling `md shape-key`, never by reimplementing the canonicaliser in Python.
Two implementations of one key drift, and the failure is silent in the worst
direction: positives quietly stop firing, or fire on the wrong shape.

**Gates:** vectors pin every `(shape, coordinator, version)` cell; a
conformance test asserts `chunks -> key == descriptor -> key` for every
evidence row; and the rule/evidence build fails on an unresolved disagreement.

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
from a 21-month-old reading (**F-633**).

Six mechanisms:

1. **Every verdict carries `(version verified, renderer, date)`, and the copy
   is always version-qualified.** The device never prints "Liana imports"; it
   prints "Liana 8.0 imports (2026-09); newer: unmeasured". The date is not
   decoration — a plate outlives releases, and a date is the one thing a future
   operator can judge against with no other context.
2. **No open-ended span, for either verdict kind.** "As of" is the only tense a
   positive may use.
3. **Committed, runnable harnesses**, each pinning the coordinator source
   revision it builds. Today every harness lives in `/scratch/.tmp`, outside
   all three repos — only the *consumer* (`scripts/importability-matrix.py`) is
   committed, never the producers. Re-measurement without a committed harness
   is a research project every time, and under ruling 6 that means the registry
   can grow rules but never evidence: refusals forever, positives never.
4. **A hand-maintained `KNOWN_RELEASES` file** — each coordinator's newest
   known release, its date, the URL it was read from — and a build-time check
   that fails, or degrades the row to `Unproven` with a stated reason, when the
   newest known release is more than one major past every verified version.
   **This is the one gate that turns "we should re-measure" into a command that
   fails.** It would fail today on Liana (8.0 verified, 15.0 known) and on
   Nunchuk.
5. **A renderer gate:** re-render every evidence descriptor with the current
   `md` and diff. Any byte change retires every Nunchuk positive until
   re-measured — automatically, in the build, not by someone remembering that
   Nunchuk round-trips text.
6. **Evidence is a snapshot with a commit**, and re-measurement produces a diff
   of verdicts — the artifact a reviewer needs.

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
   (`multisig_build.go:1898`), which makes it skippable by construction.
2. **Worst first, so overflow can only hide good news.** Order: `Refuses`,
   `ImportsAltered`, `Unproven`, `Imports`. If the registry ever exceeds the
   first page, what slides to page two is a positive, whose loss costs a
   convenience. A refusal that slid would be the W-2 class — a fact no hand can
   reach.
3. **Collapse the positives, never the negatives.** Above K rows the `Imports`
   run becomes one line ("Imports: Liana 8.0, Core 26.0-31.1 (+2 more)").
   Refusals are never collapsed; each names its reason.

**One copy of the verdict, not two.** No summary line on the consent *plus* a
detail screen elsewhere — five hand-written coordinator notices already ship
(§8a, §8f, §8x and two more), and this design retires them rather than adding a
sixth.

**Gates this repo already knows how to write:** assert the first-frame row
count against the registry size (the kind-picker lesson — a paginated screen
hides its overflow), and measure copy headroom by padding and bisecting rather
than by reasoning about it.

### `md` on the host

`md shape-key` emits the key. `md compose` and `md descriptor` print the
verdict, naming the form. The **none** case refuses with exit non-zero and
names the flag that proceeds (ruling 5) — no TTY prompt, because this project
has already shipped a tool that blocked on a TTY and looked like a hang.

The none case on the device is **its own confirm-to-proceed screen** before the
consent, in §8a's shape — bounded by construction because it has one body. It
lists the refusals with reasons and says what remains: *"Only md can rebuild
this wallet, and md cannot sign."*

## Section 5 — Rust-first sequencing

Each step with its gate. Nothing starts in the fork.

1. **`md-codec`** — the skeleton canonicaliser, the four `Verdict` kinds, the
   generated `verdicts` table, vectors pinning every `(shape, coordinator,
   version)` cell. *Gate:* the conformance test (`chunks -> key ==
   descriptor -> key` for every evidence row) and the rule/evidence build.
2. **`md-cli`** — `md shape-key`; the verdict on `md compose` and
   `md descriptor`; the none-case refusal and its flag. *Gate:* CLI vectors
   including the flag path and the template-only case.
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

## Open, and blocking a spec

- **The Core boundary release is unmeasured.** Ruling 1 requires it measured.
  `design/COORDINATOR_COMPAT_MEASUREMENTS.md` narrows it to one capability —
  all 15 discriminating shapes are `tr` with a tapscript-miniscript leaf, zero
  `wsh`/`sh` shapes differ — so the measurement is one representative
  descriptor per candidate release, not 15 shapes across 6. Until then the
  honest verdict for such a policy is `Unproven`.
- **Liana must be re-measured at v15.0** (F-633).
- **`K`, the collapse threshold**, is a measurement against the real frame, not
  a number to pick here.

## Measured context this design rests on

`design/IMPORTABILITY_composer_shapes.md` — 56 shapes (30 device-composed,
26 `md compose`), generated from `design/evidence/composer-fable-r0/`.
Totals: Nunchuk OK 20 of 30 measured; Liana OK 17 of 56; Core 25.0 OK 39 of 56;
Core 31.1 OK 54 of 56. Every preset is in the measured set, so for
preset-built policies — the common case — a measured positive can fire.
