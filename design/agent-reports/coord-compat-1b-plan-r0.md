# R0 architect review: coordinator-compat plan 1b (verdicts)

**Verdict: NOT GREEN. 0 Critical, 2 Important, 4 Minor, 1 Nit.** The design holds up. The R-1 rule matches libnunchuk's source, the Core `<0;1>` ordering and Liana's time-units-before-absolute ordering are sound, the none case is sound and its refusal names the flag, and nothing that `liana_refuse_or_warn` said goes missing. Two things block. First, the narrowed Liana class 9 refuses a policy that Liana v15.0 imports, and `md compose` prints that refusal for an ordinary `--path` spelling. Second, the new none-case refusal breaks a shipped cross-repo oracle test in the fork on any machine where md 0.20.0 is installed.

Artifact: `design/IMPLEMENTATION_PLAN_coordinator_compat_1b_verdicts.md` (engrave HEAD `eb2e9af1`). Code target: descriptor-mnemonic `d269c556`. I ran every probe on a copy of the author's assembled tree (`.tmp/cc1b-extracted`, copied to `.tmp/r0-cc1b/tree`). I added a throwaway example, `crates/md-codec/examples/r0probe.rs`, which runs `descriptor_from_text → skeleton → verdicts → describe`. The Liana harness binary is `.tmp/liana-harness-target/debug/liana-harness`, built against `.tmp/fable-liana-src-v15` (`git describe` = `v15.0`, crate 15.0.0). Probe inputs and outputs are in `.tmp/r0-cc1b/{in2,out2,in3,out3}.jsonl`. They use the X24 `md`-variant keys from `fable-liana-parse-in.jsonl`.

---

## Important

### I-1. Liana class 9 ("a second unlocked multi-key path") refuses a policy Liana v15.0 imports

**The rule** (registry, `LIANA_CLAUSES`, last clause) fires when any unlocked branch after the first has `slots.len() >= 2`. It is meant to capture "Liana folds a second SINGLE-key path (X24) but refuses a MULTI-key one (X25/X26)".

**What Liana actually does** (`liana/src/descriptors/analysis.rs`, v15.0, `from_multipath_descriptor`): it lifts the policy and calls `.normalized()`. In rust-miniscript 12.3.x `semantic.rs:359-426`, `normalized()` flattens every `k == 1` sub-threshold into its `or` parent and keeps the source order. Liana then folds every bare `Key` that follows the primary path into that path (`with_added_key`), and refuses only a non-`Key` sub. A `multi(1,C,D)` second path is therefore flattened into two bare keys and **folded**, exactly like X24's single key.

**Counterexample, measured.** The descriptor is `wsh(or_d(multi(2,A,B),or_i(multi(1,C,D),and_v(v:pkh(E),older(100)))))`, with A..E the X24 `md`-variant keys (row `cx-multi-then-1of2multi` in `.tmp/r0-cc1b/in2.jsonl`):

- Liana v15.0 harness: `ok: true`, primary `k=2` over **4** keys (C and D folded in). It is imported, and read as altered.
- Plan's `verdicts()`: `Liana 8.0-15.0: refuses (a second unlocked multi-key path)`.
- Reached from the CLI with no keys at all:
  `md compose --wrapper wsh --path 2of2 --path 1of2 --path 1of1,older=100` prints the same false refusal. compose lowers `1of2` to `multi(1,…)`. The `tr --unspendable liana` spelling prints it too, via `multi_a(1,…)`.

**Controls** (same run): `[pk(D), multi(2,A,B), rec]` → Liana refuses and the rule fires (correct). `[multi(2,A,B), pkh(D), rec]` → Liana imports and the rule stays silent (correct).

The registry's own header says a clause "must never refuse what the coordinator imports: that is a D1". The build could not catch this one because no evidence row has this shape. Operator harm is limited (Liana's actual reading is 2-of-4, which the operator should avoid anyway), but it is a wrong verdict from the rule set, which is question 1 exactly.

**Remedy.** Narrow the predicate to `b.k >= 2`. That under-approximates: an `and`-of-keys second path would be missed and left silent, which is allowed. Add the counterexample row to evidence so the build D1-checks it. The inputs and outputs above are ready: commit them under `design/evidence/coord-compat-1b/` in Task 0 next to the Nunchuk probe, and let the table store its `ImportsAltered` cell. Add a registry test for `1of2`-after-`2of2`. Fix the clause comment and the §2 bullet ("a second SINGLE-key unlocked path is imported") to say "any path Liana's normaliser flattens into bare keys (a single key or a 1-of-n)".

### I-2. The none-case refusal breaks the fork's host-oracle test once md 0.20.0 is on PATH

The plan's §2 and CHANGELOG say the behaviour change touches four md-cli tests. A shipped sibling test also depends on the old behaviour. `seedhammer` `gui/composer_fable_r0_funds_test.go:131-160`, `TestFableTwoKeylessPathsAgreeWithTheHostOracle`, runs `md compose --wrapper wsh --experimental --path …` for five ADMITTED keyless rows and calls `t.Fatalf` if exit≠0 disagrees with its table. It skips only when `md` is not on PATH (`:137`). The fork CI has no `md`, so CI stays green. The maintainer's machine keeps `md` current (`~/.cargo/bin/md`, 0.19.0 today) and runs the `gui` shard set at gates.

**Counterexample, measured with the plan's md-cli:**
`md compose --wrapper wsh --experimental --path keyless,sha256=<H>,older=5 --path 2of3` exits **1** ("no wallet coordinator md knows imports this policy … Pass --md-only"). The row `[K+older(5), 2of3]` has `admit: true`, so the test calls `t.Fatalf`. The same happens for `[keyed, K]`. As soon as md 0.20.0 is installed locally, the fork's `gui` package goes red.

**Remedy (records only, stays inside "no fork change").** In Task 0 Step 2, file a follow-up with an owning phase of "before md-cli 0.20.0 is installed on the maintainer box (else plan 3)": append `--md-only` to that test's args. The admit column then keeps meaning "the composer admits it". Add one line to Task 9 naming it. Also confirm whether the fork's `oraclelive` compose tests use pinned md binaries: they are hash-pinned (`oracle/pins.json`) and unaffected until repinned. Record that they need the same flag when repinned.

---

## Minor

### M-1. A Liana `Imports`/`ImportsAltered` cell extends to an `older()` value Liana refuses

`SkeletonKey` abstracts lock values (design §1A blind spot). The time-units clause fires only on `LockKind::OlderUnits`. Liana's `csv_check` refuses any relative lock that is not a u16 block count, and a value over 65535 without bit 22 is classified `OlderBlocks`.

**Counterexample, measured.** X24 with `older(70000)`: Liana v15.0 answers `Timelock value '70000' isn't valid or safe to use`. The plan's verdict is `Liana 8.0-15.0: imports the multipath form, but reads it as Liana 2-of-4 … (2026-09-20)`, which is the X24 cell. Reachability is low: `md encode` refuses the value (`validate_relative_timelocks`), so only a card minted before that gate or by another minter, read through `md descriptor` or `md shape-key --descriptor`, gets here. The worst case is a failed Liana import, so this does not gate.

**Remedy:** widen that clause to "a relative lock Liana cannot use" (units, or value > 0xFFFF). Declare it `LOCK_VALUES`; `class_of_message` already maps the message there.

### M-2. `policy-differential.py` now reports keyless-`wsh` compose refusals as findings

`scripts/policy-differential.py:262-269` generates keyless paths under `wsh` and treats any `md compose` refusal as a real finding. It retries only when the error names `--experimental` (`:381`). With md 0.20.0 every keyless-`wsh` case becomes a "compose" refusal. **Remedy:** add a FOLLOWUP to pass `--md-only` when `needs_experimental`, or to retry on `--md-only` the way it retries on `--experimental`.

### M-3. ms-cli's printed recipe now needs a flag it does not name

`mnemonic-secret` `crates/ms-cli/src/cmd/hashlock.rs:525` prints `for md compose:  --path <your other paths> --path keyless,<kind>=<digest>`. Under `--wrapper wsh` that recipe now fails. The refusal names `--md-only`, so this is documentation only. **Remedy:** a FOLLOWUP to mention `--experimental --md-only` (for `wsh`) in the recipe.

### M-4. "two paths with one lock" declares `reads: S` but reads `l.value`

`ReadSet` exists so that §1 (a2) can ask what a clause reads. It is harmless today, because a template carries lock values and only `KEY_IDENTITY` is ever skipped. It is still a false declaration. **Remedy:** `S.with(ReadSet::LOCK_VALUES)`, and the same for M-1's clause.

## Nit

- N-1. The xtask's `main` `--check` branch is exercised by no test. `table_is_fresh` covers the comparison, and CI never calls `--check`, so a mutation there goes unseen. That is acceptable, but the doc could stop implying `--check` is the gate.

---

## Answers to the five questions, where nothing was found

1. **Orderings.** *Core `<0;1>` first*: runtime and build both check `form_refusals` before clauses. A multipath `tr` miniscript on 24.2-25.2 is named for the spelling, and `--chain 0` falls through to "miniscript under tr". The verdict is right either way; only the name changes. *Liana units-before-absolute*: it only changes which class is named, since both refuse. *R-1*: libnunchuk `a7cfb49` `src/descriptor.cpp:689-712` sorts byte vectors of the xpub pubkeys and `std::unique`s them (`:700-701`). Liana's `unspendable_internal_xpub` (`analysis.rs:412-444`) concatenates `xkey.public_key` in `tap_tree().iter()` order with duplicates. They agree exactly when the sequence is strictly ascending, which is what `leaf_keys_ascending` computes (`pk <= p` → false). `ParseSortedMultiDescriptor` (`:411-434`) does not reorder signers, so no second key-order dependence was found. The Nunchuk keyless-`wsh` rule is confirmed at `:573` → `nunchukutils.cpp:1276` → `contrib/bitcoin` `57b47c4` `miniscript.h:1617`. Same-xpub reuse at two use-sites, where `key_partition` ignores the use-site but Liana's `DescKeyChecker` does not, is unreachable: `md encode` refuses both spellings (measured).
2. **None case.** It is sound. It fires only when all three coordinators refuse through rules, which in practice means a keyless `wsh` path, and each of those is source-backed. `sh-wsh` and `tr` keyless stay `Unproven` for Nunchuk, so they compose. The refusal text ends "Pass --md-only to compose it anyway", `--json` refuses the same way, and a pointless `--md-only` gets a note. It matches rulings 2 and 5. Shipped callers checked: `demo/sh2/rehearse.sh:43-50` (all four exit 0 under the plan's md), the toolkit (no `md compose` invocation outside `vendor/`), `me` (pinned md-codec rev, no `md` subprocess), and the fork (I-2).
3. **`liana_refuse_or_warn`.** Nothing disappears. The hashlock warning maps to the "a hash lock" clause (S, so it fires for templates). The all-locked warning, with no internal key, maps to "no unlocked path", which has the same predicate once the real internal key is branch 0. SPEC §6's refusal and the "has no effect" warning survive in `unspendable_liana_checks`. The only change in meaning is the intended one: the line now prints without the flag and names a class. The one new wrong line is I-1.
4. **False passes.** `table_is_fresh` regenerates through the same `parse` and `build` and compares against the committed bytes, so any evidence edit, rule change or xtask parse mutation that moves output turns it red. A disagreement makes `generate()` return `Err`, and the test panics. CI runs `cargo test --workspace`, which includes the xtask. `.gitattributes` pins LF on both sides. Declared-unkeyable rows are confirmed by `build` (`build.rs:187-195`). The engrave-side freshness is local-only, as §1 states.
5. **Scope.** Everything maps to design §5 steps 1-2 or to an item owned here (1a M-1, F-644 md-cli, F-655). Nothing is premature. What is missing is the records for I-2, M-2 and M-3.

ready for implementation: no
