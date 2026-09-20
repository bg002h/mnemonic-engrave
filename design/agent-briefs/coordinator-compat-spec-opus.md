# Brief — opus review of the coordinator-compatibility spec: sound, and IMPLEMENTABLE?

**Artifact:** `design/DESIGN_coordinator_compatibility.md` at `4c6c5224`.
This is the operator's spec-review gate. It is a design doc, not yet an
implementation plan.

## Two questions, and the second is the one no round has asked

**A. Is the design sound?** Specifically: did it correctly fold the fable
architect review (`design/agent-reports/coordinator-compat-fable-architect.md`,
4C/7I/4M/2N), and did the fold introduce anything new? The four Criticals were
the shape key, the missing `ImportsAltered` verdict, Nunchuk's byte round-trip,
and the un-regenerable evidence.

**B. Is it IMPLEMENTABLE as written?** Walk **plan 1 only** — md-codec +
md-cli, section 5 steps 1-2 — as an implementer who may invent nothing. At
each step: does the design make every decision the step needs, or is there a
choice left open where two readings produce materially different code? Name
each gap as *"at <step>, an implementer must decide X, and the design does not
say"*.

This lens found two Criticals on the last cycle that four correctness-shaped
rounds had missed, so spend real budget on it.

Worth probing hardest:

1. **The skeleton key.** The design says it is md-codec's *existing*
   canonicaliser (`encode.rs:59-62`) rendered as a template, with lock values
   as `kind#class`, digests as their kind, origins erased, plus fingerprint
   partitions. Go and look: does that canonicaliser actually exist and produce
   what the design assumes, or is real new work being described as "already
   owned"? What exactly is `kind#class` — the design never defines the class
   numbering, and two readings give different keys.
2. **The fingerprint partition.** Is it computable from the md1 wire alone, at
   the point the key is computed, for every payload form — including the
   template-only form with no keys seated? If it is not, the key is
   undefined for a case the composer actually produces.
3. **The verdicts table build.** "A disagreement is a build failure with a
   named class" — are the classes named? Can an implementer enumerate them?
4. **`ImportsAltered`'s payload** is `as_read: Description`. What is a
   `Description`, who writes it, and is it derived or hand-authored per row?
   A hand-authored field in a generated table is a maintenance trap.
5. **The none-case flag on `md`** — the design says "names the flag"; does it
   name it? And what is the verdict for a template-only policy with no keys,
   where no coordinator can be asked anything?

## Settled — do not re-litigate

Six operator rulings, listed in the design. Also settled: the architecture's
family (registry, source-derived refusals, measured-only positives,
coordinator-independent key) was endorsed by the architect review; the
three-plan split; and the decision NOT to build a device-side freshness clock.

## Already machine-verified by the controller — do not re-derive

- `encode.rs:59-62`, `descriptor.cpp:628-646` (Nunchuk's round-trip is string
  equality over four paths, no `INTERNAL_ALL`), `multisig_build.go:1898`,
  `gui/composer_consent.go:381` all resolve.
- The Core boundary: 15 shapes separate Core 25.0 from 31.1, **all `tr`**, zero
  `wsh`/`sh` — so the boundary is one capability, tapscript miniscript. The
  boundary release itself is UNMEASURED and the design says so.
- **F-633 is measured**: Liana v8.0 vs v15.0 over 289 descriptors — 289/289
  verdicts identical, 73/73 accepted policies identical in inferred policy and
  addresses, 10 refusals changed message only (both key-less shapes, 5 variants
  each), none changed verdict. Harness committed at `harnesses/liana/`.
- 56-shape matrix totals in `design/IMPORTABILITY_composer_shapes.md`.

## Rules of evidence

Read the real repos; do not reason from the design's prose about what code
exists. `/scratch/code/shibboleth/descriptor-mnemonic` (`b2c5d693`) and
`/scratch/code/shibboleth/seedhammer` (`7b6f2fb`). Every finding names the
state and the wrong outcome, or the decision an implementer cannot make.
Do NOT modify tracked files; delete scratch files; leave trees clean.

## Output

Severity per project standard; secret-handling is never C/I here.
**FINAL ACTION: write the report to
`design/agent-reports/coordinator-compat-spec-opus.md`** and return ONLY a
one-paragraph summary, that path, your answer to B (implementable yes/no, and
the gaps), and C/I/M/N counts.
