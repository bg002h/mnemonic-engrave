# R3 (mechanical fold verification) — `IMPLEMENTATION_PLAN_f449_stage1a_internal_key.md`

**Scope: does the r2→r3 fold (`1712cc0e..704772d9`) address r2's C-A/I-A and
its 5 Minors/3 Nits, and does it leave anything stale or contradictory.** No
fresh audit, no design opinion. Diff is 35+/29- across two hunks only (verified
`git diff --stat`); commit message: "plan: stage 1a r3 — the M-4 test cannot
pass, and the gap it guarded is not one." Plan is 624 lines (was 618).

**Verdict: 0 findings unaddressed that block (8 non-blocking Minor/Nit items
left untouched by design-scope of this fold) / 2 stale items found (both
Minor, non-blocking) / 0 bad citations.**

**ready for implementation: yes**

---

## Q1 — did the fold address each of r2's findings?

r2's blocking pair (labelled generically C-1/I-1 in the dispatch brief; r2's
own labels were C-A/I-A) plus its 5 Minors (M-A..M-E) and 3 Nits (N-A..N-C):

| # | verdict | detail |
| --- | --- | --- |
| **C-1** (= r2 C-A: the M-4 non-zero-index test is unrunnable) | **ADDRESSED** | Step 1b (ll. 331–356) is rewritten to explicitly *not* add the test, with both of r2's measured reasons folded in verbatim: `PlaceholderNotReferenced` from the n=8/4-leaf fixture is gone (the whole fixture is gone), and the canonicalise-before-validate mechanism (`encode.rs:143-147`, `canonicalize.rs:12`/`:107`) is now the plan's own stated reason a non-zero `Slot` is canonically unreachable. `tr_descriptor_with_slot` and `a_non_zero_slot_index_survives_the_round_trip` are deleted; `grep` for both across the whole 624-line file returns nothing. Step 3's "Expected: **PASS**" (l. 391) now refers only to Step 1's original byte-equality test, which r2 verified passes — the false-PASS claim r2 flagged is gone with the test it described. |
| **I-1** (= r2 I-A: File Structure "not touched" list wrong on 3 of 10 files) | **ADDRESSED** | `render.rs`, `validate.rs`, `md-cli/src/parse/template.rs` are moved out of "Not touched" into a new "Touched by stage 1a" list (ll. 95–100); the "Not touched" list (ll. 102–103) now holds only the 7 files r2 measured as correct. See Q2 for a residual precision issue in the new list's own sourcing claim — non-blocking. |
| **M-A** (Step 2 dead python line + stray `cd` to the main checkout) | **NOT ADDRESSED** | Diff does not touch Task 1 Step 2 (ll. 383–399); the `cd /scratch/code/shibboleth/descriptor-mnemonic` and the commented-out python assertion are unchanged from r2. |
| **M-B** (`phase-gate.sh` "already carries every flag" overstates the dropped freebsd check) | **NOT ADDRESSED** | Step 8's phase-gate prose is untouched by this diff. |
| **M-C** (Global Constraints bullets 2–5 are pure stage-1b material) | **NOT ADDRESSED** | ll. 22–25 unchanged. |
| **M-D** (`pre_refactor_ids.json` has no reader in this plan) | **NOT ADDRESSED** | Not touched. |
| **M-E** (determinism unstated for `dump_encodings`/`dump_ids`) | **NOT ADDRESSED** | Not touched. |
| **N-A** (`cargo fmt --check` reformats the `include!` line as printed) | **NOT ADDRESSED** | Not touched. |
| **N-B** (`vendored.rs` cannot carry `#![allow(...)]` like `mod.rs` does) | **NOT ADDRESSED** | Not touched. |
| **N-C** (Step 9 `git add` scope omits `md-cli/tests` and `md-codec/examples`, measured harmless) | **NOT ADDRESSED** | Not touched. |

**8 of 10 items are untouched by this fold.** All 8 are Minor/Nit — per this
repo's severity rule, Minor/Nit "are recorded... but do not hold a gate." The
diff's own commit message scopes it to exactly the Critical and the Important
("the M-4 test cannot pass, and the gap it guarded is not one"), which matches
what the diff actually does: two hunks, one per blocking finding, nothing
else. This is the proportional-re-review convention working as designed, not
an omission — but the 8 open Minors/Nits should be filed as follow-ups (owning
phase: this plan, before/with Task 1 implementation) rather than silently
carried forward unlabelled.

---

## Q2 — staleness sweep (whole document, by topic word)

**Deleted test/helper (`a_non_zero_slot_index_survives_the_round_trip`,
`tr_descriptor_with_slot`): clean.** `grep -n` for both strings across the
full 624-line file returns nothing. The Self-Review's Placeholder scan (l.
611: "One helper is used and not defined by any step: `load_vendored_phrase`")
is now literally true again — before this fold, `tr_descriptor_with_slot` was
a second undefined helper r2 flagged as making that sentence false; it is gone
along with the test that used it.

**Finding 1 (Minor, stale) — the Status line was not bumped.** ll. 11–13 still
read:

> **Status:** r2, folded from the stage-1a R0 (2C/8I/8M/3N) and its re-review
> (1C/5I/7M/3N). Reports: `design/agent-reports/f449-plan-stage1a-{r0,r1}.md`.
> Awaiting re-review.

This text is byte-identical to the Status line at `1712cc0e` (verified:
`git show 1712cc0e:...md | sed -n '11,13p'` matches current `sed -n
'11,13p'` exactly) — it was never touched across this fold. The document is
now at r3 per its own commit message, having folded r2's report
(1C/1I/5M/3N), but the Status line still says "r2," cites only `{r0,r1}`, and
omits `f449-plan-stage1a-r2.md` entirely. A reader following only the Status
line would not know a third round happened. Non-blocking (self-description
only, no code/gate impact), but it is exactly the class of drift Q2 is
hunting for — recommend the next touch of this plan bump it to "r3, folded
from r2's re-review (1C/1I/5M/3N)... Reports: `{r0,r1,r2}.md`."

**Finding 2 (Minor, imprecise sourcing) — the new "Touched" list's own
citation overstates and underlists.** l. 95's sentence, "**Touched by stage
1a**... and named by Step 7's ruling table: `render.rs:194`,
`to_miniscript.rs:341`, `policy_shape.rs:250`, `md-cli/src/format/json.rs:353`,
`validate.rs`, `canonicalize.rs:115`, `md-cli/src/parse/template.rs`":
- `validate.rs` is **not** named anywhere in Step 7 (ll. 494–545 — checked by
  grep, zero hits for "validate" in that span). Its presence in the list is
  correct as fact (r2's I-A measured it directly against source at
  `validate.rs:83-100`), but the attribution to "Step 7's ruling table" is
  wrong for this one entry.
- Conversely, Step 7 **does** name two production sites that are absent from
  this "Touched" list: `parse/reuse.rs:515` (l. 542, one of the "four
  production sites [that] ship in THIS commit") and `compose/tr.rs:45` (l.
  507, the `is_nums: ik.is_none()` → `InternalKey::Slot(0)` rewrite). Both are
  real stage-1a production edits by the plan's own Step 7 text, neither
  appears in the l. 95–97 convenience list.

Not blocking — the generic top-of-table bullets ("the 47 other `md-codec/src`
production sites", "the 12 `md-cli/src` production sites", ll. 86–87) already
cover every file including these two, and the specific defect this list was
added to fix (I-A's 3 wrongly-"untouched" files) is fully closed. This is a
follow-on precision gap in a list whose own stated purpose ("named by Step
7's ruling table") is not quite what it delivers.

**"1a+1b" / combined-plan mentions: all historical, none stale.** The four
occurrences (ll. 58, 257, 362, 618) all describe the retired *combined* 1a+1b
plan that this plan was split from ("The combined 1a+1b plan reached 1300
lines...") — none claims the *current* document covers both stages.

**Task-number references: consistent.** `grep -n 'Task [0-9]'` returns only
Task 0 and Task 1 (the plan's actual two tasks) plus two retrospective
mentions of "Task 2" and "Task 8" (l. 250) that are explicitly framed as a
statement about r0's old numbering ("r0 scheduled this inside Task 8, reached
after Task 2 changes identity.rs"), matching r2's already-settled M-6. No
"Task 3"/"Task 6" anywhere.

**Counts internally consistent.** 47, 12, 38, 1, 29, 65, 23, 13 each recur at
every site checked with the same value — no drift between the File Structure
table, the blast-radius table, Task 0's golden assertions, and Task 1's Rust
gate (`g.count == 65`, `g.tr_count == 23`).

---

## Q3 — citations

All four resolved against `descriptor-mnemonic` at `6cbd49d8`. **0 bad.**

| citation | claim | result |
| --- | --- | --- |
| `encode.rs:143-147` | the canonicalise-then-validate sequence | **TRUE.** `:143` = `fn encode_payload_inner(...)`, `:145` = `canonicalize::canonicalize_placeholder_indices(&mut d_canonical)?`, `:147` = `validate::validate_placeholder_usage(&d.tree, d.n)?`. Exact. |
| `canonicalize.rs:12`, `:107` | concern `Tr.key_index` | **TRUE.** `:12` is the module doc line "the tree's `KeyArg.index` and `Tr.key_index` fields;". `:107` is the `Body::Tr { is_nums, key_index, tree } =>` match arm inside `remap_indices`, whose body (`:114`) applies `*key_index = perm[...]`. Both concern `Tr.key_index`. |
| `tree.rs:559` | a non-zero-index unit test at the raw layer | **TRUE.** `fn tr_is_nums_false_round_trip()` starts at `:559`, builds `Body::Tr { is_nums: false, key_index: 2, tree: None }`, round-trips through `write_node`/`read_node` directly (not through `encode_payload`), so it runs below canonicalisation. |
| "a root `Tr`'s internal key is renumbered to `Slot(0)` on every encode" | claim, tried to construct a counterexample | **TRUE, no counterexample found.** `canonicalize_placeholder_indices` calls `walk_collect_first(&d.tree, ...)` starting at the descriptor's root node (`canonicalize.rs:178`). `walk_collect_first`'s `Body::Tr` arm (`canonicalize.rs:56-69`) registers the Tr's own `key_index` in `first_occurrences` *before* recursing into its optional tapscript `tree` (`:72`). `perm[old_idx] = new_idx` is assigned by position in `first_occurrences` (`:194-195`), so whatever is registered first gets `new_idx = 0`. Root-only is enforced elsewhere, not assumed: `is_forbidden_leaf_tag` (`validate.rs:274-279`) forbids `Tag::Tr` from appearing as a tapscript leaf, and `validate_tap_script_tree` runs inside `encode_payload_inner` whenever the descriptor's tag is `Tr` — so a `Body::Tr` can only ever be the descriptor's root, never nested. Tried: multiple placeholders before the Tr in a hypothetical non-root position (blocked by the leaf-tag forbid), and `is_nums = true` roots (out of scope — no `key_index` to renumber, not what the claim is about). No construction survives. |

---

## Build-gate coverage note

No standalone gate script applies to this plan (`scripts/plan-build-gate.sh`
in this repo targets an unrelated "seal" plan). The diff introduces no new
freestanding Rust to compile — the two hunks (a prose rewrite of the File
Structure table, and Step 1b's replacement of a test with an explanation) cite
existing source rather than adding new code blocks, so Q3's line-by-line
resolution against real `descriptor-mnemonic` source at `6cbd49d8` **is** the
machine check for this fold; nothing here is un-gated prose beyond what Q3
covers.

---

## Verdict

The fold did exactly what its commit message says: closed the one Critical
(an unrunnable, wrongly-diagnosed test) and the one Important (a File
Structure table lying about what stage 1a touches), and touched nothing else.
Both fixes hold up under direct source verification, including a deliberate
attempt to break the "renumbered to Slot(0) on every encode" claim. The 8
Minor/Nit items from r2 are untouched — correct under this repo's proportional
re-review rule, since none of them blocks, but they should be filed as
follow-ups rather than dropped. Two new non-blocking staleness items surfaced
in this sweep: the Status line was not bumped past "r2," and the new
"Touched" list slightly misattributes its own sourcing (one file wrongly
credited to Step 7, two files Step 7 actually names left off the list).
Neither is Critical or Important by this repo's severity rules.

**ready for implementation: yes**
