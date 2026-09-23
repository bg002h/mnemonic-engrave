# Coordinator compatibility, plan 1b — verdicts, the table, and the host verdict

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development`
> (recommended) or `superpowers:executing-plans` to implement this plan
> task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give md-codec the four `Verdict` kinds, a registry of source-derived
refusal rules, and a GENERATED table of measured cells built from vendored
evidence by an xtask; give md-cli `md shape-key` and a coordinator verdict on
`md compose` and `md descriptor`, with compose's none-case refusal and
`--md-only`. This is design §5 steps 1-2 and §3 mechanisms 1-2.

**Architecture:** Rules (refusal-only, each citing source) and evidence
(measured rows vendored from mnemonic-engrave) meet once, at a table build
(`md_codec::coordinator::build`, driven by `cargo xtask verdicts`), which keys
every evidence row through the ONE key implementation and fails on any
disagreement (D1-D5). At runtime a verdict per coordinator per run of
verified versions is: a spelling refusal, then a rule refusal, then — only
with keys present — a measured cell, else `Unproven`. md-cli prints those
lines and holds no coordinator knowledge of its own.

**Tech Stack:** Rust 2024, rust-version 1.85 (workspace pins); one new
workspace member (`crates/xtask`, `publish = false`, deps `md-codec` +
`serde_json`, both already in `Cargo.lock`); Python 3 in the vendoring script
only (it transcribes; it never computes a key).

**Design:** `mnemonic-engrave/design/DESIGN_coordinator_compatibility.md`
after its fourth fold (2026-09-23). Read §1, §1A, §2, §3 and §4 "`md` on the
host", and the "Controller rulings before plan 1b" section.

**Baseline:** descriptor-mnemonic `main` **`d269c556`** (md-codec 0.47.0,
md-cli 0.19.0); fork `main` **`2c9eed3`** where cited; mnemonic-engrave
`1154e716`. Every citation below is measured there.

**Where the work happens:** worktree `/scratch/code/shibboleth/dm-worktrees/cc-1b`,
branch **`cc-1b-verdicts`**, off `d269c556`. It ships as a **pull request**;
the whole-branch review is the operator's **`/code-review ultra <PR#>`**, which
replaces the opus whole-diff review — **none is scheduled here.** The
plan-level review loop (R0 to 0C/0I) is unchanged. Rust only: the fork is
plan 3.

---

## 1. R-1 and R-2 are implementable — each with one precision

**R-1 (the Nunchuk leaf order): implemented as ruled, and it stays inside
ruling 3.** R-1 reads "accept iff the leaf pubkeys are sorted-unique". A rule
may only REFUSE (ruling 3: positives are measured-only), so the rule is the
refusing half — kind 1 with leaf pubkeys NOT strictly ascending refuses,
renderer `None`, reads `KEY_IDENTITY` so a template is
`Unproven { KeysAbsent }` — and the accepting half is a MEASURED cell: the
recon's `k1-liana-sorted` probe row. That row lives only in
`/scratch/code/shibboleth/.tmp/recon-1b-nunchuk-probe.*` today, so **Task 0
commits it into mnemonic-engrave as evidence** before it can be vendored.
The key-dependent fact rides on a new NON-key `Skeleton` field,
`leaf_keys_ascending: Option<bool>`, so the SkeletonKey is untouched
(measured: the two recon cards still share one key, and the table stores only
the sorted card's cell — the unsorted card's refusal is explained by the rule,
so there is no D5 conflict).

**R-2 (evidence transport): implemented as ruled, with one limit stated.**
`scripts/vendor-coord-evidence.sh <engrave>` writes
`crates/md-codec/tests/fixtures/coordinator/evidence.jsonl` plus
`evidence.meta.json` (the sha256 of every input), and `--check` regenerates
into a temp dir and fails when either differs. The existing
`vendor-liana-evidence.sh` has NO freshness check — the pattern is extended,
not copied. **The limit:** `--check` needs an engrave checkout, which
descriptor-mnemonic's CI does not have, so it is a LOCAL gate run in Task 9
(and by anyone re-vendoring), not a CI job. What CI does check is the other
half of freshness: the xtask test `table_is_fresh` fails whenever the
committed table is not what the committed evidence builds.

## 2. What the build already told us (settled; do not re-derive)

The whole plan was applied to a scratch copy of `d269c556` and built. These
are measurements, not predictions:

- **526 evidence rows** from 21 engrave inputs (Liana 8.0 and 15.0, Nunchuk
  2.1.1, Core 24.2-31.1). The build stores **228 cells**, explains **204
  refusals** by rules, and confirms **2 rows unkeyable** (a stale-key Liana
  control; Nunchuk's PR-1746 form). **Zero disagreements** after three fixes
  the build forced, all recorded in the design fold:
  - **D3, Core spells first:** on 24.2-25.2 a multipath `tr` miniscript is
    refused for its `<0;1>` (`Key path value '<0;1>' is not a valid
    uint32`), not for its miniscript. Spelling refusals are checked before
    clauses, in the build and at runtime.
  - **Liana class 9 narrowed** (the design's "second unlocked path"): X24's
    second SINGLE-key unlocked path is imported (folded, `ImportsAltered`
    2-of-4); X25/X26's second MULTI-key one is refused.
  - **D3, Liana checks time units before absolute locks:** the fork's
    order (class 5 `after`, then class 6 older-units) names "an absolute
    lock" for `mixed-lock-bases-{wsh,tr}`, whose Liana message at both 8.0
    and 15.0 is `Timelock value '4194404' isn't valid or safe to use`
    (evidence lines `fable-liana-parse-out.jsonl:136`,
    `fable-liana-parse-out-v15.jsonl:131`, `:136`). Swapping the two back to
    the Go order reproduces exactly those three D3s. The Rust order is the
    measured one; the Go is now the one to converge (F-668, Task 0).
- **Every Liana refusal in evidence is explained by a clause**, so Liana's
  rule set does not admit measured refusals (a new unexplained one is a D2).
  Nunchuk's does (its refusals are spelling claims, design §1).
- **A template's key is not its seated card's key:** 1a renders a keyless
  card's partitions EMPTY (`[[][][]][]`), not as singletons. Harmless (a2
  forbids a template any positive regardless), and pinned by a test that
  keeps the seated key and clears only `keys_present`.
- **The none case is reachable from `md compose` only because Nunchuk gets a
  second source-derived clause** — a keyless `wsh` path, from libnunchuk
  `a7cfb49` `src/descriptor.cpp:573` → `src/nunchukutils.cpp:1276` (`IsSane`)
  → `contrib/bitcoin` `57b47c4` `src/script/miniscript.h:1617` (`IsSane`
  includes `NeedsSignature`). Without it, Nunchuk is `Unproven` on every
  template and compose's refusal could never fire. With it, a keyless
  `--experimental` compose is refused by all three, which **changes four
  existing md-cli tests** (they gain `--md-only`; Task 8) — a behaviour
  change the CHANGELOG states.
- **The F-644 shapes** each print a Liana refusal on `md compose`, with or
  without `--unspendable liana` (Task 8's test).

## 3. Deviations from the design, each named

- **`Description` has no `paths` field.** No harness records a per-path
  rendering in a comparable form; `wallet_kind` and `threshold` are what the
  Liana and Nunchuk harnesses record. `ImportsAltered` is derived from those
  (never hand-marked), scoped to the two measured cases: Liana's primary
  `k`-of-`n` (X24) and Nunchuk's single-path threshold read as `MINISCRIPT`
  (F-626). Core's reading is its addresses; a mismatch would be
  `ADDRESSES_DIFFER` (none in evidence).
- **`RuleSet.refuse` is a list of `Clause`s, each with its own `ReadSet`**,
  because design §1 (a2) asks per rule whether it reads key identity, and one
  opaque `fn` per rule set cannot answer that per class. A `FormRefusal`
  list carries spelling refusals (Core's `<0;1>`).
- **Measured verdicts carry `measured_at`** (design §3 mechanism 1 requires a
  date on every measured verdict; the design's enum omitted it).
- **Spans are runs over VERIFIED versions only.** `Core 26.0-31.1` asserts
  each verified release in that run (26.0, 26.2, 27.2, 28.4, 29.4, 30.3,
  31.1), not the unlisted point releases between them — the core-boundary
  report's own caveat.
- **The table generator is `crates/xtask`** (the design's "step-1 xtask"),
  not an md-codec example; the decision logic lives in md-codec
  (`coordinator::build`), so the xtask only parses JSON and writes a file.

## Global Constraints

- **Rust-primary**, descriptor-mnemonic only. No fork change.
- **Workspace pins:** edition 2024, `rust-version = "1.85"`. No new external
  dependency.
- **Key membership is untouched** (design §1A). R-1's fact is a `Skeleton`
  field outside `skeleton_key`.
- **A rule only refuses.** A positive comes from a table cell, never a rule.
- **One key implementation.** The xtask, the evidence gate and
  `md shape-key --descriptor` all call `md_codec::descriptor_route`; the
  vendoring script computes nothing.
- **Toolchain:** `export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH`
  (`rustup run 1.85.0` is not enough; local clippy otherwise resolves to a
  newer one). Use your own `CARGO_TARGET_DIR` under
  `/scratch/code/shibboleth/.tmp/`, never `/tmp`.

## The gate — four commands, at every task boundary

```sh
cargo nextest run --workspace --locked --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo fmt --all --check
```

(`scripts/phase-gate.sh` is a superset and may be run instead.)

**Result, measured while writing this plan:** every block below was applied
mechanically, task by task, to a `git archive` of `d269c556`; Task 5's
script was run against a clone of engrave `1154e716` carrying Task 0's
probe files; `cargo xtask verdicts` generated the table; then the four
commands ran at **each of the nine task boundaries: all green** — nextest
1537 → 1538 → 1539 → 1540 → 1540 → 1548 → 1551 → 1555 → 1555 passed, 4
skipped (pre-existing), doc/clippy/fmt rc 0. The assembled tree was compared
file-by-file with the tree the tests were developed in: identical.

**Re-running it after a fold:** `/scratch/code/shibboleth/mnemonic-engrave/scripts/cc1b-plan-extract.py <this plan>`
applies the blocks task by task to a fresh `git archive` of `d269c556`, runs
the plan's generator commands (Task 5's script against the engrave clone
`/scratch/code/shibboleth/.tmp/cc1b-engrave-mirror`, which carries Task 0
Step 1; Task 6's `cargo xtask verdicts`), and diffs each task's tree against
the tree the gate ran on. It is scratch and uncommitted; if this plan is
folded more than once, commit it as `scripts/plan-build-gate-cc1b.py`.

**Coverage line — what that run does NOT cover:** Task 0 (engrave records and
the evidence commit), the PR / push / tag steps, the Windows and macOS legs
of CI (the `.gitattributes` lines in Task 5 exist for Windows and were not
exercised), `--check` against the REAL engrave after Task 0 (Task 9 runs it),
and the named mutations — 16 of them were run and each reddened its test (marked
*measured* below); the rest are stated, not measured.

---

### Task 0: Controller pre-steps in mnemonic-engrave (records, no code)

Not implementer work. Two commits on engrave `master`, before the worktree.

- [ ] **Step 1: Commit the recon's Nunchuk probe as evidence** (R-1's
  measured positive):

```sh
cd /scratch/code/shibboleth/mnemonic-engrave
mkdir -p design/evidence/coord-compat-1b
for x in py tsv out; do
  cp /scratch/code/shibboleth/.tmp/recon-1b-nunchuk-probe.$x design/evidence/coord-compat-1b/nunchuk-kind1-probe.$x
done
git add design/evidence/coord-compat-1b
git commit -m "evidence: the recon's Nunchuk kind-1 probe (R-1), libnunchuk a7cfb49"
```

  The `.out` has three sections (`k1-liana-unsorted` REFUSE,
  `pr1746-nunchuk-form` ACCEPT, `k1-liana-sorted` ACCEPT); the `.tsv` holds
  their descriptors. The vendoring script reads exactly these two names.

- [ ] **Step 2: FOLLOWUPS (engrave `design/FOLLOWUPS.md`).** File the six
  deferred minors of `design/agent-reports/coord-compat-1a-final-review.md:368-377`
  (never filed, recon §1d), as F-661..F-666 with a `**Status:**` line each:
  #1 `plain_multi` k/n guard — **owning phase: plan 1b Task 1** (closes
  there); #2 the 20→32 pad in two places, #3 `validate.rs`'s `1 << 22`, #4
  `skeleton()` accepts what `validate()` rejects (cross-reference F-634),
  #5 the defensive `root_kind` arms (three, N-1), #6 the walker pins only the
  rendered projection — all five **owning phase: none (ownerless residue)**,
  #6 noting plan 1b moved the walker into `md_codec::descriptor_route`.
  **Re-own F-644's md-cli half** to "coordinator-compat plan 1b, Task 8".
  **Mark F-655** "owning phase: plan 1b Task 2". File **F-668**: the fork's
  `composerLianaOutsideModelClass` checks `after` (class 5) before
  older-units (class 6), and Liana hits the units check first
  (mixed-lock-bases, both versions; plan 1b §2) — owning phase: plan 3
  (convergence). File **F-667**: md-cli's
  `decompose/walk.rs:259-282` is a second copy of the kind-1 leaf-walk
  recogniser now public in `md_codec::descriptor_route` — owning phase: next
  descriptor-mnemonic release. Commit: `records: file the 1a minors, re-own
  F-644 and F-655 to coord-compat 1b`.

- [ ] **Step 3: Worktree.**
  `git -C /scratch/code/shibboleth/descriptor-mnemonic worktree add -b cc-1b-verdicts /scratch/code/shibboleth/dm-worktrees/cc-1b d269c556`

---

### Task 1: The `plain_multi` k/n guard (1a final review M-1)

Owned "before plan 1b's first rule reads `k`/`n`" — Task 4's
`liana_reads_as_built` and `nunchuk_reads_as_built` read them.
`plain_multi` reports `indices.len()` (duplicates included) with no
`nkeys == slots.len()` guard (`policy_shape.rs:454-457`, `:532-544` at
`d269c556`), so `wsh(multi(2,@0,@0,@1))` reads 2-of-3 over two keys while the
same multi behind a lock reads 0-of-0.

**Files:** Modify `crates/md-codec/src/policy_shape.rs`; Test
`crates/md-codec/tests/policy_shape.rs`.

- [ ] **Step 1: Write the failing test.** Mutation that reds it: delete the
  guard added in Step 3.

Modify `crates/md-codec/tests/policy_shape.rs`:

```diff
diff --git a/crates/md-codec/tests/policy_shape.rs b/crates/md-codec/tests/policy_shape.rs
index fa770ec..ebff3d1 100644
--- a/crates/md-codec/tests/policy_shape.rs
+++ b/crates/md-codec/tests/policy_shape.rs
@@ -579,3 +579,30 @@ fn andor_splits_into_x_and_y_or_z() {
     assert_eq!(shape.branches[0].slots, vec![0, 1]);
     assert_eq!(shape.branches[1].slots, vec![2]);
 }
+
+/// 1a final review M-1, owned "before plan 1b reads k/n" (plan 1b's
+/// `liana_reads_as_built` and `nunchuk_reads_as_built` read them). A
+/// duplicate-index multi has three indices and two distinct keys: no k-of-n
+/// describes it, and the bare spelling must answer as the wrapped one does.
+/// Mutation: delete `plain_multi`'s `nkeys == slots.len()` guard in
+/// `branch_of` -> the bare spelling reports 2-of-3 and this reds.
+#[test]
+fn a_duplicate_index_multi_reports_no_threshold_in_either_spelling() {
+    let bare = common::descriptor_of(wrap(Tag::Wsh, multikeys(Tag::Multi, 2, vec![0, 0, 1])), 2);
+    let locked = common::descriptor_of(
+        wrap(
+            Tag::Wsh,
+            node2(
+                Tag::AndV,
+                wrap(Tag::Verify, timelock(Tag::Older, 9)),
+                multikeys(Tag::Multi, 2, vec![0, 0, 1]),
+            ),
+        ),
+        2,
+    );
+    for d in [bare, locked] {
+        let b = &policy_shape(&d).branches[0];
+        assert_eq!(b.slots, vec![0, 1]);
+        assert_eq!((b.k, b.n), (0, 0), "no k-of-n for a duplicate-index multi");
+    }
+}
```

- [ ] **Step 2:** `cargo nextest run -p md-codec --test policy_shape` — FAIL
  (`(2, 3)` for the bare spelling).
- [ ] **Step 3: The guard.**

Modify `crates/md-codec/src/policy_shape.rs`:

```diff
diff --git a/crates/md-codec/src/policy_shape.rs b/crates/md-codec/src/policy_shape.rs
index 1149433..e62b641 100644
--- a/crates/md-codec/src/policy_shape.rs
+++ b/crates/md-codec/src/policy_shape.rs
@@ -451,10 +451,19 @@ fn branch_of(n: &Node, depth: u8) -> Option<Branch> {
 
     // A bare threshold-over-keys, possibly wrapped: report k-of-n and
     // whether it was the sorted spelling.
+    //
+    // `plain_multi` takes the SAME `nkeys == slots.len()` guard as
+    // `sole_multi` below (1a final review M-1): `slots` is deduplicated, so a
+    // duplicate-index multi such as `multi(2,@0,@0,@1)` has 3 indices and 2
+    // distinct keys, and "2-of-3" would claim a third signer that does not
+    // exist. Unguarded, one multi answered two ways depending on whether a
+    // wrapper or a lock sat around it.
     if let Some((k, nkeys, sorted)) = plain_multi(n) {
-        br.k = k;
-        br.n = nkeys;
-        br.sorted = sorted;
+        if nkeys as usize == br.slots.len() {
+            br.k = k;
+            br.n = nkeys;
+            br.sorted = sorted;
+        }
     } else if let Some((k, nkeys, sorted)) = sole_multi(n) {
         // A THRESHOLD BEHIND A LOCK OR A HASH IS STILL A THRESHOLD. Two
         // conditions: the branch must contain EXACTLY ONE multi node (two
```

- [ ] **Step 4:** re-run — PASS. **Step 5:** the gate. Commit
  `policy_shape: a duplicate-index multi reports no k-of-n in either spelling (1a M-1)`.

---

### Task 2: A public descriptor-text → key route (and F-655)

md-codec has no library route from descriptor text to a key: the walker is
private to `tests/skeleton_key_conformance.rs` (recon §1b). Move it into
`src/descriptor_route.rs` (feature `derive`, which owns `miniscript`),
arm for arm, returning `RouteError` instead of panicking, and add the
multipath entry the evidence needs (`descriptor_from_text` splits `<0;1>` into
its two chains and self-checks both). The conformance test then calls the
library. F-655: the recogniser gains its near-miss test.

**Files:** Create `crates/md-codec/src/descriptor_route.rs`; Modify
`crates/md-codec/src/lib.rs`; Replace `crates/md-codec/tests/skeleton_key_conformance.rs`.

**Interfaces — produces:** `descriptor_from_text(&str) -> Result<Descriptor, RouteError>`,
`descriptor_from_chains(&str, &str) -> Result<Descriptor, RouteError>`,
`skeleton_key_of_text(&str) -> Result<SkeletonKey, RouteError>`.

- [ ] **Step 1: The conformance test calls the library** (fails to compile
  until Step 3). What changes: the private walker (≈550 lines) is gone, the
  floor rises from `checked >= 40` to the corpus's 48, and the doc stops
  saying "46". Mutation (measured, reds `chunks_and_descriptor_yield_the_same_key`):
  make `is_liana_unspendable_key` return `false` — the two kind-1 vectors
  stop keying through the descriptor route.

Replace `crates/md-codec/tests/skeleton_key_conformance.rs`:

```rust
//! The conformance gate -- one key, two routes (coordinator-compat plan 1a
//! Task 5, re-homed by plan 1b).
//!
//! `chunks -> key == descriptor -> key` for every vendored keyed vector
//! (`crates/md-codec/tests/vectors/keyed_*.conformance.json`, 48 of them),
//! proving `SkeletonKey` is a property of the POLICY rather than of the door
//! it was read through.
//!
//! * **The chunk route** decodes the REAL md1 wire chunk set vendored beside
//!   each record (`<name>.phrase.txt`) via [`md_codec::chunk::reassemble`].
//! * **The descriptor route** is now the LIBRARY route,
//!   [`md_codec::descriptor_route::descriptor_from_chains`] -- plan 1b
//!   promoted this file's private walker into md-codec so the verdict
//!   table's generator and `md shape-key` share it. It self-checks every
//!   reconstruction by re-rendering both chains byte-for-byte; see that
//!   module's doc for what the round trip does and does not certify.
//!
//! Both routes feed the ONE [`skeleton`]/[`skeleton_key`] implementation.
//! `tests/coordinator_evidence.rs` runs the same gate over every vendored
//! EVIDENCE row, which this corpus does not contain (recon: 0 of 11 Liana
//! live-gate rows, 1 of 56 matrix shapes).

mod common;

use std::path::{Path, PathBuf};

use md_codec::chunk::reassemble;
use md_codec::descriptor_route::descriptor_from_chains;
use md_codec::encode::Descriptor as MdDescriptor;
use md_codec::skeleton::{skeleton, skeleton_key};
use md_codec::tag::Tag;
use md_codec::to_miniscript_descriptor;
use md_codec::tree::{Body, Node};

// ─────────────────────────────────────────────────────────────────────────
// Vector loading. A glob that matches nothing must FAIL the floor
// assertion below, not silently pass -- so `list_keyed_conformance_files`
// returns an EMPTY Vec on a missing directory (never panics), leaving the
// final `checked >= 48` assertion as the one thing that can catch it.
// ─────────────────────────────────────────────────────────────────────────

fn conformance_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/vectors"))
}

fn list_keyed_conformance_files(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("keyed_") && n.ends_with(".conformance.json"))
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    out.sort();
    out
}

/// One vendored keyed conformance record: the fields this test reads.
struct Rec {
    name: String,
    /// The chunk-set's individual md1 wire strings, header line stripped.
    phrase_chunks: Vec<String>,
    /// `chains.0.descriptor` -- real keys, chain-0 use site.
    descriptor0: String,
    /// `chains.1.descriptor` -- the SAME keys at chain-1, used only to
    /// self-check the reconstructed use-site path (see module doc).
    descriptor1: String,
}

fn load(json_path: &Path) -> Rec {
    let text = std::fs::read_to_string(json_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", json_path.display()));
    let v: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("parse {}: {e}", json_path.display()));
    let name = v["name"]
        .as_str()
        .unwrap_or_else(|| panic!("{}: missing .name", json_path.display()))
        .to_string();
    let descriptor0 = v["chains"]["0"]["descriptor"]
        .as_str()
        .unwrap_or_else(|| panic!("{name}: missing chains.0.descriptor"))
        .to_string();
    let descriptor1 = v["chains"]["1"]["descriptor"]
        .as_str()
        .unwrap_or_else(|| {
            panic!("{name}: missing chains.1.descriptor -- every keyed vector uses <0;1>/*")
        })
        .to_string();

    let phrase_path = json_path.with_file_name(format!("{name}.phrase.txt"));
    let phrase_text = std::fs::read_to_string(&phrase_path)
        .unwrap_or_else(|e| panic!("{name}: read {}: {e}", phrase_path.display()));
    let mut lines = phrase_text.lines();
    let header = lines
        .next()
        .unwrap_or_else(|| panic!("{name}: empty phrase file"));
    assert!(
        header.starts_with("chunk-set-id:"),
        "{name}: expected a chunk-set header (every keyed vector is force_chunked), got {header:?}"
    );
    let phrase_chunks: Vec<String> = lines
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    assert!(
        !phrase_chunks.is_empty(),
        "{name}: no chunk lines in {}",
        phrase_path.display()
    );

    Rec {
        name,
        phrase_chunks,
        descriptor0,
        descriptor1,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// The chunk route: the real wire, via the crate's own reassembler.
// ─────────────────────────────────────────────────────────────────────────

fn decode_chunks(chunks: &[String]) -> MdDescriptor {
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    reassemble(&refs).unwrap_or_else(|e| panic!("reassemble: {e}"))
}

// ─────────────────────────────────────────────────────────────────────────
// The gate.
// ─────────────────────────────────────────────────────────────────────────

/// Every vendored keyed vector, keyed twice: once from the md1 chunk set
/// and once from the descriptor the same record carries. The key must not
/// know which door it came through.
#[test]
fn chunks_and_descriptor_yield_the_same_key() {
    let mut checked = 0usize;
    for path in list_keyed_conformance_files(&conformance_dir()) {
        let rec = load(&path);

        let d_chunks = decode_chunks(&rec.phrase_chunks);
        let from_chunks = skeleton_key(
            &skeleton(&d_chunks)
                .unwrap_or_else(|e| panic!("{}: skeleton (chunk route): {e}", rec.name)),
        );

        let d_desc = descriptor_from_chains(&rec.descriptor0, &rec.descriptor1)
            .unwrap_or_else(|e| panic!("{}: descriptor route: {e}", rec.name));
        let from_desc = skeleton_key(
            &skeleton(&d_desc)
                .unwrap_or_else(|e| panic!("{}: skeleton (descriptor route): {e}", rec.name)),
        );

        assert_eq!(
            from_chunks, from_desc,
            "{}: the key knows its route",
            rec.name
        );
        checked += 1;
    }
    assert!(
        checked >= 48,
        "only {checked} vectors keyed -- the gate is checking almost nothing"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// Review round 1, I-1: direct unit coverage for the eleven walker arms the
// vector corpus alone never reaches. See `md_codec::descriptor_route`'s
// module doc for what the round trip does NOT certify.
// ─────────────────────────────────────────────────────────────────────────

/// A descriptor built from `tests/common/mod.rs`'s `descriptor_with_pubkeys`
/// (real xpubs, no fingerprints) with a fingerprint TLV entry ADDED for every
/// slot. `descriptor_with_pubkeys` alone renders keys with no `[origin]`
/// bracket at all (`to_miniscript.rs::assemble_origin_and_xkey`'s `origin`
/// field is `e.fingerprint.map(...)` -- `None` fingerprint means no bracket,
/// regardless of the divergent origin PATH `descriptor_with_pubkeys` already
/// sets), and `descriptor_from_chains` refuses a key with no bracket outright
/// (`RouteError::KeyWithoutOrigin`). The value is arbitrary -- these fixtures
/// exist to exercise `Tag` mappings, not to pin a specific fingerprint.
fn with_fingerprints(mut d: MdDescriptor) -> MdDescriptor {
    let fp = [0x73, 0xc5, 0xda, 0x0a];
    d.tlv.fingerprints = Some((0..d.n).map(|i| (i, fp)).collect());
    d
}

/// Ten hand-built descriptors covering the eleven arms I-1 named (`Alt` and
/// `AndB` share one fixture, a tap leaf). Each row: the fixture, and a
/// content marker proving the FORWARD rendering actually reaches the
/// fragment it claims to -- the sugar spellings are miniscript's own Display
/// output and match `tests/proptest_to_miniscript.rs`'s own pins for the
/// same fragments (`tv:` = `and_v(_,1)`, `u:` = `or_i(_,0)`, `dv:` =
/// `dupif(verify(_))`, `j:` = `nonzero`, `n:` = `zeronotequal`, `a:` = `alt`).
/// Coverage is then the SAME `descriptor_from_chains` self-check the main gate
/// uses: a wrong `Tag` mapping anywhere in the fixture makes the reconstructed
/// `Descriptor` re-render differently, and `descriptor_from_chains`'s
/// round trip catches it before this function's `marker` check ever would.
#[test]
fn eleven_uncovered_arms_round_trip() {
    use common::{descriptor_with_pubkeys, keyarg, node2, node3, timelock, tr_node, wrap};

    let true_node = || Node {
        tag: Tag::True,
        body: Body::Empty,
    };
    let false_node = || Node {
        tag: Tag::False,
        body: Body::Empty,
    };

    let cases: Vec<(&str, MdDescriptor, &str)> = vec![
        (
            "Terminal::True -- wsh(and_v(v:pk,1)), tv: sugar",
            with_fingerprints(descriptor_with_pubkeys(wrap(
                Tag::Wsh,
                node2(
                    Tag::AndV,
                    wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
                    true_node(),
                ),
            ))),
            "tv:pk(",
        ),
        (
            "Terminal::False -- wsh(or_i(pk,0)), u: sugar",
            with_fingerprints(descriptor_with_pubkeys(wrap(
                Tag::Wsh,
                node2(Tag::OrI, keyarg(Tag::PkK, 0), false_node()),
            ))),
            "u:pk(",
        ),
        (
            "Terminal::OrC -- wsh(and_v(or_c(pk,v:pk),1)), t:or_c( sugar",
            with_fingerprints(descriptor_with_pubkeys(wrap(
                Tag::Wsh,
                node2(
                    Tag::AndV,
                    node2(
                        Tag::OrC,
                        keyarg(Tag::PkK, 0),
                        wrap(Tag::Verify, keyarg(Tag::PkK, 1)),
                    ),
                    true_node(),
                ),
            ))),
            "t:or_c(",
        ),
        (
            "Terminal::DupIf -- wsh(or_i(pk,dv:older)), dv: sugar",
            with_fingerprints(descriptor_with_pubkeys(wrap(
                Tag::Wsh,
                node2(
                    Tag::OrI,
                    keyarg(Tag::PkK, 0),
                    wrap(Tag::DupIf, wrap(Tag::Verify, timelock(Tag::Older, 144))),
                ),
            ))),
            "dv:older(",
        ),
        (
            "Terminal::NonZero -- wsh(j:pk), j: sugar",
            with_fingerprints(descriptor_with_pubkeys(wrap(
                Tag::Wsh,
                wrap(Tag::NonZero, keyarg(Tag::PkK, 0)),
            ))),
            "j:pk(",
        ),
        (
            "Terminal::ZeroNotEqual -- wsh(or_i(pk,n:and_v)), n: sugar",
            with_fingerprints(descriptor_with_pubkeys(wrap(
                Tag::Wsh,
                node2(
                    Tag::OrI,
                    keyarg(Tag::PkK, 0),
                    wrap(
                        Tag::ZeroNotEqual,
                        node2(
                            Tag::AndV,
                            wrap(Tag::Verify, keyarg(Tag::PkK, 1)),
                            timelock(Tag::Older, 144),
                        ),
                    ),
                ),
            ))),
            "n:and_v(",
        ),
        (
            "Terminal::Alt + Terminal::AndB -- tr tap leaf and_b(pk,a:pk_h)",
            with_fingerprints(descriptor_with_pubkeys(tr_node(
                false,
                0,
                Some(node2(
                    Tag::AndB,
                    keyarg(Tag::PkK, 1),
                    wrap(Tag::Alt, keyarg(Tag::PkH, 2)),
                )),
            ))),
            "and_b(pk(",
        ),
        (
            "Terminal::AndOr -- wsh(andor(pk,older(144),pk))",
            with_fingerprints(descriptor_with_pubkeys(wrap(
                Tag::Wsh,
                node3(
                    Tag::AndOr,
                    keyarg(Tag::PkK, 0),
                    timelock(Tag::Older, 144),
                    keyarg(Tag::PkK, 1),
                ),
            ))),
            "andor(pk(",
        ),
        (
            "root MsDescriptor::Pkh -- bare pkh(@0)",
            with_fingerprints(descriptor_with_pubkeys(keyarg(Tag::Pkh, 0))),
            "pkh(",
        ),
        (
            "ShInner::Wpkh -- sh(wpkh(@0))",
            with_fingerprints(descriptor_with_pubkeys(wrap(Tag::Sh, keyarg(Tag::Wpkh, 0)))),
            "sh(wpkh(",
        ),
    ];

    assert_eq!(
        cases.len(),
        10,
        "ten fixtures cover the eleven named arms (Alt+AndB share one, a tap leaf)"
    );

    for (label, d, marker) in &cases {
        let rendered0 = to_miniscript_descriptor(d, 0)
            .unwrap_or_else(|e| panic!("{label}: forward render chain 0: {e}"))
            .to_string();
        assert!(
            rendered0.contains(marker),
            "{label}: fixture does not actually reach the claimed fragment -- rendered {rendered0}"
        );
        let rendered1 = to_miniscript_descriptor(d, 1)
            .unwrap_or_else(|e| panic!("{label}: forward render chain 1: {e}"))
            .to_string();
        // The walker's own round-trip self-check IS the coverage proof: if
        // this call returns Ok, `descriptor_from_chains` walked the
        // arm named by `label` and reproduced it byte-for-byte.
        descriptor_from_chains(&rendered0, &rendered1)
            .unwrap_or_else(|e| panic!("{label}: descriptor route: {e}"));
    }
}
```

- [ ] **Step 2: Register the module.**

Modify `crates/md-codec/src/lib.rs`:

```diff
diff --git a/crates/md-codec/src/lib.rs b/crates/md-codec/src/lib.rs
index eff7428..852bb1e 100644
--- a/crates/md-codec/src/lib.rs
+++ b/crates/md-codec/src/lib.rs
@@ -23,6 +23,8 @@ pub mod codex32;
 pub mod compose;
 pub mod decode;
 pub mod derive;
+#[cfg(feature = "derive")]
+pub mod descriptor_route;
 pub mod encode;
 pub mod error;
 pub mod header;
```

- [ ] **Step 3: The module**, with the F-655 near-miss unit test at its foot
  (a Liana key derived over the same two leaves in the other order must not be
  recognised). Mutation (measured, reds it): compare only `public_key` —
  the NUMS point every candidate shares — instead of the whole xpub; F-655's
  own mutation (`want == xkey` → `true`) reds it too.

Create `crates/md-codec/src/descriptor_route.rs`:

```rust
//! Descriptor TEXT to a decoded md1 [`Descriptor`], and from there to a
//! [`SkeletonKey`] — the library route coordinator-compatibility plan 1b
//! needs, because its evidence is recorded as descriptor text.
//!
//! Promoted from `tests/skeleton_key_conformance.rs`, where plan 1a built it
//! as a test-private walker (`md-codec` cannot dev-depend on `md-cli`). It
//! mirrors `src/to_miniscript.rs`'s forward converter arm for arm, in
//! reverse, and **self-checks every reconstruction** by feeding it back
//! through that forward converter and demanding a byte-identical re-render
//! of BOTH chains, checksum included. A wrong fingerprint, a dropped origin
//! component, a mis-numbered placeholder or a wrong use-site guess is an
//! `Err`, never a wrong key.
//!
//! The use site is always `/<0;1>/*` (`UseSitePath::standard_multipath()`),
//! and the self-check is what makes that assumption safe: a descriptor with
//! any other use site fails the round trip and is refused.
//!
//! What the round trip does NOT certify is unchanged from plan 1a: only an
//! executed walker arm is certified (the conformance test's
//! `eleven_uncovered_arms_round_trip` covers the arms its corpus misses), and
//! the round trip pins only the RENDERED projection of the descriptor.

use std::str::FromStr;
use std::sync::Arc;

use bitcoin::bip32::{ChildNumber, DerivationPath};
use bitcoin::hashes::Hash as _;
use miniscript::descriptor::{
    Descriptor as MsDescriptor, DescriptorPublicKey, ShInner, SinglePubKey, TapTree, Wsh,
};
use miniscript::{Legacy, Miniscript, ScriptContext, Segwitv0, Tap, Terminal, Threshold};

use crate::canonicalize::canonicalize_placeholder_indices;
use crate::encode::Descriptor;
use crate::nums::{NUMS_H_POINT_X_ONLY_HEX, liana_unspendable_xpub};
use crate::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use crate::skeleton::{SkeletonKey, skeleton, skeleton_key};
use crate::tag::Tag;
use crate::tlv::TlvSection;
use crate::tree::{Body, InternalKey, Node};
use crate::use_site_path::UseSitePath;

/// Why descriptor text could not be turned into a [`Descriptor`] or a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteError {
    /// The text is not a BIP-380 descriptor miniscript accepts.
    Parse(String),
    /// [`descriptor_from_text`] needs the multipath `/<0;1>/*` form: a
    /// single-chain descriptor cannot say what its other chain is, and the
    /// key keeps the use site.
    NotMultipath,
    /// A multipath descriptor that does not split into exactly two chains.
    NotTwoChains(usize),
    /// A fragment or key form md1 cannot carry.
    Unsupported(String),
    /// Key `@i` carries no `[fingerprint/path]` origin, so it has no
    /// fingerprint to partition by.
    KeyWithoutOrigin(u8),
    /// Key `@i` is not an extended public key.
    NotAnXpub(u8),
    /// More than 255 keys.
    TooManyKeys,
    /// The reconstruction does not re-render to the input on this chain.
    RoundTrip {
        /// Which chain disagreed.
        chain: u32,
        /// The re-render.
        got: String,
        /// The input.
        want: String,
    },
    /// `canonicalize_placeholder_indices` refused the reconstruction.
    Canonicalize(String),
    /// [`skeleton`] refused the decoded descriptor.
    Skeleton(String),
}

impl std::fmt::Display for RouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteError::Parse(e) => write!(f, "not a descriptor: {e}"),
            RouteError::NotMultipath => write!(
                f,
                "a single-chain descriptor: the key keeps the <0;1> use site, so pass the multipath form"
            ),
            RouteError::NotTwoChains(n) => write!(f, "multipath splits into {n} chains, not 2"),
            RouteError::Unsupported(what) => write!(f, "md1 cannot carry {what}"),
            RouteError::KeyWithoutOrigin(i) => {
                write!(f, "key @{i} carries no [fingerprint/path] origin")
            }
            RouteError::NotAnXpub(i) => write!(f, "key @{i} is not an extended public key"),
            RouteError::TooManyKeys => write!(f, "more than 255 keys"),
            RouteError::RoundTrip { chain, got, want } => write!(
                f,
                "the reconstruction does not round-trip chain {chain}: got {got}, want {want}"
            ),
            RouteError::Canonicalize(e) => write!(f, "canonicalize: {e}"),
            RouteError::Skeleton(e) => write!(f, "skeleton: {e}"),
        }
    }
}

impl std::error::Error for RouteError {}

/// Parse a MULTIPATH (`/<0;1>/*`) descriptor into a canonical md1
/// [`Descriptor`]: split it into its two chains and hand both to
/// [`descriptor_from_chains`].
///
/// # Errors
///
/// Any [`RouteError`]; a single-chain descriptor is [`RouteError::NotMultipath`].
pub fn descriptor_from_text(text: &str) -> Result<Descriptor, RouteError> {
    let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(text.trim())
        .map_err(|e| RouteError::Parse(e.to_string()))?;
    if !parsed.is_multipath() {
        return Err(RouteError::NotMultipath);
    }
    let singles = parsed
        .into_single_descriptors()
        .map_err(|e| RouteError::Parse(e.to_string()))?;
    if singles.len() != 2 {
        return Err(RouteError::NotTwoChains(singles.len()));
    }
    descriptor_from_chains(&singles[0].to_string(), &singles[1].to_string())
}

/// The [`SkeletonKey`] of a multipath descriptor's text — the one function
/// both the verdict table's generator and `md shape-key --descriptor` call.
///
/// # Errors
///
/// Any [`RouteError`].
pub fn skeleton_key_of_text(text: &str) -> Result<SkeletonKey, RouteError> {
    let d = descriptor_from_text(text)?;
    let s = skeleton(&d).map_err(|e| RouteError::Skeleton(e.to_string()))?;
    Ok(skeleton_key(&s))
}

/// Reconstruct the md1 [`Descriptor`] from a descriptor's chain-0 and chain-1
/// texts, self-checking the reconstruction against BOTH before returning,
/// then canonicalizing its placeholder numbering.
///
/// # Errors
///
/// Any [`RouteError`].
pub fn descriptor_from_chains(chain0: &str, chain1: &str) -> Result<Descriptor, RouteError> {
    let desc0 = MsDescriptor::<DescriptorPublicKey>::from_str(chain0)
        .map_err(|e| RouteError::Parse(e.to_string()))?;

    let mut reg = KeyRegistry { keys: Vec::new() };
    let tree = ms_descriptor_to_node(&desc0, &mut reg)?;
    let n = u8::try_from(reg.keys.len()).map_err(|_| RouteError::TooManyKeys)?;

    let mut origin_paths = Vec::with_capacity(reg.keys.len());
    let mut fingerprints = Vec::with_capacity(reg.keys.len());
    let mut pubkeys = Vec::with_capacity(reg.keys.len());
    let mut network = bitcoin::Network::Bitcoin;
    for (i, pk) in reg.keys.iter().enumerate() {
        let idx = u8::try_from(i).map_err(|_| RouteError::TooManyKeys)?;
        let DescriptorPublicKey::XPub(x) = pk else {
            return Err(RouteError::NotAnXpub(idx));
        };
        let (fp, origin_derivation) = x.origin.clone().ok_or(RouteError::KeyWithoutOrigin(idx))?;
        if x.xkey.network != bitcoin::NetworkKind::Main {
            network = bitcoin::Network::Testnet;
        }
        origin_paths.push(derivation_path_to_origin_path(&origin_derivation));
        fingerprints.push((idx, fp.to_bytes()));
        let mut bytes = [0u8; 65];
        bytes[..32].copy_from_slice(x.xkey.chain_code.as_ref());
        bytes[32..].copy_from_slice(&x.xkey.public_key.serialize());
        pubkeys.push((idx, bytes));
    }

    let mut d = Descriptor {
        n,
        path_decl: PathDecl {
            n,
            paths: PathDeclPaths::Divergent(origin_paths),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: Some(fingerprints),
            pubkeys: Some(pubkeys),
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    };

    for (chain, want) in [(0u32, chain0), (1u32, chain1)] {
        let got = crate::to_miniscript::to_miniscript_descriptor_with_network(&d, chain, network)
            .map_err(|e| RouteError::Unsupported(e.to_string()))?
            .to_string();
        if got != want {
            return Err(RouteError::RoundTrip {
                chain,
                got,
                want: want.to_string(),
            });
        }
    }

    canonicalize_placeholder_indices(&mut d)
        .map_err(|e| RouteError::Canonicalize(e.to_string()))?;
    Ok(d)
}

/// Placeholder indices are assigned in walk order;
/// `canonicalize_placeholder_indices` renumbers them afterwards.
struct KeyRegistry {
    keys: Vec<DescriptorPublicKey>,
}

impl KeyRegistry {
    fn register(&mut self, pk: &DescriptorPublicKey) -> Result<u8, RouteError> {
        self.keys.push(pk.clone());
        u8::try_from(self.keys.len() - 1).map_err(|_| RouteError::TooManyKeys)
    }
}

/// True iff `pk` is an origin-less xpub equal to Liana's recipe
/// ([`liana_unspendable_xpub`]) over the tap tree's own leaf keys, in
/// left-to-right leaf and key-occurrence order. FULL byte equality with the
/// recipe, never a structural match (F-449 SPEC §8.10).
fn is_liana_unspendable_key(
    pk: &DescriptorPublicKey,
    tree: Option<&TapTree<DescriptorPublicKey>>,
) -> bool {
    let xkey = match pk {
        DescriptorPublicKey::XPub(x) if x.origin.is_none() => x.xkey,
        DescriptorPublicKey::MultiXPub(x) if x.origin.is_none() => x.xkey,
        _ => return false,
    };
    let Some(t) = tree else { return false };
    let mut leaf_pubkeys: Vec<[u8; 33]> = Vec::new();
    for item in t.leaves() {
        for leaf_pk in item.miniscript().iter_pk() {
            match leaf_pk {
                DescriptorPublicKey::XPub(k) => leaf_pubkeys.push(k.xkey.public_key.serialize()),
                DescriptorPublicKey::MultiXPub(k) => {
                    leaf_pubkeys.push(k.xkey.public_key.serialize())
                }
                DescriptorPublicKey::Single(_) => return false,
            }
        }
    }
    let network = if xkey.network == bitcoin::NetworkKind::Main {
        bitcoin::Network::Bitcoin
    } else {
        bitcoin::Network::Testnet
    };
    liana_unspendable_xpub(&leaf_pubkeys, network) == xkey
}

fn is_nums_key(pk: &DescriptorPublicKey) -> bool {
    match pk {
        DescriptorPublicKey::Single(single) if single.origin.is_none() => {
            matches!(&single.key, SinglePubKey::XOnly(x) if x.to_string() == NUMS_H_POINT_X_ONLY_HEX)
        }
        _ => false,
    }
}

fn wrap1<Ctx: ScriptContext>(
    tag: Tag,
    inner: &Arc<Miniscript<DescriptorPublicKey, Ctx>>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    Ok(Node {
        tag,
        body: Body::Children(vec![terminal_to_node(inner, reg)?]),
    })
}

fn wrap2<Ctx: ScriptContext>(
    tag: Tag,
    l: &Arc<Miniscript<DescriptorPublicKey, Ctx>>,
    r: &Arc<Miniscript<DescriptorPublicKey, Ctx>>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    Ok(Node {
        tag,
        body: Body::Children(vec![terminal_to_node(l, reg)?, terminal_to_node(r, reg)?]),
    })
}

fn multikeys_node<const MAX: usize>(
    tag: Tag,
    thresh: &Threshold<DescriptorPublicKey, MAX>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    let k = u8::try_from(thresh.k()).map_err(|_| RouteError::TooManyKeys)?;
    let indices = thresh
        .data()
        .iter()
        .map(|pk| reg.register(pk))
        .collect::<Result<_, _>>()?;
    Ok(Node {
        tag,
        body: Body::MultiKeys { k, indices },
    })
}

fn leaf(tag: Tag, body: Body) -> Result<Node, RouteError> {
    Ok(Node { tag, body })
}

/// One miniscript node to one md1 `Node`. Mirrors `node_to_miniscript` in
/// `src/to_miniscript.rs`, reversed arm for arm.
fn terminal_to_node<Ctx: ScriptContext>(
    ms: &Miniscript<DescriptorPublicKey, Ctx>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    match &ms.node {
        Terminal::True => leaf(Tag::True, Body::Empty),
        Terminal::False => leaf(Tag::False, Body::Empty),
        // md1 always Check-wraps a key; a bare one is not an md1 shape.
        Terminal::PkK(_) | Terminal::PkH(_) | Terminal::RawPkH(_) => Err(RouteError::Unsupported(
            "a bare pk_k/pk_h/raw_pkh fragment".into(),
        )),
        Terminal::After(lt) => leaf(Tag::After, Body::Timelock(lt.to_consensus_u32())),
        Terminal::Older(lt) => leaf(Tag::Older, Body::Timelock(lt.to_consensus_u32())),
        Terminal::Sha256(h) => leaf(Tag::Sha256, Body::Hash256Body(h.to_byte_array())),
        Terminal::Hash256(h) => leaf(Tag::Hash256, Body::Hash256Body(h.to_byte_array())),
        Terminal::Ripemd160(h) => leaf(Tag::Ripemd160, Body::Hash160Body(h.to_byte_array())),
        Terminal::Hash160(h) => leaf(Tag::Hash160, Body::Hash160Body(h.to_byte_array())),
        Terminal::Alt(inner) => wrap1(Tag::Alt, inner, reg),
        Terminal::Swap(inner) => wrap1(Tag::Swap, inner, reg),
        Terminal::Check(inner) => {
            // `Check(PkK)` / `Check(PkH)` is the bare `pk(...)` / `pkh(...)`
            // fragment on the wire -- the collapse `node_to_miniscript`'s
            // `Tag::Check` arm re-applies forward.
            match &inner.node {
                Terminal::PkK(pk) => {
                    let index = reg.register(pk)?;
                    leaf(Tag::PkK, Body::KeyArg { index })
                }
                Terminal::PkH(pk) => {
                    let index = reg.register(pk)?;
                    leaf(Tag::PkH, Body::KeyArg { index })
                }
                _ => wrap1(Tag::Check, inner, reg),
            }
        }
        Terminal::DupIf(inner) => wrap1(Tag::DupIf, inner, reg),
        Terminal::Verify(inner) => wrap1(Tag::Verify, inner, reg),
        Terminal::NonZero(inner) => wrap1(Tag::NonZero, inner, reg),
        Terminal::ZeroNotEqual(inner) => wrap1(Tag::ZeroNotEqual, inner, reg),
        Terminal::AndV(l, r) => wrap2(Tag::AndV, l, r, reg),
        Terminal::AndB(l, r) => wrap2(Tag::AndB, l, r, reg),
        Terminal::AndOr(a, b, c) => Ok(Node {
            tag: Tag::AndOr,
            body: Body::Children(vec![
                terminal_to_node(a, reg)?,
                terminal_to_node(b, reg)?,
                terminal_to_node(c, reg)?,
            ]),
        }),
        Terminal::OrB(l, r) => wrap2(Tag::OrB, l, r, reg),
        Terminal::OrC(l, r) => wrap2(Tag::OrC, l, r, reg),
        Terminal::OrD(l, r) => wrap2(Tag::OrD, l, r, reg),
        Terminal::OrI(l, r) => wrap2(Tag::OrI, l, r, reg),
        Terminal::Thresh(thresh) => {
            let k = u8::try_from(thresh.k()).map_err(|_| RouteError::TooManyKeys)?;
            let children = thresh
                .data()
                .iter()
                .map(|c| terminal_to_node(c, reg))
                .collect::<Result<_, _>>()?;
            Ok(Node {
                tag: Tag::Thresh,
                body: Body::Variable { k, children },
            })
        }
        Terminal::Multi(thresh) => multikeys_node(Tag::Multi, thresh, reg),
        Terminal::SortedMulti(thresh) => multikeys_node(Tag::SortedMulti, thresh, reg),
        Terminal::MultiA(thresh) => multikeys_node(Tag::MultiA, thresh, reg),
        Terminal::SortedMultiA(thresh) => multikeys_node(Tag::SortedMultiA, thresh, reg),
    }
}

/// `wsh(...)`'s single child. `sortedmulti` is legal here only at the root.
fn wsh_to_node(wsh: &Wsh<DescriptorPublicKey>, reg: &mut KeyRegistry) -> Result<Node, RouteError> {
    if let Terminal::SortedMulti(thresh) = &wsh.as_inner().node {
        return multikeys_node(Tag::SortedMulti, thresh, reg);
    }
    terminal_to_node::<Segwitv0>(wsh.as_inner(), reg)
}

/// `sh(...)`'s single child. Mirrors `sh_inner_to_descriptor`.
fn sh_inner_to_node(
    inner: &ShInner<DescriptorPublicKey>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    match inner {
        ShInner::Wpkh(w) => {
            let index = reg.register(w.as_inner())?;
            leaf(Tag::Wpkh, Body::KeyArg { index })
        }
        ShInner::Wsh(wsh) => Ok(Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![wsh_to_node(wsh, reg)?]),
        }),
        ShInner::Ms(ms) => {
            if let Terminal::SortedMulti(thresh) = &ms.node {
                return multikeys_node(Tag::SortedMulti, thresh, reg);
            }
            terminal_to_node::<Legacy>(ms, reg)
        }
    }
}

/// One tap leaf's root. `sortedmulti_a` is legal only here.
fn tap_leaf_to_node(
    ms: &Miniscript<DescriptorPublicKey, Tap>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    if let Terminal::SortedMultiA(thresh) = &ms.node {
        return multikeys_node(Tag::SortedMultiA, thresh, reg);
    }
    terminal_to_node::<Tap>(ms, reg)
}

/// Rebuild the binary `Tag::TapTree` node from `TapTree::leaves()`'s
/// depth-first `(depth, leaf)` list by combining adjacent equal-depth
/// siblings. A single-leaf tree falls out as the bare leaf node.
fn taptree_to_node(
    tt: &TapTree<DescriptorPublicKey>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    let mut stack: Vec<(u8, Node)> = Vec::new();
    for item in tt.leaves() {
        let mut node = tap_leaf_to_node(item.miniscript(), reg)?;
        let mut depth = item.depth();
        while let Some(&(top_depth, _)) = stack.last() {
            if top_depth != depth {
                break;
            }
            let (_, left) = stack.pop().expect("just peeked");
            node = Node {
                tag: Tag::TapTree,
                body: Body::Children(vec![left, node]),
            };
            depth -= 1;
        }
        stack.push((depth, node));
    }
    match (stack.pop(), stack.is_empty()) {
        (Some((_, root)), true) => Ok(root),
        _ => Err(RouteError::Unsupported(
            "a tap tree that does not converge to one root".into(),
        )),
    }
}

fn ms_descriptor_to_node(
    desc: &MsDescriptor<DescriptorPublicKey>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    match desc {
        MsDescriptor::Wpkh(w) => {
            let index = reg.register(w.as_inner())?;
            leaf(Tag::Wpkh, Body::KeyArg { index })
        }
        MsDescriptor::Pkh(p) => {
            let index = reg.register(p.as_inner())?;
            leaf(Tag::Pkh, Body::KeyArg { index })
        }
        MsDescriptor::Sh(sh) => Ok(Node {
            tag: Tag::Sh,
            body: Body::Children(vec![sh_inner_to_node(sh.as_inner(), reg)?]),
        }),
        MsDescriptor::Wsh(wsh) => Ok(Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![wsh_to_node(wsh, reg)?]),
        }),
        MsDescriptor::Tr(tr) => {
            let internal_key = if is_nums_key(tr.internal_key()) {
                InternalKey::NumsPoint
            } else if is_liana_unspendable_key(tr.internal_key(), tr.tap_tree()) {
                InternalKey::LianaUnspendable
            } else {
                InternalKey::Slot(reg.register(tr.internal_key())?)
            };
            let tree = match tr.tap_tree() {
                Some(tt) => Some(Box::new(taptree_to_node(tt, reg)?)),
                None => None,
            };
            Ok(Node {
                tag: Tag::Tr,
                body: Body::Tr { internal_key, tree },
            })
        }
        MsDescriptor::Bare(_) => Err(RouteError::Unsupported("a bare top-level script".into())),
    }
}

fn derivation_path_to_origin_path(p: &DerivationPath) -> OriginPath {
    let components = p
        .as_ref()
        .iter()
        .map(|c| match c {
            ChildNumber::Normal { index } => PathComponent {
                hardened: false,
                value: *index,
            },
            ChildNumber::Hardened { index } => PathComponent {
                hardened: true,
                value: *index,
            },
        })
        .collect();
    OriginPath { components }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F-655: the recogniser's NEAR MISS. Liana's recipe over the SAME leaves
    /// in a DIFFERENT order is a different chain code, so it must not be
    /// recognised. Mutation: compare only `public_key` (the NUMS point every
    /// candidate shares) instead of the whole xpub -> this test reds.
    #[test]
    fn a_liana_key_derived_over_permuted_leaves_is_not_recognised() {
        let a = "xpub6DXuQW1Q2JpZyweiMewTZuMPvjG8hKhV2qoF6wL9VFxsMBExtbfqAAoR4oMG4GyxFzVdfas1v2eAdfLxyjc4Ceo5B6w6zTpf7F2BuXCJ52i";
        let b = "xpub6DXuQW1Q2JpZyteDRGW1pD34uhumfnZJfTsmjDkgd4xcq3L5XX2KUE1n4rmvcDT3RmdchfhbD9DkvSyVhUMBjUYi691iFszgKtf4Bfqe2nL";
        let xa: bitcoin::bip32::Xpub = a.parse().unwrap();
        let xb: bitcoin::bip32::Xpub = b.parse().unwrap();
        let ab = liana_unspendable_xpub(
            &[xa.public_key.serialize(), xb.public_key.serialize()],
            bitcoin::Network::Bitcoin,
        );
        let ba = liana_unspendable_xpub(
            &[xb.public_key.serialize(), xa.public_key.serialize()],
            bitcoin::Network::Bitcoin,
        );
        let leaves =
            format!("{{pk([73c5da0a/48'/0'/0'/3']{a}/0/*),pk([3f635a63/48'/0'/0'/3']{b}/0/*)}}");
        let parse = |ik: &bitcoin::bip32::Xpub| {
            MsDescriptor::<DescriptorPublicKey>::from_str(&format!("tr({ik}/0/*,{leaves})"))
                .unwrap()
        };
        let right = parse(&ab);
        let wrong = parse(&ba);
        let (MsDescriptor::Tr(r), MsDescriptor::Tr(w)) = (&right, &wrong) else {
            unreachable!()
        };
        assert!(is_liana_unspendable_key(r.internal_key(), r.tap_tree()));
        assert!(!is_liana_unspendable_key(w.internal_key(), w.tap_tree()));
    }
}
```

- [ ] **Step 4:** `cargo nextest run -p md-codec --lib --test skeleton_key_conformance` — PASS.
  **Step 5:** the gate. Commit `descriptor_route: the conformance walker becomes a public library route (F-655)`.

---

### Task 3: `Skeleton::leaf_keys_ascending` — R-1's key-material fact

Liana hashes the leaf pubkeys in wire order with duplicates
(`nums.rs:50`); libnunchuk sorts and dedups them (`GetUnspendableXpub`,
`src/descriptor.cpp:689-712`, `:700-701`). The two agree exactly when the
wire-order sequence is strictly ascending. The field is NOT in
`skeleton_key` (R-1).

**Files:** Modify `crates/md-codec/src/skeleton.rs`; Test
`crates/md-codec/tests/skeleton_key.rs`.

- [ ] **Step 1: The failing test.** Mutation (measured, reds it): `pk <= p` →
  `pk < p` (a duplicated leaf key then reads ascending).

Modify `crates/md-codec/tests/skeleton_key.rs`:

```diff
diff --git a/crates/md-codec/tests/skeleton_key.rs b/crates/md-codec/tests/skeleton_key.rs
index 9305a8d..f0009e9 100644
--- a/crates/md-codec/tests/skeleton_key.rs
+++ b/crates/md-codec/tests/skeleton_key.rs
@@ -348,3 +348,35 @@ fn golden_key_taproot_with_key_path_branch() {
         "tr(@0/<0;1>/*,{pk(@1/<0;1>/*),pk(@2/<0;1>/*)})\u{1f}[[][][]][]\u{1f}Xpub"
     );
 }
+
+/// `leaf_keys_ascending` is STRICT: Nunchuk deduplicates, Liana does not, so
+/// a repeated leaf key breaks agreement. Mutation: `pk <= p` -> `pk < p` in
+/// `skeleton.rs` -> the duplicate case reads `Some(true)` and this reds.
+#[test]
+fn leaf_keys_ascending_is_strict_and_order_sensitive() {
+    use common::{keyarg, taptree2, test_xpubs, tr_liana_unspendable_two_leaves_with_pubkeys};
+    use md_codec::tag::Tag;
+    use md_codec::tree::{Body, InternalKey, Node};
+
+    let a = test_xpubs()[0];
+    let b = test_xpubs()[1];
+    let mut ab = tr_liana_unspendable_two_leaves_with_pubkeys();
+    ab.tlv.pubkeys = Some(vec![(0, a), (1, b)]);
+    let mut ba = ab.clone();
+    ba.tlv.pubkeys = Some(vec![(0, b), (1, a)]);
+    let got = [ab, ba].map(|d| skeleton(&d).unwrap().leaf_keys_ascending);
+    assert!(
+        got == [Some(true), Some(false)] || got == [Some(false), Some(true)],
+        "exactly one order ascends: {got:?}"
+    );
+
+    let mut dup = tr_liana_unspendable_two_leaves_with_pubkeys();
+    dup.tree = Node {
+        tag: Tag::Tr,
+        body: Body::Tr {
+            internal_key: InternalKey::LianaUnspendable,
+            tree: Some(Box::new(taptree2(keyarg(Tag::PkK, 0), keyarg(Tag::PkK, 0)))),
+        },
+    };
+    assert_eq!(skeleton(&dup).unwrap().leaf_keys_ascending, Some(false));
+}
```

- [ ] **Step 2:** run — does not compile (no field). **Step 3:**

Modify `crates/md-codec/src/skeleton.rs`:

```diff
diff --git a/crates/md-codec/src/skeleton.rs b/crates/md-codec/src/skeleton.rs
index 3c3eb5b..0783680 100644
--- a/crates/md-codec/src/skeleton.rs
+++ b/crates/md-codec/src/skeleton.rs
@@ -82,6 +82,19 @@ pub struct Skeleton {
     /// verdict, it does not make a template-only card a different policy
     /// from the same card seated.
     pub keys_present: bool,
+    /// Whether a `tr` policy's tap-leaf key OCCURRENCES, in wire order
+    /// (left to right, the order Liana's recipe hashes them), have
+    /// compressed public keys in STRICTLY ascending byte order — sorted and
+    /// unique. `None` when the root is not `tr` or no keys are present.
+    ///
+    /// Key material, so NOT in [`SkeletonKey`] (ruling R-1: the key stays
+    /// coordinator-independent). It exists for one rule: libnunchuk
+    /// re-derives a Liana-style unspendable key over its signers' pubkeys
+    /// SORTED and DEDUPLICATED (`GetUnspendableXpub`, libnunchuk `a7cfb49`
+    /// `src/descriptor.cpp:689-712`, `std::sort` + `std::unique` at
+    /// `:700-701`), while Liana hashes them in wire order with duplicates;
+    /// the two chain codes agree exactly when this is `Some(true)`.
+    pub leaf_keys_ascending: Option<bool>,
 }
 
 /// Why [`skeleton`] refused to build a [`Skeleton`].
@@ -211,6 +224,7 @@ pub fn skeleton(d: &Descriptor) -> Result<Skeleton, SkeletonError> {
     // and, per the module doc, it is also simply WRONG: it cannot tell a
     // template-only card from a partial-decoded one.
     let keys_present = d.is_wallet_policy();
+    let leaf_keys_ascending = leaf_keys_ascending(d);
 
     Ok(Skeleton {
         root,
@@ -220,9 +234,53 @@ pub fn skeleton(d: &Descriptor) -> Result<Skeleton, SkeletonError> {
         fp_partition,
         key_partition,
         keys_present,
+        leaf_keys_ascending,
     })
 }
 
+/// See [`Skeleton::leaf_keys_ascending`]. Walks the tap tree in wire order —
+/// the same pre-order, left-first order `to_miniscript`'s
+/// `collect_leaf_pubkeys` gets from `TapTree::leaves()` + `iter_pk()` — and
+/// reads each occurrence's pubkey from the `Pubkeys` TLV's 65-byte
+/// `chain code (32) || compressed key (33)` entry.
+fn leaf_keys_ascending(d: &Descriptor) -> Option<bool> {
+    let Body::Tr { tree, .. } = &d.tree.body else {
+        return None;
+    };
+    if !d.is_wallet_policy() {
+        return None;
+    }
+    let pubkeys = d.tlv.pubkeys.as_ref()?;
+    let mut occurrences = Vec::new();
+    if let Some(t) = tree {
+        collect_key_occurrences(t, &mut occurrences);
+    }
+    let mut prev: Option<[u8; 33]> = None;
+    for index in occurrences {
+        let (_, entry) = pubkeys.iter().find(|(i, _)| *i == index)?;
+        let mut pk = [0u8; 33];
+        pk.copy_from_slice(&entry[32..]);
+        if prev.is_some_and(|p| pk <= p) {
+            return Some(false);
+        }
+        prev = Some(pk);
+    }
+    Some(true)
+}
+
+fn collect_key_occurrences(n: &Node, out: &mut Vec<u8>) {
+    match &n.body {
+        Body::KeyArg { index } => out.push(*index),
+        Body::MultiKeys { indices, .. } => out.extend_from_slice(indices),
+        Body::Children(children) | Body::Variable { children, .. } => {
+            for c in children {
+                collect_key_occurrences(c, out);
+            }
+        }
+        _ => {}
+    }
+}
+
 /// Render one group of slots, already ascending
 /// ([`policy_shape`]'s `group_ascending` guarantee) — `[s0,s1,...]`.
 fn render_slots(slots: &[u8]) -> String {
```

- [ ] **Step 4:** PASS. **Step 5:** the gate. Commit
  `skeleton: leaf_keys_ascending, outside the key (R-1)`.

---

### Task 4: Verdict types, the registry, and runtime verdicts

The four `Verdict` kinds, `ReadSet`, `Clause`/`FormRefusal`/`RuleSet`/
`Coordinator`, and `verdicts(&Skeleton, Option<Form>)`, which merges
consecutive verified versions with one answer into a `Span`. The table is an
empty stub until Task 6 generates it.

**The rules, and where each is established** (registry.rs carries the same
cites):

| coordinator | class (order of refusal) | reads | source |
| --- | --- | --- | --- |
| Liana 8.0-15.0 | a sortedmulti_a leaf | S | parse error in evidence (plain-2of3-tr), both versions |
| | a path with no key | S | "All spend paths must require a signature" (v15.0, F-633) |
| | one signer twice in a path | K | X11's message |
| | a key used twice | K | design §1A `key_partition` (DuplicateKey) |
| | legacy wrapper … a second unlocked multi-key path | S | the fork's `composerLianaOutsideModelClass` (`gui/composer_consent.go:396`, `2c9eed3`), ported with the double count removed, classes 5/6 swapped and class 9 narrowed (§2) |
| Nunchuk 2.1.1 | a path with no key (`wsh` only) | S | libnunchuk `a7cfb49` `descriptor.cpp:573`, `nunchukutils.cpp:1276`, `contrib/bitcoin` `miniscript.h:1617` |
| | Liana's key is not the one Nunchuk derives | K | R-1; `descriptor.cpp:689-712`, `:646` |
| Core 24.2-25.2 | `<0;1>` spelling (form); miniscript under tr; a path with no key | S | `src/script/descriptor.cpp` v24.2:1507/1525, v25.2:1510/1528; core-boundary |
| Core 26.0-28.4 | `<0;1>` spelling (form); a path with no key | S | v26.0:1290/1800, v27.2:1800; core-boundary |
| Core 29.4-31.1 | a path with no key | S | v29.2:2102, v30.0:2483 |

(`S` = `ReadSet::STRUCTURE`, `K` = `KEY_IDENTITY`.) Core line numbers were
read at the local tags of `/scratch/code/bitcoin`; 28.4, 29.4, 30.3 and 31.1
have no local tag, so their span membership rests on the core-boundary
measurement, not on a source read.

**Files:** Create `crates/md-codec/src/coordinator/mod.rs`,
`crates/md-codec/src/coordinator/registry.rs`,
`crates/md-codec/src/coordinator/table.rs` (stub); Modify
`crates/md-codec/src/lib.rs`; Create `crates/md-codec/tests/coordinator.rs`.

- [ ] **Step 1: The failing test.** Mutation (measured, reds it): shrink
  Core's `29.4-31.1` span to `29.4-30.3`.

Create `crates/md-codec/tests/coordinator.rs`:

```rust
//! Coordinator-compatibility plan 1b: verdicts, the rule/evidence build, and
//! the conformance gate over EVIDENCE rows.
//!
//! Every test names the mutation that reds it.

use md_codec::coordinator::REGISTRY;

/// Every verified version has exactly one rule set. Mutation: shrink Core's
/// 29.4-31.1 span to 29.4-30.3 -> 31.1 has none and this reds.
#[test]
fn every_verified_version_has_exactly_one_rule_set() {
    for c in REGISTRY {
        for v in c.verified {
            let at = c.position(v).unwrap();
            let n = c
                .rules
                .iter()
                .filter(|r| {
                    let (a, b) = (c.position(r.span.since.0), c.position(r.span.until.0));
                    matches!((a, b), (Some(a), Some(b)) if a <= at && at <= b)
                })
                .count();
            assert_eq!(n, 1, "{} {v}: {n} rule sets", c.name);
        }
    }
}
```

- [ ] **Step 2:** run — does not compile. **Step 3:** register, then the
  three files.

Modify `crates/md-codec/src/lib.rs`:

```diff
diff --git a/crates/md-codec/src/lib.rs b/crates/md-codec/src/lib.rs
index 852bb1e..696e930 100644
--- a/crates/md-codec/src/lib.rs
+++ b/crates/md-codec/src/lib.rs
@@ -21,6 +21,7 @@ pub mod canonicalize;
 pub mod chunk;
 pub mod codex32;
 pub mod compose;
+pub mod coordinator;
 pub mod decode;
 pub mod derive;
 #[cfg(feature = "derive")]
```

Create `crates/md-codec/src/coordinator/mod.rs`:

```rust
//! Which wallet coordinators will import a policy — coordinator-compatibility
//! plan 1b (`design/DESIGN_coordinator_compatibility.md` §1-§3 in
//! `mnemonic-engrave`).
//!
//! Three sources, one answer per coordinator per VERIFIED version:
//!
//! - **Rules** ([`RuleSet`]): refusals derived from the coordinator's source,
//!   in its own order of refusal, each citing where it is established. A
//!   rule may only ever REFUSE (ruling 3).
//! - **Evidence** (the generated table, [`cells`]): what a harness printed. The only
//!   source of a positive. Built from vendored evidence rows by `cargo xtask
//!   verdicts`, which fails on any rule/evidence disagreement it cannot
//!   resolve (the `build` module, D1-D5).
//! - **Silence**: [`Verdict::Unproven`], for everything else.
//!
//! A template (no keys) can be refused, never claimed to import (design §1
//! (a2)): every clause that reads key identity is skipped and the verdict is
//! `Unproven { KeysAbsent }`.

mod registry;
#[rustfmt::skip]
mod table;

pub use registry::REGISTRY;

use crate::policy_shape::{Branch, KeyPathKind, LockKind, RootKind};
use crate::skeleton::{Skeleton, skeleton_key};

/// A coordinator's stable identifier (`"liana"`, `"nunchuk"`, `"core"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CoordinatorId(pub &'static str);

/// A coordinator version as its APPLICATION displays it (design §3). Opaque:
/// ordered only by its position in [`Coordinator::verified`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Version(pub &'static str);

/// A closed run of VERIFIED versions (design §1 (e)). Both ends verified;
/// never open-ended (design §3 mechanism 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// First verified version of the run.
    pub since: Version,
    /// Last verified version of the run.
    pub until: Version,
}

/// Which spelling of a descriptor a measurement was taken on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Form {
    /// `/<0;1>/*` — `md descriptor`'s default.
    Multipath,
    /// `/0/*` — `md descriptor --chain 0`.
    Chain0,
    /// `/1/*` — `md descriptor --chain 1`.
    Chain1,
}

impl Form {
    /// The spelling used in notices and in the vendored evidence.
    pub fn as_str(self) -> &'static str {
        match self {
            Form::Multipath => "multipath",
            Form::Chain0 => "chain0",
            Form::Chain1 => "chain1",
        }
    }
}

/// Which tool rendered the descriptor a measurement was taken on (design
/// §1 (d)): provenance on every MEASURED verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RendererId {
    /// The rendering tool.
    pub tool: &'static str,
    /// Its version.
    pub version: &'static str,
    /// The spelling.
    pub form: Form,
}

/// ISO 8601 date a measurement was recorded. Printed for a human; never
/// compared to a clock (design §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasuredAt(pub &'static str);

/// A refusal class: the coordinator's own order-of-refusal name, plus the
/// source that establishes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reason {
    /// The class name, e.g. `"no locked path"`.
    pub class: &'static str,
    /// Where it is established.
    pub cite: &'static str,
}

/// The coordinator's OWN reading of an imported policy, as its harness
/// recorded it — never hand-authored (design §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Description {
    /// The wallet kind the coordinator named, e.g. `"MULTI_SIG"`, `"Liana"`.
    pub wallet_kind: &'static str,
    /// The `k`-of-`n` the coordinator read, where it reads one.
    pub threshold: Option<(u8, u8)>,
}

/// Why a verdict is silent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnprovenReason {
    /// No rule refuses and no measurement covers this policy.
    NoEvidence,
    /// A template: no key identity, so no positive, and no key-reading rule.
    KeysAbsent,
    /// A version outside every verified span.
    OutsideEveryVerifiedSpan,
}

/// One coordinator's answer over one span of verified versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Refused. `renderer: None` = rule-derived (spelling-independent);
    /// `Some` = measured, with its date.
    Refuses {
        /// Why.
        reason: Reason,
        /// Over which verified versions.
        span: Span,
        /// `Some` for a measured refusal.
        renderer: Option<RendererId>,
        /// `Some` for a measured refusal.
        measured_at: Option<MeasuredAt>,
    },
    /// Imported, but read as a different wallet (design §1 (c)).
    ImportsAltered {
        /// The coordinator's reading.
        as_read: Description,
        /// Over which verified versions.
        span: Span,
        /// Provenance.
        renderer: RendererId,
        /// Provenance.
        measured_at: MeasuredAt,
    },
    /// Imported as built. Measured only.
    Imports {
        /// Over which verified versions.
        span: Span,
        /// Provenance.
        renderer: RendererId,
        /// Provenance.
        measured_at: MeasuredAt,
    },
    /// Silent.
    Unproven {
        /// Why.
        reason: UnprovenReason,
        /// Over which verified versions, when known.
        span: Option<Span>,
    },
}

/// Which `Skeleton` fields a clause reads (design §1, `ReadSet`). Declared,
/// because "does this rule read key identity" cannot be asked of an opaque
/// `fn` pointer, and design §1 (a2) turns on exactly that question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadSet(u8);

impl ReadSet {
    /// Root, template, branch structure.
    pub const STRUCTURE: ReadSet = ReadSet(1);
    /// The partitions, or key material such as `leaf_keys_ascending`.
    pub const KEY_IDENTITY: ReadSet = ReadSet(2);
    /// Lock values.
    pub const LOCK_VALUES: ReadSet = ReadSet(4);
    /// Hashlock digests.
    pub const DIGESTS: ReadSet = ReadSet(8);

    /// The union of two sets.
    pub const fn with(self, other: ReadSet) -> ReadSet {
        ReadSet(self.0 | other.0)
    }

    /// Whether `other` is a subset of `self`.
    pub const fn contains(self, other: ReadSet) -> bool {
        self.0 & other.0 == other.0
    }
}

/// One refusal clause of a [`RuleSet`].
#[derive(Debug, Clone, Copy)]
pub struct Clause {
    /// The class it names, and where that is established.
    pub reason: Reason,
    /// What it reads.
    pub reads: ReadSet,
    /// Whether it refuses this policy.
    pub fires: fn(&Skeleton) -> bool,
}

/// A refusal that depends on the SPELLING, not the policy.
#[derive(Debug, Clone, Copy)]
pub struct FormRefusal {
    /// The refused spelling.
    pub form: Form,
    /// Why.
    pub reason: Reason,
}

/// A coordinator's rules over one span of verified versions.
#[derive(Debug, Clone, Copy)]
pub struct RuleSet {
    /// Where these rules hold.
    pub span: Span,
    /// The versions whose SOURCE was read to write them.
    pub source_verified_at: &'static [&'static str],
    /// In the coordinator's own order of refusal; the first that fires names
    /// the reason.
    pub clauses: &'static [Clause],
    /// Spellings refused whatever the policy.
    pub form_refusals: &'static [FormRefusal],
    /// Whether a measured refusal no clause explains is stored as a verdict
    /// (true for a coordinator whose refusals are renderer-dependent,
    /// design §1) or is a D2 build failure.
    pub measured_refusals_admitted: bool,
}

/// One wallet coordinator.
#[derive(Debug, Clone, Copy)]
pub struct Coordinator {
    /// Stable id.
    pub id: CoordinatorId,
    /// Display name.
    pub name: &'static str,
    /// Every version any measurement or source reading verified, oldest
    /// first. The ONLY versions a verdict may name.
    pub verified: &'static [&'static str],
    /// Rule sets; together they must cover every verified version.
    pub rules: &'static [RuleSet],
    /// Whether the coordinator's recorded reading matches the policy as
    /// built. `false` makes a measured import `ImportsAltered`.
    pub reads_as_built: fn(&Skeleton, &Description) -> bool,
    /// The refusal class a coordinator's own message names, when it names
    /// one — the D3 check. `None` for a generic message.
    pub class_of_message: fn(&str) -> Option<&'static str>,
}

impl Coordinator {
    /// Position of `v` in [`Coordinator::verified`].
    pub fn position(&self, v: &str) -> Option<usize> {
        self.verified.iter().position(|x| *x == v)
    }

    /// The rule set whose span holds verified version `v`.
    pub fn rule_set_at(&self, v: &str) -> Option<&RuleSet> {
        let at = self.position(v)?;
        self.rules.iter().find(|r| {
            matches!(
                (self.position(r.span.since.0), self.position(r.span.until.0)),
                (Some(a), Some(b)) if a <= at && at <= b
            )
        })
    }
}

/// One measured cell of the generated table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    /// The [`crate::skeleton::SkeletonKey`] string.
    pub key: &'static str,
    /// [`CoordinatorId`]'s string.
    pub coordinator: &'static str,
    /// A verified version.
    pub version: &'static str,
    /// The spelling measured.
    pub form: Form,
    /// What the harness recorded.
    pub outcome: CellOutcome,
    /// Provenance.
    pub renderer_tool: &'static str,
    /// Provenance.
    pub renderer_version: &'static str,
    /// Provenance.
    pub measured_at: &'static str,
    /// The evidence row it came from.
    pub source: &'static str,
}

/// A measured outcome, as stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellOutcome {
    /// Imported as built.
    Imported,
    /// Imported, read as something else.
    ImportedAltered(Description),
    /// Refused, with the coordinator's message.
    Refused(&'static str),
}

/// The measured cells, generated by the `build` module.
pub fn cells() -> &'static [Cell] {
    table::CELLS
}

/// A verdict about one coordinator.
#[derive(Debug, Clone, Copy)]
pub struct CoordinatorVerdict {
    /// Which coordinator.
    pub coordinator: &'static Coordinator,
    /// Its answer over one span.
    pub verdict: Verdict,
}

/// The per-version answer before runs are merged into spans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum At {
    Refuses(Reason, Option<(RendererId, MeasuredAt)>),
    Altered(Description, RendererId, MeasuredAt),
    Imports(RendererId, MeasuredAt),
    Unproven(UnprovenReason),
}

/// The first clause of `rs` that fires on `s`. With no keys present, a
/// clause that reads key identity is skipped: it cannot be evaluated, and a
/// structure-only refusal is still sound (design §1 (a2)).
pub fn first_refusal(rs: &RuleSet, s: &Skeleton) -> Option<Reason> {
    rs.clauses
        .iter()
        .filter(|c| s.keys_present || !c.reads.contains(ReadSet::KEY_IDENTITY))
        .find(|c| (c.fires)(s))
        .map(|c| c.reason)
}

fn at_version(c: &Coordinator, v: &'static str, s: &Skeleton, key: &str, form: Option<Form>) -> At {
    let Some(rs) = c.rule_set_at(v) else {
        return At::Unproven(UnprovenReason::OutsideEveryVerifiedSpan);
    };
    // The spelling first: a coordinator parses key expressions before it
    // reads the script (the order `build` measured on Core 24.2-25.2).
    if let Some(fr) = form.and_then(|f| rs.form_refusals.iter().find(|fr| fr.form == f)) {
        return At::Refuses(fr.reason, None);
    }
    if let Some(reason) = first_refusal(rs, s) {
        return At::Refuses(reason, None);
    }
    if !s.keys_present {
        return At::Unproven(UnprovenReason::KeysAbsent);
    }
    let Some(form) = form else {
        return At::Unproven(UnprovenReason::NoEvidence);
    };
    let Some(cell) = table::CELLS
        .iter()
        .find(|x| x.key == key && x.coordinator == c.id.0 && x.version == v && x.form == form)
    else {
        return At::Unproven(UnprovenReason::NoEvidence);
    };
    let renderer = RendererId {
        tool: cell.renderer_tool,
        version: cell.renderer_version,
        form: cell.form,
    };
    let at = MeasuredAt(cell.measured_at);
    match cell.outcome {
        CellOutcome::Imported => At::Imports(renderer, at),
        CellOutcome::ImportedAltered(d) => At::Altered(d, renderer, at),
        CellOutcome::Refused(msg) => At::Refuses(
            Reason {
                class: "measured",
                cite: msg,
            },
            Some((renderer, at)),
        ),
    }
}

fn verdict_of(at: At, span: Span) -> Verdict {
    match at {
        At::Refuses(reason, measured) => Verdict::Refuses {
            reason,
            span,
            renderer: measured.map(|m| m.0),
            measured_at: measured.map(|m| m.1),
        },
        At::Altered(as_read, renderer, measured_at) => Verdict::ImportsAltered {
            as_read,
            span,
            renderer,
            measured_at,
        },
        At::Imports(renderer, measured_at) => Verdict::Imports {
            span,
            renderer,
            measured_at,
        },
        At::Unproven(reason) => Verdict::Unproven {
            reason,
            span: Some(span),
        },
    }
}

/// Every coordinator's verdict for `s`, one entry per RUN of consecutive
/// verified versions with the same answer. `form` is the spelling the
/// caller is about to print; `None` for a template, which has none.
pub fn verdicts(s: &Skeleton, form: Option<Form>) -> Vec<CoordinatorVerdict> {
    let key = skeleton_key(s);
    let mut out = Vec::new();
    for c in REGISTRY {
        let mut run: Option<(At, &'static str, &'static str)> = None;
        for v in c.verified {
            let at = at_version(c, v, s, key.as_str(), form);
            run = match run {
                Some((prev, since, _)) if prev == at => Some((prev, since, v)),
                Some((prev, since, until)) => {
                    out.push(CoordinatorVerdict {
                        coordinator: c,
                        verdict: verdict_of(
                            prev,
                            Span {
                                since: Version(since),
                                until: Version(until),
                            },
                        ),
                    });
                    Some((at, v, v))
                }
                None => Some((at, v, v)),
            };
        }
        if let Some((prev, since, until)) = run {
            out.push(CoordinatorVerdict {
                coordinator: c,
                verdict: verdict_of(
                    prev,
                    Span {
                        since: Version(since),
                        until: Version(until),
                    },
                ),
            });
        }
    }
    out
}

/// Ruling 2's "none" case: every coordinator refuses at every verified
/// version. `Unproven` is not a refusal, so a template whose positives are
/// merely unmeasurable is never "none".
pub fn none_imports(vs: &[CoordinatorVerdict]) -> bool {
    !vs.is_empty()
        && vs
            .iter()
            .all(|v| matches!(v.verdict, Verdict::Refuses { .. }))
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.since == self.until {
            write!(f, "{}", self.since.0)
        } else {
            write!(f, "{}-{}", self.since.0, self.until.0)
        }
    }
}

/// One line per verdict, version-qualified and dated where measured
/// (design §3 mechanism 1). Never "Liana imports"; always "Liana 15.0
/// imports (…, 2026-09-20)".
pub fn describe(v: &CoordinatorVerdict) -> String {
    let name = v.coordinator.name;
    match v.verdict {
        Verdict::Refuses {
            reason,
            span,
            renderer: None,
            ..
        } => format!("{name} {span}: refuses ({})", reason.class),
        Verdict::Refuses {
            reason,
            span,
            renderer: Some(r),
            measured_at,
        } => format!(
            "{name} {span}: refused the {} form when measured ({}, {} {}, {}): {}",
            r.form.as_str(),
            measured_at.map_or("undated", |m| m.0),
            r.tool,
            r.version,
            v.coordinator.id.0,
            reason.cite
        ),
        Verdict::ImportsAltered {
            as_read,
            span,
            renderer,
            measured_at,
        } => {
            let t = as_read
                .threshold
                .map_or(String::new(), |(k, n)| format!(" {k}-of-{n}"));
            format!(
                "{name} {span}: imports the {} form, but reads it as {}{t} -- not the wallet as built ({})",
                renderer.form.as_str(),
                as_read.wallet_kind,
                measured_at.0
            )
        }
        Verdict::Imports {
            span,
            renderer,
            measured_at,
        } => format!(
            "{name} {span}: imports the {} form (measured {})",
            renderer.form.as_str(),
            measured_at.0
        ),
        Verdict::Unproven { reason, span } => {
            let span = span.map_or(String::new(), |s| format!(" {s}"));
            let why = match reason {
                UnprovenReason::NoEvidence => "not measured for this policy",
                UnprovenReason::KeysAbsent => "a template has no keys, so no import can be claimed",
                UnprovenReason::OutsideEveryVerifiedSpan => "no verified version",
            };
            format!("{name}{span}: unproven ({why})")
        }
    }
}

// ---------------------------------------------------------------------------
// Predicates shared by the rule clauses in `registry`. Each reads only what
// its clause's `ReadSet` declares.
// ---------------------------------------------------------------------------

fn unlocked(b: &Branch) -> bool {
    b.locks.is_empty()
}

/// A spend path with no key that a lock or hash alone satisfies. A `0`
/// branch (`or_i(X,0)`) has no key and no lock and is NOT one: it is
/// unsatisfiable. A `1` branch is missed, which errs toward silence.
fn keyless_gated(b: &Branch) -> bool {
    b.slots.is_empty() && (!b.locks.is_empty() || !b.hashlocks.is_empty())
}

fn any_lock(s: &Skeleton, kinds: &[LockKind]) -> bool {
    s.shape
        .branches
        .iter()
        .any(|b| b.locks.iter().any(|l| kinds.contains(&l.kind)))
}

fn is_tr(s: &Skeleton) -> bool {
    s.root == RootKind::Tr
}

fn is_nums(s: &Skeleton) -> bool {
    s.shape.key_path == KeyPathKind::Nums
}
```

Create `crates/md-codec/src/coordinator/registry.rs`:

```rust
//! The coordinators, their verified versions, and their rules.
//!
//! **Adding a version** is a string in `verified` plus, if the rules change
//! there, a new [`RuleSet`]. **Adding a coordinator** is a new entry. Neither
//! touches [`super::verdicts`] (design §1).
//!
//! Every clause is a REFUSAL. A clause may under-approximate — miss a
//! refusal, which leaves the verdict silent or to evidence — but must never
//! refuse what the coordinator imports: that is a D1 build failure against
//! the evidence (`super::build`).

use super::{
    Clause, Coordinator, CoordinatorId, Description, Form, FormRefusal, ReadSet, Reason, RuleSet,
    Span, Version, any_lock, is_nums, is_tr, keyless_gated, unlocked,
};
use crate::policy_shape::{KeyPathKind, LockKind, RootKind};
use crate::skeleton::Skeleton;

const S: ReadSet = ReadSet::STRUCTURE;
const K: ReadSet = ReadSet::KEY_IDENTITY;

const fn span(since: &'static str, until: &'static str) -> Span {
    Span {
        since: Version(since),
        until: Version(until),
    }
}

/// Every coordinator the registry knows, in the order notices print them.
pub static REGISTRY: &[Coordinator] = &[LIANA, NUNCHUK, CORE];

// ---------------------------------------------------------------------------
// Liana. Classes in Liana's own order of refusal: miniscript PARSE errors
// first (they fire before Liana sees a policy), then key checks, then the
// policy-model classes of the fork's `composerLianaOutsideModelClass`
// (fork `2c9eed3`, `gui/composer_consent.go:396`), which is ported here and
// becomes the convergence port.
// ---------------------------------------------------------------------------

const LIANA_CLAUSES: &[Clause] = &[
    Clause {
        reason: Reason {
            class: "a sortedmulti_a leaf",
            cite: "Liana v8.0 and v15.0 parse: \"unexpected «sortedmulti_a(4 args) while parsing Miniscript»\" (evidence plain-2of3-tr)",
        },
        reads: S,
        fires: |s| is_tr(s) && s.shape.branches.iter().any(|b| b.k > 0 && b.sorted),
    },
    Clause {
        reason: Reason {
            class: "a path with no key",
            cite: "Liana v15.0 parse: \"All spend paths must require a signature\" (evidence keyless-hash-path-wsh; F-633)",
        },
        reads: S,
        fires: |s| s.shape.branches.iter().any(keyless_gated),
    },
    Clause {
        reason: Reason {
            class: "one signer twice in a path",
            cite: "Liana: \"is derived from the same origin as another key present in the same spending path\" (evidence X11-wsh-samefp-in-path)",
        },
        reads: K,
        fires: |s| s.fp_partition.iter().flatten().any(|g| g.len() >= 2),
    },
    Clause {
        reason: Reason {
            class: "a key used twice",
            cite: "Liana DuplicateKey (design §1A key_partition)",
        },
        reads: K,
        fires: |s| s.key_partition.iter().any(|g| g.len() >= 2),
    },
    Clause {
        reason: Reason {
            class: "legacy wrapper",
            cite: "liana analysis.rs:586-587 (wsh or tr only)",
        },
        reads: S,
        fires: |s| !matches!(s.root, RootKind::Wsh | RootKind::Tr),
    },
    Clause {
        reason: Reason {
            class: "NUMS key path",
            cite: "liana analysis.rs:568-569",
        },
        reads: S,
        fires: is_nums,
    },
    Clause {
        reason: Reason {
            class: "no locked path",
            cite: "liana analysis.rs:554-558, :472-474, :583",
        },
        reads: S,
        fires: |s| s.shape.branches.iter().all(unlocked),
    },
    Clause {
        reason: Reason {
            class: "a hash lock",
            cite: "liana analysis.rs:186-199, :212-257",
        },
        reads: S,
        fires: |s| s.shape.branches.iter().any(|b| !b.hashlocks.is_empty()),
    },
    Clause {
        reason: Reason {
            class: "a lock in time units",
            cite: "liana csv_check :139-145; message \"Timelock value … isn't valid or safe to use\"",
        },
        // BEFORE "an absolute lock", unlike the Go (its class 6 after class
        // 5): Liana 8.0 and 15.0 refuse mixed-lock-bases-{wsh,tr}, which carry
        // both, with this class's message. The Go order is a D3 the table
        // build reports on three evidence rows.
        reads: S,
        fires: |s| any_lock(s, &[LockKind::OlderUnits]),
    },
    Clause {
        reason: Reason {
            class: "an absolute lock",
            cite: "liana analysis.rs:212-257",
        },
        reads: S,
        fires: |s| any_lock(s, &[LockKind::AfterHeight, LockKind::AfterTime]),
    },
    Clause {
        reason: Reason {
            class: "no unlocked path",
            cite: "liana analysis.rs:633",
        },
        // NOT the Go's `if KeyPath == Spendable { unlocked++ }`: the Rust
        // walk already pushes a real internal key as its own unlocked branch
        // 0, so adding it again double-counts (recon §1d). A Liana key and a
        // NUMS key push no branch, and count as nothing.
        reads: S,
        fires: |s| !s.shape.branches.iter().any(unlocked),
    },
    Clause {
        reason: Reason {
            class: "two paths with one lock",
            cite: "liana analysis.rs:624-626",
        },
        reads: S,
        fires: |s| {
            let mut seen: Vec<u32> = Vec::new();
            for b in &s.shape.branches {
                for l in &b.locks {
                    if l.kind == LockKind::OlderBlocks {
                        if seen.contains(&l.value) {
                            return true;
                        }
                        seen.push(l.value);
                    }
                }
            }
            false
        },
    },
    Clause {
        reason: Reason {
            class: "a second unlocked multi-key path",
            cite: "liana analysis.rs:611-616; evidence X24 (single key folded in, imported) vs X25/X26 (refused)",
        },
        // NARROWER than the Go's class 9, which names every second unlocked
        // path: Liana FOLDS a second single-key unlocked path into the
        // primary and imports it (X24, measured), so refusing that would be a
        // D1 false refusal. What it refuses is a second unlocked path with
        // more than one key.
        reads: S,
        fires: |s| {
            s.shape
                .branches
                .iter()
                .filter(|b| unlocked(b))
                .skip(1)
                .any(|b| b.slots.len() >= 2)
        },
    },
];

/// Liana's primary path is the ONE unlocked path, read as `k`-of-`n`. It
/// folds a second single-key unlocked path into it (X24: built as 2-of-3
/// plus 1-of-1, read as 2-of-4), which is exactly an import "not as built".
fn liana_reads_as_built(s: &Skeleton, d: &Description) -> bool {
    let mut primaries = s.shape.branches.iter().filter(|b| unlocked(b));
    let (Some(p), None) = (primaries.next(), primaries.next()) else {
        return false;
    };
    let built = match (p.k, p.slots.len()) {
        (0, 1) => Some((1, 1)),
        (0, _) => None,
        (k, n) => u8::try_from(n).ok().map(|n| (k, n)),
    };
    built.is_some() && built == d.threshold
}

fn liana_class_of_message(m: &str) -> Option<&'static str> {
    if m.contains("sortedmulti_a") {
        Some("a sortedmulti_a leaf")
    } else if m.contains("All spend paths must require a signature") {
        Some("a path with no key")
    } else if m.contains(
        "is derived from the same origin as another key present in the same spending path",
    ) {
        Some("one signer twice in a path")
    } else if m.starts_with("Timelock value") {
        Some("a lock in time units")
    } else if m.starts_with("A Liana policy requires at least one recovery path") {
        Some("no locked path")
    } else {
        None
    }
}

const LIANA: Coordinator = Coordinator {
    id: CoordinatorId("liana"),
    name: "Liana",
    verified: &["8.0", "15.0"],
    rules: &[RuleSet {
        span: span("8.0", "15.0"),
        source_verified_at: &["8.0"],
        clauses: LIANA_CLAUSES,
        form_refusals: &[],
        measured_refusals_admitted: false,
    }],
    reads_as_built: liana_reads_as_built,
    class_of_message: liana_class_of_message,
};

// ---------------------------------------------------------------------------
// Nunchuk. Two source-derived rules -- a keyless `wsh` path, and ruling
// R-1's key-order rule for Liana's key; every other refusal is measured, because Nunchuk's acceptance is a byte round trip of the
// descriptor text (libnunchuk `a7cfb49` `src/descriptor.cpp:633-648`), so a
// refusal is a claim about a spelling, not a policy (design §1).
// ---------------------------------------------------------------------------

const NUNCHUK_CLAUSES: &[Clause] = &[
    Clause {
        reason: Reason {
            class: "a path with no key",
            cite: "libnunchuk a7cfb49 src/descriptor.cpp:573 (ParseWshDescriptor -> IsValidMiniscriptTemplate), src/nunchukutils.cpp:1276 (IsSane), contrib/bitcoin 57b47c4 src/script/miniscript.h:1617 (IsSane includes NeedsSignature)",
        },
        // `wsh` only: that is the path whose source was read. A `tr` policy's
        // leaves go through `ParseTrDescriptor`, not read for this rule.
        reads: S,
        fires: |s| s.root == RootKind::Wsh && s.shape.branches.iter().any(keyless_gated),
    },
    Clause {
        reason: Reason {
            class: "Liana's key is not the one Nunchuk derives (leaf keys not in sorted order)",
            cite: "libnunchuk a7cfb49 src/descriptor.cpp:689-712 (GetUnspendableXpub sorts and dedups, :700-701) and :646 (re-render must equal the input); ruling R-1",
        },
        // Reads key MATERIAL (`leaf_keys_ascending`), so a template is
        // `Unproven { KeysAbsent }` for Nunchuk (R-1).
        reads: K,
        fires: |s| {
            s.shape.key_path == KeyPathKind::LianaUnspendable
                && s.leaf_keys_ascending == Some(false)
        },
    },
];

/// F-626: Nunchuk shows an unsorted single-path `multi` as `MINISCRIPT
/// 0-of-3` rather than a `k`-of-`n` multisig. The comparison is scoped to
/// that measured case: a single-branch threshold must read as `MULTI_SIG`
/// with the same `k`-of-`n`.
fn nunchuk_reads_as_built(s: &Skeleton, d: &Description) -> bool {
    match s.shape.branches.as_slice() {
        [b] if b.k > 0 => {
            d.wallet_kind == "MULTI_SIG"
                && u8::try_from(b.slots.len()).ok().map(|n| (b.k, n)) == d.threshold
        }
        _ => true,
    }
}

fn no_class(_: &str) -> Option<&'static str> {
    None
}

const NUNCHUK: Coordinator = Coordinator {
    id: CoordinatorId("nunchuk"),
    name: "Nunchuk",
    verified: &["2.1.1"],
    rules: &[RuleSet {
        span: span("2.1.1", "2.1.1"),
        source_verified_at: &["2.1.1"],
        clauses: NUNCHUK_CLAUSES,
        form_refusals: &[],
        measured_refusals_admitted: true,
    }],
    reads_as_built: nunchuk_reads_as_built,
    class_of_message: no_class,
};

// ---------------------------------------------------------------------------
// Bitcoin Core. Boundaries MEASURED on official release binaries
// (`design/agent-reports/coord-compat-core-boundary.md`): tapscript
// miniscript from 26.0, the `<0;1>` multipath spelling from between 28.4
// and 29.4. Source line numbers are `src/script/descriptor.cpp` at the tag.
// ---------------------------------------------------------------------------

const CORE_NO_KEY: Clause = Clause {
    reason: Reason {
        class: "a path with no key",
        cite: "bitcoin src/script/descriptor.cpp NeedsSignature: v24.2:1525, v25.2:1528, v26.0:1800, v27.2:1800, v29.2:2102, v30.0:2483",
    },
    reads: S,
    fires: |s| s.shape.branches.iter().any(keyless_gated),
};

const CORE_MULTIPATH: FormRefusal = FormRefusal {
    form: Form::Multipath,
    reason: Reason {
        class: "the <0;1> multipath spelling (use --chain 0 and --chain 1)",
        cite: "measured 24.2-28.4: \"Key path value '<0;1>' is not a valid uint32\" (coord-compat-core-boundary); bitcoin v26.0 src/script/descriptor.cpp:1290",
    },
};

const CORE_PRE_26: &[Clause] = &[
    Clause {
        reason: Reason {
            class: "miniscript under tr",
            cite: "bitcoin src/script/descriptor.cpp \"Miniscript expressions can only be used in wsh\": v24.2:1507, v25.2:1510; measured 24.2-25.2",
        },
        reads: S,
        // A tap leaf that is anything but `pk(K)` or a bare `multi_a` is
        // miniscript. Under-approximates: a lone `pkh(K)` leaf is missed.
        fires: |s| {
            is_tr(s)
                && s.shape.branches.iter().any(|b| {
                    !b.locks.is_empty()
                        || !b.hashlocks.is_empty()
                        || (b.k == 0 && b.slots.len() >= 2)
                })
        },
    },
    CORE_NO_KEY,
];

fn core_reads_as_built(_: &Skeleton, d: &Description) -> bool {
    d.wallet_kind != "ADDRESSES_DIFFER"
}

fn core_class_of_message(m: &str) -> Option<&'static str> {
    if m.contains("Miniscript expressions can only be used in wsh") || m.contains("multi_a(") {
        Some("miniscript under tr")
    } else if m.contains("is not a valid uint32") {
        Some("the <0;1> multipath spelling (use --chain 0 and --chain 1)")
    } else if m.contains("witnesses without signature exist") {
        Some("a path with no key")
    } else {
        None
    }
}

const CORE: Coordinator = Coordinator {
    id: CoordinatorId("core"),
    name: "Bitcoin Core",
    verified: &[
        "24.2", "25.0", "25.2", "26.0", "26.2", "27.2", "28.4", "29.4", "30.3", "31.1",
    ],
    rules: &[
        RuleSet {
            span: span("24.2", "25.2"),
            source_verified_at: &["24.2", "25.2"],
            clauses: CORE_PRE_26,
            form_refusals: &[CORE_MULTIPATH],
            measured_refusals_admitted: false,
        },
        RuleSet {
            span: span("26.0", "28.4"),
            source_verified_at: &["26.0", "27.2"],
            clauses: &[CORE_NO_KEY],
            form_refusals: &[CORE_MULTIPATH],
            measured_refusals_admitted: false,
        },
        RuleSet {
            span: span("29.4", "31.1"),
            source_verified_at: &["29.2", "30.0"],
            clauses: &[CORE_NO_KEY],
            form_refusals: &[],
            measured_refusals_admitted: false,
        },
    ],
    reads_as_built: core_reads_as_built,
    class_of_message: core_class_of_message,
};
```

Create `crates/md-codec/src/coordinator/table.rs`:

```rust
//! GENERATED by `cargo xtask verdicts` from
//! `crates/md-codec/tests/fixtures/coordinator/evidence.jsonl`. Do not edit.

#[allow(unused_imports)]
use super::{Cell, CellOutcome, Description, Form};

pub(super) static CELLS: &[Cell] = &[
];
```

- [ ] **Step 4:** PASS. **Step 5:** the gate. Commit
  `coordinator: verdict kinds, the registry and runtime verdicts (table empty)`.

---

### Task 5: Vendor the evidence (R-2)

**Files:** Create `scripts/vendor-coord-evidence.sh`; Modify `.gitattributes`;
generated: `crates/md-codec/tests/fixtures/coordinator/evidence.{jsonl,meta.json}`.

- [ ] **Step 1: The script.** Per-SOURCE provenance constants are cited in
  its comments; `measured_at` comes from `git blame` (the commit that
  recorded each line); Core's version is asserted against the binary's
  self-reported `/Satoshi:x.y.z/`.

Create `scripts/vendor-coord-evidence.sh`:

```bash
#!/usr/bin/env bash
# scripts/vendor-coord-evidence.sh -- vendor coordinator-compatibility
# EVIDENCE from mnemonic-engrave into this repo (ruling R-2, coordinator-compat
# plan 1b), where `cargo xtask verdicts` builds the verdict table from it.
#
# Usage:
#   scripts/vendor-coord-evidence.sh <path-to-mnemonic-engrave>          # write
#   scripts/vendor-coord-evidence.sh --check <path-to-mnemonic-engrave>  # freshness
#
# Writes crates/md-codec/tests/fixtures/coordinator/evidence.jsonl (one row per
# measurement) and evidence.meta.json (the engrave commit and the sha256 of every
# input). --check regenerates both into a temp dir and fails (exit 1) if either
# the rows or the input hashes differ from what is committed -- i.e. if engrave's
# evidence moved and this copy did not. The engrave commit itself is NOT
# compared: an unrelated engrave commit must not make this copy stale.
#
# THIS SCRIPT NEVER COMPUTES A KEY. It transcribes what each harness printed,
# plus per-SOURCE provenance constants (each cites where it is stated). Keys are
# computed by md-codec's one implementation when the table is built.
#
# Rows, and the one rule for each source:
#   Liana 8.0   composer-fable-r0/fable-liana-parse-{in,out}.jsonl, variant "md"
#   Liana 15.0  composer-fable-r0/fable-liana-parse-{in,out-v15}.jsonl, variants
#               "md" and "liana-unspendable-xpub"
#   Liana 15.0  f449-stage2/liana-live-gate-{in,expected}.jsonl (all rows)
#   Liana 15.0  f449-stage4/liana-probes-{parse-in,out}.jsonl (all rows)
#   Nunchuk     composer-fable-r0/fable-nunchuk-harness-out.txt, "<name>.multipath"
#               sections, descriptor from fable-liana-shapes.json's desc_md
#   Nunchuk     coord-compat-1b/nunchuk-kind1-probe.{tsv,out} (the recon's probe)
#   Core        coord-compat-core-boundary/core-<v>.json: ALL15 (chain0 AND chain1,
#               both were imported), K1 (same), and the multipath refusals
# measured_at is the committer date of the commit that recorded the row's line
# (git blame), so it is mechanical, not typed.
set -euo pipefail

check=0
if [[ "${1:-}" == "--check" ]]; then check=1; shift; fi
if [[ $# -ne 1 ]]; then
  echo "usage: $0 [--check] <path-to-mnemonic-engrave>" >&2
  exit 2
fi
engrave=$1
repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
out_dir="$repo_root/crates/md-codec/tests/fixtures/coordinator"
work=$out_dir
if [[ $check -eq 1 ]]; then work=$(mktemp -d); trap 'rm -rf "$work"' EXIT; fi
mkdir -p "$work"

python3 - "$engrave" "$work" <<'PY'
import hashlib, json, os, re, subprocess, sys

engrave, work = sys.argv[1], sys.argv[2]
EV = "design/evidence"

def path(rel):
    p = os.path.join(engrave, EV, rel)
    if not os.path.isfile(p):
        sys.exit(f"vendor-coord-evidence: not found: {p}")
    return p

inputs = {}
def read_lines(rel):
    p = path(rel)
    data = open(p, "rb").read()
    inputs[rel] = hashlib.sha256(data).hexdigest()
    return data.decode().split("\n")

def dates(rel):
    """Committer date (YYYY-MM-DD) of the commit that recorded each line."""
    out = subprocess.run(["git", "-C", engrave, "blame", "--line-porcelain", "--", f"{EV}/{rel}"],
                         check=True, capture_output=True, text=True).stdout
    per_line, t = [], None
    for l in out.split("\n"):
        if l.startswith("committer-time "):
            t = int(l.split()[1])
        elif l.startswith("\t"):
            per_line.append(subprocess.run(["date", "-u", "-d", f"@{t}", "+%Y-%m-%d"],
                                           check=True, capture_output=True, text=True).stdout.strip())
    return per_line

def jsonl(rel):
    lines = read_lines(rel)
    d = dates(rel)
    return [(json.loads(l), f"{rel}:{i+1}", d[i]) for i, l in enumerate(lines) if l.strip()]

rows = []
def row(id_, name, coord, version, lib, tool, tver, form, at, desc, outcome, unkeyable=None):
    r = {"id": id_, "name": name, "coordinator": coord, "version": version, "library_rev": lib,
         "renderer_tool": tool, "renderer_version": tver, "form": form,
         "measured_at": at, "descriptor": desc, "outcome": outcome}
    if unkeyable:
        r["unkeyable"] = unkeyable
    rows.append(r)

def liana_reading(rec):
    p = rec.get("primary")
    if not isinstance(p, dict):
        return None
    return {"wallet_kind": "Liana", "threshold": [p.get("k", 1), len(p["keys"])]}

def liana_outcome(rec):
    if rec.get("ok") is True:
        return {"imported": liana_reading(rec)}
    return {"refused": rec["error"]}

# Liana harness descriptors whose internal key md1 cannot carry (a stale key:
# neither NUMS nor Liana's recipe over these leaves, and origin-less).
LIANA_UNKEYABLE = {"CONTROL-stale-key-nested-B-tree-with-A-internal-key":
                   "internal key is neither NUMS nor Liana's recipe over these leaves"}

# -- Liana, fable r0 (8.0 and 15.0) ----------------------------------------
shapes = {s["name"]: s for s in json.loads("\n".join(read_lines("composer-fable-r0/fable-liana-shapes.json")))}
fin = {(r["name"], r["variant"]): r["desc"] for r, _, _ in jsonl("composer-fable-r0/fable-liana-parse-in.jsonl")}
# Liana v8.0 at 9d2fb742 and md 0.17.0: agent-reports/composer-fable-r0-liana-core.md:17
for rel, ver, lib, variants in [
    ("composer-fable-r0/fable-liana-parse-out.jsonl", "8.0", "9d2fb742", {"md"}),
    ("composer-fable-r0/fable-liana-parse-out-v15.jsonl", "15.0", "4684d5cb", {"md", "liana-unspendable-xpub"}),
]:
    for rec, id_, at in jsonl(rel):
        if rec["variant"] not in variants:
            continue
        tool, tver = ("md", "0.17.0") if rec["variant"] == "md" else ("harnesses/liana unspendable", "v15.0")
        row(id_, rec["name"], "liana", ver, lib, tool, tver, "multipath", at,
            fin[(rec["name"], rec["variant"])], liana_outcome(rec))

# -- Liana, F-449 live gate and stage-4 probes (15.0 at 4684d5cb) -----------
for in_rel, out_rel in [("f449-stage2/liana-live-gate-in.jsonl", "f449-stage2/liana-live-gate-expected.jsonl"),
                        ("f449-stage4/liana-probes-parse-in.jsonl", "f449-stage4/liana-probes-out.jsonl")]:
    din = {r["name"]: r["desc"] for r, _, _ in jsonl(in_rel)}
    for rec, id_, at in jsonl(out_rel):
        if "name" not in rec:
            assert rec.get("liana_tag") == "v15.0", rec  # the header pins the tag
            continue
        row(id_, rec["name"], "liana", "15.0", "4684d5cb", "harnesses/liana unspendable", "v15.0", "multipath",
            at, din[rec["name"]], liana_outcome(rec), LIANA_UNKEYABLE.get(rec["name"]))

# -- Nunchuk 2.1.1 (libnunchuk a7cfb49), fable r0, md 0.16.2 ----------------
# agent-reports/composer-fable-r0-nunchuk.md:6 (md 0.16.2), recon §3c (a7cfb49 = 2.1.1's pin)
rel = "composer-fable-r0/fable-nunchuk-harness-out.txt"
lines, d = read_lines(rel), dates(rel)
i = 0
while i < len(lines):
    m = re.match(r"^### (.+)\.multipath$", lines[i])
    if not m:
        i += 1
        continue
    name, start, rec, body = m.group(1), i, {}, []
    i += 1
    while i < len(lines) and not lines[i].startswith("### "):
        body.append(lines[i])
        for k, v in re.findall(r"(\w+)=(\S*)", lines[i]):
            rec.setdefault(k, v)
        i += 1
    if rec["ParseWalletDescriptor"] == "ACCEPT":
        kind = rec["wallet_type"]
        thr = [int(rec["m"]), int(rec["n"])] if kind == "MULTI_SIG" else None
        outcome = {"imported": {"wallet_kind": kind, "threshold": thr}}
    else:
        line = next(l.strip() for l in body if l.strip().startswith("ParseWalletDescriptor="))
        outcome = {"refused": line}
    row(f"{rel}:{start+1}", name, "nunchuk", "2.1.1", "a7cfb49", "md", "0.16.2", "multipath",
        d[start], shapes[name]["desc_md"], outcome)

# -- Nunchuk 2.1.1, the recon's kind-1 probe (ruling R-1's evidence) --------
rel_tsv, rel_out = "coord-compat-1b/nunchuk-kind1-probe.tsv", "coord-compat-1b/nunchuk-kind1-probe.out"
descs = {}
for l in read_lines(rel_tsv):
    if l.strip():
        n, desc = l.split("\t")[:2]
        descs[n] = desc
lines, d = read_lines(rel_out), dates(rel_out)
PR1746 = "PR-1746 form: an origin-less real internal key, which md1 cannot encode (recon §4)"
for i, l in enumerate(lines):
    m = re.match(r"^### (\S+)$", l)
    if not m:
        continue
    name = m.group(1)
    end = next((j for j in range(i + 1, len(lines)) if lines[j].startswith("### ")), len(lines))
    block = "\n".join(lines[i + 1:end])
    if "ParseWalletDescriptor=ACCEPT" in block:
        outcome = {"imported": {"wallet_kind": re.search(r"wallet_type=(\S+)", block).group(1), "threshold": None}}
    else:
        outcome = {"refused": re.search(r"ParseWalletDescriptor=REFUSE[^\n]*", block).group(0)}
    row(f"{rel_out}:{i+1}", name, "nunchuk", "2.1.1", "a7cfb49", "recon-1b-nunchuk-probe.py", "2026-09-23",
        "multipath", d[i], descs[name], outcome, PR1746 if name.startswith("pr1746") else None)

# -- Bitcoin Core, official release binaries --------------------------------
live = {r["name"]: r for r, _, _ in jsonl("f449-stage2/liana-live-gate-expected.jsonl") if "name" in r}
k1_desc = live["CONTROL-accept-kofn-recovery-flat"]["liana_desc"]
for v in ["24.2", "25.0", "25.2", "26.0", "26.2", "27.2", "28.4", "29.4", "30.3", "31.1"]:
    rel = f"coord-compat-core-boundary/core-{v}.json"
    rec = json.loads("\n".join(read_lines(rel)))
    at = dates(rel)[0]
    got = re.fullmatch(r"/Satoshi:(\d+)\.(\d+)\.\d+/", rec["version"])
    assert got and f"{got.group(1)}.{got.group(2)}" == v, (v, rec["version"])  # self-reported
    lib = rec["version"]
    def core_outcome(text):
        if text.startswith("ACCEPT"):
            return {"imported": None if "addr==md" in text else {"wallet_kind": "ADDRESSES_DIFFER", "threshold": None}}
        return {"refused": text.split("error message:")[-1].strip()}
    for name, verdict in rec["ALL15"].items():
        for form in ("chain0", "chain1"):
            row(f"{rel}:ALL15.{name}", name, "core", v, lib, "md", "0.17.0", form, at,
                shapes[name]["desc_md"], core_outcome(verdict))
    k1 = rec["K1"]
    k1_text = ("ACCEPT addr==md" if k1["addr_match"] is True else "ACCEPT")\
        if k1["verdict_import"] == "ACCEPT" else k1["getdescriptorinfo_c0"]
    for form in ("chain0", "chain1"):
        row(f"{rel}:K1", "CONTROL-accept-kofn-recovery-flat", "core", v, lib, "harnesses/liana unspendable", "v15.0", form, at,
            k1_desc, core_outcome(k1_text))
    for tag, name, desc, tool, tver in [
            ("REP", "preset-kofn-recovery-tr", shapes["preset-kofn-recovery-tr"]["desc_md"], "md", "0.17.0"),
            ("K1", "CONTROL-accept-kofn-recovery-flat", k1_desc, "harnesses/liana unspendable", "v15.0")]:
        mp = rec[tag]["getdescriptorinfo_multipath"]
        if mp != "ok":  # an ok here is a parse, not an import: no positive row
            row(f"{rel}:{tag}.multipath", name, "core", v, lib, tool, tver, "multipath", at,
                desc, {"refused": mp.split("error message:")[-1].strip()})

with open(os.path.join(work, "evidence.jsonl"), "w") as f:
    for r in rows:
        f.write(json.dumps(r, sort_keys=True, ensure_ascii=False) + "\n")
head = subprocess.run(["git", "-C", engrave, "rev-parse", "HEAD"], check=True,
                      capture_output=True, text=True).stdout.strip()
with open(os.path.join(work, "evidence.meta.json"), "w") as f:
    json.dump({"engrave_commit": head, "inputs": dict(sorted(inputs.items()))}, f, indent=1, sort_keys=True)
    f.write("\n")
print(f"vendor-coord-evidence: {len(rows)} rows from {len(inputs)} inputs")
PY

if [[ $check -eq 1 ]]; then
  stale=0
  cmp -s "$work/evidence.jsonl" "$out_dir/evidence.jsonl" || { echo "STALE: evidence.jsonl differs from a fresh vendoring" >&2; stale=1; }
  inputs() { python3 -c 'import json,sys; print(json.dumps(json.load(open(sys.argv[1]))["inputs"], sort_keys=True))' "$1"; }
  [[ "$(inputs "$work/evidence.meta.json")" == "$(inputs "$out_dir/evidence.meta.json")" ]] \
    || { echo "STALE: an engrave evidence input changed since vendoring" >&2; stale=1; }
  [[ $stale -eq 0 ]] && echo "vendor-coord-evidence: fresh"
  exit $stale
fi
```

- [ ] **Step 2: Line endings** — the xtask test compares the generated table
  byte-for-byte, so a Windows checkout must not convert either side (the
  repo's own `.gitattributes` precedent for vectors).

Modify `.gitattributes`:

```diff
diff --git a/.gitattributes b/.gitattributes
index 6509d3f..df4f561 100644
--- a/.gitattributes
+++ b/.gitattributes
@@ -23,3 +23,9 @@ vendor/** -text
 # assertion while Linux, macOS, musl and FreeBSD stayed green.
 # MUST sort after any `* text=auto` (last-match-wins per attribute).
 design/display-grouping-vectors.tsv text eol=lf
+
+# Coordinator-compat plan 1b: the xtask's `table_is_fresh` test compares the
+# generated table byte-for-byte with what the evidence builds, so a Windows
+# checkout must not CRLF-convert either side.
+crates/md-codec/src/coordinator/table.rs text eol=lf
+crates/md-codec/tests/fixtures/coordinator/* text eol=lf
```

- [ ] **Step 3: Run it** (after Task 0 Step 1):

```sh
scripts/vendor-coord-evidence.sh /scratch/code/shibboleth/mnemonic-engrave
# expect: vendor-coord-evidence: 526 rows from 21 inputs
scripts/vendor-coord-evidence.sh --check /scratch/code/shibboleth/mnemonic-engrave
# expect: vendor-coord-evidence: fresh
```

  The freshness check is non-vacuous: edit one byte of any input in a scratch
  clone and `--check` reports `STALE` (exit 1). **Step 4:** the gate (no new
  test yet; the rows are consumed in Task 6). Commit
  `evidence: vendor coordinator evidence from mnemonic-engrave (R-2)`.

---

### Task 6: The table build, the xtask, and the gate over evidence rows

`coordinator::build` keys every row, applies the rules, classifies D1-D5,
and renders `table.rs`; `crates/xtask` parses the JSONL and writes (or
`--check`s) the file. The test file grows from Task 4's one test to the
evidence-row gate and the verdict tests.

**Files:** Create `crates/md-codec/src/coordinator/build.rs`,
`crates/xtask/Cargo.toml`, `crates/xtask/src/main.rs`; Modify
`crates/md-codec/src/coordinator/mod.rs`, `Cargo.toml`, `.cargo/config.toml`;
Replace `crates/md-codec/tests/coordinator.rs`; generated: `Cargo.lock`,
`crates/md-codec/src/coordinator/table.rs`.

- [ ] **Step 1: The tests.** Each names its mutation; the ones marked
  *measured* were run and red:

  | test | mutation that reds it |
  | --- | --- |
  | `every_evidence_row_keys_identically_through_chunks_and_text` | `is_liana_unspendable_key` → `false` (*measured*) |
  | `every_verified_version_has_exactly_one_rule_set` | shrink a span (*measured*) |
  | `r1_leaf_order_splits_one_key_into_two_nunchuk_verdicts` | R-1 clause `== Some(true)` (*measured*) |
  | `core_verdicts_follow_the_measured_boundary` | drop `CORE_MULTIPATH` from 26.0-28.4 (*measured*) |
  | `a_template_is_never_claimed_to_import` | delete `at_version`'s `keys_present` return (*measured*) |
  | `imports_altered_is_derived_from_the_coordinators_own_reading` | `liana_reads_as_built` → `true`: `table_is_fresh` reds first, then this after regeneration (*measured*) |
  | `none_imports_only_when_every_run_refuses` | `all` → `any` (*measured*) |
  | `the_build_names_every_disagreement_class` | drop the D5 comparison (*measured*) |
  | xtask `table_is_fresh` | delete any `Cell` line from `table.rs` |

Replace `crates/md-codec/tests/coordinator.rs`:

```rust
//! Coordinator-compatibility plan 1b: verdicts, the rule/evidence build, and
//! the conformance gate over EVIDENCE rows.
//!
//! Every test names the mutation that reds it.

mod common;

use md_codec::chunk::{reassemble, split};
use md_codec::coordinator::build::{Disagreement, EvidenceRow, MeasuredOutcome, build};
use md_codec::coordinator::{
    CoordinatorVerdict, Form, REGISTRY, UnprovenReason, Verdict, none_imports, verdicts,
};
use md_codec::descriptor_route::descriptor_from_text;
use md_codec::encode::Descriptor;
use md_codec::skeleton::{Skeleton, skeleton, skeleton_key};

fn evidence() -> Vec<serde_json::Value> {
    let p = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/coordinator/evidence.jsonl"
    );
    std::fs::read_to_string(p)
        .unwrap_or_else(|e| panic!("{p}: {e}"))
        .lines()
        .map(|l| serde_json::from_str(l).expect("evidence row parses"))
        .collect()
}

/// The descriptor of the first evidence row named `name` from `coordinator`.
fn descriptor_named(coordinator: &str, name: &str) -> String {
    evidence()
        .into_iter()
        .find(|r| r["coordinator"] == coordinator && r["name"] == name)
        .unwrap_or_else(|| panic!("no {coordinator} evidence row named {name}"))["descriptor"]
        .as_str()
        .unwrap()
        .to_string()
}

fn keyed(text: &str) -> (Descriptor, Skeleton) {
    let d = descriptor_from_text(text).unwrap_or_else(|e| panic!("{e}: {text}"));
    let s = skeleton(&d).unwrap_or_else(|e| panic!("{e}"));
    (d, s)
}

/// The same policy with its key material removed: a template-only card.
fn template_of(d: &Descriptor) -> Skeleton {
    let mut t = d.clone();
    t.tlv.pubkeys = None;
    t.tlv.fingerprints = None;
    skeleton(&t).unwrap_or_else(|e| panic!("template: {e}"))
}

fn of<'a>(vs: &'a [CoordinatorVerdict], id: &str) -> Vec<&'a Verdict> {
    vs.iter()
        .filter(|v| v.coordinator.id.0 == id)
        .map(|v| &v.verdict)
        .collect()
}

fn span_of(v: &Verdict) -> String {
    match v {
        Verdict::Refuses { span, .. }
        | Verdict::ImportsAltered { span, .. }
        | Verdict::Imports { span, .. } => span.to_string(),
        Verdict::Unproven { span, .. } => span.map_or(String::new(), |s| s.to_string()),
    }
}

/// Design §2's gate, over EVIDENCE rows rather than the vector corpus
/// (the recon found 0 of 11 Liana live-gate rows and 1 of 56 matrix shapes in
/// that corpus): every keyable row keys the same through md1 chunks as
/// through its descriptor text. Mutation: make `is_liana_unspendable_key`
/// return false -> every kind-1 row fails to key and this reds.
#[test]
fn every_evidence_row_keys_identically_through_chunks_and_text() {
    let mut checked = 0;
    for r in evidence() {
        if r.get("unkeyable").is_some() {
            continue;
        }
        let id = r["id"].as_str().unwrap();
        let (d, s) = keyed(r["descriptor"].as_str().unwrap());
        let chunks = split(&d).unwrap_or_else(|e| panic!("{id}: split: {e}"));
        let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
        let back = reassemble(&refs).unwrap_or_else(|e| panic!("{id}: reassemble: {e}"));
        let s2 = skeleton(&back).unwrap_or_else(|e| panic!("{id}: {e}"));
        assert_eq!(
            skeleton_key(&s),
            skeleton_key(&s2),
            "{id}: the key knows its route"
        );
        checked += 1;
    }
    assert!(checked >= 524, "only {checked} evidence rows keyed");
}

/// Every verified version has exactly one rule set. Mutation: shrink Core's
/// 29.4-31.1 span to 29.4-30.3 -> 31.1 has none and this reds.
#[test]
fn every_verified_version_has_exactly_one_rule_set() {
    for c in REGISTRY {
        for v in c.verified {
            let at = c.position(v).unwrap();
            let n = c
                .rules
                .iter()
                .filter(|r| {
                    let (a, b) = (c.position(r.span.since.0), c.position(r.span.until.0));
                    matches!((a, b), (Some(a), Some(b)) if a <= at && at <= b)
                })
                .count();
            assert_eq!(n, 1, "{} {v}: {n} rule sets", c.name);
        }
    }
}

/// Ruling R-1, end to end. The recon measured two kind-1 cards whose
/// SkeletonKeys are byte-identical and whose Nunchuk verdicts are opposite;
/// the key stays coordinator-independent and the rule reads key material.
/// Mutation: flip the Nunchuk R-1 clause to `== Some(true)` -> the sorted
/// card is refused and the unsorted one is silent; this reds (and so does
/// the xtask's `table_is_fresh`, on a D1).
#[test]
fn r1_leaf_order_splits_one_key_into_two_nunchuk_verdicts() {
    let (_, unsorted) = keyed(&descriptor_named("nunchuk", "k1-liana-unsorted"));
    let (d, sorted) = keyed(&descriptor_named("nunchuk", "k1-liana-sorted"));
    assert_eq!(
        skeleton_key(&unsorted),
        skeleton_key(&sorted),
        "the recon's collision"
    );
    assert_eq!(unsorted.leaf_keys_ascending, Some(false));
    assert_eq!(sorted.leaf_keys_ascending, Some(true));

    let u = verdicts(&unsorted, Some(Form::Multipath));
    assert!(
        matches!(
            of(&u, "nunchuk")[..],
            [Verdict::Refuses { renderer: None, .. }]
        ),
        "unsorted: a rule-derived refusal, {:?}",
        of(&u, "nunchuk")
    );
    let s = verdicts(&sorted, Some(Form::Multipath));
    assert!(
        matches!(of(&s, "nunchuk")[..], [Verdict::Imports { .. }]),
        "sorted: the measured positive, {:?}",
        of(&s, "nunchuk")
    );
    let t = verdicts(&template_of(&d), None);
    assert!(
        matches!(
            of(&t, "nunchuk")[..],
            [Verdict::Unproven {
                reason: UnprovenReason::KeysAbsent,
                ..
            }]
        ),
        "a template: {:?}",
        of(&t, "nunchuk")
    );
}

/// The Core boundary, MEASURED, prints as closed runs of verified versions.
/// Per-chain spelling: refused through 25.2, imported 26.0-31.1. Multipath:
/// refused through 28.4 for the spelling, silent after (only parsed, never
/// imported, there). Mutation: drop `CORE_MULTIPATH` from the 26.0-28.4 rule
/// set -> the multipath runs change and this reds.
#[test]
fn core_verdicts_follow_the_measured_boundary() {
    let (_, s) = keyed(&descriptor_named(
        "core",
        "CONTROL-accept-kofn-recovery-flat",
    ));
    let c0 = verdicts(&s, Some(Form::Chain0));
    let core: Vec<_> = of(&c0, "core");
    assert_eq!(core.len(), 2, "{core:?}");
    assert!(
        matches!(core[0], Verdict::Refuses { reason, renderer: None, .. } if reason.class == "miniscript under tr")
    );
    assert_eq!(span_of(core[0]), "24.2-25.2");
    assert!(matches!(core[1], Verdict::Imports { .. }));
    assert_eq!(span_of(core[1]), "26.0-31.1");

    let mp = verdicts(&s, Some(Form::Multipath));
    let core: Vec<_> = of(&mp, "core");
    assert_eq!(core.len(), 2, "{core:?}");
    assert!(matches!(core[0], Verdict::Refuses { reason, .. } if reason.class.contains("<0;1>")));
    assert_eq!(span_of(core[0]), "24.2-28.4");
    assert!(matches!(
        core[1],
        Verdict::Unproven {
            reason: UnprovenReason::NoEvidence,
            ..
        }
    ));
    assert_eq!(span_of(core[1]), "29.4-31.1");
}

/// Design §1 (a2): a template can be refused, never claimed to import.
/// X24 is imported by Liana and by Core when keyed. Two templates of it must
/// claim nothing: the real one (whose partitions render EMPTY, so its key
/// differs from the seated card's -- measured here, see the design fold) and
/// the seated Skeleton with only `keys_present` cleared, which keeps the
/// seated key and so would find the measured cells if the guard were gone.
/// Mutation: delete `at_version`'s `if !s.keys_present` return -> the second
/// template reads `Imports` and this reds.
#[test]
fn a_template_is_never_claimed_to_import() {
    let (d, keyed_s) = keyed(&descriptor_named(
        "liana",
        "X24-wsh-2of3-1of1-unlocked-plus-rec",
    ));
    let real = template_of(&d);
    assert_ne!(
        skeleton_key(&real),
        skeleton_key(&keyed_s),
        "a template's partitions are empty"
    );
    let mut cleared = keyed_s.clone();
    cleared.keys_present = false;
    assert_eq!(
        skeleton_key(&cleared),
        skeleton_key(&keyed_s),
        "keys_present is not in the key"
    );
    for t in [real, cleared] {
        for form in [None, Some(Form::Multipath), Some(Form::Chain0)] {
            for v in verdicts(&t, form) {
                assert!(
                    !matches!(
                        v.verdict,
                        Verdict::Imports { .. } | Verdict::ImportsAltered { .. }
                    ),
                    "{form:?}: {} claimed a positive for a template",
                    v.coordinator.name
                );
            }
        }
    }
    // A structure-only refusal still stands without keys.
    let (d, _) = keyed(&descriptor_named("liana", "preset-hashlock-gated-wsh"));
    let t = verdicts(&template_of(&d), None);
    assert!(matches!(
        of(&t, "liana")[..],
        [Verdict::Refuses { reason, .. }] if reason.class == "a hash lock"
    ));
}

/// Design §1 (c): measured, "accepts, but not as built". X24 (Liana reads
/// 2-of-3 plus 1-of-1 as 2-of-4) and F-626 (Nunchuk reads an unsorted 2-of-3
/// as MINISCRIPT). The runtime reads the GENERATED table, so a mutation of
/// `liana_reads_as_built` (return true) reds the xtask's `table_is_fresh`
/// first; after `cargo xtask verdicts` regenerates, X24 is a plain `Imports`
/// and this reds.
#[test]
fn imports_altered_is_derived_from_the_coordinators_own_reading() {
    let (_, x24) = keyed(&descriptor_named(
        "liana",
        "X24-wsh-2of3-1of1-unlocked-plus-rec",
    ));
    let v = verdicts(&x24, Some(Form::Multipath));
    assert!(
        matches!(of(&v, "liana")[..], [Verdict::ImportsAltered { as_read, .. }] if as_read.threshold == Some((2, 4))),
        "{:?}",
        of(&v, "liana")
    );
    assert_eq!(span_of(of(&v, "liana")[0]), "8.0-15.0");

    let (_, unsorted) = keyed(&descriptor_named("nunchuk", "plain-2of3-wsh-UNSORTED"));
    let v = verdicts(&unsorted, Some(Form::Multipath));
    assert!(
        matches!(of(&v, "nunchuk")[..], [Verdict::ImportsAltered { as_read, .. }] if as_read.wallet_kind == "MINISCRIPT"),
        "{:?}",
        of(&v, "nunchuk")
    );
}

/// Ruling 2's none case: every coordinator refuses at every verified
/// version. A keyless `wsh` path is refused by all three from source; a
/// template that some coordinators refuse and others are merely silent on is
/// not "none". Mutation: `none_imports` using `any` -> the kofn `tr` template
/// reads as none and this reds.
#[test]
fn none_imports_only_when_every_run_refuses() {
    let (d, _) = keyed(&descriptor_named("liana", "keyless-hash-path-wsh"));
    let t = verdicts(&template_of(&d), None);
    assert!(none_imports(&t), "{t:?}");

    // Some runs refuse (Liana: NUMS key path; Core 24.2-25.2: miniscript
    // under tr), others are silent: not "none".
    let (d, _) = keyed(&descriptor_named("liana", "preset-kofn-recovery-tr"));
    let t = verdicts(&template_of(&d), None);
    assert!(
        t.iter()
            .any(|v| matches!(v.verdict, Verdict::Refuses { .. })),
        "precondition: {t:?}"
    );
    assert!(!none_imports(&t), "{t:?}");
}

fn row(name: &str, coordinator: &str, version: &str, outcome: MeasuredOutcome) -> EvidenceRow {
    EvidenceRow {
        id: format!("synthetic:{name}"),
        coordinator: coordinator.into(),
        version: version.into(),
        library_rev: "test".into(),
        renderer_tool: "test".into(),
        renderer_version: "0".into(),
        form: Form::Multipath,
        measured_at: "2026-09-23".into(),
        descriptor: descriptor_named("liana", name),
        outcome,
        unkeyable: None,
    }
}

/// The build fails on every disagreement class, each for its own reason.
/// Mutation: delete the D5 comparison in `build` -> the conflicting pair is
/// silently deduplicated and this reds.
#[test]
fn the_build_names_every_disagreement_class() {
    use MeasuredOutcome::{Imported, Refused};
    let generic = || Refused("Descriptor is not compatible with a Liana spending policy.".into());
    let cases: Vec<(EvidenceRow, &str)> = vec![
        // single-wsh has no locked path: a Liana rule refuses it.
        (row("single-wsh", "liana", "15.0", Imported(None)), "D1"),
        // kofn-recovery-wsh is Liana's own shape: no rule refuses it.
        (
            row("preset-kofn-recovery-wsh", "liana", "15.0", generic()),
            "D2",
        ),
        // older-units: the rule says time units; this message names another class.
        (
            row(
                "older-units-wsh",
                "liana",
                "15.0",
                Refused("A Liana policy requires at least one recovery path.".into()),
            ),
            "D3",
        ),
        (
            row("preset-kofn-recovery-wsh", "liana", "9.9", Imported(None)),
            "D4",
        ),
    ];
    for (r, want) in cases {
        let got = build(std::slice::from_ref(&r)).err().unwrap_or_default();
        let class = match got.as_slice() {
            [Disagreement::D1FalseRefusal { .. }] => "D1",
            [Disagreement::D2MissedRefusal { .. }] => "D2",
            [Disagreement::D3ReasonDrift { .. }] => "D3",
            [Disagreement::D4Orphan { .. }] => "D4",
            other => panic!("{}: {other:?}", r.id),
        };
        assert_eq!(class, want, "{}", r.id);
    }
    // D5: Nunchuk admits measured refusals, so two rows for one cell can
    // disagree only with each other.
    let a = row(
        "preset-kofn-recovery-wsh",
        "nunchuk",
        "2.1.1",
        Imported(None),
    );
    let mut b = a.clone();
    b.outcome = Refused("ParseWalletDescriptor=REFUSE".into());
    b.id = "synthetic:b".into();
    let got = build(&[a, b]).err().unwrap_or_default();
    assert!(
        matches!(got[..], [Disagreement::D5EvidenceConflict { .. }]),
        "{got:?}"
    );
    // A row declared unkeyable must really be unkeyable.
    let mut k = row("preset-kofn-recovery-wsh", "liana", "15.0", Imported(None));
    k.unkeyable = Some("claimed".into());
    let got = build(&[k]).err().unwrap_or_default();
    assert!(matches!(got[..], [Disagreement::Keying { .. }]), "{got:?}");
}
```

- [ ] **Step 2:** run — does not compile (no `build`). **Step 3:** the build.

Modify `crates/md-codec/src/coordinator/mod.rs`:

```diff
diff --git a/crates/md-codec/src/coordinator/mod.rs b/crates/md-codec/src/coordinator/mod.rs
index 01d0f7a..1202000 100644
--- a/crates/md-codec/src/coordinator/mod.rs
+++ b/crates/md-codec/src/coordinator/mod.rs
@@ -17,6 +17,8 @@
 //! (a2)): every clause that reads key identity is skipped and the verdict is
 //! `Unproven { KeysAbsent }`.
 
+#[cfg(feature = "derive")]
+pub mod build;
 mod registry;
 #[rustfmt::skip]
 mod table;
```

Create `crates/md-codec/src/coordinator/build.rs`:

```rust
//! The rule/evidence table BUILD (design §2): where rules and measurements
//! meet, once, at build time — never at runtime.
//!
//! Every vendored evidence row is keyed through the ONE key implementation
//! ([`crate::descriptor_route::descriptor_from_text`] then
//! [`crate::skeleton::skeleton`]), checked against its coordinator's rules,
//! and either stored as a measured [`super::Cell`] or reported as a
//! disagreement. Any disagreement fails the build; a person resolves it and
//! commits the resolution (a narrower or wider rule, a re-attributed class,
//! or a dropped row).
//!
//! | class | rule says | evidence says |
//! | --- | --- | --- |
//! | D1 false-refusal | refuses | imported |
//! | D2 missed-refusal | admits | refused (only where the rule set does not admit measured refusals) |
//! | D3 reason-drift | refuses for X | refused with a message naming Y |
//! | D4 orphan-evidence | no rule set spans the version | any |
//! | D5 evidence-conflict | — | two rows, one key/coordinator/version/form, different outcomes |
//!
//! D5 is not in the design's first four (recon §1c): two measurements that
//! share a key and disagree cannot be expressed by the table, and the build
//! must say so rather than keep whichever row came last.

use super::{Form, REGISTRY, first_refusal};
use crate::descriptor_route::descriptor_from_text;
use crate::skeleton::{Skeleton, skeleton, skeleton_key};

/// One vendored evidence row, as the harness recorded it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRow {
    /// `<source file>:<line>` — where the row came from.
    pub id: String,
    /// A registry coordinator id.
    pub coordinator: String,
    /// The application version measured.
    pub version: String,
    /// The library/commit the binary self-reported (provenance).
    pub library_rev: String,
    /// The tool that rendered `descriptor`.
    pub renderer_tool: String,
    /// Its version.
    pub renderer_version: String,
    /// The spelling the coordinator saw.
    pub form: Form,
    /// ISO date.
    pub measured_at: String,
    /// The MULTIPATH descriptor text the key is computed from.
    pub descriptor: String,
    /// What the coordinator did.
    pub outcome: MeasuredOutcome,
    /// `Some(reason)` when the vendoring script declares this row cannot be
    /// keyed (a form md1 cannot carry). The build asserts it indeed cannot.
    pub unkeyable: Option<String>,
}

/// What a harness recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeasuredOutcome {
    /// Imported; the coordinator's own reading, where the harness records one.
    Imported(Option<OwnedDescription>),
    /// Refused, with the coordinator's message.
    Refused(String),
}

/// An owned [`super::Description`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedDescription {
    /// See [`super::Description::wallet_kind`].
    pub wallet_kind: String,
    /// See [`super::Description::threshold`].
    pub threshold: Option<(u8, u8)>,
}

/// A stored outcome, owned.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuiltOutcome {
    /// Imported as built.
    Imported,
    /// Imported, read as something else.
    ImportedAltered {
        /// Coordinator's wallet kind.
        wallet_kind: String,
        /// Coordinator's `k`-of-`n`.
        threshold: Option<(u8, u8)>,
    },
    /// Refused (measured, admitted).
    Refused(String),
}

/// One cell of the table, owned; [`render_table`] turns a list of these
/// into `table.rs`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BuiltCell {
    /// SkeletonKey string.
    pub key: String,
    /// Coordinator id.
    pub coordinator: String,
    /// Version.
    pub version: String,
    /// Form.
    pub form: Form,
    /// Outcome.
    pub outcome: BuiltOutcome,
    /// Provenance.
    pub renderer_tool: String,
    /// Provenance.
    pub renderer_version: String,
    /// Provenance.
    pub measured_at: String,
    /// Evidence row id.
    pub source: String,
}

/// Why the build failed on one row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Disagreement {
    /// Rule refuses; evidence imported.
    D1FalseRefusal {
        /// Row.
        row: String,
        /// The rule's class.
        class: &'static str,
    },
    /// Rule admits; evidence refused.
    D2MissedRefusal {
        /// Row.
        row: String,
        /// The coordinator's message.
        message: String,
    },
    /// Both refuse, for different named classes.
    D3ReasonDrift {
        /// Row.
        row: String,
        /// The rule's class.
        rule: &'static str,
        /// The class the message names.
        evidence: &'static str,
    },
    /// No rule set spans the row's version, or the coordinator is unknown.
    D4Orphan {
        /// Row.
        row: String,
    },
    /// Two rows with one key/coordinator/version/form and different outcomes.
    D5EvidenceConflict {
        /// The rows.
        rows: (String, String),
    },
    /// A row that should key did not, or a row declared unkeyable did.
    Keying {
        /// Row.
        row: String,
        /// What happened.
        detail: String,
    },
}

/// The build's result: the stored cells, or every disagreement.
pub struct Built {
    /// Cells, sorted and deduplicated.
    pub cells: Vec<BuiltCell>,
    /// Rows the vendoring script declared unkeyable, confirmed so.
    pub unkeyable: usize,
    /// Rows a rule refused and the evidence agreed (not stored).
    pub agreed_refusals: usize,
}

/// Key one row's descriptor through the ONE key implementation.
fn key_row(row: &EvidenceRow) -> Result<Skeleton, String> {
    let d = descriptor_from_text(&row.descriptor).map_err(|e| e.to_string())?;
    skeleton(&d).map_err(|e| e.to_string())
}

/// Build the table from `rows`, or report every disagreement.
///
/// # Errors
///
/// Every [`Disagreement`] found, in row order.
pub fn build(rows: &[EvidenceRow]) -> Result<Built, Vec<Disagreement>> {
    let mut bad = Vec::new();
    let mut cells: Vec<BuiltCell> = Vec::new();
    let mut unkeyable = 0;
    let mut agreed_refusals = 0;
    for row in rows {
        let keyed = key_row(row);
        let s = match (&row.unkeyable, keyed) {
            (Some(_), Err(_)) => {
                unkeyable += 1;
                continue;
            }
            (Some(why), Ok(_)) => {
                bad.push(Disagreement::Keying {
                    row: row.id.clone(),
                    detail: format!("declared unkeyable ({why}) but it keys"),
                });
                continue;
            }
            (None, Err(e)) => {
                bad.push(Disagreement::Keying {
                    row: row.id.clone(),
                    detail: e,
                });
                continue;
            }
            (None, Ok(s)) => s,
        };
        let Some(c) = REGISTRY.iter().find(|c| c.id.0 == row.coordinator) else {
            bad.push(Disagreement::D4Orphan {
                row: row.id.clone(),
            });
            continue;
        };
        let Some(rs) = c.rule_set_at(&row.version) else {
            bad.push(Disagreement::D4Orphan {
                row: row.id.clone(),
            });
            continue;
        };
        // A spelling refusal comes FIRST: Core parses key expressions before
        // it looks at the script, so on 24.2-25.2 a multipath `tr` miniscript
        // is refused for its `<0;1>`, not for its miniscript (a D3 this
        // build found; recorded in plan 1b).
        let rule = rs
            .form_refusals
            .iter()
            .find(|f| f.form == row.form)
            .map(|f| f.reason)
            .or_else(|| first_refusal(rs, &s));
        let outcome = match (&row.outcome, rule) {
            (MeasuredOutcome::Refused(msg), Some(r)) => {
                if let Some(named) = (c.class_of_message)(msg) {
                    if named != r.class {
                        bad.push(Disagreement::D3ReasonDrift {
                            row: row.id.clone(),
                            rule: r.class,
                            evidence: named,
                        });
                    }
                }
                agreed_refusals += 1;
                continue;
            }
            (MeasuredOutcome::Refused(msg), None) => {
                if !rs.measured_refusals_admitted {
                    bad.push(Disagreement::D2MissedRefusal {
                        row: row.id.clone(),
                        message: msg.clone(),
                    });
                    continue;
                }
                BuiltOutcome::Refused(msg.clone())
            }
            (MeasuredOutcome::Imported(_), Some(r)) => {
                bad.push(Disagreement::D1FalseRefusal {
                    row: row.id.clone(),
                    class: r.class,
                });
                continue;
            }
            (MeasuredOutcome::Imported(None), None) => BuiltOutcome::Imported,
            (MeasuredOutcome::Imported(Some(d)), None) => {
                // The static `Description` borrows; the check needs only a view.
                let view = super::Description {
                    wallet_kind: leak_free(&d.wallet_kind),
                    threshold: d.threshold,
                };
                if (c.reads_as_built)(&s, &view) {
                    BuiltOutcome::Imported
                } else {
                    BuiltOutcome::ImportedAltered {
                        wallet_kind: d.wallet_kind.clone(),
                        threshold: d.threshold,
                    }
                }
            }
        };
        cells.push(BuiltCell {
            key: skeleton_key(&s).as_str().to_string(),
            coordinator: row.coordinator.clone(),
            version: row.version.clone(),
            form: row.form,
            outcome,
            renderer_tool: row.renderer_tool.clone(),
            renderer_version: row.renderer_version.clone(),
            measured_at: row.measured_at.clone(),
            source: row.id.clone(),
        });
    }
    cells.sort();
    // D5, and dedup: one cell per (key, coordinator, version, form). Two rows
    // agreeing on the outcome keep the first as the cited source.
    let mut kept: Vec<BuiltCell> = Vec::new();
    for cell in cells {
        match kept.last() {
            Some(prev)
                if prev.key == cell.key
                    && prev.coordinator == cell.coordinator
                    && prev.version == cell.version
                    && prev.form == cell.form =>
            {
                if prev.outcome != cell.outcome {
                    bad.push(Disagreement::D5EvidenceConflict {
                        rows: (prev.source.clone(), cell.source.clone()),
                    });
                }
            }
            _ => kept.push(cell),
        }
    }
    if bad.is_empty() {
        Ok(Built {
            cells: kept,
            unkeyable,
            agreed_refusals,
        })
    } else {
        Err(bad)
    }
}

/// `reads_as_built` takes a `Description` of `&'static str`s because the
/// runtime table is static. The build only needs to compare, so it matches
/// the known wallet kinds back to a static spelling, and anything else to a
/// sentinel no coordinator treats as "as built".
fn leak_free(kind: &str) -> &'static str {
    const KNOWN: &[&str] = &[
        "MULTI_SIG",
        "SINGLE_SIG",
        "MINISCRIPT",
        "Liana",
        "ADDRESSES_DIFFER",
    ];
    KNOWN
        .iter()
        .find(|k| **k == kind)
        .copied()
        .unwrap_or("UNRECOGNISED")
}

/// Render `cells` as the Rust source of `coordinator/table.rs`. Byte-stable:
/// the same cells always render the same text, which is what lets
/// `cargo xtask verdicts --check` and the xtask's own test detect a stale
/// table.
pub fn render_table(cells: &[BuiltCell], evidence_path: &str) -> String {
    let mut out = String::new();
    out.push_str("//! GENERATED by `cargo xtask verdicts` from\n");
    out.push_str(&format!("//! `{evidence_path}`. Do not edit.\n\n"));
    out.push_str("#[allow(unused_imports)]\n");
    out.push_str("use super::{Cell, CellOutcome, Description, Form};\n\n");
    out.push_str("pub(super) static CELLS: &[Cell] = &[\n");
    for c in cells {
        let form = match c.form {
            Form::Multipath => "Form::Multipath",
            Form::Chain0 => "Form::Chain0",
            Form::Chain1 => "Form::Chain1",
        };
        let outcome = match &c.outcome {
            BuiltOutcome::Imported => "CellOutcome::Imported".to_string(),
            BuiltOutcome::ImportedAltered {
                wallet_kind,
                threshold,
            } => format!(
                "CellOutcome::ImportedAltered(Description {{ wallet_kind: {wallet_kind:?}, threshold: {threshold:?} }})"
            ),
            BuiltOutcome::Refused(m) => format!("CellOutcome::Refused({m:?})"),
        };
        out.push_str(&format!(
            "    Cell {{ key: {:?}, coordinator: {:?}, version: {:?}, form: {form}, outcome: {outcome}, renderer_tool: {:?}, renderer_version: {:?}, measured_at: {:?}, source: {:?} }},\n",
            c.key, c.coordinator, c.version, c.renderer_tool, c.renderer_version, c.measured_at, c.source
        ));
    }
    out.push_str("];\n");
    out
}
```

- [ ] **Step 4: The xtask.**

Modify `Cargo.toml`:

```diff
diff --git a/Cargo.toml b/Cargo.toml
index 6cf0586..8f926e8 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -1,6 +1,6 @@
 [workspace]
 resolver = "3"
-members = ["crates/md-codec", "crates/md-cli"]
+members = ["crates/md-codec", "crates/md-cli", "crates/xtask"]
 
 [workspace.package]
 edition = "2024"
```

Modify `.cargo/config.toml`:

```diff
diff --git a/.cargo/config.toml b/.cargo/config.toml
index 5cda681..171ac5d 100644
--- a/.cargo/config.toml
+++ b/.cargo/config.toml
@@ -17,3 +17,8 @@ codegen-units = 1
 # Reduces binary size and removes a source of build-environment leakage
 # (debug info embeds local paths and toolchain build IDs).
 strip = "symbols"
+
+[alias]
+# `cargo xtask verdicts [--check]` -- builds md-codec's coordinator verdict
+# table from the vendored evidence (coordinator-compat plan 1b).
+xtask = "run --locked --package xtask --"
```

Create `crates/xtask/Cargo.toml`:

```toml
[package]
name = "xtask"
version = "0.0.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish = false
description = "Repository automation: `cargo xtask verdicts` builds md-codec's coordinator verdict table."

[lints]
workspace = true

[dependencies]
md-codec = { path = "../md-codec" }
serde_json = "1"
```

Create `crates/xtask/src/main.rs`:

```rust
//! `cargo xtask verdicts [--check]` — build md-codec's coordinator verdict
//! table (`crates/md-codec/src/coordinator/table.rs`) from the vendored
//! evidence (`crates/md-codec/tests/fixtures/coordinator/evidence.jsonl`).
//!
//! This is design §5 step 1's table generator. It parses JSON and writes a
//! file; everything that decides a verdict — keying, the rules, D1-D5 — is
//! md-codec's own `coordinator::build`, so there is one implementation of the
//! key and one of the rules. `--check` writes nothing and exits 1 when the
//! committed table is not what the evidence builds; `table_is_fresh` below
//! runs the same comparison inside the test suite.

use std::path::PathBuf;
use std::process::ExitCode;

use md_codec::coordinator::Form;
use md_codec::coordinator::build::{
    Disagreement, EvidenceRow, MeasuredOutcome, OwnedDescription, build, render_table,
};

const EVIDENCE: &str = "crates/md-codec/tests/fixtures/coordinator/evidence.jsonl";
const TABLE: &str = "crates/md-codec/src/coordinator/table.rs";

fn root() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

fn field<'a>(v: &'a serde_json::Value, k: &str, line: usize) -> Result<&'a str, String> {
    v[k].as_str()
        .ok_or_else(|| format!("{EVIDENCE}:{line}: missing string field {k:?}"))
}

fn description(v: &serde_json::Value) -> Option<OwnedDescription> {
    if v.is_null() {
        return None;
    }
    let threshold = v["threshold"].as_array().and_then(|a| {
        let k = u8::try_from(a.first()?.as_u64()?).ok()?;
        let n = u8::try_from(a.get(1)?.as_u64()?).ok()?;
        Some((k, n))
    });
    Some(OwnedDescription {
        wallet_kind: v["wallet_kind"].as_str().unwrap_or("").to_string(),
        threshold,
    })
}

/// Parse the vendored JSONL into rows.
fn parse(text: &str) -> Result<Vec<EvidenceRow>, String> {
    let mut rows = Vec::new();
    for (i, line) in text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
    {
        let n = i + 1;
        let v: serde_json::Value =
            serde_json::from_str(line).map_err(|e| format!("{EVIDENCE}:{n}: {e}"))?;
        let form = match field(&v, "form", n)? {
            "multipath" => Form::Multipath,
            "chain0" => Form::Chain0,
            "chain1" => Form::Chain1,
            other => return Err(format!("{EVIDENCE}:{n}: unknown form {other:?}")),
        };
        let o = &v["outcome"];
        let outcome = if let Some(m) = o["refused"].as_str() {
            MeasuredOutcome::Refused(m.to_string())
        } else if o.get("imported").is_some() {
            MeasuredOutcome::Imported(description(&o["imported"]))
        } else {
            return Err(format!(
                "{EVIDENCE}:{n}: outcome is neither imported nor refused"
            ));
        };
        rows.push(EvidenceRow {
            id: field(&v, "id", n)?.to_string(),
            coordinator: field(&v, "coordinator", n)?.to_string(),
            version: field(&v, "version", n)?.to_string(),
            library_rev: field(&v, "library_rev", n)?.to_string(),
            renderer_tool: field(&v, "renderer_tool", n)?.to_string(),
            renderer_version: field(&v, "renderer_version", n)?.to_string(),
            form,
            measured_at: field(&v, "measured_at", n)?.to_string(),
            descriptor: field(&v, "descriptor", n)?.to_string(),
            outcome,
            unkeyable: v["unkeyable"].as_str().map(str::to_string),
        });
    }
    Ok(rows)
}

/// The table the committed evidence builds, or why it cannot be built.
fn generate() -> Result<(String, String), String> {
    let text = std::fs::read_to_string(root().join(EVIDENCE))
        .map_err(|e| format!("read {EVIDENCE}: {e}"))?;
    let rows = parse(&text)?;
    match build(&rows) {
        Ok(built) => Ok((
            render_table(&built.cells, EVIDENCE),
            format!(
                "{} rows: {} cells stored, {} refusals the rules explain, {} unkeyable",
                rows.len(),
                built.cells.len(),
                built.agreed_refusals,
                built.unkeyable
            ),
        )),
        Err(bad) => {
            let mut msg = format!(
                "{} disagreement(s) between rules and evidence:\n",
                bad.len()
            );
            for d in &bad {
                msg.push_str(&format!("  {}\n", describe(d)));
            }
            Err(msg)
        }
    }
}

fn describe(d: &Disagreement) -> String {
    match d {
        Disagreement::D1FalseRefusal { row, class } => {
            format!("D1 false-refusal {row}: rule refuses ({class}), evidence imported")
        }
        Disagreement::D2MissedRefusal { row, message } => {
            format!("D2 missed-refusal {row}: rule admits, evidence refused: {message}")
        }
        Disagreement::D3ReasonDrift {
            row,
            rule,
            evidence,
        } => format!("D3 reason-drift {row}: rule says {rule}, message says {evidence}"),
        Disagreement::D4Orphan { row } => format!("D4 orphan {row}: no rule set spans it"),
        Disagreement::D5EvidenceConflict { rows } => {
            format!("D5 evidence-conflict {} vs {}", rows.0, rows.1)
        }
        Disagreement::Keying { row, detail } => format!("keying {row}: {detail}"),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["verdicts"] => false,
        ["verdicts", "--check"] => true,
        _ => {
            eprintln!("usage: cargo xtask verdicts [--check]");
            return ExitCode::from(2);
        }
    };
    let (table, summary) = match generate() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("xtask verdicts: {e}");
            return ExitCode::FAILURE;
        }
    };
    let path = root().join(TABLE);
    if check {
        let committed = std::fs::read_to_string(&path).unwrap_or_default();
        if committed != table {
            eprintln!("xtask verdicts: {TABLE} is stale; run `cargo xtask verdicts`");
            return ExitCode::FAILURE;
        }
        eprintln!("xtask verdicts: fresh ({summary})");
        return ExitCode::SUCCESS;
    }
    if let Err(e) = std::fs::write(&path, table) {
        eprintln!("xtask verdicts: write {TABLE}: {e}");
        return ExitCode::FAILURE;
    }
    eprintln!("xtask verdicts: wrote {TABLE} ({summary})");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The committed table is exactly what the committed evidence builds —
    /// so an evidence change without a regenerated table, a rule change that
    /// moves a cell, or a hand edit to `table.rs` is red in `cargo test`, not
    /// only in `--check`. Mutation: delete any one `Cell` line from
    /// `table.rs` -> red.
    #[test]
    fn table_is_fresh() {
        let (table, _) = generate().unwrap_or_else(|e| panic!("{e}"));
        let committed = std::fs::read_to_string(root().join(TABLE)).unwrap();
        assert!(
            committed == table,
            "{TABLE} is stale; run `cargo xtask verdicts`"
        );
    }
}
```

- [ ] **Step 5: Generate.**

```sh
cargo update --workspace --offline      # adds the xtask package to Cargo.lock; nothing else moves
cargo xtask verdicts
# expect: wrote crates/md-codec/src/coordinator/table.rs
#         (526 rows: 228 cells stored, 204 refusals the rules explain, 2 unkeyable)
cargo xtask verdicts --check            # expect: fresh (...)
```

  **If the build reports a disagreement instead, STOP and report it** — it
  is a finding about a rule or a row, resolved by a person (design §2), never
  by editing `table.rs` or dropping a row to get green.

- [ ] **Step 6:** PASS; the gate. Commit
  `coordinator: the rule/evidence table build and cargo xtask verdicts`.

---

### Task 7: `md shape-key`

**Files:** Create `crates/md-cli/src/cmd/shape_key.rs`,
`crates/md-cli/tests/cli_coordinator_verdict.rs`; Modify
`crates/md-cli/src/cmd/mod.rs`, `crates/md-cli/src/main.rs`.

- [ ] **Step 1: The tests** — agreement with the library on every distinct
  keyable evidence descriptor (73), card vs descriptor, and the single-chain
  refusal. Mutation (measured, reds the first): print the key with its
  U+001F separators replaced by spaces.

Create `crates/md-cli/tests/cli_coordinator_verdict.rs`:

```rust
//! Coordinator-compat plan 1b, the host half (design §4, "`md` on the host"):
//! `md shape-key`, and the verdict notice on `md compose` and `md descriptor`.
//!
//! Every test names the mutation that reds it.

#![allow(missing_docs)]

use std::collections::BTreeSet;
use std::process::Command as StdCommand;

use md_codec::chunk::split;
use md_codec::descriptor_route::{descriptor_from_text, skeleton_key_of_text};

/// `(stdout, stderr, exit code)`; the same helper `cli_compose_unspendable.rs`
/// and `liana_input_side.rs` define.
fn md(args: &[&str]) -> (String, String, i32) {
    let out = StdCommand::new(assert_cmd::cargo::cargo_bin("md"))
        .args(args)
        .output()
        .expect("invoke md");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().expect("md exited normally"),
    )
}

fn evidence() -> Vec<serde_json::Value> {
    let p = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../md-codec/tests/fixtures/coordinator/evidence.jsonl"
    );
    std::fs::read_to_string(p)
        .unwrap_or_else(|e| panic!("{p}: {e}"))
        .lines()
        .map(|l| serde_json::from_str(l).expect("row parses"))
        .collect()
}

fn descriptor_named(name: &str) -> String {
    evidence()
        .into_iter()
        .find(|r| r["name"] == name)
        .unwrap_or_else(|| panic!("no evidence row named {name}"))["descriptor"]
        .as_str()
        .unwrap()
        .to_string()
}

/// The md1 chunks of a keyed card, minted by the library from a descriptor.
fn card_of(descriptor: &str) -> Vec<String> {
    split(&descriptor_from_text(descriptor).unwrap()).unwrap()
}

/// Design §5 step 1: "`md shape-key` is a thin CLI over the same library
/// function, and a test asserts the two agree" -- over every distinct
/// keyable evidence descriptor, i.e. every key the table was built from.
/// Mutation: print the key with its U+001F separators replaced by spaces ->
/// every row reds.
#[test]
fn shape_key_agrees_with_the_library_on_every_evidence_descriptor() {
    let descriptors: BTreeSet<String> = evidence()
        .into_iter()
        .filter(|r| r.get("unkeyable").is_none())
        .map(|r| r["descriptor"].as_str().unwrap().to_string())
        .collect();
    assert!(descriptors.len() >= 73, "{} descriptors", descriptors.len());
    for d in &descriptors {
        let (out, err, code) = md(&["shape-key", "--descriptor", d]);
        assert_eq!(code, 0, "{err}");
        let want = skeleton_key_of_text(d).unwrap();
        assert_eq!(out.strip_suffix('\n'), Some(want.as_str()), "{d}");
    }
}

/// A card and its descriptor are one key. Mutation: key the card through
/// `DecodeOpts::partial()` with its TLV dropped -> the partitions render
/// empty and this reds.
#[test]
fn shape_key_of_a_card_equals_the_key_of_its_descriptor() {
    for name in [
        "CONTROL-accept-kofn-recovery-flat",
        "X24-wsh-2of3-1of1-unlocked-plus-rec",
    ] {
        let d = descriptor_named(name);
        let card = card_of(&d);
        let mut args = vec!["shape-key"];
        args.extend(card.iter().map(String::as_str));
        let (from_card, err, code) = md(&args);
        assert_eq!(code, 0, "{name}: {err}");
        let (from_text, _, _) = md(&["shape-key", "--descriptor", &d]);
        assert_eq!(from_card, from_text, "{name}");
    }
}

/// A single-chain descriptor cannot be keyed: the key keeps `<0;1>`.
/// Mutation: accept single-chain text by guessing `<0;1>` -> exit 0, red.
#[test]
fn shape_key_refuses_a_single_chain_descriptor() {
    let d = descriptor_named("X24-wsh-2of3-1of1-unlocked-plus-rec").replace("<0;1>", "0");
    let d = d.split('#').next().unwrap().to_string();
    let (out, err, code) = md(&["shape-key", "--descriptor", &d]);
    assert_ne!(code, 0, "{out}");
    assert!(err.contains("multipath"), "{err}");
}
```

- [ ] **Step 2:** run — FAIL (no subcommand). **Step 3:**

Modify `crates/md-cli/src/cmd/mod.rs`:

```diff
diff --git a/crates/md-cli/src/cmd/mod.rs b/crates/md-cli/src/cmd/mod.rs
index 0a1da46..9820503 100644
--- a/crates/md-cli/src/cmd/mod.rs
+++ b/crates/md-cli/src/cmd/mod.rs
@@ -159,5 +159,6 @@ pub mod gui_schema;
 pub mod inspect;
 pub mod partial;
 pub mod repair;
+pub mod shape_key;
 pub mod vectors;
 pub mod verify;
```

Modify `crates/md-cli/src/main.rs`:

```diff
diff --git a/crates/md-cli/src/main.rs b/crates/md-cli/src/main.rs
index 216aa98..7a84fac 100644
--- a/crates/md-cli/src/main.rs
+++ b/crates/md-cli/src/main.rs
@@ -323,6 +323,18 @@ enum Command {
         #[arg(long, value_name = "KIND")]
         unspendable: Option<String>,
     },
+    /// Print a policy's shape key: the canonical, coordinator-independent
+    /// summary the coordinator verdict table is keyed by (U+001F-separated:
+    /// template, partitions, key-path kind).
+    #[command(group = clap::ArgGroup::new("shape_key_input").required(true).args(["phrases", "descriptor"]))]
+    ShapeKey {
+        /// One or more md1 strings of one card.
+        #[arg(num_args = 0.., conflicts_with = "descriptor")]
+        phrases: Vec<String>,
+        /// A multipath (`<0;1>`) BIP-380 descriptor instead of a card.
+        #[arg(long, value_name = "DESCRIPTOR")]
+        descriptor: Option<String>,
+    },
     /// Emit the CONCRETE output descriptor -- real keys, key origins and the
     /// BIP-380 checksum -- for pasting into a coordinator.
     ///
@@ -1071,6 +1083,10 @@ fn dispatch(c: Command) -> Result<u8, CliError> {
             json,
             unspendable.as_deref(),
         ),
+        Command::ShapeKey {
+            phrases,
+            descriptor,
+        } => cmd::shape_key::run(&phrases, descriptor.as_deref()),
         Command::Descriptor {
             phrases,
             template,
```

Create `crates/md-cli/src/cmd/shape_key.rs`:

```rust
//! `md shape-key` -- print a policy's `SkeletonKey` (coordinator-compat plan
//! 1b; design §2, "the shape key has exactly one implementation").
//!
//! A thin CLI over md-codec: an md1 card goes through `reassemble` and
//! `skeleton`, and `--descriptor` goes through
//! `md_codec::descriptor_route::skeleton_key_of_text` -- the same function
//! `cargo xtask verdicts` keys the evidence with. Nothing here computes a key.
//! `tests/cli_shape_key.rs` asserts the CLI and the library agree on every
//! vendored evidence row.

use crate::error::CliError;
use md_codec::chunk::reassemble;
use md_codec::decode::decode_md1_string;
use md_codec::skeleton::{skeleton, skeleton_key};

pub fn run(phrases: &[String], descriptor: Option<&str>) -> Result<u8, CliError> {
    let key = match descriptor {
        Some(text) => md_codec::descriptor_route::skeleton_key_of_text(text)
            .map_err(|e| CliError::BadArg(format!("shape-key: {e}")))?,
        None => {
            let strings = crate::cmd::strip_md1_inputs(phrases);
            let d = if strings.len() == 1 {
                decode_md1_string(&strings[0])?
            } else {
                let refs: Vec<&str> = strings.iter().map(String::as_str).collect();
                reassemble(&refs)?
            };
            // Design §1A (a3): a card whose keys will not expand has no key.
            let s = skeleton(&d).map_err(|e| CliError::BadArg(format!("shape-key: {e}")))?;
            skeleton_key(&s)
        }
    };
    // The key's own spelling, U+001F separators included (design §1A): this
    // is the string the evidence table is keyed by, byte for byte.
    println!("{}", key.as_str());
    Ok(0)
}
```

- [ ] **Step 4:** PASS; the gate. Commit `md shape-key: print a policy's SkeletonKey`.

---

### Task 8: The verdict on `md compose` and `md descriptor`; the none case; replacing `liana_refuse_or_warn`

`md compose` mints, so it refuses the none case (every coordinator refuses at
every verified version), exits 1, prints nothing on stdout and names
`--md-only`. `md descriptor` reads, so it only prints (design §4, r2 C-4).
`liana_refuse_or_warn`'s two Liana warnings are DELETED, not supplemented:
the verdict names Liana's class for every shape, flag or no flag. What
remains of it (`unspendable_liana_checks`) is md's own: SPEC §6's refusal and
the "has no effect" warning. This closes F-644's md-cli half.

**Files:** Create `crates/md-cli/src/cmd/verdict.rs`; Modify
`crates/md-cli/src/cmd/{mod,compose,descriptor}.rs`, `crates/md-cli/src/main.rs`,
`crates/md-cli/README.md`, and the tests `cli_coordinator_verdict.rs`,
`cli_compose_unspendable.rs`, `cli_compose.rs`,
`keyless_hashlock_reaches_a_descriptor.rs`.

- [ ] **Step 1: The new tests.** Mutations (measured unless noted): remove the
  `verdict::notice` call from compose (F-644 test reds); drop `&& !md_only`
  (none-case test reds); drop the `KeysAbsent` return (the template-verdict
  test reds — measured on its md-codec twin in Task 6, not here);
  route `md descriptor` through a none-case refusal (not measured).

Modify `crates/md-cli/tests/cli_coordinator_verdict.rs`:

```diff
diff --git a/crates/md-cli/tests/cli_coordinator_verdict.rs b/crates/md-cli/tests/cli_coordinator_verdict.rs
index 9713ead..a3e15b2 100644
--- a/crates/md-cli/tests/cli_coordinator_verdict.rs
+++ b/crates/md-cli/tests/cli_coordinator_verdict.rs
@@ -103,3 +103,127 @@ fn shape_key_refuses_a_single_chain_descriptor() {
     assert_ne!(code, 0, "{out}");
     assert!(err.contains("multipath"), "{err}");
 }
+
+// ---------------------------------------------------------------------------
+// The verdict on `md compose` and `md descriptor` (Task 8).
+// ---------------------------------------------------------------------------
+
+const H: &str = "a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8";
+
+/// F-644's md-cli half, re-owned by plan 1b: each shape it names now carries
+/// Liana's refusal, keyed on the composed shape, with or without the flag.
+/// Mutation: remove the `verdict::notice` call from `compose::run` -> red.
+#[test]
+fn compose_names_liana_refusal_for_every_f644_shape() {
+    for spelling in [
+        &["--path", "2of3,unsorted", "--path", "2of2"][..],
+        &["--path", "2of2", "--path", "1of1,after=800000"][..],
+        &[
+            "--path",
+            "2of2",
+            "--path",
+            "1of1,older=100",
+            "--path",
+            "1of1,older=100",
+        ][..],
+        &["--path", "2of3,unsorted", "--experimental"][..],
+    ] {
+        for flag in [&["--unspendable", "liana"][..], &[][..]] {
+            let mut args = vec!["compose", "--wrapper", "tr"];
+            args.extend_from_slice(spelling);
+            args.extend_from_slice(flag);
+            let (out, err, code) = md(&args);
+            assert_eq!(code, 0, "{args:?}: {err}");
+            assert!(!out.is_empty(), "{args:?}");
+            assert!(err.contains("Liana 8.0-15.0: refuses ("), "{args:?}: {err}");
+        }
+    }
+}
+
+/// A template never claims an import (design §1 (a2)); Core's structural
+/// refusal before 26.0 still prints. Mutation: drop the `KeysAbsent` return
+/// in `md_codec::coordinator::at_version` -> the lines change and this reds.
+#[test]
+fn compose_prints_refusals_and_silences_never_imports() {
+    let (_, err, code) = md(&[
+        "compose",
+        "--wrapper",
+        "tr",
+        "--preset",
+        "kofn-recovery,2of3,older=26280",
+    ]);
+    assert_eq!(code, 0, "{err}");
+    assert!(
+        err.contains("Bitcoin Core 24.2-25.2: refuses (miniscript under tr)"),
+        "{err}"
+    );
+    assert!(
+        err.contains("Bitcoin Core 26.0-31.1: unproven (a template has no keys"),
+        "{err}"
+    );
+    assert!(!err.contains(": imports"), "{err}");
+}
+
+/// Ruling 2 and design §4: compose refuses the none case, exits non-zero,
+/// names `--md-only`, and emits nothing; `--md-only` proceeds. Mutation:
+/// drop `&& !md_only` from the refusal -> the second half reds.
+#[test]
+fn compose_refuses_the_none_case_and_md_only_proceeds() {
+    let keyless = format!("keyless,sha256={H}");
+    let base = [
+        "compose",
+        "--wrapper",
+        "wsh",
+        "--path",
+        "2of3",
+        "--path",
+        &keyless,
+        "--experimental",
+    ];
+    let (out, err, code) = md(&base);
+    assert_eq!(code, 1, "{err}");
+    assert!(out.is_empty(), "a refusal printed a template: {out}");
+    assert!(err.contains("--md-only"), "{err}");
+
+    let mut args = base.to_vec();
+    args.push("--md-only");
+    let (out, err, code) = md(&args);
+    assert_eq!(code, 0, "{err}");
+    assert!(out.starts_with("wsh("), "{out}");
+}
+
+/// `md descriptor` READS a card, so it never refuses on a verdict -- not
+/// even the none case -- and it names the spelling it printed. Mutation:
+/// route `md descriptor` through compose's none-case refusal -> the keyless
+/// card exits 1 and this reds.
+#[test]
+fn descriptor_names_the_form_and_never_refuses() {
+    let card = card_of(&descriptor_named("CONTROL-accept-kofn-recovery-flat"));
+    let mut args = vec!["descriptor"];
+    args.extend(card.iter().map(String::as_str));
+    let (_, err, code) = md(&args);
+    assert_eq!(code, 0, "{err}");
+    assert!(err.contains("multipath form"), "{err}");
+    assert!(
+        err.contains("Bitcoin Core 24.2-28.4: refuses (the <0;1> multipath spelling"),
+        "{err}"
+    );
+    args.extend(["--chain", "0"]);
+    let (_, err, code) = md(&args);
+    assert_eq!(code, 0, "{err}");
+    assert!(
+        err.contains("Bitcoin Core 26.0-31.1: imports the chain0 form"),
+        "{err}"
+    );
+
+    let card = card_of(&descriptor_named("keyless-hash-path-wsh"));
+    let mut args = vec!["descriptor", "--experimental"];
+    args.extend(card.iter().map(String::as_str));
+    let (out, err, code) = md(&args);
+    assert_eq!(code, 0, "{err}");
+    assert!(out.starts_with("wsh("), "{out}");
+    assert!(
+        err.contains("Liana 8.0-15.0: refuses (a path with no key)"),
+        "{err}"
+    );
+}
```

- [ ] **Step 2: The retired warning's tests become verdict tests** — the
  Liana class now prints with and without the flag (NUMS first without it:
  Liana's class 2), and the retired text must never print. Mutation: restore
  the hashlock warning → the "never prints" assertion reds (not measured).

Modify `crates/md-cli/tests/cli_compose_unspendable.rs`:

```diff
diff --git a/crates/md-cli/tests/cli_compose_unspendable.rs b/crates/md-cli/tests/cli_compose_unspendable.rs
index 7853ae1..6c80f3e 100644
--- a/crates/md-cli/tests/cli_compose_unspendable.rs
+++ b/crates/md-cli/tests/cli_compose_unspendable.rs
@@ -274,15 +274,20 @@ fn compose_json_carries_the_third_state() {
 //
 // | case                                   | authority          | action |
 // | §6 refuses the composed shape          | md's own rule      | REFUSE |
-// | md-legal, but Liana will not import it | Liana's policy     | WARN   |
+// | md-legal, but Liana will not import it | Liana's policy     | the coordinator verdict (plan 1b) |
 // | a real internal key was extracted      | SPEC §6 row 3      | WARN   |
+//
+// Coordinator-compat plan 1b REPLACED the second row's hand-written warning
+// with the verdict every compose prints from md-codec's registry.
 // ---------------------------------------------------------------------------
 
 const H: &str = "a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8";
-/// Stable prefixes of the two warnings, so each test can count them.
-const WARN_POLICY: &str =
-    "warning: --unspendable liana: Liana is not expected to import this wallet";
+/// The warning plan 1b retired; it must never print again.
+const RETIRED_WARN: &str = "Liana is not expected to import this wallet";
 const WARN_NO_EFFECT: &str = "warning: --unspendable liana has no effect";
+/// The registry's Liana refusal line, as `md_codec::coordinator::describe`
+/// spells it for a template.
+const LIANA_REFUSES: &str = "Liana 8.0-15.0: refuses (";
 /// SPEC §6 row 1's message as md-codec ships it (`Error::UnspendableWithSortedMultiA`)
 /// -- asserted by substring so a re-worded copy in md-cli fails here.
 const SORTEDMULTI_A_REFUSAL: &str =
@@ -324,59 +329,64 @@ fn liana_refuses_a_sortedmulti_a_leaf_and_the_default_still_composes() {
     }
 }
 
-/// Count occurrences of each warning in `err`.
-fn warns(err: &str) -> (usize, usize) {
-    (
-        err.matches(WARN_POLICY).count(),
-        err.matches(WARN_NO_EFFECT).count(),
-    )
-}
-
-/// md-legal, but outside Liana's policy model: WARN (not refuse), keyed on
-/// the composed SHAPE so a `--path`-built equivalent warns too (R1 I-f).
+/// md-legal, but outside Liana's policy model: the REGISTRY's verdict names
+/// Liana's refusal, keyed on the composed SHAPE (R1 I-f) and now independent
+/// of the flag -- the old warning fired only under `--unspendable liana`.
+/// Mutation: restore `liana_refuse_or_warn`'s hashlock warning -> the
+/// retired text prints and this reds.
 #[test]
-fn liana_warns_on_shapes_outside_liana_policy_by_shape_not_name() {
+fn liana_policy_verdict_comes_from_the_registry_by_shape_not_name() {
     let hash_path = format!("1of1,sha256={H},older=100");
     let hashlock_preset = format!("hashlock-gated,sha256={H},older=26280");
-    let cases: [(&str, Vec<&str>); 4] = [
-        ("hashlock preset", vec!["--preset", &hashlock_preset]),
+    let cases: [(&str, Vec<&str>, &str); 4] = [
+        (
+            "hashlock preset",
+            vec!["--preset", &hashlock_preset],
+            "a hash lock",
+        ),
         (
             "hashlock --path",
             vec!["--path", "2of3", "--path", &hash_path],
+            "a hash lock",
         ),
         (
-            "no-unlocked preset",
+            "decaying preset",
             vec![
                 "--preset",
                 "decaying-multisig,2of2,1of1,older1=13140,older2=26280,after=1000000",
             ],
+            "an absolute lock",
         ),
         (
             "no-unlocked --path",
             vec!["--path", "2of2,older=100", "--path", "1of1,older=200"],
+            "no unlocked path",
         ),
     ];
-    for (what, spelling) in &cases {
-        let mut args = spelling.clone();
-        args.extend_from_slice(&["--unspendable", "liana"]);
-        let (out, err, code) = compose_tr(&args);
-        assert_eq!(
-            code, 0,
-            "{what}: a Liana-policy case warns, never refuses: {err}"
-        );
-        assert!(out.contains("UNSPENDABLE(liana)"), "{what}: {out}");
-        assert_eq!(
-            warns(&err),
-            (1, 0),
-            "{what}: exactly the policy warning: {err}"
-        );
-
-        // Gated on the FLAG: the default composes the same shape silently.
-        let (_, err, code) = compose_tr(spelling);
-        assert_eq!(code, 0, "{what} default: {err}");
-        assert_eq!(warns(&err), (0, 0), "{what}: default must not warn: {err}");
+    for (what, spelling, liana_class) in &cases {
+        // Without the flag the internal key is NUMS, which Liana refuses
+        // first (its class 2) -- the verdict still prints, for its own reason.
+        for (flag, class) in [
+            (&["--unspendable", "liana"][..], *liana_class),
+            (&[][..], "NUMS key path"),
+        ] {
+            let mut args = spelling.clone();
+            args.extend_from_slice(flag);
+            let (out, err, code) = compose_tr(&args);
+            assert_eq!(
+                code, 0,
+                "{what} {flag:?}: a verdict never refuses here: {err}"
+            );
+            assert!(!out.is_empty(), "{what}: {out}");
+            assert!(
+                err.contains(&format!("{LIANA_REFUSES}{class})")),
+                "{what} {flag:?}: {err}"
+            );
+            assert!(!err.contains(RETIRED_WARN), "{what}: {err}");
+        }
     }
-    // The canonical Liana shape warns about nothing.
+    // The canonical Liana shape: no Liana refusal (a template claims no
+    // import either, so Liana is silent).
     let (_, err, code) = compose_tr(&[
         "--preset",
         "kofn-recovery,2of3,older=26280",
@@ -384,19 +394,16 @@ fn liana_warns_on_shapes_outside_liana_policy_by_shape_not_name() {
         "liana",
     ]);
     assert_eq!(code, 0, "{err}");
-    assert_eq!(
-        warns(&err),
-        (0, 0),
+    assert!(
+        !err.contains(LIANA_REFUSES),
         "kofn-recovery is Liana's own shape: {err}"
     );
 }
 
 /// SPEC §6 row 3: a bare single-key path became a REAL internal key, so
-/// `liana` has nothing to choose -- it WARNS, never silently no-ops.
-/// Steps 3 and 4 are exclusive by construction on the "no unlocked path"
-/// half (the key path IS an unlocked path), so the canonical
-/// unlocked-primary + timelocked-recovery shape prints exactly one warning.
-/// The hashlock half is not exclusive (R3 M-5): both fire there.
+/// `liana` has nothing to choose -- it WARNS, never silently no-ops. This
+/// is md's own warning and survives plan 1b. Mutation: delete the
+/// `unspendable_request_unmet` warning -> red.
 #[test]
 fn liana_over_a_real_internal_key_warns_no_effect() {
     let (out, err, code) = compose_tr(&[
@@ -407,34 +414,19 @@ fn liana_over_a_real_internal_key_warns_no_effect() {
     ]);
     assert_eq!(code, 0, "{err}");
     assert!(!out.contains("UNSPENDABLE"), "a real key path: {out}");
-    assert_eq!(warns(&err), (0, 1), "exactly the no-effect warning: {err}");
+    assert_eq!(err.matches(WARN_NO_EFFECT).count(), 1, "{err}");
     assert!(err.contains("path 1"), "names the extracted path: {err}");
 
-    let hash_path = format!("1of1,sha256={H},older=100");
-    let (_, err, code) = compose_tr(&[
-        "--path",
-        "1of1",
-        "--path",
-        &hash_path,
-        "--unspendable",
-        "liana",
-    ]);
-    assert_eq!(code, 0, "{err}");
-    assert_eq!(
-        warns(&err),
-        (1, 1),
-        "both warnings on the hashlock example: {err}"
-    );
-
     // Default: no warning.
     let (_, err, code) = compose_tr(&["--preset", "simple-timelocked-inheritance,older=26280"]);
     assert_eq!(code, 0, "{err}");
-    assert_eq!(warns(&err), (0, 0), "{err}");
+    assert_eq!(err.matches(WARN_NO_EFFECT).count(), 0, "{err}");
 }
 
-/// Step 5: `--json` keeps stdout pure JSON while every warning goes to stderr.
+/// `--json` keeps stdout pure JSON while the warning and the verdict go to
+/// stderr. Mutation: print the verdict to stdout -> the JSON parse reds.
 #[test]
-fn liana_warnings_go_to_stderr_under_json() {
+fn liana_warnings_and_verdict_go_to_stderr_under_json() {
     let hash_path = format!("1of1,sha256={H},older=100");
     let (out, err, code) = compose_tr(&[
         "--path",
@@ -449,9 +441,9 @@ fn liana_warnings_go_to_stderr_under_json() {
     let v: serde_json::Value =
         serde_json::from_str(&out).unwrap_or_else(|e| panic!("stdout not JSON ({e}): {out}"));
     assert_eq!(v["internal_key_path"], 0);
+    assert_eq!(err.matches(WARN_NO_EFFECT).count(), 1, "{err}");
     assert!(
-        !out.contains("warning"),
-        "a warning leaked to stdout: {out}"
+        err.contains(&format!("{LIANA_REFUSES}a hash lock)")),
+        "{err}"
     );
-    assert_eq!(warns(&err), (1, 1), "{err}");
 }
```

- [ ] **Step 3: Four existing tests compose a keyless path**, which every
  coordinator now refuses, so they gain `--md-only` — the behaviour change,
  not a workaround:

Modify `crates/md-cli/tests/cli_compose.rs`:

```diff
diff --git a/crates/md-cli/tests/cli_compose.rs b/crates/md-cli/tests/cli_compose.rs
index 94ce951..7e8f77a 100644
--- a/crates/md-cli/tests/cli_compose.rs
+++ b/crates/md-cli/tests/cli_compose.rs
@@ -105,6 +105,9 @@ fn compose_refuses_a_keyless_path_without_experimental_and_admits_it_with() {
         "--wrapper",
         "wsh",
         "--experimental",
+        // Plan 1b: every coordinator refuses a keyless path, so compose's
+        // none-case stop needs the operator's explicit `--md-only`.
+        "--md-only",
         "--path",
         "2of3",
         "--path",
@@ -239,6 +242,7 @@ fn compose_json_experimental_paths_join_the_slot_map() {
             "wsh",
             "--json",
             "--experimental",
+            "--md-only",
             "--path",
             "2of3",
             "--path",
```

Modify `crates/md-cli/tests/keyless_hashlock_reaches_a_descriptor.rs`:

```diff
diff --git a/crates/md-cli/tests/keyless_hashlock_reaches_a_descriptor.rs b/crates/md-cli/tests/keyless_hashlock_reaches_a_descriptor.rs
index 3f36a6f..18281ac 100644
--- a/crates/md-cli/tests/keyless_hashlock_reaches_a_descriptor.rs
+++ b/crates/md-cli/tests/keyless_hashlock_reaches_a_descriptor.rs
@@ -43,6 +43,8 @@ fn template_for(kind: &str) -> String {
             "--path",
             &format!("keyless,{kind}={h}"),
             "--experimental",
+            // Plan 1b: no coordinator imports a keyless path.
+            "--md-only",
         ])
         .assert()
         .success();
@@ -234,6 +236,7 @@ fn the_keyless_signature_rule_is_what_decompose_keys_on() {
             "--path",
             "keyless,ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b",
             "--experimental",
+            "--md-only",
         ])
         .assert()
         .success();
```

- [ ] **Step 4:** run — FAIL. **Step 5:** the implementation.

Create `crates/md-cli/src/cmd/verdict.rs`:

```rust
//! The coordinator verdict notice `md compose` and `md descriptor` print on
//! stderr (coordinator-compat plan 1b; design §4 "`md` on the host").
//!
//! One copy: every line comes from `md_codec::coordinator::describe` over the
//! generated registry. md-cli holds no coordinator knowledge of its own -- the
//! Liana warnings `liana_refuse_or_warn` used to hand-write are gone.

use md_codec::coordinator::{CoordinatorVerdict, Form, describe, verdicts};
use md_codec::encode::Descriptor;
use md_codec::skeleton::skeleton;

/// Print the notice for `d` and return the verdicts, or `None` when the card
/// has no key at all (design §1A (a3): keys that will not expand, or a walk
/// the classifier does not understand), in which case one line says so and
/// no coordinator row is printed.
pub fn notice(d: &Descriptor, form: Option<Form>, what: &str) -> Option<Vec<CoordinatorVerdict>> {
    let s = match skeleton(d) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("note: coordinators: no verdict ({e})");
            return None;
        }
    };
    let vs = verdicts(&s, form);
    eprintln!("note: coordinators for {what} (verified versions only; newer: unmeasured):");
    for v in &vs {
        eprintln!("  {}", describe(v));
    }
    Some(vs)
}
```

Modify `crates/md-cli/src/cmd/mod.rs`:

```diff
diff --git a/crates/md-cli/src/cmd/mod.rs b/crates/md-cli/src/cmd/mod.rs
index 9820503..f1a5179 100644
--- a/crates/md-cli/src/cmd/mod.rs
+++ b/crates/md-cli/src/cmd/mod.rs
@@ -161,4 +161,5 @@ pub mod partial;
 pub mod repair;
 pub mod shape_key;
 pub mod vectors;
+pub mod verdict;
 pub mod verify;
```

Modify `crates/md-cli/src/main.rs`:

```diff
diff --git a/crates/md-cli/src/main.rs b/crates/md-cli/src/main.rs
index 7a84fac..d8fcd9b 100644
--- a/crates/md-cli/src/main.rs
+++ b/crates/md-cli/src/main.rs
@@ -322,6 +322,11 @@ enum Command {
         // call site.
         #[arg(long, value_name = "KIND")]
         unspendable: Option<String>,
+        /// Compose even when every wallet coordinator md knows refuses the
+        /// policy at every verified version. md can still rebuild such a
+        /// wallet from the card; md cannot sign.
+        #[arg(long)]
+        md_only: bool,
     },
     /// Print a policy's shape key: the canonical, coordinator-independent
     /// summary the coordinator verdict table is keyed by (U+001F-separated:
@@ -1075,6 +1080,7 @@ fn dispatch(c: Command) -> Result<u8, CliError> {
             experimental,
             json,
             unspendable,
+            md_only,
         } => cmd::compose::run(
             &wrapper,
             &paths,
@@ -1082,6 +1088,7 @@ fn dispatch(c: Command) -> Result<u8, CliError> {
             experimental,
             json,
             unspendable.as_deref(),
+            md_only,
         ),
         Command::ShapeKey {
             phrases,
```

Modify `crates/md-cli/src/cmd/compose.rs`:

```diff
diff --git a/crates/md-cli/src/cmd/compose.rs b/crates/md-cli/src/cmd/compose.rs
index fa51515..d21bbf1 100644
--- a/crates/md-cli/src/cmd/compose.rs
+++ b/crates/md-cli/src/cmd/compose.rs
@@ -599,62 +599,27 @@ fn describe(e: &Experimental) -> String {
     }
 }
 
-/// F-449 stage 2 Task 2: what `--unspendable liana` does when it cannot yield
-/// an importable wallet. Called ONLY under that flag -- gate every new
-/// refusal on the FLAG, never on the shape, or the default compose of these
-/// same shapes (and the vendored vectors) goes red.
+/// F-449 stage 2 Task 2, as narrowed by coordinator-compat plan 1b: what
+/// `--unspendable liana` does that is md's OWN business. Called ONLY under
+/// that flag -- gate every refusal on the FLAG, never on the shape, or the
+/// default compose of these same shapes goes red.
 ///
-/// Split by AUTHORITY (R0 C-1/I-6):
-/// - **SPEC §6 is md's own rule → REFUSE**, through the same
+/// - **SPEC §6 is md's own rule -> REFUSE**, through the same
 ///   `validate_unspendable_shape` `md encode` runs, so its message is not
-///   re-worded here. Not a second implementation: one home, two callers --
-///   the F-600 read-back's argument in `run`. `md encode` refusing LATER is
-///   not enough, because `md descriptor` shares compose's parse path and
-///   would render the refused shape into a concrete, fundable descriptor.
-/// - **md-legal but outside Liana's policy model → WARN.** md does not own
-///   Liana's policy model (SPEC §0a); refusing a wallet md's rules admit
-///   would hard-code one coordinator into md's lowering. Keyed on the
-///   composed SHAPE, never the preset name (R1 I-f): a `--path`-built
-///   equivalent must warn too. Evidence for both halves: Liana v15.0
-///   refused `preset-hashlock-gated-tr` and `preset-decaying-multisig-tr`
-///   (md-codec `tests/fixtures/liana/cases.json`, `accepted: false`).
-/// - **SPEC §6 row 3 → WARN**: a real internal key was extracted, so there
+///   re-worded here (one home, two callers).
+/// - **SPEC §6 row 3 -> WARN**: a real internal key was extracted, so there
 ///   is no unspendable key to choose. The codec signals
 ///   (`Composed::unspendable_request_unmet`); this prints.
 ///
-/// The "no unlocked path" half requires NO real internal key (R2 M-i): a
-/// real key path IS an unlocked path, which makes this half exclusive with
-/// the row-3 warning, so the canonical unlocked-primary +
-/// timelocked-recovery shape prints one warning, not two. MEASURED, the
-/// `internal_key_path.is_none()` conjunct is REDUNDANT with the walk:
-/// `policy_shape` already pushes a real internal key as its own unlocked
-/// `Branch` first (`policy_shape.rs`, "the key path as path 0", fix round 1
-/// I-5) -- the plan's premise that `branches` holds tapscript leaves only is
-/// not true of this codec. The conjunct stays as the stated rule, so the
-/// exclusivity does not rest on a walk detail two layers away; deleting it
-/// is semantically inert today, and a mutation test cannot see it. The
-/// hashlock half does not require it (R3 M-5): Liana declines a hashlock
-/// leaf whatever the key path holds.
-fn liana_refuse_or_warn(composed: &md_codec::compose::Composed) -> Result<(), CliError> {
+/// What this function USED to do as well -- warn when a shape sat outside
+/// Liana's policy model (a hashlock; every path timelocked) -- was a second
+/// hand-written Liana classifier beside the fork's, and F-644 measured the
+/// shapes it missed. Plan 1b REPLACED it with the coordinator verdict every
+/// compose now prints (`crate::cmd::verdict`), which reads md-codec's
+/// registry: one rule set, for every shape, with or without the flag.
+fn unspendable_liana_checks(composed: &md_codec::compose::Composed) -> Result<(), CliError> {
     md_codec::validate::validate_unspendable_shape(&composed.descriptor)
         .map_err(CliError::Codec)?;
-    let shape = md_codec::policy_shape::policy_shape(&composed.descriptor);
-    let mut reasons: Vec<&str> = Vec::new();
-    if shape.branches.iter().any(|b| !b.hashlocks.is_empty()) {
-        reasons.push("a path carries a hashlock, which Liana's spending policy has no place for");
-    }
-    if composed.internal_key_path.is_none() && shape.branches.iter().all(|b| !b.locks.is_empty()) {
-        reasons.push(
-            "every path is timelocked, and Liana needs one primary path that spends without a timelock",
-        );
-    }
-    if !reasons.is_empty() {
-        eprintln!(
-            "warning: --unspendable liana: Liana is not expected to import this wallet: {}. \
-             md composes it anyway -- it is a valid md wallet -- but it is not a Liana wallet.",
-            reasons.join("; and ")
-        );
-    }
     if composed.unspendable_request_unmet {
         let path = composed.internal_key_path.map_or(0, |i| i + 1);
         eprintln!(
@@ -675,6 +640,7 @@ pub fn run(
     experimental: bool,
     json: bool,
     unspendable: Option<&str>,
+    md_only: bool,
 ) -> Result<u8, CliError> {
     let wrapper = parse_wrapper(wrapper)?;
     // F-449 stage 2 Task 1b, RULING (R2 NEW-I-1): the refusal is on the FLAG,
@@ -712,7 +678,7 @@ pub fn run(
     let composed =
         compose(&list, unspendable_kind).map_err(|e| CliError::Compose(e.to_string()))?;
     if unspendable_kind == UnspendableKind::Liana {
-        liana_refuse_or_warn(&composed)?;
+        unspendable_liana_checks(&composed)?;
     }
     if !composed.experimental.is_empty() && !experimental {
         let mut msg = String::from("this policy needs --experimental:");
@@ -820,6 +786,28 @@ pub fn run(
         )));
     }
 
+    // Coordinator-compat plan 1b: the verdict, on stderr, for every compose.
+    // compose MINTS a policy, so ruling 2's loud stop belongs here (design
+    // §4): when every coordinator refuses at every verified version, nothing
+    // is emitted unless the operator says `--md-only`. A template has no
+    // keys, so no coordinator can be claimed to import it (design §1 (a2)) --
+    // "none" is therefore only ever a set of REFUSALS, never of silences.
+    let none = crate::cmd::verdict::notice(&composed.descriptor, None, "this template")
+        .is_some_and(|vs| md_codec::coordinator::none_imports(&vs));
+    if none && !md_only {
+        return Err(CliError::Compose(
+            "no wallet coordinator md knows imports this policy: every one refuses it at every \
+             verified version (above). md can still rebuild it from the card -- but md cannot \
+             sign. Pass --md-only to compose it anyway."
+                .into(),
+        ));
+    }
+    if md_only && !none {
+        eprintln!(
+            "note: --md-only has no effect: at least one coordinator is not known to refuse this policy"
+        );
+    }
+
     #[cfg(feature = "json")]
     if json {
         use crate::format::json::SCHEMA;
```

Modify `crates/md-cli/src/cmd/descriptor.rs`:

```diff
diff --git a/crates/md-cli/src/cmd/descriptor.rs b/crates/md-cli/src/cmd/descriptor.rs
index 59ecebb..b4a950b 100644
--- a/crates/md-cli/src/cmd/descriptor.rs
+++ b/crates/md-cli/src/cmd/descriptor.rs
@@ -233,6 +233,22 @@ pub fn run(args: DescriptorArgs<'_>) -> Result<u8, CliError> {
         );
     }
 
+    // Coordinator-compat plan 1b: the verdict for the spelling about to be
+    // printed. `md descriptor` READS an existing card, usually one already
+    // engraved, so it never refuses on a verdict (design §4, r2 C-4): the
+    // notice is stderr only and the exit code is untouched.
+    let form = match args.chain {
+        None => Some(md_codec::coordinator::Form::Multipath),
+        Some(0) => Some(md_codec::coordinator::Form::Chain0),
+        Some(1) => Some(md_codec::coordinator::Form::Chain1),
+        Some(_) => None,
+    };
+    let what = match form {
+        Some(f) => format!("this descriptor, {} form", f.as_str()),
+        None => "this descriptor".to_string(),
+    };
+    let _ = crate::cmd::verdict::notice(&descriptor, form, &what);
+
     #[cfg(feature = "json")]
     if args.json {
         use crate::format::json::SCHEMA;
```

Modify `crates/md-cli/README.md`:

```diff
diff --git a/crates/md-cli/README.md b/crates/md-cli/README.md
index 7539b1b..3174615 100644
--- a/crates/md-cli/README.md
+++ b/crates/md-cli/README.md
@@ -46,7 +46,8 @@ cargo install --path crates/md-cli --no-default-features
 | `md address <STRING>...` (or `--template <T> --key @i=<XPUB>`, or keyless md1 phrases + `--from-mk1`/`--from-mk1-file`/`--seat`) | Derive bitcoin addresses from a wallet-policy-mode descriptor. `--key` also takes the origin-notated `@i=[fp/path]XPUB` form. `--chain N` / `--change`, `--index N`, `--count K`, `--network mainnet\|testnet\|signet\|regtest`, `--json`. |
 | `md descriptor <STRING>...` (same three input modes as `md address`) | Emit the CONCRETE output descriptor with its BIP-380 checksum — the string a coordinator asks for. Multipath `<0;1>` by default; `--chain N` / `--change` collapses it. `--json`. |
 | `md decompose <DESCRIPTOR>` | Read a concrete descriptor back into md's forms. `--emit template\|keys\|fingerprints\|descriptor\|commands\|all` (default `all`), `--in FILE`. |
-| `md compose --wrapper <W> (--path <SPEC>... \| --preset <NAME[,...]>)` | Lower an ordered list of spend paths, or one of six named presets, under `tr`/`wsh`/`sh-wsh`/`sh` into a keyed wallet-policy template with default origins. `--json` for the machine-readable form (with the preset's parameters). `--unspendable liana\|nums` (`tr` only; omitted = `nums`, byte-identical to earlier releases) picks the unspendable internal key when no path supplies a real one: `liana` is Liana's own derived key, which Liana needs to import the wallet. |
+| `md compose --wrapper <W> (--path <SPEC>... \| --preset <NAME[,...]>)` | Lower an ordered list of spend paths, or one of six named presets, under `tr`/`wsh`/`sh-wsh`/`sh` into a keyed wallet-policy template with default origins. `--json` for the machine-readable form (with the preset's parameters). `--unspendable liana\|nums` (`tr` only; omitted = `nums`, byte-identical to earlier releases) picks the unspendable internal key when no path supplies a real one: `liana` is Liana's own derived key, which Liana needs to import the wallet. Prints a coordinator verdict on stderr (Liana, Nunchuk, Bitcoin Core, verified versions only); when every coordinator refuses, compose refuses and names `--md-only`. |
+| `md shape-key (<MD1>... \| --descriptor <MULTIPATH-DESCRIPTOR>)` | Print the policy's shape key, the coordinator-independent summary the verdict table is keyed by. |
 | `md vectors [--out DIR]` | Regenerate the project's deterministic test-vector corpus (maintainer tool). |
 | `md repair <STRING>...` | BCH error-correct one or more chunked-form md1 strings (up to 4 substitution errors per chunk via `BCH(93,80,8)` `t=4` capacity). Atomic per-chunk semantics per plan §1 D28: ANY chunk failing capacity aborts the whole call. Exit 5 (`REPAIR_APPLIED`) on success — and, since md-cli 0.19.0, when a SINGLE corrected card (one string) carries a wire version this build cannot read (stdout = the corrected card, stderr names the version; `mnemonic repair` exits 2 there; several strings at such a version still exit 2) — exit 0 if all inputs already valid, exit 2 on unrepairable input. `--json` emits a `RepairJson` envelope byte-matching `mnemonic repair --json`. **Chunked-form only** at md-cli v0.6.0; non-chunked single-string md1 input is rejected with a wire-format error (tracked at `design/FOLLOWUPS.md` `md-codec-decode-with-correction-supports-non-chunked-md1`). |
 | `md compile <EXPR> --context tap\|segwitv0 [--unspendable-key <KEY>]` | Compile a sub-Miniscript-Policy expression into a BIP 388 template. Requires `cli-compiler` feature. `--unspendable-key` is a tap-context-only fallback hint; defaults to BIP-341 NUMS H-point when omitted. |
```

- [ ] **Step 6:** PASS — including `omitting_the_flag_is_byte_identical_to_the_previous_release`
  (stdout is untouched; the verdict is stderr). The gate. Commit
  `md compose/descriptor: the coordinator verdict, the none case and --md-only (F-644)`.

---

### Task 9: The release, and the PR

md-codec `0.47.0 → 0.48.0` (new public modules; `Skeleton` gains a public
field, a break for anyone constructing it), md-cli `0.19.0 → 0.20.0`, the
exact pin `=0.48.0` in the SAME commit (`crates/md-cli/Cargo.toml` — a split
commit fails to resolve), `Cargo.lock`, two CHANGELOG entries.

- [ ] **Step 1:**

Modify `crates/md-codec/Cargo.toml`:

```diff
diff --git a/crates/md-codec/Cargo.toml b/crates/md-codec/Cargo.toml
index 588dde8..dea9568 100644
--- a/crates/md-codec/Cargo.toml
+++ b/crates/md-codec/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "md-codec"
-version = "0.47.0"
+version = "0.48.0"
 edition.workspace = true
 rust-version.workspace = true
 license.workspace = true
```

Modify `crates/md-cli/Cargo.toml`:

```diff
diff --git a/crates/md-cli/Cargo.toml b/crates/md-cli/Cargo.toml
index 0391653..b6be00f 100644
--- a/crates/md-cli/Cargo.toml
+++ b/crates/md-cli/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "md-cli"
-version = "0.19.0"
+version = "0.20.0"
 edition.workspace = true
 rust-version.workspace = true
 license.workspace = true
@@ -25,7 +25,7 @@ json = ["dep:serde", "dep:serde_json"]
 cli-compiler = ["miniscript/compiler"]
 
 [dependencies]
-md-codec = { path = "../md-codec", version = "=0.47.0" }
+md-codec = { path = "../md-codec", version = "=0.48.0" }
 clap = { version = "4.5", features = ["derive"] }
 clap_mangen = "0.3"
 libc = "0.2"
```

Modify `CHANGELOG.md`:

```diff
diff --git a/CHANGELOG.md b/CHANGELOG.md
index 75b9c7b..189b60b 100644
--- a/CHANGELOG.md
+++ b/CHANGELOG.md
@@ -4,6 +4,65 @@ All notable changes to `md-codec` and `md-cli` are documented in this file. Each
 
 The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows [SemVer](https://semver.org/spec/v2.0.0.html) with the pre-1.0 convention that the second component (`0.X`) is the breaking-change axis.
 
+## md-cli [0.20.0] — UNRELEASED
+
+### Added
+
+- **`md shape-key`** prints a policy's shape key -- the canonical,
+  coordinator-independent summary the coordinator verdict table is keyed by
+  (U+001F-separated: template, partitions, key-path kind) -- from an md1 card
+  or, with `--descriptor`, from a multipath (`<0;1>`) descriptor. A thin CLI
+  over md-codec; a test asserts it agrees with the library on every vendored
+  evidence descriptor.
+- **Every `md compose` and `md descriptor` prints a coordinator verdict on
+  stderr** (Liana, Nunchuk, Bitcoin Core), one line per run of VERIFIED
+  versions, e.g. `Bitcoin Core 26.0-31.1: imports the chain0 form (measured
+  2026-09-23)`. Refusals come from source-derived rules; a positive is printed
+  only where a measurement backs it, and a keyless template never claims one.
+  `md descriptor` names the spelling it printed (multipath, chain0, chain1).
+  stdout and `--json` are unchanged.
+- **`md compose --md-only`.** When every coordinator refuses the policy at
+  every verified version, `md compose` now REFUSES (exit 1, nothing on stdout)
+  and names this flag; with it, compose proceeds. `md descriptor` never
+  refuses on a verdict.
+
+### Changed
+
+- **`--unspendable liana`'s Liana warnings are replaced by the verdict**
+  (F-644's md-cli half). The hand-written "Liana is not expected to import
+  this wallet" warning covered two shapes; the registry names Liana's refusal
+  class for every shape, with or without the flag. SPEC §6's refusal and the
+  "has no effect" warning are md's own rules and are unchanged.
+- **A keyless path composed with `--experimental` now also needs
+  `--md-only`**: no coordinator imports it (Liana, Nunchuk and Core all
+  refuse a spend path without a signature).
+
+## md-codec [0.48.0] — UNRELEASED
+
+### Added
+
+- **`coordinator`**: which wallet coordinators import a policy. Four verdict
+  kinds (`Refuses`, `ImportsAltered`, `Imports`, `Unproven`), a registry of
+  Liana, Nunchuk and Bitcoin Core with their verified versions, source-derived
+  refusal rules, and a GENERATED table of measured cells
+  (`coordinator/table.rs`, from `tests/fixtures/coordinator/evidence.jsonl`
+  by `cargo xtask verdicts`). The build fails on any rule/evidence
+  disagreement (D1-D5). The Bitcoin Core boundary is measured on release
+  binaries: tapscript miniscript from 26.0, the `<0;1>` spelling after 28.4.
+- **`descriptor_route`** (feature `derive`): a public route from multipath
+  descriptor TEXT to a decoded `Descriptor` and its `SkeletonKey`. Promoted
+  from the conformance test's private walker; every reconstruction is
+  self-checked by a byte-identical re-render of both chains.
+- **`Skeleton::leaf_keys_ascending`**: whether a `tr` policy's tap-leaf keys
+  are in strictly ascending order -- the one fact Nunchuk's handling of
+  Liana's unspendable key turns on. Not part of the `SkeletonKey`.
+
+### Fixed
+
+- **`policy_shape`: a duplicate-index `multi` reports no `k`-of-`n`** in its
+  bare spelling as it already did behind a wrapper (`wsh(multi(2,@0,@0,@1))`
+  was 2-of-3 over two keys).
+
 ## md-cli [0.19.0] — 2026-09-23
 
 ### Added
```

  then `cargo update --workspace --offline` (moves exactly the two versions),
  and set the two `UNRELEASED` dates when the release is cut.

- [ ] **Step 2: Freshness against the REAL engrave** (R-2's local half):
  `scripts/vendor-coord-evidence.sh --check /scratch/code/shibboleth/mnemonic-engrave`
  → `fresh`; `cargo xtask verdicts --check` → `fresh`.
- [ ] **Step 3:** the gate. Commit `release: md-codec 0.48.0, md-cli 0.20.0`.
- [ ] **Step 4: The PR.** Push `cc-1b-verdicts` and open the pull request
  against `main` (`gh pr create --repo bg002h/descriptor-mnemonic`), body
  ending with the attribution lines. **Hand the operator the PR number for
  `/code-review ultra <PR#>`** — user-triggered; no agent launches it, and no
  opus whole-diff review is scheduled. Fold its findings on the branch;
  merge only when it closes, via the repo's push ritual; then tag
  `descriptor-mnemonic-md-cli-v0.20.0` as 0.19.0 was.

---

## Hand-off

- **Plan 2** (harnesses, `KNOWN_RELEASES`, renderer gate, evidence snapshot):
  Nunchuk's harness is still only in `.tmp`; Core's `probe.py` is committed;
  the vendoring script's per-source constants are where a harness's
  self-reported version should replace a cited one. F-633 stays there.
- **Plan 3** (fork): port `coordinator` from generated Go, not hand-ported
  rules; the notice gate needs the key-kind exemption and must scan beyond
  `composerCopy*` (design §4, recounted); the Go and Rust `branches` lengths
  differ on every `tr` policy with a real key path.

## Self-review

- Spec coverage: §5 step 1 (verdict kinds, table, xtask, vectors-to-evidence
  gate) — Tasks 4-6; step 2 (`md shape-key`, the verdict, the none case,
  `--md-only`) — Tasks 7-8; §3 mechanisms 1-2 (provenance, closed spans) —
  `describe` and `Span`; 1a M-1 — Task 1; F-644 md-cli — Task 8; F-655 —
  Task 2; the six minors — Task 0; R-1 — Tasks 0, 3, 4, 6; R-2 — Task 5.
- Not in scope, by the design: mechanisms 3-6, the fork, PR-1746's encoding.
