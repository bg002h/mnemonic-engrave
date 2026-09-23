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

---

## Addendum: ruling 8 and the 0.104.0 release commit (coordinator follow-up)

**Status: DONE.**

- Branch HEAD: `0a3c629f`
- Not pushed, merged, or tagged. `git tag --points-at HEAD` is empty.

### New commits

| SHA | Contents |
|---|---|
| `f0f6dd19` | **Ruling 8.** A corrected single-string md1 at an unreadable wire version now exits **4** (VERIFY-ME), not 5. |
| `0a3c629f` | Release: mnemonic-toolkit **0.104.0**. |

**What ruling 8 changed:**
- `cmd/repair.rs` sets `candidate_seen` on the kept correction, which folds it into the existing exit-4 tier.
- Unchanged: the stdout report, the JSON verdict `"unreadable_version"`, and both version-dependent advice lines.
- The four correction tests now assert 4. One was renamed `..._is_kept_and_exits_4`.
- Docs moved with it:
  - The manual's exit-code table moves the case from row 5 to row 4.
  - The section "md1 at an unreadable wire version" gains a "Why 4, not 5" paragraph that names the difference from `md repair`.
  - The JSON verdict note says the value exits 4.
  - The module and doc comments are updated.
  - The CHANGELOG says explicitly that `md repair` exits 5 on the same card.

**Mutations re-proved** against the updated tests. Each was asserted applied, then reverted (restore verified by `diff`):

| # | Mutation | Killed by |
|---|---|---|
| **M8 (new)** | drop `candidate_seen = true`, so the exit reverts to 5 | single-string, JSON, legacy, odd-v9 |
| M1 | `correct_unreadable_md1` always returns `None` | single-string, JSON, legacy, odd-v9 |
| M2 | ruling-7 single-string guard dropped | multi-chunk |
| M3 | clean-card guard dropped | clean-v12 |
| M4 | advice parity half dropped | odd-v9 |
| M5 | advice above-newest half dropped | legacy |
| M6 | command branch bypassed | single-string, JSON, legacy, odd-v9 |
| M7 | verdict relabelled `"blessed"` | JSON |

### Release commit

**Shape measured from `cebf5779`:** the same 8 files.

| File | Change |
|---|---|
| `crates/mnemonic-toolkit/Cargo.toml` | version 0.103.2 → 0.104.0 |
| `Cargo.lock` | one line |
| `README.md` | `toolkit-version` marker |
| `crates/mnemonic-toolkit/README.md` | `toolkit-version` marker |
| `scripts/install.sh` | self-pin `mnemonic-toolkit-v0.104.0` |
| `.examples-build/gen.sh` | 6 version strings |
| `CHANGELOG.md` | new section |
| `.examples-build/Examples.md` | regenerated with `mnemonic 0.104.0`; 16 lines, version strings only (checked: no non-version diff line) |

**CHANGELOG handling:**
- A new `## mnemonic-toolkit [0.104.0] — 2026-09-23` section sits directly above `[0.103.2]`, with a SemVer-MINOR lead that gives the reason: the restore refusal and the new JSON verdict value.
- The F-642 entry moved into that section under `### Changed`.
- `[Unreleased]` still holds only the P3 display-grouping block. That block has shipped in every tag since v0.98.0; the 0.103.2 release also left it there, so I followed that precedent and did not touch it. It is stale, so it is a candidate for cleanup.

### Full gate on HEAD `0a3c629f`

Pinned toolchain 1.85.0. `CARGO_TARGET_DIR=/scratch/code/shibboleth/tk-worktrees/f642-target`.

| Gate | Result |
|---|---|
| `cargo nextest run --locked --workspace --no-fail-fast` (once, captured) | **4049 passed / 0 failed / 20 skipped** |
| `cargo clippy --locked --all-targets -- -D warnings` | clean (rc=0) |
| `cargo fmt --all --check` | clean (rc=0) |
| Examples golden (`EXAMPLES_BIN_DIR=.../debug bash .examples-build/gen.sh`, then `cmp`) | byte-identical to the committed file |
| Manual `make lint` (MD_BIN = md 0.19.0 built at cf35d61a) | rc=2. The FAIL set is **identical** to the 3 known pre-existing: `mnemonic restore --recalibrate-threads`, and `ms hashlock --kind` and `--phrase-looks-like-digest-ok`. markdownlint: 0 errors. Links: 0 errors. |
| `ci/repro/vendor-freshness.sh` | step 1/4 OK. rc=1 with the known md-codec provenance note, the same as on master. |

**Concern 1 in the report above is resolved by ruling 8.** Concerns 2 to 6 still stand. One new item: the stale P3 block in `[Unreleased]`, described above.

---

## Addendum 2: review fix round (I-1, M-1, M-4)

This round responds to `design/agent-reports/f642-toolkit-pin-bump-review.md`.

**Status: DONE.**

- Branch HEAD: `d02382d6` (fix commit, on top of the release commit `0a3c629f`).
- Not pushed, merged, or tagged.
- The 0.104.0 CHANGELOG section was edited after the release commit. The branch is untagged, so the release stays 0.104.0 with this fix inside it.

### I-1: the Liana refusal in the template-completion engine

**The rule now exists once.**

- **Predicate:** `taproot_override_classify::liana_unspendable_card(d)`. It is md-codec's own condition, `d.wire_version() == WF_UNSPENDABLE_VERSION`. That check covers the whole tree, and it is exactly when md-codec's network-less renderers return `NetworkRequiredForUnspendable`.
- **Wording:** `taproot_override_classify::LIANA_UNSPENDABLE_REFUSAL`, the only copy.
- **Refusal:** `cmd::restore::refuse_liana_unspendable(d)` returns `ModeViolation`, exit 2.

**Call sites.** No single boundary exists that every route passes through:

- The keyed route (`run_multisig`) never enters the engine.
- `verify-bundle` never enters restore's dispatch.

So the one function is called at the two route entries:

- `run_multisig`, straight after decode;
- `complete_multisig_template`, as its first statement, before the own-account, hardened-path and override gates.

The engine is shared by restore (search-address, explicit `@N=`, expect-wallet-id) and verify-bundle, so that one call covers all four.

**Keyed-route backstop.** `classify_taproot_restore`'s Liana arm is now unreachable. It is kept as defense in depth and returns the same `liana_unspendable_refusal()`, not a second wording.

**The recipe was measured, not assumed.**

- `md descriptor --network mainnet <tpl> --from-mk1 <cards>` **refuses** this Liana template card as ambiguous: 6 complete assignments that compose different wallets, because every slot declares the same path and no fingerprint.
- So the wording names `md descriptor --network <net> <md1…>` for a keyed card, and for a keyless template card `--template <what md decode prints> --key @i=<xpub> --fingerprint @i=<fp>`.
- That form was run: it renders the Liana descriptor, whose address matches `md address` of the keyed card (`bc1pv7xd8…yy4k3`).

**Tests.** `tests/cli_liana_template_refusal.rs` has 8 tests.

Fixture construction:
- **Pinned** from md-cli 0.19.0 (cf35d61a) plus this toolkit. The keys are md-codec's vendored Liana fixture `preset-kofn-recovery-tr` @0..@2 at `m/48'/0'/0'/3'`.
- **Recipe** in the file header. The cosigner mk1s come from `bundle --descriptor <keyed NUMS>`, and the template stubs from `--md1-form template`.
- **Same card as the toolkit's template:** `TPL_NUMS` is byte-identical to the toolkit's own template md1.

| Test | Asserts |
|---|---|
| restore Liana `--search-address` (true Liana address) | exit 2, the wording, `md descriptor --network`, no "NO MATCH", no md-codec internal text, no `descriptor:` on stdout |
| restore Liana explicit `--cosigner @1=/@2=` | same |
| restore Liana `--expect-wallet-id 468e0a1e…` | same |
| verify-bundle Liana `--search-address` | same; a real CLI test, since the stub cards were mintable, so no engine-boundary fallback was needed |
| restore NUMS control ×3 (search / explicit / expect-id) | exit 0, the completed first receive `bc1p4wtcc…` (search mode also checks wallet-id `9817986e…`) |
| verify-bundle NUMS control | exit 0, `OK (multisig template recomposed)`, `9817986e…` |

**RED first.** On the pre-fix binary the four Liana tests failed exactly as the review reported:

| Liana test | Pre-fix exit |
|---|---|
| search-address | 4 |
| explicit `@N=` | 1 |
| expect-wallet-id | 1 |
| verify-bundle | 4 |

The four controls passed.

**Mutations.** Each was asserted applied, then reverted (restore verified by `diff`):

- **Engine call removed** (`refuse_liana_unspendable(d)?` in `complete_multisig_template`): all four Liana tests go red. The search-address test fails with `✗ NO MATCH` and `left: 4`, the old false verdict. The four NUMS controls stay green. I ran it before and after the wording refactor, with the same result.
- **Keyed-route call removed:** `liana_unspendable_internal_key_refuses_not_nums` still passes, because the classifier backstop refuses. That is by design. The backstop itself was mutation-proven in round 1: folding Liana into Nums gave exit 0 and `tr(50929b74…)`.

**Manual re-checks, by hand:**
- The new wording prints on the Liana search command (run).
- `mnemonic inspect` of the Liana template prints `template: tr(UNSPENDABLE(liana),…)`, as the manual paragraph claims (run).

### Minors

- **M-1 (fixed):** a "Liana unspendable internal key (v0.104.0)" paragraph in the manual's `mnemonic restore` `--md1` section. It covers every mode, verify-bundle, the reason, both recipe forms, the `--from-mk1` ambiguity caveat, and that `inspect` still decodes the card. The CHANGELOG 0.104.0 bullet now covers verify-bundle and the template route, and points to the manual.
- **M-4 (fixed):** `friendly.rs`'s `NetworkRequiredForUnspendable` arm now prints `LIANA_UNSPENDABLE_REFUSAL` instead of its dead, divergent "toolkit bug" text. No toolkit route renders a Liana tree any more; if a future `?` propagates the error, it says what the refusal says. The friendly table test's needle is now `md descriptor --network`.
- **M-2, M-3:** no change. M-2 is already filed as F-650. M-3 is honest at exit 4.

### Full gate on HEAD `d02382d6`

Pinned toolchain 1.85.0. `CARGO_TARGET_DIR=/scratch/code/shibboleth/tk-worktrees/f642-target`.

| Gate | Result |
|---|---|
| `cargo nextest run --locked --workspace --no-fail-fast` (once, captured) | **4057 passed / 0 failed / 20 skipped** (4049 + 8 new) |
| `cargo clippy --locked --all-targets -- -D warnings` | clean |
| `cargo fmt --all --check` | clean |
| Examples golden (regenerated with the HEAD binary, then `cmp`) | byte-identical |
| Manual `make lint` (MD_BIN = md 0.19.0 at cf35d61a) | rc=2. The FAIL set is identical to the 3 known pre-existing. markdownlint: 0 errors. Links: 0 errors. |

### Side observation (not acted on)

`mnemonic inspect` of the `bundle --descriptor`-emitted mk1 cards for a `/3'` path prints `origin_path: m/48'/0'/0'`. Yet the completion engine renders `[fp/48'/0'/0'/3']` from the same cards, and md's seating reads them at `48'/0'/0'/3'`. So this looks like an `inspect` display truncation, not a wrong card. I did not investigate further, and it is out of F-642 scope.

Separately, `mnemonic convert --path` is silently ignored when `--template` is given. That misled my first fixture attempt.

Both are follow-up candidates.
