# IMPL — `PLAN_wallet_file_export.md` Phase 1b (`--format bitcoin-core-addresses`)

**Repo:** `mnemonic-toolkit`
**Worktree:** `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/export-phase1b`
**Branch:** `feat/export-addresses-phase1b` (off `master` @ `8a564a08`) — left in place, not merged, not pushed.
**Commit:** `e95e80a8`

Status: **every Acceptance bullet implemented, and the IMPORT gate is GREEN against
two real Bitcoin Core binaries (v27.0 and v31.1).** Three verification commands green.
Four things the plan or the brief got wrong when they met the code are in §8.

---

## 1. The CI-skip decision — there was nothing to invent

**Decision: `#[ignore]`-by-default, CONNECT-ONLY, wired into the EXISTING
`.github/workflows/bitcoind-differential.yml`, which provisions a sha256-pinned
Bitcoin Core v27.0 and runs the gate for real.**

The brief said this repo's CI "will not have `bitcoind`" and warned me off `#[ignore]`.
I found the opposite before the coordinator's correction arrived, and the correction
confirms it: `.github/workflows/bitcoind-differential.yml` already downloads Core v27.0
(sha256 `2a6974c5486f528793c79d42694b5987401e4a43c97f62b1383abf35bcee44a8`), starts an
offline `-chain=main` node, exports four wiring vars, runs `--ignored`, and stops it.
`#[ignore]` is the house pattern here **because a whole named workflow exists to run
those tests**. I built no skip mechanism, so there was nothing to tear out.

What I did instead, following `tests/bitcoind_differential.rs`'s conventions exactly:

- **`tests/bitcoind_addr_import.rs`** — CONNECT-ONLY (never spawns bitcoind), reads
  `MNEMONIC_BIN` / `BITCOINCLI_BIN` / `BITCOIND_DATADIR` / `BITCOIND_RPCPORT`.
  **Wiring UNSET → `panic!`**, wiring partial → `panic!`, wiring set but
  `chain != "main"` → `panic!`. Verified both failure modes turn RED (§5).
- **One new step in the existing job**, so one node provisioning serves both gates.
- **`paths:` extended** so the workflow actually triggers on this work:
  `bitcoind_addr_import.rs`, `wallet_export/bitcoin_core_addresses.rs`,
  `wallet_export/mod.rs`, `cmd/export_wallet.rs`, `descriptor_builder/allow.rs`.
  Without this the workflow would not have fired on the PR at all.
- **The job's display name is now `bitcoind v27.0 gates (end-to-end differential +
  addr-import)`.** It is not a required context, and a reader of a green run has to be
  able to tell which gates ran there. The workflow header documents the second gate.

### The residual visibility question, answered by measurement rather than design

The required contexts (`examples`, `test (ubuntu-latest)`, `clippy`) run
**`cargo test --workspace`**, not nextest. libtest prints the ignore reason:

```
test addr_list_imports_into_bitcoin_core_and_the_descriptor_route_does_not ... ignored, requires a pre-running offline -chain=main bitcoind (wiring env vars)
```

So a reader of a green **required** run sees the gate by name and sees that it did not
run there; a reader of the `bitcoind-differential` run sees that it did. (`cargo nextest`
reports only `20 tests skipped` with no names — worth knowing, but nextest is not what
gates this repo.)

---

## 2. What changed, file by file

**`src/wallet_export/bitcoin_core_addresses.rs`** — NEW (293 lines).
`DEFAULT_ADDRESS_COUNT = 20`, `checksummed()` (BIP-380 over `addr(<addr>)`),
`caveat_label()`, `BitcoinCoreAddressesEmitter`, `format_addr_list()`, 5 unit tests.

**`src/derive_address.rs`** — **this was the real work.** The plan is right that
descriptor→address derivation did not exist on the export path. Added
`derive_chain_addresses(descriptor, chain, count, network, what)`;
`derive_receive_addresses` is now `chain 0` through it. Every historical error string is
reproduced byte-for-byte by threading the message prefix (`what`), so nothing that
`restore` or `verify-bundle` or `build-descriptor` emits moved. A single-path descriptor
asked for chain 1 **errors** rather than silently re-deriving chain 0 — a change list
that is secretly the receive list is worse than none.

**`src/cmd/export_wallet.rs`** — `CliExportFormat::BitcoinCoreAddresses`; `--count`
(default 20); `emission_kind()` (exhaustive, no `_` arm); dispatch in `emit_payload`;
`format_requires_template` arm; `address_count` at both `EmitInputs` sites.

**`src/descriptor_builder/allow.rs`** — new `EmissionKind { Descriptor, AddressList }`
threaded into `export_admission_gate`. It selects **only the refusal's closing
sentence**. 2 new unit tests, one of which pins the Phase-1 `Descriptor` string verbatim.

**`src/cmd/restore.rs`** — `address_count: DEFAULT_ADDRESS_COUNT` at both `EmitInputs`
sites (restore has no `--count` flag). `restore` gains the format for free and stays
ungated, as Phase 1 ruled.

**`src/wallet_export/{mod,coldcard}.rs`** — module wiring; `EmitInputs.address_count`.

**`tests/cli_export_wallet_bitcoin_core_addresses.rs`** — NEW, 22 tests.
**`tests/bitcoind_addr_import.rs`** — NEW, the `#[ignore]`d import gate.
**`tests/fixtures/export_wallet_addresses/`** — the journey's four address lists +
`PROVENANCE.md` carrying source paths and sha256s.

**`tests/cli_export_wallet_allow.rs`** — the Phase-1 12-format gate sweep (was 11).
**`tests/cli_gui_schema.rs`** — format dropdown 11 → 12.

**`CHANGELOG.md`** — a Phase 1b subsection under the existing `[Unreleased]`, opening
with **ADDITIVE; nothing existing changes**.

**`docs/manual/src/40-cli-reference/41-mnemonic.md`** — `--format` row, `--count` row,
and a new *"Watching a wallet Core will not describe"* subsection.
**`docs/manual/.cspell.json`** — `getdescriptorinfo`.

---

## 3. The artifact, as it actually emits

```
$ mnemonic export-wallet --descriptor <rcw wsh> --format bitcoin-core-addresses \
    --count 5 --allow sigless-branch
WARNING: sanity rules OVERRIDDEN by --allow and FIRED: sigless-branch. …
[
  {
    "active": false,
    "desc": "addr(bc1qr6h5gahcaqa8a35p3ts0d2w6qvhmsn7dhunu5xd9kyculcgz3dwqf266zj)#nf7wvmq9",
    "internal": false,
    "label": "imported-descriptor: mnemonic bitcoin-core-addresses FIXED LIST of 5 receive + 5 change addresses (indices 0-4). NO DERIVATION: this file holds addresses, not the wallet descriptor, so Bitcoin Core cannot extend past the exported gap. Re-export with a larger --count before the last index is used.",
    "timestamp": 0
  },
  … 4 more receive …
  {
    "active": false,
    "desc": "addr(bc1q70d7dmaz2s4ur98vewnzyuuct6wzu3jpwhhhn8dz0g5agkevzjaqnn6ecs)#yq4qaer9",
    "internal": true,
    "timestamp": 0
  },
  … 4 more change …
]                                                                       [exit 0]

$ … --format bitcoin-core-addresses          (no flag)
error: export-wallet: this wallet has a spend path that requires no signature
(anyone-can-spend); rerun with --allow sigless-branch after review. The flag permits
EMISSION of this address list. Bitcoin Core does accept addr() entries — what it will
never accept is this wallet's descriptor, on any version through v31.1.  [exit 2]
```

### The three shape decisions, each settled by running Core rather than reasoning

Measured on a live v27.0 node before any code was written:

| probe | Core v27.0 |
| --- | --- |
| `label` on an `internal:false` `addr()` entry | **success: true** |
| `label` on an `internal:true` entry | **success: false** — *"Internal addresses should not have a label"* |
| an UNKNOWN key (`_mnemonic_note`) on an entry, `internal` either way | **success: true** |

1. **Change entries carry `internal: true` and NO label.** The label is not optional
   decoration on a change entry — it is an import failure. Mutation L1 (§5) reproduces it.
2. **The caveat rides the receive entries' `label`.** It is the only Core-documented
   string field available, and it is available on exactly the half where `internal` is
   false. All receive entries share one label string, so Core groups them under it.
3. **No unknown keys, despite v27 tolerating them.** That tolerance is undocumented, and
   a funds-adjacent artifact should not depend on a behaviour no release note promises.
   Recorded at the top of the module so nobody re-litigates it from the measurement alone.

### `--count` default: 20, and why it differs from the sibling surface

`mnemonic addresses --count` defaults to **10**. I did not match it. This artifact's whole
limitation is that it is a window that will not extend, so its default should be at least
the **BIP-44 gap limit (20)** — the gap every wallet uses to decide an account is empty.
`addresses` is an inspection tool, not a watch window. Both defaults are stated in help
text, in the manual, in the module doc, and in the artifact itself. `--count 0` is refused.

---

## 4. Test list and results

**22 integration** + **5 unit (emitter)** + **2 unit (allow)** = **29 run-by-default**,
plus **1 `#[ignore]`d import gate**. Counts machine-derived from
`cargo nextest list --locked --workspace`, not hand-counted.

| test | Acceptance bullet |
| --- | --- |
| `wsh_receive_addresses_equal_the_journeys` | addresses == journey (receive) |
| `wsh_change_addresses_equal_the_journeys` | addresses == journey (change) |
| `tr_receive_and_change_addresses_equal_the_journeys` | both chains, tr form, + `bc1p` anti-vacuity |
| `receive_and_change_are_disjoint` | change is DERIVED, not a copy of receive |
| `every_entry_is_non_ranged_and_inactive` | non-ranged `importdescriptors` entries |
| `change_entries_carry_no_label_because_core_refuses_one` | the label/internal rule + entry ORDER |
| `the_checksum_is_the_one_bitcoin_core_computes` | Core-computed checksum, pinned, runs everywhere |
| `the_artifact_states_its_own_count_and_the_no_derivation_caveat_in_band` | in-band caveat |
| `a_single_path_descriptor_emits_receive_only_and_says_so` | no invented change chain |
| `default_count_is_twenty_per_chain` | `--count` default stated |
| `help_states_the_default_count` | "default 20" is in `--help`, not implied |
| `count_zero_is_refused_rather_than_emitting_an_empty_watch_list` | `--count 0` |
| `count_is_silently_ignored_by_every_other_format` | **additive**: byte-identical output elsewhere |
| `tr_refuses_without_the_flag_and_names_it` | tr gated on `--allow`, named, not generic |
| `wsh_refuses_without_the_flag_too_because_the_gate_is_uniform` | Phase 1's uniform gate |
| `the_refusal_tail_does_not_repeat_the_descriptor_routes_false_sentence` | format-aware tail + descriptor wording UNMOVED |
| `a_sane_wallet_needs_no_flag` | the gate is about the wallet, not the format |
| `the_from_import_json_arm_is_gated_and_emits_like_the_others` | third arm, topology (B) |
| `the_template_arm_reaches_this_format_too` | second arm |
| `nothing_claims_the_descriptor_route_into_core_works` | the surviving constraint |
| `the_artifact_carries_addresses_and_nothing_else` | no xpub / origin / script leaks |
| `restore_reaches_the_format_without_a_flag` | `restore` stays ungated |
| *unit:* `checksum_agrees_with_bitcoin_core` | Core's `getdescriptorinfo` value, pinned |
| *unit:* `the_checksum_actually_depends_on_the_address` | anti-constant guard for the above |
| *unit:* `count_zero_is_refused` | at the emitter level |
| *unit:* `the_caveat_states_the_window_and_how_to_widen_it` | caveat content + ASCII-only |
| *unit:* `the_single_path_caveat_explains_the_missing_change_chain` | the WHY, not just a 0 |
| *unit:* `the_refusal_tail_is_format_aware_and_the_descriptor_wording_is_unmoved` | Phase-1 string verbatim |
| *unit:* `emission_kind_changes_wording_only_never_admission` | the wording axis is not an admission axis |
| *gate:* `addr_list_imports_into_bitcoin_core_and_the_descriptor_route_does_not` | **the IMPORT criterion** |

### The import gate, in five ordered assertions

Ordered so a broken harness cannot fake any of them:

1. **Anti-vacuity BEFORE Core is contacted** — emitted addresses must equal the journey's.
   A drifted derivation fails here rather than importing the wrong wallet successfully.
2. **Checksums come from Core, not from us** — every entry re-derived via
   `getdescriptorinfo`, plus `isrange == false`.
3. **The import** — per-entry `success: true`, no `error` key, entry count matches.
4. **It landed** — `listdescriptors` returns all of them, so a `success: true` that
   stored nothing is caught.
5. **The negative control** — the SAME wallet's DESCRIPTOR route, from the SAME binary,
   on the SAME node, must return `success: false` with *"witnesses without signature
   exist"*. Without this the gate proves Core accepts `addr()` entries, not that it had
   to be `addr()` entries.

**Run against two binaries, both PASS:** Core **v27.0** (the repo's pin) and Core
**v31.1** (the version the plan's ceiling claim rests on).

---

## 5. Mutation testing — 11 mutations, 11 caught

A green suite proves little. Emit-side (9), each applied alone, suite re-run:

| mutation | caught by |
| --- | --- |
| M1 change chain `1 → 0` | 4 tests incl. both journey cross-checks |
| M2 change entries dropped | 8 tests |
| M3 change marked `internal:false` | 8 tests |
| M4 `label` added to change entries | `change_entries_carry_no_label_…` |
| M5 default count `20 → 10` | `default_count_is_twenty_per_chain` |
| M6 checksum over the bare address, not `addr(…)` | `the_checksum_is_the_one_bitcoin_core_computes` |
| M7 `emission_kind` → `Descriptor` for addresses | `the_refusal_tail_does_not_repeat_…` |
| M8 caveat drops `NO DERIVATION` | the in-band test |
| M9 `--count 0` no longer refused | `count_zero_is_refused_…` |

**Live-gate anti-vacuity (2) — the gate itself must be able to fail**, or the acceptance
criterion is theatre:

| mutation | what Core said |
| --- | --- |
| L1 `label` on change entries | `entry 5 was NOT imported: {"error":{"code":-8,"message":"Internal addresses should not have a label"},"success":false}` |
| L2 checksum body corrupted | `BIP-380 checksum disagreement on addr(bc1qr6h…): ours=qf7wvmq9 core=nf7wvmq9` |

**Harness anti-vacuity (2):** wiring UNSET → FAILED; wiring pointed at a dead RPC port →
FAILED. The gate cannot report green without reaching Core.

Restored source re-verified all-pass after every mutation batch.

---

## 6. The three verification commands — all run, all green

```
$ cargo fmt --all --check
(no output)                                                    exit 0

$ cargo clippy --locked --workspace --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s)
diagnostics = 0
(forced a full re-check with `touch src/main.rs`; counted from --message-format short)

$ cargo nextest run --locked --workspace
     Summary [  51.938s] 3959 tests run: 3959 passed, 20 skipped
```

**Reconciliation.** Baseline measured on this branch point before any edit:
**3930 passed, 19 skipped** (`Starting 3930 tests across 221 binaries (19 tests skipped)`).

```
3930 + 29 added = 3959     ✓   (22 integration + 5 emitter unit + 2 allow unit,
                                counted by `cargo nextest list`, not by hand)
  19 +  1 ignored  = 20    ✓   (the import gate)
```

No pre-existing test was deleted. Two were **extended** (both deliberately, both because
a new format must be inside their invariant, not outside it):
`cli_export_wallet_allow.rs::every_format_meets_the_same_gate_before_its_own_verdict`
11 → 12 formats, and `cli_gui_schema.rs` dropdown 11 → 12.

`docs/manual`'s own `make lint` also passes (markdownlint 0, cspell 0, lychee 0,
flag-coverage OK) — see §8.4.

---

## 7. `restore` is unchanged — measured both ways

Run with the Phase-1 binary (`master` @ `8a564a08`) and the Phase-1b binary, same fixture,
same 15 `--md1` chunks:

```
mnemonic restore --md1 …(15 chunks)… --format bitcoin-core
  BEFORE: exit 0, 2694 bytes, sha256 c121fb6ca9723e22489e58b04a82edd3ffccf92d7c13acf0472933c1f95e4b18
  AFTER:  exit 0, 2694 bytes, sha256 c121fb6ca9723e22489e58b04a82edd3ffccf92d7c13acf0472933c1f95e4b18
```

Byte-identical, and re-run after the last edit. The source-level reason it cannot move:
`restore`'s only new field is `address_count`, which no other emitter reads, and
`derive_receive_addresses` delegates to `derive_chain_addresses` with `chain = 0` and the
original `"receive-address"` message prefix — the same branch, the same strings.
`restore_reaches_the_format_without_a_flag` additionally pins that `restore` reaches the
new format with **no** `--allow`, which is the other half of "restore is not gated".

---

## 8. Where the plan met the code

### 8.1 "regtest `importdescriptors`" is wrong — it must be MAINNET-offline

The Acceptance bullet says *"regtest `importdescriptors` against a pinned Core"*. Regtest
is **dead** for this constellation and `bitcoind-differential.yml`'s header already says
why: regtest rejects mainnet xpubs, and every address this wallet derives is a mainnet
`bc1…`. A regtest node could not be shown the journey's addresses at all, so the address
cross-check and the import gate would be testing two different wallets. The gate uses an
offline `-chain=main` node, the same one the differential uses. **The plan's word should
be corrected, not the workflow.** Recorded in the test's module doc so it does not get
"fixed" back.

### 8.2 "wsh works without Phase 1" is a PRE-Phase-1 measurement, and no longer holds

The Measured table says `addr()` for wsh needs no Phase 1 and tr is blocked on it. That
was true of the pre-Phase-1 parser. Post-Phase-1 the gate is **uniform**, so the sigless
`wsh` form now needs `--allow sigless-branch` for this format exactly as `tr` does — the
documented Phase-1 breaking change (F-1), reaching a format the plan wrote before the gate
existed. I did not carve an exemption: the gate is about the **wallet's spend policy**, not
about which file shape is being written, and a wallet with an anyone-can-spend path is
precisely the one an operator should be told about while exporting a watch list for it.
Both refusals are tested.

### 8.3 The Phase-1 refusal's closing sentence is FALSE for this format

*"The flag permits EMISSION of the wallet file — it does not make any wallet application
accept it."* True for every descriptor-route format. False here: these addresses do
import, which is the entire point of the format. Inheriting it would have been a
measurement-falsified purpose sentence — the same class as the plan's own round-1 C1.

Fix: `EmissionKind`, an exhaustive two-variant enum threaded into
`export_admission_gate`, selecting **only the closing sentence**. Two guards, both tested:
`EmissionKind::Descriptor` reproduces the Phase-1 string byte-for-byte, and
`emission_kind_changes_wording_only_never_admission` asserts the gate admits and refuses
identically under both variants across three wrappers — so the wording axis cannot quietly
become an admission axis. `emission_kind()` has no `_` arm, so a thirteenth format must
make the decision rather than inherit it.

### 8.4 Phase 1's manual edit left `docs/manual`'s own lint RED

Measured against `HEAD` (`8a564a08`) with the pinned markdownlint 0.13.0:

```
src/40-cli-reference/41-mnemonic.md:1050 MD012/no-multiple-blanks [Expected: 1; Actual: 2]
```

One pre-existing MD012 in the block Phase 1 added, so `manual.yml` — which triggers on
`docs/manual/**` and which Phase 1's own change touches — was already failing. Fixed in
this commit alongside my own two findings (a second MD012 and one MD040 unlabelled fence)
and one cspell word. `make lint` now reports **OK**. Flagged rather than absorbed because
it means a Phase-1 gate went un-run.

*(`make verify-examples` reports 13 drifted transcripts in my worktree; every one invokes
a sibling CLI — `md` / `ms` / `mk` — whose repo is not checked out at the expected path
here. Neither `export-wallet` transcript drifted. Environmental, not a finding.)*

### 8.5 Two things the plan understated, neither blocking

- **`derive_receive_addresses` already existed** (`derive_address.rs:75`) and is
  taproot-safe. The plan's "descriptor→address derivation does not exist in the export
  path" is right about the **export path** but the receive half of the primitive was one
  `chain` parameter away. The genuinely new work is the change chain, the single-path
  refusal, the `addr()` checksum (rust-miniscript has no `Descriptor::Addr`, so nothing
  in-tree could render one), and the wire shape.
- **`--count` had to be ruled on beyond "state the default".** `--count 0`, a single-path
  descriptor, and a >2-branch multipath descriptor are all reachable and all needed a
  decision. Refuse, receive-only-and-say-so, and refuse, respectively.

---

## 9. Not done (out of scope, flagged)

- **No Go port.** The Rust-primary rule makes this repo the leading side and the fork has
  no `export-wallet` counterpart. This phase changes no normative codec behaviour — it
  adds an output format — so nothing is pending downstream.
- **No version bump.** The CHANGELOG entry rides the existing `[Unreleased]` section.
- **No network-agreement check on the `--descriptor` arm.** `--network testnet` with a
  mainnet-xpub descriptor renders `tb1…` addresses for the same pubkeys. That asymmetry
  is pre-existing (the network cross-check loop covers resolved `--slot`s only, and every
  other format ignores it), the default path is correct, and closing it is a behaviour
  change to a shipped surface. Worth a follow-up; not smuggled into an additive phase.
- **No `[profile.test] opt-level = 2`.** Same disposition as Phase 1: a repo-wide config
  change belongs in its own commit.
- **Not merged, not pushed.** Worktree and branch left in place at `e95e80a8`.
- **Both bitcoind nodes I started (ports 29551/29552) are stopped and their datadirs
  removed.** No pre-existing bitcoind was touched; `/tmp` is back at its starting 81%.
