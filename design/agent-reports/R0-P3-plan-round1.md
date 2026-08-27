# R0 round 1 — fold-check on `IMPLEMENTATION_PLAN_P3_md_mk_mnemonic.md`

**Reviewer:** independent agent, fold-check only (not a fresh audit).
**Date:** 2026-08-27. **Artifact:** `design/IMPLEMENTATION_PLAN_P3_md_mk_mnemonic.md`
@ `495fc0b`, worktree `/scratch/code/shibboleth/_work/revp3r1/mnemonic-engrave`,
branch `review/p3r1`.

**Scope, as briefed:** did the fold fix each round-0 finding, and did it
introduce a new defect. Not a fresh audit. Settled facts from the brief (gate
exit codes, the exit-code census, the two `cli_repair.rs` tests, the mutation
result) were not re-derived, per instruction — except where independently
re-run below because the brief asked for it.

## Per-finding disposition

- **I-1** (exit-code census missed 2/3 of assertions; two missed sites pin the
  changed arm) — **LANDED.** Entry 11 now carries the three-family table (57/34
  totals, 13/6 asserting 2), names both `cli_repair.rs` tests by function and
  line, and rules the bypass width from `md`'s actual source rather than the
  draft's "uncorrectable" gloss. Verified by independent mutation re-run is
  covered by the brief's SETTLED section (335/2 fail naive, 337/0 with the
  correctly-scoped bypass); not re-run here per instruction.
- **I-2** (`"zero GUI test failures"` false; no step could make it true) —
  **LANDED, and independently re-run.** I copied `mnemonic-gui` to scratch,
  flipped the same 4 `--group-size` sites in `src/schema/mnemonic.rs` (332,
  1281, 1960, 2050 — confirmed by reading context first, all four are the
  `--group-size` help block, not incidental `"5"` matches), and ran
  `cargo test --test schema_mirror_defaults_drift` with `MNEMONIC_BIN` pointed
  at the installed `mnemonic 0.97.0` (matches both pins:
  `pinned-upstream.toml:22` and `Cargo.toml:76`, both
  `mnemonic-toolkit-v0.97.0`, confirmed by reading the files directly).
  Result: `mnemonic_defaults_and_choices_match_pinned_gui_schema` **FAILED**
  with exactly the four lines the plan and fold both cite (`bundle`, `convert`,
  `ms-shares-split`, `ms-shares-combine`, `mirror=Some("0") gui-schema=Some("5")`);
  `md_ms_mk_choices_and_defaults_match_pinned_gui_schema` stayed **ok** in the
  same run. Byte-for-byte match to the plan's claimed output. New entry 16
  (toolkit release + pin bump) precedes entry 17 (GUI mirror) in table order,
  as the finding required. Subject repo confirmed untouched afterward
  (`git status --short` in `/scratch/code/shibboleth/mnemonic-gui`: clean).
- **I-3** (23 CI-byte-compared goldens invisible to the plan's green) —
  **LANDED.** Confirmed the 19-and-4 split is inside the operative document,
  not only in prose: entry 13's gate column carries the 4-golden regeneration
  requirement, entry 14's gate column carries the 19-transcript rewrite
  requirement (and states the correct order — rewrite first, then regenerate),
  and closure conditions 14 and 16 both name the toolkit's doc-transcript
  workflows as a fourth validation surface. §0's join table adds this as its
  own row with an owner (`the mnemonic branch, inside it, before that branch
  calls itself green`).
- **I-4** (`bytecode` in the work column, absent from the gate; `verify`
  baseline wrong) — **LANDED.** Entry 5's gate now asserts all four verbs by
  name, states `md bytecode -` reproduces the identical defect, and states the
  `--template` requirement for `verify`'s gate. Closure condition 4 lists all
  four verbs and the `--template` caveat. No stray "three verbs" text remains
  anywhere in the document (grepped).
- **M-1** (E0425 paragraph omitted the `write` module) — **LANDED.** §3 now
  names two traps, `write::write_private` (the one `md`/`mk` both adopt) ahead
  of the `remedy` one, and entry 1's gate cell asserts the qualified path
  explicitly.
- **M-2** (F-293 named 2 of 4 trailing-space sites, wrong residue) —
  **LANDED.** `FOLLOWUPS.md:12709` (F-293) and the plan's own §7 both now name
  four sites across two flags and state residue 44, matching the fold's
  re-derivation from the correct argument position.
- **M-3** (`"--in`/`--out` on no verb"` wider than its search) — **LANDED.**
  Both `md`'s and `mk`'s channel prose now name the `vectors --out`/`gen-man
  --out` directory collision, and §6 (out of scope) adds an explicit entry for
  it so a later reader does not "tidy" the two meanings together.
- **M-4** (entry 9's card-boundary justification falsified) — **LANDED.**
  Entry 9's work column states the justification is "FALSE and is withdrawn,"
  the gate cell reproduces the duplicate-record counterexample, and F-311 is
  filed (confirmed present in `FOLLOWUPS.md:12989`, owning phase correctly
  "NOT P3").
- **M-5** (entry 2's control cannot catch header deletion) — **LANDED.** The
  cell now states the control asserts absence only, and explicitly flags the
  stderr assertion as the load-bearing half rather than belt-and-braces.
- **M-6** (entry 1's RED baseline went green mid-review) — **LANDED, and
  handled correctly rather than just noted.** The gate cell states the
  prerequisite conjunct "can no longer fail and is recorded as SATISFIED, not
  asserted," with the remaining live conjuncts named separately — the document
  itself states the principle ("a gate that can no longer fail is not a gate").
- **N-1** (fixture-dependent line count) — **LANDED.** Entry 9 now asserts the
  fixture-independent shape (every line starts `mk1`, blank count 0) and
  states the count is deliberately not asserted.
- **N-2** (`chunk-set-id:` design-doc count wrong and drifting) — **LANDED as
  a documented drift, not a pin.** The plan now states the number is
  self-referential and explicitly declines to gate on it. Re-run today in this
  worktree: `git ls-files design | xargs grep -l 'chunk-set-id:' | wc -l` →
  **37**, one more than the fold's 36 — expected under the plan's own
  "rises every time one is written" framing (this round's own report file
  will raise it again), not a defect. The actionable 7-file journey set is
  unaffected and not restated as wrong by the fold.
- **N-3** (no closure condition checks that follow-ups landed) —
  **LANDED.** Closure condition 19 now asserts the filed follow-ups reached
  `FOLLOWUPS.md`, and F-296, F-311, F-312, F-313 are all present in the file
  at the citations the plan and fold report give.

**13 of 13 confirmed LANDED.** No PARTIAL, NOT LANDED, or WRONGLY FIXED.

## New defects introduced by the fold

**None found.** Checks specifically run against the "incomplete propagation"
failure mode this cycle has hit three times running:

- Grepped for the superseded shapes the fold's changes should have retired:
  `"far behind"`, `"zero GUI test failures"`, `"the ONLY .code(2)"`,
  `"12 sites"`, `"histogram of"`, `"5 lines, 1 of them blank"`,
  `"recoverable from each card"`, `"one serialised prerequisite"`,
  `"two real collision points"`, `"FIVE PIECES OF WORK"`, `"three verbs"`,
  `"v0.75.0"`. Every hit that remains is inside a clause explicitly quoting
  and superseding the first draft's claim (e.g. `*"a pin far behind..."*.
  FALSE`) — none stands as an assertion of present fact. Zero orphaned stale
  claims found.
- Checked the widened bypass ruling ("any codec error out of the correcting
  decode," not "uncorrectable") is consistent everywhere `mk repair`'s exit
  code is discussed: §1.2's mk-surface section, entry 11's work and gate
  cells, and closure condition 2 all state the same width. No remaining text
  implies the narrower reading.
- Re-ran all three plan gates independently rather than trusting the fold's
  table: `plan-table-check.sh` → **0**, `plan-cite-check.sh` → **0**,
  `plan-stepref-check.sh` → **0**. Matches the brief's settled figures.
- Checked `FOLLOWUPS.md` directly for every follow-up the fold claims to have
  filed or closed (F-296 DONE, F-311/F-312/F-313 new, F-293 amended) — all
  five entries exist at the stated repo/owning-phase, not merely claimed in
  the report.
- Table renumbering (19 rows → 20 rows, entries 16–19 → 17–20) does not
  falsify any prose reference, because the document's own rule is that prose
  names work by NAME rather than number (verified: `plan-stepref-check.sh`
  passing is exactly the mechanical check for this, and it is green).
- Spot-checked the fold's own admitted gap — the three items it explicitly
  did **not** re-derive:
  - **§2 (THE BOUNDARY)** — byte-for-byte untouched by the fold's diff (not in
    the `d5a6c45..495fc0b` diff at all); read it directly and found it
    internally consistent with the rest of the document (the `write_private`
    verdict row matches entries 6/10's channel work).
  - **§10's acceptance (table entry 19, formerly 18)** — text is byte-identical
    across the diff (only the row number changed), so nothing to re-derive.
  - **`--from-md1-set` (entry 12)** — untouched by the diff; content matches
    round 0's own "checked and found CLEAN" list.

  All three hold. No defect in any of them.

## Verdict on the serialisation

**Binding in the plan itself, not only in the fold's report.** §0 carries the
join table (join / why / owner columns) and the ASCII order diagram; §4's
table caption states the operative rule directly — *"the join entries wait on
all of them in table order — the joins are sequential among themselves, not a
set"* — which is machine-adjacent (row order in the one table of record) rather
than prose a parallel implementer could miss. Entry 17's gate cell also
cross-references entry 16 explicitly (*"which is only achievable after the
release-and-pin entry above"*), so the dependency is stated twice, in two
different places, in the artifact a builder actually reads.

## Counts

**0C / 0I / 0M / 0Nit.**

This is a clean round: 13 of 13 round-0 findings landed as claimed, all four
of the brief's specifically-flagged load-bearing claims verified (one by
independent re-run reproducing the exact counterexample), the fold's own
admitted gap spot-checked clean, and no new defect — mechanical or
propagation — found anywhere in the diff.

## Method

- `mnemonic-gui` copied to scratch before any edit; original confirmed clean
  (`git status --short`) after the run. No subject repo written to.
- I-2's re-run used the pinned binary already installed at
  `/home/bcg/.cargo/bin/mnemonic` (`mnemonic 0.97.0`), matching both cited
  pins exactly — not a stand-in build.
- All three plan gates run as separate commands, exit codes read directly.
- `FOLLOWUPS.md` entries checked by grep against the exact IDs cited, not
  assumed present from the report's say-so.
- Wrote to no file in this worktree except this report and its own commit.

**Scope of my negatives.** "No superseded-shape remnants" covers the phrase
list above via `grep -n`, not a full manual re-read of every paragraph.
"§2/§10/entry-12 hold" covers a direct read for internal consistency, not a
re-verification of their original citations (round 0 already did that and the
fold did not touch them).
