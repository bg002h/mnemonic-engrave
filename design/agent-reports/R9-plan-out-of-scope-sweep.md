# R9 — Out-of-scope sweep, `IMPLEMENTATION_PLAN_mt_v0_1.md`

**Scope of this review.** One document,
`design/IMPLEMENTATION_PLAN_mt_v0_1.md` (578 lines), read-only. One question:
does the plan **direct anyone to BUILD** something `mt` v0.1 does not need?
Checked against `design/SPEC_mt_v0_1.md` (3729 lines) as source of truth.
`./scripts/plan-cite-check.sh` was reported clean by the dispatcher and was not
re-run; citation formatting/existence was out of scope here per the brief —
what was checked instead was whether each citation's **target text actually
supports the claim made about it**, which a cite-existence checker cannot
verify.

## Method

Two passes:

1. **Direction 1 (deferred/excluded work).** Grepped the plan for the excluded
   classes named in the brief: `mt qr`/QR config, `sysw`, SH2/machine
   engraving, plate counts/budgets, transaction construction/signing/timelock
   selection, script evaluation, redundancy/fountain coding, a shared BCH
   crate. Every hit was in a sentence *explaining a deferral or a won't-fix*,
   never a sentence directing construction. None found.

2. **Direction 2 (no spec requirement).** Walked every deliverable, test,
   gate, fixture and script named in P0–P6 and §2a, and for each one that
   carried a `§`-citation, pulled the actual spec text at that
   decision/appendix number (the plan's citation scheme is `§<section>.<item>`
   for decisions inside §1, refusals inside §8, open questions inside §10, and
   appendix items inside §12 — confirmed by cross-indexing the numbered lists
   at §1, §8, §10 and §12 against every citation the plan makes) and confirmed
   the cited text says what the plan claims it says. For items with no
   citation, searched the spec independently for grounding.

## Direction 1 — deferred/excluded work

No occurrence of QR/`sysw`/SH2/plate-count/plate-budget/transaction-construction/
script-evaluation/redundancy/shared-crate language directs construction.
Confirmed by grep (`-i "plate count|plate budget|fountain|redundan|sysw|SH2|
seedhammer|script eval|construct.*transaction|sign.*transaction|timelock"`)
against the plan text: every hit is inside §3 ("What this plan deliberately
does not do"), §1's fork-vs-shared-crate discussion (explicitly rejecting the
retired shared-crate direction), or a "real signed segwit transaction" used
only as **input data** for the pinned test vector — using an already-signed
transaction as a fixture is not "transaction construction or signing," it is
consuming the exact artifact `mt encode`'s spec-defined input contract
requires (§8.2e).

No finding.

## Direction 2 — spec-grounding audit

Spot-checked citations, chosen for being either load-bearing (gate content) or
the kind of specific numeric/formula claim most likely to have drifted from
the spec during the two fold rounds:

| plan claim | citation | spec text checked | result |
| --- | --- | --- | --- |
| §10.10 lists exactly 5 operator-inputs with no supply path | §10.10 | line 3018-3029 input table: 8 rows total minus PSBT (already has a path) minus plate budget (§8.7, MOVED to QR-deferred) minus module size (§8.8, QR-only per line 1162) = 5 (FROM, TO, TO-label, input values, node location) | matches exactly |
| free-text `TO` label needs its own flag | §10.4 | line 3490: "The flag is the point, not a convenience... requiring an explicit flag makes it an act of assertion" | matches |
| BEARER warning carries both halves (checked-by-shape + exotic-input-can-defeat-it) | §5 | lines 1696-1704, verbatim two-paragraph warning text | matches, plan does not add or drop either half |
| stderr legend now 5 fields (was 6 before `PLATE n OF m` deletion) | §5 | lines 1720-1761 record the count going 6→5 on 2026-08-23 | matches |
| `position = codeword_index + 4` | §1.1 | line 1046: "A BCH codeword index `k` is position `k + 4`" | matches |
| ported module list (`bch.rs, bch_decode.rs, chunk.rs, header.rs, mod.rs, pipeline.rs` + root `consts.rs, error.rs`) | mk-codec, not spec | `find` against `mnemonic-key/crates/mk-codec/src/{string_layer,}` (mechanical, not spec, but load-bearing since R8 previously caught this list mis-attributed to `md`) | matches exactly, 1:1 |
| BALANCED not FILLED chunking, `bytes_per_chunk = ceil(len/count)` | §3b | lines 1464-1479, normative block, byte-identical formula | matches |
| CUT / PREFIX rows appended below STATUS | §1.1 | lines 720-725, literal example block | matches |
| row-presence table ("present when") behind P4's differential gate | §1.1 | lines 658-666, the actual table | exists, matches P4's description of what must differ vs. agree |
| SPENT — ALREADY CONFIRMED checked first, before any input classified | §1.1 | lines 543-546 | matches |
| DEAD requires parent `confirmations ≥ 1` | §1.1 | line 567 | matches |
| fixture corpus contents (binary/base64/line-wrapped/CRLF/trailing-newline/uppercase-hex/`0x`-hex/hex-PSBT/raw-tx) | §8.2e | lines 2270-2327, the ordered sniffing procedure and its rationale paragraphs | every corpus item is named or directly implied in the cited text; none invented |
| chunking property test BALANCED-vs-FILLED | §3b / §12.12 | line 1479 comment `# BALANCED, not filled`; §10 item 12 "Should mt1 FILL its chunks rather than balance them" | matches |

No citation checked came back unsupported, mis-scoped, or broader than its
target. Two places where the plan's *own* correction history is visible
(P1's header-bit-width note, P2's legend-field-count note) were verified
against current spec text rather than trusted as self-report, and both are
now accurate.

### Candidate items considered and not counted as findings

- **P4's live-node smoke test** ("A synced `bitcoind` is available on this
  machine, and one manual run against it is worth doing — but it must not
  gate CI"). Considered under "generality/infrastructure with no caller."
  Not a finding: the sentence is explicitly advisory and explicitly
  **non-gating** — it creates no deliverable, no committed script, no test
  nobody asked for. It is one sentence of context for why the node-fixture
  gate exists, not a build directive.
- **`mutate-refusals.sh` (P5) and `tests/refusals.toml` (P5).** These are
  exactly the kind of "test infrastructure built for its own sake" the brief
  warns about, so they were checked hardest. Both trace to a real gate need:
  every refusal they cover is a spec-required refusal from §8, and the
  scripts exist to prove those *already-required* refusal tests are not
  vacuous (R8 gates I6/I7) — they test that spec-mandated behavior is
  actually gated, not new behavior. In scope.
- **P0's CI/lint/opt-level setup.** Standard repo bootstrapping matching
  constellation convention (and this repo's own standing CLAUDE.md), not a
  feature deliverable; not evaluated against the spec because it isn't
  spec-shaped work.
- **§1's fork-vs-shared-crate discussion and §2a's WON'T-FIX record.** Per the
  brief, these are decision records / retracted-approach explanations, not
  build directives. Confirmed neither directs any construction — §1 concludes
  by forking (matching `mk`, the established pattern) and explicitly rejects
  parameterization/shared-crate/future-absorption framing; §2a explains why
  three bespoke tests are deliberately NOT being added, with reasoning.

## Result

**Clean. 0 Critical, 0 Important, 0 Minor.**

No deliverable, test, gate, fixture or script in this plan builds a deferred
or excluded thing, and none was found to lack a traceable spec requirement.
The plan's heavy citation density held up under verification of citation
*content*, not just citation *existence*: every spot-checked claim matched the
cited spec text exactly, including several numeric/formula claims (input-count
arithmetic, bit-position formulas, field counts) that would be the first place
a stale citation shows up.
