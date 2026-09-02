# Composer Stage 0 Implementation Plan — md-codec `compose`, `md compose`, and the compose vector family

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the normative, fixed lowering from an ordered spend-path list to a BIP-388 template in Rust (`descriptor-mnemonic`), with its refusals, its `md compose` surface and a tagged vector family the Go port will be measured against.

**Architecture:** A new `md_codec::compose` module builds md-codec's existing `Descriptor` tree directly (`Node`/`Body`/`Tag`), so the Go port mirrors a tree construction; text comes from the existing renderer, round-trip from the existing parser, encoding from the existing serialiser. `md compose` is a thin subcommand beside `md compile` with the opposite contract (fixed rules, no search). Vectors join `test_vectors::MANIFEST` so `md vectors` exports them into the `.conformance.json` files the fork's conformance test already consumes.

**Tech Stack:** Rust 2024 edition workspace at `/scratch/code/shibboleth/descriptor-mnemonic`; crates `md-codec` (lib, `derive` feature pulls `miniscript` 13.0.0 via workspace patch `ff4732e`) and `md-cli` (binary `md`, clap, features `json` default and `cli-compiler`); `cargo nextest run --locked`; `cargo fmt --check`; `cargo clippy -D warnings`.

**Spec:** `design/SPEC_wallet_policy_composer.md` (R0 rounds 0-3 folded). Stages: `design/STAGED_PLAN_wallet_policy_composer.md`. Rulings: `design/BRAINSTORM_wallet_policy_composer.md` §2, §3.12.

**Baseline revision (for `scripts/plan-staleness-check.sh`):** descriptor-mnemonic `3b0944fb` (docs-only commits `480e54fe`, `b19dca7b` since).

**STATUS: R0 GREEN 2026-09-02 (0 Critical / 0 Important open).** Round 0: fidelity (opus, 0C/4I/5M/3N) and mutation/claims (sonnet, 0C/2I/4M/2N), folded at `891b17d` and `fb65f2c`; round 1: fold verification 20/20 FIXED, one new Minor folded at `761ded7`; round 2: verification of the post-round-1 folds, 0 new defects / 0 false claims. Build gate green at every fold (toolchain 1.85.0; 52 compose tests, 51 pass + the pinned MANIFEST red); the CLI and Task 8 fragments hand-wired in the gate's scratch copy ran the whole md-cli suite 761/761. Lenses run on this plan: fidelity-to-spec by constructed counterexample, can-every-test-fail by mutation, fold verification ×2. Implementation may begin: one implementer, UC off, in a descriptor-mnemonic worktree.

## Global Constraints

- Rust first: nothing in the fork is touched by this stage (CLAUDE.md Rust-primary rule).
- `md_codec::Error` stays a pure wire/decode taxonomy; composition errors are a separate `ComposeError` (`crates/md-codec/src/render.rs:17` states the same rule for `RenderError`).
- Every composed template must satisfy the wire's own admission when encoded: `encode_payload` canonicalises placeholder indices and validates (`crates/md-codec/src/encode.rs:99-121`); the composer emits first-appearance numbering ITSELF so that `canonicalize_placeholder_indices` is the identity on its output (spec §5 numbering row).
- Bounds, verbatim from the spec: paths 1..=8; per path n 1..=9, 1 ≤ k ≤ n; total slots 1..=32 (`crates/md-codec/src/error.rs:57-59` `KeyCountOutOfRange`); `older` 1..=65535 blocks or `0x400000 + u`, u in 1..=65535; `after` 1..=499,999,999 height, 500,000,000..=2,147,483,647 time (spec §4c).
- Lowering rules verbatim from spec §5: `or_d(P, R)` iff P is a bare unlocked, unhashed `multi` with n ≥ 2, else `or_i(P, R)`; `and_v(v:KEYS, and_v(v:sha256(H), LOCK))`; sole unlocked unhashed multi-key path → `sortedmulti`/`sortedmulti_a`, any other multi-key path → `multi`/`multi_a`; one key → `pkh` in wsh, `pk` in tr; internal key = first-listed unlocked, unhashed one-key path, else NUMS; tr spine right, leaf j at depth min(j, m−1); m = 1 → bare leaf; m = 0 → no tree; use-site `/<0;1>/*`.
- Origins, verbatim from spec §4f: `m/48'/0'/account'/T'` with T = 2 (wsh), 1 (sh(wsh)), 2 (sh), 3 (tr); an unseated slot takes the LOWEST account not already declared by any slot, in ascending emitted index; invariant: no two slots share an origin unless both declare distinct fingerprints.
- The rendered template TEXT carries no origins: `render::descriptor_to_template` writes `@0/<0;1>/*` and the origins live in `path_decl` (descriptor-mnemonic F-219; `md decode` prints them on stderr). Every origin assertion in this plan reads `path_decl`; the inline-origin form `@0/48'/0'/0'/2'/<0;1>/*` is produced only by `compose::template_with_origins` and is what `md compose` prints and what the vector corpus stores as `template`.
- Keyless paths are wsh-only and EXPERIMENTAL; unsorted where sorted was legal is EXPERIMENTAL; the library REPORTS both, the CLI requires `--experimental` to emit them. `md encode --experimental` is the precedent, but at `3b0944fb` `md encode` gates a signature-free spend path ONLY under `tr` (`Descriptor::from_str`'s tr-only sanity gate); a `wsh` template with such a path encodes with exit 0, keyed or unkeyed (measured; descriptor-mnemonic follow-up `md-encode-keyless-template-sigless-path-not-gated`, owned by this stage). Task 8 closes that so `md compose` → `md encode` agree.
- Exit codes: `main` prints `md: {e}` and exits 1 for every `CliError` EXCEPT `BadArg`, which has its own arm and exits 2 (`crates/md-cli/src/main.rs:787-800`). `CliError::Compose` takes the generic arm: exit 1.
- `compose` is UNCONDITIONAL (not behind `cli-compiler`); only the compile cross-check test needs the `compiler` feature, as a dev-dependency.
- Build gate before every fold: `scripts/plan-build-gate-md.sh design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md` extracts every ```rust block anchored on the NEW files below (`Create`/`Add to` append, `Replace` supersedes), builds, runs the compose tests, clippies, and compile-checks md-cli. Fragments of existing files (`lib.rs`, `main.rs`, `cmd/mod.rs`, `test_vectors.rs`, `Cargo.toml`) are NOT assembled by it and are called out per task.
- Commits: stage paths explicitly (no `git add -A`); one task, one commit; messages end with the standing trailers.

---

## File Structure

New files (mechanically assembled by the build gate):

| file | responsibility |
| --- | --- |
| `crates/md-codec/src/compose/mod.rs` | public types (`Wrapper`, `Lock`, `KeySet`, `SpendPath`, `PathList`, `SlotOrigin`, `Composed`, `Slot`, `Experimental`, `ComposeError`), structural validation, `default_origin`, `compose` and `compose_with` |
| `crates/md-codec/src/compose/lowering.rs` | the path body, the wsh `or_d`/`or_i` chain, first-appearance numbering, §4f origins and the invariant, the shared `finish` |
| `crates/md-codec/src/compose/tr.rs` | internal-key extraction, the right spine, NUMS |
| `crates/md-codec/src/compose/presets.rs` | the five archetypes and plain k-of-n as path lists |
| `crates/md-codec/tests/compose_lowering.rs` | text-level and tree-level tests of the lowering, numbering identity, refusals, round trip through the wire |
| `crates/md-codec/tests/compose_support.rs` | helpers shared by the compose test files via `#[path]`: journey xpubs, per-slot distinct keys, `keyed()`, `concrete_policy()`, and (from Task 5) the tagged vector `family()` |
| `crates/md-codec/tests/compose_crosscheck.rs` | the §5b cross-check as one reusable `cross_check()`: convert, `sanity_check` (its documented failure for keyless wsh), an address, `lift` equality against the compiler; run over the reference wallets (Task 4), the whole family (Task 5) and every preset (Task 7) |
| `crates/md-cli/src/cmd/compose.rs` | `md compose`: the path DSL, `--experimental` gating, text and `--json` output |
| `crates/md-cli/tests/cli_compose.rs` | CLI tests: text round trip through `md encode` → `md decode`, refusals, JSON shape |
| `crates/md-cli/tests/cli_compose_encode_gate.rs` | `md encode` refuses a signature-free spend path under `wsh` unless `--experimental`, keyed and unkeyed (Task 8) |

Modified files (fragments; a reviewer's execution pass, and named in the task that touches them):

| file | change |
| --- | --- |
| `crates/md-codec/src/lib.rs` | `pub mod compose;` |
| `crates/md-codec/Cargo.toml` | dev-dependency `miniscript = { workspace = true, features = ["compiler"] }` |
| `crates/md-codec/src/test_vectors.rs` | the `compose_*` MANIFEST entries |
| `crates/md-cli/src/cmd/mod.rs` | `pub mod compose;` |
| `crates/md-cli/src/main.rs` | the `Compose { .. }` clap variant and its dispatch arm |
| `crates/md-cli/src/error.rs` | `CliError::Compose(String)` |
| `crates/md-cli/src/parse/template.rs` | `parse_template_ext` runs `sanity_check()` for every wrapper, and the `--experimental` relaxed re-check for `wsh`/`sh` as it already does for `tr` leaves (Task 8) |

---

### Task 1: The compose module's types and structural refusals

**Files:**
- Create: `crates/md-codec/src/compose/mod.rs`, `crates/md-codec/src/compose/lowering.rs` (a stub this task; Task 2 replaces it)
- Modify: `crates/md-codec/src/lib.rs` (one line after `pub mod codex32;`: `pub mod compose;`)
- Test: `crates/md-codec/tests/compose_lowering.rs`

**Interfaces:**
- Consumes: `md_codec::encode::Descriptor`, `md_codec::origin_path::{OriginPath, PathComponent}`, `md_codec::tag::Tag`.
- Produces: everything in the type block below, plus `template_with_origins(&Composed) -> Result<String, RenderError>`; later tasks add the lowering behind `compose` and `compose_with` without changing these signatures. Private helpers `SpendPath::is_bare_multi`, `SpendPath::is_bare_single` and `Wrapper::is_legacy` are visible to the child modules `lowering`, `tr` and `presets` (a child module sees its parent's private items).

- [ ] **Step 1: Write the failing structural-refusal tests**

Create `crates/md-codec/tests/compose_lowering.rs`:

```rust
//! Lowering tests for `md_codec::compose` (SPEC_wallet_policy_composer.md §4, §5).
//!
//! Every expected template string below is the FIXED spelling the Go port must
//! reproduce byte for byte; a change here is a normative change.

use md_codec::canonicalize::canonicalize_placeholder_indices;
use md_codec::chunk::{reassemble, split};
use md_codec::compose::{
    compose, compose_with, template_with_origins, ComposeError, Composed, Experimental, KeySet,
    Lock, PathList, SlotOrigin, SpendPath, Wrapper, MAX_PATHS, MAX_SLOTS,
};
use md_codec::encode::{encode_md1_string, encode_payload};
use md_codec::origin_path::{OriginPath, PathComponent, PathDeclPaths};
use md_codec::render::descriptor_to_template;

const H1: [u8; 32] = [0xa8; 32];

fn keys(k: u8, n: u8) -> SpendPath {
    SpendPath { keys: Some(KeySet { k, n, sorted: true }), hash: None, lock: None }
}

fn unsorted(k: u8, n: u8) -> SpendPath {
    SpendPath { keys: Some(KeySet { k, n, sorted: false }), hash: None, lock: None }
}

fn with_lock(mut p: SpendPath, lock: Lock) -> SpendPath {
    p.lock = Some(lock);
    p
}

fn with_hash(mut p: SpendPath, h: [u8; 32]) -> SpendPath {
    p.hash = Some(h);
    p
}

fn keyless(h: [u8; 32], lock: Option<Lock>) -> SpendPath {
    SpendPath { keys: None, hash: Some(h), lock }
}

fn list(wrapper: Wrapper, paths: Vec<SpendPath>) -> PathList {
    PathList { wrapper, paths }
}

fn hardened(values: &[u32]) -> OriginPath {
    OriginPath {
        components: values
            .iter()
            .map(|v| PathComponent { hardened: true, value: *v })
            .collect(),
    }
}

// ---- §4e structural refusals -------------------------------------------------

#[test]
fn compose_refuses_an_empty_path_list() {
    let err = compose(&list(Wrapper::Wsh, vec![])).unwrap_err();
    assert_eq!(err, ComposeError::NoPaths);
}

#[test]
fn compose_refuses_more_than_max_paths() {
    let paths: Vec<SpendPath> = (0..(MAX_PATHS + 1)).map(|_| keys(1, 1)).collect();
    let err = compose(&list(Wrapper::Wsh, paths)).unwrap_err();
    assert_eq!(err, ComposeError::TooManyPaths { got: MAX_PATHS + 1 });
}

#[test]
fn compose_refuses_a_policy_with_no_keyed_path() {
    let err = compose(&list(Wrapper::Wsh, vec![keyless(H1, None)])).unwrap_err();
    assert_eq!(err, ComposeError::NoKeyedPath);
}

#[test]
fn compose_refuses_a_lock_only_path() {
    let lock_only = SpendPath { keys: None, hash: None, lock: Some(Lock::OlderBlocks(100)) };
    let err = compose(&list(Wrapper::Wsh, vec![keys(1, 1), lock_only])).unwrap_err();
    assert_eq!(err, ComposeError::LockOnlyPath { path: 1 });
}

#[test]
fn compose_refuses_a_keyless_path_under_tr() {
    let err = compose(&list(Wrapper::Tr, vec![keys(2, 3), keyless(H1, None)])).unwrap_err();
    assert_eq!(err, ComposeError::KeylessUnderTr { path: 1 });
}

#[test]
fn compose_refuses_bad_thresholds() {
    assert_eq!(
        compose(&list(Wrapper::Wsh, vec![keys(0, 2)])).unwrap_err(),
        ComposeError::BadThreshold { path: 0, k: 0, n: 2 }
    );
    assert_eq!(
        compose(&list(Wrapper::Wsh, vec![keys(3, 2)])).unwrap_err(),
        ComposeError::BadThreshold { path: 0, k: 3, n: 2 }
    );
    assert_eq!(
        compose(&list(Wrapper::Wsh, vec![keys(1, 10)])).unwrap_err(),
        ComposeError::BadThreshold { path: 0, k: 1, n: 10 }
    );
}

#[test]
fn compose_refuses_a_thirty_third_slot() {
    // 3 × 9 + 6 = 33 slots.
    let paths = vec![keys(9, 9), keys(9, 9), keys(9, 9), keys(6, 6)];
    let err = compose(&list(Wrapper::Wsh, paths)).unwrap_err();
    assert_eq!(err, ComposeError::TooManySlots { got: 33, max: MAX_SLOTS });
}

#[test]
fn compose_admits_exactly_thirty_two_slots() {
    // 3 × 9 + 5 = 32 slots. Passes only once the lowering exists (Task 2).
    let paths = vec![keys(9, 9), keys(9, 9), keys(9, 9), keys(5, 5)];
    assert!(compose(&list(Wrapper::Wsh, paths)).is_ok());
}

#[test]
fn compose_refuses_legacy_wrappers_outside_the_single_sorted_multi_shape() {
    for w in [Wrapper::Sh, Wrapper::ShWsh] {
        assert_eq!(
            compose(&list(w, vec![keys(1, 1)])).unwrap_err(),
            ComposeError::LegacyWrapperShape
        );
        assert_eq!(
            compose(&list(w, vec![keys(2, 3), keys(1, 1)])).unwrap_err(),
            ComposeError::LegacyWrapperShape
        );
        assert_eq!(
            compose(&list(w, vec![with_lock(keys(2, 3), Lock::OlderBlocks(10))])).unwrap_err(),
            ComposeError::LegacyWrapperShape
        );
        assert_eq!(
            compose(&list(w, vec![unsorted(2, 3)])).unwrap_err(),
            ComposeError::LegacyWrapperShape
        );
    }
}

#[test]
fn compose_refuses_lock_operands_outside_the_consensus_bands() {
    // Each case pins the BAND NAMED in the refusal, not only that a refusal fired.
    let cases: &[(Lock, &str)] = &[
        (Lock::OlderBlocks(0), "older in blocks needs 1..=65535"),
        (Lock::OlderUnits(0), "older in 512-second units needs 1..=65535"),
        (Lock::AfterHeight(0), "after height needs 1..=499999999"),
        (Lock::AfterHeight(500_000_000), "after height needs 1..=499999999"),
        (Lock::AfterTime(499_999_999), "after time needs 500000000..=2147483647"),
        (Lock::AfterTime(2_147_483_648), "after time needs 500000000..=2147483647"),
    ];
    for (lock, why) in cases {
        let err = compose(&list(Wrapper::Wsh, vec![with_lock(keys(1, 1), *lock)])).unwrap_err();
        assert_eq!(err, ComposeError::LockOutOfRange { path: 0, why }, "{lock:?}");
    }
}

#[test]
fn lock_operand_bands_are_inclusive_at_both_ends() {
    // BIP-68 / BIP-65 / BIP-379 boundaries, straight from `Lock::operand`; no
    // lowering involved, so this passes from Task 1.
    use md_codec::tag::Tag;
    assert_eq!(Lock::OlderBlocks(1).operand(), Ok((Tag::Older, 1)));
    assert_eq!(Lock::OlderBlocks(65535).operand(), Ok((Tag::Older, 65535)));
    assert_eq!(Lock::OlderUnits(1).operand(), Ok((Tag::Older, 0x0040_0001)));
    assert_eq!(Lock::OlderUnits(65535).operand(), Ok((Tag::Older, 0x0040_ffff)));
    assert_eq!(Lock::AfterHeight(1).operand(), Ok((Tag::After, 1)));
    assert_eq!(Lock::AfterHeight(499_999_999).operand(), Ok((Tag::After, 499_999_999)));
    assert_eq!(Lock::AfterTime(500_000_000).operand(), Ok((Tag::After, 500_000_000)));
    assert_eq!(Lock::AfterTime(2_147_483_647).operand(), Ok((Tag::After, 2_147_483_647)));
    assert!(Lock::OlderUnits(0).operand().is_err(), "0x400000 alone is a lock of ZERO units, i.e. none (filed md-older-zero-time-units-not-refused)");
}
```

- [ ] **Step 2: Run the tests to verify they fail to compile**

Run: `cd /scratch/code/shibboleth/descriptor-mnemonic && cargo nextest run --locked -p md-codec --test compose_lowering 2>&1 | tail -5`
Expected: FAIL with `unresolved import md_codec::compose` (the module does not exist).

- [ ] **Step 3: Write the types, the validator and the lowering stub**

Create `crates/md-codec/src/compose/mod.rs`:

```rust
//! Fixed, search-free lowering of an ORDERED spend-path list to a BIP-388
//! wallet-policy template (`design/SPEC_wallet_policy_composer.md` §4, §5 in
//! `mnemonic-engrave`).
//!
//! WHY A LOWERING AND NOT THE COMPILER. rust-miniscript's compiler picks
//! fragments by cost and its output moves between versions and contexts
//! (measured 2026-09-01: `andor` for a two-path wsh, `pk`/`pkh` flipped by
//! cost, taproot leaves reordered). Two implementations must agree byte for
//! byte on what a policy IS, so the rules here are fixed and the compiler is
//! only a cross-check of validity and meaning (spec §5b).
//!
//! WHY THE TREE AND NOT TEXT. The Go port has no template-text parser; it
//! decodes md1. Building `Descriptor`'s tree directly is what the port mirrors;
//! text comes from `render::descriptor_to_template` for humans and vectors.
//!
//! Errors here are [`ComposeError`], not [`crate::Error`]: the codec's error is
//! a pure wire/decode taxonomy and stays one.
//!
//! Layout: this file holds the types, the structural validator and the two
//! entry points; `lowering.rs` holds the path body, the wsh chain, numbering
//! and origins; `tr.rs` the taproot spine; `presets.rs` the archetypes.

use crate::encode::Descriptor;
use crate::origin_path::{OriginPath, PathComponent, PathDeclPaths};
use crate::render::{descriptor_to_template, RenderError};
use crate::tag::Tag;

mod lowering;

/// Spec §4: at most eight spend paths.
pub const MAX_PATHS: usize = 8;
/// Spec §4b: at most nine keys in one path.
pub const MAX_KEYS_PER_PATH: u8 = 9;
/// Spec §4b: the wire's 5-bit `path_decl.n` caps a policy at 32 slots.
pub const MAX_SLOTS: u8 = 32;
/// BIP-68: bit 22 selects 512-second units.
pub const SEQUENCE_TYPE_FLAG: u32 = 1 << 22;
/// BIP-65: operands at or above this are Unix times, below are heights.
pub const LOCKTIME_THRESHOLD: u32 = 500_000_000;
/// BIP-379: miniscript admits absolute locktimes up to 2^31 - 1.
pub const MAX_ABSOLUTE_LOCKTIME: u32 = 0x7fff_ffff;

/// The script wrapper (spec §4a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wrapper {
    /// `tr(...)`; any path list.
    Tr,
    /// `wsh(...)`; any path list.
    Wsh,
    /// `sh(wsh(...))`; one unlocked, unhashed sorted multi-key path only.
    ShWsh,
    /// `sh(...)`; one unlocked, unhashed sorted multi-key path only.
    Sh,
}

impl Wrapper {
    /// BIP-48 script-type component for a seed-derived slot (spec §4f).
    pub fn script_type(self) -> u32 {
        match self {
            Wrapper::Wsh | Wrapper::Sh => 2,
            Wrapper::ShWsh => 1,
            Wrapper::Tr => 3,
        }
    }

    fn is_legacy(self) -> bool {
        matches!(self, Wrapper::Sh | Wrapper::ShWsh)
    }
}

/// One timelock, in the operator's units (spec §4c).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lock {
    /// `older(n)`, n blocks, 1..=65535.
    OlderBlocks(u16),
    /// `older(0x400000 + u)`, u units of 512 seconds, 1..=65535.
    OlderUnits(u16),
    /// `after(h)`, a block height, 1..=499,999,999.
    AfterHeight(u32),
    /// `after(t)`, a Unix time, 500,000,000..=2,147,483,647.
    AfterTime(u32),
}

impl Lock {
    /// The tag and the consensus operand this lock encodes to, or the
    /// out-of-range reason.
    pub fn operand(self) -> Result<(Tag, u32), &'static str> {
        match self {
            Lock::OlderBlocks(0) => Err("older in blocks needs 1..=65535"),
            Lock::OlderBlocks(b) => Ok((Tag::Older, u32::from(b))),
            Lock::OlderUnits(0) => Err("older in 512-second units needs 1..=65535"),
            Lock::OlderUnits(u) => Ok((Tag::Older, SEQUENCE_TYPE_FLAG + u32::from(u))),
            Lock::AfterHeight(h) if h == 0 || h >= LOCKTIME_THRESHOLD => {
                Err("after height needs 1..=499999999")
            }
            Lock::AfterHeight(h) => Ok((Tag::After, h)),
            Lock::AfterTime(t) if !(LOCKTIME_THRESHOLD..=MAX_ABSOLUTE_LOCKTIME).contains(&t) => {
                Err("after time needs 500000000..=2147483647")
            }
            Lock::AfterTime(t) => Ok((Tag::After, t)),
        }
    }
}

/// k-of-n over FRESH slots (spec §4b, C5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeySet {
    /// Threshold.
    pub k: u8,
    /// Key count; each key is a new slot.
    pub n: u8,
    /// Sorted (`sortedmulti`/`sortedmulti_a`) where the position allows it;
    /// `false` asks for `multi`/`multi_a` there, which is EXPERIMENTAL.
    pub sorted: bool,
}

/// One spend path: keys, optional hash, optional lock (spec §4b).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendPath {
    /// `None` is a keyless path: wsh-only, needs `hash`, EXPERIMENTAL.
    pub keys: Option<KeySet>,
    /// A `sha256(H)` hashlock; H is the SHA-256 of a 32-byte preimage.
    pub hash: Option<[u8; 32]>,
    /// At most one timelock.
    pub lock: Option<Lock>,
}

impl SpendPath {
    fn is_bare_multi(&self) -> bool {
        matches!(self.keys, Some(KeySet { n, .. }) if n >= 2) && self.hash.is_none() && self.lock.is_none()
    }

    fn is_bare_single(&self) -> bool {
        matches!(self.keys, Some(KeySet { n: 1, .. })) && self.hash.is_none() && self.lock.is_none()
    }
}

/// The operator's ordered list under one wrapper (spec §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathList {
    /// The script wrapper every path sits under.
    pub wrapper: Wrapper,
    /// The spend paths, in the operator's listed order.
    pub paths: Vec<SpendPath>,
}

/// A slot's declared origin and, when known, its master fingerprint (spec §4f).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotOrigin {
    /// The declared derivation origin (spec §4f).
    pub origin: OriginPath,
    /// The master fingerprint, when the seating knows it.
    pub fingerprint: Option<[u8; 4]>,
}

/// Where an emitted slot came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    /// The emitted `@index` (first appearance in the template text).
    pub index: u8,
    /// Index into `PathList::paths`.
    pub path: usize,
    /// Position within that path's key set, 0-based.
    pub ordinal: u8,
}

/// The EXPERIMENTAL conditions a list triggered (spec §4b, §5; C16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Experimental {
    /// Path `.0` has no key.
    KeylessPath(usize),
    /// Path `.0` asked for unsorted keys where sorted was legal.
    UnsortedKeys(usize),
}

/// A lowered policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composed {
    /// The template tree with every slot's origin (declared or §4f default),
    /// ready for `encode_payload` / `chunk::split` /
    /// `render::descriptor_to_template`. Keys are bound by the caller.
    pub descriptor: Descriptor,
    /// Every emitted slot, in emitted order.
    pub slots: Vec<Slot>,
    /// The path extracted as the taproot internal key, if any.
    pub internal_key_path: Option<usize>,
    /// Every EXPERIMENTAL condition the list triggered.
    pub experimental: Vec<Experimental>,
}

/// Why a list cannot be lowered (spec §4e, §4c, §4f).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposeError {
    /// The list has no paths.
    NoPaths,
    /// More than [`MAX_PATHS`] paths.
    TooManyPaths {
        /// The number of paths given.
        got: usize,
    },
    /// No path carries a key (BIP-388 l.191).
    NoKeyedPath,
    /// Path `path` has neither keys nor hash: anyone could spend after its lock.
    LockOnlyPath {
        /// 0-based index into `PathList::paths`.
        path: usize,
    },
    /// Path `path` is keyless under `tr`.
    KeylessUnderTr {
        /// 0-based index into `PathList::paths`.
        path: usize,
    },
    /// `k`/`n` outside 1 ≤ k ≤ n ≤ 9.
    BadThreshold {
        /// 0-based index into `PathList::paths`.
        path: usize,
        /// The threshold given.
        k: u8,
        /// The key count given.
        n: u8,
    },
    /// More than [`MAX_SLOTS`] slots in total.
    TooManySlots {
        /// The slot count the list would need.
        got: usize,
        /// The wire's cap.
        max: u8,
    },
    /// `sh`/`sh(wsh)` with anything but one unlocked, unhashed sorted multi-key path.
    LegacyWrapperShape,
    /// A lock operand outside spec §4c.
    LockOutOfRange {
        /// 0-based index into `PathList::paths`.
        path: usize,
        /// The band that was missed, in the words the operator sees.
        why: &'static str,
    },
    /// `compose_with` was given a declaration slice of the wrong length.
    WrongSlotCount {
        /// Declarations given.
        got: usize,
        /// Slots the policy has.
        want: usize,
    },
    /// Two slots would declare the same origin without two distinct fingerprints.
    IndistinguishableSlots {
        /// The lower emitted slot index.
        a: u8,
        /// The higher emitted slot index.
        b: u8,
    },
    /// A preset's parameters do not form the archetype it is named for
    /// (`presets`, spec §4d): e.g. decaying tiers that do not unlock later.
    PresetShape {
        /// What the archetype needs, in the words the operator sees.
        why: &'static str,
    },
}

impl core::fmt::Display for ComposeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ComposeError::NoPaths => write!(f, "a policy needs at least one spend path"),
            ComposeError::TooManyPaths { got } => {
                write!(f, "a policy holds at most {MAX_PATHS} spend paths; got {got}")
            }
            ComposeError::NoKeyedPath => {
                write!(f, "every wallet needs at least one path with a key")
            }
            ComposeError::LockOnlyPath { path } => write!(
                f,
                "path {} has only a time lock, so anyone could spend after it; add a key or a hash",
                path + 1
            ),
            ComposeError::KeylessUnderTr { path } => write!(
                f,
                "path {} has no key; this build will not put a key-less path in taproot (use wsh, or add a key)",
                path + 1
            ),
            ComposeError::BadThreshold { path, k, n } => write!(
                f,
                "path {}: {k}-of-{n} is not admitted (1 <= k <= n <= {MAX_KEYS_PER_PATH})",
                path + 1
            ),
            ComposeError::TooManySlots { got, max } => {
                write!(f, "this wallet would have {got} key slots; the wire holds at most {max}")
            }
            ComposeError::LegacyWrapperShape => write!(
                f,
                "legacy wrappers hold one plain sorted multisig only (n >= 2, no lock, no hash); use wsh or tr"
            ),
            ComposeError::LockOutOfRange { path, why } => {
                write!(f, "path {}: {why}", path + 1)
            }
            ComposeError::WrongSlotCount { got, want } => {
                write!(f, "declarations for {got} slots given, but the policy has {want}")
            }
            ComposeError::IndistinguishableSlots { a, b } => write!(
                f,
                "slots @{a} and @{b} declare the same origin without two distinct fingerprints; a template like that cannot be restored"
            ),
            ComposeError::PresetShape { why } => write!(f, "preset: {why}"),
        }
    }
}

impl std::error::Error for ComposeError {}

/// Structural validation of a list (spec §4e), before any lowering.
///
/// Returns the total slot count on success.
pub fn validate(list: &PathList) -> Result<usize, ComposeError> {
    if list.paths.is_empty() {
        return Err(ComposeError::NoPaths);
    }
    if list.paths.len() > MAX_PATHS {
        return Err(ComposeError::TooManyPaths { got: list.paths.len() });
    }
    let mut slots = 0usize;
    let mut any_keyed = false;
    for (i, p) in list.paths.iter().enumerate() {
        if let Some(ks) = p.keys {
            if ks.k == 0 || ks.n == 0 || ks.k > ks.n || ks.n > MAX_KEYS_PER_PATH {
                return Err(ComposeError::BadThreshold { path: i, k: ks.k, n: ks.n });
            }
            slots += usize::from(ks.n);
            any_keyed = true;
        } else if p.hash.is_none() {
            return Err(ComposeError::LockOnlyPath { path: i });
        } else if list.wrapper == Wrapper::Tr {
            return Err(ComposeError::KeylessUnderTr { path: i });
        }
        if let Some(lock) = p.lock {
            if let Err(why) = lock.operand() {
                return Err(ComposeError::LockOutOfRange { path: i, why });
            }
        }
    }
    if !any_keyed {
        return Err(ComposeError::NoKeyedPath);
    }
    if slots > usize::from(MAX_SLOTS) {
        return Err(ComposeError::TooManySlots { got: slots, max: MAX_SLOTS });
    }
    if list.wrapper.is_legacy() {
        let sole = list.paths.len() == 1 && list.paths[0].is_bare_multi();
        let sorted = matches!(list.paths.first().and_then(|p| p.keys), Some(KeySet { sorted: true, .. }));
        if !(sole && sorted) {
            return Err(ComposeError::LegacyWrapperShape);
        }
    }
    Ok(slots)
}

/// The §4f default origin for a slot: `m/48'/0'/account'/T'`.
pub fn default_origin(wrapper: Wrapper, account: u32) -> OriginPath {
    OriginPath {
        components: vec![
            PathComponent { hardened: true, value: 48 },
            PathComponent { hardened: true, value: 0 },
            PathComponent { hardened: true, value: account },
            PathComponent { hardened: true, value: wrapper.script_type() },
        ],
    }
}

/// Lower a list with every slot UNSEATED: each slot takes the §4f default
/// origin at the lowest account not yet declared (so slot `i` gets account
/// `i`), and no fingerprint.
pub fn compose(list: &PathList) -> Result<Composed, ComposeError> {
    let n = validate(list)?;
    let none: Vec<Option<SlotOrigin>> = vec![None; n];
    compose_with(list, &none)
}

/// Lower a list with per-slot declarations, indexed by EMITTED slot index
/// (call [`compose`] first to learn the slot map). `None` means unseated.
pub fn compose_with(
    list: &PathList,
    declared: &[Option<SlotOrigin>],
) -> Result<Composed, ComposeError> {
    let n = validate(list)?;
    if declared.len() != n {
        return Err(ComposeError::WrongSlotCount { got: declared.len(), want: n });
    }
    lowering::lower(list, declared)
}

/// The rendered template with each slot's origin written inline
/// (`@0/48'/0'/0'/2'/<0;1>/*`): the form `md encode` reads back to the same
/// card, the form `md compose` prints, and the form the vector corpus stores.
/// The plain renderer omits origins by design (descriptor-mnemonic F-219).
pub fn template_with_origins(c: &Composed) -> Result<String, RenderError> {
    let mut out = descriptor_to_template(&c.descriptor)?;
    let n = usize::from(c.descriptor.n);
    let origins: Vec<&OriginPath> = match &c.descriptor.path_decl.paths {
        PathDeclPaths::Shared(o) => vec![o; n],
        PathDeclPaths::Divergent(v) => v.iter().collect(),
    };
    for (i, o) in origins.iter().enumerate() {
        let mut rendered = String::new();
        for comp in &o.components {
            rendered.push('/');
            rendered.push_str(&comp.value.to_string());
            if comp.hardened {
                rendered.push('\'');
            }
        }
        // `@1/` never occurs inside `@10/` or `@11/`, and a slot appears once.
        out = out.replace(&format!("@{i}/"), &format!("@{i}{rendered}/"));
    }
    Ok(out)
}
```

Create `crates/md-codec/src/compose/lowering.rs` (a stub; Task 2 replaces this file in full):

```rust
//! The lowering proper. STUB until its tests exist (Task 2 replaces this file).

use super::{Composed, ComposeError, PathList, SlotOrigin};

pub(super) fn lower(list: &PathList, declared: &[Option<SlotOrigin>]) -> Result<Composed, ComposeError> {
    let _ = (list, declared);
    unimplemented!("the wsh lowering lands with its tests")
}
```

Add to `crates/md-codec/src/lib.rs`, directly after `pub mod codex32;`:

```text
pub mod compose;
```

- [ ] **Step 4: Run the refusal tests to verify they pass and the rest fail on the stub**

Run: `cargo nextest run --locked -p md-codec --test compose_lowering 2>&1 | tail -15`
Expected: the nine refusal tests and `lock_operand_bands_are_inclusive_at_both_ends` PASS (10 passed; nothing reaches the stub); `compose_admits_exactly_thirty_two_slots` FAILS with `not implemented: the wsh lowering lands with its tests`.

- [ ] **Step 5: Format, clippy, commit**

Run: `cargo fmt --all && cargo clippy --locked -p md-codec --all-targets -- -D warnings 2>&1 | tail -3`
Expected (measured at implementation, 2026-09-02): fmt clean; clippy reports TRANSIENT dead code at this task and at Task 2 — `is_bare_single` (consumed by Task 3's `tr.rs`), `Numbered::path_index` (Task 3), and the Task 1 test file's imports that Task 2's tests consume. The plan's build gate assembles Tasks 1-3 together and never sees this state. Do not add `#[allow(dead_code)]` or reorder; commit with fmt clean and record the clippy output; clippy is clean from Task 3 onward and at Task 9's workspace run.

```bash
git add crates/md-codec/src/compose/mod.rs crates/md-codec/src/compose/lowering.rs crates/md-codec/src/lib.rs crates/md-codec/tests/compose_lowering.rs
git commit -m "md-codec: compose module -- types, structural refusals, lock bands (composer S0 task 1)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 2: The path body, the wsh combinator chain, numbering and origins

**Files:**
- Replace: `crates/md-codec/src/compose/lowering.rs` (the whole file; the Task 1 stub goes away)
- Create: `crates/md-codec/src/compose/tr.rs` (a stub this task; Task 3 replaces it)
- Modify: `crates/md-codec/src/compose/mod.rs` (add `mod tr;` beside `mod lowering;`)
- Test: `crates/md-codec/tests/compose_lowering.rs`

**Interfaces:**
- Consumes: Task 1's types.
- Produces (all `pub(super)`, used by `tr.rs` in Task 3): `struct Numbered<'a> { path: &'a SpendPath, path_index: usize, slots: Vec<u8> }`; `fn number(list: &PathList, first: Option<usize>) -> (Vec<Numbered<'_>>, Vec<Slot>)`; `fn path_body(p: &Numbered<'_>, tap: bool, sorted_legal: bool) -> Node`; `fn experimental(list: &PathList, sole_sorted_legal: impl Fn(usize) -> bool) -> Vec<Experimental>`; `fn finish(list, declared, tree: Node, slots: Vec<Slot>, internal_key_path: Option<usize>, experimental: Vec<Experimental>) -> Result<Composed, ComposeError>`; `fn lower` complete for `Wsh`, `Sh`, `ShWsh` and dispatching `Tr` to `super::tr::lower_tr(list, declared)`.

- [ ] **Step 1: Write the failing wsh lowering tests**

Add to `crates/md-codec/tests/compose_lowering.rs`:

```rust
// ---- §5 wsh lowering, by rendered text -----------------------------------------

fn text(list: &PathList) -> String {
    descriptor_to_template(&compose(list).unwrap().descriptor).unwrap()
}

/// Every slot's origin, in slot order, read from `path_decl` (the rendered
/// text never carries origins: descriptor-mnemonic F-219).
fn origins(c: &Composed) -> Vec<OriginPath> {
    match &c.descriptor.path_decl.paths {
        PathDeclPaths::Shared(o) => vec![o.clone(); c.descriptor.n as usize],
        PathDeclPaths::Divergent(v) => v.clone(),
    }
}

#[test]
fn unseated_slots_take_ascending_default_accounts_under_the_wrapper_script_type() {
    let c = compose(&list(Wrapper::Wsh, vec![keys(2, 3)])).unwrap();
    assert_eq!(origins(&c), vec![hardened(&[48, 0, 0, 2]), hardened(&[48, 0, 1, 2]), hardened(&[48, 0, 2, 2])]);
    assert_eq!(
        template_with_origins(&c).unwrap(),
        "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))"
    );
    // One slot: a shared declaration, not a one-element divergent list.
    let c = compose(&list(Wrapper::Wsh, vec![keys(1, 1)])).unwrap();
    assert!(matches!(c.descriptor.path_decl.paths, PathDeclPaths::Shared(_)));
    assert_eq!(origins(&c), vec![hardened(&[48, 0, 0, 2])]);
    // Script types: sh(wsh) is 1', sh is 2', tr is 3'.
    let c = compose(&list(Wrapper::ShWsh, vec![keys(2, 2)])).unwrap();
    assert_eq!(origins(&c), vec![hardened(&[48, 0, 0, 1]), hardened(&[48, 0, 1, 1])]);
    let c = compose(&list(Wrapper::Sh, vec![keys(2, 2)])).unwrap();
    assert_eq!(origins(&c), vec![hardened(&[48, 0, 0, 2]), hardened(&[48, 0, 1, 2])]);
    // tr's 3' is asserted in Task 3, once the taproot lowering exists.
}

#[test]
fn template_with_origins_inlines_two_digit_slots_without_touching_their_prefixes() {
    // Hand-written, NOT printer-generated: `@1/` must not be rewritten inside
    // `@10/` or `@11/`. Twelve slots: a 9-of-9 head and a 3-of-3 tail.
    let c = compose(&list(Wrapper::Wsh, vec![keys(9, 9), keys(3, 3)])).unwrap();
    assert_eq!(
        template_with_origins(&c).unwrap(),
        "wsh(or_d(multi(9,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*,@3/48'/0'/3'/2'/<0;1>/*,@4/48'/0'/4'/2'/<0;1>/*,@5/48'/0'/5'/2'/<0;1>/*,@6/48'/0'/6'/2'/<0;1>/*,@7/48'/0'/7'/2'/<0;1>/*,@8/48'/0'/8'/2'/<0;1>/*),multi(3,@9/48'/0'/9'/2'/<0;1>/*,@10/48'/0'/10'/2'/<0;1>/*,@11/48'/0'/11'/2'/<0;1>/*)))"
    );
}

#[test]
fn sole_unlocked_multi_path_under_wsh_is_sortedmulti() {
    assert_eq!(
        text(&list(Wrapper::Wsh, vec![keys(2, 3)])),
        "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))"
    );
}

#[test]
fn sole_unsorted_multi_path_under_wsh_is_multi_and_experimental() {
    let c = compose(&list(Wrapper::Wsh, vec![unsorted(2, 3)])).unwrap();
    assert_eq!(
        descriptor_to_template(&c.descriptor).unwrap(),
        "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))"
    );
    assert_eq!(c.experimental, vec![Experimental::UnsortedKeys(0)]);
}

#[test]
fn single_key_under_wsh_is_pkh() {
    assert_eq!(text(&list(Wrapper::Wsh, vec![keys(1, 1)])), "wsh(pkh(@0/<0;1>/*))");
}

#[test]
fn a_locked_multi_path_is_unsorted_multi_without_the_experimental_mark() {
    // Sorted forms cannot nest inside a fragment (BIP-383/388; md refuses), so
    // the lowering forces `multi` and does NOT report it as chosen-unsorted.
    let c = compose(&list(Wrapper::Wsh, vec![with_lock(keys(2, 3), Lock::OlderBlocks(26280)), keys(1, 1)]))
        .unwrap();
    assert_eq!(
        descriptor_to_template(&c.descriptor).unwrap(),
        "wsh(or_i(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),older(26280)),pkh(@3/<0;1>/*)))"
    );
    assert!(c.experimental.is_empty());
}

#[test]
fn two_path_wsh_with_a_bare_multi_head_uses_or_d() {
    // The reference two-path wallet, wsh form (spec §5, C21/C23).
    let l = list(Wrapper::Wsh, vec![keys(2, 3), with_lock(keys(1, 1), Lock::OlderBlocks(26280))]);
    assert_eq!(
        text(&l),
        "wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pkh(@3/<0;1>/*),older(26280))))"
    );
}

#[test]
fn a_single_key_head_uses_or_i_not_or_d() {
    // I1/C21: or_d(pkh(P1), R) is dominated and publishes P1's key.
    let l = list(Wrapper::Wsh, vec![keys(1, 1), with_lock(keys(1, 1), Lock::OlderBlocks(100))]);
    assert_eq!(
        text(&l),
        "wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(100))))"
    );
}

#[test]
fn conjunct_order_is_keys_hash_lock() {
    let p = with_lock(with_hash(keys(2, 3), H1), Lock::AfterHeight(1_000_000));
    let l = list(Wrapper::Wsh, vec![p, keys(1, 1)]);
    let h = "a8".repeat(32);
    assert_eq!(
        text(&l),
        format!(
            "wsh(or_i(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:sha256({h}),after(1000000))),pkh(@3/<0;1>/*)))"
        )
    );
}

#[test]
fn a_keyless_wsh_path_is_admitted_and_marked_experimental() {
    let l = list(Wrapper::Wsh, vec![keys(2, 3), keyless(H1, Some(Lock::AfterHeight(1_383_520)))]);
    let c = compose(&l).unwrap();
    let h = "a8".repeat(32);
    assert_eq!(
        descriptor_to_template(&c.descriptor).unwrap(),
        format!(
            "wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:sha256({h}),after(1383520))))"
        )
    );
    assert_eq!(c.experimental, vec![Experimental::KeylessPath(1)]);
}

#[test]
fn eight_paths_chain_right_associatively_and_the_last_stands_alone() {
    let paths: Vec<SpendPath> = (0..8).map(|i| with_lock(keys(1, 1), Lock::OlderBlocks(100 + i))).collect();
    let t = text(&list(Wrapper::Wsh, paths));
    assert_eq!(t.matches("or_i(").count(), 7, "{t}");
    assert!(t.ends_with(",older(107))))))))))"), "{t}");
}

#[test]
fn legacy_wrappers_wrap_the_single_sorted_multi() {
    assert_eq!(
        text(&list(Wrapper::ShWsh, vec![keys(2, 3)])),
        "sh(wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*)))"
    );
    assert_eq!(
        text(&list(Wrapper::Sh, vec![keys(2, 3)])),
        "sh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))"
    );
}

#[test]
fn a_time_lock_of_one_unit_encodes_as_0x400001() {
    let c = compose(&list(Wrapper::Wsh, vec![with_lock(keys(1, 1), Lock::OlderUnits(1))])).unwrap();
    let text = descriptor_to_template(&c.descriptor).unwrap();
    assert!(text.contains("older(4194305)"), "{text}");
}

#[test]
fn slots_are_numbered_by_first_appearance_and_canonicalisation_is_identity() {
    let l = list(Wrapper::Wsh, vec![keys(2, 3), with_lock(keys(1, 1), Lock::OlderBlocks(26280))]);
    let c = compose(&l).unwrap();
    let indices: Vec<u8> = c.slots.iter().map(|s| s.index).collect();
    assert_eq!(indices, vec![0, 1, 2, 3]);
    assert_eq!(c.slots[3].path, 1);
    let mut d = c.descriptor.clone();
    canonicalize_placeholder_indices(&mut d).unwrap();
    assert_eq!(d, c.descriptor, "compose must emit canonical numbering itself");
}

#[test]
fn composed_templates_encode_and_round_trip_through_the_wire() {
    let l = list(Wrapper::Wsh, vec![keys(2, 3), with_lock(keys(1, 1), Lock::OlderBlocks(26280))]);
    let c = compose(&l).unwrap();
    let (_bytes, bits) = encode_payload(&c.descriptor).unwrap();
    assert!(bits > 0);
    let chunks = split(&c.descriptor).unwrap();
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    let back = reassemble(&refs).unwrap();
    assert_eq!(back, c.descriptor);
    if let Ok(s) = encode_md1_string(&c.descriptor) {
        assert!(s.starts_with("md1"));
    }
}

// ---- §4f declared origins and the invariant (wsh half) --------------------------

#[test]
fn compose_with_refuses_two_slots_at_one_origin_unless_both_fingerprints_differ() {
    let l = list(Wrapper::Wsh, vec![keys(2, 2)]);
    let same = hardened(&[48, 0, 0, 2]);
    // Neither fingerprinted: refused.
    let d = vec![
        Some(SlotOrigin { origin: same.clone(), fingerprint: None }),
        Some(SlotOrigin { origin: same.clone(), fingerprint: None }),
    ];
    assert_eq!(compose_with(&l, &d).unwrap_err(), ComposeError::IndistinguishableSlots { a: 0, b: 1 });
    // One fingerprinted: still refused (the one-card-fills-two-slots case).
    let d = vec![
        Some(SlotOrigin { origin: same.clone(), fingerprint: Some([9, 9, 9, 9]) }),
        Some(SlotOrigin { origin: same.clone(), fingerprint: None }),
    ];
    assert_eq!(compose_with(&l, &d).unwrap_err(), ComposeError::IndistinguishableSlots { a: 0, b: 1 });
    // Both fingerprinted and distinct: admitted, as a shared origin.
    let d = vec![
        Some(SlotOrigin { origin: same.clone(), fingerprint: Some([9, 9, 9, 9]) }),
        Some(SlotOrigin { origin: same, fingerprint: Some([8, 8, 8, 8]) }),
    ];
    assert!(compose_with(&l, &d).is_ok());
}

#[test]
fn compose_with_refuses_a_declaration_slice_of_the_wrong_length() {
    let l = list(Wrapper::Wsh, vec![keys(2, 2)]);
    assert_eq!(compose_with(&l, &[None]).unwrap_err(), ComposeError::WrongSlotCount { got: 1, want: 2 });
}
```

- [ ] **Step 2: Run the new tests to verify they fail**

Run: `cargo nextest run --locked -p md-codec --test compose_lowering 2>&1 | tail -20`
Expected: every test in this step except `compose_with_refuses_a_declaration_slice_of_the_wrong_length` (which never reaches the stub) FAILS with `not implemented: the wsh lowering lands with its tests`.

- [ ] **Step 3: Implement the lowering for the wsh family**

Replace `crates/md-codec/src/compose/lowering.rs` in full:

```rust
//! The lowering proper (spec §5): path body, wsh chain, slot numbering, §4f
//! origins and the shared `finish`. Taproot's spine lives in `tr.rs` and uses
//! the `pub(super)` pieces here.

use super::{
    default_origin, Composed, ComposeError, Experimental, KeySet, PathList, Slot, SlotOrigin,
    SpendPath, Wrapper,
};
use crate::encode::Descriptor;
use crate::origin_path::{OriginPath, PathDecl, PathDeclPaths};
use crate::tag::Tag;
use crate::tlv::TlvSection;
use crate::tree::{Body, Node};
use crate::use_site_path::UseSitePath;

/// A path with its slot indices already assigned.
pub(super) struct Numbered<'a> {
    pub(super) path: &'a SpendPath,
    pub(super) path_index: usize,
    pub(super) slots: Vec<u8>,
}

fn key_leaf(tag_single: Tag, tag_multi: Tag, tag_sorted: Tag, ks: KeySet, slots: &[u8], sorted_legal: bool) -> Node {
    if ks.n == 1 {
        return Node { tag: tag_single, body: Body::KeyArg { index: slots[0] } };
    }
    let tag = if sorted_legal && ks.sorted { tag_sorted } else { tag_multi };
    Node { tag, body: Body::MultiKeys { k: ks.k, indices: slots.to_vec() } }
}

fn verify(x: Node) -> Node {
    Node { tag: Tag::Verify, body: Body::Children(vec![x]) }
}

fn and_v(a: Node, b: Node) -> Node {
    Node { tag: Tag::AndV, body: Body::Children(vec![verify(a), b]) }
}

/// `and_v(v:KEYS, and_v(v:sha256(H), LOCK))`, dropping absent parts (spec §5).
pub(super) fn path_body(p: &Numbered<'_>, tap: bool, sorted_legal: bool) -> Node {
    let mut parts: Vec<Node> = Vec::with_capacity(3);
    if let Some(ks) = p.path.keys {
        let (single, multi, sorted) = if tap {
            (Tag::PkK, Tag::MultiA, Tag::SortedMultiA)
        } else {
            (Tag::PkH, Tag::Multi, Tag::SortedMulti)
        };
        parts.push(key_leaf(single, multi, sorted, ks, &p.slots, sorted_legal));
    }
    if let Some(h) = p.path.hash {
        parts.push(Node { tag: Tag::Sha256, body: Body::Hash256Body(h) });
    }
    if let Some(lock) = p.path.lock {
        let (tag, operand) = lock.operand().expect("validated by `validate`");
        parts.push(Node { tag, body: Body::Timelock(operand) });
    }
    let mut it = parts.into_iter().rev();
    let mut acc = it.next().expect("a path has at least one part after validation");
    for part in it {
        acc = and_v(part, acc);
    }
    acc
}

/// Listed order, recursive, last path alone: `or_d` iff the head is a bare
/// multi-key set, else `or_i` (spec §5, C21, C23).
fn wsh_chain(paths: &[Numbered<'_>]) -> Node {
    let sole = paths.len() == 1;
    let mut nodes: Vec<Node> = paths
        .iter()
        .map(|p| path_body(p, false, sole && p.path.is_bare_multi()))
        .collect();
    let mut acc = nodes.pop().expect("at least one path");
    let heads = &paths[..paths.len() - 1];
    for (p, node) in heads.iter().zip(nodes).rev() {
        let tag = if p.path.is_bare_multi() { Tag::OrD } else { Tag::OrI };
        acc = Node { tag, body: Body::Children(vec![node, acc]) };
    }
    acc
}

/// Slot numbering by first appearance in the EMITTED text (spec §5). For
/// wsh that is listed order; for tr the extracted internal key comes first.
/// The returned `Numbered` list is in LISTED order regardless.
pub(super) fn number(list: &PathList, first: Option<usize>) -> (Vec<Numbered<'_>>, Vec<Slot>) {
    let mut order: Vec<usize> = Vec::with_capacity(list.paths.len());
    if let Some(f) = first {
        order.push(f);
    }
    order.extend((0..list.paths.len()).filter(|i| Some(*i) != first));
    let mut next: u8 = 0;
    let mut slots = Vec::new();
    let mut by_path: Vec<Option<Numbered<'_>>> = (0..list.paths.len()).map(|_| None).collect();
    for pi in order {
        let p = &list.paths[pi];
        let mut mine = Vec::new();
        if let Some(ks) = p.keys {
            for ordinal in 0..ks.n {
                slots.push(Slot { index: next, path: pi, ordinal });
                mine.push(next);
                next += 1;
            }
        }
        by_path[pi] = Some(Numbered { path: p, path_index: pi, slots: mine });
    }
    let numbered: Vec<Numbered<'_>> = by_path.into_iter().flatten().collect();
    (numbered, slots)
}

/// The EXPERIMENTAL marks: every keyless path; every path that asked for
/// unsorted keys at a position where sorted was legal.
pub(super) fn experimental(list: &PathList, sole_sorted_legal: impl Fn(usize) -> bool) -> Vec<Experimental> {
    let mut out = Vec::new();
    for (i, p) in list.paths.iter().enumerate() {
        match p.keys {
            None => out.push(Experimental::KeylessPath(i)),
            Some(ks) if ks.n >= 2 && !ks.sorted && sole_sorted_legal(i) => {
                out.push(Experimental::UnsortedKeys(i))
            }
            _ => {}
        }
    }
    out
}

/// §4f: declared origins for seated slots; the lowest free default account
/// for unseated ones; the pairwise-distinguishability invariant.
#[allow(clippy::type_complexity)]
fn origins(
    list: &PathList,
    declared: &[Option<SlotOrigin>],
) -> Result<(PathDecl, Option<Vec<(u8, [u8; 4])>>), ComposeError> {
    let n = declared.len();
    let mut per_slot: Vec<Option<SlotOrigin>> = declared.to_vec();
    let mut taken: Vec<OriginPath> = per_slot.iter().flatten().map(|s| s.origin.clone()).collect();
    for slot in per_slot.iter_mut() {
        if slot.is_none() {
            let mut account: u32 = 0;
            loop {
                let candidate = default_origin(list.wrapper, account);
                if !taken.contains(&candidate) {
                    taken.push(candidate.clone());
                    *slot = Some(SlotOrigin { origin: candidate, fingerprint: None });
                    break;
                }
                account += 1;
            }
        }
    }
    let resolved: Vec<SlotOrigin> = per_slot.into_iter().map(|s| s.expect("filled above")).collect();
    for a in 0..n {
        for b in (a + 1)..n {
            if resolved[a].origin == resolved[b].origin {
                let distinct = match (resolved[a].fingerprint, resolved[b].fingerprint) {
                    (Some(x), Some(y)) => x != y,
                    _ => false,
                };
                if !distinct {
                    return Err(ComposeError::IndistinguishableSlots { a: a as u8, b: b as u8 });
                }
            }
        }
    }
    let all_same = resolved.windows(2).all(|w| w[0].origin == w[1].origin);
    let paths = if all_same {
        PathDeclPaths::Shared(resolved[0].origin.clone())
    } else {
        PathDeclPaths::Divergent(resolved.iter().map(|s| s.origin.clone()).collect())
    };
    let fps: Vec<(u8, [u8; 4])> = resolved
        .iter()
        .enumerate()
        .filter_map(|(i, s)| s.fingerprint.map(|fp| (i as u8, fp)))
        .collect();
    let fingerprints = if fps.is_empty() { None } else { Some(fps) };
    Ok((PathDecl { n: n as u8, paths }, fingerprints))
}

/// Assemble the `Descriptor` around a finished tree.
pub(super) fn finish(
    list: &PathList,
    declared: &[Option<SlotOrigin>],
    tree: Node,
    slots: Vec<Slot>,
    internal_key_path: Option<usize>,
    experimental: Vec<Experimental>,
) -> Result<Composed, ComposeError> {
    let (path_decl, fingerprints) = origins(list, declared)?;
    let mut tlv = TlvSection::new_empty();
    tlv.fingerprints = fingerprints;
    let descriptor = Descriptor {
        n: declared.len() as u8,
        path_decl,
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv,
    };
    Ok(Composed { descriptor, slots, internal_key_path, experimental })
}

pub(super) fn lower(list: &PathList, declared: &[Option<SlotOrigin>]) -> Result<Composed, ComposeError> {
    match list.wrapper {
        Wrapper::Tr => super::tr::lower_tr(list, declared),
        Wrapper::Wsh | Wrapper::Sh | Wrapper::ShWsh => {
            let (numbered, slots) = number(list, None);
            let sole = list.paths.len() == 1;
            let inner = wsh_chain(&numbered);
            let tree = match list.wrapper {
                Wrapper::Sh => Node { tag: Tag::Sh, body: Body::Children(vec![inner]) },
                Wrapper::ShWsh => Node {
                    tag: Tag::Sh,
                    body: Body::Children(vec![Node { tag: Tag::Wsh, body: Body::Children(vec![inner]) }]),
                },
                _ => Node { tag: Tag::Wsh, body: Body::Children(vec![inner]) },
            };
            let exp = experimental(list, |i| sole && list.paths[i].is_bare_multi());
            finish(list, declared, tree, slots, None, exp)
        }
    }
}
```

Create `crates/md-codec/src/compose/tr.rs` (a stub; Task 3 replaces this file in full):

```rust
//! Taproot lowering. STUB until its tests exist (Task 3 replaces this file).

use super::{Composed, ComposeError, PathList, SlotOrigin};

pub(super) fn lower_tr(list: &PathList, declared: &[Option<SlotOrigin>]) -> Result<Composed, ComposeError> {
    let _ = (list, declared);
    unimplemented!("the taproot lowering lands with its tests")
}
```

Add to `crates/md-codec/src/compose/mod.rs`, directly after `mod lowering;`:

```rust
mod tr;
```

- [ ] **Step 4: Run the tests**

Run: `cargo nextest run --locked -p md-codec --test compose_lowering 2>&1 | tail -20`
Expected: every test PASSES (no taproot test exists yet). If a rendered string differs from an expectation ONLY in a spelling the renderer owns (e.g. how divergent origins are inlined), the renderer is the authority: fix the test string, note it in the commit, and carry the corrected spelling into the spec's §5 vectors.

- [ ] **Step 5: Format, clippy, commit**

Run: `cargo fmt --all && cargo clippy --locked -p md-codec --all-targets -- -D warnings 2>&1 | tail -3`
Expected: fmt clean; clippy still reports the Task 1 transient dead code (`is_bare_single`, `path_index`) until Task 3 lands — same handling as Task 1.

```bash
git add crates/md-codec/src/compose/mod.rs crates/md-codec/src/compose/lowering.rs crates/md-codec/src/compose/tr.rs crates/md-codec/tests/compose_lowering.rs
git commit -m "md-codec: compose -- path body, wsh or_d/or_i chain, legacy wrap, numbering, default origins + invariant (composer S0 task 2)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 3: Taproot: internal-key extraction, the right spine, NUMS

**Files:**
- Replace: `crates/md-codec/src/compose/tr.rs` (the whole file; the Task 2 stub goes away)
- Test: `crates/md-codec/tests/compose_lowering.rs`

**Interfaces:**
- Consumes: Task 2's `pub(super)` `Numbered`, `number`, `path_body`, `experimental`, `finish`.
- Produces: `lower_tr(list: &PathList, declared: &[Option<SlotOrigin>]) -> Result<Composed, ComposeError>`; with it `compose` is complete for every wrapper.

- [ ] **Step 1: Write the failing taproot tests**

Add to `crates/md-codec/tests/compose_lowering.rs`:

```rust
// ---- §5 taproot lowering ---------------------------------------------------------

const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

#[test]
fn two_path_taproot_with_no_single_key_uses_nums_and_two_leaves() {
    // The reference two-path wallet, tr form (brainstorm §3.4), all slots unseated.
    // With two leaves the unlocked multi is NOT sole, so it is multi_a.
    let l = list(Wrapper::Tr, vec![keys(2, 3), with_lock(keys(1, 1), Lock::OlderBlocks(26280))]);
    let c = compose(&l).unwrap();
    assert_eq!(
        descriptor_to_template(&c.descriptor).unwrap(),
        format!("tr({NUMS},{{multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(26280))}})")
    );
    assert_eq!(c.internal_key_path, None);
}

#[test]
fn the_unlocked_single_key_becomes_the_internal_key_and_slot_zero() {
    // Path 1: 2-of-2 locked; path 2: single unlocked key; path 3: single locked key.
    let l = list(
        Wrapper::Tr,
        vec![
            with_lock(keys(2, 2), Lock::OlderBlocks(100)),
            keys(1, 1),
            with_lock(keys(1, 1), Lock::AfterHeight(900_000)),
        ],
    );
    let c = compose(&l).unwrap();
    assert_eq!(c.internal_key_path, Some(1));
    assert_eq!(c.slots[0].path, 1, "the extracted key is @0");
    assert_eq!(
        descriptor_to_template(&c.descriptor).unwrap(),
        "tr(@0/<0;1>/*,{and_v(v:multi_a(2,@1/<0;1>/*,@2/<0;1>/*),older(100)),and_v(v:pk(@3/<0;1>/*),after(900000))})"
    );
    let mut d = c.descriptor.clone();
    canonicalize_placeholder_indices(&mut d).unwrap();
    assert_eq!(d, c.descriptor);
}

#[test]
fn a_single_remaining_leaf_is_written_bare() {
    let l = list(Wrapper::Tr, vec![keys(1, 1), with_lock(keys(1, 1), Lock::OlderBlocks(65535))]);
    assert_eq!(
        text(&l),
        "tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),older(65535)))"
    );
}

#[test]
fn a_lone_single_key_is_a_key_path_only_tr() {
    assert_eq!(text(&list(Wrapper::Tr, vec![keys(1, 1)])), "tr(@0/<0;1>/*)");
}

#[test]
fn a_sole_unlocked_multi_leaf_is_sortedmulti_a() {
    let l = list(Wrapper::Tr, vec![keys(2, 3)]);
    assert_eq!(
        text(&l),
        format!("tr({NUMS},sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))")
    );
}

#[test]
fn four_leaves_form_a_right_spine() {
    let paths: Vec<SpendPath> = (0..4).map(|i| with_lock(keys(1, 1), Lock::OlderBlocks(10 + i))).collect();
    let t = text(&list(Wrapper::Tr, paths));
    // {P1,{P2,{P3,P4}}}: three opening braces, and the deepest pair is P3,P4.
    assert_eq!(t.matches('{').count(), 3, "{t}");
    assert!(t.contains("older(12)),and_v(v:pk(@3/<0;1>/*),older(13))}}})"), "{t}");
}

#[test]
fn only_the_first_listed_unlocked_single_key_is_extracted() {
    // Two unlocked single keys: the first is the internal key, the second stays a leaf.
    let l = list(Wrapper::Tr, vec![keys(2, 2), keys(1, 1), keys(1, 1)]);
    let c = compose(&l).unwrap();
    assert_eq!(c.internal_key_path, Some(1));
    assert_eq!(
        descriptor_to_template(&c.descriptor).unwrap(),
        "tr(@0/<0;1>/*,{multi_a(2,@1/<0;1>/*,@2/<0;1>/*),pk(@3/<0;1>/*)})"
    );
}

#[test]
fn taproot_templates_round_trip_through_the_wire() {
    let l = list(Wrapper::Tr, vec![keys(2, 3), with_lock(keys(1, 1), Lock::OlderBlocks(26280))]);
    let c = compose(&l).unwrap();
    let chunks = split(&c.descriptor).unwrap();
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    assert_eq!(reassemble(&refs).unwrap(), c.descriptor);
}

#[test]
fn tr_default_origins_use_script_type_three() {
    let c = compose(&list(Wrapper::Tr, vec![keys(2, 2)])).unwrap();
    assert_eq!(origins(&c), vec![hardened(&[48, 0, 0, 3]), hardened(&[48, 0, 1, 3])]);
}

#[test]
fn compose_with_uses_declared_origins_and_fills_unseated_slots_with_the_lowest_free_account() {
    let l = list(Wrapper::Tr, vec![keys(2, 2), with_lock(keys(1, 1), Lock::OlderBlocks(100))]);
    let fp_a = [0x73, 0xc5, 0xda, 0x0a];
    // Slot @0 seated at account 1, slot @2 seated at account 0; slot @1 unseated.
    let declared = vec![
        Some(SlotOrigin { origin: hardened(&[48, 0, 1, 3]), fingerprint: Some(fp_a) }),
        None,
        Some(SlotOrigin { origin: hardened(&[48, 0, 0, 3]), fingerprint: Some([1, 2, 3, 4]) }),
    ];
    let c = compose_with(&l, &declared).unwrap();
    // Accounts 0 and 1 are taken, so the unseated slot @1 gets account 2.
    assert_eq!(origins(&c), vec![hardened(&[48, 0, 1, 3]), hardened(&[48, 0, 2, 3]), hardened(&[48, 0, 0, 3])]);
    assert_eq!(c.descriptor.tlv.fingerprints, Some(vec![(0, fp_a), (2, [1, 2, 3, 4])]));
    // No path is an unlocked single key (the 1-of-1 is locked), so NUMS and two leaves.
    assert_eq!(
        template_with_origins(&c).unwrap(),
        format!("tr({NUMS},{{multi_a(2,@0/48'/0'/1'/3'/<0;1>/*,@1/48'/0'/2'/3'/<0;1>/*),and_v(v:pk(@2/48'/0'/0'/3'/<0;1>/*),older(100))}})")
    );
}
```

- [ ] **Step 2: Run the new tests to verify they fail**

Run: `cargo nextest run --locked -p md-codec --test compose_lowering 2>&1 | tail -20`
Expected: the taproot tests FAIL with `not implemented: the taproot lowering lands with its tests`; every wsh test still PASSES.

- [ ] **Step 3: Implement the taproot lowering**

Replace `crates/md-codec/src/compose/tr.rs` in full:

```rust
//! Taproot lowering (spec §5, tr rows; C17, C18, M1): internal-key extraction,
//! the right spine in listed order, NUMS when no key is extracted.

use super::lowering::{experimental, finish, number, path_body, Numbered};
use super::{Composed, ComposeError, PathList, SlotOrigin, SpendPath};
use crate::tag::Tag;
use crate::tree::{Body, Node};

/// The first-listed unlocked, unhashed one-key path, if any (spec §5, M1).
fn internal_key_path(list: &PathList) -> Option<usize> {
    list.paths.iter().position(SpendPath::is_bare_single)
}

/// Right spine in listed order: `{P1,{P2,{P3,P4}}}`; one leaf is bare; no leaf
/// is no tree.
fn spine(mut leaves: Vec<Node>) -> Option<Box<Node>> {
    let mut acc = leaves.pop()?;
    for leaf in leaves.into_iter().rev() {
        acc = Node { tag: Tag::TapTree, body: Body::Children(vec![leaf, acc]) };
    }
    Some(Box::new(acc))
}

pub(super) fn lower_tr(list: &PathList, declared: &[Option<SlotOrigin>]) -> Result<Composed, ComposeError> {
    let ik = internal_key_path(list);
    let (numbered, slots) = number(list, ik);
    let leaf_paths: Vec<&Numbered<'_>> = numbered.iter().filter(|n| Some(n.path_index) != ik).collect();
    let m = leaf_paths.len();
    let leaves: Vec<Node> = leaf_paths
        .iter()
        .map(|n| path_body(n, true, m == 1 && n.path.is_bare_multi()))
        .collect();
    let tree = Node {
        tag: Tag::Tr,
        body: Body::Tr { is_nums: ik.is_none(), key_index: 0, tree: spine(leaves) },
    };
    let exp = experimental(list, |i| m == 1 && Some(i) != ik && list.paths[i].is_bare_multi());
    finish(list, declared, tree, slots, ik, exp)
}
```

- [ ] **Step 4: Run the whole compose test file**

Run: `cargo nextest run --locked -p md-codec --test compose_lowering 2>&1 | tail -30`
Expected: all tests PASS. A string mismatch HERE comes from the tree (spine shape, `multi_a` vs `sortedmulti_a`, which key was extracted, numbering), which the spec fixes and the Go port copies; the renderer-authority rule of Task 2 applies only to a spelling the renderer owns, never to a tree difference. Fix the lowering, not the expectation.

- [ ] **Step 5: Format, clippy, commit**

Run: `cargo fmt --all && cargo clippy --locked -p md-codec --all-targets -- -D warnings 2>&1 | tail -3`
Expected: clean.

```bash
git add crates/md-codec/src/compose/tr.rs crates/md-codec/tests/compose_lowering.rs
git commit -m "md-codec: compose -- taproot internal-key extraction, right spine, NUMS, declared origins (composer S0 task 3)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 4: The keyed cross-check — sanity, lift equality, addresses

**Files:**
- Modify: `crates/md-codec/Cargo.toml` (under `[dev-dependencies]` add `miniscript = { workspace = true, features = ["compiler"] }`)
- Create: `crates/md-codec/tests/compose_support.rs` (helpers; included by `#[path]` from the other compose test files, and compiled as an empty test binary of its own)
- Test: `crates/md-codec/tests/compose_crosscheck.rs`

**Interfaces:**
- Consumes: `compose`, `compose_with`; `md_codec::to_miniscript::to_miniscript_descriptor(&Descriptor, chain: u32)`; `miniscript::policy::Concrete::{compile, compile_tr}`, `miniscript::Descriptor::{new_wsh, new_sh_wsh, new_sh, lift, iter_pk, derive_at_index}`, `Miniscript::from_str_ext` with `ExtParams::new().top_unsafe()`; `bitcoin::bip32::Xpub::derive_pub`.
- Produces (in `compose_support.rs`): `XPUB`, `FP`, `NUMS`, `hardened()`, `slot_xpubs(n) -> Vec<Xpub>`, `xpub_bytes(&Xpub) -> [u8; 65]`, `keyed(&PathList) -> Descriptor`, `concrete_policy(&PathList, &Composed, &[String]) -> String`; (in `compose_crosscheck.rs`) `cross_check(name, &PathList, keyless_wsh: bool) -> String` — the whole §5b contract for one list, which Task 5 runs over the family and Task 7 over every preset.

- [ ] **Step 1: Write the failing cross-check tests**

Create `crates/md-codec/tests/compose_support.rs`:

```rust
//! Helpers shared by the compose integration tests. Included with
//! `#[path = "compose_support.rs"] mod support;` from `compose_crosscheck.rs`
//! and `compose_vectors.rs`; cargo also compiles this file as a test binary of
//! its own (with no tests), hence the allows: the workspace lints `pub` items
//! for docs, and an unused helper in one includer is dead code in that binary.
#![allow(dead_code, missing_docs)]

use std::str::FromStr;

use md_codec::compose::{compose, compose_with, Composed, PathList, SlotOrigin};
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
        components: values.iter().map(|v| PathComponent { hardened: true, value: *v }).collect(),
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
                base.derive_pub(&secp, &[child]).expect("unhardened derivation")
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
    c.descriptor.tlv.pubkeys = Some(xs.iter().enumerate().map(|(i, x)| (i as u8, xpub_bytes(x))).collect());
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
            parts.push(if ks.n == 1 { pks[0].clone() } else { format!("thresh({},{})", ks.k, pks.join(",")) });
        }
        if let Some(h) = p.hash {
            parts.push(format!("sha256({})", hex(&h)));
        }
        if let Some(lock) = p.lock {
            let (tag, v) = lock.operand().expect("validated");
            let name = if matches!(tag, Tag::Older) { "older" } else { "after" };
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
```

Create `crates/md-codec/tests/compose_crosscheck.rs`:

```rust
//! Spec §5b: every composed template converts to a rust-miniscript descriptor,
//! passes `sanity_check` (a keyless-wsh path fails it with exactly the
//! signature rule, and parses under `top_unsafe`), derives an address, and —
//! where every path has a key — lifts to the same semantic policy as the
//! COMPILER's output for the same spend conditions. `cross_check` is the whole
//! contract for one list; Task 5 runs it over the family, Task 7 over presets.

use std::str::FromStr;

#[path = "compose_support.rs"]
mod support;
use support::*;

use md_codec::compose::{compose, KeySet, Lock, PathList, SpendPath, Wrapper};
use md_codec::render::descriptor_to_template;
use md_codec::to_miniscript::to_miniscript_descriptor;
use miniscript::bitcoin::Network;
use miniscript::descriptor::DescriptorPublicKey;
use miniscript::policy::{Concrete, Liftable};
use miniscript::{Descriptor, ExtParams, Legacy, Miniscript, Segwitv0};

fn keys(k: u8, n: u8) -> SpendPath {
    SpendPath { keys: Some(KeySet { k, n, sorted: true }), hash: None, lock: None }
}

fn locked(k: u8, n: u8, lock: Lock) -> SpendPath {
    SpendPath { keys: Some(KeySet { k, n, sorted: true }), hash: None, lock: Some(lock) }
}

fn two_path(wrapper: Wrapper) -> PathList {
    PathList { wrapper, paths: vec![keys(2, 3), locked(1, 1, Lock::OlderBlocks(26280))] }
}

/// The §5b legs for one list. Returns the index-0 address (empty for a
/// keyless-wsh list, whose leg is the documented sanity FAILURE).
pub fn cross_check(name: &str, list: &PathList, keyless_wsh: bool) -> String {
    let d = keyed(list);
    let c = compose(list).unwrap_or_else(|e| panic!("{name}: {e}"));
    let conv = to_miniscript_descriptor(&d, 0).unwrap_or_else(|e| panic!("{name}: convert: {e}"));
    if keyless_wsh {
        let e = conv.sanity_check().expect_err("a signature-free path must fail the default sanity check");
        assert!(e.to_string().contains("require a signature"), "{name}: {e}");
        return String::new();
    }
    conv.sanity_check().unwrap_or_else(|e| panic!("{name}: sanity: {e}"));
    let addr = conv
        .derive_at_index(0)
        .unwrap_or_else(|e| panic!("{name}: derive: {e}"))
        .address(Network::Bitcoin)
        .unwrap_or_else(|e| panic!("{name}: address: {e}"))
        .to_string();
    // The converter's own key strings, in traversal (= emitted) order, minus
    // the NUMS internal key; the compiler gets the SAME key values, so the
    // lifted policies differ only if the spend conditions do.
    let key_strings: Vec<String> = conv.iter_pk().map(|k| k.to_string()).filter(|k| !k.starts_with(NUMS)).collect();
    assert_eq!(key_strings.len(), c.slots.len(), "{name}: one key per slot");
    let policy = concrete_policy(list, &c, &key_strings);
    let concrete = Concrete::<DescriptorPublicKey>::from_str(&policy).unwrap_or_else(|e| panic!("{name}: {policy}: {e}"));
    let theirs: Descriptor<DescriptorPublicKey> = match list.wrapper {
        Wrapper::Wsh => Descriptor::new_wsh(concrete.compile::<Segwitv0>().unwrap()).unwrap(),
        Wrapper::ShWsh => Descriptor::new_sh_wsh(concrete.compile::<Segwitv0>().unwrap()).unwrap(),
        Wrapper::Sh => Descriptor::new_sh(concrete.compile::<Legacy>().unwrap()).unwrap(),
        Wrapper::Tr => {
            // Same internal-key decision as the lowering: NUMS when no path is
            // an unlocked single key, else let the compiler extract one (the
            // lifted OR is the same set whichever key sits on the key path).
            let nums = c.internal_key_path.is_none().then(|| DescriptorPublicKey::from_str(NUMS).unwrap());
            concrete.compile_tr(nums).unwrap_or_else(|e| panic!("{name}: compile_tr: {e}"))
        }
    };
    let ours = conv.lift().unwrap().normalized().sorted();
    let theirs = theirs.lift().unwrap().normalized().sorted();
    assert_eq!(ours, theirs, "{name}: same spend conditions, whatever the fragments");
    addr
}

#[test]
fn the_reference_two_path_wallets_pass_the_cross_check() {
    let wsh = cross_check("two_path_wsh", &two_path(Wrapper::Wsh), false);
    assert!(wsh.starts_with("bc1q"), "{wsh}");
    let tr = cross_check("two_path_tr", &two_path(Wrapper::Tr), false);
    assert!(tr.starts_with("bc1p"), "{tr}");
}

#[test]
fn the_cross_check_notices_a_wrong_lowering() {
    // Mutation in the TEST, not the code: hand the compiler a DIFFERENT policy
    // (threshold 1 instead of 2) and the lift equality must fail — a check that
    // can fail is the only kind worth running over the family.
    let list = two_path(Wrapper::Wsh);
    let d = keyed(&list);
    let c = compose(&list).unwrap();
    let conv = to_miniscript_descriptor(&d, 0).unwrap();
    let key_strings: Vec<String> = conv.iter_pk().map(|k| k.to_string()).collect();
    let wrong = concrete_policy(&list, &c, &key_strings).replacen("thresh(2,", "thresh(1,", 1);
    let concrete = Concrete::<DescriptorPublicKey>::from_str(&wrong).unwrap();
    let theirs = Descriptor::new_wsh(concrete.compile::<Segwitv0>().unwrap()).unwrap().lift().unwrap().normalized().sorted();
    let ours = conv.lift().unwrap().normalized().sorted();
    assert_ne!(ours, theirs, "the lift comparison must be able to fail");
}

#[test]
fn a_keyless_wsh_path_is_admitted_with_top_unsafe_and_refused_by_the_default_sanity() {
    let list = PathList {
        wrapper: Wrapper::Wsh,
        paths: vec![keys(2, 3), SpendPath { keys: None, hash: Some(H), lock: Some(Lock::AfterHeight(1_383_520)) }],
    };
    assert_eq!(cross_check("keyless_wsh", &list, true), "");
    // The inner miniscript the device would emit, parsed two ways.
    let d = keyed(&list);
    let text = descriptor_to_template(&d).unwrap();
    // Strip exactly ONE `wsh(` and ONE `)`: `trim_end_matches(')')` would eat
    // every closing paren of the inner script.
    let mut inner = text
        .strip_prefix("wsh(")
        .and_then(|t| t.strip_suffix(')'))
        .expect("a wsh template")
        .to_string();
    for (i, xpub) in XPUB.iter().enumerate().take(3) {
        inner = inner.replace(&format!("@{i}/<0;1>/*"), &format!("[73c5da0a/48'/0'/{i}'/2']{xpub}/<0;1>/*"));
    }
    let sane = Miniscript::<DescriptorPublicKey, Segwitv0>::from_str(&inner);
    assert!(sane.is_err(), "the default parse must refuse a sigless spend path");
    let insane = Miniscript::<DescriptorPublicKey, Segwitv0>::from_str_ext(&inner, &ExtParams::new().top_unsafe())
        .expect("top_unsafe admits the keyless path and nothing else");
    assert!(insane.lift().is_ok());
}
```

- [ ] **Step 2: Add the dev-dependency and run the tests to verify what fails**

Add to `crates/md-codec/Cargo.toml` under `[dev-dependencies]`:

```text
miniscript = { workspace = true, features = ["compiler"] }
```

Run: `cargo nextest run --locked -p md-codec --test compose_crosscheck 2>&1 | tail -20`
Expected: the file compiles; every test PASSES (the lowering is complete after Task 3). The feature adds no new crate, so `Cargo.lock` does not change and `--locked` accepts it; if it did change, that is a finding to report, not a lock to commit.

- [ ] **Step 3: Format, clippy, commit**

Run: `cargo fmt --all && cargo clippy --locked -p md-codec --all-targets --all-features -- -D warnings 2>&1 | tail -3`
Expected: clean.

```bash
git add crates/md-codec/Cargo.toml crates/md-codec/tests/compose_support.rs crates/md-codec/tests/compose_crosscheck.rs
git commit -m "md-codec: compose cross-check -- reusable 5b check (sanity, address, lift equality vs the compiler), top_unsafe for keyless wsh (composer S0 task 4)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 5: The compose vector family and its tag coverage

**Files:**
- Modify: `crates/md-codec/src/test_vectors.rs` (add `compose_*` entries to `MANIFEST`), `crates/md-codec/tests/compose_support.rs` (the tagged `family()` and `SINGULAR_TAGS`), `crates/md-codec/tests/compose_crosscheck.rs` (the family-wide §5b test)
- Test: `crates/md-codec/tests/compose_vectors.rs`

**Interfaces:**
- Consumes: `md_codec::test_vectors::{MANIFEST, Vector}`; the `md vectors` exporter (`crates/md-cli/src/cmd/vectors.rs:16`) which writes `.conformance.json` for every KEYED vector; the fork's `TestKeyedConformanceAgreesWithRust` globs `keyed_*.conformance.json`, so every keyed compose vector is named `keyed_compose_*`.
- Produces: `family() -> Vec<(&'static str, PathList, String, Vec<&'static str>)>` and `SINGULAR_TAGS` in `compose_support.rs`; the vector names the Go builder (Stage 2) reproduces byte for byte; the tag table §12 item 1 requires; and the §5b cross-check run over every family member.

- [ ] **Step 1: Write the failing tag-coverage test**

Add to `crates/md-codec/tests/compose_support.rs` (the family lives here so both `compose_vectors.rs` and `compose_crosscheck.rs` can iterate it):

```rust
use md_codec::compose::{KeySet, Lock, SpendPath, Wrapper};

pub fn k(k: u8, n: u8) -> SpendPath {
    SpendPath { keys: Some(KeySet { k, n, sorted: true }), hash: None, lock: None }
}
pub fn u(k: u8, n: u8) -> SpendPath {
    SpendPath { keys: Some(KeySet { k, n, sorted: false }), hash: None, lock: None }
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
    SpendPath { keys: None, hash: Some(h), lock: l }
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
    ]
}

/// Tags with exactly ONE legal shape, exempt from the two-vector rule and said
/// so here: a taptree with m = 0 leaves is one unlocked single key and nothing
/// else (spec §12 item 1).
pub const SINGULAR_TAGS: &[&str] = &["spine:0"];
```

Create `crates/md-codec/tests/compose_vectors.rs`:

```rust
//! Spec §12 item 1: TAGGED coverage of the lowering. Every compose vector in
//! `MANIFEST` is listed in `support::family()` with the spec rows it exercises;
//! every non-singular tag must appear in at least two vectors; every listed
//! vector's stored template must be exactly what `compose` renders (with
//! origins inlined) for its path list.

use std::collections::BTreeMap;

#[path = "compose_support.rs"]
mod support;
use support::*;

use md_codec::compose::{compose, template_with_origins};
use md_codec::render::descriptor_to_template;
use md_codec::test_vectors::MANIFEST;

#[test]
fn every_family_entry_renders_as_listed() {
    // Gate-runnable without the MANIFEST: the lowering against the listed text.
    for (name, list, expected, _) in &family() {
        let c = compose(list).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(&descriptor_to_template(&c.descriptor).unwrap(), expected, "{name}");
    }
}

#[test]
fn every_compose_vector_in_the_manifest_is_exactly_what_compose_renders() {
    // MANIFEST templates carry inline origins (the parse-input form), so the
    // comparison is against `template_with_origins`.
    for (name, list, _, tags) in &family() {
        if tags.contains(&"no-corpus") {
            assert!(MANIFEST.iter().all(|v| v.name != *name), "{name}: a no-corpus vector must not be in MANIFEST (the exporter would refuse it)");
            continue;
        }
        let v = MANIFEST.iter().find(|v| v.name == *name).unwrap_or_else(|| panic!("MANIFEST lacks {name}"));
        let c = compose(list).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(template_with_origins(&c).unwrap(), v.template, "{name}");
        assert_eq!(v.path, None, "{name}: compose vectors carry inline origins, never a --path override");
    }
}

#[test]
fn every_compose_manifest_entry_is_in_the_family() {
    let fam = family();
    for v in MANIFEST.iter().filter(|v| v.name.contains("compose_")) {
        assert!(fam.iter().any(|(n, _, _, _)| *n == v.name), "untagged compose vector {}", v.name);
    }
}

#[test]
fn every_tag_appears_in_at_least_two_vectors() {
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, _, _, tags) in family() {
        for t in tags {
            *count.entry(t).or_default() += 1;
        }
    }
    let thin: Vec<_> = count.iter().filter(|(t, c)| **c < 2 && !SINGULAR_TAGS.contains(t)).collect();
    assert!(thin.is_empty(), "tags with fewer than two vectors: {thin:?}");
    for t in SINGULAR_TAGS {
        assert_eq!(count.get(t), Some(&1), "a singular tag has exactly one vector: {t}");
    }
    // The spec's required tag list, every member present (spec §12 item 1).
    for required in [
        "w:tr", "w:wsh", "w:sh-wsh", "w:sh", "paths:1", "paths:2", "paths:3", "paths:4", "slots:32",
        "spine:0", "spine:1", "spine:2", "spine:3", "spine:7", "ik:extracted-first", "ik:extracted-later",
        "ik:nums", "lock:none", "lock:blocks", "lock:units", "lock:height", "lock:time", "hash", "sorted",
        "unsorted", "keyless-wsh", "fp:distinct", "fp:one-seed-one-path", "fp:one-seed-two-paths",
        "origins:default-tr", "origins:default-wsh", "origins:default-sh-wsh", "origins:default-sh",
    ] {
        assert!(count.contains_key(required), "required tag missing from the family: {required}");
    }
}

#[test]
fn keyed_compose_vectors_bind_at_most_the_four_journey_keys() {
    for v in MANIFEST.iter().filter(|v| v.name.starts_with("keyed_compose_")) {
        assert!(!v.keys.is_empty(), "{}: a keyed_ vector must bind keys so md vectors emits .conformance.json", v.name);
        assert!(v.keys.len() <= XPUB.len(), "{}: the journey fixture has four keys", v.name);
        assert_eq!(v.keys.len(), v.fingerprints.len(), "{}", v.name);
    }
}
```

Add to `crates/md-codec/tests/compose_crosscheck.rs` (spec §12 item 1: "the §5b cross-check holds" for EVERY vector, not for three hand-picked shapes):

```rust
#[test]
fn every_family_entry_passes_the_5b_cross_check() {
    for (name, list, _, tags) in family() {
        let keyless = tags.contains(&"keyless-wsh");
        let addr = cross_check(name, &list, keyless);
        if !keyless {
            assert!(addr.starts_with("bc1") || addr.starts_with('3'), "{name}: {addr}");
        }
    }
}
```

- [ ] **Step 2: Run it to see which vectors the MANIFEST lacks**

Run: `cargo nextest run --locked -p md-codec --test compose_vectors --test compose_crosscheck 2>&1 | tail -20`
Expected: `every_family_entry_renders_as_listed`, `every_tag_appears_in_at_least_two_vectors` and `every_family_entry_passes_the_5b_cross_check` PASS (the listed texts were generated by the lowering and checked by the plan's build gate); `every_compose_vector_in_the_manifest_is_exactly_what_compose_renders` FAILS with `MANIFEST lacks keyed_compose_wsh_sole_sortedmulti`. Two tests pass VACUOUSLY at this point because `MANIFEST` holds no compose entry yet: `every_compose_manifest_entry_is_in_the_family` and `keyed_compose_vectors_bind_at_most_the_four_journey_keys`; both become real once the entries are pasted below.

- [ ] **Step 3: Generate the manifest entries from the lowering, never by hand**

The MANIFEST form is `template_with_origins` (inline origins, the parse-input form the keyed corpus already uses); it is printed by a helper test, never typed. Add to `crates/md-codec/tests/compose_vectors.rs`, at the end (the printer; kept, `--no-capture` shows it):

```rust
#[test]
fn print_family_templates_for_the_manifest() {
    for (name, list, _, tags) in family() {
        if tags.contains(&"no-corpus") {
            continue;
        }
        let c = compose(&list).unwrap();
        println!("{name}\t{}", template_with_origins(&c).unwrap());
    }
}
```

Run: `cargo nextest run --locked -p md-codec --test compose_vectors print_family --no-capture 2>&1 | grep -E 'compose_'`
Expected: twenty-six `name<TAB>template` lines (the two `no-corpus` keyless-wsh vectors are not printed: they live in `family()` and the cross-check only).

Add to `crates/md-codec/src/test_vectors.rs`, inside `MANIFEST` after the last existing entry, one `Vector { .. }` per printed line, `template` pasted verbatim. For every `keyed_compose_*` entry bind the journey keys to slots in emitted order with fingerprint `[0x73, 0xc5, 0xda, 0x0a]` on each (the origins are INLINE in the template, so `path: None`), EXCEPT the two `*_distinct_fingerprints` entries, which bind the same keys with fingerprints `(0, [0x11; 4]), (1, [0x22; 4]), (2, [0x33; 4]), (3, [0x44; 4])` (a master fingerprint is a declaration the xpub cannot contradict, so any value is legal). The four `compose_*` (unkeyed) entries keep their inline origins too (the composed policies have no canonical origin for the encoder to default to) and leave `keys: &[]`, `fingerprints: &[]`; they need no keys because `compose_wsh_eight_paths`, `compose_tr_seven_leaves`, `compose_wsh_thirty_two_slots` and `compose_tr_thirty_two_slots` exceed the four journey keys. The two keyless-path vectors (`compose_wsh_keyless_hash_path`, `compose_wsh_keyless_hash_only`) are `no-corpus`: NOT pasted, because `md vectors` and the corpus tests parse every MANIFEST template under the minting disposition (`parse_template` passes `Disposition::Refuse`, `crates/md-cli/src/parse/template.rs:2618`), which after Task 8 refuses a signature-free path without `--experimental`. They stay pinned by `every_family_entry_renders_as_listed` and the §5b cross-check, and Stage 2 mirrors them from `family()`. Use `force_chunked: true` on EVERY compose entry (measured at implementation: the smallest keyed one, a single xpub, is already 131 data symbols against the codex32 regular code's 80 cap, and the unkeyed ones carry divergent inline origins that also exceed it; `test_vectors.rs`'s own note says all keyed entries are chunked for this reason). The four journey xpubs, in slot order @0..@3, are the `XPUB` constants of Task 4 (copy them; the manifest is `&'static str`).

Template example for the first entry, as the printer emits it (do not retype the others; paste them):

```text
Vector {
    name: "keyed_compose_wsh_sole_sortedmulti",
    template: "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))",
    keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2)],
    fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a])],
    force_chunked: false,
    path: None,
},
```

with `const XPUB_JOURNEY_0: &str = "xpub6DkFAXW...";` (and 1..3) declared above `MANIFEST` in the same file.

- [ ] **Step 4: Run every corpus test and the exporter**

Run: `cargo nextest run --locked -p md-codec 2>&1 | tail -8 && cargo run --locked -p md-cli --bin md -- vectors --out /tmp/compose-vectors >/dev/null && ls /tmp/compose-vectors | grep -c 'keyed_compose_.*conformance.json'` (`--bin md`: md-cli builds two binaries)
Expected: all md-codec tests PASS (including the pre-existing corpus tests that iterate `MANIFEST`: a compose vector that they reject is a defect to fix in the lowering, not in the test); the exporter writes 22 `keyed_compose_*.conformance.json` files (the keyed count in `family()`; 26 compose entries in MANIFEST, 28 in `family()`). Then `cargo nextest run --locked -p md-cli`: `template_roundtrip.rs`, `vector_corpus.rs` and `corpus_origin_consistency.rs` iterate `MANIFEST` through the real parser and encoder and must PASS, EXCEPT the corpus-drift test, which fails until Task 9 regenerates the committed corpus (measured: 126 "Only in" lines for the new compose files and ZERO "differ" lines — any "differ" line is a finding, stop). Everything else green here; the drift test green after Task 9.

- [ ] **Step 5: Format, clippy, commit**

Run: `cargo fmt --all && cargo clippy --locked -p md-codec --all-targets --all-features -- -D warnings 2>&1 | tail -3`
Expected: clean.

```bash
git add crates/md-codec/src/test_vectors.rs crates/md-codec/tests/compose_support.rs crates/md-codec/tests/compose_vectors.rs crates/md-codec/tests/compose_crosscheck.rs
git commit -m "md-codec: compose vector family -- 28 tagged vectors (26 in MANIFEST), every required tag twice, the 5b cross-check over all of them, keyed ones export conformance.json (composer S0 task 5)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 6: `md compose` — the subcommand

**Files:**
- Create: `crates/md-cli/src/cmd/compose.rs`
- Modify: `crates/md-cli/src/cmd/mod.rs` (append `pub mod compose;`), `crates/md-cli/src/error.rs` (add variant `Compose(String)` directly after `BadArg(String),` and a `Display` arm `CliError::Compose(m) => write!(f, "{m}")` directly after `BadArg`'s; `Compose` takes `main`'s generic `Err(e)` arm, which prints `md: {e}` and exits 1 — `BadArg` alone has its own arm and exits 2), `crates/md-cli/src/main.rs` (the clap variant and dispatch arm below)
- Test: `crates/md-cli/tests/cli_compose.rs`

**Interfaces:**
- Consumes: `md_codec::compose::{compose, PathList, SpendPath, KeySet, Lock, Wrapper, Experimental}`, `md_codec::render::descriptor_to_template`, `crate::output_advisory::{emit_output_class_advisory, OutputClass::Template}`, `crate::error::CliError`.
- Produces: `cmd::compose::{parse_path, parse_wrapper, run}`.

**The DSL (normative for this CLI; the device has its own pickers):**

| flag | form | example |
| --- | --- | --- |
| `--wrapper` | `tr` / `wsh` / `sh-wsh` / `sh` | `--wrapper tr` |
| `--path` (repeatable, listed order) | `<k>of<n>[,<opt>]*` or `keyless[,<opt>]*` with opts `older=<blocks>`, `older=<units>u`, `after=<height>`, `after=<seconds>t`, `sha256=<64 hex>`, `unsorted` | `--path 2of3 --path 1of1,older=26280` |
| `--experimental` | required when any path is keyless or asks for unsorted where sorted was legal | |
| `--json` | `{"schema":..,"template":..,"template_with_origins":..,"wrapper":..,"slots":[{"index":0,"path":1,"ordinal":0},..],"internal_key_path":null,"experimental":[]}` | |
| (stdout, text mode) | the `template_with_origins` form, one line | |

- [ ] **Step 1: Write the failing CLI tests**

Create `crates/md-cli/tests/cli_compose.rs`:

```rust
//! `md compose` (SPEC_wallet_policy_composer.md §10 item 1): fixed lowering
//! from a path DSL to a BIP-388 template; round-trips through `md encode` and
//! `md decode`; refuses per §4e; gates EXPERIMENTAL shapes.

use assert_cmd::Command;
use predicates::prelude::*;

fn md() -> Command {
    Command::cargo_bin("md").expect("md binary")
}

#[test]
fn compose_two_path_wsh_prints_the_fixed_template() {
    md().args(["compose", "--wrapper", "wsh", "--path", "2of3", "--path", "1of1,older=26280"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*),and_v(v:pkh(@3/48'/0'/3'/2'/<0;1>/*),older(26280))))",
        ));
}

#[test]
fn compose_output_round_trips_through_encode_and_decode() {
    let out = md()
        .args(["compose", "--wrapper", "tr", "--path", "2of3", "--path", "1of1,older=26280"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let with_origins = String::from_utf8(out.stdout).unwrap().lines().next().unwrap().to_string();
    assert!(with_origins.contains("@0/48'/0'/0'/3'/<0;1>/*"), "{with_origins}");
    // `md decode` prints the renderer's origin-less text (F-219); get that form from --json.
    let js = md()
        .args(["compose", "--json", "--wrapper", "tr", "--path", "2of3", "--path", "1of1,older=26280"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&js.stdout).unwrap();
    let template = v["template"].as_str().unwrap().to_string();
    assert_eq!(v["template_with_origins"].as_str().unwrap(), with_origins);
    let enc = md().args(["encode", &with_origins]).output().unwrap();
    assert!(enc.status.success(), "{}", String::from_utf8_lossy(&enc.stderr));
    let chunks: Vec<String> = String::from_utf8(enc.stdout)
        .unwrap()
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' '))
        .map(str::to_string)
        .collect();
    assert!(!chunks.is_empty());
    let mut dec = md();
    dec.arg("decode");
    for c in &chunks {
        dec.arg(c);
    }
    dec.assert().success().stdout(predicate::str::starts_with(template));
}

#[test]
fn compose_refuses_a_keyless_path_without_experimental_and_admits_it_with() {
    let h = "a8".repeat(32);
    md().args(["compose", "--wrapper", "wsh", "--path", "2of3", "--path", &format!("keyless,sha256={h},after=1383520")])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("--experimental"));
    md().args(["compose", "--wrapper", "wsh", "--experimental", "--path", "2of3", "--path", &format!("keyless,sha256={h},after=1383520")])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("and_v(v:sha256({h}),after(1383520))")))
        .stderr(predicate::str::contains("EXPERIMENTAL"));
}

#[test]
fn compose_refuses_structural_defects_with_the_spec_wording() {
    md().args(["compose", "--wrapper", "tr", "--path", "2of3", "--path", "keyless,sha256=00", "--experimental"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("sha256 needs 64 hex characters"));
    md().args(["compose", "--wrapper", "sh", "--path", "1of1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("legacy wrappers hold one plain sorted multisig only"));
    md().args(["compose", "--wrapper", "wsh", "--path", "1of1,older=65536"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("older in blocks needs 1..=65535"));
    md().args(["compose", "--wrapper", "wsh", "--path", "1of1,older=4194305"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("older in blocks needs 1..=65535"));
    // A Unix time typed without its suffix is refused WITH the suffix named.
    md().args(["compose", "--wrapper", "wsh", "--path", "1of1,after=1893456000"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("after=1893456000t"));
}

#[test]
fn compose_says_when_unsorted_had_no_effect() {
    md().args(["compose", "--wrapper", "wsh", "--path", "2of3,unsorted", "--path", "1of1,older=10"])
        .assert()
        .success()
        .stderr(predicate::str::contains("`unsorted` has no effect here"))
        .stderr(predicate::str::contains("EXPERIMENTAL").not());
}

#[test]
fn compose_json_names_slots_internal_key_and_experimental() {
    md().args(["compose", "--wrapper", "tr", "--json", "--path", "2of2,older=100", "--path", "1of1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"internal_key_path\": 1"))
        .stdout(predicate::str::contains("\"template_with_origins\": \"tr(@0/48'/0'/0'/3'/<0;1>/*,"))
        .stdout(predicate::str::contains("\"index\": 0"))
        .stdout(predicate::str::contains("\"experimental\": []"));
}
```

- [ ] **Step 2: Run to verify the tests fail**

Run: `cargo nextest run --locked -p md-cli --test cli_compose 2>&1 | tail -8`
Expected: FAIL, `md compose` is an unrecognized subcommand (clap exit 2 on every case; the round-trip test fails at the first assert).

- [ ] **Step 3: Write the subcommand**

Create `crates/md-cli/src/cmd/compose.rs`:

```rust
//! `md compose` -- the FIXED lowering surface (SPEC_wallet_policy_composer.md
//! §10 item 1). The opposite contract to `md compile`: no search, no cost
//! model, the same answer from every implementation, forever.
//!
//! Not to be confused with `crate::seat::compose`, which SEATS keys into an
//! existing keyless card; this module builds the card's policy from a path list.

use crate::error::CliError;
use md_codec::compose::{compose, template_with_origins, Experimental, KeySet, Lock, PathList, SpendPath, Wrapper};
use md_codec::render::descriptor_to_template;

pub fn parse_wrapper(s: &str) -> Result<Wrapper, CliError> {
    match s {
        "tr" => Ok(Wrapper::Tr),
        "wsh" => Ok(Wrapper::Wsh),
        "sh-wsh" => Ok(Wrapper::ShWsh),
        "sh" => Ok(Wrapper::Sh),
        other => Err(CliError::Compose(format!("--wrapper {other}: expected tr, wsh, sh-wsh or sh"))),
    }
}

fn parse_u32(s: &str, what: &str) -> Result<u32, CliError> {
    s.parse::<u32>().map_err(|_| CliError::Compose(format!("{what}: `{s}` is not a number in 0..=4294967295")))
}

fn parse_u16(s: &str, what: &str) -> Result<u16, CliError> {
    s.parse::<u16>().map_err(|_| CliError::Compose(format!("{what}: `{s}` is not a number in 0..=65535")))
}

/// One `--path` value: `<k>of<n>[,opt]*` or `keyless[,opt]*`.
pub fn parse_path(s: &str) -> Result<SpendPath, CliError> {
    let mut parts = s.split(',');
    let head = parts.next().unwrap_or("");
    let keys = if head == "keyless" {
        None
    } else {
        let (k, n) = head
            .split_once("of")
            .ok_or_else(|| CliError::Compose(format!("path `{s}`: expected <k>of<n> or keyless")))?;
        let k = k.parse::<u8>().map_err(|_| CliError::Compose(format!("path `{s}`: k `{k}` is not a small number")))?;
        let n = n.parse::<u8>().map_err(|_| CliError::Compose(format!("path `{s}`: n `{n}` is not a small number")))?;
        Some(KeySet { k, n, sorted: true })
    };
    let mut path = SpendPath { keys, hash: None, lock: None };
    for opt in parts {
        if opt == "unsorted" {
            match path.keys.as_mut() {
                Some(ks) => ks.sorted = false,
                None => return Err(CliError::Compose(format!("path `{s}`: `unsorted` needs keys"))),
            }
            continue;
        }
        let (name, value) = opt
            .split_once('=')
            .ok_or_else(|| CliError::Compose(format!("path `{s}`: option `{opt}` needs a value")))?;
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
                        Err(_) => return Err(CliError::Compose(format!("path `{s}`: older in blocks needs 1..=65535, got {v}"))),
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
                return Err(CliError::Compose(format!("path `{s}`: at most one lock per path")));
            }
            "sha256" => {
                if path.hash.is_some() {
                    return Err(CliError::Compose(format!("path `{s}`: at most one hash per path")));
                }
                if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
                    return Err(CliError::Compose(format!("path `{s}`: sha256 needs 64 hex characters, lowercase")));
                }
                let mut h = [0u8; 32];
                for (i, chunk) in value.as_bytes().chunks(2).enumerate() {
                    let hi = (chunk[0] as char).to_digit(16).expect("checked") as u8;
                    let lo = (chunk[1] as char).to_digit(16).expect("checked") as u8;
                    h[i] = (hi << 4) | lo;
                }
                path.hash = Some(h);
            }
            other => return Err(CliError::Compose(format!("path `{s}`: unknown option `{other}`"))),
        }
    }
    Ok(path)
}

fn describe(e: &Experimental) -> String {
    match e {
        Experimental::KeylessPath(i) => format!("path {} has no key (bearer access to whoever holds the preimage)", i + 1),
        Experimental::UnsortedKeys(i) => format!("path {} uses unsorted keys where sorted was possible (key order is part of this wallet)", i + 1),
    }
}

pub fn run(wrapper: &str, paths: &[String], experimental: bool, json: bool) -> Result<u8, CliError> {
    let wrapper = parse_wrapper(wrapper)?;
    let paths: Vec<SpendPath> = paths.iter().map(|p| parse_path(p)).collect::<Result<_, _>>()?;
    let list = PathList { wrapper, paths };
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
    // rather than accept a typed request silently.
    for (i, p) in list.paths.iter().enumerate() {
        if matches!(p.keys, Some(KeySet { n, sorted: false, .. }) if n >= 2)
            && !composed.experimental.contains(&Experimental::UnsortedKeys(i))
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
        let v = serde_json::json!({
            "schema": SCHEMA,
            "template": template,
            "template_with_origins": with_origins,
            "wrapper": wrapper_name(wrapper),
            "slots": slots,
            "internal_key_path": composed.internal_key_path,
            "experimental": exp,
        });
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        crate::output_advisory::emit_output_class_advisory(crate::output_advisory::OutputClass::Template, &mut std::io::stderr());
        return Ok(0);
    }
    let _ = json;

    // The inline-origin form: what `md encode` reads back to the same card.
    println!("{with_origins}");
    crate::output_advisory::emit_output_class_advisory(crate::output_advisory::OutputClass::Template, &mut std::io::stderr());
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
```

Append to `crates/md-cli/src/cmd/mod.rs`:

```text
pub mod compose;
```

Add to `crates/md-cli/src/error.rs`, directly after `BadArg(String),`:

```text
    /// `md compose` refusals: structural (§4e), lock band (§4c), DSL, or an
    /// EXPERIMENTAL shape without `--experimental`. The message is complete in
    /// itself; `main`'s generic arm prefixes `md: ` and exits 1 (`BadArg`,
    /// above, is the one variant with its own arm, exiting 2).
    Compose(String),
```

and, directly after the `BadArg` arm of `impl fmt::Display for CliError`:

```text
            CliError::Compose(m) => write!(f, "{m}"),
```

Add to `crates/md-cli/src/main.rs`, in the `Command` enum (`crates/md-cli/src/main.rs:96`) directly after the `Compile { .. }` variant:

```text
    /// Lower an ORDERED list of spend paths to a BIP-388 template by FIXED rules
    /// (SPEC_wallet_policy_composer.md §5). The opposite of `compile`: no search,
    /// no cost model, the same text from every implementation.
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
        #[arg(long)]
        json: bool,
    },
```

and in the dispatch `match`, directly after the `Command::Compile { .. }` arm:

```text
        Command::Compose { wrapper, paths, experimental, json } => {
            cmd::compose::run(&wrapper, &paths, experimental, json)
        }
```

- [ ] **Step 4: Run the CLI tests and the full md-cli suite**

Run: `cargo nextest run --locked -p md-cli 2>&1 | tail -8`
Expected: all PASS, including `cli_compose` and the pre-existing `gui-schema`/`gen-man` tests, which enumerate subcommands and may pin a count or a flag set; if one fails only because a NEW subcommand exists, update that test's expectation in this task and name it in the commit.

- [ ] **Step 5: Format, clippy, commit**

Run: `cargo fmt --all && cargo clippy --locked -p md-cli --all-targets --all-features -- -D warnings 2>&1 | tail -3`
Expected: clean.

```bash
git add crates/md-cli/src/cmd/compose.rs crates/md-cli/src/cmd/mod.rs crates/md-cli/src/error.rs crates/md-cli/src/main.rs crates/md-cli/tests/cli_compose.rs
git commit -m "md-cli: md compose -- fixed lowering from a path DSL, --experimental gate, --json (composer S0 task 6)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 7: The five presets as path lists

**Files:**
- Create: `crates/md-codec/src/compose/presets.rs`
- Modify: `crates/md-codec/src/compose/mod.rs` (add `pub mod presets;` beside `mod lowering;`)
- Test: `crates/md-codec/tests/compose_lowering.rs`

**Interfaces:**
- Consumes: Task 1's `validate`, `KeySet`, `Lock`, `PathList`, `SpendPath`, `Wrapper`, `ComposeError`.
- Produces: `md_codec::compose::presets::{plain_multisig, simple_timelocked_inheritance, kofn_recovery, tiered_recovery, hashlock_gated, decaying_multisig}` each returning `Result<PathList, ComposeError>` from its parameters; Stage 3's preset picker calls the Go mirror of these.

- [ ] **Step 1: Write the failing preset tests**

Add to `crates/md-codec/tests/compose_lowering.rs`:

```rust
// ---- §4d presets --------------------------------------------------------------------

use md_codec::compose::presets;

#[test]
fn presets_compose_and_carry_the_documented_shapes() {
    let p = presets::plain_multisig(Wrapper::Wsh, 2, 3).unwrap();
    assert_eq!(text(&p), text(&list(Wrapper::Wsh, vec![keys(2, 3)])));

    let p = presets::simple_timelocked_inheritance(Wrapper::Wsh, 65535).unwrap();
    assert_eq!(p.paths.len(), 2);
    assert_eq!(p.paths[1].lock, Some(Lock::OlderBlocks(65535)));

    let p = presets::kofn_recovery(Wrapper::Tr, 2, 3, 52560).unwrap();
    assert_eq!(p.paths[0].keys, Some(KeySet { k: 2, n: 3, sorted: true }));
    assert_eq!(p.paths[1].lock, Some(Lock::OlderBlocks(52560)));

    let p = presets::tiered_recovery(Wrapper::Wsh, 2, 2, 2, 3, 4032).unwrap();
    assert_eq!(p.paths.len(), 2);

    let p = presets::hashlock_gated(Wrapper::Wsh, H1, 144).unwrap();
    assert!(p.paths[0].hash.is_some());
    assert_eq!(p.paths[1].lock, Some(Lock::OlderBlocks(144)));

    let p = presets::decaying_multisig(Wrapper::Wsh, 2, 3, 1, 2, 1000, 2000, 4_000_000).unwrap();
    assert_eq!(p.paths.len(), 3);
    assert_eq!(p.paths[1].keys, Some(KeySet { k: 1, n: 2, sorted: true }), "the recovery quorum is no harder than the primary: that is the decay");
    // Same threshold over MORE keys is admitted: it is no harder to satisfy.
    assert!(presets::decaying_multisig(Wrapper::Wsh, 2, 3, 2, 5, 1000, 2000, 4_000_000).is_ok());
    assert_eq!(p.paths[2].lock, Some(Lock::AfterHeight(4_000_000)));
    for l in [
        presets::plain_multisig(Wrapper::Wsh, 2, 3).unwrap(),
        presets::simple_timelocked_inheritance(Wrapper::Wsh, 65535).unwrap(),
        presets::kofn_recovery(Wrapper::Tr, 2, 3, 52560).unwrap(),
        presets::tiered_recovery(Wrapper::Wsh, 2, 2, 2, 3, 4032).unwrap(),
        presets::hashlock_gated(Wrapper::Wsh, H1, 144).unwrap(),
        presets::decaying_multisig(Wrapper::Wsh, 2, 3, 1, 2, 1000, 2000, 4_000_000).unwrap(),
    ] {
        compose(&l).unwrap_or_else(|e| panic!("{l:?}: {e}"));
    }
}

#[test]
fn presets_lower_to_their_pinned_templates() {
    // Spec §10 item 3: "the five presets as Concrete policies + expected
    // templates". The Concrete-policy half is the §5b cross-check in
    // `compose_crosscheck.rs`; this is the expected-template half, pinned as
    // literals so a preset that drifts in SHAPE (which tier carries the lock,
    // which quorum is smaller) fails here.
    let h = "a8".repeat(32);
    let cases: Vec<(&str, PathList, String)> = vec![
        ("plain_multisig", presets::plain_multisig(Wrapper::Wsh, 2, 3).unwrap(),
         "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))".to_string()),
        ("simple_timelocked_inheritance", presets::simple_timelocked_inheritance(Wrapper::Wsh, 65535).unwrap(),
         "wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(65535))))".to_string()),
        ("kofn_recovery", presets::kofn_recovery(Wrapper::Tr, 2, 3, 52560).unwrap(),
         format!("tr({NUMS},{{multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(52560))}})")),
        ("tiered_recovery", presets::tiered_recovery(Wrapper::Wsh, 2, 2, 2, 3, 4032).unwrap(),
         "wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*),and_v(v:multi(2,@2/<0;1>/*,@3/<0;1>/*,@4/<0;1>/*),older(4032))))".to_string()),
        ("hashlock_gated", presets::hashlock_gated(Wrapper::Wsh, H1, 144).unwrap(),
         format!("wsh(or_i(and_v(v:pkh(@0/<0;1>/*),sha256({h})),and_v(v:pkh(@1/<0;1>/*),older(144))))")),
        ("decaying_multisig", presets::decaying_multisig(Wrapper::Wsh, 2, 3, 1, 2, 1000, 2000, 4_000_000).unwrap(),
         "wsh(or_i(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),older(1000)),or_i(and_v(v:multi(1,@3/<0;1>/*,@4/<0;1>/*),older(2000)),and_v(v:pkh(@5/<0;1>/*),after(4000000)))))".to_string()),
    ];
    for (name, list, expected) in cases {
        assert_eq!(text(&list), expected, "{name}");
    }
}

#[test]
fn presets_refuse_parameters_the_grammar_refuses() {
    assert!(matches!(presets::plain_multisig(Wrapper::Wsh, 3, 2), Err(ComposeError::BadThreshold { .. })));
    assert!(matches!(presets::simple_timelocked_inheritance(Wrapper::Wsh, 0), Err(ComposeError::LockOutOfRange { path: 1, .. })));
    assert!(matches!(presets::kofn_recovery(Wrapper::Wsh, 2, 3, 70_000), Err(ComposeError::LockOutOfRange { path: 1, .. })));
    // The refusal names the tier that carries the bad lock, not tier 1.
    assert_eq!(
        presets::tiered_recovery(Wrapper::Wsh, 2, 2, 2, 3, 70_000).unwrap_err().to_string(),
        "path 2: older in blocks needs 1..=65535"
    );
    // Decay must be a decay: later tiers unlock LATER, and the recovery quorum is not larger.
    assert!(matches!(presets::decaying_multisig(Wrapper::Wsh, 2, 3, 1, 2, 2000, 1000, 4_000_000), Err(ComposeError::PresetShape { .. })));
    assert!(matches!(presets::decaying_multisig(Wrapper::Wsh, 1, 2, 2, 3, 1000, 2000, 4_000_000), Err(ComposeError::PresetShape { .. })));
}
```

Add to `crates/md-codec/tests/compose_crosscheck.rs` (spec §10 item 3's Concrete-policy half: every preset through the §5b legs, with its own keys):

```rust
#[test]
fn every_preset_passes_the_5b_cross_check() {
    use md_codec::compose::presets;
    let cases: Vec<(&str, PathList)> = vec![
        ("plain_multisig", presets::plain_multisig(Wrapper::Wsh, 2, 3).unwrap()),
        ("simple_timelocked_inheritance", presets::simple_timelocked_inheritance(Wrapper::Wsh, 65535).unwrap()),
        ("kofn_recovery_tr", presets::kofn_recovery(Wrapper::Tr, 2, 3, 52560).unwrap()),
        ("kofn_recovery_wsh", presets::kofn_recovery(Wrapper::Wsh, 2, 3, 52560).unwrap()),
        ("tiered_recovery", presets::tiered_recovery(Wrapper::Wsh, 2, 2, 2, 3, 4032).unwrap()),
        ("hashlock_gated", presets::hashlock_gated(Wrapper::Wsh, H, 144).unwrap()),
        ("decaying_multisig", presets::decaying_multisig(Wrapper::Wsh, 2, 3, 1, 2, 1000, 2000, 4_000_000).unwrap()),
    ];
    for (name, list) in cases {
        let addr = cross_check(name, &list, false);
        assert!(addr.starts_with("bc1"), "{name}: {addr}");
    }
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo nextest run --locked -p md-codec --test compose_lowering presets 2>&1 | tail -5`
Expected: FAIL, `could not find presets in compose`.

- [ ] **Step 3: Write the presets**

Create `crates/md-codec/src/compose/presets.rs`:

```rust
//! The toolkit's five archetypes, plus plain k-of-n multisig, as path lists
//! over THIS grammar (spec §4d, C2). Same spend conditions as
//! `mnemonic build-descriptor`'s goldens, not byte-identical to them: this
//! lowering is one fixed spelling.

use super::{validate, ComposeError, KeySet, Lock, PathList, SpendPath, Wrapper};

fn ks(k: u8, n: u8) -> SpendPath {
    SpendPath { keys: Some(KeySet { k, n, sorted: true }), hash: None, lock: None }
}

/// `older` in blocks for the path at `path` (0-based), so a refusal names the
/// tier that carries the bad lock.
fn blocks(b: u32, path: usize) -> Result<Lock, ComposeError> {
    u16::try_from(b)
        .ok()
        .filter(|b| *b >= 1)
        .map(Lock::OlderBlocks)
        .ok_or(ComposeError::LockOutOfRange { path, why: "older in blocks needs 1..=65535" })
}

fn checked(list: PathList) -> Result<PathList, ComposeError> {
    validate(&list)?;
    Ok(list)
}

/// One unlocked k-of-n path: the Multisig Build shape C7 migrates.
pub fn plain_multisig(wrapper: Wrapper, k: u8, n: u8) -> Result<PathList, ComposeError> {
    checked(PathList { wrapper, paths: vec![ks(k, n)] })
}

/// Primary key now; heir after `older_blocks`.
pub fn simple_timelocked_inheritance(wrapper: Wrapper, older_blocks: u32) -> Result<PathList, ComposeError> {
    let mut heir = ks(1, 1);
    heir.lock = Some(blocks(older_blocks, 1)?);
    checked(PathList { wrapper, paths: vec![ks(1, 1), heir] })
}

/// k-of-n now; one recovery key after `older_blocks`.
pub fn kofn_recovery(wrapper: Wrapper, k: u8, n: u8, older_blocks: u32) -> Result<PathList, ComposeError> {
    let mut recovery = ks(1, 1);
    recovery.lock = Some(blocks(older_blocks, 1)?);
    checked(PathList { wrapper, paths: vec![ks(k, n), recovery] })
}

/// k1-of-n1 now; k2-of-n2 (distinct keys) after `older_blocks`.
pub fn tiered_recovery(wrapper: Wrapper, k1: u8, n1: u8, k2: u8, n2: u8, older_blocks: u32) -> Result<PathList, ComposeError> {
    let mut tier2 = ks(k2, n2);
    tier2.lock = Some(blocks(older_blocks, 1)?);
    checked(PathList { wrapper, paths: vec![ks(k1, n1), tier2] })
}

/// A key plus a hash now; a second key after `older_blocks`.
pub fn hashlock_gated(wrapper: Wrapper, hash: [u8; 32], older_blocks: u32) -> Result<PathList, ComposeError> {
    let mut gated = ks(1, 1);
    gated.hash = Some(hash);
    let mut later = ks(1, 1);
    later.lock = Some(blocks(older_blocks, 1)?);
    checked(PathList { wrapper, paths: vec![gated, later] })
}

/// k1-of-n1 after `older1`; a recovery quorum k2-of-n2 (distinct keys) that is
/// NO HARDER to satisfy than the primary (`k2 <= k1`; `n2` is free, since more
/// keys at the same threshold only widen the ways to spend) after
/// `older2 > older1`; one final key after `after_height`. The toolkit's
/// archetype takes the primary and recovery quorums as separate parameters and
/// refuses tiers that do not unlock progressively later; so does this. What
/// "decay" means here is exactly those two guards, nothing more.
#[allow(clippy::too_many_arguments)]
pub fn decaying_multisig(
    wrapper: Wrapper,
    k1: u8,
    n1: u8,
    k2: u8,
    n2: u8,
    older1: u32,
    older2: u32,
    after_height: u32,
) -> Result<PathList, ComposeError> {
    if older2 <= older1 {
        return Err(ComposeError::PresetShape { why: "decaying tiers must unlock progressively later (the second older must exceed the first)" });
    }
    if k2 > k1 {
        return Err(ComposeError::PresetShape { why: "a decaying multisig decays: the recovery threshold cannot exceed the primary threshold" });
    }
    let mut t1 = ks(k1, n1);
    t1.lock = Some(blocks(older1, 0)?);
    let mut t2 = ks(k2, n2);
    t2.lock = Some(blocks(older2, 1)?);
    let mut t3 = ks(1, 1);
    t3.lock = Some(Lock::AfterHeight(after_height));
    checked(PathList { wrapper, paths: vec![t1, t2, t3] })
}
```

Add to `crates/md-codec/src/compose/mod.rs`, directly after `mod tr;`:

```rust
pub mod presets;
```

- [ ] **Step 4: Run, format, clippy, commit**

Run: `cargo nextest run --locked -p md-codec 2>&1 | tail -5 && cargo fmt --all && cargo clippy --locked -p md-codec --all-targets --all-features -- -D warnings 2>&1 | tail -3`
Expected: all PASS, clean.

```bash
git add crates/md-codec/src/compose/mod.rs crates/md-codec/src/compose/presets.rs crates/md-codec/tests/compose_lowering.rs crates/md-codec/tests/compose_crosscheck.rs
git commit -m "md-codec: compose presets -- plain k-of-n and the five toolkit archetypes as path lists, pinned templates, 5b-checked (composer S0 task 7)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 8: `md encode` gates a signature-free spend path under every wrapper, not only `tr`

**Files:**
- Modify: `crates/md-cli/src/parse/template.rs` (`parse_template_ext`, the `ms_desc` construction, `crates/md-cli/src/parse/template.rs:2677-2702`)
- Test: `crates/md-cli/tests/cli_compose_encode_gate.rs`

**Interfaces:**
- Consumes: `miniscript::Descriptor::sanity_check`, `Miniscript::ext_check(&ExtParams) -> Result<(), AnalysisError>` (rust-miniscript's `analyzable.rs`, line 242 at the pinned `ff4732e`; a git dependency, so outside the cite gate's roots), `miniscript::descriptor::ShInner::{Wsh, Wpkh, Ms}`, `Wsh::as_inner() -> &Miniscript<Pk, Segwitv0>`, `Sh::as_inner() -> &ShInner<Pk>`.
- Produces: `md encode` refusing "All spend paths must require a signature" for `wsh`/`sh(wsh)`/`sh` exactly as it already does for `tr`, and admitting the shape with `--experimental` plus the existing warning.

**Why this is in Stage 0.** Measured at `3b0944fb` with `target/debug/md`: `md encode` on `wsh(or_d(multi(2,...),and_v(v:sha256(H),after(1383520))))` exits 0 with no warning, keyed (`--key @0..@2`) or unkeyed; the same shape under `tr` is refused with `template parse error: miniscript parse failed: All spend paths must require a signature`. Cause: `Descriptor::from_str` runs the sanity gate only for `tr` (the code's own comment at `crates/md-cli/src/parse/template.rs:2678` calls it "`from_str`'s tr-only sanity gate"), and `parse_template_ext` never calls `sanity_check()` itself. After Task 6, `md compose` refuses that shape without `--experimental` and then prints a template `md encode` accepts silently — the EXPERIMENTAL gate one command deep. Follow-up `md-encode-keyless-template-sigless-path-not-gated` in descriptor-mnemonic `design/FOLLOWUPS.md` is owned by this stage.

- [ ] **Step 1: Write the failing tests**

Create `crates/md-cli/tests/cli_compose_encode_gate.rs`:

```rust
//! `md encode` and `md compose` must agree on the EXPERIMENTAL gate: a spend
//! path that needs no signature is refused under EVERY wrapper unless
//! `--experimental`, which then warns. Before this task only `tr` was gated.

use assert_cmd::Command;
use predicates::prelude::*;

fn md() -> Command {
    Command::cargo_bin("md").expect("md binary")
}

const H: &str = "a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8";
const XPUB: [&str; 3] = [
    "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf",
    "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk",
    "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR",
];

fn sigless_wsh() -> String {
    format!("wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*),and_v(v:sha256({H}),after(1383520))))")
}

#[test]
fn encode_refuses_a_sigless_wsh_path_unkeyed_unless_experimental() {
    md().args(["encode", &sigless_wsh()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("must require a signature"));
    md().args(["encode", "--experimental", &sigless_wsh()])
        .assert()
        .success()
        .stderr(predicate::str::contains("relaxed the signature rule"));
}

#[test]
fn encode_refuses_a_sigless_wsh_path_keyed_unless_experimental() {
    let keyed = |extra: &[&str]| {
        let mut args: Vec<String> = vec!["encode".into()];
        args.extend(extra.iter().map(|s| s.to_string()));
        args.push(sigless_wsh());
        for (i, x) in XPUB.iter().enumerate() {
            args.push("--key".into());
            args.push(format!("@{i}={x}"));
            args.push("--fingerprint".into());
            args.push(format!("@{i}=73c5da0a"));
        }
        md().args(&args).assert()
    };
    keyed(&[]).failure().code(1).stderr(predicate::str::contains("must require a signature"));
    keyed(&["--experimental"]).success().stderr(predicate::str::contains("relaxed the signature rule"));
}

#[test]
fn encode_still_admits_a_signed_wsh_policy_without_the_flag() {
    let two_path = "wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*),and_v(v:pkh(@3/48'/0'/3'/2'/<0;1>/*),older(26280))))";
    md().args(["encode", two_path])
        .assert()
        .success()
        .stderr(predicate::str::contains("signature").not());
}
```

- [ ] **Step 2: Run to verify the two refusals fail today**

Run: `cargo nextest run --locked -p md-cli --test cli_compose_encode_gate 2>&1 | tail -8`
Expected: the two `*_unless_experimental` tests FAIL (the unflagged encode exits 0 today); the regression guard PASSES.

- [ ] **Step 3: Run the sanity check for every wrapper, and the relaxed re-check for every wrapper**

In `crates/md-cli/src/parse/template.rs`, `parse_template_ext`, replace the `let ms_desc = if experimental { ... } else { ... };` construction (`:2677-2702`) with:

```text
    let ms_desc = if experimental {
        // Parse the tree WITHOUT `from_str`'s tr-only sanity gate, then re-apply
        // every rule except `top_unsafe` ourselves -- per leaf for `tr`, on the
        // one script for `wsh`, `sh(wsh)` and `sh`.
        let tree = miniscript::expression::Tree::from_str(&substituted)
            .map_err(|e| CliError::TemplateParse(format!("miniscript parse failed: {e}")))?;
        let d = <MsDescriptor<DescriptorPublicKey> as miniscript::expression::FromTree>::from_tree(
            tree.root(),
        )
        .map_err(|e| CliError::TemplateParse(format!("miniscript parse failed: {e}")))?;
        let relaxed = miniscript::miniscript::analyzable::ExtParams::new().top_unsafe();
        // `ext_check` returns miniscript's analysis error, not `miniscript::Error`;
        // a generic helper takes whatever it is.
        fn relaxed_err<E: std::fmt::Display>(e: E) -> CliError {
            CliError::TemplateParse(format!(
                "miniscript parse failed even with --experimental: {e} \
                 (--experimental relaxes ONLY the signature rule; malleability, \
                 resource limits, repeated keys and timelock mixing still apply)"
            ))
        }
        // MINTING verbs only (`Disposition::Refuse`: encode). Reading verbs
        // (`Warn`: verify, inspect) must keep reading already-engraved plates
        // whose shapes the sanity rules reject -- the N1 C1 placement
        // constraint; measured: an unconditional check here made `md verify`
        // refuse a legacy `sh(multi(1,@0/**,@0/**))` plate (repeated keys),
        // failing n1_admission_taxonomy's two reading-verb tests.
        let minting = matches!(reuse, crate::parse::reuse::Disposition::Refuse);
        match &d {
            MsDescriptor::Tr(inner) => {
                for item in inner.leaves() {
                    item.miniscript().ext_check(&relaxed).map_err(relaxed_err)?;
                }
            }
            MsDescriptor::Wsh(w) if minting => w.as_inner().ext_check(&relaxed).map_err(relaxed_err)?,
            MsDescriptor::Sh(sh) if minting => match sh.as_inner() {
                miniscript::descriptor::ShInner::Wsh(w) => w.as_inner().ext_check(&relaxed).map_err(relaxed_err)?,
                miniscript::descriptor::ShInner::Ms(ms) => ms.ext_check(&relaxed).map_err(relaxed_err)?,
                miniscript::descriptor::ShInner::Wpkh(_) => {}
            },
            _ => {}
        }
        d
    } else {
        let d = MsDescriptor::<DescriptorPublicKey>::from_str(&substituted)
            .map_err(|e| CliError::TemplateParse(format!("miniscript parse failed: {e}")))?;
        // `from_str` runs the sanity gate for `tr` only; `md compose` refuses a
        // signature-free path under every wrapper, and `encode` must agree
        // (follow-up md-encode-keyless-template-sigless-path-not-gated). MINTING
        // verbs only, for the reason given in the experimental branch above.
        if matches!(reuse, crate::parse::reuse::Disposition::Refuse) {
            d.sanity_check()
                .map_err(|e| CliError::TemplateParse(format!("miniscript parse failed: {e}")))?;
        }
        d
    };
```

`ext_check` returns miniscript's analysis error type, not `miniscript::Error` (a typed closure fails with E0631 at the pinned revision, measured); the generic `relaxed_err` above accepts it. Machine-checked by the controller in the gate's scratch copy before this plan was committed: see the fold commit message for the md-cli suite result.

- [ ] **Step 4: Run the md-cli suite WHOLE, and read any pre-existing failure as a finding**

Run: `cargo nextest run --locked -p md-cli 2>&1 | tail -12`
Expected: all PASS, including `cli_compose_encode_gate`, `cmd_encode`'s `experimental_admits_a_keyless_spend_path` (tr, unchanged), `n1_admission_taxonomy`'s two reading-verb tests (`r_n1a_card_verifies_at_exit_0_with_a_warning`, `verify_template_warns_and_completes_on_a_refused_shape` — these are WHY the gate is minting-only; an unconditional gate failed both, measured in the plan's scratch copy) and every corpus test. **If a PRE-EXISTING test or corpus vector is now refused by `sanity_check` (malleability, timelock mixing, resource limits), STOP and report it: a shipped template that rust-miniscript's sanity rules reject is a finding for the operator, not something to relax here.** Then: `cargo nextest run --locked -p md-codec 2>&1 | tail -3` (unchanged, must stay green).

- [ ] **Step 5: Format, clippy, commit**

Run: `cargo fmt --all && cargo clippy --locked -p md-cli --all-targets --all-features -- -D warnings 2>&1 | tail -3`
Expected: clean.

```bash
git add crates/md-cli/src/parse/template.rs crates/md-cli/tests/cli_compose_encode_gate.rs
git commit -m "md-cli: encode runs the miniscript sanity gate for every wrapper, not only tr; --experimental relaxes the signature rule alike (composer S0 task 8)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 9: Whole-stage gate, vendoring, release notes

**Files:**
- Modify: `descriptor-mnemonic/CHANGELOG.md` (repo root; its unreleased heading is `## md-cli [Unreleased]`, and md-codec gets a matching `## md-codec [Unreleased]` heading if none exists), per `descriptor-mnemonic/design/RELEASE_PROCESS.md` (there is no `RELEASE_CHECKLIST.md`; measured)
- Vendor: copy `/tmp/compose-vectors/keyed_compose_*.{template,bytes.hex,phrase.txt,descriptor.json,conformance.json}` into the fork's `md/testdata/vectors/` in a SEPARATE commit on the fork, made by Stage 2's implementer, not here (Rust first; the fork is not touched by this stage).

- [ ] **Step 1: Run the whole workspace the way CI does**

Run: `cd /scratch/code/shibboleth/descriptor-mnemonic && cargo fmt --all --check && cargo clippy --locked --workspace --all-targets --all-features -- -D warnings 2>&1 | tail -3 && cargo nextest run --locked --workspace --all-features 2>&1 | tail -6 && cargo test --locked --workspace --all-features --doc 2>&1 | tail -3`
Expected: fmt clean, clippy clean, every test PASS, doctests PASS. (`cargo test` with threads, not only nextest: CI's threaded runner has exposed shared-state bugs nextest's process isolation hid.)

- [ ] **Step 2: Regenerate and diff the vector corpus**

Run: `cargo run --locked -p md-cli --bin md -- vectors --out crates/md-codec/tests/vectors 2>&1 | tail -2 && git status --short crates/md-codec/tests/vectors | head -30`
Expected: only NEW `compose_*` / `keyed_compose_*` files appear (26 vectors' worth; the two `no-corpus` entries produce none); no existing vector file changes (a changed pre-existing file means the lowering or the exporter altered something it must not).

- [ ] **Step 3: Record the release note and commit**

Add a line under the unreleased section of the crate's changelog naming: `md_codec::compose` (new module, unconditional), `md compose` (new subcommand), the 28 compose vectors, the dev-dependency on miniscript's `compiler` feature, and the `md encode` behaviour change (a signature-free spend path is now refused under every wrapper unless `--experimental`; before, only under `tr`).

```bash
git add crates/md-codec/tests/vectors CHANGELOG.md
git commit -m "md-codec/md-cli: compose vector corpus regenerated; release note (composer S0 task 9)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

- [ ] **Step 4: Hand off**

The stage is complete when: every task's commit exists; the workspace gate of this task is green; the whole-diff independent review (an opus execution review over `git diff <baseline>..HEAD`, persisted to `mnemonic-engrave/design/agent-reports/composer-S0-exec-review-r0.md`) returns 0 Critical / 0 Important after folds; and the version bump + crates.io publish follow `descriptor-mnemonic/design/RELEASE_PROCESS.md`. Stage 1 begins only then, with its own detailed plan.

---

## Self-review (run by the plan author before dispatch)

1. **Spec coverage, Stage 0 scope (matches `STAGED_PLAN` S0: §5, §10 items 1 and 3, §12 items 1, 4, 7):** §4 bounds → Task 1 `validate` + tests; §4c → `Lock::operand` + tests; §4f defaults and invariant → Task 2 `origins` + Task 3 tests; §5 rows → Tasks 2 and 3 (key set, conjunct order, or_d/or_i, spine, internal key, NUMS, numbering, use-site); §5b → Task 4's `cross_check`, run over the reference wallets (Task 4), the whole family (Task 5) and every preset (Task 7); §4d presets and §10 item 3 (Concrete policies = the preset cross-check; expected templates = `presets_lower_to_their_pinned_templates`) → Task 7; §10 item 1 → Task 6, and the `md encode` parity it needs → Task 8; §12 item 1 → Task 5 tagged coverage with the spec's required tag list asserted; §12 item 4's HOST half (every §4e refusal, the §4c bands in and out per kind including `older(0x400000)`, the 33rd slot, the one-fingerprint invariant) → Tasks 1-3 tests, its device half → Stage 3; §12 item 7's host half → the lock tests, its device half → Stage 2. Not in this stage by design: §6 inputs (Stage 1), §7-§9 device work (Stages 2-3), §12 items 2, 3, 5, 6, 8-11 (Stages 2-4).
2. **Placeholder scan:** no TBD/TODO; every code step carries its code. The manifest entries in Task 5 are generated by the printer test and pasted, which is stated as the method rather than left implicit.
3. **Type consistency:** `compose(&PathList) -> Result<Composed, ComposeError>`, `compose_with(&PathList, &[Option<SlotOrigin>])`, `Composed { descriptor, slots, internal_key_path, experimental }`, `Slot { index, path, ordinal }`, `Lock::operand() -> Result<(Tag, u32), &'static str>`, `Wrapper::script_type() -> u32`, `presets::*(..) -> Result<PathList, ComposeError>` are used with these exact shapes in every task.

## What the build gate covers, and does not

`scripts/plan-build-gate-md.sh design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md` copies descriptor-mnemonic WITH its `rust-toolchain.toml` (1.85.0) and `.cargo/`, assembles `crates/md-codec/src/compose/{mod,lowering,tr,presets}.rs` (Tasks 1-3 and 7 in plan order; a `Replace` anchor makes the stub `lowering.rs` and `tr.rs` give way to their full files, and the `mod tr;` / `pub mod presets;` additions to `mod.rs` are appended blocks), builds md-codec with all targets, runs the compose test binaries (`compose_lowering.rs`, `compose_crosscheck.rs`, `compose_vectors.rs`, and the test-less `compose_support.rs`) with `--no-fail-fast`, clippies md-codec with `-D warnings`, and compile-checks md-cli with `cmd/compose.rs`, `cli_compose.rs` and `cli_compose_encode_gate.rs` present. It synthesises the one-line fragments the plan states in prose: `pub mod compose;` in both crates, the `miniscript` `compiler` dev-dependency, and the `CliError::Compose(String)` variant with its `Display` arm.

Result at the round-0 fold commit: md-codec builds; 51 of 52 compose tests pass (the family-wide and preset §5b cross-checks among them) and the 52nd is the PINNED red (`every_compose_vector_in_the_manifest_is_exactly_what_compose_renders` failing with `MANIFEST lacks`, because `test_vectors.rs` is a fragment the gate does not assemble; the gate accepts exactly that failure and no other); clippy clean; md-cli compiles.

Two tests pass VACUOUSLY in the gate because `MANIFEST` holds no compose entry there: `every_compose_manifest_entry_is_in_the_family` and `keyed_compose_vectors_bind_at_most_the_four_journey_keys` (0 iterations). Task 5's paste makes both real; until then they prove nothing and are not counted as coverage.

NOT covered by the gate: the `main.rs` clap variant and dispatch arm; Task 8's `parse/template.rs` change; the `test_vectors.rs` MANIFEST entries (pasted from the printer in Task 5); `lib.rs` beyond the module line; the `md vectors` export count; the Go port.

Checked BY HAND at the round-1 fold, in the gate's scratch copy with the `main.rs` fragments and Task 8's `parse/template.rs` fragment applied: `cargo nextest run -p md-cli --locked --no-fail-fast` → 761 passed, 0 failed, 1 skipped — every `cli_compose.rs` assertion, every `cli_compose_encode_gate.rs` assertion, and the pre-existing suite including `n1_admission_taxonomy`'s reading-verb tests (which the first, unconditional draft of Task 8 had failed). Still unchecked by anything but the implementer's Task 5 and Task 9 runs: the 26 pasted MANIFEST entries through the corpus tests and the exporter.
