# F-642 implementation report: mnemonic-toolkit md-codec 0.47.0 pin bump, golden refresh, `mnemonic repair` convergence

**Status: DONE_WITH_CONCERNS**

- Branch: `f642-md-codec-0.47`
- Worktree: `/scratch/code/shibboleth/tk-worktrees/f642`
- Base: `master` @ `53457147`. The brief said "main", but the toolkit's default branch is `master`.
- Branch HEAD: `e60dd57b`
- Not pushed, not merged, not tagged.

## Commits (master..HEAD)

| SHA | Piece | Contents |
|---|---|---|
| `f79aaa7c` | 1 | Pin moves to `cf35d61af0f058029df841f4882945e53f86cd4b` (md-codec 0.47.0). Also: `Cargo.lock`, `cargo vendor` re-run (only `vendor/md-codec` changed), the compile repairs, the new restore refusal plus its test, the Error arms, and CHANGELOG. |
| `0f92682f` | 2 | install.sh and four doc workflows move md-cli to `descriptor-mnemonic-md-cli-v0.19.0`. `.examples-build/Examples.md` is regenerated **in the same commit** (the known trap). |
| `e60dd57b` | 3 | `mnemonic repair` goes through `md_codec::correct_chunks`: a corrected single-string md1 at an unreadable wire version exits 5. Includes tests, manual, and CHANGELOG. |

## Piece 1: what the bump forced (measured by the compiler, not the brief)

The brief listed Error variants only. The bump also broke `Body::Tr`: md-codec 0.46.0 replaced `is_nums: bool` and `key_index: u8` with `internal_key: InternalKey::{Slot(u8), NumsPoint, LianaUnspendable}`. That produced 37 compile errors across these files:

- `restore.rs`
- `parse_descriptor.rs` (walk_tr and 2 tests)
- `synthesize.rs` (test)
- `taproot_override_classify.rs`
- `template.rs` (2 constructors and 5 tests)
- `tests/cli_restore_taproot.rs`
- `error.rs`
- `friendly.rs`

**Mechanical migration.** `is_nums: true` becomes `NumsPoint`; otherwise `Slot(key_index)`. No toolkit constructor emits `LianaUnspendable`.

**Error variants.** I diffed `crates/md-codec/src/error.rs` between `b2c5d693` and `cf35d61a`. There are exactly 5 new variants:

- `NetworkRequiredForUnspendable`
- `UnspendableWithSortedMultiA`
- `UnspendableUseSiteNotCanonical { idx: Option<u8> }`
- `UnspendableNotRootTr`
- `NonMinimalWireVersion { got, minimal }`

All five route to exit 2 in `error.rs` and have prose in `friendly.rs`. The friendly table test grew from 45 to 51 rows; `idx` has both a `None` row and a `Some` row. `WireVersionMismatch`'s text also changed to "accepted versions: 4, 8".

**New funds-safety refusal.** `classify_taproot_restore` now refuses `InternalKey::LianaUnspendable` with ModeViolation, exit 2. Without the refusal, the Template arm would render `tr(NUMS, multi_a(...))`, which is a different wallet.

- **Test:** `liana_unspendable_internal_key_refuses_not_nums` in `tests/cli_restore_taproot.rs`. The builder is generalised to `build_tr_leaf_descriptor(internal_key, ...)`; the @-in-both builder now wraps it with `Slot(0)`.
- **Mutation:** mapping Liana to `TaprootInternalKey::Nums` fails the test with `code=0` and stdout `descriptor: tr(50929b74...,multi_a(2,...))#jwp6nzr2`. That is the hazard, reproduced.
- **Parity:** `restorable_taproot_override_card` now matches `NumsPoint` only, so Liana counts as unrestorable there too.
- **No mutation run** for that parity predicate change.
- **Printed recipe verified before shipping:** `md descriptor --network` exists at cf35d61a. `md decode` has no `--network`; I had drafted it first and fixed it.

**Other network-less callers.** `synthesize.rs:994` `template_admissible` calls `to_miniscript_descriptor(..).is_ok()`. A Liana tree now returns `Err(NetworkRequiredForUnspendable)`, so it is refused rather than mis-rendered. The only other callers are in tests.

## Piece 2: pins and golden

- **Tag checked on the remote:** `git ls-remote ... 'refs/tags/descriptor-mnemonic-md-cli-v0.19.0^{}'` returns `cf35d61a...`, which equals the Cargo.toml rev.
- **Five pins moved together:** install.sh, cross-tool-differential.yml, manual.yml, quickstart.yml, technical-manual.yml. manual-gui.yml's v0.11.0 was left alone (a separate, pre-existing tier).
- **Golden regenerated** with `EXAMPLES_BIN_DIR=$CARGO_TARGET_DIR/debug bash .examples-build/gen.sh`. The diff is one line, the install table row. No `inspect`/template output moved, because no example carries a Liana card, so F-642's `UNSPENDABLE(liana)` golden concern has nothing to change today. After piece 3, a re-run is byte-identical.

## Piece 3: repair convergence

Design choice: keep `repair_card`'s `Ok` contract unchanged for the auto-repair sites (convert, inspect, verify-bundle).

- **New error variant.** `repair_via_md_codec` maps `WireVersionMismatch` to a new `RepairError::WireVersionUnsupported { got }`. Its Display is byte-identical to the old `PostCorrectionDecodeFailed { chunk_index: None }` text, and its indel-trigger routing is the same. A unit test pins both: `wire_version_unsupported_renders_and_routes_as_before`.
- **Only `mnemonic repair` handles it.** `cmd/repair.rs run()` catches the variant for md1 and calls the new `repair::correct_unreadable_md1(chunks, got)`. That function returns `Some` only when there is exactly one string AND `correct_chunks` corrected at least one character.
- **Output on the new path:**
  - a new `SetVerify::UnreadableVersion { got }`, reported in JSON as verdict `"unreadable_version"`;
  - exit 5;
  - md's two stderr lines with the `repair:` prefix: first the accepted set, then the advice (an even version above 8 is sent to "a newer md"; any other version is called "pre-v0.30 or misread");
  - no output-class advisory.
- **Everything else still exits 2** with the same error as before.

**TDD.** `tests/cli_repair_unsupported_version.rs` has 6 tests, using md-cli's own fixtures from cf35d61a.

- **RED first:** the 4 exit-5 tests failed with `left: 2 right: 5` and the error "post-correction decode failed: wire-format version mismatch: got 12; accepted versions: 4, 8". The 2 exit-2 controls (multi-string, clean v12) passed, pinning prior behaviour.
- **Mutations:** each was asserted applied and reverted from a backup (restore verified by `diff`). Every one was killed by its intended test:

| # | Mutation | Killed by |
|---|---|---|
| M1 | `correct_unreadable_md1` always returns `None` | single-string exit-5, JSON, legacy, odd-v9 |
| M2 | ruling-7 single-string guard dropped | multi-chunk exit-2 |
| M3 | clean-card guard dropped (exits 0) | clean-v12 exit-2 |
| M4 | advice parity half (`got % 2 == 0`) dropped | odd-v9 |
| M5 | advice above-newest half (`got > newest`) dropped | legacy v0 |
| M6 | cmd branch bypassed (`if false`) | single-string, JSON, legacy, odd-v9 |
| M7 | verdict relabelled `"blessed"` | JSON |

**Docs.** The manual (`41-mnemonic.md`) gets:

- a new section, "md1 at an unreadable wire version", anchor `#mnemonic-repair-md1-unreadable-version`;
- exit-5 and exit-2 rows in the exit-code table;
- a JSON verdict note.

The module doc comment and CHANGELOG `[Unreleased]` are also updated.

## Gates (final tree, `e60dd57b`)

Toolchain: pinned 1.85.0 (`~/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin` prepended). `CARGO_TARGET_DIR=/scratch/code/shibboleth/tk-worktrees/f642-target`.

| Gate | Result |
|---|---|
| `cargo nextest run --locked --workspace --no-fail-fast` (captured once per stage) | **4049 passed / 0 failed / 20 skipped**. Progression: 4041 after the bump alone, 4042 at `f79aaa7c`. |
| `cargo clippy --locked --all-targets -- -D warnings` | clean |
| `cargo fmt --all --check` | clean |
| `bash ci/repro/vendor-freshness.sh` | step 1/4 OK ("vendor/ satisfies Cargo.lock"), then rc=1 with the md-codec "no offline provenance anchor" note. Also rc=1 on master: this is the known `md-codec-unpublishable-while-patched-to-miniscript-master` item, not this change. |
| `make lint` (docs/manual), MD_BIN = md 0.19.0 built at cf35d61a | rc=2 with 3 FAILs, a set **identical** to the same lint run with md 0.18.0. None of them is md's or repair's: `mnemonic restore --recalibrate-threads`, and `ms hashlock --kind` and `--phrase-looks-like-digest-ok`. markdownlint: 0 errors. Links: 0 errors. |
| Examples golden | regenerates byte-identical at HEAD |

## Rulings

- **Ruling:** the three `Unspendable*` refusals go to exit 2.
  - **Why:** they are detectable from the card alone and refused, the same class as `DuplicateKeySlots` and `ForbiddenTapTreeLeaf`.
  - **Cost if wrong:** a script keyed on 2 vs 1. Low.
- **Ruling:** `NonMinimalWireVersion` goes to exit 2.
  - **Why:** it is an encode-side reject no public encoder reaches, analogous to `PayloadTooLongForSingleString`.
  - **Cost if wrong:** nil, since it is unreachable.
- **Ruling:** `NetworkRequiredForUnspendable` goes to exit 2.
  - **Why:** it is a render-time refusal, the nearest sibling of `AddressDerivationFailed`. The friendly text calls it a toolkit bug and names `md descriptor --network`.
  - **Cost if wrong:** if it should read as internal error (1), scripts see 2. Low; no current path reaches it except `template_admissible`, which swallows it.
- **Ruling:** restore REFUSES a Liana internal key (ModeViolation, exit 2) rather than rendering it.
  - **Why:** the toolkit has no network-aware render arm, and folding the key into NUMS is a silent wrong wallet (mutation-proven).
  - **Cost if wrong:** a Liana user cannot restore via `mnemonic` and must use `md descriptor`, a capability gap rather than a funds risk.
- **Ruling:** the JSON verdict for the new path is the new value `"unreadable_version"`, not `"blessed"` or `"candidate"`.
  - **Why:** `"candidate"` means exit 4 and `"blessed"` means verified; neither is true here.
  - **Cost if wrong:** a GUI or JSON consumer that enumerates verdicts meets a new value. The wire shape is not schema_mirror-gated, so under the paired-PR rule mnemonic-gui should learn this value.
- **Ruling:** the new `RepairError` variant is an indel trigger.
  - **Why:** it preserves the routing of `PostCorrectionDecodeFailed`, the variant it was carved from. The single-string correction branch runs before the indel arm, so a corrected card exits 5 even under `--max-indel`.
  - **Cost if wrong:** under `--max-indel`, a multi-string or clean card at an unreadable version still enters indel search, as before.

## Concerns

1. **Exit 5 contradicts the toolkit's own exit-5 meaning.** The toolkit's manual defines exit 5 as "corrected AND self-verified". Since v0.86.0 (`toolkit-v0860-demote`), it demotes a touched **non-chunked single-string** md1 correction to exit 4, because no content-id oracle checks it. The new unreadable-version path verifies strictly less (BCH only, no decode at all), yet exits 5 as the brief and md parity require. I implemented 5 as asked and documented it in the manual as "NOT self-verified; treat like a VERIFY-ME candidate". An operator ruling is recommended: parity with md (5) or the toolkit's own rule (4). Switching to 4 is a one-line change: set `candidate_seen` in the new branch, and adjust 5 tests and the docs.
2. **`md compose --unspendable <KIND>` (new in md-cli 0.19.0) has no manual coverage, and no gate can see that.** `md compose` is missing from `docs/manual/tests/cli-subcommands.list` and from `42-md.md` entirely. The lint's whole-chapter substring match would also accept `--unspendable` via `--unspendable-key`. This predates F-642 (compose was never listed), so the pin move turns nothing red, but the mirror invariant is unmet. I did not widen scope; this is a follow-up candidate.
3. **Stale text in descriptor-mnemonic, fixed by this work.** In descriptor-mnemonic, `crates/md-cli/src/cmd/repair.rs`'s module doc (the "DIVERGENCE, not parity ... until the toolkit adopts correct_chunks (F-642)" block) and F-642's `D26` note become stale once this branch ships. That repo is outside this brief.
4. **Pre-existing manual lint FAILs.** These would make the `manual` job red independent of this branch: `mnemonic restore --recalibrate-threads` undocumented, and `ms hashlock --kind` and `--phrase-looks-like-digest-ok` undocumented.
5. **Two lockfiles are still stale, as they were in the previous bump.** `fuzz/Cargo.lock` still pins md-codec 0.42.0 from the registry. `docs/technical-manual/examples` pins `md-codec-v0.32.0`. Neither is part of the main workspace, and the precedent bump (`b9c884b5`) left both alone.
6. **CHANGELOG ordering.** The `[Unreleased]` bullet about install.sh landed in `f79aaa7c`, one commit before the pin move itself (`0f92682f`). Both are on the branch, so this only matters when bisecting.

## Deviations from the brief

- The branch is based on `master` (the toolkit has no `main`).
- Piece 1 was much larger than "Error arms": the `Body::Tr` → `InternalKey` migration is described above, and it added a new restore refusal that the brief did not anticipate.
- The golden refresh changed only the install-table line; there was no `inspect` template drift to refresh.
- Commits 1 and 3 have no separate "failing test" commit. The RED runs are recorded in this report and in commit 3's message.
