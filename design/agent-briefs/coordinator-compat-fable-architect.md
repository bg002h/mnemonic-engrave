# Brief — fable architect opinion on the coordinator-compatibility design

**Dispatched at the operator's explicit request.** (Standing policy is that
fable is not a reviewer tier; the operator asked for this one by name, which
overrides it.)

**Artifact:** `design/DESIGN_coordinator_compatibility.md` at `d92f90ad`.
Sections 1-2 are approved by the operator; 3-5 are unwritten, and saying what
belongs in them is squarely in scope.

## The question

**Is this the right architecture for a device that tells an operator which
wallet coordinators will import the policy they just built?**

Not a correctness review of prose. An architect's opinion: what is wrong with
the shape of this, what will hurt in two years, and what would you do
differently. Be direct — a polite "looks reasonable" is worthless here.

## Six operator rulings are SETTLED — do not re-litigate

They are listed in the design. In particular do not re-argue: Core reported as
a version boundary; the none-case being a confirm-to-proceed rather than a
refusal; positives only where measured; Rust-first; `md` refusing with a flag
rather than prompting; designing for growth. **If one of them is
architecturally unworkable, say so once, with the consequence — that is
different from re-opening it.**

## Where to push hardest

1. **The shape key.** It is the load-bearing abstraction: a canonical form of
   the policy that decides whether measured evidence applies to the thing in
   front of the operator. Too coarse and a measured `Imports` gets claimed for
   a shape that was never measured — a false positive that ends on steel. Too
   fine and it never matches and the feature is a permanent dash. Is the
   proposed content (wrapper, key-path kind, per-path k/n/sorted, lock KIND,
   hash kind, path count, lock-value EQUALITY classes) right? What is missing,
   and what is in it that should not be?
2. **Rules vs evidence as separate sources of truth.** Refusals come from
   source-derived rules that generalise; positives come from a measured table.
   Is that split sound, or does it produce a system where the two disagree and
   nobody notices? What happens when a rule says "refuses" and the evidence
   table says a real harness imported it?
3. **Growth.** More versions per coordinator, more coordinators. Does the
   `RuleSet` span model actually hold up, or does it collapse the first time a
   coordinator's behaviour changes in a way that is not monotonic in version?
4. **Staleness (section 3, unwritten).** The device bakes a model of three
   third-party programs onto plates that outlive those programs' releases.
   What is the right design — version pins in the verdict, a measured-at date,
   a re-measure gate, a refusal to claim anything older than N? Say what you
   would build.
5. **The device surface (section 4, unwritten).** The consent screen's row list
   must stay bounded as coordinators are added; this project has already been
   bitten by a paginated screen whose overflow made one of four kinds
   unchoosable. What is the right bounded display for an unbounded registry?

## Context you may want

- `design/IMPORTABILITY_composer_shapes.md` — the 56-shape measured matrix.
- `design/SPEC_wallet_policy_composer.md` §7e (consent), §8a, §8f, §8x.
- Fork `/scratch/code/shibboleth/seedhammer`, `gui/composer_consent.go:381`
  (`composerLianaOutsideModelClass`) — the existing one-coordinator classifier
  this generalises.
- Primary `/scratch/code/shibboleth/descriptor-mnemonic` at `b2c5d693`.

## Output

Severity per project standard (Critical / Important / Minor / Nit) for
anything you consider a defect in the design, plus a clearly-marked
**architect's opinion** section that is allowed to be a judgement rather than a
finding. If you think the whole approach is wrong, lead with that.

**FINAL ACTION: write the report to
`design/agent-reports/coordinator-compat-fable-architect.md`** and return ONLY
a one-paragraph summary, that path, and your counts. Do not return it inline.
