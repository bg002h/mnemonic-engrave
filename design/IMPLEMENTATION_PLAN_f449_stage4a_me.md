# F-449 stage 4a: `me` reads version-8 plates and stops miscounting plates it cannot read

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `me` (crate `crates/me-cli`) decodes version-8 md1 plates, and it never states a plate count, or a card confirmation, that it did not compute. Where it cannot read a plate, it names the wire version.

**Architecture:** there are four pieces of work (SPEC §9a). (1) Move md-codec off crates.io `0.42` onto the descriptor-mnemonic git rev `cf35d61a` (md-codec 0.47.0), with the rust-miniscript `[patch.crates-io]` the new md-codec requires and a `cargo update -p miniscript`, without which the patch is silently unused. (2) Repair the one `Body::Tr` literal that §3f's type change breaks, plus one test that the patch turns from an `Err` into a panic. (3) F-635: `bundle.rs`'s unchunked path refuses an undecodable plate instead of skipping it. (4) `sysw/record.rs`'s confirmation walk keeps the decoder's reason, so `pack`, `show` and `--expect` name an unsupported wire version. A release (`me 0.11.0`) follows.

**Tech stack:** Rust 1.85.0 (CI's pin), md-codec 0.47.0 by git rev, rust-miniscript `ff4732e5` by `[patch.crates-io]`, `cargo nextest` for the suite.

**Spec:** `design/SPEC_liana_unspendable_internal_key.md` — §9's stage-4a row, §9a (all four items), §8b, §6a, and §8.9's two `me` rows. Continuity: `design/CONTINUITY_f449_stage2.md`.

**Baseline revisions** (for `scripts/plan-staleness-check.sh`; every `file:line` below was measured at these):
- `mnemonic-engrave` master **`8aea0d36`** (`me` 0.10.0). Suite: `cargo nextest run --locked -p mnemonic-engrave` gives **664 run, 664 passed, 2 skipped**.
- `descriptor-mnemonic` main **`cf35d61a`** = tag `descriptor-mnemonic-md-cli-v0.19.0` (md-codec 0.47.0, md-cli 0.19.0), on `origin/main`.
- `seedhammer` fork main **`7b6f2fb`**. Stage 3 has **not** landed: the fork's `md/` reads no wire version 8. Task 5 depends on this.

**Execution gate, run by the plan author and reproducible.** Every code block in this plan was applied to a scratch copy of `8aea0d36` (`/scratch/code/shibboleth/.tmp/f449-4a-probe`; the full diff is saved as `/scratch/code/shibboleth/.tmp/f449-4a-probe.patch`, 752 lines) and then built and run on toolchain 1.85.0:
- `cargo clippy --all-targets --locked -- -D warnings`: clean.
- `cargo fmt --check`: clean. The blocks below are already in rustfmt's output form.
- `cargo nextest run --locked -p mnemonic-engrave`: **675 run, 675 passed, 2 skipped**, which is the 664 at baseline plus the 11 new tests.
- `cargo test --locked -p mnemonic-engrave` (what CI's `test (rust + go)` job runs): 675 passed, 0 failed, 2 ignored.
- `cargo metadata --locked`: resolves.
- Mutation pass: **9 of 9 mutations KILLED** (the M-rows in each task). A tenth, M1 (revert the unpin), was measured on the pristine baseline and both Task-1 tests fail there.

**`./scripts/plan-build-gate.sh` result on this plan: exit 3, `EXTRACTED NOTHING`, by design.** That gate assembles only anchors naming `src/seal/*.rs` or `tests/seal_cli.rs`. `./scripts/plan-build-gate-me.sh` likewise recognises only `src/sysw/composer_*.rs` and `tests/sysw_composer*.rs`, and also exits 3. This plan creates none of those files. Every block here either modifies an existing file or goes into the new `tests/f449_stage4a.rs`, and neither gate assembles such blocks. **Neither gate is a close condition for this plan.** The scratch-copy run above is the executed gate. Its coverage line: *it proves every block compiles, passes clippy and fmt, and that each named test FAILS under its named mutation. It does not prove that the prose around a block matches the block, that the CHANGELOG text is accurate, or anything about the release workflow, the device or the fork.*

## Global Constraints

- **Toolchain:** `export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH` before every cargo command. CI pins `RUST_TOOLCHAIN: '1.85.0'` (`.github/workflows/release.yml:48`).
- **Own target dir per worktree:** `export CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/f449-4a-target`. A shared target bakes paths across worktrees, and `/tmp` is a 32 GB tmpfs.
- **Suite:** `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast`. Capture it to a file once and grep that file; never run it twice to get counts.
- **CI's other gates**, run before every commit: `cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`.
- **The pin is a git REV, never a path and never a tag.** A path dependency does not resolve in CI (the same reason `mt-codec` is a rev, `crates/me-cli/Cargo.toml:66-81`), and a tag can be moved. `cf35d61af0f058029df841f4882945e53f86cd4b`.
- **The miniscript rev MUST equal descriptor-mnemonic's.** `descriptor-mnemonic/Cargo.toml:43-44` at `cf35d61a` is `miniscript = { git = "https://github.com/rust-bitcoin/rust-miniscript", rev = "ff4732e5f75aa555682343cb180fa72ee3e8e9d5" }`. mnemonic-toolkit uses the same rev (`mnemonic-toolkit/Cargo.toml:35`).
- **Refuse rather than miscount (SPEC §8b ruling).** Of the two remedies the spec allows, this plan takes "`me bundle` refuses the payload naming the version". The chunked path already refuses (`bundle.rs:397-398`, `?`), and the unchunked path must match it.
- **The accepted-version set has ONE spelling: md-codec's.** Every message renders `md_codec::Error::WireVersionMismatch { got }` through its `Display` (`wire-format version mismatch: got 12; accepted versions: 4, 8`, md-codec `error.rs:34`). A second hand-written "4, 8" would go wrong the day the codec accepts 12.
- **`me sysw pack` still WARNS and proceeds** on an unconfirmed card (§12.6 / D6, `tests/sysw_cli.rs:670-675`). Task 3 changes the words, not the exit code.
- **Stage paths explicitly.** Never `git add -A`. Commit messages go through `git commit -F <file>`, because fish eats backticks. Check each message with `git log -1 --format=%B`.
- **Two repos.** Everything here is **mnemonic-engrave**, including `design/FOLLOWUPS.md`. F-651 is a *descriptor-mnemonic* defect, but it is filed in THIS repo's FOLLOWUPS, as F-642..F-646 were.
- **Pushes and the release need no permission prompt** (standing operator ruling). The release still has a hard precondition: see Task 5 Step 0.

## Measured facts — do NOT re-derive

| # | fact | how measured |
| --- | --- | --- |
| F1 | `crates/me-cli/Cargo.toml:26` is `md-codec = "0.42"`; `Cargo.lock:547-550` is `md-codec 0.42.0`, `registry+…crates.io-index`. | read |
| F2 | Pointing the pin at the git rev with **no** patch fails to compile md-codec: 3 × E0599 (`derive_at_index`, `into_definite` at `derive.rs:148/150`; `Terminal::SortedMultiA` at `to_miniscript.rs:670`). | RUN |
| F3 | **Adding the `[patch.crates-io]` alone changes nothing.** Cargo prints `warning: Patch miniscript v13.0.0 (…ff4732e5…) was not used in the crate graph` and the same 3 errors, because `Cargo.lock` holds `miniscript 13.1.0` (> the rev's `13.0.0`). `cargo update -p miniscript` then moves the lock onto the rev, and the lock diff is exactly the two stanzas shown in Task 1 Step 4. | RUN |
| F4 | §3f's type change breaks **exactly one site** in `me`: the `Body::Tr { is_nums: false, key_index: 0, tree: None }` literal at `src/descriptor/md1.rs:351-357` (2 × E0559, lines 354 and 355). `bundle.rs:250` matches `Body::Tr { tree, .. }`, and the `..` absorbs the change, as with md-cli's `seat/compose.rs:148`. | RUN (`cargo check --all-targets`) |
| F5 | After F4's repair, **one test goes red**: `descriptor_seam::the_two_derivations_agree_wherever_both_can_derive` **panics** inside rust-miniscript (`src/lib.rs:355`, "Script cannot be larger than 520 bytes, but got 547 bytes"). The panic is on row `narrowed/sh-sortedmulti-16-keys`. On 0.42 + miniscript 13.1.0 the same call returned `Err(AddressDerivationFailed{…})`. **Not reachable in production:** `md1::address`/`derivation_twin` have no non-test caller (grep over `src/`), and the CLI refuses that row at admission first (`descriptor_refusals::row_key_count_exceeded`, green). But `md_codec::split` ADMITS the descriptor (29 chunks) and `Descriptor::derive_address` then panics, which makes it a **md-codec defect**. Under the Rust-primary rule it is fixed in descriptor-mnemonic first → F-651. | RUN (probe tests in both trees) |
| F6 | `me convert` of a v8 string passes on 0.42 unchanged (SPEC §9, `lib.rs:75-83` → `validate.rs:95-100`, BCH only). Not a gate. | spec, re-read |
| F7 | **F-635 is live at wire version 4, not only latent.** `md encode "tr(50929b74…3ac0,{pk(@0/<0;1>/*),pk(@1/<0;1>/*)})"` (md-cli 0.19.0) emits `md1yppqqxqu22z54hcefkda7r46w`, an origin-less 2-key template. Strict `decode_md1_string` refuses it (`MissingExplicitOrigin`). `me` 0.10.0's `bundle` prints `backup needs 1 public plate` with **no TEMPLATE note**, which is F-602's defect coming back through F-635. The *chunked* form of the same class is already refused (`SetIncompleteMd`, RUN). | RUN |
| F8 | On 0.10.0 the v8 kind-1 template `md1cpfdsssj6tvyywtsqrq0zjs4n7gdve74ar402` bundles as `backup needs 1 public plate`, exit 0, manifest `key_slots: 0`, `keyless_template: false`, with no notes at all. After the unpin: `key_slots: 2`, `keyless_template: true`. | RUN, both trees |
| F9 | md-codec 0.47 on the fixtures: `V12_SINGLE` → `WireVersionMismatch{got:12}`; `v12_chunk()` → `parse_line` `Md1WireVersion`, and `decode_md1_string` → `WireVersionMismatch{got:12}`; `V4_UNDECODABLE` → `BitStreamTruncated{requested:14, available:1}`; the origin-less v8 template `md1gppqqxq799p20d5hxuzu2c9la` → `MissingExplicitOrigin{idx:0}`. That last one is why the v8 fixture carries origins. | RUN |
| F10 | `sysw/record.rs:251-252` reduces `reassemble`/`decode_md1_string` to `.is_ok()`. `me sysw pack` then says `an md1/mk1 this tool could not decode` (`main.rs:2110-2116`), `show` says `unconfirmed — engraveable, but the device REPLACES the legend` (`main.rs:2306-2307`, `:2323-2329`), and **`--expect descriptor` says the card "does not reassemble"** (`sysw/expect.rs:202`, `:263-274`), which is false for a whole single card. None of the three names a version. | read + RUN |
| F11 | `me` has **no vendoring check** (there is no `ci/` directory, and `release.yml` has no vendor step). `ms`'s `ci/repro/vendor-freshness.sh` has no counterpart here. | `ls`, grep |
| F12 | The fuzz crate (`crates/me-cli/fuzz/`, its own workspace, not in CI) is already stale: its lock says `mnemonic-engrave 0.3.0` and `md-codec 0.40.0`, and `cargo check --locked` there fails with "lock file needs to be updated". After the unpin it would also need the `[patch.crates-io]`. It is out of scope → F-653. | RUN |
| F13 | **`crates/me-cli/CHANGELOG.md` has no `[0.10.0]` section**, although `v0.10.0` is tagged (2026-09-16, object `1fa8dd99`). Its `[Unreleased]` holds F-493/F-504, both of which are ancestors of `v0.10.0`. Nothing records `1cbecbfd` (the tpub refusal, post-0.10.0) either. | `git merge-base --is-ancestor`, read |
| F14 | Releases: the tag is `v<ver>` (annotated, message from a file), and `release.yml` builds and signs on `refs/tags/v*`. The release commit touches exactly `crates/me-cli/Cargo.toml`, `Cargo.lock` and `crates/me-cli/CHANGELOG.md` (`9e4ccad2`). `me-preview`'s version must equal `me`'s (`tests/cli.rs:22-23`; the release sets it by `-ldflags -X main.version`). | `git show --stat`, read |
| F15 | `demo/sh2` (stage 5) embeds no `me` download. `demo/sh2/build-payload.sh:117,122` runs `me sysw pack` / `me sysw show` **from PATH**, so stage 5 depends on the local `me` being ≥ 0.11.0 (Task 5 Step 8). | grep |
| F16 | The only FOLLOWUPS entry owned by stage 4a is **F-635** (`design/FOLLOWUPS.md:19210`). The highest F-number at `8aea0d36` is F-646. | grep `stage 4a`, sort |

## Review Focus

These are the five failure modes the spec implies that are most likely to reach a person, with the test that pins each one:

1. **An origin-less template plate at v4.** A real `md encode` output that `md decode` reads (with VERIFY-ME), which `me bundle` must refuse rather than count as one plate. Pinned by `a_version_4_origin_less_template_is_refused_not_miscounted` (Task 2). This refusal is **new behaviour for a plate 0.10.0 accepted**. The CHANGELOG says so, and F-652 carries the better remedy (a partial decode that counts correctly).
2. **One good plate and one bad plate in the same bundle.** Must refuse, not average. `a_good_plate_does_not_carry_a_bad_one` (Task 2).
3. **A chunked plate at an unsupported version.** It was always refused, but its message named no version. `bundle_names_the_version_of_a_chunk_it_cannot_read` (Task 2), `the_walk_keeps_the_version_for_both_shapes` (Task 3).
4. **`--expect descriptor` on a card at an unsupported version.** It must not say "does not reassemble". `expect_descriptor_names_the_version_instead_of_calling_it_incomplete` (Task 3).
5. **A v8 card `me` now calls "confirmed" on a device that cannot read it.** No code gate is possible here, because `me` cannot know the flashed firmware. The guard is Task 5 Step 0's precondition: no release until stage 3 is in fork main. **This is what no test covers.**

---

## File Structure

- **Modify** `Cargo.toml` (root). Append the `[patch.crates-io]` miniscript override (Task 1).
- **Modify** `crates/me-cli/Cargo.toml:26`. The md-codec pin becomes a git rev (Task 1). Line `:3`, the version, changes in Task 5.
- **Modify** `Cargo.lock`. Two stanzas in Task 1 and one in Task 5.
- **Modify** `crates/me-cli/src/descriptor/md1.rs:45`, `:351-357`. The `InternalKey::Slot(0)` repair (Task 1).
- **Modify** `crates/me-cli/tests/descriptor_seam.rs:1127-1164`. Skip the twin on the named over-limit rows (Task 1).
- **Modify** `crates/me-cli/src/bundle.rs:34-35`, `:96`, `:210-212`, `:368-376`, and tests `:558`, `:1071`. F-635 (Task 2).
- **Modify** `crates/me-cli/src/sysw/record.rs:210-275`. `Unconfirmed`, `mdmk_unconfirmed_why` and `md_verdict` (Task 3).
- **Modify** `crates/me-cli/src/main.rs:2110-2116` (`report_unconfirmed`) and `:2296-2307` (`print_records`) (Task 3).
- **Modify** `crates/me-cli/src/sysw/expect.rs:68`, `:139-145`, `:202`, `:224-230`, `:263-275`, and test `:312`. `Unmet::UnreadableVersion` (Task 3).
- **Create** `crates/me-cli/tests/f449_stage4a.rs`. This task's tests, which grow task by task.
- **Modify** `crates/me-cli/CHANGELOG.md` (Task 5), `design/FOLLOWUPS.md` (Task 4), `design/SPEC_liana_unspendable_internal_key.md` (Task 4).
- **Not touched:** `validate.rs`, `convert`, `seal/record.rs` (its `decode_public_set` already refuses, with md-codec's message), `crates/mnemonic-io-lib`, the fuzz crate (F-653), and the fork.

---

## Task 0: worktree, and re-validate this plan against the tree

This plan's GREEN expires. Standing directive 2026-08-27: re-validate immediately before dispatch.

- [ ] **Step 1: Worktree.**

```bash
cd /scratch/code/shibboleth/mnemonic-engrave
git worktree add -b f449-stage4a /scratch/code/shibboleth/me-worktrees/f449-stage4a master
cd /scratch/code/shibboleth/me-worktrees/f449-stage4a
export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH
export CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/f449-4a-target
```

- [ ] **Step 2: Staleness.** `scripts/plan-staleness-check.sh design/IMPLEMENTATION_PLAN_f449_stage4a_me.md . 8aea0d36`. Any drifted citation in `crates/` gets re-resolved before its task runs. Also confirm that `git -C /scratch/code/shibboleth/descriptor-mnemonic rev-parse descriptor-mnemonic-md-cli-v0.19.0^{commit}` is still `cf35d61af0f058029df841f4882945e53f86cd4b`.

- [ ] **Step 3: Baseline.** `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast > /scratch/code/shibboleth/.tmp/f449-4a-base.log 2>&1`. Expect `664 tests run: 664 passed, 2 skipped`. If master moved, record the new count. Every "+N" below is relative to it.

---

## Task 1: the unpin and its patch, plus the two repairs it forces (SPEC §9a items 1 and 2)

**Files:**
- Modify: `Cargo.toml`, `crates/me-cli/Cargo.toml:26`, `Cargo.lock`
- Modify: `crates/me-cli/src/descriptor/md1.rs:45`, `:351-357`
- Modify: `crates/me-cli/tests/descriptor_seam.rs:1127-1164`
- Create: `crates/me-cli/tests/f449_stage4a.rs`

**Interfaces:**
- Consumes: md-codec 0.47.0's `md_codec::tree::InternalKey::{Slot(u8), NumsPoint, LianaUnspendable}` and `Body::Tr { internal_key: InternalKey, tree: Option<Box<Node>> }`.
- Produces: nothing new in the API. `me` now links md-codec 0.47.0.

- [ ] **Step 1: Write the failing tests.** Create `crates/me-cli/tests/f449_stage4a.rs`:

```rust
//! F-449 stage 4a (`design/SPEC_liana_unspendable_internal_key.md` §8.9's two
//! `me` rows, §8b, §6a, §9a): what `me` does with an md1 at wire version 8,
//! and with one at a version it does not read.
//!
//! Every fixture is MEASURED, never hand-built from a reading of the spec:
//! - `V8_TEMPLATE`: md-cli 0.19.0 (descriptor-mnemonic `cf35d61a`),
//!   `md encode "tr(UNSPENDABLE(liana),{pk(@0/48'/0'/0'/3'/<0;1>/*),pk(@1/48'/0'/1'/3'/<0;1>/*)})"`.
//!   A 2-key TEMPLATE at wire version 8. me 0.10.0 (md-codec 0.42) bundled it
//!   as "backup needs 1 public plate" with no template note (key_slots 0).
//! - `V12_SINGLE`: descriptor-mnemonic `crates/md-cli/tests/cli_repair_unsupported_version.rs`
//!   `V12_CLEAN` -- BCH-clean, single-string, header version 12.
//! - `V4_UNDECODABLE`: `wrap_payload([0x20, 0xff × 12], 100)` -- a v4 single
//!   string that passes its checksum and fails decode (`BitStreamTruncated`).
//! - `v12_chunk()`: `wrap_payload([0xC8, 0 × 12], 100)` -- first symbol
//!   0b11001, chunked flag set, chunk-header version 12.
#![cfg(unix)]

use assert_cmd::Command;

const V8_TEMPLATE: &str = "md1cpfdsssj6tvyywtsqrq0zjs4n7gdve74ar402";

struct Out {
    code: i32,
    out: String,
    err: String,
}

fn me(args: &[&str], stdin: &str) -> Out {
    let o = Command::cargo_bin("me")
        .unwrap()
        .args(args)
        .write_stdin(stdin.to_string())
        .output()
        .unwrap();
    Out {
        code: o.status.code().unwrap(),
        out: String::from_utf8_lossy(&o.stdout).into_owned(),
        err: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

// ---- Task 1: the unpin. Both FAIL on me 0.10.0 (md-codec 0.42). ----------

/// The version-8 template is counted from what it ENCODES. On md-codec 0.42
/// the decode failed, was skipped, and the checklist said "1 public plate"
/// with no TEMPLATE note -- cut that one plate and the wallet is gone.
#[test]
fn a_version_8_template_plate_is_counted_from_what_it_encodes() {
    let r = me(&["bundle"], &format!("{V8_TEMPLATE}\n"));
    assert_eq!(r.code, 0, "{}", r.err);
    let m: serde_json::Value = serde_json::from_str(&r.out).expect("manifest JSON");
    assert_eq!(m["key_slots"], 2, "{}", r.out);
    assert_eq!(m["keyless_template"], true, "{}", r.out);
    assert!(r.err.contains("this md1 is a TEMPLATE"), "{}", r.err);
    assert!(r.err.contains("declares 2 key slots"), "{}", r.err);
}

/// `me sysw` confirms a version-8 card: the §12.6 walk decodes it.
#[test]
fn sysw_confirms_a_version_8_card() {
    assert_eq!(
        mnemonic_engrave::sysw::record::mdmk_unconfirmed(&[V8_TEMPLATE.to_string()]),
        Vec::<usize>::new()
    );
}
```

The file's module doc already describes Task 2's fixtures. Only `V8_TEMPLATE` is declared here, because an unused `const` in an integration test is `dead_code` and fails clippy's `-D warnings`. Task 2 adds the rest.

- [ ] **Step 2: Run, and see both FAIL.** `cargo nextest run --locked -p mnemonic-engrave --test f449_stage4a`. Measured on `8aea0d36`: `a_version_8_…` fails with `left: Number(0)`, `right: 2`, and `sysw_confirms_…` fails with `left: [0]`, `right: []`. **This is mutation M1** (revert the unpin), and it is killed.

- [ ] **Step 3: The pin and the patch.** In `crates/me-cli/Cargo.toml`, replace line 26 (`md-codec = "0.42"`) with:

```toml
# A GIT dependency pinned to a rev, not a crates.io version (F-449 stage 4a,
# SPEC_liana_unspendable_internal_key §9a). md-codec is unpublishable while it
# depends on the rust-miniscript git patch below, so crates.io stopped at 0.42
# -- which reads no wire version but 4, and fails every version-8 plate.
# cf35d61a is `descriptor-mnemonic-md-cli-v0.19.0` (md-codec 0.47.0). A rev,
# not the tag: a tag can be moved, a rev cannot. Same shape as mnemonic-toolkit.
#
# NEEDS the root `[patch.crates-io]` miniscript override: md-codec 0.47 calls
# APIs that exist only at that rev (`derive_at_index`, `Terminal::SortedMultiA`).
md-codec = { git = "https://github.com/bg002h/descriptor-mnemonic", rev = "cf35d61af0f058029df841f4882945e53f86cd4b" }
```

Append to the root `Cargo.toml`, after `[profile.dev]`:

```toml

# md-codec (crates/me-cli/Cargo.toml) is built against rust-miniscript at this
# rev, not a crates.io release: it carries PR #953 (taptree Display) and #915
# (`sortedmulti` as a Terminal), in no release through 13.1.0. The rev MUST
# equal descriptor-mnemonic's own `[patch.crates-io]` at the md-codec rev pinned
# there, or md-codec does not compile. Adding this line is not enough on its
# own: Cargo.lock held miniscript 13.1.0 (> this rev's 13.0.0), so the patch is
# ignored ("Patch ... was not used in the crate graph") until
# `cargo update -p miniscript` moves the lock onto it.
[patch.crates-io]
miniscript = { git = "https://github.com/rust-bitcoin/rust-miniscript", rev = "ff4732e5f75aa555682343cb180fa72ee3e8e9d5" }
```

- [ ] **Step 4: Move the lock (F3). This needs network.** Run `cargo update -p miniscript`, then `git diff Cargo.lock`. The diff must be **exactly** these two stanzas, and anything else is a stop-and-report:

```text
 name = "md-codec"
-version = "0.42.0"
-source = "registry+https://github.com/rust-lang/crates.io-index"
-checksum = "336f2c0c7f8d394136992f0b9f3d527f94a2023b55772dbed07ac09e9abcfd8f"
+version = "0.47.0"
+source = "git+https://github.com/bg002h/descriptor-mnemonic?rev=cf35d61af0f058029df841f4882945e53f86cd4b#cf35d61af0f058029df841f4882945e53f86cd4b"
…
 name = "miniscript"
-version = "13.1.0"
-source = "registry+https://github.com/rust-lang/crates.io-index"
-checksum = "cd35e2c377504e50159561884b03610711db6c2fec6c89f2b98d94016684726b"
+version = "13.0.0"
+source = "git+https://github.com/rust-bitcoin/rust-miniscript?rev=ff4732e5f75aa555682343cb180fa72ee3e8e9d5#ff4732e5f75aa555682343cb180fa72ee3e8e9d5"
```

`cargo check -p mnemonic-engrave --all-targets` must print **no** `Patch … was not used` warning. The only errors it may print are F4's two E0559s.

- [ ] **Step 5: The §3f repair (F4).** In `crates/me-cli/src/descriptor/md1.rs`, line 45:

```rust
use md_codec::tree::{Body, InternalKey, Node};
```

and at `:351-357`:

```rust
            Script::P2TR => Node {
                tag: Tag::Tr,
                body: Body::Tr {
                    internal_key: InternalKey::Slot(0),
                    tree: None,
                },
            },
```

`Slot(0)` is exactly what `is_nums: false, key_index: 0` meant: a real key at slot 0.

- [ ] **Step 6: The seam test repair (F5).** In `crates/me-cli/tests/descriptor_seam.rs`, `the_two_derivations_agree_wherever_both_can_derive`: replace `let (mut agreed, mut against_file) = (0usize, 0usize);` and the `let twin = …` binding, and add the pin before the existing `agreed >= 25` assertion. The whole function after the edit is:

```rust
#[test]
fn the_two_derivations_agree_wherever_both_can_derive() {
    let d = doc();
    let (mut agreed, mut against_file) = (0usize, 0usize);
    let mut oversize: Vec<&str> = Vec::new();
    for r in rows(&d) {
        let input = r["input"].as_str().unwrap();
        let Ok(parsed) = mnemonic_engrave::descriptor::cascade::cascade(
            &mnemonic_engrave::descriptor::cascade::normalise(input),
        ) else {
            continue;
        };
        let per_key = mnemonic_engrave::descriptor::derive::address_0(&parsed);
        // F-449 stage 4a: md-codec 0.47 under rust-miniscript ff4732e PANICS
        // deriving a script over the 520-byte P2SH limit (0.42 under 13.1.0
        // returned Err). Production never calls the twin on such a row --
        // admission refuses it first (`row_key_count_exceeded`) -- so the
        // differential skips exactly the rows admission refuses for that
        // reason, and names them below. Tracked upstream as F-651.
        use mnemonic_engrave::descriptor::{admit, refusal::Row};
        let over_limit = matches!(
            admit::admit(&parsed, admit::Path::Md1),
            Err(ref e) if e.row == Row::KeyCountExceeded
        );
        if over_limit {
            oversize.push(name(r));
        }
        let twin = (!over_limit)
            .then(|| mnemonic_engrave::descriptor::md1::derivation_twin(&parsed).ok())
            .flatten()
            .and_then(|(b, i)| {
                let net = mnemonic_engrave::descriptor::md1::network(&parsed);
                mnemonic_engrave::descriptor::md1::address(&b, 0, i?, net).ok()
            });
        if let (Some(a), Some(b)) = (&per_key, &twin) {
            assert_eq!(a, b, "{}: the two derivations disagree", name(r));
            agreed += 1;
        }
        // And where the FILE carries a device-measured address, the per-key
        // walk must land on it — the assertion that makes agreement mean
        // "agrees with the device" rather than "agrees with itself".
        if let (Some(a), Some(want)) = (&per_key, r.get("address_0").and_then(|v| v.as_str())) {
            assert_eq!(a, want, "{}: per-key address_0 vs the device", name(r));
            against_file += 1;
        }
    }
    // Pinned by NAME, so the skip cannot quietly widen to a row that derives.
    assert_eq!(
        oversize,
        [
            "narrowed/sh-sortedmulti-16-keys",
            "narrowed/wsh-sortedmulti-21-keys"
        ],
        "the twin is skipped on exactly the key-count-exceeded rows"
    );
    assert!(
        agreed >= 25,
        "only {agreed} rows exercised the differential — the loop has gone vacuous"
    );
    assert_eq!(
        against_file, POP.address_0,
        "every device-measured address_0 is reached by the per-key walk"
    );
}
```

Why this shape and not the obvious alternatives. Each alternative was measured:
- **Gating the twin on `host_admits(input)`:** drops `agreed` from **28 to 19** and fails the `>= 25` floor, because nine non-admitted rows derive fine and agree.
- **`continue` before `per_key`:** would also drop the row from `against_file`, whose count is exact (`POP.address_0`).
- **`catch_unwind`:** would hide F-651's panic rather than name it.

The skip changes no count. `agreed` is **28** both at baseline and after, and `against_file` is **20** both before and after, so neither skipped row ever contributed. The name list was measured as two rows, not one; my first guess named only the 16-key row.

- [ ] **Step 7: Run the whole suite.** `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast > /scratch/code/shibboleth/.tmp/f449-4a-t1.log 2>&1`. Expect **666 run, 666 passed, 2 skipped** (664 + 2). Then run `cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`.

**Mutations that must be KILLED (measured):**
- **M1**, revert Steps 3-4: kills both Task-1 tests (Step 2).
- **M9**, `let over_limit = false && matches!(…)`: the test panics in rust-miniscript.
- **M10**, widen to `admit::admit(&parsed, admit::Path::Md1).is_err()`: the name-pin assertion fails.

- [ ] **Step 8: Commit.**

```bash
git add Cargo.toml Cargo.lock crates/me-cli/Cargo.toml crates/me-cli/src/descriptor/md1.rs \
        crates/me-cli/tests/descriptor_seam.rs crates/me-cli/tests/f449_stage4a.rs
git commit -F /scratch/code/shibboleth/.tmp/f449-4a-t1.msg
```

Subject: `me: md-codec 0.47.0 by git rev + the miniscript patch (F-449 stage 4a, §9a items 1-2)`. The body names F3 (the patch alone is unused) and F5 (Err → panic, F-651), and pastes the Step 7 counts.

---

## Task 2: F-635, the unchunked path refuses what it cannot count (SPEC §8b, §9a item 3)

**Files:**
- Modify: `crates/me-cli/src/bundle.rs` (`BundleError`, its `Display`, `parse_line`, `run_bundle`'s unchunked loop, two unit tests)
- Modify: `crates/me-cli/tests/f449_stage4a.rs` (add)

**Interfaces:**
- Consumes: `md_codec::decode::decode_md1_string(&str) -> Result<Descriptor, md_codec::Error>`, and `md_codec::Error::WireVersionMismatch { got: u8 }`.
- Produces: `BundleError::Md1WireVersion(String, u8)`, which was `(String)` and is a **public break**, plus the new `BundleError::Md1Undecodable(String, md_codec::Error)`. Both map to exit 4 through `exit_code()`'s `_` arm. `run_bundle` returns `Err` for any unchunked md1 that does not decode.

- [ ] **Step 1: Write the failing tests.** Add the Task-2 fixtures to `crates/me-cli/tests/f449_stage4a.rs`, directly under `const V8_TEMPLATE`:

```rust
const V12_SINGLE: &str = "md1uzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg";
const V4_UNDECODABLE: &str = "md1yrlllllllllllllllllltrn9jd5mjtn77";
const V4_ORIGINLESS_TEMPLATE: &str = "md1yppqqxqu22z54hcefkda7r46w";
/// md-codec's own rendering of the refusal. Asserted as a substring so the
/// accepted set is read from the codec, never restated here.
const V12_NAMED: &str = "wire-format version mismatch: got 12; accepted versions: 4, 8";

fn v12_chunk() -> String {
    let mut payload = vec![0u8; 13];
    payload[0] = 0xC8;
    md_codec::codex32::wrap_payload(&payload, 100).unwrap()
}
```

Then append the Task-2 tests to the end of the file:

```rust
// ---- Task 2: F-635, the fail-open at bundle.rs's unchunked path. ----------

/// §8.9 row "`me` fail-open": a bundle never states a plate count it did
/// not compute.
#[test]
fn bundle_refuses_an_unchunked_plate_at_an_unsupported_version() {
    let r = me(&["bundle"], &format!("{V12_SINGLE}\n"));
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(r.out.is_empty(), "no manifest: {}", r.out);
    assert!(!r.err.contains("backup needs"), "no count: {}", r.err);
    assert!(r.err.contains("unsupported md1 wire version"), "{}", r.err);
    assert!(r.err.contains(V12_NAMED), "the version is NAMED: {}", r.err);
    assert!(!r.err.contains("does not decode"), "{}", r.err);
}

#[test]
fn bundle_refuses_an_unchunked_plate_that_does_not_decode() {
    let r = me(&["bundle"], &format!("{V4_UNDECODABLE}\n"));
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(r.out.is_empty(), "no manifest: {}", r.out);
    assert!(!r.err.contains("backup needs"), "no count: {}", r.err);
    assert!(r.err.contains("md1 plate does not decode"), "{}", r.err);
}

/// F-635 is not only a version-8 problem: this is a REAL encoder output at
/// wire version 4 -- md-cli 0.19.0, `md encode
/// "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{pk(@0/<0;1>/*),pk(@1/<0;1>/*)})"`
/// -- an origin-less 2-key TEMPLATE that strict decode refuses
/// (`MissingExplicitOrigin`). me 0.10.0 bundled it as "backup needs 1 public
/// plate" with no template note; the chunked shape of the same class was
/// already refused (`SetIncompleteMd`).
#[test]
fn a_version_4_origin_less_template_is_refused_not_miscounted() {
    let r = me(&["bundle"], &format!("{V4_ORIGINLESS_TEMPLATE}\n"));
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(r.out.is_empty(), "{}", r.out);
    assert!(!r.err.contains("backup needs"), "{}", r.err);
    assert!(r.err.contains("requires explicit origin"), "{}", r.err);
}

/// A decodable plate beside the bad one does not rescue the bundle: the
/// refusal is per plate, not "at least one plate decoded".
#[test]
fn a_good_plate_does_not_carry_a_bad_one() {
    let r = me(&["bundle"], &format!("{V8_TEMPLATE}\n{V12_SINGLE}\n"));
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(r.out.is_empty(), "{}", r.out);
}

/// The chunked shape already refused; it now NAMES the version too.
#[test]
fn bundle_names_the_version_of_a_chunk_it_cannot_read() {
    let r = me(&["bundle"], &format!("{}\n", v12_chunk()));
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(r.err.contains(V12_NAMED), "{}", r.err);
}
```

- [ ] **Step 2: Run, and see them FAIL.** Before Step 3, the three unchunked tests exit 0 with `backup needs 1 public plate` (F7, F8), `a_good_plate_…` exits 0, and `bundle_names_…` exits 4 but prints only `unsupported md1 wire version`, with no version.

- [ ] **Step 3: Implement.** In `crates/me-cli/src/bundle.rs`, replace the `Md1WireVersion` variant (`:34-35`):

```rust
    /// An md1 string has a wire version this build does not read. The `u8` is
    /// the version the header carries, so the refusal can NAME it (SPEC
    /// §8b/§6a: an older `me` meeting a newer plate must say which).
    Md1WireVersion(String, u8),
    /// An unchunked md1 plate passed BCH validation but did not decode. The
    /// plate count is a completeness claim, and it cannot be computed from a
    /// plate that does not decode (F-635), so the bundle is refused.
    Md1Undecodable(String, md_codec::Error),
```

Replace its `Display` arm (`:96`):

```rust
            // The accepted set is md-codec's, rendered through its own
            // Display -- a second spelling of "4, 8" here would drift the day
            // the codec accepts 12.
            BundleError::Md1WireVersion(_, got) => write!(
                f,
                "unsupported md1 wire version: {} -- this build of me cannot read \
                 this plate, so it states no plate count for it",
                md_codec::Error::WireVersionMismatch { got: *got }
            ),
            BundleError::Md1Undecodable(_, e) => write!(
                f,
                "md1 plate does not decode ({e}) -- it passes its checksum, but the \
                 plate count is computed from what the plate encodes, so no count \
                 can be stated for it"
            ),
```

The message must not contain the phrase `backup needs`. My first draft said "how many plates the backup needs", and the test's `!contains("backup needs")` assertion caught it. That is the assertion working, but it is also why the wording above avoids the phrase.

In `parse_line` (`:210-212`):

```rust
                Err(md_codec::Error::WireVersionMismatch { got }) => {
                    Err(BundleError::Md1WireVersion(s.to_string(), got))
                }
```

In `run_bundle`, replace the `if let Ok(d) = …` block (`:371-375`) and keep the comment above it:

```rust
        //
        // F-635 (SPEC_liana_unspendable_internal_key §8b): this was
        // `if let Ok(d) = …`, so a plate that did not decode was SKIPPED while
        // still being pushed below -- `key_slots`, `keyless_template` and
        // `hashlock_kinds` kept their zero values and the checklist stated a
        // count computed from nothing. Measured on me 0.10.0: a version-8 2-key
        // TEMPLATE plate printed "backup needs 1 public plate" with no template
        // note. The chunked path below already refuses via `?`; this is now
        // the same rule for the unchunked shape.
        let d = md_codec::decode::decode_md1_string(s).map_err(|e| match e {
            md_codec::Error::WireVersionMismatch { got } => {
                BundleError::Md1WireVersion(s.clone(), got)
            }
            e => BundleError::Md1Undecodable(s.clone(), e),
        })?;
        hashlock_kinds.extend(descriptor_hash_kinds(&d));
        key_slots = key_slots.max(d.n as usize);
        keyless_template |= !d.is_wallet_policy();
```

Update the two unit tests that name the old shape. In `no_bundle_error_display_leaks_the_input_body` (`:558`), replace `BundleError::Md1WireVersion(CANARY.into()),` with:

```rust
            BundleError::Md1WireVersion(CANARY.into(), 12),
            BundleError::Md1Undecodable(
                CANARY.into(),
                md_codec::Error::BitStreamTruncated {
                    requested: 14,
                    available: 1,
                },
            ),
```

This adds the new variant to the B8 no-leak sweep. Its `Display` interpolates only the codec error, never `s`.

In `parse_line_md1_wrong_wire_version_rejected` (`:1071`), the pattern becomes `Err(BundleError::Md1WireVersion(_, 0))`. The fixture's version nibble is 0 (`:1024-1033`), so this assertion now also pins that the version is carried out.

- [ ] **Step 4: Run.** The suite gives **671 run, 671 passed, 2 skipped** (666 + 5). No pre-existing test changed its verdict: the unchunked fixtures in `bundle.rs`/`tests/cli.rs` (`md1yqpqqxqq8xtwhw4xwn4qh`) decode, which was measured with the whole suite green after this step. Then run fmt and clippy.

**Mutations that must be KILLED (measured):**
- **M2**, swallow the error again (drop `?` and wrap the three statements in `if let Ok(d) = d`): kills `bundle_refuses_…_unsupported_version`, `bundle_refuses_…_does_not_decode`, `a_version_4_origin_less_template_…` and `a_good_plate_…`.
- **M3**, delete the `WireVersionMismatch` arm in `run_bundle`'s `map_err`: kills `bundle_refuses_…_unsupported_version`. That test's `!contains("does not decode")` is what catches it, because `Md1Undecodable`'s `Display` would otherwise still print the version through `{e}`.
- **M4**, `Md1WireVersion(s.to_string(), 0)` in `parse_line`: kills `bundle_names_the_version_of_a_chunk_it_cannot_read`.

- [ ] **Step 5: Commit.** Stage `crates/me-cli/src/bundle.rs` and `crates/me-cli/tests/f449_stage4a.rs`. Subject: `me bundle: an md1 plate that does not decode is refused, never counted from nothing (F-635)`. The body includes F7 (live at v4) and the behaviour change for origin-less single templates.

---

## Task 3: `me sysw` reports an unsupported wire version rather than reducing it to "unconfirmed" (SPEC §6a, §9a item 4, §8.9 row)

**Files:**
- Modify: `crates/me-cli/src/sysw/record.rs:210-275`
- Modify: `crates/me-cli/src/main.rs:2110-2116`, `:2296-2307`
- Modify: `crates/me-cli/src/sysw/expect.rs`
- Modify: `crates/me-cli/tests/f449_stage4a.rs` (add)

**Interfaces:**
- Consumes: Task 2's fixtures `V12_SINGLE`, `V4_UNDECODABLE`, `V12_NAMED` and `v12_chunk()`.
- Produces:
  - `pub enum sysw::record::Unconfirmed { Undecodable, UnsupportedWireVersion(u8) }` (Debug, Clone, Copy, PartialEq, Eq);
  - `Unconfirmed::version_note(&self) -> Option<String>`;
  - `pub fn sysw::record::mdmk_unconfirmed_why(&[String]) -> Vec<(usize, Unconfirmed)>`, sorted by index;
  - `mdmk_unconfirmed` keeps its signature and becomes the projection of `mdmk_unconfirmed_why`, so the frozen callers in `sysw/vectors.rs:107,239` are untouched;
  - `sysw::expect::Unmet::UnreadableVersion { kind: Kind, index: usize, got: u8 }`, which is a public break.

**Scope ruling, stated.** SPEC §8.9's row names `sysw/record.rs:251-252`. F10 measured **three** surfaces that read that walk. `--expect descriptor` is the third. It turns the same `.is_ok()` into *"records of that kind ARE present, but the set does not reassemble"*, which is a false statement about a whole card, and that is §6a's defect shape exactly. Fixing `pack` and `show` while leaving `--expect` would be fixing the rule in two of its three copies. *Cost if wrong:* one extra `Unmet` variant that a reviewer could ask to be deferred.

- [ ] **Step 1: Write the failing tests.** Append to `crates/me-cli/tests/f449_stage4a.rs`:

```rust
// ---- Task 3: §6a on `me sysw` -- REPORTED, never reduced to "unconfirmed". -

#[test]
fn the_walk_keeps_the_version_for_both_shapes() {
    use mnemonic_engrave::sysw::record::{mdmk_unconfirmed_why, Unconfirmed};
    for s in [V12_SINGLE.to_string(), v12_chunk()] {
        assert_eq!(
            mdmk_unconfirmed_why(&[s.clone()]),
            vec![(0, Unconfirmed::UnsupportedWireVersion(12))],
            "{s}"
        );
    }
    assert_eq!(
        mdmk_unconfirmed_why(&[V4_UNDECODABLE.to_string()]),
        vec![(0, Unconfirmed::Undecodable)]
    );
}

#[test]
fn pack_names_the_wire_version_and_does_not_call_it_undecodable() {
    let r = me(&["sysw", "pack", "--no-passphrase", V12_SINGLE], "");
    assert_eq!(r.code, 0, "D6: it WARNS and proceeds: {}", r.err);
    assert!(
        r.err.contains(&format!(
            "record 0, as given (records count from 0): an md1 this build of me does not \
             read -- {V12_NAMED}"
        )),
        "{}",
        r.err
    );
    assert!(r.err.contains("SECRET"), "{}", r.err);
    assert!(!r.err.contains("could not decode"), "{}", r.err);
}

#[test]
fn show_names_the_wire_version_beside_the_record() {
    let dir = tempfile::tempdir().unwrap();
    let bin = dir.path().join("p.bin");
    let packed = me(
        &[
            "sysw",
            "pack",
            "--no-passphrase",
            V12_SINGLE,
            "--out",
            bin.to_str().unwrap(),
        ],
        "",
    );
    assert_eq!(packed.code, 0, "{}", packed.err);
    let r = me(&["sysw", "show", bin.to_str().unwrap()], "");
    assert_eq!(r.code, 0, "{}", r.err);
    assert!(
        r.out.contains(&format!(
            "public record 0: md1/mk1 — unconfirmed — engraveable, but the device REPLACES \
             the legend; an md1 this build of me does not read -- {V12_NAMED}"
        )),
        "{}",
        r.out
    );
}

/// `--expect descriptor` read the same walk and called a whole single card
/// "present, but the set does not reassemble" -- false for this card.
#[test]
fn expect_descriptor_names_the_version_instead_of_calling_it_incomplete() {
    let dir = tempfile::tempdir().unwrap();
    let bin = dir.path().join("p.bin");
    let r = me(
        &[
            "sysw",
            "pack",
            "--no-passphrase",
            "--expect",
            "descriptor",
            V12_SINGLE,
            "--out",
            bin.to_str().unwrap(),
        ],
        "",
    );
    assert_eq!(r.code, 4, "{}", r.err);
    assert!(!bin.exists(), "nothing written");
    assert!(r.err.contains(V12_NAMED), "{}", r.err);
    assert!(!r.err.contains("does not reassemble"), "{}", r.err);
}
```

- [ ] **Step 2: Run, and see them FAIL.** `the_walk_…` fails to compile until Step 3 (`mdmk_unconfirmed_why` and `Unconfirmed` do not exist yet). Once the record.rs half compiles, the three CLI tests fail on the old wording.

- [ ] **Step 3: `sysw/record.rs`.** Replace the head of `pub fn mdmk_unconfirmed` (`:210-211`) so the function becomes a projection, followed by the new items:

```rust
pub fn mdmk_unconfirmed(records: &[String]) -> Vec<usize> {
    mdmk_unconfirmed_why(records)
        .into_iter()
        .map(|(i, _)| i)
        .collect()
}

/// Why an md1/mk1 record is unconfirmed.
///
/// F-449 stage 4a (SPEC_liana_unspendable_internal_key §6a, §8.9): the walk
/// below reduced every decoder answer to `.is_ok()`, so a well-formed md1 at a
/// wire version this build does not read was reported exactly like a broken
/// one -- "could not decode", with nothing naming the version. The version is
/// the one fact the operator needs (a newer `me`, or newer firmware, reads it),
/// so it is carried out rather than discarded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unconfirmed {
    /// The real decoder refused it: an incomplete set, a foreign chunk, or a
    /// string that passes its checksum and does not decode.
    Undecodable,
    /// An md1 whose header carries a wire version this build does not read.
    /// The `u8` is that version.
    UnsupportedWireVersion(u8),
}

impl Unconfirmed {
    /// The operator-facing reason, for the `UnsupportedWireVersion` case; the
    /// accepted set comes from md-codec's own `Display`, never a second copy.
    pub fn version_note(&self) -> Option<String> {
        match self {
            Unconfirmed::Undecodable => None,
            Unconfirmed::UnsupportedWireVersion(got) => Some(format!(
                "an md1 this build of me does not read -- {}",
                md_codec::Error::WireVersionMismatch { got: *got }
            )),
        }
    }
}

/// Map an md-codec decode result to a verdict: `None` is confirmed.
fn md_verdict<T>(r: Result<T, md_codec::Error>) -> Option<Unconfirmed> {
    match r {
        Ok(_) => None,
        Err(md_codec::Error::WireVersionMismatch { got }) => {
            Some(Unconfirmed::UnsupportedWireVersion(got))
        }
        Err(_) => Some(Unconfirmed::Undecodable),
    }
}

/// [`mdmk_unconfirmed`] with the reason kept: `(index, why)` for every
/// unconfirmed record, sorted by index. [`mdmk_unconfirmed`] is this with the
/// reasons dropped, so the two can never disagree about WHICH records.
pub fn mdmk_unconfirmed_why(records: &[String]) -> Vec<(usize, Unconfirmed)> {
    use std::collections::BTreeMap;
```

The old body continues from here unchanged, apart from these edits:
- `let mut out: Vec<usize> = Vec::new();` becomes `let mut out: Vec<(usize, Unconfirmed)> = Vec::new();`
- `None => out.push(i),` becomes `None => out.push((i, Unconfirmed::Undecodable)),`
- the decode match and its tail become:

```rust
        let verdict = match (hrp, csid) {
            ('d', Some(_)) => md_verdict(md_codec::reassemble(&set)),
            ('d', None) => md_verdict(md_codec::decode_md1_string(set[0])),
            // R2/R6 (`design/agent-reports/impl-me-cli-csid-warning.md`):
            // bind the card on success (the verdict is still `None` --
            // confirmed -- iff decode succeeded) to
            // recompute-and-warn on a stamped/derived chunk_set_id
            // mismatch. The mutation gate for THIS surface is deleting the
            // `warn_chunk_set_id_mismatch` line in the `Ok` arm.
            ('k', _) => match mk_codec::decode(&set) {
                Ok(card) => {
                    crate::csid_warn::warn_chunk_set_id_mismatch(
                        crate::csid_warn::chunk_set_id_comparison(&set, &card),
                    );
                    None
                }
                Err(_) => Some(Unconfirmed::Undecodable),
            },
            _ => Some(Unconfirmed::Undecodable),
        };
        if let Some(why) = verdict {
            out.extend(idxs.into_iter().map(|i| (i, why)));
        }
    }

    out.sort_unstable_by_key(|&(i, _)| i);
    out
}
```

The R2/R6 comment's "`.is_ok()`'s CONTROL-FLOW meaning … still `true`" is rewritten above. Left as it was, it would describe a mechanism the code no longer has.

**Why the version read is trustworthy on both shapes** (F9, measured). A **chunked** v12 record fails `ChunkHeader::read` inside `seal::record::chunk_key`, so it groups as its own card `('d', None)`. `decode_md1_string` then dispatches on bit 0 to `reassemble_with_opts(&[s])`, which reads the chunk header's own version field, `got: 12`. An **unchunked** v12 record goes to `Header::read`, which also gives `got: 12`. A `('d', Some(csid))` group contains only records whose chunk header already parsed at a supported version, so the bit-shifted "version" misread that ruling 7 had to cut out of `md repair` cannot arise here.

- [ ] **Step 4: `main.rs`.** `report_unconfirmed` (`:2110-2116`):

```rust
    for (i, why) in mnemonic_engrave::sysw::record::mdmk_unconfirmed_why(records) {
        match why.version_note() {
            // F-449 stage 4a (SPEC §6a): name the version, and do not claim the
            // plate is broken -- a newer `me`, or newer firmware, may read it.
            Some(note) => eprintln!(
                "me: record {i}, as given (records count from 0): {note}. It cannot be \
                 confirmed here, and a device that cannot read it either will treat it \
                 as a SECRET"
            ),
            None => eprintln!(
                "me: record {i}, as given (records count from 0): an md1/mk1 this tool \
                 could not decode; the device will treat it as a SECRET"
            ),
        }
    }
```

In `print_records`, `:2296` becomes `let mdmk_unconfirmed = sysw::record::mdmk_unconfirmed_why(&records);` and the `MdMk` arm (`:2305-2308`) becomes:

```rust
            sysw::record::Class::MdMk => {
                let why = mdmk_unconfirmed
                    .iter()
                    .find(|(j, _)| *j == i)
                    .map(|&(_, w)| w);
                let state = confirmation_state(why.is_some());
                match why.and_then(|w| w.version_note()) {
                    // F-449 stage 4a (SPEC §6a): the same state, with the reason.
                    Some(note) => println!("public record {i}: md1/mk1 — {state}; {note}"),
                    None => println!("public record {i}: md1/mk1 — {state}"),
                }
            }
```

**Keep it ONE call per invocation.** The walk emits the R2/R6 `csid` warning as a side effect, and `sysw_cli.rs:113-139` pins that no surface double-warns. `mdmk_unconfirmed_why` **replaces** the old call. It must never be added beside it.

- [ ] **Step 5: `sysw/expect.rs`.** Line 68 becomes `use super::record::{card_hrp, mdmk_unconfirmed_why, Class, Unconfirmed};`. `Unmet` gains:

```rust
    /// A record of this kind is present, but it is an md1 at a wire version
    /// this build does not read (F-449 stage 4a, SPEC §6a). Reported apart
    /// from [`Unmet::Incomplete`], whose "does not reassemble" would be a
    /// false statement about a card that may be whole.
    UnreadableVersion { kind: Kind, index: usize, got: u8 },
```

In `check`, `let mdmk_bad = mdmk_unconfirmed(records);` becomes `let mdmk_bad = mdmk_unconfirmed_why(records);`, and the `Kind::Descriptor | Kind::Cosigner` arm becomes:

```rust
            Kind::Descriptor | Kind::Cosigner => {
                let want = if kind == Kind::Descriptor { 'd' } else { 'k' };
                let mut broken = Vec::new();
                for &(i, why) in &mdmk_bad {
                    if card_hrp(&records[i]) != Some(want) {
                        continue;
                    }
                    match why {
                        Unconfirmed::UnsupportedWireVersion(got) => {
                            out.push(Unmet::UnreadableVersion {
                                kind,
                                index: i,
                                got,
                            })
                        }
                        Unconfirmed::Undecodable => broken.push(i),
                    }
                }
                broken
            }
```

`describe` gains the arm:

```rust
        Unmet::UnreadableVersion { kind, index, got } => format!(
            "--expect {} was not met: record {index} (records count from 0) is of that \
             kind, but it is an md1 this build of me does not read -- {}.\n      \
             It may be a whole card; this build cannot confirm it. Nothing was written.",
            kind.name(),
            md_codec::Error::WireVersionMismatch { got: *got }
        ),
```

In the test module, `the_mdmk_walk_is_blind_to_mt1_and_that_is_why_there_are_three` (`:312`) now calls `crate::sysw::record::mdmk_unconfirmed(&half)`, because the module-level import no longer brings in the name.

- [ ] **Step 6: Run.** The suite gives **675 run, 675 passed, 2 skipped** (671 + the 4 tests in this task; 11 new across Tasks 1-3). Pre-existing guards that MUST stay green, and were measured green:
  - `the_descriptor_show_block_leaves_every_other_container_byte_identical`. The `show` capture's `unconfirmed` line is an incomplete set (`Undecodable`), so it stays byte-identical.
  - `pack_warns_once_per_unconfirmed_record_and_still_succeeds`.
  - the `the_two_walks_agree_wherever_both_have_an_answer` pin.
  - `sysw/vectors.rs`'s frozen `mdmk_unconfirmed` vectors.

**Mutations that must be KILLED (measured):**
- **M5**, `md_verdict` maps `WireVersionMismatch` to `Undecodable`: kills `the_walk_…`, `pack_…`, `show_…` and `expect_…`.
- **M6**, `why.version_note().filter(|_| false)` in `report_unconfirmed`: kills `pack_…`.
- **M7**, the same in `print_records`: kills `show_…`.
- **M8**, `Unconfirmed::UnsupportedWireVersion(_) => broken.push(i)` in `check`: kills `expect_…`.

- [ ] **Step 7: Commit.** Stage `crates/me-cli/src/sysw/record.rs`, `crates/me-cli/src/sysw/expect.rs`, `crates/me-cli/src/main.rs` and `crates/me-cli/tests/f449_stage4a.rs`. Subject: `me sysw: an md1 at an unsupported wire version is named, never reduced to "unconfirmed" (F-449 §6a)`.

---

## Task 4: records. F-635 closed, three follow-ups filed, and the spec made true again

This is the stage-2 plan's standing Task 7 rule: every stage ends with a sweep for what it made FALSE.

**Files:** `design/FOLLOWUPS.md`, `design/SPEC_liana_unspendable_internal_key.md`. Both are in mnemonic-engrave.

- [ ] **Step 1: Close F-635.** Its first body line (`design/FOLLOWUPS.md:19212`) becomes `**Status:** CLOSED <date> in mnemonic-engrave <Task 2 SHA> (F-449 stage 4a Task 2).` Follow it with one sentence: the unchunked path refuses (`Md1WireVersion` names the version, `Md1Undecodable` names the codec error), the defect was live at v4 as well as v8 (F7), and it is pinned by the five Task-2 tests in `tests/f449_stage4a.rs`. Then run `scripts/followups-status.sh`, which must print `OK`.

- [ ] **Step 2: File three follow-ups.** Take the next three free numbers, re-measured at write time, because stages 3 and 4 may file entries concurrently. F-647..F-650 were taken concurrently by the controller; the three entries below were filed by the controller as F-651, F-652 and F-653 (engrave `HEAD` after this plan commit), so this step VERIFIES they exist rather than filing them. Each entry gets `**Status:** OPEN` as its first body line and an **owning phase**. Whichever number F-651 lands on, the Task 1 comment in `descriptor_seam.rs` must cite that number. Grep for it.
  - **F-651 — `md_codec::Descriptor::derive_address` panics on a descriptor `md_codec::split` admits** (`sh(sortedmulti(2, 16 keys))`, 547-byte script, rust-miniscript `ff4732e5` `src/lib.rs:355`). It was `Err` under 13.1.0. The owning phase is **the next descriptor-mnemonic release**, and the fix is Rust-primary, with a test vector. The entry includes the reproduction from F5 and states that `me` is unaffected in production (admission first). Tags: `#descriptor-mnemonic` `#md-codec` `#panic`. After it lands, Task 1's skip in `descriptor_seam.rs` can return to deriving and assert `Err`.
  - **F-652 — `me bundle` refuses origin-less templates that `md decode` reads.** Both the unchunked path (since Task 2) and the chunked path (before it) use strict decode. `DecodeOpts::partial()` (md-codec `decode.rs:55`) would decode them, and `key_slots`/`keyless_template`/`hashlock_kinds` do not depend on origins, so the count could be stated correctly instead of being refused. The owning phase is none: this is ownerless residue and a UX improvement, because the refusal is safe.
  - **F-653 — the fuzz workspace cannot build.** `crates/me-cli/fuzz/Cargo.lock` is frozen at `mnemonic-engrave 0.3.0`/`md-codec 0.40.0` (`cargo check --locked` fails there at `8aea0d36`), and since Task 1 it also needs the root's `[patch.crates-io]`, because patches do not cross workspaces. Ownerless.

- [ ] **Step 3: Reconcile the spec.** Search the spec for the OLD claims this stage falsified and edit each one. Each edit here was measured:
  - **§9a item 1:** *"Pointing … at the local md-codec 0.45.1"* is stale. The pin is the git rev `cf35d61a`, md-codec 0.47.0. The item should also record F3: the patch alone is ignored until `cargo update -p miniscript`.
  - **§9a item 2:** record the measured blast radius, which is one struct literal (`md1.rs:351-357`, 2 errors), plus F5's Err→panic in one test (F-651).
  - **§8.9's two `me` rows:** mark them DONE with the Task 2 and Task 3 SHAs, in the same form as the `md repair` row (`2 — DONE (…)`).
  - **§9's stage-4a row:** add a **Status:** clause naming the SHAs. Also add that the `--expect` surface was a third reader of the walk and was fixed with it.
  - **§8b:** it says the fail-open is "latent today". Replace that: it was live at v4 too (F7).

- [ ] **Step 4: Re-run the citations** with a command, not by eye: `scripts/plan-cite-check.sh design/SPEC_liana_unspendable_internal_key.md`. Paste the output into the commit. Read every line it prints for the sections you touched, because printing a line proves it exists, not that it says what the sentence claims.

- [ ] **Step 5: Commit records separately from code.** Stage `design/FOLLOWUPS.md` and `design/SPEC_liana_unspendable_internal_key.md`. Subject: `records: F-449 stage 4a -- F-635 closed, F-651..F-653 verified, spec §8b/§8.9/§9/§9a reconciled`.

- [ ] **Step 6: Whole-diff review (mandatory, risk set: a funds-adjacent completeness claim).** The controller dispatches one **opus** adversarial execution review over `git diff master..f449-stage4a`. The report goes to `design/agent-reports/f449-stage4a-whole-diff.md`, written by the agent. The brief states what is already machine-verified (the execution gate and M1-M10 above), so reviewer budget goes to what tools cannot reach:
  - (a) is there a fourth reader of the walk;
  - (b) can any path still print a plate count after a decode failure;
  - (c) does the wording mislead an operator holding a mixed-version fleet.

  Persist, fold, gate and re-dispatch until it returns 0C/0I.

- [ ] **Step 7: Merge.** Merge with `git merge --no-ff f449-stage4a` into master, from the main checkout. Then run the suite once on master and record the count.

---

## Task 5: release `me` 0.11.0

**Files:** `crates/me-cli/Cargo.toml:3`, `Cargo.lock` (the `mnemonic-engrave` stanza), `crates/me-cli/CHANGELOG.md`.

- [ ] **Step 0: PRECONDITION. Stop if stage 3 has not landed in fork main.** `me sysw`'s "confirmed" is a prediction of what the device will decode. After Task 1, `me` confirms a v8 card, but fork main `7b6f2fb`, and every image flashed from it, cannot read v8. The device would treat that card as a SECRET and replace its legend, while `me sysw show` said "confirmed". `me` cannot know which firmware is flashed, so the only guard is ordering, and SPEC §9 already orders 3 → 4 → 4a. **Check:** the fork's `md/` accepts version 8, i.e. `git -C /scratch/code/shibboleth/seedhammer log --oneline origin/main | grep -i 'stage 3'` returns the stage-3 merge, **and** `cd /scratch/code/shibboleth/seedhammer && /scratch/code/shibboleth/.toolchain/go/bin/go test ./md/ -run 'Version8|WireVersion'` shows v8 tests that pass. If either is missing, Tasks 0-4 may be merged, but **do not tag**. Report the release as blocked on stage 3, and do not install the master build as the local `me` (Step 8) either. *Cost if wrong:* an operator packs a v8 card, `me` calls it confirmed, and the device blanks its legend.

- [ ] **Step 1: The number.** `0.10.0 → 0.11.0`. Under the pre-1.0 convention the minor number is the breaking axis (the `acbfcc93` message). This release breaks public API: `BundleError::Md1WireVersion(String)` becomes `(String, u8)`, the new variants `BundleError::Md1Undecodable` and `sysw::expect::Unmet::UnreadableVersion` are additions to exhaustive public enums, and the new `sysw::record::Unconfirmed`/`mdmk_unconfirmed_why` are also public. The operator-visible change is that `me bundle` now refuses a plate 0.10.0 accepted (F7).

- [ ] **Step 2: CHANGELOG. Repair the missing `[0.10.0]` first (F13), then add 0.11.0.** In `crates/me-cli/CHANGELOG.md`:
  1. Insert `## [0.10.0] - 2026-09-16` directly below `## [Unreleased]`. The existing two `### Changed` entries (F-493, F-504) move under it, since both are ancestors of `v0.10.0`. Add that release's headline change, with bullets taken **from `git show -s acbfcc93`**, not invented: the §6 hash-kind record grammar; `ComposerRecord::Hash` carrying a `HashLock`; the §8n refusal naming the kind's width; and `me sysw show` naming the record's kind. Also its "NOT BREAKING on the wire" note.
  2. Add `## [0.11.0] - <release date>` above it, with:
     - **Changed:** md-codec 0.47.0 by git rev plus the miniscript patch. `me` reads version-8 md1 (F-449 kind 1), and it did not before.
     - **Changed:** a testnet `tpub` in a `key:` record is refused as unsupported. This is `1cbecbfd` and has no CHANGELOG line yet.
     - **Fixed:** F-635, with the v4 origin-less template called out as **newly refused**.
     - **Fixed:** `me sysw pack`/`show`/`--expect` name an unsupported wire version.
     - **Breaking (library):** the enum changes from Step 1.

     Keep an empty `## [Unreleased]` at the top. Match the file's `## [x.y.z] - YYYY-MM-DD` form, with an ASCII hyphen.

- [ ] **Step 3: Version.** `crates/me-cli/Cargo.toml:3` becomes `version = "0.11.0"`. Then run `cargo update -p mnemonic-engrave --offline` or an ordinary `cargo check`. `git diff Cargo.lock` must show only the `mnemonic-engrave` stanza's version line.

- [ ] **Step 4: Gate.** Capture the suite once to a file and grep it: `cargo nextest run --locked --no-fail-fast`. Then `cargo test --locked`, which is CI's command, `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo build --locked --bin me && $CARGO_TARGET_DIR/debug/me --version`, which must print `me 0.11.0`.

- [ ] **Step 5: Release commit on master.** Stage `crates/me-cli/Cargo.toml`, `Cargo.lock` and `crates/me-cli/CHANGELOG.md`, and commit with `-F`. Subject: `me 0.11.0 -- reads md1 wire version 8; never counts a plate it cannot decode (F-635)`. The body carries the Step 4 counts. `REL=$(git rev-parse HEAD)`.

- [ ] **Step 6: Push, with master FROZEN.** Empty your hands first: `git status --short` must be empty of tracked changes, and `git ls-remote origin refs/heads/ci/staging` must be empty. Then run `scripts/push-via-staging.sh master` in the foreground. Any `Bypassed rule violations` line is a failure: stop and do not tag. After `git fetch origin`, `origin/master` must equal `$REL`. **No commits to master between the staging push and the final push.**

- [ ] **Step 7: Tag and verify.**
  - Write the tag message to a file. The first line is `me v0.11.0 -- reads md1 wire version 8; a plate it cannot decode is refused, not counted`, followed by two lines from the `[0.11.0]` section.
  - `git tag -a v0.11.0 "$REL" -F <file>`. `git cat-file -p v0.11.0 | head -1` must show `object $REL`. Then `git push origin v0.11.0`.
  - Watch the tag run: `gh run list --repo bg002h/mnemonic-engrave --commit $REL --json databaseId,event,status,conclusion`, then `gh run watch <id> --repo bg002h/mnemonic-engrave --exit-status`. Record the per-job conclusions, including `assemble + sign + release`. **Never move or delete a pushed tag.**
  - Check the assets: `gh release view v0.11.0 --repo bg002h/mnemonic-engrave --json url,assets` should list 7 assets.
  - In `/scratch/code/shibboleth/.tmp/me-v0.11.0/`, run `minisign -Vm SHA256SUMS -P RWQPmgBXsuw5yi8W0SfDr8KF+IqY/Z5U2p724emSODS1UPfJBP3agbKW` and `sha256sum -c --ignore-missing SHA256SUMS`, untar the linux-amd64 archive, and confirm `./*/me --version` prints `me 0.11.0`.
  - **Three behaviour checks on the released binary**, each quoted in the report:
    1. `echo md1cpfdsssj6tvyywtsqrq0zjs4n7gdve74ar402 | ./*/me bundle`: exit 0, and stderr contains `this md1 is a TEMPLATE` and `declares 2 key slots`.
    2. `echo md1uzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg | ./*/me bundle`: exit 4, stderr names `got 12; accepted versions: 4, 8`, and there is no `backup needs`.
    3. `./*/me sysw pack --no-passphrase md1uzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg --out ./p.bin`: exit 0, and stderr names `got 12` and no `could not decode`.

  A miss on any of these means NOT RELEASED, even though the tag exists.

- [ ] **Step 8: Refresh the local binaries.** This is standing permission ("keep local binaries current"), and stage 5 depends on it (F15). Run `cargo install --locked --path crates/me-cli`, then `scripts/build-preview.sh` and copy `target/release/me-preview` beside the installed `me`, because `me-preview`'s version must equal `me`'s (F14). Check it with `command -v me && me --version`, measuring by path, not by name.

- [ ] **Step 9: Records.** Continuity: stage 4a is SHIPPED, naming the tag and the SHAs, and stage 5 is next. SPEC §9's stage-4a row gets a status note naming `v0.11.0`. The release report goes to `design/agent-reports/push-me-v0.11.0-release.md`, agent-written if a push agent runs it.

---

## Self-Review

**Spec coverage.**

| Spec item | Where it is done |
| --- | --- |
| §9a item 1, the unpin plus `[patch.crates-io]` | Task 1 Steps 3-4, including F3, which the spec did not know |
| §9a item 2, §3f's repairs | Task 1 Step 5 (one literal, F4) and Step 6 (the test the patch breaks, F5) |
| §9a item 3, §8b / F-635 | Task 2 |
| §9a item 4, `sysw/record.rs:251-252` | Task 3, extended to the third reader of the walk, `--expect` (scope ruling stated in Task 3) |
| §8.9's `me` fail-open row | Task 2's five tests |
| §8.9's `me` record-confirmation row | Task 3's four tests |
| The spec's anti-gate ("me round-trips a version-8 payload" is NOT a gate) | Respected. The only v8 tests are Task 1's two, and both FAIL on the status quo (M1, measured) |
| The release | Task 5 |
| Stage 5's dependency | Task 5 Step 8 (F15) |

FOLLOWUPS owned by stage 4a: F-635 only (F16), closed in Task 4.

**Deliberately absent, owned elsewhere:** the Go port and the device (stages 3 and 4), `demo/sh2` (stage 5), the toolkit pin (F-642), the md-codec panic (F-651, descriptor-mnemonic), partial-decode counting (F-652), and the fuzz crate (F-653).

**Placeholder scan.** Placeholders appear in three kinds of spot:
- `<date>`, `<Task 2 SHA>` and `<release date>` are filled at execution time.
- The CHANGELOG bullets in Task 5 Step 2 are specified by source (`git show -s acbfcc93`, `1cbecbfd`), not written out here. This is deliberate: a release note transcribed twice is a second copy that drifts.
- F-651, F-652 and F-653 were filed by the controller when this plan was committed; Task 4 Step 2 verifies them.

**Type consistency.** These names are the same in every task, and all were compiled together in the probe:
- `Unconfirmed::{Undecodable, UnsupportedWireVersion(u8)}`, `mdmk_unconfirmed_why -> Vec<(usize, Unconfirmed)>` and `version_note() -> Option<String>`;
- `BundleError::Md1WireVersion(String, u8)` and `Md1Undecodable(String, md_codec::Error)`;
- `Unmet::UnreadableVersion { kind, index, got }`.

**Build-gate coverage, stated rather than assumed.**
- `./scripts/plan-build-gate.sh` and `./scripts/plan-build-gate-me.sh` both **exit 3 (EXTRACTED NOTHING)** on this plan, and that is correct: every block edits an existing file or goes into a test file neither gate knows.
- The executed gate is the scratch-copy run described in the header: clippy/fmt/nextest/`cargo test`/`cargo metadata --locked` all green, 675/675, with M1-M10 all killed.
- **It does not cover:**
  - prose matching blocks;
  - the CHANGELOG's accuracy;
  - the release workflow;
  - Task 5 Step 0's device precondition;
  - whether the fold of any review keeps the blocks compiling. A fold re-earns the gate: re-apply to a scratch copy and re-run.
- `scripts/plan-cite-check.sh` on this plan was **measured: exit 1, 19 ok / 20 DANGLING, and all 20 are false alarms.** The script joins citations to the repo ROOT. This plan cites me-cli files as `bundle.rs:…`/`sysw/record.rs:…`/`src/descriptor/md1.rs:…` (relative to `crates/me-cli/`), and cites descriptor-mnemonic and rust-miniscript by bare basename (`error.rs:34`, `decode.rs:55`, `derive.rs:148`, `to_miniscript.rs:670`, `src/lib.rs:355`). **Do not read its exit code as a verdict.** Every me-cli citation in this plan was instead resolved by printing the line at `8aea0d36` (`sed -n`) and reading it against the claim. That pass moved `md1.rs:353-357` to `:351-357`, because the replaced block starts at `Script::P2TR => Node {`.

**The one risk worth naming.** Task 2 turns a silent miscount into a refusal, and that refusal lands on a plate an operator can legitimately hold: an origin-less template from `md encode` (F7), which `md decode` reads with a VERIFY-ME note. That is the correct direction, because the old answer was F-602's "cut one plate and the wallet is gone". But it is new friction, and the CHANGELOG must say so plainly. F-652 is the route to counting it correctly instead of refusing it.
