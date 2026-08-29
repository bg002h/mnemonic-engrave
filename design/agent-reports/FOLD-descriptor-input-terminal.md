# Terminal fold — descriptor-input spec: gate-to-vectors refactor + r18/r19 residue

**Target:** `design/SPEC_descriptor_input.md`, edited in place at the tip after
the r19 persist (`37536f4`). One file changed, **+190/−54** (`git diff
--numstat`), 1707 → 1843 lines. Nothing committed, nothing pushed — per the
brief, a separate reviewer verifies this fold; this report does not certify it
GREEN.

**Sources folded, read in full:**
`design/agent-reports/R0-descriptor-input-spec-r18.md` (its new-M1, new-M2,
new-N1 — unfolded at r19) and `design/agent-reports/R0-descriptor-input-spec-r19.md`
(its r19-M1, r19-M2, and the §11-item-4 mnemonic-first plan note).

---

## Part 1 — the gate-to-vectors refactor

**Why.** Rounds 15–19 produced eight findings against §5.1's discriminator and
§6's cause selection, every one a prose-precision defect in a disjunct, a
scope, or a mirror. The fix relocates the precision into the executable
artifact the constellation already trusts: §7's shared vectors.

### §5.1 (the discriminator bullet, rewritten in four blocks)

1. **Intent + invariant, NORMATIVE, stated in a few sentences.**
   Invariant 1: no record-shaped input ever hears descriptor vocabulary or a
   changed exit code — pinned by the record-refusal test
   (`crates/me-cli/tests/sysw_cli.rs:1928`, cite-gate: resolves) and bounded by
   the classifier's six admitted shapes
   (`crates/me-cli/src/sysw/mod.rs:211` = `pub fn classify_with`, cite-gate:
   resolves; r19 measured that none of the six begins `identifier(`).
   Invariant 2: every admitted descriptor spelling — §4's four formats, all
   fifteen §4.5 rows, conjunct 7's five use-site spellings, buried-in-multi-
   record placements — reaches the descriptor surfaces (the "--as decides"
   block, a §6 row, or the multi-record row).
2. **The precision clause:** §7's `gate`-tagged rows are NORMATIVE; where any
   reading of the guidance disagrees with a gate row, the row wins.
3. **The four shape tests demoted to explicitly NON-normative implementation
   guidance**, kept verbatim in substance, with two residue fixes folded in
   at their new home (r19-M1 and r18-M2, below).
4. **The routing tail restated as three explicit cases** (parses whole /
   splits as records / neither), the third being new: when the gate opens and
   NEITHER parse succeeds, §6's whole-input cause selection chooses the
   refusal — the honest statement r19-M2 demanded, replacing the deleted
   "(aligned with §6's cause-selection steps)" claim.

### §6 (cause selection)

- New **Scope** paragraph after the five-step rule: gate is per-LINE, cause
  selection is whole-INPUT, a *deliberate divergence, not an alignment*
  (r19-M2). The buried-key case gets its outcome explicitly: a records file
  whose only descriptor-shaped line is a bare `Zpub…`/`tpub…`/`[fp]xpub…`-
  no-path opens the gate, fires no step 1–4, and lands on **step 5's generic
  four-forms text at exit 3**; the dedicated rows stay reachable for the key
  alone; whether cause selection should follow the opening LINE is named as
  a plan-phase design call (r19's own ruling).
- Step 4's parenthetical no longer says "mirrored from §5.1's gate" (which
  now names guidance): it says "the same leading-segment test as §5.1's gate
  guidance, r18's new-I2". No test content changed.

### §7 (vectors)

- **Row schema:** new bullet defining the four gate fields, REQUIRED on
  `gate`-tagged rows and absent elsewhere: `gate_open` (bool), `outcome`
  (`record-refusal` | `as-decides` | `descriptor-refusal` | `multi-record`),
  `refusal_row` (slug of the §6 row, on the two §6 outcomes), `exit_code`
  (2/3/4). Asserted by the Rust test only; the Go test ignores them.
- **New required-row bullet** with seven clauses, each an adversarial class
  rounds 16–19 built: (1) the fifteen §4.5 rows under `--as` omitted
  (as-decides/exit 2 on rows 1,2,5,6,7,13,14 — seven; descriptor-refusal/
  exit 3 on rows 3,4,8,9,10,11,12,15 — eight); (2) six hostile-payload
  records incl. a `text:` record carrying a real xpub token (the brief's
  "base58-ish" class); (3) a mistyped bare mnemonic; (4) four malformed
  bech32 strings, one per record class md1/mk1/ms1/mt1; (5) the multi-record
  split MNEMONIC FIRST, with the r19 plan note (descriptor-first witnesses
  nothing) stated inline; (6) the three buried refused keys → step-5 generic
  text, exit 3; (7) three edge tokens — 77-byte payload, `[` alone,
  trailing-slash `xpub…/`.
- **Manifest:** new `gate` tag, min **33**. Overlap rule generalised from
  "exactly two rows" to the named set: the fifteen §4.5 rows carry `gate` as
  a second tag; the original pair (`xpub…\n`, bare-`xpub`) now carry three
  tags. The `covers` schema bullet updated to match ("additional tags …
  only for the rows the manifest names").

### The manifest arithmetic, recomputed from scratch (not inherited)

```
minima:  4 + 15 + 14 + 1 + 5 + 3 + 3 + 6 + 33 = 84 tag-slots
gate min: 15 + 6 + 1 + 4 + 1 + 3 + 3          = 33  (the seven clauses)
overlap slots: 15 (gate on the 15 §4.5 rows)
             +  2 (whitespace on xpub…\n; formats-happy on bare-xpub) = 17
floor:   84 − 17 = 67 physical rows
cross-check by the other route: 49 (r19-verified floor)
             + 18 new gate-only physical rows (6+1+4+1+3+3) = 67  ✓
```

All four numbers (84, 33, 17, 67) were also machine-summed from the edited
file by script (`sweeps.py`): table minima sum = **84** over **9** tags,
stated slots = 84, stated floor = 67, gate min = 33 = clause sum. CONSISTENT
on every axis.

## Part 2 — the residue, folded from the full report texts

- **r18 new-M1** (§5.3's closing absolute): "No refusal **names** a flag…"
  with a neither-path-only exception → "No refusal **points the operator at**
  a flag that refuses in the current build; **any** refusal may DESCRIBE a
  path's future availability — describing routes nothing." The principle is
  now unconditional, exactly the repair r18 proposed; the clause records that
  the stock replacement and the window variant are descriptions, not routes.
  No refusal text changed.
- **r18 new-M2** ("identifier" undefined): defined in the T1 guidance as
  `[A-Za-z][A-Za-z0-9_]*`, with the two decisions r18 asked to be seen made:
  underscore included (bare `or_d(…)`/`multi_a(…)` reach §6's miniscript
  row) and `v:pkh(…)` failing on every reading.
- **r18 new-N1 / §9 item 7**: rewritten. It no longer claims §6 "has not
  been walked" — the walk RAN, the walk lens closed at r19; what item 7 now
  records is only that the rows added/reworded after the walk were covered
  by closure rounds as text, and a row-by-row walk of the final table is a
  plan-phase option, not a gate.
- **r19-M1** ("uniform per-line scope" over-claims): the umbrella now reads
  "the first three applied to EVERY line … the JSON test to the WHOLE input
  (per-line except JSON)". The word "uniform" is gone from the file (grep:
  0 hits).
- **r19-M2**: the §6 Scope paragraph and §5.1's third routing case, above.
- **r19 plan note**: §11 item 4 now requires the multi-record row's test to
  use the MNEMONIC-FIRST ordering, with the counterexample rationale.
- **Status header**: GREEN re-closed at r19 (0C/0I, walk lens complete,
  60/60 decision table), r1…r19 lineage, and — explicitly — that this
  terminal fold awaits its own verification round: "the GREEN above is
  r19's, not the fold's".

## Sweep and gate results

| check | method | result |
| --- | --- | --- |
| W5 quoted-span sweep | whitespace-flattened file, all `*"…"*` spans extracted, matched against the r19 pattern widened with `terminal fold`, `gate bullet`, `clause \d`, `invariant \d` | **45 spans, 0 violations** — count identical to r16–r19; the fold added no operator-visible quoted spans |
| citation gate | `./scripts/plan-cite-gate.sh design/SPEC_descriptor_input.md` | **exactly the 5 known cross-repo Rust failures** (`md-codec/src/tlv.rs:10`, `src/encode.rs:17`, `src/tlv.rs:24`, `src/use_site_path.rs:43`, `src/use_site_path.rs:49` — all in the descriptor-mnemonic repo, hand-verified in prior rounds). Both citations this fold leans on resolve: `crates/me-cli/src/sysw/mod.rs:211` → `pub fn classify_with`, `sysw_cli.rs:1928` → the record-refusal test fn |
| superseded-phrasing sweep | grep for `uniform per-line`, `aligned with §6`, `49 physical`, `51 tag`, `the two rows the manifest names`, `have not been walked`, `round 8 (2026`, `by these tests` | **0 stale hits** (the one `second tag` hit is this fold's own new text) |
| manifest arithmetic | script-summed from the edited file | consistent, shown above |
| diff containment | `git diff --numstat` | **1 file, +190/−54** — the spec only |

## What I deliberately did NOT do

- **No decision-table cell changed.** The refactor relocates precision; every
  outcome stated in the new text (the 7/8 split of §4.5 under `--as` omitted,
  the buried-key step-5 outcome, MREC mnemonic-first, the edge tokens) is
  transcribed from r18's and r19's derivations of the settled 60-cell table,
  not re-derived differently. I found no cell I believe wrong.
- **No commit, no push** — the brief reserves those for the controller.
- **No gate row for a bare `or_d(…)` line.** r18-M2's prescribed fix was one
  clause naming the character class, which is what was folded; the input is
  outside the brief's seven enumerated classes. If the reviewer wants it
  pinned executably, it is one added row and a +1 on `gate`'s minimum
  (33→34, floor 67→68).
- **No `deadbeef: xpub…` gate row** (the r17-intended T2 flip, r18 table (c)
  last row): same reason — not in the brief's enumeration. Same one-row
  remedy available.
- **§6's five-step cause-selection rule remains NORMATIVE prose**, per the
  mandate ("they order diagnostics, a different job") — only its false
  alignment claim was removed and its scope stated honestly.
- **Did not re-run the `me 0.7.0` invocations or the base58check
  constructions** — every behavioural number in the new text is carried with
  its provenance from the r18/r19 reports, which measured them at the same
  tree; this fold changed no behaviour to invalidate them.
- **Did not touch** `SPEC_systemwide_payloads`, FOLLOWUPS, or any other file.

## For the verifying reviewer

The three questions this fold should be checked on: (1) does the
intent+invariant pair, plus the 33 gate rows, pin everything the four prose
tests pinned (the demotion loses nothing)? (2) is the 84/17/67 arithmetic
right (recompute, do not inherit — shown above from two directions)?
(3) did any relocated sentence change a settled outcome (the 60-cell table
is the acceptance witness)?
