# Review r1: mnemonic-engrave PR #68 (`claude/project-thread-utm1aa`, head 67c3f1c)

Independent adversarial review of `git diff origin/master...claude/project-thread-utm1aa`
(4 commits: 8e91318 E1, cac4b85 E2, fc5f7c5 X7, 67c3f1c E5). Nothing in the repo was edited, committed or pushed.

**Verdict: 0 Critical, 1 Important, 6 Minor, 5 Nit.** The code fixes are correct and their tests discriminate.
The Important finding is that normative spec text still prescribes the old, leaking message.

## Evidence run

- `cargo test --locked -j 2 -p mnemonic-engrave --no-fail-fast`, captured once. All four `bh1125*` rows pass
  (`classify::tests::bh1125_e1_unknown_hrp_display_carries_no_input`, `bh1125_e1_unknown_hrp_never_echoes_the_input`,
  `bh1125_x7_md1_strings_never_returns_a_card_the_decoder_refuses`, `bh1125_e5_a_set_that_is_not_the_balanced_split_is_not_confirmed`).
  The failures were all environmental:
  - `cross_lang` ×2 and `preview_cross_lang`: no seedhammer submodule.
  - `history_purge` ×3: zsh/fish are not installed.
  - `sysw_cli::pack_does_not_print_a_digest_for_a_write_that_failed`: the run is as root.
  - `seal::v2_tests::v2_pw_form_0_keeps_the_checksum`: a timing check. It passes when run alone.
  - `seal::v2_tests::v2_titles_refusals`: fails only at the timing assert, `v2_tests.rs:857` (`< 200ms`, "before any KDF"), also when run alone. The changed message assert at `:851` passes before that line.
  - `v1_open_treats_text_records_as_records` (the other changed expectation) passes.
- I fetched BIP-68 and BIP-112 from github.com/bitcoin/bips master. I fetched Bitcoin Core `src/consensus/tx_verify.cpp` at master and at v27.0.
- `Cargo.lock` diff touches exactly two entries: md-codec (0.48.5@c8bb97a5 → 0.48.7@c8cde234) and mt-codec (eec7910b → 9b094e0b). There is no `vendor/` in this repo.
- Miniscript patch: `Cargo.toml:25` is `ff4732e5…`, and `git -C descriptor-mnemonic show c8cde234:Cargo.toml` line 55 is the same rev. They are equal.
- c8cde234 is on descriptor-mnemonic `origin/main`. 9b094e0b is on mnemonic-transaction `origin/main`.

## 1. E1 (no echo of input text)

The fix holds for every path I could find.

- `ClassifyError::UnknownHrp` now carries `Option<&'static str>` (`crates/me-cli/src/classify.rs:30`). Its derived `Debug` and its `Display` cannot carry input bytes.
- All three consumers render only `ClassifyError`:
  - the converter, `lib.rs:48` → `main.rs:1024` `eprintln!("me: {e}")`;
  - `me bundle`, `bundle.rs:107` `"cannot classify input: {e}"`;
  - `me seal`, `seal/record.rs:156` `Unclassifiable(e.to_string())`.
- `BundleError::Classify(String, _)` still holds the raw line, and `BundleError` derives `Debug`. No production path formats it with `{:?}`: the only `{e:?}` in `main.rs` are `:2228` (passphrase generation) and `:2428` (wire header), neither of which reaches this.
- No `--json` error output exists for these verbs.
- The `mr` exception leaks nothing. The match runs on the lowercased HRP, and the printed value is the list's own `&'static str`.
- Other codec errors carry no input body. md-codec `unwrap_string` errors carry at most one character (`codex32.rs:167`). mk-codec `InvalidHrp` is already redacted in `ValidateError`. ms-codec's `WrongHrp{got}` can only be "ms" here.

## 2. E2 (BIP-68 spec text) checked against the sources

Every fact checks out against BIP-68 §Specification and Core `CalculateSequenceLocks`:

| Claim | Source |
|---|---|
| nVersion ≥ 2 | BIP-68 l.26: "transactions with an nVersion greater than or equal to 2" |
| Read unsigned | Core v27 `static_cast<uint32_t>(tx.nVersion) >= 2`; master `tx.version >= 2` with uint32 version |
| Disable bit 31 | BIP-68 l.30-32; Core `SEQUENCE_LOCKTIME_DISABLE_FLAG` skip |
| Type bit 22, 512 s | BIP-68 l.38 |
| Mask 0x0000ffff | BIP-68 l.40 |
| Zero means no lock | BIP-68 l.46/51 |
| Coinbase exempt | BIP-68 l.53 |

The quoted fragments are verbatim. The rule's wording ("bit 31 clear and a non-zero low 16 bits", "bits 16–21 and 23–30 ignored") is correct. `non-BIP68-final` is Core's real reject reason. The PR hunks are byte-identical to mnemonic-transaction's T1 copy (`origin/claude/project-thread-utm1aa`), so the "mt1 design lives here, re-sync there" hazard is respected for this change. The two SPEC copies already differed in other sections before this PR.

## 3. Pins

- **md-codec at c8cde234 differs from the tag `md-codec-v0.48.7` (e11b4ce1).** It adds the following (`git diff md-codec-v0.48.7 c8cde234 -- crates/md-codec`):
  - `to_miniscript_descriptor_with_network` now refuses a chain no key has;
  - the chain check in `derive.rs` moves into the shared `check_chain_in_range`;
  - README/doctest changes.
- `me` calls only `Descriptor::derive_address` (`descriptor/md1.rs:422`). Its check is the same logic moved, so `me` sees no behaviour change beyond the intended refusals: `split` → `encode_payload` now decodes its own output.
- `me`'s builder emits no `@0` and no redundant override (`md1.rs:242-249`). So the 0.48.6 template-id change and the 0.48.7 refusal are unreachable from `me`'s own builds today.
- mt-codec moves eec7910b → 9b094e0b in three mt-codec commits:
  - 95c39a3: the balanced-split refusal (intended);
  - 3923ad6: additive `DecodedChunk.invariant_prefix` / `input_position` fields;
  - 1b336c2: tests.
  
  `me` builds no `DecodedChunk` literal, and the build is clean.

## Findings

### Important

**I-1. Normative spec text still prescribes the leaking message.** The spec is `design/SPEC_encrypted_payload_delivery.md`. Rule text in §14 (`:2408-2409`) says the public-section refusal reads `unrecognized HRP '<the record's opening characters>' (expected md, mk, ms, or mt)`. Vector row N-public-titles (`:2602`) and TRACE-style row `v1_open_treats_text_records_as_records` (`:2735`) quote the old exact string `'text:726566756769756d2d706c6'`. The PR changed both tests (`seal/v2_tests.rs:851-856, 864-869`) but none of the three spec sites.

Failure scenario: a later implementer, or the Go port, conforms to §14 as written. That puts back the exact echo E1 removed, because §14 *requires* printing the record's opening characters. Here those characters are hex of free text, which may be secret. The spec and the tests now disagree on a normative string.

Fix: amend the three sites to the new wording, with an E1 amendment note. Also update `design/SPEC_sh2_demo_payload.md:224-228`, whose console transcript shows `'key:6'` / `'key:5b37336335646'`. That one is descriptive, so it can be Minor if it is treated as non-normative.

### Minor

**M-1. A user loses diagnostic information for public, non-secret prefixes.** The whitelist is `["mr"]` only (`classify.rs:38`). Before this PR:
- `me seal --plaintext <bitcoin address>` printed `unrecognized HRP 'bc'`, recorded as expected behaviour in `design/agent-reports/RECON-payload-walk-matrix.md:134` and `hashlock-H5-spec-R0-r0-journey.md:61`;
- `key:`/`text:`/`tx:` records printed their prefix (`SPEC_sh2_demo_payload.md:224`; `FOLLOWUPS.md:11090`, `:12365`).

All of these now get one generic message. An operator who pastes an address, or a composer record into `me seal`, no longer learns what they pasted.

Fix: add `bc`, `tb`, `bcrt` to `KNOWN_PUBLIC_HRPS`. Name a fixed set of reserved record prefixes (`key:`, `text:`, `tx:`, `pass:`, `hash:`, `now:`) by the prefix only, never the body. Both are static strings, so nothing from the input is echoed.

**M-2. The bug hunt's second fix direction was not taken, and nothing tells the operator.** A mistyped ms1 (`msl0…`) or a seed phrase followed by a `1` still exits 4 ("invalid") with a generic message. Exit 3 means "refused secret". Nothing tells the operator they just fed secret material to a public-strings tool. This is acceptable as scoped, since nothing is printed any more. Record it in FOLLOWUPS, or add a cheap secret-shape check: an `ms`-prefixed codex32 body, or a run of BIP-39 words, refused with exit 3.

**M-3. The amended spec wording for a seconds lock is slightly early.** The wording is "counted from the block that confirms the coin it spends" (`SPEC_mt_v0_1.md:2626-2627`, `:2915-2916`). In Core, `nCoinTime` is the MTP of the block *before* the coin's block, and it is compared against the MTP of the spending block's parent (`tx_verify.cpp:80, 94`, and the BIP-68 "mining date … median-time-past of the previous block"). MTP lags wall-clock by about an hour. A recoverer who reads `RELATIVE LOCK 51200 SECONDS` as wall-clock seconds after confirmation can broadcast roughly 1-2 h early and be refused `non-BIP68-final`. That is fail-safe and retryable, but it is on steel. Fix: say "measured in median-time-past, from the block before the one that confirms the coin" on the stderr line, and optionally add a margin note. The BLOCKS wording is exact: nMinHeight = coinHeight + n − 1.

**M-4. The new legend line exceeds the measured per-line width, and was not re-measured.** The line is `RELATIVE LOCK 32768 BLOCKS ON INPUT 0` (37 chars). The worst case is `RELATIVE LOCK 33553920 SECONDS ON INPUT 12` (43 chars). Both exceed the 35-char field and line budget (`SPEC_mt_v0_1.md:1769`; `RESULTS_legend_budget_2026-08-22.txt:1`, "35 chars/line"). The minimal legend's "fits budget, 6 lines" no longer holds for a multi-input pre-signed recovery spend: each relative-locked input adds one line that wraps to two. The spec says so (`:2944-2952`) but does not re-measure.

Fix: re-run `legend.rs`, or use a shorter spelling, or collapse inputs that share a lock into one line (`RELATIVE LOCK 32768 BLOCKS ON INPUTS 0-4`). This belongs in the mt T1 thread as well.

**M-5. The F-700 pin equality holds only after mnemonic-refugium moves.** The comment at `crates/me-cli/Cargo.toml:39-40` says this rev "EQUALS mnemonic-refugium's workspace pin". mnemonic-refugium `origin/master` and its `origin/claude/project-thread-utm1aa` both still pin c8bb97a5. Only an uncommitted local change in `/home/user/mnemonic-refugium` moves it to c8cde23. No artifact links both crates today: refugium-wallet links only mr-codec, at c8bb97a5. But if this PR merges first, the invariant is false on master. Merge-order requirement: land refugium's re-pin first or together, and check that it is the full rev `c8cde2346114ae461cbeb3be93ed9b46b00cc31b`.

**M-6. No CHANGELOG entry.** `crates/me-cli/CHANGELOG.md [Unreleased] ### Fixed` records every bug-hunt fix in this repo (bh-me-I4, I10, canonical-depth). The E1 change alters printed text that operators and docs quote, and the two pin bumps add refusals. None of the three is recorded.

### Nit

- **N-1.** The `Cargo.toml:33-37` comment says "main after tag md-codec-v0.48.7 (0.48.6's fix and 0.48.7's encode-decodes check)". It omits that c8cde234 also carries the `to_miniscript` chain refusal and the derive refactor, which are not in the tag (e11b4ce1). Cross-repo issue: descriptor-mnemonic's own `CHANGELOG.md` at c8cde234 lists the chain refusal under `md-codec [0.48.7]`, but the tag `md-codec-v0.48.7` does not contain it. One version number now names two codes. Worth a descriptor-mnemonic FOLLOWUP. There is no `me` impact (see §3).
- **N-2.** `tests/bh1125.rs`, `"mnemonic"` case: `me bundle` sees the line `abandon … about` with no `1` and returns `NoSeparator` both before and after the fix. That half of the case does not discriminate. The converter half and the `seed-note` case do.
- **N-3.** E5 and X7 rows assert only `is_none()` / `is_err()`, not the reason.
  - E5: mnemonic-transaction's row `r2_recut_even_vector_is_refused_in_both_orders` (9b094e0b) shows the same 38/36/37… re-cut is BCH-valid and refused with `ChunkLengthMismatch`, and that the old decoder returned the true bytes in index order. So the `me` row is RED on the old pin, for the right reason.
  - X7: I did not run it against 0.48.5. That it would fail there rests on md-codec's 0.48.7 CHANGELOG ("an `@0` or redundant use-site override … encoded at `Ok`").
  - The X7 row builds a descriptor `me`'s builder never produces (`md1.rs:242-249`), so it pins the codec guard, not a reachable `me` defect. The doc comment says as much.
- **N-4.** The BIP-112 sentence (`SPEC_mt_v0_1.md:2889-2891`) is simplified. CSV also fails on a transaction version below 2, a type-flag mismatch, or the disable flag set in nSequence, and it is a NOP when the operand's disable flag is set. The conclusion still holds.
- **N-5.** `design/FIX_me_recovery_checklist_partial_keys.md:8` still says "current pin c8bb97a5". It is a historical document, so it may stay.

## Answers to the brief's questions

1. **E1:** no remaining print path for input text through ClassifyError, `--json`, `me seal` or Debug. The `mr` exception is safe. Legitimate information lost: the address and record-prefix hints (M-1).
2. **E2:** correct against BIP-68 and Core source. The only imprecision is the seconds-lock reference point (M-3), plus the BIP-112 simplification (N-4).
3. **Pins:**
   - md-codec c8cde234 matches refugium's pending local change only (M-5).
   - Cargo.lock changes only the two intended entries.
   - There is no vendor/ directory.
   - The miniscript patch equals descriptor-mnemonic's ff4732e5 at c8cde234.
   - No `me`-visible change beyond the intended refusals (N-1).
4. **Tests:** E1, E5 and X7 rows fail without their fixes, with the caveats in N-2 and N-3.
