# Composer Stage 0b Implementation Plan — `md compose --preset` and the six archetype vectors

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close follow-up F-453. `md_codec::compose::presets` (composer Stage 0, shipped) already has the six archetype constructors, but `md compose` has no way to reach them and the corpus has no vector pinning any of them. Give `md compose` a `--preset <name>[,<param>=<value>...]` surface that calls those constructors directly, and export ONE `MANIFEST` vector per archetype so the fork's Stage 3 GUI preset picker has a Rust byte oracle to build against and check itself with — Rust first, nothing normative in Go (CLAUDE.md Rust-primary rule).

**Architecture:** `--preset` is a thin CLI-layer parser (`crates/md-cli/src/cmd/compose.rs`) over the SAME six functions `md_codec::compose::presets` already exports; it introduces no new `md_codec` types or lowering behaviour. The six new `MANIFEST` entries are built the same way: `family()`'s tuple's `PathList` element is produced by CALLING the preset constructor (not by hand-reconstructing the path list), while its expected-template field stays a hand-typed literal — the same drift-detection shape every other `family()` row already uses (`crates/md-codec/tests/compose_support.rs:207-301`), so a constructor whose parameter order or internal default drifts fails the SAME generic tests the 26 existing compose vectors already run under (`compose_vectors.rs`, `compose_crosscheck.rs`) with zero new test code.

**Tech Stack:** Rust 2024 edition workspace at `/scratch/code/shibboleth/descriptor-mnemonic`; crates `md-codec` (lib) and `md-cli` (binary `md`, clap 4.5 derive, features `json` default and `cli-compiler`); `cargo nextest run --locked`; `cargo fmt --check`; `cargo clippy -D warnings`.

**Spec:** `design/SPEC_wallet_policy_composer.md` §4d (the six presets and their parameters, C2), §5 (the lowering the presets feed), §10 item 3 (Rust host work: "the five presets as Concrete policies + expected templates" — already done in S0 Task 7; this plan is the CLI surface and the corpus half), §12 item 1 (tagged coverage; the tag vocabulary lives in `crates/md-codec/tests/compose_support.rs`). Follow-up: `design/FOLLOWUPS.md` F-453 (`composer-preset-vectors-missing`), filed 2026-09-02 from `composer-S2-plan-R0-r0-fidelity` M-5.

**Baseline revision (for `scripts/plan-staleness-check.sh`):** descriptor-mnemonic `66bdf2f4` (composer S0 shipped to `main`; `git status --short` clean at that revision when this plan was authored). Every `path:line` citation below was read at this revision.

**STATUS: R0 round 0 folded and round-1-verified with one Important returned; that Important is now folded too, not yet re-verified.** Round 0: fidelity+design lens (opus, `design/agent-reports/composer-S0b-plan-R0-r0-fidelity.md`, 0C/4I/4M/2N) and tests lens (`design/agent-reports/composer-S0b-plan-R0-r0-tests.md`, 0C/0I/0M/2N, all 12 mutations caught), both folded (I-1..I-4, M-1..M-4, N-1/N-2 fidelity, N-1/N-2 tests). Round 1: fold-verification lens (`design/agent-reports/composer-S0b-plan-R0-r1-fold-verification.md`, 0C/1I/1M) VERIFIED I-1..I-4 and M-1..M-3 live (including the name-first mutation and all ten legacy pairs), and found the round-0 fold of M-4 was a TAUTOLOGY: the added test asserted a hardcoded fixture's `.len() == 6` rather than iterating `PRESET_NAMES`, and a live mutation (a 7th, unmatched `PRESET_NAMES` entry) compiled, passed clippy, passed all 31 tests, and then PANICKED on a real CLI invocation (`unreachable!()`, exit 101) — worse than the original finding's failure mode. Folded here: the coverage test now iterates `PRESET_NAMES` itself and calls `parse_preset` directly (moved to a `#[cfg(test)] mod tests` unit test embedded in `compose.rs`, since `md-cli` ships no library target and the black-box `cli_compose_preset.rs` cannot reach either); the `match`'s `other` arm returns a preset-naming `CliError::Compose` instead of `unreachable!()`, so the CLI cannot panic on this table even mid-drift. Both fixes re-confirmed against the SAME mutation: the test now fails with a named message, and the CLI now exits 1 gracefully instead of panicking. The Minor (M-3's wording paraphrase) is also folded: `need_after_height`'s message now reads "reads as a block height", matching `--path`'s own wording verbatim rather than paraphrasing it. Re-verify per the build-gate note below before the next round; do not begin implementation before this plan reaches 0 Critical / 0 Important per `CLAUDE.md`'s R0 gate.

## What is already machine-verified (reviewer budget goes elsewhere)

Every claim in this plan that a tool can check was checked against a scratch copy of descriptor-mnemonic at `66bdf2f4` (toolchain 1.85.0, the repo's pin) before this plan was written, not derived from reading the lowering rules by hand:

- The six `presets::*` signatures, their `ComposeError` variants, and every refusal wording quoted below (`LegacyWrapperShape`, `LockOutOfRange`, `PresetShape`, `BadThreshold`) are read verbatim from `crates/md-codec/src/compose/mod.rs` and `crates/md-codec/src/compose/presets.rs` at `66bdf2f4` and additionally exercised through a throwaway example binary (`cargo run -p md-codec --example`) that printed each `Display` string live.
- Every one of the six preset templates this plan pins (Task 1's family rows, Task 2's CLI tests) was produced by actually running the shipped `target/debug/md compose --wrapper ... --path ...` (the pre-`--preset` equivalent of what each archetype builds) and, separately, `md encode` → `md decode` → `md address` round trips with the four journey xpubs bound — not hand-derived from the lowering table. All six round-tripped byte-identically and one (`kofn_recovery` under `tr`) derived a real `bc1p...` address.
- Every file this task creates or replaces (`crates/md-cli/src/cmd/compose.rs`, `crates/md-cli/tests/cli_compose_preset.rs`, `crates/md-codec/tests/compose_support.rs`) was written into a full scratch copy of the workspace and BUILT: `cargo build -p md-codec --all-targets` and `cargo build -p md-cli --all-targets` both succeed (the latter WITH `main.rs` hand-wired per Task 2 Step 4 — see the build-gate note below); `cargo nextest run -p md-codec -E 'binary(/^compose_/)'` is 52/52 green (no pinned red — the scratch copy had a hand-wired `test_vectors.rs`, exactly the fragment the real build gate does not assemble); `cargo nextest run -p md-cli -E "binary(/^cli_compose/) + test(every_preset_name_parses_with_some_valid_parameters)"` is 31/31 green (9 pre-existing + 22 new across the original draft and both R0 folds, zero regressions; the filter now spans two binaries since the R0 round-1 fold moved one test into a `#[cfg(test)]` unit test in `compose.rs`, run under `bin/md`, because the black-box `cli_compose_preset.rs` cannot call `parse_preset` or read `PRESET_NAMES` directly); `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features -- -D warnings` are both clean on both crates. The corpus exporter was run for real: `md vectors` wrote exactly 30 new files (six vectors × five files each: `.bytes.hex`, `.phrase.txt`, `.descriptor.json`, `.template`, `.conformance.json`), the pre-existing `vectors_output_matches_committed_corpus` drift test went from failing ("drift detected") to passing once those files were written, and the full `-p md-cli` suite is 783/783 (761 pre-existing + 22 new). `-p md-codec --workspace` is unaffected at 1318/1318 as measured at HEAD before this plan's changes (the six new tuples in `family()` are DATA consumed by existing `#[test]` functions, not new test functions, so the md-codec test count does not move).
- **`scripts/plan-build-gate-md.sh`, run for real against THIS plan file, does NOT complete unmodified — a genuine finding, not a re-derivation.** Steps 1-5 (scratch copy, extraction, md-codec build, the 52 compose tests with exactly the one pinned red, clippy) all pass exactly as predicted. Step 6 (`cargo test -p md-cli --locked --no-run`) FAILS to compile in the bare scratch copy, and the script's `set -euo pipefail` aborts on it — NOT a graceful "not covered", an actual halt. Cause: this is the first composer plan to CHANGE an EXISTING function's signature (`cmd::compose::run` gains a `preset: Option<&str>` parameter) that a live, un-hand-wired `main.rs` fragment already calls with the OLD 4-argument arity — S0's own Task 6 only ever ADDED a brand-new call site, so its bare gate runs never hit this class of break. Confirmed by hand-wiring ONLY Task 2 Step 4's `main.rs` diff into the same scratch copy afterward: `cargo test -p md-cli --locked --no-run` then compiles cleanly with zero errors. **Consequence for this plan's own build gate, stated plainly: run `scripts/plan-build-gate-md.sh` for steps 1-5 as-is, then hand-wire Task 2 Step 4's `main.rs` diff into the SAME scratch copy (`${TMPDIR:-/tmp}/plan-build-gate-md`) before judging step 6** — exactly the same hand-wiring S0's own Task 8/9 folds already needed for their `parse/template.rs` and CLI fragments. This is not a script bug to fix (the script correctly refuses to guess at fragment content); it is a real limitation this plan's authoring surfaced by actually running the gate rather than assuming reuse from reading the script's source alone.
- `crates/md-cli/tests/cmd_gui_schema.rs` and `crates/md-cli/tests/cli_output_class.rs` were grepped for the string `compose` and neither references it (`grep -n compose` on both returns nothing) — adding `--preset` introduces no golden-schema fixture to update, unlike the caution S0 Task 6 recorded for its own `--wrapper`/`--path` addition.
- The fork's `scripts/vendor-compose-vectors.sh` (`/scratch/code/shibboleth/seedhammer`, `main` at `321acb5`) globs `^(keyed_)?compose_` — `keyed_compose_preset_*` matches with NO script change, confirmed by reading the script, not assumed. The fork's `md/compose_vectors_pin_test.go` hardcodes both a 26-name list and the literal `126` file count (`:83-84`); both are fork-side follow-on work this plan does not touch (see Task 3).

**Correction to the F-453 filing:** the follow-up's phrasing "mnemonic-engrave carries that script for the fork" is not accurate — `scripts/vendor-compose-vectors.sh` lives ONLY in the fork repo (`seedhammer/scripts/vendor-compose-vectors.sh`), not in mnemonic-engrave (confirmed: `find /scratch/code/shibboleth/mnemonic-engrave -iname vendor-compose-vectors.sh` finds nothing). This plan does not touch the fork either way — Task 3 only names the follow-on work.

## Global Constraints

- **Rust first:** nothing in this plan touches the fork or any Go file (CLAUDE.md Rust-primary rule). The fork's re-vendoring is named in Task 3 as a pointer only.
- **No wire-format change, no new `md_codec` types, no new lowering rule.** `--preset` is a CLI-layer parser over the SIX ALREADY-SHIPPED `presets::*` functions (`crates/md-codec/src/compose/presets.rs`, S0 Task 7); the vector additions are new `PathList`s built by CALLING those functions, never new path-list-construction logic.
- **Exit codes, unchanged from S0:** `main` prints `md: {e}` and exits 1 for every `CliError` except `BadArg`, which exits 2 (`crates/md-cli/src/main.rs:806-819`). A `--preset` grammar error (unknown name, missing/extra parameter) is `CliError::Compose` — exit 1, the SAME class `--path` grammar errors already use — not a new `BadArg`. The ONE exit-2 case this plan adds is `--path`/`--preset` both given or NEITHER given, which is `clap`'s own `ArgGroup` refusal (measured: `error: the argument '--path <PATH>' cannot be used with '--preset <PRESET>'` / `error: the following required arguments were not provided: <--path <PATH>\|--preset <PRESET>>`, both exit 2 — clap's own arg-parsing errors always exit 2, unrelated to `CliError`).
- **MSRV per `rust-toolchain.toml`** (1.85.0, unchanged since S0); `cargo nextest run --locked`; `cargo fmt --check`; `cargo clippy -D warnings`; stage paths explicitly (no `git add -A`).
- **S0's lessons, still binding:** the renderer omits origins (`descriptor_to_template` writes `@0/<0;1>/*`; `template_with_origins` inlines them — every MANIFEST `template` field and every family-row literal below is the WITH-origins form); `force_chunked: true` on every new MANIFEST entry (S0 Task 5's measured reason — the smallest new entry, `keyed_compose_preset_simple_timelocked_inheritance` at two slots, is smaller than the smallest S0 entry that needed it, but this plan follows the SAME rule S0 states rather than re-deriving a new boundary: "measured at implementation: the smallest keyed one... is already 131 data symbols against the codex32 regular code's 80 cap" — re-verified directly in this plan's own scratch run, where the exporter wrote `.conformance.json` for all six without complaint at `force_chunked: true`); `--bin md` when a workspace target is ambiguous (md-cli builds two binaries).
- **Build gate — REUSED script, but its bare run needs one extra manual step for THIS plan, discovered by actually running it (see "What is already machine-verified" below).** `scripts/plan-build-gate-md.sh design/IMPLEMENTATION_PLAN_composer_S0b_presets.md` (mnemonic-engrave) needs NO script edits. Its `ok` anchor regex already covers every gate-assembled file this plan touches (`crates/md-codec/tests/compose_support.rs` matches `crates/md-codec/tests/compose_[A-Za-z0-9_]+\.rs$`; `crates/md-cli/src/cmd/compose.rs` and `crates/md-cli/tests/cli_compose_preset.rs` match their own listed patterns verbatim), and its pinned-red acceptance (`every_compose_vector_in_the_manifest_is_exactly_what_compose_renders` failing with `MANIFEST lacks`) is generic over vector NAME. Steps 1-5 (scratch copy, extraction, md-codec build, the 52 compose tests with the one pinned red, clippy) run unmodified and green, verified live. Step 6 (`cargo test -p md-cli --locked --no-run`) does NOT pass bare — it HALTS the script (`set -euo pipefail`) with a real compile error, because this plan changes `cmd::compose::run`'s arity and the un-hand-wired `main.rs` fragment still calls the old one; S0's own Task 6 never hit this because it only ever added a brand-new call site. **Fold procedure: run the script for steps 1-5, then hand-wire Task 2 Step 4's `main.rs` diff into its scratch copy before judging step 6** — the same extra step S0's own Task 8/9 folds already needed for their fragments, verified here to make step 6 compile cleanly.
- Commits: stage paths explicitly; one task, one commit; messages end with the standing trailers.

---

## File Structure

| file | change | gate-assembled? |
| --- | --- | --- |
| `crates/md-codec/tests/compose_support.rs` | Replace (whole file): `presets` import, six new `family()` rows, six `preset:<name>` entries in `SINGULAR_TAGS` | yes (`crates/md-codec/tests/compose_[A-Za-z0-9_]+\.rs$`) |
| `crates/md-codec/src/test_vectors.rs` | Add (fragment): six `Vector { .. }` entries appended to `MANIFEST` | no — hand-wired at the gate, exactly like S0 Task 5 |
| `crates/md-cli/src/cmd/compose.rs` | Replace (whole file): `PresetParams`, `parse_preset`, `parse_kofn`, `parse_sha256_hex` (factored out of the existing `parse_path` sha256 arm), `hex32`, `preset_params_json`, `run`'s new `preset: Option<&str>` parameter | yes (matches its own listed pattern) |
| `crates/md-cli/tests/cli_compose_preset.rs` | Create | yes (matches `crates/md-cli/tests/cli_compose[A-Za-z0-9_]*\.rs$`) |
| `crates/md-cli/src/main.rs` | Add (fragment): `#[command(group = ...)]` on the `Compose` variant, a new `preset: Option<String>` field, `paths` loses its own `required = true` (enforced by the group instead), the dispatch arm passes `preset.as_deref()` | no — fragment, same as S0 Task 6 |
| `CHANGELOG.md` | Add: one line each under `## md-cli [Unreleased]` and `## md-codec [Unreleased]` | no |

No change to `crates/md-cli/src/error.rs` (the `Compose(String)` variant already exists from S0 Task 6 and covers every new refusal), `crates/md-codec/src/compose/{mod,presets}.rs` (the six constructors and every type they need already ship), `crates/md-codec/tests/compose_crosscheck.rs` or `crates/md-codec/tests/compose_vectors.rs` (both already iterate `family()`/`MANIFEST` generically — see Task 1).

---

### Task 1: The six preset vectors — `family()`, tags, `MANIFEST`, the exporter

**Files:**
- Replace: `crates/md-codec/tests/compose_support.rs`
- Add to: `crates/md-codec/src/test_vectors.rs`
- Test (pre-existing, unchanged): `crates/md-codec/tests/compose_vectors.rs`, `crates/md-codec/tests/compose_crosscheck.rs`

**Interfaces:**
- Consumes: `md_codec::compose::presets::{plain_multisig, simple_timelocked_inheritance, kofn_recovery, tiered_recovery, hashlock_gated, decaying_multisig}` (all shipped, `crates/md-codec/src/compose/presets.rs:28-140`); `family()`'s existing tuple shape `(&'static str, PathList, String, Vec<&'static str>)` (`crates/md-codec/tests/compose_support.rs:207`); `SINGULAR_TAGS: &[&str]` (`crates/md-codec/tests/compose_support.rs:306`).
- Produces: six new `family()` rows and six new `MANIFEST` entries; no new function, no new test.

**Default parameters (decided here) — R0 fidelity I-1's ruling: these are BOTH the vectors' parameters AND the device's offered default shape for each archetype, not fixture numbers that happen to land on screen.** SPEC §4d says presets "POPULATE a path list the operator then edits", and `IMPLEMENTATION_PLAN_composer_S3_fork_gui.md` Task A10 makes that binding: each Go preset entry's `list` is "transcribed from the primary's exported vector's `descriptor.json` path list… **Do not invent a shape here**" (`:4723`) — so whatever this plan picks IS what the operator sees first, by construction, not by accident. Every free parameter is `2-of-3` and `older=26280` — the journey's own canonical values, already used repeatedly in `family()` (`keyed_compose_wsh_two_path_or_d`, `keyed_compose_tr_two_path_nums`) and in S0's own `cli_compose.rs` example test. `tiered_recovery` is `2of2,1of2` and `decaying_multisig` is `2of2,1of1` — the SMALLEST legal shape of each archetype, and that smallness is a UX choice, not a test-fixture one: a preset is a starter the operator edits (§4d), so the smallest legal shape is the honest starter — a wider tier is one edit away, and a preset that starts wider than it needs to asks the operator to narrow it, which is backwards for a starter. That both shapes also fit inside the four journey xpubs every `keyed_compose_*` vector is bound by (`compose_vectors.rs`'s `keyed_compose_vectors_bind_at_most_the_four_journey_keys`) is a convenience of this choice, not its reason — if a later ruling decides an archetype's default should be wider, the FIXTURE is widened (more journey xpubs), never the archetype narrowed to fit it. `older1=13140` is half of `older2=26280`, and `after=1_000_000` reuses `keyed_compose_wsh_three_paths`'s own `AfterHeight` value — both chosen to land on already-used, already-cross-checked numbers rather than arbitrary ones. `hashlock_gated` and `simple_timelocked_inheritance` take no key-count parameter at all (always 1-of-1 on each side) so only `older=26280` is free there.

**R0 fidelity I-2 — one wrapper per archetype, not a wrapper cross-product.** `kofn_recovery`'s vector is `Wrapper::Tr`; the other five are `Wrapper::Wsh`. F-453 asks for one vector per ARCHETYPE (`mnemonic-engrave/design/FOLLOWUPS.md:15411`), not one per (archetype, wrapper) pair, and this plan does not export twelve vectors to get both: the wrapper is a parameter of the `PathList` the operator or device picks, not an axis of the archetype itself, and a single vector already pins what does not vary by wrapper — the archetype's PARAMETER ORDER and LOWERING. `IMPLEMENTATION_PLAN_composer_S3_fork_gui.md` Task A10 currently drafts `TestComposerPresetsReproduceTheirVendoredVectors` as one loop over `composerPresets(md.ComposeWsh)` expecting all six presets to have a WSH vendored vector (`:4717`); since `kofn_recovery`'s is `Tr`, that loop needs narrowing to "each preset that HAS a vector at this wrapper, with the wrapper named in the assertion". That correction lands in the S3 plan, scheduled by the controller after S3's own r1 verification — NOT made here, since this plan touches only descriptor-mnemonic.

- [ ] **Step 1: Add the family rows without the MANIFEST entries; run to see the expected, PINNED-shape failure**

Replace `crates/md-codec/tests/compose_support.rs` in full:

```rust
//! Helpers shared by the compose integration tests. Included with
//! `#[path = "compose_support.rs"] mod support;` from `compose_crosscheck.rs`
//! and `compose_vectors.rs`; cargo also compiles this file as a test binary of
//! its own (with no tests), hence the allows: the workspace lints `pub` items
//! for docs, and an unused helper in one includer is dead code in that binary.
#![allow(dead_code, missing_docs)]

use std::str::FromStr;

use md_codec::compose::{Composed, PathList, SlotOrigin, compose, compose_with};
use md_codec::encode::Descriptor;
use md_codec::origin_path::{OriginPath, PathComponent};
use md_codec::tag::Tag;
use miniscript::bitcoin::bip32::{ChildNumber, Xpub};
use miniscript::bitcoin::secp256k1::Secp256k1;

/// The wallet-policy journey's four cosigners: one master (73c5da0a) at
/// m/48'/0'/{0..3}'/2'. Real public keys; nothing here is a secret.
pub const XPUB: [&str; 4] = [
    "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf",
    "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk",
    "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR",
    "xpub6E6Z3Ss5TXJYNJp4U1q3NZ3pCn82i7KXQAKUtNnzLJ3cCdchQeSdFvXemizaHUF7wNwRQAB8mPdoZhGHLiv49cWPtCnoJY3Az3E8JKxH9Mq",
];
pub const FP: [u8; 4] = [0x73, 0xc5, 0xda, 0x0a];
pub const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";
pub const H: [u8; 32] = [0xa8; 32];
pub const HH: &str = "a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8";

pub fn hardened(values: &[u32]) -> OriginPath {
    OriginPath {
        components: values
            .iter()
            .map(|v| PathComponent {
                hardened: true,
                value: *v,
            })
            .collect(),
    }
}

/// `n` DISTINCT xpubs: the journey's four for the first four slots, then
/// unhardened children of the first one. BIP-32 makes a (fingerprint, path)
/// pair name exactly ONE key, so binding one xpub at two declared origins would
/// describe an impossible wallet (descriptor-mnemonic F-217,
/// `corpus_origin_consistency.rs`); a 32-slot policy therefore needs 32 keys.
pub fn slot_xpubs(n: usize) -> Vec<Xpub> {
    let secp = Secp256k1::verification_only();
    let base = Xpub::from_str(XPUB[0]).expect("fixture xpub parses");
    (0..n)
        .map(|i| {
            if i < XPUB.len() {
                Xpub::from_str(XPUB[i]).expect("fixture xpub parses")
            } else {
                let child = ChildNumber::from_normal_idx(i as u32).expect("small index");
                base.derive_pub(&secp, &[child])
                    .expect("unhardened derivation")
            }
        })
        .collect()
}

/// 65 wire bytes (chain code ‖ compressed point).
pub fn xpub_bytes(x: &Xpub) -> [u8; 65] {
    let mut out = [0u8; 65];
    out[..32].copy_from_slice(&x.chain_code[..]);
    out[32..].copy_from_slice(&x.public_key.serialize());
    out
}

/// Seat every slot at `m/48'/0'/<slot>'/T'` under one master fingerprint and
/// bind distinct xpub bytes: a KEYED descriptor the converter can derive.
pub fn keyed(list: &PathList) -> Descriptor {
    let unseated = compose(list).expect("list is composable");
    let n = unseated.slots.len();
    let declared: Vec<Option<SlotOrigin>> = (0..n)
        .map(|i| {
            Some(SlotOrigin {
                origin: hardened(&[48, 0, i as u32, list.wrapper.script_type()]),
                fingerprint: Some(FP),
            })
        })
        .collect();
    let mut c = compose_with(list, &declared).expect("declared origins compose");
    let xs = slot_xpubs(n);
    c.descriptor.tlv.pubkeys = Some(
        xs.iter()
            .enumerate()
            .map(|(i, x)| (i as u8, xpub_bytes(x)))
            .collect(),
    );
    c.descriptor
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// The rust-miniscript CONCRETE policy with the same spend conditions as
/// `list`, over `keys` (one key string per emitted slot, in slot order). This is
/// the compiler's input for the §5b lift-equality leg; it is built from the path
/// list, not from the lowered tree, so it cannot inherit a lowering defect.
pub fn concrete_policy(list: &PathList, c: &Composed, keys: &[String]) -> String {
    let mut paths: Vec<String> = Vec::new();
    for (pi, p) in list.paths.iter().enumerate() {
        let mut parts: Vec<String> = Vec::new();
        if let Some(ks) = p.keys {
            let pks: Vec<String> = c
                .slots
                .iter()
                .filter(|slot| slot.path == pi)
                .map(|slot| format!("pk({})", keys[usize::from(slot.index)]))
                .collect();
            parts.push(if ks.n == 1 {
                pks[0].clone()
            } else {
                format!("thresh({},{})", ks.k, pks.join(","))
            });
        }
        if let Some(h) = p.hash {
            parts.push(format!("sha256({})", hex(&h)));
        }
        if let Some(lock) = p.lock {
            let (tag, v) = lock.operand().expect("validated");
            let name = if matches!(tag, Tag::Older) {
                "older"
            } else {
                "after"
            };
            parts.push(format!("{name}({v})"));
        }
        let mut acc = parts.pop().expect("a path has a part");
        while let Some(x) = parts.pop() {
            acc = format!("and({x},{acc})");
        }
        paths.push(acc);
    }
    let mut acc = paths.pop().expect("a list has a path");
    while let Some(x) = paths.pop() {
        acc = format!("or({x},{acc})");
    }
    acc
}

use md_codec::compose::presets;
use md_codec::compose::{KeySet, Lock, SpendPath, Wrapper};

pub fn k(k: u8, n: u8) -> SpendPath {
    SpendPath {
        keys: Some(KeySet { k, n, sorted: true }),
        hash: None,
        lock: None,
    }
}
pub fn u(k: u8, n: u8) -> SpendPath {
    SpendPath {
        keys: Some(KeySet {
            k,
            n,
            sorted: false,
        }),
        hash: None,
        lock: None,
    }
}
pub fn lk(mut p: SpendPath, l: Lock) -> SpendPath {
    p.lock = Some(l);
    p
}
pub fn hs(mut p: SpendPath, h: [u8; 32]) -> SpendPath {
    p.hash = Some(h);
    p
}
pub fn kl(h: [u8; 32], l: Option<Lock>) -> SpendPath {
    SpendPath {
        keys: None,
        hash: Some(h),
        lock: l,
    }
}
pub fn pl(w: Wrapper, paths: Vec<SpendPath>) -> PathList {
    PathList { wrapper: w, paths }
}

/// (vector name, path list, rendered text WITHOUT origins, tags). The text is
/// the fixed spelling the Go builder reproduces; origins are added by
/// `template_with_origins` for the MANIFEST form. Tags are the spec rows:
/// `w:<wrapper>`, `paths:<n>`, `head:<bare-multi|single|locked>`,
/// `ik:<extracted-first|extracted-later|nums|none>`,
/// `lock:<none|blocks|units|height|time>`, `hash`, `sorted`, `unsorted`,
/// `keyless-wsh`, `spine:<m>`, `slots:32`; the §4f default-origin tag
/// `origins:default-<wrapper>` (every family vector is unseated, so every one
/// carries the wrapper's default origins); and the MANIFEST binding's
/// fingerprint case, the spec's three: `fp:distinct` (four distinct declared
/// fingerprints), `fp:one-seed-one-path` (one master fingerprint on two or
/// more slots of ONE path), `fp:one-seed-two-paths` (one master fingerprint
/// across two or more paths); `fp:none` marks the unkeyed vectors. `no-corpus`
/// marks an entry pinned by these tests and the §5b cross-check but NOT stored
/// in `MANIFEST`: the exporter and the corpus tests parse under the MINTING
/// disposition, which after Task 8 refuses a signature-free path unless
/// `--experimental`, so the two keyless-wsh vectors cannot be exported. Stage 2
/// mirrors them from this list directly.
pub fn family() -> Vec<(&'static str, PathList, String, Vec<&'static str>)> {
    let tr32: Vec<SpendPath> = (0..8).map(|_| k(4, 4)).collect();
    vec![
        // ---- wsh family
        ("keyed_compose_wsh_sole_sortedmulti", pl(Wrapper::Wsh, vec![k(2, 3)]),
         "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))".to_string(),
         vec!["w:wsh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-wsh"]),
        ("keyed_compose_wsh_two_path_or_d", pl(Wrapper::Wsh, vec![k(2, 3), lk(k(1, 1), Lock::OlderBlocks(26280))]),
         "wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pkh(@3/<0;1>/*),older(26280))))".to_string(),
         vec!["w:wsh", "paths:2", "head:bare-multi", "lock:blocks", "ik:none", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-wsh"]),
        // Same list as the previous entry; the MANIFEST binds four DISTINCT fingerprints.
        ("keyed_compose_wsh_two_path_distinct_fingerprints", pl(Wrapper::Wsh, vec![k(2, 3), lk(k(1, 1), Lock::OlderBlocks(26280))]),
         "wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pkh(@3/<0;1>/*),older(26280))))".to_string(),
         vec!["w:wsh", "paths:2", "head:bare-multi", "lock:blocks", "ik:none", "fp:distinct", "origins:default-wsh"]),
        ("keyed_compose_wsh_single_head_or_i", pl(Wrapper::Wsh, vec![k(1, 1), lk(k(1, 1), Lock::OlderUnits(15188))]),
         "wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(4209492))))".to_string(),
         vec!["w:wsh", "paths:2", "head:single", "lock:units", "ik:none", "fp:one-seed-two-paths", "origins:default-wsh"]),
        ("keyed_compose_wsh_locked_head_or_i", pl(Wrapper::Wsh, vec![lk(k(2, 2), Lock::AfterHeight(905_000)), k(1, 1)]),
         "wsh(or_i(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*),after(905000)),pkh(@2/<0;1>/*)))".to_string(),
         vec!["w:wsh", "paths:2", "head:locked", "lock:height", "ik:none", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-wsh"]),
        ("keyed_compose_wsh_hash_and_time", pl(Wrapper::Wsh, vec![k(1, 1), lk(hs(k(2, 2), H), Lock::AfterTime(1_893_456_000))]),
         format!("wsh(or_i(pkh(@0/<0;1>/*),and_v(v:multi(2,@1/<0;1>/*,@2/<0;1>/*),and_v(v:sha256({HH}),after(1893456000)))))"),
         vec!["w:wsh", "paths:2", "head:single", "lock:time", "hash", "ik:none", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-wsh"]),
        ("keyed_compose_wsh_three_paths", pl(Wrapper::Wsh, vec![k(1, 1), lk(k(1, 1), Lock::OlderBlocks(4032)), lk(k(1, 1), Lock::AfterHeight(1_000_000))]),
         "wsh(or_i(pkh(@0/<0;1>/*),or_i(and_v(v:pkh(@1/<0;1>/*),older(4032)),and_v(v:pkh(@2/<0;1>/*),after(1000000)))))".to_string(),
         vec!["w:wsh", "paths:3", "head:single", "lock:blocks", "lock:height", "ik:none", "fp:one-seed-two-paths", "origins:default-wsh"]),
        ("keyed_compose_wsh_unsorted_sole", pl(Wrapper::Wsh, vec![u(2, 3)]),
         "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))".to_string(),
         vec!["w:wsh", "paths:1", "head:bare-multi", "lock:none", "unsorted", "ik:none", "fp:one-seed-one-path", "origins:default-wsh"]),
        // ---- legacy wrappers
        ("keyed_compose_sh_wsh_sole", pl(Wrapper::ShWsh, vec![k(2, 3)]),
         "sh(wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*)))".to_string(),
         vec!["w:sh-wsh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-sh-wsh"]),
        ("keyed_compose_sh_wsh_one_of_two", pl(Wrapper::ShWsh, vec![k(1, 2)]),
         "sh(wsh(sortedmulti(1,@0/<0;1>/*,@1/<0;1>/*)))".to_string(),
         vec!["w:sh-wsh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-sh-wsh"]),
        ("keyed_compose_sh_sole", pl(Wrapper::Sh, vec![k(2, 2)]),
         "sh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))".to_string(),
         vec!["w:sh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-sh"]),
        ("keyed_compose_sh_two_of_four", pl(Wrapper::Sh, vec![k(2, 4)]),
         "sh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*,@3/<0;1>/*))".to_string(),
         vec!["w:sh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-sh"]),
        // ---- taproot family
        ("keyed_compose_tr_two_path_nums", pl(Wrapper::Tr, vec![k(2, 3), lk(k(1, 1), Lock::OlderBlocks(26280))]),
         format!("tr({NUMS},{{multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(26280))}})"),
         vec!["w:tr", "paths:2", "ik:nums", "spine:2", "lock:blocks", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-tr"]),
        // Same list as the previous entry; the MANIFEST binds four DISTINCT fingerprints.
        ("keyed_compose_tr_two_path_distinct_fingerprints", pl(Wrapper::Tr, vec![k(2, 3), lk(k(1, 1), Lock::OlderBlocks(26280))]),
         format!("tr({NUMS},{{multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(26280))}})"),
         vec!["w:tr", "paths:2", "ik:nums", "spine:2", "lock:blocks", "fp:distinct", "origins:default-tr"]),
        ("keyed_compose_tr_extracted_first", pl(Wrapper::Tr, vec![k(1, 1), lk(k(1, 1), Lock::OlderBlocks(65535))]),
         "tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),older(65535)))".to_string(),
         vec!["w:tr", "paths:2", "ik:extracted-first", "spine:1", "lock:blocks", "fp:one-seed-two-paths", "origins:default-tr"]),
        ("keyed_compose_tr_extracted_later_four_paths", pl(Wrapper::Tr, vec![lk(k(1, 1), Lock::OlderBlocks(10)), lk(k(1, 1), Lock::AfterHeight(1_000_000)), k(1, 1), lk(k(1, 1), Lock::OlderUnits(100))]),
         "tr(@0/<0;1>/*,{and_v(v:pk(@1/<0;1>/*),older(10)),{and_v(v:pk(@2/<0;1>/*),after(1000000)),and_v(v:pk(@3/<0;1>/*),older(4194404))}})".to_string(),
         vec!["w:tr", "paths:4", "ik:extracted-later", "spine:3", "lock:blocks", "lock:height", "lock:units", "fp:one-seed-two-paths", "origins:default-tr"]),
        ("keyed_compose_tr_three_paths_extracted_later", pl(Wrapper::Tr, vec![lk(k(1, 1), Lock::OlderBlocks(10)), k(1, 1), lk(k(1, 1), Lock::OlderUnits(5))]),
         "tr(@0/<0;1>/*,{and_v(v:pk(@1/<0;1>/*),older(10)),and_v(v:pk(@2/<0;1>/*),older(4194309))})".to_string(),
         vec!["w:tr", "paths:3", "ik:extracted-later", "spine:2", "lock:blocks", "lock:units", "fp:one-seed-two-paths", "origins:default-tr"]),
        ("keyed_compose_tr_nums_three_leaves", pl(Wrapper::Tr, vec![lk(k(1, 1), Lock::OlderBlocks(1)), lk(k(1, 1), Lock::OlderBlocks(2)), lk(k(2, 2), Lock::AfterHeight(2))]),
         format!("tr({NUMS},{{and_v(v:pk(@0/<0;1>/*),older(1)),{{and_v(v:pk(@1/<0;1>/*),older(2)),and_v(v:multi_a(2,@2/<0;1>/*,@3/<0;1>/*),after(2))}}}})"),
         vec!["w:tr", "paths:3", "ik:nums", "spine:3", "lock:blocks", "lock:height", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-tr"]),
        ("keyed_compose_tr_sole_sortedmulti_a", pl(Wrapper::Tr, vec![k(2, 3)]),
         format!("tr({NUMS},sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))"),
         vec!["w:tr", "paths:1", "ik:nums", "spine:1", "lock:none", "sorted", "fp:one-seed-one-path", "origins:default-tr"]),
        ("keyed_compose_tr_key_path_only", pl(Wrapper::Tr, vec![k(1, 1)]),
         "tr(@0/<0;1>/*)".to_string(),
         vec!["w:tr", "paths:1", "ik:extracted-first", "spine:0", "lock:none", "origins:default-tr"]),
        ("keyed_compose_tr_unsorted_sole_leaf", pl(Wrapper::Tr, vec![u(2, 2)]),
         format!("tr({NUMS},multi_a(2,@0/<0;1>/*,@1/<0;1>/*))"),
         vec!["w:tr", "paths:1", "ik:nums", "spine:1", "lock:none", "unsorted", "fp:one-seed-one-path", "origins:default-tr"]),
        ("keyed_compose_tr_hash_leaf", pl(Wrapper::Tr, vec![k(2, 2), lk(hs(k(1, 1), H), Lock::AfterTime(1_893_456_000))]),
         format!("tr({NUMS},{{multi_a(2,@0/<0;1>/*,@1/<0;1>/*),and_v(v:pk(@2/<0;1>/*),and_v(v:sha256({HH}),after(1893456000)))}})"),
         vec!["w:tr", "paths:2", "ik:nums", "spine:2", "hash", "lock:time", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-tr"]),
        // ---- unkeyed: EXPERIMENTAL shapes and the size boundaries (more slots than the four journey keys)
        ("compose_wsh_keyless_hash_path", pl(Wrapper::Wsh, vec![k(2, 3), kl(H, Some(Lock::AfterHeight(1_383_520)))]),
         format!("wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:sha256({HH}),after(1383520))))"),
         vec!["w:wsh", "paths:2", "head:bare-multi", "keyless-wsh", "hash", "lock:height", "ik:none", "fp:none", "origins:default-wsh", "no-corpus"]),
        ("compose_wsh_keyless_hash_only", pl(Wrapper::Wsh, vec![k(1, 1), kl(H, None)]),
         format!("wsh(or_i(pkh(@0/<0;1>/*),sha256({HH})))"),
         vec!["w:wsh", "paths:2", "head:single", "keyless-wsh", "hash", "lock:none", "ik:none", "fp:none", "origins:default-wsh", "no-corpus"]),
        ("compose_wsh_eight_paths", pl(Wrapper::Wsh, (0..8).map(|i| lk(k(1, 1), Lock::OlderBlocks(100 + i))).collect()),
         "wsh(or_i(and_v(v:pkh(@0/<0;1>/*),older(100)),or_i(and_v(v:pkh(@1/<0;1>/*),older(101)),or_i(and_v(v:pkh(@2/<0;1>/*),older(102)),or_i(and_v(v:pkh(@3/<0;1>/*),older(103)),or_i(and_v(v:pkh(@4/<0;1>/*),older(104)),or_i(and_v(v:pkh(@5/<0;1>/*),older(105)),or_i(and_v(v:pkh(@6/<0;1>/*),older(106)),and_v(v:pkh(@7/<0;1>/*),older(107))))))))))".to_string(),
         vec!["w:wsh", "paths:8", "head:locked", "lock:blocks", "ik:none", "fp:none", "origins:default-wsh"]),
        ("compose_tr_seven_leaves", pl(Wrapper::Tr, (0..8).map(|i| if i == 0 { k(1, 1) } else { lk(k(1, 1), Lock::OlderBlocks(100 + i)) }).collect()),
         "tr(@0/<0;1>/*,{and_v(v:pk(@1/<0;1>/*),older(101)),{and_v(v:pk(@2/<0;1>/*),older(102)),{and_v(v:pk(@3/<0;1>/*),older(103)),{and_v(v:pk(@4/<0;1>/*),older(104)),{and_v(v:pk(@5/<0;1>/*),older(105)),{and_v(v:pk(@6/<0;1>/*),older(106)),and_v(v:pk(@7/<0;1>/*),older(107))}}}}}})".to_string(),
         vec!["w:tr", "paths:8", "ik:extracted-first", "spine:7", "lock:blocks", "fp:none", "origins:default-tr"]),
        ("compose_wsh_thirty_two_slots", pl(Wrapper::Wsh, vec![k(9, 9), k(9, 9), k(9, 9), k(5, 5)]),
         "wsh(or_d(multi(9,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*,@3/<0;1>/*,@4/<0;1>/*,@5/<0;1>/*,@6/<0;1>/*,@7/<0;1>/*,@8/<0;1>/*),or_d(multi(9,@9/<0;1>/*,@10/<0;1>/*,@11/<0;1>/*,@12/<0;1>/*,@13/<0;1>/*,@14/<0;1>/*,@15/<0;1>/*,@16/<0;1>/*,@17/<0;1>/*),or_d(multi(9,@18/<0;1>/*,@19/<0;1>/*,@20/<0;1>/*,@21/<0;1>/*,@22/<0;1>/*,@23/<0;1>/*,@24/<0;1>/*,@25/<0;1>/*,@26/<0;1>/*),multi(5,@27/<0;1>/*,@28/<0;1>/*,@29/<0;1>/*,@30/<0;1>/*,@31/<0;1>/*)))))".to_string(),
         vec!["w:wsh", "paths:4", "slots:32", "head:bare-multi", "lock:none", "ik:none", "fp:none", "origins:default-wsh"]),
        ("compose_tr_thirty_two_slots", pl(Wrapper::Tr, tr32),
         format!("tr({NUMS},{{multi_a(4,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*,@3/<0;1>/*),{{multi_a(4,@4/<0;1>/*,@5/<0;1>/*,@6/<0;1>/*,@7/<0;1>/*),{{multi_a(4,@8/<0;1>/*,@9/<0;1>/*,@10/<0;1>/*,@11/<0;1>/*),{{multi_a(4,@12/<0;1>/*,@13/<0;1>/*,@14/<0;1>/*,@15/<0;1>/*),{{multi_a(4,@16/<0;1>/*,@17/<0;1>/*,@18/<0;1>/*,@19/<0;1>/*),{{multi_a(4,@20/<0;1>/*,@21/<0;1>/*,@22/<0;1>/*,@23/<0;1>/*),{{multi_a(4,@24/<0;1>/*,@25/<0;1>/*,@26/<0;1>/*,@27/<0;1>/*),multi_a(4,@28/<0;1>/*,@29/<0;1>/*,@30/<0;1>/*,@31/<0;1>/*)}}}}}}}}}}}}}})"),
         vec!["w:tr", "paths:8", "slots:32", "ik:nums", "spine:7", "lock:none", "fp:none", "origins:default-tr"]),
        // ---- presets (F-453): ONE vector per archetype, built by CALLING the
        // constructor (so a drifted parameter order or default changes the
        // PathList here too), with a hand-typed expected-text literal (so a
        // drifted LOWERING still fails `every_family_entry_renders_as_listed`).
        // Parameters: 2-of-3 and older=26280 wherever the archetype leaves
        // them free (the journey's own canonical values, §4d fixes no
        // defaults); every vector stays within the four journey xpubs.
        ("keyed_compose_preset_plain_multisig", presets::plain_multisig(Wrapper::Wsh, 2, 3).unwrap(),
         "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))".to_string(),
         vec!["w:wsh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-wsh", "preset:plain-multisig"]),
        ("keyed_compose_preset_simple_timelocked_inheritance", presets::simple_timelocked_inheritance(Wrapper::Wsh, 26280).unwrap(),
         "wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(26280))))".to_string(),
         vec!["w:wsh", "paths:2", "head:single", "lock:blocks", "ik:none", "fp:one-seed-two-paths", "origins:default-wsh", "preset:simple-timelocked-inheritance"]),
        ("keyed_compose_preset_kofn_recovery", presets::kofn_recovery(Wrapper::Tr, 2, 3, 26280).unwrap(),
         format!("tr({NUMS},{{multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(26280))}})"),
         vec!["w:tr", "paths:2", "ik:nums", "spine:2", "lock:blocks", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-tr", "preset:kofn-recovery"]),
        ("keyed_compose_preset_tiered_recovery", presets::tiered_recovery(Wrapper::Wsh, 2, 2, 1, 2, 26280).unwrap(),
         "wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*),and_v(v:multi(1,@2/<0;1>/*,@3/<0;1>/*),older(26280))))".to_string(),
         vec!["w:wsh", "paths:2", "head:bare-multi", "lock:blocks", "ik:none", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-wsh", "preset:tiered-recovery"]),
        ("keyed_compose_preset_hashlock_gated", presets::hashlock_gated(Wrapper::Wsh, H, 26280).unwrap(),
         format!("wsh(or_i(and_v(v:pkh(@0/<0;1>/*),sha256({HH})),and_v(v:pkh(@1/<0;1>/*),older(26280))))"),
         // R0 fidelity N-1: the head path is one key PLUS a hash, unlocked --
         // neither head:bare-multi (n = 1), head:single (is_bare_single needs
         // no hash), nor head:locked (no lock). `head:hashed` names this
         // fourth shape; it joins SINGULAR_TAGS below since this is the only
         // family vector with it.
         vec!["w:wsh", "paths:2", "head:hashed", "lock:blocks", "hash", "ik:none", "fp:one-seed-two-paths", "origins:default-wsh", "preset:hashlock-gated"]),
        ("keyed_compose_preset_decaying_multisig", presets::decaying_multisig(Wrapper::Wsh, 2, 2, 1, 1, 13140, 26280, 1_000_000).unwrap(),
         "wsh(or_i(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*),older(13140)),or_i(and_v(v:pkh(@2/<0;1>/*),older(26280)),and_v(v:pkh(@3/<0;1>/*),after(1000000)))))".to_string(),
         vec!["w:wsh", "paths:3", "head:locked", "lock:blocks", "lock:height", "ik:none", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-wsh", "preset:decaying-multisig"]),
    ]
}

/// Tags with exactly ONE legal shape, exempt from the two-vector rule and said
/// so here: a taptree with m = 0 leaves is one unlocked single key and nothing
/// else (spec §12 item 1). The six `preset:<name>` tags join it for the SAME
/// structural reason: F-453's deliverable is explicitly ONE MANIFEST vector
/// per archetype, so each has exactly one legal vector by construction, not by
/// coverage gap. §12 item 1's own required-tag list (`compose_vectors.rs`) is
/// NOT extended with `preset:*` — presets are not one of the axes that list
/// names, so nothing there needs touching.
pub const SINGULAR_TAGS: &[&str] = &[
    "spine:0",
    // R0 fidelity N-1: the ONLY family vector whose head path is a single key
    // plus a hash, unlocked (neither bare-multi, single, nor locked).
    "head:hashed",
    "preset:plain-multisig",
    "preset:simple-timelocked-inheritance",
    "preset:kofn-recovery",
    "preset:tiered-recovery",
    "preset:hashlock-gated",
    "preset:decaying-multisig",
];
```

Run: `cd /scratch/code/shibboleth/descriptor-mnemonic && cargo nextest run --locked -p md-codec --test compose_vectors --test compose_crosscheck 2>&1 | tail -15`
Expected: `every_family_entry_renders_as_listed`, `every_tag_appears_in_at_least_two_vectors` and `every_family_entry_passes_the_5b_cross_check` (in `compose_crosscheck.rs`) PASS — the six new literals were verified against a real `md compose`/`encode`/`decode`/`address` run before this plan was written, and `SINGULAR_TAGS` already covers the six new one-vector tags. `every_compose_vector_in_the_manifest_is_exactly_what_compose_renders` FAILS with `MANIFEST lacks keyed_compose_preset_plain_multisig` (the first of the six missing entries, alphabetically first among the new names in iteration order) — this IS the SAME pinned-red shape `scripts/plan-build-gate-md.sh` already accepts, now for a new vector name.

- [ ] **Step 2: Add the six MANIFEST entries**

Add to `crates/md-codec/src/test_vectors.rs`, inside `MANIFEST` after the last existing entry (`compose_tr_thirty_two_slots`), before the closing `];`. Every template below was produced by the actually-shipped `md compose`/`encode`/`decode` round trip, not typed by hand from the lowering table; `XPUB_JOURNEY_0..3` are the existing constants declared above `MANIFEST` (`crates/md-codec/src/test_vectors.rs:49-52`, unchanged):

```rust
    Vector { name: "keyed_compose_preset_plain_multisig",
        template: "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_preset_simple_timelocked_inheritance",
        template: "wsh(or_i(pkh(@0/48'/0'/0'/2'/<0;1>/*),and_v(v:pkh(@1/48'/0'/1'/2'/<0;1>/*),older(26280))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_preset_kofn_recovery",
        template: "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*),and_v(v:pk(@3/48'/0'/3'/3'/<0;1>/*),older(26280))})",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_preset_tiered_recovery",
        template: "wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*),and_v(v:multi(1,@2/48'/0'/2'/2'/<0;1>/*,@3/48'/0'/3'/2'/<0;1>/*),older(26280))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_preset_hashlock_gated",
        template: "wsh(or_i(and_v(v:pkh(@0/48'/0'/0'/2'/<0;1>/*),sha256(a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8)),and_v(v:pkh(@1/48'/0'/1'/2'/<0;1>/*),older(26280))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_compose_preset_decaying_multisig",
        template: "wsh(or_i(and_v(v:multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*),older(13140)),or_i(and_v(v:pkh(@2/48'/0'/2'/2'/<0;1>/*),older(26280)),and_v(v:pkh(@3/48'/0'/3'/2'/<0;1>/*),after(1000000)))))",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
```

Run: `cargo nextest run --locked -p md-codec 2>&1 | tail -8`
Expected: all md-codec tests PASS, 52 in the compose binaries (unchanged from S0 — six new tuples in `family()` are data consumed by existing `#[test]` functions, not new test functions) plus the pre-existing corpus tests that iterate `MANIFEST`.

- [ ] **Step 3: Run the exporter and the corpus tests; confirm the file-count delta**

Run: `cargo run --locked -p md-cli --bin md -- vectors --out /tmp/compose-vectors-f453 >/dev/null && ls /tmp/compose-vectors-f453 | grep -c 'keyed_compose_preset_.*conformance.json' && rm -rf /tmp/compose-vectors-f453`
Expected: `6` (one `.conformance.json` per new vector; measured live during this plan's authoring). Then `cargo nextest run --locked -p md-cli 2>&1 | tail -8`: `template_roundtrip.rs`, `vector_corpus.rs` (its drift-comparison sub-test) and `corpus_origin_consistency.rs` iterate `MANIFEST` through the real parser and encoder and must PASS, EXCEPT the corpus-drift test (`vectors_output_matches_committed_corpus`), which fails until Task 3 regenerates the committed corpus under `crates/md-codec/tests/vectors/` — measured during authoring: exactly 30 "Only in" lines (six vectors × five files: `.bytes.hex`, `.phrase.txt`, `.descriptor.json`, `.template`, `.conformance.json`) and ZERO "differ" lines; any "differ" line is a finding, stop.

- [ ] **Step 4: Format, clippy, commit**

Run: `cargo fmt --all && cargo clippy --locked -p md-codec --all-targets --all-features -- -D warnings 2>&1 | tail -3`
Expected: clean (measured: zero warnings on this exact diff).

```bash
git add crates/md-codec/tests/compose_support.rs crates/md-codec/src/test_vectors.rs
git commit -m "md-codec: six preset MANIFEST vectors -- one per archetype, built by calling presets::*, singular preset:<name> tags (F-453 composer S0b task 1)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 2: `md compose --preset` — the CLI surface

**Files:**
- Replace: `crates/md-cli/src/cmd/compose.rs`
- Modify (fragment): `crates/md-cli/src/main.rs`
- Create: `crates/md-cli/tests/cli_compose_preset.rs`

**Interfaces:**
- Consumes: `md_codec::compose::{presets, compose, template_with_origins, Experimental, KeySet, Lock, PathList, SpendPath, Wrapper, ComposeError}`; `crate::error::CliError` (its existing `Compose(String)` variant, `crates/md-cli/src/error.rs:46,132` — unchanged).
- Produces: `cmd::compose::{parse_wrapper, parse_path, parse_preset, PresetParams, PRESET_NAMES, run}` (`run`'s signature changes from `(wrapper, paths, experimental, json)` to `(wrapper, paths, preset, experimental, json)`).

**The grammar (normative for this CLI; the fork's picker builds the same six shapes on its own inputs, not by parsing this string):**

| flag | form | example |
| --- | --- | --- |
| `--preset` | `<name>[,<k>of<n>]*[,<param>=<value>]*`, mutually exclusive with `--path` (a `clap::ArgGroup`, exactly one of the two required) | `kofn-recovery,2of3,older=26280` |

| preset | grammar | maps to |
| --- | --- | --- |
| `plain-multisig` | `<k>of<n>` | `presets::plain_multisig(wrapper, k, n)` |
| `simple-timelocked-inheritance` | `older=<n>` | `presets::simple_timelocked_inheritance(wrapper, older)` |
| `kofn-recovery` | `<k>of<n>,older=<n>` | `presets::kofn_recovery(wrapper, k, n, older)` |
| `tiered-recovery` | `<k1>of<n1>,<k2>of<n2>,older=<n>` (tier 1 before tier 2, by LISTED order) | `presets::tiered_recovery(wrapper, k1, n1, k2, n2, older)` |
| `hashlock-gated` | `sha256=<64 hex>,older=<n>` | `presets::hashlock_gated(wrapper, hash, older)` |
| `decaying-multisig` | `<k1>of<n1>,<k2>of<n2>,older1=<n>,older2=<n>,after=<n>` | `presets::decaying_multisig(wrapper, k1, n1, k2, n2, older1, older2, after)` |

Design decisions, stated once here rather than re-derived per preset: the `<k>of<n>` tokens are POSITIONAL (consumed in listed order — the only way to fill two key-set parameters unambiguously without inventing a second naming scheme); every `<param>=<value>` token is matched BY NAME, so it can appear in any order and a duplicate is refused; `older`/`older1`/`older2`/`after` all take PLAIN BLOCKS OR HEIGHT numbers, never `--path`'s `Nu`/`Ht` unit-suffix forms — every `presets::*` constructor's own lock parameter is a bare `u32` blocks/height count (`crates/md-codec/src/compose/presets.rs:18-25`'s `blocks()` helper always builds `Lock::OlderBlocks`, never `OlderUnits`; `decaying_multisig`'s `after_height` is always `Lock::AfterHeight`, never `AfterTime`), so there is no unit ambiguity to disambiguate and no suffix grammar to invent. `unsorted` is never a preset parameter: `presets::ks` hardcodes `sorted: true` in every key set it builds, so there is nothing for it to toggle — a `--path` list can ask for `unsorted`, a `--preset` never can. Two `<k>of<n>` parameters need distinct lock names (`older1`/`older2`) only where an archetype HAS two ambiguous locks (`decaying-multisig`); every other multi-lock-free archetype keeps the bare `older`. **R0 fidelity M-1:** for `decaying-multisig`, `older1` locks the FIRST (primary) tier and `older2` the second — the primary tier does NOT spend immediately, only after `older1`; the grammar table above names this, and `--preset`'s own `--help` text (Task 2 Step 4) says so too, so an operator reading `decaying-multisig,2of2,1of1,older1=13140,...` does not assume the 2-of-2 spends today.

**R0 fidelity I-3 and M-4, two more design notes before the refusal table:** first, `parse_preset` checks the preset NAME against `PRESET_NAMES` before parsing any `<k>of<n>`/`<param>=<value>` token, so an unknown name is ALWAYS reported as "expected one of …", even when a token given alongside it is also malformed — a malformed-token message never masks a name that was never a preset in the first place. Second, `PRESET_NAMES` and the six `match` arms inside `parse_preset` are two lists that could drift (a name added to one and not the other would compile and clippy-pass, and the "expected one of" line would then advertise a name that does not work). **R0 round-1 fold-verification found the first attempt at closing this did not: a hardcoded `[(&str, &str); 6]` fixture asserting its own `.len() == 6` is a tautology that cannot fail under any such drift, confirmed live by adding a 7th, unmatched name to `PRESET_NAMES` — it compiled, passed clippy, passed all 31 tests, and then PANICKED (`unreachable!()`, exit 101) on a real `md compose --preset phantom-preset,2of3` invocation.** Two fixes, both machine-verified against that exact mutation: the final `match` arm is now `other => Err(CliError::Compose(...))`, naming the preset, instead of `unreachable!()` — the CLI must never panic on its own table, even mid-drift; and the coverage test is now a `#[cfg(test)] mod tests` UNIT test embedded in `compose.rs` itself (the established pattern this crate already uses for `seat::partition`/`format::text`, since `md-cli` ships no library target and a black-box `tests/cli_compose_preset.rs` integration test cannot call `parse_preset` or read `PRESET_NAMES` directly). That test iterates `PRESET_NAMES` ITSELF and calls `parse_preset` on each entry with a valid parameter set, so a 7th name with no matching fixture or `match` arm fails the test — re-confirmed against the SAME mutation: the test now fails with the message ``PRESET_NAMES gained `phantom-preset` with no valid-parameter fixture in this test``, and the CLI itself now exits 1 with `preset phantom-preset: internal error -- ...` instead of panicking.

**Refusals (one test each, Task 2 Step 1):**

| condition | wording | exit |
| --- | --- | --- |
| unknown preset name | `--preset <name>: expected one of plain-multisig, simple-timelocked-inheritance, kofn-recovery, tiered-recovery, hashlock-gated, decaying-multisig` | 1 |
| a required parameter missing | `preset <name> needs <param>=<n>` (or `needs exactly N <k>of<n> parameter(s), got M` for a wrong `<k>of<n>` count) | 1 |
| an admitted-nowhere parameter | `preset <name> admits no <param>= parameter` | 1 |
| a parameter the CONSTRUCTOR itself rejects | the `ComposeError`'s own `Display`, verbatim, e.g. `path 2: older in blocks needs 1..=65535` (`kofn-recovery,...,older=70000` — `presets::kofn_recovery`'s own `blocks()` guard, NOT the `--path` DSL's separate pre-check, which has different wording) | 1 |
| a non-plain preset under `sh`/`sh(wsh)` (§4d: "under sh/sh(wsh) only the plain k-of-n preset is offered") | `legacy wrappers hold one plain sorted multisig only (n >= 2, no lock, no hash); use wsh or tr` — the SAME `ComposeError::LegacyWrapperShape` a hand-built `--path` list of the same shape gives; no CLI-side special case needed | 1 |
| `--path` and `--preset` both given, or neither given | clap's own `ArgGroup` error (exit 2, not `CliError`) — measured: `error: the argument '--path <PATH>' cannot be used with '--preset <PRESET>'` / `error: the following required arguments were not provided:` then `<--path <PATH>` and `--preset <PRESET>` joined by a pipe on the next line (the pipe is omitted here because the table gate cannot see inside code spans) | 2 |
| a named parameter given twice | `` preset <name>: `<key>=` given twice `` (R0 fidelity I-3) | 1 |
| a `<k>of<n>` token that is not `<k>of<n>` (name already valid) | `` preset <name>: `<tok>` is not <k>of<n> `` (R0 fidelity I-3) | 1 |
| a `<k>of<n>` value too large to fit a `u8` | `` preset <name>: k `<v>` is not a small number `` (or `n`) — a PARSE failure, distinct from `BadThreshold` above: nothing fits `u8` yet to name in "1 <= k <= n <= 9"; a value that DOES fit `u8` but violates that band (e.g. `2of15`) already reaches the real constructor and surfaces `BadThreshold`'s own text verbatim, so the two messages are right for two different failures (R0 fidelity I-3) | 1 |
| a named value that does not parse as a number | `` preset <name> <param>: `<v>` is not a number in 0..=4294967295 `` (R0 fidelity I-3) | 1 |
| `hashlock-gated` with `sha256=` missing, or not 64 lowercase hex characters | `preset <name> needs sha256=<64 hex>` / `sha256 needs 64 hex characters, lowercase` (R0 fidelity I-3) | 1 |
| `decaying-multisig`'s `after=` at or above the Unix-time band | names `--path` as the only remedy, since the preset grammar has no `t` suffix: `` preset <name>: after=<v> is read as a block height and is above the height band (1..=499999999); presets cannot express a Unix time -- use --path with `after=<v>t` instead `` (R0 fidelity M-3) | 1 |

**R0 fidelity N-2 — `--json`'s `SCHEMA` constant is NOT bumped for the new `preset` field, and stays that way.** `crates/md-cli/src/format/json.rs:6` (`pub const SCHEMA: &str = "md-cli/1";`) has never moved for an additive field on any subcommand's JSON, measured against this crate's own history — most directly, S0's OWN `md compose --json` shipped a brand-new subcommand's entire JSON shape without bumping it. `preset` is additive and always present (`null` for a `--path`-built policy, asserted by `path_json_names_no_preset`), so a consumer reading an unknown field is the worst case, not a broken one; bumping `SCHEMA` for one more optional field on one subcommand would be new practice invented for this plan, not applied practice.

- [ ] **Step 1: Write the failing CLI tests**

Create `crates/md-cli/tests/cli_compose_preset.rs`:

```rust
//! `md compose --preset` (F-453, SPEC_wallet_policy_composer.md §4d, C2): the
//! six archetypes as one-tap presets, mutually exclusive with `--path`, still
//! honouring `--wrapper`/`--experimental`/`--json`. Grammar:
//! `--preset <name>[,<k>of<n>]*[,<param>=<value>]*`.

use assert_cmd::Command;
use predicates::prelude::*;

fn md() -> Command {
    Command::cargo_bin("md").expect("md binary")
}

#[test]
fn preset_plain_multisig_matches_the_equivalent_path_list() {
    let preset = md()
        .args([
            "compose",
            "--wrapper",
            "wsh",
            "--preset",
            "plain-multisig,2of3",
        ])
        .output()
        .unwrap();
    let path = md()
        .args(["compose", "--wrapper", "wsh", "--path", "2of3"])
        .output()
        .unwrap();
    assert!(preset.status.success());
    assert_eq!(preset.stdout, path.stdout);
}

#[test]
fn preset_kofn_recovery_matches_the_equivalent_path_list_under_tr() {
    let preset = md()
        .args([
            "compose",
            "--wrapper",
            "tr",
            "--preset",
            "kofn-recovery,2of3,older=26280",
        ])
        .output()
        .unwrap();
    let path = md()
        .args([
            "compose",
            "--wrapper",
            "tr",
            "--path",
            "2of3",
            "--path",
            "1of1,older=26280",
        ])
        .output()
        .unwrap();
    assert!(preset.status.success());
    assert_eq!(preset.stdout, path.stdout);
}

#[test]
fn preset_tiered_recovery_and_decaying_multisig_and_hashlock_gated_compose() {
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "tiered-recovery,2of2,1of2,older=26280",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*),and_v(v:multi(1,@2/48'/0'/2'/2'/<0;1>/*,@3/48'/0'/3'/2'/<0;1>/*),older(26280))))",
    ));
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "decaying-multisig,2of2,1of1,older1=13140,older2=26280,after=1000000",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "wsh(or_i(and_v(v:multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*),older(13140)),or_i(and_v(v:pkh(@2/48'/0'/2'/2'/<0;1>/*),older(26280)),and_v(v:pkh(@3/48'/0'/3'/2'/<0;1>/*),after(1000000)))))",
    ));
    let h = "a8".repeat(32);
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        &format!("hashlock-gated,sha256={h},older=26280"),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains(format!(
        "sha256({h})),and_v(v:pkh(@1/48'/0'/1'/2'/<0;1>/*),older(26280))"
    )));
}

#[test]
fn preset_and_path_are_mutually_exclusive() {
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--path",
        "2of3",
        "--preset",
        "plain-multisig,2of3",
    ])
    .assert()
    .failure()
    .code(2)
    .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn compose_refuses_when_neither_path_nor_preset_given() {
    md().args(["compose", "--wrapper", "wsh"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn preset_refuses_an_unknown_name() {
    md().args(["compose", "--wrapper", "wsh", "--preset", "frobnicate,2of3"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "expected one of plain-multisig, simple-timelocked-inheritance, kofn-recovery, tiered-recovery, hashlock-gated, decaying-multisig",
        ));
}

#[test]
fn preset_refuses_a_missing_parameter() {
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "kofn-recovery,2of3",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "preset kofn-recovery needs older=<n>",
    ));
}

#[test]
fn preset_refuses_an_extra_parameter() {
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "plain-multisig,2of3,older=10",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "preset plain-multisig admits no older= parameter",
    ));
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "plain-multisig,2of3,1of1",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "preset plain-multisig needs exactly 1 <k>of<n> parameter, got 2",
    ));
}

#[test]
fn preset_propagates_a_parameter_the_constructor_rejects() {
    // kofn_recovery's own `blocks()` guard, not the CLI's --path pre-check:
    // exercises the SAME ComposeError::LockOutOfRange a hand-built --path list
    // would hit, propagated verbatim.
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "kofn-recovery,2of3,older=70000",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "path 2: older in blocks needs 1..=65535",
    ));
}

#[test]
fn preset_every_non_plain_archetype_refuses_under_both_legacy_wrappers_spec_4d_shape() {
    // SPEC §4d: "under sh/sh(wsh) only the plain k-of-n preset is offered."
    // No CLI special-case is needed: every non-plain archetype's PathList
    // fails `validate`'s legacy-wrapper-shape check the same way a hand-built
    // --path list with the same shape would. R0 fidelity M-2: this was tested
    // for one of the ten (archetype, wrapper) pairs; all ten now run.
    let non_plain: [(&str, &str); 5] = [
        ("simple-timelocked-inheritance", "older=100"),
        ("kofn-recovery", "2of3,older=100"),
        ("tiered-recovery", "2of2,1of2,older=100"),
        (
            "hashlock-gated",
            "sha256=a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8,older=100",
        ),
        (
            "decaying-multisig",
            "2of2,1of1,older1=100,older2=200,after=1000",
        ),
    ];
    for wrapper in ["sh", "sh-wsh"] {
        for (name, params) in &non_plain {
            md().args(["compose", "--wrapper", wrapper, "--preset", &format!("{name},{params}")])
                .assert()
                .failure()
                .code(1)
                .stderr(predicate::str::contains(
                    "legacy wrappers hold one plain sorted multisig only (n >= 2, no lock, no hash); use wsh or tr",
                ));
        }
        md().args([
            "compose",
            "--wrapper",
            wrapper,
            "--preset",
            "plain-multisig,2of3",
        ])
        .assert()
        .success();
    }
}

#[test]
fn preset_decaying_multisig_propagates_preset_shape_refusals() {
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "decaying-multisig,2of2,2of3,older1=2000,older2=1000,after=100",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "preset: decaying tiers must unlock progressively later (the second older must exceed the first)",
    ));
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "decaying-multisig,1of2,2of3,older1=1000,older2=2000,after=100",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "preset: a decaying multisig decays: the recovery threshold cannot exceed the primary threshold",
    ));
}

#[test]
fn preset_unknown_name_wins_over_a_malformed_token() {
    // R0 fidelity I-3: `parse_preset` checks the NAME before parsing any
    // token, so an unknown name is reported even when a token is ALSO
    // malformed -- not "`2/3` is not <k>of<n>" for a name that was never a
    // preset in the first place.
    md().args(["compose", "--wrapper", "wsh", "--preset", "multisig,2/3"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "expected one of plain-multisig, simple-timelocked-inheritance, kofn-recovery, tiered-recovery, hashlock-gated, decaying-multisig",
        ));
}

#[test]
fn preset_refuses_a_duplicate_named_parameter() {
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "kofn-recovery,2of3,older=100,older=200",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "preset kofn-recovery: `older=` given twice",
    ));
}

#[test]
fn preset_refuses_a_malformed_kofn_token() {
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "plain-multisig,2/3",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "preset plain-multisig: `2/3` is not <k>of<n>",
    ));
}

#[test]
fn preset_refuses_a_kofn_magnitude_that_does_not_fit_a_small_number() {
    // R0 fidelity I-3: this is a DIFFERENT failure class from BadThreshold --
    // 300 does not fit u8 at all, so there is no (k, n) pair yet to name in
    // BadThreshold's "1 <= k <= n <= 9" wording. A value that DOES fit u8 but
    // violates that band (e.g. 2of15) already reaches the real constructor and
    // surfaces BadThreshold's own text verbatim (asserted below).
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "plain-multisig,300of3",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "preset plain-multisig: k `300` is not a small number",
    ));
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "plain-multisig,2of300",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "preset plain-multisig: n `300` is not a small number",
    ));
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "plain-multisig,2of15",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "2-of-15 is not admitted (1 <= k <= n <= 9)",
    ));
}

#[test]
fn preset_refuses_a_non_numeric_named_value() {
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "kofn-recovery,2of3,older=soon",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "preset kofn-recovery older: `soon` is not a number in 0..=4294967295",
    ));
}

#[test]
fn preset_refuses_a_missing_or_malformed_sha256() {
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "hashlock-gated,older=1",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "preset hashlock-gated needs sha256=<64 hex>",
    ));
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "hashlock-gated,sha256=ab,older=1",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "sha256 needs 64 hex characters, lowercase",
    ));
}

#[test]
fn preset_decaying_multisig_after_in_the_time_band_names_path_as_the_remedy() {
    // R0 fidelity M-3: decaying-multisig's `after` always builds a HEIGHT lock
    // (`presets::decaying_multisig` never emits `AfterTime`) and the preset
    // grammar has no `t` suffix to ask for a time lock -- unlike --path's
    // `after=<T>t`. A Unix-time-sized value therefore names --path, the only
    // way to express it, rather than a bare band refusal with no remedy.
    md().args([
        "compose",
        "--wrapper",
        "wsh",
        "--preset",
        "decaying-multisig,2of2,1of1,older1=100,older2=200,after=1893456000",
    ])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
        "after=1893456000 reads as a block height and is above the height band (1..=499999999); presets cannot express a Unix time -- use --path with `after=1893456000t` instead",
    ));
}

#[test]
fn preset_never_needs_experimental() {
    // Every presets::* key set is `sorted: true` and every archetype is keyed
    // (never a bare hash-only path), so `composed.experimental` is always
    // empty for a preset -- --experimental is accepted but never required.
    for args in [
        vec![
            "compose",
            "--wrapper",
            "wsh",
            "--preset",
            "plain-multisig,2of3",
        ],
        vec![
            "compose",
            "--wrapper",
            "tr",
            "--preset",
            "kofn-recovery,2of3,older=26280",
        ],
    ] {
        md().args(&args)
            .assert()
            .success()
            .stderr(predicate::str::contains("EXPERIMENTAL").not());
    }
}

#[cfg(feature = "json")]
#[test]
fn preset_json_names_the_preset_and_its_resolved_parameters() {
    let out = md()
        .args([
            "compose",
            "--wrapper",
            "tr",
            "--json",
            "--preset",
            "kofn-recovery,2of3,older=26280",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["preset"]["name"], "kofn-recovery");
    assert_eq!(v["preset"]["params"]["k"], 2);
    assert_eq!(v["preset"]["params"]["n"], 3);
    assert_eq!(v["preset"]["params"]["older_blocks"], 26280);
}

#[cfg(feature = "json")]
#[test]
fn path_json_names_no_preset() {
    let out = md()
        .args(["compose", "--wrapper", "wsh", "--json", "--path", "2of3"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["preset"].is_null());
}
```

- [ ] **Step 2: Run to verify the tests fail**

Run: `cargo nextest run --locked -p md-cli --test cli_compose_preset 2>&1 | tail -8`
Expected: FAIL (R0 tests-lens N-1: this is a clap RUNTIME parse rejection, not a `rustc` compile error — the workspace compiles cleanly at this point) — `--preset` is not a recognised flag yet, so every test using it fails immediately with clap's own `error: unexpected argument '--preset' found`, exit 2 (the crate itself still builds since `cli_compose_preset.rs` is a separate test binary).

- [ ] **Step 3: Replace `crates/md-cli/src/cmd/compose.rs`**

Replace `crates/md-cli/src/cmd/compose.rs` in full:

```rust
//! `md compose` -- the FIXED lowering surface (SPEC_wallet_policy_composer.md
//! §10 item 1). The opposite contract to `md compile`: no search, no cost
//! model, the same answer from every implementation, forever.
//!
//! Not to be confused with `crate::seat::compose`, which SEATS keys into an
//! existing keyless card; this module builds the card's policy from a path list.

use crate::error::CliError;
use md_codec::compose::{
    Experimental, KeySet, Lock, PathList, SpendPath, Wrapper, compose, presets,
    template_with_origins,
};
use md_codec::render::descriptor_to_template;

pub fn parse_wrapper(s: &str) -> Result<Wrapper, CliError> {
    match s {
        "tr" => Ok(Wrapper::Tr),
        "wsh" => Ok(Wrapper::Wsh),
        "sh-wsh" => Ok(Wrapper::ShWsh),
        "sh" => Ok(Wrapper::Sh),
        other => Err(CliError::Compose(format!(
            "--wrapper {other}: expected tr, wsh, sh-wsh or sh"
        ))),
    }
}

fn parse_u32(s: &str, what: &str) -> Result<u32, CliError> {
    s.parse::<u32>()
        .map_err(|_| CliError::Compose(format!("{what}: `{s}` is not a number in 0..=4294967295")))
}

fn parse_u16(s: &str, what: &str) -> Result<u16, CliError> {
    s.parse::<u16>()
        .map_err(|_| CliError::Compose(format!("{what}: `{s}` is not a number in 0..=65535")))
}

/// One `--path` value: `<k>of<n>[,opt]*` or `keyless[,opt]*`.
pub fn parse_path(s: &str) -> Result<SpendPath, CliError> {
    let mut parts = s.split(',');
    let head = parts.next().unwrap_or("");
    let keys = if head == "keyless" {
        None
    } else {
        let (k, n) = head.split_once("of").ok_or_else(|| {
            CliError::Compose(format!("path `{s}`: expected <k>of<n> or keyless"))
        })?;
        let k = k
            .parse::<u8>()
            .map_err(|_| CliError::Compose(format!("path `{s}`: k `{k}` is not a small number")))?;
        let n = n
            .parse::<u8>()
            .map_err(|_| CliError::Compose(format!("path `{s}`: n `{n}` is not a small number")))?;
        Some(KeySet { k, n, sorted: true })
    };
    let mut path = SpendPath {
        keys,
        hash: None,
        lock: None,
    };
    for opt in parts {
        if opt == "unsorted" {
            match path.keys.as_mut() {
                Some(ks) => ks.sorted = false,
                None => {
                    return Err(CliError::Compose(format!(
                        "path `{s}`: `unsorted` needs keys"
                    )));
                }
            }
            continue;
        }
        let (name, value) = opt.split_once('=').ok_or_else(|| {
            CliError::Compose(format!("path `{s}`: option `{opt}` needs a value"))
        })?;
        match name {
            "older" if path.lock.is_none() => {
                path.lock = Some(if let Some(units) = value.strip_suffix('u') {
                    Lock::OlderUnits(parse_u16(units, "older units")?)
                } else {
                    // A number above 65535 is refused by the codec with the
                    // §4c wording; parse as u32 so the message names the band.
                    let v = parse_u32(value, "older blocks")?;
                    match u16::try_from(v) {
                        Ok(b) => Lock::OlderBlocks(b),
                        Err(_) => {
                            return Err(CliError::Compose(format!(
                                "path `{s}`: older in blocks needs 1..=65535, got {v}"
                            )));
                        }
                    }
                });
            }
            "after" if path.lock.is_none() => {
                path.lock = Some(if let Some(t) = value.strip_suffix('t') {
                    Lock::AfterTime(parse_u32(t, "after time")?)
                } else {
                    let h = parse_u32(value, "after height")?;
                    if h >= md_codec::compose::LOCKTIME_THRESHOLD {
                        // The band refusal alone never names the remedy; the
                        // operator who typed a Unix time needs the suffix.
                        return Err(CliError::Compose(format!(
                            "path `{s}`: after={h} reads as a block height and is above the height band (1..=499999999); for a Unix time write after={h}t"
                        )));
                    }
                    Lock::AfterHeight(h)
                });
            }
            "older" | "after" => {
                return Err(CliError::Compose(format!(
                    "path `{s}`: at most one lock per path"
                )));
            }
            "sha256" => {
                if path.hash.is_some() {
                    return Err(CliError::Compose(format!(
                        "path `{s}`: at most one hash per path"
                    )));
                }
                let h = parse_sha256_hex(value, &format!("path `{s}`"))?;
                path.hash = Some(h);
            }
            other => {
                return Err(CliError::Compose(format!(
                    "path `{s}`: unknown option `{other}`"
                )));
            }
        }
    }
    Ok(path)
}

/// `value` as 32 lowercase-hex bytes, or a `{ctx}: sha256 needs ...` refusal.
/// Shared by `--path ...,sha256=HEX` and `--preset hashlock-gated,sha256=HEX`.
fn parse_sha256_hex(value: &str, ctx: &str) -> Result<[u8; 32], CliError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(CliError::Compose(format!(
            "{ctx}: sha256 needs 64 hex characters, lowercase"
        )));
    }
    let mut h = [0u8; 32];
    for (i, chunk) in value.as_bytes().chunks(2).enumerate() {
        let hi = (chunk[0] as char).to_digit(16).expect("checked") as u8;
        let lo = (chunk[1] as char).to_digit(16).expect("checked") as u8;
        h[i] = (hi << 4) | lo;
    }
    Ok(h)
}

fn hex32(h: &[u8; 32]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(64);
    for b in h {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// The resolved parameters of one `--preset` invocation, named for `--json`'s
/// `preset` field (SPEC §4d, C2). One variant per `md_codec::compose::presets`
/// constructor, same field names as its arguments.
#[derive(Debug, Clone, Copy)]
pub enum PresetParams {
    PlainMultisig {
        k: u8,
        n: u8,
    },
    SimpleTimelockedInheritance {
        older_blocks: u32,
    },
    KofnRecovery {
        k: u8,
        n: u8,
        older_blocks: u32,
    },
    TieredRecovery {
        k1: u8,
        n1: u8,
        k2: u8,
        n2: u8,
        older_blocks: u32,
    },
    HashlockGated {
        sha256: [u8; 32],
        older_blocks: u32,
    },
    DecayingMultisig {
        k1: u8,
        n1: u8,
        k2: u8,
        n2: u8,
        older1: u32,
        older2: u32,
        after_height: u32,
    },
}

/// The six archetype names, kebab-case, in the order `--preset --help` and
/// every "expected one of" refusal lists them.
pub const PRESET_NAMES: [&str; 6] = [
    "plain-multisig",
    "simple-timelocked-inheritance",
    "kofn-recovery",
    "tiered-recovery",
    "hashlock-gated",
    "decaying-multisig",
];

fn parse_kofn(tok: &str, ctx: &str) -> Result<(u8, u8), CliError> {
    let (k, n) = tok
        .split_once("of")
        .ok_or_else(|| CliError::Compose(format!("{ctx}: `{tok}` is not <k>of<n>")))?;
    let k = k
        .parse::<u8>()
        .map_err(|_| CliError::Compose(format!("{ctx}: k `{k}` is not a small number")))?;
    let n = n
        .parse::<u8>()
        .map_err(|_| CliError::Compose(format!("{ctx}: n `{n}` is not a small number")))?;
    Ok((k, n))
}

/// `--preset <name>[,<k>of<n>]*[,<param>=<value>]*` (SPEC §4d, C2; the CLI
/// grammar this task defines). The `<k>of<n>` tokens are consumed IN LISTED
/// ORDER to fill the archetype's key-set parameters (tier 1 before tier 2,
/// where an archetype has two); `<param>=<value>` tokens are matched BY NAME,
/// in any order, against exactly the constructor's remaining arguments.
/// `unsorted` is never a preset parameter: every `presets::*` key set is
/// `sorted: true` by construction (`presets::ks`), so there is nothing for it
/// to toggle. Every constructor call runs through `checked` (`validate`), so
/// a legacy-wrapper shape or an out-of-band lock surfaces as the SAME
/// `ComposeError` a hand-built `--path` list with the same shape would give.
pub fn parse_preset(wrapper: Wrapper, s: &str) -> Result<(PresetParams, PathList), CliError> {
    let mut parts = s.split(',');
    let name = parts.next().unwrap_or("");
    if !PRESET_NAMES.contains(&name) {
        return Err(CliError::Compose(format!(
            "--preset {name}: expected one of {}",
            PRESET_NAMES.join(", ")
        )));
    }
    let ctx = format!("preset {name}");
    let mut ofs: Vec<(u8, u8)> = Vec::new();
    let mut named: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    for tok in parts {
        match tok.split_once('=') {
            Some((k, v)) => {
                if named.insert(k, v).is_some() {
                    return Err(CliError::Compose(format!("{ctx}: `{k}=` given twice")));
                }
            }
            None => ofs.push(parse_kofn(tok, &ctx)?),
        }
    }
    let need_ofs = |want: usize| -> Result<(), CliError> {
        if ofs.len() != want {
            return Err(CliError::Compose(format!(
                "{ctx} needs exactly {want} <k>of<n> parameter{}, got {}",
                if want == 1 { "" } else { "s" },
                ofs.len()
            )));
        }
        Ok(())
    };
    let named_only = |allowed: &[&str]| -> Result<(), CliError> {
        for k in named.keys() {
            if !allowed.contains(k) {
                return Err(CliError::Compose(format!("{ctx} admits no {k}= parameter")));
            }
        }
        Ok(())
    };
    let need_u32 = |k: &str| -> Result<u32, CliError> {
        let v = named
            .get(k)
            .ok_or_else(|| CliError::Compose(format!("{ctx} needs {k}=<n>")))?;
        parse_u32(v, &format!("{ctx} {k}"))
    };
    // `presets::decaying_multisig`'s `after_height` argument always builds
    // `Lock::AfterHeight` (never `AfterTime`), and the preset grammar has no
    // `t`-suffix to ask for a time lock at all -- unlike `--path`'s
    // `after=<H>|after=<T>t`. A value at or above the Unix-time band therefore
    // cannot be satisfied by retyping it; the only remedy is `--path`, which
    // this names, mirroring `--path`'s own "reads as a block height" wording
    // (`parse_path`'s `after` arm, above) rather than propagating the bare
    // `ComposeError::LockOutOfRange` text with no remedy.
    let need_after_height = |k: &str| -> Result<u32, CliError> {
        let v = need_u32(k)?;
        if v >= md_codec::compose::LOCKTIME_THRESHOLD {
            return Err(CliError::Compose(format!(
                "{ctx}: {k}={v} reads as a block height and is above the height band (1..=499999999); presets cannot express a Unix time -- use --path with `after={v}t` instead"
            )));
        }
        Ok(v)
    };
    let map_ce = |e: md_codec::compose::ComposeError| CliError::Compose(e.to_string());
    match name {
        "plain-multisig" => {
            need_ofs(1)?;
            named_only(&[])?;
            let (k, n) = ofs[0];
            let list = presets::plain_multisig(wrapper, k, n).map_err(map_ce)?;
            Ok((PresetParams::PlainMultisig { k, n }, list))
        }
        "simple-timelocked-inheritance" => {
            need_ofs(0)?;
            named_only(&["older"])?;
            let older_blocks = need_u32("older")?;
            let list =
                presets::simple_timelocked_inheritance(wrapper, older_blocks).map_err(map_ce)?;
            Ok((
                PresetParams::SimpleTimelockedInheritance { older_blocks },
                list,
            ))
        }
        "kofn-recovery" => {
            need_ofs(1)?;
            named_only(&["older"])?;
            let (k, n) = ofs[0];
            let older_blocks = need_u32("older")?;
            let list = presets::kofn_recovery(wrapper, k, n, older_blocks).map_err(map_ce)?;
            Ok((PresetParams::KofnRecovery { k, n, older_blocks }, list))
        }
        "tiered-recovery" => {
            need_ofs(2)?;
            named_only(&["older"])?;
            let (k1, n1) = ofs[0];
            let (k2, n2) = ofs[1];
            let older_blocks = need_u32("older")?;
            let list =
                presets::tiered_recovery(wrapper, k1, n1, k2, n2, older_blocks).map_err(map_ce)?;
            Ok((
                PresetParams::TieredRecovery {
                    k1,
                    n1,
                    k2,
                    n2,
                    older_blocks,
                },
                list,
            ))
        }
        "hashlock-gated" => {
            need_ofs(0)?;
            named_only(&["sha256", "older"])?;
            let hex = named
                .get("sha256")
                .ok_or_else(|| CliError::Compose(format!("{ctx} needs sha256=<64 hex>")))?;
            let sha256 = parse_sha256_hex(hex, &ctx)?;
            let older_blocks = need_u32("older")?;
            let list = presets::hashlock_gated(wrapper, sha256, older_blocks).map_err(map_ce)?;
            Ok((
                PresetParams::HashlockGated {
                    sha256,
                    older_blocks,
                },
                list,
            ))
        }
        "decaying-multisig" => {
            need_ofs(2)?;
            named_only(&["older1", "older2", "after"])?;
            let (k1, n1) = ofs[0];
            let (k2, n2) = ofs[1];
            let older1 = need_u32("older1")?;
            let older2 = need_u32("older2")?;
            let after_height = need_after_height("after")?;
            let list =
                presets::decaying_multisig(wrapper, k1, n1, k2, n2, older1, older2, after_height)
                    .map_err(map_ce)?;
            Ok((
                PresetParams::DecayingMultisig {
                    k1,
                    n1,
                    k2,
                    n2,
                    older1,
                    older2,
                    after_height,
                },
                list,
            ))
        }
        other => Err(CliError::Compose(format!(
            "preset {other}: internal error -- PRESET_NAMES advertises this name but no lowering rule exists for it (this is a bug in md, not a mistake in your command)"
        ))),
    }
}

#[cfg(feature = "json")]
fn preset_params_json(p: &PresetParams) -> serde_json::Value {
    let (name, params) = match *p {
        PresetParams::PlainMultisig { k, n } => {
            ("plain-multisig", serde_json::json!({ "k": k, "n": n }))
        }
        PresetParams::SimpleTimelockedInheritance { older_blocks } => (
            "simple-timelocked-inheritance",
            serde_json::json!({ "older_blocks": older_blocks }),
        ),
        PresetParams::KofnRecovery { k, n, older_blocks } => (
            "kofn-recovery",
            serde_json::json!({ "k": k, "n": n, "older_blocks": older_blocks }),
        ),
        PresetParams::TieredRecovery {
            k1,
            n1,
            k2,
            n2,
            older_blocks,
        } => (
            "tiered-recovery",
            serde_json::json!({ "k1": k1, "n1": n1, "k2": k2, "n2": n2, "older_blocks": older_blocks }),
        ),
        PresetParams::HashlockGated {
            sha256,
            older_blocks,
        } => (
            "hashlock-gated",
            serde_json::json!({ "sha256": hex32(&sha256), "older_blocks": older_blocks }),
        ),
        PresetParams::DecayingMultisig {
            k1,
            n1,
            k2,
            n2,
            older1,
            older2,
            after_height,
        } => (
            "decaying-multisig",
            serde_json::json!({ "k1": k1, "n1": n1, "k2": k2, "n2": n2, "older1": older1, "older2": older2, "after_height": after_height }),
        ),
    };
    serde_json::json!({ "name": name, "params": params })
}

fn describe(e: &Experimental) -> String {
    match e {
        Experimental::KeylessPath(i) => format!(
            "path {} has no key (bearer access to whoever holds the preimage)",
            i + 1
        ),
        Experimental::UnsortedKeys(i) => format!(
            "path {} uses unsorted keys where sorted was possible (key order is part of this wallet)",
            i + 1
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    wrapper: &str,
    paths: &[String],
    preset: Option<&str>,
    experimental: bool,
    json: bool,
) -> Result<u8, CliError> {
    let wrapper = parse_wrapper(wrapper)?;
    let (list, preset_params): (PathList, Option<PresetParams>) = match preset {
        Some(spec) => {
            let (params, list) = parse_preset(wrapper, spec)?;
            (list, Some(params))
        }
        None => {
            let paths: Vec<SpendPath> = paths
                .iter()
                .map(|p| parse_path(p))
                .collect::<Result<_, _>>()?;
            (PathList { wrapper, paths }, None)
        }
    };
    let composed = compose(&list).map_err(|e| CliError::Compose(e.to_string()))?;
    if !composed.experimental.is_empty() && !experimental {
        let mut msg = String::from("this policy needs --experimental:");
        for e in &composed.experimental {
            msg.push_str("\n  ");
            msg.push_str(&describe(e));
        }
        return Err(CliError::Compose(msg));
    }
    for e in &composed.experimental {
        eprintln!("warning: EXPERIMENTAL: {}", describe(e));
    }
    // `unsorted` where sorted was never available is dropped by the lowering
    // (spec §5a: the §8b confirm fires only where sorted was legal); say so
    // rather than accept a typed request silently. No preset ever sets
    // `sorted: false` (`presets::ks` always sorts), so this loop is inert for
    // every `--preset` list and unchanged from `--path`'s behaviour.
    for (i, p) in list.paths.iter().enumerate() {
        if matches!(p.keys, Some(KeySet { n, sorted: false, .. }) if n >= 2)
            && !composed
                .experimental
                .contains(&Experimental::UnsortedKeys(i))
        {
            eprintln!(
                "note: path {}: `unsorted` has no effect here; sorted keys are not available in this position, so it is multi either way",
                i + 1
            );
        }
    }
    let template = descriptor_to_template(&composed.descriptor).map_err(CliError::Render)?;
    let with_origins = template_with_origins(&composed).map_err(CliError::Render)?;

    #[cfg(feature = "json")]
    if json {
        use crate::format::json::SCHEMA;
        let slots: Vec<serde_json::Value> = composed
            .slots
            .iter()
            .map(|s| serde_json::json!({ "index": s.index, "path": s.path, "ordinal": s.ordinal }))
            .collect();
        let exp: Vec<String> = composed.experimental.iter().map(describe).collect();
        let preset_json = preset_params.as_ref().map(preset_params_json);
        let v = serde_json::json!({
            "schema": SCHEMA,
            "template": template,
            "template_with_origins": with_origins,
            "wrapper": wrapper_name(wrapper),
            "slots": slots,
            "internal_key_path": composed.internal_key_path,
            "experimental": exp,
            "preset": preset_json,
        });
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        crate::output_advisory::emit_output_class_advisory(
            crate::output_advisory::OutputClass::Template,
            &mut std::io::stderr(),
        );
        return Ok(0);
    }
    let _ = json;

    // The inline-origin form: what `md encode` reads back to the same card.
    println!("{with_origins}");
    crate::output_advisory::emit_output_class_advisory(
        crate::output_advisory::OutputClass::Template,
        &mut std::io::stderr(),
    );
    Ok(0)
}

fn wrapper_name(w: Wrapper) -> &'static str {
    match w {
        Wrapper::Tr => "tr",
        Wrapper::Wsh => "wsh",
        Wrapper::ShWsh => "sh-wsh",
        Wrapper::Sh => "sh",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // R0 round-1 fold-verification (Important): the ORIGINAL version of this
    // test iterated a hand-typed `[(&str, &str); 6]` fixture and asserted its
    // `.len() == 6` -- a tautology that cannot fail under any edit to
    // `PRESET_NAMES` or the `match` in `parse_preset`. Confirmed live: adding
    // a 7th, unmatched name to `PRESET_NAMES` compiled, passed clippy, passed
    // all 31 CLI tests, and then PANICKED (`unreachable!()`, exit 101) on a
    // real `md compose --preset <name>,...` invocation. This version iterates
    // `PRESET_NAMES` ITSELF and calls `parse_preset` directly (only possible
    // from inside this crate -- `cli_compose_preset.rs` is a black-box
    // integration test with no access to either), so a name added to
    // `PRESET_NAMES` with no matching valid-parameter fixture or no matching
    // `match` arm fails HERE, not in production.
    #[test]
    fn every_preset_name_parses_with_some_valid_parameters() {
        fn valid_params(name: &str) -> &'static str {
            match name {
                "plain-multisig" => "2of3",
                "simple-timelocked-inheritance" => "older=26280",
                "kofn-recovery" => "2of3,older=26280",
                "tiered-recovery" => "2of2,1of2,older=26280",
                "hashlock-gated" => {
                    "sha256=a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8,older=26280"
                }
                "decaying-multisig" => "2of2,1of1,older1=13140,older2=26280,after=1000000",
                other => panic!(
                    "PRESET_NAMES gained `{other}` with no valid-parameter fixture in this test"
                ),
            }
        }
        for name in PRESET_NAMES {
            let spec = format!("{name},{}", valid_params(name));
            parse_preset(Wrapper::Wsh, &spec).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }
}
```

- [ ] **Step 4: Wire `--preset` into `main.rs`**

`crates/md-cli/src/main.rs:282-297` currently reads (verbatim, at `66bdf2f4`):

```text
    Compose {
        /// tr | wsh | sh-wsh | sh
        #[arg(long, value_name = "WRAPPER", required = true)]
        wrapper: String,
        /// One spend path in listed order: `<k>of<n>[,older=N|older=Nu|after=H|after=Tt][,sha256=HEX][,unsorted]`
        /// or `keyless,sha256=HEX[,older=..|after=..]`. Repeatable.
        #[arg(long = "path", value_name = "PATH", required = true, action = clap::ArgAction::Append)]
        paths: Vec<String>,
        /// Admit key-less paths and unsorted-where-sorted-was-legal, with a warning.
        #[arg(long)]
        experimental: bool,
        /// Emit JSON: the origin-less template, the inline-origin template, the
        /// slot map, the taproot internal-key path and the EXPERIMENTAL marks.
        #[arg(long)]
        json: bool,
    },
```

Replace it with:

```text
    #[command(group = clap::ArgGroup::new("path_source").required(true).args(["paths", "preset"]))]
    Compose {
        /// tr | wsh | sh-wsh | sh
        #[arg(long, value_name = "WRAPPER", required = true)]
        wrapper: String,
        /// One spend path in listed order: `<k>of<n>[,older=N|older=Nu|after=H|after=Tt][,sha256=HEX][,unsorted]`
        /// or `keyless,sha256=HEX[,older=..|after=..]`. Repeatable. Mutually
        /// exclusive with --preset.
        #[arg(long = "path", value_name = "PATH", action = clap::ArgAction::Append)]
        paths: Vec<String>,
        /// One of the six named archetypes (SPEC_wallet_policy_composer.md
        /// §4d): `<name>[,<k>of<n>]*[,<param>=<value>]*`, e.g.
        /// `kofn-recovery,2of3,older=26280`. Mutually exclusive with --path.
        /// For decaying-multisig, `older1` locks the FIRST (primary) tier and
        /// `older2` the second -- the primary tier does not spend immediately.
        #[arg(long, value_name = "PRESET")]
        preset: Option<String>,
        /// Admit key-less paths and unsorted-where-sorted-was-legal, with a warning.
        #[arg(long)]
        experimental: bool,
        /// Emit JSON: the origin-less template, the inline-origin template, the
        /// slot map, the taproot internal-key path, the EXPERIMENTAL marks, and
        /// (with --preset) the resolved preset name and parameters.
        #[arg(long)]
        json: bool,
    },
```

(`--path` drops its own `required = true`: the group's `.required(true)` now enforces "at least one of `paths`/`preset`", and the group's default `multiple(false)` enforces "at most one" — clap's precedent for this exact pattern is the pre-existing `descriptor_input`/`address_input` groups at `crates/md-cli/src/main.rs:315,551`, which use `.multiple(true)` instead because THEIR members may legally combine; `path_source` deliberately does not.)

`crates/md-cli/src/main.rs:993-998` currently reads:

```text
        Command::Compose {
            wrapper,
            paths,
            experimental,
            json,
        } => cmd::compose::run(&wrapper, &paths, experimental, json),
```

Replace it with:

```text
        Command::Compose {
            wrapper,
            paths,
            preset,
            experimental,
            json,
        } => cmd::compose::run(&wrapper, &paths, preset.as_deref(), experimental, json),
```

No change to `crates/md-cli/src/error.rs`: `CliError::Compose(String)` already exists (`:46,132`) and every new refusal above uses it.

- [ ] **Step 5: Run the CLI tests and the full md-cli suite**

Run: `cargo nextest run --locked -p md-cli --no-fail-fast -E "binary(/^cli_compose/) + test(every_preset_name_parses_with_some_valid_parameters)" 2>&1 | tail -30` (the second clause reaches the `#[cfg(test)]` unit test embedded in `compose.rs`, run under the `bin/md` binary, not a `cli_compose*` one)
Expected: 31/31 PASS — the 9 pre-existing `cli_compose.rs`/`cli_compose_encode_gate.rs` tests, 21 in `cli_compose_preset.rs` (14 from the original draft, 8 added folding R0 fidelity I-3/M-2/M-3/M-4, minus 1 moved out in the R0 round-1 fold — see below), plus 1 `#[cfg(test)]` unit test embedded in `compose.rs` itself (`cmd::compose::tests::every_preset_name_parses_with_some_valid_parameters`, R0 round-1: the black-box integration test cannot call `parse_preset` or read `PRESET_NAMES`, so the real M-4 coverage moved to where those are visible); measured live, zero regressions.

Run: `cargo nextest run --locked -p md-cli 2>&1 | tail -8`
Expected: 783/783 PASS once Task 1's corpus is regenerated (Task 3); measured live at 782/783 with only the not-yet-regenerated `vectors_output_matches_committed_corpus` red before that regeneration — the pre-existing `gui-schema`/`cli_output_class` tests are unaffected (`grep -n compose` on both source files returns nothing, so `--preset` adds no golden fixture to update, unlike S0 Task 6's own caution about that class of test).

- [ ] **Step 6: Format, clippy, commit**

Run: `cargo fmt --all && cargo clippy --locked -p md-cli --all-targets --all-features -- -D warnings 2>&1 | tail -3`
Expected: clean (measured: zero warnings on this exact diff, after `cargo fmt --all` normalised line-wrapping in `compose.rs` and `cli_compose_preset.rs`).

```bash
git add crates/md-cli/src/cmd/compose.rs crates/md-cli/src/main.rs crates/md-cli/tests/cli_compose_preset.rs
git commit -m "md-cli: md compose --preset -- the six archetypes over presets::*, mutually exclusive with --path, --json preset field (F-453 composer S0b task 2)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 3: Whole-workspace gate, corpus regeneration, release notes

**Files:**
- Modify: `crates/md-codec/tests/vectors/` (regenerated, new files only)
- Modify: `descriptor-mnemonic/CHANGELOG.md` (repo root; `## md-cli [Unreleased]` at `:7`, `## md-codec [Unreleased]` at `:69`, both already exist — unlike S0 Task 9, no new heading is needed)

- [ ] **Step 1: Run the whole workspace the way CI does**

Run: `cd /scratch/code/shibboleth/descriptor-mnemonic && cargo fmt --all --check && cargo clippy --locked --workspace --all-targets --all-features -- -D warnings 2>&1 | tail -3 && cargo nextest run --locked --workspace --all-features 2>&1 | tail -6 && cargo test --locked --workspace --all-features --doc 2>&1 | tail -3`
Expected: fmt clean, clippy clean, every test PASS, doctests PASS. Baseline measured at `66bdf2f4` before this plan's changes: `1318 tests run: 1318 passed, 3 skipped`. This plan adds 22 tests total (21 in `cli_compose_preset.rs` — 14 from the original draft plus 8 folding R0 fidelity I-3/M-2/M-3/M-4, minus 1 moved to `compose.rs` in the R0 round-1 fold — plus 1 `#[cfg(test)]` unit test embedded in `compose.rs` itself) and zero new `#[test]` functions in md-codec (the six new `family()` rows, and the `head:hashed` tag folding N-1, are data, consumed by pre-existing tests) — expect **1340 tests run: 1340 passed, 3 skipped**. (R0 tests-lens N-2: a scratch copy that has not been through the CURRENT `plan-build-gate-md.sh` -- specifically its `design/display-grouping-vectors.tsv*` copy step, added `a13feec` -- shows one extra red here, `md-codec::display_grouping_conformance::conformance_vectors_pass`, for the missing sidecar file; that is a scratch-setup gap already root-caused and fixed in the gate script, not a defect in this plan's diff, and does not reproduce when the current script is used.) Doctests: 0 for md-codec, unaffected either way (measured: this crate ships none).

- [ ] **Step 2: Regenerate and diff the vector corpus**

Run: `cargo run --locked -p md-cli --bin md -- vectors --out crates/md-codec/tests/vectors 2>&1 | tail -2 && git status --short crates/md-codec/tests/vectors | head -40`
Expected: only NEW `keyed_compose_preset_*` files appear — 30 of them (six vectors × five files: `.bytes.hex`, `.phrase.txt`, `.descriptor.json`, `.template`, `.conformance.json`; measured live during authoring); no existing vector file changes (a changed pre-existing file means this task's addition altered something it must not — the six new entries are pure additions, appended after the last existing `MANIFEST` entry, so nothing upstream of them should re-render differently).

- [ ] **Step 3: Record the release note and commit**

Add to `descriptor-mnemonic/CHANGELOG.md`, under the existing `## md-cli [Unreleased]` heading (`:7`), in its `### Added` list, directly after the existing `md compose` bullet:

```text
- `md compose --preset <name>[,<k>of<n>]*[,<param>=<value>]*` (F-453): the six
  `md_codec::compose::presets` archetypes (`plain-multisig`,
  `simple-timelocked-inheritance`, `kofn-recovery`, `tiered-recovery`,
  `hashlock-gated`, `decaying-multisig`) as one-tap presets, mutually
  exclusive with `--path` (a clap `ArgGroup`; exactly one of the two is
  required). `--json` gains a `preset` field naming the resolved preset and
  its parameters (`null` for a `--path`-built policy). No new lowering
  behaviour: every preset is a thin call into the already-shipped
  constructors. See `design/SPEC_wallet_policy_composer.md` §4d.
```

Add to `descriptor-mnemonic/CHANGELOG.md`, under the existing `## md-codec [Unreleased]` heading (`:69`), in its `### Added` list, directly after the existing "28 tagged compose vectors" bullet:

```text
- Six more compose vectors, `keyed_compose_preset_<name>`, one per archetype
  (F-453) -- the family and the MANIFEST grow to 34 tagged / 32 in MANIFEST,
  all keyed, all within the four journey xpubs, each carrying a new singular
  `preset:<name>` tag.
```

```bash
git add crates/md-codec/tests/vectors CHANGELOG.md
git commit -m "md-codec/md-cli: preset vector corpus regenerated; release notes (F-453 composer S0b task 3)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

- [ ] **Step 4: Hand off**

The stage is complete when: every task's commit exists; the workspace gate of this task is green; the whole-diff independent review (an opus execution review over `git diff <baseline>..HEAD`, persisted to `mnemonic-engrave/design/agent-reports/composer-S0b-exec-review-r0.md`) returns 0 Critical / 0 Important after folds; and the branch is merged to `main` via the `ci/staging` ritual (same as S0). The version bump + crates.io publish remain BLOCKED by the pre-existing `md-codec-derive-feature-depends-on-unpublished-miniscript-apis` follow-up (unaffected by this plan; `--preset` uses no miniscript API).

**Fork re-vendoring (named here, NOT this plan's work — Rust first, the fork is untouched by this stage). R0 fidelity I-4: ownership decided, not left open.** Once this plan merges to `main`, the fork's `scripts/vendor-compose-vectors.sh` (`/scratch/code/shibboleth/seedhammer`, currently `main` at `321acb5`) re-run against the new `descriptor-mnemonic` revision picks up the six `keyed_compose_preset_*` names with NO script change (its glob is `^(keyed_)?compose_`, confirmed by reading `scripts/vendor-compose-vectors.sh:16` on the fork). The vendored file count moves from the currently-committed **126** (22 keyed × 5 + 4 unkeyed × 4, per the fork's own `md/compose_vectors_pin_test.go:82-84`) to **156** (28 keyed × 5 + 4 unkeyed × 4 — measured directly: this plan's own scratch run produced exactly 156 files in `crates/md-codec/tests/vectors/` after regeneration). **`IMPLEMENTATION_PLAN_composer_S3_fork_gui.md` Task A10 owns two hand edits that re-running the script alone will not make, and A10's own text currently gets both wrong** (a correction for the controller to schedule on the S3 plan, NOT made here): its `md/compose_vectors_pin_test.go`'s hardcoded `composeVectorNames` list (26 names) needs the six new names added, and its hardcoded `126` (`:83-84`) needs to become `156` — but A10 Step 1 (`:4709`) currently asserts the opposite, that the pin test "passes at the new counts (it asserts the file count, so the constant... moves with it)", which is false: both the name list and the `126` literal are Go constants that move only by hand. A10's own Files list (`:4686-4689`) does not name `compose_vectors_pin_test.go` and its `git add` (`:4736`) does not stage it, either. A10 also names the wrong vendored prefix (`preset_*` instead of the actual `keyed_compose_preset_*`) and omits the fifth vendored file per vector (`.conformance.json`, required by the fork's own `TestEveryKeyedComposeVectorHasAConformanceRecord`). This plan states the ownership and the exact expected counts (126 → 156, 26 → 32 names) so the S3-side fix is unambiguous; it does not itself touch the fork or `IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`.

## Self-review (run by the plan author before dispatch)

1. **Spec coverage:** §4d's six archetypes and their parameters → Task 1 (constructors already shipped; this plan adds the corpus) and Task 2 (the CLI grammar); §4d's "under sh/sh(wsh) only plain-multisig" rule → Task 2's refusal table and its `preset_every_non_plain_archetype_refuses_under_both_legacy_wrappers_spec_4d_shape` test, covering all ten (archetype, wrapper) pairs (R0 fidelity M-2; no new code needed — the existing `ComposeError::LegacyWrapperShape` already covers it); §12 item 1's tagged-coverage requirement → Task 1's `family()` rows and `SINGULAR_TAGS` additions (verified NOT to break `every_tag_appears_in_at_least_two_vectors`'s required list: one new tag STRING is introduced beyond the six deliberately-singular `preset:*` ones — `head:hashed`, folding R0 fidelity N-1 — and it is added to `SINGULAR_TAGS` too, with exactly one vector, so the required list and the two-vector rule are unaffected). Not in this stage: any change to §5's lowering, §9's device work, or the fork (Stage 2 shipped the Go builder already; Stage 3's GUI picker is the eventual consumer of what this plan produces, named but not built here; Task 3 states which two fork-side edits Stage 3's Task A10 owns, per R0 fidelity I-4).
2. **Placeholder scan:** no TBD/TODO; every code block is the ACTUAL content verified in a scratch build before this plan was written (see "What is already machine-verified"), not a description of what code should do.
3. **Type consistency:** `presets::plain_multisig(Wrapper, u8, u8) -> Result<PathList, ComposeError>`, `presets::simple_timelocked_inheritance(Wrapper, u32) -> Result<PathList, ComposeError>`, `presets::kofn_recovery(Wrapper, u8, u8, u32) -> Result<PathList, ComposeError>`, `presets::tiered_recovery(Wrapper, u8, u8, u8, u8, u32) -> Result<PathList, ComposeError>`, `presets::hashlock_gated(Wrapper, [u8; 32], u32) -> Result<PathList, ComposeError>`, `presets::decaying_multisig(Wrapper, u8, u8, u8, u8, u32, u32, u32) -> Result<PathList, ComposeError>` are used with these EXACT shapes (read from `crates/md-codec/src/compose/presets.rs` at `66bdf2f4`) in every task. `cmd::compose::run`'s new signature `(wrapper: &str, paths: &[String], preset: Option<&str>, experimental: bool, json: bool) -> Result<u8, CliError>` is used identically in Task 2's `main.rs` dispatch arm.

## What is NOT covered by any gate

Unlike S0, this plan needs no NEW build-gate SCRIPT — `scripts/plan-build-gate-md.sh` already covers every gate-assembled file this plan touches (see Global Constraints). But its bare run stops one step short of md-cli's compile-check for THIS plan specifically (measured, not assumed — see "What is already machine-verified"): step 6 fails until `crates/md-cli/src/main.rs`'s clap variant and dispatch arm (Task 2 Step 4) are hand-wired into the scratch copy, because this plan changes an existing function's arity that main.rs's real, un-patched dispatch arm still calls with the old signature. Hand-wire that fragment before trusting step 6's result. What remains genuinely uncovered even after that hand-wiring, same as S0: `crates/md-codec/src/test_vectors.rs`'s pasted `MANIFEST` entries (Task 1 Step 2; hand-verified in this plan's own scratch run, not by the gate); the exporter's file count and the CHANGELOG prose (Task 3; hand-verified live during authoring, re-verify at fold time since the gate does not check either). The fork is out of scope end to end — Task 3's vendoring note is a pointer, not a gate.
