# R0 — `IMPLEMENTATION_PLAN_encrypted_payload_hostA.md` (rewrite) @ bc0798d

Reviewer: opus. **The plan was EXECUTED, not read** — every file written out into
a scratch copy of the crate with real deps, all 9 tasks run.
Verdict: **5 Critical / 5 Important / 6 Minor / 4 Nit — GATE BLOCKED.**

## Verified clean (do not re-derive)

- **Zero API mismatches.** Every cited signature exists: `md_codec::reassemble`
  (chunk.rs:311), `codex32::unwrap_string`, `mk_codec::decode`,
  `string_layer::decode_string(...).corrections_applied`, `ms_codec::decode`,
  `pbkdf2::pbkdf2_hmac::<Sha256>`, aes-gcm 0.10, bip39 2.2, `rand::rng()`,
  `usize::div_ceil`.
- **md-codec 0.40 → 0.42: 103/103 existing tests pass.** Claim confirmed.
- **All six vectors verify TWICE independently** — once via the plan's own code,
  once from a from-scratch Python implementation built off §6's offset table.
  Every length, sha256, derived key, tag and all six 52-byte headers match.
  Both §6.6 hashes confirmed.
- `main.rs` facts confirmed (`enum Command` :39, `run() -> i32` :74-82,
  `run_bundle_cli` :164, `write_private` :375); the `if let Some(Command::Seal…)`
  bindings coerce correctly.
- `validate.rs` line numbers correct; the extraction is behaviour-preserving.
- **The 200 ms KDF-gate bound is sound, not flaky**: 47 µs on the real path vs
  **8.19 s** for 2M rounds in a debug build — ~4000× margin.

## The five Criticals

1. **`seals_and_prints_the_passphrase_to_stderr_only` was a FALSE PASS.** The
   12-token heuristic returns the prose header `passphrase — write this down and
   store it APART from the machine:` (exactly 12 tokens), so `first ==
   "passphrase"` and the §2.3 guard degenerated to `!uf2.contains("passphrase")`.
   **Proven by mutation**: copying the real twelve words into the UF2's padding
   left the test green. This is the only test between the ciphertext and the key
   that opens it.
2. **`public_only_payload_prints_no_passphrase` FAILS against its own output** —
   `RECORD THIS WHOLE LINE. The device shows the same value; if it` is also
   exactly 12 tokens.
3. **`record_or_mnemonic` swallowed the `--group-size 0` guidance.** The remedy
   for the round-3 Critical survived only on the *public* path; the secret path
   is the DEFAULT, i.e. exactly where an operator pasting `mnemonic bundle`
   output lands.
4. **`encode_section` trimmed before checking, so a CR survived** — `\r` is
   `char::is_whitespace`, so §6.4's "no CR anywhere" was silently normalised
   away. An implementation defect, not a test defect.
5. **The freshness test did not compile** — `*a.passphrase` on
   `Option<Zeroizing<String>>`, blocking all six vectors behind a one-line typo.

## The five Importants

- `use record::RecordKind;` unused → `-D warnings` fails the build.
- `out: &PathBuf` trips `clippy::ptr_arg` → `-D warnings` fails.
- **No vector was ever DECRYPTED.** §11.1's "round-trip seal/open" had no task;
  blob sha256s prove the bytes are STABLE, not PARSEABLE, and vector B's purpose
  (catching a hardcoded count *on decrypt*) was half-realised.
- **§11.4's two mandated AAD negatives were absent.** Task 2's
  `open_fails_on_tampered_aad` swaps `b"aad-one"`/`b"aad-two"`, which proves the
  field is wired up — not that AAD is `header ‖ public section`. An
  implementation setting `aad = header` alone passed every test.
- **Three §6.2 bounds had zero coverage** — `reserved`, and sealed-shape
  `kdf_id`/`aead_id`. Mutation-proved: deleting all three left the wire suite
  green.

## Minors / Nits
pubhash's byte-flip test mutated record indices, not section endpoints ·
Task 4's `--lib validate` filter runs 4 tests, none covering the branch it
rewires · `--seal-secret` is not in the spec (safer than spec, but a divergence)
· `me hash` accepted an `ms1` and skipped the card-set decode · secret records
never zeroized · the combined 1..24 cap untested · out-of-range `--iterations`
silently ignored on the public-only path · `me hash` shape-flag validation
untested · `--all-targets` clippy lints on test code · a ~1-in-30,000 flake in
the §2.3 containment check when the first BIP-39 word is 3 letters.

## Controller fold note (2026-08-07)

All 20 folded. The passphrase-detection defect is the root of two Criticals and
is fixed once, with a `passphrase_line` helper that requires all-lowercase ASCII
tokens so no prose line can match — and the containment assertion now uses the
LONGEST word, closing the flake. The CR fix is in the implementation, per the
reviewer's explicit instruction not to weaken the assertion. Expected counts
reconciled: Task 7 now states 52 for `--lib seal`.
