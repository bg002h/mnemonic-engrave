# Continuity — wallet-id enumerate R0 + demo 3b corrections (2026-09-17, later)

Supersedes the OPEN §1 of `CONTINUITY_2026-09-17_demo_and_v098.md`. Everything
below is on git; nothing is held locally.

## The spec cycle is REVIEW-CLOSED, not implementation-ready

`mnemonic-toolkit` PR **#80** (`spec/wallet-id-enumerate-r0`, 11 commits).
Operator closed the review after the fourth lens: *"No more lenses after this
agent returns."*

Lenses run, each persisted verbatim in its OWN commit before its fold:

| lens | result | report in `design/agent-reports/` |
| --- | --- | --- |
| build gate (pre-review) | falsified 4 draft claims | — (commit `fd29eaa1`) |
| correctness/design (opus) | 1C/7I/7M/2N | `wallet-id-enumerate-spec-r0-round-1.md` |
| fold-check ×3 (sonnet) | 2 fold-introduced defects; clean; 1 partial | `…-fold-check-round-{1,2,3}.md` |
| implementability (opus) | 2C/7I | `wallet-id-enumerate-implementability-lens.md` |
| adversarial input (opus) | 3C/2I | `wallet-id-enumerate-adversarial-input-lens.md` |

**The last two lenses each found MORE Criticals than the first, and the round
before them came back CLEAN with the citation gate passing.** Lens-closure, not
finding-closure — exactly as the constellation rule says.

### Gate
`scripts/spec-citation-gate.sh` (committed): **45 assertions** — 39 `file:line`
citations re-resolved against source, 6 arithmetic recomputed. It caught a
fold-introduced citation **three times**, each in the fold that introduced it
(`verify_bundle.rs:951`→`946`, `permutation_search.rs:1071`→`1066`,
`restore.rs:1950`→`1951`). Run it before any future fold of this spec.

## TWO SHIPPED FUNDS-SAFETY DEFECTS FOUND — both filed in toolkit FOLLOWUPS.md

Neither caused by the spec. Both measured, then confirmed at source.

1. **`expect-wallet-id-silently-discarded-under-explicit-placement`** — a
   CORRECT `--expect-wallet-id` plus any `--cosigner @N=` emits a DIFFERENT
   wallet at **exit 0** under a `✓ wallet-id (completed)` line.
   `complete_explicit_assignment` (`restore.rs:1793`) takes no ctx, so the flag
   is structurally unreachable. **Fix before or with the implementation.**
2. **`sortedmulti-subset-search-never-enumerates-the-true-wallet`** — a correct
   full 16-byte id returns `✗ NO MATCH` on `--own-account-max` /
   `--search-cosigner-subset`. **GATES this cycle** (SPEC §3.9): the spec
   deletes the refusal that currently shields that operator, turning it into a
   confident false negative. §3.9 carves the combination out of enumerate mode
   until fixed, and requires a test pinning the carve-out.

## Operator rulings recorded in the spec
- **C1 / §2.1** — shown the measured lone-spurious risk (0.01% exact-pool,
  **36.69%** at `--own-account-max`) and three alternatives: *"Emit it, with a
  loud warning."* §3.8 makes it un-suppressible and present in BOTH stderr and
  `--json`.
- **§5** — `verify-bundle` keeps refusing; a verifier must not PASS on a list.

## Demo 3b — DONE and pushed (engrave `e910fd63`, CI green, no bypass)
- `--search-address` added as the way out of a short id (operator request).
- **Retracted "USE BOTH TOGETHER"** — `restore.rs:2005` is
  `if id_search {…} else if addr_search {…}`, so supplying both silently ignores
  the address. Filed as spec §3.7 (`conflicts_with`, on BOTH surfaces).
- Ladder corrected: the threshold for that 3-slot wallet is **10 hex**, not 16.
- Added the mk1 trap: the demo's bare-xpub `--cosigner` works only because that
  fixture has zero cosigner fingerprints; a real engraved wallet needs mk1 cards
  or it returns NO MATCH with a correct id.

## Resume here
1. **PR #80** — merge when `examples` + `clippy` + `test (ubuntu-latest)` pass
   (`sibling pins` / `g6 invariant` are the documented non-required reds).
2. **Fix the two shipped defects** (#1 above especially) — that is real code and
   needs its own TDD cycle + whole-diff review; it is NOT part of the spec PR.
3. Only then implement the spec, honouring §3.9's carve-out.
4. Untouched from the earlier anchor: v0.98.0 Linux musl gap; hardware-blocked
   F-583/584/585/591; MINISIGN_SECRET_KEY for md/ms/mk/mt.

## Note on repo sharing
A concurrent session was flashing SH2 boards #2 and #3 in this repo throughout.
Board #3 is burned and flashed (`b34b6829`). Demo commits were held unpushed
until that session's work was on origin, then pushed via
`scripts/push-via-staging.sh` — green, no bypass.
