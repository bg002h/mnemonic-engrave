# Review r2 (fold reviewer): mnemonic-engrave PR #68 (`claude/project-thread-utm1aa`, head 30f8e69)

Scope: `git diff 67c3f1c..HEAD`. That is 094979e E3, 4790e74 E4, 683bac4 (fold of r1 I-1) and 30f8e69 (r1 report and FOLLOWUPS). E3 and E4 are reviewed in full. I edited, committed and pushed nothing. A temporary worktree at 67c3f1c was used to build the old `me` for byte comparisons. It has been removed, and `target/debug/me` was restored to the HEAD build.

**Verdict: 0 Critical, 1 Important, 3 Minor, 4 Nit.** The I-1 fold is complete and the r1 fold is faithful. E4 leaves the container bytes unchanged, as it claims. E3's predicate is correct. But E3 silently falsified one row of the normative shared vector file, and the test that pins that column still passes because it uses a stale predicate.

## Evidence run

- `CARGO_INCREMENTAL=0 cargo test --locked -j 2 -p mnemonic-engrave --no-fail-fast`, captured once: 846 ok. Every failure is one of the known environmental failures:
  - `cross_lang` ×2 and `preview_cross_lang`: no seedhammer submodule;
  - `history_purge` ×3: no zsh or fish;
  - `sysw_cli::pack_does_not_print_a_digest_for_a_write_that_failed`: the run is as root;
  - `seal::v2_tests::v2_titles_refusals`: the 200 ms timing assert.

  All new rows pass: `row_md1_origin_mismatch`, `bh1125_e3_*`, the three `bh1125_e4_*` rows, the vocabulary/schema/sha rows, and `pack_pinned_mk1_warns_chunk_set_id_mismatch_on_stderr` (exactly one warning).
- **E3 is RED on the old pin:**
  - The 67c3f1c binary packs `wpkh([73c5da0a/{84h/0h/7h | 84h/0h/0h/5h | 84h}]E3_XPUB/<0;1>/*)` under `--as md1` (exit 0). HEAD refuses all three (exit 3).
- **E3 edge cases.** I forged xpubs with stdlib Python by re-encoding E3_XPUB's depth, child-number and chain-code bytes (scratchpad `r2/forge.py`). The HEAD binary, under `--as md1`:

  | input | result |
  |---|---|
  | master xpub (depth 0), no origin | refused: "depth 0, but its origin `84h/0h/0h` (implied by the script …) has 3 steps". Correct, because before E3 md1 recorded 84h/0h/0h for a master key. |
  | `[fp]master` | refused earlier by the existing fingerprint-no-path row |
  | BIP-48 `wsh(sortedmulti(2,[…/48h/0h/0h/2h]D4,[…/48h/0h/0h/2h]D4'))`, depth 4, child 2h | packs |
  | `'` notation `84'/0'/0'` on depth 3, child 0h | packs |
  | depth 3, child 0 (unhardened) vs `84h/0h/0h` | refused, "child number 0 … ends in 0h": the hardened bit is compared |
  | `84h/0h/0` vs child 0 | packs |
  | depth 5, five-step origin | packs |
  | origin-less depth-3 key, child 1h | refused, implied-origin wording |
  | origin-less depth-3 key, child 0h | packs |
  | bare-key promotion | packs |
  | `multi` with a mismatch, `--as md1` or `--as` omitted | exit 3, with the `multi` remedy |
  | `sortedmulti` or `wpkh` with a mismatch, `--as` omitted | exit 2, the choice block |

- Multipath is not involved: the predicate reads only `origin`, `depth` and `child_number`, never `children`.
- No key material leaks. The text names the key by the existing truncated `key_name` (`[fp/path]xpub6CUGRUonZS…3fDVmz`), which is public. Nothing from secret input reaches it.
- **E4 container bytes.** I compared the old binary (67c3f1c) with HEAD using `cmp` on `--no-now` output. All of these are byte-identical, with the same exit code and the same count of chunk-set-id warnings (1/1 where the mk1 pair is present, else 0/0):
  - partial md1 A+B, with `--no-passphrase`, with no flag, and with `--passphrase-words 4`;
  - mnemonic + partial md1 with `--no-passphrase`;
  - confirmed md1;
  - the pinned-mismatch mk1 pair;
  - the mk1 pair + partial md1;
  - a single mk1 chunk;
  - md1 + `text:` + mk1.

  Sealed (mnemonic + partial md1, `--passphrase-words 4`): both are SEALED, 296 bytes, with the same digest line. The stderr diff is only the added "Not sealed: records 1 and 2 …" sentence.

## 1. I-1 (no-echo spec text): fixed

- `SPEC_encrypted_payload_delivery.md:2408-2412` (the §14 rule) carries the new text. `:2605` (N-public-titles) and `:2738` (`v1_open_treats_text_records_as_records`) carry it too. Each is byte-equal to `classify.rs:50-53` and to the asserts at `seal/v2_tests.rs:851-856, 864-869`, and each has an E1 amendment note.
- `SPEC_sh2_demo_payload.md:221-234` and `demo/sh2/build-payload.sh:19-23` are updated.
- What remains is not a defect:
  - `SPEC_systemwide_payloads.md:1876` ("`me seal` still answers "unrecognized HRP 'mr'"") is still true, because `mr` is whitelisted.
  - `FIX_me_cli_arg_contract.md:43` (`'text:7'`) records "master result" on 2026-11-22. It is historical.
  - `IMPLEMENTATION_PLAN_me_converter.md:312` and `FOLLOWUPS.md:11090, :12365` are historical.
- No normative text still requires an echo.

## 2. Fold faithfulness: faithful

- 30f8e69 files exactly M-1..M-6 and N-1..N-5 as `status=open` entries with correct grades and anchors, plus one extra N entry (`bh1125-e4-sealing-line-quotes`).
- The saved report `design/agent-reports/bughunt-2026-11-25-mnemonic-engrave-review-r1.md` is byte-identical to the scratchpad original.
- 683bac4 touches only the three spec/demo sites above. Neither commit changes code.

## Findings

### Important

**I-1 (r2). E3 made the normative vector row `accepted/sh-wsh-sortedmulti-16-keys` false, and the column's test cannot see it.**

- `crates/me-cli/testdata/descriptor_seam_vectors.json` (shared with the seedhammer fork) has `"md1_admits": true` on that row. SPEC_descriptor_input §7 (`:1703-1711`) defines the field as "whether `me sysw pack --as md1` carries this row's input".
- The row's 16 keys are depth-5 xpubs ("16 unhardened children of the dc567276 fixture key"), each declared with the 4-step origin `48h/0h/0h/2h`. That is exactly E3's depth clause.
- Measured: the 67c3f1c binary packs it with `--as md1` (exit 0). HEAD refuses it (exit 3, `md1-origin-mismatch`, 16 clauses). `--as descriptor` still packs it.
- The guard does not fire. `the_md1_column_matches_the_representability_rules` (`tests/descriptor_seam.rs:904-958`) computes `got` as `admit(..).is_ok() && md1_offenders(p).is_empty()`. `md1_offenders` (`admit.rs:365`) checks only §5.3(a)/(a″), not the new row, so the test passes on a false column. `POP.md1_admits_true: 15` (`:153`) is hand-counted and so is unchanged.
- `the_md1_route_derives_every_carried_address` calls `md1::build` directly and bypasses the refusal.
- The commit message says the vectors file changed "only `refusal_rows`". That is literally true, and it is the defect: a row's verdict changed and the file did not.

Failure scenarios:
1. The Go port's seam test, or any second implementation conforming to §7, reads `md1_admits=true` for this row. If it implements E3 it disagrees with the normative vector. If it trusts the vector, it treats a wallet whose md1 card would record the wrong origin for 16 cosigners as carriable.
2. Any future regression that drops the E3 refusal for depth mismatches still passes the column test.

Fix, either way:
- (a) Correct the fixture's origins to the true five steps (`48h/0h/0h/2h/<i>`). This keeps the `accepted-extreme` coverage on the md1 route, but it moves canonical, wallet-id and sha, and the fork's copy must follow.
- (b) Flip `md1_admits` to false in `rows.py`, regenerate, re-pin the sha, set `POP.md1_admits_true` to 14 and `cited` to 7, and widen the cited-slug allowlist to include `md1-origin-mismatch`.

In both cases, switch the column test's predicate to `md1_refusals(p, "", "").is_empty()`, so it tracks every md1 row rather than two of them. Also scan the other 73 rows. I did: with a stdlib parser, this is the only explicit-origin row that E3 flips. The four origin-less promotion rows pass under the implied path.

### Minor

**M-1. The identification block still prints a `wallet-id` for a wallet that md1 now refuses as mis-originated.**
- `descriptor/identify.rs:53-68` prints `wallet-id:` whenever `md1::build` succeeds. Its own comment says the line is "Emitted only when the wallet HAS an md1 policy form", so that a refused shape does not get "a DIFFERENT wallet, whose id would then sit two lines above the compare prompt".
- For an E3 input, `build` succeeds with the contradictory origin. The block prints `wallet-id: bbd2dc3a…` (16-key row, `--as md1`) immediately before the md1-origin-mismatch refusal. It prints the same on the `--as`-omitted choice block.
- Scenario: an operator records that id as "my wallet's md1 id", then re-exports with the corrected origin, and gets a different id for the same addresses.
- Fix: print the `none` variant, or a mismatch-specific variant, when `md1_refusals` would return `md1-origin-mismatch`. Or document that the id is "of the descriptor as written".

**M-2. E4's sentence overstates the device behaviour for an md1 with an unsupported wire version.**
- `main.rs:3036-3044` says unconditionally "an md1/mk1 this tool cannot confirm, which the device will treat as SECRET".
- `report_unconfirmed`, printed a few lines later for the `UnsupportedWireVersion` case (`main.rs:2558-2563`), deliberately says "a device that cannot read it either will treat it as a SECRET", because newer firmware may read it.
- Scenario: a card from a newer md-codec gets two contradictory statements on one screen.
- Fix: use `mdmk_unconfirmed_why` quietly, and qualify the clause when every unconfirmed record is version-noted. Or soften it to "which a device that cannot read it will treat as SECRET".

**M-3. E3 does not catch the same class of false derivation for origin-less keys whose implied path is only a guess. This predates E3, but the new spec row overstates what it catches.**
- `cascade.rs:862-866` gives every origin-less key the script's mainnet path, and md1 writes that path to steel (`md1.rs:307`).
- Measured: `wpkh(tpub…depth3,child 0h/<0;1>/*)` packs under `--as md1` with origin `84h/0h/0h`, where the true path is presumably `84h/1h/0h`.
- An origin-less depth-3 mainnet key from a BIP-44 or BIP-49 account inside `wpkh()` would likewise be recorded as 84h. E3 only checks depth and the last step.
- The §6 row's rationale ("an origin on steel is permanent") applies equally here.
- Fix: log a FOLLOWUP. Options are to refuse origin-less keys under `--as md1`, or to have the identification block flag the implied origin. Graded Minor because it is out of E3's ruled scope.

### Nit

- **N-1.** `main.rs:3041`: "records 0 and 1 are an md1/mk1 this tool cannot confirm" is a grammatical mismatch; "are md1/mk1 records" would fix it. In the `--no-passphrase` and SEALED branches the note is spliced after an already-wrapped line, which gives an over-long line followed by a hard break mid-sentence (see the `mnem-partial-nopp` stderr).
- **N-2.** The comment at `main.rs:2191-2195` ("`decide_sealing` itself is BYTE-UNCHANGED") is now stale wording; the decision is unchanged, the function is not. The doc comment at `sysw/record.rs:282-292` lists the walk's callers but not the new quiet caller.
- **N-3.** SPEC_descriptor_input `:1546` ("**§6 is 35 data rows.**") and `:2153` ("leaving **35**") are not annotated. Only `:43` notes the 36th row.
- **N-4.** The new refusal repeats the full clause per key. On the 16-key row it is about 3 KB of one sentence, ending "this key was not made at" in the singular. Consider grouping keys that share a clause.

## Answers to the brief

1. **I-1:** all three §14 sites and both descriptive sites are fixed and byte-match the code. No normative text still requires an echo.
2. **E3:**
   - Correct for depth 0: the origin must be empty, and `[fp]` alone is refused earlier, so any master key with an implied origin is refused, rightly.
   - The hardened bit is compared, and `h` and `'` parse to the same value.
   - Multipath does not affect the predicate.
   - BIP-48 cosigners and origin-less account-0 keys still pack. The fingerprint-only form was already refused.
   - No legitimate card is newly refused. The one vector row it flips is itself mis-originated, but the flip went unrecorded (I-1).
   - It leaks no key material.
   - The vectors file change is only the added row, which is the problem: one row's `md1_admits` is now false.
3. **E4:** the container bytes and the sealing decision are unchanged (cmp across 9 payloads, plus the sealed case by size, digest and verdict). The chunk-set-id warning prints exactly once. The wording is accurate in the NOT SEALED, SEALED and `--no-passphrase` branches, except for the version-note overstatement (M-2) and grammar (N-1).
4. **Fold:** faithful. All M and N items are logged, and nothing else changed.
