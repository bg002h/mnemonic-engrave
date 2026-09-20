# DESIGN — which wallet coordinators will take this policy

**Status: DRAFT, sections 1-2 approved by the operator; 3-5 unwritten.**
Brainstorm in progress 2026-09-20. Not a spec yet.

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

## Section 1 — the data model (APPROVED)

The unit is not a coordinator; it is a **coordinator at a version range**.

```rust
struct Coordinator { id: CoordinatorId, name: &'static str, rules: &'static [RuleSet] }

struct RuleSet {
    since: Version,                    // inclusive
    until: Option<Version>,            // exclusive; None = "and newer"
    measured_at: &'static [Version],   // releases evidence actually came from
    refuse: fn(&PolicyShape) -> Option<Reason>,
}

enum Verdict { Refuses { reason, span }, Imports { span }, Unproven }
```

**Adding a version** is a new `RuleSet` inside an existing coordinator.
**Adding a wallet** is a new `Coordinator`. Neither touches the classifier, the
display, or any existing rule.

Ruling 1 stops being a Core special case and becomes the general output shape:
Core is a coordinator with two rule sets, one below the boundary that refuses
tapscript miniscript and one above that does not, and "26 and newer" is what
the aggregator prints when they disagree. A future Nunchuk 2.2 that starts
accepting `multi_a` is the identical mechanism with no new concept.

**The shape key is coordinator-independent** — a canonical form of the policy
alone: wrapper, key-path kind, per-path k/n/sorted, lock KIND, hash kind, path
count, plus lock-value EQUALITY classes (Liana's "two paths with one lock"
turns on two values being equal, not on what they are). Never key material,
never lock values. That independence is what lets a new coordinator reuse the
whole existing evidence table: measure the new wallet against the shapes
already enumerated, and every old key keeps its meaning.

**Evidence is a generated table**, `shape_key -> {(coordinator, version)}`,
holding only what a real harness printed OK. `Imports` requires a hit in it;
a rule alone can only ever produce `Refuses` or `Unproven`.

## Section 2 — rules and the evidence pipeline (APPROVED)

**A rule is a predicate over `PolicyShape` written in the coordinator's own
order of refusal**, each clause citing the source that establishes it. That
shape already exists and works: `composerLianaOutsideModelClass`
(fork `gui/composer_consent.go:381`) cites `analysis.rs:554-558`, `:586-587`,
`csv_check:139-145` and returns the FIRST class that applies, which is what
makes the reason it names the one the coordinator would actually hit first. It
moves to Rust as the template; the Go becomes the convergence port.

The other two read the same way. **Nunchuk**: the `wsh(sortedmulti(` prefix
gates its multisig route (`descriptor.cpp:596-598`), `ParseDescriptors`
round-trips only four of five contexts (`:658-661`), and `multi_a` / raw NUMS
refuse. **Core**: miniscript in `wsh` only below the boundary release, key-less
paths refused as "witnesses without signature exist", `multi_a` leaves
unparsed.

**The pipeline's one hard rule: the shape key has exactly one implementation.**
`scripts/importability-matrix.py` already generates the matrix from
`design/evidence/`; it gains a second output, the shape-key table — but it must
compute those keys **by calling md-codec's own canonicaliser**, not by
reimplementing it in Python. Two implementations of one key is a defect class
this repo keeps finding: they drift, and the failure is silent in the worst
direction — `Imports` quietly stops firing, or fires on the wrong shape. So the
generator shells out to `md` (a small `md shape-key` subcommand) and the table
is a transcript of md-codec's own answers.

That also makes the table regenerable and diffable: re-running after a new
measurement round produces a diff showing exactly which shapes changed verdict,
which is the artifact a reviewer needs.

**Gates:** vectors pin every rule's verdict per shape, and a conformance test
asserts every committed key round-trips through the canonicaliser — so a
key-format change that would orphan the table fails loudly instead of silently
emptying the positives.

## Sections 3-5 — NOT YET WRITTEN

3. **Staleness** (promoted to first-class at the operator's direction): the
   device bakes a model of three third-party programs, at pinned versions, onto
   plates that outlive those versions.
4. **The two surfaces**: the SH2 consent screen (whose row list must stay
   bounded as coordinators are added — a paginated screen hides its overflow)
   and `md`'s refusal-plus-flag.
5. **Rust-first sequencing and testing.**

## Measured context this design rests on

`design/IMPORTABILITY_composer_shapes.md` — 56 shapes (30 device-composed,
26 `md compose`), generated from `design/evidence/composer-fable-r0/`.
Totals: Nunchuk OK 20 of 30 measured; Liana OK 17 of 56; Core 25.0 OK 39 of 56;
Core 31.1 OK 54 of 56. Every preset is in the measured set, so for
preset-built policies — the common case — a measured "yes" can fire.
