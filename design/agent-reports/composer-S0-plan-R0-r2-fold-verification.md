# Fold verification — composer S0 plan, round 1 → two follow-through folds

**Artifact:** `design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md` (+ one paragraph of `design/STAGED_PLAN_wallet_policy_composer.md`).
**BEFORE:** `891b17d` (the AFTER of round-1 verification, `design/agent-reports/composer-S0-plan-R0-r1-fold-verification.md`). **AFTER:** `761ded7` (= working tree).
**Commits under review:** `fb65f2c` (controller's own follow-through after hand-checking Task 8 in a scratch copy: gate the sanity/`ext_check` rules on `Disposition::Refuse`; tag the two keyless-wsh vectors `no-corpus`; keep them out of `MANIFEST`; genericize `ext_check`'s error type; rewrite coverage) and `761ded7` (response to round-1's one new Minor: `decaying_multisig`'s doc comment restated to match its guard, plus a positive same-threshold-wider assertion).
**Lens:** mechanical fold verification — did these two folds introduce a new defect, and are the specific claims their new text makes actually true against the real `descriptor-mnemonic@3b0944fb` source and the pinned `rust-miniscript@ff4732e` source. Read-only on every repo; no `.jsonl` file read; no file written outside this report path.

**What I ran:** `git show`/`git diff 891b17d..761ded7 -- design/` for the exact hunks; `git show 3b0944fb:...` against `descriptor-mnemonic` for every cited file/line (parse/template.rs, cmd/encode.rs, cmd/verify.rs, cmd/build.rs, cmd/vectors.rs, cmd/decompose/mod.rs, cmd/compile.rs, format/text.rs, format/json.rs, parse/reuse.rs, cmd/descriptor.rs, cmd/address.rs, tests/n1_admission_taxonomy.rs); a full-repo grep for every other caller of `parse_template`/`parse_template_ext`; `sed`/`grep` reads of the pinned rust-miniscript checkout at `/home/bcg/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/ff4732e/src/{miniscript/analyzable.rs,descriptor/{mod,segwitv0,sh}.rs}` to confirm `sanity_check`/`ext_check`/`as_inner`/`ExtParams` signatures and semantics (in particular that `Miniscript::sanity_check` calls `has_repeated_keys()` → `AnalysisError::RepeatedPubkeys`); hand-trace of `decaying_multisig`'s new positive test call against `validate()`/`checked()`/`ks()`/`blocks()` in the plan's own Task 1/7 text. Did **not** re-run `scripts/plan-build-gate-md.sh` (the stated build-gate result is accepted per the brief).

## VERDICT: 0 new defects / 0 false claims

Both folds' new text is true against the cited sources. One pre-existing inaccuracy (not introduced by either fold under review, and already flagged as immaterial by the round-1 report) is noted below for completeness, not counted against this round.

## Claims checked

**1. Disposition claim — TRUE.** `crates/md-cli/src/parse/template.rs` at `3b0944fb`: `parse_template` (line 2612) is a thin wrapper that always calls `parse_template_ext(..., false, crate::parse::reuse::Disposition::Refuse)` (line 2622-2624). `cmd/encode.rs:69-75` passes `crate::parse::reuse::Disposition::Refuse` explicitly, with the comment "N1's REFUSE disposition: `encode` MINTS". `cmd/verify.rs:52-58` passes `crate::parse::reuse::Disposition::Warn` explicitly, with the comment "N1's WARN disposition, not REFUSE. `verify` READS". `cmd/vectors.rs:54` calls `parse_template` (→ always `Refuse`) and its own test comment (`cmd/vectors.rs:229-234`) states "The generator's ONLY route from a `Vector` to a `Descriptor` is `parse_template`... and that entry carries N1's REFUSE disposition" — confirming the plan's "`md vectors` parses under the minting disposition" claim. Task 8's new fragment's `let minting = matches!(reuse, crate::parse::reuse::Disposition::Refuse);` and its two `if minting` guards (on `MsDescriptor::Wsh`/`MsDescriptor::Sh`) plus the wrapped `sanity_check()` call are exactly what the plan's Task 8 text and the `fb65f2c` commit message describe.

**2. The N1 reading-verb claim — TRUE.** `crates/md-cli/tests/n1_admission_taxonomy.rs`: `verify_template_warns_and_completes_on_a_refused_shape` (line 461) and `r_n1a_card_verifies_at_exit_0_with_a_warning` (line 713) both invoke `md verify` (via the `verify_card`/direct-CLI helpers), i.e. the reading verb, against `T_N1A = "wsh(sortedmulti(2,@0/<0;1>/*,@0/<0;1>/*))"` (line 119) — a repeated-key shape (the same `@0` slot bound twice). Confirmed independently against the pinned miniscript source (`analyzable.rs:225-234`): `Miniscript::sanity_check()` checks `self.has_repeated_keys()` and returns `Err(AnalysisError::RepeatedPubkeys)` if true, and `Wsh::sanity_check` (`segwitv0.rs:56-58`) delegates straight into it. So an **unconditional** `d.sanity_check()` in `parse_template_ext`'s non-experimental branch — which is what round-0's Task 8 draft added — would indeed reject `T_N1A` regardless of `verify`'s `Warn` disposition, exactly as the plan's fragment comment says, and exactly why the two named tests are the correct evidence for gating the check to `minting` only. (The plan's illustrative example text, `sh(multi(1,@0/**,@0/**))`, is BIP-388's own canonical name for the "repeated keys" example class used throughout this codebase's docs/tests — not a literal quote of `T_N1A`'s string, which differs in wrapper/threshold/path-spelling but is the same violation class. Not a defect: the codebase itself uses that canonical name the same way elsewhere, e.g. `design/SPEC_wallet_form_converter.md:224`, `tests/seating_vectors.rs:713`.)

**3. The no-corpus scheme — TRUE, and the two named tests correctly need no change.** In `family()` (lines 1856-1868 of the working-tree plan), exactly two entries carry `"no-corpus"` — `compose_wsh_keyless_hash_path` and `compose_wsh_keyless_hash_only` — and they are exactly the two (and only two) entries carrying `"keyless-wsh"`. `every_compose_vector_in_the_manifest_is_exactly_what_compose_renders` (line 1918-1931) now special-cases `tags.contains(&"no-corpus")`: it asserts `MANIFEST.iter().all(|v| v.name != *name)` then `continue`s, skipping the normal MANIFEST-lookup/render-compare — matching "skips them AND asserts their absence." `print_family_templates_for_the_manifest` (line 2002-2010) skips the same two before printing. Counts: `family()` lists 22 `keyed_compose_*` + 6 unkeyed `compose_*` = 28 total (line-by-line count of the vec! literals, matches "28 in family()"); of the 6 unkeyed, 2 are `no-corpus`, leaving 22 keyed + 4 unkeyed = 26 pasted (matches Task 5's "twenty-six" printer-count line 2014, the Task 5 commit-message's "28 tagged vectors (26 in MANIFEST)" at line ~2045, and Task 9's "26 vectors' worth" regeneration line at 2896); the exporter's 22 `keyed_compose_*.conformance.json` count (line 2027-2028) is unaffected since both no-corpus entries are unkeyed. The `STAGED_PLAN_wallet_policy_composer.md` S2 paragraph (lines 93-96) states Go mirrors "the 26 in the corpus, plus the two `no-corpus` keyless-wsh entries of S0's `family()`... because the exporter cannot emit an EXPERIMENTAL shape" — matches. Neither `every_compose_manifest_entry_is_in_the_family` nor `keyed_compose_vectors_bind_at_most_the_four_journey_keys` needs to change, and the fold correctly left both untouched: the first iterates `MANIFEST.iter().filter(|v| v.name.contains("compose_"))` and only asserts the MANIFEST→family direction (Π⊆Σ) — an entry's *absence* from `MANIFEST` is invisible to it, so removing the two no-corpus rows from `MANIFEST` changes nothing this test checks. The second filters `MANIFEST.iter().filter(|v| v.name.starts_with("keyed_compose_"))`, and both no-corpus rows are named with the plain `compose_` prefix (unkeyed) and were never in `MANIFEST` to begin with, so they were never in this test's iteration space regardless of tagging.

**4. The Task 8 code fragment — TRUE** (all referenced names exist and the replacement region is correctly identified, modulo one pre-existing, immaterial line-count-off-by-2 in the cited range). Verified against `descriptor-mnemonic@3b0944fb`, `crates/md-cli/src/parse/template.rs`: line 2677 is exactly `let ms_desc = if experimental {`, and the `if/else` construction it opens runs through line 2702 (`};`), not line 2700 as the "Files:" line states (`crates/md-cli/src/parse/template.rs:2677-2700`, line 2716 of the current plan) — the range is 2 lines short of the actual close. **This citation is unchanged since `891b17d`** (confirmed: `git show 891b17d:design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md` has the identical `:2677-2700` string at line 2699) and was already surfaced and explicitly judged immaterial by the round-1 report's claim 7 ("2 lines short of the closing `};`, which is immaterial to 'really spans'") — so it predates both commits under review here and is not a new defect from `fb65f2c`/`761ded7`. All names the new fragment references exist at the pinned `rust-miniscript@ff4732e`: `Wsh::as_inner() -> &Miniscript<Pk, Segwitv0>` (`descriptor/segwitv0.rs:38`), `Sh::as_inner() -> &ShInner<Pk>` (`descriptor/sh.rs:112`), `Miniscript::ext_check(&ExtParams) -> Result<(), AnalysisError>` at exactly line 242 of `analyzable.rs` as cited, `ExtParams::new()`/`.top_unsafe()` (`analyzable.rs:46,87`), `Descriptor::sanity_check(&self) -> Result<(), Error>` (`descriptor/mod.rs:317`), and `ShInner::{Wsh, Wpkh, Ms}` (used identically elsewhere in the real repo, e.g. `decompose/mod.rs`'s `DescriptorType` match). `reuse` is the function's own parameter (already in scope); `relaxed`/`relaxed_err`/`MsDescriptor` are pre-existing locals/imports in the surrounding function. The generic `relaxed_err<E: std::fmt::Display>(e: E) -> CliError` change is required and correctly reasoned: `ext_check` returns `AnalysisError` (line 242), not `miniscript::Error`, so a closure typed `|e: miniscript::Error|` (as the pre-Task-8 code used for the `tr`-only case, which only ever called `.ext_check`, never `.sanity_check`) would indeed fail to type-check once the same `relaxed_err` is reused for a `sanity_check()` error site too — E0631 is the correct diagnosis.

**5. `761ded7` — TRUE.** The full diff of `761ded7` (isolated via `git show 761ded7 -- design/`) touches only two spots, both in `crates/md-codec/src/compose/presets.rs`'s plan text: the doc comment on `decaying_multisig` and one test. New doc comment: "a recovery quorum k2-of-n2 (distinct keys) that is NO HARDER to satisfy than the primary (`k2 <= k1`; `n2` is free...) after `older2 > older1`... What 'decay' means here is exactly those two guards, nothing more." The function body's only two guards are `if older2 <= older1 { return Err(...) }` and `if k2 > k1 { return Err(...) }` — no check on `n2` at all — so the doc now states *exactly* the two enforced conditions and nothing more, matching the guard precisely. The new positive assertion `presets::decaying_multisig(Wrapper::Wsh, 2, 3, 2, 5, 1000, 2000, 4_000_000).is_ok()` is consistent with both the guard and `validate()` (`design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md:573-608`): `older2(2000) > older1(1000)` ✓, `k2(2) <= k1(2)` ✓ (same threshold, exactly the "same-threshold wider recovery tier" case); `ks(2,3)`+`ks(2,5)`+`ks(1,1)` gives per-path thresholds `2<=3`, `2<=5`, `1<=1` (all legal, all `n <= MAX_KEYS_PER_PATH=9`), total slots `3+5+1=9 <= MAX_SLOTS=32`, `Wrapper::Wsh` is not `is_legacy()` so the sole-sorted legacy-shape branch is skipped, and `blocks(1000,0)`/`blocks(2000,1)` both fit `u16` and are `>=1` — every branch of `validate`/`checked` returns `Ok`, so `.is_ok()` is a true assertion.

**6. Propagation — clean.** Grepped the working-tree plan and staged plan for every superseded phrasing named in the brief: `"six \`compose_*\`"` — 0 hits (now correctly "four `compose_*`" at line 2028, referring to the unkeyed non-no-corpus set). `"twenty-eight \`name<TAB>template\`"` — 0 hits (now "twenty-six" at line 2014). `"a SMALLER recovery quorum"` — 0 hits (replaced by "NO HARDER to satisfy" / "no harder than the primary"). Every remaining occurrence of the word "unconditional" (lines 2826, 2865, 2900, 2932) is a historical/explanatory reference to the discovered defect or an unrelated fact (module unconditionality), never a live claim that the current gate itself is unconditional — confirmed by reading each in context. `"the behaviour needs the modified binary"` — 0 hits; the "NOT covered by the gate" paragraph was rewritten to state plainly what was hand-checked (main.rs wiring + Task 8's fragment, 761/0/1 whole-suite result) versus what remains for the implementer (the 26 pasted MANIFEST entries through the corpus tests/exporter) — consistent with the brief's stated already-settled facts, and not overclaiming beyond them. A bare `"Commands"` (the old wrong-enum-name spelling, a round-0 finding) does not recur. No leftover unconditional `MsDescriptor::Wsh(w) =>` / `MsDescriptor::Sh(sh) =>` arm exists anywhere else in the file (grepped; each appears exactly once, both correctly `if minting`-guarded).

## Callers of parse_template / parse_template_ext

| call site | disposition passed | minting or reading verb | now sanity-gated (Task 8)? |
|---|---|---|---|
| `cmd/encode.rs:69` (`encode::run`) | `Refuse` (explicit) | **Minting** — mints an encoded card | Yes — full gate (ext_check for Tr/Wsh/Sh under `--experimental`, `sanity_check()` otherwise) |
| `cmd/verify.rs:52` (`verify::run`) | `Warn` (explicit) | **Reading** — checks an already-engraved plate | No — `minting=false`; the new `Wsh`/`Sh` `ext_check` arms and the `sanity_check()` call are both skipped |
| `cmd/build.rs:65` (`build_descriptor`, shared by `cmd/descriptor.rs:118` and `cmd/address.rs:38`) | `Refuse` (via `parse_template`) | **Minting-adjacent** — `parse_template`'s own doc comment (line 2606-2607) groups `md descriptor --template` and `md address --template` with `encode` and `vectors` under the REFUSE disposition; both derive a fresh descriptor/address from an operator-supplied template, not from an already-engraved plate | Yes |
| `cmd/vectors.rs:54` (exporter `run`) | `Refuse` (via `parse_template`) | **Minting** — writes the corpus/`.conformance.json` files; this is the exact call the plan's no-corpus reasoning is about | Yes — this is why the two keyless-wsh vectors cannot be exported |
| `cmd/vectors.rs:241` (`#[test] the_generator_refuses_a_forbidden_shape_template`) | `Refuse` (via `parse_template`) | Test only, asserting the generator's own refusal boundary | Yes, N/A to production behavior |
| `decompose/mod.rs:478` (`decompose`) | `Refuse` (via `parse_template`) | **Reading** — decomposes an already-existing `Descriptor` into a template — BUT the call's `Err` is only turned into an advisory `notes` line ("`md encode` may not accept this template as printed..."), never a returned error, so `decompose` itself never fails because of this gate | Yes (gate applies), but its effect is text-only, not a refusal of the reading verb |
| `compile.rs:139` (`#[cfg(test)] mod tests`, `compiles_and_reparses`) | `Refuse` (via `parse_template`) | Test-only helper, no production caller | Yes, N/A to production behavior |
| `format/text.rs` (13 call sites, all inside `#[cfg(test)] mod tests`) | `Refuse` (via `parse_template`) | Test-only (round-trip tests for the template renderer) | Yes, N/A to production behavior |
| `format/json.rs:409` (`#[cfg(test)] mod descriptor_json_tests`) | `Refuse` (via `parse_template`) | Test-only | Yes, N/A to production behavior |

Every production (non-test) caller other than `verify` passes `Refuse`; `verify` is the sole production `Warn` caller. This matches the plan's "minting only" framing: Task 8 does not invent a new minting/reading split, it reuses the disposition parameter N1's own `reuse::check` (pre-existing, `crate::parse::reuse::Disposition`) already threads through every one of these call sites — consistent with the source's own "single-source rule" comment in `parse_template_ext`'s doc.

## New defects introduced

None found in `fb65f2c` or `761ded7`.

## What I ran

```
git log --oneline -5 891b17d..761ded7
git show --stat fb65f2c; git show --stat 761ded7
git diff 891b17d..761ded7 -- design/
git show 761ded7 -- design/                      # isolate r1's own diff from fb65f2c's
git show 891b17d:design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md | grep -n 'template.rs:2677-2700'

# descriptor-mnemonic @ 3b0944fb (HEAD at time of check)
git -C descriptor-mnemonic show 3b0944fb:crates/md-cli/src/parse/template.rs | sed -n '2600,2760p'
git -C descriptor-mnemonic show 3b0944fb:crates/md-cli/src/parse/template.rs | sed -n '2677,2704p'
grep -rn 'parse_template_ext(\|parse_template(' descriptor-mnemonic/crates/md-cli/src
grep -rln 'parse_template_ext\|template::parse_template\b' descriptor-mnemonic   # cross-check the caller set
sed -n '1,70p'  descriptor-mnemonic/crates/md-cli/src/cmd/vectors.rs
sed -n '225,250p' descriptor-mnemonic/crates/md-cli/src/cmd/vectors.rs
sed -n '30,70p'  descriptor-mnemonic/crates/md-cli/src/cmd/verify.rs
sed -n '45,85p'  descriptor-mnemonic/crates/md-cli/src/cmd/encode.rs
sed -n '1,70p'   descriptor-mnemonic/crates/md-cli/src/cmd/build.rs
grep -rn 'build_descriptor(' descriptor-mnemonic/crates/md-cli/src
sed -n '100,150p' descriptor-mnemonic/crates/md-cli/src/compile.rs
sed -n '450,490p' descriptor-mnemonic/crates/md-cli/src/decompose/mod.rs
sed -n '1,35p;240,275p' descriptor-mnemonic/crates/md-cli/src/format/text.rs
sed -n '395,415p' descriptor-mnemonic/crates/md-cli/src/format/json.rs
grep -n 'r_n1a_card_verifies_at_exit_0_with_a_warning\|verify_template_warns_and_completes_on_a_refused_shape\|T_N1A' descriptor-mnemonic/crates/md-cli/tests/n1_admission_taxonomy.rs
sed -n '440,480p;690,730p' descriptor-mnemonic/crates/md-cli/tests/n1_admission_taxonomy.rs
grep -rln 'sh(multi(1,@0' descriptor-mnemonic   # confirm the plan's example text is a known canonical name, not a fabricated literal

# rust-miniscript @ ff4732e (pinned)
grep -rn 'fn sanity_check' /home/bcg/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/ff4732e/src
sed -n '200,260p' /home/bcg/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/ff4732e/src/miniscript/analyzable.rs
sed -n '300,340p' /home/bcg/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/ff4732e/src/descriptor/mod.rs
sed -n '45,70p'  /home/bcg/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/ff4732e/src/descriptor/segwitv0.rs
grep -n 'pub fn as_inner' /home/bcg/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/ff4732e/src/descriptor/{segwitv0,sh}.rs
grep -n 'pub fn new\|pub fn top_unsafe' /home/bcg/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/ff4732e/src/miniscript/analyzable.rs
sed -n '242p' /home/bcg/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/ff4732e/src/miniscript/analyzable.rs

# plan-internal hand-trace for the decaying_multisig positive assertion
sed -n '560,625p;2595,2690p' design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md   # validate()/checked()/ks()/blocks() + Wrapper::is_legacy + MAX_* constants
grep -n 'MAX_SLOTS\|MAX_KEYS_PER_PATH\|MAX_PATHS\b\|fn is_legacy' design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md

# no-corpus scheme
grep -n 'no-corpus' design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md
sed -n '1731,1994p' design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md
sed -n '85,105p' design/STAGED_PLAN_wallet_policy_composer.md

# propagation grep
grep -n "six \`compose_\\\*\`\|twenty-eight\|SMALLER recovery\|behaviour needs the modified binary\|\"Commands\"" design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md design/STAGED_PLAN_wallet_policy_composer.md
grep -n 'unconditional\|twenty-six' design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md
grep -n 'MsDescriptor::Wsh(w)\|MsDescriptor::Sh(sh)' design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md

# round-1 report (read for context only; its grades are not re-derived)
cat design/agent-reports/composer-S0-plan-R0-r1-fold-verification.md
```

Did not read any `.jsonl` file. Did not re-run `scripts/plan-build-gate-md.sh`; the build-gate/hand-check result stated in the dispatch brief (52 compose tests, 51 pass, 1 pinned red; clippy clean; md-cli compiles; whole md-cli suite by hand at 761/0/1) is accepted as already machine-checked and is not re-derived here.
