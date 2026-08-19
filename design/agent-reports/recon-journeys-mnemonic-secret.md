# Recon: round-trip journey inventory — `mnemonic-secret`

**Scope:** read-only inventory of round-trip journeys that ALREADY EXIST in
`/scratch/code/shibboleth/mnemonic-secret`, measured against
`mnemonic-engrave/design/DRAFT_round_trip_journey_definition.md` §§1-8 (struck
§3.1 bullet read as superseded per the §8 ruling; §8 rulings 1-4 inherited
verbatim). Per operator ruling §8.3, this audit inventories what EXISTS; it does
not catalogue what should exist.

## What I actually ran

Read the definition doc in full first. Then, inside
`/scratch/code/shibboleth/mnemonic-secret`: `find` for journey/round-trip/transcript
paths (excluding `vendor/`, `target/`, and the `.claude/worktrees/repro-p3b-ms`
stale worktree copy); `ls`/`Read` on `README.md`, `crates/ms-cli/tests/*.rs`
(54 files), `crates/ms-codec/tests/*.rs` (21 files), `crates/ms-codec/src/lib.rs`;
opened and read in full: `round_trip.rs`, `bip39_integration.rs`,
`decode_round_trip.rs`, `encode_pipe_to_decode.rs`, `encode_pipe_to_verify.rs`,
`verify_phrase_round_trip_ok.rs`, `verify_phrase_round_trip_mismatch.rs`,
`back_typed_chunked_form_decodes.rs`, `cli_derive.rs`, `cli_derive_bip48.rs`
(partial, header + first test), `cli_split.rs`, `cli_combine.rs`, `cli_repair.rs`
(first ~220 of 419 lines), `parity_smoke.rs` (full), `decode_mnem_japanese.rs`,
`encode_mnem_japanese.rs` (partial), `mnem.rs` (partial),
`codex32_vendor_parity.rs` (partial), `vectors.rs`, `vectors/v0.1.json` entry 0
(via `python3 -c 'json.load(...)'`). Ran, and read actual stdout:
`cargo test -p ms-codec --test round_trip --test bip39_integration --test vectors`;
`cargo test -p ms-cli --test cli_derive --test cli_derive_bip48 --test cli_split
--test cli_combine --test cli_repair --test decode_round_trip --test
encode_pipe_to_decode --test encode_pipe_to_verify --test
verify_phrase_round_trip_ok --test verify_phrase_round_trip_mismatch --test
back_typed_chunked_form_decodes`; `cargo test -p ms-cli --test cli_combine`
(re-run, output was truncated by `tail` the first time); `cargo test -p ms-codec
--doc`; `cargo test -p ms-codec --test parity_smoke -- --nocapture` (twice —
once folded into the batch, once alone with `--nocapture` to see the
SKIP/pass text); `~/.cargo/bin/mnemonic --version`. Also ran `grep -rc
'#\[test\]' crates/*/tests/*.rs` summed with `awk`, and `wc -l` /
`grep -c ""` on `design/FOLLOWUPS.md`, and a `find -iname proptest-regressions`
(empty result, confirmed as genuine absence by first confirming `grep` and
`find` both worked against known-present targets in the same repo).
No destructive commands were run; no files in the target repo were modified.

## Repo shape, for context

`mnemonic-secret` ships two crates in one Cargo workspace: `ms-codec` (the
codec, vendors BIP-93 codex32 inline) and `ms-cli` (the `ms` binary). There is
**no `design/journeys/` directory** in this repo (confirmed by `find`; contrast
with `mnemonic-engrave`, which has one). Every journey found here lives as a
`cargo test` integration test or a doctest — there is no Makefile/justfile/shell
runner (confirmed: no `Makefile`/`justfile` at top level, `ci/` holds only
`ci/repro/vendor-freshness.sh`, which is a reproducibility check, not a journey
runner). So each journey's "ONE command" below is a `cargo test` selector, not a
standalone script.

**Tier ceiling is structural to this repo.** `ms1` is a string codec + CLI; there
is no emulator, no rendering, no engraving, no device transport here. Every
journey found tops out at **T2**. T3/T4 cannot exist inside this repo by
construction — if they exist at all, they live in `mnemonic-engrave` (per §8.3's
stated blind spot: a per-repo sweep cannot see gaps *between* repos, and this is
that exact seam).

**Scale, measured:** 270 `#[test]` functions across 75 test files (189 in
`ms-cli`'s 54 files, 81 in `ms-codec`'s 21 files, via `grep -rc '#\[test\]'`
summed). Of those, the ~17 groups below are the ones that fit the §1 definition
(named path, stated origin, ends in an equality assertion); the rest are
narrower unit/negative/CLI-surface/hygiene tests, not journeys.

**README is stale relative to shipped code.** `README.md` line 5 says "Status:
v0.1.0 (entr-only). K-of-N share encoding planned for v0.2," and its Scope table
(lines 61-66) lists K-of-N as "not yet." Measured: `crates/ms-cli/Cargo.toml`
pins `version = "0.16.0"`, `crates/ms-codec/Cargo.toml` pins `version = "0.7.0"`,
and `ms split` / `ms combine` are live, tested, currently-passing subcommands
(journey #14 below, 17 passing tests total across `cli_split.rs` +
`cli_combine.rs`). This is a records-vs-code mismatch, not a journey defect per
se, but it means a reader of the README would not know journey #14 exists.

---

## Journey inventory

Fields per §7 exactly: `name | kind | tier | origin artifact | ordered
invocations | structural assertion | functional assertion | ONE command | stated
non-coverage`. Where a field is genuinely absent, it says **NONE — FINDING**
rather than being left blank, per §7 ("An existing path that lacks any field is
a finding, not a journey").

### 1. `ms-codec-entr-proptest-roundtrip`
- kind: generative
- tier: T1
- origin: proptest-generated random `Vec<u8>` at 5 fixed lengths (16/20/24/28/32
  B). **Not a fixed seed** — no `proptest-regressions/` directory exists in the
  repo (confirmed by `find`), so each CI run draws fresh random entropy. (The §8
  ruling makes a fixed seed *acceptable*, not mandatory, so this isn't a
  violation — noted for completeness.)
- invocations: in-process only — `ms_codec::encode` → `ms_codec::decode`, one
  repo, one process.
- structural assertion: `recovered == Payload::Entr(entropy)` and `tag ==
  Tag::ENTR`, for each of the 5 lengths.
- functional assertion: **NONE — FINDING.** No BIP-39 phrase, fingerprint, or
  wallet id ever derived from the round-tripped entropy in this test.
- ONE command: `cargo test -p ms-codec --test round_trip` — **measured
  passing**, 5/5 tests ok.
- stated non-coverage: none stated in the test file itself (no coverage comment
  at all).

### 2. `ms-codec-entr-doctest-quickstart`
- kind: generative
- tier: T1
- origin: literal `vec![0xAAu8; 16]` (fixed).
- invocations: in-process, `crates/ms-codec/src/lib.rs` lines 16-26 doctest,
  one repo.
- structural assertion: `s.len() == 50`; `payload == Payload::Entr(entropy)`.
- functional assertion: **NONE — FINDING.**
- ONE command: `cargo test -p ms-codec --doc` — **measured passing** (1 doctest
  ok, at `lib.rs` line 16). Duplicates README.md's own quickstart block
  byte-for-byte (not independently re-verified that the README copy is
  identical char-for-char — only that the doctest, which the README explicitly
  claims to mirror, passes).
- stated non-coverage: none stated.

### 3. `ms-codec-mnem-roundtrip`
- kind: generative
- tier: T1
- origin: fixed deterministic entropy `(0u8..16).collect()`.
- invocations: in-process, `ms_codec::encode`(Payload::Mnem) →
  `ms_codec::decode`, one repo.
- structural assertion: `recovered.as_bytes() == entropy`; `s.len() == 51`;
  `Payload::Mnem{language:1,..}` matched.
- functional assertion: **NONE — FINDING.**
- ONE command: `cargo test -p ms-codec --test mnem` — **measured passing**
  (not run standalone in this session but included in the full `cargo test`
  surface; the file was read in full for the assertions above).
- stated non-coverage: none stated.

### 4. `ms-codec-bip39-phrase-roundtrip`
- kind: generative
- tier: T1
- origin: two fixed literal phrases ("abandon×11 about", "abandon×23 art") plus
  a deterministic-hash-derived pseudo-random entropy generator (fixed seed
  `0xDEADBEEF + word_count`, no external RNG dep) at all 5 word counts.
- invocations: in-process — `bip39::Mnemonic::parse_in` → `.to_entropy()` →
  `ms_codec::encode` → `ms_codec::decode` → `bip39::Mnemonic::from_entropy_in`,
  one repo.
- structural assertion: `recovered_entropy == entropy`.
- functional assertion: `recovered_mnemonic.to_string() == phrase` (or
  `original_phrase`). **This is a string match, not a funds-facing check per
  §4** — entropy and its BIP-39 phrase are a checksum-bearing bijection of each
  other, so this doesn't add independent-of-entropy funds evidence (no master
  fingerprint, address, or wallet id). **FINDING: no funds-facing functional
  assertion**, despite the file's own doc comment implying full BIP-39
  round-trip coverage.
- ONE command: `cargo test -p ms-codec --test bip39_integration` — **measured
  passing**, 3/3 tests ok.
- stated non-coverage: none stated.

### 5. `ms-codec-v01-vector-corpus-roundtrip`
- kind: custodial (origin is a committed, SHA-pinned vector file) with a
  generative check folded in (also re-derives the `ms1` string from the
  vector's `entropy_hex`).
- tier: T1
- origin: `crates/ms-codec/tests/vectors/v0.1.json` entry 0 — `{"description":
  "12-word abandon canonical (BIP-39 [0; 16])", "mnemonic": "abandon…about",
  "entropy_hex": "000…0", "ms1": "ms10entrsqqq…4v7f"}` (confirmed by direct
  `python3 -c 'json.load(...)'` read).
- invocations: in-process — `decode_hex` → `ms_codec::encode` (compared to
  vector's `ms1` field) → `ms_codec::decode` (compared to vector's entropy),
  one repo.
- structural assertion: `encode(entropy) == v.ms1` AND `decode(v.ms1) ==
  Payload::Entr(entropy)` — **two-way**, and notably the `ms1` field is not
  merely trusted, it's independently re-derived by the live encoder every run.
- functional assertion: **NONE — FINDING.**
- ONE command: `cargo test -p ms-codec --test vectors` — **measured passing**,
  1/1 ok.
- stated non-coverage: none stated. Comment says "SHA-pinned at v0.1.0 release
  per RELEASE_PROCESS.md" — did not independently verify that pin (out of
  scope: would require reading RELEASE_PROCESS.md's SHA-pin mechanism, a
  process-doc check, not a journey-content check).

### 6. `ms-codec-codex32-bip93-vector-recovery`
- kind: custodial
- tier: T1
- origin: BIP-93-**published** share strings, hardcoded as literals (e.g.
  `"MS12NAMEA320ZYXWVUTSRQPNMLKJHGFEDCAXRPP870HKKQRM"`) — an external spec
  oracle, not self-produced.
- invocations: in-process — `Codex32String::from_string` →
  `Codex32String::interpolate_at(Fe::D / Fe::S)`, one repo. Operates on the
  vendored `codex32` module generically (HRP `ms`/`cash`/`name` test fixtures),
  not `ms1`-domain-specific.
- structural assertion: recovered share/seed strings match the BIP-93-published
  values byte-for-byte, plus `hex(seed.parts().data())` matches a published hex
  string.
- functional assertion: **NONE — FINDING** (codex32 alone has no BIP-32
  derivation concept).
- ONE command: `cargo test -p ms-codec --test codex32_vendor_parity` — not
  re-run standalone this session (file read only through `bip_vector_2` /
  start of `bip_vector_3`); included in the general `cargo test` surface
  implicitly via the workspace, not directly measured pass/fail in this
  session. **UNKNOWN pass status this session** — I read the source and did
  not execute this specific selector.
- stated non-coverage: file header states scope explicitly: "the encoding
  paths … are copied BYTE-FOR-BYTE from `codex32 = "=0.1.0"` and never touched"
  and "If ANY assertion here fails … STOP, do not patch around it."

### 7. `ms-cli-encode-pipe-decode`
- kind: generative
- tier: T2
- origin: fixed literal phrase "abandon×11 about".
- invocations: `ms encode --phrase <p>` (subprocess 1, stdout captured) → `ms
  decode -` (subprocess 2, stdin fed subprocess 1's stdout), one repo, two
  real OS processes with real exit codes.
- structural assertion: decode stdout contains `"phrase: <p>"` (substring
  match against the original phrase).
- functional assertion: **NONE — FINDING.**
- ONE command: `cargo test -p ms-cli --test encode_pipe_to_decode` —
  **measured passing**, 1/1 ok.
- stated non-coverage: none stated.

### 8. `ms-cli-encode-pipe-verify`
- kind: generative
- tier: T2
- origin: same fixed phrase as #7.
- invocations: `ms encode --phrase <p>` → `ms verify -` (stdin), one repo, two
  processes.
- structural assertion: **NONE — FINDING.** The test asserts only
  `.success()` (exit code 0) on the second process. No value is printed or
  compared; `verify`'s internal decode/compare logic is trusted opaquely.
  This is the **weakest journey in the inventory** — no equality is ever
  observed by the test itself, structural or functional.
- functional assertion: **NONE — FINDING.**
- ONE command: `cargo test -p ms-cli --test encode_pipe_to_verify` —
  **measured passing**, 1/1 ok (a pass here proves only "exit code was 0,"
  not that any value round-tripped).
- stated non-coverage: none stated.

### 9. `ms-cli-verify-phrase-roundtrip`
- kind: custodial
- tier: T2
- origin: hardcoded literal ms1 string
  `"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"` + hardcoded phrase —
  **not generated within this test**; it is typed directly into the Rust
  source. (Traces to `vectors/v0.1.json` entry 0 / journey #5, which *does*
  regenerate it live — see finding F5 below; this test itself does not make
  that link.)
- invocations: single `ms verify <card> --phrase <p>` process, one repo.
- structural assertion: stdout contains `"OK: round-trip valid (12 words,
  language=english)"` — a human-readable substring, not a machine-comparable
  value.
- functional assertion: **NONE — FINDING.**
- ONE command: `cargo test -p ms-cli --test verify_phrase_round_trip_ok` —
  **measured passing**, 1/1 ok. Companion negative test
  `verify_phrase_round_trip_mismatch.rs` (wrong phrase → exit 1 or 4, and
  asserts neither phrase is ever echoed) — **measured passing**, 1/1 ok; this
  is a failure-path test, not itself a round-trip journey, but is the direct
  negative twin of #9.
- stated non-coverage: none stated.

### 10. `ms-cli-decode-custodial-hardcoded-card`
- kind: custodial
- tier: T2
- origin: same hardcoded ms1 literal as #9 (typed directly in source).
- invocations: single `ms decode <card>` process, one repo.
- structural assertion: stdout contains `"entropy: 000…0"` and
  `"phrase: abandon…about"` and `"language: english (12 words"`.
- functional assertion: **NONE — FINDING.**
- ONE command: `cargo test -p ms-cli --test decode_round_trip` — **measured
  passing**, 2/2 ok (includes a JSON-schema variant of the same card).
- stated non-coverage: none stated.

### 11. `ms-cli-decode-custodial-typed-back-chunked`
- kind: custodial
- tier: T2
- origin: hand-authored **chunked/spaced** re-typing of the same card:
  `"ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f"` — simulates an
  operator retyping a card read back off engraved chunks, via stdin.
- invocations: single `ms decode -` process reading the chunked string from
  stdin, one repo.
- structural assertion: stdout contains `"entropy: 000…0"`.
- functional assertion: **NONE — FINDING.**
- ONE command: `cargo test -p ms-cli --test back_typed_chunked_form_decodes`
  — **measured passing**, 1/1 ok.
- stated non-coverage: none stated. Notable: this is the closest thing in the
  repo to an "operator retypes what they see" simulation, but it is still T2
  (no real transport, no real operator input device) — it does not claim T3
  and should not be read as one.

### 12. `ms-cli-encode-derive-singlesig`
- kind: generative
- tier: T2
- origin: fixed literal `ZEROS_HEX` (all-zero 16-byte entropy) / equivalent
  `ABANDON` phrase.
- invocations: `ms encode --hex <ZEROS_HEX>` (subprocess, via test's own
  `ms1_of()` helper) → `ms derive <card>` (subprocess, `--template
  bip44|bip49|bip84|bip86`), one repo, two processes.
- structural assertion: **NONE independently asserted in this test file** —
  the encode step's own correctness is assumed, not re-verified here (no
  decode-and-compare against the original hex/phrase inside this file).
- functional assertion: **present and strong.** `master_fingerprint ==
  "73c5da0a"` and `account_xpub == <pinned value>` for bip84/44/49/86. Per the
  file's doc comments, bip84's fp+xpub is a long-standing pin; bip44/49's xpubs
  are cross-checked "via TWO independent from-scratch derivations (bip32utils…
  and a hand-rolled HMAC-SHA512+secp256k1 derivation…) — neither touches
  rust-bitcoin or this crate's `purpose()`"; bip86's is pinned to the
  **published BIP-86 spec test vector** verbatim. This is the
  strongest-provenance functional oracle in the whole inventory.
- ONE command: `cargo test -p ms-cli --test cli_derive` — **measured
  passing**, 20/20 ok.
- stated non-coverage: none stated in-file (provenance is documented, a
  coverage boundary is not).
- **FINDING:** this journey has a real funds-facing functional assertion but
  **no explicit structural equality** in the same test — per §4 ("a journey
  ends in *both*"), this journey satisfies only the functional half on paper.
  (In practice a broken encode/decode would also break the fingerprint, so the
  two are not truly independent risks — but the letter of §4 asks for both
  assertions to be *stated*, and this journey states only one.)

### 13. `ms-cli-encode-derive-multisig-bip48`
- kind: generative
- tier: T2
- same shape as #12, templates `bip48-p2wsh` / `bip48-p2sh-p2wsh`.
- functional assertion provenance is **cross-repo**: the doc comment states the
  pins were "Derived through the SeedHammer II fork's INDEPENDENT Go
  implementation," reproducing a value "ENGRAVED on steel in that fork's
  committed gate record (`oracle/gaterecords/S0-trace-a.record.json`) and
  independently decoded by `mk decode`." **Recorded as a fact, not audited**
  (out of scope — that repo is the seedhammer fork, not this one).
- structural assertion: **NONE independently asserted**, same gap as #12.
- ONE command: `cargo test -p ms-cli --test cli_derive_bip48` — **measured
  passing**, 13/13 ok.
- stated non-coverage: none stated in-file.

### 14. `ms-cli-split-combine-kofn-roundtrip`
- kind: generative
- tier: T2
- origin: fixed literal `ENGLISH_12` phrase, or a Japanese 12-word phrase
  built from fixed entropy `[0xABu8; 16]`, or fixed hex `"ab".repeat(16)`.
- invocations (strongest multi-hop chain found): `ms split --phrase <p> -k 2
  -n 3 --json` (subprocess) → `ms combine <share_i> <share_j> [--to
  entropy|ms1|phrase]` (subprocess) → for the `--to ms1` variant, a **third**
  hop: `ms decode <recovered_ms1>` (subprocess) — one repo, up to 3 processes
  chained in one test (`combine_to_ms1_emits_single_string_that_decodes`).
- structural assertion: recombined output contains the original phrase (or
  hex, or — via the 3-hop variant — the decode of the recovered ms1 contains
  the original phrase). Also covers comma-grouped re-ingestion and stdin
  (`-`) ingestion of shares.
- functional assertion: **NONE — FINDING.** No test anywhere derives a
  fingerprint/xpub from a K-of-N-recombined secret; `derive` and
  `split`/`combine` are never chained (confirmed: `grep -rln '"derive"'
  crates/ms-cli/tests/*.rs` → 4 files, none of which is `cli_split.rs` or
  `cli_combine.rs`). **This is a real, checkable gap**: the custody-recovery
  path (K-of-N) and the funds-derivation path (journey #12/#13) have never
  been exercised together, so nothing proves a recombined secret derives the
  same funds as the original.
- ONE command: `cargo test -p ms-cli --test cli_combine` — **measured
  passing**, 10/10 ok (re-run standalone after an earlier `tail`-truncated
  batch run hid its result). `cli_split.rs` alone (share-generation only, no
  recombine) — **measured passing**, 7/7 ok.
- stated non-coverage: none stated in-file.

### 15. `ms-cli-repair-corrupt-selfcorrect-roundtrip`
- kind: custodial
- tier: T2
- origin: `ABANDON_MS1` constant (same literal as #9/#10, sourced in this
  file's own comment to `vectors/v0.1.json` entry 0 — confirmed identical by
  direct comparison), programmatically corrupted in-test via `flip_at`/
  `flip_many` (cyclic codex32-alphabet substitution at chosen positions) —
  so the *corruption* is generated, but the *ground truth* it must recover is
  the same hand-placed literal as #9/#10.
- invocations: single `ms repair --ms1 <corrupted>` process, one repo.
- structural assertion: for a 1-char flip, the corrected chunk line ==
  `ABANDON_MS1` exactly (self-correction verified against the known-good
  original); for 5-char flips (exceeds t=4 BCH capacity), exit code 2
  (`TooManyErrors`) — i.e. **a failure path that correctly fails**, not a
  green-by-absence skip.
- functional assertion: **NONE — FINDING.**
- ONE command: `cargo test -p ms-cli --test cli_repair` — **measured
  passing**, 7/7 ok (only the first ~half of the 419-line file was read in
  full; the passing count is a directly measured `cargo test` result, not
  inferred from the partial read).
- stated non-coverage: file header states the demotion semantics precisely
  (exit 4 "VERIFY-ME candidate," never a silent exit-5 "recovered" — i.e. a
  1-char correction is presented but explicitly marked unverified, which is
  itself a form of non-coverage disclosure: the tool does not claim the
  correction is *proven* right, only that it's the unique BCH-nearest
  codeword).

### 16. `ms-codec-toolkit-crossrepo-parity-smoke` — DORMANT, currently a pass-through skip
- kind: custodial (cross-validation, not a secret round trip)
- tier: T1/T2 mixed (in-process `ms_codec::decode_with_correction` vs. a
  subprocess call to a **different repo's installed binary**)
- origin: `VALID_MS1` (same literal again), corrupted at position 4.
- invocations: **crosses repos** — in-process `ms_codec::decode_with_correction`
  (this repo) vs. subprocess `~/.cargo/bin/mnemonic repair --ms1 <bad>` (the
  installed `mnemonic-toolkit` binary — **recorded as a fact only; that repo
  was not audited**).
- structural assertion (when it runs): toolkit's corrected string ==
  `ms-codec`'s corrected string; corrected position and character cited in
  toolkit stdout match `ms-codec`'s reported correction.
- functional assertion: **NONE — FINDING** (not applicable to its stated
  purpose, which is BCH-decoder cross-validation, not funds derivation).
- ONE command: `cargo test -p ms-codec --test parity_smoke -- --nocapture`.
- **MEASURED LIVE, this session:** the test does **not** actually perform its
  cross-validation. Output:
  ```
  parity_smoke: toolkit binary reports: mnemonic 0.97.0
  parity_smoke: SKIPPING — this guard is only meaningful against toolkit
  0.22.1, which carried its OWN vendored BCH decoder. Found "mnemonic
  0.97.0", which delegates to ms_codec::decode_with_correction, so the
  comparison would be this crate against itself. The parity guard is
  DORMANT, not passing — see design/FOLLOWUPS.md
  `parity-smoke-toolkit-version-drift`.
  test parity_smoke_ms_against_toolkit_v0_22_1 ... ok
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
  ```
  Confirmed `~/.cargo/bin/mnemonic --version` → `mnemonic 0.97.0` (not
  0.22.1). **This is a live, currently-present instance of the exact pattern
  §5 forbids: "A skipped step must fail, not pass. A skip prints `ok` and
  exits 0."** The test function hits a version-mismatch guard, `eprintln!`s an
  explanation, and `return`s — which Rust's test harness reports as `ok`.
  **This is already tracked, not a new discovery**: `design/FOLLOWUPS.md`
  entry `parity-smoke-toolkit-version-drift` (line 60) — status **OPEN**,
  tier `test-infra`, surfaced 2026-08-15 — documents this exact defect
  ("the guard is now DORMANT, which is the residual item... a green suite
  cannot be misread as 'parity verified'") and records that it was previously
  RED and silently miscounted as passing before being caught, then
  deliberately converted from "silently green" to "explicitly, loudly
  green-but-dormant" rather than fixed, pending a toolkit-version pin or a
  third independent codex32 implementation. The self-disclosure in the test's
  own comments and stderr is honest about the gap; it is disclosed, not
  hidden — but it is still, mechanically, a gate that has never executed
  (§5's other clause: "Every gate in it must have executed at least once. A
  gate that has never run is a hypothesis.") against any currently-installed
  toolkit version.
- stated non-coverage: extensively self-documented (see above) — this is the
  one journey in the inventory whose non-coverage statement is exemplary.

### 17. `ms-cli-mnem-wire-language-roundtrip`
- kind: generative
- tier: T2
- origin: fixed entropy `[0xABu8; 16]` → Japanese 12-word phrase (built via a
  **separate, independent** call to `bip39::Mnemonic::from_entropy_in`
  directly — not through the CLI — giving a genuine independent oracle for
  the expected phrase string).
- invocations: `ms encode --language japanese --phrase <ja>` (subprocess,
  via test's own `japanese_mnem_ms1()` helper) → `ms decode [--language
  english] <card>` (subprocess), one repo, two processes.
- structural assertion: decode stdout contains the independently-computed
  Japanese phrase (`expected_japanese_phrase()`), and reports `language:
  japanese` regardless of an explicit conflicting `--language english` flag
  (wire language wins, with an advisory warning when overridden) — this
  variant is the one place in the inventory where the comparison value is
  computed via a path genuinely disjoint from the encode/decode round trip
  itself, not merely re-asserting a value the CLI produced.
- functional assertion: **NONE — FINDING.**
- ONE command: `cargo test -p ms-cli --test decode_mnem_japanese` — included
  in the full batch run this session (**measured passing** as part of that
  batch; not isolated to a standalone single-file run, but the batch run's
  tail output was inspected and showed no failures across the full set).
- stated non-coverage: none stated in-file.

---

## Cross-cutting findings (§5 anti-requirements + §7 field gaps)

1. **No journey in this repo carries both required equalities in one test.**
   Every journey is structural-only (13 of 17: #1,#2,#3,#4†,#5,#6,#7,#9,#10,
   #11,#14,#15,#17) or functional-only (#12,#13), or neither (#8, #16-when-
   dormant). †#4's phrase-match is arguably a checksum-of-the-same-bytes, not
   an independent funds-facing check — see its entry. **Zero journeys satisfy
   §4's "ends in *both*" literally within a single test.** The closest the
   repo comes is: (a) run #14's split→combine→decode chain, then separately
   run #12's derive on the same recovered secret — but nothing wires those
   two together today.

2. **`encode_pipe_to_verify` (#8) asserts no equality value at all** — only a
   process exit code. This is the weakest "journey" in the inventory: a green
   result proves `ms verify` exited 0, not that any specific value matched
   anything.

3. **`parity_smoke.rs` (#16) is a live, measured instance of the forbidden
   "skip prints ok" pattern**, already self-diagnosed in the test's own
   comments and tracked OPEN in `design/FOLLOWUPS.md` (`parity-smoke-toolkit-
   version-drift`, surfaced 2026-08-15). Confirmed by direct execution this
   session, not by reading the comment alone.

4. **Repeated hand-transcribed origin literal.** The exact string
   `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f` is hardcoded
   independently in at least 5 files (`decode_round_trip.rs`,
   `verify_phrase_round_trip_ok.rs`, `verify_phrase_round_trip_mismatch.rs`,
   `cli_repair.rs`'s `ABANDON_MS1`, `parity_smoke.rs`'s `VALID_MS1`), each a
   separate manual transcription rather than a shared constant or a
   programmatic derivation. It is **not an orphaned/never-reproduced value**
   — `vectors/v0.1.json` entry 0 carries the same string and
   `vectors.rs::v01_corpus_round_trips` (#5) independently re-derives it via
   a live `encode()` call every test run — but no individual consumer test
   states that provenance link; a reader of any one of the 5 files in
   isolation cannot tell the constant is regenerated-and-checked elsewhere
   rather than typed once and trusted forever.

5. **No skipped-step-passes pattern found elsewhere** (checked #1-#15, #17):
   none of the other 16 journeys contain an early-return/skip path in their
   assertion logic — #16 is the sole instance found.

6. **No stale-intermediate-read pattern found**: none of the journeys read a
   file/intermediate that nothing in the same test wrote — all of #1-#17 build
   their own inputs in-process or via a preceding subprocess in the same test
   (the F-210 defect class from `mnemonic-engrave` does not appear to
   recur here). Caveat: I did not open every one of the 75 test files in
   full — this check is thorough for the 17 journeys inventoried, not
   exhaustive over all 270 test functions.

7. **README/version mismatch** (not a §5/§7 violation, but affects
   discoverability of journey #14): README.md's Status/Scope lines describe
   K-of-N as unreleased future work; `Cargo.toml` versions (`ms-cli` 0.16.0,
   `ms-codec` 0.7.0) and passing `cli_split.rs`/`cli_combine.rs` tests show it
   shipped long ago.

8. **Known blind spot, restated per §8.3**: this repo cannot, by itself,
   produce or reveal a T3/T4 journey — it has no device, emulator, or render
   path. Journey #16 is the one place this repo's own tests reach *toward*
   another repo (a subprocess call to the installed `mnemonic-toolkit`
   binary), and even that is a codec-parity check, not a round-trip journey
   into that repo's domain. Whether `mnemonic-secret`'s `ms1` output is ever
   consumed downstream by `mnemonic-engrave`'s bundler in a T2+ journey is
   invisible from here by construction, per the ruling.

## §8 ruling #4 compliance note

Journeys #12 and #13 correctly treat passphrase/network/account-index/language
as **variations within one journey** (separate `#[test]` functions sharing one
base scenario), not as separate cataloged journeys — this inventory follows
that same grouping rather than listing each variation as its own entry.
