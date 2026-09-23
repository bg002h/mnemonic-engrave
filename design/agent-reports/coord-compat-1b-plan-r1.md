# R1 re-review: coordinator-compat plan 1b, fold of R0 (0bbfc5a7..HEAD)

**Independent of the fold's author.** Scope: `git -C mnemonic-engrave diff 0bbfc5a7 HEAD`
(the plan, the design, and `scripts/policy-differential.py`). Question: did the fold
close each R0 finding (I-1, I-2, M-1..M-4, N-1), and did the fold's new text introduce
a Critical or Important defect. Everything below was executed, not read, on a scratch
assembly of descriptor-mnemonic `d269c556` at `/scratch/code/shibboleth/.tmp/cc1b-extracted`
via `scripts/cc1b-plan-extract.py`, rebuilt fresh from the CURRENT (post-fold) plan text
(not reused from the author's own gate artifacts). Toolchain:
`PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH`. No real
checkout was modified (`git status --short` clean in both `mnemonic-engrave` and
`descriptor-mnemonic` after the session).

## Per-finding table

| Finding | Verdict | Evidence |
|---|---|---|
| **I-1** Liana class 9 false-refuses a Liana-imported 1-of-n second path | **CLOSED** | `md compose --wrapper wsh --path 2of2 --path 1of2 --path 1of1,older=100` now prints `Liana 8.0-15.0: unproven (a template has no keys, so no import can be claimed)` — the false refusal is gone (was: `refuses (a second unlocked multi-key path)`). The `tr --unspendable liana` spelling likewise prints `unproven`. Constructed a `k>=2` case Liana v15.0 actually refuses (`--path 2of2 --path 2of2 --path 1of1,older=100`, and separately `--path 1of1 --path 2of2 --path 1of1,older=100`, matching R0's `cx-single-then-multi` shape): both print `Liana 8.0-15.0: refuses (a second unlocked k-of-n path with k >= 2)`, matching R0's ground truth (`{"error":"Descriptor is not compatible with a Liana spending policy.","name":"cx-single-then-multi","ok":false}` in `.tmp/r0-cc1b/out2.jsonl`) and R0's own controls exactly. |
| **M-1** Liana relative-lock clause missed `older()` values Liana refuses as blocks | **CLOSED** | `md compose`'s own path builder refuses `older` > 65535 at the CLI layer (`older in blocks needs 1..=65535, got 70000`), so the rule can only be reached with raw descriptor text (as a minted-elsewhere card would arrive). Wrote a throwaway example (`crates/md-codec/examples/r1probe.rs`, scratch-only, removed after use) driving `descriptor_from_text → skeleton → coordinator::verdicts`. `older(70000)` (R0's exact `x24-older70000` descriptor) → `Liana 8.0-15.0: refuses (a relative lock Liana cannot use)`, matching R0's measured Liana message exactly. `older(65535)` (same descriptor, value substituted) → `Liana 8.0-15.0: imports the multipath form, but reads it as Liana 2-of-4 …` (does NOT refuse). Also checked the boundary one past: `older(65536)` → refuses, confirming the `> 0xFFFF` predicate is exactly where the plan claims. |
| **I-2** Fork oracle test breaks once md 0.20.0 is installed | **CLOSED (records)** | F-669 is drafted in Task 0 Step 2 (plan lines 262-271) naming the exact test (`TestFableTwoKeylessPathsAgreeWithTheHostOracle`, `gui/composer_fable_r0_funds_test.go:131-160` at `2c9eed3`), the fix (`--md-only`), and an explicit owning phase ("before md-cli 0.20.0 is installed on this box, i.e. plan 1b's release (Task 9)"). The release task (line 5094) adds a standalone hold: "Do NOT `cargo install` md 0.20.0 on this box until F-669's fork fix … has landed," placed between the release commit (Step 3) and the PR (Step 4) — correctly ordered and non-dangling. |
| **M-2** `policy-differential.py` misreports keyless-`wsh` refusals as findings | **CLOSED** | Diff confirms the retry-on-`--md-only` block was added to `Runner.compose()` (mirrors the existing `--experimental` retry pattern exactly), and Task 0 Step 2 instructs committing this working-tree edit in the records commit. |
| **M-3** ms-cli's printed recipe needs an undocumented flag | **CLOSED (records)** | F-670 drafted in the same Task 0 Step 2 block (plan lines 271-276), naming the file/line (`crates/ms-cli/src/cmd/hashlock.rs:525`), the fix (`--experimental --md-only`), classified documentation-only, owning phase "next mnemonic-secret release." |
| **M-4** Clauses declared `reads: S` but read lock values | **CLOSED** | `const SV: ReadSet = ReadSet::STRUCTURE.with(ReadSet::LOCK_VALUES);` added; both the relative-lock clause and "two paths with one lock" now declare `reads: SV` (confirmed by diff and by the extracted `registry.rs`). |
| **N-1** `--check` doc implies it's the gate | **CLOSED** | xtask module doc now reads "`--check` … a local convenience. THE GATE is `table_is_fresh` below … CI never calls `--check`, and no test exercises `main`." |

## New mutations re-applied (asked for one; re-applied three)

All three, applied to the scratch assembly, reverted cleanly afterward (byte-identical to
the staged reference), each confirmed to redden exactly as the fold's new text claims:

1. `registry.rs` class-9 clause `b.k >= 2` → `b.k >= 1`: `liana_class_9_refuses_a_second_threshold_only_from_k_2` **FAILS** (`cx-multi-then-1of2multi` now wrongly refused). `cargo test -p xtask table_is_fresh` also reds, with **`D1 false-refusal coord-compat-1b/liana-r0-probes-out.jsonl:1`** — exactly the disagreement class the test's own doc comment claims.
2. Same clause, `b.k >= 2` → `b.k >= 3`: same test **FAILS** (`cx-single-then-multi` no longer refused). `table_is_fresh` reds with 5 **`D2 missed-refusal`** disagreements (X25/X26 rows in `fable-liana-parse-out{,-v15}.jsonl`) — matches "the build reports D2s on X25/X26."
3. Relative-lock clause: dropped the `|| (l.kind == LockKind::OlderBlocks && l.value > 0xFFFF)` disjunct: `cargo test -p xtask table_is_fresh` **FAILS** with **`D2 missed-refusal coord-compat-1b/liana-r0-probes-out.jsonl:5: … Timelock value '70000' isn't valid or safe to use`** — the exact line/message the plan cites for this mutation.

## Independently re-derived facts (not taken on the author's word)

- `cargo xtask verdicts` on the fresh assembly: **532 rows, 230 cells stored, 206 refusals explained, 2 unkeyable** — matches the plan's claim exactly.
- `cargo nextest run --workspace --locked --all-features`: **1556 passed, 4 skipped** — matches the plan's claimed final task-boundary count exactly (my first attempt, without `--locked --all-features`, only found 1518 — the flags matter; re-ran with the plan's own gate invocation and it matched).
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --all-features --locked`: rc 0.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: rc 0.
- `cargo fmt --check`: rc 0.
- `scripts/cc1b-plan-extract.py`, re-run fresh against the current (post-fold) plan text: all 9 task boundaries build and match the staged reference (T1..T9 IDENTICAL).

## New findings

None. No Critical or Important defect found in the fold's new text within the diff scope
(I-1's registry clause, M-1's widened clause and `SV` ReadSet, the two new F-numbers and
their owning phases, the release-task hold, `policy-differential.py`'s retry, and the new
boundary test). All citations the fold added (file:line, disagreement classes, message
text, evidence-row names) were re-grepped/re-executed and hold.

ready for implementation: yes
