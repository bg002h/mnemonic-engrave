# F-449 stage 3 — the Go port: wire version 8 and the three-state internal key in the fork's `md/`

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** the SeedHammer II firmware reads a wire-version-8 (kind 1, "Liana unspendable") md1 plate correctly. It computes the same identities Rust computes, refuses to show an address it cannot derive, and names an unsupported wire version instead of calling the plate "not md1".

**Architecture:** the fork's `md/` package mirrors md-codec's tree codec. This stage ports md-codec 0.47.0's wire change into it. `trBody`'s `isNums bool` becomes a three-state `InternalKeyKind`, and a version parameter is threaded through `readNode`/`writeNode` (the kind bit exists only at version 8). The version is derived from the tree at every writer, including the identity hashes. The cross-language contract is a set of **Rust-primary kind-1 vectors that do not exist yet**. Task 1 adds them to md-codec's MANIFEST first, and Task 5 vendors them. The device side changes in three places only: the one address-derivation caller switches on the kind and refuses kind 1, one message helper names an unsupported version on the gather, inspect and bundle surfaces, and the device's own kind-1 derivation stays stage 4.

**Tech stack:** Go 1.26.7 (`/scratch/code/shibboleth/.toolchain/go`), TinyGo via the fork's nix flake (size only), Rust 1.85.0 for the Task 1 prerequisite in descriptor-mnemonic.

**Spec:** `design/SPEC_liana_unspendable_internal_key.md`. §9's stage-3 row is binding: the three-state `EmitTapLeavesChunks` (§7a.1), the version-derived identity at the three Go sites (§3e), §6a's `gatherIgnored` split, and the provenance pin. The gate is the §8 vectors in Go, including §8.4's dispatch leg, §8.5's identity leg and §8.9's stage-3 row.
**Continuity:** `design/CONTINUITY_f449_stage2.md`. Its hard-won facts are not re-derived here.

**Baseline revisions (record them; a staleness check needs all three):**
- descriptor-mnemonic `main` **`cf35d61a`** (md-codec 0.47.0, md-cli 0.19.0, tag `descriptor-mnemonic-md-cli-v0.19.0`).
- seedhammer fork `main` **`7b6f2fb`** (clean).
- mnemonic-engrave `master` **`bebb532a`**. It moved from `8aea0d36` while this plan was being written, which is why it is recorded here.

**Worktrees:**
- fork: `/scratch/code/shibboleth/sh-worktrees/f449-stage3`, branch `f449-stage3` off fork `main`.
- descriptor-mnemonic (Task 1): `/scratch/code/shibboleth/dm-worktrees/f449-stage3-vectors`, branch `f449-stage3-vectors` off `main`.

## How this plan was checked before review

Every code block in Tasks 1 and 3 to 6 was applied to **scratch exports** of the two baseline trees and run. Nothing was committed and neither real repo was touched. The results:

| task boundary | what ran | result |
| --- | --- | --- |
| Task 1 (dm) | `scripts/phase-gate.sh` in a `git archive cf35d61a` copy | **exit 0**; nextest **1536 passed / 4 skipped** (was 1536 with 4 red before the harness fixes, see Task 1) |
| Task 3 (fork) | `go vet ./md/`, `go test ./md/`, gui address tests | green; md **167** tests (baseline count) |
| Task 4 | same | green; md **169** |
| Task 5 | same, after re-vendoring with the widened script | green; md **179**; `vendored 260 files, 53 vectors` |
| Task 6 | `scripts/gui-shard-test.sh ./gui/ 24` | `RESULT: ok -- all 1379 tests ran across 24 shards` (29 s), then one more test (below): **1380** |
| all | every non-gui package (`go test $(go list ./... \| grep -v '/gui$')`) | 55 `ok`, 0 fail |
| all | TinyGo size, the CLAUDE.md recipe | baseline **1,655,388 B flash / 63,336 B RAM** at 7b6f2fb, end state **1,658,284 / 63,352** (+2,896 / +16) |

The mutation results in the Self-Review were measured against that end state, and every one applied (checked, not assumed). **The existing `scripts/plan-build-gate-go.sh` cannot check this plan.** It assembles only new files named `md/compose*.go`, `mk/compose*.go`, `sysw/composer_*.go` and `gui/composer_*.go`, and every file here is either an existing file or a new file outside those patterns. So a reviewer should **not** read its exit code as a verdict. The scratch runs above are the gate. What they do **not** cover is listed under "What no gate covers".

## R0 folds — BINDING additions to the tasks below (R0 GREEN, 0C/0I/5M/2N)

Source: `design/agent-reports/f449-plan-stage3-r0.md`. These change the tasks
they name. Where a task's text disagrees, this section wins.

- **m1, Task 1 (Rust first) and Task 5 (Go): pin the READ side of the kind bit.**
  Both languages' polarity tests pin only the write side. R0 measured that a
  reader which always yields Liana at v8 passes md 179/179 and Rust
  1536/1536, and so does a reader that reads a kind bit for a Slot key.
  - **Task 1:** extend Rust's
    `the_kind_bit_polarity_is_pinned_on_the_wire_not_just_round_tripped` to read
    each golden back through `read_node`: `[0x06,0x00]`@8 → `NumsPoint`,
    `[0x07,0x00]`@8 → `LianaUnspendable`, `[0x06]`@4 → `NumsPoint`. Also add
    one Slot-at-v8 golden (no kind bit) read back as the slot.
  - **Task 5:** give `TestKindBitPolarityIsPinnedOnTheWire` the same read-back
    via `readNode(c.want, 0, c.version)` → `c.ik`, with the same Slot-at-v8
    row.
  - **Mutations to prove:** X1 (`if kind {` → `if kind || true {` in
    `readNodeDepth`'s Tr arm), X2 (a kind bit read for a Slot at v8), and the
    Rust twin of X1. Each must redden the new read-back, and none of them does
    today.
- **m2, Review Focus 2 and "What no gate covers" bullet 6:** correct both. On
  the stage-3 inspect screen a kind-1 wallet shows `Complex policy - cannot
  display safely` with its keys, and NO key-path line, NO `Policy id:` and NO
  address. Task 8's sweep records that kind 1 loses the inspect `Policy id:`
  line until stage 4.
- **m3, Task 6 (`TestVersion8CardsAreReadOnEveryRoute`):** the single-card half
  also asserts a POSITIVE: `uiContains(all.String(), "Keys: 3")`, or the
  display title. Absences alone pass on a flow that draws nothing.
- **m4, Task 8's §9 status line:** add "stage 3 alone lets the device copy
  kind-1 cards verbatim without an address (`noAddressLines`); the engrave
  refusal of §7a.3 is stage 4". No code change.
- **m5, Task 5 Step 7:** the recursive `readNodeDepth(r, kiw, depth+1)` sites
  number **10**, not 13. The compiler catches any site you miss.
- **n1, Task 3 Step 4:** the `case md.InternalKeyLianaUnspendable:` Skip arm in
  `gui/taproot_script_path_test.go` is unreachable for both kind-1 vectors,
  because `md.TapLeavesChunks` errors first. Keep it, commented as
  future-proofing. The real gate is `stillUnsupported`.
- **n2:** engrave master has moved again. Re-grep the free follow-up IDs at
  filing time (F-654 and F-655 were still free at R0).

## Global Constraints

- **Rust leads, Go follows (CLAUDE.md).** Every Go behaviour in this plan is a port of md-codec 0.47.0. If a port step meets Rust behaviour that looks wrong, file a Rust follow-up and port Rust as it is. Never diverge in Go.
- **Every tree writer takes its version from `descriptor.wireVersion()`, never a constant.** That covers `encodePayload`, `split`'s chunk header, `WalletDescriptorTemplateId` and `WalletPolicyId`. At version 4 no kind bit is written, so a constant would give two wallets with different addresses one id (SPEC §3e).
- **A caller of `EmitTapLeavesChunks`/`TapLeavesChunks` switches on the kind and refuses a kind it cannot derive.** It never maps a non-Slot kind to the NUMS point (SPEC §7a row 1: a different wallet's addresses).
- **Do NOT port §6's mint refusals into `encodePayload`.** `Reassemble` re-encodes every decoded card to check its chunk-set id (`md/chunk.go` `Reassemble` → `computeEncodingID` → `encodePayload`). A refusal there would make the device reject cards Rust decodes (Rust keeps them mint-only, `encode.rs:242-253`). No Go producer of kind 1 exists in this stage, so the refusals move to stage 4 with the composer port (F-654, Task 8).
- **One phrasing per message.** The unsupported-version text lives in one helper (`md1VersionMessage`), and every surface calls it.
- **The message states an observation.** "This firmware cannot read md1 version N." It never claims the card is intact or came from a newer tool, because a BCH mis-correction can land on any header value (F-643).
- **Three repos; every step names its repo.** Task 1 is **descriptor-mnemonic**. Tasks 2 to 7 are the **fork** (worktree above). Task 8 is **mnemonic-engrave** (`design/FOLLOWUPS.md`, the spec).
- **The vendor scripts must be given the dm path explicitly.** Both `scripts/vendor-compose-vectors.sh` and the new `scripts/vendor-liana-cases.sh` default to `$HERE/../descriptor-mnemonic`. From the worktree that is `/scratch/code/shibboleth/sh-worktrees/descriptor-mnemonic`, which does not exist.
- **Shell:** every command block is bash. The interactive shell on this machine is fish. Commit messages go through `git commit -F <file>` (fish eats backticks), and the file ends with the attribution lines your session's system reminder gives.
- **Go toolchain:** `export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH`. **Rust toolchain:** `export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH` (bare `cargo clippy` is 0.1.98 and fails the baseline).
- **Baseline noise a gate must diff against, not assert empty:**
  - `gofmt -l .` lists exactly five files at 7b6f2fb: `gui/transaction.go`, `gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`. This was measured.
  - `go vet ./gui/` reports two errors at 7b6f2fb: `gui/freetext_sizeproof_golden_test.go:111:13` and `gui/transaction_golden_test.go:104:13`, both "testing.ArtifactDir requires go1.26". `go.mod` says `go 1.25.10`. This was measured. `go vet ./md/` is clean.

## Review Focus

These are inputs the spec implies but no task's tests fully exercise, most likely first:

1. **A board not yet flashed with stage 3 meets a v8 plate.** It still says "Not an md1 descriptor chunk." (measured at 7b6f2fb: `DecodeChunks` → `md: wire version mismatch` → `gatherIgnored`). No code can fix firmware already on a board. This is **documentation only**: Task 8 records it for the stage 5 runbook ("flash before reading kind-1 plates").
2. **A kind-1 wallet inspected on the stage-3 device.** *(Corrected per R0 m2, measured at fork `43294c6`.)* It shows `Complex policy - cannot display safely` with its keys, and NO key-path line, NO `Policy id:` line and NO address (tested: the `stillUnsupported` entries, Task 5). Its kind-0 twin shows `Policy id: c5788d70…`. The key-path line exists only on the template and composer consent screens, where it reads `Key-path: none (script paths only)`, which is true of kind 1. `policyShape` maps every non-Slot kind to `KeyPathNUMS` (`md/policy_shape.go:126`) until stage 4 adds §7's sibling kind. Nothing on screen is false. **Not our concern at stage 3**; the lost `Policy id:` line is recorded in the spec's §9 status and belongs to stage 4.
3. **An operator types a v8 or v12 single string on the keyboard.** The route is the codex32 keyboard, then "Fix?", then `mdmkFlow`. It is exercised: a v12 single string through `mdmkFlow` (Task 6), and a v8 single string that reads (Task 6, positive twin). Correction itself is version-agnostic BCH and unchanged.
4. **A payload or bundle that mixes a v8 card with v4 cards.** Each card is independent. The complete v8 chunk set becomes a bundle card (Task 6, positive twin).
5. **An mk1 key card minted for the kind-0 twin, offered against the kind-1 policy.** Both stub flavours differ (Task 5, `TestKind0AndKind1TwinsNeverShareAnIdentity`). The device's seat and verify screens consume `md.FormAwareStub*` unchanged, so no gui test is added.

---

## File Structure

**descriptor-mnemonic (Task 1):**
- Modify `crates/md-codec/src/test_vectors.rs`: three MANIFEST entries (`liana_taproot`, `keyed_tr_liana_kofn_recovery`, `keyed_tr_liana_nested_two_recoveries`).
- Create `crates/md-codec/tests/vectors/{keyed_tr_liana_kofn_recovery,keyed_tr_liana_nested_two_recoveries}.{bytes.hex,conformance.json,descriptor.json,phrase.txt,template}` and `liana_taproot.{bytes.hex,descriptor.json,phrase.txt,template}`, all generated by `md vectors`.
- Modify `crates/md-codec/tests/wire_version_8.rs`: `wire_version_is_derived_from_the_tree_not_assumed` derives 4 or 8 per vector.
- Modify `crates/md-codec/tests/skeleton_key_conformance.rs`: recognise the kind-1 internal key by the recipe, and re-render with a network.
- Create `crates/md-cli/tests/snapshots/json_snapshots__{decode,inspect}@liana_taproot.snap`.

**fork:**
- Modify `md/md.go`: `InternalKeyKind`, `trBody.ik`, `trBody.isNums()`, `ErrUnsupportedWireVersion` + `WireVersionError`, version constants, `readNode` threading and the kind bit.
- Modify `md/encode.go`: `descriptor.wireVersion()`, `writeNode` threading and the kind bit, `errLianaNeedsVersion8`.
- Modify `md/chunk.go`, `md/template_id.go`, `md/walletpolicyid.go`: version from the tree.
- Modify `md/canonicalize.go`, `md/compose.go`, `md/duplicate_keys.go`, `md/policy_shape.go`, `md/encode_singlesig.go`: mechanical, for the `ik` field.
- Modify `md/tapleaves.go`: `TapLeavesChunks` and `EmitTapLeavesChunks` return `InternalKeyKind`.
- Create `md/liana.go`: the §2 recipe and the leaf-key walker.
- Modify `md/bits.go:1-5`: the package doc, which is the **provenance pin**.
- Modify `md/testdata/README.md`, `md/testdata/compose_vectors.provenance.json` (regenerated), `scripts/vendor-compose-vectors.sh`.
- Create `md/testdata/liana_cases.json`, `md/testdata/liana_cases.provenance.json` and `scripts/vendor-liana-cases.sh`.
- Tests, modified: `md/conformance_keyed_test.go`, `md/compose_vectors_pin_test.go`, `md/testdata_test.go`, `md/md_test.go`, `md/encode_test.go`, `md/compose_pkh_emit_test.go` and the mechanical `isNums` sites.
- Tests, created: `md/liana_test.go`, `md/liana_walk_test.go`, `md/wire_version8_test.go`.
- Modify `gui/policy_address.go`: the three-state switch and the refusal.
- Create `gui/md1_version.go`: the one message.
- Modify `gui/md1_gather.go`, `gui/mk1_inspect.go`, `gui/gui.go`, `gui/bundle.go`, `gui/bundle_flow.go`: the §6a split.
- Tests: `gui/policy_address_test.go`, `gui/taproot_script_path_test.go`, and new `gui/md1_version_test.go`.

**mnemonic-engrave (Task 8):** `design/FOLLOWUPS.md` (F-643 ruling, new F-654), `design/SPEC_liana_unspendable_internal_key.md` (the reconciliation sweep).

**Not touched:** `md/compose.go`'s semantics. Only its one `trBody` literal changes; it gets no Liana selection and keeps its own pin at `66bdf2f4`. Also untouched: `address/`, `sysw/`, and every Rust source file (only tests and vectors change in Task 1).

---

## Task 1 (descriptor-mnemonic): kind-1 vectors in the primary's MANIFEST

**Why this is first, and why it is Rust.** Rust has **no** kind-1 golden anywhere: `crates/md-codec/tests/vectors/` is byte-identical between `b2c5d693` and `cf35d61a` (`git diff --stat` is empty). Its kind-1 tests build a descriptor in-test by swapping the internal key (`tests/common/liana.rs` `kind1_from_vector`) and assert only *distinctness* (`tests/liana_unspendable.rs:251-306`). A Go port can satisfy distinctness while disagreeing with Rust about every byte, for example with the kind bit's polarity inverted on both sides. The cross-language contract has to be Rust output, so it lands in Rust first (the Rust-primary rule).

**Files:** see File Structure. Work in `/scratch/code/shibboleth/dm-worktrees/f449-stage3-vectors`.

- [ ] **Step 1: Create the worktree.**
```bash
git -C /scratch/code/shibboleth/descriptor-mnemonic worktree add /scratch/code/shibboleth/dm-worktrees/f449-stage3-vectors -b f449-stage3-vectors main
cd /scratch/code/shibboleth/dm-worktrees/f449-stage3-vectors
git rev-parse HEAD   # expect cf35d61af0f058029df841f4882945e53f86cd4b
```

- [ ] **Step 2: Add the three MANIFEST entries.** In `crates/md-codec/src/test_vectors.rs`, add `liana_taproot` immediately after the `nums_taproot` entry (it is that vector's kind-1 twin: same tree, same path, `UNSPENDABLE(liana)` in place of the H hex):
```rust
    Vector { name: "liana_taproot",      template: "tr(UNSPENDABLE(liana),multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
        keys: &[], fingerprints: &[], force_chunked: false, path: Some("48'/0'/0'/2'") },
```
Add the two keyed entries immediately before `Vector { name: "keyed_compose_preset_kofn_recovery",`. The first is that preset's kind-1 twin: same body, keys and fingerprints. The second is the F-640 nested shape Liana v15.0 accepts, over the same four keys in the same order:
```rust
    Vector { name: "keyed_tr_liana_kofn_recovery",
        template: "tr(UNSPENDABLE(liana),{multi_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*),and_v(v:pk(@3/48'/0'/3'/3'/<0;1>/*),older(26280))})",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
    Vector { name: "keyed_tr_liana_nested_two_recoveries",
        template: "tr(UNSPENDABLE(liana),{multi_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*),{and_v(v:pk(@2/48'/0'/2'/3'/<0;1>/*),older(26280)),and_v(v:pk(@3/48'/0'/3'/3'/<0;1>/*),older(52560))}})",
        keys: &[(0, XPUB_JOURNEY_0), (1, XPUB_JOURNEY_1), (2, XPUB_JOURNEY_2), (3, XPUB_JOURNEY_3)],
        fingerprints: &[(0, [0x73, 0xc5, 0xda, 0x0a]), (1, [0x73, 0xc5, 0xda, 0x0a]), (2, [0x73, 0xc5, 0xda, 0x0a]), (3, [0x73, 0xc5, 0xda, 0x0a])],
        force_chunked: true, path: None },
```
The names are chosen for the fork's gates. `keyed_*` enrols the two keyed vectors in `md/conformance_keyed_test.go`'s `keyed_*` globs. `liana_*` is what Task 5 widens the vendor selector to.

- [ ] **Step 3: Regenerate the corpus and confirm only the new vectors appear.**
```bash
export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH
cargo build --locked -p md-cli --all-features
./target/debug/md vectors --out crates/md-codec/tests/vectors
git status --short crates/md-codec/tests/vectors
```
Expected: exactly **14 untracked files** (5 + 5 + 4) and **no `M` line**. Measured: `md vectors` emits 320 files, and the 306 that already exist regenerate byte-identical at `cf35d61a`. The directory holds 308 files; the 2 it holds beyond the 306 are not `md vectors` output and are untouched. Any modified file means the tree moved; stop and find out why before continuing.

Expected contents, measured, for the reviewer to check against:
- `liana_taproot.phrase.txt` = `md1gzfdsssj5qqcreqygvtam2lm5fenw4v`, and `.bytes.hex` = `4092d84212a00181e4044300`. Compare `nums_taproot` = `2092d84212a00181c80886`: byte 0 `0x20` becomes `0x40` (version 4 to 8), and the rest shifts by the one inserted kind bit.
- Both keyed records render the same internal key: `tr(xpub661MyMwAqRbcEbgXQvrrEg24KVSoGuYnxzsnZDLn7qLYyUHkHn8JLN1dGtjhoBP1JR66FMgfKu7FHP8q7urFoREbkXkSPWASTvaJGMLhuns/0/*,…`. This is SPEC §2's structure-independence property, measured in Rust.
- `keyed_tr_liana_kofn_recovery.conformance.json`: `md1_encoding_id` `116659077988624006ef630ddf96c982`, `wallet_descriptor_template_id` `f99cc42e1ff68bae546c4d1070e5963f`, `wallet_policy_id` `cb913ed2f6ca08903f67a183aae088d5`. The phrase is 8 chunks, each starting `md13…`: data symbol `3` = `0b10001`, which is version 8 with the chunked flag set.

- [ ] **Step 4: Run the suite and see the four measured REDs.**
```bash
cargo nextest run --locked --workspace --all-features --no-fail-fast 2>&1 | tee /tmp/claude-1000/f449s3-dm-red.txt | grep -E "Summary|^\s+FAIL"
```
Expected, measured: `1536 tests run: 1532 passed, 4 failed`.
- `md-codec::wire_version_8 wire_version_is_derived_from_the_tree_not_assumed`, at `wire_version_8.rs:76`: `left: 8, right: 4` for `keyed_tr_liana_kofn_recovery`. It asserts **every** root-tr vector is version 4.
- `md-codec::skeleton_key_conformance chunks_and_descriptor_yield_the_same_key`, at `skeleton_key_conformance.rs:571`: "key @0 carries no [origin]". Its descriptor→md parser seats the origin-less internal xpub as a slot. This is the phantom-slot defect §4a fixed in `md decompose`, reproduced in a test harness.
- `md-cli::json_snapshots decode_json_snapshots` and `inspect_json_snapshots`: new snapshots for `liana_taproot`.

None of the four is a product defect. All are harness assumptions or new snapshots, so the fixes below are test-only.

- [ ] **Step 5: Fix `wire_version_8.rs`.** Replace the loop that begins after `assert!(!vectors.is_empty(), "no root tr vectors in the corpus");` in `wire_version_is_derived_from_the_tree_not_assumed` with:
```rust
    // F-449 stage 3 prerequisite: the corpus now carries wire-kind-1
    // vectors (`keyed_tr_liana_*`, `liana_taproot`), so the expected
    // version is DERIVED from the root internal key, never assumed to be 4.
    // Both populations must be non-empty, or one half proves nothing.
    let (mut v4, mut v8) = (0usize, 0usize);
    for name in &vectors {
        let d = decode_vendored(&load_vendored_phrase(name)).unwrap();
        let kind1 = matches!(
            d.tree.body,
            Body::Tr {
                internal_key: InternalKey::LianaUnspendable,
                ..
            }
        );
        let want = if kind1 { 8 } else { 4 };
        assert_eq!(d.wire_version(), want, "{name}");
        if kind1 {
            v8 += 1;
        } else {
            v4 += 1;
        }
    }
    assert!(v4 > 0, "no version-4 root tr vector in the corpus");
    assert!(v8 > 0, "no version-8 (kind-1) root tr vector in the corpus");
}
```
(`Body` and `InternalKey` are already in scope through `tests/common/liana.rs`.)

- [ ] **Step 6: Fix `skeleton_key_conformance.rs`.** Three edits:

(a) Add this function immediately above `fn is_nums_key(`:
```rust
/// True iff `pk` is an origin-less xpub equal to SPEC §2's recipe over the
/// tap tree's own leaf keys, in left-to-right leaf / key-occurrence order.
fn is_liana_unspendable_key(
    pk: &DescriptorPublicKey,
    tree: Option<&miniscript::descriptor::TapTree<DescriptorPublicKey>>,
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
    let want = md_codec::nums::liana_unspendable_xpub(&leaf_pubkeys, network);
    want == xkey
}
```
(b) In `ms_descriptor_to_node`'s `MsDescriptor::Tr(tr)` arm, between the `is_nums_key` branch and the `Slot` branch:
```rust
            } else if is_liana_unspendable_key(tr.internal_key(), tr.tap_tree()) {
                // SPEC §4a: the recogniser `md decompose` ships, by FULL
                // byte equality with the recipe over THESE leaves -- never
                // a structural match (SPEC §8.10's weakening mutation).
                InternalKey::LianaUnspendable
```
(c) In `parse_descriptor`, the two re-render calls `to_miniscript_descriptor(&d, 0)` and `to_miniscript_descriptor(&d, 1)` become `md_codec::to_miniscript::to_miniscript_descriptor_with_network(&d, 0, bitcoin::Network::Bitcoin)` and the same with `1`. The network-less entry point refuses kind 1 with `NetworkRequiredForUnspendable`, which was measured as the next failure after (b). Every record in the corpus is mainnet.

- [ ] **Step 7: Accept the two new snapshots after reading them.** The run leaves `crates/md-cli/tests/snapshots/json_snapshots__{decode,inspect}@liana_taproot.snap.new`. `cargo-insta` is not installed. Read each file: it must show `"is_nums": true`, `"unspendable_kind": "liana_unspendable"` and `"schema": "md-cli/2"`. Then accept it by dropping insta's `assertion_line:` header line:
```bash
cd crates/md-cli/tests/snapshots
for f in json_snapshots__*@liana_taproot.snap.new; do grep -v '^assertion_line:' "$f" > "${f%.new}"; rm "$f"; done
cd -
```

- [ ] **Step 8: Gate.** Run `cargo fmt --all`, then `./scripts/phase-gate.sh`. Expected: **exit 0**, nextest `1536 tests run: 1536 passed, 4 skipped` (measured). Paste the summary lines into the commit message.

- [ ] **Step 9: Mutation check.** This proves the new harness branch is load-bearing. In `is_liana_unspendable_key`, replace `want == xkey` with `true`, then run `cargo nextest run --locked -p md-codec -E 'binary(skeleton_key_conformance)'`. Expected: the run stays **green**. This is the honest result, and it is recorded here: the dm corpus has no near-miss internal key, so this branch's precision is **not** gated in Rust. The Go port's equivalent **is** gated (Task 5, `TestLianaReductionRefusesANearMiss`). Revert the mutation, then file the Rust gap as part of Task 8 (F-655, owning phase: next descriptor-mnemonic release).

- [ ] **Step 10: Commit, merge and push.**
```bash
git add crates/md-codec/src/test_vectors.rs crates/md-codec/tests/wire_version_8.rs \
  crates/md-codec/tests/skeleton_key_conformance.rs crates/md-codec/tests/vectors/liana_taproot.* \
  crates/md-codec/tests/vectors/keyed_tr_liana_* crates/md-cli/tests/snapshots/json_snapshots__*@liana_taproot.snap
git commit -F <msgfile>   # "vectors: three wire-kind-1 vectors for the Go port (F-449 stage 3)" + phase-gate lines
git -C /scratch/code/shibboleth/descriptor-mnemonic merge --no-ff f449-stage3-vectors
cd /scratch/code/shibboleth/descriptor-mnemonic && ./scripts/push-via-staging.sh
```
Record the **merge SHA** on `main`; Task 5 pins it. There is no version bump. Precedent: MANIFEST additions land on `main` untagged, for example `af3ba0b2` ("vectors: one hashlock-gated vector per hash kind"). The fork pins by commit, not by crate version. The push is the last action on this repo in the turn ("freeze means empty your hands first").

---

## Task 2 (fork): worktree and measured baseline

- [ ] **Step 1: Create the worktree.**
```bash
git -C /scratch/code/shibboleth/seedhammer worktree add /scratch/code/shibboleth/sh-worktrees/f449-stage3 -b f449-stage3 main
cd /scratch/code/shibboleth/sh-worktrees/f449-stage3 && git rev-parse --short HEAD   # 7b6f2fb
export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH
```

- [ ] **Step 2: Measure the baseline, once, to files.**
```bash
go test ./md/ -list '.*' | grep -cE '^(Test|Fuzz|Example)'          # expect 167
go test -count=1 ./md/ > /tmp/claude-1000/f449s3-md-base.txt 2>&1; tail -1 /tmp/claude-1000/f449s3-md-base.txt
/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24 > /tmp/claude-1000/f449s3-gui-base.txt 2>&1
tail -1 /tmp/claude-1000/f449s3-gui-base.txt                          # expect "all 1376 tests ran"
gofmt -l . | sort                                                      # expect the five-file set
go vet ./md/ && go vet ./gui/ 2>&1 | grep -c ArtifactDir               # md clean; gui 2
```
If any count differs from the plan, **stop and re-validate the plan against the tree**. This plan's GREEN expires when the tree moves (CLAUDE.md).

---

## Task 3 (fork): the internal key is three-state — ZERO wire change

**Why a separate task.** SPEC §3f shipped Rust as refactor-first so the wire change would sit alone in its own diff, and this task mirrors that. It also closes the one trap a bool-plus-bool design would leave open. `canonicalize.go` rebuilds `trBody` **field by field** at two sites, `remapIndices` (`:146`) and `cloneNode` (`:323`). A second bool added beside `isNums` would be silently dropped there, and a kind-1 card would re-encode as NUMS. A single `ik` field carries all three states through those copies.

**Interfaces produced:**
- `type InternalKeyKind uint8` with `InternalKeySlot` (zero value), `InternalKeyNUMS` and `InternalKeyLianaUnspendable`.
- `trBody{ik InternalKeyKind; keyIndex uint8; tree *node}`.
- `func (b trBody) isNums() bool`.
- `TapLeavesChunks(strs) (internalKeyIndex uint8, ik InternalKeyKind, leaves []TapLeaf, err error)`.
- `EmitTapLeavesChunks(strs, keys) (internalKeyIndex uint8, ik InternalKeyKind, leaves []TapLeafScript, err error)`.

- [ ] **Step 1: The type.** In `md/md.go`, replace the `trBody` struct (`:114-118` at 7b6f2fb) and add the kind type before `type keyArgBody`:
```go
type trBody struct { // Tr
	// ik is the internal key's kind (port of md-codec tree.rs InternalKey).
	// keyIndex is meaningful ONLY when ik == InternalKeySlot. The zero value
	// is InternalKeySlot, which is what the pre-refactor `isNums: false`
	// zero value meant, so a literal that omits ik is unchanged in meaning.
	ik       InternalKeyKind
	keyIndex uint8
	tree     *node
}

// isNums reports the WIRE bit is_nums: true for every internal key that is
// not a placeholder slot (NUMS or Liana-unspendable). It is the bit, not the
// kind -- a caller that needs to tell the two unspendable kinds apart must
// switch on ik, never on this.
func (b trBody) isNums() bool { return b.ik != InternalKeySlot }

// InternalKeyKind is a taproot internal key's three states -- the port of
// md-codec's `tree::InternalKey` (md-codec 0.47.0, SPEC_liana_unspendable
// _internal_key §3f). Exported because EmitTapLeavesChunks and
// TapLeavesChunks return it: a two-state bool cannot say "Liana-unspendable",
// and reading kind 1 as NUMS derives a DIFFERENT WALLET's addresses (§7a).
type InternalKeyKind uint8

const (
	// InternalKeySlot -- a real, spendable key at a placeholder slot.
	InternalKeySlot InternalKeyKind = iota
	// InternalKeyNUMS -- the BIP-341 NUMS H point, raw x-only. Wire kind 0.
	InternalKeyNUMS
	// InternalKeyLianaUnspendable -- Liana's unspendable xpub, derived from the
	// tap tree's leaf keys (SPEC §2). Wire kind 1; exists only at wire version 8.
	InternalKeyLianaUnspendable
)
```

- [ ] **Step 2: The mechanical sites (24 non-test lines, measured with `grep -rn isNums md/*.go gui/*.go | grep -v _test.go`).** Apply exactly these edits:
  - `md/md.go` `readNodeDepth` Tr arm: `b = trBody{isNums: isNums, keyIndex: keyIndex, tree: sub}` becomes
    ```go
    		ik := InternalKeySlot
    		if isNums {
    			ik = InternalKeyNUMS
    		}
    		b = trBody{ik: ik, keyIndex: keyIndex, tree: sub}
    ```
  - `md/md.go:957` and `:1282`: `b.isNums` becomes `b.isNums()`.
  - `md/canonicalize.go:111`, `:147` and `:221`: `b.isNums` becomes `b.isNums()`. At `:146` and `:323`, `nb := trBody{isNums: b.isNums, keyIndex: b.keyIndex}` becomes `nb := trBody{ik: b.ik, keyIndex: b.keyIndex}`.
  - `md/duplicate_keys.go:199` and `:342`: `!b.isNums` becomes `!b.isNums()`. `md/policy_shape.go:126`: `b.isNums` becomes `b.isNums()`. `md/encode.go:206-207`: `b.isNums` becomes `b.isNums()` (twice).
  - `md/encode_singlesig.go:90` (the comment) and `:99`: `trBody{isNums: false, …}` becomes `trBody{ik: InternalKeySlot, …}`.
  - `md/compose.go:961`: replace the `tree :=` line with
    ```go
    	// ik < 0: no path became the internal key, so it is the NUMS point.
    	// (Stage 3 ports no Liana selection into the composer; that is the port
    	// of md-codec's `--unspendable`, owned by F-449 stage 4.)
    	ikKind := InternalKeyNUMS
    	if ik >= 0 {
    		ikKind = InternalKeySlot
    	}
    	tree := node{tag: tagTr, body: trBody{ik: ikKind, keyIndex: 0, tree: spine}}
    ```
  - `md/tapleaves.go`:
    - `TapLeavesChunks` (`:58`), `tapLeaves` (`:66`) and `EmitTapLeavesChunks` (`:188`): the `isNUMS bool` result becomes `ik InternalKeyKind`.
    - Every `return 0, false, nil, …` becomes `return 0, InternalKeySlot, nil, …`.
    - `return b.keyIndex, b.isNums, out, nil` (`:81` and `:204`) becomes `return b.keyIndex, b.ik, out, nil`.
    - Add this to `EmitTapLeavesChunks`'s doc comment, after its first paragraph:
    ```go
    // THE INTERNAL KEY IS THREE-STATE (F-449, SPEC §7a.1). internalKeyIndex is
    // meaningful only when ik == InternalKeySlot. A caller must switch on ik and
    // REFUSE any kind it cannot derive -- never map a non-Slot kind to the NUMS
    // point: InternalKeyLianaUnspendable read as NUMS derives a DIFFERENT wallet's
    // addresses (SPEC §7a row 1).
    //
    ```

- [ ] **Step 3: The one production caller, `gui/policy_address.go:132-158`.** Rename `isNUMS` to `ik` in the probe line, and replace the `if isNUMS { … } else { … }` internal-key block with:
```go
			var ikey *secp256k1.PublicKey
			switch ik {
			case md.InternalKeyNUMS:
				ikey, err = address.NUMSInternalKey()
			case md.InternalKeySlot:
				internal, iok := byIndex[ikIndex]
				if !iok {
					return "", errors.New("gui: taproot internal key has no @N entry")
				}
				ikey, err = address.DeriveChild(internal, index, change)
			default:
				// SPEC §7a.3: a kind this firmware cannot derive gets NO address
				// -- never the NUMS branch, which would show a DIFFERENT
				// wallet's addresses. The probe below turns this error into
				// "no address source". Liana-unspendable derivation is stage 4.
				return "", errUnderivableInternalKey
			}
```
Add this above `// complexAddressDeriver is complexAddressSource's body`:
```go
// errUnderivableInternalKey: the policy's taproot internal key is a kind this
// firmware cannot derive (SPEC §7a.3).
var errUnderivableInternalKey = errors.New("gui: taproot internal key of a kind this firmware cannot derive")
```
The refusal reaches the operator through the existing probe (`if _, err := src(0, false); err != nil { return nil, false }`), so every screen shows the "no address" wording it already has.

- [ ] **Step 4: Test-side mechanical edits (17 lines, measured).**
  - In `md/*_test.go`: `trBody{isNums: true` becomes `trBody{ik: InternalKeyNUMS`, and `tr.isNums ||`, `if b.isNums {` and `if tb.isNums {` become `isNums()` calls.
  - `md/f533_internal_key_reuse_test.go:21-22` keeps its `isNums bool` parameter and maps it:
    ```go
    	ik := InternalKeySlot
    	if isNums {
    		ik = InternalKeyNUMS
    	}
    	return node{tag: tagTr, body: trBody{ik: ik, keyIndex: keyIndex, tree: tree}}
    ```
  - `md/testdata_test.go:301`: `tb := trBody{ik: InternalKeySlot, keyIndex: d.KeyIndex}; if d.IsNums { tb.ik = InternalKeyNUMS }`.
  - `md/compose_pkh_emit_test.go:155-160` and `:212-217`: `_, ik, leaves, err := EmitTapLeavesChunks(…)` with `if ik != InternalKeyNUMS || len(leaves) != N`.
  - `gui/taproot_script_path_test.go:45-54`: `internalIdx, ik, leaves, err := md.TapLeavesChunks(chunks)`, and the `if isNUMS { t.Skip(…) }` becomes
    ```go
    			switch ik {
    			case md.InternalKeySlot:
    			case md.InternalKeyNUMS:
    				t.Skip("NUMS internal key: no key-path spend, not this gate's subject")
    			case md.InternalKeyLianaUnspendable:
    				// F-449: the device's kind-1 derivation is stage 4 (SPEC §7a.2).
    				// Its refusal until then is TestEveryKeyedVectorReachesAnAddress's
    				// stillUnsupported entry, which FAILS the day stage 4 derives it.
    				t.Skip("Liana-unspendable internal key: device derivation is F-449 stage 4")
    			default:
    				t.Fatalf("%s: internal-key kind %d is unknown to this gate", name, ik)
    			}
    ```
  - After the edits, `grep -rn 'isNums\b[^(]' md gui --include='*.go'` must print only the `func (b trBody) isNums()` line, the `readNodeDepth` local variable, and the f533 helper's parameter.

- [ ] **Step 5: Gate. This task's tests ARE the existing suite.** It is a refactor, and its correctness claim is "no byte moved".
```bash
gofmt -l . | sort          # the five-file set, nothing more
go vet ./md/ && go build ./...
go test -count=1 ./md/     # ok; 167 tests (the byte-parity goldens TestEncodePayloadGoldens/TestEncodeMD1StringGoldens are the proof)
go test -count=1 -run 'TestEveryKeyedVectorReachesAnAddress|TestTaprootScriptPathMatchesRust|Emit|Pkh' ./gui/   # ok
```
Measured green at this boundary.

- [ ] **Step 6: Commit** `md: the tr internal key is three-state (InternalKeyKind), zero wire change (F-449 stage 3)`.

---

## Task 4 (fork): SPEC §2's recipe, checked against Liana's own golden xpubs

**Scope call, stated.** §9 puts "recompute §2" in stage 4's gui branch. But the recipe's Rust home is md-codec (`nums.rs:50`, `liana_unspendable_xpub`), and Task 5's descriptor gate needs it to reduce a kind-1 record **by byte equality**. Without it, that gate needs either an exclusion or a structural match, and the structural match is the §8.10 weakening. So the pure function is ported here, into `md/`, network-free. Stage 4 wires it to a device address.

- [ ] **Step 1: Vendor the evidence from the primary.** Create `scripts/vendor-liana-cases.sh`, mode 755:
```bash
#!/usr/bin/env bash
# Vendor the Rust primary's Liana recipe evidence (SPEC_liana_unspendable
# _internal_key §8.1) into md/testdata/liana_cases.json and write the provenance
# pin md/liana_test.go checks. Re-run on every re-pin.
#
#   scripts/vendor-liana-cases.sh [/path/to/descriptor-mnemonic]
#
# The primary copy is itself vendored from mnemonic-engrave's harness evidence by
# descriptor-mnemonic's scripts/vendor-liana-evidence.sh; this repo takes the
# PRIMARY's copy, never the evidence directly (Rust-primary rule).
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="${1:-$HERE/../descriptor-mnemonic}"
REL="crates/md-codec/tests/fixtures/liana/cases.json"
DST="$HERE/md/testdata/liana_cases.json"
PIN="$HERE/md/testdata/liana_cases.provenance.json"
[ -f "$SRC/$REL" ] || { echo "no cases at $SRC/$REL" >&2; exit 2; }
commit=$(git -C "$SRC" rev-parse HEAD)
clean=true; [ -z "$(git -C "$SRC" status --porcelain -- "$REL")" ] || clean=false
cp "$SRC/$REL" "$DST"
python3 - "$PIN" "$commit" "$clean" "$DST" "$REL" <<'PY'
import hashlib, json, sys, datetime
pin, commit, clean, dst, rel = sys.argv[1], sys.argv[2], sys.argv[3] == "true", sys.argv[4], sys.argv[5]
doc = {
  "_comment": [
    "PROVENANCE PIN for md/testdata/liana_cases.json (F-449 stage 3, SPEC §8.1 Go leg).",
    "Generated by scripts/vendor-liana-cases.sh; never edited by hand.",
    "md/liana_test.go fails if the file's sha256 disagrees or the case count is not the one it asserts.",
  ],
  "repo": "descriptor-mnemonic",
  "remote": "git@github.com:bg002h/descriptor-mnemonic.git",
  "commit": commit,
  "repo_clean_when_recorded": clean,
  "path": rel,
  "sha256": hashlib.sha256(open(dst, "rb").read()).hexdigest(),
  "recorded_at": datetime.date.today().isoformat(),
}
json.dump(doc, open(pin, "w"), indent=2); open(pin, "a").write("\n")
print("vendored %s, primary %s" % (rel, commit[:12]))
PY
```
Run it with the **explicit** path: `./scripts/vendor-liana-cases.sh /scratch/code/shibboleth/descriptor-mnemonic`. Check first that `git -C /scratch/code/shibboleth/descriptor-mnemonic rev-parse HEAD` is Task 1's merge SHA. The file is unchanged since `cf35d61a`; Task 1 does not touch it.

- [ ] **Step 2: Write the failing tests.** Create `md/liana_test.go`:
```go
package md

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"os"
	"path/filepath"
	"sort"
	"testing"
)

// ─── SPEC §8.1 Go leg: the recipe against LIANA's own golden xpubs ───────────

type lianaCase struct {
	Name           string   `json:"name"`
	Accepted       bool     `json:"accepted"`
	LeafPubkeysHex []string `json:"leaf_pubkeys_hex"`
	ExpectedXpub   string   `json:"expected_xpub"`
}

// lianaCases loads the vendored evidence AND checks it against its provenance
// pin, so a hand-edited or stale copy is a failure rather than a silent pass.
func lianaCases(t *testing.T) []lianaCase {
	t.Helper()
	raw, err := os.ReadFile(filepath.Join("testdata", "liana_cases.json"))
	if err != nil {
		t.Fatalf("INCONCLUSIVE: %v -- run scripts/vendor-liana-cases.sh", err)
	}
	pinRaw, err := os.ReadFile(filepath.Join("testdata", "liana_cases.provenance.json"))
	if err != nil {
		t.Fatalf("INCONCLUSIVE: no provenance pin: %v", err)
	}
	var pin struct {
		Commit string `json:"commit"`
		SHA256 string `json:"sha256"`
	}
	if err := json.Unmarshal(pinRaw, &pin); err != nil {
		t.Fatal(err)
	}
	if sum := sha256.Sum256(raw); hex.EncodeToString(sum[:]) != pin.SHA256 || pin.Commit == "" {
		t.Fatalf("liana_cases.json disagrees with its pin (or the pin names no commit)")
	}
	var cs []lianaCase
	if err := json.Unmarshal(raw, &cs); err != nil {
		t.Fatal(err)
	}
	// Nine at descriptor-mnemonic cf35d61a: the eight fable-r0 evidence shapes
	// plus stage 2's nested ACCEPT. A count, not ">0": a truncated copy that
	// kept one case would otherwise pass.
	if len(cs) != 9 {
		t.Fatalf("liana_cases.json carries %d cases, want 9", len(cs))
	}
	return cs
}

func caseLeaves(t *testing.T, c lianaCase) [][33]byte {
	t.Helper()
	out := make([][33]byte, 0, len(c.LeafPubkeysHex))
	for _, h := range c.LeafPubkeysHex {
		b, err := hex.DecodeString(h)
		if err != nil || len(b) != 33 {
			t.Fatalf("%s: leaf pubkey %q: %v (len %d)", c.Name, h, err, len(b))
		}
		var pk [33]byte
		copy(pk[:], b)
		out = append(out, pk)
	}
	return out
}

func TestLianaRecipeReproducesEveryGoldenXpub(t *testing.T) {
	for _, c := range lianaCases(t) {
		want, err := parseExtendedKey(c.ExpectedXpub)
		if err != nil {
			t.Fatalf("%s: expected_xpub: %v", c.Name, err)
		}
		if want.depth != 0 || want.parentFP != 0 || want.child != 0 {
			t.Fatalf("%s: golden xpub is not depth 0 / parent 0 / child 0", c.Name)
		}
		if got := lianaUnspendableKey(caseLeaves(t, c)); got != want.material {
			t.Errorf("%s: recipe\n  go:     %x\n  liana:  %x", c.Name, got, want.material)
		}
	}
}

// Sorting or deduplicating is the PR-1746 recipe, a DIFFERENT xpub (SPEC §2).
// Asserted on real cases so the property is not vacuous: each case below has
// leaves that are not already in sorted order.
func TestLianaRecipeIsOrderSensitiveAndKeepsDuplicates(t *testing.T) {
	sortedDiffers := 0
	for _, c := range lianaCases(t) {
		leaves := caseLeaves(t, c)
		sorted := append([][33]byte(nil), leaves...)
		sort.Slice(sorted, func(i, j int) bool { return bytes.Compare(sorted[i][:], sorted[j][:]) < 0 })
		if !slicesEqual33(sorted, leaves) {
			if lianaUnspendableKey(sorted) == lianaUnspendableKey(leaves) {
				t.Errorf("%s: sorting the leaves did not change the key", c.Name)
			}
			sortedDiffers++
		}
	}
	if sortedDiffers == 0 {
		t.Fatal("every case is already sorted -- the order property was never exercised")
	}
	// Dedup: a synthetic duplicate. Two occurrences of one key hash differently
	// from one occurrence.
	one := caseLeaves(t, lianaCases(t)[0])[:1]
	two := append(append([][33]byte(nil), one...), one...)
	if lianaUnspendableKey(one) == lianaUnspendableKey(two) {
		t.Error("a duplicated leaf key hashed the same as a single one -- the recipe deduplicates")
	}
}

func slicesEqual33(a, b [][33]byte) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}
```
(`parseExtendedKey` already exists in `md/conformance_keyed_test.go`, in the same package.)

- [ ] **Step 3: Run it; expected FAIL** with `undefined: lianaUnspendableKey`.

- [ ] **Step 4: Implement.** Create `md/liana.go`:
```go
package md

import "crypto/sha256"

// Liana's unspendable internal key (F-449, SPEC_liana_unspendable_internal_key
// §2). Port of md-codec nums.rs liana_unspendable_xpub + to_miniscript.rs
// collect_leaf_pubkeys, md-codec 0.47.0.
//
// NETWORK-FREE BY DESIGN. md carries key material as 65 bytes (chain code ‖
// compressed pubkey, the Pubkeys TLV layout), never as base58, so this returns
// that material. The xpub/tpub version bytes are a RENDER-time choice (§2 step
// 5) and belong to whoever renders.

// numsHCompressed is BIP-341's H point, compressed (0x02 ‖ x). Same x as the
// raw NUMS internal key md renders for wire kind 0.
var numsHCompressed = [33]byte{
	0x02,
	0x50, 0x92, 0x9b, 0x74, 0xc1, 0xa0, 0x49, 0x54, 0xb7, 0x8b, 0x4b, 0x60, 0x35, 0xe9, 0x7a, 0x5e,
	0x07, 0x8a, 0x5a, 0x0f, 0x28, 0xec, 0x96, 0xd5, 0x47, 0xbf, 0xee, 0x9a, 0xce, 0x80, 0x3a, 0xc0,
}

// lianaUnspendableKey is §2's recipe: chain code = sha256 of the leaf keys'
// 33-byte compressed pubkeys concatenated IN THE ORDER GIVEN -- not sorted, not
// deduplicated -- and the public key is always H. Depth 0, parent fingerprint 0
// and child number 0 are implied by the caller's rendering.
func lianaUnspendableKey(leafPubkeys [][33]byte) [65]byte {
	h := sha256.New()
	for _, pk := range leafPubkeys {
		h.Write(pk[:])
	}
	var out [65]byte
	copy(out[:32], h.Sum(nil))
	copy(out[32:], numsHCompressed[:])
	return out
}
```

- [ ] **Step 5: Run; expected PASS**, with md at **169** tests (measured). All nine cases reproduce Liana's golden xpub byte for byte (measured). That includes the four Liana refused on policy shape and the nested ACCEPT. Commit `md: port SPEC §2's Liana unspendable-key recipe, checked against Liana's golden xpubs (F-449 stage 3)`, including `scripts/vendor-liana-cases.sh`, `md/testdata/liana_cases.json` and its `.provenance.json`.

---

## Task 5 (fork): wire version 8 — the kind bit, the version threading, and the Rust vectors that bind it

**Interfaces produced:**
- `ErrUnsupportedWireVersion`, `type WireVersionError struct{ Got uint8 }` (with `Is`), `isSupportedVersion(uint8) bool`, `wfUnspendableVersion = 8`.
- `readNode(r, kiw, wireVersion)`, `writeNode(w, n, kiw, wireVersion)`, `(*descriptor).wireVersion() uint8`, `errLianaNeedsVersion8`.
- `lianaLeafPubkeys(*descriptor) ([][33]byte, error)`.
- Test-side: `reduceLianaInternalKey(chain, body string, d *descriptor) (string, error)` and `decodeSingleForTest([]string) (*descriptor, error)`.

- [ ] **Step 1: Vendor the kind-1 vectors, and widen the selector.**
  - In `scripts/vendor-compose-vectors.sh`, change the selector `grep -E '^(keyed_|compose_)'` to `grep -E '^(keyed_|compose_|liana_)'`. Add one comment line above it: `# liana_* (F-449 stage 3): the keyless kind-1 twin of nums_taproot.`
  - Check `git -C /scratch/code/shibboleth/descriptor-mnemonic rev-parse HEAD` equals Task 1's merge SHA, and that `git -C … status --porcelain -- crates/md-codec/tests/vectors` is empty.
  - Run `./scripts/vendor-compose-vectors.sh /scratch/code/shibboleth/descriptor-mnemonic`. Expected, measured: `vendored 260 files, 53 vectors`. `git status --short` shows the 14 new files and `md/testdata/compose_vectors.provenance.json` modified, and **nothing else modified**, because the other 246 files are byte-identical.
  - In `md/compose_vectors_pin_test.go`:
    - Append to `composeVectorNames`:
      ```go
      	// WIRE KIND 1, 50 -> 53 (F-449 stage 3). The primary's first vectors whose
      	// taproot internal key is Liana's unspendable xpub: two keyed (conformance
      	// records, so the keyed gates enrol them) and one keyless single-string
      	// twin of nums_taproot (the only version-8 SINGLE-string card).
      	"keyed_tr_liana_kofn_recovery",
      	"keyed_tr_liana_nested_two_recoveries",
      	"liana_taproot",
      ```
    - Change the file-count literal and its comment: `246` becomes `260` (48 keyed × 5 + 5 keyless × 4).
    - Widen `isComposeVectorFile` to also accept the `liana_` prefix:
      ```go
      	if !strings.HasPrefix(name, "compose_") && !strings.HasPrefix(name, "keyed_") &&
      		!strings.HasPrefix(name, "liana_") {
      ```
      Without this, `liana_taproot`'s files sit in the directory with **no** directory-scan coverage. This was measured: the pin test did not flag them.
  - In `md/testdata_test.go`, append `"liana_taproot"` to `singleStringVectorNames`, with a comment naming it `nums_taproot`'s kind-1 twin. In the loader's `case "Tr":` add the field and the mapping:
    ```go
    			// md-cli/2 (md-cli 0.18.0+): "liana_unspendable" or ABSENT.
    			UnspendableKind *string `json:"unspendable_kind"`
    ```
    ```go
    		if d.UnspendableKind != nil {
    			// An unknown value is a vector this loader cannot represent; guessing
    			// NUMS would build a DIFFERENT wallet and the parity test would blame
    			// the encoder.
    			if *d.UnspendableKind != "liana_unspendable" || !d.IsNums {
    				t.Fatalf("Tr: unspendable_kind %q with is_nums=%v is not representable", *d.UnspendableKind, d.IsNums)
    			}
    			tb.ik = InternalKeyLianaUnspendable
    		}
    ```

- [ ] **Step 2: Run `go test ./md/`. Expected FAIL. This IS §8.4's "v8 refused by a v4 decoder" leg, measured on the old decoder before it is replaced.** Measured at this exact state (Tasks 3 and 4 plus Step 1), five tests fail:
  - `TestKeyedConformanceAgreesWithRust/keyed_tr_liana_*`: `DecodeChunks: md: wire version mismatch`, twice.
  - `TestKeyedConformanceDescriptorsAgreeWithTheirTemplates/keyed_tr_liana_*`: `ExpandWalletPolicyChunks: md: wire version mismatch`, twice.
  - `TestCanonicalizeIdempotentOnGoldens/liana_taproot`: `decodePayloadValidated("md1gzfdsssj5qqcreqygvtam2lm5fenw4v"): md: wire version mismatch`.
  - `TestEncodePayloadGoldens/liana_taproot` and `TestEncodeMD1StringGoldens/liana_taproot`: parity.

  Any other failure means the vendoring went wrong. Paste the failing lines into the task's commit message. They are the only record of what the pre-stage-3 firmware says about these plates.

- [ ] **Step 3: Write the failing tests.** Create `md/wire_version8_test.go`:
```go
package md

import (
	"bytes"
	"encoding/hex"
	"errors"
	"slices"
	"testing"

	"seedhammer.com/codex32"
)

// ─── SPEC §8.4 Go leg: the round trip through the DISPATCH ───────────────────
//
// Not through decodePayload alone: version 5 was unusable only because of the
// bit-0 dispatch (SPEC §3c), which decodePayload never sees. So every leg below
// enters where the device does -- ParseChunkHeader, Decode, Reassemble.

func TestVersion8SingleStringThroughTheDispatch(t *testing.T) {
	s := loadPhraseChunks(t, "liana_taproot")
	if len(s) != 1 {
		t.Fatalf("liana_taproot is %d strings, want the one single-string v8 card", len(s))
	}
	h, err := ParseChunkHeader(s[0])
	if err != nil || h.Chunked {
		t.Fatalf("ParseChunkHeader = %+v, %v; want a single (non-chunked) md1", h, err)
	}
	if _, err := Decode(s[0]); err != nil {
		t.Fatalf("Decode: %v", err)
	}
	d, err := decodeSingleForTest(s)
	if err != nil {
		t.Fatal(err)
	}
	if b := d.tree.body.(trBody); b.ik != InternalKeyLianaUnspendable {
		t.Fatalf("decoded internal key %d, want InternalKeyLianaUnspendable", b.ik)
	}
	// Re-encode: the SAME string Rust minted (byte parity via the string).
	if got, err := encodeMD1String(d); err != nil || got != s[0] {
		t.Fatalf("encodeMD1String = %q, %v; want %q", got, err, s[0])
	}
	// Its kind-0 twin still decodes as NUMS at version 4, unchanged.
	twin, err := decodeSingleForTest(loadPhraseChunks(t, "nums_taproot"))
	if err != nil {
		t.Fatal(err)
	}
	if b := twin.tree.body.(trBody); b.ik != InternalKeyNUMS || twin.wireVersion() != wfRedesignVersion {
		t.Fatalf("nums_taproot: ik %d version %d, want NUMS at 4", b.ik, twin.wireVersion())
	}
}

func TestVersion8ChunkSetsThroughTheDispatch(t *testing.T) {
	for _, name := range []string{"keyed_tr_liana_kofn_recovery", "keyed_tr_liana_nested_two_recoveries"} {
		chunks := loadPhraseChunks(t, name)
		for i, c := range chunks {
			h, err := ParseChunkHeader(c)
			if err != nil || !h.Chunked || h.Version != wfUnspendableVersion {
				t.Fatalf("%s chunk %d: %+v, %v; want a chunked header at version 8", name, i, h, err)
			}
		}
		d, err := Reassemble(chunks)
		if err != nil {
			t.Fatalf("%s: Reassemble: %v", name, err)
		}
		if b := d.tree.body.(trBody); b.ik != InternalKeyLianaUnspendable {
			t.Fatalf("%s: internal key %d, want InternalKeyLianaUnspendable", name, b.ik)
		}
		// The chunk WRITER carries the derived version in every chunk header
		// (chunk.rs:280), so re-splitting reproduces Rust's chunk set exactly.
		again, err := split(d)
		if err != nil {
			t.Fatalf("%s: split: %v", name, err)
		}
		if !slices.Equal(again, chunks) {
			t.Fatalf("%s: split(Reassemble(x)) != x\n  go:   %v\n  rust: %v", name, again, chunks)
		}
	}
}

// withVersion rewrites an md1 string's 4-bit wire version and re-checksums it,
// producing a BCH-valid card at a version no encoder emits.
func withVersion(t *testing.T, s string, v uint8) string {
	t.Helper()
	syms, err := codex32.MDDataSymbols(s)
	if err != nil {
		t.Fatal(err)
	}
	syms = append([]byte(nil), syms...)
	if syms[0]&1 == 1 { // chunked: symbol 0 is [v3 v2 v1 v0 chunked=1]
		syms[0] = v<<1 | 1
	} else { // single: symbol 0 is [divergent v3 v2 v1 v0]
		syms[0] = syms[0]&0b10000 | v&0b1111
	}
	return codex32.AssembleMD1(syms)
}

// An UNSUPPORTED version is refused with an error that NAMES it (SPEC §6a),
// single-string and chunked. 12 is the one remaining even version (§3a).
func TestAnUnsupportedWireVersionIsRefusedByName(t *testing.T) {
	single := withVersion(t, loadPhraseChunks(t, "liana_taproot")[0], 12)
	chunk := withVersion(t, loadPhraseChunks(t, "keyed_tr_liana_kofn_recovery")[0], 12)

	_, errDecode := Decode(single)
	_, errHeader := ParseChunkHeader(chunk)
	_, errReassemble := Reassemble([]string{chunk})
	for what, err := range map[string]error{"Decode(single)": errDecode,
		"ParseChunkHeader(chunk)": errHeader, "Reassemble(chunk)": errReassemble} {
		var wv *WireVersionError
		if !errors.As(err, &wv) || wv.Got != 12 || !errors.Is(err, ErrUnsupportedWireVersion) {
			t.Errorf("%s = %v; want a *WireVersionError{Got: 12} matching ErrUnsupportedWireVersion", what, err)
		}
	}
	// Both accepted versions still parse: the refusal is the set, not "!= 4".
	for _, v := range []uint8{wfRedesignVersion, wfUnspendableVersion} {
		if _, err := ParseChunkHeader(withVersion(t, loadPhraseChunks(t, "keyed_tr_liana_kofn_recovery")[0], v)); err != nil {
			t.Errorf("version %d refused: %v", v, err)
		}
	}
}

// ─── SPEC §8.5 Go leg: identity DISTINCTNESS between the kind-0/kind-1 twins ─
//
// keyed_compose_preset_kofn_recovery and keyed_tr_liana_kofn_recovery carry the
// SAME template body, keys and fingerprints; only the internal key's kind
// differs. Equality of each with Rust is TestKeyedConformanceAgreesWithRust;
// this asserts the twins never collide -- in the full ids AND in both 4-byte
// mk1 stub flavours, which distinct ids do not imply (SPEC §3e).
func TestKind0AndKind1TwinsNeverShareAnIdentity(t *testing.T) {
	k0, err := Reassemble(vectorChunksFor(t, "keyed_compose_preset_kofn_recovery"))
	if err != nil {
		t.Fatal(err)
	}
	k1, err := Reassemble(vectorChunksFor(t, "keyed_tr_liana_kofn_recovery"))
	if err != nil {
		t.Fatal(err)
	}
	type idFn struct {
		name string
		f    func(*descriptor) ([]byte, error)
	}
	for _, fn := range []idFn{
		{"WalletPolicyId", func(d *descriptor) ([]byte, error) { x, e := WalletPolicyId(d); return x[:], e }},
		{"WalletDescriptorTemplateId", func(d *descriptor) ([]byte, error) { x, e := WalletDescriptorTemplateId(d); return x[:], e }},
		{"WalletPolicyIDStub", func(d *descriptor) ([]byte, error) { x, e := WalletPolicyIDStub(d); return x[:], e }},
		{"WalletDescriptorTemplateIdStub", func(d *descriptor) ([]byte, error) { x, e := WalletDescriptorTemplateIdStub(d); return x[:], e }},
		{"FormAwareStub", func(d *descriptor) ([]byte, error) { x, e := FormAwareStub(d); return x[:], e }},
		{"md1 encoding id", func(d *descriptor) ([]byte, error) { x, e := computeEncodingID(d); return x[:], e }},
	} {
		a, errA := fn.f(k0)
		b, errB := fn.f(k1)
		if errA != nil || errB != nil {
			t.Fatalf("%s: %v / %v", fn.name, errA, errB)
		}
		if hex.EncodeToString(a) == hex.EncodeToString(b) {
			t.Errorf("%s: kind 0 and kind 1 share %x -- two wallets with different addresses, one identity", fn.name, a)
		}
	}
}

// canonicalize has TWO trBody copy sites (remapIndices and cloneNode), and a
// canonical card only ever reaches cloneNode -- remapIndices is skipped on the
// identity permutation. So this feeds it a NON-canonical kind-1 tree: the same
// multi_a with its placeholders written out of first-occurrence order. A copy
// site that rebuilt the kind from the is_nums bit would re-encode it as NUMS at
// version 4; the kind must survive, so the bytes equal the canonical card's.
func TestCanonicalizeKeepsTheKindOnANonCanonicalTree(t *testing.T) {
	d, err := decodeSingleForTest(loadPhraseChunks(t, "liana_taproot"))
	if err != nil {
		t.Fatal(err)
	}
	want, _, err := encodePayload(d)
	if err != nil {
		t.Fatal(err)
	}
	tb := d.tree.body.(trBody)
	leaf := tb.tree.body.(multiKeysBody)
	if !slices.Equal(leaf.indices, []uint8{0, 1, 2}) {
		t.Fatalf("liana_taproot's leaf is %v, want the canonical [0 1 2] this test permutes", leaf.indices)
	}
	permuted := node{tag: tb.tree.tag, body: multiKeysBody{k: leaf.k, indices: []uint8{1, 2, 0}}}
	nc := *d
	nc.tree = node{tag: tagTr, body: trBody{ik: tb.ik, tree: &permuted}}
	got, _, err := encodePayload(&nc)
	if err != nil {
		t.Fatal(err)
	}
	if !slices.Equal(got, want) {
		t.Fatalf("non-canonical kind-1 tree encodes to %x, want the canonical %x", got, want)
	}
}

// ─── SPEC §3d: the kind bit, pinned on the WIRE ──────────────────────────────

// A symmetric polarity inversion (write AND read both flipped) round-trips
// perfectly, so no encode-then-decode test can see it (SPEC §8.10). These are
// golden bytes, identical to Rust's
// wire_version_8.rs::the_kind_bit_polarity_is_pinned_on_the_wire_not_just_round_tripped:
// Tag::Tr (000001) | is_nums 1 | kind | has_tree 0, MSB-first, zero-padded.
func TestKindBitPolarityIsPinnedOnTheWire(t *testing.T) {
	for _, c := range []struct {
		name    string
		ik      InternalKeyKind
		version uint8
		want    []byte
		bits    int
	}{
		{"liana at v8", InternalKeyLianaUnspendable, wfUnspendableVersion, []byte{0x07, 0x00}, 9},
		{"nums at v8", InternalKeyNUMS, wfUnspendableVersion, []byte{0x06, 0x00}, 9},
		{"nums at v4 (no kind bit)", InternalKeyNUMS, wfRedesignVersion, []byte{0x06}, 8},
	} {
		var w bitWriter
		if err := writeNode(&w, node{tag: tagTr, body: trBody{ik: c.ik}}, 0, c.version); err != nil {
			t.Fatalf("%s: %v", c.name, err)
		}
		if w.bitLen() != c.bits || !bytes.Equal(w.intoBytes(), c.want) {
			t.Errorf("%s: %d bits %x, want %d bits %x", c.name, w.bitLen(), w.intoBytes(), c.bits, c.want)
		}
	}
}

func TestLianaHasNoRepresentationBelowVersion8(t *testing.T) {
	var w bitWriter
	err := writeNode(&w, node{tag: tagTr, body: trBody{ik: InternalKeyLianaUnspendable}}, 0, wfRedesignVersion)
	if err != errLianaNeedsVersion8 {
		t.Fatalf("writeNode(Liana, v4) = %v, want errLianaNeedsVersion8", err)
	}
}

// ─── SPEC §3d/§3e: the version is DERIVED from the tree ──────────────────────

func TestWireVersionIsDerivedFromTheTree(t *testing.T) {
	v4, v8 := 0, 0
	for _, name := range composeVectorNames {
		chunks := vectorChunksFor(t, name)
		d, err := Reassemble(chunks)
		if err != nil {
			d, err = decodeSingleForTest(chunks)
			if err != nil {
				t.Fatalf("%s: %v", name, err)
			}
		}
		b, isTr := d.tree.body.(trBody)
		if d.tree.tag != tagTr || !isTr {
			continue
		}
		want := uint8(wfRedesignVersion)
		if b.ik == InternalKeyLianaUnspendable {
			want = wfUnspendableVersion
			v8++
		} else {
			v4++
		}
		if got := d.wireVersion(); got != want {
			t.Errorf("%s: wireVersion %d, want %d", name, got, want)
		}
	}
	if v4 == 0 || v8 == 0 {
		t.Fatalf("root-tr populations: %d at v4, %d at v8 -- one half proves nothing", v4, v8)
	}
}

// decodeSingleForTest decodes a single-string md1 to the internal descriptor
// (Decode returns only the summary Template).
func decodeSingleForTest(chunks []string) (*descriptor, error) {
	if len(chunks) != 1 {
		return nil, errChunkSetEmpty
	}
	b, bits, err := unwrapString(chunks[0])
	if err != nil {
		return nil, err
	}
	return decodePayloadValidated(b, bits)
}
```
Create `md/liana_walk_test.go`:
```go
package md

import (
	"encoding/json"
	"slices"
	"strings"
	"testing"

	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"github.com/btcsuite/btcd/chaincfg/v2"
)

// ─── SPEC §8.6 Go leg: structure independence over the vendored kind-1 cards ─

// kofn_recovery ({multi_a(2,A,B,C), pk(D)&older}) and nested_two_recoveries
// ({multi_a(2,A,B), {pk(C)&older, pk(D)&older}}) hold the SAME four keys in the
// SAME order, so they MUST derive one internal key -- and Rust rendered the
// same xpub for both (measured: xpub661MyMwAqRbcEbgXQvrr…). A test pins it so
// nobody "fixes" it into a tree-dependent hash.
func TestLianaKeyDependsOnTheLeafKeysNotTheTree(t *testing.T) {
	var keys [][65]byte
	for _, name := range []string{"keyed_tr_liana_kofn_recovery", "keyed_tr_liana_nested_two_recoveries"} {
		d, err := Reassemble(vectorChunksFor(t, name))
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		leaves, err := lianaLeafPubkeys(d)
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		if len(leaves) != 4 {
			t.Fatalf("%s: %d leaf-key occurrences, want 4", name, len(leaves))
		}
		keys = append(keys, lianaUnspendableKey(leaves))
	}
	if keys[0] != keys[1] {
		t.Errorf("the two trees derive different internal keys:\n  %x\n  %x", keys[0], keys[1])
	}
}

// ─── SPEC §8.10: the near-miss the positive corpus cannot supply ─────────────

// A valid recipe output over a DIFFERENT leaf set (here: the same four keys
// reversed) is a perfectly formed Liana key -- H pubkey, depth 0, mainnet --
// for a different wallet. reduceLianaInternalKey must refuse it; a check
// weakened to "the pubkey is H" accepts it and relabels a stranger's key as
// this wallet's own.
func TestLianaReductionRefusesANearMiss(t *testing.T) {
	name := "keyed_tr_liana_kofn_recovery"
	var rec keyedConformanceRecord
	if err := json.Unmarshal(vectorRecordFor(t, name), &rec); err != nil {
		t.Fatal(err)
	}
	d, err := Reassemble(vectorChunksFor(t, name))
	if err != nil {
		t.Fatal(err)
	}
	body, _, _ := strings.Cut(rec.Chains["0"].Descriptor, "#")
	if _, err := reduceLianaInternalKey("0", body, d); err != nil {
		t.Fatalf("control: the genuine record is refused: %v", err)
	}
	leaves, err := lianaLeafPubkeys(d)
	if err != nil {
		t.Fatal(err)
	}
	slices.Reverse(leaves)
	near := lianaUnspendableKey(leaves)
	nearKey := hdkeychain.NewExtendedKey(chaincfg.MainNetParams.HDPublicKeyID[:],
		near[32:], near[:32], []byte{0, 0, 0, 0}, 0, 0, false)
	rest := strings.TrimPrefix(body, "tr(")
	_, after, _ := strings.Cut(rest, "/")
	forged := "tr(" + nearKey.String() + "/" + after
	if _, err := reduceLianaInternalKey("0", forged, d); err == nil {
		t.Fatal("a recipe output over a DIFFERENT leaf order was accepted as this wallet's internal key")
	}
}
```
In `md/md_test.go:167`, `wantErr: errWireVersion,` becomes `wantErr: ErrUnsupportedWireVersion,`. In `md/encode_test.go:142`, `:154` and `:166`, the calls `writeNode(&w, n, K)` gain a fourth argument `wfRedesignVersion`.

- [ ] **Step 4: Teach the conformance gates kind 1 and the encoding id.** In `md/conformance_keyed_test.go`:

(a) In `TestKeyedConformanceAgreesWithRust`, immediately before `if rec.WalletPolicyID == rec.WalletDescriptorTemplateID {`, add the md1 encoding-id assertion. The field has been parsed since R3 and never asserted. **Measured 48 of 48 records agreeing** (46 existing + 2 kind-1) before this assertion was written, so it adds coverage without turning anything red:
```go
			// md1_encoding_id: the ONE id that hashes the header, so the only
			// one that sees the wire VERSION (identity.rs:45). Parsed since R3
			// and never asserted; measured 48 of 48 records agreeing before
			// this assertion was added (F-449 stage 3).
			eid, err := computeEncodingID(d)
			if err != nil {
				t.Fatalf("%s: computeEncodingID: %v", name, err)
			}
			if got := hex.EncodeToString(eid[:]); got != rec.Md1EncodingID {
				t.Errorf("%s: md1_encoding_id\n  go:   %s\n  rust: %s", name, got, rec.Md1EncodingID)
			}
```
(b) In `assertDescriptorsAgree`, immediately before `matches := descriptorKeyRe.FindAllStringSubmatchIndex(body, -1)`:
```go
		// F-449 stage 3: a wire-kind-1 card renders its internal key as an
		// ORIGIN-LESS xpub, which descriptorKeyRe (bracket-anchored) never
		// matches, so D1' would compare a raw xpub against the template's
		// marker. Reduce it to the marker ONLY on full byte equality with the
		// Go port's own recipe over the same card.
		if strings.HasPrefix(rec.Template, lianaTemplatePrefix) {
			d, err := Reassemble(chunks)
			if err != nil {
				t.Fatalf("%s: Reassemble: %v", name, err)
			}
			reducedBody, err := reduceLianaInternalKey(chain, body, d)
			if err != nil {
				t.Errorf("chain %s: %v", chain, err)
				continue
			}
			body = reducedBody
		}
```
(c) Above `// parsedExtendedKey is the part`:
```go
// lianaTemplatePrefix is how a wire-kind-1 template opens (md-codec
// nums.rs LIANA_UNSPENDABLE_MARKER inside tr()).
const lianaTemplatePrefix = "tr(UNSPENDABLE(liana),"

// reduceLianaInternalKey rewrites a rendered kind-1 descriptor's leading
// `tr(<xpub>/<chain>/*,` to `tr(UNSPENDABLE(liana),`, and REFUSES unless every
// clause holds:
//   - the key derives at <chain>/* -- SPEC §2 step 6, 0/i and 1/i in every
//     port, whatever the wallet's use-site;
//   - it is a MAINNET xpub at depth 0, parent 0, child 0 (§2 steps 4-5; every
//     vendored record is mainnet);
//   - its 65 bytes EQUAL lianaUnspendableKey over d's own leaf keys.
//
// The last clause is FULL byte equality, never "the pubkey is H": a structural
// match accepts a valid recipe output computed over a DIFFERENT leaf set, and
// a positive-only corpus cannot tell the two apart (SPEC §8.10) --
// TestLianaReductionRefusesANearMiss is the input that separates them.
func reduceLianaInternalKey(chain, body string, d *descriptor) (string, error) {
	rest, ok := strings.CutPrefix(body, "tr(")
	if !ok {
		return "", fmt.Errorf("a kind-1 record's descriptor does not open with tr(: %.24s…", body)
	}
	xkey, after, ok := strings.Cut(rest, "/")
	if !ok {
		return "", fmt.Errorf("no derivation suffix after the internal key")
	}
	suffix := chain + "/*,"
	if !strings.HasPrefix(after, suffix) {
		return "", fmt.Errorf("the internal key derives at /%.8s…, want /%s", after, suffix)
	}
	if !strings.HasPrefix(xkey, "xpub") {
		return "", fmt.Errorf("the internal key %.12s… is not a mainnet xpub", xkey)
	}
	parsed, err := parseExtendedKey(xkey)
	if err != nil {
		return "", err
	}
	if parsed.depth != 0 || parsed.parentFP != 0 || parsed.child != 0 {
		return "", fmt.Errorf("the internal key is depth %d parent %08x child %d, want 0/0/0",
			parsed.depth, parsed.parentFP, parsed.child)
	}
	leaves, err := lianaLeafPubkeys(d)
	if err != nil {
		return "", err
	}
	if want := lianaUnspendableKey(leaves); parsed.material != want {
		return "", fmt.Errorf("the internal key is not the recipe over this card's leaves\n"+
			"  record: %x\n  go:     %x", parsed.material, want)
	}
	return lianaTemplatePrefix + strings.TrimPrefix(after, suffix), nil
}
```

- [ ] **Step 5: The device's refusal of kind 1 becomes a named expectation.** In `gui/policy_address_test.go`, `stillUnsupported := map[string]string{}` becomes:
```go
	stillUnsupported := map[string]string{
		// F-449 stage 3 ships the codec half (md decodes kind 1 and reports it
		// three-state); the device's kind-1 DERIVATION is stage 4 (SPEC §7a.2).
		// Until then SPEC §7a.3 requires NO address rather than the NUMS
		// branch's addresses of a different wallet -- and the F-613 mirror
		// below proves no deriver produces one. Stage 4 deletes both entries:
		// this test FAILS, naming them, the day the device derives kind 1.
		"keyed_tr_liana_kofn_recovery":         "Liana-unspendable internal key: device derivation is F-449 stage 4",
		"keyed_tr_liana_nested_two_recoveries": "Liana-unspendable internal key: device derivation is F-449 stage 4",
	}
```

- [ ] **Step 6: Run; expected FAIL** (`undefined: wfUnspendableVersion`, `ErrUnsupportedWireVersion`, …).

- [ ] **Step 7: Implement the wire change.**

`md/md.go`:
- Add `"fmt"` to the imports. `fmt` is already linked into package `md` by `compose.go`, so this adds no TinyGo dependency.
- Delete `errWireVersion` from the var block at `:22`, and add above that block:
```go
// ErrUnsupportedWireVersion matches (via errors.Is) every refusal of a
// well-formed md1 whose 4-bit wire version is outside the set this build
// reads. The concrete error is a *WireVersionError naming the version, so a
// caller can say WHICH version rather than "not an md1" (SPEC §6a).
var ErrUnsupportedWireVersion = errors.New("md: unsupported wire version")

// WireVersionError is the concrete unsupported-version refusal. Got is the
// version the header DECLARES -- an observation, not a claim that the card is
// intact or that any tool ever wrote it (F-643: a BCH mis-correction beyond
// capacity can land on any header value).
type WireVersionError struct{ Got uint8 }

func (e *WireVersionError) Error() string {
	return fmt.Sprintf("md: wire version %d is not one this build reads (accepted: %d, %d)",
		e.Got, wfRedesignVersion, wfUnspendableVersion)
}

// Is makes errors.Is(err, ErrUnsupportedWireVersion) true.
func (e *WireVersionError) Is(target error) bool { return target == ErrUnsupportedWireVersion }
```
- Replace the header block at `:300-302`:
```go
// ─── Header (port of header.rs:26-56). 5 bits; version ∈ {4, 8}. ─────────────

const (
	wfRedesignVersion    = 4 // header.rs WF_REDESIGN_VERSION
	wfUnspendableVersion = 8 // header.rs WF_UNSPENDABLE_VERSION (F-449)
)

// isSupportedVersion is header.rs is_supported_version: exactly {4, 8}.
func isSupportedVersion(v uint8) bool {
	return v == wfRedesignVersion || v == wfUnspendableVersion
}
```
- In `readHeader`, `if version != wfRedesignVersion { return header{}, errWireVersion }` becomes `if !isSupportedVersion(version) { return header{}, &WireVersionError{Got: version} }`.
- `readNode` and `readNodeDepth` gain the parameter:
```go
// readNode reads one node. wireVersion is the version the header declared:
// the Tr arm reads a kind bit only at version 8 (tree.rs read_node).
func readNode(r *bitReader, kiw uint8, wireVersion uint8) (node, error) {
	return readNodeDepth(r, kiw, wireVersion, 0)
}

func readNodeDepth(r *bitReader, kiw uint8, wireVersion uint8, depth uint8) (node, error) {
```
- Every recursive `readNodeDepth(r, kiw, depth+1)` (13 sites) becomes `readNodeDepth(r, kiw, wireVersion, depth+1)`.
- The Tr arm, from Task 3's form, becomes:
```go
	case tagTr:
		// is_nums(1) | [kind(1) iff is_nums && v==8] | [key_index(kiw) iff
		// !is_nums] | has_tree(1) | [tree] -- tree.rs read_node's Tr arm.
		isNums, err := r.readBool()
		if err != nil {
			return node{}, err
		}
		ik := InternalKeySlot
		if isNums {
			ik = InternalKeyNUMS
			if wireVersion == wfUnspendableVersion {
				kind, err := r.readBool()
				if err != nil {
					return node{}, err
				}
				if kind { // SPEC §3d: kind bit 1 = Liana unspendable, 0 = NUMS.
					ik = InternalKeyLianaUnspendable
				}
			}
		}
```
Delete Task 3's four-line `ik := …` block before `b = trBody{ik: ik, …}`, which is now redundant.
- `decodePayload`: `readNode(r, kiw)` becomes `readNode(r, kiw, h.version)`.

`md/encode.go`:
- Add `errLianaNeedsVersion8` to the var block:
```go
	// errLianaNeedsVersion8: a Liana-unspendable internal key reached a write
	// at a version below 8, where it has no kind bit (SPEC §3d).
	errLianaNeedsVersion8 = errors.New("md: Liana-unspendable internal key needs wire version 8")
```
- Add, above `// kiw = ⌈log₂(n)⌉`:
```go
// wireVersion is the MINIMUM wire version that can express d's tree -- port of
// md-codec encode.rs Descriptor::wire_version (SPEC §3d/§3e): 8 iff some Tr
// node anywhere carries a Liana-unspendable internal key, else 4. Every writer
// of tree bits takes its version from here, the identity hashes included, so
// two wallets that differ only in the internal-key kind never share an id.
func (d *descriptor) wireVersion() uint8 {
	if needsV8(d.tree) {
		return wfUnspendableVersion
	}
	return wfRedesignVersion
}

func needsV8(n node) bool {
	switch b := n.body.(type) {
	case trBody:
		return b.ik == InternalKeyLianaUnspendable || (b.tree != nil && needsV8(*b.tree))
	case childrenBody:
		for _, c := range b.children {
			if needsV8(c) {
				return true
			}
		}
	case variableBody:
		for _, c := range b.children {
			if needsV8(c) {
				return true
			}
		}
	}
	return false
}
```
- `writeNode` gains `wireVersion uint8` as its fourth parameter. Add this to its doc comment:
```go
//
// wireVersion is the version this write targets (tree.rs write_node): the Tr
// arm writes a kind bit only at version 8. Callers pass
// descriptor.wireVersion(), NEVER a constant -- the identity hashes included
// (SPEC §3e: at version 4 the two unspendable kinds hash identically).
```
- Its three recursive calls pass `wireVersion` through. Its Tr arm becomes:
```go
		w.write(uint64(b2u(b.isNums())), 1)
		if b.isNums() {
			if wireVersion == wfUnspendableVersion {
				w.write(uint64(b2u(b.ik == InternalKeyLianaUnspendable)), 1)
			} else if b.ik == InternalKeyLianaUnspendable {
				// Kind 1 has no representation below version 8; writing it
				// would silently emit a NUMS wallet. Fail closed (Rust
				// debug_asserts the same state at tree.rs write_node).
				return errLianaNeedsVersion8
			}
		}
		if !b.isNums() {
```
- In `encodePayload`, add `version := dc.wireVersion()` before `var w bitWriter`. The header's `version: wfRedesignVersion` becomes `version: version`, and `writeNode(&w, dc.tree, width)` becomes `writeNode(&w, dc.tree, width, version)`.

`md/chunk.go`:
- `readChunkHeader` becomes `if !isSupportedVersion(version) { return ChunkHeader{}, &WireVersionError{Got: version} }`. Update its doc comment: `errWireVersion if the 4-bit version != WF_REDESIGN_VERSION` becomes `a *WireVersionError if the 4-bit version is outside {4, 8}`.
- In `split`, `Version: wfRedesignVersion,` becomes `Version: d.wireVersion(), // chunk.rs:280 -- never a constant`.

`md/template_id.go:53`: `writeNode(&w, d.tree, width, d.wireVersion()) // SPEC §3e`. `md/walletpolicyid.go:42`: `writeNode(&treeW, dc.tree, width, dc.wireVersion()) // SPEC §3e`.

`md/liana.go`: change the import to `import (\n\t"crypto/sha256"\n\t"errors"\n)`, and add the walker:
```go
var (
	errLianaNotKind1   = errors.New("md: internal key is not Liana-unspendable")
	errLianaNoLeaves   = errors.New("md: Liana-unspendable internal key over a tree with no leaves")
	errLianaMissingKey = errors.New("md: a leaf key has no xpub on this card; the Liana internal key cannot be computed")
)

// lianaLeafPubkeys walks d's root tr script tree in tap-tree left-to-right
// order, and each leaf's key expressions in pre-order, one entry per key
// OCCURRENCE (rust-miniscript TapTree::leaves() + Miniscript::iter_pk(), which
// is what Rust feeds the recipe). Each occurrence's pubkey is bytes [32:65] of
// its slot's Pubkeys-TLV entry.
//
// NEVER the derived-key-sorted order address.MultiALeafScript builds a
// sortedmulti_a script from (SPEC §6 row 1, fable M-8): this reads the WRITTEN
// indices off the wire tree, before any derivation.
func lianaLeafPubkeys(d *descriptor) ([][33]byte, error) {
	b, ok := d.tree.body.(trBody)
	if d.tree.tag != tagTr || !ok || b.ik != InternalKeyLianaUnspendable {
		return nil, errLianaNotKind1
	}
	if b.tree == nil {
		return nil, errLianaNoLeaves
	}
	var idx []uint8
	collectKeyOccurrences(*b.tree, &idx)
	out := make([][33]byte, 0, len(idx))
	for _, i := range idx {
		x, ok := xpubForId(d, i)
		if !ok {
			return nil, errLianaMissingKey
		}
		var pk [33]byte
		copy(pk[:], x[32:65])
		out = append(out, pk)
	}
	return out, nil
}

// collectKeyOccurrences appends every placeholder index in n, left-first
// pre-order. A TapTree node's two children are visited left then right, which
// is the leaves' left-to-right order; inside a leaf it is the fragment order.
func collectKeyOccurrences(n node, out *[]uint8) {
	switch b := n.body.(type) {
	case keyArgBody:
		*out = append(*out, b.index)
	case multiKeysBody:
		*out = append(*out, b.indices...)
	case childrenBody:
		for _, c := range b.children {
			collectKeyOccurrences(c, out)
		}
	case variableBody:
		for _, c := range b.children {
			collectKeyOccurrences(c, out)
		}
	}
}
```

- [ ] **Step 8: Gate.**
```bash
gofmt -l . | sort   # the five-file set
go vet ./md/ && go build ./...
go test -count=1 ./md/          # ok; 179 tests (measured)
go test -count=1 -run 'TestEveryKeyedVectorReachesAnAddress|TestTaprootScriptPathMatchesRust' ./gui/   # ok
```
Measured at this state:
- `TestKeyedConformanceAgreesWithRust` passes both kind-1 records. **Go's `wallet_policy_id`, `wallet_descriptor_template_id` and `md1_encoding_id` equal Rust's**, which is §8.5's equality half.
- `TestKeyedConformanceDescriptorsAgreeWithTheirTemplates` reduces both kind-1 descriptors by byte equality.

- [ ] **Step 9: Commit.** Title: `md: wire version 8 and the Liana kind bit, identities version-derived (F-449 stage 3)`. The message includes Step 2's RED lines and names the pinned dm SHA.

---

## Task 6 (fork): SPEC §6a — name an unsupported wire version on every surface

**The measured surfaces.** Every place a `md.ParseChunkHeader`/`md.Decode` version refusal reached an operator at 7b6f2fb:

| surface | 7b6f2fb | measured at |
| --- | --- | --- |
| gather, a later chunk | `gatherIgnored` → "Not an md1 descriptor chunk." | `gui/md1_gather.go:31-33`, `:122` |
| gather, the **first** chunk | `g.offer(first)`'s status discarded; the screen reads "Captured 0 of 0." | `gui/md1_gather.go:90` |
| inspect, a single card | `md.Decode` default arm → "Can't decode this descriptor." | `gui/gui.go:2786-2787` |
| bundle / payload | `classify` → `clsDrop` → `bundleDropped` → "Not an md1/mk1 card." | `gui/bundle.go:178-181`, `:245-247`, `gui/bundle_flow.go:97-98` |

§6a names only the first row. The bundle row is **the same false statement on a second surface**, and a one-surface fix would leave the other copy saying it. `sysw/confirm.go:123` is **not** changed. It classifies an unsupported version as unconfirmed, which matches Rust's `chunk_key` (its own comment at `sysw/confirm.go:112-120` records the agreement).

- [ ] **Step 1: Write the failing tests.** Create `gui/md1_version_test.go`:
```go
package gui

import (
	"strings"
	"testing"

	"seedhammer.com/codex32"
)

// md1AtVersion rewrites an md1 string's 4-bit wire version and re-checksums
// it: a BCH-valid card at a version no encoder emits. (Test-local twin of
// md/wire_version8_test.go's withVersion; gui cannot import md's test code.)
func md1AtVersion(t *testing.T, s string, v uint8) string {
	t.Helper()
	syms, err := codex32.MDDataSymbols(s)
	if err != nil {
		t.Fatal(err)
	}
	syms = append([]byte(nil), syms...)
	if syms[0]&1 == 1 {
		syms[0] = v<<1 | 1
	} else {
		syms[0] = syms[0]&0b10000 | v&0b1111
	}
	return codex32.AssembleMD1(syms)
}

// SPEC §6a / §8.9 stage-3 row, the gather half. A v12 chunk is a well-formed
// md1 this firmware does not read: it must NOT be "Not an md1 descriptor
// chunk.", and the message must name 12. A v8 chunk -- the plate stage 3
// exists to read -- is ADDED, not refused.
func TestMD1GathererNamesAnUnsupportedVersion(t *testing.T) {
	v8 := loadVectorChunks(t, "keyed_tr_liana_kofn_recovery")[0]
	g := &md1Gatherer{}
	if st := g.offer(md1AtVersion(t, v8, 12)); st != gatherUnsupportedVersion || g.refusedVersion != 12 {
		t.Fatalf("v12 chunk: status %v version %d, want gatherUnsupportedVersion naming 12", st, g.refusedVersion)
	}
	if msg := md1VersionMessage(g.refusedVersion); !strings.Contains(msg, "12") || strings.Contains(msg, "Not an md1") {
		t.Fatalf("message %q must name the version and must not deny the card is md1", msg)
	}
	if st := (&md1Gatherer{}).offer(v8); st != gatherAdded {
		t.Fatalf("v8 chunk: status %v, want gatherAdded -- this firmware reads version 8", st)
	}
	if st := (&md1Gatherer{}).offer("not an md1 chunk"); st != gatherIgnored {
		t.Fatalf("garbage: status %v, want gatherIgnored (unchanged)", st)
	}
}

// The same refusal through the two routes an operator actually takes from the
// inspect chooser: a chunked v12 card (gather flow, first chunk) and a single
// v12 card (md.Decode's default arm). Both must show the ONE phrasing.
func TestMdmkFlowNamesAnUnsupportedMD1Version(t *testing.T) {
	for _, c := range []struct{ name, card string }{
		{"chunked", md1AtVersion(t, loadVectorChunks(t, "keyed_tr_liana_kofn_recovery")[0], 12)},
		{"single", md1AtVersion(t, loadVectorChunks(t, "liana_taproot")[0], 12)},
	} {
		t.Run(c.name, func(t *testing.T) {
			p := newPlatform()
			p.engraver = newEngraver()
			ctx := NewContext(p)
			frame, quit := runUI(ctx, func() { mdmkFlow(ctx, &descriptorTheme, mdmkText(c.card)) })
			defer quit()
			var all strings.Builder
			for i := 0; i < 6; i++ {
				s, ok := frame()
				if !ok {
					break
				}
				all.WriteString(s + "\n")
				if i == 0 {
					chooseInspect(&ctx.Router)
				}
			}
			got := all.String()
			if !uiContains(got, md1VersionMessage(12)) {
				t.Errorf("no %q on screen; got %q", md1VersionMessage(12), got)
			}
			for _, lie := range []string{"Not an md1 descriptor chunk", "Can't decode this descriptor", "Captured 0 of 0"} {
				if uiContains(got, lie) {
					t.Errorf("screen says %q about a well-formed md1 at version 12", lie)
				}
			}
		})
	}
}

// And the bundle channel, which classifies the same strings on its own.
func TestBundleNamesAnUnsupportedMD1Version(t *testing.T) {
	for _, c := range []struct{ name, card string }{
		{"chunked", md1AtVersion(t, loadVectorChunks(t, "keyed_tr_liana_kofn_recovery")[0], 12)},
		{"single", md1AtVersion(t, loadVectorChunks(t, "liana_taproot")[0], 12)},
	} {
		g := &bundleGatherer{}
		st := g.offer(mdmkText(c.card))
		if st != bundleUnsupportedMD1Version || g.refusedMD1Version != 12 {
			t.Errorf("%s: status %v version %d, want bundleUnsupportedMD1Version naming 12", c.name, st, g.refusedMD1Version)
		}
		s := &bundleGatherScreen{g: g}
		if msg := s.feedback(st); msg != md1VersionMessage(12) {
			t.Errorf("%s: feedback %q, want %q", c.name, msg, md1VersionMessage(12))
		}
	}
}

// The positive twins: version 8 is READ on every route that refuses 12, so the
// refusal is the accepted SET and not "anything but 4". A single v8 card
// reaches the display (no version message, no "Can't decode"), and a complete
// v8 chunk set becomes a bundle card.
func TestVersion8CardsAreReadOnEveryRoute(t *testing.T) {
	single := loadVectorChunks(t, "liana_taproot")[0]
	p := newPlatform()
	p.engraver = newEngraver()
	ctx := NewContext(p)
	frame, quit := runUI(ctx, func() { mdmkFlow(ctx, &descriptorTheme, mdmkText(single)) })
	var all strings.Builder
	for i := 0; i < 6; i++ {
		s, ok := frame()
		if !ok {
			break
		}
		all.WriteString(s + "\n")
		if i == 0 {
			chooseInspect(&ctx.Router)
		}
	}
	quit()
	for _, lie := range []string{"cannot read md1 version", "Can't decode this descriptor"} {
		if uiContains(all.String(), lie) {
			t.Errorf("v8 single card: screen says %q", lie)
		}
	}

	g := &bundleGatherer{}
	var last bundleOfferStatus
	for _, c := range loadVectorChunks(t, "keyed_tr_liana_kofn_recovery") {
		last = g.offer(mdmkText(c))
	}
	if last != bundleCardComplete || len(g.cards) != 1 {
		t.Fatalf("v8 chunk set: last status %v with %d cards, want bundleCardComplete and one card", last, len(g.cards))
	}
}
```

- [ ] **Step 2: Run; expected FAIL** (`undefined: gatherUnsupportedVersion`, …).

- [ ] **Step 3: Implement.** Create `gui/md1_version.go`:
```go
package gui

import (
	"errors"
	"fmt"

	"seedhammer.com/md"
)

// md1VersionRefusal reports whether err is md's unsupported-wire-version
// refusal (SPEC_liana_unspendable_internal_key §6a), and the version the card
// DECLARES.
func md1VersionRefusal(err error) (uint8, bool) {
	var wv *md.WireVersionError
	if errors.As(err, &wv) {
		return wv.Got, true
	}
	return 0, false
}

// md1VersionMessage is the ONE phrasing every surface shows for that refusal,
// so no two screens say it differently. It states what the card declares and
// what this firmware can do -- never that the card is intact, current or from
// a newer tool: a BCH mis-correction can land on any header value (F-643).
func md1VersionMessage(v uint8) string {
	return fmt.Sprintf("This firmware cannot read md1 version %d.", v)
}

// md1StringVersionRefusal is md1VersionRefusal for a raw md1 string: the
// chunk header for a chunked card, the payload header for a single one
// (md.ParseChunkHeader does not read a single card's version).
func md1StringVersionRefusal(s string) (uint8, bool) {
	if _, err := md.ParseChunkHeader(s); err != nil {
		return md1VersionRefusal(err)
	}
	_, err := md.Decode(s)
	return md1VersionRefusal(err)
}
```
Then make these edits:
- `gui/mk1_inspect.go`, the `gatherStatus` consts: after `gatherAdded` add
  ```go
  	// gatherUnsupportedVersion: a well-formed md1 chunk whose wire version this
  	// firmware does not read (SPEC §6a). NOT gatherIgnored, whose message --
  	// "Not an md1 descriptor chunk." -- is false about such a card.
  	gatherUnsupportedVersion
  ```
- `gui/md1_gather.go`:
  - Add a field to `md1Gatherer`: `refusedVersion uint8`, with the comment "the wire version the last gatherUnsupportedVersion chunk declared, for the operator message".
  - In `offer`, the `if err != nil { return gatherIgnored }` block becomes:
    ```go
    	if err != nil {
    		if v, ok := md1VersionRefusal(err); ok {
    			g.refusedVersion = v
    			return gatherUnsupportedVersion
    		}
    		return gatherIgnored
    	}
    ```
  - In `md1GatherFlow`, the bare `g.offer(first)` (`:90`) becomes:
    ```go
    	// first came from a chunked md1 mdmkText; it primes the set -- unless its
    	// version is one this firmware does not read, when there is no set to
    	// gather and "Captured 0 of 0" would be a screen about nothing.
    	if g.offer(first) == gatherUnsupportedVersion {
    		showError(ctx, th, "md1 descriptor", md1VersionMessage(g.refusedVersion))
    		return false
    	}
    ```
  - After the `case gatherIgnored:` arm (`:121-122`), add `case gatherUnsupportedVersion: msg = md1VersionMessage(g.refusedVersion)`.
- `gui/gui.go:2786`, the `default:` arm of the single-card `md.Decode` switch:
  ```go
  				default:
  					if v, ok := md1VersionRefusal(err); ok {
  						showError(ctx, th, "md1 descriptor", md1VersionMessage(v))
  						break
  					}
  					showError(ctx, th, "md1 descriptor", "Can't decode this descriptor.")
  ```
- `gui/bundle.go`:
  - Add `bundleUnsupportedMD1Version` at the end of the `bundleOfferStatus` consts, commented "a well-formed md1 at a wire version this firmware does not read (SPEC §6a) — NOT bundleDropped, whose message is false about it".
  - Add a field to `bundleGatherer`: `refusedMD1Version uint8`.
  - In `offer`'s `default:` arm:
    ```go
    	default:
    		if hasMDPrefix(str) {
    			if v, ok := md1StringVersionRefusal(str); ok {
    				g.refusedMD1Version = v
    				return bundleUnsupportedMD1Version
    			}
    		}
    		return bundleDropped
    ```
  - In `offerStandaloneMD1`, the `md.Decode` error arm (`:246-247`):
    ```go
    	if err != nil {
    		if v, ok := md1VersionRefusal(err); ok {
    			g.refusedMD1Version = v
    			return bundleUnsupportedMD1Version
    		}
    		return bundleDropped
    	}
    ```
- `gui/bundle_flow.go`, `feedback`: after the `bundleDropped` arm add `case bundleUnsupportedMD1Version: return md1VersionMessage(s.g.refusedMD1Version)`.

- [ ] **Step 4: Run; expected PASS.**
```bash
go test -count=1 -v -run 'TestMD1GathererNamesAnUnsupportedVersion|TestMdmkFlowNamesAnUnsupportedMD1Version|TestBundleNamesAnUnsupportedMD1Version|TestVersion8CardsAreReadOnEveryRoute' ./gui/
```
Confirm each test prints `=== RUN` (a filter that matches nothing reports `ok`) and `--- PASS`.

- [ ] **Step 5: Whole gui package, sharded.** Run `…/gui-shard-test.sh ./gui/ 24`. Expected: `RESULT: ok -- all 1380 tests ran` (1376 + 4). Commit `gui: name an unsupported md1 wire version on the gather, inspect and bundle surfaces (F-449 stage 3, SPEC §6a)`.

---

## Task 7 (fork): the provenance pin, the whole gate, the size, and landing

- [ ] **Step 1: The package's provenance pin.** `md/bits.go:1-5` is the `md` package doc and the package's pin. At 7b6f2fb it reads `md-codec @ 0.42.0` and "Chunked md1 is detected and refused", which has been false since reassembly landed. Replace lines 1-5 with:
```go
// Package md is the SeedHammer fork's Go port of the md1 (descriptor)
// constellation codec. md1 is PUBLIC; no secret handling.
//
// PROVENANCE PIN (Rust-primary rule). The wire format -- versions {4, 8}, the
// taproot internal key's three states and kind bit -- and the identity hashes
// track descriptor-mnemonic/crates/md-codec 0.47.0 (descriptor-mnemonic
// cf35d61a, tag descriptor-mnemonic-md-cli-v0.19.0), with the kind-1 vectors
// vendored from descriptor-mnemonic <TASK-1 MERGE SHA>. Known lags, each with a
// follow-up: pathless shared origins (F-166); md-codec's compose `--unspendable`
// selection and SPEC §6's kind-1 mint refusals (F-654). compose.go carries its
// own, older pin (md-codec::compose at 66bdf2f4). Vendored corpora are pinned
// per file in md/testdata/*.provenance.json.
package md
```
Substitute the real SHA. A reviewer greps for `<TASK-1` and expects no hit.

- [ ] **Step 2: `md/testdata/README.md`.** Append a section, `### Wire kind 1 (F-449 stage 3)`. It names:
  - the three vectors and what each is for (`liana_taproot` = `nums_taproot`'s twin and the only single-string v8 card; `keyed_tr_liana_kofn_recovery` = `keyed_compose_preset_kofn_recovery`'s twin, the identity-distinctness pair; `keyed_tr_liana_nested_two_recoveries` = the F-640 nested ACCEPT shape, the structure-independence pin);
  - the widened `liana_` selector and why `isComposeVectorFile` widened with it;
  - `liana_cases.json` with its own pin and script;
  - the regenerate command `scripts/vendor-compose-vectors.sh /scratch/code/shibboleth/descriptor-mnemonic` (explicit path, see Global Constraints).

- [ ] **Step 3: The whole gate, run once, captured.**
```bash
gofmt -l . | sort > /tmp/claude-1000/f449s3-gofmt.txt; diff <(printf '%s\n' gui/transaction.go gui/transaction_golden_test.go gui/transaction_txrecord_test.go mt/mt.go mt/mt_test.go) /tmp/claude-1000/f449s3-gofmt.txt
go vet ./md/ && go vet ./gui/ 2>&1 | grep -v ArtifactDir | grep -v '^#' ; echo "vet: only the 2 baseline ArtifactDir lines expected"
go test -count=1 $(go list ./... | grep -v '/gui$') > /tmp/claude-1000/f449s3-nongui.txt 2>&1; grep -vE '^ok|no test files' /tmp/claude-1000/f449s3-nongui.txt
/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24 > /tmp/claude-1000/f449s3-gui.txt 2>&1; tail -1 /tmp/claude-1000/f449s3-gui.txt
```
Expected, measured:
- the gofmt diff is empty;
- the non-gui grep prints nothing (55 `ok`);
- gui prints `all 1380 tests ran`;
- md is 179 tests.

- [ ] **Step 4: Firmware size.** This stage adds code to the image. From the worktree, with nix on PATH:
```bash
export PATH=/nix/var/nix/profiles/default/bin:$PATH
nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
```
Measured: baseline **1,655,388 B flash / 63,336 B RAM** at 7b6f2fb, end state **1,658,284 / 63,352** (+2,896 B flash, +16 B RAM). If `nix develop` refuses the worktree's flake, `nix develop /scratch/code/shibboleth/seedhammer -c …` uses the same flake at 7b6f2fb, which is how these numbers were taken. `md/liana.go`'s recipe is test-only in this stage and dead-code eliminated, so stage 4 will add its bytes. Record both lines in the commit message.

- [ ] **Step 5: The mutation pass.** Re-run the table in the Self-Review against the real tree. For each row: apply the mutation, **check it applied** (the replaced text existed), check the build **compiled** (a build failure reads as a pass in a naive grep), run the named package, and confirm the listed test fails. Then revert with `git checkout -- <file>`, and only on a clean tree (never on a file holding unstaged work).

- [ ] **Step 6: Commit and land.** Commit the pin and README as `md: provenance pin to md-codec 0.47.0 (F-449 stage 3)`. Then:
```bash
git -C /scratch/code/shibboleth/seedhammer merge --no-ff f449-stage3
cd /scratch/code/shibboleth/seedhammer && ./scripts/push-via-staging.sh
```
Commits use the fork's configured signing, authored Brian Goss. **No flash in this stage.** The device-visible payoff arrives with stage 4, and a flash needs the operator at the board. The push is the last action on the fork in the turn.

---

## Task 8 (mnemonic-engrave): records — F-643 ruled, F-654/F-655 filed, the spec reconciled

- [ ] **Step 1: Rule on F-643.** In `design/FOLLOWUPS.md`, append a `**Ruling (F-449 stage 3):**` paragraph to F-643 and re-own it to **next descriptor-mnemonic release after F-449 stage 2** (with F-645/F-646/F-648/F-651). Keep **Status:** OPEN. Content, measured:
  - **The Go side has no exit-5 analogue.** The device's only correction path is typed entry, "Fix?" and `codex32.Correct` (`gui/gui.go:1296`). The operator confirms a per-position diff against the card (`confirmCorrectionFlow`), and the corrected string then goes through the full decoder (`mdmkFlow` → `md.Decode`/`Reassemble`) before anything uses it. So the device never presents a corrected string it has not decoded.
  - **The direction "require a structural check the version-independent part can pass" is unimplementable.** Past the 5-bit header, a single card's layout at an unsupported version is unknowable. That is the same reason ruling 7 cut the exit-5 branch to single strings.
  - **The remedy is wording**, and stage 3 applies it on the device. The message states what the card *declares* ("This firmware cannot read md1 version N.") and never that it is intact. For md-cli, the exit-5 stderr's "take the corrected card to a newer md, which may read wire version {got}" should add that a correction beyond BCH capacity can also land here. This is message precision, not blocking, and belongs to that release.

- [ ] **Step 2: File F-654, owning phase: F-449 stage 4.** Title: "The Go composer has no Liana kind and no §6 kind-1 mint refusals". Content:
  - `md.ComposeWith` (`md/compose.go:653`) has no counterpart to md-codec's `UnspendableKind` parameter (stage 2, `compose/tr.rs`).
  - SPEC §6's four kind-1 refusals (`validate_unspendable_shape`, `validate_minimal_wire_version`, reached from `encode.rs:251-252`) are not ported.
  - Both belong with the first Go **producer** of kind 1, which is stage 4's choice screen.
  - **Constraint for whoever does it:** the refusals must not go into `encodePayload`, because `Reassemble` re-encodes decoded cards for the chunk-set-id check (see this plan's Global Constraints).
  - Take the next free ID at filing time (F-654 at `bebb532a`; re-grep, because the stage 4a plan is being written concurrently).

- [ ] **Step 3: File F-655, owning phase: next descriptor-mnemonic release.** Title: "`skeleton_key_conformance.rs`'s kind-1 recogniser has no near-miss vector". This is Task 1 Step 9's measured survivor: replacing the recipe equality with `true` keeps the Rust harness green. Direction: add a near-miss kind-1 descriptor (a recipe output over a different leaf order) to the Rust harness, as the fork's `TestLianaReductionRefusesANearMiss` does.

- [ ] **Step 4: The reconciliation sweep** (the standing rule from stage 2's Task 7). Grep for the superseded claims. Rewrite each against the tree, citing SHAs:
  - **§8.9's stage-3 row** reads "a version-8 chunk yields 'unsupported wire version'". On stage-3 firmware a v8 chunk is **read** (Task 6's positive twin). Reword it to "a chunk at a version OUTSIDE the accepted set (e.g. 12) yields the named-version message on the gather, inspect and bundle surfaces". Add: "a board not flashed with stage 3 still says 'Not an md1 descriptor chunk.' about a v8 plate. No code can fix a board already in the field, so the stage 5 runbook must say to flash before reading kind-1 plates." This is the same correction stage 2 made to its own row.
  - **§6a row 1:** `errWireVersion is unexported (md/md.go:22)` is now `md.ErrUnsupportedWireVersion` + `*md.WireVersionError`.
  - **§7a.1:** "returns a two-state `(keyIndex, isNums)`" is now three-state. **§7a.3:** the address half of the refusal shipped in stage 3 (`gui/policy_address.go`'s `default:` arm, gated by the `stillUnsupported` entries). The engrave half stays stage 4.
  - **§8.5:** "a different 12-word phrase" has **no Go surface**. `md/` ports no phrase (`grep -n -i phrase md/walletpolicyid.go md/template_id.go md/identity.go` finds none), so the Go leg is ids plus both stub flavours plus the encoding id.
  - **§3c:** the citation `md/md.go:1235` was `:1236` at 7b6f2fb and moves again with this stage. Re-resolve every Go citation in §3c, §3e, §6a and §7a against the landed fork SHA with the resolve-and-print one-liner (stage 2 plan's Self-Review), and paste the output into the commit.
  - **§9's stage-3 row:** add a **Status:** line naming the fork merge SHA and Task 1's dm SHA. List what landed beyond the row: the recipe port, the bundle surface, the `md1_encoding_id` assertion, and the address half of §7a.3. List what moved: F-654 to stage 4.
  - Run `scripts/followups-status.sh` (expect `OK`). Commit the sweep separately from the follow-ups.

---

## What no gate covers

- **What the device screen actually shows.** Every gui assertion reads extracted text, and text extraction cannot see clipping. The new message is 42 characters, about the length of the existing "No fix within 4 changes - check your typing". Stage 5's emulator walk is where it is seen.
- **Hardware.** Nothing is flashed in this stage.
- **The old decoder's refusal of a v8 plate (§8.4's last leg)** is a **measurement** at 7b6f2fb (Task 5 Step 2's RED lines), not a regression gate. That decoder no longer exists in the tree.
- **Kind 1 over a `sortedmulti_a` leaf.** The walker reads written indices, as Rust's `iter_pk` does. No vector exists, because Rust refuses the shape at mint (§6 row 1) and it is reachable only as a hand-crafted payload.
- **The Rust harness recogniser's precision** (Task 1 Step 9: no near-miss vector in dm) goes to F-655.
- **`policyShape` reports kind 1 as `KeyPathNUMS`** until stage 4's §7 sibling kind. *(Corrected per R0 m2.)* On the inspect screen this shows as no `Policy id:` line and no address for kind 1, not as a NUMS key-path line. Nothing shown is false, but kind 1 loses the one inspect line that would distinguish it until stage 4.
- **`sysw/confirm.go`** now confirms v8 records (it was unconfirmed at 7b6f2fb). No fork test covers v8 there. The record-class vectors come from `me`'s Rust primary, which is stage 4a's.
- **`plan-build-gate-go.sh`** extracts nothing from this plan (see "How this plan was checked"). A fold to this plan re-earns the gate by re-applying its blocks to a scratch export of the baseline trees, not by that script.

---

## Self-Review

**Spec coverage (§9's stage-3 row, item by item):**
- Three-state `EmitTapLeavesChunks` (§7a.1): **Task 3**, plus the same change to `TapLeavesChunks`.
- Version-derived identity at `md/encode.go:417`, `md/template_id.go:53` and `md/walletpolicyid.go:42` (§3e): **Task 5** Step 7. All three citations were re-resolved at 7b6f2fb and **did not drift**. `split`'s chunk header is a fourth writer, and it is covered too.
- §6a's `gatherIgnored` split: **Task 6**, on three surfaces.
- The provenance pin: **Task 7** Step 1 (package doc) and **Task 5** Step 1 (corpus pin).
- §8.4 Go leg: **Task 5** (single and chunked, both versions, v12 refused by name) plus the Step 2 measurement.
- §8.5 Go leg: equality via `TestKeyedConformanceAgreesWithRust` (+ `md1_encoding_id`), distinctness via `TestKind0AndKind1TwinsNeverShareAnIdentity`. The existing-v4 identities are byte-preserved: all 46 prior keyed records still pass the same gate.
- §8.9's stage-3 row: **Task 6**, with the row reworded in Task 8.
- F-643: **Task 8** Step 1.
- Also covered: §8.1 in Go (Task 4, all nine Liana goldens), §8.6 (Task 5, `TestLianaKeyDependsOnTheLeafKeysNotTheTree`) and §8.10's two named classes (below).

**Mutation table.** Measured against the end state in scratch. Every mutation applied and compiled.

| # | mutation | caught by |
| --- | --- | --- |
| M1 | kind-bit polarity inverted on write only | `TestKindBitPolarityIsPinnedOnTheWire` + 11 more |
| M1b | polarity inverted on write **and** read (symmetric; SPEC §8.10) | `TestKindBitPolarityIsPinnedOnTheWire`, `TestKeyedConformanceAgreesWithRust`, `TestEncodePayloadGoldens`, … (11) |
| M2a | identity sites pass `wfRedesignVersion` | `TestKeyedConformanceAgreesWithRust`, `TestKind0AndKind1TwinsNeverShareAnIdentity`, `FuzzWalletPolicyId` |
| M2b | M2a **and** the `errLianaNeedsVersion8` guard removed (silent collision) | `TestKind0AndKind1TwinsNeverShareAnIdentity`, `TestKeyedConformanceAgreesWithRust`, `TestLianaHasNoRepresentationBelowVersion8` |
| M3 | `split` writes `wfRedesignVersion` in chunk headers | `TestVersion8ChunkSetsThroughTheDispatch` |
| M4 | `readChunkHeader` accepts only 4 | `TestAnUnsupportedWireVersionIsRefusedByName`, the conformance gates, … (8) |
| M5a | `remapIndices` rebuilds the kind from `isNums()` | `TestCanonicalizeKeepsTheKindOnANonCanonicalTree` (**the only test that reaches that site**) |
| M5b | `cloneNode` rebuilds the kind from `isNums()` | 11 tests incl. `TestCanonicalizeIdempotentOnGoldens` |
| M6 | `lianaLeafPubkeys` sorts by pubkey | `TestKeyedConformanceDescriptorsAgreeWithTheirTemplates`, `TestLianaReductionRefusesANearMiss` |
| M7 | the D1' reduction skips the byte-equality clause (SPEC §8.10's weakening) | `TestLianaReductionRefusesANearMiss` |
| M8 | `wireVersion()` always 4 | 12 tests |
| M9 | the recipe deduplicates | `TestLianaRecipeIsOrderSensitiveAndKeepsDuplicates` |
| M10 | `readHeader` accepts any version | `TestAnUnsupportedWireVersionIsRefusedByName`, `TestDecodeNegative` |
| G1 | gatherer returns `gatherIgnored` for a version refusal | `TestMD1GathererNamesAnUnsupportedVersion`, `TestMdmkFlowNamesAnUnsupportedMD1Version/chunked` |
| G2 | first-chunk check removed | `TestMdmkFlowNamesAnUnsupportedMD1Version/chunked` |
| G3 | single-card arm stays generic | `TestMdmkFlowNamesAnUnsupportedMD1Version/single` |
| G4 / G5 | bundle default / standalone arm stays `bundleDropped` | `TestBundleNamesAnUnsupportedMD1Version` |
| G6 | kind 1 grouped into the NUMS branch (`case md.InternalKeyNUMS, md.InternalKeyLianaUnspendable:`), SPEC §7a row 1 | `TestEveryKeyedVectorReachesAnAddress` ("listed as unsupported … but now derives") |
| G7 | 8 treated as unsupported in both headers | `TestVersion8CardsAreReadOnEveryRoute` (both halves) |

Two survivors were found and closed while writing this plan:
- M5a first **survived**, because every vendored card is canonical and `remapIndices` is skipped on the identity permutation. `TestCanonicalizeKeepsTheKindOnANonCanonicalTree` exists for it.
- An index-sort version of M6 is **inert by construction**: canonical placeholder numbering makes first-occurrence order ascending. So the row sorts by pubkey instead.

**Type consistency.** These are the names used across tasks, each checked against the scratch build:
- `InternalKeyKind` / `InternalKeySlot` / `InternalKeyNUMS` / `InternalKeyLianaUnspendable` / `trBody.ik` / `trBody.isNums()` (Task 3)
- `lianaUnspendableKey` (Task 4)
- `lianaLeafPubkeys`, `wireVersion()`, `WireVersionError{Got}`, `ErrUnsupportedWireVersion`, `wfUnspendableVersion`, `errLianaNeedsVersion8` (Task 5)
- `gatherUnsupportedVersion`, `md1Gatherer.refusedVersion`, `bundleUnsupportedMD1Version`, `bundleGatherer.refusedMD1Version`, `md1VersionRefusal`, `md1StringVersionRefusal`, `md1VersionMessage` (Task 6)

**Placeholder scan.** One deliberate substitution remains: `<TASK-1 MERGE SHA>` in Task 7 Step 1, which cannot be known before Task 1 runs. Task 7 tells the reviewer to grep for it.

**Scope calls a reviewer may reject, each stated in its task:**
- the recipe port in `md/` (Task 4, needed for byte-equality D1');
- the bundle surface (Task 6, the same false statement on a second surface);
- the `md1_encoding_id` assertion (Task 5, zero-cost and the only id that sees the version);
- the address half of §7a.3 landing now (Task 3, forced, because the signature change leaves the one caller no silent option).

**Deliberately absent, owned elsewhere:**
- the device's kind-1 derivation, §7's `KeyPathKind` sibling and §0b's screen (stage 4);
- the Go composer's Liana selection and §6 mint refusals (F-654, stage 4);
- `me` (stage 4a);
- the toolkit (F-642);
- the demo site (stage 5).
