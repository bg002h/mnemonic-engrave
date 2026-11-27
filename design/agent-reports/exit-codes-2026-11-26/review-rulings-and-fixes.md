# Independent review: Brian's §2b rulings and the two "fix soon" items (2026-11-27)

Scope: mnemonic-engrave dddff0c, 19f6e23, c7f1f16; mnemonic-transaction 9c85c94;
mnemonic-key cdcaeb4; mnemonic-toolkit 09f5f8e0; mnemonic-gui 8e2a973. Worktrees
/home/user/fx/<repo>. I edited no source. Builds were debug (opt-level 2, `DEBUG=0`), not
`--release`. Probe inputs went to the session scratchpad.

**Verdict: 0 Critical, 0 Important, 6 Minor, 5 Nit.** The two implementations of the
mixed-set-id refusal gave the same result on every edge shape I tried. `--offline`
runs no `bitcoin-cli` on any path. The phrase fix does what its commit claims. The
remaining wipe gaps are outside that file (ms-codec, and the record stream). The
operator ruling of 2026-08-27 puts secret-handling findings below Important, so they
are graded Minor.

---

## Critical

None.

## Important

None.

## Minor

**M1. ms-codec's `validate_phrase` still makes two unwiped copies of the phrase, on every parse (secret handling).**
- Where: `mnemonic-secret@62b4d9d crates/ms-codec/src/hashlock.rs:343-349`, `looks_like_ms1`.
  It is called from `validate_phrase` (`:374`), which `parse_phrase` calls at
  `crates/me-cli/src/sysw/composer_records.rs:439`.
- What happens: `raw.trim().to_ascii_lowercase()` builds a heap `String`, and
  `.chars().filter(..).collect()` builds a second one. Both hold the lowercased phrase
  without hyphens, spaces or commas. Both are freed without a wipe.
- How often: `composer_records::parse` runs on every record many times per run.
  The callers include `sysw::classify_with` (mod.rs:483), the admission check (mod.rs:743),
  `now_indices` (composer_records.rs:391), `report_phrase_record_kind_disagreement`
  (main.rs:3320) and `preimage_of` (main.rs:3496). So each `phrase:` record leaves
  several such copies.
- Everything else in me-cli is sound. `from_utf8` and `split_once` borrow. The
  `Zeroizing` buffer is sized up front, so it does not reallocate. `Clone` of
  `Zeroizing<String>` stays `Zeroizing`. `PartialEq` copies nothing. No `format!` or
  `to_string` touches the phrase.
- Related: the new FOLLOWUP `sh-phrase-record-decoded-buffer-unwiped` says "no Rust change
  owed". On this point that is not accurate.
- Fix: in ms-codec, run the ms1-shape test over the bytes in place (skip the separators,
  compare ASCII case-insensitively), or build the copies in `Zeroizing<String>`. Then file
  the companion item in mnemonic-secret and correct the FOLLOWUP's "no Rust change owed".

**M2. The hex form of the phrase is never wiped (secret handling, older than this commit).**
- What happens: `read_records` (main.rs:3703-3720) keeps each `phrase:<hex>` record in a
  plain `Vec<String>`. It also keeps the whole document in a plain `String` (`document: raw`).
  `split_record_stream` (`mnemonic-io-lib/src/records.rs:16`) copies every line again.
  The hex reverses to the phrase in one step, so wiping the decoded copy shrinks the
  exposure but does not end it.
- Contrast: `phrase_record()` already returns `Zeroizing<String>` on the build side.
- The commit message does not overstate this. It says "the decoded phrase".
- Fix: read the record stream into `Zeroizing` buffers, as `convert` and `bundle` already
  do (main.rs:1047-1106). Or file this as its own FOLLOWUP, so it is not taken as closed.

**M3. `mt` says "re-cut it" for an unreadable spare string and still exits 0.**
- Measured with `mt decode --offline`. Input: the corpus vector `even`, plus a copy of
  chunk 3 with 12 characters changed. Result: rc 0. stderr has "UNREADABLE STRING. string 7
  …" and then "if it is one of THESE, re-cut it from the strings mt has just verified"
  (`crates/mt-cli/src/main.rs:1710-1716`).
- Why this is defensible: the advice is conditional. mt cannot tell which plate the string
  came from, and the text says so.
- Why it is still a gap: Brian's wording (spec §2b item 2) is "Whenever `mt` tells the
  operator to re-cut a plate, the run adds 16". The SPEC_mt hunk at `:3483` is categorical
  too: "16 is also set whenever `mt` tells the operator to re-engrave a plate". So this
  message, as worded, is a counter-example to both. No test pins this case at 0.
- Fix, either of:
  - Get Brian's ruling that conditional advice about an unidentified plate does not count.
    Then narrow the two spec sentences to "a plate mt identifies", and add a test that
    pins 0.
  - Or set 16 when `set.unreadable` is non-empty.

**M4. `verify --offline` (64) is built but absent from the constellation spec.**
- Where: §2b item 1 (`SPEC_constellation_exit_codes.md:342-344`) and §7d (`:881-882`).
  Both describe `--offline` and `--offline` with `--bitcoin-cli`.
- The gap: neither says `mt verify --offline` is 64. mt builds it
  (`offline_unsupported_guard`, main.rs:2718-2739) and the SPEC_mt copies record it (both
  carry the `:3447` line).
- Fix: add to §7d, after the `mt verify --bitcoin-cli` bullet: "`mt verify --offline` is
  usage, 64, for the same reason."

**M5. The toolkit manual row for the fingerprint mismatch quotes an error text the build does not print.**
- Where: `docs/manual/src/40-cli-reference/41-mnemonic.md:1062`. 09f5f8e0 added "(exit 2)"
  to this row.
- The row quotes `error: slot @{N} fingerprint mismatch: --slot says X, descriptor inline
  [Y/...] disagrees; supply consistent values.`
- The build prints `--slot @{idx}.fingerprint specifies {slot} but descriptor @{idx}
  annotation specifies {anno}` (`src/cmd/bundle.rs:1908`, `DescriptorParse`, exit 2).
  `tests/bughunt_s1.rs:1621` pins that text.
- The phrase-fingerprint row below it (`:1063`) is stale in the same way (compare
  `bundle.rs:1855`).
- The exit code in the row is right. The text drift is older than this commit.
- Fix: quote the real messages. The `manual-mr`/flag lint does not check printed text, so
  nothing catches this.

**M6. As root, `me sysw pack --out /dev/null` changes `/dev/null` to mode 0600. That is why `pack_does_not_print_a_digest_for_a_write_that_failed` fails here.**
- Where: `crates/mnemonic-io-lib/src/write.rs:121-142`. `write_direct` calls
  `set_permissions(0o600)` on whatever it opened, including a character device.
- What I measured:
  - `/dev/null` in this container is `crw------- root root`, mtime Nov 24 23:38. That
    predates dddff0c, so an earlier root run of this test left it that way.
  - At HEAD the test fails at its first assertion, `sysw_cli.rs:2919`: "the write to
    /dev/null must fail". Root may open and chmod a root-owned 0600 device, so the write
    succeeds.
- The claim holds:
  - It is the root environment: as a non-root user, `set_permissions` fails with EPERM, so
    the write fails as the test expects.
  - It is independent of dddff0c: the commit does not touch `emit`, `write_out`,
    `write_private` or the `text:` record path. I read the diff for that; I did not build
    dddff0c^.
- Defects behind it, both older than this commit:
  - In the product: run as root, `--out /dev/null` chmods the system `/dev/null`, which
    breaks it for every other user.
  - In the test: running it as root mutates the host.
- Fix:
  - In `write_direct`, skip `set_permissions` and `set_len` unless `metadata().is_file()`.
  - Make the test pick a failing destination that does not rely on non-root rights (for
    example a path under a read-only directory), or skip it when `geteuid()==0`.

## Nit

- **N1.** mt's ordering tests do not prove "usage before input read": every 64 case feeds
  valid input. I measured it by hand instead: `echo garbage | mt decode --offline
  --bitcoin-cli /bin/false` → 64, and `echo garbage | mt verify --offline` → 64. Add one
  garbage-input row to `offline_with_a_named_bitcoin_cli_is_usage`.
- **N2.** Under `--offline`, `report::no_node_warning` says "run mt inspect again without
  --offline" (report.rs:739), and `decode` prints it too (main.rs:1848). The "mt inspect"
  verb was already wrong for decode before this commit; the new line repeats it.
- **N3.** mk's refusal says "or chunks of two cards mixed" even when three ids are named. The
  toolkit text says the same. The wording is cosmetic and the two tools agree.
- **N4.** `SPEC_constellation_exit_codes.md:881` is a line about 95 columns long in a file
  otherwise wrapped at 79 (the reflow was left undone).
- **N5.** §7b (`mk`) and §7f (toolkit) list each tool's moves but not the new repair refusal
  or the bundle path 1→2. §2b items 3 and 5 carry both, so this is only a missing
  cross-reference.

---

## Checked and sound

### 1. me dddff0c (phrase record)

- **Shared decoder equivalence.**
  - Read: record.rs `unhex_lower` and `nibble` (`:504-538`) accept only even length and
    bytes `0-9a-f`. The old composer decoder did the same: an `is_ascii_digit || a..=f`
    prefilter, then `to_digit(16)`. Multi-byte UTF-8 is rejected byte-wise in both. So the
    two accept and reject the same strings and decode them to the same bytes.
  - The `key:`/`hash:`/`now:` parsers are therefore unchanged.
  - Measured: `the_shared_decoder_matches_the_composer_grammar_string_for_string` passes.
- **The equivalence test is real.**
  - It calls `super::unhex_lower`, which is the imported record.rs function.
  - It enumerates every string of up to 4 characters over 13 boundary characters (30,941
    inputs) plus the corpus bodies.
  - It asserts that both the accepted and the refused side are non-trivial (more than 100
    inputs each).
  - The stated mutation (accept `A-F` in `nibble`) would red it.
- **Debug and the preimage path.**
  - Debug is redacted for `PhraseRecord` and for the enclosing `ComposerRecord`. Measured:
    `sysw_composer_records`, 27 passed.
  - `preimage_of` returns `Zeroizing<[u8;32]>`, and `HashlockMethod::preimage` hands back
    ms-codec's own `Zeroizing`. Neither copies X.
  - The callers (main.rs:3436-3463) use `&x` only for `digest_of`.

### 2. mt 9c85c94

- **No path spawns `bitcoin-cli` under `--offline`.**
  - Read: `Command::new` appears only at node.rs:201, inside a `Node` method. A `Node` is
    built only by `Node::find`, which returns `None` first thing when `is_offline()`.
  - `go_offline()` is set in `main` before any verb runs.
  - Every subcommand defines `offline`, so `get_flag` cannot panic.
- **Guard order.** `offline_with_bitcoin_cli_guard` runs before `read_input` in `encode`,
  `decode` and `inspect`. `offline_unsupported_guard` runs before input in `verify`.
- **When 16 is set.** It is set exactly when `discarded_corrections > 0`, which is the same
  branch that prints "it is the one to re-cut" and the JSON "re-cut it". Two clean copies
  stay 0.
- **No-node wording.** Under `--offline`, every no-node headline says "no node was asked",
  in prose and in JSON.
- **Measured:** the 7 targeted tests pass, including the stub-on-PATH control and
  `--offline` cases, `verify_refuses_offline` and both duplicate cases. The two hand-run
  64s are under N1.

### 3. mk cdcaeb4 and toolkit 09f5f8e0

- **The two algorithms are the same.** Both take short groups only, bucket them by declared
  total, OR their indices, and refuse when at least 2 groups together cover every index.
  Complete, single-string, malformed and unparseable groups are excluded. The check runs
  after the clean not-a-chunk refusal and before the per-group shape loop.
- **Measured** with fresh cards from `mk encode --chunk-set-id`: three 3-chunk cards (ids
  0x11111, 0x22222, 0x33333), one 2-chunk card (0x44444), and a 2-chunk card under
  0x11111. Each case was run through `mk repair`, `mnemonic repair --mk1` and
  `mnemonic repair --json --mk1`:

  | Case | Exit (all three) | Stdout |
  | --- | --- | --- |
  | Three ids cover | 2 | empty |
  | Overlapping cover | 2 | empty |
  | One chunk corrected, cover | 2 | empty |
  | Complete card + B0 + C1 + C2 | 2 | empty |
  | Cover of 3 + a non-covering group of 2 | 2 | empty |
  | Garbage + cover | 2 | empty |
  | Duplicate index + cover | 2 | empty |
  | Same id, different totals | 2 | empty |
  | Two ids, no cover | 0 | printed |
  | Different totals | 0 | printed |
  | Complete card + another card's lone chunk | 0 | printed |
  | Corrected, no cover | 16 | printed |

- **Refusal output.** Both name every id on stderr. Under `--json` the toolkit prints
  nothing on stdout.
- **One-card verbs.** `mk decode`, `inspect`, `verify` and `address` refuse the mix (2,
  `ChunkSetIdMismatch`).
- **N8 vector.** The new N8 row (2/2) is right: `gen_mk_vectors.rs:661-668` builds it from
  chunk 0 of card 0x12345 and chunk 1 of card 0x67890, both 2-chunk cards.
- **Measured (toolkit tests),** `cargo test --test all` filtered, 31 passed with
  `MK_BIN=` set, so mk was cross-checked too. This includes
  `c1_mk_negative_vectors_repair_as_mk_does`, `ruling_3_…`,
  `bundle_retains_row19_inline_vs_slot_path_mismatch_refusal` (now `code(2)`) and the
  `verify_bundle_multi_cosigner_mk1` file.
- **`path-mismatch` / `path-conflict` references.**
  - Only bundle.rs:2845 emits `path-conflict`, and it maps to INVALID.
  - restore.rs:4278, :4848 and :4982 still emit `path-mismatch`, which error.rs:724 maps to
    NO (1).
  - No `"path-mismatch"` string appears in mnemonic-gui, refugium-wallet or
    docs/manual-gui.
  - The remaining mentions in design docs are historical.
  - The JSON `kind` rename is in the CHANGELOG.

### 4. GUI 8e2a973

- **Method.** I emulated the six regexes in Python; the Rust and Python `regex` semantics
  are identical for these constructs, and `\d` is Unicode in both. I ran the emulation
  against `target/debug/mnemonic gui-schema --classify-descriptor` on 34 shapes. Inputs
  the toolkit accepts include `sh(wpkh(@0/<0;1>/*h))`, `/*h`, `/*'`, `/**`, `<0;1;2>`,
  `<1;0>`, prefix and suffix origins, an uppercase fp, a bare-fp prefix, `@00` and
  `sh(pkh(@0))`. Other inputs are toolkit parse errors (exit 2): nested
  `sh(sh(wpkh))`/`sh(wsh(wpkh))`, a non-`@` xpub key, `@1` without `@0`, both origins, and
  `<0;0>`.
- **Result.** The GUI and the toolkit agree on every shape the toolkit accepts. The other
  shapes are parse errors in the toolkit, so the GUI's verdict there changes nothing.
- **One divergence, older than this commit.** A descriptor with a valid `#checksum`
  disagrees: `sh(wpkh(@0))#209apqrz` is canonical in the toolkit and NonCanonical in the
  GUI. The pkh/wpkh/tr arms have done the same since before this commit, because their
  regexes are `$`-anchored. Worth one FOLLOWUP: strip a trailing `#[0-9a-z]{8}` before
  matching.
- I did not build or run the GUI crate.

### 5. Spec 19f6e23 and c7f1f16

- **Constellation spec.** §2b, R9 (`:606-607`), §7d (`:878-893`) and the §10 table
  (`:1285-1300`) match what was built, apart from M3 and M4.
- **SPEC_mt.** I diffed the two copies (`c7f1f16` against mt `9c85c94`). The §2b hunks are
  identical in both: the §1.1 duplicate rule, the one-liner paragraph, "Offline means",
  (b1), the `verify --offline` line, the flag table row and the exit-status paragraph.
  Every other difference is the known drift (FOLLOWUP `spec-mt-resync-exit-codes`).
  c7f1f16 says it did not apply the (b) `--json` notice hunk, and mt's copy has that
  paragraph.

### Manuals (09f5f8e0)

- **Built behaviour.** Measured or read for each: mk and toolkit repair (2 for covering
  groups, decision 7 otherwise), bundle path 2 with kind `path-conflict`, and the
  Appendix J rows for mt (`--offline` stays 0; with `--bitcoin-cli` or on `verify`, 64;
  re-cut 16) and for mk and the toolkit.
- **Exception: M5.** The fingerprint row's stderr text does not match the build.

## Measured versus read

| Item | Measured | Read only |
| --- | --- | --- |
| 1 | Decoder-equivalence unit test. `sysw_composer_records` (27 passed). The `/dev/null` test failing at HEAD, its assertion, and the 0600 state of `/dev/null`. | That the test is independent of dddff0c (dddff0c^ was not built). The ms-codec copies (M1). The record stream (M2). |
| 2 | 7 targeted `exit_codes` tests. The two garbage-input 64s. The unreadable spare at 0 with the "re-cut" text. | That no other spawn site exists. |
| 3 | 12-shape matrix on both binaries and `--json`. One-card verbs. Toolkit targeted tests with `MK_BIN`. | `path-mismatch` references. Manual diffs. |
| 4 | Python emulation of the regex table against the real toolkit classifier. | The GUI crate itself (not built). |
| 5 | Diff of the two SPEC_mt copies. | Constellation spec text. |
