# F-672 / F-673 independent adversarial review — md-codec 0.48.1 / md-cli 0.20.1

Date 2026-09-23. Reviewer: one opus agent, independent of the implementer.
Nothing was committed, and both worktrees were left clean.

Scope:
- descriptor-mnemonic `39ddced8..f5d052c9`.
- mnemonic-engrave `76a71379..a1a292b9`.
- The engrave worktree HEAD has since moved to `80735588`, another agent's F-671 records commit. `a1a292b9` is its ancestor, and that commit touches nothing reviewed here.

**Result: 0 Critical, 0 Important, 3 Minor, 1 Nit.** I could not construct a key or address rendered wrong on any network, any change to mainnet output beyond the intended verdict notes, or a Core import claim without a measured row.

## Method

I built everything myself from `git archive` exports under `/scratch/code/shibboleth/.tmp/f672rev/`:
- Toolchain 1.85.0, release builds of `md` at `39ddced8` (OLD) and `f5d052c9` (NEW).
- Harness scripts `matrix.py`, `core.py`, `tmpl.py`, `tmpl2.py` and `corpus.sh`, with logs alongside.
- Core: the release binaries under `.tmp/core-releases/`.

I kept my own gates independent of the implementer's `derive.py`. Base58 and the BIP-380 checksum are reimplemented in `matrix.py`.

## 1. F-672: network rendering

**Corpus matrix (`matrix.py`).**
- Inputs: all 68 phrase vectors, 48 of them keyed. Every keyed vector is multi-chunk, with 2 to 24 md1 strings.
- Renders: 4 networks × {multipath, `--chain 0`, `--chain 1`} = 576 renders of `md descriptor`, covering 1,824 extended keys.
- Checks on every render:
  - every key's version bytes are the network's (`0488B21E` on mainnet, `043587CF` on testnet, signet and regtest);
  - the checksum verifies;
  - swapping every key back to the mainnet version and recomputing the checksum gives OLD's mainnet render byte for byte. So pubkey, chain code, depth, child number, parent fingerprint, origins, paths and tree are all unchanged. This covers kind 0 (NUMS), kind 1 (Liana), real internal keys, wsh, and the 24-chunk `keyed_tr_pathological`.
- Also: `md address` on 4 networks × 2 chains × 3 indices, NEW == OLD.
- Result: **0 failures.**
- **Control:** the same harness with OLD as NEW flags all 432 test-network renders (48 × 3 forms × 3 networks), so the gate can fail.

**Core regtest (`core.py`).** Each of the 48 regtest multipath descriptors was imported (`importdescriptors`, active, watch-only), and `deriveaddresses(desc,[0,2])` was compared with `md address --network regtest` for chains 0 and 1.
- Core 31.1: 48/48 import, 48/48 addresses match.
- Core 29.4: 48/48 import, 48/48 addresses match.
- **Control:** OLD's regtest renders import 0/48 on both versions (error -5).

**Template route and D-2 (`tmpl.py`).**
- Shapes: `--template` + `--key` with the D-2 regtest tpubs, for kofn-liana (kind 1), kofn-nums (kind 0), tiered-liana, real internal key, and wsh.
- Lock values: `older` = 1, 26280, 65535.
- Core versions: 29.4, 30.3 and 31.1.
- D-2's exact command reproduces the pinned `D2_EXPECTED` (`#cxm3rd99`).
- All 45 renders import on Core, with addresses equal to md's.

**Decompose recipe.**
- `md decompose --network regtest --emit commands`: route 1 was executed as printed, and re-rendering its card under regtest gives the identical descriptor. That held for 12 of 12 non-NUMS cases.
- Route 2 (`md encode --out policy.md1`, `mk encode --keys` with tpub records) runs, and the resulting policy + mk1 cards seat back through `md descriptor --from-mk1-file --network regtest` to the identical descriptor. This also exercises the seating route.
- Signet and testnet route-1 recipes round-trip for 4 of 4 cases.
- The kofn-nums decompose refusal (`RAW public key`) is pre-existing: OLD refuses the same mainnet input. It is outside this release.

**Other paths I checked for a second copy of the bug.**
- `xpub_from_tlv_bytes` has one caller (`assemble_origin_and_xkey`), and both the chain and multipath builders now take `network`.
- `md descriptor`'s three input routes (positional, `--template`, `--from-mk1`) all reach the `_with_network` render at `cmd/descriptor.rs:204-212`.
- The `disposition.rs` / `matching.rs` / `compose.rs` network-less calls are tests only.
- The coordinator verdict is computed from the parsed `Descriptor`, never the rendered text, so the network cannot reach it.
- No evidence row contains a `tpub` (0 of 548), so F-672 cannot re-key the table.

**Mainnet corpus diff, rebuilt independently (`corpus.sh`).** OLD and NEW were run over 68 vectors × 12 commands = 816 files, with stdout and stderr merged and exit codes appended. The commands were `descriptor` in its multipath, `--chain 0`, `--chain 1` and `--json` forms, `address` in three forms, `decode`, `inspect`, and `decompose --emit all|commands|descriptor`.
- **8 of 816 differ:** the `.desc` and `.descjson` outputs of `keyed_compose_preset_kofn_recovery`, `keyed_compose_tr_two_path_{nums,distinct_fingerprints}` and `keyed_tr_liana_kofn_recovery`.
- Every differing line is exactly this, ×8:
  - removed: `Bitcoin Core 29.4-31.1: unproven (not measured for this policy)`;
  - added: `29.4: imports the multipath form (measured 2026-09-24)` / `30.3: unproven (not measured for this policy)` / `31.1: imports the multipath form (measured 2026-09-24)`.
- The implementer's claim is reproduced.

## 2. F-673: the evidence

**Copies are byte-exact.**
- `coord-compat-e2e/core-composer-tr.out` is byte-equal (`cmp`) to `e2e-live-site-wallets/live/core-mainnet.txt`.
- `core-composer-tr.sh` is byte-equal to `live/core-mainnet.sh`.
- Each of the 8 `.tsv` descriptors equals the `^tr(` line of `live/<name>.descriptor.txt`, which is the text the script imported.

**Every cell maps to a measured line.**
- I checked all 16 new cells' `source` line numbers against the `.out`:
  - lines 2/6/10/14/18/22/26/30 are 29.4 kofn-nums, kofn-liana, tiered-nums, tiered-liana, then the four `-distinct` wallets;
  - lines 35..63 are the same 8 on 31.1.
- Each is `import_ok=True … wallet_addrs=MATCH` under a header that self-reports `v29.4.0` / `v31.1.0`, `chain=main`.
- The skeleton keys pair correctly:
  - kofn = `multi_a(2,@0,@1,@2)` + `pk(@3)`; tiered = `multi_a(2,@0,@1)` + `multi_a(1,@2,@3)`;
  - Nums vs LianaUnspendable per wallet;
  - `[[0,1,2]]` for shared seeds vs `[[0][1][2]]` for `-distinct`.
- 8 keys × 2 versions = 16 cells.
- All cells are `Form::Multipath`, and no chain0/chain1 or 30.3 cell was added. The chain-form verdicts on these vectors are byte-identical before and after, per the corpus diff above.
- The renderer label `md 0.20.0` matches the evidence report (`agent-reports/e2e-live-site-wallets.md:109`).
- The vendor block asserts the self-reported version and exactly 16 rows, so a malformed or truncated `.out` cannot silently drop a row.

**The lock-value abstraction is sound for Core import, and I measured it.** The key abstracts `older(N)` to `older-blocks#1`. I imported the kofn and tiered shapes (both Liana and NUMS), a real internal key and wsh on Core regtest:
- at `older` 1, 26280 and 65535, on 29.4, 30.3 and 31.1;
- at 65536 and 4194303, on 29.4 and 31.1.

Every one imports with matching addresses. 65536 and 4194303 are values md itself refuses to encode, but which still classify as `older-blocks`. That agrees with Core's miniscript: `older` needs only 1 ≤ n < 2³¹, sanity-checks only timelock *mixing* (these shapes have one lock), and tapscript has no ops or script-size limit at this size. Values with bit 22 set classify as `older-units`, a different key, and correctly stay `unproven`.

**30.3 correctly stays unproven.** No vendored row measures 30.3 on these shapes, and the design says a run never spans an unmeasured version. `core_imports_the_composer_tr_wallets_only_where_measured` pins the separate span `"30.3"` as `Unproven{NoEvidence}`.

## 3. `core_verdicts_follow_the_measured_boundary`: still a real assertion

The test asserts:
- the exact span vector `["24.2-28.4","29.4","30.3","31.1"]`;
- `Refuses` with a `<0;1>` class at index 0;
- `Imports` at 1 and 3;
- `Unproven{NoEvidence}` at 2.

This is stricter than before. It used to check only the length and two spans, and it now also pins which verdict kind each span carries. It would redden if 30.3 were wrongly claimed (spans merge), if either measured cell vanished, or if the refusal boundary moved.

**Mutation (its own stated one):** removing `CORE_MULTIPATH` from the 26.0-28.4 rule set (`registry.rs:390`) turns both this test and the new F-673 test red, with spans `["24.2-25.2","26.0-28.4","29.4","30.3","31.1"]`. I restored the file afterwards.

## 4. False passes: F-672 mutation re-applied

I chose a subtler mutation than the implementer's M1. Only the **single-chain** caller in `to_miniscript_descriptor_with_network` passes `Network::Bitcoin`, while the multipath path stays fixed. That reddens 3 of 9 targeted tests:
- `every_rendered_key_carries_the_requested_network_across_the_corpus`;
- `test_networks_render_every_key_as_tpub_matching_the_vector`;
- `every_test_network_renders_every_key_as_tpub`.

So the `--chain` path is independently covered. The baseline was 9/9 green, and I restored the file afterwards.

## Findings

**Minor-1: undocumented behaviour change in public md-codec API.**
- `descriptor_route::descriptor_from_text` / `descriptor_from_chains` round-trip through `to_miniscript_descriptor_with_network`. An all-`tpub` descriptor used to fail that round trip and now succeeds.
- Counterexample:
  - `md shape-key --descriptor "$(md descriptor <keyed_tr_liana_kofn_recovery> --network regtest)"`;
  - 0.20.0 prints `the reconstruction does not round-trip chain 0`;
  - 0.20.1 prints the shape key.
- Mixed-network text (0.20.0's buggy regtest output) is still refused. The change is a fix, but the CHANGELOG mentions only the renderer. Add one line under md-codec 0.48.1 / md-cli 0.20.1.

**Minor-2: the user-visible `measured 2026-09-24` postdates the measurement.** The measurement was taken on 2026-09-23 local time, and the date comes from the engrave committer date in UTC (implementer concern 2). It is not a false claim of import, but the printed date names a day the measurement did not happen. A follow-up on the vendor script's date rule would fix it.

**Minor-3 (pre-existing, not a regression): the network in the decompose refusal message.** It names `testnet` when the user passed `--network regtest` or `signet`, because it prints the NetworkKind (implementer concern 4). File a follow-up.

**Nit: 30.3 is cheaply provable.**
- My regtest runs show Core 30.3 importing kofn-liana, kofn-nums, tiered-liana, a real internal key and wsh, with matching addresses.
- This is not vendored evidence: regtest only, and not all 8 wallets. `unproven` is right for this release.
- A 30.3 row in the committed harness would close the gap.

## Verdict

- F-672: every key carries the requested network on all four networks, across three forms and three input routes.
- Key and chain-code material are unchanged.
- Core 29.4, 30.3 and 31.1 regtest import the output, with addresses equal to md's.
- Mainnet output changes only in the 8 intended verdict notes.
- F-673: the 16 cells are each a measured, byte-traced row, and the lock abstraction holds for Core import.
- The changed test is a real, stronger assertion, and both mutations redden.

ready to ship: yes
