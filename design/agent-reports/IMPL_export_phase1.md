# IMPL — `PLAN_wallet_file_export.md` Phase 1 (`export-wallet --allow`)

**Repo:** `mnemonic-toolkit`
**Worktree:** `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/export-phase1`
**Branch:** `feat/export-allow-phase1` (off `master` @ `5f88071c`) — left in place, not merged, not pushed.
**Commits:** `0c672a4c` (pure refactor), `05ac190b` (implementation).

Status: **all Acceptance bullets implemented.** Three verification commands green.
Two findings about the plan are in §7; neither blocked the work.

---

## 1. Acceptance bullet 1 — every strict parse site, enumerated and annotated

I re-derived this rather than trusting the brief's table, and found **one more
production site** plus **four post-gate re-parses** it did not name.

| # | site (post-change line) | fn | arm(s) served | before | after |
| --- | --- | --- | --- | --- | --- |
| 1 | `cmd/export_wallet.rs:581` | `run` | `--descriptor` intake | strict | **LENIENT** |
| 2 | `cmd/export_wallet.rs:698` | `run` | `--descriptor` (script-type derive) | strict | **LENIENT** |
| 3 | `cmd/export_wallet.rs:899` | `run_from_import_json` | `--from-import-json` (script-type derive) | strict | **LENIENT** |
| **G1** | `cmd/export_wallet.rs:763` | `run` | **`--descriptor` AND `--template` AND `--slot`** | — | **ADMISSION GATE** |
| **G2** | `cmd/export_wallet.rs:1004` | `run_from_import_json` | **`--from-import-json`** | — | **ADMISSION GATE** |
| 4 | `wallet_export/pipeline.rs:39` | `build_descriptor_string` | `--template`/`--slot` canonicalization | strict | **strict, deliberately KEPT** |
| 5 | `wallet_export/bitcoin_core.rs:53` | `format_bitcoin_core_importdescriptors` | post-gate, all arms | strict | **LENIENT** |
| 6 | `wallet_export/pipeline.rs:192` | `descriptor_to_bip388_wallet_policy` | post-gate, all arms | strict | **LENIENT** |
| 7 | `wallet_export/bsms.rs:107` | `BsmsEmitter::emit` (4-line) | post-gate, all arms | strict | **LENIENT** |
| 8 | `wallet_export/green.rs:57` | `GreenEmitter::emit` (taproot probe) | post-gate, all arms | strict | **LENIENT** |

`export_wallet.rs:1313` is a test, not production — confirmed (the `#[cfg(test)]`
boundary is at `:1050`).

Sites 5–8 are exactly F-2's category: *"every downstream re-parse then becomes a
lenient parse of an already-admitted string."* The plan states the principle but
the Acceptance list names only `:524`. Leaving them strict would have failed the
`tr` export-with-flag baseline at `bitcoin_core.rs` and the bip388 keyless-leaf
vector at `pipeline.rs` — a strict re-parse downstream of the gate is a second
admission point with `tr`-only reach and no `--allow` awareness.

**Site 4 is the finding, and I did NOT gate it.** See §7.1.

### Why sites 2 and 3 had to move too

The Acceptance bullet names `:524` only, but the gate sits *downstream* of `:640`
and `:826`. A strict parse at either would refuse a sigless `tr` before the gate
could waive it — a second, `--allow`-deaf admission point in front of the real
one. Making all three lenient is what "the intake gate is the only admission
point" actually costs in code.

---

## 2. What changed, file by file

**`crates/mnemonic-toolkit/src/descriptor_builder/allow.rs`** — NEW. Two halves:

- *Moved verbatim* (commit `0c672a4c`, pure refactor): `CliAllow` + `kind()`/`kebab()`,
  `allow_set()`, `emit_allow_notes()` from `cmd/build_descriptor.rs`;
  `AllowSet::to_ext_params()` from `descriptor_builder/gate.rs`.
- *New* (commit `05ac190b`): `export_enforces()` (ruling (b) as an exhaustive
  match — a sixth rule breaks compilation), `descriptor_has_sigless_branch()`
  (per-wrapper fired-detection: per-leaf for `tr`, top-level for `wsh`/`sh`/`bare`,
  structurally-false for `pkh`/`wpkh`), `export_admission_gate()`, and
  `emit_export_allow_notes()` (export wordings). Plus 10 unit tests.

**`crates/mnemonic-toolkit/src/parse_descriptor.rs`** — added
`parse_descriptor_lenient()`. Goes through miniscript's public
`expression::FromTree` instead of `FromStr`: same tree walk, same key parsing,
**same BIP-380 checksum verification** (`expression::Tree::from_str` does it), minus
the tapleaf `ext_check(&ExtParams::sane())` that `FromStr` runs on `tr()` and
nothing else. Malformed input and bad checksums still error — pinned by two unit tests.

**`crates/mnemonic-toolkit/src/cmd/export_wallet.rs`** — `--allow` arg; sites 1–3
lenient; gates G1/G2 with the note printer at each; a module-doc table of the
per-arm topology.

**`wallet_export/{bitcoin_core,bsms,green}.rs` + `pipeline.rs`** — post-gate
re-parses made lenient; `pipeline.rs:39` gets a comment recording why it stays strict.

**`CHANGELOG.md`** — new `[Unreleased]` section with **both** behaviour-change lines
plus a Migration block (round-5 caught the singular).

**`docs/manual/src/40-cli-reference/41-mnemonic.md`** — `--allow` row plus a
"Reviewed sanity opt-out" subsection including the measured target table and an
explicit *"`restore` is not gated"*. Diff is `54 insertions(+), 0 deletions` — a
pure insertion, verified with `git diff | grep '^-'`. (My first pass silently
dropped the `--from-import-json-index` row; the no-deletions check caught it and
it was restored before commit.)

**`tests/cli_export_wallet_allow.rs`** + `tests/fixtures/export_wallet_allow/` — NEW
(the two rcw descriptors, two hand-built `--from-import-json` envelopes, and the
wsh md1 card for the `restore` invariant).

---

## 3. The surface, as it actually prints

```
$ mnemonic export-wallet --descriptor <rcw wsh> --format bitcoin-core
error: export-wallet: this wallet has a spend path that requires no signature
(anyone-can-spend); rerun with --allow sigless-branch after review. The flag
permits EMISSION of the wallet file — it does not make any wallet application
accept it.                                                              [exit 2]

$ … --allow sigless-branch
WARNING: sanity rules OVERRIDDEN by --allow and FIRED: sigless-branch. This
wallet has a spend path that needs no signature — anyone who learns the
descriptor can move the funds. You asked to emit a watch-only file for it
anyway, after review.                                                   [exit 0]

$ … --allow malleable            (rule NEVER RAN — no verdict claimed)
note: --allow malleable has no effect on export-wallet — only sigless-branch is
enforced here; the descriptor was NOT checked against malleable

$ … --allow sigless-branch       (on a SANE wallet — rule RAN, verdict earned)
note: --allow sigless-branch was requested but did not fire (the descriptor
passes that rule without it)
```

R3-2 holds structurally: `emit_export_allow_notes` reaches the "passes that rule"
branch only through `export_enforces(*a) == true`, so an unenforced rule cannot
print it. Asserted per unenforced rule at both the unit and CLI levels, and the
mutation that removes the distinction is caught (M4/M5 in §4).

---

## 4. Test list and results

**25 integration** (`tests/cli_export_wallet_allow.rs`) + **10 unit**
(`descriptor_builder::allow::export_allow_tests`). All 35 pass.

| test | Acceptance bullet |
| --- | --- |
| `flagless_sigless_wsh_now_refuses` | baseline: flagless refusal, wsh (**breaking**) |
| `flagless_sigless_tr_refuses_and_names_the_flag` | baseline: flagless refusal, tr |
| `sigless_wsh_exports_with_the_flag` | baseline: export-with-flag |
| `sigless_tr_exports_with_the_flag` | baseline: export-with-flag; also pins `:524` leniency |
| `fired_warning_speaks_the_export_act_not_the_authoring_act` | fired warning + M2 export wording |
| `requested_but_not_fired_note_on_a_sane_wallet` | requested-not-fired note |
| `from_import_json_sigless_wsh_refuses_flagless` | R3-1 hole closed |
| `from_import_json_sigless_wsh_exports_with_the_flag` | envelope gated like `--descriptor` |
| `from_import_json_taproot_stays_refused_by_fix_alpha_with_or_without_allow` | `Fix-α` untouched |
| `requesting_one_rule_admits_no_other` | I4 over-admission (2 arms × 4 rules) |
| `keyless_leaf_survives_the_bip388_transforming_emitter` | I4(b) transforming-emitter vector |
| `bip388_keyless_leaf_still_refuses_without_the_flag` | same, negative direction |
| `note_matrix_row1_allow_sigless_branch_every_arm` | matrix row 1, all 3 columns (4 cells) |
| `note_matrix_row2_unenforced_rule_every_arm` | matrix row 2, all 3 columns × 4 rules |
| `note_matrix_row2_columns_are_identical` | **the uniform gate's observable signature** |
| `passes_that_rule_is_never_printed_for_a_rule_that_did_not_run` | R3-2, per unenforced rule, 3 arms |
| `restore_md1_sigless_wsh_still_emits_flagless` | `restore` out of scope |
| `restore_has_no_allow_flag` | `restore` gains no flag |
| `help_text_never_claims_it_enables_import_into_a_wallet_app` | the surviving constraint |
| `all_five_rule_names_parse_on_the_export_surface` | shared vocabulary |
| `the_sane_control_is_actually_sane` | positive control for row 2 |
| `every_format_meets_the_same_gate_before_its_own_verdict` | **ONLY admission point** (11 formats) |
| `with_the_flag_each_format_falls_through_to_its_own_verdict` | gate is not a format filter |
| `sparrow_descriptor_passthrough_refusal_is_untouched_for_a_sane_wallet` | PLAN §2 incidental safety |
| `rust_first_vectors_for_the_new_refusals` | Open Q4 Rust-first vectors |
| *unit:* `export_enforces_exactly_sigless_branch` | ruling (b) as a partition |
| *unit:* `lenient_parse_admits_a_keyless_tapleaf_that_strict_parse_refuses` | leniency is real and scoped |
| *unit:* `lenient_parse_still_rejects_malformed_input` | leniency ≠ permissive parsing |
| *unit:* `lenient_parse_still_verifies_the_bip380_checksum` | no transcription hole opened |
| *unit:* `sigless_detection_is_per_wrapper` | F-3, 13 vectors across 5 wrappers |
| *unit:* `gate_refuses_flagless_and_admits_with_the_flag` | gate, 3 wrappers |
| *unit:* `gate_admits_a_sane_descriptor_and_reports_nothing_fired` | did-not-fire reachable |
| *unit:* `allowing_another_rule_does_not_admit_a_sigless_branch` | I4 at unit level |
| *unit:* `unenforced_rules_never_claim_a_verdict` | R3-2 at printer level |
| *unit:* `export_and_build_warnings_are_distinct` | M2, pinned against the build wording |

**Row-2 column identity, as implemented:** the test picks the note line for each
rule from all three arms and asserts **byte-equality**, not substring containment,
so a divergence cannot hide in wording.

**One false PASS caught during the red phase.**
`passes_that_rule_is_never_printed_for_a_rule_that_did_not_run` passed against the
*unimplemented* code, because every invocation failed at clap and produced no notes
at all — so every "must not contain" was vacuously true. It now asserts `exit 0`
and asserts the note is actually printed before asserting what it must not say.

### Mutation check — 5 mutations, 5 caught

| mutation | caught by |
| --- | --- |
| M1 `tr` per-leaf detection → `false` | `flagless_sigless_tr_refuses_…`, `rust_first_vectors_…` |
| M2 `wsh` top-level detection → `false` | 8 tests |
| M3 gate 2 (`--from-import-json`) deleted | 5 tests |
| M4 unenforced rules reuse the build wording | 3 tests |
| M5 enforced-rule partition widened to all five | 3 tests |

---

## 5. The three verification commands — all run, all green

```
$ cargo fmt --all --check
(no output)                                                    exit 0

$ cargo clippy --locked --workspace --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s)
0 warnings, 0 errors
(forced a full re-check with `touch src/main.rs`; diagnostic count = 0)

$ cargo nextest run --locked --workspace
     Summary [  59.204s] 3928 tests run: 3928 passed, 19 skipped
```

Arithmetic: the pre-change baseline on this branch was **3893 passed**; I added
25 + 10 = 35 tests; 3893 + 35 = **3928**. No pre-existing test was lost, and none
was modified.

`cargo fmt` caught real drift twice — once after the refactor, once after the
implementation. The check is not decorative here.

Clippy also caught one real nit in my own test file (`needless_lifetimes`), fixed
before commit.

---

## 6. `restore` is unchanged — measured, not argued

The hard constraint, run before and after:

```
mnemonic restore --md1 …(15 chunks)… --format bitcoin-core
  BEFORE: exit 0, 2694 bytes, sha256 c121fb6ca9723e22489e58b04a82edd3ffccf92d7c13acf0472933c1f95e4b18
  AFTER:  exit 0, 2694 bytes, sha256 c121fb6ca9723e22489e58b04a82edd3ffccf92d7c13acf0472933c1f95e4b18
```

Byte-identical. Two tests pin it: one asserts exit 0 with **no** `--allow` note on
stderr *and* that restore's flagless stdout byte-matches `export-wallet`'s
**flagged** stdout for the same wallet (which is what proves restore was not gated,
rather than merely not crashed); the other asserts `restore --allow` is a clap
usage error.

The source-level argument for why relaxing the shared emitters cannot reach
restore: **every descriptor `restore` feeds to `emit_payload` has already passed a
strict `Descriptor::from_str`** —

- `restore.rs:2387` for the multisig arm (and its `--template` sub-arm goes through
  `build_descriptor_string`, site 4, which is still strict);
- `restore.rs:3254` for the non-template multisig arm (`template_opt.is_none()`);
- `restore.rs:2073` for single-sig.

So the strings restore hands to the emitters are strict-parseable by construction,
and a lenient re-parse of a strict-parseable string is the identical parse.
`cmd/restore.rs:2496` and `:2801` are untouched — `git diff` shows no change to
`restore.rs` at all.

---

## 7. Where the plan met the code

Two things. Neither blocked implementation; both are reported rather than
quietly absorbed.

### 7.1 There is a FOURTH production parse site — `wallet_export/pipeline.rs:39`

The brief's table lists three (`:524`, `:640`, `:826`) and asks me to report a
fourth rather than quietly gate it. There is one:
`build_descriptor_string()` at `wallet_export/pipeline.rs:39` strictly parses on
the **`--template`/`--slot`** arm.

**I left it strict, and it is not an admission point for `sigless-branch`.** It
canonicalizes the *builder's own* output, and every `export-wallet` template is a
`pk` / `multi` / `multi_a` / `sortedmulti` quorum, so it cannot produce a sigless
branch — which is exactly the plan's own reasoning for why the `--template` cell
of the matrix carries only the did-not-fire note. The uniform gate still runs on
its result and simply never fires there (asserted by the row-1 column-3 test).

Two reasons not to relax it, either sufficient on its own:

1. It is also **`restore`'s** canonicalizer. Relaxing it is a `restore` behaviour
   change, and Phase 1 makes none.
2. Under a maximally literal reading of "(b), enforced uniformly", the other four
   rules should not run here either — so a `--template tr-multi-a` with a
   duplicated cosigner xpub, which this site refuses today, would start emitting.
   That is a fifth new behaviour, on the arm the plan explicitly describes as
   emitting-only-the-note. **Nobody ruled on it**, so I did not do it.

The rationale is recorded at the site so it outlives this report. **If a reviewer
wants this arm relaxed too, that is a decision with its own release note and its
own `restore` impact analysis — it is not a follow-up.**

### 7.2 The second behaviour change is wider than "the intake becomes lenient" sounds

The plan is right that `:524` going lenient is an operator-visible change needing
its own release note. Worth stating what it *is*, because the phrasing understates
it: because the gate re-enforces only `sigless-branch`, making the intake lenient
means **`export-wallet` stops running miniscript's other four sanity rules on `tr()`
leaves**. A taproot descriptor that is malleable, repeats a key, mixes height/time
timelocks or exceeds resource limits used to be refused at `:524` and is now emitted.

This follows directly from composing (b) with uniform enforcement — it is the *same*
asymmetry-removal the plan chose, running in the permissive direction rather than
the restrictive one, and the plan does name it as one of the two changes. I wrote it
out explicitly in the CHANGELOG and the Migration block rather than leaving a reader
to derive it. Flagging it here because it is the kind of consequence a reviewer
should see stated once in plain words.

---

## 8. Plan claims I re-measured, which held

- `Descriptor::from_str` runs sanity on `tr` **only**: confirmed in the pinned
  source at `descriptor/mod.rs:1138-1149` (rev `ff4732e`) —
  `if let Descriptor::Tr(…) = ret { ret.sanity_check()?; for item in inner.leaves()
  { …ext_check(&ExtParams::sane())? } }`. Upstream's own comment calls it
  *"weird/broken behavior from 12.x"* (rust-miniscript issue #734).
- `export-wallet --descriptor <rcw wsh> --format bitcoin-core` really did exit 0 with
  **2694 bytes** pre-change, and `<rcw tr>` really did exit 2 with *"All spend paths
  must require a signature"*. Both re-measured on this branch before touching anything.
- A **sigless `wsh` envelope really did exit 0 flagless** on `--from-import-json`
  pre-change — the R3-1 hole is real, not hypothetical, and I have the fixture that
  reproduced it.
- The 11-format sweep confirms the gate precedes every per-format verdict, so the
  "ONLY admission point" claim is behaviourally asserted, not just structurally argued.
- `WshInner` no longer exists at this miniscript rev (PR #915 moved `SortedMulti` into
  a `Terminal`), so `Wsh::as_inner()` returns the `Miniscript` directly and `ShInner`
  has three variants. The per-wrapper detector matches the real API, not the 12.x one.

---

## 9. Not done (out of scope, flagged)

- **No Go port.** The Rust-primary rule makes this repo the leading side, and the
  fork has no `export-wallet` counterpart; the new refusal semantics are pinned here
  with vectors (`rust_first_vectors_for_the_new_refusals`) so a future port has
  something to converge on.
- **No version bump.** The CHANGELOG entry is `[Unreleased]`; cutting `0.98.0` is a
  release decision.
- **No `[profile.test] opt-level = 2`.** The constellation CLAUDE.md recommends it and
  this workspace has no `[profile.*]` at all, but it is a repo-wide config change
  belonging in its own commit, not this one.
- **`Fix-α` untouched**, as instructed. The taproot-envelope test asserts the refusal
  is Fix-α's own message specifically so nobody "fixes" a red test by relaxing it.
- **Not merged, not pushed.** The worktree and branch are left in place.
