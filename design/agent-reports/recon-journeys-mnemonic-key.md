# Recon — round-trip journeys that EXIST in `mnemonic-key`

**Scope:** read-only inventory of `/scratch/code/shibboleth/mnemonic-key` against
`design/DRAFT_round_trip_journey_definition.md` §7's unit schema. Per operator
ruling §8.3, this catalogues what exists; it does not propose what should exist.

## What I actually ran

`cd /scratch/code/shibboleth/mnemonic-key`, then: `git log --oneline -10`; `ls`
and `find` over `crates/`, `design/`, `.github/`, and repo-root for scripts;
`grep -ril "journey"` and `grep -ril "round.trip"` across the repo (source +
design docs); `Read` on every test file in `crates/mk-codec/tests/` and
`crates/mk-cli/tests/` that mixes `encode(`/`decode(` calls (found by `grep -l
encode ... | xargs grep -l decode`) or that looked journey-shaped by name
(`cli_derive.rs`, `cli_address.rs`, `cli_address_bip_vectors.rs`,
`cli_slip132.rs`, `cli_mk1_repair_reverify.rs`, `cli_repair.rs`); `grep -rn
"Command::new\|Command::cargo_bin"` to confirm no test spawns any binary other
than `mk`; `grep -n "round.trip|journey"` and targeted `Read`s of
`design/FOLLOWUPS.md` and `design/SPEC_test_hardening_T4_mk_external_oracle.md`
to check provenance claims against this repo's own recorded history; `(grep -rn
"#\[ignore\]" crates/*/tests/*.rs || echo NONE FOUND)`; `ls
crates/mk-cli/src/cmd/v0.1.json` (confirms an old duplicated-corpus file is
gone); and `cargo test -p <crate> --test <file> -- --nocapture` individually for
every file discussed below, plus one final `cargo test --workspace` for the
whole-suite tally. All pass/fail counts and test names below are pasted from
those runs, not inferred from source.

## Headline findings (read this first)

1. **No journey in this repo satisfies §4.** §4 requires a structural equality
   *and* a functional equality in the *same* journey. Every candidate I found
   is one or the other, never both. The codec-level (T1) round trips are
   structural-only (`assert_eq!(recovered, card)` — full `KeyCard` struct
   equality); the CLI-level (T2) address/derive journeys are functional-only
   (address or child-xpub/fingerprint match). Nothing in the suite decodes a
   round-tripped card back to bytes/struct *and* derives/checks a
   funds-relevant identity from it in one test.
2. **Every journey in this repo is custodial, never generative.** `mk1`'s
   origin artifact is always an already-existing xpub (an artifact in hand),
   never entropy or a BIP-39 phrase — `mk-codec` has no encode-from-seed path.
   The generative half (seed → xpub) necessarily lives in a different repo.
   Per §2, "an audit that finds only custodial coverage has found a hole, not
   a pass" — but here the hole is structural to what this crate does, not a
   coverage gap this repo can close; it is the cross-repo blind spot ruling §8
   #3 already names.
3. **No test in this repo prints a non-coverage statement (§6).** I ran every
   journey discussed below with `--nocapture`; none emit what they did *not*
   cover. The stated non-coverage that exists (e.g. "only 1 of 8 SLIP-0132
   arms is oracle-anchored") lives in `design/FOLLOWUPS.md`, not in the
   journey's own output — that satisfies documentation but not §6's literal
   requirement.
4. **No test spawns any binary other than `mk`.** Confirmed by grep across all
   `crates/*/tests/*.rs`. So every CLI-level test tops out at **T2**; nothing
   in this repo reaches T3 (emulator/device flow) or T4 (engraving) — expected
   for this repo (it has no device/engraving surface), stated for completeness.
5. **Naming collision to flag, not a defect:** this repo's own design docs use
   `T2`/`T4-a`/`T4-b`/`T4-c` as *test-hardening tier labels*
   (`design/SPEC_test_hardening_T4_mk_external_oracle.md`,
   `design/RECON_T4_mk_external_oracle.md`) predating and unrelated to the
   round-trip-journey-definition doc's `T1`..`T4-metal` *loop-extent* tiers. A
   reader cross-referencing "T4" between the two documents will get the wrong
   idea; they do not mean the same thing.
6. **No `#[ignore]`d test anywhere** in `crates/*/tests/*.rs` (`grep -rn
   "#\[ignore\]"` → no matches). Every gate discussed below has executed — I
   ran it. `cargo test --workspace` is fully green: mk-cli 162 passed / 0
   failed across 17 integration-test files; mk-codec's non-`gen-vectors`
   integration suite likewise all green (round_trip 3, vectors 3,
   proptest_roundtrip 4, canonical_payload 4, xpub_compact_external_oracle 1,
   plus the adjacent validation suites). No red suite, no skip-that-passes
   found.

---

## Journeys found

### J1 — `mk-codec` KeyCard structural round trip (T1, codec-only)

| field | value |
| --- | --- |
| name | `mk-codec::round_trip` (3 variants) |
| kind | custodial (origin = an in-process-constructed `KeyCard`, not entropy) |
| tier | T1 |
| origin artifact | a synthetic `KeyCard` built inline in the test (test-local xpub, stub, fingerprint, path) |
| invocations | `mk_codec::encode()` → `mk_codec::decode()`, same process, no CLI, no cross-repo call |
| structural assertion | `assert_eq!(recovered, card)` — full struct equality (present) |
| functional assertion | **NONE** — no address, no derived fingerprint, no wallet-id check |
| one command | `cargo test -p mk-codec --test round_trip` — **ran, 3 passed, 0 failed** |
| stated non-coverage | none printed |

Files/tests: `crates/mk-codec/tests/round_trip.rs` —
`round_trip_single_xpub_one_policy_id_stub`,
`deterministic_round_trip_with_explicit_chunk_set_id`,
`round_trip_fingerprint_omitted`. Per §8 ruling #4, I treat these three as one
base journey with variations (grouping flags, fingerprint presence), not three
journeys.

**FINDING:** missing functional assertion (§7's explicitly named finding
class). The same shape, same missing half, recurs in three more places I
verified by execution:
- `crates/mk-codec/tests/proptest_roundtrip.rs::keycard_roundtrip` —
  property-based (`proptest`) generalization of J1 over arbitrary `KeyCard`
  values; `cargo test -p mk-codec --test proptest_roundtrip` → 4 passed
  (includes 3 panic-freedom fuzz-style cells, not round trips).
- `crates/mk-codec/tests/vectors.rs::every_vector_round_trips` — same
  structural-only pattern but walks the SHA-pinned corpus
  `src/test_vectors/v0.1.json` (pin `V0_1_SHA256` checked, corpus asserted
  `>=18` clean / `>=22` negative vectors); `cargo test -p mk-codec --test
  vectors` → 3 passed. This one additionally checks
  `encode_bytecode(input) == canonical_bytecode_hex` byte-for-byte against the
  pinned corpus — still no address/identity check.
- `crates/mk-codec/tests/canonical_payload.rs` — same pattern at the
  pre-chunking public API (`KeyCard::canonical_payload_bytes` /
  `from_canonical_payload_bytes`); `cargo test -p mk-codec --test
  canonical_payload` → 4 passed.

### J2 — `mk-cli`-housed but T1 (not T2) codec round trip

| field | value |
| --- | --- |
| name | `encode_decode_round_trip` |
| kind | custodial |
| tier | **T1**, despite living in the `mk-cli` test crate |
| origin artifact | test-local `KeyCard` (V1 fixture constants) |
| invocations | `mk_codec::encode()` → `mk_codec::decode()` called **directly as a library**, in-process — **no `mk` binary is spawned** |
| structural assertion | per-field equality (`policy_id_stubs`, `origin_fingerprint`, `origin_path`, `xpub`) |
| functional assertion | none |
| one command | `cargo test -p mk-cli --test round_trip encode_decode_round_trip` — ran (part of the 5-test file), passed |
| stated non-coverage | none |

File: `crates/mk-cli/tests/round_trip.rs:22-38`.

**FINDING (location/tier mislabeling risk):** this test lives in the file most
likely to be read as "the mk-cli round trip," but it never touches the CLI
binary. A reader inferring T2 coverage from the file's name/location would be
wrong. Not a defect in the test itself — flagged because §3 says a tier claim
must never be read as a higher one, and file placement here invites exactly
that misreading.

### J3 — `mk-cli` from-md1 derivation (T2-partial, hand-transcribed golden)

| field | value |
| --- | --- |
| name | `from_md1_derivation` |
| kind | custodial |
| tier | T2 (one real CLI process spawn) |
| origin artifact | a hardcoded md1 string literal `PKH_BASIC_MD1` embedded in the test file (comment claims provenance: "descriptor-mnemonic's `pkh_basic.phrase.txt`," refreshed against md-codec v0.34.0) |
| invocations | `mk encode --xpub <V1_XPUB> --origin-fingerprint <V1_FP_HEX> --origin-path <V1_PATH> --from-md1 <PKH_BASIC_MD1> --group-size 0` (1 CLI spawn) → `mk_codec::decode()` in-process (library call, **not** `mk decode`) |
| structural assertion | **NONE** — there is no "original card" to compare the round trip against; the md1 literal is the origin and nothing decodes/re-derives it independently within this repo |
| functional assertion | `card.policy_id_stubs[0] == EXPECTED_STUB` (`[0x55, 0x9e, 0x64, 0xb2]`), a WalletDescriptorTemplateId-class check (§4 names this identity type explicitly) |
| one command | `cargo test -p mk-cli --test round_trip from_md1_derivation` — ran, passed |
| stated non-coverage | none |

File: `crates/mk-cli/tests/round_trip.rs:58-93`.

**FINDING:** the doc comment says `EXPECTED_STUB` is "a frozen literal computed
ONCE, out-of-band" and explicitly forbids the test body from recomputing it
(to avoid tautology) — this is exactly §7's named finding class: *"a path
whose 'expected' values were transcribed by hand from a run nobody has
repeated."* This repo already tracks a related, now-resolved history on this
same test: `design/FOLLOWUPS.md:139-145`
(`from-md1-derivation-wire-version-skew`) records that this cell silently
failed for 3 release cycles (`WireVersionMismatch { got: 0 }`) before the
md1 literal was refreshed in mk-cli v0.4.1 — i.e. a hand-pinned origin literal
in this exact test previously rotted undetected. **I confirmed the cell passes
today** (ran above), so that specific incident is closed, but the mechanism
that let it happen (a hand-pinned literal nothing re-derives) is unchanged.

### J4 — `mk derive` vs `bitcoin`-crate independent derivation (T2, functional-only)

| field | value |
| --- | --- |
| name | `relative_path_derivation_matches_bitcoin` (+ `index_sugar_equals_path_m0`, `multisig_card_is_allowed_for_derive` as variants) |
| kind | custodial |
| tier | T2 (1 CLI spawn) |
| origin artifact | test-local `KeyCard` (`V2_84_MAIN` xpub), encoded in-process via `mk_codec::encode()` |
| invocations | `mk derive <card...> --path m/0/5 --json` |
| structural assertion | **NONE** |
| functional assertion | CLI's `child_xpub` / `child_fingerprint` JSON fields compared against `bitcoin::bip32::Xpub::derive_pub` called independently in the test process — a genuinely different code path from mk-cli's own derivation |
| one command | `cargo test -p mk-cli --test cli_derive` — ran, **8 passed, 0 failed** |
| stated non-coverage | none |

File: `crates/mk-cli/tests/cli_derive.rs`. `child_xpub_roundtrips_through_encode`
(same file) is a smoke test only — checks CLI exit-success and that stdout
contains the literal substring `"mk1"` — it does **not** decode-and-compare,
so it does not close J4's structural gap.

**FINDING:** missing structural assertion (the mk1 card side of this journey
is only ever encoded, never decoded back and compared).

### J5 — `mk address` vs hand-pinned literals claimed "toolkit-computed" (T2, functional-only, provenance gap — self-acknowledged)

| field | value |
| --- | --- |
| name | `account_84_first_address_matches_toolkit` (+ `account_84_p2tr_override_matches_toolkit`, `account_44_default_p2pkh_matches_toolkit` as address-type variants) |
| kind | custodial |
| tier | T2 (1 CLI spawn) |
| origin artifact | test-local `KeyCard` xpub constants (`V2_84_MAIN`, `V9_44_MAIN`) |
| invocations | `mk address <card...> --count 1 [--address-type p2tr]` |
| structural assertion | **NONE** |
| functional assertion | rendered address string contains a hardcoded literal constant (e.g. `V2_84_M0_0_P2WPKH`) |
| one command | `cargo test -p mk-cli --test cli_address` — ran, **15 passed, 0 failed** |
| stated non-coverage | none |

File: `crates/mk-cli/tests/cli_address.rs`. Its own doc comment (lines 4-6)
claims: *"expected addresses are independently computed by the toolkit's
`mnemonic convert --to address` (cross-tool, not self-referential)."*

**FINDING (verified, not inferred):** I grepped this whole repo for any script,
`include!`, or runtime invocation that could regenerate or re-check these
literals against the toolkit — `grep -rn "independently computed|cross-tool|
regenerat"` across `*.rs`/`*.sh` finds no such mechanism. Nothing in this repo
re-runs the claimed cross-tool computation; the constants are hand-pasted
strings whose only evidence is the comment's word. This is §7's named finding
("transcribed by hand from a run nobody has repeated"), **and it is not a
discovery — this repo's own design doc already says so, more sharply than I
would**: `design/SPEC_test_hardening_T4_mk_external_oracle.md:4` frames this
exact test's constants as *"a regression pin, not external validation"* and
warns that `mk`'s `derive_support.rs` was "copy-derived from" the toolkit's
`address_search.rs`, so *"a semantic mistake copied into BOTH... is invisible
to that cross-check."* That gap analysis is precisely why J6 below exists.

### J6 — `mk encode` → `mk address` vs published BIP-84/86 vectors (T2, the strongest journey in the repo)

| field | value |
| --- | --- |
| name | `bip84_published_vector_matches_verbatim`, `bip86_published_vector_matches_verbatim` |
| kind | custodial |
| tier | T2 — **two** real CLI process spawns, separate processes, real exit codes |
| origin artifact | published BIP-84/BIP-86 account zpub/xpub, literal constants sourced from the BIP text ("Re-verified character-for-character against the live BIP text 2026-07-10") |
| invocations | `mk encode --xpub <published zpub/xpub> --origin-path <m/84'\|86'/0'/0'> --policy-id-stub deadbeef --privacy-preserving --group-size 0` (CLI #1) → `mk address <all emitted mk1 lines> --count 2 --chain both` (CLI #2) |
| structural assertion | **NONE** — the mk1 card is never decoded and compared for structural (byte/xpub) equality in this test; it goes straight from encode to rendered address |
| functional assertion | rendered **receive AND change** addresses (both satisfied — §4's "change addresses are not optional" clause is met) match the BIP's own published first-address vectors, verbatim |
| one command | `cargo test -p mk-cli --test cli_address_bip_vectors` — ran, **2 passed, 0 failed** |
| stated non-coverage | none printed in test output |

File: `crates/mk-cli/tests/cli_address_bip_vectors.rs`. This repo's own SPEC
(`design/SPEC_test_hardening_T4_mk_external_oracle.md`) documents that this
test's account key MUST be ingested via the real `mk encode --xpub` SLIP-0132
normalization path (not a library shortcut) specifically to avoid the J5
provenance gap, and names the exact mutation this journey is designed to catch
(swapping `84'`/`49'` purpose arms or the p2wpkh/p2tr script-builder branch).

**This is the closest thing to a full §4 journey in the repo** — real
multi-process CLI chain, external (not self-referential) functional oracle,
both chains checked. **Its one gap against §4 is the missing structural half**:
nothing here decodes the emitted mk1 card and asserts it reproduces the
original xpub/path/stub bytes — the journey only proves the *address render*
is correct, not that the *card itself* round-trips.

### J7 — SLIP-0132 zpub normalization structural round trip (T2, structural-only)

| field | value |
| --- | --- |
| name | `encode_accepts_zpub_with_matching_path`, `published_bip84_zpub_normalizes_and_matches_own_version_swap` |
| kind | custodial |
| tier | T2 (1 CLI spawn + 1 in-process library decode) |
| origin artifact | a zpub — either a locally re-versioned test constant (via a local `to_slip132()` helper using `bitcoin::base58`), or, in the second test, the actual **published BIP-84 zpub** literal |
| invocations | `mk encode --xpub <zpub> --origin-path ... --group-size 0` → `mk_codec::decode()` in-process |
| structural assertion | decoded card's `.xpub` field equals a reference card's `.xpub` field (built by re-versioning/normalizing the same key another way) |
| functional assertion | **NONE** |
| one command | `cargo test -p mk-cli --test cli_slip132` — ran, **9 passed, 0 failed** |
| stated non-coverage | **documented, but not in the journey's own output**: `design/FOLLOWUPS.md:429` (`mk-slip0132-byte-parity-test-self-referential`, status `resolved`) states verbatim: *"only the zpub arm (1 of 8 SLIP-0132 entries) is published-vector-anchored; ypub/Ypub/Zpub + testnet arms remain self-referential."* This satisfies documentation but is a §6 violation in the strict sense — the running test prints no such statement itself. |

File: `crates/mk-cli/tests/cli_slip132.rs`.

### Not counted as journeys (checked, explicitly excluded, with reason)

- `crates/mk-codec/tests/xpub_compact_external_oracle.rs` — a genuine external
  oracle (from-scratch base58 decoder, zero shared code, checked against
  published BIP-32 test vector 1; `cargo test -p mk-codec --test
  xpub_compact_external_oracle` → 1 passed) but it is a **one-shot field
  extraction check** on `XpubCompact::from_xpub`, not an encode→decode round
  trip of an mk1 card. No origin-to-destination loop exists here to apply the
  §7 schema to.
- `crates/mk-cli/tests/cli_mk1_repair_reverify.rs` (10 passed) and
  `crates/mk-cli/tests/cli_repair.rs` (8 passed) — `mk repair`'s BCH
  tri-state re-verify tests. These validate exit-code/advisory behavior on
  deliberately corrupted chunks; they do not assert a funds-relevant identity
  match between an origin and a recovered artifact, so they don't fit the §7
  unit schema. Both green, both real CLI invocations, no `#[ignore]`.
- `crates/mk-cli/tests/{decode_grouped,encode_chunk_set_id,
  encode_grouping_flags,template_id_stub,cli_output_class,
  version_help_exit_codes,gen_man,gui_schema}.rs` — narrow single-feature CLI
  tests (display grouping, advisory text, help/exit-code contracts, GUI schema
  shape). `template_id_stub.rs` repeats J3's "frozen literal, computed once
  out-of-band" pattern for form-aware stub derivation (its own doc comment
  says so explicitly) but is a unit test of one function, not a round trip.
  None of these traverse an origin→destination loop; excluded on that basis,
  not reviewed for quality (out of scope per the brief).

## Anti-requirement (§5) checks

- **Reads an intermediate nothing writes:** none found. Every journey above
  either builds its origin artifact inline in the same test process or spawns
  `mk encode` and captures its stdout directly (no file intermediate is read
  that isn't produced in the same run). One historical instance of a related
  defect *did* exist and is resolved: `design/FOLLOWUPS.md:147-155`
  (`mk-cli-vector-corpus-inlined`) records that `mk-cli` used to `include_str!`
  a stale **working copy** of the vector corpus at
  `crates/mk-cli/src/cmd/v0.1.json`, separate from the canonical file — I
  confirmed by `ls` that this file no longer exists; mk-codec 0.3.0 promoted
  the corpus to a shared `pub const` and the duplicate was dropped.
- **Asserts against a value the journey itself produced, no independent
  source:** J1/J2's `decode(encode(card)) == card` is the sanctioned
  round-trip shape (comparing against the test author's literal input, not
  something the code-under-test produced), not a violation. J5's literals
  are the closest candidate for concern — I could not confirm or refute
  independence from inside this repo (no re-run mechanism exists here to
  check), so I report this as **UNKNOWN, not a firm violation** — consistent
  with this repo's own SPEC treating it as an open provenance gap rather than
  a proven-wrong value.
- **A skipped step that passes instead of failing:** none found — `grep -rn
  "#\[ignore\]" crates/*/tests/*.rs` returned no matches.
- **A gate that has never executed:** none found among the journeys above — I
  ran every one of them this session and pasted the pass counts.
- **Empty output is not proof of absence:** applied throughout — every
  negative claim above (no functional assertion, no structural assertion, no
  cross-repo invocation, no `#[ignore]`) was checked by running the actual
  command or grep and reading real output, not inferred from silence.

## Known blind spot (per ruling §8 #3, restated as required)

This recon is a single-repo sweep. It cannot see gaps *between* repos — in
particular: whether the md1/toolkit-side literals referenced by J3/J5 actually
match a real run of the tools they claim provenance from, and whether the
generative half (seed → xpub) that would make an `mk1` journey end-to-end
actually composes cleanly with what exists here. That composition question is
explicitly out of this recon's scope per the operator ruling and is not
assessed here.

## File index (absolute paths)

- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/tests/round_trip.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/tests/proptest_roundtrip.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/tests/vectors.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/tests/canonical_payload.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/tests/xpub_compact_external_oracle.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-cli/tests/round_trip.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-cli/tests/cli_derive.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-cli/tests/cli_address.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-cli/tests/cli_address_bip_vectors.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-cli/tests/cli_slip132.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-cli/tests/cli_mk1_repair_reverify.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-cli/tests/cli_repair.rs`
- `/scratch/code/shibboleth/mnemonic-key/design/FOLLOWUPS.md` (lines 139-145,
  423-431, 147-155 cited above)
- `/scratch/code/shibboleth/mnemonic-key/design/SPEC_test_hardening_T4_mk_external_oracle.md`
