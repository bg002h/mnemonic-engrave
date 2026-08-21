# R5 — Test efficacy review: mnemonic-key `0feaaaa..main`

Independent verifier, hunting FALSE PASSES only (not design). Scope: `crates/mk-cli/tests/keys_batch.rs` (new), `crates/mk-cli/tests/template_id_stub.rs` (heavily changed), `crates/mk-cli/tests/gui_schema.rs` (added test), `crates/mk-cli/src/keyfile.rs` `#[cfg(test)] mod tests`. `vendor/` excluded.

Method: mutate production source, rebuild, run the named test(s) with both `cargo nextest run --locked --offline -E 'binary(<name)>'` and `cargo test --offline -p mk-cli --test <name>` (also a full `-p mk-cli` sweep for whole-suite survivors), record PASS/FAIL, then `git checkout --` the file before the next mutant. Working tree was restored after every mutant; final `git status --porcelain` shows only the pre-existing `?? design/SPEC_chunk_set_id_verification.md`.

Baseline (pre-mutation, both runners): `cargo nextest run --locked --offline -p mk-cli` → **132 tests run: 132 passed, 0 skipped**. `cargo test --offline -p mk-cli --test keys_batch --test template_id_stub --test gui_schema --bin mk` → all green (25 tests across the three integration files + the bin's inline `#[cfg(test)]` modules).

---

## Findings

### Important — `verify_from_keyless_template_md1_matches_template_id_stub`: never checks the actual stub VALUE; passes under two independent corruptions of the identity it claims to pin

`file: crates/mk-cli/tests/template_id_stub.rs:159-216`

The test's own name and doc claim it proves "verify agrees with the correct `WalletDescriptorTemplateId`-derived stub." Its three steps are: (1) encode a card via `--from-md1 <template>`, (2) verify that same card against `--from-md1 <template>` — expects exit 0, (3) verify a card **explicitly stamped with the literal `3d190af3`** (`POLICY_STUB_FOR_TEMPLATE`, hardcoded, unrelated to the derivation under test) against `--from-md1 <template>` — expects exit 4 (`ContentMismatch`).

Steps 1+2 are a tautology: both sides call the *same* (possibly-broken) `derive_stub_from_md1_card`, so they always agree regardless of correctness. Step 3 only proves the derived stub differs from one arbitrary literal — it never compares against `EXPECTED_TEMPLATE_STUB` (`0x559e64b2`), the actual golden the other tests in the file pin. Two independent mutations of the identity-selection logic confirm this test cannot tell correct from broken:

**Surviving mutant A** — swap the keyless (`else`) arm to also use `WalletPolicyId` instead of `WalletDescriptorTemplateId` (the inverse-form bug the probe brief called out):

`crates/mk-cli/src/cmd/mod.rs:131-135`
```rust
let id_bytes = if descriptor.is_wallet_policy() {
    *md_codec::compute_wallet_policy_id(&descriptor)?.as_bytes()
} else {
    // MUTANT: keyless arm also uses WalletPolicyId (should be TemplateId).
    *md_codec::compute_wallet_policy_id(&descriptor)?.as_bytes()
};
```
Evidence: `cargo nextest run --locked --offline -E 'binary(template_id_stub)'` → **8 tests run: 5 passed, 3 failed** (`encode_from_keyless_template_md1_uses_template_id_stub`, `single_string_and_chunk_set_mix`, `one_wallet_two_forms_two_stubs` fail — correctly). `verify_from_keyless_template_md1_matches_template_id_stub` is listed among the 5 PASSES. Isolated confirmation: `cargo test --offline -p mk-cli --test template_id_stub verify_from_keyless_template_md1_matches_template_id_stub -- --nocapture` → `test result: ok. 1 passed; 0 failed`.

**Surviving mutant B** — shift the stub byte window from `[..4]` to `[1..5]` (corrupts every stub, both forms):

`crates/mk-cli/src/cmd/mod.rs:137`
```rust
stub.copy_from_slice(&id_bytes[1..5]);   // was &id_bytes[..4]
```
Evidence: `cargo nextest run --locked --offline -E 'binary(template_id_stub)'` → **8 tests run: 2 passed, 6 failed** (6 of the 8 tests in the file correctly fail). `verify_from_keyless_template_md1_matches_template_id_stub` is one of only 2 survivors. `cargo test --offline -p mk-cli --test template_id_stub` confirms the same 6-failure list, with the verify test absent from it (`2 passed; 6 failed`).

Both mutations are caught by sibling tests in the same file (`encode_from_keyless_template_md1_uses_template_id_stub`, `one_wallet_two_forms_two_stubs`), so the *suite* does not regress silently — but the test named specifically for this property provides zero of that coverage. A future refactor that "fixes" step 3's literal to something that happens to equal the (broken) derived value — e.g. if someone regenerates `POLICY_STUB_FOR_TEMPLATE` from a future buggy build — would make this test pass even more vacuously with no other signal.

**Suggested assertion**: add, right after building the encode-side card in step 1, `assert_eq!(mk_codec::decode(&refs).unwrap().policy_id_stubs[0], EXPECTED_TEMPLATE_STUB, "encode must stamp the golden template-id stub, not just agree with itself");` — anchoring to the same independent golden the other tests in the file use.

---

### Critical — `mk encode --keys --json`: cards[1] and cards[2] can be silently swapped in the JSON `cards` array and NOTHING in the suite (132/132) catches it

`file: crates/mk-cli/tests/keys_batch.rs:280-321` (`json_batch_wraps_the_single_card_object`)

The only JSON-batch test checks `card_count`, `cards.len()`, and that `cards[0]`'s fields equal a single-card encode of `KEYS[0]`. It never inspects `cards[1]` or `cards[2]`. The plain-text batch tests (`batch_matches_per_key_loop`, `record_order_follows_file_order`) never pass `--json`, so they provide no cross-check either. Result: the JSON array — the contract `mnemonic-gui` and any programmatic consumer reads — can present a cosigner's card at the wrong index while `card_count` is correct, `cards.len()` is correct, and `cards[0]` is correct.

**Surviving mutant** — swap `cards[1]` and `cards[2]` in the JSON envelope while leaving the plain-text emission path (and `cards[0]`) untouched:

`crates/mk-cli/src/cmd/encode.rs:233-236` (`emit_json_batch`)
```rust
fn emit_json_batch(minted: &[Vec<String>]) -> Result<()> {
    // MUTANT: swap cards[1] and cards[2] (index 0 stays put) so the JSON
    // array order diverges from the plain-output order after the first card.
    let mut reordered: Vec<&Vec<String>> = minted.iter().collect();
    if reordered.len() > 2 {
        reordered.swap(1, 2);
    }
    let cards: Vec<_> = reordered.iter().map(|s| card_json(s)).collect();
    ...
```
Evidence:
- `cargo nextest run --locked --offline -E 'binary(keys_batch)'` → **8 tests run: 8 passed, 0 skipped**.
- Full-crate sweep, `cargo nextest run --locked --offline -p mk-cli` → **132 tests run: 132 passed, 0 skipped**.

This is exactly the failure mode the module's own doc comment (`keyfile.rs:13-19`) warns about for the record-parsing format — "a desync here does not fail — it mints a card naming the WRONG master" — reproduced one layer up, in the JSON serialization the batch feature exists to make consumable, with zero test coverage across the whole crate.

**Suggested assertion**: extend `json_batch_wraps_the_single_card_object` (or add a new test) to check *every* index, not just `cards[0]`:
```rust
for (i, (fp, path, x)) in KEYS.iter().enumerate() {
    let single: Value = /* encode KEYS[i] alone, --json */;
    for key in ["mk1_strings", "chunk_count", "code_variant"] {
        assert_eq!(cards[i][key], single[key], "batch card[{i}] must equal a single-card encode of KEYS[{i}]");
    }
}
```

---

## Structural checks (no mutation needed)

- No test in scope asserts only `status.success()` without inspecting output content; every success-path test in `keys_batch.rs`, `template_id_stub.rs`, and `gui_schema.rs` goes on to parse and assert on stdout/JSON.
- No substring assertion in scope is short enough to plausibly match an unrelated message (`"BIP-380"`, `"chunk set incomplete: got 3 chunks, expected 4"`, `"mutually exclusive"` + the specific flag name, `":3:"` anchored to a line-number position, etc. — all specific).
- No `assert_ne!` in `template_id_stub.rs` is used standalone in place of a possible `assert_eq!` to a known value: every `assert_ne!` (`POLICY_STUB_FOR_TEMPLATE`, `EXPECTED_TEMPLATE_STUB` in the keyed test, `KEYLESS_FORM_POLICY_STUB`) is paired with a preceding `assert_eq!` to the correct golden in the same test — except in `verify_from_keyless_template_md1_matches_template_id_stub`, covered above as its own finding (there the "known value" comparison, `assert_eq!(status.code(), Some(4))`, is real but derived from an unrelated literal rather than anchored to a golden).
- `KEYED_POLICY_A_CHUNKS`/`KEYED_POLICY_B_CHUNKS` (the two chunk-set fixtures) are both genuinely exercised by every assertion that names them — `two_chunk_sets_are_two_cards_in_order` and `interleaved_chunk_sets_still_group_by_set_id` both assert the *pair* `[EXPECTED_KEYED_POLICY_A_STUB, EXPECTED_KEYED_POLICY_B_STUB]` in order, not a subset or an unordered set, so neither fixture could be silently dropped without failing.
- `--group-size` in batch mode: not independently probed as a mutant. `mk encode`'s rendering loop calls the identical `render_grouped(s, args.group_size, ...)` for both the single-card and `--keys` branches (`crates/mk-cli/src/cmd/encode.rs:195-199`) — there is no batch-specific code path to mutate that non-batch tests (`encode_grouping_flags.rs`) wouldn't equally catch. Not a batch-specific gap.
- The "ignore the record PATH" probe (fingerprint mutant's inverse) **was tried and killed**: mapping every batch card to the *first* record's path (same depth/last-child so `KeyCard::new`'s own xpub/path sanity check doesn't trip first) is caught by `batch_matches_per_key_loop` (full-vector equality across all 3 cards) — `record_order_follows_file_order` alone does *not* catch it (it only pins card `[0]`, whose path was unchanged by this specific mutant), but the suite as a whole does.

---

## VERDICT: 2 surviving mutants, 1 Critical, 1 Important, 0 Minor, 0 Nit

### Mutants tried that were correctly KILLED
1. `derive_stub_from_md1_card`: keyless arm → `WalletPolicyId` (is_wallet_policy()→true equivalent) — killed by `encode_from_keyless_template_md1_uses_template_id_stub`, `single_string_and_chunk_set_mix`, `one_wallet_two_forms_two_stubs` (survived only against `verify_from_keyless_template_md1_matches_template_id_stub`, see Important finding above).
2. `derive_stub_from_md1_card`: stub byte window `[..4]` → `[1..5]` — killed by 6 of 8 `template_id_stub.rs` tests (survived only against the same verify test).
3. `derive_stub_from_md1_card`: swallow a `reassemble` failure and return a zero stub instead of erroring — killed by `incomplete_chunk_set_is_refused`.
4. `mk encode --keys` cards mapping: ignore each record's own path, use the first record's path for every card — killed by `batch_matches_per_key_loop`.
5. `emit_json_batch`: reverse the full JSON `cards` array vs. plain-output order — killed by `json_batch_wraps_the_single_card_object`.
6. `emit_json_batch`: off-by-one `card_count` (`cards.len() - 1`) with a correct `cards` array — killed by `json_batch_wraps_the_single_card_object`.
7. `read_key_records`: silently skip an unparseable line instead of erroring — killed by `bad_record_names_the_line_number`.
8. `gui_schema::flag_from_arg`: drop the `REQUIRED_IN_GUI_FORM` override (keep `CLI_ONLY_FLAGS` filter) — killed by `gui_schema_encode_xpub_and_origin_path_required` and `keys_flag_is_cli_only_and_absent_from_gui_schema`.
9. `gui_schema::build_schema`: drop the `CLI_ONLY_FLAGS` filter (keep `REQUIRED_IN_GUI_FORM`) — killed by `keys_flag_is_cli_only_and_absent_from_gui_schema`.

Pre-existing "already proven killed" mutants from the dispatch brief (grouping never-merge/merge-all/`md1_chunk_set_id`-always-None/adjacency-grouping; unconditional `is_wallet_policy()`→true; drop fingerprint/reverse order/off-by-one line number/accept use-site suffix/no blank line/accept conflicting flags) were not re-run — brief states these are already confirmed.
