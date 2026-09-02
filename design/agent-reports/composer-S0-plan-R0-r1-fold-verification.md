# Fold verification — composer S0 plan, round 0 → fold

**Artifact:** `design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md` (+ one paragraph of `design/STAGED_PLAN_wallet_policy_composer.md`).
**BEFORE:** `3a799fa`. **AFTER / fold commit:** `891b17d` (= working tree).
**Responds to:** `design/agent-reports/composer-S0-plan-R0-r0-fidelity.md` (opus, persisted `b820b64`, 0C/4I/5M/3N) and `design/agent-reports/composer-S0-plan-R0-r0-tests.md` (sonnet, persisted `f531cff`, 0C/2I/4M/2N).
**Lens:** mechanical fold verification — for each of the 16 findings, did the fold's own new text fix the defect AS THE FINDING STATED IT (not as its suggested remedy stated it), and does the fold's new text introduce anything new that is wrong. Read-only on every repo; no `.jsonl` file read; no file written outside the one report path.

**What I ran:** `git show`/`git diff 3a799fa..891b17d -- design/` to extract BEFORE/AFTER text and the exact hunks; a Python AST-style parser (regex + bracket-balanced splitter, not hand-counting) over Task 5's `family()` to extract all 28 entries' tags and path-list structure and cross-tabulate them against the required-tag list and the fp-tag rules; `grep`/`git show` against `descriptor-mnemonic@3b0944fb` for the two cited `file:line` facts. Compile/test-pass state at the fold commit is taken as already machine-checked per the brief (52 compose tests, 51 pass, 1 pinned red; clippy clean; md-cli compiles) and is not re-derived here.

## VERDICT: 15 FIXED / 1 FIXED-WITH-CAVEAT / 0 PARTIAL / 0 NOT FIXED / 0 DECLINED — 0 regressions, 1 new defect (Minor, in the fold's own new text)

All 16 findings across both reports are fixed as their own text stated the defect. One fix (fidelity I-3, `decaying_multisig`) closes the concrete defect the finding demonstrated but leaves a new, narrower gap in the same function that the finding did not raise — recorded below as a new defect, not as a failure to fix I-3. No superseded phrasing survives anywhere I grepped for it.

## Per-finding table

| id | one-line title | fold's response (AFTER text, brief quote) | verdict |
|---|---|---|---|
| fidelity I-1 | required-tag list collapses 2 of 3 spec fingerprint cases; no origins-per-wrapper tag | `family()` doc comment now names "the spec's three: `fp:distinct`... `fp:one-seed-one-path`... `fp:one-seed-two-paths`"; `origins:default-<wrapper>` added; both are in the `every_tag_appears_in_at_least_two_vectors` required list (line ~1950) | FIXED |
| fidelity I-2 | §10 item 3 preset templates undelivered; preset tests can't fail on wrong archetype shape | new test `presets_lower_to_their_pinned_templates` pins 6 literal expected templates, one per preset, "so a preset that drifts in SHAPE... fails here" | FIXED |
| fidelity I-3 | `decaying_multisig` forces both tiers to the same k-of-n; can't express its own archetype | signature is now `decaying_multisig(wrapper, k1, n1, k2, n2, older1, older2, after_height)`; refuses `older2 <= older1` and `k2 > k1` via new `ComposeError::PresetShape` | FIXED (see new-defect note below) |
| fidelity I-4 | §5b cross-check runs 3 hand-picked shapes, never the 28-vector family | `cross_check()` extracted as one reusable fn; `every_family_entry_passes_the_5b_cross_check` (Task 5) runs it over all 28, `every_preset_passes_the_5b_cross_check` (Task 7) over all 6 presets; `concrete_policy()` built from the PATH LIST | FIXED |
| fidelity M-1 | `main.rs` fragment names `Commands`; enum is `Command` | every occurrence reads `Command`/`Command::Compose`; cites `crates/md-cli/src/main.rs:96` | FIXED |
| fidelity M-2 | `blocks()` refusal always names path 0, regardless of which tier is bad | `fn blocks(b: u32, path: usize)`; every preset call site passes its own tier's index; test pins `"path 2: older in blocks needs 1..=65535"` for a tier-2 defect | FIXED |
| fidelity M-3 | Task 3's "renderer is the authority" rule widens Task 2's narrow qualifier to "any string mismatch" | Task 3 Step 4 now reads: "the renderer-authority rule of Task 2 applies only to a spelling the renderer owns, never to a tree difference. Fix the lowering, not the expectation." | FIXED |
| fidelity M-4 | self-review defers all of §12 item 4, contradicting `STAGED_PLAN`'s assignment of it to S0 | self-review item 1 now names "§12 item 4's HOST half... → Tasks 1-3 tests, its device half → Stage 3"; `STAGED_PLAN` paragraph updated to match | FIXED |
| fidelity M-5 | repo follow-up `md-encode-keyless-template-sigless-path-not-gated` (owned by this stage) has no task | new Task 8 (`md encode` gates a signature-free path under every wrapper); Task numbering 8→9 shifts the old whole-stage-gate task down | FIXED |
| fidelity N-1 | test named `..._carry_four_journey_keys_or_fewer` never asserts `<= 4` | renamed `keyed_compose_vectors_bind_at_most_the_four_journey_keys`; adds `assert!(v.keys.len() <= XPUB.len(), ...)` | FIXED |
| fidelity N-2 | `unsorted` silently discarded where sorted wasn't legal, no note either stream | new CLI note "`unsorted` has no effect here..." (mod.rs) + new test `compose_says_when_unsorted_had_no_effect` | FIXED |
| fidelity N-3 | `after=<unix-time>` without `t` refuses without naming the suffix | new message: "...for a Unix time write after={h}t"; test asserts stderr contains `"after=1893456000t"` | FIXED |
| tests I-1 | `template_with_origins` has no test independently proving no-collision at ≥10 slots | new hand-written (non-printer) test `template_with_origins_inlines_two_digit_slots_without_touching_their_prefixes`, 12 slots, literal expected string | FIXED |
| tests I-2 | Task 2 Step 4's "every test PASSES" is false — its own new test panics on the `tr` stub | `Wrapper::Tr` sub-case removed from `unseated_slots_take_ascending_default_accounts_under_the_wrapper_script_type` (comment: "tr's 3' is asserted in Task 3..."); Task 3 gains `tr_default_origins_use_script_type_three` | FIXED |
| tests M-1 | doc comment claims "exits 1 like every other `CliError`," false for `BadArg` | comment now: "exits 1 (`BadArg`, above, is the one variant with its own arm, exiting 2)"; top-of-plan Global Constraints line states the exception explicitly | FIXED |
| tests M-2 | 2 more vacuously-passing gate tests not named alongside the 1 the plan calls out | "What the build gate covers, and does not" section now names both `every_compose_manifest_entry_is_in_the_family` and `keyed_compose_vectors_bind_at_most_the_four_journey_keys` as vacuous (0 iterations) until Task 5 | FIXED |
| tests M-3 | consensus-band test wildcards `why` (`{ path: 0, .. }`), can't catch wrong-band wording | rewritten to `assert_eq!(err, ComposeError::LockOutOfRange { path: 0, why }, ...)` against the literal `why` string per case | FIXED |
| tests M-4 | test named "the first listed..." has a fixture with only one candidate, doesn't independently prove firstness | renamed to `the_unlocked_single_key_becomes_the_internal_key_and_slot_zero` (fixture unchanged; the overclaiming name is gone); `only_the_first_listed_unlocked_single_key_is_extracted` remains the test that has two candidates | FIXED |
| tests N-1 | 3 preset-refusal assertions check `.is_err()` without the `ComposeError` kind | `presets_refuse_parameters_the_grammar_refuses` now matches `Err(ComposeError::BadThreshold { .. })` / `Err(ComposeError::LockOutOfRange { path: N, .. })` / `Err(ComposeError::PresetShape { .. })` per case | FIXED |
| tests N-2 | `seat::compose` / `compose` module name overlap undocumented | new doc comment on `cmd/compose.rs`: "Not to be confused with `crate::seat::compose`, which SEATS keys into an existing keyless card..." | FIXED |

(Fidelity N-1/N-2/N-3 numbering and tests N-1/N-2 numbering both appear in the source reports; all 20 rows above are the union of both reports' findings, 16 distinct + the fidelity/tests N-1 both being separate items in each report, giving 20 total rows — matches "0C/4I/5M/3N" + "0C/2I/4M/2N" = 12+8 = 20.)

## Regressions / incomplete propagation

**None found.** Phrasings grepped for survival in the AFTER text, all absent unless noted as an intentional contrast:

- `"Commands"` (bare, as an enum name) — 0 occurrences. The only `Command::` hits are the CLI's own enum (`Command::Compose`) and `assert_cmd::Command::cargo_bin("md")` (an unrelated, correctly-named type from a different crate) — not a collision.
- `"fp:one-seed"` (the old collapsed tag, as opposed to `fp:one-seed-one-path`/`fp:one-seed-two-paths`) — 0 occurrences.
- `"unseated-slot origins per wrapper"` (the old ungated coverage-gap phrase) — 0 occurrences; superseded by the `origins:default-<wrapper>` tag now in the required list.
- `the_first_listed_unlocked_single_key_becomes_the_internal_key_and_slot_zero` (old test name) — 0 occurrences.
- `keyed_compose_vectors_carry_four_journey_keys_or_fewer` (old test name) — 0 occurrences.
- `"exits 1 like every other"` / any unqualified "every CliError exits 1" — 0 occurrences; every remaining "exits 1" statement carries the `BadArg` exception.
- `"three hand-picked shapes"` — 1 occurrence, but as the FIX's own contrast ("the §5b cross-check holds... for EVERY vector, not for three hand-picked shapes") documenting what was fixed, not a survival of the old behavior.
- Stray `Task 8` meaning the old "whole-stage gate, vendoring, release notes" task — 0 occurrences; every `Task 8` reference in the AFTER text is the new `md encode` parity task, and the old task is consistently `Task 9`.
- Leftover `47`/`48` test-count phrasing — 0 occurrences; the one count statement present reads "51 of 52 compose tests pass... the 52nd is the PINNED red," matching the machine-checked number.
- `TBD`/`TODO`/`FIXME`/`XXX` — 0 occurrences outside the self-declared "no TBD/TODO" placeholder-scan line.

## Claims checked

1. **Retagged `family()` tag arithmetic — TRUE.** Parsed all 28 entries programmatically (not hand-counted). All 33 required tags appear; every one has count ≥ 2 except `spine:0` (count exactly 1, and it is the sole member of `SINGULAR_TAGS`, matching its own exemption test `assert_eq!(count.get(t), Some(&1), ...)`). `fp:distinct` appears on exactly the two `*_distinct_fingerprints`-named entries (count 2). `origins:default-<w>` matches its entry's `w:<w>` tag on all 28/28 entries (scripted check, 0 mismatches). For every `keyed_compose_*` entry (the `fp:none`-tagged `compose_*` entries are deliberately unkeyed and exempt from this rule by design — confirmed by the doc comment "`fp:none` marks the unkeyed vectors"): `fp:one-seed-one-path` is present iff some path's key-set size ≥ 2 (0 mismatches, both directions); `fp:one-seed-two-paths` is present iff ≥ 2 keyed paths (0 mismatches, both directions). One entry, `keyed_compose_tr_key_path_only` (single path, single slot), carries no `fp:*` tag at all — correctly, since a 1-slot list cannot exhibit any fingerprint-repetition case and it is not one of the "unkeyed" `compose_*` vectors either; this is not a violation of the required-tag rule (which only requires each *tag* to have ≥ 2 carriers, not that each *vector* carry a tag).
2. **Task 2's Tr removal + Task 3's tr default-origin assertion — TRUE.** `unseated_slots_take_ascending_default_accounts_under_the_wrapper_script_type` (line 749 in AFTER) tests only Wsh (2 cases) / ShWsh / Sh, with an explicit comment "tr's 3' is asserted in Task 3, once the taproot lowering exists." Task 3 adds `tr_default_origins_use_script_type_three` (line 1329) asserting `hardened(&[48,0,0,3])` / `hardened(&[48,0,1,3])`.
3. **Task 7's pinned templates, `decaying_multisig` signature, `blocks()` path index — TRUE.** `presets_lower_to_their_pinned_templates` pins exactly 6 literals (plain_multisig, simple_timelocked_inheritance, kofn_recovery, tiered_recovery, hashlock_gated, decaying_multisig). All 6 `decaying_multisig` call sites in the plan use the new 8-argument form `(Wrapper, k1, n1, k2, n2, older1, older2, after_height)`. Every `blocks(...)` call site (6 of them) passes an explicit path index; `LockOutOfRange.path: usize` and `Display` formats it as `path + 1`, matching the pinned refusal text `"path 2: older in blocks needs 1..=65535"` for a tier-2 (index-1) defect.
4. **No `Commands`; no unqualified "every CliError exits 1" — TRUE.** See Regressions section above; both confirmed absent, and the doc comment now states the `BadArg`-exits-2 exception directly beside the `Compose` variant.
5. **Task 4 cross-check builds from the PATH LIST; `the_cross_check_notices_a_wrong_lowering` exists and can fail — TRUE.** `concrete_policy(list: &PathList, c: &Composed, keys: &[String])` iterates `list.paths` (not the lowered `Node`/`Body` tree), with its own doc comment: "it is built from the path list, not from the lowered tree, so it cannot inherit a lowering defect." `the_cross_check_notices_a_wrong_lowering` mutates the compiler's policy string (`thresh(2,` → `thresh(1,`) and asserts `assert_ne!(ours, theirs, "the lift comparison must be able to fail")`.
6. **Coverage/self-review names §12 item 4's host half and the two vacuous-in-gate tests — TRUE.** Self-review item 1: "§12 item 4's HOST half (every §4e refusal, the §4c bands in and out per kind including `older(0x400000)`, the 33rd slot, the one-fingerprint invariant) → Tasks 1-3 tests, its device half → Stage 3." "What the build gate covers, and does not" section names `every_compose_manifest_entry_is_in_the_family` and `keyed_compose_vectors_bind_at_most_the_four_journey_keys` as the two 0-iteration vacuous passes.
7. **`main.rs:96` is `enum Command {` at `3b0944fb`; `template.rs:2677-2700` spans the `ms_desc` construction — TRUE.** `git show 3b0944fb:crates/md-cli/src/main.rs` line 96 is exactly `enum Command {` (line 95 is `#[derive(Debug, Subcommand)]`). `git show 3b0944fb:crates/md-cli/src/parse/template.rs` line 2677 is exactly `let ms_desc = if experimental {`, and line 2678 is exactly the comment the plan quotes ("`from_str`'s tr-only sanity gate"); the full `if/else` construction runs through line 2702 (`};`) — the cited range 2677-2700 covers the construction's opening through its `else` arm, 2 lines short of the closing `};`, which is immaterial to "really spans."

## New defects introduced by the fold

**One, Minor, in `decaying_multisig`'s own new doc comment vs. its own new guard.** The doc comment added by this fold reads:

> "k1-of-n1 after `older1`; **a SMALLER recovery quorum** k2-of-n2 (distinct keys) after `older2 > older1`... The toolkit's archetype takes the primary and recovery quorums as separate parameters and refuses tiers that do not unlock progressively later; so does this."

But the guard the same fold added only checks `k2 > k1` (threshold) and `older2 <= older1` (timing) — it does not check `n2` against `n1` at all. A call such as `decaying_multisig(Wrapper::Wsh, 2, 3, 2, 5, 1000, 2000, H)` (same threshold, and a *larger* keyset for the "recovery" tier) passes both guards and is not "a smaller recovery quorum" by any reading — it is a same-or-larger one. This does not resurrect fidelity I-3 (the finding's own stated defect — that the function *could not vary* the recovery tier's shape at all — is unambiguously fixed: the pinned test now demonstrates `k1=2,n1=3 → k2=1,n2=2`, a genuine decay), and no test in the plan exercises the gap. Filing it here as new text this fold introduced that overstates its own guarantee, not as a reopening of I-3.

No other new defects found: literals cross-checked internally consistent (the `decaying_multisig` pinned template `wsh(or_i(and_v(v:multi(2,@0,@1,@2),older(1000)),or_i(and_v(v:multi(1,@3,@4),older(2000)),and_v(v:pkh(@5),after(4000000)))))` matches the call `decaying_multisig(Wrapper::Wsh, 2, 3, 1, 2, 1000, 2000, 4_000_000)` tier-for-tier); `PresetShape { why: &'static str }` field type matches its two call-site literals; `LOCKTIME_THRESHOLD` referenced from Task 6 is defined `pub` in Task 1 (line 300); no Task-numbering, tag-count, or file-table inconsistency found anywhere the diff touched.

## What I ran

```
git -C mnemonic-engrave show f531cff --stat; b820b64 --stat        # confirm report commits
cat design/agent-reports/composer-S0-plan-R0-r0-{fidelity,tests}.md
git -C mnemonic-engrave log --oneline 3a799fa..891b17d -- design/
git -C mnemonic-engrave diff 3a799fa..891b17d --stat -- design/
git -C mnemonic-engrave show 891b17d --stat                        # confirm fold commit's own scope
git -C mnemonic-engrave show 891b17d -- design/STAGED_PLAN_wallet_policy_composer.md
git -C mnemonic-engrave show 3a799fa:design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md > plan_before.md
git -C mnemonic-engrave show 891b17d:design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md > plan_after.md
git -C mnemonic-engrave show 891b17d -- design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md > plan_diff.txt
# Python: bracket-balanced parse of family()'s 28 tuples -> per-entry tags,
# per-entry path-list slot counts (via the k()/u()/lk()/hs()/kl() helper
# signatures read from the plan itself), cross-tabulated against the
# required-tag list and the fp-tag rules; 0 mismatches on all four claim-1 sub-checks.
grep -n <phrase> plan_after.md   # for every superseded-phrasing check in "Regressions"
git -C descriptor-mnemonic show 3b0944fb:crates/md-cli/src/main.rs | sed -n '90,100p'
git -C descriptor-mnemonic show 3b0944fb:crates/md-cli/src/main.rs | grep -n '^enum Command'
git -C descriptor-mnemonic show 3b0944fb:crates/md-cli/src/parse/template.rs | sed -n '2670,2705p'
git -C descriptor-mnemonic show 3b0944fb:crates/md-cli/src/parse/template.rs | grep -n 'let ms_desc = if experimental'
```

Did not read any `.jsonl` file. Did not re-run `scripts/plan-build-gate-md.sh` (compile/test-pass state at the fold commit is stated as already machine-checked in the dispatch brief and was not treated as a finding).
