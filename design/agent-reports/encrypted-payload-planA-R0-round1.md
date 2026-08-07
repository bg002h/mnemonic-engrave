# R0 round 1 — `IMPLEMENTATION_PLAN_encrypted_payload_hostA.md`

Reviewer: opus, plan-gate review. Dispatched 2026-08-07.
Verdict: **2 Critical / 8 Important / 9 Minor / 5 Nit — GATE BLOCKED.**

API verification and vector transcription both came back CLEAN — the reviewer
built a scratch crate with the plan's verbatim `Cargo.toml` stanza, `crypto.rs`
and `passphrase.rs`, compiled it against the real crates, and reproduced all
three vectors byte-exactly. `rand::rng()` (not `thread_rng()`) confirmed correct
for rand 0.9; `ms_codec::decode` confirmed at `ms-codec-0.7.0/src/lib.rs:55`.

## Findings

### [CRITICAL] `ms1` has no explicit opt-in flag — sealing a seed is silent
§9 requires "`ms1` → `0x03` (admitted per §12 item 6, **kept behind an explicit opt-in flag so that sealing a seed is never accidental**)". The plan had no such flag and routed `Ok(Format::Ms) => Payload::Ms1(s)` silently. It was also absent from "What Plan A does NOT cover", so it was a dropped requirement rather than a deferral.
**Failure scenario:** operator runs `me seal ms10entrs…` for a dry run, or pastes the wrong line of a bundle file, and puts offline-attackable seed ciphertext on the machine with no confirmation.
**Fix applied:** `--seal-secret` flag; `ms1` refused without it whether standalone or as a bundle record, checked on *classification* rather than on a caller-supplied claim; exits `EXIT_REFUSED`; tests `refuses_ms1_without_the_opt_in_flag` (asserts nothing is written) and `seals_ms1_with_the_opt_in_flag` (asserts `blob[11] == 0x03`).

### [CRITICAL] `Payload::Bip39` (kind `0x02`) was never validated
§9 requires a checksum-valid mnemonic; §11.1 requires "a mislabelled input is refused at seal time, not emitted." The plan performed zero validation on the `0x02` path — no word count, no checksum, no normalisation — while its own comment claimed `seal() validates it downstream`.
**Failure scenario:** `me seal "hunter2 please"` emits a well-formed 0x02 blob; the operator loads it, types twelve words, waits out the KDF, and gets "payload unreadable" — which §2.2 item 4 has taught them to read as tampering. A typo'd word in a real 24-word seed reaches the plate through the same hole.
**Fix applied:** `bip39::Mnemonic::parse_in` gate (accepts all five legal lengths), normalise-before-seal, new `SealError::Payload`, plus `payload_kind_byte_matches_the_input_shape` and `refuses_a_checksum_invalid_bip39_payload`.

### [IMPORTANT] Task 8's handler could not compile
Three mismatches against the real `main.rs`: the enum is `Command` (line 39) not `Commands`; `fn run() -> i32` (line 71) has **no `match`** — dispatch is a series of early returns; and `?` in a function returning `i32` is a hard error.
**Fix applied:** rewritten as `fn run_seal_cli(...) -> i32` mirroring `run_bundle_cli` (`main.rs:164`), with an early return after the existing `Command::Bundle` block, and the crate's `EXIT_*` contract instead of `?`.

### [IMPORTANT] `iterations` never bound-checked on the seal side
`Header::decode` enforces the range; `encode` and `seal_deterministic` did not. `--iterations 5` would emit a blob the device provably rejects; `--iterations 3000000000` would burn hours on the laptop first.
**Fix applied:** range check in `seal_deterministic` before the KDF, new `SealError::Iterations`, boundary tests at 99_999 / 100_000 / 2_000_000 / 2_000_001.

### [IMPORTANT] Defaulting the UF2 to stdout wrote the passphrase into the file
The plan made `--out` optional and sent the binary UF2 to stdout, then `println!`ed the passphrase to the **same stream**.
**Failure scenario:** `me seal md1… > payload.uf2` — the universal idiom — writes the 12-word passphrase into the file, at the shell's umask, beside the ciphertext it opens. §2.3: "The passphrase must never be stored with the machine. The entire security argument collapses if both artefacts sit in the same place."
**Fix applied:** `--out` is `required = true`; the passphrase goes to **stderr**; a test asserts no passphrase word appears in the output file's bytes.

### [IMPORTANT] The RED step reported 0 tests and exit 0 in five of eight tasks
Each task created the new `.rs` file in one step but added `pub mod …;` only in the *implementation* step. An undeclared file is not compiled, so the RED command matched nothing. Verified by execution: `running 0 tests … ok. 0 passed; EXIT=0`.
**Failure scenario:** the implementer sees `ok`, records the phase verified, and never establishes a failing test — this project's documented false-PASS class applied to the TDD gate itself.
**Fix applied:** module wiring moved into the same step that creates the test file, in all five tasks, with the reason stated inline.

### [IMPORTANT] `refuses_a_user_supplied_passphrase` / `has_no_addr_flag` asserted nothing
Both asserted only `.failure()`. Clap exits non-zero for an unknown argument **and** for a known argument whose value fails to parse — the tests could not distinguish them. Adding `#[arg(long)] passphrase: Option<String>` with checksum validation would keep the test green while breaking §8's prohibition.
**Fix applied:** both now assert `stderr` contains "unexpected argument" **and** that `--help` does not list the flag.

### [IMPORTANT] Mutation row 6 had no possible killer
`decode` returns at the `ct_len > 8191` check before reaching the region-fit arithmetic. For 32-bit `48 + ct_len + 16` to wrap, `ct_len` must be ≥ `0xFFFF_FFC0` — already rejected. **No input can reach the region check**, so no Rust test can observe whether it is 32- or 64-bit.
**Fix applied:** row dropped; the test renamed away; a code comment records that §6.2's overflow warning targets the **Go port** (32-bit `int`), which is why the `0xFFFF_FFF0` requirement lives in §11.2 and not §11.1.

### [IMPORTANT] `encode_bundle` validated a trimmed record and encoded the untrimmed one
`validate_record` trims internally; `encode_bundle` joined the originals. A trailing space survives to the device, where `codex32.inputChar` has no mapping for `0x20`. Structurally identical to the round-3 Critical and to the pre-existing A3/F4 finding.
**Fix applied:** trim once at the boundary, validate and encode the same `trimmed` vector; test `surrounding_whitespace_does_not_change_the_encoding`.

### [IMPORTANT] No test asserted `payload_kind`; `MdMk` and `Ms1` were never exercised
Vectors A and C pin `0x02`/`0x04` only transitively through sha256s. Swapping the `MdMk` and `Ms1` discriminants left the whole suite green — and would surface only on hardware, after the Go port had bound to the wrong Rust behaviour.
**Fix applied:** table-driven `payload_kind_byte_matches_the_input_shape` asserting `blob[11]` for all four kinds.

## Minors and Nits (all folded)
Task 5 expected count 10→9 · Task 2 Interfaces listed unused imports (`-D warnings` build failure) · `SealError::TooLarge` fired for an empty payload (split out `Empty`) · `kdf_id`/`aead_id`/`TooShort` had no test · `encoding_is_deterministic` could not fail (replaced with the whitespace-invariance test) · exit codes ignored the crate's `EXIT_*` contract · §11.4's passphrase-before-KDF ordering had no seal-level test (added, timing-asserted) · secrets lived un-zeroized in clap's `Vec<String>` (moved into `Zeroizing` at the handler boundary) · UF2 per-block fields checked on block 0 only (now checked on every block, with a short final block) · extraction placement would orphan `validate`'s doc comment · `assert_eq!(…, true)` → `assert!` · `to_uf2(&[])` inconsistency → `debug_assert!` · `pbkdf2`'s `sha2` feature redundant → `features = ["hmac"]` · Task 4 Interfaces listed `first_noncanonical` as both consumed and produced.

## VERDICT
Critical: 2   Important: 8   Minor: 9   Nit: 5
GATE: BLOCKED

CONFIDENCE: API existence, feature resolution, all `Zeroizing` deref forms, every checksum outcome, all three derived keys, all vector lengths and sha256s, and the `0 passed / EXIT=0` RED-step behaviour were verified by **executing real crate source**. Repo citations (`validate.rs:63-91`/`:74-77`, `main.rs:39`/`:62-65`/`:71`/`:375`, `lib.rs:8-9`, MSRV 1.85.0) verified by reading. Clap's unknown-flag exit behaviour and the region-check unreachability proof were reasoned, not executed.

## Controller note (2026-08-07)
All 24 findings folded. `main.rs`'s real dispatch shape was read directly
(`sed -n '71,100p'`, `'164,178p'`) before rewriting Task 8 — the original handler
had been written from memory rather than from the file, which is what produced
the compile-breaking Important.

---

# R0 round 2 — fold verification

Reviewer: sonnet, scoped fold check. Dispatched 2026-08-07.
Verdict: **0 Critical / 0 Important / 2 Minor / 0 Nit — GATE PASS.**

All 10 round-1 Critical/Important fixes verified landed. Task 8's handler traced
against the real `main.rs` and confirmed to compile: `Command` (singular) matches;
match ergonomics bind `payload: &Vec<String>` which coerces to `&[String]` via
the same deref pattern already live in that file (`write_private(path, &bytes)`);
`EXIT_*`, `write_private`, `Zeroizing`, `PathBuf` all in scope; no `?` remains.
All five modules confirmed declared exactly once, in the step creating their file.

The 200 ms bound in `refuses_an_invalid_passphrase_without_running_the_kdf` was
judged sound and low-flake: the checksum gate precedes any KDF call in code
order, and 2,000,000 PBKDF2 rounds cannot finish in 200 ms even in a debug build.

## Minor 1 — `payload_kind_byte_matches_the_input_shape` reused one (salt, iv)
Four distinct plaintexts sealed under one derived key and one GCM nonce — the
same defect class that broke the first draft of vector C, and inconsistent with
§11.1's own pair-uniqueness assertion. Not exploitable: the blobs are asserted on
and dropped inside the test, never persisted or transmitted. AAD differing does
not mitigate it; GCM's forbidden attack depends on (key, nonce) alone.
**Folded:** salt and IV now vary per case (`^ i`), with the reason in a comment.

## Minor 2 — stale expected test count
Task 8 Step 4 said "Expected: 5 passed"; the fold added four CLI tests, making 9.
**Folded:** corrected to 9.

## VERDICT
Critical: 0   Important: 0   Minor: 2   Nit: 0
GATE: **PASS**

Both Minors folded inline; being a count fix and a test-hygiene fix to an
already-passing gate, they do not re-trigger a round.
