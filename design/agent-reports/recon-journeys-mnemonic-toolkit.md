# Recon: round-trip journeys that EXIST in `mnemonic-toolkit`

Scope: read-only inventory of `/scratch/code/shibboleth/mnemonic-toolkit` only.
Governed by `/scratch/code/shibboleth/mnemonic-engrave/design/DRAFT_round_trip_journey_definition.md`
(read in full before this recon; §3.1's first T4 bullet is struck/superseded by
the §8 ruling — noted, not re-argued; all four §8 rulings inherited).

## What I actually ran

`ls`/`find` over the repo tree (top level, `crates/`, `docs/technical-manual/`,
`docs/manual-gui/`, `.github/workflows/`, `scripts/`, `ci/`); `grep -rIl` for
`round.trip|roundtrip` across `crates/`, `fuzz/`, `scripts/`, `docs/`, `design/`
(217 hits, mostly source doc-comments, narrowed by hand below); `git grep -il
journey` repo-wide (a second, content-based method after the filename-based
`find -iname '*journey*'` returned only one irrelevant hit — confirming the
filename search under-reported: journeys exist named `20-j1-single-sig.md`,
not `*journey*`). Full `Read` of 9 test files end-to-end: `cli_convert_round_
trips.rs`, `cli_import_wallet_roundtrip.rs`, `cli_standalone_bijections.rs`,
`cli_wallet_cross_format_convergence.rs`, `prop_backup_restore_roundtrip.rs`,
`bitcoind_differential.rs` (1563 lines, in two paged reads), `src/wallet_
import/roundtrip.rs` (2111 lines, paged), `cli_cycleA_phase2_funds_proof.rs`,
`lib_slip39_roundtrip.rs`; partial reads (head/grep) of `prop_subset_search_
roundtrip.rs`, `prop_template_completion_roundtrip.rs`, the 4 `docs/technical-
manual/examples/examples/*-api-roundtrip.rs` cargo examples, `crates/wc-codec/
src/lib.rs` + `tests/pipeline.rs`, `docs/manual-gui/tutorial/{20,30,40,50,60}-
j*.md`. Read `.github/workflows/bitcoind-differential.yml`, `technical-
manual.yml`, `docs/technical-manual/tests/verify-examples.sh`, and the 4
committed golden `.out` transcript files for the api-roundtrip examples.
Ran `gh run list --workflow=bitcoind-differential.yml --limit 5` (measured:
5/5 `success`, `schedule`-triggered, 2026-08-15 .. 2026-08-19 — the gate
demonstrably executes daily, not a hypothesis). Ran `cargo test -p mnemonic-
toolkit --test cli_cycleA_phase2_funds_proof --test prop_backup_restore_
roundtrip --test bitcoind_differential -- --list` (measured test counts: 9 /
5 / 13 = 27 named tests, pasted verbatim below). Ran the non-`#[ignore]`,
non-multi-minute-property cells of those three files (`cargo test ... --
--skip bitcoind_` and `--skip backup_restore_roundtrip --skip tr_taproot_
roundtrip`): **21 of 21 passed**, measured just now, not described from a doc
comment. Did NOT run the full 64-case `backup_restore_roundtrip` proptest
(read-only recon budget; each case spawns 3 CLI processes) — flagged UNKNOWN
below, not assumed green. Did not run anything requiring network or a live
bitcoind locally (the differential's own gate execution is evidenced via the
CI run list instead).

---

## Part A — journeys that satisfy the full §7 schema (or come closest)

### J1 — `bundle_descriptor_multipath_restores_to_true_bip84_first_receive`

| field | value |
|---|---|
| kind | generative (origin = BIP-39 test-vector phrase) |
| tier | T2 (2 separate `mnemonic` CLI process invocations, `assert_cmd`, real exit codes) |
| origin artifact | fixed BIP-39 phrase `abandon×11 about` (`TREZOR_12`) → derived `m/84'/0'/0'` xpub+fingerprint (derived in-test via the `bitcoin`/`bip39` crates directly, NOT via the toolkit) → concrete `wpkh(...<0;1>/*)` descriptor |
| invocations | `mnemonic bundle --descriptor <concrete> --network mainnet --json --no-engraving-card` → `mnemonic restore --md1 <chunks...> --network mainnet` |
| structural assertion | restore's printed descriptor contains the PRESERVED `<0;1>/*` multipath use-site (not collapsed to bare `/*`) |
| functional assertion | restore's reported first-receive address == the independently-derived BIP-84 oracle `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`, AND explicitly `!contains` the known-wrong collapsed address `bc1q8vph849lf3e9rrj85hsxrzlv949rtahe794k6p` (that wrong value is itself independently re-derived and cross-checked in the sibling test `collapsed_wrong_oracle_value_independently_confirmed`, not trusted from the SPEC prose) |
| one command | `cargo test -p mnemonic-toolkit --test cli_cycleA_phase2_funds_proof bundle_descriptor_multipath_restores_to_true_bip84_first_receive` — **measured: compiles, passes** |
| stated non-coverage | single-sig only; mainnet only; T2 ceiling (no device/emulator); file itself frames this as a regression lock for one specific collapse bug, not general coverage |

File: `crates/mnemonic-toolkit/tests/cli_cycleA_phase2_funds_proof.rs:297-340`.
This is the single cleanest journey found: both assertions present, functional
oracle independently sourced (not toolkit-derived), negative check included.

### J2 — `template_completion_anti_vacuity_leg` / `subset_search_completion_anti_vacuity_leg`

| field | value |
|---|---|
| kind | generative (fixed BIP-39 phrases `SEED_A`/`SEED_B`/`SEED_C`) |
| tier | T2, default-CI (NOT `#[ignore]`) |
| origin artifact | fixed seed phrases → keyless md1 TEMPLATE + per-cosigner mk1 (`bundle --md1-form=template`/`=policy`) |
| invocations | `mnemonic bundle --md1-form template ...` + `mnemonic bundle --md1-form policy ...` → `mnemonic restore --md1 <template> --from phrase=<own> [--account N \| --own-account-max K] --cosigner <mk1s> --expect-wallet-id <id> --json` |
| structural assertion | none beyond the wallet-id match enforced by `--expect-wallet-id` (a WalletPolicyId equality, but not separately asserted as its own field in this cell) |
| functional assertion | completed `first_addresses` == an INDEPENDENT `rust-miniscript` `derive_receive`/`derive_at_index` of the ORIGINAL concrete descriptor; `subset_search_...` additionally asserts the result DIFFERS from the wrong-account (own@0) golden, proving the search resolved the true non-zero account rather than passing vacuously |
| one command | `cargo test -p mnemonic-toolkit --test bitcoind_differential template_completion_anti_vacuity_leg` / `subset_search_completion_anti_vacuity_leg` — **measured: pass** |
| stated non-coverage | no external (non-rust-miniscript) oracle in this leg — same-ecosystem risk the file's own header names explicitly ("STRESS-A's O3 is a same-ecosystem rust-miniscript oracle... Bitcoin Core is the only oracle outside that ecosystem") |

File: `crates/mnemonic-toolkit/tests/bitcoind_differential.rs:790-810, 1059-1095`.

### J3 — `bitcoind_end_to_end_differential` / `bitcoind_template_completion_differential` / `bitcoind_subset_search_completion_differential`

| field | value |
|---|---|
| kind | generative |
| tier | does not cleanly fit DRAFT's 4-tier scheme — T2 CLI pipeline PLUS an external process oracle (Bitcoin Core v27.0, pinned + sha256-verified, CI-provisioned). **Finding: an oracle-differential shape exists in this repo and the DRAFT names no tier for it.** |
| origin artifact | 9-shape descriptor corpus (wpkh, pkh, wsh-multi/sortedmulti 2-of-3, sh-wsh, timelocked, thresh, tr-NUMS multi_a/sortedmulti_a — the last is a toolkit-fork-only shape) built from fixed literal xpubs; the template-completion legs use fixed seed phrases |
| invocations | `mnemonic bundle --descriptor ...` → `mnemonic restore --md1 ...` → (out-of-repo) `bitcoin-cli -chain=main deriveaddresses <desc> [0,N]` / `getdescriptorinfo` |
| structural assertion | BIP-380 checksum: Core's `getdescriptorinfo` checksum == the toolkit's reconstructed-descriptor checksum |
| functional assertion | toolkit reported addresses == Core `deriveaddresses` on BOTH the reconstructed AND the original descriptor, chain-0 (receive) AND **chain-1 (change)** — the only journey found in this repo that explicitly checks the change branch, satisfying DRAFT §4's "change addresses are not optional" clause |
| one command | in CI: `.github/workflows/bitcoind-differential.yml` provisions bitcoind then runs `cargo test -p mnemonic-toolkit --test bitcoind_differential -- --ignored --nocapture`. **No single local command** — a local operator must hand-provision bitcoind + export 3 env vars first; the workflow is the only "one command" surface |
| stated non-coverage | mainnet-offline bitcoind only; corpus explicitly excludes shapes `restore` refuses (documented in-file); no T3/emulator/device, no T4/engraving |

File: `crates/mnemonic-toolkit/tests/bitcoind_differential.rs:343-477, 812-875, 1097-1161`.
**Gate-execution check (§5 "every gate must have executed"): PASSED, machine-
verified.** `gh run list --workflow=bitcoind-differential.yml --limit 5`
returned 5/5 `success`, `schedule`-triggered (daily cron 05:17 UTC),
2026-08-15 through 2026-08-19 — this is the best-evidenced journey in the
inventory for "the gate is not a hypothesis."

### J4 — `smoke_handpicked_policies` (+ sibling property `backup_restore_roundtrip`, UNKNOWN-at-full-scale)

| field | value |
|---|---|
| kind | **ambiguous — does not fit either of DRAFT §2's two kinds.** Origin is neither entropy/a seed (generative) nor an artifact already in hand (custodial): it is a SYNTHESIZED wallet-policy IR spec fed to `build-descriptor`, keyed off a small fixed pool of 5 LITERAL xpub strings (not derived from any phrase). **Finding: this repo's single richest round-trip harness doesn't fit the DRAFT's taxonomy.** |
| tier | T2 (3 sequential CLI invocations) |
| origin artifact | one of 10 typed-template policy schemas (`build_policy(schema, seed=42)`), rendered via `mnemonic build-descriptor --spec -` |
| invocations | `mnemonic build-descriptor --spec - --network mainnet --format descriptor` → `mnemonic bundle --descriptor <desc> --network mainnet --json --no-engraving-card` → `mnemonic restore --md1 ... --count 2 --json` |
| structural assertion | O1: original vs. reconstructed descriptor, `rust-miniscript`-normalized-AST-equal modulo key identity; O2: md1 fixed-point (re-bundling the reconstruction reproduces byte-identical md1) |
| functional assertion | O3: chain-0 addresses, INDEPENDENTLY derived via `rust-miniscript` from the ORIGINAL descriptor, == restore's reported addresses |
| one command | `cargo test -p mnemonic-toolkit --test prop_backup_restore_roundtrip smoke_handpicked_policies` — **measured: passes**. Full random property: `... backup_restore_roundtrip` — **NOT run (UNKNOWN at full 64-case scale)**; its sibling oracle-self-test cells (below) did pass |
| stated non-coverage | O3's oracle is same-ecosystem `rust-miniscript` (same admission as J2/J3); the file explicitly says Bitcoin Core is the only outside-ecosystem oracle it has |

File: `crates/mnemonic-toolkit/tests/prop_backup_restore_roundtrip.rs:609-625`
(smoke), `:404-443` (the property itself).

**Notable positive discipline (best-in-repo):** this file carries 5 permanent
"oracle self-test" cells (`oracle1_rejects_dropped_timelock`,
`oracle1_rejects_multi_sortedmulti_swap`, `oracle1_rejects_masked_timelock_
value`, `oracle1_accepts_keyless_equivalent_redepth`, `oracle3_rejects_wrong_
descriptor_address`) that PROVE the O1/O3 oracles would catch known-bad
mutations, directly defending against the §5 "asserts against a value the
journey itself produced" trap. **Measured: all 5 pass, just now.**

**Finding on `backup_restore_roundtrip` itself (not `smoke_handpicked_
policies`):** it samples `(schema, seed)` via `proptest::any::<u64>()` — a
NEW random seed every run — and sets `failure_persistence: None`, so a
counterexample is not even saved for automatic replay (only printed). This
is the one candidate journey in the repo whose inputs demonstrably change
per run, in tension with DRAFT §8 ruling #2's premise that a journey's origin
should be pinned ("a journey whose inputs change per run cannot serve as a
regression test"). `smoke_handpicked_policies` is the fixed-seed (`seed=42`)
deterministic sibling and is the safer citation for "the one command."

### J5 — `tr_taproot_smoke_both_variants`

Same file/pipeline as J4, fixed shapes (`multi_a`/`sortedmulti_a`, not random)
— structural (O1) + functional (O3, asserts `bc1p`-prefix P2TR address) +
md1 fixed-point (O2). **Measured: passes** (part of the 11-test batch run
above). One command: `cargo test -p mnemonic-toolkit --test prop_backup_
restore_roundtrip tr_taproot_smoke_both_variants`.

---

## Part B — journeys with a missing §7 field (findings, not journeys, per §7's own rule)

> "An existing path that lacks any field is A FINDING, not a journey."

### F1 — B1–B6 standalone bijections: no functional assertion

File: `crates/mnemonic-toolkit/tests/cli_standalone_bijections.rs` (6 tests:
`b1_xpub_to_mk1_to_xpub_singlesig`, `b2_..._reverse_edges...`,
`b3_..._multisig_per_cosigner`, `b4/b5/b6_descriptor_to_md1_to_descriptor_*`).
Kind: generative (BIP-39 phrases). Tier: **hybrid, not one value** — B1–B3
(mk1 leg) go through the CLI (`mnemonic convert --from mk1=...`); B4–B6 (md1
leg) call `md_codec::chunk::reassemble`/`split` **in-process**, no CLI spawn
— a T1/T2 mix inside one file. Structural: xpub==xpub or md1==md1,
byte-identical. **No cell in this file derives or compares an address —
purely structural bijections.** One command: `cargo test -p mnemonic-toolkit
--test cli_standalone_bijections`.

### F2 — `cli_import_wallet_roundtrip.rs` + `cli_wallet_cross_format_convergence.rs`: no functional assertion anywhere

Combined this is the **second-largest round-trip-shaped surface in the repo**
(≈20 test cells across both files: `core_bundle_roundtrip_*` ×6,
`c1`/`c2`/`c3`/`c4`/`c_neg` convergence, `h_hop_idempotence_*` ×2,
`concrete_vs_atn_descriptor_converge_md1_mk1`, `concrete_duplicate_cosigner_
rejected_bip388`). Kind: neither generative nor custodial per DRAFT §2 —
origin is CONCRETE XPUBS supplied as raw `--slot @N.xpub=...` CLI flags (no
seed, no pre-existing card). **Finding: a third origin shape (export-
generate-from-slots) exists here and isn't named by DRAFT §2's two kinds.**
Tier: T2. Structural: decoded key-material equality (xpub/fingerprint/path
triples, threshold, cosigner count, md1 tree tag) across formats, or
`roundtrip.semantic_match == true` for one cell. **Functional: none.** No
cell in either file derives or compares a receive/change ADDRESS. Per DRAFT
§4 ("a tool can be made to accept input it previously rejected while
silently dropping part of it — the structural check is what catches that,"
but both are explicitly required together): a same-xpub-but-wrong-
derivation-path bug would not be caught anywhere in these ~20 cells. These
are genuinely well-built metamorphic/convergence tests (`c_neg` anti-vacuity
cell is good practice) — they are simply half a journey each by the DRAFT's
bar. One command per file: `cargo test -p mnemonic-toolkit --test cli_
import_wallet_roundtrip` / `--test cli_wallet_cross_format_convergence`.

### F3 — `cli_convert_round_trips.rs`: no functional assertion (scope-ambiguous)

3 tests: `round_trip_phrase_to_entropy_to_phrase`, `round_trip_entropy_to_
ms1_to_entropy`, `round_trip_phrase_to_ms1_to_phrase_via_entropy_
intermediate`. Kind: generative (fixed `TREZOR_24`). Tier: T2 (2 sequential
`mnemonic convert` invocations). Structural: byte-identical phrase/entropy/
ms1. Functional: none — `convert` never touches an xpub, address, or
wallet-id; there is no derivation path or network flag at this layer. Noted
as a finding per §7's literal rule, but flagged with a caveat: it's an open
question whether DRAFT §4's functional-equality clause is even meant to
apply to a pure codec-transcoding command with no wallet concept — this repo
gives no ruling either way.

### F4 — wc-codec `tests/pipeline.rs` / `lib_slip39_roundtrip.rs`: below the wallet layer

`crates/wc-codec/tests/pipeline.rs` (word-card value-engine P4 integration
KATs + proptest) and `crates/mnemonic-toolkit/tests/lib_slip39_roundtrip.rs`
(SLIP-39 split/combine, 200-trial deterministic matrix, `SEED_BASE` fixed —
this one genuinely IS a fixed-seed generative journey per §8 ruling #2,
unlike J4's full property). Both are **T1** (in-process library calls, no
CLI, no process spawn). Both assert structural byte-identity only (recovered
payload/secret == original) — **no functional assertion**, but this looks
like a property of the layer (payload bytes / SLIP-39 shares, not yet an
xpub or address) rather than an oversight; noted, not asserted as a defect.
One command: `cargo test -p wc-codec --test pipeline` /
`cargo test -p mnemonic-toolkit --test lib_slip39_roundtrip roundtrip_
default_matrix_200_trials`.

---

## Part C — mislabeled: named "round-trip," is not one (strong finding)

### F5 — the four `docs/technical-manual/examples/examples/*-api-roundtrip.rs` cargo examples

Machine-verified via `docs/technical-manual/tests/verify-examples.sh`
(`make verify-examples`), wired into `.github/workflows/technical-manual.yml`
on push/PR to `crates/mnemonic-toolkit/{src,tests}/**` and `docs/technical-
manual/**`, plus `tech-manual-v*` tags — **this gate does run regularly**,
which makes the finding worse, not better: it inspires false confidence.

- **`md-codec-api-roundtrip.rs`, `mk-codec-api-roundtrip.rs`, `ms-codec-
  api-roundtrip.rs`** each call `encode` then `decode` in-process on a
  hand-built struct/payload, but **contain zero `assert_eq!`/`assert!`**
  comparing the decoded value back to the original input. Each `main()`
  only `println!`s a summary (`"decode ok: n=1 tag=Wpkh"`,
  `"decode ok: stubs=1 path=84'/0'/0'"`, `"decode ok: tag=entr kind=Entr
  bytes=16"`) and unconditionally returns `Ok(())`. **The only verification
  is an out-of-process CI step diffing that println output against a
  COMMITTED golden `.out` file** — confirmed by reading `docs/technical-
  manual/transcripts/{md,mk,ms}-codec-api-roundtrip.out` verbatim (they
  contain exactly those three lines each, nothing more). This is precisely
  the DRAFT §5 anti-requirement: *"must not assert against a value the
  journey itself produced with no independent source... a snapshot test
  blesses whatever the code did, bug included."* If encode/decode silently
  dropped or mutated a field and the golden was captured post-bug, the
  transcript-diff would keep passing forever — the Rust code itself asserts
  nothing.
- **`mnemonic-toolkit-api-roundtrip.rs` is worse: it never calls `encode` at
  all.** It `serde_json::from_str`s a single HARDCODED JSON literal
  (`let json_fixture = r#"{"schema_version":"4",...}"#`) and prints field
  values. **Despite the filename, there is no round trip in this file
  whatsoever.** It is a fixture-deserialize smoke test, mislabeled.

One command per example: `cargo run --manifest-path docs/technical-manual/
examples/Cargo.toml --example md-codec-api-roundtrip` (etc.); CI verification
via `make -C docs/technical-manual verify-examples MNEMONIC_BIN=... MD_BIN=md
MS_BIN=ms MK_BIN=mk EXAMPLES_DIR=...`.

---

## Part D — a distinct, uncatalogued journey class found (not audited — out of budget)

`git grep -il journey` (a second, content-based method, run after the
filename-based `find -iname '*journey*'` returned only one irrelevant hit)
surfaced `docs/manual-gui/tutorial/{20-j1-single-sig,30-j2-multisig,
40-j3-degrading-vault,50-j4-taproot-twin,60-j5-watch-only}.md` — a GUI-driven
worked-tutorial with its own "Journey 1..5" numbering, machine-verified via
`verify-tutorial-figures`/`verify-tutorial-transcripts` (named in
`docs/manual-gui/Makefile`'s `lint` target; the underlying script/rule was
not traced within this recon's budget). `grep -c` for `restore|export-
wallet` inside each chapter: **J1 = 0 hits (encode-only)**; J2 = 17 hits;
J3 = 19 hits; J4 = 23 hits; J5 = 17 hits — J2 through J5 appear to close an
encode→decode loop (`bundle`/`export-wallet` → `restore`) through the GUI
form layer, which J1 does not. **This was NOT fully cataloged against the
§7 schema** (would require reading ~300-450 lines per chapter plus tracing
the transcript-verification mechanism) and should be the next pass's first
target — it is a materially different journey SHAPE (GUI form → screenshot/
transcript-verified, not CLI-process-spawned) than everything in Parts A-C.

---

## §5 anti-requirement checks performed

- **Reads an intermediate nothing writes:** not observed in any journey
  read. `tests/fixtures/wallet_import/*` reads are of static, committed
  fixture files (custodial origin artifacts), not journey-written
  intermediates — appropriate, not a violation.
- **Asserts against a self-produced value, no independent source:** **found**
  — see F5 above (the golden-transcript mechanism for 3 of the 4 api-
  roundtrip examples). By contrast, J1/J2/J4's functional oracles are
  independently re-derived via the `bitcoin`/`bip39`/`rust-miniscript`
  crates directly, not read back from the toolkit's own prior output — the
  correct pattern, done well in those files.
- **A skipped step that passes instead of failing:** **found, with the
  authors' own caveat on record.** `bitcoind_end_to_end_differential` (and
  its two siblings) do `let Some(w) = read_wiring() else { eprintln!(...);
  return; }` when all 3 wiring env vars are unset — a bare `return` from a
  `#[test]` fn is a PASS, not a failure. The file's own doc-comment
  distinguishes this deliberately from the partially-set case (`panic!`).
  Because the test is also `#[ignore]`-gated, ambient `cargo test` never
  hits this path; but an operator who runs `cargo test -- --ignored`
  without also exporting the wiring vars gets a **false green**, not the
  loud failure DRAFT §5 asks for. Reported as a finding per the brief's
  instruction to report violations seen, not to adjudicate the tradeoff.
- **A gate that has never executed:** checked positively for the one
  candidate most likely to be a hypothesis (bitcoind-differential.yml,
  since it needs external provisioning) — **it has executed, 5/5 success,
  daily, machine-verified via `gh run list`.** No T3 (emulator) or T4
  (engraving) journeys exist anywhere in this repo, which is expected/
  correct (those tiers belong to the sibling `mnemonic-engrave` repo) —
  not itself a finding.
- **Empty output is not proof of absence:** applied explicitly once — the
  filename-based `find -iname '*journey*'` returned only one irrelevant
  hit, which on its own would have wrongly suggested "no journey concept
  exists in this repo." A second method (`git grep -il journey`, content
  not filename) surfaced the real 28-file hit set including the Part D
  tutorial class. Recorded so the negative isn't taken at face value
  anywhere else in this report either.

## Measured test-name/count evidence (not hand-counted)

```
$ cargo test -p mnemonic-toolkit --test cli_cycleA_phase2_funds_proof \
    --test prop_backup_restore_roundtrip --test bitcoind_differential -- --list
bitcoind_differential.rs:            9 tests
cli_cycleA_phase2_funds_proof.rs:    5 tests
prop_backup_restore_roundtrip.rs:   13 tests
```
Non-`#[ignore]`, non-full-property cells of those 3 files run just now:
**21 of 21 `ok`** (5 + 5 in one batch, 11 in a second — see transcript in
this recon's tool history; not re-pasted here to keep this report short).

```
$ gh run list --workflow=bitcoind-differential.yml --limit 5 \
    --json databaseId,status,conclusion,createdAt,event
5/5 conclusion=success, event=schedule, 2026-08-15 .. 2026-08-19
```

## Known blind spot (per §8 ruling #3, restated as required)

This is a single-repo sweep. It cannot see gaps *between* repos (e.g.
whether a card `mnemonic-toolkit` `bundle` emits is ever actually engraved
and read back by `mnemonic-engrave`/SeedHammer, or whether the two repos'
notions of a "journey" — this repo's CLI-process test journeys vs. the
sibling's device/emulator/engraving journeys — compose into anything a real
operator could run as one command). That composition gap is exactly where
round trips break, per the ruling, and this recon does not close it.
